// 이 예제가 보여 주는 것: 함수 정의, 매개변수(parameter), 반환값, 조기 반환(`return`)
// 실행: cargo run -p p0-ch3-common-concepts --example 07_functions

fn main() {
    // 함수는 호출하는 곳보다 아래에 정의돼 있어도 됩니다.
    // Rust는 정의 순서가 아니라 "호출하는 곳에서 보이는 범위에 정의돼 있는가"만 봅니다.
    greet();

    // 매개변수에는 반드시 타입을 적습니다. 호출할 때 넘기는 구체적인 값은 인자(argument)라고 부릅니다.
    print_labeled_measurement(5, 'h'); // 출력: 측정값: 5h

    // 반환값이 있는 함수는 그 결과를 변수에 담거나 다른 식 안에서 바로 쓸 수 있습니다.
    let x = plus_one(5);
    println!("plus_one(5) = {x}"); // 출력: plus_one(5) = 6
    println!("plus_one(plus_one(5)) = {}", plus_one(plus_one(5))); // 출력: plus_one(plus_one(5)) = 7

    println!("넓이 = {}", rectangle_area(3, 4)); // 출력: 넓이 = 12

    // 튜플을 반환하면 값 여러 개를 한 번에 돌려줄 수 있고, 받는 쪽에서 구조 분해합니다.
    let (quotient, remainder) = divide(17, 5);
    println!("17 / 5 = 몫 {quotient}, 나머지 {remainder}"); // 출력: 17 / 5 = 몫 3, 나머지 2

    println!("distance(3, 10) = {}", distance(3, 10)); // 출력: distance(3, 10) = 7
    println!("distance(10, 3) = {}", distance(10, 3)); // 출력: distance(10, 3) = 7
}

// 함수와 변수 이름은 snake_case(소문자 + 밑줄)로 짓는 것이 관례입니다.
// 반환 타입을 적지 않은 함수는 유닛 `()`를 반환합니다.
fn greet() {
    println!("안녕하세요!"); // 출력: 안녕하세요!
}

// 매개변수가 여러 개면 쉼표로 구분하고, 각각 타입을 적습니다.
// 시그니처에 타입을 강제하는 덕분에 컴파일러는 함수 본문과 호출하는 쪽을 따로따로 검사할 수 있고,
// 다른 곳에서는 타입 표기를 거의 생략해도 됩니다.
fn print_labeled_measurement(value: i32, unit_label: char) {
    println!("측정값: {value}{unit_label}");
}

// 반환 타입은 `->` 뒤에 적습니다. 반환값에는 이름을 붙이지 않습니다.
// 함수 본문의 마지막 표현식(세미콜론이 없는 줄)의 값이 곧 반환값입니다.
fn plus_one(x: i32) -> i32 {
    x + 1 // 여기에 세미콜론을 붙이면 컴파일 에러가 납니다 (compile_fail/e0308_tail_expression_semicolon.rs).
}

fn rectangle_area(width: u32, height: u32) -> u32 {
    width * height
}

fn divide(dividend: i32, divisor: i32) -> (i32, i32) {
    (dividend / divisor, dividend % divisor)
}

// `return`은 함수 중간에서 바로 값을 돌려주고 끝낼 때 씁니다(조기 반환).
// 함수 끝에서는 `return`을 쓰지 않고 마지막 표현식으로 반환하는 것이 Rust의 관례입니다.
// (`if`는 09_if 예제에서 자세히 다룹니다.)
fn distance(a: i32, b: i32) -> i32 {
    if a > b {
        return a - b;
    }
    b - a
}
