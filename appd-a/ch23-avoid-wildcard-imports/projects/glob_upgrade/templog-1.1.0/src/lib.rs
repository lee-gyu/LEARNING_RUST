//! templog 1.1.0: 이 저장소에서 지어낸 온도 기록 라이브러리입니다.
//!
//! 1.0.0의 항목은 한 글자도 바꾸지 않았고, 파일 아래쪽에 새 항목 두 개(`Alert`, `Stats`)를
//! 더했습니다. 기존 항목의 시그니처와 동작이 그대로이므로 마이너 버전만 올립니다.

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

// ---- 1.1.0에서 추가한 항목 ----

/// 기준 온도를 넘은 기록 하나.
#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    pub index: usize,
    pub celsius: f64,
}

/// 기록에 대한 통계.
pub trait Stats {
    /// 최고 온도와 최저 온도의 차(일교차). 기록이 없으면 0.0.
    fn spread(&self) -> f64;

    /// `limit`를 넘은 기록마다 `Alert`를 하나씩 만듭니다.
    fn alerts(&self, limit: f64) -> Vec<Alert>;
}

impl Stats for Log {
    fn spread(&self) -> f64 {
        let max = self.readings.iter().copied().fold(f64::MIN, f64::max);
        let min = self.readings.iter().copied().fold(f64::MAX, f64::min);
        if self.readings.is_empty() {
            0.0
        } else {
            max - min
        }
    }

    fn alerts(&self, limit: f64) -> Vec<Alert> {
        self.readings
            .iter()
            .enumerate()
            .filter(|&(_, &c)| c > limit)
            .map(|(index, &celsius)| Alert { index, celsius })
            .collect()
    }
}
