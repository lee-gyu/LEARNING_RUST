// 기대하는 에러: E0592
// 핵심 메시지: duplicate definitions with name `new`
//
// Java·C#·C++처럼 매개변수만 다른 생성자를 두 개 만들려 한 코드입니다. Rust에는 오버로딩이 없어서, 한 타입의
// `impl` 블록 안에 같은 이름의 연관 함수를 둘 둘 수 없습니다.
//
// 고치는 법: 이름을 다르게 짓습니다(examples/03_overloading_alternatives.rs의 1번).
//   fn new(secs: u64) -> Timer { … }
//   fn with_label(label: &str, secs: u64) -> Timer { … }
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch6-no-classes-inheritance compile_fail/e0592_overloaded_new.rs

struct Timer {
    label: String,
    secs: u64,
}

impl Timer {
    fn new(secs: u64) -> Timer {
        Timer {
            label: String::from("타이머"),
            secs,
        }
    }

    // E0592: 같은 이름 `new`의 두 번째 정의
    fn new(label: &str, secs: u64) -> Timer {
        Timer {
            label: label.to_string(),
            secs,
        }
    }
}

fn main() {
    let timer = Timer::new(180);
    println!("{}: {}초", timer.label, timer.secs);
}
