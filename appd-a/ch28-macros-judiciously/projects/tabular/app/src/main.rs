// 손으로 쓴 Columns 구현과 #[derive(Columns)]를 비교하고, 선언적 매크로 row!를 씁니다.
//
// 실행(projects/tabular 디렉터리에서): cargo run

// tabular가 트레이트와 derive 매크로를 같은 이름으로 재내보내므로 한 줄로 둘 다 가져옵니다.
use tabular::{Columns, render, row};

// 1. 손으로 쓴 판. 처음에는 제목, 저자, 연도만 있었고 나중에 pages 필드를 더했습니다.
//    cells()에는 pages를 더했지만 NAMES에 더하는 것은 잊었습니다.
//    두 곳이 맞아야 한다는 것은 컴파일러가 모르는 약속이라 에러도 경고도 나지 않습니다.
struct BookByHand {
    title: String,
    author: String,
    year: u16,
    pages: u32,
}

impl Columns for BookByHand {
    const NAMES: &'static [&'static str] = &["제목", "저자", "연도"];

    fn cells(&self) -> Vec<String> {
        vec![
            self.title.clone(),
            self.author.clone(),
            self.year.to_string(),
            self.pages.to_string(),
        ]
    }
}

// 2. derive 판. NAMES와 cells()가 같은 필드 목록에서 만들어지므로 필드를 더하면 둘이 함께 바뀝니다.
//    보조 속성 #[column(..)]은 derive 매크로가 선언한 것이라 이 derive가 붙은 구조체 안에서만 쓸 수 있습니다.
#[derive(Columns)]
struct Book {
    #[column(rename = "제목")]
    title: String,
    #[column(rename = "저자")]
    author: String,
    #[column(rename = "연도")]
    year: u16,
    #[column(rename = "쪽수")]
    pages: u32,
    #[column(skip)]
    shelf: u32, // 서가 번호: 내부용이라 표에서 뺍니다.
}

fn main() {
    let by_hand = vec![
        BookByHand {
            title: "러스트 첫걸음".to_string(),
            author: "한빛나".to_string(),
            year: 2024,
            pages: 412,
        },
        BookByHand {
            title: "소유권 이야기".to_string(),
            author: "도윤서".to_string(),
            year: 2022,
            pages: 288,
        },
    ];
    println!("1. 손으로 쓴 impl: 필드를 더한 뒤 NAMES를 고치지 않음");
    println!("{}", render(&by_hand));

    let mut books = vec![
        Book {
            title: "러스트 첫걸음".to_string(),
            author: "한빛나".to_string(),
            year: 2024,
            pages: 412,
            shelf: 3,
        },
        Book {
            title: "소유권 이야기".to_string(),
            author: "도윤서".to_string(),
            year: 2022,
            pages: 288,
            shelf: 1,
        },
        Book {
            title: "매크로 공방".to_string(),
            author: "서지안".to_string(),
            year: 2025,
            pages: 356,
            shelf: 2,
        },
    ];
    // 표에서 뺀 필드도 코드에서는 그대로 씁니다. 서가 번호 순으로 정렬합니다.
    books.sort_by_key(|b| b.shelf);
    println!();
    println!("2. #[derive(Columns)]");
    println!("{}", render(&books));
    println!("Book::NAMES = {:?}", Book::NAMES);

    let total_pages: u32 = books.iter().map(|b| b.pages).sum();
    let newest = books.iter().map(|b| b.year).max().unwrap_or(0);
    println!();
    println!("3. row!: 식의 글자가 머리글이 됨");
    println!("{}", row!(books.len(), total_pages, newest));
}
