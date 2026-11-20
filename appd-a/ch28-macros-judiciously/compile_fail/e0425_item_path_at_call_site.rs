// 기대하는 에러: E0425 cannot find function `record` in this scope
//   ... in this macro invocation (handlers::login 안의 호출)
//   help: consider importing this function: use crate::metrics::record;
//
// 위생은 지역 변수와 레이블에만 적용됩니다. 함수 같은 항목의 이름은 매크로를 "부른 곳"에서 찾습니다.
// `metrics` 모듈 안에서 부르면 `record`가 보이므로 컴파일되지만, `handlers` 모듈에서 부르면 찾지 못합니다.
// 컴파일러의 help(부르는 곳에 use를 더하라)는 이 호출 하나만 고칩니다. 매크로를 쓰는 모든 곳, 다른 크레이트까지
// 같은 use를 요구하게 되므로 따르지 않습니다. 매크로 본문을 `$crate::metrics::record($event)`로 고칩니다
// (examples/08_dollar_crate.rs).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0425_item_path_at_call_site.rs

mod metrics {
    pub fn record(event: &str) {
        println!("{event}");
    }

    #[macro_export]
    macro_rules! track {
        ($event:expr) => {
            record($event)
        };
    }

    pub fn startup() {
        track!("startup"); // 같은 모듈 안: 컴파일됩니다.
    }
}

mod handlers {
    pub fn login() {
        crate::track!("login"); // 다른 모듈: E0425
    }
}

fn main() {
    metrics::startup();
    handlers::login();
}
