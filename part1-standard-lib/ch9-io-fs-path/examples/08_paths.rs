// Path와 PathBuf로 경로를 나누고, 조립하고, 비교합니다.
//
//   1) Path와 PathBuf의 관계는 str과 String의 관계와 같습니다(빌린 것 / 소유한 것).
//   2) 경로 나누기: parent, file_name, file_stem, extension
//   3) 경로 조립: join, push, pop, with_extension, with_file_name
//   4) 경로 연산은 문자열이 아니라 구성 요소(component) 단위입니다.
//   5) 출력에는 display()가 필요하고, &str로 바꾸는 to_str()은 실패할 수 있습니다.
//   6) impl AsRef<Path>를 받으면 &str, String, &Path, PathBuf를 모두 받을 수 있습니다.
//   7) 경로 연산은 파일 시스템을 보지 않습니다. exists, canonicalize는 파일 시스템에 물어봅니다.
//
// 실행: cargo run -p p1-ch9-io-fs-path --example 08_paths
//
// 결과는 유닉스(macOS, Linux) 기준입니다. Windows는 구분자(\)와 루트(C:\ 등)가 달라 결과가 다릅니다.
// 7번 구역은 std::env::temp_dir() 아래에 이 예제 전용 디렉터리를 만들고, 끝날 때 지웁니다.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

// 6번 구역: 경로처럼 쓸 수 있는 것이면 무엇이든 받습니다. std::fs의 함수들이 이렇게 생겼습니다.
fn kind_of(path: impl AsRef<Path>) -> &'static str {
    let path = path.as_ref(); // 여기서부터는 &Path입니다.
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => "Rust 소스",
        Some("md") => "마크다운",
        Some(_) => "기타 파일",
        None => "확장자 없음",
    }
}

// &Path를 받는 함수에는 &PathBuf도 넘길 수 있습니다(&String → &str과 같은 역참조 강제 변환).
fn depth(path: &Path) -> usize {
    path.components().count()
}

fn main() -> io::Result<()> {
    println!("1. Path는 빌린 경로, PathBuf는 소유한 경로입니다");
    let borrowed: &Path = Path::new("docs/guide.md"); // 문자열을 빌려 Path로 봅니다(할당 없음).
    let mut owned: PathBuf = borrowed.to_path_buf(); // 새 버퍼에 복사해 소유합니다.
    owned.set_extension("txt"); // 소유한 쪽만 고칠 수 있습니다.
    println!("    borrowed = {borrowed:?}, owned = {owned:?}");
    println!(
        "    depth(&owned) = {}, depth(Path::new(\"a/b/c\")) = {}",
        depth(&owned),
        depth(Path::new("a/b/c"))
    );
    // 출력:
    //     borrowed = "docs/guide.md", owned = "docs/guide.txt"
    //     depth(&owned) = 2, depth(Path::new("a/b/c")) = 3

    println!("2. 경로 나누기");
    for text in [
        "archive/2024/report.tar.gz",
        "/home/user/.bashrc",
        "notes/",
        "/",
    ] {
        let path = Path::new(text);
        println!(
            "    {text:<27} parent={:?} file_name={:?} stem={:?} ext={:?}",
            path.parent(),
            path.file_name(),
            path.file_stem(),
            path.extension()
        );
    }
    // 출력:
    //     archive/2024/report.tar.gz  parent=Some("archive/2024") file_name=Some("report.tar.gz") stem=Some("report.tar") ext=Some("gz")
    //     /home/user/.bashrc          parent=Some("/home/user") file_name=Some(".bashrc") stem=Some(".bashrc") ext=None
    //     notes/                      parent=Some("") file_name=Some("notes") stem=Some("notes") ext=None
    //     /                           parent=None file_name=None stem=None ext=None

    println!("3. 경로 조립");
    let base = Path::new("/srv/app");
    println!(
        "    join(\"logs/today.log\") → {:?}",
        base.join("logs/today.log")
    );
    // 절대 경로를 붙이면 앞의 경로가 통째로 바뀝니다. 흔한 실수라서 clippy가 join_absolute_paths로 경고합니다.
    #[expect(
        clippy::join_absolute_paths,
        reason = "절대 경로를 join하면 어떻게 되는지 보여 주기 위함"
    )]
    let replaced = base.join("/etc/hosts");
    println!("    join(\"/etc/hosts\")     → {replaced:?}");
    let mut path = PathBuf::from("/srv/app");
    path.push("config");
    path.push("app.toml");
    println!("    push 두 번             → {path:?}");
    path.pop(); // 마지막 구성 요소를 떼어 냅니다(parent로 바꿈).
    println!("    pop                    → {path:?}");
    let report = Path::new("out/report.md");
    println!(
        "    with_extension(\"html\") → {:?}",
        report.with_extension("html")
    );
    println!(
        "    with_file_name(\"a.md\") → {:?}",
        report.with_file_name("a.md")
    );
    let collected: PathBuf = ["usr", "local", "bin"].iter().collect(); // 구성 요소를 모아 만듭니다.
    println!("    [\"usr\",\"local\",\"bin\"] → {collected:?}");
    // 출력:
    //     join("logs/today.log") → "/srv/app/logs/today.log"
    //     join("/etc/hosts")     → "/etc/hosts"
    //     push 두 번             → "/srv/app/config/app.toml"
    //     pop                    → "/srv/app/config"
    //     with_extension("html") → "out/report.html"
    //     with_file_name("a.md") → "out/a.md"
    //     ["usr","local","bin"] → "usr/local/bin"

    println!("4. 비교와 검사는 구성 요소 단위입니다");
    let messy = Path::new("/usr//local/./bin/../lib/");
    let parts: Vec<Component> = messy.components().collect();
    println!("    components → {parts:?}");
    println!(
        "    Path \"a//b/\" == \"a/b\" → {}, 문자열로는 → {}",
        Path::new("a//b/") == Path::new("a/b"),
        "a//b/" == "a/b"
    );
    let passwd = Path::new("/etc/passwd");
    println!(
        "    /etc/passwd starts_with \"/e\" → Path: {}, str: {}",
        passwd.starts_with("/e"),
        "/etc/passwd".starts_with("/e")
    );
    let conf = Path::new("/etc/resolv.conf");
    println!(
        "    /etc/resolv.conf ends_with \"conf\" → {}, extension() → {:?}",
        conf.ends_with("conf"),
        conf.extension()
    );
    let file = Path::new("/srv/app/logs/today.log");
    println!(
        "    strip_prefix(\"/srv/app\") → {:?}",
        file.strip_prefix("/srv/app")
    );
    println!(
        "    strip_prefix(\"/srv/ap\")  → {:?}",
        file.strip_prefix("/srv/ap")
    );
    let ancestors: Vec<&Path> = file.ancestors().collect();
    println!("    ancestors → {ancestors:?}");
    // 출력:
    //     components → [RootDir, Normal("usr"), Normal("local"), Normal("bin"), ParentDir, Normal("lib")]
    //     Path "a//b/" == "a/b" → true, 문자열로는 → false
    //     /etc/passwd starts_with "/e" → Path: false, str: true
    //     /etc/resolv.conf ends_with "conf" → false, extension() → Some("conf")
    //     strip_prefix("/srv/app") → Ok("logs/today.log")
    //     strip_prefix("/srv/ap")  → Err(StripPrefixError(()))
    //     ancestors → ["/srv/app/logs/today.log", "/srv/app/logs", "/srv/app", "/srv", "/"]

    println!("5. 출력과 문자열 변환");
    let memo = Path::new("문서/회의 메모.txt");
    println!("    display() → {}", memo.display()); // 사람에게 보여 줄 때
    println!("    {{:?}}      → {memo:?}"); // 따옴표와 이스케이프가 붙는 디버그 형식
    println!("    to_str()  → {:?}", memo.to_str()); // UTF-8이 아니면 None
    println!("    to_string_lossy() → {:?}", memo.to_string_lossy()); // 고칠 수 없는 바이트는 U+FFFD로
    // 출력:
    //     display() → 문서/회의 메모.txt
    //     {:?}      → "문서/회의 메모.txt"
    //     to_str()  → Some("문서/회의 메모.txt")
    //     to_string_lossy() → "문서/회의 메모.txt"

    println!("6. impl AsRef<Path>는 경로처럼 쓸 수 있는 것을 모두 받습니다");
    let text = String::from("src/main.rs");
    let buf = PathBuf::from("README.md");
    println!("    &str     → {}", kind_of("lib.rs"));
    println!("    &String  → {}", kind_of(&text));
    println!("    String   → {}", kind_of(text)); // 값을 넘기면 이동합니다.
    println!("    &PathBuf → {}", kind_of(&buf));
    println!("    &Path    → {}", kind_of(buf.as_path()));
    println!("    PathBuf  → {}", kind_of(buf));
    println!("    &str     → {}", kind_of("Makefile"));
    // 출력:
    //     &str     → Rust 소스
    //     &String  → Rust 소스
    //     String   → Rust 소스
    //     &PathBuf → 마크다운
    //     &Path    → 마크다운
    //     PathBuf  → 마크다운
    //     &str     → 확장자 없음

    println!("7. 경로 연산은 파일 시스템을 보지 않습니다");
    let ghost = Path::new("no-such-dir/plan.txt");
    println!(
        "    없는 경로도 나눌 수 있음: extension={:?}, parent={:?}",
        ghost.extension(),
        ghost.parent()
    );
    println!(
        "    exists() = {}, fs::exists() = {:?}",
        ghost.exists(),
        fs::exists(ghost)
    );
    match ghost.canonicalize() {
        Ok(p) => println!("    canonicalize → {p:?}"),
        Err(e) => println!("    canonicalize → 에러 {:?}", e.kind()),
    }
    // 실제 디렉터리에서 ".."가 든 경로를 만들어 봅니다.
    let dir = std::env::temp_dir().join(format!("p1-ch9-08-{}", std::process::id()));
    fs::create_dir_all(dir.join("sub"))?;
    fs::write(dir.join("data.txt"), "x")?;
    let winding = dir.join("sub/../data.txt");
    let has_parent_dir = |p: &Path| p.components().any(|c| c == Component::ParentDir);
    println!(
        "    sub/../data.txt: 구성 요소에 \"..\"가 남아 있나 {}, exists() = {}",
        has_parent_dir(&winding),
        winding.exists()
    );
    let canonical = winding.canonicalize()?; // 파일 시스템을 따라가며 ".."와 심볼릭 링크를 풉니다.
    let canonical_dir = dir.canonicalize()?;
    println!(
        "    canonicalize 뒤: \"..\"가 남아 있나 {}, 디렉터리 기준 상대 경로 = {:?}",
        has_parent_dir(&canonical),
        canonical.strip_prefix(&canonical_dir).unwrap()
    );
    // temp_dir()이 심볼릭 링크를 거치는 경로라면 canonicalize 결과의 앞부분이 달라집니다(환경마다 다름).
    println!(
        "    temp_dir() 쪽 경로와 canonicalize 결과가 같은가 {}",
        canonical_dir == dir
    );
    fs::remove_dir_all(&dir)?;
    // 출력:
    //     없는 경로도 나눌 수 있음: extension=Some("txt"), parent=Some("no-such-dir")
    //     exists() = false, fs::exists() = Ok(false)
    //     canonicalize → 에러 NotFound
    //     sub/../data.txt: 구성 요소에 ".."가 남아 있나 true, exists() = true
    //     canonicalize 뒤: ".."가 남아 있나 false, 디렉터리 기준 상대 경로 = "data.txt"
    //     temp_dir() 쪽 경로와 canonicalize 결과가 같은가 false
    // (마지막 줄은 macOS 결과입니다. macOS의 임시 디렉터리는 /var/... 아래에 있는데 /var가
    //  /private/var를 가리키는 심볼릭 링크라서 false가 됩니다. 다른 환경에서는 true일 수 있습니다.)

    Ok(())
}
