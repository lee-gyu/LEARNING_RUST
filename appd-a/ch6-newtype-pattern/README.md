# ch6. 뉴타입 패턴 적극 활용하기

경사각 `5.0`이 도(degree)인지 라디안(radian)인지, 함수에 넘긴 `2, 5`가 "2쪽, 쪽당 5개"인지 "쪽당 2개, 5쪽"인지는 `f64`와 `usize`만 보고는 알 수 없습니다. 같은 표현을 쓰는 서로 다른 뜻을 컴파일러가 구별하게 만드는 가장 가벼운 도구가 **뉴타입 패턴(newtype pattern)**, 즉 필드 하나짜리 튜플 구조체입니다. [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)에서 뉴타입이 무엇인지 봤고, [ch1](../ch1-express-data-structures/README.md)은 세 가지 숙제를 이 챕터로 미뤘습니다. 줄과 칸을 바꿔 넘기는 실수까지 컴파일 에러로 만들기, `bool` 인자를 구별하기, `percent: 150`처럼 타입은 맞지만 범위가 틀린 값을 막기입니다. 이 챕터는 그 숙제를 풀면서 **언제 뉴타입을 만들 만한가, 그 대가는 무엇인가**를 다룹니다. 대가는 분명합니다. 뉴타입은 안쪽 타입이 가진 트레이트와 메서드를 하나도 물려받지 않으므로, 필요한 것을 하나씩 다시 열어 줘야 합니다.

- 원제: [Item 6: Embrace the newtype pattern](https://effective-rust.com/newtype.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch6-newtype-pattern`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

문법은 다시 설명하지 않습니다. 다음 본편 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch5](../../part0-getting-started/ch5-structs/README.md) 2.7절: 튜플 구조체
- [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.12절: 생성자에서 검사하는 검증 타입 `Guess`
- [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.7절: 고아 규칙과 일관성
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.4~2.5절: `Deref`와 역참조 강제 변환
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.12~2.13절: 뉴타입 패턴과 타입 별칭
- [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md): 연산자, 비교, 해시 트레이트 구현법

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 타입 별칭(`type`)과 뉴타입의 차이를 설명하고, 단위가 다른 값을 뉴타입으로 구별해 단위 혼동을 컴파일 에러로 만들 수 있다.
2. 타입이 같고 뜻이 다른 인자(순서를 바꾸기 쉬운 수, `bool` 플래그)를 뉴타입으로 구별하고, 필드를 감춘 뉴타입의 생성자에서 값의 범위를 검증할 수 있다.
3. 뉴타입에 derive할 트레이트와 구현할 연산자를 고를 수 있고, `Deref`로 안쪽 타입을 통째로 열면 안 되는 이유를 설명할 수 있다.
4. 고아 규칙에 막힌 구현을 뉴타입으로 우회하고, 내 트레이트를 정의하는 방법과 비교해 어느 쪽을 쓸지 고를 수 있다.
5. 뉴타입이 치르는 위임 구현의 비용을 설명하고, 서식 지정을 잃지 않게 위임할 수 있으며, 뉴타입이 과한 경우를 판단할 수 있다.

## 2. 핵심 개념

지침마다 문제 상황, 비권장 코드, 권장 코드, 지침을 따르지 않아도 되는 경우 순서로 설명합니다. 코드는 핵심만 싣고, 두 형태를 함께 실행해 비교하는 전체 코드는 `examples/`에 있습니다.

### 2.1 단위는 별칭이 아니라 뉴타입으로

**문제 상황.** 경사로의 높이를 "길이 × sin(경사각)"으로 계산합니다. 현장 도면은 경사각을 도로 적는데, 높이를 계산하는 함수는 `f64::sin`을 쓰므로 라디안을 받아야 합니다. 두 코드를 서로 다른 사람이 썼습니다.

**비권장 코드.** 단위를 타입 별칭으로 적어 둡니다.

```rust
pub type Degrees = f64;
pub type Radians = f64;

pub fn ramp_height(length_m: f64, slope: Radians) -> f64 {
    length_m * slope.sin()
}

let slope: Degrees = 5.0;
ramp_height(12.0, slope) // 변환을 빠뜨렸지만 컴파일됨 → -11.51m
```

별칭은 새 타입을 만들지 않습니다(Part 0 ch19 2.13절). `Degrees`, `Radians`, `f64`는 컴파일러에게 모두 같은 타입이므로, 시그니처의 `slope: Radians`는 사람만 읽을 수 있는 주석입니다. [`01_unit_newtype.rs`](examples/01_unit_newtype.rs)에서 길이 12m 경사로에 도면의 5도를 그대로 넘기면 높이가 -11.51m로 나옵니다. 이 값은 누가 봐도 이상하지만, 같은 실수가 3도에서는 1.69m를 냅니다. 맞는 값은 0.63m인데, 1.69m도 경사로 높이로는 그럴듯해서 눈으로 걸러지지 않습니다. 단위 실수는 늘 엉뚱한 숫자로 드러나지 않습니다.

**권장 코드.** 단위마다 뉴타입을 만들고, 단위를 바꾸는 길을 메서드 하나로 정합니다.

```rust
pub struct Degrees(pub f64);
pub struct Radians(pub f64);

impl Degrees {
    pub fn to_radians(self) -> Radians {
        Radians(self.0.to_radians())
    }
}

pub fn ramp_height(length_m: f64, slope: Radians) -> f64 {
    length_m * slope.0.sin()
}

ramp_height(12.0, slope.to_radians()) // 0.63m (3도), 1.05m (5도)
```

이제 `ramp_height(12.0, slope)`는 ``expected `Radians`, found `Degrees` ``(E0308)로 컴파일되지 않습니다(4.1절). 비권장 판의 실수가 테스트나 리뷰에 기대지 않고 빌드 단계에서 걸립니다. 변환은 바깥(도면)에서 값이 들어오는 **경계(boundary)** 에서 `to_radians()`로 한 번 하고, 그 뒤로는 `Radians`로 다닙니다. [ch1](../ch1-express-data-structures/README.md) 2.1절의 "변환은 경계에서"와 같은 생각입니다.

실행 중 비용은 없습니다. Part 0 ch19 2.12절에서 본 것처럼 뉴타입의 크기는 안쪽 타입과 같습니다. 크기뿐 아니라 메모리 배치까지 안쪽 타입과 같다고 보장해야 할 때(C와 주고받을 때 등) 붙이는 `#[repr(transparent)]`는 `ch34-ffi-boundaries`와 Part 10 `ch2-data-layout`에서 다룹니다.

두 단위 사이에 `impl From<Degrees> for Radians`를 구현해 두면, 같은 E0308에 ``help: call `Into::into` on this expression to convert `Degrees` into `Radians` ``라는 도움말이 붙습니다(구현이 없으면 이 도움말도 없습니다). 변환을 `From`으로 열지, 이름 있는 메서드로 열지는 `ch5-type-conversions`에서 다룹니다.

**따르지 않아도 되는 경우.**

- 시간이라면 직접 만들지 않습니다. 표준 라이브러리의 `Duration`이 이미 단위를 감춘 타입입니다. `Duration::from_millis(1500)`과 `Duration::from_secs(1)`처럼 단위는 만들 때만 드러나고, 그 뒤로는 초와 밀리초가 섞일 일이 없습니다.
- 프로그램 전체가 한 단위만 쓰고 바깥에서 다른 단위가 들어올 일이 없다면, 섞일 상대가 없으므로 뉴타입이 막아 줄 실수도 없습니다.
- 별칭이 맞는 곳은 따로 있습니다. `io::Result<T>`처럼 **긴 타입 이름을 줄이는** 데 씁니다. 구별하려는 목적이라면 별칭은 아무것도 하지 않습니다.

### 2.2 같은 타입의 인자는 뉴타입으로 구별한다

[ch1](../ch1-express-data-structures/README.md) 2.2절의 `SeatPos { row, col }`은 이름을 붙여 실수를 **보이게** 했지만 막지는 못했습니다. `draw(pos.row, pos.col)`도 컴파일되었습니다. 이 절은 그 실수를 컴파일 에러로 바꿉니다.

**문제 상황.** 게시판 글을 한 쪽에 몇 개씩 나눠 보여 주는 함수가 쪽 번호, 쪽당 개수, 최신 글부터 보일지 여부를 받습니다.

**비권장 코드.**

```rust
fn list_page_raw<'a>(posts: &[&'a str], page: usize, per_page: usize, newest_first: bool) -> Vec<&'a str>

list_page_raw(&POSTS, 2, 5, true) // 의도: 2쪽, 쪽당 5개
list_page_raw(&POSTS, 5, 2, true) // "개수, 쪽" 순서로 기억한 사람의 호출: 컴파일됨
```

SQL의 `LIMIT 5 OFFSET ..`처럼 개수를 먼저 적는 데 익숙한 사람은 두 수를 바꿔 넘기기 쉽습니다. [`02_distinct_args.rs`](examples/02_distinct_args.rs)에서 의도한 호출은 글 다섯 개(`"글07"`~`"글03"`)를, 바꾼 호출은 5쪽의 글 두 개(`"글04"`, `"글03"`)를 돌려줍니다. 컴파일러는 둘 다 `usize`라서 알 수 없고, 호출 지점의 `true`는 시그니처를 찾아보기 전에는 뜻을 알 수 없습니다.

**권장 코드.** 인자마다 뉴타입을 만들고, 매개변수 자리에서 바로 구조 분해합니다.

```rust
struct Page(pub usize);
struct PerPage(pub usize);
struct NewestFirst(pub bool);

fn list_page<'a>(
    posts: &[&'a str],
    Page(page): Page,
    PerPage(per_page): PerPage,
    NewestFirst(newest_first): NewestFirst,
) -> Vec<&'a str>

list_page(&POSTS, Page(2), PerPage(5), NewestFirst(true))
```

매개변수 패턴으로 풀었으므로 본문은 비권장 판과 한 글자도 다르지 않습니다. 바뀐 것은 시그니처, 즉 부르는 쪽과의 약속뿐입니다. 두 인자의 순서를 바꾸면 ch1 4.2절에서 열거형 인자를 바꿨을 때와 같은 에러가 납니다(핵심 줄만 옮겼습니다). ch1의 좌석 배치도 같은 방법으로 풉니다. 필드를 `row: Row`, `col: Col`처럼 뉴타입으로 두고 `draw(x: Col, y: Row)`로 받으면 `draw(pos.row, pos.col)`이 이 에러로 멈춥니다.

```text
error[E0308]: arguments to this function are incorrect
help: swap these arguments
```

`bool`을 감싸는 방법은 ch1 2.3절의 열거형과 견줘 고릅니다.

| | `bool` 뉴타입 `NewestFirst(bool)` | 열거형 `Order::{Newest, Oldest}` |
| --- | --- | --- |
| 호출 지점 | `NewestFirst(true)` | `Order::Newest` |
| 본문에서 쓰기 | `if newest_first`, `!newest_first` | `match` |
| 선택지 늘리기 | 할 수 없음 | 배리언트를 추가 |

선택지가 앞으로도 참과 거짓 둘뿐이라면 뉴타입이 가볍습니다. "인기순"처럼 셋째 선택지가 생길 수 있다면 열거형을 고릅니다.

이 판에도 남은 구멍이 있습니다. 필드가 공개라서 `Page(0)`을 만들 수 있고, 그러면 `(page - 1) * per_page`가 넘칩니다. 값의 범위까지 지키는 방법이 다음 절입니다.

**따르지 않아도 되는 경우.**

- 인자가 하나뿐이고 함수 이름이 뜻을 말해 줄 때(`set_nonblocking(true)`)는 그대로 둡니다(ch1 2.3절).
- 인자가 여럿이면 이름 붙은 필드의 구조체 하나로 묶는 것도 방법입니다. `Query { per_page: 5, page: 2 }`는 필드 이름으로 맞추므로 적는 순서가 결과를 바꾸지 않습니다. 필드가 많아 만드는 일 자체가 번거로워지면 빌더 패턴(`ch7-builders`)을 씁니다.
- 바로 옆에서 한두 번 부르는 비공개 도우미 함수라면 타입을 만드는 비용이 막아 줄 실수보다 큽니다.

### 2.3 범위가 정해진 값은 생성자에서 검증한다

[ch1](../ch1-express-data-structures/README.md) 2.4절은 `Discount::Rate { percent: 150 }`을 여전히 만들 수 있다는 문제를 남겼습니다. 타입은 맞지만 값의 범위가 틀린 경우입니다. 답은 Part 0 ch9 2.12절의 `Guess`와 같은 구조를 뉴타입으로 만드는 것입니다.

**문제 상황.** 다운로드 서버가 보내 온 진행률을 막대로 그립니다. 서버에 버그가 있어 가끔 150이 들어옵니다.

**비권장 코드.**

```rust
fn progress_bar_raw(percent: u8) -> String {
    let filled = usize::from(percent) * WIDTH / 100;
    format!("[{:<WIDTH$}] {percent:>3}%", "#".repeat(filled))
}
```

`u8`은 0~255를 담으므로 "0~100이어야 한다"는 규칙은 함수 이름에만 있습니다. [`03_validated_newtype.rs`](examples/03_validated_newtype.rs)에서 150을 넣으면 `[###############] 150%`처럼 막대가 틀을 뚫습니다. 이 함수 안에서 검사하더라도, 진행률을 받는 함수가 늘 때마다 같은 검사를 반복해야 하고 하나라도 빠뜨리면 그 함수가 구멍이 됩니다.

**권장 코드.** 필드를 감춘 뉴타입을 모듈 안에 두고, 검사하는 생성자를 유일한 통로로 둡니다.

```rust
mod percent {
    pub struct Percent(u8); // 필드에 pub이 없음

    impl Percent {
        pub fn new(value: u8) -> Option<Percent> {
            (value <= 100).then_some(Percent(value))
        }
        pub fn get(self) -> u8 { self.0 }
        pub fn remaining(self) -> u8 { 100 - self.0 } // 넘칠 수 없음
    }
}

fn progress_bar(p: Percent) -> String // 다시 검사하지 않음
```

Part 0 ch9의 세 가지 장치가 그대로 쓰입니다. 비공개 필드, 검사하는 생성자, 값을 바꾸는 메서드가 없다는 것입니다. 모듈 밖에서 `percent::Percent(150)`이라고 쓰면 `` error[E0603]: tuple struct constructor `Percent` is private ``입니다. `Guess`처럼 이름 붙은 필드였다면 E0451이었을 자리가, 튜플 구조체에서는 "생성자가 비공개"라는 E0603으로 나옵니다. 매개변수 자리에서 `Percent(v): Percent`로 구조 분해하는 것도 같은 E0603으로 막힙니다. 안쪽 값은 `get()`으로만 읽습니다.

검사는 경계에서 한 번만 합니다. 예제의 `main`은 서버에서 받은 `u8`을 `Percent::new`로 바꾸면서 150을 거부하고, 그 뒤의 `progress_bar`와 `remaining`은 검사하지 않습니다. `remaining`의 `100 - self.0`은 불변식("값은 100 이하") 덕분에 넘치지 않으므로 `checked_sub`도 필요 없습니다. 시그니처의 `Percent`가 "검사가 끝난 값"이라는 증명 역할을 합니다.

`new`가 `Option`을 돌려주는 것은 실패 이유가 "범위를 벗어남" 하나뿐이기 때문입니다(ch1 2.6절). 이유를 담은 에러 타입을 설계하는 방법은 `ch4-idiomatic-errors`, 같은 검사를 `TryFrom<u8>`으로 여는 방법은 `ch5-type-conversions`에서 다룹니다.

검증 뉴타입에 derive를 붙일 때는 **값을 새로 만드는 derive**를 조심합니다. `derive(Default)`는 필드마다 `default()`를 불러 생성자를 거치지 않고 값을 만듭니다. `Percent`라면 `Percent(0)`이라 문제없지만, 1 이상이어야 하는 쪽당 개수에 붙이면 무효한 값 `PerPage(0)`이 생깁니다. 반대로 `Clone`, `PartialEq`, `Ord`처럼 있는 값을 복제하거나 비교하기만 하는 derive는 불변식을 깨지 않습니다.

표준 라이브러리에도 같은 설계가 많습니다. `NonZero::<u32>::new(0)`은 `None`을, `String::from_utf8(bytes)`는 바이트가 올바른 UTF-8일 때만 `Ok(String)`을 돌려줍니다. 그래서 `String`을 받는 함수는 UTF-8을 다시 검사하지 않습니다.

**따르지 않아도 되는 경우.**

- 허용 범위가 실행 중에 정해질 때(관리자가 정하는 쪽당 최대 개수 등)는 타입 하나에 범위를 담기 어렵습니다. 값을 쓰는 곳에서 그때의 설정으로 검사합니다.
- 값이 한 곳에서 들어와 그 자리에서 쓰이고 끝난다면 그 자리의 검사 한 번으로 충분합니다. 검증 뉴타입은 같은 값이 여러 함수를 거칠 때 값어치를 합니다.

### 2.4 열어 줄 연산을 고른다

뉴타입을 만들고 나면 `.0`을 적는 일이 귀찮아집니다. 안쪽 타입의 메서드와 연산을 한꺼번에 쓰고 싶어지고, Part 0 ch14에서 배운 `Deref`가 그 방법처럼 보입니다.

**문제 상황.** 드론이 x축에서 반시계 방향으로 30도를 향해 있다가 45도와 60도를 차례로 돌고, 그 방향으로 10m 전진합니다. 방향은 `Degrees`로 다룹니다.

**비권장 코드.**

```rust
impl Deref for DegreesDeref {
    type Target = f64;
    fn deref(&self) -> &f64 { &self.0 }
}

let (dx, dy) = (10.0 * heading.cos(), 10.0 * heading.sin()); // 컴파일됨
```

`DegreesDeref`에는 `cos`가 없지만, 메서드 탐색이 자동 역참조를 거쳐 `f64::cos`를 찾아냅니다(Part 0 ch5 2.10절, Part 0 ch14 2.5절). `f64::cos`는 라디안을 받으므로 135도를 135라디안으로 계산해, [`04_choose_operations.rs`](examples/04_choose_operations.rs)는 맞는 값 `(-7.07, 7.07)` 대신 `(-9.96, 0.88)`을 출력합니다. 2.1절에서 별칭을 썼을 때와 같은 종류의 실수입니다. `Deref`를 구현한 순간 뉴타입은 메서드 호출에서 다시 `f64`처럼 행동하고, 단위를 구별하려던 목적이 사라집니다. 맞는 변환 메서드를 불러도 `heading.to_radians()`는 `Radians`가 아니라 맨 `f64`를 돌려주므로, 단위 정보가 또 한 번 사라집니다. `&DegreesDeref`가 `&f64` 매개변수에 그대로 넘어가는 역참조 강제 변환도 함께 열립니다.

Part 0 ch19 2.12절의 `Wrapper(Vec<String>)`는 `Deref`를 구현해 `w.len()`을 썼습니다. 그 뉴타입의 목적은 고아 규칙을 피하는 것이었으므로 안쪽 메서드를 여는 것이 목적을 해치지 않았습니다. 목적이 **구별**인 뉴타입에서는 사정이 다릅니다. Rust API 가이드라인도 `Deref`는 스마트 포인터에만 구현하라고 권합니다(C-DEREF). `Deref`가 메서드 탐색에 끼어드는 방식이 스마트 포인터를 위해 설계되었기 때문입니다.

**권장 코드.** 뜻이 있는 연산만 하나씩 엽니다.

```rust
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Degrees(f64);

impl Add for Degrees { /* 도 + 도 = 도 */ }
impl Degrees { fn to_radians(self) -> Radians { .. } }
impl Radians { fn cos(self) -> f64 { .. } fn sin(self) -> f64 { .. } }

let rad = heading.to_radians();
let (dx, dy) = (10.0 * rad.cos(), 10.0 * rad.sin()); // (-7.07, 7.07)
```

`heading.cos()`는 이제 E0599입니다(4.2절). 고르는 기준은 "이 연산이 이 단위에서도 뜻이 있는가"입니다.

| 후보 | `Degrees`에서 | 이유 |
| --- | --- | --- |
| `Debug`, `Clone`, `Copy` | derive | 작은 값이고, 출력하거나 복사해도 뜻이 바뀌지 않음 |
| `PartialEq`, `PartialOrd` | derive | 같은 단위끼리의 비교는 뜻이 있음(예제의 `heading > Degrees(90.0)`) |
| `Eq`, `Ord`, `Hash` | derive할 수 없음 | `f64`가 구현하지 않으므로 E0277 |
| `Add<Degrees>` | 구현 | 도 + 도 = 도 |
| `Mul<f64>` | 필요하면 구현 | 각도 × 배율 = 각도 |
| `Mul<Degrees>`, `Add<Radians>` | 구현하지 않음 | 뜻이 없음. 컴파일러가 막음(E0369, E0308) |
| `cos`, `sin` | `Radians`에만 | 라디안을 받는 계산 |
| `Deref<Target = f64>` | 구현하지 않음 | `f64`의 메서드가 모두 열림 |

구현하지 않은 조합이 어떤 에러가 되는지도 알아 두면 좋습니다. `Add` 구현이 아예 없는 타입끼리 더하면 `` error[E0369]: cannot add `Radians` to `Radians` ``이고, `Add<Degrees>`만 있는 `Degrees`에 `Radians`를 더하면 오른쪽 타입이 틀렸다는 E0308이 됩니다(4.1절).

표준 라이브러리의 `Instant`와 `Duration`이 이 기준을 잘 보여 줍니다. 시각 + 시간 간격 = 시각(`Instant + Duration`), 시각 − 시각 = 시간 간격(`Instant - Instant`)은 구현되어 있지만, 시각 + 시각은 뜻이 없으므로 구현되어 있지 않습니다. `Instant + Instant`는 ``expected `Duration`, found `Instant` ``(E0308)입니다. 연산자 구현 문법은 [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 2.8절, 공개 타입에 어떤 표준 트레이트를 붙일지는 `ch10-standard-traits`에서 다룹니다.

**따르지 않아도 되는 경우.** 뉴타입의 목적이 구별이 아니라 트레이트 구현 하나를 얹는 것뿐이라면(2.5절) 안쪽 메서드를 열어도 목적을 해치지 않습니다. 그때도 `Deref`보다는 2.6절의 `as_str()` 같은 읽기용 메서드 하나가 무엇을 여는지 분명합니다.

### 2.5 고아 규칙에 막히면 감싼다

[Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.7절에서 고아 규칙과 그 이유인 일관성을 배웠고, Part 0 ch19 4.4절에서 `Vec<String>`에 `Display`를 구현하려다 E0117을 봤습니다. 이 절은 규칙에 막혔을 때 고를 수 있는 두 길을 비교합니다.

**문제 상황.** 빌드 단계별 소요 시간을 보고하는데, 보고를 출력하는 함수 `print_report<T: Display>`는 다른 크레이트의 것이라 고칠 수 없습니다. 그런데 `Duration`은 디버그 형식만 있어서 `83.4s`, `600ms`처럼 단위가 제각각으로 나옵니다. "1분 23.4초" 형식으로 보여 주고 싶습니다.

**비권장 코드.**

```rust
impl fmt::Display for Duration { .. } // E0117
```

`Display`도 `Duration`도 이 크레이트의 것이 아니므로 고아 규칙에 걸립니다(4.3절). 이 구현이 허용된다면, 표준 라이브러리가 나중에 `Duration`에 `Display`를 구현하거나 다른 크레이트가 같은 구현을 했을 때 어느 구현을 쓸지 정할 수 없습니다. 에러의 마지막 줄 ``define and implement a trait or new type instead``가 두 가지 길을 알려 줍니다.

**권장 코드.** [`05_orphan_rule_wrapper.rs`](examples/05_orphan_rule_wrapper.rs)는 두 길을 모두 보여 줍니다.

```rust
// 길 1: 내 트레이트(트레이트가 내 것이므로 허용)
trait HumanTime { fn human(&self) -> String; }
impl HumanTime for Duration { .. }

// 길 2: 뉴타입(타입이 내 것이므로 허용)
struct Elapsed(Duration);
impl fmt::Display for Elapsed { .. }

print_report(&STEPS.map(|(n, d)| (n, Elapsed(d))));
```

| | 내 트레이트 `HumanTime` | 뉴타입 `Elapsed` |
| --- | --- | --- |
| 직접 부르기 | `d.human()` | `Elapsed(d).to_string()` |
| `T: Display`를 요구하는 함수에 넘기기 | 할 수 없음. `String`으로 바꿔 넘김 | 그대로 넘김 |
| `Display`를 구현하면 따라오는 것(`to_string`, `{}`) | 없음 | 모두 |
| `Duration`의 연산(합계, 비교) | 그대로 씀 | 그대로 씀(출력할 때만 감쌈) |

고르는 기준은 **바깥의 API가 그 트레이트를 요구하는가**입니다. `print_report`처럼 `Display` 바운드를 건 함수, `HashMap` 키의 `Hash + Eq`, `str::parse`의 `FromStr`은 내 트레이트로 대신할 수 없습니다. `print_report(&STEPS)`는 `` `Duration` doesn't implement `std::fmt::Display` ``(E0277)이므로, 내 트레이트 쪽은 단계마다 `String`을 새로 만들어 넘겨야 합니다. 바깥 API가 관여하지 않고 내 코드에서만 부를 메서드라면 내 트레이트가 더 가볍습니다.

예제의 `Elapsed`는 합계 같은 계산을 `Duration`으로 하고 **출력할 때만** 감쌉니다. 그래서 `Duration`의 `Sum`, `Add`, 비교를 `Elapsed`에 다시 구현하지 않아도 됩니다. 표준 라이브러리도 같은 방법을 씁니다. `Path`는 운영체제에 따라 유니코드가 아닌 바이트를 담을 수 있어서 `Display`를 구현하지 않고, `path.display()`가 `Display`를 구현한 도우미 구조체 `std::path::Display`를 돌려줍니다. `println!("{}", path)`라고 쓰면 컴파일러가 ``call `.display()` on it``이라고 알려 줍니다.

일관성 규칙 때문에 한 타입에는 같은 트레이트의 구현이 하나뿐이라는 점도 뉴타입으로 풉니다. `u32`에는 이미 오름차순 `Ord`가 있으므로, 내림차순으로 정렬하려면 `pub struct Reverse<T>(pub T);`로 감싼 `std::cmp::Reverse`를 씁니다. "같은 타입, 다른 구현"이 필요하면 감쌉니다.

**따르지 않아도 되는 경우.** 트레이트나 타입 가운데 하나가 내 것이면 그냥 구현합니다. 둘 다 남의 것이어도 바깥 API가 그 트레이트를 요구하지 않는다면 내 트레이트로 충분합니다.

### 2.6 위임 구현의 비용을 알고 치른다

뉴타입은 안쪽 타입의 트레이트와 메서드를 하나도 물려받지 않습니다. 이것이 2.4절에서 단위를 지켜 준 성질이자, 뉴타입의 비용입니다. 필요한 것은 하나씩 다시 열어야 하고, 손으로 넘겨 주는 이 **위임 구현**에도 틀릴 자리가 있습니다.

**문제 상황.** 주문서의 상품 코드 `Sku(String)`와 수량 `Qty(u32)`를 표로 출력하고 합계를 냅니다. 표는 부르는 쪽이 `{sku:<8}`, `{qty:>4}`로 칸을 맞춥니다. 뉴타입에는 아무것도 없으므로 `sku.starts_with('A')`는 E0599, `iter().sum::<Qty>()`는 `` a value of type `Qty` cannot be made by summing an iterator over elements of type `Qty` ``(E0277)입니다.

**비권장 코드.** 서둘러 위임합니다.

```rust
impl fmt::Display for Sku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

let total = Qty(lines.iter().map(|(_, q)| q.0).sum()); // 꺼내서 더하고 다시 감쌈
```

[`06_delegation_cost.rs`](examples/06_delegation_cost.rs)에서 이 구현으로 표를 출력하면 `|A-100|3|`처럼 너비가 모두 무시됩니다. `write!(f, "{}", ..)`는 새 서식 문자열로 쓰기 때문에, 부르는 쪽이 `f`에 담아 보낸 너비와 정렬이 안쪽 `{}`까지 전달되지 않습니다(원리는 [Part 1 ch3](../../part1-standard-lib/ch3-strings-fmt/README.md) 2.10절). 합계는 맞지만, `.0`으로 꺼내 `u32`로 더하는 동안에는 단위 보호가 없고 이런 코드가 합계를 내는 곳마다 생깁니다.

**권장 코드.**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sku(String);

impl Sku {
    pub fn as_str(&self) -> &str { &self.0 }
}

impl fmt::Display for Sku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f) // f를 그대로 넘김
    }
}

impl Sum for Qty { .. }
let total: Qty = lines.iter().map(|(_, q)| *q).sum();
```

안쪽 값의 `Display::fmt`에 `f`를 그대로 넘기면 너비, 정렬, 정밀도가 함께 넘어가서 `|A-100   |   3|`처럼 칸이 맞습니다. `String`의 메서드는 하나씩 위임하는 대신 읽기 전용 `&str`을 내주는 `as_str()` 하나를 두어, `sku.as_str().starts_with('A')`처럼 `&str`의 메서드를 모두 쓰게 합니다. `&mut String`은 내주지 않으므로 바깥에서 코드를 고쳐 쓸 길은 열리지 않습니다.

무엇을 어떻게 되찾는지 정리하면 다음과 같습니다.

| 안쪽 타입에 있던 것 | 뉴타입에서 되찾는 방법 | 예제 06 |
| --- | --- | --- |
| `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Default` | 안쪽 타입이 구현했다면 derive 한 줄 | `#[derive(..)]` |
| `Display` | 손으로 위임. `f`를 그대로 넘김 | `fmt::Display::fmt(&self.0, f)` |
| 연산자(`Add` 등), `Sum` | 손으로, 뜻이 있는 것만(2.4절) | `impl Add`, `impl Sum` |
| `From`, `TryFrom`, `FromStr`, `AsRef`, `Borrow` | 손으로(`ch5-type-conversions`, Part 1 ch6) | - |
| 고유 메서드(`len`, `starts_with` 등) | 하나씩 위임하거나 읽기용 접근자 하나 | `as_str()` |

derive가 되는 트레이트는 비용이 거의 없고, 비용은 `Display`, 연산자, 변환 트레이트, 고유 메서드에서 생깁니다. 이 반복을 줄여 주는 derive 크레이트도 있습니다. 예를 들어 [`derive_more`](https://crates.io/crates/derive_more)("Adds #[derive(x)] macros for more traits")는 `Display`, `Add`, `Sum`, `From` 같은 트레이트를 derive로 구현해 줍니다. 쓰려는 derive마다 Cargo 기능을 켜야 하므로 문서를 확인하고 씁니다. 이 저장소의 예제는 외부 크레이트 없이 손으로 씁니다.

**뉴타입이 과한 경우.** 뉴타입은 다음 셋 가운데 하나에 해당할 때 비용을 치를 만합니다. **섞일 상대가 있다**(같은 표현, 다른 뜻), **지킬 불변식이 있다**(범위, 형식), **남의 트레이트를 붙여야 한다**(고아 규칙). 셋 다 아니라면 만들지 않습니다.

- 한 함수 안에서 만들어지고 쓰이고 끝나는 중간값은 섞일 상대가 없습니다.
- 안쪽 타입의 기능 대부분이 필요한 값은 위임 비용이 큽니다. 일반 문자열 처리나 수치 계산 내부처럼 모든 값이 같은 뜻이면 감쌀 이유가 없습니다.
- 경계에서 들어오자마자 다른 형식으로 바뀌어 나가는 값은 감싸도 지켜 줄 구간이 없습니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 컴파일러 출력을 싣고 있습니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **E0117의 `note:`가 늘었습니다.** 책의 출력에는 ``define and implement a trait or new type instead`` 한 줄만 있지만, 1.100.0은 그 앞에 ``impl doesn't have any local type before any uncovered type parameters``와 Reference의 고아 규칙 절 링크를 함께 보여 줍니다(4.3절). 해결 방법을 알려 주는 마지막 줄은 그대로입니다.
- **책에 없는 지침.** 2.3절(생성자에서 값의 범위 검증)과 2.4절(열어 줄 연산 고르기, `Deref`를 피하는 이유)은 책의 Item 6이 다루는 내용이 아닙니다(2026-11-19에 온라인판에서 확인). 2.3절은 [ch1](../ch1-express-data-structures/README.md)이 넘긴 주제를 Part 0 ch9의 검증 타입과 이어서, 2.4절은 Rust API 가이드라인(C-DEREF)을 근거로 이 챕터가 더했습니다.
- **달라지지 않은 점.** 단위가 다른 뉴타입을 넘겼을 때 `From` 구현이 있으면 E0308에 `Into::into`를 부르라는 도움말이 붙는 것은 책과 같습니다. 1.100.0에서 `From` 구현을 지우면 이 도움말도 사라지는 것을 확인했습니다(2.1절).

### 2.8 기억할 것

- 별칭은 이름이고 뉴타입은 타입입니다. 단위나 뜻을 구별하려면 뉴타입을 만들고, 변환은 경계에서 이름 있는 메서드로 합니다.
- 같은 타입의 인자가 나란하면(수 여러 개, `bool` 플래그) 뉴타입으로 감싸 순서 실수를 컴파일 에러로 만듭니다.
- 범위 규칙은 비공개 필드와 검사하는 생성자로 한 번만 검사하고, 안쪽 코드는 그 타입을 받는 것으로 검사를 대신합니다.
- 뜻이 있는 연산만 하나씩 엽니다. `Deref`로 안쪽 타입을 통째로 열지 않습니다.
- 남의 트레이트를 남의 타입에 붙여야 할 때는 감쌉니다. 바깥 API가 그 트레이트를 요구하지 않으면 내 트레이트로도 충분합니다.
- 위임은 안쪽 구현에 그대로 넘깁니다. 섞일 상대도, 지킬 불변식도, 붙일 트레이트도 없으면 뉴타입을 만들지 않습니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행합니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_unit_newtype.rs`](examples/01_unit_newtype.rs) | 단위는 뉴타입으로 | `type Degrees = f64` | `struct Degrees(f64)` | 2.1 |
| [`02_distinct_args.rs`](examples/02_distinct_args.rs) | 같은 타입 인자 구별 | `usize, usize, bool` | `Page, PerPage, NewestFirst` | 2.2 |
| [`03_validated_newtype.rs`](examples/03_validated_newtype.rs) | 생성자에서 범위 검증 | `percent: u8` | `Percent::new` | 2.3 |
| [`04_choose_operations.rs`](examples/04_choose_operations.rs) | 열어 줄 연산 고르기 | `impl Deref<Target = f64>` | `Add`, `to_radians`만 | 2.4 |
| [`05_orphan_rule_wrapper.rs`](examples/05_orphan_rule_wrapper.rs) | 고아 규칙 우회 | (E0117, 4.3절) | `struct Elapsed(Duration)` | 2.5 |
| [`06_delegation_cost.rs`](examples/06_delegation_cost.rs) | 위임 구현의 비용 | `write!(f, "{}", self.0)` | `fmt::Display::fmt(&self.0, f)` | 2.6 |

### 01_unit_newtype — 단위는 뉴타입으로

길이 12m 경사로에 도면의 경사각 3도와 5도를 넣습니다. 별칭 판은 도를 라디안 자리에 그대로 넘겨 틀린 높이를 내고, 뉴타입 판은 `to_radians()`를 거쳐야만 컴파일됩니다. 마지막 줄은 별칭으로 선언한 두 변수 사이에 아무 변환 없이 대입되는 것을 보여 줍니다.

```sh
cargo run -p er-ch6-newtype-pattern --example 01_unit_newtype
```

```text
경사로 길이 12m
[비권장] 타입 별칭
  3도 -> 높이 1.69m
  5도 -> 높이 -11.51m
[권장] 뉴타입
  Degrees(3.0) -> 높이 0.63m
  Degrees(5.0) -> 높이 1.05m
별칭: a = 5.0, b = 5.0
```

1.69m는 `12 × sin(3라디안)`, 0.63m는 `12 × sin(3도)`입니다.

### 02_distinct_args — 같은 타입 인자 구별

글 12개(`"글01"`이 가장 오래됨)를 최신 글부터 한 쪽에 5개씩 나눠 2쪽을 보여 주려 합니다. 비권장 판은 두 `usize`를 바꾼 호출도 받아서 5쪽(쪽당 2개)을 돌려줍니다.

```sh
cargo run -p er-ch6-newtype-pattern --example 02_distinct_args
```

```text
의도: 최신 글부터, 한 쪽에 5개씩, 2쪽
[비권장] usize, usize, bool
  (2, 5, true) -> ["글07", "글06", "글05", "글04", "글03"]
  (5, 2, true) -> ["글04", "글03"]
[권장] Page, PerPage, NewestFirst
  (Page(2), PerPage(5), NewestFirst(true)) -> ["글07", "글06", "글05", "글04", "글03"]
  (Page(1), PerPage(3), NewestFirst(false)) -> ["글01", "글02", "글03"]
```

### 03_validated_newtype — 생성자에서 범위 검증

서버가 보낸 진행률 0, 40, 100, 150으로 막대를 그립니다. 비권장 판은 150에서 막대가 틀을 뚫고, 권장 판은 경계에서 150을 거부한 뒤 나머지 값만 `Percent`로 그립니다.

```sh
cargo run -p er-ch6-newtype-pattern --example 03_validated_newtype
```

```text
[비권장] u8을 그대로 받음
    0 -> [          ]   0%
   40 -> [####      ]  40%
  100 -> [##########] 100%
  150 -> [###############] 150%
[권장] 경계에서 Percent로 바꾸고, 안쪽은 Percent만 받음
    0 -> [          ]   0% 남음 100%
   40 -> [####      ]  40% 남음 60%
  100 -> [##########] 100% 남음 0%
  150 -> 거부: 0~100 범위가 아님
```

### 04_choose_operations — 열어 줄 연산 고르기

두 판 모두 `Add`로 방향을 135도까지 더하는 데까지는 같습니다. `Deref`를 구현한 판은 `heading.cos()`가 `f64::cos`로 연결되어 틀린 이동량을 내고, `to_radians()`도 맨 `f64`를 돌려줍니다. 권장 판은 `Radians`를 거쳐야만 삼각함수를 부를 수 있습니다.

```sh
cargo run -p er-ch6-newtype-pattern --example 04_choose_operations
```

```text
30도에서 출발, 45도와 60도를 차례로 돌고 10m 전진
[비권장] Deref<Target = f64>
  방향 DegreesDeref(135.0), 이동 (-9.96, 0.88)
  heading.to_radians() = 2.3562 (타입은 f64)
[권장] 같은 단위의 덧셈, 단위 변환, 라디안의 삼각함수만 엶
  방향 Degrees(135.0), 이동 (-7.07, 7.07)
  직각보다 큰 방향인가: true
```

### 05_orphan_rule_wrapper — 고아 규칙 우회

`impl Display for Duration`은 컴파일되지 않으므로(4.3절) 예제에는 컴파일되는 세 가지만 있습니다. `Duration`의 디버그 형식, 내 트레이트 `HumanTime`, 뉴타입 `Elapsed`입니다. 내 트레이트 쪽은 `print_report`에 넘기려고 `String` 목록을 따로 만들고, 뉴타입 쪽은 그대로 넘깁니다. 합계는 `Duration`의 `Sum`으로 구한 뒤 출력할 때만 감쌉니다.

```sh
cargo run -p er-ch6-newtype-pattern --example 05_orphan_rule_wrapper
```

```text
[그대로] Duration에는 디버그 형식뿐입니다
    컴파일: 83.4s
    테스트: 7.2s
    문서: 600ms
[대안] 내 트레이트 HumanTime
    d.human(): 1분 23.4초
    print_report에는 String으로 바꿔 넘김
      컴파일: 1분 23.4초
      테스트: 7.2초
      문서: 0.6초
[권장] 뉴타입 Elapsed
    print_report에 그대로 넘김
      컴파일: 1분 23.4초
      테스트: 7.2초
      문서: 0.6초
    합계: 1분 31.2초
```

### 06_delegation_cost — 위임 구현의 비용

같은 주문서를 `write!`로 위임한 뉴타입과 `f`를 그대로 넘긴 뉴타입으로 출력합니다. 출력 함수 `print_table`은 두 판이 같고 `{sku:<8}`, `{qty:>4}`로 칸을 맞추지만, 비권장 판에서는 너비가 사라집니다. 권장 판은 `Sum` 구현으로 합계를 내고, `as_str()`로 `&str`의 메서드를 쓰고, derive한 `Ord`로 최댓값을 찾습니다.

```sh
cargo run -p er-ch6-newtype-pattern --example 06_delegation_cost
```

```text
[비권장] write!로 위임
    |A-100|3|
    |BX-7|12|
    |C-2048|1|
    합계 16
[권장] 안쪽 구현에 그대로 위임
    |A-100   |   3|
    |BX-7    |  12|
    |C-2048  |   1|
    합계 16
    A로 시작하는 코드: ["A-100"]
    가장 많이 주문한 상품: Some(Sku("BX-7"))
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch6-newtype-pattern
rustc --edition 2024 compile_fail/e0308_mixed_angle_units.rs --out-dir /tmp/er-ch6
rustc --edition 2024 compile_fail/e0599_inner_method_on_newtype.rs --out-dir /tmp/er-ch6
rustc --edition 2024 compile_fail/e0117_display_for_duration.rs --out-dir /tmp/er-ch6
```

### 4.1 단위가 다른 뉴타입을 넘기거나 더함 — E0308

파일: [`compile_fail/e0308_mixed_angle_units.rs`](compile_fail/e0308_mixed_angle_units.rs)

별칭이었다면 조용히 틀린 값을 냈을 두 줄입니다. 도를 라디안 매개변수에 넘기고, 도에 라디안을 더합니다.

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_mixed_angle_units.rs:40:31
   |
40 |     let h = ramp_height(12.0, slope); // E0308: 변환을 빠뜨림
   |             -----------       ^^^^^ expected `Radians`, found `Degrees`
   |             |
   |             arguments to this function are incorrect
   |
note: function defined here
  --> compile_fail/e0308_mixed_angle_units.rs:34:4
   |
34 | fn ramp_height(length_m: f64, slope: Radians) -> f64 {
   |    ^^^^^^^^^^^                --------------

error[E0308]: mismatched types
  --> compile_fail/e0308_mixed_angle_units.rs:42:25
   |
42 |     let total = slope + tilt; // E0308: 도와 라디안을 더함
   |                 -----   ^^^^ expected `Degrees`, found `Radians`
   |                 |
   |                 expected because this is `Degrees`
```

- **읽는 법**: 두 에러 모두 `` expected `X`, found `Y` ``가 핵심이고, 단위가 곧 타입 이름이라 무엇이 틀렸는지 바로 읽힙니다. 두 번째 에러가 E0369가 아니라 E0308인 이유는 `Degrees`에 `Add`가 구현되어 있기 때문입니다. `Add`의 오른쪽 타입 기본값이 `Self`이므로 컴파일러는 오른쪽에 `Degrees`를 기대했고, `Radians`가 와서 타입이 틀렸다고 말합니다.
- **원인**: 뉴타입은 안쪽 타입이 같아도 서로 다른 타입입니다(2.1절).
- **해결**: 첫 줄은 `ramp_height(12.0, slope.to_radians())`로 경계에서 바꿔 넘깁니다. 둘째 줄은 무엇을 하려던 것인지 다시 봅니다. 도에 더할 값이라면 처음부터 `Degrees`로 만들고, 정말 라디안 값을 더해야 한다면 한쪽을 변환한 뒤 더합니다. `Add<Radians> for Degrees`를 구현해 에러를 없애는 것은 단위 변환을 덧셈 안에 숨기는 일이라 권하지 않습니다.

### 4.2 뉴타입에 안쪽 타입의 메서드를 부름 — E0599

파일: [`compile_fail/e0599_inner_method_on_newtype.rs`](compile_fail/e0599_inner_method_on_newtype.rs)

```text
error[E0599]: no method named `cos` found for struct `Degrees` in the current scope
  --> compile_fail/e0599_inner_method_on_newtype.rs:33:29
   |
14 | struct Degrees(f64);
   | -------------- method `cos` not found for this struct
...
33 |     let dx = 10.0 * heading.cos(); // E0599
   |                             ^^^ method not found in `Degrees`
   |
help: one of the expressions' fields has a method of the same name
   |
33 |     let dx = 10.0 * heading.0.cos(); // E0599
   |                             ++
```

- **읽는 법**: 뉴타입 `Degrees`에는 `cos`가 없다는 에러입니다. 이 에러는 "도를 그대로 삼각함수에 넣으려 했다"는 실수를 잡은 것이므로 반가운 에러입니다.
- **함정**: `help:`는 필드 `.0`에 같은 이름의 메서드가 있다며 `heading.0.cos()`를 제안합니다. 컴파일러는 메서드 이름만 보고 제안할 뿐 단위는 모릅니다. 이 제안을 따르면 컴파일은 되지만 135도를 135라디안으로 계산하는, 2.4절의 `Deref` 판과 같은 버그가 됩니다. 뉴타입의 `.0`을 꺼내는 자리는 단위 보호가 풀리는 자리이므로, 그런 제안은 한 번 더 따져 봅니다.
- **해결**: 단위를 바꾼 뒤 라디안의 메서드를 부릅니다. `heading.to_radians().cos()`(예제 04의 권장 판). `Degrees`에 `cos`를 직접 만들고 싶다면 그 안에서 변환하는 `fn cos(self) -> f64 { self.to_radians().cos() }`로 위임합니다.

### 4.3 남의 타입에 남의 트레이트를 구현 — E0117

파일: [`compile_fail/e0117_display_for_duration.rs`](compile_fail/e0117_display_for_duration.rs)

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
  --> compile_fail/e0117_display_for_duration.rs:18:1
   |
18 | impl fmt::Display for Duration {
   | ^^^^^^^^^^^^^^^^^^^^^^--------
   |                       |
   |                       `Duration` is not defined in the current crate
   |
   = note: impl doesn't have any local type before any uncovered type parameters
   = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
   = note: define and implement a trait or new type instead
```

- **읽는 법**: 밑줄 아래의 `` `Duration` is not defined in the current crate ``가 원인이고, 마지막 `note:`가 해결책입니다. 첫 번째 `note:`의 "local type"은 이 크레이트에서 정의한 타입을 뜻합니다. 구현에 이 크레이트의 타입이 하나도 없다는 말입니다.
- **원인**: Part 0 ch10 2.7절의 고아 규칙입니다. `Display`도 `Duration`도 표준 라이브러리의 것입니다. 타입이 표준 라이브러리의 기본 타입이면 첫 줄이 `...can be implemented for primitive types`로 바뀝니다(`impl Ord for f64`로 확인).
- **해결**: 마지막 `note:`의 두 길 가운데 고릅니다(2.5절). `Display`를 요구하는 함수에 넘겨야 하면 뉴타입 `struct Elapsed(Duration);`에 구현하고, 내 코드에서 직접 부르기만 한다면 내 트레이트 `HumanTime`을 정의해 `Duration`에 구현합니다(예제 05).

## 5. 연습 문제

### 연습 1. 쪽 번호에 범위 검증 넣기

[`02_distinct_args.rs`](examples/02_distinct_args.rs)의 `Page`와 `PerPage`를 예제 03처럼 필드를 감춘 검증 뉴타입으로 바꾸세요. `Page`는 1 이상, `PerPage`는 1~50입니다. `Page::new(0)`, `PerPage::new(0)`, `PerPage::new(51)`은 `None`이어야 하고, `list_page`는 같은 결과를 내야 합니다.

<details>
<summary>힌트</summary>

두 타입을 `mod paging { .. }` 안으로 옮기고 필드의 `pub`을 지웁니다. 생성자는 `(n >= 1).then_some(Page(n))`, `(1..=50).contains(&n).then_some(PerPage(n))`처럼 쓰고, 값을 읽는 `get(self) -> usize`를 둡니다. 필드가 비공개가 되면 모듈 밖의 `list_page`에서 매개변수 패턴 `Page(page): Page`가 E0603으로 막히므로, 매개변수를 `page: Page`로 바꾸고 본문 첫 줄에서 `let page = page.get();`으로 꺼냅니다. 이제 `(page - 1)`은 넘칠 수 없습니다. 표준 라이브러리의 `NonZero<usize>`를 필드 타입으로 써서 "0이 아님"을 표현하는 방법도 있습니다.

</details>

### 연습 2. 방향과 회전량 나누기

[`04_choose_operations.rs`](examples/04_choose_operations.rs)는 방향과 회전량을 모두 `Degrees`로 다뤘습니다. 그런데 "방향 + 방향"은 뜻이 없습니다. 방향 `Heading`과 회전량 `Turn`을 나눠 `Heading + Turn = Heading`과 `Turn + Turn = Turn`만 구현하세요. `Heading`은 언제나 0 이상 360 미만으로 유지합니다. `Heading(30.0) + Turn(300.0) + Turn(90.0)`은 `Heading(60.0)`, `Turn(300.0) + Turn(90.0)`은 `Turn(390.0)`이어야 하고, `Heading(1.0) + Heading(2.0)`은 컴파일되지 않아야 합니다.

<details>
<summary>힌트</summary>

`impl Add<Turn> for Heading { type Output = Heading; fn add(self, t: Turn) -> Heading { Heading((self.0 + t.0).rem_euclid(360.0)) } }`처럼 오른쪽 타입을 `Turn`으로 지정합니다. `rem_euclid`는 음수에도 0 이상의 나머지를 돌려줍니다. 회전량은 한 바퀴를 넘을 수 있어야 하므로 `Turn + Turn`에서는 나머지를 구하지 않습니다. 이렇게 하면 `Heading + Heading`은 ``expected `Turn`, found `Heading` ``(E0308)입니다. 표준 라이브러리의 `Instant + Duration = Instant`와 같은 설계입니다(2.4절).

</details>

### 연습 3. 단계 목록을 한 줄로 출력하기

[`05_orphan_rule_wrapper.rs`](examples/05_orphan_rule_wrapper.rs)의 단계 목록 전체를 `컴파일 1분 23.4초 → 테스트 7.2초 → 문서 0.6초`처럼 한 줄로 출력하는 `Display`를 구현하세요. `impl Display for [(&str, Duration)]`나 `impl Display for Vec<(&str, Elapsed)>`로 직접 구현할 수 있을지 먼저 예상해 보세요.

<details>
<summary>힌트</summary>

둘 다 E0117입니다. 슬라이스와 `Vec`은 이 크레이트의 타입이 아니고, 안에 내 타입 `Elapsed`가 들어 있어도 마찬가지입니다(Part 0 ch10 2.7절의 `Vec<SocialPost>`와 같은 경우). 목록을 감싸는 뉴타입 `struct Timeline<'a>(&'a [(&'a str, Duration)]);`를 만들고 `impl fmt::Display for Timeline<'_>`에서 `enumerate()`로 돌며 둘째 항목부터 앞에 `" → "`를 쓰고, 각 항목은 `write!(f, "{name} {}", Elapsed(*d))?`로 씁니다. 슬라이스를 빌려 감쌌으므로 목록을 복사하지 않습니다.

</details>

### 연습 4. 뉴타입을 만들 것인가

다음 각각에 뉴타입을 만들지 판단하고, 2.6절의 세 기준(섞일 상대, 지킬 불변식, 붙일 트레이트) 가운데 무엇에 해당하는지 말해 보세요.

1. 공개 함수 `fn move_post(post_id: u64, board_id: u64)`의 두 인자
2. 함수 안에서 줄 수를 세는 `let mut lines = 0usize;`
3. 회원 가입에서 받는 이메일 주소 문자열
4. 행렬 곱셈 함수 안에서만 쓰는 `f64` 원소들

<details>
<summary>힌트</summary>

1번은 만듭니다. 같은 `u64` 두 개가 나란히 있어 바꿔 넘기기 쉽고(섞일 상대), 공개 함수라 부르는 쪽이 많습니다. 2번은 만들지 않습니다. 함수 안에서 끝나는 중간값이라 섞일 상대가 없습니다. 3번은 만들 만합니다. "@가 있다" 같은 형식 규칙을 경계에서 한 번 검사하고(지킬 불변식), 다른 문자열(이름, 비밀번호)과 섞이지 않게 합니다. 4번은 만들지 않습니다. 모든 원소가 같은 뜻의 수이고 안쪽 타입의 산술 전부가 필요하므로, 위임 비용만 커집니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 6: Embrace the newtype pattern](https://effective-rust.com/newtype.html). 이 챕터는 이 Item이 다루는 지침의 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- Rust API 가이드라인
  - [Newtypes provide static distinctions (C-NEWTYPE)](https://rust-lang.github.io/api-guidelines/type-safety.html#c-newtype): 같은 표현의 다른 뜻을 뉴타입으로 구별하라는 지침
  - [Only smart pointers implement Deref (C-DEREF)](https://rust-lang.github.io/api-guidelines/predictability.html#c-deref): `Deref`를 스마트 포인터에만 구현하라는 지침(2.4절)
- 레퍼런스: [Orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules) — E0117의 `note:`가 가리키는 고아 규칙의 정확한 조건
- 표준 라이브러리 문서
  - [`std::cmp::Reverse`](https://doc.rust-lang.org/std/cmp/struct.Reverse.html): 순서를 뒤집는 뉴타입
  - [`std::path::Display`](https://doc.rust-lang.org/std/path/struct.Display.html): `Path`를 출력하기 위한 도우미 구조체
  - [`std::num::NonZero`](https://doc.rust-lang.org/std/num/struct.NonZero.html), [`std::time::Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html), [`std::time::Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html)
- 크레이트: [`derive_more`](https://crates.io/crates/derive_more) — 뉴타입의 위임 구현을 derive로 줄여 주는 크레이트(2.6절)
- 에러 설명: `rustc --explain E0308`, `rustc --explain E0599`, `rustc --explain E0117`
