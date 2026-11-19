# ch2. 공통 동작을 타입 시스템으로 표현하기

함수에 "동작"을 넘기는 방법은 여럿입니다. 함수 포인터, 클로저, 트레이트 바운드를 건 제네릭, 트레이트 객체가 모두 같은 일을 하는 것처럼 보이고, 본편에서 각각의 문법도 배웠습니다. 이 챕터는 문법을 다시 설명하지 않습니다. 동작을 받는 매개변수를 **어떤 타입으로 선언할지** 고르는 판단만 다룹니다.

다섯 지침이 모두 같은 원리에서 나옵니다. 함수의 시그니처는 부르는 쪽과 맺는 약속이고, 받는 쪽이 필요 이상으로 좁게 선언하면 그 비용은 부르는 쪽이 치릅니다. 우회 코드를 쓰거나, 틀린 결과를 받거나, 아예 부르지 못합니다. 반대로 시그니처만으로는 드러나지 않는 조건이 있다면 그 조건을 타입으로 드러내야 합니다.

- 원제: [Item 2: Use the type system to express common behavior](https://effective-rust.com/use-types-2.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch2-express-common-behavior`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 동작을 받는 매개변수를 `fn` 포인터와 `Fn*` 바운드 중 무엇으로 선언할지 고를 수 있고, `fn` 포인터가 어울리는 경우를 설명할 수 있다.
2. 함수 본문이 콜백을 부르는 방식을 보고 가장 느슨한 바운드(`FnOnce`, `FnMut`, `Fn`)를 고를 수 있고, `Fn`이 꼭 필요한 경우를 설명할 수 있다.
3. 구체 타입 대신 필요한 동작만 묶은 트레이트로 매개변수를 받는 시그니처를 작성할 수 있고, 그렇게까지 하지 않아도 되는 경우를 판단할 수 있다.
4. 메서드 시그니처로 드러나지 않는 약속을 마커 트레이트로 표현하고 바운드로 요구할 수 있다.
5. 타입이 다른 구현을 섞어 받거나 실행 중에 골라야 할 때 트레이트 객체 매개변수를 쓸 수 있고, 제네릭 함수에 `?Sized`를 붙여 트레이트 객체도 받게 할 수 있다.

## 2. 핵심 개념

### 2.1 동작을 받는 다섯 가지 모양

| 받는 모양 | 시그니처 예 | 넘길 수 있는 것 | 문법은 |
| --- | --- | --- | --- |
| 함수 포인터(function pointer) | `f: fn(i32) -> bool` | 함수, 캡처하지 않는 클로저 | [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.16절 |
| `Fn*` 바운드 | `f: impl FnMut(i32) -> bool` | 함수, 함수 포인터, 바운드에 맞는 모든 클로저 | [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.5~2.6절 |
| 트레이트 바운드 | `s: &mut impl Storage`, `<S: Storage>` | 트레이트를 구현한 타입 하나 | [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.9절 |
| 트레이트 객체(trait object) | `s: &dyn Storage`, `&[Box<dyn Storage>]` | 트레이트를 구현한 타입 아무것이나, 섞어서 | [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 2.4~2.7절 |
| 마커 트레이트(marker trait) 바운드 | `<C: Idempotent>` | 약속을 표시한 타입만 | 이 챕터 2.5절 |

메서드도 같은 질문에 답합니다. 메서드는 수신자(`&self`, `&mut self`, `self`)로 "이 메서드가 자기 값에 무엇을 하는가"를 시그니처에 드러냅니다([Part 0 ch5](../../part0-getting-started/ch5-structs/README.md)). 이 세 수신자는 2.3절의 `Fn`, `FnMut`, `FnOnce`와 그대로 대응합니다.

### 2.2 지침 1: `fn` 포인터보다 `Fn*` 바운드로 받는다

**문제 상황.** 측정값 목록에서 경보 조건을 처음 만족하는 위치를 찾는 함수를 만듭니다. 조건은 부르는 쪽이 정하므로 동작을 매개변수로 받아야 합니다. Part 0 ch19에서 함수를 `fn(i32) -> bool` 타입으로 넘길 수 있다고 배웠으니 그렇게 선언하고 싶어집니다.

**비권장 코드.**

```rust
fn first_alert_ptr(readings: &[i32], is_alert: fn(i32) -> bool) -> Option<usize> {
    readings.iter().position(|&t| is_alert(t))
}
```

부르는 쪽이 "그날 평균보다 5도 이상 높으면 경보"처럼 실행 중에 계산한 값을 기준으로 쓰려 하면 막힙니다. `|t| t >= limit`는 `limit`을 캡처하는데, 함수 포인터에는 코드의 주소만 들어가므로 캡처한 값을 함께 넘길 자리가 없습니다(E0308, 4.1절). 부르는 쪽에 남는 방법은 기준값을 코드에 박아 넣은 클로저, 미리 써 둔 함수, 전역 변수뿐입니다. [`01_fn_pointer_vs_fn_bound.rs`](examples/01_fn_pointer_vs_fn_bound.rs)에서 미리 써 둔 `is_warm`(20도 이상)으로 대신하면, 그날의 기준(16도)과 달라서 17도를 놓치고 "경보 없음"이 나옵니다. 컴파일은 되지만 의도와 다른 결과입니다.

**권장 코드.**

```rust
fn first_alert(readings: &[i32], mut is_alert: impl FnMut(i32) -> bool) -> Option<usize> {
    readings.iter().position(|&t| is_alert(t))
}
```

본문은 그대로이고 매개변수 타입만 바뀌었습니다. 이렇게 바꿔도 잃는 것이 없다는 점이 중요합니다. 함수와 함수 포인터도 `Fn`, `FnMut`, `FnOnce`를 모두 구현하므로, 비권장 판에 넘길 수 있던 것은 권장 판에도 모두 넘길 수 있습니다. 받을 수 있는 범위가 넓어지기만 합니다. `impl Trait` 대신 `<F: FnMut(i32) -> bool>`처럼 제네릭 바운드로 써도 같습니다. `Fn`이 아니라 `FnMut`을 고른 이유는 다음 절에서 설명합니다.

**지침을 따르지 않아도 되는 경우.** 함수 포인터는 이름을 적을 수 있는 구체 타입이고, `Copy`이며, 크기가 포인터 하나로 정해져 있습니다. 이 성질이 필요할 때는 함수 포인터가 맞습니다.

- **상수 표나 정적 변수에 동작을 담을 때.** 클로저의 타입은 이름을 적을 수 없으므로 `const` 배열의 원소 타입으로 쓸 수 없습니다. 예제 01은 `type Rule = fn(i32) -> bool;`로 `const PRESETS: [(&str, Rule); 2]` 표를 만듭니다.
- **구조체 필드에 캡처 없는 동작을 담으면서 타입 매개변수를 늘리고 싶지 않을 때.**
- **C 코드에 콜백을 넘길 때.** C 함수가 받는 콜백은 코드 주소 하나라서 Rust 쪽에서도 함수 포인터로 넘깁니다(Part 10 `ch12-ffi-beneath-std`).

이때도 동작을 *받는* 함수는 `impl FnMut`으로 두는 편이 낫습니다. 예제 01의 3번 구역처럼 함수 포인터 표에서 꺼낸 값도 그대로 넘길 수 있기 때문입니다.

### 2.3 지침 2: 가장 느슨한 `Fn*` 바운드를 고른다

Part 0 ch13 2.6절에서 "클로저를 어떻게 부를지에 맞춰 필요한 만큼만 요구한다"는 기준을 배웠습니다. 여기서는 같은 기준을 반대 방향에서, 즉 받는 쪽이 지나치게 엄격하게 요구했을 때 부르는 쪽이 치르는 비용으로 봅니다.

| 받는 쪽 본문이 콜백을 부르는 방식 | 고를 바운드 | 부르는 쪽이 넘길 수 있는 클로저 |
| --- | --- | --- |
| 많아야 한 번 | `FnOnce` | 모든 클로저 |
| 여러 번, 차례로 한 곳에서 | `FnMut` | 캡처한 값을 읽거나 바꾸는 클로저 |
| 공유된 채로(`&self` 메서드 안, 여러 스레드에서 동시에) | `Fn` | 캡처한 값을 읽기만 하는 클로저 |

받는 쪽 입장에서 `FnOnce`가 가장 적게 요구하고 `Fn`이 가장 많이 요구합니다. 그래서 "가장 느슨한 바운드"는 표의 위쪽부터 따져서 본문이 허락하는 첫 줄입니다.

**문제 상황.** 단계 목록을 실행하는 함수가 단계마다 `on_step`을, 끝나면 `on_done`을 한 번 부릅니다.

**비권장 코드.** 본문은 권장 판과 같고, 두 콜백 모두에 `Fn`을 요구합니다.

```rust
fn run_job_strict<S, D>(steps: &[&str], on_step: S, on_done: D)
where
    S: Fn(usize, &str),
    D: Fn(usize),
```

부르는 쪽은 로그를 `Vec`에 쌓고, 끝나면 미리 만들어 둔 보고서(`Clone`을 구현하지 않은 `Report`)를 보낼 목록으로 옮기고 싶습니다. 그런데 `Fn` 클로저는 캡처한 값을 바꿀 수도, 밖으로 옮길 수도 없습니다. 그래서 [`02_loosest_fn_bound.rs`](examples/02_loosest_fn_bound.rs)의 1번 구역은 이렇게 돌아갑니다.

```rust
let log = RefCell::new(Vec::new());             // 바꾸려면 내부 가변성이 필요
let draft = Cell::new(Some(Report { /* .. */ })); // 옮기려면 Option에 넣고 take
run_job_strict(
    &steps,
    |i, name| log.borrow_mut().push(format!("{i}. {name}")),
    |n| {
        if let Some(mut report) = draft.take() { /* .. */ } // 두 번째 호출(None)도 생각해야 함
    },
);
```

결과는 같지만 `RefCell`, `Cell`, `Option`은 모두 시그니처 때문에 생긴 우회로입니다(`Cell`과 `RefCell`은 [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md)). 특히 `on_done`은 실제로 한 번만 불리는데도, 시그니처가 `Fn`이라 여러 번 불릴 수 있다고 가정하고 `None`인 경우까지 처리해야 합니다.

컴파일러도 이 상황을 알아봅니다. 바꾸는 클로저를 인자 자리에 바로 쓰면 에러가 받는 쪽 매개변수를 가리키며 ``change this to accept `FnMut` instead of `Fn` ``이라고 알려 줍니다(4.2절). 옮기는 클로저를 쓰면 E0507과 함께 ``consider cloning the value if the performance cost is acceptable``이라는 도움말이 나오는데, 이것은 부르는 쪽에서 복제 비용을 치르라는 제안입니다. 받는 쪽을 고칠 수 있다면 바운드를 `FnOnce`로 바꾸는 것이 맞는 해결입니다.

**권장 코드.**

```rust
fn run_job<S, D>(steps: &[&str], mut on_step: S, on_done: D)
where
    S: FnMut(usize, &str), // 차례로 여러 번 부름
    D: FnOnce(usize),      // 마지막에 한 번만 부름
```

부르는 쪽은 `|i, name| log.push(..)`와 `|n| { let mut report = draft; .. outbox.push(report); }`처럼 평범한 클로저를 넘깁니다. `Fn`을 구현하는 클로저는 `FnMut`과 `FnOnce`도 구현하므로 느슨한 바운드는 엄격한 바운드를 만족하는 클로저도 모두 받고, 예제 02의 3번 구역처럼 읽기만 하는 클로저도 그대로 들어갑니다.

**세 트레이트와 세 수신자.** 표준 라이브러리 문서에서 세 트레이트의 호출 메서드는 `Fn::call(&self, ..)`, `FnMut::call_mut(&mut self, ..)`, `FnOnce::call_once(self, ..)`입니다. 즉 바운드를 고르는 질문은 "본문이 콜백을 `&`로 다루는가, `&mut`으로 다루는가, 소비하는가"와 같습니다. 메서드의 수신자를 고를 때도 같은 원리로, 읽기만 하는 메서드는 `&self`로 받아야 부르는 쪽이 가장 자유롭습니다.

**지침을 따르지 않아도 되는 경우(`Fn`이 맞는 경우).** 본문이 콜백을 공유된 채로 부른다면 `Fn`이 가장 느슨한 바운드입니다.

- **콜백을 필드에 담고 `&self` 메서드에서 부를 때.** 예제 02의 `Gate`가 이 경우입니다. 바운드를 `FnMut`으로 바꾸면 ``error[E0596]: cannot borrow `self.allow` as mutable, as it is behind a `&` reference``가 나고, 컴파일러는 `check(&mut self, ..)`로 바꾸라고 제안합니다. 그 제안을 따르면 이번에는 `check`를 부르는 모든 쪽이 `Gate`를 가변으로 빌려야 하므로, 공유해서 쓰는 검사기라면 `Fn`을 고릅니다.
- **여러 스레드에서 동시에 부를 때.** 이때는 `Fn`에 더해 `Sync`도 필요합니다([Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md)의 스코프 스레드).

한 가지 더 기억할 점이 있습니다. 공개한 함수의 바운드를 나중에 조이면(`FnMut` → `Fn`) 지금까지 잘 컴파일되던 부르는 쪽 코드가 E0525나 E0596으로 깨집니다(4.2절). 처음부터 본문이 허락하는 만큼 느슨하게 정해 둡니다.

### 2.4 지침 3: 구체 타입 대신 트레이트로 받는다

**문제 상황.** 메모들을 저장소에 백업하는 함수가 있습니다. 처음에는 저장소가 디스크뿐이라 디스크 타입을 그대로 받았습니다. 나중에 "실제로 쓰지 않고 무엇이 몇 바이트 쓰일지 미리 보기"가 필요해졌습니다.

**비권장 코드.**

```rust
fn backup_to_disk(disk: &mut Disk, notes: &[(&str, &str)]) -> usize
```

부르는 쪽은 `Disk` 말고는 넘길 것이 없습니다. [`03_trait_over_concrete.rs`](examples/03_trait_over_concrete.rs)의 1번 구역은 미리 보기를 하려다가 `[디스크에 씀]` 줄 두 개, 즉 실제 쓰기를 일으킵니다. `dry_run: bool` 매개변수를 더하는 우회로도 있지만, 그러면 함수 안에 분기가 늘고 다음 요구(다른 저장소, 테스트용 가짜 저장소)가 올 때마다 이 함수를 다시 고쳐야 합니다.

**권장 코드.** 함수가 저장소에 바라는 동작은 "이름과 내용을 받아 저장하고 쓴 바이트 수를 알려 준다" 하나뿐입니다. 그 동작만 트레이트로 묶고, 매개변수는 그 트레이트로 받습니다.

```rust
trait Storage {
    fn save(&mut self, name: &str, body: &str) -> usize;
}

fn backup(storage: &mut impl Storage, notes: &[(&str, &str)]) -> usize
```

이제 부르는 쪽이 직접 `Preview`라는 구현을 만들어 넘길 수 있고, `backup`은 한 줄도 고치지 않습니다. 확인한 뒤에는 같은 함수에 `&mut disk`를 넘겨 실제로 씁니다(3번 구역).

트레이트 바운드는 "이 함수가 매개변수에게 바라는 능력의 목록"입니다. 바운드가 없는 타입 매개변수의 값으로는 옮기거나 버리는 것 말고 할 수 있는 일이 거의 없고(Part 0 ch10 4.1절의 E0369), 바운드를 하나 더할 때마다 쓸 수 있는 능력이 하나 늘어납니다. 그래서 바운드에는 함수가 실제로 쓰는 것만 적습니다. `backup`이 저장소 이름도 출력해야 한다면 그때 `S: Storage + Display`처럼 늘립니다.

**지침을 따르지 않아도 되는 경우.**

- **구현이 하나뿐이고 크레이트 안에서만 쓰는 함수.** 쓰일지 모르는 두 번째 구현을 위해 트레이트를 미리 만들면 정의, `impl`, 바운드처럼 읽을 곳만 늘어납니다. 구체 타입 매개변수를 나중에 `impl Trait`로 바꿔도 크레이트 안의 호출부는 대부분 그대로 컴파일됩니다. 예제 03의 3번 구역에서 `backup(&mut disk, ..)`가 바뀐 시그니처로도 그대로 동작하는 것과 같습니다.
- 반대로 **공개 API**라면 미리 생각할 가치가 있습니다. 트레이트 자체도 공개 약속이 되므로 무엇을 넣을지 신중히 정합니다. 트레이트에 메서드를 나중에 더하는 문제는 `ch13-default-implementations`에서 다룹니다.

### 2.5 지침 4: 시그니처로 말할 수 없는 약속은 마커 트레이트로

**문제 상황.** 응답이 가끔 사라지는 연결로 계정에 명령을 보냅니다. 응답이 사라지면 명령이 적용됐는지 알 수 없으므로, 다시 보내도 안전한 명령만 재시도해야 합니다. 다시 보내도 안전한 명령은 **멱등(idempotent)**, 즉 여러 번 적용해도 한 번 적용한 것과 결과가 같은 명령입니다. 한도를 300으로 맞추는 `SetLimit(300)`은 멱등이고, 500포인트를 적립하는 `AddPoints(500)`은 멱등이 아닙니다. 그런데 두 명령의 메서드 시그니처는 `fn apply(&self, account: &mut Account)`로 똑같아서, 시그니처만으로는 둘을 구별할 수 없습니다.

**비권장 코드.**

```rust
fn send_with_retry_any<C: Command>(link: &mut FlakyLink, command: &C, account: &mut Account) -> u32
```

[`04_marker_trait.rs`](examples/04_marker_trait.rs)의 1번 구역에서 첫 응답이 사라진 연결로 `AddPoints(500)`을 보내면, 첫 전송도 실제로는 적용됐으므로 재시도한 만큼 더 적립되어 `points = 1000`이 됩니다. 컴파일은 되지만 틀린 결과입니다.

**권장 코드.** 메서드가 하나도 없는 트레이트를 만들어, 구현하는 쪽이 "이 명령은 멱등이다"라고 표시하게 합니다. 재시도 함수는 그 표시를 바운드로 요구합니다.

```rust
trait Idempotent: Command {}       // 메서드 없음. 슈퍼트레이트 덕분에 apply는 쓸 수 있음
impl Idempotent for SetLimit {}    // AddPoints에는 붙이지 않음

fn send_with_retry<C: Idempotent>(link: &mut FlakyLink, command: &C, account: &mut Account) -> u32
```

이제 `send_with_retry(&mut link, &AddPoints(500), &mut account)`는 컴파일되지 않습니다(핵심 줄만 옮겼습니다).

```text
error[E0277]: the trait bound `AddPoints: Idempotent` is not satisfied
help: the trait `Idempotent` is not implemented for `AddPoints`
help: the trait `Idempotent` is implemented for `SetLimit`
```

부르는 쪽은 이 에러를 보고 "이 명령은 재시도하면 안 된다"는 사실을 알게 되고, 한 번만 보내는 `send_once`를 골라 실패를 따로 처리합니다(3번 구역).

마커 트레이트의 약속은 컴파일러가 검사하지 못합니다. `impl Idempotent for AddPoints {}`라고 거짓으로 적어도 컴파일됩니다. 그래서 마커 트레이트는 그 타입을 만든 사람이 성질을 확인하고 붙이는 것이고, 문서에 무엇을 약속하는지 분명히 적어 둡니다. 표준 라이브러리에도 같은 방식의 트레이트가 있습니다. `Eq`는 `PartialEq`의 `==`에 "값은 언제나 자기 자신과 같다"는 약속을 더하는 트레이트이고, `HashMap`의 키는 이 약속을 요구합니다. `Send`와 `Sync`는 거짓 약속이 미정의 동작으로 이어지므로 `unsafe impl`로만 직접 구현할 수 있습니다. `Idempotent`는 거짓 약속이 논리 오류에 그치므로 보통 트레이트로 충분합니다. 표준 트레이트 각각을 언제 구현할지는 `ch10-standard-traits`에서 다룹니다.

**지침을 따르지 않아도 되는 경우.**

- **약속이 타입이 아니라 값마다 다를 때.** 같은 타입의 명령 중 일부만 멱등이라면 타입에 붙이는 표시로는 구별할 수 없습니다. `fn is_idempotent(&self) -> bool` 같은 메서드로 실행 중에 확인합니다.
- **성질을 타입 구조로 직접 표현할 수 있을 때.** 컴파일러가 검사할 수 있는 표현이 검사할 수 없는 약속보다 낫습니다. 상태나 불변식을 타입으로 표현하는 방법은 `ch1-express-data-structures`에서 다룹니다.

### 2.6 지침 5: 섞어 받거나 실행 중에 고르려면 트레이트 객체로 받는다

**문제 상황.** 가격에 할인 규칙(퍼센트 할인 `PercentOff`, 정액 할인 `AmountOff`)을 운영자가 정한 순서대로 적용합니다. 규칙의 종류와 순서는 `"amount:3000,percent:10,amount:500"` 같은 설정 문자열에서 실행 중에 읽습니다. 할인은 순서에 따라 결과가 달라집니다.

**비권장 코드.**

```rust
fn apply_rules_generic<R: PriceRule>(price: u32, rules: &[R]) -> u32
```

제네릭 매개변수 `R`은 호출 한 번에 구체 타입 하나로 정해지므로 `&[R]`에는 한 종류의 규칙만 담을 수 있습니다. 부르는 쪽은 규칙을 타입별로 나눠 두 번 부를 수밖에 없고, [`05_trait_object_param.rs`](examples/05_trait_object_param.rs)의 1번 구역에서는 500원 할인이 10% 할인보다 먼저 적용되어 최종 가격이 14850원이 됩니다. 운영자가 의도한 값은 14800원입니다.

**권장 코드.**

```rust
fn apply_rules(price: u32, rules: &[Box<dyn PriceRule>], report: &mut dyn FnMut(&str, u32)) -> u32
```

`Box<dyn PriceRule>`은 어떤 규칙이 들어 있든 같은 타입이므로 한 슬라이스에 섞어 담을 수 있고, 설정에서 읽은 순서가 그대로 지켜집니다. 클로저도 트레이트 객체로 받을 수 있어서, 진행 상황을 알려 줄 콜백을 `&mut dyn FnMut(&str, u32)`로 받았습니다. 부르는 쪽은 `&mut |label, p| { .. }`처럼 클로저의 가변 참조를 넘깁니다.

부르는 쪽이 무엇을 가지고 있는지에 따라 받는 쪽 시그니처의 모양이 달라집니다.

| 부르는 쪽이 가진 것 | 받는 쪽 매개변수 |
| --- | --- |
| 규칙 하나를 빌려줌 | `rule: &dyn PriceRule` |
| 소유한 규칙들의 목록(`Vec<Box<dyn PriceRule>>`) | `rules: &[Box<dyn PriceRule>]` |
| 빌린 규칙들의 목록 | `rules: &[&dyn PriceRule]` |
| 상태를 바꾸는 콜백 | `report: &mut dyn FnMut(&str, u32)` |
| 구체 타입이든 트레이트 객체든 하나 | `<R: PriceRule + ?Sized>(rule: &R)` |

마지막 줄의 `?Sized`는 놓치기 쉽습니다. 타입 매개변수에는 적지 않아도 `Sized` 바운드가 붙어 있어서, `&R`로 참조를 받더라도 `R`이 `dyn PriceRule`이 될 수 없습니다. 그래서 `preview<R: PriceRule>`에 `rules[1].as_ref()`(`&dyn PriceRule`)를 넘기면 E0277입니다(4.3절). `?Sized`로 이 기본 바운드를 풀면 예제 05의 3번 구역처럼 `&AmountOff`와 `&dyn PriceRule`을 모두 받습니다(`?Sized`의 뜻은 [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.15절).

트레이트 객체로 받으려면 트레이트가 **dyn 호환(dyn compatible)** 이어야 합니다. `Self`를 돌려주거나 수신자 말고 다른 매개변수로 `Self`를 받는 메서드, 타입 매개변수가 있는 메서드가 있으면 안 됩니다(규칙은 Part 0 ch17 2.9절). 현재 툴체인은 세 경우를 이렇게 알려 줍니다(세 메서드를 가진 트레이트 `Mergeable`로 확인했고, 핵심 줄만 옮겼습니다).

```text
error[E0038]: the trait `Mergeable` is not dyn compatible
  ...because method `merge` references the `Self` type in this parameter
  ...because method `similar` references the `Self` type in this parameter
  ...because method `show` has generic type parameters
```

트레이트 객체로 받을 생각이 있는 트레이트라면 처음부터 이 조건을 지켜 설계합니다. `PriceRule`의 두 메서드는 수신자가 `&self`이고 시그니처에 `Self`나 타입 매개변수가 없으므로 dyn 호환입니다.

**지침을 따르지 않아도 되는 경우.** 호출 한 번에 한 가지 타입만 다룬다면 제네릭(`impl Trait`, `<T: Trait>`)으로 충분하고, 이것이 기본 선택입니다. 지침 3의 `backup`이 그런 경우입니다. 두 방법의 비용(만들어지는 코드의 크기, 호출 속도)과 dyn 호환 조건의 자세한 비교는 `ch12-generics-vs-trait-objects`에서 다룹니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **object safety → dyn compatible.** 책은 트레이트 객체로 쓸 수 있는 트레이트를 "object safe"라고 부르지만, 현재 컴파일러와 The Reference는 "dyn compatible"이라고 부릅니다(2.6절의 E0038 메시지). 이 저장소는 "dyn 호환"으로 씁니다. 규칙의 내용(`Self`를 돌려주거나 받는 메서드, 제네릭 메서드 금지)은 같습니다.
- **함수 포인터 비교에 경고가 납니다.** 책은 함수 포인터가 `Eq`를 구현해 `==`로 비교할 수 있다고 설명합니다. 지금도 컴파일은 되지만, 1.100.0에서는 기본으로 켜진 린트 `unpredictable_function_pointer_comparisons`가 경고합니다. 같은 함수도 주소가 다를 수 있고, 다른 함수가 합쳐져 주소가 같아질 수도 있어서 비교 결과를 믿을 수 없다는 이유입니다.

  ```text
  warning: function pointer comparisons do not produce meaningful results since their addresses are not guaranteed to be unique
    = note: `#[warn(unpredictable_function_pointer_comparisons)]` on by default
  help: refactor your code, or use `std::ptr::fn_addr_eq` to suppress the lint
  ```

  주소 비교가 정말 필요하면 1.85.0에서 안정화된 `std::ptr::fn_addr_eq`를 씁니다. 이 함수의 결과는 `==`와 같고, "믿기 어려운 비교임을 알고 쓴다"는 의도를 드러내 경고를 없앨 뿐입니다. 함수 이름(함수 아이템) 둘을 바로 비교하면 지금도 E0369이고, E0308과 함께 ``different fn items have unique types, even if their signatures are the same``와 `as fn(i32) -> i32`로 바꾸라는 도움말이 붙습니다.
- **달라지지 않은 점.** 책이 "쓰는 시점에는 불안정하다"고 적은 `Fn` 트레이트 직접 구현은 1.100.0에서도 불안정합니다. `impl Fn<(i32,)> for X`는 ``error[E0183]: manual implementations of `Fn` are experimental``입니다. 그래서 `Fn*` 바운드를 만족하는 것은 여전히 함수, 함수 포인터, 클로저뿐입니다.

### 2.8 기억할 것

- 동작을 받는 매개변수는 `impl FnMut(..)` 같은 `Fn*` 바운드로 선언합니다. 함수 포인터 `fn(..)`는 이름을 적을 수 있는 구체 타입이 필요할 때(상수 표, C 콜백)만 씁니다.
- 바운드는 본문이 콜백을 부르는 방식대로 고릅니다. 한 번이면 `FnOnce`, 차례로 여러 번이면 `FnMut`, 공유된 채로 부르면 `Fn`입니다.
- 함수가 바라는 것이 동작이라면 구체 타입 대신 그 동작만 묶은 트레이트로 받고, 바운드에는 실제로 쓰는 것만 적습니다.
- 시그니처로 드러나지 않는 약속은 마커 트레이트로 표시하고 바운드로 요구합니다. 약속이 사실인지는 구현하는 사람이 책임집니다.
- 타입이 다른 구현을 섞어 받거나 실행 중에 고른다면 트레이트 객체로 받습니다. 제네릭 함수가 트레이트 객체도 받게 하려면 `?Sized`를 붙입니다.

## 3. 예제 안내

| 파일 | 지침 | 비권장 형태 | 권장 형태 |
| --- | --- | --- | --- |
| `01_fn_pointer_vs_fn_bound` | 1 | `fn(i32) -> bool` 매개변수 | `impl FnMut(i32) -> bool` |
| `02_loosest_fn_bound` | 2 | 두 콜백 모두 `Fn` | `FnMut`과 `FnOnce` |
| `03_trait_over_concrete` | 3 | `&mut Disk` | `&mut impl Storage` |
| `04_marker_trait` | 4 | `C: Command` | `C: Idempotent` |
| `05_trait_object_param` | 5 | `&[R]` (`R: PriceRule`) | `&[Box<dyn PriceRule>]` |

### 01_fn_pointer_vs_fn_bound — 함수 포인터와 `Fn*` 바운드

[`examples/01_fn_pointer_vs_fn_bound.rs`](examples/01_fn_pointer_vs_fn_bound.rs)

1번 구역은 함수 포인터로 받는 함수에 기준값을 박아 넣은 클로저와 미리 써 둔 `is_warm`을 넘깁니다. 2번 구역은 기준값을 캡처한 클로저를 `FnMut` 바운드 함수에 넘깁니다. 3번 구역은 권장 함수가 함수 이름과 함수 포인터 표(`PRESETS`)의 값도 받는다는 것을, 4번 구역은 함수 포인터가 `Copy`이고 크기가 포인터 하나라는 것을 보여 줍니다.

```sh
cargo run -p er-ch2-express-common-behavior --example 01_fn_pointer_vs_fn_bound
```

```text
평균 11도, 경보 기준 16도 이상
1. 비권장: fn 포인터로 받는 함수
    값을 박아 넣은 클로저: 3번 측정값 17도
    미리 써 둔 is_warm: 경보 없음
2. 권장: FnMut 바운드로 받는 함수
    기준을 캡처한 클로저: 3번 측정값 17도
3. 권장 함수는 함수와 fn 포인터도 그대로 받습니다
    is_warm: 0번 측정값 27도
    폭염: 2번 측정값 34도
    한파: 경보 없음
4. fn 포인터가 어울리는 곳: 이름을 적을 수 있는 구체 타입이 필요할 때
    size_of::<Rule>() = 8, 복사한 규칙으로 36도 판단: true
```

`size_of::<Rule>() = 8`은 64비트 환경에서의 값입니다.

### 02_loosest_fn_bound — 가장 느슨한 바운드

[`examples/02_loosest_fn_bound.rs`](examples/02_loosest_fn_bound.rs)

1번 구역은 `Fn`만 받는 `run_job_strict`에 맞추느라 `RefCell`과 `Cell<Option<Report>>`를 쓰고, 2번 구역은 `FnMut`/`FnOnce`를 받는 `run_job`에 평범한 클로저를 넘깁니다. 두 구역의 출력이 같다는 것이 요점입니다. 달라진 것은 부르는 쪽 코드뿐입니다. 3번 구역은 느슨한 바운드가 읽기만 하는 클로저도 받는다는 것을, 4번 구역은 `&self` 메서드에서 부르는 규칙이라 `Fn`이 맞는 `Gate`를 보여 줍니다.

```sh
cargo run -p er-ch2-express-common-behavior --example 02_loosest_fn_bound
```

```text
1. 비권장: Fn만 받는 함수에 맞추느라 우회로를 씁니다
    log = ["1. 압축", "2. 전송", "3. 검증"]
    outbox = ["야간 백업 (3단계)"]
2. 권장: 가장 느슨한 바운드라 평범한 클로저를 그대로 넘깁니다
    log = ["1. 압축", "2. 전송", "3. 검증"]
    outbox = ["야간 백업 (3단계)"]
3. 느슨한 바운드도 Fn 클로저를 그대로 받습니다
    단계 1: 압축
    단계 2: 전송
    단계 2개 끝
4. Fn이 맞는 경우: &self 메서드에서 부르는 규칙
    17세 거절, 25세 통과
```

### 03_trait_over_concrete — 구체 타입 대신 트레이트

[`examples/03_trait_over_concrete.rs`](examples/03_trait_over_concrete.rs)

`Disk`는 쓸 때마다 `[디스크에 씀]`을 출력해 부수 효과를 눈에 보이게 합니다. 1번 구역은 `Disk`만 받는 함수로 미리 보기를 하려다 실제로 쓰고, 2번 구역은 `Storage`를 구현한 `Preview`를 넘겨 디스크를 건드리지 않습니다. 3번 구역은 같은 `backup`으로 진짜 디스크에 씁니다. 크기 23바이트는 `"우유 사기"`(UTF-8로 13바이트)와 `"rust study"`(10바이트)의 합입니다.

```sh
cargo run -p er-ch2-express-common-behavior --example 03_trait_over_concrete
```

```text
1. 비권장: 미리 보기를 하려 해도 Disk밖에 넘길 것이 없습니다
    [디스크에 씀] todo.txt
    [디스크에 씀] idea.txt
    예상 크기 23바이트, 디스크의 파일 수 2
2. 권장: 미리 보기 저장소를 넘기면 디스크는 건드리지 않습니다
    예상 크기 23바이트, 쓸 파일 ["todo.txt", "idea.txt"], 디스크의 파일 수 0
3. 확인한 뒤에 같은 함수로 진짜 디스크에 씁니다
    [디스크에 씀] todo.txt
    [디스크에 씀] idea.txt
    쓴 크기 23바이트, 디스크의 파일 수 2
```

### 04_marker_trait — 마커 트레이트

[`examples/04_marker_trait.rs`](examples/04_marker_trait.rs)

`FlakyLink`는 명령을 적용한 뒤 처음 `lost_replies`번은 응답이 사라진 것처럼 `false`를 돌려줍니다. 1번 구역은 아무 명령이나 재시도해서 포인트가 두 번 적립되고, 2번 구역은 멱등인 `SetLimit`만 재시도해서 결과가 맞습니다. 3번 구역은 멱등이 아닌 명령을 `send_once`로 한 번만 보냅니다(응답은 사라졌지만 적용은 됐으므로 `points: 500`).

```sh
cargo run -p er-ch2-express-common-behavior --example 04_marker_trait
```

```text
1. 비권장: 아무 명령이나 재시도
    500포인트 적립, 2번 보냄 → points = 1000
2. 권장: 멱등인 명령은 재시도해도 결과가 같습니다
    한도 300으로 설정, 2번 보냄 → limit = 300
3. 권장: 멱등이 아닌 명령은 재시도 함수에 넘길 수 없습니다
    응답 받음: false, 최종 상태 Account { limit: 300, points: 500 }
```

### 05_trait_object_param — 트레이트 객체로 받기

[`examples/05_trait_object_param.rs`](examples/05_trait_object_param.rs)

1번 구역은 제네릭 슬라이스 함수를 타입별로 두 번 불러 적용 순서가 바뀐 결과를, 2번 구역은 설정 문자열을 `parse_rule`로 읽어 만든 `Vec<Box<dyn PriceRule>>`을 정해진 순서대로 적용한 결과와 `&mut dyn FnMut` 콜백이 모은 단계별 가격을 보여 줍니다. 3번 구역은 `?Sized`를 붙인 `preview`가 구체 타입과 트레이트 객체를 모두 받는 것을 보여 줍니다.

```sh
cargo run -p er-ch2-express-common-behavior --example 05_trait_object_param
```

```text
1. 비권장: 타입별로 나눠서 두 번 부릅니다
    최종 가격 14850원
2. 권장: 트레이트 객체로 받아 정해진 순서대로 적용합니다
    3000원 할인: 17000원
    10% 할인: 15300원
    500원 할인: 14800원
    최종 가격 14800원
3. ?Sized를 붙인 제네릭 함수는 구체 타입과 트레이트 객체를 모두 받습니다
    구체 타입: 1000원 할인 → 19000원
    트레이트 객체: 10% 할인 → 18000원
```

## 4. 자주 하는 실수

세 실수 모두 **에러는 부르는 쪽 줄에서 나지만 원인은 받는 쪽 시그니처**에 있습니다. 받는 쪽 코드를 고칠 수 있다면 받는 쪽을 고치는 것이 맞는 해결입니다. 직접 확인하려면 챕터 디렉터리에서 다음을 실행합니다(`--out-dir`은 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
rustc --edition 2024 compile_fail/e0308_capturing_closure_to_fn_pointer.rs --out-dir /tmp/er-ch2
rustc --edition 2024 compile_fail/e0525_fnmut_closure_to_fn_bound.rs --out-dir /tmp/er-ch2
rustc --edition 2024 compile_fail/e0277_dyn_to_implicit_sized.rs --out-dir /tmp/er-ch2
```

### 4.1 캡처하는 클로저를 함수 포인터 매개변수에 넘김 — E0308

[`compile_fail/e0308_capturing_closure_to_fn_pointer.rs`](compile_fail/e0308_capturing_closure_to_fn_pointer.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_capturing_closure_to_fn_pointer.rs:19:44
   |
19 |     let found = first_alert_ptr(&readings, |t| t >= limit); // E0308
   |                 ---------------            ^^^^^^^^^^^^^^ expected fn pointer, found closure
   |                 |
   |                 arguments to this function are incorrect
   |
   = note: expected fn pointer `fn(i32) -> bool`
                 found closure `{closure@compile_fail/e0308_capturing_closure_to_fn_pointer.rs:19:44: 19:47}`
note: closures can only be coerced to `fn` types if they do not capture any variables
  --> compile_fail/e0308_capturing_closure_to_fn_pointer.rs:19:53
   |
19 |     let found = first_alert_ptr(&readings, |t| t >= limit); // E0308
   |                                                     ^^^^^ `limit` captured here
note: function defined here
  --> compile_fail/e0308_capturing_closure_to_fn_pointer.rs:12:4
   |
12 | fn first_alert_ptr(readings: &[i32], is_alert: fn(i32) -> bool) -> Option<usize> {
   |    ^^^^^^^^^^^^^^^                   -------------------------
```

- **읽는 법**: 두 번째 `note`가 캡처한 변수(`limit`)를 가리키고, 마지막 `note`가 받는 쪽 매개변수 `is_alert: fn(i32) -> bool`을 가리킵니다. 캡처하지 않는 클로저는 함수 포인터로 바뀔 수 있지만, 캡처한 값을 실을 자리가 없는 함수 포인터로는 캡처하는 클로저를 바꿀 수 없다는 뜻입니다. Part 0 ch19에서는 같은 에러를 배열에 클로저를 담는 상황에서 봤습니다. 여기서는 받는 함수의 시그니처가 원인입니다.
- **해결**: 받는 쪽을 `is_alert: impl FnMut(i32) -> bool`로 바꿉니다(예제 01의 `first_alert`). 받는 쪽이 다른 크레이트라 고칠 수 없다면, 부르는 쪽은 캡처를 없애야 합니다. 값을 코드에 박아 넣거나 전역 상태를 쓰게 되는데, 이것이 지침 1이 막으려는 비용입니다.

### 4.2 `FnMut` 클로저를 `Fn` 바운드에 넘김 — E0525 (인자 자리에 바로 쓰면 E0596)

[`compile_fail/e0525_fnmut_closure_to_fn_bound.rs`](compile_fail/e0525_fnmut_closure_to_fn_bound.rs)

```text
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnMut`
  --> compile_fail/e0525_fnmut_closure_to_fn_bound.rs:26:18
   |
26 |     let record = |i: usize, name: &str| log.push(format!("{i}. {name}"));
   |                  ^^^^^^^^^^^^^^^^^^^^^^ --- closure is `FnMut` because it mutates the variable `log` here
   |                  |
   |                  this closure implements `FnMut`, not `Fn`
27 |     run_job_strict(&["압축", "전송"], record); // E0525
   |     --------------                    ------ the requirement to implement `Fn` derives from here
   |     |
   |     required by a bound introduced by this call
   |
note: required by a bound in `run_job_strict`
  --> compile_fail/e0525_fnmut_closure_to_fn_bound.rs:17:8
   |
15 | fn run_job_strict<S>(steps: &[&str], on_step: S)
16 | where
17 |     S: Fn(usize, &str),
   |        ^^^^^^^^^^^^^^^ required by this bound in `run_job_strict`
```

- **읽는 법**: 클로저를 변수에 담아 두었다가 넘기면, 클로저의 종류(`FnMut`)는 정의할 때 이미 정해졌고 그 뒤에 `Fn` 요구를 만납니다. 그래서 "이 클로저는 `FnMut`일 뿐"이라는 E0525가 나오고, `note`가 요구의 출처인 받는 쪽 바운드를 가리킵니다.
- **같은 클로저를 인자 자리에 바로 쓰면** 컴파일러가 바운드를 먼저 보고 클로저를 `Fn`으로 만들려다 실패하므로 에러가 바뀝니다. 이때는 받는 쪽 매개변수에 직접 도움말이 붙습니다. 아래는 26~27행을 `run_job_strict(&["압축", "전송"], |i, name| log.push(format!("{i}. {name}")));` 한 줄로 바꿔 컴파일한 결과의 핵심 줄입니다.

  ```text
  error[E0596]: cannot borrow `log` as mutable, as it is a captured variable in a `Fn` closure
  15 | fn run_job_strict<S>(steps: &[&str], on_step: S)
     |                                               - change this to accept `FnMut` instead of `Fn`
  ```

  값을 밖으로 옮기는 클로저를 `Fn` 바운드 자리에 바로 쓰면 E0507(``cannot move out of `draft`, a captured variable in an `Fn` closure``)이 납니다. 이때 붙는 ``consider cloning the value`` 도움말은 부르는 쪽에서 복제하라는 우회로입니다.
- **해결**: 받는 쪽 바운드를 본문이 부르는 방식에 맞게 느슨하게 합니다. `run_job_strict`는 차례로 부르기만 하므로 `S: FnMut(usize, &str)`로 바꾸고 매개변수에 `mut`을 붙입니다(예제 02의 `run_job`). Part 0 ch13 4.4절의 E0507은 표준 라이브러리 함수(`sort_by_key`)가 정한 바운드에 부르는 쪽이 맞추는 상황이었고, 여기서는 받는 쪽 바운드를 직접 고칠 수 있는 상황입니다.

### 4.3 트레이트 객체를 암묵적 `Sized` 제네릭에 넘김 — E0277

[`compile_fail/e0277_dyn_to_implicit_sized.rs`](compile_fail/e0277_dyn_to_implicit_sized.rs)

```text
error[E0277]: the size for values of type `dyn PriceRule` cannot be known at compilation time
  --> compile_fail/e0277_dyn_to_implicit_sized.rs:33:28
   |
33 |     println!("{}", preview(chosen.as_ref(), 10_000)); // E0277
   |                    ------- ^^^^^^^^^^^^^^^ doesn't have a size known at compile-time
   |                    |
   |                    required by a bound introduced by this call
   |
   = help: the trait `Sized` is not implemented for `dyn PriceRule`
note: required by an implicit `Sized` bound in `preview`
  --> compile_fail/e0277_dyn_to_implicit_sized.rs:27:12
   |
27 | fn preview<R: PriceRule>(rule: &R, price: u32) -> u32 {
   |            ^ required by the implicit `Sized` requirement on this type parameter in `preview`
help: consider removing this method call, as the receiver has type `Box<dyn PriceRule>` and `Box<dyn PriceRule>: Sized` trivially holds
   |
33 -     println!("{}", preview(chosen.as_ref(), 10_000)); // E0277
33 +     println!("{}", preview(chosen, 10_000)); // E0277
   |
help: consider relaxing the implicit `Sized` restriction
   |
27 | fn preview<R: PriceRule + ?Sized>(rule: &R, price: u32) -> u32 {
   |                         ++++++++
```

- **읽는 법**: `note`의 "implicit `Sized` bound"가 핵심입니다. `R: PriceRule`이라고만 적었어도 `R: Sized`가 함께 붙어 있어서, 매개변수가 참조(`&R`)인데도 `R = dyn PriceRule`이 될 수 없습니다.
- **도움말이 둘입니다.** 첫 번째(`.as_ref()`를 지우라는 것)를 따르면 `Box<dyn PriceRule>`을 `&R` 자리에 넘기게 되어 E0308(``expected `&_`, found `Box<dyn PriceRule>` ``)이 납니다. 받는 쪽을 고칠 수 있다면 두 번째 도움말이 맞습니다.
- **해결**: `fn preview<R: PriceRule + ?Sized>(rule: &R, ..)`로 바꿉니다(예제 05의 `preview`). 본문이 `R`을 참조로만 다룬다면 `?Sized`를 붙여도 잃는 것이 없습니다.

## 5. 연습 문제

### 연습 1. 바운드 고르기

다음 함수와 메서드에 붙일 가장 느슨한 `Fn*` 바운드를 고르고, 예제 파일 하나에 직접 써서 컴파일해 확인하세요.

1. `fn pick<T, F>(cached: Option<T>, urgent: bool, make: F) -> T` — `urgent`이면 `make()`를 부르고, 아니면 `cached`가 `Some`일 때 그 값을, `None`일 때 `make()`를 부릅니다.
2. `fn poll_until<F>(limit: u32, check: F) -> Option<u32>` — `1..=limit`의 각 수로 `check(n)`을 불러 처음 `true`가 나온 수를 돌려줍니다.
3. `struct Formatter<F> { render: F }`의 `fn show(&self, value: i32) -> String` — 필드의 클로저로 값을 문자열로 바꿉니다.

<details>
<summary>힌트</summary>

1번은 `make()`가 두 곳에 있지만 어느 경로로 가든 많아야 한 번 불리므로 `FnOnce() -> T`로 충분합니다. 컴파일러는 갈래별로 따로 검사하므로 이 코드가 컴파일됩니다. 2번은 차례로 여러 번 부르므로 `FnMut(u32) -> bool`이고 매개변수에 `mut`이 필요합니다. 3번은 `&self`로 부르므로 `Fn(i32) -> String`입니다. 3번을 `FnMut`으로 바꿔 E0596이 나는 것도 확인해 보세요.

</details>

### 연습 2. 두 곳에 함께 쓰는 저장소

예제 03에 저장소 두 개를 받아 같은 내용을 양쪽에 쓰는 `Mirror<A, B>`를 추가하고, `backup(&mut mirror, &notes)`로 `Disk`와 `Preview`에 동시에 백업해 보세요. `backup`은 고치지 않습니다.

<details>
<summary>힌트</summary>

`struct Mirror<A, B> { first: A, second: B }`를 만들고 `impl<A: Storage, B: Storage> Storage for Mirror<A, B>`에서 `save`가 두 필드의 `save`를 차례로 부르게 합니다. 돌려줄 바이트 수는 첫 번째 저장소의 값으로 정하면 됩니다. 트레이트로 받는 함수였기 때문에 "여러 저장소를 하나처럼" 묶는 타입도 끼워 넣을 수 있다는 점을 확인하세요.

</details>

### 연습 3. 어떤 명령이 멱등인가

예제 04에 다음 명령을 추가하고, 각각 `Idempotent`를 구현해야 하는지 판단해 보세요. 구현한 명령만 `send_with_retry`에 넘겨지는지 확인합니다.

- `ResetPoints`: 포인트를 0으로 만듭니다.
- `DoubleLimit`: 한도를 두 배로 만듭니다.
- `SetLimitIfLower(u32)`: 현재 한도가 주어진 값보다 낮을 때만 그 값으로 올립니다.

<details>
<summary>힌트</summary>

두 번 적용한 결과가 한 번 적용한 결과와 같은지 직접 계산해 봅니다. `ResetPoints`는 0 → 0이라 멱등이고, `DoubleLimit`은 100 → 200 → 400이라 멱등이 아닙니다. `SetLimitIfLower(300)`은 한도가 100이든 500이든 두 번째 적용에서 아무것도 바뀌지 않으므로 멱등입니다. `DoubleLimit`에 `impl Idempotent`를 붙여도 컴파일러는 막지 않는다는 것, 즉 마커 트레이트의 약속은 사람이 확인해야 한다는 것도 직접 확인해 보세요.

</details>

### 연습 4. 새 할인 규칙 추가

예제 05에 가격 상한을 두는 규칙 `CapAt(u32)`(가격이 상한보다 크면 상한으로 내림, `"cap:15000"`)를 추가하고, 설정 문자열 `"cap:15000,amount:500"`을 적용해 보세요. 고쳐야 하는 함수와 고치지 않아도 되는 함수를 나눠 보세요.

<details>
<summary>힌트</summary>

새 타입에 `impl PriceRule`을 쓰고, 문자열을 타입으로 바꾸는 `parse_rule`에 `"cap"` 갈래를 하나 더합니다. `apply_rules`와 `preview`는 고치지 않습니다. 20000원에 적용하면 15000원으로 내려간 뒤 500원이 빠져 14500원이 됩니다. 반대로 `apply_rules_generic`을 쓰는 부르는 쪽은 규칙 타입이 하나 늘 때마다 그 타입만 담은 배열과 호출을 하나씩 더 만들어야 하고, 호출 순서를 정하는 일도 떠안는다는 점과 비교해 보세요.

</details>

## 6. 참고 자료

- [Effective Rust, Item 2: Use the type system to express common behavior](https://effective-rust.com/use-types-2.html): 이 챕터가 다루는 지침의 출처입니다. 함수와 메서드, 함수 포인터, 클로저, 트레이트, 트레이트 바운드, 트레이트 객체를 차례로 다룹니다.
- [The Rust Reference: Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility): 트레이트 객체로 쓸 수 있는 트레이트의 전체 규칙입니다. E0038 메시지가 가리키는 문서입니다.
- [`std::ptr::fn_addr_eq`](https://doc.rust-lang.org/std/ptr/fn.fn_addr_eq.html): 함수 포인터의 주소를 비교하는 함수입니다. 같은 함수가 다른 주소를 갖거나 다른 함수가 같은 주소를 갖는 경우를 설명합니다.
- [rustc 린트 목록: `unpredictable_function_pointer_comparisons`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unpredictable-function-pointer-comparisons): 함수 포인터를 `==`로 비교할 때 나는 경고입니다.
- 본편: [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md)(트레이트 바운드), [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md)(클로저 캡처와 `Fn` 트레이트), [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)(트레이트 객체와 dyn 호환), [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)(함수 포인터, `?Sized`)
