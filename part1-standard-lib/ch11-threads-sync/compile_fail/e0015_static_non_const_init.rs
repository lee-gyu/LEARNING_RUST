// 기대하는 에러: E0015
// 핵심 메시지: cannot call non-const associated function `HashMap::<String, u32>::new` in statics
//
// static의 초기값은 컴파일할 때 계산되어야 합니다. Mutex::new는 const fn이지만
// HashMap::new는 const fn이 아니므로 static의 초기값 안에서 부를 수 없습니다.
//
// 확인: rustc --edition 2024 compile_fail/e0015_static_non_const_init.rs --out-dir /tmp/rust-p1-ch11

use std::collections::HashMap;
use std::sync::Mutex;

static PRICES: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new()); // → E0015

fn main() {
    PRICES.lock().unwrap().insert(String::from("커피"), 4500);
    println!("{:?}", PRICES.lock().unwrap().get("커피"));
}
