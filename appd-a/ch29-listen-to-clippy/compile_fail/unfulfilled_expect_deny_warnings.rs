// 기대하는 에러: 에러 코드 없음 — error: this lint expectation is unfulfilled
//   (`#[deny(unfulfilled_lint_expectations)]` implied by `#[deny(warnings)]`)
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/unfulfilled_expect_deny_warnings.rs
//
// 처음에는 `total`을 쓰지 않아서 `#[expect(unused_variables)]`를 붙였습니다. 그 뒤 출력에 `total`을
// 쓰도록 고쳤는데 속성은 그대로 남았습니다. `#[allow]`였다면 아무 말도 없었겠지만, `#[expect]`는
// "기대한 경고가 나지 않았다"고 알려 줍니다. 경고를 에러로 올린 빌드(CI의 `-D warnings`와 같음)에서는
// 그 알림도 에러가 되어 빌드가 멈춥니다. 린트 에러라서 에러 코드가 없습니다(README 4.3절).
#![deny(warnings)]

fn main() {
    #[expect(unused_variables, reason = "합계 출력은 다음 단계에서 붙임")]
    let total = 3 + 4;
    println!("합계: {total}");
}
