// 기대하는 에러: E0369 cannot multiply `&str` by `{integer}` (매크로 판)
//              E0308 mismatched types, expected `u32`, found `&str` (얇은 매크로 판)
//
// 같은 실수(숫자 자리에 문자열을 넘김)를 두 모양의 매크로에 저질러, 에러가 어디를 가리키는지 비교합니다.
// - 계산을 매크로 안에서 하는 판: 에러가 매크로 "정의" 안의 `*`를 가리키고, 부른 곳은 "in this macro invocation"으로만
//   표시됩니다. 매크로가 다른 매크로를 부르면 그 사슬은 "this error originates in the macro ..." 한 줄로 줄어듭니다.
// - 계산을 함수에 맡기는 얇은 매크로 판: 함수의 시그니처가 타입을 정하므로, 에러가 부른 곳의 인자를 가리킵니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0369_logic_inside_macro.rs

// 계산을 매크로 안에서 하는 판: report!가 percent!를 부릅니다.
macro_rules! percent {
    ($part:expr, $whole:expr) => {
        $part * 100 / $whole
    };
}

macro_rules! report {
    ($label:expr, $part:expr, $whole:expr) => {
        println!("{}: {}%", $label, percent!($part, $whole))
    };
}

// 얇은 매크로 판: 계산은 타입이 적힌 함수가 하고, 매크로는 넘기기만 합니다.
fn percent_of(part: u32, whole: u32) -> u32 {
    part * 100 / whole
}

macro_rules! report_thin {
    ($label:expr, $part:expr, $whole:expr) => {
        println!("{}: {}%", $label, percent_of($part, $whole))
    };
}

fn main() {
    let done = "3"; // 실수: 숫자가 아니라 문자열
    let total = 4;
    report!("진행률", done, total); // E0369, 매크로 정의 안을 가리킴
    report_thin!("진행률", done, total); // E0308, 이 줄의 `done`을 가리킴
}
