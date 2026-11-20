//! 빌드 스크립트: C 프로그램(csrc/driver.c)을 정적 라이브러리로 컴파일해 이 실행 파일에 링크합니다.
//! driver.c는 Rust가 내보낸 textstat_* 함수를 이름으로 부르므로, 링커가 그 이름을 이 크레이트의
//! #[unsafe(no_mangle)] 함수에서 찾습니다.

fn main() {
    // cc가 환경 변수에 대한 rerun-if-env-changed를 출력하므로, C 파일도 직접 알려 줍니다.
    println!("cargo::rerun-if-changed=csrc/driver.c");
    println!("cargo::rerun-if-changed=csrc/textstat.h");

    cc::Build::new()
        .file("csrc/driver.c")
        .include("csrc")
        .warnings_into_errors(true)
        .compile("textstat_driver");
}
