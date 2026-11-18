// 기대하는 에러: error[E0277]: the trait bound `Chars<'_>: ExactSizeIterator` is not satisfied
// 핵심 메시지: the trait `ExactSizeIterator` is not implemented for `Chars<'_>`
//             note: required for `Enumerate<Chars<'_>>` to implement `DoubleEndedIterator`
//             note: required by a bound in `rev`
//             (이어서 E0599: the method `collect` exists for struct `Rev<Enumerate<Chars<'_>>>`, but its trait bounds were not satisfied)
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_rev_needs_exact_size.rs --out-dir /tmp/rust-p1-ch5
//
// chars()는 뒤에서도 꺼낼 수 있으므로(Chars: DoubleEndedIterator) chars().rev()는 됩니다. 그런데 enumerate를 거치면 안 됩니다.
// 문서의 Enumerate 페이지에는 impl<I> DoubleEndedIterator for Enumerate<I> where I: ExactSizeIterator + DoubleEndedIterator 가 있습니다.
// 뒤에서 꺼낸 원소에 붙일 순번(맨 뒤 원소는 몇 번인가)을 알려면 남은 개수를 정확히 알아야 하는데,
// Chars는 UTF-8 바이트를 끝까지 읽기 전에는 글자 수를 모르므로 ExactSizeIterator가 아닙니다.
// 고치는 방법:
//   1) 바이트 위치로 충분하면 char_indices().rev()를 씁니다(CharIndices는 DoubleEndedIterator).
//   2) 글자 순번이 필요하면 먼저 모읍니다: let chars: Vec<char> = word.chars().collect();
//      chars.iter().enumerate().rev() — 슬라이스 반복자는 ExactSizeIterator입니다.
//   3) 순번을 뒤집은 기준으로 세도 되면 rev().enumerate()로 순서를 바꿉니다(examples/07_double_ended.rs).

fn main() {
    let word = "반복자";
    let from_back: Vec<(usize, char)> = word.chars().enumerate().rev().collect();
    println!("{from_back:?}");
}
