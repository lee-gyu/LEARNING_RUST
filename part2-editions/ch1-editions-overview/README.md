# ch1. 에디션 개요

Part 2의 첫 챕터입니다. Part 0과 Part 1을 지나는 동안 "2021 에디션부터", "2024 에디션에서 바뀌었다"는 말을 여러 번 만났습니다.

- [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md): `rustc`는 에디션을 적지 않으면 2015 에디션으로 컴파일하므로 `--edition 2024`를 붙였습니다.
- [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.11절: 배열의 `into_iter()`는 2021 에디션부터 원소를 값으로 내놓습니다.
- [Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.2절: let 체인은 2024 에디션에서만 쓸 수 있습니다.
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.7절, [Part 1 ch10](../../part1-standard-lib/ch10-env-process-time/README.md) 2.4절: `unsafe extern`, `static mut` 참조 금지, `env::set_var`의 unsafe화는 2024 에디션의 변화입니다.
- [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.2절: 프렐류드의 목록은 크레이트의 에디션이 정합니다.

이 챕터는 그 **에디션이라는 제도 자체**를 다룹니다. 왜 생겼는지, 무엇을 바꿀 수 있고 무엇은 바꿀 수 없는지, 에디션이 다른 크레이트들이 어떻게 한 프로그램 안에서 함께 동작하는지를 직접 확인합니다. 에디션마다 무엇이 바뀌었는지는 이어지는 [Part 2 ch2](../ch2-rust-2015-2018/README.md)~[ch5](../ch5-rust-2024-ecosystem/README.md)에서 하나씩 살펴봅니다.

- 패키지: `p2-ch1-editions-overview`
- 기준 문서: [Edition Guide 1. What are editions?](https://doc.rust-lang.org/edition-guide/editions/index.html), [1.1 Creating a new project](https://doc.rust-lang.org/edition-guide/editions/creating-a-new-project.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024), macOS (aarch64). 비교를 위해 이 환경에 이미 설치되어 있던 rustc 1.98.1을 두 번 썼습니다(2.5절).
- 독립 프로젝트 [`projects/edition_mix`](projects/edition_mix/Cargo.toml)는 1.100.0에 안정화된 API를 쓰므로 rustc 1.100.0 이상이 필요합니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Rust가 하위 호환성(backward compatibility)을 약속하면서도 새 키워드 같은 호환되지 않는 변경(breaking change)을 들여오는 방법으로 에디션(edition)을 설명하고, 에디션이 없었다면 깨졌을 코드의 예를 들 수 있다.
2. 에디션은 크레이트마다 고르는 것이고 컴파일러 하나가 모든 에디션을 지원한다는 것을 설명하고, 에디션이 다른 크레이트들을 한 프로그램으로 링크하는 워크스페이스를 만들 수 있다.
3. 다른 에디션의 크레이트가 키워드와 같은 이름으로 공개한 항목을 raw 식별자로 부르고, 매크로 본문이 매크로를 정의한 크레이트의 에디션으로 읽힌다는 것을 예로 설명할 수 있다.
4. 에디션이 바꾸는 것(그 크레이트의 문법과 일부 의미, 프렐류드, 도구의 기본값)과 바꾸지 않는 것(표준 라이브러리, 크레이트 사이의 인터페이스, 대부분의 새 기능)을 구분하고, 에디션과 컴파일러 버전이 별개라는 것을 `edition`과 `rust-version`으로 설명할 수 있다.
5. `rustc --edition`, `Cargo.toml`의 `edition`, `cargo new --edition`으로 에디션을 정하고, `edition`을 빠뜨렸을 때의 경고와 `cargo build -v`로 실제로 넘어가는 값을 확인할 수 있다. 컴파일 에러가 에디션 때문인지 의심하고 확인할 수 있다.

## 2. 핵심 개념

### 2.1 이 챕터의 지도

| 절 | 질문 | 확인하는 곳 |
| --- | --- | --- |
| 2.2 | 에디션은 왜 있나 | [`compile_fail/keyword_async_identifier.rs`](compile_fail/keyword_async_identifier.rs) |
| 2.3 | 에디션은 누가 고르고, 컴파일러는 몇 개인가 | 예제 02 |
| 2.4 | 에디션이 바꾸는 것과 바꾸지 않는 것 | 예제 01, 02 |
| 2.5 | 에디션과 컴파일러 버전은 어떻게 다른가 | 예제 02, `projects/edition_mix` |
| 2.6 | 에디션은 어디에 적나 | `projects/edition_mix` |
| 2.7~2.8 | 에디션이 다른 크레이트를 어떻게 함께 쓰나 | `projects/edition_mix` |
| 2.9 | 에디션마다 무엇이 바뀌었나 | 표 |

### 2.2 왜 에디션이 필요한가

**하위 호환성 약속.** Edition Guide는 2015년 5월에 나온 Rust 1.0이 "정체 없는 안정성(stability without stagnation)"을 핵심 원칙으로 세웠다고 설명합니다. 한 번 안정(stable) 버전으로 나온 기능은 앞으로의 모든 릴리스에서 계속 지원한다는 약속입니다. 그래서 컴파일러를 새 버전으로 올려도 어제 컴파일되던 코드는 오늘도 컴파일되어야 합니다. 큰 프로그램을 오래 유지하려면 이 약속이 필요합니다.

**그런데 깨뜨려야 할 때가 있습니다.** 대표적인 경우가 새 키워드입니다. 초기 Rust에는 `async`와 `await` 키워드가 없었으므로 `async`는 아무 데나 쓸 수 있는 평범한 이름이었습니다. 다음 코드는 2015 에디션에서 지금도 경고 없이 컴파일됩니다.

```rust
fn send(message: &str) -> String {
    let async = message.len() > 10; // "나중에 보낼까?"를 담은 변수
    if async {
        format!("[나중에 보냄] {}", message)
    } else {
        format!("[바로 보냄] {}", message)
    }
}
```

컴파일러가 어느 날 `async`를 키워드로 바꾸면 이 코드는 컴파일되지 않습니다. 약속을 깨는 것입니다.

**에디션은 이런 변경을 고른 크레이트에만 적용합니다.** 호환되지 않는 변경은 다음 에디션에 넣고, 크레이트가 그 에디션을 고를 때만 적용합니다. `async`는 2018 에디션부터 키워드이고, 최신 컴파일러도 2015 에디션 크레이트를 컴파일할 때는 `async`를 이름으로 읽습니다. 같은 파일을 2018 이후의 에디션으로 컴파일하면 에러가 납니다(4.1절).

```text
error: expected identifier, found keyword `async`
```

그래서 Rust에는 두 가지 "올리기"가 있습니다.

| | 컴파일러를 새 버전으로 올리기 | 크레이트의 에디션을 올리기 |
| --- | --- | --- |
| 기존 코드 | 그대로 컴파일됩니다(하위 호환성 약속) | 고쳐야 할 수 있습니다 |
| 누가 정하나 | 툴체인을 설치한 사람 | 그 크레이트의 `Cargo.toml` |
| 얻는 것 | 새 기능, 새 표준 라이브러리 API, 버그 수정 | 새 키워드와 문법, 정리된 규칙 |

에디션을 올리는 작업을 마이그레이션(migration)이라고 합니다. 대부분은 `cargo fix --edition`이 자동 수정(automatic fix)으로 처리하며, 예를 들어 `async`라는 이름을 `r#async`로 바꿔 줍니다. 절차는 [Part 2 ch6](../ch6-migration-practice/README.md)에서 다룹니다.

### 2.3 에디션은 크레이트마다 고르고, 컴파일러 하나가 모두 압니다

**컴파일러 하나, 에디션 넷.** 에디션마다 다른 컴파일러가 있는 것이 아닙니다. `rustc --help`의 한 항목이 이것을 보여 줍니다.

```text
        --edition <2015|2018|2021|2024|future>
                        Specify which edition of the compiler to use when
                        compiling code. The default is 2015 and the latest
                        stable edition is 2024.
```

[`02_rustc_edition_matrix.rs`](examples/02_rustc_edition_matrix.rs)는 짧은 코드 조각 여덟 개를 같은 `rustc`에 `--edition`만 바꿔 넘기고, 컴파일되는지(O) 에러인지(X)를 표로 보여 줍니다.

```text
    2015 2018 2021 2024  코드 조각
     O    O    O    O    기준: println! 한 줄
     O    X    X    X    async를 변수 이름으로
     X    O    O    O    하위 모듈 안에서 use circle::area
     X    X    O    O    use 없이 u8::try_from
     O    O    O    X    gen을 변수 이름으로
     X    X    X    O    let 체인
     O    O    O    O    let-else (1.65.0)
     O    O    O    O    bool::toggle (1.100.0)
```

`async`부터 let 체인까지 다섯 줄은 에디션에 따라 결과가 갈립니다. 같은 컴파일러가 `--edition` 값에 따라 다른 규칙을 적용한 것입니다. 각 줄이 어느 에디션의 어떤 변경인지는 2.9절의 표에 있고, 마지막 두 줄은 2.4절에서 다룹니다.

**에디션은 크레이트 단위입니다.** Cargo는 패키지마다 `Cargo.toml`의 `edition`을 읽어, 그 패키지의 크레이트를 컴파일할 때 `--edition`으로 넘깁니다. 그래서 한 프로그램의 의존성 그래프 안에 2015 에디션 크레이트와 2024 에디션 크레이트가 섞여 있는 것이 정상입니다. Edition Guide는 에디션을 만들 때 가장 중요한 규칙으로 "한 에디션의 크레이트는 다른 에디션으로 컴파일된 크레이트와 매끄럽게 함께 동작해야 한다"를 꼽습니다. 그 결과 어떤 크레이트가 언제 에디션을 올릴지는 그 크레이트만의 결정이 되고, 그 크레이트를 쓰는 쪽도 그 크레이트가 쓰는 쪽도 영향을 받지 않습니다. 에디션 때문에 생태계가 갈라지지 않는다는 뜻입니다.

**어떻게 가능한가.** 이 규칙을 지키려면 에디션이 바꿀 수 있는 것이 "겉"으로 한정됩니다. Edition Guide의 표현으로 에디션의 변화는 "피부 깊이(skin deep)"이고, 어느 에디션으로 쓴 코드든 컴파일러 안에서 결국 같은 내부 표현이 됩니다. 크레이트의 함수, 타입, 트레이트는 그 내부 표현으로 다른 크레이트에 전달되므로, 쓰는 쪽은 그것이 어느 에디션으로 쓰였는지 신경 쓰지 않습니다. 2.7절에서 직접 확인합니다.

### 2.4 에디션이 바꾸는 것과 바꾸지 않는 것

**바꾸는 것: 그 크레이트의 소스 코드를 읽는 규칙.**

| 분야 | 예 | 관련된 곳 |
| --- | --- | --- |
| 키워드 | `async`·`await`(2018), `gen` 예약(2024) | 이 챕터 4.1절, 4.2절 |
| 이름을 찾는 규칙 | `use` 경로의 시작점(2018), 프렐류드의 목록(2021에 `TryFrom` 추가) | [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.7절, [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.2절 |
| 같은 코드의 뜻 | 배열의 `into_iter()`(2021), `if let`의 임시값 범위(2024) | [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.11절, [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.10절 |
| 새 문법 | let 체인(2024) | [Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.2절 |
| 린트 수준 | `dyn` 없는 트레이트 객체가 에러로(2021), `static_mut_refs`가 deny로(2024) | [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 2.4절, [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.7절 |
| 도구의 기본값 | Cargo 리졸버 버전, rustdoc 문서 테스트 병합, rustfmt 스타일 에디션 | [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md) 예제 안내 |

[`01_edition_effects.rs`](examples/01_edition_effects.rs)는 이 저장소의 예제가 2024 에디션이라서 보였던 동작 다섯 가지를 모아 실행합니다. Part 0과 Part 1에서 만난 그 동작들은 표준 라이브러리나 컴파일러 버전이 아니라 **에디션의 효과**였습니다. 같은 코드를 옛 에디션으로 컴파일해 비교한 결과는 다음과 같습니다(옛 에디션에서는 문법 에러가 나는 3번·5번 구역을 뺀 사본으로 확인했습니다).

| 구역 | 2024 (예제 01) | 2021 | 2015·2018 |
| --- | --- | --- | --- |
| 1. `[10, 20, 30].into_iter().next()`의 타입 | `Option<i32>` | `Option<i32>` | `Option<&i32>`, `array_into_iter` 경고 |
| 2. `use` 없이 `u8::try_from(300)` | `Err(TryFromIntError(PosOverflow))` | 같음 | E0599. `use std::convert::TryFrom;`을 넣으면 같음 |
| 4. `if let`의 `else` 안에서 `try_borrow_mut()` | `true` | `false` | `false` |

2015·2018 에디션의 `array_into_iter` 경고는 `` this method call resolves to `<&[T; N] as IntoIterator>::into_iter` (due to backwards compatibility), but will resolve to `<[T; N] as IntoIterator>::into_iter` in Rust 2021 ``입니다. "하위 호환성 때문에" 옛 뜻을 유지하고 있다고 컴파일러가 직접 말해 줍니다.

**바꾸지 않는 것.**

- **표준 라이브러리.** 모든 에디션이 같은 표준 라이브러리 하나를 씁니다. 새 버전에서 추가된 API는 2015 에디션 크레이트에서도 쓸 수 있습니다. 예제 02의 마지막 줄(`bool::toggle`, 1.100.0)이 그것입니다. 2021 에디션 프렐류드에 `TryFrom`이 들어간 것도 트레이트가 새로 생긴 것이 아니라 "`use` 없이 쓸 수 있는 이름의 목록"이 바뀐 것입니다.
- **대부분의 새 기능.** 기존 코드를 깨뜨리지 않는 새 문법은 모든 에디션에 들어옵니다. let-else(1.65.0), 서식 문자열 안의 변수 이름(`{state}`), `dbg!`는 모두 2015 에디션에서 컴파일됩니다. 에디션 전용이 되는 것은 새 키워드가 필요한 기능(`async fn`은 2018 에디션부터)이나 에디션에서 바뀐 규칙에 기대는 기능입니다. let 체인은 2024 에디션에서 바뀐 `if let`의 임시값 범위를 전제로 하므로 2024 에디션 전용입니다(Edition Guide 5.1.3절).
- **크레이트 사이의 인터페이스.** 2015 에디션 크레이트가 정의한 구조체와 메서드를 2024 에디션 크레이트가 그대로 씁니다(2.7절).

The Book 부록 E는 "에디션의 변화는 컴파일러가 코드를 처음 파싱하는 방식에만 영향을 준다"고 짧게 설명합니다. 실제로는 위 표처럼 이름을 찾는 규칙, 메서드 호출의 뜻, 임시값이 버려지는 시점도 바뀝니다. 핵심은 그 차이가 모두 **그 크레이트의 소스 코드를 읽는 방법**에 머문다는 점입니다.

**예외: 모든 에디션에 적용된 변경.** 드물게 에디션과 상관없이 모든 크레이트에 적용되는 변경도 있습니다. Edition Guide 5.1.10절은 2024 에디션의 변경으로 소개했던 never 타입 폴백 변경이 "1.100부터 더 이상 에디션 전용이 아니다"라고 적고 있습니다. 실제로 이 툴체인에서 그 절의 예제(제네릭 함수 `f()?;`)는 2015·2021·2024 에디션 모두 같은 E0277(`` the trait bound `!: Default` is not satisfied ``)로 실패했습니다. never 타입 폴백이 무엇인지는 [Part 2 ch4](../ch4-rust-2024-language/README.md)에서 다룹니다.

### 2.5 에디션과 컴파일러 버전은 다릅니다

"Rust 2024"는 2024년에 나온 컴파일러가 아니라 에디션의 이름입니다. 둘을 나란히 놓으면 다음과 같습니다.

| | 컴파일러 버전 | 에디션 |
| --- | --- | --- |
| 이름 | 1.85.0, 1.100.0 … | 2015, 2018, 2021, 2024 |
| 나오는 주기 | 6주마다(The Book 부록 E) | 지금까지 3년 간격 |
| 정하는 곳 | 설치한 툴체인(`rustc --version`) | 각 크레이트의 `Cargo.toml`(`edition`) |
| 바꾸면 | 새 기능과 API가 생기고 기존 코드는 그대로 컴파일됨 | 호환되지 않는 변경이 그 크레이트에만 적용됨 |

에디션은 특정 컴파일러 버전과 함께 안정화됩니다. Edition Guide에 따르면 2018 에디션은 1.31.0, 2021 에디션은 1.56.0, 2024 에디션은 1.85.0과 함께 나왔습니다. 그래서 2024 에디션 크레이트를 컴파일하려면 1.85.0 이상이 필요하고, 그 뒤의 컴파일러는 그때까지 나온 에디션을 모두 지원합니다. Cargo도 이 관계를 알고 있어서, 아래의 `rust-version`을 1.85.0보다 낮게 적은 2024 에디션 패키지는 거절합니다(연습 4).

반대 방향도 성립합니다. **옛 에디션 크레이트가 최신 컴파일러를 요구할 수 있습니다.** `projects/edition_mix`의 `jobs_2015`는 2015 에디션이지만 1.100.0에 안정화된 `bool::toggle`을 쓰므로 rustc 1.100.0 이상이 필요합니다. 이것을 적는 필드가 `rust-version`입니다.

```toml
[package]
name = "jobs_2015"
edition = "2015"        # 소스를 어느 에디션의 규칙으로 읽을지
rust-version = "1.100"  # 컴파일하려면 rustc가 몇 버전 이상이어야 하는지
```

이 환경에 남아 있던 rustc 1.98.1로 이 프로젝트를 빌드하면 Cargo가 컴파일을 시작하기 전에 거절합니다.

```text
error: rustc 1.98.1 is not supported by the following package:
  jobs_2015@0.1.0 requires rustc 1.100
```

예제 02도 환경 변수 `RUSTC`로 다른 rustc를 받을 수 있습니다. rustup으로 설치한 다른 툴체인이 있다면 `rustup which rustc --toolchain <버전>`으로 그 rustc의 경로를 얻습니다. `RUSTC`는 Cargo가 예제를 컴파일할 때 쓰는 rustc도 바꾸므로 예제 자체도 그 버전으로 다시 컴파일됩니다.

```sh
RUSTC=$(rustup which rustc --toolchain 1.98.1) cargo run -p p2-ch1-editions-overview --example 02_rustc_edition_matrix
```

rustc 1.98.1로 실행하자 버전 줄이 `rustc 1.98.1 (48a229cea 2026-09-01)`로 바뀐 것 말고는 표의 마지막 줄 하나만 달랐습니다.

```text
     X    X    X    X    bool::toggle (1.100.0)
```

나머지 일곱 줄은 1.100.0과 똑같았습니다. 에디션에 따른 결과는 그대로이고, 컴파일러 버전이 마지막 줄의 결과를 바꾼 것입니다. `rust-version`을 보고 의존성의 버전을 고르는 2024 에디션의 리졸버는 [Part 2 ch5](../ch5-rust-2024-ecosystem/README.md)에서, `rust-version` 필드 자체는 Part 3에서 다룹니다.

### 2.6 에디션을 적는 곳

**`rustc --edition`.** `rustc`는 `--edition`이 없으면 2015 에디션으로 컴파일합니다([Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md)). 새 문법에서 에러가 나는 것은 물론이고, 2018 에디션 이후의 규칙으로 쓴 평범한 코드도 2015의 규칙으로 읽혀 엉뚱한 에러가 납니다(4.3절). 받는 값은 `2015`, `2018`, `2021`, `2024`, `future`이고, 그 밖의 값은 바로 거절됩니다.

```text
$ rustc --edition 2019 a.rs
error: argument for `--edition` must be one of: <2015|2018|2021|2024|future>. (instead was `2019`)
$ rustc --edition future a.rs
error: the crate requires edition future, but the latest edition supported by this Rust version is 2024
```

**`Cargo.toml`의 `edition`.** Cargo를 쓸 때는 `[package]`의 `edition` 키가 에디션을 정합니다. `cargo new`는 그 Cargo가 아는 가장 최신 안정 에디션(1.100.0에서는 `edition = "2024"`)을 적어 주고, `--edition`으로 다른 값을 고를 수 있습니다.

```console
$ cargo new --edition 2018 foo18
    Creating binary (application) `foo18` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
$ cat foo18/Cargo.toml
[package]
name = "foo18"
version = "0.1.0"
edition = "2018"

[dependencies]
$ cargo new --edition 2019 foo19
error: invalid value '2019' for '--edition <YEAR>'
  [possible values: 2015, 2018, 2021, 2024]

  tip: a similar value exists: '2021'

For more information, try '--help'.
```

**`edition`을 빠뜨리면 2015입니다.** Edition Guide는 에디션 제도가 2017년 말에야 구상되었기 때문에, 에디션을 적지 않은 코드는 하위 호환성을 위해 2015 에디션으로 읽는다고 설명합니다. 그 전에 만든 `Cargo.toml`에는 `edition` 키가 없었으니까요. 대신 1.100.0의 Cargo는 빌드할 때마다 경고합니다. `projects/edition_mix`의 `jobs_2015/Cargo.toml`에서 `edition` 줄을 지우면 다음과 같습니다.

```text
warning: jobs_2015/Cargo.toml: `package.edition` is unspecified, defaulting to `2015` while the latest is `2024`
warning: `jobs_2015` (manifest) generated 1 warning
```

없는 연도는 매니페스트를 읽는 단계에서 에러입니다. 2015와 2024 사이의 없는 연도와 2024보다 뒤의 연도는 메시지가 다릅니다. 뒤의 연도는 "이 Cargo가 아직 모르는 미래의 에디션"으로 다룹니다(2025, 2026, 2030도 같은 메시지였습니다).

```text
error: failed to parse manifest at `…/Cargo.toml`

Caused by:
  failed to parse the `edition` key

Caused by:
  supported edition values are `2015`, `2018`, `2021`, or `2024`, but `2019` is unknown
```

```text
Caused by:
  this version of Cargo is older than the `2027` edition, and only supports `2015`, `2018`, `2021`, and `2024` editions.
```

**이 저장소의 방식.** 챕터 패키지의 `Cargo.toml`에는 `edition = "2024"` 대신 `edition.workspace = true`가 적혀 있습니다. 루트 `Cargo.toml`의 `[workspace.package]`에 있는 `edition = "2024"`를 물려받는다는 뜻입니다. 워크스페이스 안에서도 에디션은 패키지마다 정해지는 값이고, 물려받기는 같은 값을 여러 번 적지 않게 해 주는 편의 기능일 뿐입니다. 워크스페이스 설정은 Part 3에서 다룹니다.

**타깃별 `edition`은 쓰지 않습니다.** Cargo에는 `[lib]`, `[[bin]]`, `[[example]]` 같은 타깃마다 `edition`을 따로 적는 키도 있지만, 1.100.0에서 적으면 `` warning: Cargo.toml: `edition` is set on binary `…` which is deprecated ``가 나옵니다. 더는 권장하지 않는(deprecated) 키라는 뜻입니다. 에디션을 다르게 하고 싶으면 패키지를 나눕니다. 이 파트의 `projects/`가 그렇게 되어 있습니다.

**Cargo가 넘기는 값 확인하기.** `cargo build -v`는 Cargo가 실행하는 `rustc` 명령을 보여 줍니다. `projects/edition_mix`에서 처음부터 빌드하면(`cargo clean` 뒤) 크레이트마다 다른 `--edition`이 넘어가는 것이 보입니다. rustc의 경로와 명령의 뒷부분은 `…`로 줄였습니다.

```text
     Running `…/rustc --crate-name idgen_2021 --edition=2021 idgen_2021/src/lib.rs …`
     Running `…/rustc --crate-name jobs_2015 --edition=2015 jobs_2015/src/lib.rs …`
     Running `…/rustc --crate-name app_2024 --edition=2024 app_2024/src/main.rs …`
```

같은 rustc가 세 번, 매번 다른 `--edition`으로 실행되었습니다. 앞의 두 줄은 동시에 컴파일되므로 순서가 바뀔 수 있습니다. `cargo fmt -v`도 크레이트마다 `rustfmt --edition 2015 …`처럼 그 크레이트의 에디션을 넘깁니다. rustfmt도 코드를 읽으려면 에디션을 알아야 하기 때문입니다(4.7절).

**실행 중인 프로그램은 자기 에디션을 모릅니다.** 에디션은 컴파일할 때 소스를 읽는 규칙일 뿐입니다. `cargo run`이 프로그램에 넣어 주는 `CARGO_PKG_*` 환경 변수에도 에디션은 없습니다. 예제 01이 에디션 값을 출력하는 대신 에디션에 따라 달라지는 동작을 보여 주는 이유입니다.

### 2.7 에디션이 다른 크레이트를 한 프로그램으로: `projects/edition_mix`

```text
projects/edition_mix/
├── Cargo.toml     # 가상 매니페스트: [workspace] resolver = "3", members = [세 패키지]
├── jobs_2015/     # 라이브러리, edition = "2015", rust-version = "1.100"
├── idgen_2021/    # 라이브러리, edition = "2021"
└── app_2024/      # 바이너리, edition = "2024". 위의 두 라이브러리에 의존
```

[`app_2024/Cargo.toml`](projects/edition_mix/app_2024/Cargo.toml)의 의존성 목록에는 에디션이 없습니다. 에디션은 각 크레이트가 자기 `Cargo.toml`에서 정하고, 쓰는 쪽은 상대의 에디션을 알 필요가 없기 때문입니다.

```toml
[dependencies]
jobs_2015 = { path = "../jobs_2015" }
idgen_2021 = { path = "../idgen_2021" }
```

최상위 [`Cargo.toml`](projects/edition_mix/Cargo.toml)은 `[package]`가 없는 가상 매니페스트라서 `edition`이 없습니다. 그래서 의존성 리졸버의 버전을 `resolver = "3"`으로 직접 적었습니다. 이 줄을 지우면 `` warning: virtual workspace defaulting to `resolver = "1"` despite one or more workspace members being on edition 2024 which implies `resolver = "3"` ``가 나옵니다. 리졸버의 기본값도 에디션이 정하는 "도구의 기본값"(2.4절)의 하나입니다.

**2015 크레이트의 `async` 함수.** [`jobs_2015`](projects/edition_mix/jobs_2015/src/lib.rs)는 2015 에디션이므로 `async`와 `await`를 함수 이름으로 씁니다. [`idgen_2021`](projects/edition_mix/idgen_2021/src/lib.rs)은 2021 에디션이므로 `gen`이라는 함수를 둡니다.

```rust
// jobs_2015/src/lib.rs (2015 에디션)
pub fn async(name: &str, steps: u32) -> Job { /* … */ }
pub fn await(job: Job) -> String { /* … */ }

// idgen_2021/src/lib.rs (2021 에디션)
pub fn gen(prefix: &str, n: u32) -> Option<String> { /* … */ }
```

2024 에디션인 [`app_2024`](projects/edition_mix/app_2024/src/main.rs)에서는 세 이름이 모두 키워드라서 그대로 쓰면 에러입니다(4.5절). 이럴 때 raw 식별자(raw identifier)를 씁니다. 이름 앞에 `r#`를 붙이면 "키워드가 아니라 글자 그대로의 이름"이라는 뜻이 되어, 2024 에디션 코드에서도 그 이름의 항목을 가리킬 수 있습니다. raw 식별자의 자세한 규칙은 [Part 2 ch2](../ch2-rust-2015-2018/README.md)에서 다룹니다.

```rust
// app_2024/src/main.rs (2024 에디션)
let mut job = jobs_2015::r#async("백업", 3);
job.toggle_pause(); // 2015 크레이트의 타입과 메서드도 그대로 씁니다
let report = jobs_2015::r#await(job);
let small = idgen_2021::r#gen("user", 7);
```

`r#async`는 2024 에디션 쪽에서 쓰는 표기이고, 가리키는 것은 `jobs_2015`가 정의한 `async` 함수 그대로입니다. `jobs_2015`는 바꿀 것이 없습니다.

**각 크레이트는 자기 에디션의 문법을 씁니다.** `idgen_2021`은 `use` 없이 `u16::try_from`을 씁니다(2021 프렐류드). `app_2024`는 let 체인 안에서 두 크레이트의 함수를 함께 부릅니다. let 체인을 쓸 수 있는지는 그 코드를 쓴 `app_2024`의 에디션이 정하고, 부르는 함수를 만든 크레이트의 에디션과는 상관없습니다.

```rust
if let Some(id) = idgen_2021::r#gen("job", 42)
    && jobs_2015::initial(&id) == 'j'
{
    println!("    let 체인 안에서 두 크레이트의 함수를 함께: {id}");
}
```

`jobs_2015`는 2015 에디션이지만 let-else와 1.100.0의 `bool::toggle`을 씁니다. 2.4절의 "바꾸지 않는 것"이 이 크레이트 안에서도 보입니다.

### 2.8 매크로 본문은 매크로를 정의한 크레이트의 에디션으로 읽힙니다

함수는 컴파일된 뒤의 모습으로 다른 크레이트에 건너가지만, `macro_rules!` 매크로는 토큰째로 건너가 쓰는 크레이트 안에서 펼쳐집니다([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.19절). 그러면 펼쳐진 코드는 어느 에디션의 규칙으로 읽힐까요? 답은 토큰마다 다릅니다. 컴파일러는 토큰마다 어느 크레이트의 소스에서 왔는지를 기억해 두고 그 크레이트의 에디션을 적용합니다. 그래서 **매크로 본문에 적힌 토큰은 매크로를 정의한 크레이트의 에디션**을, 매크로를 부를 때 넘긴 인자의 토큰은 부른 크레이트의 에디션을 따릅니다.

`jobs_2015`의 `run_now!`는 본문에서 `async`를 변수 이름으로 씁니다.

```rust
// jobs_2015/src/lib.rs (2015 에디션)
#[macro_export]
macro_rules! run_now {
    ($name:expr) => {{
        let async = $crate::async($name, 1);
        $crate::await(async)
    }};
}
```

2024 에디션인 `app_2024`에서 `jobs_2015::run_now!("정리")`를 불러도 컴파일되고 `정리: 1단계 완료`가 나옵니다. 본문의 `async`는 2015 에디션 크레이트에서 온 토큰이기 때문입니다.

키워드만 그런 것이 아닙니다. 세 크레이트에 본문이 똑같은 `first_item!` 매크로를 두고 같은 배열을 넘기면 결과 타입이 갈립니다.

```rust
macro_rules! first_item {
    ($array:expr) => {
        $array.into_iter().next()
    };
}
```

```text
    2015 크레이트의 매크로: Some(10), core::option::Option<&i32>
    2021 크레이트의 매크로: Some(10), core::option::Option<i32>
    2024 크레이트의 매크로: Some(10), core::option::Option<i32>
```

배열 `numbers`는 세 번 모두 `app_2024`에서 넘긴 것이지만, `.into_iter()`라는 토큰은 각 매크로의 본문에서 왔습니다. 2015·2018 에디션에서는 배열에 메서드 호출 문법으로 `.into_iter()`를 부르면 참조를 내놓으므로(2.4절의 표), `jobs_2015`의 매크로만 `Option<&i32>`가 됩니다. 자세한 내용은 [Part 2 ch3](../ch3-rust-2021/README.md)에서 다룹니다.

이 규칙 덕분에 라이브러리의 매크로는 그것을 쓰는 크레이트가 에디션을 올려도 뜻이 바뀌지 않습니다. 반대로 라이브러리가 자기 에디션을 올리면 매크로 본문이 새 규칙으로 읽히므로, 매크로를 고치는 것은 라이브러리 작성자의 몫입니다. 이때 매크로 본문의 에러는 라이브러리가 아니라 **매크로를 쓰는 크레이트를 컴파일할 때** 드러납니다(연습 1).

### 2.9 에디션별 대표 변경 한눈에 보기

Edition Guide의 목차를 기준으로 정리했습니다. 처음 보는 항목이 많아도 괜찮습니다. 이 표는 "어디서 다루는지"를 찾기 위한 지도입니다.

| 에디션 | 함께 나온 버전 | 분야 | 대표 변경 | 다루는 곳 |
| --- | --- | --- | --- | --- |
| 2015 | 1.0 | 기준선 | 에디션 제도 이전의 Rust입니다. `edition`을 적지 않으면 이 에디션입니다 | [ch2](../ch2-rust-2015-2018/README.md) |
| 2018 | 1.31.0 | 언어 | 경로와 모듈 시스템 정리(`use` 경로를 다른 경로와 같은 규칙으로), `async`·`await`·`dyn` 키워드와 `try` 예약 | [ch2](../ch2-rust-2015-2018/README.md) |
| 2018 | | 언어·Cargo | 트레이트 메서드의 이름 없는 매개변수 금지(2015에서는 deprecated 경고), Cargo의 타깃 자동 탐색 규칙 변경 | [ch2](../ch2-rust-2015-2018/README.md) |
| 2021 | 1.56.0 | 언어 | 배열의 `into_iter()`가 값을 내놓음, 클로저 분리 캡처, `panic!` 매크로 일관성, 예약 문법(`c"…"` 같은 접두사 자리), `dyn` 없는 트레이트 객체가 에러 | [ch3](../ch3-rust-2021/README.md) |
| 2021 | | 프렐류드·Cargo | 프렐류드에 `TryFrom`·`TryInto`·`FromIterator` 추가, 기능 리졸버 2가 기본 | [ch3](../ch3-rust-2021/README.md) |
| 2024 | 1.85.0 | 언어 | let 체인, `if let`·마지막 표현식의 임시값 범위, RPIT 라이프타임 캡처, `unsafe extern`·unsafe 속성·`static mut` 참조 금지, `gen` 예약 | [ch4](../ch4-rust-2024-language/README.md) |
| 2024 | | 표준 라이브러리 | 프렐류드에 `Future`·`IntoFuture` 추가, `Box<[T]>`의 `into_iter()`, `env::set_var` 등이 unsafe 함수로 | [ch5](../ch5-rust-2024-ecosystem/README.md) |
| 2024 | | 도구 | `rust-version`을 고려하는 리졸버 3, rustdoc 문서 테스트 병합, rustfmt 스타일 에디션 | [ch5](../ch5-rust-2024-ecosystem/README.md) |

예제 02의 표에서 에디션에 따라 갈린 다섯 줄은 이 표의 2018(`async` 키워드, `use` 경로), 2021(프렐류드), 2024(`gen` 예약, let 체인) 항목입니다. Edition Guide 5.1.10절의 never 타입 폴백 변경은 2.4절에서 본 대로 1.100부터 모든 에디션에 적용되므로 표에서 뺐습니다.

**`rust-next/`와 다음 에디션.** 1.100.0 툴체인에 들어 있는 로컬 Edition Guide에는 `rust-next/` 디렉터리가 있지만 목차에는 없습니다. 그 안의 파일 16개는 모두 다른 곳으로 이동시키는 안내 페이지입니다. `rust-next/index.html`은 Rust 2021 장으로, 나머지는 `dbg!`·`Pin`·`Future`의 표준 라이브러리 문서, `cargo vendor` 문서, const 평가에 관한 Reference 절, 그리고 1.32~1.36의 릴리스 공지(`TryFrom`·`TryInto`, `alloc` 크레이트, `MaybeUninit` 등)로 옮겨 갑니다. 파일 이름과 옮겨 가는 곳으로 보아, 2018 에디션 이후 일반 릴리스로 들어온 기능을 소개하던 옛 페이지의 주소를 남겨 둔 것으로 보입니다(옛 페이지 자체는 확인하지 못했습니다). 다음 에디션에 관한 내용은 없습니다. 그 기능들은 에디션이 아니라 컴파일러 버전을 따라 들어왔으므로, 2.4절의 "대부분의 새 기능은 모든 에디션에 들어온다"의 예이기도 합니다. 그중 `dbg!`와 `std::convert::TryFrom`이 2015 에디션에서 동작하는 것은 직접 확인했습니다.

다음 에디션의 흔적은 도구 쪽에 있습니다. `rustc --edition future`와 `Cargo.toml`의 `edition = "future"`는 아직 이름이 없는 다음 에디션을 nightly에서 시험하기 위한 값입니다. 1.100.0 안정 툴체인에서는 앞의 것이 2.6절의 에러로, 뒤의 것이 `` feature `unstable-editions` is required ``로 거절됩니다. nightly는 Part 11 `ch1-nightly-toolchain`에서 다루며, 이 챕터에서는 nightly로 확인하지 못했습니다.

### 2.10 이 챕터에서 다루지 않는 것

| 주제 | 다루는 곳 |
| --- | --- |
| 각 에디션의 변경 상세 | [Part 2 ch2](../ch2-rust-2015-2018/README.md), [ch3](../ch3-rust-2021/README.md), [ch4](../ch4-rust-2024-language/README.md), [ch5](../ch5-rust-2024-ecosystem/README.md) |
| raw 식별자의 자세한 규칙 | [Part 2 ch2](../ch2-rust-2015-2018/README.md) |
| `cargo fix --edition`, 에디션 호환성 린트 그룹, 마이그레이션 절차 | [Part 2 ch6](../ch6-migration-practice/README.md) |
| 리졸버 버전과 `rust-version`을 고려하는 리졸버 | [Part 2 ch3](../ch3-rust-2021/README.md), [ch5](../ch5-rust-2024-ecosystem/README.md) |
| `rust-version` 필드, 워크스페이스와 매니페스트 상세 | Part 3 |
| 릴리스 채널, nightly, `edition = "future"` | Part 11 `ch1-nightly-toolchain` |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_edition_effects.rs`](examples/01_edition_effects.rs) | 이 크레이트가 2024 에디션이라서 보이는 동작 다섯 가지 | 4 |
| [`02_rustc_edition_matrix.rs`](examples/02_rustc_edition_matrix.rs) | 같은 rustc에 `--edition`만 바꿔 코드 조각 여덟 개를 컴파일한 결과 | 1, 2, 4, 5 |
| [`projects/edition_mix`](projects/edition_mix/Cargo.toml) | 2015·2021·2024 에디션 크레이트를 한 프로그램으로 링크, raw 식별자, 에디션 경계를 넘는 매크로 | 2, 3, 4, 5 |
| [`compile_fail/`](compile_fail/keyword_async_identifier.rs) | 에디션에 따라 결과가 갈리는 컴파일 에러 4개(4절) | 1, 5 |

예제 명령은 저장소 루트에서 실행합니다. 두 예제는 패닉 없이 종료 코드 0으로 곧바로 끝납니다.

### 01_edition_effects — 이 크레이트의 에디션이 정한 동작들

```sh
cargo run -p p2-ch1-editions-overview --example 01_edition_effects
```

```text
1. 배열의 into_iter()는 원소를 값으로 내놓습니다 (2021 에디션부터)
    Some(10), core::option::Option<i32>
2. TryFrom은 use 없이 씁니다 (2021 에디션 프렐류드)
    u8::try_from(300) = Err(TryFromIntError(PosOverflow))
3. let 체인 (2024 에디션 전용 문법)
    42는 10보다 큽니다
4. if let 조건식의 임시값은 else로 가기 전에 버려집니다 (2024 에디션)
    else 안에서 가변 대여: true
5. gen은 예약 키워드입니다 (2024 에디션)
    r#gen(1) = 1103527590
```

- 옛 에디션에서의 결과는 2.4절의 표에 있습니다.
- **4번 구역**: `log.borrow()`가 만든 `Ref` 가드는 조건식의 임시값입니다. 2024 에디션에서는 패턴이 맞지 않으면 `else`에 들어가기 전에 버려지므로 `else` 안에서 다시 가변으로 빌릴 수 있습니다([Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.10절).
- **5번 구역**: `fn r#gen`처럼 정의할 때도 `r#`를 붙입니다. 이 함수를 2021 에디션 크레이트에서 부르면 그냥 `gen(1)`로 부를 수 있습니다.

### 02_rustc_edition_matrix — rustc 하나, 에디션 넷

```sh
cargo run -p p2-ch1-editions-overview --example 02_rustc_edition_matrix
```

```text
1. 이 rustc가 아는 에디션
    rustc 1.100.0 (0ef1bc386 2026-11-09)
    --edition <2015|2018|2021|2024|future>
2. 같은 rustc에 --edition만 바꿔 컴파일한 결과 (O: 컴파일됨, X: 에러)
    2015 2018 2021 2024  코드 조각
     O    O    O    O    기준: println! 한 줄
     O    X    X    X    async를 변수 이름으로
     X    O    O    O    하위 모듈 안에서 use circle::area
     X    X    O    O    use 없이 u8::try_from
     O    O    O    X    gen을 변수 이름으로
     X    X    X    O    let 체인
     O    O    O    O    let-else (1.65.0)
     O    O    O    O    bool::toggle (1.100.0)
```

- 둘째 줄(`rustc` 버전)은 툴체인에 따라 다릅니다. rustc 1.100.0보다 낮은 툴체인이라면 마지막 줄이 `X`로 바뀝니다(2.5절).
- 예제는 임시 디렉터리(`rust-p2-ch1-matrix-<프로세스 ID>`)에 코드 조각을 파일로 쓰고, 조각마다 네 에디션의 `rustc --edition <연도> --emit=metadata`를 동시에 실행한 뒤 성공 여부만 봅니다. 실행 파일은 만들지 않으며, 끝나면 임시 디렉터리를 지웁니다. 이 환경에서 1초 안에 끝났습니다.
- `rustc`는 환경 변수 `RUSTC`가 있으면 그 경로를, 없으면 `PATH`에서 찾습니다. 찾지 못하면 `rustc를 실행하지 못했습니다`라고 출력하고 끝납니다.
- 예제는 에러 메시지를 버리고 성공 여부만 표시합니다. 메시지는 4절에서 봅니다. `async` 줄은 4.1절, `use circle::area` 줄은 4.3절, `u8::try_from` 줄은 4.7절, `gen` 줄은 4.2절, let 체인 줄은 4.4절입니다.

### projects/edition_mix — 에디션이 다른 크레이트 셋을 한 프로그램으로

```sh
cd part2-editions/ch1-editions-overview/projects/edition_mix
cargo run
```

```text
1. 에디션이 다른 크레이트의 함수 부르기
    r#async → r#await: 백업: 3단계 일시정지
    r#gen: Some("user-00007"), None
2. 크레이트마다 자기 에디션의 문법을 씁니다
    let 체인 안에서 두 크레이트의 함수를 함께: job-00042
3. 매크로 본문은 매크로를 정의한 크레이트의 에디션으로 읽힙니다
    jobs_2015::run_now!: 정리: 1단계 완료
    2015 크레이트의 매크로: Some(10), core::option::Option<&i32>
    2021 크레이트의 매크로: Some(10), core::option::Option<i32>
    2024 크레이트의 매크로: Some(10), core::option::Option<i32>
```

- **1번 구역**: `job.toggle_pause()`를 한 번 불렀으므로 "일시정지"입니다. `r#gen("user", 70_000)`이 `None`인 것은 번호가 `u16`의 범위를 넘기 때문입니다.
- 실행 파일이 `app_2024` 하나뿐이라 `cargo run`만으로 실행됩니다. 이 디렉터리는 루트 워크스페이스와 분리된 독립 워크스페이스이므로, 저장소 루트의 `cargo build --workspace`에는 포함되지 않습니다.

크레이트마다 넘어가는 에디션만 골라 보려면 다음처럼 합니다(macOS·Linux의 셸 기준).

```sh
cargo clean
cargo build -v 2>&1 | grep -o -- '--crate-name [a-z0-9_]* --edition=[0-9]*' | sort
```

```text
--crate-name app_2024 --edition=2024
--crate-name idgen_2021 --edition=2021
--crate-name jobs_2015 --edition=2015
```

검사 도구도 크레이트마다 그 에디션으로 실행됩니다. 다음 두 명령은 경고 없이 통과합니다.

```sh
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 어떤 에디션에서 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일하고, 파일 맨 위의 주석에 적힌 대로 `--edition`을 바꿔 가며 결과를 비교합니다. 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part2-editions/ch1-editions-overview
rustc --edition 2024 compile_fail/keyword_async_identifier.rs --out-dir /tmp/rust-p2-ch1   # 에러
rustc --edition 2015 compile_fail/keyword_async_identifier.rs --out-dir /tmp/rust-p2-ch1   # 통과
```

| 파일 | 실패하는 에디션 | 통과하는 에디션 |
| --- | --- | --- |
| `keyword_async_identifier.rs` | 2018, 2021, 2024 | 2015 |
| `reserved_keyword_gen.rs` | 2024 | 2015, 2018, 2021 |
| `e0432_use_relative_path_2015.rs` | 2015 (`--edition`을 빠뜨린 경우) | 2018, 2021, 2024 |
| `let_chains_2021.rs` | 2015, 2018, 2021 | 2024 |

### 4.1 2018 이후 에디션에서 `async`를 이름으로 씀

파일: [`compile_fail/keyword_async_identifier.rs`](compile_fail/keyword_async_identifier.rs)

```text
error: expected identifier, found keyword `async`
  --> compile_fail/keyword_async_identifier.rs:16:9
   |
16 |     let async = message.len() > 10; // → 2018 에디션부터 에러
   |         ^^^^^ expected identifier, found keyword
   |
help: escape `async` to use it as an identifier
   |
16 |     let r#async = message.len() > 10; // → 2018 에디션부터 에러
   |         ++

error: this `if` expression is missing a block after the condition
  --> compile_fail/keyword_async_identifier.rs:17:5
   |
17 |     if async {
   |     ^^

error[E0308]: mismatched types
  --> compile_fail/keyword_async_identifier.rs:17:8
   |
17 |       if async {
   |  ________^
18 | |         format!("[나중에 보냄] {}", message)
19 | |     } else {
   | |_____^ expected `bool`, found `async` block
```

(뒤의 두 에러는 `help:`와 `note:` 줄을 줄였습니다.)

- **읽는 법**: 첫 에러가 원인입니다. 이름(identifier)이 와야 할 자리에 키워드가 왔다는 뜻이고, `help:`가 raw 식별자 `r#async`를 제안합니다. 뒤의 두 에러는 그 여파입니다. 2018 이후의 에디션에서 `async { … }`는 async 블록이므로, 컴파일러는 `if async { … }`를 "조건이 async 블록인 `if`"로 읽었습니다. 같은 글자가 에디션에 따라 전혀 다른 뜻이 된 것입니다.
- **원인**: 2015 에디션에서 쓴 코드를 2018 이후의 에디션으로 컴파일했습니다. 같은 파일을 `--edition 2015`로 컴파일하면 경고 없이 통과합니다.
- **해결**: 이름을 바꿀 수 있다면 `send_later`처럼 키워드가 아닌 이름으로 바꿉니다. 다른 크레이트가 이미 그 이름으로 공개해서 바꿀 수 없다면 `r#async`라고 씁니다(2.7절). 에디션을 올리면서 생긴 에러라면 `cargo fix --edition`이 `r#`를 붙여 줍니다([Part 2 ch6](../ch6-migration-practice/README.md)).

### 4.2 2024 에디션에서 `gen`을 이름으로 씀

파일: [`compile_fail/reserved_keyword_gen.rs`](compile_fail/reserved_keyword_gen.rs)

```text
error: expected identifier, found reserved keyword `gen`
  --> compile_fail/reserved_keyword_gen.rs:15:4
   |
15 | fn gen(seed: u32) -> u32 {
   |    ^^^ expected identifier, found reserved keyword
   |
help: escape `gen` to use it as an identifier
   |
15 | fn r#gen(seed: u32) -> u32 {
   |    ++

error: expected expression, found reserved keyword `gen`
  --> compile_fail/reserved_keyword_gen.rs:20:17
   |
20 |     let first = gen(1); // → 2024 에디션에서 에러
   |                 ^^^ expected expression
```

- **읽는 법**: 4.1절과 같은 모양이지만 `keyword`가 아니라 `reserved keyword`(예약 키워드)입니다. 예약 키워드는 아직 쓰이는 문법은 없지만 나중을 위해 자리를 잡아 둔 키워드입니다. Edition Guide 5.1.13절은 반복자를 쉽게 만드는 gen 블록을 위해 2024 에디션에서 `gen`을 예약했다고 설명합니다.
- **원인**: 2021 에디션까지 평범한 이름이던 `gen`을 2024 에디션 코드에서 썼습니다. "2021에서는 되고 2024에서는 안 되는" 코드의 대표적인 예입니다.
- **해결**: 정의와 호출 모두 `r#gen`으로 씁니다(예제 01의 5번 구역). 새로 짓는 이름이라면 `next_random`처럼 다른 이름을 고르는 편이 읽기 좋습니다.

### 4.3 `--edition` 없이 `rustc`로 컴파일함 — E0432

파일: [`compile_fail/e0432_use_relative_path_2015.rs`](compile_fail/e0432_use_relative_path_2015.rs)

```sh
rustc compile_fail/e0432_use_relative_path_2015.rs --out-dir /tmp/rust-p2-ch1
```

```text
error[E0432]: unresolved import `circle`
  --> compile_fail/e0432_use_relative_path_2015.rs:22:9
   |
22 |     use circle::area; // → 2015 에디션에서 E0432
   |         ^^^^^^
   |
help: a similar path exists
   |
22 |     use self::circle::area; // → 2015 에디션에서 E0432
   |         ++++++
```

- **읽는 법**: `use` 경로의 `circle`을 찾지 못했다는 에러입니다. 메시지 어디에도 "에디션"이라는 말이 없다는 점에 주의합니다. 코드가 틀린 것처럼 보이지만, 같은 파일을 `--edition 2018` 이상으로 컴파일하면 통과합니다.
- **원인**: `--edition`을 빠뜨려 2015 에디션으로 컴파일했습니다. 2015 에디션에서 `use` 경로는 언제나 크레이트 루트에서 시작하므로 `shapes` 모듈 안의 `circle`을 크레이트 루트에서 찾다가 실패했습니다. 2018 에디션부터는 `use` 경로도 다른 경로와 같은 규칙을 따르므로 현재 모듈에 있는 이름(`circle`)으로 시작할 수 있습니다([Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.7절).
- **해결**: `rustc --edition 2024 …`로 컴파일합니다. 단일 파일을 `rustc`로 직접 컴파일할 때는 언제나 `--edition`을 붙이고, 이해할 수 없는 에러가 나면 먼저 에디션을 의심합니다. `help:`의 `self::circle::area`는 두 에디션에서 모두 통하는 표기입니다.

### 4.4 2021 이전 에디션에서 let 체인을 씀

파일: [`compile_fail/let_chains_2021.rs`](compile_fail/let_chains_2021.rs)

```text
error: let chains are only allowed in Rust 2024 or later
  --> compile_fail/let_chains_2021.rs:15:8
   |
15 |     if let Ok(n) = input.parse::<u32>()
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

- **읽는 법**: 앞의 세 경우와 달리 이 에러는 필요한 에디션(`Rust 2024 or later`)을 직접 알려 줍니다. 에러 메시지가 에디션을 말해 주는 경우도 있고(4.4절), 말해 주지 않는 경우도 있습니다(4.1~4.3절).
- **원인**: let 체인은 2024 에디션에서 바뀐 `if let`의 임시값 범위를 전제로 하므로 2024 에디션 전용입니다(2.4절). 새 컴파일러를 써도 크레이트의 에디션이 2021이면 쓸 수 없습니다.
- **해결**: 크레이트의 에디션을 2024로 올리거나, 에디션을 그대로 둔다면 `if let` 안에 `if`를 중첩합니다([Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.2절).

### 4.5 다른 에디션 크레이트의 함수를 키워드 이름 그대로 부름

이 실수는 크레이트가 둘 이상 필요해서 `compile_fail/`에 두지 않았습니다. `projects/edition_mix/app_2024/src/main.rs`에서 `jobs_2015::r#async(`를 `jobs_2015::async(`로 바꾸고 `cargo build`를 실행하면 다음 에러가 납니다.

```text
error: expected identifier, found keyword `async`
  --> app_2024/src/main.rs:23:30
   |
23 |     let mut job = jobs_2015::async("백업", 3);
   |                              ^^^^^ expected identifier, found keyword
   |
help: escape `async` to use it as an identifier
   |
23 |     let mut job = jobs_2015::r#async("백업", 3);
   |                              ++
```

- **원인**: `jobs_2015`에서 `async`는 평범한 함수 이름이지만, 그 이름을 쓰는 곳은 2024 에디션인 `app_2024`의 소스입니다. 이름을 읽는 규칙은 그 이름을 **쓴** 크레이트의 에디션이 정합니다.
- **해결**: `help:`대로 `jobs_2015::r#async`라고 씁니다. 함수를 만든 쪽(`jobs_2015`)은 고칠 필요가 없습니다.

### 4.6 `Cargo.toml`에서 `edition`을 빠뜨림

경고만 나고 빌드는 성공하므로 놓치기 쉽습니다(2.6절).

```text
warning: jobs_2015/Cargo.toml: `package.edition` is unspecified, defaulting to `2015` while the latest is `2024`
```

- **원인**: `edition` 키가 없으면 Cargo는 하위 호환성을 위해 2015 에디션으로 컴파일합니다. 손으로 `Cargo.toml`을 만들거나 다른 곳에서 `[package]` 부분만 복사해 왔을 때 생깁니다.
- **해결**: `edition = "2024"`처럼 명시합니다. 옛 코드를 그대로 유지하려는 것이라면 `edition = "2015"`라고 적어 경고를 없애고 의도를 드러냅니다. 2015로 컴파일되는 줄 모르고 새 문법을 쓰면 4.3절·4.4절과 같은 에러가 납니다.

### 4.7 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| 2018 에디션 크레이트에서 `use` 없이 `u8::try_from(n)` | E0599 `` no associated function or constant named `try_from` found for type `u8` ``. `help:`는 `use std::convert::TryFrom;`을 제안하지만 에디션은 말하지 않음 | `use`를 추가하거나 에디션을 2021 이상으로 (2.4절) |
| 2024 에디션 파일에 `--edition` 없이 `rustfmt` 실행 | 이 환경에서 rustfmt는 2015 에디션으로 읽어 `let chains are only allowed in Rust 2024 or later`로 실패 | `cargo fmt`를 쓰거나 `rustfmt --edition 2024 파일` (2.6절) |
| 2015 에디션 파일에 `rustfmt --edition 2024` | `` expected identifier, found keyword `async` `` | 파일의 에디션에 맞춰 `--edition 2015` |
| `[[bin]]` 등 타깃 테이블에 `edition` | `` `edition` is set on binary `…` which is deprecated `` 경고 | 패키지를 나눔 (2.6절) |
| 가상 매니페스트에 `resolver`가 없고 멤버 중에 2024 에디션이 있음 | `` virtual workspace defaulting to `resolver = "1"` `` 경고 | `[workspace]`에 `resolver = "3"` (2.7절) |
| `cargo new --edition 2019`, `edition = "2019"`, `rustc --edition 2019` | 모두 에러. 없는 에디션 | `2015`, `2018`, `2021`, `2024` 중 하나 (2.6절) |
| 2015 에디션 크레이트에 `async fn` | E0670 `` `async fn` is not permitted in Rust 2015 `` | 에디션을 2018 이상으로 ([Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md) 자주 하는 실수 6) |

## 5. 연습 문제

### 연습 1. 2015 크레이트의 에디션을 올려 보기

`projects/edition_mix/jobs_2015/Cargo.toml`의 `edition`을 `"2018"`로 바꾸고 `cargo run`을 실행하세요. 어떤 에러가 나는지 확인하고, `jobs_2015`의 코드를 고쳐 다시 실행되게 만드세요. 그다음 `"2021"`로 바꿔 3번 구역의 출력이 어떻게 달라지는지 확인하세요.

<details>
<summary>힌트</summary>

- 2018로 바꾸면 먼저 `jobs_2015` 자신의 `pub fn async`와 `pub fn await`에서 에러가 납니다. 두 함수의 정의를 `pub fn r#async`, `pub fn r#await`로 바꿉니다. 함수를 부르는 `app_2024`는 이미 `r#`를 쓰므로 고칠 필요가 없습니다.
- 함수만 고치면 `jobs_2015`는 컴파일되지만, 이번에는 `app_2024`를 컴파일할 때 `jobs_2015::run_now!("정리")` 줄에서 `` expected identifier, found keyword `async` `` 등 에러 4개가 납니다. 각 에러는 `jobs_2015/src/lib.rs`의 매크로 본문(62·63번 줄)도 함께 가리킵니다. 매크로 본문은 쓰는 쪽에서 펼쳐질 때 비로소 읽히기 때문입니다(2.8절). 본문에 나오는 `async` 세 곳과 `await` 한 곳을 모두 `r#async`, `r#await`로 고칩니다.
- 2018에서는 `jobs_2015::first_item!`이 여전히 `Option<&i32>`이고, 2021로 바꾸면 `Option<i32>`가 됩니다. 배열의 `into_iter()`가 바뀐 것이 2021 에디션이기 때문입니다.
- 이 작업은 2.2절에서 말한 마이그레이션을 손으로 한 것입니다. `cargo fix --edition`은 [Part 2 ch6](../ch6-migration-practice/README.md)에서 다룹니다. 끝나면 `git checkout -- .`(이 프로젝트 디렉터리에서)으로 되돌립니다.

</details>

### 연습 2. 다음 에디션에서 깨질 이름 미리 찾기

`jobs_2015/src/lib.rs` 맨 위에 `#![warn(keyword_idents)]`를, `idgen_2021/src/lib.rs` 맨 위에 `#![warn(keyword_idents_2024)]`를 추가하고 `cargo build`를 실행하세요. 경고가 몇 개 나오는지, 경고 문구가 무엇을 알려 주는지 정리하세요.

<details>
<summary>힌트</summary>

- `jobs_2015`에서는 `` `async` is a keyword in the 2018 edition ``이 4개, `` `await` is a keyword in the 2018 edition ``이 2개, 모두 6개가 나왔습니다. 매크로 본문의 이름도 포함됩니다.
- 각 경고의 `help:`는 `` you can use a raw identifier to stay compatible: `r#async` ``이고, `warning:` 줄은 `this is accepted in the current edition (Rust 2015) but is a hard error in Rust 2018!`입니다. 지금 에디션에서는 괜찮지만 다음 에디션에서는 에러가 된다는 뜻입니다.
- `idgen_2021`에서는 `` `gen` is a keyword in the 2024 edition `` 경고 1개가 나옵니다.
- 이 린트(lint)들은 기본 린트 수준이 allow(꺼짐)입니다(`rustc -W help`로 확인). `cargo fix --edition`은 이런 에디션 호환성 린트를 켜고 그 제안을 적용하는 방식으로 동작합니다([Part 2 ch6](../ch6-migration-practice/README.md)).

</details>

### 연습 3. 에디션 표에 줄 더하기

[`02_rustc_edition_matrix.rs`](examples/02_rustc_edition_matrix.rs)의 `SNIPPETS`에 두 줄을 추가하세요. 하나는 C 문자열 리터럴 `fn main() { let s = c"hi"; println!("{:?}", s); }`, 다른 하나는 `dyn` 없는 트레이트 객체 `fn call(f: Box<Fn()>) { f() } fn main() { call(Box::new(|| {})); }`입니다. 실행하기 전에 각 줄의 결과를 2.9절의 표로 예상해 보세요. 배열의 크기 `8`도 바꿔야 합니다.

<details>
<summary>힌트</summary>

- C 문자열 리터럴은 `X X O O`였습니다. 리터럴 자체는 1.77.0에 안정화되었지만, 2018 이전 에디션에서 `c"hi"`는 이름 `c`와 문자열 `"hi"`로 나뉘어 읽혀 문법 에러가 납니다. 2021 에디션이 `이름"…"` 꼴을 예약 문법으로 비워 두었기 때문에 그 자리에 들어올 수 있었습니다([Part 2 ch3](../ch3-rust-2021/README.md)).
- `Box<Fn()>`은 `O O X X`였습니다. 2015·2018 에디션에서는 `trait objects without an explicit dyn are deprecated` 경고만 나고 컴파일되므로 `O`입니다. 2021 에디션부터 이 경고가 에러(E0782)가 되었습니다([Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 2.4절).
- 예제는 에러 메시지를 버리므로, 메시지를 보고 싶으면 그 조각을 파일로 저장해 `rustc --edition <연도>`로 직접 컴파일합니다.

</details>

### 연습 4. 에디션과 `rust-version` 구분하기

`jobs_2015/Cargo.toml`의 `rust-version`을 `"1.101"`로 바꾸고 `cargo build`를 실행해 보세요. 에러 메시지를 읽고, 다음 두 질문에 답하세요. (1) 2015 에디션 크레이트가 rustc 1.100.0 이상을 요구하는 것은 모순인가? (2) `edition = "2024"`인 크레이트가 `rust-version = "1.80"`이라고 적는 것은 말이 되는가?

<details>
<summary>힌트</summary>

- 메시지는 `` error: rustc 1.100.0 is not supported by the following package: `` 다음 줄에 `jobs_2015@0.1.0 requires rustc 1.101`이었습니다. 확인한 뒤 `"1.100"`으로 되돌립니다.
- (1) 모순이 아닙니다. 에디션은 소스를 읽는 규칙이고, `bool::toggle` 같은 표준 라이브러리 API는 에디션과 상관없이 컴파일러 버전을 따라 들어옵니다(2.4절, 2.5절).
- (2) 말이 되지 않습니다. 2024 에디션은 1.85.0에 안정화되었으므로, 그보다 낮은 컴파일러는 이 크레이트의 에디션 자체를 모릅니다(2.5절). Cargo도 이것을 검사해서, 매니페스트를 읽는 단계에서 `rust-version 1.80 is incompatible with the version (1.85.0) required by the specified edition (2024)`로 거절합니다. 새 패키지를 하나 만들어 직접 확인해 보세요.

</details>

## 6. 참고 자료

- The Rust Edition Guide (기준 문서)
  - [Introduction](https://doc.rust-lang.org/edition-guide/introduction.html), [1. What are editions?](https://doc.rust-lang.org/edition-guide/editions/index.html): 하위 호환성 약속과 에디션이 필요한 이유(`let async = 1;`의 예), "에디션은 생태계를 나누지 않는다"는 규칙, 변화가 "skin deep"인 이유, 자동 마이그레이션.
  - [1.1 Creating a new project](https://doc.rust-lang.org/edition-guide/editions/creating-a-new-project.html): `cargo new`의 기본 에디션, `--edition`, 잘못된 연도에 대한 에러.
  - 에디션별 개요: [2. Rust 2015](https://doc.rust-lang.org/edition-guide/rust-2015/index.html)(에디션을 적지 않으면 2015인 이유), [3. Rust 2018](https://doc.rust-lang.org/edition-guide/rust-2018/index.html), [4. Rust 2021](https://doc.rust-lang.org/edition-guide/rust-2021/index.html), [5. Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html). 각 장의 Info 표에 그 에디션이 나온 버전이 있습니다.
  - [3.3 New keywords](https://doc.rust-lang.org/edition-guide/rust-2018/new-keywords.html), [5.1.13 `gen` keyword](https://doc.rust-lang.org/edition-guide/rust-2024/gen-keyword.html), [5.1.3 let chains](https://doc.rust-lang.org/edition-guide/rust-2024/let-chains.html), [5.1.10 Never type fallback change](https://doc.rust-lang.org/edition-guide/rust-2024/never-type-fallback.html): 이 챕터의 에러 예제와 2.4절의 예외에 쓴 절입니다.
- The Rust Programming Language: [Appendix E: Editions](https://doc.rust-lang.org/book/appendix-05-editions.html)(6주 릴리스와 3년 주기 에디션, 모든 컴파일러가 이전 에디션을 모두 지원하고 함께 링크한다는 설명), [Appendix A: Keywords](https://doc.rust-lang.org/book/appendix-01-keywords.html)의 Raw Identifiers 절(2015 에디션 라이브러리의 `try` 함수를 `r#try`로 부르는 예).
- 에러 설명: `rustc --explain E0432`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다. 4.1·4.2·4.4절의 에러처럼 코드가 없는 에러는 메시지로 찾습니다.
