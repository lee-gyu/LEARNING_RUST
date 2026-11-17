// 이 예제가 보여 주는 것: 불변 변수(`let`), 가변 변수(`let mut`), 상수(`const`)의 차이
// 실행: cargo run -p p0-ch3-common-concepts --example 01_variables

// 상수는 함수 밖(전역 스코프)에도 선언할 수 있어서, 프로그램 여러 곳에서 쓰는 값을 한곳에 모아 둘 때 씁니다.
// 상수에는 타입 표기가 반드시 필요합니다. 컴파일러가 추론해 주지 않습니다.
const SECONDS_PER_MINUTE: u32 = 60;

// 상수의 값은 컴파일할 때 계산됩니다. 그래서 `10800`이라고 쓰는 대신
// 계산식을 그대로 적어 의미를 드러낼 수 있고, 실행 시간에는 비용이 들지 않습니다.
const THREE_HOURS_IN_SECONDS: u32 = 60 * SECONDS_PER_MINUTE * 3;

fn main() {
    // 1. 불변 변수: `let`으로 만든 변수는 기본적으로 값을 바꿀 수 없습니다.
    //    "이 값은 바뀌지 않는다"는 약속을 컴파일러가 지켜 주므로,
    //    코드를 읽을 때 이 값이 어디선가 몰래 바뀌었을까 걱정할 필요가 없습니다.
    let x = 5;
    println!("x = {x}"); // 출력: x = 5
    // x = 6; // 주석을 풀면 E0384 컴파일 에러가 납니다 (compile_fail/e0384_assign_twice_to_immutable.rs).

    // 2. 가변 변수: `mut`를 붙이면 값을 바꿀 수 있습니다.
    //    `mut`는 읽는 사람에게 "이 변수는 뒤에서 바뀐다"고 알리는 표시이기도 합니다.
    let mut count = 0;
    println!("count = {count}"); // 출력: count = 0
    count = 1; // 새 값을 대입합니다.
    count += 1; // `count = count + 1`과 같은 뜻입니다.
    println!("count = {count}"); // 출력: count = 2

    // 3. 상수: 이름은 관례상 대문자와 밑줄(UPPER_SNAKE_CASE)로 짓습니다.
    println!("3시간 = {THREE_HOURS_IN_SECONDS}초"); // 출력: 3시간 = 10800초

    // 상수는 함수 안 같은 좁은 스코프에도 선언할 수 있습니다.
    // 이 상수는 `main` 함수 안에서만 보입니다.
    const MAX_POINTS: u32 = 100_000; // 숫자 사이의 `_`는 읽기 쉽게 끊어 쓰는 구분자입니다.
    println!("최대 점수 = {MAX_POINTS}"); // 출력: 최대 점수 = 100000

    // 4. 불변 변수와 상수의 차이: 불변 변수의 값은 실행 중에 정해져도 됩니다.
    //    아래 `snapshot`은 실행 중에 바뀐 `count`의 그 순간 값을 담습니다.
    //    상수는 컴파일할 때 값이 확정돼야 하므로 같은 일을 할 수 없습니다.
    //    `const SNAPSHOT: i32 = count;`라고 쓰면 E0435
    //    (attempt to use a non-constant value in a constant) 에러가 납니다.
    let snapshot = count;
    count += 10;
    println!("snapshot = {snapshot}, count = {count}"); // 출력: snapshot = 2, count = 12
}
