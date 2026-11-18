# ch2. Option과 Result: 콤비네이터, 상호 변환, `?`

`Option<T>`와 `Result<T, E>`는 Part 0에서 이미 여러 번 썼습니다. [Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md)에서는 `match`, `if let`, `let...else`로 `Option`을 열었고, [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)에서는 `Result`와 `?`, `unwrap_or_else`로 에러를 다뤘습니다. 두 챕터 모두 "`map`, `and_then`, `ok_or` 같은 더 많은 메서드는 Part 1 `ch2-option-result`에서 다룬다"고 미뤄 두었고, ch9의 Rustlings 안내는 `map_err`도 이 챕터로 넘겼습니다. 이 챕터가 그 약속을 지킵니다.

API 문서의 `Option` 페이지에는 메서드가 수십 개 있습니다. 알파벳순으로 외울 수는 없으므로, 이 챕터는 메서드를 **"무엇을 하고 싶은가"** 로 묶어 익힙니다. 안의 값 바꾸기, 실패할 수 있는 단계 잇기, 대안 고르기, 조건 묻기, 원본을 남긴 채 들여다보기, 제자리에서 꺼내고 채우기, `Option`과 `Result` 오가기, 반복자와 함께 쓰기입니다. 메서드마다 `ch1-std-overview`에서 배운 방법으로 시그니처를 읽고, Part 0에서 `match`로 길게 썼던 코드가 어떻게 줄어드는지 비교합니다.

- 패키지: `p1-ch2-option-result`
- 기준 문서: [`std::option`](https://doc.rust-lang.org/std/option/index.html), [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html), [`std::result`](https://doc.rust-lang.org/std/result/index.html), [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
- 확인한 툴체인: rustc 1.98.1, cargo 1.98.1 (Edition 2024)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 콤비네이터의 시그니처(`self`를 받는 방식과 클로저 바운드)를 읽고, 안의 값을 바꾸는 `map`·`map_err`·`map_or` 계열과 실패할 수 있는 단계를 잇는 `and_then`을 구별해 쓸 수 있다. Part 0에서 `match`로 쓴 코드를 콤비네이터로 바꿔 쓸 수 있다.
2. 대안을 고르는 메서드(`unwrap_or` 계열, `or`, `or_else`)와 조건을 묻는 메서드(`is_some_and`, `is_none_or`, `is_ok_and`, `is_err_and`, `filter`)를 쓸 수 있고, 인자를 즉시 평가하는 메서드와 `_else`로 끝나는 게으른 메서드의 차이를 실행 결과로 설명할 수 있다.
3. 대부분의 콤비네이터가 `self`를 소비한다는 것을 시그니처에서 읽어 내고, `as_ref`·`as_deref`·`as_mut`·`copied`·`cloned`로 원본을 남긴 채 다루며, `take`·`replace`·`insert`·`get_or_insert_with`로 `&mut` 너머의 `Option`을 제자리에서 꺼내고 채울 수 있다.
4. `ok_or`·`ok_or_else`·`ok`·`err`·`transpose`·`flatten`으로 `Option`과 `Result`를 오갈 수 있고, 같은 일을 `?`와 콤비네이터로 쓸 때의 차이를 설명하며 `ok_or(..)?`, `map_err(..)?`, `transpose()?` 같은 조합을 쓸 수 있다.
5. `Option`을 반복자처럼 쓰고, 반복자의 결과를 `collect`·`sum`으로 `Result<Vec<T>, E>`나 `Option<Vec<T>>` 등에 모을 수 있으며, 이때 첫 실패에서 멈춘다는 것을 설명할 수 있다.

## 2. 핵심 개념

### 2.1 콤비네이터와 시그니처 읽기

**콤비네이터(combinator)** 는 `Option`이나 `Result`를 받아 새 `Option`/`Result`(또는 값 하나)를 돌려주는 메서드입니다. 결과가 다시 `Option`/`Result`이므로 점(`.`)으로 계속 이어 붙일 수 있습니다. `match`는 모든 경우를 빠짐없이 적게 해 주는 좋은 도구지만, "`Some`이면 안의 값을 바꾸고 `None`이면 `None`" 같은 모양을 매번 적다 보면 정작 하고 싶은 일(값을 바꾼다)이 갈래 사이에 묻힙니다. 콤비네이터는 그 반복되는 모양에 이름을 붙인 것입니다.

콤비네이터를 고를 때는 시그니처에서 두 가지를 봅니다.

**① 첫 매개변수가 `self`를 어떻게 받는가.** 이것이 원본이 어떻게 되는지를 정합니다.

| 첫 매개변수 | 예 | 부른 뒤의 원본 |
| --- | --- | --- |
| `&self` | `is_some`, `is_ok`, `as_ref`, `as_deref` | 그대로 남음 |
| `self` | `map`, `and_then`, `unwrap_or`, `ok_or`, `is_some_and`, `filter` 등 대부분 | 소비됨. 단, 안의 타입이 `Copy`면 `Option`/`Result`도 `Copy`라서 복사본이 넘어감 |
| `&mut self` | `take`, `replace`, `insert`, `get_or_insert_with`, `as_mut` | 그 자리에서 바뀜 |

**② 클로저 바운드가 무엇을 받아 무엇을 돌려주는가.** 같은 "클로저를 받는 메서드"라도 바운드가 다르면 하는 일이 다릅니다.

```rust
pub fn map<U, F>(self, f: F) -> Option<U>      where F: FnOnce(T) -> U
pub fn and_then<U, F>(self, f: F) -> Option<U> where F: FnOnce(T) -> Option<U>
pub fn unwrap_or_else<F>(self, f: F) -> T      where F: FnOnce() -> T
```

바운드가 거의 모두 `FnOnce`인 것은 클로저를 **많아야 한 번** 부르기 때문입니다([Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md)의 2.7절). `map`은 `Some`일 때 한 번 부르고, `None`이면 아예 부르지 않습니다. 이처럼 결과가 이미 정해지면 나머지 계산을 건너뛰는 것을 **단락 평가(short-circuit)** 라고 부르며, 이 챕터의 모든 콤비네이터와 `?`, 그리고 2.11절의 `collect`가 이렇게 동작합니다.

### 2.2 안의 값 바꾸기: `map`, `map_err`, `map_or`, `map_or_else`

**`map`** 은 `Some`/`Ok` 안의 값만 클로저로 바꾸고, `None`/`Err`는 그대로 통과시킵니다. Part 0 ch6의 `plus_one`을 나란히 놓으면 이렇습니다.

```rust
fn plus_one_match(x: Option<i32>) -> Option<i32> {   // Part 0 ch6
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}

fn plus_one(x: Option<i32>) -> Option<i32> {         // 이 챕터
    x.map(|i| i + 1)
}
```

Part 0 ch6의 예제는 clippy가 이 `match`를 `map`으로 바꾸라고 제안하는 것(`clippy::manual_map`)을 `#[expect]`로 막아 두었습니다. 이 챕터의 예제 01도 비교를 위해 같은 방법을 씁니다. 시그니처의 `F: FnOnce(T) -> U`에서 `U`는 `T`와 달라도 됩니다. `text.find('=').map(|pos| &text[..pos])`는 `Option<usize>`를 `Option<&str>`로 바꿉니다.

`Result`의 `map`도 `Ok` 쪽만 바꿉니다(`fn map<U, F>(self, op: F) -> Result<U, E>`). 반대로 **`Err` 쪽만 바꾸는 것이 `map_err`** 입니다.

```rust
pub fn map_err<F, O>(self, op: O) -> Result<T, F> where O: FnOnce(E) -> F
```

이번에는 에러 타입이 `E`에서 `F`로 바뀌고 `Ok`는 손대지 않습니다. 표준 라이브러리의 에러에 "무엇을 하다 실패했는지"를 덧붙이거나, 에러를 다른 에러 타입으로 감쌀 때 씁니다. Part 0 ch9의 Rustlings `errors6.rs`에 나온 `map_err(ParsePosNonzeroError::Creation)`이 바로 "`Err` 안의 값을 그 배리언트로 감싸기"입니다. 튜플 배리언트의 이름은 안의 값을 받아 열거형을 돌려주는 함수처럼 쓸 수 있으므로([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)) 클로저 없이 이름만 넘깁니다.

```rust
let port: Result<u16, String> = input
    .parse::<u16>()
    .map_err(|e| format!("포트 값 '{input}' 읽기 실패: {e}"));
```

**바꾼 뒤 바로 값으로 꺼내기**: `map(f).unwrap_or(기본값)`을 한 번에 하는 메서드가 있습니다. 결과는 `Option`이 아니라 값 하나입니다.

| 메서드 | 시그니처 (Option) | `None`/`Err`일 때 |
| --- | --- | --- |
| `map_or(기본값, f)` | `fn map_or<U, F>(self, default: U, f: F) -> U` | 기본값 |
| `map_or_else(g, f)` | `fn map_or_else<U, D, F>(self, default: D, f: F) -> U` | `g()`의 결과. `Result`판은 `g(에러)` |
| `map_or_default(f)` | `fn map_or_default<U, F>(self, f: F) -> U where U: Default` | `U`의 기본값 |

두 가지를 주의합니다. 첫째, **기본값(또는 기본값 클로저)이 첫 번째 인자**이고 `Some`/`Ok`일 때의 클로저가 두 번째입니다. 둘째, `Result`의 `map_or_else`는 기본값 클로저에 에러 값을 넘기므로(`D: FnOnce(E) -> U`) 성공과 실패를 각각 다른 문장으로 바꿀 수 있습니다. `map_or_default`는 Rust 1.98.0에서 안정화된 새 메서드로, `map_or(0, f)`처럼 타입의 기본값을 쓰는 경우를 줄여 줍니다(`Default` 트레이트는 `ch6-core-traits`). 예제는 [`01_map_family.rs`](examples/01_map_family.rs)입니다.

### 2.3 실패할 수 있는 단계 잇기: `and_then`

`map`과 `and_then`의 차이는 클로저가 돌려주는 타입 하나뿐입니다.

| 메서드 | 클로저 바운드 | 결과 |
| --- | --- | --- |
| `map` | `FnOnce(T) -> U` | 클로저의 결과를 `map`이 `Some`으로 싸서 `Option<U>` |
| `and_then` | `FnOnce(T) -> Option<U>` | 클로저가 돌려준 `Option<U>`를 그대로 씀 |

그래서 **다음 단계 자체가 실패할 수 있으면 `and_then`** 입니다. "텍스트 첫 줄의 첫 단어를 정수로 읽기"는 첫 줄이 없을 수도, 단어가 없을 수도, 숫자가 아닐 수도 있는 세 단계입니다. `match`로 쓰면 단계마다 한 겹씩 깊어지지만, `and_then`으로 쓰면 위에서 아래로 읽힙니다.

```rust
// 전: match 세 겹
match text.lines().next() {
    Some(line) => match line.split_whitespace().next() {
        Some(word) => match word.parse::<i32>() {
            Ok(n) => Some(n),
            Err(_) => None,
        },
        None => None,
    },
    None => None,
}

// 후: and_then 체인
text.lines()
    .next()
    .and_then(|line| line.split_whitespace().next())
    .and_then(|word| word.parse::<i32>().ok())
```

여기서 `map`을 잘못 고르면 클로저가 돌려준 `Option`을 `map`이 한 번 더 싸서 `Option<Option<&str>>`가 됩니다. `flatten()`이 그 두 겹을 한 겹으로 벗기므로 **`and_then`은 `map` 다음에 `flatten`을 한 것과 같습니다.** API 문서는 다른 언어에서 이 연산을 flatmap이라고 부른다고 소개합니다. 함수의 반환 타입이 정해져 있으면 컴파일러가 이 실수를 E0308로 잡아 줍니다(4.3절).

예제 02의 3번 구역은 단계마다 출력문을 넣어 단락 평가를 보여 줍니다. `"x"`는 1단계에서 `None`이 되므로 2·3단계의 클로저가 한 번도 실행되지 않습니다.

**`Result`의 `and_then`은 에러 타입이 같아야 합니다.** 시그니처가 `F: FnOnce(T) -> Result<U, E>`라서, 다음 단계가 돌려주는 에러도 앞 단계와 같은 `E`여야 합니다. `parse`의 에러(`ParseIntError`)와 범위 검사의 에러가 다르면 그대로 이을 수 없고(4.4절), **`map_err`로 먼저 에러 타입을 맞춘 뒤** `and_then`으로 잇습니다.

예제 02는 두 종류의 실패를 담는 열거형 `AgeError`(배리언트 `NotNumber(ParseIntError)`, `OutOfRange(i32)`)를 만들어 이렇게 잇습니다.

```rust
fn parse_age(input: &str) -> Result<i32, AgeError> {
    input
        .trim()
        .parse::<i32>()
        .map_err(AgeError::NotNumber) // Result<i32, ParseIntError> → Result<i32, AgeError>
        .and_then(check_range)        // fn check_range(n: i32) -> Result<i32, AgeError>
}
```

에러 열거형을 제대로 설계하는 방법(`Display`, `Error` 트레이트 구현 등)은 `ch8-errors-panic`에서 다룹니다. 이 챕터의 열거형은 `derive(Debug)`만 붙인 단순한 모양입니다. 예제는 [`02_and_then.rs`](examples/02_and_then.rs)입니다.

### 2.4 대안 고르기: `unwrap_or` 계열과 `or`, `or_else`

값이 없거나 실패했을 때 무엇으로 대신할지 정하는 메서드는 두 무리입니다.

| 메서드 | 결과 | `None`/`Err`일 때 |
| --- | --- | --- |
| `unwrap_or(값)` | `T` | 주어진 값 |
| `unwrap_or_else(클로저)` | `T` | 클로저의 결과. `Result`판은 클로저가 에러를 받음(`FnOnce(E) -> T`) |
| `unwrap_or_default()` | `T` | `T`의 기본값(정수 `0`, 빈 `String`, 빈 `Vec`) |
| `or(다른 Option/Result)` | `Option<T>`/`Result<T, F>` | 주어진 `Option`/`Result` |
| `or_else(클로저)` | `Option<T>`/`Result<T, F>` | 클로저가 돌려준 `Option`/`Result`. `Result`판은 클로저가 에러를 받음 |

`unwrap_or` 무리는 값을 꺼내 끝내고, `or` 무리는 결과가 여전히 `Option`/`Result`라서 "첫 번째로 찾은 것"을 고르는 체인을 만들 수 있습니다.

```rust
let name = lookup(4)
    .or_else(|| lookup_backup(4))   // 첫 저장소에 없으면 두 번째 저장소
    .unwrap_or("이름 없음");        // 둘 다 없으면 기본값
```

`Result`의 `or_else`는 실패했을 때 **다른 방법으로 다시 시도**하는 데 씁니다. 예제 03의 `parse_number`는 10진수로 읽어 보고, 실패하면 `"0x"`를 떼고 16진수로 다시 읽습니다. 시그니처 `fn or_else<F, O>(self, op: O) -> Result<T, F>`에서 보듯 다시 시도한 결과의 에러 타입 `F`는 처음의 `E`와 달라도 됩니다. `and_then`이 성공 쪽에서 다음 단계를 잇는다면 `or_else`는 실패 쪽에서 대안을 잇는, 서로 거울 같은 메서드입니다.

### 2.5 즉시 평가와 게으른 평가

`unwrap_or`와 `unwrap_or_else`처럼 이름이 `_else`로 끝나는 짝이 있는 이유는 **대안을 언제 계산하는가**가 다르기 때문입니다. Rust는 함수를 부르기 전에 인자부터 모두 계산합니다. 그래서 값을 받는 쪽은 대안이 필요 없는 `Some`/`Ok`일 때도 인자를 이미 계산해 둡니다. 이것이 **즉시 평가(eager evaluation)** 입니다. 클로저를 받는 쪽은 대안이 정말 필요할 때만 클로저를 부르므로 **게으릅니다(lazy)**.

예제 03의 `default_name`은 불릴 때마다 한 줄을 출력합니다.

```rust
let eager = lookup(id).unwrap_or(default_name());      // 인자 자리에서 default_name()을 먼저 부름
let lazy = lookup(id).unwrap_or_else(default_name);    // 함수 이름만 넘김. None일 때만 부름
```

3절의 03 출력에서 회원 1을 보면, 이름이 있어서 대안이 필요 없는데도 `unwrap_or` 쪽에서 `(default_name 실행됨)`이 출력되었습니다. 대안을 만드는 데 비용이 들거나(문자열 할당, 파일 읽기) 부수 효과가 있다면(출력, 카운터 증가) 이것은 낭비이거나 버그입니다.

| 즉시 평가 (값을 받음) | 게으른 평가 (클로저를 받음) |
| --- | --- |
| `unwrap_or` | `unwrap_or_else` |
| `or` | `or_else` |
| `map_or` | `map_or_else` |
| `ok_or` | `ok_or_else` (2.9절) |
| `get_or_insert` | `get_or_insert_with` (2.8절) |
| `and` | `and_then` |

**어느 쪽을 고를까?** 대안이 이미 만들어진 값(리터럴, 정수 하나, `ConfigError::Missing("port")`처럼 리터럴만 담은 배리언트)이면 값을 받는 메서드로 충분합니다. 함수 호출, `format!`, `String::from`처럼 계산이 필요하면 `_else`를 씁니다. 반대로 `unwrap_or_else(|| 0)`처럼 이미 있는 값을 클로저로 감싸기만 하면 clippy가 값을 받는 메서드로 바꾸라고 알려 줍니다.

```text
warning: unnecessary closure used to substitute value for `Option::None`
  = note: `#[warn(clippy::unnecessary_lazy_evaluations)]` on by default
help: use `unwrap_or` instead
```

표의 마지막 줄 `and_then`은 2.3절의 그 메서드입니다. 짝인 `and(다른 Option)`은 앞이 `Some`이면 주어진 `Option`을 그대로 돌려줄 뿐 앞의 값을 쓰지 못하므로, 단계를 잇는 데는 `and_then`을 씁니다. 예제는 [`03_fallbacks.rs`](examples/03_fallbacks.rs)입니다.

### 2.6 조건 묻기: `is_some_and`, `is_none_or`, `is_ok_and`, `is_err_and`, `filter`

`is_some()`/`is_ok()`는 값이 있는지만 묻습니다. "값이 있고, 그 값이 조건을 만족하는가"는 `is_some_and`로 묻습니다.

```rust
age.map_or(false, |a| a >= 19)   // 전: clippy::unnecessary_map_or가 is_some_and를 제안
age.is_some_and(|a| a >= 19)     // 후
```

| 메서드 | `Some(v)`/`Ok(v)`일 때 | `None`/`Err(e)`일 때 | 안정화 |
| --- | --- | --- | --- |
| `is_some_and(f)` | `f(v)` | `false` | 1.70.0 |
| `is_none_or(f)` | `f(v)` | `true` | 1.82.0 |
| `is_ok_and(f)` | `f(v)` | `false` | 1.70.0 |
| `is_err_and(f)` | `false` | `f(e)` | 1.70.0 |

`is_none_or`는 "없거나, 있다면 조건을 만족하는가"라서 **선택 입력을 검사**할 때 꼭 맞습니다. 할인 코드는 입력하지 않아도 되지만 입력했다면 6글자여야 한다면 `code.is_none_or(|c| c.chars().count() == 6)`입니다. `is_err_and`는 에러 값을 받으므로 "어떤 에러였는가"를 물을 수 있습니다. 예제 04는 `ParseIntError`의 `kind()`가 돌려주는 `IntErrorKind`로 빈 입력인지 확인합니다.

**`filter`** 는 조건을 만족하는 `Some`만 남기고 나머지는 `None`으로 바꿉니다. 결과가 여전히 `Option`이므로 체인 중간에 끼울 수 있습니다.

```rust
pub fn filter<P>(self, predicate: P) -> Option<T> where P: FnOnce(&T) -> bool
```

클로저가 `T`가 아니라 `&T`를 받는 것은, 조건이 참이면 그 값을 그대로 돌려줘야 하므로 들여다보기만 하기 때문입니다. 반복자의 `filter`가 항목의 참조를 받는 것(Part 0 ch13)과 같은 이유이며, API 문서도 `Option`을 "원소가 0개나 1개인 반복자"로 상상해 보라고 설명합니다. Part 0 ch9의 `port_of`는 `filter`와 `and_then`으로 이렇게 쓸 수 있습니다.

```rust
fn port_of(line: &str) -> Option<u16> {
    line.split_once('=')
        .filter(|(key, _)| key.trim() == "port")
        .and_then(|(_, value)| value.trim().parse().ok())
}
```

`Result`에는 `filter`가 없습니다. 조건에 맞지 않을 때 어떤 에러를 넣을지 정할 수 없기 때문이며, 그럴 때는 2.3절처럼 `and_then`으로 직접 `Err`를 돌려줍니다.

**이 메서드들도 `self`를 소비합니다.** `Option<String>`에 `is_some_and`를 부르면 원본이 이동합니다. 원본을 계속 쓰려면 다음 절의 `as_deref()`로 참조를 담은 `Option`을 만들어 묻습니다. 예제는 [`04_queries.rs`](examples/04_queries.rs)입니다.

### 2.7 소유권을 유지한 채 들여다보기: `as_ref`, `as_deref`, `as_mut`, `copied`, `cloned`

2.1절에서 본 것처럼 대부분의 콤비네이터는 `self`를 가져갑니다. 원본을 남기려면 **참조를 담은 새 `Option`을 만들어 그것을 소비**하게 합니다.

```text
&Option<T>        --as_ref()-->    Option<&T>
&Option<String>   --as_deref()-->  Option<&str>
&mut Option<T>    --as_mut()-->    Option<&mut T>
Option<&T>        --copied()-->    Option<T>     (T: Copy)
Option<&T>        --cloned()-->    Option<T>     (T: Clone)
```

이것이 특히 필요한 곳은 **빌려 온 구조체의 `Option` 필드**입니다. `&User`만 받은 함수에서 `user.nickname.map(...)`을 부르면 빌린 값의 필드를 가져가려는 것이라 E0507입니다(4.1절).

```rust
fn nickname_len(user: &User) -> usize {
    user.nickname.as_ref().map_or(0, |n| n.chars().count())   // as_ref: Option<&String>
}

fn display_name(user: &User) -> &str {
    user.nickname.as_deref().unwrap_or(&user.name)            // as_deref: Option<&str>
}
```

`display_name`에서 `as_ref()`를 쓰면 `Option<&String>`이라 `unwrap_or`의 기본값도 `&String`이어야 합니다. `as_ref().unwrap_or("손님")`은 ``expected `&String`, found `&str` ``(E0308)입니다. `as_deref()`는 `String`이 역참조로 `str`이 되는 성질([Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)의 `Deref`)을 이용해 `Option<&str>`을 바로 만들어 주므로, `&str` 기본값이나 `Some("부산")` 같은 리터럴과 비교할 때 편리합니다.

**`copied`와 `cloned`** 는 반대 방향입니다. `first()`, `get()`, `iter().max()` 같은 메서드는 컬렉션 안을 가리키는 `Option<&T>`를 돌려줍니다. 그 참조를 들고 있는 동안에는 컬렉션을 고칠 수 없으므로, 값만 필요하면 복사해 둡니다.

```rust
let mut scores = vec![72, 95, 88];
let best: Option<i32> = scores.iter().max().copied();
scores.push(100); // best가 scores를 빌리고 있지 않으므로 괜찮습니다.
```

`copied()`를 빼면 `best`가 `Option<&i32>`가 되어, `best`를 쓰는 동안 `push`하면 ``E0502: cannot borrow `scores` as mutable because it is also borrowed as immutable``입니다. `String`처럼 `Copy`가 아닌 타입은 `cloned()`로 복제합니다.

**안의 타입이 `Copy`면 `Option`/`Result`도 `Copy`입니다.** `Option<i32>`에 `map`을 불러도 복사본이 넘어갈 뿐이라 원래 변수를 그대로 쓸 수 있습니다. 소비 문제는 `Option<String>`, `Option<Vec<T>>`처럼 `Copy`가 아닌 타입을 담았을 때만 생깁니다(4.2절). `Result`에도 `as_ref`(`&Result<T, E>` → `Result<&T, &E>`), `as_deref`, `as_mut`, `copied`, `cloned`가 같은 뜻으로 있습니다. 예제는 [`05_references.rs`](examples/05_references.rs)입니다.

### 2.8 제자리에서 꺼내고 채우기: `take`, `replace`, `take_if`, `insert`, `get_or_insert_with`

이 메서드들은 `&mut self`를 받습니다. `Option`을 소비하지 않고 변수나 필드에 둔 채 안의 값만 꺼내거나 바꿉니다.

| 메서드 | 시그니처 | 하는 일 |
| --- | --- | --- |
| `take` | `fn take(&mut self) -> Option<T>` | 값을 꺼내 돌려주고 그 자리에 `None`을 남김 |
| `replace` | `fn replace(&mut self, value: T) -> Option<T>` | 새 값을 넣고 예전 값을 돌려줌 |
| `take_if` | `fn take_if<P>(&mut self, predicate: P) -> Option<T>` | 조건이 참일 때만 `take` (1.80.0) |
| `insert` | `fn insert(&mut self, value: T) -> &mut T` | 무조건 넣고, 넣은 값의 가변 참조를 돌려줌 |
| `get_or_insert_with` | `fn get_or_insert_with<F>(&mut self, f: F) -> &mut T` | `None`일 때만 클로저로 만들어 넣고, 안의 값의 가변 참조를 돌려줌 |

**`take`가 필요한 이유**: `&mut self` 메서드에서 `Option` 필드의 값을 밖으로 가져가고 싶다고 해 봅시다. `self.item`을 그냥 돌려주면 컴파일러가 거부합니다.

```text
error[E0507]: cannot move out of `self.item` which is behind a mutable reference
3 |     fn pick_up(&mut self) -> Option<String> { self.item }
  |                                               ^^^^^^^^^ move occurs because `self.item` has type `Option<String>`, which does not implement the `Copy` trait
help: consider cloning the value if the performance cost is acceptable
```

`&mut self`는 빌린 것이라, 필드를 옮겨 가면 원래 주인의 구조체에 비어 있는 구멍이 생기기 때문입니다. 컴파일러는 `clone()`을 제안하지만, 값을 정말로 넘겨주려는 것이라면 `self.item.take()`가 답입니다. 꺼낸 자리에 `None`을 채워 구멍을 막으므로 복제 없이 옮길 수 있습니다.

**`get_or_insert_with`** 는 "처음 필요할 때 한 번만 계산해 두기"에 씁니다. 예제 06의 `Report::summary`는 첫 요청에서만 요약을 계산하고, 두 번째부터는 저장해 둔 값을 돌려줍니다. 기본값으로 채우는 `get_or_insert_default`(1.83.0), 값을 직접 받는 `get_or_insert`도 있습니다.

`Option`이 아닌 아무 타입에나 쓸 수 있는 `std::mem::take`와 `std::mem::replace`도 있으며 `ch7-smart-pointers-memory`에서 다룹니다. 예제는 [`06_in_place.rs`](examples/06_in_place.rs)입니다.

### 2.9 `Option`과 `Result` 오가기

| 바꾸는 방향 | 메서드 | 결과 |
| --- | --- | --- |
| `Option<T>` → `Result<T, E>` | `ok_or(e)`, `ok_or_else(\|\| e)` | `Some(v)` → `Ok(v)`, `None` → `Err(e)` |
| `Result<T, E>` → `Option<T>` | `ok()` | `Ok(v)` → `Some(v)`, `Err(_)` → `None` (에러를 버림) |
| `Result<T, E>` → `Option<E>` | `err()` | `Ok(_)` → `None`, `Err(e)` → `Some(e)` (성공 값을 버림) |
| `Option<Result<T, E>>` ↔ `Result<Option<T>, E>` | `transpose()` | `None` ↔ `Ok(None)`, `Some(Ok(v))` ↔ `Ok(Some(v))`, `Some(Err(e))` ↔ `Err(e)` |
| `Option<Option<T>>` → `Option<T>` | `flatten()` | 한 겹만 벗김 |
| `Result<Result<T, E>, E>` → `Result<T, E>` | `flatten()` | 한 겹만 벗김 (1.89.0에서 안정화) |

`ok_or`도 2.5절의 규칙을 따릅니다. 예제 07의 1번 구역에서 `ok_or(missing(key))`는 값을 찾았을 때도 에러 메시지 `String`을 만들었다가 버립니다. 에러 값을 `format!`으로 만든다면 `ok_or_else`를 씁니다.

**`transpose`가 필요한 곳**: "있으면 읽어야 하고, 읽다가 실패할 수도 있는 선택 값"입니다. 설정에 `timeout`이 없으면 기본값을 쓰고, 있으면 숫자여야 한다고 해 봅시다. `find_value(text, "timeout")`은 `Option<&str>`이고, 여기에 실패할 수 있는 `parse`를 `map`하면 `Option<Result<u32, E>>`가 됩니다. 함수는 "에러인가"를 먼저 알리고 싶으므로 `transpose()`로 `Result<Option<u32>, E>`로 뒤집습니다.

```rust
fn optional_timeout(text: &str) -> Result<Option<u32>, ConfigError> {
    find_value(text, "timeout")
        .map(|raw| {
            raw.parse::<u32>()
                .map_err(|e| ConfigError::BadNumber("timeout", e))
        })
        .transpose()
}
```

이렇게 해 두면 호출한 쪽에서 `optional_timeout(text)?.unwrap_or(30)` 한 줄로 "에러면 전파, 없으면 기본값"을 쓸 수 있습니다(2.10절). 이름이 같은 `transpose`가 `Result<Option<T>, E>` 쪽에도 있어서 반대 방향으로 되돌립니다.

**`flatten`** 은 2.3절에서 본 대로 `map`이 만든 두 겹을 벗깁니다. `Result`의 `flatten`은 Rust 1.89.0에서 안정화되었습니다. 둘 다 한 번에 한 겹만 벗기며, 애초에 두 겹을 만들지 않으려면 `and_then`을 씁니다. 예제는 [`07_convert_and_question.rs`](examples/07_convert_and_question.rs)입니다.

### 2.10 `?`와 콤비네이터

`식?`은 `Some(v)`/`Ok(v)`면 `v`를 꺼내고, `None`/`Err(e)`면 **그 자리에서 함수를 끝내며** `None`/`Err(e)`를 돌려줍니다(Part 0 ch9). 콤비네이터도 실패를 그대로 흘려보낸다는 점에서 같은 일을 하므로, 같은 함수를 세 가지로 쓸 수 있습니다.

```rust
// (1) match
match a.parse::<i32>() {
    Ok(x) => match b.parse::<i32>() {
        Ok(y) => Ok(x * y),
        Err(e) => Err(e),
    },
    Err(e) => Err(e),
}

// (2) 콤비네이터
a.parse::<i32>()
    .and_then(|x| b.parse::<i32>().map(|y| x * y))

// (3) ?
let x = a.parse::<i32>()?;
let y = b.parse::<i32>()?;
Ok(x * y)
```

세 함수의 결과는 모든 입력에서 같습니다(예제 07의 5번 구역). 어느 것을 쓸지는 이렇게 고릅니다.

| 상황 | 고를 것 | 이유 |
| --- | --- | --- |
| 함수가 `Option`/`Result`를 돌려주고, 중간 값을 여러 개 함께 써야 함 | `?` | 값마다 이름을 붙여 위에서 아래로 씁니다. (2)는 `x`를 쓰려고 둘째 단계가 클로저 안으로 들어가 중첩됩니다. |
| 값 하나를 단계별로 바꾸거나 걸러 냄 | 콤비네이터 | `.and_then(..).filter(..).map(..)`이 단계 그대로 읽힙니다. |
| 함수가 `()`나 다른 타입을 돌려줌 (`main`, 출력만 하는 함수) | 콤비네이터 | `?`는 `Option`/`Result`를 돌려주는 함수 안에서만 쓸 수 있습니다. |
| 실패해도 함수를 끝내지 않고 기본값으로 계속 | 콤비네이터 | `?`는 언제나 함수를 끝냅니다. `unwrap_or` 등으로 그 자리에서 정리합니다. |
| 클로저 안 (`map`의 클로저 등) | 콤비네이터, 또는 `collect` 뒤에 `?` | 클로저 안의 `?`는 클로저에서 빠져나갈 뿐 바깥 함수를 끝내지 않습니다(2.11절). |

두 방법은 섞어 쓰는 것이 보통입니다. **`?` 앞에서 콤비네이터로 타입을 맞추는** 조합이 특히 자주 나옵니다.

| 함수의 반환 타입 | 만난 값 | 쓰는 모양 |
| --- | --- | --- |
| `Result<T, E>` | `Option<U>` | `.ok_or(에러)?`, `.ok_or_else(\|\| 에러)?` |
| `Option<T>` | `Result<U, F>` | `.ok()?` |
| `Result<T, E>` | `Result<U, F>` (에러 타입이 다름) | `.map_err(F를 E로 바꾸는 함수)?` |
| `Result<T, E>` | `Option<Result<U, E>>` | `.transpose()?` |

```rust
fn required_port(text: &str) -> Result<u16, ConfigError> {
    let raw = find_value(text, "port").ok_or(ConfigError::Missing("port"))?;
    let port = raw
        .parse::<u16>()
        .map_err(|e| ConfigError::BadNumber("port", e))?;
    Ok(port)
}
```

`?`는 에러 타입 사이에 `From` 트레이트로 정의된 변환이 있으면 에러를 저절로 바꿔 줍니다. Part 0 ch9에서 `Box<dyn Error>`를 돌려주는 함수가 `ParseIntError`에 그냥 `?`를 쓸 수 있었던 이유입니다. 직접 만든 `ConfigError`에는 그런 변환을 정의하지 않았으므로 `map_err`로 바꿨습니다. `From`은 `ch6-core-traits`에서, 에러 타입에 변환을 붙여 `map_err` 없이 `?`를 쓰게 하는 설계는 `ch8-errors-panic`에서 다룹니다.

`?`를 어떤 타입에 쓸 수 있는지는 표준 라이브러리의 `Try` 트레이트가 정합니다. 이 트레이트는 아직 불안정(nightly 전용) 기능이라 안정판에서는 직접 구현할 수 없으므로, 우리가 `?`를 쓸 수 있는 것은 표준 라이브러리가 구현해 둔 `Option`, `Result` 등뿐입니다. Part 0 ch9의 에러 메시지에 나왔던 `FromResidual`도 이 기능의 일부입니다.

### 2.11 반복자와의 만남

**`Option`을 반복자로 쓰기**: `Some(v)`는 `v` 하나를, `None`은 아무것도 내놓지 않는 반복자처럼 동작합니다. 그래서 반복할 수 있는 것을 받는 `chain`이나 `Vec::extend`에 `Option`을 그대로 넘기면 "있을 때만 하나 덧붙이기"가 됩니다.

```rust
let extra: Option<i32> = bonus.parse().ok();
let numbers: Vec<i32> = (1..=3).chain(extra).collect(); // Some(4)면 [1, 2, 3, 4], None이면 [1, 2, 3]
```

다만 `for x in extra { ... }`처럼 `Option` 하나를 `for`로 돌면 `if let`으로 쓰라는 경고(`for_loops_over_fallibles`)가 납니다(4.6절).

**실패를 건너뛰기**: `filter_map(|s| s.parse().ok())`는 클로저가 `Some(v)`를 돌려준 원소만 `v`로 남기고, 반복자의 `flatten()`은 `Option`(또는 `Result`) 원소에서 `None`(`Err`)을 건너뛰고 안의 값만 꺼냅니다. 실패한 원소를 조용히 버리는 방법이며, 이 어댑터들은 `ch5-iterators`에서 자세히 다룹니다.

**`Result<Vec<T>, E>`로 모으기**: Part 0 ch13에서 `collect`는 "무엇으로 모을지"를 타입 표기로 정한다고 했습니다. 원소가 `Result<T, E>`인 반복자는 세 가지로 모을 수 있습니다.

| 모으는 타입 | 결과 |
| --- | --- |
| `Vec<Result<T, E>>` | 성공과 실패를 모두 그대로 보관 |
| `Result<Vec<T>, E>` | 전부 `Ok`면 `Ok(벡터)`, 하나라도 `Err`면 **첫 번째 `Err`** |
| `Vec<T>` | 불가능. E0277(4.5절) |

`Result<Vec<T>, E>`로 모으면 첫 `Err`를 만나는 순간 더 꺼내지 않습니다. 예제 08의 3번 구역은 클로저에 출력문을 넣어 이것을 확인합니다. `["10", "스물", "30", ""]`에서 `"스물"`까지만 읽고 멈춥니다. `Option`도 같아서, `Option<Vec<T>>`로 모으면 하나라도 `None`일 때 `None`입니다. `sum`(과 `product`)도 `Option`/`Result`를 결과 타입으로 받을 수 있습니다.

```rust
fn sum_csv(line: &str) -> Result<i32, ParseIntError> {
    line.split(',').map(|part| part.trim().parse::<i32>()).sum()
}
```

Part 0 ch9 연습 1을 `for`와 `?`로 풀었다면, 같은 일이 반복자 한 줄이 됩니다. 반환 타입이 `Result<i32, _>`이므로 `sum`이 "전부 `Ok`면 합계, 아니면 첫 `Err`"를 만듭니다.

**클로저 안에서는 `?` 대신 `collect` 뒤에 `?`**: `map`의 클로저 안에서 `?`를 쓰면 바깥 함수가 아니라 클로저에서 빠져나갑니다. 클로저가 `i32`를 돌려주도록 쓰면 이렇게 거부됩니다.

```text
error[E0277]: the `?` operator can only be used in a closure that returns `Result` or `Option` (or another type that implements `FromResidual`)
3 |     let nums: Vec<i32> = line.split(',').map(|p| p.trim().parse::<i32>()?).collect();
  |                                              ---                        ^ cannot use the `?` operator in a closure that returns `i32`
```

원소마다 `Result`를 돌려주게 두고, `Result<Vec<_>, _>`로 모은 결과에 `?`를 한 번 붙입니다.

```rust
let numbers = line
    .split(',')
    .map(|part| part.trim().parse::<i32>())
    .collect::<Result<Vec<i32>, _>>()?;
```

`collect`가 이렇게 여러 타입을 만들 수 있는 원리(`FromIterator` 트레이트)는 `ch5-iterators`에서 다룹니다. 예제는 [`08_iter_collect.rs`](examples/08_iter_collect.rs)입니다.

### 2.12 메서드 선택표

"이럴 때 이 메서드"를 한곳에 모았습니다. 빈칸(—)은 그 타입에 해당 메서드가 없다는 뜻입니다.

| 하고 싶은 일 | `Option<T>` | `Result<T, E>` | 절 |
| --- | --- | --- | --- |
| 안의 값을 바꾼다 | `map` | `map` | 2.2 |
| 에러 값을 바꾸거나 감싼다 | — | `map_err` | 2.2 |
| 바꾼 뒤 바로 값으로 꺼낸다 | `map_or`, `map_or_else`, `map_or_default` | 같음 | 2.2 |
| 실패할 수 있는 다음 단계를 잇는다 | `and_then` | `and_then` | 2.3 |
| 없으면/실패하면 기본값으로 꺼낸다 | `unwrap_or`, `unwrap_or_else`, `unwrap_or_default` | 같음 | 2.4 |
| 없으면/실패하면 다른 것으로 대신한다 | `or`, `or_else` | `or`, `or_else` | 2.4 |
| 값이 있고 조건을 만족하는가 | `is_some_and` | `is_ok_and`, `is_err_and` | 2.6 |
| 없거나, 있다면 조건을 만족하는가 | `is_none_or` | — | 2.6 |
| 조건에 맞지 않으면 `None`으로 | `filter` | — | 2.6 |
| 원본을 남기고 들여다본다 | `as_ref`, `as_deref` | `as_ref`, `as_deref` | 2.7 |
| 안의 값을 제자리에서 고친다 | `as_mut` | `as_mut` | 2.7 |
| 참조를 담은 것을 값을 담은 것으로 | `copied`, `cloned` | `copied`, `cloned` | 2.7 |
| 꺼내고 `None`을 남긴다 | `take`, `take_if` | — | 2.8 |
| 바꿔 넣고 예전 값을 받는다 | `replace` | — | 2.8 |
| 비어 있으면 채우고 `&mut T`를 받는다 | `get_or_insert_with`, `get_or_insert`, `get_or_insert_default` | — | 2.8 |
| 무조건 넣고 `&mut T`를 받는다 | `insert` | — | 2.8 |
| `Option` → `Result` | `ok_or`, `ok_or_else` | — | 2.9 |
| `Result` → `Option` | — | `ok`, `err` | 2.9 |
| `Option<Result>` ↔ `Result<Option>` | `transpose` | `transpose` | 2.9 |
| 두 겹을 한 겹으로 | `flatten` | `flatten` | 2.9 |
| 실패를 함수 밖으로 전파한다 | `?` | `?` | 2.10 |
| 반복자의 결과를 모은다 | `collect::<Option<Vec<_>>>()`, `sum` | `collect::<Result<Vec<_>, _>>()`, `sum` | 2.11 |

이 밖에도 두 `Option`을 튜플로 묶는 `zip`, 둘 중 정확히 하나만 `Some`일 때 그것을 고르는 `xor`, 값을 바꾸지 않고 들여다보기만 하는 `inspect`(`Result`에는 `inspect_err`도) 등이 있습니다. 새 메서드를 만나면 API 문서의 "Method overview" 절에서 어느 무리에 속하는지 먼저 보고, 시그니처의 `self` 모양과 클로저 바운드를 읽으면 됩니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_map_family.rs`](examples/01_map_family.rs) | `map`(`match`와 비교), `map_err`, `map_or`, `map_or_else`, `map_or_default` | 1 |
| [`examples/02_and_then.rs`](examples/02_and_then.rs) | `match` 세 겹과 `and_then` 체인, `map`이 만드는 두 겹, 단락 평가, `map_err` 후 `and_then` | 1 |
| [`examples/03_fallbacks.rs`](examples/03_fallbacks.rs) | `unwrap_or` 계열, `or`/`or_else`, 즉시 평가와 게으른 평가 | 2 |
| [`examples/04_queries.rs`](examples/04_queries.rs) | `is_some_and`, `is_none_or`, `is_ok_and`, `is_err_and`, `filter` | 2 |
| [`examples/05_references.rs`](examples/05_references.rs) | `as_ref`, `as_deref`, `as_mut`, `copied`, `cloned`, `Copy`인 `Option` | 3 |
| [`examples/06_in_place.rs`](examples/06_in_place.rs) | `take`, `replace`, `take_if`, `insert`, `get_or_insert_with` | 3 |
| [`examples/07_convert_and_question.rs`](examples/07_convert_and_question.rs) | `ok_or`/`ok_or_else`, `ok`/`err`, `transpose`, `flatten`, `match`·콤비네이터·`?` 비교, `?`로 잇기 | 2, 4 |
| [`examples/08_iter_collect.rs`](examples/08_iter_collect.rs) | `Option`을 반복자로, `filter_map`/`flatten`, `collect`와 `sum`으로 모으기, `collect` 뒤 `?` | 5 |
| [`compile_fail/`](compile_fail) | 콤비네이터를 쓰다 자주 만나는 컴파일 에러 | 1, 3, 5 |

예제 명령은 저장소 루트에서 실행한다고 보고 썼습니다. 어느 예제도 패닉하지 않으며 모두 종료 코드 0으로 끝납니다.

### 01_map_family — 안의 값 바꾸기

```console
$ cargo run -q -p p1-ch2-option-result --example 01_map_family
== 1. map: Some/Ok 안의 값만 바꾸기 ==
Some(5) → match: Some(6), map: Some(6)
None → match: None, map: None
find('=') 앞부분: Some("key")
find(':') 앞부분: None
"21" → Ok(42)
"스물하나" → Err(ParseIntError { kind: InvalidDigit })

== 2. map_err: Err 안의 값만 바꾸기 ==
Ok(8080)
Err("포트 값 '80a' 읽기 실패: invalid digit found in string")
Err("포트 값 '70000' 읽기 실패: number too large to fit in target type")

== 3. map_or, map_or_else: 바꾼 뒤 바로 꺼내기 ==
words.get(1)의 글자 수: 3
words.get(5)의 글자 수: 0
"42" → 성공(42)
"" → 실패(cannot parse integer from empty string)

== 4. map_or_default: None/Err면 타입의 기본값 ==
words.get(0)의 바이트 수: 6
words.get(9)의 바이트 수: 0
```

볼 점:

- 1번 구역에서 `match` 판과 `map` 판의 결과가 같습니다. `find(':')`가 `None`이면 클로저가 불리지 않고 `None`이 그대로 나옵니다.
- `"스물하나"`의 `Err`는 `map`을 거쳐도, 2번 구역의 `Ok(8080)`은 `map_err`를 거쳐도 그대로입니다. 각 메서드는 자기 쪽만 바꿉니다. `ParseIntError`의 메시지는 표준 라이브러리가 정한 영어 문장입니다.

### 02_and_then — 실패할 수 있는 단계 잇기

```console
$ cargo run -q -p p1-ch2-option-result --example 02_and_then
== 1. match 세 겹과 and_then 체인 ==
"42 개\n둘째 줄" → match: Some(42), and_then: Some(42)
"" → match: None, and_then: None
"   \n7" → match: None, and_then: None
"사십이" → match: None, and_then: None

== 2. map을 쓰면 Option이 두 겹이 됩니다 ==
  map만 쓴 중간 결과: Some(Some("안녕"))
flatten 후: Some("안녕")
  map만 쓴 중간 결과: None
flatten 후: None

== 3. 단락 평가: 실패한 뒤의 단계는 실행되지 않습니다 ==
입력 "3"
  2단계 실행: 3 × 1000
  3단계 실행: 3000 × 1000
  결과: Some(3000000)
입력 "5000"
  2단계 실행: 5000 × 1000
  3단계 실행: 5000000 × 1000
  결과: None
입력 "x"
  결과: None

== 4. Result 단계 잇기: map_err로 에러 타입을 맞춘 뒤 and_then ==
"27" → 27살
"200" → 범위 밖 (200)
"스물" → 숫자가 아님 (invalid digit found in string)
```

볼 점:

- 1번 구역의 세 `None`은 실패한 단계가 각각 다릅니다. 첫 줄이 없거나(`""`), 첫 줄에 단어가 없거나(`"   \n7"`), 숫자가 아닙니다(`"사십이"`).
- 2번 구역의 `Some(Some("안녕"))`이 `map`을 잘못 골랐을 때 생기는 두 겹입니다.
- 3번 구역에서 `"x"`는 결과 줄만 있고 단계 출력이 없습니다. 첫 단계에서 실패해 뒤의 클로저가 불리지 않았습니다.

### 03_fallbacks — 대안 고르기와 평가 시점

```console
$ cargo run -q -p p1-ch2-option-result --example 03_fallbacks
== 1. unwrap_or(값)과 unwrap_or_else(클로저) ==
회원 1
    (default_name 실행됨)
  unwrap_or      → 김하나
  unwrap_or_else → 김하나
회원 9
    (default_name 실행됨)
  unwrap_or      → 손님
    (default_name 실행됨)
  unwrap_or_else → 손님

== 2. unwrap_or_default: 타입의 기본값 ==
count = 0, nickname = ""

== 3. or(Option)과 or_else(클로저): 다른 Option으로 대신하기 ==
회원 2
    (lookup_backup 실행됨)
  or      → Some("이두리")
  or_else → Some("이두리")
회원 3
    (lookup_backup 실행됨)
  or      → Some("박세나")
    (lookup_backup 실행됨)
  or_else → Some("박세나")
    (lookup_backup 실행됨)
회원 4의 최종 이름: 이름 없음

== 4. Result에서 대안 고르기 ==
"255" → Ok(255)
"0xff" → Ok(255)
"zz" → Err(ParseIntError { kind: InvalidDigit })
"0x10" → 16
  "열여섯" 읽기 실패, 0을 씁니다: invalid digit found in string
"열여섯" → 0
```

볼 점:

- `(… 실행됨)` 줄이 언제 나오는지 보세요. `unwrap_or`와 `or`는 `Some`일 때도 대안을 계산하고, `_else` 판은 `None`일 때만 계산합니다. 회원 2의 `or_else` 쪽에는 `(lookup_backup 실행됨)`이 없고, 첫 저장소에 없는 회원 3은 두 방법 모두 두 번째 저장소에서 찾았습니다.
- `"0xff"`는 10진수 읽기에 실패한 뒤 `or_else`의 16진수 읽기로 성공했습니다.

### 04_queries — 조건 묻기

```console
$ cargo run -q -p p1-ch2-option-result --example 04_queries
== 1. is_some_and ==
Some(25) → map_or: true, is_some_and: true
Some(12) → map_or: false, is_some_and: false
None → map_or: false, is_some_and: false

== 2. is_none_or ==
None → true
Some("SPRING") → true
Some("A1") → false

== 3. is_ok_and, is_err_and ==
"42" → 짝수인가: true, 빈 입력인가: false
"7" → 짝수인가: false, 빈 입력인가: false
"" → 짝수인가: false, 빈 입력인가: true
"4x" → 짝수인가: false, 빈 입력인가: false

== 4. filter ==
"80" → Some(80)
"120" → None
"백" → None
"port=8080" → Some(8080)
"port = 443" → Some(443)
"host=localhost" → None
"port=abc" → None
"# 주석" → None

== 5. 조건 메서드도 self를 소비합니다 ==
'페'로 시작하는가: true, 원본: Some("페리스")
```

볼 점:

- 2번 구역에서 `None`은 `true`입니다. `is_none_or`는 "입력하지 않았거나, 입력했다면 올바른가"를 묻습니다.
- 4번 구역의 `port_of` 결과는 Part 0 ch9 `06_question_mark_option` 예제의 결과와 같습니다. 5번 구역은 `as_deref()`로 물었으므로 원본을 그대로 출력할 수 있습니다.

### 05_references — 원본을 남긴 채 들여다보기

```console
$ cargo run -q -p p1-ch2-option-result --example 05_references
== 1. as_ref, as_deref: 빌린 Option 들여다보기 ==
김하나 → 별명 글자 수: 3, 표시 이름: 하나둘
이두리 → 별명 글자 수: 0, 표시 이름: 이두리
바이트 수: Some(6), 부산인가: true, 원본: Some("부산")

== 2. as_mut: 안의 값을 제자리에서 고치기 ==
Some(["rust", "std"])

== 3. copied, cloned: Option<&T>를 Option<T>로 ==
추가 전 최고점: Some(95), 지금 점수들: [72, 95, 88, 100]
복제한 첫 이름: Some("가람"), 원본: ["가람", "나래"]

== 4. Result에도 같은 메서드가 있습니다 ==
글자 수: Ok(5), 원본: Ok("설정 내용")

== 5. 안의 타입이 Copy면 Option/Result도 Copy입니다 ==
원본: Some(3), 두 배: Some(6)
```

볼 점:

- 1번 구역의 `"부산"`은 한글 두 글자라 UTF-8로 6바이트입니다(Part 0 ch8). 물어본 뒤에도 원본이 그대로 출력됩니다.
- 3번 구역은 최고점을 복사해 둔 뒤에 `push`했습니다. 최고점은 추가하기 전 값인 95입니다.

### 06_in_place — 제자리에서 꺼내고 채우기

```console
$ cargo run -q -p p1-ch2-option-result --example 06_in_place
== 1. take: 꺼내고 None 남기기 ==
첫 번째: Some("우산"), 두 번째: None, 사물함: None

== 2. replace: 바꿔 넣고 예전 값 받기 ==
예전 물건: None, 사물함: Some("가방")
예전 물건: Some("가방"), 사물함: Some("책")

== 3. take_if: 조건이 맞을 때만 꺼내기 ==
우산이면 꺼냄 → None, 사물함: Some("책")
책이면 꺼냄   → Some("책"), 사물함: None

== 4. insert: 무조건 넣고 &mut T 받기 ==
retries = Some(3)

== 5. get_or_insert_with: 없을 때만 만들어 넣기 ==
  (요약을 계산합니다)
첫 요청: 3명, 합계 245점
두 번째 요청: 3명, 합계 245점
```

볼 점:

- `take`를 두 번 부르면 두 번째는 `None`입니다. 첫 번째 `take`가 `None`을 남겨 두었기 때문입니다.
- 5번 구역의 `(요약을 계산합니다)`는 한 번만 나옵니다.

### 07_convert_and_question — 오가기와 `?`

```console
$ cargo run -q -p p1-ch2-option-result --example 07_convert_and_question
== 1. Option → Result: ok_or, ok_or_else ==
키 port
    (에러 메시지를 만듦: port)
  ok_or      → Ok("8080")
  ok_or_else → Ok("8080")
키 host
    (에러 메시지를 만듦: host)
  ok_or      → Err("host 설정이 없습니다")
    (에러 메시지를 만듦: host)
  ok_or_else → Err("host 설정이 없습니다")

== 2. Result → Option: ok, err ==
"42" → ok(): Some(42), err(): None
"사십이" → ok(): None, err(): Some(ParseIntError { kind: InvalidDigit })

== 3. transpose: Option<Result>와 Result<Option> 뒤집기 ==
"timeout=5" → Ok(Some(5))
"port=1" → Ok(None)
"timeout=abc" → Err(BadNumber("timeout", ParseIntError { kind: InvalidDigit }))
되돌리기: Some(Ok(5))

== 4. flatten: 두 겹을 한 겹으로 ==
6 × 7: map → Ok(Ok(42)), flatten → Ok(42)
6 × 칠: map → Ok(Err(ParseIntError { kind: InvalidDigit })), flatten → Err(ParseIntError { kind: InvalidDigit })

== 5. 같은 함수를 match, 콤비네이터, ?로 ==
"6" × "7": match Ok(42) / 콤비네이터 Ok(42) / ? Ok(42)
"육" × "7": match Err(ParseIntError { kind: InvalidDigit }) / 콤비네이터 Err(ParseIntError { kind: InvalidDigit }) / ? Err(ParseIntError { kind: InvalidDigit })
"6" × "": match Err(ParseIntError { kind: Empty }) / 콤비네이터 Err(ParseIntError { kind: Empty }) / ? Err(ParseIntError { kind: Empty })

== 6. ?로 잇기: ok_or(..)?, map_err(..)?, transpose()? ==
"port=8080\ntimeout=5" → port=8080, timeout=5
"port=8080" → port=8080, timeout=30
"timeout=5" → port 설정이 없습니다
"port=80a" → port 값이 숫자가 아닙니다: invalid digit found in string
"port=8080\ntimeout=-1" → timeout 값이 숫자가 아닙니다: invalid digit found in string
```

볼 점:

- 3번 구역의 세 결과가 `transpose`의 세 경우(`Some(Ok)`, `None`, `Some(Err)`)입니다.
- 5번 구역에서 세 가지로 쓴 함수의 결과가 모든 입력에서 같습니다. 6번 구역의 마지막 줄은 `timeout`이 `u32`라서 음수 부호도 잘못된 글자로 본 것입니다.

### 08_iter_collect — 반복자와 함께 쓰기

```console
$ cargo run -q -p p1-ch2-option-result --example 08_iter_collect
== 1. Option을 반복자로 쓰기 ==
보너스 "4" → [1, 2, 3, 4]
보너스 "없음" → [1, 2, 3]
tags = ["rust", "과일"]

== 2. 실패를 건너뛰기: filter_map, flatten ==
filter_map: [10, 30]
flatten: [3, 5]

== 3. Result<Vec<T>, E>로 collect: 하나라도 실패하면 Err ==
Vec<Result<..>>로: [Ok(10), Err(ParseIntError { kind: InvalidDigit }), Ok(30), Err(ParseIntError { kind: Empty })]
  읽는 중: "10"
  읽는 중: "스물"
Result<Vec<..>>로: Err(ParseIntError { kind: InvalidDigit })
모두 성공하면: Ok([1, 2, 3])

== 4. Option<Vec<T>>로 collect: 하나라도 None이면 None ==
["사과", "우유"] → Some([1200, 2500])
["사과", "수박"] → None

== 5. Option, Result로 sum ==
["빵", "우유"] 합계 → Some(5500)
["빵", "케이크"] 합계 → None
sum_csv("10, 20, 30") → Ok(60)
sum_csv("1, 두, 3") → Err(ParseIntError { kind: InvalidDigit })

== 6. collect한 뒤 ?로 전파하기 ==
parse_all("4, 5, 6")
  (parse_all: 모두 읽었습니다)
  → Ok([4, 5, 6])
parse_all("4, 오, 6")
  → Err(ParseIntError { kind: InvalidDigit })
```

볼 점:

- 3번 구역에서 같은 반복자를 `Vec<Result<..>>`로 모으면 네 원소가 모두 남고, `Result<Vec<..>>`로 모으면 `"스물"`에서 멈춥니다.
- 6번 구역의 두 번째 호출에서는 `(parse_all: 모두 읽었습니다)`가 나오지 않습니다. `collect` 결과에 붙인 `?`가 함수를 끝냈습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch2-option-result
rustc --edition 2024 compile_fail/e0507_map_behind_reference.rs --out-dir /tmp/rust-p1-ch2
```

아래 인용은 핵심 줄만 옮긴 것입니다. 표준 라이브러리 소스 위치를 가리키는 `--> /rustc/…/option.rs` 같은 줄은 툴체인마다 다르므로 뺐습니다.

### 4.1 빌린 `Option`에 `map` — E0507

파일: [`compile_fail/e0507_map_behind_reference.rs`](compile_fail/e0507_map_behind_reference.rs)

```text
error[E0507]: cannot move out of `user.nickname` which is behind a shared reference
  --> compile_fail/e0507_map_behind_reference.rs:22:5
   |
22 |     user.nickname.map(|n| n.chars().count()).unwrap_or(0)
   |     ^^^^^^^^^^^^^ -------------------------- `user.nickname` moved due to this method call
   |     |
   |     move occurs because `user.nickname` has type `Option<String>`, which does not implement the `Copy` trait
   |
note: `Option::<T>::map` takes ownership of the receiver `self`, which moves `user.nickname`
help: consider calling `.as_ref()` to borrow the value's contents
   |
22 |     user.nickname.as_ref().map(|n| n.chars().count()).unwrap_or(0)
   |                  +++++++++
```

- **읽는 법**: `note:`가 원인을 짚습니다. `map`이 메서드를 부른 대상인 `self`의 소유권을 가져가므로 `user.nickname`이 이동하는데, `user`는 빌려 온 것(`behind a shared reference`)이라 옮길 수 없습니다.
- **원인**: 2.1절의 표에서 `map`의 첫 매개변수는 `self`입니다. 빌린 값의 필드를 옮기면 원래 주인의 구조체에 비어 있는 필드가 남습니다. `unwrap()`, `unwrap_or()`, `and_then()` 등 `self`를 받는 다른 메서드도 똑같이 거부됩니다.
- **해결**: 컴파일러의 첫 번째 `help:`대로 `as_ref()`를 끼웁니다. `&str`이 필요하면 `as_deref()`가 더 편합니다(2.7절, 예제 05). `help:`는 `as_mut()`과 `clone()`도 제안하지만, 읽기만 하는 함수에는 가변 대여가 필요 없고 복제는 불필요한 할당입니다.

### 4.2 `map`으로 소비한 `Option`을 다시 사용 — E0382

파일: [`compile_fail/e0382_map_consumes_option.rs`](compile_fail/e0382_map_consumes_option.rs)

```text
error[E0382]: borrow of moved value: `nickname`
  --> compile_fail/e0382_map_consumes_option.rs:21:20
   |
18 |     let nickname: Option<String> = Some(String::from("페리스"));
   |         -------- move occurs because `nickname` has type `Option<String>`, which does not implement the `Copy` trait
19 |     let len = nickname.map(|s| s.chars().count());
   |                        -------------------------- `nickname` moved due to this method call
20 |     println!("글자 수: {len:?}");
21 |     println!("원본: {nickname:?}");
   |                      ^^^^^^^^ value borrowed here after move
   |
note: `Option::<T>::map` takes ownership of the receiver `self`, which moves `nickname`
help: consider calling `.as_ref()` to borrow the value's contents
```

- **읽는 법**: 19행의 `map` 호출에서 `nickname`이 이동했고(`moved due to this method call`), 21행에서 이동한 값을 빌리려 했습니다. Part 0 ch6의 E0382는 `match`가 값을 옮긴 경우였는데, 여기서는 메서드의 `self`가 옮겼습니다.
- **원인**: `map`은 `Option`을 통째로 가져갑니다. 같은 코드가 `Option<i32>`에서는 컴파일되는데, 안의 타입이 `Copy`면 `Option`도 `Copy`라서 복사본이 넘어가기 때문입니다(예제 05의 5번 구역).
- **해결**: 원본을 계속 쓸 것이라면 `nickname.as_ref().map(...)`이나 `nickname.as_deref().map(...)`으로 참조를 담은 `Option`을 소비하게 합니다. 원본이 더 필요 없다면 출력을 `map`보다 앞으로 옮깁니다.

### 4.3 `and_then` 자리에 `map` — E0308

파일: [`compile_fail/e0308_map_instead_of_and_then.rs`](compile_fail/e0308_map_instead_of_and_then.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_map_instead_of_and_then.rs:17:5
   |
16 | fn first_number(text: &str) -> Option<i32> {
   |                                ----------- expected `Option<i32>` because of return type
17 |     text.lines().next().map(|line| line.trim().parse().ok())
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Option<i32>`, found `Option<Option<_>>`
   |
   = note: expected enum `Option<i32>`
              found enum `Option<Option<_>>`
help: use the `?` operator to extract the `Option<Option<_>>` value, propagating an `Option::None` value to the caller
   |
17 |     text.lines().next().map(|line| line.trim().parse().ok())?
   |                                                             +
```

- **읽는 법**: `found Option<Option<_>>`가 핵심입니다. 반환 타입은 한 겹인데 식은 두 겹입니다.
- **원인**: 클로저가 `parse().ok()`로 이미 `Option`을 돌려주는데, `map`이 그 결과를 `Some`으로 한 번 더 쌌습니다(2.3절).
- **해결**: `map`을 `and_then`으로 바꿉니다. 컴파일러가 제안하는 끝의 `?`도 컴파일되고 결과도 같지만(이 함수가 `Option`을 돌려주므로 바깥 `None`이면 빠져나가고, 아니면 안쪽 `Option`이 반환값이 됨), "실패할 수 있는 단계를 잇는다"는 뜻은 `and_then`이 더 직접 드러냅니다. 두 겹이 이미 만들어졌다면 `flatten()`으로 벗길 수도 있습니다.

### 4.4 에러 타입이 다른 단계를 `and_then`으로 이음 — E0308

파일: [`compile_fail/e0308_and_then_error_type.rs`](compile_fail/e0308_and_then_error_type.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_and_then_error_type.rs:21:53
   |
21 |         .and_then(|n| if n > 0 { Ok(n) } else { Err("양수가 아닙니다") })
   |                                                 --- ^^^^^^^^^^^^^^^^^ expected `ParseIntError`, found `&str`
   |                                                 |
   |                                                 arguments to this enum variant are incorrect

error[E0308]: mismatched types
  --> compile_fail/e0308_and_then_error_type.rs:19:5
   |
18 |   fn parse_positive(s: &str) -> Result<i32, &'static str> {
   |                                 ------------------------- expected `Result<i32, &'static str>` because of return type
19 | /     s.trim()
20 | |         .parse::<i32>()
21 | |         .and_then(|n| if n > 0 { Ok(n) } else { Err("양수가 아닙니다") })
   | |_________________________________________________________________________^ expected `Result<i32, &str>`, found `Result<i32, ParseIntError>`
```

- **읽는 법**: 에러가 두 개입니다. 첫째는 클로저 안의 `Err("...")`가 `ParseIntError`여야 한다는 것, 둘째는 그 결과 체인 전체가 `Result<i32, ParseIntError>`로 남아 반환 타입과 맞지 않는다는 것입니다. 원인은 하나이므로 첫 번째 에러부터 봅니다.
- **원인**: `Result::and_then`의 바운드는 `F: FnOnce(T) -> Result<U, E>`입니다. 클로저가 돌려주는 에러 타입이 앞 단계(`parse`)의 `E`와 같아야 합니다.
- **해결**: `and_then` 앞에서 `map_err`로 에러 타입을 먼저 맞춥니다. 이 파일이라면 `.map_err(|_| "숫자가 아닙니다")`를 `parse` 뒤에 넣으면 됩니다. 두 종류의 에러를 구분해 남기고 싶다면 예제 02의 `AgeError`처럼 열거형을 만들어 `map_err(AgeError::NotNumber)`로 감쌉니다.

### 4.5 `Result` 원소를 `Vec<T>`로 `collect` — E0277

파일: [`compile_fail/e0277_collect_results_into_vec.rs`](compile_fail/e0277_collect_results_into_vec.rs)

```text
error[E0277]: a value of type `Vec<i32>` cannot be built from an iterator over elements of type `Result<i32, ParseIntError>`
  --> compile_fail/e0277_collect_results_into_vec.rs:19:69
   |
19 |     let numbers: Vec<i32> = inputs.iter().map(|s| s.parse::<i32>()).collect();
   |                                                                     ^^^^^^^ value of type `Vec<i32>` cannot be built from `std::iter::Iterator<Item=Result<i32, ParseIntError>>`
   |
help: the trait `FromIterator<Result<i32, ParseIntError>>` is not implemented for `Vec<i32>`
      but trait `FromIterator<i32>` is implemented for it
note: the method call chain might not have had the expected associated types
   |
19 |     let numbers: Vec<i32> = inputs.iter().map(|s| s.parse::<i32>()).collect();
   |                                    ------ ^^^^^^^^^^^^^^^^^^^^^^^^^ `Iterator::Item` changed to `Result<i32, ParseIntError>` here
   |                                    |
   |                                    `Iterator::Item` is `&&str` here
```

- **읽는 법**: 첫 줄이 문제를 그대로 말합니다. `Vec<i32>`는 `i32` 원소로만 만들 수 있는데(`but trait FromIterator<i32> is implemented`), 원소가 `Result<i32, ParseIntError>`입니다. `note:`는 원소 타입이 `map`에서 `&&str`에서 `Result`로 바뀌었다고 체인을 따라 짚어 줍니다.
- **원인**: 실패할 수 있는 원소를 어떻게 할지 정하지 않았습니다. Part 0 ch13의 E0283은 결과 타입을 아예 적지 않은 경우였고, 이번에는 적은 타입이 원소와 맞지 않는 경우입니다.
- **해결**: 실패를 어떻게 다룰지를 결과 타입으로 정합니다(2.11절). 하나라도 실패하면 전체 실패로 보려면 `Result<Vec<i32>, _>`, 실패도 보관하려면 `Vec<Result<i32, _>>`, 실패를 건너뛰려면 `filter_map(|s| s.parse().ok())`입니다. 세 가지 모두 예제 08에 있습니다.

### 4.6 그 밖의 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| `&mut self` 메서드에서 `self.item`(`Option` 필드)을 그대로 돌려줌 | E0507 ``cannot move out of `self.item` which is behind a mutable reference`` (컴파일러는 `clone()`을 제안) | 값을 넘겨주려는 것이면 `self.item.take()` (2.8절) |
| 함수가 `Option<String>`을 받아 `.as_ref().unwrap_or("기본값")` | E0308 ``expected `&String`, found `&str` `` | `as_deref()`로 `Option<&str>`을 만듭니다(2.7절). |
| `map`의 클로저 안에서 `?` | E0277 ``the `?` operator can only be used in a closure that returns `Result` or `Option` `` | 원소마다 `Result`를 돌려주고 `collect::<Result<Vec<_>, _>>()?` (2.11절) |
| `found.map(\|pos\| println!("{pos}"));`처럼 출력만 하려고 `map` | 컴파일러는 아무 경고도 하지 않음. clippy는 ``called `map(f)` on an `Option` value where `f` is a closure that returns the unit type `()` `` (`clippy::option_map_unit_fn`) | 부수 효과만 낼 때는 `if let Some(pos) = found { ... }` |
| `Result`에 `map`만 하고 결과를 버림 | 경고 ``unused `Result` that must be used`` | 결과를 쓰거나 `?`로 전파합니다. `Option`의 `map`·`unwrap_or` 결과는 버려도 경고가 없으니 더 주의합니다. |
| `if x.is_some() { x.unwrap() }` | clippy ``called `unwrap` on `x` after checking its variant with `is_some` `` (`clippy::unnecessary_unwrap`) | `if let Some(v) = x`나 콤비네이터로 씁니다. |
| `unwrap_or_else(\|\| 0)`, `map_or(false, \|v\| ...)` | clippy `unnecessary_lazy_evaluations`, `unnecessary_map_or` | `unwrap_or(0)`, `is_some_and(...)` (2.5절, 2.6절) |
| `for x in some_option { ... }` | 경고 ``for loop over an `Option`. This is more readably written as an `if let` statement`` | `if let Some(x) = some_option` (2.11절) |

## 5. 연습 문제

### 연습 1. `match` 없이 다시 쓰기

다음 두 함수를 `match`, `if let`, `?` 없이 콤비네이터 체인 하나로 다시 쓰세요. 결과는 아래와 같아야 합니다.

```rust
fn age_of(record: &str) -> Option<u32> {   // "이름:나이" 형식
    match record.split_once(':') {
        Some((_, age)) => match age.trim().parse() {
            Ok(n) => Some(n),
            Err(_) => None,
        },
        None => None,
    }
}

fn describe(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("{}글자", n.chars().count()),
        None => String::from("없음"),
    }
}
```

```text
age_of("하나:27") = Some(27)
age_of("두리:스물") = None
age_of("세나") = None
describe(Some("페리스")) = "3글자"
describe(None) = "없음"
```

<details>
<summary>힌트</summary>

- `age_of`는 두 번째 단계(`parse`)가 실패할 수 있으므로 `and_then`이고, 그 안에서 `Result`를 `ok()`로 `Option`으로 바꿉니다. 다시 쓰기 전의 `match` 판을 clippy로 검사하면 안쪽 `match`에 `clippy::manual_ok_err`가 걸립니다. `split_once`가 돌려주는 튜플은 클로저 매개변수에서 `|(_, age)|`처럼 패턴으로 꺼냅니다.
- `describe`는 두 경우 모두 `String`을 만듭니다. `map_or(String::from("없음"), ...)`도 동작하지만 `Some`일 때도 `"없음"` 문자열을 할당합니다. 2.5절을 떠올려 `map_or_else(|| ..., |n| ...)`로 쓰고, 인자 순서(기본값 클로저가 먼저)에 주의합니다.

</details>

### 연습 2. 주문 처리

상품의 재고를 돌려주는 함수가 있습니다.

```rust
fn stock_of(item: &str) -> Option<u32> {
    match item {
        "사과" => Some(5),
        "우유" => Some(0),
        _ => None,
    }
}
```

`fn order(item: &str, qty: &str) -> Result<u32, String>`을 작성하세요. 수량 문자열을 읽어 주문하고, 주문 후 남는 재고를 돌려줍니다. 실패하면 이유가 드러나는 `String` 에러를 돌려줍니다. `match`와 `if`를 쓰지 말고 `?`와 콤비네이터로만 쓰세요.

```text
order("사과", "3") = Ok(2)
order("사과", "9") = Err("재고 부족: 남은 5개, 주문 9개")
order("우유", "1") = Err("재고 부족: 남은 0개, 주문 1개")
order("수박", "1") = Err("없는 상품: 수박")
order("사과", "세 개") = Err("수량이 숫자가 아닙니다('세 개'): invalid digit found in string")
```

<details>
<summary>힌트</summary>

- 세 단계입니다. ① `stock_of(item)`의 `Option`을 `ok_or_else(|| format!(...))?`로 `Result`로 바꿔 꺼냅니다. 메시지를 `format!`으로 만들므로 `ok_or`가 아니라 `ok_or_else`입니다. ② `qty.trim().parse::<u32>()`의 `ParseIntError`를 `map_err(|e| format!(...))?`로 `String`으로 바꿉니다. ③ `stock.checked_sub(qty)`는 빼서 음수가 되면 `None`을 돌려주므로(Part 0 ch3의 `checked_*`), 다시 `ok_or_else`로 `Result`로 바꾸면 그것이 함수의 마지막 값입니다.
- 2.10절의 `?` 조합표에서 ①은 첫째 줄, ②는 셋째 줄의 모양입니다.

</details>

### 연습 3. 줄마다 숫자인 텍스트의 합계

`fn total(text: &str) -> Result<i32, String>`을 작성하세요. 텍스트의 각 줄에는 정수가 하나씩 있습니다. 빈 줄은 건너뛰고, 숫자가 아닌 줄이 있으면 **처음 만난 그 줄의 번호(1부터)** 와 내용이 담긴 에러를 돌려줍니다. `for` 문 없이 반복자 체인 하나로 쓰세요.

```text
total("10\n20\n\n30") = Ok(60)
total("10\n스물\n30\n마흔") = Err("2번째 줄 '스물': invalid digit found in string")
total("") = Ok(0)
```

<details>
<summary>힌트</summary>

- `text.lines().enumerate()`로 `(0부터 시작하는 번호, 줄)`을 얻고, `filter(|(_, line)| !line.trim().is_empty())`로 빈 줄을 거릅니다(Part 0 ch13).
- `map`의 클로저에서 `line.trim().parse::<i32>()`에 `map_err`로 줄 번호를 붙인 `String` 에러를 만듭니다. 번호는 `i + 1`입니다.
- 마지막에 `sum()`을 부르면 반환 타입 `Result<i32, String>`을 보고 "첫 `Err`에서 멈추는 합계"가 됩니다(2.11절). 넷째 줄 `"마흔"`도 숫자가 아니지만 에러는 첫 번째 것만 나옵니다.

</details>

### 연습 4. 칭호가 있는 플레이어

칭호를 가질 수도, 갖지 않을 수도 있는 플레이어를 만드세요.

```rust
struct Player {
    name: String,
    title: Option<String>,
}
```

메서드 세 개를 구현합니다. `display(&self) -> String`은 칭호가 있으면 `"[칭호] 이름"`, 없으면 이름만 돌려줍니다. `promote(&mut self, new_title: &str) -> Option<String>`은 칭호를 바꾸고 예전 칭호를 돌려주며, `demote(&mut self) -> Option<String>`은 칭호를 빼앗아 돌려줍니다. `main`에서 다음 순서로 불러 같은 출력이 나오게 하세요.

```text
하나
promote → 이전 칭호: None
[견습] 하나
promote → 이전 칭호: Some("견습")
[기사] 하나
demote → 뺏긴 칭호: Some("기사")
하나
demote → 뺏긴 칭호: None
```

<details>
<summary>힌트</summary>

- `display`는 `&self`만 받으므로 `self.title.map(...)`은 E0507입니다(4.1절). `self.title.as_deref()`로 `Option<&str>`을 만든 뒤 `map_or_else(|| self.name.clone(), |t| format!("[{t}] {}", self.name))`처럼 씁니다.
- `promote`는 `replace`, `demote`는 `take`입니다(2.8절). 둘 다 `&mut self`를 받으므로 `main`의 변수에 `mut`이 필요합니다.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::option`](https://doc.rust-lang.org/std/option/index.html): [Method overview](https://doc.rust-lang.org/std/option/index.html#method-overview) 절이 메서드를 이 챕터처럼 목적별로 묶어 놓았습니다(Querying the variant, Adapters for working with references, Extracting the contained value, Transforming contained values, Boolean operators). [Boolean operators](https://doc.rust-lang.org/std/option/index.html#boolean-operators)의 표는 `and`/`or`/`xor`, `and_then`/`or_else`의 결과를 모든 경우에 대해 보여 주고, [Iterating over `Option`](https://doc.rust-lang.org/std/option/index.html#iterating-over-option)과 [Collecting into `Option`](https://doc.rust-lang.org/std/option/index.html#collecting-into-option)은 2.11절의 내용입니다.
  - [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html): 메서드마다 시그니처, 안정화된 버전, 예제가 있습니다. [`map_or_default`](https://doc.rust-lang.org/std/option/enum.Option.html#method.map_or_default)(1.98.0), [`is_none_or`](https://doc.rust-lang.org/std/option/enum.Option.html#method.is_none_or)(1.82.0), [`take_if`](https://doc.rust-lang.org/std/option/enum.Option.html#method.take_if)(1.80.0)처럼 최근에 추가된 메서드도 확인할 수 있습니다. 🔬 표시가 붙은 메서드(`zip_with` 등)는 nightly 전용이라 안정판에서는 쓸 수 없습니다.
  - [`std::result`](https://doc.rust-lang.org/std/result/index.html): [Results must be used](https://doc.rust-lang.org/std/result/index.html#results-must-be-used), [The question mark operator](https://doc.rust-lang.org/std/result/index.html#the-question-mark-operator-), [Method overview](https://doc.rust-lang.org/std/result/index.html#method-overview), [Collecting into `Result`](https://doc.rust-lang.org/std/result/index.html#collecting-into-result) 절이 이 챕터와 이어집니다.
  - [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html): [`map_err`](https://doc.rust-lang.org/std/result/enum.Result.html#method.map_err), [`transpose`](https://doc.rust-lang.org/std/result/enum.Result.html#method.transpose), [`flatten`](https://doc.rust-lang.org/std/result/enum.Result.html#method.flatten)(1.89.0)의 예제를 함께 보세요. 페이지 아래쪽 `FromIterator`와 `Sum` 구현 설명에 "첫 `Err`에서 더 꺼내지 않는다"는 동작이 예제와 함께 적혀 있습니다.
- Rust By Example: [Error handling](https://doc.rust-lang.org/rust-by-example/error.html) 장의 다음 절
  - [`Option` & `unwrap`](https://doc.rust-lang.org/rust-by-example/error/option_unwrap.html): `match`로 명시적으로 처리하는 것과 `unwrap`으로 암묵적으로 처리하는 것의 차이. Part 0의 복습입니다.
  - [Combinators: `map`](https://doc.rust-lang.org/rust-by-example/error/option_unwrap/map.html): 음식 손질 단계를 `match` 판과 `map` 체인 판으로 비교합니다(2.2절).
  - [Combinators: `and_then`](https://doc.rust-lang.org/rust-by-example/error/option_unwrap/and_then.html): `match` 판, `map` + `flatten` 판, `and_then` 판을 나란히 보여 줍니다(2.3절).
  - [Defaults: `or`, `or_else`, `get_or_insert`, `get_or_insert_with`](https://doc.rust-lang.org/rust-by-example/error/option_unwrap/defaults.html): `or`, `or_else`, `get_or_insert`, `get_or_insert_with`를 즉시 평가와 게으른 평가, 원본을 바꾸는지로 나눠 설명합니다(2.4, 2.5, 2.8절).
  - [`map` for `Result`](https://doc.rust-lang.org/rust-by-example/error/result/result_map.html): 두 문자열을 곱하는 함수를 `match` 판과 `and_then` + `map` 판으로 씁니다. 예제 07의 5번 구역과 같은 문제입니다.
  - [Iterating over `Result`s](https://doc.rust-lang.org/rust-by-example/error/iter_result.html): 실패를 건너뛰기(`filter_map`), 실패를 따로 모으기(`map_err`), 전체를 실패로 만들기(`collect`), 성공과 실패를 나누기(`partition`). 마지막 방법은 `ch5-iterators`를 배운 뒤에 읽으면 좋습니다.
- Rustlings (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - [`12_options`](https://github.com/rust-lang/rustlings/tree/main/exercises/12_options): `options1.rs`~`options3.rs`. Part 0 ch6에서 이미 풀었다면, `match`로 쓴 답을 이 챕터의 메서드로 다시 써 보세요. `options3.rs`는 `match` 대상에 `&`를 붙이는 것 외에 `as_ref()`로도 풀 수 있습니다.
  - [`13_error_handling`](https://github.com/rust-lang/rustlings/tree/main/exercises/13_error_handling): `errors6.rs`가 Part 0 ch9에서 미뤄 둔 `map_err` 문제입니다. 빈칸을 `s.parse().map_err(ParsePosNonzeroError::ParseInt)?`처럼 채우면 2.10절의 `map_err(..)?` 모양이 됩니다.
  - [`18_iterators`](https://github.com/rust-lang/rustlings/tree/main/exercises/18_iterators): `iterators3.rs`는 나눗셈 결과들을 `Result<Vec<_>, _>`와 `Vec<Result<_, _>>` 두 가지로 모으는 문제로, 2.11절의 표와 같습니다.
