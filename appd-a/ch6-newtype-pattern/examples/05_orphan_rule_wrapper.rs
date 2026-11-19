// 지침: 남의 타입에 남의 트레이트를 구현해야 한다면(고아 규칙에 막힘), 그 타입을 뉴타입으로 감싸
//       내 타입으로 만든 뒤 구현합니다.
// 시도:   impl Display for Duration   → E0117 (compile_fail/e0117_display_for_duration.rs)
// 대안:   내 트레이트 HumanTime을 정의해 Duration에 구현   (내 코드에서 직접 부를 때만 충분)
// 권장:   struct Elapsed(Duration); impl Display for Elapsed (Display를 요구하는 API에 넘길 때)
//
// 빌드 단계별 소요 시간을 "1분 23.4초" 형식으로 보고합니다. 보고 함수는 `T: Display`를 요구합니다.
//
// 실행: cargo run -p er-ch6-newtype-pattern --example 05_orphan_rule_wrapper

use std::fmt;
use std::time::Duration;

const STEPS: [(&str, Duration); 3] = [
    ("컴파일", Duration::from_millis(83_400)),
    ("테스트", Duration::from_millis(7_200)),
    ("문서", Duration::from_millis(600)),
];

// 다른 크레이트의 함수라고 생각하세요. 시그니처는 고칠 수 없고, 값에 Display를 요구합니다.
fn print_report<T: fmt::Display>(rows: &[(&str, T)]) {
    for (name, value) in rows {
        println!("      {name}: {value}");
    }
}

// 시간을 "1분 23.4초"로 바꾸는 계산입니다. 아래 두 방식이 함께 씁니다.
fn korean_time(d: Duration) -> String {
    let ms = d.as_millis();
    let (min, rest) = (ms / 60_000, ms % 60_000);
    let (sec, tenth) = (rest / 1_000, rest % 1_000 / 100);
    if min > 0 {
        format!("{min}분 {sec}.{tenth}초")
    } else {
        format!("{sec}.{tenth}초")
    }
}

// ---- 대안: 내 트레이트 -----------------------------------------------------

// 트레이트가 내 것이면 남의 타입에도 구현할 수 있습니다(고아 규칙이 허용).
// 하지만 print_report가 요구하는 것은 HumanTime이 아니라 Display라서, 그대로는 넘길 수 없습니다.
trait HumanTime {
    fn human(&self) -> String;
}

impl HumanTime for Duration {
    fn human(&self) -> String {
        korean_time(*self)
    }
}

// ---- 권장: 뉴타입 ----------------------------------------------------------

// Elapsed는 이 크레이트의 타입이므로 표준 트레이트 Display를 구현할 수 있습니다.
// 계산(합계, 비교)은 Duration으로 하고, 보여 줄 때만 감쌉니다. 그래서 Duration의 연산을
// Elapsed에 다시 구현할 필요가 없습니다.
struct Elapsed(Duration);

impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // pad로 쓰면 부르는 쪽이 준 너비·정렬도 따릅니다(06_delegation_cost.rs 참고).
        f.pad(&korean_time(self.0))
    }
}

fn main() {
    println!("[그대로] Duration에는 디버그 형식뿐입니다");
    for (name, d) in STEPS {
        println!("    {name}: {d:?}");
    }
    // 출력:
    //     컴파일: 83.4s
    //     테스트: 7.2s
    //     문서: 600ms

    println!("[대안] 내 트레이트 HumanTime");
    let (_, d) = STEPS[0];
    println!("    d.human(): {}", d.human());
    // print_report(&STEPS)는 E0277입니다(Duration은 Display가 아님).
    // 단계마다 String을 새로 만들어 넘겨야 합니다.
    println!("    print_report에는 String으로 바꿔 넘김");
    let rows: Vec<(&str, String)> = STEPS.iter().map(|&(n, d)| (n, d.human())).collect();
    print_report(&rows);
    // 출력:
    //     d.human(): 1분 23.4초
    //     print_report에는 String으로 바꿔 넘김
    //       컴파일: 1분 23.4초
    //       테스트: 7.2초
    //       문서: 0.6초

    println!("[권장] 뉴타입 Elapsed");
    println!("    print_report에 그대로 넘김");
    let rows = STEPS.map(|(n, d)| (n, Elapsed(d)));
    print_report(&rows);
    // 합계는 Duration이 이미 구현한 Sum으로 구하고, 출력할 때만 감쌉니다.
    let total: Duration = STEPS.iter().map(|&(_, d)| d).sum();
    // Display를 구현하면 to_string()도 따라옵니다.
    let text = Elapsed(total).to_string();
    println!("    합계: {text}");
    // 출력:
    //     print_report에 그대로 넘김
    //       컴파일: 1분 23.4초
    //       테스트: 7.2초
    //       문서: 0.6초
    //     합계: 1분 31.2초
}
