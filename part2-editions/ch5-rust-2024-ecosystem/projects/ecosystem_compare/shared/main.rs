// 같은 소스 파일을 e2021(2021 에디션)과 e2024(2024 에디션) 두 패키지가 함께 컴파일합니다.
// 두 에디션 모두에서 컴파일되지만, 1·2번 구역은 출력이 달라지고 3번 구역은 같습니다.
//
// 실행(projects/ecosystem_compare 디렉터리에서):
//   cargo run -q --bin ecosystem_2021
//   cargo run -q --bin ecosystem_2024
//
// 이 파일은 두 패키지가 함께 쓰므로 서식을 정하는 패키지가 둘입니다. cargo fmt -p e2021은
// 2021 스타일로, cargo fmt -p e2024는 2024 스타일로 서식하고, -p 없이 실행하면 멤버 목록에서 먼저
// 나오는 e2021 쪽으로 한 번만 서식합니다. 그래서 두 스타일에서 서식이 같은 코드만 썼습니다
// (README 연습 3).

// 2021 에디션으로 컴파일할 때 2번 구역이 일부러 2021의 동작을 보여 주느라 생기는 경고만 막습니다.
// boxed_slice_into_iter는 "이 호출의 뜻이 2024에서 바뀐다"는 경고이고, 2024 에디션에서는 아무것도
// 찾지 못합니다. 그래서 #[expect]가 아니라 #[allow]를 썼습니다(#[expect]는 e2024 쪽에서 경고).
#![allow(boxed_slice_into_iter)]

use std::any::type_name_of_val;
use std::env;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

const DEMO_KEY: &str = "P2CH5_DEMO";

/// 무엇이든 "들여다보는" 확장 트레이트라고 생각합니다. 포괄 구현이라 퓨처에도 구현됩니다.
/// 2021 에디션 시절에 만든 코드라서 이름이 Future::poll과 겹친다는 것을 몰랐습니다.
trait Peek {
    fn poll(&self, _cx: &mut Context<'_>) -> Poll<()> {
        println!("    → Peek::poll 실행");
        Poll::Pending
    }
}

impl<T> Peek for T {}

fn main() {
    // SAFETY: main의 첫머리라 아직 main 스레드 하나뿐이고, 이 프로그램은 스레드를 만들지 않습니다.
    // 2021 에디션에서는 unsafe 블록이 없어도 되지만, 있어도 unused_unsafe 경고가 나지 않습니다.
    unsafe {
        env::set_var(DEMO_KEY, "설정됨");
    }

    println!("[{}]", env!("CARGO_PKG_NAME"));

    println!("1. 프렐류드의 Future와 이름이 겹치는 메서드");
    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = pin!(async {
        println!("    → async 본문 실행");
    });
    let by_method = fut.as_mut().poll(&mut cx);
    println!("    fut.as_mut().poll(&mut cx) = {by_method:?}");
    let by_path = Peek::poll(&fut.as_mut(), &mut cx);
    println!("    Peek::poll(..)             = {by_path:?}");

    println!("2. Box<[T]>에 .into_iter()를 부르면");
    let scores: Box<[u32]> = vec![70, 85, 90].into_boxed_slice();
    let first = scores.into_iter().next();
    println!("    첫 원소: {first:?}, {}", type_name_of_val(&first));

    println!("3. unsafe 블록 안에서 부른 set_var (두 에디션이 같음)");
    println!("    {DEMO_KEY} = {:?}", env::var(DEMO_KEY));
}
