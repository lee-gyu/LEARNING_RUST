// Q2.7 — 메모리 버그로 이어지는 정수 실수를 드러내기
//
// C에서는 버퍼 크기를 계산하다 생긴 정수 실수가 메모리 버그로 이어지곤 합니다. `count * size`가
// 오버플로로 작은 값이 되면 작은 버퍼를 할당하고 큰 데이터를 쓰게 되고, `-1`이 `size_t`로 바뀌면 아주
// 큰 길이가 됩니다. Rust는 이런 변환을 조용히 하지 않고, 실패할 수 있는 계산은 `Option`이나
// `Result`로 돌려줍니다. 비트를 다른 타입으로 읽는 것도 이름 있는 안전한 함수로 합니다.
//
// 실행: cargo run -p faq-ch2-memory-safety --example 04_size_and_conversion

/// 원소 `count`개, 원소 하나에 `size`바이트인 버퍼의 전체 크기. 오버플로가 나면 `None`입니다.
fn buffer_size(count: usize, size: usize) -> Option<usize> {
    count.checked_mul(size)
}

fn main() {
    println!("1. 크기 곱셈의 오버플로");
    println!("  1,000개 × 16바이트 = {:?}", buffer_size(1_000, 16));
    // 출력:   1,000개 × 16바이트 = Some(16000)
    println!(
        "  (usize::MAX / 8 + 1)개 × 16바이트 = {:?}",
        buffer_size(usize::MAX / 8 + 1, 16)
    );
    // 출력:   (usize::MAX / 8 + 1)개 × 16바이트 = None
    // C의 부호 없는 곱셈은 오버플로가 나면 조용히 작은 값으로 돌아갑니다(래핑). `checked_mul`은
    // 오버플로를 `None`으로 알립니다. 그냥 `*`를 쓰면 디버그 빌드는 패닉, 릴리스 빌드는 래핑입니다(ch3).

    println!("2. 음수 길이");
    let from_header: i64 = -1; // 네트워크에서 읽은 값이라고 해 봅니다
    println!("  usize::try_from(-1) = {:?}", usize::try_from(from_header));
    // 출력:   usize::try_from(-1) = Err(TryFromIntError(NegOverflow))
    // 부호 있는 수를 길이(`usize`)로 바꾸는 변환은 자동으로 일어나지 않고, `try_from`은 음수를 거부합니다.
    // `TryFrom`과 바이트 변환은 Part 1 ch1에서 다룹니다.

    println!("3. 잘라 내는 변환은 코드에 보이게");
    let wide: i32 = 300;
    let narrow = wide as u8;
    println!("  300_i32 as u8 = {narrow}");
    // 출력:   300_i32 as u8 = 44
    println!("  u8::try_from(300) = {:?}", u8::try_from(wide));
    // 출력:   u8::try_from(300) = Err(TryFromIntError(PosOverflow))
    // `as`는 범위를 넘는 비트를 잘라 냅니다(300 = 256 + 44). 다만 그렇게 하겠다는 것이 코드에 적혀 있고,
    // 값을 지키고 싶으면 `try_from`을 씁니다. C처럼 대입만으로 일어나는 변환은 없습니다.

    println!("4. 비트를 다른 타입으로 읽기");
    let bits: u32 = 0x3f80_0000;
    println!("  f32::from_bits(0x3f80_0000) = {}", f32::from_bits(bits));
    // 출력:   f32::from_bits(0x3f80_0000) = 1
    println!(
        "  u32::from_le_bytes([1, 0, 0, 0]) = {}",
        u32::from_le_bytes([1, 0, 0, 0])
    );
    // 출력:   u32::from_le_bytes([1, 0, 0, 0]) = 1
    // C는 포인터를 다른 타입의 포인터로 바꿔 읽지만, Rust의 안전한 코드에서는 이렇게 뜻이 정해진
    // 함수를 씁니다. 참조를 다른 타입으로 캐스트하는 코드는 컴파일되지 않습니다(README 4.2절).
}
