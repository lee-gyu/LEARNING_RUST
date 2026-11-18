// 기대하는 에러: error[E0277]: the trait bound `Color: FromStr` is not satisfied
// 핵심 메시지:   the trait `FromStr` is not implemented for `Color`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_parse_without_fromstr.rs --out-dir /tmp/rust-p1-ch3
//
// parse::<T>()의 시그니처는 `fn parse<F: FromStr>(&self) -> Result<F, F::Err>`입니다.
// Color는 FromStr을 구현하지 않았으므로 "문자열에서 Color를 만드는 방법"이 없습니다.
// 컴파일러는 derive(Debug)만 보고 문자열 형식을 짐작해 주지 않습니다.
// 고치는 방법: `impl FromStr for Color { type Err = ...; fn from_str(s: &str) -> ... }`를 작성합니다
//             (examples/08_parse_fromstr.rs의 Version과 같은 방법).

#[derive(Debug)]
enum Color {
    Red,
    Green,
}

fn main() {
    let color = "red".parse::<Color>();
    println!("{color:?}");
    let _ = (Color::Red, Color::Green);
}
