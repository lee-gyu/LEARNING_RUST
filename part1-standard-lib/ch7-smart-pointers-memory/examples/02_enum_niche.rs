// 이 예제가 보여 주는 것: 열거형의 크기(가장 큰 배리언트 + 태그)와 니치 최적화,
//                        mem::discriminant로 배리언트만 비교하기
// 실행: cargo run -p p1-ch7-smart-pointers-memory --example 02_enum_niche
//
// 크기는 64비트 환경, rustc 1.100.0에서 확인한 값입니다. 2번 구역(Box, 참조)은 std::option 문서가
// 보장하는 결과이고, 3·4번 구역은 이 툴체인의 컴파일러가 해 준 최적화라 보장되지는 않습니다.

use std::mem;

// 아래 타입들은 크기만 재고 값을 만들지 않으므로, 쓰지 않는 배리언트에 대한 경고를 예상된 것으로 표시합니다.
#[expect(dead_code, reason = "크기만 재는 타입이라 값을 만들지 않습니다")]
enum Direction {
    North,
    East,
    South,
    West,
}

#[expect(dead_code, reason = "크기만 재는 타입이라 값을 만들지 않습니다")]
enum Shape {
    Dot,
    Circle(i32),
    Rect(i32, i32),
}

// Part 0 ch8·ch17의 Cell 열거형과 같은 모양입니다. 이 챕터의 std::cell::Cell과 헷갈리지 않게 이름만 바꿨습니다.
#[expect(dead_code, reason = "크기만 재는 타입이라 값을 만들지 않습니다")]
enum SheetCell {
    Int(i32),
    Float(f64),
    Text(String),
}

// 두 배리언트가 모두 String을 담습니다.
#[expect(dead_code, reason = "크기만 재는 타입이라 값을 만들지 않습니다")]
enum TwoTexts {
    Title(String),
    Body(String),
}

// Part 0 ch14의 cons 리스트입니다.
#[expect(dead_code, reason = "크기만 재는 타입이라 값을 만들지 않습니다")]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

enum Event {
    Click { x: i32, y: i32 },
    Key(char),
    Quit,
}

impl Event {
    fn label(&self) -> String {
        match self {
            Event::Click { x, y } => format!("Click({x},{y})"),
            Event::Key(c) => format!("Key({c})"),
            Event::Quit => String::from("Quit"),
        }
    }
}

fn size<T>(name: &str) {
    println!("{name:<26} {:>2}바이트", size_of::<T>());
}

fn main() {
    // 태그(tag)는 "지금 어느 배리언트인가"를 적는 칸입니다. 배리언트가 4개면 1바이트로 충분합니다.
    // Shape는 가장 큰 Rect(8바이트) + 태그인데, 태그가 i32의 정렬(4)에 맞춰 자리를 잡아 12바이트입니다.
    // u32, u64, f64는 모든 비트 조합이 올바른 값이라 None을 적을 빈자리가 없고, 태그 칸이 따로 붙습니다.
    println!("== 1. 열거형의 크기 = 가장 큰 배리언트 + 태그 ==");
    size::<Direction>("Direction");
    size::<Shape>("Shape");
    size::<Option<u32>>("Option<u32>");
    size::<Option<u64>>("Option<u64>");
    size::<Option<f64>>("Option<f64>");
    // 출력:
    // Direction                   1바이트
    // Shape                      12바이트
    // Option<u32>                 8바이트
    // Option<u64>                16바이트
    // Option<f64>                16바이트

    // Box와 참조는 절대 0(null)이 아닙니다. 그래서 "포인터 칸이 0이면 None"으로 정하면 태그가 필요 없습니다.
    // 이것을 니치 최적화(niche optimization)라고 부릅니다. 니치는 그 타입이 결코 쓰지 않는 값(빈자리)입니다.
    println!("\n== 2. 니치 최적화: 포인터는 0이 될 수 없습니다 (문서가 보장) ==");
    size::<Box<i32>>("Box<i32>");
    size::<Option<Box<i32>>>("Option<Box<i32>>");
    size::<&i32>("&i32");
    size::<Option<&i32>>("Option<&i32>");
    size::<Option<&str>>("Option<&str>");
    size::<List>("List (Cons/Nil)");
    // 출력:
    // Box<i32>                    8바이트
    // Option<Box<i32>>            8바이트
    // &i32                        8바이트
    // Option<&i32>                8바이트
    // Option<&str>               16바이트
    // List (Cons/Nil)            16바이트
    // List는 i32(4) + 빈칸(4) + Box(8) = 16바이트이고, Nil은 Box 칸이 0인 값으로 나타냅니다.

    // bool은 0과 1만 쓰므로 2~255가 비어 있습니다. char는 0x10FFFF까지만 쓰므로 그 위가 비어 있습니다.
    // Direction은 0~3만 쓰는 태그 하나뿐이라 나머지 값이 비어 있습니다.
    println!("\n== 3. 니치 최적화: 값의 범위가 좁은 타입 (이 툴체인의 결과) ==");
    size::<Option<bool>>("Option<bool>");
    size::<Option<Option<bool>>>("Option<Option<bool>>");
    size::<Option<char>>("Option<char>");
    size::<Option<Direction>>("Option<Direction>");
    // 출력:
    // Option<bool>                1바이트
    // Option<Option<bool>>        1바이트
    // Option<char>                4바이트
    // Option<Direction>           1바이트

    // String(속은 Vec<u8>)의 용량 칸은 현재 구현에서 0..=isize::MAX만 쓰도록 표시되어 있어,
    // 최상위 비트가 1인 값 수많은 개가 비어 있습니다. 컴파일러는 그 자리에 배리언트 정보를 적습니다.
    println!("\n== 4. 큰 타입 안의 빈자리 (이 툴체인의 결과) ==");
    size::<String>("String");
    size::<Option<String>>("Option<String>");
    size::<SheetCell>("SheetCell (Int/Float/Text)");
    size::<Option<Option<String>>>("Option<Option<String>>");
    size::<Option<Option<Box<i32>>>>("Option<Option<Box<i32>>>");
    size::<TwoTexts>("TwoTexts");
    // 출력:
    // String                     24바이트
    // Option<String>             24바이트
    // SheetCell (Int/Float/Text) 24바이트
    // Option<Option<String>>     24바이트
    // Option<Option<Box<i32>>>   16바이트
    // TwoTexts                   32바이트
    // - SheetCell: Int, Float의 데이터는 용량 칸과 겹치지 않는 자리에 두고, 용량 칸의 빈 값으로 배리언트를 구분합니다.
    // - Option<Option<String>>: 빈 값이 여러 개라 None 두 종류를 모두 적을 수 있습니다.
    // - Option<Option<Box<i32>>>: Box의 빈자리는 0 하나뿐이라, 바깥 Option은 태그 칸을 따로 둡니다.
    // - TwoTexts: 두 배리언트가 모두 String 하나를 통째로 담아 용량 칸까지 쓰므로, 빈자리를 쓸 수 없어 태그 칸이 붙습니다.

    // mem::discriminant는 배리언트만 비교할 수 있는 값을 돌려줍니다. 필드 값은 무시합니다.
    println!("\n== 5. mem::discriminant: 필드는 무시하고 배리언트만 비교 ==");
    let a = Event::Click { x: 10, y: 20 };
    let b = Event::Click { x: 0, y: 0 };
    let c = Event::Key('q');
    let same_ab = mem::discriminant(&a) == mem::discriminant(&b);
    let same_ac = mem::discriminant(&a) == mem::discriminant(&c);
    println!("{} 와 {}: 같은 배리언트? {same_ab}", a.label(), b.label());
    // 출력: Click(10,20) 와 Click(0,0): 같은 배리언트? true
    println!("{} 와 {}: 같은 배리언트? {same_ac}", a.label(), c.label());
    // 출력: Click(10,20) 와 Key(q): 같은 배리언트? false

    // 배리언트가 몇 가지든 "같은 종류의 이벤트만 남기기"를 한 줄 조건으로 쓸 수 있습니다.
    let events = [a, c, Event::Quit, b, Event::Key('w')];
    let wanted = mem::discriminant(&Event::Key(' '));
    let keys: Vec<String> = events
        .iter()
        .filter(|e| mem::discriminant(*e) == wanted)
        .map(Event::label)
        .collect();
    println!("Key 이벤트만: {keys:?}"); // 출력: Key 이벤트만: ["Key(q)", "Key(w)"]
}
