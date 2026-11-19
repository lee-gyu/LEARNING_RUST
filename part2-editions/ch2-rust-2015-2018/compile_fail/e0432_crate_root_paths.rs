// 기대하는 결과: 2018·2021·2024 에디션에서 E0432와 E0433으로 실패, 2015 에디션에서는 경고 없이 통과
// 핵심 메시지: error[E0432]: unresolved import `units`
//              note: `use` statements changed in Rust 2018
//              error[E0433]: cannot find `units` in the list of imported crates
//
// 2015 에디션의 두 규칙을 2018 이후 에디션으로 컴파일하면 생기는 에러입니다.
// - use 안의 경로는 크레이트 루트에서 출발했습니다. 그래서 하위 모듈 report 안에서도 `use units::...`가
//   크레이트 루트의 units 모듈을 찾았습니다. 2018부터 use 경로는 현재 모듈(report)에서 출발하므로
//   report 안에 units가 없어 E0432가 납니다.
// - ::로 시작하는 경로도 크레이트 루트에서 출발했습니다. 2018부터 ::는 외부 크레이트만 가리키므로
//   units라는 외부 크레이트를 찾다가 E0433이 납니다.
// 고치는 방법: 둘 다 crate::로 시작합니다. `use crate::units::to_celsius;`, `crate::units::SCALE`.
//
// 확인(실패): rustc --edition 2024 compile_fail/e0432_crate_root_paths.rs --out-dir /tmp/rust-p2-ch2
// 확인(통과): rustc --edition 2015 compile_fail/e0432_crate_root_paths.rs --out-dir /tmp/rust-p2-ch2

mod units {
    pub const SCALE: &str = "섭씨";

    pub fn to_celsius(f: f64) -> f64 {
        (f - 32.0) / 1.8
    }
}

mod report {
    use units::to_celsius; // → 2018부터 E0432

    pub fn line(f: f64) -> String {
        format!("{} {:.1}도", ::units::SCALE, to_celsius(f)) // → 2018부터 E0433
    }
}

fn main() {
    println!("{}", report::line(212.0));
}
