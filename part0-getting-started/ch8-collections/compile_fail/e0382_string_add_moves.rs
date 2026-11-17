// 기대하는 에러: E0382 — borrow of moved value: `s1`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0382_string_add_moves.rs
//
// + 연산자는 왼쪽 String을 self로 받아(소유권 이동) 그 버퍼에 오른쪽 &str을 덧붙인 뒤 돌려줍니다.
// 그래서 s1 + &s2 이후에는 s1을 쓸 수 없습니다. 오른쪽의 s2는 빌려주기만 했으므로 계속 쓸 수 있습니다.

fn main() {
    let s1 = String::from("안녕, ");
    let s2 = String::from("세상!");

    let s3 = s1 + &s2;

    println!("{s3}");
    println!("{s1}");
}
