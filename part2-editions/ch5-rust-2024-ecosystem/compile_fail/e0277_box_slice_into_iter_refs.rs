// 기대하는 에러: E0277
// 핵심 메시지: a value of type `Vec<&u32>` cannot be built from an iterator over elements of type `u32`
// 에디션: 2024에서 실패 / 2015·2018·2021에서 통과(boxed_slice_into_iter 경고와 함께, 실행하면 "[90, 85]")
//
// 2021 에디션까지 Box<[T]>에 메서드 호출 문법으로 .into_iter()를 부르면 &[T]의 반복자가 골라져
// &T를 내놓았습니다. 이 파일은 그것을 믿고 참조를 모읍니다. 2024 에디션에서는 같은 호출이
// Box<[T]>의 IntoIterator(값을 내놓음)를 고르므로, &u32를 기대한 Vec<&u32>와 타입이 맞지 않습니다.
// 고치려면 참조가 필요한 곳에 .iter()를 씁니다. 모든 에디션에서 &T를 내놓습니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_box_slice_into_iter_refs.rs --out-dir /tmp/rust-p2-ch5   → 에러
//   rustc --edition 2021 compile_fail/e0277_box_slice_into_iter_refs.rs --out-dir /tmp/rust-p2-ch5   → 통과(경고)

fn main() {
    let scores: Box<[u32]> = vec![70, 85, 90].into_boxed_slice();
    // 높은 점수 두 개를 "가리키는" 목록을 만듭니다. scores는 그대로 두고 싶습니다.
    let mut top: Vec<&u32> = scores.into_iter().collect(); // → 2024 에디션에서 E0277
    top.sort_by(|a, b| b.cmp(a));
    top.truncate(2);
    println!("{:?}", top);
}
