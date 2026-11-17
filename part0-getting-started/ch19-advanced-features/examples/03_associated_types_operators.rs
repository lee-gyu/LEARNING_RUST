// 연관 타입과 제네릭 트레이트의 차이, 기본 제네릭 타입 매개변수(Add<Rhs = Self>)와
// 연산자 오버로딩(Point + Point, Millimeters + Meters)을 보여 줍니다.
//
// 실행: cargo run -p p0-ch19-advanced-features --example 03_associated_types_operators

use std::ops::Add;

// 1. 연관 타입: 표준 라이브러리의 Iterator는 type Item을 연관 타입으로 둡니다.
struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32; // Counter에 대한 Iterator 구현은 하나뿐이고, Item도 하나로 정해집니다.

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 3 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

// 2. 비교용: 같은 일을 제네릭 트레이트로 정의하면
//    한 타입에 타입 인자만 바꿔 여러 번 구현할 수 있습니다.
trait GenericIterator<T> {
    fn next_item(&mut self) -> Option<T>;
}

impl GenericIterator<u32> for Counter {
    fn next_item(&mut self) -> Option<u32> {
        self.next() // 위에서 구현한 Iterator::next를 그대로 씁니다.
    }
}

impl GenericIterator<String> for Counter {
    fn next_item(&mut self) -> Option<String> {
        self.next().map(|n| format!("#{n}"))
    }
}

// 연관 타입에 조건을 걸 때는 Iterator<Item = u32>처럼 씁니다(ch13).
fn total(iter: impl Iterator<Item = u32>) -> u32 {
    iter.sum()
}

// 3. 연산자 오버로딩: + 연산자는 std::ops::Add 트레이트의 add 메서드입니다.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

// Add의 정의는 trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Self::Output; }입니다.
// Rhs를 적지 않았으므로 기본값 Self, 즉 Point + Point를 구현합니다.
impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// 4. Rhs를 직접 정하면 서로 다른 타입끼리 더할 수 있습니다.
#[derive(Debug)]
struct Millimeters(u32);
#[derive(Debug)]
struct Meters(u32);

impl Add<Meters> for Millimeters {
    type Output = Millimeters;

    fn add(self, other: Meters) -> Millimeters {
        Millimeters(self.0 + other.0 * 1000)
    }
}

fn main() {
    println!("1. 연관 타입: 타입 표기 없이 씁니다");
    let mut counter = Counter { count: 0 };
    println!("    counter.next() = {:?}", counter.next()); // 출력:     counter.next() = Some(1)
    println!("    total(Counter) = {}", total(Counter { count: 0 })); // 출력:     total(Counter) = 6

    println!("2. 제네릭 트레이트: 어느 구현인지 알려 줘야 합니다");
    let mut counter = Counter { count: 0 };
    // `counter.next_item()`만 쓰면 u32 구현인지 String 구현인지 정할 수 없어 E0283입니다.
    let as_number: Option<u32> = counter.next_item();
    let as_text: Option<String> = counter.next_item();
    let by_path = GenericIterator::<u32>::next_item(&mut counter);
    println!("    {as_number:?}, {as_text:?}, {by_path:?}"); // 출력:     Some(1), Some("#2"), Some(3)

    println!("3. Point + Point");
    let p = Point { x: 1, y: 0 } + Point { x: 2, y: 3 };
    println!("    {p:?}"); // 출력:     Point { x: 3, y: 3 }
    // a + b는 a.add(b)를 짧게 쓴 것입니다. Point가 Copy라서 p를 여러 번 쓸 수 있습니다.
    println!("    p + p == p.add(p)? {}", p + p == p.add(p)); // 출력:     p + p == p.add(p)? true

    println!("4. Millimeters + Meters");
    let length = Millimeters(500) + Meters(2);
    println!("    {length:?}"); // 출력:     Millimeters(2500)
    // 반대 순서(Meters + Millimeters)는 구현하지 않았으므로 E0369입니다.

    println!("5. ch8의 String + &str도 Add입니다");
    let hello = String::from("안녕");
    // 표준 라이브러리에 impl Add<&str> for String { type Output = String; ... }이 있습니다.
    let greeting = Add::add(hello, "하세요");
    println!("    {greeting}"); // 출력:     안녕하세요
}
