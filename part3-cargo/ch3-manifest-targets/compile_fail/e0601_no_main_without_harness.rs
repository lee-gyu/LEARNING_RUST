// 기대하는 에러: E0601
//   error[E0601]: `main` function not found in crate `e0601_no_main_without_harness`
//
// 통합 테스트에 `harness = false`를 적어 놓고 `#[test]` 함수만 둔 실수입니다.
//
//     [[test]]
//     name = "report"
//     harness = false
//
// 평소에는 Cargo가 테스트 타깃을 `rustc --test`로 컴파일하고, 그러면 컴파일러가
// `#[test]` 함수를 모아 실행하는 `main`을 대신 만들어 넣습니다(테스트 러너).
// `harness = false`는 그 `--test`를 빼라는 뜻이므로 `main`을 직접 써야 합니다.
// `rustc`로 이 파일을 그냥 컴파일하는 것이 바로 `--test`가 빠진 상황입니다.
// `[[bin]]`의 `path`가 `main`이 없는 파일을 가리킬 때도 같은 에러가 납니다.
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0601_no_main_without_harness.rs
// (`--test`를 붙여 컴파일하면 성공합니다.)

#[test]
fn boiling_point() {
    assert_eq!(100.0_f64 * 9.0 / 5.0 + 32.0, 212.0);
}
