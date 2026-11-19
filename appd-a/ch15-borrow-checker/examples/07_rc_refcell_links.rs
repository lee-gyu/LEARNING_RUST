//! 지침 6: 서로 얽힌 데이터에만 `Rc<RefCell<T>>`를 쓰고, 소유하지 않는 연결은 `Weak`로 둔다.
//!
//! 05·06의 채팅 서비스를 스마트 포인터로 만든 판입니다. 사용자 명부가 `Rc<RefCell<User>>`를
//! 소유하고, 방은 사용자를 가리킵니다.
//! 1~2번 구역: 방이 `Rc`를 담으면 탈퇴한 사용자가 방에 남고(비권장), `Weak`를 담으면 사라집니다(권장).
//! 3번 구역: 대여 규칙 위반이 컴파일 에러가 아니라 실행 중 실패가 되는 모습을
//!          `try_borrow_mut`으로 관찰합니다(패닉시키지 않음).
//! 4번 구역: ch9가 남긴 "두 클로저가 같은 상태를 고침(E0499)"을 `RefCell`로 덮는 판(비권장)과
//!          상태 대신 데이터를 돌려주게 구조를 바꾼 판(권장)입니다.
//!
//! 실행: cargo run -p er-ch15-borrow-checker --example 07_rc_refcell_links

use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct User {
    nickname: String,
    unread: u32,
}

type Shared = Rc<RefCell<User>>;

fn user(nickname: &str) -> Shared {
    Rc::new(RefCell::new(User {
        nickname: nickname.to_string(),
        unread: 0,
    }))
}

/// 비권장: 방도 사용자를 소유합니다.
struct RcRoom {
    title: &'static str,
    members: Vec<Shared>,
}

/// 권장: 방은 가리키기만 합니다. 사용자의 수명은 명부가 정합니다.
struct WeakRoom {
    title: &'static str,
    members: Vec<Weak<RefCell<User>>>,
}

impl WeakRoom {
    fn line(&self) -> String {
        let names: Vec<String> = self
            .members
            .iter()
            .map(|w| match w.upgrade() {
                Some(u) => u.borrow().nickname.clone(),
                None => "(탈퇴한 사용자)".to_string(),
            })
            .collect();
        format!("{}: {}", self.title, names.join(", "))
    }
}

/// 새 메시지 알림 처리기. 사용자를 고쳐야 하므로 가변으로 빌립니다.
/// 실제 코드라면 `borrow_mut()`을 썼을 자리이고, 그랬다면 실패가 곧 패닉입니다.
fn notify(member: &Shared) -> Result<u32, std::cell::BorrowMutError> {
    let mut u = member.try_borrow_mut()?;
    u.unread += 1;
    Ok(u.unread)
}

fn directory() -> Vec<Shared> {
    vec![user("하늘"), user("바다"), user("산")] // 0, 1, 2
}

fn rc_version() {
    println!("[비권장] 방이 Rc를 담음");
    let mut dir = directory();
    let rooms = [
        RcRoom {
            title: "등산 모임",
            members: vec![Rc::clone(&dir[0]), Rc::clone(&dir[2])],
        },
        RcRoom {
            title: "독서 모임",
            members: vec![Rc::clone(&dir[1]), Rc::clone(&dir[0])],
        },
    ];
    // 닉네임은 명부에서 한 번만 고치면 됩니다. 사본이 없으므로 모든 방에 보입니다.
    dir[2].borrow_mut().nickname = String::from("산들바람");
    // 탈퇴: 명부에서 뺍니다. 방은 번호가 아니라 포인터를 담으므로 어긋날 번호도 없습니다.
    let sea = Rc::downgrade(&dir.remove(1));
    println!("  닉네임 변경과 바다 탈퇴 뒤");
    for room in &rooms {
        let names: Vec<String> = room
            .members
            .iter()
            .map(|u| u.borrow().nickname.clone())
            .collect();
        println!("    {}: {}", room.title, names.join(", "));
    }
    println!(
        "    명부에서 뺀 바다를 붙잡은 강한 참조: {}개 (독서 모임의 Rc)",
        sea.strong_count()
    );
    // 출력:     등산 모임: 하늘, 산들바람
    // 출력:     독서 모임: 바다, 하늘     ← 탈퇴한 사용자가 방에 남음
    // 출력:     명부에서 뺀 바다를 붙잡은 강한 참조: 1개 (독서 모임의 Rc)
}

fn weak_version() -> (Vec<Shared>, Vec<WeakRoom>) {
    println!("[권장] 방이 Weak를 담음");
    let mut dir = directory();
    let rooms = vec![
        WeakRoom {
            title: "등산 모임",
            members: vec![Rc::downgrade(&dir[0]), Rc::downgrade(&dir[2])],
        },
        WeakRoom {
            title: "독서 모임",
            members: vec![Rc::downgrade(&dir[1]), Rc::downgrade(&dir[0])],
        },
    ];
    dir[2].borrow_mut().nickname = String::from("산들바람");
    let sea = Rc::downgrade(&dir.remove(1)); // 마지막 강한 참조가 사라져 해제됨
    println!("  닉네임 변경과 바다 탈퇴 뒤");
    for room in &rooms {
        println!("    {}", room.line());
    }
    println!("    바다의 강한 참조: {}개", sea.strong_count());
    // 출력:     등산 모임: 하늘, 산들바람
    // 출력:     독서 모임: (탈퇴한 사용자), 하늘
    // 출력:     바다의 강한 참조: 0개
    (dir, rooms) // 명부가 사용자를 소유하므로 함께 돌려줍니다.
}

fn main() {
    rc_version();
    let (_dir, rooms) = weak_version();

    println!("[실행 중 대여 검사] 이름을 보여 주는 동안 알림 처리기가 같은 사용자를 고침");
    let hiking = &rooms[0];
    println!("  Ref를 쥔 채 알림");
    for member in hiking.members.iter().filter_map(Weak::upgrade) {
        let shown = member.borrow(); // 화면에 그리는 동안 빌림
        match notify(&member) {
            Ok(n) => println!("    {}: 안 읽은 메시지 {n}", shown.nickname),
            Err(e) => println!("    {}: 알림 실패 ({e})", shown.nickname),
        }
    }
    // 출력:     하늘: 알림 실패 (RefCell already borrowed)
    // 출력:     산들바람: 알림 실패 (RefCell already borrowed)
    println!("  필요한 값만 꺼내 대여를 끝낸 뒤 알림");
    for member in hiking.members.iter().filter_map(Weak::upgrade) {
        let name = member.borrow().nickname.clone(); // Ref는 이 문장에서 버려짐
        match notify(&member) {
            Ok(n) => println!("    {name}: 안 읽은 메시지 {n}"),
            Err(e) => println!("    {name}: 알림 실패 ({e})"),
        }
    }
    // 출력:     하늘: 안 읽은 메시지 1
    // 출력:     산들바람: 안 읽은 메시지 1

    two_closures();
}

/// ch9의 4.3절: `filter`와 `filter_map`의 클로저가 둘 다 `notes`를 고치면 E0499입니다.
fn two_closures() {
    let lines = ["/join 등산", "", "안녕하세요", "/leave 독서", "/join"];

    println!("[비권장] 두 클로저가 RefCell로 같은 기록을 고침");
    let notes = RefCell::new(Vec::new());
    let commands: Vec<&str> = lines
        .iter()
        .filter(|line| {
            if line.is_empty() {
                notes.borrow_mut().push("빈 줄".to_string()); // 실행 중 대여 검사
                false
            } else {
                true
            }
        })
        .filter_map(|line| match line.strip_prefix('/') {
            Some(cmd) if cmd.contains(' ') => Some(cmd),
            _ => {
                notes.borrow_mut().push(format!("명령 아님: {line}")); // 실행 중 대여 검사
                None
            }
        })
        .collect();
    println!("  명령 {commands:?}");
    println!("  기록 {:?}", notes.borrow());
    // 출력:   명령 ["join 등산", "leave 독서"]
    // 출력:   기록 ["빈 줄", "명령 아님: 안녕하세요", "명령 아님: /join"]

    println!("[권장] 줄마다 결과를 데이터로 돌려주고, 고치는 일은 한 곳에서");
    let (ok, bad): (Vec<_>, Vec<_>) = lines
        .iter()
        .map(|line| classify(line))
        .partition(Result::is_ok);
    let commands: Vec<&str> = ok.into_iter().flatten().collect();
    let notes: Vec<String> = bad.into_iter().filter_map(Result::err).collect();
    println!("  명령 {commands:?}");
    println!("  기록 {notes:?}");
    // 출력: (비권장 판과 같음)
}

fn classify(line: &str) -> Result<&str, String> {
    if line.is_empty() {
        return Err("빈 줄".to_string());
    }
    match line.strip_prefix('/') {
        Some(cmd) if cmd.contains(' ') => Ok(cmd),
        _ => Err(format!("명령 아님: {line}")),
    }
}
