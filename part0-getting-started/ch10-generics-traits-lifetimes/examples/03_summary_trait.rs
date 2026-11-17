// 이 예제가 보여 주는 것: 트레이트 정의와 구현, 기본 구현과 재정의, 트레이트를 use로 가져와야 메서드를 쓸 수 있다는 점,
//                        내 트레이트를 표준 라이브러리 타입(Vec)에 구현하기(고아 규칙이 허용하는 경우)
// 실행: cargo run -p p0-ch10-generics-traits-lifetimes --example 03_summary_trait

// 뉴스 기사와 SNS 글을 모아 요약해 보여 주는 라이브러리를 모듈 하나로 흉내 냅니다(모듈은 ch7).
mod aggregator {
    // 트레이트(trait)는 "이 타입이 할 수 있는 일"을 메서드 시그니처의 목록으로 정의한 것입니다.
    pub trait Summary {
        // 본문 없이 세미콜론으로 끝나는 메서드는 구현하는 타입이 반드시 직접 작성해야 합니다.
        fn summarize_author(&self) -> String;

        // 본문이 있는 메서드는 기본 구현(default implementation)입니다.
        // 구현하는 타입이 따로 작성하지 않으면 이 본문이 쓰입니다.
        // 기본 구현은 같은 트레이트의 다른 메서드(아직 본문이 없는 summarize_author도)를 부를 수 있습니다.
        fn summarize(&self) -> String {
            format!("({}님의 글 더 보기...)", self.summarize_author())
        }
    }

    pub struct NewsArticle {
        pub headline: String,
        pub location: String,
        pub author: String,
    }

    // `impl 트레이트 for 타입`: NewsArticle에 Summary를 구현합니다.
    impl Summary for NewsArticle {
        fn summarize_author(&self) -> String {
            self.author.clone()
        }

        // 기본 구현이 있는 메서드도 같은 시그니처로 다시 쓰면 재정의(override)됩니다.
        fn summarize(&self) -> String {
            format!("{}, {} ({})", self.headline, self.author, self.location)
        }
    }

    pub struct SocialPost {
        pub username: String,
        pub content: String,
    }

    // SocialPost는 반드시 써야 하는 summarize_author만 구현하고, summarize는 기본 구현을 그대로 씁니다.
    impl Summary for SocialPost {
        fn summarize_author(&self) -> String {
            format!("@{}", self.username)
        }
    }

    // 트레이트와 타입 중 적어도 하나가 내 크레이트의 것이어야 impl을 쓸 수 있습니다(고아 규칙).
    // Vec은 표준 라이브러리의 타입이지만 Summary는 이 크레이트의 트레이트이므로 허용됩니다.
    // 반대로 표준 라이브러리의 트레이트(Display)를 표준 라이브러리의 타입(Vec)에 구현하는 것은 E0117로 거부됩니다.
    impl Summary for Vec<SocialPost> {
        fn summarize_author(&self) -> String {
            format!("{}명", self.len())
        }

        fn summarize(&self) -> String {
            format!("글 {}개가 이어진 스레드", self.len())
        }
    }
}

// 타입뿐 아니라 트레이트(Summary)도 가져와야 합니다. 트레이트의 메서드는 그 트레이트가 스코프에 있을 때만
// 점(.)으로 부를 수 있기 때문입니다. ch2에서 rand의 RngExt를 가져와야 random_range를 쓸 수 있던 것과 같습니다.
// `Summary`를 지우면 E0599(no method named `summarize` found ...)와 함께
// "trait `Summary` which provides `summarize` is implemented but not in scope"라는 도움말이 나옵니다.
use aggregator::{NewsArticle, SocialPost, Summary};

fn main() {
    let article = NewsArticle {
        headline: String::from("페리스, 러스트 마스코트 10주년"),
        location: String::from("서울"),
        author: String::from("김기자"),
    };
    let post = SocialPost {
        username: String::from("ferris"),
        content: String::from("오늘은 트레이트를 배웠다"),
    };

    println!("1. 같은 트레이트를 구현한 서로 다른 타입");
    println!("    기사: {}", article.summarize());
    println!("    SNS:  {}", post.summarize());
    // 출력:
    //     기사: 페리스, 러스트 마스코트 10주년, 김기자 (서울)
    //     SNS:  (@ferris님의 글 더 보기...)

    println!("2. 기본 구현은 구현 타입이 정의한 다른 메서드를 불러 씁니다");
    // SocialPost의 summarize는 트레이트의 기본 구현이고, 그 안에서 SocialPost의 summarize_author를 부릅니다.
    println!("    post.summarize_author() = {}", post.summarize_author());
    println!("    post.content = {}", post.content);
    // 출력:
    //     post.summarize_author() = @ferris
    //     post.content = 오늘은 트레이트를 배웠다

    println!("3. 트레이트 메서드도 `트레이트::메서드(&값)` 형태로 부를 수 있습니다");
    // 점 표기와 같은 호출입니다(자동 참조는 ch5). 어느 트레이트의 메서드인지가 코드에 드러납니다.
    println!("    {}", Summary::summarize(&article));
    // 출력: 페리스, 러스트 마스코트 10주년, 김기자 (서울)

    println!("4. 내 트레이트를 표준 라이브러리 타입 Vec에 구현했습니다");
    let thread = vec![
        post,
        SocialPost {
            username: String::from("crab"),
            content: String::from("저도요"),
        },
    ];
    println!(
        "    {} / 작성자 {}",
        thread.summarize(),
        thread.summarize_author()
    );
    // 출력: 글 2개가 이어진 스레드 / 작성자 2명
}
