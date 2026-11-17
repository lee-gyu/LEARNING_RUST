// 한 가지 패턴에만 관심이 있을 때는 match 대신 if let으로 짧게 씁니다.
// if let ... else는 패턴이 맞지 않을 때 할 일을 정하고, else if let으로 다른 패턴을 이어서 검사할 수 있습니다.
// 대신 if let은 match와 달리 모든 경우를 다뤘는지 검사하지 않습니다.
//
// 실행: cargo run -p p0-ch6-enums-pattern-matching --example 06_if_let

#[derive(Debug)]
enum OrderStatus {
    Received,
    Shipped { courier: String, tracking: u32 },
    Cancelled(String),
}

// if let도 표현식이므로 값을 만들어 낼 수 있습니다. 이때는 else가 꼭 있어야 합니다.
// 패턴이 맞지 않을 때도 무언가를 돌려줘야 하기 때문입니다.
fn greeting(nickname: Option<&str>) -> String {
    if let Some(name) = nickname {
        format!("안녕하세요, {name}님!")
    } else {
        String::from("안녕하세요, 손님!")
    }
}

fn main() {
    let max_connections: Option<u32> = Some(100);

    println!("1. match로 한 패턴만 처리하면 _ => ()가 덧붙습니다");
    // Clippy도 이 코드를 보면 if let을 쓰라고 제안합니다(clippy::single_match).
    // 2번과 비교하려고 일부러 남겨 두었으므로 expect로 표시합니다.
    #[expect(clippy::single_match)]
    match max_connections {
        Some(max) => println!("    최대 접속 수: {max}"),
        _ => (),
    }
    // 출력:
    //     최대 접속 수: 100

    println!("2. if let으로 쓰면 같은 일을 더 짧게 쓸 수 있습니다");
    // "if let 패턴 = 값"의 순서로 씁니다. 값이 패턴과 맞으면 블록을 실행하고, 맞지 않으면 건너뜁니다.
    if let Some(max) = max_connections {
        println!("    최대 접속 수: {max}");
    }
    let no_limit: Option<u32> = None;
    if let Some(max) = no_limit {
        println!("    최대 접속 수: {max}"); // None이므로 실행되지 않습니다.
    }
    // 출력:
    //     최대 접속 수: 100

    println!("3. if let ... else: 패턴이 맞지 않을 때 할 일을 정합니다");
    println!("    {}", greeting(Some("러스타")));
    println!("    {}", greeting(None));
    // 출력:
    //     안녕하세요, 러스타님!
    //     안녕하세요, 손님!

    let orders = [
        OrderStatus::Received,
        OrderStatus::Shipped {
            courier: String::from("번개택배"),
            tracking: 1234,
        },
        OrderStatus::Cancelled(String::from("단순 변심")),
        OrderStatus::Shipped {
            courier: String::from("새벽택배"),
            tracking: 5678,
        },
    ];

    println!("4. 열거형에서 한 배리언트만 골라 처리하고, 나머지는 else에서 셉니다");
    let mut others = 0;
    // &orders를 순회하므로 status는 &OrderStatus이고, 바인딩된 courier와 tracking도 참조입니다(05 예제).
    for status in &orders {
        if let OrderStatus::Shipped { courier, tracking } = status {
            println!("    배송 중: {courier} 운송장 {tracking}번");
        } else {
            others += 1;
        }
    }
    println!("    배송 중이 아닌 주문: {others}건");
    // 출력:
    //     배송 중: 번개택배 운송장 1234번
    //     배송 중: 새벽택배 운송장 5678번
    //     배송 중이 아닌 주문: 2건

    println!("5. else if let으로 다른 패턴을 이어서 검사할 수 있습니다");
    for status in &orders {
        if let OrderStatus::Cancelled(reason) = status {
            println!("    취소: {reason}");
        } else if let OrderStatus::Received = status {
            println!("    접수");
        } else {
            println!("    그 밖의 상태: {status:?}");
        }
    }
    // 출력:
    //     접수
    //     그 밖의 상태: Shipped { courier: "번개택배", tracking: 1234 }
    //     취소: 단순 변심
    //     그 밖의 상태: Shipped { courier: "새벽택배", tracking: 5678 }
    // 이렇게 갈래가 늘어나면 match가 더 읽기 쉽습니다. if let 사슬은 빠진 경우가 있어도 컴파일러가 알려 주지 않습니다.
}
