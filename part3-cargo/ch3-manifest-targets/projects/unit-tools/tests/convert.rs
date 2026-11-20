//! 관례로 찾은 통합 테스트입니다. 테스트 러너(libtest)가 `#[test]` 함수를 모아 실행합니다.

#[test]
fn round_trip() {
    let celsius = 36.5;
    let back = units::f_to_c(units::c_to_f(celsius));
    assert!((back - celsius).abs() < 1e-9);
}

#[test]
fn marathon_in_miles() {
    assert!((units::km_to_mi(42.195) - 26.219).abs() < 0.001);
}
