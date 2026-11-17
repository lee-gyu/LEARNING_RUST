// 기대하는 에러: E0277 — the trait bound `Weather: Summary` is not satisfied
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_trait_bound_not_satisfied.rs
//
// notify는 "Summary를 구현한 타입"만 받겠다고 시그니처에 적었습니다(트레이트 바운드).
// Weather는 Summary를 구현하지 않았으므로, 호출하는 곳에서 컴파일러가 거부합니다.
// 해결: Weather에 `impl Summary for Weather { ... }`를 작성하거나, 다른 함수를 씁니다.

trait Summary {
    fn summarize(&self) -> String;
}

struct SocialPost {
    username: String,
    content: String,
}

impl Summary for SocialPost {
    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}

struct Weather {
    city: String,
    celsius: i32,
}

fn notify(item: &impl Summary) {
    println!("속보! {}", item.summarize());
}

fn main() {
    let post = SocialPost {
        username: String::from("ferris"),
        content: String::from("트레이트를 배우는 중"),
    };
    notify(&post); // SocialPost는 Summary를 구현했으므로 괜찮습니다.

    let today = Weather {
        city: String::from("서울"),
        celsius: 21,
    };
    println!("{} {}도", today.city, today.celsius);
    notify(&today); // Weather는 Summary를 구현하지 않았습니다.
}
