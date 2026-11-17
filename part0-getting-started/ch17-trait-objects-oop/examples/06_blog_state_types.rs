// 상태를 타입으로 표현한 블로그 글입니다. (The Book 18.3절의 두 번째 구현)
// - 상태마다 별도의 타입(DraftPost, PendingReviewPost, Post)을 두고, 그 상태에서 할 수 있는 메서드만 붙입니다.
// - 상태를 바꾸는 메서드는 self를 받아 옛 값을 소비하고 다음 상태의 값을 돌려줍니다.
// - 잘못된 순서의 호출(초안의 content, 초안의 approve 등)은 실행 중에 무시되는 대신 컴파일 에러가 됩니다.
//
// 실행: cargo run -p p0-ch17-trait-objects-oop --example 06_blog_state_types

mod blog {
    // 게시된 글입니다. 내용을 읽는 content()는 이 타입에만 있습니다.
    pub struct Post {
        content: String,
    }

    // 초안입니다. 글을 추가하고 검토를 요청할 수 있습니다.
    pub struct DraftPost {
        content: String,
    }

    // 검토 중인 글입니다. 승인만 할 수 있습니다.
    pub struct PendingReviewPost {
        content: String,
    }

    impl Post {
        // "글은 언제나 초안으로 시작한다"는 규칙을 Post::new가 DraftPost를 돌려주는 것으로 표현합니다.
        // Clippy는 new가 Self가 아닌 타입을 돌려주면 경고하지만(new_ret_no_self), 여기서는 의도한 설계입니다.
        #[expect(
            clippy::new_ret_no_self,
            reason = "새 글이 언제나 초안으로 시작한다는 것을 보여 주려는 의도된 설계"
        )]
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

        // self를 값으로 받으므로 이 메서드를 부르면 DraftPost는 소비되어 사라집니다.
        // 내용(String)은 복사 없이 새 값으로 옮겨집니다.
        pub fn request_review(self) -> PendingReviewPost {
            PendingReviewPost {
                content: self.content,
            }
        }
    }

    impl PendingReviewPost {
        // 필드가 비공개라 모듈 밖에서는 Post { .. }로 Post를 만들 수 없습니다.
        // 게시된 Post를 얻는 유일한 길이 이 approve입니다.
        pub fn approve(self) -> Post {
            Post {
                content: self.content,
            }
        }
    }
}

use blog::Post;

// 값의 타입 이름에서 마지막 부분(예: "DraftPost")만 돌려줍니다.
// std::any::type_name은 디버깅용 함수라 정확한 표기는 컴파일러 버전에 따라 바뀔 수 있으므로,
// 프로그램의 동작을 이 문자열에 기대면 안 됩니다. 여기서는 타입이 바뀌는 것을 눈으로 보려고만 씁니다.
fn short_type_name<T>(_value: &T) -> &'static str {
    let full = std::any::type_name::<T>(); // 예: "06_blog_state_types::blog::DraftPost"
    // rsplit은 split과 같지만 뒤에서부터 나눕니다. 첫 조각이 마지막 "::" 뒤의 이름입니다.
    full.rsplit("::").next().unwrap_or(full)
}

// 초안을 쓰고, 검토를 요청하고, 승인받는 과정을 한 함수로 묶었습니다.
// 반환 타입이 Post이므로 이 함수 안에서 단계를 하나라도 빼먹으면 컴파일되지 않습니다.
fn write_and_publish(text: &str) -> Post {
    let mut draft = Post::new();
    draft.add_text(text);
    draft.request_review().approve()
}

fn main() {
    println!("1. Post::new()는 Post가 아니라 DraftPost를 돌려줍니다");
    let mut post = Post::new();
    post.add_text("오늘 점심은 샐러드를 먹었다");
    println!("    post의 타입: {}", short_type_name(&post)); // 출력: post의 타입: DraftPost
    // post.content()는 E0599입니다. DraftPost에는 content 메서드가 없습니다(compile_fail/e0599_draft_content.rs).
    // post.approve()도 E0599입니다. 객체형(05)에서는 조용히 무시되던 호출이 이제 컴파일 에러가 됩니다.

    println!("2. 검토를 요청하면 PendingReviewPost가 됩니다");
    // request_review가 옛 post를 소비하므로, 돌려받은 값을 같은 이름으로 섀도잉해 받습니다.
    let post = post.request_review();
    println!("    post의 타입: {}", short_type_name(&post)); // 출력: post의 타입: PendingReviewPost
    // post.add_text("...")는 E0599입니다. 검토 중인 글에는 add_text가 없습니다.

    println!("3. 승인하면 비로소 content()가 있는 Post가 됩니다");
    let post = post.approve();
    println!("    post의 타입: {}", short_type_name(&post)); // 출력: post의 타입: Post
    println!("    content() = {:?}", post.content()); // 출력: content() = "오늘 점심은 샐러드를 먹었다"

    println!("4. 단계를 거치지 않고는 Post를 얻을 수 없습니다");
    let published = write_and_publish("내일은 비빔밥");
    println!(
        "    {}: {}",
        short_type_name(&published),
        published.content()
    ); // 출력: Post: 내일은 비빔밥
    // 이름을 달리해 옛 값을 다시 쓰려고 해도 막힙니다.
    //     let pending = draft.request_review();
    //     draft.add_text("...");   // E0382: borrow of moved value: `draft`
    // request_review(self)가 draft의 소유권을 가져갔기 때문입니다(ch4의 이동).
}
