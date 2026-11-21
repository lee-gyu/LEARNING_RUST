// 기대하는 에러: `unused_must_use` 린트를 deny로 올린 크레이트에서 `Result`를 버림(에러 코드 없음)
// 핵심 메시지: unused `Result` that must be used
//
// `save`가 실패를 `Result`로 돌려주는데, 부르는 쪽이 그 값을 쓰지 않고 버립니다. Java의 빈 `catch`나
// Go에서 `err`를 확인하지 않는 것과 같은 실수입니다. `Result`에는 `#[must_use]`가 붙어 있어 기본으로는
// 경고가 나고, 크레이트 맨 위의 `#![deny(unused_must_use)]`가 그 경고를 에러로 올렸습니다.
//
// 고치는 법: 실패를 처리하거나(`if let Err(e) = save("") { … }`), 호출한 쪽에 넘기거나(`save("")?`),
// 정말 무시해도 되면 무시한다는 것을 적습니다(`let _ = save("");`).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch5-no-null-no-exceptions compile_fail/unused_must_use_result.rs

#![deny(unused_must_use)]

fn save(text: &str) -> Result<(), String> {
    if text.is_empty() {
        Err("빈 내용은 저장할 수 없음".to_string())
    } else {
        Ok(())
    }
}

fn main() {
    save(""); // 실패했는지 확인하지 않고 버림
    println!("저장했습니다");
}
