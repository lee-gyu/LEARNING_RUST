// 기대하는 에러: E0382 — borrow of moved value: `s1`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0382_use_after_move.rs
//
// `let s2 = s1;`에서 String의 소유권이 s2로 이동했으므로 s1은 더 이상 유효하지 않습니다.
// s1까지 쓸 수 있다면 두 변수가 같은 힙 메모리를 해제하려는 이중 해제가 생깁니다.

fn main() {
    let s1 = String::from("hello");
    let s2 = s1;

    println!("{s1}, world!");
    println!("{s2}");
}
