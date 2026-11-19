// 기대하는 결과: 2018·2021·2024 에디션에서 실패, 2015 에디션에서 통과
// 핵심 메시지: expected identifier, found keyword `async` (에러 코드 없음)
// 함께 나오는 에러: `if async {`를 async 블록으로 읽은 E0308 (expected `bool`, found `async` block)
//
// async는 2015 에디션에서는 평범한 식별자였고, 2018 에디션부터 키워드입니다. 2015 에디션 시절에는
// 아래처럼 "비동기로 보낼지"를 async라는 이름에 담는 코드가 문제없이 컴파일되었습니다.
// 키워드를 새로 만들면 이런 코드가 깨지므로, Rust는 이 변경을 2018 에디션을 고른 크레이트에만
// 적용합니다. 에디션을 그대로 두면 이 코드는 지금의 컴파일러에서도 계속 컴파일됩니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/keyword_async_identifier.rs --out-dir /tmp/rust-p2-ch1   → 에러
//   rustc --edition 2018 compile_fail/keyword_async_identifier.rs --out-dir /tmp/rust-p2-ch1   → 에러
//   rustc --edition 2015 compile_fail/keyword_async_identifier.rs --out-dir /tmp/rust-p2-ch1   → 통과

fn send(message: &str) -> String {
    let async = message.len() > 10; // → 2018 에디션부터 에러
    if async {
        format!("[나중에 보냄] {}", message)
    } else {
        format!("[바로 보냄] {}", message)
    }
}

fn main() {
    println!("{}", send("안녕"));
}
