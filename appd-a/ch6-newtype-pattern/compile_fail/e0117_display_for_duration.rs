// 기대하는 에러: E0117 — only traits defined in the current crate can be implemented for types
//                          defined outside of the crate
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0117_display_for_duration.rs --out-dir /tmp/er-ch6
//
// `Display`(표준 라이브러리의 트레이트)도 `Duration`(표준 라이브러리의 타입)도 이 크레이트의 것이
// 아니므로 고아 규칙에 걸립니다. 표준 라이브러리가 나중에 `impl Display for Duration`을 추가하거나
// 다른 크레이트가 같은 구현을 하면 어느 쪽을 쓸지 정할 수 없기 때문입니다.
// 해결: 에러의 마지막 note가 말하는 두 길 가운데 하나를 고릅니다.
//   - new type: `struct Elapsed(Duration);`에 `Display`를 구현합니다. `Display`를 요구하는 API에
//     넘겨야 할 때는 이 길뿐입니다(examples/05_orphan_rule_wrapper.rs의 권장 판).
//   - trait: 내 트레이트를 정의해 `Duration`에 구현합니다. 내 코드에서 직접 부를 때만 충분합니다.

use std::fmt;
use std::time::Duration;

impl fmt::Display for Duration {
    // E0117
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}초", self.as_secs())
    }
}

fn main() {
    println!("{}", Duration::from_secs(90));
}
