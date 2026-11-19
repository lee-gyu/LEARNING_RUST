//! 생략된 라이프타임 읽기: 생략 규칙이 고른 관계가 의도와 다르면 직접 적고, 숨은 라이프타임은 '_로 드러냅니다.
//!
//! 실행: cargo run -p er-ch14-lifetimes --example 04_elided_lifetimes
//!
//! 생략은 라이프타임이 없다는 뜻이 아닙니다. 컴파일러가 정해진 규칙으로 채워 넣은 관계가
//! 시그니처에 그대로 있고, 부르는 쪽은 그 관계로 검사받습니다.

use std::fmt;

/// 블로그 글의 첫 줄(제목)과 나머지(본문)를 복사 없이 들여다보는 뷰 타입입니다.
struct PostView<'a> {
    text: &'a str,
}

impl<'a> PostView<'a> {
    fn new(text: &'a str) -> Self {
        PostView { text }
    }

    /// 비권장: 생략 규칙 3에 따라 반환값의 라이프타임이 `&self`, 즉 뷰를 빌린 구간이 됩니다.
    /// 실제로 돌려주는 것은 뷰가 아니라 뷰가 가리키는 글의 조각입니다.
    fn title_elided(&self) -> &str {
        self.text.lines().next().unwrap_or("")
    }

    /// 권장: 반환값이 글(`'a`)에서 온다는 사실을 그대로 적습니다. 본문은 똑같습니다.
    fn title(&self) -> &'a str {
        self.text.lines().next().unwrap_or("")
    }

    fn body_len(&self) -> usize {
        self.text
            .lines()
            .skip(1)
            .map(|line| line.chars().count())
            .sum()
    }
}

// 뷰를 함수 안에서 만들고 제목만 돌려주는 도우미.
// title_elided()를 쓰면 지역 뷰를 빌린 값을 돌려주는 셈이 되어 E0515입니다
// (compile_fail/e0515_elided_output_tied_to_view.rs).
fn title_of(post: &str) -> &str {
    let view = PostView::new(post);
    view.title()
}

// 비권장: 반환 타입 `PostView`에 라이프타임이 숨어 있습니다. 읽는 사람은 이 함수가
// 인자를 빌린 값을 돌려준다는 것을 시그니처만 보고 알 수 없습니다.
#[expect(
    mismatched_lifetime_syntaxes,
    reason = "숨은 라이프타임을 보여 주려고 일부러 '_를 생략했습니다"
)]
fn open_hidden(post: &str) -> PostView {
    PostView::new(post.trim_start())
}

// 권장: `'_`는 새 라이프타임을 만들지 않고 "여기 생략된 라이프타임이 있다"고만 표시합니다.
// 뜻은 위 함수와 완전히 같습니다(생략 규칙 2에 따라 post의 라이프타임).
fn open(post: &str) -> PostView<'_> {
    PostView::new(post.trim_start())
}

// impl 블록에서 이름을 쓸 일이 없는 라이프타임도 '_로 적습니다. Formatter<'_>도 같은 표시입니다.
impl fmt::Display for PostView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "《{}》 본문 {}자", self.title(), self.body_len())
    }
}

fn main() {
    let posts = [
        String::from("러스트 첫걸음\n설치하고 cargo new로 시작했다."),
        String::from("대여 검사기와 친해지기\n에러 메시지를 끝까지 읽자."),
    ];

    println!("1. 생략된 판도 뷰가 살아 있는 동안에는 잘 동작합니다");
    let view = PostView::new(&posts[0]);
    let title = view.title_elided(); // title이 쓰이는 동안 view도 살아 있어야 합니다.
    println!("    제목: {title}");
    // 출력:     제목: 러스트 첫걸음

    println!("2. 결과가 뷰보다 오래 살아야 하면 'a를 직접 적은 판이 필요합니다");
    println!("    도우미가 돌려준 제목: {}", title_of(&posts[1]));
    // 뷰를 글마다 임시로 만들고 바로 버려도, 제목은 posts가 살아 있는 동안 유효합니다.
    // title_elided()로 바꾸면 클로저가 임시 뷰를 빌린 값을 돌려주게 되어 E0515입니다.
    let titles: Vec<&str> = posts.iter().map(|p| PostView::new(p).title()).collect();
    println!("    모은 제목: {titles:?}");
    // 출력:
    //     도우미가 돌려준 제목: 대여 검사기와 친해지기
    //     모은 제목: ["러스트 첫걸음", "대여 검사기와 친해지기"]

    println!("3. '_는 생략된 라이프타임이 있다는 표시일 뿐, 뜻은 같습니다");
    let hidden = open_hidden(&posts[0]);
    let marked = open(&posts[0]);
    println!("    숨긴 판: {hidden}");
    println!("    '_ 판: {marked}");
    // 출력:
    //     숨긴 판: 《러스트 첫걸음》 본문 21자
    //     '_ 판: 《러스트 첫걸음》 본문 21자
}
