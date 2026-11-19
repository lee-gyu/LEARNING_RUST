// 01_prelude_future: 2024 에디션 프렐류드에 들어온 Future와 IntoFuture
//
// 2024 에디션 크레이트에는 std::prelude::rust_2024가 자동으로 들어오고, 이 목록에는 2021 프렐류드에
// 없던 Future와 IntoFuture가 있습니다. 그래서 이 파일에는 std::future를 가져오는 use가 없습니다.
// 같은 파일을 2021 에디션으로 컴파일하면 두 트레이트를 이름으로 적은 곳마다 E0405
// (cannot find trait ... in this scope), 경로로 부른 `IntoFuture::into_future`에서 E0433이 납니다.
//
// 트레이트가 프렐류드에 들어오면 그 메서드(poll, into_future)도 모든 모듈에서 메서드 탐색의 후보가
// 됩니다. 2·3번 구역은 같은 이름의 메서드를 가진 사용자 트레이트가 2024 에디션에서 어떻게 되는지
// 보여 줍니다. 2021 에디션과의 비교는 projects/ecosystem_compare의 1번 구역에서 실행해 봅니다.
//
// 실행: cargo run -p p2-ch5-rust-2024-ecosystem --example 01_prelude_future

use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

// --- 1번 구역에서 쓰는 것들 ---

// 종류가 다른 퓨처를 한 목록에 담으려면 트레이트 객체 `dyn Future`로 상자에 넣습니다.
// 타입에 `Future`를 이름으로 적지만, 2024 에디션이라 use 없이 씁니다.
fn jobs() -> Vec<Pin<Box<dyn Future<Output = u32>>>> {
    vec![Box::pin(async { 40 + 2 }), Box::pin(std::future::ready(7))]
}

// 설정을 메서드로 이어 붙인 뒤 .await 한 번으로 실행하는 빌더입니다(Part 1 ch13 2.8절).
// `.await`는 먼저 IntoFuture::into_future를 부르므로, 빌더 자체는 퓨처가 아니어도 됩니다.
struct Order {
    menu: &'static str,
    shots: u32,
}

impl Order {
    fn new(menu: &'static str) -> Self {
        Order { menu, shots: 1 }
    }

    fn shots(mut self, shots: u32) -> Self {
        self.shots = shots;
        self
    }
}

impl IntoFuture for Order {
    type Output = String;
    type IntoFuture = std::future::Ready<String>;

    fn into_future(self) -> Self::IntoFuture {
        std::future::ready(format!("{} (샷 {}개)", self.menu, self.shots))
    }
}

// 아무 IntoFuture나 받아 한 번만 폴링합니다. 실행기 없이 결과를 보려는 장치입니다.
// 이 예제의 퓨처는 모두 첫 폴링에서 끝나므로 Pending이 나오지 않습니다.
fn poll_once<F: IntoFuture>(f: F) -> Poll<F::Output> {
    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = pin!(f.into_future());
    fut.as_mut().poll(&mut cx)
}

// --- 2번 구역: &self로 poll을 받는 사용자 트레이트 ---

// 무엇이든 "들여다보는" 확장 트레이트라고 생각합니다. 포괄 구현이라 퓨처에도 구현됩니다.
trait Peek {
    fn poll(&self, _cx: &mut Context<'_>) -> Poll<()> {
        println!("    Peek::poll 실행");
        Poll::Pending
    }
}

impl<T> Peek for T {}

// --- 3번 구역: self로 into_future를 받는 사용자 트레이트 ---

// IntoFuture가 안정화(1.64)되기 전에 만든 확장 트레이트라고 생각합니다.
trait IntoTask: Sized {
    fn into_future(self) -> (&'static str, Self) {
        ("작업으로 포장됨", self)
    }
}

impl<F: Future> IntoTask for F {}

fn main() {
    println!("1. Future와 IntoFuture를 use 없이 씁니다");
    let results: Vec<Poll<u32>> = jobs().into_iter().map(poll_once).collect();
    println!("    jobs()                 → {results:?}"); // 출력: jobs()                 → [Ready(42), Ready(7)]
    let order = async { Order::new("라테").shots(2).await };
    println!("    Order 빌더를 .await    → {:?}", poll_once(order)); // 출력: Order 빌더를 .await    → Ready("라테 (샷 2개)")

    println!("2. &self로 받는 poll: 메서드 문법은 Future::poll을 부릅니다");
    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = pin!(async { println!("    async 본문 실행") });
    // 수신자 Pin<&mut _>를 그대로 받는 Future::poll이 첫 단계에서 찾아지므로,
    // 자동 참조(&)가 필요한 Peek::poll까지 가지 않습니다. 2021 에디션에서는 Future가 스코프에
    // 없어서 Peek::poll이 불렸습니다.
    let by_method = fut.as_mut().poll(&mut cx); // 출력: async 본문 실행
    println!("    fut.as_mut().poll(&mut cx)     = {by_method:?}"); // 출력: fut.as_mut().poll(&mut cx)     = Ready(())
    // 사용자 트레이트의 메서드를 부르려면 트레이트 이름을 적습니다.
    let by_path = Peek::poll(&fut.as_mut(), &mut cx); // 출력: Peek::poll 실행
    println!("    Peek::poll(&fut.as_mut(), ..)  = {by_path:?}"); // 출력: Peek::poll(&fut.as_mut(), ..)  = Pending

    println!("3. self로 받는 into_future: 메서드 문법은 모호해서 에러(E0034)입니다");
    // `std::future::ready(7).into_future()`라고 쓰면 IntoFuture와 IntoTask가 같은 단계의 후보라서
    // E0034입니다(compile_fail/e0034_prelude_into_future_ambiguous.rs). 트레이트 이름으로 고릅니다.
    let (label, inner) = IntoTask::into_future(std::future::ready(7));
    println!(
        "    IntoTask::into_future(..)   → {label}, {:?}",
        poll_once(inner)
    ); // 출력: IntoTask::into_future(..)   → 작업으로 포장됨, Ready(7)
    let plain = IntoFuture::into_future(std::future::ready(7));
    println!("    IntoFuture::into_future(..) → {:?}", poll_once(plain)); // 출력: IntoFuture::into_future(..) → Ready(7)
}
