//! 통합 테스트에만 Cargo가 알려 주는 환경 변수와, `[env]`가 테스트 프로세스에도 들어오는지 확인합니다.

use std::path::Path;
use std::process::Command;

#[test]
fn runs_the_binary() {
    // CARGO_BIN_EXE_<바이너리 이름>: 통합 테스트를 컴파일할 때 Cargo가 알려 주는, 이 패키지 실행 파일의 절대 경로
    let exe = env!("CARGO_BIN_EXE_configured");
    let out = Command::new(exe).arg("테스트").output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("안녕하세요, 테스트\n"));
}

#[test]
fn env_table_reaches_tests() {
    // .cargo/config.toml의 [env]는 cargo test가 실행하는 테스트 프로세스에도 들어옵니다.
    let data = std::env::var("APP_DATA").expect("[env]의 APP_DATA");
    assert!(Path::new(&data).is_absolute());
    // CARGO_TARGET_TMPDIR: 통합 테스트가 자유롭게 파일을 둘 수 있는 target 디렉터리 안의 경로
    assert!(Path::new(env!("CARGO_TARGET_TMPDIR")).is_dir());
}
