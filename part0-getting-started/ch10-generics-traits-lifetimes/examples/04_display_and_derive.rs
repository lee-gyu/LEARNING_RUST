// 이 예제가 보여 주는 것: {}와 {:?}가 각각 Display와 Debug 트레이트라는 것, Display를 직접 구현하는 방법,
//                        #[derive(...)]가 트레이트 구현을 자동으로 만들어 준다는 것
// 실행: cargo run -p p0-ch10-generics-traits-lifetimes --example 04_display_and_derive

use std::fmt;

// #[derive(Debug)]는 "Debug 트레이트의 impl을 컴파일러가 대신 작성해 달라"는 요청입니다.
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

// Display는 derive할 수 없습니다. 사용자에게 보여 줄 모양은 타입마다 다르므로 직접 정합니다(ch5 2.6절).
// Display 트레이트는 `fmt` 메서드 하나를 요구합니다.
// - f: 출력이 쌓이는 곳(Formatter)입니다. 화면에 바로 찍는 대신 여기에 씁니다.
// - write!(f, ...)는 println!과 같은 서식 문자열을 받아 f에 씁니다.
// - fmt::Result는 쓰기의 성공·실패를 알리는 Result 타입입니다(Result는 ch9).
// 너비·정렬 같은 서식 지정을 직접 구현한 Display에서 다루는 방법은 Part 1 ch3-strings-fmt에서 배웁니다.
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// 여러 트레이트를 한꺼번에 derive할 수 있습니다. 각각 다음 기능을 줍니다.
// - Clone: .clone()으로 복제     - Copy: 대입할 때 이동 대신 복사(필드가 모두 Copy여야 함)
// - PartialEq: ==, !=           - PartialOrd: <, >, <=, >= (필드를 선언 순서대로 비교)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Version(u32, u32, u32);

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.0, self.1, self.2)
    }
}

// 01_generic_largest.rs의 제네릭 함수와 같습니다. T는 PartialOrd를 구현한 타입이어야 합니다.
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn main() {
    let p = Point { x: 3, y: -2 };

    println!("1. {{}}는 Display, {{:?}}는 Debug 트레이트의 구현을 부릅니다");
    println!("    Display: {p}");
    println!("    Debug:   {p:?}");
    println!("    필드:    x = {}, y = {}", p.x, p.y);
    // 출력:
    //     Display: (3, -2)
    //     Debug:   Point { x: 3, y: -2 }
    //     필드:    x = 3, y = -2

    println!("2. Display를 구현하면 to_string()과 format!도 쓸 수 있습니다");
    // to_string은 Display를 구현한 모든 타입에 자동으로 생기는 메서드입니다(포괄 구현, 06 예제).
    let text: String = p.to_string();
    let sentence = format!("점 {p}에서 출발");
    println!("    text = {text:?}, sentence = {sentence}");
    // 출력: text = "(3, -2)", sentence = 점 (3, -2)에서 출발

    println!("3. derive가 만들어 준 구현으로 비교와 복사를 합니다");
    let current = Version(1, 8, 2);
    let next = Version(1, 10, 0);
    let copied = current; // Copy를 derive했으므로 이동이 아니라 복사입니다. current를 계속 쓸 수 있습니다.
    println!("    {current} == {copied} ? {}", current == copied);
    println!("    {current} < {next} ? {}", current < next);
    // 출력:
    //     v1.8.2 == v1.8.2 ? true
    //     v1.8.2 < v1.10.0 ? true
    // PartialOrd는 첫 필드부터 차례로 비교합니다. 1 == 1이므로 다음 필드 8 < 10으로 결정됩니다.
    // 문자열 "1.8.2" < "1.10.0"으로 비교했다면 '8' > '1'이라서 false가 나왔을 것입니다.

    println!("4. derive한 PartialOrd 덕분에 제네릭 함수 largest의 조건을 만족합니다");
    let releases = [Version(1, 2, 0), Version(2, 0, 1), Version(1, 10, 3)];
    println!("    최신 버전: {}", largest(&releases));
    // 출력: 최신 버전: v2.0.1
    // Version에서 PartialOrd를 지우면 이 줄이 E0277(can't compare `Version` with `Version`)로 거부됩니다.
}
