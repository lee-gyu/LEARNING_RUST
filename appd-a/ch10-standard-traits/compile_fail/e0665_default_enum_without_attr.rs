// 기대하는 에러: E0665 — `#[derive(Default)]` on enum with no `#[default]`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0665_default_enum_without_attr.rs --out-dir /tmp/er-ch10
//
// 구조체는 필드마다 default()를 부르면 되지만, 열거형은 어느 배리언트가 기본값인지
// 컴파일러가 정할 수 없습니다. 배리언트를 선언한 순서로 정해 주지도 않습니다.
// 해결: 기본값으로 삼을 배리언트에 #[default]를 붙입니다(examples/05_default_meaningful.rs의 Orientation).
//       고를 배리언트가 없다면, 그 열거형에는 Default를 구현하지 않는 것이 맞습니다.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Orientation {
    Portrait,
    Landscape,
}

fn main() {
    println!("{:?} {:?}", Orientation::default(), Orientation::Landscape);
}
