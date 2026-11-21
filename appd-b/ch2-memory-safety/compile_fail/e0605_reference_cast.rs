// 기대하는 에러: E0605
// 핵심 메시지: non-primitive cast: `&u32` as `&f32`
//
// 타입 안전성: 한 타입의 값을 가리키는 참조를 다른 타입의 참조로 바꿔 읽을 수 없습니다.
// C의 `*(float *)&bits`처럼 포인터를 다른 타입의 포인터로 캐스트하면 그 메모리를 새 타입으로
// 읽게 되는데, 그 비트가 새 타입의 올바른 값인지는 아무도 확인하지 않습니다. Rust의 `as`는
// 원시 타입끼리의 변환(숫자 사이 등)에만 쓸 수 있고, 참조의 타입을 바꾸지는 못합니다.
// 원시 포인터를 거치는 캐스트(`&bits as *const u32 as *const f32`)는 컴파일되지만, 그 포인터를
// 따라가 읽는 것은 `unsafe`입니다(E0133, Part 0 ch19).
//
// 고치는 법: 뜻이 정해진 안전한 변환 함수를 씁니다.
//   let value = f32::from_bits(bits);   // 예제 04
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch2-memory-safety compile_fail/e0605_reference_cast.rs

fn main() {
    let bits: u32 = 0x3f80_0000;
    let value: &f32 = &bits as &f32; // E0605
    println!("{value}");
}
