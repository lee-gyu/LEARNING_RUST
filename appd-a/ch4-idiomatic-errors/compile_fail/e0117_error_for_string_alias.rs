// 기대하는 에러: E0117 — only traits defined in the current crate can be implemented for types defined outside of the crate
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0117_error_for_string_alias.rs --out-dir /tmp/rust-er-ch4
//
// String 에러를 Error로 만들고 싶어서 별칭을 붙인 뒤 Error를 구현하려 했습니다.
// 하지만 별칭은 이름만 하나 더 붙일 뿐 새 타입을 만들지 않으므로, 이 impl은 `impl Error for String`과 같습니다.
// Error도 String도 이 크레이트의 것이 아니므로 고아 규칙에 걸립니다.
// 해결: struct SkuError(String);처럼 내 크레이트의 타입으로 감싼 뒤(뉴타입) Display와 Error를 구현합니다.
//       examples/01_string_to_newtype.rs의 SkuError가 고친 모양입니다.

use std::error::Error;

type SkuError = String;

impl Error for SkuError {}

fn check_sku(code: &str) -> Result<(), SkuError> {
    if code.contains('-') {
        Ok(())
    } else {
        Err(format!("상품 코드에 '-'가 없습니다: {code:?}"))
    }
}

fn main() {
    if let Err(e) = check_sku("KR102") {
        let boxed: Box<dyn Error> = Box::new(e);
        println!("{boxed}");
    }
}
