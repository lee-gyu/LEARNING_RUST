// 패턴이 쓰이는 일곱 자리를 차례로 보여 줍니다.
// match 갈래만 패턴 자리인 것이 아닙니다. let, if let, while let, for, 함수·클로저 매개변수,
// let...else의 왼쪽도 모두 패턴입니다. 지금까지 써 온 `let x = 5;`의 x도
// "무엇이든 x에 묶어라"라는 가장 단순한 패턴이었습니다.
//
// 실행: cargo run -p p0-ch18-patterns --example 01_pattern_places

// 함수 매개변수도 패턴입니다. 튜플의 참조를 받아 &(x, y) 패턴으로 그 자리에서 풉니다.
// &는 "참조를 벗기고 안의 값을 꺼내라"는 패턴입니다(05_binding_modes.rs).
fn print_coordinates(&(x, y): &(i32, i32)) {
    println!("    현재 위치: ({x}, {y})");
}

// if let, else if, else if let, else를 섞어 쓴 사슬입니다.
// 각 조건이 서로 다른 값을 검사해도 됩니다. match는 값 하나만 검사하므로 이런 모양을 만들 수 없습니다.
fn discount(coupon: Option<&str>, is_member_day: bool, age_text: &str) -> String {
    if let Some(code) = coupon {
        format!("쿠폰 {code} 할인")
    } else if is_member_day {
        String::from("회원의 날 10% 할인")
    } else if let Ok(age) = age_text.parse::<u32>() {
        // 새로 묶인 age는 이 블록 안에서만 쓸 수 있으므로 나이 비교를 한 단계 안쪽에 둡니다.
        if age >= 65 {
            String::from("경로 우대 할인")
        } else {
            String::from("할인 없음")
        }
    } else {
        String::from("나이를 확인할 수 없음")
    }
}

// Edition 2024의 let 체인: if 조건에서 let 패턴과 bool 조건을 &&로 잇습니다.
// 앞의 let이 묶은 age를 && 뒤의 조건에서 바로 쓸 수 있어서, 위 함수의 중첩 if가 사라집니다.
// 사슬의 어느 한 고리라도 실패하면(숫자가 아니거나, 65 미만이면) else로 갑니다.
fn senior_discount(age_text: &str) -> &'static str {
    if let Ok(age) = age_text.parse::<u32>()
        && age >= 65
    {
        "경로 우대 할인"
    } else {
        "할인 없음"
    }
}

// let...else: 패턴이 맞으면 바인딩이 아래 스코프 전체에 남고, 맞지 않으면 else에서 흐름을 끊습니다.
fn greet(line: &str) {
    let Some((name, age)) = line.split_once(':') else {
        println!("    형식이 잘못됐습니다: {line}");
        return;
    };
    println!("    {name}님은 {age}살입니다");
}

fn main() {
    println!("1. match 갈래: => 왼쪽이 패턴입니다");
    for dice in [Some(3), None] {
        let message = match dice {
            Some(n) => format!("{n}칸 이동"),
            None => String::from("아직 던지지 않음"),
        };
        println!("    {dice:?} → {message}");
    }
    // 출력:
    //     Some(3) → 3칸 이동
    //     None → 아직 던지지 않음

    println!("2. let: 등호 왼쪽도 패턴입니다");
    let x = 5; // 변수 이름 하나뿐인 패턴: 어떤 값이든 x에 묶습니다.
    let (a, b, c) = (1, 2, 3); // 튜플 패턴: 원소 개수가 같아야 맞습니다.
    println!("    x = {x}, a = {a}, b = {b}, c = {c}");
    // 출력:
    //     x = 5, a = 1, b = 2, c = 3

    println!("3. if let / else if / else if let / else 사슬");
    println!("    {}", discount(Some("WELCOME"), true, "70"));
    println!("    {}", discount(None, true, "70"));
    println!("    {}", discount(None, false, "70"));
    println!("    {}", discount(None, false, "30"));
    println!("    {}", discount(None, false, "모름"));
    // 출력:
    //     쿠폰 WELCOME 할인
    //     회원의 날 10% 할인
    //     경로 우대 할인
    //     할인 없음
    //     나이를 확인할 수 없음

    println!("4. let 체인: let 패턴과 조건을 &&로 잇기 (Edition 2024)");
    for text in ["70", "30", "모름"] {
        println!("    {text}: {}", senior_discount(text));
    }
    // 출력:
    //     70: 경로 우대 할인
    //     30: 할인 없음
    //     모름: 할인 없음

    println!("5. while let: 패턴이 맞는 동안 반복합니다");
    let mut stack = vec![1, 2, 3];
    // pop은 꺼낼 원소가 있으면 Some(원소), 비었으면 None을 돌려줍니다.
    // None이 나와 패턴이 맞지 않는 순간 반복이 끝납니다.
    while let Some(top) = stack.pop() {
        println!("    꺼낸 값: {top}");
    }
    println!("    남은 스택: {stack:?}");
    // 출력:
    //     꺼낸 값: 3
    //     꺼낸 값: 2
    //     꺼낸 값: 1
    //     남은 스택: []

    println!("6. for: for 바로 뒤가 패턴입니다");
    let fruits = ["사과", "배", "감"];
    // enumerate는 (인덱스, 원소) 튜플을 내놓고, (index, fruit) 패턴이 그 튜플을 풉니다.
    for (index, fruit) in fruits.iter().enumerate() {
        println!("    {index}번: {fruit}");
    }
    // 출력:
    //     0번: 사과
    //     1번: 배
    //     2번: 감

    println!("7. 함수와 클로저의 매개변수");
    print_coordinates(&(3, 5));
    let rects = [(3, 4), (5, 6)];
    // 클로저 매개변수도 패턴입니다. iter()가 내놓는 &(i32, i32)를 &(w, h)로 풉니다.
    let areas: Vec<i32> = rects.iter().map(|&(w, h)| w * h).collect();
    println!("    넓이: {areas:?}");
    // 출력:
    //     현재 위치: (3, 5)
    //     넓이: [12, 30]

    println!("8. let...else");
    greet("민지:20");
    greet("이름만");
    // 출력:
    //     민지님은 20살입니다
    //     형식이 잘못됐습니다: 이름만
}
