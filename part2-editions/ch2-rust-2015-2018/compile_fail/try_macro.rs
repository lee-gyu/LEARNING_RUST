// 기대하는 결과: 2018·2021·2024 에디션에서 컴파일 에러, 2015 에디션에서는 경고(deprecated)와 함께 통과
// 핵심 메시지: use of deprecated `try` macro
//              note: in the 2018 edition `try` is a reserved keyword, and the `try!()` macro is deprecated
//
// try!(식)은 ? 연산자가 생기기 전에 에러를 전파하던 표준 라이브러리 매크로입니다. 2018 에디션부터 try는
// 예약 키워드라서 try!(..)를 매크로 호출로 읽지 않고 에러를 냅니다. 2015 에디션에서도 이 매크로는
// deprecated라서 경고가 납니다.
// 고치는 방법: `let n = text.parse::<i32>()?;`처럼 ?로 바꿉니다. 컴파일러는 r#try!(..)도 함께 제안하는데,
// 이것은 컴파일은 되지만 deprecated 경고가 그대로 남습니다.
//
// 확인(실패): rustc --edition 2024 compile_fail/try_macro.rs --out-dir /tmp/rust-p2-ch2
// 확인(통과): rustc --edition 2015 compile_fail/try_macro.rs --out-dir /tmp/rust-p2-ch2

use std::num::ParseIntError;

fn double(text: &str) -> Result<i32, ParseIntError> {
    let n = try!(text.parse::<i32>()); // → 2018부터 에러
    Ok(n * 2)
}

fn main() {
    println!("{:?}", double("21"));
}
