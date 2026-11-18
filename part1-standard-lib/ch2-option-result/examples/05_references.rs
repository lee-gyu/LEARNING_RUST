// 이 예제가 보여 주는 것: 소유권을 유지한 채 들여다보기 — as_ref, as_deref, as_mut, copied, cloned
// 실행: cargo run -p p1-ch2-option-result --example 05_references
//
// map, and_then, unwrap_or, is_some_and 등 대부분의 콤비네이터는 시그니처의 첫 매개변수가 self입니다.
// 즉 Option/Result를 통째로 가져가(소비해) 버립니다. 원본을 계속 쓰려면 먼저 "참조를 담은 Option"으로
// 바꾼 뒤 그것을 소비하게 합니다. 그 변환을 하는 메서드가 as_ref, as_deref, as_mut이고,
// 반대로 참조를 담은 Option을 값을 담은 Option으로 되돌리는 메서드가 copied, cloned입니다.
//
//   &Option<T>         --as_ref()-->   Option<&T>
//   &Option<String>    --as_deref()--> Option<&str>     (String이 역참조로 str이 되는 것을 이용)
//   &mut Option<T>     --as_mut()-->   Option<&mut T>
//   Option<&T>         --copied()-->   Option<T>        (T: Copy)
//   Option<&T>         --cloned()-->   Option<T>        (T: Clone)

struct User {
    name: String,
    nickname: Option<String>,
}

// &User만 빌려 왔으므로 user.nickname을 소비하는 메서드를 바로 부를 수 없습니다.
// user.nickname.map(...)이라고 쓰면 E0507입니다(compile_fail/e0507_map_behind_reference.rs).
// as_ref()가 Option<&String>을 새로 만들고, map_or는 그 새 Option을 소비합니다. 원본은 건드리지 않습니다.
fn nickname_len(user: &User) -> usize {
    user.nickname.as_ref().map_or(0, |n| n.chars().count())
}

// as_deref()는 Option<&str>을 만듭니다. 그래서 &str 기본값을 unwrap_or에 바로 줄 수 있습니다.
// (as_ref()였다면 Option<&String>이라 기본값도 &String이어야 합니다.)
// 돌려주는 &str은 user 안의 String을 가리키므로, 생략 규칙(Part 0 ch10)에 따라 user와 같은 라이프타임입니다.
fn display_name(user: &User) -> &str {
    user.nickname.as_deref().unwrap_or(&user.name)
}

fn main() {
    println!("== 1. as_ref, as_deref: 빌린 Option 들여다보기 ==");
    let users = [
        User {
            name: String::from("김하나"),
            nickname: Some(String::from("하나둘")),
        },
        User {
            name: String::from("이두리"),
            nickname: None,
        },
    ];
    for user in &users {
        println!(
            "{} → 별명 글자 수: {}, 표시 이름: {}",
            user.name,
            nickname_len(user),
            display_name(user)
        );
    }
    // 출력: 김하나 → 별명 글자 수: 3, 표시 이름: 하나둘
    // 출력: 이두리 → 별명 글자 수: 0, 표시 이름: 이두리

    // 지역 변수도 마찬가지입니다. as_ref()로 물어본 뒤에도 city는 그대로 남아 있습니다.
    let city: Option<String> = Some(String::from("부산"));
    let len = city.as_ref().map(|c| c.len()); // c: &String
    let is_busan = city.as_deref() == Some("부산"); // Option<&str>끼리 비교
    println!("바이트 수: {len:?}, 부산인가: {is_busan}, 원본: {city:?}");
    // 출력: 바이트 수: Some(6), 부산인가: true, 원본: Some("부산")

    println!("\n== 2. as_mut: 안의 값을 제자리에서 고치기 ==");
    let mut tags: Option<Vec<&str>> = Some(vec!["rust"]);
    // as_mut()은 Option<&mut Vec<&str>>를 만듭니다. Part 0 ch6의 `if let Some(v) = &mut tags`와 같은 일입니다.
    if let Some(list) = tags.as_mut() {
        list.push("std");
    }
    println!("{tags:?}");
    // 출력: Some(["rust", "std"])

    println!("\n== 3. copied, cloned: Option<&T>를 Option<T>로 ==");
    // first(), get(), iter().max() 같은 메서드는 컬렉션 안을 가리키는 Option<&T>를 돌려줍니다.
    // 참조를 들고 있는 동안에는 컬렉션을 고칠 수 없으므로, 값만 필요하면 복사해 둡니다.
    let mut scores = vec![72, 95, 88];
    let best: Option<i32> = scores.iter().max().copied(); // &i32 → i32 (i32는 Copy)
    scores.push(100); // best가 scores를 빌리고 있지 않으므로 고칠 수 있습니다.
    println!("추가 전 최고점: {best:?}, 지금 점수들: {scores:?}");
    // 출력: 추가 전 최고점: Some(95), 지금 점수들: [72, 95, 88, 100]
    // copied()를 빼면 best는 Option<&i32>이고, best를 쓰는 동안 push하면 E0502(가변 대여 충돌)입니다.

    let names = vec![String::from("가람"), String::from("나래")];
    let first: Option<String> = names.first().cloned(); // String은 Copy가 아니므로 cloned()로 복제
    println!("복제한 첫 이름: {first:?}, 원본: {names:?}");
    // 출력: 복제한 첫 이름: Some("가람"), 원본: ["가람", "나래"]

    println!("\n== 4. Result에도 같은 메서드가 있습니다 ==");
    // Result의 as_ref: &Result<T, E> → Result<&T, &E>
    let loaded: Result<String, String> = Ok(String::from("설정 내용"));
    let chars = loaded.as_ref().map(|s| s.chars().count());
    println!("글자 수: {chars:?}, 원본: {loaded:?}");
    // 출력: 글자 수: Ok(5), 원본: Ok("설정 내용")

    println!("\n== 5. 안의 타입이 Copy면 Option/Result도 Copy입니다 ==");
    // Option<i32>는 Copy이므로 map이 self를 가져가도 복사본이 넘어갈 뿐, 원래 변수는 그대로 쓸 수 있습니다.
    // Option<String>은 Copy가 아니므로 같은 코드가 E0382입니다(compile_fail/e0382_map_consumes_option.rs).
    let maybe_count: Option<i32> = "3".parse().ok();
    let doubled = maybe_count.map(|n| n * 2);
    println!("원본: {maybe_count:?}, 두 배: {doubled:?}");
    // 출력: 원본: Some(3), 두 배: Some(6)
}
