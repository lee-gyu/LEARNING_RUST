// 기대하는 에러: 에러 코드 없음
//   error: expected one of `::`, `;`, or `as`, found `-`
//
// 패키지 이름 `text-stat`을 코드에 그대로 쓴 실수입니다.
// Rust 식별자에는 `-`를 쓸 수 없어서 `text`까지만 이름으로 읽히고, 그 뒤의 `-`는
// `::`나 `;`, `as`가 와야 할 자리에 온 엉뚱한 글자가 됩니다.
// Cargo는 패키지 이름의 `-`를 `_`로 바꿔 크레이트 이름을 만들므로 `text_stat`이라고 써야 합니다.
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/hyphen_in_crate_name.rs
//
// 이 파일은 문법 단계에서 실패하므로 `text_stat` 크레이트가 없어도 같은 에러를 볼 수 있습니다.

use text-stat::count;

fn main() {
    println!("{}", count("a b").words);
}
