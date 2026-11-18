// 기대하는 에러: E0277 — `Path` doesn't implement `std::fmt::Display` (와 `OsStr`도 같은 에러)
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_path_display.rs --out-dir /tmp/rust-p1-ch9
//
// 경로는 운영체제의 문자열이라 UTF-8이 아닐 수 있습니다. 그래서 Path와 OsStr은 Display를 구현하지 않고,
// UTF-8이 아닌 부분을 어떻게 보여 줄지 정해 둔 display()를 거쳐 출력하게 합니다.
// file_name()이 돌려주는 &OsStr도 같은 이유로 {}로 출력할 수 없습니다.
// 해결: path.display(), name.display()로 출력합니다. 디버그 형식 {:?}은 바로 쓸 수 있습니다.
//       (examples/08_paths.rs의 5번 구역)

use std::path::Path;

fn main() {
    let path = Path::new("notes/2024/todo.txt");
    println!("파일: {}", path);
    let name = path.file_name().unwrap();
    println!("이름: {name}");
}
