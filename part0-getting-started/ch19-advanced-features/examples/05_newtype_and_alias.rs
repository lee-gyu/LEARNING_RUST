// 뉴타입 패턴(고아 규칙 우회, 타입 안전성)과 타입 별칭(type)을 비교해 보여 줍니다.
//
// 실행: cargo run -p p0-ch19-advanced-features --example 05_newtype_and_alias

use std::fmt;
use std::mem::size_of;
use std::ops::Deref;

// 1. 고아 규칙 우회: Display(표준)를 Vec<String>(표준)에 직접 구현할 수는 없습니다(E0117).
//    Vec<String>을 감싼 내 타입을 만들면 "내 타입"에 구현하는 것이 되어 허용됩니다.
struct Wrapper(Vec<String>);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

// 뉴타입은 안쪽 타입의 메서드를 물려받지 않습니다.
// Deref(ch14)를 구현하면 &Wrapper가 &Vec<String>처럼 동작해 len(), iter() 등을 쓸 수 있습니다.
impl Deref for Wrapper {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

// 2. 타입 안전성: 둘 다 속은 u32지만 서로 다른 타입입니다.
#[derive(Debug, Clone, Copy)]
struct UserId(u32);
#[derive(Debug, Clone, Copy)]
struct OrderId(u32);

fn cancel_order(id: OrderId) -> String {
    format!("주문 {}번을 취소했습니다", id.0)
}

// 3. 타입 별칭: 새 타입이 아니라 이미 있는 타입의 다른 이름입니다.
type Kilometers = i32;

// 긴 타입 이름을 줄이는 것이 별칭의 주된 쓸모입니다.
type Thunk = Box<dyn Fn() -> String + Send + 'static>;

fn make_thunks() -> Vec<Thunk> {
    vec![
        Box::new(|| String::from("나중에 실행 1")),
        Box::new(|| String::from("나중에 실행 2")),
    ]
}

// std::io::Result<T>처럼 에러 타입을 고정한 Result 별칭을 만들 수 있습니다.
#[derive(Debug)]
struct ParseError(String);
type Result<T> = std::result::Result<T, ParseError>;

fn parse_age(text: &str) -> Result<u8> {
    text.trim()
        .parse()
        .map_err(|_| ParseError(format!("나이가 아닙니다: '{text}'")))
}

fn main() {
    println!("1. 뉴타입으로 외부 타입에 외부 트레이트 구현하기");
    let w = Wrapper(vec![String::from("안녕"), String::from("세상")]);
    println!("    w = {w}"); // 출력:     w = [안녕, 세상]
    println!("    w.len() = {}, 첫 원소 = {:?}", w.len(), w.first()); // 출력:     w.len() = 2, 첫 원소 = Some("안녕")

    println!("2. 뉴타입으로 값을 헷갈리지 않게 하기");
    let user = UserId(7);
    let order = OrderId(1024);
    println!("    {}", cancel_order(order)); // 출력:     주문 1024번을 취소했습니다
    // cancel_order(user)는 E0308(expected `OrderId`, found `UserId`)입니다.
    println!("    user = {user:?}, 안의 값 = {}", user.0); // 출력:     user = UserId(7), 안의 값 = 7
    // 감싸기만 했으므로 실행 중 비용이 없습니다. 크기도 안쪽 타입과 같습니다.
    println!(
        "    size_of::<UserId>() = {}, size_of::<u32>() = {}",
        size_of::<UserId>(),
        size_of::<u32>()
    ); // 출력:     size_of::<UserId>() = 4, size_of::<u32>() = 4

    println!("3. 타입 별칭은 같은 타입입니다");
    let distance: Kilometers = 5;
    let extra: i32 = 3;
    // Kilometers와 i32는 완전히 같은 타입이라 그대로 더할 수 있습니다. 뉴타입과 달리 실수를 막아 주지 않습니다.
    println!("    distance + extra = {}", distance + extra); // 출력:     distance + extra = 8
    for thunk in make_thunks() {
        println!("    {}", thunk());
    }
    // 출력:
    //     나중에 실행 1
    //     나중에 실행 2

    println!("4. Result 별칭");
    println!("    {:?}", parse_age("42")); // 출력:     Ok(42)
    match parse_age("마흔") {
        Ok(age) => println!("    나이: {age}"),
        Err(ParseError(message)) => println!("    에러: {message}"), // 출력:     에러: 나이가 아닙니다: '마흔'
    }
}
