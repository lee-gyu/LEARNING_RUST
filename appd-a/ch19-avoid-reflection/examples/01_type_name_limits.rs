//! 지침 1: `type_name`은 진단에만 쓴다
//!
//! 학습 플랫폼의 활동 기록(이벤트)을 처리 경로로 나누는 함수를 두 가지로 만듭니다.
//! - 비권장: `type_name`이 돌려준 문자열을 비교해 경로를 고릅니다.
//! - 권장: 경로를 트레이트 메서드로 두어 각 타입이 직접 답하게 합니다.
//!
//! 실행: cargo run -p er-ch19-avoid-reflection --example 01_type_name_limits
//!
//! `type_name`의 결과는 형식이 보장되지 않는 진단용 문자열입니다. 아래 `// 출력:`
//! 주석은 rustc 1.100.0에서 실행한 결과이며, 툴체인이 바뀌면 달라질 수 있습니다.

use std::any::{type_name, type_name_of_val};

/// 학습 플랫폼에서 일어나는 활동 기록
mod events {
    /// 수강 신청
    pub struct Enrolled;
    /// 퀴즈 제출
    pub struct QuizSubmitted;

    /// 예전 시스템에서 넘어온 수강 신청 기록. 이름은 같지만 다른 타입입니다.
    pub mod legacy {
        pub struct Enrolled;
    }
}

use events::{Enrolled, QuizSubmitted, legacy};

/// 같은 종류의 이벤트를 여러 개 묶어 한 번에 보내는 묶음
struct Batch<E>(Vec<E>);

// ---------------------------------------------------------------
// 비권장: 타입 이름 문자열로 분기
// ---------------------------------------------------------------

/// 처음 실행해 본 `type_name` 출력을 그대로 복사해 비교합니다.
/// 문자열에 예제 크레이트 이름(`01_type_name_limits`)과 모듈 경로까지 들어 있다는 점에
/// 주의하세요. 타입을 다른 모듈이나 크레이트로 옮기면 이 비교는 모두 빗나갑니다.
fn route_by_exact_name<E>(_event: &E) -> &'static str {
    match type_name::<E>() {
        "01_type_name_limits::events::Enrolled" => "환영 메일",
        "01_type_name_limits::events::QuizSubmitted" => "성적 처리",
        _ => "버림",
    }
}

/// 경로가 바뀌어도 맞히려고 끝부분만 비교합니다.
/// 경로 문제는 피했지만, 이름이 같은 다른 타입까지 잡게 됩니다.
fn route_by_name_suffix<E>(_event: &E) -> &'static str {
    let name = type_name::<E>();
    if name.ends_with("Enrolled") {
        "환영 메일"
    } else if name.ends_with("QuizSubmitted") {
        "성적 처리"
    } else {
        "버림"
    }
}

// ---------------------------------------------------------------
// 권장: 처리 경로를 트레이트로
// ---------------------------------------------------------------

/// 이벤트가 어느 경로로 가야 하는지는 그 타입이 답합니다.
/// 이 트레이트를 구현하지 않은 타입은 `route_by_trait`에 넘길 수조차 없으므로,
/// "모르는 타입이라 버림" 같은 갈래가 생기지 않습니다.
trait Event {
    fn route(&self) -> &'static str;
}

impl Event for Enrolled {
    fn route(&self) -> &'static str {
        "환영 메일"
    }
}

impl Event for QuizSubmitted {
    fn route(&self) -> &'static str {
        "성적 처리"
    }
}

impl Event for legacy::Enrolled {
    fn route(&self) -> &'static str {
        "옛 형식 변환"
    }
}

/// 묶음은 안에 든 이벤트와 같은 경로로 보냅니다.
/// `E: Event`라는 바운드 덕분에 `Batch<Enrolled>`든 `Batch<QuizSubmitted>`든 한 번에 해결됩니다.
impl<E: Event> Event for Batch<E> {
    fn route(&self) -> &'static str {
        self.0.first().map_or("버림", Event::route)
    }
}

/// 참조로 넘어와도 가리키는 값과 같은 경로입니다.
/// 표준 라이브러리도 `Display`, `Debug` 등에 같은 모양의 포괄 구현을 둡니다.
impl<E: Event + ?Sized> Event for &E {
    fn route(&self) -> &'static str {
        (**self).route()
    }
}

fn route_by_trait<E: Event>(event: &E) -> &'static str {
    event.route()
}

// ---------------------------------------------------------------

/// 같은 이벤트를 세 방법으로 처리해 결과를 모읍니다.
fn all_routes<E: Event>(event: &E) -> [&'static str; 3] {
    [
        route_by_exact_name(event),
        route_by_name_suffix(event),
        route_by_trait(event),
    ]
}

/// 한 줄에 기대값과 세 방법의 결과를 나란히 적습니다. 기대와 다르면 "(틀림)"을 붙입니다.
fn row(label: &str, expected: &str, results: [&str; 3]) {
    let marked: Vec<String> = results
        .iter()
        .map(|got| {
            if *got == expected {
                got.to_string()
            } else {
                format!("{got}(틀림)")
            }
        })
        .collect();
    println!("    {label}: 기대 {expected} | {}", marked.join(" | "));
}

type Minutes = u32;
type Score = u32;

fn main() {
    let enrolled = Enrolled;
    let submitted = QuizSubmitted;
    let old = legacy::Enrolled;
    let batch = Batch(vec![Enrolled, Enrolled]);
    // 반복자에서 꺼낸 원소처럼, 값 대신 참조를 들고 있는 경우
    let borrowed: &Enrolled = &enrolled;

    println!("1. type_name이 돌려준 문자열 (진단용, 툴체인에 따라 다를 수 있음)");
    println!("    {}", type_name::<Enrolled>());
    // 출력:     01_type_name_limits::events::Enrolled
    println!("    {}", type_name::<&Enrolled>());
    // 출력:     &01_type_name_limits::events::Enrolled
    println!("    {}", type_name::<Batch<Enrolled>>());
    // 출력:     01_type_name_limits::Batch<01_type_name_limits::events::Enrolled>
    println!("    {}", type_name::<legacy::Enrolled>());
    // 출력:     01_type_name_limits::events::legacy::Enrolled

    println!("2. 처리 경로 고르기: 기대 | 전체 이름 비교 | 끝부분 비교 | 트레이트");
    row("Enrolled", "환영 메일", all_routes(&enrolled));
    // 출력:     Enrolled: 기대 환영 메일 | 환영 메일 | 환영 메일 | 환영 메일
    row("QuizSubmitted", "성적 처리", all_routes(&submitted));
    // 출력:     QuizSubmitted: 기대 성적 처리 | 성적 처리 | 성적 처리 | 성적 처리
    // E = &Enrolled 가 되어 이름 앞에 &가 붙습니다.
    row("&Enrolled", "환영 메일", all_routes(&borrowed));
    // 출력:     &Enrolled: 기대 환영 메일 | 버림(틀림) | 환영 메일 | 환영 메일
    // 제네릭 인자가 이름 안에 들어가 끝부분도 "Enrolled>"가 됩니다.
    row("Batch<Enrolled>", "환영 메일", all_routes(&batch));
    // 출력:     Batch<Enrolled>: 기대 환영 메일 | 버림(틀림) | 버림(틀림) | 환영 메일
    // 끝부분 비교는 이름이 같은 다른 타입을 구별하지 못합니다.
    row("legacy::Enrolled", "옛 형식 변환", all_routes(&old));
    // 출력:     legacy::Enrolled: 기대 옛 형식 변환 | 버림(틀림) | 환영 메일(틀림) | 옛 형식 변환

    println!("3. 이름에 남지 않는 것");
    // 별칭은 새 타입이 아니라 이름을 하나 더 붙인 것이라, type_name에는 원래 타입만 나옵니다.
    println!(
        "    type Minutes = u32 → {}, type Score = u32 → {}",
        type_name::<Minutes>(),
        type_name::<Score>()
    );
    // 출력:     type Minutes = u32 → u32, type Score = u32 → u32
    // 라이프타임도 이름에 나오지 않습니다. 영원히 유효한 참조와 지역 변수를 빌린 참조는
    // 다른 타입이지만 같은 문자열을 돌려받습니다.
    let local = String::from("지역 문자열");
    let short_lived: &str = &local;
    let forever: &'static str = "리터럴";
    println!(
        "    &'static str → {}, 지역 String을 빌린 &str → {}",
        type_name_of_val(&forever),
        type_name_of_val(&short_lived)
    );
    // 출력:     &'static str → &str, 지역 String을 빌린 &str → &str
}
