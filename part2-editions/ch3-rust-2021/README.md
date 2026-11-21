# ch3. 2021 에디션의 변경

2021 에디션(공식 이름 "Rust 2021")은 Rust 1.56.0(2021-10-21)과 함께 안정화된 세 번째 에디션입니다. 앞 파트에서는 이 에디션이 가져온 변화를 결과만 짚고 지나갔습니다.

- [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.11절: "2021 에디션부터는 `[a, b, c].into_iter()`가 원소 값을 내놓는다." 같은 챕터 2.3절은 클로저가 "변수마다" 캡처 방식을 고른다고 설명했습니다.
- [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md): "2021 에디션부터 `dyn`은 생략할 수 없다."
- [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.2절: 프렐류드 표의 "`TryFrom`, `TryInto`, `FromIterator` — 2021부터"
- [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md): "2021 에디션부터 `panic!` 매크로의 페이로드는 언제나 `&'static str` 또는 `String`"
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)와 [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md): `c"hello"` 리터럴

이 챕터는 각 변화가 2018 에디션에서는 어땠고, 왜 "에디션"이라는 경계를 넘어야만 들어올 수 있었는지를 **같은 코드를 두 에디션으로 컴파일해서** 확인합니다. Edition Guide 4절의 열 가지 항목을 모두 다룹니다.

- 패키지: `p2-ch3-rust-2021`
- 기준 문서: [Edition Guide 4. Rust 2021](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64)
- 실습 자료는 세 종류입니다. `examples/`는 이 저장소의 에디션(2024)으로 컴파일되며 2021에서 들어와 지금도 이어지는 동작을 보여 줍니다. `projects/`는 같은 코드를 2018과 2021로 컴파일해 비교하는 독립 프로젝트이고, `compile_fail/`은 `rustc --edition`으로 에디션을 바꿔 가며 컴파일해 보는 파일입니다.
- 앞 챕터: Part 2 [`ch1-editions-overview`](../ch1-editions-overview/README.md)(에디션은 크레이트마다 고르고, 에디션이 다른 크레이트도 함께 링크된다), [ch2](../ch2-rust-2015-2018/README.md)(2018 에디션의 변화, raw 식별자 `r#`). ch1의 2.4절은 배열 `into_iter()`와 `use` 없는 `u8::try_from`을 "에디션의 효과"의 예로 한 줄씩 보였고, 이 챕터는 그 원리와 2018/2021 비교를 자세히 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 2021 에디션의 변화 열 가지를 "2018 코드가 컴파일 에러가 되는 것", "컴파일은 되지만 동작이 바뀌는 것", "Cargo의 동작이 바뀌는 것"으로 나누고, 각각이 왜 에디션 경계 없이는 들어올 수 없었는지 설명할 수 있다.
2. 같은 소스를 2018과 2021로 컴파일해 결과가 달라지는 지점(배열 `.into_iter()`, 클로저의 drop 순서, 인자가 하나인 `panic!`, 프렐류드와 이름이 겹치는 메서드, 접두사 토큰, `$p:pat`)을 실행해 확인하고, 메서드 탐색 순서·캡처 단위·토큰 나누기 규칙으로 그 차이를 설명할 수 있다.
3. 2018 코드를 2021로 옮길 때 항목별로 손으로 고칠 수 있다(`.iter()`, `let _ = &x;`, `panic!("{}", x)`와 `panic_any`, 트레이트 이름을 적은 호출, 공백 넣기, `$x:pat_param`, `dyn`과 `..=`).
4. 기능 리졸버 1과 2가 의존성의 기능을 합치는 방식의 차이를 설명하고, `edition`과 `resolver` 필드의 관계와 가상 매니페스트에 `resolver`를 직접 적어야 하는 이유를 말할 수 있다.
5. 2021 에디션부터 쓸 수 있게 된 문법(`c"..."`/`cr"..."`, raw 라이프타임 `'r#이름`, `$p:pat`에 넘기는 `A | B`)과 프렐류드의 새 트레이트를 현재 에디션의 코드에서 쓸 수 있다.

## 2. 핵심 개념

### 2.1 이 챕터의 지도

| 절 | 항목 | 2018 코드를 2021로 컴파일하면 | 마이그레이션 린트 (기본 수준) | 확인하는 곳 |
| --- | --- | --- | --- | --- |
| 2.2 | 프렐류드에 `TryFrom`/`TryInto`/`FromIterator` 추가 | 같은 이름의 메서드와 모호해져 에러가 나거나, 다른 메서드가 조용히 호출됨 | `rust_2021_prelude_collisions` (allow) | 예제 01, `same_code` 4, 4.2절 |
| 2.3 | 기본 기능 리졸버가 2로 | 의존성의 기능이 덜 켜져 빌드가 깨지거나 동작이 바뀜 | 없음 | `feature_resolver` |
| 2.4 | 배열의 `IntoIterator` | `arr.into_iter()`가 `&T` 대신 `T`를 내놓음 | `array_into_iter` (warn) | 예제 02, `same_code` 1 |
| 2.5 | 클로저의 분리 캡처 | drop 시점이 바뀌고, 클로저가 `Send`·`Clone`을 잃거나 얻음 | `rust_2021_incompatible_closure_captures` (allow) | 예제 03, `same_code` 2, 4.3절 |
| 2.6 | 패닉 매크로 일관성 | 인자가 하나인 `panic!`의 메시지가 바뀌거나 에러 | `non_fmt_panics` (warn) | 예제 04, `same_code` 3, 4.4절 |
| 2.7 | 예약 문법(접두사) | `z"hey"` 같은 모양이 에러, 매크로 입력의 토큰 수가 바뀜 | `rust_2021_prefixes_incompatible_syntax` (allow) | 예제 05, `same_code` 5, 4.5절 |
| 2.8 | C 문자열 리터럴 | 2018의 `c "..."` 두 토큰이 `c"..."` 한 토큰이 됨 | 2.7과 같음 | 예제 05 |
| 2.9 | raw 라이프타임 | 2018의 `'r`, `#`, `a` 세 토큰이 `'r#a` 한 토큰이 됨 | 2.7과 같음 | 예제 05, `same_code` 5 |
| 2.10 | 경고에서 에러로 승격 | `dyn` 없는 트레이트 객체, `...` 범위 패턴이 에러 | `bare_trait_objects`, `ellipsis_inclusive_range_patterns` (warn) | 4.1절 |
| 2.11 | `macro_rules`의 `:pat`과 `\|` | `$x:pat \|` 규칙이 에러, `$p:pat`이 받는 입력이 바뀜 | `rust_2021_incompatible_or_patterns` (allow) | 예제 06, `same_code` 6, 4.6절 |

**깨지는 방식은 세 가지입니다.** 컴파일 에러로 드러나는 변화는 고칠 곳을 컴파일러가 알려 주므로 오히려 다루기 쉽습니다. 위험한 것은 **두 에디션에서 모두 컴파일되는데 결과가 다른** 경우입니다. 배열 `.into_iter()`, 클로저의 drop 순서, 인자가 하나인 `panic!`, `&self`를 받는 사용자 메서드와 `TryInto`의 충돌, 매크로 입력의 토큰 수, `$p:pat`을 쓰는 매크로가 여기에 속합니다. [`projects/same_code`](projects/same_code)는 이 여섯 가지를 한 소스 파일에 모아 2018과 2021로 각각 실행합니다(3절).

**마이그레이션 린트.** 표의 린트는 모두 에디션 호환성 린트 그룹 `rust-2021-compatibility`에 들어 있습니다. 기본 수준이 warn인 넷(`array_into_iter`, `non_fmt_panics`, `bare_trait_objects`, `ellipsis_inclusive_range_patterns`)은 2018 코드를 평소처럼 컴파일해도 경고로 나옵니다. allow인 넷은 그룹을 직접 켜야 보입니다. 기본 수준은 `rustc -W help`로 확인했습니다.

```sh
rustc --edition 2018 -W rust-2021-compatibility 파일.rs
```

`cargo fix --edition`은 이 그룹을 켜고 린트가 제안하는 자동 수정을 적용하는 명령입니다. 절차는 [Part 2 ch6](../ch6-migration-practice/README.md)에서 다룹니다. 이 챕터에서는 각 항목을 손으로 고치는 방법을 익힙니다. 린트가 모든 변화를 잡지는 못한다는 점도 기억해 둡니다. 3절의 `same_code`에서 6번 구역(`$p:pat`)의 동작 변화는 어떤 린트도 알려 주지 않았습니다.

**절마다 같은 순서로 설명합니다.** 2018에서는 어땠나 → 2021에서 무엇이 바뀌었나 → 왜 에디션 변경이어야 했나(기존 코드가 깨지는 지점) → 2018 코드를 옮길 때 어떻게 고치나.

### 2.2 프렐류드에 `TryFrom`, `TryInto`, `FromIterator`가 들어왔습니다

**2018에서는.** 2015와 2018 에디션은 같은 프렐류드(`std::prelude::v1`)를 씁니다. `TryFrom`/`TryInto`(1.34.0에 안정화)와 `FromIterator`는 이 목록에 없어서 `use`가 필요했습니다. 빠뜨리면 이렇게 나옵니다(핵심 줄만 옮겼습니다).

```text
error[E0599]: no method named `try_into` found for type `i32` in the current scope
  = note: 'std::convert::TryInto' is included in the prelude starting in Edition 2021
help: trait `TryInto` which provides `try_into` is implemented but not in scope; perhaps you want to import it
```

**2021에서는.** `std::prelude::rust_2021`은 v1에 이 세 트레이트를 더한 목록입니다([Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.2절의 표). 그래서 `u8::try_from(200)`, `300_u16.try_into()`, `Vec::from_iter(..)`를 `use` 없이 씁니다([`01_prelude_2021.rs`](examples/01_prelude_2021.rs)의 1번 구역).

**왜 에디션이 필요했나.** 프렐류드에 **타입이나 함수**를 더하는 것은 기존 코드를 깨지 않습니다. 직접 정의하거나 `use`로 가져온 이름이 프렐류드의 이름보다 우선하기 때문입니다. 하지만 **트레이트**를 더하면 그 트레이트의 메서드가 모든 모듈에서 메서드 탐색의 후보가 됩니다. 메서드 탐색은 다음 순서로 진행합니다([Part 0 ch5](../../part0-getting-started/ch5-structs/README.md) 2.10절의 자동 참조가 이 과정입니다).

1. 수신자 타입을 `T`, `&T`, `&mut T` 순서로 하나씩 넓혀 가며 봅니다(역참조로 이어지는 단계는 여기서는 생략합니다).
2. 한 단계 안에서는 고유 메서드(타입의 `impl` 블록에 직접 정의한 메서드)를 먼저, 그다음 스코프에 있는 트레이트의 메서드를 찾습니다.
3. 후보를 찾은 첫 단계에서 멈춥니다. 같은 단계에 트레이트 메서드 후보가 둘 이상이면 모호하다는 에러입니다.

`TryInto`는 `From`의 포괄 구현을 거쳐 사실상 모든 타입에 구현되어 있고([Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 2.4절), `try_into(self)`는 수신자를 값으로 받으므로 언제나 첫 단계의 후보가 됩니다. 그래서 2018 코드에 `try_into`라는 메서드가 이미 있으면 사용자 쪽 메서드의 모양에 따라 결과가 갈립니다.

| 2018 코드의 `try_into` | 2021에서 `x.try_into()` | 확인한 곳 |
| --- | --- | --- |
| 사용자 트레이트의 `fn try_into(self)` | 첫 단계에 후보가 둘이라 E0034 | 4.2절 |
| 사용자 트레이트의 `fn try_into(&self)` | 첫 단계의 `TryInto::try_into`가 이겨서 **조용히** 표준 메서드를 호출 | 예제 01의 2번 구역, `same_code` 4번 구역 |
| 고유 메서드 `fn try_into(self)` | 같은 단계에서 고유 메서드가 이기므로 변화 없음 | 예제 01의 3번 구역 |
| `dyn Trait`의 메서드(트레이트를 가져오지 않아도 부를 수 있음) | `Box<dyn Trait>`의 `TryInto`가 잡혀 E0283 | 4.7절 |

두 번째 줄이 가장 위험합니다. `same_code`의 4번 구역은 300을 0..=255로 잘라 내는 `Clamp::try_into(&self)`를 정의하고 `x.try_into()`를 부릅니다.

```rust
trait Clamp {
    fn try_into(&self) -> Result<u8, String>;
}
let x: i32 = 300;
let by_method: Result<u8, _> = x.try_into();
// 2018: Ok(255)                              ← Clamp::try_into
// 2021: Err(TryFromIntError(PosOverflow))    ← TryInto::try_into
```

`Result<u8, _>`의 에러 타입을 `_`로 두었기 때문에 두 에디션 모두 컴파일되고, 값만 달라집니다. 2018에서 `rust_2021_prelude_collisions` 린트를 켜면 이 줄을 짚어 줍니다(핵심 줄만 옮겼습니다). 다만 경고 문구는 "2021에서 에러가 된다"고 말하지만, 1.100.0에서 실제로 2021로 컴파일해 보면 에러 없이 표준 메서드가 호출되었습니다.

```text
warning: trait method `try_into` will become ambiguous in Rust 2021
99 |     let by_method: Result<u8, _> = x.try_into();
   |                                    ^^^^^^^^^^^^ help: disambiguate the associated function: `Clamp::try_into(&x)`
   = warning: this is accepted in the current edition (Rust 2018) but is a hard error in Rust 2021!
```

**옮길 때.** 사용자 트레이트의 메서드를 부르려던 것이라면 트레이트 이름을 경로에 적습니다. `Clamp::try_into(&x)` 또는 완전 정규화 문법 `<i32 as Clamp>::try_into(&x)`입니다(예제 01의 2번 구역). 연관 함수도 마찬가지로, Edition Guide의 예인 `<Vec<i32>>::from_iter(None)`은 2021에서 E0034이고 `<Vec<i32> as MyTrait<()>>::from_iter(None)`으로 고칩니다. `dyn Trait`에서 부르는 경우는 `(&*f).try_into()`처럼 수신자를 `&dyn Trait`로 바꾸면 트레이트 객체의 메서드가 먼저 잡힙니다. 메서드 이름을 바꾸는 것도 방법입니다.

2024 에디션은 프렐류드에 `Future`와 `IntoFuture`를 더했고, 같은 종류의 충돌을 잡는 `rust_2024_prelude_collisions` 린트가 있습니다([Part 2 ch5](../ch5-rust-2024-ecosystem/README.md)).

### 2.3 기본 기능 리졸버가 2가 되었습니다

**기능과 기능 통합.** Cargo의 기능(feature)은 크레이트가 선택해서 켤 수 있게 만든 코드 묶음입니다. 크레이트는 `Cargo.toml`의 `[features]`에 기능 이름을 적고 코드에서 `#[cfg(feature = "이름")]`이나 `cfg!(feature = "이름")`으로 확인하며, 그 크레이트를 쓰는 쪽은 의존성 선언에 `features = ["이름"]`을 적어 켭니다([Part 0 ch16](../../part0-getting-started/ch16-async/README.md)에서 tokio의 `features = ["full"]`을 보았습니다). 같은 의존성을 여러 곳에서 다른 기능으로 요청하면 Cargo는 그 크레이트를 한 번만 빌드하면서 요청된 기능을 모두 켭니다. 이것을 기능 통합(feature unification)이라고 하고, 어떤 기능을 켤지 정하는 Cargo의 알고리즘을 기능 리졸버라고 부릅니다. 기능 자체는 [Part 3 ch5](../../part3-cargo/ch5-features/README.md)에서 자세히 다룹니다.

**2018에서는.** 기능 리졸버 1은 의존성이 **어디에서** 요청되었든 기능을 모두 합칩니다. 테스트에서만 쓰는 dev-dependency의 기능도, 다른 운영체제에서만 쓰는 의존성의 기능도 일반 빌드에 섞입니다.

**2021에서는.** `edition = "2021"`이면 `resolver = "2"`를 적은 것과 같습니다. 리졸버 2는 1.51.0에 선택 사항으로 들어왔고, 2021 에디션에서 기본값이 되었습니다. 리졸버 2는 다음 세 경우에 기능을 합치지 않습니다(Cargo Book의 "Feature resolver version 2" 절).

- 지금 빌드하지 않는 대상 플랫폼 전용 의존성(`[target.'cfg(windows)'.dependencies]` 등)의 기능은 켜지 않습니다.
- 빌드 의존성(`[build-dependencies]`)과 절차적 매크로가 켠 기능은 일반 의존성과 합치지 않습니다.
- dev-dependency의 기능은 테스트나 예제처럼 dev-dependency가 필요한 대상을 빌드할 때만 켭니다.

[`projects/feature_resolver`](projects/feature_resolver)가 세 번째 경우를 재현합니다. 라이브러리 `flags`는 `verbose` 기능 하나를 가지고, `app2018`과 `app2021`은 `flags`를 일반 의존성으로는 기능 없이, dev-dependency로는 `verbose`를 켜서 씁니다. 두 패키지의 `Cargo.toml`은 이름과 `edition`만 다르고 소스 파일은 똑같습니다.

```toml
[dependencies]
flags = { path = "../flags" }

[dev-dependencies]
flags = { path = "../flags", features = ["verbose"] }
```

```text
app2018 main: flags의 verbose = true     ← 리졸버 1: dev-dependency의 기능이 실행 파일에도 섞임
app2018 예제: flags의 verbose = true
app2021 main: flags의 verbose = false    ← 리졸버 2: 실행 파일에는 켜지지 않음
app2021 예제: flags의 verbose = true     ← 예제는 dev-dependency를 쓰는 대상이라 켜짐
```

**리졸버는 워크스페이스에 하나입니다.** 의존성 그래프 전체를 한 번에 풀기 때문에, 리졸버 버전은 워크스페이스 루트(워크스페이스가 아니면 그 패키지)의 설정 하나만 쓰고 의존성의 설정은 무시합니다. 리졸버의 기본값은 ch1 2.4절에서 본 "도구의 기본값" 가운데 하나입니다. `flags`는 2024 에디션(리졸버 3을 뜻함)이지만 `app2018` 안에서는 리졸버 1로 풀립니다. 그래서 이 프로젝트는 `app2018`과 `app2021`을 하나의 워크스페이스로 묶지 않고 각자 `[workspace]`를 가진 별도의 루트로 두었습니다. `resolver` 필드를 직접 적으면 에디션이 뜻하는 값보다 우선합니다. `app2018`의 `[workspace]`에 `resolver = "2"`를 적으면 main도 `false`가 되고, `app2021`의 `[package]`에 `resolver = "1"`을 적으면 main도 `true`가 되는 것을 확인했습니다.

**가상 매니페스트에는 직접 적어야 합니다.** `[package]` 없이 `[workspace]`만 있는 루트 `Cargo.toml`을 가상 매니페스트라고 하고([ch1](../ch1-editions-overview/README.md) 2.7절), 이런 워크스페이스를 가상 워크스페이스(virtual workspace)라고 부릅니다. 이 저장소의 루트가 그렇고, `projects/same_code`도 그렇습니다. 루트 패키지가 없으니 에디션으로 리졸버를 정할 수 없어서, 적지 않으면 멤버가 2021 에디션이어도 리졸버 1이 쓰입니다. 1.100.0의 Cargo는 이 경우 경고를 냅니다(`same_code`의 `resolver` 줄을 지우고 확인했습니다).

```text
warning: virtual workspace defaulting to `resolver = "1"` despite one or more workspace members being on edition 2021 which implies `resolver = "2"`
  |
  = note: to keep the current resolver, specify `workspace.resolver = "1"` in the workspace root's manifest
  = note: to use the edition 2021 resolver, specify `workspace.resolver = "2"` in the workspace root's manifest
```

**왜 에디션이 필요했나.** 리졸버는 언어가 아니라 Cargo의 동작이지만, Cargo도 패키지의 `edition`을 읽어 기본값을 정합니다. 리졸버 2로 바꾸면 그동안 우연히 켜져 있던 기능이 꺼집니다. 어떤 의존성이 "다른 누군가가 켜 줄 것"이라고 기대한 기능을 쓰고 있었다면, 그 기능에만 있는 함수가 사라져 빌드가 깨지거나 `cfg!`에 따라 동작이 바뀝니다. 또 같은 크레이트가 기능 조합마다 따로 빌드되어 빌드 시간이 늘 수 있습니다. 그래서 새 에디션을 고를 때 함께 바뀌도록 했습니다.

**옮길 때.** 자동 수정은 없습니다. 실제로 쓰는 기능은 자기 `Cargo.toml`의 의존성 선언에 직접 적는 것이 원칙입니다. 어떤 기능이 어디서 켜지는지는 `cargo tree`로 봅니다. 다음은 두 앱에서 일반 의존성(`-e normal`)만 따라가며 패키지와 켜진 기능(`{f}`)을 출력한 것입니다(경로는 줄여 적었습니다).

```text
$ cargo tree -e normal -f '{p} {f}'        # app2018에서
app2018 v0.1.0 (…/feature_resolver/app2018)
└── flags v0.1.0 (…/feature_resolver/flags) verbose

$ cargo tree -e normal -f '{p} {f}'        # app2021에서
app2021 v0.1.0 (…/feature_resolver/app2021)
└── flags v0.1.0 (…/feature_resolver/flags)
```

Edition Guide는 이 밖에 `cargo tree -d`(두 번 빌드되는 패키지), `cargo tree -e features -i 패키지`(기능이 어디서 흘러 들어오는지)를 소개하고, `cargo fix --edition`이 꺼지게 될 기능을 보고한다고 설명합니다. 2024 에디션은 기본값을 `resolver = "3"`(의존성의 `rust-version`을 고려하는 리졸버)으로 바꿨고, 이 저장소의 루트 `Cargo.toml`도 `resolver = "3"`입니다([Part 2 ch5](../ch5-rust-2024-ecosystem/README.md)).

### 2.4 배열이 값으로 `IntoIterator`를 구현합니다

**2018에서는.** 1.53.0 전까지 `IntoIterator`는 배열의 참조(`&[T; N]`, `&mut [T; N]`)에만 구현되어 있었습니다. 그래서 `for e in [1, 2, 3]`은 에러였고, `arr.into_iter()`는 메서드 탐색이 첫 단계(`[T; N]`)에서 후보를 못 찾고 둘째 단계(`&[T; N]`)로 넘어가 참조의 구현을 골랐습니다. 이름은 `into_iter`인데 실제로는 `iter()`와 같았던 것입니다.

**1.53.0의 절충.** 배열에 `impl IntoIterator for [T; N]`을 더하면 첫 단계에서 새 구현이 잡혀 기존 `arr.into_iter()`의 뜻이 바뀝니다. 그렇다고 트레이트 구현을 2021 에디션에만 둘 수도 없습니다. 에디션은 크레이트마다 다르지만 표준 라이브러리와 타입은 모든 크레이트가 함께 쓰기 때문입니다. 그래서 1.53.0은 구현을 모든 에디션에 넣고, 2015·2018 에디션 코드에서 **배열에 `.into_iter()`를 메서드 호출 문법으로 부를 때만** 컴파일러가 새 구현이 없는 것처럼 참조의 구현을 고르게 했습니다. 다른 문법은 모든 에디션에서 값을 내놓습니다. 2018과 2021로 직접 컴파일해 항목 타입을 확인한 결과입니다.

| 쓰는 모양 | 2018 | 2021 이후 |
| --- | --- | --- |
| `arr.into_iter()` | `&i32` | `i32` |
| `arr.iter()` | `&i32` | `&i32` |
| `IntoIterator::into_iter(arr)` | `i32` | `i32` |
| `for x in arr` | `i32` | `i32` |
| `(0..3).zip(arr)` | `(i32, i32)` | `(i32, i32)` |

[`02_array_into_iter.rs`](examples/02_array_into_iter.rs)가 2021 이후의 동작을, `same_code`의 1번 구역이 두 에디션의 차이를 보여 줍니다. 어느 쪽 뜻을 쓸지는 `.into_iter()`라는 토큰을 쓴 크레이트의 에디션이 정합니다. 그래서 2015 크레이트의 매크로 본문에 있는 `.into_iter()`는 2024 크레이트에서 펼쳐져도 참조를 내놓습니다([ch1](../ch1-editions-overview/README.md) 2.8절의 `first_item!`).

**왜 에디션이 필요했나.** 항목 타입이 `&T`에서 `T`로 바뀌면 결과는 세 갈래입니다. `arr.into_iter().map(|x| x * 2)`처럼 참조에도 값에도 같은 연산이 있으면 아무 일도 없습니다(예제 02의 2번 구역은 2018에서도 같은 결과입니다). `Vec<&T>`를 모으던 코드는 타입 에러가 납니다. 반대로 예제 02의 3번 구역 `names.into_iter().collect::<Vec<String>>()`은 2018에서 E0277(`a value of type Vec<String> cannot be built from an iterator over elements of type &String`)입니다. 가장 곤란한 것은 두 에디션에서 모두 컴파일되면서 결과만 달라지는 경우로, `same_code`의 1번 구역은 항목의 타입 이름을 출력해 그 차이를 드러냅니다. `arr.into_iter()`를 쓴 기존 코드가 많았기 때문에 구현을 추가하는 것만으로 해결할 수 없었습니다.

**옮길 때.** `array_into_iter` 린트는 1.41.0부터 모든 에디션에서 기본으로 경고합니다(Edition Guide). 2018에서 나오는 경고는 이렇습니다(핵심 줄만 옮겼습니다).

```text
warning: this method call resolves to `<&[T; N] as IntoIterator>::into_iter` (due to backwards compatibility), but will resolve to `<[T; N] as IntoIterator>::into_iter` in Rust 2021
  = warning: this changes meaning in Rust 2021
  = note: `#[warn(array_into_iter)]` (part of `#[warn(rust_2021_compatibility)]`) on by default
help: use `.iter()` instead of `.into_iter()` to avoid ambiguity
help: or use `IntoIterator::into_iter(..)` instead of `.into_iter()` to explicitly iterate by value
```

참조를 원했다면 `.iter()`로 바꿉니다. 모든 에디션에서 `&T`입니다. 값을 원했다면 `IntoIterator::into_iter(arr)`로 쓰고, 2021로 옮긴 뒤 `arr.into_iter()`로 줄입니다. Clippy도 2018에서 같은 줄을 `into_iter_on_ref`(`this .into_iter() call is equivalent to .iter()`)로 잡습니다.

2024 에디션은 `Box<[T]>`에 같은 일을 했습니다. 2021 에디션에서 `boxed.into_iter()`는 아직 `&T`를 내놓고 `boxed_slice_into_iter` 린트가 경고합니다([Part 2 ch5](../ch5-rust-2024-ecosystem/README.md)). `IntoIterator` 자체는 [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.11절과 [Part 1 ch5](../../part1-standard-lib/ch5-iterators/README.md)에서 다뤘습니다.

### 2.5 클로저가 필드 단위로 캡처합니다(분리 캡처)

**2018에서는.** [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.3절은 "컴파일러가 본문이 그 변수로 무엇을 하는지 보고, 그 일을 할 수 있는 가장 약한 방식을 변수마다 고른다"고 설명했습니다. 2018 에디션까지는 이 "변수마다"가 문자 그대로였습니다. 본문이 `a.x`만 써도 클로저는 `a` 전체를 캡처했습니다.

**2021에서는.** 캡처의 단위가 본문이 실제로 쓰는 **경로**(`pair.0`, `job.name`)가 되었습니다. 캡처 방식(불변 대여, 가변 대여, 이동)도 경로마다 고릅니다. 이것을 분리 캡처(disjoint capture)라고 합니다. Edition Guide는 경우에 따라 쓰는 것보다 넓게 캡처하기도 한다고 적고 있습니다. 예를 들어 원시 포인터를 역참조하는 `*(px.0)`은 `px.0`까지만 캡처합니다(4.3절의 에러 메시지가 그 예입니다).

분리 캡처 덕분에 2018에서 거절되던 코드가 컴파일됩니다. [`03_disjoint_capture.rs`](examples/03_disjoint_capture.rs)의 1~3번 구역이며, 같은 파일을 2018로 컴파일해 에러를 확인했습니다.

| 예제 03의 구역 | 2021 이후 | 2018 |
| --- | --- | --- |
| 1. 클로저가 `pair.0`을 가변으로 빌린 동안 `pair.1`을 바꿈 | 됨 | E0499 (`cannot borrow pair.1 as mutable more than once at a time`) |
| 2. `point.0`을 옮긴 뒤 `point.1`을 읽는 클로저 | 됨 | E0382 (`borrow of partially moved value: point`) |
| 3. `Rc` 필드가 있는 `job`의 `job.name`만 다른 스레드로 | 됨 | E0277 (`` `Rc<Vec<u8>>` cannot be sent between threads safely ``) |

**왜 에디션이 필요했나.** 캡처하는 값이 바뀌면 이미 컴파일되던 코드에서 두 가지가 달라집니다.

첫째, **drop 시점**입니다. 클로저가 값을 가져가면 그 값은 클로저와 함께 버려집니다. 2021에서는 클로저가 가져간 필드만 클로저와 함께 버려지고, 나머지 필드는 원래 변수의 스코프가 끝날 때 버려집니다. `same_code`의 2번 구역은 `t.0`만 옮기는 클로저를 부릅니다.

```rust
let t = (Noisy("t.0"), Noisy("t.1"));
let consume = || drop(t.0);
consume();
println!("    consume() 호출 끝");
```

2018에서는 `drop: t.0`, `drop: t.1`, `consume() 호출 끝` 순서로, 2021에서는 `drop: t.0`, `consume() 호출 끝`, `drop: t.1` 순서로 출력됩니다(3절의 실행 결과). 2018의 클로저는 `t` 전체를 가져가므로 `consume()`이 끝날 때 남은 `t.1`도 함께 버립니다. 메모리가 언제 해제되는지만 다르다면 상관없지만, `Drop`에 부수 효과(잠금 해제, 파일 닫기, 로그)가 있으면 프로그램의 동작이 바뀝니다.

둘째, **클로저가 구현하는 트레이트**입니다. 클로저는 캡처한 값이 모두 `Clone`이면 `Clone`이고, 모두 `Send`이면 `Send`입니다(`Sync`, `UnwindSafe` 같은 자동 트레이트도 같습니다). 캡처하는 값이 바뀌면 이 결과도 바뀝니다. 예제 03의 3번 구역처럼 새로 `Send`가 되기도 하지만, 반대로 잃기도 합니다. 원시 포인터를 스레드로 넘기려고 구조체로 감싸고 `unsafe impl Send`를 붙인 코드가 대표적입니다. 2018의 클로저는 감싼 구조체를 캡처해서 `Send`였지만, 2021의 클로저는 안쪽 필드인 원시 포인터만 캡처해서 `Send`가 아닙니다(4.3절).

**옮길 때.** `rust_2021_incompatible_closure_captures` 린트(allow)를 켜면 drop 순서나 트레이트 구현이 바뀌는 클로저를 짚어 줍니다. 다음은 `same_code`의 2018 멤버에 린트 그룹을 켰을 때의 경고입니다(3절의 명령).

```text
warning: changes to closure capture in Rust 2021 will affect drop order
  --> e2018/../shared/main.rs:82:23
   |
82 |         let consume = || drop(t.0);
   |                       ^^      --- in Rust 2018, this closure captures all of `t`, but in Rust 2021, it will only capture `t.0`
...
85 |     }
   |     - in Rust 2018, `t` is dropped here, but in Rust 2021, only `t.0` will be dropped here as part of the closure
   |
help: add a dummy let to cause `t` to be fully captured
   |
82 |         let consume = || { let _ = &t; drop(t.0) };
   |                          +++++++++++++           +
```

`help:`의 `let _ = &t;`를 "더미 let"이라고 부릅니다. `&t`는 계산해야 하는 표현식이므로 클로저가 `t` 전체를 캡처하게 되고, 2018과 같은 동작이 됩니다(예제 03의 5번 구역). 비슷해 보이는 `let _ = t;`는 아무것도 캡처하지 않습니다. `_` 패턴은 오른쪽 값을 읽지 않고, `t`는 계산할 것이 없는 "자리"(변수)이기 때문입니다. `cargo fix --edition`은 동작이 바뀔 수 있는 곳에 더미 let을 보수적으로 넣으므로, 넣어 준 줄 중 상당수는 지워도 됩니다. 이 판단은 [Part 2 ch6](../ch6-migration-practice/README.md)에서 다룹니다.

### 2.6 `panic!`이 언제나 서식 문자열을 처리합니다

**2018에서는.** `panic!`은 인자가 둘 이상일 때만 서식 처리를 했습니다. 인자가 하나면 그 값을 들여다보지도 않고 페이로드(패닉과 함께 전달되는 값)로 넘겼습니다. 그래서 문자열이 아닌 값(`panic!(404)`)도 받았고, `panic!("{}")`은 `{}`라는 글자를 메시지로 썼습니다. 1.58.0에서 서식 문자열이 변수 이름을 직접 캡처하게 되자(`println!("{name}")`) 문제가 커졌습니다. 같은 모양의 `panic!("{name}")`이 2018에서는 `{name}`이라는 글자를 출력했기 때문입니다.

**2021에서는.** `panic!`은 `println!`처럼 첫 인자를 언제나 서식 문자열로 처리합니다. `assert!(조건, ..)`의 메시지도 같습니다. Edition Guide에 따르면 그전까지 조금씩 달랐던 `core::panic!`과 `std::panic!`도 2021에서 같아졌습니다. 다음 표는 2018 열과 2021 열을 각각 컴파일해 실행한 결과입니다([`04_panic_format.rs`](examples/04_panic_format.rs), `same_code`의 3번 구역).

| 코드 (`name = "러스트"`) | 2018 | 2021 이후 |
| --- | --- | --- |
| `panic!("{name} 설정 없음")` | `&str "{name} 설정 없음"` (경고) | `String "러스트 설정 없음"` |
| `panic!("중괄호 {{}}")` | `&str "중괄호 {{}}"` (경고) | `&str "중괄호 {}"` |
| `assert!(false, "{name}")` | `"{name}"` 그대로 (경고) | 서식 처리됨 |
| `panic!("{}")` | `&str "{}"` (경고) | 에러: `1 positional argument in format string, but no arguments were given` |
| `panic!(404)`, `panic!(msg)` | 그 값이 페이로드 (경고) | 에러: `format argument must be a string literal` (4.4절) |

2021 이후의 페이로드는 서식 인자가 없을 때처럼 컴파일할 때 문자열이 정해지면 `&'static str`, 그렇지 않으면 `String`입니다. 그 밖의 타입을 페이로드로 넘기는 방법은 함수 `std::panic::panic_any(값)` 하나입니다([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)).

**왜 에디션이 필요했나.** 같은 소스가 다른 메시지를 내고, `panic!(값)`처럼 쓰던 코드는 컴파일되지 않게 되기 때문입니다. 특히 메시지가 바뀌는 쪽은 컴파일러가 막아 주지 않습니다.

**옮길 때.** `non_fmt_panics` 린트(warn)가 2018에서 해당 줄을 모두 짚어 줍니다. 인자가 하나인 리터럴에 중괄호가 있으면 이렇게 나옵니다(핵심 줄만 옮겼습니다).

```text
warning: panic message contains an unused formatting placeholder
  = note: this message is not used as a format string when given without arguments, but will be in Rust 2021
help: add the missing argument
help: or add a "{}" format string to use the message literally
```

고치는 방법은 세 가지입니다. 문자열을 글자 그대로 쓰려면 `panic!("{}", msg)`, 중괄호를 글자로 쓰려면 `{{` `}}`, 문자열이 아닌 값을 넘기려면 `std::panic::panic_any(값)`입니다. `panic_any`는 매크로가 아니라 함수입니다. 참고로 문자열이 아닌 페이로드로 패닉하면 기본 훅은 메시지를 보여 줄 수 없습니다. Edition Guide는 이때의 출력을 `panicked at 'Box<Any>'`로 적고 있지만, 1.100.0에서는 `thread 'main' (스레드 ID) panicked at 파일:줄:칸:` 다음 줄에 `Box<dyn Any>`가 출력되었습니다(스레드 ID는 실행할 때마다 다릅니다).

### 2.7 예약 문법: 식별자에 붙은 접두사

**2018에서는.** 소스는 먼저 토큰으로 나뉩니다. 2018에서 `z"hey"`는 식별자 `z`와 문자열 `"hey"`, 두 토큰이었습니다. `k#async`는 `k`, `#`, `async` 세 토큰입니다. 이런 모양은 보통 코드에서는 문법에 맞지 않아 쓸 일이 없지만, 토큰을 그대로 받는 매크로에는 문제없이 넘어갔습니다. 키워드 뒤에 붙여 쓴 `match"가" { .. }`는 키워드 `match`와 문자열이라서 보통 코드로도 컴파일되었습니다.

**2021에서는.** 식별자 바로 뒤에 `"`, `'`, `#`이 붙은 모양(`접두사"..."`, `접두사'.'`, `접두사#식별자`, `접두사#123`)과 `'접두사#`을 예약했습니다. 접두사가 될 수 있는 것은 아무 식별자이며, 이미 뜻이 있는 `b`, `r`, `br`과 1.77.0에 생긴 `c`, `cr`(2.8절)만 리터럴로 읽고 나머지는 토큰을 나누는 단계에서 에러입니다. 키워드도 예외가 아니어서 `match"가" { .. }`는 2021부터 `` prefix `match` is unknown `` 에러입니다. [`05_reserved_prefixes.rs`](examples/05_reserved_prefixes.rs)의 1번 구역은 토큰 수를 세는 매크로로 이것을 보여 줍니다. 2018 열은 같은 매크로를 2018로 컴파일해 확인했습니다.

| 매크로 입력 | 2018 | 2021 이후 |
| --- | --- | --- |
| `c"hi"`, `cr"hi"` | 2개 | 1개 (C 문자열 리터럴) |
| `c "hi"` | 2개 | 2개 |
| `b"hi"`, `r#"raw"#` | 1개 | 1개 (2015부터 있던 리터럴) |
| `'r#a` | 3개 | 1개 (raw 라이프타임, 2.9절) |
| `'r #a` | 3개 | 3개 |
| `z"hey"`, `k#ident`, `f"x"` | 2개, 3개, 2개 | 에러: `` prefix `z` is unknown `` 등 (4.5절) |

**왜 에디션이 필요했나.** 목적은 앞으로 새 리터럴이나 키워드 문법을 **에디션을 기다리지 않고** 더할 자리를 미리 마련하는 것입니다. Edition Guide는 아직 약속하지 않은 후보로 `k#keyword`(아직 키워드가 아닌 에디션에서 미래의 키워드 쓰기), `f"..."`(서식 문자열), `s"..."`(`String` 리터럴)를 듭니다. 실제로 2021 에디션 안에서 이 자리에 C 문자열 리터럴(1.77.0)과 raw 라이프타임(1.83.0)이 들어왔습니다. 하지만 예약하는 것 자체가 기존 매크로 입력을 다르게 나누므로 에디션 경계가 필요했습니다. `same_code`의 5번 구역은 같은 `count_tt!(c"hi")`가 2018에서 2, 2021에서 1을 내는 것을 보여 줍니다.

**옮길 때.** 접두사와 그 뒤 사이에 공백을 넣습니다(`z "hey"`, `k #ident`). `rust_2021_prefixes_incompatible_syntax` 린트(allow)가 넣을 자리를 알려 줍니다. 다음은 `same_code`의 2018 멤버에서 나온 경고입니다(핵심 줄만 옮겼습니다).

```text
warning: this is parsed as a C string literal in Rust 2021 and onward
106 |         count_tt!(c"hi"),
    |                   ^^^^^
    = warning: this changes meaning in Rust 2021
help: consider inserting whitespace here to avoid this
106 |         count_tt!(c "hi"),
    |                    +
```

2024 에디션은 `#"..."` 모양을 더 예약했습니다([Part 2 ch4](../ch4-rust-2024-language/README.md)).

### 2.8 C 문자열 리터럴 `c"..."`, `cr"..."`

**2018에서는.** `c"hello"`는 식별자 `c`와 문자열, 두 토큰이므로 표현식 자리에서는 문법 에러입니다. 1.100.0은 에디션을 알려 줍니다(핵심 줄만 옮겼습니다).

```text
error: expected one of `!`, `.`, `::`, `;`, `?`, `else`, `{`, or an operator, found `"hello"`
  = note: you may be trying to write a c-string literal
  = note: c-string literals require Rust 2021 or later
```

**2021에서는.** 1.77.0부터 `c"..."`는 끝에 0 바이트가 붙은 `&'static CStr`입니다. 중간에 `\0`을 쓰면 컴파일 에러이고, `\xff` 같은 바이트 이스케이프와 `\u{..}`를 쓸 수 있습니다. `cr"..."`은 역슬래시를 이스케이프로 처리하지 않는 raw C 문자열이며(예제 05의 2번 구역), `cr#"..."#`처럼 `#`으로 감싸면 안에 따옴표를 쓸 수 있습니다. 타입과 쓰는 법은 [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 2.4절에서 다뤘습니다.

**왜 2021 이후에만 쓸 수 있나.** 이 리터럴은 2021 에디션이 나온 뒤에 생겼지만 새 에디션이 필요하지 않았습니다. 2.7절의 예약 덕분에 2021 에디션에서 `c"`는 이미 "접두사가 붙은 리터럴" 자리였기 때문입니다. 반대로 2015·2018 에디션에서는 `c "..."` 두 토큰이라는 뜻이 이미 있으므로, 그 뜻을 바꾸지 않으려고 이 리터럴을 들이지 않았습니다. 2018 코드를 옮길 때 신경 쓸 곳은 `c"…"`를 두 토큰으로 받던 매크로 호출뿐이며, 2.7절과 같은 린트가 짚어 줍니다.

### 2.9 raw 라이프타임 `'r#이름`

**2018에서는.** `'r#a`는 `'r`, `#`, `a` 세 토큰이라 라이프타임 자리에서는 에러입니다(`` expected one of `,`, `:`, or `>`, found `#` ``).

**2021에서는.** 1.83.0부터 `'r#이름`을 라이프타임으로 쓸 수 있습니다. [Part 2 ch2](../ch2-rust-2015-2018/README.md)의 raw 식별자 `r#match`가 식별자에 하는 일을 라이프타임에 합니다. `'r#a`는 `'a`와 같은 라이프타임이고, 키워드와 같은 이름도 쓸 수 있습니다. 예제 05의 3번 구역은 `'r#gen`을 씁니다.

```rust
fn longer<'r#gen>(a: &'r#gen str, b: &'r#gen str) -> &'r#gen str {
    if a.len() >= b.len() { a } else { b }
}
```

**왜 필요한가.** 라이프타임에는 키워드를 쓸 수 없습니다(`lifetimes cannot use keyword names`). 새 에디션이 키워드를 늘리면 그 이름의 라이프타임을 쓰던 코드가 깨지므로, 마이그레이션 도구가 고쳐 쓸 문법이 필요합니다. 실제로 `'gen`은 2021에서는 되지만, `gen`이 키워드가 된 2024 에디션에서는 에러이고 `'r#gen`으로 바꿔야 합니다(직접 확인했습니다).

**옮길 때.** 2018에서 `'r#foo`를 세 토큰으로 매크로에 넘기던 코드만 고치면 됩니다. Edition Guide는 `'r# foo`처럼 띄우라고 안내하지만, 1.100.0에서 `'r# a`는 2021 에디션에서 `` prefix `'r` is unknown `` 에러였습니다. 컴파일러와 린트가 제안하는 대로 `'r #a`처럼 `'r` 바로 뒤를 띄웁니다(예제 05의 1번 구역에서 3개로 나뉩니다).

### 2.10 경고에서 에러로 승격된 두 린트

2018 에디션에서 경고만 내던 두 문법이 2021부터 에러입니다. 둘 다 더는 권장하지 않는(deprecated) 문법이라 오래전부터 경고가 났으므로, 경고 없는 2018 코드라면 고칠 것이 없습니다.

| 문법 | 2018 (경고) | 2021 이후 (에러) | 고치기 |
| --- | --- | --- | --- |
| `dyn` 없는 트레이트 객체 `Box<Shape>` | `trait objects without an explicit dyn are deprecated` | E0782 `expected a type, found a trait` (4.1절) | `Box<dyn Shape>` |
| `...` 범위 패턴 `90...100` | `` `...` range patterns are deprecated `` | E0783 `` `...` range patterns are deprecated `` | `90..=100` |

2018의 경고에는 `this is accepted in the current edition (Rust 2018) but is a hard error in Rust 2021!`이라는 줄이 붙습니다. **왜 에디션이 필요했나**는 간단합니다. 경고를 에러로 바꾸면 경고를 무시하던 코드가 컴파일되지 않기 때문입니다. `dyn`이 생긴 배경은 [Part 2 ch2](../ch2-rust-2015-2018/README.md)에서, 트레이트 객체와 범위 패턴은 [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)과 [Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md)에서 다뤘습니다.

### 2.11 `macro_rules`의 `$p:pat`이 `A | B`를 받습니다

**2018에서는.** 1.53.0부터 `|`를 패턴 안 어디에나 쓸 수 있게 되었습니다(`Some(1 | 2)`). 그러나 매크로 규칙의 `$p:pat`은 그대로 맨 바깥의 `|`를 받지 않았습니다. 이 성질을 이용해 `$($p:pat)|+`처럼 `|`를 패턴 사이의 구분자로 쓰는 매크로가 있었기 때문입니다(Edition Guide는 그런 예로 1.51.0 시절의 `matches!`를 듭니다).

**2021에서는.** `$p:pat`이 `1 | 2` 전체를 패턴 하나로 받습니다. 그래서 `$x:pat` 바로 뒤에 `|`가 오는 규칙은 뜻이 모호해져 거절됩니다(4.6절). 예전 동작이 필요하면 1.53.0에 추가된 `$p:pat_param`을 씁니다. 모든 에디션에서 쓸 수 있습니다. [`06_pat_in_macros.rs`](examples/06_pat_in_macros.rs)의 3번 구역이 두 지정자의 차이를 보여 줍니다.

```text
    whole!(1 | 2..=5 | _) → ["1 | 2..=5 | _"]
    split!(1 | 2..=5 | _) → ["1", "2..=5", "_"]
```

`whole!`은 규칙이 `($p:pat)` 하나라서 입력 전체를 패턴 하나로 받았고, `split!`은 `($($p:pat_param)|+)`라서 `|`마다 끊어 패턴 셋을 받았습니다.

**에디션은 매크로를 정의한 크레이트의 것입니다.** 매크로 규칙을 어떻게 해석할지는 매크로를 부른 크레이트가 아니라 정의한 크레이트의 에디션이 정합니다. 매크로 본문의 토큰이 정의한 크레이트의 에디션으로 읽히는 것([ch1](../ch1-editions-overview/README.md) 2.8절)과 같은 원리입니다. `same_code`의 6번 구역에서 2021 크레이트가 부른 `macros2018::classify!(1 | 2)`는 2018 규칙대로 "규칙 2"로 갑니다.

**왜 에디션이 필요했나.** 두 가지가 바뀝니다. `$x:pat |`처럼 쓴 매크로 정의는 컴파일 에러가 되고, 규칙이 여럿인 매크로는 같은 입력이 **다른 규칙**으로 가서 조용히 다른 코드로 펼쳐질 수 있습니다. 예제 06의 `classify!(1 | 2)`는 2018에서 "규칙 2", 2021에서 "규칙 1"입니다.

**옮길 때.** `rust_2021_incompatible_or_patterns` 린트(allow)는 `$x:pat` 뒤에 `|`가 오는 정의를 짚고 `pat_param`을 제안합니다. 4.6절의 파일을 `rustc --edition 2018 -W rust-2021-compatibility`로 컴파일한 결과입니다(핵심 줄만 옮겼습니다).

```text
warning: the meaning of the `pat` fragment specifier is changing in Rust 2021, which may affect this macro
  --> compile_fail/pat_fragment_followed_by_or.rs:14:6
   |
14 |     ($x:pat | $y:pat) => { // → `$x:pat` is followed by `|`
   |      ^^^^^^ help: use pat_param to preserve semantics: `$x:pat_param`
   |
   = warning: this is accepted in the current edition (Rust 2018) but is a hard error in Rust 2021!
```

다만 이 린트는 `classify!`처럼 받는 규칙이 바뀌는 경우는 짚지 않았습니다. 규칙이 여럿이고 앞 규칙에 `$p:pat`이 있는 매크로는 직접 살펴보고, 2018의 동작이 필요하면 `pat_param`으로 바꿉니다. 프래그먼트 지정자 전체는 Part 9 `ch2-macros`에서, 2024 에디션의 `expr` 프래그먼트 변경은 [Part 2 ch4](../ch4-rust-2024-language/README.md)에서 다룹니다.

### 2.12 이 챕터에서 다루지 않는 것

| 주제 | 다루는 곳 |
| --- | --- |
| 에디션이라는 제도, 크레이트마다 다른 에디션을 함께 링크하기, 매크로 본문의 에디션 | Part 2 [`ch1-editions-overview`](../ch1-editions-overview/README.md) |
| 2018 에디션의 변화(경로와 모듈, `async`/`await`/`dyn` 키워드), raw 식별자 | [Part 2 ch2](../ch2-rust-2015-2018/README.md) |
| 2024 에디션의 언어 변화(`gen` 키워드, `#"..."` 예약, `expr` 프래그먼트, 임시값 범위 등) | [Part 2 ch4](../ch4-rust-2024-language/README.md) |
| 2024 에디션의 프렐류드(`Future`/`IntoFuture`), `resolver = "3"`, `Box<[T]>`의 `IntoIterator` | [Part 2 ch5](../ch5-rust-2024-ecosystem/README.md) |
| `cargo fix --edition`의 절차와 자동 수정이 넣어 주는 코드 | [Part 2 ch6](../ch6-migration-practice/README.md) |
| 기능, 의존성 해석의 상세 | [Part 3 ch5](../../part3-cargo/ch5-features/README.md), [ch2](../../part3-cargo/ch2-dependencies/README.md) |
| 매크로 프래그먼트 지정자 전체 | Part 9 `ch2-macros` |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_prelude_2021.rs`](examples/01_prelude_2021.rs) | `use` 없이 쓰는 세 트레이트, 이름이 겹치는 사용자 트레이트, 고유 메서드의 우선순위 | 2, 3, 5 |
| [`02_array_into_iter.rs`](examples/02_array_into_iter.rs) | 모양별 항목 타입, `Copy` 배열, 원소를 값으로 옮겨 내기, 배열을 `IntoIterator` 자리에 넘기기 | 2, 3, 5 |
| [`03_disjoint_capture.rs`](examples/03_disjoint_capture.rs) | 새로 컴파일되는 세 경우, 필드만 캡처할 때의 drop 시점, 더미 let | 2, 3 |
| [`04_panic_format.rs`](examples/04_panic_format.rs) | 변수 이름 캡처, 중괄호, `"{}"`, `panic_any`, `assert!`의 메시지 | 2, 3 |
| [`05_reserved_prefixes.rs`](examples/05_reserved_prefixes.rs) | 접두사와 토큰 수, `c"..."`/`cr"..."`, raw 라이프타임 | 2, 5 |
| [`06_pat_in_macros.rs`](examples/06_pat_in_macros.rs) | `$p:pat`과 `A \| B`, `pat_param`으로 나누기 | 2, 3, 5 |
| [`projects/same_code`](projects/same_code) | 같은 소스를 2018과 2021로 컴파일한 결과 비교(여섯 구역) | 1, 2 |
| [`projects/feature_resolver`](projects/feature_resolver) | 에디션만 다른 두 패키지에서 dev-dependency의 기능이 섞이는지 | 4 |
| [`compile_fail/`](compile_fail) | 2018에서는 통과하고 2021부터 실패하는 코드 6개(4절) | 1, 3 |

`examples/`의 명령은 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 곧바로 끝나고 실행할 때마다 같은 출력이 나옵니다. 예제의 "2018에서는" 주석은 해당 예제 파일(또는 그 일부)을 `rustc --edition 2018`로 컴파일해 확인한 결과입니다.

### 01_prelude_2021 — 프렐류드의 새 트레이트와 이름 충돌

```sh
cargo run -p p2-ch3-rust-2021 --example 01_prelude_2021
```

```text
1. use 없이 쓰는 세 트레이트
    u8::try_from(200) = Ok(200)
    300u16.try_into() = Err(TryFromIntError(PosOverflow))
    Vec::from_iter    = [1, 4, 9, 16], String::from_iter = 러스트
2. 이름이 겹치는 사용자 트레이트
    x.try_into()               = Err(TryFromIntError(PosOverflow))
    Clamp::try_into(&x)        = Ok(255)
    <i32 as Clamp>::try_into() = Ok(255)
3. 고유 메서드는 트레이트 메서드보다 먼저 찾아집니다
    Celsius(36.6).try_into() = Ok(37)
```

- **1번 구역**: 2018로 컴파일하면 이 구역의 네 줄이 모두 E0599입니다. `300_u16`을 쓴 것은 파일 위의 `Clamp`가 `i32`에만 구현되어 있어 충돌하지 않게 하려는 것입니다.
- **2번 구역**: 첫 줄이 2018에서는 `Ok(255)`입니다(2.2절). 트레이트 이름을 적은 두 줄은 어느 에디션에서나 `Clamp`의 메서드를 부릅니다.
- **3번 구역**: `Celsius`의 고유 메서드 `try_into(self)`는 `TryInto::try_into(self)`와 같은 단계에 있지만 고유 메서드가 이깁니다. 2018로 컴파일해도 `Ok(37)`입니다.

### 02_array_into_iter — 배열의 `IntoIterator`

```sh
cargo run -p p2-ch3-rust-2021 --example 02_array_into_iter
```

```text
1. 반복자가 내놓는 항목의 타입
    arr.into_iter()                → i32
    arr.iter()                     → &i32
    IntoIterator::into_iter(&arr)  → &i32
    IntoIterator::into_iter(arr)   → i32
2. Copy 배열은 into_iter() 뒤에도 쓸 수 있습니다
    doubled = [20, 40, 60], arr = [10, 20, 30]
3. Copy가 아닌 원소는 복제 없이 옮겨 낼 수 있습니다
    owned = ["민수", "지영", "현우", "서연"]
4. IntoIterator를 받는 곳에는 배열을 그대로 넘깁니다
    zip = [('a', 1), ('b', 2), ('c', 3)]
    extend + chain = [0, 1, 2, 3, 4]
```

- **1번 구역**: 반복자를 소비하지 않고 `Iterator::Item`의 타입 이름만 출력합니다. 2018로 컴파일하면 첫 줄만 `&i32`로 바뀌고 나머지는 같습니다. `IntoIterator::into_iter(&arr)`는 2018의 `arr.into_iter()`가 실제로 부르던 것입니다.
- **2번 구역**: `[i32; 3]`은 `Copy`라서 `into_iter()`가 복사본을 가져가고 `arr`는 그대로 남습니다. 이 구역은 2018에서도 출력이 같습니다. `&i32`에도 `* 2`가 있어서입니다.
- **3번 구역**: 2018에서는 E0277입니다(2.4절). `iter()`로는 `String`을 복제하지 않고 옮길 수 없습니다.

### 03_disjoint_capture — 클로저의 분리 캡처

```sh
cargo run -p p2-ch3-rust-2021 --example 03_disjoint_capture
```

```text
1. 서로 다른 필드를 동시에 빌리기
    ("왼쪽!", "오른쪽?")
2. 필드 하나를 옮긴 뒤 다른 필드를 쓰는 클로저
    남은 필드: y좌표
    옮겨 간 필드: x좌표
3. Send인 필드만 다른 스레드로 보내기
    백업 작업을 다른 스레드에서 시작
    cache는 그대로: [1, 2, 3]
4. 필드 하나만 캡처하면 버려지는 시점이 달라집니다
    안쪽 블록 끝
    drop: t.0
    바깥 블록 끝
    drop: t.1
5. let _ = &t; 로 변수 전체를 캡처하게 만들기
    안쪽 블록 끝
    drop: u.0
    drop: u.1
    바깥 블록 끝
```

- **1~3번 구역**: 2018에서는 각각 E0499, E0382, E0277입니다(2.5절의 표). 3번 구역의 스레드는 결과를 돌려받아 `main`에서 출력하므로 출력 순서가 일정합니다.
- **4번 구역**: `move` 클로저가 `t.0`만 가져가므로, 안쪽 블록이 끝날 때 클로저와 함께 `t.0`만 버려집니다. 2018에서는 "안쪽 블록 끝" 바로 뒤에 `t.0`과 `t.1`이 함께 버려집니다.
- **5번 구역**: 더미 let 하나로 4번 구역의 2018 동작을 2021에서 재현합니다.

### 04_panic_format — 패닉 매크로의 서식 처리

```sh
cargo run -p p2-ch3-rust-2021 --example 04_panic_format
```

```text
1. 서식 문자열 안의 변수 이름이 값으로 바뀝니다
    String "러스트 설정을 찾을 수 없습니다"
2. 중괄호를 글자로 쓰려면 {{ }}로 씁니다
    &str "빈 자리표시자 {}는 글자입니다"
3. 이미 만든 문자열을 그대로 쓰려면 "{}"를 앞에 둡니다
    String "{key}는 치환하지 않습니다"
4. 문자열이 아닌 값은 panic_any로 넘깁니다
    ErrorCode(404)
5. assert!의 메시지도 서식 문자열입니다
    String "러스트은(는) ASCII가 아닙니다"
```

- 패닉 훅을 아무것도 하지 않는 클로저로 바꿔 두었으므로 표준 에러에는 아무것도 나오지 않습니다. 모든 패닉은 `catch_unwind`로 붙잡으므로 종료 코드는 0입니다.
- 같은 파일을 2018로 컴파일하면 경고 세 개(`unused formatting placeholder` 둘, `contains braces` 하나)와 함께 1·5번 구역이 `&str "{name} …"`, 2번 구역이 `&str "빈 자리표시자 {{}}는 글자입니다"`로 바뀝니다. 3·4번 구역은 같습니다.

### 05_reserved_prefixes — 접두사, C 문자열 리터럴, raw 라이프타임

```sh
cargo run -p p2-ch3-rust-2021 --example 05_reserved_prefixes
```

```text
1. 접두사가 붙은 모양은 토큰 하나입니다
    c"hi"      → 토큰 1개
    c "hi"     → 토큰 2개
    b"hi"      → 토큰 1개
    r#"raw"#   → 토큰 1개
    'r#a       → 토큰 1개
    'r #a      → 토큰 3개
    z "hey"    → 토큰 2개
2. C 문자열 리터럴 c"..."와 cr"..."
    "안녕" → [236, 149, 136, 235, 133, 149, 0]
    "caf\xe9" → [99, 97, 102, 233]
    cr"C:\temp" == c"C:\\temp": true
3. raw 라이프타임 'r#이름
    longer = 에디션 가이드
    head = 2021
```

- **1번 구역**: 왼쪽은 `stringify!`로 입력을 그대로 보여 준 것입니다. 2018과의 비교는 2.7절의 표에 있습니다. `stringify!(c"hi")`는 2018에서도 `c"hi"`로 보이므로, 토큰 수를 세어야 차이가 드러납니다.
- **2번 구역**: `c"caf\xe9"`의 `\xe9`는 UTF-8이 아닌 바이트 하나(233)입니다. `&str` 리터럴에서는 쓸 수 없는 이스케이프입니다.
- **3번 구역**: `head`는 선언을 `'a`로, 사용을 `'r#a`로 해서 둘이 같은 라이프타임임을 보여 줍니다.

### 06_pat_in_macros — `$p:pat`과 `$p:pat_param`

```sh
cargo run -p p2-ch3-rust-2021 --example 06_pat_in_macros
```

```text
1. $p:pat이 1 | 2를 패턴 하나로 받습니다
    classify!(1 | 2)       → 규칙 1: 패턴 하나
    classify!(Some(1 | 2)) → 규칙 1: 패턴 하나
2. 그래서 패턴을 받는 매크로에 A | B를 넘길 수 있습니다
    is_match!(grade, 'A' | 'B') → true
3. pat과 pat_param이 같은 입력을 나누는 방식
    whole!(1 | 2..=5 | _) → ["1 | 2..=5 | _"]
    split!(1 | 2..=5 | _) → ["1", "2..=5", "_"]
```

- 2018로 컴파일하면 1번 구역의 첫 줄이 "규칙 2"가 되고, 2번 구역의 `is_match!`와 3번 구역의 `whole!`은 `` no rules expected `|` `` 에러입니다. `split!`과 `Some(1 | 2)`는 2018에서도 같습니다.

### projects/same_code — 같은 소스, 다른 에디션

가상 매니페스트 하나에 멤버 셋이 있습니다. `e2018`과 `e2021`은 `[[bin]]`의 `path = "../shared/main.rs"`로 **같은 소스 파일**을 컴파일하고, 두 `Cargo.toml`은 패키지·실행 파일 이름과 `edition`만 다릅니다. 타깃마다 `edition`을 적는 키는 deprecated이므로([ch1](../ch1-editions-overview/README.md) 2.6절) 에디션마다 패키지를 나눴습니다. `macros2018`은 6번 구역에서 쓰는 2018 에디션 라이브러리입니다.

```sh
cd part2-editions/ch3-rust-2021/projects/same_code
cargo run -q -p e2018
```

```text
[e2018]
1. 배열에 .into_iter()를 부르면
    항목 타입: &i32
2. 클로저가 t.0만 옮길 때의 drop 순서
    drop: t.0
    drop: t.1
    consume() 호출 끝
3. 인자가 하나뿐인 panic!의 메시지
    (name = 러스트)
    panic!("{name} 설정 없음") → "{name} 설정 없음"
    panic!("중괄호 {{}}")    → "중괄호 {{}}"
4. 프렐류드의 TryInto와 이름이 겹치는 메서드
    x.try_into()        = Ok(255)
    Clamp::try_into(&x) = Ok(255)
5. 접두사 모양의 토큰 수
    c"hi" → 2개, 'r#a → 3개
6. 매크로의 $p:pat에 1 | 2를 넘기면
    이 크레이트의 classify!   → 규칙 2: 패턴이 아닌 토큰들
    macros2018::classify!    → 규칙 2: 패턴이 아닌 토큰들
```

```sh
cargo run -q -p e2021
```

```text
[e2021]
1. 배열에 .into_iter()를 부르면
    항목 타입: i32
2. 클로저가 t.0만 옮길 때의 drop 순서
    drop: t.0
    consume() 호출 끝
    drop: t.1
3. 인자가 하나뿐인 panic!의 메시지
    (name = 러스트)
    panic!("{name} 설정 없음") → "러스트 설정 없음"
    panic!("중괄호 {{}}")    → "중괄호 {}"
4. 프렐류드의 TryInto와 이름이 겹치는 메서드
    x.try_into()        = Err(TryFromIntError(PosOverflow))
    Clamp::try_into(&x) = Ok(255)
5. 접두사 모양의 토큰 수
    c"hi" → 1개, 'r#a → 1개
6. 매크로의 $p:pat에 1 | 2를 넘기면
    이 크레이트의 classify!   → 규칙 1: 패턴 하나
    macros2018::classify!    → 규칙 2: 패턴이 아닌 토큰들
```

두 실행 모두 종료 코드 0이고, 여섯 구역 모두 결과가 달라집니다. 대조군으로 넣은 두 줄은 두 실행에서 같습니다. `Clamp::try_into(&x)`는 트레이트 이름을 적어 불렀기 때문이고, `macros2018::classify!`는 2021 크레이트가 불러도 2018 크레이트에 정의된 매크로라서 2018 규칙을 따르기 때문입니다(2.11절).

- **경고 처리**: 공유 소스 맨 위의 `#![allow(array_into_iter, non_fmt_panics, clippy::into_iter_on_ref)]`는 2018로 컴파일할 때 1·3번 구역이 일부러 내는 경고(2.4절, 2.6절)를 막습니다. 2021로 컴파일할 때는 이 린트들이 아무것도 찾지 못합니다. 그래서 `#[expect]`가 아니라 `#[allow]`를 썼습니다. `#[expect]`로 쓰면 e2021 쪽에서 `this lint expectation is unfulfilled` 경고가 납니다.
- **마이그레이션 린트 보기**: `cargo rustc`는 이 패키지를 컴파일하는 `rustc`에 `--` 뒤의 옵션을 더 넘깁니다.

  ```sh
  cargo rustc -q -p e2018 -- -W rust-2021-compatibility
  ```

  경고 넷이 나옵니다. 5번 구역의 `c"hi"`와 `'r#a`(`rust_2021_prefixes_incompatible_syntax`), 4번 구역의 `x.try_into()`(`rust_2021_prelude_collisions`), 2번 구역의 클로저(`rust_2021_incompatible_closure_captures`)입니다. 1·3번 구역의 린트는 소스에서 `allow`했으므로 나오지 않고, **6번 구역의 변화는 어떤 린트도 알려 주지 않습니다.**
- **`resolver = "2"`**: 가상 매니페스트라서 직접 적었습니다(2.3절). 멤버가 2018과 2021이므로 2021 에디션의 기본값인 `"2"`를 골랐습니다. 이 프로젝트는 기능을 쓰지 않으므로 `"1"`이어도 결과는 같습니다.

### projects/feature_resolver — 에디션이 정하는 기능 리졸버

세 디렉터리가 각자의 워크스페이스 루트입니다. `flags`는 `verbose` 기능을 가진 라이브러리이고, `app2018`과 `app2021`은 이름과 `edition`만 다른 `Cargo.toml`, 똑같은 `src/main.rs`와 `examples/with_dev_deps.rs`를 가집니다.

```sh
cd part2-editions/ch3-rust-2021/projects/feature_resolver/app2018
cargo run -q
cargo run -q --example with_dev_deps
cd ../app2021
cargo run -q
cargo run -q --example with_dev_deps
```

```text
app2018 main: flags의 verbose = true
app2018 예제: flags의 verbose = true
app2021 main: flags의 verbose = false
app2021 예제: flags의 verbose = true
```

모두 종료 코드 0입니다. 출력의 의미와 `cargo tree` 결과는 2.3절에 있습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 2018 에디션에서는 컴파일되지만 2021 이후에서는 실패하는 코드입니다. 챕터 디렉터리에서 에디션을 바꿔 가며 컴파일해 봅니다. 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part2-editions/ch3-rust-2021
rustc --edition 2024 compile_fail/e0034_prelude_try_into_ambiguous.rs --out-dir /tmp/rust-p2-ch3   # 실패
rustc --edition 2021 compile_fail/e0034_prelude_try_into_ambiguous.rs --out-dir /tmp/rust-p2-ch3   # 실패
rustc --edition 2018 compile_fail/e0034_prelude_try_into_ambiguous.rs --out-dir /tmp/rust-p2-ch3   # 통과
```

여섯 파일 모두 2021과 2024에서 같은 에러로 실패하고 2018에서 통과하는 것을 확인했습니다. 아래 인용은 `--edition 2024`의 결과이며, 표준 라이브러리 소스 위치(`/rustc/0ef1bc38.../library/...`)를 가리키는 줄은 생략했습니다.

### 4.1 `dyn` 없는 트레이트 객체 — E0782

파일: [`compile_fail/e0782_bare_trait_object.rs`](compile_fail/e0782_bare_trait_object.rs)

```text
error[E0782]: expected a type, found a trait
  --> compile_fail/e0782_bare_trait_object.rs:23:29
   |
23 | fn total_area(shapes: &[Box<Shape>]) -> f64 { // → E0782
   |                             ^^^^^
   |
help: you can add the `dyn` keyword if you want a trait object
   |
23 | fn total_area(shapes: &[Box<dyn Shape>]) -> f64 { // → E0782
   |                             +++
```

- **읽는 법**: 타입 자리에 트레이트 이름 `Shape`가 왔다는 뜻입니다. 2021부터 트레이트 이름만으로는 타입이 아닙니다.
- **2018에서는**: `trait objects without an explicit dyn are deprecated` 경고와 함께 컴파일되고, 실행하면 `13`(2×2 + 3×3)을 출력합니다.
- **해결**: `help:`대로 `Box<dyn Shape>`로 씁니다. `...` 범위 패턴(E0783)도 같은 방식으로 승격되었습니다(2.10절).

### 4.2 프렐류드의 `TryInto`와 사용자 트레이트가 부딪힘 — E0034

파일: [`compile_fail/e0034_prelude_try_into_ambiguous.rs`](compile_fail/e0034_prelude_try_into_ambiguous.rs)

```text
error[E0034]: multiple applicable items in scope
  --> compile_fail/e0034_prelude_try_into_ambiguous.rs:26:35
   |
26 |     let r: Result<u8, String> = x.try_into(); // → E0034
   |                                   ^^^^^^^^ multiple `try_into` found
   |
   = note: candidate #1 is defined in an impl of the trait `TryInto` for the type `T`
note: candidate #2 is defined in an impl of the trait `Clamp` for the type `i32`
  --> compile_fail/e0034_prelude_try_into_ambiguous.rs:19:5
help: disambiguate the method for candidate #1
26 +     let r: Result<u8, String> = TryInto::try_into(x); // → E0034
help: disambiguate the method for candidate #2
26 +     let r: Result<u8, String> = Clamp::try_into(x); // → E0034
```

(`note:`와 `help:` 아래의 소스 인용은 줄여 적었습니다.)

- **읽는 법**: 후보 #1은 모든 타입 `T`에 대한 `TryInto`의 포괄 구현, 후보 #2는 사용자 트레이트 `Clamp`입니다. 둘 다 `self`를 값으로 받으므로 메서드 탐색의 같은 단계에서 찾아졌습니다.
- **원인**: 2018에서는 `TryInto`가 스코프에 없어 후보가 하나였습니다. 2018로 컴파일해 실행하면 `Ok(255)`입니다.
- **해결**: 원래 뜻대로 `Clamp::try_into(x)`로 씁니다.
- **에러가 나서 다행인 경우입니다.** `Clamp`가 `&self`를 받았다면 첫 단계에서 `TryInto`만 찾아져 표준 메서드가 호출됩니다(2.2절). 이 파일처럼 결과를 `Result<u8, String>`으로 적어 두었다면 그래도 E0271(`` type mismatch resolving `<u8 as TryFrom<i32>>::Error == String` ``)로 드러나지만, `Result<u8, _>`였다면 아무 에러 없이 값만 바뀝니다. 두 경우 모두 직접 확인했습니다.

### 4.3 분리 캡처로 클로저가 `Send`를 잃음 — E0277

파일: [`compile_fail/e0277_disjoint_capture_not_send.rs`](compile_fail/e0277_disjoint_capture_not_send.rs)

```text
error[E0277]: `*mut i32` cannot be sent between threads safely
  --> compile_fail/e0277_disjoint_capture_not_send.rs:25:32
   = help: within `{closure@compile_fail/e0277_disjoint_capture_not_send.rs:25:32: 25:39}`, the trait `Send` is not implemented for `*mut i32`
note: required because `px.0` is used within this closure
  --> compile_fail/e0277_disjoint_capture_not_send.rs:28:18
   |
28 |         unsafe { *(px.0) += 10 };
   |                  ^^^^^^^
note: required by a bound in `spawn`
```

(클로저 전체에 밑줄을 긋는 첫 번째 소스 인용은 줄여 적었습니다.)

- **읽는 법**: 핵심은 `` note: required because `px.0` is used within this closure ``입니다. 클로저가 `px` 전체(`Ptr`, `unsafe impl Send`)가 아니라 `px.0`(`*mut i32`)을 캡처했고, 원시 포인터는 `Send`가 아닙니다.
- **원인**: 분리 캡처(2.5절). 2018에서는 `px` 전체를 캡처하므로 컴파일되고, 실행하면 `15`를 출력합니다.
- **해결**: 클로저 본문 첫 줄에 `let _ = &px;`를 넣어 `px` 전체를 캡처하게 합니다. 2024로 컴파일해 `15`가 출력되는 것을 확인했습니다. 원시 포인터를 꺼내는 메서드(`fn get(&self) -> *mut i32`)를 `Ptr`에 두고 `px.get()`으로 부르는 것도 방법입니다. 메서드 호출은 `px` 전체를 빌리기 때문입니다.

### 4.4 `panic!`에 문자열이 아닌 값을 넘김 — 에러 코드 없음

파일: [`compile_fail/panic_non_string_payload.rs`](compile_fail/panic_non_string_payload.rs)

```text
error: format argument must be a string literal
  --> compile_fail/panic_non_string_payload.rs:16:16
   |
16 |         panic!(code); // → format argument must be a string literal
   |                ^^^^
   |
help: you might be missing a string literal to format with
   |
16 |         panic!("{}", code); // → format argument must be a string literal
   |                +++++
```

- **읽는 법**: `panic!`의 첫 인자는 서식 문자열 리터럴이어야 한다는 뜻입니다. `println!(code)`가 거절되는 것과 같은 이유입니다.
- **2018에서는**: `non_fmt_panics` 경고(`panic message is not a string literal`, `this usage of panic!() is deprecated; it will be a hard error in Rust 2021`)와 함께 컴파일됩니다. 실행하면 페이로드가 `u16`인 패닉이 일어나 표준 에러에 `Box<dyn Any>`가 출력되고 종료 코드 101로 끝납니다.
- **해결**: 메시지가 목적이면 `help:`대로 `panic!("{}", code)`입니다. 값 자체를 페이로드로 넘겨 `catch_unwind` 쪽에서 `downcast_ref::<u16>()`으로 꺼내려면 `std::panic::panic_any(code)`입니다. 2018의 경고도 이 두 가지를 제안합니다.

### 4.5 매크로 입력에 알 수 없는 접두사 — 에러 코드 없음

파일: [`compile_fail/unknown_prefix_in_macro_call.rs`](compile_fail/unknown_prefix_in_macro_call.rs)

```text
error: prefix `z` is unknown
  --> compile_fail/unknown_prefix_in_macro_call.rs:20:30
   |
20 |     println!("{}", count_tt!(z"hey")); // → prefix `z` is unknown
   |                              ^ unknown prefix
   |
   = note: prefixed identifiers and literals are reserved since Rust 2021
help: consider inserting whitespace here
   |
20 |     println!("{}", count_tt!(z "hey")); // → prefix `z` is unknown
   |                               +
```

- **읽는 법**: `z"`를 접두사가 붙은 리터럴로 읽으려 했는데 `z`는 정해진 접두사가 아니라는 뜻입니다. 이 에러는 토큰을 나누는 단계에서 나므로, 매크로가 그 토큰을 어떻게 쓰는지와 상관없습니다.
- **2018에서는**: 통과하고, 실행하면 `2`(토큰 `z`와 `"hey"`)를 출력합니다.
- **해결**: `z "hey"`처럼 띄웁니다. 결과는 2018과 같은 `2`입니다.

### 4.6 `$x:pat` 뒤에 `|`를 둔 매크로 규칙 — 에러 코드 없음

파일: [`compile_fail/pat_fragment_followed_by_or.rs`](compile_fail/pat_fragment_followed_by_or.rs)

```text
error: `$x:pat` is followed by `|`, which is not allowed for `pat` fragments
  --> compile_fail/pat_fragment_followed_by_or.rs:14:13
   |
14 |     ($x:pat | $y:pat) => { // → `$x:pat` is followed by `|`
   |      ------ ^ not allowed after `pat` fragments
   |      |
   |      help: try a `pat_param` fragment specifier instead: `$x:pat_param`
   |
   = note: allowed there are: `=>`, `,`, `=`, `if`, `if let` or `in`
```

- **읽는 법**: `$x:pat` 바로 뒤에 올 수 있는 토큰은 `note:`에 적힌 것뿐입니다. 2021의 `$x:pat`은 `|`까지 삼킬 수 있으므로, 뒤에 `|`가 오면 어디서 끊을지 정할 수 없습니다.
- **원인**: 매크로를 **정의**하는 단계의 에러라서, 매크로를 한 번도 부르지 않아도 납니다. 2018에서는 통과하고 실행하면 `둘 중 하나`를 출력합니다.
- **해결**: `help:`대로 `$x:pat_param`으로 바꿉니다. 2024로 컴파일해 `둘 중 하나`가 출력되는 것을 확인했습니다.

### 4.7 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| 2021 이후에서 `match score { 90...100 => .. }` | E0783 `` `...` range patterns are deprecated `` | `90..=100` (2.10절) |
| 2018에서 `use` 없이 `big.try_into()` | E0599, `note: 'std::convert::TryInto' is included in the prelude starting in Edition 2021` | `use std::convert::TryInto;` 또는 2021 이후로 (2.2절) |
| 2018에서 `c"hello"` | `` expected one of `!`, `.`, `::`, … found `"hello"` ``, `note: c-string literals require Rust 2021 or later` | 2021 이후로 옮기거나 `CStr::from_bytes_with_nul(b"hello\0")` (2.8절) |
| 2021 이후에서 `panic!("{}")` | `1 positional argument in format string, but no arguments were given` | 글자 그대로라면 `panic!("{{}}")` (2.6절) |
| 2021 이후에서 `Box<dyn Code>`인 `f`에 `f.try_into()` (`Code`에 `try_into(&self)`가 있음) | E0283 `` type annotations needed for `Result<_, _>` `` (`Box<dyn Code>`의 `TryInto`가 잡힘) | `(&*f).try_into()` (2.2절) |
| 2024에서 라이프타임 이름 `'gen` | `lifetimes cannot use keyword names` | `'r#gen` (2.9절) |
| 2021 이후에서 매크로 입력 `'r# a` (Edition Guide의 안내) | `` prefix `'r` is unknown `` | `'r #a` (2.9절) |
| 2021 이후에서 `match"가" { .. }` | `` prefix `match` is unknown `` | `match "가"` (2.7절) |
| 가상 매니페스트에 `resolver` 없이 2021 멤버 | 경고 `` virtual workspace defaulting to `resolver = "1"` `` | `[workspace]`에 `resolver = "2"` (2.3절) |
| 에디션마다 나기도 하고 안 나기도 하는 경고를 `#![expect(..)]`로 막음 | 경고가 안 나는 에디션에서 `this lint expectation is unfulfilled` | `#![allow(..)]`와 이유 주석 (3절 `same_code`) |

## 5. 연습 문제

### 연습 1. 2018 코드를 손으로 옮기기

다음은 2018 에디션에서 경고 두 개와 함께 컴파일되는 코드입니다. 먼저 2021로 컴파일했을 때 어디서 에러가 나고 어디서 출력이 달라질지 예측한 뒤, `rustc --edition 2018`과 `--edition 2021`로 확인하세요. 그다음 **2021에서도 2018과 똑같이 출력되도록** 고치세요.

```rust
use std::convert::TryInto;

struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop: {}", self.0);
    }
}

fn main() {
    let scores = [90, 75, 60];
    let refs: Vec<&i32> = scores.into_iter().collect();
    let pair = (Noisy("앞"), Noisy("뒤"));
    let take_first = || drop(pair.0);
    take_first();
    println!("take_first 호출 끝");
    let level: Result<u8, _> = 300_i32.try_into();
    println!("{:?} {:?}", refs, level);
    if refs.len() > 5 {
        panic!("점수가 {}개를 넘었습니다");
    }
}
```

2018에서의 출력은 다음과 같습니다.

```text
drop: 앞
drop: 뒤
take_first 호출 끝
[90, 75, 60] Err(TryFromIntError(PosOverflow))
```

<details>
<summary>힌트</summary>

- 2021에서는 에러가 둘입니다. `scores.into_iter()`가 `i32`를 내놓아 `Vec<&i32>`로 모을 수 없고(E0277), `panic!("점수가 {}개를 넘었습니다")`는 인자가 없는 자리표시자라서 에러입니다. 앞의 것은 참조를 원한 것이므로 `.iter()`로 바꿉니다(2.4절).
- `panic!`은 실행되지 않는 줄이지만 2021에서는 컴파일부터 되지 않습니다. 2018에서 이 패닉이 냈을 메시지는 `점수가 {}개를 넘었습니다`라는 글자 그대로이므로, 2018과 같게 하려면 `panic!("점수가 {{}}개를 넘었습니다")`입니다. 원래 의도가 개수를 넣는 것이었다면 `panic!("점수가 {}개를 넘었습니다", refs.len())`이 맞겠지요. 마이그레이션 중에는 "예전과 같게"와 "원래 의도대로"를 구분해야 합니다.
- 두 에러만 고치면 2021에서 `drop: 뒤`가 맨 마지막 줄로 옮겨 갑니다. 클로저가 `pair.0`만 가져가서 `pair.1`은 `main`이 끝날 때 버려지기 때문입니다(2.5절). 클로저 본문에 `let _ = &pair;`를 넣으면 2018과 같은 순서가 됩니다.
- `use std::convert::TryInto;`는 2021에서 프렐류드와 겹치지만 에러도 경고도 아닙니다. `try_into`는 원래 표준 메서드였으므로 결과도 같습니다.

</details>

### 연습 2. 토큰 수 예측하기

[`05_reserved_prefixes.rs`](examples/05_reserved_prefixes.rs)의 `count_tt!`를 새 파일에 복사하고, 다음 입력의 토큰 수를 2018과 2021에서 각각 예측한 뒤 `rustc --edition 2018`/`2021`로 확인하세요. 에러가 나는 입력은 공백을 넣어 고친 뒤의 토큰 수도 적으세요.

`br"x"`, `cr#"x"#`, `r#match`, `'r#fn`, `abc'x'`, `k#async`, `x#1`

<details>
<summary>힌트</summary>

- `br"x"`와 `r#match`(raw 식별자)는 2018 이전부터 있던 토큰이라 어느 에디션에서나 1개입니다.
- `cr#"x"#`와 `'r#fn`은 2021에서 1개, 2018에서는 여러 개로 나뉩니다. 2018에서 `cr`은 그냥 식별자입니다.
- `abc'x'`, `k#async`, `x#1`은 2.7절의 예약 모양(`접두사'.'`, `접두사#식별자`, `접두사#123`)입니다. 2021에서는 `` prefix `…` is unknown `` 에러이고, 에러 메시지의 `help:`가 공백을 넣을 자리를 알려 줍니다.

</details>

### 연습 3. 다른 플랫폼 전용 의존성의 기능

[`projects/feature_resolver`](projects/feature_resolver)를 저장소 밖으로 복사하고, 두 앱의 `[dev-dependencies]`를 지운 뒤 다음을 넣으세요. 지금 쓰는 컴퓨터가 Windows가 아니라면 `app2018`과 `app2021`의 `cargo run -q`는 각각 무엇을 출력할까요? 예측한 뒤 확인하세요. Windows라면 `cfg(windows)`를 `cfg(unix)`로 바꿔서 해 봅니다.

```toml
[target.'cfg(windows)'.dependencies]
flags = { path = "../flags", features = ["verbose"] }
```

<details>
<summary>힌트</summary>

- 2.3절에 적은 리졸버 2의 첫 번째 규칙입니다. 리졸버 1은 대상 플랫폼과 상관없이 기능을 합치므로 macOS에서도 `true`, 리졸버 2는 지금 빌드하지 않는 플랫폼의 의존성 기능을 켜지 않으므로 `false`입니다. macOS에서 이렇게 나오는 것을 확인했습니다.
- `cargo tree -e normal -f '{p} {f}'`로도 확인해 보세요.
- 복사한 디렉터리에서 실행하는 이유는, 저장소 안의 프로젝트를 고치면 `Cargo.lock`과 예제 설명이 달라지기 때문입니다.

</details>

### 연습 4. 2021에서만 `Clone`인 클로저

다음 코드는 2021 에디션에서 `6 6`과 `0`을 출력합니다. 2018에서는 컴파일되지 않습니다. 어떤 에러가 날지, 그리고 2021에서 마지막 줄이 `task.lock`을 쓸 수 있는 이유를 설명하세요. 그런 다음 클로저 본문에 한 줄을 더해 2021에서도 2018과 같은 에러가 나게 만들어 보세요.

```rust
use std::sync::Mutex;

struct Task {
    name: String,
    lock: Mutex<u32>,
}

fn main() {
    let task = Task { name: String::from("정리"), lock: Mutex::new(0) };
    let describe = move || task.name.len();
    let copy = describe.clone();
    println!("{} {}", describe(), copy());
    println!("{:?}", task.lock.lock().unwrap());
}
```

<details>
<summary>힌트</summary>

- 클로저는 캡처한 값이 모두 `Clone`이면 `Clone`입니다(2.5절). 2021의 클로저는 `task.name`(`String`)만 가져가고, 2018의 클로저는 `task` 전체를 가져가는데 `Mutex`가 `Clone`이 아니어서 `Task`도 `Clone`이 아닙니다.
- 2018의 에러는 `describe.clone()`에서 나는 E0277(`` the trait bound `Task: Clone` is not satisfied ``)이고, `` note: required because `task` is used within this closure ``가 붙습니다.
- `6`은 `"정리"`의 바이트 수입니다. 2021에서는 `task.lock`이 클로저로 옮겨 가지 않았으므로 마지막 줄에서 쓸 수 있습니다.
- 더할 한 줄은 더미 let(`let _ = &task;`)입니다.

</details>

## 6. 참고 자료

- Edition Guide (기준 문서)
  - [4. Rust 2021](https://doc.rust-lang.org/edition-guide/rust-2021/index.html): RFC 3085, 안정화 버전 1.56.0.
  - [4.1 Additions to the prelude](https://doc.rust-lang.org/edition-guide/rust-2021/prelude.html): 충돌하는 트레이트 메서드, `dyn Trait`의 메서드, 고유 메서드는 마이그레이션이 필요 없다는 설명과 린트가 고려하는 조건(Implementation Reference).
  - [4.2 Default Cargo feature resolver](https://doc.rust-lang.org/edition-guide/rust-2021/default-cargo-resolver.html): `bstr`과 `diesel`의 빌드 실패 예, `cargo tree`로 기능 살펴보기.
  - [4.3 IntoIterator for arrays](https://doc.rust-lang.org/edition-guide/rust-2021/IntoIterator-for-arrays.html): 1.53.0의 절충과 `array_into_iter` 린트.
  - [4.4 Disjoint capture in closures](https://doc.rust-lang.org/edition-guide/rust-2021/disjoint-capture-in-closures.html): 와일드카드 패턴, drop 순서, 트레이트 구현(원시 포인터 래퍼 예).
  - [4.5 Panic macro consistency](https://doc.rust-lang.org/edition-guide/rust-2021/panic-macro-consistency.html)
  - [4.6 Reserved syntax](https://doc.rust-lang.org/edition-guide/rust-2021/reserved-syntax.html): 앞으로 쓸 수 있는 접두사 후보.
  - [4.7 Raw lifetimes](https://doc.rust-lang.org/edition-guide/rust-2021/raw-lifetimes.html): 마이그레이션 예의 `'r# foo`는 1.100.0에서 에러였습니다(2.9절).
  - [4.8 Warnings promoted to errors](https://doc.rust-lang.org/edition-guide/rust-2021/warnings-promoted-to-error.html)
  - [4.9 Or patterns in macro-rules](https://doc.rust-lang.org/edition-guide/rust-2021/or-patterns-macro-rules.html)
  - [4.10 C-string literals](https://doc.rust-lang.org/edition-guide/rust-2021/c-string-literals.html): 이스케이프와 raw C 문자열의 예.
- Cargo Book
  - [Dependency Resolution: Feature resolver version 2](https://doc.rust-lang.org/cargo/reference/resolver.html#feature-resolver-version-2)와 [Resolver versions](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions): 리졸버 2가 기능을 합치지 않는 세 경우, 버전별 기본값, 가상 매니페스트의 `resolver`.
  - [Features: Feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification): 기능 통합의 원리. [Part 3 ch5](../../part3-cargo/ch5-features/README.md)에서 자세히 읽습니다.
- 표준 라이브러리: [`std::prelude`](https://doc.rust-lang.org/std/prelude/index.html)의 "Prelude contents" 절. 2015·2018은 `v1`, 2021은 여기에 `TryFrom`/`TryInto`/`FromIterator`, 2024는 다시 `Future`/`IntoFuture`를 더한다고 적혀 있습니다.
- 에러 설명: `rustc --explain E0034`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
