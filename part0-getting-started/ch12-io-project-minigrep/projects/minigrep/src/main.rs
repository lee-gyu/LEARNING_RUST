//! minigrep의 바이너리 크레이트입니다.
//!
//! 인자를 모으고, 라이브러리에 설정 해석과 실행을 맡기고, 에러가 나면 알린 뒤
//! 종료 코드 1로 끝내는 일만 합니다. 로직은 모두 src/lib.rs에 있습니다.

use std::env;
use std::process;

// 같은 패키지의 라이브러리 크레이트는 패키지 이름(minigrep)으로 가리킵니다.
use minigrep::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = match Config::build(&args) {
        Ok(config) => config,
        Err(err) => {
            // 에러 메시지는 표준 에러(eprintln!)로 보냅니다. 표준 출력을 파일로 돌려도
            // 이 메시지는 화면에 남고, 파일에는 검색 결과만 들어갑니다.
            eprintln!("인자 문제: {err}");
            eprintln!("사용법: minigrep <검색어> <파일 경로>");
            eprintln!("(대소문자를 무시하려면 환경 변수 IGNORE_CASE를 설정합니다)");
            process::exit(1);
        }
    };

    if let Err(e) = minigrep::run(config) {
        eprintln!("실행 오류: {e}");
        process::exit(1);
    }
}
