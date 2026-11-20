# ch17. 공유 상태 병렬성은 조심해서 쓰기

[Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md)에서 Rust의 동시성을 "겁 없는 동시성"이라고 배웠습니다. 여러 스레드가 같은 데이터를 동기화 없이 고치는 코드는 컴파일되지 않습니다. 그런데 컴파일러가 막는 것은 **데이터 경쟁**까지입니다. 두 스레드가 서로 상대가 쥔 잠금을 기다리며 영원히 멈추는 **교착 상태(deadlock)** 와, 각각은 잠금 안에서 고쳤는데도 함께 맞아야 하는 두 데이터가 어긋나는 일은 컴파일러가 보지 못합니다. 둘 다 운 나쁜 실행 순서에서만 나타나서 테스트도 잘 잡지 못합니다. 이 챕터는 Rust가 막는 것을 표 하나로 정리한 뒤(2.1절), 막지 못하는 것을 피하는 지침을 다룹니다. 함께 맞아야 하는 데이터는 잠금 하나로 묶고(지침 1), 잠금은 도우미 메서드 안에 가두고(지침 2), 가능하면 공유 상태 대신 채널을 쓰고(지침 3), 잠금 순서는 마지막 수단으로 정하되 정했으면 검사합니다(지침 4). 앞 챕터들이 미뤄 둔 숙제도 여기서 받습니다. [ch11](../ch11-drop-raii/README.md)의 "한 번에 끝나야 할 일을 쪼개는 문제, 잠금 순서와 교착 상태", [ch15](../ch15-borrow-checker/README.md)의 "`Arc<Mutex<T>>`와 교착 상태", [ch10](../ch10-standard-traits/README.md)의 "`Send`/`Sync`를 무엇이 막는가", [ch14](../ch14-lifetimes/README.md)의 "스레드와 `'static`의 나머지 이야기"입니다.

- 원제: [Item 17: Be wary of shared-state parallelism](https://effective-rust.com/deadlock.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch17-shared-state-parallelism`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 챕터의 내용을 안다고 보고 진행합니다. 스레드와 동기화 도구의 사용법은 그쪽으로 넘깁니다.

- [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 2.4~2.10절, 4절: `move` 클로저, 채널, `Mutex`와 `Arc`, `Send`/`Sync`, 데이터 경쟁과 경쟁 조건, 동시성 컴파일 에러
- [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 2.2절, 2.4~2.8절: 스코프 스레드, `try_lock`·`Barrier`·잠그는 순서·포이즈닝, `RwLock`, `Condvar`, `sync_channel`, 원자 타입
- [ch8](../ch8-references-pointers/README.md) 2.6절: 스마트 포인터를 고르는 순서(스레드를 넘는가)
- [ch10](../ch10-standard-traits/README.md) 2.9절: 자동 트레이트를 잃지 않았는지 확인하기
- [ch11](../ch11-drop-raii/README.md) 2.2·2.3절: 가드를 거쳐서만 자원에 닿게 하기, 가드의 범위를 블록과 `drop`으로 좁히기
- [ch14](../ch14-lifetimes/README.md) 2.7절: `T: 'static`과 `thread::spawn`

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 데이터 경쟁의 세 조건을 Rust가 막는 장치(대여 규칙, 소유권 이동, `Send`/`Sync` 자동 트레이트, 데이터를 감싼 `Mutex<T>`)와 연결하고, 동시성 컴파일 에러가 어떤 경쟁을 막았는지 설명할 수 있다. 필드 하나 때문에 타입이 `Sync`를 잃는 경우를 알아볼 수 있다.
2. 두 함수가 잠금 둘을 반대 순서로 잡을 때 교착 상태가 생기는 과정과, 잠금 범위를 좁혀 피하면 함께 맞아야 하는 데이터가 어긋나는 이유를 설명하고, 그런 데이터를 잠금 하나로 묶어 두 문제를 함께 없앨 수 있다.
3. 잠금을 도우미 메서드 안에서만 잡고 풀도록 API를 설계하고, 잠금을 쥔 채 클로저를 부르거나 `MutexGuard`를 돌려주는 API가 왜 교착 상태를 부르는지 설명할 수 있다.
4. 공유 상태를 한 스레드가 소유하고 채널로 요청을 받는 설계로 바꿀 수 있고, 두 설계의 비용을 비교해 고를 수 있다.
5. 잠금 순서를 정해야 할 때 그 순서를 실행 중에 검사하는 방법과, 교착 상태 검출 도구(`no_deadlocks`, `parking_lot::deadlock`, ThreadSanitizer)의 용도를 설명할 수 있다.

## 2. 핵심 개념

2.1절은 Rust가 막는 것과 막지 못하는 것을 정리하는 **이해** 부분이고, 2.2~2.5절이 **지침**입니다. 지침은 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 씁니다. 예제는 모두 호텔 프런트 하나를 다룹니다. 프런트에는 장부(투숙객 → 객실)와 배정표(객실 → 투숙객)가 있고, 두 표는 언제나 서로 맞아야 합니다. 컴파일되지 않는 코드는 `compile_fail/`에 두고(4절), 컴파일되지만 멈추거나 틀린 결과를 내는 코드는 `examples/`에서 권장 코드와 함께 실행합니다.

### 2.1 Rust가 막는 것과 막지 못하는 것

**막는 것: 데이터 경쟁.** 데이터 경쟁(data race)은 둘 이상의 스레드가 같은 메모리에 접근하고, 적어도 하나가 쓰고, 그 접근 사이에 순서를 정하는 동기화가 없을 때 생깁니다(Part 0 ch15 2.1절). Rust에서 이 세 조건을 모두 갖춘 안전한 코드는 컴파일되지 않습니다. 아래 표의 에러는 모두 1.100.0에서 같은 모양의 코드로 확인했습니다.

| 데이터 경쟁을 만들 코드 | 결과 | 막는 규칙 | 에러 예제 |
| --- | --- | --- | --- |
| 스코프 스레드 하나는 쓰고 하나는 읽음 | E0502 | 대여 규칙: 고치는 동안 다른 대여 없음 | 이 챕터 4.2절 |
| 스코프 스레드 둘이 같은 `Vec`에 `push` | E0499 | 대여 규칙 | [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 4.1절 |
| `spawn`한 스레드가 지역 변수를 빌림 | E0373 | `spawn`의 `'static` 바운드 | [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 4.1절 |
| 보낸 값을 다시 씀, `Mutex`를 스레드 여럿으로 옮김 | E0382 | 소유권 이동 | Part 0 ch15 4.2·4.3절 |
| `Arc` 안의 값을 잠금 없이 고침 | E0596 ``cannot borrow data in an `Arc` as mutable`` | `Arc`는 `&T`만 내줌 | `Rc`판: [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 4.5절 |
| `Rc`를 다른 스레드로 보냄, `Arc<RefCell<T>>`를 공유 | E0277 (`Send`, `Sync`) | 자동 트레이트 | Part 0 ch15 4.4·4.5절 |
| `Cell` 필드가 있는 구조체를 `Arc`로 공유 | E0277 (`Sync`) | 자동 트레이트는 필드에서 옴 | 이 챕터 4.1절 |
| `MutexGuard`를 다른 스레드로 보냄, `Receiver`를 공유 | E0277 | 자동 트레이트 | Part 1 ch11 4.2·4.3절 |

세 가지만 짚습니다.

- **규칙은 대여 규칙 하나입니다.** "공유하면 읽기만, 고치려면 혼자"([ch15](../ch15-borrow-checker/README.md) 2.1절)가 스레드 사이에서도 그대로 적용됩니다. `Mutex<T>`는 이 규칙을 실행 중에 지키는 도구입니다. 데이터가 `Mutex` **안에** 있으므로 잠그지 않고는 닿을 길이 없습니다. 잠금과 데이터가 따로 있는 언어(책은 C++를 예로 듭니다)에서는 잠금을 빠뜨린 메서드 하나가 경쟁을 만들어도 컴파일됩니다.
- **`Send`/`Sync`를 막는 것은 필드입니다.** 둘은 자동 트레이트(auto trait)라서 필드가 모두 `Sync`이면 구조체도 `Sync`입니다. 조회 횟수를 세려고 `Cell<u32>` 필드 하나를 넣으면 구조체 전체가 `Sync`를 잃고, `Arc`로 나눠 주는 코드가 모두 E0277이 됩니다(4.1절). 에러는 ``required by this field``로 그 필드를 가리킵니다. 타입을 만든 쪽이 이 변화를 빌드에서 알아채는 방법은 [ch10](../ch10-standard-traits/README.md) 2.9절의 `assert_send_sync` 검사입니다. `unsafe impl Send`로 컴파일러 대신 보증하는 일은 [`ch16-avoid-unsafe`](../ch16-avoid-unsafe/README.md)와 Part 10 `ch9-concurrency`에서 다룹니다. 안전한 Rust 밖, 즉 `unsafe` 코드와 FFI 경계([`ch34-ffi-boundaries`](../ch34-ffi-boundaries/README.md))에서는 데이터 경쟁이 여전히 생길 수 있습니다.
- **스레드에 데이터를 넘기는 길은 셋입니다(책에 없는 지침).** ch14 2.7절에서 `spawn`이 `'static`을 요구하는 이유는 "빌린 것을 품지 말라"는 것이었습니다. 그 바운드를 만족시키는 방법에 따라 비용이 다릅니다.

| 길 | 쓰는 것 | 맞는 경우 | 비용 |
| --- | --- | --- | --- |
| 빌려 주기 | 스코프 스레드 + `&T`(고치려면 `&Mutex<T>`) | 스레드가 스코프 안에서 반드시 끝남 | 없음. 멈춘 스레드가 있으면 스코프도 끝나지 않음 |
| 함께 소유하기 | `thread::spawn` + `Arc<T>` | 스레드가 언제 끝날지 모름 | 원자적 카운트. `Arc<Mutex<T>>`라면 잠금 |
| 영원히 두기 | `static`, `OnceLock`, `Box::leak` | 프로그램 내내 사는 설정 | 해제되지 않음 (ch14 2.7절, Part 1 ch11 2.9절) |
| 넘겨주기 | 채널로 값을 보냄 | 한 번에 한 스레드만 쓰면 됨 | 메시지 하나의 이동 (지침 3) |

스마트 포인터를 고르는 순서에서 "스레드를 넘는가"에 `Arc` 또는 스코프 스레드의 `&T`라고 답했던 것([ch8](../ch8-references-pointers/README.md) 2.6절)의 자세한 판이 이 표입니다. 이 챕터의 예제 01과 03은 일부러 멈추는 스레드를 만들기 때문에 스코프 스레드를 쓸 수 없고 `spawn` + `Arc`를 씁니다.

**막지 못하는 것.** 교착 상태와 어긋난 상태는 컴파일러가 보는 것과 종류가 다릅니다. 대여 검사기는 함수 하나씩 "이 참조가 살아 있는 동안 무엇이 금지되는가"를 봅니다. 잠금은 실행 중에 얻고 풀며, 교착 상태는 "스레드 1이 이 줄에 있을 때 스레드 2가 저 줄에 있으면"이라는 **실행 순서**의 성질입니다. 함수 하나하나는 옳고, 한 스레드에서 차례로 부르는 테스트도 통과합니다(예제 05의 1번 구역).

| 막지 못하는 것 | 모양 | 다루는 곳 |
| --- | --- | --- |
| 교착 상태: 잠금 순서 뒤집힘 | 두 작업이 잠금 둘을 반대 순서로 잡음 | 지침 1·4, 예제 01·05 |
| 교착 상태: 같은 스레드의 재잠금 | 잠금을 쥔 채 부른 코드가 같은 잠금을 다시 잡음 | 지침 2, 예제 03 |
| 교착 상태: 서로 기다리기 | 두 스레드가 서로의 답을 기다림(채널로도 생김) | 지침 3 |
| 어긋난 상태(경쟁 조건) | 함께 맞아야 하는 일을 잠금 둘(또는 두 번의 잠금)로 나눔 | 지침 1, 예제 02, Part 0 ch15 2.10절 |
| 잠금을 오래 쥠 | 느린 일을 잠금 안에서 함 | [ch11](../ch11-drop-raii/README.md) 2.3절, Part 1 ch11 2.4절 |
| 패닉과 포이즈닝 | 잠금을 쥔 스레드가 패닉함 | Part 1 ch11 2.4절, [`ch18-dont-panic`](../ch18-dont-panic/README.md) |

지침을 시도할 순서는 지침 번호와 다릅니다. 지침은 문제가 드러나는 순서로 놓았고, 설계할 때는 다음 순서로 묻습니다.

| 물을 것 | 그렇다면 | 지침 |
| --- | --- | --- |
| 정말 여러 스레드가 같은 데이터를 고쳐야 하나 | 아니면 한 스레드가 소유하고 채널로 요청을 받음 | 3 |
| 함께 맞아야 하는 데이터가 있나 | 잠금 하나로 묶음 | 1 |
| 누가 잠금을 잡고 푸나 | 그 타입의 도우미 메서드만. 쥔 채 모르는 코드를 부르지 않음 | 2 |
| 그래도 한 작업이 잠금 여럿을 쥐어야 하나 | 순서를 정하고, 문서에만 두지 말고 검사함 | 4 |

### 2.2 지침 1: 함께 맞아야 하는 데이터는 잠금 하나로 묶는다

**문제 상황.** 프런트의 두 작업이 동시에 들어옵니다. 손님 서아가 302호에 체크인하고(`check_in`), 직원이 점검할 301호를 비웁니다(`close_room`). 처음에는 301호에 하준이 묵고 있습니다. 두 표를 따로 잠그면 서로 다른 객실을 다루는 작업끼리는 덜 기다릴 것 같아서 잠금을 둘로 나눴습니다.

**비권장 코드 1: 잠금 둘, 반대 순서.** 손님이 오면 장부부터, 객실을 비울 때는 배정표부터 고치는 것이 자연스러워 보입니다.

```rust
fn check_in(&self, guest: &str, room: u32) {
    let mut guests = self.guests.lock().unwrap(); // ① 장부
    guests.insert(guest.to_string(), room);
    let mut rooms = self.rooms.lock().unwrap();   // ② 배정표
    rooms.insert(room, guest.to_string());
}

fn close_room(&self, room: u32) {
    let mut rooms = self.rooms.lock().unwrap();   // ① 배정표 ← 순서가 뒤집힘
    rooms.remove(&room);
    let mut guests = self.guests.lock().unwrap(); // ② 장부
    guests.retain(|_, r| *r != room);
}
```

스레드 1이 장부를, 스레드 2가 배정표를 잠근 직후라면, 스레드 1은 배정표를, 스레드 2는 장부를 기다립니다. 상대는 자기가 기다리는 잠금을 놓지 않으므로 둘 다 영원히 멈춥니다. 이것이 **잠금 순서 뒤집힘(lock inversion)** 입니다. 스레드가 셋 이상이고 잠금이 셋 이상이어도, 기다림이 원을 그리면 같은 일이 생깁니다.

[`01_lock_inversion.rs`](examples/01_lock_inversion.rs)는 이 순간을 매번 만듭니다. 두 작업이 첫 잠금을 쥔 순간에 서로를 최대 100ms 기다리게 하는 관찰용 장치(`Meeting`)를 넣었습니다. 실제 코드에서는 이 틈이 운영체제가 스레드를 바꾸는 순간에 우연히 생깁니다.

```text
1. 비권장: 잠금 둘, check_in은 장부 → 배정표, close_room은 배정표 → 장부
    두 작업이 동시에 첫 잠금을 쥠: 예
    1초 안에 끝난 작업: 0/2 → 교착 상태
    지금: 장부 잠김, 배정표 잠김
```

컴파일러도 clippy도 아무 말을 하지 않습니다. 같은 두 함수를 한 스레드에서 차례로 부르면 문제없이 끝납니다.

**비권장 코드 2: 잠금 범위를 좁혀 한 번에 하나만.** 교착 상태가 두 잠금을 **함께** 쥐어서 생겼으니, 하나를 놓은 뒤에 다른 하나를 잡으면 될 것 같습니다.

```rust
fn check_in(&self, guest: &str, room: u32) {
    self.guests.lock().unwrap().insert(guest.to_string(), room); // 장부만 잠깐
    // ← 아무 잠금도 쥐지 않은 틈
    self.rooms.lock().unwrap().insert(room, guest.to_string());  // 배정표만 잠깐
}
```

교착 상태는 사라집니다. 그 대신 한 작업이 두 임계 구역(critical section, 잠금을 쥐고 하는 구간)으로 쪼개졌고, 그 사이의 틈에서 다른 스레드가 반쯤 고친 상태를 봅니다. [`02_split_critical_section.rs`](examples/02_split_critical_section.rs)는 두 가지 틈을 보여 줍니다.

```text
1. check_in("서아", 302)의 두 단계 사이에 현황판이 두 표를 읽음
  [비권장] 잠금 둘, 범위를 좁혀 한 번에 하나만 잠금
    현황판이 본 것: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준"} → 서로 맞지 않음
2. close_room(301)의 두 단계 사이에 check_in("서아", 301)이 끼어듦
  [비권장] 잠금 둘, 범위를 좁혀 한 번에 하나만 잠금
    끝난 뒤: 장부 {}, 배정표 {301: "서아"} → 서로 맞지 않음
```

1번은 잠깐 보이는 어긋남입니다. 현황판은 장부에는 있고 배정표에는 없는 서아를 봅니다. 2번은 남는 어긋남입니다. 점검이 배정표에서 301호를 비운 뒤, 체크인이 두 표에 서아를 넣고, 점검이 이어서 "301호에 묵는 사람"을 장부에서 지웁니다. 배정표에는 서아가 남고 장부에서는 사라졌습니다. 모든 접근이 잠금 안에서 일어났으므로 데이터 경쟁은 없습니다. Part 0 ch15 2.10절의 "읽고 한 번, 쓰고 한 번 잠그는" 카운터와 같은 경쟁 조건입니다. [ch11](../ch11-drop-raii/README.md) 2.3절이 "범위를 좁힌다는 것은 필요 없는 일을 빼는 것이지, 한 번에 끝나야 하는 일을 쪼개는 것이 아니다"라고 미뤄 둔 문제가 이것입니다.

**권장 코드: 잠금 하나.** 두 표 사이에는 "서로 맞는다"는 불변식이 걸쳐 있습니다. 불변식이 걸친 데이터 전체를 한 구조체에 담고 잠금 하나로 지킵니다.

```rust
struct Ledger {
    guests: BTreeMap<String, u32>, // 장부
    rooms: BTreeMap<u32, String>,  // 배정표
}

struct OneLock {
    ledger: Mutex<Ledger>,
}

fn check_in(&self, guest: &str, room: u32) {
    let mut ledger = self.ledger.lock().unwrap();
    ledger.guests.insert(guest.to_string(), room);
    ledger.rooms.insert(room, guest.to_string()); // 두 단계가 한 임계 구역 안에 있음
}
```

잠금이 하나뿐이니 순서가 없고(교착 상태 없음), 한 작업이 한 임계 구역이니 틈이 없습니다(어긋남 없음). 같은 관찰 장치를 그대로 둔 채 실행한 결과입니다.

```text
2. 권장: 장부와 배정표를 잠금 하나로 묶음
    두 작업이 동시에 첫 잠금을 쥠: 아니오
    1초 안에 끝난 작업: 2/2 → 모두 끝남
    지금: 장부 {"서아": 302}, 배정표 {302: "서아"}
```

상대가 잠금을 기다리는 중이라 "동시에 첫 잠금을 쥐는" 순간 자체가 생기지 않습니다. 예제 02의 권장 판에서도 현황판은 체크인이 끝난 뒤의 상태만 보고, 점검과 체크인이 겹쳐도 두 표가 맞습니다.

예제 01의 3번 구역은 잠금 둘을 두되 모든 작업이 "장부 → 배정표" 순서로 잡는 판입니다. 이것도 끝까지 끝나지만, 모든 작업이 순서를 지킨다는 데 기댑니다. 책은 이것을 마지막 수단으로 둡니다(지침 4).

**지침을 따르지 않아도 되는 경우.**

- 데이터 사이에 걸친 불변식이 없으면 잠금을 나눠도 됩니다. 객실별 청소 기록처럼 장부·배정표와 상관없는 데이터를 한 잠금에 묶으면 관계없는 작업끼리 기다리기만 합니다. 묶을 기준은 "함께 맞아야 하는가"이지 "같은 타입에 있는가"가 아닙니다.
- 읽기가 대부분이라 잠금 하나가 병목이라면 잠금을 쪼개기 전에 `RwLock<Ledger>`로 바꿔 봅니다. 잠금은 여전히 하나입니다(Part 1 ch11 2.5절). 값 하나짜리 카운터는 원자 타입으로 충분하지만, 여러 값에 걸친 불변식은 원자 변수 여러 개로 지킬 수 없습니다(Part 1 ch11 2.8절).
- 측정해 보니 정말 잠금을 나눠야 한다면, 한 작업이 잠금 여럿을 쥐는 곳마다 지침 4를 따릅니다.

### 2.3 지침 2: 잠금은 도우미 메서드 안에서 잡고 풀고, 쥔 채로 모르는 코드를 부르지 않는다

**문제 상황.** 지침 1대로 잠금 하나를 쓰는 `Front` 타입을 다른 팀이 씁니다. 그 팀이 "오늘 떠나는 손님(하준, 도윤)을 체크아웃시키는" 코드를 씁니다. `Front`에는 장부를 둘러보는 메서드와 체크아웃 메서드가 있습니다.

```rust
fn check_out(&self, guest: &str) -> Option<u32> {
    let mut ledger = self.ledger.lock().unwrap();
    let room = ledger.guests.remove(guest)?;
    ledger.rooms.remove(&room);
    Some(room)
}
```

**비권장 코드 1: 잠금을 쥔 채 클로저를 부름.** 둘러보기 메서드가 잠금을 쥔 채 부르는 쪽의 클로저를 부릅니다.

```rust
fn for_each_guest_locked(&self, mut f: impl FnMut(&str, u32)) {
    let ledger = self.ledger.lock().unwrap();
    for (name, room) in &ledger.guests {
        f(name, *room); // 이 클로저가 무엇을 할지 Front는 모름
    }
}

// 부르는 쪽
front.for_each_guest_locked(|name, _room| {
    if LEAVING_TODAY.contains(&name) {
        vacated.extend(front.check_out(name)); // 같은 잠금을 같은 스레드가 다시 잠금
    }
});
```

처음에 이 메서드를 만들 때 클로저는 목록을 출력하기만 했을 것입니다. 나중에 누군가 클로저 안에서 `check_out`을 부르면, 이미 쥔 잠금을 같은 스레드가 다시 잠급니다. `Mutex::lock` 문서는 이때의 동작을 정하지 않고 "두 번째 호출은 돌아오지 않는다(예를 들어 패닉하거나 교착 상태가 된다)"고만 적습니다. 이 환경(macOS)에서는 멈췄습니다. 클로저가 다른 객체의 잠금을 잡는다면 이번에는 두 잠금의 순서가 클로저를 쓴 사람에게 달립니다. 책이 이 지침의 이유로 드는 것도 같습니다. 잠금을 쥔 채 부르는 코드가 앞으로 어떻게 바뀔지를 `Front`는 통제할 수 없습니다.

**비권장 코드 2: `MutexGuard`를 돌려줌.**

```rust
fn ledger(&self) -> MutexGuard<'_, Ledger> {
    self.ledger.lock().unwrap()
}

// 부르는 쪽
let ledger = front.ledger(); // 떠날 손님을 고르려고 가드를 받음
let leaving: Vec<String> = ledger.guests.keys().filter(..).cloned().collect();
leaving.iter().filter_map(|name| front.check_out(name)).collect() // 가드가 아직 살아 있음
```

가드는 `Drop`이 있으므로 마지막 사용이 아니라 스코프 끝까지 잠금을 쥡니다([ch11](../ch11-drop-raii/README.md) 2.3절). 부르는 쪽은 `ledger`를 다 쓴 것 같지만 `check_out`은 같은 잠금을 기다립니다. 가드를 돌려주는 순간, 잠금을 얼마나 오래 쥘지와 쥔 채 무엇을 할지가 모두 부르는 쪽으로 넘어갑니다. 책은 교착 상태의 관점에서 이것을 클로저를 부르는 것과 같은 종류의 위험으로 봅니다.

[`03_keep_locks_inside.rs`](examples/03_keep_locks_inside.rs)에서 두 비권장 판은 300ms 안에 끝나지 않고, 그 뒤에도 잠금은 잠긴 채입니다.

**권장 코드.** 잠금은 `Front`의 도우미 메서드 안에서만 잡고 풉니다. 부르는 쪽의 코드는 잠금을 푼 뒤에 부르고, 가드 대신 값을 돌려줍니다.

```rust
/// 잠금 안에서는 사본만 만들고, 잠금을 푼 뒤에 클로저를 부릅니다.
fn for_each_guest(&self, mut f: impl FnMut(&str, u32)) {
    let snapshot = self.guests(); // 잠금은 이 줄 안에서 잡고 풀림
    for (name, room) in &snapshot {
        f(name, *room);
    }
}

/// 가드 대신 값을 돌려줍니다.
fn guests(&self) -> Vec<(String, u32)> {
    let ledger = self.ledger.lock().unwrap();
    ledger.guests.iter().map(|(n, r)| (n.clone(), *r)).collect()
}
```

부르는 쪽의 클로저는 비권장 1과 똑같은데 끝까지 끝나고, 비운 객실은 `[303, 301]`입니다. 고친 곳은 `Front` 안 한 곳뿐입니다. 책은 잠금 범위를 "작고 뻔하게" 두라고 하며, 관련된 잠금 아래에서 값을 읽고 쓰는 도우미 메서드를 권합니다. `check_out`처럼 두 표를 함께 고치는 일도 도우미 메서드 하나가 한 임계 구역으로 처리합니다.

사본에는 비용이 있습니다. 사본을 만든 뒤 `check_out`을 부르기 전에 다른 스레드가 먼저 체크아웃시켰을 수 있습니다. 그래서 `check_out`은 `Option`을 돌려주어 "이미 없음"을 알립니다. "확인하고 고치기"가 한 덩어리여야 한다면(예: 아직 그 객실에 있을 때만 내보내기), 그 조건까지 받는 도우미 메서드를 만들어 한 임계 구역에서 처리합니다. 가드를 넘겨 부르는 쪽에 맡기지 않습니다(연습 1).

잠금을 쥔 채 하지 말아야 할 일은 클로저만이 아닙니다(책에 없는 지침). 채널의 `recv`, 가득 찬 `sync_channel`의 `send`, 다른 스레드의 `join`, 다른 잠금처럼 **언제 끝날지 이 코드가 모르는 기다림**은 모두 같은 위험이 있습니다. `Condvar::wait`만 예외로, 기다리는 동안 잠금을 풉니다(Part 1 ch11 2.4·2.6절). 잠금이 풀리는 곳을 뻔하게 하는 다른 방법은 가드를 조건식에 만들지 않는 것입니다. `match self.ledger.lock().unwrap().guests.get(name)`은 `match` 전체가 끝날 때까지 잠금을 쥡니다(2.6절의 표).

예제의 비권장 판에 clippy는 기본 설정에서 아무 경고도 내지 않습니다. nursery 그룹의 `significant_drop_tightening`(기본 꺼짐)을 켜면 비권장 2의 `let ledger = f.ledger();`에 ``temporary with significant `Drop` can be early dropped``라고 경고하지만, 올바른 `check_out`과 예제 01·05의 정상 코드에도 경고해서 잡음이 많았습니다.

**지침을 따르지 않아도 되는 경우.**

- 잠금 안에서 부르는 클로저를 그 자리에서 쓰고, 그 클로저가 잠금에 닿을 수 없다면 괜찮습니다. 권장 판의 `check_out` 안에도 `guests.retain(|_, r| *r != room)` 같은 클로저가 있습니다. 위험한 것은 "부르는 쪽이 넘긴, 무엇을 할지 모르는" 클로저입니다.
- 잠금 자체가 목적인 저수준 타입은 가드를 돌려줍니다. `Mutex::lock`이 그렇고, 예제 05의 `RankedMutex::lock`도 그렇습니다. 이런 타입은 가드를 쥔 채 무엇을 하면 안 되는지를 문서로 밝힙니다.

### 2.4 지침 3: 공유 상태 대신 채널로 소유권을 넘긴다

**문제 상황.** 지침 1·2를 지키면 공유 상태도 안전하게 쓸 수 있습니다. 하지만 그 안전은 이 타입을 고치는 모든 사람이 두 지침을 계속 지킨다는 데 기댑니다. 책은 조언을 "가능하면 공유 상태를 피하라"로 시작하고, Part 0 ch15 2.5절에서 본 "메모리를 공유해서 통신하지 말고, 통신해서 메모리를 공유하라"를 인용합니다.

**비권장 코드.** 모든 스레드가 `Arc<Mutex<Ledger>>`의 복제본을 들고 각자 잠급니다. 예제 01·02의 판이 모두 이 모양입니다.

```rust
let desk = Arc::new(OneLock { ledger: Mutex::new(ledger) });
let d1 = Arc::clone(&desk);
thread::spawn(move || d1.check_in("서아", 302)); // 각 스레드가 안에서 lock()
let d2 = Arc::clone(&desk);
thread::spawn(move || d2.close_room(301));
```

지금은 옳더라도, 잠금을 어디서 잡고 풀지와 무엇을 쥔 채 무엇을 부르는지가 이 타입을 쓰는 코드 전체에 걸린 약속이 됩니다.

**권장 코드.** 장부는 프런트 스레드 **하나**가 소유하고, 다른 스레드는 요청을 메시지로 보냅니다. 답이 필요한 요청은 답을 받을 송신자를 함께 보냅니다.

```rust
enum Request {
    CheckIn { guest: String, room: u32, reply: Sender<()> },
    CloseRoom { room: u32, reply: Sender<Vec<String>> },
    Snapshot { reply: Sender<(Guests, Rooms)> },
}

fn run_front(inbox: Receiver<Request>, mut ledger: Ledger) -> (Ledger, usize) {
    for request in inbox { /* 요청 하나를 끝까지 처리하고 답을 보냄 */ }
    (ledger, handled) // 송신자가 모두 사라지면 반복이 끝나고 장부를 돌려줌
}
```

`Ledger`의 메서드는 평범한 `&mut self` 메서드이고 잠금이 없습니다. 부르는 쪽에는 `FrontHandle`이라는 손잡이를 주어, 요청을 보내고 답을 기다리는 일을 메서드 안에 감춥니다. 손잡이의 `check_in`과 `close_room`은 지침 2의 도우미 메서드와 같은 모양입니다.

[`04_channel_redesign.rs`](examples/04_channel_redesign.rs)는 예제 01과 02의 장면을 그대로 다시 봅니다.

```text
1. 01의 두 작업을 두 스레드가 같은 순간에 요청
    close_room(301)이 내보낸 손님: ["하준"]
    지금: 장부 {"서아": 302}, 배정표 {302: "서아"} → 서로 맞음
2. 체크인과 점검을 500번씩 되풀이하는 동안 현황판이 500번 읽음
    현황판이 어긋난 상태를 본 횟수: 0/500
```

잠금이 없으니 잠금 순서도 없습니다. 프런트 스레드는 요청 하나를 끝까지 처리한 뒤에 다음 요청을 꺼내므로 **요청 하나가 곧 임계 구역 하나**이고, 두 단계 사이의 틈이 생길 수 없습니다. 불변식을 지키는 코드는 `Ledger`의 메서드 안에만 있어서 스레드 없이 단위 테스트할 수 있습니다. 마지막에 손잡이를 버리면 반복이 끝나고, `join`이 장부를 돌려줍니다. `Arc::into_inner`로 꺼낼 일도 없습니다.

**비용.** 채널판이 공짜는 아닙니다.

- 모든 요청을 스레드 하나가 처리하므로 처리량은 잠금 하나와 비슷합니다. 읽기만 하는 요청도 차례를 기다립니다.
- 요청 타입과 답 채널을 만드는 코드가 늘어납니다. 요청마다 채널을 하나씩 만드는 비용도 있습니다.
- 채널도 교착 상태를 없애 주지는 않습니다. 프런트 스레드가 요청을 처리하다가 다른 스레드의 답을 기다리는데, 그 스레드가 프런트의 답을 기다리고 있으면 둘 다 멈춥니다. 서로에게 요청하고 답을 기다리는 두 스레드를 같은 모양으로 만들어 확인했습니다(실행 순서에 따라 멈추기도 하고 끝나기도 합니다). 요청을 처리하는 스레드는 다른 스레드의 답을 기다리지 않게 설계합니다.
- `channel()`은 용량 제한이 없어서 처리가 느리면 요청이 쌓입니다. `sync_channel(n)`로 배압을 줍니다(Part 1 ch11 2.7절).
- 프런트 스레드가 패닉하면 손잡이의 `send`와 `recv`가 실패합니다. 예제는 `expect`로 처리했는데, 패닉을 어떻게 다룰지는 [`ch18-dont-panic`](../ch18-dont-panic/README.md)의 주제입니다.

**지침을 따르지 않아도 되는 경우.** Part 1 ch11 2.7절의 "채널이냐, 공유 상태냐" 표가 기준입니다. 여러 스레드가 같은 값을 계속 읽는 설정이나 캐시는 `RwLock`이나 `RwLock<Arc<T>>` 스냅숏이, 카운터와 플래그는 원자 타입이 더 단순합니다. 데이터를 겹치지 않는 조각으로 나눠 계산한다면 공유 자체가 필요 없습니다(스코프 스레드와 `chunks_mut`, Part 1 ch11 2.2절).

### 2.5 지침 4: 잠금 순서는 마지막 수단으로 정하고, 정했으면 검사한다

**문제 상황.** 잠금을 하나로 묶을 수 없는 경우가 있습니다. 서로 다른 모듈이 각자 잠금을 가진 경우, 같은 타입의 두 값 사이에서 무언가를 옮기는 경우(두 호텔 사이에서 손님을 옮기기), 측정 결과 잠금을 나눠야 하는 경우입니다. 그러면 한 작업이 잠금 여럿을 쥐게 되고, 모든 작업이 같은 순서로 잠가야 합니다. 같은 타입의 두 값은 id 같은 비교할 수 있는 값으로 순서를 정합니다(Part 1 ch11 2.4절). 책은 잠금 순서(lock hierarchy)를 정하고, 문서로 남기고, 테스트하고, 지켜지는지 감시하는 것을 **마지막 수단**으로 둡니다. 모든 사람이 순서를 한 번도 어기지 않아야 동작하는 방식이기 때문입니다.

**비권장 코드: 순서를 주석으로만 정함.** 예제 01의 비권장 판에 "잠금 순서: 장부 → 배정표"라는 주석을 붙여도 `close_room`은 여전히 배정표부터 잡습니다. [`05_checked_lock_order.rs`](examples/05_checked_lock_order.rs)의 1번 구역은 이 프런트를 한 스레드에서 차례로 부르는 단위 테스트입니다.

```text
1. 비권장: 잠금 순서를 주석으로만 정함 (장부 → 배정표)
    check_in, close_room 차례로 실행: 모두 끝남 → 테스트 통과
```

교착 상태는 두 작업이 겹치는 운 나쁜 순서에서만 나타나므로, 보통의 테스트는 순서 위반을 찾지 못합니다.

**권장 코드: 잠금마다 순위를 붙이고 잠글 때 검사합니다(책에 없는 지침).** 책은 "감시하라"까지만 말하고 방법은 정하지 않습니다. 예제 05는 한 가지 방법을 구현합니다. 잠금마다 순위를 붙이고, 이 스레드가 지금 쥔 잠금보다 순위가 낮거나 같은 잠금을 잡으려 하면 잠그기 전에 `Err`를 돌려줍니다.

```rust
thread_local! {
    /// 이 스레드가 지금 쥐고 있는 잠금들의 (순위, 이름)
    static HELD: RefCell<Vec<(u32, &'static str)>> = const { RefCell::new(Vec::new()) };
}

fn lock(&self) -> Result<RankedGuard<'_, T>, OrderError> {
    let highest = HELD.with_borrow(|held| held.iter().copied().max());
    if let Some(holding) = highest
        && holding.0 >= self.rank
    {
        return Err(OrderError { holding, wanted: (self.rank, self.name) });
    }
    let guard = self.inner.lock().unwrap();
    HELD.with_borrow_mut(|held| held.push((self.rank, self.name)));
    Ok(RankedGuard { rank: self.rank, guard })
}
// RankedGuard의 Drop이 HELD에서 자기 기록을 지웁니다.
```

장부를 순위 1, 배정표를 순위 2로 만들면 같은 단위 테스트가 위반을 찾습니다.

```text
2. 권장: 잠금마다 순위를 붙이고 잠글 때 검사
    check_in("서아", 302): Ok
    close_room(301): Err(배정표(2)를 쥔 채 장부(1)를 잠그려 함)
```

검사는 **이 스레드가 쥔 잠금만** 보므로 다른 스레드의 실행 순서와 상관없이 결정적입니다. 교착 상태가 실제로 일어나기를 기다리지 않고, 위반하는 코드 경로를 한 번 지나가기만 하면 찾습니다. 같은 순위끼리도 거부하므로 같은 잠금을 다시 잡는 실수(지침 2)도 찾습니다. 비용은 잠글 때마다 스레드 지역 변수를 읽고 쓰는 일인데, 필요하면 `#[cfg(debug_assertions)]`로 디버그 빌드에서만 검사하게 할 수 있습니다. 같은 타입의 두 값처럼 순위가 같은 잠금을 함께 잡아야 하는 경우는 순위를 `(단계, id)` 쌍으로 넓혀야 합니다(연습 3).

**도구.** 책은 교착 상태 검출 도구를 CI에 넣으라고 권하며 셋을 소개합니다. 이 챕터는 의존성을 추가하지 않았고, 아래는 문서로 확인한 내용입니다(2026-11-19). CI에 넣는 방법은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 다룹니다.

| 도구 | 하는 일 | 확인한 것 |
| --- | --- | --- |
| [`no_deadlocks`](https://crates.io/crates/no_deadlocks) | `std::sync`와 같은 API의 잠금으로 바꿔 끼우면, 교착 상태가 생겼을 때 디버그 정보를 표준 에러(또는 지정한 파일)에 씁니다 | crates.io 1.3.2 (2023-11-19) |
| [`parking_lot`](https://crates.io/crates/parking_lot)의 `deadlock` 모듈 | `deadlock_detection` 기능을 켜면 `check_deadlock()`이 교착 상태에 빠진 스레드를 찾아 줍니다. 문서는 "실험적"이며, 찾아낸 스레드는 계속 멈춰 있다고 밝힙니다 | crates.io 0.13.0, docs.rs의 모듈 문서 |
| ThreadSanitizer | rustc에서는 `-Zsanitizer=thread` 불안정 플래그로 켭니다. 책은 교착 상태 검출 도구로 소개하지만, 1.100.0의 unstable book은 "데이터 경쟁 검출 도구"라고만 설명합니다 | 1.100.0 unstable book의 sanitizer 절. nightly가 필요해 **이 챕터에서는 실행하지 못했고**, 잠금 순서를 검사하는지도 확인하지 못했습니다 |

도구가 없어도 할 수 있는 일이 있습니다(책에 없는 지침). 예제 01과 03처럼 멈출 수 있는 작업을 스레드에서 돌리고 `recv_timeout`으로 제한 시간을 두면, 영원히 멈출 테스트가 실패하는 테스트로 바뀝니다.

**지침을 따르지 않아도 되는 경우.** 잠금 하나로 충분하면(지침 1) 순서를 정할 일 자체가 없습니다. 모든 작업이 잠금을 한 번에 하나만 쥔다면 순서는 필요 없지만, 그때는 지침 1의 어긋남을 다시 따져 봅니다. 책은 이 Item을 "틀린 곳이 뻔히 보이지 않는 복잡한 코드보다 뻔히 옳은 단순한 코드"를 고르라는 말로 마칩니다. 순위 검사는 복잡한 코드를 감시하는 도구이지, 단순한 설계를 대신하지 않습니다.

### 2.6 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **E0382의 복제 제안.** 책이 보여 주는 E0382(한 값을 두 스레드의 `move` 클로저로 옮김)에는 도움말이 없고, 책의 타입은 `Clone`이 아닙니다. 1.100.0에서도 `Clone`이 아닌 타입(`Mutex<u32>`)으로 같은 모양을 만들면 도움말이 없었습니다. 그런데 상태 타입이 `Clone`을 derive했다면 ``help: consider cloning the value before moving it into the closure``가 나오고 `let value = tally.clone();`을 보여 줍니다. 공유하려던 상태를 복제하면 스레드마다 따로 고치게 되어, 컴파일은 되지만 결과가 갈라집니다. 공유가 목적이라면 `Arc<Mutex<T>>`나 지침 3의 설계로 갑니다. 책의 기준 버전에서 `Clone` 타입에 이 도움말이 나왔는지는 확인하지 못했습니다.
- **E0373은 그대로입니다.** ``closure may outlive the current function``, `note: function requires argument type to outlive 'static`, `move`를 붙이라는 `help:`가 책과 같은 모양으로 나옵니다(Part 0 ch15 4.1절의 파일로 확인).
- **2024 에디션의 `if let`.** 조건식에서 만든 가드가 `else`에 들어가기 전에 풀립니다([Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.4절). `Mutex<BTreeMap>`으로 확인한 결과입니다.

| 모양 | `else`/`None` 갈래에서 `try_lock()` (2024) | (2021) |
| --- | --- | --- |
| `if let Some(n) = m.lock().unwrap().get(&301) { .. } else { .. }` | 성공 | 실패 |
| `match m.lock().unwrap().get(&301) { Some(..) => .., None => .. }` | 실패(가드가 `match` 끝까지 삶) | 실패 |
| `let found = m.lock().unwrap().get(&301).cloned();` 뒤 | 성공 | 성공 |

에디션과 상관없이 잠금이 풀리는 곳이 뻔한 것은 셋째 모양입니다.

- **스코프 스레드.** `thread::scope`는 1.63.0에서 안정화되어 책의 기준 버전에도 있었지만 책은 다루지 않습니다. 스레드가 스코프 안에서 끝난다면 `Arc` 없이 빌려 쓸 수 있습니다(2.1절의 표). 멈출 수 있는 스레드에는 쓸 수 없다는 점도 함께 기억합니다.
- **nightly 전용인 것.** 가드를 내주지 않고 값을 넣고 빼는 `Mutex::set`·`replace`·`get_cloned`는 1.100.0에서 `lock_value_accessors` 기능이 필요합니다(E0658으로 확인). 같은 스레드가 다시 잠글 수 있는 `ReentrantLock`, 포이즈닝이 없는 `std::sync::nonpoison`도 로컬 문서에서 nightly 전용으로 표시됩니다.
- **책에 없는 지침.** 다음은 책의 Item 17이 다루지 않는 내용입니다(2026-11-19에 온라인판에서 확인).
  - 2.1절의 "스레드에 데이터를 넘기는 길" 표: `thread::spawn`·`thread::scope` 문서, ch8 2.6절과 ch14 2.7절이 근거입니다.
  - 2.2절 "따르지 않아도 되는 경우"의 `RwLock`과 원자 타입: Part 1 ch11 2.5·2.8절이 근거입니다.
  - 2.3절의 "잠금을 쥔 채 기다리지 않는다"와 조건식의 가드: Part 1 ch11 2.4절, `Condvar::wait` 문서, 위 표의 실행 결과가 근거입니다.
  - 2.4절의 채널판 비용(채널로 생기는 교착 상태, 배압): 직접 실행한 결과와 Part 1 ch11 2.7절이 근거입니다.
  - 2.5절의 순위 검사(`RankedMutex`)와 제한 시간을 둔 테스트: 책은 "잠금 순서를 정하고 감시하라"와 "검출 도구를 CI에 넣으라"까지 권합니다. 구현은 이 챕터가 만든 것이고, 예제 05·01·03의 실행 결과가 근거입니다.
- **달라지지 않은 점.** Rust는 여전히 교착 상태를 막지 않고, 표준 라이브러리에 교착 상태 검출 기능도 없습니다. `lock()`은 포이즈닝 때문에 `Result`를 돌려주고, 예제처럼 `unwrap()`하는 관례도 책과 같습니다. 포이즈닝을 언제 복구할지는 Part 1 ch11 2.4절, 패닉을 피하는 일반론은 [`ch18-dont-panic`](../ch18-dont-panic/README.md)에 있습니다.

### 2.7 기억할 것

- 안전한 Rust는 데이터 경쟁을 컴파일할 때 막습니다. `Send`/`Sync`는 필드에서 오므로, 필드 하나가 타입 전체의 스레드 안전성을 바꿉니다.
- 교착 상태와 어긋난 상태는 실행 순서의 문제라서 컴파일러도 보통의 테스트도 잡지 못합니다.
- 함께 맞아야 하는 데이터는 잠금 하나로 묶습니다. 잠금 범위를 좁혀 교착 상태를 피하면 데이터가 어긋납니다.
- 잠금은 도우미 메서드 안에서만 잡고 풉니다. 잠금을 쥔 채 클로저를 부르거나 가드를 돌려주지 않습니다.
- 가능하면 한 스레드가 데이터를 소유하고 채널로 요청을 받습니다. 잠금 순서는 마지막 수단이고, 정했으면 검사합니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행하며, 모두 종료 코드 0으로 끝나고 출력은 실행마다 같습니다. 예제마다 30번씩 실행해 출력이 같은 것을 확인했고, 짧은 기다림에 기대는 01~03은 8개를 동시에 띄우는 실행을 3번 더 했습니다. 스레드 안에서는 출력하지 않고 결과를 `main`으로 돌려받아 출력합니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 절 |
| --- | --- | --- | --- | --- |
| [`01_lock_inversion.rs`](examples/01_lock_inversion.rs) | 1 | 잠금 둘, 반대 순서 | 잠금 하나 (대안: 같은 순서) | 2.2 |
| [`02_split_critical_section.rs`](examples/02_split_critical_section.rs) | 1 | 잠금 둘, 범위를 좁힘 | 잠금 하나 | 2.2 |
| [`03_keep_locks_inside.rs`](examples/03_keep_locks_inside.rs) | 2 | 잠금을 쥔 채 클로저 호출, 가드 반환 | 사본을 만들고 푼 뒤 호출, 값 반환 | 2.3 |
| [`04_channel_redesign.rs`](examples/04_channel_redesign.rs) | 3 | (공유 상태 판은 01·02) | 프런트 스레드가 소유, 채널로 요청 | 2.4 |
| [`05_checked_lock_order.rs`](examples/05_checked_lock_order.rs) | 4 | 주석으로만 정한 순서 | 순위를 붙인 잠금 | 2.5 |

### 01_lock_inversion — 잠금 순서 뒤집힘과 잠금 하나

세 판 모두 같은 관찰 장치(`Meeting`)를 씁니다. 각 작업은 첫 잠금을 쥔 순간에 상대도 첫 잠금을 쥘 때까지 최대 100ms 기다립니다. 상대가 잠금을 기다리느라 오지 못하면 혼자 진행합니다. 비권장 판의 두 스레드는 끝나지 않으므로 `main`은 1초만 기다리고 넘어갑니다. 멈춘 스레드는 `main`이 끝날 때 프로세스와 함께 사라집니다. 스코프 스레드였다면 스코프가 멈춘 스레드를 영원히 기다려 `main`도 멈췄을 것이므로 `thread::spawn`과 `Arc`를 씁니다. 실행에는 1.5초쯤 걸립니다.

```sh
cargo run -p er-ch17-shared-state-parallelism --example 01_lock_inversion
```

```text
처음: 장부 {"하준": 301}, 배정표 {301: "하준"}
동시에 실행: check_in("서아", 302)와 close_room(301)
1. 비권장: 잠금 둘, check_in은 장부 → 배정표, close_room은 배정표 → 장부
    두 작업이 동시에 첫 잠금을 쥠: 예
    1초 안에 끝난 작업: 0/2 → 교착 상태
    지금: 장부 잠김, 배정표 잠김
2. 권장: 장부와 배정표를 잠금 하나로 묶음
    두 작업이 동시에 첫 잠금을 쥠: 아니오
    1초 안에 끝난 작업: 2/2 → 모두 끝남
    지금: 장부 {"서아": 302}, 배정표 {302: "서아"}
3. 대안: 잠금 둘, 모든 작업이 장부 → 배정표 순서
    두 작업이 동시에 첫 잠금을 쥠: 아니오
    1초 안에 끝난 작업: 2/2 → 모두 끝남
    지금: 장부 {"서아": 302}, 배정표 {302: "서아"}
```

"지금" 줄은 `try_lock`으로 기다리지 않고 본 상태입니다. 비권장 판의 두 잠금은 멈춘 스레드가 쥐고 있습니다.

### 02_split_critical_section — 범위를 좁힌 잠금 둘과 잠금 하나

관찰 장치(`Gap`)는 지정한 작업이 두 단계 사이에 이르면 다른 작업을 출발시키고, 그 작업이 끝나기를 최대 100ms 기다립니다. 잠금 하나 판에서는 다른 작업이 잠금을 기다리느라 끝날 수 없으므로 100ms 뒤에 지정한 작업이 마저 진행합니다. 이 예제의 스레드는 멈추지 않으므로 스코프 스레드를 씁니다.

```sh
cargo run -p er-ch17-shared-state-parallelism --example 02_split_critical_section
```

```text
처음: 장부 {"하준": 301}, 배정표 {301: "하준"}
1. check_in("서아", 302)의 두 단계 사이에 현황판이 두 표를 읽음
  [비권장] 잠금 둘, 범위를 좁혀 한 번에 하나만 잠금
    현황판이 본 것: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준"} → 서로 맞지 않음
    끝난 뒤: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아"} → 서로 맞음
  [권장] 잠금 하나
    현황판이 본 것: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아"} → 서로 맞음
    끝난 뒤: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아"} → 서로 맞음
2. close_room(301)의 두 단계 사이에 check_in("서아", 301)이 끼어듦
  [비권장] 잠금 둘, 범위를 좁혀 한 번에 하나만 잠금
    끝난 뒤: 장부 {}, 배정표 {301: "서아"} → 서로 맞지 않음
  [권장] 잠금 하나
    끝난 뒤: 장부 {"서아": 301}, 배정표 {301: "서아"} → 서로 맞음
```

비권장 판의 `close_room`은 01처럼 배정표부터, `check_in`은 장부부터 고치지만 두 잠금을 동시에 쥐지 않으므로 교착 상태는 없습니다. "서로 맞음"은 장부의 모든 투숙객이 배정표의 그 객실에 있고 배정표에 장부에 없는 사람이 없다는 뜻입니다.

### 03_keep_locks_inside — 잠금을 도우미 메서드 안에 가두기

네 구역 모두 같은 일(하준과 도윤을 체크아웃)을 새 스레드에서 하고, `main`은 300ms만 기다립니다. 비권장 판의 스레드는 같은 스레드가 이미 쥔 잠금을 다시 잠그다 멈춥니다. 이 동작은 문서가 정하지 않은 것이라 다른 플랫폼에서는 패닉할 수도 있습니다.

```sh
cargo run -p er-ch17-shared-state-parallelism --example 03_keep_locks_inside
```

```text
처음: 장부 {"도윤": 303, "서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아", 303: "도윤"}
오늘 떠나는 손님: ["하준", "도윤"]
1. 비권장: for_each_guest_locked가 잠금을 쥔 채 클로저를 부름
    300ms 안에 끝남: 아니오 → check_out이 같은 스레드가 쥔 잠금을 기다림
    지금: 잠김 (잠금을 쥔 스레드가 멈춰 있음)
2. 비권장: ledger()가 돌려준 MutexGuard를 쥔 채 check_out을 부름
    300ms 안에 끝남: 아니오 → check_out이 같은 스레드가 쥔 잠금을 기다림
    지금: 잠김 (잠금을 쥔 스레드가 멈춰 있음)
3. 권장: for_each_guest가 사본을 만들고 잠금을 푼 뒤 클로저를 부름
    300ms 안에 끝남: 예, 비운 객실 [303, 301]
    지금: 장부 {"서아": 302}, 배정표 {302: "서아"}
4. 권장: guests()가 가드 대신 사본을 돌려줌
    300ms 안에 끝남: 예, 비운 객실 [303, 301]
    지금: 장부 {"서아": 302}, 배정표 {302: "서아"}
```

비운 객실이 `[303, 301]`인 것은 장부(`BTreeMap`)를 이름 순서(도윤, 서아, 하준)로 돌기 때문입니다.

### 04_channel_redesign — 채널로 다시 설계

프런트 스레드는 `thread::spawn`으로 띄우고 장부를 옮겨 줍니다. 손님 스레드들은 스코프 스레드이고 `&FrontHandle`을 함께 씁니다(`Sender`는 `Sync`입니다, Part 1 ch11 2.7절). 1번 구역은 `Barrier`로 두 요청을 같은 순간에 출발시킵니다.

```sh
cargo run -p er-ch17-shared-state-parallelism --example 04_channel_redesign
```

```text
1. 01의 두 작업을 두 스레드가 같은 순간에 요청
    처음: 장부 {"하준": 301}, 배정표 {301: "하준"} → 서로 맞음
    close_room(301)이 내보낸 손님: ["하준"]
    지금: 장부 {"서아": 302}, 배정표 {302: "서아"} → 서로 맞음
2. 체크인과 점검을 500번씩 되풀이하는 동안 현황판이 500번 읽음
    현황판이 어긋난 상태를 본 횟수: 0/500
    지금: 장부 {"서아": 302}, 배정표 {302: "서아"} → 서로 맞음
3. 송신자를 모두 버리면 프런트 스레드가 끝나며 장부를 돌려줌
    처리한 요청: 2505개
    돌려받은 장부: 장부 {"서아": 302}, 배정표 {302: "서아"} → 서로 맞음
```

처리한 요청 2505개는 1번 구역의 4개(조회 2, 체크인 1, 점검 1)와 2번 구역의 2501개(체크인·점검 2000, 현황판 500, 조회 1)입니다. 요청을 처리하는 순서는 실행마다 다를 수 있지만, 두 손님 스레드가 서로 다른 객실을 다루고 현황판은 언제나 맞는 상태만 보므로 출력은 같습니다. `drop(handle)`을 빼면 3번 구역의 `join`이 영원히 기다립니다(Part 0 ch15 2.6절).

### 05_checked_lock_order — 잠금 순서를 검사하기

스레드를 쓰지 않습니다. 2번 구역의 `close_room`은 배정표를 잠근 뒤 장부에서 `Err`를 받고 돌아오므로, 배정표도 고치지 않은 채 끝납니다("지금" 줄에 하준이 그대로 있음). 3번 구역 끝의 "잠금 기록 0개"는 모든 `RankedGuard`의 `drop`이 기록을 지웠다는 확인입니다.

```sh
cargo run -p er-ch17-shared-state-parallelism --example 05_checked_lock_order
```

```text
1. 비권장: 잠금 순서를 주석으로만 정함 (장부 → 배정표)
    check_in, close_room 차례로 실행: 모두 끝남 → 테스트 통과
    장부 {"서아": 302}, 배정표 {302: "서아"}
2. 권장: 잠금마다 순위를 붙이고 잠글 때 검사
    check_in("서아", 302): Ok
    close_room(301): Err(배정표(2)를 쥔 채 장부(1)를 잠그려 함)
    지금: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아"}
3. 고친 close_room: 장부부터 잠금
    close_room(301): Ok
    지금: 장부 {"서아": 302}, 배정표 {302: "서아"}
    이 스레드가 쥔 잠금 기록: 0개
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 본편에 없던, 데이터 경쟁을 막는 두 경우를 골랐습니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다). 에러 메시지는 앞부분만 옮겼고, 표준 라이브러리 소스 위치(`/rustc/...`)를 가리키는 뒤쪽 `note:`는 생략했습니다.

```sh
cd appd-a/ch17-shared-state-parallelism
rustc --edition 2024 compile_fail/e0277_cell_field_not_sync.rs --out-dir /tmp/er-ch17
rustc --edition 2024 compile_fail/e0502_scoped_reader_and_writer.rs --out-dir /tmp/er-ch17
```

### 4.1 `Cell` 필드가 있는 구조체를 `Arc`로 공유 — E0277(`Sync`)

파일: [`compile_fail/e0277_cell_field_not_sync.rs`](compile_fail/e0277_cell_field_not_sync.rs)

```text
error[E0277]: `Cell<u32>` cannot be shared between threads safely
  --> compile_fail/e0277_cell_field_not_sync.rs:37:31
   |
37 |     let lobby = thread::spawn(move || for_lobby.vacant()); // E0277
   |                 ------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^ `Cell<u32>` cannot be shared between threads safely
   |                 |
   |                 required by a bound introduced by this call
   |
   = help: within `RoomBoard`, the trait `Sync` is not implemented for `Cell<u32>`
   = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` or `std::sync::atomic::AtomicU32` instead
note: required because it appears within the type `RoomBoard`
  --> compile_fail/e0277_cell_field_not_sync.rs:20:5
   |
18 | struct RoomBoard {
19 |     floors: Vec<u32>,  // 층마다 빈 객실 수
20 |     lookups: Cell<u32>, // 조회 횟수: &self 메서드에서 고치려고 Cell을 씀
   |     ^^^^^^^^^^^^^^^^^^ required by this field
   = note: required for `Arc<RoomBoard>` to implement `Send`
note: required because `for_lobby` is used within this closure
```

- **읽는 법**: 연결 고리를 아래에서 위로 읽습니다. 클로저가 `for_lobby`(`Arc<RoomBoard>`)를 캡처했고, `Arc<RoomBoard>`가 `Send`이려면 `RoomBoard`가 `Sync`여야 하는데, `lookups` 필드의 `Cell<u32>`가 `Sync`가 아닙니다. ``required by this field``가 원인인 필드를 가리킵니다. Part 0 ch15 4.5절의 `Arc<RefCell<T>>`와 같은 에러이지만, 여기서는 `Cell`이 사용자 정의 타입 안에 숨어 있습니다.
- **원인**: `Cell`은 원자적이지 않은 읽기와 쓰기로 값을 바꿉니다. 두 스레드가 `vacant()`를 동시에 부르면 둘 다 같은 값을 읽고 같은 값을 써서 조회 하나가 사라집니다. 자동 트레이트는 이 위험을 필드에서 구조체로, 구조체에서 `Arc`로 전해 줍니다(2.1절).
- **해결**: `note:`의 제안대로 그 필드만 `AtomicU32`로 바꾸고 `fetch_add(1, Ordering::Relaxed)`로 셉니다(같은 파일을 고쳐 `빈 객실 8개`, `조회 1번`을 확인). 여러 필드를 함께 고쳐야 한다면 그 필드들을 묶어 `Mutex`로 감쌉니다(지침 1). 타입을 만든 쪽이라면 [ch10](../ch10-standard-traits/README.md) 2.9절의 `assert_send_sync` 검사로 필드를 바꾸는 순간 알아챕니다.

### 4.2 스코프 스레드 하나는 쓰고 하나는 읽음 — E0502

파일: [`compile_fail/e0502_scoped_reader_and_writer.rs`](compile_fail/e0502_scoped_reader_and_writer.rs)

```text
error[E0502]: cannot borrow `occupied` as immutable because it is also borrowed as mutable
  --> compile_fail/e0502_scoped_reader_and_writer.rs:20:17
   |
18 |     thread::scope(|s| {
   |                    - has type `&'1 Scope<'1, '_>`
19 |         s.spawn(|| occupied += 1); // 체크인: &mut occupied
   |         -------------------------
   |         |       |  |
   |         |       |  first borrow occurs due to use of `occupied` in closure
   |         |       mutable borrow occurs here
   |         argument requires that `occupied` is borrowed for `'1`
20 |         s.spawn(|| println!("현황판: 객실 {occupied}개 사용 중")); // E0502: &occupied
   |                 ^^                         -------- second borrow occurs due to use of `occupied` in closure
   |                 |
   |                 immutable borrow occurs here
```

- **읽는 법**: [ch15](../ch15-borrow-checker/README.md) 2.2절의 세 표시 중 "그 뒤에 쓰는 곳"이 없고, 대신 ``argument requires that `occupied` is borrowed for `'1` ``가 있습니다. 첫 스레드의 가변 대여가 스코프(`'1`) 내내 이어진다는 뜻입니다. 스레드가 스코프 끝까지 살 수 있기 때문입니다.
- **원인**: 데이터 경쟁의 "적어도 하나는 쓴다" 조건 그대로입니다. 두 스레드가 동시에 살아 있는 동안 한쪽이 쓰면, 다른 쪽은 읽기만 하더라도 반쯤 바뀐 값을 볼 수 있습니다. 둘 다 쓰는 경우는 Part 1 ch11 4.1절의 E0499입니다.
- **해결**: 읽기를 스코프가 끝난 뒤로 옮기는 것이 가장 단순합니다. 정말 함께 해야 한다면 `Mutex<u32>`(값 하나라면 `AtomicU32`)로 감싸 두 스레드가 `&`로 씁니다. 스코프 스레드이므로 `Arc`는 필요 없습니다. `Mutex`로 고친 코드는 컴파일되지만, 현황판이 보는 값(12 또는 13)은 어느 스레드가 먼저 잠그느냐에 따라 달라질 수 있습니다. 데이터 경쟁은 막아도 실행 순서는 정해 주지 않는다는 것이 이 챕터의 주제입니다.

### 4.3 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| `Clone`인 상태 값을 두 `spawn` 클로저로 옮김 | E0382, `help:`가 `clone()`을 제안 | 따르면 스레드마다 사본을 고침. 공유가 목적이면 `Arc<Mutex<T>>`나 채널 (2.6절) |
| `Arc<Vec<T>>`의 `push` | E0596 ``cannot borrow data in an `Arc` as mutable`` | 고칠 값은 `Mutex`로 감쌈 (Part 0 ch14 4.5절의 `Rc`판과 같음) |
| 두 작업이 잠금 둘을 반대 순서로 잡음 | 경고 없음. 운 나쁜 순서에서 교착 상태 | 잠금 하나로 묶음 (2.2절), 순서를 검사 (2.5절) |
| 잠금 범위를 좁혀 한 작업을 두 임계 구역으로 나눔 | 경고 없음. 두 데이터가 어긋남 | 잠금 하나 (2.2절) |
| 잠금을 쥔 채 클로저를 부르거나 가드를 쥔 채 같은 타입의 메서드를 부름 | 경고 없음. 같은 스레드의 재잠금(이 환경에서는 멈춤) | 도우미 메서드 안에서만 잠금 (2.3절) |
| `match m.lock().unwrap().get(..)`의 `None` 갈래에서 다시 잠금 | 경고 없음. 가드가 `match` 끝까지 살아 있음 | `let`으로 값을 먼저 꺼냄 (2.6절의 표) |
| `let _ = m.lock().unwrap();`로 "잠가 둔다" | 컴파일 에러 (rustc `let_underscore_lock`, 기본 deny) | `let _guard = ..` ([Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 4.5절) |
| 요청을 처리하는 스레드가 다른 스레드의 답을 기다림 | 서로 기다리면 채널로도 교착 상태 | 처리 스레드는 다른 스레드를 기다리지 않게 설계 (2.4절) |

## 5. 연습 문제

### 연습 1. 객실 변경을 한 임계 구역으로

[`03_keep_locks_inside.rs`](examples/03_keep_locks_inside.rs)의 `Front`에 `fn move_guest(&self, guest: &str, to: u32) -> Result<u32, MoveError>`를 추가하세요. 손님을 지금 객실에서 `to`로 옮기고 원래 객실 번호를 돌려줍니다. 손님이 없거나 `to`에 다른 손님이 있으면 아무것도 바꾸지 않고 `Err`를 돌려줍니다. 부르는 쪽이 `check_out`과 `check_in`을 차례로 부르는 방식과 무엇이 다른지도 설명해 보세요.

<details>
<summary>힌트</summary>

잠금 한 번으로 확인(손님이 있는가, `to`가 비었는가)부터 두 표의 수정까지 모두 합니다. `ledger.rooms.contains_key(&to)`로 빈 객실인지 보고, `ledger.guests.get_mut(guest)`로 받은 `&mut u32`를 `mem::replace(room, to)`로 바꿔 옛 객실을 얻고, 배정표에서는 옛 객실을 `remove`한 뒤 `to`에 넣습니다. 이때 가드를 그대로 쓰면 `get_mut`의 결과를 쥔 채 `ledger.rooms`에 닿는 곳이 E0499입니다. 필드에 닿을 때마다 가드의 `DerefMut`이 가드 전체를 빌리기 때문입니다. `let mut guard = self.ledger.lock().unwrap(); let ledger = &mut *guard;`로 한 번 역참조해 두면 두 필드를 나눠 빌릴 수 있습니다([ch15](../ch15-borrow-checker/README.md) 2.6절). 부르는 쪽이 `check_out` 뒤에 `check_in`을 부르면 그 사이의 틈에 다른 스레드가 빈 `to`를 먼저 차지하거나, 현황판이 "어디에도 없는 손님"을 볼 수 있습니다. 예제 02의 비권장 판과 같은 문제를 부르는 쪽에서 만드는 것입니다.

</details>

### 연습 2. 채널판의 객실 변경

[`04_channel_redesign.rs`](examples/04_channel_redesign.rs)에 `Request::Move { guest, to, reply }`를 추가하고, `FrontHandle::move_guest`를 만드세요. 2번 구역 앞에서 하린을 403호에 체크인하고, 2번 구역에 "하린을 404호와 403호 사이에서 500번 옮기는" 스레드를 더해도 현황판이 어긋난 상태를 0번 보는지 확인하세요.

<details>
<summary>힌트</summary>

`Ledger`에 연습 1과 같은 로직의 `fn move_guest(&mut self, guest: &str, to: u32) -> Result<u32, String>`을 두고, `run_front`의 `match`에 갈래 하나를 더합니다. 잠금이 없으니 `&mut self` 메서드 하나가 그대로 임계 구역이고, 가드를 거치지 않으므로 연습 1의 `&mut *guard` 같은 처리도 필요 없습니다. 403호와 404호는 다른 스레드가 쓰지 않으므로 이동은 매번 성공하고 출력도 결정적입니다. 이 툴체인에서 같은 수정을 해 보니 어긋난 상태 0/500, 처리한 요청 3006개(체크인 1과 이동 500이 늘어남)였고, 다섯 번 실행한 출력이 같았습니다.

</details>

### 연습 3. 같은 타입 두 값 사이의 순서

[`05_checked_lock_order.rs`](examples/05_checked_lock_order.rs)의 `RankedMutex`는 같은 순위의 잠금을 함께 쥐는 것을 모두 거부합니다. 호텔 두 곳(`CheckedFront` 두 개) 사이에서 손님을 옮기려면 두 장부(둘 다 순위 1)를 함께 잠가야 합니다. 순위를 `(단계, id)` 쌍으로 바꾸고, 두 호텔의 장부를 id가 작은 쪽부터 잠그는 `transfer` 함수를 만드세요. 반대 순서로 잠그면 `Err`가 나야 합니다.

<details>
<summary>힌트</summary>

`rank: u32`를 `rank: (u32, u32)`로 바꾸면 튜플의 비교가 단계를 먼저, 같으면 id를 봅니다. `HELD`의 원소와 `OrderError`의 필드도 `((u32, u32), &'static str)`가 되고, `Display`에서는 순위를 `{hr:?}`로 출력합니다. `transfer(a, b, ..)`에서는 `let (first, second) = if a.id < b.id { (a, b) } else { (b, a) };`로 순서를 정한 뒤 `first.guests.lock()?`, `second.guests.lock()?` 순서로 잠급니다(Part 1 ch11 2.4절의 이체와 같은 방법). 두 호텔의 배정표(단계 2)도 필요하면 장부 둘을 먼저 잠근 뒤 같은 id 순서로 잠급니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 17: Be wary of shared-state parallelism](https://effective-rust.com/deadlock.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- 표준 라이브러리 문서
  - [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)(`lock`의 설명과 Panics 절: 같은 스레드가 다시 잠글 때), [`MutexGuard`](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html), [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html), [`Condvar`](https://doc.rust-lang.org/std/sync/struct.Condvar.html), [`Barrier`](https://doc.rust-lang.org/std/sync/struct.Barrier.html)
  - [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html)(`recv_timeout`, `sync_channel`), [`thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html), [`thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html), [`thread_local!`](https://doc.rust-lang.org/std/macro.thread_local.html)
  - [`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html), [`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html), [`Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html), [`AtomicU32`](https://doc.rust-lang.org/std/sync/atomic/type.AtomicU32.html)
- 교착 상태 검출 도구(이름과 버전, 문서만 확인): [`no_deadlocks`](https://crates.io/crates/no_deadlocks), [`parking_lot::deadlock`](https://docs.rs/parking_lot/0.13.0/parking_lot/deadlock/index.html), ThreadSanitizer는 [The Unstable Book의 sanitizer 절](https://doc.rust-lang.org/unstable-book/compiler-flags/sanitizer.html)
- clippy 린트: [`significant_drop_tightening`](https://rust-lang.github.io/rust-clippy/master/index.html#significant_drop_tightening)(nursery, 기본 꺼짐)
- 에러 설명: `rustc --explain E0277`, `E0373`, `E0382`, `E0499`, `E0502`, `E0596`
