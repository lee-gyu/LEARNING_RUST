// Context와 Waker: 웨이커가 무엇을 하는 물건인지 직접 만들어 봅니다.
// std::task::Wake 트레이트를 Arc<T>에 구현하면 unsafe 코드 없이 웨이커를 만들 수 있습니다.
// 이 예제의 웨이커는 "깨워졌다"는 연락을 받을 때마다 횟수만 셉니다.
//
// 실행: cargo run -p p1-ch13-async-primitives --example 02_waker_and_wake
//
// 4번 구역은 스코프 스레드에서 웨이커를 부르지만, 횟수는 모든 스레드가 끝난 뒤에 출력하므로
// 출력은 실행할 때마다 같습니다.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Wake, Waker};
use std::thread;

/// 깨워진 횟수를 세는 웨이커의 "알맹이". 실제 실행기라면 여기에 태스크를 대기열에 다시 넣거나
/// 잠든 스레드를 깨우는 코드가 들어갑니다(04, 08 예제).
struct CountingWaker {
    name: &'static str,
    wake_calls: AtomicUsize,        // wake(self)로 깨운 횟수
    wake_by_ref_calls: AtomicUsize, // wake_by_ref(&self)로 깨운 횟수
}

impl CountingWaker {
    fn new(name: &'static str) -> Arc<Self> {
        Arc::new(CountingWaker {
            name,
            wake_calls: AtomicUsize::new(0),
            wake_by_ref_calls: AtomicUsize::new(0),
        })
    }

    fn report(&self) -> String {
        format!(
            "{}: wake {}번, wake_by_ref {}번",
            self.name,
            self.wake_calls.load(Ordering::SeqCst),
            self.wake_by_ref_calls.load(Ordering::SeqCst)
        )
    }
}

// Wake의 메서드는 self를 Arc<Self>로 받습니다. 웨이커는 여러 곳에 복제되고 다른 스레드로도
// 건너가므로, 알맹이를 Arc로 공유해야 하기 때문입니다.
impl Wake for CountingWaker {
    // 꼭 구현해야 하는 메서드입니다. Arc 하나를 소비합니다.
    fn wake(self: Arc<Self>) {
        self.wake_calls.fetch_add(1, Ordering::SeqCst);
    }

    // 기본 구현은 Arc를 복제한 뒤 wake를 부릅니다. 복제 없이 처리할 수 있으면 재정의합니다.
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_by_ref_calls.fetch_add(1, Ordering::SeqCst);
    }
}

fn main() {
    println!("1. Arc<T: Wake>를 Waker로 바꿉니다");
    let counter = CountingWaker::new("A");
    // From<Arc<W>> for Waker (W: Wake + Send + Sync + 'static). 변환 자체는 할당 없이 Arc를 옮깁니다.
    let waker = Waker::from(Arc::clone(&counter));
    println!("    Arc의 소유자 수: {}", Arc::strong_count(&counter)); // 출력: 2 (counter, waker)

    waker.wake_by_ref(); // 웨이커를 남겨 둔 채 깨웁니다.
    waker.wake_by_ref();
    println!("    {}", counter.report()); // 출력: A: wake 0번, wake_by_ref 2번

    let copy = waker.clone(); // 웨이커 복제 = 안쪽 Arc 복제
    println!("    clone 뒤 소유자 수: {}", Arc::strong_count(&counter)); // 출력: 3
    copy.wake(); // wake는 웨이커를 소비합니다. 그 웨이커가 들고 있던 Arc도 함께 사라집니다.
    println!("    {}", counter.report()); // 출력: A: wake 1번, wake_by_ref 2번
    println!("    wake 뒤 소유자 수: {}", Arc::strong_count(&counter)); // 출력: 2

    println!("2. Context는 웨이커를 빌려 담는 상자입니다");
    let cx = Context::from_waker(&waker);
    // poll 안에서는 cx.waker()로 웨이커를 꺼내 저장하거나 깨웁니다.
    cx.waker().wake_by_ref();
    println!("    {}", counter.report()); // 출력: A: wake 1번, wake_by_ref 3번

    println!("3. will_wake: 두 웨이커가 같은 태스크를 깨우는가");
    let other = Waker::from(CountingWaker::new("B"));
    let same_arc = Waker::from(Arc::clone(&counter));
    println!(
        "    waker와 clone:           {}",
        waker.will_wake(&waker.clone())
    ); // 출력: true
    println!(
        "    같은 Arc로 따로 만든 것: {}",
        waker.will_wake(&same_arc)
    ); // 출력: true
    println!("    다른 Arc(B):             {}", waker.will_wake(&other)); // 출력: false
    println!(
        "    Waker::noop():           {}",
        waker.will_wake(Waker::noop())
    ); // 출력: false

    println!("4. Waker는 Send + Sync라서 어느 스레드에서든 깨울 수 있습니다");
    let threaded = CountingWaker::new("C");
    let waker_c = Waker::from(Arc::clone(&threaded));
    thread::scope(|s| {
        for _ in 0..4 {
            let w = waker_c.clone(); // 스레드마다 복제본을 옮겨 줍니다.
            s.spawn(move || w.wake());
        }
        s.spawn(|| waker_c.wake_by_ref()); // &Waker를 빌려 써도 됩니다(Sync).
    });
    println!("    {}", threaded.report()); // 출력: C: wake 4번, wake_by_ref 1번

    // 웨이커를 모두 버리면 Arc의 소유자도 줄어듭니다.
    drop(waker);
    drop(same_arc);
    println!(
        "    웨이커를 버린 뒤 A의 소유자 수: {}",
        Arc::strong_count(&counter)
    ); // 출력: 1
}
