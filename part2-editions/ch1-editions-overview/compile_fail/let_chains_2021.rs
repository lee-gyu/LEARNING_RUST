// 기대하는 결과: 2015·2018·2021 에디션에서 실패, 2024 에디션에서 통과
// 핵심 메시지: let chains are only allowed in Rust 2024 or later (에러 코드 없음)
//
// let 체인(Part 0 ch18)은 2024 에디션에서만 쓸 수 있는 문법입니다. 새 문법은 대부분 모든 에디션에
// 들어오지만, let 체인은 2024 에디션에서 바뀐 if let의 임시값 범위 규칙을 전제로 하기 때문에
// 2024 에디션 전용입니다. 앞의 세 파일과 달리 이 에러는 메시지에 필요한 에디션을 적어 줍니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2021 compile_fail/let_chains_2021.rs --out-dir /tmp/rust-p2-ch1   → 에러
//   rustc --edition 2024 compile_fail/let_chains_2021.rs --out-dir /tmp/rust-p2-ch1   → 통과

fn main() {
    let input = "42";
    // 다음 줄의 조건 → 2021 에디션까지 에러
    if let Ok(n) = input.parse::<u32>()
        && n > 10
    {
        println!("{} > 10", n);
    }
}
