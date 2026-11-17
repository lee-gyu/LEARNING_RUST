// 기대하는 에러: E0382 — borrow of partially moved value: `name`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0382_use_after_match_move.rs
//
// Some(n) 패턴이 Option<String> 안의 String을 n으로 옮겼습니다(이동).
// name의 일부(안에 든 String)가 빠져나갔으므로 match 뒤에 name을 통째로 쓸 수 없습니다.

fn main() {
    let name: Option<String> = Some(String::from("러스트"));

    match name {
        Some(n) => println!("이름: {n}"),
        None => println!("이름 없음"),
    }

    println!("{name:?}");
}
