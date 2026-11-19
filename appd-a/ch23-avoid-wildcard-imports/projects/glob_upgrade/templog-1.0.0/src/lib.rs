//! templog 1.0.0: 이 저장소에서 지어낸 온도 기록 라이브러리입니다.
//!
//! 사용자 코드가 이 버전에 맞춰 작성되었고, 1.1.0은 이 파일에 항목을 더하기만 합니다.

/// 이 크레이트의 버전. 실행 결과에 어느 스냅숏으로 빌드했는지 보이려고 둡니다.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 한 지점의 섭씨 온도 기록.
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
}

/// 평균. 값이 없으면 `None`.
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}
