//! 지침 5: 도구마다 성질을 따져 둘 곳(편집기, CI의 어느 작업, 손으로)을 고릅니다.
//!
//! 이 저장소의 챕터들이 다룬 도구를 성질(nightly 필요, 속도, 오탐, 같은 입력에
//! 같은 결과, 스스로 끝남, 용도)과 함께 데이터로 적고, "모두 변경마다 CI에서
//! 돌린다"는 판과 성질을 보고 고르는 판을 비교합니다. 도구를 실행하지는 않습니다.
//! 성질의 근거는 README 2.6절의 표와 각 챕터입니다.
//!
//! 실행: cargo run -p er-ch31-tooling-ecosystem --example 03_tool_placement

/// 도구를 쓰는 목적입니다.
#[derive(Clone, Copy, PartialEq)]
enum Purpose {
    /// 코드가 규칙을 지키는지 판정함(통과/실패가 있음)
    Check,
    /// 게시할 때만 의미가 있는 판정
    Release,
    /// 사람이 보고 이해하려고 씀(판정이 없음)
    Investigate,
}

/// 도구 하나와 그 성질입니다.
struct Tool {
    name: &'static str,
    chapter: &'static str,
    purpose: Purpose,
    /// stable 툴체인으로는 실행할 수 없음
    nightly: bool,
    /// 파일을 저장할 때마다 돌려도 될 만큼 빠르고, 고친 곳 가까이를 알려 줌
    fast: bool,
    /// 알려진 오탐이 있어 예외 표시가 자주 필요함
    false_positives: bool,
    /// 코드와 Cargo.lock이 그대로면 결과도 같음
    repeatable: bool,
    /// 스스로 끝남
    ends: bool,
}

/// 도구를 둘 곳입니다. 위에서 아래로 가까운(자주 도는) 곳부터 적었습니다.
#[derive(Clone, Copy, PartialEq)]
enum Place {
    EditorAndCi,
    EveryChange,
    NightlyJob,
    Scheduled,
    TimeBoxed,
    BeforeRelease,
    ByHand,
}

impl Place {
    const ALL: [Place; 7] = [
        Place::EditorAndCi,
        Place::EveryChange,
        Place::NightlyJob,
        Place::Scheduled,
        Place::TimeBoxed,
        Place::BeforeRelease,
        Place::ByHand,
    ];
    fn label(self) -> &'static str {
        match self {
            Place::EditorAndCi => "편집기 + CI(변경마다)",
            Place::EveryChange => "CI(변경마다)",
            Place::NightlyJob => "CI(nightly 툴체인 작업)",
            Place::Scheduled => "CI(정기 예약 작업)",
            Place::TimeBoxed => "시간을 정해 따로",
            Place::BeforeRelease => "게시 전",
            Place::ByHand => "필요할 때 손으로",
        }
    }
}

/// 기본값(stable, 느림, 오탐 없음, 같은 결과, 스스로 끝남)으로 도구를 만듭니다.
/// 기본값과 다른 성질만 아래 메서드로 덧붙입니다.
const fn tool(name: &'static str, chapter: &'static str, purpose: Purpose) -> Tool {
    Tool {
        name,
        chapter,
        purpose,
        nightly: false,
        fast: false,
        false_positives: false,
        repeatable: true,
        ends: true,
    }
}

impl Tool {
    const fn nightly(mut self) -> Self {
        self.nightly = true;
        self
    }
    const fn fast(mut self) -> Self {
        self.fast = true;
        self
    }
    const fn false_positives(mut self) -> Self {
        self.false_positives = true;
        self
    }
    const fn unrepeatable(mut self) -> Self {
        self.repeatable = false;
        self
    }
    const fn endless(mut self) -> Self {
        self.ends = false;
        self
    }
}

/// 성질의 근거는 README 2.6절의 표를 봅니다.
fn tools() -> Vec<Tool> {
    use Purpose::*;
    vec![
        tool("cargo fmt", "Part 0 ch1", Check).fast(),
        tool("cargo check", "Part 0 ch1", Check).fast(),
        tool("cargo clippy", "ch29", Check).fast(),
        tool("clippy pedantic", "ch29", Check).false_positives(),
        tool("cargo test", "ch30", Check),
        tool("cargo doc (-D warnings)", "ch27", Check),
        tool("cargo deny check bans licenses", "ch25", Check),
        tool("cargo-machete", "ch25", Check)
            .fast()
            .false_positives(),
        tool("cargo hack --feature-powerset", "ch26", Check),
        tool("Miri", "ch16", Check).nightly(),
        tool("cargo-udeps", "ch25", Check).nightly(),
        tool("cargo-audit", "ch25", Check).unrepeatable(),
        tool("cargo bench (criterion)", "ch30", Check).unrepeatable(),
        tool("cargo-fuzz", "ch30", Check).nightly().endless(),
        tool("cargo-semver-checks", "ch21", Release),
        tool("cargo package --list", "ch30", Release),
        tool("cargo-expand", "ch28", Investigate),
        tool("cargo tree -i", "ch25", Investigate),
        tool("Compiler Explorer", "ch30", Investigate),
    ]
}

/// 비권장: 도구의 성질을 보지 않고 모두 변경마다 도는 CI 작업에 넣습니다.
fn place_naive(_: &Tool) -> Place {
    Place::EveryChange
}

/// 권장: 성질을 위에서부터 차례로 따집니다.
fn place(t: &Tool) -> Place {
    match t.purpose {
        Purpose::Investigate => return Place::ByHand,
        Purpose::Release => return Place::BeforeRelease,
        Purpose::Check => {}
    }
    if !t.ends {
        Place::TimeBoxed
    } else if !t.repeatable {
        Place::Scheduled
    } else if t.nightly {
        Place::NightlyJob
    } else if t.fast && !t.false_positives {
        Place::EditorAndCi
    } else {
        Place::EveryChange
    }
}

/// 조건에 맞는 도구 이름을 쉼표로 잇습니다.
fn names(tools: &[Tool], pred: impl Fn(&Tool) -> bool) -> String {
    let picked: Vec<&str> = tools.iter().filter(|t| pred(t)).map(|t| t.name).collect();
    if picked.is_empty() {
        "(없음)".to_string()
    } else {
        picked.join(", ")
    }
}

fn main() {
    let tools = tools();
    println!("도구 {}개", tools.len());
    // 출력: 도구 19개

    println!("\n1. 비권장: 모두 변경마다 도는 CI 작업에 넣음");
    let in_ci = |t: &Tool| place_naive(t) == Place::EveryChange;
    println!(
        "  stable CI에서 돌지 않음(nightly 필요): {}",
        names(&tools, |t| in_ci(t) && t.nightly)
    );
    // 출력: stable CI에서 돌지 않음(nightly 필요): Miri, cargo-udeps, cargo-fuzz
    println!(
        "  코드가 그대로여도 결과가 바뀜: {}",
        names(&tools, |t| in_ci(t) && !t.repeatable)
    );
    // 출력: 코드가 그대로여도 결과가 바뀜: cargo-audit, cargo bench (criterion)
    println!(
        "  스스로 끝나지 않음: {}",
        names(&tools, |t| in_ci(t) && !t.ends)
    );
    // 출력: 스스로 끝나지 않음: cargo-fuzz
    println!(
        "  판정이 없거나 게시 때만 의미 있음: {}",
        names(&tools, |t| in_ci(t) && t.purpose != Purpose::Check)
    );

    // 2번 구역의 전체 출력은 README 3절에 옮겼습니다. 첫 자리는 이렇습니다.
    // 출력: [편집기 + CI(변경마다)] 아래에 cargo fmt, cargo check, cargo clippy
    println!("\n2. 권장: 성질을 보고 둘 곳을 고름");
    for p in Place::ALL {
        let here: Vec<String> = tools
            .iter()
            .filter(|t| place(t) == p)
            .map(|t| format!("{} ({})", t.name, t.chapter))
            .collect();
        println!("  [{}]", p.label());
        for line in here {
            println!("    {line}");
        }
    }
}
