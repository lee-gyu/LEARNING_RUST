// 기대하는 에러: E0605
// 핵심 메시지: non-primitive cast: `u8` as `Direction`
//
// C에서는 정수를 enum 변수에 그대로 넣을 수 있고, 어느 열거자에도 해당하지 않는 값도 들어갑니다. Rust의 열거형은
// 정해진 배리언트 가운데 하나만 값이 될 수 있어서(Reference: 열거형은 올바른 판별값을 가져야 하고, 그렇지 않은 값을
// 만드는 것은 미정의 동작), 정수를 열거형으로 바꾸는 as 변환을 허용하지 않습니다. 반대 방향(Direction::West as u8)은
// 언제나 올바른 값이라 허용됩니다.
//
// 고치는 법: match로 바꾸고, 해당하지 않는 정수는 None이나 Err로 돌려줍니다(TryFrom을 구현해도 됩니다).
//   let dir = match code { 0 => Some(Direction::North), 1 => Some(Direction::East), … , _ => None };
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch10-undefined-behavior compile_fail/e0605_int_to_enum.rs

#[derive(Debug)]
#[repr(u8)]
enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

fn main() {
    let code: u8 = 2;
    let dir = code as Direction; // E0605
    println!("{dir:?}, West = {}", Direction::West as u8);
}
