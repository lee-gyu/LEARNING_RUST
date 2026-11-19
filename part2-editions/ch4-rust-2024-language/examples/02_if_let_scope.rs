// 2024 에디션에서 `if let` 조건식의 임시값이 else 블록에 들어가기 전에 버려지는 것과,
// 그 덕분에 되는 코드, 그리고 `match`는 왜 그대로인지를 보여 줍니다.
//
// 실행: cargo run -p p2-ch4-rust-2024-language --example 02_if_let_scope
//
// 같은 코드를 2021 에디션으로 컴파일한 결과와 비교하려면
// projects/edition_compare의 if_let_scope_2021 / if_let_scope_2024를 실행합니다.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::RwLock;

/// 버려질 때 자기 이름을 출력하는 값입니다.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.0);
    }
}

impl Noisy {
    fn find(&self, hit: bool) -> Option<u32> {
        hit.then_some(7)
    }
}

/// 피보나치 수를 메모이제이션으로 계산합니다. `if let`의 조건식에서 `memo.borrow()`로
/// 빌린 뒤, 값이 없으면 else에서 `memo.borrow_mut()`으로 결과를 넣습니다.
fn fib(n: u64, memo: &RefCell<HashMap<u64, u64>>) -> u64 {
    if n < 2 {
        return n;
    }
    if let Some(&cached) = memo.borrow().get(&n) {
        cached
    } else {
        // 2024: 조건식의 Ref 가드는 이미 버려졌으므로 여기서 다시 빌려도 됩니다.
        // (2021 에디션이었다면 가드가 아직 살아 있어 아래 borrow_mut()이 패닉합니다.)
        let value = fib(n - 1, memo) + fib(n - 2, memo);
        memo.borrow_mut().insert(n, value);
        value
    }
}

fn main() {
    println!("1. 임시값이 버려지는 시점");
    if let Some(n) = Noisy("조건식의 임시값").find(false) {
        println!("    then 블록: {n}");
    } else {
        println!("    else 블록");
    }
    // 출력:     [drop] 조건식의 임시값
    // 출력:     else 블록

    println!("2. else에서 같은 RefCell을 다시 빌리기: 메모이제이션");
    let memo = RefCell::new(HashMap::new());
    println!("    fib(50) = {}", fib(50, &memo));
    println!("    메모에 저장된 값: {}개", memo.borrow().len());
    // 출력:     fib(50) = 12586269025
    // 출력:     메모에 저장된 값: 49개

    println!("3. then 블록과 match에서는 가드가 끝까지 살아 있습니다");
    let log = RefCell::new(vec![String::from("시작")]);
    if let Some(first) = log.borrow().first() {
        // first는 가드를 거쳐 빌린 참조이므로, 이 블록에서는 가드가 살아 있어야 합니다.
        let can_write = log.try_borrow_mut().is_ok();
        println!("    if let의 then 블록: 첫 줄 {first:?}, 가변 대여 가능? {can_write}");
    }
    match log.borrow().get(5) {
        Some(line) => println!("    5번 줄: {line}"),
        None => {
            // None 갈래에는 빌린 값이 없지만, match는 어느 갈래든 조건식의 임시값을
            // match 전체가 끝날 때까지 유지합니다.
            let can_write = log.try_borrow_mut().is_ok();
            println!("    match의 None 갈래: 가변 대여 가능? {can_write}");
        }
    }
    // 출력:     if let의 then 블록: 첫 줄 "시작", 가변 대여 가능? false
    // 출력:     match의 None 갈래: 가변 대여 가능? false

    println!("4. 읽기 잠금으로 확인하고, 없으면 else에서 쓰기 잠금");
    let config: RwLock<Option<String>> = RwLock::new(None);
    if let Some(name) = config.read().unwrap().as_deref() {
        println!("    이미 설정됨: {name}");
    } else {
        // 2024: 조건식의 읽기 잠금은 이미 풀렸으므로 write()가 곧바로 성공합니다.
        // (2021 에디션이었다면 자기가 쥔 읽기 잠금이 풀리기를 기다리며 멈추거나(교착 상태),
        //  `write` 문서의 Panics 절대로 패닉할 수 있습니다.)
        *config.write().unwrap() = Some(String::from("기본값"));
        println!("    else에서 기본값을 썼습니다");
    }
    println!("    설정: {:?}", config.read().unwrap());
    // 출력:     else에서 기본값을 썼습니다
    // 출력:     설정: Some("기본값")
}
