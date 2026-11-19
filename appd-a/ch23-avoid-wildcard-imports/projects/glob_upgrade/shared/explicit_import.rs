// 사용자 프로그램, 명시적 가져오기 판(권장). templog 1.0.0에 맞춰 작성했습니다.
// glob_import.rs와는 아래 `use templog::…` 한 줄만 다릅니다.
//
// 이 파일을 실행 파일로 컴파일하는 패키지:
//   explicit-on-v1-0  templog 1.0.0과 빌드 → 성공
//   explicit-on-v1-1  templog 1.1.0과 빌드 → 성공, 출력도 같음

use templog::{Log, VERSION, mean};

/// 이 프로그램이 직접 정의한 경보입니다. templog 1.1.0에도 같은 이름의 `Alert`가 생깁니다.
struct Alert {
    message: String,
}

/// 이 프로그램이 만든 확장 트레이트입니다.
/// templog 1.1.0의 새 트레이트 `Stats`에도 같은 이름의 `spread(&self)`가 생깁니다.
trait LogExt {
    /// 표준편차(모표준편차)
    fn spread(&self) -> f64;
}

impl LogExt for Log {
    fn spread(&self) -> f64 {
        let values = self.readings();
        let Some(m) = mean(values) else {
            return 0.0;
        };
        let variance = values.iter().map(|c| (c - m).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }
}

fn check(log: &Log, limit: f64) -> Option<Alert> {
    let m = mean(log.readings())?;
    (m > limit).then(|| Alert {
        message: format!("평균 {m:.1}도가 기준 {limit:.1}도를 넘었습니다"),
    })
}

fn main() {
    let mut log = Log::new();
    for celsius in [18.0, 21.5, 24.0, 22.5] {
        log.record(celsius);
    }

    println!("[{}] templog {VERSION}", env!("CARGO_PKG_NAME"));
    let m = mean(log.readings()).unwrap_or(0.0);
    println!("  평균 {m:.2}도, 표준편차 {:.2}도", log.spread());
    if let Some(alert) = check(&log, 20.0) {
        println!("  경보: {}", alert.message);
    }
}
