// 글롭으로 들어온 이름은 언제 지고 언제 이기는가: 의존성이 같은 이름의 함수를 더하기 전과 뒤를 비교합니다.
//
// 실행: cargo run -p er-ch23-avoid-wildcard-imports --example 01_glob_priority
//
// templog_1_0과 templog_1_1은 같은 의존 크레이트의 두 버전을 흉내 낸 모듈입니다.
// 이름을 찾는 규칙은 모듈이든 크레이트든 같으므로, 한 파일 안에서 버전 업데이트를 재현할 수 있습니다.
// 실제 Cargo 프로젝트로 재현한 판은 projects/glob_upgrade에 있습니다.

/// 의존 크레이트 templog 1.0.0이라고 생각합니다.
mod templog_1_0 {
    pub fn mean(values: &[f64]) -> f64 {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

/// templog 1.1.0: 1.0.0의 항목을 모두 그대로 내보내고 공개 함수 두 개를 더했습니다.
/// 항목을 더하기만 했으므로 시맨틱 버저닝으로는 마이너 변경입니다.
mod templog_1_1 {
    pub use super::templog_1_0::*;

    pub fn format_temp(celsius: f64) -> String {
        format!("{celsius:.1}°C")
    }

    // 사용자 코드의 명시적 가져오기에 가려져 아무도 부르지 않으므로 dead_code 경고가 납니다.
    // 실제 외부 크레이트의 공개 함수라면 이 경고는 나지 않습니다.
    #[expect(
        dead_code,
        reason = "(나)에서 명시적 가져오기에 가려지는 것을 보이려고 둔 함수"
    )]
    pub fn to_fahrenheit(celsius: f64) -> f64 {
        (celsius * 1.8 + 32.0).round() // 정수로 반올림하는 판
    }
}

/// 다른 크레이트라고 생각하는 단위 변환 모듈입니다. 사용자는 여기서 이름을 하나 가져옵니다.
mod units {
    pub fn to_fahrenheit(celsius: f64) -> f64 {
        celsius * 1.8 + 32.0
    }
}

// ---- 사용자 코드 ----
// on_1_0과 on_1_1은 첫 줄(어느 버전을 templog라고 부를지)만 다르고 나머지는 글자 하나 다르지 않습니다.

mod on_1_0 {
    use crate::templog_1_0 as templog;

    use crate::units::to_fahrenheit; // 명시적 가져오기
    use templog::*; // 모듈 수준의 글롭

    // 이 모듈에 직접 정의한 함수. 1.1.0에도 같은 이름이 생깁니다.
    fn format_temp(celsius: f64) -> String {
        format!("{celsius:.1}도")
    }

    // (가) 같은 모듈의 지역 정의와 글롭
    pub fn local_item(values: &[f64]) -> String {
        format_temp(mean(values))
    }

    // (나) 명시적 가져오기와 글롭
    pub fn named_import(values: &[f64]) -> String {
        format!("{:.2}F", to_fahrenheit(mean(values)))
    }

    // (다) 함수 본문 안의 글롭과 바깥(모듈)의 지역 정의
    pub fn glob_in_fn(values: &[f64]) -> String {
        use templog::*;
        format_temp(mean(values))
    }
}

mod on_1_1 {
    use crate::templog_1_1 as templog;

    use crate::units::to_fahrenheit; // 명시적 가져오기
    use templog::*; // 모듈 수준의 글롭

    // 이 모듈에 직접 정의한 함수. 1.1.0에도 같은 이름이 생깁니다.
    fn format_temp(celsius: f64) -> String {
        format!("{celsius:.1}도")
    }

    // (가) 같은 모듈의 지역 정의와 글롭
    pub fn local_item(values: &[f64]) -> String {
        format_temp(mean(values))
    }

    // (나) 명시적 가져오기와 글롭
    pub fn named_import(values: &[f64]) -> String {
        format!("{:.2}F", to_fahrenheit(mean(values)))
    }

    // (다) 함수 본문 안의 글롭과 바깥(모듈)의 지역 정의
    pub fn glob_in_fn(values: &[f64]) -> String {
        use templog::*;
        format_temp(mean(values))
    }
}

fn compare(label: &str, before: String, after: String) {
    let verdict = if before == after {
        "그대로"
    } else {
        "바뀜"
    };
    println!("{label}");
    println!("  templog 1.0: {before}, templog 1.1: {after} → {verdict}");
}

fn main() {
    let values = [20.5, 21.5, 22.5];

    // 같은 모듈에서는 지역 정의와 명시적 가져오기가 글롭보다 우선합니다.
    // templog 1.1에 같은 이름이 생겨도 사용자 코드는 계속 자기 함수를 부릅니다.
    compare(
        "(가) 같은 모듈의 지역 정의 vs 글롭",
        on_1_0::local_item(&values),
        on_1_1::local_item(&values),
    );
    // 출력: (가) 같은 모듈의 지역 정의 vs 글롭
    // 출력:   templog 1.0: 21.5도, templog 1.1: 21.5도 → 그대로

    compare(
        "(나) 명시적 가져오기 vs 글롭",
        on_1_0::named_import(&values),
        on_1_1::named_import(&values),
    );
    // 출력: (나) 명시적 가져오기 vs 글롭
    // 출력:   templog 1.0: 70.70F, templog 1.1: 70.70F → 그대로

    // "지역 정의가 이긴다"는 규칙은 같은 스코프 안에서만 성립합니다.
    // 함수 본문은 모듈보다 안쪽 스코프라서, 본문 안의 글롭이 바깥 모듈의 format_temp를 가립니다.
    // 에러도 경고도 없이 templog 1.1의 format_temp가 불립니다.
    compare(
        "(다) 함수 안의 글롭 vs 바깥 모듈의 지역 정의",
        on_1_0::glob_in_fn(&values),
        on_1_1::glob_in_fn(&values),
    );
    // 출력: (다) 함수 안의 글롭 vs 바깥 모듈의 지역 정의
    // 출력:   templog 1.0: 21.5도, templog 1.1: 21.5°C → 바뀜
}
