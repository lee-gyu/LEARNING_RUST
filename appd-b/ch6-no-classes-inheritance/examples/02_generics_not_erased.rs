// Q6.6 — 타입 인자마다 따로 만들어지는 제네릭
//
// Java의 제네릭은 컴파일 뒤 타입 인자를 지웁니다(타입 소거). 그래서 Java 튜토리얼의 "Restrictions on
// Generics"는 이런 일을 할 수 없다고 적습니다.
//
//   - 타입 매개변수로 새 값을 만들기:            new T()
//   - 기본형을 타입 인자로 쓰기:                List<int>
//   - 소거하면 같아지는 매개변수로 오버로딩하기:   f(List<String>)과 f(List<Integer>)
//
// Rust는 제네릭 코드를 쓰인 구체 타입마다 따로 만들고(단형성화, Part 0 ch10 2.4절), 컴파일러가 타입 인자까지
// 포함한 전체 타입으로 `impl`을 고르므로, 셋에 대응하는 일을 할 수 있습니다.
//
//   1. `T::default()`로 새 값을 만들고(`u8` 같은 기본형도 그대로 타입 인자), `size_of::<T>()`로 크기를 읽음
//   2. 타입 인자만 다른 `Vec<u8>`과 `Vec<String>`에 트레이트를 따로 구현
//
// 실행: cargo run -p faq-ch6-no-classes-inheritance --example 02_generics_not_erased

use std::fmt::Debug;
use std::mem::size_of;

/// 어떤 타입이든 기본값을 새로 만들어, 그 값과 타입의 크기를 보여 줍니다.
fn fresh<T: Default + Debug>() -> String {
    let value = T::default(); // Java의 new T() 자리
    format!("{value:?} ({}바이트)", size_of::<T>())
}

trait Describe {
    fn describe(&self) -> String;
}

// 타입 인자만 다른 두 타입에 서로 다른 구현을 줍니다.
impl Describe for Vec<u8> {
    fn describe(&self) -> String {
        format!("바이트 {}개", self.len())
    }
}

impl Describe for Vec<String> {
    fn describe(&self) -> String {
        format!("문자열 {}개: {}", self.len(), self.join(", "))
    }
}

fn report<T: Describe>(value: &T) {
    println!("  {}", value.describe());
}

fn main() {
    println!("1. 타입 인자로 새 값 만들기");
    println!("  u8: {}", fresh::<u8>());
    println!("  u64: {}", fresh::<u64>());
    println!("  String: {}", fresh::<String>());
    // 출력:   u8: 0 (1바이트)
    // 출력:   u64: 0 (8바이트)
    // 출력:   String: "" (24바이트)
    // (String의 24바이트는 64비트 환경에서 포인터·길이·용량을 담는 크기입니다. 내용은 힙에 있습니다.)

    println!("2. 타입 인자만 다른 타입에 다른 구현");
    report(&vec![0x52u8, 0x75, 0x73, 0x74]);
    report(&vec!["가".to_string(), "나".to_string()]);
    // 출력:   바이트 4개
    // 출력:   문자열 2개: 가, 나
}
