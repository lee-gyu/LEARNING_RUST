// 기대하는 에러: E0119 — conflicting implementations of trait `From<AppError>` for type `AppError`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0119_blanket_from_error.rs --out-dir /tmp/rust-p1-ch8
//
// "Error를 구현한 모든 타입 E를 AppError로 바꿀 수 있다"는 포괄 구현을 만들어 모든 에러에 ?를 쓰려 했습니다.
// 그런데 AppError 자신도 Error를 구현하므로 E = AppError인 경우가 생기고, 그 경우는 표준 라이브러리의
// `impl<T> From<T> for T`(모든 타입은 자기 자신으로 바뀐다)와 겹칩니다.
// 해결: 원인 에러 타입마다 From을 따로 구현합니다. 정말 모든 에러를 받아야 한다면 Box<dyn Error>를 씁니다.
//       (AppError에서 `impl Error`를 빼면 겹치지 않아 컴파일되지만, 그러면 AppError를 에러로 쓸 수 없습니다.)

use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct AppError {
    message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for AppError {}

impl<E: Error> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError {
            message: e.to_string(),
        }
    }
}

fn parse(text: &str) -> Result<i32, AppError> {
    Ok(text.parse::<i32>()?)
}

fn main() {
    println!("{:?}", parse("42"));
}
