// 디렉터리를 만들고, 읽고, 재귀로 순회하고, 지웁니다.
//
//   1) create_dir_all은 중간 디렉터리까지 만들고, 이미 있어도 성공합니다. create_dir은 그렇지 않습니다.
//   2) read_dir은 항목을 정해진 순서 없이 내줍니다. 같은 결과가 필요하면 직접 정렬합니다.
//   3) 하위 디렉터리까지 내려가는 순회는 read_dir을 재귀로 부르는 함수로 만듭니다.
//   4) 순회 결과를 경로 연산(extension, strip_prefix)으로 걸러 내고 정리합니다.
//   5) remove_dir은 빈 디렉터리만, remove_dir_all은 내용까지 모두 지웁니다.
//
// 실행: cargo run -p p1-ch9-io-fs-path --example 07_directories
//
// std::env::temp_dir() 아래에 이 예제 전용 디렉터리(p1-ch9-07-<프로세스 ID>)를 만들고, 끝날 때 지웁니다.
// 출력에는 그 디렉터리를 기준으로 한 상대 경로만 씁니다.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// 2번 구역: 한 디렉터리의 항목을 이름순으로 정렬해 돌려줍니다.
// read_dir의 각 항목은 io::Result<DirEntry>이므로 collect로 한꺼번에 에러를 확인합니다(ch5).
fn sorted_entries(dir: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(dir)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

// 3번 구역: 들여쓰기로 깊이를 보여 주며 재귀로 출력합니다.
fn print_tree(dir: &Path, depth: usize) -> io::Result<()> {
    for entry in sorted_entries(dir)? {
        let indent = "  ".repeat(depth + 3);
        // file_type()은 심볼릭 링크를 따라가지 않습니다. 링크를 디렉터리로 착각해 끝없이 도는 일이 없습니다.
        let file_type = entry.file_type()?;
        // file_name()은 OsString(운영체제의 문자열)을 돌려주므로 display()로 출력합니다.
        let name = entry.file_name();
        if file_type.is_dir() {
            println!("{indent}{}/", name.display());
            print_tree(&entry.path(), depth + 1)?; // entry.path()는 dir.join(이름)과 같습니다.
        } else {
            println!(
                "{indent}{} ({}바이트)",
                name.display(),
                entry.metadata()?.len()
            );
        }
    }
    Ok(())
}

// 4번 구역: 하위 디렉터리까지 모든 파일의 경로를 모읍니다.
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_files(&entry.path(), out)?;
        } else {
            out.push(entry.path());
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let root = std::env::temp_dir().join(format!("p1-ch9-07-{}", std::process::id()));

    println!("1. 디렉터리 만들기");
    // 중간의 project, src가 없어도 한 번에 만듭니다.
    fs::create_dir_all(root.join("project/src/util"))?;
    fs::create_dir_all(root.join("project/tests"))?;
    fs::create_dir_all(root.join("project/target"))?;
    let again = fs::create_dir_all(root.join("project/src")); // 이미 있어도 성공
    println!("    create_dir_all(이미 있음)   → {again:?}");
    match fs::create_dir(root.join("project/src")) {
        Ok(()) => println!("    create_dir(이미 있음)       → 성공"),
        Err(e) => println!("    create_dir(이미 있음)       → {:?}", e.kind()),
    }
    match fs::create_dir(root.join("project/docs/api")) {
        Ok(()) => println!("    create_dir(부모가 없음)     → 성공"),
        Err(e) => println!("    create_dir(부모가 없음)     → {:?}", e.kind()),
    }
    let files = [
        ("project/Cargo.toml", "[package]\nname = \"demo\"\n"),
        ("project/README.md", "# demo\n"),
        ("project/src/main.rs", "fn main() {}\n"),
        ("project/src/lib.rs", "pub mod util;\n"),
        ("project/src/util/mod.rs", "pub mod text;\n"),
        ("project/src/util/text.rs", "pub fn hi() {}\n"),
        ("project/tests/basic.rs", "#[test]\nfn ok() {}\n"),
    ];
    for (path, contents) in files {
        fs::write(root.join(path), contents)?;
    }
    // 출력:
    //     create_dir_all(이미 있음)   → Ok(())
    //     create_dir(이미 있음)       → AlreadyExists
    //     create_dir(부모가 없음)     → NotFound

    println!("2. read_dir의 항목을 이름순으로 정렬해 봅니다");
    // 정렬하지 않은 순서는 운영체제와 파일 시스템에 따라 다르고, 같은 디렉터리도 실행마다 달라질 수 있습니다.
    for entry in sorted_entries(&root.join("project"))? {
        let kind = if entry.file_type()?.is_dir() {
            "디렉터리"
        } else {
            "파일"
        };
        println!("    {:<12} {kind}", entry.file_name().display());
    }
    // 출력:
    //     Cargo.toml   파일
    //     README.md    파일
    //     src          디렉터리
    //     target       디렉터리
    //     tests        디렉터리

    println!("3. 재귀로 순회하며 트리를 출력합니다");
    println!("    project/");
    print_tree(&root.join("project"), 0)?;
    // 출력:
    //     project/
    //       Cargo.toml (24바이트)
    //       README.md (7바이트)
    //       src/
    //         lib.rs (14바이트)
    //         main.rs (13바이트)
    //         util/
    //           mod.rs (14바이트)
    //           text.rs (15바이트)
    //       target/
    //       tests/
    //         basic.rs (19바이트)

    println!("4. .rs 파일만 골라 project 기준 상대 경로로 정리합니다");
    let project = root.join("project");
    let mut all = Vec::new();
    collect_files(&project, &mut all)?;
    let mut rust_files: Vec<PathBuf> = all
        .iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        // strip_prefix는 앞쪽 경로를 구성 요소 단위로 떼어 냅니다(08 예제).
        .map(|path| path.strip_prefix(&project).unwrap().to_path_buf())
        .collect();
    rust_files.sort(); // read_dir 순서에 기대지 않도록 결과를 정렬합니다.
    let mut total = 0;
    for path in &rust_files {
        total += fs::metadata(project.join(path))?.len();
        println!("    {}", path.display());
    }
    println!(
        "    전체 파일 {}개 중 .rs 파일 {}개, 합계 {total}바이트",
        all.len(),
        rust_files.len()
    );
    // 출력:
    //     src/lib.rs
    //     src/main.rs
    //     src/util/mod.rs
    //     src/util/text.rs
    //     tests/basic.rs
    //     전체 파일 7개 중 .rs 파일 5개, 합계 75바이트

    println!("5. 디렉터리 지우기");
    match fs::remove_dir(&project) {
        Ok(()) => println!("    remove_dir(내용 있음)  → 성공"),
        Err(e) => println!("    remove_dir(내용 있음)  → {:?}", e.kind()),
    }
    let empty = fs::remove_dir(project.join("target")); // 빈 디렉터리는 remove_dir로 지워집니다.
    println!("    remove_dir(빈 target)  → {empty:?}");
    // remove_dir_all은 안의 파일과 하위 디렉터리를 모두 지웁니다. 되돌릴 수 없으니 경로를 꼭 확인합니다.
    fs::remove_dir_all(&root)?;
    println!(
        "    remove_dir_all 뒤 예제 디렉터리가 있나 {}",
        fs::exists(&root)?
    );
    // 출력:
    //     remove_dir(내용 있음)  → DirectoryNotEmpty
    //     remove_dir(빈 target)  → Ok(())
    //     remove_dir_all 뒤 예제 디렉터리가 있나 false

    Ok(())
}
