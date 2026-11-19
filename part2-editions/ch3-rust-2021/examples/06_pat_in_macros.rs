// 2021 에디션부터 macro_rules의 $p:pat은 맨 바깥에 |가 있는 패턴(A | B)도 패턴 하나로 받습니다.
// |를 패턴 사이의 구분자로 쓰고 싶으면 예전 동작을 그대로 가진 $p:pat_param을 씁니다.
// 각 구역 주석의 "2018에서는"은 같은 코드를 2018 에디션으로 컴파일했을 때의 결과입니다.
//
// 실행: cargo run -p p2-ch3-rust-2021 --example 06_pat_in_macros

/// 어느 규칙이 입력을 받았는지 알려 줍니다. 위의 규칙부터 차례로 맞춰 봅니다.
macro_rules! classify {
    ($p:pat) => {
        "규칙 1: 패턴 하나"
    };
    ($($t:tt)*) => {
        "규칙 2: 패턴이 아닌 토큰들"
    };
}

/// 값이 패턴에 맞는지 알려 줍니다. 표준 라이브러리의 matches!와 같은 일을 합니다.
macro_rules! is_match {
    ($value:expr, $p:pat) => {
        match $value {
            $p => true,
            _ => false,
        }
    };
}

/// $p:pat은 1 | 2 전체를 패턴 하나로 받습니다.
macro_rules! whole {
    ($p:pat) => {
        [stringify!($p)]
    };
}

/// $p:pat_param은 맨 바깥의 |를 받지 않으므로, |를 구분자로 써서 패턴을 하나씩 나눠 받습니다.
macro_rules! split {
    ($($p:pat_param)|+) => {
        [$(stringify!($p)),+]
    };
}

fn main() {
    println!("1. $p:pat이 1 | 2를 패턴 하나로 받습니다");
    println!("    classify!(1 | 2)       → {}", classify!(1 | 2));
    // 출력:     classify!(1 | 2)       → 규칙 1: 패턴 하나
    println!("    classify!(Some(1 | 2)) → {}", classify!(Some(1 | 2)));
    // 출력:     classify!(Some(1 | 2)) → 규칙 1: 패턴 하나
    // 2018에서는: 첫 줄이 "규칙 2"입니다. $p:pat이 1까지만 받고 | 2가 남아 규칙 1이 맞지 않기 때문입니다.
    // 괄호 안에 들어간 |(Some(1 | 2))는 1.53.0부터 모든 에디션에서 $p:pat으로 받습니다.

    println!("2. 그래서 패턴을 받는 매크로에 A | B를 넘길 수 있습니다");
    let grade = 'B';
    println!(
        "    is_match!(grade, 'A' | 'B') → {}",
        is_match!(grade, 'A' | 'B')
    );
    // 출력:     is_match!(grade, 'A' | 'B') → true
    // 2018에서는: "no rules expected `|`" 컴파일 에러입니다.

    println!("3. pat과 pat_param이 같은 입력을 나누는 방식");
    println!("    whole!(1 | 2..=5 | _) → {:?}", whole!(1 | 2..=5 | _));
    // 출력:     whole!(1 | 2..=5 | _) → ["1 | 2..=5 | _"]
    // 2018에서는: whole!도 "no rules expected `|`" 에러입니다. split!은 2018에서도 같은 결과입니다.
    println!("    split!(1 | 2..=5 | _) → {:?}", split!(1 | 2..=5 | _));
    // 출력:     split!(1 | 2..=5 | _) → ["1", "2..=5", "_"]
    // split!의 규칙을 $($p:pat)|+로 쓰면 2021부터는 "`$p:pat` is followed by `|`" 에러입니다.
    // pat_param은 1.53.0에 추가되어 모든 에디션에서 쓸 수 있습니다.
}
