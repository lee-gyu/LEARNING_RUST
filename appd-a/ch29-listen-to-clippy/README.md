# ch29. Clippy의 경고에 귀 기울이기

Clippy는 컴파일러가 통과시킨 코드에서 흔한 실수와 어색한 작성법을 찾아 경고하는 도구입니다. 경고가 늘 맞지는 않으므로, 경고를 받을 때마다 "고칠까, 다르게 쓸까, 이번만은 예외로 둘까"를 판단해야 합니다. 이 챕터는 그 판단을 다룹니다. 경고 설명을 읽고 세 가지 처리 가운데 하나를 고르는 법, 예외를 이유와 함께 남기는 법, 경고 0개를 유지해야 하는 이유, 기본으로 꺼진 린트 그룹을 어디까지 켤지, 그리고 그 결정을 프로젝트 단위로 설정하는 법을 실제 경고 출력으로 확인합니다.

- 원제: [Item 29: Listen to Clippy](https://effective-rust.com/clippy.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch29-listen-to-clippy`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, clippy 0.1.100 (2024 에디션)

책의 5장(Tooling)에 속한 Item입니다. 다음 본편과 부록 내용을 안다고 보고 진행합니다.

- [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md): `cargo clippy`로 Clippy를 부르는 법
- [ch5](../ch5-type-conversions/README.md) 2.4절: 기본으로 꺼진 `as` 캐스트 린트(`cast_possible_truncation` 등)
- [ch16](../ch16-avoid-unsafe/README.md) 2.5~2.6절: `#![deny(unsafe_code)]`와 모듈 단위 허락, `forbid`와 E0453
- [ch18](../ch18-dont-panic/README.md) 2.6절: restriction 린트를 `[lints.clippy]`와 `clippy.toml`로 켠 [`projects/panic_free_lib`](../ch18-dont-panic/projects/panic_free_lib)
- [ch22](../ch22-minimize-visibility/README.md) 2.6절(`unreachable_pub`), [ch23](../ch23-avoid-wildcard-imports/README.md) 2.2절(`wildcard_imports`), [ch26](../ch26-feature-creep/README.md) 2.2·2.5절(`cfg!`, `[lints.rust]`)

린트 수준의 우선순위 규칙 전체(`--force-warn`, `-W`/`-A`의 순서, `--cap-lints`)와 rustc 린트 그룹은 Part 5 `ch3-lints`에서, 매니페스트의 `[lints]` 전반은 [Part 3 ch3](../../part3-cargo/ch3-manifest-targets/README.md)에서, CI 구성은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 다룹니다. 문서 관련 린트의 내용은 [`ch27-document-public-interfaces`](../ch27-document-public-interfaces/README.md)가 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Clippy 경고의 설명 페이지(`cargo clippy --explain`)를 읽고, 그 경고를 고칠지, 작은 리팩터링으로 없앨지, 이유를 적고 예외로 둘지 판단할 수 있다.
2. 예외를 가장 좁은 범위에 `#[expect(…, reason = "…")]`로 표시하고, `#[allow]`보다 나은 이유와 `#[expect]`를 쓰면 안 되는 경우(빌드 설정에 따라 나는 경고)를 설명할 수 있다.
3. 경고 0개를 유지해야 하는 이유를 코드 변경과 Clippy 업그레이드의 예로 설명하고, Clippy가 잡지 못하는 버그도 있다는 것을 알 수 있다.
4. Clippy의 린트 그룹 아홉 개와 기본 수준을 구별하고, pedantic은 그룹째 켜되 맞지 않는 것을 끄고 restriction은 골라 켜는 판단을 할 수 있다.
5. `Cargo.toml`의 `[lints]`·`[workspace.lints]`와 `priority`, `clippy.toml`로 린트 설정을 프로젝트 단위로 작성하고, 린트마다 `allow`/`warn`/`deny`/`forbid` 가운데 알맞은 수준을 고를 수 있다.

## 2. 핵심 개념

2.1절에서 Clippy의 그룹과 린트 수준을 정리하고, 2.2~2.6절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 지침 1·3·4는 책이 권하는 것이고, 지침 2·5는 책 이후에 생긴 도구(`#[expect]`, `[lints]`)와 Clippy·Cargo 문서를 근거로 더했습니다(2.7절). 경고 인용은 모두 이 툴체인에서 실행한 출력이며, 위치 줄과 코드 줄, 일부 `help:` 줄을 줄인 곳이 있습니다.

### 2.1 지도: Clippy, 린트 그룹, 린트 수준

**Clippy는 rustc 위에서 도는 린트 모음입니다.** `cargo clippy`는 `cargo check`처럼 코드를 컴파일하되, rustc의 린트에 Clippy의 린트 832개(1.100.0의 `clippy-driver -W help` 기준)를 더해 검사합니다. 그래서 `cargo clippy`의 출력에는 rustc 경고(`unused_variables`, `dead_code` 등)도 함께 나옵니다. 경고 하나는 대체로 이렇게 생겼습니다.

```text
warning: this is a decimal constant
   …
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#zero_prefixed_literal
   = note: `#[warn(clippy::zero_prefixed_literal)]` on by default
help: if you mean to use an octal constant, use `0o`
```

`for further information visit` 줄의 주소는 이 툴체인(1.100.0)의 출력이고 버전마다 다릅니다. 같은 설명을 `cargo clippy --explain zero_prefixed_literal`로 터미널에서도 볼 수 있습니다. 설명은 "What it does", "Why is this bad?", 예와 고친 예로 이루어지고, 린트에 따라 "Known problems"(알려진 오탐)와 "Configuration"(`clippy.toml`로 바꿀 수 있는 값)이 붙습니다. `note:` 줄은 이 린트가 **어디서 켜졌는지**(기본값인지, 속성인지, `Cargo.toml`인지)를 알려 줍니다.

**린트 그룹.** Clippy의 린트는 모두 그룹 하나에 속하고, 그룹마다 기본 수준이 있습니다. 린트 수는 1.100.0의 `clippy-driver -W help`에서 센 것이고, 오른쪽 열은 Clippy 문서의 "Clippy's Lints" 장이 권하는 대응을 요약한 것입니다.

| 그룹 | 기본 | 린트 수 | 잡는 것 | 경고를 받으면 |
| --- | --- | --- | --- | --- |
| `correctness` | deny | 67 | 틀렸거나 쓸모없는 코드 | 고칩니다. `#[allow]`는 권하지 않음 |
| `suspicious` | warn | 86 | 거의 틀림없이 실수인 코드 | 고칩니다. 일부러 그렇게 썼다면 그 자리만 이유와 함께 예외 |
| `style` | warn | 159 | 관용적이지 않은 코드 | 대개 따릅니다. 가장 주관적인 기본 그룹이라 동의하지 않는 린트는 꺼도 됨 |
| `complexity` | warn | 145 | 단순한 일을 복잡하게 한 코드 | 대개 줄입니다 |
| `perf` | warn | 38 | 더 빠르게 쓸 수 있는 코드 | 대개 적용합니다 |
| `pedantic` | allow | 145 | 깐깐하거나 가끔 오탐이 있는 린트 | 켜려면 `#[allow]`를 넉넉히 쓸 각오로(2.5절) |
| `restriction` | allow | 135 | 언어·라이브러리 기능의 사용 자체를 막는 린트 | 그룹째 켜지 말고 골라 켬(2.5절) |
| `nursery` | allow | 52 | 아직 다듬는 중인 린트 | 골라 켬 |
| `cargo` | allow | 5 | `Cargo.toml`의 문제 | 게시할 크레이트에서 고려 |

기본으로 켜진 다섯 그룹을 합친 그룹 이름이 `clippy::all`입니다(495개). 1.100.0의 `-W help`는 이 그룹을 `clippy::default`라는 이름으로 보여 주지만(1.98.1은 `clippy::all`로 보여 줌), 1.100.0은 두 이름을 모두 받아들이고 1.98.1은 `clippy::default`를 `unknown lint`로 경고했습니다. Clippy 문서도 `clippy::all`로 설명하므로 이 챕터는 `clippy::all`이라고 부릅니다.

**린트 수준.** 린트마다 수준을 정할 수 있습니다. 이 챕터에서 쓰는 다섯 가지입니다(rustc Book의 Lint Levels 절, `force-warn`은 Part 5 `ch3-lints`).

| 수준 | 동작 | 정하는 곳 |
| --- | --- | --- |
| `allow` | 아무 말도 하지 않음 | 속성, `[lints]`, 명령줄(`-A`) |
| `expect` | `allow`처럼 막되, 그 범위에서 경고가 **나지 않으면** `unfulfilled_lint_expectations` 경고 | 속성(`#[expect]`)만 |
| `warn` | 경고 | 속성, `[lints]`, 명령줄(`-W`) |
| `deny` | 에러. 안쪽 범위에서 다시 낮출 수 있음 | 속성, `[lints]`, 명령줄(`-D`) |
| `forbid` | 에러. 크레이트 안의 어디서도 낮출 수 없음 | 속성, `[lints]`, 명령줄(`-F`) |

모든 린트 속성에는 `reason = "…"`을 덧붙일 수 있고, 그 수준으로 린트가 날 때 이유가 `note:`로 함께 출력됩니다(2.3절).

### 2.2 지침 1: 경고마다 설명을 읽고, 고칠지·리팩터링할지·예외로 둘지 고른다

**문제 상황.** 경고를 받으면 두 가지 반사 행동이 나오기 쉽습니다. help가 제안한 대로 기계적으로 고치거나, "이건 오탐"이라며 `#[allow]`로 덮는 것입니다. 책은 경고마다 연결된 설명 페이지를 읽고 그 이유가 내 코드에 해당하는지 판단하라고 권합니다. 판단의 결과는 셋 중 하나입니다.

**비권장 코드와 권장 코드.** [`01_judge_each_warning.rs`](examples/01_judge_each_warning.rs)는 기본 설정의 Clippy가 경고하는 코드 세 개를 서로 다르게 처리합니다.

**① 진짜 실수 → 고칩니다.** 파일 권한을 `const MODE: u32 = 0644;`로 적었습니다. Clippy는 `clippy::zero_prefixed_literal`(complexity)로 경고하고, help 두 개를 냅니다.

```text
warning: this is a decimal constant
help: if you mean to use a decimal constant, remove the `0` to avoid confusion
help: if you mean to use an octal constant, use `0o`
```

설명 페이지는 C 계열 언어에서는 `0`으로 시작하는 정수가 8진수지만 Rust에서는 10진수라고 적고 있습니다. 실행해 보면 `MODE`는 8진수로 `0o1204`이고 그룹 읽기 비트가 꺼져 있습니다. help 두 갈래 가운데 뜻에 맞는 `0o644`를 고르는 것은 사람의 몫입니다. 그래서 `cargo clippy --fix`도 이 경고는 고치지 않습니다(아래).

**② 취향처럼 보이는 경고 → 작은 리팩터링으로 없앱니다.** 회원 등급별 할인 규칙을 `BTreeMap<&'static str, Vec<Box<dyn Fn(u32) -> Option<u32>>>>`로 주고받는 두 함수에 `clippy::type_complexity`(complexity)가 `` very complex type used. Consider factoring parts into `type` definitions ``로 경고합니다. "이 정도는 복잡하지 않다"고 따질 수도 있지만, 책은 오탐인지 따지는 데 시간을 쓰기보다 작은 리팩터링으로 경고를 없애는 편이 대개 낫다고 말합니다. `type Rule = Box<dyn Fn(u32) -> Option<u32>>;`와 `type RuleTable = BTreeMap<&'static str, Vec<Rule>>;` 두 줄로 경고가 사라지고, 시그니처 `fn price(table: &RuleTable, grade: &str, base: u32)`도 읽기 쉬워집니다. 동작은 그대로입니다(3절의 2번 구역).

**③ 의도한 코드 → 이유를 적고 예외로 둡니다.** 이번에는 경고를 따른 쪽이 비권장입니다. 512바이트 본문을 담는 `Data`와 2바이트 `Ack`를 가진 열거형에 `clippy::large_enum_variant`(perf)가 경고하며 큰 필드를 `Box`에 넣으라고 제안합니다. 그런데 `cargo clippy --explain large_enum_variant`의 Known problems는 이렇게 말합니다. 이 린트는 실행 중 배리언트의 분포를 알 수 없고, 작은 배리언트가 드물면 `Box`가 오히려 손해이니 늘 측정해 보라는 것입니다. 이 프로토콜은 프레임의 99%가 `Data`입니다. 실행하면 help대로 바꾼 판은 프레임 1000개에 상자 990개(힙 할당 990번)를 만들고 바이트도 더 씁니다(522880 대 514000). 그래서 권장 판은 열거형을 그대로 두고 예외를 표시합니다.

```rust
#[expect(
    clippy::large_enum_variant,
    reason = "프레임의 99%가 Data라서 Box에 넣으면 할당만 늘고 메모리는 줄지 않음"
)]
pub enum Frame {
    Data([u8; PAYLOAD]),
    Ack(u16),
}
```

**그룹이 판단의 출발점입니다.** 2.1절 표의 오른쪽 열처럼, correctness와 suspicious 경고는 거의 언제나 ①이고, style·complexity는 ①이나 ②, pedantic과 restriction은 ③이 상대적으로 많습니다. 그래도 최종 판단은 설명을 읽고 내 코드에 맞춰 합니다.

**`cargo clippy --fix`는 판단을 대신하지 않습니다.** `--fix`는 기계적으로 적용해도 된다고 표시된 제안만 고칩니다. [`projects/lint_config`](projects/lint_config)의 사본에서 `cargo clippy --fix --allow-no-vcs -p before-strict`를 실행하자 경고 18개 가운데 7개(`wildcard_imports` 1, `unreachable_pub` 2, `cast_lossless` 3, `allow_attributes` 1)만 고쳤고, 그중 `#[allow(dead_code)]`는 `#[expect(dead_code)]`로 바꾸기만 해서 이유가 없다는 경고는 남았습니다. 나머지 11개는 사람이 정해야 합니다. 또 cargo 1.100.0에서 `--fix`는 다른 빌드 명령처럼 기본 타깃만 고쳤습니다(예제 파일은 그대로였음). Clippy 문서의 Usage 장은 `--fix`가 `--all-targets`를 함께 뜻한다고 적고 있지만, 1.89.0 릴리스 노트는 `cargo fix`와 `cargo clippy --fix`가 다른 빌드 명령과 같은 타깃을 고르도록 바뀌었다고 적고 있고 실제 동작도 그랬습니다. 예제와 테스트까지 고치려면 `--all-targets`를 붙입니다. 커밋하지 않은 변경이 있으면 `the working directory of this package has uncommitted changes`로 멈추므로(종료 코드 101), 커밋한 뒤 실행해서 바뀐 곳을 `git diff`로 검토합니다.

**따르지 않아도 되는 경우.** 오탐이 분명하고 리팩터링이 오히려 코드를 해칠 때는 ③으로 갑니다. 이 챕터에서 실제로 만난 오탐은 pedantic의 `doc_markdown`입니다. `projects/lint_config`의 예제 프로그램 문서 주석에 식별자를 역따옴표 없이 "print_stdout만 크레이트 루트에서 예외로 둡니다"라고 썼더니, `item in documentation is missing backticks`로 경고하며 조사까지 묶은 `` `print_stdout만` ``을 제안했습니다. 한국어 조사가 붙은 식별자를 하나의 단어로 본 것입니다. 이 경우는 식별자만 역따옴표로 감싸는 작은 수정으로 끝났지만, 고칠 방법이 없는 오탐이라면 이유를 적고 예외로 둡니다.

### 2.3 지침 2: 예외는 가장 좁은 범위에, 이유와 함께, `#[expect]`로 둔다

**문제 상황.** 예외를 두기로 했다면 어디에, 어떻게 둘지가 남습니다. 책은 항목에 붙이는 `#[allow(clippy::some_lint)]`와 크레이트 전체에 붙이는 `#![allow(…)]`를 소개합니다. 범위가 넓을수록 나중에 같은 범위에 들어온 진짜 문제까지 덮습니다.

**비권장 코드.** [`02_expect_with_reason.rs`](examples/02_expect_with_reason.rs)는 파일 첫머리에서 pedantic 린트 `cast_possible_truncation`을 켜 둡니다. `v1` 모듈에는 진행률을 계산하는 의도한 캐스트가 있어서 모듈 전체에 이유 없는 `#![allow(clippy::cast_possible_truncation)]`를 붙였습니다. 같은 모듈에는 항목 수를 1바이트 헤더에 적는 `items.len() as u8`도 있습니다. 항목 300개를 넣으면 헤더에 44가 적히고 읽는 쪽은 44개만 되찾지만, 경고는 하나도 없습니다. `#![allow]`를 지우면 두 캐스트 모두에 경고가 나고, 파일 첫머리의 `reason`이 `note:`로 붙습니다.

```text
warning: casting `usize` to `u8` may truncate the value
   = note: 길이를 1바이트 헤더에 담는 코드가 있어 잘림을 놓치지 않으려고 켬
note: the lint level is defined here
help: ... or use `try_from` and handle the error accordingly
```

**권장 코드.** 예외는 의도한 캐스트가 있는 `let` 문 하나에만, 이유와 함께 둡니다. 의도하지 않은 캐스트는 경고를 받아 `u8::try_from`으로 고쳤습니다.

```rust
let done = done.min(total);
#[expect(
    clippy::cast_possible_truncation,
    reason = "done을 total 이하로 묶었으므로 결과는 0..=100"
)]
let percent = (u64::from(done) * 100 / u64::from(total)) as u8;
percent
```

린트 속성은 식이 아니라 문장과 항목에 붙으므로 결과를 `let` 문으로 받았습니다. 속성이 없으면 이 모양에 `clippy::let_and_return`이 경고하지만, 속성이 붙은 `let` 문에는 경고하지 않았습니다. 좁은 순서는 문장 → 함수 → 모듈 → 크레이트이고, 넓힐수록 "이 범위에 앞으로 들어올 모든 코드"에 대한 예외가 됩니다.

**`#[allow]` 대신 `#[expect]`인 이유.** 나중에 누군가 이 캐스트를 `u8::try_from(…).unwrap_or(100)`으로 바꿨다고 합시다. `#[allow]`였다면 쓸모없어진 속성이 영영 남지만, `#[expect]`는 기대한 경고가 나지 않았다고 알려 줍니다(사본을 고쳐 확인, 위치 줄 생략).

```text
warning: this lint expectation is unfulfilled
   |
44 |             clippy::cast_possible_truncation,
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: done을 total 이하로 묶었으므로 결과는 0..=100
   = note: `#[warn(unfulfilled_lint_expectations)]` on by default
```

경고를 에러로 올린 빌드에서는 이 알림도 에러가 됩니다(4.3절). restriction 그룹의 `clippy::allow_attributes`를 켜면 바깥 속성 `#[allow]`를 찾아 `` help: replace it with: `expect` ``로 바꾸라고 하고, `clippy::allow_attributes_without_reason`은 `allow`와 `expect`에 `reason`이 없으면 경고합니다. `projects/lint_config`는 둘 다 켭니다(2.6절).

주의할 점이 둘 있습니다.

- **`#[expect]`는 기본으로 꺼진 린트도 그 범위에서 켭니다.** 그래서 `#[expect(clippy::float_cmp)]`가 충족된다고 해서 기본 설정의 Clippy가 그 코드를 잡는다는 뜻은 아닙니다(예제 05가 이 차이를 보입니다).
- **Clippy 린트에 대한 `#[expect]`는 `cargo clippy`를 실행할 때만 검사됩니다.** 충족되지 않은 `#[expect(clippy::unwrap_used)]`를 둔 임시 프로젝트에서 `cargo clippy`는 `unfulfilled` 경고를 냈지만 `cargo build`는 아무 말도 하지 않았습니다. rustc는 Clippy 린트를 모르기 때문입니다.

**따르지 않아도 되는 경우: 경고가 빌드 설정에 따라 나기도 하고 안 나기도 할 때.** [`03_conditional_warnings.rs`](examples/03_conditional_warnings.rs)의 함수들은 `#[cfg(debug_assertions)]`가 붙은 `push` 하나 때문에 `mut`이 필요합니다. 릴리스 빌드에서는 그 문장이 지워져 `unused_mut` 경고가 납니다. 여기에 `#[expect(unused_mut, reason = "…")]`를 붙이면 릴리스 빌드는 조용하지만 디버그 빌드(`cargo clippy`, `cargo build` 모두)에서 `this lint expectation is unfulfilled`가 납니다. 이럴 때의 대안은 셋입니다.

| 대안 | 모양 | 득실 |
| --- | --- | --- |
| 1. `#[allow]`와 이유 | `#[allow(unused_mut, reason = "…")]` | 어느 빌드에서나 조용함. 대신 낡아도 알려 주지 않음 |
| 2. 조건부 `#[expect]` | `#[cfg_attr(not(debug_assertions), expect(unused_mut, reason = "…"))]` | 정확함. 대신 `#[cfg]`의 조건을 바꿀 때 함께 바꿔야 함(연습 2) |
| 3. 경고가 생기지 않게 고치기(권장) | `if cfg!(debug_assertions) { checks.push(…); }` | `cfg!`는 리터럴이 될 뿐 문장을 지우지 않으므로 `mut`이 어느 빌드에서나 쓰임 |

에디션에 따라 나기도 하고 안 나기도 하는 경고도 같습니다([Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md)의 `projects/same_code`가 `#![allow]`를 쓴 이유). 2.6절에서 볼 `allow_attributes`를 켠 프로젝트라면 대안 2나 3을 씁니다.

### 2.4 지침 3: 경고 0개를 유지한다

**문제 상황.** 책이 굵게 강조하는 지침입니다. 경고를 고치든 예외로 두든, 코드가 Clippy 경고 없는 상태여야 합니다. 그래야 코드를 바꾸거나 Clippy를 올려서 새 경고가 생겼을 때 바로 눈에 띕니다.

**비권장 코드: 경고가 쌓인 상태.** `projects/lint_config`의 `before-strict`는 경고 18개를 안고 있습니다(3절). 이 상태에서 누군가 버그 하나를 더 넣어 경고가 19개가 되어도, 요약 줄 `generated 18 warnings`가 `generated 19 warnings`로 바뀔 뿐입니다. 경고가 쌓인 프로젝트에서는 아무도 경고 목록을 읽지 않습니다.

**권장 코드: 0개에서 시작합니다.** [`04_new_lint_on_upgrade.rs`](examples/04_new_lint_on_upgrade.rs)의 `v1::bits_needed`는 `u64` 값을 나타내는 데 필요한 비트 수를 `u32::BITS - x.leading_zeros()`로 계산합니다. `u64` 값인데 `u32`의 비트 수(32)에서 빼므로 틀린 값이 나옵니다(`1 << 40`에 9, 맞는 값은 41). `#[expect]`를 지운 사본을 검사하자, 이 저장소에 함께 설치된 Clippy 1.98.1은 아무 경고도 내지 않았고 1.100.0은 새로 생긴 `clippy::mismatched_bit_width_type`(suspicious, 기본 warn)으로 잡았습니다.

```text
warning: possible buggy implementation of `bit_width`
   |
15 |         u32::BITS - x.leading_zeros()
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: in order to calculate the bit width, `T::BITS` should match the type of the value calling `.leading_zeros()`
help: if you meant to use `u64::BITS`, use
   |
15 -         u32::BITS - x.leading_zeros()
15 +         x.bit_width()
```

1.98.1에서 경고 0개였던 프로젝트라면 1.100.0으로 올린 날 이 경고 하나가 곧바로 보이고, 1.97.0에서 안정화된 `bit_width`로 고치면 됩니다. 두 버전의 `-W help`를 비교하면 1.100.0에서 린트 10개가 새로 생겼고(그중 5개가 기본 warn), `nonstandard_macro_braces`가 allow에서 warn으로 바뀌었습니다. Clippy를 올리는 일은 곧 기본 경고 목록이 바뀌는 일입니다.

**Clippy가 조용하다고 버그가 없는 것은 아닙니다.** 같은 버그를 리터럴로 쓴 `v1_literal`(`32 - x.leading_zeros()`)은 결과가 `v1`과 똑같이 틀리지만, 1.100.0에서 pedantic과 nursery까지 켜도 아무 린트도 잡지 않았습니다. 린트는 정해진 코드 모양을 찾을 뿐입니다. 이런 버그는 테스트가 잡아야 합니다([`ch30-beyond-unit-tests`](../ch30-beyond-unit-tests/README.md)).

**CI에서 0개를 지킵니다.** 책은 CI에서 Clippy를 돌리라고 권합니다. 경고를 실패로 만드는 방법은 둘입니다. 전통적인 `cargo clippy -- -D warnings`와, Cargo 1.97.0에서 안정화된 `build.warnings` 설정(`CARGO_BUILD_WARNINGS=deny cargo clippy` 또는 `.cargo/config.toml`의 `[build] warnings = "deny"`)입니다. `before-strict`에 둘 다 실행하면 종료 코드 101로 실패하고, 뒤의 것은 경고를 경고로 보여 준 뒤 `` error: warnings are denied by `build.warnings` configuration ``으로 끝납니다. Clippy 문서는 `-D warnings`가 빌드 캐시를 무효화하므로 Cargo 1.97 이상에서는 `build.warnings`를 권합니다. CI 구성은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 다룹니다.

같은 이유로 CI의 Clippy 버전을 고정합니다. 예제 04의 `#[expect(clippy::mismatched_bit_width_type, …)]`를 1.98.1 Clippy로 검사하면 그 린트를 몰라 `` unknown lint: `clippy::mismatched_bit_width_type` ``로 경고했습니다. 개발자마다 Clippy 버전이 다르면 경고 0개라는 기준 자체가 흔들립니다.

**따르지 않아도 되는 경우.** 오래된 코드베이스에 린트를 처음 켜면 경고가 수백 개 나올 수 있습니다. 한 번에 0개로 만들 수 없다면 새로 켠 린트를 한동안 `allow`로 두고 모듈 단위로 켜 나가거나, 새 코드에만 적용하는 식으로 기준선을 옮깁니다. 중요한 것은 "지금 켜진 린트에 대해서는 0개"를 지키는 것입니다.

### 2.5 지침 4: 꺼진 린트 목록도 읽고, 쓸모 있는 것은 골라서 켠다

**문제 상황.** 책이 굵게 강조하는 또 하나의 지침은 Clippy 린트 목록을 읽어 보라는 것입니다. 지나치게 깐깐하거나 오탐이 많아 기본으로 꺼진 것까지 포함해서입니다. 켜지 않더라도 Rust 관용구를 익히는 데 도움이 됩니다.

**꺼진 린트가 잡는 버그.** [`05_default_off_lints.rs`](examples/05_default_off_lints.rs)의 두 비권장 판은 기본 설정의 Clippy가 아무 말도 하지 않는 코드입니다. 파일 첫머리의 `#![warn(clippy::float_cmp, clippy::needless_for_each)]`로 pedantic 린트 둘을 골라 켜야 경고가 납니다.

- `float_cmp`: 금액을 `f64`로 더해 `==`로 비교하는 `balanced_v1`은 `0.1 + 0.2 == 0.3`에 `false`를 돌려줍니다. 린트는 `(a - 0.3).abs() < error_margin`처럼 오차 범위로 비교하라고 제안합니다. 금액이라면 가장 작은 단위를 정수로 세는 표현으로 바꾸는 편이 낫습니다(`balanced_v2`).
- `needless_for_each`: 처음 나온 음수에서 멈추려고 `for_each` 클로저 안에 `return`을 쓴 `sum_until_negative_v1`은 음수를 건너뛰고 끝까지 더합니다(`[3, 4, -1, 10]`에 17, 뜻한 값은 7). [ch9](../ch9-iterator-transforms/README.md) 2.5절이 "도구가 해 주지 않는 판단"이라고 했던 실수인데, 이 린트의 help는 `for`로 바꾸면서 `` ...and replace `return` with `continue` ``라고 덧붙입니다. 린트를 켜지 않더라도 이 설명을 읽으면 클로저 안의 `return`이 무엇인지 배우게 됩니다.

**pedantic: 그룹째 켜고 맞지 않는 것을 끕니다.** Clippy 문서는 pedantic을 "의도적인 오탐이 있을 수 있는, 의견이 강한 린트"로 소개하며, 그룹째 켜려면 `#[allow]`를 넉넉히 쓸 각오를 하고 아니면 골라 켜라고 권합니다. `projects/lint_config`는 그룹째 켜고 `must_use_candidate` 하나를 껐습니다. 이 린트는 값을 돌려주는 `pub fn`마다 `#[must_use]`를 권하는데, 설명의 Known problems가 "오탐이 많을 것"이라고 스스로 밝힙니다. 그룹째 켜면 Clippy를 올릴 때 새 pedantic 린트도 자동으로 켜지고(1.100.0에서도 `assert_is_empty`, `manual_bit_width` 두 개가 새로 생김), 골라 켜면 그렇지 않습니다. 어느 쪽이 맞는지는 팀이 새 경고를 감당할 여유에 달렸습니다.

**restriction: 그룹째 켜지 않습니다.** Clippy 문서는 restriction 그룹을 "절대로 통째로 켜지 말라"고 강조합니다. 멀쩡한 코드에도 경고하고, 대안이 없기도 하고, 다른 린트와 모순되기도 하기 때문입니다. `projects/lint_config`의 `after`(경고 0개)에 명령줄로 그룹 전체를 켜 보았습니다.

```sh
cargo clippy -p after -- -W clippy::restriction
```

경고가 53개(린트 22종) 나왔습니다. 첫 경고는 그룹을 통째로 켰다는 것 자체에 대한 `clippy::blanket_clippy_restriction_lints`(suspicious, 기본 warn)입니다.

```text
warning: `clippy::restriction` is not meant to be enabled as a group
  = help: enable the restriction lints you need individually
```

나머지에는 `?` 연산자를 쓰지 말라는 `question_mark_used`, 모든 공개 함수에 `#[inline]`을 붙이라는 `missing_inline_in_public_items` 같은 것이 섞여 있습니다. 가장 많은 `implicit_return`(10개)은 함수 끝의 식에 `return`을 붙이라고 하는데, 그대로 따르면 기본으로 켜진 style 린트 `needless_return`이 `` unneeded `return` statement ``로 경고합니다(임시 파일로 확인). `pub(crate)`에 `pub(in crate)`로 쓰라는 `pub_with_shorthand`도 나왔는데, 같은 그룹에는 반대를 요구하는 `pub_without_shorthand`가 있습니다. restriction은 "우리 프로젝트는 이것을 쓰지 않는다"는 정책을 하나씩 고르는 목록입니다. ch18의 `panic_free_lib`이 `unwrap_used`, `indexing_slicing`, `arithmetic_side_effects` 등 패닉에 관한 여덟 개만 고른 것이 그 예입니다.

**nursery와 cargo.** nursery는 다듬는 중인 린트라 골라 켭니다. cargo 그룹은 `Cargo.toml`과 의존성 그래프를 봅니다. 책이 "설정하면 경고하게 할 수 있다"고 말한 `multiple_crate_versions`가 여기 있습니다. [ch25](../ch25-dependency-graph/README.md)의 `projects/real_graph`를 저장소 밖에 복사해 `cargo clippy -- -W clippy::multiple_crate_versions`를 실행하자 `` multiple versions for dependency `syn`: 2.0.119, 3.0.10 ``이 나왔습니다(2026-11-19에 만든 `Cargo.lock` 기준). 중복을 줄이는 방법은 ch25 2.3절에서 다뤘습니다.

**책이 든 린트의 현재 상태.** 책은 다른 Item의 지침에 대응하는 린트를 나열합니다. 1.100.0의 `clippy-driver -W help`로 확인한 그룹과 기본 수준입니다.

| Item | 기본으로 켜진 것 | 기본으로 꺼진 것 |
| --- | --- | --- |
| 머리말 | `approx_constant`(correctness, deny) | — |
| 1 | — | `fn_params_excessive_bools`, `struct_excessive_bools`(pedantic) |
| 3 | `ok_expect`, `unwrap_or_default`(style, 책의 `unwrap_or_else_default`) | `unwrap_in_result`(restriction) |
| 5 | `from_over_into`(style) | `cast_lossless`, `cast_possible_truncation`, `cast_possible_wrap`, `cast_precision_loss`, `cast_sign_loss`(pedantic), `as_conversions`(restriction) |
| 8 | `box_collection`(perf), `vec_box`, `borrowed_box`(complexity) | — |
| 9 | `iter_next_loop`(correctness, deny), `suspicious_map`(suspicious), `into_iter_on_ref`, `map_clone`, `needless_range_loop`, `unnecessary_fold`(style), `explicit_counter_loop`, `filter_map_identity`, `iter_count`, `manual_filter_map`, `manual_find_map`, `skip_while_next`, `unnecessary_filter_map`(complexity) | `explicit_iter_loop`, `explicit_into_iter_loop`, `iter_not_returning_iterator`(pedantic), `search_is_some`(nursery). `from_iter_instead_of_collect`는 제거됨 |
| 10 | `derive_ord_xor_partial_ord`, `derived_hash_with_manual_eq`(correctness, deny), `partialeq_ne_impl`(complexity) | `expl_impl_clone_on_copy`(pedantic) |
| 18 | — | `panic`, `expect_used`(restriction) |
| 21, 24·25, 26 | — | `wildcard_dependencies`, `multiple_crate_versions`, `negative_feature_names`, `redundant_feature_names`(cargo) |
| 23 | — | `wildcard_imports`(pedantic) |
| 27 | `missing_safety_doc`(style) | `missing_panics_doc`(pedantic), `undocumented_unsafe_blocks`(restriction) |

책의 이름으로 속성을 쓰면 1.100.0은 `` lint `clippy::unwrap_or_else_default` has been renamed to `clippy::unwrap_or_default` ``와 `` lint `clippy::from_iter_instead_of_collect` has been removed: lint has proved problematic ``으로 알려 줍니다(rustc `renamed_and_removed_lints`).

**따르지 않아도 되는 경우.** 린트를 켜는 데는 비용이 듭니다. 기본 설정에서 경고 0개였던 `before/lib.rs`에 `projects/lint_config`의 설정을 걸자 18개가 나왔고 그중 13개가 pedantic 린트였습니다(2.6절의 표). 그 하나하나를 판단해야 했습니다. 작은 스크립트나 실험용 코드라면 기본 그룹만으로 충분합니다.

### 2.6 지침 5: 린트 설정은 프로젝트 단위로, 수준은 목적에 맞게 정한다

**문제 상황.** 이 챕터의 예제 파일들은 `#![warn(clippy::float_cmp)]`처럼 파일마다 속성으로 린트를 켰습니다. 파일이 하나일 때는 괜찮지만, 크레이트가 여럿인 프로젝트에서 크레이트 루트마다 같은 속성 목록을 복사하면 금방 어긋납니다. 책은 속성만 소개하지만, 지금은 Cargo가 린트 설정을 매니페스트에 모으는 방법을 제공합니다.

**비권장 코드.** [`projects/lint_config`](projects/lint_config)의 `before/lib.rs`는 설정 없이 쓴 독서 기록 라이브러리입니다. 같은 파일을 두 패키지가 함께 씁니다. `[lints]`를 비워 둔 `before-default`는 기본 설정의 Clippy로 경고가 **0개**입니다. 워크스페이스 설정을 물려받는 `before-strict`는 **18개**(린트 13종)입니다.

| 린트 | 그룹 | 개수 | `after`에서의 처리 |
| --- | --- | --- | --- |
| `cast_lossless` | pedantic | 3 | `f64::from`으로 고침 |
| `cast_possible_truncation`, `cast_sign_loss` | pedantic | 2, 1 | 진행률은 정수 계산으로 리팩터링, 책 수는 `u8::try_from`으로 고침 |
| `cast_precision_loss` | pedantic | 1 | `u64`→`f64`에는 `From`이 없으므로 `#[expect(…, reason)]` |
| `missing_errors_doc`, `missing_panics_doc` | pedantic | 2, 1 | 문서에 `# Errors` 절을 쓰고, 패닉하던 `unwrap`을 없앰 |
| `needless_pass_by_value`, `format_push_string` | pedantic | 1, 1 | `&str`로 받기, `write!` 사용 |
| `wildcard_imports` | pedantic | 1 | `use std::collections::BTreeMap;` (`use units::*`는 `clippy.toml`이 허용) |
| `unwrap_used` | restriction | 1 | `?`로 `Option`을 돌려줌 |
| `allow_attributes`, `allow_attributes_without_reason` | restriction | 1, 1 | 아무도 부르지 않던 함수를 지움 |
| `unreachable_pub` | rustc | 2 | `pub(crate)`로 좁힘 |

**권장 코드 ①: `[workspace.lints]`와 `[lints]`.** 워크스페이스 루트의 `Cargo.toml`에 설정을 한 번 적고, 멤버는 `[lints] workspace = true`로 물려받습니다. 표 이름의 `rust`와 `clippy`는 린트 이름의 `::` 앞부분입니다(`clippy::unwrap_used` → `[lints.clippy]`의 `unwrap_used`).

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
unreachable_pub = "warn"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
must_use_candidate = "allow"
unwrap_used = "warn"
allow_attributes = "warn"
allow_attributes_without_reason = "warn"
print_stdout = "warn"
```

- **물려받기는 명시해야 합니다.** 멤버에 `[lints]` 표가 아예 없으면 cargo 1.100.0은 `` warning: missing `[lints]` to inherit `[workspace.lints]` ``(`cargo::missing_lints_inheritance`)를 내고, 물려받으려면 `workspace = true`를, 물려받지 않으려면 빈 `[lints]`를 쓰라고 안내합니다. `before-default`가 빈 `[lints]`를 둔 이유입니다.
- **`priority`로 순서를 정합니다.** Cargo는 표 안의 순서를 무시하므로, 그룹을 켜고 그 안의 린트 하나를 끄려면 그룹의 `priority`를 낮춰야 합니다(기본값 0). 빠뜨리면 Clippy의 `lint_groups_priority`(correctness, deny)가 `` error: lint group `pedantic` has the same priority (0) as a lint ``로 막고 `pedantic = { level = "warn", priority = -1 }`을 제안합니다.
- **`[lints.rust]`는 `cargo build`에도, `[lints.clippy]`는 `cargo clippy`에만 적용됩니다.** `cargo build -p before-strict`는 `unreachable_pub` 두 개만 경고했습니다.
- **`[lints]`는 패키지 단위입니다.** 의존성에는 적용되지 않고(Cargo Book), 같은 패키지의 예제와 테스트에는 모두 적용됩니다. 멤버 하나에서 일부만 바꿀 수도 없습니다. `workspace = true` 옆에 다른 린트를 적으면 매니페스트 에러입니다(4.4절). 그래서 `after/examples/report.rs`는 크레이트 루트의 `#![expect(clippy::print_stdout, reason = "…")]`로 출력 금지에서 빠집니다.

**권장 코드 ②: `clippy.toml`.** 린트의 수준이 아니라 **동작**을 바꾸는 설정 파일입니다. Clippy는 패키지 디렉터리에서 위로 올라가며 이 파일을 찾습니다.

```toml
allowed-wildcard-imports = ["units"]  # wildcard_imports: 경로에 units가 든 글롭은 허용
allow-unwrap-in-tests = true          # unwrap_used: 테스트 코드는 허용
```

ch23이 미뤘던 `allowed-wildcard-imports`는 경로의 한 마디로 허용할 글롭을 지정합니다(경로에 `prelude`가 든 글롭은 원래 허용). 효과는 설정 파일이 없는 빈 디렉터리를 `CLIPPY_CONF_DIR`로 가리켜 보면 드러납니다. `CLIPPY_CONF_DIR=$(mktemp -d) cargo clippy -p after --all-targets`는 `use units::*;`의 `wildcard_imports` 1개와 테스트 안의 `unwrap_used` 4개를 경고했습니다. Clippy 문서는 이 파일 형식이 불안정하며 나중에 바뀔 수 있다고 적고 있습니다. 키 목록은 Clippy 문서의 Lint Configuration 장에 있습니다.

**린트 수준을 고르는 기준.** ch16이 미룬 질문입니다. 같은 린트도 어디에 어떤 수준으로 두느냐에 따라 효과가 다릅니다.

| 수준 | 이럴 때 | 이 챕터의 예 |
| --- | --- | --- |
| `warn` | 대부분의 린트. 개발 중에는 경고로 보고, CI에서 경고를 실패로 만듦(2.4절) | `pedantic`, `unwrap_used`, `unreachable_pub` |
| `deny` | 개발 중에도 그 자리에서 멈춰야 하지만, 이유가 있으면 안쪽에서 예외를 둘 수 있어야 할 때 | ch18 `panic_free_lib`의 패닉 린트 |
| `forbid` | 크레이트 안에 예외가 하나도 있으면 안 될 때. `#[expect]`조차 거부됨(4.1절) | `unsafe_code` |
| `allow` | 그룹째 켠 뒤 맞지 않는 하나를 끌 때, 조건에 따라 나는 경고(2.3절) | `must_use_candidate` |
| `expect` | 이유가 있는 개별 예외. 속성으로만 씀 | `cast_precision_loss` 한 곳 |

`deny`로 둔 린트는 실험 중인 코드까지 빌드를 막으므로, 막을 이유가 분명한 것에만 씁니다. Clippy 문서도 `deny`는 "CI/CD 스크립트에서 가장 유용하다"고 적고 있습니다. `forbid`는 예외의 여지를 없애므로 `unsafe_code`처럼 "이 크레이트에는 절대 없다"고 말할 수 있는 것에만 겁니다.

**따르지 않아도 되는 경우.** 파일 하나짜리 예제나 크레이트 하나뿐인 작은 프로젝트라면 크레이트 루트의 `#![warn(…)]`로 충분합니다(이 챕터의 예제가 그렇게 했고, ch18은 `[lints]`를 썼습니다). `[lints]`는 Cargo 1.74부터 지원하므로 그보다 오래된 툴체인을 지원해야 하는 크레이트는 속성을 씁니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc·cargo 1.100.0, Clippy 0.1.100에서 확인한 차이는 다음과 같습니다.

- **린트 그룹.** 책은 경고를 Correctness, Idiom, Concision, Performance, Readability 다섯 범주로 설명합니다. 지금의 Clippy는 그룹 아홉 개로 나누고, 그중 다섯 개(correctness, suspicious, style, complexity, perf)가 기본으로 켜져 있습니다(2.1절). 기본 그룹은 `clippy::all`이고, 1.100.0의 `-W help`는 같은 그룹을 `clippy::default`로도 보여 줍니다.
- **린트 이름.** 책의 `unwrap_or_else_default`는 `unwrap_or_default`로 바뀌었고, `from_iter_instead_of_collect`는 제거되었습니다(2.5절). Clippy 1.100.0에는 린트가 832개 있고, 1.98.1보다 10개 늘었습니다.
- **책 이후에 생긴 도구.** 로컬 문서의 릴리스 노트로 확인했습니다. `#[expect]`와 `reason`(rustc 1.81.0), `Cargo.toml`의 `[lints]`와 `[workspace.lints]`(Cargo 1.74.0), `build.warnings`(Cargo 1.97.0), `cargo clippy --fix`의 타깃 선택 변경(Cargo 1.89.0). Cargo 1.100.0에는 물려받기를 빠뜨린 멤버를 알리는 `cargo::missing_lints_inheritance`가 있습니다(Cargo Book의 Lints 장).
- **문서와 다른 동작.** Clippy 문서(1.100.0)의 `lint_groups_priority` 설명은 "`lints.workspace = true`로 물려받은 린트는 검사하지 않는다"고 적지만, 1.100.0에서는 `[workspace.lints]`의 같은 실수도 같은 에러로 막았습니다(임시 워크스페이스로 확인). Usage 장의 "`--fix`는 `--all-targets`를 뜻한다"는 설명도 1.89.0 이후의 실제 동작과 다릅니다(2.2절).
- **책에 없는 지침.** 다음은 책의 Item 29가 명시적으로 권하지 않는 것입니다(2026-11-20에 온라인판에서 확인). 책은 Clippy의 다섯 범주, 설명 페이지 읽기와 알려진 오탐, `#[allow]`·`#![allow]`와 리팩터링 선호, 경고 0개 유지(굵게), CI에서 Clippy 실행, 린트 목록 읽기(굵게), 다른 Item에 대응하는 린트 목록을 다룹니다. `pedantic` 같은 그룹 이름, `clippy.toml`, `[lints]`, `#[expect]`는 다루지 않습니다.
  - 지침 1의 그룹별 대응과 `cargo clippy --fix`: Clippy 문서의 Clippy's Lints 장과 Usage 장, 프로젝트 사본에서 실행한 결과가 근거입니다.
  - 지침 2 전체(가장 좁은 범위, `reason`, `#[expect]`, 조건에 따라 나는 경고의 대안, `allow_attributes`): rustc Book의 Lint Levels 절, `clippy::allow_attributes`의 설명, 예제 02·03의 실행 결과가 근거입니다.
  - 지침 3의 `build.warnings`와 Clippy 버전 고정: Clippy 문서의 Usage 장, 1.98.1과 1.100.0으로 실행한 결과가 근거입니다.
  - 지침 4의 "pedantic은 그룹째 켜고 끄기, restriction은 골라 켜기": Clippy 문서의 Usage 장과 첫 페이지, `blanket_clippy_restriction_lints`의 설명, `after`에 restriction을 켜 본 결과가 근거입니다.
  - 지침 5 전체(`[lints]`, `[workspace.lints]`, `priority`, `clippy.toml`, 린트 수준을 고르는 기준): Cargo Book의 The `[lints]` section 절과 workspaces 장, Clippy 문서의 Configuration 장, `projects/lint_config`의 실행 결과가 근거입니다.

### 2.8 기억할 것

- 경고마다 설명(`cargo clippy --explain`)을 읽고 고칠지, 작은 리팩터링으로 없앨지, 이유를 적고 예외로 둘지 고릅니다. 오탐을 따지기보다 다르게 쓰는 편이 대개 빠릅니다.
- 예외는 가장 좁은 범위에 `#[expect(…, reason = "…")]`로 둡니다. 빌드 설정에 따라 나는 경고만은 `#[allow]`(이유와 함께)나 `cfg_attr`, 더 좋게는 경고가 생기지 않는 코드로 처리합니다.
- 경고 0개를 유지해야 새 경고가 보입니다. CI에서 경고를 실패로 만들고 Clippy 버전을 고정합니다. 그래도 Clippy가 못 잡는 버그는 테스트가 잡습니다.
- 꺼진 린트 목록도 읽어 봅니다. pedantic은 그룹째 켜고 맞지 않는 것을 끄거나 골라 켜고, restriction은 정책으로 고른 것만 켭니다.
- 설정은 `[workspace.lints]`에 모으고 멤버는 `workspace = true`로 물려받습니다. 그룹에는 `priority = -1`, 동작 조정은 `clippy.toml`, `forbid`는 예외가 없어야 할 때만 씁니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 끝나고, `cargo clippy -- -D warnings`를 통과합니다(03은 `--release`로도 확인). 비권장 판에 붙인 `#[expect]`를 지우면 경고가 나는 것을 확인했습니다. 경고 출력에서 위치 줄은 줄였습니다. `projects/lint_config`는 그 디렉터리에서 실행합니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_judge_each_warning.rs`](examples/01_judge_each_warning.rs) | 1 | `0644`, 복잡한 타입, help대로 `Box`에 넣은 열거형 | `0o644`, 타입 별칭, `#[expect]`로 둔 열거형 | 2.2 |
| [`02_expect_with_reason.rs`](examples/02_expect_with_reason.rs) | 2 | 모듈 전체의 이유 없는 `#![allow]` | 문장 하나의 `#[expect(…, reason)]`, `try_from` | 2.3 |
| [`03_conditional_warnings.rs`](examples/03_conditional_warnings.rs) | 2 | (README) 조건부 경고에 `#[expect]` | `#[allow]`+이유, `cfg_attr`, `cfg!` | 2.3 |
| [`04_new_lint_on_upgrade.rs`](examples/04_new_lint_on_upgrade.rs) | 3 | `u32::BITS - x.leading_zeros()`, `32 - …` | `x.bit_width()` | 2.4 |
| [`05_default_off_lints.rs`](examples/05_default_off_lints.rs) | 4 | `f64`의 `==`, `for_each` 안의 `return` | 정수 금액, `for`와 `break` | 2.5 |
| [`projects/lint_config`](projects/lint_config) | 5 | `before-strict`: 경고 18개 | `after`: 경고 0개 | 2.6 |

### 01_judge_each_warning — 고치기, 리팩터링, 예외

```sh
cargo run -p er-ch29-listen-to-clippy --example 01_judge_each_warning
```

```text
1. 진짜 실수: 고친다 (clippy::zero_prefixed_literal)
  비권장 v1: MODE = 0o1204, 그룹 읽기 false
  권장 v2  : MODE = 0o644, 그룹 읽기 true
2. 취향처럼 보이는 경고: 작은 리팩터링으로 없앤다 (clippy::type_complexity)
  일반 40000원 → 비권장 v1 37000원, 권장 v2 37000원
  회원 60000원 → 비권장 v1 49000원, 권장 v2 49000원
  손님 20000원 → 비권장 v1 20000원, 권장 v2 20000원
3. 의도한 코드: 이유를 적고 예외로 둔다 (clippy::large_enum_variant)
  프레임 1000개: 본문 v1 506880바이트 = v2 506880바이트, Ack v1 10개 = v2 10개
  비권장 v1 (Box): 프레임 하나 16바이트, 모두 522880바이트, 상자 990개
  권장 v2 (그대로): 프레임 하나 514바이트, 모두 514000바이트, 상자 0개
```

3번 구역의 크기는 64비트 플랫폼(macOS aarch64) 기준입니다. `#[expect]` 넷을 지우면 `zero_prefixed_literal` 1개, `type_complexity` 3개(함수 안의 `let` 타입 표기 포함), `large_enum_variant` 1개가 기본 설정에서 경고합니다.

### 02_expect_with_reason — 예외의 범위와 이유

```sh
cargo run -p er-ch29-listen-to-clippy --example 02_expect_with_reason
```

```text
1. 비권장 v1: 모듈 전체에 이유 없는 #[allow]
  percent(30, 40) = 75
  항목 300개 → 헤더 44, 읽는 쪽이 되찾은 항목 44개 (경고 없음)
2. 권장 v2: 문장 하나에만 #[expect(…, reason)], 나머지는 고침
  percent(30, 40) = 75
  항목 300개는 1바이트 헤더에 담을 수 없음 → 기록하지 않고 알림
  항목 3개 → 헤더 3, 되찾은 항목 3개
```

### 03_conditional_warnings — 빌드 설정에 따라 나는 경고

```sh
cargo run -p er-ch29-listen-to-clippy --example 03_conditional_warnings
cargo run -p er-ch29-listen-to-clippy --example 03_conditional_warnings --release
```

```text
빌드: 디버그 (debug_assertions = true)
  대안 1 allow     : ["형식 검사", "불변식 검사"]
  대안 2 cfg_attr  : ["형식 검사", "불변식 검사"]
  대안 3 cfg! 매크로: ["형식 검사", "불변식 검사"]
```

`--release`로 실행하면 첫 줄이 `빌드: 릴리스 (debug_assertions = false)`가 되고 세 목록이 모두 `["형식 검사"]`입니다. 같은 함수에 `#[expect(unused_mut, reason = "…")]`를 붙인 판을 따로 만들어 검사하면 디버그 빌드에서만 다음 경고가 납니다(릴리스는 조용함).

```text
warning: this lint expectation is unfulfilled
 --> examples/…:3:9
  |
3 |         unused_mut,
  |         ^^^^^^^^^^
  |
  = note: 릴리스 빌드에서는 아래 push가 지워져 mut이 쓰이지 않음
  = note: `#[warn(unfulfilled_lint_expectations)]` on by default
```

### 04_new_lint_on_upgrade — Clippy를 올리면 생기는 경고

```sh
cargo run -p er-ch29-listen-to-clippy --example 04_new_lint_on_upgrade
```

```text
                     x    v1 BITS판      v1 32판          v2
            5000000000           1           1          33
         1099511627776           9           9          41
  18446744073709551615          32          32          64
v2는 작은 값도 됩니다: 0 → 0, 5 → 3
```

두 `v1`은 `x < 2^32`이면 뺄셈이 넘쳐 디버그 빌드에서 패닉하므로 2^32 이상의 값만 넣었습니다. 1.98.1 Clippy와의 비교는 `#[expect]`를 지운 사본에서 `cargo +1.98.1 clippy`로 했습니다. **이 저장소의 rustup(1.29.3)은 설치되지 않은 툴체인을 이름으로 가리키기만 해도 내려받으므로**, 따라 해 볼 때는 `rustup toolchain list`에 그 버전이 있는지 먼저 확인합니다.

### 05_default_off_lints — 기본으로 꺼진 린트가 잡는 것

```sh
cargo run -p er-ch29-listen-to-clippy --example 05_default_off_lints
```

```text
1. clippy::float_cmp (pedantic)
  비권장 v1: 0.1 + 0.2 == 0.3 → false
  권장 v2  : 1 + 2 == 3 (10분의 1 단위 정수) → true
2. clippy::needless_for_each (pedantic)
  [3, 4, -1, 10]에서 처음 음수 전까지의 합: 비권장 v1 17, 권장 v2 7
```

`#[expect]` 둘을 지우면 `` strict comparison of `f32` or `f64` ``와 `` needless use of `for_each` ``가 나고, 파일 첫머리의 `#![warn(…)]`까지 지우면 경고가 사라집니다.

### projects/lint_config — 프로젝트 단위 설정

워크스페이스 하나에 패키지 셋이 있습니다. `before/lib.rs`(설정 전에 쓴 코드)를 `before-default`(빈 `[lints]`)와 `before-strict`(`workspace = true`)가 함께 쓰고, `after`는 고친 판입니다. `default-members`가 경고 0개인 두 패키지라서 `-p` 없이 실행하는 명령은 통과합니다.

```sh
cd appd-a/ch29-listen-to-clippy/projects/lint_config
cargo clippy --all-targets -- -D warnings  # 통과 (before-default, after)
cargo test                                 # after: test result: ok. 3 passed
cargo clippy -p before-strict              # 경고 18개, 종료 코드 0
cargo clippy -p before-strict -- -D warnings          # 실패 (종료 코드 101)
CARGO_BUILD_WARNINGS=deny cargo clippy -p before-strict # 실패 (종료 코드 101)
cargo run -p after --example report
```

```text
이번 달 독서
- 러스트 입문 (100/400)
- 소유권 이야기 (240/240)
러스트 입문: 25%
소유권 이야기: 100%
없는 책: 기록 없음
남은 시간: 7.5시간
내보낼 책 수: 2
```

`before-strict`의 경고는 2.6절의 표와 같습니다. 그 가운데 `unreachable_pub`의 출력은 ch22에서 본 것과 같은 모양이고, 끝에 이 설정이 어디서 왔는지가 붙습니다.

```text
warning: unreachable `pub` item
  --> before-strict/../before/lib.rs:13:5
   |
13 |     pub const PAGES_PER_HOUR: u32 = 40;
   |     ---^^^^^^^^^^^^^^^^^^^^^^^^^^
   |     |
   |     help: consider restricting its visibility: `pub(crate)`
   |
   = help: or consider exporting it for use by other crates
   = note: requested on the command line with `-W unreachable-pub`
```

`requested on the command line`은 Cargo가 `[lints]`를 rustc와 Clippy의 명령줄 인자로 바꿔 넘기기 때문에 나오는 표현입니다. 검증이 끝나면 `cargo clean`으로 `target/`을 지웁니다.

## 4. 자주 하는 실수

`compile_fail/`의 세 파일은 Clippy 없이 `rustc`만으로 실패합니다. 챕터 디렉터리에서 아래처럼 확인합니다. Clippy나 Cargo에서만 나는 실수는 4.4절의 표에 모았습니다.

```sh
rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0453_expect_under_forbid.rs
rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0452_reason_not_last.rs
rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/unfulfilled_expect_deny_warnings.rs
```

### 4.1 `forbid` 아래에서 `#[expect]`로 예외를 둠 — E0453

[`compile_fail/e0453_expect_under_forbid.rs`](compile_fail/e0453_expect_under_forbid.rs)는 크레이트에 `#![forbid(missing_docs)]`를 걸고, 테스트 도우미 하나에 이유를 적은 `#[expect]`를 붙였습니다.

```text
error[E0453]: expect(missing_docs) incompatible with previous forbid
  --> compile_fail/e0453_expect_under_forbid.rs:17:14
   |
 8 | #![forbid(missing_docs)]
   |           ------------ `forbid` level set here
...
17 |     #[expect(missing_docs, reason = "테스트 전용 도우미라 문서를 쓰지 않음")] // E0453
   |              ^^^^^^^^^^^^ overruled by previous forbid
```

**원인.** `forbid`는 크레이트 안의 어디서도 수준을 낮출 수 없습니다. ch16 4.2절은 `#[allow]`가 거부되는 것을 보였는데, 이유를 적은 `#[expect]`도 똑같이 거부됩니다. 같은 파일을 `clippy::` 린트로 바꾸면 `rustc`는 통과시키고 `clippy-driver`만 E0453을 냈습니다. rustc는 Clippy 린트의 수준을 따지지 않기 때문입니다.

**해결.** 예외가 있을 수 있는 정책이라면 `#![deny(missing_docs)]`로 바꿉니다(이렇게 고치면 컴파일됨). 예외가 정말 없어야 한다면 `forbid`를 두고 그 도우미에 문서를 씁니다.

### 4.2 `reason`을 린트 이름 앞에 씀 — E0452

[`compile_fail/e0452_reason_not_last.rs`](compile_fail/e0452_reason_not_last.rs)

```text
error[E0452]: malformed lint attribute input
 --> compile_fail/e0452_reason_not_last.rs:9:10
  |
9 | #[expect(reason = "다음 단계에서 쓸 함수", dead_code)] // E0452
  |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ reason in lint attribute must come last
```

**원인.** `reason`은 린트 이름들 뒤에, 한 번만, 문자열 리터럴로 적어야 합니다. 같은 E0452가 `reason = 3`(`reason must be a string literal`)과 `reason`을 두 번 쓴 경우(`reason in lint attribute must come last`)에도 나왔습니다.

**해결.** `#[expect(dead_code, reason = "다음 단계에서 쓸 함수")]`로 순서를 바꿉니다.

### 4.3 고친 뒤 남은 `#[expect]`가 `deny(warnings)`에서 에러가 됨 — 에러 코드 없음

[`compile_fail/unfulfilled_expect_deny_warnings.rs`](compile_fail/unfulfilled_expect_deny_warnings.rs)

```text
error: this lint expectation is unfulfilled
  --> compile_fail/unfulfilled_expect_deny_warnings.rs:13:14
   |
13 |     #[expect(unused_variables, reason = "합계 출력은 다음 단계에서 붙임")]
   |              ^^^^^^^^^^^^^^^^
   |
   = note: 합계 출력은 다음 단계에서 붙임
note: the lint level is defined here
  --> compile_fail/unfulfilled_expect_deny_warnings.rs:10:9
   |
10 | #![deny(warnings)]
   |         ^^^^^^^^
   = note: `#[deny(unfulfilled_lint_expectations)]` implied by `#[deny(warnings)]`
```

**원인.** 쓰지 않던 변수에 `#[expect(unused_variables)]`를 붙였다가, 나중에 그 변수를 출력하도록 고치고 속성은 남겼습니다. `#[expect]`가 제 할 일을 한 것이고, 경고를 에러로 올린 빌드(CI의 `-D warnings`와 같은 효과)라서 에러가 되었습니다. 린트가 낸 에러라 에러 코드가 없습니다.

**해결.** 낡은 `#[expect]`를 지웁니다. 지우면 컴파일됩니다. `#[allow]`였다면 아무도 몰랐을 것이므로, 이 에러는 `#[expect]`를 쓰는 이유이기도 합니다.

### 4.4 그 밖의 실수

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| `[lints]`에서 그룹과 그 안의 린트를 같은 priority로 둠 | `` error: lint group `pedantic` has the same priority (0) as a lint ``(`lint_groups_priority`, deny) | 그룹에 `priority = -1` | 2.6 |
| `workspace = true` 옆에 멤버만의 린트를 더함 | `` cannot override `workspace.lints` in `lints`, either remove the overrides or `lints.workspace = true` and manually specify the lints `` | 크레이트 루트의 `#![expect(…)]`·`#![allow(…)]`로 예외를 두거나, 그 멤버만 전체를 직접 적음 | 2.6 |
| 멤버에 `[lints]` 표를 두지 않음 | `` warning: missing `[lints]` to inherit `[workspace.lints]` ``. 설정이 적용되지 않음 | `workspace = true`, 물려받지 않을 거면 빈 `[lints]` | 2.6 |
| restriction을 그룹째 켬 | `` `clippy::restriction` is not meant to be enabled as a group ``와 서로 모순되는 경고들 | 필요한 린트만 골라 켬 | 2.5 |
| 빌드 설정에 따라 나는 경고에 `#[expect]` | 한쪽 빌드에서 `this lint expectation is unfulfilled` | `#[allow]`+이유, `cfg_attr`, 경고가 없는 코드 | 2.3, 03 |
| `#[expect]`가 통과하니 기본 Clippy도 잡는다고 믿음 | `#[expect]`가 꺼진 린트를 그 범위에서 켰을 뿐 | 속성을 지우고 `cargo clippy`로 확인, `clippy-driver -W help`로 기본 수준 확인 | 2.3, 05 |
| 새 버전의 린트를 `#[expect]`에 적고 옛 Clippy로 검사 | `` unknown lint: `clippy::mismatched_bit_width_type` `` | CI와 개발 환경의 Clippy 버전을 맞춤 | 2.4 |
| 책의 옛 린트 이름을 씀 | `has been renamed to …`, `has been removed: …` | 새 이름을 쓰거나 지움 | 2.5 |

## 5. 연습 문제

### 연습 1. restriction 린트 하나 더 고르기

[`projects/lint_config`](projects/lint_config)의 `[workspace.lints.clippy]`에 `as_conversions = "warn"`을 더하고 `cargo clippy --all-targets`를 실행하세요. 무엇이 걸리고, 어떻게 처리하는 것이 맞을까요? `as`를 모두 없앨 수 없는 이유도 설명하세요.

<details>
<summary>힌트</summary>

`after`의 `units::hours`에 있는 `pages as f64` 한 곳이 `` using a potentially dangerous silent `as` conversion ``으로 걸립니다(라이브러리와 테스트 빌드에서 같은 경고가 한 번씩 보고됨). `u64`에서 `f64`로 가는 `From`도 `TryFrom`도 표준 라이브러리에 없으므로 `as`를 없앨 수는 없습니다. 이미 붙어 있는 `#[expect]`의 린트 목록에 `clippy::as_conversions`를 더하면 같은 이유 하나로 두 린트를 함께 예외로 둘 수 있고, `-D warnings`로도 통과합니다. `cargo clippy -p before-strict`로 보면 `as` 여섯 개에 모두 경고가 붙어 18개가 24개로 늘어납니다.

</details>

### 연습 2. 조건이 어긋난 `cfg_attr`

[`03_conditional_warnings.rs`](examples/03_conditional_warnings.rs)의 대안 2(`checks_cfg_attr`)에서 `push`에 붙은 `#[cfg(debug_assertions)]`만 `#[cfg(test)]`로 바꾸고, `cfg_attr`의 조건은 그대로 두세요. `cargo clippy`와 `cargo clippy --release`의 결과를 예측한 뒤 확인하세요.

<details>
<summary>힌트</summary>

디버그 빌드에서는 테스트 빌드가 아니므로 `push`가 지워지고 `unused_mut`이 나지만, `cfg_attr`의 조건(`not(debug_assertions)`)이 거짓이라 `#[expect]`가 붙지 않습니다. 그래서 `variable does not need to be mutable` 경고가 납니다. 릴리스 빌드에서는 `#[expect]`가 붙고 `unused_mut`도 나므로 조용합니다. 두 조건을 따로 관리해야 한다는 것이 대안 2의 약점이고, 대안 3은 이런 일이 생기지 않습니다.

</details>

### 연습 3. `--fix`가 고치는 것과 남기는 것

`projects/lint_config`를 저장소 밖에 복사하고, 사본에서 `cargo clippy --fix --allow-no-vcs -p before-strict`를 실행하세요. `before/lib.rs`가 어떻게 바뀌었고 경고가 몇 개 남는지 확인한 뒤, 남은 경고를 `after/src/lib.rs`의 처리와 비교하세요. `--fix`가 고치지 않은 경고들의 공통점은 무엇인가요?

<details>
<summary>힌트</summary>

`use std::collections::BTreeMap;`, `pub(crate)` 두 곳, `f64::from` 세 곳, `#[allow(dead_code)]` → `#[expect(dead_code)]`가 바뀌고 경고 11개(린트 9종)가 남습니다. 남은 것은 캐스트의 잘림과 부호, 정밀도 손실, `unwrap`, 문서의 `# Errors`·`# Panics` 절, 값으로 받는 인자처럼 **무엇이 맞는지 코드의 뜻을 알아야 정할 수 있는 것**입니다. 예를 들어 `as u8`을 `u8::try_from`으로 바꾸면 반환 타입과 실패 처리까지 정해야 합니다. 새로 생긴 `#[expect(dead_code)]`에는 이유가 없어서 `allow_attributes_without_reason`이 계속 경고합니다. `after`는 그 함수를 지웠습니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 29: Listen to Clippy](https://effective-rust.com/clippy.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- Clippy 문서: [소개와 그룹 표](https://doc.rust-lang.org/clippy/), [Usage](https://doc.rust-lang.org/clippy/usage.html)(`--fix`, `build.warnings`, pedantic·restriction), [Clippy's Lints](https://doc.rust-lang.org/clippy/lints.html)(그룹별 권고), [Configuration](https://doc.rust-lang.org/clippy/configuration.html), [Lint Configuration](https://doc.rust-lang.org/clippy/lint_configuration.html)(`allowed-wildcard-imports`, `allow-unwrap-in-tests`). 로컬에서는 `rustc --print sysroot` 아래 `share/doc/rust/html/clippy/`에 같은 문서가 있습니다.
- [Clippy 린트 목록(1.100.0)](https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html). 린트 하나의 설명은 `cargo clippy --explain <린트 이름>`으로도 봅니다.
- Cargo Book: [The `[lints]` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section)(`priority`), [workspaces의 lints 표](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table), [Lints](https://doc.rust-lang.org/cargo/reference/lints.html)(`missing_lints_inheritance`)
- rustc Book: [Lint Levels](https://doc.rust-lang.org/rustc/lints/levels.html)(`expect`, `reason`)
- 로컬 문서의 릴리스 노트(`share/doc/rust/html/releases.md`): 1.74.0(Cargo `[lints]`), 1.81.0(`#[expect]`), 1.89.0(`cargo clippy --fix`의 타깃 선택), 1.97.0(`build.warnings`, `bit_width`)
- 에러 설명: `rustc --explain E0452`, `rustc --explain E0453`
