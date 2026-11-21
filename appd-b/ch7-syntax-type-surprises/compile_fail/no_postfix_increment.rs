// 기대하는 에러: 에러 코드 없음(구문 분석 단계의 에러)
// 핵심 메시지: error: Rust has no postfix increment operator
//
// C·Java·JavaScript처럼 `count++`로 1을 더하려 한 코드입니다. Rust에는 `++`와 `--` 연산자가 없습니다.
// 컴파일러는 이 습관을 알고 있어서 `+= 1`로 바꾸라고 제안합니다.
//
// 고치는 법: count += 1;
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch7-syntax-type-surprises compile_fail/no_postfix_increment.rs

fn main() {
    let words = ["하나", "둘", "셋"];
    let mut count = 0;
    for word in words {
        println!("{word}");
        count++;
    }
    println!("{count}개");
}
