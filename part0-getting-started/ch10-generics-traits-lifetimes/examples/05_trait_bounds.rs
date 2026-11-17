// 이 예제가 보여 주는 것: 매개변수의 impl Trait, 트레이트 바운드 문법 T: Trait, +로 여러 바운드, where 절,
//                        반환 위치의 impl Trait
// 실행: cargo run -p p0-ch10-generics-traits-lifetimes --example 05_trait_bounds

use std::fmt::{self, Debug, Display};

trait Summary {
    fn summarize(&self) -> String;
}

#[derive(Debug)]
struct NewsArticle {
    headline: String,
    author: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{} - {}", self.headline, self.author)
    }
}

#[derive(Debug)]
struct SocialPost {
    username: String,
    content: String,
}

impl Summary for SocialPost {
    fn summarize(&self) -> String {
        format!("@{}: {}", self.username, self.content)
    }
}

// SocialPost에만 Display도 구현합니다. 그래서 SocialPost는 Summary + Display를, NewsArticle은 Summary만 만족합니다.
impl Display for SocialPost {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {}", self.username, self.content)
    }
}

// (1) impl Trait 문법: "Summary를 구현한 어떤 타입의 참조든 받는다"
// 함수 안에서는 Summary의 메서드만 쓸 수 있습니다. 어떤 타입이 올지 모르니 필드에는 접근할 수 없습니다.
fn notify(item: &impl Summary) {
    println!("    속보! {}", item.summarize());
}

// (2) 트레이트 바운드 문법: (1)과 같은 뜻을 길게 쓴 것입니다. (1)은 사실 이 형태의 줄임말입니다.
fn notify_bound<T: Summary>(item: &T) {
    println!("    속보! {}", item.summarize());
}

// (3) impl Trait를 두 번 쓰면 두 인자가 서로 다른 타입이어도 됩니다.
fn notify_two(a: &impl Summary, b: &impl Summary) {
    println!("    1) {}", a.summarize());
    println!("    2) {}", b.summarize());
}

// (4) 같은 T를 두 번 쓰면 두 인자는 반드시 같은 타입이어야 합니다. 이런 제약은 바운드 문법으로만 쓸 수 있습니다.
fn notify_same<T: Summary>(a: &T, b: &T) {
    println!("    같은 타입 둘: {} / {}", a.summarize(), b.summarize());
}

// (5) +로 바운드를 여러 개 겁니다. item은 Summary와 Display를 모두 구현해야 합니다.
// impl Trait 문법으로는 `item: &(impl Summary + Display)`라고 씁니다.
fn notify_and_print<T: Summary + Display>(item: &T) {
    println!("    요약: {}", item.summarize()); // Summary가 있어서 부를 수 있습니다.
    println!("    표시: {item}"); // Display가 있어서 {}로 출력할 수 있습니다.
}

// (6) where 절: 바운드가 길어지면 시그니처 뒤로 옮겨 함수 이름과 매개변수를 읽기 쉽게 합니다.
// `fn compare<T: Summary + Display, U: Summary + Debug>(t: &T, u: &U) -> String`과 같은 뜻입니다.
fn compare<T, U>(t: &T, u: &U) -> String
where
    T: Summary + Display,
    U: Summary + Debug,
{
    format!("{t} vs {u:?}")
}

// (7) 반환 위치의 impl Trait: "Summary를 구현한 어떤 타입 하나를 돌려준다"
// 호출한 쪽은 구체 타입이 SocialPost라는 사실을 모르고, Summary의 메서드만 쓸 수 있습니다.
// 갈래마다 다른 타입을 돌려줄 수는 없습니다(compile_fail/e0308_impl_trait_two_types.rs).
fn make_reply(to: &str) -> impl Summary {
    SocialPost {
        username: String::from("bot"),
        content: format!("{to}님, 글 잘 봤습니다"),
    }
}

fn main() {
    let article = NewsArticle {
        headline: String::from("러스트 1.98 출시"),
        author: String::from("김기자"),
    };
    let post = SocialPost {
        username: String::from("ferris"),
        content: String::from("바운드를 배우는 중"),
    };
    let post2 = SocialPost {
        username: String::from("crab"),
        content: String::from("저도요"),
    };

    println!("1. impl Trait 매개변수와 트레이트 바운드는 같은 뜻입니다");
    notify(&article);
    notify_bound(&post);
    // 출력:
    //     속보! 러스트 1.98 출시 - 김기자
    //     속보! @ferris: 바운드를 배우는 중
    // notify(&42)처럼 Summary를 구현하지 않은 값을 넘기면 E0277입니다(compile_fail/e0277_trait_bound_not_satisfied.rs).

    println!("2. impl Trait 두 개는 서로 다른 타입을 받습니다");
    notify_two(&article, &post);
    // 출력:
    //     1) 러스트 1.98 출시 - 김기자
    //     2) @ferris: 바운드를 배우는 중

    println!("3. 같은 T를 쓰면 같은 타입만 받습니다");
    notify_same(&post, &post2);
    // 출력: 같은 타입 둘: @ferris: 바운드를 배우는 중 / @crab: 저도요
    // notify_same(&article, &post)는 E0308(expected `&NewsArticle`, found `&SocialPost`)로 거부됩니다.

    println!("4. +로 건 바운드는 모두 만족해야 합니다");
    notify_and_print(&post);
    // 출력:
    //     요약: @ferris: 바운드를 배우는 중
    //     표시: [ferris] 바운드를 배우는 중
    // NewsArticle은 Display가 없으므로 notify_and_print(&article)는 E0277입니다.

    println!("5. where 절로 쓴 바운드");
    println!("    {}", compare(&post2, &article));
    // 출력: [crab] 저도요 vs NewsArticle { headline: "러스트 1.98 출시", author: "김기자" }

    println!("6. 반환 위치의 impl Trait");
    let reply = make_reply(&post.username);
    println!("    {}", reply.summarize());
    // 출력: @bot: ferris님, 글 잘 봤습니다
    // reply의 타입은 호출한 쪽에서 `impl Summary`로만 보입니다. 그래서 reply.username처럼 필드에 접근하면
    // E0609(no field `username` on type `impl Summary`)가 납니다.
}
