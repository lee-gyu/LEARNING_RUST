// 기대하는 에러: E0599
// 핵심 메시지: no method named `to_uppercase` found for enum `Option<T>` in the current scope
//
// Kotlin의 `nickname?.uppercase()`나 TypeScript의 `nickname?.toUpperCase()`를 옮기며 `?.` 자리에 `.`만 쓴
// 코드입니다. `find_nickname`이 돌려주는 것은 `String`이 아니라 `Option<String>`이고,
// `Option`에는 `to_uppercase` 메서드가 없습니다. "값이 없을 수도 있다"는 것이 타입에 들어 있으므로, 안의 값을
// 쓰기 전에 없는 경우를 어떻게 할지 정해야 합니다.
//
// 고치는 법:
//   nickname.as_deref().map(str::to_uppercase)                 // ?. 처럼: Option<String>
//   nickname.as_deref().map_or(String::new(), str::to_uppercase)  // ?. 와 ?: 를 함께
//   if let Some(name) = &nickname { … }                          // if (nickname != null) 처럼
// (`len`으로 시험하면 `Option`에 바깥에서 쓸 수 없는 같은 이름의 메서드가 있어 E0624가 나옵니다.)
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch5-no-null-no-exceptions compile_fail/e0599_method_on_option.rs

fn find_nickname(id: u32) -> Option<String> {
    if id == 1 {
        Some("hajun".to_string())
    } else {
        None
    }
}

fn main() {
    let nickname = find_nickname(1);
    println!("{}", nickname.to_uppercase()); // E0599
}
