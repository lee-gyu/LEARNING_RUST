// 이 예제가 보여 주는 것: 화씨와 섭씨 온도 변환 (The Book 3장 끝 연습 과제의 풀이)
//   이 챕터에서 배운 함수, `f64` 연산, `for`와 배열·범위, `as` 변환을 함께 씁니다.
// 실행: cargo run -p p0-ch3-common-concepts --example 13_temperature

// 섭씨 = (화씨 - 32) × 5 / 9
// 실수 리터럴에는 `32.0`처럼 소수점을 붙입니다. `32`라고 쓰면 정수여서 `f64`와 섞어 계산할 수 없습니다.
fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

// 화씨 = 섭씨 × 9 / 5 + 32
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

fn main() {
    // 서식 `{:>7.1}`: 오른쪽 정렬(`>`), 전체 너비 7칸, 소수점 아래 1자리(`.1`)로 출력합니다.
    // 값의 자릿수가 달라도 열이 가지런히 맞습니다.
    println!("화씨 → 섭씨");
    for fahrenheit in [-40.0, 0.0, 32.0, 98.6, 212.0] {
        let celsius = fahrenheit_to_celsius(fahrenheit);
        println!("{fahrenheit:>7.1}°F = {celsius:>6.1}°C");
    }
    // 출력:
    // 화씨 → 섭씨
    //   -40.0°F =  -40.0°C
    //     0.0°F =  -17.8°C
    //    32.0°F =    0.0°C
    //    98.6°F =   37.0°C
    //   212.0°F =  100.0°C

    // 범위는 정수만 만들 수 있으므로, 정수로 반복한 뒤 `as f64`로 실수로 바꿔 계산합니다.
    println!("섭씨 → 화씨");
    for step in 0..=5 {
        let celsius = (step * 20 - 20) as f64; // -20, 0, 20, 40, 60, 80
        let fahrenheit = celsius_to_fahrenheit(celsius);
        println!("{celsius:>7.1}°C = {fahrenheit:>6.1}°F");
    }
    // 출력:
    // 섭씨 → 화씨
    //   -20.0°C =   -4.0°F
    //     0.0°C =   32.0°F
    //    20.0°C =   68.0°F
    //    40.0°C =  104.0°F
    //    60.0°C =  140.0°F
    //    80.0°C =  176.0°F

    // 두 함수가 서로 역함수인지 왕복 변환으로 확인합니다.
    // 실수 계산에는 아주 작은 오차가 생길 수 있으므로 `==` 대신 차이가 충분히 작은지 비교합니다.
    // `abs()`는 절댓값을 돌려주는 `f64`의 메서드이고, `1e-9`는 0.000000001을 지수로 쓴 실수 리터럴입니다.
    let original: f64 = 36.5;
    let round_trip = fahrenheit_to_celsius(celsius_to_fahrenheit(original));
    let is_same = (original - round_trip).abs() < 1e-9;
    println!("{original}°C → 화씨 → 섭씨 = {round_trip:.1}°C, 같은가? {is_same}");
    // 출력: 36.5°C → 화씨 → 섭씨 = 36.5°C, 같은가? true
}
