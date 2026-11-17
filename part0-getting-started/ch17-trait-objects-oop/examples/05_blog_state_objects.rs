// 상태 패턴(state pattern)을 트레이트 객체로 구현한 블로그 글입니다. (The Book 18.3절의 첫 번째 구현)
// - 글은 초안 → 검토 중 → 게시됨 순서로만 바뀌고, 게시되기 전에는 내용이 보이지 않습니다.
// - 상태마다 구조체를 하나씩 두고, 상태별 동작은 State 트레이트의 구현에 나눠 둡니다.
// - Post는 현재 상태를 Option<Box<dyn State>>로 들고 있다가, 상태에게 일을 맡기고 돌려받은 새 상태로 바꿉니다.
// - 상태를 바꾸는 메서드는 self: Box<Self>를 받아 옛 상태를 소비하고, take()로 Option에서 상태를 꺼냅니다.
//
// 실행: cargo run -p p0-ch17-trait-objects-oop --example 05_blog_state_objects

mod blog {
    pub struct Post {
        // 상태는 비공개입니다. 바깥에서는 공개 메서드로만 상태를 바꿀 수 있습니다.
        // Option인 이유는 상태를 바꾸는 동안 잠깐 꺼내 둘 자리가 필요하기 때문입니다(request_review 참고).
        state: Option<Box<dyn State>>,
        content: String,
    }

    impl Post {
        // 새 글은 언제나 초안으로 시작합니다. 다른 상태로 시작할 방법은 없습니다.
        pub fn new() -> Post {
            Post {
                state: Some(Box::new(Draft)),
                content: String::new(),
            }
        }

        // 글을 추가하는 것은 상태와 상관없는 동작이라 Post가 직접 처리합니다.
        // (초안일 때만 허용하는 방법은 연습 문제로 남겨 둡니다.)
        pub fn add_text(&mut self, text: &str) {
            self.content.push_str(text);
        }

        // 내용을 보여 줄지는 현재 상태가 정합니다. 상태에게 Post 자신을 넘겨 물어봅니다.
        pub fn content(&self) -> &str {
            // state는 언제나 Some입니다(아래 메서드들이 take()한 뒤 바로 다시 채우기 때문).
            // as_ref()는 Option<Box<dyn State>>를 빌려 Option<&Box<dyn State>>로 바꿉니다.
            self.state.as_ref().unwrap().content(self)
        }

        pub fn state_name(&self) -> &'static str {
            self.state.as_ref().unwrap().name()
        }

        pub fn request_review(&mut self) {
            // take()는 Option 안의 값을 꺼내고 그 자리에 None을 남깁니다.
            // self는 &mut이라 self.state의 소유권을 그냥 가져갈 수는 없는데(E0507),
            // take()로 꺼내면 state 필드에는 None이 남아 Post는 여전히 온전한 값입니다.
            if let Some(s) = self.state.take() {
                // s.request_review()는 옛 상태를 소비하고 새 상태를 돌려줍니다.
                self.state = Some(s.request_review())
            }
        }

        pub fn approve(&mut self) {
            if let Some(s) = self.state.take() {
                self.state = Some(s.approve())
            }
        }
    }

    // pub이 없으므로 State와 각 상태 구조체는 blog 모듈 밖에서 보이지 않습니다.
    // 상태가 몇 개이고 어떻게 바뀌는지는 Post의 구현 세부입니다(캡슐화).
    trait State {
        // self: Box<Self>는 "Box에 담긴 상태일 때만 부를 수 있고, 그 Box의 소유권을 가져간다"는 뜻입니다.
        // 옛 상태는 이 호출로 소비되어 다시 쓸 수 없고, 메서드가 돌려준 상태가 새 상태가 됩니다.
        fn request_review(self: Box<Self>) -> Box<dyn State>;
        fn approve(self: Box<Self>) -> Box<dyn State>;

        // 기본 구현: 대부분의 상태에서는 내용을 보여 주지 않습니다.
        // 반환값이 post에서 빌린 것이므로 라이프타임 'a로 둘을 이어 줍니다(ch10).
        fn content<'a>(&self, _post: &'a Post) -> &'a str {
            ""
        }

        fn name(&self) -> &'static str;
    }

    struct Draft;

    impl State for Draft {
        fn request_review(self: Box<Self>) -> Box<dyn State> {
            Box::new(PendingReview)
        }

        // 초안은 바로 승인할 수 없습니다. 상태를 바꾸지 않고 자기 자신을 돌려줍니다.
        fn approve(self: Box<Self>) -> Box<dyn State> {
            self
        }

        fn name(&self) -> &'static str {
            "초안"
        }
    }

    struct PendingReview;

    impl State for PendingReview {
        fn request_review(self: Box<Self>) -> Box<dyn State> {
            self // 이미 검토 중입니다.
        }

        fn approve(self: Box<Self>) -> Box<dyn State> {
            Box::new(Published)
        }

        fn name(&self) -> &'static str {
            "검토 중"
        }
    }

    struct Published;

    impl State for Published {
        fn request_review(self: Box<Self>) -> Box<dyn State> {
            self
        }

        fn approve(self: Box<Self>) -> Box<dyn State> {
            self
        }

        // 게시된 상태만 기본 구현을 재정의해 실제 내용을 돌려줍니다.
        fn content<'a>(&self, post: &'a Post) -> &'a str {
            &post.content
        }

        fn name(&self) -> &'static str {
            "게시됨"
        }
    }
}

use blog::Post;

fn main() {
    println!("1. Option::take는 값을 꺼내고 그 자리에 None을 남깁니다");
    let mut slot = Some(String::from("상태"));
    let taken = slot.take();
    println!("    taken = {taken:?}, slot = {slot:?}"); // 출력: taken = Some("상태"), slot = None

    println!("2. 새 글은 초안이고, 초안의 내용은 보이지 않습니다");
    let mut post = Post::new();
    post.add_text("오늘 점심은 샐러드를 먹었다");
    println!(
        "    상태: {}, content() = {:?}",
        post.state_name(),
        post.content()
    ); // 출력: 상태: 초안, content() = ""

    println!("3. 초안을 바로 승인하려 해도 아무 일도 일어나지 않습니다");
    post.approve(); // 컴파일은 되지만, Draft의 approve가 자기 자신을 돌려줄 뿐입니다.
    println!(
        "    상태: {}, content() = {:?}",
        post.state_name(),
        post.content()
    ); // 출력: 상태: 초안, content() = ""

    println!("4. 검토를 요청하면 검토 중이 됩니다");
    post.request_review();
    println!(
        "    상태: {}, content() = {:?}",
        post.state_name(),
        post.content()
    ); // 출력: 상태: 검토 중, content() = ""

    println!("5. 승인되면 게시되고, 이제야 내용이 보입니다");
    post.approve();
    println!(
        "    상태: {}, content() = {:?}",
        post.state_name(),
        post.content()
    ); // 출력: 상태: 게시됨, content() = "오늘 점심은 샐러드를 먹었다"

    println!("6. main은 처음부터 끝까지 같은 변수 post 하나만 다룹니다");
    // post의 타입은 계속 Post입니다. 상태가 바뀌어도 쓰는 쪽의 코드는 같은 메서드를 부를 뿐입니다.
    post.request_review(); // 게시된 글에는 효과가 없습니다.
    println!("    상태: {}", post.state_name()); // 출력: 상태: 게시됨
}
