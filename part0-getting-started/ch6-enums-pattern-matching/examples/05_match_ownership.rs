// match도 대입이나 함수 인자와 같은 소유권 규칙(ch4)을 따릅니다.
// - 값을 match하면, 패턴이 묶은 Copy가 아닌 데이터(String 등)는 바인딩된 변수로 이동합니다.
// - 참조(&, &mut)를 match하면, 바인딩된 변수도 참조가 되어 원본은 그대로 남습니다.
// - Copy 타입을 묶거나 _로 묶지 않으면 이동이 일어나지 않습니다.
//
// 실행: cargo run -p p0-ch6-enums-pattern-matching --example 05_match_ownership

#[derive(Debug)]
enum OrderStatus {
    Received,
    Shipped { courier: String, tracking: u32 },
    Cancelled(String),
}

// 상태를 통째로 받아(소유권을 가져와) 취소 사유 String을 꺼내 돌려줍니다.
// reason에는 String이 복제 없이 그대로 옮겨 갑니다. 원본 상태가 더 필요 없을 때 쓰는 방식입니다.
fn into_cancel_reason(status: OrderStatus) -> Option<String> {
    match status {
        OrderStatus::Cancelled(reason) => Some(reason),
        _ => None,
    }
}

// 가변 참조를 받아 상태 안의 데이터를 그 자리에서 고칩니다.
// status는 &mut OrderStatus이므로 바인딩된 tracking은 &mut u32, reason은 &mut String이 됩니다.
fn mark_checked(status: &mut OrderStatus) {
    match status {
        OrderStatus::Received => (),
        OrderStatus::Shipped { courier, tracking } => {
            courier.push_str("(확인)");
            *tracking += 1; // 정수 자체를 바꾸려면 *로 참조를 따라갑니다(ch4).
        }
        OrderStatus::Cancelled(reason) => reason.push_str("(확인)"),
    }
}

fn main() {
    println!("1. 값을 match하면 String은 바인딩된 변수로 이동합니다");
    let status = OrderStatus::Cancelled(String::from("배송 지연"));
    match status {
        // String의 소유권이 reason으로 옮겨 갑니다. reason은 이 갈래가 끝날 때 drop됩니다.
        OrderStatus::Cancelled(reason) => println!("    취소 사유: {reason}"),
        // 이름 있는 포괄 패턴은 값 전체를 묶으므로, 이 갈래로 오면 값 전체가 other로 이동합니다.
        other => println!("    다른 상태: {other:?}"),
    }
    // 어느 갈래로 가든 값의 일부나 전부가 이동하므로, 아래 줄의 주석을 풀면 E0382가 납니다.
    // 컴파일러는 값 전체를 옮기는 other 갈래를 짚어 "borrow of moved value: `status`"라고 알려 줍니다.
    // println!("{status:?}");
    // 출력:
    //     취소 사유: 배송 지연

    println!("2. 소유권을 넘겨 데이터를 꺼내 오는 함수");
    let reason = into_cancel_reason(OrderStatus::Cancelled(String::from("단순 변심")));
    let nothing = into_cancel_reason(OrderStatus::Received);
    println!("    {reason:?}, {nothing:?}");
    // 출력:
    //     Some("단순 변심"), None

    println!("3. 참조를 match하면 바인딩도 참조가 되어 원본을 계속 쓸 수 있습니다");
    let status = OrderStatus::Shipped {
        courier: String::from("번개택배"),
        tracking: 1234,
    };
    match &status {
        // &status를 match하므로 courier는 &String, tracking은 &u32입니다. 아무것도 이동하지 않습니다.
        OrderStatus::Shipped { courier, tracking } => {
            println!("    {courier} 운송장 {tracking}번")
        }
        OrderStatus::Received => println!("    접수됨"),
        OrderStatus::Cancelled(reason) => println!("    취소됨: {reason}"),
    }
    println!("    match 뒤에도 원본 사용 가능: {status:?}");
    // 출력:
    //     번개택배 운송장 1234번
    //     match 뒤에도 원본 사용 가능: Shipped { courier: "번개택배", tracking: 1234 }

    println!("4. 가변 참조를 match하면 바인딩으로 원본 안의 데이터를 고칠 수 있습니다");
    let mut shipped = status; // 3번의 status를 가변 변수로 옮깁니다.
    let mut cancelled = OrderStatus::Cancelled(String::from("주소 오류"));
    mark_checked(&mut shipped);
    mark_checked(&mut cancelled);
    println!("    {shipped:?}");
    println!("    {cancelled:?}");
    // 출력:
    //     Shipped { courier: "번개택배(확인)", tracking: 1235 }
    //     Cancelled("주소 오류(확인)")

    println!("5. 묶은 값이 Copy 타입이면 이동이 아니라 복사입니다");
    let count: Option<i32> = Some(3);
    match count {
        Some(n) => println!("    n = {n}"), // i32는 Copy이므로 n에는 복사본이 들어갑니다.
        None => println!("    없음"),
    }
    println!("    count는 그대로: {count:?}");
    // 출력:
    //     n = 3
    //     count는 그대로: Some(3)

    println!("6. _는 값을 묶지 않으므로 이동도 일어나지 않습니다");
    let status = OrderStatus::Cancelled(String::from("품절"));
    match status {
        OrderStatus::Cancelled(_) => println!("    취소된 주문입니다"),
        OrderStatus::Received => println!("    접수된 주문입니다"),
        OrderStatus::Shipped {
            courier: _,
            tracking: _,
        } => println!("    배송 중인 주문입니다"),
    }
    println!("    match 뒤에도 원본 사용 가능: {status:?}");
    // 출력:
    //     취소된 주문입니다
    //     match 뒤에도 원본 사용 가능: Cancelled("품절")
}
