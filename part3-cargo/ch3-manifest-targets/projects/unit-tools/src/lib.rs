//! 단위를 바꾸는 함수 모음입니다.
//!
//! 패키지 이름은 `unit-tools`이지만 `Cargo.toml`의 `[lib] name = "units"` 때문에
//! 크레이트 이름은 `units`입니다. 코드에서는 `units::c_to_f`처럼 씁니다.

/// 섭씨를 화씨로 바꿉니다.
///
/// ```
/// assert_eq!(units::c_to_f(100.0), 212.0);
/// ```
pub fn c_to_f(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

/// 화씨를 섭씨로 바꿉니다.
pub fn f_to_c(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// 킬로미터를 마일로 바꿉니다.
pub fn km_to_mi(km: f64) -> f64 {
    km / 1.609_344
}

/// 명령줄 인자를 숫자로 읽습니다. 인자가 없으면 `default`를 돌려줍니다.
///
/// # Errors
///
/// 인자가 숫자가 아니면 사람이 읽을 메시지를 돌려줍니다.
pub fn number_or(arg: Option<String>, default: f64) -> Result<f64, String> {
    match arg {
        Some(text) => text.parse().map_err(|_| format!("숫자가 아닙니다: {text}")),
        None => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freezing_point() {
        assert!((c_to_f(0.0) - 32.0).abs() < 1e-9);
        assert!(f_to_c(32.0).abs() < 1e-9);
    }
}
