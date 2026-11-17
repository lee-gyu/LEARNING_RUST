// 이 예제가 보여 주는 것: 제네릭 타입 매개변수, 트레이트 바운드, 라이프타임을 한 함수에서 함께 쓰는 방법
// 실행: cargo run -p p0-ch10-generics-traits-lifetimes --example 09_generics_bounds_lifetimes

use std::fmt::Display;

// 라이프타임도 제네릭 매개변수의 한 종류라서, 타입 매개변수 T와 함께 같은 <> 안에 선언합니다.
// 관례상 라이프타임을 먼저 씁니다.
// - 'a: 반환값은 x, y에서 빌린 것이므로 07 예제의 longest와 같은 관계를 적습니다.
// - T: Display: 본문에서 ann을 {}로 출력하므로 Display가 필요합니다. 바운드는 where 절에 적었습니다.
// ann은 값으로 받고 반환값과 상관이 없으므로 라이프타임을 붙이지 않습니다.
fn longest_with_an_announcement<'a, T>(x: &'a str, y: &'a str, ann: T) -> &'a str
where
    T: Display,
{
    println!("    알림! {ann}");
    if x.len() > y.len() { x } else { y }
}

// 세 가지가 서로 맞물린 경우입니다. 같은 타입 T의 값 둘 중 "표시했을 때 더 긴 쪽"의 참조를 돌려줍니다.
// - T: 어떤 타입이든 받는 제네릭
// - T: Display: 길이를 재려고 to_string()을 쓰는데, to_string은 Display가 있어야 생깁니다(06 예제의 포괄 구현).
// - 'a: 입력 참조가 둘이라 생략 규칙으로 정해지지 않으므로, 반환값이 두 인자 모두에서 올 수 있다고 적습니다.
fn longer_display<'a, T: Display>(a: &'a T, b: &'a T) -> &'a T {
    if a.to_string().len() >= b.to_string().len() {
        a
    } else {
        b
    }
}

// Display만 구현한 직접 만든 타입입니다. T 자리에 들어갈 수 있다는 것을 보여 줍니다.
struct Temperature(i32);

impl Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "섭씨 {}도", self.0)
    }
}

fn main() {
    println!("1. 같은 함수에 T만 바꿔 넘깁니다");
    let string1 = String::from("abcd");
    let string2 = "xyz";
    let result =
        longest_with_an_announcement(string1.as_str(), string2, "오늘은 누군가의 생일입니다!");
    println!("    더 긴 문자열: {result}");
    let result = longest_with_an_announcement(string1.as_str(), string2, 2024); // T = i32
    println!("    더 긴 문자열: {result}");
    let result = longest_with_an_announcement(string1.as_str(), string2, Temperature(21)); // T = Temperature
    println!("    더 긴 문자열: {result}");
    // 출력:
    //     알림! 오늘은 누군가의 생일입니다!
    //     더 긴 문자열: abcd
    //     알림! 2024
    //     더 긴 문자열: abcd
    //     알림! 섭씨 21도
    //     더 긴 문자열: abcd

    println!("2. 제네릭·바운드·라이프타임이 맞물린 함수");
    let x = 12345;
    let y = 678;
    println!("    정수: {}", longer_display(&x, &y));
    let hot = Temperature(35);
    let cold = Temperature(-15);
    println!("    온도: {}", longer_display(&hot, &cold));
    // 출력:
    //     정수: 12345
    //     온도: 섭씨 -15도
    // 더 "큰" 값이 아니라 표시했을 때 더 "긴" 값을 고릅니다. "-15"는 "35"보다 한 글자(1바이트) 깁니다.

    println!("3. 반환된 참조는 두 인자가 모두 살아 있는 동안 쓸 수 있습니다");
    let first = String::from("러스트");
    let chosen;
    {
        let second = String::from("Rust");
        chosen = longer_display(&first, &second);
        println!("    블록 안에서: {chosen}");
    }
    // 출력: 블록 안에서: 러스트
    // "러스트"는 한글 3글자 × 3바이트 = 9바이트라서 "Rust"(4바이트)보다 깁니다(UTF-8은 ch8).
    // chosen을 블록 밖에서 쓰면, 실제로는 first를 가리켜도 second 때문에 E0597이 납니다.
}
