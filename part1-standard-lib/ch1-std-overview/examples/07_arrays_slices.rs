// 이 예제가 보여 주는 것: 배열과 슬라이스의 기본 메서드
//   배열 → 슬라이스 변환, 꺼내기(`first`/`last`/`get`/`split_first`), 나누기(`split_at`),
//   묶어 보기(`windows`/`chunks`), 찾기(`contains`/`starts_with`), 제자리 수정(`swap`/`reverse`/`fill`), 거꾸로 돌기
// 실행: cargo run -p p1-ch1-std-overview --example 07_arrays_slices

fn main() {
    // ── 1. 배열의 메서드 대부분은 슬라이스의 메서드입니다 ────────────────────────
    // 배열 문서: "Arrays coerce to slices ([T]), so a slice method may be called on an array."
    // 배열 `[T; N]` 자체의 메서드는 `map`, `as_slice`, `each_ref` 등 몇 개뿐이고,
    // `len`, `first`, `contains` 같은 메서드는 배열이 슬라이스 `[T]`로 바뀌어 불리는 것입니다.
    let temps: [i32; 7] = [21, 23, 19, 25, 28, 22, 20];
    let doubled = temps.map(|t| t * 2); // 배열의 메서드: 같은 길이의 새 배열
    println!("doubled = {doubled:?}"); // 출력: doubled = [42, 46, 38, 50, 56, 44, 40]
    let as_slice: &[i32] = temps.as_slice(); // `&temps`, `&temps[..]`도 같은 슬라이스입니다.
    println!("len = {}", as_slice.len()); // 출력: len = 7

    // ── 2. 원소 꺼내기: 패닉 대신 Option ───────────────────────────────────
    // 비어 있을 수 있는 슬라이스에서 `[0]`은 패닉하지만 `first`는 None을 돌려줍니다.
    let empty: [i32; 0] = [];
    println!("{:?} {:?} {:?}", temps.first(), temps.last(), empty.first()); // 출력: Some(21) Some(20) None
    println!("{:?} {:?}", temps.get(3), temps.get(10)); // 출력: Some(25) None

    // `split_first`는 (첫 원소, 나머지 슬라이스)를 한꺼번에 돌려줍니다.
    if let Some((head, rest)) = temps.split_first() {
        println!("head = {head}, rest = {rest:?}"); // 출력: head = 21, rest = [23, 19, 25, 28, 22, 20]
    }

    // ── 3. 나누기 ────────────────────────────────────────────────────────
    // `split_at(mid)`는 [0, mid)와 [mid, len)으로 나눕니다. mid가 길이보다 크면 패닉합니다.
    let (weekdays, weekend) = temps.split_at(5);
    println!("평일 = {weekdays:?}, 주말 = {weekend:?}"); // 출력: 평일 = [21, 23, 19, 25, 28], 주말 = [22, 20]
    // 패닉이 싫으면 1.80.0에 추가된 `split_at_checked`를 씁니다.
    println!("{:?}", temps.split_at_checked(9)); // 출력: None

    // ── 4. windows와 chunks ────────────────────────────────────────────────
    // `windows(n)`: 길이 n짜리 창을 한 칸씩 밀며 겹치게 봅니다. 인접한 값끼리 비교할 때 씁니다.
    for w in temps.windows(2) {
        if w[1] > w[0] + 4 {
            println!("{} → {}: 크게 올랐습니다", w[0], w[1]);
        }
    }
    // 출력: 19 → 25: 크게 올랐습니다
    // `chunks(n)`: 겹치지 않게 n개씩 자릅니다. 마지막 조각은 n보다 짧을 수 있습니다.
    for chunk in temps.chunks(3) {
        println!("chunk = {chunk:?}");
    }
    // 출력: chunk = [21, 23, 19]
    // 출력: chunk = [25, 28, 22]
    // 출력: chunk = [20]

    // ── 5. 찾기 ──────────────────────────────────────────────────────────
    // `contains`는 값 하나를, `starts_with`/`ends_with`는 슬라이스 하나를 (참조로) 받습니다.
    let path = ["home", "ferris", "docs", "notes.txt"];
    println!("contains = {}", path.contains(&"ferris")); // 출력: contains = true
    println!("starts_with = {}", path.starts_with(&["home", "ferris"])); // 출력: starts_with = true
    println!("ends_with = {}", path.ends_with(&["notes.txt"])); // 출력: ends_with = true

    // ── 6. 제자리 수정: &mut self 메서드 ─────────────────────────────────────
    // `swap`, `reverse`, `fill`은 `&mut self`를 받으므로 `mut` 변수에서만 부를 수 있습니다.
    let mut cards = [1, 2, 3, 4, 5];
    cards.swap(0, 4);
    println!("swap(0, 4) → {cards:?}"); // 출력: swap(0, 4) → [5, 2, 3, 4, 1]
    cards.reverse();
    println!("reverse   → {cards:?}"); // 출력: reverse   → [1, 4, 3, 2, 5]
    cards[1..4].fill(0); // 부분 슬라이스에도 그대로 쓸 수 있습니다.
    println!("fill      → {cards:?}"); // 출력: fill      → [1, 0, 0, 0, 5]

    // ── 7. 반복과 합계 ────────────────────────────────────────────────────
    // `iter()`는 원소의 참조를 차례로 내놓는 반복자를 돌려줍니다(Part 0 ch13).
    // 끝에서부터 돌려면 `rev()`를 붙입니다. 어댑터 전반은 `ch5-iterators`에서 다룹니다.
    let countdown: Vec<i32> = [1, 2, 3].iter().rev().copied().collect();
    println!("countdown = {countdown:?}"); // 출력: countdown = [3, 2, 1]
    let sum: i32 = temps.iter().sum();
    println!("평균 = {:.1}", sum as f64 / temps.len() as f64); // 출력: 평균 = 22.6

    // ── 8. 배열끼리의 비교와 복사 ───────────────────────────────────────────
    // 원소가 비교 가능하면 배열도 `==`, `<`로 비교합니다. 앞에서부터 처음 다른 원소로 결정됩니다(사전식 비교).
    println!("{} {}", [1, 2, 3] == [1, 2, 4], [1, 2, 9] < [1, 3, 0]); // 출력: false true
    // 원소가 `Copy`면 배열도 `Copy`입니다. 대입하면 통째로 복사되어 서로 독립입니다.
    let original = [0; 3];
    let mut copy = original;
    copy[0] = 9;
    println!("original = {original:?}, copy = {copy:?}"); // 출력: original = [0, 0, 0], copy = [9, 0, 0]
}
