// 프로세스가 시작할 때 운영체제에게서 받는 것을 읽습니다.
//
//   1) 명령줄 인자: env::args (UTF-8이 아니면 패닉)와 env::args_os (패닉하지 않음)
//   2) 환경 변수: env::var / env::var_os / env::vars_os, 그리고 VarError의 두 경우
//   3) 현재 디렉터리(current_dir)와 실행 파일 경로(current_exe)
//   4) 컴파일 대상 플랫폼 상수 env::consts
//
// 실행: cargo run -p p1-ch10-env-process-time --example 01_args_env
//       cargo run -p p1-ch10-env-process-time --example 01_args_env -- 하나 "둘 셋" $'\xff'
//       P1CH10_GREETING=안녕 cargo run -p p1-ch10-env-process-time --example 01_args_env
//
// 아래 "출력:" 주석은 저장소 루트에서 인자와 P1CH10_ 환경 변수 없이 실행한 결과입니다(macOS, aarch64).
// 인자와 환경 변수를 바꿔 실행한 결과는 README의 예제 안내에 있습니다.

use std::env::{self, VarError};
use std::path::Path;

// env::var의 결과를 세 갈래로 나눠 설명합니다.
// "없음"과 "있지만 비어 있음"과 "있지만 UTF-8이 아님"은 서로 다른 상황입니다.
fn describe_var(name: &str) -> String {
    match env::var(name) {
        Ok(value) if value.is_empty() => "설정됨, 빈 문자열".to_string(),
        Ok(value) => format!("설정됨, 값 {value:?}"),
        Err(VarError::NotPresent) => "설정되지 않음 (NotPresent)".to_string(),
        // NotUnicode는 원래 값을 OsString으로 돌려줍니다. 디버그 형식은 깨진 바이트를 \xFF처럼 보여 줍니다.
        Err(VarError::NotUnicode(raw)) => format!("UTF-8이 아님 (NotUnicode), 원래 값 {raw:?}"),
    }
}

// 환경 변수로 설정을 받을 때 자주 쓰는 모양입니다.
// 없으면 기본값, 있는데 숫자가 아니면 에러 메시지를 돌려줍니다.
fn workers_from_env() -> Result<u32, String> {
    match env::var("P1CH10_WORKERS") {
        Err(VarError::NotPresent) => Ok(4),
        Err(VarError::NotUnicode(_)) => Err("P1CH10_WORKERS가 UTF-8이 아닙니다".to_string()),
        Ok(text) => text
            .trim()
            .parse()
            .map_err(|e| format!("P1CH10_WORKERS={text:?}를 읽지 못했습니다: {e}")),
    }
}

fn main() {
    println!("1. env::args_os: 인자를 검사 없이 OsString으로 받습니다");
    // args_os()는 UTF-8인지 검사하지 않으므로 어떤 인자에도 패닉하지 않습니다.
    // 반복자는 한 번 쓰면 끝나므로 여러 번 볼 값은 Vec에 모읍니다.
    let raw_args: Vec<_> = env::args_os().collect();
    println!("    개수(0번 포함): {}", raw_args.len()); // 출력:     개수(0번 포함): 1
    for (i, arg) in raw_args.iter().enumerate().skip(1) {
        // to_str()은 UTF-8이면 Some(&str), 아니면 None입니다.
        match arg.to_str() {
            Some(text) => println!("    {i}번: UTF-8 {text:?}"),
            None => println!(
                "    {i}번: UTF-8 아님 {arg:?} → 손실 변환 {:?}",
                arg.to_string_lossy()
            ),
        }
    }

    println!("2. env::args: String으로 받지만 UTF-8이 아닌 인자가 있으면 패닉합니다");
    if raw_args.iter().all(|arg| arg.to_str().is_some()) {
        let args: Vec<String> = env::args().collect();
        // 0번 인자는 관례상 실행 파일 경로입니다. 실행 방법에 따라 상대·절대 경로가 달라지므로 파일 이름만 봅니다.
        let program = Path::new(&args[0]).file_name().and_then(|n| n.to_str());
        println!("    0번 인자의 파일 이름: {program:?}"); // 출력:     0번 인자의 파일 이름: Some("01_args_env")
        println!("    1번부터: {:?}", &args[1..]); // 출력:     1번부터: []
    } else {
        println!("    UTF-8이 아닌 인자가 있어 env::args()를 부르지 않습니다");
    }

    println!("3. 환경 변수 읽기");
    println!("    P1CH10_GREETING: {}", describe_var("P1CH10_GREETING")); // 출력:     P1CH10_GREETING: 설정되지 않음 (NotPresent)
    // var_os는 UTF-8 검사를 하지 않으므로 Result가 아니라 Option입니다.
    println!(
        "    var_os(\"P1CH10_GREETING\").is_some(): {}",
        env::var_os("P1CH10_GREETING").is_some()
    ); // 출력:     var_os("P1CH10_GREETING").is_some(): false
    // 이름에 '='가 들어가면 운영체제에 물어볼 수 없는 이름이라 "없음"으로 취급합니다.
    println!("    \"A=B\": {}", describe_var("A=B")); // 출력:     "A=B": 설정되지 않음 (NotPresent)
    // PATH 같은 값은 환경마다 다르므로 값 대신 있는지와 항목이 있는지만 봅니다.
    // split_paths는 PATH의 구분자(유닉스 ':', Windows ';')로 나눕니다.
    let path_entries = env::var_os("PATH").map(|p| env::split_paths(&p).count());
    println!(
        "    PATH 항목이 하나 이상: {}",
        path_entries.is_some_and(|n| n > 0)
    ); // 출력:     PATH 항목이 하나 이상: true
    // vars()는 값 하나라도 UTF-8이 아니면 패닉하므로, 전체를 훑을 때는 vars_os()가 안전합니다.
    let total = env::vars_os().count();
    println!("    환경 변수가 하나 이상: {}", total > 0); // 출력:     환경 변수가 하나 이상: true
    println!("    작업자 수 설정: {:?}", workers_from_env()); // 출력:     작업자 수 설정: Ok(4)

    println!("4. 현재 디렉터리와 실행 파일");
    // 현재 디렉터리는 "프로그램을 실행한 곳"이고, 상대 경로는 이 디렉터리를 기준으로 풀립니다.
    let cwd = env::current_dir().expect("현재 디렉터리를 읽을 수 있어야 합니다");
    println!("    current_dir는 절대 경로: {}", cwd.is_absolute()); // 출력:     current_dir는 절대 경로: true
    println!(
        "    current_dir에 Cargo.toml이 있음: {}",
        cwd.join("Cargo.toml").is_file()
    ); // 출력:     current_dir에 Cargo.toml이 있음: true
    // 실행 파일 경로는 "프로그램 파일이 있는 곳"입니다. 두 경로는 서로 관계가 없습니다.
    let exe = env::current_exe().expect("실행 파일 경로를 알 수 있어야 합니다");
    println!("    current_exe 파일 이름: {:?}", exe.file_name()); // 출력:     current_exe 파일 이름: Some("01_args_env")
    let parent = exe.parent().and_then(|p| p.file_name());
    println!("    current_exe가 든 디렉터리 이름: {parent:?}"); // 출력:     current_exe가 든 디렉터리 이름: Some("examples")

    println!("5. 컴파일 대상 플랫폼 (env::consts)");
    // 실행 중에 알아내는 값이 아니라, 컴파일할 때 정해진 상수입니다.
    println!(
        "    OS = {:?}, FAMILY = {:?}, ARCH = {:?}",
        env::consts::OS,
        env::consts::FAMILY,
        env::consts::ARCH
    ); // 출력:     OS = "macos", FAMILY = "unix", ARCH = "aarch64"
    println!(
        "    EXE_SUFFIX = {:?}, DLL_EXTENSION = {:?}",
        env::consts::EXE_SUFFIX,
        env::consts::DLL_EXTENSION
    ); // 출력:     EXE_SUFFIX = "", DLL_EXTENSION = "dylib"
}
