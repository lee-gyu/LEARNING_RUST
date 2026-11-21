// Q2.1 · Q2.4 — 공간적 안전성: 객체 밖의 메모리에 닿는 길이 없다
//
// 배열·슬라이스·문자열에서 범위를 벗어날 수 있는 접근을 해 보고, Rust가 각각을 어떻게
// 처리하는지 봅니다. `get`처럼 `Option`을 돌려주는 메서드는 범위를 벗어나면 `None`을,
// 인덱싱(`[]`)은 패닉을 냅니다(패닉은 03에서 봅니다). 어느 쪽이든 옆 메모리를 읽지 않습니다.
//
// 실행: cargo run -p faq-ch2-memory-safety --example 01_spatial_checks

fn main() {
    let scores = [72, 85, 91, 64, 88];

    println!("1. 인덱스 하나");
    println!("  scores.get(2)  = {:?}", scores.get(2));
    // 출력:   scores.get(2)  = Some(91)
    println!("  scores.get(5)  = {:?}", scores.get(5));
    // 출력:   scores.get(5)  = None
    // 배열의 길이는 5이므로 인덱스 5는 배열 밖입니다. C라면 배열 바로 뒤의 메모리를 읽을 수 있습니다(미정의 동작).

    println!("2. 범위");
    println!("  scores.get(1..3)  = {:?}", scores.get(1..3));
    // 출력:   scores.get(1..3)  = Some([85, 91])
    println!("  scores.get(3..10) = {:?}", scores.get(3..10));
    // 출력:   scores.get(3..10) = None
    // 끝이 길이를 넘는 범위는 앞부분만 잘라 주지도 않고 통째로 거부합니다.

    println!("3. 문자열: 바이트 범위가 글자 경계에 맞아야 함");
    let name = "한글";
    println!(
        "  바이트 길이 {}, 글자 수 {}",
        name.len(),
        name.chars().count()
    );
    // 출력:   바이트 길이 6, 글자 수 2
    println!("  name.get(0..3) = {:?}", name.get(0..3));
    // 출력:   name.get(0..3) = Some("한")
    println!("  name.get(0..1) = {:?}", name.get(0..1));
    // 출력:   name.get(0..1) = None
    // "한"은 UTF-8로 3바이트입니다. 0..1은 글자 중간에서 끊기므로, 잘린 바이트가 `&str`이 되어
    // 다른 코드가 올바른 UTF-8이라고 믿고 읽는 일이 생기지 않게 거부합니다.

    println!("4. 인덱스 없이 훑기");
    let total: i32 = scores.iter().sum();
    let best = scores.iter().max();
    println!("  합계 {total}, 최고점 {best:?}");
    // 출력:   합계 400, 최고점 Some(91)
    // 반복자는 처음부터 끝까지만 훑으므로 범위를 벗어날 인덱스가 아예 없습니다.
}
