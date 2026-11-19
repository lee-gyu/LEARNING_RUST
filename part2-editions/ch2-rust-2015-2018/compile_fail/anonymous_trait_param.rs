// 기대하는 결과: 2018·2021·2024 에디션에서 컴파일 에러, 2015 에디션에서는 경고(anonymous_parameters)와 함께 통과
// 핵심 메시지: expected one of `:`, `@`, or `|`, found `)`
//              note: anonymous parameters are removed in the 2018 edition (see RFC 1685)
//
// 2015 에디션에서는 본문이 없는 트레이트 메서드 선언에 매개변수 이름을 빼고 타입만 적을 수 있었습니다.
// 2018 에디션부터는 모든 매개변수에 이름(패턴)이 있어야 하므로, 파서가 f64를 매개변수 이름으로 읽고
// 그 뒤에 와야 할 `: 타입`을 찾다가 `)`를 만나 에러를 냅니다.
// 고치는 방법: `fn scale(&self, factor: f64) -> f64;`처럼 이름을 붙이거나, 이름이 필요 없으면 `_: f64`.
//
// 확인(실패): rustc --edition 2024 compile_fail/anonymous_trait_param.rs --out-dir /tmp/rust-p2-ch2
// 확인(통과): rustc --edition 2015 compile_fail/anonymous_trait_param.rs --out-dir /tmp/rust-p2-ch2

trait Shape {
    fn scale(&self, f64) -> f64; // → 2018부터 에러
}

struct Square(f64);

impl Shape for Square {
    fn scale(&self, factor: f64) -> f64 {
        self.0 * factor
    }
}

fn main() {
    println!("{}", Square(2.0).scale(3.0));
}
