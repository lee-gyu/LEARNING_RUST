// 지침: 타입이 같고 뜻이 다른 인자(쪽 번호와 쪽당 개수, bool 플래그)를 뉴타입으로 구별합니다.
// 비권장: list_page_raw(posts, page: usize, per_page: usize, newest_first: bool)
// 권장:   list_page(posts, Page, PerPage, NewestFirst)
//
// 게시판 글 12개를 한 쪽에 몇 개씩 나눠 보여 줍니다.
//
// 실행: cargo run -p er-ch6-newtype-pattern --example 02_distinct_args

const POSTS: [&str; 12] = [
    "글01", "글02", "글03", "글04", "글05", "글06", "글07", "글08", "글09", "글10", "글11", "글12",
];

// ---- 비권장 ----------------------------------------------------------------

// usize 두 개와 bool 하나입니다. 순서를 바꿔 넘겨도 타입이 모두 맞으므로 컴파일러는 알 수 없고,
// 호출 지점의 `2, 5, true`는 시그니처를 찾아보기 전에는 뜻을 알 수 없습니다.
fn list_page_raw<'a>(
    posts: &[&'a str],
    page: usize,
    per_page: usize,
    newest_first: bool,
) -> Vec<&'a str> {
    let mut ordered = posts.to_vec();
    if newest_first {
        ordered.reverse();
    }
    ordered
        .into_iter()
        .skip((page - 1) * per_page)
        .take(per_page)
        .collect()
}

// ---- 권장 ------------------------------------------------------------------

// 인자마다 뉴타입을 하나씩 만듭니다. 필드가 공개(pub)라서 만드는 쪽은 `Page(2)`처럼 씁니다.
// 값의 범위까지 지키려면 필드를 감추고 생성자에서 검사합니다(03_validated_newtype.rs).
#[derive(Debug, Clone, Copy)]
struct Page(pub usize); // 1부터 셉니다.

#[derive(Debug, Clone, Copy)]
struct PerPage(pub usize);

// bool도 감쌀 수 있습니다. 선택지가 앞으로도 참/거짓 둘뿐이라면 열거형보다 가볍습니다.
#[derive(Debug, Clone, Copy)]
struct NewestFirst(pub bool);

// 매개변수 자리에서 바로 구조 분해하면 본문은 비권장 판과 한 글자도 다르지 않습니다.
// 바뀐 것은 시그니처, 즉 부르는 쪽과의 약속뿐입니다.
fn list_page<'a>(
    posts: &[&'a str],
    Page(page): Page,
    PerPage(per_page): PerPage,
    NewestFirst(newest_first): NewestFirst,
) -> Vec<&'a str> {
    let mut ordered = posts.to_vec();
    if newest_first {
        ordered.reverse();
    }
    ordered
        .into_iter()
        .skip((page - 1) * per_page)
        .take(per_page)
        .collect()
}

fn main() {
    println!("의도: 최신 글부터, 한 쪽에 5개씩, 2쪽");

    println!("[비권장] usize, usize, bool");
    let intended = list_page_raw(&POSTS, 2, 5, true);
    // 실수: SQL의 `LIMIT 5 OFFSET ..`에 익숙한 사람이 "개수, 쪽" 순서로 넘겼습니다.
    let swapped = list_page_raw(&POSTS, 5, 2, true);
    println!("  (2, 5, true) -> {intended:?}");
    println!("  (5, 2, true) -> {swapped:?}");
    // 출력:
    //   (2, 5, true) -> ["글07", "글06", "글05", "글04", "글03"]
    //   (5, 2, true) -> ["글04", "글03"]

    println!("[권장] Page, PerPage, NewestFirst");
    let page = list_page(&POSTS, Page(2), PerPage(5), NewestFirst(true));
    println!("  (Page(2), PerPage(5), NewestFirst(true)) -> {page:?}");
    // 출력:   (Page(2), PerPage(5), NewestFirst(true)) -> ["글07", "글06", "글05", "글04", "글03"]

    // list_page(&POSTS, PerPage(5), Page(2), NewestFirst(true))는 E0308이고,
    // 컴파일러가 `help: swap these arguments`로 고칠 방법까지 알려 줍니다(README 2.2절).
    let oldest = list_page(&POSTS, Page(1), PerPage(3), NewestFirst(false));
    println!("  (Page(1), PerPage(3), NewestFirst(false)) -> {oldest:?}");
    // 출력:   (Page(1), PerPage(3), NewestFirst(false)) -> ["글01", "글02", "글03"]
}
