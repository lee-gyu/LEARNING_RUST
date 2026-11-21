# ch28. 매크로는 신중하게 쓰기

매크로는 코드를 만들어 내는 코드입니다. 손으로 맞춰 써야 하는 반복을 없애 주지만, 부르는 쪽에서는 보통의 함수 호출처럼 보이면서 안에서는 무슨 일이든 할 수 있습니다. 인자를 두 번 실행하고, 부른 함수에서 `return`하고, 몰래 `&mut`를 붙이고, 다른 크레이트에서 펼쳐지면 엉뚱한 이름을 찾습니다. 이 챕터는 "매크로로 쓸 것인가, 어떤 매크로로 쓸 것인가, 쓴다면 어떻게 놀라움을 줄일 것인가"를 다룹니다. 지침마다 비권장 매크로와 권장 형태(함수, 제네릭, 덜 놀라운 매크로)를 같은 예제에서 실행해 비교하고, 마지막으로 구조체의 필드를 훑는 derive 매크로를 직접 만듭니다.

- 원제: [Item 28: Use macros judiciously](https://effective-rust.com/macros.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch28-macros-judiciously` (예제는 외부 크레이트를 쓰지 않습니다)
- 독립 프로젝트: `projects/tabular` (derive 매크로 워크스페이스, `syn` 3.0.10, `quote` 1.0.47, `proc-macro2` 1.0.107)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), rustfmt 1.11.0

책의 5장(Tooling)에 속한 Item입니다. 다음 본편·부록 내용을 안다고 보고 다시 설명하지 않습니다.

- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.18~2.20절: `macro_rules!`의 갈래와 반복(`$(...),*`), 프래그먼트 지정자 `expr`·`ident`·`literal`, `stringify!`, 절차적 매크로 세 종류, `syn`·`quote`로 만든 derive 매크로 `hello_macro`
- [ch12](../ch12-generics-vs-trait-objects/README.md): 제네릭, 트레이트 바운드, 단형성화
- [ch19](../ch19-avoid-reflection/README.md): 리플렉션 대신 derive가 컴파일할 때 만든 코드를 쓰는 이유
- [ch22](../ch22-minimize-visibility/README.md): 가시성과 E0603, [ch24](../ch24-re-export-dependencies/README.md): 재내보내기(`pub use`)
- [Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.11절과 [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.12절: 매크로 규칙은 정의한 크레이트의 에디션으로 해석되고, 2024 에디션의 `expr`는 `const { .. }`와 `_`도 받습니다.

프래그먼트 지정자 전체, 반복과 위생의 상세 규칙, 절차적 매크로 세 종류의 작성법 전체는 Part 9 `ch2-macros`에서, 매크로 디버깅 도구를 포함한 도구 지도는 [`ch31-tooling-ecosystem`](../ch31-tooling-ecosystem/README.md)에서 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 반복 코드를 함수, 제네릭, 매크로 중 어디로 없앨지 고르고(추상화 사다리), 매크로만 할 수 있는 일(타입 이름 자리의 반복, 서로 맞아야 하는 코드, 식의 글자와 부른 위치)을 예로 들어 설명할 수 있다.
2. 인자를 한 번만 평가하고, 숨은 제어 흐름과 숨은 참조가 없으며, 서식을 `format_args!`에 맡기는 선언적 매크로를 작성할 수 있다.
3. 위생(hygiene)이 지역 변수에만 적용되고 함수·타입 이름은 부른 곳에서 찾는다는 것을 설명하고, 내보내는 매크로를 `$crate`와 `::`로 시작하는 경로로 작성할 수 있다.
4. 절차적 매크로 세 종류 중 무엇을 고를지 판단하고, 필드를 훑는 derive 매크로를 보조 속성과 정확한 위치를 가리키는 에러까지 갖춰 작성할 수 있다.
5. 매크로의 비용(에러 메시지, `rustfmt` 같은 도구, 코드 팽창)을 실제 컴파일러 출력으로 설명하고, 일은 함수에 맡기는 얇은 매크로로 그 비용을 줄일 수 있다.

## 2. 핵심 개념

2.1절에서 매크로의 종류와 추상화 사다리를 정리하고, 2.2~2.8절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 2.9절은 매크로의 비용입니다. 지침 6과 몇몇 대안은 책에 없는 것을 더했습니다(2.10절).

### 2.1 지도: 매크로의 종류와 추상화 사다리

**토큰 위에서 동작합니다.** C 전처리기의 매크로도 토큰 단위로 바꿔 끼우지만, 인자의 토큰을 문법 구조와 상관없이 본문 자리에 그대로 이어 붙입니다. Rust의 매크로는 소스를 토큰으로 나눈 뒤 토큰(절차적 매크로는 토큰 스트림)을 받아 토큰을 돌려주되 `$x:expr` 같은 프래그먼트로 인자를 문법 단위로 받고, 그 결과는 보통의 코드와 똑같이 해석·타입 검사됩니다. 그래서 괄호를 빠뜨려 연산자 우선순위가 바뀌는 C 매크로의 고전적인 실수가 없고, `macro_rules!` 매크로는 위생적이어서 매크로 안의 지역 변수가 부르는 쪽의 같은 이름 변수와 섞이지 않습니다(예제 04).

**네 가지 매크로.**

| 종류 | 부르는 모양 | 받는 것 → 하는 일 | 정의하는 곳 | 흔한 정도 |
| --- | --- | --- | --- | --- |
| 선언적 매크로 | `row!(a, b)` | 토큰 → 규칙에 맞춰 바꿔 씀 | 같은 크레이트 어디서나(`macro_rules!`) | 흔함 |
| 함수형 절차적 매크로 | `sql!(…)` | 토큰 스트림 → 직접 해석해 새 토큰 | `proc-macro` 크레이트 | 드묾. 입력 모양이 정해져 있으면 선언적 매크로가 더 쉬움 |
| 속성 매크로 | `#[route(…)] fn f() {}` | 속성 인자와 항목 → 항목을 **바꿔 놓음** | `proc-macro` 크레이트 | 프레임워크에서 씀 |
| derive 매크로 | `#[derive(Columns)]` | 구조체·열거형 → 뒤에 코드를 **덧붙임**(원래 정의는 그대로) | `proc-macro` 크레이트 | 절차적 매크로 중 가장 흔함 |

**`!`는 경고 신호입니다.** 매크로 호출은 함수 호출처럼 보이지만 함수가 아닙니다. `m!(x)`는 `x`를 옮기지 않고 그 자리에서 고칠 수도, 두 번 실행할 수도, 부른 함수를 끝낼 수도 있습니다. 책은 `!`를 "여기서는 보통의 규칙이 적용되지 않을 수 있다"는 표시로 읽으라고 말합니다. 지침 3~5는 이 표시가 있어도 놀라지 않게 매크로를 쓰는 방법입니다.

**추상화 사다리.** 반복을 없애는 도구는 아래 칸부터 고릅니다.

| 칸 | 반복의 모양 | 도구 | 예제 01 |
| --- | --- | --- | --- |
| 1 | 같은 일을 같은 타입에 | 함수 | `mean_f64(&[f64])` |
| 2 | 같은 일을 트레이트 바운드를 공유하는 여러 타입에 | 제네릭 | `mean<T: Copy + Into<f64>>` |
| 3 | 타입으로는 묶이지 않고 코드의 모양만 같음 | 매크로 | `impl_limits!(i8 u8 …)` |

**위생은 지역 변수에만 적용됩니다.** The Reference는 `macro_rules!`의 위생을 "혼합 위생(mixed-site hygiene)"이라고 부릅니다. 지역 변수와 레이블은 매크로를 **정의한 곳**에서 찾고, 함수·타입·모듈·매크로 같은 나머지 이름은 매크로를 **부른 곳**에서 찾습니다. 앞의 것 때문에 매크로가 부르는 쪽의 지역 변수를 몰래 고칠 수 없고(4.1절), 뒤의 것 때문에 지침 6이 필요합니다(4.2절). 절차적 매크로는 위생이 없어서, 만들어 낸 코드가 부른 자리에 직접 쓴 코드처럼 해석됩니다.

### 2.2 지침 1: 함수나 제네릭으로 충분하면 매크로를 쓰지 않는다

**문제 상황.** 여러 숫자 타입의 평균을 구하는 코드가 반복됩니다. `u8` 조각, `i32` 조각, `f32` 조각마다 같은 계산입니다. 타입마다 함수를 쓰기 싫어서 매크로를 떠올립니다.

**비권장 코드.** [`01_abstraction_ladder.rs`](examples/01_abstraction_ladder.rs)의 `mean!`은 인자를 `as f64`로 바꿔 더합니다.

```rust
macro_rules! mean {
    ($($x:expr),+ $(,)?) => {{
        let xs = [$($x as f64),+];
        xs.iter().sum::<f64>() / xs.len() as f64
    }};
}
```

동작은 합니다. 문제는 매크로가 아무것도 약속하지 않는다는 것입니다. 받을 수 있는 타입이 시그니처에 없고, `as`가 받아 주는 타입이면 무엇이든 펼쳐집니다. `u64` 값 2^53 + 1과 1을 넘기면 `4503599627370496`이 나옵니다. 정확한 평균은 `4503599627370497`인데, `as f64`가 조용히 반올림했습니다. 또 매크로는 값이 아니므로 `map(mean!)`처럼 넘길 수 없고, 부르는 곳마다 본문이 통째로 펼쳐집니다.

**권장 코드.** 2칸(제네릭)으로 충분합니다.

```rust
fn mean<T: Copy + Into<f64>>(xs: &[T]) -> f64 {
    xs.iter().map(|&x| x.into()).sum::<f64>() / xs.len() as f64
}
```

바운드 `Into<f64>`가 "f64로 손실 없이 바뀌는 타입"만 받는다고 말합니다. `mean(&[big, 1])`(`u64`)은 `` the trait bound `f64: From<u64>` is not satisfied ``(E0277)로 컴파일되지 않습니다. 실수가 실행 결과가 아니라 컴파일 에러로 드러나는 것입니다. 함수이므로 `groups.into_iter().map(mean::<i32>)`처럼 값으로 넘길 수도 있습니다.

**매크로가 필요한 칸.** 같은 예제의 3번 구역은 정수 타입마다 `MIN`과 `MAX`를 트레이트 `Limits`로 묶습니다. 정수 타입의 `MIN`/`MAX`는 타입마다 따로 정의된 연관 상수이고 그 둘을 묶는 표준 트레이트가 없으므로, `impl<T> Limits for T`라는 제네릭 구현은 쓸 수 없습니다. 남은 길은 타입 이름만 다른 `impl` 블록 여섯 개를 손으로 쓰는 것과 매크로입니다.

```rust
macro_rules! impl_limits {
    ($($t:ty)*) => {
        $( impl Limits for $t {
            const LOW: Self = <$t>::MIN;
            const HIGH: Self = <$t>::MAX;
        } )*
    };
}
impl_limits!(i8 u8 i16 u16 i32 u32);
```

매크로가 만든 구현은 다시 제네릭 함수 `saturate<T: Limits + …>`가 씁니다(`saturate::<u8>(300) = 255`). 매크로는 사다리 꼭대기에서 아래 칸이 쓸 재료를 만들 뿐, 계산은 아래 칸이 합니다. 표준 라이브러리도 정수 타입마다 같은 메서드를 구현할 때 이렇게 합니다.

**따르지 않아도 되는 경우.** 인자 개수가 정해지지 않은 호출 모양 자체가 목적이면(`vec![1, 2, 3]`처럼) 매크로가 맞습니다. 다만 슬라이스 하나를 받는 함수로 같은 일을 할 수 있는지 먼저 봅니다.

### 2.3 지침 2: 서로 맞아야 하는 코드와 부른 곳의 정보는 매크로로 만든다

**문제 상황.** 책이 꼽는 매크로의 자리는 함수나 제네릭으로 만들 수 없는 반복, 특히 **따로 떨어진 코드가 서로 맞아야 하는** 반복입니다. 요금제 열거형에 "모든 요금제 목록", "코드 문자열", "코드로 요금제 찾기"가 따라다닌다고 합시다.

**비권장 코드.** [`02_single_source.rs`](examples/02_single_source.rs)의 `v1`은 네 곳을 손으로 맞췄고, 나중에 `Team`을 더했습니다.

```rust
pub const ALL: [Plan; 3] = [Plan::Free, Plan::Basic, Plan::Pro];   // Team을 빠뜨림
pub fn code(self) -> &'static str { match self { /* 네 갈래 */ } }  // 컴파일러가 Team을 요구함
pub fn from_code(code: &str) -> Option<Plan> {
    match code { "free" => Some(Plan::Free), /* … */ _ => None }      // "team"을 빠뜨림
}
```

컴파일러가 챙겨 준 곳은 `_` 갈래 없이 열거형을 받는 `code`뿐입니다(빠뜨리면 E0004). 배열 길이까지 손으로 적은 `ALL`과 문자열을 받아 `_` 갈래가 꼭 있어야 하는 `from_code`는 아무 경고 없이 어긋났고, 실행하면 `ALL`에 요금제가 3개뿐이고 `Team`의 왕복(`from_code(code())`)이 `None`입니다.

**권장 코드.** 요금제마다 한 줄씩 모든 정보를 표 하나에 두고, 매크로가 열거형과 메서드를 모두 만듭니다(`v2`).

```rust
plans! {
    Free  { code: "free",  label: "무료",   price: 0 },
    Basic { code: "basic", label: "베이직", price: 9_900 },
    Pro   { code: "pro",   label: "프로",   price: 19_900 },
    Team  { code: "team",  label: "팀",     price: 49_000 },
}
```

`ALL`, `code`, `label`, `monthly_price`, `from_code`가 모두 같은 줄 목록에서 펼쳐지므로 한 줄을 더하면 다섯 곳이 함께 바뀝니다. 표의 모양은 일부러 구조체 리터럴처럼 골랐습니다. 책은 매크로 문법이 "보통의 Rust처럼 보이거나, 확실히 다르게 보이라"고 권합니다. 어설프게 비슷한 문법은 읽는 사람이 Rust 규칙을 잘못 적용하게 만듭니다.

**부른 곳의 정보.** 함수는 인자의 값만 받습니다. [`03_call_site.rs`](examples/03_call_site.rs)는 같은 검사를 세 판으로 씁니다.

| 판 | 아는 것 | 실패 출력 |
| --- | --- | --- |
| 보통 함수 `check_v1(stock >= ordered)` | 결과(`false`)뿐 | `검사 실패` |
| `#[track_caller]` 함수 `check_v2(…)` | 부른 파일과 줄 | `검사 실패 (…/03_call_site.rs:56)` |
| 매크로 `check!(…)` | 식의 글자와 부른 위치 | `` 검사 실패: `stock >= ordered` (…/03_call_site.rs:60) `` |

매크로는 `stringify!($cond)`, `file!()`, `line!()`을 컴파일할 때 넣습니다. 식의 글자는 매크로만 얻을 수 있지만, **위치만 필요하면 `#[track_caller]`로 충분합니다.** 이 속성을 붙인 함수 안에서 `std::panic::Location::caller()`는 그 함수를 부른 위치를 돌려줍니다(1.46에서 안정화). `assert!`가 식의 글자를, `Option::unwrap`이 `#[track_caller]`로 부른 위치를 보여 주는 것이 각각의 예입니다.

**따르지 않아도 되는 경우.** 표에서 만들어지는 열거형은 소스에 보통의 Rust로 적혀 있지 않습니다. 책이 지적하듯 코드 탐색 도구가 매크로가 만든 정의를 따라가기 어렵고, 배리언트에 문서 주석을 달려면 표 매크로가 그것까지 받아 주게 만들어야 합니다. 필드나 배리언트마다 코드를 만드는 일이 전부라면, 타입 정의는 보통의 Rust로 두고 derive를 붙이는 쪽이 낫습니다(지침 7). 표 매크로는 `proc-macro` 크레이트를 따로 둘 만큼은 아닐 때의 가벼운 선택입니다.

### 2.4 지침 3: 인자는 한 번만 평가한다

**문제 상황.** 매크로는 인자의 값이 아니라 코드를 받습니다. 본문에 `$value`를 두 번 쓰면 부르는 쪽이 넘긴 식이 두 번 실행됩니다.

**비권장 코드.** [`04_evaluate_once.rs`](examples/04_evaluate_once.rs)의 `at_least!`는 값이 최솟값보다 작으면 최솟값을 돌려줍니다.

```rust
macro_rules! at_least {
    ($value:expr, $min:expr) => {
        if $value < $min { $min } else { $value }
    };
}
```

읽을 때마다 7, 2, 7, 2를 차례로 내놓는 센서로 `at_least!(sensor.read(), 5)`를 부르면 결과가 **2**이고 센서를 두 번 읽었습니다. 펼친 코드 `if sensor.read() < 5 { 5 } else { sensor.read() }`가 조건에서 읽은 7로 "5 이상"이라고 판단한 뒤 결과로는 다시 읽은 2를 돌려준 것입니다. 부수 효과가 두 번 일어났을 뿐 아니라, 비교한 값과 돌려준 값이 서로 다른 값이 되었습니다.

**권장 코드.** 세 가지 방법을 같은 예제에서 실행합니다. 모두 결과 7, 읽은 횟수 1입니다.

- **지역 변수에 한 번 바인드합니다**(`at_least_once!`). 본문을 `{ let value = $value; let min = $min; if value < min { min } else { value } }`로 씁니다. 매크로 안의 `value`는 위생 덕분에 부르는 쪽의 `value`와 섞이지 않습니다. 예제는 부르는 쪽 변수 `value = 5`를 `$min`으로 넘겨 이것을 확인합니다. 글자를 바꿔 끼우는 매크로였다면 `let min = value;`가 방금 읽은 센서 값 2를 가리켰겠지만, 결과는 5입니다.
- **프래그먼트를 `ident`로 좁힙니다**(`at_least_ident!`). 변수 이름은 몇 번 평가해도 부수 효과가 없습니다. 부르는 쪽이 `let reading = sensor.read();`로 먼저 담아야 하고, 식을 넘기면 `` no rules expected `.` ``으로 거부됩니다(4.3절).
- **함수로 씁니다**(`at_least`). 함수의 인자는 부르기 전에 정확히 한 번 평가됩니다. 이 경우는 표준 라이브러리의 `Ord::max`(`sensor.read().max(5)`)가 이미 같은 일을 하므로, 지침 1에 따라 애초에 매크로가 필요 없었습니다.

**따르지 않아도 되는 경우.** 인자를 일부러 여러 번 또는 조건부로 평가하는 것이 매크로의 목적이면(`debug_assert!`는 릴리스 빌드에서 인자를 평가하지 않습니다) 괜찮습니다. 대신 그 사실을 문서에 적습니다.

### 2.5 지침 4: 숨은 제어 흐름과 숨은 참조를 넣지 않는다

책은 매크로가 "보통의 Rust처럼 동작하게" 하라고 권하고, 따로 두 가지를 짚습니다. 비지역 제어 흐름을 피할 것, 확장 결과가 참조를 몰래 끼워 넣지 않게 할 것입니다.

**문제 상황 ①: 숨은 `return`.** 펼친 코드 안의 `return`과 `?`는 매크로가 아니라 **매크로를 부른 함수**에서 돌아갑니다. 부르는 쪽 코드에는 아무 표시가 없습니다.

**비권장 코드 ①.** [`05_hidden_control_flow.rs`](examples/05_hidden_control_flow.rs)의 `setting!`은 키가 없으면 `return Err(..)`로 빠져나갑니다. 이 매크로로 키 셋의 값을 출력하는 함수를 씁니다.

```rust
fn print_all_v1(map: &HashMap<&str, &str>, keys: &[&str]) -> Result<(), String> {
    for key in keys {
        let value = setting!(map, key);   // 이 줄에서 함수가 끝날 수 있다는 표시가 없음
        println!("  {key} = {value}");
    }
    println!("  (모든 키를 확인했습니다)");
    Ok(())
}
```

`port`가 없는 설정으로 부르면 `host` 한 줄만 출력하고 함수가 `Err`로 끝납니다. 반복문도, 마지막 줄도 건너뜁니다. 또 이 매크로는 `Result<_, String>`을 돌려주는 함수 안에서만 쓸 수 있습니다. `main`에서 부르면 매크로 정의 안의 `return Err(..)`를 가리키는 E0308이 납니다(4.5절).

**권장 코드 ①.** 실패를 값으로 돌려주는 함수 `setting(map, key) -> Result<&str, String>`을 쓰고, 흐름은 부르는 쪽이 고릅니다. `print_all_v2`는 `match`로 빠진 키를 알리고 다음 키로 넘어가 세 키를 모두 확인합니다. 빠져나가는 것이 맞는 `server_address`에서는 `setting(map, "port")?`라고 씁니다. 동작은 매크로 판과 같지만 줄 끝의 `?`가 그 사실을 보여 줍니다.

**문제 상황 ②: 숨은 참조.** 보통의 Rust에서 `f(scores)`는 `scores`를 옮기거나 복사합니다. 그 자리에서 고치려면 `f(&mut scores)`라고 써야 하고, 이 `&mut`가 읽는 사람에게 "여기서 값이 바뀐다"고 알려 줍니다. 매크로는 받은 코드 앞에 `&mut`를 붙여 펼칠 수 있습니다.

**비권장 코드 ②.** [`06_hidden_reference.rs`](examples/06_hidden_reference.rs)의 `normalize_v1!`은 본문 첫 줄이 `let scores = &mut $scores;`입니다. `normalize_v1!(scores)` 뒤에도 `scores`를 쓸 수 있고, 값은 `[40, 80, 20]`에서 `[50, 100, 25]`로 바뀌어 있습니다. `roster_v1!(names)`도 안에서 `&$names`로 빌리므로, 함수였다면 이동 때문에 쓸 수 없을(E0382) `names`를 뒤에서 그대로 씁니다.

**권장 코드 ②.** 빌림은 부르는 쪽이 씁니다. 함수 `normalize(scores: &mut [u32])`라면 시그니처가 이것을 강제하고, 매크로가 꼭 필요하더라도 본문이 인자를 `&mut [u32]`로 받게 하면(`normalize_v2!`) 부르는 쪽이 `normalize_v2!(&mut scores)`라고 써야 합니다. `normalize_v2!(scores)`는 `` expected `&mut [u32]`, found `Vec<{integer}>` ``(E0308)이고, help가 `&mut`를 붙이라고 제안합니다.

**따르지 않아도 되는 경우.**

- **제어 흐름이 매크로의 목적일 때.** 책도 이 경우는 허용하되 동작을 분명히 문서화하라고 합니다. 이 챕터의 프로젝트가 쓰는 `syn::parse_macro_input!`이 그 예입니다. 해석에 실패하면 컴파일 에러를 담은 토큰 스트림을 부른 함수에서 `return`하고, `syn` 문서는 "`proc_macro::TokenStream`을 돌려주는 함수에서 불러야 한다"고 밝혀 둡니다.
- **서식 매크로처럼 빌리는 것이 관례일 때.** `println!`과 `assert_eq!`는 인자를 불변으로 빌려 읽기만 하고(`assert_eq!`의 본문은 `match (&$left, &$right)`로 시작합니다), 이것은 널리 알려진 관례입니다. 프로젝트의 `row!`도 출력만 하는 매크로라서 이 관례를 따릅니다. 고치거나 옮기는 동작을 숨기는 것이 문제입니다.

### 2.6 지침 5: 서식은 `format_args!`에 맡기고 `format!` 문법을 따른다

**문제 상황.** 로그 매크로를 직접 만듭니다. 수준이 낮은 로그(여기서는 디버그)는 버립니다.

**비권장 코드.** [`07_format_args.rs`](examples/07_format_args.rs)의 `log_v1!`은 조각을 공백으로 이어 붙이는 문법을 새로 만들었습니다. `log_v1!(Level::Info; "포트", port, "에서 대기 중")`처럼 씁니다.

```rust
macro_rules! log_v1 {
    ($level:expr; $($part:expr),+ $(,)?) => {
        emit_parts($level, &[$($part.to_string()),+])
    };
}
```

- 서식이 고정되어 `포트 8080 에서 대기 중`처럼 공백이 어색하고, 정밀도를 줄 방법이 없어 `12.345678 ms`가 그대로 나옵니다. 너비, 디버그 형식, 변수 이름을 바로 쓰는 `{port}` 같은 `format!`의 기능을 쓰려면 모두 다시 구현해야 합니다.
- 조각마다 `to_string()`을 부른 뒤 함수에 넘기므로, 버려질 디버그 로그도 먼저 서식화됩니다. 예제는 `Display::fmt`가 불린 횟수를 세는 `Snapshot`으로 이것을 보여 줍니다(1번).

**권장 코드.** 나머지 토큰을 그대로 `format_args!`에 넘기고, 받는 함수는 `fmt::Arguments`를 받습니다.

```rust
fn emit(level: Level, args: fmt::Arguments<'_>) {
    if level < MIN_LEVEL { return; }
    println!("  [{level:?}] {args}");
}

macro_rules! log_v2 {
    ($level:expr, $($arg:tt)+) => {
        emit($level, format_args!($($arg)+))
    };
}
```

`log_v2!(Level::Info, "평균 응답 {avg_ms:.1}ms")`처럼 `format!` 문법이 그대로 됩니다. 표준 문서에 따르면 `format_args!`는 서식 문자열을 컴파일할 때 검사하고, 다른 서식 매크로와 달리 힙 할당을 하지 않습니다. 값은 `fmt::Arguments`를 실제로 출력할 때 서식화되므로, 수준을 먼저 확인하는 `emit`에서는 버려진 로그의 `Snapshot`이 한 번도 서식화되지 않습니다(0번). 서식 문자열과 인자 개수가 맞지 않으면(`log_v2!(Level::Info, "{} {}", port)`) `println!`과 같은 `2 positional arguments in format string, but there is 1 argument` 에러가 부른 곳의 서식 문자열을 가리킵니다.

**따르지 않아도 되는 경우.** 서식과 상관없는 매크로라면 해당하지 않습니다. 또 책이 예로 드는 `log` 크레이트처럼 이미 있는 로그 매크로를 쓸 수 있다면 직접 만들지 않습니다.

### 2.7 지침 6: 내보내는 매크로는 `$crate`로 자기 크레이트를 가리킨다

**문제 상황.** 라이브러리가 `#[macro_export]`로 매크로를 내보내고, 그 본문이 라이브러리의 함수를 부릅니다. `#[macro_export]` 매크로는 모듈 안에 정의해도 크레이트 루트에 놓입니다(`crate::metrics::track!`은 E0433, `crate::track!`으로 부름). 2.1절에서 본 대로 본문의 함수 이름은 **부른 곳**에서 찾으므로, 어떻게 적느냐에 따라 부르는 곳에서 깨집니다.

**비권장 코드.** 세 가지 잘못된 판이 있고, 셋 다 매크로를 정의한 크레이트 안에서는 동작합니다.

| 본문에 쓴 경로 | 정의한 크레이트 안 | 다른 크레이트에서 | 확인 |
| --- | --- | --- | --- |
| `record(..)` (경로 없음) | 같은 모듈에서만 동작 | E0425: 그 크레이트에서 `record`를 찾음 | 4.2절, `user_fails` |
| `crate::metrics::record(..)` | 동작(clippy `crate_in_macro_def` 경고) | E0425: `crate`가 **부른** 크레이트의 루트 | 예제 08, `user_fails` |
| `$crate::format_row(..)` (비공개 함수) | 동작 | E0603: 찾아가지만 비공개 | `user_fails` |

[`projects/tabular`](projects/tabular)의 라이브러리 `tabular`는 비교를 위해 이 세 판(`row_bare!`, `row_crate!`, `row_private!`)을 권장 판 `row!`와 함께 내보냅니다. `tabular`의 테스트 `every_row_macro_works_inside_the_defining_crate`는 넷을 모두 불러 통과합니다. 그런데 `tabular`를 의존성으로 쓰는 [`user_fails`](projects/tabular/user_fails)에서 넷을 부르면 `row!`만 컴파일되고 나머지 셋은 3절의 에러가 납니다. **매크로를 정의한 크레이트의 테스트만으로는 이 실수를 찾을 수 없다**는 것이 이 지침의 핵심입니다.

**권장 코드.** `$crate`는 어디서 펼쳐지든 매크로를 정의한 크레이트를 가리키는 특별한 메타변수입니다. 모듈 경로까지 전부 적습니다.

```rust
#[macro_export]
macro_rules! row {
    ($($value:expr),+ $(,)?) => {
        $crate::__format_row(&[$(
            (::core::stringify!($value), &$value as &dyn ::core::fmt::Display)
        ),+])
    };
}

#[doc(hidden)]
pub fn __format_row(pairs: &[(&str, &dyn Display)]) -> String { format_row(pairs) }
```

- **`$crate`는 가시성을 바꾸지 않습니다.** 펼친 코드가 부르는 항목은 부르는 곳에서 보여야 하므로 `pub`이어야 합니다. 공개 API로 내놓을 생각이 없는 도우미는 `#[doc(hidden)]`으로 문서에서 숨기고 이름 앞에 `__`를 붙이는 것이 관례입니다(The Reference의 Hygiene 절, [ch22](../ch22-minimize-visibility/README.md)).
- **재내보낸 의존성도 같은 원리입니다.** 매크로 본문이 의존성 `dep`의 항목을 쓴다면, 라이브러리에서 `pub use dep;`로 재내보내고 본문에서 `$crate::dep::item`이라고 씁니다. 부르는 쪽은 `dep`을 직접 의존하지 않아도 되고, 버전도 라이브러리가 쓰는 것과 같습니다([ch24](../ch24-re-export-dependencies/README.md)). 이 챕터의 프로젝트가 쓰는 `syn` 3.0.10이 실제로 그렇게 합니다. `#[doc(hidden)]`을 붙인 모듈 `__private`(소스는 `src/export.rs`)에서 의존성 `quote`와 `Ok`, `Err` 같은 표준 항목을 재내보내고, `parse_macro_input!`은 `$crate::__private::Ok`처럼 그 경로로 씁니다. 표준 라이브러리의 `assert_eq!`도 본문에서 `$crate::panicking::AssertKind::Eq`를 씁니다.
- **프렐류드의 이름도 가려질 수 있습니다.** 부르는 쪽 모듈에 `type Result<T> = core::result::Result<T, HandlerError>;` 같은 별칭은 흔합니다. 본문에 `Result<$ty, String>`이라고 쓴 매크로를 그 모듈에서 펼치면 `` type alias takes 1 generic argument but 2 generic arguments were supplied ``(E0107)입니다(예제 08의 사본으로 확인). 예제 08의 `parse_as!`처럼 `::core::result::Result`, `::std::format!`, `::core::stringify!`로 적으면 어떤 모듈에서 펼쳐도 같은 것을 가리킵니다.

예제 [`08_dollar_crate.rs`](examples/08_dollar_crate.rs)는 한 크레이트 안에서 `$crate` 판과 `crate::` 판이 같은 결과를 내는 것과(1번), 부르는 쪽의 `Result` 별칭에 가려지지 않는 매크로(2번)를 보여 줍니다. `crate::` 판에서 `#[expect(…)]` 다섯 줄을 지우고 `cargo clippy`를 실행하면 기본 설정에서 다음 경고가 납니다(줄 번호는 지운 뒤의 것).

```text
warning: `crate` references the macro call's crate
  --> appd-a/ch28-macros-judiciously/examples/08_dollar_crate.rs:28:13
   |
28 |             crate::metrics::record($event)
   |             ^^^^^ help: to reference the macro definition's crate, use: `$crate`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#crate_in_macro_def
   = note: `#[warn(clippy::crate_in_macro_def)]` on by default
```

이 린트는 `#[macro_export]`가 붙은 매크로에만 경고합니다(내보내지 않는 매크로의 `crate::`에는 경고하지 않는 것을 사본으로 확인). 경로 없이 쓴 판(`record(..)`)과 비공개 항목을 부르는 판은 clippy도 잡지 않으므로, 내보내는 매크로는 다른 크레이트에서 불러 보는 테스트가 필요합니다. 통합 테스트(`tests/`)는 라이브러리를 바깥 크레이트로 쓰므로 이 검사에 맞습니다. 프로젝트 사본의 `tabular/tests/`에서 `row_bare!`를 부르자 `user_fails`와 같은 E0425로 실패했습니다. 테스트 종류별 쓰임은 [`ch30-beyond-unit-tests`](../ch30-beyond-unit-tests/README.md)에서, 문서 테스트는 Part 4 `ch3-doc-tests`에서 다룹니다.

**따르지 않아도 되는 경우.** 내보내지 않는 매크로를 같은 모듈 안에서만 쓴다면 경로를 짧게 써도 됩니다. 다만 모듈 밖에서 쓰기 시작하는 순간 4.2절의 에러가 나므로, 처음부터 `$crate`(또는 `crate::`)로 적어 두는 편이 낫습니다.

### 2.8 지침 7: 필드·배리언트마다 만드는 코드는 derive로 만들고, 먼저 크레이트를 찾는다

**문제 상황.** 구조체를 표로 출력하려면 열 이름 목록과 각 행의 칸이 필요하고, 둘은 필드 목록과 개수·순서가 같아야 합니다. [ch19](../ch19-avoid-reflection/README.md)에서 본 대로 Rust에는 필드를 훑는 리플렉션이 없으므로, 손으로 쓰거나 컴파일할 때 만들어야 합니다.

**비권장 코드.** [`app/src/main.rs`](projects/tabular/app/src/main.rs)의 `BookByHand`는 `Columns`를 손으로 구현했습니다. 나중에 `pages` 필드를 더하면서 `cells()`에는 넣고 `NAMES`에는 넣는 것을 잊었습니다.

```rust
impl Columns for BookByHand {
    const NAMES: &'static [&'static str] = &["제목", "저자", "연도"];   // "쪽수"를 빠뜨림
    fn cells(&self) -> Vec<String> {
        vec![self.title.clone(), self.author.clone(), self.year.to_string(), self.pages.to_string()]
    }
}
```

두 곳이 맞아야 한다는 것은 컴파일러가 모르는 약속이라 에러도 경고도 없습니다. 표를 그려 보면 머리글은 셋, 칸은 넷입니다(3절).

**권장 코드.** 타입 정의는 보통의 Rust로 두고 `#[derive(Columns)]`를 붙입니다.

```rust
#[derive(Columns)]
struct Book {
    #[column(rename = "제목")]
    title: String,
    /* author, year, pages도 같은 모양 */
    #[column(skip)]
    shelf: u32, // 내부용이라 표에서 뺍니다.
}
```

[`tabular_derive`](projects/tabular/tabular_derive/src/lib.rs)는 필드를 한 번 훑어 `NAMES`와 `cells()`를 함께 만들므로, 필드를 더하거나 지우면 둘이 함께 바뀝니다. 책의 말대로 derive는 데이터 정의와 저절로 맞습니다. [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)의 `hello_macro`가 타입 이름만 썼다면, 이 매크로는 필드를 읽고 보조 속성까지 해석합니다. 구성에서 눈여겨볼 점은 다음과 같습니다.

- **보조 속성(helper attribute).** `#[proc_macro_derive(Columns, attributes(column))]`가 `#[column(..)]`을 선언합니다. 이 속성은 아무 일도 하지 않고 derive 매크로가 읽기만 하며, derive가 붙은 항목 안에서만 쓸 수 있습니다. `syn` 3.0.10의 `Attribute::parse_nested_meta`로 `skip`과 `rename = "…"`을 읽습니다.
- **에러는 패닉 대신 `syn::Error`로.** 구조체가 아니거나 모르는 속성 키를 쓰면 `syn::Error::new_spanned`로 만든 에러를 `into_compile_error()`로 바꿔 돌려줍니다. 에러가 `enum Shelf`의 이름이나 `#[column(hide)]`의 `hide`를 정확히 가리킵니다(4.5절).
- **칸 코드에 필드의 위치(span)를 붙입니다.** `quote_spanned!(field.ty.span() => …)`로 만들었기 때문에, `Display`가 없는 필드(`Vec<String>`)를 넣으면 E0277이 그 필드를 가리킵니다. 같은 코드를 `quote!`로 만들면 에러가 `#[derive(Columns)]`만 가리켰습니다(사본으로 확인).
- **만든 코드는 `::tabular::Columns`처럼 맨 앞의 `::`부터 적습니다.** 절차적 매크로에는 `$crate`가 없고 위생도 없습니다. The Reference는 `::std::option::Option` 같은 절대 경로와 `__internal_foo` 같은 겹치지 않을 이름을 쓰라고 권합니다. 그래서 `Vec`, `String`, `vec!`도 `::std::`부터 적었습니다. 이 방식은 쓰는 쪽이 의존성 이름을 바꾸면(`[dependencies]`의 `package` 키) 깨진다는 한계가 있습니다.
- **트레이트 크레이트가 derive를 재내보냅니다.** `tabular`의 `pub use tabular_derive::Columns;` 덕분에 `app`은 `tabular` 하나만 의존하고, `use tabular::Columns;` 한 줄로 트레이트와 derive 매크로를 함께 가져옵니다. 둘은 이름 공간이 달라 같은 이름으로 재내보낼 수 있습니다. `hello_macro`가 `use`를 두 줄 쓴 것과 비교해 보세요.

**어떤 절차적 매크로를 고르나.** 필드나 배리언트마다 코드를 만드는 일이면, 타입 정의를 통째로 입력으로 받아 타입을 만들어 내는 매크로(2.3절의 표 매크로나 속성 매크로)보다 derive를 고릅니다. 책은 derive가 더 관용적이고 읽기 쉽다고 말합니다. derive는 원래 정의를 바꾸지 않으므로 소스에 적힌 구조체가 그대로 진실입니다. 항목 자체를 바꿔야 하면(함수 앞뒤에 코드를 감싸기) 속성 매크로, 입력이 Rust 문법이 아니면 함수형 매크로입니다.

**따르지 않아도 되는 경우: 이미 있는 크레이트.** derive가 프로젝트에 특화된 일이 아니라면 직접 쓰기 전에 크레이트를 찾습니다([ch25](../ch25-dependency-graph/README.md)). 책은 직렬화의 `serde`, 정수와 C 스타일 열거형 배리언트 사이의 변환을 위한 `enumn`, `num_enum`, `num_derive`, `strum`을 듭니다(이 챕터에서는 이름만 소개하고 쓰지 않습니다). 반대로 `proc-macro` 크레이트 하나를 들이는 비용(빌드 시간, `syn` 같은 의존성, 빌드할 때 실행되는 코드)이 일에 비해 크면 손으로 쓴 구현과 테스트로 버팁니다.

### 2.9 매크로의 비용과 줄이는 방법

책은 매크로의 단점으로 가독성(읽는 사람이 매크로의 작은 언어를 배워야 함), 도구, 컴파일 에러, 코드 팽창을 듭니다. 이 툴체인에서 확인한 모습은 다음과 같습니다.

**에러가 매크로 정의 안을 가리킵니다.** [`compile_fail/e0369_logic_inside_macro.rs`](compile_fail/e0369_logic_inside_macro.rs)는 숫자 자리에 문자열을 넘기는 같은 실수를 두 매크로에 저지릅니다. 계산을 매크로 안에서 하는 `report!`(안에서 `percent!`를 부름)는 E0369가 `percent!` **정의** 안의 `*`를 가리키고, 부른 줄에는 "in this macro invocation"만 붙습니다. 매크로 사슬은 `` this error originates in the macro `percent` which comes from the expansion of the macro `report` `` 한 줄로 줄어듭니다. 사슬을 모두 보는 `-Z macro-backtrace`는 에러 메시지가 말하듯 nightly 전용입니다. 다른 크레이트의 매크로라면 더 나빠서, `user_fails`의 에러는 부른 줄만 가리키고 매크로 본문은 보여 주지 않습니다(3절).

**얇은 매크로가 이 비용을 줄입니다.** 같은 파일의 `report_thin!`은 계산을 타입이 적힌 함수 `percent_of(part: u32, whole: u32)`에 넘기기만 합니다. 이번에는 E0308 `` expected `u32`, found `&str` ``가 **부른 줄의 `done`** 을 가리킵니다. 이 챕터의 권장 판(`check!`, `log_v2!`, `row!`)은 모두 이 모양입니다. 매크로는 컴파일할 때만 알 수 있는 것(식의 글자, 위치, 서식 문자열)을 덧붙여 함수를 한 번 부르고, 일은 함수가 합니다. 인자가 함수 인자로 한 번 평가되므로 지침 3도 저절로 지켜집니다.

**`rustfmt`는 함수 호출처럼 보이는 매크로만 다듬습니다.** 다음 파일을 `rustfmt --edition 2024`로 서식화해 보았습니다.

```rust
macro_rules! table {
    ( $( $k:ident => $v:expr ),* $(,)? ) => {{
        let mut v = Vec::new();
            $(   v.push(  (stringify!($k),$v)  );   )*
        v
    }};
}
macro_rules! pair { ($a:expr, $b:expr) => { ($a,$b) }; }
// main 안:
let t = table!{ alpha=>1,   beta  =>   2 };
let p = pair!(1+2,   3   *4);
```

`pair!(1 + 2, 3 * 4)`와 `pair`의 본문(`($a, $b)`)은 다듬어졌지만, `table!` 호출은 `!` 뒤에 공백 하나만 더해지고 안쪽은 그대로였고, 반복이 있는 `table` 본문도 손대지 않았습니다. 이 챕터의 `plans!` 표도 `cargo fmt`가 정렬을 건드리지 않습니다. 호출 모양이 보통의 Rust 식이면 도구가 보통의 Rust처럼 다뤄 줍니다. 2.3절의 "보통의 Rust처럼 보이게"가 가독성만의 문제가 아닌 이유입니다.

**코드 팽창과 펼친 결과 보기.** 매크로 한 줄이 수십·수백 줄로 펼쳐질 수 있고, 펼친 코드는 소스에 보이지 않아 필요 없어져도 알아채기 어렵습니다. 펼친 결과를 보는 도구로 책은 `cargo-expand`를 소개합니다(`cargo install cargo-expand`로 설치하고 `cargo expand`로 실행, Part 0 ch19 2.18절). 이 저장소에는 설치되어 있지 않아 실행해 확인하지 못했고, 결과도 싣지 않았습니다. 이 챕터는 대신 펼친 코드의 효과를 실행 결과로 보이고, 손으로 적은 "펼친 모양"은 주석에 그렇다고 밝혔습니다(예제 04).

### 2.10 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이와 더한 내용은 다음과 같습니다.

- **`syn` 3.** 이 챕터의 프로젝트는 `syn` 3.0.10을 씁니다. 사용한 API(`parse_macro_input!`, `DeriveInput`, `Attribute::parse_nested_meta`, `Error::new_spanned`, `into_compile_error`)는 로컬 레지스트리에 내려받은 3.0.10 소스에서 확인했습니다.
- **2024 에디션의 `expr` 프래그먼트.** 2024 에디션에서 `$x:expr`는 `const { .. }` 블록과 `_`도 받습니다([Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.12절). 이 챕터의 매크로는 그 차이에 영향을 받지 않습니다.
- **에러의 매크로 사슬.** 1.100.0은 중첩 매크로의 에러에 `this error originates in the macro … which comes from the expansion of the macro …`를 붙여 사슬의 양 끝을 보여 줍니다. 전체 사슬(`-Z macro-backtrace`)은 nightly 전용입니다.
- **`rustfmt`.** 책은 `rustfmt`가 매크로 본문을 잘 다루지 못한다고만 말합니다. rustfmt 1.11.0은 함수 호출 모양의 매크로 호출과 단순한 `macro_rules!` 본문은 서식화하고, 사용자 정의 문법의 호출과 반복이 있는 본문은 그대로 둡니다(2.9절).
- **책에 없는 지침과 내용.** 다음은 책의 Item 28이 명시적으로 권하지 않는 것입니다(2026-11-20에 온라인판에서 확인. 책의 페이지에는 `$crate`가 나오지 않습니다).
  - **지침 6 전체**(`$crate`, `$crate`와 가시성, `#[doc(hidden)] pub` 도우미, 재내보낸 의존성, `::`로 시작하는 경로): The Reference의 Macros By Example > Hygiene 절, Procedural Macros의 hygiene 설명, clippy `crate_in_macro_def`(1.100.0에서 기본 warn), `user_fails`의 실제 컴파일 결과, `syn` 3.0.10 소스가 근거입니다.
  - **지침 2의 `#[track_caller]` 대안**: 로컬 문서의 `std::panic::Location`(1.46.0)과 릴리스 노트 1.46.0, 예제 03의 실행 결과가 근거입니다.
  - **2.9절의 얇은 매크로**(일은 함수에, 매크로는 컴파일할 때만 아는 것을 덧붙이기)와 지침 7의 `quote_spanned!`: `compile_fail/e0369_logic_inside_macro.rs`와 프로젝트 사본의 컴파일 결과가 근거입니다.
  - **지침 1의 "매크로에는 바운드가 없다"는 비교**(`as`로 값을 잃는 `mean!`)와 **지침 5의 "버려지는 로그를 서식화하지 않는다"는 비교**: 예제 01과 07의 실행 결과, `format_args!`의 표준 문서가 근거입니다. 책은 지침 자체(제네릭으로 충분하면 제네릭, `format_args!` 사용)만 말합니다.

### 2.11 기억할 것

- 사다리의 아래 칸부터 고릅니다. 함수 → 제네릭 → 매크로. 매크로는 타입 이름 자리의 반복, 서로 맞아야 하는 코드, 식의 글자처럼 아래 칸이 할 수 없는 일에만 씁니다.
- 매크로는 보통의 Rust처럼 동작하게 합니다. 인자는 한 번만 평가하고, `return`·`?`·`&mut`를 부르는 쪽 몰래 넣지 않습니다.
- 서식은 `format_args!`에 넘기고 `format!` 문법을 따릅니다.
- 내보내는 매크로는 `$crate::`와 `::`로 시작하는 경로로 쓰고, 펼친 코드가 부르는 항목은 `pub`이어야 합니다. 정의한 크레이트의 테스트로는 이 실수를 못 찾습니다.
- 필드·배리언트마다 만드는 코드는 derive로, 직접 쓰기 전에 크레이트를 찾습니다. 매크로는 얇게 두고 일은 함수에 맡깁니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 끝납니다. 예제 03의 경로는 Cargo가 rustc에 넘기는 소스 경로(워크스페이스 루트 기준)라서, 챕터 디렉터리에서 `cargo run`을 실행해도 같습니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_abstraction_ladder.rs`](examples/01_abstraction_ladder.rs) | 1 | 제네릭으로 충분한 `mean!` | 제네릭 `mean`, 매크로가 필요한 `impl_limits!` | 2.2 |
| [`02_single_source.rs`](examples/02_single_source.rs) | 2 | 손으로 맞춘 열거형·목록 | 표 하나에서 만드는 `plans!` | 2.3 |
| [`03_call_site.rs`](examples/03_call_site.rs) | 2 | 위치·식을 모르는 함수 | `#[track_caller]`, 얇은 `check!` | 2.3 |
| [`04_evaluate_once.rs`](examples/04_evaluate_once.rs) | 3 | 인자를 두 번 평가하는 `at_least!` | 바인드, `ident`, 함수 | 2.4 |
| [`05_hidden_control_flow.rs`](examples/05_hidden_control_flow.rs) | 4 | 숨은 `return`이 있는 `setting!` | `Result`를 돌려주는 함수 | 2.5 |
| [`06_hidden_reference.rs`](examples/06_hidden_reference.rs) | 4 | 몰래 `&mut`·`&`를 붙이는 매크로 | `&mut`가 보이는 함수·매크로 | 2.5 |
| [`07_format_args.rs`](examples/07_format_args.rs) | 5 | 서식을 새로 만든 `log_v1!` | `format_args!`에 넘기는 `log_v2!` | 2.6 |
| [`08_dollar_crate.rs`](examples/08_dollar_crate.rs) | 6 | `crate::`로 쓴 `track_crate!` | `$crate::` 판, `::`로 시작하는 경로 | 2.7 |
| [`projects/tabular`](projects/tabular) | 6, 7 | 손으로 쓴 `impl Columns`, `$crate` 없는 매크로 | `#[derive(Columns)]`, `row!` | 2.7, 2.8 |

### 01_abstraction_ladder — 함수, 제네릭, 매크로

```sh
cargo run -p er-ch28-macros-judiciously --example 01_abstraction_ladder
```

```text
1. 함수: 한 타입
  mean_f64([1.5, 2.5, 3.5]) = 2.5
2. 제네릭: 바운드를 공유하는 여러 타입
  mean(&[1u8, 2, 6]) = 3
  mean(&[10i32, -20, 40]) = 10
  mean(&[0.5f32, 1.5]) = 1
  map(mean::<i32>) = [2.0, 15.0]
  비권장 mean!(2^53 + 1, 1) = 4503599627370496
  정확한 평균                = 4503599627370497
3. 매크로: 타입 이름만 다른 impl 블록
  saturate::<u8>(300)    = 255
  saturate::<i8>(-200)   = -128
  saturate::<u16>(70000) = 65535
  saturate::<i32>(42)    = 42
```

### 02_single_source — 정보는 한 곳에

```sh
cargo run -p er-ch28-macros-judiciously --example 02_single_source
```

```text
1. 비권장: 손으로 맞춘 목록
  ALL = ["free", "basic", "pro"]
  Free: code() = "free", from_code(code()) = Some(Free)
  Basic: code() = "basic", from_code(code()) = Some(Basic)
  Pro: code() = "pro", from_code(code()) = Some(Pro)
  Team: code() = "team", from_code(code()) = None
2. 권장: 표 하나에서 열거형과 메서드를 모두 만듦
  ALL = ["free", "basic", "pro", "team"]
  Free: 무료 0원, from_code(code()) = Some(Free)
  Basic: 베이직 9900원, from_code(code()) = Some(Basic)
  Pro: 프로 19900원, from_code(code()) = Some(Pro)
  Team: 팀 49000원, from_code(code()) = Some(Team)
```

### 03_call_site — 부른 곳의 정보

```sh
cargo run -p er-ch28-macros-judiciously --example 03_call_site
```

```text
1. 보통 함수
  검사 실패
2. #[track_caller] 함수
  검사 실패 (appd-a/ch28-macros-judiciously/examples/03_call_site.rs:56)
3. 매크로
  검사 실패: `stock >= ordered` (appd-a/ch28-macros-judiciously/examples/03_call_site.rs:60)
  돌려받은 값: false
```

### 04_evaluate_once — 인자를 한 번만 평가하기

```sh
cargo run -p er-ch28-macros-judiciously --example 04_evaluate_once
```

```text
1. 비권장: 인자를 두 번 평가하는 매크로
  at_least!(sensor.read(), 5)  결과 2, 센서를 읽은 횟수 2
2. 권장: 한 번만 평가
  at_least_once!               결과 7, 센서를 읽은 횟수 1
  at_least_once!(.., value)    결과 5, 센서를 읽은 횟수 1
  at_least_ident!(reading, 5)  결과 7, 센서를 읽은 횟수 1
  at_least(sensor.read(), 5)   결과 7, 센서를 읽은 횟수 1
```

셋째 줄만 센서가 2를 내놓고 최솟값으로 부르는 쪽 변수 `value`(5)를 넘긴 경우입니다(2.4절의 위생 확인).

### 05_hidden_control_flow — 숨은 return과 `Result`

```sh
cargo run -p er-ch28-macros-judiciously --example 05_hidden_control_flow
```

```text
1. 비권장: 매크로 안의 숨은 return
  host = 127.0.0.1
  돌려받은 값: Err("`port` 설정이 없습니다")
2. 권장: Result를 돌려주는 함수, 흐름은 부르는 쪽이 고름
  host = 127.0.0.1
  `port` 설정이 없습니다 (건너뜀)
  name = demo
  (모든 키를 확인했습니다)
  server_address: Err("`port` 설정이 없습니다")
```

### 06_hidden_reference — 몰래 붙인 `&mut`

```sh
cargo run -p er-ch28-macros-judiciously --example 06_hidden_reference
```

```text
1. 비권장: 매크로가 몰래 빌림
  normalize_v1!(scores) 뒤 scores = [50, 100, 25]
  roster_v1!(names) = 3명: 하나, 두리, 세찬, 뒤에도 names.len() = 3
2. 권장: 빌림을 부르는 쪽에 드러냄
  normalize(&mut scores) 뒤 scores = [50, 100, 25]
  normalize_v2!(&mut scores) 뒤 scores = [50, 100, 25]
  roster(&names) = 3명: 하나, 두리, 세찬
```

결과는 같습니다. 다른 것은 부르는 줄에 `&mut`와 `&`가 보이는지입니다. `normalize_v2!`는 빌림을 드러내는 모양을 보이려고 둔 것이고, 실제로는 함수 `normalize`로 충분합니다(지침 1).

### 07_format_args — 서식을 새로 만들지 않기

```sh
cargo run -p er-ch28-macros-judiciously --example 07_format_args
```

```text
1. 비권장: 서식을 새로 만든 매크로
  [Info] 포트 8080 에서 대기 중
  [Info] 평균 응답 12.345678 ms
  버려진 로그를 위해 Snapshot을 서식화한 횟수: 1
2. 권장: format_args!에 넘기는 매크로
  [Info] 포트 8080에서 대기 중
  [Info] 평균 응답 12.3ms
  [Info] 포트 [  8080]
  버려진 로그를 위해 Snapshot을 서식화한 횟수: 0
  [Info] 상태 = 3
```

### 08_dollar_crate — `$crate`와 `::`로 시작하는 경로

```sh
cargo run -p er-ch28-macros-judiciously --example 08_dollar_crate
```

```text
1. 이 크레이트 안에서는 crate::도 $crate::도 같은 곳을 가리킴
  [1] main에서 track!
  [2] main에서 track_crate!
  [3] handlers::login에서 track!
  [4] handlers::login에서 track_crate!
2. 부르는 쪽의 Result 별칭에 가려지지 않는 경로
  port(" 8080 ") = Ok(8080)
  port("80a")    → `text`을(를) u16(으)로 읽지 못함: invalid digit found in string
```

### projects/tabular — derive 매크로와 `$crate`

독립 워크스페이스입니다. 처음 빌드할 때 crates.io에서 `syn`, `quote`, `proc-macro2`와 그 의존성을 내려받습니다. `cargo`의 진행 메시지(`Compiling …`)는 싣지 않았습니다.

```text
projects/tabular/
├── Cargo.toml          # [workspace] members = tabular, tabular_derive, app / exclude = user_fails
├── tabular/            # Columns 트레이트, render, row!(와 비교용 비권장 매크로 셋), derive 재내보내기
├── tabular_derive/     # proc-macro = true: #[derive(Columns)]와 보조 속성 #[column(..)]
├── app/                # 손으로 쓴 impl과 derive 비교, row! 사용
└── user_fails/         # 일부러 컴파일에 실패하는 별도 워크스페이스: tabular의 매크로를 다른 크레이트에서 부름
```

```sh
cd appd-a/ch28-macros-judiciously/projects/tabular
cargo run
```

```text
1. 손으로 쓴 impl: 필드를 더한 뒤 NAMES를 고치지 않음
제목 | 저자 | 연도
러스트 첫걸음 | 한빛나 | 2024 | 412   ← 칸 4개, 머리글 3개
소유권 이야기 | 도윤서 | 2022 | 288   ← 칸 4개, 머리글 3개

2. #[derive(Columns)]
제목 | 저자 | 연도 | 쪽수
소유권 이야기 | 도윤서 | 2022 | 288
매크로 공방 | 서지안 | 2025 | 356
러스트 첫걸음 | 한빛나 | 2024 | 412
Book::NAMES = ["제목", "저자", "연도", "쪽수"]

3. row!: 식의 글자가 머리글이 됨
books.len() | total_pages | newest
3 | 1056 | 2025
```

derive 판의 행은 표에서 뺀 `shelf`(서가 번호) 순서로 정렬했습니다. `cargo test`는 `tabular`의 단위 테스트 3개를 통과하고(비권장 매크로 셋을 포함한 네 매크로가 모두 통과하는 테스트 포함), `cargo clippy --all-targets -- -D warnings`와 `cargo fmt --check`도 통과합니다.

```sh
cd user_fails
cargo build     # 실패 (종료 코드 101)
```

```text
error[E0425]: cannot find function `__format_row` in the crate root
  --> src/main.rs:14:20
   |
14 |     println!("{}", row_crate!(total, average)); // E0425: crate::는 이 크레이트(user_fails)의 루트
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in the crate root
   |
   = note: this error originates in the macro `row_crate` (in Nightly builds, run with -Z macro-backtrace for more info)

error[E0603]: function `format_row` is private
  --> src/main.rs:15:20
   |
15 |     println!("{}", row_private!(total, average)); // E0603: $crate로 찾아갔지만 비공개 함수
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ private function
   |
note: the function `format_row` is defined here
  --> …/projects/tabular/tabular/src/lib.rs:47:1
   |
47 | fn format_row(pairs: &[(&str, &dyn Display)]) -> String {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: this error originates in the macro `row_private` (in Nightly builds, run with -Z macro-backtrace for more info)

error[E0425]: cannot find function `__format_row` in this scope
  --> src/main.rs:13:20
   |
13 |     println!("{}", row_bare!(total, average)); // E0425: 이 크레이트에서 __format_row를 찾음
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope
   |
   = note: this error originates in the macro `row_bare` (in Nightly builds, run with -Z macro-backtrace for more info)
```

12행의 `row!`에는 에러가 없습니다. 세 에러 모두 부른 줄의 매크로 호출 전체를 가리킬 뿐, 원인이 있는 `tabular`의 매크로 본문은 보여 주지 않습니다. 쓰는 쪽은 자기 코드에 잘못이 없는데도 고칠 수 없는 에러를 받습니다. 실습이 끝나면 두 디렉터리에서 `cargo clean`으로 `target/`을 지웁니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 챕터 디렉터리에서 컴파일합니다. 실패하면 아무 파일도 생기지 않지만, 고친 뒤 확인할 때를 위해 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd appd-a/ch28-macros-judiciously
rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0425_hygiene_local_variable.rs
```

### 4.1 매크로가 부르는 쪽의 지역 변수를 쓰려 함 — E0425

[`compile_fail/e0425_hygiene_local_variable.rs`](compile_fail/e0425_hygiene_local_variable.rs)의 `add_to_total!(5)`는 본문에서 `total += $x`를 합니다.

```text
error[E0425]: cannot find value `total` in this scope
  --> compile_fail/e0425_hygiene_local_variable.rs:14:9
   |
12 | macro_rules! add_to_total {
13 |     ($x:expr) => {
14 |         total += $x
   |         ^^^^^ not found in this scope
...
20 |     add_to_total!(5);
   |     ---------------- in this macro invocation
   |
help: an identifier with the same name exists, but is not accessible due to macro hygiene
  --> compile_fail/e0425_hygiene_local_variable.rs:19:13
   |
19 |     let mut total = 0;
   |             ^^^^^
```

**원인.** 매크로 본문의 지역 변수 이름은 매크로를 정의한 곳에서 찾습니다(2.1절). 부르는 쪽의 `total`은 이름이 같아도 다른 변수입니다. help가 정확히 위생 때문이라고 짚어 줍니다.

**해결.** 고칠 대상을 인자로 받습니다(`add_to!(total, 5)` → `$target += $x`). 지침 4에 따라 `add_to!(&mut total, 5)`처럼 빌림을 드러내면 더 좋습니다. 위생은 몰래 고치는 매크로를 막아 주는 안전장치입니다.

### 4.2 경로 없이 쓴 함수 이름 — E0425

[`compile_fail/e0425_item_path_at_call_site.rs`](compile_fail/e0425_item_path_at_call_site.rs)의 `track!`은 `metrics` 모듈 안에서 정의되어 본문에 `record($event)`라고만 씁니다. `metrics` 안에서 부른 곳은 컴파일되고, `handlers` 모듈에서 부른 곳만 실패합니다.

```text
error[E0425]: cannot find function `record` in this scope
  --> compile_fail/e0425_item_path_at_call_site.rs:22:13
   |
20 |     macro_rules! track {
21 |         ($event:expr) => {
22 |             record($event)
   |             ^^^^^^ not found in this scope
...
33 |         crate::track!("login"); // 다른 모듈: E0425
   |         ---------------------- in this macro invocation
   |
help: consider importing this function
   |
32 +     use crate::metrics::record;
   |
```

**원인.** 4.1절과 같은 에러 코드지만 방향이 반대입니다. 함수 이름은 위생의 대상이 아니라서 매크로를 **부른 곳**(`handlers`)에서 찾습니다.

**해결.** help(부른 곳에 `use`를 더하기)는 따르지 않습니다. 이 호출 하나만 고칠 뿐, 매크로를 쓰는 모든 곳과 다른 크레이트에 같은 `use`를 요구하게 됩니다. 매크로 본문을 `$crate::metrics::record($event)`로 고칩니다(지침 6, 예제 08).

### 4.3 프래그먼트에 맞지 않는 인자 — `no rules expected`

[`compile_fail/no_rules_expected_ident.rs`](compile_fail/no_rules_expected_ident.rs)는 첫 인자를 `ident`로만 받는 `at_least_ident!`(예제 04)에 `sensor.read()`를 넘깁니다.

```text
error: no rules expected `.`
  --> compile_fail/no_rules_expected_ident.rs:29:35
   |
21 | macro_rules! at_least_ident {
   | --------------------------- when calling this macro
...
29 |     let r = at_least_ident!(sensor.read(), 5);
   |                                   ^ no rules expected this token in macro call
   |
note: while trying to match `,`
  --> compile_fail/no_rules_expected_ident.rs:22:18
   |
22 |     ($value:ident, $min:expr) => {
   |                  ^
```

**원인.** 에러 코드가 없는 매크로 규칙 맞추기 에러입니다. `sensor`까지는 `ident`로 맞았고, 다음 토큰 `.`이 규칙의 `,` 자리와 맞지 않았습니다. note가 어느 규칙의 어느 토큰과 맞춰 보다 실패했는지 알려 줍니다.

**해결.** 의도한 거절입니다. 부르는 쪽에서 `let reading = sensor.read();`로 먼저 바인드합니다. 규칙이 여러 개인 매크로에서는 note가 마지막으로 시도한 규칙만 가리키므로, 갈래를 적게 유지하는 것이 에러를 읽기 쉽게 합니다.

### 4.4 계산을 매크로 안에서 하면 에러가 정의를 가리킴 — E0369 / E0308

[`compile_fail/e0369_logic_inside_macro.rs`](compile_fail/e0369_logic_inside_macro.rs)는 2.9절에서 본 대로 두 에러를 함께 냅니다.

```text
error[E0369]: cannot multiply `&str` by `{integer}`
  --> compile_fail/e0369_logic_inside_macro.rs:15:15
   |
13 | / macro_rules! percent {
14 | |     ($part:expr, $whole:expr) => {
15 | |         $part * 100 / $whole
   | |               ^ --- {integer}
16 | |     };
17 | | }
   | |_- this error originates in the macro `percent` which comes from the expansion of the macro `report`
...
39 |       report!("진행률", done, total); // E0369, 매크로 정의 안을 가리킴
   |       ------------------------------
   |       |                 |
   |       |                 &str
   |       in this macro invocation

error[E0308]: mismatched types
  --> compile_fail/e0369_logic_inside_macro.rs:40:25
   |
32 |         println!("{}: {}%", $label, percent_of($part, $whole))
   |                                     ---------- arguments to this function are incorrect
...
40 |     report_thin!("진행률", done, total); // E0308, 이 줄의 `done`을 가리킴
   |                            ^^^^ expected `u32`, found `&str`
```

**원인.** 매크로의 인자에는 타입이 없습니다. 타입 검사는 펼친 뒤에 일어나므로, 계산이 매크로 안에 있으면 에러도 그 안에서 납니다.

**해결.** 계산을 타입이 적힌 함수로 옮기고 매크로는 넘기기만 합니다(`report_thin!`). 에러가 부른 줄의 인자를 가리키고, 함수 시그니처가 기대하는 타입을 알려 줍니다.

### 4.5 그 밖의 실수

모두 이 툴체인에서 실제로 컴파일해 확인했습니다(작은 파일, 또는 예제·프로젝트의 사본을 고쳐서).

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| 정의보다 위에서 `macro_rules!` 매크로를 부름 | `` cannot find macro `greet` in this scope ``, note `a macro with the same name exists, but it appears later` | 정의를 위로 옮김(Part 0 ch19 2.18절) | 2.1 |
| `#[macro_export]` 매크로를 정의한 모듈 경로로 부름 | E0433 `` cannot find `hit` in `metrics` `` | `crate::hit!`처럼 크레이트 루트 경로로 | 2.7, 08 |
| 숨은 `return Err`가 있는 매크로를 `main`에서 부름 | E0308 `` expected `()`, found `Result<_, String>` ``, 매크로 정의 안을 가리킴 | `Result`를 돌려주는 함수와 `?` | 2.5, 05 |
| 빌림을 드러내야 하는 매크로에 `&mut`를 빠뜨림 | E0308 `` expected `&mut [u32]`, found `Vec<{integer}>` `` | `normalize_v2!(&mut scores)` | 2.5, 06 |
| 본문에 `Result<$ty, String>`이라고 쓴 매크로를 `Result` 별칭이 있는 모듈에서 부름 | E0107 `type alias takes 1 generic argument but 2 generic arguments were supplied` | `::core::result::Result`로 | 2.7, 08 |
| `format_args!`에 넘기는 매크로에 인자를 덜 줌 | `2 positional arguments in format string, but there is 1 argument` | 서식 문자열과 인자 개수를 맞춤 | 2.6, 07 |
| 매크로 본문에 `crate::`를 씀 | clippy `crate_in_macro_def` 경고, 다른 크레이트에서 E0425 | `$crate::` | 2.7, 08, `user_fails` |
| 펼친 코드가 비공개 함수를 부름 | 다른 크레이트에서만 E0603 | `#[doc(hidden)] pub fn __이름` | 2.7, `user_fails` |
| `#[derive(Columns)]`를 열거형에 붙임 | `Columns는 구조체에만 derive할 수 있습니다`, 열거형 이름을 가리킴 | 구조체에만 씀 | 2.8 |
| 모르는 보조 속성 키(`#[column(hide)]`) | `` column 속성에는 `skip`이나 `rename = "…"`만 쓸 수 있습니다 ``, `hide`를 가리킴 | `skip` 또는 `rename` | 2.8 |
| `Display`가 없는 필드(`Vec<String>`)에 derive | E0277 `` the trait bound `Vec<String>: ToString` is not satisfied ``, 그 필드를 가리킴 | 필드를 `#[column(skip)]`하거나 `Display` 타입으로 | 2.8 |
| derive 없이 보조 속성만 씀 | `` cannot find attribute `column` in this scope ``, note `` `column` is an attribute that can be used by the derive macro `Columns` `` | `#[derive(Columns)]`를 붙임 | 2.8 |

## 5. 연습 문제

### 연습 1. 표에 한 줄, 손으로 쓴 판에 한 줄

[`02_single_source.rs`](examples/02_single_source.rs)에 요금제 `Enterprise`(코드 `"enterprise"`, 이름 "엔터프라이즈", 월 199,000원)를 더하세요. `v2`에서는 몇 줄을 고쳐야 하나요? `v1`에서 모든 곳을 맞게 고치려면 몇 곳을 고쳐야 하고, 그중 컴파일러가 알려 주는 곳은 어디인가요?

<details>
<summary>힌트</summary>

`v2`는 `plans!` 표에 `Enterprise { code: "enterprise", label: "엔터프라이즈", price: 199_000 },` 한 줄이면 `ALL`, `code`, `label`, `monthly_price`, `from_code`가 함께 바뀝니다. `v1`은 열거형, `ALL`(배열 길이 포함), `code`, `from_code` 네 곳에 `main`의 검사 목록까지 다섯 곳입니다. 열거형에 배리언트만 더하고 빌드하면 E0004(`` `v1::Plan::Enterprise` not covered ``)가 `code`의 `match`만 가리킵니다. `code`를 고치면 에러는 사라지고 `` variant `Enterprise` is never constructed `` 경고가 남는데, 이것은 `ALL`이나 `main`의 목록 중 한 곳에 넣기만 해도 사라집니다. `from_code`가 맞는지는 실행해 봐야 압니다.

</details>

### 연습 2. 놀라지 않는 `setting!`과 그 매크로가 필요한지

[`05_hidden_control_flow.rs`](examples/05_hidden_control_flow.rs)의 `setting!`을 `return` 없이 `Result<&str, String>`으로 평가되도록 고쳐, 부르는 쪽이 `setting!(map, key)?`처럼 쓰게 하세요. `print_all_v1`의 동작은 어떻게 바뀌나요? 다 고친 뒤, 이 매크로가 함수 `setting`보다 나은 점이 남아 있나요?

<details>
<summary>힌트</summary>

본문을 `` { let key = $key; $map.get(key).copied().ok_or_else(|| format!("`{key}` 설정이 없습니다")) } ``로 바꿉니다. `print_all_v1`의 `let value = setting!(map, key);`는 이제 `?`를 붙여야 컴파일되고, 그 `?`가 "여기서 끝날 수 있다"를 보여 줍니다. `?` 대신 `match`를 쓰면 `print_all_v2`와 같은 동작도 고를 수 있습니다. 남은 매크로는 함수 `setting`과 하는 일이 같으므로 지침 1에 따라 함수로 충분합니다. 식의 글자를 에러 메시지에 넣고 싶다면(`stringify!($key)`) 그때 다시 매크로가 이유를 갖습니다.

</details>

### 연습 3. `user_fails`를 `tabular` 쪽만 고쳐 통과시키기

[`projects/tabular/user_fails`](projects/tabular/user_fails)는 그대로 두고 [`tabular/src/lib.rs`](projects/tabular/tabular/src/lib.rs)의 비권장 매크로 셋만 고쳐 `user_fails`가 빌드되게 하세요. `tabular`의 테스트도 계속 통과해야 합니다. 고친 뒤 `#[expect(clippy::crate_in_macro_def, …)]`는 어떻게 되나요?

<details>
<summary>힌트</summary>

세 매크로의 본문을 모두 `$crate::__format_row(..)`로 바꿉니다. `row_private!`는 본문 대신 `format_row`를 `#[doc(hidden)] pub`으로 바꿔도 됩니다. 고친 `row_crate!`에서는 더 이상 경고가 나지 않으므로 `#[expect]`가 충족되지 않아 `this lint expectation is unfulfilled` 경고(`unfulfilled_lint_expectations`)가 납니다. 속성도 지웁니다. `tabular`를 고친 뒤에는 `user_fails`에서 `cargo build` 전에 따로 할 일이 없습니다(path 의존성은 바뀐 소스를 다시 컴파일합니다). 출력은 같은 표가 네 번입니다.

</details>

### 연습 4. 튜플 구조체에도 `#[derive(Columns)]`

[`tabular_derive`](projects/tabular/tabular_derive/src/lib.rs)가 튜플 구조체(`struct Pair(u32, String);`)도 받도록 고치세요. 열 이름은 `"0"`, `"1"`처럼 필드 번호로 합니다. `app`에 `#[derive(Columns)] struct Pair(u32, String);`를 더해 `render(&[Pair(1, "하나".to_string())])`가 `0 | 1` 머리글과 `1 | 하나`를 출력하는지 확인하세요.

<details>
<summary>힌트</summary>

`let Fields::Named(fields) = … else`를 `match &data.fields`로 바꾸고 `Fields::Unnamed(fields)` 갈래를 더합니다. `fields.unnamed.iter().enumerate()`로 번호 `i`를 얻고, 필드 접근에는 `syn::Index::from(i)`를 씁니다(`quote_spanned!` 안에서 `&self.#index`). 열 이름은 `i.to_string()`입니다. 보조 속성 해석은 이름 붙은 필드와 같은 코드를 함께 쓰도록 함수로 빼면 깔끔합니다. `Fields::Unit`은 기존 에러를 그대로 둡니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 28: Use macros judiciously](https://effective-rust.com/macros.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- The Reference, [Macros By Example](https://doc.rust-lang.org/reference/macros-by-example.html): Metavariables, Repetitions, Scoping·exporting·importing(`#[macro_export]`), Hygiene(혼합 위생, `$crate`, `$crate`와 가시성)
- The Reference, [Procedural Macros](https://doc.rust-lang.org/reference/procedural-macros.html): 세 종류의 정의 모양, derive 매크로의 보조 속성, 절차적 매크로의 위생(절대 경로를 쓰라는 권고)
- 표준 라이브러리 문서: `format_args!`와 `std::fmt::Arguments`(컴파일할 때 검사, 힙 할당 없음), `std::panic::Location`(1.46.0). 로컬 문서(`rustc --print sysroot` 아래 `share/doc/rust/html`)로 확인했습니다.
- [`syn` 3.0.10 문서](https://docs.rs/syn/3.0.10/syn/): `DeriveInput`, `parse_macro_input!`, `Attribute::parse_nested_meta`, `Error`
- [`quote` 1.0.47 문서](https://docs.rs/quote/1.0.47/quote/): `quote!`의 반복 문법(`#(...),*`), `quote_spanned!`
- [The Little Book of Rust Macros](https://lukaswirth.dev/tlborm/): `macro_rules!`를 깊이 다루는 문서입니다. Part 0 ch19가 링크한 옛 주소(veykril.github.io/tlborm)는 이 주소로 넘어갑니다.
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.18~2.20절과 `projects/hello_macro`: `macro_rules!` 문법과 derive 매크로 프로젝트의 기본 구성
