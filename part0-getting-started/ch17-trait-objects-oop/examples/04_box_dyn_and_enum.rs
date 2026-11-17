// 트레이트 객체가 쓰이는 대표적인 자리와, 같은 문제를 열거형으로 푸는 방법을 비교합니다.
// - 반환 위치: ch10에서 E0308로 막혔던 "갈래마다 다른 타입 반환"을 Box<dyn Summary>로 풉니다.
// - Box<dyn Error>: ch9부터 써 온 이 타입도 "Error를 구현한 어떤 타입의 값"을 담은 트레이트 객체입니다.
// - 열거형과 트레이트 객체: ch8의 Cell 열거형과 같은 일을 트레이트 객체로 하고, 무엇이 다른지 봅니다.
//
// 실행: cargo run -p p0-ch17-trait-objects-oop --example 04_box_dyn_and_enum

use std::error::Error;
use std::fmt;

// ---------------------------------------------------------------------------
// 1. 반환 위치의 트레이트 객체
// ---------------------------------------------------------------------------

trait Summary {
    fn summarize(&self) -> String;
}

struct NewsArticle {
    headline: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("[기사] {}", self.headline)
    }
}

struct SocialPost {
    username: String,
}

impl Summary for SocialPost {
    fn summarize(&self) -> String {
        format!("[글] @{}", self.username)
    }
}

// ch10의 `-> impl Summary`는 "구체 타입 하나"를 숨긴 것이라 두 타입을 섞어 돌려줄 수 없었습니다.
// `-> Box<dyn Summary>`는 "Summary를 구현한 어떤 타입의 값을 담은 Box"라는 한 가지 타입이므로,
// 두 갈래가 각각 Box<NewsArticle>, Box<SocialPost>를 만들어도 모두 Box<dyn Summary>로 바뀝니다.
fn returns_summarizable(switch: bool) -> Box<dyn Summary> {
    if switch {
        Box::new(NewsArticle {
            headline: String::from("러스트 2024 에디션 공개"),
        })
    } else {
        Box::new(SocialPost {
            username: String::from("ferris"),
        })
    }
}

// ---------------------------------------------------------------------------
// 2. Box<dyn Error>
// ---------------------------------------------------------------------------

// 직접 만든 에러 타입입니다. Error 트레이트는 Debug와 Display를 먼저 구현한 타입에만
// 구현할 수 있습니다(이런 관계를 슈퍼트레이트라고 하며 ch19에서 다룹니다).
#[derive(Debug)]
struct ReservedPort(u16);

impl fmt::Display for ReservedPort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}번 포트는 시스템용으로 예약되어 있습니다", self.0)
    }
}

// Error의 메서드는 모두 기본 구현이 있으므로 빈 블록으로 충분합니다.
impl Error for ReservedPort {}

// 실패하는 경우가 두 종류입니다. 숫자가 아니면 ParseIntError, 1024보다 작으면 ReservedPort.
// 에러 타입이 Box<dyn Error>이므로 둘 다 같은 반환 타입에 담을 수 있습니다.
fn parse_port(text: &str) -> Result<u16, Box<dyn Error>> {
    // `?`는 ParseIntError를 Box에 담아 Box<dyn Error>로 바꾼 뒤 돌려줍니다.
    let port: u16 = text.trim().parse()?;
    if port < 1024 {
        // 직접 만든 에러는 Box::new로 감싸면 Box<dyn Error>가 됩니다. Box<dyn Draw>와 같은 원리입니다.
        return Err(Box::new(ReservedPort(port)));
    }
    Ok(port)
}

// ---------------------------------------------------------------------------
// 3. 열거형과 트레이트 객체
// ---------------------------------------------------------------------------

// (가) ch8의 방식: 담을 수 있는 종류를 배리언트로 모두 나열합니다.
enum Cell {
    Int(i32),
    Float(f64),
    Text(String),
}

impl Cell {
    // 동작마다 match 하나가 필요하고, match는 모든 배리언트를 다뤄야 합니다.
    fn render(&self) -> String {
        match self {
            Cell::Int(n) => n.to_string(),
            Cell::Float(x) => format!("{x:.1}"),
            Cell::Text(s) => s.clone(),
        }
    }

    fn number(&self) -> Option<f64> {
        match self {
            Cell::Int(n) => Some(f64::from(*n)),
            Cell::Float(x) => Some(*x),
            Cell::Text(_) => None,
        }
    }
}

// (나) 트레이트 객체 방식: 칸이 갖춰야 할 동작만 트레이트로 정합니다.
trait CellValue {
    fn render(&self) -> String;

    // 숫자가 아닌 칸이 더 흔하다고 보고 기본 구현을 둡니다.
    fn number(&self) -> Option<f64> {
        None
    }
}

// 표준 라이브러리 타입에 내 트레이트를 구현하는 것은 고아 규칙(ch10)이 허용합니다.
impl CellValue for i32 {
    fn render(&self) -> String {
        self.to_string()
    }

    fn number(&self) -> Option<f64> {
        Some(f64::from(*self))
    }
}

impl CellValue for f64 {
    fn render(&self) -> String {
        format!("{self:.1}")
    }

    fn number(&self) -> Option<f64> {
        Some(*self)
    }
}

impl CellValue for String {
    fn render(&self) -> String {
        self.clone()
    }
}

// 나중에 새 종류의 칸이 필요해졌습니다. 위의 코드는 한 줄도 고치지 않고 구현만 추가합니다.
// 열거형 방식이었다면 Cell에 배리언트를 추가하고, render와 number의 match를 모두 고쳐야 합니다.
impl CellValue for bool {
    fn render(&self) -> String {
        if *self { "예" } else { "아니요" }.to_string()
    }
}

fn main() {
    println!("1. 반환 타입 Box<dyn Summary>: 갈래마다 다른 타입을 돌려줄 수 있습니다");
    for switch in [true, false] {
        let item = returns_summarizable(switch);
        println!("    switch = {switch}: {}", item.summarize());
    }
    // 출력:
    //     switch = true: [기사] 러스트 2024 에디션 공개
    //     switch = false: [글] @ferris

    println!("2. Box<dyn Error>에는 Error를 구현한 어떤 에러든 담깁니다");
    for input in ["8080", "abc", "70000", "80"] {
        match parse_port(input) {
            Ok(port) => println!("    {input:>5} → 포트 {port}"),
            // {e}는 Box 안에 든 실제 에러 타입의 Display 구현을 부릅니다(동적 디스패치).
            Err(e) => println!("    {input:>5} → 에러: {e}"),
        }
    }
    // 출력:
    //      8080 → 포트 8080
    //       abc → 에러: invalid digit found in string
    //     70000 → 에러: number too large to fit in target type
    //        80 → 에러: 80번 포트는 시스템용으로 예약되어 있습니다
    println!(
        "    Box<dyn Error>의 크기: {}바이트",
        size_of::<Box<dyn Error>>()
    ); // 출력: Box<dyn Error>의 크기: 16바이트

    println!("3. 같은 행을 열거형과 트레이트 객체로 담아 봅니다");
    let enum_row = [
        Cell::Text(String::from("사과")),
        Cell::Int(3),
        Cell::Float(1200.5),
    ];
    let enum_text: Vec<String> = enum_row.iter().map(|cell| cell.render()).collect();
    let enum_sum: f64 = enum_row.iter().filter_map(|cell| cell.number()).sum();
    println!("    열거형:        {:?}, 숫자 합계 {enum_sum}", enum_text);
    // 출력: 열거형:        ["사과", "3", "1200.5"], 숫자 합계 1203.5

    // Box::new(3)은 Box<i32>, Box::new(1200.5)는 Box<f64>이고, 배열의 타입 표기를 보고
    // 모두 Box<dyn CellValue>로 바뀝니다.
    let dyn_row: [Box<dyn CellValue>; 4] = [
        Box::new(String::from("사과")),
        Box::new(3),
        Box::new(1200.5),
        Box::new(true), // 열거형에는 없던 종류도 담을 수 있습니다.
    ];
    let dyn_text: Vec<String> = dyn_row.iter().map(|cell| cell.render()).collect();
    let dyn_sum: f64 = dyn_row.iter().filter_map(|cell| cell.number()).sum();
    println!("    트레이트 객체: {:?}, 숫자 합계 {dyn_sum}", dyn_text);
    // 출력: 트레이트 객체: ["사과", "3", "1200.5", "예"], 숫자 합계 1203.5

    // 열거형 값은 배열 안에 그대로 들어갑니다. 어느 배리언트인지 구분하는 정보까지 이 크기 안에 있습니다
    // (이 Cell은 가장 큰 배리언트인 String과 크기가 같습니다. 그 원리는 Part 1 ch7-smart-pointers-memory에서 다룹니다).
    // 트레이트 객체는 값마다 힙에 Box를 하나씩 만들고, 배열에는 두 칸짜리 포인터만 둡니다.
    println!(
        "    원소 하나의 크기: Cell {}바이트, Box<dyn CellValue> {}바이트",
        size_of::<Cell>(),
        size_of::<Box<dyn CellValue>>()
    ); // 출력: 원소 하나의 크기: Cell 24바이트, Box<dyn CellValue> 16바이트
}
