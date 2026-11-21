# ch11. 스레드와 동기화

[Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md)에서 `thread::spawn`과 `join`, `move` 클로저, 채널, `Arc<Mutex<T>>`, `Send`/`Sync`를 배웠습니다. 그 뒤로 여러 챕터가 스레드와 동기화에 관한 일을 이 챕터로 미뤄 두었습니다.

- Part 0 ch15: 지역 변수를 `move` 없이 빌려 쓰는 스코프 스레드, `RwLock`, `Condvar`, 원자 타입
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md): `static mut` 대신 쓰는 원자 타입과 처음 쓸 때 초기화되는 `OnceLock`/`LazyLock`
- [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md): `thread::Builder`(스레드 이름과 생성 실패 처리), `Condvar`, `Mutex` 없는 카운터
- [Part 1 ch7](../ch7-smart-pointers-memory/README.md): `Cell`/`RefCell`/`OnceCell`/`LazyCell`/`Rc`의 스레드 안전한 짝, `static`에 `OnceCell`을 두면 나는 E0277의 해결
- [Part 1 ch8](../ch8-errors-panic/README.md): 패닉 훅이 다른 스레드의 패닉에도 불린다는 점, 스레드의 패닉과 `join`의 `Err`

이 챕터는 이 약속들을 여섯 갈래로 정리합니다. 스레드를 만드는 방법(2.2~2.3절), 잠금을 고르는 방법(2.4~2.5절), 조건을 기다리는 방법(2.6절), 채널을 더 쓰는 방법(2.7절), 잠금 없이 값 하나를 고치는 원자 타입(2.8절), 한 번만 초기화하는 전역 값(2.9절)입니다. 마지막 2.10절에 "이럴 때 이 도구" 선택표가 있습니다. Part 0 ch15에서 다룬 내용은 반복하지 않습니다.

- 패키지: `p1-ch11-threads-sync`
- 기준 문서: [`std::thread`](https://doc.rust-lang.org/std/thread/index.html), [`std::sync`](https://doc.rust-lang.org/std/sync/index.html), [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html), [`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024), macOS (스레드 생성 실패의 에러 내용과 `available_parallelism` 값은 환경마다 다를 수 있습니다)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `thread::scope`로 지역 변수를 `move` 없이 빌려 쓰는 스레드를 만들고, 그것이 허용되는 이유(스코프가 끝나기 전에 모든 스레드가 join됨)와 스코프 안에서 패닉했을 때의 동작을 설명할 수 있다. `thread::Builder`로 스레드에 이름을 붙이고 생성 실패를 `io::Result`로 처리할 수 있다.
2. `Mutex`의 `try_lock`, 가드 범위 줄이기, 잠그는 순서 정하기, 포이즈닝 복구, `get_mut`/`into_inner`를 상황에 맞게 쓰고, `RwLock`이 `Mutex`보다 나은 경우를 설명할 수 있다.
3. `Condvar`로 조건이 만족될 때까지 기다리는 코드를 `wait_while`로 작성하고, 가짜 깨어남과 "신호는 쌓이지 않는다"는 점 때문에 조건을 항상 다시 확인해야 하는 이유를 설명할 수 있다. 용량이 정해진 큐나 `sync_channel`로 배압이 있는 생산자-소비자를 작성할 수 있다.
4. 원자 타입의 `fetch_add`, `compare_exchange`, `try_update`로 잠금 없이 값을 고치고, `load` 뒤에 `store`하는 코드가 왜 틀리는지와 `Ordering`을 고르는 실용 지침을 설명할 수 있다.
5. `OnceLock`/`LazyLock`으로 전역 설정과 캐시를 만들고, `static`에 둘 수 있는 값의 조건(`Sync`, 컴파일할 때 계산되는 초기값)을 설명하며, 상황에 맞는 동기화 도구를 고를 수 있다.

## 2. 핵심 개념

### 2.1 이 챕터의 지도

Part 0 ch15에서 배운 도구와 이 챕터에서 더하는 도구를 나란히 놓으면 다음과 같습니다.

| 하려는 일 | Part 0 ch15 | 이 챕터 |
| --- | --- | --- |
| 스레드 만들기 | `thread::spawn` + `move` | `thread::scope`(빌려 쓰기), `thread::Builder`(이름·스택 크기·실패 처리) — 2.2, 2.3절 |
| 값을 함께 고치기 | `Arc<Mutex<T>>` | `try_lock`, 포이즈닝 복구, `RwLock` — 2.4, 2.5절 |
| 무언가를 기다리기 | `join`, `recv` | `Condvar`, `Barrier` — 2.6절 |
| 메시지 보내기 | `channel`, `recv`, `try_recv` | `sync_channel`과 배압, `try_iter` — 2.7절 |
| 정수 하나 고치기 | `Mutex<i32>` | 원자 타입 — 2.8절 |
| 전역 값 | (Part 0 ch19의 `static Mutex`) | `OnceLock`, `LazyLock`, `static` 원자 변수 — 2.9절 |

이 챕터의 모든 도구는 `std::thread`, `std::sync`, `std::sync::mpsc`, `std::sync::atomic`에 있습니다. 표준 라이브러리 문서는 `std::sync`의 저수준 도구가 "실수하기 쉽고 불편하다"고 말하며, 그래서 `Mutex`, `Condvar` 같은 고수준 도구를 함께 제공한다고 설명합니다. 이 챕터의 예제도 고수준 도구부터 쓰고, 원자 타입은 값 하나를 다루는 데만 씁니다.

### 2.2 스코프 스레드: `thread::scope`

Part 0 ch15에서 `thread::spawn`에 지역 변수를 빌리는 클로저를 넘기면 E0373이 났습니다. 새 스레드가 언제 끝날지 모르는데 지역 변수는 함수가 끝나면 사라지기 때문입니다. 그래서 `spawn`은 클로저에 `'static`을 요구했고, 우리는 `move`로 값을 옮기거나 `Arc`로 감싸야 했습니다.

**스코프 스레드(scoped thread)** 는 이 문제를 반대쪽에서 풉니다. "스레드가 언제 끝날지 모른다"를 "이 블록이 끝나기 전에 반드시 끝난다"로 바꾸는 것입니다(1.63.0).

```rust
let words = vec!["사과", "바나나", "체리", "두리안"];
let limit = 2;

thread::scope(|s| {
    let long = s.spawn(|| words.iter().filter(|w| w.chars().count() > limit).count());
    let total = s.spawn(|| words.iter().map(|w| w.chars().count()).sum::<usize>());
    println!("{} {}", long.join().unwrap(), total.join().unwrap());
}); // 여기서 스코프 안의 모든 스레드가 끝났음이 보장됩니다.

println!("{words:?}"); // 대여가 끝났으므로 다시 마음대로 씁니다.
```

`thread::scope`는 클로저에 `Scope` 값 `s`를 넘기고, 스레드는 `s.spawn`으로 만듭니다. 두 `spawn`의 시그니처를 비교하면 차이가 한눈에 보입니다.

```rust
// std::thread::spawn
F: FnOnce() -> T + Send + 'static,
// Scope::spawn
F: FnOnce() -> T + Send + 'scope,
```

`'static`이 `'scope`로 바뀌었습니다. `'scope`는 "스코프가 살아 있는 동안"이라는 라이프타임이고, `scope` 함수는 이 라이프타임이 끝나기 전에 **join하지 않은 스레드까지 모두 join합니다.** 따라서 스코프 밖에서 만든 지역 변수(`words`, `limit`)는 스코프 스레드보다 오래 산다는 것이 보장되고, 클로저가 그것을 빌려도 댕글링 참조가 생길 수 없습니다. 문서는 빌려 오는 대상의 라이프타임을 `'env`라고 부르며 `'env: 'scope` 관계를 `Scope` 타입에 적어 둡니다.

이 보장 덕분에 다음 일들이 가능해집니다.

- **읽기 전용 대여를 여러 스레드가 함께**: 위 코드처럼 `&words`를 두 스레드가 동시에 읽습니다. `Vec<&str>`은 `Sync`이므로 `&Vec`은 `Send`입니다.
- **겹치지 않는 가변 대여**: `numbers.chunks_mut(3)`이 내주는 `&mut [u32]` 조각을 스레드마다 하나씩 넘기면, 각 조각에 가변 참조가 하나뿐이므로 대여 규칙을 지키면서 동시에 고칠 수 있습니다. Part 0 ch15의 연습 1에서 조각마다 `to_vec()`으로 복사해 넘기던 일이 사라집니다.
- **핸들을 버려도 기다려 줌**: `s.spawn(..)`의 핸들(`ScopedJoinHandle`)을 변수에 담지 않아도 스코프가 끝날 때 join됩니다. `thread::spawn`의 핸들을 버리면 스레드가 떨어져 나가는(detach) 것과 다릅니다.
- **스코프의 반환값**: `thread::scope`는 클로저가 돌려준 값을 그대로 돌려주므로, 스코프 안에서 `join`한 결과를 바깥으로 꺼낼 수 있습니다.

스코프 스레드에서도 `move`를 쓸 때가 있습니다. 반복문 변수(`i`)나 `chunks_mut`이 준 `&mut` 조각처럼 **반복마다 새로 생기는 값**은 클로저가 빌리면 반복이 끝날 때 사라지므로 옮겨야 합니다. 이때 `move`가 옮기는 것은 참조(`&mut [u32]`)이지 데이터가 아닙니다. 공유할 큰 값까지 옮기지 않으려면 `let words = &words;`처럼 참조를 만든 뒤 `move`합니다(예제 04, 05에서 여러 번 쓰는 모양입니다).

대여 규칙은 스코프 안에서도 그대로입니다. 두 스레드가 같은 `Vec`을 동시에 가변으로 빌리면 E0499입니다(4.1절). 또 빌려 줄 변수는 `thread::scope`를 부르기 **전에** 만들어야 합니다. 스코프 클로저 안에서 만든 지역 변수는 그 클로저가 끝날 때 사라지지만 스레드는 그 뒤(스코프가 끝날 때)까지 살 수 있으므로 E0373이 납니다(4.5절의 표).

**스코프 안의 패닉.** join한 스레드의 패닉은 `join`이 돌려주는 `Err`로 받고, 스코프는 정상적으로 끝납니다. join하지 않은 스레드가 패닉하면 스코프는 나머지 스레드를 모두 기다린 뒤 **`scope` 함수 자체가 패닉합니다.** 이때 페이로드는 `"a scoped thread panicked"`라는 `&str`입니다([`01_scoped_threads.rs`](examples/01_scoped_threads.rs)의 5번 구역). 패닉한 스레드를 따로 처리하고 싶다면 스코프가 끝나기 전에 직접 `join`합니다.

| | `thread::spawn` | `thread::scope` + `s.spawn` |
| --- | --- | --- |
| 지역 변수 | `move`로 옮기거나 `Arc`로 공유 | 그냥 빌림 |
| 언제 끝나는가 | 모름(`join`해야 앎) | 스코프가 끝나기 전에 반드시 |
| 핸들을 버리면 | 떨어져 나감(detach) | 스코프 끝에서 자동 join |
| 어울리는 일 | 프로그램 내내 사는 작업자, 서버의 연결 처리 | 일을 나눠 계산하고 결과를 모으기 |

### 2.3 스레드 다루기: `Builder`, 현재 스레드, 스레드 지역 변수

**`thread::Builder`** 는 스레드를 만들기 전에 설정을 정하는 타입입니다. 문서에 따르면 정할 수 있는 것은 이름과 스택 크기 두 가지입니다.

```rust
let handle = thread::Builder::new()
    .name("작업자-1".to_string())   // 패닉 메시지와 운영체제 도구에 보이는 이름
    .stack_size(256 * 1024)         // 바이트 단위. 기본값은 플랫폼마다 다르며 현재 2 MiB
    .spawn(|| thread::current().name().map(String::from))?; // io::Result<JoinHandle<T>>
```

`thread::spawn`은 안에서 기본 설정의 `Builder`로 `spawn`하고 결과를 `unwrap`합니다. 그래서 운영체제가 스레드를 만들지 못하면 `thread::spawn`은 패닉하고, `Builder::spawn`은 `Err`를 돌려줍니다. 예제 02의 2번 구역은 `stack_size(usize::MAX)`로 일부러 실패시키며, 이 환경(macOS)에서는 `invalid stack size`(`InvalidInput`)가 나왔습니다. 스코프 스레드에는 `Builder::spawn_scoped(s, 클로저)`를 씁니다. 이름에 널 문자(`\0`)가 있으면 `spawn`이 패닉한다고 문서에 적혀 있습니다. 스택 크기는 환경 변수 `RUST_MIN_STACK`으로도 정할 수 있고, `Builder::stack_size`가 그보다 우선합니다.

**현재 스레드 정보.** `thread::current()`는 지금 실행 중인 스레드의 `Thread` 값을 돌려줍니다. `JoinHandle::thread()`로도 같은 스레드의 `Thread`를 얻습니다.

| 메서드 | 돌려주는 것 |
| --- | --- |
| `name()` | `Option<&str>`. `Builder`로 이름을 붙인 스레드와 `main` 스레드는 `Some`, `thread::spawn`으로 만든 스레드는 `None` |
| `id()` | `ThreadId`. 스레드마다 다르며 `==`로 비교할 수 있습니다. 숫자로 꺼내는 `as_u64`는 아직 nightly 전용입니다. |

**`available_parallelism()`** (1.59.0)은 이 프로그램이 동시에 돌릴 만한 스레드 수의 추정값을 `io::Result<NonZero<usize>>`로 돌려줍니다. 보통 CPU 코어 수와 같지만, 가상 머신이나 컨테이너가 제한을 걸면 더 작아질 수 있습니다. 알 수 없는 환경에서는 `Err`이므로 `.map(|n| n.get()).unwrap_or(1)`처럼 기본값을 둡니다. 문서는 이 값을 매번 다시 계산하므로 자주 불리는 코드에서 부르지 말라고 권합니다.

**스레드 지역 변수(thread-local).** `thread_local!`로 선언한 변수는 전역 변수처럼 어디서든 이름으로 접근하지만, **스레드마다 따로 있습니다.** 공유하지 않으므로 동기화가 필요 없고, 안의 값을 바꾸려면 ch7의 `Cell`/`RefCell`을 씁니다.

```rust
thread_local! {
    static CALLS: Cell<u32> = const { Cell::new(0) };
}

CALLS.set(CALLS.get() + 1); // LocalKey<Cell<T>>의 get/set (1.73.0)
```

초기값이 상수라면 `const { }`로 감싸는 것이 권장되며, 감싸지 않으면 Clippy가 `missing_const_for_thread_local`로 알려 줍니다. 예제 02의 5번 구역에서 `main`이 세 번 부른 값(3)과 새 스레드가 한 번 부른 값(1)이 서로 영향을 주지 않는 것을 확인합니다. `RefCell`이 든 스레드 지역 변수에는 `with_borrow`/`with_borrow_mut`(1.73.0)이 있습니다.

**그 밖의 함수.** `thread::sleep(d)`는 최소 `d`만큼 잠들고, `thread::yield_now()`는 이번 실행 시간을 다른 스레드에 양보합니다. `thread::park()`는 다른 스레드가 `Thread::unpark()`로 깨울 때까지 잠드는 저수준 도구로, 비동기 실행기를 만들 때 쓰이므로 [ch13](../ch13-async-primitives/README.md)에서 다룹니다. 이 챕터에서 "기다리기"는 `Condvar`와 채널로 합니다.

**스레드의 패닉과 패닉 훅.** [Part 1 ch8](../ch8-errors-panic/README.md) 2.8절에서 패닉 훅이 프로세스 전체에 하나뿐이라서 `Send + Sync + 'static`이어야 한다고 했습니다. 훅은 **패닉한 스레드에서** 실행되므로, 훅 안에서 `thread::current().name()`을 부르면 패닉한 스레드의 이름을 얻습니다. 예제 02의 6번 구역은 이름을 붙인 스레드 `'계산기'`가 0으로 나누어 패닉하게 하고, 바꿔 둔 훅이 그 이름을 출력하며, `main`은 `join`의 `Err`에서 페이로드를 꺼냅니다. 패닉은 그 스레드만 멈추고, `join`의 `Err`는 [Part 1 ch8](../ch8-errors-panic/README.md)의 `catch_unwind`와 같은 `Box<dyn Any + Send>` 페이로드를 담습니다.

### 2.4 `Mutex` 더 쓰기

#### `try_lock`: 기다리지 않는 잠금

`lock()`은 잠금을 얻을 때까지 기다리고, `try_lock()`은 기다리지 않고 바로 결과를 돌려줍니다. 반환 타입 `TryLockResult<MutexGuard<T>>`는 세 가지 결과를 가집니다.

| 결과 | 뜻 |
| --- | --- |
| `Ok(guard)` | 잠금을 얻었습니다. |
| `Err(TryLockError::WouldBlock)` | 다른 누군가 잠그고 있어 지금은 얻을 수 없습니다. |
| `Err(TryLockError::Poisoned(e))` | 잠글 수는 있지만 포이즈닝된 상태입니다. `e.into_inner()`로 가드를 꺼낼 수 있습니다. |

이 챕터의 예제는 `try_lock`을 "지금 잠겨 있는가"를 관찰하는 도구로 씁니다. `lock()`으로 관찰하면 잠겨 있을 때 프로그램이 멈추기 때문입니다.

#### 가드가 살아 있는 범위 줄이기

`MutexGuard`가 살아 있는 동안 다른 스레드는 모두 기다립니다. 그러므로 **잠금 안에서는 공유 값을 읽고 고치는 일만 하고, 오래 걸리는 계산이나 입출력은 잠금 밖에서** 합니다. 흔한 방법은 필요한 것만 복제하고 가드를 바로 버리는 것입니다.

```rust
// (가) 가드를 쥔 채로 일함: 일하는 동안 다른 스레드가 모두 기다림
let guard = config.lock().unwrap();
let _work = guard.join(";");

// (나) 복사본을 만들고 가드는 이 문장 끝에서 버림
let snapshot = config.lock().unwrap().clone();
let _work = snapshot.join(";");
```

[`03_mutex_more.rs`](examples/03_mutex_more.rs)의 2번 구역은 두 경우에 다른 스레드의 `try_lock`이 각각 `Err(WouldBlock)`과 `Ok`가 되는 것을 보여 줍니다. 가드를 일찍 버리는 방법은 Part 0 ch15에서 본 대로 블록을 닫거나 `drop(guard)`를 부르는 것입니다. `let _ = m.lock().unwrap();`처럼 `_`에 담으면 가드가 그 자리에서 버려져 아무것도 잠그지 않는데, 이것은 거의 항상 실수라서 rustc가 `let_underscore_lock`으로 **컴파일을 거부합니다**(4.5절).

**`Barrier`.** 예제 03과 04는 스레드 사이의 진행 시점을 맞추려고 `std::sync::Barrier`를 씁니다. `Barrier::new(n)`을 만들고 각 스레드가 `wait()`를 부르면, `n`개 스레드가 모두 도착할 때까지 먼저 온 스레드가 기다렸다가 함께 풀려납니다. 한 번 풀린 뒤에도 다시 쓸 수 있습니다. 이 챕터에서는 "두 스레드가 모두 잠금을 쥔 순간"처럼 실행 순서에 따라 달라지는 상황을 매번 같은 순서로 재현하는 데 씁니다.

#### 교착 상태를 피하는 습관: 잠그는 순서 정하기

Part 0 ch15 2.10절의 교착 상태는 두 스레드가 잠금 두 개를 서로 다른 순서로 잡을 때 생깁니다. 예제 03의 3번 구역은 `Barrier`로 "스레드 1은 `a`를, 스레드 2는 `b`를 쥔" 순간을 만든 뒤 상대 잠금을 `try_lock`으로 시도합니다. 둘 다 `Err(WouldBlock)`입니다. `lock()`이었다면 두 스레드가 영원히 기다렸을 것입니다.

해결책은 **모든 스레드가 같은 순서로 잠그는 것**입니다. 계좌 이체처럼 어느 쪽이 먼저 올지 모르는 경우에는 id처럼 비교할 수 있는 값으로 순서를 정합니다.

```rust
fn transfer(from: &Account, to: &Account, amount: i64) {
    let (first, second) = if from.id < to.id { (from, to) } else { (to, from) };
    let mut first_balance = first.balance.lock().unwrap();
    let mut second_balance = second.balance.lock().unwrap();
    // ...
}
```

두 스레드가 반대 방향으로 이체를 만 번씩 해도 끝까지 실행되고 합계는 그대로입니다. 그 밖의 습관도 정리해 둡니다.

- 같은 스레드에서 같은 `Mutex`를 두 번 `lock()`하지 않습니다. 문서는 이때 "두 번째 호출이 돌아오지 않는다(패닉하거나 교착 상태가 될 수 있다)"고만 말합니다.
- 가드를 쥔 채로 다른 모듈의 함수나 사용자가 넘긴 클로저를 부르지 않습니다. 그 안에서 무엇을 잠글지 알 수 없습니다.
- 잠금을 쥔 채로 채널의 `recv`나 `Condvar`가 아닌 다른 기다림을 하지 않습니다.

#### 포이즈닝을 만났을 때의 선택지

가드를 쥔 스레드가 패닉하면 `Mutex`는 포이즈닝(poisoning)되고, 그 뒤의 `lock()`과 `try_lock()`은 `Err`를 돌려줍니다(Part 0 ch15). 패닉이 고치던 도중에 일어났다면 값의 불변식이 깨져 있을 수 있기 때문입니다. 예제 03의 4번 구역에서는 장부에 항목 50을 넣은 직후 패닉해서, 항목의 합(350)과 `total`(300)이 어긋난 상태가 남습니다.

| 선택 | 코드 | 언제 |
| --- | --- | --- |
| 패닉을 이어 간다 | `m.lock().unwrap()` | 깨진 상태를 다룰 방법이 없을 때. 지금까지 써 온 기본값 |
| 상태를 확인하고 계속 쓴다 | `m.lock().unwrap_or_else(PoisonError::into_inner)` 또는 `match`의 `Err(p) => p.into_inner()` | 값을 검사하거나 다시 계산해서 바로잡을 수 있을 때 |
| 복구했다고 표시한다 | 바로잡은 뒤 `m.clear_poison()` (1.77.0) | 다음 `lock()`부터 다시 `Ok`를 받게 하고 싶을 때 |
| 상태만 확인한다 | `m.is_poisoned()` | 기록이나 진단. 다른 스레드가 살아 있으면 곧바로 바뀔 수 있음 |

문서는 포이즈닝이 "권고(advisory)"일 뿐이라고 설명합니다. `into_inner`로 언제든 값에 닿을 수 있고, 모든 패닉을 잡아낸다는 보장도 없습니다(패닉 훅이나 `Drop` 안에서 잠근 경우 등). 그래서 `unsafe` 코드는 안전성을 포이즈닝에 기대면 안 됩니다. 포이즈닝과 예외 안전성의 세부는 Part 10 `ch8-unwinding`에서 다룹니다. 참고로 `OnceLock`은 포이즈닝되지 않고, `LazyLock`은 초기화 함수가 패닉하면 그 뒤의 모든 접근이 패닉합니다(2.9절).

#### 혼자 가지고 있으면 잠글 필요가 없습니다

| 메서드 | 받는 것 | 하는 일 |
| --- | --- | --- |
| `get_mut()` | `&mut Mutex<T>` | 실제로 잠그지 않고 `&mut T`를 줍니다(`LockResult`). `&mut`가 이미 "나만 접근한다"를 보장하기 때문입니다. |
| `into_inner()` | `Mutex<T>` (소비) | `Mutex`를 없애고 안의 값을 돌려줍니다(`LockResult`). |

`&mut Mutex`에 `lock()`을 부르면 Clippy가 `mut_mutex_lock`으로 `get_mut`을 쓰라고 알려 줍니다. 스레드에 `Arc`로 나눠 준 `Mutex`도 스레드가 모두 끝난 뒤에는 `Arc::into_inner(shared)`(1.70.0)로 `Mutex`를 되찾고 `into_inner()`로 값을 꺼낼 수 있습니다(예제 03의 5번 구역). `Arc::into_inner`는 마지막 강한 참조일 때만 `Some`입니다.

### 2.5 `RwLock`: 읽기는 여럿이, 쓰기는 혼자

**읽기-쓰기 잠금(reader-writer lock)** `RwLock<T>`는 "읽기 잠금 여러 개 **또는** 쓰기 잠금 하나"를 허락합니다. 대여 규칙(`&T` 여러 개 또는 `&mut T` 하나)을 실행 중에 지키는 장치라고 볼 수 있습니다. `Mutex`는 읽기와 쓰기를 구분하지 않아 읽기만 하는 스레드끼리도 서로 기다립니다.

| 메서드 | 돌려주는 가드 | 다른 스레드가 동시에 할 수 있는 것 |
| --- | --- | --- |
| `read()` / `try_read()` | `RwLockReadGuard` (`Deref`) | 다른 읽기 |
| `write()` / `try_write()` | `RwLockWriteGuard` (`Deref` + `DerefMut`) | 아무것도 |

[`04_rwlock.rs`](examples/04_rwlock.rs)의 1번 구역은 읽는 스레드 셋이 읽기 잠금을 쥔 채 `Barrier`(4명)에서 `main`과 만나게 합니다. 셋이 모두 `Barrier`를 통과했다는 것 자체가 "동시에 읽고 있었다"는 증거입니다. `Mutex`였다면 한 스레드만 잠금을 얻어 나머지가 `Barrier`에 도착하지 못하므로 프로그램이 멈췄을 것입니다. 그 순간 `main`의 `try_read`는 `Ok`, `try_write`는 `Err(WouldBlock)`입니다.

**언제 `Mutex`보다 나은가.** 읽기가 대부분이고 쓰기가 드물며, 읽는 동안 잠금을 쥐는 시간이 어느 정도 길 때입니다(설정, 캐시, 라우팅 표 등). 반대로 잠금을 아주 잠깐만 쥐거나 쓰기가 잦으면 `RwLock`의 관리 비용 때문에 `Mutex`가 더 빠를 수 있습니다. 확신이 없으면 `Mutex`로 시작합니다.

알아 둘 차이가 몇 가지 있습니다.

- **타입 조건**: 여러 스레드가 `&T`로 동시에 읽으므로 `RwLock<T>`가 `Sync`이려면 `T: Send + Sync`여야 합니다. `Mutex<T>`는 `T: Send`면 충분합니다(Part 0 ch15 2.9절의 표).
- **우선순위는 운영체제가 정함**: 문서는 쓰기를 기다리는 스레드가 있을 때 새 읽기를 막을지는 플랫폼마다 다르다고 적습니다. 그래서 같은 스레드에서 읽기 잠금을 쥔 채 다시 `read()`하면, 그 사이 다른 스레드가 `write()`를 기다리는 경우 교착 상태가 될 수 있습니다.
- **포이즈닝**: 쓰기 잠금을 쥔 채 패닉했을 때만 포이즈닝됩니다. 읽기 중의 패닉은 값을 바꾸지 못했으므로 포이즈닝하지 않습니다.
- **쓰기 잠금으로 읽기만**: `write()`로 얻은 가드를 읽기에만 쓰면 Clippy가 `readonly_write_lock`으로 알려 줍니다.

#### `RwLock<Arc<T>>`: 스냅숏을 들고 가기

읽는 쪽이 오래 걸리는 일을 한다면 읽기 잠금도 오래 쥐게 되고, 그동안 쓰는 쪽이 기다립니다. 이때 값을 `Arc`로 감싸 두면 읽는 쪽은 **`Arc`를 복제하는 순간에만** 잠금을 쥐고, 복제한 스냅숏으로 일합니다.

```rust
let config = RwLock::new(Arc::new(Config { version: 1, greeting: "안녕".to_string() }));

// 읽는 쪽: 읽기 잠금은 이 문장 안에서만
let snapshot = Arc::clone(&config.read().unwrap());

// 쓰는 쪽: 스냅숏을 가진 쪽이 있으면 make_mut이 복제해 갈라선 뒤 고침
let mut current = config.write().unwrap();
let cfg = Arc::make_mut(&mut current);
cfg.version = 2;
```

예제 04의 3번 구역에서 읽는 스레드 둘이 스냅숏을 든 채로 `main`이 설정을 바꿉니다. 읽기 잠금이 없으므로 `write()`는 바로 성공하고, `Arc::make_mut`은 다른 소유자가 있으므로 안의 값을 복제해 새 할당으로 갈라섭니다(`Arc::ptr_eq`가 `false`). 읽는 스레드는 끝까지 `v1 안녕`을 보고, 새로 읽는 쪽은 `v2 반가워요`를 봅니다. 읽는 도중에 값이 바뀌지 않는다는 점도 장점입니다.

여기서 쓴 `Arc::make_mut`, `Arc::ptr_eq`, 2.4절의 `Arc::into_inner`처럼 **`Arc`에는 [Part 1 ch7](../ch7-smart-pointers-memory/README.md) 2.8절에서 본 `Rc`의 함수가 같은 이름과 같은 의미로 있습니다.** `get_mut`, `try_unwrap`, `unwrap_or_clone`, `strong_count`, `downgrade`(→ `std::sync::Weak`), 자기 자신을 가리키는 `Weak`를 만드는 `new_cyclic`(1.60.0)도 마찬가지입니다. 차이는 카운트를 원자적으로 고친다는 것뿐입니다. 이 툴체인에서 새로 안정화된 `Arc::map`(1.100.0)은 안의 값으로 새 값을 만들어 `Arc`에 담되, 가능하면 같은 할당을 재사용합니다.

### 2.6 `Condvar`: 조건이 만족될 때까지 기다리기

"큐에 일감이 들어올 때까지", "준비가 끝날 때까지"처럼 **어떤 조건이 참이 될 때까지** 기다려야 할 때가 있습니다. 반복문에서 `lock()`하고 확인하고 풀기를 되풀이하면(바쁜 대기, busy waiting) 기다리는 동안에도 CPU를 계속 씁니다. **조건 변수(condition variable)** `Condvar`는 스레드를 CPU를 쓰지 않는 상태로 재워 두었다가, 조건을 바꾼 쪽이 깨워 주게 하는 도구입니다.

`Condvar`는 언제나 `Mutex`와 짝을 이룹니다. 조건(예: "큐가 비어 있다")은 `Mutex` 안의 값이고, `Condvar`는 "그 값이 바뀌었을지 모른다"는 알림만 전합니다.

```rust
let ready = Mutex::new(false);
let signal = Condvar::new();

// 기다리는 쪽
let guard = ready.lock().unwrap();
let guard = signal.wait_while(guard, |ready| !*ready).unwrap(); // 조건이 참인 동안 잠듦

// 깨우는 쪽
*ready.lock().unwrap() = true; // 조건을 먼저 바꾸고
signal.notify_all();           // 기다리는 스레드를 깨움
```

**`wait`가 가드를 받아서 돌려주는 이유.** `wait(guard)`는 세 가지 일을 합니다. 잠금을 풀고, 잠들고, 깨어나면 다시 잠근 뒤 가드를 돌려줍니다. 잠든 동안 잠금을 쥐고 있으면 조건을 바꿔 줄 다른 스레드가 영원히 잠금을 얻지 못하므로 풀어야 합니다. 그런데 "풀기"와 "잠들기" 사이에 틈이 있으면, 그 틈에 다른 스레드가 조건을 바꾸고 알림을 보낸 뒤 이쪽이 잠들어 알림을 놓칠 수 있습니다. 그래서 `wait`는 두 일을 **원자적으로** 합니다. 가드를 값으로 받는 시그니처는 "잠금을 쥔 상태에서만 부를 수 있다"를 타입으로 강제하고, 돌려받은 가드는 "깨어났을 때 다시 잠긴 상태"를 나타냅니다.

**조건은 항상 다시 확인합니다.** 이유는 두 가지입니다.

1. **가짜 깨어남(spurious wakeup)**: 문서는 `wait`가 아무도 알리지 않았는데도 돌아올 수 있다고 적고 있습니다. 운영체제 수준의 사정으로 생기는 일입니다.
2. **알림은 쌓이지 않습니다**: 문서에 따르면 `notify_one`은 "어떤 식으로도 버퍼링되지 않습니다." 아무도 기다리지 않을 때 보낸 알림은 그냥 사라집니다.

[`05_condvar.rs`](examples/05_condvar.rs)의 2번 구역이 두 번째 이유를 보여 줍니다. 조건을 `true`로 바꾸고 `notify_one()`을 먼저 부른 뒤, (가) 조건 없이 `wait_timeout`으로 기다리면 이미 지나간 알림을 기다리다 시간이 다 됩니다(`timed_out: true`). (나) `wait_timeout_while`로 조건을 보며 기다리면 조건이 이미 거짓이라 기다리지 않고 바로 돌아옵니다(`timed_out: false`). 그래서 `wait`를 직접 쓸 때는 `if`가 아니라 `while`로 감싸고, 대부분은 이 반복을 대신해 주는 `wait_while`(1.42.0)을 씁니다.

```rust
// wait를 직접 쓸 때의 모양
while !*started {
    started = cvar.wait(started).unwrap();
}
```

| 메서드 | 기다리는 동안 | 돌아올 때 |
| --- | --- | --- |
| `wait(guard)` | 알림이 올 때까지 (가짜 깨어남 있음) | `LockResult<MutexGuard>` |
| `wait_while(guard, 조건)` | 조건이 `true`인 동안 | 조건이 `false`가 되었을 때 |
| `wait_timeout(guard, dur)` | 알림이 오거나 `dur`가 지날 때까지 | `(가드, WaitTimeoutResult)`. `timed_out()`으로 시간 초과 여부 확인 |
| `wait_timeout_while(guard, dur, 조건)` | 조건이 `true`인 동안, 최대 `dur` | 위와 같음. 시간 초과면 조건은 여전히 `true` |
| `notify_one()` | — | 기다리는 스레드 하나를 깨움 |
| `notify_all()` | — | 기다리는 스레드를 모두 깨움 |

`notify_one`과 `notify_all`은 깨워야 할 상대가 몇인지로 고릅니다. 출발 신호처럼 모두가 진행해야 하면 `notify_all`, 큐에 일감 하나가 들어왔다면 그것을 가져갈 소비자 하나만 깨우면 되므로 `notify_one`입니다. 시간 제한이 있는 대기에 쓰는 `Duration`은 [ch10](../ch10-env-process-time/README.md)에서 다뤘습니다.

**`Condvar` 하나에는 `Mutex` 하나.** 문서는 한 `Condvar`를 시간을 두고 여러 `Mutex`와 쓰면 패닉할 수 있다고 경고합니다. 이 챕터의 예제를 작성하는 동안 실제로 만났는데, 앞에서 쓴 `Condvar`를 다른 `Mutex`와 함께 쓰자 이 환경(macOS)에서 `attempted to use a condition variable with two mutexes`로 패닉했습니다. 그래서 `Condvar`는 보통 지키는 `Mutex`와 같은 구조체에 나란히 둡니다.

#### 용량이 정해진 큐: 생산자-소비자

`Mutex<VecDeque<T>>`와 `Condvar` 두 개로 용량이 정해진 큐를 만들 수 있습니다. 비어 있으면 꺼내는 쪽(소비자)이, 가득 차면 넣는 쪽(생산자)이 기다립니다.

```rust
struct BoundedQueue<T> {
    items: Mutex<VecDeque<T>>,
    not_empty: Condvar, // 소비자가 기다리는 조건
    not_full: Condvar,  // 생산자가 기다리는 조건
    capacity: usize,
}

fn push(&self, item: T) { // 예제의 push는 넣은 뒤의 길이도 돌려줍니다(핵심만 옮김).
    let guard = self.items.lock().unwrap();
    let mut items = self.not_full.wait_while(guard, |items| items.len() == self.capacity).unwrap();
    items.push_back(item);
    drop(items);                 // 깨울 상대가 바로 잠글 수 있게 먼저 놓고
    self.not_empty.notify_one(); // 소비자 하나를 깨움
}
// pop은 not_empty에서 기다리고, 꺼낸 뒤 not_full로 생산자 하나를 깨웁니다.
```

생산자가 소비자보다 빠르면 큐가 차고, 그때부터 생산자는 소비자가 하나를 꺼낼 때까지 기다립니다. 이렇게 받는 쪽의 속도가 보내는 쪽을 늦추는 것을 **배압(backpressure)** 이라고 합니다. 용량 제한이 없으면 느린 소비자 앞에 일감이 끝없이 쌓여 메모리가 바닥날 수 있습니다. 예제 05의 3번 구역에서 생산자 둘이 10개씩 넣는 동안 큐 길이는 한 번도 3을 넘지 않습니다. 같은 일을 표준 라이브러리가 이미 만들어 둔 것이 다음 절의 `sync_channel`입니다.

### 2.7 채널 더 쓰기: `sync_channel`과 배압

`mpsc`에는 두 종류의 채널이 있습니다.

| 만드는 함수 | 송신자 | 버퍼 | `send`가 기다리는가 |
| --- | --- | --- | --- |
| `channel()` | `Sender<T>` | 제한 없음 | 기다리지 않음 |
| `sync_channel(n)` | `SyncSender<T>` | 미리 정한 `n`칸 | 칸이 모두 찼으면 빈칸이 생길 때까지 기다림 |

**용량이 정해진 채널(bounded channel)** `sync_channel(n)`은 2.6절의 큐와 같은 배압을 줍니다. [`06_sync_channel.rs`](examples/06_sync_channel.rs)의 2번 구역은 생산자가 10개를 보내는 동안 `main`이 50ms 동안 아무것도 받지 않게 합니다. `channel()`이면 그동안 10개를 모두 보내고, `sync_channel(3)`이면 3개를 보낸 뒤 네 번째 `send`에서 멈춰 있습니다. `main`이 받기 시작하면 생산자도 다시 움직여 결국 10개가 모두 도착합니다.

`n = 0`이면 버퍼가 없는 **랑데부(rendezvous) 채널**이 됩니다. `send`는 받는 쪽이 그 값을 실제로 가져갈 때까지 끝나지 않으므로, 보내는 쪽과 받는 쪽이 만나는 시점을 맞추는 데 씁니다(예제 06의 3번 구역에서 50ms 동안 보낸 개수 0).

`SyncSender`에는 기다리지 않는 `try_send`도 있습니다.

| `try_send` 결과 | 뜻 |
| --- | --- |
| `Ok(())` | 버퍼에 넣었습니다. |
| `Err(TrySendError::Full(v))` | 버퍼가 가득 찼습니다(랑데부 채널이면 기다리는 수신자가 없음). `v`는 보내려던 값입니다. |
| `Err(TrySendError::Disconnected(v))` | 수신자가 사라졌습니다. |

받는 쪽 메서드를 모으면 다음과 같습니다. Part 0 ch15에서 `recv`, `try_recv`, `recv_timeout`, `for` 문(`into_iter`)을 봤으므로 새로운 것은 `try_iter`입니다.

| 메서드 | 기다리는가 | 끝나는 때 |
| --- | --- | --- |
| `recv()` | 메시지가 올 때까지 | 송신자가 모두 사라지고 남은 메시지도 없으면 `Err(RecvError)` |
| `recv_timeout(dur)` (1.12.0) | 최대 `dur` | `Err(Timeout)` 또는 `Err(Disconnected)` |
| `iter()` / `for m in rx` | 메시지마다 기다림 | 송신자가 모두 사라지면 반복 끝 |
| `try_iter()` (1.15.0) | 기다리지 않음 | 지금 쌓인 메시지를 다 꺼내면 끝(송신자가 살아 있어도) |

`try_iter`는 "지금까지 온 것만 한꺼번에 처리하고 다른 일로 돌아가는" 반복에 맞습니다(예제 06의 4번 구역).

**`Sender`는 `Sync`, `Receiver`는 `Sync`가 아닙니다.** `Sender<T>`는 `T: Send`이면 `Sync`라서, 스코프 스레드들이 `clone` 없이 `&tx`를 함께 쓸 수 있습니다(예제 06의 5번 구역). 이때도 채널을 닫으려면 스코프가 끝난 뒤 원래의 `tx`를 `drop`해야 합니다. 반면 `Receiver`는 `mpsc`(single consumer)라는 이름대로 소비자가 하나라는 전제로 만들어져 `Sync`가 아닙니다. 두 스레드가 `&rx`를 함께 쓰려 하면 E0277입니다(4.3절). 작업자 여럿이 일감을 나눠 받아야 한다면 [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md)의 스레드 풀처럼 `Arc<Mutex<Receiver<T>>>`로 감쌉니다. 소비자가 여럿인 `std::sync::mpmc`는 이 툴체인에서 아직 nightly 전용입니다.

**채널이냐, 공유 상태냐.** 둘 다 쓸 수 있는 상황이 많습니다. 다음 기준으로 고릅니다.

| 이럴 때 | 고르는 것 |
| --- | --- |
| 값의 주인이 단계마다 바뀜(생산 → 가공 → 저장) | 채널. 값이 한 번에 한 스레드에만 있으므로 잠금이 필요 없음 |
| 일감을 작업자에게 나눠 주고 결과를 모음 | 채널 (일감용, 결과용) |
| 여러 스레드가 같은 값을 계속 보고 고침(카운터, 캐시, 설정) | 공유 상태(`Mutex`, `RwLock`, 원자 타입) |
| 받는 쪽이 느릴 때 보내는 쪽을 늦춰야 함 | `sync_channel` (또는 2.6절의 큐) |
| "어떤 상태가 될 때까지" 기다림 | `Condvar` (상태가 공유 값일 때), 채널 (사건을 메시지로 보낼 수 있을 때) |

### 2.8 원자 타입: 잠금 없이 값 하나 고치기

**원자 타입(atomic type)** 은 정수와 `bool` 같은 단순한 값을 여러 스레드가 `&`로 함께 고칠 수 있게 하는 타입입니다. `AtomicBool`, `AtomicI8`~`AtomicI64`, `AtomicU8`~`AtomicU64`, `AtomicIsize`, `AtomicUsize`, `AtomicPtr`가 있습니다(128비트는 nightly 전용). CPU가 제공하는 원자적 명령으로 구현되어 `Mutex`처럼 운영체제에 잠금을 요청하지 않습니다. 원자 타입은 `Sync`이므로 여러 스레드가 `&AtomicUsize`를 함께 쓸 수 있고, 스레드에 나눠 줄 때는 `Arc<AtomicUsize>`나 스코프 스레드의 참조를 씁니다. Part 0 ch15에서 `Arc`의 참조 카운트가 "원자적 연산"으로 고쳐진다고 했는데, 그 연산이 바로 이것입니다.

참고로 1.100.0 문서에서 `AtomicUsize`는 `pub type AtomicUsize = Atomic<usize>;`처럼 아직 nightly 전용인 제네릭 타입 `Atomic<T>`의 별칭으로 보입니다. 쓰는 방법은 똑같고, 에러나 경고 메시지에 `Atomic::<usize>` 같은 이름이 나올 수 있다는 점만 알아 두면 됩니다.

주요 메서드는 다음과 같습니다(모두 `&self`를 받고, 메모리 순서(memory ordering) `Ordering`을 인자로 받습니다).

| 메서드 | 하는 일 | 돌려주는 것 |
| --- | --- | --- |
| `new(v)` | 만들기. `const fn`이라 `static`에 둘 수 있음 | — |
| `load(o)` / `store(v, o)` | 읽기 / 쓰기 | 값 / `()` |
| `swap(v, o)` | 바꿔 넣기 | 이전 값 |
| `fetch_add(v, o)`, `fetch_sub`, `fetch_max`, `fetch_min`, `fetch_and`, `fetch_or`, `fetch_xor` | 읽고-계산하고-쓰기를 한 번에 | 이전 값. `fetch_add`는 오버플로하면 래핑 |
| `compare_exchange(현재, 새 값, 성공 o, 실패 o)` | 지금 값이 `현재`와 같을 때만 `새 값`으로 바꿈 | `Ok(이전 값)` 또는 `Err(지금 값)` |
| `try_update(o, o, f)` (1.95.0) | `f(지금 값)`이 `Some(새 값)`이면 바꿈. 경쟁하면 `f`를 다시 부름 | `Ok(이전 값)` 또는 `Err(지금 값)` |
| `update(o, o, f)` (1.95.0) | `f(지금 값)`으로 무조건 바꿈 | 이전 값 |
| `get_mut()`, `into_inner()` | `&mut`나 소유권이 있을 때 그냥 읽고 쓰기 | `&mut T`, `T` |

**`load` 후 `store`가 틀린 이유.** `load`와 `store`는 각각은 원자적이지만, 둘 사이에 다른 스레드가 끼어들 수 있습니다. Part 0 ch15 2.10절에서 잠금을 둘로 나눴을 때와 똑같은 경쟁 조건입니다. [`07_atomics.rs`](examples/07_atomics.rs)의 1번 구역에서 스레드 4개가 10만 번씩 더하면 `fetch_add`는 항상 400000이지만, `load` 후 `store`는 이 환경에서 대략 106000~109000이 나왔습니다(실행마다 다름). **읽고 고쳐서 쓰는 일은 `fetch_*`, `compare_exchange`, `try_update`/`update` 같은 메서드 하나로** 합니다.

**비교 후 교환(compare-and-swap).** `compare_exchange(0, id, ..)`는 "지금 값이 0일 때만 `id`로 바꾼다"를 한 번에 합니다. 여러 스레드가 동시에 시도해도 성공하는 것은 하나뿐이므로, 예제 07의 2번 구역처럼 "선착순 한 명"을 정할 때 씁니다. 더 복잡한 규칙은 `try_update`가 편합니다. 안에서 `compare_exchange`를 반복하며, 클로저가 `None`을 돌려주면 바꾸지 않습니다.

```rust
// 재고가 0이면 더 빼지 않습니다(checked_sub가 None).
stock.try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
```

예제 07의 3번 구역에서 재고 5개에 8개 스레드가 동시에 주문하면 정확히 5개만 성공합니다. 예전 이름 `fetch_update`(1.45.0)는 1.99.0부터 "일관성을 위해 `try_update`로 이름을 바꿨다"는 이유로 쓰지 않는 메서드(deprecated)가 되어, 이 툴체인에서 쓰면 `deprecated` 경고가 납니다. 다른 문서나 예전 코드에서 `fetch_update`를 보면 `try_update`로 읽으면 됩니다.

**메모리 순서(`Ordering`).** 원자 연산은 모두 `Ordering`을 받습니다. 이것은 그 연산 **주변의 다른 메모리 접근**이 다른 스레드에게 어떤 순서로 보이는지를 정합니다. 컴파일러와 CPU는 성능을 위해 메모리 접근 순서를 바꾸기 때문에 이런 지정이 필요합니다. 이 챕터에서는 실용 지침만 정합니다.

- **확신이 없으면 `SeqCst`**: 가장 강한 순서로, 모든 스레드가 원자 연산들의 순서 하나에 동의합니다. 조금 느릴 수 있지만 틀리지 않습니다.
- **단순한 카운터나 ID 발급은 `Relaxed`**: 그 값 자체만 정확하면 되고, 그 값으로 다른 데이터의 준비 여부를 알리지 않을 때입니다(예제 07의 1·5번 구역).
- **원자 변수로 "다른 데이터가 준비됐다"를 알릴 때**(플래그를 보고 다른 메모리를 읽음)는 `Release`/`Acquire` 짝이 필요합니다. 이것을 직접 설계하기보다 `Mutex`, 채널, `OnceLock`을 쓰는 것이 안전합니다.

`load`에 `Release`처럼 그 연산에 맞지 않는 순서를 상수로 주면 rustc의 `invalid_atomic_ordering`(기본 deny)이 컴파일을 거부합니다(4.5절). 메모리 순서의 정확한 의미와 `Acquire`/`Release`로 동기화를 만드는 방법, 스핀락 만들기는 Part 10 `ch9-concurrency`에서 다룹니다.

**원자 타입이 맞지 않는 곳.** 원자 타입은 **값 하나**만 원자적으로 다룹니다. "잔액 두 개의 합이 일정하다"처럼 여러 값에 걸친 불변식이 있다면 원자 변수 여러 개로는 지킬 수 없으므로 `Mutex` 하나로 묶습니다. 반대로 `Mutex<bool>`이나 `Mutex<usize>`는 원자 타입으로 바꿀 수 있는 경우가 많지만, Clippy의 `mutex_atomic`/`mutex_integer`는 기본으로 꺼져 있는(restriction 그룹) 검사라서 경고하지 않습니다. 직접 판단해야 합니다.

**`AtomicBool` 멈춤 신호와 `static` 원자 변수.** 예제 07의 4번 구역은 작업자가 반복할 때마다 `stop.load(Relaxed)`를 확인하고, `main`이 `stop.store(true, Relaxed)`로 멈추게 합니다. 5번 구역은 [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.5절의 `static mut COUNTER`를 대신하는 모양입니다.

```rust
static NEXT_ID: AtomicU64 = AtomicU64::new(1); // unsafe 없이 어디서든 고칠 수 있습니다.

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed) // 더하기 전의 값이 이번 ID
}
```

`static`이 아니라 `const`로 선언하면 쓰는 곳마다 새 값이 만들어져 `fetch_add`가 아무 효과가 없습니다. rustc의 `const_item_interior_mutations`와 Clippy의 `declare_interior_mutable_const`가 경고합니다(ch7 4.6절의 `const Cell`과 같은 함정).

### 2.9 한 번만 초기화: `OnceLock`과 `LazyLock`

[Part 1 ch7](../ch7-smart-pointers-memory/README.md) 4.4절에서 `static CONFIG: OnceCell<String>`은 E0277이었고, 컴파일러는 `note: ... use std::sync::OnceLock instead`라고 제안했습니다. `OnceCell`/`LazyCell`은 `Sync`가 아니라서 `static`에 둘 수 없기 때문입니다. 그 스레드 안전한 짝이 `OnceLock`(1.70.0)과 `LazyLock`(1.80.0)입니다. 메서드 이름과 의미는 ch7의 짝과 같고, 여러 스레드가 동시에 처음 접근해도 **초기화는 정확히 한 번** 일어나며 나머지 스레드는 초기화가 끝날 때까지 기다립니다. 이렇게 처음 필요할 때 값을 만드는 것을 **지연 초기화(lazy initialization)** 라고 합니다.

| `OnceLock<T>` 메서드 | 하는 일 |
| --- | --- |
| `get()` | 채워졌으면 `Some(&T)` |
| `set(v)` | 비어 있으면 채우고 `Ok(())`, 이미 찼으면 `Err(v)`로 돌려줌 |
| `get_or_init(f)` | 비어 있으면 `f()`로 채운 뒤 `&T`. 동시에 불려도 `f`는 한 번만 실행 |
| `wait()` (1.86.0) | 다른 스레드가 채울 때까지 기다린 뒤 `&T` |
| `get_mut()`, `take()`, `into_inner()` | `&mut`나 소유권이 있을 때 |

```rust
static CONFIG: OnceLock<Config> = OnceLock::new();

CONFIG.set(Config { name: "주문 서버".to_string(), workers: 4 }); // 프로그램 시작 때 한 번
let c: &'static Config = CONFIG.get().unwrap();                  // 어디서든 읽기
```

`OnceLock`은 **실행 중에야 알 수 있는 값**(명령줄 인자나 설정 파일에서 읽은 값)을 한 번 정해 두고 계속 읽는 전역에 맞습니다. `static`에 든 값의 참조라서 `&'static Config`를 얻는다는 점도 편리합니다. 스레드에 넘길 때 `Arc`가 필요 없습니다. 구조체 필드에 두면 ch7의 `OnceCell` 캐시를 여러 스레드가 함께 쓰는 판이 됩니다. [`08_once_lock_lazy_lock.rs`](examples/08_once_lock_lazy_lock.rs)의 2번 구역에서 8개 스레드가 동시에 `summary()`를 불러도 요약은 한 번만 계산되고, 모두 같은 `String`을 가리키는 `&str`을 받습니다.

**`LazyLock<T, F>`** 는 초기화 함수를 만들 때 함께 넘겨 두는 판입니다. 처음 역참조할 때(메서드 호출, 인덱싱, `*`) 그 함수가 불립니다.

```rust
static UNIT_PRICES: LazyLock<HashMap<&'static str, u32>> =
    LazyLock::new(|| HashMap::from([("커피", 4500), ("차", 4000), ("빵", 3000)]));

let price = UNIT_PRICES["커피"]; // 이 줄에서 처음이면 초기화
```

`LazyLock::get(&UNIT_PRICES)`(1.94.0)는 초기화를 일으키지 않고 들여다보기만 하며, `LazyLock::force(&x)`는 지금 바로 초기화합니다. 문서는 초기화 함수가 패닉하면 `LazyLock`이 포이즈닝되고, 그 뒤 어느 스레드에서 접근하든 패닉하며 `Mutex`와 달리 복구할 방법이 없다고 설명합니다.

`OnceLock`과 `LazyLock`의 선택은 ch7과 같습니다. 만드는 방법이 하나로 정해져 있으면 `LazyLock`, 상황에 따라 다른 값으로 채우거나(`set`) 채우는 시점을 직접 정해야 하면 `OnceLock`입니다.

#### `static`에 둘 수 있는 것과 없는 것

`static` 변수에는 두 가지 조건이 있습니다.

1. **타입이 `Sync`여야 합니다.** 어느 스레드에서든 `&`로 접근할 수 있기 때문입니다(ch7의 E0277).
2. **초기값이 컴파일할 때 계산되어야 합니다.** 그래서 초기값 안에서는 `const fn`만 부를 수 있습니다(4.4절의 E0015).

| 초기값 | 가능한가 | 이유 |
| --- | --- | --- |
| `AtomicU64::new(1)`, `AtomicBool::new(false)` | 가능 | `const fn`, `Sync` |
| `Mutex::new(Vec::new())`, `RwLock::new(String::new())`, `Condvar::new()` | 가능 | 모두 `const fn`(1.63.0부터) |
| `OnceLock::new()`, `LazyLock::new(\|\| ...)` | 가능 | `const fn`. 실제 값은 실행 중에 만듦 |
| `Mutex::new(HashMap::new())` | 불가(E0015) | `HashMap::new`는 `const fn`이 아님 → `LazyLock::new(\|\| Mutex::new(HashMap::new()))` |
| `Mutex::new(vec![String::from("관리자")])` | 불가(E0015 등) | 힙 할당과 `String::from`은 컴파일할 때 실행할 수 없음 → `LazyLock` |
| `OnceCell::new()`, `Cell::new(0)`, `RefCell::new(..)` | 불가(E0277) | `Sync`가 아님 → `OnceLock`, 원자 타입, `Mutex` |

예제 08의 `ORDER_LOG`(`Mutex<Vec<String>>`)는 초기값이 `const`로 만들어지므로 `LazyLock` 없이 `static`에 바로 둡니다. 반면 `UNIT_PRICES`는 `HashMap`이라 `LazyLock`이 필요합니다. 이 저장소에서 `static mut`를 쓸 일은 없습니다. 여기 나온 도구들이 같은 일을 `unsafe` 없이 해 주기 때문입니다. `static mut`의 참조를 금지한 2024 에디션의 규칙은 [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md)에서 다룹니다.

### 2.10 이럴 때 이 도구

| 상황 | 도구 | 절 |
| --- | --- | --- |
| 일을 나눠 계산하고 결과를 모음. 지역 데이터를 빌려 씀 | `thread::scope` | 2.2 |
| 프로그램 내내 사는 작업자, 끝나기를 기다리지 않을 수도 있음 | `thread::spawn` / `Builder::spawn` | 2.3 |
| 스레드에 이름을 붙이거나 생성 실패를 처리 | `thread::Builder` | 2.3 |
| 스레드마다 따로 쓰는 전역 값 | `thread_local!` | 2.3 |
| 여러 스레드가 값을 함께 읽고 고침 | `Mutex<T>` (공유는 `Arc` 또는 스코프 스레드) | 2.4 |
| 읽기가 대부분이고 쓰기가 드묾 | `RwLock<T>`, 오래 읽으면 `RwLock<Arc<T>>` | 2.5 |
| 어떤 상태가 될 때까지 잠들어 기다림 | `Mutex` + `Condvar`의 `wait_while` | 2.6 |
| 모든 스레드가 같은 지점에 도착하기를 기다림 | `Barrier` | 2.4 |
| 값의 소유권을 넘기며 일을 전달 | `mpsc::channel` | 2.7 |
| 받는 쪽이 느릴 때 보내는 쪽을 늦춤 | `mpsc::sync_channel(n)` | 2.7 |
| 카운터, 플래그, ID 발급처럼 값 하나 | 원자 타입 | 2.8 |
| 실행 중에 정하는 전역 설정, 한 번 계산하는 캐시 | `OnceLock` | 2.9 |
| 만드는 방법이 정해진 전역 값(`HashMap` 등) | `LazyLock` | 2.9 |

ch7의 표를 이어서 단일 스레드 도구와 스레드 안전한 짝을 다시 정리합니다. 왼쪽 도구로 충분하다면(값이 스레드를 넘나들지 않는다면) 왼쪽이 더 빠릅니다. 잘못 고르면 컴파일러가 `Send`/`Sync` 에러로 알려 줍니다.

| 단일 스레드 | 스레드 안전한 짝 | 이 챕터의 예제 |
| --- | --- | --- |
| `Rc<T>` | `Arc<T>` | 03, 04 |
| `Cell<u32>`, `Cell<bool>` | `AtomicU32`, `AtomicBool` | 07 |
| `RefCell<T>` | `Mutex<T>`, `RwLock<T>` | 03, 04 |
| `OnceCell<T>` | `OnceLock<T>` | 08 |
| `LazyCell<T, F>` | `LazyLock<T, F>` | 08 |

**표준 라이브러리에 없는 것.** 컬렉션을 자동으로 여러 스레드에 나눠 처리하는 병렬 반복자, 놀고 있는 스레드가 다른 스레드의 일감을 가져가는 작업 훔치기(work stealing) 스케줄러, 소비자가 여럿인 채널(`mpmc`, nightly 전용), 범용 스레드 풀은 표준 라이브러리에 없습니다. 필요하면 이 챕터의 도구로 직접 만들거나(Part 0 ch20의 스레드 풀처럼) crates.io의 크레이트를 씁니다.

### 2.11 이 챕터에서 다루지 않는 것

| 주제 | 한 줄 소개 | 다루는 곳 |
| --- | --- | --- |
| 메모리 순서의 정확한 의미, `Acquire`/`Release`로 동기화 만들기, 스핀락 | 원자 연산 주변의 메모리 접근이 다른 스레드에 보이는 순서 | Part 10 `ch9-concurrency` |
| `Send`/`Sync` 직접 구현 | `unsafe impl`로 컴파일러 대신 스레드 안전성을 보증 | Part 10 `ch9-concurrency` |
| 포이즈닝과 예외 안전성 | 패닉이 남긴 반쯤 바뀐 상태를 다루는 설계 | Part 10 `ch8-unwinding` |
| `park`/`unpark`, `Waker`로 실행기 만들기 | 비동기 태스크를 깨우는 원리 | [ch13](../ch13-async-primitives/README.md) |
| 소켓 서버에서 연결마다 스레드 쓰기 | TCP 리스너와 스레드 | [ch12](../ch12-net/README.md) |
| `static mut` 참조 금지 | 2024 에디션의 `static_mut_refs` | [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_scoped_threads.rs`](examples/01_scoped_threads.rs) | 빌려 읽기, 겹치지 않는 가변 대여, 자동 join, 스코프의 반환값, 스코프 안의 패닉 | 1 |
| [`02_thread_builder.rs`](examples/02_thread_builder.rs) | `Builder`의 이름·스택 크기·실패 처리, `ThreadId`, `available_parallelism`, `thread_local!`, 패닉 훅과 `join` | 1 |
| [`03_mutex_more.rs`](examples/03_mutex_more.rs) | `try_lock`, 가드 범위, 잠그는 순서, 포이즈닝 복구, `get_mut`/`into_inner`/`Arc::into_inner` | 2 |
| [`04_rwlock.rs`](examples/04_rwlock.rs) | 동시에 쥔 읽기 잠금, 혼자 쥐는 쓰기 잠금, `RwLock<Arc<T>>`와 `Arc::make_mut` | 2 |
| [`05_condvar.rs`](examples/05_condvar.rs) | `notify_all` 출발 신호, 쌓이지 않는 알림과 `wait_timeout_while`, 용량이 정해진 큐의 생산자-소비자 | 3 |
| [`06_sync_channel.rs`](examples/06_sync_channel.rs) | `try_send`와 `Full`, 배압, 랑데부 채널, `try_iter`, `&Sender` 공유 | 3 |
| [`07_atomics.rs`](examples/07_atomics.rs) | `fetch_add`와 `load` 후 `store`, `compare_exchange`, `try_update`, `AtomicBool`, `static` 원자 변수 | 4 |
| [`08_once_lock_lazy_lock.rs`](examples/08_once_lock_lazy_lock.rs) | `OnceLock` 전역 설정과 `wait`, 필드에 둔 `OnceLock`, `LazyLock` `static` | 5 |
| [`compile_fail/`](compile_fail) | 스코프 스레드·가드·수신자·`static` 초기값에서 만나는 컴파일 에러 4개(4절) | 1, 2, 3, 5 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 끝나고 몇 초 안에 끝납니다. 스레드 안에서 출력하지 않고 결과를 돌려받아 `main`에서 출력하거나, 순서와 무관한 합계·개수만 출력하므로 실행할 때마다 같은 출력이 나옵니다. 예외는 07의 경쟁 조건 줄과 02의 `available_parallelism` 값입니다. 01과 03은 일부러 패닉하는 스레드가 있어 표준 에러에 기본 패닉 훅의 메시지가 나옵니다. 그 메시지의 스레드 ID는 실행할 때마다 달라지며, 예제 파일의 `// 출력:` 주석은 표준 출력만 적었습니다.

### 01_scoped_threads — 스코프 스레드

```sh
cargo run -p p1-ch11-threads-sync --example 01_scoped_threads
```

```text
1. 스코프 스레드는 지역 변수를 move 없이 빌려 읽습니다
    2글자보다 긴 단어 2개, 글자 수 합계 10
    스코프가 끝난 뒤에도 words를 씁니다: ["사과", "바나나", "체리", "두리안"]
2. 겹치지 않는 조각이면 여러 스레드가 동시에 가변으로 빌릴 수 있습니다
    [10, 20, 30, 40, 50, 60, 70, 80]
3. join하지 않은 스레드도 스코프가 끝날 때 자동으로 기다립니다
    조각별 합: [325, 950, 1575, 2200], 전체 합: 5050
4. scope는 클로저의 반환값을 그대로 돌려줍니다
    가장 짧은 단어 Some(1)글자, 가장 긴 단어 Some(4)글자
5. 스코프 안의 패닉

thread '<unnamed>' (7380740) panicked at part1-standard-lib/ch11-threads-sync/examples/01_scoped_threads.rs:98:37:
attempt to divide by zero
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    (가) join이 Err를 돌려줬는가: true

thread '<unnamed>' (7380741) panicked at part1-standard-lib/ch11-threads-sync/examples/01_scoped_threads.rs:109:28:
attempt to divide by zero

thread 'main' (7380724) panicked at part1-standard-lib/ch11-threads-sync/examples/01_scoped_threads.rs:108:13:
a scoped thread panicked
    (나) scope가 패닉했는가: true, 페이로드: a scoped thread panicked
```

- **3번 구역**: 1~100을 25개씩 나눠 합을 구하는 Part 0 ch15 예제 02의 4번 구역을 스코프 스레드로 다시 쓴 것입니다. 핸들을 모으지도, `to_vec()`으로 복사하지도 않습니다. 각 스레드는 `slots`의 칸 하나를 `&mut`로 받아 결과를 직접 씁니다.
- **5번 구역**: `thread '...'`로 시작하는 줄은 표준 에러입니다. (나)에서는 작업 스레드의 패닉(109번 줄) 뒤에 `main` 스레드의 패닉(108번 줄, `thread::scope` 호출)이 한 번 더 나옵니다. 이 두 번째 패닉을 `catch_unwind`가 붙잡았습니다.

### 02_thread_builder — 스레드 설정과 정보

```sh
cargo run -p p1-ch11-threads-sync --example 02_thread_builder
```

```text
1. Builder로 이름을 붙이면 스레드 안에서 thread::current().name()으로 읽습니다
    Builder로 만든 스레드: Some("작업자-1")
    thread::spawn 스레드: None
    main 스레드:          Some("main")
    spawn_scoped 스레드:  Some("스코프-작업자")
2. Builder::spawn은 실패를 패닉 대신 io::Result의 Err로 알려 줍니다
    스레드를 만들지 못했습니다: invalid stack size (종류: InvalidInput)
3. ThreadId는 스레드를 구별하는 값입니다
    핸들의 id == 스레드 안의 id: true
    main의 id == 작업 스레드의 id: false
4. available_parallelism은 동시에 돌릴 스레드 수의 추정값입니다
    이 컴퓨터의 추정값: 18
5. thread_local! 변수는 스레드마다 따로 있습니다
    main에서 세 번 부른 뒤: 3, 새 스레드에서 한 번 부른 뒤: 1
    main의 값은 그대로: 3
6. 패닉 훅은 프로세스 전체에 하나라서 어느 스레드의 패닉에도 불립니다
    [훅] 스레드 '계산기'의 패닉: attempt to divide by zero
    join이 받은 Err의 페이로드: attempt to divide by zero
```

- **2번 구역**: 에러 메시지는 macOS에서 확인한 것입니다. 운영체제에 따라 다른 에러가 나올 수 있습니다.
- **4번 구역**: `18`은 이 컴퓨터의 값이며 실행 환경마다 다릅니다.
- **6번 구역**: `[훅]` 줄은 바꿔 둔 훅이 `eprintln!`으로 표준 에러에 쓴 것입니다. 기본 훅이 아니므로 스레드 ID가 없고, 이름 `'계산기'`는 `Builder::name`으로 붙인 것입니다. 끝에서 `take_hook`으로 기본 훅을 되돌립니다.

### 03_mutex_more — `Mutex` 더 쓰기

```sh
cargo run -p p1-ch11-threads-sync --example 03_mutex_more
```

```text
1. try_lock은 기다리지 않고 지금 잠글 수 있는지만 알려 줍니다
    아무도 안 잠갔을 때: Ok(가드)
    가드가 살아 있을 때: Err(WouldBlock)
    가드를 버린 뒤:      Ok(가드)
2. 오래 걸리는 일은 잠금 밖에서 합니다
    (가) 가드를 쥔 채 일하는 동안 다른 스레드: Err(WouldBlock)
    (나) 복사본으로 일하는 동안 다른 스레드:   Ok(가드)
3. 여러 잠금을 서로 다른 순서로 잡으면 교착 상태가 됩니다
    a를 쥔 스레드 1이 b를 시도: Err(WouldBlock)
    b를 쥔 스레드 2가 a를 시도: Err(WouldBlock)
    순서를 지킨 이체 2만 번 뒤: x = 1000, y = 1000, 합계 = 2000
4. 포이즈닝: 가드를 쥔 스레드가 패닉하면 그 뒤의 잠금은 Err를 돌려줍니다

thread '<unnamed>' (7381066) panicked at part1-standard-lib/ch11-threads-sync/examples/03_mutex_more.rs:168:49:
called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    작업 스레드의 join: Err(패닉)
    is_poisoned: true
    try_lock: Err(Poisoned)
    꺼내 본 상태: Ledger { entries: [100, 200, 50], total: 300 } (항목 합 350)
    복구 후 is_poisoned: false
5. 혼자 가지고 있으면 잠글 필요가 없습니다: get_mut, into_inner
    get_mut 후 into_inner: [1, 2, 3]
    Arc::into_inner로 꺼낸 값: [10, 20, 30]
```

- **2·3번 구역**: `Barrier`로 두 스레드가 만나는 지점을 정했기 때문에 `try_lock`을 시도하는 순간 상대가 반드시 잠금을 쥐고 있습니다. 그래서 결과가 매번 같습니다.
- **4번 구역**: `"수수료".parse::<i64>()`가 실패해 `unwrap`이 패닉합니다. `total`을 고치기 전에 패닉했으므로 항목의 합(350)과 `total`(300)이 어긋나 있고, 예제는 합을 다시 계산해 바로잡은 뒤 `clear_poison`을 부릅니다.
- **5번 구역**: 세 스레드가 넣는 순서는 실행마다 다를 수 있어 정렬한 뒤 출력합니다.

### 04_rwlock — 읽기-쓰기 잠금

```sh
cargo run -p p1-ch11-threads-sync --example 04_rwlock
```

```text
1. 읽기 잠금은 여러 스레드가 동시에 쥘 수 있습니다
    세 스레드가 읽는 중에 try_read:  Ok
    세 스레드가 읽는 중에 try_write: Err(WouldBlock)
    읽은 메뉴의 글자 수: [2, 2, 3]
    모두 끝난 뒤 try_write: Ok
2. 쓰기 잠금은 혼자만 가집니다
    쓰는 중에 다른 스레드의 try_read: Err(WouldBlock), try_write: Err(WouldBlock)
    쓰기를 마친 뒤 읽은 값: 10
3. RwLock<Arc<T>>: 읽는 쪽은 스냅숏을 들고 가고 잠금은 바로 놓습니다
    바꾸기 전 같은 Arc를 가진 소유자 수: 4
    make_mut 뒤 바꾸기 전과 같은 할당인가: false
    읽는 스레드가 본 값: ["v1 안녕", "v1 안녕"]
    지금 설정: v2 반가워요
```

3번 구역의 소유자 수 4는 `config` 안의 `Arc`, 읽는 스레드 둘의 스냅숏, 그리고 비교를 위해 `main`이 만든 `before`입니다. 다른 소유자가 있으므로 `make_mut`은 값을 복제했고, 바꾼 뒤의 `Arc`는 `before`와 다른 할당입니다.

### 05_condvar — 조건 변수

```sh
cargo run -p p1-ch11-threads-sync --example 05_condvar
```

```text
1. 출발 신호: 모든 작업자를 한꺼번에 깨울 때는 notify_all
    ["작업자 1: ready = true", "작업자 2: ready = true", "작업자 3: ready = true"]
2. notify는 쌓이지 않습니다: 조건을 확인하지 않는 대기는 신호를 놓칩니다
    (가) 조건 없이 기다림  → 시간 초과: true
    (나) 조건을 보며 기다림 → 시간 초과: false
    (다) 아무도 바꾸지 않음 → 시간 초과: true
3. 생산자-소비자: 용량 3짜리 큐에 생산자 2개가 10개씩 넣습니다
    받은 개수: 20, 합계: 1110
    생산자 0의 값이 넣은 순서대로 왔는가: true
    생산자 1의 값이 넣은 순서대로 왔는가: true
    큐 길이가 용량(3)을 넘은 적이 없는가: true
```

- **1번 구역**: 작업자가 `wait_while`에 도착하기 전에 `main`이 신호를 보냈더라도, 조건(`ready`)이 이미 `true`라 기다리지 않고 지나갑니다. 그래서 언제 실행해도 세 작업자가 모두 출발합니다.
- **2번 구역**: (가)와 (다)는 30ms씩 기다린 뒤 시간 초과로 돌아옵니다. (가)는 가짜 깨어남이 일어나면 이론상 `false`가 될 수도 있지만 이 환경에서는 늘 `true`였습니다.
- **3번 구역**: 생산자 0은 1~10, 생산자 1은 101~110을 넣습니다. 두 생산자의 값이 섞이는 순서는 실행마다 다르지만, 같은 생산자의 값끼리는 넣은 순서를 지킵니다.

### 06_sync_channel — 용량이 정해진 채널

```sh
cargo run -p p1-ch11-threads-sync --example 06_sync_channel
```

```text
1. sync_channel(2): 용량이 다 차면 try_send는 Full을 돌려줍니다
    try_send(1): Ok(())
    try_send(2): Ok(())
    try_send(3): Err(TrySendError::Full(..))
    recv():      Ok(1)
    try_send(3): Ok(())
    수신자를 버린 뒤 try_send(4): Err(TrySendError::Disconnected(..))
2. 배압: 받는 쪽이 느리면 보내는 쪽이 기다립니다
    channel():         50ms 동안 보낸 개수 10, 받은 합계 55
    sync_channel(3):   50ms 동안 보낸 개수  3, 받은 합계 55
3. sync_channel(0): 받는 쪽이 받을 때까지 send가 끝나지 않습니다(랑데부)
    아무도 기다리지 않을 때 try_send: Err(TrySendError::Full(..))
    50ms 동안 보낸 개수 0, 받은 합계 55
4. try_iter는 지금 쌓여 있는 것만 꺼내고 기다리지 않습니다
    try_iter로 꺼낸 값: [5, 6, 7], 남은 것: Err(Empty)
5. Sender는 Sync라서 스코프 스레드들이 &tx로 함께 보낼 수 있습니다
    받은 값(정렬): [1, 4, 9]
```

- **1번 구역**: `TrySendError`의 디버그 형식은 안에 든 값 대신 `..`을 보여 줍니다. 값은 `Full(v)`의 `v`로 꺼낼 수 있습니다.
- **2·3번 구역**: 생산자가 멈추는 지점은 채널의 용량이 정하므로 50ms 뒤의 개수는 매번 같습니다. `main`이 50ms를 기다리는 동안 생산자 스레드가 한 번도 실행되지 못할 만큼 컴퓨터가 바쁘다면 개수가 더 작게 나올 수 있습니다.

### 07_atomics — 원자 타입

```sh
cargo run -p p1-ch11-threads-sync --example 07_atomics
```

```text
1. fetch_add는 읽기-더하기-쓰기를 한 번에 합니다
    fetch_add:     400000 (기대값 400000)
    load 후 store: 106779 (기대값 400000)
2. compare_exchange: 지금 값이 예상과 같을 때만 바꿉니다
    성공한 스레드 수: 1, 차지한 번호가 1~8 중 하나인가: true
3. try_update: 조건을 만족할 때만 고치기(재고 5개를 8명이 동시에 주문)
    주문 성공 5, 실패 3, 남은 재고 0
4. AtomicBool로 작업자에게 멈추라고 알리기
    작업자가 신호를 보고 멈췄습니다 (한 번 이상 일했는가: true)
5. static 원자 변수로 프로그램 전체에서 겹치지 않는 ID 나눠 주기
    받은 ID(정렬): [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
    다음에 나갈 ID: 13
```

- **1번 구역**: `load 후 store` 줄은 일부러 만든 경쟁 조건이라 값이 실행마다 다릅니다. 이 환경(디버그 빌드)에서 열두 번 실행했을 때 106000~109000 사이였고, 400000이 나온 적은 없었습니다.
- **2번 구역**: 어느 스레드가 이겼는지는 실행마다 다르므로 번호 대신 성공한 수를 출력합니다.
- **4번 구역**: 작업자가 몇 번 반복했는지는 실행마다 다릅니다.

### 08_once_lock_lazy_lock — 한 번만 초기화

```sh
cargo run -p p1-ch11-threads-sync --example 08_once_lock_lazy_lock
```

```text
1. OnceLock: 실행 중에 한 번 정하고 그 뒤로는 읽기만 하는 전역 설정
    set 전 get: None
    첫 set: Ok(())
    두 번째 set: Err("다른 서버")
    스레드들이 읽은 값: ["주문 서버/4", "주문 서버/4", "주문 서버/4"]
    wait로 받은 값: 42
2. 필드에 둔 OnceLock: 여러 스레드가 동시에 처음 요청해도 계산은 한 번
    요약: 러스트 표준…, 계산 횟수: 1
    8개 스레드가 같은 String을 받았는가: true
3. LazyLock: 처음 접근할 때 초기화되는 static
    접근 전: 초기화됨? false
    접근 후: 초기화됨? true
    주문별 금액: [9000, 9000, 4000, 4500], 합계: 26500
    초기화 클로저가 불린 횟수: 1
    ORDER_LOG에 쌓인 주문 수: 4
```

- **1번 구역**: 두 번째 `set`은 넣으려던 `Config`를 `Err`에 담아 돌려줍니다. 예제는 그 값의 `name`만 꺼내 출력했습니다.
- **2번 구역**: `std::ptr::eq`로 여덟 개의 `&str`이 같은 메모리를 가리키는지 비교합니다. 주소 자체는 실행마다 다르므로 출력하지 않습니다.
- **3번 구역**: 네 스레드가 거의 동시에 `UNIT_PRICES`에 처음 접근하지만 초기화 클로저는 한 번만 불립니다. 초기화 횟수는 `static` 원자 변수 `TABLE_INIT_CALLS`로 셉니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch11-threads-sync
rustc --edition 2024 compile_fail/e0499_scoped_threads_mut_borrow.rs --out-dir /tmp/rust-p1-ch11
```

`note: required by a bound in ...` 아래에 표준 라이브러리 소스 위치(`/rustc/0ef1bc38.../library/std/src/thread/scoped.rs:...`)가 나오는 경우가 있습니다. 경로의 해시 값은 툴체인마다 다르며, 아래 인용에서는 그 줄을 생략했습니다.

### 4.1 두 스코프 스레드가 같은 `Vec`을 가변으로 빌림 — E0499

파일: [`compile_fail/e0499_scoped_threads_mut_borrow.rs`](compile_fail/e0499_scoped_threads_mut_borrow.rs)

```text
error[E0499]: cannot borrow `scores` as mutable more than once at a time
  --> compile_fail/e0499_scoped_threads_mut_borrow.rs:16:17
   |
14 |     thread::scope(|s| {
   |                    - has type `&'1 Scope<'1, '_>`
15 |         s.spawn(|| scores.push(40)); // 첫 번째 가변 대여
   |         ---------------------------
   |         |       |  |
   |         |       |  first borrow occurs due to use of `scores` in closure
   |         |       first mutable borrow occurs here
   |         argument requires that `scores` is borrowed for `'1`
16 |         s.spawn(|| scores.push(50)); // 두 번째 가변 대여 → E0499
   |                 ^^ ------ second borrow occurs due to use of `scores` in closure
   |                 |
   |                 second mutable borrow occurs here
```

- **읽는 법**: [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md)에서 본 E0499와 같은 메시지입니다. 두 클로저가 `scores.push`를 부르려고 `scores`를 가변으로 빌렸습니다. `argument requires that scores is borrowed for '1`은 첫 번째 대여가 스코프(`'1`) 내내 이어진다는 뜻입니다. 스레드가 스코프가 끝날 때까지 살 수 있기 때문입니다.
- **원인**: 스코프 스레드는 빌려 쓸 수 있게 해 줄 뿐, 대여 규칙을 느슨하게 하지 않습니다. 두 스레드가 동시에 `push`하면 하나가 버퍼를 재할당하는 동안 다른 하나가 옛 버퍼에 쓰는 데이터 경쟁이 됩니다.
- **해결**: 무엇을 하려는지에 따라 고릅니다. 스레드마다 값을 돌려받아 스코프가 끝난 뒤 `main`이 `push`하거나(`let (a, b) = thread::scope(|s| { .. (a.join().unwrap(), b.join().unwrap()) });`), 겹치지 않는 조각(`chunks_mut`, `split_at_mut`)을 나눠 주거나(예제 01의 2번 구역), 정말 같은 `Vec`에 함께 넣어야 하면 `Mutex<Vec<i32>>`로 감쌉니다(스코프 스레드이므로 `Arc`는 필요 없습니다).

### 4.2 `MutexGuard`를 다른 스레드로 보냄 — E0277(`Send`)

파일: [`compile_fail/e0277_mutex_guard_not_send.rs`](compile_fail/e0277_mutex_guard_not_send.rs)

```text
error[E0277]: `std::sync::MutexGuard<'_, Vec<&str>>` cannot be sent between threads safely
  --> compile_fail/e0277_mutex_guard_not_send.rs:18:17
   |
18 |         s.spawn(move || guard.push("작업자가 기록"));
   |           ----- -------^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |           |     |
   |           |     `std::sync::MutexGuard<'_, Vec<&str>>` cannot be sent between threads safely
   |           |     within this `{closure@compile_fail/e0277_mutex_guard_not_send.rs:18:17: 18:24}`
   |           required by a bound introduced by this call
   |
   = help: within `{closure@compile_fail/e0277_mutex_guard_not_send.rs:18:17: 18:24}`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, Vec<&str>>`
note: required because `guard` is used within this closure
```

- **읽는 법**: Part 0 ch15의 `Rc` E0277과 같은 모양입니다. 클로저가 캡처한 `guard`(`MutexGuard`)가 `Send`가 아니라서 `s.spawn`이 요구하는 `Send`를 만족하지 못합니다.
- **원인**: `main` 스레드가 잠근 잠금을 작업 스레드가 풀게 하려 했습니다. `MutexGuard` 문서에 따르면 POSIX 스레드(pthreads)를 쓰는 플랫폼에서는 잠금을 얻은 스레드에서 풀어야 하므로, 가드가 다른 스레드에서 버려지지 않도록 `MutexGuard`를 `Send`가 아닌 타입으로 만들었습니다. 덕분에 "잠근 스레드와 푸는 스레드가 다르다"는 실수가 컴파일 에러가 됩니다.
- **해결**: 가드가 아니라 `Mutex`(의 참조)를 넘기고, 작업 스레드가 직접 잠급니다. `s.spawn(|| log.lock().unwrap().push("작업자가 기록"));` 이때 `main`이 가드를 쥔 채로 스코프를 열면 작업 스레드는 영원히 기다리므로, `main`의 가드는 그 전에 버립니다.

### 4.3 여러 스레드가 `Receiver`를 `&`로 함께 씀 — E0277(`Sync`)

파일: [`compile_fail/e0277_receiver_not_sync.rs`](compile_fail/e0277_receiver_not_sync.rs)

```text
error[E0277]: `std::sync::mpsc::Receiver<i32>` cannot be shared between threads safely
  --> compile_fail/e0277_receiver_not_sync.rs:21:25
   |
21 |         let a = s.spawn(|| rx.iter().take(3).sum::<i32>());
   |                   ----- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `std::sync::mpsc::Receiver<i32>` cannot be shared between threads safely
   |                   |
   |                   required by a bound introduced by this call
   |
   = help: the trait `Sync` is not implemented for `std::sync::mpsc::Receiver<i32>`
   = note: required for `&std::sync::mpsc::Receiver<i32>` to implement `Send`
note: required because `rx` is used within this closure
```

- **읽는 법**: 이번에는 "보낼 수 없다(sent)"가 아니라 "공유할 수 없다(shared)"이고 빠진 트레이트는 `Sync`입니다. `= note:`가 연결 고리를 알려 줍니다. 클로저는 `rx`를 빌렸으므로 `&Receiver`를 캡처했고, `&Receiver`가 `Send`이려면 `Receiver`가 `Sync`여야 합니다(Part 0 ch15 2.9절).
- **원인**: `mpsc`의 수신자는 소비자가 하나라는 전제로 만들어졌습니다(2.7절). 스코프 스레드라서 라이프타임 문제는 없지만, `Send`/`Sync` 검사는 스코프 스레드에도 그대로 적용됩니다.
- **해결**: 작업자들이 일감을 나눠 받아야 한다면 `let rx = Mutex::new(rx);`로 감싸고 각 작업자가 `while let Ok(job) = rx.lock().unwrap().recv() { .. }`로 하나씩 꺼냅니다(스레드 풀 바깥에서는 Part 0 ch20처럼 `Arc<Mutex<Receiver>>`). 이렇게 고친 코드는 이 환경에서 합계 21을 출력했습니다. 작업자마다 채널을 따로 만들어 일감을 나눠 보내는 방법도 있습니다.

### 4.4 `static`의 초기값에서 `const`가 아닌 함수를 부름 — E0015

파일: [`compile_fail/e0015_static_non_const_init.rs`](compile_fail/e0015_static_non_const_init.rs)

```text
error[E0015]: cannot call non-const associated function `HashMap::<String, u32>::new` in statics
  --> compile_fail/e0015_static_non_const_init.rs:12:57
   |
12 | static PRICES: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new()); // → E0015
   |                                                         ^^^^^^^^^^^^^^
   |
   = note: calls in statics are limited to constant functions, tuple structs and tuple variants
   = note: consider wrapping this expression in `std::sync::LazyLock::new(|| ...)`
```

- **읽는 법**: `static`의 초기값에서는 `const fn`, 튜플 구조체, 튜플 배리언트만 부를 수 있는데 `HashMap::new`는 그 어느 것도 아니라는 뜻입니다. 두 번째 `note:`가 해결책까지 알려 줍니다.
- **원인**: `static`의 값은 실행 파일에 미리 들어가므로 컴파일할 때 계산되어야 합니다. `Mutex::new`는 `const fn`이지만, `HashMap::new`는 실행할 때마다 다른 해시 시드를 고르기 때문에 `const fn`이 아닙니다.
- **해결**: `note:`대로 `static PRICES: LazyLock<Mutex<HashMap<String, u32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));`로 바꿉니다. 처음 접근할 때 만들어지고, 쓰는 코드(`PRICES.lock().unwrap().insert(..)`)는 역참조 덕분에 그대로입니다. 안의 값이 `const`로 만들어지는 경우(`Vec::new()`, `String::new()`, `BTreeMap::new()`)에는 `LazyLock` 없이 그대로 됩니다(2.9절의 표).

### 4.5 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| `let _ = m.lock().unwrap();`로 "잠가 둔다" | 컴파일 에러 `non-binding let on a synchronization lock` (rustc `let_underscore_lock`, 기본 deny). `_`는 값을 바로 버리므로 아무것도 잠그지 않음 | 가드를 쓸 동안 `let guard = ..`로 이름을 붙임 (2.4절) |
| `a.load(Ordering::Release)` | 컴파일 에러 `atomic loads cannot have Release or AcqRel ordering` (rustc `invalid_atomic_ordering`, 기본 deny) | 읽기에는 `Acquire`, `SeqCst`, `Relaxed` 중 하나 (2.8절) |
| `const COUNTER: AtomicUsize = AtomicUsize::new(0);`에 `fetch_add` | 경고 `const_item_interior_mutations`와 Clippy `declare_interior_mutable_const`. 쓸 때마다 새 값이 생겨 늘 0 | `static`으로 선언 (2.8절) |
| `thread::scope(\|s\| { let local = vec![..]; s.spawn(\|\| println!("{local:?}")); })` | E0373 `closure may outlive the current function, but it borrows local` | 빌려 줄 변수는 `thread::scope` 앞에서 만들거나, `move`로 스레드에 옮김 (2.2절) |
| `let h = thread::scope(\|s\| s.spawn(\|\| data.len()));`로 핸들을 스코프 밖으로 꺼냄 | 에러 코드 없이 `lifetime may not live long enough` | 스코프 안에서 `join`하고 결과만 돌려줌 (2.2절) |
| `Condvar` 하나를 서로 다른 `Mutex`와 씀 | 실행 중 패닉 `attempted to use a condition variable with two mutexes` (macOS) | `Condvar`마다 짝이 되는 `Mutex` 하나 (2.6절) |
| `Condvar::wait`를 `if`로 한 번만 감쌈 | 가짜 깨어남이나 놓친 알림 때문에 조건이 거짓인데 진행하거나 영원히 기다림 | `while` 또는 `wait_while` (2.6절) |
| `x.fetch_update(..)` | 경고 `use of deprecated method ... renamed to try_update for consistency` | `try_update` (1.95.0) (2.8절) |
| `&mut Mutex`에 `lock()` | Clippy `mut_mutex_lock` | `get_mut()` (2.4절) |
| `RwLock::write()`로 얻은 가드를 읽기에만 씀 | Clippy `readonly_write_lock` | `read()` (2.5절) |
| `thread_local!`의 상수 초기값을 `const { }` 없이 씀 | Clippy `missing_const_for_thread_local` | `static X: Cell<u32> = const { Cell::new(0) };` (2.3절) |
| 같은 스레드에서 이미 쥔 `Mutex`를 다시 `lock()` | 컴파일은 되지만 문서상 "돌아오지 않음"(교착 상태나 패닉) | 가드를 먼저 버리거나 가드를 함수 인자로 넘김 (2.4절) |

## 5. 연습 문제

### 연습 1. 스코프 스레드로 단어 수 세기

긴 문자열 `text`를 줄 단위로 나눠(`lines()`), 줄 목록을 네 조각으로 나누고 스코프 스레드 네 개가 각자 맡은 줄의 단어 수를 셉니다. `main`은 네 결과를 더해 출력합니다. `to_string()`, `to_vec()`, `clone()`, `Arc`를 쓰지 않아야 합니다.

```rust
let text = "가는 말이 고와야\n오는 말이 곱다\n낮말은 새가 듣고\n밤말은 쥐가 듣는다\n\
            티끌 모아 태산\n세 살 버릇 여든까지 간다\n백지장도 맞들면 낫다\n등잔 밑이 어둡다";
// 출력: 단어 수: 26
```

<details>
<summary>힌트</summary>

- `let lines: Vec<&str> = text.lines().collect();`로 줄 목록을 만들고, `lines.chunks(2)`로 네 조각으로 나눕니다. 조각은 `&[&str]`이고 `lines`를 빌린 것입니다.
- `thread::scope(|s| { .. })` 안에서 조각마다 `s.spawn(move || chunk.iter().map(|l| l.split_whitespace().count()).sum::<usize>())`로 스레드를 만들고, 핸들을 `Vec`에 모아 `join`한 값을 더해 스코프의 반환값으로 돌려줍니다. `move`가 옮기는 것은 조각(참조)입니다(2.2절).
- Part 0 ch15의 연습 1과 비교해 보세요. 무엇이 사라졌나요?

</details>

### 연습 2. `Mutex` 없는 작업 카운터

[Rustlings `threads2.rs`](https://github.com/rust-lang/rustlings/tree/main/exercises/20_threads)는 스레드 10개가 끝날 때마다 `jobs_done` 카운터를 하나씩 올리는 문제로, 보통 `Arc<Mutex<JobStatus>>`로 풉니다. 같은 프로그램을 `Mutex` 없이 `AtomicUsize`로, 그리고 `Arc` 없이 `thread::scope`로 다시 작성하세요. 마지막에 `jobs done: 10`을 출력합니다. 스레드가 하나 끝날 때마다 "지금까지 끝난 작업 수"도 돌려받아, 정렬했을 때 `[1, 2, ..., 10]`이 되는지 확인하세요.

<details>
<summary>힌트</summary>

- `let jobs_done = AtomicUsize::new(0);`을 스코프 앞에 두고, 스레드 안에서 `jobs_done.fetch_add(1, Ordering::Relaxed) + 1`을 돌려줍니다. `fetch_add`는 더하기 전의 값을 돌려주므로 `+ 1`이 이 스레드가 만든 값입니다.
- 돌려받은 값을 정렬하면 정확히 `1..=10`입니다. `fetch_add`가 읽기와 쓰기를 한 번에 하므로 두 스레드가 같은 값을 받을 수 없기 때문입니다. `load` 뒤 `store`로 바꿔 실행해 보고 중복이 생기는지도 확인해 보세요(2.8절).

</details>

### 연습 3. 닫을 수 있는 큐

[`05_condvar.rs`](examples/05_condvar.rs)의 `BoundedQueue`에 `close(&self)`를 추가하세요. 닫힌 뒤에는 `pop`이 큐가 빌 때까지는 남은 값을, 비면 `None`을 돌려줍니다(`fn pop(&self) -> Option<T>`). 소비자는 개수를 미리 알지 못해도 `while let Some(v) = queue.pop()`으로 모든 값을 받고 끝나야 합니다. 생산자 둘이 끝난 뒤 `main`이 `close`를 부르게 하세요.

<details>
<summary>힌트</summary>

- `Mutex` 안의 상태를 `struct State<T> { items: VecDeque<T>, closed: bool }`로 바꿉니다. 조건 변수의 조건은 `Mutex` 안의 값이어야 하기 때문입니다(2.6절).
- `pop`의 대기 조건은 `|st| st.items.is_empty() && !st.closed`입니다. 깨어난 뒤 `st.items.pop_front()`가 `None`이면 닫힌 것입니다.
- `close`는 `closed = true`로 바꾼 뒤 `not_empty.notify_all()`을 부릅니다. 기다리는 소비자가 여럿일 수 있고, 모두 "닫혔다"를 알아야 하므로 `notify_one`이 아닙니다.
- 채널에서 송신자를 모두 버리면 `for` 문이 끝나는 것(Part 0 ch15)과 같은 일을 직접 만든 것입니다.

</details>

### 연습 4. 전역 계산 캐시

`fn slow_square(n: u64) -> u64`(10ms를 잔 뒤 `n * n`을 돌려줌)의 결과를 전역 캐시에 기억하는 `fn cached_square(n: u64) -> u64`를 만드세요. 캐시는 `static CACHE: LazyLock<Mutex<HashMap<u64, u64>>>`, 실제 계산 횟수는 `static COMPUTED: AtomicUsize`로 셉니다. 스코프 스레드 8개가 각자 `[1, 2, 3, 1, 2, 3]`을 모두 계산할 때, 계산 횟수가 3이 되게 하세요.

<details>
<summary>힌트</summary>

- 먼저 `let cached = CACHE.lock().unwrap().get(&n).copied();`로 확인하고 가드를 버린 뒤, 없으면 계산하고 다시 잠가서 넣는 방법은 잠금 안에서 오래 걸리는 일을 하지 않는다는 점에서 좋지만(2.4절), 두 스레드가 동시에 "없다"를 보고 둘 다 계산할 수 있어 횟수가 3보다 커집니다. 이 환경에서 확인했을 때는 8개 스레드가 모두 같은 순간에 "없다"를 봐서 24번 계산했습니다.
- 횟수를 정확히 3으로 만드는 간단한 방법은 가드를 쥔 채로 확인부터 계산, 저장까지 하는 것입니다(`let mut cache = CACHE.lock().unwrap(); *cache.entry(n).or_insert_with(|| ..)`). 대신 계산하는 동안 다른 스레드는 모두 기다립니다.
- 두 장점을 모두 원하면 값 자리에 `Arc<OnceLock<u64>>`를 두는 방법이 있습니다. `HashMap<u64, Arc<OnceLock<u64>>>`에서 칸만 잠깐 잠가서 꺼낸 뒤, 잠금 밖에서 그 칸의 `get_or_init(|| slow_square(n))`을 부릅니다. 같은 `n`은 한 번만 계산되고, 다른 `n`은 동시에 계산됩니다(2.9절).

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::thread`](https://doc.rust-lang.org/std/thread/index.html): 스레드 모델, 이름, 스택 크기(기본 2 MiB와 `RUST_MIN_STACK`), 스레드 지역 저장소의 개요.
  - [`thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html): 스코프 스레드의 예제와 `'scope`/`'env` 라이프타임 설명(Lifetimes 절), 스코프 안 패닉(Panics 절).
  - [`thread::Builder`](https://doc.rust-lang.org/std/thread/struct.Builder.html): `name`, `stack_size`, `spawn`의 Errors·Panics 절, `spawn_scoped`.
  - [`std::sync`](https://doc.rust-lang.org/std/sync/index.html): 동기화가 필요한 이유(컴파일러와 CPU의 순서 바꾸기)와 고수준 동기화 도구 목록.
  - [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html): Poisoning 절(포이즈닝이 권고일 뿐인 이유), 가드를 일찍 버리는 예제, `clear_poison`.
  - [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html): 우선순위가 운영체제에 달린 점과 그로 인한 교착 상태 예, 읽기 중 패닉은 포이즈닝하지 않는다는 설명.
  - [`Condvar`](https://doc.rust-lang.org/std/sync/struct.Condvar.html): `wait`의 원자적 해제·재잠금, 가짜 깨어남, `notify_one`이 버퍼링되지 않는다는 설명.
  - [`Barrier`](https://doc.rust-lang.org/std/sync/struct.Barrier.html), [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html), [`OnceLock`](https://doc.rust-lang.org/std/sync/struct.OnceLock.html), [`LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html)
  - [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html): 두 종류의 채널(제한 없는 `channel`, 용량이 정해진 `sync_channel`)과 용량 0인 랑데부 채널, 연결이 끊겼을 때.
  - [`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html): 원자 타입이 어디서 쓸 수 있는지(Portability 절), `static`에 두는 예, 메모리 모델 개요.
- Rust By Example: [Std misc](https://doc.rust-lang.org/rust-by-example/std_misc.html) 장
  - [Threads](https://doc.rust-lang.org/rust-by-example/std_misc/threads.html): 스레드 10개를 만들고 핸들을 모아 `join`합니다. 같은 일을 `thread::scope`로 바꿔 보세요. 핸들을 모으는 반복문이 필요 없어집니다.
  - [Channels](https://doc.rust-lang.org/rust-by-example/std_misc/channels.html): 송신자를 스레드마다 복제해 보냅니다. `mpsc::channel()`을 `mpsc::sync_channel(0)`으로 바꾸고 타입 표기의 `Sender<i32>`를 `SyncSender<i32>`로 고치면, 각 스레드의 `send`가 `main`이 `recv`할 때까지 기다리게 됩니다(2.7절).
- Rustlings: [`20_threads`](https://github.com/rust-lang/rustlings/tree/main/exercises/20_threads)
  - `threads1.rs`: 스레드 10개의 결과를 `JoinHandle`로 모읍니다. 풀고 나서 `thread::scope`로 다시 써 보세요.
  - `threads2.rs`: "`Arc`만으로는 공유 값을 고칠 수 없다"는 문제입니다. 연습 2에서 원자 타입으로 다시 풉니다.
  - `threads3.rs`: 송신자 하나를 두 스레드가 써야 하는 문제로, 보통 `tx.clone()`으로 풉니다. 풀고 나서 `thread::spawn` 대신 스코프 스레드로 바꿔 `&tx`를 나눠 주는 방법도 생각해 보세요(2.7절).
- [The Rustonomicon: Atomics](https://doc.rust-lang.org/nomicon/atomics.html): 메모리 순서(`SeqCst`, `Acquire`/`Release`, `Relaxed`)를 자세히 설명합니다. "확신이 없으면 `SeqCst`"라는 2.8절의 지침도 여기에 있습니다. Part 10 `ch9-concurrency`에서 다시 읽습니다.
- 에러 설명: `rustc --explain E0015`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
