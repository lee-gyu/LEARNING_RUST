// 기대하는 에러: error[E0716]: temporary value dropped while borrowed
// 핵심 메시지:   creates a temporary value which is freed while still in use
//                temporary value is freed at the end of this statement
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0716_trim_temporary.rs --out-dir /tmp/rust-p1-ch3
//
// format!은 새 String을 만들지만, 그 String을 변수에 담지 않았으므로 이 문장이 끝나면 버려집니다.
// trim()은 그 String을 빌린 &str을 돌려주므로, 다음 줄에서 label을 쓸 때는 가리키던 곳이 이미 없습니다.
// 고치는 방법: 소유자(String)를 먼저 변수에 담아 오래 살게 합니다.
//   let raw = format!("  {} 번 ", number);
//   let label = raw.trim();

fn main() {
    let number = 7;
    let label = format!("  {} 번 ", number).trim();
    println!("[{label}]");
}
