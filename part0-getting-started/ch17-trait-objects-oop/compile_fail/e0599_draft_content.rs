// 기대하는 에러: E0599 — no method named `content` found for struct `DraftPost` in the current scope
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0599_draft_content.rs
//
// 상태를 타입으로 표현한 설계에서는 초안(DraftPost)에 content 메서드가 아예 없습니다.
// 객체형 상태 패턴(examples/05_blog_state_objects.rs)이라면 빈 문자열이 나왔을 호출이
// 여기서는 컴파일 에러가 됩니다. "게시 전에는 내용을 볼 수 없다"는 규칙을 컴파일러가 검사하는 것입니다.
// 해결: request_review()와 approve()를 거쳐 Post를 얻은 뒤 content()를 부릅니다.

mod blog {
    pub struct Post {
        content: String,
    }

    pub struct DraftPost {
        content: String,
    }

    pub struct PendingReviewPost {
        content: String,
    }

    impl Post {
        pub fn new() -> DraftPost {
            DraftPost {
                content: String::new(),
            }
        }

        pub fn content(&self) -> &str {
            &self.content
        }
    }

    impl DraftPost {
        pub fn add_text(&mut self, text: &str) {
            self.content.push_str(text);
        }

        pub fn request_review(self) -> PendingReviewPost {
            PendingReviewPost {
                content: self.content,
            }
        }
    }

    impl PendingReviewPost {
        pub fn approve(self) -> Post {
            Post {
                content: self.content,
            }
        }
    }
}

use blog::Post;

fn main() {
    let mut post = Post::new();
    post.add_text("오늘 점심은 샐러드를 먹었다");
    println!("{}", post.content()); // post는 아직 DraftPost입니다.
}
