// 값을 고르는 패턴들: 리터럴, 이름 있는 변수, 상수, | (여러 패턴 중 하나), 범위.
// 이름 있는 변수가 바깥 변수와 비교되는 것이 아니라 "새 변수"가 된다는 함정도 함께 봅니다.
//
// 실행: cargo run -p p0-ch18-patterns --example 03_literals_ranges

// 패턴 안에 상수 이름을 쓰면 새 변수가 아니라 그 상수의 값과 비교합니다.
const LUCKY: u32 = 7;

// 두 나머지를 튜플로 묶어 리터럴 패턴과 맞춥니다. 갈래 순서가 중요합니다.
// (0, 0)을 맨 위에 두지 않으면 15가 (0, _) 갈래에 먼저 걸려 "Fizz"가 됩니다.
fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => String::from("FizzBuzz"),
        (0, _) => String::from("Fizz"),
        (_, 0) => String::from("Buzz"),
        _ => n.to_string(),
    }
}

// char 범위 패턴과 |를 함께 씁니다. '가'..='힣'은 현대 한글 음절 전체(U+AC00~U+D7A3)입니다.
fn classify(c: char) -> &'static str {
    match c {
        '가'..='힣' => "한글 음절",
        'a'..='z' | 'A'..='Z' => "영문자",
        '0'..='9' => "숫자",
        _ => "기타",
    }
}

// u8의 모든 값(0~255)을 범위 패턴으로 빠짐없이 나눴으므로 _ 갈래가 필요 없습니다.
// 컴파일러는 범위 패턴이 덮는 구간을 계산해 완전성을 검사합니다.
fn digits(n: u8) -> u8 {
    match n {
        0..=9 => 1,
        10..=99 => 2,
        100..=u8::MAX => 3,
    }
}

// 끝을 포함하지 않는 범위(80..90 = 80~89)와 끝이 열린 범위(90.. = 90 이상)도 패턴에 쓸 수 있습니다.
// 이 함수도 u32의 모든 값을 덮으므로 _가 필요 없습니다.
fn grade(score: u32) -> char {
    match score {
        90.. => 'A',
        80..90 => 'B',
        70..80 => 'C',
        0..70 => 'F',
    }
}

fn main() {
    println!("1. 리터럴 패턴: 값이 같으면 맞습니다");
    for cmd in ["start", "stop", "jump"] {
        let action = match cmd {
            "start" => "시작합니다",
            "stop" => "멈춥니다",
            _ => "모르는 명령입니다",
        };
        println!("    {cmd}: {action}");
    }
    let line: Vec<String> = (1..=15).map(fizzbuzz).collect();
    println!("    {}", line.join(" "));
    // 출력:
    //     start: 시작합니다
    //     stop: 멈춥니다
    //     jump: 모르는 명령입니다
    //     1 2 Fizz 4 Buzz Fizz 7 8 Fizz Buzz 11 Fizz 13 14 FizzBuzz

    println!("2. 이름 있는 변수는 무엇이든 맞고, 같은 이름의 바깥 변수를 가립니다");
    let x = Some(5);
    let y = 10;
    match x {
        Some(50) => println!("    50을 받았습니다"),
        // 이 y는 바깥의 y(10)와 비교하는 것이 아니라, Some 안의 값을 묶는 "새 변수 y"입니다.
        // 그래서 Some(5)와 맞고, 갈래 안에서 y는 5입니다(섀도잉).
        Some(y) => println!("    맞았습니다, y = {y}"),
        _ => println!("    기본 갈래, x = {x:?}"),
    }
    println!("    match 뒤: x = {x:?}, y = {y}");
    // 출력:
    //     맞았습니다, y = 5
    //     match 뒤: x = Some(5), y = 10

    println!("3. 상수는 패턴 안에서 값으로 비교됩니다");
    for n in [3, 7] {
        match n {
            LUCKY => println!("    {n}: 행운의 숫자!"),
            _ => println!("    {n}: 보통 숫자"),
        }
    }
    // 출력:
    //     3: 보통 숫자
    //     7: 행운의 숫자!

    println!("4. | 로 여러 패턴 중 하나와 맞추기");
    for day in [Some(1), Some(6), Some(7), Some(9), None] {
        // |는 다른 패턴 안쪽에도 쓸 수 있습니다. Some(6 | 7)은 Some(6) | Some(7)과 같습니다.
        let kind = match day {
            Some(6 | 7) => "주말",
            Some(1..=5) => "평일",
            Some(_) => "없는 요일",
            None => "요일 정보 없음",
        };
        println!("    {day:?}: {kind}");
    }
    // 출력:
    //     Some(1): 평일
    //     Some(6): 주말
    //     Some(7): 주말
    //     Some(9): 없는 요일
    //     None: 요일 정보 없음

    println!("5. ..= 범위 패턴 (정수와 char)");
    for c in ['한', 'Q', '7', '!'] {
        println!("    '{c}': {}", classify(c));
    }
    // 출력:
    //     '한': 한글 음절
    //     'Q': 영문자
    //     '7': 숫자
    //     '!': 기타

    println!("6. 범위로 모든 값을 덮으면 _ 없이도 완전합니다");
    for n in [7, 42, 255] {
        println!("    {n}: {}자리", digits(n));
    }
    // 출력:
    //     7: 1자리
    //     42: 2자리
    //     255: 3자리

    println!("7. 끝을 포함하지 않는 범위(a..b)와 끝이 열린 범위(a..)");
    for score in [95, 90, 89, 72, 30] {
        println!("    {score}점: {}", grade(score));
    }
    // 출력:
    //     95점: A
    //     90점: A
    //     89점: B
    //     72점: C
    //     30점: F
}
