//! 빌드 스크립트(build.rs)가 data/units.txt로 만든 코드를 `include!`로 가져다 씁니다.
//!
//! 실행:
//!   cargo run -q -p p3-ch8-build-scripts --example 01_generated_units
//!   cargo run -q -p p3-ch8-build-scripts --example 01_generated_units --features extra-units

// OUT_DIR은 빌드 스크립트가 결과를 둔 디렉터리입니다. 빌드 스크립트가 있는 패키지를 컴파일할 때만 있습니다.
// 만든 파일의 내용을 이 자리에 붙여 넣은 것처럼 컴파일됩니다.
include!(concat!(env!("OUT_DIR"), "/units.rs"));

fn convert(value: f64, from: &str, to: &str) -> Option<f64> {
    Some(value * factor(from)? / factor(to)?)
}

fn main() {
    let names: Vec<&str> = UNITS.iter().map(|(n, _)| *n).collect();
    println!("단위 {}개: {}", UNITS.len(), names.join(", "));
    for (value, from, to) in [(5.0, "km", "m"), (250.0, "cm", "m"), (1.0, "mi", "km")] {
        match convert(value, from, to) {
            Some(v) => println!("{value} {from} = {v} {to}"),
            None => println!("{value} {from} → {to}: 모르는 단위"),
        }
    }

    // 출력(기본):
    // 출력: 단위 4개: mm, cm, m, km
    // 출력: 5 km = 5000 m
    // 출력: 250 cm = 2.5 m
    // 출력: 1 mi → km: 모르는 단위
    // 출력(--features extra-units):
    // 출력: 단위 7개: mm, cm, m, km, in, ft, mi
    // 출력: 5 km = 5000 m
    // 출력: 250 cm = 2.5 m
    // 출력: 1 mi = 1.609344 km
}
