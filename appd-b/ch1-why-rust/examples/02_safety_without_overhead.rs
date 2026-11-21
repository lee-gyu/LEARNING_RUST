// Q1.1 · Q1.8 — 안전장치가 실행 중 메모리를 얼마나 더 쓰는가
//
// "값이 없을 수 있다", "이 수는 미터다" 같은 정보를 타입에 담아도 값의 크기가 그대로인지,
// 반복자 체인 같은 고수준 코드가 힙에 무엇을 만드는지 `size_of`로 확인합니다. 반대로 비용이
// 드는 곳(슬라이스 참조가 들고 다니는 길이)도 함께 봅니다.
//
// 크기는 64비트 플랫폼 기준입니다(포인터 8바이트). 32비트 플랫폼에서는 포인터 크기가 달라집니다.
//
// 실행: cargo run -p faq-ch1-why-rust --example 02_safety_without_overhead

use std::mem::{size_of, size_of_val};

/// 단위를 타입으로 구분하는 뉴타입(Part 0 ch19). 안에는 `f64` 하나뿐입니다.
#[derive(Clone, Copy)]
struct Meters(f64);

/// C에서 흔히 쓰는 모양의 인덱스 반복문. 반복자 체인과 결과를 비교하려고 일부러 이렇게 씁니다.
#[expect(
    clippy::needless_range_loop,
    reason = "반복자 체인과 비교할 C식 인덱스 반복문"
)]
fn sum_by_index(readings: &[i32]) -> i32 {
    let mut total = 0;
    for i in 0..readings.len() {
        if readings[i] % 2 == 1 {
            total += readings[i] * 10;
        }
    }
    total
}

fn compare(plain: &str, plain_size: usize, safe: &str, safe_size: usize) {
    println!("  {plain:<10} {plain_size:>2}바이트  →  {safe:<18} {safe_size:>2}바이트");
}

fn main() {
    println!("[1] 널이 될 수 있음을 타입으로 드러내도 크기는 그대로");
    compare(
        "&u8",
        size_of::<&u8>(),
        "Option<&u8>",
        size_of::<Option<&u8>>(),
    );
    compare(
        "Box<u64>",
        size_of::<Box<u64>>(),
        "Option<Box<u64>>",
        size_of::<Option<Box<u64>>>(),
    );
    // 출력:   &u8         8바이트  →  Option<&u8>         8바이트
    // 출력:   Box<u64>    8바이트  →  Option<Box<u64>>    8바이트
    // 참조와 `Box`는 0번지를 가리킬 수 없으므로, 컴파일러가 0을 `None`의 표시로 씁니다.
    // C의 널 포인터와 같은 크기지만, `Option<&u8>`를 `&u8`처럼 바로 쓸 수는 없고
    // `match`·`if let`·`unwrap`(None이면 패닉)처럼 None을 다루는 길을 거쳐야 합니다.

    println!("[2] 단위를 타입으로 구분해도 크기는 그대로");
    compare("f64", size_of::<f64>(), "Meters(f64)", size_of::<Meters>());
    // 출력:   f64         8바이트  →  Meters(f64)         8바이트
    // `Meters`와 `f64`를 섞어 더하는 실수는 컴파일 에러가 되지만, 실행 중에는 그냥 `f64`입니다.

    println!("[3] 경계 검사에 쓰는 길이는 참조가 함께 들고 다님");
    compare("&u8", size_of::<&u8>(), "&[u8]", size_of::<&[u8]>());
    // 출력:   &u8         8바이트  →  &[u8]              16바이트
    // 슬라이스 참조는 주소와 길이를 함께 가집니다. C에서 포인터와 길이를 따로 넘기던 것을
    // 한 값으로 묶은 것이고, 인덱싱할 때 이 길이로 경계를 검사합니다. 이곳은 비용이 있는 곳입니다.

    println!("[4] 반복자 체인은 힙에 아무것도 만들지 않는 작은 값");
    let readings = [3, 1, 4, 1, 5, 9, 2, 6];
    let chain = readings.iter().filter(|&&x| x % 2 == 1).map(|&x| x * 10);
    println!("  filter + map 체인의 크기: {}바이트", size_of_val(&chain));
    // 출력:   filter + map 체인의 크기: 16바이트
    // 체인은 원래 배열을 가리키는 반복자(주소 두 개)일 뿐이고, 클로저는 캡처한 것이 없어 크기가 0입니다.

    let by_chain: i32 = chain.sum();
    let by_loop = sum_by_index(&readings);
    println!("  체인의 합 {by_chain}, 인덱스 반복문의 합 {by_loop}");
    // 출력:   체인의 합 190, 인덱스 반복문의 합 190

    let distance = Meters(1.5);
    println!("  거리 {}m", distance.0);
    // 출력:   거리 1.5m
}
