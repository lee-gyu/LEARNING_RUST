//! 라이프타임의 범위: 값은 drop되거나 이동될 때까지, 참조는 마지막 사용까지, 임시값은 문장 끝까지 삽니다.
//!
//! 실행: cargo run -p er-ch14-lifetimes --example 02_lifetime_spans
//!
//! 대여 검사기가 비교하는 것은 "참조가 쓰이는 구간"이 "가리키는 값이 살아 있는 구간" 안에 들어가는가
//! 하나뿐입니다. 이 예제는 세 가지 구간이 각각 어디서 끝나는지 출력으로 확인합니다.

/// drop될 때 이름을 출력하는 관찰용 타입입니다.
struct Tracked(&'static str);

impl Tracked {
    fn label(&self) -> &str {
        self.0
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        println!("    [drop] {}", self.0);
    }
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

fn main() {
    println!("1. 참조는 마지막 사용에서 끝납니다");
    let mut draft = String::from("장보기\n두부, 대파");
    let title = first_line(&draft); // draft를 빌리는 참조가 시작됩니다.
    println!("    제목: {title}"); // title의 마지막 사용. 대여가 여기서 끝납니다.
    // title이라는 이름은 main 끝까지 남아 있지만 대여는 이미 끝났으므로 draft를 고칠 수 있습니다.
    draft.push_str(", 계란");
    println!("    고친 초안: {:?}", draft);
    // 출력:
    //     제목: 장보기
    //     고친 초안: "장보기\n두부, 대파, 계란"

    println!("2. 값의 라이프타임은 이동에서도 끝납니다");
    let mut archive: Vec<String> = Vec::new();
    let title = first_line(&draft);
    println!("    보관하기 전 제목: {title}"); // 이것이 마지막 사용이면 아래의 이동이 허용됩니다.
    archive.push(draft); // draft의 값은 archive로 옮겨 가고, draft라는 이름의 라이프타임은 여기서 끝납니다.
    // 여기서 title을 다시 쓰면 E0505입니다(compile_fail/e0505_move_while_borrowed.rs).
    // String을 옮겨도 힙의 글자는 그대로이지만, 대여 검사기는 메모리가 아니라 소유자를 추적합니다.
    println!("    보관함의 첫 줄: {}", first_line(&archive[0]));
    // 출력:
    //     보관하기 전 제목: 장보기
    //     보관함의 첫 줄: 장보기

    println!("3. 임시값은 문장이 끝날 때 drop됩니다");
    println!("    문장 안에서 쓰기: {}", Tracked("임시 쪽지").label());
    println!("    다음 문장");
    // 출력:
    //     문장 안에서 쓰기: 임시 쪽지
    //     [drop] 임시 쪽지
    //     다음 문장

    println!("4. let으로 바로 묶은 임시값은 블록 끝까지 삽니다(임시값 수명 연장)");
    {
        let kept = &Tracked("묶어 둔 쪽지");
        println!("    블록 안: {}", kept.label());
        println!("    블록의 마지막 문장");
    }
    // 출력:
    //     블록 안: 묶어 둔 쪽지
    //     블록의 마지막 문장
    //     [drop] 묶어 둔 쪽지
}
