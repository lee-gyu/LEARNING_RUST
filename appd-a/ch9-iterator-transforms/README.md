# ch9. 반복문 대신 반복자 변환: 어디까지 바꾸고 어디서 멈출까

[Part 1 ch5](../../part1-standard-lib/ch5-iterators/README.md)에서 반복자를 이루는 세 트레이트(`Iterator`, `IntoIterator`, `FromIterator`)와 어댑터·소비 메서드를 "무엇을 하고 싶은가"로 묶어 익혔고, [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md)에서 반복문을 체인으로 바꾼 짧은 예와 성능 이야기를 보았습니다. 이 챕터는 메서드 사용법을 다시 설명하지 않습니다. 이미 있는 `for` 반복문을 반복자 체인으로 **어떤 순서로 바꾸고**, 단계마다 **무엇이 나아지는지**, 그리고 **어디서 멈춰야 더 읽기 좋은지**를 판단하는 기준을 다룹니다.

[ch3](../ch3-option-result-transforms/README.md)가 `match`를 변환 메서드로 바꾸는 이야기였다면, 이 챕터는 같은 생각을 반복문에 적용합니다. 그리고 ch3이 미뤄 둔 질문, 즉 반복문 안에서 `?`로 전파하던 실패를 체인으로 옮길 때 무엇을 고를지에 답합니다. 판단의 중심에는 하나의 사실이 있습니다. **클로저 안의 `return`·`?`·`break`·`continue`는 바깥 함수나 바깥 반복문에 닿지 않습니다.**

- 원제: [Item 9: Consider using iterator transforms instead of explicit loops](https://effective-rust.com/iterators.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch9-iterator-transforms`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 인덱스 반복문을 `for x in &v`와 어댑터 체인으로 한 단계씩 바꾸면서, 단계마다 무엇이 사라지는지(인덱스 실수, 경계 검사, 본문에 묻힌 의도) 설명할 수 있다. `break`는 `take_while`로, `continue`는 `filter`/`filter_map`으로 옮길 수 있다.
2. 반복문의 누적 변수가 무엇을 만드는지 보고 `collect`·`count`·`sum`·`find`·`enumerate`·`fold`/`reduce` 중 알맞은 것을 고를 수 있고, 바꾼 뒤에도 동작이 같은지(같은 최댓값을 어떻게 고르는지 등) 확인할 수 있다.
3. 원소를 앞에서부터 한 번 읽기만 하는 함수의 매개변수를 `impl IntoIterator<Item = T>`로 열 수 있고, 슬라이스로 받아야 하는 경우를 구별할 수 있다.
4. 반복문 안에서 `?`로 전파하던 실패를 체인으로 옮길 때 `collect::<Result<..>>()`, `filter_map`, `partition`, `try_fold`/`try_for_each` 중 무엇을 쓸지 "실패한 원소를 어떻게 다뤄야 하는가"로 고를 수 있다.
5. 부수 효과가 본문일 때, 탈출 조건이나 함께 바꾸는 상태가 여럿일 때, 클로저 안에 큰 `match`가 들어갈 때는 명시적 반복문이 낫다는 것을 클로저의 경계를 근거로 설명하고 골라 쓸 수 있다.

## 2. 핵심 개념

2.1~2.6절은 지침 하나씩을 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 2.1~2.4절은 반복문을 체인으로 바꾸는 쪽, 2.5~2.6절은 반복문을 남기는 쪽입니다. 메서드의 동작이 궁금하면 Part 1 ch5의 2.9절(어댑터 선택표)과 2.14절(소비 메서드 선택표)을 함께 보세요.

### 2.1 반복문을 한 단계씩 체인으로 바꾸기

**문제 상황.** 악기 연습 일지에서 "처음 2주 동안 인정되는 연습 시간"을 구합니다. 일지는 날짜순이고, 기록을 빠뜨린 날은 `minutes`가 `None`이며, 하루에 60분까지만 인정합니다. 반복문 하나에 멈추는 조건(`break`), 건너뛰는 조건(`continue`), 계산, 누적이 함께 들어 있습니다.

**비권장 코드.** [`01_loop_to_chain.rs`](examples/01_loop_to_chain.rs)의 0단계입니다.

```rust
for i in 0..log.len() {
    if log[i].day > LAST_DAY {
        break;
    }
    let Some(minutes) = log[i].minutes else {
        continue;
    };
    total += minutes.min(DAILY_CAP);
}
```

**권장 코드로 가는 길.** 같은 함수를 네 번 고쳐 씁니다. 다섯 판은 모든 입력에서 같은 값을 돌려줍니다(예제 01의 1번 구역).

| 단계 | 바뀐 곳 | 사라진 것 | clippy (기본 설정) |
| --- | --- | --- | --- |
| 0. 인덱스 | `for i in 0..log.len()`, `log[i]` | — | `needless_range_loop` |
| 1. 원소 | `for session in log` | 인덱스, 인덱스마다 하는 경계 검사 | 없음 |
| 2. `break` → `take_while` | `log.iter().take_while(\|s\| s.day <= LAST_DAY)` | 본문의 `break` | 없음 |
| 3. `continue` → `filter_map` | `.filter_map(\|s\| s.minutes)` | `let ... else { continue; }` | 없음 |
| 4. 본문 → `map`, 누적 → `sum` | `.map(\|m\| m.min(DAILY_CAP)).sum()` | 가변 변수 `total` | 없음 |

```rust
log.iter()
    .take_while(|s| s.day <= LAST_DAY) // 처음 2주까지
    .filter_map(|s| s.minutes)         // 기록이 있는 날만
    .map(|m| m.min(DAILY_CAP))         // 하루 상한 적용
    .sum()                             // 모두 더함
```

단계마다 나아지는 점은 다음과 같습니다.

- **0 → 1: 인덱스가 사라집니다.** 0단계에서 `i`는 `log`를 가리키는 데만 쓰입니다. 그런데도 읽는 사람은 범위가 `0..log.len()`이 맞는지, `log[i]`가 매번 같은 원소인지 확인해야 합니다. 인덱싱은 실행 중에 경계 검사(bounds check)를 거치고, 범위를 벗어나면 패닉합니다(패닉을 피하는 설계는 [`ch18-dont-panic`](../ch18-dont-panic/README.md)). `for session in log`에는 인덱스가 없으므로 잘못 쓸 인덱스도, 벗어날 범위도 없습니다.
- **1 → 2 → 3: 흐름 제어가 반복문 머리로 올라옵니다.** "어디까지 보는가"(`take_while`)와 "무엇을 건너뛰는가"(`filter_map`)가 본문 중간의 `break`, `continue`가 아니라 이름 붙은 어댑터가 됩니다. `break`의 조건은 "멈출 조건"(`day > 14`)이고 `take_while`의 조건은 "계속할 조건"(`day <= 14`)이라 **방향이 뒤집힌다**는 점에 주의합니다. 걸러 내면서 `Option`을 벗기는 일은 `filter`와 `map` 둘이 아니라 `filter_map` 하나가 맡습니다([Part 1 ch5](../../part1-standard-lib/ch5-iterators/README.md) 2.5~2.6절).
- **3 → 4: 가변 상태가 사라집니다.** 본문의 계산이 `map`이 되면 남은 것은 "더한다"뿐이고, 이것은 소비 메서드 `sum`입니다. 체인을 위에서 아래로 읽으면 함수가 하는 일이 그대로 문장이 됩니다.

**인덱스판에서 실제로 일어나는 실수.** 예제 01의 2번 구역은 날짜를 1일부터 세다 보니 인덱스도 1부터 시작한 판(`for i in 1..log.len()`)입니다. 컴파일도 되고 패닉도 없지만 첫날의 45분을 빠뜨려 215분이 아니라 170분이 나옵니다. clippy의 제안을 보면 이 실수가 드러납니다.

```text
warning: the loop variable `i` is only used to index `log`
   --> examples/01_loop_to_chain.rs:97:14
help: consider using an iterator
    |
 97 -     for i in 1..log.len() {
 97 +     for <item> in log.iter().skip(1) {
```

반복자로 쓰면 "첫 원소를 건너뛴다"는 것을 `skip(1)`이라고 적어야 하므로 의도하지 않은 건너뛰기가 숨을 곳이 없습니다.

성능에 대해서는 이 챕터에서 재지 않습니다. 반복자 체인이 손으로 쓴 반복문과 같은 수준으로 최적화되는 이유는 Part 0 ch13 2.15절과 Part 1 ch5 2.4절에서 보았고, 측정과 최적화의 판단은 [`ch20-avoid-over-optimizing`](../ch20-avoid-over-optimizing/README.md)에서 다룹니다.

**지침을 따르지 않아도 되는 경우: 인덱스 자체가 계산의 재료일 때.** 두 배열을 다른 위치로 함께 읽거나(`v[i] * w[n - 1 - i]`), 이웃한 원소를 비교하는(`v[i] > v[i - 1]`) 반복문에는 clippy도 경고하지 않습니다. 이 경우도 `zip`과 `rev`, `windows(2)` 같은 대안이 있지만, 인덱스 산술이 문제의 본질이라면 인덱스가 드러난 반복문이 더 정직할 수 있습니다. 인덱스를 출력에도 쓰는 반복문(`println!("{i}: {}", v[i])`)은 clippy가 `.enumerate()`를 쓰라고 제안합니다.

### 2.2 누적 변수는 무엇을 만드는지 보고 소비 메서드로

**문제 상황.** 반복문 앞에서 `let mut`으로 만든 변수를 본문에서 고치는 코드는 흔합니다. 그 변수가 무엇을 만드는지(컬렉션, 개수, 합, 첫 원소, 가장 큰 것)를 알려면 반복문 전체를 읽어야 합니다. [`02_accumulator_to_consumer.rs`](examples/02_accumulator_to_consumer.rs)는 받은편지함 하나로 다섯 가지 누적을 비교합니다.

**비권장 코드와 권장 코드.**

| 반복문이 하는 일 | 바꿀 것 | clippy (기본 설정) |
| --- | --- | --- |
| 조건에 맞는 원소를 새 `Vec`에 `push` | `filter`/`map` + `collect` | 잡지 않음 |
| 카운터에 `+= 1` | `filter` + `count` | 잡지 않음 |
| 합계에 `+=` | `map` + `sum` | `fold(0, \|acc, x\| acc + x)`로 썼을 때만 `unnecessary_fold` |
| 찾으면 `return Some(x)`, 끝에서 `None` | `find` | `manual_find` |
| 순번 변수를 따로 셈 | `enumerate` | `explicit_counter_loop` |
| 더 크면 바꾸기 | `max_by_key` 등 (동작 확인 필요, 아래) | 잡지 않음 |
| 위 어디에도 맞지 않는 누적 | `fold`, `reduce` | — |

```rust
// 반복문판
let mut subjects = Vec::new();
for mail in inbox {
    if mail.unread {
        subjects.push(mail.subject);
    }
}

// 체인판
inbox.iter().filter(|m| m.unread).map(|m| m.subject).collect()
```

체인판은 첫 단어(`filter`)를 읽는 순간 "걸러 낸다", 마지막 단어(`collect`)를 읽는 순간 "모은다"를 압니다. 반복문판은 `subjects`가 처음에 비어 있고, 본문에서 `push`만 되고, 다른 곳에서 고쳐지지 않는다는 것까지 확인해야 같은 결론에 이릅니다.

clippy가 잡는 세 가지는 예제에서 `#[expect]`로 막아 두었습니다. 속성을 지우면 다음 경고가 납니다(핵심 줄만 옮겼습니다).

```text
warning: this `.fold` can be written more succinctly using another method
54 |     inbox.iter().map(|m| m.size_kb).fold(0, |acc, kb| acc + kb)
   |                                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try: `sum()`

warning: manual implementation of `Iterator::find`
   | help: replace with an iterator: `inbox.iter().find(|&mail| mail.size_kb > limit_kb).map(|v| v as _)`

warning: the variable `no` is used as a loop counter
79 |     for mail in inbox {
   |     ^^^^^^^^^^^^^^^^^ help: consider using: `for (no, mail) in (1..).zip(inbox.iter())`
```

`manual_find`의 제안 끝에 붙은 `.map(|v| v as _)`는 이 함수에서는 필요 없습니다. 예제의 체인판 `inbox.iter().find(|m| m.size_kb > limit_kb)`가 그대로 컴파일됩니다. 기계적인 제안은 출발점으로 쓰고 다듬습니다.

**바꾸기 전에 동작이 같은지 확인합니다.** 예제 02의 4번 구역에서 "가장 큰 메일"을 반복문판은 `견적서`, `max_by_key`판은 `설계 도면`이라고 답합니다. 두 메일의 크기가 3100KB로 같기 때문입니다. 반복문판은 더 클 때만(`>`) 바꾸므로 먼저 나온 것을 남기고, `max_by_key`는 같은 최댓값 중 **마지막** 원소를 돌려준다고 문서에 정해져 있습니다(Part 1 ch5 2.11절). 소비 메서드는 저마다 규칙이 정해져 있으므로, 원래 반복문의 규칙과 같은지 확인한 뒤 바꿉니다. 규칙이 맞는 메서드가 없으면 `reduce`에 원래 비교를 그대로 옮깁니다.

```rust
inbox.iter().reduce(|best, m| if m.size_kb > best.size_kb { m } else { best })
```

**`fold`는 마지막에 고릅니다.** `fold`는 어떤 누적이든 표현할 수 있어서, 거꾸로 말하면 이름만으로는 무엇을 만드는지 알려 주지 않습니다. 전용 메서드가 있으면 그것을 쓰고(`sum`, `count`, `max`...), 없을 때 `fold`를 씁니다. 예제 02의 5번 구역은 읽지 않은 메일의 개수와 합계를 튜플 하나로 들고 다니며 평균을 구합니다.

```rust
let (count, total) = inbox
    .iter()
    .filter(|m| m.unread)
    .fold((0, 0), |(n, sum), m| (n + 1, sum + m.size_kb));
total.checked_div(count) // 읽지 않은 메일이 없으면 None
```

**지침을 따르지 않아도 되는 경우.** 누적값이 둘을 넘거나, 한 누적값이 다른 누적값에 따라 바뀌기 시작하면 `fold`의 클로저는 반복문 본문을 옮겨 놓은 것과 다르지 않습니다. 이때는 반복문이 낫습니다(2.6절). 반대로, 슬라이스처럼 여러 번 돌아도 싼 데이터라면 `fold`로 묶지 않고 `count()`와 `sum()`을 따로 구하는 것도 좋은 선택입니다. 한 번 도는 것보다 읽기 쉬운 것이 먼저입니다.

### 2.3 받는 쪽 시그니처를 `impl IntoIterator`로 열기

반복문을 체인으로 바꾸고 나면 함수의 매개변수도 다시 보게 됩니다. [ch8](../ch8-references-pointers/README.md) 2.2절에서 읽기만 하는 매개변수는 `&Vec<T>`가 아니라 `&[T]`로 받는다는 지침을 보았습니다. 이 절은 한 걸음 더 나갑니다. 본문이 원소를 앞에서부터 한 번 읽기만 한다면 슬라이스조차 필요 이상입니다.

**문제 상황.** 카페의 하루 판매량(잔 수)에서 목표(120잔)를 넘긴 날을 세는 함수입니다. 부르는 쪽이 가진 데이터는 `Vec<u32>`, 날짜를 키로 하는 `BTreeMap<&str, u32>`, 지난주와 이번 주 두 `Vec`, 포스기 기록 문자열입니다.

**비권장 코드.**

```rust
fn hit_days_slice(cups: &[u32], target: u32) -> usize {
    cups.iter().filter(|&&c| c >= target).count()
}
```

[`03_open_signatures.rs`](examples/03_open_signatures.rs)의 1번 구역에서 `Vec`은 그대로 넘어가지만, 맵의 값은 `values().copied().collect()`로, 두 주는 `concat()`으로, 포스기 기록은 `collect()`로 새 `Vec`을 만든 뒤에야 넘길 수 있습니다. 함수는 원소를 한 번 읽고 버리는데, 부르는 쪽은 그것을 위해 원소 전체를 한곳에 모아 둬야 합니다.

**권장 코드.** 본문이 실제로 쓰는 능력, 즉 "원소를 차례로 꺼낼 수 있다"만 요구합니다.

```rust
fn hit_days(cups: impl IntoIterator<Item = u32>, target: u32) -> usize {
    cups.into_iter().filter(|&c| c >= target).count()
}

hit_days(this_week.iter().copied(), TARGET);                     // 빌린 Vec
hit_days(by_date.values().copied(), TARGET);                     // 맵의 값
hit_days(last_week.iter().chain(&this_week).copied(), TARGET);   // 이어 붙이기만
hit_days(cups_in(pos_log), TARGET);                              // 반복자를 돌려주는 함수
hit_days([120, 119], TARGET);                                    // 배열
```

[ch2](../ch2-express-common-behavior/README.md) 2.3절의 "가장 느슨한 바운드"와 같은 생각입니다. 원소 여러 개를 받는 매개변수를 받을 수 있는 것이 많은 순서로 늘어놓으면 `impl IntoIterator` → `impl Iterator` → `&[T]` → `&Vec<T>`입니다. 모든 반복자는 `IntoIterator`이므로(Part 1 ch5 2.2절의 포괄 구현) `impl Iterator`로 받는 것보다 `impl IntoIterator`로 받는 것이 더 많이 받습니다. `impl Iterator<Item = u32>` 매개변수에 `Vec`을 넘기면 ``E0277: `Vec<{integer}>` is not an iterator``입니다.

원소 타입을 값(`u32`)으로 정했으므로 빌린 컬렉션은 `.iter().copied()`를 붙여 넘깁니다. `hit_days(&this_week, TARGET)`는 원소가 `&u32`라서 ``E0271: type mismatch resolving `<&Vec<{integer}> as IntoIterator>::Item == u32` ``입니다. 값과 참조를 모두 받는 바운드(`Borrow`)는 ch8 2.4절에서 다뤘습니다.

**돌려줄 때도 반복자로.** 예제의 `cups_in`은 기록에서 꺼낸 잔 수를 `Vec`이 아니라 `impl Iterator<Item = u32>`로 돌려줍니다(Part 1 ch5 2.18절). 그래서 3번 구역에서 부르는 쪽이 `take(2)`로 앞부분만 보거나 `sum()`으로 더할 때 중간 `Vec`이 생기지 않고, `hit_days(cups_in(pos_log), TARGET)`처럼 함수 경계를 넘어 체인이 이어집니다.

**지침을 따르지 않아도 되는 경우: 본문이 슬라이스의 능력을 쓸 때.** 예제 03의 4번 구역은 "전날보다 많이 판 날"을 셉니다. 이웃한 두 날을 함께 보는 `windows(2)`는 슬라이스의 메서드이고, 반복자에는 안정판에서 쓸 수 있는 같은 메서드가 없습니다. 길이(`len`)가 먼저 필요하거나, 여러 번 돌거나, 정렬하거나 인덱스로 접근하는 함수도 마찬가지입니다. 이런 함수를 `impl IntoIterator`로 받으면 본문 첫 줄에서 `collect`하게 되어, 부르는 쪽이 하던 일을 받는 쪽이 숨겨서 할 뿐입니다. 이때는 `&[T]`가 정직한 시그니처입니다. 또 `impl IntoIterator`는 제네릭이라 넘기는 타입마다 함수가 따로 만들어집니다([`ch12-generics-vs-trait-objects`](../ch12-generics-vs-trait-objects/README.md)). 크레이트 안에서 한 가지 타입으로만 부른다면 굳이 열 필요가 없습니다.

### 2.4 반복문 안의 `?`를 체인으로 옮길 때의 선택지

**문제 상황.** 동아리 회비 입금 내역 `"이름 금액"` 줄들을 읽습니다. 반복문판은 줄마다 `parse_line(line)?`을 부르고, 한 줄이라도 틀리면 함수가 그 에러를 돌려주며 끝납니다. 이것을 `map`으로 옮기려고 클로저 안에 `?`를 쓰면 바깥 함수가 아니라 클로저에서 빠져나가려 하므로 클로저의 반환 타입이 맞지 않습니다(`for_each`라면 4.1절의 E0277). 체인에서는 "실패한 원소를 어떻게 다룰지"를 어댑터나 소비 메서드로 직접 정해야 합니다.

**비교.** [`04_collect_results.rs`](examples/04_collect_results.rs)는 같은 여섯 줄(3번째 줄은 금액이 `만오천`, 5번째 줄은 금액이 없음)을 여섯 가지로 읽습니다. "읽은 줄"은 `parse_line`이 불린 횟수입니다.

| 판 | 핵심 모양 | 섞인 입력의 결과 | 읽은 줄 | 실패한 줄은 |
| --- | --- | --- | --- | --- |
| 0. 반복문 | `all.push(parse_line(line)?)` | `Err(금액 오류 "다솜 만오천")` | 3/6 | 첫 실패에서 함수가 끝남 |
| 1. `collect` | `.map(..).collect::<Result<Vec<_>, _>>()` | 같음 | 3/6 | 첫 `Err`를 돌려줌 |
| 2. `filter_map` | `.filter_map(\|l\| parse_line(l).ok())` | 4명 | 6/6 | 조용히 버림 |
| 3. `partition` | `.partition(Result::is_ok)` | 4명 + 에러 2개 | 6/6 | 따로 모음 |
| 4. `try_fold` | `.try_fold(0u32, \|sum, l\| ..)` | `Err(금액 오류 "다솜 만오천")` | 3/6 | 첫 실패, 합계 넘침도 같은 길로 |
| 5. `try_for_each` | `.try_for_each(\|l\| { ..; Ok(()) })` | 같음, 장부에 2명 | 3/6 | 첫 실패, 그 전의 일은 이미 반영됨 |

1번이 반복문판과 같은 결과를 내는 이유는 `Result<V, E>: FromIterator<Result<T, E>>` 구현이 첫 `Err`에서 멈추기 때문입니다(Part 1 ch5 2.3절). 결과 타입은 함수의 반환 타입이 정해 주므로 터보피시도 필요 없습니다.

```rust
fn read_all(lines: &[&str]) -> Result<Vec<Dues>, DuesError> {
    lines.iter().map(|line| parse_line(line)).collect()
}
```

**무엇을 고를까.** 기준은 "실패한 원소를 어떻게 다뤄야 하는가"입니다.

| 요구 | 고를 것 |
| --- | --- |
| 하나라도 틀리면 전체가 무효(설정, 거래 묶음) | `collect::<Result<Vec<_>, _>>()` 뒤에 `?`. 값 하나로 접는다면 `sum::<Result<_, _>>()`나 `try_fold` |
| 틀린 것은 무시해도 됨(최선을 다해 읽기) | `filter_map(\|x\| f(x).ok())`. `.ok()`가 에러를 버린다는 결정이 코드에 드러남 |
| 틀린 것을 모두 보고해야 함(사용자에게 고칠 줄 알려 주기) | `partition`, 또는 반복문 + `match` |
| 접는 중에 다른 이유로도 실패함(넘침, 한도) | `try_fold`. 클로저가 `Result`를 돌려주므로 클로저 안에서 `?`를 쓸 수 있음 |
| 원소마다 일을 하다가 실패하면 멈춤 | 반복문 + `?`. 체인 끝이라면 `try_for_each` |

4번의 `try_fold` 클로저 안에서는 `?`를 씁니다. 클로저 자신이 `Result`를 돌려주므로 `?`는 클로저를 끝내고, `try_fold`가 그 `Err`를 받아 순회를 멈춥니다. "클로저 안의 `?`는 바깥에 닿지 않는다"는 사실을 거꾸로 이용하는 셈입니다.

```rust
lines.iter().try_fold(0u32, |sum, line| {
    let dues = parse_line(line)?;
    sum.checked_add(dues.amount).ok_or(DuesError::Overflow)
})
```

`fold(Ok(0), |acc, x| ..)`처럼 `fold`의 누적값을 `Result`나 `Option`으로 두면 결과는 같아도 첫 실패 뒤에 남은 원소를 끝까지 돕니다. clippy는 이 모양을 `manual_try_fold`로 잡습니다(``usage of `Iterator::fold` on a type that implements `Try` ``). `.map(parse).flatten()`도 `Err`를 조용히 버리는 코드인데, clippy의 `map_flatten`은 이것을 `flat_map`으로 줄이라고만 제안하고 에러를 버린다는 점은 알려 주지 않습니다.

**지침을 따르지 않아도 되는 경우: 나눠 모으기는 반복문도 좋습니다.** `partition`은 `Result`를 벗기지 않으므로, 예제 04의 `read_split`은 `flatten`과 `filter_map(Result::err)`로 한 번씩 더 돌아 안의 값을 꺼냅니다(`Result`도 `Ok`면 값 하나, `Err`면 0개를 내놓는 `IntoIterator`라서 `flatten`이 `Ok` 안의 값만 남깁니다). 같은 일을 하는 반복문 + `match`판(`read_split_loop`)은 한 번 돌면서 바로 두 `Vec`에 나눠 담고, 결과도 같습니다. 이 경우는 어느 쪽을 골라도 좋습니다. 외부 크레이트 [`itertools`](https://docs.rs/itertools/latest/itertools/trait.Itertools.html)(docs.rs에서 0.15.0 확인)에는 `Result`의 반복자를 받아 `Ok` 값과 `Err` 값을 두 컬렉션에 나눠 담는 `partition_result`가 있습니다. 이 챕터의 예제는 외부 크레이트를 쓰지 않습니다.

### 2.5 부수 효과가 본문이면 `for_each`보다 `for`

**문제 상황.** 로켓 발사 전 점검표를 차례로 확인하며 통과한 항목을 기록하고, 실패한 항목을 만나면 그 이름을 돌려주며 멈춥니다. 이 반복문은 값을 만들지 않고 "일을 합니다". 반복문을 반복자로 바꾸는 습관이 붙으면 이런 본문도 `for_each`로 옮기고 싶어집니다.

**비권장 코드.** [`05_for_each_vs_for.rs`](examples/05_for_each_vs_for.rs)의 `countdown_for_each`는 `for`판의 본문을 그대로 클로저에 옮겼습니다.

```rust
checks.iter().for_each(|c| {
    if !c.passed {
        return; // 바깥 함수가 아니라 이 클로저에서 빠져나갑니다
    }
    report.push(c.name);
});
Ok(())
```

컴파일되고 경고도 없습니다. 하지만 클로저 안의 `return`은 클로저 한 번의 호출만 끝내므로 `continue`처럼 동작합니다. 예제 05의 2번 구역에서 이 판은 `연료 압력`이 실패했는데도 뒤의 두 항목을 계속 기록하고 `Ok(())`를 돌려줍니다. `for`판은 `Err("연료 압력")`과 함께 두 항목만 기록합니다.

클로저 본문은 반복문의 본문이 아니라 따로 불리는 함수입니다. 그래서 흐름을 바꾸는 키워드가 모두 클로저에서 멈춥니다.

| 클로저 안에서 쓴 것 | 결과 |
| --- | --- |
| `return` | 클로저 한 번만 끝냄. `for_each`에서는 `continue`처럼 동작(컴파일됨, 경고 없음) |
| `?` | 클로저가 `Result`/`Option`을 돌려줄 때만 됨. `for_each`의 클로저에서는 E0277(4.1절) |
| `break`, `continue` | E0267(4.2절) |

**권장 코드.** `for`로 씁니다. 체인을 꼭 유지해야 한다면 `try_for_each`가 실패에서 멈출 수 있습니다.

```rust
for c in checks {
    if !c.passed {
        return Err(c.name);
    }
    report.push(c.name);
}
Ok(())

// 체인판: 클로저가 Err를 돌려주면 try_for_each가 멈추고 그 Err를 돌려줍니다
checks.iter().try_for_each(|c| {
    if !c.passed {
        return Err(c.name);
    }
    report.push(c.name);
    Ok(())
})
```

두 판의 결과는 같습니다. 그래도 `for`판이 낫습니다. `try_for_each`판은 끝에 `Ok(())`를 직접 돌려줘야 하고, `return`이 "함수를 끝낸다"가 아니라 "클로저가 `Err`를 돌려주고 그래서 순회가 멈춘다"는 두 단계로 읽힙니다. 1번 구역처럼 흐름 제어가 전혀 없는 본문이라도 `for_each`가 얻게 해 주는 것은 없습니다. 값을 만드는 단계는 어댑터와 소비 메서드로, 일을 하는 단계는 `for`로 쓴다고 나누면 읽는 사람이 둘을 바로 구별할 수 있습니다.

clippy에는 `for_each`를 `for`로 바꾸라는 `needless_for_each`가 있지만 pedantic 그룹이라 기본으로 꺼져 있습니다. 이 판단은 도구가 해 주지 않습니다.

**지침을 따르지 않아도 되는 경우.** 긴 체인 끝에서 짧은 부수 효과 하나를 낼 때(`.filter(..).map(..).for_each(|line| out.push(line))`)는 `for_each`도 읽기 좋습니다. 이때도 `for line in 체인 { .. }`으로 쓸 수 있다는 점은 같습니다. 원소 하나하나가 아니라 결과 전체를 모으는 것이 목적이라면 `out.extend(체인)`이 더 짧습니다.

### 2.6 탈출 조건과 상태가 많으면 반복문에서 멈추기

**문제 상황.** 올라가는 엘리베이터에 큐의 사람을 태웁니다. 내려가는 사람은 건너뛰고(`continue`), 정원 4명이 차거나(`break`) 다음 사람이 타면 300kg을 넘을 때(`break`) 마감합니다. 함수는 탑승자, 건너뛴 사람, 무게, 마감 이유 네 가지를 돌려줍니다.

**권장 코드.** [`06_keep_the_loop.rs`](examples/06_keep_the_loop.rs)의 `board_loop`입니다. 조건 셋이 각자 한 줄씩이고, 네 가지 상태를 같은 자리에서 고칩니다.

```rust
for w in queue {
    if !w.going_up {
        b.skipped.push(w.name);
        continue;
    }
    if b.riders.len() == CAPACITY {
        b.stop = Some(Stop::Full);
        break;
    }
    if b.load_kg + w.kg > MAX_KG {
        b.stop = Some(Stop::Overweight(w.name));
        break;
    }
    b.load_kg += w.kg;
    b.riders.push(w.name);
}
```

**비권장 코드.** 같은 일을 `try_fold` 하나로 옮긴 `board_try_fold`입니다. 클로저 안에서는 `continue`와 `break`를 쓸 수 없으므로 `std::ops::ControlFlow`로 바꿔 돌려줍니다. `ControlFlow`는 `Continue(값)`과 `Break(값)` 두 배리언트를 가진 표준 라이브러리의 열거형이고, `try_fold`는 `Break`를 `Err`처럼 받아 순회를 멈춥니다.

```rust
let flow = queue.iter().try_fold(Boarding::default(), |mut b, w| {
    if !w.going_up {
        b.skipped.push(w.name);
        return ControlFlow::Continue(b);
    }
    // ... 두 조건은 return ControlFlow::Break(b)
    ControlFlow::Continue(b)
});
match flow {
    ControlFlow::Continue(b) | ControlFlow::Break(b) => b,
}
```

결과는 세 큐 모두에서 같습니다(예제 06의 1번 구역). 하지만 본문은 그대로 클로저 안에 있고, 상태를 매번 받아서 다시 돌려주는 일과 `ControlFlow`를 벗기는 `match`가 더해졌을 뿐입니다. 체인이 줄여 준 것이 없습니다. 상태를 클로저 하나에 몰지 않고 단계마다 나눠 고치려 하면, 두 클로저가 같은 상태를 가변으로 빌려 E0499가 납니다(4.3절).

**클로저 안에 큰 `match`가 들어갈 때.** 3번 구역은 TV 리모컨 버튼 목록을 차례로 적용합니다. 버튼마다 볼륨, 음소거, 채널, 이전 채널 중 일부를 고치고, `"전원"`을 누르면 남은 버튼을 무시합니다. `for` + `match`판은 버튼마다 갈래 하나이고 `"전원" => break`도 그 갈래 중 하나입니다. 체인판은 `take_while(|&&b| b != "전원")`이 전원을 맡고, `fold`의 클로저 안에 나머지 갈래가 든 `match`가 들어갑니다. 클로저 본문이 곧 반복문 본문이라 체인이 덜어 준 것이 거의 없고, 버튼 처리 규칙을 두 곳에서 읽어야 합니다. ch3 2.5절의 "클로저 안에 `match`가 들어갔다면 이미 `match`를 쓰면서 한 겹 더 깊어진 것"과 같은 판단입니다.

책도 본문이 크거나 여러 일을 하는 반복문, 함수 전체를 끝내는 에러 처리는 명시적으로 두라고 권하고, 바꾸는 것이 억지스러우면 바꾸지 말라고 말합니다.

**지침을 따르지 않아도 되는 경우: 돌려줄 것이 하나뿐이면 체인으로 충분합니다.** 같은 엘리베이터라도 탑승자 명단만 필요하다면, 조건마다 어댑터 하나씩으로 나뉜 체인이 읽기 좋습니다(2번 구역, 세 큐에서 `for`판의 명단과 같음).

```rust
queue
    .iter()
    .filter(|w| w.going_up) // 올라가는 사람만
    .take(CAPACITY)         // 정원까지
    .scan(0, |load, w| {
        // 한도를 넘는 사람이 나오면 None으로 멈춤
        *load += w.kg;
        (*load <= MAX_KG).then_some(w.name)
    })
    .collect()
```

`scan`은 상태(누적 무게)를 들고 가다가 `None`을 돌려주면 멈추는 어댑터입니다(Part 1 ch5 2.8절). 탈출 조건이 여럿이어도 결과가 하나고 조건마다 맞는 어댑터가 있으면 체인이 낫습니다. 반복문으로 돌아갈 신호는 **돌려줄 것이 여럿**이거나, **여러 상태를 한 자리에서 함께** 고치거나, **클로저 안에 다시 갈래**가 생기는 것입니다.

### 2.7 고르는 순서

반복문을 새로 쓰거나 고칠 때 다음 순서로 물어봅니다.

1. **값을 만드는가, 일을 하는가?** 출력·기록·전송처럼 일을 한다면 `for`로 씁니다(2.5절).
2. **무엇을 만드는가?** 컬렉션은 `collect`, 개수는 `count`, 합은 `sum`, 첫 원소는 `find`, 가장 큰 것은 `max_by_key` 등, 그 밖의 값 하나는 `fold`/`reduce`입니다. 바꾼 뒤 같은 값이 여럿일 때의 규칙을 확인합니다(2.2절).
3. **어디까지 보고, 무엇을 건너뛰고, 원소를 어떻게 바꾸는가?** `take_while`/`take`, `filter`/`filter_map`, `map`입니다. `break`의 조건은 뒤집어 `take_while`에 넣습니다(2.1절).
4. **실패할 수 있는가?** 하나라도 틀리면 무효인지, 버려도 되는지, 모두 보고해야 하는지로 `collect::<Result<..>>()`, `filter_map`, `partition`을 고릅니다. 접으면서 실패하면 `try_fold`입니다(2.4절).
5. **탈출 조건마다 결과가 다르거나, 여러 상태를 함께 바꾸거나, 클로저 안에 `match`가 필요한가?** 그렇다면 `for`에서 멈춥니다(2.6절).
6. **이 반복을 함수로 뽑는가?** 원소를 한 번 읽기만 하면 매개변수는 `impl IntoIterator<Item = T>`, 슬라이스의 능력이 필요하면 `&[T]`입니다(2.3절).

### 2.8 책과 달라진 점

책은 Rust 1.70과 2018 에디션을 기준으로 합니다. 저장소 툴체인(1.100.0, 2024 에디션)에서 확인한 차이는 다음과 같습니다.

- **`try_find`는 아직 안정판에 없습니다.** 책은 실패할 수 있는 클로저를 받는 소비 메서드로 `try_for_each`, `try_fold`와 함께 `try_find`를 소개하고, 불안정하다는 말은 하지 않습니다. 1.100.0의 로컬 문서에서 `try_find`에는 nightly 전용 실험 API 표시가 붙어 있고, 안정판에서 부르면 ``E0658: use of unstable library feature `try_find` ``입니다. 실패할 수 있는 조건으로 찾기는 `for` + `?`로 씁니다.
- **clippy가 반복문 모양을 알려 줍니다.** 책은 린트를 언급하지 않습니다. 지금 clippy는 기본 설정에서 `needless_range_loop`(2.1절), `unnecessary_fold`·`manual_find`·`explicit_counter_loop`(2.2절), `manual_try_fold`·`map_flatten`(2.4절)을 경고합니다. 반면 `needless_for_each`와 `needless_collect`는 기본으로 꺼져 있어서, 2.5절의 판단과 "모았다가 바로 다시 도는" 코드는 직접 찾아야 합니다.
- **책에 없는 지침.** 2.3절(받는 쪽 매개변수를 `impl IntoIterator`로 열기)은 책의 Item 9가 권하는 내용이 아닙니다(2026-11-19에 온라인판에서 확인). 책은 `IntoIterator`를 `for` 반복문이 쓰는 트레이트로만 설명합니다. 이 지침은 [`IntoIterator` 문서](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)가 이 트레이트를 바운드로 쓰는 것이 흔하다며 함수 인자의 예를 보여 준다는 점과 ch2의 "가장 느슨한 바운드"를 근거로 이 챕터가 더한 것입니다. 2.4절의 선택표 가운데 `partition`과 `try_fold`를 고르는 기준도 같습니다. 책은 `Result<Vec<_>, _>`로 모으기를 권하고 `flatten`으로 실패를 버릴 수 있다고 말하지만, `partition`과 `try_fold`는 소비 메서드 목록에서 소개할 뿐 실패를 다루는 방법으로 언제 고를지는 말하지 않습니다. 그 기준은 이 챕터가 정리했습니다.

### 2.9 기억할 것

- 인덱스 → 원소 → 어댑터 → 소비 메서드 순서로 바꿉니다. `break`는 조건을 뒤집어 `take_while`로, `continue`는 `filter`/`filter_map`으로, 누적 변수는 무엇을 만드는지에 맞는 소비 메서드로 갑니다.
- 소비 메서드로 바꾼 뒤에는 동작이 정말 같은지(같은 최댓값 중 무엇을 고르는지) 확인합니다. 맞는 메서드가 없으면 `reduce`와 `fold`에 원래 규칙을 옮깁니다.
- 원소를 한 번 읽기만 하는 함수는 `impl IntoIterator<Item = T>`로 받고, 길이·인덱스·`windows`가 필요하면 `&[T]`로 받습니다.
- 클로저 안의 `return`·`?`·`break`·`continue`는 바깥에 닿지 않습니다. 반복문의 `?`는 실패한 원소를 어떻게 다룰지에 따라 `collect::<Result<..>>()`, `filter_map`, `partition`, `try_fold`/`try_for_each`로 옮깁니다.
- 일을 하는 본문, 결과가 여럿인 탈출, 함께 바뀌는 여러 상태, 클로저 안의 `match`는 `for`에서 멈추라는 신호입니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_loop_to_chain.rs`](examples/01_loop_to_chain.rs) | 같은 함수의 다섯 판(인덱스, 원소, `take_while`, `filter_map`, `map` + `sum`), 1부터 센 인덱스의 실수 | 1 |
| [`examples/02_accumulator_to_consumer.rs`](examples/02_accumulator_to_consumer.rs) | `push`·카운터·합계·찾기·순번·최댓값·두 누적값을 소비 메서드로, 같은 최댓값 처리 | 2 |
| [`examples/03_open_signatures.rs`](examples/03_open_signatures.rs) | `&[u32]`판과 `impl IntoIterator`판에 여러 데이터 넘기기, 반복자를 돌려주는 함수, 슬라이스가 맞는 함수 | 3 |
| [`examples/04_collect_results.rs`](examples/04_collect_results.rs) | 같은 입력을 반복문 + `?`, `collect`, `filter_map`, `partition`, `try_fold`, `try_for_each`로 읽고 읽은 줄 수 비교 | 4 |
| [`examples/05_for_each_vs_for.rs`](examples/05_for_each_vs_for.rs) | 부수 효과만 내는 본문, `for_each` 안의 `return`이 `continue`처럼 동작하는 모습, `try_for_each` | 5 |
| [`examples/06_keep_the_loop.rs`](examples/06_keep_the_loop.rs) | 탈출 조건과 결과가 여럿인 반복문과 `try_fold` + `ControlFlow`, 결과가 하나일 때의 체인, 클로저 안의 `match` | 5 |
| [`compile_fail/`](compile_fail) | 반복문 본문을 클로저로 옮기다 만나는 컴파일 에러 세 가지 | 4, 5 |

예제 명령은 저장소 루트에서 실행한다고 보고 썼습니다. 어느 예제도 패닉하지 않으며 모두 종료 코드 0으로 끝나고, 실행할 때마다 같은 결과가 나옵니다. clippy가 잡는 비권장 판에는 `#[expect(clippy::…, reason = "…")]`를 붙여 두었습니다. 속성을 지우고 `cargo clippy -p er-ch9-iterator-transforms --examples`를 실행하면 경고를 직접 볼 수 있습니다. 2절에 인용한 경고의 행 번호는 그 예제의 `#[expect]`를 모두 지웠을 때의 번호입니다.

### 01_loop_to_chain — 인덱스 반복문에서 체인까지

```console
$ cargo run -q -p er-ch9-iterator-transforms --example 01_loop_to_chain
== 1. 다섯 판의 결과 비교 ==
일지 A (9일치) → 215분 (다섯 판 일치: true)
빈 일지 → 0분 (다섯 판 일치: true)
15일째부터 쓴 일지 → 0분 (다섯 판 일치: true)

== 2. 인덱스판의 실수: 1부터 센 인덱스 ==
체인판: 215분, 1부터 센 인덱스판: 170분
```

볼 점:

- 일지 A의 215분은 45 + 60(90분 중) + 30 + 60(75분 중) + 20입니다. 15일째부터는 `take_while`이 멈추므로 120분과 50분은 보지도 않습니다.
- `15일째부터 쓴 일지`는 첫 원소에서 `take_while`의 조건이 거짓이라 아무것도 더하지 않습니다. 반복문판의 첫 `break`와 같습니다.

### 02_accumulator_to_consumer — 누적 변수를 소비 메서드로

```console
$ cargo run -q -p er-ch9-iterator-transforms --example 02_accumulator_to_consumer
== 1. push → collect ==
반복문판 ["회의록", "휴가 신청", "설계 도면"], 체인판 ["회의록", "휴가 신청", "설계 도면"] (일치: true)

== 2. 카운터와 합계 → count, sum ==
읽지 않은 메일: 반복문판 3통, 체인판 3통
전체 용량: fold판 6553KB, sum판 6553KB

== 3. 찾으면 return, 순번 변수 → find, enumerate ==
1000KB보다 큰 첫 메일: 반복문판 견적서, 체인판 견적서
5000KB보다 큰 첫 메일: 반복문판 없음, 체인판 없음
번호 붙이기: ["1. 회의록", "2. 견적서"] … (일치: true)

== 4. 최댓값 갱신: 같은 값이 있을 때 ==
반복문판(>로 갱신): 견적서
max_by_key판: 설계 도면
reduce판(같은 비교): 견적서

== 5. 누적값이 둘 → fold ==
받은편지함의 읽지 않은 메일 평균: 반복문판 Some(1119), fold판 Some(1119)
모두 읽은 편지함의 읽지 않은 메일 평균: 반복문판 None, fold판 None
```

볼 점:

- 4번 구역만 판끼리 결과가 다릅니다. 컴파일러도 clippy도 이 차이를 알려 주지 않으므로, 같은 값이 들어 있는 입력으로 직접 확인해야 합니다.
- 5번 구역의 평균은 정수 나눗셈이라 3358 ÷ 3 = 1119입니다. 읽지 않은 메일이 없으면 `checked_div`가 0으로 나누는 대신 `None`을 돌려줍니다.

### 03_open_signatures — 받는 쪽 열기

```console
$ cargo run -q -p er-ch9-iterator-transforms --example 03_open_signatures
== 1. &[u32]판: 부르는 쪽이 먼저 Vec을 만듭니다 ==
이번 주 4일, 날짜별 기록 2일, 두 주 합쳐 5일, 포스기 기록 2일

== 2. impl IntoIterator판: 가진 것을 그대로 넘깁니다 ==
이번 주 4일, 날짜별 기록 2일, 두 주 합쳐 5일, 포스기 기록 2일, 배열 1일
이번 주 Vec을 통째로 넘김: 4일

== 3. 반복자를 돌려받은 쪽이 고릅니다 ==
앞의 두 줄 [128, 75], 합계 334잔

== 4. 슬라이스가 맞는 경우: 이웃한 원소를 함께 봄 ==
전날보다 많이 판 날: 3일
```

볼 점:

- 1번과 2번 구역의 결과는 같습니다. 다른 것은 1번 구역의 소스에만 있는 `values`, `joined`, `parsed` 세 `Vec`입니다.
- 2번 구역의 마지막 호출은 `this_week`를 값으로 넘깁니다. `Vec<u32>`도 `IntoIterator<Item = u32>`이므로 더 쓰지 않을 컬렉션은 복사 없이 통째로 넘길 수 있습니다.

### 04_collect_results — 반복문의 `?`를 옮기는 여섯 가지

```console
$ cargo run -q -p er-ch9-iterator-transforms --example 04_collect_results
==== 깨끗한 입력 (3줄) ====
0. for + ?     : Ok(["가온", "나래", "라온"]) (읽은 줄 3/3)
1. collect     : Ok(["가온", "나래", "라온"]) (읽은 줄 3/3)
2. filter_map  : ["가온", "나래", "라온"] (읽은 줄 3/3)
3. partition   : ["가온", "나래", "라온"] / 에러 [] (읽은 줄 3/3)
   for + match : partition과 같은 결과 true (읽은 줄 3/3)
4. try_fold    : Ok(50000) (읽은 줄 3/3)
5. try_for_each: Ok(()) / 장부 {"가온": 15000, "나래": 15000, "라온": 20000} (읽은 줄 3/3)

==== 섞인 입력 (6줄) ====
0. for + ?     : Err(금액 오류 "다솜 만오천") (읽은 줄 3/6)
1. collect     : Err(금액 오류 "다솜 만오천") (읽은 줄 3/6)
2. filter_map  : ["가온", "나래", "라온", "보람"] (읽은 줄 6/6)
3. partition   : ["가온", "나래", "라온", "보람"] / 에러 ["금액 오류 \"다솜 만오천\"", "금액 없음 \"미르\""] (읽은 줄 6/6)
   for + match : partition과 같은 결과 true (읽은 줄 6/6)
4. try_fold    : Err(금액 오류 "다솜 만오천") (읽은 줄 3/6)
5. try_for_each: Err(금액 오류 "다솜 만오천") / 장부 {"가온": 15000, "나래": 15000} (읽은 줄 3/6)

==== try_fold만의 실패: 합계가 u32를 넘음 ====
4. try_fold    : Err(합계가 너무 큼) (읽은 줄 2/3)
```

볼 점:

- 깨끗한 입력에서는 여섯 판이 모두 같은 일을 합니다. 판의 차이는 실패가 있을 때만 드러나므로, 반복문을 바꿀 때는 틀린 입력으로 시험해 봅니다.
- 섞인 입력에서 첫 실패에서 멈추는 판(0, 1, 4, 5)은 3줄만 읽습니다. 다섯째 줄 `미르`의 문제는 `partition`판만 알려 줍니다.
- 5번의 장부에는 실패 전에 처리한 두 명이 이미 들어 있습니다. `try_for_each`는 그 전에 한 일을 되돌리지 않습니다. 반복문 + `?`도 같습니다.
- 읽은 줄 수는 `parse_line`이 정적 변수 `AtomicUsize`(Part 1 ch11)에 센 값입니다. 관찰을 위한 장치일 뿐 판의 일부가 아닙니다.

### 05_for_each_vs_for — 일을 하는 본문

```console
$ cargo run -q -p er-ch9-iterator-transforms --example 05_for_each_vs_for
== 1. 부수 효과만 낼 때: 결과는 같습니다 ==
["전원: 통과", "통신: 통과"] … (for_each판과 for판 일치: true)

== 2. 실패하면 멈춰야 할 때 ==
for판: Err("연료 압력"), 통과 기록 ["전원", "통신"]
for_each판: Ok(()), 통과 기록 ["전원", "통신", "항법 장치", "발사대 고정"]
try_for_each판: Err("연료 압력"), 통과 기록 ["전원", "통신"]
```

볼 점:

- 2번 구역의 `for_each`판은 틀린 결과를 내지만 컴파일러와 clippy 기본 설정 모두 아무 말도 하지 않습니다. 틀린 코드가 조용히 통과하는 이 경우가 `for`를 고를 가장 큰 이유입니다.
- 세 판은 시그니처가 같아서 2번 구역에서 함수 포인터 배열에 담아 차례로 불렀습니다(함수 포인터는 ch2 2.2절).

### 06_keep_the_loop — 반복문이 나은 곳

```console
$ cargo run -q -p er-ch9-iterator-transforms --example 06_keep_the_loop
== 1. 탈출 조건이 여럿이고 결과도 여럿: for판과 try_fold판 ==
큐 A → 탑승 ["가람", "다온", "루다"] 225kg, 건너뜀 ["나리"], 마감 Some(Overweight("마루")) (두 판 일치: true)
큐 B → 탑승 ["가람", "나리", "다온", "루다"] 210kg, 건너뜀 [], 마감 Some(Full) (두 판 일치: true)
큐 C → 탑승 ["가람"] 70kg, 건너뜀 ["나리"], 마감 None (두 판 일치: true)

== 2. 명단만 필요하면: filter + take + scan ==
큐 A → ["가람", "다온", "루다"] (for판의 명단과 일치: true)
큐 B → ["가람", "나리", "다온", "루다"] (for판의 명단과 일치: true)
큐 C → ["가람"] (for판의 명단과 일치: true)

== 3. 클로저 안의 match: for + match판과 take_while + fold판 ==
Tv { volume: 11, muted: true, channel: 7, previous: 11 } (두 판 일치: true)
```

볼 점:

- 큐 A의 `바다`(50kg)는 타도 한도 안이지만 타지 못합니다. `마루`에서 마감했기 때문입니다. `for`판의 `break`와 체인판의 `scan`(`None`에서 멈춤)이 같은 규칙을 따릅니다.
- 3번 구역에서 `"전원"` 뒤의 `"채널 9"`는 적용되지 않았습니다. `볼륨+`가 음소거를 풀고 뒤의 `음소거`가 다시 걸었으므로 마지막 상태는 `muted: true`입니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd appd-a/ch9-iterator-transforms
rustc --edition 2024 compile_fail/e0277_question_in_for_each.rs --out-dir /tmp/rust-er-ch9
```

세 실수는 모두 같은 뿌리에서 나옵니다. 반복문의 본문을 그대로 클로저에 옮겼는데, 클로저 본문은 반복문 본문이 아니라 따로 불리는 함수입니다.

### 4.1 `for_each`의 클로저 안에서 `?` — E0277

파일: [`compile_fail/e0277_question_in_for_each.rs`](compile_fail/e0277_question_in_for_each.rs)

```text
error[E0277]: the `?` operator can only be used in a closure that returns `Result` or `Option` (or another type that implements `FromResidual`)
  --> compile_fail/e0277_question_in_for_each.rs:15:44
   |
14 |     lines.iter().for_each(|line| {
   |                           ------ this function should return `Result` or `Option` to accept `?`
15 |         let cups: u32 = line.trim().parse()?;
   |                                            ^ cannot use the `?` operator in a closure that returns `()`
```

- **읽는 법**: 14행의 레이블이 클로저를 "this function"이라고 부릅니다. `?`가 빠져나가려는 곳은 바깥 함수 `add_all`이 아니라 이 클로저이고, `for_each`가 받는 클로저는 `()`를 돌려주므로 `Err`를 돌려줄 수 없다는 뜻입니다.
- **원인**: `for line in lines { totals.push(line.trim().parse()?); }`의 본문을 `for_each`로 옮겼습니다. 바깥 함수의 반환 타입이 `Result`라는 사실은 클로저와 상관이 없습니다. [Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md) 2.11절에서는 `map`의 클로저(`i32`를 돌려줌)에서 같은 에러를 보았습니다.
- **해결**: 2.4절의 표로 고릅니다. 원소마다 일을 하다 실패하면 멈추는 것이 목적이므로 `for` 반복문으로 되돌리는 것이 가장 간단합니다. 체인을 유지하려면 `try_for_each`로 바꾸고 클로저 끝에서 `Ok(())`를 돌려줍니다. 값을 모으는 것이 목적이면 `collect::<Result<Vec<u32>, _>>()?`로 모은 뒤 `extend`합니다.

### 4.2 클로저 안에서 `break` — E0267

파일: [`compile_fail/e0267_break_inside_closure.rs`](compile_fail/e0267_break_inside_closure.rs)

```text
error[E0267]: `break` inside of a closure
  --> compile_fail/e0267_break_inside_closure.rs:16:13
   |
14 |     checks.iter().for_each(|&(name, ok)| {
   |                            ------------- enclosing closure
15 |         if !ok {
16 |             break;
   |             ^^^^^ cannot `break` inside of a closure
```

- **읽는 법**: `enclosing closure` 레이블이 `break`를 감싼 것이 반복문이 아니라 클로저라고 알려 줍니다. 클로저 안에는 끝낼 반복문이 없습니다. `continue`를 쓰면 `` `continue` inside of a closure ``로 같은 E0267입니다.
- **원인**: 실패한 점검에서 멈추던 `for` 반복문을 `for_each`로 옮겼습니다.
- **해결**: 부수 효과(`passed.push`)가 본문이므로 `for`로 되돌립니다(2.5절). 이 함수처럼 통과한 이름을 모으는 것이 전부라면 `checks.iter().take_while(|(_, ok)| *ok).map(|(name, _)| *name).collect()`처럼 멈추는 조건을 어댑터에 맡깁니다. `break`를 `return`으로 바꾸면 컴파일은 되지만 `continue`처럼 동작해 결과가 틀립니다(예제 05의 2번 구역).

### 4.3 체인의 두 클로저가 같은 상태를 고침 — E0499

파일: [`compile_fail/e0499_two_closures_same_state.rs`](compile_fail/e0499_two_closures_same_state.rs)

```text
error[E0499]: cannot borrow `notes` as mutable more than once at a time
  --> compile_fail/e0499_two_closures_same_state.rs:24:21
   |
17 |         .filter(|line| {
   |                 ------ first mutable borrow occurs here
18 |             if line.is_empty() {
19 |                 notes.push("빈 줄".to_string());
   |                 ----- first borrow occurs due to use of `notes` in closure
...
24 |         .filter_map(|line| match line.parse() {
   |          ---------- ^^^^^^ second mutable borrow occurs here
   |          |
   |          first borrow later used by call
...
27 |                 notes.push(format!("숫자가 아님: {line}"));
   |                 ----- second borrow occurs due to use of `notes` in closure
```

- **읽는 법**: 첫 번째 대여는 `filter`의 클로저가 `notes`를 캡처하면서, 두 번째 대여는 `filter_map`의 클로저가 캡처하면서 생겼습니다. `first borrow later used by call`은 첫 클로저가 `filter_map`을 부르는 시점에도 살아 있다는 뜻입니다.
- **원인**: 어댑터는 클로저를 필드로 들고 있다가(Part 1 ch5 2.4절) `collect`가 돌 때 번갈아 부릅니다. 그래서 두 클로저가 `notes`를 가변으로 빌린 채 동시에 살아 있어야 합니다. 반복문 본문이라면 한 자리에서 차례로 고치므로 문제가 되지 않습니다.
- **해결**: 상태를 여러 단계에서 고쳐야 한다면 `for` + `match`로 씁니다(2.6절). 체인을 유지하려면 상태를 고치는 단계를 하나로 합칩니다. 이 경우 `filter_map` 하나가 빈 줄도 함께 처리하면 됩니다. `RefCell`로 감싸 대여 검사를 실행 중으로 미루는 방법도 있지만, 대여 규칙을 피하기 위한 구조 변경은 [`ch15-borrow-checker`](../ch15-borrow-checker/README.md)에서 다룹니다.

### 4.4 그 밖의 실수

모두 이 툴체인에서 직접 컴파일하거나 실행해 확인했습니다.

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| `for_each` 안에서 `return`으로 멈추려 함 | 컴파일됨, 경고 없음. `continue`처럼 동작해 결과가 틀림 | `for`, 또는 `try_for_each` (2.5절) |
| 최댓값 갱신 반복문(`>`)을 `max_by_key`로 바꿈 | 같은 값이 있으면 다른 원소가 나옴 | `reduce`에 원래 비교, 또는 `min_by_key(\|x\| Reverse(키))` (2.2절) |
| `break` 조건을 그대로 `take_while`에 넣음 | 컴파일됨. 대개 첫 원소에서 바로 멈춰 결과가 비어 버림 | 조건을 뒤집음(멈출 조건 → 계속할 조건) (2.1절) |
| `.map(parse).flatten()` | clippy `map_flatten`. `flat_map`으로 바꿔도 `Err`는 조용히 버려짐 | 버릴 생각이면 `filter_map(\|x\| parse(x).ok())`, 아니면 2.4절의 표 |
| `fold(Ok(0), \|acc, x\| ..)` / `fold(Some(0), ..)` | clippy `manual_try_fold`. 첫 실패 뒤에도 끝까지 돎 | `try_fold` (2.4절) |
| `Item = u32`를 받는 함수에 `&vec` | E0271 `` type mismatch resolving `<&Vec<{integer}> as IntoIterator>::Item == u32` `` | `vec.iter().copied()` (2.3절) |
| `impl Iterator` 매개변수에 `Vec` | E0277 `` `Vec<{integer}>` is not an iterator `` | 매개변수를 `impl IntoIterator`로 (2.3절) |
| 안정판에서 `try_find` | E0658 `` use of unstable library feature `try_find` `` | `for` + `?` (2.8절) |

## 5. 연습 문제

### 연습 1. 인덱스 반복문 풀기

온라인 강의 목록의 재생 시간(분)에서 "수강한 시간"을 구하는 함수입니다. 0분인 강의부터는 아직 열리지 않은 것이라 세지 않고, 5분 미만의 맛보기 영상은 뺍니다. 2.1절의 단계를 밟아 체인으로 바꾸고, 매개변수를 `impl IntoIterator<Item = u32>`로 바꾸세요.

```rust
fn watched_minutes(lectures: &[u32]) -> u32 {
    let mut total = 0;
    for i in 0..lectures.len() {
        if lectures[i] == 0 {
            break;
        }
        if lectures[i] < 5 {
            continue;
        }
        total += lectures[i];
    }
    total
}
```

```text
vec![12, 3, 45, 0, 30]                               → 57
vec![]                                               → 0
vec![4, 4, 60]                                       → 60
[30, 2, 0] (배열을 그대로)                           → 30
BTreeMap {"1강": 20, "2강": 0, "3강": 15}의 값       → 20
```

<details>
<summary>힌트</summary>

- `break`는 `take_while(|&m| m != 0)`, `continue`는 `filter(|&m| m >= 5)`입니다. 조건의 방향을 확인하세요. `continue`의 조건(`< 5`)도 `filter`에서는 남길 조건(`>= 5`)으로 뒤집힙니다.
- 원소가 `u32` 값이므로 클로저 인자는 `&u32`이고, `|&m|` 패턴으로 받으면 `m`이 `u32`가 됩니다.
- `Vec`은 `v.iter().copied()`로, 맵은 `map.values().copied()`로 넘깁니다. `BTreeMap`은 키 순서로 돌므로 `20, 0, 15`가 되어 `2강`에서 멈춥니다.

</details>

### 연습 2. 최다 득표자

투표 결과에서 총 투표 수와 최다 득표자(동률이면 먼저 나온 후보)를 구하는 반복문을 소비 메서드로 바꾸세요. 먼저 `max_by_key`로 써 보고 결과가 왜 다른지 설명한 뒤 고치세요.

```rust
fn tally(votes: &[(&'static str, u32)]) -> (u32, Option<&'static str>) {
    let mut total = 0;
    let mut winner: Option<(&'static str, u32)> = None;
    for &(name, count) in votes {
        total += count;
        if winner.is_none_or(|(_, best)| count > best) {
            winner = Some((name, count));
        }
    }
    (total, winner.map(|(name, _)| name))
}
```

```text
[("가람", 12), ("나리", 30), ("다온", 30)] → (72, Some("나리"))   (max_by_key로 쓰면 "다온")
[]                                         → (0, None)
```

<details>
<summary>힌트</summary>

- 총 투표 수는 `votes.iter().map(|&(_, c)| c).sum()`입니다. 슬라이스는 두 번 돌아도 싸므로 `fold` 하나로 묶지 않아도 됩니다(2.2절).
- `max_by_key`는 같은 최댓값 중 마지막 원소를 돌려줍니다. 반복문의 규칙(먼저 나온 것 유지)을 그대로 옮기려면 `reduce(|best, v| if v.1 > best.1 { v } else { best })`를 씁니다.
- `min_by_key`는 같은 최솟값 중 첫 원소를 돌려주므로 `min_by_key(|&&(_, c)| std::cmp::Reverse(c))`로도 같은 답이 나옵니다.

</details>

### 연습 3. 장애물 좌표 읽기, 두 가지 요구

게임 맵 파일의 각 줄은 장애물 좌표 `"x,y"`입니다. 다음 도우미가 주어집니다.

```rust
fn parse_point(line: &str) -> Option<(i32, i32)> {
    let (x, y) = line.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}
```

반복문 없이 두 함수를 작성하세요. `load_strict`는 맵을 불러올 때 쓰므로 한 줄이라도 틀리면 첫 번째로 틀린 줄을 알려 주며 실패합니다. `load_lenient`는 편집기에서 쓰므로 맞는 좌표는 모두 읽고, 틀린 줄의 번호(1부터)를 모아 돌려줍니다.

```rust
fn load_strict(lines: &[&str]) -> Result<Vec<(i32, i32)>, String>
fn load_lenient(lines: &[&str]) -> (Vec<(i32, i32)>, Vec<usize>)
```

```text
입력: ["3,4", " 10 , -2", "x,1", "7", "0,0"]
load_strict  → Err("3번째 줄: x,1")
load_lenient → ([(3, 4), (10, -2), (0, 0)], [3, 4])
입력: ["1,2", "3,4"]
load_strict  → Ok([(1, 2), (3, 4)])
```

<details>
<summary>힌트</summary>

- 줄 번호가 필요하므로 두 함수 모두 `enumerate()`로 시작합니다.
- `load_strict`: `parse_point(line).ok_or_else(|| format!("{}번째 줄: {line}", i + 1))`로 원소마다 `Result`를 만들고 `collect`합니다. 반환 타입이 결과 타입을 정합니다(2.4절의 1번).
- `load_lenient`: 원소마다 `Result<(i32, i32), usize>`(에러 자리에 줄 번호)를 만들어 `partition(Result::is_ok)`한 뒤 예제 04처럼 벗깁니다. 반복문 + `match`로 써도 좋다고 했지만, 이 연습에서는 체인으로 써 보고 두 판을 비교해 보세요.

</details>

### 연습 4. 어디서 멈출까

도서 반납 처리입니다. 연체된 책을 만나면 처리를 멈추고 그 제목을 돌려주려고 했는데, 테스트가 실패합니다.

```rust
fn process_returns(
    books: &[(&'static str, bool)], // (제목, 연체 여부)
    shelf: &mut Vec<&'static str>,
) -> Option<&'static str> {
    books.iter().for_each(|&(title, overdue)| {
        if overdue {
            return;
        }
        shelf.push(title);
    });
    None
}
```

입력 `[("어린 왕자", false), ("데미안", true), ("모모", false)]`에 대해 (1) 실행 결과와 `shelf`를 먼저 예상하고, (2) 의도대로 `Some("데미안")`과 `["어린 왕자"]`가 나오게 두 가지로 고치세요. 하나는 `for` 반복문, 하나는 반복자 메서드를 쓴 판입니다. 2.5절의 기준으로 어느 쪽을 남길지 정하세요.

<details>
<summary>힌트</summary>

- (1)은 `None`과 `["어린 왕자", "모모"]`입니다. `return`이 클로저만 끝내므로 `데미안`을 건너뛰고 `모모`까지 처리합니다.
- `for`판은 `if overdue { return Some(title); }`입니다.
- 반복자판은 `try_for_each`의 클로저에서 연체된 책을 `Err(title)`로 돌려주고, 결과 `Result<(), &str>`에 `.err()`를 붙여 `Option<&str>`로 바꿉니다. 결과는 같지만, 본문이 `shelf`에 기록하는 일이므로 `for`판이 더 직접적입니다.

</details>

## 6. 참고 자료

- *Effective Rust* (David Drysdale): [Item 9: Consider using iterator transforms instead of explicit loops](https://effective-rust.com/iterators.html). 이 챕터가 다룬 지침의 범위를 정한 기준 문서입니다. 반복문을 반복자로 바꾸는 단계, `Result` 값을 모으는 방법, 명시적 반복문이 나은 경우를 다룹니다. 책은 인덱스 반복문의 경계 검사에 관해 Sergey Davidoff의 글 [How to avoid bounds checks in Rust (without unsafe!)](https://shnatsel.medium.com/how-to-avoid-bounds-checks-in-rust-without-unsafe-f65e618b4c1e)를 인용합니다. 성능을 따지는 이야기는 [`ch20-avoid-over-optimizing`](../ch20-avoid-over-optimizing/README.md)에서 이어집니다.
- 본편의 관련 챕터: Part 0 [ch13](../../part0-getting-started/ch13-closures-iterators/README.md)(어댑터와 소비 메서드, 반복문을 체인으로 바꾼 첫 예, 비용 없는 추상화), [ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md)(`let...else`). Part 1 [ch5](../../part1-standard-lib/ch5-iterators/README.md)(`IntoIterator`를 받는 함수, `Result`로 `collect`하는 원리, `partition`, 어댑터·소비 메서드 선택표, `scan`, `try_fold`/`try_for_each`, `impl Iterator` 반환), [ch2](../../part1-standard-lib/ch2-option-result/README.md)(클로저 안의 `?`, `Result<Vec<_>, _>`로 모으기).
- 부록의 관련 챕터: [ch2](../ch2-express-common-behavior/README.md)(가장 느슨한 바운드), [ch3](../ch3-option-result-transforms/README.md)(변환 메서드에서 어디서 멈출까), [ch8](../ch8-references-pointers/README.md)(읽기만 하는 매개변수는 슬라이스로). 직접 만든 컬렉션에 `FromIterator`를 구현할지는 [`ch10-standard-traits`](../ch10-standard-traits/README.md)에서 다룹니다.
- 표준 라이브러리 문서: [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)(`max_by_key`의 같은 값 처리, `try_fold`, `try_for_each`, nightly 전용인 `try_find`), [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)(함수 인자를 `IntoIterator`로 받는 예), [`ControlFlow`](https://doc.rust-lang.org/std/ops/enum.ControlFlow.html)(`try_for_each`에서 일찍 멈추는 예).
- Clippy 린트 목록: [rust-1.100.0 판](https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html). 이 챕터에 나온 `needless_range_loop`, `unnecessary_fold`, `manual_find`, `explicit_counter_loop`, `manual_try_fold`, `map_flatten`, `needless_for_each`의 설명과 기본 수준을 찾아볼 수 있습니다.
