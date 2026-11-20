//! 지침 2: 도구는 사람이 읽는 출력이 아니라 정해진 기계용 출력을 읽습니다.
//!
//! `projects/meta_lint/sample_ws`(패키지 3개짜리 워크스페이스)에 `cargo metadata`와
//! `cargo tree`를 실행해 출력의 구조를 보고, 같은 질문에 사람용 출력이나 JSON 글자를
//! 긁어 답한 판과 JSON 구조를 따라 답한 판을 비교합니다.
//!
//! 외부 크레이트 없이 std만 쓰려고 JSON을 읽는 작은 파서를 직접 두었습니다.
//! 실제 도구는 `cargo_metadata` 크레이트로 타입을 통해 읽습니다(projects/meta_lint).
//! 두 명령 모두 빌드하지 않고, `--offline --locked`로 네트워크 접근과 Cargo.lock
//! 변경을 막습니다.
//!
//! 실행: cargo run -p er-ch31-tooling-ecosystem --example 02_read_metadata

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

// ---------------------------------------------------------------------------
// 작은 JSON 파서. cargo가 만든 올바른 JSON만 읽는다고 보고, 형식이 틀리면 패닉합니다.
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Json {
    Null,
    Bool(bool),
    /// 숫자는 이 예제에서 계산하지 않으므로 글자 그대로 둡니다.
    Num(String),
    Str(String),
    Arr(Vec<Json>),
    /// 키 순서를 보존하려고 맵 대신 벡터에 담습니다.
    Obj(Vec<(String, Json)>),
}

static NULL: Json = Json::Null;

impl Json {
    /// 객체의 키를 찾습니다. 없거나 객체가 아니면 null을 돌려줍니다.
    fn get(&self, key: &str) -> &Json {
        match self {
            Json::Obj(fields) => fields
                .iter()
                .find(|(k, _)| k == key)
                .map_or(&NULL, |(_, v)| v),
            _ => &NULL,
        }
    }
    fn str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    fn arr(&self) -> &[Json] {
        match self {
            Json::Arr(items) => items,
            _ => &[],
        }
    }
    fn keys(&self) -> Vec<&str> {
        match self {
            Json::Obj(fields) => fields.iter().map(|(k, _)| k.as_str()).collect(),
            _ => Vec::new(),
        }
    }
    /// 출력용 짧은 표기입니다.
    fn brief(&self) -> String {
        match self {
            Json::Null => "null".to_string(),
            Json::Bool(b) => b.to_string(),
            Json::Num(n) => n.clone(),
            Json::Str(s) => format!("{s:?}"),
            Json::Arr(items) => format!("배열({}개)", items.len()),
            Json::Obj(fields) => format!("객체({}개 키)", fields.len()),
        }
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn parse(text: &str) -> Json {
        let mut p = Parser {
            s: text.as_bytes(),
            i: 0,
        };
        let value = p.value();
        p.ws();
        assert_eq!(p.i, p.s.len(), "JSON 뒤에 글자가 남았습니다");
        value
    }

    fn ws(&mut self) {
        while self.s[self.i..]
            .first()
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.i += 1;
        }
    }

    fn next(&mut self) -> u8 {
        let b = self.s[self.i];
        self.i += 1;
        b
    }

    fn literal(&mut self, word: &str, value: Json) -> Json {
        assert!(
            self.s[self.i..].starts_with(word.as_bytes()),
            "{word}가 아닙니다"
        );
        self.i += word.len();
        value
    }

    fn value(&mut self) -> Json {
        self.ws();
        match self.s[self.i] {
            b'{' => {
                self.i += 1;
                let mut fields = Vec::new();
                loop {
                    self.ws();
                    match self.next() {
                        b'}' => return Json::Obj(fields),
                        b',' => continue,
                        b'"' => {
                            let key = self.string();
                            self.ws();
                            assert_eq!(self.next(), b':', "키 뒤에 ':'가 없습니다");
                            let value = self.value();
                            fields.push((key, value));
                        }
                        b => panic!("객체 안에 예상하지 못한 글자 {:?}", b as char),
                    }
                }
            }
            b'[' => {
                self.i += 1;
                let mut items = Vec::new();
                loop {
                    self.ws();
                    match self.s[self.i] {
                        b']' => {
                            self.i += 1;
                            return Json::Arr(items);
                        }
                        b',' => self.i += 1,
                        _ => items.push(self.value()),
                    }
                }
            }
            b'"' => {
                self.i += 1;
                Json::Str(self.string())
            }
            b't' => self.literal("true", Json::Bool(true)),
            b'f' => self.literal("false", Json::Bool(false)),
            b'n' => self.literal("null", Json::Null),
            _ => {
                let start = self.i;
                while self.s[self.i..]
                    .first()
                    .is_some_and(|b| b"+-.eE0123456789".contains(b))
                {
                    self.i += 1;
                }
                let digits = std::str::from_utf8(&self.s[start..self.i]).expect("숫자");
                Json::Num(digits.to_string())
            }
        }
    }

    /// 여는 따옴표 다음부터 닫는 따옴표까지 읽습니다. cargo는 한글 같은 문자를
    /// UTF-8 그대로 내보내므로 바이트를 모았다가 마지막에 한 번에 바꿉니다.
    fn string(&mut self) -> String {
        let mut out = Vec::new();
        loop {
            match self.next() {
                b'"' => break,
                b'\\' => match self.next() {
                    b'n' => out.push(b'\n'),
                    b't' => out.push(b'\t'),
                    b'r' => out.push(b'\r'),
                    b'b' => out.push(0x08),
                    b'f' => out.push(0x0c),
                    b'u' => {
                        let hex = std::str::from_utf8(&self.s[self.i..self.i + 4]).expect("hex");
                        self.i += 4;
                        let code = u32::from_str_radix(hex, 16).expect("hex");
                        // 서로게이트 쌍은 이 예제의 입력에 나오지 않으므로 대체 문자로 둡니다.
                        let c = char::from_u32(code).unwrap_or(char::REPLACEMENT_CHARACTER);
                        out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
                    }
                    other => out.push(other), // \" \\ \/
                },
                b => out.push(b),
            }
        }
        String::from_utf8(out).expect("JSON 문자열은 UTF-8입니다")
    }
}

// ---------------------------------------------------------------------------
// cargo 실행
// ---------------------------------------------------------------------------

/// `--format-version`을 적고 의존성 해석을 건너뜁니다(`resolve`가 null).
const NO_DEPS: &[&str] = &[
    "metadata",
    "--format-version",
    "1",
    "--no-deps",
    "--offline",
];
/// 의존성 해석까지 합니다. `--locked`는 Cargo.lock을 바꾸지 않겠다는 약속입니다.
const FULL: &[&str] = &["metadata", "--format-version", "1", "--offline", "--locked"];

/// `cargo run`이 넣어 주는 `CARGO`(지금 쓰는 cargo의 경로)를 씁니다.
fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

fn sample_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("projects/meta_lint/sample_ws/Cargo.toml")
}

/// cargo 하위 명령을 sample_ws에 실행하고 (표준 출력, 표준 에러)를 돌려줍니다.
fn run_cargo(args: &[&str]) -> (String, String) {
    let out = Command::new(cargo())
        .args(args)
        .arg("--manifest-path")
        .arg(sample_manifest())
        .output()
        .expect("cargo를 실행하지 못했습니다");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(out.status.success(), "cargo {args:?} 실패:\n{stderr}");
    (String::from_utf8_lossy(&out.stdout).into_owned(), stderr)
}

// ---------------------------------------------------------------------------
// 출력 읽기
// ---------------------------------------------------------------------------

/// 패키지 ID → 패키지 이름. ID의 모양은 Cargo가 정하는 불투명한 값이므로
/// 글자를 잘라 이름을 얻지 않고, packages 배열에서 찾아 봅니다.
fn name_of<'a>(meta: &'a Json, id: &str) -> &'a str {
    meta.get("packages")
        .arr()
        .iter()
        .find(|p| p.get("id").str() == Some(id))
        .and_then(|p| p.get("name").str())
        .expect("packages에 없는 ID입니다")
}

/// 경로 값은 절대 경로라서 워크스페이스 루트를 떼고 보여 줍니다.
fn relative<'a>(path: &'a str, root: &str) -> &'a str {
    path.strip_prefix(root)
        .unwrap_or(path)
        .trim_start_matches('/')
}

/// 의존성 종류는 "dev", "build", 또는 null(일반 의존성)입니다.
fn kind_of(dep: &Json) -> &str {
    dep.get("kind").str().unwrap_or("normal")
}

fn show_packages(meta: &Json) {
    let root = meta.get("workspace_root").str().expect("workspace_root");
    let mut packages: Vec<&Json> = meta.get("packages").arr().iter().collect();
    packages.sort_by_key(|p| p.get("name").str());
    println!("  packages {}개 (이름순):", packages.len());
    for p in packages {
        // publish: null이면 제한 없음, 빈 배열이면 게시 금지(publish = false)
        let publish = match p.get("publish") {
            Json::Null => "제한 없음".to_string(),
            Json::Arr(regs) if regs.is_empty() => "막힘".to_string(),
            other => other.brief(),
        };
        let manifest = p.get("manifest_path").str().expect("manifest_path");
        println!(
            "    {} {}  {}  publish: {publish}, license: {}",
            p.get("name").str().unwrap_or("?"),
            p.get("version").str().unwrap_or("?"),
            relative(manifest, root),
            p.get("license").brief(),
        );
        for d in p.get("dependencies").arr() {
            let path = d.get("path").str().map_or("", |_| ", path");
            println!(
                "      의존성 {}  req {}  ({}{path})",
                d.get("name").str().unwrap_or("?"),
                d.get("req").brief(),
                kind_of(d),
            );
        }
    }
}

fn show_resolve(full: &Json) {
    let resolve = full.get("resolve");
    println!("  resolve의 키: {}", resolve.keys().join(", "));
    println!(
        "  resolve.root = {} (가상 매니페스트라 '현재 패키지'가 없음)",
        resolve.get("root").brief()
    );
    let mut nodes: Vec<&Json> = resolve.get("nodes").arr().iter().collect();
    nodes.sort_by_key(|n| name_of(full, n.get("id").str().expect("id")));
    println!("  resolve.nodes {}개:", nodes.len());
    for n in nodes {
        let from = name_of(full, n.get("id").str().expect("id"));
        // deps[].name은 패키지 이름이 아니라 라이브러리 타깃 이름(밑줄)입니다.
        let deps: Vec<String> = n
            .get("deps")
            .arr()
            .iter()
            .map(|d| {
                let pkg = name_of(full, d.get("pkg").str().expect("pkg"));
                let lib = d.get("name").str().expect("name");
                let kind = kind_of(&d.get("dep_kinds").arr()[0]);
                format!("{pkg} [{kind}, name={lib}]")
            })
            .collect();
        if deps.is_empty() {
            println!("    {from} → (없음)");
        } else {
            println!("    {from} → {}", deps.join(", "));
        }
    }
}

/// resolve 그래프를 따라가며 `start`가 직간접으로 의존하는 패키지 이름을 모읍니다.
/// 같은 패키지를 두 번 만나도 한 번만 셉니다.
fn reachable<'a>(full: &'a Json, start: &str) -> BTreeSet<&'a str> {
    let nodes = full.get("resolve").get("nodes").arr();
    let mut seen = BTreeSet::new();
    let mut stack = vec![start];
    while let Some(id) = stack.pop() {
        let node = nodes
            .iter()
            .find(|n| n.get("id").str() == Some(id))
            .expect("resolve에 없는 ID입니다");
        for d in node.get("deps").arr() {
            let pkg = d.get("pkg").str().expect("pkg");
            if seen.insert(name_of(full, pkg)) {
                stack.push(pkg);
            }
        }
    }
    seen
}

fn main() {
    println!("대상: projects/meta_lint/sample_ws/Cargo.toml");

    println!("\n1. --format-version 없이 실행하면");
    let (_, stderr) = run_cargo(&["metadata", "--no-deps", "--offline"]);
    for line in stderr.lines() {
        println!("  {line}");
    }
    // 출력: warning: please specify `--format-version` flag explicitly to avoid compatibility problems

    println!("\n2. cargo metadata --format-version 1 --no-deps");
    let (no_deps_text, _) = run_cargo(NO_DEPS);
    let meta = Parser::parse(&no_deps_text);
    println!("  최상위 키: {}", meta.keys().join(", "));
    let version = meta.get("version").brief();
    let resolve = meta.get("resolve").brief();
    println!("  version = {version}, resolve = {resolve}");
    // 출력: version = 1, resolve = null
    let members = meta.get("workspace_members").arr().len();
    let defaults = meta.get("workspace_default_members").arr().len();
    println!("  workspace_members {members}개, workspace_default_members {defaults}개");
    // 출력: workspace_members 3개, workspace_default_members 3개
    show_packages(&meta);

    println!("\n3. cargo metadata --format-version 1 (--no-deps 없이, 의존성 해석 포함)");
    let (full_text, _) = run_cargo(FULL);
    let full = Parser::parse(&full_text);
    show_resolve(&full);

    println!("\n4. 같은 질문을 두 가지로 읽기");
    println!("  질문 A: 워크스페이스의 패키지는 몇 개인가?");
    let scraped = no_deps_text.matches("\"name\":").count();
    println!("    비권장: JSON 글자에서 \"name\": 을 센 수 = {scraped}");
    // 출력: 비권장: JSON 글자에서 "name": 을 센 수 = 9
    let counted = meta.get("packages").arr().len();
    println!("    권장: packages 배열의 길이 = {counted}");
    // 출력: 권장: packages 배열의 길이 = 3

    println!("  질문 B: ledger-cli가 직간접으로 의존하는 패키지는 몇 개인가?");
    let (tree, _) = run_cargo(&["tree", "-p", "ledger-cli", "--offline", "--locked"]);
    let root = meta.get("workspace_root").str().expect("workspace_root");
    for line in tree.lines() {
        println!("      {}", line.replace(root, "…"));
    }
    let lines = tree.lines().count() - 1;
    println!("    비권장: 위 cargo tree 출력의 줄 수 - 1 = {lines}");
    // 출력: 비권장: 위 cargo tree 출력의 줄 수 - 1 = 3
    let cli_id = full
        .get("packages")
        .arr()
        .iter()
        .find(|p| p.get("name").str() == Some("ledger-cli"))
        .and_then(|p| p.get("id").str())
        .expect("ledger-cli");
    let deps: Vec<&str> = reachable(&full, cli_id).into_iter().collect();
    println!(
        "    권장: resolve 그래프를 따라 센 수 = {} ({})",
        deps.len(),
        deps.join(", ")
    );
    // 출력: 권장: resolve 그래프를 따라 센 수 = 2 (ledger-core, ledger-report)
}
