//! 프로필의 `debug`(디버그 정보의 양)와 `strip` 설정이 백트레이스에 무엇을 남기는지 봅니다.
//!
//! 실행:
//!   cargo run -q -p p3-ch6-profiles-build-performance --example 02_debuginfo_backtrace
//!   cargo run -q -p p3-ch6-profiles-build-performance --example 02_debuginfo_backtrace --release
//!
//! 프로필 설정을 환경 변수로 바꿔 실행해 보는 명령은 README 3절에 있습니다.

use std::backtrace::Backtrace;

// `#[inline(never)]`는 컴파일러가 이 함수의 본문을 부르는 자리에 펼쳐 넣는 최적화(인라인)를
// 하지 못하게 합니다. 최적화한 release 빌드에서도 `inner`가 백트레이스에 자기 프레임으로 남게 하려고 붙였습니다.
#[inline(never)]
fn inner() -> String {
    Backtrace::force_capture().to_string()
}

#[inline(never)]
fn outer() -> String {
    inner()
}

fn main() {
    let bt = outer();
    // 백트레이스는 "번호: 함수 이름" 줄과, 디버그 정보가 있으면 그 아래 "at 파일:줄:열" 줄로 이뤄집니다.
    let lines: Vec<&str> = bt.lines().map(str::trim).collect();
    let is_inner =
        |l: &&str| !l.starts_with("at ") && (l.ends_with(" inner") || l.ends_with("::inner"));

    let Some(i) = lines.iter().position(is_inner) else {
        println!("inner 프레임: 이름으로 찾지 못함");
        return;
    };
    let name = lines[i].split_once(": ").map_or(lines[i], |(_, n)| n);
    println!("inner 프레임의 이름: {name}");
    match lines.get(i + 1).filter(|l| l.starts_with("at ")) {
        // 경로의 앞부분은 실행한 디렉터리에 따라 달라지므로 파일 이름부터 출력합니다.
        Some(at) => println!(
            "inner 프레임의 위치: {}",
            at.rsplit('/').next().unwrap_or(at)
        ),
        None => println!("inner 프레임의 위치: (정보 없음)"),
    }

    // 출력(dev 프로필):
    // 출력: inner 프레임의 이름: 02_debuginfo_backtrace::inner
    // 출력: inner 프레임의 위치: 02_debuginfo_backtrace.rs:15:5
    // 출력(release 프로필):
    // 출력: inner 프레임의 이름: 02_debuginfo_backtrace::inner
    // 출력: inner 프레임의 위치: (정보 없음)
    // 출력(release + CARGO_PROFILE_RELEASE_DEBUG=line-tables-only):
    // 출력: inner 프레임의 이름: inner
    // 출력: inner 프레임의 위치: 02_debuginfo_backtrace.rs:15:5
    // 출력(release + CARGO_PROFILE_RELEASE_STRIP=symbols):
    // 출력: inner 프레임: 이름으로 찾지 못함
}
