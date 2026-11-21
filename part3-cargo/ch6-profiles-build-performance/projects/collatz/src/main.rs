//! `1..=N`에서 콜라츠 수열이 가장 긴 시작 값을 두 방법으로 구하고 걸린 시간을 출력합니다.
//! 같은 소스를 프로필만 바꿔 빌드해 실행 파일 크기와 속도를 비교하는 실습용입니다.
//!
//! 실행: cargo run --release -- [N]     (N을 생략하면 1000000)
//! 시간(ms) 값은 실행할 때마다, 컴퓨터마다 달라집니다.

use std::time::Instant;

#[derive(Debug, thiserror::Error)]
enum ArgError {
    #[error("범위의 끝은 1 이상의 정수여야 합니다: {0:?}")]
    NotPositive(String),
}

fn parse_limit(arg: Option<String>) -> Result<u64, ArgError> {
    match arg {
        None => Ok(1_000_000),
        Some(s) => match s.parse::<u64>() {
            Ok(n) if n >= 1 => Ok(n),
            _ => Err(ArgError::NotPositive(s)),
        },
    }
}

fn main() {
    let limit = match parse_limit(std::env::args().nth(1)) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("에러: {e}");
            std::process::exit(2);
        }
    };
    println!("범위: 1..={limit}");

    let t = Instant::now();
    let (start, steps) = chain::longest(limit);
    report("일반 함수:  ", start, steps, t);

    let t = Instant::now();
    let (start, steps) = chain::longest_by(limit, |n| {
        if n.is_multiple_of(2) {
            n / 2
        } else {
            3 * n + 1
        }
    });
    report("제네릭 함수:", start, steps, t);
}

fn report(label: &str, start: u64, steps: u32, t: Instant) {
    println!(
        "{label} {start} ({steps}단계), {} ms",
        t.elapsed().as_millis()
    );
}
