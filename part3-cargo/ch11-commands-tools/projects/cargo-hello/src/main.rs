//! 사용자 정의 하위 명령 `cargo hello`: `cargo metadata`의 JSON을 읽어 워크스페이스를 요약합니다.
//!
//! PATH에 이 실행 파일(`cargo-hello`)이 있으면 `cargo hello [--manifest-path 경로] [--deps]`로 부를 수 있습니다.
//! 설치하지 않고 써 보려면: cargo build && PATH="$PWD/target/debug:$PATH" cargo hello

use std::path::Path;
use std::process::{Command, ExitCode};

use serde_json::Value;

const HELP: &str = "cargo hello: cargo metadata를 읽어 워크스페이스를 요약합니다

사용법: cargo hello [--manifest-path <Cargo.toml 경로>] [--deps]
  --deps  의존성 그래프(resolve)에서 멤버마다 직접 의존성을 보여 줍니다";

/// 경로의 마지막 구성 요소. 절대 경로의 앞부분은 컴퓨터마다 다르므로 줄여 보입니다.
fn last(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned());
    format!("…/{}", name.unwrap_or_default())
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().collect();
    // `cargo hello …`로 부르면 Cargo가 `cargo-hello hello …`로 실행합니다.
    // 둘째 인자로 하위 명령 이름이 오므로 걷어 냅니다(직접 실행하면 오지 않습니다).
    println!("받은 인자: {:?}", &args[1..]);
    if args.get(1).map(String::as_str) == Some("hello") {
        args.remove(1);
    }
    let rest = &args[1..];
    if rest.iter().any(|a| a == "--help" || a == "-h") {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let deps = rest.iter().any(|a| a == "--deps");
    let manifest = rest
        .iter()
        .position(|a| a == "--manifest-path")
        .and_then(|i| rest.get(i + 1));

    // Cargo가 하위 명령을 부를 때 넣어 주는 CARGO 환경 변수로, 같은 cargo를 다시 부릅니다.
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(&cargo);
    cmd.args(["metadata", "--format-version", "1"]);
    if !deps {
        cmd.arg("--no-deps"); // 의존성 해석을 하지 않아 빠르고 네트워크도 쓰지 않습니다
    }
    if let Some(m) = manifest {
        cmd.args(["--manifest-path", m]);
    }
    let out = cmd.output().expect("cargo metadata 실행");
    if !out.status.success() {
        eprint!("{}", String::from_utf8_lossy(&out.stderr));
        return ExitCode::FAILURE;
    }
    // Value는 JSON 값 하나를 나타내는 열거형입니다. meta["키"]·v[0]은 없으면 패닉하지 않고 Value::Null을,
    // as_str()·as_array()는 그 종류의 값일 때만 Some을 돌려줍니다.
    let meta: Value = serde_json::from_slice(&out.stdout).expect("cargo metadata의 JSON");

    println!(
        "CARGO 환경 변수: {}",
        if std::env::var("CARGO").is_ok() {
            "있음"
        } else {
            "없음"
        }
    );
    println!(
        "워크스페이스 루트: {}",
        last(meta["workspace_root"].as_str().unwrap_or("?"))
    );
    println!(
        "target 디렉터리: {}",
        last(meta["target_directory"].as_str().unwrap_or("?"))
    );

    let members: Vec<&str> = meta["workspace_members"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let packages = meta["packages"].as_array().cloned().unwrap_or_default();
    println!("멤버 {}개", members.len());
    for pkg in packages
        .iter()
        .filter(|p| members.contains(&p["id"].as_str().unwrap_or("")))
    {
        let targets: Vec<String> = pkg["targets"]
            .as_array()
            .map(|ts| {
                ts.iter()
                    .map(|t| {
                        format!(
                            "{}:{}",
                            t["kind"][0].as_str().unwrap_or("?"),
                            t["name"].as_str().unwrap_or("?")
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        println!(
            "  {} {}  [{}]",
            pkg["name"].as_str().unwrap_or("?"),
            pkg["version"].as_str().unwrap_or("?"),
            targets.join(", ")
        );
    }

    if deps {
        // resolve.nodes: 해석이 끝난 그래프. 노드마다 그 패키지가 직접 쓰는 패키지의 ID 목록(dependencies)이 있습니다.
        let nodes = meta["resolve"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        println!("그래프의 패키지 {}개", nodes.len());
        for node in nodes
            .iter()
            .filter(|n| members.contains(&n["id"].as_str().unwrap_or("")))
        {
            let name = |id: &str| -> String {
                packages
                    .iter()
                    .find(|p| p["id"] == id)
                    .map(|p| {
                        format!(
                            "{}@{}",
                            p["name"].as_str().unwrap_or("?"),
                            p["version"].as_str().unwrap_or("?")
                        )
                    })
                    .unwrap_or_else(|| id.to_string())
            };
            let direct: Vec<String> = node["dependencies"]
                .as_array()
                .map(|ds| ds.iter().filter_map(Value::as_str).map(name).collect())
                .unwrap_or_default();
            println!(
                "  {} → {}",
                name(node["id"].as_str().unwrap_or("")),
                if direct.is_empty() {
                    "(없음)".to_string()
                } else {
                    direct.join(", ")
                }
            );
        }
    }
    ExitCode::SUCCESS
}
