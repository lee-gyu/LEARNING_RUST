# ch16. 비동기 프로그래밍: async, await, 퓨처, 스트림

> 이 챕터는 The Book **17장**에 해당합니다. 이 저장소는 Book 14장(Cargo)을 Part 3으로 옮겼기 때문에 ch14부터 저장소 챕터 번호가 Book 장 번호보다 하나 작습니다.

ch15에서는 스레드로 여러 일을 동시에 진행했습니다. 스레드는 강력하지만 하나하나가 운영체제의 자원이라, "네트워크 응답 1만 개를 기다린다"처럼 **기다리는 일이 아주 많은** 프로그램에서 기다림마다 스레드를 하나씩 쓰기는 부담스럽습니다. 이 챕터는 또 다른 방법인 비동기(asynchronous) 프로그래밍을 다룹니다. 코드에 "여기서 잠깐 멈췄다가 나중에 이어 갈 수 있다"는 지점을 표시해 두면, 런타임이 기다리는 동안 다른 일을 하도록 알아서 순서를 정해 줍니다.

- 패키지: `p0-ch16-async`
- 기준 문서: [The Book 17장 Fundamentals of Asynchronous Programming: Async, Await, Futures, and Streams](https://doc.rust-lang.org/book/ch17-00-async-await.html)
- 사용 크레이트: `trpl` 0.3.0 (예제 전용이라 `[dev-dependencies]`에 있습니다. 2.13절)
- 확인한 툴체인: rustc 1.98.1, Edition 2024, macOS
- 네트워크 대신 타이머: The Book 17.1절의 "웹 페이지 제목 가져오기" 예제는 인터넷 접속이 필요하므로, 이 챕터에서는 느린 작업을 `trpl::sleep` 타이머로 흉내 냅니다. 예제는 모두 인터넷 없이 실행되고 몇 초 안에 끝납니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 동시성과 병렬성의 차이를 설명할 수 있다. `async fn`이 "퓨처를 돌려주는 평범한 함수"로 바뀐다는 것, 퓨처는 `.await`하거나 런타임에 넘기기 전에는 실행되지 않는다는 것, `main`이 `async`일 수 없어 `trpl::block_on`으로 런타임을 시작한다는 것을 설명할 수 있다.
2. `trpl::join`·`join!`으로 여러 퓨처를 함께 기다리고, `trpl::spawn_task`로 태스크를 만들고, `trpl::select`로 먼저 끝나는 쪽을 고르는 코드를 작성할 수 있다. 이것들을 조합해 `timeout` 같은 새 비동기 함수를 만들 수 있다.
3. 비동기 채널로 퓨처 사이에 메시지를 주고받고, 수신 루프가 끝나려면 송신자가 모두 버려져야 한다는 것(`async move`)을 설명할 수 있다. 퓨처는 await 지점에서만 차례를 넘긴다는 것과, 그 사이에 오래 도는 블로킹 코드의 문제와 `yield_now`의 역할을 설명할 수 있다.
4. 스트림을 `StreamExt`의 `next`와 어댑터로 처리할 수 있다. `Future`·`Stream` 트레이트의 모양과 `Pin`·`Unpin`이 왜 필요한지(자기 참조 퓨처)를 설명하고, `Box::pin`이나 `pin!`으로 "cannot be unpinned" 에러를 고칠 수 있다.
5. 퓨처·태스크·스레드의 관계와 선택 기준을 설명하고, 스레드에서 계산한 결과를 비동기 채널로 async 코드에 넘길 수 있다.

## 2. 핵심 개념

2.1~2.4절은 비동기의 기초(학습 목표 1), 2.5~2.7절은 여러 퓨처 다루기(목표 2), 2.8~2.9절은 채널과 양보(목표 3), 2.10~2.11절은 스트림과 트레이트(목표 4), 2.12절은 태스크와 스레드(목표 5)입니다. 2.13절은 `trpl`과 실무에서 쓰는 크레이트를 비교합니다.

### 2.1 동시성과 병렬성

지금까지는 두 단어를 섞어 썼지만, 이 챕터부터는 구분해야 합니다.

- **동시성(concurrency)**: 여러 작업이 모두 끝나기 전에 번갈아 진행되는 것입니다. 한 사람이 프로젝트 두 개를 맡아, 한쪽이 막히면 다른 쪽으로 넘어가며 일하는 모습입니다. 어느 한순간에는 한 가지 일만 합니다.
- **병렬성(parallelism)**: 여러 작업이 정말 같은 순간에 실행되는 것입니다. 팀원 두 명이 프로젝트를 하나씩 맡아 각자 일하는 모습입니다.

```text
동시성: 한 실행 흐름이 번갈아 진행        병렬성: 두 실행 흐름이 같은 순간에 진행
  흐름 1: A1 B1 A2 B2 A3                   흐름 1: A1 A2 A3
                                           흐름 2: B1 B2 B3
```

CPU 코어가 하나뿐이면 동시성만 가능하고, 코어가 여럿이면 병렬로도 실행할 수 있습니다. ch15의 스레드는 운영체제가 코어에 나눠 주므로 병렬로 실행될 수 있었습니다.

동시성이 왜 필요한지는 기다림을 생각하면 분명해집니다. 영상 인코딩처럼 계산 자체가 오래 걸리는 일을 **CPU 바운드(CPU-bound)**, 네트워크 응답이나 파일처럼 데이터가 오기를 기다리는 시간이 대부분인 일을 **I/O 바운드(I/O-bound)** 라고 합니다. I/O 바운드 작업은 기다리는 동안 CPU가 놀고 있으므로, 그 시간에 다른 작업을 진행하면 훨씬 효율적입니다. 기다림마다 스레드를 만들 수도 있지만 스레드가 수천, 수만 개가 되면 자원이 문제가 됩니다. 비동기 프로그래밍은 적은 수의 스레드로 많은 기다림을 번갈아 처리하는 방법입니다. Rust의 비동기 코드는 기본적으로 동시성을 표현하고, 어떤 런타임을 쓰느냐에 따라 안에서 병렬로 실행되기도 합니다(2.12절).

### 2.2 퓨처, `async`, `.await`

**퓨처(future)** 는 "지금은 준비되지 않았지만 나중에 준비될 값"입니다. Rust에서는 `Future` 트레이트를 구현한 타입이 퓨처입니다. 퓨처를 직접 구현할 일은 드물고, 보통은 `async` 키워드로 만듭니다.

- `async` 블록: `async { ... }`는 그 자리에서 실행되지 않고, 블록 내용을 담은 퓨처 값이 됩니다.
- `async fn`: 호출하면 본문 대신 퓨처를 돌려주는 함수입니다.
- `.await`: 퓨처의 결과를 기다립니다. 결과가 아직 준비되지 않았으면 지금 퓨처를 멈추고 런타임에 차례를 넘깁니다. 이렇게 멈출 수 있는 자리를 **await 지점(await point)** 이라고 합니다.

컴파일러는 `async fn`을 "본문 전체를 `async move` 블록에 담아 돌려주는 평범한 함수"로 바꿉니다. 다음 두 함수는 같은 뜻입니다([`examples/01_async_fn_and_lazy_futures.rs`](examples/01_async_fn_and_lazy_futures.rs)).

```rust
async fn slow_add(a: u32, b: u32) -> u32 {
    trpl::sleep(Duration::from_millis(100)).await;
    a + b
}

fn slow_add_desugared(a: u32, b: u32) -> impl Future<Output = u32> {
    async move {
        trpl::sleep(Duration::from_millis(100)).await;
        a + b
    }
}
```

- 반환 타입 `impl Future<Output = u32>`는 [ch10](../ch10-generics-traits-lifetimes/README.md)의 반환 위치 `impl Trait`입니다. "`Future` 트레이트를 구현한 어떤 타입인데, 끝나면 `u32`를 내놓는다"는 뜻입니다. `Output`은 `Future` 트레이트의 연관 타입(associated type)으로, ch13에서 본 `Iterator`의 `Item`처럼 트레이트를 구현하는 쪽이 정하는 타입입니다. 연관 타입은 `ch19-advanced-features`에서 자세히 다룹니다.
- `async move`인 이유: 함수는 퓨처를 만들어 돌려주고 바로 끝납니다. 퓨처는 나중에 실행되므로 함수의 지역 변수(매개변수 `a`, `b`)를 빌려서는 안 되고, 소유권을 가져가야 합니다. ch15에서 `thread::spawn`에 `move` 클로저를 넘긴 것과 같은 이유입니다.
- Edition 2024에서는 `Future` 트레이트가 프렐류드에 들어 있어 `use` 없이 쓸 수 있습니다.
- Clippy는 이렇게 풀어 쓴 함수를 보면 "`async fn`으로 쓰라"고 경고합니다(`manual_async_fn`). 두 형태가 같다는 증거이기도 합니다. 예제에서는 비교를 위해 `#[expect(clippy::manual_async_fn)]`을 붙였습니다.

`.await`는 JavaScript나 C#처럼 앞에 붙이는 언어와 달리 **뒤에 붙는(postfix)** 키워드입니다. `fetch(url).await.text().await`처럼 메서드 호출과 자연스럽게 이어 쓸 수 있습니다. `.await`는 `async fn`이나 `async` 블록 안에서만 쓸 수 있습니다. 멈췄다가 이어 가려면 그 코드 자체가 퓨처여야 하기 때문입니다(4.1절, 4.2절).

### 2.3 퓨처는 게으릅니다

퓨처는 누군가 `.await`하거나 런타임에 넘겨 주기 전에는 **아무 일도 하지 않습니다.** ch13의 반복자가 `next`를 부르기 전에는 아무 일도 하지 않았던 것과 같습니다. `async fn`을 호출해도 본문은 한 줄도 실행되지 않습니다.

```rust
let future = slow_add(1, 2);        // 본문은 아직 실행되지 않습니다
let sum = trpl::block_on(future);   // 여기서 비로소 실행됩니다
```

[`01`](examples/01_async_fn_and_lazy_futures.rs)을 실행하면 `slow_add(1, 2)`를 호출한 직후에는 본문의 출력이 없다가, `block_on`에 넘긴 뒤에야 `[slow_add] 본문 시작`이 나옵니다. 퓨처를 기다리지 않고 버리면 본문은 끝내 실행되지 않습니다. ch15의 `thread::spawn`은 호출하는 순간 새 스레드가 실행을 시작했다는 점과 대조됩니다.

게으름 덕분에 퓨처를 만들어 두고 "언제, 어떻게 실행할지"(차례로, 함께, 먼저 끝나는 쪽만 등)를 나중에 정할 수 있습니다. 반대로 `.await`를 깜빡하면 아무 일도 일어나지 않으므로, 컴파일러가 경고를 냅니다(4.5절).

### 2.4 런타임과 `block_on`, 그리고 `main`이 `async`일 수 없는 이유

퓨처를 실제로 실행하고, 멈춘 퓨처를 기억했다가 준비되면 다시 이어 주는 코드를 **런타임(runtime)** 이라고 합니다. 런타임에서 퓨처를 실행하는 부분은 **실행기(executor)** 라고도 부릅니다. 많은 언어는 런타임을 언어에 내장하지만, Rust 표준 라이브러리에는 런타임이 없습니다. 웹 서버용, 임베디드용처럼 용도에 따라 서로 다른 장단점을 가진 런타임을 크레이트로 골라 쓰게 하려는 것입니다.

그래서 `main`은 `async`일 수 없습니다(E0752, 4.3절). `async fn main`은 "`main`을 호출하면 퓨처가 나온다"는 뜻인데, 프로그램의 출발점인 `main`이 돌려준 퓨처를 실행해 줄 주체가 없습니다. `main`은 평범한 함수로 두고, 그 안에서 런타임을 시작해 최상위 퓨처를 넘깁니다.

```rust
fn main() {
    trpl::block_on(async {
        let sum = slow_add(1, 2).await;
        println!("{sum}");
    });
}
```

`trpl::block_on`은 런타임을 만들고, 넘겨받은 퓨처가 끝날 때까지 현재 스레드를 블로킹(blocking)한 채 그 퓨처를 실행한 뒤 결과를 돌려줍니다. 동기 코드와 비동기 코드가 만나는 경계입니다. `trpl` 0.3.0 소스에서 확인한 구현은 세 줄입니다.

```rust
pub fn block_on<F: Future>(future: F) -> F::Output {
    let rt = Runtime::new().unwrap(); // tokio의 Runtime
    rt.block_on(future)
}
```

호출할 때마다 새 런타임을 만들고, 퓨처가 끝나 함수가 돌아올 때 런타임도 정리됩니다. tokio 튜토리얼의 [Hello Tokio](https://tokio.rs/tokio/tutorial/hello-tokio)는 `#[tokio::main] async fn main()`이 이것과 같은 모양(`Runtime::new().unwrap()`으로 런타임을 만들고 `block_on`)의 평범한 `main`으로 바뀐다고 설명합니다. "async `main`을 쓸 수 있는" 런타임도 사실은 이렇게 평범한 `main`을 만들어 주는 것입니다.

### 2.5 여러 퓨처를 함께 기다리기: `join`, `join!`

한 `async` 블록 안의 `.await`는 적힌 순서대로 하나씩 진행됩니다. 첫 번째 퓨처가 끝나야 두 번째 퓨처가 시작되므로, 걸리는 시간이 더해집니다. 여러 퓨처를 **함께** 진행시키려면 하나로 묶어야 합니다([`02`](examples/02_join_and_spawn_task.rs)).

```rust
let a = work("A", 300).await;          // 300ms
let b = work("B", 200).await;          // 그 뒤 200ms → 약 500ms

let (a, b) = trpl::join(work("A", 300), work("B", 200)).await;   // 약 300ms
let (x, y, z) = trpl::join!(fut1, fut2, fut3);                    // 3개 이상
```

- `trpl::join(a, b)`는 두 퓨처를 번갈아 진행시키는 새 퓨처를 만듭니다. A가 `sleep`에서 멈춘 동안 B를 진행하므로 전체 시간은 가장 긴 쪽만큼입니다. 결과는 끝난 순서가 아니라 **인자 순서대로** 튜플에 담깁니다.
- `trpl::join!(...)` 매크로는 개수가 코드에 정해진 퓨처 여러 개를 한꺼번에 기다립니다. 출력 타입이 서로 달라도 됩니다. 매크로 자체가 기다리는 코드로 펼쳐지므로 뒤에 `.await`를 붙이지 않습니다.
- 개수가 실행 중에 정해지면(`Vec`에 담긴 퓨처들) `trpl::join_all`을 씁니다(2.11절).

`join`은 공정(fair)합니다. 깨어날 때마다 아직 끝나지 않은 퓨처를 인자 순서대로 모두 확인하므로, 같은 입력이면 실행할 때마다 같은 순서로 진행됩니다. 스레드의 실행 순서는 운영체제가 정하지만, 퓨처의 실행 순서는 런타임과 `join` 같은 조합 함수가 정합니다.

### 2.6 태스크 만들기: `spawn_task`

**태스크(task)** 는 런타임이 관리하는 독립된 실행 단위입니다. `trpl::spawn_task`에 퓨처를 넘기면 런타임이 그 퓨처를 별도의 태스크로 실행하고, `thread::spawn`처럼 곧바로 핸들(`JoinHandle`)을 돌려줍니다.

```rust
let name = String::from("배경 작업");
let handle = trpl::spawn_task(async move {
    trpl::sleep(Duration::from_millis(200)).await;
    name.len()
});
// ... 그동안 다른 일 ...
let len = handle.await.unwrap();
```

- 일반 퓨처는 게으르지만, 태스크로 넘긴 퓨처는 `.await`하지 않아도 런타임이 곧바로 실행하기 시작합니다. [`02`](examples/02_join_and_spawn_task.rs)의 4번에서 `[배경 작업] 시작`이 `handle.await`보다 먼저 출력되는 것으로 확인할 수 있습니다.
- `JoinHandle`도 퓨처입니다. 태스크가 패닉하는 등 정상적으로 끝나지 못하면 `Err`가 오므로 출력 타입은 `Result`이고, 그래서 `.await.unwrap()`으로 꺼냅니다.
- 태스크는 만든 블록보다 오래 살 수 있으므로 바깥 지역 변수를 빌릴 수 없습니다. `async move`로 소유권을 넘깁니다. 빠뜨리면 E0373입니다(4.6절).
- `block_on`이 돌아오면 런타임이 정리되면서 아직 끝나지 않은 태스크도 **취소**됩니다. `main`이 끝나면 다른 스레드도 끝났던 것(ch15)과 같습니다. 끝까지 실행하려면 핸들을 `.await`합니다.

`join`과 `spawn_task`는 둘 다 여러 퓨처를 동시에 진행시키지만 차이가 있습니다. `join`으로 묶은 퓨처들은 한 퓨처로 합쳐져 **같은 태스크 안에서** 번갈아 실행됩니다. 반면 `spawn_task`로 만든 태스크는 런타임이 따로 관리하므로, 여러 스레드를 쓰는 런타임에서는 다른 태스크와 병렬로 실행될 수 있습니다(2.12절).

### 2.7 먼저 끝나는 쪽 고르기: `select`, 그리고 `timeout` 만들기

`trpl::select(a, b)`는 두 퓨처 중 **먼저 끝난 쪽**의 결과를 돌려줍니다([`03`](examples/03_select_and_timeout.rs)). 둘 다 이길 수 있으므로 `Result`가 아니라 `Either`를 씁니다.

```rust
use trpl::Either;

match trpl::select(slow, fast).await {
    Either::Left(result) => { /* 첫 번째 인자(slow)가 먼저 끝남 */ }
    Either::Right(result) => { /* 두 번째 인자(fast)가 먼저 끝남 */ }
}
```

- **진 퓨처는 버려집니다(취소).** 퓨처를 버리면 마지막으로 멈춘 await 지점에서 그대로 끝나고, 그 뒤의 코드는 실행되지 않습니다. `03`의 1번에서 300ms를 더 기다려도 느린 작업의 `끝` 줄이 나오지 않는 이유입니다. `trpl::select`는 `futures` 크레이트의 `select`를 감싼 것으로, 원래 `select`는 진 퓨처를 돌려줘 나중에 계속 기다릴 수 있게 하지만 `trpl`은 단순하게 하려고 버립니다(`trpl` 소스의 문서 주석).
- `select`는 **공정하지 않습니다.** 매번 첫 번째 인자부터 확인하므로, 둘 다 이미 준비돼 있으면 항상 왼쪽이 이깁니다(`03`의 2번).

퓨처는 값이고 제네릭 함수의 인자로 받을 수 있으므로, 이런 조합으로 새 비동기 함수를 만들 수 있습니다. The Book 17.3절의 `timeout`은 `select`와 `sleep`의 경주입니다.

```rust
async fn timeout<F: Future>(future_to_try: F, max_time: Duration) -> Result<F::Output, Duration> {
    match trpl::select(future_to_try, trpl::sleep(max_time)).await {
        Either::Left(output) => Ok(output),
        Either::Right(()) => Err(max_time),
    }
}
```

`F::Output`은 "`F`가 끝나면 내놓는 값의 타입"입니다. 시도할 퓨처를 첫 번째 인자로 둔 것은 `select`가 왼쪽부터 확인하기 때문입니다. 같은 순간에 둘 다 끝나 있으면 시간 초과보다 성공을 먼저 인정합니다. 실무에서는 같은 일을 하는 `tokio::time::timeout`을 씁니다(2.13절).

### 2.8 비동기 채널

`trpl::channel()`은 ch15의 `std::sync::mpsc::channel`처럼 송신자(`tx`)와 수신자(`rx`)를 돌려주는 다중 생산자·단일 소비자 채널입니다. 차이는 받는 쪽에 있습니다([`04`](examples/04_async_channel.rs)).

| | `std::sync::mpsc` (ch15) | `trpl::channel` |
| --- | --- | --- |
| 보내기 | `tx.send(v)` | `tx.send(v)`: 크기 제한이 없는 채널이라 기다릴 일이 없으므로 `async`가 아닌 평범한 메서드입니다. |
| 받기 | `rx.recv()`: 메시지가 올 때까지 **스레드를 블로킹** | `rx.recv().await`: 메시지가 올 때까지 **이 퓨처만 멈추고** 다른 퓨처에 차례를 넘깁니다. |
| 채널이 닫히면 | `Err` | `None` |

`recv().await`는 `Option`을 돌려주므로 `while let`으로 받습니다. 채널은 **모든 송신자가 버려져야** 닫히고, 그때 `recv`가 `None`을 돌려줘 반복이 끝납니다.

```rust
let (tx, mut rx) = trpl::channel();

let tx_fut = async move {          // tx의 소유권이 이 블록으로 들어옵니다
    for message in ["하나", "둘", "셋"] {
        tx.send(message).unwrap();
        trpl::sleep(Duration::from_millis(100)).await;
    }
};                                 // 블록이 끝나면 tx가 버려지고 채널이 닫힙니다

let rx_fut = async {
    while let Some(message) = rx.recv().await {
        println!("받음: {message}");
    }
};

trpl::join(tx_fut, rx_fut).await;
```

- 보내는 코드와 받는 코드를 **한 블록에** 쓰면 `.await`가 순서대로 진행되므로, 다 보낸 뒤에야 받기 시작합니다. 번갈아 진행시키려면 위처럼 퓨처를 나눠 `join`합니다.
- `tx_fut`에서 `move`를 빼면 블록은 `tx`를 빌리기만 합니다. 블록이 끝나도 `tx`는 바깥에 살아 있으므로 채널이 닫히지 않고, `rx_fut`의 `while let`은 영원히 다음 메시지를 기다립니다. 컴파일 에러가 나지 않는 실수라 더 위험합니다. `04`의 3번은 이 상황을 500ms 제한(`select` + `sleep`)으로 감싸 직접 보여 줍니다.
- 송신자를 늘리려면 `tx.clone()`을 쓰고, 각 생산자 블록이 모두 `async move`여야 합니다. 하나라도 남아 있으면 채널이 닫히지 않습니다.

### 2.9 await 지점에서만 차례가 넘어갑니다: `yield_now`

스레드는 운영체제가 언제든 강제로 멈추고 다른 스레드를 실행합니다(선점형, preemptive). 퓨처는 다릅니다. 런타임은 **await 지점에서만** 퓨처를 멈추고 다른 퓨처로 넘어갈 수 있습니다. 각 퓨처가 스스로 차례를 넘겨야 하므로 이를 **협력적 멀티태스킹(cooperative multitasking)** 이라고 합니다. await 지점 사이의 코드는 평범한 동기 코드라서, 그 사이에서 오래 걸리면 같은 태스크의 다른 퓨처는 전혀 진행하지 못합니다. 이를 **굶주림(starvation)** 이라고 합니다([`05`](examples/05_yield_now.rs)).

```rust
fn slow(name: &str, ms: u64) {
    thread::sleep(Duration::from_millis(ms));   // 퓨처가 아니라 스레드를 블로킹합니다
    println!("[{name}] {ms}ms 계산");
}

let a = async {
    slow("a", 30);
    trpl::yield_now().await;   // 여기서 차례를 넘깁니다
    slow("a", 10);
    // ...
};
```

- `thread::sleep`은 스레드 자체를 블로킹하는 평범한 함수이고, `trpl::sleep`은 "이만큼 뒤에 깨워 달라"고 런타임에 맡기는 퓨처입니다. 이름은 비슷하지만 async 코드 안에서의 의미는 정반대입니다. 계산이 오래 걸리는 함수, 블로킹 I/O 함수도 `thread::sleep`과 같은 문제를 일으킵니다.
- `trpl::yield_now().await`는 "지금 바로 차례를 넘기겠다"는 뜻의 퓨처입니다. 기다릴 시간이 없으므로 `sleep(1ms)`로 흉내 내는 것보다 의도가 분명하고 빠릅니다.
- `yield_now`는 번갈아 진행되게 할 뿐 계산을 나누지는 않습니다. `05`의 1번과 2번은 출력 순서만 다르고 걸린 시간(약 200ms)은 같습니다. 한 스레드에서 차례로 계산하기 때문입니다.
- 블로킹 코드는 타이머도 늦춥니다. `05`의 3번에서 50ms 뒤에 깨워 달라고 한 퓨처는, 같은 태스크의 다른 퓨처가 300ms 동안 스레드를 붙잡고 있었기 때문에 약 300ms에야 깨어납니다.

양보에도 비용이 있으므로 무조건 자주 넣는 것이 좋지는 않습니다. 정말 무거운 계산이라면 async 코드 안에서 쪼개기보다 스레드에 맡기는 편이 낫습니다(2.12절, 연습 4).

### 2.10 스트림: 비동기 반복자

**스트림(stream)** 은 시간에 걸쳐 값을 하나씩 내놓는 "비동기 반복자"입니다. 반복자의 `next()`는 값을 바로 돌려주지만, 스트림의 `next()`는 퓨처라서 `.await`로 기다립니다([`06`](examples/06_streams.rs)).

```rust
use trpl::StreamExt;   // next, map, filter 등은 이 트레이트에 있습니다

let mut stream = trpl::stream_from_iter(["사과", "배", "감"]);
while let Some(fruit) = stream.next().await {
    println!("{fruit}");
}

let squares: Vec<i32> = trpl::stream_from_iter(1..=10)
    .filter(|n| n % 2 == 0)
    .map(|n| n * n)
    .collect()      // collect도 퓨처를 돌려줍니다
    .await;
```

- 스트림을 만드는 방법: `trpl::stream_from_iter`는 반복자를, `trpl::ReceiverStream::new(rx)`는 채널 수신자를 스트림으로 바꿉니다. 수신자 스트림은 메시지가 도착할 때마다 항목을 내놓고, 송신자가 모두 버려지면 끝납니다.
- `next`와 어댑터는 `StreamExt` 트레이트에 있으므로 `use trpl::StreamExt;`가 필요합니다. 빠뜨리면 E0599입니다(4.8절). 이름의 `Ext`는 확장(extension)의 줄임말입니다. `Stream` 트레이트에는 구현하는 쪽이 꼭 작성해야 하는 메서드(`poll_next`, 2.11절)만 두고, 편의 메서드는 모든 `Stream` 타입에 포괄 구현(ch10)된 확장 트레이트에 모아 두는 관례입니다.
- 반복자에 있던 `map`, `filter`, `take`, `collect` 등과 함께 시간을 다루는 어댑터도 있습니다. `timeout`은 다음 항목이 정해진 시간 안에 오지 않으면 `Err`를 내놓고 계속 기다리며, `merge`는 두 스트림을 하나로 합쳐 어느 쪽이든 먼저 나온 항목을 내놓습니다.
- `timeout`처럼 안에 타이머를 품은 스트림은 `Unpin`이 아니어서, `next`를 부르기 전에 `pin!`으로 고정해야 합니다(2.11절, 4.7절).

`Stream`과 `StreamExt`는 아직 표준 라이브러리에 없습니다. `trpl`은 `tokio-stream` 크레이트의 것을 다시 내보냅니다.

### 2.11 비동기 트레이트 들여다보기: `Future`, `Pin`, `Unpin`, `Stream`

이 절은 직접 구현하지 않고 **읽을 수 있을 정도**를 목표로 합니다. 퓨처와 실행기를 직접 만드는 것은 Part 1 `ch13-async-primitives`에서 다룹니다.

#### `Future` 트레이트와 `Poll`

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

런타임은 퓨처의 `poll`을 불러 "다 됐니?"라고 묻습니다. 준비됐으면 `Poll::Ready(값)`, 아직이면 `Poll::Pending`이 옵니다. `Pending`이면 런타임은 그 퓨처를 잠재워 두고 다른 퓨처를 실행하다가, 퓨처가 기다리던 일(타이머, 메시지 도착 등)이 일어나면 다시 `poll`합니다. 이 "다시 깨우기"에 쓰이는 것이 `cx: &mut Context`입니다. `.await`는 결국 이 `poll` 호출로 바뀝니다.

컴파일러는 `async` 블록마다 await 지점을 경계로 나눈 **상태 기계(state machine)** 를 만듭니다. 개념적으로는 "어느 await 지점에서 멈췄는지"를 배리언트로, 그때 살아 있어야 하는 지역 변수를 필드로 가진 열거형입니다. [`07`](examples/07_pin_and_join_all.rs)의 1번은 이것을 크기로 보여 줍니다. 1024바이트 배열을 await 뒤에서도 쓰는 퓨처는 1056바이트였고, await 전에 다 쓰는 퓨처는 24바이트였습니다(rustc 1.98.1, `trpl` 0.3.0에서 확인한 값으로 버전에 따라 다를 수 있습니다). await 너머로 들고 가야 하는 값만 퓨처 안에 저장되는 것입니다. 이 타입은 컴파일러가 만든 이름 없는 타입이라, 내용이 똑같은 `async` 블록 두 개도 서로 다른 타입입니다(4.4절).

#### `Pin`이 필요한 이유: 자기 참조 퓨처

`07`의 `keeps_data`는 자기 안의 `data`를 가리키는 참조 `first`를 await 너머로 들고 갑니다. 퓨처가 자기 자신을 가리키는 **자기 참조(self-referential)** 상태가 되는 것입니다. 이런 값을 메모리의 다른 곳으로 옮기면(이동) 참조는 옛 주소를 그대로 가리키게 됩니다.

```text
옮기기 전 (퓨처가 0x1000에 있음)        옮긴 뒤 (퓨처가 0x2000으로 이동)
┌──────────────────────────┐            ┌──────────────────────────┐
│ data:  [7, 7, 7, ...]    │◄─┐         │ data:  [7, 7, 7, ...]    │
│ first: 0x1000 ───────────┼──┘         │ first: 0x1000 ───────────┼──► 옛 자리 (이미 무효)
└──────────────────────────┘            └──────────────────────────┘
```

[ch4](../ch4-ownership/README.md)에서 본 댕글링 참조와 같은 문제입니다. 그래서 퓨처는 처음 `poll`된 뒤로는 **옮기면 안 됩니다.**

- **`Pin<P>`**: 포인터 `P`(`&mut T`, `Box<T>` 등)를 감싸서 "가리키는 값은 이제 제자리에서 움직이지 않는다"고 보장합니다. 고정되는 것은 포인터가 아니라 가리키는 값입니다. `Pin<Box<T>>`에서 `Box`는 옮겨져도 힙에 있는 `T`는 그대로입니다. `Future::poll`이 `self: Pin<&mut Self>`를 받는 것은 "고정된 퓨처만 실행할 수 있다"는 뜻입니다.
- **`Unpin`**: "고정된 상태에서도 옮겨도 괜찮은 타입"이라는 표시입니다. `Send`·`Sync`(ch15)처럼 메서드가 없는 마커 트레이트이고 컴파일러가 자동으로 구현합니다. `i32`, `String`, `Vec`처럼 대부분의 타입은 `Unpin`이지만, `async` 블록이 만드는 퓨처는 자기 참조가 생길 수 있으므로 `Unpin`이 아닙니다.
- 처음 `poll`하기 전에는 자기 참조가 아직 없으므로 퓨처를 자유롭게 옮겨도 됩니다. 퓨처를 변수에 담았다가 함수에 넘기거나 `Vec`에 넣는 것이 괜찮은 이유입니다.

평소에는 직접 고정할 일이 없습니다. `.await`, `join`, `select`, `join_all`이 내부에서 알아서 고정하기 때문입니다. 직접 고정해야 하는 경우는 대표적으로 두 가지입니다.

1. **서로 다른 퓨처를 한 컬렉션에 담을 때**: `async` 블록마다 타입이 다르므로 공통 타입인 `dyn Future<Output = ()>`(트레이트 객체, `ch17-trait-objects-oop`에서 다룹니다)로 묶어야 합니다. 이때 `Box<dyn Future>`는 `Future`를 구현하지 않으므로(`Box<T>`는 `T: Unpin`일 때만 `Future`입니다) E0277이 납니다(4.7절). `Box::pin`으로 고정한 `Pin<Box<dyn Future<Output = ()>>>`를 쓰거나, 힙 할당 없이 `pin!` 매크로로 지금 자리에 고정한 `Pin<&mut dyn Future<Output = ()>>`를 씁니다(`07`의 3·4번).
2. **`Unpin`이 아닌 스트림에 `next`를 부를 때**: `StreamExt::next`는 `Self: Unpin`을 요구합니다. `pin!(stream)`으로 고정하면 `Pin<&mut 스트림>`은 `Unpin`이므로 `next`를 부를 수 있습니다(`06`의 4번).

같은 `async fn`을 여러 번 호출해 만든 퓨처들은 타입이 같으므로 `Vec`에 그대로 담아 `trpl::join_all`에 넘길 수 있습니다(`07`의 2번). `join_all`을 쓸 때 직접 고정해야 하는 것은 타입이 다른 퓨처를 섞을 때뿐입니다.

#### `Stream` 트레이트

`trpl`이 쓰는 `Stream` 트레이트(`futures-core` 크레이트에 정의되어 있고 `tokio-stream`이 다시 내보냄)의 핵심은 다음과 같습니다.

```rust
pub trait Stream {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}
```

`Iterator::next`의 `Option<Item>`과 `Future::poll`의 `Poll`을 합친 모양입니다. 바깥 `Poll`은 "다음 항목이 준비됐는가", 안쪽 `Option`은 "항목이 더 있는가"를 나타냅니다. 우리가 부른 `next`는 `StreamExt`에 있는 `fn next(&mut self) -> Next<'_, Self> where Self: Unpin`이고, `Next`는 `poll_next`를 불러 주는 퓨처입니다.

### 2.12 퓨처, 태스크, 스레드

세 가지는 동시성의 단위가 서로 다릅니다.

| 단위 | 관리하는 쪽 | 동시성이 생기는 곳 |
| --- | --- | --- |
| 스레드 | 운영체제 | 스레드 **사이** |
| 태스크 | 런타임 | 태스크 사이, 그리고 한 태스크 **안**에서 `join`·`select`로 묶은 퓨처 사이 |
| 퓨처 | (태스크 안에서 실행됨) | 가장 작은 단위. 퓨처 하나가 다른 퓨처 여럿을 품을 수 있습니다 |

런타임의 실행기는 태스크를 관리하고, 태스크는 퓨처를 실행합니다. 태스크는 운영체제가 아니라 라이브러리 코드가 관리하므로 가볍습니다. [`08`](examples/08_tasks_and_threads.rs)의 1번은 태스크 1만 개가 각자 100ms씩 기다리는데도 전체가 약 100ms에 끝나는 것을 보여 줍니다. 스레드 1만 개를 만들었다면 스레드마다 운영체제 자원을 따로 잡아야 합니다.

`trpl::block_on`이 만드는 tokio 런타임은 여러 작업자 스레드(worker thread)를 씁니다. `08`의 2번에서 `block_on`에 넘긴 퓨처와 `join`으로 묶은 퓨처는 `main` 스레드에서, `spawn_task`로 만든 태스크는 `tokio-rt-worker`라는 이름의 작업자 스레드에서 실행됩니다(이 이름은 tokio가 정한 것으로 버전에 따라 다를 수 있습니다). 그래서 태스크끼리는 병렬로 실행될 수 있습니다. The Book은 많은 런타임이 바쁜 스레드의 태스크를 한가한 스레드로 옮기는 작업 훔치기(work stealing)를 쓴다고 설명합니다.

The Book이 제시하는 선택 기준은 다음과 같습니다.

- **계산이 많고 잘 나뉘는 일(CPU 바운드)**: 스레드. 예: 데이터를 여러 조각으로 나눠 각각 처리하기.
- **기다림이 많고 동시에 많이 일어나는 일(I/O 바운드)**: async. 예: 여러 곳에서 서로 다른 때에 도착하는 메시지 처리하기.
- **둘 다 필요하면 함께** 씁니다.

함께 쓰는 대표적인 방법은 무거운 계산을 스레드에 맡기고 결과를 비동기 채널로 받는 것입니다. 비동기 채널의 `send`는 `async`가 아니므로 평범한 스레드에서 부를 수 있고, 받는 쪽은 `recv().await`로 다른 퓨처를 막지 않고 기다립니다(`08`의 3번).

```rust
let (tx, mut rx) = trpl::channel();
thread::spawn(move || {
    let sum: u64 = (1..=1_000_000u64).sum();   // 무거운 계산은 스레드에서
    tx.send(sum).unwrap();
});
trpl::block_on(async {
    while let Some(sum) = rx.recv().await {    // 결과는 async 코드에서 기다림
        println!("{sum}");
    }
});
```

tokio를 직접 쓸 때는 블로킹 작업을 런타임이 관리하는 별도 스레드에서 실행하는 `tokio::task::spawn_blocking`도 있습니다(`trpl`은 이 함수를 내보내지 않습니다).

### 2.13 `trpl`과 실무에서 쓰는 크레이트

`trpl`은 The Book을 위해 만든 학습용 크레이트입니다. 대부분 다른 크레이트의 항목을 다시 내보낼 뿐이며, 주로 `tokio`(가장 널리 쓰이는 비동기 런타임), `futures`(퓨처 조합 도구), `tokio-stream`(스트림)에서 가져옵니다. Book 17.1절의 웹 예제를 위한 `get`, `Html`도 있어서 HTTP 클라이언트(`reqwest`)와 HTML 파서(`scraper`)까지 의존하므로, 처음 빌드할 때 100개가 넘는 크레이트를 내려받아 컴파일합니다.

이 저장소에서는 이미 추가해 두었으므로 따로 실행할 필요는 없습니다. 추가할 때 쓴 명령과 결과는 다음과 같습니다.

```sh
cargo add --dev trpl -p p0-ch16-async
```

```toml
[dev-dependencies]
trpl = "0.3.0"
```

실무에서는 `trpl` 대신 `tokio`를 직접 의존성에 넣고, 필요하면 `futures`와 `tokio-stream`을 함께 씁니다. 아래 표의 가운데 열은 `trpl` 0.3.0 소스(`src/lib.rs`)에서 확인한 것입니다. 오른쪽 열의 `#[tokio::main]`, `tokio::spawn`, `tokio::join!`, `tokio::select!`, `tokio::time::timeout`, `unbounded_channel`, `tokio_stream::iter`, `futures::future::join_all`은 tokio 1.54.1·tokio-stream 0.1.19·futures 0.3.34로 작은 프로젝트를 만들어 직접 빌드·실행해 보았습니다.

| `trpl` | 실제로 가리키는 항목 | tokio로 직접 쓸 때 |
| --- | --- | --- |
| `block_on(f)` | `tokio::runtime::Runtime::new().unwrap().block_on(f)` | `#[tokio::main] async fn main()` |
| `spawn_task` | `tokio::task::spawn` | `tokio::spawn` (같은 함수) |
| `sleep` | `tokio::time::sleep` | 같음 |
| `yield_now` | `tokio::task::yield_now` | 같음 |
| `channel`, `Sender`, `Receiver` | `tokio::sync::mpsc::unbounded_channel`, `UnboundedSender`, `UnboundedReceiver` | 같음. 크기 제한이 있는 `tokio::sync::mpsc::channel(n)`도 있습니다 |
| `join`, `join3`, `join_all`, `join!` | `futures::future::join`, `join3`, `join_all`, `futures::join!` | `tokio::join!` 또는 `futures` 그대로 |
| `select`, `Either` | `futures::future::select`를 감싸 진 퓨처를 버리게 한 함수, `futures::future::Either` | `tokio::select!` 매크로 |
| (없음, 2.7절에서 직접 만듦) | | `tokio::time::timeout` |
| `stream_from_iter`, `Stream`, `StreamExt` | `tokio_stream::iter`, `Stream`, `StreamExt` | 같음 |
| `ReceiverStream`, `interval`, `IntervalStream` | `tokio_stream::wrappers::UnboundedReceiverStream`, `tokio::time::interval`, `tokio_stream::wrappers::IntervalStream` | 같음 |

- `trpl::channel`이 크기 제한 없는 채널을 쓰는 이유는 `std::sync::mpsc::channel`과 이름·동작을 맞추기 위해서라고 `trpl` 소스 주석에 적혀 있습니다. tokio에서는 `channel`이 크기 제한이 있는 채널이고, 제한 없는 채널은 `unbounded_channel`입니다.
- `#[tokio::main]`을 쓰려면 tokio의 기능을 켜야 합니다. tokio 튜토리얼은 `tokio = { version = "1", features = ["full"] }`을 씁니다.

> **Book 및 다른 자료와 달라진 점**: `trpl` 0.3.0에서 `run`이 `block_on`으로, `race`가 `select`로 이름이 바뀌었고 현재 The Book은 새 이름을 씁니다. 이전 판의 Book이나 그것을 따른 자료에는 `trpl::run`, `trpl::race`로 적혀 있을 수 있습니다. `trpl` 0.3.0은 옛 이름도 같은 동작으로 남겨 두었으므로(CHANGELOG에는 "하위 호환 릴리스"라고 적혀 있습니다) 그대로 빌드됩니다. 또 Book이 인용한 에러 메시지는 의존 크레이트 버전이 달라 이 저장소에서 본 것과 조금 다릅니다. 4.7절의 `join_all` 에러는 Book에는 하나만 인용되어 있지만 여기서는 같은 메시지의 에러가 세 번 나오고, 4.8절의 E0599는 Book보다 제안 목록이 짧습니다.

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다. 처음 실행할 때는 `trpl`과 의존 크레이트를 내려받아 컴파일하느라 시간이 걸립니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_async_fn_and_lazy_futures.rs`](examples/01_async_fn_and_lazy_futures.rs) | `async fn`과 풀어 쓴 `impl Future` 함수, 퓨처의 게으름, `block_on` | 1 |
| [`02_join_and_spawn_task.rs`](examples/02_join_and_spawn_task.rs) | 차례로 await vs `join`, `join!`, `spawn_task`와 `JoinHandle`, 태스크 취소 | 2 |
| [`03_select_and_timeout.rs`](examples/03_select_and_timeout.rs) | `select`와 `Either`, 진 퓨처의 취소, `select`의 불공정함, 직접 만든 `timeout` | 2 |
| [`04_async_channel.rs`](examples/04_async_channel.rs) | `trpl::channel`, `async move`와 채널 닫힘, 끝나지 않는 수신 루프, 여러 생산자 | 3 |
| [`05_yield_now.rs`](examples/05_yield_now.rs) | 블로킹 코드로 인한 굶주림, `yield_now`, 늦어지는 타이머 | 3 |
| [`06_streams.rs`](examples/06_streams.rs) | `stream_from_iter`, `StreamExt` 어댑터, `ReceiverStream`, `timeout` 스트림과 `pin!` | 4 |
| [`07_pin_and_join_all.rs`](examples/07_pin_and_join_all.rs) | 상태 기계로서의 퓨처 크기, `join_all`, `Box::pin`, `pin!` | 4 |
| [`08_tasks_and_threads.rs`](examples/08_tasks_and_threads.rs) | 태스크 1만 개, 퓨처·태스크가 실행되는 스레드, 스레드 + 비동기 채널 | 5 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 4개 (4절) | 1, 4 |

**출력에 대한 주의**: 비동기 예제의 출력에는 시간과 실행 순서가 들어갑니다. 예제는 다음 원칙으로 만들었습니다.

- 걸린 시간은 실행마다 몇 ms씩 달라지므로 100ms 단위로 반올림해 `약 300ms`처럼 출력합니다.
- 순서가 타이머로 정해지는 줄은 대기 시간을 50ms 이상 벌려 두어 보통 아래와 같이 나옵니다. 하지만 컴퓨터가 몹시 바쁘면 달라질 수 있습니다. 어느 부분이 그런지는 각 예제 파일 맨 위의 "주의"에 적었습니다.
- 타이머가 아니라 await 지점의 위치로 순서가 정해지는 줄(`03`의 2번, `05` 전체 등)은 실행마다 같습니다.

### 01_async_fn_and_lazy_futures — `async fn`, 게으른 퓨처, `block_on`

`slow_add`는 `async fn`이고, `slow_add_desugared`는 컴파일러가 바꾸는 모양을 손으로 쓴 함수입니다. 1번과 2번 사이에 `[slow_add]` 줄이 없다는 것, 5번에서 버린 퓨처의 `[slow_add]` 줄이 끝까지 나오지 않는다는 것을 확인하세요.

```sh
cargo run -p p0-ch16-async --example 01_async_fn_and_lazy_futures
```

```text
1. async fn을 호출하면 퓨처만 만들어집니다
    slow_add(1, 2)를 호출했지만 본문은 아직 한 줄도 실행되지 않았습니다
2. block_on에 넘기면 그제야 실행됩니다
    [slow_add] 본문 시작: 1 + 2
    [slow_add] 본문 끝
    block_on이 돌려준 값: 3
3. async 블록 안에서는 .await로 다른 퓨처의 결과를 기다립니다
    [slow_add] 본문 시작: 1 + 2
    [slow_add] 본문 끝
    [slow_add_desugared] 본문 시작: 3 + 10
    [slow_add_desugared] 본문 끝
    결과: 26
4. async 블록도 만들기만 해서는 실행되지 않습니다
    async 블록을 변수에 담았습니다
    [async 블록] 실행됨
    block_on이 돌려준 값: 블록의 결과
5. 기다리지 않고 버린 퓨처는 끝내 실행되지 않습니다
    퓨처를 버렸습니다. [slow_add] 줄은 출력되지 않습니다
```

### 02_join_and_spawn_task — `join`, `join!`, `spawn_task`

1번과 2번은 같은 두 작업을 차례로 기다릴 때와 `join`으로 함께 기다릴 때의 시간을 비교합니다. 2번에서 A가 먼저 시작했지만 B가 먼저 끝나고, 결과 튜플은 그래도 인자 순서 `(300, 200)`이라는 점을 보세요. 4번은 태스크가 `.await` 전에 이미 실행 중이라는 것을, 5번은 기다리지 않은 태스크가 `block_on`과 함께 사라진다는 것을 보여 줍니다.

```sh
cargo run -p p0-ch16-async --example 02_join_and_spawn_task
```

```text
1. 차례로 await하면 시간이 더해집니다
    [A] 시작
    [A] 끝 (300ms)
    [B] 시작
    [B] 끝 (200ms)
    결과 (300, 200), 걸린 시간: 약 500ms
2. join으로 묶으면 함께 기다리므로 가장 긴 쪽만큼 걸립니다
    [A] 시작
    [B] 시작
    [B] 끝 (200ms)
    [A] 끝 (300ms)
    결과 (300, 200), 걸린 시간: 약 300ms
3. join!은 개수가 정해진 퓨처 여러 개를 한꺼번에 기다립니다
    [C] 시작
    [C] 끝 (100ms)
    결과: (100, 문자열, true)
4. spawn_task는 퓨처를 별도의 태스크로 실행합니다
    [배경 작업] 시작
    [main] 그동안 100ms짜리 다른 일을 했습니다
    [배경 작업] 끝
    태스크가 돌려준 값: 13, 걸린 시간: 약 200ms
5. block_on이 끝나면 기다리지 않은 태스크는 함께 사라집니다
    [main] 50ms 뒤 block_on을 끝냅니다
    block_on이 돌아왔습니다
```

`13`은 `"배경 작업"`의 UTF-8 바이트 수입니다(한글 4자 × 3바이트 + 공백 1바이트, [ch8](../ch8-collections/README.md)).

### 03_select_and_timeout — `select`와 직접 만든 `timeout`

1번에서 진 퓨처(느린 작업)의 `끝` 줄이 끝까지 나오지 않는 것은 `select`가 진 퓨처를 버렸기 때문입니다. 2번은 타이머 없이 둘 다 바로 끝나는 퓨처를 경주시켜, `select`가 항상 첫 번째 인자부터 확인한다는 것을 보여 줍니다. 3번의 두 번째 호출에서도 `[긴 작업] 끝` 줄이 나오지 않습니다.

```sh
cargo run -p p0-ch16-async --example 03_select_and_timeout
```

```text
1. select는 먼저 끝난 쪽의 결과만 돌려줍니다
    [빠른 작업] 끝 (100ms)
    오른쪽(빠른 작업)이 이겼습니다: 빠른 작업의 결과
    300ms를 더 기다렸지만 [느린 작업] 끝 줄은 나오지 않았습니다
2. 둘 다 바로 끝나는 퓨처라면 항상 첫 번째 인자가 이깁니다
    1회: 왼쪽
    2회: 왼쪽
    3회: 왼쪽
3. select와 sleep을 조합해 timeout을 만듭니다
    [짧은 작업] 끝 (100ms)
    성공: 짧은 작업의 결과
    200ms 안에 끝나지 않아 포기했습니다
```

### 04_async_channel — 비동기 채널

2번에서 `보냄`과 `받음`이 번갈아 나오는 것은 보내는 퓨처가 `sleep`에서 멈출 때마다 `join`이 받는 퓨처를 진행시키기 때문입니다. 3번은 `move`를 뺀 버전으로, 메시지는 다 받았지만 `tx`가 살아 있어 루프가 끝나지 않습니다. 영원히 멈추지 않도록 500ms 제한을 걸어 두었습니다.

```sh
cargo run -p p0-ch16-async --example 04_async_channel
```

```text
1. send는 바로 넣고, recv는 .await로 기다립니다
    받음: 안녕
2. 보내는 퓨처와 받는 퓨처를 join하면 번갈아 진행됩니다
    보냄: 하나
    받음: 하나
    보냄: 둘
    받음: 둘
    보냄: 셋
    받음: 셋
    채널이 닫혀 수신 루프가 끝났습니다
3. move를 빼면 tx가 살아 있어 수신 루프가 끝나지 않습니다
    받음: 하나
    받음: 둘
    500ms가 지나도 끝나지 않아 포기했습니다
4. tx.clone()으로 생산자를 둘로 늘립니다
    받은 순서: ["A1", "B1", "A2", "B2", "A3", "B3"]
```

4번의 순서는 A가 0·100·200ms에, B가 50·150·250ms에 보내도록 맞춘 결과입니다.

### 05_yield_now — 굶주림과 `yield_now`

1번과 2번은 같은 계산을 하고 `yield_now`가 있느냐만 다릅니다. 출력 순서는 달라지지만 걸린 시간은 같다는 점을 확인하세요. 3번의 ticker는 50ms 뒤에 깨워 달라고 했지만 blocker가 스레드를 300ms 동안 붙잡고 있어 늦게 깨어납니다.

```sh
cargo run -p p0-ch16-async --example 05_yield_now
```

```text
1. await 지점이 없으면 한 퓨처가 끝날 때까지 다른 퓨처는 기다리기만 합니다
    [a] 시작
    [a] 30ms 계산
    [a] 10ms 계산
    [a] 20ms 계산
    [a] 끝
    [b] 시작
    [b] 70ms 계산
    [b] 10ms 계산
    [b] 60ms 계산
    [b] 끝
    걸린 시간: 약 200ms
2. yield_now로 사이사이 차례를 넘기면 번갈아 진행됩니다
    [a] 시작
    [a] 30ms 계산
    [b] 시작
    [b] 70ms 계산
    [a] 10ms 계산
    [b] 10ms 계산
    [a] 20ms 계산
    [b] 60ms 계산
    [a] 끝
    [b] 끝
    걸린 시간: 약 200ms
3. 블로킹 코드는 타이머도 늦춥니다
    [blocker] 300ms 계산
    [ticker] 50ms 타이머가 약 300ms에 깨어났습니다
```

### 06_streams — 스트림과 `StreamExt`

3번은 메시지가 100ms 간격으로 도착하는 것을, 4번은 `timeout` 어댑터가 늦은 항목 앞에 `Err`를 하나 끼워 넣고 계속 기다린다는 것을 보여 줍니다. 4번의 `pin!`을 지우고 빌드하면 4.7절의 두 번째 에러를 볼 수 있습니다.

```sh
cargo run -p p0-ch16-async --example 06_streams
```

```text
1. 반복자로 스트림을 만들고 next().await로 하나씩 꺼냅니다
    사과
    배
    감
2. 반복자처럼 어댑터를 이어 붙이고 collect로 모읍니다
    짝수의 제곱: [4, 16, 36, 64, 100]
3. 채널 수신자를 스트림으로 감싸면 메시지가 도착하는 대로 나옵니다
    약 100ms에 도착: 하나
    약 200ms에 도착: 둘
    약 300ms에 도착: 셋
    송신자가 버려져 스트림이 끝났습니다
4. timeout 어댑터는 다음 항목이 늦으면 Err를 내놓고 계속 기다립니다
    받음: 빠른 메시지 1
    200ms 동안 아무것도 오지 않았습니다
    받음: 늦은 메시지
    받음: 빠른 메시지 2
```

### 07_pin_and_join_all — 상태 기계, `join_all`, `Box::pin`, `pin!`

1번의 크기는 rustc 1.98.1, `trpl` 0.3.0에서 확인한 값이며 버전에 따라 달라질 수 있습니다. 숫자 자체보다 1024바이트 배열이 퓨처 안에 들어 있느냐 없느냐의 차이를 보세요. 2번은 끝난 순서(3, 2, 1)와 결과의 순서(`[2, 4, 6]`)가 다르다는 것을 보여 줍니다. 3번의 `Box::pin`을 `Box::new`로, 타입의 `Pin<Box<...>>`를 `Box<...>`로 바꾸면 4.7절의 첫 번째 에러를 볼 수 있습니다.

```sh
cargo run -p p0-ch16-async --example 07_pin_and_join_all
```

```text
1. 퓨처는 await 너머로 쓰는 지역 변수를 자기 안에 저장합니다
    keeps_data의 크기: 1056바이트
    drops_data의 크기: 24바이트
    [keeps_data] await 뒤에도 data를 씁니다: 7
    [drops_data] await 뒤에는 복사한 값만 씁니다: 7
2. 같은 async fn이 만든 퓨처는 타입이 같아 Vec에 담을 수 있습니다
    [double_later(3)] 끝
    [double_later(2)] 끝
    [double_later(1)] 끝
    결과: [2, 4, 6]
3. 서로 다른 async 블록은 Box::pin으로 고정해 한 Vec에 담습니다
    [greet] 안녕하세요
    [count] 1
    [count] 2
4. pin!은 힙 할당 없이 지금 자리에 고정합니다
    [first] 실행
    [second] 실행
```

### 08_tasks_and_threads — 태스크와 스레드

1번의 걸린 시간은 느린 컴퓨터에서는 더 길게 나올 수 있습니다. 그래도 태스크 1만 개 × 100ms를 더한 시간(1000초)과는 비교할 수 없이 짧습니다. 2번의 작업자 스레드 이름은 tokio 버전에 따라 다를 수 있습니다. 3번에서 스레드가 끝나는 순서는 실행마다 다를 수 있어 id 순으로 정렬해 출력합니다.

```sh
cargo run -p p0-ch16-async --example 08_tasks_and_threads
```

```text
1. 태스크 10,000개가 각자 100ms씩 기다려도 전체는 짧게 끝납니다
    결과의 합: 50005000, 걸린 시간: 약 100ms
2. 퓨처와 태스크가 실행되는 스레드
    block_on의 퓨처: main
    join한 퓨처 두 개: main, main
    spawn_task의 태스크: tokio-rt-worker
3. 무거운 계산은 스레드에서 하고, 결과는 비동기 채널로 받습니다
    스레드 1: 1부터 1000000까지의 합 = 500000500000
    스레드 2: 1부터 2000000까지의 합 = 2000001000000
    스레드 3: 1부터 3000000까지의 합 = 4500001500000
```

## 4. 자주 하는 실수

4.1~4.4절은 `compile_fail/`의 파일입니다. 외부 크레이트 없이 재현되므로 챕터 디렉터리에서 `rustc`로 직접 확인합니다.

```sh
cd part0-getting-started/ch16-async
rustc --edition 2024 compile_fail/e0728_await_outside_async.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 고쳐서 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요. 에러 코드의 설명은 `rustc --explain E0728`처럼 볼 수 있습니다.

4.5~4.8절은 `trpl`이 있어야 재현되므로 `compile_fail/`에 두지 않았습니다. 메시지는 `trpl` 0.3.0을 의존성으로 둔 새 프로젝트(`cargo new hello-async`, `cargo add trpl`)의 `src/main.rs`에서 확인한 것입니다.

### 4.1 `async`가 아닌 함수에서 `.await` — E0728

파일: [`compile_fail/e0728_await_outside_async.rs`](compile_fail/e0728_await_outside_async.rs)

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
  --> compile_fail/e0728_await_outside_async.rs:16:28
   |
15 | fn main() {
   | --------- this is not `async`
16 |     let n = fetch_number().await;
   |                            ^^^^^ only allowed inside `async` functions and blocks
```

- **읽는 법**: `^^^^^`가 문제의 `.await`를, `this is not async`가 그것을 둘러싼 함수를 가리킵니다.
- **원인**: `.await`는 "결과가 준비될 때까지 멈추고 런타임에 차례를 넘긴다"는 뜻인데, 평범한 함수는 퓨처가 아니라서 멈췄다가 이어 갈 방법이 없습니다(2.2절).
- **해결**: 동기 코드에서 퓨처의 결과가 필요하면 런타임에 맡깁니다. `let n = trpl::block_on(fetch_number());` 또는 `trpl::block_on(async { let n = fetch_number().await; ... })`처럼 씁니다. 이 함수가 다른 async 코드에서 불리는 함수라면 함수 자체를 `async fn`으로 바꿉니다.

### 4.2 `async fn` 안의 클로저에서 `.await` — E0728

파일: [`compile_fail/e0728_await_in_closure.rs`](compile_fail/e0728_await_in_closure.rs)

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
  --> compile_fail/e0728_await_in_closure.rs:19:43
   |
19 |     numbers.into_iter().map(|n| double(n).await).collect()
   |                             ---           ^^^^^ only allowed inside `async` functions and blocks
   |                             |
   |                             this is not `async`
```

- **읽는 법**: 바깥 함수 `double_all`은 `async fn`인데도 에러가 납니다. `this is not async`가 가리키는 것이 함수가 아니라 클로저 `|n|`이라는 점이 단서입니다.
- **원인**: 클로저는 그 자체로 별개의 함수이고 `async`가 아닙니다. 바깥이 `async`여도 클로저 본문 안에서는 `.await`를 쓸 수 없습니다. 반복자 어댑터(ch13)에 클로저를 넘기다가 자주 만나는 에러입니다.
- **해결**: 두 가지 방법이 있습니다. (1) `for n in numbers { result.push(double(n).await); }`처럼 `for` 반복문으로 바꿔 `async fn` 본문에서 직접 기다립니다. 하나씩 차례로 실행됩니다. (2) `map`으로는 퓨처만 만들고(`.await` 없이), 모두 한꺼번에 기다립니다. `trpl::join_all(numbers.into_iter().map(double)).await`는 모든 퓨처를 함께 진행시킵니다(연습 2).

### 4.3 `async fn main` — E0752

파일: [`compile_fail/e0752_async_main.rs`](compile_fail/e0752_async_main.rs)

```text
error[E0752]: `main` function is not allowed to be `async`
  --> compile_fail/e0752_async_main.rs:20:1
   |
20 | async fn main() {
   | ^^^^^^^^^^^^^^^ `main` function is not allowed to be `async`
```

- **원인**: `async fn main`이 돌려줄 퓨처를 실행해 줄 런타임이 표준 라이브러리에 없습니다(2.4절).
- **해결**: `main`은 평범한 `fn main()`으로 두고 본문을 `trpl::block_on(async { ... })`으로 감쌉니다. tokio를 직접 쓰면 `#[tokio::main]` 속성을 붙인 `async fn main`을 쓸 수 있는데, 이 속성이 `main`을 같은 모양의 평범한 함수로 바꿔 주는 것입니다.

### 4.4 서로 다른 `async` 블록을 한 `Vec`에 담기 — E0308

파일: [`compile_fail/e0308_async_blocks_differ.rs`](compile_fail/e0308_async_blocks_differ.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_async_blocks_differ.rs:20:31
   |
13 |     let hello = async {
   |                 ----- the expected `async` block
...
16 |     let bye = async {
   |               ----- the found `async` block
...
20 |     let futures = vec![hello, bye];
   |                               ^^^ expected `async` block, found a different `async` block
   |
   = note: expected `async` block `{async block@compile_fail/e0308_async_blocks_differ.rs:13:17: 13:22}`
              found `async` block `{async block@compile_fail/e0308_async_blocks_differ.rs:16:15: 16:20}`
   = note: no two async blocks, even if identical, have the same type
   = help: consider pinning your async block and casting it to a trait object
```

- **읽는 법**: `{async block@파일:줄:열}`은 컴파일러가 만든 이름 없는 타입을 "어디에 있는 async 블록"으로 부른 것입니다. 두 번째 `note`가 핵심입니다. 내용이 같아도 async 블록끼리는 타입이 다릅니다.
- **원인**: `async` 블록마다 고유한 상태 기계 타입이 만들어집니다(2.11절). `Vec`의 원소는 모두 같은 타입이어야 합니다.
- **해결**: `help`대로 고정한 뒤 트레이트 객체로 바꿉니다. `use std::pin::Pin;`을 추가하고 `let futures: Vec<Pin<Box<dyn Future<Output = ()>>>> = vec![Box::pin(hello), Box::pin(bye)];`로 씁니다([`07`](examples/07_pin_and_join_all.rs)의 3번). 같은 `async fn`을 여러 번 호출해 만든 퓨처라면 타입이 같으므로 이렇게 할 필요가 없습니다.

### 4.5 `.await`를 빠뜨림 — `unused_must_use` 경고

에러가 아니라 경고라서 프로그램은 실행됩니다. 하지만 `.await`를 빠뜨린 `hello()`는 아무 일도 하지 않습니다.

```rust
use std::time::Duration;

async fn hello() {
    trpl::sleep(Duration::from_millis(10)).await;
    println!("안녕하세요");
}

fn main() {
    trpl::block_on(async {
        hello();
        println!("끝");
    });
}
```

```text
warning: unused implementer of `Future` that must be used
  --> src/main.rs:10:9
   |
10 |         hello();
   |         ^^^^^^^
   |
   = note: futures do nothing unless you `.await` or poll them
   = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
```

실행하면 `끝`만 출력되고 `안녕하세요`는 나오지 않습니다.

- **원인**: `hello()`는 퓨처를 만들기만 하고, 그 퓨처는 기다려 주는 쪽 없이 바로 버려집니다(2.3절). 첫 번째 `note`의 `futures do nothing unless you .await or poll them`이 정확히 그 뜻입니다.
- **해결**: `hello().await;`로 기다립니다. `let _ = hello();`로 바꾸면 경고는 사라지지만 여전히 `끝`만 출력되고 아무 일도 일어나지 않으니 주의하세요.

### 4.6 `spawn_task`에 `move` 없이 바깥 변수 쓰기 — E0373

```rust
fn main() {
    trpl::block_on(async {
        let name = String::from("러스트");
        let handle = trpl::spawn_task(async {
            println!("안녕하세요, {name}님");
        });
        handle.await.unwrap();
    });
}
```

```text
error[E0373]: async block may outlive the current function, but it borrows `name`, which is owned by the current function
 --> src/main.rs:4:39
  |
4 |         let handle = trpl::spawn_task(async {
  |                                       ^^^^^ may outlive borrowed value `name`
5 |             println!("안녕하세요, {name}님");
  |                                    ---- `name` is borrowed here
  |
  = note: async blocks are not executed immediately and must either take a reference or ownership of outside variables they use
help: to force the async block to take ownership of `name` (and any other referenced variables), use the `move` keyword
  |
4 |         let handle = trpl::spawn_task(async move {
  |                                             ++++
```

- **읽는 법**: ch15에서 `thread::spawn`에 `move` 없는 클로저를 넘겼을 때와 거의 같은 메시지입니다. "closure" 자리가 "async block"으로 바뀌었을 뿐입니다.
- **원인**: 태스크는 런타임이 따로 실행하므로 이 블록이 끝난 뒤에도 살아 있을 수 있습니다. 그런데 async 블록은 `name`을 빌리기만 했으므로, 태스크가 실행될 때 `name`이 이미 버려졌을 수 있습니다.
- **해결**: `help`대로 `async move`로 소유권을 넘깁니다. 바깥에서도 계속 써야 하는 값이면 `let name2 = name.clone();`으로 복제해 넘기거나, ch15의 `Arc`로 공유합니다.

### 4.7 고정되지 않은 퓨처·스트림 — E0277 (cannot be unpinned)

**`Box<dyn Future>`를 `join_all`에 넘김**

```rust
use std::future::Future;

fn main() {
    trpl::block_on(async {
        let a = async { println!("a") };
        let b = async { println!("b") };
        let futures: Vec<Box<dyn Future<Output = ()>>> = vec![Box::new(a), Box::new(b)];
        trpl::join_all(futures).await;
    });
}
```

```text
error[E0277]: `dyn Future<Output = ()>` cannot be unpinned
   --> src/main.rs:8:24
    |
  8 |         trpl::join_all(futures).await;
    |         -------------- ^^^^^^^ the trait `Unpin` is not implemented for `dyn Future<Output = ()>`
    |         |
    |         required by a bound introduced by this call
    |
    = note: consider using the `pin!` macro
            consider using `Box::pin` if you need to access the pinned value outside of the current scope
    = note: required for `Box<dyn Future<Output = ()>>` to implement `Future`
note: required by a bound in `join_all`
   --> .../futures-util-0.3.34/src/future/join_all.rs:105:14
```

(같은 메시지의 E0277이 세 번 나옵니다. 경로 앞부분은 줄였습니다.)

- **읽는 법**: 아래에서 위로 읽습니다. `join_all`은 원소가 `Future`이기를 요구하고(`required by a bound in join_all`), `Box<dyn Future>`가 `Future`이려면 안의 `dyn Future`가 `Unpin`이어야 하는데(`required for Box<...> to implement Future`), 그렇지 않다는 것입니다.
- **원인**: `Box<T>`는 `T: Unpin`일 때만 `Future`를 구현합니다. `dyn Future`는 어떤 퓨처든 될 수 있으므로 `Unpin`이라고 보장할 수 없습니다(2.11절).
- **해결**: `note`의 두 제안대로 고칩니다. `Box::pin(a)`로 고정해 `Vec<Pin<Box<dyn Future<Output = ()>>>>`에 담거나, `pin!(async { ... })`로 고정해 `Vec<Pin<&mut dyn Future<Output = ()>>>`에 담습니다([`07`](examples/07_pin_and_join_all.rs)의 3·4번).

**`Unpin`이 아닌 스트림에 `next` 호출**

[`06`](examples/06_streams.rs)의 4번에서 `pin!`을 빼고 `let mut messages = ReceiverStream::new(rx).timeout(Duration::from_millis(200));`으로 쓰면 다음 에러가 납니다. 아래는 `06_streams.rs`를 새 프로젝트의 `src/main.rs`로 복사해 그렇게 고친 뒤 확인한 것으로, 길어서 중간을 줄였습니다. 같은 메시지의 에러가 두 번 나오고, 쓰이지 않게 된 `use std::pin::pin;`에 대한 경고도 함께 나옵니다.

```text
error[E0277]: `PhantomPinned` cannot be unpinned
    --> src/main.rs:82:43
     |
  82 |         while let Some(result) = messages.next().await {
     |                                           ^^^^ within `(PhantomData<&()>, PhantomPinned)`, the trait `Unpin` is not implemented for `PhantomPinned`
     |
     = note: consider using the `pin!` macro
             consider using `Box::pin` if you need to access the pinned value outside of the current scope
     = note: required because it appears within the type `(PhantomData<&()>, PhantomPinned)`
     = note: required for `tokio::time::sleep::Sleep` to implement `Unpin`
...
     = note: required for `tokio_stream::stream_ext::timeout::Timeout<ReceiverStream<&str>>` to implement `Unpin`
note: required by a bound in `trpl::StreamExt::next`
...
 144 |     fn next(&mut self) -> Next<'_, Self>
     |        ---- required by a bound in this associated function
 145 |     where
 146 |         Self: Unpin,
     |               ^^^^^ required by this bound in `StreamExt::next`
```

- **읽는 법**: 메시지가 길지만 `note: required for ...` 줄을 따라가면 됩니다. `next`는 스트림이 `Unpin`이기를 요구하는데, `Timeout` 스트림 안의 `Sleep`(타이머 퓨처) 안의 `PhantomPinned`가 `Unpin`이 아닙니다. `PhantomPinned`는 "이 타입은 `Unpin`이 아니다"라고 표시하는 표준 라이브러리의 표식 타입입니다.
- **해결**: `let mut messages = pin!(ReceiverStream::new(rx).timeout(Duration::from_millis(200)));`처럼 `std::pin::pin!`으로 고정한 뒤 `next`를 부릅니다.

### 4.8 `StreamExt`를 가져오지 않음 — E0599

`use trpl::StreamExt;` 없이 다음 코드를 빌드합니다.

```rust
fn main() {
    trpl::block_on(async {
        let mut stream = trpl::stream_from_iter([1, 2, 3]);
        while let Some(value) = stream.next().await {
            println!("{value}");
        }
    });
}
```

```text
error[E0599]: no method named `next` found for struct `tokio_stream::iter::Iter<I>` in the current scope
 --> src/main.rs:4:40
  |
4 |         while let Some(value) = stream.next().await {
  |                                        ^^^^
  |
  = help: items from traits can only be used if the trait is in scope
help: the following traits which provide `next` are implemented but not in scope; perhaps you want to import one of them
  |
1 + use futures_util::stream::stream::StreamExt;
  |
1 + use trpl::StreamExt;
  |
help: there is a method `try_next` with a similar name
```

- **원인**: 트레이트의 메서드는 그 트레이트가 스코프에 있어야 부를 수 있습니다(ch10). `next`는 `StreamExt`의 메서드입니다.
- **해결**: `use trpl::StreamExt;`를 추가합니다. 제안 목록의 첫 번째 `futures_util::...::StreamExt`는 `trpl`이 내부에서 쓰는 다른 크레이트의 비슷한 트레이트입니다. 내 프로젝트가 `futures_util`에 직접 의존하지 않으므로 이 제안을 그대로 쓰면 ``cannot find module or crate `futures_util` ``(E0433)가 납니다. `try_next` 제안은 이 경우와 관계없습니다.

### 4.9 컴파일은 되지만 멈추거나 느린 코드

컴파일러가 잡지 못하는 실수 두 가지는 예제로 직접 확인했습니다.

- **수신 루프가 끝나지 않음**: 송신 퓨처에 `move`가 없거나 송신자를 하나라도 남겨 두면 채널이 닫히지 않아 `while let Some(..) = rx.recv().await`가 영원히 기다립니다([`04`](examples/04_async_channel.rs)의 3번, 2.8절). 프로그램이 끝나지 않으면 남아 있는 송신자가 없는지 먼저 확인하세요.
- **async 코드 안의 블로킹 호출**: `thread::sleep`, 오래 걸리는 계산, 블로킹 I/O를 await 지점 없이 실행하면 같은 태스크의 다른 퓨처와 타이머가 모두 멈춥니다([`05`](examples/05_yield_now.rs), 2.9절). async 코드 안에서는 `trpl::sleep`처럼 비동기 버전을 쓰고, 무거운 계산은 스레드에 맡깁니다.

## 5. 연습 문제

### 연습 1. 재시도하는 `timeout`

[`03_select_and_timeout.rs`](examples/03_select_and_timeout.rs)의 `timeout`을 이용해, 제한 시간 안에 끝나지 않으면 다시 시도하는 함수 `retry`를 작성하세요. `retry(make_future, max_attempts, limit)`는 `make_future(1)`, `make_future(2)`, ...로 시도마다 새 퓨처를 만들어 `limit` 안에 끝나는지 확인하고, 처음 성공한 결과를 `Some`으로, 모두 실패하면 `None`을 돌려줍니다. 실패할 때마다 한 줄을 출력합니다.

시험용으로 `n`번째 시도가 `(400 - 150 * n)`ms 걸리는 `async fn attempt(n: u64) -> String`(`"n번째 시도 성공"`을 돌려줌)을 만들고 `retry(attempt, 3, Duration::from_millis(200))`을 실행하면 다음과 같이 출력되어야 합니다. `max_attempts`를 1로 바꾸면 `None`이 나와야 합니다.

```text
1번째 시도: 200ms 안에 끝나지 않았습니다
Some("2번째 시도 성공")
```

<details>
<summary>힌트</summary>

"시도 번호를 받아 퓨처를 만드는 것"은 클로저나 함수로 받습니다(ch13). 시그니처는 `async fn retry<F, Fut>(mut make_future: F, max_attempts: u64, limit: Duration) -> Option<Fut::Output> where F: FnMut(u64) -> Fut, Fut: Future`입니다. 본문은 `for n in 1..=max_attempts`로 돌면서 `match timeout(make_future(n), limit).await`의 `Ok(output)`이면 `return Some(output)`, `Err(limit)`이면 실패 메시지를 출력합니다. 반복이 끝나면 `None`입니다. `async fn attempt`는 그 자체로 `FnMut(u64) -> 퓨처`를 만족하므로 `retry(attempt, ...)`처럼 이름만 넘길 수 있습니다. 1번째 시도는 250ms라서 실패하고, 2번째 시도는 100ms라서 성공합니다.

</details>

### 연습 2. 클로저 안의 `.await`를 두 가지로 고치고 시간 비교하기

[`compile_fail/e0728_await_in_closure.rs`](compile_fail/e0728_await_in_closure.rs)를 `examples/`에 복사(예: `examples/09_exercise_double.rs`)해 다음을 하세요.

1. `double`의 본문 앞에 `trpl::sleep(Duration::from_millis(100)).await;`를 넣어 느린 작업으로 만듭니다.
2. `double_all`을 4.2절의 두 방법(`for` 반복문, `trpl::join_all`)으로 각각 고친 함수 `double_all_sequential`, `double_all_concurrent`를 만듭니다.
3. `main`에서 `trpl::block_on`으로 두 함수에 `vec![1, 2, 3]`을 넘기고, 결과와 걸린 시간을 출력합니다.

두 함수 모두 `[2, 4, 6]`을 돌려주지만, 걸린 시간은 약 300ms와 약 100ms로 달라야 합니다.

<details>
<summary>힌트</summary>

`for` 반복문 버전은 `let mut result = Vec::new(); for n in numbers { result.push(double(n).await); } result`입니다. `.await`가 반복마다 차례로 진행되므로 100ms × 3입니다. `join_all` 버전은 `trpl::join_all(numbers.into_iter().map(double)).await` 한 줄입니다. `map(double)`은 퓨처를 만들기만 하고(게으름), `join_all`이 세 퓨처를 함께 진행시키므로 약 100ms입니다. 결과 `Vec`의 순서는 넣은 순서와 같습니다. 시간은 [`02`](examples/02_join_and_spawn_task.rs)처럼 `Instant::now()`와 `elapsed()`로 잽니다. 이렇게 같은 `async fn`에서 나온 퓨처들은 타입이 같으므로 고정(`pin!`)이 필요 없습니다.

</details>

### 연습 3. 타이머 스트림과 메시지 스트림 합치기

다음 두 스트림을 `merge`로 합쳐, 항목이 나올 때마다 시작 후 경과 시간(ms)과 함께 출력하세요. 두 스트림이 모두 끝나면 `끝`을 출력합니다.

- 틱 스트림: 100ms마다 `"틱"`을 내놓고, 5개를 낸 뒤 끝납니다.
- 메시지 스트림: `spawn_task`로 만든 태스크가 150ms 뒤에 `"메시지 1"`, 그 뒤 200ms 뒤에 `"메시지 2"`를 채널로 보내고 끝납니다.

출력의 예는 다음과 같습니다. 시각은 실행마다 몇 ms씩 다릅니다.

```text
   1ms 틱
 101ms 틱
 151ms 메시지 1
 201ms 틱
 301ms 틱
 353ms 메시지 2
 401ms 틱
끝
```

<details>
<summary>힌트</summary>

틱 스트림은 `trpl::IntervalStream::new(trpl::interval(Duration::from_millis(100)))`로 만듭니다. `interval`은 정해진 간격마다 깨어나는 타이머이고, `IntervalStream`은 그것을 "깨어날 때마다 시각(`Instant`)을 내놓는 스트림"으로 바꿉니다. 첫 틱은 기다리지 않고 바로 나옵니다. `.map(|_| String::from("틱")).take(5)`로 항목을 문자열로 바꾸고 5개에서 끊습니다. 메시지 스트림은 `ReceiverStream::new(rx)`이고 보내는 값도 `String`이어야 합니다. `merge`는 두 스트림의 항목 타입이 같아야 하기 때문입니다. `let mut merged = ticks.merge(messages);` 뒤에 `while let Some(item) = merged.next().await`로 꺼냅니다. 이 조합은 `Unpin`이라 `pin!`이 필요 없지만, 컴파일러가 Unpin 에러를 내면 4.7절처럼 `pin!`으로 감쌉니다. 합친 스트림은 두 스트림이 모두 끝나야(틱 5개, 송신자 버려짐) 끝납니다.

</details>

### 연습 4. 블로킹 계산을 스레드로 옮기기

[`05_yield_now.rs`](examples/05_yield_now.rs)의 3번에서 ticker가 300ms가 아니라 원래 의도대로 50ms 무렵에 깨어나도록 `blocker`를 고치세요. `slow("blocker", 300)` 호출 자체는 그대로 두고, 그 계산이 끝났다는 것을 blocker가 `.await`로 기다리게 만듭니다. 경과 시간은 반올림하지 말고 `start.elapsed().as_millis()`로 출력해 확인하세요.

<details>
<summary>힌트</summary>

[`08`](examples/08_tasks_and_threads.rs)의 3번과 같은 방법입니다. blocker 안에서 `let (tx, mut rx) = trpl::channel();`을 만들고, `thread::spawn(move || { slow("blocker", 300); tx.send(()).unwrap(); });`로 계산을 스레드에 맡긴 뒤 `rx.recv().await;`로 기다립니다. `recv().await`는 스레드를 블로킹하지 않고 차례를 넘기므로, 그동안 ticker의 50ms 타이머가 제때 깨어납니다. ticker가 50ms 남짓에, blocker가 300ms 남짓에 끝나야 합니다. tokio를 직접 쓴다면 이런 용도로 `tokio::task::spawn_blocking`이 있습니다(2.12절).

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [17. Fundamentals of Asynchronous Programming: Async, Await, Futures, and Streams](https://doc.rust-lang.org/book/ch17-00-async-await.html): 동시성과 병렬성(2.1절)
  - [17.1 Futures and the Async Syntax](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html): 퓨처, `async fn`의 변환, 런타임과 `block_on`, `select`(2.2~2.4절, 2.7절). 웹 페이지 제목 예제는 인터넷이 필요합니다.
  - [17.2 Applying Concurrency with Async](https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html): `spawn_task`, `join`, 비동기 채널(2.5·2.6·2.8절)
  - [17.3 Working With Any Number of Futures](https://doc.rust-lang.org/book/ch17-03-more-futures.html): `yield_now`, 직접 만드는 `timeout`(2.7·2.9절). 제목과 달리 현재 판에서 `join_all`은 17.5절에 있습니다.
  - [17.4 Streams: Futures in Sequence](https://doc.rust-lang.org/book/ch17-04-streams.html): `stream_from_iter`와 `StreamExt`(2.10절)
  - [17.5 A Closer Look at the Traits for Async](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html): `Future`, `Pin`, `Unpin`, `Stream`, `join_all`(2.11절). 자기 참조 퓨처를 옮길 때의 그림이 있습니다.
  - [17.6 Futures, Tasks, and Threads](https://doc.rust-lang.org/book/ch17-06-futures-tasks-threads.html): 선택 기준과 함께 쓰기(2.12절)
- [`trpl` 문서 (docs.rs)](https://docs.rs/trpl/latest/trpl/): 이 챕터에서 쓴 함수와 타입의 목록. 각 항목에서 원래 크레이트(tokio, futures, tokio-stream)의 문서로 이어집니다.
- [Tokio 튜토리얼](https://tokio.rs/tokio/tutorial): 실무에서 tokio를 직접 쓰는 방법. 첫 페이지에 tokio가 맞지 않는 경우(CPU 바운드 계산만 하는 프로그램, 파일을 많이 읽는 일, 요청 하나만 보내는 일)가 정리되어 있습니다.
  - [Hello Tokio](https://tokio.rs/tokio/tutorial/hello-tokio): `#[tokio::main]`이 펼쳐지는 모양(2.4절)
  - [Spawning](https://tokio.rs/tokio/tutorial/spawning), [Channels](https://tokio.rs/tokio/tutorial/channels), [Select](https://tokio.rs/tokio/tutorial/select), [Streams](https://tokio.rs/tokio/tutorial/streams): 이 챕터의 2.6·2.8·2.7·2.10절을 tokio API로 다시 볼 수 있습니다.
- 표준 라이브러리 문서
  - [`std::future::Future`](https://doc.rust-lang.org/std/future/trait.Future.html), [`std::pin` 모듈](https://doc.rust-lang.org/std/pin/index.html)(자기 참조와 `Pin`이 필요한 이유를 자세히 설명), [`pin!` 매크로](https://doc.rust-lang.org/std/pin/macro.pin.html), [`Unpin`](https://doc.rust-lang.org/std/marker/trait.Unpin.html)
- Rustlings
  - [`24_async`](https://github.com/rust-lang/rustlings/tree/main/exercises/24_async): `#[tokio::main]`과 `tokio::spawn`으로 세 반의 평균 점수를 동시에 계산하는 문제입니다. `async`가 빠진 함수(4.1절)와 `JoinHandle`을 `.await`하지 않은 코드를 고칩니다. 2026년 11월 확인 기준으로 저장소 main 브랜치에만 있고 최신 릴리스(v6.5.0)에는 없습니다.
- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/): Rust 비동기 프로그래밍을 깊이 다루는 공식 자료입니다. 첫 페이지에 재작성 중이라 빠진 부분이 있다고 적혀 있습니다.
- 이후 챕터: 트레이트 객체 `dyn`은 `ch17-trait-objects-oop`, 퓨처와 실행기를 직접 구현하는 것은 Part 1 `ch13-async-primitives`에서 다룹니다.
