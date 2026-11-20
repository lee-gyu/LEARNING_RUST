//! tally 래퍼를 쓰는 프로그램입니다. unsafe가 한 줄도 없습니다.
//!
//! 실행: cargo run -q (이 디렉터리에서)

use std::thread;
use tally::{Config, Tally, TallyError};

fn main() -> Result<(), TallyError> {
    println!("1. C가 값으로 돌려준 기본 설정");
    println!("   {:?}", Config::default());

    println!("2. 응답 시간 12개를 기록");
    let config = Config {
        low: 0,
        bucket_width: 50,
        bucket_count: 4,
    };
    let mut latency = Tally::new(config)?;
    for ms in [12, 48, 51, 77, 99, 130, -3, 250, 0, 49, 199, 200] {
        latency.record(ms);
    }
    println!("   {:?}", latency.summary());

    println!("3. 이름: 소유한 문자열을 넘기고(CString), 빌린 문자열을 받음(&CStr)");
    println!("   바꾸기 전: {:?}", latency.name());
    latency.set_name("응답 시간(ms)")?;
    println!("   바꾼 뒤:   {:?}", latency.name().to_str());

    println!("4. C가 할당한 표 → String으로 복사 → tally_string_free로 해제");
    print!("{}", latency.render()?);

    println!("5. NULL과 상태 코드를 Result로");
    let bad = Config {
        bucket_width: 0,
        ..config
    };
    println!("   너비 0인 설정: {:?}", Tally::new(bad).err());
    let long = latency.set_name("서른한 바이트를 넘는 아주 긴 이름");
    println!("   긴 이름: {long:?}");
    let nul = latency.set_name("응답\0시간");
    println!("   중간의 0 바이트: {nul:?}");
    if let Err(e) = nul {
        println!("   Display: {e}");
    }
    println!("   이름은 그대로: {:?}", latency.name().to_str());

    println!("6. Send: 다른 스레드로 옮겨 쓰고 돌려받기");
    let handle = thread::spawn(move || {
        latency.record(120);
        latency
    });
    let latency = handle.join().expect("작업 스레드가 패닉했습니다");
    println!(
        "   스레드에서 하나 더 기록한 뒤 count = {}",
        latency.summary().count
    );
    Ok(())
    // latency가 여기서 버려지며 tally_free가 불립니다.
}
