// 기대하는 에러: E0432
//   error[E0432]: unresolved import `crate::count`
//   no `count` in the root
//
// `src/main.rs`에서 같은 패키지의 라이브러리(`src/lib.rs`)에 있는 함수를
// `crate::`로 가져오려 한 실수를 파일 하나로 줄인 것입니다.
//
// 패키지 하나에 `src/lib.rs`와 `src/main.rs`가 함께 있으면 크레이트는 둘입니다.
// `crate::`는 "지금 컴파일하는 크레이트의 루트"라서, `main.rs` 안에서는 바이너리 크레이트
// 자신을 가리킵니다. 라이브러리 크레이트는 외부 크레이트처럼 이름으로 가져옵니다.
//
//     use text_stat::count;   // 패키지 text-stat의 라이브러리 크레이트
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0432_crate_is_not_the_library.rs

use crate::count;

fn main() {
    println!("{}", count("a b").words);
}
