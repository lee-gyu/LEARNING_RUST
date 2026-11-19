//! 지침 2: 막힌 체인은 단계마다 타입을 적어 쪼개고, 원인이 된 단계를 옮긴다.
//!
//! 이메일 주소에서 도메인을 소문자로 꺼내는 함수를 `&str`을 돌려주게 쓰면 E0515입니다.
//! 에러를 없애려고 언제나 새 `String`을 돌려주는 판(비권장)과, 체인을 쪼개 원인 단계
//! (`to_lowercase`)를 찾아 뒤로 옮기고 `Cow`로 "바꿀 것이 있을 때만" 새로 만드는 판(권장)을
//! 같은 주소 목록으로 실행해, 새로 만든 문자열의 수를 비교합니다.
//!
//! 실행: cargo run -p er-ch15-borrow-checker --example 02_split_the_chain

use std::borrow::Cow;

// 출발점(컴파일되지 않음, E0515 cannot return value referencing temporary value):
//
//     fn domain(email: &str) -> &str {
//         email.trim().to_lowercase().rsplit('@').next().unwrap_or("")
//     }
//
// 단계마다 타입을 적어 쪼개면 에러가 "지역 변수 `lowered`를 빌린 값을 돌려준다"로 바뀌어,
// 받은 문자열이 아니라 새로 만든 String을 빌리고 있다는 것이 드러납니다.
//
//     let trimmed: &str = email.trim();               // 입력을 빌림
//     let lowered: String = trimmed.to_lowercase();   // 새 값. 함수가 끝나면 사라짐
//     let part: Option<&str> = lowered.rsplit('@').next(); // lowered를 빌림
//     part.unwrap_or("")                              // E0515 ... local variable `lowered`

/// 비권장: 에러가 사라지도록 언제나 소유한 값을 돌려줍니다.
/// 이미 소문자인 주소에도 `to_lowercase`와 `to_string`이 String을 하나씩 만듭니다.
fn domain_always_owned(email: &str) -> String {
    email
        .trim()
        .to_lowercase()
        .rsplit('@')
        .next()
        .unwrap_or("")
        .to_string()
}

/// 권장: 입력을 빌리는 단계를 앞에, 새 값을 만드는 단계를 뒤에 둡니다.
/// 바꿀 글자가 없으면 입력의 일부를 그대로 빌려 돌려줍니다.
fn domain(email: &str) -> Cow<'_, str> {
    let trimmed: &str = email.trim();
    let part: &str = trimmed.rsplit('@').next().unwrap_or("");
    if part.chars().any(char::is_uppercase) {
        Cow::Owned(part.to_lowercase()) // 바꿀 것이 있을 때만 새로 만듦
    } else {
        Cow::Borrowed(part) // email의 일부. 복사 없음
    }
}

fn main() {
    let emails = [
        "mina@example.com",
        "  JUN@Example.COM ",
        "hana@mail.kr",
        "ops@example.com",
    ];

    println!("[비권장] 언제나 String을 돌려줌");
    let owned: Vec<String> = emails.iter().map(|e| domain_always_owned(e)).collect();
    println!("  {owned:?}");
    // 출력:   ["example.com", "example.com", "mail.kr", "example.com"]
    // to_lowercase와 to_string은 부를 때마다 String을 하나씩 만듭니다.
    println!(
        "  새로 만든 String: {}개 (주소마다 to_lowercase 1 + to_string 1)",
        emails.len() * 2
    );
    // 출력:   새로 만든 String: 8개 (주소마다 to_lowercase 1 + to_string 1)

    println!("[권장] 원인 단계(to_lowercase)를 뒤로 옮기고 Cow로 돌려줌");
    let mut made = 0;
    for e in emails {
        let d = domain(e);
        let kind = match &d {
            Cow::Borrowed(_) => "빌림",
            Cow::Owned(_) => {
                made += 1;
                "새로 만듦"
            }
        };
        println!("  {d:<12} {kind}");
    }
    // 출력:   example.com  빌림
    // 출력:   example.com  새로 만듦      ← 대문자가 있던 "  JUN@Example.COM "
    // 출력:   mail.kr      빌림
    // 출력:   example.com  빌림
    println!("  새로 만든 String: {made}개");
    // 출력:   새로 만든 String: 1개

    // 부르는 쪽이 비교만 한다면 소문자로 바꿀 필요조차 없습니다.
    let same = emails
        .iter()
        .filter(|e| raw_domain(e).eq_ignore_ascii_case("example.com"))
        .count();
    println!("  example.com 주소: {same}개 (새로 만든 String 0개)");
    // 출력:   example.com 주소: 3개 (새로 만든 String 0개)
}

/// 비교만 할 때: 입력을 빌리는 단계만 남깁니다.
fn raw_domain(email: &str) -> &str {
    email.trim().rsplit('@').next().unwrap_or("")
}
