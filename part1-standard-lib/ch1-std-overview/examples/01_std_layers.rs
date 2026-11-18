// 이 예제가 보여 주는 것: 표준 라이브러리는 `core` → `alloc` → `std` 세 층으로 되어 있고,
// `std`는 아래 두 크레이트의 항목을 재내보내기(re-export)하므로 같은 타입이 여러 경로로 보입니다.
// 실행: cargo run -p p1-ch1-std-overview --example 01_std_layers

// `core`와 `std`는 경로에 바로 쓸 수 있지만, `alloc`은 이 선언이 있어야 경로로 쓸 수 있습니다.
// `extern crate`는 "이 크레이트를 내 크레이트에 연결해 이름으로 쓰겠다"는 선언입니다.
// 보통의 프로그램은 `std`를 통해 같은 항목을 쓰므로 이 줄이 필요 없습니다.
// (`std` 없이 `core`/`alloc`만 쓰는 `#![no_std]` 프로그램은 Part 8 `ch1-intro-no-std`에서 다룹니다.)
extern crate alloc;

use std::any::{TypeId, type_name};

// 매개변수 타입을 `std` 경로로 적은 함수입니다.
fn describe(value: std::option::Option<i32>) -> &'static str {
    match value {
        Some(_) => "값 있음",
        None => "값 없음",
    }
}

fn main() {
    // ── 1. 같은 타입, 다른 경로 ──────────────────────────────────────────
    // `core`의 경로로 만든 값을 `std` 경로로 적은 매개변수에 그대로 넘길 수 있습니다.
    // 두 경로가 가리키는 것이 같은 타입이기 때문입니다(다른 타입이면 E0308).
    let from_core: core::option::Option<i32> = core::option::Option::Some(1);
    println!("describe(from_core) = {}", describe(from_core)); // 출력: describe(from_core) = 값 있음

    // `TypeId`는 타입마다 유일한 값입니다. 같으면 같은 타입입니다.
    let same_option =
        TypeId::of::<core::option::Option<i32>>() == TypeId::of::<std::option::Option<i32>>();
    println!("core Option == std Option ? {same_option}"); // 출력: core Option == std Option ? true

    let same_vec = TypeId::of::<alloc::vec::Vec<u8>>() == TypeId::of::<std::vec::Vec<u8>>();
    println!("alloc Vec == std Vec ? {same_vec}"); // 출력: alloc Vec == std Vec ? true

    // ── 2. 실제로 정의된 크레이트 엿보기 ──────────────────────────────────
    // `type_name`은 디버깅용으로 타입 이름을 문자열로 돌려줍니다.
    // 문서에 "정확한 내용과 형식은 정해져 있지 않고 컴파일러 버전에 따라 바뀔 수 있다"고 적혀 있으므로
    // 프로그램의 동작을 이 문자열에 기대면 안 됩니다. 여기서는 구경만 합니다(rustc 1.98.1 기준 출력).
    println!("{}", type_name::<Option<i32>>()); // 출력: core::option::Option<i32>
    println!("{}", type_name::<Vec<i32>>()); // 출력: alloc::vec::Vec<i32>
    println!("{}", type_name::<String>()); // 출력: alloc::string::String
    println!("{}", type_name::<std::collections::HashMap<String, i32>>()); // 출력: std::collections::hash::map::HashMap<alloc::string::String, i32>
    // `Option`은 힙도 운영체제도 필요 없으니 `core`에, `Vec`/`String`은 힙 할당이 필요하니 `alloc`에,
    // `HashMap`은 무작위 시드(운영체제가 주는 난수)가 필요하니 `std`에 정의되어 있습니다.

    // ── 3. 원시 타입의 메서드도 층마다 다릅니다 ────────────────────────────
    // `i32::pow`나 `f64::abs`는 `core`에 있으므로 운영체제가 없는 환경에서도 쓸 수 있습니다.
    println!(
        "2_i32.pow(10) = {}, (-2.7_f64).abs() = {}",
        2_i32.pow(10),
        (-2.7_f64).abs()
    ); // 출력: 2_i32.pow(10) = 1024, (-2.7_f64).abs() = 2.7

    // `f64::sqrt`, `f64::floor`처럼 플랫폼의 수학 라이브러리 호출로 구현될 수 있는 메서드는 `std`에서 추가됩니다.
    // (rustc 1.98.1 문서 기준으로 `core` 판 `f64` 페이지에는 `sqrt`도 `floor`도 없습니다.)
    println!("2.0_f64.sqrt() = {}", 2.0_f64.sqrt()); // 출력: 2.0_f64.sqrt() = 1.4142135623730951

    // 슬라이스의 `sort`는 보조 버퍼를 할당하므로 `alloc`에서, 할당이 필요 없는 `sort_unstable`은 `core`에서 옵니다.
    // 정렬 자체는 `ch4-collections`에서 다루므로 여기서는 결과만 봅니다.
    let mut a = [3, 1, 2];
    let mut b = [3, 1, 2];
    a.sort();
    b.sort_unstable();
    println!("sort = {a:?}, sort_unstable = {b:?}"); // 출력: sort = [1, 2, 3], sort_unstable = [1, 2, 3]

    // ── 4. `std`에만 있는 것: 운영체제가 필요한 기능 ───────────────────────
    // 환경 변수, 파일, 스레드, 시간 등은 운영체제에 기대므로 `std`에만 있습니다.
    // 예를 들어 스레드는 운영체제가 만들어 주므로 `std::thread`에만 있습니다.
    println!("지금 스레드 이름: {:?}", std::thread::current().name()); // 출력: 지금 스레드 이름: Some("main")
}
