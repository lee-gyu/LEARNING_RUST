// 예제(examples/)와 테스트는 dev-dependencies를 함께 빌드하는 대상입니다.
fn main() {
    println!(
        "{} 예제: flags의 verbose = {}",
        env!("CARGO_PKG_NAME"),
        flags::verbose_enabled()
    );
}
