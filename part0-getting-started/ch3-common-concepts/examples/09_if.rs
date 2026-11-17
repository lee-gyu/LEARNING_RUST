// 이 예제가 보여 주는 것: `if` 표현식 — `bool` 조건, `else if` 연쇄, 값을 내는 `if`
// 실행: cargo run -p p0-ch3-common-concepts --example 09_if

fn main() {
    // ── 1. 조건은 반드시 `bool`이어야 합니다 ───────────────────────────────
    let number = 3;
    if number < 5 {
        println!("조건이 참입니다"); // 출력: 조건이 참입니다
    } else {
        println!("조건이 거짓입니다");
    }

    // JavaScript나 Python과 달리 Rust는 정수를 `bool`로 자동 변환하지 않습니다.
    // `if number { ... }`는 컴파일 에러입니다 (compile_fail/e0308_if_condition_not_bool.rs).
    // "0이 아니면"이라는 뜻을 원한다면 비교식으로 직접 씁니다.
    if number != 0 {
        println!("number는 0이 아닙니다"); // 출력: number는 0이 아닙니다
    }

    // ── 2. `else if`: 위에서부터 검사해 처음으로 참인 분기 하나만 실행합니다 ──
    // 6은 2로도 나누어떨어지지만, 3으로 나누어떨어지는 분기가 먼저 참이 되므로 그 아래는 검사하지 않습니다.
    let number = 6;
    if number % 4 == 0 {
        println!("{number}은 4로 나누어떨어집니다");
    } else if number % 3 == 0 {
        println!("{number}은 3으로 나누어떨어집니다"); // 출력: 6은 3으로 나누어떨어집니다
    } else if number % 2 == 0 {
        println!("{number}은 2로 나누어떨어집니다");
    } else {
        println!("{number}은 4, 3, 2로 나누어떨어지지 않습니다");
    }

    // ── 3. `if`는 표현식이므로 값을 냅니다 ─────────────────────────────────
    // 각 분기 블록의 마지막 표현식이 `if` 전체의 값이 됩니다. 다른 언어의 삼항 연산자(`? :`) 역할입니다.
    let condition = true;
    let value = if condition { 5 } else { 6 };
    println!("value = {value}"); // 출력: value = 5

    // 모든 분기는 같은 타입의 값을 내야 합니다. 변수 `value`의 타입이 컴파일할 때 하나로 정해져야 하기 때문입니다.
    // `if condition { 5 } else { "six" }`는 E0308(`if` and `else` have incompatible types) 에러입니다.

    let temperature = 28;
    let advice = if temperature >= 30 {
        "에어컨을 켜세요"
    } else if temperature >= 20 {
        "적당한 날씨입니다"
    } else {
        "겉옷을 챙기세요"
    };
    println!("{temperature}도: {advice}"); // 출력: 28도: 적당한 날씨입니다

    // ── 4. 함수의 마지막 표현식으로 쓰는 `if` ──────────────────────────────
    // `return` 없이 `if` 표현식의 값이 그대로 함수의 반환값이 됩니다.
    println!("sign(-7) = {}", sign(-7)); // 출력: sign(-7) = -1
    println!("sign(0) = {}", sign(0)); // 출력: sign(0) = 0
    println!("sign(42) = {}", sign(42)); // 출력: sign(42) = 1
}

fn sign(n: i32) -> i32 {
    if n > 0 {
        1
    } else if n < 0 {
        -1
    } else {
        0
    }
}
