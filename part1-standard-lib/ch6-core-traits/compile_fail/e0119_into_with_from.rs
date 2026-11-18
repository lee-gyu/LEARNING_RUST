// 기대하는 에러: error[E0119]: conflicting implementations of trait `Into<Seconds>` for type `Minutes`
// 핵심 메시지: note: conflicting implementation in crate `core`:
//                     - impl<T, U> Into<U> for T
//                       where U: From<T>;
//
// From<Minutes> for Seconds를 구현하는 순간, 표준 라이브러리의 포괄 구현이
// Into<Seconds> for Minutes를 이미 만들어 둡니다. 같은 (트레이트, 타입) 쌍에 구현이 둘이 되므로 거부됩니다.
// 해결: Into 구현을 지웁니다. From 하나면 into()도 쓸 수 있습니다(examples/01_from_into.rs).
//
// 확인: rustc --edition 2024 compile_fail/e0119_into_with_from.rs --out-dir <임시 디렉터리>

struct Minutes(u64);
struct Seconds(u64);

impl From<Minutes> for Seconds {
    fn from(m: Minutes) -> Self {
        Seconds(m.0 * 60)
    }
}

impl Into<Seconds> for Minutes {
    fn into(self) -> Seconds {
        Seconds(self.0 * 60)
    }
}

fn main() {
    let s: Seconds = Minutes(2).into();
    println!("{}초", s.0);
}
