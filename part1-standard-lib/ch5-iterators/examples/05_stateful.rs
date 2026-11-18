// 이 예제가 보여 주는 것: 반복하는 동안 상태를 들고 가는 도구 — scan, peekable(peek, next_if, next_if_eq), by_ref
// 실행: cargo run -p p1-ch5-iterators --example 05_stateful
//
// map의 클로저는 원소 하나만 보고 결정합니다. "지금까지의 합", "다음 원소가 무엇인가", "앞부분만 쓰고 나머지는 나중에"처럼
// 앞뒤 맥락이 필요할 때 이 도구들을 씁니다.

// 계산식 문자열을 잘라 만든 토큰입니다.
enum Token {
    Num(u32),
    Op(char),
}

// 토큰을 [수 12], [연산자 +] 모양의 문자열로 보여 줍니다.
fn show(tokens: &[Token]) -> String {
    let parts: Vec<String> = tokens
        .iter()
        .map(|t| match t {
            Token::Num(n) => format!("[수 {n}]"),
            Token::Op(c) => format!("[연산자 {c}]"),
        })
        .collect();
    parts.join(" ")
}

// peekable로 "다음 글자가 숫자인 동안" 이어 읽어 여러 자리 수를 하나의 토큰으로 만듭니다.
fn tokenize(src: &str) -> Vec<Token> {
    let mut chars = src.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(c) = chars.next() {
        if let Some(first) = c.to_digit(10) {
            let mut n = first;
            // next_if: 다음 원소가 조건을 만족할 때만 꺼냅니다. 만족하지 않으면 꺼내지 않고 그대로 둡니다.
            while let Some(d) = chars.next_if(|c| c.is_ascii_digit()) {
                n = n * 10 + d.to_digit(10).unwrap_or(0);
            }
            tokens.push(Token::Num(n));
        } else if !c.is_whitespace() {
            tokens.push(Token::Op(c));
        }
    }
    tokens
}

fn main() {
    println!("== 1. scan: 중간 상태를 하나씩 내놓기 ==");
    let transactions = [5000, -1200, -3000, 2000, -4500, 1000];
    // scan(초기 상태, |&mut 상태, 원소| -> Option<내놓을 값>)
    // 상태는 클로저 밖(Scan 구조체 안)에 살아 있고, 클로저는 &mut로 받아 고칩니다.
    let balances: Vec<i32> = transactions
        .iter()
        .scan(0, |balance, &t| {
            *balance += t;
            Some(*balance)
        })
        .collect();
    let final_balance: i32 = transactions.iter().sum();
    println!("잔액 변화: {balances:?}");
    println!("최종 잔액: {final_balance}");
    // 출력: 잔액 변화: [5000, 3800, 800, 2800, -1700, -700]
    // 출력: 최종 잔액: -700
    // sum이나 fold는 마지막 결과 하나만, scan은 중간 결과를 모두 내놓습니다.

    // 클로저가 None을 돌려주면 scan은 거기서 끝납니다.
    let until_overdrawn: Vec<i32> = transactions
        .iter()
        .scan(0, |balance, &t| {
            *balance += t;
            if *balance < 0 { None } else { Some(*balance) }
        })
        .collect();
    println!("마이너스 직전까지: {until_overdrawn:?}");
    // 출력: 마이너스 직전까지: [5000, 3800, 800, 2800]

    println!("\n== 2. peekable: 꺼내지 않고 다음 원소 보기 ==");
    let mut it = [10, 20, 30].iter().peekable();
    println!("peek: {:?}", it.peek());
    println!("peek: {:?}", it.peek()); // 몇 번 봐도 꺼내지 않습니다.
    println!("next: {:?}", it.next());
    println!("peek: {:?}", it.peek());
    // 출력: peek: Some(10)
    // 출력: peek: Some(10)
    // 출력: next: Some(10)
    // 출력: peek: Some(20)

    // peek가 None이면 방금 꺼낸 것이 마지막 원소입니다.
    let mut names = ["가람", "나래", "다온"].into_iter().peekable();
    let mut line = String::new();
    while let Some(name) = names.next() {
        line.push_str(name);
        if names.peek().is_some() {
            line.push_str(" → ");
        }
    }
    println!("마지막 판별: {line}");
    // 출력: 마지막 판별: 가람 → 나래 → 다온

    println!("\n== 3. next_if / next_if_eq ==");
    println!("tokenize: {}", show(&tokenize("12+345 * 6")));
    // 출력: tokenize: [수 12] [연산자 +] [수 345] [연산자 *] [수 6]

    // 연속된 같은 글자 묶기(run-length). next_if_eq(&c)는 다음 원소가 c와 같을 때만 꺼냅니다.
    let mut chars = "aaabccdddd".chars().peekable();
    let mut runs = Vec::new();
    while let Some(c) = chars.next() {
        let mut count = 1;
        while chars.next_if_eq(&c).is_some() {
            count += 1;
        }
        runs.push(format!("{c}{count}"));
    }
    println!("run-length: {}", runs.join(" "));
    // 출력: run-length: a3 b1 c2 d4

    println!("\n== 4. by_ref: 앞부분만 쓰고 나머지는 나중에 ==");
    let post = "제목: 반복자\n작성자: 페리스\n\n본문 첫 줄\n본문 둘째 줄";
    let mut lines = post.lines();
    // take_while은 self를 받으므로 그냥 부르면 lines가 이동해 버립니다.
    // by_ref()는 &mut lines를 돌려주고, &mut I도 Iterator이므로 그 위에 어댑터를 붙일 수 있습니다.
    let headers: Vec<&str> = lines.by_ref().take_while(|l| !l.is_empty()).collect();
    let body: Vec<&str> = lines.collect(); // 남은 줄을 이어서 꺼냅니다.
    println!("머리글: {headers:?}");
    println!("본문:   {body:?}");
    // 출력: 머리글: ["제목: 반복자", "작성자: 페리스"]
    // 출력: 본문:   ["본문 첫 줄", "본문 둘째 줄"]

    // 주의: take_while은 조건이 false인 원소를 "확인하려고 꺼낸 뒤" 버립니다. 위에서는 빈 줄이라 괜찮았지만,
    // 그 원소가 필요하면 잃어버립니다.
    let mut nums = [1, 2, 3, 10, 4, 5].into_iter();
    let small: Vec<i32> = nums.by_ref().take_while(|&n| n < 5).collect();
    let rest: Vec<i32> = nums.collect();
    println!("take_while: {small:?}, 나머지 {rest:?}  ← 10이 사라짐");
    // 출력: take_while: [1, 2, 3], 나머지 [4, 5]  ← 10이 사라짐

    // peekable의 next_if는 조건이 false인 원소를 꺼내지 않으므로 잃어버리지 않습니다.
    let mut nums = [1, 2, 3, 10, 4, 5].into_iter().peekable();
    let mut small = Vec::new();
    while let Some(n) = nums.next_if(|&n| n < 5) {
        small.push(n);
    }
    let rest: Vec<i32> = nums.collect();
    println!("next_if:    {small:?}, 나머지 {rest:?}");
    // 출력: next_if:    [1, 2, 3], 나머지 [10, 4, 5]
}
