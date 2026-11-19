// 글롭이 새 트레이트를 스코프에 들여오면, 에러 없이 다른 메서드가 불릴 수 있습니다.
//
// 실행: cargo run -p er-ch23-avoid-wildcard-imports --example 02_new_trait_method
//
// templog 1.1.0이 새 트레이트 Calibrated를 더했고, 그 메서드 이름이 사용자의 확장 트레이트
// 메서드(LogExt::calibrate)와 같습니다. 두 메서드의 수신자 모양이 다르다는 점이 핵심입니다.
//   사용자:  fn calibrate(&mut self, offset: f64)        제자리에서 보정
//   1.1.0:   fn calibrate(&self, offset: f64) -> Log     보정한 사본을 돌려줌
// 수신자가 같다면(둘 다 &self) E0034로 컴파일이 멈춥니다. 그 경우는 projects/glob_upgrade에서 봅니다.

/// 의존 크레이트 templog 1.0.0이라고 생각합니다.
mod templog_1_0 {
    #[derive(Debug, Clone, Default)]
    pub struct Log {
        readings: Vec<f64>,
    }

    impl Log {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn record(&mut self, celsius: f64) {
            self.readings.push(celsius);
        }

        pub fn readings(&self) -> &[f64] {
            &self.readings
        }

        pub fn readings_mut(&mut self) -> &mut [f64] {
            &mut self.readings
        }
    }

    pub fn mean(values: &[f64]) -> f64 {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

/// templog 1.1.0: 1.0.0의 항목을 그대로 내보내고 트레이트 하나를 더했습니다(마이너 변경).
mod templog_1_1 {
    pub use super::templog_1_0::*;

    /// 원본은 그대로 두고 보정한 사본을 돌려주는 API입니다.
    pub trait Calibrated {
        fn calibrate(&self, offset: f64) -> Log;
    }

    impl Calibrated for Log {
        fn calibrate(&self, offset: f64) -> Log {
            let mut copy = Log::new();
            for &c in self.readings() {
                copy.record(c + offset);
            }
            copy
        }
    }
}

// ---- 사용자 코드 ----

/// 사용자가 templog 1.0.0 시절에 만든 확장 트레이트: 기록을 제자리에서 보정합니다.
trait LogExt {
    fn calibrate(&mut self, offset: f64);
}

impl LogExt for templog_1_0::Log {
    fn calibrate(&mut self, offset: f64) {
        for c in self.readings_mut() {
            *c += offset;
        }
    }
}

// 아래 세 모듈의 run 본문은 주석을 빼면 글자 하나 다르지 않습니다. 다른 것은 위쪽 두 줄의 use뿐입니다.

/// 비권장: 글롭, templog 1.0.0
mod glob_on_1_0 {
    use crate::templog_1_0 as templog;
    use templog::*;

    use crate::LogExt;

    pub fn run() -> f64 {
        let mut log = Log::new();
        for c in [20.5, 21.5, 22.5] {
            log.record(c);
        }
        log.calibrate(-0.5); // 센서가 0.5도 높게 재므로 보정합니다.
        mean(log.readings())
    }
}

/// 비권장: 글롭, templog 1.1.0
mod glob_on_1_1 {
    use crate::templog_1_1 as templog;
    use templog::*;

    // 1.0.0과 똑같이 LogExt를 가져오지만, 아래 호출이 Calibrated 쪽으로 넘어가서
    // 이 모듈에서는 LogExt가 한 번도 쓰이지 않게 됩니다. 그래서 unused_imports 경고가 납니다.
    // 이 경고가 동작이 바뀌었다는 유일한 단서이고, 모듈 안에 LogExt를 쓰는 다른 호출이
    // 하나라도 있으면 이 경고마저 나지 않습니다.
    #[expect(
        unused_imports,
        reason = "templog 1.1의 Calibrated가 호출을 가져가 LogExt가 쓰이지 않게 된 모습을 보입니다"
    )]
    use crate::LogExt;

    pub fn run() -> f64 {
        let mut log = Log::new();
        for c in [20.5, 21.5, 22.5] {
            log.record(c);
        }
        // 메서드 탐색은 수신자를 Log, &Log, &mut Log 순서로 맞춰 봅니다.
        // &Log 단계에서 Calibrated::calibrate(&self)가 먼저 잡히므로
        // &mut Log 단계의 LogExt::calibrate(&mut self)까지 가지 않습니다.
        // 돌려받은 사본은 버려지고 log는 보정되지 않습니다. 컴파일 에러는 없습니다.
        log.calibrate(-0.5);
        mean(log.readings())
    }
}

/// 권장: 쓰는 이름만 명시적으로 가져오기, templog 1.1.0
mod named_on_1_1 {
    use crate::templog_1_1 as templog;
    use templog::{Log, mean};

    use crate::LogExt;

    pub fn run() -> f64 {
        let mut log = Log::new();
        for c in [20.5, 21.5, 22.5] {
            log.record(c);
        }
        // Calibrated를 가져오지 않았으므로 후보는 LogExt::calibrate 하나뿐입니다.
        log.calibrate(-0.5);
        mean(log.readings())
    }
}

fn main() {
    println!("기록 20.5, 21.5, 22.5도를 -0.5도 보정한 뒤의 평균");
    println!("  글롭, templog 1.0:   {:.2}도", glob_on_1_0::run());
    // 출력:   글롭, templog 1.0:   21.00도
    println!("  글롭, templog 1.1:   {:.2}도", glob_on_1_1::run());
    // 출력:   글롭, templog 1.1:   21.50도
    println!("  명시적, templog 1.1: {:.2}도", named_on_1_1::run());
    // 출력:   명시적, templog 1.1: 21.00도
}
