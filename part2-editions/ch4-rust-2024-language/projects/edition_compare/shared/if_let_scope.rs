//! 같은 코드를 2021 에디션과 2024 에디션으로 컴파일해서, `if let` 조건식에서 만든
//! 임시값이 언제 버려지는지 비교합니다.
//!
//! 실행(이 프로젝트 디렉터리에서):
//!   cargo run -q --bin if_let_scope_2021
//!   cargo run -q --bin if_let_scope_2024
//!
//! 이 파일 하나를 e2021 패키지와 e2024 패키지가 함께 씁니다(각 Cargo.toml의 [[bin]] path).

use std::cell::RefCell;
use std::sync::RwLock;

/// 버려질 때 자기 이름을 출력하는 값입니다. 임시값이 버려지는 순간을 눈으로 보려고 씁니다.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.0);
    }
}

impl Noisy {
    /// `hit`이 참이면 `Some(7)`, 거짓이면 `None`을 돌려줍니다.
    fn find(&self, hit: bool) -> Option<u32> {
        hit.then_some(7)
    }
}

fn main() {
    println!("1. 패턴이 맞지 않아 else로 갈 때");
    // `Noisy(..)`는 조건식(scrutinee)에서 만든 임시값입니다.
    if let Some(n) = Noisy("조건식의 임시값").find(false) {
        println!("    then 블록: {n}");
    } else {
        println!("    else 블록");
    }
    println!("    if let 다음 문장");
    // 출력(2021): else 블록 → [drop] 조건식의 임시값 → if let 다음 문장
    // 출력(2024): [drop] 조건식의 임시값 → else 블록 → if let 다음 문장

    println!("2. 패턴이 맞아 then으로 갈 때 (두 에디션이 같음)");
    // then 블록에서는 패턴이 묶은 값이 임시값을 빌리고 있을 수 있으므로,
    // 두 에디션 모두 then 블록이 끝난 뒤에 버립니다.
    if let Some(n) = Noisy("조건식의 임시값").find(true) {
        println!("    then 블록: {n}");
    } else {
        println!("    else 블록");
    }
    // 출력(두 에디션): then 블록: 7 → [drop] 조건식의 임시값

    println!("3. match (두 에디션이 같음)");
    // match의 조건식에서 만든 임시값은 어느 에디션에서든 match 전체가 끝날 때까지 삽니다.
    match Noisy("조건식의 임시값").find(false) {
        Some(n) => println!("    Some 갈래: {n}"),
        None => println!("    None 갈래"),
    }
    // 출력(두 에디션): None 갈래 → [drop] 조건식의 임시값

    println!("4. RefCell: 비어 있으면 else에서 채우기");
    let cache: RefCell<Vec<u32>> = RefCell::new(Vec::new());
    if let Some(first) = cache.borrow().first() {
        println!("    캐시에 있음: {first}");
    } else {
        // 조건식의 `cache.borrow()`가 만든 `Ref` 대여 가드가 아직 살아 있다면
        // `borrow_mut()`은 패닉합니다. 패닉 대신 `try_borrow_mut()`으로 관찰합니다.
        match cache.try_borrow_mut() {
            Ok(mut items) => {
                items.push(1);
                println!("    else에서 가변 대여 성공: 캐시에 1을 넣었습니다");
            }
            Err(_) => println!("    else에서 가변 대여 실패: 조건식의 Ref가 아직 살아 있습니다"),
        }
    }
    println!("    캐시: {:?}", cache.borrow());
    // 출력(2021): 가변 대여 실패, 캐시: []
    // 출력(2024): 가변 대여 성공, 캐시: [1]

    println!("5. RwLock: 값이 없으면 else에서 쓰기");
    let flag: RwLock<Option<bool>> = RwLock::new(None);
    if let Some(on) = *flag.read().unwrap() {
        println!("    값: {on}");
    } else {
        // 조건식의 읽기 잠금이 아직 살아 있는데 `write()`를 부르면, 자기가 쥔 읽기 잠금이
        // 풀리기를 기다리며 멈추거나(교착 상태) 패닉할 수 있습니다(`write` 문서의 Panics 절).
        // 그런 일이 생기지 않도록 기다리지 않는 `try_write()`로 관찰만 합니다.
        match flag.try_write() {
            Ok(mut slot) => {
                *slot = Some(true);
                println!("    else에서 쓰기 잠금 성공");
            }
            Err(e) => println!("    else에서 쓰기 잠금 실패: {e}"),
        }
    }
    println!("    값: {:?}", *flag.read().unwrap());
    // 출력(2021): else에서 쓰기 잠금 실패: try_lock failed because the operation would block → 값: None
    // 출력(2024): else에서 쓰기 잠금 성공 → 값: Some(true)
    // 2021에서는 조건식의 읽기 잠금이 else 블록에서도 아직 잡혀 있어서 실패합니다.
}
