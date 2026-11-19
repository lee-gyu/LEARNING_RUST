// 02_rustc_edition_matrix: rustc 하나가 네 에디션을 모두 컴파일합니다
//
// 짧은 코드 조각 여덟 개를 임시 디렉터리에 파일로 쓰고, 같은 rustc에 `--edition`만 바꿔 가며
// 넘겨 컴파일되는지(O) 에러인지(X)를 표로 보여 줍니다. 실행 파일은 만들지 않고 검사만 하도록
// `--emit=metadata`를 씁니다(cargo check가 쓰는 방식과 같습니다).
//
// - 위쪽 다섯 줄: 에디션에 따라 결과가 갈리는 코드입니다.
// - 아래쪽 두 줄: 에디션이 생긴 뒤에 들어온 기능이지만 모든 에디션에서 컴파일됩니다.
//   이 두 줄의 결과를 정하는 것은 에디션이 아니라 컴파일러 버전입니다.
//
// rustc는 환경 변수 RUSTC가 있으면 그 경로를, 없으면 PATH에서 찾습니다.
// 임시 디렉터리는 실행이 끝나면 지웁니다.
//
// 실행: cargo run -p p2-ch1-editions-overview --example 02_rustc_edition_matrix

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::Path;
use std::process::{self, Child, Command, Stdio};

const EDITIONS: [&str; 4] = ["2015", "2018", "2021", "2024"];

/// 표의 한 줄: 설명과 컴파일할 소스 코드
struct Snippet {
    label: &'static str,
    source: &'static str,
}

const SNIPPETS: [Snippet; 8] = [
    Snippet {
        label: "기준: println! 한 줄",
        source: r#"fn main() { println!("hi"); }"#,
    },
    // async는 2018 에디션부터 키워드입니다.
    Snippet {
        label: "async를 변수 이름으로",
        source: r#"fn main() { let async = 1; println!("{}", async); }"#,
    },
    // 2015 에디션의 use 경로는 크레이트 루트에서 시작합니다. 2018 에디션부터는 현재 모듈의
    // 이름(여기서는 하위 모듈 circle)으로 시작할 수 있습니다.
    Snippet {
        label: "하위 모듈 안에서 use circle::area",
        source: r#"
mod shapes {
    mod circle { pub fn area() -> u32 { 3 } }
    use circle::area;
    pub fn report() -> u32 { area() }
}
fn main() { println!("{}", shapes::report()); }
"#,
    },
    // TryFrom은 2021 에디션부터 프렐류드에 들어 있습니다.
    Snippet {
        label: "use 없이 u8::try_from",
        source: r#"fn main() { println!("{:?}", u8::try_from(300i32)); }"#,
    },
    // gen은 2024 에디션부터 예약 키워드입니다.
    Snippet {
        label: "gen을 변수 이름으로",
        source: r#"fn main() { let gen = 1; println!("{}", gen); }"#,
    },
    // let 체인은 2024 에디션에서만 쓸 수 있습니다.
    Snippet {
        label: "let 체인",
        source: r#"fn main() { let x = Some(3); if let Some(n) = x && n > 1 { println!("{}", n); } }"#,
    },
    // let-else는 1.65.0에 안정화된 문법입니다. 기존 코드를 깨뜨리지 않으므로 모든 에디션에 들어왔습니다.
    Snippet {
        label: "let-else (1.65.0)",
        source: r#"fn main() { let Some(n) = Some(3) else { return; }; println!("{}", n); }"#,
    },
    // bool::toggle은 1.100.0에 안정화된 표준 라이브러리 메서드입니다.
    Snippet {
        label: "bool::toggle (1.100.0)",
        source: r#"fn main() { let mut on = false; on.toggle(); println!("{}", on); }"#,
    },
];

fn main() -> io::Result<()> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));

    println!("1. 이 rustc가 아는 에디션");
    // rustc를 찾지 못하면 표를 만들 수 없으므로 안내만 하고 끝냅니다.
    let version = match Command::new(&rustc).arg("--version").output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(e) => {
            println!("    rustc를 실행하지 못했습니다: {e}");
            return Ok(());
        }
    };
    println!("    {version}"); // 출력: rustc 1.100.0 (0ef1bc386 2026-11-09) — 툴체인마다 다름
    let help = Command::new(&rustc).arg("--help").output()?;
    let help = String::from_utf8_lossy(&help.stdout);
    if let Some(line) = help.lines().find(|line| line.contains("--edition")) {
        println!("    {}", line.trim()); // 출력: --edition <2015|2018|2021|2024|future>
    }

    // 프로세스 ID를 붙여, 여러 번 동시에 실행해도 디렉터리가 겹치지 않게 합니다.
    let dir = env::temp_dir().join(format!("rust-p2-ch1-matrix-{}", process::id()));
    fs::create_dir_all(&dir)?;
    let result = print_matrix(&rustc, &dir);
    // 표를 만들다 실패해도 임시 디렉터리는 지웁니다.
    fs::remove_dir_all(&dir)?;
    result
}

fn print_matrix(rustc: &OsStr, dir: &Path) -> io::Result<()> {
    println!("2. 같은 rustc에 --edition만 바꿔 컴파일한 결과 (O: 컴파일됨, X: 에러)");
    println!("    2015 2018 2021 2024  코드 조각");
    for (i, snippet) in SNIPPETS.iter().enumerate() {
        let file = dir.join(format!("snippet_{i}.rs"));
        fs::write(&file, snippet.source)?;

        // 한 조각의 네 에디션을 동시에 컴파일하고 나서 차례로 기다립니다.
        // 에디션마다 출력 디렉터리를 나눠 같은 파일을 동시에 쓰지 않게 합니다.
        let mut children: Vec<Child> = Vec::new();
        for edition in EDITIONS {
            let child = Command::new(rustc)
                .arg("--edition")
                .arg(edition)
                .arg("--emit=metadata")
                .arg("--out-dir")
                .arg(dir.join(edition))
                .arg(&file)
                .stdout(Stdio::null())
                .stderr(Stdio::null()) // 에러 메시지는 버리고 성공 여부만 봅니다
                .spawn()?;
            children.push(child);
        }

        let mut row = String::from("   ");
        for mut child in children {
            let mark = if child.wait()?.success() { "O" } else { "X" };
            row.push_str(&format!("  {mark}  "));
        }
        println!("{row}  {}", snippet.label);
    }
    Ok(())
}
