// 이 예제가 보여 주는 것: Default(derive, #[default], ..Default::default(), 손으로 구현),
//                        Default를 바운드로 쓰는 제네릭 코드, Clone과 Copy의 차이, clone_from으로 버퍼 재사용
// 실행: cargo run -p p1-ch6-core-traits --example 08_default_clone_copy

use std::collections::HashMap;
use std::rc::Rc;

// ---------- Default ----------
// 모든 필드의 기본값(0, false, 빈 String, 빈 Vec, None)으로 충분하면 derive합니다.
#[derive(Debug, Default)]
struct SearchOptions {
    query: String,
    limit: usize,
    case_sensitive: bool,
    tags: Vec<String>,
    page: Option<u32>,
}

// 필드 타입의 기본값이 이 타입에 맞지 않으면(포트 0, 작업자 0명) 손으로 구현합니다.
#[derive(Debug)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: String::from("127.0.0.1"),
            port: 8080,
            workers: 4,
        }
    }
}

// 열거형은 #[default]로 기본 배리언트를 고릅니다(데이터가 없는 배리언트만 가능).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

// T: Default 바운드가 있으면 "아무 타입이든 빈 값으로 되돌리기"를 쓸 수 있습니다.
fn reset<T: Default>(value: &mut T) {
    *value = T::default();
}

// ---------- Clone / Copy ----------
// 필드가 모두 Copy이고 정리할 자원이 없으므로 Copy가 될 수 있습니다.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Pixel {
    x: u16,
    y: u16,
}

// String과 Vec은 힙 버퍼를 소유하므로 Copy가 될 수 없고, Clone만 derive합니다.
// derive한 clone은 필드마다 clone을 부르므로 힙 데이터까지 새로 복사합니다(깊은 복사).
#[derive(Debug, Clone)]
struct Playlist {
    title: String,
    songs: Vec<String>,
}

fn main() {
    println!("== 1. derive(Default) ==");
    let opts = SearchOptions::default();
    println!("{opts:?}");
    // 출력: SearchOptions { query: "", limit: 0, case_sensitive: false, tags: [], page: None }

    // 구조체 갱신 문법(Part 0 ch5)과 함께 쓰면 "일부만 바꾸고 나머지는 기본값"이 됩니다.
    let custom = SearchOptions {
        query: String::from("러스트"),
        limit: 20,
        ..Default::default()
    };
    println!(
        "{} {} {} {:?} {:?}",
        custom.query, custom.limit, custom.case_sensitive, custom.tags, custom.page
    );
    // 출력: 러스트 20 false [] None

    println!("\n== 2. 손으로 구현한 Default와 #[default] ==");
    let dev = ServerConfig {
        port: 3000,
        ..Default::default()
    };
    println!("{}:{} 작업자 {}명", dev.host, dev.port, dev.workers); // 출력: 127.0.0.1:3000 작업자 4명
    let theme = Theme::default();
    println!("{theme:?} {}", [Theme::Light, Theme::Dark].contains(&theme)); // 출력: System false

    println!("\n== 3. Default를 쓰는 표준 API ==");
    let mut draft = String::from("임시 저장 글");
    let mut counts = vec![3, 1, 4];
    reset(&mut draft);
    reset(&mut counts);
    println!("{draft:?} {counts:?}"); // 출력: "" []
    let parsed: i32 = "abc".parse().unwrap_or_default(); // 실패하면 i32::default() = 0
    println!("{parsed}"); // 출력: 0
    let mut by_team: HashMap<&str, Vec<&str>> = HashMap::new();
    by_team.entry("청팀").or_default().push("민수"); // 없으면 Vec::default()를 넣고 push
    println!("{:?}", by_team.get("청팀")); // 출력: Some(["민수"])

    println!("\n== 4. Copy: 대입하면 비트가 복사되고 원본도 그대로 씁니다 ==");
    let a = Pixel { x: 3, y: 4 };
    let mut b = a; // 이동이 아니라 복사
    b.x = 100;
    println!("{a:?} {b:?}"); // 출력: Pixel { x: 3, y: 4 } Pixel { x: 100, y: 4 }

    println!("\n== 5. Clone: 명시적으로 부르고, 무엇을 복사할지는 타입이 정합니다 ==");
    let original = Playlist {
        title: String::from("출근길"),
        songs: vec![String::from("노래1"), String::from("노래2")],
    };
    let mut copy = original.clone(); // String과 Vec의 힙 데이터까지 새로 만듭니다.
    copy.title.push_str(" (편집)");
    copy.songs.push(String::from("노래3"));
    println!("{} {:?}", original.title, original.songs); // 출력: 출근길 ["노래1", "노래2"]
    println!("{} {:?}", copy.title, copy.songs); // 출력: 출근길 (편집) ["노래1", "노래2", "노래3"]

    // Rc의 clone은 데이터를 복사하지 않고 참조 카운트만 늘립니다(Part 0 ch14).
    let shared = Rc::new(original);
    let another = Rc::clone(&shared);
    println!(
        "같은 데이터? {}, 강한 참조 {}개",
        Rc::ptr_eq(&shared, &another),
        Rc::strong_count(&shared)
    );
    // 출력: 같은 데이터? true, 강한 참조 2개

    println!("\n== 6. clone_from: 이미 있는 버퍼를 다시 씁니다 ==");
    let lines = [
        String::from("첫 줄"),
        String::from("둘째 줄입니다"),
        String::from("셋째"),
    ];
    let mut buf = String::with_capacity(64);
    let start = buf.as_ptr(); // 버퍼 주소(값은 출력하지 않고 비교만 합니다)
    let mut reused = true;
    for line in &lines {
        buf.clone_from(line); // buf = line.clone()과 결과는 같지만, 용량이 충분하면 새로 할당하지 않습니다.
        reused &= buf.as_ptr() == start;
    }
    println!("{buf} / 같은 버퍼를 계속 썼나? {reused}"); // 출력: 셋째 / 같은 버퍼를 계속 썼나? true

    let mut fresh = String::with_capacity(64);
    let before = fresh.as_ptr();
    fresh = lines[0].clone(); // 새 String을 만든 뒤 대입하므로 원래 버퍼는 버려집니다.
    println!("{fresh} / 새 버퍼인가? {}", fresh.as_ptr() != before); // 출력: 첫 줄 / 새 버퍼인가? true
}
