// io::Error의 kind()로 에러의 종류를 구분해, 종류마다 다르게 대응합니다.
// "파일이 없으면 새로 만들고, 그 밖의 에러는 알린다"는 The Book 9.2절의 예를 따라 합니다.
//
// 실행: cargo run -p p0-ch9-error-handling --example 03_error_kind
//
// 파일은 임시 디렉터리(std::env::temp_dir())에 만들고, 끝날 때 지웁니다.
// 4번 구역의 에러 종류와 메시지는 운영체제에 따라 다를 수 있습니다(macOS에서 확인).

use std::fs::{self, File};
use std::io::ErrorKind;
use std::path::Path;

// Path는 파일 경로를 빌려 보는 타입이고, PathBuf는 경로를 소유하는 타입입니다.
// &str과 String의 관계와 같습니다(Part 1 ch9-io-fs-path에서 자세히 다룹니다).
fn open_or_create(path: &Path) {
    match File::open(path) {
        Ok(_file) => println!("    있는 파일을 열었습니다"),
        // 바깥 match: 성공인가 실패인가
        Err(error) => match error.kind() {
            // 안쪽 match: 실패했다면 어떤 종류의 실패인가
            ErrorKind::NotFound => match File::create(path) {
                Ok(_file) => println!("    파일이 없어서 새로 만들었습니다"),
                Err(e) => println!("    파일이 없어서 만들려고 했지만 실패했습니다: {e}"),
            },
            // 따로 대응하지 않는 종류는 이름 있는 포괄 패턴(ch6)으로 받아 종류와 메시지를 알립니다.
            // ErrorKind는 앞으로 배리언트가 늘어날 수 있다고 표시된(#[non_exhaustive]) 열거형이라,
            // 다른 크레이트에서 match할 때는 포괄 갈래가 반드시 있어야 합니다. 없으면 E0004입니다.
            other => println!("    파일을 열지 못했습니다 [{other:?}]: {error}"),
        },
    }
}

fn main() {
    let dir = std::env::temp_dir();
    let path = dir.join("p0_ch9_03_memo.txt");

    // 이전 실행이 중간에 멈춰 파일이 남아 있을 수 있으므로 먼저 지웁니다.
    // 파일이 없으면 remove_file이 Err를 돌려주지만, 여기서는 어느 쪽이든 상관없으므로
    // let _ =로 "결과를 일부러 버린다"고 표시합니다. (그냥 버리면 unused_must_use 경고가 납니다.)
    let _ = fs::remove_file(&path);

    println!("1. 처음에는 파일이 없으므로 NotFound 갈래에서 새로 만듭니다");
    open_or_create(&path);
    // 출력: 파일이 없어서 새로 만들었습니다

    println!("2. 같은 경로를 다시 열면 이번에는 Ok입니다");
    open_or_create(&path);
    // 출력: 있는 파일을 열었습니다

    println!("3. 없는 디렉터리 안의 파일은 열 수도, 만들 수도 없습니다");
    open_or_create(&dir.join("p0_ch9_03_no_such_dir").join("memo.txt"));
    // 출력: 파일이 없어서 만들려고 했지만 실패했습니다: No such file or directory (os error 2)
    // File::open과 File::create가 둘 다 NotFound로 실패합니다. 안쪽 match의 Err 갈래입니다.

    println!("4. NotFound가 아닌 에러는 포괄 패턴 갈래로 갑니다");
    // memo.txt는 파일인데 그 아래에 다른 파일이 있는 것처럼 경로를 만들었습니다.
    open_or_create(&path.join("child.txt"));
    // 출력: 파일을 열지 못했습니다 [NotADirectory]: Not a directory (os error 20)

    println!("5. File::create_new는 이미 있는 파일을 덮어쓰지 않고 AlreadyExists로 실패합니다");
    match File::create_new(&path) {
        Ok(_file) => println!("    새 파일을 만들었습니다"),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => println!("    이미 있는 파일이라 만들지 않았습니다"),
            other => println!("    만들지 못했습니다 [{other:?}]: {e}"),
        },
    }
    // 출력: 이미 있는 파일이라 만들지 않았습니다

    // 정리: 만든 파일을 지웁니다. 지우지 못해도 예제의 결과에는 영향이 없으므로 결과를 버립니다.
    let _ = fs::remove_file(&path);
}
