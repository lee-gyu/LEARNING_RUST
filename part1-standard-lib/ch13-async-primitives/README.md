# ch13. 비동기 부품: `Future`와 `Pin`

[Part 0 ch16](../../part0-getting-started/ch16-async/README.md)에서는 `trpl` 크레이트의 런타임 위에서 `async`/`.await`를 썼고, `Future`·`Pin`·`Unpin`은 "읽을 수 있을 정도"로만 들여다봤습니다. 그 뒤로 여러 챕터가 비동기의 밑바닥을 이 챕터로 미뤄 두었습니다.

- Part 0 ch16: 퓨처와 실행기를 직접 만들기(2.11절에서 읽기만 한 `Future::poll`, `Pin`, `Unpin`을 실제로 구현하고 쓰기)
- [Part 1 ch6](../ch6-core-traits/README.md): 자동 트레이트로서의 `Unpin`
- [Part 1 ch7](../ch7-smart-pointers-memory/README.md): 고정된 위치가 필요한 값을 위한 `Box::pin`
- [Part 1 ch11](../ch11-threads-sync/README.md): 비동기 실행기를 만들 때 쓰는 `thread::park`/`unpark`

이 챕터의 줄기는 **`.await`가 실제로 하는 일을 손으로 한 번 만들어 보는 것**입니다. 외부 크레이트 없이 표준 라이브러리만으로 퓨처를 손으로 폴링하고(2.2절), 웨이커를 만들고(2.3절), 퓨처를 직접 구현하고(2.4·2.6·2.8절), 고정을 이해하고(2.7절), 마지막에는 퓨처를 실행하는 실행기를 두 가지 만듭니다(2.5·2.9절). 2.10절에서 우리가 만든 것과 실제 런타임을 비교하며 Part 0 ch16으로 돌아갑니다.

- 패키지: `p1-ch13-async-primitives` (의존성 없음)
- 기준 문서: [`std::future`](https://doc.rust-lang.org/std/future/index.html), [`std::task`](https://doc.rust-lang.org/std/task/index.html), [`std::pin`](https://doc.rust-lang.org/std/pin/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024), macOS. 예제에 unsafe 코드는 없습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `Future` 트레이트의 시그니처를 읽고 `Poll::Ready`/`Poll::Pending`의 뜻을 설명할 수 있다. `Waker::noop()`과 `pin!`으로 퓨처를 손으로 폴링하고, "`Pending`을 돌려주려면 깨울 준비를 먼저 해야 한다"와 "끝난 퓨처를 다시 폴링하지 않는다"는 계약을 설명할 수 있다.
2. 웨이커가 무엇을 하는지 설명하고, `std::task::Wake`를 `Arc<T>`에 구현해 unsafe 없이 웨이커를 만들 수 있다. `wake`/`wake_by_ref`/`clone`/`will_wake`를 구별해 쓸 수 있다.
3. `Future`를 직접 구현할 수 있다. 스스로 깨우는 퓨처, 다른 스레드가 공유 상태와 저장해 둔 웨이커로 완료시키는 퓨처(타이머, 일회용 채널), 두 퓨처를 함께 기다리는 `join`을 만들고, `async fn`이 만드는 상태 기계와 손으로 쓴 `poll`을 비교할 수 있다.
4. `Pin`이 왜 필요한지(자기 참조 상태 기계)와 `Unpin`인 타입에는 아무 제약도 아니라는 것을 설명할 수 있다. `Unpin`이 아닌 퓨처를 `pin!`이나 `Box::pin`으로 고정해 `as_mut()`으로 여러 번 폴링하고, `Pin<Box<dyn Future<Output = T>>>`를 읽고 쓸 수 있다.
5. `park`/`unpark`로 `block_on`을, 채널을 태스크 대기열로 써서 여러 태스크를 돌리는 실행기를 만들 수 있다. 깨우지 않으면 멈추고 쉬지 않고 깨우면 바쁜 폴링이 된다는 것, 그리고 실제 런타임이 이것과 무엇이 다른지 설명할 수 있다.

## 2. 핵심 개념

### 2.1 이 챕터의 지도

Part 0 ch16에서 `trpl::block_on(async { work().await })`라고 쓰면 일어나던 일을 부품으로 나누면 다음과 같습니다.

```text
실행기(block_on, 2.5절)                    퓨처(async 블록, 직접 구현한 타입)
  │  ① 퓨처를 고정(Pin, 2.7절)하고
  │  ② 웨이커(2.3절)를 Context에 담아 poll ───────►  진행할 수 있는 만큼 진행
  │                                                 ├─ 끝났으면 Poll::Ready(값)
  │  ◄──────────────────────────────────────────────┤
  │                                                 └─ 아직이면 웨이커를 맡겨 두고 Poll::Pending
  │  ③ Pending이면 잠듦
  │  ◄──── ④ 기다리던 일이 끝나면 누군가(타이머 스레드 등) 웨이커로 깨움
  └─ ⑤ 다시 poll
```

| 부품 | 표준 라이브러리의 항목 | 절 | 예제 |
| --- | --- | --- | --- |
| 퓨처와 결과 | `Future`, `Poll` | 2.2 | 01 |
| 깨우는 연락처 | `Context`, `Waker`, `Wake` | 2.3 | 02 |
| 퓨처 구현 | `impl Future for ..` | 2.4, 2.6, 2.8 | 03, 05, 07 |
| 실행기 | (직접 만듦) `thread::park`/`unpark`, 채널 | 2.5, 2.9 | 04, 08 |
| 고정 | `Pin`, `pin!`, `Box::pin`, `Unpin` | 2.7 | 06 |
| 도구 | `ready`, `pending`, `poll_fn`, `task::ready!`, `IntoFuture` | 2.8 | 01, 07 |

### 2.2 `Future` 트레이트 해부

[Part 1 ch1](../ch1-std-overview/README.md)에서 익힌 방법대로 시그니처를 한 조각씩 읽습니다.

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

- `type Output`: 퓨처가 끝나면 내놓는 값의 타입(연관 타입)입니다. `async fn f() -> u32`의 퓨처는 `Output = u32`입니다.
- `self: Pin<&mut Self>`: 첫 번째 매개변수 `self`에 타입을 직접 적은 형태입니다. "고정된 퓨처에 대한 가변 참조"로만 부를 수 있다는 뜻이며, 왜 그런지는 2.7절에서 봅니다.
- `cx: &mut Context<'_>`: 이 퓨처를 실행 중인 태스크의 웨이커를 담은 상자입니다(2.3절).
- 반환값 `Poll<Output>`: 끝났으면 `Ready(값)`, 아직이면 `Pending`입니다. `Iterator::next`가 `Option`으로 "다음 값 / 끝"을 알렸다면 `poll`은 `Poll`로 "끝(값) / 아직"을 알립니다. `Poll`에는 `is_ready`, `is_pending`, `map` 같은 메서드가 있고 `#[must_use]`라서 결과를 버리면 경고가 납니다(4.6절).

**손으로 폴링하기.** 실행기 없이도 `poll`을 직접 부를 수 있습니다([`01_poll_by_hand.rs`](examples/01_poll_by_hand.rs)).

```rust
let mut cx = Context::from_waker(Waker::noop()); // 연락을 받아도 아무 일도 하지 않는 웨이커(1.85.0)
let mut fut = pin!(async { 1 + 2 });             // async 블록은 고정해야 poll할 수 있습니다
let poll = fut.as_mut().poll(&mut cx);           // Ready(3)
```

`01`의 3번 구역은 Part 0 ch16 2.3절의 "퓨처는 게으르다"를 `poll` 수준에서 보여 줍니다. async 블록을 만든 직후에는 본문이 실행되지 않고, **첫 `poll`에서** 본문이 실행됩니다. 안쪽 퓨처가 바로 `Ready`인 `.await`(`future::ready(1).await`)에서는 멈추지 않고 그대로 지나갑니다. 반대로 `future::pending()`을 기다리는 블록은 첫 `poll`에서 `await 전` 줄을 출력한 뒤 `Pending`을 돌려주고, 그 뒤로 몇 번을 다시 폴링해도 본문을 처음부터 실행하지 않습니다. 퓨처는 멈춘 await 지점을 기억했다가 거기서 이어 갑니다.

**`poll`의 계약.** `Future` 문서의 "Return value"와 "Panics" 절이 정한 약속은 다음과 같습니다. 컴파일러가 검사하지 않으므로 퓨처를 구현하는 쪽과 폴링하는 쪽이 지켜야 합니다.

1. **`Pending`을 돌려줄 때는 깨울 준비를 먼저 합니다.** `cx`의 웨이커를 복제해 저장해 두고, 진행할 수 있게 되면 그 웨이커의 `wake`를 부릅니다. 실행기는 깨운 퓨처만 다시 폴링하므로, 연락을 남기지 않은 `Pending`은 영원히 잠듭니다(2.5절).
2. **여러 번 폴링되었다면 가장 최근 `poll`의 웨이커만 깨우면 됩니다**(2.6절).
3. **끝난 퓨처를 다시 폴링하지 않습니다.** 무슨 일이 일어날지는 트레이트가 정하지 않습니다(패닉, 영원히 블로킹 등). 다만 `poll`이 `unsafe`가 아니므로 미정의 동작은 일어나지 않습니다. `01`의 4번 구역에서 확인한 메시지는 async 블록이 `` `async fn` resumed after completion ``, `future::ready`가 `` `Ready` polled after completion ``입니다.
4. **`poll`은 빨리 돌아와야 하고 블로킹하면 안 됩니다.** 오래 걸리는 일은 다른 스레드에 맡기라고 문서는 권합니다.

### 2.3 `Context`와 `Waker`

**웨이커(waker)** 는 "이 태스크를 다시 폴링해 달라"고 실행기에 알리는 핸들입니다. 실행기가 웨이커를 만들어 `Context`에 담아 `poll`에 넘기고, 퓨처는 `Pending`을 돌려주기 전에 `cx.waker()`로 꺼내 저장하거나 바로 깨웁니다. 문서에 따르면 지금 `Context`가 하는 일은 빌려 담은 `&Waker`를 꺼내 주는 것뿐입니다(`Context::from_waker`로 만들고 `cx.waker()`로 꺼냄). nightly에는 `Context`에 다른 정보를 더 싣는 기능이 실험 중입니다.

| 메서드 | 하는 일 |
| --- | --- |
| `wake(self)` | 깨웁니다. 웨이커를 소비합니다. |
| `wake_by_ref(&self)` | 웨이커를 남겨 둔 채 깨웁니다. `waker.clone().wake()`보다 이쪽을 쓰라고 문서가 권하며, 그렇게 쓰면 Clippy가 `waker_clone_wake`로 알려 줍니다. |
| `clone()` | 같은 태스크를 깨우는 웨이커를 하나 더 만듭니다. 저장해 둘 때 씁니다. |
| `will_wake(&other)` | 두 웨이커가 같은 태스크를 깨우는지 확인합니다. `true`면 확실히 같지만 `false`가 "다르다"를 보장하지는 않습니다(최선 노력). |
| `clone_from(&other)` | `*w = other.clone()`과 같지만, `will_wake`가 `true`면 복제를 건너뜁니다. |
| `Waker::noop()` | 아무 일도 하지 않는 웨이커의 `&'static` 참조(1.85.0). 손으로 폴링하거나 시험할 때 씁니다. |

웨이커는 `Clone + Send + Sync`라서 실행기와 관계없는 다른 스레드에서도 부를 수 있습니다. 그래서 "블로킹 작업이 끝난 스레드가 퓨처를 깨운다"는 모양이 가능합니다(2.6절).

**웨이커 만들기: `Wake` 트레이트.** 웨이커의 실제 동작(무엇을 깨울지)은 실행기가 정합니다. 표준 라이브러리에는 두 가지 방법이 있습니다. `RawWaker`와 함수 포인터 표(`RawWakerVTable`)로 만드는 방법은 `unsafe`이고, `Wake` 트레이트(1.51.0)는 안전한 대안입니다.

```rust
pub trait Wake {
    fn wake(self: Arc<Self>);
    fn wake_by_ref(self: &Arc<Self>) { self.clone().wake(); } // 기본 구현(개념상)
}

impl<W: Wake + Send + Sync + 'static> From<Arc<W>> for Waker
```

`self`를 `Arc<Self>`로 받는 이유는 웨이커가 여러 곳에 복제되고 다른 스레드로 건너가기 때문입니다. 알맹이를 [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md)의 `Arc`로 공유하므로 `Send + Sync + 'static`도 필요합니다. `Waker::from(arc)`는 할당 없이 `Arc`를 웨이커로 옮기고, 웨이커를 복제하면 `Arc`가 복제되고 버리면 `Arc`도 버려집니다. [`02_waker_and_wake.rs`](examples/02_waker_and_wake.rs)는 깨워진 횟수만 세는 웨이커로 이것을 확인합니다. `Waker::from` 직후 소유자 수가 2, `clone` 뒤 3, `wake` 뒤 다시 2가 됩니다. 실제 실행기라면 `wake` 안에서 "잠든 스레드 깨우기"(2.5절)나 "태스크를 대기열에 다시 넣기"(2.9절)를 합니다. `wake`를 빈 몸체로 구현하면 Clippy가 `manual_noop_waker`로 `Waker::noop()`을 쓰라고 알려 줍니다.

### 2.4 퓨처 직접 구현하기

[`03_custom_future.rs`](examples/03_custom_future.rs)는 세 가지 퓨처를 만들고, 깨워진 횟수를 세는 웨이커로 손으로 폴링하는 `drive` 함수로 실행합니다.

```rust
struct Countdown { remaining: u32 }

impl Future for Countdown {
    type Output = &'static str;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.remaining == 0 {
            return Poll::Ready("발사!");
        }
        self.remaining -= 1;
        cx.waker().wake_by_ref(); // 기다릴 일이 따로 없으니 "바로 다시 불러 달라"
        Poll::Pending
    }
}
```

- **바로 끝나는 퓨처**(`Immediate`): 첫 `poll`에서 `Option::take`로 값을 꺼내 `Ready`를 돌려줍니다. 표준 라이브러리의 `future::ready`와 같은 일입니다.
- **스스로 깨우는 퓨처**(`Countdown`): `Pending`을 세 번 돌려주며 그때마다 `wake_by_ref`를 부릅니다. 차례를 다른 태스크에 한 번 양보하는 Part 0 ch16의 `yield_now`가 이런 모양입니다.
- **깨우는 것을 잊은 퓨처**(`Forgetful`): 두 번째 `poll`이면 `Ready`였겠지만, 첫 `Pending`에서 연락을 남기지 않았으므로 `drive`는 다시 폴링할 근거가 없어 멈춥니다(2.2절의 계약 1).

`mut self: Pin<&mut Self>`로 받은 뒤 `self.remaining -= 1`처럼 평범하게 필드를 고칠 수 있는 것은 `Countdown`의 필드가 모두 `Unpin`이라 `Countdown`도 `Unpin`이기 때문입니다(2.7절). `Immediate<T>`는 `T`에 따라 `Unpin`이 달라지므로 `impl<T: Unpin> Future for Immediate<T>`로 조건을 붙였습니다.

**`async fn`과 손으로 쓴 상태 기계.** Part 0 ch16 2.11절은 컴파일러가 `async` 블록을 "await 지점을 경계로 나눈 상태 기계"로 바꾼다고 했습니다. `03`의 4번 구역은 같은 일을 하는 두 코드를 나란히 실행합니다.

```rust
async fn add_slowly(a: u32, b: u32) -> u32 {
    Countdown { remaining: 1 }.await; // await 지점 1
    Countdown { remaining: 1 }.await; // await 지점 2
    a + b
}

enum AddSlowly {                       // 배리언트 = 멈춰 있는 자리
    Start { a: u32, b: u32 },
    AtFirstAwait { a: u32, b: u32, inner: Countdown },  // 필드 = 그 자리를 넘어서도 필요한 값
    AtSecondAwait { a: u32, b: u32, inner: Countdown },
    Done,
}
```

`AddSlowly::poll`은 `loop` 안에서 `match &mut *self`로 지금 상태를 보고, 진행할 수 있는 만큼 다음 상태로 넘깁니다. `.await` 자리에서는 안쪽 퓨처를 **같은 `cx`로** 폴링하고, `Pending`이면 바깥도 그대로 `Pending`을 돌려줍니다.

```rust
AddSlowly::AtFirstAwait { a, b, inner } => {
    if Pin::new(inner).poll(cx).is_pending() {
        return Poll::Pending; // 안쪽 퓨처가 남긴 연락처가 곧 바깥 태스크의 연락처
    }
    let (a, b) = (*a, *b);
    *self = AddSlowly::AtSecondAwait { a, b, inner: Countdown { remaining: 1 } };
}
```

이것이 `.await` 한 번이 하는 일의 전부입니다. 두 코드 모두 폴링 1·2번은 `Pending`, 3번은 `Ready(5)`이고 시작·중간·끝 줄이 같은 위치에 찍힙니다(3절의 출력). `Done` 상태는 끝난 뒤 다시 폴링되었을 때 패닉으로 알리기 위한 것으로, 컴파일러가 만든 퓨처도 같은 일을 합니다(2.2절의 `` `async fn` resumed after completion ``).

### 2.5 최소 실행기 `block_on`

퓨처 하나를 현재 스레드에서 끝까지 실행하는 `block_on`을 만듭니다. 재료는 [Part 1 ch11](../ch11-threads-sync/README.md)에서 이름만 본 `thread::park`/`Thread::unpark`입니다.

- `thread::park()`는 현재 스레드의 **토큰**이 생길 때까지 잠들고, 토큰을 소비하며 돌아옵니다.
- `thread.unpark()`는 그 스레드의 토큰을 만듭니다(이미 있으면 그대로, 쌓이지 않음). 그래서 **`unpark`가 `park`보다 먼저 와도 신호가 사라지지 않습니다.** `park`는 이유 없이 돌아올 수도 있다고 문서에 적혀 있습니다.

[`04_block_on.rs`](examples/04_block_on.rs)의 핵심은 다음과 같습니다.

```rust
struct ThreadWaker { thread: Thread, woken: AtomicBool }

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) { self.wake_by_ref(); }
    fn wake_by_ref(self: &Arc<Self>) {
        self.woken.store(true, Ordering::SeqCst); // 연락이 왔다는 표시를 남기고
        self.thread.unpark();                     // block_on을 부른 스레드를 깨웁니다
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let (inner, waker) = thread_waker(); // Arc<ThreadWaker>와 거기서 만든 Waker
    let mut cx = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => {
                while !inner.woken.swap(false, Ordering::SeqCst) {
                    thread::park();
                }
            }
        }
    }
}
```

`Wake` 문서의 예제는 `Pending`이면 그냥 `thread::park()`를 부르고, "단순함을 위해 정확성을 양보했다"고 적어 둡니다. 이 챕터는 `woken` 표시를 하나 더 두었습니다. `park`가 가짜로 돌아오거나 다른 코드가 `unpark`를 불러도, 표시가 없으면 다시 잠들므로 연락 없이 폴링하지 않습니다. 반대로 `poll` 안의 다른 코드(문서는 `println!`을 예로 듭니다)가 토큰을 먼저 소비해 버려도, 표시가 남아 있으므로 잠들지 않고 다시 폴링합니다. 스스로 깨우는 퓨처는 `poll` 도중에 이미 표시를 남기므로 잠들지 않고 바로 다시 폴링됩니다.

`04`는 이 실행기로 세 가지를 관찰합니다.

- **2번 구역**: `countdown(3)`은 정확히 4번 폴링됩니다. `async` 블록 안에서 `countdown(1).await`과 `countdown(2).await`을 차례로 기다리면 2 + 3번입니다.
- **3번 구역, 깨우지 않으면 멈춥니다**: `block_on`에 `Forgetful`을 넘기면 첫 `Pending` 뒤에 영원히 잠듭니다. 예제는 실제로 멈추지 않도록 `park_timeout`으로 50ms만 기다리는 `block_on_for`를 따로 만들어 `None`을 확인합니다. 프로그램이 끝나지 않고 CPU도 쓰지 않는다면, `Pending`을 돌려주면서 웨이커를 저장하지 않은 퓨처가 있는지 먼저 의심합니다.
- **4번 구역, 계속 깨우면 바쁜 폴링입니다**: 시각만 확인하고 아직이면 `wake_by_ref`를 부르는 `BusySleep`은 30ms 동안 1000번이 훨씬 넘게 폴링됩니다(이 환경의 디버그 빌드에서 64만~99만 번). 결과는 맞지만 기다리는 내내 CPU를 씁니다. 스스로 깨우기는 차례를 양보할 때 한 번 쓰는 도구이지 "기다리는" 방법이 아닙니다. 다른 스레드가 깨워 주는 타이머는 2번만 폴링됩니다(2.6절).

Part 0 ch16의 `trpl::block_on`은 tokio 런타임을 만들어 같은 일을 합니다. 퓨처를 고정하고, 웨이커를 만들어 폴링하고, `Pending`이면 잠들었다가 깨면 다시 폴링합니다.

### 2.6 다른 스레드가 완료시키는 퓨처: 공유 상태와 저장해 둔 웨이커

진짜로 무언가를 "기다리는" 퓨처는 다음 모양입니다. [`05_timer_and_oneshot.rs`](examples/05_timer_and_oneshot.rs)의 `Sleep`으로 봅니다.

```text
퓨처의 poll                                     일을 끝낸 다른 스레드
  잠금 → done이면 Ready                         잠금 → done = true
       → 아니면 cx의 웨이커를 저장 → Pending          → 저장된 웨이커를 꺼냄(take)
                                                잠금 해제 → waker.wake()
```

```rust
struct TimerState { done: bool, waker: Option<Waker> }

struct Sleep {
    duration: Duration,
    state: Option<Arc<Mutex<TimerState>>>, // 첫 poll 전에는 None
    polls: u32,
}
```

- **완료 확인과 웨이커 저장은 같은 잠금 안에서** 합니다. 잠금 없이 "아직이네" 하고 확인한 직후 타이머 스레드가 끝나 버리면, 스레드는 저장된 웨이커가 없어 아무도 깨우지 못하고, 퓨처는 그 뒤에 웨이커를 저장한 채 영원히 잠듭니다. 잠금 하나가 두 동작 사이의 틈을 막습니다.
- 다른 스레드는 웨이커를 **잠금을 놓은 뒤에** 부릅니다. 깨어난 쪽이 곧바로 같은 잠금을 잡으려 할 것이기 때문입니다([Part 1 ch11](../ch11-threads-sync/README.md) 2.4절의 "가드 범위 줄이기"와 같은 이유입니다).
- **게으름을 지킵니다.** 스레드는 `sleep()`을 부를 때가 아니라 첫 `poll`에서 시작합니다. `Sleep`의 필드는 모두 `Unpin`이므로 `self.get_mut()`으로 `&mut Sleep`을 꺼내 필드를 따로따로 빌립니다(2.7절).
- **마지막 웨이커만 깨웁니다.** 이미 저장된 웨이커가 있으면 `old.clone_from(cx.waker())`로 바꿉니다. 같은 태스크면 복제를 건너뛰고, 다른 태스크로 옮겨 갔다면 새 웨이커로 바꿉니다. `05`의 3번 구역은 타이머를 웨이커 A로, 이어서 B로 폴링한 뒤 시간이 되면 **B만** 깨워지는 것을 확인합니다.

`05`의 1번 구역에서 `sleep(50ms)`는 정확히 두 번 폴링됩니다(첫 `Pending`, 깨어난 뒤 `Ready`). `04`의 4번 구역의 바쁜 폴링과 비교해 보세요. 30ms 타이머 두 개를 `.await`로 차례로 기다리면 60ms 이상 걸립니다. 함께 기다리는 방법은 2.8절에서 만듭니다.

**일회용 채널(oneshot).** 같은 모양으로 "값 하나를 다른 스레드에서 받는" 퓨처를 만듭니다. `Receiver`가 퓨처이고 `Output`은 `Option<T>`입니다. 핵심은 `Sender`의 `Drop`입니다. 값을 보내든 보내지 않든 송신자가 사라지면 받는 쪽을 깨웁니다. 깨우지 않으면 값 없이 버려진 송신자를 기다리는 퓨처가 영원히 `Pending`으로 남습니다. `send(self, v)`는 값만 넣고, 함수가 끝날 때 `self`가 버려지면서 `Drop`이 깨웁니다. Part 0 ch16의 비동기 채널과 `spawn_task`의 `JoinHandle`도 같은 원리(값이 오면, 또는 보내는 쪽이 사라지면 저장된 웨이커로 깨움)로 동작합니다. [Part 0 ch16](../../part0-getting-started/ch16-async/README.md) 2.12절의 "무거운 계산은 스레드에서 하고 결과는 async 코드에서 기다리기"도 `05`의 2번 구역처럼 표준 라이브러리만으로 만들 수 있습니다.

### 2.7 `Pin`과 `Unpin`

**왜 고정해야 하나.** Part 0 ch16 2.11절은 await 너머로 지역 변수의 참조를 들고 가는 퓨처가 자기 자신을 가리키게 되고, 그런 퓨처를 옮기면 참조가 옛 자리를 가리킨다고 그림으로 설명했습니다. [`06_pin_unpin.rs`](examples/06_pin_unpin.rs)의 3번 구역은 이것을 주소로 확인합니다.

```rust
let mut future = Box::pin(async {
    let data = [7u8; 32];
    let first = &data[0];                         // data를 가리키는 참조
    data_address.set(first as *const u8 as usize);
    yield_once().await;                            // 이 await 너머로 first를 들고 갑니다
    *first
});
```

첫 `poll` 뒤 `data`의 주소는 퓨처의 메모리 범위(`&*future`의 주소부터 `size_of_val(&*future)`바이트) **안에** 있습니다. await 너머로 쓰는 값은 퓨처 안에 저장되므로 `first`는 퓨처 자신의 일부를 가리킵니다. Rust의 이동은 바이트를 그대로 복사하는 것이고 안쪽 포인터를 고쳐 주지 않으므로, 이 퓨처를 옮기면 `first`는 댕글링 참조가 됩니다. 대여 검사기는 "자기 안을 가리키는 참조"를 추적하지 못하므로, 대신 **처음 폴링한 뒤로는 옮기지 못하게** 막는 장치가 `Pin`입니다.

**`Pin<P>`가 보장하는 것.** `Pin`은 값이 아니라 포인터(`&mut T`, `Box<T>` 등)를 감쌉니다. `Pin<P>`로 한 번 고정된 값은 버려질 때까지 그 자리에 있습니다. 안전한 코드에서는 `Pin<&mut T>`에서 `&mut T`를 꺼낼 수 없으므로 `mem::replace`나 `mem::swap`([Part 1 ch7](../ch7-smart-pointers-memory/README.md) 2.5절)으로 값을 빼낼 수도 없습니다. `Future::poll`이 `self: Pin<&mut Self>`를 받는 것은 "고정된 퓨처만 실행할 수 있다"는 뜻입니다. 고정은 컴파일러의 특별한 기능이 아니라 `Pin` 타입이 제공하는 API의 제약으로 지켜지는 약속입니다.

**`Unpin`: 고정이 아무 제약도 아닌 타입.** 대부분의 타입은 자기 안을 가리키는 포인터가 없어 옮겨도 안전합니다. 이런 타입에는 `Unpin`이 구현되어 있고, `T: Unpin`이면 `Pin<&mut T>`는 `&mut T`와 똑같이 쓸 수 있습니다. [Part 1 ch6](../ch6-core-traits/README.md)에서 미뤄 둔 `Unpin`의 성질은 다음과 같습니다.

- `Send`/`Sync`처럼 메서드가 없는 **자동 트레이트(auto trait)** 입니다. 필드가 모두 `Unpin`이면 구조체·열거형도 자동으로 `Unpin`입니다(`Countdown`, `Sleep`).
- 원시 타입, `String`, `Vec`, `&T`, `&mut T`는 `Unpin`입니다. **`Box<T>`는 `T`와 관계없이 `Unpin`** 이고(옮겨지는 것은 포인터뿐), 그래서 `Pin<Box<T>>`도 언제나 `Unpin`입니다.
- `async` 블록과 `async fn`이 만든 퓨처는 자기 참조가 생길 수 있으므로 `Unpin`이 아닙니다(`!Unpin`). 표준 라이브러리의 `std::marker::PhantomPinned` 필드를 넣으면 직접 만든 타입도 `Unpin`에서 빠집니다.
- `Unpin`은 고정되지 않은 값에는 아무 영향이 없습니다. `!Unpin`인 퓨처도 고정하기 전(첫 `poll` 전)에는 변수에 담거나 함수에 넘기며 마음대로 옮길 수 있습니다.

| 하려는 일 | API | 조건 |
| --- | --- | --- |
| 고정하기(그 자리) | `pin!(value)` → `Pin<&mut T>` (1.68.0) | 없음. 힙 할당 없음. 결과는 그 블록 밖으로 내보낼 수 없음 |
| 고정하기(힙) | `Box::pin(value)` → `Pin<Box<T>>`, 이미 `Box`면 `Box::into_pin` | 없음. 옮기고 저장하고 돌려줄 수 있음 |
| 고정하기(`Unpin`) | `Pin::new(&mut value)` | `T: Unpin` |
| 여러 번 폴링 | `pinned.as_mut()` → 다시 빌린 `Pin<&mut T>` | 없음. `poll`이 `Pin<&mut T>`를 값으로 받아 소비하기 때문에 필요 |
| 평범한 `&mut T`로 | `pinned.get_mut()`, `Pin::into_inner` | `T: Unpin` |
| 같은 자리에 새 값 | `pinned.set(new_value)` | 없음. 옛 값은 그 자리에서 버려짐 |

`06`의 1번 구역은 `Unpin`인 `i32`와 `YieldOnce`를 `Pin::new`로 감싸고, `get_mut`으로 되돌리고, 폴링한 뒤에 다른 변수로 옮겨 다시 폴링합니다. 2번 구역은 `!Unpin`인 `async fn` 퓨처를 `pin!`과 `Box::pin`으로 고정합니다. `Pin<Box<_>>`를 다른 변수로 옮겨도 옮겨지는 것은 `Box` 포인터뿐이라 힙에 있는 퓨처의 주소는 그대로입니다. 고정하지 않고 `poll`을 부르면 E0599(4.1절), `!Unpin`에 `Pin::new`를 쓰면 E0277(4.2절)입니다.

**`pin!`과 `Box::pin` 고르기.** 한 함수 안에서 만들어 끝까지 폴링한다면 할당이 없는 `pin!`을 씁니다(`block_on`이 그렇습니다). 퓨처를 구조체 필드에 두거나, `Vec`에 모으거나, 함수에서 돌려줘야 하면 `Box::pin`입니다.

**`Pin<Box<dyn Future<Output = T>>>` 읽기.** 안쪽부터 읽습니다. `dyn Future<Output = T>`는 "`T`를 내놓는 어떤 퓨처"라는 트레이트 객체([Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md))이고, 크기를 모르므로 `Box`에 담고, `poll`하려면 고정해야 하므로 `Pin`으로 감쌉니다. 실행기에 넘길 때는 `+ Send`가 붙습니다(2.9절). 이 타입이 필요한 곳은 다음과 같습니다.

- 조건에 따라 서로 다른 `async` 블록을 돌려주는 함수(`06`의 `greeting`)와 서로 다른 퓨처를 한 컬렉션에 담기(Part 0 ch16 4.4절)
- 재귀 `async fn`: 자기 자신의 퓨처를 안에 품으면 크기가 무한해지므로 `Box::pin(sum_to(n - 1)).await`처럼 한 단계 간접 참조를 둡니다(4.4절)
- 연관 타입에 `impl Future`를 쓸 수 없는 곳(`IntoFuture::IntoFuture`, 2.8절)

**unsafe 없이 퓨처를 구현하는 요령.** 이 챕터의 퓨처는 모두 `Unpin`입니다. 필드를 `Unpin`인 것(숫자, `Option<Waker>`, `Arc<..>`)으로만 두거나, 안쪽 퓨처를 `Pin<Box<F>>`로 들고 있으면 `get_mut`이나 `DerefMut`으로 필드를 고칠 수 있습니다. 안쪽 퓨처를 상자 없이 필드로 품고 `Pin<&mut Self>`에서 `Pin<&mut 필드>`를 꺼내는 일(고정 투영, pin projection)은 `get_unchecked_mut`, `map_unchecked_mut`, `Pin::new_unchecked` 같은 `unsafe` 함수가 필요합니다. 고정의 약속을 컴파일러 대신 사람이 지켜야 하기 때문이며, Part 10(Rustonomicon)에서 다룹니다.

### 2.8 퓨처를 조합하는 퓨처와 `std::future`의 도구

**직접 만든 `join`.** [`07_join_and_tools.rs`](examples/07_join_and_tools.rs)의 `Join`은 두 퓨처를 `Pin<Box<_>>`로 들고, 결과를 보관할 칸을 하나씩 둡니다.

```rust
struct Join<A: Future, B: Future> {
    a: Pin<Box<A>>,
    b: Pin<Box<B>>,
    a_out: Option<A::Output>,
    b_out: Option<B::Output>,
}
```

`poll`은 아직 끝나지 않은 쪽만 폴링하고(끝난 퓨처를 다시 폴링하면 안 되므로), 두 칸이 모두 차면 `Ready((a, b))`를 돌려줍니다. 두 퓨처에 **같은 `cx`를** 넘기므로 어느 쪽이 깨우든 `Join`이 다시 폴링되고, 그때 아직 준비되지 않은 쪽은 한 번 더 `Pending`을 돌려줄 뿐입니다. 출력 칸까지 `Unpin`이어야 `Join` 전체가 `Unpin`이 되어 `get_mut`을 쓸 수 있으므로 `A::Output: Unpin` 조건을 붙였습니다. `07`의 1번 구역에서 진행 순서는 `A1 B1 A2 B2 A3`로 번갈아 가고 결과는 인자 순서대로 `(300, 200)`입니다. 50ms 타이머 두 개를 `join`하면 100ms가 아니라 50ms 남짓에 끝납니다. Part 0 ch16 2.5절에서 `trpl::join`이 공정하다고 한 것(깨어날 때마다 끝나지 않은 퓨처를 인자 순서대로 모두 확인)도 이런 구조이기 때문입니다.

**표준 라이브러리의 도구.** 모두 안정 API이며 `std::future`, `std::task`에 있습니다.

| 도구 | 하는 일 | 예제 |
| --- | --- | --- |
| `future::ready(v)` | 첫 `poll`에서 `Ready(v)`. `Unpin` | 01, 07 |
| `future::pending()` | 영원히 `Pending`. 웨이커를 저장하지도 않음 | 01 |
| `future::poll_fn(\|cx\| ..)` (1.64.0) | 클로저 하나로 퓨처를 만듦. 클로저가 `Unpin`이면 `PollFn`도 `Unpin` | 07 |
| `task::ready!(poll식)` (1.64.0) | `Pending`이면 바로 `return Poll::Pending`, `Ready(v)`면 `v` | 07 |
| `IntoFuture` (1.64.0) | `.await`가 먼저 부르는 변환. Edition 2024 프렐류드에 있음 | 07 |
| `Poll::map`, `is_ready`, `is_pending` | `Poll` 다루기 | 01 |

- **`poll_fn`**: `07`의 `yield_now`는 `bool` 하나를 캡처한 클로저로 `YieldOnce`와 같은 퓨처를 만듭니다. 2번 구역은 `poll_fn(|cx| Poll::Ready(timer.as_mut().poll(cx))).await`로 다른 퓨처를 **딱 한 번** 폴링해 본 결과(`Pending`)를 값으로 받습니다. 그 뒤 `timer.await`로 끝까지 기다리는데, `timer`는 `Pin<&mut Sleep>`입니다. `Pin<P>`(가리키는 것이 퓨처일 때), `Box<F>`와 `&mut F`(`F: Unpin`일 때)도 `Future`를 구현하기 때문입니다.
- **`ready!`**: `Map`의 `poll`은 `let value = ready!(this.inner.as_mut().poll(cx));` 한 줄로 "아직이면 나도 아직"을 표현합니다. 문서가 보여 주는 펼친 모양은 `match fut.poll(cx) { Poll::Ready(t) => t, Poll::Pending => return Poll::Pending }`입니다. `?` 연산자가 `Err`를 전파하는 것과 닮았습니다.
- **`IntoFuture`**: `x.await`는 먼저 `IntoFuture::into_future(x)`를 부르고 그 결과를 폴링합니다. 모든 `Future`는 자기 자신을 돌려주는 `IntoFuture`를 갖고 있습니다. `07`의 `Reminder`처럼 설정을 메서드로 이어 붙인 빌더를 `.await` 한 번으로 실행할 때 씁니다(`Reminder::new("물 마시기").after_millis(30).await`). `.await`할 수 없는 값을 기다리면 "`...` is not a future"(E0277)입니다(4.6절).

`std::future::join!` 매크로, `std::task::waker_fn`, `LocalWaker`, `Context::ext`는 아직 nightly 전용입니다(로컬 문서의 Experimental 표시로 확인).

### 2.9 여러 태스크를 돌리는 실행기

`block_on`은 퓨처 하나만 실행합니다. [`08_mini_executor.rs`](examples/08_mini_executor.rs)는 Part 0 ch16의 `spawn_task`처럼 여러 **태스크**를 돌리는 실행기를 만듭니다. 태스크는 실행기에 맡긴 최상위 퓨처이고, 대기열은 [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md)의 `mpsc` 채널입니다.

```rust
type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

struct Task {
    name: &'static str,
    future: Mutex<Option<BoxFuture>>, // 끝나면 None
    queue: Sender<Arc<Task>>,         // 자기 자신을 다시 넣을 대기열
    polls: AtomicUsize,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) { self.wake_by_ref(); }
    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self.queue.send(Arc::clone(self)); // 웨이커 = "이 태스크를 대기열에 다시 넣기"
    }
}
```

- `spawn`은 퓨처를 `Box::pin`으로 고정해 `Task`에 담고 대기열에 넣습니다. `run`은 `recv()`로 태스크를 하나 꺼내 그 태스크로 만든 웨이커로 폴링하고, `Ready`면 퓨처를 버립니다. 대기열이 비면 `recv()`에서 잠들므로 바쁜 폴링이 아닙니다. 타이머 스레드가 웨이커를 부르면 태스크가 대기열에 들어가고 실행기가 깨어납니다.
- **`Option`인 이유**: 한 태스크가 폴링되기 전에 두 번 깨워지면 대기열에 두 번 들어갑니다. 끝난 뒤에 꺼낸 것은 `None`을 보고 건너뜁니다.
- **`Mutex`, `Send + 'static`인 이유**: 웨이커가 `Arc<Task>`를 품은 채 타이머 스레드로 건너가므로 `Arc<Task>`가 `Send`여야 하고, 그러려면 `Task`가 `Send + Sync`여야 합니다. `Mutex<T>`는 `T: Send`면 `Sync`이므로, 결국 퓨처가 `Send`여야 합니다. `'static`은 태스크가 `spawn`을 부른 코드보다 오래 살 수 있기 때문입니다. `Rc`를 await 너머로 들고 가는 퓨처를 넘기면 4.5절의 에러가 납니다. tokio의 `spawn`(Part 0 ch16의 `spawn_task`)이 `Send + 'static`을 요구하는 이유도 같습니다.

1번 구역은 스스로 양보하는 태스크 A(3단계)·B(2단계)·C(1단계)를 실행해 `A1 B1 C1 A2 B2 A3` 순서와, 끝난 순서 `C`(폴링 2번) → `B`(3번) → `A`(4번)를 보여 줍니다. 대기열이 선입선출이라 양보한 태스크는 맨 뒤로 갑니다. 태스크는 await 지점에서만 차례를 넘기는 협력적 멀티태스킹이므로, 한 태스크가 양보 없이 오래 계산하면 나머지가 모두 기다립니다(Part 0 ch16 2.9절). 2번 구역은 20·60·100ms 타이머를 기다리는 태스크 셋이 각각 2번씩만 폴링되고 전체가 100ms 남짓(합인 180ms보다 짧게)에 끝나는 것을 확인합니다.

### 2.10 우리가 만든 것과 실제 런타임의 차이

이 챕터의 부품은 실제 런타임과 같은 원리로 움직이지만 규모와 효율이 다릅니다. Part 0 ch16에서 `trpl`(안은 tokio)이 해 주던 일과 비교하면 다음과 같습니다.

| | 이 챕터 | 실제 런타임(tokio 등) |
| --- | --- | --- |
| 타이머 | 타이머마다 스레드 하나(`Sleep`) | 타이머 휠 같은 자료 구조 하나가 모든 타이머를 관리하고 시간이 된 웨이커를 부름 |
| 입출력 | 없음 | 소켓을 논블로킹으로 두고 운영체제의 이벤트 통지(`epoll`, `kqueue`, IOCP)로 "준비됨"을 받아 웨이커를 부름 |
| 실행 스레드 | 현재 스레드 하나 | 작업자 스레드 여럿, 바쁜 스레드의 태스크를 한가한 스레드가 가져가는 작업 훔치기 |
| 여러 퓨처 함께 기다리기 | `Join`은 어느 쪽이 깨우든 둘 다 폴링 | 퓨처마다 웨이커를 따로 만들어 깨운 것만 폴링하기도 함(수천 개를 함께 기다릴 때) |
| 태스크의 결과 | 없음(연습 4) | `JoinHandle`이 퓨처. 2.6절의 oneshot과 같은 원리 |
| 블로킹 진입점 | `block_on`(park/unpark) | `Runtime::block_on`, `#[tokio::main]` |

[ch12](../ch12-net/README.md)의 논블로킹 소켓이 돌려주는 `WouldBlock`은 `Pending`과 닮았습니다. 차이는 "준비되면 알려 주는 연락처", 곧 웨이커입니다. 표준 라이브러리에는 운영체제의 이벤트 통지를 기다리는 기능이 없어서 비동기 네트워킹과 비동기 파일 입출력은 크레이트(런타임)가 제공합니다. 표준 라이브러리가 주는 것은 이 챕터에서 쓴 공통 부품(`Future`, `Waker`, `Pin`)까지이고, 덕분에 서로 다른 런타임과 라이브러리가 같은 퓨처 타입으로 이어집니다.

그 밖에 알아 둘 것은 다음과 같습니다.

- **스트림**: Part 0 ch16의 `Stream`은 크레이트(`futures-core`)의 것이었습니다. 표준 라이브러리에는 같은 모양의 `std::async_iter::AsyncIterator`(`poll_next`)가 있지만 nightly 전용입니다(Part 11).
- **비동기 클로저**: `async || { .. }`와 이를 위한 `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce` 트레이트는 안정이고(1.85.0) 모든 에디션의 프렐류드에 들어 있습니다. `Fn` 계열의 비동기 짝입니다.

### 2.11 이 챕터에서 다루지 않는 것

| 주제 | 한 줄 소개 | 다루는 곳 |
| --- | --- | --- |
| `RawWaker`/`RawWakerVTable`, `Waker::new` | 함수 포인터 표와 원시 포인터로 할당 없이 웨이커를 만듦. 규약을 어기면 미정의 동작이라 `unsafe` | Part 10 |
| `Pin::new_unchecked`, `get_unchecked_mut`, 고정 투영 | 고정의 약속을 사람이 보증하며 안쪽 필드를 고정된 채로 꺼냄 | Part 10 |
| `Send`/`Sync` 직접 구현 | `unsafe impl`로 스레드 안전성을 보증 | Part 10 `ch9-concurrency` |
| `AsyncIterator`, `future::join!`, `LocalWaker` | 아직 불안정한 비동기 API | Part 11 |
| `Any`와 다운캐스트 | 실행 중 타입 확인 | [ch14](../ch14-ffi-any-misc/README.md) |
| 동기화 도구 자체(`Mutex`, `Condvar`, 채널, 원자 타입) | 이 챕터는 쓰기만 함 | [ch11](../ch11-threads-sync/README.md) |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_poll_by_hand.rs`](examples/01_poll_by_hand.rs) | `Waker::noop()`으로 손으로 폴링, `ready`/`pending`, async 블록의 게으름과 await 지점, 끝난 퓨처를 다시 폴링할 때의 패닉 | 1 |
| [`02_waker_and_wake.rs`](examples/02_waker_and_wake.rs) | `Wake` 구현, `wake`/`wake_by_ref`/`clone`과 `Arc` 소유자 수, `will_wake`, 다른 스레드에서 깨우기 | 2 |
| [`03_custom_future.rs`](examples/03_custom_future.rs) | 바로 끝나는 퓨처, 스스로 깨우는 `Countdown`, 깨우는 것을 잊은 퓨처, `async fn`과 손으로 쓴 상태 기계 | 1, 3 |
| [`04_block_on.rs`](examples/04_block_on.rs) | `park`/`unpark`의 토큰, `block_on`, 깨우지 않으면 멈춤, 바쁜 폴링 | 5 |
| [`05_timer_and_oneshot.rs`](examples/05_timer_and_oneshot.rs) | 공유 상태와 저장한 웨이커로 만든 타이머, oneshot 채널과 `Drop`, 마지막 웨이커만 깨우기 | 2, 3 |
| [`06_pin_unpin.rs`](examples/06_pin_unpin.rs) | `Pin::new`/`get_mut`, `pin!`/`Box::pin`/`set`, 자기 참조 주소 확인, `Pin<Box<dyn Future>>`, 재귀 `async fn` | 4 |
| [`07_join_and_tools.rs`](examples/07_join_and_tools.rs) | 직접 만든 `join`, `poll_fn`, `task::ready!`로 만든 `map`, `IntoFuture` 빌더 | 3 |
| [`08_mini_executor.rs`](examples/08_mini_executor.rs) | 태스크 대기열(채널)과 `spawn`, 태스크를 대기열에 다시 넣는 웨이커, 타이머를 기다리는 태스크 | 5 |
| [`compile_fail/`](compile_fail) | 고정·`poll` 시그니처·재귀·`Send`에서 만나는 컴파일 에러 5개(4절) | 1, 3, 4, 5 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 몇 초 안에 끝나고 표준 에러에 아무것도 출력하지 않습니다. 시간은 "몇 ms 이상/미만인가"라는 비교 결과로만 출력하고, 실행마다 달라지는 폴링 횟수(04의 4번 구역)는 "1000번을 넘는가"로 출력하므로 여러 번 실행해도 출력이 같았습니다. 다만 시간 비교는 컴퓨터가 몹시 바빠 스레드가 제때 실행되지 못하면 달라질 수 있습니다. 메모리 주소는 출력하지 않고 비교 결과만 출력합니다.

### 01_poll_by_hand — 손으로 폴링하기

```sh
cargo run -p p1-ch13-async-primitives --example 01_poll_by_hand
```

```text
1. std::future::ready: 첫 poll에서 바로 Ready
    첫 poll: Ready(42)
    is_ready: true, map(|v| v * 2): Ready(84)
2. std::future::pending: 몇 번을 폴링해도 Pending
    1번째 poll: Pending
    2번째 poll: Pending
    3번째 poll: Pending
3. async 블록: 본문은 만들 때가 아니라 첫 poll에서 실행됩니다
    퓨처를 만들었습니다 (본문은 아직 실행되지 않음)
    [본문] 실행 시작
    [본문] a + b를 돌려줍니다
    첫 poll: Ready(3)
    [본문] await 전
    stuck 1번째 poll: Pending
    stuck 2번째 poll: Pending
    stuck 3번째 poll: Pending
4. 끝난 퓨처를 다시 폴링하면 안 됩니다
    async 블록 첫 poll: Ready(7)
    다시 poll: 패닉 — `async fn` resumed after completion
    ready(42)를 다시 poll: 패닉 — `Ready` polled after completion
```

4번 구역은 [Part 1 ch8](../ch8-errors-panic/README.md)의 `catch_unwind`로 패닉을 붙잡고, 그동안 패닉 훅을 아무것도 출력하지 않는 훅으로 바꿔 둡니다. 페이로드는 async 블록 쪽이 `&str`, `Ready` 쪽이 `String`이라 둘 다 확인합니다. async 블록이 만든 퓨처도 메시지에는 `async fn`이라고 나옵니다.

### 02_waker_and_wake — 웨이커 만들기

```sh
cargo run -p p1-ch13-async-primitives --example 02_waker_and_wake
```

```text
1. Arc<T: Wake>를 Waker로 바꿉니다
    Arc의 소유자 수: 2
    A: wake 0번, wake_by_ref 2번
    clone 뒤 소유자 수: 3
    A: wake 1번, wake_by_ref 2번
    wake 뒤 소유자 수: 2
2. Context는 웨이커를 빌려 담는 상자입니다
    A: wake 1번, wake_by_ref 3번
3. will_wake: 두 웨이커가 같은 태스크를 깨우는가
    waker와 clone:           true
    같은 Arc로 따로 만든 것: true
    다른 Arc(B):             false
    Waker::noop():           false
4. Waker는 Send + Sync라서 어느 스레드에서든 깨울 수 있습니다
    C: wake 4번, wake_by_ref 1번
    웨이커를 버린 뒤 A의 소유자 수: 1
```

`Wake`의 두 메서드를 모두 구현해 어느 쪽으로 불렸는지 따로 셉니다. 웨이커의 `wake`는 알맹이의 `wake`를, `wake_by_ref`는 알맹이의 `wake_by_ref`를 부릅니다. 같은 `Arc`에서 따로 만든 웨이커끼리도 `will_wake`가 `true`인 것은 둘이 같은 알맹이를 가리키기 때문입니다.

### 03_custom_future — 퓨처 직접 구현하기

```sh
cargo run -p p1-ch13-async-primitives --example 03_custom_future
```

```text
1. 바로 끝나는 퓨처
    poll 1: Ready("준비된 값")
2. Countdown { remaining: 3 }: Pending 3번, 그때마다 스스로 깨움
    poll 1: Pending, 그사이 웨이커가 불렸는가: true
    poll 2: Pending, 그사이 웨이커가 불렸는가: true
    poll 3: Pending, 그사이 웨이커가 불렸는가: true
    poll 4: Ready("발사!")
3. 깨우는 것을 잊은 퓨처
    poll 1: Pending, 그사이 웨이커가 불렸는가: false
    → 아무도 깨우지 않았으므로 더 폴링하지 않습니다
    결과: None
4. async fn과 손으로 쓴 상태 기계는 같은 순서로 진행합니다
  (가) async fn add_slowly(2, 3)
      [async fn] 시작
    poll 1: Pending, 그사이 웨이커가 불렸는가: true
      [async fn] 중간
    poll 2: Pending, 그사이 웨이커가 불렸는가: true
      [async fn] 끝
    poll 3: Ready(5)
  (나) AddSlowly::Start { a: 2, b: 3 }
      [상태 기계] 시작
    poll 1: Pending, 그사이 웨이커가 불렸는가: true
      [상태 기계] 중간
    poll 2: Pending, 그사이 웨이커가 불렸는가: true
      [상태 기계] 끝
    poll 3: Ready(5)
```

`drive`는 `poll` 전후로 웨이커가 불린 횟수를 비교해, `Pending`인데 연락이 없으면 멈춥니다. 실행기라면 이 퓨처를 영원히 기다리게 되는 상황입니다(04의 3번 구역).

### 04_block_on — 최소 실행기

```sh
cargo run -p p1-ch13-async-primitives --example 04_block_on
```

```text
1. 재료: unpark가 park보다 먼저 와도 신호는 사라지지 않습니다
    park가 바로 돌아왔는가(10ms 미만): true
2. block_on으로 퓨처를 끝까지 실행합니다
    countdown(3)이 폴링된 횟수: 4
    async 블록 안의 두 countdown 폴링 횟수 합: 5
3. 깨우지 않으면 실행기는 다시 폴링하지 않습니다
    결과: None
    포기하기 전에 50ms 이상 기다렸는가: true
4. 스스로 계속 깨우면 바쁜 폴링이 됩니다
    30ms를 기다리는 동안 폴링 횟수가 1000번을 넘는가: true
    걸린 시간이 30ms 이상인가: true
```

1번 구역은 `block_on`을 부르기 전에 실행합니다. `block_on`이 스스로 깨우는 퓨처를 실행하는 동안 `unpark`가 토큰을 남겨 두므로, 뒤에서 실행하면 그 토큰 때문에 결과가 같아도 이유가 달라지기 때문입니다.

### 05_timer_and_oneshot — 다른 스레드가 완료시키는 퓨처

```sh
cargo run -p p1-ch13-async-primitives --example 05_timer_and_oneshot
```

```text
1. 타이머: 스레드가 시간이 되면 깨워 줍니다
    sleep(50ms): 폴링 2번, 50ms 이상 걸렸는가: true
    30ms 두 번을 차례로 await: 60ms 이상 걸렸는가: true
2. oneshot: 다른 스레드가 보낸 값 하나를 기다립니다
    받은 값: Some(500000500000)
    보내지 않고 버린 경우: None
3. 웨이커가 바뀌면 마지막 poll의 웨이커만 깨웁니다
    A로 poll: Pending, B로 poll: Pending
    깨워진 웨이커: B, 다른 연락이 더 있는가: false
    B로 다시 poll: Ready(3)
```

- **2번 구역**: 보내는 쪽은 스코프 스레드([Part 1 ch11](../ch11-threads-sync/README.md) 2.2절)이고, 같은 스코프 안에서 `block_on(rx)`로 기다립니다. 타이머 스레드는 웨이커를 부른 직후 끝나므로 예제가 끝날 때 남는 스레드가 없습니다.
- **3번 구역**: 깨워지면 자기 이름을 채널로 보내는 웨이커 두 개로 손으로 폴링합니다. `main`은 `rx.recv()`로 타이머가 깨울 때까지 기다리므로 시간에 기대지 않고 결과가 정해집니다. 마지막 `poll`이 세 번째라서 `Ready(3)`입니다.

### 06_pin_unpin — 고정

```sh
cargo run -p p1-ch13-async-primitives --example 06_pin_unpin
```

```text
1. Unpin 타입에게 Pin은 아무 제약도 아닙니다
    n = 7
    YieldOnce: 옮기기 전 Pending, 옮긴 뒤 Ready(())
    i32, String, YieldOnce, Pin<Box<async 블록>>은 Unpin입니다 (컴파일로 확인)
2. Unpin이 아닌 async 블록 고정하기: pin!(스택)과 Box::pin(힙)
    pin!:     값 10, 폴링 2번
    set 뒤:   값 20
    Box::pin: 첫 poll Pending, 옮긴 뒤 poll Ready(30), 퓨처 주소가 그대로인가: true
3. 왜 고정해야 하나: await 너머로 들고 가는 참조는 퓨처 자신을 가리킵니다
    첫 poll: Pending, data가 퓨처의 메모리 안에 있는가: true
    두 번째 poll: Ready(7)
4. Pin<Box<dyn Future<Output = T>>>: 서로 다른 퓨처를 한 타입으로
    안녕하세요 (폴링 1번)
    hello (폴링 2번)
    재귀 async fn sum_to(100) = 5050
```

1번 구역의 `assert_unpin::<T: Unpin>`은 아무 일도 하지 않는 함수입니다. 호출이 컴파일된다는 것이 곧 `Unpin`이라는 확인입니다. 주소는 `&*future as *const _ as usize`처럼 참조를 원시 포인터로, 다시 정수로 바꿔 비교만 합니다. 이 변환은 `unsafe`가 아닙니다(원시 포인터를 역참조할 때만 `unsafe`가 필요합니다).

### 07_join_and_tools — 조합과 도구

```sh
cargo run -p p1-ch13-async-primitives --example 07_join_and_tools
```

```text
1. 직접 만든 join: 두 작업이 번갈아 진행됩니다
    진행 순서: ["A1", "B1", "A2", "B2", "A3"]
    결과(인자 순서대로): (300, 200)
    50ms 타이머 두 개를 join: 50ms 이상인가 true, 100ms 미만인가 true
2. poll_fn: 한 번만 폴링해 보고 결과를 값으로 받기
    첫 폴링 결과: Pending
3. task::ready!로 만든 map
    100점
    ready(21)에 map: 42
4. IntoFuture: .await가 into_future()를 불러 줍니다
    알림: 물 마시기 (30ms 이상 기다렸는가: true)
    알림: 바로
```

`block_on`과 `sleep`은 04·05에서 옮겨 왔습니다. 진행 기록장은 모든 퓨처가 한 스레드에서 실행되므로 `RefCell<Vec<String>>`([Part 1 ch7](../ch7-smart-pointers-memory/README.md))이고, 대여는 `push` 한 문장 안에서 끝나므로 await 너머로 이어지지 않습니다.

### 08_mini_executor — 여러 태스크를 돌리는 실행기

```sh
cargo run -p p1-ch13-async-primitives --example 08_mini_executor
```

```text
1. 스스로 양보하는 태스크 셋: 대기열 순서대로 번갈아 실행됩니다
    실행 순서: A1 B1 C1 A2 B2 A3
    끝난 순서와 폴링 횟수: [("C", 2), ("B", 3), ("A", 4)]
2. 타이머를 기다리는 태스크 셋: 기다리는 동안 실행기는 잠듭니다
    태스크별 폴링 횟수: [("긴", 2), ("중간", 2), ("짧은", 2)]
    전체 시간이 가장 긴 타이머(100ms) 이상인가: true, 세 타이머의 합(180ms)보다 짧은가: true
```

2번 구역에서 태스크가 끝나는 순서는 타이머 스레드에 달려 있으므로 이름순으로 정렬해 출력합니다. 1번 구역의 기록장은 태스크가 `Send + 'static`이어야 하므로 `Arc<Mutex<Vec<String>>>`입니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch13-async-primitives
rustc --edition 2024 compile_fail/e0599_poll_without_pin.rs --out-dir /tmp/rust-p1-ch13
```

`note: required by a bound in ...` 아래에 표준 라이브러리 소스 위치(`/rustc/0ef1bc38.../library/core/src/pin.rs:...`)가 나오는 경우가 있습니다. 경로의 해시 값과 줄 번호는 툴체인마다 다르며, 아래 인용에서는 그 줄을 생략했습니다.

### 4.1 고정하지 않은 퓨처에 `poll` — E0599

파일: [`compile_fail/e0599_poll_without_pin.rs`](compile_fail/e0599_poll_without_pin.rs)

```text
error[E0599]: no method named `poll` found for `async` block `{async block@compile_fail/e0599_poll_without_pin.rs:13:22: 13:27}` in the current scope
  --> compile_fail/e0599_poll_without_pin.rs:14:25
   |
14 |     let result = future.poll(&mut cx); // 고정하지 않은 퓨처에 poll → E0599
   |                         ^^^^ method not found in `{async block@compile_fail/e0599_poll_without_pin.rs:13:22: 13:27}`
   |
help: consider pinning the expression
   |
14 ~     let mut pinned = std::pin::pin!(future);
15 ~     let result = pinned.as_mut().poll(&mut cx); // 고정하지 않은 퓨처에 poll → E0599
   |
```

- **읽는 법**: `Future`가 프렐류드에 있는데도 "메서드를 찾지 못했다"고 합니다. `poll`의 수신자가 `Pin<&mut Self>`라서, 메서드 탐색이 `future`, `&future`, `&mut future`를 차례로 시도해도 맞는 수신자가 없기 때문입니다.
- **원인**: `poll`은 고정된 퓨처에만 부를 수 있습니다(2.7절).
- **해결**: `help`대로 `pin!`으로 고정하고 `as_mut()`으로 부릅니다. 퓨처를 다른 곳에 저장해야 하면 `Box::pin(future)`입니다. 손으로 폴링할 이유가 없다면 `.await`나 실행기에 맡기는 것이 보통입니다.

### 4.2 `Unpin`이 아닌 퓨처에 `Pin::new` — E0277

파일: [`compile_fail/e0277_pin_new_async_block.rs`](compile_fail/e0277_pin_new_async_block.rs)

```text
error[E0277]: `{async block@compile_fail/e0277_pin_new_async_block.rs:14:22: 14:27}` cannot be unpinned
  --> compile_fail/e0277_pin_new_async_block.rs:15:27
   |
15 |     let result = Pin::new(&mut future).poll(&mut cx); // Unpin이 아닌 값에 Pin::new → E0277
   |                  -------- ^^^^^^^^^^^ the trait `Unpin` is not implemented for `{async block@compile_fail/e0277_pin_new_async_block.rs:14:22: 14:27}`
   |                  |
   |                  required by a bound introduced by this call
   |
   = note: consider using the `pin!` macro
           consider using `Box::pin` if you need to access the pinned value outside of the current scope
note: required by a bound in `Pin::<Ptr>::new`
```

- **읽는 법**: Part 0 ch16 4.7절에서 `join_all`과 스트림의 `next`가 냈던 "cannot be unpinned"와 같은 메시지입니다. 이번에는 `required by a bound in Pin::<Ptr>::new`가 요구한 쪽이 `Pin::new` 자신이라는 것을 알려 줍니다.
- **원인**: `Pin::new`는 가리키는 값이 `Unpin`일 때만 안전합니다. `Unpin`이면 고정이 아무 약속도 아니기 때문입니다. async 블록의 퓨처는 `Unpin`이 아니므로 `Pin::new`로는 "옮기지 않는다"는 약속을 할 수 없습니다. 같은 이유로 고정된 `!Unpin` 퓨처에 `get_mut()`을 불러 `&mut`로 꺼내려 해도 같은 E0277이 납니다.
- **해결**: `note`의 두 제안대로 `pin!(future)`나 `Box::pin(future)`로 고정합니다. 직접 만든 퓨처를 `Pin::new`로 다루고 싶다면 필드를 `Unpin`인 것으로만 구성합니다(2.7절).

### 4.3 `poll`의 `self`를 `&mut self`로 적음 — E0053

파일: [`compile_fail/e0053_poll_with_mut_self.rs`](compile_fail/e0053_poll_with_mut_self.rs)

```text
error[E0053]: method `poll` has an incompatible type for trait
  --> compile_fail/e0053_poll_with_mut_self.rs:19:13
   |
19 |     fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
   |             ^^^^^^^^^ expected `Pin<&mut Countdown>`, found `&mut Countdown`
   |
   = note: expected signature `fn(Pin<&mut Countdown>, &mut Context<'_>) -> Poll<_>`
              found signature `fn(&mut Countdown, &mut Context<'_>) -> Poll<_>`
help: change the self-receiver type to match the trait
   |
19 -     fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
19 +     fn poll(self: Pin<&mut Countdown>, cx: &mut Context<'_>) -> Poll<()> {
   |
```

- **읽는 법**: `expected signature`와 `found signature`를 위아래로 맞춰 보면 첫 번째 매개변수만 다릅니다.
- **원인**: 트레이트 메서드를 구현할 때는 시그니처가 트레이트와 같아야 합니다. `Iterator::next(&mut self)`에 익숙하면 자주 하는 실수입니다.
- **해결**: `use std::pin::Pin;`을 추가하고 `fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)`로 씁니다. 본문에서 `self.remaining -= 1`처럼 필드를 고치려면 `mut`이 필요하고, `Countdown`이 `Unpin`이라 그대로 동작합니다([`03`](examples/03_custom_future.rs)의 `Countdown`).

### 4.4 재귀 `async fn` — E0733

파일: [`compile_fail/e0733_recursive_async_fn.rs`](compile_fail/e0733_recursive_async_fn.rs)

```text
error[E0733]: recursion in an async fn requires boxing
  --> compile_fail/e0733_recursive_async_fn.rs:9:1
   |
 9 | async fn sum_to(n: u64) -> u64 {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
10 |     if n == 0 { 0 } else { n + sum_to(n - 1).await } // 자기 자신을 그대로 await → E0733
   |                                ------------------- recursive call here
   |
   = note: a recursive `async fn` call must introduce indirection such as `Box::pin` to avoid an infinitely sized future
```

- **원인**: `.await`하는 동안 안쪽 퓨처는 바깥 퓨처 안에 저장됩니다(2.4절의 `inner` 필드). 안쪽이 같은 타입의 퓨처라면 그 안에 또 같은 퓨처가 들어 있어 크기가 끝없이 커집니다. [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)의 재귀 타입(`enum List { Cons(i32, List), Nil }`)과 같은 문제입니다.
- **해결**: `note`대로 `n + Box::pin(sum_to(n - 1)).await`로 씁니다. 바깥 퓨처에는 포인터 하나만 저장되고 안쪽 퓨처는 힙에 고정됩니다([`06`](examples/06_pin_unpin.rs)의 4번 구역).

### 4.5 `Send`가 아닌 값을 await 너머로 들고 간 퓨처를 `spawn` — 에러 코드 없음

파일: [`compile_fail/future_not_send_across_await.rs`](compile_fail/future_not_send_across_await.rs)

```text
error: future cannot be sent between threads safely
  --> compile_fail/future_not_send_across_await.rs:20:5
   |
20 | /     spawn(async {
21 | |         let name = Rc::new(String::from("태스크"));
22 | |         yield_now().await; // name이 이 await 지점 너머에서도 쓰임 → 퓨처가 Send가 아님
23 | |         println!("{name}");
24 | |     });
   | |______^ future created by async block is not `Send`
   |
   = help: within `{async block@compile_fail/future_not_send_across_await.rs:20:11: 20:16}`, the trait `Send` is not implemented for `Rc<String>`
note: future is not `Send` as this value is used across an await
  --> compile_fail/future_not_send_across_await.rs:22:21
   |
21 |         let name = Rc::new(String::from("태스크"));
   |             ---- has type `Rc<String>` which is not `Send`
22 |         yield_now().await; // name이 이 await 지점 너머에서도 쓰임 → 퓨처가 Send가 아님
   |                     ^^^^^ await occurs here, with `name` maybe used later
note: required by a bound in `spawn`
```

- **읽는 법**: 에러 코드(`E....`)가 없는 에러입니다. Part 0 ch15의 `thread::spawn`과 `Rc` E0277과 달리, 두 번째 `note`가 **어느 await 지점** 너머로 무엇을 들고 갔는지 짚어 줍니다.
- **원인**: await 너머로 쓰는 값은 퓨처 안에 저장되므로(2.7절), 그 값이 `Send`가 아니면 퓨처도 `Send`가 아닙니다. `spawn`은 태스크가 다른 스레드에서 깨워지고 실행될 수 있어 `Send`를 요구합니다(2.9절).
- **해결**: 여러 스레드에서 공유할 값이면 `Rc` 대신 `Arc`를 씁니다. `Rc`를 await 전에만 쓴다면 `{ let name = Rc::new(..); println!("{name}"); }`처럼 **블록으로 감싸** await 전에 스코프를 끝냅니다. 이 툴체인에서 확인했을 때 await 전에 `drop(name);`을 부르는 것만으로는 같은 에러가 났으므로, 블록으로 스코프 자체를 끝내는 쪽이 확실합니다.

### 4.6 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| `fut.as_mut().poll(&mut cx);`의 결과를 버림 | 경고 ``unused `Poll` that must be used`` (`unused_must_use`) | `Pending`을 처리하거나, 일부러 버린다면 `let _ = ..` (2.2절) |
| `let _ = work();`로 퓨처를 버림 | Clippy `let_underscore_future`: ``non-binding `let` on a future``. 퓨처는 실행되지 않음 | `.await`하거나 실행기에 넘김 (Part 0 ch16 4.5절) |
| `async { work() }`처럼 퓨처를 돌려주는 async 블록 | Clippy `async_yields_async`(기본 deny): `an async construct yields a type which is itself awaitable` | `async { work().await }` |
| `cx.waker().clone().wake()` | Clippy `waker_clone_wake`: ``cloning a `Waker` only to wake it`` | `cx.waker().wake_by_ref()` (2.3절) |
| 빈 `wake`를 가진 `impl Wake` | Clippy `manual_noop_waker`: `manual implementation of a no-op waker` | `Waker::noop()` (2.3절) |
| `thread::spawn(..)`의 `JoinHandle`을 `.await` | E0277 `` `JoinHandle<i32>` is not a future ``, ``must be a future or must implement `IntoFuture` to be awaited`` | 스레드의 결과는 05의 oneshot 같은 퓨처로 받음 (2.6절) |
| `Pending`을 돌려주면서 웨이커를 저장하지 않음 | 컴파일은 되지만 실행기가 영원히 잠듦 | 저장하고 일이 끝나면 `wake` (04의 3번 구역) |
| 끝난 퓨처를 다시 `poll` | 컴파일은 되지만 대개 패닉 | 끝났는지 기억해 두고 다시 폴링하지 않음 (`Join`의 `Option` 칸) |
| 기다리는 동안 `wake_by_ref`를 계속 부름 | 컴파일은 되지만 바쁜 폴링으로 CPU를 계속 씀 | 기다리는 일을 끝낸 쪽이 깨우게 함 (04의 4번 구역, 2.6절) |
| 완료 확인과 웨이커 저장 사이에 잠금을 놓음 | 컴파일은 되지만 드물게 깨움을 놓치고 영원히 잠듦 | 같은 잠금 안에서 확인하고 저장 (2.6절) |

## 5. 연습 문제

### 연습 1. 폴링 횟수를 세는 감싸개

어떤 퓨처든 감싸서 결과와 함께 폴링된 횟수를 돌려주는 `CountPolls<F>`를 만드세요. `count_polls(fut).await`의 결과는 `(F::Output, u32)`입니다. [`05_timer_and_oneshot.rs`](examples/05_timer_and_oneshot.rs)에 추가해 `sleep(30ms)`(2번)과, [`04_block_on.rs`](examples/04_block_on.rs)의 `BusySleep`을 옮겨 와 감싼 것(1000번 이상)을 비교하세요. unsafe 코드 없이 작성해야 합니다.

<details>
<summary>힌트</summary>

- 필드는 `inner: Pin<Box<F>>`와 `polls: u32`입니다. `Pin<Box<F>>`는 언제나 `Unpin`이므로 `CountPolls<F>`도 `F`와 관계없이 `Unpin`이고, `poll` 안에서 `self.get_mut()`을 쓸 수 있습니다(2.7절).
- `poll`은 `this.polls += 1;` 뒤에 `let output = ready!(this.inner.as_mut().poll(cx));`로 안쪽을 폴링하고 `Poll::Ready((output, this.polls))`를 돌려줍니다. 깨우는 일은 안쪽 퓨처가 같은 `cx`로 이미 맡았으므로 감싸개는 할 일이 없습니다.
- 05의 `Sleep`은 이미 폴링 횟수를 돌려주므로 `Output`이 `(u32, u32)`가 됩니다. 두 숫자가 같은지도 확인해 보세요.

</details>

### 연습 2. 직접 만든 `select`와 `timeout`

[`07_join_and_tools.rs`](examples/07_join_and_tools.rs)의 `Join`을 참고해, 두 퓨처 중 **먼저 끝나는 쪽**의 결과를 돌려주는 `select(a, b)`를 만드세요. 결과 타입은 `enum Either<L, R> { Left(L), Right(R) }`입니다. 이것으로 `timeout(future, limit) -> Result<T, Duration>`을 만들어 `timeout(sleep(50ms), 20ms)`는 `Err`, `timeout(sleep(10ms), 50ms)`는 `Ok`가 되는지 확인하세요.

<details>
<summary>힌트</summary>

- `Select<A, B>`는 `a: Pin<Box<A>>`, `b: Pin<Box<B>>`만 가지면 됩니다. 출력을 보관할 칸이 없으므로 `A::Output: Unpin` 조건도 필요 없습니다.
- `poll`에서 `a`를 먼저 폴링해 `Ready(v)`면 `Ready(Either::Left(v))`, 아니면 `b`를 폴링해 `Ready(w)`면 `Ready(Either::Right(w))`, 둘 다 아니면 `Pending`입니다. 둘이 동시에 준비되면 언제나 `a`가 이기므로 공정하지 않습니다(Part 0 ch16 2.7절의 `select`와 같습니다). `poll_fn` 문서의 `naive_select` 예제도 같은 모양입니다.
- 진 퓨처는 `select`가 끝나 `Select`가 버려질 때 함께 버려집니다. 이것이 Rust 비동기의 **취소**입니다. 버려진 `Sleep`의 타이머 스레드는 시간이 되면 저장되어 있던 웨이커를 한 번 더 부르고 끝납니다. 필요 없어진 웨이커가 한 번 더 불리는 셈이지만, `block_on`은 `woken` 표시를 확인해 한 번 더 폴링하거나 다시 잠들 뿐이라 문제가 되지 않습니다(2.5절).
- `timeout`은 `async fn`으로 쓰면 간단합니다. `match select(future, sleep(limit)).await { Either::Left(v) => Ok(v), Either::Right(()) => Err(limit) }`.

</details>

### 연습 3. 개수가 정해지지 않은 퓨처 함께 기다리기

`join_all(futures: Vec<Pin<Box<dyn Future<Output = T>>>>) -> JoinAll<T>`를 만드세요. 모든 퓨처가 끝나면 넣은 순서대로 결과를 담은 `Vec<T>`를 돌려줍니다. 서로 다른 `async` 블록 세 개(각각 30ms·10ms·20ms를 `sleep`한 뒤 문자열을 돌려줌)를 담아 실행하고, 결과의 순서와 전체 시간(30ms 이상 60ms 미만)을 확인하세요.

<details>
<summary>힌트</summary>

- 필드는 `futures: Vec<Pin<Box<dyn Future<Output = T>>>>`와 `outputs: Vec<Option<T>>`(처음에 `None`으로 채움)입니다. `T: Unpin` 조건을 붙이면 `get_mut`을 쓸 수 있습니다.
- `poll`은 `outputs[i]`가 `None`인 것만 폴링해 끝나면 채우고, 모두 `Some`이면 `outputs.iter_mut().map(|o| o.take().unwrap()).collect()`로 꺼냅니다. `futures.iter_mut().zip(outputs.iter_mut())`로 두 `Vec`을 함께 돌면 대여 문제가 없습니다.
- 서로 다른 async 블록을 `Vec`에 담으려면 `Box::pin(async { .. }) as Pin<Box<dyn Future<Output = String>>>`처럼 타입을 맞춥니다(Part 0 ch16 4.4절, 2.7절).

</details>

### 연습 4. 실행기에 `JoinHandle` 붙이기

[`08_mini_executor.rs`](examples/08_mini_executor.rs)의 `spawn`이 `JoinHandle<T>`를 돌려주게 바꾸세요. `JoinHandle<T>`는 그 태스크의 결과(`Option<T>`)를 내놓는 퓨처입니다. 태스크 하나가 `sleep(30ms)` 뒤 `42`를 돌려주고, 다른 태스크가 그 핸들을 `.await`해 받은 값을 기록장에 남기게 해서 `Some(42)`가 기록되는지 확인하세요.

<details>
<summary>힌트</summary>

- [`05_timer_and_oneshot.rs`](examples/05_timer_and_oneshot.rs)의 `oneshot`을 옮겨 옵니다. `JoinHandle<T>`는 `Receiver<T>` 그 자체(또는 그것을 감싼 구조체)입니다.
- 원래의 `spawn`은 `spawn_unit`처럼 이름을 바꿔 두고, 새 `spawn<T: Send + 'static>(&mut self, name, future: impl Future<Output = T> + Send + 'static) -> Receiver<T>` 안에서 `let (tx, rx) = oneshot(); self.spawn_unit(name, async move { tx.send(future.await); }); rx`처럼 결과를 보내는 `async move` 블록으로 감싸 넘깁니다.
- `main`은 async가 아니므로 핸들을 `.await`할 수 없습니다. 핸들을 기다리는 쪽도 태스크로 `spawn`합니다. 핸들을 넘기려면 `Receiver<T>`가 `Send`여야 하는데, `Arc<Mutex<Slot<T>>>`이므로 `T: Send`면 됩니다.
- 결과를 보내기 전에 태스크의 퓨처가 버려지면(예: 실행기가 그 태스크를 끝까지 실행하지 않음) `Sender`의 `Drop` 덕분에 기다리는 쪽은 `None`을 받습니다.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::future`](https://doc.rust-lang.org/std/future/index.html), [`Future`](https://doc.rust-lang.org/std/future/trait.Future.html): `poll`의 Return value 절(웨이커 저장, 마지막 웨이커만 깨우기), Runtime characteristics 절(바쁜 폴링 금지, 블로킹 금지), Panics 절(끝난 뒤 다시 폴링).
  - [`IntoFuture`](https://doc.rust-lang.org/std/future/trait.IntoFuture.html): `.await`의 변환 순서와 "async 빌더" 예제. [`poll_fn`](https://doc.rust-lang.org/std/future/fn.poll_fn.html): 고정된 상태를 캡처하는 `naive_select` 예제(연습 2).
  - [`std::task`](https://doc.rust-lang.org/std/task/index.html), [`Poll`](https://doc.rust-lang.org/std/task/enum.Poll.html), [`Context`](https://doc.rust-lang.org/std/task/struct.Context.html), [`Waker`](https://doc.rust-lang.org/std/task/struct.Waker.html)(`wake`의 보장, `noop`, `clone_from` 예제), [`Wake`](https://doc.rust-lang.org/std/task/trait.Wake.html)(`block_on` 예제와 그 한계에 대한 메모).
  - [`std::pin`](https://doc.rust-lang.org/std/pin/index.html): 이동이란 무엇인가, 주소에 민감한 값, `Unpin`, 고정 투영까지 자세히 설명합니다. 문서 스스로 "unsafe 없이 `Pin`을 쓰는 사람은 자세히 읽지 않아도 된다"고 하므로 앞부분과 `Unpin` 절부터 읽습니다. [`Pin`](https://doc.rust-lang.org/std/pin/struct.Pin.html)(고정하는 세 방법), [`pin!`](https://doc.rust-lang.org/std/pin/macro.pin.html)(손으로 폴링하는 `block_on` 예제), [`Unpin`](https://doc.rust-lang.org/std/marker/trait.Unpin.html).
  - [`thread::park`](https://doc.rust-lang.org/std/thread/fn.park.html): 토큰 모델, 가짜로 돌아올 수 있다는 점, 다른 코드가 토큰을 소비할 수 있다는 경고(2.5절).
- [The Book 17.5 A Closer Look at the Traits for Async](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html): `Future`, `Pin`, `Unpin`, `Stream`을 읽는 수준으로 설명합니다. Part 0 ch16 2.11절의 기준이었고, 이 챕터는 그 다음 단계입니다.
- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/) (async book)
  - [Under the Hood: Executing Futures and Tasks](https://rust-lang.github.io/async-book/02_execution/01_chapter.html)와 이어지는 [The Future Trait](https://rust-lang.github.io/async-book/02_execution/02_future.html), [Task Wakeups with Waker](https://rust-lang.github.io/async-book/02_execution/03_wakeups.html)(스레드로 만든 타이머 퓨처), [Applied: Build an Executor](https://rust-lang.github.io/async-book/02_execution/04_executor.html)(채널로 만든 실행기): 05·08 예제와 같은 구조입니다. 다만 웨이커를 만들 때 `futures` 크레이트의 `ArcWake`를 쓰므로, 이 챕터처럼 표준 라이브러리의 `Wake`로 바꿔 읽으면 됩니다.
  - [Pinning](https://rust-lang.github.io/async-book/part-reference/pinning.html): 고정을 깊이 다루는 참고 장. 첫 요약(TL;DR) 절만 읽어도 2.7절의 내용이 정리됩니다.
- 이전 챕터: [Part 0 ch16](../../part0-getting-started/ch16-async/README.md)(async/await와 런타임), [ch11](../ch11-threads-sync/README.md)(이 챕터의 실행기가 쓰는 `Arc`·`Mutex`·채널·원자 타입), [ch12](../ch12-net/README.md)(논블로킹 소켓과 `WouldBlock`).
