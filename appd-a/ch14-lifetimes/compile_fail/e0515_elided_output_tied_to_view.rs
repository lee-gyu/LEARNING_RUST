// 기대하는 에러: E0515
// 핵심 메시지: cannot return value referencing local variable `view`
//
// title_elided의 시그니처 `fn title_elided(&self) -> &str`는 생략 규칙 3에 따라
// `fn title_elided<'s>(&'s self) -> &'s str`, 즉 "반환값은 뷰를 빌린 동안만 유효하다"는 뜻입니다.
// title_of는 뷰를 지역 변수로 만들었으므로, 그 뷰를 빌린 값을 돌려줄 수 없습니다.
// 에러는 title_of의 마지막 줄을 가리키지만 원인은 title_elided의 시그니처입니다.
// 반환값은 뷰가 아니라 뷰가 가리키는 글에서 오므로 `-> &'a str`로 적으면 통과합니다
// (examples/04_elided_lifetimes.rs의 title).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0515_elided_output_tied_to_view.rs --out-dir /tmp/er-ch14

struct PostView<'a> {
    text: &'a str,
}

impl<'a> PostView<'a> {
    fn new(text: &'a str) -> Self {
        PostView { text }
    }

    fn title_elided(&self) -> &str {
        self.text.lines().next().unwrap_or("")
    }
}

fn title_of(post: &str) -> &str {
    let view = PostView::new(post);
    view.title_elided() // E0515: 지역 변수 view를 빌린 값을 돌려줌
}

fn main() {
    let post = String::from("러스트 첫걸음\n설치하고 cargo new로 시작했다.");
    println!("{}", title_of(&post));
}
