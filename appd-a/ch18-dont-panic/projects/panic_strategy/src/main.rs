//! `catch_unwind`가 패닉을 붙잡지 못하는 세 경우를 직접 봅니다.
//!
//! 이 디렉터리에서 실행합니다.
//!
//! - `cargo run -q`                   작업 3개를 하나씩 `catch_unwind`로 감싸 실행(되감기 프로필)
//! - `cargo run -q --profile abort`   같은 작업을 중단 프로필로 실행
//! - `cargo run -q -- extern-c`       `extern "C"` 함수 밖으로 나가려는 패닉
//! - `cargo run -q -- drop-panic`     되감기 중에 `drop`에서 또 패닉(이중 패닉)
//!
//! 첫 명령만 종료 코드 0으로 끝나고, 나머지 셋은 프로세스가 중단됩니다.

use std::env;
use std::panic;
use std::process::ExitCode;

/// 썸네일 너비 = 원본 너비 / 축소 배율. 배율이 0이면 0으로 나누기 패닉입니다.
fn thumbnail_width(width: u32, scale: u32) -> u32 {
    width / scale
}

fn run_jobs() {
    let jobs = [
        ("표지.png", 1200, 4),
        ("배너.png", 800, 0),
        ("아이콘.png", 512, 8),
    ];
    let mut done = 0;
    for (name, width, scale) in jobs {
        // 작업마다 경계를 둡니다. 되감기 전략에서는 배너 작업의 패닉이 여기서 멈춥니다.
        match panic::catch_unwind(|| thumbnail_width(width, scale)) {
            Ok(w) => {
                done += 1;
                println!("  {name} → 너비 {w}px");
            }
            Err(_) => println!("  {name} → 실패 (패닉을 붙잡음)"),
        }
    }
    println!("작업 {}개 중 {done}개 성공", jobs.len());
}

/// C 라이브러리에 등록하는 진행률 콜백이라고 가정합니다.
/// `extern "C"` 함수는 패닉이 함수 밖으로 되감겨 나갈 수 없습니다.
extern "C" fn on_progress(percent: i32) {
    if !(0..=100).contains(&percent) {
        panic!("진행률은 0~100이어야 합니다: {percent}");
    }
    println!("  진행률 {percent}%");
}

fn across_extern_c() {
    let result = panic::catch_unwind(|| {
        on_progress(40);
        on_progress(140);
    });
    // 아래 줄은 실행되지 않습니다. 패닉이 on_progress 밖으로 나가려는 순간 프로세스가 중단됩니다.
    println!("  붙잡았나: {}", result.is_err());
}

/// 업로드가 끝나면 지워야 하는 임시 파일이라고 가정합니다.
struct TempUpload {
    name: &'static str,
}

impl Drop for TempUpload {
    fn drop(&mut self) {
        // 비권장: drop에서 패닉합니다. 되감기 중에 이 패닉이 나면 이중 패닉으로 중단됩니다.
        panic!("임시 파일 {}을(를) 지우지 못했습니다", self.name);
    }
}

fn panic_while_unwinding() {
    let result = panic::catch_unwind(|| {
        let _upload = TempUpload { name: "photo.tmp" };
        panic!("업로드 중 연결이 끊겼습니다");
    });
    // 아래 줄은 실행되지 않습니다.
    println!("  붙잡았나: {}", result.is_err());
}

fn main() -> ExitCode {
    let strategy = if cfg!(panic = "unwind") {
        "unwind(되감기)"
    } else {
        "abort(중단)"
    };
    println!("패닉 전략: {strategy}");
    match env::args().nth(1).as_deref() {
        None => run_jobs(),
        Some("extern-c") => across_extern_c(),
        Some("drop-panic") => panic_while_unwinding(),
        Some(other) => {
            eprintln!("알 수 없는 모드: {other} (extern-c, drop-panic 중 하나)");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
