// 이 예제가 보여 주는 것: 바운드를 단 impl 블록으로 조건을 만족하는 타입에만 메서드를 주기(조건부 구현),
//                        조건을 만족하는 모든 타입에 트레이트를 한꺼번에 구현하기(포괄 구현, blanket implementation)
// 실행: cargo run -p p0-ch10-generics-traits-lifetimes --example 06_conditional_and_blanket_impl

use std::fmt::Display;

struct Pair<T> {
    x: T,
    y: T,
}

// 바운드 없는 impl: T가 무엇이든 모든 Pair<T>에 new가 있습니다.
impl<T> Pair<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

// 바운드 있는 impl: T가 Display와 PartialOrd를 모두 구현할 때만 Pair<T>에 cmp_display가 생깁니다.
// 본문에서 >=(PartialOrd)와 {}(Display)를 쓰므로 이 두 능력이 꼭 필요합니다.
// 바운드가 다르기 때문에 위의 impl과 블록을 나눌 수밖에 없습니다(ch5에서 미뤄 둔 "impl 블록을 나누는 이유").
impl<T: Display + PartialOrd> Pair<T> {
    fn cmp_display(&self) {
        if self.x >= self.y {
            println!("    더 큰 쪽은 x = {}", self.x);
        } else {
            println!("    더 큰 쪽은 y = {}", self.y);
        }
    }
}

// Display도 PartialOrd도 구현하지 않은 타입입니다.
struct Ticket {
    seat: u32,
}

// 내가 만든 트레이트입니다.
trait Bracketed {
    fn bracketed(&self) -> String;
}

// 포괄 구현: "Display를 구현한 모든 타입 T"에 Bracketed를 구현합니다.
// i32, f64, &str, String은 물론, 앞으로 누군가 Display를 구현할 타입까지 모두 bracketed()를 얻습니다.
// 본문에서 {self}로 출력하므로 T: Display 바운드가 필요합니다.
impl<T: Display> Bracketed for T {
    fn bracketed(&self) -> String {
        format!("[{self}]")
    }
}

// Display를 직접 구현한 타입입니다. 포괄 구현 덕분에 Bracketed를 따로 구현하지 않아도 됩니다.
struct Score(u32);

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}점", self.0)
    }
}

fn main() {
    println!("1. 바운드를 만족하는 Pair에는 cmp_display가 있습니다");
    let numbers = Pair::new(3, 7); // Pair<i32>: i32는 Display + PartialOrd
    numbers.cmp_display();
    let words = Pair::new("사과", "바나나"); // Pair<&str>: &str도 Display + PartialOrd
    words.cmp_display();
    // 출력:
    //     더 큰 쪽은 y = 7
    //     더 큰 쪽은 x = 사과

    println!("2. 바운드를 만족하지 않는 Pair에도 new는 있습니다");
    let tickets = Pair::new(Ticket { seat: 12 }, Ticket { seat: 30 });
    println!("    좌석: {}, {}", tickets.x.seat, tickets.y.seat);
    // 출력: 좌석: 12, 30
    // 아래 줄의 주석을 풀면 E0599가 납니다. 메시지는
    // "the method `cmp_display` exists for struct `Pair<Ticket>`, but its trait bounds were not satisfied"입니다.
    // tickets.cmp_display();

    println!("3. 포괄 구현: Display를 구현한 모든 타입이 bracketed()를 얻습니다");
    println!(
        "    {} {} {} {}",
        42.bracketed(),
        2.5.bracketed(),
        "러스트".bracketed(),
        Score(95).bracketed()
    );
    // 출력: [42] [2.5] [러스트] [95점]

    println!("4. 표준 라이브러리도 포괄 구현을 씁니다: Display가 있으면 to_string()이 생깁니다");
    // 표준 라이브러리 문서의 ToString 구현 목록에는 `impl<T> ToString for T where T: Display + ?Sized`가 있습니다.
    // (?Sized는 str처럼 크기가 정해지지 않은 타입도 포함한다는 뜻이며, ch19에서 다룹니다.)
    // 그래서 Score는 to_string을 직접 구현하지 않았는데도 쓸 수 있습니다. ToString은 프렐류드에 있어 use도 필요 없습니다.
    let a: String = 3.to_string();
    let b: String = Score(80).to_string();
    println!("    {a:?}, {b:?}");
    // 출력: "3", "80점"
}
