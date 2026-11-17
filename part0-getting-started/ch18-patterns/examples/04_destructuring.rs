// 구조 분해: 값을 만들 때의 모양 그대로 패턴을 써서 값을 부분으로 나눕니다.
// 구조체, 열거형, 중첩된 열거형, 튜플과 구조체를 섞은 값을 차례로 풀어 봅니다.
//
// 실행: cargo run -p p0-ch18-patterns --example 04_destructuring

struct Point {
    x: i32,
    y: i32,
}

enum Color {
    Rgb(u8, u8, u8),
    Hsv(u16, u8, u8),
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(Color), // 배리언트 안에 다른 열거형이 들어 있습니다.
}

// 구조체 패턴에 리터럴을 섞으면 "이 필드는 이 값이어야 한다"는 조건이 됩니다.
// 원점은 두 축 위에 모두 있지만 위에서부터 맞춰 보므로 첫 갈래에서 끝납니다.
fn locate(p: Point) -> String {
    match p {
        Point { x: 0, y: 0 } => String::from("원점"),
        Point { x, y: 0 } => format!("x축 위 (x = {x})"),
        Point { x: 0, y } => format!("y축 위 (y = {y})"),
        Point { x, y } => format!("어느 축에도 없음 ({x}, {y})"),
    }
}

// 배리언트의 모양(없음, 구조체형, 튜플형)대로 패턴을 쓰고,
// 중첩된 열거형은 패턴도 중첩해서 한 번에 안쪽까지 풉니다.
fn handle(msg: Message) -> String {
    match msg {
        Message::Quit => String::from("종료"),
        Message::Move { x, y } => format!("이동: ({x}, {y})"),
        Message::Write(text) => format!("글자 쓰기: {text}"),
        Message::ChangeColor(Color::Rgb(r, g, b)) => format!("색 변경: RGB({r}, {g}, {b})"),
        Message::ChangeColor(Color::Hsv(h, s, v)) => format!("색 변경: HSV({h}, {s}, {v})"),
    }
}

fn main() {
    println!("1. 구조체 구조 분해: 필드 이름으로 꺼냅니다");
    let p = Point { x: 3, y: -7 };
    // 필드 x의 값을 변수 a에, 필드 y의 값을 변수 b에 묶습니다. "필드: 변수" 순서입니다.
    let Point { x: a, y: b } = p;
    // 변수 이름을 필드 이름과 같게 하려면 줄여 씁니다. 필드 초기화 축약(ch5)의 반대 방향입니다.
    let Point { x, y } = p;
    println!("    a = {a}, b = {b} / x = {x}, y = {y}");
    // i32 필드는 복사되므로 p는 그대로 남아 있습니다.
    println!("    p는 그대로: ({}, {})", p.x, p.y);
    // 출력:
    //     a = 3, b = -7 / x = 3, y = -7
    //     p는 그대로: (3, -7)

    println!("2. 구조체 패턴에 리터럴 섞기");
    for p in [
        Point { x: 0, y: 0 },
        Point { x: 5, y: 0 },
        Point { x: 0, y: -2 },
        Point { x: 1, y: 1 },
    ] {
        println!("    {}", locate(p));
    }
    // 출력:
    //     원점
    //     x축 위 (x = 5)
    //     y축 위 (y = -2)
    //     어느 축에도 없음 (1, 1)

    println!("3. 열거형과 중첩된 열거형 구조 분해");
    let messages = vec![
        Message::Move { x: 10, y: 20 },
        Message::Write(String::from("안녕")),
        Message::ChangeColor(Color::Rgb(255, 128, 0)),
        Message::ChangeColor(Color::Hsv(200, 50, 80)),
        Message::Quit,
    ];
    for msg in messages {
        println!("    {}", handle(msg));
    }
    // 출력:
    //     이동: (10, 20)
    //     글자 쓰기: 안녕
    //     색 변경: RGB(255, 128, 0)
    //     색 변경: HSV(200, 50, 80)
    //     종료

    println!("4. 튜플과 구조체를 섞은 값도 한 번에 풉니다");
    let ((feet, inches), Point { x, y }) = ((5, 11), Point { x: 3, y: -10 });
    println!("    키 {feet}피트 {inches}인치, 위치 ({x}, {y})");
    // 출력:
    //     키 5피트 11인치, 위치 (3, -10)
}
