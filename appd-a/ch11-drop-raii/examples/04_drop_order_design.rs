// 지침 4(책에 없는 지침): drop 순서를 가드 설계에 씁니다.
//
// 공유 문서를 고칠 때는 편집 잠금을 얻고, 고친 줄을 초안에 모았다가, 초안이 버려질 때 문서에
// 반영합니다. 초안은 잠금이 살아 있는 동안 반영돼야 합니다. 잠금이 먼저 풀리면 그 틈에 다른
// 편집자가 끼어들 수 있습니다. 두 가드의 순서는 컴파일러가 확인해 주지 않습니다.
//
//   1. 지역 변수는 선언의 역순으로 버려집니다 → 잠금을 먼저 얻고 초안을 나중에 만듭니다.
//   2. 구조체 필드는 선언 순서대로 버려집니다 → 먼저 정리돼야 하는 초안 필드를 위에 둡니다.
//   3. 임시값은 그 문장이 끝날 때 버려집니다 → 변수에 담지 않은 잠금은 그 문장에서 풀립니다.
//
// 실행: cargo run -p er-ch11-drop-raii --example 04_drop_order_design

use std::cell::{Cell, RefCell};

struct Doc {
    lines: RefCell<Vec<String>>,
    locked_by: Cell<Option<&'static str>>,
}

impl Doc {
    fn new() -> Doc {
        Doc {
            lines: RefCell::new(Vec::new()),
            locked_by: Cell::new(None),
        }
    }

    fn lock(&self, who: &'static str) -> EditLock<'_> {
        self.locked_by.set(Some(who));
        EditLock { doc: self, who }
    }
}

/// 편집 잠금 가드. 버려질 때 잠금을 풉니다.
#[must_use = "버리면 잠금이 바로 풀립니다"]
struct EditLock<'a> {
    doc: &'a Doc,
    who: &'static str,
}

impl EditLock<'_> {
    fn holder(&self) -> &'static str {
        self.who
    }
}

impl Drop for EditLock<'_> {
    fn drop(&mut self) {
        self.doc.locked_by.set(None);
        println!("      [drop] 편집 잠금 해제 ({})", self.who);
    }
}

/// 고친 줄을 모으는 초안. 버려질 때 문서에 반영하고, 그 순간 잠금이 있었는지 출력합니다.
struct Draft<'a> {
    doc: &'a Doc,
    pending: Vec<&'static str>,
}

impl<'a> Draft<'a> {
    fn new(doc: &'a Doc) -> Draft<'a> {
        Draft {
            doc,
            pending: Vec::new(),
        }
    }

    fn write(&mut self, line: &'static str) {
        self.pending.push(line);
    }
}

impl Drop for Draft<'_> {
    fn drop(&mut self) {
        let when = match self.doc.locked_by.get() {
            Some(who) => format!("{who}의 잠금 안에서"),
            None => String::from("잠금이 풀린 뒤 → 다른 편집자가 끼어들 수 있음"),
        };
        let mut lines = self.doc.lines.borrow_mut();
        lines.extend(self.pending.iter().map(|s| s.to_string()));
        println!("      [drop] 초안 {}줄 반영: {when}", self.pending.len());
    }
}

/// 비권장: 잠금을 먼저 선언했으므로 잠금이 먼저 풀립니다.
struct SessionLockFirst<'a> {
    lock: EditLock<'a>,
    draft: Draft<'a>,
}

/// 권장: 필드는 선언 순서대로 버려집니다. 초안이 잠금보다 먼저 정리돼야 하므로
/// 이 두 필드의 순서를 바꾸지 마세요.
struct Session<'a> {
    draft: Draft<'a>,
    lock: EditLock<'a>,
}

fn main() {
    let doc = Doc::new();

    println!("1. 지역 변수는 선언의 역순으로 버려집니다");
    println!("    [비권장] 초안을 먼저 만들고 잠금을 나중에 얻음");
    {
        let mut draft = Draft::new(&doc);
        let _lock = doc.lock("민지");
        draft.write("1안: 표지 제목 수정");
    }
    // 출력:       [drop] 편집 잠금 해제 (민지)
    // 출력:       [drop] 초안 1줄 반영: 잠금이 풀린 뒤 → 다른 편집자가 끼어들 수 있음
    println!("    [권장] 잠금을 먼저 얻고 초안을 나중에 만듦");
    {
        let _lock = doc.lock("민지");
        let mut draft = Draft::new(&doc);
        draft.write("2안: 목차 정리");
    }
    // 출력:       [drop] 초안 1줄 반영: 민지의 잠금 안에서
    // 출력:       [drop] 편집 잠금 해제 (민지)
    println!("    [권장] 선언 순서를 바꿀 수 없다면 drop으로 순서를 직접 정함");
    {
        let mut draft = Draft::new(&doc);
        let lock = doc.lock("민지");
        draft.write("3안: 그림 번호 수정");
        drop(draft);
        drop(lock);
    }
    // 출력:       [drop] 초안 1줄 반영: 민지의 잠금 안에서
    // 출력:       [drop] 편집 잠금 해제 (민지)

    println!("2. 구조체 필드는 선언 순서대로 버려집니다 (만든 순서와는 상관없음)");
    println!("    [비권장] lock 필드를 위에 선언");
    {
        let mut session = SessionLockFirst {
            lock: doc.lock("서준"),
            draft: Draft::new(&doc),
        };
        session.draft.write("4안: 참고 문헌 추가");
        println!("      잠금 보유자: {}", session.lock.holder());
    }
    // 출력:       잠금 보유자: 서준
    // 출력:       [drop] 편집 잠금 해제 (서준)
    // 출력:       [drop] 초안 1줄 반영: 잠금이 풀린 뒤 → 다른 편집자가 끼어들 수 있음
    println!("    [권장] draft 필드를 위에 선언");
    {
        let mut session = Session {
            lock: doc.lock("서준"), // 잠금을 먼저 얻는 것은 위와 같습니다.
            draft: Draft::new(&doc),
        };
        session.draft.write("5안: 맺음말 추가");
        println!("      잠금 보유자: {}", session.lock.holder());
    }
    // 출력:       잠금 보유자: 서준
    // 출력:       [drop] 초안 1줄 반영: 서준의 잠금 안에서
    // 출력:       [drop] 편집 잠금 해제 (서준)

    println!("3. 임시값은 그 문장이 끝날 때 버려집니다");
    let holder = doc.lock("하린").holder(); // 잠금 가드는 이 문장 끝에서 버려집니다.
    // 출력:       [drop] 편집 잠금 해제 (하린)
    println!(
        "    잠금을 얻은 사람 {holder}, 다음 문장에서 본 잠금 상태 {:?}",
        doc.locked_by.get()
    );
    // 출력:     잠금을 얻은 사람 하린, 다음 문장에서 본 잠금 상태 None

    println!("문서에 반영된 줄: {:?}", doc.lines.borrow());
    // 출력: 문서에 반영된 줄: ["1안: 표지 제목 수정", "2안: 목차 정리", "3안: 그림 번호 수정", "4안: 참고 문헌 추가", "5안: 맺음말 추가"]
}
