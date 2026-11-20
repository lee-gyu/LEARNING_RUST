//! 두 래퍼가 함께 쓰는 데모 프로그램입니다. unsafe가 한 줄도 없습니다.
//!
//! `handwritten/`(손으로 쓴 선언)과 `generated/tally`(bindgen이 만든 선언)가 이 파일을
//! `[[bin]]`으로 빌드합니다. 두 래퍼는 이름(`tally`)과 공개 API가 같으므로 같은 코드가
//! 양쪽에서 컴파일됩니다. 출력을 나란히 놓으면, 선언의 출처만 다른 빌드가 C 라이브러리의
//! 판에 따라 어떻게 달라지는지 보입니다.
//!
//! 실행(C 소스의 판은 환경 변수 TALLY_C_VERSION으로 고름, 기본 1):
//!   handwritten/에서  cargo run
//!   generated/에서    cargo run -p tally                          (커밋해 둔 생성 결과)
//!                     cargo run -p tally --features tally-sys/bindgen  (빌드 때 생성)

use std::process::ExitCode;
use tally::{Config, Declarations, Tally, TallyError};

fn main() -> ExitCode {
    println!(
        "[C 소스 v{} · {}]",
        tally::C_VERSION,
        tally::DECLARATIONS.label()
    );

    // v3은 tally_record의 인자 수를 바꿨습니다. v3 헤더에서 생성하지 않은 선언으로 빌드했다면
    // 인자 2개짜리 선언으로 인자 3개짜리 C 함수를 부르게 되고, 그것은 미정의 동작입니다.
    // 이 데모는 그런 빌드를 실행하지 않고 멈춥니다(빌드가 조용히 통과했다는 것까지만 보입니다).
    if tally::C_VERSION == "3" && tally::DECLARATIONS != Declarations::Generated {
        eprintln!("선언이 v3 헤더와 맞지 않습니다(tally_record의 인자 수가 다름).");
        eprintln!("실행하면 미정의 동작이므로 여기서 멈춥니다.");
        return ExitCode::from(2);
    }

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("에러: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), TallyError> {
    println!("1. 헤더의 #define을 옮긴 상수");
    println!(
        "   MAX_BUCKETS = {}, NAME_MAX = {}",
        tally::MAX_BUCKETS,
        tally::NAME_MAX
    );

    println!("2. C가 값으로 돌려준 요약 (표본 12개)");
    let config = Config {
        low: 0,
        bucket_width: 50,
        bucket_count: 4,
    };
    let mut t = Tally::new(config)?;
    for ms in [12, 48, 51, 77, 99, 130, -3, 250, 0, 49, 199, 200] {
        t.record(ms);
    }
    // Rust 쪽 구조체 정의로 읽은 값과, C가 자기 구조체를 읽어 만든 문자열을 나란히 봅니다.
    println!("   Rust가 읽은 값: {:?}", t.summary());
    let table = t.render()?;
    println!("   C가 쓴 표 머리: {}", table.lines().next().unwrap_or(""));

    println!("3. 이름을 NAME_MAX 바이트에 맞춰 잘라 넣기");
    let wanted = "응답 시간(밀리초, 하루 평균)";
    // NAME_MAX 이하의 가장 가까운 글자 경계에서 자릅니다(UTF-8 글자를 반으로 자르지 않음).
    let cut = &wanted[..wanted.floor_char_boundary(tally::NAME_MAX)];
    println!("   {cut:?} ({}바이트) → {:?}", cut.len(), t.set_name(cut));

    println!("4. C의 상태 코드를 Result로");
    println!("   빈 이름: {:?}", t.set_name(""));
    let long = t.set_name(&"가".repeat(20));
    println!("   60바이트 이름: {long:?}");
    if let Err(e) = long {
        println!("   Display: {e}");
    }
    Ok(())
}
