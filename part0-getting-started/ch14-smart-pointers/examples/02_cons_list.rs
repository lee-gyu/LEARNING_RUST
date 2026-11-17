// Box<T>로 재귀 타입(자기 자신을 품는 타입)인 cons 리스트를 만듭니다.
// - 열거형의 크기는 "가장 큰 배리언트 + 어느 배리언트인지 표시하는 태그"로 정해집니다.
// - List 안에 List를 직접 넣으면 크기 계산이 끝나지 않습니다(compile_fail/e0072_recursive_type.rs).
// - Box<List>는 포인터 한 칸이라 크기가 정해져 있으므로, 계산이 유한한 값으로 끝납니다.
//
// 실행: cargo run -p p0-ch14-smart-pointers --example 02_cons_list

// cons 리스트: 각 칸이 (값, 다음 칸)으로 이뤄지고, 마지막 칸은 Nil입니다.
// 다음 칸을 Box에 담았기 때문에 Cons 배리언트의 크기는 i32 하나 + 포인터 하나로 고정됩니다.
#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use crate::List::{Cons, Nil};

// 크기 계산을 보여 주기 위한 재귀가 아닌 열거형입니다. 값은 만들지 않고 크기만 잽니다.
#[expect(dead_code, reason = "크기만 재는 타입이라 배리언트를 만들지 않습니다")]
enum Shape {
    Dot,
    Circle(i32),
    Rect(i32, i32),
}

// 리스트의 합을 재귀로 구합니다.
fn sum(list: &List) -> i32 {
    match list {
        // rest의 타입은 &Box<List>인데 sum은 &List를 받습니다.
        // &Box<List>가 &List로 자동으로 바뀌는 것은 역참조 강제 변환 덕분입니다(03번 예제).
        Cons(value, rest) => value + sum(rest),
        Nil => 0,
    }
}

// 리스트를 "1 -> 2 -> 3 -> Nil" 모양의 문자열로 만듭니다.
fn to_text(list: &List) -> String {
    match list {
        Cons(value, rest) => format!("{value} -> {}", to_text(rest)),
        Nil => String::from("Nil"),
    }
}

fn main() {
    println!("1. Box로 다음 칸을 감싸 리스트를 만듭니다");
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
    // Box는 디버그 형식으로 출력할 때 안의 값만 보여 줍니다.
    println!("    {list:?}"); // 출력: Cons(1, Cons(2, Cons(3, Nil)))
    println!("    {}", to_text(&list)); // 출력: 1 -> 2 -> 3 -> Nil

    println!("2. 재귀 타입은 재귀 함수로 다룹니다");
    println!("    합계 = {}", sum(&list)); // 출력: 합계 = 6

    println!("3. 열거형의 크기 = 가장 큰 배리언트 + 태그");
    // Shape에서 가장 큰 배리언트는 Rect(i32 두 개 = 8바이트)입니다.
    // 여기에 어느 배리언트인지 적는 태그 칸이 붙어 12바이트가 됩니다.
    // (태그 값 자체는 1바이트면 충분하지만 i32에 맞춰 4바이트 단위로 자리를 잡습니다.)
    println!("    Shape: {}바이트", size_of::<Shape>()); // 출력: Shape: 12바이트

    println!("4. List의 크기 계산은 Box 덕분에 유한한 값으로 끝납니다");
    // Cons = i32(4) + Box<List>(8). 8바이트 값은 8의 배수 위치에 놓여야 하므로
    // i32 뒤에 빈칸 4바이트가 생겨 16바이트가 됩니다.
    // 태그 칸이 따로 없는 것은 Box가 절대 주소 0(null)을 갖지 않기 때문입니다.
    // 컴파일러는 Box 자리에 0이 들어 있으면 Nil로 보는 식으로 태그를 아낍니다.
    println!("    Box<List>: {}바이트", size_of::<Box<List>>()); // 출력: Box<List>: 8바이트
    println!("    List: {}바이트", size_of::<List>()); // 출력: List: 16바이트
    // 리스트가 길든 비어 있든 List 값 하나의 크기는 같습니다. 나머지 칸은 모두 힙에 있습니다.
    let empty = Nil;
    println!(
        "    원소 3개짜리: {}바이트, 빈 리스트: {}바이트",
        size_of_val(&list),
        size_of_val(&empty)
    ); // 출력: 원소 3개짜리: 16바이트, 빈 리스트: 16바이트

    println!("5. Box는 안의 값을 소유하므로, 리스트를 Box에 넣으면 소유권이 이동합니다");
    let longer = Cons(0, Box::new(list)); // list는 이제 longer의 일부입니다.
    println!("    {}", to_text(&longer)); // 출력: 0 -> 1 -> 2 -> 3 -> Nil
    // println!("{list:?}"); // E0382: list는 이미 Box 안으로 이동했습니다.
    // 같은 리스트를 두 리스트가 함께 쓰고 싶다면 Box로는 안 됩니다. 05번 예제의 Rc<T>가 필요합니다.
}
