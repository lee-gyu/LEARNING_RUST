# ch1. 타입 시스템으로 데이터 구조 표현하기

같은 데이터라도 어떤 타입으로 표현하느냐에 따라 컴파일러가 잡아 줄 수 있는 실수가 달라집니다. 포트 번호를 `i64`로 둘지 `u16`으로 둘지, 두 선택지를 `bool` 두 개로 받을지 열거형 두 개로 받을지, "없음"을 `-1`로 나타낼지 `Option`으로 나타낼지는 모두 컴파일되는 선택입니다. 차이는 잘못된 값이 프로그램에 들어왔을 때 드러납니다. 어떤 선택에서는 컴파일러가 막아 주고, 어떤 선택에서는 틀린 결과가 조용히 출력됩니다. 이 챕터는 본편에서 배운 기본 타입, 집합 타입, 열거형, `Option`, `Result`를 "무엇을 어떤 타입으로 표현할 것인가"의 관점에서 다시 봅니다. 여섯 지침(guideline)이 모두 같은 목표를 향합니다. **무효한 상태(invalid state)를 타입으로 표현할 수 없게 만드는 것**입니다.

- 원제: [Item 1: Use the type system to express your data structures](https://effective-rust.com/use-types.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch1-express-data-structures`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

문법은 다시 설명하지 않습니다. 다음 본편 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md): 정수·`bool`·`char` 같은 스칼라 타입, 튜플과 배열
- [Part 0 ch5](../../part0-getting-started/ch5-structs/README.md): 구조체와 튜플 구조체
- [Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md): 데이터를 가진 열거형, `match`의 완전성 검사, `Option`
- [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md): `Result`와 `?`
- [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md), [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md): `as`, `From`, `TryFrom`의 동작

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Rust가 정수 타입 사이에서 넓히는 변환(widening)까지 자동으로 하지 않는 이유를 설명하고, 값의 범위에 맞는 타입을 골라 변환을 입력 경계 한 곳에 모을 수 있다.
2. 원소 타입이 같은 튜플이 어떤 실수를 부르는지 설명하고, 이름 붙은 필드의 구조체로 바꿔야 할 때와 튜플로 충분할 때를 구별할 수 있다.
3. 뜻이 서로 다른 `bool` 매개변수를 열거형으로 바꿔, 인자 순서 실수를 컴파일 에러로 만들고 새 선택지를 안전하게 추가할 수 있다.
4. "이 필드는 저 필드가 X일 때만 의미가 있다"는 필드 조합을 찾아 데이터를 가진 열거형으로 바꾸고, 그 결과 어떤 무효한 상태가 사라지는지 설명할 수 있다.
5. 센티널 값(sentinel value) 대신 `Option`, 에러 코드 대신 `Result`를 써서 부재와 실패를 타입에 드러낼 수 있다.

## 2. 핵심 개념

지침마다 문제 상황, 비권장 코드, 권장 코드, 지침을 따르지 않아도 되는 경우 순서로 설명합니다. 코드는 핵심만 싣고, 두 형태를 함께 실행해 비교하는 전체 코드는 `examples/`에 있습니다.

### 2.1 기본 타입: 범위에 맞는 타입을 고르고, 변환은 경계에서

**문제 상황.** 설정 파일에서 포트 번호를 읽습니다. 포트는 0~65535이므로 `u16`이 맞지만, "나중에 어떤 값이 올지 모르니 넉넉하게" `i64`로 읽고 싶어집니다. 그런데 주소를 만드는 함수는 포트를 `u16`으로 받습니다.

C 계열 언어라면 `i64` 값을 `u16` 매개변수에 그대로 넘겨도 컴파일러가 값을 잘라 넣고, 경고 옵션을 따로 켜지 않으면 아무 말도 하지 않습니다. Rust는 이것을 E0308로 거부합니다. 값이 반드시 들어가는 넓히는 변환도 마찬가지여서, `u16` 값을 `u32` 자리에 두는 것도 에러입니다(4.1절). 변환이 일어나는 곳마다 "여기서 값이 바뀔 수 있는가"를 따져야 하는데, 변환이 자동으로 일어나면 그 지점이 코드에 보이지 않습니다. Rust는 모든 변환을 코드에 적게 해서 그 지점을 드러냅니다. 컴파일러의 `help:`도 두 경우를 구별합니다. 좁히는 변환(narrowing)에는 "들어가지 않으면 패닉한다"는 단서와 함께 `try_into().unwrap()`을, 넓히는 변환에는 단서 없이 `into()`를 제안합니다.

**비권장 코드.** 컴파일 에러를 없애려고 `as`를 붙입니다.

```rust
fn load_port_wide(text: &str) -> Result<i64, ParseIntError> {
    text.parse()
}

Ok(port) => address(port as u16), // "70000" → 127.0.0.1:4464, "-1" → 127.0.0.1:65535
```

`as`는 [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md)에서 본 것처럼 넘치는 위쪽 비트를 버리고 아무것도 알리지 않습니다. 에러 메시지는 사라졌지만 에러가 묻던 질문("이 값이 `u16`에 들어가는가")에는 아무도 답하지 않았고, 프로그램은 엉뚱한 포트로 연결합니다. clippy는 기본 설정에서 이 `as`를 잡지 않습니다. pedantic 그룹의 `cast_possible_truncation`을 켜면(`-W clippy::cast_possible_truncation`) `` casting `i64` to `u16` may truncate the value ``라고 경고하고 `try_from`을 제안합니다.

**권장 코드.** 처음부터 `u16`으로 읽습니다.

```rust
fn load_port(text: &str) -> Result<u16, ParseIntError> {
    text.parse()
}
```

타입을 `u16`으로 정한 것만으로 파싱이 범위 검사를 겸합니다. `"70000"`은 `number too large to fit in target type`, `"-1"`은 `invalid digit found in string`으로 거부됩니다. 이 함수를 통과한 값은 언제나 올바른 포트이므로 안쪽 코드는 다시 확인하지 않아도 됩니다. 변환은 바깥 세계(텍스트, 다른 라이브러리가 넘겨준 `i64`)와 만나는 **경계(boundary)** 한 곳에서만 하고, 안쪽에서는 의미에 맞는 타입으로 다닙니다. 이미 다른 타입으로 받은 값이라면 `u16::try_from(value)`가 같은 역할을 합니다. `as`, `From`, `TryFrom` 가운데 무엇을 고를지는 `ch5-type-conversions`에서 다룹니다.

넓히는 변환에서는 **위치**도 중요합니다. 0~65535 범위의 포트는 65536개라 `u16`에 들어가지 않으므로, 포트 개수를 세는 함수의 반환 타입은 `u32`여야 합니다. 이때 E0308의 `help:`를 그대로 따라 `(last - first + 1).into()`라고 쓰면 컴파일은 되지만, 계산은 여전히 `u16`으로 하고 결과만 넓힙니다. `ports_in_range(0, 65535)`를 부르면 디버그 빌드는 `attempt to add with overflow`로 패닉하고, 릴리스 빌드는 `0`을 돌려줍니다. 넓히는 변환은 계산하기 **전에** 합니다.

```rust
fn ports_in_range(first: u16, last: u16) -> u32 {
    u32::from(last) - u32::from(first) + 1
}
```

`char`도 같은 원칙으로 설계되어 있습니다. `'가' as u32`는 언제나 되지만, 반대 방향인 `u32 as char`는 E0604(`` only `u8` can be cast as `char`, not `u32` ``)이고 `char::from_u32`가 `Option<char>`를 돌려줍니다. 모든 `u32` 값이 유니코드 스칼라 값은 아니기 때문입니다.

**따르지 않아도 되는 경우.**

- 범위에 맞추라는 말은 "가장 작은 타입"을 고르라는 뜻이 아니라 "의미에 맞는 타입"을 고르라는 뜻입니다. 합계나 차이처럼 도메인이 범위를 정해 주지 않는 계산용 값은 넉넉한 타입(`u64`, `i64`)이 오히려 안전합니다.
- 해시나 체크섬처럼 위쪽 비트를 잘라 내는 것이 원하는 동작이면 `as`가 맞는 도구입니다.

→ [`examples/01_integer_types.rs`](examples/01_integer_types.rs)

### 2.2 집합 타입: 같은 타입이 나란하면 이름을 붙인다

**문제 상황.** 좌석 번호를 받아 몇째 줄 몇째 칸인지 돌려주는 함수를 만듭니다. 줄과 칸은 둘 다 `usize`입니다.

**비권장 코드.**

```rust
// 돌려주는 값은 (줄, 칸)
fn seat_position_tuple(seat_no: usize) -> (usize, usize) {
    (seat_no / SEATS_PER_ROW, seat_no % SEATS_PER_ROW)
}

let (x, y) = seat_position_tuple(8); // 호출한 쪽은 (가로, 세로)로 읽음
```

돌려주는 쪽의 순서는 (줄, 칸), 즉 (세로, 가로)인데, 받는 쪽은 화면 좌표의 관례대로 (가로, 세로)로 읽었습니다. 두 원소의 타입이 같으므로 컴파일러는 구별할 수 없습니다. 게다가 `let (x, y) = ...`는 코드 리뷰에서도 자연스러워 보입니다. 결과적으로 1줄 3칸 대신 3줄 1칸(둘 다 0부터 셈)에 표시합니다.

**권장 코드.**

```rust
struct SeatPos {
    row: usize,
    col: usize,
}

let pos = seat_position(8);
draw(pos.col, pos.row);
```

필드 이름으로 꺼내므로 순서를 기억할 필요가 없습니다. 구조체 패턴으로 풀 때도 `let SeatPos { col, row } = pos;`처럼 이름으로 맞추므로 적는 순서가 결과를 바꾸지 않습니다.

다만 이름은 실수를 **보이게** 할 뿐 막지는 못합니다. `draw(pos.row, pos.col)`도 컴파일됩니다. 그래도 `x` 자리에 `pos.row`를 넘기는 코드는 읽는 순간 어색하므로, 튜플을 풀던 때와 달리 리뷰에서 걸러집니다. 줄과 칸을 바꿔 넘기는 것까지 컴파일 에러로 만들려면 둘을 서로 다른 타입으로 감싸야 합니다. 그 방법인 뉴타입 패턴은 `ch6-newtype-pattern`에서 다룹니다.

| 데이터의 모양 | 고를 타입 |
| --- | --- |
| 같은 타입 여러 개, 위치마다 따로 이름이 없음(일주일의 기온 7개) | 배열 `[T; N]` |
| 타입이 서로 다른 값 몇 개를 잠깐 묶었다가 바로 풂 | 튜플 |
| 값마다 따로 의미가 있음, 특히 타입이 같을 때 | 이름 붙은 필드의 구조체 |

**따르지 않아도 되는 경우.** 원소 타입이 서로 다르면 순서를 바꿔 읽은 값을 쓰는 곳에서 타입 에러가 나므로 튜플이어도 안전합니다. 순서가 문맥에서 자명하고 받자마자 푸는 경우도 튜플로 충분합니다. 표준 라이브러리의 `str::split_once`는 `(&str, &str)`을 돌려주지만, "구분자 앞, 구분자 뒤"라는 순서가 글을 읽는 순서와 같아서 헷갈릴 일이 적습니다.

→ [`examples/02_named_fields.rs`](examples/02_named_fields.rs)

### 2.3 `bool` 매개변수 대신 열거형

**문제 상황.** 본문에서 단어를 세는 함수에 선택지가 두 개(대소문자 구분 여부, 단어 단위 여부) 있습니다.

**비권장 코드.**

```rust
fn count_flags(text: &str, word: &str, case_sensitive: bool, whole_word: bool) -> usize

count_flags(TEXT, WORD, false, true) // 의도: 대소문자 무시 + 단어 단위 → 3
count_flags(TEXT, WORD, true, false) // 두 bool의 순서를 바꿈 → 2, 컴파일도 실행도 문제없음
```

문제는 두 가지입니다. 호출 지점의 `false, true`만 보고는 무엇을 켰는지 알 수 없어서 시그니처를 찾아봐야 합니다. 그리고 두 인자의 타입이 같으므로 순서를 바꿔도 컴파일되고, 다른 답이 조용히 나옵니다.

**권장 코드.** 선택지마다 열거형을 하나씩 만듭니다.

```rust
enum Case {
    Sensitive,
    Insensitive,
}

enum Scope {
    Substring,
    WholeWord,
}

fn count(text: &str, word: &str, case: Case, scope: Scope) -> usize

count(TEXT, WORD, Case::Insensitive, Scope::WholeWord) // 3
```

이렇게 바꾸면 세 가지를 얻습니다.

1. 호출 지점이 스스로 설명합니다. 시그니처를 찾아보지 않아도 무엇을 골랐는지 보입니다.
2. `Case`와 `Scope`는 서로 다른 타입이므로 순서를 바꾸면 E0308이고, 컴파일러가 `help: swap these arguments`로 고칠 방법까지 알려 줍니다(4.2절).
3. 선택지를 늘릴 수 있습니다. `bool`에는 세 번째 값이 없으므로, "찾는 말에 대문자가 있을 때만 구분"하는 스마트 케이스(smart case) 같은 선택지를 넣으려면 매개변수를 통째로 바꿔야 합니다. 열거형에는 `Case::Smart`를 추가하면 되고, 배리언트를 모두 적은 `match`가 빠짐없이 E0004로 멈춰서 고칠 곳을 알려 줍니다(4.3절).

치르는 비용은 타입 정의 몇 줄뿐입니다. 이 툴체인에서 `size_of::<Case>()`는 `bool`과 같은 1바이트입니다.

**따르지 않아도 되는 경우.**

- 메서드 이름이 `bool`의 뜻을 말해 주고 인자가 그 하나뿐일 때는 `bool`로 충분합니다. 표준 라이브러리의 `OpenOptions::new().read(true).write(true)`나 `TcpStream::set_nonblocking(true)`가 그렇습니다. 호출 지점에서 뜻이 보이고, 순서를 헷갈릴 다른 `bool`도 없습니다.
- 뜻이 앞으로도 참과 거짓뿐이고 다른 `bool`과 섞이지 않게만 하고 싶다면, 뉴타입(`ch6-newtype-pattern`)이 더 가벼운 선택일 수 있습니다.

clippy에는 `bool` 매개변수가 많은 함수를 잡는 `fn_params_excessive_bools`가 있지만, pedantic 그룹이라 기본으로 꺼져 있고 켜도 `bool`이 3개를 넘을 때만 경고합니다. 이 예제의 `count_flags`처럼 두 개인 경우는 사람이 판단해야 합니다.

공개 라이브러리의 열거형에 배리언트를 추가하면 사용자 코드의 `match`도 E0004로 깨지므로 호환되지 않는 변경이 됩니다. 이 문제와 `#[non_exhaustive]` 속성은 [ch21](../ch21-semver/README.md)(Item 21)에서 다룹니다.

→ [`examples/03_bool_args_to_enum.rs`](examples/03_bool_args_to_enum.rs)

### 2.4 필드 조합 대신 데이터를 가진 열거형

[Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md)의 2.3절에서 "종류" 필드를 가진 구조체를 열거형으로 바꾸는 이유를 봤습니다. 이 절은 그 판단을 실제 코드에서 어떻게 알아차리는지, 그리고 무효한 상태가 실제로 어떤 문제를 일으키는지에 집중합니다.

**문제 상황.** 상품의 할인 규칙은 "할인 없음", "정률(%)", "정액(원)" 가운데 하나입니다. 관리 화면에서 규칙을 만들고 바꿀 수 있습니다.

**비권장 코드.**

```rust
struct DiscountFields {
    enabled: bool,
    is_rate: bool,     // enabled일 때만 의미가 있습니다.
    rate_percent: u32, // enabled && is_rate일 때만 의미가 있습니다.
    amount_won: u32,   // enabled && !is_rate일 때만 의미가 있습니다.
}
```

경고 신호는 **"~일 때만 의미가 있습니다"라는 주석**입니다. 이 주석은 타입이 지키지 못하는 규칙을 사람의 기억에 맡긴다는 표시입니다. `bool` 두 개의 네 조합 가운데 `enabled: false`인 두 조합은 같은 뜻이고, 숫자 필드 둘 중 하나는 언제나 무시돼야 합니다. 무시돼야 할 필드에 0이 아닌 값이 들어 있으면 무효한 상태인데, 그 값이 무슨 뜻인지는 타입이 정해 주지 않으므로 코드마다 다르게 해석합니다.

[`04_fields_to_enum.rs`](examples/04_fields_to_enum.rs)에서는 3000원 정액 할인을 정률로 바꾸려고 `is_rate`만 켜고, 비율을 넣기 전에 저장합니다. 그러자 `amount_won`부터 확인하는 설명 함수는 "3000원 할인"이라고 하고, `is_rate`부터 확인하는 결제 함수는 할인 없이 20000원을 받습니다. 두 함수 모두 올바른 값에서는 맞게 동작하므로, 어느 한쪽의 버그라고 하기도 어렵습니다. 문제는 그런 값을 만들 수 있게 한 타입에 있습니다.

**권장 코드.** 경우마다 필요한 데이터만 가진 열거형을 씁니다.

```rust
enum Discount {
    None,
    Rate { percent: u32 },
    Amount { won: u32 },
}
```

- "정률인데 금액도 있는" 값은 만들 방법이 없습니다. 정액을 정률로 바꾸는 일은 값 전체를 새 배리언트로 바꾸는 일이라(`discount = Discount::Rate { percent: 10 };`), 옛 금액이 남을 자리가 없습니다.
- 설명 함수와 결제 함수는 같은 배리언트를 `match`로 나누므로 해석이 갈라질 수 없습니다.
- 필드를 `Option`으로 바꾸는 것(`rate: Option<u32>, amount: Option<u32>`)은 해결책이 아닙니다. 둘 다 `Some`인 값을 여전히 만들 수 있습니다. "여럿 중 정확히 하나"는 열거형만 표현합니다.

경우의 수로 보면 차이가 분명합니다. 구조체가 표현할 수 있는 상태의 수는 필드마다의 경우의 수를 **곱한** 것이고, 열거형은 배리언트마다의 경우의 수를 **더한** 것입니다. 그래서 구조체를 곱 타입(product type), 열거형을 합 타입(sum type)이라고 부르고, 둘을 합쳐 대수적 데이터 타입(algebraic data type)이라고 합니다. 무효한 상태는 대개 곱하면서 생긴 남는 조합입니다. 덤으로 이 툴체인에서는 크기도 줄었습니다. `DiscountFields`는 12바이트, `Discount`는 8바이트입니다(기본 표현의 메모리 배치는 보장되지 않으므로 다른 툴체인에서는 다를 수 있습니다).

남은 무효한 상태도 있습니다. `Discount::Rate { percent: 150 }`은 여전히 만들 수 있습니다. 이처럼 값 하나의 범위에 관한 규칙은 열거형이 아니라, 값을 만드는 경로를 제한하는 타입으로 지킵니다(`ch6-newtype-pattern`).

**따르지 않아도 되는 경우.**

- 필드들이 정말 독립적이어서 모든 조합이 유효하면 구조체가 맞습니다. 글자 서식의 굵게·기울임·밑줄은 여덟 조합이 모두 의미가 있으므로 `bool` 필드 세 개로 충분합니다. 경고 신호인 주석이 붙을 일이 없습니다.
- 데이터베이스의 행이나 JSON처럼 바깥 형식이 평평한 모양을 요구할 때는, 그 모양을 경계에서만 쓰고 안으로 들어올 때 열거형으로 바꿉니다(2.1절의 경계와 같은 생각입니다).

→ [`examples/04_fields_to_enum.rs`](examples/04_fields_to_enum.rs)

### 2.5 센티널 값 대신 `Option`

센티널 값은 타입의 정상 값 가운데 하나를 골라 "없음"이라는 특별한 뜻을 붙인 것입니다. `-1`, 빈 문자열, `0`, `u32::MAX` 같은 값이 자주 쓰입니다. null 대신 `Option`을 쓰는 이유는 Part 0 ch6에서 봤습니다. 센티널은 null이 없는 Rust에서도 손쉽게 만들 수 있다는 점에서 따로 주의해야 합니다.

**문제 상황.** 순환 버스 노선에서 어떤 정류장의 다음 정류장을 찾습니다. 그리고 사용자 설정에서 값을 조회하되, 설정이 없으면 기본값을 씁니다.

**비권장 코드.**

```rust
fn position_or_minus_one(name: &str) -> i32 // 못 찾으면 -1

let i = position_or_minus_one(name);
let next = (i + 1) as usize % ROUTE.len(); // -1을 확인하지 않음: -1 + 1 = 0 → 첫 정류장

fn setting_or_empty<'a>(settings: &HashMap<&str, &'a str>, key: &str) -> &'a str // 없으면 ""
```

센티널의 문제는 두 가지입니다.

1. **확인을 잊으면 정상 값처럼 계산에 섞입니다.** 센티널도 같은 타입의 값이므로 컴파일러는 막을 수 없습니다. 노선에 없는 "공항"의 다음 정류장으로 "터미널"이 나옵니다.
2. **진짜 값과 겹칠 수 있습니다.** 사용자가 프롬프트를 일부러 비워 둔 값(`""`)과 설정하지 않은 상태가 같은 `""`가 되어, 비워 둔 프롬프트가 기본값 `"> "`으로 바뀝니다.

**권장 코드.**

```rust
fn position(name: &str) -> Option<usize> {
    ROUTE.iter().position(|stop| *stop == name)
}

fn setting<'a>(settings: &HashMap<&str, &'a str>, key: &str) -> Option<&'a str> {
    settings.get(key).copied()
}
```

`Option<usize>`는 `usize`처럼 계산에 쓸 수 없으므로 `None`을 처리하지 않고는 위치를 꺼낼 수 없습니다(첫 번째 문제 해결). `None`과 `Some("")`은 서로 다른 값이므로 "없음"과 "빈 값"이 구별됩니다(두 번째 문제 해결). 덤으로 `-1`을 담느라 필요했던 부호 있는 타입도 필요 없어져서 위치를 `usize`로 돌려줄 수 있습니다. 표준 라이브러리도 같은 선택을 합니다. `Iterator::position`, `str::find`, `HashMap::get`, `slice::first`는 모두 `Option`을 돌려줍니다. `copied`, `unwrap_or`, `?` 같은 도구로 `Option`을 다루는 방법은 `ch3-option-result-transforms`에서 정리합니다.

**따르지 않아도 되는 경우.**

- 빈 컬렉션이 곧 "없음"이라면 `Option<Vec<T>>`로 감싸지 않습니다. 글에 붙은 태그 목록은 태그가 없으면 빈 `Vec`으로 충분하고, `None`과 `Some(vec![])`이 같은 뜻인 타입은 오히려 무효한 상태를 하나 더 만듭니다. "아직 응답하지 않음"과 "응답했지만 아무것도 고르지 않음"처럼 둘을 구별해야 할 때만 `Option<Vec<T>>`를 씁니다.
- C 함수나 파일 형식처럼 바깥이 센티널을 정해 두었다면, 그 값은 경계에서 `Option`으로 바꿔 들여옵니다.

→ [`examples/05_sentinel_to_option.rs`](examples/05_sentinel_to_option.rs)

### 2.6 에러 코드 대신 `Result`

**문제 상황.** `"HH:MM"` 형식의 시각을 자정부터 지난 분으로 바꾸고, 회의 시작·끝 시각으로 회의 길이를 계산합니다. 형식이 틀리거나 범위를 벗어나면 실패해야 합니다.

**비권장 코드.** 실패를 음수 코드로 알립니다.

```rust
fn minutes_or_code(hhmm: &str) -> i32 // 성공: 0 이상, 형식 오류: -1, 범위 오류: -2

fn duration_discouraged(start: &str, end: &str) -> i32 {
    minutes_or_code(end) - minutes_or_code(start) // "10:3O"(영문자 O) → -1 - 570 = -571분
}
```

센티널의 첫 번째 문제가 그대로 나타납니다. 코드를 확인하지 않은 뺄셈이 -571분 같은 숫자를 만들어 냅니다. 여기에 두 가지가 더 붙습니다. `-1`은 형식 오류, `-2`는 범위 오류라는 뜻이 문서에만 있어서, 실패 이유가 늘어날 때마다 번호를 정하고 모든 호출자가 그 표를 알아야 합니다. 그리고 반환값을 통째로 버려도 컴파일러는 아무 말도 하지 않습니다.

**권장 코드.**

```rust
enum TimeError {
    BadFormat,
    OutOfRange,
    EndBeforeStart,
}

fn minutes(hhmm: &str) -> Result<u32, TimeError>

fn duration(start: &str, end: &str) -> Result<u32, TimeError> {
    let start = minutes(start)?;
    let end = minutes(end)?;
    end.checked_sub(start).ok_or(TimeError::EndBeforeStart)
}
```

- 성공 값과 실패 이유가 서로 다른 배리언트에 있어서 섞일 수 없습니다. 음수 코드를 담을 필요가 없으므로 성공 값의 타입도 `u32`로 정확해집니다.
- 실패 이유는 번호가 아니라 이름이 있는 배리언트입니다.
- `Result`는 `#[must_use]`가 붙은 타입이라서, 반환값을 쓰지 않고 버리면 기본으로 켜진 `unused_must_use` 린트가 경고합니다. `i32` 에러 코드를 버리면 아무 경고도 없습니다.

  ```
  warning: unused `Result` that must be used
  ...
     = note: this `Result` may be an `Err` variant, which should be handled
  ```

- 표준 타입이므로 `?`와 표준 라이브러리의 메서드를 그대로 쓸 수 있습니다. 위의 `duration`은 `?`로 실패를 바로 위로 넘깁니다. 에러 타입을 어떻게 설계할지는 `ch4-idiomatic-errors`에서 다룹니다.

`Option`과 `Result` 중에서는 이렇게 고릅니다. "찾지 못함"처럼 실패 이유가 하나뿐이고 자명하면 `Option`, 호출한 쪽이 이유에 따라 다르게 행동하거나 사용자에게 이유를 알려야 하면 `Result`입니다.

**따르지 않아도 되는 경우.** 프로세스의 종료 코드나 C에 내보내는 함수처럼 바깥이 정수 코드를 요구하는 경계가 있습니다. 그때도 안쪽은 `Result`로 다니다가, 경계에서 마지막에 코드로 바꿉니다.

→ [`examples/06_error_code_to_result.rs`](examples/06_error_code_to_result.rs)

### 2.7 책과 달라진 점

책은 rustc 1.70, 2018 에디션 기준으로 컴파일러 출력을 싣고 있습니다. 1.100.0에서 확인한 결과, 에러 코드와 제안의 내용은 같고 표시 방식만 달라졌습니다.

- 열거형 인자의 순서를 바꿨을 때의 `help: swap these arguments`는 책에서 바꿀 부분에 물결표(`~`)를 그은 한 줄로 나오지만, 1.100.0에서는 고치기 전 줄(`-`)과 고친 뒤 줄(`+`)을 나란히 보여 줍니다(4.2절).
- E0004의 `note:`에서 책은 빠진 배리언트에 `^`, 열거형 이름에 `-`를 긋지만, 1.100.0에서는 반대로 열거형 이름에 `^`, 빠진 배리언트에 `-`와 `not covered`를 긋습니다(4.3절).
- **책에 없는 지침.** 2.1절의 "값의 범위에 맞는 정수 타입을 고르고, 변환은 경계에서 한다"는 책의 Item 1이 명시적으로 권하는 내용이 아닙니다(2026-11-19에 온라인판에서 확인). 책은 Rust에 암묵적인 수치 변환이 없다는 점까지 설명하고, 타입을 고르는 기준은 이 챕터가 더했습니다.

### 2.8 기억할 것

- 변환은 경계에서 한 번 하고, 안쪽에서는 의미에 맞는 타입으로 다닙니다. 정수 타입 불일치 에러를 `as`로 덮지 않습니다.
- 같은 타입의 값이 나란히 있으면 이름을 붙입니다. 튜플은 구조체로, `bool` 인자는 열거형으로 바꿉니다.
- "~일 때만 의미가 있다"는 주석은 데이터를 가진 열거형으로 바꾸라는 신호입니다.
- 없음은 `Option`, 실패는 `Result`로 나타냅니다. 정상 값과 같은 타입의 특별한 값으로 나타내지 않습니다.
- 타입으로 막은 실수는 테스트를 따로 쓰지 않아도 다시 생기지 않습니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행합니다.

| 파일 | 지침 | 관련 절 |
| --- | --- | --- |
| [`01_integer_types.rs`](examples/01_integer_types.rs) | 범위에 맞는 정수 타입, 경계에서 변환 | 2.1 |
| [`02_named_fields.rs`](examples/02_named_fields.rs) | 같은 타입 튜플 대신 구조체 | 2.2 |
| [`03_bool_args_to_enum.rs`](examples/03_bool_args_to_enum.rs) | `bool` 매개변수 대신 열거형 | 2.3 |
| [`04_fields_to_enum.rs`](examples/04_fields_to_enum.rs) | 필드 조합 대신 데이터를 가진 열거형 | 2.4 |
| [`05_sentinel_to_option.rs`](examples/05_sentinel_to_option.rs) | 센티널 값 대신 `Option` | 2.5 |
| [`06_error_code_to_result.rs`](examples/06_error_code_to_result.rs) | 에러 코드 대신 `Result` | 2.6 |

### 01_integer_types — 범위에 맞는 정수 타입

같은 입력 세 개를 `i64`로 읽어 `as u16`으로 맞추는 쪽과 처음부터 `u16`으로 읽는 쪽에 넣습니다. 비권장 쪽은 범위 밖의 값을 엉뚱한 포트로 바꾸고, 권장 쪽은 입력 단계에서 거부합니다. 마지막 구역은 넓히는 변환을 계산 전에 해야 하는 이유를 `checked_add`로 패닉 없이 보여 줍니다.

```sh
cargo run -p er-ch1-express-data-structures --example 01_integer_types
```

```
[비권장] i64로 읽고 `as u16`으로 맞춤
  "8080"   -> 127.0.0.1:8080
  "70000"  -> 127.0.0.1:4464
  "-1"     -> 127.0.0.1:65535
[권장] 처음부터 u16으로 읽음
  "8080"   -> 127.0.0.1:8080
  "70000"  -> 거부: number too large to fit in target type
  "-1"     -> 거부: invalid digit found in string
[넓히기] 0~65535 범위의 포트 개수
  u16으로 계산: None
  u32로 넓힌 뒤 계산: 65536
  1024~2047: 1024
```

70000은 65536 + 4464이므로 `as u16`이 위쪽 비트를 버리면 4464가 남습니다. -1은 모든 비트가 1이라 `u16`의 최댓값 65535가 됩니다.

### 02_named_fields — 같은 타입 튜플 대신 구조체

8번 좌석(0부터 셈)의 위치를 튜플과 구조체로 받아 배치도에 `#`으로 표시합니다. 튜플을 (x, y)로 잘못 읽은 쪽은 다른 자리에 표시합니다.

```sh
cargo run -p er-ch1-express-data-structures --example 02_named_fields
```

```
[비권장] 튜플 (usize, usize)
  x=1, y=3
    .....
    .....
    .....
    .#...
[권장] 구조체 SeatPos { row, col }
  SeatPos { row: 1, col: 3 }
    .....
    ...#.
    .....
    .....
  이름으로 풀기: col=3, row=1
```

### 03_bool_args_to_enum — `bool` 매개변수 대신 열거형

`bool` 두 개의 순서를 바꾼 호출이 조용히 다른 답을 내는 것을 보이고, 열거형 판으로 네 조합을 모두 실행합니다. 네 결과가 모두 다르므로 어떤 순서 실수든 틀린 답으로 이어집니다.

```sh
cargo run -p er-ch1-express-data-structures --example 03_bool_args_to_enum
```

```
본문: "Rust rust RUSTY trust Rust", 찾는 말: "rust"
[비권장] bool 두 개
  의도한 호출 (false, true): 3
  순서를 바꾼 호출 (true, false): 2
[권장] 열거형 두 개
  count(.., Case::Insensitive, Scope::WholeWord): 3
  대소문자 구분 + 부분 문자열: 2
  대소문자 구분 + 단어 단위: 1
  대소문자 무시 + 부분 문자열: 5
  대소문자 무시 + 단어 단위: 3
```

"부분 문자열"은 `trust` 안의 `rust`까지 세고, "단어 단위"는 공백으로 나눈 단어가 정확히 같을 때만 셉니다.

### 04_fields_to_enum — 필드 조합 대신 데이터를 가진 열거형

올바른 필드 조합에서는 설명과 결제가 일치하다가, `is_rate`만 바꾼 무효한 상태에서 둘이 갈라지는 것을 보입니다. 열거형 판은 배리언트를 통째로 바꾸므로 항상 일치합니다.

```sh
cargo run -p er-ch1-express-data-structures --example 04_fields_to_enum
```

```
[비권장] 필드 조합 (정가 20000원)
  DiscountFields { enabled: true, is_rate: false, rate_percent: 0, amount_won: 3000 }
    설명: 3000원 할인 / 결제: 17000원
  DiscountFields { enabled: true, is_rate: true, rate_percent: 0, amount_won: 3000 }
    설명: 3000원 할인 / 결제: 20000원
[권장] 데이터를 가진 열거형
  Amount { won: 3000 }: 3000원 할인 / 결제: 17000원
  Rate { percent: 10 }: 10% 할인 / 결제: 18000원
  None: 할인 없음 / 결제: 20000원
```

### 05_sentinel_to_option — 센티널 값 대신 `Option`

1번 구역은 `-1` 센티널을 확인하지 않아 노선에 없는 정류장에도 다음 정류장이 나오는 상황을, 2번 구역은 빈 문자열 센티널이 사용자가 일부러 비워 둔 값과 겹치는 상황을 보여 줍니다.

```sh
cargo run -p er-ch1-express-data-structures --example 05_sentinel_to_option
```

```
[1] 순환 노선의 다음 정류장
  병원 -> 비권장: 터미널 / 권장: 터미널
  공항 -> 비권장: 터미널 / 권장: 노선에 없는 정류장
[2] 설정 조회 (기본값: prompt는 "> ", editor는 "vi")
  prompt: 비권장 "" -> "> " / 권장 Some("") -> ""
  editor: 비권장 "" -> "vi" / 권장 None -> "vi"
```

"병원"은 노선의 마지막 정류장이라 다음이 "터미널"인 것이 맞습니다. 비권장 판은 노선에 없는 "공항"에도 같은 답을 내므로, 비권장 판의 출력만 보고는 두 답 가운데 어느 것이 틀렸는지 알 수 없습니다.

### 06_error_code_to_result — 에러 코드 대신 `Result`

네 회의 가운데 첫째만 올바른 입력입니다. 비권장 판은 에러 코드가 뺄셈에 섞여 그럴듯한 음수를 출력하고, 권장 판은 실패 이유를 출력합니다.

```sh
cargo run -p er-ch1-express-data-structures --example 06_error_code_to_result
```

```
[비권장] 실패하면 음수 코드
  09:30~10:15: 45분
  09:30~10:3O: -571분
  23:00~24:30: -1382분
  14:00~13:00: -60분
[권장] Result<u32, TimeError>
  09:30~10:15: 45분
  09:30~10:3O: 계산할 수 없음 (BadFormat)
  23:00~24:30: 계산할 수 없음 (OutOfRange)
  14:00~13:00: 계산할 수 없음 (EndBeforeStart)
```

-571은 `-1 - 570`(형식 오류 코드에서 9시 30분을 뺀 값), -1382는 `-2 - 1380`(범위 오류 코드에서 23시를 뺀 값)입니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd appd-a/ch1-express-data-structures
rustc --edition 2024 compile_fail/e0308_no_implicit_int_conversion.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 고쳐서 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로 `--out-dir`로 다른 디렉터리를 지정하세요.

### 4.1 정수 타입이 다른 값을 그대로 씀 — E0308

파일: [`compile_fail/e0308_no_implicit_int_conversion.rs`](compile_fail/e0308_no_implicit_int_conversion.rs)

`u16` 계산 결과를 `u32` 반환 자리에 두고(넓히는 변환), `i64` 값을 `u16` 매개변수에 넘깁니다(좁히는 변환).

```
error[E0308]: mismatched types
  --> compile_fail/e0308_no_implicit_int_conversion.rs:16:5
   |
15 | fn ports_in_range(first: u16, last: u16) -> u32 {
   |                                             --- expected `u32` because of return type
16 |     last - first + 1 // E0308: u16 값을 u32 자리에 돌려줌
   |     ^^^^^^^^^^^^^^^^ expected `u32`, found `u16`
   |
help: you can convert a `u16` to a `u32`
   |
16 |     (last - first + 1).into() // E0308: u16 값을 u32 자리에 돌려줌
   |     +                ++++++++

error[E0308]: mismatched types
  --> compile_fail/e0308_no_implicit_int_conversion.rs:21:28
   |
21 |     println!("{}", address(configured)); // E0308: i64 값을 u16 매개변수에 넘김
   |                    ------- ^^^^^^^^^^ expected `u16`, found `i64`
   |                    |
   |                    arguments to this function are incorrect
...
help: you can convert an `i64` to a `u16` and panic if the converted value doesn't fit
   |
21 |     println!("{}", address(configured.try_into().unwrap())); // E0308: i64 값을 u16 매개변수에 넘김
   |                                      ++++++++++++++++++++
```

- **읽는 법**: 두 에러 모두 `` expected `X`, found `Y` ``가 핵심입니다. `help:`의 문구를 비교해 보세요. 넓히는 쪽은 그냥 "바꿀 수 있다"고 하지만, 좁히는 쪽은 "들어가지 않으면 패닉한다(panic if the converted value doesn't fit)"는 단서를 답니다. 컴파일러도 두 변환의 위험을 다르게 봅니다.
- **원인**: Rust는 정수 타입 사이에서 어떤 변환도 자동으로 하지 않습니다(2.1절).
- **해결**: 두 제안을 그대로 받아들이지 않습니다. 넓히는 쪽의 `(last - first + 1).into()`는 `u16`으로 계산한 뒤에 넓히므로 0~65535에서 넘칩니다. `u32::from(last) - u32::from(first) + 1`처럼 계산 전에 넓힙니다. 좁히는 쪽의 `try_into().unwrap()`은 잘못된 설정값에서 패닉합니다. `configured`를 처음부터 `u16`으로 읽거나, 경계에서 `u16::try_from(configured)`의 `Err`를 처리합니다. 어느 쪽이든 `as`로 덮지는 않습니다.

### 4.2 열거형 인자의 순서를 바꿈 — E0308

파일: [`compile_fail/e0308_swapped_enum_args.rs`](compile_fail/e0308_swapped_enum_args.rs)

`bool` 두 개였다면 조용히 다른 답을 냈을 실수가 열거형 두 개에서는 컴파일 에러가 됩니다.

```
error[E0308]: arguments to this function are incorrect
  --> compile_fail/e0308_swapped_enum_args.rs:33:13
   |
33 |     let n = count("Rust rust", "rust", Scope::WholeWord, Case::Insensitive); // E0308
   |             ^^^^^                      ----------------  ----------------- expected `Scope`, found `Case`
   |                                        |
   |                                        expected `Case`, found `Scope`
   |
note: function defined here
  --> compile_fail/e0308_swapped_enum_args.rs:21:4
   |
21 | fn count(text: &str, word: &str, case: Case, scope: Scope) -> usize {
   |    ^^^^^
help: swap these arguments
   |
33 -     let n = count("Rust rust", "rust", Scope::WholeWord, Case::Insensitive); // E0308
33 +     let n = count("Rust rust", "rust", Case::Insensitive, Scope::WholeWord); // E0308
   |
```

- **읽는 법**: 첫 줄이 `mismatched types`가 아니라 `arguments to this function are incorrect`입니다. 컴파일러가 인자 여러 개를 함께 보고, 각 인자 아래에 기대한 타입과 받은 타입을 적었습니다. `help:`의 `-` 줄이 지금 코드, `+` 줄이 고친 코드입니다.
- **원인**: `count`는 셋째 자리에 `Case`, 넷째 자리에 `Scope`를 받는데 순서를 바꿔 넘겼습니다.
- **해결**: `help:`대로 두 인자의 자리를 바꿉니다. 이 제안은 그대로 받아들여도 됩니다. 타입이 서로 다르므로 자리를 맞추는 방법이 하나뿐이기 때문입니다.

### 4.3 열거형에 배리언트를 추가함 — E0004

파일: [`compile_fail/e0004_new_case_variant.rs`](compile_fail/e0004_new_case_variant.rs)

[Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md)의 4.1절에서 배리언트를 추가하면 `match` 하나가 E0004로 멈추는 것을 봤습니다. 여기서는 `bool`이었다면 담을 수 없었던 세 번째 선택지 `Case::Smart`를 추가하고, 그 영향을 받는 `match` **두 곳**이 한 번에 보고되는 것을 확인합니다.

```
error[E0004]: non-exhaustive patterns: `Case::Smart` not covered
  --> compile_fail/e0004_new_case_variant.rs:19:15
   |
19 |         match self {
   |               ^^^^ pattern `Case::Smart` not covered
   |
note: `Case` defined here
  --> compile_fail/e0004_new_case_variant.rs:11:6
   |
11 | enum Case {
   |      ^^^^
...
14 |     Smart, // 새로 추가한 배리언트
   |     ----- not covered
   = note: the matched value is of type `Case`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
22 ~             Case::Insensitive => "대소문자 무시",
23 ~             Case::Smart => todo!(),
   |

error[E0004]: non-exhaustive patterns: `Case::Smart` not covered
  --> compile_fail/e0004_new_case_variant.rs:27:15
...
error: aborting due to 2 previous errors
```

- **읽는 법**: 같은 에러가 `label`(19행)과 `normalize`(27행)에서 한 번씩 나옵니다. 에러 목록이 곧 "새 선택지를 반영해야 할 곳"의 목록입니다. 프로그램이 커져서 `match`가 여러 파일에 흩어져 있어도 마찬가지입니다.
- **원인**: 두 `match` 모두 배리언트를 모두 적었는데, 새 배리언트 `Smart`의 갈래가 없습니다.
- **해결**: 두 `match`에 `Case::Smart` 갈래를 추가합니다. `help:`의 `todo!()`는 실행되면 패닉하는 자리 채우기이므로 실제 동작으로 바꿔야 합니다. `_ => ...` 갈래를 추가해도 컴파일은 되지만, 다음에 배리언트가 늘어날 때 이 목록을 받을 수 없게 됩니다. 실제로 고치는 방법은 연습 2에서 해 봅니다.

## 5. 연습 문제

### 연습 1. 경계에서 `try_from`으로 바꾸기

[`01_integer_types.rs`](examples/01_integer_types.rs)의 `load_port_wide`가 다른 라이브러리의 함수라서 `i64`를 돌려주는 것은 바꿀 수 없다고 합시다. `connect_discouraged`에서 `as u16`을 없애고, 범위 밖의 값을 거부하도록 고치세요. 출력은 다음과 같아야 합니다.

```
  "8080"   -> 127.0.0.1:8080
  "70000"  -> 거부: number too large to fit in target type
  "-1"     -> 거부: number too small to fit in target type
```

<details>
<summary>힌트</summary>

`Ok(port)` 갈래 안에서 `u16::try_from(port)`를 `match`로 나눕니다. `Ok(p)`이면 `address(p)`, `Err(e)`이면 `format!("거부: {e}")`입니다. `u16::try_from`의 에러는 값이 너무 크면 `number too large to fit in target type`, 너무 작으면 `number too small to fit in target type`을 출력합니다. 권장 판(`load_port`)과 비교하면 `-1`의 거부 이유가 다른데, 권장 판은 텍스트를 `u16`으로 파싱하다가 `-` 기호에서 실패하고, 이 판은 `-1`이라는 `i64`를 `u16`으로 바꾸다가 실패하기 때문입니다.

</details>

### 연습 2. 스마트 케이스 추가하기

[`03_bool_args_to_enum.rs`](examples/03_bool_args_to_enum.rs)의 `Case`에 `Smart` 배리언트를 추가하세요. 스마트 케이스는 찾는 말에 대문자가 하나라도 있으면 대소문자를 구분하고, 없으면 무시합니다. 먼저 배리언트만 추가해 컴파일러가 어디를 짚는지 확인한 뒤, 에러를 모두 고치고 다음 결과가 나오는지 확인하세요.

```
스마트 케이스 "rust" Substring: 5
스마트 케이스 "rust" WholeWord: 3
스마트 케이스 "Rust" Substring: 2
스마트 케이스 "Rust" WholeWord: 2
```

<details>
<summary>힌트</summary>

E0004는 `Case::label`과 `Case::normalize` 두 곳에서 납니다. `label`에는 `Case::Smart => "스마트 케이스"`를 추가하면 됩니다. `normalize`는 지금 문자열 하나만 받는데, `Smart`는 찾는 말을 봐야 구분 여부를 정할 수 있습니다. `fn normalize(self, s: &str, word: &str) -> String`으로 매개변수를 늘리고, 안에서 `let sensitive = match self { Case::Sensitive => true, Case::Insensitive => false, Case::Smart => word.chars().any(char::is_uppercase) };`로 정한 뒤 `sensitive`에 따라 그대로 두거나 소문자로 바꿉니다. 함수 안의 지역 변수로 `bool`을 쓰는 것은 문제가 없습니다. 이 지침은 호출하는 쪽에 드러나는 매개변수에 관한 것입니다. `count`에서는 `case.normalize(text, word)`와 `case.normalize(word, word)`로 부릅니다.

</details>

### 연습 3. 알림 설정 타입 설계하기

다음 구조체는 주문 알림 설정입니다.

```rust
struct NotifyFields {
    enabled: bool,
    by_email: bool, // enabled일 때만 의미가 있습니다.
    email: String,  // enabled && by_email일 때만 의미가 있습니다.
    phone: String,  // enabled && !by_email일 때만 의미가 있습니다.
}
```

(가) 알림은 "받지 않음", "이메일로 받음", "문자로 받음" 가운데 하나라는 요구 사항에 맞게 타입을 다시 설계하세요. (나) 요구 사항이 "이메일과 문자를 각각 켜고 끌 수 있다(둘 다 받을 수도 있다)"로 바뀌면 타입은 어떻게 달라져야 할까요? 두 설계에서 각각 어떤 무효한 상태가 사라지는지 설명해 보세요.

<details>
<summary>힌트</summary>

(가)는 2.4절과 같은 모양입니다. `enum Notify { Off, Email(String), Sms(String) }`처럼 경우마다 필요한 주소 하나만 담으면, "이메일인데 전화번호도 있는" 값이나 "꺼져 있는데 주소가 있는" 값이 사라집니다. (나)에서는 두 채널이 서로 독립적이어서 네 조합(둘 다 끔, 이메일만, 문자만, 둘 다)이 모두 유효합니다. 이때는 `struct Notify { email: Option<String>, sms: Option<String> }`처럼 채널마다 `Option` 필드를 두는 구조체가 맞습니다. "꺼짐"은 둘 다 `None`인 상태라서 `enabled` 같은 별도 필드가 필요 없고, "켜졌는데 주소가 없는" 값도 만들 수 없습니다. 2.4절에서 `Option` 필드 두 개가 해결책이 아니었던 것은 "정확히 하나"라는 요구 때문이었습니다. 요구 사항이 바뀌면 맞는 타입도 바뀝니다.

</details>

### 연습 4. 평균의 센티널 없애기

다음 함수는 점수가 하나도 없으면 `0.0`을 돌려줍니다. `0.0`이 왜 센티널로서 나쁜지 설명하고, `Option<f64>`를 돌려주도록 고치세요. `[]`, `[0, 0]`, `[80, 95]`를 넣었을 때 `None`, `Some(0.0)`, `Some(87.5)`가 나와야 합니다.

```rust
fn average(scores: &[u32]) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let sum: u32 = scores.iter().sum();
    f64::from(sum) / scores.len() as f64
}
```

<details>
<summary>힌트</summary>

모든 학생이 0점을 받은 반의 평균도 `0.0`입니다. 센티널이 진짜 값과 겹치는 경우(2.5절의 두 번째 문제)라서, 호출한 쪽은 "점수가 없음"과 "평균이 0점"을 구별할 수 없습니다. 비어 있을 때 `return None;`, 마지막 줄을 `Some(f64::from(sum) / scores.len() as f64)`로 바꾸면 됩니다. `{:?}`로 출력하면 `None Some(0.0) Some(87.5)`입니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 1: Use the type system to express your data structures](https://effective-rust.com/use-types.html). 이 챕터는 이 Item이 다루는 지침의 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- 표준 라이브러리 문서
  - [`u16`](https://doc.rust-lang.org/std/primitive.u16.html), [`i64`](https://doc.rust-lang.org/std/primitive.i64.html): 정수 타입의 범위와 `try_from` 등의 구현
  - [`char::from_u32`](https://doc.rust-lang.org/std/primitive.char.html#method.from_u32): `u32`에서 `char`로 가는 변환이 `Option`을 돌려주는 이유
  - [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html), [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
- 레퍼런스: [The `non_exhaustive` attribute](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute) — 공개 열거형에 배리언트를 추가할 수 있게 하는 속성([ch21](../ch21-semver/README.md)에서 다룸)
- 위키백과: [Algebraic data type](https://en.wikipedia.org/wiki/Algebraic_data_type) — 곱 타입과 합 타입
- 에러 설명: `rustc --explain E0308`, `rustc --explain E0004`
