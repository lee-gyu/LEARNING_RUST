# ch15. 겁 없는 동시성: 스레드, 메시지 전달, 공유 상태

여러 작업을 동시에 실행하는 프로그램은 빠르지만, 실행할 때마다 결과가 달라지는 버그가 숨어들기 쉽습니다. Rust는 소유권과 타입 검사로 이런 동시성 버그의 상당수를 **컴파일할 때** 잡아 줍니다. Rust 문서는 이 성질을 겁 없는 동시성(fearless concurrency)이라고 부릅니다. 이 챕터에서는 스레드를 만들고, 채널로 값을 주고받고, `Mutex`와 `Arc`로 값을 함께 쓰는 방법을 배우면서, 컴파일러가 거부하는 코드마다 **어떤 버그를 막았는지** 확인합니다.

- 패키지: `p0-ch15-concurrency`
- 기준 문서: [The Book 16장 Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- 확인한 툴체인: rustc 1.100.0, Edition 2024. 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).
- 챕터 번호: 이 저장소는 The Book 14장을 Part 3으로 옮겼기 때문에 Book 16장이 `ch15`입니다. 이 문서에서 "N장"은 The Book의 장을, `chN`은 이 저장소의 챕터를 가리킵니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `thread::spawn`으로 스레드를 만들고 `JoinHandle`의 `join`으로 끝나기를 기다리며, 스레드가 돌려준 값을 받을 수 있다. `join`을 부르는 위치에 따라 실행 순서가 달라지는 이유와, `main`이 끝나면 다른 스레드도 끝난다는 것을 설명할 수 있다.
2. 스레드에 넘기는 클로저에 `move`가 필요한 이유를 설명하고, 값을 옮기거나 복제해서 넘기는 코드를 작성할 수 있다.
3. `mpsc::channel`로 스레드 사이에 값을 보내는 코드를 작성하고, `send`가 소유권을 옮긴다는 점, `recv`와 `try_recv`의 차이, 모든 송신자가 drop되어야 수신 반복이 끝나는 이유를 설명할 수 있다.
4. `Arc<Mutex<T>>`로 여러 스레드가 함께 고치는 값을 만들고, `MutexGuard`가 drop될 때 잠금이 풀린다는 것과 `Rc<RefCell<T>>`와의 대응 관계를 설명할 수 있다.
5. `Send`와 `Sync`의 뜻과 `Rc`, `RefCell`이 이를 구현하지 않는 이유를 설명하고, 동시성 코드의 컴파일 에러(E0373, E0382, E0277)가 어떤 버그를 막았는지 읽어 낼 수 있다.

## 2. 핵심 개념

### 2.1 겁 없는 동시성

동시성(concurrency)은 프로그램의 여러 부분이 서로 독립적으로 실행되는 것이고, 병렬성(parallelism)은 여러 부분이 실제로 같은 순간에 실행되는 것입니다. The Book 16장은 둘을 굳이 나누지 않고 "동시성"이라고 부르며, 이 챕터도 그렇게 씁니다. 둘의 차이는 비동기를 다루는 [ch16](../ch16-async/README.md)에서 중요해집니다.

동시성 프로그램이 어려운 까닭은 **실행 순서를 보장할 수 없기** 때문입니다. 두 스레드 중 어느 쪽이 먼저 실행될지는 운영체제가 정하고, 그 결정은 실행할 때마다 바뀔 수 있습니다. 그래서 동시성 버그는 테스트에서는 멀쩡하다가 특정한 순서로 실행된 날에만 나타나고, 다시 재현하기도 어렵습니다.

가장 대표적인 동시성 버그가 [ch4](../ch4-ownership/README.md) 2.9절에서 배운 **데이터 경쟁(data race)** 입니다. 다음 세 조건이 동시에 맞으면 생깁니다.

- 둘 이상의 스레드가 같은 데이터에 동시에 접근한다.
- 그중 적어도 하나가 데이터를 쓴다.
- 접근을 동기화하는 장치가 없다.

Rust 팀은 처음에 메모리 안전과 동시성 문제를 서로 다른 방법으로 풀어야 하는 과제라고 생각했지만, 소유권과 타입 시스템이 두 문제를 함께 다루는 도구라는 것을 알게 되었습니다. 이 챕터에서 만나는 컴파일 에러는 새로운 규칙이 아니라, 지금까지 배운 소유권·대여·트레이트 규칙을 스레드에 적용한 결과입니다.

| 컴파일러가 거부하는 코드 | 에러 | 막은 버그 | 절 |
| --- | --- | --- | --- |
| 지역 변수를 빌리는 클로저로 스레드 만들기 | E0373 | 스레드가 이미 해제된 값을 읽음(댕글링 참조) | 2.4, 4.1 |
| `send`한 값을 다시 사용 | E0382 | 두 스레드가 동기화 없이 같은 값에 접근(데이터 경쟁) | 2.5, 4.2 |
| `Mutex` 하나를 여러 스레드로 이동 | E0382 | 같은 값을 여러 스레드가 각자 소유하고 각자 해제(이중 해제) | 2.8, 4.3 |
| `Rc`를 다른 스레드로 보내기 | E0277 | 참조 카운트가 틀어져 값이 너무 일찍 해제되거나 누수 | 2.8, 4.4 |
| `RefCell`을 여러 스레드가 함께 쓰기 | E0277 | 두 스레드가 동시에 가변 대여를 얻음(데이터 경쟁) | 2.9, 4.5 |

다만 컴파일러가 모든 동시성 버그를 잡아 주지는 않습니다. 교착 상태와 논리적인 경쟁 조건은 2.10절에서 따로 봅니다.

### 2.2 스레드 만들기: `thread::spawn`

스레드(thread)는 한 프로그램 안에서 독립적으로 실행되는 흐름입니다. 프로그램을 시작하면 `main` 함수를 실행하는 **메인 스레드**가 하나 있고, `thread::spawn`에 클로저를 넘기면 그 클로저를 실행하는 새 스레드가 생깁니다. Rust 표준 라이브러리의 스레드 하나는 운영체제 스레드 하나에 그대로 대응합니다(1:1 모델).

```rust
use std::thread;
use std::time::Duration;

let handle = thread::spawn(|| {
    for i in 1..=4 {
        println!("[새 스레드] {i}");
        thread::sleep(Duration::from_millis(1));
    }
});

for i in 1..=3 {
    println!("[main] {i}");
    thread::sleep(Duration::from_millis(1));
}
```

`spawn`은 새 스레드가 일을 끝낼 때까지 기다리지 않고 바로 돌아옵니다. 그래서 두 반복문이 동시에 진행되고 출력이 섞입니다. `thread::sleep`은 지금 스레드를 잠깐 멈추는 함수로, 그동안 운영체제가 다른 스레드를 실행할 기회가 생깁니다. 그래도 번갈아 실행된다는 보장은 없습니다. 섞이는 순서는 운영체제의 스케줄링에 달려 있어서 실행할 때마다 달라질 수 있습니다.

**메인 스레드가 끝나면 프로그램 전체가 끝납니다.** 다른 스레드가 아직 일하는 중이어도 함께 끝나 버립니다. [`01_spawn_and_join.rs`](examples/01_spawn_and_join.rs)의 4번 구역에서, 다섯 줄을 출력해야 하는 스레드가 중간에 끊기는 것을 볼 수 있습니다.

### 2.3 끝나기를 기다리기: `JoinHandle`과 `join`

`thread::spawn`은 `JoinHandle<T>`를 돌려줍니다. 이 값의 `join` 메서드를 부르면 **그 스레드가 끝날 때까지 지금 스레드가 멈춰서 기다립니다.** 이렇게 스레드가 다른 일을 하지 못하고 멈춰 있는 것을 블록(block)된다고 합니다.

`join`을 어디에 두느냐에 따라 프로그램의 모양이 달라집니다.

- 메인 스레드가 자기 일을 다 한 **뒤에** `join`하면, 두 스레드가 동시에 일하고 메인 스레드는 마지막에 새 스레드를 기다립니다.
- `spawn` **바로 다음에** `join`하면, 메인 스레드는 새 스레드가 끝날 때까지 아무것도 하지 않습니다. 스레드를 만들었어도 동시에 실행되는 구간이 없습니다.

`JoinHandle<T>`의 `T`는 클로저가 돌려주는 값의 타입이고, `join`은 `Result<T, _>`를 돌려줍니다. 그래서 스레드가 계산한 결과를 `join`으로 받을 수 있습니다.

```rust
let handle = thread::spawn(|| 1 + 2); // JoinHandle<i32>
let three = handle.join().unwrap();   // 3
```

`join`이 `Result`를 돌려주는 까닭은 스레드가 패닉할 수 있기 때문입니다. 새 스레드에서 패닉이 나면 **그 스레드만** 멈추고, 메인 스레드는 `join`이 돌려준 `Err`로 그 사실을 알게 됩니다. 지금까지 쓴 `join().unwrap()`은 "새 스레드가 패닉했다면 메인 스레드도 패닉하겠다"는 뜻입니다. [`02_move_and_return.rs`](examples/02_move_and_return.rs)의 5번 구역이 이 경우를 보여 줍니다.

`JoinHandle`을 변수에 담지 않고 버리면 스레드는 계속 실행되지만 더 이상 기다릴 방법이 없습니다. 이것을 스레드를 떼어 낸다(detach)고 합니다.

### 2.4 스레드에 값 넘기기: `move` 클로저

새 스레드에서 메인 스레드의 값을 쓰려면 클로저가 그 값을 캡처해야 합니다. ch13에서 배웠듯이 클로저는 필요한 만큼만 캡처하므로, 읽기만 하는 값은 **빌립니다**. 그런데 스레드에서는 이것이 문제가 됩니다.

```rust
let v = vec![1, 2, 3];
let handle = thread::spawn(|| {
    println!("{v:?}"); // v를 빌리려 함 → E0373
});
```

컴파일러는 새 스레드가 언제까지 실행될지 알 수 없습니다. 만약 메인 스레드가 `v`를 먼저 버린다면(drop) 새 스레드는 해제된 메모리를 읽게 됩니다. 그래서 `thread::spawn`은 **지역 변수를 빌린 클로저를 받지 않습니다.** 이 조건은 `spawn`의 시그니처에 `'static` 바운드로 적혀 있습니다.

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

`F: 'static`은 "이 클로저 안에 언제 사라질지 모르는 지역 변수의 참조가 들어 있지 않다"는 뜻입니다. `String`, `Vec<T>`처럼 데이터를 직접 소유하는 값은 이 조건을 만족합니다. (`Send`는 2.9절에서 봅니다.)

해결책은 클로저 앞에 `move`를 붙여 값의 **소유권을 새 스레드로 옮기는** 것입니다.

```rust
let v = vec![1, 2, 3];
let handle = thread::spawn(move || {
    println!("{v:?}"); // 이제 v의 소유자는 이 스레드입니다.
});
```

`move`는 "빌리지 말고 가져가라"는 지시일 뿐, 소유권 규칙을 비켜 가게 해 주지는 않습니다. `v`를 스레드로 옮긴 뒤 메인 스레드에서 `drop(v)`처럼 다시 쓰면 이번에는 E0382가 납니다. 이때 컴파일러의 `help:`는 복제본을 만들어 넘기라고 제안합니다.

```
help: consider cloning the value before moving it into the closure
  |
4 ~     let value = v.clone();
5 ~     let handle = thread::spawn(move || {
6 ~         println!("{value:?}");
```

정리하면 스레드에 값을 넘기는 방법은 다음과 같습니다.

| 상황 | 방법 | 예제 |
| --- | --- | --- |
| 메인 스레드가 더는 쓰지 않음 | `move`로 옮김 | 02의 1번 구역 |
| 다 쓴 뒤 돌려받아야 함 | `move`로 옮기고 클로저의 반환값으로 돌려받음 | 02의 2번 구역 |
| 두 스레드가 각자 따로 써야 함 | `clone()`한 사본을 `move`로 옮김 | 02의 3번 구역 |
| 여러 스레드가 같은 값을 함께 써야 함 | `Arc`로 감싸고 복제본을 하나씩 옮김 | 2.8절, 06 |

`i32` 같은 `Copy` 타입은 `move`해도 복사본이 들어가므로 원래 변수를 계속 쓸 수 있습니다. 스레드가 반드시 어떤 스코프 안에서 끝나도록 보장하면 지역 변수를 빌려 쓰는 것도 가능한데, 이것이 스코프 스레드(`thread::scope`)이며 [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md)에서 다룹니다.

### 2.5 메시지 전달: 채널

스레드끼리 데이터를 주고받는 첫 번째 방법은 **메시지 전달(message passing)** 입니다. Go 언어 문서의 표어가 이 생각을 잘 요약합니다. "메모리를 공유해서 통신하지 말고, 통신해서 메모리를 공유하라."

표준 라이브러리는 이를 위해 채널(channel)을 제공합니다. 채널은 값을 넣는 송신자(transmitter)와 값을 꺼내는 수신자(receiver), 두 끝으로 이뤄집니다.

```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    tx.send(String::from("안녕하세요")).unwrap();
});

let received = rx.recv().unwrap(); // "안녕하세요"
```

- `mpsc`는 multiple producer, single consumer의 약자입니다. 송신자는 여럿일 수 있지만 수신자는 하나뿐입니다(`Receiver`는 `clone`할 수 없습니다).
- `channel()`은 `(송신자, 수신자)` 튜플을 돌려줍니다. 보낼 값의 타입은 `send`에 넘기는 값을 보고 추론됩니다.
- 보내는 스레드가 송신자를 가지고 있어야 하므로 `move`로 `tx`를 넘깁니다.

**`send`는 값의 소유권을 옮깁니다.** 보낸 뒤에는 그 값을 쓸 수 없고(E0382, 4.2절) 받은 쪽이 새 소유자가 됩니다. 이것이 채널이 안전한 이유입니다. 보낸 쪽이 계속 그 값을 읽을 수 있다면, 받은 스레드가 값을 바꾸거나 버리는 동안 보낸 스레드가 읽는 데이터 경쟁이 생길 수 있습니다. [`03_channel_basics.rs`](examples/03_channel_basics.rs)의 2번 구역은 받은 `String`이 보낸 `String`과 같은 힙 데이터를 가리킨다는 것(`true`)을 확인합니다. 채널은 글자를 복사하지 않고 주인만 바꿉니다.

채널의 한쪽 끝이 모두 drop되면 채널이 닫혔다고 합니다. 세 메서드는 채널의 상태에 따라 다음 값을 돌려줍니다.

| 메서드 | 기다리는가 | 돌려주는 값 |
| --- | --- | --- |
| `tx.send(v)` | 기다리지 않음 | `Ok(())`. 수신자가 사라졌으면 `Err(SendError(v))`에 보내려던 값을 담아 돌려줌 |
| `rx.recv()` | 메시지가 올 때까지 기다림 | `Ok(메시지)`. 송신자가 모두 사라지고 남은 메시지도 없으면 `Err(RecvError)` |
| `rx.try_recv()` | 기다리지 않음 | `Ok(메시지)`. 지금 메시지가 없으면 `Err(Empty)`, 송신자가 모두 사라지고 남은 메시지도 없으면 `Err(Disconnected)` |

`try_recv`는 기다리는 동안 다른 일을 해야 할 때 씁니다. 반복문에서 `try_recv`로 확인하고, `Empty`이면 다른 일을 조금 한 뒤 다시 확인하는 식입니다(03의 7번 구역).

### 2.6 수신자를 반복자로 쓰기와 여러 생산자

수신자를 `for` 문에 넣으면 메시지가 올 때마다 하나씩 꺼냅니다. 메시지가 없으면 기다리고, **채널이 닫히면(송신자가 모두 drop되면) 반복이 끝납니다.**

```rust
for received in rx {
    println!("받음: {received}");
}
```

송신자를 `clone`하면 같은 채널로 이어진 송신자가 하나 더 생깁니다. 스레드마다 복제본을 하나씩 넘기면 여러 생산자가 한 수신자에게 보낼 수 있습니다.

```rust
let (tx, rx) = mpsc::channel();
for producer in 1..=3 {
    let tx = tx.clone(); // 스레드마다 자기 송신자를 가집니다.
    thread::spawn(move || {
        for n in 1..=3 {
            tx.send((producer, n)).unwrap();
        }
    });
}
drop(tx); // 원래 송신자는 main이 직접 버립니다.

for (producer, n) in rx {
    println!("생산자 {producer}의 {n}번째 메시지");
}
```

여기서 `drop(tx)`가 중요합니다. `drop`은 ch14에서 배운 `std::mem::drop`으로, 값을 스코프가 끝나기 전에 버립니다. 원래의 `tx`를 아무 스레드에도 넘기지 않고 `main`이 계속 들고 있으면, 생산자 스레드가 모두 끝나도 송신자가 하나 남아 있으므로 채널이 닫히지 않습니다. 그러면 `for` 문은 마지막 메시지를 받은 뒤에도 다음 메시지를 **영원히** 기다립니다. 컴파일 에러도 패닉도 없이 프로그램이 멈춰 있기만 하므로 원인을 찾기 어렵습니다. [`04_multiple_producers.rs`](examples/04_multiple_producers.rs)의 3번 구역은 영원히 기다리는 대신 최대 대기 시간을 정하는 `recv_timeout`으로 이 상황을 보여 줍니다. 송신자가 남아 있으면 `Err(Timeout)`(아직 안 옴)이, 마지막 송신자를 버린 뒤에는 `Err(Disconnected)`(닫힘)가 나옵니다.

여러 생산자의 메시지가 섞이는 순서는 실행마다 다르지만, **같은 송신자가 보낸 메시지끼리는 보낸 순서대로 도착합니다.**

### 2.7 공유 상태: `Mutex<T>`

두 번째 방법은 여러 스레드가 **같은 데이터에 함께 접근하는** 것입니다. 채널에서는 값을 보내면 보낸 쪽이 더는 쓸 수 없으니 소유자가 하나인 방식이고, 공유 상태는 소유자가 여럿인 방식입니다.

뮤텍스(mutex)는 mutual exclusion(상호 배제)의 줄임말로, 한 번에 한 스레드만 데이터에 접근하게 하는 장치입니다. 데이터에 접근하려는 스레드는 먼저 **잠금(lock)** 을 얻어야 하고, 다 쓰면 잠금을 풀어 다른 스레드가 쓸 수 있게 해야 합니다. 마이크가 하나뿐인 토론회와 같습니다. 마이크를 받아야 말할 수 있고, 다 말하면 넘겨줘야 합니다.

다른 언어에서 뮤텍스가 다루기 어렵다고 알려진 까닭은 "쓰기 전에 잠그기"와 "다 쓰면 풀기"라는 두 규칙을 사람이 기억해야 하기 때문입니다. Rust의 `Mutex<T>`는 두 규칙을 타입으로 강제합니다.

```rust
use std::sync::Mutex;

let m = Mutex::new(5);
{
    let mut num = m.lock().unwrap(); // num의 타입은 MutexGuard<i32>
    *num = 6;
} // num이 drop되며 잠금이 풀립니다.
println!("{m:?}"); // Mutex { data: 6, poisoned: false, .. }
```

- **잠그지 않고는 쓸 수 없습니다.** `m`의 타입은 `i32`가 아니라 `Mutex<i32>`이므로, 안의 `i32`에 닿는 길은 `lock()`뿐입니다.
- **풀기를 잊을 수 없습니다.** `lock()`이 돌려주는 `MutexGuard`는 ch14에서 배운 스마트 포인터입니다. `Deref`/`DerefMut`을 구현하므로 `*num`으로 안의 값을 읽고 쓰거나 `num.push(4)`처럼 안의 값의 메서드를 부를 수 있고, `Drop`을 구현하므로 스코프를 벗어나면 잠금이 저절로 풀립니다. 일찍 풀고 싶으면 `drop(num)`을 부릅니다.
- `m`은 `mut`가 아닌데도 안의 값을 바꿀 수 있습니다. `Mutex<T>`는 `RefCell<T>`처럼 **내부 가변성**을 제공합니다.

`lock()`은 다른 스레드가 잠금을 쥐고 있으면 풀릴 때까지 기다립니다. 반환 타입이 `Result`인 까닭은 포이즈닝(poisoning) 때문입니다. 잠금을 쥔 스레드가 패닉하면 데이터가 고치다 만 상태로 남았을 수 있으므로, 그 뒤의 `lock()`은 `Err`로 이 사실을 알립니다. 이 챕터에서는 `unwrap()`으로 패닉을 이어 갑니다.

`MutexGuard`가 언제 drop되는지는 변수에 담았는지에 따라 다릅니다.

```rust
// (가) 변수에 담으면 그 변수의 스코프가 끝날 때까지 잠겨 있습니다.
{
    let mut numbers = m.lock().unwrap();
    numbers.push(4);
} // 여기서 풀립니다.

// (나) 변수에 담지 않으면 그 문장이 끝날 때 바로 풀립니다.
m.lock().unwrap().push(5);
```

[`05_mutex_basics.rs`](examples/05_mutex_basics.rs)는 `try_lock`(기다리지 않는 `lock`)과 `Mutex`의 디버그 형식으로 지금 잠겨 있는지를 직접 확인합니다. `MutexGuard`가 살아 있는 동안 디버그 형식은 안의 값 대신 `"<locked>"`를 보여 줍니다.

**같은 스레드에서 이미 쥔 잠금을 다시 `lock()`하면 안 됩니다.** 표준 라이브러리 문서는 이때의 동작을 정해 두지 않았고, 두 번째 `lock()`이 돌아오지 않는다(패닉하거나 영원히 기다릴 수 있다)고만 적고 있습니다. `RefCell`의 `borrow_mut`를 두 번 부르면 패닉으로 바로 알려 주는 것과 다릅니다(05의 5번 구역).

### 2.8 여러 스레드가 함께 소유하기: `Arc<T>`

`Mutex`를 여러 스레드에 넘기려고 하면 소유권 문제에 부딪힙니다. `Mutex`도 평범한 값이라서 소유자는 하나뿐입니다.

```rust
let counter = Mutex::new(0);
for _ in 0..10 {
    thread::spawn(move || {
        *counter.lock().unwrap() += 1; // 첫 바퀴에서 counter가 이동 → E0382
    });
}
```

ch14에서는 값 하나에 소유자를 여럿 두기 위해 `Rc<T>`를 썼습니다. 그런데 `Rc<Mutex<i32>>`로 바꾸면 이번에는 E0277(`` `Rc<...>` cannot be sent between threads safely``)이 납니다. `Rc`는 `clone`과 drop 때마다 참조 카운트를 평범한 정수 연산으로 고칩니다. 두 스레드가 동시에 카운트를 고치면 한쪽의 변경이 사라질 수 있고(2.10절의 "읽고-더하고-쓰기" 문제와 같습니다), 카운트가 틀리면 값이 아직 쓰이는 중에 해제되거나 영영 해제되지 않습니다.

`Arc<T>`(atomically reference counted)는 `Rc<T>`와 사용법이 같지만 참조 카운트를 **원자적(atomic) 연산**으로 고칩니다. 원자적 연산은 다른 스레드가 중간에 끼어들 수 없는 한 덩어리로 실행되는 연산입니다. 그렇다면 왜 항상 `Arc`를 쓰지 않을까요? 원자적 연산은 일반 연산보다 비용이 크기 때문입니다. 한 스레드에서만 쓰는 값이라면 `Rc`가 더 빠릅니다. Rust는 필요할 때만 비용을 치르게 하고, 잘못 고르면 컴파일러가 알려 줍니다.

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let mut handles = Vec::new();
for _ in 0..10 {
    let counter = Arc::clone(&counter); // 스레드마다 Arc 복제본 하나
    handles.push(thread::spawn(move || {
        for _ in 0..1000 {
            *counter.lock().unwrap() += 1;
        }
    }));
}
for handle in handles {
    handle.join().unwrap();
}
println!("{}", *counter.lock().unwrap()); // 항상 10000
```

`Arc::clone`이 복제하는 것은 포인터와 참조 카운트뿐이고, `Mutex`와 그 안의 값은 하나입니다. 스레드가 끝나면 그 스레드가 가진 복제본이 drop되어 카운트가 줄어듭니다. 몇 번을 실행해도 결과는 10000입니다([`06_shared_counter.rs`](examples/06_shared_counter.rs)).

이 구조는 ch14의 `Rc<RefCell<T>>`와 똑같고, 타입 두 개만 바뀝니다.

| 역할 | 스레드 하나 | 여러 스레드 |
| --- | --- | --- |
| 소유자를 여럿 두기 | `Rc<T>` | `Arc<T>` |
| 불변 바인딩 안의 값 바꾸기(내부 가변성) | `RefCell<T>` | `Mutex<T>` |
| 이미 쓰는 중일 때 또 접근하면 | `borrow_mut`가 패닉 | `lock`이 풀릴 때까지 기다림 |
| 함께 쓰는 형태 | `Rc<RefCell<T>>` | `Arc<Mutex<T>>` |
| 컴파일러가 막아 주지 못하는 위험 | 참조 순환으로 인한 메모리 누수 | 교착 상태 |

### 2.9 `Send`와 `Sync`

`Rc`를 스레드로 보내려 했을 때 컴파일러가 댄 이유는 ``the trait `Send` is not implemented``였습니다. `Send`와 `Sync`는 `std::marker`에 있는 **마커 트레이트(marker trait)** 로, 구현할 메서드가 하나도 없고 "이 타입은 이런 일을 해도 안전하다"는 표시만 합니다.

- **`Send`**: 이 타입의 값은 **소유권을 다른 스레드로 넘겨도** 안전합니다.
- **`Sync`**: 이 타입의 값을 **여러 스레드가 참조(`&T`)로 동시에 써도** 안전합니다. 정확히는 `&T`가 `Send`이면 `T`는 `Sync`입니다.

`thread::spawn`의 시그니처(2.4절)에 `F: Send`가 있으므로, 클로저가 캡처한 값이 하나라도 `Send`가 아니면 스레드를 만들 수 없습니다. 또 `Arc<T>`는 `T`가 `Send + Sync`일 때만 `Send`입니다. `Arc`로 감싼 값은 여러 스레드가 `&T`로 함께 보기 때문입니다.

대부분의 타입은 이 두 트레이트를 **자동으로** 구현합니다. 구성 요소가 모두 `Send`인 타입은 `Send`이고, 모두 `Sync`인 타입은 `Sync`입니다. 직접 만든 구조체도 필드가 모두 `Send`이면 아무것도 적지 않아도 `Send`입니다([`07_send_sync.rs`](examples/07_send_sync.rs)의 1번 구역).

| 타입 | `Send` | `Sync` | 이유 |
| --- | --- | --- | --- |
| `i32`, `bool`, `String` 등 | O | O | 평범한 값. 함께 쓸 때는 `&T`로 읽기만 할 수 있음 |
| `Vec<T>`, 직접 만든 구조체 | 구성 요소가 모두 `Send`일 때 | 구성 요소가 모두 `Sync`일 때 | 자동 구현 |
| `Rc<T>` | X | X | 참조 카운트를 원자적이지 않은 연산으로 고침 |
| `Arc<T>` | `T: Send + Sync`일 때 | `T: Send + Sync`일 때 | 참조 카운트를 원자적 연산으로 고침 |
| `RefCell<T>`, `Cell<T>` | `T: Send`일 때 | X | 대여 상태를 원자적이지 않게 셈 |
| `Mutex<T>` | `T: Send`일 때 | `T: Send`일 때 | 한 번에 한 스레드만 들여보냄 |

`RefCell`이 `Send`이면서 `Sync`는 아니라는 점이 두 트레이트의 차이를 잘 보여 줍니다. `RefCell`을 통째로 다른 스레드에 **넘기는** 것은 괜찮습니다. 넘긴 뒤에는 그 스레드 하나만 쓰기 때문입니다(07의 2번 구역). 하지만 여러 스레드가 **함께** 쓰면, 두 스레드가 동시에 "아무도 안 빌렸다"고 확인한 뒤 각자 가변 참조를 얻을 수 있습니다. 바로 데이터 경쟁입니다. 반대로 `Mutex<T>`는 한 번에 한 스레드만 안의 값에 들여보내므로, `T`가 `Sync`가 아니어도 `Sync`입니다.

`Send`와 `Sync`를 직접 구현하는 것은 `unsafe` 코드입니다. 컴파일러가 확인해 주던 안전성을 프로그래머가 대신 보증해야 하기 때문입니다. `unsafe`는 [ch19](../ch19-advanced-features/README.md)에서, `Send`/`Sync`를 직접 구현하는 방법은 Part 10 `ch9-concurrency`에서 다룹니다.

### 2.10 컴파일러가 막아 주지 못하는 것

Rust가 컴파일 시점에 막아 주는 것은 **데이터 경쟁**입니다. 동시성 버그가 모두 데이터 경쟁인 것은 아닙니다.

**경쟁 조건(race condition)** 은 실행 순서에 따라 결과가 달라지는 논리 오류입니다. 06의 3번 구역은 읽을 때 한 번 잠그고, 쓸 때 다시 잠급니다.

```rust
let current = *counter.lock().unwrap(); // 잠그고 읽은 뒤 이 문장이 끝나면 풀림
*counter.lock().unwrap() = current + 1; // 다시 잠그고 씀
```

모든 접근이 `Mutex` 안에서 일어나므로 데이터 경쟁은 없고, 컴파일도 됩니다. 하지만 두 문장 사이에 다른 스레드가 끼어들면, 두 스레드가 같은 값(예: 5)을 읽고 같은 값(6)을 써서 더하기 하나가 사라집니다. 이 환경에서 10개 스레드가 1000번씩 더했을 때 결과는 실행마다 달랐고, 대략 2000에서 4500 사이였습니다. 잠금은 "읽고-더하고-쓰기"처럼 **한 덩어리로 실행되어야 하는 일 전체**를 감싸야 합니다.

**교착 상태(deadlock)** 는 두 스레드가 서로 상대가 쥔 잠금을 기다리며 영원히 멈추는 상태입니다.

```rust
// 스레드 1                       // 스레드 2
let _x = a.lock().unwrap();       let _y = b.lock().unwrap();
let _y = b.lock().unwrap();       let _x = a.lock().unwrap();
```

스레드 1이 `a`를, 스레드 2가 `b`를 잠근 직후라면 둘 다 다음 줄에서 영원히 기다립니다. 한쪽이 두 잠금을 다 얻은 뒤에 다른 쪽이 시작되면 멀쩡히 끝나기도 하므로, 실행할 때마다 멈출 수도 있고 아닐 수도 있습니다. 그래서 이 챕터는 교착 상태를 실행 예제로 만들지 않았습니다. 흔한 예방법은 여러 잠금을 쥘 때 **모든 스레드가 같은 순서로** 잠그는 것입니다. 같은 스레드에서 같은 `Mutex`를 두 번 잠그는 것(2.7절)과, 송신자를 버리지 않아 수신 반복이 끝나지 않는 것(2.6절)도 같은 종류의 "영원히 기다리는" 버그입니다.

ch14에서 `Rc`의 참조 순환이 컴파일러가 막아 주지 못하는 메모리 누수였던 것처럼, 교착 상태와 경쟁 조건은 설계로 피해야 하는 문제입니다.

### 2.11 이 챕터에서 다루지 않는 것

| 주제 | 한 줄 소개 | 다루는 곳 |
| --- | --- | --- |
| 스코프 스레드 `thread::scope` | 스코프가 끝나기 전에 스레드가 반드시 끝나도록 보장해, 지역 변수를 `move` 없이 빌려 쓰게 함 | [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) |
| `RwLock<T>` | 읽기는 여러 스레드가 동시에, 쓰기는 한 스레드만 하게 하는 잠금 | [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) |
| `Condvar` | 어떤 조건이 만족될 때까지 스레드를 재워 두었다가 깨우는 장치 | [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) |
| 원자 타입(`AtomicUsize` 등) | 정수 카운터 같은 단순한 값을 `Mutex` 없이 여러 스레드에서 안전하게 고침 | [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) |
| 비동기(`async`/`await`) | 스레드 대신 태스크로 동시성을 얻는 방법과 스레드와의 비교 | [ch16](../ch16-async/README.md) |
| 스레드 풀 | 스레드를 미리 만들어 두고 일을 나눠 주는 구조 | [ch20](../ch20-web-server-project/README.md) |
| `Send`/`Sync` 직접 구현, 메모리 순서 | `unsafe`로 동시성 타입 만들기 | Part 10 `ch9-concurrency` |

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다. 여러 스레드가 함께 출력하는 줄의 순서는 실행할 때마다 달라질 수 있습니다. 아래 출력은 이 환경에서 실제로 실행한 결과이고, 순서나 값이 달라질 수 있는 부분은 출력 아래에 따로 적었습니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_spawn_and_join.rs`](examples/01_spawn_and_join.rs) | 스레드 생성, `join` 위치에 따른 실행 순서, `main`이 끝나면 끊기는 스레드 | 1 |
| [`02_move_and_return.rs`](examples/02_move_and_return.rs) | `move`로 값 넘기기, `JoinHandle<T>`로 결과 받기, 복제해서 넘기기, 패닉한 스레드의 `join` | 1, 2 |
| [`03_channel_basics.rs`](examples/03_channel_basics.rs) | `send`의 소유권 이동, `recv`, 채널이 닫혔을 때, `try_recv` | 3 |
| [`04_multiple_producers.rs`](examples/04_multiple_producers.rs) | 수신자를 반복자로 쓰기, 송신자 복제, 송신자가 남아 있으면 닫히지 않는 채널 | 3 |
| [`05_mutex_basics.rs`](examples/05_mutex_basics.rs) | `lock`과 `MutexGuard`, drop 시점과 잠금 해제, `RefCell`과의 비교 | 4 |
| [`06_shared_counter.rs`](examples/06_shared_counter.rs) | `Rc<RefCell<T>>`와 `Arc<Mutex<T>>`의 대응, 공유 카운터, 경쟁 조건 | 4 |
| [`07_send_sync.rs`](examples/07_send_sync.rs) | 자동으로 붙는 `Send`/`Sync`, `RefCell`은 `Send`지만 `Sync`가 아님 | 5 |
| [`compile_fail/`](compile_fail/) | 동시성 코드에서 컴파일러가 거부하는 대표 코드 5개(4절) | 2, 3, 4, 5 |

### 01_spawn_and_join — 스레드 만들기와 기다리기

```sh
cargo run -p p0-ch15-concurrency --example 01_spawn_and_join
```

```
1. join을 뒤에 두면 두 스레드가 동시에 실행됩니다
    [main] 1
    [새 스레드] 1
    [main] 2
    [새 스레드] 2
    [main] 3
    [새 스레드] 3
    [새 스레드] 4
    → 두 스레드가 모두 끝났습니다
2. join을 바로 부르면 새 스레드가 끝난 뒤에야 main이 진행합니다
    [새 스레드] 1
    [새 스레드] 2
    [새 스레드] 3
    [새 스레드] 4
    [main] 1
    [main] 2
    [main] 3
3. 스레드를 여러 개 만들고 JoinHandle을 모아 두었다가 한꺼번에 기다립니다
    [스레드 1] 일하는 중
    [스레드 2] 일하는 중
    [스레드 3] 일하는 중
    → 세 스레드가 모두 끝났습니다
4. join하지 않으면 main이 끝날 때 남은 스레드도 함께 끝나 버립니다
    [새 스레드] 1/5
    [새 스레드] 2/5
    [새 스레드] 3/5
    [main] 끝납니다 (join하지 않음)
```

- **1번 구역**: `[main]`과 `[새 스레드]` 줄이 섞이는 순서는 보장되지 않습니다. 이 환경에서는 위처럼 번갈아 나오는 경우가 대부분이었지만, 다른 컴퓨터나 바쁜 상황에서는 다르게 섞일 수 있습니다. 확실한 것은 일곱 줄이 모두 나온 뒤에 "→ 두 스레드가 모두 끝났습니다"가 나온다는 점입니다. `join`이 새 스레드를 끝까지 기다리기 때문입니다.
- **2번 구역**: `spawn` 바로 다음에 `join`했으므로 순서가 항상 위와 같습니다.
- **3번 구역**: 세 스레드의 출력 순서는 실행마다 다를 수 있습니다(이 환경에서도 `1, 3, 2` 순서가 나온 적이 있습니다). "→ 세 스레드가 모두 끝났습니다"는 항상 세 줄 뒤에 나옵니다.
- **4번 구역**: 새 스레드는 50ms마다 한 줄씩 다섯 줄을 출력하려 하지만, `main`이 120ms 뒤에 끝나면서 함께 끝납니다. 몇 번째 줄까지 나오는지는 실행마다 다를 수 있습니다(이 환경에서는 매번 `3/5`까지였습니다). "끝까지 마쳤습니다" 줄은 나오지 않습니다.

### 02_move_and_return — 값을 넘기고 결과를 돌려받기

```sh
cargo run -p p0-ch15-concurrency --example 02_move_and_return
```

```
1. move 클로저로 값의 소유권을 새 스레드에 넘깁니다
    [새 스레드] v = [1, 2, 3]
2. 스레드가 돌려준 값은 join으로 받습니다
    돌려받은 벡터: ["하나", "둘", "셋"]
3. 원본도 계속 써야 하면 복제(clone)해서 넘깁니다
    스레드가 계산한 합: 60, main의 원본: [10, 20, 30]
4. 일을 나눠 맡기고 결과를 모읍니다
    0번 조각의 합: 325
    1번 조각의 합: 950
    2번 조각의 합: 1575
    3번 조각의 합: 2200
    전체 합: 5050
5. 스레드가 패닉하면 join이 Err를 돌려줍니다

thread '<unnamed>' (6423711) panicked at part0-getting-started/ch15-concurrency/examples/02_move_and_return.rs:76:14:
index out of bounds: the len is 0 but the index is 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    스레드가 패닉했습니다. main은 계속 실행됩니다.
```

- **2번 구역**: 벡터의 소유권이 새 스레드로 들어갔다가 클로저의 반환값으로 다시 `main`에 돌아옵니다. `JoinHandle<Vec<String>>`이라는 타입 표기가 "이 스레드는 `Vec<String>`을 돌려준다"는 뜻입니다.
- **4번 구역**: 1부터 100까지를 25개씩 네 조각으로 나눠 스레드 네 개가 각자 합을 구합니다. 스레드는 아무 순서로 끝나도 되지만, `main`이 핸들을 만든 순서대로 `join`하고 출력하므로 출력 순서는 항상 위와 같습니다. 이처럼 스레드 안에서 출력하지 않고 결과를 돌려받아 `main`에서 출력하면 출력 순서를 정할 수 있습니다. `numbers.chunks(25)`가 돌려주는 조각은 `numbers`를 빌린 슬라이스이므로, `to_vec()`으로 스레드가 소유할 `Vec`을 따로 만들어 넘깁니다.
- **5번 구역**: `thread '<unnamed>' ...`부터 세 줄(앞의 빈 줄 포함)은 표준 에러로 나가는 패닉 메시지입니다. `<unnamed>`는 이름을 붙이지 않은 스레드라는 뜻이고, 괄호 안의 숫자는 스레드 ID로 실행할 때마다 달라집니다. 패닉은 그 스레드만 멈췄으므로 마지막 줄이 출력되고 프로그램은 종료 코드 0으로 끝납니다.

### 03_channel_basics — 채널의 기본

```sh
cargo run -p p0-ch15-concurrency --example 03_channel_basics
```

```
1. 채널을 만들고 값 하나를 주고받습니다
    받은 값: 안녕하세요
2. send는 값을 복사하지 않고 소유권을 옮깁니다
    받은 값: 힙에 있는 글자
    보낸 값과 같은 힙 데이터인가? true
3. recv는 값이 올 때까지 기다립니다
    받은 값: 42
    100ms 이상 기다렸는가? true
4. 송신자가 모두 사라지면 recv는 Err를 돌려줍니다
    Ok("첫째")
    Ok("둘째")
    Err(RecvError)
5. 수신자가 사라지면 send가 Err를 돌려주고, 보내려던 값을 되돌려줍니다
    sending on a closed channel: 돌려받은 값 = 받을 사람이 없는 편지
6. try_recv는 기다리지 않고 지금 상태를 바로 알려 줍니다
    아직 아무것도 안 보냄: Err(Empty)
    하나 보낸 뒤:        Ok(7)
    송신자를 버린 뒤:    Err(Disconnected)
7. try_recv로 기다리는 동안 다른 일을 할 수 있습니다
    받은 값: 작업 완료
    기다리는 동안 다른 일을 5번 했습니다
```

- **3번 구역**: 보내는 스레드가 100ms를 잔 뒤에 보냈으므로, `recv`가 그동안 기다렸다는 것을 시간 측정으로 확인합니다. `Instant::now()`는 지금 시각을 기록하고, `elapsed()`는 그때부터 흐른 시간을 돌려줍니다(시간 측정은 [Part 1 ch10](../../part1-standard-lib/ch10-env-process-time/README.md)에서 다룹니다).
- **4번 구역**: 송신자가 사라져도 이미 채널에 들어 있던 메시지는 받을 수 있고, 다 꺼낸 다음에야 `Err(RecvError)`가 나옵니다.
- **6번 구역**: `send`는 기다리지 않으므로 스레드 하나 안에서도 보내고 받을 수 있습니다. 덕분에 `try_recv`의 세 가지 결과를 순서가 정해진 출력으로 볼 수 있습니다.
- **7번 구역**: "다른 일을 N번 했습니다"의 N은 실행마다 다를 수 있습니다. 이 환경에서 여러 번 실행했을 때는 4나 5였습니다.

### 04_multiple_producers — 반복자로 받기와 여러 생산자

```sh
cargo run -p p0-ch15-concurrency --example 04_multiple_producers
```

```
1. 수신자를 for 문에 넣으면 메시지를 차례로 꺼냅니다
    받음: 채널로
    받음: 보낸
    받음: 메시지
    받음: 네 개
    → 송신자가 사라져서 반복이 끝났습니다
2. 송신자를 clone해 여러 생산자가 한 수신자에게 보냅니다
    생산자 1의 1번째 메시지
    생산자 2의 1번째 메시지
    생산자 3의 1번째 메시지
    생산자 1의 2번째 메시지
    생산자 3의 2번째 메시지
    생산자 2의 2번째 메시지
    생산자 1의 3번째 메시지
    생산자 3의 3번째 메시지
    생산자 2의 3번째 메시지
    받은 메시지 수: 9
    생산자 1의 메시지가 도착한 순서: [1, 2, 3]
    생산자 2의 메시지가 도착한 순서: [1, 2, 3]
    생산자 3의 메시지가 도착한 순서: [1, 2, 3]
3. 송신자가 하나라도 남아 있으면 채널은 닫히지 않습니다
    Ok("유일한 메시지")
    Err(Timeout)
    Err(Disconnected)
```

- **1번 구역**: 송신자가 하나뿐이므로 보낸 순서 그대로 도착합니다.
- **2번 구역**: "생산자 P의 N번째 메시지" 아홉 줄의 순서는 실행마다 다릅니다. 그 아래 네 줄은 항상 같습니다. 메시지는 모두 9개가 도착하고, 같은 생산자의 메시지끼리는 보낸 순서를 지킵니다. 예제의 `drop(tx);` 줄을 지우고 실행하면 아홉 줄을 받은 뒤 프로그램이 멈춘 채 끝나지 않습니다. 확인해 봤다면 `Ctrl+C`로 끝내세요.
- **3번 구역**: 작업 스레드의 송신자는 사라졌지만 `main`의 `tx`가 남아 있는 동안 `recv_timeout`은 `Err(Timeout)`을 돌려줍니다. `for` 문이었다면 여기서 영원히 기다렸을 것입니다. `drop(tx)` 뒤에야 `Err(Disconnected)`가 나옵니다.

### 05_mutex_basics — `Mutex`와 `MutexGuard`

```sh
cargo run -p p0-ch15-concurrency --example 05_mutex_basics
```

```
1. lock으로 잠그고, 받은 MutexGuard로 값을 바꿉니다
    처음:   Mutex { data: 5, poisoned: false, .. }
    바꾼 뒤: Mutex { data: 6, poisoned: false, .. }
2. MutexGuard가 살아 있는 동안은 잠겨 있습니다
    guard가 있을 때 try_lock 성공? false
    guard가 있을 때: Mutex { data: "<locked>", poisoned: false, .. }
    drop 후 try_lock 성공? true
    drop 후: Mutex { data: "공유 데이터", poisoned: false, .. }
3. MutexGuard로 안의 값의 메서드를 바로 부를 수 있습니다
    길이 4, 합 10
4. 변수에 담지 않은 MutexGuard는 그 문장이 끝날 때 바로 풀립니다
    다음 문장에서 try_lock 성공? true
    [1, 2, 3, 4, 5]
5. RefCell과 Mutex는 "이미 쓰는 중"을 다르게 다룹니다
    RefCell: 빌려 간 상태에서 또 빌리기 성공? false
    Mutex:   잠긴 상태에서 또 잠그기 성공? false
```

스레드를 하나만 쓰므로 출력은 항상 같습니다. `poisoned: false`는 잠금을 쥔 스레드가 패닉한 적이 없다는 뜻입니다(2.7절). 5번 구역에서 `try_lock` 대신 `lock`을 쓰면 같은 스레드가 이미 쥔 잠금을 다시 기다리게 되므로, 예제는 기다리지 않는 `try_borrow_mut`와 `try_lock`으로만 비교합니다.

### 06_shared_counter — `Arc<Mutex<T>>`로 만든 공유 카운터

```sh
cargo run -p p0-ch15-concurrency --example 06_shared_counter
```

```
1. (복습) 한 스레드 안에서는 Rc<RefCell<T>>로 값을 함께 소유하고 바꿉니다
    결과: 3
2. 여러 스레드에서는 Arc<Mutex<T>>로 같은 일을 합니다
    결과: 10000 (기대값 10000)
    남은 Arc 소유자 수: 1
3. 잠금을 둘로 나누면 컴파일은 되지만 결과가 틀립니다
    결과: 2599 (기대값 10000)
```

- **2번 구역**: 10개 스레드가 1000번씩 더하므로 결과는 항상 10000입니다. 모든 스레드를 `join`한 뒤에는 스레드에 넘긴 `Arc` 복제본이 모두 drop되었으므로 `Arc::strong_count`가 1입니다.
- **3번 구역**: 결과 값은 실행마다 다릅니다. 이 환경에서 여러 번 실행했을 때는 대략 2000에서 4500 사이였고, 10000이 나온 적은 없었습니다. 이유는 2.10절에 있습니다.

### 07_send_sync — `Send`와 `Sync`

```sh
cargo run -p p0-ch15-concurrency --example 07_send_sync
```

```
1. 필드가 모두 Send인 구조체는 자동으로 Send입니다
    [새 스레드] 주문 7: ["커피", "빵"]
2. RefCell은 Send라서 통째로 다른 스레드에 넘길 수는 있습니다
    돌려받은 RefCell 안의 값: [1, 2, 3, 4]
3. 여러 스레드가 함께 읽으려면 Sync여야 합니다
    [스레드 1] 읽은 값: "문자열"
    [스레드 2] 읽은 값: "문자열"
    [스레드 1] 읽은 값: [1, 2, 3]
    [스레드 2] 읽은 값: [1, 2, 3]
    [스레드 1] 읽은 값: Order { id: 8, items: ["차"] }
    [스레드 2] 읽은 값: Order { id: 8, items: ["차"] }
```

3번 구역의 `read_from_two_threads`는 값 하나를 `Arc`로 감싸 두 스레드가 함께 읽게 하는 제네릭 함수입니다. 트레이트 바운드 `T: Debug + Send + Sync + 'static`이 "여러 스레드가 함께 읽어도 되는 타입만 받는다"는 조건을 표현합니다. 스레드가 읽은 내용을 문자열로 돌려받아 `main`에서 출력하므로 출력 순서는 항상 같습니다. 예제 안에 주석 처리된 `read_from_two_threads(RefCell::new(0));`의 주석을 풀면 `` `RefCell<{integer}>` cannot be shared between threads safely``(E0277)가, `read_from_two_threads(std::rc::Rc::new(0));`의 주석을 풀면 `Rc`가 `Send`도 `Sync`도 아니라는 E0277이 납니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch15-concurrency
rustc --edition 2024 compile_fail/e0373_thread_borrows_local.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐서 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요.

각 절의 **막은 버그**는, 컴파일러가 이 코드를 받아들였다면 실행 중에 어떤 일이 생길 수 있었는지를 설명합니다.

### 4.1 `move` 없이 지역 변수를 빌리는 스레드 — E0373

파일: [`compile_fail/e0373_thread_borrows_local.rs`](compile_fail/e0373_thread_borrows_local.rs)

```
error[E0373]: closure may outlive the current function, but it borrows `v`, which is owned by the current function
  --> compile_fail/e0373_thread_borrows_local.rs:16:32
   |
16 |     let handle = thread::spawn(|| {
   |                                ^^ may outlive borrowed value `v`
17 |         println!("새 스레드가 받은 벡터: {v:?}");
   |                                           - `v` is borrowed here
   |
note: function requires argument type to outlive `'static`
  --> compile_fail/e0373_thread_borrows_local.rs:16:18
   |
16 |       let handle = thread::spawn(|| {
   |  __________________^
17 | |         println!("새 스레드가 받은 벡터: {v:?}");
18 | |     });
   | |______^
help: to force the closure to take ownership of `v` (and any other referenced variables), use the `move` keyword
   |
16 |     let handle = thread::spawn(move || {
   |                                ++++
```

- **읽는 법**: 첫 줄이 "클로저가 지금 함수보다 오래 살 수 있는데, 지금 함수가 소유한 `v`를 빌린다"고 말합니다. `note:`는 그 근거로 `spawn`이 인자에 `'static`을 요구한다는 것(2.4절)을 가리킵니다.
- **막은 버그**: `main`이 `v`를 버린 뒤에도 새 스레드가 실행 중이라면, 새 스레드는 해제된 메모리를 읽게 됩니다(해제 후 사용). 실행 순서에 따라 나타났다 사라졌다 하는 전형적인 동시성 버그입니다.
- **해결**: `help:`대로 `move`를 붙여 `v`의 소유권을 스레드로 옮깁니다([`02_move_and_return.rs`](examples/02_move_and_return.rs)의 1번 구역). `main`에서도 `v`를 계속 써야 한다면 `clone()`한 사본을 옮깁니다.

### 4.2 `send`한 값을 다시 사용 — E0382

파일: [`compile_fail/e0382_use_after_send.rs`](compile_fail/e0382_use_after_send.rs)

```
error[E0382]: borrow of moved value: `val`
  --> compile_fail/e0382_use_after_send.rs:19:26
   |
17 |         let val = String::from("안녕");
   |             --- move occurs because `val` has type `String`, which does not implement the `Copy` trait
18 |         tx.send(val).unwrap();
   |                 --- value moved here
19 |         println!("보낸 값: {val}");
   |                             ^^^ value borrowed here after move
```

- **읽는 법**: ch4에서 본 E0382와 똑같은 메시지입니다. 18번 줄의 `send`가 `val`의 소유권을 가져갔습니다(value moved here).
- **막은 버그**: 받은 스레드는 이제 `val`의 소유자이므로 값을 바꾸거나 버릴 수 있습니다. 보낸 스레드가 그 순간에 `val`을 읽으면, 동기화 없이 두 스레드가 같은 데이터에 접근하는 데이터 경쟁이 되거나 이미 해제된 메모리를 읽게 됩니다.
- **해결**: 필요한 일을 `send` 전에 끝냅니다. 보낸 뒤에도 같은 내용이 필요하면 `tx.send(val.clone())`처럼 사본을 보냅니다.

### 4.3 `Arc` 없이 `Mutex`를 여러 스레드로 옮기기 — E0382

파일: [`compile_fail/e0382_mutex_moved_into_threads.rs`](compile_fail/e0382_mutex_moved_into_threads.rs)

```
error[E0382]: borrow of moved value: `counter`
  --> compile_fail/e0382_mutex_moved_into_threads.rs:30:25
   |
15 |     let counter = Mutex::new(0);
   |         ------- move occurs because `counter` has type `std::sync::Mutex<i32>`, which does not implement the `Copy` trait
...
18 |     for _ in 0..10 {
   |     -------------- inside of this loop
19 |         let handle = thread::spawn(move || {
   |                                    ------- value moved into closure here, in previous iteration of loop
...
30 |     println!("결과: {}", *counter.lock().unwrap());
   |                           ^^^^^^^ value borrowed here after move
```

- **읽는 법**: "value moved into closure here, in previous iteration of loop"가 핵심입니다. 반복문의 첫 바퀴에서 `counter`가 첫 번째 스레드의 클로저로 이동했으므로, 그 뒤로는 `counter`를 쓸 수 없습니다. 에러는 30번 줄에서 보고되지만 원인은 19번 줄의 `move`입니다.
- **막은 버그**: `Mutex`는 여러 스레드가 번갈아 **접근**하게 해 줄 뿐, 소유자를 여럿 두게 해 주지는 않습니다. 같은 값을 열 개의 스레드가 각자 소유한다면 각 스레드가 끝날 때마다 그 값을 해제하려 할 것입니다(이중 해제).
- **해결**: `Arc::new(Mutex::new(0))`으로 감싸고, 반복문 안에서 `let counter = Arc::clone(&counter);`로 복제본을 만들어 옮깁니다([`06_shared_counter.rs`](examples/06_shared_counter.rs)의 2번 구역).

### 4.4 `Rc`를 다른 스레드로 보내기 — E0277(`Send`)

파일: [`compile_fail/e0277_rc_is_not_send.rs`](compile_fail/e0277_rc_is_not_send.rs)

4.3의 코드를 ch14에서 배운 `Rc`로 고쳐 본 것입니다.

```
error[E0277]: `Rc<std::sync::Mutex<i32>>` cannot be sent between threads safely
  --> compile_fail/e0277_rc_is_not_send.rs:22:36
   |
22 |           let handle = thread::spawn(move || {
   |                        ------------- ^------
   |                        |             |
   |  ______________________|_____________within this `{closure@compile_fail/e0277_rc_is_not_send.rs:22:36: 22:43}`
   | |                      |
   | |                      required by a bound introduced by this call
23 | |             let mut num = counter.lock().unwrap();
24 | |             *num += 1;
25 | |         });
   | |_________^ `Rc<std::sync::Mutex<i32>>` cannot be sent between threads safely
   |
   = help: within `{closure@compile_fail/e0277_rc_is_not_send.rs:22:36: 22:43}`, the trait `Send` is not implemented for `Rc<std::sync::Mutex<i32>>`
note: required because `counter` is used within this closure
  --> compile_fail/e0277_rc_is_not_send.rs:23:27
   |
23 |             let mut num = counter.lock().unwrap();
   |                           ^^^^^^^
note: required by a bound in `spawn`
  --> /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/std/src/thread/functions.rs:125:0
```

- **읽는 법**: 메시지가 길지만 첫 줄과 `= help:` 줄만 보면 됩니다. "`Rc<Mutex<i32>>`는 스레드 사이에 안전하게 보낼 수 없다", 이유는 "`Send` 트레이트가 구현되어 있지 않다"입니다. 아래의 두 `note:`는 "클로저가 이 `Rc`를 캡처했고(required because `counter` is used within this closure), `spawn`이 클로저에 `Send`를 요구한다(required by a bound in `spawn`)"는 연결 고리를 보여 줍니다. 마지막 `-->`가 가리키는 `/rustc/...` 경로는 표준 라이브러리 소스의 위치로, 내 코드의 위치가 아닙니다.
- **막은 버그**: 여러 스레드가 동시에 `Rc`를 `clone`하거나 drop하면 참조 카운트 갱신이 서로 덮어써질 수 있습니다. 카운트가 실제보다 작아지면 다른 스레드가 쓰는 중에 값이 해제되고, 커지면 영영 해제되지 않습니다.
- **해결**: `Rc`를 `Arc`로 바꿉니다. `use std::rc::Rc;`를 `use std::sync::Arc;`로, `Rc::new`와 `Rc::clone`을 `Arc::new`와 `Arc::clone`으로 바꾸면 됩니다. 사용법이 같기 때문입니다.

### 4.5 `RefCell`을 여러 스레드가 함께 쓰기 — E0277(`Sync`)

파일: [`compile_fail/e0277_refcell_is_not_sync.rs`](compile_fail/e0277_refcell_is_not_sync.rs)

이번에는 `Arc`로 감쌌지만, 안에 `Mutex` 대신 `RefCell`을 넣었습니다.

```
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
  --> compile_fail/e0277_refcell_is_not_sync.rs:23:36
   |
23 |           let handle = thread::spawn(move || {
   |  ______________________-------------_^
   | |                      |
   | |                      required by a bound introduced by this call
24 | |             *counter.borrow_mut() += 1;
25 | |         });
   | |_________^ `RefCell<i32>` cannot be shared between threads safely
   |
   = help: the trait `Sync` is not implemented for `RefCell<i32>`
   = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
   = note: required for `Arc<RefCell<i32>>` to implement `Send`
```

(뒤의 `note:` 두 개는 4.4와 같아 생략했습니다.)

- **읽는 법**: 이번에는 "보낼 수 없다(sent)"가 아니라 "**공유**할 수 없다(shared)"이고, 빠진 트레이트도 `Sync`입니다. 마지막 `= note:`가 연결 고리를 알려 줍니다. `Arc<RefCell<i32>>`가 `Send`이려면 `RefCell<i32>`가 `Sync`여야 하는데(2.9절), 그렇지 않습니다. 컴파일러는 `RwLock`을 제안하는데, `RwLock`은 Part 1에서 다룹니다. 이 챕터에서 배운 `Mutex`로도 해결됩니다.
- **막은 버그**: `RefCell`은 "지금 빌려 간 사람이 있는가"를 평범한 정수로 셉니다. 두 스레드가 동시에 `borrow_mut`를 부르면 둘 다 "아무도 안 빌렸다"고 보고 가변 참조를 하나씩 얻을 수 있습니다. 가변 참조가 동시에 둘이 되는 것, 즉 데이터 경쟁입니다.
- **해결**: `RefCell`을 `Mutex`로, `borrow_mut()`를 `lock().unwrap()`으로, `borrow()`를 `lock().unwrap()`으로 바꿉니다(2.8절의 대응표).

### 4.6 컴파일은 되지만 멈추거나 틀리는 코드

다음 세 가지는 컴파일러가 잡아 주지 않으므로 코드를 쓸 때 직접 확인해야 합니다.

- **송신자를 버리지 않음**: `for msg in rx`가 끝나지 않고 프로그램이 멈춥니다. 반복문 앞에 원래의 `tx`를 `drop`했는지 확인합니다(2.6절, 04의 3번 구역).
- **같은 `Mutex`를 같은 스레드에서 두 번 잠금, 또는 여러 잠금을 스레드마다 다른 순서로 잠금**: 교착 상태로 프로그램이 멈출 수 있습니다. `MutexGuard`를 필요 이상으로 오래 들고 있지 않은지, 여러 잠금을 항상 같은 순서로 얻는지 확인합니다(2.10절).
- **한 덩어리여야 할 일을 잠금 둘로 나눔**: 결과가 실행마다 달라집니다. 읽고 고치고 쓰는 일 전체를 `MutexGuard` 하나로 감쌌는지 확인합니다(06의 3번 구역).

## 5. 연습 문제

### 연습 1. 스레드 네 개로 나눠 계산하기

1부터 1000까지의 수 중 3의 배수이거나 5의 배수인 수의 합을 구하세요. 수를 250개씩 네 조각으로 나눠 스레드 네 개가 각자 자기 조각의 합을 구하고, `main`이 네 결과를 더해 출력합니다. 답은 `234168`입니다.

<details>
<summary>힌트</summary>

[`02_move_and_return.rs`](examples/02_move_and_return.rs)의 4번 구역과 구조가 같습니다. `(1..=1000).collect::<Vec<u64>>()`로 벡터를 만들고 `chunks(250)`으로 나눈 뒤, 조각마다 `to_vec()`으로 스레드가 소유할 벡터를 만들어 `move`로 넘깁니다. 스레드 안에서는 `part.iter().filter(|n| *n % 3 == 0 || *n % 5 == 0).sum::<u64>()`처럼 합을 구해 돌려줍니다. `main`은 `JoinHandle`들을 `Vec`에 모아 두었다가 차례로 `join().unwrap()`해서 더합니다.

</details>

### 연습 2. 채널로 결과 모으기

작업 스레드 세 개를 만드세요. 첫 번째 스레드는 1~10, 두 번째는 11~20, 세 번째는 21~30의 각 수를 제곱해 채널로 하나씩 보냅니다. `main`은 `for` 문으로 받은 값을 모두 더하고, 받은 개수와 합을 출력합니다. `받은 개수: 30, 합: 9455`가 나와야 합니다. 완성한 뒤 `drop(tx)` 줄을 지우고 다시 실행해 보세요. 무슨 일이 일어나고, 왜 그런가요?

<details>
<summary>힌트</summary>

[`04_multiple_producers.rs`](examples/04_multiple_producers.rs)의 2번 구역처럼 `for worker in 0..3` 반복문 안에서 `let tx = tx.clone();`으로 스레드마다 송신자를 만들어 `move`로 넘깁니다. 스레드 안에서는 `let start = worker * 10 + 1;`로 시작 값을 정하고 `for n in start..=start + 9`로 `tx.send(n * n)`을 보냅니다. 반복문이 끝나면 `drop(tx);`로 원래 송신자를 버린 다음 `for square in rx`로 받습니다. `drop(tx)`를 지우면 30개를 다 받은 뒤에도 `main`의 `tx`가 살아 있어 채널이 닫히지 않으므로, 프로그램이 출력 없이 멈춥니다. `Ctrl+C`로 끝내세요.

</details>

### 연습 3. 컴파일 에러를 따라가며 고치기

아래 코드는 세 스레드가 공유 기록에 한 줄씩 추가하는 프로그램입니다. 컴파일되지 않습니다. 에러 메시지를 읽고 한 번에 한 곳씩 고쳐 보세요. 몇 번 고쳐야 하고, 각 단계의 에러는 무엇을 막았나요?

```rust
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

fn main() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut handles = Vec::new();
    for id in 1..=3 {
        let log = Rc::clone(&log);
        handles.push(thread::spawn(move || {
            log.borrow_mut().push(format!("스레드 {id} 완료"));
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("기록 {}개: {:?}", log.borrow().len(), log.borrow());
}
```

<details>
<summary>힌트</summary>

처음에는 `` `Rc<RefCell<Vec<String>>>` cannot be sent between threads safely``(E0277, `Send`)가 납니다. `Rc`를 `Arc`로 바꾸면(4.4절) 이번에는 `` `RefCell<Vec<String>>` cannot be shared between threads safely``(E0277, `Sync`)가 납니다. `RefCell`을 `Mutex`로 바꾸면(4.5절) 컴파일됩니다. 이때 `borrow_mut()`와 `borrow()`는 `lock().unwrap()`으로 바꿉니다. 마지막 `println!`에서는 `let log = log.lock().unwrap();`로 잠금을 한 번만 얻은 뒤 `log.len()`과 `log`를 출력하세요. 한 문장 안에서 `lock()`을 두 번 부르면 첫 번째 `MutexGuard`가 아직 살아 있는 상태에서 다시 잠그게 됩니다(2.7절). 실행하면 `기록 3개`가 출력되고, 세 줄의 순서는 실행마다 다를 수 있습니다.

</details>

### 연습 4. 사라지는 더하기 고치기

[`06_shared_counter.rs`](examples/06_shared_counter.rs)의 3번 구역은 결과가 10000보다 작게 나옵니다. 잠금 횟수를 늘리지 않고 이 구역의 반복문 본문만 고쳐서, 몇 번을 실행해도 `결과: 10000 (기대값 10000)`이 나오게 하세요.

<details>
<summary>힌트</summary>

지금 코드는 `counter.lock()`을 두 번 불러서, 읽은 뒤 잠금이 한 번 풀렸다가 쓸 때 다시 잠깁니다. `let mut num = counter.lock().unwrap();`로 `MutexGuard`를 변수에 담으면, 그 변수의 스코프(반복문 본문)가 끝날 때까지 잠금이 유지됩니다. 그 안에서 `let current = *num;`으로 읽고 `*num = current + 1;`로 쓰면 읽기와 쓰기 사이에 다른 스레드가 끼어들 수 없습니다. 2번 구역의 `*counter.lock().unwrap() += 1;`이 한 문장으로 같은 일을 하는 이유도 생각해 보세요.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [16. Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
  - [16.1 Using Threads to Run Code Simultaneously](https://doc.rust-lang.org/book/ch16-01-threads.html)
  - [16.2 Transfer Data Between Threads with Message Passing](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
  - [16.3 Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
  - [16.4 Extensible Concurrency with Send and Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html)
- Rust By Example: [Std misc](https://doc.rust-lang.org/rust-by-example/std_misc.html) 장의 다음 절
  - [Threads](https://doc.rust-lang.org/rust-by-example/std_misc/threads.html): 스레드 여러 개를 만들고 `join`하기
  - [Testcase: map-reduce](https://doc.rust-lang.org/rust-by-example/std_misc/threads/testcase_mapreduce.html): 일을 나눠 스레드에 맡기고 결과를 합치기. 연습 1과 같은 구조입니다. `'static` 문자열 슬라이스를 스레드에 넘기는 방식과 `split_whitespace`, `char::to_digit`이 나오는데, 읽기 어려운 부분은 건너뛰고 구조만 보면 됩니다.
  - [Channels](https://doc.rust-lang.org/rust-by-example/std_misc/channels.html): 송신자 복제와 `recv`
  - `Arc`는 Std library types 장의 [Arc](https://doc.rust-lang.org/rust-by-example/std/arc.html) 절에 있습니다.
- Rustlings (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - [`20_threads`](https://github.com/rust-lang/rustlings/tree/main/exercises/20_threads): `threads1.rs`(`JoinHandle`로 결과 모으기), `threads2.rs`(`Arc`만으로는 값을 고칠 수 없을 때), `threads3.rs`(송신자를 두 스레드에 넘기기)
  - [`19_smart_pointers`](https://github.com/rust-lang/rustlings/tree/main/exercises/19_smart_pointers)의 `smart_pointers3.rs`: `Rc` 대신 `Arc`로 여러 스레드에 벡터 공유하기
- 표준 라이브러리 문서
  - [`std::thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html): `'static`과 `Send` 바운드가 필요한 이유
  - [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html), [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html), [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html)
  - [`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html), [`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html)
- 에러 설명: `rustc --explain E0373`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
