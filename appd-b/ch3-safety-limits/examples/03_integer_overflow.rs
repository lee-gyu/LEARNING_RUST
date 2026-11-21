// Q3.6 — 릴리스 빌드에서 래핑된 길이가 어디까지 가는가
//
// Part 0 ch3 예제 04에서 본 오버플로 규칙(디버그는 패닉, 릴리스는 래핑, 뜻을 밝힌 메서드 4종)을
// 메모리 안전성의 눈으로 다시 봅니다. 패킷의 길이를 헤더 200바이트 + 본문 100바이트로 계산하는데,
// 길이 필드가 `u8`(최대 255)이라 300을 담을 수 없습니다.
//
//   1. 권장: `checked_add`로 계산하고, 넘치면 요청을 거부
//   2. 비권장: 그냥 `+`로 계산하고, 그 길이로 만든 버퍼에 300바이트를 담으려 함
//
// 2번은 빌드에 따라 다르게 끝납니다. 디버그 빌드는 덧셈에서 패닉하고(종료 코드 101), 릴리스
// 빌드는 래핑된 길이 44로 버퍼를 만든 뒤 담을 자리가 없어 거부합니다(종료 코드 0). 어느 쪽도
// 버퍼 밖에 쓰지 않습니다. C였다면 44바이트 버퍼에 300바이트를 복사해 힙을 망가뜨렸을 자리입니다.
//
// 값은 `black_box`로 감쌌습니다. 컴파일러가 값을 미리 알면 `arithmetic_overflow` 린트가 빌드를
// 멈추기 때문입니다(Part 0 ch3 4절).
//
// 실행(디버그 빌드, 일부러 패닉하며 종료 코드 101):
//   cargo run -p faq-ch3-safety-limits --example 03_integer_overflow
// 실행(릴리스 빌드, 종료 코드 0):
//   cargo run -p faq-ch3-safety-limits --example 03_integer_overflow --release
// 실행(릴리스 빌드에서도 오버플로 검사를 켬, 종료 코드 101):
//   CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true cargo run -p faq-ch3-safety-limits --example 03_integer_overflow --release

use std::hint::black_box;

fn main() {
    let header: u8 = black_box(200);
    let body: u8 = black_box(100);
    let payload = vec![1u8; 300]; // 실제로 담아야 할 데이터

    println!("1. 권장: checked_add로 길이 계산");
    match header.checked_add(body) {
        Some(len) => println!("  길이 {len}"),
        None => println!("  길이가 u8을 넘으므로 요청 거부"),
    }
    // 출력:   길이가 u8을 넘으므로 요청 거부

    println!("2. 비권장: 그냥 +로 길이 계산");
    let len = header + body; // 디버그 빌드는 여기서 패닉
    println!("  계산한 길이 {len}");
    // 출력(릴리스 빌드):   계산한 길이 44
    let mut buffer = vec![0u8; usize::from(len)];
    match buffer.get_mut(..payload.len()) {
        Some(slot) => {
            slot.copy_from_slice(&payload);
            println!("  {}바이트를 담음", payload.len());
        }
        None => println!(
            "  {}바이트 버퍼에 {}바이트를 담을 자리가 없어 거부",
            buffer.len(),
            payload.len()
        ),
    }
    // 출력(릴리스 빌드):   44바이트 버퍼에 300바이트를 담을 자리가 없어 거부
    // `get_mut`이 범위를 검사합니다. `buffer.copy_from_slice(&payload)`처럼 길이가 다른 슬라이스를
    // 그대로 복사하려 해도 패닉할 뿐 버퍼 밖에 쓰지는 않습니다. 래핑은 메모리 오류가 아니라
    // "틀린 길이"라는 논리 오류로 남습니다.
}
