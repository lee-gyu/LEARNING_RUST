// 기대하는 에러: E0753
//   error[E0753]: expected outer doc comment
//   help: to annotate the function, change the doc comment from inner to outer style
//
// 확인: rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0753_inner_doc_after_item.rs
//
// `//!`는 그것을 감싸는 것(파일 맨 위라면 크레이트나 모듈)의 문서입니다. 그래서 감싸는 것의
// 내용보다 앞, 즉 파일의 첫 항목보다 앞에만 올 수 있습니다. 크레이트 문서를 `use` 아래에 적는
// 실수가 흔합니다. 컴파일러의 help는 `///`로 바꾸라고 제안하지만, 그러면 크레이트 소개가 바로
// 아래 함수의 문서가 되어 버립니다. 이 경우 맞는 수정은 `//!` 줄들을 파일 맨 위로 옮기는 것입니다.
// (아래 함수에 이미 `///` 문서가 있으면 help가 `//` 주석으로 바꾸라고 제안합니다. 이것도 크레이트
// 문서를 없애 버리는 수정입니다.)

use std::collections::HashMap;

//! 이름과 전화번호를 담는 작은 주소록입니다.

pub fn empty() -> HashMap<String, String> {
    HashMap::new()
}
