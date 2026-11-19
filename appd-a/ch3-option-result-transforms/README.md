# ch3. `match` 대신 변환 메서드로: 어디까지 바꾸고 어디서 멈출까

[Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md)에서 `Option`과 `Result`의 메서드를 "무엇을 하고 싶은가"로 묶어 익혔습니다. 그렇다고 이미 있는 `match`를 언제, 어디까지 바꿔야 하는지까지 정해지지는 않습니다. 이 챕터는 메서드 사용법을 다시 설명하지 않습니다. 같은 함수를 `match` → `if let`·`let...else` → `?` → 변환 메서드 순서로 한 단계씩 바꿔 보면서, 단계마다 무엇이 나아지는지와 어느 단계에서 멈춰야 더 읽기 좋은지를 판단하는 기준을 다룹니다. 그 과정에서 함께 마주치는 세 가지 문제도 봅니다. `Option`과 `Result` 중 무엇을 돌려줄지, 에러 타입을 맞춰 `?`에 넘기는 방법, 그리고 참조 뒤의 값을 옮기지 않고 변환하는 방법입니다.

- 원제: [Item 3: Prefer Option and Result transforms over explicit match expressions](https://effective-rust.com/transform.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch3-option-result-transforms`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `Option`/`Result`를 다루는 `match`를 `if let`·`let...else`·`?`·변환 메서드 체인으로 한 단계씩 바꾸면서 단계마다 무엇이 줄어드는지 설명할 수 있다. 중간 값 여러 개를 함께 쓰는 함수는 `?` 단계에서 멈추는 것이 낫다고 판단할 수 있다.
2. 실패 이유를 알려야 하는지를 기준으로 반환 타입을 `Option`과 `Result` 중에서 고를 수 있다. `ok_or`와 `ok()`로 둘 사이를 바꿔 `?`를 쓸 수 있는 모양을 만들 수 있다.
3. 에러 갈래마다 `return Err(..)`를 적은 `match`를 `map_err(..)?`나 `From`에 기대는 `?`로 바꿀 수 있다. 위치 같은 맥락을 붙여야 할 때 `From`이 아니라 `map_err`를 쓰는 이유를 설명할 수 있다.
4. 참조 뒤의 `Option`에 변환 메서드나 `?`를 쓸 때 `as_ref`·`as_deref`가 필요한 이유를 설명할 수 있다. 컴파일러가 제안하는 `clone()` 대신 이 메서드로 고칠 수 있다.
5. 부수 효과만 낼 때, 갈래마다 하는 일이 클 때, 에러 종류에 따라 일부만 처리할 때는 변환 메서드보다 `if let`이나 `match`가 낫다는 것을 설명하고 골라 쓸 수 있다.

## 2. 핵심 개념

2.1~2.5절은 지침 하나씩을 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 메서드의 시그니처와 동작이 궁금하면 Part 1 ch2의 2.12절 선택표를 함께 보세요.

### 2.1 `match`를 한 단계씩 바꾸기

**문제 상황.** `Option`을 돌려주는 단계가 여럿 이어지는 함수를 `match`로 쓰면 단계마다 `None => None` 갈래가 붙고 코드가 한 겹씩 깊어집니다. 그러면 함수가 정말 하려는 일이 갈래 사이에 흩어집니다. 예제 01의 함수가 하려는 일은 "이름으로 학생을 찾아 점수를 등급으로 바꾼다"입니다.

**비권장 코드.** 예제 01의 0단계입니다.

```rust
fn grade_of_match(roster: &[Student], name: &str) -> Option<char> {
    match find(roster, name) {
        Some(student) => match student.score {
            Some(score) => Some(grade(score)),
            None => None,
        },
        None => None,
    }
}
```

**권장 코드로 가는 길.** 같은 함수를 네 번 고쳐 씁니다. 다섯 판은 모든 입력에서 같은 값을 돌려줍니다(예제 01의 1번 구역).

| 단계 | 핵심 모양 | 줄어든 것 | 남은 문제 | clippy (기본 설정) |
| --- | --- | --- | --- | --- |
| 0. `match` | 위 코드 | — | 단계마다 실패 갈래, 중첩 | 안쪽 `match`에만 `manual_map` |
| 1. `if let` | `if let Some(student) = .. { if let Some(score) = .. { return Some(..); } } None` | `None` 갈래 | 중첩, 실패 값 `None`이 함수 끝으로 멀어짐 | `collapsible_if` |
| 2. `let...else` | `let Some(student) = .. else { return None; };` | 중첩 | `else { return None; }`의 반복 | `question_mark` |
| 3. `?` | `let student = find(roster, name)?;` | 실패 처리가 한 글자로 | 중간 이름(`student`, `score`) | 없음 |
| 4. 체인 | `find(roster, name).and_then(\|s\| s.score).map(grade)` | 중간 이름 | — | 없음 |

단계마다 줄어드는 것은 **실패를 처리하는 코드를 적는 횟수**입니다. 0단계는 실패 갈래를 단계마다 하나씩 적습니다. 1단계의 `if let`([Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md) 2.10절)은 관심 있는 경우만 적으므로 `None` 갈래는 사라집니다. 하지만 중첩은 남고, 실패했을 때 돌려주는 `None`이 실패가 일어난 자리에서 멀리 떨어진 함수 끝으로 갑니다. clippy는 두 `if`를 let 체인([Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.2절) 하나로 합치라고 제안합니다.

```text
warning: this `if` statement can be collapsed
   = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
   |
73 ~     if let Some(student) = find(roster, name)
74 ~         && let Some(score) = student.score {
```

2단계의 `let...else`(Part 0 ch6 2.11절)는 중첩을 풀어 행복한 경로를 왼쪽에 한 줄로 세웁니다. 그런데 `else` 블록이 하는 일이 "`None`을 돌려주며 끝내기"뿐이라면 그것은 정확히 `?`가 하는 일입니다. clippy도 그렇게 알려 줍니다.

```text
warning: this `let...else` may be rewritten with the `?` operator
85 | /     let Some(student) = find(roster, name) else {
86 | |         return None;
87 | |     };
   | |______^ help: replace it with: `let student = find(roster, name)?;`
   = note: `#[warn(clippy::question_mark)]` on by default
```

3단계에서 4단계로 넘어가면 `student`와 `score`라는 이름이 사라집니다. 이 이름들은 값을 다음 단계로 넘기는 데만 쓰였기 때문입니다. 체인에는 단계의 성격을 알려 주는 메서드 이름만 남습니다. 다시 실패할 수 있는 단계는 `and_then`, 실패하지 않는 단계는 `map`입니다.

clippy가 모든 단계를 안내하지는 않는다는 점도 알아 두세요. 0단계에서 `manual_map`의 제안대로 안쪽을 `student.score.map(grade)`로 바꾸면, 바깥 `match`(`Some`이면 `Option`을 돌려주는 식, `None`이면 `None`)가 남습니다. 이것은 `and_then`과 같은 모양이지만 기본 린트는 잡지 않습니다.

**지침을 따르지 않아도 되는 경우: 중간 값을 함께 쓰면 `?`에서 멈춥니다.** 예제 01의 2번 구역은 "학생 점수와 짝 학생 점수의 차이"를 구합니다. 마지막 계산에 두 점수가 모두 필요합니다.

```rust
// ?판
let student = find(roster, name)?;
let my_score = student.score?;
let buddy = find(roster, student.buddy?)?;
let buddy_score = buddy.score?;
Some(my_score.abs_diff(buddy_score))

// 체인판
find(roster, name).and_then(|student| {
    student.score.and_then(|my_score| {
        student
            .buddy
            .and_then(|buddy| find(roster, buddy))
            .and_then(|buddy| buddy.score)
            .map(|buddy_score| my_score.abs_diff(buddy_score))
    })
})
```

체인은 값 하나만 다음 단계로 넘깁니다. `my_score`를 마지막까지 쓰려면 뒤 단계 전체가 `my_score`를 캡처한 클로저 안으로 들어가야 하므로, 0단계에서 없앤 중첩이 클로저 모양으로 되돌아옵니다. 이럴 때는 값마다 이름을 붙여 위에서 아래로 읽히는 `?`판이 낫습니다([Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md) 2.10절의 표와 같은 기준입니다). 비슷하게, `else` 블록이 `return None` 말고 다른 일(기록 남기기, `continue`)도 한다면 `let...else`에서 멈춥니다. 이 경우는 `?`로 바꿀 수 없고 clippy도 제안하지 않습니다.

### 2.2 실패 이유를 알려야 하면 `Result`

**문제 상황.** 좌석 코드 `"B12"`를 읽는 함수가 실패하는 이유는 넷입니다. 비어 있을 수도, 열 문자가 틀렸을 수도, 번호가 숫자가 아닐 수도, 범위를 벗어났을 수도 있습니다. 이 함수가 `Option`을 돌려주면 네 이유가 모두 `None` 하나로 뭉개집니다. 호출한 쪽은 사용자에게 "좌석 코드가 잘못되었습니다"라는 말밖에 할 수 없습니다.

**비권장 코드.**

```rust
fn parse_seat_opt(code: &str) -> Option<Seat> {
    let mut chars = code.chars();
    let row = chars.next().filter(|c| ('A'..='F').contains(c))?;
    let number = chars.as_str().parse::<u32>().ok().filter(|n| (1..=30).contains(n))?;
    Some(Seat { row, number })
}
```

**권장 코드.** 실패한 자리마다 이유를 담는 `Result`를 돌려줍니다. `chars().next()`처럼 `Option`을 돌려주는 단계는 `ok_or`로 `Result`로 바꿔야 `?`가 `SeatError`를 돌려줄 수 있습니다.

```rust
fn parse_seat(code: &str) -> Result<Seat, SeatError> {
    let mut chars = code.chars();
    let row = chars.next().ok_or(SeatError::Empty)?;
    if !('A'..='F').contains(&row) {
        return Err(SeatError::BadRow(row));
    }
    let rest = chars.as_str();
    let number: u32 = rest
        .parse()
        .map_err(|_| SeatError::BadNumber(rest.to_string()))?;
    // ... 범위 검사 후 Ok(Seat { row, number })
}
```

`SeatError::Empty`처럼 이미 만들어진 값은 `ok_or`로 넘기고, `to_string()`처럼 할당이 필요한 에러 값은 클로저를 받는 메서드로 실패할 때만 만듭니다(Part 1 ch2 2.5절). 이제 호출한 쪽은 이유마다 다른 안내를 할 수 있습니다. 예제 02의 1번 구역에서 `"Z3"`에 대해 `Option`판은 "좌석 코드가 잘못되었습니다", `Result`판은 "Z열은 없습니다. A~F열 중에서 고르세요"를 출력합니다.

`Result`를 고를 이유가 하나 더 있습니다. `Result`에는 `#[must_use]`가 붙어 있어서 결과를 버리면 컴파일러가 경고합니다. `Option`에는 붙어 있지 않으므로 `parse_seat_opt("Z3");`처럼 실패를 버려도 아무 말이 없습니다. 예제 02 끝의 `#[expect(unused_must_use, ..)]`를 지우고 빌드하면 다음 경고를 볼 수 있습니다.

```text
warning: unused `Result` that must be used
    |
181 |     parse_seat("Z3");
    |     ^^^^^^^^^^^^^^^^
    |
    = note: this `Result` may be an `Err` variant, which should be handled
    = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
help: use `let _ = ...` to ignore the resulting value
```

**지침을 따르지 않아도 되는 경우: 없는 것이 정상이면 `Option`.** "B13 좌석의 예약자"를 조회했는데 예약자가 없다면 그것은 에러가 아니라 빈 좌석이라는 정상적인 상태입니다. 그래서 조회 함수 `occupant`는 `Option<&str>`을 돌려줍니다. 그런데 같은 조회를 "예약 취소" 문맥에서 쓰면, 예약이 없다는 것은 이제 에러입니다. **무엇이 에러인지는 함수 하나가 아니라 그 값을 쓰는 문맥이 정합니다.** 그러므로 조회 함수는 그대로 두고, 그것을 에러로 보는 쪽이 경계에서 바꿉니다.

```rust
fn cancel(bookings: &[(Seat, &str)], code: &str) -> Result<String, SeatError> {
    let seat = parse_seat(code)?;
    let name = occupant(bookings, seat).ok_or(SeatError::NotBooked(seat))?;
    Ok(format!("{seat} 좌석의 {name} 예약을 취소합니다"))
}
```

반대 방향도 있습니다. 신청서의 희망 좌석은 적지 않아도 되고, 잘못 적었으면 무시하고 자동 배정한다고 합시다. 이 함수를 부르는 쪽은 이유를 쓰지 않으므로 `Option`이 맞습니다. 이때는 `parse_seat(wish.trim()).ok()`로 이유를 그 자리에서 버립니다. 처음부터 `Option`만 돌려주는 `parse_seat_opt`와는 다릅니다. 이유를 버리기로 한 결정이 한 곳에 드러나고, 이유가 필요한 다른 호출자는 여전히 `parse_seat`를 쓸 수 있습니다.

### 2.3 에러 타입을 맞춰 `?`에 넘기기

**문제 상황.** 세트 기록 `"12 x 40.5"`(횟수 x 무게)를 읽는 함수는 `Option`(`split_once`), `Result<_, ParseIntError>`(횟수), `Result<_, ParseFloatError>`(무게)를 차례로 만납니다. 이 함수의 에러 타입은 `SetError` 하나이므로 각 실패를 `SetError`로 바꿔 돌려줘야 합니다.

**비권장 코드.** 예제 03의 (가)판입니다. 실패 갈래가 단계마다 세 줄씩 들어가서, 정작 계산하는 마지막 줄보다 길어집니다.

```rust
let reps: u32 = match reps.trim().parse() {
    Ok(n) => n,
    Err(e) => return Err(SetError::BadReps(e)),
};
```

이 모양은 clippy가 알려 주지 않으므로 직접 알아봐야 합니다. 기본으로 켜진 `question_mark` 린트가 잡는 갈래와 잡지 않는 갈래는 다음과 같습니다.

| 에러 갈래의 모양 | `question_mark` |
| --- | --- |
| `Err(e) => return Err(e)` | 잡음 |
| `Err(e) => return Err(e.into())` | 잡음 |
| `Err(e) => return Err(From::from(e))` | 잡지 않음 |
| `Err(e) => return Err(SetError::BadReps(e))` | 잡지 않음 |
| `None => return Err(SetError::NoSeparator)` | 잡지 않음 |

셋째 줄은 `?`가 실제로 하는 일([Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 2.3절)과 같은데도 잡히지 않는다는 점이 눈에 띕니다.

**권장 코드.** 두 가지가 있습니다. (나)판은 `map_err`로 에러 타입을 맞춘 뒤 `?`를 붙입니다. `return`과 갈래가 사라지고 "어떤 배리언트로 감싸는가"만 남습니다.

```rust
let (reps, weight) = line.split_once('x').ok_or(SetError::NoSeparator)?;
let reps: u32 = reps.trim().parse().map_err(SetError::BadReps)?;
let weight: f64 = weight.trim().parse().map_err(SetError::BadWeight)?;
```

(다)판은 `impl From<ParseIntError> for SetError`와 `impl From<ParseFloatError> for SetError`를 한 번 적어 두고 `?`만 붙입니다. `?`는 `Err`를 돌려주기 전에 `From::from`을 불러 에러를 바꿔 줍니다.

```rust
let reps: u32 = reps.trim().parse()?; // ParseIntError → SetError::BadReps
let weight: f64 = weight.trim().parse()?; // ParseFloatError → SetError::BadWeight
```

세 판의 결과는 모든 입력에서 같습니다(예제 03의 1번 구역). 둘 중 무엇을 고를지는 [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 2.4절의 표를 따릅니다. 원인 타입 하나가 배리언트 하나에 대응하고 어디서 났는지가 중요하지 않으면 `From`을, 그렇지 않으면 `map_err`를 씁니다. 에러 타입을 어떻게 설계할지는 `ch4-idiomatic-errors`, `From`/`Into` 변환을 언제 구현할지는 `ch5-type-conversions`에서 다룹니다.

**지침을 따르지 않아도 되는 경우: `From`이 아니라 `map_err`를 써야 할 때.** `From::from`은 에러 값 하나만 받으므로 "몇 번째 줄이었는가" 같은 맥락을 알 수 없습니다. 그런 맥락은 그 값을 아는 자리에서 `map_err`의 클로저로 붙입니다.

```rust
for (i, line) in log.lines().enumerate() {
    total += volume_from(line).map_err(|source| LogError { line_no: i + 1, source })?;
}
```

에러를 전파하지 않고 그 자리에서 처리하는 경우라면 `?` 자체가 맞지 않습니다. 이 경우는 2.5절에서 봅니다.

### 2.4 참조 뒤의 값은 `as_ref`로 빌려서 변환하기

**문제 상황.** `match &self.tracking_no { Some(no) => .., None => .. }`는 참조를 조건식으로 받으므로 기본 바인딩 모드(Part 0 ch18 2.6절)에 따라 `no`가 저절로 `&String`으로 바인딩됩니다. 이 `match`를 변환 메서드로 바꾸면서 `&`를 그냥 지우면 문제가 생깁니다. `map`, `unwrap_or` 같은 변환 메서드와 `?`는 `Option`을 `self`로, 즉 통째로 가져갑니다. 그런데 `&self`로 빌린 구조체의 필드는 가져갈 수 없으므로 E0507이 납니다(4.1절).

**비권장 코드.** 이때 컴파일러가 제안하는 고치는 방법이 `clone()`인 경우가 많습니다. 제안대로 하면 컴파일은 되지만 부를 때마다 문자열을 새로 할당해 복사합니다.

```rust
fn label_clone(&self) -> String {
    self.tracking_no.clone().unwrap_or_else(|| String::from("발송 전"))
}
```

**권장 코드.** `as_ref()`는 `&Option<String>`을 `Option<&String>`으로, `as_deref()`는 `Option<&str>`로 바꿉니다. 변환 메서드가 가져가는 것은 참조를 담은 이 새 `Option`이므로 필드는 그대로 남습니다.

```rust
fn label(&self) -> &str {
    self.tracking_no.as_deref().unwrap_or("발송 전")
}

fn carrier(&self) -> Option<&str> {
    let no = self.tracking_no.as_deref()?; // ? 앞에도 같은 규칙
    no.get(..2)
}
```

예제 04의 2번 구역은 두 판이 돌려준 문자열이 필드와 같은 버퍼를 가리키는지 비교합니다. `clone`판은 `false`, `as_deref`판은 `true`입니다. clippy는 이 변환을 알고 있어서, `match &self.tracking_no`를 `map`으로 바꾸라고 할 때 제안하는 코드에 `as_ref()`를 넣어 줍니다.

```text
warning: manual implementation of `Option::map`
18 | /         match &self.tracking_no {
19 | |             Some(no) => Some(no.chars().count()),
20 | |             None => None,
21 | |         }
   | |_________^ help: try: `self.tracking_no.as_ref().map(|no| no.chars().count())`
```

컴파일러의 E0507 `help:`는 쓴 코드에 따라 다릅니다. 저장소 툴체인에서 확인한 결과는 다음과 같습니다. `clone()`만 제안받았다면 먼저 `as_ref()`·`as_deref()`로 풀리는지 생각해 보세요.

| 빌린 필드에 쓴 것 | `help:`가 제안하는 것 |
| --- | --- |
| `self.tracking_no.map_or(..)` (클로저를 받는 메서드) | `as_ref()`, `as_mut()`, `clone()` |
| `self.tracking_no.unwrap_or(..)` | `clone()`만 |
| `self.tracking_no?` | `clone()`만 |

**지침을 따르지 않아도 되는 경우.** 두 가지입니다.

- 안의 타입이 `Copy`면 `Option`도 `Copy`입니다. `Option<u32>` 필드에 `map`을 부르면 복사본이 넘어가므로 `as_ref`가 필요 없습니다(예제 04의 `weight_kg`).
- `self`를 받는(소비하는) 메서드는 필드를 가져가도 됩니다. 여기서 `as_deref()`를 쓰면 오히려 `to_string()`으로 따로 복사해야 합니다(예제 04의 `into_label`).

### 2.5 변환 메서드보다 `if let`·`match`가 나은 곳

**문제 상황.** 변환 메서드는 "값을 받아 값을 돌려주는" 짧은 단계를 이을 때 가장 읽기 좋습니다. 이 모양에서 벗어나는 일까지 변환 메서드로 쓰면 오히려 읽기 어려워집니다. 예제 05는 세 경우를 비교합니다.

**비권장 코드와 권장 코드.**

| 경우 | 비권장 | 권장 | 이유 |
| --- | --- | --- | --- |
| 값이 있을 때 부수 효과만 냄 | `coupon.map(\|code\| log.push(..));`, `None => {}` 갈래가 있는 `match` | `if let Some(code) = coupon { .. }` | `map`은 새 `Option`을 만드는 메서드인데 그 결과를 버립니다. 빈 갈래는 읽을 거리만 늘립니다. |
| 성공·실패 갈래가 각각 여러 줄이고 하는 일이 다름 | `result.map_or_else(\|e\| { .. }, \|amount\| { .. })` | `match result { Ok(amount) => { .. } Err(e) => { .. } }` | `map_or_else`는 실패 클로저를 먼저 받습니다. 블록이 길면 어느 것이 어느 경우인지 인자 순서를 기억해야 합니다. |
| 에러 종류에 따라 일부는 처리하고 일부는 전파 | `.or_else(\|e\| match e.kind() { .. })` | 매치 가드를 쓴 `match` | 클로저 안에 `match`가 들어갔다면 이미 `match`를 쓰면서 한 겹 더 깊어진 것입니다. |

첫 번째 경우는 clippy가 두 비권장 모양을 모두 잡아 `if let`을 제안합니다.

```text
warning: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`
   = note: `#[warn(clippy::single_match)]` on by default

warning: called `map(f)` on an `Option` value where `f` is a closure that returns the unit type `()`
   = note: `#[warn(clippy::option_map_unit_fn)]` on by default
help: use `if let` instead
```

세 번째 경우의 권장 코드는 결과 네 가지를 같은 깊이에 나란히 놓습니다. 매치 가드는 [Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.8절에서 배웠습니다.

```rust
match text.trim().parse::<u8>() {
    Ok(n) => Ok(n),
    Err(e) if *e.kind() == IntErrorKind::Empty => Ok(1),
    Err(e) if *e.kind() == IntErrorKind::PosOverflow => Ok(u8::MAX),
    Err(e) => Err(e),
}
```

예제에는 없지만 하나 더 있습니다. **바깥의 흐름을 바꿔야 할 때**입니다. 클로저 안의 `return`이나 `?`는 클로저에서만 빠져나가고, 클로저 안에서는 바깥 반복문을 `continue`·`break`할 수도 없습니다(Part 1 ch2 2.10·2.11절). 반복문 안에서 값이 없으면 다음 항목으로 넘어가는 코드는 `let ... else { continue; };`([Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md) 2.11절)로 씁니다.

**지침을 따르지 않아도 되는 경우: 갈래가 짧으면 변환 메서드도 좋습니다.** `code.map_or(0, |c| c.len())`, `age.is_some_and(|a| a >= 19)`처럼 두 갈래가 각각 짧은 식 하나라면 `match`보다 변환 메서드 한 줄이 낫습니다. 기준은 메서드의 종류가 아니라 갈래의 크기와 갈래마다 하는 일의 성격입니다.

### 2.6 고르는 순서

새로 쓰거나 고칠 코드 앞에서 다음 순서로 물어봅니다.

1. **함수가 `Option`/`Result`를 돌려주고, 실패를 그대로 넘기는가?** 그렇다면 `?`를 씁니다. 만난 값의 타입이 함수의 반환 타입과 다르면 `?` 앞에서 맞춥니다. `Option`→`Result`는 `ok_or`/`ok_or_else`, `Result`→`Option`은 `ok()`, 에러 타입이 다르면 `map_err`나 `From`입니다.
2. **값 하나가 단계를 거쳐 흘러가는가?** 그렇다면 체인으로 씁니다. 다시 실패할 수 있는 단계는 `and_then`, 아닌 단계는 `map`, 걸러 내는 단계는 `filter`입니다. 끝에서 값으로 꺼낼 때는 `unwrap_or` 계열을 씁니다.
3. **중간 값 여러 개를 마지막에 함께 쓰는가?** 그렇다면 `?`에서 멈춥니다.
4. **있을 때만 할 일이 있고 없으면 아무것도 안 하는가?** 그렇다면 `if let`입니다. 없을 때 빠져나가되 `return None` 말고 다른 일도 한다면 `let...else`입니다.
5. **갈래마다 하는 일이 크거나, 에러 종류마다 다르게 처리하는가?** 그렇다면 `match`입니다.
6. **값이 빌린 것인가?** 그렇다면 1~2번을 적용하기 전에 `as_ref()`/`as_deref()`를 먼저 붙입니다.

### 2.7 책과 달라진 점

책은 Rust 1.70과 2018 에디션을 기준으로 합니다. 저장소 툴체인(1.100.0, 2024 에디션)에서 확인한 차이는 다음과 같습니다.

- **E0507의 `help:`.** 책이 보여 주는 E0507 출력에는 `help:` 줄이 없습니다. 지금 컴파일러는 고치는 방법을 제안하지만, 2.4절의 표처럼 `unwrap_or`나 `?`에는 `clone()`만 제안합니다. 컴파일러의 제안을 그대로 따르면 책이 권하는 `as_ref()` 대신 복사하는 코드가 됩니다.
- **겹친 `if let`.** 2024 에디션에서는 let 체인을 쓸 수 있고, clippy의 `collapsible_if`가 겹친 `if let`을 let 체인으로 합치라고 제안합니다(2.1절). 이 제안을 따라도 `return`과 함수 끝의 `None`은 남으므로 `?`나 체인이 여전히 더 짧습니다.
- **책에 없는 지침.** 2.5절(변환 메서드보다 `if let`·`match`가 나은 곳)은 책의 Item 3이 명시적으로 다루는 내용이 아닙니다(2026-11-19에 온라인판에서 확인). 책은 명시적 `match`보다 변환 메서드를 우선하라고 권하고, 어디서 멈출지는 이 챕터가 "지침을 따르지 않아도 되는 경우"로 더한 판단입니다.

### 2.8 기억할 것

- `match` → `if let`·`let...else` → `?` → 체인으로 갈수록 실패 처리를 적는 횟수가 줄어듭니다. 값 하나가 흘러가면 체인까지 가고, 중간 값을 함께 쓰면 `?`에서 멈춥니다.
- 실패 이유가 있으면 `Result`, 없는 것이 정상이면 `Option`을 돌려줍니다. 무엇이 에러인지는 문맥이 정하므로 경계에서 `ok_or`·`ok()`로 바꿉니다.
- `?` 앞에서 타입을 맞춥니다. `ok_or`, `ok()`, `map_err`를 쓰거나 `From`을 구현합니다. 위치 같은 맥락은 `map_err`로만 붙일 수 있습니다.
- `match &x`를 변환 메서드로 바꿀 때 `&`는 `as_ref()`/`as_deref()`가 됩니다. 컴파일러의 `clone()` 제안을 따르기 전에 먼저 이것을 떠올립니다.
- 부수 효과는 `if let`으로, 큰 갈래와 에러 종류별 처리는 `match`로 씁니다. clippy가 일부 모양을 알려 주지만 전부 알려 주지는 않습니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_refactor_steps.rs`](examples/01_refactor_steps.rs) | 같은 함수의 다섯 판(`match`, `if let`, `let...else`, `?`, 체인), 중간 값을 함께 쓰는 함수의 `?`판과 체인판 | 1 |
| [`examples/02_option_or_result.rs`](examples/02_option_or_result.rs) | 이유를 버리는 `Option`과 이유를 담는 `Result`, 문맥에 따른 `ok_or`·`ok()`, `#[must_use]` | 2 |
| [`examples/03_error_mapping.rs`](examples/03_error_mapping.rs) | 에러 갈래를 손으로 쓴 `match`, `map_err(..)?`, `From` + `?`, `map_err`로 위치 붙이기 | 3 |
| [`examples/04_borrowed_transforms.rs`](examples/04_borrowed_transforms.rs) | `match &field`와 `as_ref`, `clone` 대신 `as_deref`, `?` 앞의 `as_deref`, `as_ref`가 필요 없는 경우 | 4 |
| [`examples/05_where_to_stop.rs`](examples/05_where_to_stop.rs) | 부수 효과는 `if let`, 큰 갈래는 `match`, 일부 에러만 처리할 때는 매치 가드 | 5 |
| [`compile_fail/`](compile_fail) | `?`를 쓰다 만나는 컴파일 에러 세 가지 | 2, 3, 4 |

예제 명령은 저장소 루트에서 실행한다고 보고 썼습니다. 어느 예제도 패닉하지 않으며 모두 종료 코드 0으로 끝납니다. 비권장 판 중 린트가 잡는 것에는 `#[expect(clippy::…, reason = "…")]`(예제 02는 `#[expect(unused_must_use, ..)]`)를 붙여 두었습니다. 속성을 지우고 `cargo clippy -p er-ch3-option-result-transforms --examples`를 실행하면 경고를 직접 볼 수 있습니다. 2절에 인용한 경고의 행 번호는 그 예제의 `#[expect]`를 모두 지웠을 때의 번호입니다.

### 01_refactor_steps — `match`에서 체인까지

```console
$ cargo run -q -p er-ch3-option-result-transforms --example 01_refactor_steps
== 1. 값 하나가 흘러가는 함수: 다섯 판의 결과 비교 ==
가온 → Some('A') (다섯 판 일치: true)
나래 → None (다섯 판 일치: true)
다솜 → Some('C') (다섯 판 일치: true)
없는이 → None (다섯 판 일치: true)

== 2. 중간 값 두 개를 함께 쓰는 함수: ?판과 체인판 비교 ==
가온 → ?판: Some(14), 체인판: Some(14)
나래 → ?판: None, 체인판: None
다솜 → ?판: None, 체인판: None
라온 → ?판: None, 체인판: None
```

볼 점:

- 1번 구역의 두 `None`은 실패한 단계가 다릅니다. `나래`는 점수가 없고, `없는이`는 명단에 없습니다. 어느 판이든 실패한 단계 뒤는 실행하지 않고 `None`을 돌려줍니다.
- 2번 구역의 세 `None`도 각각 다른 단계에서 실패했습니다. `나래`는 자기 점수가 없고, `다솜`은 짝이 없고, `라온`의 짝 `미르`는 명단에 없습니다. 두 판의 결과는 같고, 차이는 소스 코드의 읽기 쉬움뿐입니다.

### 02_option_or_result — 이유를 전하는 반환 타입 고르기

```console
$ cargo run -q -p er-ch3-option-result-transforms --example 02_option_or_result
== 1. Option은 실패했다는 사실만, Result는 이유까지 전합니다 ==
"B12"
  Option: Some(B12) → B12 좌석을 골랐습니다
  Result: Ok(B12) → B12 좌석을 골랐습니다
""
  Option: None → 좌석 코드가 잘못되었습니다
  Result: Err(Empty) → 좌석 코드를 입력하세요
"Z3"
  Option: None → 좌석 코드가 잘못되었습니다
  Result: Err(BadRow('Z')) → Z열은 없습니다. A~F열 중에서 고르세요
"B"
  Option: None → 좌석 코드가 잘못되었습니다
  Result: Err(BadNumber("")) → 번호 ""를 숫자로 읽을 수 없습니다
"B99"
  Option: None → 좌석 코드가 잘못되었습니다
  Result: Err(OutOfRange(99)) → 99번은 없습니다. 1~30번 중에서 고르세요

== 2. 없는 것이 에러인지는 문맥이 정합니다 ==
조회 B12 → Some("가온")
조회 B13 → None
취소 C3 → C3 좌석의 나래 예약을 취소합니다
취소 C4 → NotBooked(C4): C4 좌석에는 예약이 없습니다
취소 C99 → OutOfRange(99): 99번은 없습니다. 1~30번 중에서 고르세요

== 3. 이유를 쓰지 않는 곳에서는 ok()로 버립니다 ==
"이름=다솜, 희망=A7" → A7
"이름=라온, 희망=맨 앞" → F30
"이름=미르" → F30
```

볼 점:

- 1번 구역에서 `Option`판의 안내는 네 입력 모두 같습니다. 이유마다 안내를 만드는 `advice` 함수는 갈래마다 하는 일이 달라서 `match`로 썼습니다(2.5절).
- 2번 구역에서 `B13`의 `None`은 조회 결과일 뿐이지만, `C4`를 취소할 때는 같은 "없음"이 `NotBooked` 에러가 됩니다. `C99`는 `parse_seat`의 에러가 `?`로 그대로 올라온 것입니다.
- 3번 구역은 희망 좌석이 잘못되었을 때와 아예 없을 때를 구분하지 않고 둘 다 자동 배정(`F30`)합니다. 이 함수에는 그 구분이 필요 없습니다.

### 03_error_mapping — 에러 갈래 줄이기

```console
$ cargo run -q -p er-ch3-option-result-transforms --example 03_error_mapping
== 1. match판, map_err판, From판의 결과 비교 ==
"12 x 40.5" → Ok(486.0) (세 판 일치: true)
"12 40.5" → Err(NoSeparator) (세 판 일치: true)
"열두 x 40.5" → Err(BadReps(ParseIntError { kind: InvalidDigit })) (세 판 일치: true)
"12 x 사십" → Err(BadWeight(ParseFloatError { kind: Invalid })) (세 판 일치: true)

== 2. map_err로 위치를 덧붙이기 ==
합계 1265kg
2번째 줄에서 실패: BadWeight(ParseFloatError { kind: Invalid })
```

볼 점:

- 세 판 모두 횟수를 읽지 못하면 `BadReps`, 무게를 읽지 못하면 `BadWeight`입니다. `From`판에서는 `?`가 원인 에러의 타입(`ParseIntError`인가 `ParseFloatError`인가)을 보고 어느 `from`을 부를지 고릅니다.
- 2번 구역의 둘째 기록은 `42,5`처럼 쉼표를 써서 둘째 줄에서 실패했습니다. 줄 번호는 `SetError`에는 없고 `map_err`가 붙인 `LogError`에만 있습니다.

### 04_borrowed_transforms — 빌린 값 변환하기

```console
$ cargo run -q -p er-ch3-option-result-transforms --example 04_borrowed_transforms
== 1. match는 참조를 받고, 변환 메서드는 as_ref가 필요합니다 ==
발송: match판 Some(12), as_ref판 Some(12)
대기: match판 None, as_ref판 None

== 2. 값을 꺼낼 때: clone() 대신 as_deref() ==
발송: clone판 "CJ2026111901", as_deref판 "CJ2026111901"
대기: clone판 "발송 전", as_deref판 "발송 전"
필드와 같은 버퍼인가: clone판 false, as_deref판 true

== 3. ? 앞에도 as_deref() ==
발송: Some("CJ"), 대기: None

== 4. as_ref가 필요 없는 경우 ==
무게(Copy인 Option<u32>): 발송 Some(1.25), 대기 None
into_label: "CJ2026111901"
```

볼 점:

- 2번 구역의 두 판은 같은 문자열을 출력하지만 `clone`판은 새로 할당한 버퍼입니다. 주소 값은 실행마다 다르므로 출력하지 않고, 필드의 버퍼와 주소가 같은지만 비교했습니다.
- 4번 구역의 `into_label`은 `self`를 소비하므로, 빌려서 쓰는 일이 모두 끝난 뒤 마지막에 불렀습니다.

### 05_where_to_stop — `if let`과 `match`가 나은 곳

```console
$ cargo run -q -p er-ch3-option-result-transforms --example 05_where_to_stop
== 1. 부수 효과만 낼 때: if let ==
Some("WELCOME10") → match판 ["WELCOME10 적용"], map판 ["WELCOME10 적용"], if let판 ["WELCOME10 적용"]
None → match판 [], map판 [], if let판 []

== 2. 갈래마다 하는 일이 다를 때: match ==
결제 42000원, 적립 420점, 무료 배송 (두 판 일치: true)
결제 9900원, 적립 99점, 배송비 3000원 (두 판 일치: true)
결제 실패(Declined): 다른 카드를 쓰세요 (두 판 일치: true)
결제 실패(Timeout): 잠시 뒤 다시 시도하세요 (두 판 일치: true)

== 3. 일부 에러만 처리할 때: match + 매치 가드 ==
"3" → Ok(3) (두 판 일치: true)
"" → Ok(1) (두 판 일치: true)
"300" → Ok(255) (두 판 일치: true)
"-2" → Err(ParseIntError { kind: InvalidDigit }) (두 판 일치: true)
"셋" → Err(ParseIntError { kind: InvalidDigit }) (두 판 일치: true)
```

볼 점:

- 모든 구역에서 판끼리 결과가 같습니다. 이 예제의 비교 기준은 결과가 아니라 소스 코드를 읽을 때 드는 수고입니다. 두 판의 함수를 나란히 놓고 읽어 보세요.
- 3번 구역의 `"-2"`는 `u8`로 읽으므로 음수 부호부터 숫자가 아닌 문자로 취급되어 `InvalidDigit`입니다. `"300"`은 `u8`의 범위를 넘어 `PosOverflow`이므로 최대값으로 바뀌었습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd appd-a/ch3-option-result-transforms
rustc --edition 2024 compile_fail/e0507_question_on_borrowed_option.rs --out-dir /tmp/rust-er-ch3
```

아래 인용은 핵심 줄만 옮긴 것입니다. 표준 라이브러리 소스 위치를 가리키는 `--> /rustc/…` 줄은 툴체인마다 다르므로 뺐습니다.

### 4.1 빌린 `Option`에 `?` — E0507

파일: [`compile_fail/e0507_question_on_borrowed_option.rs`](compile_fail/e0507_question_on_borrowed_option.rs)

```text
error[E0507]: cannot move out of `self.tracking_no` which is behind a shared reference
  --> compile_fail/e0507_question_on_borrowed_option.rs:24:18
   |
24 |         let no = self.tracking_no?;
   |                  ^^^^^^^^^^^^^^^^-
   |                  |
   |                  `self.tracking_no` moved due to the question mark operator
   |                  move occurs because `self.tracking_no` has type `Option<String>`, which does not implement the `Copy` trait
   |
note: the question mark operator is desugared into a call to `std::ops::Try::branch`, which takes ownership of the receiver `self`, which moves `self.tracking_no`
help: consider cloning the value if the performance cost is acceptable
   |
24 |         let no = self.tracking_no.clone()?;
   |                                  ++++++++
```

- **읽는 법**: `note:`가 원인을 짚습니다. `?`는 `Try::branch`라는 메서드 호출로 바뀌는데, 이 메서드가 `self`를 받으므로 `self.tracking_no`가 이동합니다. 그런데 `self`는 빌린 것(`behind a shared reference`)이라 옮길 수 없습니다.
- **원인**: `match &self.tracking_no { Some(no) => no, None => return None }`를 `?`로 바꾸면서 `&`까지 함께 지웠습니다. `match`는 참조를 조건식으로 받을 수 있지만 `?`는 `Option`을 통째로 가져갑니다. [Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md) 4.1절의 E0507은 `map`이 가져간 경우이고, 이번에는 `?`가 가져갔습니다.
- **해결**: `let no = self.tracking_no.as_deref()?;`처럼 참조를 담은 `Option`에 `?`를 붙입니다(예제 04의 `carrier`). 이번 `help:`는 `as_ref()`를 제안하지 않고 `clone()`만 제안합니다. 읽기만 하는 함수에서 문자열을 복사할 이유는 없습니다.

### 4.2 `Option`을 돌려주는 함수에서 `Result`에 `?` — E0277

파일: [`compile_fail/e0277_result_question_in_option_fn.rs`](compile_fail/e0277_result_question_in_option_fn.rs)

```text
error[E0277]: the `?` operator can only be used on `Option`s, not `Result`s, in a function that returns `Option`
  --> compile_fail/e0277_result_question_in_option_fn.rs:17:37
   |
15 | fn wished_count(form: &str) -> Option<u32> {
   | ------------------------------------------ this function returns an `Option`
16 |     let (_, text) = form.split_once("수량=")?;
17 |     let n: u32 = text.trim().parse()?;
   |                                     ^ use `.ok()?` if you want to discard the `Result<!, _>` error information
```

- **읽는 법**: 함수가 `Option`을 돌려주므로 `?`는 `Option`에만 쓸 수 있다는 뜻입니다. 16행의 `split_once(..)?`는 `Option`이라 통과했고 17행의 `parse()?`만 거부됐습니다. 레이블의 `Result<!, _>`는 `?`가 실패했을 때 떼어 낸 값의 타입입니다. 성공 쪽은 있을 수 없으므로 never 타입 `!`가 들어가 있습니다. 레이블 전체는 "에러 정보를 버리려면 `.ok()?`를 쓰라"는 뜻입니다.
- **원인**: `?`는 에러 타입 사이의 변환(`From`)은 해 주지만 `Result`와 `Option` 사이를 건너가지는 않습니다. [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 4.2절에서 본 것은 반대 방향(`Result` 함수에서 `Option`에 `?`)이었습니다.
- **해결**: 2.2절의 기준으로 고릅니다. 이 함수를 부르는 쪽이 이유를 쓰지 않으면 `text.trim().parse().ok()?`로 이유를 버립니다. 이유를 알려야 하면 함수의 반환 타입을 `Result`로 바꾸고, `split_once(..)?` 쪽을 `ok_or(..)?`로 고칩니다.

### 4.3 `ok_or`에 함수의 에러 타입이 아닌 값 — E0277

파일: [`compile_fail/e0277_ok_or_without_from.rs`](compile_fail/e0277_ok_or_without_from.rs)

```text
error[E0277]: `?` couldn't convert the error to `SetError`
  --> compile_fail/e0277_ok_or_without_from.rs:31:62
   |
30 | fn reps_of(line: &str) -> Result<u32, SetError> {
   |                           --------------------- expected `SetError` because of this
31 |     let (reps, _) = line.split_once('x').ok_or("구분자 x가 없습니다")?;
   |                                          ----------------------------^ the trait `From<&str>` is not implemented for `SetError`
   |                                          |
   |                                          this can't be annotated with `?` because it has type `Result<_, &str>`
   |
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
help: the trait `From<&str>` is not implemented for `SetError`
      but trait `From<ParseIntError>` is implemented for it
```

- **읽는 법**: `ok_or(..)`의 결과가 `Result<_, &str>`이고, `?`가 `&str`을 `SetError`로 바꾸려고 `From<&str>`을 찾았지만 없다는 뜻입니다. `help:`는 `From<ParseIntError>`는 있다고 알려 줍니다. 그래서 32행의 `parse()?`는 문제없이 통과합니다.
- **원인**: `Option`을 `Result`로 바꾸는 것까지는 맞았지만, 에러 값으로 함수의 에러 타입이 아닌 문자열을 넣었습니다.
- **해결**: `ok_or(SetError::NoSeparator)?`처럼 처음부터 함수의 에러 타입 값을 넣습니다(예제 03). `From<&str>`을 구현해서 문자열을 받아 주는 방법도 있지만, 그러면 호출한 쪽은 다시 문장을 읽어야 이유를 알 수 있습니다(2.2절).

### 4.4 그 밖의 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| `From`이 있는데 `?` 앞에 `map_err(\|e\| e.into())`를 붙임 | `parse::<u32>()`처럼 타입을 적었다면 E0283 `type annotations needed`. 쓰는 모양에 따라 E0282 | `?`가 이미 `From::from`을 부르므로 `into()`의 목적 타입을 정할 수 없습니다. `map_err`를 지우고 `?`만 씁니다. |
| `match`를 체인으로 바꾸면서 다시 실패하는 단계에 `map`을 씀 | `Option<Option<_>>`, 반환 타입이 정해져 있으면 E0308 | `and_then`으로 씁니다([Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md) 4.3절). |
| `ok_or(format!(..))`처럼 에러 값을 만드는 데 할당이 듦 | 경고 없음. 성공해도 문자열을 만들었다가 버림 | `ok_or_else(\|\| ..)`나 `map_err(\|_\| ..)`로 실패할 때만 만듭니다(Part 1 ch2 2.5절). |
| `let ... else { return None; };`나 `Err(e) => return Err(e)` | clippy `question_mark` | `?`로 바꿉니다(2.1절, 2.3절). |

## 5. 연습 문제

### 연습 1. 네 겹 `match` 풀기

도서의 ISBN에서 두 번째 부분(`"979-11-6224-123-4"`의 `11`)을 숫자로 읽는 함수입니다. 이 함수를 먼저 `?`판으로, 다음에 체인판으로 바꿔 쓰고 어느 쪽이 더 읽기 좋은지 정하세요. 세 판의 결과는 아래와 같아야 합니다.

```rust
struct Book {
    title: &'static str,
    isbn: Option<&'static str>, // 등록 전이면 None
}

fn group_code(shelf: &[Book], title: &str) -> Option<u32> {
    match shelf.iter().find(|b| b.title == title) {
        Some(book) => match book.isbn {
            Some(isbn) => match isbn.split('-').nth(1) {
                Some(part) => match part.parse::<u32>() {
                    Ok(n) => Some(n),
                    Err(_) => None,
                },
                None => None,
            },
            None => None,
        },
        None => None,
    }
}
```

```text
러스트 입문 (979-11-6224-123-4) → Some(11)
소유권 이야기 (ISBN 없음)       → None
패턴 연습장 (979)               → None
트레이트 산책 (979-가나-11-5)   → None
없는 책                         → None
```

<details>
<summary>힌트</summary>

- 원래 함수를 clippy로 검사하면 가장 안쪽 `match` 하나에만 `manual_ok_err`가 걸리고, 바깥의 세 겹은 아무 경고도 없습니다. 나머지는 2.1절의 단계를 직접 밟아야 합니다.
- 각 단계가 넘기는 값은 하나뿐이므로(책 → ISBN → 부분 문자열 → 숫자) 체인까지 가는 것이 좋습니다. 다시 실패할 수 있는 단계가 셋이므로 `and_then`이 세 번 나옵니다. 마지막 단계의 `Result`는 `ok()`로 `Option`으로 바꿉니다.

</details>

### 연습 2. 이유를 알려 주는 판

연습 1의 함수는 다섯 경우 중 네 경우에 `None`만 돌려줍니다. 실패 이유를 담는 에러 열거형(배리언트 `NoSuchBook`, `NoIsbn`, `NoGroupPart`, `BadGroupPart(ParseIntError)`)을 만들고, `fn group_code(..) -> Result<u32, CodeError>`로 바꾸세요. `match` 없이 `?`와 변환 메서드만 씁니다.

```text
러스트 입문   → Ok(11)
소유권 이야기 → Err(NoIsbn)
패턴 연습장   → Err(NoGroupPart)
트레이트 산책 → Err(BadGroupPart(ParseIntError { kind: InvalidDigit }))
없는 책       → Err(NoSuchBook)
```

<details>
<summary>힌트</summary>

- 앞의 세 단계는 `Option`이므로 `ok_or(CodeError::…)?`, 마지막 단계는 `map_err(CodeError::BadGroupPart)`입니다. 2.6절의 1번 질문에 해당합니다.
- 배리언트가 모두 이미 만들어진 값이라 `ok_or_else`까지 쓸 필요는 없습니다.
- 마지막 줄을 `part.parse().map_err(CodeError::BadGroupPart)`로 끝내면 `Ok(..)`로 다시 감싸지 않아도 됩니다.

</details>

### 연습 3. 빌린 필드에서 도메인 꺼내기

```rust
struct Profile {
    email: Option<String>,
}
```

`fn domain(&self) -> Option<&str>`를 구현하세요. 이메일이 없거나 `@`가 없으면 `None`입니다. `match`를 쓰지 않고, 반환값이 필드 안의 문자열을 가리키게(복사 없이) 만드세요.

```text
Some("ferris@rust-lang.org") → Some("rust-lang.org")
Some("주소없음")             → None
None                         → None
```

<details>
<summary>힌트</summary>

- `self.email?`는 4.1절의 E0507입니다. `self.email.as_deref()?`로 `&str`을 꺼냅니다.
- `str::split_once('@')`는 `Option<(&str, &str)>`을 돌려줍니다. `map(|(_, domain)| domain)`으로 뒷부분만 남깁니다.
- `clone()`이나 `to_string()`을 쓰면 반환 타입이 `Option<String>`이 되어야 하므로 시그니처와 맞지 않습니다.

</details>

### 연습 4. 어디서 멈출까

clippy 경고가 나지 않게 고치되, 이 챕터의 기준으로 어느 단계에서 멈출지 정하세요.

```rust
fn total_price(menu: &[(&str, u32)], order: &str) -> Option<u32> {
    let Some((name, qty)) = order.split_once('*') else {
        return None;
    };
    let Some(&(_, price)) = menu.iter().find(|(n, _)| *n == name.trim()) else {
        return None;
    };
    let qty: u32 = match qty.trim().parse() {
        Ok(q) => q,
        Err(_) => return None,
    };
    Some(price * qty)
}
```

메뉴가 `[("김밥", 3500), ("라면", 4000)]`일 때 결과는 아래와 같아야 합니다.

```text
"김밥 * 2" → Some(7000)
"라면*1" → Some(4000)
"떡볶이 * 1" → None
"김밥 * 두" → None
"김밥 두 줄" → None
```

<details>
<summary>힌트</summary>

- clippy는 두 `let...else`에 `question_mark`를 냅니다. 셋째 `match`(`Err(_) => return None`)는 잡지 않지만 같은 모양이므로 `.ok()?`로 바꿉니다(4.2절).
- 마지막 줄이 `price`와 `qty`를 함께 쓰므로 체인으로 가면 클로저가 겹칩니다. 2.1절의 기준대로 `?`판에서 멈추는 것이 좋습니다.

</details>

## 6. 참고 자료

- *Effective Rust* (David Drysdale): [Item 3: Prefer Option and Result transforms over explicit match expressions](https://effective-rust.com/transform.html). 이 챕터가 다룬 지침의 범위를 정한 기준 문서입니다. `Option`과 `Result`를 잇는 변환 메서드를 한 장의 그림(Figure 1-1)으로 정리해 두었으므로, 2.6절의 순서와 함께 보면 좋습니다.
- 본편의 관련 챕터: Part 0 [ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md)(`if let`, `let...else`), [ch9](../../part0-getting-started/ch9-error-handling/README.md)(`?` 연산자), [ch18](../../part0-getting-started/ch18-patterns/README.md)(let 체인, 기본 바인딩 모드, 매치 가드). Part 1 [ch2](../../part1-standard-lib/ch2-option-result/README.md)(메서드별 사용법, 즉시 평가와 게으른 평가, 메서드 선택표), [ch6](../../part1-standard-lib/ch6-core-traits/README.md)(`?`가 부르는 `From::from`), [ch8](../../part1-standard-lib/ch8-errors-panic/README.md)(`From`과 `map_err` 중 고르는 기준, 에러 열거형).
- 표준 라이브러리 문서: [`std::option`](https://doc.rust-lang.org/std/option/index.html)과 [`std::result`](https://doc.rust-lang.org/std/result/index.html)의 Method overview 절. `std::result`의 [Results must be used](https://doc.rust-lang.org/std/result/index.html#results-must-be-used) 절은 2.2절의 `#[must_use]`를, [The question mark operator](https://doc.rust-lang.org/std/result/index.html#the-question-mark-operator-) 절은 `?`가 하는 일을 설명합니다.
