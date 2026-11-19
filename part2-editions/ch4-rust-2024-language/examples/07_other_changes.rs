// 2024 에디션의 나머지 언어 변경을 한 조각씩 보여 줍니다.
// `gen` 예약 키워드, 패턴의 `mut`/`ref`/`&` 제한, 매크로의 `expr` 프래그먼트 지정자, 예약 문법.
//
// 실행: cargo run -p p2-ch4-rust-2024-language --example 07_other_changes

/// 2021 에디션 코드에서 옮겨 온 타입이라고 칩니다. 메서드 이름이 `gen`이었습니다.
struct Counter {
    next: u32,
}

impl Counter {
    /// 2024 에디션에서 `gen`은 예약 키워드라서 raw 식별자 `r#gen`으로만 이름을 붙일 수 있습니다.
    fn r#gen(&mut self) -> u32 {
        self.next += 1;
        self.next
    }

    /// 새로 짓는 코드라면 키워드와 겹치지 않는 이름이 더 편합니다.
    fn next_value(&mut self) -> u32 {
        self.r#gen()
    }
}

/// 입력이 어느 규칙에 맞았는지 알려 주는 매크로입니다.
macro_rules! which_rule {
    ($e:expr) => {
        "1번 규칙($e:expr)"
    };
    (const $b:block) => {
        "2번 규칙(const 블록)"
    };
}

/// 1번 규칙만 `expr_2021`로 바꾼 매크로입니다.
macro_rules! which_rule_2021 {
    ($e:expr_2021) => {
        "1번 규칙($e:expr_2021)"
    };
    (const $b:block) => {
        "2번 규칙(const 블록)"
    };
}

/// 토큰 트리(`tt`)가 몇 개인지 알려 주는 매크로입니다. 예약 문법을 확인하는 데 씁니다.
macro_rules! token_count {
    ($a:tt) => {
        1
    };
    ($a:tt $b:tt $c:tt) => {
        3
    };
}

fn main() {
    println!("1. gen 키워드");
    let mut counter = Counter { next: 0 };
    let r#gen = counter.r#gen(); // 변수 이름도 r#gen으로 씁니다.
    let next = counter.next_value();
    // 서식 문자열 안에서는 r# 없이 {gen}이라고 씁니다({r#gen}은 에러).
    println!("    counter.r#gen() = {gen}, counter.next_value() = {next}");
    // 출력:     counter.r#gen() = 1, counter.next_value() = 2

    println!("2. 참조를 맞추는 패턴 안에서는 mut/ref/&를 쓸 수 없습니다");
    let pair = &(String::from("러스트"), 2024);
    let (name, year) = pair; // 기본 바인딩 모드: name은 &String, year는 &i32
    println!("    기본 바인딩 모드: {name}, {year}");
    // `let (name, mut year) = pair;`는 2024 에디션에서 에러입니다.
    // 값을 복사해 고치고 싶다면 바깥 &부터 직접 적습니다(완전히 명시한 패턴).
    let &(ref title, mut edition) = pair;
    edition += 3;
    println!("    완전히 명시한 패턴: {title}, {edition}");
    // 출력:     기본 바인딩 모드: 러스트, 2024
    // 출력:     완전히 명시한 패턴: 러스트, 2027

    println!("3. 매크로의 expr 프래그먼트 지정자");
    println!(
        "    expr      + const 블록 → {}",
        which_rule!(const { 1 + 1 })
    );
    println!(
        "    expr_2021 + const 블록 → {}",
        which_rule_2021!(const { 1 + 1 })
    );
    // 출력:     expr      + const 블록 → 1번 규칙($e:expr)
    // 출력:     expr_2021 + const 블록 → 2번 규칙(const 블록)

    println!("4. 예약 문법: #\"...\"#와 ##");
    // raw 문자열 r#"..."#는 예전처럼 토큰 하나입니다.
    println!("    r#\"foo\"#   → 토큰 {}개", token_count!(r#"foo"#));
    // `#"foo"#`와 `###`는 2024 에디션에서 에러입니다. 공백으로 띄우면 예전처럼 토큰 세 개입니다.
    println!("    # \"foo\" #  → 토큰 {}개", token_count!(# "foo" #));
    println!("    # # #      → 토큰 {}개", token_count!(# # #));
    // 출력:     r#"foo"#   → 토큰 1개
    // 출력:     # "foo" #  → 토큰 3개
    // 출력:     # # #      → 토큰 3개
}
