// 기대하는 에러: error[E0277]: the `?` operator can only be used on `Option`s, not `Result`s,
//               in a function that returns `Option`
// 핵심 메시지: use `.ok()?` if you want to discard the `Result<!, _>` error information
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_result_question_in_option_fn.rs --out-dir /tmp/rust-er-ch3
//
// 이 함수는 Option을 돌려주므로, 안에서 ?가 함수를 끝낼 때 돌려줄 수 있는 것은 None뿐입니다.
// 그런데 parse()는 Result를 돌려주고, Err 안의 ParseIntError를 None으로 바꾸는 일은 ?가 해 주지 않습니다.
// ?는 에러 타입 사이의 변환(From)은 해 주지만, Result와 Option 사이를 건너가지는 않습니다.
// 고치는 방법은 두 가지이고, 무엇을 고를지는 실패 이유가 필요한지에 달렸습니다(README 2.2절).
//   1. 이유가 필요 없으면 ok()로 Result를 Option으로 바꾼 뒤 ?:  let n: u32 = text.trim().parse().ok()?;
//   2. 이유를 호출한 쪽에 알려야 하면 함수의 반환 타입을 Result로 바꿉니다.

fn wished_count(form: &str) -> Option<u32> {
    let (_, text) = form.split_once("수량=")?;
    let n: u32 = text.trim().parse()?;
    Some(n)
}

fn main() {
    println!("{:?}", wished_count("이름=가온, 수량=3"));
}
