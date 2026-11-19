// 실행 파일은 dev-dependencies가 필요 없는 빌드 대상입니다.
fn main() {
    println!(
        "{} main: flags의 verbose = {}",
        env!("CARGO_PKG_NAME"),
        flags::verbose_enabled()
    );
}
