// 이 예제가 보여 주는 것: 필드 이름이 없는 튜플 구조체와 필드가 아예 없는 유닛 구조체
//   필드 타입이 같아도 구조체 이름이 다르면 서로 다른 타입이라는 점을 확인합니다.
// 실행: cargo run -p p0-ch5-structs --example 04_tuple_and_unit_structs

// 튜플 구조체(tuple struct): 이름은 붙이되 필드는 타입만 적습니다.
// 둘 다 f64 하나를 감쌌지만 Celsius와 Fahrenheit는 서로 다른 타입입니다.
#[derive(Debug)]
struct Celsius(f64);

#[derive(Debug)]
struct Fahrenheit(f64);

// 필드가 여러 개인 튜플 구조체. 빨강·초록·파랑의 순서가 관례로 정해져 있어 필드 이름이 없어도 읽기 쉽습니다.
#[derive(Debug)]
struct Color(u8, u8, u8);

// 유닛 구조체(unit-like struct): 필드가 없습니다. 중괄호도 괄호도 없이 세미콜론으로 끝납니다.
// 데이터 없이 "이런 종류의 값"이라는 표시만 필요할 때 씁니다. 쓰임새는 트레이트를 배우는 ch10에서 볼 수 있습니다.
#[derive(Debug)]
struct AlwaysEqual;

// 매개변수 타입이 Celsius이므로 Fahrenheit나 맨 f64를 넘기면 컴파일 에러입니다.
// 섭씨와 화씨를 모두 f64로 다뤘다면 이런 실수를 컴파일러가 잡을 수 없었을 것입니다.
fn to_fahrenheit(celsius: Celsius) -> Fahrenheit {
    // 튜플처럼 `.0`, `.1`로 필드에 접근합니다.
    Fahrenheit(celsius.0 * 9.0 / 5.0 + 32.0)
}

fn main() {
    println!("1. 튜플 구조체는 이름(타입)이 붙은 튜플입니다");
    // 만들 때는 함수를 호출하듯 `이름(값, ...)`으로 씁니다.
    let body = Celsius(36.5);
    println!("    body.0 = {}, {body:?}", body.0);
    // 출력: body.0 = 36.5, Celsius(36.5)

    println!("2. 필드 타입이 같아도 구조체 이름이 다르면 다른 타입입니다");
    let converted = to_fahrenheit(body);
    println!("    converted.0 = {}, {converted:?}", converted.0);
    // 출력: converted.0 = 97.7, Fahrenheit(97.7)
    // 아래 줄의 주석을 풀면 E0308: expected `Celsius`, found `Fahrenheit` 에러가 납니다.
    // let twice = to_fahrenheit(converted);

    println!("3. 구조 분해할 때는 구조체 이름까지 적습니다");
    let orange = Color(255, 165, 0);
    // 튜플이라면 `let (r, g, b) = ...`라고 썼겠지만, 튜플 구조체는 `Color(r, g, b)` 모양으로 풉니다.
    let Color(r, g, b) = orange;
    println!("    r = {r}, g = {g}, b = {b}");
    // 출력: r = 255, g = 165, b = 0
    // Color의 필드는 모두 Copy인 u8이라 구조 분해 후에도 orange를 그대로 쓸 수 있습니다.
    println!("    {orange:?}");
    // 출력: Color(255, 165, 0)

    println!("4. 유닛 구조체는 필드 없이 이름만 있습니다");
    let marker = AlwaysEqual; // 만들 때도 이름만 씁니다.
    println!("    {marker:?}");
    // 출력: AlwaysEqual

    println!("5. 이름을 붙여 감싸도 크기는 늘지 않습니다");
    // `size_of::<타입>()`은 그 타입의 값이 차지하는 바이트 수입니다(ch4).
    // Celsius는 f64와 똑같이 8바이트이므로, 타입을 구분하는 데 실행 비용이 들지 않습니다.
    println!("    f64: {}바이트", size_of::<f64>());
    println!("    Celsius: {}바이트", size_of::<Celsius>());
    println!("    Color: {}바이트", size_of::<Color>());
    println!("    AlwaysEqual: {}바이트", size_of::<AlwaysEqual>());
    // 출력:
    //     f64: 8바이트
    //     Celsius: 8바이트
    //     Color: 3바이트
    //     AlwaysEqual: 0바이트
}
