// Any와 다운캐스트: 타입을 실행 중에 묻기.
// TypeId와 type_name, &dyn Any의 is/downcast_ref/downcast_mut, Box<dyn Any>의 downcast,
// "상자 자체도 Any"라는 함정, 그리고 패닉 페이로드가 같은 원리라는 것을 보여 줍니다.
//
// 실행: cargo run -p p1-ch14-ffi-any-misc --example 05_any_downcast
//
// TypeId의 값은 툴체인마다 달라서 출력하지 않고 같은지만 비교합니다.
// type_name이 돌려주는 문자열은 진단용이라 툴체인에 따라 달라질 수 있습니다.

use std::any::{Any, TypeId, type_name, type_name_of_val};

type Meters = u32; // 타입 별칭: u32에 붙인 다른 이름일 뿐 새 타입이 아닙니다.
struct UserId(#[expect(dead_code, reason = "TypeId 비교에만 쓰는 뉴타입입니다")] u32); // 뉴타입: u32와 다른 새 타입

/// 어떤 'static 타입의 값이든 받아서, 아는 타입이면 꺼내 설명합니다.
fn describe(value: &dyn Any) -> String {
    // downcast_ref::<T>()는 안에 든 값이 정확히 T일 때만 Some(&T)입니다.
    if let Some(n) = value.downcast_ref::<i32>() {
        format!("i32 {n}")
    } else if let Some(s) = value.downcast_ref::<String>() {
        format!("String {s:?} ({}바이트)", s.len())
    } else if let Some(s) = value.downcast_ref::<&str>() {
        format!("&str {s:?}")
    } else if value.is::<f64>() {
        // 값이 필요 없고 타입만 확인하면 is::<T>()로 충분합니다.
        String::from("f64 (타입만 확인)")
    } else {
        String::from("모르는 타입")
    }
}

/// 안에 든 값이 u32이면 1 올립니다.
fn bump_if_counter(value: &mut dyn Any) -> bool {
    match value.downcast_mut::<u32>() {
        Some(n) => {
            *n += 1;
            true
        }
        None => false,
    }
}

/// Part 1 ch8의 패닉 페이로드 함수와 같은 모양입니다.
fn payload_text(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        format!("&str 페이로드: {s}")
    } else if let Some(s) = payload.downcast_ref::<String>() {
        format!("String 페이로드: {s}")
    } else {
        String::from("문자열이 아닌 페이로드")
    }
}

fn main() {
    println!("1. TypeId: 타입마다 하나인 식별자");
    // Any 트레이트의 type_id()는 값의 타입의 TypeId를, TypeId::of::<T>()는 T의 TypeId를 줍니다.
    println!(
        "    7i32.type_id() == TypeId::of::<i32>(): {}",
        7i32.type_id() == TypeId::of::<i32>()
    );
    println!(
        "    &str과 String: {}",
        TypeId::of::<&str>() == TypeId::of::<String>()
    );
    // 별칭은 같은 타입이므로 TypeId도 같고, 뉴타입은 다른 타입이므로 TypeId가 다릅니다.
    println!(
        "    별칭 Meters와 u32: {}, 뉴타입 UserId와 u32: {}",
        TypeId::of::<Meters>() == TypeId::of::<u32>(),
        TypeId::of::<UserId>() == TypeId::of::<u32>()
    );
    // 출력:     7i32.type_id() == TypeId::of::<i32>(): true
    // 출력:     &str과 String: false
    // 출력:     별칭 Meters와 u32: true, 뉴타입 UserId와 u32: false

    println!("2. type_name: 사람이 읽는 타입 이름 (진단용)");
    println!("    {}", type_name::<Vec<Option<&str>>>());
    println!("    {}", type_name_of_val(&1.5));
    // 출력:     alloc::vec::Vec<core::option::Option<&str>>
    // 출력:     f64
    // 트레이트 객체를 넘기면 안에 든 타입이 아니라 "dyn Any"라고만 말합니다.
    let erased: &dyn Any = &42u8;
    println!("    {}", type_name_of_val(erased)); // 출력:     dyn core::any::Any

    println!("3. &dyn Any에서 구체 타입 꺼내기: is, downcast_ref");
    let owned = String::from("러스트");
    let values: [&dyn Any; 6] = [&42, &owned, &"러스트", &1.5, &42u8, &vec![1, 2]];
    for value in values {
        println!("    {}", describe(value));
    }
    // 출력:     i32 42
    // 출력:     String "러스트" (9바이트)
    // 출력:     &str "러스트"
    // 출력:     f64 (타입만 확인)
    // 출력:     모르는 타입
    // 출력:     모르는 타입
    // 42u8은 i32가 아니므로 "모르는 타입"입니다. 다운캐스트는 정확히 같은 타입만 찾고 변환하지 않습니다.

    println!("4. downcast_mut로 고치기");
    let mut hits = 9u32;
    let mut label = String::from("방문 수");
    let (a, b) = (bump_if_counter(&mut hits), bump_if_counter(&mut label));
    println!("    u32였는가: {a}, String이었는가(u32로 꺼내기): {b}, hits = {hits}");
    // 출력:     u32였는가: true, String이었는가(u32로 꺼내기): false, hits = 10

    println!("5. Box<dyn Any>::downcast는 소유권째 꺼내고, 틀리면 상자를 돌려줍니다");
    let boxed: Box<dyn Any> = Box::new(String::from("상자 안의 문자열"));
    // downcast는 Result<Box<T>, Box<dyn Any>>입니다. Err에 원래 상자가 들어 있어 다시 시도할 수 있습니다.
    let boxed = match boxed.downcast::<i32>() {
        Ok(n) => {
            println!("    i32: {n}");
            return;
        }
        Err(original) => {
            println!("    i32가 아니라서 상자를 돌려받았습니다");
            original
        }
    };
    let text: Box<String> = boxed.downcast::<String>().unwrap();
    println!("    String으로 꺼냄: {text}");
    // 출력:     i32가 아니라서 상자를 돌려받았습니다
    // 출력:     String으로 꺼냄: 상자 안의 문자열

    println!("6. 상자 자체도 Any입니다");
    let boxed: Box<dyn Any> = Box::new(5u8);
    // Box<dyn Any>도 'static 타입이라 Any를 구현합니다. 그래서 boxed.type_id()는
    // 메서드 탐색에서 상자 자체의 type_id를 먼저 찾습니다. Clippy가 type_id_on_box로 알려 주는 실수입니다.
    #[expect(
        clippy::type_id_on_box,
        reason = "상자 자체의 TypeId가 나오는 것을 보여 주려는 코드입니다"
    )]
    let box_id = boxed.type_id();
    let inner_id = (*boxed).type_id(); // *boxed는 dyn Any이므로 안에 든 값의 TypeId입니다.
    println!(
        "    boxed.type_id()가 Box<dyn Any>의 것인가: {}, (*boxed).type_id()가 u8의 것인가: {}",
        box_id == TypeId::of::<Box<dyn Any>>(),
        inner_id == TypeId::of::<u8>()
    );
    // 출력:     boxed.type_id()가 Box<dyn Any>의 것인가: true, (*boxed).type_id()가 u8의 것인가: true
    // 같은 이유로 &boxed를 넘기면 상자가 Any 값이 되어 다운캐스트가 모두 실패합니다.
    let number: Box<dyn Any> = Box::new(42);
    println!(
        "    describe(&number) = {}, describe(&*number) = {}",
        describe(&number),
        describe(&*number)
    );
    // 출력:     describe(&number) = 모르는 타입, describe(&*number) = i32 42

    println!("7. 패닉 페이로드도 Box<dyn Any + Send>입니다");
    // panic!("상수") 의 페이로드는 &'static str, 서식 인자가 있으면 String입니다(Part 1 ch8).
    // 여기서는 패닉 없이 같은 타입의 값을 직접 만들어 같은 함수로 꺼냅니다.
    let payloads: Vec<Box<dyn Any + Send>> = vec![
        Box::new("상수 메시지"),
        Box::new(format!("{}번 시도 실패", 3)),
        Box::new(404u16),
    ];
    for payload in &payloads {
        println!("    {}", payload_text(payload.as_ref()));
    }
    // 출력:     &str 페이로드: 상수 메시지
    // 출력:     String 페이로드: 3번 시도 실패
    // 출력:     문자열이 아닌 페이로드
}
