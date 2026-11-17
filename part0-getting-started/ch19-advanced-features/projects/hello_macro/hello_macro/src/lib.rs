//! `HelloMacro` 트레이트를 정의하는 라이브러리 크레이트입니다.
//!
//! 이 트레이트는 타입마다 직접 구현할 수도 있지만, 구현이 "타입 이름을 출력한다"는
//! 같은 모양이라서 `hello_macro_derive` 크레이트의 `#[derive(HelloMacro)]`가 대신 작성해 줍니다.

/// 자기 타입의 이름으로 인사하는 트레이트입니다.
pub trait HelloMacro {
    fn hello_macro();
}
