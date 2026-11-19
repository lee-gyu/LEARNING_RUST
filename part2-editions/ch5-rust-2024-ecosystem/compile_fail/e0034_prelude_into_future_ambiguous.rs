// 기대하는 에러: E0034
// 핵심 메시지: multiple applicable items in scope — multiple `into_future` found
// 에디션: 2024에서 실패 / 2015·2018·2021에서 통과(실행하면 "작업으로 포장됨" 출력)
//
// 2024 에디션 프렐류드에 IntoFuture가 들어오면서, 같은 이름의 메서드를 가진 사용자 트레이트와
// 부딪힙니다. 모든 퓨처는 IntoFuture를 구현하고(자기 자신을 돌려주는 포괄 구현), 두 메서드가 모두
// self를 값으로 받으므로 메서드 탐색의 같은 단계에서 둘 다 찾아집니다.
// 2021 에디션까지는 IntoFuture가 스코프에 없어서 IntoTask::into_future 하나뿐이었습니다.
// 고치려면 IntoTask::into_future(ready)처럼 트레이트 이름을 경로에 적습니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0034_prelude_into_future_ambiguous.rs --out-dir /tmp/rust-p2-ch5   → 에러
//   rustc --edition 2021 compile_fail/e0034_prelude_into_future_ambiguous.rs --out-dir /tmp/rust-p2-ch5   → 통과

use std::future::{Future, Ready};

/// IntoFuture가 안정화(1.64)되기 전에 만든 확장 트레이트라고 생각합니다.
trait IntoTask: Sized {
    fn into_future(self) -> (&'static str, Self) {
        ("작업으로 포장됨", self)
    }
}

impl<F: Future> IntoTask for F {}

fn main() {
    let ready: Ready<i32> = std::future::ready(7);
    let (label, _fut) = ready.into_future(); // → 2024 에디션에서 E0034
    println!("{}", label);
}
