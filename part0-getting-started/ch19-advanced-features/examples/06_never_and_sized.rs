// never 타입 `!`(값을 만들지 않고 흐름을 끊는 표현식의 타입)과
// 동적 크기 타입(DST), Sized 트레이트, ?Sized 바운드를 보여 줍니다.
//
// 실행: cargo run -p p0-ch19-advanced-features --example 06_never_and_sized
//
// 크기(바이트 수)는 64비트 환경에서 확인한 값입니다.

use std::fmt::{self, Display};
use std::mem::{size_of, size_of_val};

// 1. 반환 타입이 !인 함수는 절대 돌아오지 않습니다. 끝까지 가면 반드시 패닉합니다.
fn fail(message: &str) -> ! {
    panic!("복구할 수 없는 상태: {message}");
}

// Option::unwrap과 같은 모양입니다. None 갈래의 타입은 !이므로 match 전체의 타입은 T가 됩니다.
fn my_unwrap<T>(value: Option<T>) -> T {
    match value {
        Some(v) => v,
        None => fail("값이 없습니다"),
    }
}

// 2. ?Sized: T 자리에 크기를 컴파일 시점에 모르는 타입(str, [T], dyn Trait)도 올 수 있게 합니다.
//    T를 직접 받을 수는 없으므로 &T로 받습니다.
fn byte_size<T: ?Sized>(value: &T) -> usize {
    size_of_val(value) // 참조가 가리키는 값의 실제 크기
}

struct Celsius(f64);

impl Display for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}°C", self.0)
    }
}

fn main() {
    println!("1. never 타입");
    let inputs = ["10", "스물", "30"];
    let mut sum = 0;
    for text in inputs {
        // Ok 갈래는 u32, Err 갈래의 continue는 !입니다. !는 어떤 타입으로도 바뀔 수 있으므로 number는 u32입니다.
        let number: u32 = match text.parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        sum += number;
    }
    println!("    숫자만 더한 값: {sum}"); // 출력:     숫자만 더한 값: 40

    // if의 두 갈래도 타입이 같아야 하지만, panic!(타입 !)은 어느 쪽에도 놓일 수 있습니다.
    let divisor = 4;
    let half: i32 = if divisor != 0 {
        100 / divisor
    } else {
        panic!("0으로 나눌 수 없습니다")
    };
    println!("    100 / {divisor} = {half}"); // 출력:     100 / 4 = 25
    println!("    my_unwrap(Some(7)) = {}", my_unwrap(Some(7))); // 출력:     my_unwrap(Some(7)) = 7

    println!("2. 동적 크기 타입은 참조(포인터) 뒤에 둡니다");
    // 참조가 가리키는 str 값의 크기는 문자열마다 다릅니다. 그래서 str은 크기가 정해지지 않은 타입입니다.
    let short: &str = "hi";
    let long: &str = "안녕하세요";
    println!(
        "    가리키는 str의 크기: short = {}, long = {}",
        byte_size(short),
        byte_size(long)
    ); // 출력:     가리키는 str의 크기: short = 2, long = 15
    // 반면 참조 자체의 크기는 일정합니다. &str은 (주소, 길이) 두 칸이라 &i32의 두 배입니다.
    println!(
        "    &i32: {}, &str: {}, &[i32]: {}, &dyn Display: {}, Box<str>: {}",
        size_of::<&i32>(),
        size_of::<&str>(),
        size_of::<&[i32]>(),
        size_of::<&dyn Display>(),
        size_of::<Box<str>>()
    ); // 출력:     &i32: 8, &str: 16, &[i32]: 16, &dyn Display: 16, Box<str>: 16

    println!("3. ?Sized 바운드 덕분에 크기가 정해진 타입과 아닌 타입을 모두 받습니다");
    let numbers = [1, 2, 3];
    let slice: &[i32] = &numbers;
    let shown: &dyn Display = &Celsius(36.5);
    println!("    byte_size(&5_i32)  = {}", byte_size(&5_i32)); // 출력:     byte_size(&5_i32)  = 4
    println!("    byte_size(slice)   = {}", byte_size(slice)); // 출력:     byte_size(slice)   = 12
    println!("    byte_size(\"hello\") = {}", byte_size("hello")); // 출력:     byte_size("hello") = 5
    println!("    byte_size({shown})  = {}", byte_size(shown)); // 출력:     byte_size(36.5°C)  = 8
}
