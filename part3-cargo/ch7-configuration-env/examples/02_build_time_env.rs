//! 직접 정한 환경 변수를 컴파일할 때(`option_env!`)와 실행할 때(`std::env::var`) 읽어,
//! 두 값이 언제 같고 언제 달라지는지 봅니다.
//!
//! 실행(README 3절의 순서대로):
//!   cargo run -q -p p3-ch7-configuration-env --example 02_build_time_env
//!   GREETING=안녕 cargo run -q -p p3-ch7-configuration-env --example 02_build_time_env
//!   GREETING=잘가 ./target/debug/examples/02_build_time_env

fn main() {
    // `option_env!`는 변수가 없으면 컴파일 에러 대신 `None`이 됩니다(`env!`는 에러).
    // 읽은 값은 실행 파일에 박히고, Cargo는 이 변수를 기억해 두었다가 값이 바뀌면 다시 컴파일합니다.
    let built = option_env!("GREETING").unwrap_or("(없음)");
    let now = std::env::var("GREETING").unwrap_or_else(|_| "(없음)".to_string());
    println!("GREETING 컴파일 시: {built}");
    println!("GREETING 실행 시:   {now}");

    // 출력(GREETING 없이 cargo run):
    // 출력: GREETING 컴파일 시: (없음)
    // 출력: GREETING 실행 시:   (없음)
    // 출력(GREETING=안녕 cargo run):
    // 출력: GREETING 컴파일 시: 안녕
    // 출력: GREETING 실행 시:   안녕
    // 출력(이어서 GREETING=잘가로 실행 파일을 직접 실행):
    // 출력: GREETING 컴파일 시: 안녕
    // 출력: GREETING 실행 시:   잘가
}
