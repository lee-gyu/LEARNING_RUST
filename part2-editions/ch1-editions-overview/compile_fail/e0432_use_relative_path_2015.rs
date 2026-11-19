// 기대하는 결과: 2015 에디션에서 실패(E0432), 2018·2021·2024 에디션에서 통과
// 핵심 메시지: error[E0432]: unresolved import `circle`
//
// rustc는 --edition을 적지 않으면 2015 에디션으로 컴파일합니다(Part 0 ch1). 이 파일은 2018 에디션
// 이후의 모듈 규칙(Part 0 ch7)으로 쓴 평범한 코드인데, --edition을 빠뜨리면 2015 에디션의 규칙으로
// 읽혀 에러가 납니다. 2015 에디션에서 use 경로는 언제나 크레이트 루트에서 시작하므로 `circle`을
// 크레이트 루트에서 찾다가 실패합니다. 2018 에디션부터는 현재 모듈(shapes)에 있는 이름으로
// 시작할 수 있습니다. 에러 메시지에 "에디션"이라는 말이 나오지 않는다는 점에 주의합니다.
//
// 확인(챕터 디렉터리에서):
//   rustc compile_fail/e0432_use_relative_path_2015.rs --out-dir /tmp/rust-p2-ch1                 → 에러(2015)
//   rustc --edition 2024 compile_fail/e0432_use_relative_path_2015.rs --out-dir /tmp/rust-p2-ch1  → 통과
//   rustc --edition 2018 compile_fail/e0432_use_relative_path_2015.rs --out-dir /tmp/rust-p2-ch1  → 통과

mod shapes {
    mod circle {
        pub fn area(radius: u32) -> u32 {
            3 * radius * radius // 원주율을 3으로 어림합니다
        }
    }

    use circle::area; // → 2015 에디션에서 E0432

    pub fn report() -> String {
        format!("반지름 2인 원의 넓이: 약 {}", area(2))
    }
}

fn main() {
    println!("{}", shapes::report());
}
