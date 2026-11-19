# ch2. 2015 에디션과 2018 에디션: 옛 코드 읽기

Part 0과 Part 1에서 쓴 코드는 모두 2024 에디션이었습니다. 그런데 오래된 블로그 글, 몇 해 동안 고쳐지지 않은 크레이트의 소스, 옛 Stack Overflow 답변을 열면 `extern crate`, `#[macro_use]`, `try!`, `Box<Trait>`, `mod.rs`처럼 지금은 쓰지 않는 모양이 나옵니다. 이 챕터는 Rust 1.0부터 이어진 기준선인 **2015 에디션**과, 처음으로 에디션을 바꾼 **2018 에디션**의 변경을 다룹니다. 목표는 옛 코드를 만났을 때 "그 모양이 무엇이었고, 지금은 어떻게 쓰고, 왜 바뀌었는지"를 읽어 내는 것입니다.

[Part 2 ch1](../ch1-editions-overview/README.md)에서 본 것처럼 에디션은 크레이트마다 고르는 문법의 판이고, 에디션이 다른 크레이트끼리도 함께 링크됩니다. 이 챕터의 실습 프로젝트(`projects/scoreboard`)도 2015 에디션 라이브러리 하나를 2015판 프로그램과 2018판 프로그램이 함께 씁니다.

- 패키지: `p2-ch2-rust-2015-2018`
- 기준 문서: [Edition Guide 2절 Rust 2015](https://doc.rust-lang.org/edition-guide/rust-2015/index.html), [3절 Rust 2018](https://doc.rust-lang.org/edition-guide/rust-2018/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64). "2015에서는 통과, 2018에서는 에러" 같은 설명은 모두 이 툴체인에서 `rustc --edition 2015`, `2018`, `2021`, `2024`로 직접 컴파일해 확인한 결과입니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 2015 에디션이 왜 "기본 에디션"인지 설명하고, 2018 에디션에서 바뀐 다섯 가지(경로와 모듈 시스템, 익명 트레이트 매개변수, 새 키워드, 원시 포인터의 메서드 호출, Cargo)를 말할 수 있다. 2018 무렵에 생긴 기능 중 2015 에디션에서도 쓸 수 있는 것과 없는 것을 구분할 수 있다.
2. 2015 시절 코드의 `extern crate`, `#[macro_use]`, 크레이트 루트에서 출발하는 `use` 경로, `::`로 시작하는 경로, `mod.rs`를 알아보고 같은 뜻의 2018 이후 코드로 옮겨 쓸 수 있다.
3. `try!`, 익명 트레이트 매개변수, `dyn` 없는 트레이트 객체, `...` 범위 패턴을 알아보고 지금 문법(`?`, 이름 있는 매개변수, `dyn`, `..=`)으로 고쳐 쓸 수 있다. 각 모양이 2015/2018/2021/2024에서 통과·경고·에러 중 무엇인지 2.11절의 표에서 찾을 수 있다.
4. 2018 에디션에서 키워드가 된 `async`, `await`, `dyn`과 예약 키워드가 된 `try` 때문에 깨지는 코드를 알아보고, raw 식별자(`r#`)로 옛 에디션 크레이트의 항목을 부를 수 있다. raw 식별자로도 쓸 수 없는 이름과, 출력·매크로에서 raw 식별자가 어떻게 보이는지 설명할 수 있다.

## 2. 핵심 개념

### 2.1 이 챕터의 지도

| 절 | 주제 | 확인하는 곳 |
| --- | --- | --- |
| 2.2 | 2015 에디션과 2018 에디션 | — |
| 2.3 | `extern crate`와 `#[macro_use]` | `projects/scoreboard`, 예제 02 |
| 2.4 | `use` 경로와 `::` 경로 | 예제 02, `projects/scoreboard`, 4.4절 |
| 2.5 | `mod.rs` 없이 `foo.rs` + `foo/` | `projects/scoreboard` |
| 2.6 | 익명 트레이트 매개변수 | 예제 03, 4.2·4.5절 |
| 2.7 | 새 키워드 `dyn`, `async`, `await`, `try` | 예제 03, 4.1·4.3절 |
| 2.8 | raw 식별자 `r#` | 예제 01, `projects/scoreboard`, 4.6절 |
| 2.9 | 추론 변수를 가리키는 원시 포인터의 메서드 호출 | 예제 03 |
| 2.10 | Cargo의 변경 | — |
| 2.11 | 한눈에 보기: 에디션별 결과 표 | — |

2018 이후의 모듈 시스템 자체(모듈 트리, 경로, `pub`, `use`, 파일 나누기)는 [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md)에서 이미 배웠습니다. 이 챕터는 그것을 다시 설명하지 않고 **2015와 무엇이 다른지**에 집중합니다.

### 2.2 2015 에디션과 2018 에디션

**2015 에디션: 기준선.** Rust 1.0은 2015년 5월에 나왔습니다. 그 전의 Rust는 거의 날마다 바뀌어서 큰 프로그램을 쓰기도, 배우기도 어려웠습니다. 1.0과 함께 하위 호환성(backward compatibility)을 약속한 것이 2015 에디션의 주제인 **안정성(stability)** 입니다. 에디션이라는 제도는 2017년 말에야 구상되었으므로 "Rust 2015"는 나중에 붙인 이름이고, 지금도 **에디션을 적지 않았을 때 쓰는 기본 에디션**입니다. Edition Guide의 표현대로 2015 에디션에서 떠날 일은 있어도 2015 에디션으로 옮겨 올 일은 없습니다.

기본 에디션이라는 것은 두 곳에서 드러납니다. `rustc`는 `--edition`을 주지 않으면 2015로 컴파일합니다([Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md)의 "자주 하는 실수" 6번). `Cargo.toml`에 `edition`을 적지 않은 패키지도 2015 에디션이 되며, Cargo가 경고합니다.

```text
warning: Cargo.toml: `package.edition` is unspecified, defaulting to `2015` while the latest is `2024`
```

그래서 `edition` 줄이 없는 오래된 `Cargo.toml`을 만나면 그 크레이트는 2015 에디션입니다.

**2018 에디션: 첫 번째 새 에디션.** 에디션 제도는 2018 에디션과 함께 만들어졌고(RFC 2052), Rust 1.31.0에서 나왔습니다. 주제는 **생산성**입니다. 이 무렵 나온 기능 대부분은 기존 코드를 깨지 않았기 때문에 에디션과 상관없이 **모든 에디션에 들어갔고**, 기존 코드를 깨는 일부(대표적으로 모듈 시스템 변경)만 2018 에디션에 묶였습니다([ch1](../ch1-editions-overview/README.md) 2.4절의 "에디션이 바꾸는 것과 바꾸지 않는 것"). 이 구분은 옛 코드를 읽을 때 중요합니다. 지금 툴체인으로 2015 에디션 크레이트를 컴파일해 보면 다음과 같습니다.

| 2018 무렵의 새 기능 | 2015 에디션에서 (1.100.0) |
| --- | --- |
| `dyn Trait`, `..=` 범위 패턴, `?` 연산자 | 됨 |
| `crate::`로 시작하는 경로, raw 식별자 `r#` | 됨 |
| `foo.rs`와 `foo/bar.rs`로 모듈 나누기(2.5절) | 됨 |
| `use 크레이트::매크로;`로 매크로 가져오기 | `extern crate`가 있으면 됨 |
| `extern crate` 없이 `use 크레이트::…` | 안 됨 (E0432, 2.3절) |
| 현재 모듈에서 출발하는 `use` 경로 | 안 됨 (E0432, [ch1](../ch1-editions-overview/README.md)의 `compile_fail/e0432_use_relative_path_2015.rs`) |
| 본문이 있는 트레이트 메서드의 매개변수 패턴 | 안 됨 (E0642, 4.5절) |
| `async fn`, `async` 블록, `.await` | 안 됨 (`async fn`은 E0670, `.await`는 필드 접근으로 읽혀 E0609) |

옛 코드에서 보이는 낡은 모양에는 그래서 두 종류가 있습니다. 하나는 **에디션 때문에 그렇게 쓸 수밖에 없던 것**(`extern crate`, 크레이트 루트 기준 `use`)이고, 다른 하나는 **그때는 아직 없던 기능이라 그렇게 쓴 것**(`?` 대신 `try!`, `dyn` 없는 `Box<Trait>`)입니다. 앞의 것은 에디션을 올려야 바뀌고, 뒤의 것은 2015 에디션 안에서도 지금 문법으로 고칠 수 있습니다.

Edition Guide 3절은 2018 에디션의 변경을 다섯 항목으로 정리합니다. 2.3~2.5절이 3.1(경로와 모듈 시스템), 2.6절이 3.2(익명 트레이트 매개변수), 2.7·2.8절이 3.3(새 키워드), 2.9절이 3.4(원시 포인터), 2.10절이 3.5(Cargo)입니다.

### 2.3 `extern crate`와 `#[macro_use]`가 사라짐

2015 에디션 크레이트는 의존하는 크레이트를 크레이트 루트에서 **`extern crate`로 선언**했습니다. 실습 프로젝트의 2015판은 이렇게 시작합니다.

```rust
// projects/scoreboard/app2015/src/main.rs (2015 에디션)
#[macro_use]
extern crate scorekit;

mod record;
mod report;
```

`extern crate scorekit;`은 크레이트 루트에 `scorekit`이라는 이름을 만듭니다. 2015 에디션에서는 `use` 경로가 언제나 크레이트 루트에서 출발하므로(2.4절), 이 이름 덕분에 어느 모듈에서나 `use scorekit::ScoreError;`가 통합니다. `#[macro_use]`는 그 크레이트가 `#[macro_export]`로 내보낸 매크로(여기서는 `bar!`)를 **이 크레이트 전체에서** 쓸 수 있게 합니다.

2018 에디션부터는 두 줄 모두 필요 없습니다. Cargo는 `Cargo.toml`의 `[dependencies]`에 적은 크레이트를 `rustc`에 `--extern scorekit=…`으로 넘기고, 이렇게 받은 크레이트는 **외부 프렐류드(extern prelude)** 에 들어갑니다([Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.2절). 2018부터 외부 프렐류드의 이름은 모든 모듈에서, `use` 안에서도 밖에서도 보입니다. Edition Guide의 말대로 "의존성을 `Cargo.toml`에 추가하면 끝이고, 두 번째 단계는 없습니다". 매크로도 다른 항목처럼 경로로 가져옵니다.

```rust
// projects/scoreboard/app2018/src/report.rs (2018 에디션)
use scorekit::bar; // #[macro_use] extern crate scorekit; 대신
```

`#[macro_use]`는 크레이트 전체에 매크로를 풀어 놓지만 `use`는 그 모듈에만 이름을 들여옵니다. 그래서 2018판은 `bar!`를 실제로 쓰는 `report.rs`에 `use`를 둡니다.

**2015에서 `extern crate`를 지우면.** 2015판 `main.rs`에서 두 줄을 지우고 빌드하면 이렇게 됩니다(일부 줄 생략).

```text
error[E0432]: unresolved import `scorekit`
 --> app2015/src/record.rs:5:5
  |
5 | use scorekit::ScoreError;
  |     ^^^^^^^^ use of unresolved module or unlinked crate `scorekit`
  |
  = help: if you're trying to use a dependency named `scorekit`, upgrade your edition to be able to reference it with a `use` declaration
help: on Rust 2015, `extern crate` is required to specify a dependency on an external crate

error: cannot find macro `bar` in this scope
```

같은 파일의 `scorekit::try(..)`라는 식 안의 경로는 에러가 나지 않았다는 점을 보세요. 지금 툴체인에서는 2015 에디션에도 외부 프렐류드가 있어서 식 안의 경로는 `extern crate` 없이 통합니다. 막히는 것은 `use` 선언과 매크로입니다. The Reference도 "2015 에디션에서는 `use` 선언에서 외부 프렐류드의 크레이트를 가리킬 수 없으므로 `extern crate`를 쓰는 것이 관례"라고 설명합니다. 그래서 2015 에디션 코드에는 거의 언제나 `extern crate`가 있습니다.

**크레이트에 다른 이름 붙이기.** 2015의 `extern crate scorekit as sk;`는 2018부터 `use scorekit as sk;`로 씁니다. 다만 둘은 범위가 다릅니다. 크레이트 루트의 `extern crate … as sk`는 크레이트 전체에서 `sk`를 쓰게 하지만, `use … as sk`는 그 모듈에만 별칭을 만듭니다. 하위 모듈에서도 쓰려면 그 모듈에서 `use crate::sk;`처럼 다시 가져옵니다. 예제 02는 `use std as stdlib;`로 이 모양을 보여 줍니다.

**`extern crate`가 아직 필요한 곳.** Rust와 함께 배포되는 sysroot 크레이트 중 Cargo가 `--extern`으로 넘기지 않는 것(`alloc`, nightly의 `test` 등)은 2024 에디션에서도 `extern crate alloc;`이 필요합니다(예제 02의 7번 구역). 또 `extern crate`는 2018 이후에도 **에러가 아닙니다**. 기본 설정에서는 경고도 없고, 관용구 린트 그룹 `rust_2018_idioms`를 켜면 `unused_extern_crates`가 "unused extern crate"라고 알려 줍니다. 이 린트 그룹과 자동 수정은 Part 2 `ch6-migration-practice`에서 다룹니다.

### 2.4 `use` 경로와 `::` 경로: 크레이트 루트 기준에서 현재 모듈 기준으로

2015 에디션의 경로 규칙은 **어디에 쓰느냐에 따라** 달랐습니다.

- `use` 선언 안의 경로는 언제나 **크레이트 루트**에서 출발합니다.
- 코드(식, 타입) 안의 경로는 **현재 모듈**에서 출발합니다.
- `::`로 시작하는 경로는 **크레이트 루트**에서 출발합니다.

크레이트 루트(`main.rs`, `lib.rs`)에서는 세 규칙의 결과가 같아서 차이가 보이지 않습니다. 문제는 하위 모듈입니다. Edition Guide는 이를 "하위 모듈이 있을 만큼 큰 프로젝트를 만들기 전까지는 모든 것이 쉬워 보인다"고 표현합니다. 코드를 크레이트 루트에서 하위 모듈로 옮기면 식 안의 경로는 깨지고 `use`는 멀쩡하거나, 그 반대가 되는 식이었습니다.

2018 에디션부터는 **`use` 안의 경로와 코드 안의 경로가 같은 규칙**을 따르고, 어느 모듈에서든 같은 뜻입니다. 경로는 다음 중 하나로 시작합니다. 각각은 [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.4절에서 배운 그대로입니다.

| 시작 | 뜻 |
| --- | --- |
| 현재 모듈에 있는 이름 (자식 모듈, `use`로 들여온 이름) | 상대 경로 |
| `self::`, `super::` | 현재 모듈, 부모 모듈 |
| `crate::` | 크레이트 루트 |
| 외부 크레이트 이름 (`std::`, `scorekit::`) | 외부 프렐류드의 크레이트 |
| `::크레이트이름` | 외부 크레이트만 (지역 이름과 겹칠 때 쓰는 모양) |

실습 프로젝트의 `report/table.rs`(모듈 `report::table`)에서 같은 일을 하는 경로를 비교하면 다음과 같습니다.

| 2015판 | 2018판 | 무엇이 달라졌나 |
| --- | --- | --- |
| `use record::Record;` | `use crate::record::Record;` | 2018에서 `use record::…`는 현재 모듈(`report::table`)의 `record`를 찾으므로 E0432 |
| `::report::grade(average)` | `crate::report::grade(average)` | 2018에서 `::`는 외부 크레이트 전용이라 `report`라는 크레이트를 찾다가 E0433 |
| `pub use self::table::render;` (`report` 안) | `pub use table::render;` | 2015에서는 자식 모듈도 `self::`를 붙여야 `use`에서 찾음 |
| `use super::Style;` | `use super::Style;` | 그대로 |

**글자는 같은데 뜻이 다른 줄.** 두 판의 `record.rs`에는 똑같이 `use scorekit::ScoreError;`가 있습니다. 2015판에서 이 `scorekit`은 `extern crate`가 크레이트 루트에 만든 이름이고, 2018판에서는 외부 프렐류드의 크레이트입니다. 크레이트 루트에 둔 `use`나 `std::…`로 시작하는 `use`는 이렇게 두 에디션에서 글자가 같은 경우가 많습니다. `std`는 2015에서 컴파일러가 크레이트 루트에 자동으로 넣어 주었기 때문입니다.

**같은 이름이 겹칠 때.** 2018 이후 지역 모듈이 외부 크레이트와 이름이 같으면, 그 이름으로 시작하는 경로는 지역 모듈을 가리킵니다. 외부 크레이트를 콕 집으려면 `::`를 붙입니다. 예제 02의 6번 구역은 `mod core`를 만들고 `core::answer()`와 `::core::cmp::min(3, 8)`을 함께 씁니다. 2015 에디션이었다면 `::core`가 크레이트 루트의 `mod core`를 가리키므로 `::core::cmp::min`은 E0433으로 실패합니다.

**`pub(in 경로)`.** 공개 범위를 모듈로 지정하는 `pub(in 경로)`([Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 참고 자료의 `pub(crate)`와 같은 가족)의 경로도 2018부터는 `crate`, `self`, `super`로 시작해야 합니다. `mod shop { pub mod cart { pub(in shop) fn secret() … } }`처럼 2015 에디션에서 통하던 코드를 2018로 컴파일하면 이렇게 거절됩니다(위치 줄 생략).

```text
error: relative paths are not supported in visibilities in 2018 edition or later
  |
3 |         pub(in shop) fn secret() -> u8 {
  |                ^^^^ help: try: `crate::shop`
```

**왜 바꿨나.** 규칙이 하나이면 코드를 다른 모듈로 옮겨도 경로가 그대로 통합니다. Edition Guide가 보여 주는 예처럼 2018에서는 크레이트 루트에 있던 코드 덩어리(`use`, 모듈, 함수)를 `mod submodule { … }` 안에 통째로 넣어도 한 글자도 고치지 않고 컴파일됩니다. 2015에서는 그 안의 `use`를 모두 `self::`로 고쳐야 했습니다.

### 2.5 `mod.rs` 없이 `foo.rs` + `foo/`

하위 모듈을 가진 모듈의 파일 배치도 2015 시절 코드와 지금 코드가 다르게 보이는 곳입니다. 실습 프로젝트의 두 판은 `report` 모듈(하위 모듈 `table`을 가짐)을 이렇게 둡니다.

```text
app2015/src/                   app2018/src/
├── main.rs                    ├── main.rs
├── record.rs                  ├── record.rs
└── report/                    ├── report.rs        ← report 모듈의 본문
    ├── mod.rs  ← report의 본문 └── report/
    └── table.rs                   └── table.rs
```

Edition Guide는 2015 에디션에서는 하위 모듈이 있는 모듈의 본문을 반드시 `foo/mod.rs`에 두어야 했고, 2018부터 그 제한이 풀려 `foo.rs`와 `foo/` 디렉터리를 함께 둘 수 있게 되었다고 설명합니다. 편집기에서 `mod.rs`라는 이름의 탭이 여러 개 열려 구분하기 어려운 문제를 없애려는 변경입니다.

그런데 **지금 툴체인에서는 2015 에디션에서도 `foo.rs` + `foo/` 방식이 동작합니다.** app2015의 `report/mod.rs`를 `report.rs`로 옮겨도 그대로 빌드되고, 반대로 `mod.rs` 방식은 2024 에디션에서도 동작합니다. 이 기능은 2.2절의 표처럼 모든 에디션에 들어간 쪽이고, Edition Guide의 설명은 2018 에디션이 나올 무렵의 사정입니다. 그래서 `mod.rs`는 에디션 차이라기보다 **그 시절의 관례**로 읽으면 됩니다. 두 방식의 규칙과 둘을 동시에 두었을 때의 E0761은 [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.11절에 있습니다. 프로젝트 안에서 한 방식으로 통일하고 싶다면 Clippy의 restriction 린트 `clippy::mod_module_files`(`mod.rs` 금지)나 `clippy::self_named_module_files`(`foo.rs` 금지)를 켭니다. 둘 다 기본으로는 꺼져 있습니다.

### 2.6 익명 트레이트 매개변수

2015 에디션에서는 트레이트 메서드에 매개변수 **이름 없이 타입만** 적을 수 있었습니다.

```rust
// 2015 에디션 (projects/scoreboard/app2015/src/report/mod.rs)
pub trait Style {
    fn line(&self, &str, u32) -> String;
}
```

2018 에디션부터는 보통 함수처럼 모든 매개변수에 이름(패턴)이 있어야 합니다(RFC 1685). 쓰지 않을 매개변수라면 `_`를 씁니다.

```rust
// 2018 에디션 이후
pub trait Style {
    fn line(&self, name: &str, score: u32) -> String;
}
```

2015 에디션에서도 이 문법은 경고 대상입니다. 린트 `anonymous_parameters`는 에디션 호환성 린트 그룹 `rust_2018_compatibility`에 속하고 기본으로 켜져 있습니다. 아래는 app2015에서 `#[allow(anonymous_parameters)]` 줄을 지우고 빌드한 결과의 일부입니다.

```text
warning: anonymous parameters are deprecated and will be removed in the next edition
   |
13 |     fn line(&self, &str, u32) -> String;
   |                    ^^^^ help: try naming the parameter or explicitly ignoring it: `_: &str`
   |
   = warning: this is accepted in the current edition (Rust 2015) but is a hard error in Rust 2018!
```

**왜 바꿨나.** 매개변수 이름은 그 값이 무엇인지 알려 주는 문서입니다. `fn line(&self, &str, u32)`만 보고는 `u32`가 점수인지 줄 번호인지 알 수 없습니다. 또 이름이 빠질 수 있으면 파서는 `f64` 하나만 보고 그것이 이름인지 타입인지 정할 수 없습니다. 2018에서 같은 코드를 컴파일하면 컴파일러가 "이름이라면 타입을 붙이고, 타입이라면 `_:`를 붙이라"는 두 가지 제안을 함께 내는 것이 그 모호함을 보여 줍니다(4.2절).

이름이 반드시 있게 되면서 할 수 있게 된 일도 있습니다. Edition Guide의 요약대로 2018부터 **본문이 있는** 트레이트 메서드는 매개변수 자리에 반박 불가능한 패턴을 쓸 수 있습니다. 예제 03의 `fn add(&self, (a, b): (i32, i32)) -> i32 { a + b }`가 그 모양입니다. 같은 모양을 2015로 컴파일하면 E0642로 거절되는데, 메시지가 `patterns aren't allowed in methods without bodies`라서 본문이 있는데도 "본문이 없는 메서드"라고 말합니다(4.5절). 본문이 없는 선언에서는 지금도 모든 에디션에서 E0642입니다.

### 2.7 새 키워드: `dyn`, `async`, `await`, 예약된 `try`

Rust의 키워드에는 세 종류가 있습니다(The Reference). **엄격한 키워드**(strict)는 정해진 자리 밖에서 이름으로 쓸 수 없고, **예약 키워드**(reserved)는 아직 쓰임새가 없지만 앞으로를 위해 같은 제한을 받으며, **약한 키워드**(weak)는 특정 문맥에서만 키워드입니다. 2018 에디션은 다음을 바꿨습니다.

| 단어 | 2015 에디션 | 2018 에디션 이후 | 쓰임새 |
| --- | --- | --- | --- |
| `dyn` | 약한 키워드 (타입 자리에서만) | 엄격한 키워드 | 트레이트 객체 `dyn Trait` ([Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)) |
| `async`, `await` | 키워드 아님 | 엄격한 키워드 | 비동기, Rust 1.39.0에서 안정화 ([Part 0 ch16](../../part0-getting-started/ch16-async/README.md)) |
| `try` | 키워드 아님 | 예약 키워드 | `try` 블록을 위해 예약. 1.100.0에서도 불안정(`try { … }`는 E0658 `` `try` expression is experimental ``) |

키워드가 새로 생기면 그 단어를 이름으로 쓰던 코드가 깨집니다. 2015 에디션에서 문제없던 `fn try(..)`, `let dyn = 3;`, `let async = ..;`가 2018부터는 파싱 단계에서 거절됩니다(`try`·`dyn`은 4.1절, `async`는 [ch1](../ch1-editions-overview/README.md) 4.1절). 이런 이름을 쓰던 크레이트를 부르는 방법이 2.8절의 raw 식별자입니다. 반대로 2015 에디션에서는 `async fn`을 쓸 수 없습니다(E0670, [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md)의 "자주 하는 실수" 6번).

2015 에디션 코드에 `-W rust-2018-compatibility`(에디션 호환성 린트 그룹)를 주면 린트 `keyword_idents_2018`이 `` `async` is a keyword in the 2018 edition ``이라는 경고와 함께 `` help: you can use a raw identifier to stay compatible: `r#async` ``를 미리 알려 줍니다. 이 린트의 기본 수준은 allow라서 그냥은 아무 경고도 없습니다.

**`try!` 매크로.** 표준 라이브러리의 `try!(식)`은 1.0부터 있던 매크로로, `Ok`면 값을 꺼내고 `Err`면 `From`으로 변환해 곧바로 돌려줍니다. `?` 연산자가 이 매크로를 대신하려고 들어왔고, `try!`는 1.39.0부터 deprecated입니다(`std::try!` 문서). 2015 시절 코드에는 `try!`가 아주 흔합니다.

| 모양 | 2015 | 2018 이후 |
| --- | --- | --- |
| `let n = try!(text.parse::<i32>());` | 경고 (deprecated) | 에러 `` use of deprecated `try` macro `` |
| `let n = r#try!(text.parse::<i32>());` | 경고 (deprecated) | 경고 (deprecated) |
| `let n = text.parse::<i32>()?;` | 통과 | 통과 |

예제 03의 1번 구역이 `r#try!`와 `?`의 결과가 같다는 것을 보여 줍니다. `r#try!`는 옛 코드를 급히 컴파일만 시킬 때 쓰는 모양이고, 고칠 때는 `?`로 바꿉니다. `?`의 쓰임은 [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)에 있습니다.

**`dyn`을 붙이는 이유.** 2015 시절 코드는 트레이트 객체 타입을 `Box<Style>`, `&Style`처럼 트레이트 이름만으로 적었습니다. `dyn Trait` 문법은 2015 에디션에서도 쓸 수 있으므로(`dyn`이 약한 키워드인 덕분), 이 모양은 "그때는 아직 없던 문법"에 속합니다. Edition Guide는 트레이트 이름만 쓰는 문법이 잘못된 결정이었다고 설명합니다. 트레이트 이름과 타입 이름이 같은 자리에 오니 숙련자에게도 모호하고, 제네릭이나 `impl Trait`보다 자주 쓰이지도 않으며 때로는 더 느리고 아예 쓸 수 없을 때도 많은 기능을 가장 짧은 문법으로 우대했다는 것입니다. `dyn`을 붙이면 동적 디스패치라는 것이 눈에 보이고 `impl Trait`와 짝이 맞습니다(예제 03의 3번 구역).

**2021 에디션에서 에러가 되는 두 모양.** `dyn` 없는 트레이트 객체와 `a...b` 범위 패턴은 2015 시절 코드에서 아주 흔하지만, 2015와 2018에서는 **경고**이고 2021부터 **에러**입니다. 끝을 포함하는 범위는 식에서 처음부터 `..=`였는데 패턴에서만 `...`를 쓰던 것을 `..=`로 맞춘 것입니다([Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md)의 범위 패턴). 두 경고(문구는 3절의 `projects/scoreboard` 표)에는 `this is accepted in the current edition (Rust 2015) but is a hard error in Rust 2021!`이라는 줄이 붙습니다. 2021 에디션의 에러(E0782, E0783)는 Part 2 `ch3-rust-2021`에서 다룹니다.

### 2.8 raw 식별자 `r#`

**raw 식별자(raw identifier)** 는 이름 앞에 `r#`를 붙여 "이것은 키워드가 아니라 이름이다"라고 표시하는 문법입니다. 엄격한 키워드와 예약 키워드를 거의 모두 이름으로 쓸 수 있게 해 줍니다.

```rust
fn r#try(text: &str) -> Option<u32> { text.trim().parse().ok() }
struct Job { r#type: &'static str, r#async: bool }
let job = Job { r#type: "batch", r#async: true };
println!("{}", job.r#type); // job.type이라고 쓰면 expected identifier, found keyword `type`
```

**왜 필요한가.** 2.7절처럼 새 에디션이 키워드를 늘리면, 옛 에디션 크레이트가 공개한 `try`라는 함수를 새 에디션 크레이트에서 `scorekit::try(..)`라고 부를 수 없습니다. 파서가 `try`를 키워드로 읽기 때문입니다. 실습 프로젝트의 2018판은 2015 에디션 라이브러리 `scorekit`의 `try` 함수를 이렇게 부릅니다.

```rust
// projects/scoreboard/app2018/src/record.rs (2018 에디션)
let score = scorekit::r#try(parts.next().unwrap_or(""))?;
```

에디션이 다른 크레이트를 함께 링크할 수 있다는 약속은 raw 식별자가 있어서 지켜집니다. [ch1](../ch1-editions-overview/README.md) 2.7절의 `projects/edition_mix`가 2015 크레이트의 `async`·`await` 함수를 2024 에디션에서 `r#async`·`r#await`로 부른 것도 같은 원리이고, The Book의 부록 A도 같은 예(`try` 함수를 가진 2015 라이브러리)를 듭니다. 키워드와 같은 이름은 에디션과 상관없이 쓸모가 있기도 합니다. 설정 파일이나 외부 데이터의 `type`, `match` 같은 항목 이름을 필드 이름에 그대로 옮길 때입니다.

**`r#`는 이름의 일부가 아닙니다.** The Reference는 "`r#` 접두사는 실제 식별자에 포함되지 않는다"고 정합니다. `let r#count = 3;`으로 만든 변수를 `count`로 읽을 수 있고(예제 01의 2번 구역), 키워드가 아닌 이름에 `r#`를 붙여도 아무 효과가 없습니다. 그래서 어떤 단어에 `r#`가 필요한지는 **그 코드를 컴파일하는 크레이트의 에디션**이 정합니다. `scorekit`을 2018 에디션으로 올리면 정의를 `pub fn r#try(..)`로 고쳐야 하지만, 2015판 프로그램은 여전히 `scorekit::try(..)`로 부릅니다(5절 연습 2). 2024 에디션 코드에서는 2024에 예약된 `gen`에도 `r#`가 필요합니다(Part 2 `ch4-rust-2024-language`).

raw 식별자 문법 자체는 2015 에디션에서도 쓸 수 있습니다. `r#async`, `r#try`는 2015 에디션 코드에서도 컴파일됩니다.

**쓸 수 없는 이름.** `crate`, `self`, `super`, `Self`와 `_`는 `r#`를 붙여도 이름이 될 수 없습니다. 이들은 경로의 시작점이라서 이름으로 허용하면 경로가 모호해지고, 2015 에디션에서도 이미 이름으로 쓸 수 없었으므로 raw 식별자가 필요한 상황 자체가 생기지 않습니다. `r#crate`는 모든 에디션에서 `` `crate` cannot be a raw identifier `` 에러입니다(4.6절). 반면 `r#static`, `r#extern`, `r#match`처럼 다른 키워드는 모두 됩니다.

**출력과 매크로에서 보이는 모습.** `r#`가 이름의 일부가 아니므로, 이름을 문자열로 바꾸는 기능은 대부분 `r#`를 뺍니다. 예외는 토큰을 있는 그대로 다루는 매크로입니다. 예제 01의 3번 구역에서 확인한 결과입니다.

| 기능 | 결과 | `r#` |
| --- | --- | --- |
| `#[derive(Debug)]` 출력 | `Job { type: "batch", async: true }` | 뺌 |
| `std::any::type_name` | `…::legacy::dyn::Marker` | 뺌 |
| 서식 문자열의 이름 `{type}` | `let r#type = …;` 뒤 `println!("{type}")`가 동작 | 쓰면 에러 |
| `stringify!(legacy::r#try(x))` | `legacy::r#try(x)` | 남김 |
| `module_path!()` | `01_raw_identifiers::legacy::r#dyn` | 남김 |

서식 문자열 안에서는 이름 자리의 키워드를 `r#` 없이 씁니다. `let r#type = "batch";` 다음에 `{r#type}`이라고 쓰면 컴파일러가 거절합니다(위치 줄 생략).

```text
error: invalid format string: raw identifiers are not supported
  |
3 |     println!("{r#type}");
  |                --^^^^
  |                |
  |                raw identifier used here in format string
  |                help: remove the `r#`
  |
  = note: identifiers in format strings can be keywords and don't need to be prefixed with `r#`
```

**매크로 이름에도 쓸 수 있습니다.** 2018 이후 `try!`를 부르는 유일한 방법이 `r#try!`인 것처럼, `r#`는 매크로 이름에도 붙습니다(2.7절).

### 2.9 추론 변수를 가리키는 원시 포인터의 메서드 호출

[Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.2절의 원시 포인터를 `as *mut _`처럼 **가리키는 타입을 `_`로 남긴 채** 만들고 곧바로 메서드를 부르는 코드가 있습니다.

```rust
let x = &mut 0;
let p = x as *mut _;
println!("{}", p.is_null());
```

메서드를 찾으려면 `p`의 타입을 알아야 하는데, 가리키는 타입이 아직 추론 변수(`_`)입니다. 2015 에디션에서는 린트 `tyvar_behind_raw_pointer`(에디션 호환성 린트 그룹 `rust_2018_compatibility`, 기본으로 켜짐)가 경고(`type annotations needed`)만 하고 넘어갔고, 2018 에디션부터는 에러입니다. Edition Guide는 "린트가 하드 에러가 되었다"고만 적는데, 1.100.0에서 나오는 에러 코드는 E0282입니다(위치 줄 생략).

```text
error[E0282]: type annotations needed for `*mut _`
  |
3 |     let p = x as *mut _;
  |         ^
4 |     println!("{}", p.is_null());
  |                      ------- cannot call a method on a raw pointer with an unknown pointee type
  |
help: consider giving `p` an explicit type, where the placeholder `_` is specified
  |
3 |     let p: *mut _ = x as *mut _;
  |          ++++++++
```

`help:`의 제안대로 `let p: *mut _ = …`라고 적어도 `_`가 그대로 남아서 같은 E0282가 다시 납니다. `let p: *mut i32 = x;`처럼 가리키는 타입을 구체적으로 적어야 합니다(예제 03의 5번 구역). 원시 포인터를 직접 다루는 코드 자체가 드물어서 옛 코드에서도 자주 보이는 모양은 아닙니다.

### 2.10 Cargo의 변경

2018 에디션은 Cargo가 빌드 대상(target)을 찾는 방식도 바꿨습니다. 2015 에디션 패키지는 `Cargo.toml`에 `[[bin]]` 같은 대상을 하나라도 직접 적으면 `src/main.rs`, `src/bin/*.rs` 같은 다른 대상을 자동으로 찾지 않고(Cargo가 "An explicit [[bin]] section is specified in Cargo.toml which currently disables Cargo from automatically inferring other binary targets"라고 경고합니다), `path` 없이 적은 `[[bin]] name = "extra"`의 파일을 `src/extra.rs`에서도 찾아 줍니다(경고 `` path `src/extra.rs` was erroneously implicitly accepted for binary `extra` ``). 2018 에디션부터는 대상을 직접 적어도 나머지 대상을 계속 자동으로 찾고, `src/extra.rs`는 추측하지 않아 `src/bin/extra.rs`가 없으면 에러입니다. 둘 다 이 툴체인에서 확인했습니다. Edition Guide는 그 밖에 현재 디렉터리의 패키지를 설치할 때 `cargo install`만으로는 안 되고 `cargo install --path .`라고 써야 한다는 변경도 적고 있는데, 이것은 사용자 환경을 바꾸는 명령이라 이 챕터에서는 실행해 확인하지 않았습니다. 빌드 대상과 `cargo install`은 Part 3에서 자세히 다룹니다.

### 2.11 한눈에 보기: 에디션별 결과 표

모든 칸은 rustc 1.100.0에서 `rustc --edition …`으로 직접 컴파일한 결과입니다. "통과"는 경고 없이 컴파일된다는 뜻이고, "경고"는 기본 설정에서 경고가 나지만 컴파일된다는 뜻입니다. 외부 크레이트가 필요한 줄은 `--extern`으로 작은 라이브러리를 넘겨 확인했습니다.

**경로와 모듈**

| 2015 시절 모양 | 2015 | 2018 | 2021 | 2024 | 지금 모양 |
| --- | --- | --- | --- | --- | --- |
| `extern crate foo;` + `use foo::X;` | 통과 | 통과 | 통과 | 통과 | `extern crate` 줄을 지움 |
| `extern crate` 없이 `use foo::X;` | E0432 | 통과 | 통과 | 통과 | — |
| `extern crate` 없이 식 안의 `foo::f()` | 통과 | 통과 | 통과 | 통과 | — |
| `#[macro_use] extern crate foo;` | 통과 | 통과 | 통과 | 통과 | `use foo::매크로;` |
| `extern crate foo as f;` | 통과 | 통과 | 통과 | 통과 | `use foo as f;` |
| 하위 모듈에서 `use a::x;` (`a`는 루트의 모듈) | 통과 | E0432 | E0432 | E0432 | `use crate::a::x;` |
| 자식 모듈을 `use child::x;` (`self::` 없이) | E0432 | 통과 | 통과 | 통과 | — |
| `::a::x` (`a`는 루트의 모듈) | 통과 | E0433 | E0433 | E0433 | `crate::a::x` |
| `::std::x`, `crate::a::x`, `self::x`, `super::x` | 통과 | 통과 | 통과 | 통과 | — |
| `pub(in a)` (`a`는 루트의 모듈) | 통과 | 에러 | 에러 | 에러 | `pub(in crate::a)` |
| `foo/mod.rs` | 통과 | 통과 | 통과 | 통과 | `foo.rs` (권장) |
| `foo.rs` + `foo/bar.rs` | 통과 | 통과 | 통과 | 통과 | — |

**키워드와 문법**

| 2015 시절 모양 | 2015 | 2018 | 2021 | 2024 | 지금 모양 |
| --- | --- | --- | --- | --- | --- |
| `async`, `await`, `dyn`, `try`를 이름으로 | 통과 | 에러 | 에러 | 에러 | `r#async` 등 |
| `r#async`, `r#try` (raw 식별자) | 통과 | 통과 | 통과 | 통과 | — |
| `async fn` | E0670 | 통과 | 통과 | 통과 | — |
| `try!(식)` | 경고 | 에러 | 에러 | 에러 | `식?` |
| `r#try!(식)` | 경고 | 경고 | 경고 | 경고 | `식?` |
| 트레이트 메서드의 익명 매개변수 `fn f(&self, u8);` | 경고 | 에러 | 에러 | 에러 | `fn f(&self, x: u8);` |
| `Box<Trait>`, `&Trait` | 경고 | 경고 | E0782 | E0782 | `Box<dyn Trait>` |
| `Box<dyn Trait>` | 통과 | 통과 | 통과 | 통과 | — |
| 범위 패턴 `0...9` | 경고 | 경고 | E0783 | E0783 | `0..=9` |
| `let p = x as *mut _; p.is_null()` | 경고 | E0282 | E0282 | E0282 | `let p: *mut i32 = x;` |

"에러"라고만 적은 칸은 E 코드 없이 파싱 에러로 나오는 경우입니다.

### 2.12 이 챕터에서 다루지 않는 것

- 에디션이 무엇이고 에디션이 다른 크레이트가 어떻게 함께 링크되는지, 매크로 본문이 어느 에디션으로 읽히는지는 [Part 2 ch1](../ch1-editions-overview/README.md)에서 다뤘습니다.
- 2021 에디션에서 에러가 된 것(E0782, E0783)의 자세한 설명은 Part 2 `ch3-rust-2021`, 2024 에디션의 `gen` 키워드는 Part 2 `ch4-rust-2024-language`에서 다룹니다.
- 경고를 따라 코드를 자동으로 고치는 `cargo fix --edition`과 에디션 호환성 린트 그룹 `rust_2018_compatibility`, 관용구 린트 그룹 `rust_2018_idioms`(`unused_extern_crates`, `elided_lifetimes_in_paths` 등)를 쓰는 마이그레이션 절차는 Part 2 `ch6-migration-practice`에서 다룹니다.
- 매크로의 `$crate`, 매크로 위생, `#[macro_use]`의 자세한 범위 규칙은 Part 9 `ch2-macros`, Cargo의 빌드 대상과 의존성 설정은 Part 3에서 다룹니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_raw_identifiers.rs`](examples/01_raw_identifiers.rs) | 키워드와 같은 이름을 `r#`로 정의하고 부르기, `r#`는 이름의 일부가 아님, 출력과 매크로에서 보이는 모습 | 4 |
| [`02_paths_today.rs`](examples/02_paths_today.rs) | 2018 이후의 경로 시작점 전부(`crate::`, `self::`, `super::`, 크레이트 이름, `::크레이트`), `use std as stdlib`, `extern crate alloc` | 2 |
| [`03_then_and_now.rs`](examples/03_then_and_now.rs) | `try!` → `?`, 익명 매개변수 → 이름·`_`·패턴, `dyn`, `..=`, 원시 포인터의 타입 적기 | 3 |
| [`projects/scoreboard`](projects/scoreboard) | 같은 점수표 프로그램의 2015판과 2018판, 두 판이 함께 쓰는 2015 에디션 라이브러리 | 1, 2, 3, 4 |
| [`compile_fail/`](compile_fail) | 에디션에 따라 결과가 갈리는 컴파일 에러 6개(4절) | 2, 3, 4 |

`examples/`는 이 저장소의 워크스페이스 설정대로 2024 에디션으로 컴파일되므로 "지금의 모양"만 담았습니다. 2015 에디션으로 컴파일되는 코드는 `projects/scoreboard`에 있습니다. 예제 명령은 저장소 루트에서 실행합니다. 세 예제 모두 종료 코드 0으로 곧바로 끝나고, 실행할 때마다 같은 내용을 출력합니다.

### 01_raw_identifiers — raw 식별자

```sh
cargo run -p p2-ch2-rust-2015-2018 --example 01_raw_identifiers
```

```text
1. 키워드와 같은 이름을 r#로 부릅니다
    legacy::r#try("42")  = Some(42)
    legacy::r#try("4X")  = None
    job.r#type = batch, job.r#async = true
2. r#는 이름의 일부가 아닙니다
    r#count로 만들고 count로 읽기: 4
    2024 에디션에서는 r#gen
3. 출력과 매크로에서 보이는 모습
    Job { type: "batch", async: true }
    서식 문자열의 {type}: batch
    stringify!: legacy::r#try(x)
    module_path!: 01_raw_identifiers::legacy::r#dyn
    type_name: 01_raw_identifiers::legacy::dyn::Marker
```

- **1번 구역**: `mod legacy`가 2015 크레이트의 API를 흉내 냅니다. 함수 `r#try`, 필드 `r#type`·`r#async`, 모듈 `r#dyn`을 정의할 때도 부를 때도 `r#`를 붙입니다.
- **2번 구역**: `r#count`와 `count`는 같은 변수입니다. `gen`은 2024 에디션에서 예약되었으므로 이 파일에서는 `r#gen`이라고 써야 합니다.
- **3번 구역**: 2.8절의 표를 출력으로 확인합니다. `derive(Debug)`와 `type_name`은 `r#`를 빼고, `stringify!`와 `module_path!`는 남깁니다.

### 02_paths_today — 2018 이후의 경로

```sh
cargo run -p p2-ch2-rust-2015-2018 --example 02_paths_today
```

```text
1. 절대 경로 crate::
    crate::shop::name() = 상점
2. 상대 경로 super::, self::
    super::name() = 상점
    self::label() = 장바구니
3. use도 현재 모듈 기준
    use price::total; total() = 4500
4. 함수 안의 use Size::*
    Small = 작음, Large = 큼
5. 외부 크레이트 이름은 어디서나
    std::cmp::max(3, 8) = 8
6. 지역 모듈 core와 외부 크레이트 ::core
    core::answer()          = 42
    ::core::cmp::min(3, 8)  = 3
7. 별칭을 붙인 크레이트와 extern crate alloc
    stdlib::cmp::max(3, 8) = 8
    alloc::rc::Rc 강한 참조 수 = 1
```

- **3·4번 구역**: `shop::cart` 모듈 안의 `use price::total;`과 함수 안의 `use Size::*;`는 2015 에디션이었다면 `self::`를 붙여야 했던 줄입니다. 같은 모양을 2015로 컴파일했을 때의 E0432는 [ch1](../ch1-editions-overview/README.md)의 `compile_fail/e0432_use_relative_path_2015.rs`에서 볼 수 있습니다.
- **6번 구역**: 지역 모듈 `core`가 외부 크레이트 `core`를 가립니다. `::core::…`는 언제나 외부 크레이트입니다.
- **7번 구역**: `use std as stdlib;`는 2015의 `extern crate std as stdlib;`를 대신하는 모양입니다. 파일 위쪽의 `extern crate alloc;`을 지우면 `alloc::rc::Rc`에서 E0433이 납니다.

### 03_then_and_now — 2015 시절 문법과 지금 문법

```sh
cargo run -p p2-ch2-rust-2015-2018 --example 03_then_and_now
```

```text
1. try!(식) → 식?
    r#try! 로: Ok(42)
    ?      로: Ok(42)
    실패도 같음: true
2. 트레이트 메서드의 매개변수에는 이름이나 패턴
    greet  = 안녕하세요, 지영님
    volume = 보통
    add    = 5
3. 트레이트 객체에는 dyn
    안녕하세요, 민수님
    Hello, 민수
    impl Greeter로: Hello, 민수
4. 범위 패턴은 ..=
    95, 85, 40 → ABF
    식의 1..=3 → [1, 2, 3]
5. 가리키는 타입을 모르는 원시 포인터에 메서드 호출
    p.is_null() = false
```

- 각 구역 위의 주석에 2015 모양을 적어 두었습니다. 그 모양 그대로는 2024 에디션에서 컴파일되지 않습니다.
- **1번 구역**: `r#try!`를 쓴 함수에는 `#[expect(deprecated, reason = …)]`를 붙였습니다. 이 속성을 지우면 `` use of deprecated macro `r#try`: use the `?` operator instead `` 경고가 납니다.
- **2번 구역**: `volume`의 `_: u8`은 이름이 필요 없는 매개변수, `add`의 `(a, b): (i32, i32)`는 2018부터 본문이 있는 트레이트 메서드에 허용된 패턴입니다.

### projects/scoreboard — 같은 프로그램의 2015판과 2018판

`"이름 점수"` 줄들을 읽어 두 가지 스타일로 점수표를 출력하는 작은 프로그램입니다. 세 패키지로 된 워크스페이스입니다.

```text
projects/scoreboard/
├── Cargo.toml            가상 매니페스트 (members = scorekit, app2015, app2018)
├── scorekit/             라이브러리, edition = "2015". 함수 try, 매크로 bar!, 에러 타입 ScoreError
├── app2015/              바이너리, edition = "2015". scorekit에 경로 의존성
│   └── src/ main.rs, record.rs, report/mod.rs, report/table.rs
└── app2018/              바이너리, edition = "2018". 같은 scorekit에 경로 의존성
    └── src/ main.rs, record.rs, report.rs, report/table.rs
```

`scorekit`은 2015 에디션으로 쓴 "옛 크레이트"라는 설정입니다. 2015에서는 `try`가 키워드가 아니므로 `pub fn try(text: &str) -> Result<u32, ScoreError>`를 공개할 수 있습니다. `#[macro_export]`로 내보낸 `bar!` 매크로는 펼쳐진 곳에서도 함수를 찾을 수 있도록 `$crate::bar_with(..)`를 부릅니다(`$crate`는 모든 에디션에서 같은 뜻입니다).

```sh
cd part2-editions/ch2-rust-2015-2018/projects/scoreboard
cargo run -p app2015
cargo run -p app2018
```

두 명령 모두 종료 코드 0이고 출력이 한 글자도 다르지 않습니다.

```text
건너뜀: 점수가 아닙니다: "8O"
[plain]
  민수  87점 B
  지영  93점 A
  수진 100점 A
  현우  72점 C
  평균 88점 (B)
[graph]
  민수 ########
  지영 #########
  수진 ##########
  현우 #######
  평균 88점 (B)
```

첫 줄은 `철호 8O`(숫자 0 대신 영문자 O)가 `scorekit::try`에서 에러가 되어 `try!`/`?`로 전파된 결과입니다.

**두 판의 차이.** `diff -u`로 비교한 결과입니다(주석이 바뀐 줄은 일부 생략). `Cargo.toml`은 `name`과 `edition` 두 줄만 다릅니다.

```diff
--- app2015/src/main.rs
+++ app2018/src/main.rs
-// 2015: 외부 크레이트는 `extern crate`로 선언해야 `use` 경로에서 쓸 수 있습니다.
-// `#[macro_use]`를 붙이면 그 크레이트가 내보낸 매크로(bar!)를 이 크레이트 어디서나 쓸 수 있습니다.
-#[macro_use]
-extern crate scorekit;
+// 2018: Cargo.toml의 [dependencies]에 적은 크레이트는 `extern crate` 없이 어디서나 이름으로 씁니다.
+// 매크로도 다른 항목처럼 `use`로 가져옵니다(report.rs의 `use scorekit::bar;`).
```

```diff
--- app2015/src/record.rs
+++ app2018/src/record.rs
 use scorekit::ScoreError;
 ...
-#[allow(deprecated)]
 pub fn parse_line(line: &str) -> Result<Record, ScoreError> {
     let mut parts = line.split_whitespace();
     let name = parts.next().unwrap_or("");
-    let score = try!(scorekit::try(parts.next().unwrap_or("")));
+    let score = scorekit::r#try(parts.next().unwrap_or(""))?;
```

```diff
--- app2015/src/report/mod.rs
+++ app2018/src/report.rs
 mod table;
 
-pub use self::table::render;
+pub use table::render;
 
+use scorekit::bar;
+
-#[allow(anonymous_parameters)]
 pub trait Style {
-    fn line(&self, &str, u32) -> String;
+    fn line(&self, name: &str, score: u32) -> String;
 }
 ...
-#[allow(bare_trait_objects)]
-pub fn style_for(name: &str) -> Box<Style> {
+pub fn style_for(name: &str) -> Box<dyn Style> {
 ...
-#[allow(ellipsis_inclusive_range_patterns)]
 pub fn grade(score: u32) -> char {
     match score {
-        90...100 => 'A',
-        80...89 => 'B',
-        70...79 => 'C',
+        90..=100 => 'A',
+        80..=89 => 'B',
+        70..=79 => 'C',
```

```diff
--- app2015/src/report/table.rs
+++ app2018/src/report/table.rs
-use record::Record;
+use crate::record::Record;
 use super::Style;
 
-#[allow(bare_trait_objects)]
-pub fn render(records: &[Record], style: &Style) {
+pub fn render(records: &[Record], style: &dyn Style) {
 ...
-    println!("  평균 {}점 ({})", average, ::report::grade(average));
+    println!("  평균 {}점 ({})", average, crate::report::grade(average));
```

여기에 파일 배치의 차이(`report/mod.rs` → `report.rs`, 2.5절)가 더해집니다. 바뀐 줄을 모두 모으면 2.3~2.8절의 내용이 그대로 나옵니다.

**2015판이 막아 둔 경고.** app2015는 옛 모양을 보이려고 경고가 나는 코드를 일부러 쓰고, 그 자리에 `#[allow(…)]`와 이유 주석을 달았습니다. 막은 린트는 네 가지입니다. `#[allow]`를 지우고 빌드하면 다음 경고가 나옵니다(위치 줄은 생략).

| 막은 린트 | 위치 | 경고 문구 |
| --- | --- | --- |
| `deprecated` | `record.rs`의 `try!` | `` use of deprecated macro `try`: use the `?` operator instead `` |
| `anonymous_parameters` | `report/mod.rs`의 `Style::line` | `anonymous parameters are deprecated and will be removed in the next edition` |
| `bare_trait_objects` | `report/mod.rs`의 `Box<Style>`, `report/table.rs`의 `&Style` | `` trait objects without an explicit `dyn` are deprecated `` |
| `ellipsis_inclusive_range_patterns` | `report/mod.rs`의 `grade` | `` `...` range patterns are deprecated `` |

`#[allow]`를 지운 2015판을 빌드하면 Cargo가 마지막 줄에 `` run `cargo fix --bin "app2015" -p app2015` to apply 7 suggestions ``라고도 알려 줍니다. 이 자동 수정은 Part 2 `ch6-migration-practice`에서 다룹니다.

**섞어 쓰면 나는 에러.** 2018판에서 `r#`를 빼고 `scorekit::try(..)`라고 쓰면 파싱 에러가 납니다. [ch1](../ch1-editions-overview/README.md) 4.5절에서 2015 크레이트의 `async` 함수를 2024 에디션에서 그대로 불렀을 때와 같은 에러입니다.

```text
error: expected identifier, found reserved keyword `try`
  --> app2018/src/record.rs:18:27
   |
18 |     let score = scorekit::try(parts.next().unwrap_or(""))?;
   |                           ^^^ expected identifier, found reserved keyword
   |
help: escape `try` to use it as an identifier
   |
18 |     let score = scorekit::r#try(parts.next().unwrap_or(""))?;
   |                           ++
```

2018판 `table.rs`에 2015판의 `use record::Record;`와 `::report::grade(..)`를 넣으면 4.4절과 같은 E0432와 E0433이 나옵니다. 반대로 2018판 `main.rs`에 `#[macro_use] extern crate scorekit;`을 다시 넣어도 에러는 나지 않습니다(같은 매크로를 `use`로도 가져오므로 `` unused `#[macro_use]` import `` 경고만 납니다).

검증 명령(이 디렉터리에서): `cargo build`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`가 모두 경고 없이 통과합니다. `cargo fmt`는 패키지마다 `Cargo.toml`의 `edition`으로 소스를 읽으므로 2015 문법(익명 매개변수, `...` 패턴)이 있는 app2015도 문제없이 검사합니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 이 파트에서는 **에디션에 따라 결과가 달라지는 것**이 핵심이므로, 각 파일 맨 위 주석에 어느 에디션에서 실패하고 어느 에디션에서 통과하는지와 확인 명령을 적었습니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일합니다. `--edition`을 빼면 2015 에디션으로 컴파일된다는 점에 주의합니다.

```sh
cd part2-editions/ch2-rust-2015-2018
rustc --edition 2024 compile_fail/keyword_dyn_try_identifiers.rs --out-dir /tmp/rust-p2-ch2
rustc --edition 2015 compile_fail/keyword_dyn_try_identifiers.rs --out-dir /tmp/rust-p2-ch2
```

| 파일 | 실패하는 에디션 | 통과하는 에디션 |
| --- | --- | --- |
| `keyword_dyn_try_identifiers.rs` | 2018, 2021, 2024 | 2015 |
| `anonymous_trait_param.rs` | 2018, 2021, 2024 | 2015 (경고) |
| `try_macro.rs` | 2018, 2021, 2024 | 2015 (경고) |
| `e0432_crate_root_paths.rs` | 2018, 2021, 2024 | 2015 |
| `e0642_trait_param_pattern_2015.rs` | 2015 | 2018, 2021, 2024 |
| `raw_ident_reserved.rs` | 모든 에디션 | — |

[ch1](../ch1-editions-overview/README.md)의 `compile_fail/`에도 이 챕터와 이어지는 파일이 둘 있습니다. `keyword_async_identifier.rs`는 `async`를 변수 이름으로 쓴 2015 코드(2018 이후 실패), `e0432_use_relative_path_2015.rs`는 현재 모듈에서 출발하는 `use` 경로를 2015로 컴파일한 경우(2015에서만 E0432)입니다.

### 4.1 옛 코드의 이름이 새 키워드와 겹침

파일: [`compile_fail/keyword_dyn_try_identifiers.rs`](compile_fail/keyword_dyn_try_identifiers.rs)

```text
error: expected identifier, found reserved keyword `try`
  --> compile_fail/keyword_dyn_try_identifiers.rs:14:4
   |
14 | fn try(text: &str) -> Option<u32> {
   |    ^^^ expected identifier, found reserved keyword
   |
help: escape `try` to use it as an identifier
   |
14 | fn r#try(text: &str) -> Option<u32> {
   |    ++

error: expected identifier, found keyword `dyn`
  --> compile_fail/keyword_dyn_try_identifiers.rs:20:9
   |
20 |     let dyn = 3; // → 2018부터 에러: dyn은 키워드
   |         ^^^ expected identifier, found keyword

error: expected expression, found reserved keyword `try`
  --> compile_fail/keyword_dyn_try_identifiers.rs:21:17
   |
21 |     let total = try("7").unwrap_or(0) + dyn;
   |                 ^^^ expected expression
```

- **읽는 법**: "이름이 와야 할 자리에 키워드가 왔다"는 파싱 에러입니다. 예약 키워드 `try`에는 "reserved keyword", 엄격한 키워드 `dyn`에는 "keyword"라고 구분해 알려 줍니다. E 코드가 없고, 세 번째 에러처럼 키워드를 식 자리에서 만난 곳에는 `expected expression`이 붙습니다. 21행 끝의 `dyn`에는 따로 에러가 나오지 않습니다. 같은 문장 앞쪽의 `try`에서 이미 파싱이 실패했기 때문입니다.
- **원인**: 2015 에디션에서 키워드가 아니던 `try`와 약한 키워드이던 `dyn`이 2018 에디션에서 각각 예약 키워드, 엄격한 키워드가 되었습니다(2.7절). 2015로 컴파일하면 경고 없이 통과하고 `10`을 출력합니다.
- **해결**: 컴파일러의 `help:`대로 정의와 사용하는 곳 모두에 `r#`를 붙입니다(2.8절). `dyn`에도 같은 모양의 `` help: escape `dyn` to use it as an identifier ``가 붙습니다(위 인용에서는 생략). 내 코드라면 `parse_score`, `count`처럼 키워드가 아닌 이름으로 바꾸는 편이 읽기 좋습니다. 다른 크레이트가 공개한 이름이라면 `r#`가 유일한 방법입니다.

### 4.2 익명 트레이트 매개변수

파일: [`compile_fail/anonymous_trait_param.rs`](compile_fail/anonymous_trait_param.rs)

```text
error: expected one of `:`, `@`, or `|`, found `)`
  --> compile_fail/anonymous_trait_param.rs:14:24
   |
14 |     fn scale(&self, f64) -> f64; // → 2018부터 에러
   |                        ^ expected one of `:`, `@`, or `|`
   |
   = note: anonymous parameters are removed in the 2018 edition (see RFC 1685)
help: if this is a parameter name, give it a type
   |
14 |     fn scale(&self, f64: TypeName) -> f64; // → 2018부터 에러
   |                        ++++++++++
help: if this is a type, explicitly ignore the parameter name
   |
14 |     fn scale(&self, _: f64) -> f64; // → 2018부터 에러
   |                     ++
```

- **읽는 법**: 파서는 `f64`를 매개변수 이름(패턴)으로 읽고, 그 뒤에 `:`(타입), `@`(바인딩), `|`(or 패턴) 중 하나를 기다리다 `)`를 만났습니다. `note:`가 진짜 원인을 알려 줍니다.
- **원인**: 2015 에디션에서 허용되던 익명 매개변수가 2018에서 제거되었습니다(2.6절). 2015에서는 `anonymous_parameters` 경고와 함께 통과합니다.
- **해결**: 두 번째 `help:`가 맞습니다. `f64`는 타입이므로 `factor: f64`처럼 이름을 붙이거나 `_: f64`로 씁니다. 첫 번째 제안(`f64: TypeName`)은 `f64`가 이름이었을 경우를 위한 것입니다.

### 4.3 `try!`를 그대로 씀

파일: [`compile_fail/try_macro.rs`](compile_fail/try_macro.rs)

```text
error: use of deprecated `try` macro
  --> compile_fail/try_macro.rs:17:13
   |
17 |     let n = try!(text.parse::<i32>()); // → 2018부터 에러
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: in the 2018 edition `try` is a reserved keyword, and the `try!()` macro is deprecated
help: you can use the `?` operator instead
   |
17 -     let n = try!(text.parse::<i32>()); // → 2018부터 에러
17 +     let n = text.parse::<i32>()?; // → 2018부터 에러
   |
help: alternatively, you can still access the deprecated `try!()` macro using the "raw identifier" syntax
   |
17 |     let n = r#try!(text.parse::<i32>()); // → 2018부터 에러
   |             ++
```

- **읽는 법**: 메시지는 "deprecated 매크로를 썼다"지만 실제로는 경고가 아니라 에러입니다. `note:`가 이유(2018에서 `try`는 예약 키워드)를 알려 줍니다.
- **원인**: 2015 시절 에러 전파 매크로 `try!`를 2018 이후 에디션에서 그대로 썼습니다. 2015에서는 `` use of deprecated macro `try`: use the `?` operator instead `` 경고와 함께 통과합니다.
- **해결**: 첫 번째 `help:`대로 `?`로 바꿉니다. 두 번째 제안 `r#try!`는 컴파일은 되지만 deprecated 경고가 남습니다(예제 03).

### 4.4 2015의 크레이트 루트 기준 경로 — E0432, E0433

파일: [`compile_fail/e0432_crate_root_paths.rs`](compile_fail/e0432_crate_root_paths.rs)

```text
error[E0432]: unresolved import `units`
  --> compile_fail/e0432_crate_root_paths.rs:26:9
   |
26 |     use units::to_celsius; // → 2018부터 E0432
   |         ^^^^^
   |
   = note: `use` statements changed in Rust 2018; read more at <https://doc.rust-lang.org/edition-guide/rust-2018/module-system/path-clarity.html>
help: a similar path exists
   |
26 |     use crate::units::to_celsius; // → 2018부터 E0432
   |         +++++++

error[E0433]: cannot find `units` in the list of imported crates
  --> compile_fail/e0432_crate_root_paths.rs:29:32
   |
29 |         format!("{} {:.1}도", ::units::SCALE, to_celsius(f)) // → 2018부터 E0433
   |                                 ^^^^^ could not find `units` in the list of imported crates
```

- **읽는 법**: 첫 에러는 하위 모듈 `report` 안의 `use units::…`가 `report` 안에서 `units`를 찾지 못했다는 뜻이고, `note:`가 2018의 `use` 변경을 짚어 줍니다(링크는 `path-changes.html`로 넘어갑니다). 두 번째 에러의 "the list of imported crates"는 외부 크레이트 목록, 즉 2018에서 `::`가 찾는 곳입니다.
- **원인**: 2015 에디션의 두 규칙(`use` 경로와 `::` 경로는 크레이트 루트에서 출발)으로 쓴 코드를 2018 이후 에디션으로 컴파일했습니다(2.4절). 2015에서는 경고 없이 통과합니다.
- **해결**: 둘 다 `crate::`로 시작합니다. `use crate::units::to_celsius;`, `crate::units::SCALE`. 이렇게 고친 코드는 2015 에디션에서도 통과합니다.

### 4.5 2015 크레이트의 트레이트 메서드에 매개변수 패턴을 씀 — E0642

파일: [`compile_fail/e0642_trait_param_pattern_2015.rs`](compile_fail/e0642_trait_param_pattern_2015.rs) (2015 에디션에서만 실패)

```sh
rustc --edition 2015 compile_fail/e0642_trait_param_pattern_2015.rs --out-dir /tmp/rust-p2-ch2
```

```text
error[E0642]: patterns aren't allowed in methods without bodies
  --> compile_fail/e0642_trait_param_pattern_2015.rs:19:20
   |
19 |     fn rect(&self, (w, h): (u32, u32)) -> u32 {
   |                    ^^^^^^
   |
help: give this argument a name or use an underscore to ignore it
   |
19 -     fn rect(&self, (w, h): (u32, u32)) -> u32 {
19 +     fn rect(&self, _: (u32, u32)) -> u32 {
   |

error[E0425]: cannot find value `w` in this scope
  --> compile_fail/e0642_trait_param_pattern_2015.rs:21:9
   |
21 |         w * h
   |         ^ not found in this scope
```

- **읽는 법**: 트레이트 메서드의 매개변수 자리에 패턴 `(w, h)`가 왔다는 에러입니다. 메시지는 "본문이 없는 메서드"라고 하지만 이 메서드에는 본문이 있습니다. 컴파일러가 패턴을 버린 탓에 본문의 `w`, `h`가 가리킬 것이 없어져 E0425가 두 개 뒤따릅니다(`h`의 것은 생략).
- **원인**: 2015 에디션에서는 트레이트 메서드의 매개변수가 이름 없이 타입만 올 수도 있었기 때문에(2.6절) 패턴을 받지 않았습니다. 2018부터는 본문이 있는 트레이트 메서드에 패턴을 쓸 수 있어서 같은 파일이 경고 없이 통과합니다. 지금 문법에 익숙한 사람이 오래된 2015 크레이트를 고칠 때 만나는 실수입니다.
- **해결**: `help:`의 `_: (u32, u32)`를 그대로 따르면 본문의 `w`, `h`가 여전히 없어서 E0425가 남습니다. `fn rect(&self, size: (u32, u32)) -> u32 { let (w, h) = size; w * h }`처럼 이름을 붙이고 본문 안에서 구조 분해합니다. 이렇게 고친 코드는 모든 에디션에서 통과합니다.

### 4.6 raw 식별자로도 쓸 수 없는 이름

파일: [`compile_fail/raw_ident_reserved.rs`](compile_fail/raw_ident_reserved.rs) (모든 에디션에서 실패)

```text
error: `crate` cannot be a raw identifier
  --> compile_fail/raw_ident_reserved.rs:16:5
   |
16 |     r#crate: String, // → 에러: `crate` cannot be a raw identifier
   |     ^^^^^^^

error: `self` cannot be a raw identifier
  --> compile_fail/raw_ident_reserved.rs:21:9
   |
21 |     let r#self = Package {
   |         ^^^^^^
```

- **읽는 법**: `r#`를 붙여도 이 이름은 식별자가 될 수 없다는 뜻입니다. 같은 이름을 쓴 곳마다 에러가 나므로 이 파일에서는 여섯 개가 나옵니다. 같은 파일의 `r#type`은 문제없습니다.
- **원인**: `crate`, `self`, `super`, `Self`, `_`는 raw 식별자에서 제외됩니다(2.8절). 2015 에디션에서도 같은 에러입니다.
- **해결**: `krate`, `self_`처럼 다른 이름을 씁니다.

### 4.7 그 밖의 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| 2015 에디션에서 `extern crate` 없이 `use 크레이트::…` | E0432, `` on Rust 2015, `extern crate` is required… `` (2.3절) | 크레이트 루트에 `extern crate 크레이트;` |
| 2015 에디션(또는 `--edition` 없는 `rustc`)에서 `async fn` | E0670 `` `async fn` is not permitted in Rust 2015 `` | 에디션을 2018 이후로 |
| 2018 이후 `pub(in a)` | `relative paths are not supported in visibilities in 2018 edition or later` (2.4절) | `pub(in crate::a)` |
| 서식 문자열에 `{r#type}` | `invalid format string: raw identifiers are not supported` (2.8절) | `{type}` |
| 필드 이름 `r#type`을 `job.type`으로 읽음 | `` expected identifier, found keyword `type` `` | `job.r#type` |
| 2015 에디션에서 하위 모듈의 `use 자식모듈::…` | E0432 ([ch1](../ch1-editions-overview/README.md)의 `compile_fail/e0432_use_relative_path_2015.rs`) | `use self::자식모듈::…` |
| 2018 이후 에디션에서 2015 크레이트의 `async`·`try` 같은 이름을 그대로 부름 | `` expected identifier, found keyword `async` `` ([ch1](../ch1-editions-overview/README.md) 4.5절, 3절의 `projects/scoreboard`) | `크레이트::r#async` |

## 5. 연습 문제

### 연습 1. 2015 코드를 모든 에디션에서 통과하게 고치기

아래는 2015 에디션으로만 컴파일되는 코드입니다. `compile_fail/` 밖의 임시 파일에 저장하고, **2015와 2024 양쪽에서 경고 없이** 컴파일되도록 고치세요. 출력은 `B 85` 그대로여야 합니다.

```rust
mod grade {
    pub fn of(score: u32) -> char {
        match score {
            90...100 => 'A',
            80...89 => 'B',
            _ => 'F',
        }
    }
}

mod report {
    use grade::of;

    pub trait Show {
        fn show(&self, u32) -> String;
    }

    pub struct Line;

    impl Show for Line {
        fn show(&self, score: u32) -> String {
            format!("{} {}", of(score), score)
        }
    }

    pub fn print(s: &Show, text: &str) -> Result<(), ::std::num::ParseIntError> {
        let score = try!(text.parse::<u32>());
        println!("{}", s.show(score));
        Ok(())
    }
}

fn main() {
    report::print(&report::Line, "85").unwrap();
}
```

<details>
<summary>힌트</summary>

- 먼저 `rustc --edition 2015`로 경고를 보고, `rustc --edition 2024`로 에러를 봅니다. 고칠 곳은 다섯 종류입니다(`...` 패턴, `use grade::of`, 익명 매개변수, `&Show`, `try!`). `use grade::of`는 2015에서 기본으로는 경고가 없고, `rustc --edition 2015 -W rust-2018-compatibility`로 컴파일하면 `` absolute paths must start with `self`, `super`, `crate`, or an external crate name in the 2018 edition `` 경고가 납니다.
- 2.2절의 표에 따르면 `..=`, `crate::` 경로, `dyn`, `?`는 모두 2015 에디션에서도 쓸 수 있습니다. 그래서 지금 문법으로 고치면 두 에디션에서 함께 통과합니다.
- `::std::num::ParseIntError`는 고칠 필요가 없습니다. 2015에서는 크레이트 루트의 `std`, 2018 이후에는 외부 크레이트 `std`를 가리켜 결과가 같습니다(2.4절).

</details>

### 연습 2. 옛 라이브러리를 2018 에디션으로 올리기

`projects/scoreboard/scorekit/Cargo.toml`의 `edition`을 `"2018"`로 바꾸세요. 빌드가 실패하면 `scorekit`만 고쳐서 통과시키고, 두 프로그램(`app2015`, `app2018`)은 **한 글자도 고치지 않은 채** 이전과 같은 출력을 내는지 확인하세요. 왜 `app2015`는 여전히 `scorekit::try`라고 부를 수 있는지 설명해 보세요. 확인이 끝나면 `edition`을 `"2015"`로 되돌립니다.

<details>
<summary>힌트</summary>

- 에러는 하나입니다. `scorekit/src/lib.rs`의 `pub fn try(`에서 `` expected identifier, found reserved keyword `try` ``가 납니다. 정의를 `pub fn r#try(`로 바꿉니다.
- 함수의 진짜 이름은 여전히 `try`입니다(`r#`는 이름의 일부가 아님, 2.8절). `r#`가 필요한지는 그 이름을 **쓰는 코드**의 에디션이 정하므로, 2015 에디션인 `app2015`는 `scorekit::try`, 2018 에디션인 `app2018`은 `scorekit::r#try`로 그대로 부릅니다.
- `bar!` 매크로와 `$crate`는 고칠 필요가 없습니다. 에디션을 올린 라이브러리도 `#[macro_use] extern crate`(app2015)와 `use scorekit::bar;`(app2018) 양쪽에서 쓸 수 있습니다.

</details>

### 연습 3. 2015판의 에디션만 올려 보기

`projects/scoreboard/app2015/Cargo.toml`의 `edition`만 `"2018"`로 바꾸면 에러가 몇 개, 어디서 날지 소스를 보고 먼저 예상한 뒤 `cargo build -p app2015`로 확인하세요. 그다음 `app2018/Cargo.toml`의 `edition`을 `"2021"`, `"2024"`로 차례로 바꿔 빌드하고 출력이 같은지 확인하세요. 끝나면 두 값을 되돌립니다.

<details>
<summary>힌트</summary>

- `app2015`는 다섯 개입니다. 익명 매개변수 두 개(`&str` 뒤의 `,`와 `u32` 뒤의 `)`에서 각각 `` expected one of `:`, `@`, or `|` ``), `try!` 하나, `use record::Record`의 E0432 하나, `::report::grade`의 E0433 하나입니다. `#[macro_use] extern crate`는 2018에서도 유효하고, `Box<Style>`과 `...` 패턴은 2018에서도 경고일 뿐인 데다 `#[allow]`로 막혀 있어 에러가 아닙니다. `scorekit::try`는 바깥의 `try!`가 먼저 에러가 되어 따로 보고되지 않습니다.
- `app2018`은 2021과 2024에서도 고칠 것 없이 빌드되고 출력도 같습니다. 2018판이 2021에서 에러가 될 모양(`dyn` 없는 트레이트 객체, `...` 패턴)을 이미 쓰지 않기 때문입니다.
- 워크스페이스 `Cargo.toml`에 `resolver = "3"`이 없으면 2021 이상으로 올렸을 때 Cargo가 `` virtual workspace defaulting to `resolver = "1"` ``라는 경고를 냅니다. 리졸버는 Part 2 `ch3-rust-2021`과 Part 3에서 다룹니다.

</details>

## 6. 참고 자료

- The Edition Guide (기준 문서)
  - [Rust 2015](https://doc.rust-lang.org/edition-guide/rust-2015/index.html): 2015 에디션의 주제(안정성)와 "기본 에디션"인 이유.
  - [Rust 2018](https://doc.rust-lang.org/edition-guide/rust-2018/index.html): RFC 2052, 1.31.0, 대부분의 기능이 모든 에디션에 들어갔다는 설명.
  - [Path and module system changes](https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html): `extern crate`, sysroot 크레이트 예외, 매크로 가져오기, 크레이트 이름 바꾸기, `crate::`, `::`, `mod.rs`, `use` 경로의 통일. `mod.rs`에 관한 설명은 2.5절의 차이를 함께 봅니다.
  - [Anonymous trait function parameters deprecated](https://doc.rust-lang.org/edition-guide/rust-2018/trait-fn-parameters.html), [New keywords](https://doc.rust-lang.org/edition-guide/rust-2018/new-keywords.html), [Method dispatch for raw pointers to inference variables](https://doc.rust-lang.org/edition-guide/rust-2018/tyvar-behind-raw-pointer.html), [Cargo changes](https://doc.rust-lang.org/edition-guide/rust-2018/cargo.html).
- The Rust Reference
  - [Keywords](https://doc.rust-lang.org/reference/keywords.html): 엄격한·예약·약한 키워드 목록과 에디션별 차이(2018의 `async`·`await`·`dyn`·`try`, 2015의 약한 키워드 `dyn`).
  - [Identifiers](https://doc.rust-lang.org/reference/identifiers.html): raw 식별자의 정의와 `r#`로도 쓸 수 없는 이름(`_`, `crate`, `self`, `Self`, `super`).
  - [Paths](https://doc.rust-lang.org/reference/paths.html): `::`로 시작하는 경로의 에디션별 차이. [Use declarations](https://doc.rust-lang.org/reference/items/use-declarations.html): 2015의 `use` 경로가 크레이트 루트 기준이라는 것과 외부 프렐류드를 쓸 수 없다는 것. [Preludes](https://doc.rust-lang.org/reference/names/preludes.html): 외부 프렐류드와 2015에서 `std`가 크레이트 루트에 들어가던 규칙.
- The Book [부록 A: 키워드](https://doc.rust-lang.org/book/appendix-01-keywords.html)의 Raw Identifiers 절: `r#match` 예와 2015 라이브러리의 `try` 함수를 부르는 예.
- [`std::try!` 문서](https://doc.rust-lang.org/std/macro.try.html): 1.39.0부터 deprecated, 2018에서는 `r#try`로 불러야 한다는 설명.
