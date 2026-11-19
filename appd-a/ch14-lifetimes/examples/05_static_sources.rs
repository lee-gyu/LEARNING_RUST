//! 'static 참조는 어디서 오는가: 리터럴, static 항목, 상수 승격, 그리고 Box::leak.
//!
//! 실행: cargo run -p er-ch14-lifetimes --example 05_static_sources
//!
//! &'static T는 "프로그램이 끝날 때까지 유효한 참조"입니다. 그런 참조를 만들 수 있는 것은
//! 어느 스택 프레임에도 속하지 않아 drop되지 않는 데이터뿐입니다. 입력 참조가 없는 함수가
//! 참조를 돌려준다면, 돌려줄 수 있는 것은 이런 'static 참조뿐입니다.

/// static 항목: 프로그램 전체에서 위치가 하나로 정해진 값입니다.
static PLANETS: [&str; 4] = ["수성", "금성", "지구", "화성"];

/// const 항목: 이름 붙은 값일 뿐, 쓰는 곳마다 값이 복사될 수 있습니다.
const EARTH_DAYS: u32 = 365;

// 1. 문자열 리터럴은 실행 파일 안에 들어 있으므로 &'static str입니다.
fn orbit_unit() -> &'static str {
    "일"
}

// 2. static 항목을 빌린 참조는 'static입니다.
fn planet_name(index: usize) -> Option<&'static str> {
    PLANETS.get(index).copied()
}

// 3. 상수 승격: 컴파일할 때 값이 정해지고 Drop도 내부 가변성도 없는 표현식의 참조는
//    컴파일러가 정적 데이터로 올려 'static 참조로 만들어 줍니다.
fn earth_year() -> &'static u32 {
    &EARTH_DAYS
}

fn inner_orbits() -> &'static [u32] {
    &[88, 225, EARTH_DAYS, 687]
}

// 4. Box::leak: 힙의 값에서 소유자를 없애 버립니다. drop할 소유자가 없으므로 값은 프로그램이
//    끝날 때까지 남고, 그래서 'static 참조를 얻습니다. 메모리 안전하지만 되찾을 수 없는 누수입니다.
fn leak_label(probe_no: u32) -> &'static str {
    let label = format!("탐사선-{probe_no}호");
    Box::leak(label.into_boxed_str())
}

fn main() {
    println!("1. 문자열 리터럴");
    println!("    공전 주기의 단위: {}", orbit_unit());
    // 출력:     공전 주기의 단위: 일

    println!("2. static 항목");
    let names: Vec<&'static str> = (0..5).filter_map(planet_name).collect();
    println!("    {names:?}");
    // 출력:     ["수성", "금성", "지구", "화성"]

    println!("3. 상수 승격");
    println!("    지구의 1년: {}{}", earth_year(), orbit_unit());
    println!("    안쪽 행성의 공전 주기: {:?}", inner_orbits());
    // 함수 안에서 쓴 표현식도 승격됩니다. let의 임시값 수명 연장만으로는 'static이 될 수 없습니다.
    let doubled: &'static u32 = &(EARTH_DAYS * 2);
    println!("    두 해: {doubled}{}", orbit_unit());
    // 출력:
    //     지구의 1년: 365일
    //     안쪽 행성의 공전 주기: [88, 225, 365, 687]
    //     두 해: 730일

    println!("4. Box::leak (누수)");
    let mut leaked_bytes = 0;
    let mut labels: Vec<&'static str> = Vec::new();
    for no in 1..=3 {
        let label = leak_label(no);
        leaked_bytes += label.len();
        labels.push(label);
    }
    println!("    {labels:?}");
    println!("    되찾을 수 없게 된 바이트: {leaked_bytes} (부를 때마다 늘어납니다)");
    // 출력:
    //     ["탐사선-1호", "탐사선-2호", "탐사선-3호"]
    //     되찾을 수 없게 된 바이트: 42 (부를 때마다 늘어납니다)
}
