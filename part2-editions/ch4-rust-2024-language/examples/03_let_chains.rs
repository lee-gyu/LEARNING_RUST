// let 체인이 2024 에디션에서만 되는 이유(임시값 범위)와 `while` 조건에서의 함정을 보여 줍니다.
//
// 실행: cargo run -p p2-ch4-rust-2024-language --example 03_let_chains
//
// let 체인 문법 자체는 Part 0 ch18에서 배웠습니다. 2021 에디션으로 컴파일하면
// "let chains are only allowed in Rust 2024 or later" 에러가 납니다
// (part2-editions/ch1-editions-overview/compile_fail/let_chains_2021.rs).

/// 버려질 때 자기 이름을 출력하는 값입니다.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.0);
    }
}

impl Noisy {
    /// 평가되었다는 것을 출력하고 `value`를 그대로 돌려줍니다.
    fn get(&self, value: Option<i32>) -> Option<i32> {
        println!("    평가: {}", self.0);
        value
    }
}

/// "3, 4" 같은 문자열을 정수 두 개로 읽습니다. let 세 개와 bool 조건 하나를 잇습니다.
fn parse_pair(text: &str) -> Option<(i32, i32)> {
    if let Some((left, right)) = text.split_once(',')
        && let Ok(x) = left.trim().parse::<i32>()
        && let Ok(y) = right.trim().parse::<i32>()
        && x <= y
    {
        Some((x, y))
    } else {
        None
    }
}

fn main() {
    println!("1. let과 bool 조건을 &&로 잇기");
    for text in ["3, 4", "3, 넷", "5, 4", "34"] {
        println!("    {text:?} → {:?}", parse_pair(text));
    }
    // 출력:     "3, 4" → Some((3, 4))
    // 출력:     "3, 넷" → None
    // 출력:     "5, 4" → None
    // 출력:     "34" → None

    println!("2. 고리가 실패하면, 그때까지 만든 임시값을 모두 버린 뒤 else로 갑니다");
    if let Some(a) = Noisy("첫째 고리의 임시값").get(Some(1))
        && let Some(b) = Noisy("둘째 고리의 임시값").get(None)
    {
        println!("    then: {a} {b}");
    } else {
        println!("    else 블록");
    }
    // 출력:     평가: 첫째 고리의 임시값
    // 출력:     평가: 둘째 고리의 임시값
    // 출력:     [drop] 둘째 고리의 임시값
    // 출력:     [drop] 첫째 고리의 임시값
    // 출력:     else 블록
    // Edition Guide는 let 체인이 2024 에디션의 `if let` 임시값 규칙(else 전에 버림)에
    // 기대기 때문에 2024 에디션에서만 열렸다고 설명합니다.

    println!("3. while 조건의 함정: 조건이 거짓이 되는 순간에도 pop은 이미 일어났습니다");
    let mut stack = vec![1, 2, 3, 4, 5];
    while let Some(top) = stack.pop()
        && top > 2
    {
        println!("    처리: {top}");
    }
    println!("    남은 스택: {stack:?}");
    // 출력:     처리: 5
    // 출력:     처리: 4
    // 출력:     처리: 3
    // 출력:     남은 스택: [1]
    // 2는 pop()으로 꺼냈지만 `top > 2`가 거짓이라 처리되지 않고 그대로 사라졌습니다.

    println!("4. 고친 모양: 꺼내기 전에 last()로 먼저 확인합니다");
    let mut stack = vec![1, 2, 3, 4, 5];
    while let Some(&top) = stack.last()
        && top > 2
    {
        stack.pop();
        println!("    처리: {top}");
    }
    println!("    남은 스택: {stack:?}");
    // 출력:     처리: 5
    // 출력:     처리: 4
    // 출력:     처리: 3
    // 출력:     남은 스택: [1, 2]
}
