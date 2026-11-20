//! 원칙 "실패는 종료 코드로, 빠짐없이": 검사 단계를 돌리는 쪽(셸 스크립트, CI)의 규칙을 비교합니다.
//!
//! 1. 단계가 실패했는데 종료 코드가 0이 되는 셸 모양 셋을 bash로 실제로 실행합니다.
//! 2. 첫 실패에서 멈추는 실행과 끝까지 돌고 요약하는 실행이, 문제 셋을 모두 고칠 때까지
//!    CI를 몇 번 돌리게 하는지 셉니다(단계의 결과는 projects/pollcount의 변경 실험에서 가져온 값).
//! 3. 설치되지 않은 도구를 쓰는 단계를 조용히 통과시키는 실행기와, 로컬에서는 건너뛰고
//!    CI에서는 실패로 다루는 실행기를 비교합니다.
//!
//! 1번은 bash가 있어야 합니다(macOS·Linux에는 기본으로 있음).
//!
//! 실행: cargo run -p er-ch32-continuous-integration --example 02_step_runner

use std::collections::BTreeSet;
use std::process::Command;

/// bash에 스크립트를 넘겨 종료 코드를 받습니다. 표준 출력은 버립니다.
fn bash(flags: &[&str], script: &str) -> Option<i32> {
    let out = Command::new("bash")
        .args(flags)
        .arg("-c")
        .arg(script)
        .output()
        .ok()?;
    out.status.code()
}

fn part1_lost_exit_codes() {
    println!("1. 실패한 명령의 종료 코드가 사라지는 곳");
    // 실패하는 cargo 명령 대신 종료 코드 101로 끝나는 서브셸을 씁니다.
    let failing = "(echo 'test result: FAILED'; exit 101)";
    // GitHub Actions에서 run 단계의 shell을 적지 않으면 bash -e {0},
    // shell: bash라고 적으면 bash --noprofile --norc -eo pipefail {0}로 실행됩니다.
    let default_shell = ["-e"];
    let explicit_bash = ["--noprofile", "--norc", "-eo", "pipefail"];
    // `sh ci.sh`처럼 파일로 실행하는 스크립트는 set -e를 쓰지 않는 한 -e 없이 돕니다.
    let script_file: [&str; 0] = [];
    let cases: [(&str, &[&str], String); 6] = [
        (
            "비권장: 파이프, shell 미지정(bash -e)",
            &default_shell,
            format!("{failing} | tee /dev/null"),
        ),
        (
            "권장:   파이프, shell: bash(-eo pipefail)",
            &explicit_bash,
            format!("{failing} | tee /dev/null"),
        ),
        (
            "비권장: 실패를 || true로 덮음",
            &explicit_bash,
            format!("{failing} || true"),
        ),
        ("권장:   덮지 않음", &explicit_bash, failing.to_string()),
        (
            "비권장: -e 없는 스크립트의 반복문",
            &script_file,
            "for code in 101 0; do (exit $code); done".to_string(),
        ),
        (
            "권장:   반복문 안에서 || exit",
            &script_file,
            "for code in 101 0; do (exit $code) || exit; done".to_string(),
        ),
    ];
    for (label, flags, script) in &cases {
        match bash(flags, script) {
            Some(code) => println!("  {label}  → 종료 코드 {code}"),
            None => println!("  {label}  → bash를 실행하지 못했습니다"),
        }
    }
    // 출력:   비권장: 파이프, shell 미지정(bash -e)  → 종료 코드 0
    // 출력:   권장:   파이프, shell: bash(-eo pipefail)  → 종료 코드 101
    // 출력:   비권장: 실패를 || true로 덮음  → 종료 코드 0
    // 출력:   권장:   덮지 않음  → 종료 코드 101
    // 출력:   비권장: -e 없는 스크립트의 반복문  → 종료 코드 0
    // 출력:   권장:   반복문 안에서 || exit  → 종료 코드 101
    // 파이프의 종료 코드는 마지막 명령(tee)의 것이고, pipefail이 있어야 앞 명령의 실패가
    // 드러납니다. -e가 없으면 반복문 전체의 결과는 마지막 반복의 결과입니다(기능 조합마다
    // cargo를 부르는 반복문이 흔한 예). 로그에 FAILED가 찍혀 있어도 CI는 종료 코드만 보므로
    // 초록으로 표시합니다.
}

/// 단계 이름과, 이번 변경에 든 문제 가운데 그 단계를 실패시키는 것의 종료 코드입니다.
/// 값은 projects/pollcount에서 문제를 하나씩 넣어 ci.sh가 낸 결과입니다(README 2.2절의 표).
const STEPS: [(&str, Option<(&str, i32)>); 8] = [
    ("lockfile", None),
    ("fmt", Some(("서식 어긋남", 1))),
    ("conventions", None),
    ("clippy", None),
    ("test", Some(("실패하는 문서 테스트", 101))),
    ("examples", None),
    ("docs", Some(("깨진 문서 링크", 101))),
    ("msrv", None),
];

/// 남은 문제로 단계를 한 번 돕니다. 실패한 단계의 (이름, 문제, 종료 코드)를 돌려줍니다.
fn one_run<'a>(
    remaining: &BTreeSet<&'a str>,
    stop_at_first: bool,
) -> (Vec<(&'a str, &'a str, i32)>, usize) {
    let mut failed = Vec::new();
    let mut executed = 0;
    for (step, problem) in STEPS {
        executed += 1;
        if let Some((name, code)) = problem
            && remaining.contains(name)
        {
            failed.push((step, name, code));
            if stop_at_first {
                break;
            }
        }
    }
    (failed, executed)
}

/// 초록이 될 때까지 "돌리고, 보고된 실패를 고치고, 다시 돌리기"를 되풀이합니다.
fn until_green(stop_at_first: bool) {
    let mut remaining: BTreeSet<&str> = STEPS
        .iter()
        .filter_map(|(_, p)| p.map(|(n, _)| n))
        .collect();
    let mut runs = 0;
    loop {
        runs += 1;
        let (failed, executed) = one_run(&remaining, stop_at_first);
        if failed.is_empty() {
            println!("    {runs}회차: 단계 {executed}개 실행, 모두 통과 (종료 코드 0)");
            break;
        }
        let shown: Vec<String> = failed
            .iter()
            .map(|(step, _, code)| format!("{step}({code})"))
            .collect();
        println!(
            "    {runs}회차: 단계 {executed}개 실행, 실패 {} (종료 코드 1)",
            shown.join(" ")
        );
        // 사람은 보고된 실패만 고칠 수 있습니다.
        for (_, name, _) in failed {
            remaining.remove(name);
        }
    }
    println!("    → 초록이 될 때까지 CI를 {runs}번 기다림");
}

fn part2_fail_fast_or_keep_going() {
    println!("2. 한 변경에 문제가 셋(서식, 문서 테스트, 문서 링크)일 때");
    println!("  첫 실패에서 멈춤 (ci.sh의 기본)");
    until_green(true);
    // 출력:     1회차: 단계 2개 실행, 실패 fmt(1) (종료 코드 1)
    // 출력:     2회차: 단계 5개 실행, 실패 test(101) (종료 코드 1)
    // 출력:     3회차: 단계 7개 실행, 실패 docs(101) (종료 코드 1)
    // 출력:     4회차: 단계 8개 실행, 모두 통과 (종료 코드 0)
    // 출력:     → 초록이 될 때까지 CI를 4번 기다림
    println!("  끝까지 돌고 요약 (ci.sh --keep-going, 또는 단계마다 따로 도는 CI 작업)");
    until_green(false);
    // 출력:     1회차: 단계 8개 실행, 실패 fmt(1) test(101) docs(101) (종료 코드 1)
    // 출력:     2회차: 단계 8개 실행, 모두 통과 (종료 코드 0)
    // 출력:     → 초록이 될 때까지 CI를 2번 기다림
    // 첫 실패에서 멈추면 기계 시간은 아끼지만, 사람이 기다리는 횟수가 늘어납니다.
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Local,
    Ci,
}

/// 설치되지 않은 도구를 쓰는 단계를 만났을 때의 판정입니다.
fn judge_missing_tool(mode: Mode, silent_pass: bool) -> (&'static str, i32) {
    if silent_pass {
        ("통과로 처리", 0)
    } else if mode == Mode::Local {
        ("건너뜀(로그에 남김)", 0)
    } else {
        ("실패", 1)
    }
}

fn part3_missing_tools() {
    println!("3. cargo-deny가 설치되지 않은 환경에서 deny 단계");
    for (label, silent) in [
        ("비권장: 조용히 통과", true),
        ("권장:   ci.sh의 규칙", false),
    ] {
        for (mode_name, mode) in [("로컬", Mode::Local), ("CI", Mode::Ci)] {
            let (what, code) = judge_missing_tool(mode, silent);
            println!("  {label}, {mode_name}: {what}, 종료 코드 {code}");
        }
    }
    // 출력:   비권장: 조용히 통과, 로컬: 통과로 처리, 종료 코드 0
    // 출력:   비권장: 조용히 통과, CI: 통과로 처리, 종료 코드 0
    // 출력:   권장:   ci.sh의 규칙, 로컬: 건너뜀(로그에 남김), 종료 코드 0
    // 출력:   권장:   ci.sh의 규칙, CI: 실패, 종료 코드 1
    // CI에서 조용히 통과시키면, 도구 설치 단계가 깨진 날부터 아무것도 검사하지 않는
    // 초록 빌드가 이어집니다. 로컬에서는 모든 사람에게 모든 도구를 요구하지 않으려고 건너뜁니다.
}

fn main() {
    part1_lost_exit_codes();
    part2_fail_fast_or_keep_going();
    part3_missing_tools();
}
