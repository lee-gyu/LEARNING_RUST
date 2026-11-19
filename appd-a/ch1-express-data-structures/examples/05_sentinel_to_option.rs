// 지침: "없음"을 -1이나 빈 문자열 같은 센티널 값으로 나타내지 말고 Option으로 나타냅니다.
// 비권장: 못 찾으면 -1을 돌려주는 위치 함수, 설정이 없으면 ""를 돌려주는 조회 함수
// 권장:   Option<usize>, Option<&str>
//
// 실행: cargo run -p er-ch1-express-data-structures --example 05_sentinel_to_option

use std::collections::HashMap;

// 마지막 정류장 다음은 다시 첫 정류장인 순환 버스 노선입니다.
const ROUTE: [&str; 4] = ["터미널", "시청", "도서관", "병원"];

// ---- 1. -1 센티널 -------------------------------------------------------------

// 비권장: 찾지 못하면 -1. 반환 타입 i32는 -1이 특별하다는 사실을 담지 못합니다.
fn position_or_minus_one(name: &str) -> i32 {
    for (i, stop) in ROUTE.iter().enumerate() {
        if *stop == name {
            return i as i32;
        }
    }
    -1
}

fn next_stop_discouraged(name: &str) -> &'static str {
    // -1을 확인하는 것을 잊었습니다. -1 + 1 = 0이므로 계산은 멀쩡히 이어지고
    // 노선에 없는 정류장의 "다음 정류장"으로 첫 정류장이 나옵니다.
    let i = position_or_minus_one(name);
    let next = (i + 1) as usize % ROUTE.len();
    ROUTE[next]
}

// 권장: 찾지 못하면 None. 호출하는 쪽은 None을 처리하지 않고는 위치를 꺼낼 수 없습니다.
fn position(name: &str) -> Option<usize> {
    ROUTE.iter().position(|stop| *stop == name)
}

fn next_stop(name: &str) -> Option<&'static str> {
    let i = position(name)?; // None이면 여기서 None을 돌려주고 끝납니다.
    Some(ROUTE[(i + 1) % ROUTE.len()])
}

// ---- 2. 빈 문자열 센티널 -------------------------------------------------------

// 비권장: 설정이 없으면 ""를 돌려줍니다. 그런데 ""는 사용자가 실제로 고를 수 있는 값이기도 합니다.
fn setting_or_empty<'a>(settings: &HashMap<&str, &'a str>, key: &str) -> &'a str {
    settings.get(key).copied().unwrap_or("")
}

// 권장: 설정이 없으면 None, 비워 두었으면 Some("").
fn setting<'a>(settings: &HashMap<&str, &'a str>, key: &str) -> Option<&'a str> {
    settings.get(key).copied()
}

fn main() {
    println!("[1] 순환 노선의 다음 정류장");
    for name in ["병원", "공항"] {
        let discouraged = next_stop_discouraged(name);
        let recommended = match next_stop(name) {
            Some(next) => next.to_string(),
            None => String::from("노선에 없는 정류장"),
        };
        println!("  {name} -> 비권장: {discouraged} / 권장: {recommended}");
    }
    // 출력:
    //   병원 -> 비권장: 터미널 / 권장: 터미널
    //   공항 -> 비권장: 터미널 / 권장: 노선에 없는 정류장

    println!("[2] 설정 조회 (기본값: prompt는 \"> \", editor는 \"vi\")");
    // 사용자는 프롬프트를 일부러 비워 두었고, editor는 설정하지 않았습니다.
    let settings = HashMap::from([("prompt", ""), ("theme", "dark")]);
    for (key, default) in [("prompt", "> "), ("editor", "vi")] {
        // 비권장: ""이면 "없음"으로 보고 기본값을 씁니다. 일부러 비운 값과 구별할 수 없습니다.
        let raw = setting_or_empty(&settings, key);
        let discouraged = if raw.is_empty() { default } else { raw };
        // 권장: None일 때만 기본값을 씁니다.
        let found = setting(&settings, key);
        let recommended = found.unwrap_or(default);
        println!("  {key}: 비권장 {raw:?} -> {discouraged:?} / 권장 {found:?} -> {recommended:?}");
    }
    // 출력:
    //   prompt: 비권장 "" -> "> " / 권장 Some("") -> ""
    //   editor: 비권장 "" -> "vi" / 권장 None -> "vi"
}
