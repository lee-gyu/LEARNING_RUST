// macro_rules!로 선언적 매크로를 만듭니다: vec! 흉내 내기(반복 $(...),*),
// 갈래가 여러 개인 매크로와 재귀, 코드 자체를 받는 매크로(stringify!),
// 함수로는 할 수 없는 항목(구조체, 트레이트 구현) 만들기를 보여 줍니다.
//
// 실행: cargo run -p p0-ch19-advanced-features --example 08_macro_rules

use std::fmt;

// 매크로는 쓰기 전에(파일에서 위쪽에) 정의되어 있어야 합니다. 함수와 다른 점입니다.

// 1. vec! 흉내 내기
//    $( ... ),*  : 괄호 안의 패턴이 쉼표로 구분되어 0번 이상 반복됩니다.
//    $x:expr     : 표현식 하나와 맞춰 보고, 맞으면 그 코드를 $x라는 이름으로 붙잡습니다.
//    $(,)?       : 끝에 쉼표가 하나 있어도 되고 없어도 됩니다.
macro_rules! my_vec {
    // 빈 호출 my_vec![]은 이 갈래가 먼저 받습니다. 이 갈래가 없으면 아래 갈래가 push 없는
    // `let mut temp_vec`을 만들어 unused_mut 경고가 납니다. 매크로가 만든 코드도 똑같이 검사되기 때문입니다.
    () => {
        Vec::new()
    };
    ( $( $x:expr ),* $(,)? ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x); // 붙잡은 $x마다 이 줄이 한 번씩 만들어집니다.
            )*
            temp_vec
        }
    };
}

// 2. 갈래가 여러 개인 매크로: match처럼 위에서부터 차례로 맞춰 봅니다.
//    두 번째 갈래는 자기 자신을 다시 불러(재귀) 인자 개수에 상관없이 동작합니다.
macro_rules! max {
    ($x:expr) => {
        $x
    };
    ($x:expr, $($rest:expr),+) => {{
        let first = $x;
        let rest = max!($($rest),+);
        if first > rest { first } else { rest }
    }};
}

// 3. 매크로는 값이 아니라 코드를 받습니다. stringify!는 받은 코드를 그대로 문자열로 만듭니다.
macro_rules! show {
    ($e:expr) => {
        println!("    {} = {:?}", stringify!($e), $e)
    };
}

// 4. 매크로는 항목(구조체 정의, 트레이트 구현)도 만들 수 있습니다. 함수로는 할 수 없는 일입니다.
//    $name:ident는 식별자(이름), $unit:literal은 리터럴과 맞춰 봅니다.
macro_rules! unit_type {
    ($name:ident, $unit:literal) => {
        #[derive(Debug, Clone, Copy)]
        struct $name(f64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}{}", self.0, $unit)
            }
        }
    };
}

unit_type!(Meters, "m");
unit_type!(Seconds, "초");

fn main() {
    println!("1. my_vec!");
    let empty: Vec<i32> = my_vec![];
    // 펼쳐진 코드(Vec::new() 뒤에 push를 이어 쓰는 모양)를 보고 Clippy가 vec![]을 쓰라고 권합니다.
    // vec!의 동작을 흉내 내는 것이 이 예제의 목적이므로, 호출한 곳에 예상된 경고라고 표시합니다.
    #[expect(clippy::vec_init_then_push, reason = "vec!을 흉내 낸 매크로라서")]
    let numbers = my_vec![1, 2, 3];
    #[expect(clippy::vec_init_then_push, reason = "vec!을 흉내 낸 매크로라서")]
    let words = my_vec!["가", "나",]; // 끝의 쉼표도 허용됩니다.
    println!("    {empty:?} {numbers:?} {words:?}"); // 출력:     [] [1, 2, 3] ["가", "나"]

    println!("2. max!: 인자 개수가 정해져 있지 않습니다");
    println!("    max!(7) = {}", max!(7)); // 출력:     max!(7) = 7
    println!("    max!(3, 9, 2) = {}", max!(3, 9, 2)); // 출력:     max!(3, 9, 2) = 9
    println!("    max!(1.5, 0.5) = {}", max!(1.5, 0.5)); // 출력:     max!(1.5, 0.5) = 1.5

    println!("3. show!: 코드와 값을 함께 출력");
    show!(1 + 2 * 3); // 출력:     1 + 2 * 3 = 7
    show!(numbers.len()); // 출력:     numbers.len() = 3
    show!(max!(4, 8)); // 출력:     max!(4, 8) = 8

    println!("4. unit_type!이 만든 타입");
    let distance = Meters(100.0);
    let time = Seconds(9.58);
    println!("    {time} 동안 {distance} 달렸습니다"); // 출력:     9.58초 동안 100m 달렸습니다
    println!("    초속 {:.2}m", distance.0 / time.0); // 출력:     초속 10.44m
}
