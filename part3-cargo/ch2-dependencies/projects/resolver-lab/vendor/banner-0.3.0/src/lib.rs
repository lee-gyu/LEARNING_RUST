//! 연습용 레지스트리의 가상 크레이트 `banner` 0.3.0입니다. `tint = "1.1"`에 의존합니다.

/// `banner`가 링크된 `tint`의 버전입니다.
pub fn tint_version() -> &'static str {
    tint::VERSION
}

/// 색 코드를 앞에 붙인 한 줄을 만듭니다. 모르는 색이면 코드 자리에 `?`를 넣습니다.
pub fn line(color: &str, text: &str) -> String {
    // `yellow`는 tint 1.1.0에서 생겼습니다. 그래서 요구 사항이 "1.0"이 아니라 "1.1"입니다.
    let code = tint::hex(color).unwrap_or("?");
    format!("[{code}] {text}")
}
