// 여러 구체 타입의 값을 한 컬렉션에 담는 세 가지 방법을 비교합니다.
// - 한 종류만 담는다면: 제네릭 `Shelf<T>` (값을 `Vec` 버퍼 안에 그대로 둠)
// - 여러 종류를 섞고, 종류가 열려 있다면: `Vec<Box<dyn Loanable>>` (값마다 따로 할당)
// - 여러 종류를 섞지만, 종류가 닫혀 있다면: 열거형 `Media` (값을 버퍼 안에, `match`로 꺼냄)
//
// 실행: cargo run -p er-ch12-generics-vs-trait-objects --example 04_heterogeneous_collection
//
// 크기는 64비트 환경(macOS aarch64, rustc 1.100.0)에서 잰 값입니다. 구조체와 열거형의
// 크기는 컴파일러가 정하는 메모리 배치에 따라 달라질 수 있습니다.

trait Loanable {
    fn title(&self) -> &str;
    fn loan_days(&self) -> u32;
}

struct Book {
    title: String,
}

struct Dvd {
    title: String,
    age_limit: u8,
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

/// 다른 크레이트에서 나중에 만든 타입이라고 생각합니다. 이 모듈은 `Loanable`을 구현하기만 하고,
/// 아래의 카트 코드는 이 타입을 모릅니다.
mod games {
    pub struct BoardGame {
        pub title: String,
    }

    impl super::Loanable for BoardGame {
        fn title(&self) -> &str {
            &self.title
        }
        fn loan_days(&self) -> u32 {
            3
        }
    }
}

// ── 1. 한 종류만: 제네릭 ────────────────────────────────────────────

struct Shelf<T: Loanable> {
    items: Vec<T>,
}

impl<T: Loanable> Shelf<T> {
    fn total_days(&self) -> u32 {
        self.items.iter().map(|item| item.loan_days()).sum()
    }
}

// ── 2. 여러 종류, 열린 집합: 트레이트 객체 ─────────────────────────────

fn describe_cart(cart: &[Box<dyn Loanable>]) -> Vec<String> {
    cart.iter()
        .map(|item| format!("{}({}일)", item.title(), item.loan_days()))
        .collect()
}

// ── 3. 여러 종류, 닫힌 집합: 열거형 ────────────────────────────────────

enum Media {
    Book(Book),
    Dvd(Dvd),
}

impl Media {
    /// 열거형은 `match`로 배리언트 안의 값을 꺼내 그 타입에만 있는 필드를 볼 수 있습니다.
    fn age_limit(&self) -> Option<u8> {
        match self {
            Media::Book(_) => None,
            Media::Dvd(dvd) => Some(dvd.age_limit),
        }
    }
}

/// 열거형도 트레이트를 구현할 수 있습니다. 그러면 `Media`는 구체 타입 하나이므로
/// 제네릭 `Shelf<Media>`에 담아 정적 디스패치로 다룰 수 있습니다.
impl Loanable for Media {
    fn title(&self) -> &str {
        match self {
            Media::Book(book) => book.title(),
            Media::Dvd(dvd) => dvd.title(),
        }
    }
    fn loan_days(&self) -> u32 {
        match self {
            Media::Book(book) => book.loan_days(),
            Media::Dvd(dvd) => dvd.loan_days(),
        }
    }
}

fn book(title: &str) -> Book {
    Book {
        title: title.to_string(),
    }
}

fn dvd(title: &str, age_limit: u8) -> Dvd {
    Dvd {
        title: title.to_string(),
        age_limit,
    }
}

fn main() {
    println!("1. 한 종류만 담는다면: 제네릭 Shelf<Book>");
    let shelf = Shelf {
        items: vec![book("러스트 입문"), book("함께 배우는 러스트")],
    };
    println!(
        "    책 {}권, 대출 기간 합 {}일, 원소 하나의 크기 {}바이트",
        shelf.items.len(),
        shelf.total_days(),
        size_of::<Book>()
    );
    // 출력:     책 2권, 대출 기간 합 28일, 원소 하나의 크기 24바이트

    println!("2. 섞어 담고, 종류가 열려 있다면: Vec<Box<dyn Loanable>>");
    // 반납함에서 꺼낸 순서대로 담습니다. 다른 모듈의 BoardGame도 카트 코드를 고치지 않고 들어갑니다.
    let cart: Vec<Box<dyn Loanable>> = vec![
        Box::new(book("러스트 입문")),
        Box::new(dvd("우주 다큐", 12)),
        Box::new(games::BoardGame {
            title: "보드게임 카탄".to_string(),
        }),
    ];
    println!("    {}", describe_cart(&cart).join(", "));
    // 출력:     러스트 입문(14일), 우주 다큐(7일), 보드게임 카탄(3일)
    println!(
        "    원소 하나의 크기 {}바이트(넓은 포인터), 값은 원소마다 따로 할당한 상자 안에",
        size_of::<Box<dyn Loanable>>()
    );
    // 출력:     원소 하나의 크기 16바이트(넓은 포인터), 값은 원소마다 따로 할당한 상자 안에

    println!("3. 섞어 담고, 종류가 닫혀 있다면: 열거형 Shelf<Media>");
    // `Media`는 구체 타입 하나이므로 1번의 제네릭 `Shelf<T>`를 그대로 씁니다.
    let returns = Shelf {
        items: vec![
            Media::Book(book("러스트 입문")),
            Media::Dvd(dvd("우주 다큐", 12)),
            Media::Dvd(dvd("동물 애니메이션", 0)),
        ],
    };
    for media in &returns.items {
        match media.age_limit() {
            Some(limit) if limit > 0 => {
                println!("    {} - {limit}세 이상", media.title())
            }
            _ => println!("    {}", media.title()),
        }
    }
    // 출력:     러스트 입문
    // 출력:     우주 다큐 - 12세 이상
    // 출력:     동물 애니메이션
    println!(
        "    대출 기간 합 {}일, 원소 하나의 크기 {}바이트(가장 큰 배리언트 기준)",
        returns.total_days(),
        size_of::<Media>()
    );
    // 출력:     대출 기간 합 28일, 원소 하나의 크기 32바이트(가장 큰 배리언트 기준)
}
