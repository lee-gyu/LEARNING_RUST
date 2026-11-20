# ch22. 가시성은 필요한 만큼만 열기

[Part 0 ch7](../../part0-getting-started/ch7-modules/README.md)에서 모든 항목이 기본으로 비공개이고 `pub`을 붙여야 바깥에서 쓸 수 있다는 규칙을 배웠습니다. 이 챕터는 규칙이 아니라 판단을 다룹니다. 항목 하나를 **어디까지 열 것인가**입니다. 책의 답은 제목 그대로 "가능한 한 적게"입니다. 공개한 항목은 그것을 쓰는 코드가 기대는 약속이 되고, 한 번 내준 약속은 거둬들이기 어렵기 때문입니다. 반대로 닫아 둔 것은 언제든 열 수 있습니다. 이 비대칭이 챕터 전체의 근거입니다.

- 원제: [Item 22: Minimize visibility](https://effective-rust.com/visibility.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch22-minimize-visibility`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

문법은 다시 설명하지 않습니다. 다음 내용을 안다고 보고 진행합니다.

- [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.5·2.6·2.10절: 기본 비공개와 `pub`, 구조체 필드와 열거형 배리언트의 공개 규칙, `pub use` 재내보내기
- [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md) 2.8절: 단위 테스트 모듈은 자식 모듈이라 비공개 항목을 쓸 수 있음
- [Part 2 ch2](../../part2-editions/ch2-rust-2015-2018/README.md) 2.4절: `pub(in 경로)`의 경로는 2018 에디션부터 `crate`, `self`, `super`로 시작해야 함
- [ch6](../ch6-newtype-pattern/README.md) 2.3절: 필드를 감추고 생성자에서 검증하는 뉴타입
- [ch16](../ch16-avoid-unsafe/README.md) 2.5절: 비공개 필드가 불변식(invariant)의 경계가 된다는 것(E0451)

앞 챕터가 이 챕터로 미룬 것도 받습니다. [ch7](../ch7-builders/README.md) 2.3절의 "옵션 구조체의 필드가 공개라서 내부 표현을 바꿀 수 없다"는 문제는 2.3절에서, [ch15](../ch15-borrow-checker/README.md) 2.7절의 세대 ID를 비공개 필드로 지키는 일은 2.4절에서 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 비공개, `pub(super)`, `pub(in 경로)`, `pub(crate)`, `pub`이 각각 어디까지 보이게 하는지 설명하고, 항목을 쓰는 코드가 있는 위치를 보고 가장 좁은 단계를 고를 수 있다.
2. 공개한 항목을 비공개로 되돌리거나 공개 필드의 타입을 바꾸는 일이 왜 사용자 코드를 깨는지 설명하고, 필드를 비공개로 두고 메서드로 열어 내부 표현을 바꿀 수 있게 설계할 수 있다.
3. 핸들과 상태 타입의 필드·생성자를 감춰 라이브러리만 그 값을 만들게 하고, 하위 모듈을 비공개로 둔 채 `pub use`로 공개 경로를 정할 수 있다.
4. `dead_code`의 "never used" 경고와 `unreachable_pub`, `private_interfaces` 린트를 읽고, 선언한 가시성과 실제로 닿는 범위가 어긋난 곳을 고칠 수 있다.

## 2. 핵심 개념

2.1절은 가시성 단계와 "공개는 되돌리기 어렵다"는 비대칭을 정리하는 **이해** 부분입니다. 2.2~2.6절이 지침이며, 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 씁니다. 두 판을 함께 실행해 비교하는 전체 코드는 `examples/`에 있습니다. 책이 권하지 않은 판단은 2.7절에 "책에 없는 지침"으로 밝혔습니다.

### 2.1 공개 범위는 약속의 범위다

**가시성 단계.** 가시성(visibility)은 모듈을 단위로 정해집니다. 아무것도 붙이지 않은 항목은 그 항목이 있는 모듈과 그 아래(자손 모듈)에서만 보입니다. `pub` 뒤에 괄호로 범위를 적으면 그 사이의 단계를 고를 수 있습니다.

| 선언 | 보이는 곳 | 주로 쓰는 곳 |
| --- | --- | --- |
| (없음) = `pub(self)` | 그 항목이 있는 모듈과 그 아래 | 기본값. `pub(self)`라고 적을 일은 거의 없음 |
| `pub(super)` = `pub(in super)` | 한 단계 위 모듈과 그 아래 | 형제 모듈이 함께 쓰는 도우미 |
| `pub(in crate::a::b)` | 적은 조상 모듈과 그 아래 | 두 단계 이상 위의 모듈 아래에서 함께 쓰는 도우미 |
| `pub(crate)` = `pub(in crate)` | 크레이트 전체 | 크레이트 여러 곳이 쓰지만 사용자에게 내놓지 않을 도우미 |
| `pub` | 그 항목을 품은 모듈이 보이는 모든 곳(다른 크레이트 포함) | 공개 API |

책은 괄호가 붙은 네 가지를 쓸모가 큰 순서로 `pub(crate)`, `pub(super)`, `pub(in 경로)`, `pub(self)`라고 소개합니다. `pub(crate)`가 첫째인 이유는 "크레이트 안 여러 곳에서 쓰지만 크레이트 사용자에게는 보이면 안 되는 도우미"가 그만큼 흔하기 때문입니다. 크레이트 경계에서 실제로 막힌다는 것은 Part 0 ch7 참고 자료에 적힌 대로이며, 이 챕터에서도 라이브러리와 바이너리 두 크레이트로 다시 확인했습니다. 다른 크레이트에서 `pub(crate)` 메서드를 부르면 ``error[E0624]: method `helper` is private``, `pub(crate)` 함수를 부르면 ``error[E0603]: function `crate_fn` is private``입니다. 컴파일러는 범위를 좁힌 항목을 범위 밖에서 쓰면 그냥 "비공개(private)"라고 부릅니다.

**함께 공개되는 것과 따로 열어야 하는 것.**

- `pub enum`은 배리언트와 배리언트의 필드까지 모두 공개합니다. `pub trait`은 메서드까지 모두 공개합니다. 둘 다 안쪽에 가시성을 따로 적을 수 없어서, 적으면 ``error[E0449]: visibility qualifiers are not permitted here``와 함께 ``enum variants and their fields always share the visibility of the enum they are in``(트레이트는 ``trait items always share the visibility of their trait``)라는 노트가 붙습니다.
- `pub struct`의 필드와 메서드, `pub mod` 안의 항목은 하나씩 `pub`을 붙여야 공개됩니다(Part 0 ch7 2.5·2.6절).
- 반대로 `pub` 항목도 그것을 품은 모듈이 보이지 않으면 바깥에서 보이지 않습니다. 경로의 모든 단계를 지나갈 수 있어야 합니다.

**되돌리기 어려운 쪽은 공개다.** 가시성을 바꾸는 일은 방향에 따라 사용자 코드에 미치는 영향이 다릅니다.

| 바꾸는 일 | 사용자 코드 | 버전 |
| --- | --- | --- |
| 비공개 항목을 공개 | 그대로 컴파일됨(새로 쓸 수 있는 것이 늘 뿐) | 대체로 마이너 |
| 공개 항목을 비공개로 | 그 항목을 쓰던 코드가 E0603·E0624·E0616 등으로 깨짐 | 메이저 |
| 공개 필드의 타입이나 구성을 바꿈 | 그 필드를 쓰던 코드가 깨짐(2.3절) | 메이저 |
| 비공개 필드·비공개 함수를 바꿈 | 그대로 컴파일됨 | 패치 또는 마이너 |

책이 드는 이유는 둘입니다. 첫째, 공개는 되돌리기 어렵습니다. 공개 항목을 다시 닫으려면 메이저 버전을 올려야 합니다. 둘째(책은 이쪽이 더 중요하다고 봅니다), 닫아 두면 선택지가 남습니다. 내부 자료구조를 드러내면 나중에 더 효율적인 알고리즘으로 바꾸는 일이 호환되지 않는 변경이 되고, 드러난 도우미 함수에는 결국 바깥 코드가 기대기 시작합니다. 어떤 변경이 호환되지 않는 변경인지의 전체 규칙과 버전 번호의 약속은 [ch21](../ch21-semver/README.md)에서 다룹니다. 이 챕터에서는 "공개 범위가 곧 약속의 범위"라는 것만 가져갑니다.

책은 이 걱정이 주로 **사용자가 여럿이고 오래 쓰이는 라이브러리**의 것이라고 범위를 정합니다. 다만 임시로 만든 코드가 오래 남는 일이 흔하므로 습관으로 들여 둘 만하다고 덧붙입니다.

### 2.2 지침 1: 기본은 비공개로 두고, 열 때는 쓰는 곳까지만 연다

**문제 상황.** 영화관 예매 모듈을 만듭니다. 좌석을 잡는 `take_seat`은 예매 절차(`booking::book`)가 결제를 확인한 뒤에만 불러야 합니다. 요금표 `base_price`는 예매 절차가 쓰는 내부 계산입니다. 둘 다 `cinema` 안의 다른 모듈에서 쓰므로 무언가는 열어야 합니다.

**비권장 코드.** 컴파일러가 막을 때마다 `pub`을 붙여 나갑니다. [`01_visibility_levels.rs`](examples/01_visibility_levels.rs)의 `cinema_loose`는 모든 모듈과 항목이 `pub`입니다.

```rust
pub mod cinema_loose {
    pub mod hall {
        pub struct Cinema { pub taken: Vec<bool>, pub tickets: u32, pub revenue: u32 }
        impl Cinema {
            pub fn take_seat(&mut self, seat: usize) -> Result<(), String> { .. }
            // new, record_sale, report도 pub
        }
    }
    pub mod booking { pub fn book(..) -> Result<Ticket, String> { .. } }
    pub mod pricing { pub mod rates { pub fn base_price(seat: usize) -> u32 { .. } } }
}

let bypass = hall.take_seat(4); // main이 예매 절차를 건너뜀
```

`booking`에서 쓰려고 연 `take_seat`이 `main`에도 열렸습니다. 결제도 표 발급도 없이 좌석이 잡혀 `팔린 좌석 2석, 발급한 표 1장, 매출 9000원`이라는 어긋난 상태가 됩니다. `main`은 내부 요금표 `base_price`도 직접 읽습니다. 이제 요금 계산의 시그니처를 바꾸면 `main`이 깨집니다.

**권장 코드.** 항목마다 "누가 쓰는가"를 묻고 그 범위까지만 엽니다.

```rust
pub mod cinema {
    pub use booking::{Ticket, book}; // 바깥에 보일 이름만 올림
    pub use hall::Cinema;

    mod hall {
        impl Cinema {
            pub fn new(seats: usize) -> Cinema { .. }
            pub(super) fn take_seat(&mut self, seat: usize) -> Result<(), String> { .. }
        }
    }
    mod pricing {
        pub(super) mod rates {
            pub(in crate::cinema) fn base_price(seat: usize) -> u32 { .. }
        }
    }
}
```

| 항목 | 쓰는 곳 | 고른 단계 |
| --- | --- | --- |
| `Cinema`, `Cinema::new`, `report`, `book`, `Ticket` | 크레이트 밖(사용자) | `pub` |
| `audit::note` | `main`과 `cinema::booking` | `pub(crate)` |
| `hall::Cinema::take_seat` | 형제 모듈 `booking` | `pub(super)`: `hall`의 부모 `cinema`와 그 아래 |
| `pricing::rates::base_price` | 두 단계 위 `cinema` 아래의 `booking` | `pub(in crate::cinema)` |
| 필드, 나머지 | 같은 모듈 | (없음) |

`base_price`에 `pub(super)`를 붙이면 `pricing`까지만 보여 `booking`이 쓸 수 없고, `pub(crate)`는 필요보다 넓습니다. 그래서 조상을 직접 적는 `pub(in crate::cinema)`입니다. `rates` 모듈 자체에도 `pub(super)`가 필요한 것은 경로의 모든 단계를 지나갈 수 있어야 하기 때문입니다. 이제 `main`에서 `hall.take_seat(4)`를 쓰면 E0624(4.1절), `cinema::pricing::rates::base_price(4)`를 쓰면 ``error[E0603]: module `pricing` is private``입니다. 예제의 권장 판에서는 같은 입력이 `팔린 좌석 2석, 발급한 표 2장, 매출 21000원`으로 맞아떨어집니다.

`pub(in 경로)`의 경로는 그 항목의 **조상** 모듈이어야 합니다. "이 형제 모듈에게만"이라는 뜻으로 `pub(in crate::cinema::seats)`처럼 적으면 E0742입니다(4.2절). 둘 다에게 보여야 한다면 가장 가까운 공통 조상을 적습니다.

고르는 순서를 정리하면 이렇습니다. 같은 모듈에서만 쓰면 아무것도 붙이지 않습니다. 부모 모듈이나 형제 모듈이 쓰면 `pub(super)`, 더 위의 조상 아래에서 쓰면 `pub(in crate::…)`, 크레이트 곳곳에서 쓰면 `pub(crate)`, 사용자에게 내놓을 때만 `pub`입니다. 단위 테스트 때문에 넓힐 필요는 없습니다. `#[cfg(test)] mod tests`는 자식 모듈이라 비공개 항목을 그대로 씁니다(Part 0 ch11 2.8절).

**지침을 따르지 않아도 되는 경우.** 책이 범위를 정한 대로, 이 지침의 값어치는 사용자가 여럿인 라이브러리에서 가장 큽니다. 혼자 쓰고 버릴 작은 프로그램이라면 단계를 하나하나 고르는 수고가 얻는 것보다 클 수 있습니다. 그때도 바깥에 내놓을 것과 아닌 것을 `pub`과 `pub(crate)`로 나누는 정도는 비용이 거의 없습니다.

### 2.3 지침 2: 구조체의 필드는 비공개로 두고 메서드로 연다

**문제 상황.** 음악 재생 대기열 `PlayQueue`의 첫 판은 곡을 `Vec<String>`에 담습니다. 맨 앞 곡을 꺼내는 `Vec::remove(0)`은 남은 원소를 모두 한 칸씩 당기므로 대기열이 길수록 느려집니다. 둘째 판에서는 앞에서 꺼내기가 싼 `VecDeque`로 바꾸려 합니다. 책이 말한 "더 효율적인 자료구조로 바꾸기"입니다.

**비권장 코드.** 첫 판이 필드를 공개했습니다([`02_private_fields.rs`](examples/02_private_fields.rs)의 `queue_pub_v1`).

```rust
pub struct PlayQueue {
    pub songs: Vec<String>, // 표현(Vec)이 곧 공개 API
}

// 사용자 코드
let mut q = PlayQueue { songs: Vec::new() };
q.songs.push(title.to_string());
let first = q.songs.remove(0);
```

실행 결과는 멀쩡합니다. 문제는 둘째 판을 낼 때 드러납니다. 필드 타입을 `VecDeque<String>`으로 바꾸고 위 사용자 코드를 그대로 컴파일하면 세 곳이 깨집니다(임시 디렉터리에서 확인, 핵심 줄만 옮김).

```text
error[E0308]: mismatched types
  |     let mut q = playlist::PlayQueue { songs: Vec::new() };
  |                                              ^^^^^^^^^^ expected `VecDeque<String>`, found `Vec<_>`
error[E0599]: no method named `push` found for struct `VecDeque<T, A>` in the current scope
error[E0308]: mismatched types
  |     let now: String = q.songs.remove(0);
  |                       ^^^^^^^^^^^^^^^^^ expected `String`, found `Option<String>`
```

`VecDeque::remove`는 `Option`을 돌려주고 `push`라는 메서드가 없습니다. 사용자 코드는 "대기열"이 아니라 "`Vec`"에 기대고 있었습니다. 필드를 하나 **더하는** 것도 마찬가지입니다. 같은 곡이 두 번 들어가지 않게 `seen: HashSet<String>` 필드를 추가하면 사용자의 구조체 리터럴이 ``error[E0063]: missing field `seen` in initializer of `queue_pub_v1::PlayQueue` ``로 깨집니다(연습 2).

**권장 코드.** 필드는 비공개로 두고, 사용자가 할 일(넣기, 꺼내기, 남은 곡 보기)을 메서드로 엽니다.

```rust
pub struct PlayQueue {
    songs: Vec<String>, // 둘째 판에서는 VecDeque<String>
}

impl PlayQueue {
    pub fn new() -> PlayQueue
    pub fn add(&mut self, title: &str)
    pub fn play_next(&mut self) -> Option<String>         // 둘째 판: self.songs.pop_front()
    pub fn upcoming(&self) -> impl Iterator<Item = &str>  // 반복자의 구체 타입도 약속하지 않음
}
```

예제의 `queue_v1`(`Vec`)과 `queue_v2`(`VecDeque`)는 공개 메서드의 시그니처가 같고 내부만 다릅니다. 두 판을 쓰는 사용자 코드 `listen_v1`과 `listen_v2`는 함수 이름과 `use` 줄 말고는 한 글자도 다르지 않으며, 결과도 같습니다. `upcoming`의 반환 타입을 `std::iter::Map<std::slice::Iter<'_, String>, …>` 같은 구체 타입으로 적지 않고 `impl Iterator`로 둔 것도 같은 생각입니다. 구체 타입을 적었다면 `Vec`이라는 표현이 반환 타입으로 새어 나가, 둘째 판(`VecDeque`의 반복자는 타입이 다름)에서 시그니처가 바뀌었을 것입니다.

[ch7](../ch7-builders/README.md) 2.3절의 옵션 구조체가 이 문제를 그대로 안고 있습니다. `..Default::default()`로 쓰려면 모든 필드가 공개여야 하고, 그래서 `timeout: Duration`을 "제한 없음"을 담을 수 있는 `Option<Duration>`으로 바꾸는 순간 `timeout: Duration::from_secs(10)`을 적은 사용자 코드가 ``expected `Option<Duration>`, found `Duration` ``(E0308)로 깨집니다(임시 디렉터리에서 확인). ch7이 규칙이나 필수 값이 있는 타입에 빌더를 권한 것은 필드를 감출 수 있기 때문이기도 합니다.

Rust API 가이드라인도 같은 권고를 합니다(C-STRUCT-PRIVATE). 공개 필드는 표현을 고정하고, 사용자가 필드를 마음대로 고칠 수 있어서 타입이 값을 검증하거나 불변식을 지킬 수 없다는 것이 이유입니다. 둘째 이유는 2.4절에서 봅니다.

**지침을 따르지 않아도 되는 경우.**

- 필드가 곧 타입의 뜻인 수동적인 데이터라면 공개 필드도 괜찮습니다. 가이드라인이 드는 예외도 C 구조체 같은 수동적인 복합 데이터입니다. `Point { x, y }`의 `x`와 `y`는 다른 표현으로 바꿀 일이 없고, 필드 사이에 지킬 규칙도 없습니다.
- 사용자가 구조체 갱신 문법으로 일부만 고쳐 쓰는 편의가 중요한 옵션 구조체도 같은 경우입니다(ch7 2.3절의 세 조건). 대신 필드의 타입을 바꾸는 것은 호환되지 않는 변경이라는 것을 받아들입니다. 필드를 나중에 더할 수 있게 구조체에 `#[non_exhaustive]`를 붙이는 방법은 [ch21](../ch21-semver/README.md)와 Part 9 `ch4-attributes`에서 다룹니다.
- 크레이트 안에서만 쓰는 타입이라면 표현을 바꿀 때 컴파일러가 고칠 곳을 모두 알려 주므로, `pub(crate)` 필드로 충분할 때가 많습니다.

### 2.4 지침 3: 핸들의 필드를 감춰, 라이브러리만 만들고 고치게 한다 (책에 없는 지침)

이 절은 책의 Item 22가 직접 권하지 않습니다. 근거는 2.3절의 API 가이드라인이 드는 둘째 이유(불변식)와, [ch15](../ch15-borrow-checker/README.md) 2.7절이 남긴 세대 ID입니다.

**문제 상황.** ch15의 채팅 명부는 사용자를 칸에 담고, `UserId { slot, generation }`으로 가리킵니다. 탈퇴하면 칸의 세대가 1 올라가므로, 그 칸을 새로 쓰는 사람을 옛 ID로는 찾을 수 없습니다. ch15에서는 모든 코드가 한 모듈에 있었습니다. 이 명부를 라이브러리로 내놓으면 사용자 코드도 `UserId`와 명부의 칸을 만지게 됩니다.

**비권장 코드.** [`03_opaque_handle.rs`](examples/03_opaque_handle.rs)의 `roster_open`은 `UserId`의 필드와 명부의 칸(`slots`)이 `pub`입니다. 바다가 탈퇴하고 별이 그 칸을 다시 쓴 뒤, 바다의 옛 ID를 들고 있던 사용자 코드가 두 가지를 시도합니다.

```rust
// (1) 위조: "세대가 안 맞으니 하나 올려서 다시 찾자"
let forged = roster_open::UserId { slot: sea.slot, generation: sea.generation + 1 };
r.get(forged)                               // Some("별")

// (2) 내부 고치기: 칸의 세대를 바다가 받았던 값으로 되돌림
r.slots[sea.slot].generation = sea.generation;
r.get(sea)                                  // Some("별")
r.get(star)                                 // None
```

세대 ID가 막으려던 일이 그대로 일어납니다. 바다의 대화창에 별의 정보가 보이고, 별이 정상적으로 받은 ID는 오히려 쓸 수 없게 됩니다. 명부 쪽 코드에는 잘못이 없습니다. 명부가 지키는 불변식("ID의 세대가 칸의 세대와 같을 때만 그 칸을 가리킨다", "칸의 세대는 탈퇴할 때만 오른다")을 바깥 코드가 직접 깰 수 있었을 뿐입니다.

**권장 코드.** `UserId`의 필드와 칸을 모두 감춥니다.

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UserId { slot: usize, generation: u32 } // 만드는 곳은 Roster::join뿐

struct Slot { generation: u32, nickname: Option<String> } // 타입 자체가 비공개

pub struct Roster { slots: Vec<Slot>, free: Vec<usize> } // 세대를 바꾸는 곳은 leave뿐
```

사용자 코드는 `join`이 준 ID를 복사하고, 비교하고, `get`·`leave`에 넘길 수만 있습니다. 위의 (1)은 ``error[E0451]: fields `slot` and `generation` of struct `roster::UserId` are private``, (2)와 같은 필드 읽기·쓰기는 E0616으로 컴파일되지 않습니다. 예제의 권장 판에서 바다의 옛 ID는 `None`을 돌려주고, 옛 ID로 탈퇴를 시도해도 `false`로 거절됩니다.

필드를 감춘 핸들은 2.3절의 이점도 함께 얻습니다. 나중에 `UserId`를 `u64` 하나에 칸 번호와 세대를 묶어 담는 표현으로 바꿔도 사용자 코드는 그대로입니다. 같은 원리를 검증 뉴타입에 적용한 것이 [ch6](../ch6-newtype-pattern/README.md) 2.3절, `unsafe`의 건전성에 적용한 것이 [ch16](../ch16-avoid-unsafe/README.md) 2.5절입니다.

**지침을 따르지 않아도 되는 경우.** 데이터베이스의 기본 키처럼 바깥에서 정해져 들어오는 번호라면 라이브러리가 만들 수 있는 값이 아니므로 숨길 이유가 없습니다. 그때도 `UserId(u64)` 같은 뉴타입으로 다른 정수와 섞이지 않게 하는 것은 ch6의 지침입니다.

### 2.5 지침 4: 모듈 구조는 감추고, 공개 경로는 `pub use`로 정한다

**문제 상황.** 주문 모듈 `order`가 장바구니 `Cart`와 상태 타입 `Paid`(결제 완료), `Shipped`(배송됨)를 가집니다. 파일이 커져서 상태 타입을 하위 모듈 `states::paid`, `states::shipped`로 나눴습니다. `Paid`는 `Cart::checkout`이 결제를 확인했을 때만 만들어져야 합니다.

**비권장 코드.** 나눈 하위 모듈을 모두 `pub mod`로, 생성자 `Paid::new`도 `pub`으로 둡니다([`04_public_paths.rs`](examples/04_public_paths.rs)의 `order_loose`).

```rust
use order_loose::states::paid::Paid as LoosePaid; // 내부 모듈 구조가 사용자 코드에 적힘
let fake = LoosePaid::new(1, 0);                  // 장바구니도 결제도 없이 "결제 완료"
let shipped = fake.ship("FAKE-0001");             // 그대로 배송됨: 0원
```

문제가 둘입니다. 사용자 코드에 내부 파일 구조(`states::paid`)가 경로로 적혔으므로, 나중에 `paid`와 `shipped`를 한 파일로 합치면 사용자 코드가 깨집니다. 그리고 상태 전이(`Cart` → `Paid` → `Shipped`)를 건너뛰는 생성자가 열려 있습니다.

**권장 코드.** 하위 모듈은 비공개로 두고, 사용자가 이름을 적어야 하는 타입만 `pub use`로 위로 올립니다. 생성자는 `order` 안에서만 부르도록 `pub(in crate::order)`로 묶습니다.

```rust
pub mod order {
    pub use cart::Cart;               // 공개 경로는 이 두 줄이 정함
    pub use states::{Paid, Shipped};

    mod cart { /* Cart::checkout이 Paid::new를 부름 */ }
    mod states {
        pub use paid::Paid;           // 비공개 모듈 안의 pub use: order에서 states::Paid로 쓰게 함
        pub use shipped::Shipped;
        mod paid {
            pub struct Paid { items: usize, total: u32 }
            impl Paid {
                pub(in crate::order) fn new(items: usize, total: u32) -> Paid { .. }
                pub fn ship(self, tracking: &str) -> Shipped { Shipped::new(self.total, tracking) }
            }
        }
        mod shipped { /* Shipped::new도 pub(in crate::order) */ }
    }
}

fn receipt(p: &order::Paid) -> String // 사용자는 공개 경로로 타입을 적음
```

사용자는 `order::Paid`라는 이름으로 함수 시그니처에 타입을 적을 수 있지만, 만들 수는 없습니다(`order::Paid::new(1, 0)`은 ``error[E0624]: associated function `new` is private``). 내부 경로 `order::states::paid::Paid`는 ``error[E0603]: module `states` is private``입니다. 예제의 3번 구역은 `type_name`으로 공개 경로와 실제로 정의된 곳을 나란히 보여 줍니다.

```text
    order::Paid     → 04_public_paths::order::states::paid::Paid
    std::iter::Map  → core::iter::adapters::map::Map<alloc::vec::into_iter::IntoIter<u32>, fn(u32) -> u32>
```

**표준 라이브러리의 근거.** 책은 `pub(in 경로)`의 예로 표준 라이브러리의 반복자 어댑터를 듭니다. 1.100.0의 소스(로컬 문서의 `src/core/iter/`)에서 확인한 배치는 위 예제와 같습니다.

- `core/src/iter/mod.rs`에 `mod adapters;`(비공개)와 `pub use self::adapters::{Chain, …, Map, …}`이 있습니다. 그래서 사용자는 `std::iter::Map`(std가 core를 재내보낸 경로)으로 적지만, `type_name`은 정의된 곳인 `core::iter::adapters::map::Map`을 출력합니다.
- `adapters/mod.rs`도 `mod map;`(비공개)과 `pub use self::{…, map::Map, …}`로 한 단계 위에 올립니다.
- `adapters/map.rs`의 `Map::new`는 `pub(in crate::iter) const fn new(iter: I, f: F) -> Map<I, F>`이고, `traits/iterator.rs`의 `Iterator::map`이 `Map::new(self, f)`를 부릅니다. `adapters/` 디렉터리의 파일 23개에 `pub(in crate::iter)`가 모두 24번 나옵니다.
- `Map`의 필드는 `pub(crate) iter: I`와 비공개 `f: F`입니다. `iter`가 `pub(crate)`인 이유는 소스 주석에 따르면 `SplitWhitespace`의 `as_str` 때문입니다. 크레이트 안의 다른 모듈이 필요로 하는 만큼만 연 예입니다.

**지침을 따르지 않아도 되는 경우.** 모듈 자체가 사용자에게 뜻 있는 이름 공간이라면 `pub mod`로 엽니다. `std::collections::hash_map`이 그런 예로, 사용자는 `hash_map::Entry`처럼 모듈 이름을 함께 적습니다. 하위 모듈이 없는 작은 크레이트라면 감출 구조도 없습니다. 공개 경로를 올릴 때도 [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.10절의 기준대로 "쓰는 사람이 생각하는 구조"를 따릅니다. 이 챕터의 `pub use`는 자기 크레이트 안의 항목만 다루며, 다른 크레이트의 타입을 재내보내는 일은 [ch24](../ch24-re-export-dependencies/README.md)에서 다룹니다.

### 2.6 지침 5: "never used" 경고와 린트로 공개 범위를 점검한다

**문제 상황.** 기온 통계 모듈 `stats_loose`에 중앙값 함수 `median`을 `pub`으로 만들어 두었습니다. 사용자가 쓰라고 만든 함수입니다. 그런데 `median`은 비공개 하위 모듈 `math` 안에 있고, 아무도 재내보내지 않았습니다. 선언은 `pub`이지만 크레이트 밖에서는 닿을 길이 없습니다.

**비권장 코드.** [`05_visibility_lints.rs`](examples/05_visibility_lints.rs)의 `stats_loose`에는 이런 어긋남이 세 가지 있고, 각각 `#[expect(…)]`로 표시했습니다. 속성을 지우고 컴파일하면 다음 경고가 납니다(위치 줄은 생략).

```text
warning: function `median` is never used
  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: unreachable `pub` item
   |         pub fn mean(xs: &[i32]) -> f64 {
   |         ---^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |         |
   |         help: consider restricting its visibility: `pub(super)`
   |
   = help: or consider exporting it for use by other crates

warning: type `stats_loose::Summary` is more private than the item `stats_loose::summarize`
   |     pub fn summarize(xs: &[i32]) -> Summary {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ function `stats_loose::summarize` is reachable at visibility `pub`
note: but type `stats_loose::Summary` is only usable at visibility `pub(self)`
   = note: `#[warn(private_interfaces)]` on by default
```

(`median`에는 `unreachable_pub` 경고도 하나 더 붙습니다.)

- **`dead_code`("never used").** 책이 주목하라는 경고입니다. `pub`을 붙였는데도 "쓰이지 않는다"고 나오면, 실제로는 가시성 규칙 때문에 바깥에서 쓸 수 없다는 신호인 경우가 많습니다. 컴파일러는 크레이트 밖에서 닿을 수 있는 `pub` 항목은 누군가 쓸 수 있다고 보고 경고하지 않지만, 닿지 않는 `pub` 항목은 비공개 항목과 똑같이 검사합니다.
- **`unreachable_pub`(책에 없는 린트).** 기본값은 allow이고, 이 파일은 첫 줄의 `#![warn(unreachable_pub)]`로 켭니다. 크레이트 밖에서 닿지 않는 `pub` 항목을 모두 찾아, 실제로 닿는 범위를 `help:`로 제안합니다. 여기서는 `math`가 `stats_loose` 안의 비공개 모듈이라 `pub(super)`를 제안합니다. 린트 문서는 이 린트가 기존 코드에서 너무 많이 걸려 allow로 두었고, 언젠가 warn으로 올리기를 바란다고 적고 있습니다.
- **`private_interfaces`(책에 없는 린트, 기본 warn).** 공개 함수가 자신보다 좁은 타입을 시그니처에 쓰면 경고합니다. `summarize`는 `pub`이지만 바깥에서 부르면 ``error: type `stats_loose::Summary` is private``(에러 코드 없음)로 막혀서, 실제로는 쓸 수 없는 API입니다. 제네릭 바운드나 `where` 절에 쓴 경우는 `private_bounds`가 잡습니다.

**권장 코드.** 경고마다 "닿아야 하는 곳"을 정하고 선언을 그에 맞춥니다.

```rust
pub mod stats {
    pub use math::median;         // 공개하려던 함수: 공개 경로를 엶 → 경고 없음

    mod math {
        pub(super) fn mean(..)    // stats만 씀: 실제로 닿는 범위로 좁힘
        pub fn median(..)
    }

    pub struct Summary { .. }     // 공개 함수가 돌려주는 타입은 함께 공개(필드는 비공개 + 메서드)
    pub fn summarize(xs: &[i32]) -> Summary
}
```

좁혀 두면 경고가 다시 일합니다. 연습 3에서 확인하듯, 아무도 쓰지 않는 `pub(crate) fn variance`는 ``function `variance` is never used``로 잡히지만, 크레이트 밖에서 닿는 `pub fn variance`는 경고가 없습니다. 크레이트 안의 도우미를 `pub(crate)`로 적어 두면 더 이상 쓰지 않게 됐을 때 컴파일러가 알려 줍니다.

**예제 파일(바이너리 크레이트)에서도 같은가.** `examples/*.rs`는 파일마다 바이너리 크레이트입니다. 1.100.0에서는 바이너리 크레이트도 크레이트 루트에서 닿는 `pub` 항목을 `dead_code`로 경고하지 않습니다. 그런 항목은 1.97.0에서 추가된 별도 린트 `dead_code_pub_in_binary`(기본 allow)가 맡습니다. 그래서 이 챕터의 예제는 API 역할을 하는 모듈을 `pub mod`로 두어 라이브러리와 같은 조건에서 경고를 보여 줍니다. 비공개 모듈 안의 `pub` 항목은 라이브러리에서든 바이너리에서든 `dead_code` 대상입니다.

**지침을 따르지 않아도 되는 경우.** 작성 중인 코드에서는 아직 부르는 곳이 없는 함수가 많아 `dead_code` 경고가 잡음이 됩니다. 그때는 모듈에 `#[expect(dead_code, reason = "…")]`를 붙여 이유를 남깁니다. `unreachable_pub`은 기존 코드에 처음 켜면 경고가 많으므로, 새 모듈부터 또는 라이브러리 크레이트에만 켜는 것도 방법입니다. 크레이트 전체에 켜는 `Cargo.toml`의 `[lints]` 설정은 [`ch29-listen-to-clippy`](../ch29-listen-to-clippy/README.md)에서 다룹니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **바이너리 크레이트의 `pub` 항목과 `dead_code`.** 1.97.0 릴리스 노트에 allow 기본값의 `dead_code_pub_in_binary` 린트가 추가되었다고 적혀 있습니다. 1.100.0에서 바이너리 크레이트의 루트에 둔 쓰지 않는 `pub fn`은 경고가 없고, `-W dead_code_pub_in_binary`를 주면 ``in libraries, `pub` items can be used by dependent crates; in binaries, they cannot`` 노트와 함께 경고됩니다. 비공개 모듈 안의 쓰지 않는 `pub fn`은 라이브러리와 바이너리 모두에서 `dead_code` 경고가 납니다(같은 파일을 `--crate-type lib`와 바이너리로 컴파일해 비교). 책의 조언("never used 경고에 주목하라")은 그대로 통합니다.
- **비공개 타입이 드러난 공개 시그니처.** 1.74.0 릴리스 노트는 `private_in_public` 린트를 `private_interfaces`와 `private_bounds`로 바꿨다고 적습니다(RFC 2145). 1.100.0에서 `pub fn`의 시그니처에 비공개 타입을 쓰면 에러가 아니라 경고이고, 그 함수를 타입이 보이지 않는 곳에서 부를 때 에러 코드 없는 ``type `…` is private`` 에러가 납니다. `rustc --explain E0446`은 이제 트레이트 구현의 연관 타입에 비공개 타입을 쓴 경우를 예로 듭니다. 책은 이 주제를 다루지 않습니다.
- **반복자 어댑터의 위치.** 책은 `std::iter::adapters::map::Map::new`라고 적지만, 1.100.0 소스에서 정의는 `core::iter::adapters::map`에 있고 `std::iter`는 core의 것을 재내보냅니다. `Map::new`가 `pub(in crate::iter)`라는 점은 같습니다.
- **책에 없는 지침.** 다음은 책의 Item 22가 명시적으로 권하지 않는 내용입니다(2026-11-19에 온라인판에서 확인). 책은 가시성 문법과 단계, 열거형·트레이트의 예외, E0616·E0624, `dead_code` 경고, 되돌리기 어려움과 선택지를 남기는 이유, API 가이드라인의 "구조체는 비공개 필드를 가진다"까지 다룹니다.
  - 2.3절의 "지침을 따르지 않아도 되는 경우"(수동적인 데이터, 옵션 구조체): Rust API 가이드라인 C-STRUCT-PRIVATE의 예외와 ch7이 근거입니다.
  - 2.4절 전체(핸들의 필드를 감춰 위조를 막기): API 가이드라인 C-STRUCT-PRIVATE가 드는 불변식 이유와 ch15의 세대 ID가 근거입니다.
  - 2.5절의 "생성자를 `pub(in …)`으로 묶어 상태 전이를 건너뛰지 못하게 한다": 책은 반복자 어댑터를 `pub(in 경로)`의 예로만 듭니다. 표준 라이브러리 소스에서 `Map::new`를 `Iterator::map`만 부른다는 것을 확인한 결과가 근거입니다.
  - 2.6절의 `unreachable_pub`, `private_interfaces`·`private_bounds`, "좁히면 `dead_code`가 다시 일한다": rustc 1.100.0의 린트 문서와 직접 컴파일한 결과가 근거입니다.
  - 2.3절에서 이름만 든 `#[non_exhaustive]`, 그리고 이 챕터가 다루지 않는 `#[doc(hidden)]`과 봉인된 트레이트(sealed trait, 바깥 크레이트가 구현하지 못하게 막은 공개 트레이트, API 가이드라인 C-SEALED)는 책의 이 Item에 나오지 않습니다.
- **달라지지 않은 점.** 기본 비공개 규칙, 괄호가 붙은 가시성 네 가지의 뜻, 열거형 배리언트와 트레이트 메서드가 함께 공개된다는 점, E0616·E0624의 메시지 모양은 책의 설명과 같습니다.

### 2.8 기억할 것

- 공개 범위는 약속의 범위입니다. 닫힌 것을 여는 일은 쉽고, 연 것을 닫는 일은 사용자를 깹니다.
- 항목마다 "누가 쓰는가"를 묻고 그 범위까지만 엽니다. 같은 모듈 → 없음, 형제 → `pub(super)`, 조상 아래 → `pub(in crate::…)`, 크레이트 → `pub(crate)`, 사용자 → `pub`.
- 구조체의 필드는 비공개로 두고 메서드로 엽니다. 표현을 바꿀 자유와 불변식을 지킬 능력이 함께 남습니다.
- 하위 모듈은 감추고 공개 경로는 `pub use`로 정합니다. 사용자가 만들면 안 되는 타입의 생성자는 `pub(in …)`으로 묶습니다.
- `pub`인데 "never used"라면 길이 닫혀 있다는 신호입니다. `unreachable_pub`과 `private_interfaces`로 선언과 실제 범위를 맞춥니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행하며, 모두 종료 코드 0으로 끝나고 출력은 실행마다 같습니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 절 |
| --- | --- | --- | --- | --- |
| [`01_visibility_levels.rs`](examples/01_visibility_levels.rs) | 1 | 모든 모듈과 항목이 `pub` | `pub(super)`, `pub(in crate::cinema)`, `pub(crate)`, `pub use` | 2.2 |
| [`02_private_fields.rs`](examples/02_private_fields.rs) | 2 | `pub songs: Vec<String>` | 비공개 필드 + 메서드, `Vec` → `VecDeque` | 2.3 |
| [`03_opaque_handle.rs`](examples/03_opaque_handle.rs) | 3 | 필드가 공개인 `UserId`와 칸 | 필드를 감춘 `UserId`와 명부 | 2.4 |
| [`04_public_paths.rs`](examples/04_public_paths.rs) | 4 | `pub mod` 하위 모듈, `pub fn new` | 비공개 하위 모듈 + `pub use`, `pub(in crate::order) fn new` | 2.5 |
| [`05_visibility_lints.rs`](examples/05_visibility_lints.rs) | 5 | 경고 세 가지를 `#[expect]`로 표시 | 선언과 실제 범위를 맞춤 | 2.6 |

### 01_visibility_levels — 가시성 단계 고르기

1번 구역의 비권장 판에서는 `main`이 예매 절차를 건너뛰고 `take_seat`을 직접 불러 좌석 수와 표 수가 어긋납니다. 2번 구역의 권장 판은 같은 예매를 `book`으로만 하며, 대괄호 안은 `pub(crate)` 도우미 `audit::note`를 부른 곳의 `module_path!()`입니다.

```sh
cargo run -p er-ch22-minimize-visibility --example 01_visibility_levels
```

```text
1. 비권장: 모든 모듈과 항목이 pub
    좌석 0 예매: 표 발급 (9000원)
    main이 take_seat(4)를 직접 부름: Ok(())
    팔린 좌석 2석, 발급한 표 1장, 매출 9000원
    main이 내부 요금표를 직접 읽음: base_price(4) = 12000원
2. 권장: 쓰는 곳까지만 연다
    [01_visibility_levels::cinema::booking] 좌석 0 판매 기록
    좌석 0 예매: 표 발급 (9000원)
    좌석 4 예매(10000원 냄): 거절 - 12000원이 필요합니다
    [01_visibility_levels::cinema::booking] 좌석 4 판매 기록
    좌석 4 예매: 표 발급 (12000원)
    팔린 좌석 2석, 발급한 표 2장, 매출 21000원
3. pub(crate) 도우미는 크레이트 어디서나 부를 수 있다
    [01_visibility_levels] main에서 남긴 기록
```

`main` 끝의 주석 두 줄을 풀면 각각 ``error[E0624]: method `take_seat` is private``와 ``error[E0603]: module `pricing` is private``가 납니다.

### 02_private_fields — 비공개 필드와 내부 표현 바꾸기

세 사용자 코드가 같은 일(네 곡을 넣고 두 곡을 재생한 뒤 한 곡을 더 넣기)을 합니다. 결과는 모두 같습니다. 차이는 둘째 판으로 바꿀 때 고쳐야 하는 사용자 코드의 양이며, 비권장 판은 세 군데가 깨지고(2.3절) 권장 판은 한 곳도 깨지지 않습니다.

```sh
cargo run -p er-ch22-minimize-visibility --example 02_private_fields
```

```text
1. 비권장: 필드가 pub인 첫 판(Vec)
    재생: 아침 산책 → 비 오는 오후 / 대기: 퇴근길, 한밤의 라디오, 주말 드라이브
2. 권장: 필드를 감추고 메서드로 연다
    첫 판(Vec):         재생: 아침 산책 → 비 오는 오후 / 대기: 퇴근길, 한밤의 라디오, 주말 드라이브
    둘째 판(VecDeque):  재생: 아침 산책 → 비 오는 오후 / 대기: 퇴근길, 한밤의 라디오, 주말 드라이브
    두 판의 결과가 같음: true
```

### 03_opaque_handle — 필드를 감춘 핸들

바다(칸 0)와 하늘(칸 1)이 가입하고, 바다가 탈퇴한 뒤 별이 칸 0을 다시 씁니다. 비권장 판에서는 사용자 코드가 ID를 위조하고 칸의 세대를 되돌려, 바다의 옛 ID가 별을 가리키게 됩니다.

```sh
cargo run -p er-ch22-minimize-visibility --example 03_opaque_handle
```

```text
1. 비권장: UserId의 필드와 명부의 칸이 pub
    바다의 옛 ID로 조회: None
    별의 ID로 조회: Some("별")
    옛 ID의 세대를 고쳐 만든 ID로 조회: Some("별")
    칸의 세대를 되돌린 뒤 바다의 옛 ID로 조회: Some("별")
    같은 때 별의 ID로 조회: None
2. 권장: 필드를 감춘 UserId와 명부
    바다의 옛 ID로 조회: None
    별의 ID로 조회: Some("별")
    옛 ID로 탈퇴 시도: false
    ID끼리 비교는 가능: 바다의 옛 ID == 별의 ID → false
```

`main` 끝의 주석 두 줄을 풀면 E0451과 E0616이 납니다(메시지는 2.4절).

### 04_public_paths — 감춘 모듈과 공개 경로

1번 구역은 공개된 생성자로 결제 없이 `Paid`를 만들어 배송합니다. 2번 구역은 `Cart::checkout`을 거쳐야만 `Paid`를 얻습니다. 3번 구역의 화살표 오른쪽이 각 타입이 실제로 정의된 곳입니다.

```sh
cargo run -p er-ch22-minimize-visibility --example 04_public_paths
```

```text
1. 비권장: 내부 경로와 생성자가 공개
    결제 없이 만든 Paid: 상품 1개, 0원
    그대로 배송됨: 0원, 송장 FAKE-0001
2. 권장: 하위 모듈은 비공개, 타입은 pub use로 올림
    70000원 냄: 거절 - 80000원이 필요합니다
    80000원 냄: 상품 2개, 80000원 결제 완료
    배송: 80000원, 송장 KR-2026-1119
3. 공개 경로와 정의된 곳
    order::Paid     → 04_public_paths::order::states::paid::Paid
    order::Shipped  → 04_public_paths::order::states::shipped::Shipped
    std::iter::Map  → core::iter::adapters::map::Map<alloc::vec::into_iter::IntoIter<u32>, fn(u32) -> u32>
```

`main`의 주석 두 줄을 풀면 ``error[E0624]: associated function `new` is private``와 ``error[E0603]: module `states` is private``가 납니다.

### 05_visibility_lints — 경고로 공개 범위 점검하기

두 판의 실행 결과는 같습니다. 이 예제의 비교 대상은 컴파일 경고이며, 비권장 판의 `#[expect(…)]` 세 개를 지우면 2.6절의 경고가 납니다.

```sh
cargo run -p er-ch22-minimize-visibility --example 05_visibility_lints
```

```text
1. 비권장: 경고 세 개를 #[expect]로 표시한 판
    7개: 최저 -3, 최고 12, 평균 4.3
2. 권장: 닿아야 할 곳까지만, 닿아야 할 곳에는 반드시
    7개: 최저 -3, 최고 12, 평균 4.3
    stats::median → Some(5)
    stats::summarize → 최고 12
```

`main`의 주석 두 줄을 풀면 ``error: type `stats_loose::Summary` is private``(에러 코드 없음)와 ``error[E0603]: module `math` is private``가 납니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다). 본편과 앞 챕터에 이미 있는 가시성 에러는 다시 만들지 않았습니다. 비공개 함수·모듈의 E0603과 비공개 필드의 E0616은 [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 4.1·4.2절, 비공개 필드가 있는 구조체 리터럴의 E0451은 [ch16](../ch16-avoid-unsafe/README.md) 4.3절, 튜플 구조체 생성자의 E0603은 [ch6](../ch6-newtype-pattern/README.md) 2.3절에 있습니다.

```sh
cd appd-a/ch22-minimize-visibility
rustc --edition 2024 compile_fail/e0624_restricted_method.rs --out-dir /tmp/er-ch22
rustc --edition 2024 compile_fail/e0742_pub_in_not_ancestor.rs --out-dir /tmp/er-ch22
rustc --edition 2024 compile_fail/e0364_reexport_crate_item.rs --out-dir /tmp/er-ch22
```

### 4.1 범위를 좁힌 메서드를 범위 밖에서 부름 — E0624

파일: [`compile_fail/e0624_restricted_method.rs`](compile_fail/e0624_restricted_method.rs)

```text
error[E0624]: method `take_seat` is private
  --> compile_fail/e0624_restricted_method.rs:51:32
   |
27 |             pub(super) fn take_seat(&mut self, seat: usize) -> bool {
   |             ------------------------------------------------------- private method defined here
...
51 |     println!("직접 잡기: {}", hall.take_seat(4)); // E0624
   |                                    ^^^^^^^^^ private method
```

- **읽는 법**: `take_seat`에는 `pub(super)`가 붙어 있는데도 메시지는 "private"입니다. 범위를 좁힌 항목을 그 범위 밖에서 쓰면 컴파일러는 비공개 항목과 똑같이 취급합니다. `defined here` 표시가 선언을 보여 주므로 어느 단계로 열었는지 바로 확인할 수 있습니다.
- **원인**: `take_seat`은 `cinema::hall`에 있고 `pub(super)`라서 `cinema`와 그 아래에서만 보입니다. `main`은 크레이트 루트에 있습니다.
- **해결**: 이 에러는 대개 지킬 대상입니다. `main`은 `cinema::book`으로 예매합니다. 정말로 바깥에서 좌석만 잡는 기능이 필요하다면 `take_seat`을 넓히지 말고, 필요한 규칙을 지키는 공개 메서드를 따로 만듭니다. 같은 메시지가 다른 크레이트에서 `pub(crate)` 메서드를 부를 때도 나옵니다(2.1절).

### 4.2 `pub(in 경로)`에 조상이 아닌 모듈을 적음 — E0742

파일: [`compile_fail/e0742_pub_in_not_ancestor.rs`](compile_fail/e0742_pub_in_not_ancestor.rs)

```text
error[E0742]: visibilities can only be restricted to ancestor modules
  --> compile_fail/e0742_pub_in_not_ancestor.rs:21:20
   |
21 |             pub(in crate::cinema::seats) fn seat_rate(seat: usize) -> u32 {
   |                    ^^^^^^^^^^^^^^^^^^^^
```

- **읽는 법**: "ancestor modules"가 핵심입니다. `pub(in 경로)`는 "이 모듈에게 보여 준다"가 아니라 "이 조상 모듈 아래 전체에 보여 준다"는 뜻입니다.
- **원인**: `seat_rate`는 `cinema::pricing::rates`에 있습니다. 조상은 `rates`, `pricing`, `cinema`, `crate`뿐이고, `cinema::seats`는 옆 가지입니다. 가시성은 항목이 있는 곳에서 위로만 넓힐 수 있으므로, 옆 가지의 모듈 하나에게만 보여 주는 방법은 없습니다.
- **해결**: 쓰는 곳(`seats`)과 정의된 곳(`rates`)의 가장 가까운 공통 조상인 `pub(in crate::cinema)`를 적습니다(이렇게 고치면 `좌석 4: 12000원`을 출력함을 확인). 정말 `seats`만 써야 한다면 함수를 `seats` 안으로 옮기는 것이 맞습니다. 2015 에디션 시절의 `pub(in cinema)`처럼 경로를 `crate`·`self`·`super` 없이 적으면 E0742가 아니라 다른 에러가 납니다([Part 2 ch2](../../part2-editions/ch2-rust-2015-2018/README.md) 2.4절).

### 4.3 `pub(crate)` 항목을 `pub use`로 재내보냄 — E0364

파일: [`compile_fail/e0364_reexport_crate_item.rs`](compile_fail/e0364_reexport_crate_item.rs)

```text
error[E0364]: `total_of` is only public within the crate, and cannot be re-exported outside
  --> compile_fail/e0364_reexport_crate_item.rs:12:13
   |
12 |     pub use cart::total_of; // E0364
   |             ^^^^^^^^^^^^^^
   |
note: consider marking `total_of` as `pub` in the imported module
  --> compile_fail/e0364_reexport_crate_item.rs:12:13
   |
12 |     pub use cart::total_of; // E0364
   |             ^^^^^^^^^^^^^^
```

- **읽는 법**: 재내보내기는 원래 항목보다 넓게 열 수 없다는 뜻입니다. 공개 범위는 항목을 선언한 곳에서 정해지고, `pub use`는 이미 열린 항목에 경로를 하나 더 붙일 뿐입니다. 원래 항목이 `pub(in crate::order)`였다면 메시지는 ``…is private, and cannot be re-exported``가 됩니다.
- **원인**: 크레이트 안의 도우미로 만든 `total_of`(`pub(crate)`)를 사용자에게도 내놓으려 했습니다.
- **해결**: 먼저 "정말 공개할 것인가"를 정합니다. 공개한다면 `note:`대로 `cart` 안의 선언을 `pub fn total_of`로 바꿉니다(`cart` 모듈은 비공개 그대로 둬도 됩니다. 2.5절의 배치). 공개하지 않는다면 `pub use` 줄과 `main`의 `order::total_of` 호출을 지웁니다. 두 방법 모두 `합계 80000원`을 출력하며 컴파일됨을 확인했습니다. 크레이트 안에서만 짧은 경로를 쓰고 싶다면 `pub(crate) use cart::total_of;`로 재내보낼 수는 있습니다.

### 4.4 그 밖의 실수

모두 이 툴체인에서 임시 파일로 컴파일해 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| `pub enum`의 배리언트 필드나 `pub trait`의 메서드에 `pub`을 붙임 | E0449 ``visibility qualifiers are not permitted here`` | 지움. 열거형·트레이트와 같은 가시성을 가짐 (2.1절) |
| 비공개 모듈을 `pub use self::states;`로 재내보냄 | E0365 ``re-export of private `states` ``(같은 이름이라 E0255도 함께) | 모듈 대신 안의 항목을 재내보냄 (2.5절) |
| `pub fn`의 시그니처에 비공개 타입 | 경고 `private_interfaces`. 바깥에서 부르면 에러 코드 없는 ``type `…` is private`` | 타입을 함수만큼 열거나 함수를 좁힘 (2.6절) |
| 공개 필드의 타입을 `Vec`에서 `VecDeque`로 바꿈 | 사용자 코드에서 E0308, E0599 | 필드를 감추고 메서드로 엶 (2.3절) |
| 공개 필드가 있는 구조체에 필드를 추가 | 사용자의 구조체 리터럴에서 E0063 ``missing field`` | 필드를 감춤. 또는 `#[non_exhaustive]`([ch21](../ch21-semver/README.md)) |
| 바깥 크레이트에서 `pub(crate)` 메서드·함수를 부름 | E0624 / E0603 (… is private) | 공개 API로 쓸 것인지 정해 `pub`으로 (2.1절) |

## 5. 연습 문제

### 연습 1. 예매 취소 넣기

[`01_visibility_levels.rs`](examples/01_visibility_levels.rs)의 권장 판에 예매 취소 `cancel(hall: &mut Cinema, ticket: Ticket) -> Result<(), String>`을 추가하세요. 취소하면 좌석이 비고 매출에서 표 값이 빠집니다. 새로 만드는 함수와 메서드마다 가시성을 고르고, `main`에서 좌석만 비우는 일은 컴파일되지 않는지 확인하세요.

<details>
<summary>힌트</summary>

`cancel`은 사용자가 부르므로 `booking` 안에 `pub fn`으로 두고 `cinema`에서 `pub use`로 올립니다. 좌석을 비우는 `release_seat`과 매출을 빼는 메서드는 `take_seat`처럼 `hall` 안에 `pub(super)`로 둡니다. `cancel`이 `Ticket`을 값으로 받으면 같은 표로 두 번 취소하는 코드가 E0382(``use of moved value``)로 컴파일되지 않고, `Ticket`의 필드가 비공개라서 사용자가 표를 지어내면 E0451입니다. 이것이 2.4절의 핸들과 같은 원리입니다. `main`에서 `hall.release_seat(0)`을 쓰면 4.1절과 같은 E0624(``method `release_seat` is private``)가 납니다. 좌석 1을 예매한 뒤 취소하면 현황이 `팔린 좌석 2석, 발급한 표 2장, 매출 21000원`으로 돌아옵니다.

</details>

### 연습 2. 대기열에 중복 막기 더하기

[`02_private_fields.rs`](examples/02_private_fields.rs)에 셋째 판 `queue_v3`를 만드세요. 이미 대기열에 있는 곡은 `add`해도 다시 들어가지 않습니다. 사용자 코드 `listen_v3`의 본문은 `listen_v2`와 같아야 합니다. 그다음 비권장 판 `queue_pub_v1`의 구조체에 같은 목적의 `pub seen: HashSet<String>` 필드를 추가하면 `listen_pub_v1`에서 무슨 일이 생기는지 확인하세요.

<details>
<summary>힌트</summary>

`queue_v3`의 `PlayQueue`에 `songs: VecDeque<String>`과 `seen: HashSet<String>`을 두고, `add`에서 `self.seen.insert(title.to_string())`이 `true`일 때만 `push_back`합니다. `play_next`에서 꺼낸 곡은 `seen`에서도 지웁니다. 필드가 비공개라서 필드를 더해도 사용자 코드는 그대로입니다. 비권장 판은 ``error[E0063]: missing field `seen` in initializer of `queue_pub_v1::PlayQueue` ``로 깨집니다. 게다가 사용자가 `q.songs.push(..)`로 곡을 넣으면 `seen`이 갱신되지 않아 두 필드가 어긋납니다. 공개 필드로는 필드 사이의 규칙도 지킬 수 없습니다.

</details>

### 연습 3. 좁혀야 경고가 일한다

[`05_visibility_lints.rs`](examples/05_visibility_lints.rs)의 권장 판 `stats`에 분산을 구하는 `variance(xs: &[i32]) -> f64`를 추가하되 아무 데서도 부르지 마세요. 가시성을 `pub(crate)`로 했을 때와 `pub`으로 했을 때의 컴파일 경고를 비교하고, 그 차이를 설명하세요.

<details>
<summary>힌트</summary>

`pub(crate)`이면 ``warning: function `variance` is never used``가 나고, `pub`이면 아무 경고도 없습니다. `stats`가 `pub mod`라서 `pub fn variance`는 크레이트 밖에서 닿는 항목이고, 컴파일러는 바깥 사용자가 쓸 수도 있다고 보기 때문입니다(이 예제는 바이너리 크레이트지만 2.6절에서 본 것처럼 1.100.0에서는 같은 기준입니다). 그래서 크레이트 안의 도우미에 `pub`을 붙이면 더 이상 쓰지 않게 돼도 아무도 알려 주지 않습니다. `cargo build`로 경고를 보려면 예제를 고친 뒤 `cargo build -p er-ch22-minimize-visibility --example 05_visibility_lints`를 실행합니다.

</details>

### 연습 4. 가시성 고르기

다음 항목에 붙일 가시성을 고르고 이유를 적어 보세요.

1. 라이브러리의 여러 모듈이 쓰는 날짜 파싱 도우미
2. `parser::lexer` 모듈의 토큰 타입. `parser::grammar` 모듈만 씀
3. 공개 함수 `load_config`가 돌려주는 `Config` 구조체의 필드 `retries: u8`(0~10만 허용)
4. 공개 타입 `Matrix`의 행 우선 저장 `Vec<f64>` 필드
5. 단위 테스트에서만 부르는 비공개 함수

<details>
<summary>힌트</summary>

1은 `pub(crate)`입니다(2.2절). 2는 `lexer`의 부모가 `parser`이므로 `pub(super)`면 형제 `grammar`가 씁니다. 3은 범위 규칙이 있으므로 필드를 비공개로 두고 `retries()` 읽기 메서드와 검사하는 생성자(또는 ch7의 빌더)를 엽니다(2.3절, ch6 2.3절). 4는 저장 방식(행 우선, 열 우선, 희소 행렬)을 바꿀 가능성이 크므로 비공개로 두고 `get(row, col)` 같은 메서드를 엽니다. 5는 넓힐 필요가 없습니다. `#[cfg(test)] mod tests`는 자식 모듈이라 비공개 함수를 부를 수 있습니다(Part 0 ch11 2.8절).

</details>

## 6. 참고 자료

- 기준 문서: [Item 22: Minimize visibility](https://effective-rust.com/visibility.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- The Rust Reference: [Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html) — 두 가지 접근 규칙, `pub(in path)`·`pub(crate)`·`pub(super)`·`pub(self)`의 정의, 재내보내기와 가시성
- Rust API 가이드라인 [Future proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html): [C-STRUCT-PRIVATE](https://rust-lang.github.io/api-guidelines/future-proofing.html#structs-have-private-fields-c-struct-private)(구조체는 비공개 필드를 가진다), C-SEALED(봉인된 트레이트), C-NEWTYPE-HIDE(뉴타입으로 구현 세부를 감춤)
- rustc 린트: [`unreachable_pub`](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unreachable-pub), [`dead_code_pub_in_binary`](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#dead-code-pub-in-binary), [`dead_code`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#dead-code), [`private_interfaces`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#private-interfaces), [`private_bounds`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#private-bounds)
- 표준 라이브러리 소스: `rustc --print sysroot` 아래 `share/doc/rust/html/src/core/iter/`의 `mod.rs`, `adapters/mod.rs`, `adapters/map.rs`, `traits/iterator.rs`(2.5절)
- 에러 설명: `rustc --explain E0624`, `rustc --explain E0742`, `rustc --explain E0364`, `rustc --explain E0365`, `rustc --explain E0449`, `rustc --explain E0446`
