// 기대하는 결과: 2024 에디션에서 실패, 2015·2018·2021 에디션에서 통과
// 핵심 메시지: expected identifier, found reserved keyword `gen` (에러 코드 없음)
//
// gen은 2024 에디션에서 예약 키워드가 되었습니다. 아직 gen으로 시작하는 문법이 안정화되지는
// 않았지만, 나중에 gen 블록을 들여올 자리를 미리 잡아 둔 것입니다. 2021 에디션까지는 난수나 ID를
// "생성(generate)"하는 함수에 gen이라는 이름을 흔히 썼고, 그런 코드는 2021 에디션에서는 지금도
// 컴파일됩니다. 2024 에디션에서 같은 이름이 필요하면 r#gen이라고 씁니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/reserved_keyword_gen.rs --out-dir /tmp/rust-p2-ch1   → 에러
//   rustc --edition 2021 compile_fail/reserved_keyword_gen.rs --out-dir /tmp/rust-p2-ch1   → 통과

// 선형 합동 생성기: 이전 값으로 다음 의사 난수를 만듭니다.
// 다음 줄의 함수 이름 → 2024 에디션에서 에러
fn gen(seed: u32) -> u32 {
    seed.wrapping_mul(1_103_515_245).wrapping_add(12_345)
}

fn main() {
    let first = gen(1); // → 2024 에디션에서 에러
    println!("{}", first);
}
