// 지침 4-①: 매크로 안에 숨은 제어 흐름(return, ?, break)을 두지 않습니다.
//
// 매크로가 펼친 코드 안의 `return`은 매크로가 아니라 "매크로를 부른 함수"에서 돌아갑니다.
// 부르는 쪽 코드에는 `return`도 `?`도 보이지 않으므로, 읽는 사람은 그 줄에서 함수가 끝날 수 있다는 것을 모릅니다.
// 실패할 수 있는 일은 `Result`를 돌려주는 함수로 쓰고, 흐름은 부르는 쪽이 `?`나 `match`로 고르게 합니다.
//
// 실행: cargo run -p er-ch28-macros-judiciously --example 05_hidden_control_flow

use std::collections::HashMap;

/// "키=값" 줄들을 읽어 맵으로 만듭니다.
fn parse(text: &str) -> HashMap<&str, &str> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .collect()
}

// 비권장: 키가 없으면 매크로를 부른 함수에서 `return Err(..)`로 빠져나갑니다.
macro_rules! setting {
    ($map:expr, $key:expr) => {{
        let key = $key;
        match $map.get(key) {
            Some(value) => *value,
            None => return Err(format!("`{key}` 설정이 없습니다")),
        }
    }};
}

// 키 목록의 값을 모두 출력하려는 함수입니다. 반복문 안의 한 줄이 함수 전체를 끝낼 수 있다는 것이 보이지 않습니다.
fn print_all_v1(map: &HashMap<&str, &str>, keys: &[&str]) -> Result<(), String> {
    for key in keys {
        let value = setting!(map, key);
        println!("  {key} = {value}");
    }
    println!("  (모든 키를 확인했습니다)");
    Ok(())
}

// 권장: 실패를 값으로 돌려주는 함수입니다.
fn setting<'a>(map: &HashMap<&str, &'a str>, key: &str) -> Result<&'a str, String> {
    map.get(key)
        .copied()
        .ok_or_else(|| format!("`{key}` 설정이 없습니다"))
}

// 부르는 쪽이 흐름을 고릅니다. 여기서는 빠진 키를 알리고 다음 키로 넘어갑니다.
fn print_all_v2(map: &HashMap<&str, &str>, keys: &[&str]) {
    for key in keys {
        match setting(map, key) {
            Ok(value) => println!("  {key} = {value}"),
            Err(e) => println!("  {e} (건너뜀)"),
        }
    }
    println!("  (모든 키를 확인했습니다)");
}

// 빠져나가는 것이 맞는 곳에서는 `?`를 씁니다. 같은 동작이지만 줄 끝의 `?`가 그 사실을 보여 줍니다.
fn server_address(map: &HashMap<&str, &str>) -> Result<String, String> {
    let host = setting(map, "host")?;
    let port = setting(map, "port")?;
    Ok(format!("{host}:{port}"))
}

fn main() {
    let map = parse("host=127.0.0.1\nname=demo\n");
    let keys = ["host", "port", "name"];

    println!("1. 비권장: 매크로 안의 숨은 return");
    let result = print_all_v1(&map, &keys);
    // 출력:   host = 127.0.0.1
    println!("  돌려받은 값: {result:?}"); // 출력:   돌려받은 값: Err("`port` 설정이 없습니다")

    println!("2. 권장: Result를 돌려주는 함수, 흐름은 부르는 쪽이 고름");
    print_all_v2(&map, &keys);
    // 출력:   host = 127.0.0.1
    // 출력:   `port` 설정이 없습니다 (건너뜀)
    // 출력:   name = demo
    // 출력:   (모든 키를 확인했습니다)
    println!("  server_address: {:?}", server_address(&map)); // 출력:   server_address: Err("`port` 설정이 없습니다")
}
