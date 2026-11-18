// 이 예제가 보여 주는 것: OnceCell<T> — 한 번만 채우는 칸, &self 메서드에서 계산 결과를 캐시해 &T로 빌려 주기,
//                        LazyCell<T, F> — 처음 쓸 때 초기화 함수를 부르는 값
// 실행: cargo run -p p1-ch7-smart-pointers-memory --example 08_once_lazy
//
// OnceCell은 한 번 채운 값을 다시 바꾸지 않습니다(&mut가 있을 때만 예외). 그래서 안의 값을 가리키는 &T를
// 마음껏 빌려 줘도 그 값이 바뀌거나 사라질 걱정이 없고, RefCell처럼 대여를 세지 않아도 됩니다.

use std::cell::{LazyCell, OnceCell};

struct Article {
    body: String,
    summary: OnceCell<String>,
}

impl Article {
    fn new(body: &str) -> Self {
        Article {
            body: body.to_string(),
            summary: OnceCell::new(),
        }
    }

    // &self를 받아 &str을 돌려줍니다. 처음 부를 때만 요약을 만들고, 이후에는 만들어 둔 값을 빌려 줍니다.
    // RefCell<Option<String>>이었다면 Ref 가드 때문에 &str을 바로 돌려줄 수 없습니다(07 예제의 Ref::map 참고).
    fn summary(&self) -> &str {
        self.summary.get_or_init(|| {
            println!("  (요약을 만듭니다)");
            let first: String = self.body.chars().take(8).collect();
            format!("{first}…")
        })
    }

    // 본문을 바꾸면 요약도 버려야 합니다. OnceCell::take는 &mut self가 필요합니다.
    // &mut self를 받았다는 것은 summary()가 빌려 준 &str이 하나도 남아 있지 않다는 뜻이므로 안전합니다.
    fn set_body(&mut self, body: &str) {
        self.body = body.to_string();
        self.summary.take();
    }
}

fn main() {
    println!("== 1. OnceCell: get / set ==");
    let winner: OnceCell<&str> = OnceCell::new();
    println!("처음 get: {:?}", winner.get()); // 출력: 처음 get: None
    println!("첫 set: {:?}", winner.set("민수")); // 출력: 첫 set: Ok(())
    // 이미 채워졌으면 넣으려던 값을 Err에 담아 돌려줍니다. 안의 값은 그대로입니다.
    println!("둘째 set: {:?}", winner.set("지영")); // 출력: 둘째 set: Err("지영")
    println!("get: {:?}", winner.get()); // 출력: get: Some("민수")

    println!("\n== 2. get_or_init: &self 메서드에서 한 번만 계산하기 ==");
    let article = Article::new("러스트의 스마트 포인터와 메모리 이야기");
    let s1 = article.summary();
    // 출력:   (요약을 만듭니다)
    let s2 = article.summary(); // 두 번째부터는 클로저를 부르지 않습니다.
    println!("s1 = {s1}, s2 = {s2}"); // 출력: s1 = 러스트의 스마트…, s2 = 러스트의 스마트…
    // s1과 s2는 같은 String을 가리키는 &str 두 개입니다. 둘을 동시에 쥐고 있어도 검사할 것이 없습니다.
    println!("같은 곳을 가리키는가? {}", std::ptr::eq(s1, s2)); // 출력: 같은 곳을 가리키는가? true

    println!("\n== 3. &mut가 있으면 비우고 다시 채울 수 있습니다 ==");
    let mut article = article;
    article.set_body("Cell과 RefCell 고르기");
    println!("새 요약 = {}", article.summary());
    // 출력:
    //   (요약을 만듭니다)
    // 새 요약 = Cell과 Re…
    // 앞 8글자만 잘라 요약으로 씁니다. 공백도 한 글자로 셉니다.

    println!("\n== 4. LazyCell: 처음 쓸 때 초기화 ==");
    // LazyCell은 OnceCell과 초기화 함수를 한데 묶은 것입니다. 역참조(*, 메서드 호출)할 때 함수가 불립니다.
    let settings = LazyCell::new(|| {
        println!("  (설정을 읽습니다)");
        vec![("theme", "dark"), ("lang", "ko")]
    });
    println!("아직 초기화 전: {:?}", LazyCell::get(&settings)); // 출력: 아직 초기화 전: None
    println!("항목 수 = {}", settings.len()); // Vec의 len을 부르려고 역참조하는 순간 초기화됩니다.
    // 출력:
    //   (설정을 읽습니다)
    // 항목 수 = 2
    println!("첫 항목 = {:?}", settings[0]); // 출력: 첫 항목 = ("theme", "dark")
    println!(
        "초기화 뒤 값이 있는가? {}",
        LazyCell::get(&settings).is_some()
    ); // 출력: 초기화 뒤 값이 있는가? true
    // LazyCell::get(1.94.0)은 초기화하지 않고 들여다보기만 합니다.
    // 메서드가 아니라 연관 함수로 부르는 것은 Rc::strong_count처럼 안의 타입의 메서드와 이름이 겹치지 않게 하려는 것입니다.
}
