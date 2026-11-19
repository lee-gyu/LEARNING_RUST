//! 기능(feature) `verbose` 하나를 가진 작은 라이브러리입니다.

/// 이 크레이트가 `verbose` 기능을 켠 채로 컴파일되었는지 돌려줍니다.
///
/// `cfg!(feature = "verbose")`는 컴파일할 때 `true`나 `false`로 정해집니다.
pub fn verbose_enabled() -> bool {
    cfg!(feature = "verbose")
}
