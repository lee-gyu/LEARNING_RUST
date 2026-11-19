// 기대하는 결과: 2018·2021·2024 에디션에서 컴파일 에러, 2015 에디션에서는 경고 없이 통과
// 핵심 메시지: expected identifier, found reserved keyword `try`
//              expected identifier, found keyword `dyn`
//
// 2015 에디션에서 try는 키워드가 아니었고, dyn은 타입 자리에서만 키워드로 읽히는 약한 키워드였습니다.
// 그래서 함수 이름 try, 변수 이름 dyn을 쓸 수 있었습니다. 2018 에디션에서 dyn은 엄격한 키워드,
// try는 예약 키워드가 되면서 같은 코드가 파싱 단계에서 거절됩니다.
// (async를 이름으로 쓴 경우는 Part 2 ch1-editions-overview의 compile_fail/keyword_async_identifier.rs에 있습니다.)
// 고치는 방법: 컴파일러가 제안하는 대로 r#try, r#dyn처럼 raw 식별자로 쓰거나, 내 코드라면 다른 이름을 고릅니다.
//
// 확인(실패): rustc --edition 2024 compile_fail/keyword_dyn_try_identifiers.rs --out-dir /tmp/rust-p2-ch2
// 확인(통과): rustc --edition 2015 compile_fail/keyword_dyn_try_identifiers.rs --out-dir /tmp/rust-p2-ch2

fn try(text: &str) -> Option<u32> {
    // → 2018부터 에러: try는 예약 키워드
    text.parse().ok()
}

fn main() {
    let dyn = 3; // → 2018부터 에러: dyn은 키워드
    let total = try("7").unwrap_or(0) + dyn;
    println!("{}", total);
}
