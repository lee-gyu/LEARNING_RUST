// Pin과 Unpin: 고정이 왜 필요하고, 언제 아무 제약도 아닌지 확인합니다.
// Unpin 타입은 Pin::new와 get_mut으로 자유롭게 다루고, Unpin이 아닌 async 블록은 pin!(스택)이나
// Box::pin(힙)으로 고정한 뒤 as_mut()으로 여러 번 폴링합니다. 마지막으로 서로 다른 퓨처를
// Pin<Box<dyn Future<Output = T>>> 하나의 타입으로 다룹니다.
//
// 실행: cargo run -p p1-ch13-async-primitives --example 06_pin_unpin
//
// 메모리 주소 자체는 실행마다 다르므로 출력하지 않고, 주소를 비교한 결과(true/false)만 출력합니다.

use std::cell::Cell;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

/// 처음 한 번은 Pending(스스로 깨움), 그다음에 Ready를 돌려주는 퓨처. 필드가 bool뿐이라 Unpin입니다.
struct YieldOnce {
    yielded: bool,
}

fn yield_once() -> YieldOnce {
    YieldOnce { yielded: false }
}

impl Future for YieldOnce {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// 고정된 퓨처를 Ready가 될 때까지 폴링하고 결과와 폴링 횟수를 돌려줍니다.
/// 이 예제의 퓨처는 모두 스스로 깨우므로 noop 웨이커로 계속 폴링해도 됩니다.
/// F: ?Sized라서 Pin<&mut dyn Future<Output = T>>도 받습니다.
fn poll_to_end<F: Future + ?Sized>(mut future: Pin<&mut F>) -> (F::Output, u32) {
    let mut cx = Context::from_waker(Waker::noop());
    let mut polls = 0;
    loop {
        polls += 1;
        // poll은 Pin<&mut F>를 값으로 받아 소비합니다. as_mut()은 future를 다시 빌린
        // 새 Pin<&mut F>를 만들어 주므로 반복문에서 여러 번 부를 수 있습니다.
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return (output, polls);
        }
    }
}

/// 컴파일러가 T: Unpin을 확인해 주는 함수. Unpin이 아닌 타입을 넘기면 E0277입니다.
fn assert_unpin<T: Unpin>(_: &T) {}

async fn numbered(n: u32) -> u32 {
    yield_once().await;
    n * 10
}

/// 조건에 따라 서로 다른 async 블록을 돌려줍니다. 블록마다 타입이 다르므로 힙에 고정하고
/// 트레이트 객체로 바꿔 하나의 반환 타입으로 맞춥니다.
fn greeting(korean: bool) -> Pin<Box<dyn Future<Output = String>>> {
    if korean {
        Box::pin(async { String::from("안녕하세요") })
    } else {
        Box::pin(async {
            yield_once().await;
            String::from("hello")
        })
    }
}

/// 재귀 async fn. 자기 자신의 퓨처를 품으면 크기가 무한해지므로 Box::pin으로 한 단계 간접 참조를
/// 둡니다(E0733, README 4.4절).
async fn sum_to(n: u64) -> u64 {
    if n == 0 {
        0
    } else {
        n + Box::pin(sum_to(n - 1)).await
    }
}

fn main() {
    let mut cx = Context::from_waker(Waker::noop());

    println!("1. Unpin 타입에게 Pin은 아무 제약도 아닙니다");
    let mut n = 5;
    let mut pinned: Pin<&mut i32> = Pin::new(&mut n); // i32: Unpin이라 Pin::new 가능
    *pinned += 1; // Unpin이면 DerefMut으로 바로 고칠 수 있습니다.
    let plain: &mut i32 = pinned.get_mut(); // 평범한 &mut로 되돌리기도 자유롭습니다.
    *plain += 1;
    println!("    n = {n}"); // 출력: n = 7

    let mut first = yield_once();
    let poll = Pin::new(&mut first).poll(&mut cx);
    let mut moved = first; // 폴링한 뒤에 다른 변수로 옮깁니다. Unpin이므로 괜찮습니다.
    let again = Pin::new(&mut moved).poll(&mut cx);
    println!("    YieldOnce: 옮기기 전 {poll:?}, 옮긴 뒤 {again:?}"); // 출력: Pending, Ready(())

    assert_unpin(&n);
    assert_unpin(&String::new());
    assert_unpin(&moved);
    let boxed_block = Box::pin(async {});
    assert_unpin(&boxed_block); // 안의 async 블록은 Unpin이 아니어도 Pin<Box<_>>는 Unpin
    println!("    i32, String, YieldOnce, Pin<Box<async 블록>>은 Unpin입니다 (컴파일로 확인)");

    println!("2. Unpin이 아닌 async 블록 고정하기: pin!(스택)과 Box::pin(힙)");
    let mut on_stack = pin!(numbered(1));
    let (value, polls) = poll_to_end(on_stack.as_mut());
    println!("    pin!:     값 {value}, 폴링 {polls}번"); // 출력: 값 10, 폴링 2번

    // set: 고정된 자리의 퓨처를 같은 타입의 새 퓨처로 바꿉니다. 옛 퓨처는 그 자리에서 버려집니다.
    on_stack.set(numbered(2));
    let (value, _) = poll_to_end(on_stack.as_mut());
    println!("    set 뒤:   값 {value}"); // 출력: 값 20

    let mut on_heap = Box::pin(numbered(3));
    let before = on_heap.as_mut().poll(&mut cx);
    let address_before = &*on_heap as *const _ as usize; // 힙에 있는 퓨처의 주소
    let mut moved_box = on_heap; // Pin<Box<_>>를 옮겨도 옮겨지는 것은 Box 포인터뿐입니다.
    let address_after = &*moved_box as *const _ as usize;
    let after = moved_box.as_mut().poll(&mut cx);
    println!(
        "    Box::pin: 첫 poll {before:?}, 옮긴 뒤 poll {after:?}, 퓨처 주소가 그대로인가: {}",
        address_before == address_after
    ); // 출력: Pending, Ready(30), true

    println!("3. 왜 고정해야 하나: await 너머로 들고 가는 참조는 퓨처 자신을 가리킵니다");
    let data_address = Cell::new(0usize);
    let mut future = Box::pin(async {
        let data = [7u8; 32];
        let first = &data[0]; // data를 가리키는 참조
        data_address.set(first as *const u8 as usize);
        yield_once().await; // 이 await 지점 너머로 first를 들고 갑니다.
        *first
    });
    let poll = future.as_mut().poll(&mut cx); // 첫 poll에서 data가 퓨처 안에 만들어집니다.
    let start = &*future as *const _ as usize;
    let end = start + size_of_val(&*future);
    println!(
        "    첫 poll: {poll:?}, data가 퓨처의 메모리 안에 있는가: {}",
        (start..end).contains(&data_address.get())
    ); // 출력: Pending, true
    // 이 퓨처를 다른 곳으로 옮긴다면 first는 옛 자리를 가리키게 됩니다. 그래서 async 블록은
    // Unpin이 아니고, 고정된 뒤에만 poll할 수 있습니다.
    println!("    두 번째 poll: {:?}", future.as_mut().poll(&mut cx)); // 출력: Ready(7)

    println!("4. Pin<Box<dyn Future<Output = T>>>: 서로 다른 퓨처를 한 타입으로");
    let mut greetings = [greeting(true), greeting(false)];
    for g in &mut greetings {
        // g: &mut Pin<Box<dyn Future<Output = String>>>, as_mut()은 Pin<&mut dyn Future<..>>
        let (text, polls) = poll_to_end(g.as_mut());
        println!("    {text} (폴링 {polls}번)");
    }
    // 출력: 안녕하세요 (폴링 1번), hello (폴링 2번)
    let (total, _) = poll_to_end(pin!(sum_to(100)));
    println!("    재귀 async fn sum_to(100) = {total}"); // 출력: 5050
}
