// 같은 함수의 제네릭판과 `dyn`판을 나란히 놓고, 무엇이 달라지는지 봅니다.
// 제네릭판은 쓰인 구체 타입마다 본문이 한 벌씩 만들어지고(단형성화),
// `dyn`판은 본문이 한 벌이며 실행 중에 vtable을 거쳐 메서드를 찾습니다(동적 디스패치).
//
// 실행: cargo run -p er-ch12-generics-vs-trait-objects --example 01_generic_vs_dyn
//
// 주의: `type_name`이 돌려주는 문자열은 진단용이라 형식이 보장되지 않고,
// 툴체인이 바뀌면 달라질 수 있습니다. 아래 출력은 rustc 1.100.0에서 확인한 것입니다.

use std::any::{type_name, type_name_of_val};

/// 도서관에서 빌려줄 수 있는 자료.
trait Loanable {
    fn title(&self) -> &str;
    fn loan_days(&self) -> u32;
}

struct Book {
    title: String,
}

struct Dvd {
    title: String,
}

impl Loanable for Book {
    fn title(&self) -> &str {
        &self.title
    }
    fn loan_days(&self) -> u32 {
        14
    }
}

impl Loanable for Dvd {
    fn title(&self) -> &str {
        &self.title
    }
    fn loan_days(&self) -> u32 {
        7
    }
}

/// 제네릭판: `due_note::<Book>`과 `due_note::<Dvd>`가 따로 컴파일됩니다.
/// 각 사본 안에서 `T`는 이미 구체 타입이므로 `type_name::<T>()`가 그 타입을 알려 줍니다.
fn due_note<T: Loanable>(item: &T) -> String {
    format!(
        "{} → {}일 뒤 반납 (본문이 아는 타입: {})",
        item.title(),
        item.loan_days(),
        short(type_name::<T>())
    )
}

/// `dyn`판: 본문은 한 벌뿐입니다. 본문이 아는 것은 "`Loanable`을 구현한 무엇"뿐이고,
/// `item.loan_days()`는 넓은 포인터에 붙은 vtable에서 함수 주소를 읽어 부릅니다.
fn due_note_dyn(item: &dyn Loanable) -> String {
    format!(
        "{} → {}일 뒤 반납 (본문이 아는 타입: {})",
        item.title(),
        item.loan_days(),
        short(type_name_of_val(item))
    )
}

/// 타입 이름에서 이 예제 크레이트의 경로(`01_generic_vs_dyn::`)를 지워 읽기 쉽게 합니다.
fn short(name: &str) -> String {
    name.replace(concat!(module_path!(), "::"), "")
}

fn main() {
    let book = Book {
        title: "러스트 입문".to_string(),
    };
    let dvd = Dvd {
        title: "우주 다큐".to_string(),
    };

    println!("1. 결과는 같습니다");
    println!("    제네릭판: {}", due_note(&book));
    // 출력:     제네릭판: 러스트 입문 → 14일 뒤 반납 (본문이 아는 타입: Book)
    println!("    제네릭판: {}", due_note(&dvd));
    // 출력:     제네릭판: 우주 다큐 → 7일 뒤 반납 (본문이 아는 타입: Dvd)
    println!("    dyn판:    {}", due_note_dyn(&book));
    // 출력:     dyn판:    러스트 입문 → 14일 뒤 반납 (본문이 아는 타입: dyn Loanable)
    println!("    dyn판:    {}", due_note_dyn(&dvd));
    // 출력:     dyn판:    우주 다큐 → 7일 뒤 반납 (본문이 아는 타입: dyn Loanable)

    // 함수 아이템의 타입 이름으로 "어떤 함수가 만들어졌는가"를 봅니다.
    // 제네릭판은 타입 인자가 이름에 들어간 별개의 함수 둘, dyn판은 하나입니다.
    println!("2. 만들어진 함수");
    println!(
        "    제네릭판: {}, {}",
        short(type_name_of_val(&due_note::<Book>)),
        short(type_name_of_val(&due_note::<Dvd>))
    );
    // 출력:     제네릭판: due_note<Book>, due_note<Dvd>
    println!("    dyn판:    {}", short(type_name_of_val(&due_note_dyn)));
    // 출력:     dyn판:    due_note_dyn

    // 넘기는 인자도 다릅니다. 제네릭판은 값의 주소만, dyn판은 주소와 vtable 주소를 넘깁니다.
    println!("3. 넘기는 인자의 크기 (64비트 기준)");
    println!(
        "    제네릭판 &Book: {}바이트, dyn판 &dyn Loanable: {}바이트",
        size_of::<&Book>(),
        size_of::<&dyn Loanable>()
    );
    // 출력:     제네릭판 &Book: 8바이트, dyn판 &dyn Loanable: 16바이트
}
