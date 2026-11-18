// 이 예제가 보여 주는 것: ToOwned와 Cow(쓰기 시 복제). 고칠 것이 없으면 빌린 채로 두고,
//                        고쳐야 할 때만 소유한 사본을 만들어 불필요한 할당을 줄입니다.
// 실행: cargo run -p p1-ch6-core-traits --example 04_cow
//
// pub enum Cow<'a, B: ?Sized + ToOwned + 'a> {
//     Borrowed(&'a B),                       // 빌린 값 (예: &str, &[T])
//     Owned(<B as ToOwned>::Owned),          // 소유한 값 (예: String, Vec<T>)
// }
// B = str이면 Cow<str>은 "&str 또는 String", B = [T]이면 Cow<[T]>는 "&[T] 또는 Vec<T>"입니다.
// 어느 쪽인지 출력할 때는 주소 값 대신 matches!(.., Cow::Borrowed(_))의 참/거짓을 씁니다.

use std::borrow::Cow;

// clippy의 ptr_arg는 "&Cow<B> 대신 &B를 받으라"고 권합니다. 내용만 읽는 함수라면 그게 맞지만,
// 이 함수는 어느 배리언트인지 살펴보는 것이 목적이라 Cow 자체가 필요합니다.
#[expect(
    clippy::ptr_arg,
    reason = "배리언트를 검사하려면 Cow 자체를 받아야 합니다"
)]
fn is_borrowed<B: ?Sized + ToOwned>(c: &Cow<'_, B>) -> bool {
    matches!(c, Cow::Borrowed(_))
}

// HTML 특수 문자를 바꿉니다. 바꿀 글자가 없으면 입력을 그대로 빌려 돌려줍니다(할당 없음).
fn escape_html(input: &str) -> Cow<'_, str> {
    if !input.contains(['<', '>', '&']) {
        return Cow::Borrowed(input);
    }
    let mut out = String::with_capacity(input.len() + 8);
    for c in input.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

// 비교용: 언제나 새 String을 만들어 돌려주는 버전
fn escape_html_always(input: &str) -> String {
    escape_html(input).into_owned()
}

// 음수를 0으로 바꿉니다. to_mut()은 처음 부를 때만 슬라이스를 Vec으로 복제합니다.
fn clamp_negatives(values: &mut Cow<'_, [i32]>) {
    for i in 0..values.len() {
        if values[i] < 0 {
            values.to_mut()[i] = 0;
        }
    }
}

fn main() {
    // 1. String::from_utf8_lossy (ch3): 올바른 UTF-8이면 빌리고, 고칠 곳이 있을 때만 String을 만듭니다.
    println!("== 1. from_utf8_lossy ==");
    let good = "안녕".as_bytes();
    let bad = [0xEC, 0x95, 0x88, 0xFF];
    let a = String::from_utf8_lossy(good);
    let b = String::from_utf8_lossy(&bad);
    println!("{a} 빌림? {}", is_borrowed(&a)); // 출력: 안녕 빌림? true
    println!("{b} 빌림? {}", is_borrowed(&b)); // 출력: 안� 빌림? false

    // 2. 직접 만든 Cow 반환 함수: 할당 횟수 비교
    println!("\n== 2. 필요할 때만 할당하기 ==");
    let comments = [
        "좋은 글 감사합니다",
        "a < b 이면 참",
        "다음 편 기다려요",
        "Tom & Jerry",
        "추천합니다",
        "잘 읽었습니다",
    ];
    let mut owned_count = 0;
    for text in comments {
        let escaped = escape_html(text);
        if !is_borrowed(&escaped) {
            owned_count += 1;
            println!("바뀜: {text} → {escaped}");
        }
    }
    // 출력:
    // 바뀜: a < b 이면 참 → a &lt; b 이면 참
    // 바뀜: Tom & Jerry → Tom &amp; Jerry
    let always: Vec<String> = comments.iter().map(|t| escape_html_always(t)).collect();
    println!(
        "String 버전: {}번 할당, Cow 버전: {owned_count}번 할당",
        always.len()
    );
    // 출력: String 버전: 6번 할당, Cow 버전: 2번 할당

    // 3. Cow는 Deref를 구현하므로 안쪽 &str의 메서드를 그대로 씁니다. &str과 == 비교도 됩니다.
    println!("\n== 3. Deref와 비교 ==");
    let plain = escape_html("추천합니다");
    println!(
        "{} {} {}",
        plain.len(),
        plain.starts_with("추천"),
        plain == "추천합니다"
    ); // 출력: 15 true true

    // 4. to_mut(): 빌린 상태에서 처음 고칠 때 한 번만 복제합니다.
    println!("\n== 4. to_mut ==");
    let readings = [3, 5, 2];
    let mut no_change = Cow::from(&readings[..]);
    clamp_negatives(&mut no_change);
    println!("{no_change:?} 빌림? {}", is_borrowed(&no_change)); // 출력: [3, 5, 2] 빌림? true

    let raw = [4, -1, 7, -3];
    let mut fixed = Cow::from(&raw[..]);
    clamp_negatives(&mut fixed); // 음수가 두 개지만 복제는 첫 번째 to_mut()에서 한 번뿐입니다.
    println!("{fixed:?} 빌림? {}", is_borrowed(&fixed)); // 출력: [4, 0, 7, 0] 빌림? false
    println!("원본은 그대로: {raw:?}"); // 출력: 원본은 그대로: [4, -1, 7, -3]

    // 5. into_owned(): Borrowed면 복제하고, Owned면 안의 값을 그대로 꺼냅니다(복제 없음).
    println!("\n== 5. into_owned ==");
    let owned_cow = escape_html("<b>");
    let ptr_before = owned_cow.as_ptr(); // Owned 안 String 버퍼의 주소(비교만 합니다)
    let s: String = owned_cow.into_owned();
    println!("{s} 같은 버퍼인가? {}", s.as_ptr() == ptr_before); // 출력: &lt;b&gt; 같은 버퍼인가? true

    // 6. ToOwned: 빌린 타입에서 소유 타입을 만드는 트레이트. Cow가 "복제할 때" 쓰는 것이 이것입니다.
    //    Clone은 &T → T만 만들 수 있어서 &str → String, &[T] → Vec<T>를 표현하지 못합니다.
    println!("\n== 6. ToOwned ==");
    let word: String = "러스트".to_owned(); // str::to_owned → String
    let nums: Vec<i32> = [1, 2, 3][..].to_owned(); // [i32]::to_owned → Vec<i32>
    let mut reused = String::from("이전 내용");
    "새 내용".clone_into(&mut reused); // reused의 버퍼를 다시 써서 복사합니다.
    println!("{word} {nums:?} {reused}"); // 출력: 러스트 [1, 2, 3] 새 내용
}
