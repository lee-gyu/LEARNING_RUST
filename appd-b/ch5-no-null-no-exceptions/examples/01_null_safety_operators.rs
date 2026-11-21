// Q5.1 · Q5.2 · Q5.5 · Q5.8 — Kotlin·TypeScript의 null 연산자를 Option으로 옮기기
//
// 회원 → 주소 → 도시를 따라가는 조회입니다. 회원이 없을 수도, 주소가 없을 수도 있습니다.
// Kotlin과 TypeScript의 null 안전 연산자가 Rust에서는 어떤 메서드나 문법이 되는지 나란히 봅니다.
//
//   Kotlin/TS                 Rust
//   user?.address             user.and_then(|u| u.address.as_ref())
//   city ?: "미등록"           .unwrap_or("미등록")
//   city!!  /  city!          .expect("…")  (None이면 패닉)
//   if (user != null) {…}     if let Some(user) = … {…}
//   (함수 안의 조기 반환)        ?  (Option을 돌려주는 함수 안에서)
//
// Option의 메서드 자체는 Part 1 ch2에서 자세히 다룹니다. 여기서는 대응 관계만 봅니다.
//
// 실행: cargo run -p faq-ch5-no-null-no-exceptions --example 01_null_safety_operators

use std::collections::HashMap;

struct Address {
    city: String,
}

struct User {
    name: String,
    address: Option<Address>, // 주소를 등록하지 않은 회원도 있음
}

fn members() -> HashMap<u32, User> {
    HashMap::from([
        (
            1,
            User {
                name: "하준".to_string(),
                address: Some(Address {
                    city: "부산".to_string(),
                }),
            },
        ),
        (
            2,
            User {
                name: "서아".to_string(),
                address: None,
            },
        ),
    ])
}

/// `?.`의 사슬: 어느 단계에서든 없으면 None
fn city_chain(db: &HashMap<u32, User>, id: u32) -> Option<&str> {
    db.get(&id)
        .and_then(|u| u.address.as_ref())
        .map(|a| a.city.as_str())
}

/// 같은 일을 `?`로: Option을 돌려주는 함수 안에서 None이면 바로 None을 돌려줌
fn city_question(db: &HashMap<u32, User>, id: u32) -> Option<&str> {
    let user = db.get(&id)?;
    let address = user.address.as_ref()?;
    Some(&address.city)
}

fn main() {
    let db = members();

    println!("1. ?. 사슬과 ?: 기본값");
    for id in [1, 2, 3] {
        let city = city_chain(&db, id).unwrap_or("미등록");
        println!("  회원 {id}: {city}");
    }
    // 출력:   회원 1: 부산
    // 출력:   회원 2: 미등록
    // 출력:   회원 3: 미등록

    println!("2. 같은 일을 ?로");
    for id in [1, 2, 3] {
        println!("  회원 {id}: {:?}", city_question(&db, id));
    }
    // 출력:   회원 1: Some("부산")
    // 출력:   회원 2: None
    // 출력:   회원 3: None

    println!("3. if (user != null) → if let");
    if let Some(user) = db.get(&2) {
        println!(
            "  {}님은 주소 등록 여부: {}",
            user.name,
            user.address.is_some()
        );
    }
    // 출력:   서아님은 주소 등록 여부: false

    println!("4. !! → expect (없을 리 없을 때만)");
    let first = db.get(&1).expect("회원 1은 시작할 때 넣어 두었음");
    println!("  {}", first.name);
    // 출력:   하준
    // `db.get(&3).expect(…)`였다면 여기서 패닉합니다. Kotlin의 `!!`가 NPE를 던지는 것과 같은 자리입니다.
}
