// 기대하는 에러: E0308 — `if` and `else` have incompatible types
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0308_impl_trait_two_types.rs
//
// 반환 위치의 `impl Summary`는 "Summary를 구현한 어떤 타입 하나"라는 뜻입니다.
// 이름만 숨겼을 뿐, 컴파일러는 이 함수가 돌려주는 구체 타입을 정확히 하나로 정해야 합니다.
// 갈래에 따라 NewsArticle과 SocialPost를 섞어 돌려주면 그 하나를 정할 수 없습니다.
// 해결: 한 가지 타입만 돌려주도록 설계를 바꿉니다. 여러 타입을 섞어 돌려주는 방법(트레이트 객체)은
// ch17-trait-objects-oop에서 배웁니다.

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

fn returns_summarizable(switch: bool) -> impl Summary {
    if switch {
        NewsArticle {
            headline: String::from("러스트 2024 에디션 공개"),
        }
    } else {
        SocialPost {
            username: String::from("ferris"),
        }
    }
}

fn main() {
    println!("{}", returns_summarizable(true).summarize());
}
