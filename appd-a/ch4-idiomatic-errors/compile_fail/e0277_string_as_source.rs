// 기대하는 에러: E0277 — the trait bound `String: std::error::Error` is not satisfied
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_string_as_source.rs --out-dir /tmp/rust-er-ch4
//
// 아래 계층(check_sku)이 실패를 String으로 알립니다. 위 계층의 LineError는 그 String을 원인으로 품고
// source()로 내주려 했습니다. 그런데 source()가 돌려줄 수 있는 것은 &(dyn Error + 'static)뿐이고,
// String은 Error를 구현하지 않으므로 &String을 &dyn Error로 바꿀 수 없습니다.
// 해결: 필드 타입을 Error를 구현한 뉴타입(SkuError)으로 바꿉니다. 아래 계층의 함수를 고칠 수 없더라도
//       받은 String을 경계에서 SkuError로 감싸면 됩니다.
//       examples/01_string_to_newtype.rs의 LineError와 check_sku가 고친 모양입니다.
//       String을 그대로 품겠다면 source()는 포기하고 메시지를 Display에 이어 붙입니다(같은 예제의 LineErrorFlat).

use std::error::Error;
use std::fmt;

fn check_sku(code: &str) -> Result<(), String> {
    if code.contains('-') {
        Ok(())
    } else {
        Err(format!("상품 코드에 '-'가 없습니다: {code:?}"))
    }
}

#[derive(Debug)]
struct LineError {
    line_no: usize,
    cause: String,
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}번째 줄을 처리하지 못했습니다", self.line_no)
    }
}

impl Error for LineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}

fn main() {
    let result = check_sku("KR102").map_err(|cause| LineError { line_no: 2, cause });
    if let Err(e) = result {
        println!("{e}");
    }
}
