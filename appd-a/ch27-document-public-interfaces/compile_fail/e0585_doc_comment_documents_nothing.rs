// 기대하는 에러: E0585
//   error[E0585]: found a documentation comment that doesn't document anything
//   help: doc comments must come before what they document, if a comment was intended use `//`
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0585_doc_comment_documents_nothing.rs
//
// `///`는 "바로 뒤에 오는 항목의 문서"라는 뜻의 문법이지 주석의 다른 모양이 아닙니다. 블록의
// 끝처럼 뒤에 아무것도 없는 곳에 쓰면 붙을 대상이 없어 컴파일 에러가 납니다. 함수 본문의 `let`
// 문장 앞처럼 대상은 있지만 rustdoc이 문서를 만들지 않는 곳이면 에러 대신 `unused_doc_comments`
// 경고가 납니다. 둘 다 고치는 방법은 같습니다. 코드를 읽는 사람을 위한 설명은 `//`로 씁니다.
// (같은 `///`를 블록의 마지막 표현식 바로 뒤에 쓰면 E0585가 아니라 에러 코드 없는 구문 에러
// `expected one of ... found doc comment`가 납니다. 이 파일은 `;`로 끝난 문장 뒤에 둡니다.)

fn print_total(prices: &[u64]) {
    let sum: u64 = prices.iter().sum();
    println!("합계: {sum}원");
    /// 할인은 여기서 적용하지 않습니다. 할인은 결제 단계의 일입니다.
}

fn main() {
    print_total(&[1200, 800]);
}
