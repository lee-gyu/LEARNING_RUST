# ch21. 시맨틱 버저닝이 약속하는 것

Cargo는 버전 번호를 약속으로 읽습니다. `Cargo.toml`에 `parcel = "1.4"`라고 적은 사용자는 "1.4.0 이상 2.0.0 미만이면 어느 버전이든 지금 코드와 맞는다"고 믿고, Cargo는 그 범위에서 가장 새 버전을 골라 줍니다. 이 믿음이 맞으려면 라이브러리를 만드는 쪽이 어떤 변경이 사용자 코드를 깨는지 정확히 알고 버전의 자리를 올려야 합니다. 이 챕터는 그 약속을 양쪽에서 봅니다. **작성자 쪽**에서는 항목을 더하기만 해도 사용자가 깨지는 경우, 컴파일러가 잡지 못하는 동작 변경, 깨질 변경을 사용자가 알아차리게 내보내는 방법을 다룹니다. **사용자 쪽**에서는 버전 요구 사항이 실제로 어떤 버전을 허용하는지와 업데이트마다 무엇을 기대할 수 있는지를 다룹니다.

- 원제: [Item 21: Understand what semantic versioning promises](https://effective-rust.com/semver.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch21-semver`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

앞의 부록 챕터가 이 챕터로 미룬 것을 여기서 받습니다(괄호 안은 이 챕터의 절). [ch1](../ch1-express-data-structures/README.md)과 [ch12](../ch12-generics-vs-trait-objects/README.md)의 배리언트 추가와 `#[non_exhaustive]`(2.5절), [ch2](../ch2-express-common-behavior/README.md)의 바운드 조이기(2.3절, 4.3절), [ch7](../ch7-builders/README.md)의 필드 추가(2.3절, 2.5절), [ch10](../ch10-standard-traits/README.md)의 트레이트 구현 떼기(4.4절)와 `derive(Ord)`의 필드 순서(2.4절), [ch12](../ch12-generics-vs-trait-objects/README.md)의 dyn 호환(4.2절), [ch13](../ch13-default-implementations/README.md)이 미룬 호환되지 않는 변경의 규칙(2.3절), [ch19](../ch19-avoid-reflection/README.md)의 지원할 툴체인 버전(2.7절)입니다. 다음 본편 내용을 안다고 보고 진행합니다.

- [Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md) 2.3절: 버전 요구 사항 `"0.10.4"`의 뜻, `Cargo.lock`, `cargo update`. 4.4절: `rand` 0.8 코드가 0.10에서 깨지는 모습
- [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md): 크레이트, 모듈, `pub`
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.19절: `macro_rules!`(예제가 같은 사용자 코드를 여러 버전에 펼칠 때 씀)
- [Part 2 ch1](../../part2-editions/ch1-editions-overview/README.md) 2.5절, [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.6절: `rust-version`과 리졸버 3

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 버전 번호의 세 자리가 사용자에게 하는 약속과, 0.x에서 Cargo가 가장 왼쪽의 0이 아닌 자리를 메이저처럼 다룬다는 것을 설명할 수 있다. 버전 요구 사항(`"1.4"`, `"~1.4"`, `"=1.4.7"`, `"*"`)이 허용하는 범위와 Cargo가 그중 무엇을 고르는지 Cargo로 직접 확인할 수 있다.
2. 라이브러리의 변경이 호환되는지를 "옛 사용자 코드가 그대로 컴파일되고 같은 동작을 하는가"로 따질 수 있고, 포괄 구현·제네릭 메서드 추가, 바운드 조이기, 트레이트 구현 떼기, `derive(Ord)` 필드 재배치가 사용자를 깨는 이유를 설명할 수 있다.
3. 나중에 배리언트나 필드를 더할 공개 타입에 `#[non_exhaustive]`를 붙이고, 그 효과가 크레이트 밖에서만 나타난다는 것과 나중에 붙이는 것이 왜 호환되지 않는 변경인지 설명할 수 있다.
4. 호환되지 않는 변경을 `#[deprecated]`를 붙인 마이너 릴리스와 옛 API를 지우는 메이저 릴리스로 나눠 내보내고, 동작이 바뀌는 변경은 타입을 바꿔 컴파일 에러로 드러내는 코드를 작성할 수 있다.
5. `rust-version`으로 최소 지원 Rust 버전(MSRV)을 적고 clippy의 `incompatible_msrv` 린트로 지켜지는지 확인할 수 있다.

## 2. 핵심 개념

2.1절에서 버전 번호가 무엇을 약속하는지 정리하고, 2.2~2.6절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 지침 1은 사용자 쪽, 지침 2~5는 작성자 쪽입니다. 모든 예제는 가상의 택배 라이브러리 `parcel`의 여러 버전을 씁니다. `examples/`는 한 파일 안에 버전마다 모듈(`v1_0`, `v1_1` 등)을 두고 같은 사용자 코드를 각 버전에 대해 컴파일하며, 크레이트 경계나 Cargo 자체가 필요한 실습은 `projects/`에 두었습니다.

### 2.1 지도: 버전 번호가 약속하는 것

시맨틱 버저닝(semantic versioning, SemVer)은 버전을 `메이저.마이너.패치` 세 자리로 적고, 어느 자리를 올렸는지로 변경의 성격을 알립니다.

| 올린 자리 | 작성자가 하는 약속 | 사용자가 기대할 수 있는 것 |
| --- | --- | --- |
| 패치 (1.4.7 → 1.4.12) | 공개 API는 그대로이고 버그만 고쳤다 | 그대로 올려도 된다 |
| 마이너 (1.4.x → 1.6.0) | 기존 API는 그대로이고 새 API를 더했다(옛 API를 더는 권장하지 않는다(deprecated)고 표시하기도 함) | 그대로 올려도 된다. 단, 새 API를 쓰기 시작하면 옛 버전으로 돌아갈 수 없다 |
| 메이저 (1.x → 2.0.0) | 호환되지 않는 변경이 있다 | 코드를 고칠 각오를 한다. 컴파일되더라도 쓰임새를 다시 확인한다 |

두 가지 규칙이 이 표를 받칩니다. 첫째, **한 번 낸 버전은 고치지 않습니다.** 무엇이든 바꾸면 새 버전을 내고, 릴리스마다 git 태그를 달아 어느 커밋이 어느 버전인지 남깁니다. crates.io도 이미 올라간 버전을 덮어쓰지 못하게 합니다(Cargo Book의 Publishing on crates.io). 실수로 낸 버전은 고치는 대신 게시 취소(yank)해서 새로 고르는 대상에서 빼는데, 이미 `Cargo.lock`에 그 버전을 적은 사용자는 계속 그 버전을 씁니다. 둘째, **0.x는 초기 개발 단계입니다.** semver.org는 0.y.z에서는 무엇이든 바뀔 수 있다고 정하지만, Cargo는 여기에 규칙을 하나 더합니다. 가장 왼쪽의 0이 아닌 자리를 메이저처럼 다루는 것입니다. 그래서 0.3.9 → 0.4.0과 0.0.3 → 0.0.4는 메이저 변경이고, 0.3.1 → 0.3.9는 패치나 마이너에 해당합니다. Part 0 ch2 4.4절의 `rand` 0.8 → 0.10이 Cargo가 보기에 메이저 변경이었던 이유입니다. 0.x에서는 세 종류의 변경을 두 자리로만 나타낼 수 있으므로, 책은 API가 쓸 만해지면 0.x에 머물지 말고 1.0.0으로 가라고 권합니다.

표의 "그대로 올려도 된다"에는 단서가 있습니다. 공개 API의 사용자가 충분히 많으면 문서로 약속하지 않은 동작에도 누군가는 기대게 됩니다(하이럼의 법칙, Hyrum's Law). 그래서 패치 하나가 누군가의 코드를 깨는 일은 실제로 일어나고, 작성자는 시그니처만이 아니라 동작의 변화도 따져야 합니다(2.4절).

### 2.2 지침 1(사용자): 버전 요구 사항은 호환 범위로 적고, `"*"`는 쓰지 않는다

**문제 상황.** 사용자 쪽 패키지 `shop`이 `parcel`에 의존합니다. crates.io에 `parcel`이 0.3.9, 0.4.0, 1.4.7, 1.4.12, 1.6.0, 2.0.0 여섯 버전으로 올라와 있다고 하고, `shop`의 `Cargo.toml`에 무엇을 적을지 정해야 합니다.

[`projects/version_requirements`](projects/version_requirements/Cargo.toml)는 이 상황을 네트워크 없이 재현합니다. 프로젝트의 `.cargo/config.toml`이 crates.io 대신 `vendor/` 디렉터리를 쓰도록 바꾸고(Cargo Book의 Source Replacement), `vendor/`에 여섯 버전을 넣어 두었습니다. 이 모양은 `cargo vendor`가 만드는 디렉터리 소스와 같습니다. 버전마다 있는 `.cargo-checksum.json`은 디렉터리 소스에 꼭 있어야 하는 메타데이터 파일로, 원래는 파일이 실수로 바뀌지 않았는지 확인할 체크섬을 담지만 여기서는 목록을 비워 두었습니다. Cargo는 실제 레지스트리에서 하듯 요구 사항을 만족하는 버전 가운데 가장 높은 것을 고릅니다. 요구 사항을 바꿔 가며 `Cargo.lock`을 지우고 실행한 결과는 다음과 같습니다(범위는 Cargo Book의 Specifying Dependencies, 고른 버전과 괄호 안의 메시지는 Cargo의 실제 출력).

| 요구 사항 | 뜻(허용 범위) | Cargo가 고른 버전 | Cargo가 덧붙인 말 |
| --- | --- | --- | --- |
| `"1.4"` | `>=1.4.0, <2.0.0` | 1.6.0 | `(available: v2.0.0)` |
| `"1.4.7"` (`"^1.4.7"`과 같음) | `>=1.4.7, <2.0.0` | 1.6.0 | `(available: v2.0.0)` |
| `"~1.4"` | `>=1.4.0, <1.5.0` | 1.4.12 | `(available: v1.6.0)` |
| `"~1.4.7"` | `>=1.4.7, <1.5.0` | 1.4.12 | `(available: v1.6.0)` |
| `"=1.4.7"` | 1.4.7만 | 1.4.7 | `(available: v1.6.0)` |
| `"1.*"`, `"~1"` | `>=1.0.0, <2.0.0` | 1.6.0 | `(available: v2.0.0)` |
| `">=1.4, <1.6"` | 적은 그대로 | 1.4.12 | `(available: v1.6.0)` |
| `"1.7"` | `>=1.7.0, <2.0.0` | 없음, 에러 | 4.5절 |
| `"0.3"` | `>=0.3.0, <0.4.0` | 0.3.9 | `(available: v2.0.0)` |
| `"0"` | `>=0.0.0, <1.0.0` | 0.4.0 | `(available: v2.0.0)` |
| `"*"` | 모든 버전 | 2.0.0 | 없음. `shop`이 E0308로 컴파일되지 않음 |

가장 흔한 오해는 첫 줄입니다. `"1.4"`는 "1.4.x"가 아니라 "1.4.0 이상의 1.x"라서 1.6.0이 골라졌습니다. 1.4.x 안에 머물게 하는 것은 `"~1.4"`입니다. 0.x에서는 2.1절의 규칙대로 `"0.3"`이 0.4.0으로 넘어가지 않습니다. 괄호 안의 `available`은 요구 사항 밖에 더 새 버전이 있다는 알림입니다. 고른 버전과 호환되는 더 새 버전이 있으면 그것을(`~1.4`의 1.6.0), 없으면 가장 새 버전을(2.0.0) 알려 주고, 가장 새 버전을 고르면 `Adding` 줄 자체가 나오지 않습니다.

**비권장 코드.**

```toml
[dependencies]
parcel = "*"
```

`"*"`는 "어떤 버전이든 괜찮다"는 뜻이라 메이저 버전의 경계를 지우는 요구 사항입니다. 이 줄을 적은 날에는 1.6.0이 골라져 빌드되다가, 2.0.0이 게시된 뒤 `Cargo.lock` 없이 빌드하는 사람(새로 내려받은 동료, CI, 이 패키지를 의존성으로 쓰는 다른 프로젝트)은 2.0.0을 받고 다음 에러를 봅니다.

```text
error[E0308]: mismatched types
  --> src/main.rs:10:44
   |
10 |     println!("1200g 배송비: {}원", parcel::fee(1200));
   |                                    ----------- ^^^^ expected `Grams`, found integer
```

2.0.0은 메이저 버전을 올려 "호환되지 않는다"고 제대로 알렸는데, 요구 사항이 그 알림을 무시한 것입니다. crates.io는 `"*"` 의존성을 가진 패키지의 게시를 거부합니다(Cargo Book과 책의 설명이며, 이 저장소에서는 게시를 시도해 확인하지 않았습니다). `"0.*"`도 0.x의 모든 메이저를 허용하므로 같은 문제가 있습니다.

**권장 코드.**

```toml
[dependencies]
parcel = "1.4"   # >=1.4.0, <2.0.0
```

캐럿 요구 사항(기본 형식)은 "이 버전 이상, 다음 메이저 미만"이라 작성자의 약속(마이너와 패치는 호환)과 정확히 맞물립니다. 버그 수정과 새 기능은 받고, 깨질 수 있는 메이저는 사용자가 요구 사항을 직접 `"2"`로 바꿀 때만 받습니다. 빌드를 재현하는 일은 `Cargo.lock`이 맡으므로(Part 0 ch2 2.3절) 요구 사항을 좁게 고정할 이유도 없습니다.

요구 사항의 아래쪽 경계는 **실제로 필요한 가장 낮은 버전**이어야 합니다. 1.6.0에서 생긴 `fee_with_insurance`를 쓰기 시작했다면 요구 사항을 `"1.6"`으로 올립니다. `"1.4"`로 두면 지금은 1.6.0이 골라져 빌드되지만, 이 패키지를 다른 프로젝트가 쓰면서 그쪽 `Cargo.lock`이 1.4.12를 고르는 순간 `fee_with_insurance`가 없어 컴파일되지 않습니다. 2.1절 표의 "새 API를 쓰기 시작하면 옛 버전으로 돌아갈 수 없다"를 요구 사항에 그대로 적어 두는 것입니다(연습 1).

**지침을 따르지 않아도 되는 경우.** 함께 만들어 함께 게시하는 크레이트 쌍은 `=`로 묶기도 합니다. 이 저장소가 ch4에서 쓰는 `thiserror` 2.0.21은 절차적 매크로 크레이트 `thiserror-impl`을 `=2.0.21`로, ch7의 `derive_builder` 0.20.2는 `derive_builder_macro`를 `=0.20.2`로 요구합니다(로컬 레지스트리 캐시에 내려받은 두 크레이트의 `Cargo.toml`로 확인). 두 크레이트의 내부 인터페이스는 공개 약속이 아니므로 버전을 하나로 맞춰야 하기 때문입니다. 그 밖에 애플리케이션에서 버전을 고정할지, 범위로 둘지, `cargo update`로 언제 올릴지는 [ch25](../ch25-dependency-graph/README.md)에서 다룹니다.

### 2.3 지침 2(작성자): 변경마다 "옛 사용자 코드가 그대로 컴파일되는가"를 따진다

**문제 상황.** 공개 항목을 지우거나 이름·시그니처를 바꾸면 메이저라는 것은 분명합니다. 어려운 것은 "더하기만 한" 변경입니다. 대부분은 마이너지만, 일부는 더하기만 했는데도 사용자 코드를 깹니다.

**비권장 코드.** "항목을 더했으니 마이너"라고 판단하고 아래 변경을 1.1.0으로 냅니다. 모두 `compile_fail/`에서 1.0.0 시절의 사용자 코드가 실제로 깨지는 모습을 확인할 수 있습니다(4절).

```rust
// 공개 트레이트에 포괄 구현을 더함 → 사용자의 impl Label for Ticket이 E0119
impl<T: Display> Label for T { /* .. */ }

// 기본 구현이 있는 제네릭 메서드를 더함 → 사용자의 dyn Notifier가 E0038
fn notify_all<I: IntoIterator<Item = String>>(&self, messages: I) -> Vec<String> { /* .. */ }

// 바운드에 Ord를 더함 → summarize(&[1.2_f64, 0.8])이 E0277
pub fn summarize<T: Display + Ord>(weights: &[T]) -> String { /* .. */ }
```

**권장 코드.** 판단 기준을 "더했는가, 지웠는가"가 아니라 **"1.0.0 시절의 사용자 코드가 그대로 컴파일되고 같은 동작을 하는가"** 로 둡니다. [`01_compatible_changes.rs`](examples/01_compatible_changes.rs)는 이 기준을 통과하는 다섯 변경(새 함수, 새 고유 메서드, 새 기본 메서드, 비공개 필드가 이미 있는 구조체에 필드 추가, 바운드 느슨하게 하기)을 1.1.0에 넣고, 1.0.0 시절의 사용자 코드를 매크로로 두 버전에 펼쳐 같은 결과가 나오는지 봅니다(3절). 위의 세 변경은 다음처럼 사용자를 깨지 않는 모양으로 바꾸거나, 바꿀 수 없으면 메이저로 미룹니다.

| 하려던 일 | 대신 하는 방법 | 확인한 곳 |
| --- | --- | --- |
| `Display` 타입 모두에 `Label`을 주기 | 포괄 구현은 메이저에서. 마이너에서는 라이브러리 자신의 타입에만 구현 | 4.1절 |
| 여러 메시지를 한 번에 보내는 메서드 | 새 메서드에 `where Self: Sized`를 붙임([ch12](../ch12-generics-vs-trait-objects/README.md) 2.6절) | 4.2절 |
| 정렬해서 요약하기 | 기존 함수는 두고 `summarize_sorted<T: Display + Ord>`를 새로 더함 | 4.3절, 연습 4 |

더하기처럼 보이지만 사용자를 깨는 변경을 앞 챕터의 것까지 모으면 다음과 같습니다.

| 변경 | 깨지는 사용자 코드 | 에러 | 다룬 곳 |
| --- | --- | --- | --- |
| 열거형에 배리언트 추가 | `_` 갈래 없는 `match` | E0004 | [ch1](../ch1-express-data-structures/README.md) 2.3절, 2.5절 |
| 공개 필드만 있는 구조체에 필드 추가 | 구조체 리터럴 | E0063 | [ch7](../ch7-builders/README.md) 2.2절, 2.5절 |
| 트레이트에 기본 메서드 추가 | 사용자 트레이트와 같은 이름의 메서드 호출 | E0034 | [ch13](../ch13-default-implementations/README.md) 2.5절, 4.2절 |
| 트레이트에 필수 메서드 추가 | 사용자의 모든 `impl` | E0046 | [ch13](../ch13-default-implementations/README.md) 2.5절 |
| 공개 트레이트에 포괄 구현 추가 | 겹치는 사용자의 `impl` | E0119 | 4.1절 |
| 제네릭 기본 메서드 추가 | `dyn Trait` | E0038 | 4.2절, [ch12](../ch12-generics-vs-trait-objects/README.md) 2.6절 |
| 바운드 조이기 | 조건을 못 맞추는 인자 | E0277 | 4.3절, [ch2](../ch2-express-common-behavior/README.md) 2.3절 |
| 트레이트 구현 떼기 | 그 트레이트를 쓰는 곳 | E0277 등 | 4.4절, [ch10](../ch10-standard-traits/README.md) 2.3절, 2.9절 |
| 이미 있는 타입에 `#[non_exhaustive]` | `match`, 리터럴, 패턴 | E0004, E0639, E0638 | 2.5절 |
| 새 공개 항목 | 사용자의 와일드카드 가져오기(`use parcel::*`)와 이름 충돌 | | [ch23](../ch23-avoid-wildcard-imports/README.md) |

**트레이트 구현을 더하는 것은 왜 대부분 마이너일까요?** 컴파일러가 사용자 쪽에 "그 구현이 없다는 사실"에 기대는 코드를 처음부터 쓰지 못하게 막아 두기 때문입니다. 표준 라이브러리 타입으로 확인해 보면, 사용자가 `impl<T: Display> Describe for T`와 `impl Describe for Vec<u8>`를 함께 쓰면 `Vec<u8>`이 `Display`가 아닌데도 E0119이고, 이유가 이렇게 나옵니다.

```text
  = note: upstream crates may add a new impl of trait `std::fmt::Display` for type `std::vec::Vec<u8>` in future versions
```

의존하는 크레이트가 나중에 구현을 더할 자리를 일관성 규칙이 미리 비워 두는 것입니다. 반대로 떼는 쪽에는 그런 보호가 없으므로, [ch10](../ch10-standard-traits/README.md)의 말대로 구현을 떼는 것이 더하는 것보다 훨씬 큰 변경입니다.

Cargo Book의 [SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)는 이런 변경을 메이저, 마이너, "깨질 수도 있음(possibly-breaking)" 세 종류로 나눈 목록입니다. 기본 메서드 추가와 고유 메서드 추가처럼 이름 충돌로만 깨지는 변경, MSRV 올리기가 세 번째 종류이며, 보통 마이너로 내되 프로젝트가 정합니다. 목록 전체와 게시 절차는 Part 3 `ch10-publishing-semver`에서 다룹니다. 이 목록을 사람이 릴리스마다 대조하기는 어려우므로, 두 릴리스의 공개 API를 rustdoc JSON으로 비교해 규칙 위반을 찾는 `cargo-semver-checks`(`cargo install cargo-semver-checks --locked`로 설치, 게시 전에 `cargo semver-checks` 실행) 같은 도구가 있습니다. 이 저장소에서는 설치하지 않아 실행해 확인하지 못했습니다. 저장소의 설명에 따르면 기본으로 crates.io의 직전 버전과 비교하고, 모든 호환성 문제를 잡지는 못합니다.

**지침을 따르지 않아도 되는 경우.** 사용자를 모두 내가 고칠 수 있다면 버전 규칙은 짐일 뿐입니다. 워크스페이스 안에서만 쓰는 `publish = false` 크레이트는 호환되지 않는 변경을 하고 컴파일 에러가 가리키는 곳을 함께 고치면 됩니다. 실행 파일(애플리케이션)도 Cargo Book이 고정된 규칙을 두지 않고 SemVer의 취지를 따르라고만 안내합니다. 또 공개 API를 줄여 두면 따질 변경 자체가 줄어듭니다. 가시성으로 약속을 줄이는 방법은 [ch22](../ch22-minimize-visibility/README.md)에서 다룹니다.

### 2.4 지침 3(작성자): 컴파일러가 잡지 못하는 동작 변경도 약속으로 다룬다

**문제 상황.** 배송 예약 칸 `Slot`은 `day`, `window` 순서로 선언하고 `PartialOrd`/`Ord`를 derive했습니다. derive한 비교는 필드를 선언한 순서대로 비교하므로(`Ord` 문서: "lexicographic ordering based on the top-to-bottom declaration order of the struct's members") 정렬하면 날짜가 먼저, 같은 날이면 이른 시간대가 먼저 옵니다. 다음 패치에서 관리자가 필드를 보기 좋게 재배치하려 합니다.

**비권장 코드.** 필드 순서만 바꿉니다. 이름과 타입이 모두 그대로라 패치로 냅니다.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot {
    pub window: Window, // 앞으로 옮김
    pub day: u8,
}
```

사용자 코드는 필드 이름을 적은 구조체 리터럴만 쓰므로 경고 하나 없이 컴파일됩니다. 그런데 [`02_silent_behavior_change.rs`](examples/02_silent_behavior_change.rs)에서 같은 사용자 코드의 결과가 달라집니다.

```text
[1.0.0]
  배송 순서: 1일 오전, 1일 저녁, 2일 오전, 2일 오후
  마지막 배송 안내: 2일 오후
[1.0.1 비권장: 필드만 재배치]
  배송 순서: 1일 오전, 2일 오전, 2일 오후, 1일 저녁
  마지막 배송 안내: 1일 저녁
```

**권장 코드.** 비교 순서를 선언 순서에서 떼어 내 손으로 적고, 순서의 약속을 검사로 지킵니다.

```rust
impl Ord for Slot {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.day, self.window).cmp(&(other.day, other.window))
    }
}

pub fn order_contract_holds() -> bool { // 라이브러리의 단위 테스트에 해당
    Slot { day: 1, window: Window::Evening } < Slot { day: 2, window: Window::Morning }
}
```

이제 필드를 어떻게 재배치해도 정렬 결과는 1.0.0과 같고, 예제에서 비권장 판의 같은 검사는 `실패`를 출력합니다. 같은 원리로 `Ord`를 derive한 열거형의 배리언트 순서를 바꾸는 것도 동작 변경입니다. 일반화하면, 정렬 순서·기본값·반올림 방식·출력 형식처럼 컴파일러가 보지 못하는 공개 동작은 문서나 테스트로 "약속"과 "약속 아님"을 나눠 두고, 약속한 동작을 바꾸면 시그니처가 같아도 호환되지 않는 변경으로 다룹니다. Cargo Book도 실행 중 동작에 영향을 주는 변경을 호환되지 않는 변경으로 볼지는 관리자가 판단할 문제라고 적습니다.

**지침을 따르지 않아도 되는 경우.** 버그 수정은 동작을 바꾸는 것이 목적입니다. 틀린 계산을 바로잡는 패치는 누군가 그 틀린 값에 기대고 있더라도 패치로 내고, 변경 기록에 무엇이 바뀌었는지 적습니다. 또 처음부터 약속하지 않는다고 밝힌 동작은 바꿔도 됩니다. 표준 라이브러리의 `HashMap` 순회 순서가 그런 예입니다(Part 0 ch8). 다만 하이럼의 법칙 때문에 그런 변경도 사용자가 많으면 누군가를 깨므로, 그 가능성을 알고 냅니다.

### 2.5 지침 4(작성자): 나중에 더할 곳은 처음부터 `#[non_exhaustive]`로 열어 둔다

**문제 상황.** `parcel`의 배송 상태 `Status`는 지금 접수, 배송 중, 배송 완료 세 가지지만 반송 같은 상태가 곧 늘 것 같습니다. 배송 옵션 `Options`도 서명 여부 하나로 시작하지만 보험 같은 필드가 늘 것입니다.

**비권장 코드.** 1.0.0을 속성 없이 냅니다. 사용자는 자연스럽게 모든 배리언트를 나열한 `match`와 구조체 리터럴을 씁니다. 그러면 1.1.0에서 배리언트를 더하는 순간 `match`가 E0004([ch1](../ch1-express-data-structures/README.md)의 `compile_fail/e0004_new_case_variant.rs`), 필드를 더하는 순간 리터럴이 E0063으로 깨지므로([ch7](../ch7-builders/README.md) 2.2절) 둘 다 메이저 변경이 됩니다. 뒤늦게 `#[non_exhaustive]`를 붙이는 것도 해결이 되지 못합니다. 그 사용자 코드를 그대로 모은 [`projects/non_exhaustive/broken_shop`](projects/non_exhaustive/broken_shop/src/main.rs)은 속성이 붙은 `parcel`에 대해 E0004, E0639, E0638로 컴파일되지 않고, `parcel/src/lib.rs`에서 두 속성을 지우면 컴파일되어 실행됩니다. 속성을 나중에 붙이는 일 자체가 호환되지 않는 변경입니다.

**권장 코드.** 늘어날 타입에는 1.0.0부터 속성을 붙이고, 구조체는 크레이트 밖에서 만들 길(생성 함수와 세터)을 함께 줍니다.

```rust
#[non_exhaustive]
pub enum Status { Accepted, InTransit, Delivered, /** 1.1.0에서 추가 */ Returned }

#[derive(Default)]
#[non_exhaustive]
pub struct Options { pub signature: bool, /** 1.1.0에서 추가 */ pub insured: bool }

impl Options {
    pub fn new() -> Self { Options::default() }
    pub fn signature(mut self, yes: bool) -> Self { self.signature = yes; self }
}
```

[`projects/non_exhaustive`](projects/non_exhaustive/Cargo.toml)는 라이브러리 `parcel`(1.1.0)과 사용자 `shop`을 두 크레이트로 나눴습니다. `#[non_exhaustive]`는 정의한 크레이트 밖에서만 효과가 있기 때문입니다. 크레이트 밖의 `shop`에는 세 가지가 요구됩니다.

- 열거형의 `match`에는 `_` 갈래가 있어야 합니다. 배리언트를 **모두 나열해도** 필요합니다. 다음 버전에서 늘 수 있기 때문입니다.
- 구조체는 리터럴로 만들 수 없습니다. `Options { insured: true, ..Default::default() }` 같은 구조체 갱신 문법도 리터럴이라 막힙니다([ch7](../ch7-builders/README.md) 2.3절의 `Default` + `..`).
- 구조 분해 패턴에는 `..`가 있어야 합니다.

`shop`은 1.0.0 시절에 이 규칙대로 쓴 코드라 1.1.0에서 생긴 `Returned`를 모르지만 그대로 컴파일되고, 새 배리언트는 `_` 갈래로 들어갑니다(`KR-0777: 배송 상태가 바뀌었습니다(반송)`). 반면 `parcel` 크레이트 안에서는 속성이 효과가 없어서, `label` 함수는 `_` 없이 네 배리언트를 나열하고 `express` 함수는 구조체 리터럴을 씁니다. 라이브러리 쪽은 배리언트를 더할 때 컴파일러의 도움을 그대로 받습니다.

**지침을 따르지 않아도 되는 경우.** 앞으로도 늘지 않을 닫힌 집합에는 붙이지 않습니다. `#[non_exhaustive]`는 사용자에게서 완전성 검사를 영원히 빼앗기 때문입니다. `_` 갈래가 있으면 새 배리언트가 생겨도 컴파일러가 알려 주지 않고 조용히 기본 처리로 흘러갑니다(위의 "배송 상태가 바뀌었습니다"). 표준 라이브러리에서 `std::cmp::Ordering`과 `Option`에는 속성이 없어 모든 경우를 나열하면 되고, 종류가 늘어 온 `std::io::ErrorKind`에는 속성이 있어 `_` 없는 `match`가 E0004(``note: `ErrorKind` is marked as non-exhaustive``)입니다. 구조체에 비공개 필드가 이미 있다면 크레이트 밖에서 리터럴로 만들 수 없으므로 속성이 필요 없고, 필드를 비공개로 두는 방법 자체는 [ch22](../ch22-minimize-visibility/README.md)에서 다룹니다. 속성의 문법과 배리언트에 붙이는 경우는 Part 9 `ch4-attributes`에서 다룹니다.

### 2.6 지침 5(작성자): 호환되지 않는 변경은 사용자가 알아차리게 내보낸다

메이저 버전을 올리는 것만으로는 부족합니다. 사용자가 무엇을 고쳐야 하는지 알 수 있어야 합니다. 방법은 두 가지입니다.

**문제 상황 1.** 배송 조회 `track(no) -> Option<Status>`는 송장 번호의 형식이 틀린 경우와 등록되지 않은 경우를 모두 `None`으로 돌려줘 구별할 수 없습니다. `Result`를 돌려주는 API로 바꾸려 합니다.

**비권장 코드.** 2.0.0에서 `track`을 바로 지우고 새 함수로 바꿉니다. 사용자는 2.0.0으로 올린 날 처음으로 바뀐 것을 알게 되고, 옮길 곳도 에러 메시지에서 짐작해야 합니다(``error[E0425]: cannot find function `track` in module `v2_0` ``).

**권장 코드.** 두 단계로 나눕니다. 1.1.0(마이너)에서 새 API `lookup`을 더하고, 옛 API는 새 API로 다시 구현해 동작을 지키면서 deprecated로 표시해 지울 것을 예고합니다. 2.0.0(메이저)에서 옛 API를 지웁니다.

```rust
#[deprecated(
    since = "1.1.0",
    note = "lookup을 쓰세요. 형식 오류와 미등록을 구별합니다. 2.0.0에서 지웁니다"
)]
pub fn track(no: &str) -> Option<Status> {
    lookup(no).ok()
}
```

1.1.0으로 올린 사용자의 옛 코드는 그대로 컴파일되어 같은 결과를 내고, 부르는 곳마다 경고를 받습니다. [`04_deprecate_then_remove.rs`](examples/04_deprecate_then_remove.rs)의 `#[expect(deprecated, ..)]`를 지우고 빌드하면 다음 경고가 나옵니다. 예제는 한 크레이트라 경로가 `v1_1::track`으로 나옵니다. `projects/non_exhaustive`의 `parcel`에 `#[deprecated]` 함수를 넣고 `shop`에서 불러 보니, 다른 크레이트에서는 같은 경고가 `` use of deprecated function `parcel::함수이름` ``으로 나왔습니다.

```text
warning: use of deprecated function `v1_1::track`: lookup을 쓰세요. 형식 오류와 미등록을 구별합니다. 2.0.0에서 지웁니다
  = note: `#[warn(deprecated)]` on by default
```

`note`에는 무엇으로 옮기면 되는지와 언제 지워지는지를 적습니다. 경고를 보고 `lookup`으로 옮긴 사용자는 2.0.0에서 고칠 것이 없습니다. semver.org도 deprecated 표시는 마이너로 내고, 지우기 전에 예고를 담은 마이너 릴리스가 적어도 하나 있어야 한다고 안내합니다.

**문제 상황 2.** 보관함 예약 `reserve(locker, hold: u32)`의 `hold`는 1.x에서 "일" 단위였습니다. 2.0.0에서 시간 단위로 바꾸려 합니다.

**비권장 코드.** 메이저 버전은 올렸지만 시그니처는 `hold: u32` 그대로 둡니다. [`03_breaking_change_must_break.rs`](examples/03_breaking_change_must_break.rs)에서 1.x 시절의 `reserve(7, 3)`(3일)은 2.0.0에서 경고 없이 컴파일되고 3시간짜리 예약이 됩니다(`7번 보관함, 3시간 보관`). 요구 사항을 `"2"`로 바꾼 사용자가 컴파일이 되는 것을 보고 끝났다고 생각하면, 택배는 사흘이 아니라 세 시간 뒤에 반송됩니다.

**권장 코드.** 동작이 호환되지 않게 바뀌면 타입도 바꿔서 옛 사용법이 컴파일되지 않게 만듭니다.

```rust
pub fn reserve(locker: u32, hold: Duration) -> Reservation { /* .. */ }
```

이제 옛 코드는 2.0.0에서 고칠 곳마다 에러가 납니다.

```text
error[E0308]: mismatched types
68 |         $parcel::reserve(7, 3).summary()
   |         ----------------    ^ expected `Duration`, found integer
```

고친 코드 `reserve(7, Duration::from_hours(3 * 24))`는 단위가 부르는 곳에 드러납니다. 단위를 타입에 담는 방법은 [ch6](../ch6-newtype-pattern/README.md)의 단위 뉴타입과 같습니다. 사용자 쪽에서 얻을 교훈도 함께 기억합니다. 메이저 업데이트는 컴파일이 되더라도 바뀐 쓰임새가 없는지 변경 기록을 읽고 확인합니다.

**지침을 따르지 않아도 되는 경우.** deprecated로 예고하는 기간은 사용자에게 옮길 시간을 주려는 것이므로, 사용자가 없거나 모두 내가 고칠 수 있는 크레이트(2.3절)라면 바로 지워도 됩니다. 새 API를 옛 API와 함께 둘 수 없는 경우(옛 API를 남기는 것 자체가 위험한 경우 등)도 예고 없이 메이저에서 바꿀 수밖에 없습니다. 이때는 변경 기록에 옮기는 방법을 자세히 적습니다. 타입을 바꿀 마땅한 방법이 없는 동작 변경이라면 함수 이름을 바꾸는 것(`reserve` → `reserve_hours`)도 옛 코드를 컴파일 에러로 만드는 방법입니다.

### 2.7 그 밖의 약속: MSRV, 라이선스, 기본 기능

공개 API는 타입과 함수만이 아닙니다. 책은 라이선스와 기본 기능 집합을 API의 일부로 보라고 하고, MSRV도 그렇게 볼 수 있다고 덧붙입니다.

- **최소 지원 Rust 버전(MSRV).** 새 버전이 더 최신 컴파일러에서만 빌드되면, 툴체인을 바로 올릴 수 없는 사용자에게는 호환되지 않는 변경입니다. 책은 MSRV를 API의 일부로 볼 "수도 있다"고 하고, Cargo Book은 MSRV 올리기를 "깨질 수도 있음"으로 분류하되 보통은 마이너로 다루라고 권합니다. 어느 쪽을 택하든 먼저 MSRV를 `Cargo.toml`의 `rust-version`에 적어 둡니다. 그러면 그보다 오래된 rustc에서는 Cargo가 컴파일을 시작하기 전에 거절하고([Part 2 ch1](../../part2-editions/ch1-editions-overview/README.md) 2.5절), 2024 에디션의 리졸버 3은 사용자의 `rust-version`에 맞는 의존성 버전을 골라 줍니다([Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.6절).
- **라이선스.** 라이선스를 바꾸면 사용자가 그 크레이트를 계속 쓸 수 있는지가 달라지므로 호환되지 않는 변경입니다.
- **기본 기능 집합.** 기본으로 켜진 기능을 끄면 그 기능에 기대던 사용자가 깨집니다. 기능을 설계하는 지침은 [ch26](../ch26-feature-creep/README.md)에서 다룹니다.

적어 둔 MSRV는 지켜지는지 확인해야 합니다. clippy의 `incompatible_msrv` 린트(기본 warn, suspicious 그룹)는 `rust-version`보다 나중에 안정화된 표준 라이브러리 API를 쓰면 경고합니다. `projects/non_exhaustive/parcel`은 `rust-version = "1.85"`인데, 여기에 1.86에서 안정화된 `get_disjoint_mut`으로 두 칸을 맞바꾸는 함수를 넣고 `cargo clippy`를 실행하면 그 호출을 가리키며 다음과 같이 경고합니다.

```text
warning: current MSRV (Minimum Supported Rust Version) is `1.85.0` but this item is stable since `1.86.0`
   = note: `#[warn(clippy::incompatible_msrv)]` on by default
```

1.85에서도 되는 `split_at_mut` 같은 API로 바꾸거나, `get_disjoint_mut`이 꼭 필요하면 `rust-version`을 올리고 그것을 변경 기록에 적습니다.

[ch19](../ch19-avoid-reflection/README.md) 2.5절이 "1.86.0보다 오래된 툴체인도 지원해야 하면 `as_any` 모양을 유지한다"고 한 판단이 바로 이 MSRV를 정하는 일입니다. 린트는 표준 라이브러리 API만 보므로 언어 기능이나 의존성의 MSRV까지 확인하려면 그 버전의 툴체인으로 실제로 빌드해 봐야 합니다. CI에서 MSRV 툴체인으로 빌드하고 라이선스를 점검하는 방법은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)과 [ch25](../ch25-dependency-graph/README.md)에서 다룹니다.

### 2.8 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc·cargo 1.100.0에서 확인한 차이와 이 챕터가 더한 것은 다음과 같습니다.

- **Cargo가 요구 사항 밖의 새 버전을 알려 줍니다.** 책은 새 버전을 알아내는 수단으로 `cargo update`와 Dependabot을 듭니다. cargo 1.100.0은 버전을 고를 때 요구 사항 밖에 더 새 버전이 있으면 `Adding parcel v1.6.0 (available: v2.0.0)`처럼 함께 알려 줍니다(2.2절).
- **리졸버 3과 `rust-version`.** 2024 에디션은 `rust-version`에 맞는 의존성 버전을 먼저 고르는 리졸버 3을 씁니다. 이 챕터의 `Locking 1 package to highest Rust 1.100.0 compatible version` 줄이 그 흔적입니다. 같은 패키지에 `resolver = "2"`를 적으면 이 줄이 `Locking 1 package to highest compatible version`으로 바뀝니다(자세한 것은 Part 2 ch5 2.6절).
- **MSRV를 확인하는 린트.** clippy `incompatible_msrv`가 기본으로 켜져 있어 `rust-version`을 적기만 하면 표준 라이브러리 API의 MSRV 위반을 잡습니다(2.7절).
- **용어.** 책의 "object safe"는 현재 컴파일러에서 "dyn compatible"입니다(4.2절의 E0038 메시지). 이 저장소는 "dyn 호환"으로 씁니다.
- **기준 문서끼리의 차이.** 책은 포괄 구현 추가를 호환되지 않는 변경으로 들지만, Cargo Book의 SemVer 목록에는 이 항목이 없습니다. 4.1절에서 1.1.0에 포괄 구현을 더하면 사용자의 구현이 E0119로 깨지는 것을 확인했으므로 책을 따랐습니다.
- **책에 없는 지침과 내용.** 다음은 책의 Item 21이 명시적으로 권하지 않는 것입니다(2026-11-19에 온라인판에서 확인).
  - 2.1절의 게시 취소(yank): Cargo Book의 Publishing on crates.io가 근거입니다.
  - 지침 1의 "요구 사항의 아래쪽 경계를 실제로 쓰는 API가 생긴 버전으로 올린다"와 `=`로 묶는 크레이트 쌍: Cargo Book의 Specifying Dependencies(기본 요구 사항은 최소 버전을 정한다), 연습 1의 실행 결과, `thiserror` 2.0.21과 `derive_builder` 0.20.2의 매니페스트가 근거입니다. 연습용 레지스트리(디렉터리 소스)는 Cargo Book의 Source Replacement를 따랐습니다.
  - 지침 2의 "깨질 수도 있음" 분류와 트레이트 구현 추가가 대부분 마이너인 이유: Cargo Book의 SemVer Compatibility와 E0119의 `upstream crates may add a new impl` 설명이 근거입니다. 각 비권장 변경을 마이너로 하는 방법(4.1~4.3절의 해결)은 이 챕터가 정했고, `where Self: Sized`는 ch12 2.6절이 근거입니다.
  - 지침 3 전체(필드 재배치, 손으로 쓴 `Ord`, 순서 검사): 책은 하이럼의 법칙까지만 말합니다. `Ord` 문서의 derive 설명, ch10 2.2절, 예제 02의 실행 결과가 근거입니다.
  - 지침 4의 "처음부터 붙인다"와 따르지 않아도 되는 경우: 책은 `#[non_exhaustive]`가 아니면 배리언트 추가가 깨지고, 나중에 붙이는 것도 깨진다는 데까지 말합니다. Cargo Book의 enum-variant-new·attr-adding-non-exhaustive 항목(처음 도입할 때 붙이라는 권고), `io::ErrorKind`의 선언, `projects/non_exhaustive`의 실행 결과가 근거입니다.
  - 지침 2·3·5의 "따르지 않아도 되는 경우": 책은 예외를 따로 다루지 않습니다. Cargo Book의 Application compatibility 절과 이 챕터의 판단입니다.
  - 2.7절의 `rust-version`과 `incompatible_msrv`: Cargo Book의 The rust-version field와 clippy의 실행 결과가 근거입니다.

### 2.9 기억할 것

- 버전 번호는 약속입니다. 패치와 마이너는 "옛 코드가 그대로 동작한다", 메이저는 "고쳐야 할 수 있다"입니다. 0.x에서는 가장 왼쪽의 0이 아닌 자리가 메이저입니다.
- 요구 사항은 `"1.4"`처럼 캐럿으로 적습니다. `"1.4"`는 `>=1.4.0, <2.0.0`이고, 아래쪽 경계는 실제로 쓰는 API가 생긴 버전입니다. `"*"`는 메이저 경계를 지웁니다.
- 변경이 호환되는지는 "옛 사용자 코드가 그대로 컴파일되고 같은 동작을 하는가"로 따집니다. 포괄 구현, 제네릭 기본 메서드, 바운드 조이기, 트레이트 구현 떼기, `derive(Ord)` 필드 재배치는 더하기나 정리처럼 보여도 사용자를 깹니다.
- 늘어날 공개 열거형과 구조체에는 처음부터 `#[non_exhaustive]`를 붙입니다. 나중에 붙이는 것은 메이저 변경입니다.
- 지울 API는 마이너에서 새 API와 `#[deprecated(since, note)]`로 예고하고 다음 메이저에서 지웁니다. 동작이 바뀌면 타입을 바꿔 옛 코드를 컴파일 에러로 만듭니다.
- MSRV는 `rust-version`에 적고 clippy `incompatible_msrv`로 확인합니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행합니다. 모든 예제는 같은 사용자 코드를 라이브러리의 여러 버전(모듈)에 대해 컴파일하고 결과를 나란히 보여 줍니다. `projects/`는 해당 디렉터리로 이동해 실행합니다. 출력에 든 절대 경로는 `<저장소>/…`로 바꿔 적었습니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_compatible_changes.rs`](examples/01_compatible_changes.rs) | 2 | — | 호환되는 다섯 변경을 넣은 1.1.0 | 2.3 |
| [`02_silent_behavior_change.rs`](examples/02_silent_behavior_change.rs) | 3 | `derive(Ord)` 필드 재배치 | 손으로 쓴 `Ord`와 순서 검사 | 2.4 |
| [`03_breaking_change_must_break.rs`](examples/03_breaking_change_must_break.rs) | 5 | 단위만 바꾼 `hold: u32` | `hold: Duration` | 2.6 |
| [`04_deprecate_then_remove.rs`](examples/04_deprecate_then_remove.rs) | 5 | (바로 지우면 E0425) | 1.1.0의 `#[deprecated]`, 2.0.0에서 제거 | 2.6 |
| [`projects/version_requirements`](projects/version_requirements/Cargo.toml) | 1 | `"*"` | `"1.4"` | 2.2 |
| [`projects/non_exhaustive`](projects/non_exhaustive/Cargo.toml) | 4 | `broken_shop`(일부러 실패) | `parcel` + `shop` | 2.5, 2.7 |

### 01_compatible_changes — 호환되는 변경

`v1_0`과 `v1_1` 모듈이 `parcel` 1.0.0과 1.1.0입니다. 1.0.0 시절에 쓴 사용자 코드(사용자가 `Carrier`를 구현한 `QuickBike` 포함)는 매크로 `user_code!` 하나로 적고 두 버전에 펼쳤습니다. 실제로는 `Cargo.toml`의 버전만 바꿔 다시 빌드하는 것에 해당합니다. 마지막 구역은 1.1.0에서 더해진 것을 쓰는 새 코드입니다.

```sh
cargo run -p er-ch21-semver --example 01_compatible_changes
```

```text
[같은 사용자 코드를 parcel 1.0.0으로 컴파일]
  번개 퀵 1200g → 3120원
  번개 퀵 800g → 3080원
  무게 합계: 1.2kg + 0.8kg
[같은 사용자 코드를 parcel 1.1.0으로 컴파일]
  번개 퀵 1200g → 3120원
  번개 퀵 800g → 3080원
  무게 합계: 1.2kg + 0.8kg
두 결과가 같은가: true
[1.1.0에서 새로 쓴 코드]
  6400g, 무거움 true, 깨지기 쉬움 true, 예상 3일
  무게 합계: 1.5kg + 2.0kg
  심야 우편: 7000원
```

마지막 구역의 코드는 1.0.0으로 컴파일되지 않으므로, 이런 코드를 쓰기 시작한 사용자는 요구 사항을 `"1.1"`로 올려야 합니다(2.2절).

### 02_silent_behavior_change — 컴파일러가 잡지 못하는 변경

```sh
cargo run -p er-ch21-semver --example 02_silent_behavior_change
```

```text
[1.0.0]
  배송 순서: 1일 오전, 1일 저녁, 2일 오전, 2일 오후
  마지막 배송 안내: 2일 오후
  라이브러리의 순서 검사: 통과
[1.0.1 비권장: 필드만 재배치]
  배송 순서: 1일 오전, 2일 오전, 2일 오후, 1일 저녁
  마지막 배송 안내: 1일 저녁
  라이브러리의 순서 검사: 실패
[1.0.1 권장: 재배치 + 손으로 쓴 Ord]
  배송 순서: 1일 오전, 1일 저녁, 2일 오전, 2일 오후
  마지막 배송 안내: 2일 오후
  라이브러리의 순서 검사: 통과
```

### 03_breaking_change_must_break — 동작이 바뀌면 타입도 바꾸기

```sh
cargo run -p er-ch21-semver --example 03_breaking_change_must_break
```

```text
[1.x] 옛 코드: 7번 보관함, 72시간 보관
[2.0.0 비권장] 옛 코드가 그대로 컴파일됨: 7번 보관함, 3시간 보관
[2.0.0 권장] 옛 코드는 E0308, 고친 코드: 7번 보관함, 72시간 보관
```

`main`의 주석 `// user_code_1x!(v2_good)은 …`을 `let _ = user_code_1x!(v2_good);`으로 바꾸면 2.6절의 E0308을 직접 볼 수 있습니다.

### 04_deprecate_then_remove — 예고한 뒤 지우기

```sh
cargo run -p er-ch21-semver --example 04_deprecate_then_remove
```

```text
[옛 코드 + 1.0.0] KR-1203 → Some(InTransit), kr1203 → None, KR-9999 → None
[옛 코드 + 1.1.0] KR-1203 → Some(InTransit), kr1203 → None, KR-9999 → None
[옮긴 코드 + 1.1.0] KR-1203 → InTransit, kr1203 → 형식 오류, KR-9999 → 미등록
[옮긴 코드 + 2.0.0] KR-1203 → InTransit, kr1203 → 형식 오류, KR-9999 → 미등록
```

옛 코드는 1.1.0에서도 같은 결과를 냅니다. `#[expect(deprecated, ..)]` 속성을 지우면 2.6절의 경고가, 주석 `// old_user_code!(v2_0)은 …`을 코드로 바꾸면 E0425가 나옵니다.

### projects/version_requirements — Cargo가 고르는 버전

`Cargo.toml`이 사용자 패키지 `shop`이고, `vendor/`가 연습용 레지스트리입니다. 저장소에 함께 둔 `Cargo.lock`에는 기본 요구 사항 `"1.4"`로 고른 1.6.0이 적혀 있으므로, 처음 실행하면 고르는 과정 없이 1.6.0을 씁니다. 요구 사항을 바꿨다면 **`Cargo.lock`을 지우고** 실행합니다. 지우지 않으면 Cargo는 적혀 있는 버전이 새 요구 사항도 만족하는 한 그대로 씁니다(`Cargo.lock`의 동작은 Part 0 ch2 2.3절, 자세한 관리는 [ch25](../ch25-dependency-graph/README.md)).

```sh
cd appd-a/ch21-semver/projects/version_requirements
rm Cargo.lock
cargo run
```

```text
     Locking 1 package to highest Rust 1.100.0 compatible version
      Adding parcel v1.6.0 (available: v2.0.0)
   Compiling parcel v1.6.0
   Compiling shop v0.1.0 (<저장소>/appd-a/ch21-semver/projects/version_requirements)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in (시간 생략)
     Running `target/debug/shop`
Cargo가 고른 parcel: 1.6.0
1200g 배송비: 3000원
```

새로 만든 `Cargo.lock`의 `parcel` 항목에는 `source = "registry+https://github.com/rust-lang/crates.io-index"`가 적힙니다. 디렉터리 소스는 crates.io를 "대신"하는 것이라 원래 소스의 이름을 그대로 쓰기 때문입니다. 2.2절의 표는 `Cargo.toml`의 `parcel = "1.4"`를 바꿀 때마다 `Cargo.lock`을 지우고 `cargo run`을 실행해 얻었습니다.

### projects/non_exhaustive — 크레이트 경계 밖의 `#[non_exhaustive]`

두 패키지(`parcel`, `shop`)의 워크스페이스입니다. `shop`의 의존성 `parcel = { path = "../parcel", version = "1.1" }`처럼 path 의존성에 `version`을 함께 적으면 Cargo가 그 경로의 패키지 버전이 요구 사항에 맞는지 검사합니다(4.5절).

```sh
cd appd-a/ch21-semver/projects/non_exhaustive
cargo run -p shop
```

```text
KR-0001: 주문이 접수되었습니다
KR-1203: 상품이 이동 중입니다
KR-0042: 배송이 끝났습니다
KR-0777: 배송 상태가 바뀌었습니다(반송)
서명 true, 배송비 3500원 (빠른 배송 4500원)
```

`cargo clippy --all-targets -- -D warnings`는 경고 없이 끝납니다. `broken_shop`은 워크스페이스에서 뺀 독립 패키지이며, 일부러 컴파일에 실패합니다(4.5절).

```sh
cd broken_shop
cargo build    # E0639 2개, E0638 1개, E0004 1개로 실패 (종료 코드 101)
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 파일마다 `mod parcel`을 다른 사람이 관리하는 라이브러리의 1.1.0으로, 나머지를 1.0.0 시절에 쓴 사용자 코드로 봅니다. 즉 "이 변경을 마이너로 냈다면 사용자가 보게 될 에러"입니다. 챕터 디렉터리에서 확인합니다.

```sh
cd appd-a/ch21-semver
rustc --edition 2024 compile_fail/e0119_new_blanket_impl.rs --out-dir /tmp/er-ch21
rustc --edition 2024 compile_fail/e0038_new_generic_trait_method.rs --out-dir /tmp/er-ch21
rustc --edition 2024 compile_fail/e0277_tightened_bound.rs --out-dir /tmp/er-ch21
rustc --edition 2024 compile_fail/e0277_removed_trait_impl.rs --out-dir /tmp/er-ch21
```

### 4.1 공개 트레이트에 포괄 구현을 더함 — E0119

파일: [`compile_fail/e0119_new_blanket_impl.rs`](compile_fail/e0119_new_blanket_impl.rs)

```text
error[E0119]: conflicting implementations of trait `Label` for type `Ticket`
  --> compile_fail/e0119_new_blanket_impl.rs:47:1
   |
25 |     impl<T: Display> Label for T {
   |     ---------------------------- first implementation here
...
47 | impl Label for Ticket {
   | ^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `Ticket`
```

- **읽는 법**: 같은 타입 `Ticket`에 같은 트레이트 `Label`의 구현이 둘(라이브러리의 포괄 구현 25행, 사용자의 구현 47행)이라는 뜻입니다. `Ticket`이 `Display`를 구현하므로 포괄 구현의 조건을 만족합니다.
- **원인**: 사용자는 1.0.0 시절 아무 문제 없는 구현을 썼고, 라이브러리가 1.1.0에서 그와 겹치는 구현을 더했습니다. 사용자의 `[취급주의]` 문구를 지키면서 고칠 방법이 사용자 쪽에는 없습니다.
- **해결**: 라이브러리가 고칩니다. 이미 공개한 트레이트에 포괄 구현을 더하는 일은 메이저 버전에서 하고, 마이너에서는 라이브러리 자신의 타입에 대한 구현만 더합니다(2.3절).

### 4.2 공개 트레이트에 제네릭 기본 메서드를 더함 — E0038

파일: [`compile_fail/e0038_new_generic_trait_method.rs`](compile_fail/e0038_new_generic_trait_method.rs)

```text
error[E0038]: the trait `Notifier` is not dyn compatible
  --> compile_fail/e0038_new_generic_trait_method.rs:47:31
   |
47 |     let channels: Vec<Box<dyn Notifier>> = vec![Box::new(Sms), Box::new(Mail)]; // E0038
   |                               ^^^^^^^^ `Notifier` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
...
22 |         fn notify_all<I: IntoIterator<Item = String>>(&self, messages: I) -> Vec<String> {
   |            ^^^^^^^^^^ ...because method `notify_all` has generic type parameters
   = help: consider moving `notify_all` to another trait
```

- **읽는 법**: 에러는 사용자 코드(47행)에서 나지만, 원인은 `note`가 가리키는 라이브러리의 새 메서드(22행)입니다. 기본 구현이 있어서 사용자의 `impl`은 깨지지 않았는데, 트레이트 객체로 쓰던 곳이 깨졌습니다.
- **원인**: 타입 매개변수가 있는 메서드는 vtable에 넣을 수 없으므로 트레이트 전체가 dyn 호환이 아니게 됩니다. [ch12](../ch12-generics-vs-trait-objects/README.md) 4.2절의 E0038은 슈퍼트레이트 `Clone` 때문이었고, 여기서는 나중에 더한 메서드 하나 때문입니다.
- **해결**: 라이브러리가 새 메서드에 `where Self: Sized`를 붙이면 트레이트는 dyn 호환으로 남고 마이너 변경이 됩니다(ch12 2.6절). 이렇게 고친 판이 컴파일되는 것을 같은 모양의 코드로 확인했습니다. `help`의 "다른 트레이트로 옮기라"도 방법입니다.

### 4.3 바운드를 조임 — E0277

파일: [`compile_fail/e0277_tightened_bound.rs`](compile_fail/e0277_tightened_bound.rs)

```text
error[E0277]: the trait bound `f64: Ord` is not satisfied
  --> compile_fail/e0277_tightened_bound.rs:32:38
   |
32 |     println!("{}", parcel::summarize(&weights)); // E0277
   |                    ----------------- ^^^^^^^^ the trait `Ord` is not implemented for `f64`
   |                    |
   |                    required by a bound introduced by this call
...
note: required by a bound in `summarize`
  --> compile_fail/e0277_tightened_bound.rs:21:35
   |
21 |     pub fn summarize<T: Display + Ord>(weights: &[T]) -> String {
   |                                   ^^^ required by this bound in `summarize`
```

- **읽는 법**: `note`가 가리키는 `Ord`가 1.1.0에서 더한 바운드입니다. `f64`는 NaN 때문에 `Ord`를 구현하지 않습니다([ch10](../ch10-standard-traits/README.md) 2.5절).
- **원인**: 바운드를 조이는 것은 "받던 것을 덜 받는" 변경입니다. 이름과 인자 수가 그대로라 작은 변경처럼 보입니다. [ch2](../ch2-express-common-behavior/README.md) 2.3절의 `FnMut` → `Fn`도 같은 원리입니다. 반대로 바운드를 느슨하게 하는 것은 예제 01처럼 마이너입니다.
- **해결**: 기존 함수의 바운드는 그대로 두고, 정렬하는 판을 새 함수로 더합니다(연습 4). 새 함수 추가는 마이너입니다.

### 4.4 트레이트 구현을 뗌 — E0277

파일: [`compile_fail/e0277_removed_trait_impl.rs`](compile_fail/e0277_removed_trait_impl.rs)

```text
error[E0277]: the trait bound `TrackingNo: Hash` is not satisfied
  --> compile_fail/e0277_removed_trait_impl.rs:37:18
   |
37 |         *lookups.entry(TrackingNo::new(no)).or_insert(0) += 1; // E0277
   |                  ^^^^^ ------------------- `TrackingNo` doesn't satisfy the trait bound
   |                  |
   |                  the trait `Hash` is not implemented for `TrackingNo`
   |
note: required by a bound in `HashMap::<K, V, S, A>::entry`
help: consider annotating `TrackingNo` with `#[derive(Hash)]`
```

- **읽는 법**: `HashMap`의 키에 필요한 `Hash`가 `TrackingNo`에 없다는 뜻입니다. `help`는 라이브러리 쪽 선언에 `#[derive(Hash)]`를 붙이라고 하는데, 이 파일이 한 크레이트라서 나오는 제안일 뿐 실제로는 사용자가 고칠 수 없는 다른 크레이트의 타입입니다.
- **원인**: 라이브러리 안에서는 해시를 쓰지 않아 라이브러리의 테스트는 모두 통과합니다. 떼어 낸 구현을 쓰던 코드는 모두 라이브러리 밖에 있습니다. 구현을 떼는 것은 공개 항목을 지우는 것과 같습니다(2.3절).
- **해결**: 공개 타입에 붙인 트레이트 구현은 메이저 버전이 아니면 떼지 않습니다. 붙일 때 "계속 약속할 수 있는가"를 따지는 것이 [ch10](../ch10-standard-traits/README.md) 2.9절의 체크리스트입니다. `Copy`를 떼면 E0382가 된다는 것은 ch10 2.3절에서 봤습니다.

### 4.5 그 밖의 에러

모두 이 챕터의 예제와 프로젝트를 그대로, 또는 표에 적은 대로 고쳐서 1.100.0에서 확인했습니다. 경로는 `<저장소>/…`로 바꿔 적었습니다.

| 상황 | 결과 | 다룬 곳 |
| --- | --- | --- |
| `#[non_exhaustive]` 열거형을 크레이트 밖에서 `_` 없이 `match` | E0004 ``non-exhaustive patterns: `_` not covered``, ``note: `Status` is marked as non-exhaustive, so a wildcard `_` is necessary to match exhaustively`` | 2.5절, `broken_shop` |
| `#[non_exhaustive]` 구조체를 크레이트 밖에서 리터럴(갱신 문법 포함)로 만듦 | E0639 `cannot create non-exhaustive struct using struct expression` | 2.5절, `broken_shop` |
| 같은 구조체를 `..` 없이 구조 분해 | E0638 `` `..` required with struct marked as non-exhaustive `` (help: `..`를 끝에 더하라) | 2.5절, `broken_shop` |
| 메이저에서 지운 함수를 부름 | E0425 ``cannot find function `track` in module `v2_0` `` | 2.6절, 예제 04 |
| 단위를 타입에 담은 함수에 옛 인자를 넘김 | E0308 ``expected `Duration`, found integer`` | 2.6절, 예제 03 |
| `parcel = "*"`로 2.0.0이 골라짐 | E0308 ``expected `Grams`, found integer`` | 2.2절 |
| 어느 버전도 맞지 않는 요구 사항(`"1.7"`) | ``error: failed to select a version for the requirement `parcel = "^1.7"` ``, `candidate versions found which didn't match: 2.0.0, 1.6.0, 1.4.12, ...` | 아래 설명 |
| path 의존성의 `version = "1.2"`가 그 경로의 패키지(1.1.0)와 맞지 않음 | ``error: failed to select a version for the requirement `parcel = "^1.2"` ``, `candidate versions found which didn't match: 1.1.0`, `location searched: <저장소>/…/parcel` | `projects/non_exhaustive` |

맞는 버전이 없을 때의 에러는 요구 사항을 `^1.7`로 다시 적어 보여 줍니다. 앞에 아무것도 없는 요구 사항이 캐럿이라는 것을 Cargo가 직접 확인해 주는 셈입니다. 연습용 레지스트리에서는 `note: perhaps a crate was updated and forgotten to be re-vendored?`가 덧붙는데, 디렉터리 소스이기 때문에 나오는 안내이므로 무시합니다.

## 5. 연습 문제

### 연습 1. 새 마이너 API를 쓰기 시작하면

[`projects/version_requirements`](projects/version_requirements/Cargo.toml)의 `src/main.rs`에 1.6.0에서 생긴 `parcel::fee_with_insurance(1200, 50000)`의 결과를 출력하는 줄을 더하세요. 요구 사항이 `"1.4"`일 때와 `"~1.4"`일 때 각각 `Cargo.lock`을 지우고 `cargo run`을 실행해 결과를 비교하고, 요구 사항을 어떻게 고쳐야 하는지 정하세요.

<details>
<summary>힌트</summary>

`"1.4"`에서는 1.6.0이 골라져 `3500`(3000원 + 가액의 1%인 500원)이 출력됩니다. `"~1.4"`에서는 1.4.12가 골라지고 1.4.12에는 그 함수가 없어서 ``error[E0425]: cannot find function `fee_with_insurance` in crate `parcel` ``이 납니다. `"~1.4"`는 다른 사람의 `Cargo.lock`이 1.4.x를 고른 상황을 흉내 낸 것입니다. 요구 사항을 `"1.6"`으로 올리면 두 상황 모두 해결됩니다(2.2절).

</details>

### 연습 2. 몇 번째 자리를 올릴까

`parcel` 1.3.0을 낸 뒤 다음 변경을 하려 합니다. 각각 다음 버전을 1.3.1, 1.4.0, 2.0.0 중 무엇으로 내야 할지 정하세요.

1. 내부 함수의 성능을 개선했고 결과는 같다.
2. `#[non_exhaustive]`가 붙은 `Status`에 `Lost` 배리언트를 더한다.
3. 비공개 필드가 없는 `pub struct Address { pub city: String }`에 `pub zip: String`을 더한다.
4. 공개 트레이트 `Carrier`에 기본 구현이 없는 메서드 `tracking_url`을 더한다.
5. `pub fn fee(weight_g: u32)`를 `pub fn fee(weight_g: u64)`로 바꾼다.
6. 공개 함수 `cancel`에 `#[deprecated]`를 붙이고 대신 쓸 `cancel_order`를 더한다.

<details>
<summary>힌트</summary>

1은 1.3.1, 2와 6은 1.4.0, 3·4·5는 2.0.0입니다. 3은 구조체 리터럴이 E0063, 4는 사용자의 모든 `impl`이 E0046으로 깨집니다. 5는 `fee(w)`의 `w`가 `u32` 변수인 사용자 코드가 E0308로 깨집니다(정수 리터럴만 넘기던 사용자는 그대로 컴파일되므로 "모든 사용자가 깨지지는 않지만 일부가 깨지는" 변경도 메이저입니다). 2는 `shop`처럼 `_` 갈래를 둔 사용자 코드가 그대로 컴파일되지만, 새 배리언트가 `_` 갈래로 흘러간다는 동작의 변화는 변경 기록에 적어 둡니다.

</details>

### 연습 3. 배리언트와 필드 하나 더 늘리기

[`projects/non_exhaustive`](projects/non_exhaustive/Cargo.toml)의 `parcel`을 1.2.0으로 올리고, `Status`에 `Lost`(분실) 배리언트를, `Options`에 `pub night_delivery: bool` 필드와 세터를 더하세요. `status_of`가 `"KR-4040"`에 `Lost`를 돌려주게 하고, `shop`을 한 글자도 고치지 않은 채 `cargo run -p shop`이 되는지 확인하세요. 그다음 `shop`의 `Cargo.toml` 요구 사항이 `"1.1"`인데 1.2.0이 쓰이는 이유를 설명하세요.

<details>
<summary>힌트</summary>

`parcel` 안에서는 `label`의 `match`가 ``E0004 non-exhaustive patterns: `Status::Lost` not covered``로 처리할 곳을 알려 줍니다(크레이트 안에서는 속성이 효과가 없으므로). `express`의 구조체 리터럴에도 새 필드를 채워야 합니다. `shop`은 `_` 갈래와 `Options::new()`를 쓰므로 그대로 컴파일되고, `shop/src/main.rs`의 송장 목록에 `"KR-4040"`을 더하면 `KR-4040: 배송 상태가 바뀌었습니다(분실)`이 나옵니다. `"1.1"`은 `>=1.1.0, <2.0.0`이므로 1.2.0을 허용합니다. `broken_shop`은 여전히 실패합니다. 이제는 `parcel`에서 두 속성을 지워도 `Lost`가 빠진 `match`(E0004), `night_delivery`가 빠진 리터럴(E0063)과 패턴(E0027) 때문에 실패합니다. 속성 없이 1.0.0을 냈다면 1.2.0이 메이저였어야 하는 이유입니다.

</details>

### 연습 4. 바운드를 조이지 않고 기능 더하기

[`compile_fail/e0277_tightened_bound.rs`](compile_fail/e0277_tightened_bound.rs)를 복사해, `summarize`의 바운드를 1.0.0의 `T: Display`로 되돌리고 정렬하는 판 `summarize_sorted`를 새로 더한 1.1.0으로 고치세요. 사용자 코드의 `f64` 무게는 `summarize`로, 정수 kg 무게(`u32`)는 `summarize_sorted`로 출력되게 하세요.

<details>
<summary>힌트</summary>

`pub fn summarize_sorted<T: Display + Ord>(weights: &[T]) -> String`의 본문은 파일의 1.1.0 `summarize` 본문을 그대로 쓰면 됩니다. `parcel::summarize(&[1.2_f64, 0.8, 2.5])`는 `1.2kg + 0.8kg + 2.5kg`, `parcel::summarize_sorted(&[3_u32, 1, 2])`는 `1kg + 2kg + 3kg`을 냅니다. `f64`도 정렬하고 싶다면 `total_cmp`로 정렬하는 별도 함수를 둡니다(ch10 2.5절).

</details>

## 6. 참고 자료

- 기준 문서: [Item 21: Understand what semantic versioning promises](https://effective-rust.com/semver.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- [Semantic Versioning 2.0.0](https://semver.org/): 세 자리의 규칙, 낸 버전을 고치지 않는다는 규칙, 0.y.z, deprecated 표시를 마이너로 낸다는 FAQ
- The Cargo Book
  - [SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html): 호환되지 않는 변경의 전체 목록(메이저, 마이너, 깨질 수도 있음). 이 챕터가 든 항목은 `enum-variant-new`, `attr-adding-non-exhaustive`, `generic-bounds-tighten`, `trait-object-safety`, `trait-new-default-item`, `env-new-rust` 등입니다.
  - [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html): 캐럿·`~`·`*`·비교 요구 사항의 범위 표, path 의존성에 `version`을 함께 적을 때의 검사
  - [Source Replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html): `projects/version_requirements`가 쓴 디렉터리 소스와 `replace-with`
  - [Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html): 낸 버전은 덮어쓸 수 없다는 것과 게시 취소(yank)
- [Rust API Guidelines: Future proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html): 봉인된 트레이트(C-SEALED), 비공개 필드(C-STRUCT-PRIVATE) 등 나중에 바꿀 여지를 남기는 설계
- [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks): 두 릴리스의 공개 API를 비교해 SemVer 위반을 찾는 도구(이 저장소에서는 실행해 확인하지 못함)
- 레퍼런스: [The `non_exhaustive` attribute](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute), [The `deprecated` attribute](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-deprecated-attribute)
- clippy: [`incompatible_msrv`](https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#incompatible_msrv)
- 에러 설명: `rustc --explain E0119`, `rustc --explain E0038`, `rustc --explain E0277`, `rustc --explain E0639`, `rustc --explain E0638`
