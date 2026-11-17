// 기대하는 에러: error[E0277]: `?` couldn't convert the error to `&str`
// 핵심 메시지: the trait `From<std::io::Error>` is not implemented for `&str`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_question_mark_error_type.rs --out-dir /tmp/rust-ch12
//
// Config::build와 모양을 맞추려고 run의 에러 타입도 &'static str로 정한 경우입니다.
// fs::read_to_string의 에러 타입은 std::io::Error인데, ?는 에러를 함수의 에러 타입으로
// 바꿔서(From 트레이트로 변환해서) 돌려줍니다. io::Error를 &str로 바꾸는 방법은 정의돼
// 있지 않으므로 컴파일러가 거부합니다.
// 고치는 방법: run의 반환 타입을 Result<(), Box<dyn Error>>로 바꿉니다.
// Error 트레이트를 구현한 에러는 무엇이든 Box<dyn Error>로 바뀔 수 있습니다.

use std::fs;

struct Config {
    query: String,
    file_path: String,
}

fn run(config: Config) -> Result<(), &'static str> {
    let contents = fs::read_to_string(config.file_path)?;

    println!("검색어: {}", config.query);
    println!("파일 내용:\n{contents}");

    Ok(())
}

fn main() {
    let config = Config {
        query: String::from("the"),
        file_path: String::from("poem.txt"),
    };

    if let Err(e) = run(config) {
        println!("실행 오류: {e}");
    }
}
