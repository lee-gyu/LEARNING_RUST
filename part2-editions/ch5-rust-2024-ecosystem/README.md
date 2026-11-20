# ch5. 2024 에디션의 생태계 변경: 표준 라이브러리, Cargo, rustdoc, rustfmt

Part 2의 다섯 번째 챕터입니다. [ch4](../ch4-rust-2024-language/README.md)가 2024 에디션(공식 이름 "Rust 2024")의 **언어** 변경을 다뤘다면, 이 챕터는 같은 에디션이 **언어 바깥**에서 바꾼 것을 다룹니다. 이 챕터의 줄기는 하나입니다. **컴파일러만 에디션을 보는 것이 아닙니다.** 표준 라이브러리의 프렐류드와 함수의 안전성, Cargo의 의존성 리졸버와 매니페스트 검사, rustdoc의 문서 테스트, rustfmt의 서식이 모두 크레이트의 에디션 값을 읽고 동작을 바꿉니다.

앞 챕터들이 이 챕터로 미뤄 둔 것도 여기서 정리합니다.

- [Part 1 ch10](../../part1-standard-lib/ch10-env-process-time/README.md) 2.4절: `env::set_var`가 왜 unsafe인지 보았습니다. 이 함수가 2021 에디션까지는 안전한 함수였다가 바뀐 경위와 옮기는 방법(2.5절)
- [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md)의 `projects/adder`: `cargo test` 끝에 붙는 `all doctests ran in …; merged doctests compilation took …` 줄(2.8절)
- [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.2절의 프렐류드 표에서 "2024부터"인 `Future`, `IntoFuture`(2.3절)
- [ch3](../ch3-rust-2021/README.md) 2.4절의 배열 `IntoIterator`와 같은 일을 `Box<[T]>`에 한 변경(2.4절)
- [ch1](../ch1-editions-overview/README.md) 2.5절과 [ch3](../ch3-rust-2021/README.md) 2.3절이 예고한 `resolver = "3"`(2.6절)

- 패키지: `p2-ch5-rust-2024-ecosystem`
- 기준 문서: [Edition Guide 5.2 Standard library](https://doc.rust-lang.org/edition-guide/rust-2024/standard-library.html), [5.3 Cargo](https://doc.rust-lang.org/edition-guide/rust-2024/cargo.html), [5.4 Rustdoc](https://doc.rust-lang.org/edition-guide/rust-2024/rustdoc.html), [5.5 Rustfmt](https://doc.rust-lang.org/edition-guide/rust-2024/rustfmt.html)와 그 하위 절 12개
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, rustfmt 1.11.0, macOS (aarch64). 2021 에디션의 동작은 같은 툴체인에서 `--edition 2021`이나 `edition = "2021"`로 확인했습니다. 상속 `default-features`의 예전 동작(2.7절)만 이 환경에 남아 있던 cargo 1.98.1로 확인했습니다.
- 실습 자료: `examples/`(2024 에디션의 동작), [`projects/ecosystem_compare`](projects/ecosystem_compare/Cargo.toml)(같은 코드를 2021과 2024로 비교), [`projects/msrv_resolver`](projects/msrv_resolver/Cargo.toml)(리졸버 3, crates.io의 `smol_str`을 내려받으므로 처음 빌드할 때 네트워크가 필요합니다), `compile_fail/`.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 언어가 아닌 도구도 에디션을 본다는 것을 설명하고, rustc(표준 라이브러리 관련 규칙), Cargo, rustdoc, rustfmt가 각각 에디션 값을 어디서 받아 무엇을 정하는지 `-v` 출력으로 확인할 수 있다.
2. 2024 에디션 표준 라이브러리의 세 변경(프렐류드의 `Future`/`IntoFuture`, `Box<[T]>`의 `.into_iter()`, 새로 unsafe가 된 함수)이 2021 코드에서 컴파일 에러나 조용한 동작 변화로 나타나는 것을 실행해 확인하고, 트레이트 이름 적기·`.iter()`·`unsafe` 블록과 `SAFETY` 주석·`Command::env`·`pre_exec`로 고칠 수 있다.
3. 함수 하나의 안전성이 부르는 코드의 에디션에 따라 달라지는 장치를 설명하고, `set_var`를 옮길 때 "unsafe 블록으로 감싸기"보다 나은 선택지를 고를 수 있다.
4. rust-version을 고려하는 리졸버(리졸버 3)가 의존성 버전을 고르는 방식과 `Cargo.lock`의 역할을 설명하고, 에디션·`resolver`·`--ignore-rust-version`·`resolver.incompatible-rust-versions`로 켜고 끌 수 있다. 2024 에디션이 거부하는 매니페스트 키와 상속 `default-features`의 변화를 알아볼 수 있다.
5. 문서 테스트 병합이 바꾸는 것과 `standalone_crate`가 필요한 경우, 중첩 `include_str!`의 경로 기준을 설명할 수 있다. rustfmt의 `edition`과 `style_edition`을 구분하고 2021/2024 스타일의 차이를 실제 출력으로 보일 수 있다.

## 2. 핵심 개념

### 2.1 이 챕터의 지도

Edition Guide 5.2~5.5절의 항목입니다. "2021 코드에 생기는 일"은 2021 에디션 코드를 그대로 2024 에디션으로 옮겼을 때의 결과입니다. 린트는 에디션 호환성 린트 그룹 `rust-2024-compatibility`에 들어 있는 것이고, 기본 린트 수준은 `rustc -W help`로 확인했습니다.

| 절 | 항목 (Edition Guide) | 2021 코드에 생기는 일 | 미리 알려 주는 것 (2021 기본 수준) | 확인하는 곳 |
| --- | --- | --- | --- | --- |
| 2.3 | 프렐류드에 `Future`/`IntoFuture` (5.2.1) | E0034, 또는 다른 메서드가 **조용히** 호출됨 | `rust_2024_prelude_collisions` (allow) | 예제 01, `ecosystem_compare` 1, 4.1절 |
| 2.4 | `Box<[T]>`의 `IntoIterator` (5.2.2) | `.into_iter()`가 `&T` 대신 `T`를 내놓음 | `boxed_slice_into_iter` (**warn**) | 예제 02, `ecosystem_compare` 2, 4.2·4.3절 |
| 2.5 | 새로 unsafe가 된 함수 (5.2.3) | E0133 | `deprecated_safe_2024` (allow) | 예제 03, 4.4·4.5절 |
| 2.6 | rust-version 인식 리졸버 (5.3.1) | 의존성이 더 낮은 버전으로 골라질 수 있음 | 없음 | `msrv_resolver` |
| 2.7 | 테이블·키 이름 일관성 (5.3.2) | 매니페스트 에러 | Cargo 경고 | 2.7절 |
| 2.7 | 상속 `default-features` (5.3.3) | 꺼지지 않던 기본 기능이 꺼짐 | Cargo 경고 | 2.7절 |
| 2.8 | 문서 테스트 병합 (5.4.1) | 줄 번호·모듈 경로에 기댄 문서 테스트 실패 | 없음 | `ecosystem_compare` 문서 테스트 |
| 2.9 | 중첩 `include!` 경로 (5.4.2) | 문서 테스트 컴파일 에러 | 없음 | `ecosystem_compare` 문서 테스트 |
| 2.10 | rustfmt 스타일 에디션 (5.5.1~5.5.4) | `cargo fmt`가 서식을 바꿈 | 없음 | 예제 04, `ecosystem_compare` |

[ch3](../ch3-rust-2021/README.md), [ch4](../ch4-rust-2024-language/README.md)와 같이 각 절은 **2021에서는 → 2024에서는 → 왜 바꿨나 → 옮길 때** 순서로 설명합니다. `cargo fix --edition`이 이 변경들 가운데 무엇을 자동으로 고치는지는 [Part 2 ch6](../ch6-migration-practice/README.md)에서 다룹니다. 미리 말해 두면, 리졸버·문서 테스트·`include!` 경로에는 자동 수정이 없고, `set_var`의 자동 수정은 `unsafe` 블록으로 감싸 주기만 합니다(2.5절).

### 2.2 네 도구는 에디션을 어디서 읽나

[ch1](../ch1-editions-overview/README.md) 2.6절에서 Cargo가 크레이트마다 `rustc --edition`을 넘기는 것을 보았습니다. 다른 도구도 마찬가지입니다.

| 도구 | 에디션 값을 받는 곳 | 그 값으로 정하는 것 (이 챕터) |
| --- | --- | --- |
| rustc | `--edition` (Cargo가 패키지의 `edition`을 넘김) | 자동으로 들어오는 프렐류드(`std::prelude::rust_2024`), `Box<[T]>`의 `.into_iter()`가 고르는 구현, `set_var` 등이 unsafe인지 |
| Cargo | `Cargo.toml`의 `edition` | 리졸버 버전의 기본값(워크스페이스 루트 패키지의 값만), 받아들이는 매니페스트 키, 상속 `default-features` 처리 |
| rustdoc | `--edition` (`cargo test --doc`이 넘김), 코드 블록의 `edition2021` 같은 표시 | 문서 테스트를 묶어 컴파일할지, `include!` 경로의 기준 |
| rustfmt | `--edition` (`cargo fmt`가 크레이트마다 넘김), `rustfmt.toml`의 `edition`. 스타일은 `style_edition`/`--style-edition`, 없으면 `edition`을 따름 | 소스를 읽는 문법, 서식에 쓰는 스타일 가이드의 판 |

`-v`를 붙이면 넘어가는 값이 보입니다. `projects/ecosystem_compare`에서 실행했습니다(경로는 `…`로 줄였습니다).

```text
$ cargo test --doc -p e2024 -v     # 출력 중 한 줄의 앞부분
     Running `…/rustdoc --edition=2024 --crate-type lib --color auto --crate-name e2024 --test …`
$ cargo fmt -v
[lib (2021)] "…/e2021/src/lib.rs"
[lib (2024)] "…/e2024/src/lib.rs"
[bin (2021)] "…/shared/main.rs"
rustfmt --edition 2021 …/e2021/src/lib.rs …/shared/main.rs
rustfmt --edition 2024 …/e2024/src/lib.rs
```

`rustfmt`를 Cargo 없이 직접 실행하면 아무도 에디션을 넘겨주지 않으므로 2015 에디션으로 읽습니다([ch1](../ch1-editions-overview/README.md) 4.7절). 2.10절에서 다시 봅니다.

**rustc 쪽의 규칙은 코드를 쓴 크레이트의 에디션을 따릅니다.** 프렐류드, `Box<[T]>`의 `.into_iter()`, `set_var`의 안전성은 모두 그 호출을 **적은** 크레이트의 에디션으로 정해집니다. 매크로도 [ch1](../ch1-editions-overview/README.md) 2.8절의 규칙을 따릅니다. 2021 크레이트의 매크로 본문에 `std::env::set_var(..)`와 `$b.into_iter().next()`를 두고 2024 크레이트에서 펼쳤더니, `unsafe` 없이 컴파일되었고 `.into_iter()`는 `Option<&u32>`를 내놓았습니다(임시 프로젝트로 확인했습니다). 반면 Cargo의 리졸버는 워크스페이스에 하나뿐이라 루트 패키지의 에디션만 봅니다(2.6절).

### 2.3 프렐류드에 `Future`와 `IntoFuture`가 들어왔습니다

**2021에서는.** 2021 에디션의 프렐류드(`std::prelude::rust_2021`)에는 `Future`와 `IntoFuture`가 없습니다. 반환 타입에 `impl Future`를 적거나 `IntoFuture`를 구현하려면 `use std::future::Future;`가 필요했습니다. [`01_prelude_future.rs`](examples/01_prelude_future.rs)를 `rustc --edition 2021`로 컴파일하면 두 트레이트를 이름으로 적은 곳마다 E0405(`` cannot find trait `Future` in this scope ``)가 납니다.

**2024에서는.** `std::prelude::rust_2024`는 2021 프렐류드에 `Future`와 `IntoFuture` 두 트레이트를 더한 목록입니다([Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.2절의 표). 예제 01의 1번 구역은 `use` 없이 `dyn Future`와 `impl IntoFuture`를 씁니다. `.await`가 먼저 `IntoFuture::into_future`를 부른다는 것은 [Part 1 ch13](../../part1-standard-lib/ch13-async-primitives/README.md) 2.8절에서 보았습니다.

**왜 에디션이 필요했나.** [ch3](../ch3-rust-2021/README.md) 2.2절에서 본 대로, 프렐류드에 **트레이트**를 더하면 그 트레이트의 메서드가 모든 모듈에서 메서드 탐색의 후보가 됩니다. 메서드 탐색은 수신자 타입을 `T`, `&T`, `&mut T` 순서로 넓혀 가며 후보를 찾고, 후보를 찾은 첫 단계에서 멈춥니다. `Future::poll`은 `self: Pin<&mut Self>`를, `IntoFuture::into_future`는 `self`를 받습니다. 그래서 이미 `poll`이나 `into_future`라는 메서드를 가진 사용자 트레이트가 있으면 결과가 갈립니다. 세 경우를 이 툴체인으로 확인했습니다.

| 2021 코드의 사용자 트레이트 | 2024에서 메서드 호출 문법으로 부르면 | 확인한 곳 |
| --- | --- | --- |
| `fn into_future(self)` (`IntoTask`, 퓨처에 포괄 구현) | 같은 단계에 후보가 둘이라 E0034 | 4.1절 |
| `fn poll(&self, cx: &mut Context<'_>) -> Poll<()>` (`Peek`) | 자동 참조가 필요 없는 `Future::poll`이 먼저 찾아져 **조용히** 퓨처를 폴링함 | 예제 01의 2번 구역, `ecosystem_compare` 1번 구역 |
| `fn poll(&self)` (Edition Guide의 `MyPoller`) | 역시 `Future::poll`이 골라지고, 인자 수가 맞지 않아 E0061 | 아래 인용 |

두 번째 줄이 가장 위험합니다. `ecosystem_compare`의 1번 구역은 같은 줄 `fut.as_mut().poll(&mut cx)`가 2021에서는 `Peek::poll`을 불러 `Pending`을, 2024에서는 async 블록을 실행해 `Ready(())`를 내놓습니다(3절). 반환 타입이 우연히 `Poll<()>`로 같아서 어느 쪽도 에러가 나지 않습니다.

Edition Guide는 세 번째 줄의 `MyPoller` 예를 "어느 메서드를 부를지 모호해진다"고 설명하지만, 1.100.0에서 실제로 나온 것은 모호성 에러(E0034)가 아니라 다음 에러였습니다. 컴파일러가 `Future::poll`을 골랐다고 직접 알려 줍니다.

```text
error[E0061]: this method takes 1 argument but 0 arguments were supplied
note: the `poll` call is resolved to the method in `std::future::Future`, shadowing the method of the same name on trait `MyPoller`
```

**옮길 때.** 사용자 트레이트의 메서드를 부르려던 것이라면 트레이트 이름을 경로에 적습니다. `Peek::poll(&fut.as_mut(), &mut cx)`, `IntoTask::into_future(ready)`입니다(예제 01의 2·3번 구역). 이렇게 쓴 코드는 두 에디션에서 같은 메서드를 부릅니다. 바뀔 곳을 미리 찾으려면 2021 에디션에서 린트를 켭니다. `ecosystem_compare`에서 실행한 결과입니다(핵심 줄만 옮겼습니다).

```text
$ cargo rustc -q -p e2021 --bin ecosystem_2021 -- -W rust-2024-compatibility
warning: trait method `poll` will become ambiguous in Rust 2024
  --> e2021/../shared/main.rs:50:21
   |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^ help: disambiguate the associated function: `Peek::poll(&fut.as_mut(), &mut cx)`
```

ch3에서 `TryInto`에 대해 본 것처럼, 경고는 "모호해진다", "2024에서는 에러"라고 말하지만 이 경우의 실제 결과는 에러가 아니라 조용한 동작 변화입니다. 경고 문구보다 결과 표를 믿습니다.

### 2.4 `Box<[T]>`가 값으로 `IntoIterator`를 구현합니다

**2021에서는.** 1.80.0 전까지 `Box<[T]>`([Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.7절)에는 `IntoIterator` 구현이 없었습니다. 그래서 `boxed.into_iter()`는 메서드 탐색이 `Box<[T]>`에서 후보를 찾지 못하고 역참조를 따라가 `&[T]`의 구현을 골랐고, `&T`를 내놓았습니다. 이름은 `into_iter`인데 실제로는 `iter()`였던 것입니다.

**1.80.0의 절충과 2024에서는.** 1.80.0은 `Box<[T]>`, `&Box<[T]>`, `&mut Box<[T]>`의 `IntoIterator` 구현을 **모든 에디션에** 더했습니다. 대신 2021 이전 에디션 코드에서 **메서드 호출 문법으로** `.into_iter()`를 부를 때만 컴파일러가 `Box<[T]>`의 구현을 못 본 척합니다. 그러면 메서드 탐색이 다음 단계로 넘어가 `&Box<[T]>`의 구현을 고르므로 예전처럼 `&T`가 나옵니다. 이 절 끝에 인용한 경고 문구의 `<&Box<[T]> as IntoIterator>::into_iter`가 그것입니다. 표준 라이브러리 소스에서 `IntoIterator` 트레이트에 붙은 속성이 이 장치입니다(로컬 문서의 소스 페이지 `core/iter/traits/collect.rs`).

```rust
#[rustc_skip_during_method_dispatch(array, boxed_slice)]
pub const trait IntoIterator { /* … */ }
```

`array`는 ch3에서 본 배열의 예외이고, `boxed_slice`가 이 변경입니다. 2024 에디션에서는 이 예외가 없어져 `boxed.into_iter()`가 값을 내놓습니다. 두 에디션에서 항목 타입을 확인한 결과입니다(`boxed: Box<[u32]>`).

| 쓰는 모양 | 2021 | 2024 |
| --- | --- | --- |
| `boxed.into_iter()` | `&u32` | `u32` |
| `boxed.iter()` | `&u32` | `&u32` |
| `(&boxed).into_iter()` | `&u32` | `&u32` |
| `IntoIterator::into_iter(boxed)` | `u32` | `u32` |
| `for x in boxed` | `u32` | `u32` |
| `(0..3).zip(boxed)` | `(i32, u32)` | `(i32, u32)` |

[`02_box_slice_into_iter.rs`](examples/02_box_slice_into_iter.rs)가 2024 에디션의 동작을, `ecosystem_compare`의 2번 구역이 두 에디션의 차이(`Option<&u32>`와 `Option<u32>`)를 보여 줍니다. 예제 02의 1번 구역이 출력하는 반복자 타입은 `alloc::vec::into_iter::IntoIter<u32>`입니다. `Box<[T]>`를 `Vec<T>`로 바꾸는 데 복사가 필요 없어서 `Vec`의 반복자를 그대로 씁니다.

**왜 에디션이 필요했나.** 배열과 같은 이유입니다([ch3](../ch3-rust-2021/README.md) 2.4절). 항목 타입이 `&T`에서 `T`로 바뀌면 `Vec<&T>`를 모으던 코드는 타입 에러가 나고(4.2절), 반대로 값을 기대한 코드는 2021에서 에러가 납니다(4.3절). 두 에디션에서 모두 컴파일되면서 결과만 다른 코드도 생깁니다. 구현을 추가하는 것만으로는 기존 `boxed.into_iter()`의 뜻이 바뀌므로, 메서드 문법의 뜻을 바꾸는 일만 에디션에 묶었습니다.

**옮길 때.** `boxed_slice_into_iter` 린트는 2024 이전 모든 에디션에서 기본으로 경고합니다. 2021에서 나오는 경고입니다(핵심 줄만 옮겼습니다).

```text
warning: this method call resolves to `<&Box<[T]> as IntoIterator>::into_iter` (due to backwards compatibility), but will resolve to `<Box<[T]> as IntoIterator>::into_iter` in Rust 2024
  = warning: this changes meaning in Rust 2024
  = note: `#[warn(boxed_slice_into_iter)]` (part of `#[warn(rust_2024_compatibility)]`) on by default
help: use `.iter()` instead of `.into_iter()` to avoid ambiguity
help: or use `IntoIterator::into_iter(..)` instead of `.into_iter()` to explicitly iterate by value
```

참조를 원했다면 `.iter()`로, 값을 원했다면 `IntoIterator::into_iter(boxed)`로 바꿉니다. 둘 다 모든 에디션에서 같은 뜻입니다.

### 2.5 새로 unsafe가 된 함수: `set_var`, `remove_var`, `before_exec`

**2021에서는.** `env::set_var`, `env::remove_var`, 유닉스의 `CommandExt::before_exec`는 `unsafe` 없이 부를 수 있었고, 기본 설정에서는 **경고도 나지 않습니다**. 미리 알려 주는 린트 `deprecated_safe_2024`의 기본 수준이 allow이기 때문입니다. 켜면 이렇게 나옵니다(`rustc --edition 2021 -W deprecated-safe-2024`, 핵심 줄만 옮겼습니다).

```text
warning: call to deprecated safe function `std::env::set_var` is unsafe and requires unsafe block
  = warning: this is accepted in the current edition (Rust 2021) but is a hard error in Rust 2024!
help: you can wrap the call in an `unsafe` block if you can guarantee that the environment access only happens in single-threaded code
  |
3 +     // FIXME: Audit that the environment access only happens in single-threaded code.
4 ~     unsafe { env::set_var("A_KEY", "1") };
```

`before_exec`는 1.37.0부터 deprecated라서 2021에서도 `` use of deprecated method `std::os::unix::process::CommandExt::before_exec`: should be unsafe, use `pre_exec` instead `` 경고가 납니다. 같은 일을 하는 `pre_exec`는 처음부터 unsafe 함수입니다.

**2024에서는.** 세 함수를 `unsafe` 블록 밖에서 부르면 E0133입니다. `set_var`의 에러는 [Part 1 ch10](../../part1-standard-lib/ch10-env-process-time/README.md) 4.2절에, `remove_var`와 `before_exec`의 에러는 4.4·4.5절에 있습니다. 이유도 Part 1 ch10 2.4절에서 보았습니다. 유닉스 계열에서는 다른 스레드가 환경을 읽는 동안 바꾸면 미정의 동작이 될 수 있고, 누가 환경을 읽는지(C 라이브러리 포함) 프로그램이 다 알 수 없습니다.

**에디션에 따라 안전성이 달라지는 장치.** 표준 라이브러리는 하나인데 어떻게 2021 크레이트에서는 안전하고 2024 크레이트에서는 unsafe일까요? 로컬 문서의 소스 페이지(`std/env.rs`)를 보면 함수 자체는 `unsafe fn`으로 선언되어 있고, 컴파일러에게만 의미가 있는 속성이 붙어 있습니다.

```rust
#[rustc_deprecated_safe_2024(
    audit_that = "the environment access only happens in single-threaded code"
)]
pub unsafe fn set_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) { /* … */ }
```

이 속성은 "2024 이전 에디션의 코드가 부를 때는 안전한 함수처럼 받아 주라"는 뜻입니다. 그래서 2021 코드는 `unsafe` 없이도, `unsafe` 블록 안에서도 부를 수 있습니다. **2021에서 `unsafe` 블록으로 감싸도 `unused_unsafe` 경고가 나지 않으므로**, 에디션을 올리기 전에 미리 고쳐 둘 수 있습니다(`ecosystem_compare`의 3번 구역이 두 에디션에서 같은 소스로 컴파일됩니다). `audit_that`의 문장은 위 `help:`의 `FIXME:` 주석으로 쓰입니다. `before_exec`에는 `audit_that = "the closure is async-signal-safe"`가 붙어 있습니다.

**왜 에디션이 필요했나.** Edition Guide의 설명대로, 이 함수들은 처음부터 unsafe였어야 했지만 안전한 함수로 안정화되었습니다. 함수에 `unsafe`를 붙이면 기존 호출이 모두 `unsafe` 블록을 요구하게 되므로 호환되지 않는 변경입니다. 그래서 2024 에디션 코드에서만 unsafe로 다룹니다.

**옮길 때.** `unsafe { }`로 감싸면 컴파일은 되지만, 그것은 "이 호출이 안전 조건을 지킨다"고 프로그래머가 서명하는 일입니다. 자동 수정도 감싸고 `FIXME` 주석을 달 뿐 조건을 확인해 주지 않습니다. 먼저 **왜 환경을 바꾸는지** 묻고 고릅니다. [`03_env_unsafe.rs`](examples/03_env_unsafe.rs)의 구역 번호입니다.

| 하려던 일 | 고르는 방법 | 예제 03 |
| --- | --- | --- |
| 정말 이 프로세스의 환경을 바꿔야 함 | 다른 스레드를 만들기 전(`main` 첫머리)에 `unsafe` 블록으로 부르고, 왜 안전한지 `// SAFETY:` 주석에 적음 | 1번 |
| 자식 프로세스에 환경 변수를 넘기거나 빼기 | `Command::env`, `env_remove`, `env_clear` — 자식의 환경만 정하므로 unsafe가 아님 | 2번 |
| 함수나 테스트에 설정 값을 넘기기 | 환경 변수는 한 번만 읽고(`env::var`는 안전) 값을 인자로 넘김 | 3번 |
| `before_exec`를 쓰던 코드 | `pre_exec`로 바꾸고 클로저가 비동기 시그널 안전한지 확인한 뒤 `SAFETY` 주석 | 4번 |

세 번째 줄은 테스트에서 특히 중요합니다. `cargo test`는 기본으로 테스트들을 여러 스레드에서 동시에 실행하므로([Part 0 ch11](../../part0-getting-started/ch11-testing/README.md)), 테스트 안에서 `set_var`를 부르면 "다른 스레드가 없다"는 조건을 지킬 수 없습니다. 값을 해석하는 함수(`parse_level`)를 따로 두고 값을 직접 넘겨 시험합니다.

### 2.6 rust-version을 고려하는 리졸버 (리졸버 3)

**배경.** [ch1](../ch1-editions-overview/README.md) 2.5절에서 본 `rust-version`은 "이 패키지는 rustc 몇 버전부터 지원한다"는 약속입니다. 이 약속은 내 코드만으로 지켜지지 않습니다. 의존성의 새 버전이 더 높은 `rust-version`을 요구하면, 오래된 컴파일러를 쓰는 사용자는 빌드할 수 없게 됩니다.

**2021에서는.** 리졸버 2([ch3](../ch3-rust-2021/README.md) 2.3절)는 버전 요구 사항(`"0.3"`은 `>=0.3.0, <0.4.0`)을 만족하는 **가장 높은 버전**을 고르고 `rust-version`은 보지 않습니다. 1.84.0부터는 설정 `resolver.incompatible-rust-versions = "fallback"`으로 아래 동작을 미리 켤 수 있었습니다.

**2024에서는.** `edition = "2024"`는 `resolver = "3"`을 뜻하고, 리졸버 3은 그 설정의 기본값을 `fallback`으로 바꿉니다. `fallback`은 버전 요구 사항을 만족하는 버전 가운데 **`rust-version`이 내 패키지의 `rust-version` 이하인 버전**을 먼저 고르고, 그런 버전이 하나도 없을 때만 맞지 않는 버전을 고릅니다(에러를 내지 않습니다). Cargo Book의 설명입니다.

[`projects/msrv_resolver`](projects/msrv_resolver/Cargo.toml)는 의존성이 하나(`smol_str = "0.3"`)인 2024 에디션 패키지이고, `rust-version = "1.85"`입니다. 1.85는 2024 에디션이 안정화된 버전이라 2024 에디션 패키지가 적을 수 있는 가장 낮은 값입니다(더 낮추면 Cargo가 `` rust-version 1.80 is incompatible with the version (1.85.0) required by the specified edition (2024) ``로 거절합니다). **작성 시점(2026-11-19)에** crates.io의 `smol_str` 0.3.x는 0.3.4부터 `rust-version = "1.89"`이고, 그 앞에서 고를 수 있는 가장 높은 버전은 `rust-version`이 없는 0.3.2입니다(0.3.3은 게시가 취소(yank)되어 새로 고르지 않습니다). `Cargo.lock`을 지우고 다시 만들면 다음과 같습니다.

```text
$ cargo generate-lockfile                 # edition = "2024" (리졸버 3)
    Updating crates.io index
     Locking 11 packages to highest Rust 1.85 compatible versions
      Adding smol_str v0.3.2 (available: v0.3.7, requires Rust 1.89)
```

`[package]`에 `resolver = "2"`를 적거나 `edition = "2021"`로 바꾸면 결과가 바뀝니다.

```text
$ cargo generate-lockfile                 # resolver = "2"
    Updating crates.io index
     Locking 10 packages to highest compatible versions
$ cargo tree
msrv_resolver v0.1.0 (…/msrv_resolver)
└── smol_str v0.3.7
```

- **고른 이유를 알려 줍니다.** `(available: v0.3.7, requires Rust 1.89)`는 "더 높은 버전이 있지만 `rust-version`이 맞지 않아 고르지 않았다"는 뜻입니다.
- **`rust-version`이 없는 버전은 맞는 것으로 봅니다.** 0.3.2가 골라진 이유입니다.
- **패키지에 `rust-version`이 없으면 지금 쓰는 rustc 버전이 기준입니다.** `rust-version` 줄을 지우자 `Locking 10 packages to highest Rust 1.100.0 compatible versions`가 나오고 0.3.7이 골라졌습니다.
- **`Cargo.lock`이 실제 버전을 고정합니다.** 리졸버는 `Cargo.lock`을 만들거나 갱신할 때만 버전을 고릅니다. 저장소에 함께 넣은 `Cargo.lock`에는 `smol_str` 0.3.2가 적혀 있으므로, 나중에 0.3.8이 나와도 `cargo build`는 0.3.2를 씁니다. 다른 사람이 같은 커밋을 빌드해도 같은 버전입니다. 버전이 다시 골라지는 것은 `cargo update`를 실행하거나 `Cargo.toml`의 요구 사항을 바꿨을 때입니다. `Cargo.lock`에는 켜지 않은 선택적 의존성(optional dependency)도 기록되므로 "11 packages"이지만, 실제로 컴파일되는 것은 `cargo tree`에 나온 `smol_str` 하나입니다.
- **끄는 방법.** 한 번만 무시하려면 `--ignore-rust-version`을, 계속 끄려면 설정 `resolver.incompatible-rust-versions = "allow"`(`.cargo/config.toml`의 `[resolver]` 표, 또는 환경 변수 `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=allow`)를 씁니다. 세 가지 모두 0.3.7을 고르는 것을 확인했습니다.

```text
$ cargo update --ignore-rust-version
     Locking 1 package to highest compatible version
    Removing serde v1.0.229
    Updating smol_str v0.3.2 -> v0.3.7
$ cargo update
     Locking 2 packages to highest Rust 1.85 compatible versions
      Adding serde v1.0.229
 Downgrading smol_str v0.3.7 -> v0.3.2 (available: v0.3.7, requires Rust 1.89)
```

(`Updating crates.io index` 줄은 뺐습니다.) 두 번째 명령처럼, 리졸버 3에서 그냥 `cargo update`를 실행하면 버전을 **낮추기도** 합니다. `cargo add`도 `rust-version`을 봅니다. `rust-version = "1.85"`인 임시 패키지 `a`에서 `cargo add smol_str`을 실행하자 `` warning: ignoring smol_str@0.3.7 (which requires rustc 1.89) to maintain a's rust-version of 1.85 ``를 출력하고 `smol_str = "0.3.2"`를 적었습니다.

**가상 매니페스트에는 직접 적어야 합니다.** 리졸버는 워크스페이스 전체에 하나이고 루트 패키지의 설정만 씁니다. `[package]`가 없는 가상 매니페스트는 에디션이 없으므로, 멤버가 모두 2024 에디션이어도 `resolver`를 적지 않으면 리졸버 1이 쓰입니다([ch1](../ch1-editions-overview/README.md) 2.7절, [ch3](../ch3-rust-2021/README.md) 2.3절). 위 패키지를 가상 워크스페이스의 멤버로 옮겨 확인하자 `` warning: virtual workspace defaulting to `resolver = "1"` despite one or more workspace members being on edition 2024 which implies `resolver = "3"` ``와 함께 0.3.7이 골라졌고, `[workspace]`에 `resolver = "3"`을 적자 0.3.2가 골라졌습니다. 이 저장소의 루트 `Cargo.toml`과 `projects/ecosystem_compare`가 `resolver = "3"`을 적은 이유입니다.

**왜 에디션이 필요했나.** 코드가 컴파일되지 않게 되는 변경은 아니지만, 같은 `Cargo.toml`에서 고르는 의존성 버전이 바뀝니다. 지금까지 최신 버전을 받던 프로젝트가 갑자기 낮은 버전을 받게 되므로 기본값을 바꾸는 시점을 에디션에 맞췄습니다. 자동 마이그레이션은 없고, Edition Guide는 CI에서 최신 의존성으로도 검사하기를 권합니다. 의존성 해석의 자세한 규칙은 [Part 3 ch2](../../part3-cargo/ch2-dependencies/README.md)에서 다룹니다.

### 2.7 Cargo 매니페스트 정리: 키 이름과 상속 `default-features`

**테이블·키 이름 일관성.** `Cargo.toml`에는 같은 것을 두 가지로 적는 이름이 있었습니다. 2024 에디션은 밑줄 이름과 `[project]`를 받아들이지 않습니다. 2021과 2024 매니페스트로 각각 실험한 결과입니다(`cargo check`, 2024는 첫 에러에서 멈추므로 키마다 따로 확인했습니다).

| 옛 이름 → 지금 이름 | 2021 에디션 (경고) | 2024 에디션 (에러) |
| --- | --- | --- |
| `[project]` → `[package]` | `` `[project]` is deprecated in favor of `[package]` `` | `` `[project]` is not supported as of the 2024 Edition, please use `[package]` `` |
| `dev_dependencies` → `dev-dependencies` | `` `dev_dependencies` is deprecated in favor of `dev-dependencies` and will not work in the 2024 edition `` | `` `dev_dependencies` is unsupported as of the 2024 edition; instead use `dev-dependencies` `` |
| `build_dependencies` → `build-dependencies` | (위와 같은 모양) | (위와 같은 모양) |
| `default_features` → `default-features` | (위와 같은 모양, 끝에 ``(in the `helper` dependency)``) | (위와 같은 모양) |
| `crate_type` → `crate-type` | (위와 같은 모양, 끝에 ``(in the `app` library target)``) | (위와 같은 모양) |
| `proc_macro` → `proc-macro` | (위와 같은 모양) | (위와 같은 모양) |

2024의 에러는 `` error: failed to parse manifest at `…/Cargo.toml` `` 다음 `Caused by:` 아래에 나옵니다. 2021에서 두 이름을 함께 쓰면 `` `dev_dependencies` is redundant with `dev-dependencies`, preferring `dev-dependencies` in the `app` package ``처럼 하이픈 이름을 고릅니다. 고치는 방법은 이름을 하이픈 쪽으로 바꾸는 것뿐이고, 2021에서도 그대로 동작합니다.

**상속 `default-features`.** 워크스페이스는 `[workspace.dependencies]`에 의존성을 한 번 적고, 멤버가 `helper = { workspace = true }`로 물려받게 할 수 있습니다([Part 3 ch4](../../part3-cargo/ch4-workspaces/README.md)). 멤버가 물려받으면서 `default-features = false`를 적었을 때의 동작이 에디션과 Cargo 버전에 따라 다릅니다. 기본 기능 `fancy`를 가진 `helper`를 워크스페이스에 `helper = { path = "helper" }`로 적고, 멤버 `app`이 `helper = { workspace = true, default-features = false }`로 물려받아 `cfg!(feature = "fancy")`를 출력해 확인했습니다.

| `app`의 에디션 | cargo 1.100.0 | cargo 1.98.1 |
| --- | --- | --- |
| 2021 | 기본 기능이 **켜짐**. `` `default-features` is ignored for helper, since `default-features` was not specified for `workspace.dependencies.helper`; overriding workspace `default-features` to false requires Rust 1.99+ and the 2024 edition `` 경고 | 켜짐. 경고 문구의 끝이 `this could become a hard error in the future` |
| 2024 | 기본 기능이 **꺼짐**, 경고 없음 | 에러 `` `default-features = false` cannot override workspace's `default-features` `` |

1.85~1.98에서 2024 에디션은 이 조합을 거부했고, 1.99부터는 멤버의 `default-features = false`가 워크스페이스의 설정을 덮어씁니다. 2021 코드에서 "무시되던" 설정이 2024에서 갑자기 효과를 내는 것이므로, 옮길 때 그 줄이 정말 필요한지 확인합니다. `cargo fix --edition`은 이 줄을 지워서 예전 동작을 유지합니다(Edition Guide). 또 기능은 함께 빌드하는 멤버끼리 합쳐집니다. 기본 기능을 켜는 다른 멤버와 `cargo build --workspace`로 함께 빌드하자 `app`에서도 켜졌고, `cargo run -p app`으로 따로 빌드하면 꺼졌습니다.

### 2.8 rustdoc: 문서 테스트 병합

**2021에서는.** rustdoc은 문서 테스트(doc test)마다 코드 블록을 `fn main() { … }`으로 감싸 **실행 파일 하나씩** 컴파일하고 실행합니다. 문서 테스트가 100개면 100번 컴파일하므로 느립니다.

**2024에서는.** 크레이트 전체가 2024 에디션이면 rustdoc은 문서 테스트들을 **한 바이너리로 묶어** 한 번 컴파일합니다(문서 테스트 병합, merged doctests). 그래도 **실행은 테스트마다 따로** 합니다. Edition Guide도 "테스트는 여전히 독립된 프로세스에서 실행되므로 전역 static 같은 전역 상태도 그대로 동작한다"고 설명합니다. [`ecosystem_compare`의 `e2024/src/lib.rs`](projects/ecosystem_compare/e2024/src/lib.rs)에서 확인할 수 있습니다. 전역 `static` 카운터를 올리는 `issue_ticket()`의 두 문서 테스트가 모두 "처음 발급한 번호는 0"이라고 단언하는데, 두 에디션에서 모두 통과합니다. 같은 프로세스에서 실행되었다면 하나는 1을 받았을 것입니다.

같은 내용의 라이브러리를 두 에디션으로 두고 `cargo test --doc`을 실행한 결과입니다(`-q`, 3절에 전체가 있습니다).

```text
$ cargo test -q --doc -p e2021          $ cargo test -q --doc -p e2024
running 4 tests                         running 2 tests
....                                    ..
test result: ok. 4 passed; …            test result: ok. 2 passed; …

                                        running 2 tests
                                        ..
                                        test result: ok. 2 passed; …

                                        all doctests ran in 0.76s; merged doctests compilation took 0.11s
```

2024 쪽은 구역이 둘입니다. 첫 구역이 한 바이너리로 묶인 테스트(`issue_ticket`의 둘)이고, 둘째 구역이 따로 컴파일한 테스트입니다. 마지막 줄은 병합했을 때만 나오며, 시간은 실행마다 다릅니다. Part 0 ch11의 `adder`에서 본 줄이 이것입니다.

**묶이지 않는 테스트.** 다음은 따로 컴파일되어 둘째 구역으로 갔습니다.

- `compile_fail` 표시가 있는 코드 블록, `edition2021` 같은 에디션 표시가 있는 코드 블록(에디션 표시 블록은 자기 구역을 따로 가졌습니다)
- `standalone_crate` 표시가 있는 코드 블록 — 자동으로 판단하지 못하는 경우에 직접 붙이는 표시입니다

Edition Guide는 이 밖에 `#![feature(...)]` 같은 크레이트 속성, `global_allocator` 같은 전역 속성, `$crate`를 쓰는 매크로 정의도 묶지 않는다고 적고 있습니다. `#![allow(unused)]`가 있는 블록은 이 툴체인에서 그대로 묶였습니다.

**묶이면 달라지는 것.** 코드가 놓이는 파일과 줄 번호, 모듈 경로가 바뀝니다. `e2024/src/lib.rs`의 크레이트 문서에는 `std::panic::Location::caller()`의 줄 번호가 3이라고 단언하는 테스트가 있습니다. 따로 컴파일하면 rustdoc이 앞에 붙이는 줄들 다음인 3번째 줄에서 실행되지만, 이 블록에서 `standalone_crate`를 지우고 실행하면 이렇게 실패합니다(임시 디렉터리 경로는 줄였습니다).

```text
---- e2024/src/lib.rs - (line 9) stdout ----
Test executable failed (exit status: 101).

stderr:

thread 'main' (…) panicked at <임시 디렉터리>/doctest_bundle_2024.rs:7:1:
assertion `left == right` failed
  left: 6
 right: 3
```

`Location`이 가리킨 곳은 rustdoc이 만든 묶음 파일 `doctest_bundle_2024.rs`의 6번째 줄이었고, 패닉은 그다음 줄(7번째)의 단언에서 났습니다. `module_path!()`도 따로 컴파일할 때는 `rust_out`이었지만 묶였을 때는 `doctest_bundle_2024::__doctest_2` 같은 값이었습니다. Edition Guide는 `std::any::type_name`의 결과도 달라진다고 경고합니다.

**조용히 따로 컴파일되는 경우.** 묶음 바이너리의 컴파일이 실패하면 rustdoc은 **아무 메시지 없이** 모든 문서 테스트를 따로 컴파일합니다. `e2024`의 `USAGE.md`에 있는 문서 테스트(2.9절)에서 `standalone_crate`를 지우면 테스트는 모두 통과하지만, 구역이 `running 4 tests` 하나로 합쳐지고 `merged doctests compilation took 0.02s`처럼 시간이 아주 짧아졌습니다. 묶음 컴파일이 이 테스트 때문에 실패해서 병합의 이득이 사라진 것입니다. 이 테스트에만 `standalone_crate`를 붙이자 나머지가 다시 묶였습니다. 또 `cargo test --doc -- --show-output`처럼 테스트 옵션을 주면 이 툴체인에서는 병합하지 않고 구역 하나로 실행되었습니다(`--test-threads=1`, `--no-capture`는 그대로 병합).

**왜 에디션이 필요했나.** 병합은 컴파일 시간을 크게 줄이지만, 위처럼 묶이면 동작이 달라지는 문서 테스트가 드물게 있습니다. 그래서 크레이트가 2024 에디션을 고를 때만 병합합니다. 2021 크레이트에서 `edition2024` 표시를 붙여도 병합되지 않습니다(Edition Guide).

**옮길 때.** 자동 수정은 없습니다. 에디션을 올린 뒤 `cargo test --doc`을 실행해 실패하는 테스트를 찾고, 고칠 수 없으면 `standalone_crate`를 붙입니다. 구역이 하나뿐이고 병합 시간이 아주 짧다면 묶음 컴파일이 실패한 것이므로 원인이 된 테스트를 찾아 표시합니다. 2021 크레이트에 `standalone_crate`를 미리 붙여 두어도 경고 없이 받아들여졌습니다. 문서 테스트를 쓰는 방법 자체는 Part 4 `ch3-doc-tests`에서 다룹니다.

### 2.9 rustdoc: 중첩 `include!`의 경로

**상황.** README 같은 Markdown 파일을 `#[doc = include_str!("../USAGE.md")]`로 가져와 문서로 쓰면, 그 안의 코드 블록도 문서 테스트가 됩니다. 그 문서 테스트가 다시 `include_str!`, `include_bytes!`, `include!`로 파일을 읽을 때 상대 경로의 기준이 에디션마다 다릅니다.

| | 경로의 기준 | `ecosystem_compare`에서 쓴 경로 | 테스트 이름 |
| --- | --- | --- | --- |
| 2021 | `include_str!("../USAGE.md")`를 적은 **소스 파일**(`src/lib.rs`) | `include_str!("../data/greeting.txt")` | `e2021/src/lib.rs - greeting (line 42)` |
| 2024 | 문서 테스트가 들어 있는 **Markdown 파일** | `include_str!("data/greeting.txt")` | `e2024/src/../USAGE.md - greeting (line 10)` |

2021에서는 Markdown 파일의 위치 정보가 문서 테스트에 전달되지 않아서 소스 파일 기준으로 읽었고, 테스트 이름과 에러 위치도 `src/lib.rs`를 가리켰습니다. 2024에서는 위치 정보가 전달되어 Markdown 파일 기준이 됩니다. 2021의 경로를 그대로 둔 채 2024로 옮기면 이렇게 실패합니다.

```text
error: couldn't read `e2024/src/../../data/greeting.txt`: No such file or directory (os error 2)
  --> e2024/src/../USAGE.md:12:13
   |
12 | let plain = include_str!("../data/greeting.txt");
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: there is a file with the same name in a different directory
   |
12 - let plain = include_str!("../data/greeting.txt");
12 + let plain = include_str!("data/greeting.txt");
```

자동 수정은 없지만 `help:`가 고칠 경로를 알려 줍니다. Markdown 파일이 `src/`와 같은 깊이의 디렉터리(예: `docs/`)에 있으면 `../`로 시작하는 경로가 두 기준에서 우연히 같은 곳을 가리켜 차이가 드러나지 않았습니다. 크레이트 루트의 README를 가져올 때처럼 깊이가 다를 때 문제가 됩니다.

### 2.10 rustfmt: 스타일 에디션

**에디션과 스타일 에디션은 다른 설정입니다.** rustfmt에게 에디션은 두 가지 일을 합니다.

- `edition`: 소스를 **어느 에디션의 문법으로 읽을지**. 2015로 읽으면 `async`는 이름이고 let 체인은 문법 에러입니다.
- `style_edition`: 읽은 코드를 **어느 판의 Rust 스타일 가이드로 배치할지**. 2024 스타일이 새로 나왔고, 2015·2018·2021 스타일은 서로 같습니다(같은 입력의 결과가 똑같은 것을 확인했습니다).

`style_edition`을 정하지 않으면 `edition`과 같은 값을 씁니다. 정하는 곳은 다음과 같습니다.

| 정하는 곳 | `edition` | `style_edition` |
| --- | --- | --- |
| `cargo fmt` | 크레이트마다 `Cargo.toml`의 `edition`을 `--edition`으로 넘김 | 따로 정하지 않으면 `edition`을 따름 |
| 명령줄 | `rustfmt --edition 2024` | `rustfmt --style-edition 2024` |
| 설정 파일 `rustfmt.toml`(또는 `.rustfmt.toml`) | `edition = "2024"` | `style_edition = "2024"` |
| 아무것도 없을 때 (`rustfmt`를 직접 실행) | 2015 | 2015 |

[`04_rustfmt_style_edition.rs`](examples/04_rustfmt_style_edition.rs)의 2번 구역은 2015 에디션에서만 읽히는 코드(`let async = …`)를 여러 설정으로 서식해 이 표를 확인합니다.

```text
  rustfmt                                읽기 성공 → use std::num::{NonZeroU16, NonZeroU8};
  rustfmt --style-edition 2024           읽기 성공 → use std::num::{NonZeroU8, NonZeroU16};
  rustfmt --edition 2024                 읽기 실패 → error: expected identifier, found keyword `async`
  rustfmt.toml: style_edition = "2024"   읽기 성공 → use std::num::{NonZeroU8, NonZeroU16};
  rustfmt.toml: edition = "2024"         읽기 실패 → error: expected identifier, found keyword `async`
```

`style_edition`은 서식만 바꾸고 읽는 문법은 그대로 2015라서 `async`를 이름으로 읽었습니다. `edition`을 2024로 정하면 읽는 문법과 서식이 함께 2024가 됩니다. `--edition 2024 --style-edition 2021`처럼 둘을 따로 정할 수도 있고, 예제 04의 1번 구역이 그렇게 실행합니다.

**2021 스타일과 2024 스타일의 차이.** 예제 04의 1번 구역 출력에서 확인합니다(3절에 전체가 있습니다).

| 변경 (Edition Guide 절) | 2021 스타일 | 2024 스타일 |
| --- | --- | --- |
| 버전 정렬 (5.5.4) | `{self, stdout, Write}`, `{NonZeroU16, NonZeroU64, NonZeroU8}` | `{self, Write, stdout}`, `{NonZeroU8, NonZeroU16, NonZeroU64}` |
| raw 식별자 정렬 (5.5.3) | `jobs::blocking`, `jobs::queue`, `jobs::r#async` 순서 | `jobs::r#async`, `jobs::blocking`, `jobs::queue` 순서 |
| 뒤따르는 주석 (5.5.2) | 다음 줄 주석을 앞 줄 주석 위치까지 들여씀 | 들여쓰지 않음 |
| `loop` 하나뿐인 클로저 (5.5.2) | `\|\| loop { … }` | `\|\| { loop { … } }` |
| 마지막 표현식의 짧은 `if`/`else` (5.5.2) | 여러 줄 | `if total > 5 { "큼" } else { "작음" }` 한 줄 |
| 중첩 튜플 접근 (5.5.2) | `pair.0 .0` | `pair.0.0` |

버전 정렬은 이름 속의 숫자를 값의 크기로 비교하고(`U8`이 `U16`보다 앞), 대문자로 시작하는 이름과 소문자로 시작하는 이름을 따로 모으지 않습니다. raw 식별자 정렬은 `r#async`를 `r#`를 뗀 `async`로 보고 정렬합니다. Edition Guide 5.5.2절에는 이 밖에도 `where` 절의 빈 줄 제거, `impl` 제네릭의 들여쓰기 등 여러 서식 수정이 있습니다. `where` 절의 빈 줄이 2024 스타일에서 지워지는 것도 확인했습니다.

**왜 에디션이 필요했나.** rustfmt에는 "새 버전의 rustfmt가 예전 버전이 서식한 결과를 바꾸지 않는다"는 안정성 약속이 있습니다. 툴체인을 올릴 때마다 서식 변경 때문에 diff가 쏟아지는 일을 막기 위해서입니다. 그 대신 스타일 가이드를 고칠 수 없게 되어, 위의 `pair.0 .0` 같은 오래된 버그도 남아 있었습니다. 스타일 에디션은 이 약속을 지키면서 스타일을 바꿀 수 있게 한 장치입니다. 고른 사람만 새 스타일을 받습니다.

**`ecosystem_compare`에서 보기.** 두 패키지의 `src/lib.rs`는 따로 있는 파일이라, `greeting` 함수의 본문이 각자의 스타일로 서식되어 있습니다. 두 패키지에서 `cargo fmt --check`가 모두 통과합니다.

```text
// e2021/src/lib.rs (2021 스타일)         // e2024/src/lib.rs (2024 스타일)
pub fn greeting(formal: bool) -> … {      pub fn greeting(formal: bool) -> … {
    if formal {                               if formal { "안녕하세요" } else { "안녕" }
        "안녕하세요"                          }
    } else {
        "안녕"
    }
}
```

반면 `shared/main.rs`는 두 패키지가 함께 쓰는 파일입니다. 2.2절의 `cargo fmt -v` 출력처럼 `-p` 없이 실행하면 멤버 목록에서 먼저 나오는 `e2021` 쪽으로 **한 번만** 서식하고, `cargo fmt -p e2024`는 같은 파일을 2024 스타일로 서식합니다. 그래서 이 파일에는 두 스타일의 결과가 같은 코드만 썼습니다(연습 3).

**옮길 때.** 에디션을 올린 뒤 `cargo fmt`를 한 번 실행하면 서식이 2024 스타일로 바뀝니다. 서식 변경만 담은 커밋을 따로 만들면 검토하기 쉽습니다. 에디터의 "저장할 때 서식"은 `rustfmt`를 직접 실행하는 경우가 많아, `Cargo.toml`의 에디션을 모르고 2015 스타일로 서식할 수 있습니다. Edition Guide는 그래서 프로젝트에 `style_edition = "2024"`를 적은 `rustfmt.toml`을 두기를 권합니다. 위 표에서 보듯 이 설정은 서식만 정하므로, 2024 문법을 쓰는 코드라면 `edition = "2024"`도 함께 적어야 에디터가 읽을 수 있습니다.

### 2.11 이 챕터에서 다루지 않는 것

| 주제 | 다루는 곳 |
| --- | --- |
| 2024 에디션의 언어 변경(RPIT 캡처, 임시값 범위, `unsafe extern` 등) | [Part 2 ch4](../ch4-rust-2024-language/README.md) |
| 2021의 프렐류드 추가, 기능 리졸버 2, 배열 `IntoIterator` | [Part 2 ch3](../ch3-rust-2021/README.md) |
| `cargo fix --edition`으로 이 챕터의 변경을 자동으로 고치는 절차 | [Part 2 ch6](../ch6-migration-practice/README.md) |
| 의존성 해석, 매니페스트와 타깃, 워크스페이스 상속의 상세 | [Part 3 ch2](../../part3-cargo/ch2-dependencies/README.md), [ch3](../../part3-cargo/ch3-manifest-targets/README.md), [ch4](../../part3-cargo/ch4-workspaces/README.md) |
| 기능(feature)의 상세 | Part 3 `ch5-features` |
| 문서 주석과 문서 테스트 작성법, rustdoc 상세 | Part 4 (`ch3-doc-tests` 등) |
| unsafe와 비동기 시그널 안전성의 깊은 내용 | Part 10 |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_prelude_future.rs`](examples/01_prelude_future.rs) | `use` 없이 쓰는 `Future`/`IntoFuture`, 이름이 겹치는 사용자 메서드의 2024 동작과 고치는 법 | 1, 2 |
| [`02_box_slice_into_iter.rs`](examples/02_box_slice_into_iter.rs) | `Box<[T]>`의 `.into_iter()`가 값을 내놓는 것, `.iter()`, 메서드 문법이 아닌 모양 | 2 |
| [`03_env_unsafe.rs`](examples/03_env_unsafe.rs) | `set_var`/`remove_var`의 unsafe 블록과 `SAFETY`, `Command::env`, 설정을 인자로, `pre_exec` | 2, 3 |
| [`04_rustfmt_style_edition.rs`](examples/04_rustfmt_style_edition.rs) | 같은 입력의 2021/2024 스타일 서식, `edition`과 `style_edition`을 정하는 곳 | 1, 5 |
| [`projects/ecosystem_compare`](projects/ecosystem_compare/Cargo.toml) | 같은 소스의 2021/2024 실행 결과, 문서 테스트 병합, 중첩 `include_str!`, 에디션별 서식 | 1, 2, 5 |
| [`projects/msrv_resolver`](projects/msrv_resolver/Cargo.toml) | 리졸버 3이 `rust-version`에 맞는 의존성 버전을 고르는 것 | 4 |

예제 명령은 저장소 루트에서 실행합니다. 네 예제 모두 종료 코드 0으로 곧바로 끝납니다.

### 01_prelude_future — 프렐류드의 `Future`와 `IntoFuture`

```sh
cargo run -p p2-ch5-rust-2024-ecosystem --example 01_prelude_future
```

```text
1. Future와 IntoFuture를 use 없이 씁니다
    jobs()                 → [Ready(42), Ready(7)]
    Order 빌더를 .await    → Ready("라테 (샷 2개)")
2. &self로 받는 poll: 메서드 문법은 Future::poll을 부릅니다
    async 본문 실행
    fut.as_mut().poll(&mut cx)     = Ready(())
    Peek::poll 실행
    Peek::poll(&fut.as_mut(), ..)  = Pending
3. self로 받는 into_future: 메서드 문법은 모호해서 에러(E0034)입니다
    IntoTask::into_future(..)   → 작업으로 포장됨, Ready(7)
    IntoFuture::into_future(..) → Ready(7)
```

- 실행기 없이 결과를 보려고, 아무것도 하지 않는 웨이커(`Waker::noop()`)로 한 번만 폴링하는 `poll_once`를 썼습니다([Part 1 ch13](../../part1-standard-lib/ch13-async-primitives/README.md) 2.2절). 이 예제의 퓨처는 모두 첫 폴링에서 끝납니다.
- **2번 구역**: 같은 퓨처를 메서드 문법으로 부르면 `Future::poll`, 트레이트 이름을 적으면 `Peek::poll`이 불립니다. 2021 에디션에서는 앞의 것도 `Peek::poll`이었습니다(`ecosystem_compare`).

### 02_box_slice_into_iter — `Box<[T]>`의 `IntoIterator`

```sh
cargo run -p p2-ch5-rust-2024-ecosystem --example 02_box_slice_into_iter
```

```text
1. boxed.into_iter()는 값을 내놓습니다 (2024 에디션)
    반복자: alloc::vec::into_iter::IntoIter<u32>
    첫 원소: Some(70), core::option::Option<u32>
2. 값으로 꺼내야만 되는 일: String을 복제 없이 옮기기
    ["가람", "나래", "다온"]
    ["가람님", "나래님", "다온님"]
3. 참조가 필요하면 .iter() — 모든 에디션에서 &T
    합계 245, core::option::Option<&u32>
    boxed는 그대로: [70, 85, 90]
4. 메서드 문법이 아닌 모양은 1.80부터 모든 에디션에서 값
    for x in boxed: u32
    IntoIterator::into_iter(boxed): core::option::Option<u32>
```

- **2번 구역**: `Box<[String]>`에서 `String`을 복제 없이 꺼내 `Vec<String>`으로 옮겨 담습니다. 2021 에디션에서는 같은 줄이 E0277입니다(4.3절).
- 2021 에디션에서의 결과는 2.4절의 표에 있습니다.

### 03_env_unsafe — 새로 unsafe가 된 함수와 옮기는 길

```sh
cargo run -p p2-ch5-rust-2024-ecosystem --example 03_env_unsafe
```

```text
1. set_var와 remove_var는 unsafe 블록 안에서 부릅니다
    set_var 뒤:    Ok("info")
    remove_var 뒤: Err(NotPresent)
2. 자식에게만 넘기기: Command::env와 env_remove는 unsafe가 아닙니다
    자식이 본 P2CH5_LEVEL    = Ok("debug")
    자식이 본 CARGO_PKG_NAME = Err(NotPresent)
    부모가 본 P2CH5_LEVEL    = Err(NotPresent)
    부모가 본 CARGO_PKG_NAME = Ok("p2-ch5-rust-2024-ecosystem")
3. 설정은 한 번 읽어 인자로 넘깁니다
    환경에서 읽은 값: Info
    parse_level(Some("quiet")) = Quiet
    parse_level(Some("debug")) = Debug
    parse_level(Some("엉뚱한 값")) = Info
4. before_exec 대신 pre_exec (유닉스)
    exec 전에 멈춤: Operation not permitted (os error 1) (PermissionDenied)
```

- **1번 구역**: `set_var`는 `main`의 첫머리, 스레드를 만들기 전에 부르고 `// SAFETY:` 주석에 그 근거를 적었습니다.
- **2번 구역**: 예제가 자기 자신을 `child` 인자와 함께 자식 프로세스로 다시 실행합니다. 자식에게는 `P2CH5_LEVEL`이 있고 `CARGO_PKG_NAME`이 없지만, 부모의 환경은 그대로입니다. `CARGO_PKG_NAME`은 `cargo run`이 넣어 주는 변수라서, 실행 파일을 직접 실행하면 마지막 줄이 `Err(NotPresent)`가 됩니다.
- **4번 구역**: `pre_exec`의 클로저는 자식 쪽에서 `exec` 직전에 실행됩니다. 클로저가 `Err`를 돌려주면 새 프로그램은 실행되지 않고 그 에러가 부모의 `output()` 결과로 돌아옵니다. 에러 문구(`Operation not permitted`)는 OS가 정하는 것이라 플랫폼마다 다를 수 있습니다. 유닉스 계열이 아니면 "건너뜁니다"를 출력합니다.

### 04_rustfmt_style_edition — 같은 입력, 다른 스타일

```sh
cargo run -p p2-ch5-rust-2024-ecosystem --example 04_rustfmt_style_edition
```

출력의 앞부분과 끝부분입니다. 1번 구역의 나머지는 2.10절의 표에 정리했습니다.

```text
사용한 rustfmt: rustfmt 1.11.0-stable (0ef1bc3860 2026-11-09)
1. 같은 입력, 다른 스타일 에디션 (--edition 2024 --style-edition 2021 / 2024)
[버전 정렬: 숫자는 값의 크기로, 대문자 이름과 소문자 이름을 섞어서]
  (2021 스타일)
    use std::io::{self, stdout, Write};
    use std::num::{NonZeroU16, NonZeroU64, NonZeroU8};
  (2024 스타일)
    use std::io::{self, Write, stdout};
    use std::num::{NonZeroU8, NonZeroU16, NonZeroU64};
…
  (2024 스타일)
    fn main() {
        let total = 10; // 합계
        // 다음 줄은 합계와 상관없는 주석
        std::thread::spawn(|| {
            loop {
                println!("작업");
            }
        });
        let size = || {
            if total > 5 { "큼" } else { "작음" }
        };
        let pair = ((1, 2), 3);
        let _ = (size(), pair.0.0);
    }
2. 무엇이 에디션과 스타일 에디션을 정하는가 (입력: let async = ...; 와 use 한 줄)
  rustfmt                                읽기 성공 → use std::num::{NonZeroU16, NonZeroU8};
  (2.10절의 다섯 줄)
```

- 예제는 코드 조각을 `rustfmt`의 표준 입력으로 넘기고 표준 출력을 받습니다. 파일을 고치지 않습니다.
- 다른 `rustfmt.toml`의 영향을 받지 않도록 임시 디렉터리(`rust-p2-ch5-fmt-<프로세스 ID>`)에 설정 파일을 만들고 그 디렉터리에서 `rustfmt`를 실행합니다. `rustfmt`는 현재 디렉터리부터 위로 올라가며 설정 파일을 찾습니다. 끝나면 임시 디렉터리를 지웁니다.
- 첫 줄은 툴체인마다 다릅니다. 환경 변수 `RUSTFMT`가 있으면 그 경로의 `rustfmt`를 쓰고, 실행하지 못하면 설치 방법을 출력하고 끝납니다.

### projects/ecosystem_compare — 같은 코드, 두 에디션, 네 도구

```text
projects/ecosystem_compare/
├── Cargo.toml        # 가상 매니페스트: resolver = "3", members = ["e2021", "e2024"]
├── shared/main.rs    # 두 패키지의 실행 파일(ecosystem_2021, ecosystem_2024)이 함께 쓰는 소스
├── e2021/            # edition = "2021": src/lib.rs(문서 테스트), USAGE.md, data/greeting.txt
└── e2024/            # edition = "2024": e2021과 같은 내용을 2024 에디션에 맞게 고친 것
```

```sh
cd part2-editions/ch5-rust-2024-ecosystem/projects/ecosystem_compare
cargo run -q --bin ecosystem_2021
cargo run -q --bin ecosystem_2024
```

```text
[e2021]                                         [e2024]
1. 프렐류드의 Future와 이름이 겹치는 메서드     1. 프렐류드의 Future와 이름이 겹치는 메서드
    → Peek::poll 실행                               → async 본문 실행
    fut.as_mut().poll(&mut cx) = Pending            fut.as_mut().poll(&mut cx) = Ready(())
    → Peek::poll 실행                               → Peek::poll 실행
    Peek::poll(..)             = Pending            Peek::poll(..)             = Pending
2. Box<[T]>에 .into_iter()를 부르면             2. Box<[T]>에 .into_iter()를 부르면
    첫 원소: Some(70), core::option::Option<&u32>   첫 원소: Some(70), core::option::Option<u32>
3. unsafe 블록 안에서 부른 set_var (두 에디션이 같음)
    P2CH5_DEMO = Ok("설정됨")                       P2CH5_DEMO = Ok("설정됨")
```

(두 실행의 출력을 나란히 놓았습니다. 3번 구역의 제목 줄은 둘 다 같습니다.) 둘 다 종료 코드 0입니다.

- **1번 구역**: 같은 줄이 다른 메서드를 부릅니다(2.3절). 트레이트 이름을 적은 대조군 `Peek::poll(..)`은 두 에디션이 같습니다.
- **2번 구역**: 같은 줄의 항목 타입이 다릅니다(2.4절). 2021로 컴파일할 때 이 줄이 일부러 내는 `boxed_slice_into_iter` 경고는 파일 맨 위의 `#![allow(boxed_slice_into_iter)]`로 막았습니다. 2024로 컴파일할 때는 이 린트가 아무것도 찾지 못하므로 `#[expect]`가 아니라 `#[allow]`를 썼습니다.
- **3번 구역**: `unsafe` 블록 안의 `set_var`가 두 에디션에서 모두 경고 없이 컴파일됩니다(2.5절). 2024 에디션 쪽으로 미리 고쳐 둔 코드는 2021에서도 그대로 동작합니다.

문서 테스트는 패키지마다 실행합니다.

```sh
cargo test --doc -p e2021
cargo test --doc -p e2024
```

```text
   Doc-tests e2021

running 4 tests
test e2021/src/lib.rs - issue_ticket (line 21) ... ok
test e2021/src/lib.rs - issue_ticket (line 28) ... ok
test e2021/src/lib.rs - (line 8) ... ok
test e2021/src/lib.rs - greeting (line 42) ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

```text
   Doc-tests e2024

running 2 tests
test e2024/src/lib.rs - issue_ticket (line 30) ... ok
test e2024/src/lib.rs - issue_ticket (line 23) ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test e2024/src/../USAGE.md - greeting (line 10) ... ok
test e2024/src/lib.rs - (line 9) ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

all doctests ran in 0.76s; merged doctests compilation took 0.11s
```

(`Compiling`, `Finished` 줄은 뺐습니다.) 테스트들은 동시에 실행되므로 한 구역 안의 순서는 실행마다 달라지고, 시간도 실행마다 다릅니다. 문서 테스트에 관한 두 패키지의 차이는 세 곳입니다. 2024 쪽의 줄 번호 테스트(`(line 9)`)와 `USAGE.md`의 테스트에는 `standalone_crate`가 붙어 있고(2.8절), `USAGE.md`의 경로가 다릅니다(2.9절). 2024 쪽의 줄 번호가 1~2씩 큰 것은 크레이트 문서의 설명 줄 수가 다르기 때문입니다.

이 프로젝트는 루트 워크스페이스와 분리되어 있어 이 디렉터리에서 다음이 경고 없이 통과합니다. `cargo test`는 문서 테스트까지 실행합니다.

```sh
cargo build
cargo clippy --workspace -- -D warnings
cargo fmt --check
cargo test
```

### projects/msrv_resolver — rust-version을 고려하는 리졸버

```sh
cd part2-editions/ch5-rust-2024-ecosystem/projects/msrv_resolver
cargo run -q
cargo tree
```

```text
"서울": 6바이트, 힙에 할당했나: false
"이 문장은 스물세 바이트보다 훨씬 깁니다": 56바이트, 힙에 할당했나: true
msrv_resolver v0.1.0 (…/msrv_resolver)
└── smol_str v0.3.2
```

- 처음 빌드할 때 crates.io에서 `smol_str`을 내려받습니다. 함께 넣은 `Cargo.lock`이 0.3.2를 고정하므로, 나중에 새 버전이 나와도 이 결과는 같습니다. `cargo update`를 실행하면 그 시점의 crates.io를 보고 다시 고르므로 결과가 달라질 수 있습니다.
- `SmolStr`은 23바이트까지의 문자열을 힙 할당 없이 값 안에 담는 타입입니다. 이 프로젝트에서 볼 것은 실행 결과보다 Cargo가 고른 버전입니다(2.6절).
- `cargo clippy -- -D warnings`, `cargo fmt --check`가 통과합니다. 테스트는 없습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 어떤 에디션에서 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc --edition`을 바꿔 가며 컴파일합니다. 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part2-editions/ch5-rust-2024-ecosystem
rustc --edition 2024 compile_fail/e0133_remove_var_in_2024.rs --out-dir /tmp/rust-p2-ch5   # 에러
rustc --edition 2021 compile_fail/e0133_remove_var_in_2024.rs --out-dir /tmp/rust-p2-ch5   # 통과
```

| 파일 | 실패하는 에디션 | 통과하는 에디션 |
| --- | --- | --- |
| `e0034_prelude_into_future_ambiguous.rs` | 2024 (E0034) | 2015, 2018, 2021 |
| `e0277_box_slice_into_iter_refs.rs` | 2024 (E0277) | 2015, 2018, 2021 (`boxed_slice_into_iter` 경고) |
| `e0277_box_slice_by_value_2021.rs` | 2015, 2018, 2021 (E0277) | 2024 |
| `e0133_remove_var_in_2024.rs` | 2024 (E0133) | 2015, 2018, 2021 (경고 없음) |
| `e0133_before_exec_in_2024.rs` | 2024 (E0133, deprecated 경고) | 2015, 2018, 2021 (deprecated 경고). 유닉스 전용 |

네 에디션 모두로 컴파일해 위 결과를 확인했고, 통과하는 에디션에서는 실행까지 해 보았습니다. 아래 인용은 표준 라이브러리 소스 위치(`/rustc/0ef1bc38…/library/…`)를 가리키는 줄을 생략했습니다.

### 4.1 사용자 트레이트의 `into_future`가 프렐류드의 `IntoFuture`와 부딪힘 — E0034

파일: [`compile_fail/e0034_prelude_into_future_ambiguous.rs`](compile_fail/e0034_prelude_into_future_ambiguous.rs)

```text
error[E0034]: multiple applicable items in scope
  --> compile_fail/e0034_prelude_into_future_ambiguous.rs:28:31
   |
28 |     let (label, _fut) = ready.into_future(); // → 2024 에디션에서 E0034
   |                               ^^^^^^^^^^^ multiple `into_future` found
   |
   = note: candidate #1 is defined in an impl of the trait `IntoFuture` for the type `F`
note: candidate #2 is defined in an impl of the trait `IntoTask` for the type `F`
help: disambiguate the method for candidate #1
   |
28 +     let (label, _fut) = IntoFuture::into_future(ready); // → 2024 에디션에서 E0034
help: disambiguate the method for candidate #2
   |
28 +     let (label, _fut) = IntoTask::into_future(ready); // → 2024 에디션에서 E0034
```

(`note:`와 `help:` 아래의 소스 인용은 줄였습니다.)

- **읽는 법**: 후보 #1은 "모든 퓨처는 자기 자신을 돌려주는 `IntoFuture`"라는 표준 라이브러리의 포괄 구현, 후보 #2는 사용자 트레이트입니다. 둘 다 `self`를 값으로 받아 메서드 탐색의 같은 단계에서 찾아졌습니다.
- **원인**: 2021 에디션에서는 `IntoFuture`가 스코프에 없어 후보가 하나였습니다. 2021로 컴파일해 실행하면 `작업으로 포장됨`을 출력합니다. 파일이 `use std::future::{Future, Ready};`로 `Future`를 직접 가져오는 것은 2021에서도 `impl<F: Future>`를 쓰기 위해서이고, `IntoFuture`는 가져오지 않았습니다.
- **해결**: 원래 뜻대로 `IntoTask::into_future(ready)`라고 씁니다. 이 형태는 모든 에디션에서 같은 메서드를 부릅니다. `help:`의 두 제안 중 무엇을 고를지는 원래 의도가 정합니다.

### 4.2 `Box<[T]>`의 `.into_iter()`가 참조를 내놓는다고 가정함 — E0277

파일: [`compile_fail/e0277_box_slice_into_iter_refs.rs`](compile_fail/e0277_box_slice_into_iter_refs.rs)

```text
error[E0277]: a value of type `Vec<&u32>` cannot be built from an iterator over elements of type `u32`
  --> compile_fail/e0277_box_slice_into_iter_refs.rs:17:49
   |
17 |     let mut top: Vec<&u32> = scores.into_iter().collect(); // → 2024 에디션에서 E0277
   |                                                 ^^^^^^^ value of type `Vec<&u32>` cannot be built from `std::iter::Iterator<Item=u32>`
   |
help: the trait `FromIterator<_>` is not implemented for `Vec<&u32>`
      but trait `FromIterator<&_>` is implemented for it
   = help: for that trait implementation, expected `&u32`, found `u32`
```

- **읽는 법**: `collect`가 만들려는 `Vec<&u32>`는 참조를 받아야 하는데, 반복자가 `u32`를 내놓는다는 뜻입니다. 에러는 `collect`에서 나지만 원인은 그 앞의 `.into_iter()`입니다.
- **원인**: 2021 에디션까지 `scores.into_iter()`는 `&u32`를 내놓았습니다(2.4절). 2021로 컴파일하면 `boxed_slice_into_iter` 경고와 함께 통과하고 `[90, 85]`를 출력합니다.
- **해결**: 참조가 필요하면 `scores.iter()`로 씁니다. 모든 에디션에서 `&u32`이고, `scores`도 그대로 남습니다.

### 4.3 반대로, 2021 크레이트에서 값을 내놓는다고 가정함 — E0277

파일: [`compile_fail/e0277_box_slice_by_value_2021.rs`](compile_fail/e0277_box_slice_by_value_2021.rs). 2024에서는 통과하고 2021에서 실패하는 파일이라 이름 끝에 `_2021`을 붙였습니다.

```sh
rustc --edition 2021 compile_fail/e0277_box_slice_by_value_2021.rs --out-dir /tmp/rust-p2-ch5
```

```text
error[E0277]: a value of type `Vec<String>` cannot be built from an iterator over elements of type `&String`
  --> compile_fail/e0277_box_slice_by_value_2021.rs:18:54
   |
18 |     let mut members: Vec<String> = names.into_iter().collect(); // → 2021 에디션에서 E0277
   |                                                      ^^^^^^^ value of type `Vec<String>` cannot be built from `std::iter::Iterator<Item=&String>`
   |
help: the trait `FromIterator<&String>` is not implemented for `Vec<String>`
      but trait `FromIterator<String>` is implemented for it
```

- **원인**: 2024 에디션에 익숙해진 채 2021 크레이트에 코드를 옮겨 적었습니다. 2021에서는 `names.into_iter()`가 `&String`을 내놓습니다. 2024로 컴파일하면 `["가람", "나래", "다온"]`을 출력합니다.
- **해결**: 2021 크레이트에서 값을 원하면 `IntoIterator::into_iter(names)` 또는 `names.into_vec()`을 씁니다. 크레이트의 에디션을 2024로 올려도 됩니다.

### 4.4 2024 에디션에서 `unsafe` 없이 `env::remove_var` — E0133

파일: [`compile_fail/e0133_remove_var_in_2024.rs`](compile_fail/e0133_remove_var_in_2024.rs)

```text
error[E0133]: call to unsafe function `remove_var` is unsafe and requires unsafe block
  --> compile_fail/e0133_remove_var_in_2024.rs:20:5
   |
20 |     env::remove_var("P2CH5_TOKEN"); // → 2024 에디션에서 E0133
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
```

- **읽는 법**: [Part 1 ch10](../../part1-standard-lib/ch10-env-process-time/README.md) 4.2절의 `set_var`와 같은 에러입니다. 에러 메시지에는 에디션 이야기가 없습니다. 같은 파일을 `--edition 2021`로 컴파일하면 경고도 없이 통과합니다(2.5절).
- **원인**: 자식 프로세스에 비밀 값이 새지 않도록 내 프로세스의 환경에서 지웠습니다.
- **해결**: 목적이 "자식에게 주지 않기"라면 내 환경은 그대로 두고 `Command::new("printenv").arg("P2CH5_TOKEN").env_remove("P2CH5_TOKEN")`처럼 자식 몫만 뺍니다. 내 환경을 꼭 바꿔야 한다면 스레드를 만들기 전에 `unsafe` 블록 안에서 `SAFETY` 주석과 함께 부릅니다(예제 03의 1번 구역).

### 4.5 2024 에디션에서 `before_exec` — E0133과 deprecated 경고

파일: [`compile_fail/e0133_before_exec_in_2024.rs`](compile_fail/e0133_before_exec_in_2024.rs) (유닉스 계열 전용)

```text
warning: use of deprecated method `std::os::unix::process::CommandExt::before_exec`: should be unsafe, use `pre_exec` instead
  --> compile_fail/e0133_before_exec_in_2024.rs:21:10
   |
21 |         .before_exec(|| Ok(())) // → 2024 에디션에서 E0133
   |          ^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

error[E0133]: call to unsafe function `before_exec` is unsafe and requires unsafe block
  --> compile_fail/e0133_before_exec_in_2024.rs:20:18
```

- **읽는 법**: 경고와 에러가 함께 나옵니다. 경고는 1.37.0부터 모든 에디션에서 나오던 것이고("unsafe여야 하니 `pre_exec`를 쓰라"), 에러가 2024 에디션의 변경입니다. 2021로 컴파일하면 경고만 나고 통과합니다.
- **해결**: `unsafe` 블록으로 감싸 `before_exec`를 계속 쓰기보다 `pre_exec`로 바꿉니다. 클로저 안에서는 할당이나 잠금 없이 시스템 호출 정도만 해야 하고, 그 근거를 `SAFETY` 주석에 적습니다(예제 03의 4번 구역). 참고로 `pre_exec`를 `unsafe` 없이 부르면 2021에서는 `requires unsafe function or block`, 2024에서는 `requires unsafe block`으로 E0133의 문구가 조금 다릅니다([ch4](../ch4-rust-2024-language/README.md) 2.12절의 `unsafe_op_in_unsafe_fn`과 관련이 있습니다).

### 4.6 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| 2024 에디션 `Cargo.toml`에 `dev_dependencies`, `[project]` 등 | 매니페스트 에러 (2.7절의 표) | 하이픈 이름과 `[package]`로 |
| 2024 에디션 패키지에 `rust-version = "1.80"` | `` rust-version 1.80 is incompatible with the version (1.85.0) required by the specified edition (2024) `` | 1.85 이상으로 (2.6절) |
| 가상 매니페스트에 `resolver`가 없음 | 리졸버 1이 쓰이고 `rust-version`을 보지 않음, 경고 | `[workspace]`에 `resolver = "3"` (2.6절) |
| 2024 문서 테스트에서 `Location`의 줄 번호나 `module_path!()`를 단언 | 묶이면 값이 달라져 실패 (2.8절) | `standalone_crate`를 붙이거나 단언을 고침 |
| 2024 문서 테스트 하나가 묶음 컴파일을 깨뜨림 | 실패 메시지 없이 모든 문서 테스트를 따로 컴파일(구역 하나, 병합 시간이 아주 짧음) | 원인 테스트에 `standalone_crate` (2.8절) |
| 2024 에디션으로 옮긴 뒤 README 속 문서 테스트의 `include_str!("../…")` | `couldn't read …` (2.9절) | Markdown 파일 기준 경로로 |
| 에디터가 저장할 때 `rustfmt`를 직접 실행 | 2015 에디션으로 읽어 2024 문법에서 에러, 서식도 2015 스타일 | `rustfmt.toml`에 `edition`과 `style_edition` (2.10절) |
| 두 패키지가 함께 쓰는 소스 파일 | `cargo fmt -p e2021`과 `-p e2024`의 결과가 다를 수 있음 | 두 스타일에서 같은 코드로 쓰거나 파일을 나눔 (연습 3) |

## 5. 연습 문제

### 연습 1. 두 에디션에서 같은 결과가 나오게 고치기

[`projects/ecosystem_compare/shared/main.rs`](projects/ecosystem_compare/shared/main.rs)의 1·2번 구역을 고쳐서 `ecosystem_2021`과 `ecosystem_2024`의 출력이 첫 줄(`[e2021]`/`[e2024]`)을 빼고 모두 같아지게 하세요. 1번 구역은 두 에디션 모두 `Peek::poll`이, 2번 구역은 두 에디션 모두 `Option<u32>`가 나와야 합니다. 다 고쳤으면 맨 위의 `#![allow(boxed_slice_into_iter)]`가 아직 필요한지도 확인하세요.

<details>
<summary>힌트</summary>

- 1번 구역: 메서드 문법 대신 `Peek::poll(&fut.as_mut(), &mut cx)`로 씁니다. 2021 에디션에서 `-W rust-2024-compatibility`를 켰을 때 `help:`가 제안한 모양 그대로입니다(2.3절).
- 2번 구역: `IntoIterator::into_iter(scores).next()`로 씁니다. 메서드 문법이 아니라서 1.80 이후 모든 에디션에서 값을 내놓습니다(2.4절의 표).
- 2번 구역을 고치면 `boxed_slice_into_iter`가 찾을 것이 없어지므로 `#![allow(..)]` 줄을 지워도 `cargo clippy --workspace -- -D warnings`가 통과합니다. bash나 zsh라면 `diff <(cargo run -q --bin ecosystem_2021) <(cargo run -q --bin ecosystem_2024)`로 남은 차이(첫 줄)를 볼 수 있습니다.
- 끝나면 이 프로젝트 디렉터리에서 `git checkout -- .`로 되돌립니다.

</details>

### 연습 2. 리졸버가 고르는 버전 예측하기

`projects/msrv_resolver/Cargo.toml`을 아래처럼 바꿔 가며 `cargo update`를 실행하기 전에 `smol_str`의 어느 버전이 골라질지 예측하고, `cargo tree`로 확인하세요. 각 단계가 끝나면 `Cargo.toml`을 원래대로 돌리고 `cargo update`를 한 번 더 실행합니다.

1. `rust-version = "1.89"`
2. `rust-version` 줄을 지움
3. `rust-version = "1.85"`는 그대로 두고 `[package]`에 `resolver = "2"`를 더함

<details>
<summary>힌트</summary>

- 작성 시점의 결과는 셋 다 0.3.7이었습니다. 1번은 `Locking 1 package to highest Rust 1.89 compatible version`, 2번은 지금 쓰는 rustc(`Rust 1.100.0`)가 기준이 되고, 3번은 `rust-version`을 보지 않습니다(2.6절).
- 원래대로 돌린 뒤의 `cargo update`는 0.3.7을 0.3.2로 낮춥니다(`Downgrading`). 끝나면 `git checkout -- Cargo.toml Cargo.lock`으로 되돌려도 됩니다.
- 그 사이에 crates.io에 새 버전이 나왔다면 결과가 다를 수 있습니다. `cargo update --verbose`는 왜 그 버전을 골랐는지 더 자세히 보여 줍니다.

</details>

### 연습 3. 함께 쓰는 파일의 서식

`projects/ecosystem_compare/shared/main.rs`의 `use std::pin::pin;`을 `use std::pin::{Pin, pin};`으로 바꾸고, 1번 구역의 `let mut fut = pin!(async {`를 `let mut fut: Pin<&mut _> = pin!(async {`로 바꾸세요. 그다음 `cargo fmt --check`, `cargo fmt --check -p e2021`, `cargo fmt --check -p e2024`를 차례로 실행하고, 결과가 왜 다른지 설명하세요.

<details>
<summary>힌트</summary>

- 2024 스타일은 버전 정렬로 `{Pin, pin}`, 2021 스타일은 `{pin, Pin}`입니다(2.10절).
- `cargo fmt --check`와 `-p e2021`은 `-use std::pin::{Pin, pin};` / `+use std::pin::{pin, Pin};` diff를 보이며 실패하고, `-p e2024`는 통과했습니다. `-p` 없이 실행하면 같은 파일을 한 번만, 멤버 목록에서 먼저 나오는 `e2021`의 에디션으로 서식하기 때문입니다(`cargo fmt -v`로 확인).
- 이 프로젝트처럼 한 파일을 두 에디션이 함께 쓴다면 두 스타일의 결과가 같은 코드만 쓰거나, 프로젝트에 `rustfmt.toml`을 두고 `style_edition`을 하나로 정합니다. 끝나면 `git checkout -- shared/main.rs`로 되돌립니다.

</details>

### 연습 4. 묶음 컴파일을 깨뜨리는 문서 테스트 찾기

`projects/ecosystem_compare/e2024/USAGE.md`의 코드 블록 첫 줄 ```` ```standalone_crate ````를 ```` ``` ````로 바꾸고 `cargo test --doc -p e2024`를 실행하세요. 모든 테스트가 통과하는데도 무엇이 달라졌는지 출력에서 두 가지를 찾으세요. 그다음 원래대로 돌리고, 이번에는 `e2024/src/lib.rs` 맨 위 코드 블록의 `standalone_crate`를 지워 실행해 보세요.

<details>
<summary>힌트</summary>

- 첫 번째: `running …` 구역이 하나(4개)로 합쳐지고, `merged doctests compilation took`의 시간이 0.02초 정도로 짧아집니다. 묶음 컴파일이 실패해서 rustdoc이 모든 테스트를 따로 컴파일한 것입니다(2.8절).
- 두 번째: 줄 번호 테스트가 `left: 6`, `right: 3`으로 실패하고, 패닉 위치가 `doctest_bundle_2024.rs`입니다. `cargo test --doc -p e2021`에서는 같은 테스트가 표시 없이 통과하는 것과 비교합니다.
- 끝나면 `git checkout -- e2024`로 되돌립니다.

</details>

## 6. 참고 자료

- The Rust Edition Guide (기준 문서)
  - [5. Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html): 2024 에디션의 전체 목차와 안정화 버전(1.85.0).
  - 5.2 Standard library: [Changes to the prelude](https://doc.rust-lang.org/edition-guide/rust-2024/prelude.html)(`MyPoller` 예와 `rust_2024_prelude_collisions`), [Add IntoIterator for Box<[T]>](https://doc.rust-lang.org/edition-guide/rust-2024/intoiterator-box-slice.html), [Unsafe functions](https://doc.rust-lang.org/edition-guide/rust-2024/newly-unsafe-functions.html)(`set_var`, `remove_var`, `before_exec`와 `deprecated_safe_2024`).
  - 5.3 Cargo: [Rust-version aware resolver](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html), [Table and key name consistency](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-table-key-names.html), [Override inherited default-features](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-inherited-default-features.html)(1.99에서 바뀐 동작).
  - 5.4 Rustdoc: [Rustdoc combined tests](https://doc.rust-lang.org/edition-guide/rust-2024/rustdoc-doctests.html)(`standalone_crate`), [Rustdoc nested include! change](https://doc.rust-lang.org/edition-guide/rust-2024/rustdoc-nested-includes.html).
  - 5.5 Rustfmt: [Style edition](https://doc.rust-lang.org/edition-guide/rust-2024/rustfmt-style-edition.html), [Formatting fixes](https://doc.rust-lang.org/edition-guide/rust-2024/rustfmt-formatting-fixes.html), [Raw identifier sorting](https://doc.rust-lang.org/edition-guide/rust-2024/rustfmt-raw-identifier-sorting.html), [Version sorting](https://doc.rust-lang.org/edition-guide/rust-2024/rustfmt-version-sorting.html).
- The Cargo Book: [Dependency Resolution의 Rust version 절](https://doc.rust-lang.org/cargo/reference/resolver.html#rust-version)과 [Resolver versions 절](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions), [`resolver.incompatible-rust-versions` 설정](https://doc.rust-lang.org/cargo/reference/config.html#resolverincompatible-rust-versions), [The rust-version field](https://doc.rust-lang.org/cargo/reference/rust-version.html)(`cargo add`도 `rust-version`을 본다는 설명).
- The rustdoc book: [Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html).
- 표준 라이브러리 문서: [`std::prelude`](https://doc.rust-lang.org/std/prelude/index.html), [`std::env::set_var`](https://doc.rust-lang.org/std/env/fn.set_var.html)(Safety 절과 `Command::env` 안내), [`CommandExt::pre_exec`](https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html#tymethod.pre_exec). 2.4·2.5절에 인용한 속성은 툴체인과 함께 설치된 로컬 문서의 소스 페이지(`rustc --print sysroot` 아래 `share/doc/rust/html/src/`)에서 확인했습니다.
- 에러 설명: `rustc --explain E0034`, `rustc --explain E0133`처럼 에러 코드를 넣어 실행합니다.
