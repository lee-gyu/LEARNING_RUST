// 열거형(enum)은 "가능한 경우 중 정확히 하나"인 값을 나타내는 타입입니다.
// 배리언트마다 데이터 없음 / 구조체형 / 튜플형으로 서로 다른 데이터를 담을 수 있다는 것을,
// 같은 내용을 구조체 하나로 표현했을 때와 비교해 보여 줍니다.
//
// 실행: cargo run -p p0-ch6-enums-pattern-matching --example 01_enum_variants

// 이 예제는 두 방법의 "모양"을 비교하는 것이 목적이라, 값을 만들어 {:?}로 출력하기만 하고
// 필드를 읽지 않습니다. derive(Debug)가 만든 출력 코드는 "읽었다"로 치지 않으므로
// "never read"/"never constructed" 같은 dead_code 경고가 납니다. 의도한 것이므로 expect로 표시합니다.
// 열거형의 필드를 읽는 방법(match)은 02 예제에서 배웁니다.
#![expect(dead_code)]

// ---------------------------------------------------------------------------
// 방법 1. 구조체 하나 + "종류" 필드
// ---------------------------------------------------------------------------

// 배송 상태의 "종류"만 나타내는 열거형입니다. 배리언트에 데이터가 없으므로
// ch2에서 본 `Ordering`(Less, Greater, Equal)과 같은 모양입니다.
#[derive(Debug)]
enum StatusKind {
    Received,
    Shipped,
    Delivered,
    Cancelled,
}

// 종류마다 필요한 데이터가 다르지만, 구조체는 모든 필드를 항상 채워야 합니다.
#[derive(Debug)]
struct OrderRecord {
    kind: StatusKind,
    courier: String, // Shipped일 때만 의미가 있습니다.
    tracking: u32,   // Shipped일 때만 의미가 있습니다.
    place: String,   // Delivered일 때만 의미가 있습니다.
    reason: String,  // Cancelled일 때만 의미가 있습니다.
}

// ---------------------------------------------------------------------------
// 방법 2. 데이터를 가진 열거형
// ---------------------------------------------------------------------------

// 배리언트마다 자기에게 필요한 데이터만 가집니다.
#[derive(Debug)]
enum OrderStatus {
    Received,                                   // 데이터 없음
    Shipped { courier: String, tracking: u32 }, // 구조체형: 이름 붙은 필드
    Delivered(String),                          // 튜플형: 수령 장소
    Cancelled(String),                          // 튜플형: 취소 사유
}

// 매개변수 타입이 OrderStatus 하나뿐이지만, 네 배리언트 중 무엇이든 받을 수 있습니다.
// 소유권을 가져갈 필요가 없으므로 참조로 빌립니다(ch4).
fn log(status: &OrderStatus) {
    println!("    [로그] {status:?}");
}

fn main() {
    println!("1. 구조체 하나로 표현하면 쓰지 않는 필드도 채워야 합니다");
    let received = OrderRecord {
        kind: StatusKind::Received,
        courier: String::new(), // 접수 단계에는 택배사가 없지만 무언가를 넣어야 합니다.
        tracking: 0,            // 0이 "없음"이라는 약속은 코드 어디에도 드러나지 않습니다.
        place: String::new(),
        reason: String::new(),
    };
    println!("    {received:?}");
    // 출력:
    //     OrderRecord { kind: Received, courier: "", tracking: 0, place: "", reason: "" }

    println!("2. 구조체로는 말이 안 되는 조합도 만들 수 있습니다");
    let broken = OrderRecord {
        kind: StatusKind::Cancelled,       // 취소된 주문인데
        courier: String::from("번개택배"), // 택배사와 운송장 번호가 있습니다.
        tracking: 1234,
        place: String::new(),
        reason: String::new(), // 정작 취소 사유는 비어 있습니다.
    };
    println!("    {broken:?}");
    // 출력:
    //     OrderRecord { kind: Cancelled, courier: "번개택배", tracking: 1234, place: "", reason: "" }
    // 컴파일러는 이 값이 이상하다는 것을 알 수 없습니다. 규칙이 타입에 들어 있지 않기 때문입니다.

    println!("3. 열거형은 배리언트마다 필요한 데이터만 담습니다");
    // 배리언트 이름은 열거형 이름 아래에 있으므로 `OrderStatus::Received`처럼 ::로 부릅니다.
    // 데이터를 가진 배리언트는 값을 만드는 함수처럼 씁니다: Delivered(...)에 String을 넣으면 OrderStatus가 나옵니다.
    let received = OrderStatus::Received;
    let shipped = OrderStatus::Shipped {
        courier: String::from("번개택배"),
        tracking: 1234,
    };
    let delivered = OrderStatus::Delivered(String::from("문 앞"));
    let cancelled = OrderStatus::Cancelled(String::from("단순 변심"));
    println!("    {received:?}");
    println!("    {shipped:?}");
    println!("    {delivered:?}");
    println!("    {cancelled:?}");
    // 출력:
    //     Received
    //     Shipped { courier: "번개택배", tracking: 1234 }
    //     Delivered("문 앞")
    //     Cancelled("단순 변심")
    // "취소됐는데 운송장 번호가 있는" 값은 만들 방법이 아예 없습니다. Cancelled에는 사유만 들어가기 때문입니다.

    println!("4. 모양이 달라도 타입은 하나이므로 한 함수로 받고, 한 배열에 담을 수 있습니다");
    log(&shipped);
    log(&cancelled);
    // 배열은 모든 원소의 타입이 같아야 합니다(ch3). 네 값이 모두 OrderStatus이므로 한 배열에 들어갑니다.
    // 배리언트마다 구조체를 따로 만들었다면 타입이 네 개가 되어 이렇게 할 수 없습니다.
    let history = [received, shipped, delivered, cancelled];
    println!("    배열 길이: {}", history.len());
    // 출력:
    //     [로그] Shipped { courier: "번개택배", tracking: 1234 }
    //     [로그] Cancelled("단순 변심")
    //     배열 길이: 4
}
