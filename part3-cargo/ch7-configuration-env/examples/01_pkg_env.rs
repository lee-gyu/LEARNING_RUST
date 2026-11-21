//! Cargo가 크레이트에 알려 주는 환경 변수를 컴파일할 때(`env!`)와 실행할 때(`std::env::var`) 각각 읽어 봅니다.
//!
//! 실행:
//!   cargo run -q -p p3-ch7-configuration-env --example 01_pkg_env
//!   ./target/debug/examples/01_pkg_env          (위 명령 뒤에, Cargo를 거치지 않고 직접 실행)

use std::path::Path;

/// 경로의 마지막 구성 요소만 남깁니다. 절대 경로의 앞부분은 컴퓨터마다 다릅니다.
fn last(p: &str) -> String {
    let name = Path::new(p)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned());
    format!("…/{}", name.unwrap_or_default())
}

/// 실행할 때 환경 변수를 읽습니다. 없으면 "(없음)"입니다.
fn at_run(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| "(없음)".to_string())
}

fn main() {
    // `env!`는 컴파일할 때 값을 읽어 문자열 리터럴로 바꿉니다. 실행 파일 안에 값이 박혀 있습니다.
    println!("[컴파일할 때 넣은 값]");
    println!("CARGO_PKG_NAME:        {}", env!("CARGO_PKG_NAME"));
    println!("CARGO_PKG_VERSION:     {}", env!("CARGO_PKG_VERSION"));
    println!(
        "  MAJOR/MINOR/PATCH/PRE: {}/{}/{}/{:?}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        env!("CARGO_PKG_VERSION_PRE"),
    );
    println!("CARGO_PKG_DESCRIPTION: {}", env!("CARGO_PKG_DESCRIPTION"));
    // 매니페스트에 없는 키는 변수가 없는 것이 아니라 빈 문자열입니다.
    println!("CARGO_PKG_HOMEPAGE:    {:?}", env!("CARGO_PKG_HOMEPAGE"));
    println!("CARGO_CRATE_NAME:      {}", env!("CARGO_CRATE_NAME"));
    println!("CARGO_BIN_NAME:        {}", env!("CARGO_BIN_NAME"));
    println!(
        "CARGO_MANIFEST_DIR:    {}",
        last(env!("CARGO_MANIFEST_DIR"))
    );

    // 같은 변수를 실행할 때 읽으면, `cargo run`으로 실행했을 때만 있습니다.
    println!("[실행할 때 환경에 있는 값]");
    println!("CARGO_PKG_NAME:        {}", at_run("CARGO_PKG_NAME"));
    let dir = at_run("CARGO_MANIFEST_DIR");
    let dir = if dir == "(없음)" { dir } else { last(&dir) };
    println!("CARGO_MANIFEST_DIR:    {dir}");

    // 출력(cargo run으로 실행):
    // 출력: [컴파일할 때 넣은 값]
    // 출력: CARGO_PKG_NAME:        p3-ch7-configuration-env
    // 출력: CARGO_PKG_VERSION:     0.0.0
    // 출력:   MAJOR/MINOR/PATCH/PRE: 0/0/0/""
    // 출력: CARGO_PKG_DESCRIPTION: Part 3 ch7: Cargo 설정과 환경 변수
    // 출력: CARGO_PKG_HOMEPAGE:    ""
    // 출력: CARGO_CRATE_NAME:      01_pkg_env
    // 출력: CARGO_BIN_NAME:        01_pkg_env
    // 출력: CARGO_MANIFEST_DIR:    …/ch7-configuration-env
    // 출력: [실행할 때 환경에 있는 값]
    // 출력: CARGO_PKG_NAME:        p3-ch7-configuration-env
    // 출력: CARGO_MANIFEST_DIR:    …/ch7-configuration-env
    // 출력(직접 실행): 위와 같고 마지막 두 줄만 다릅니다.
    // 출력: CARGO_PKG_NAME:        (없음)
    // 출력: CARGO_MANIFEST_DIR:    (없음)
}
