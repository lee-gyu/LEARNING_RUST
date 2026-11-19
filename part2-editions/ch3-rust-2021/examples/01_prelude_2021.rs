// 2021 에디션 프렐류드에 들어온 TryFrom, TryInto, FromIterator를 use 없이 쓰고,
// 같은 이름의 메서드를 가진 사용자 트레이트와 부딪힐 때 무엇이 호출되는지 확인합니다.
//
// 실행: cargo run -p p2-ch3-rust-2021 --example 01_prelude_2021

/// TryInto보다 먼저 만들어진 사용자 트레이트라고 생각합니다.
/// 범위를 넘는 값을 에러로 돌려주지 않고 0..=255로 잘라 냅니다.
trait Clamp {
    fn try_into(&self) -> Result<u8, String>;
}

impl Clamp for i32 {
    fn try_into(&self) -> Result<u8, String> {
        // clamp 뒤에는 0..=255 안이므로 실패하지 않는 변환입니다.
        Ok(u8::try_from((*self).clamp(0, 255)).expect("0..=255 안의 값"))
    }
}

/// 고유 메서드(impl 블록에 직접 정의한 메서드)가 트레이트 메서드와 이름이 같은 경우입니다.
struct Celsius(f64);

impl Celsius {
    // TryInto::try_into와 이름도, self를 값으로 받는 모양도 같습니다.
    fn try_into(self) -> Result<i32, String> {
        if self.0.is_finite() {
            Ok(self.0.round() as i32)
        } else {
            Err(String::from("유한한 값이 아닙니다"))
        }
    }
}

fn main() {
    println!("1. use 없이 쓰는 세 트레이트");
    // 2018 에디션이라면 아래 네 줄은 use std::convert::{TryFrom, TryInto}; 와
    // use std::iter::FromIterator; 가 있어야 컴파일됩니다(없으면 E0599).
    let small = u8::try_from(200_i32);
    // (아래 Clamp는 i32에만 구현했으므로 u16에서는 겹치지 않습니다. 겹치는 경우는 2번 구역에서 봅니다.)
    let big: Result<u8, _> = 300_u16.try_into();
    let squares = Vec::from_iter((1..=4).map(|n| n * n));
    let word = String::from_iter(['러', '스', '트']);
    println!("    u8::try_from(200) = {small:?}");
    // 출력:     u8::try_from(200) = Ok(200)
    println!("    300u16.try_into() = {big:?}");
    // 출력:     300u16.try_into() = Err(TryFromIntError(PosOverflow))
    println!("    Vec::from_iter    = {squares:?}, String::from_iter = {word}");
    // 출력:     Vec::from_iter    = [1, 4, 9, 16], String::from_iter = 러스트

    println!("2. 이름이 겹치는 사용자 트레이트");
    let x: i32 = 300;
    // 메서드 탐색은 수신자를 값 그대로(i32) 받는 메서드를 먼저 찾고, 없을 때만 &i32로 넓힙니다.
    // 프렐류드의 TryInto::try_into는 self를 값으로 받으므로 첫 단계에서 찾아지고,
    // &self를 받는 Clamp::try_into는 다음 단계라서 밀려납니다.
    // 2018 에디션에서는 TryInto가 스코프에 없어서 같은 줄이 Clamp::try_into를 불렀습니다(Ok(255)).
    let by_method: Result<u8, _> = x.try_into();
    // 사용자 트레이트의 메서드를 부르려면 트레이트 이름을 경로에 적습니다.
    let by_path = Clamp::try_into(&x);
    let fully_qualified = <i32 as Clamp>::try_into(&x);
    println!("    x.try_into()               = {by_method:?}");
    // 출력:     x.try_into()               = Err(TryFromIntError(PosOverflow))
    println!("    Clamp::try_into(&x)        = {by_path:?}");
    // 출력:     Clamp::try_into(&x)        = Ok(255)
    println!("    <i32 as Clamp>::try_into() = {fully_qualified:?}");
    // 출력:     <i32 as Clamp>::try_into() = Ok(255)

    println!("3. 고유 메서드는 트레이트 메서드보다 먼저 찾아집니다");
    // Celsius에도 TryInto::try_into가 있지만(모든 타입에 대한 포괄 구현), 같은 단계에서는
    // 고유 메서드가 이깁니다. 그래서 이런 코드는 에디션을 바꿔도 그대로 동작합니다.
    let degrees = Celsius(36.6).try_into();
    println!("    Celsius(36.6).try_into() = {degrees:?}");
    // 출력:     Celsius(36.6).try_into() = Ok(37)
}
