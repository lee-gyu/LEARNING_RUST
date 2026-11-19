// 에디션별 결과: 2024에서 실패, 2021에서 통과
// 기대하는 에러(2024): E0716 — temporary value dropped while borrowed
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0716_tail_expr_narrowed.rs --out-dir /tmp/rust-p2-ch4   # 실패
//   rustc --edition 2021 compile_fail/e0716_tail_expr_narrowed.rs --out-dir /tmp/rust-p2-ch4   # 통과
//
// 블록 `{ &String::from("1234") }`의 마지막 표현식이 임시 `String`을 만들고 그 참조를
// 블록의 값으로 내보냅니다. 2021 에디션에서는 이 임시값이 바깥 문장 끝까지 살아 있어서
// `.len()`이 그것을 읽을 수 있었습니다. 2024 에디션에서는 블록이 끝나는 순간 버려지므로,
// 블록 밖의 `.len()`이 이미 버려진 값을 빌리게 됩니다.
// 해결: 블록의 결과를 `let`으로 묶습니다. `let text = { &String::from("1234") };` 다음
// `let len = text.len();` 처럼 쓰면 임시값 수명 연장 규칙 덕분에 `String`이 `text`만큼 삽니다.

fn main() {
    let len = { &String::from("1234") }.len(); // → 2024에서 E0716
    println!("{len}");
}
