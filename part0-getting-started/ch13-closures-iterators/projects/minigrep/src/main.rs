//! minigrep의 바이너리 크레이트입니다(ch13 개선판).
//!
//! 인자 반복자를 라이브러리에 넘기고, 에러가 나면 알린 뒤 종료 코드 1로 끝내는 일만 합니다.
//! 로직은 모두 src/lib.rs에 있습니다.

use std::env;
use std::process;

// 같은 패키지의 라이브러리 크레이트는 패키지 이름(minigrep)으로 가리킵니다.
use minigrep::Config;

fn main() {
    // env::args()가 돌려주는 반복자를 벡터에 모으지 않고 그대로 넘깁니다. 반복자의 소유권이
    // build로 옮겨 가고, build는 거기서 String을 하나씩 꺼내 Config로 옮깁니다.
    //
    // unwrap_or_else는 Ok면 안의 Config를 꺼내고, Err일 때만 클로저에 에러 값을 넘겨 부릅니다.
    // 클로저는 process::exit로 끝나므로 돌아오지 않습니다(반환 타입 !). 그래서 Config를
    // 돌려주지 않아도 "Config를 돌려주는 FnOnce" 자리에 맞습니다.
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        // 에러 메시지는 표준 에러(eprintln!)로 보냅니다. 표준 출력을 파일로 돌려도
        // 이 메시지는 화면에 남고, 파일에는 검색 결과만 들어갑니다.
        eprintln!("인자 문제: {err}");
        eprintln!("사용법: minigrep <검색어> <파일 경로>");
        eprintln!("(대소문자를 무시하려면 환경 변수 IGNORE_CASE를 설정합니다)");
        process::exit(1);
    });

    // run은 성공해도 꺼낼 값이 ()뿐이므로, Err인 경우만 골라내는 if let이 더 알맞습니다.
    if let Err(e) = minigrep::run(config) {
        eprintln!("실행 오류: {e}");
        process::exit(1);
    }
}
