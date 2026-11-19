# ch26. 기능이 불어나는 것을 경계하기

Cargo의 기능(feature)은 크레이트의 일부를 켜고 끌 수 있게 해 줍니다. 쓰지 않는 의존성을 빼서 빌드를 가볍게 하는 좋은 도구이지만, 크레이트 작성자의 눈에는 "스위치"로 보이는 기능이 빌드 그래프 전체에서는 전혀 다르게 동작합니다. 기능은 내가 켜지 않아도 그래프의 다른 크레이트가 켤 수 있고, 한 번 켜지면 그 크레이트를 쓰는 모든 곳에서 켜집니다. 이 챕터는 그 성질에서 나오는 판단을 다룹니다. 기능이 바탕으로 삼는 `cfg` 조건부 컴파일을 먼저 정리하고, 기능은 왜 더하기만 해야 하는지, 왜 공개 필드와 트레이트 메서드를 기능으로 가리면 안 되는지, 기능이 늘수록 검사할 빌드가 어떻게 불어나는지를 실제로 깨지는 프로젝트로 확인합니다.

- 원제: [Item 26: Be wary of `feature` creep](https://effective-rust.com/features.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch26-feature-creep`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

책의 4장(Dependencies)을 마무리하는 Item입니다. 다음 본편 내용을 안다고 보고 진행합니다.

- [Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.3절: 기능, `#[cfg(feature = "…")]`, 기능 통합(feature unification), 기능 리졸버 1과 2가 통합하는 범위, `cargo tree -f '{p} {f}'`
- [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.7절: 워크스페이스에서 함께 빌드하는 멤버끼리 기능이 합쳐지는 것
- [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md): 모듈, `pub`, `pub use`
- [ch13](../ch13-default-implementations/README.md): 트레이트의 기본 구현

`[features]` 테이블과 선택적 의존성(optional dependency)의 문법 전반은 Part 3 `ch5-features`에서, `--cfg`와 `--check-cfg`의 상세는 Part 5 `ch5-conditional-compilation`에서 다룹니다. 이 챕터는 판단에 필요한 만큼만 설명합니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `#[cfg]`, `cfg!`, `cfg_select!`, `#[cfg_attr]`의 차이(특히 거짓인 쪽 코드가 타입 검사를 받는지)를 설명하고, 빠진 플랫폼 없이 플랫폼별 코드를 작성할 수 있다.
2. 기능 통합 때문에 기능이 더하기만 해야 하는 이유를 설명하고, 동작을 바꾸거나 서로 함께 켤 수 없는 기능을 더하기만 하는 기능으로 다시 설계할 수 있다. 통합된 기능 집합을 `cargo tree`로 확인할 수 있다.
3. 공개 필드나 트레이트 메서드를 기능으로 가렸을 때 쓰는 쪽 코드가 어떻게 깨지는지 설명하고, 비공개 필드와 생성자, 별도 트레이트 같은 대안을 고를 수 있다.
4. 선택적 의존성과 `dep:` 문법으로 기능 목록을 관리하고, `unexpected_cfgs` 린트로 기능 이름과 cfg 이름의 오타를 잡을 수 있다.
5. 기능 N개가 만드는 2^N가지 빌드를 설명하고, `--no-default-features`, `--all-features`, `required-features`로 기능 조합을 검증할 수 있다.

## 2. 핵심 개념

2.1절에서 조건부 컴파일과 기능의 관계를 정리하고, 2.2~2.6절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 지침 2~5는 책이 권하는 것이고, 지침 1 전체와 각 지침에 더한 도구(`unexpected_cfgs`, `required-features` 등)는 책 이후의 Rust와 Cargo 문서를 근거로 더했습니다(2.7절).

### 2.1 지도: 조건부 컴파일과 기능

**cfg 조건.** 조건부 컴파일(conditional compilation)은 컴파일할 때 정해진 이름과 값에 따라 코드를 남기거나 지우는 일입니다. 이름만 있는 것(`unix`, `debug_assertions`)과 이름과 값의 쌍(`target_os = "macos"`)이 있고, `all(…)`, `any(…)`, `not(…)`과 늘 참·거짓인 `true`·`false`(1.88)로 묶어 cfg 조건을 만듭니다. 값을 정하는 쪽은 넷입니다.

| 정하는 쪽 | 예 | 확인 방법 |
| --- | --- | --- |
| 툴체인(대상 플랫폼) | `target_os`, `target_family`, `unix`, `windows`, `target_arch`, `target_pointer_width`, `target_endian` | `rustc --print cfg` |
| 빌드 설정 | `debug_assertions`(디버그 빌드), `panic = "unwind"`, `test`(테스트 빌드) | `rustc --print cfg`(`test`는 `cargo test`로 테스트를 빌드할 때만 켜짐) |
| Cargo의 기능 | `feature = "fast-hash"` | `cargo build -v` |
| 빌드하는 사람 | `--cfg expensive_checks` (`RUSTFLAGS`로 넘김) | 예제 03 |

이 환경(macOS, Apple Silicon)에서 `rustc --print cfg`는 `target_arch="aarch64"`, `target_family="unix"`, `target_os="macos"`, `target_pointer_width="64"`, `target_endian="little"`, `panic="unwind"`, `debug_assertions` 등을 출력했습니다(`target_feature` 줄 수십 개는 줄임). `rustc --print cfg --target wasm32-unknown-unknown`처럼 다른 타깃을 주면 그 타깃의 값을 볼 수 있습니다. `target_family="wasm"`, `target_os="unknown"`, `target_pointer_width="32"`, `panic="abort"`가 나왔습니다.

**cfg를 쓰는 네 가지 형태.** 거짓인 쪽 코드를 어떻게 다루는지가 서로 다릅니다.

| 형태 | 하는 일 | 거짓인 쪽 코드 | 예제 |
| --- | --- | --- | --- |
| `#[cfg(조건)]` | 붙은 항목·문장·필드를 남기거나 지움 | 지워지므로 타입 검사를 받지 않음 | 01의 `v1`, 3번 구역 |
| `#[cfg_attr(조건, 속성)]` | 조건이 참일 때만 속성을 붙임 | 속성만 빠짐 | 01의 4번 구역 |
| `cfg!(조건)` | `true`/`false` 리터럴이 됨 | 남아서 컴파일되고 타입 검사를 받음 | 01의 `line_ending`, 02의 `v1` |
| `cfg_select! { 조건 => {…} _ => {…} }` | 처음 참인 갈래만 남김(1.95) | 지워짐. 참인 갈래가 없으면 컴파일 에러 | 01의 `v2` |

**기능은 `feature`라는 이름의 cfg 값입니다.** `cargo build -v -p er-ch26-feature-creep --example 02_additive_features --features fast-hash`의 rustc 호출에서 cfg와 관련된 인자만 골라 보면 다음과 같습니다(순서는 정리함).

```text
--cfg 'feature="fast-hash"'
--check-cfg 'cfg(feature, values("fast-hash"))'
--check-cfg 'cfg(docsrs,test)'
--check-cfg 'cfg(expensive_checks)'
```

Cargo는 켠 기능마다 `--cfg 'feature="…"'`를 넘기고, 선언된 기능 목록 전체를 `--check-cfg`로 알려 줍니다(2.5절). 컴파일러가 보기에 기능은 `target_os`와 다를 것 없는 cfg 값이므로, 기능으로 가린 코드에도 위 표가 그대로 적용됩니다. 한 가지 다른 점은 크레이트마다 따로 정해진다는 것입니다. 크레이트 안의 `cfg(feature = "x")`는 **자기 크레이트의** 기능만 가리킵니다. 의존하는 크레이트의 기능이 켜졌는지는 물어볼 방법이 없습니다. 2.4절의 문제가 여기서 나옵니다.

**기능 통합.** Part 2 ch3에서 본 대로, 같은 크레이트를 그래프의 여러 곳에서 다른 기능으로 요청하면 Cargo는 요청된 기능의 **합집합**으로 그 크레이트를 한 번만 빌드합니다. 기본 기능(`default`)도 똑같이 합쳐집니다. 그래서 의존성 선언에 `default-features = false`를 적는 것은 "나는 요청하지 않는다"는 뜻일 뿐, 꺼진다는 보장이 아닙니다. 리졸버 2와 3은 빌드 의존성, 지금 빌드하지 않는 플랫폼 전용 의존성, 필요 없는 dev-dependency의 기능은 합치지 않지만, 일반 의존성끼리는 여전히 합칩니다. 이 챕터의 프로젝트는 모두 리졸버 3이고, 일반 의존성끼리의 통합만 다룹니다.

### 2.2 지침 1: 플랫폼별 코드에는 나머지 갈래를 두고, 값만 다르면 `cfg!`로 한 벌만 쓴다

**문제 상황.** 설정 파일을 둘 디렉터리가 운영체제마다 다릅니다. macOS, Linux, Windows의 경로를 각각 돌려주는 함수가 필요합니다.

**비권장 코드.** 플랫폼마다 함수를 한 벌씩 정의합니다([`01_cfg_forms.rs`](examples/01_cfg_forms.rs)의 `v1`).

```rust
#[cfg(target_os = "macos")]
pub fn config_dir() -> &'static str { "~/Library/Application Support" }
#[cfg(target_os = "linux")]
pub fn config_dir() -> &'static str { "~/.config" }
#[cfg(windows)]
pub fn config_dir() -> &'static str { "%APPDATA%" }
```

macOS에서는 잘 빌드되고 잘 동작합니다. 문제는 지금 보이지 않는 두 가지입니다.

- **목록에 없는 플랫폼에서는 함수가 없습니다.** FreeBSD나 wasm에서 빌드하면 `config_dir`이 통째로 지워져, 부르는 코드가 모두 E0425(이름을 찾을 수 없음)로 실패합니다. 예제는 이것을 피하려고 부르는 쪽에도 `#[cfg(any(target_os = "macos", target_os = "linux", windows))]`를 되풀이했습니다. 조건을 두 곳에서 맞춰야 하는 것 자체가 비용입니다.
- **다른 플랫폼의 판은 검사받지 않습니다.** `#[cfg]`로 지운 코드는 컴파일러가 타입을 보기 전에 사라집니다. 예제의 3번 구역은 `&str`을 `u32`로 돌려주는 틀린 함수를 `#[cfg(false)]` 아래에 두었는데도 경고 하나 없이 빌드됩니다. macOS에서 빌드하는 동안 `#[cfg(windows)]` 판도 같은 처지라서, 그 안의 실수는 누군가 Windows에서 빌드할 때 비로소 드러납니다.

**권장 코드.** 나머지 갈래를 반드시 둡니다. 1.95부터는 `cfg_select!`가 이것을 한 곳에 모아 줍니다(`v2`).

```rust
pub fn config_dir() -> &'static str {
    cfg_select! {
        target_os = "macos" => { "~/Library/Application Support" }
        target_os = "linux" => { "~/.config" }
        windows => { "%APPDATA%" }
        _ => { "./config" }
    }
}
```

`_` 갈래가 있으니 어떤 플랫폼에서든 함수가 있고, 부르는 쪽은 조건 없이 부릅니다. `cfg_select!`가 없던 때에는 마지막 판에 `#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]`를 붙여 같은 일을 했습니다. 조건을 손으로 맞춰야 하므로 하나를 더할 때 빠뜨리기 쉽습니다.

**값만 다르면 `cfg!`로 충분합니다.** 줄 끝 문자처럼 갈래마다 값만 다르고 양쪽 코드가 모든 플랫폼에서 컴파일될 수 있다면, `if cfg!(windows) { "\r\n" } else { "\n" }` 한 벌로 씁니다. `cfg!`는 리터럴이 될 뿐이므로 어느 플랫폼에서 빌드하든 두 갈래가 모두 타입 검사를 받습니다.

`#[cfg_attr]`는 속성에 같은 일을 합니다. 예제의 `BuildInfo`는 `#[cfg_attr(debug_assertions, derive(Debug))]`라서 디버그 빌드에서만 `Debug`를 구현합니다. `--release`로 실행하면 4번 구역의 출력이 `Debug 구현이 없습니다`로 바뀝니다. 기능과 함께 쓰면 "기능이 켜졌을 때만 derive를 붙인다"(`#[cfg_attr(feature = "…", derive(…))]`)는 모양이 됩니다. 기능이 트레이트 구현을 더하기만 하므로 2.3절의 지침과도 맞습니다.

**따르지 않아도 되는 경우.**

- **다른 플랫폼에는 아예 없는 API를 쓸 때.** `std::os::unix`의 확장 트레이트처럼 특정 플랫폼에서만 존재하는 항목을 쓰는 코드는 `cfg!`로 감쌀 수 없습니다. 거짓인 갈래도 컴파일되므로 없는 이름을 찾지 못합니다(4.1절의 E0425). 이때는 `#[cfg]`가 맞고, 대신 나머지 갈래를 둡니다.
- **지원하지 않는 플랫폼을 일부러 거부할 때.** `cfg_select!`에서 `_` 갈래를 빼면, 목록 밖 플랫폼에서는 부르는 곳마다 흩어진 E0425 대신 `` none of the predicates in this `cfg_select` evaluated to true ``라는 에러 하나가 `cfg_select!` 자리에서 납니다(어느 갈래도 맞지 않는 파일을 컴파일해 확인). 의도를 알리는 데는 이쪽이 낫습니다.

### 2.3 지침 2: 기능은 더하기만 한다

**문제 상황.** 기능을 켜면 무언가가 **더해지기만** 해야 합니다. 이런 기능을 더하기만 하는(가산적, additive) 기능이라고 부릅니다. 반대로 기능을 켜면 있던 동작이 바뀌거나 사라지는 경우를 생각해 봅시다. 기능은 그래프 어디선가 켜지면 모두에게 켜지므로, 그 크레이트를 쓰는 다른 누군가의 동작도 함께 바뀝니다. 책은 서로 함께 켤 수 없는 기능을 특히 경계합니다. 서로 다른 사용자가 각각 하나씩 켜면 둘이 함께 켜지기 때문입니다.

**비권장 코드 ①: 기능이 같은 함수의 결과를 바꿉니다.** [`02_additive_features.rs`](examples/02_additive_features.rs)의 `v1::fingerprint`는 문자열의 지문(해시)을 계산합니다. `fast-hash` 기능이 켜지면 더 단순한 알고리즘으로 바뀝니다.

```rust
pub fn fingerprint(s: &str) -> u32 {
    if cfg!(feature = "fast-hash") { super::poly31(s) } else { super::fnv1a(s) }
}
```

기본 빌드에서는 디스크에 저장해 둔 지문 3개를 모두 다시 찾지만, `--features fast-hash`로 실행하면 0개를 찾습니다(3절의 출력). 컴파일 에러도 경고도 없습니다. 명령줄의 `--features`는 "그래프의 다른 크레이트가 이 기능을 켠 상황"을 대신합니다. 내 크레이트는 그 기능을 켠 적이 없는데, 같은 빌드에 그 기능을 켜는 크레이트가 하나 들어오는 순간 저장 형식이 깨집니다.

**비권장 코드 ②: 서로 함께 켤 수 없는 기능.** [`projects/exclusive_features/broken`](projects/exclusive_features/broken)은 날짜 서식 라이브러리 `datefmt`에 기능 `iso`(기본)와 `korean`을 두고, 둘이 같은 이름의 함수 `render`를 서로 다르게 정의합니다.

```rust
#[cfg(feature = "iso")]
pub fn render(d: &Date) -> String { /* "2026-11-19" */ }

#[cfg(feature = "korean")]
pub fn render(d: &Date) -> String { /* "2026년 11월 19일" */ }
```

`report`는 `datefmt`를 기본 기능으로 쓰고, `invoice`는 두 기능이 충돌한다는 것을 알고 `default-features = false, features = ["korean"]`으로 씁니다. 둘 다 각자 테스트를 통과하고, `cargo tree -p report -f "{p} {f}"`와 `cargo tree -p invoice -f "{p} {f}"`는 `datefmt`의 기능을 각각 `default,iso`와 `korean`으로 보여 줍니다. 그런데 둘을 함께 쓰는 `app`은 빌드되지 않습니다.

```text
$ cargo tree -p app -f "{p} {f}"
app v0.1.0 (…/broken/app)
├── invoice v0.1.0 (…/broken/invoice)
│   └── datefmt v0.1.0 (…/broken/datefmt) default,iso,korean
└── report v0.1.0 (…/broken/report)
    └── datefmt v0.1.0 (…/broken/datefmt) default,iso,korean
```

`app`에서 `datefmt`는 `default,iso,korean`으로 한 번 빌드되고, `invoice`가 끈 기본 기능도 `report` 쪽 요청으로 켜졌습니다. 에러는 `datefmt` 안에서 납니다(3절). `app`의 작성자는 잘못한 것이 없고, 고칠 수 있는 것도 없습니다. 남의 크레이트가 켠 기능을 끌 방법이 없기 때문입니다. `datefmt`에는 Cargo Book이 "어쩔 수 없을 때" 권하는 안전장치 `compile_error!`도 넣었지만, 메시지가 친절해질 뿐 상황은 같습니다. 또 워크스페이스 안에서는 **함께 빌드하는 패키지끼리** 기능이 합쳐집니다. `cargo test -p report`와 `cargo test -p invoice`는 각각 통과하지만, `cargo test -p report -p invoice`는 `app` 없이도 같은 에러로 실패합니다. `cargo tree -e features -i datefmt -p app`을 실행하면 각 기능을 누가 켰는지 거꾸로 따라갈 수 있습니다.

**권장 코드.** 기능은 새 이름의 함수를 더하기만 합니다. 02의 `v2`는 `fingerprint`를 늘 FNV-1a로 두고, `fast-hash`가 켜지면 `fingerprint_fast`를 **더합니다**. 어떤 빌드에서도 `fingerprint`의 결과는 같습니다.

[`projects/exclusive_features/additive`](projects/exclusive_features/additive)는 `datefmt`를 같은 생각으로 다시 설계했습니다.

```toml
[dependencies]
weekday = { path = "../weekday", optional = true }

[features]
default = ["korean"]
korean = ["dep:weekday"]
```

```rust
pub fn iso(d: &Date) -> String { /* 늘 있음 */ }

#[cfg(feature = "korean")]
pub fn korean(d: &Date) -> String { /* weekday로 요일을 붙임 */ }
```

ISO 서식은 의존성이 필요 없으므로 기능으로 가리지 않았습니다. 한국어 서식은 요일 계산 크레이트 `weekday`(무거운 의존성을 흉내 냄)를 끌어오므로 기능 하나로 묶었습니다. `report`는 `default-features = false`로 `weekday`를 피하고, `invoice`는 `korean`을 켭니다. `app`에서는 `korean`이 켜지고 `weekday`도 빌드되지만, `report`가 부르는 `iso`는 그대로이므로 아무것도 깨지지 않습니다.

```text
$ cargo tree -p app -f "{p} {f}"
app v0.1.0 (…/additive/app)
├── invoice v0.1.0 (…/additive/invoice)
│   └── datefmt v0.1.0 (…/additive/datefmt) default,korean
│       └── weekday v0.1.0 (…/additive/weekday)
└── report v0.1.0 (…/additive/report)
    └── datefmt v0.1.0 (…/additive/datefmt) default,korean (*)
```

더하기만 하는 기능에는 "켜면 덜어진다"는 모양이 없습니다. `no_std`를 지원할 때도 `no_std` 기능으로 `std`를 끄는 대신 `std` 기능으로 `std`를 쓰는 코드를 켜는 것이 같은 원리입니다(Cargo Book, `ch33-no-std`).

**따르지 않아도 되는 경우.** 책은 예외를 두지 않습니다. 정말 배타적인 선택(예: 서로 다른 백엔드 둘이 같은 심볼을 정의)이 필요하다면 Cargo Book의 "Mutually exclusive features" 절이 권하는 대안을 봅니다.

- **기능을 패키지로 나눕니다.** `datefmt-iso`와 `datefmt-korean`처럼 따로 두면 한 빌드에 둘 다 들어와도 부딪히지 않습니다.
- **실행 중에 고릅니다.** 두 동작을 모두 컴파일해 두고 설정 파일, 명령줄 인자, 환경 변수로 고릅니다.
- **우선순위를 정합니다.** 둘 다 켜지면 한쪽이 이기게 합니다. 빌드는 되지만 진 쪽을 기대한 크레이트의 동작이 조용히 바뀝니다(연습 1에서 확인합니다).
- 그래도 남는다면 `compile_error!`로 함께 켜졌음을 분명히 알립니다. 의존성으로 쓰이지 않는 바이너리 패키지의 기능은 다른 크레이트가 켤 수 없으므로 통합 문제가 생기지 않지만, `--all-features`로 검사하는 도구에서는 여전히 실패합니다.

### 2.4 지침 3: 공개 필드와 트레이트 메서드를 기능으로 가리지 않는다

**문제 상황.** 알림 라이브러리 `notice`에 첨부 파일 지원을 `attachments` 기능으로 더합니다. 메시지 구조체에 첨부 파일 필드가, 통로 트레이트에 "첨부 파일을 몇 개까지 받는가" 메서드가 필요합니다. 기능을 모르는 사용자에게는 영향이 없게 하고 싶습니다.

**비권장 코드.** [`projects/gated_api/broken`](projects/gated_api/broken)의 `notice`는 공개 필드와 필수 메서드를 기능으로 가립니다.

```rust
pub struct Message {
    pub to: String,
    pub body: String,
    #[cfg(feature = "attachments")]
    pub attachments: Vec<String>,
}

pub trait Channel {
    fn name(&self) -> &str;
    fn deliver(&self, msg: &Message) -> String;
    #[cfg(feature = "attachments")]
    fn max_attachments(&self) -> usize;
}
```

경보 크레이트 `alerts`는 이 기능을 모르고, `Message { to, body }` 리터럴과 `impl Channel for Sms { name, deliver }`를 씁니다. 혼자서는 테스트까지 통과합니다. 그런데 애플리케이션 `app`이 자기 용도로 `notice`의 `attachments`를 켜자, `app`이 고칠 수 없는 `alerts` 안에서 두 에러가 납니다.

```text
error[E0046]: not all trait items implemented, missing: `max_attachments`
  --> alerts/src/lib.rs:12:1
   |
12 | impl Channel for Sms {
   | ^^^^^^^^^^^^^^^^^^^^ missing `max_attachments` in implementation
   |
   = help: implement the missing item: `fn max_attachments(&self) -> usize { todo!() }`

error[E0063]: missing field `attachments` in initializer of `Message`
  --> alerts/src/lib.rs:26:5
   |
26 |     Message {
   |     ^^^^^^^ missing `attachments`
```

`alerts`의 작성자가 help대로 메서드와 필드를 미리 적어 두면 어떻게 될까요? 이번에는 기능이 꺼진 `alerts` 혼자의 빌드가 E0407(`` method `max_attachments` is not a member of trait `Channel` ``)과 E0560(`` struct `Message` has no field named `attachments` ``)으로 실패합니다(프로젝트의 사본으로 확인). 책의 말대로 쓰는 쪽은 그 기능이 켜졌는지 알 수 없으므로, 적어도 안 적어도 어느 한 빌드에서 깨집니다. 2.1절에서 본 대로 `alerts`가 `cfg(feature = "attachments")`로 물어볼 수도 없습니다. 그것은 `alerts` 자신의 기능을 가리킵니다. [`compile_fail/e0560_gated_pub_field.rs`](compile_fail/e0560_gated_pub_field.rs)는 이 "양쪽 다 실패"를 한 파일로 보여 줍니다(4.2절).

**권장 코드.** [`projects/gated_api/fixed`](projects/gated_api/fixed)는 기능이 공개 API를 바꾸지 않고 더하기만 하게 고쳤습니다.

```rust
pub struct Message {
    to: String,
    body: String,
    #[cfg(feature = "attachments")]
    attachments: Vec<String>, // 비공개라 밖에서는 보이지 않음
}

impl Message {
    pub fn new(to: impl Into<String>, body: impl Into<String>) -> Self { /* … */ }
    #[cfg(feature = "attachments")]
    pub fn attach(mut self, file: impl Into<String>) -> Self { /* … */ }
}

// Channel은 name과 deliver만 가집니다(기능과 상관없음).
#[cfg(feature = "attachments")]
pub trait AttachmentChannel: Channel {
    fn max_attachments(&self) -> usize;
    fn deliver_checked(&self, msg: &Message) -> Result<String, String> { /* 기본 구현 */ }
}
```

- **필드는 비공개로, 만들기는 생성자로.** 쓰는 쪽은 `Message::new("운영팀", …)`로 만들므로 숨은 필드가 몇 개든 같은 코드가 모든 빌드에서 컴파일됩니다. 기능은 `attach`와 `attachments` 메서드를 더할 뿐입니다. 비공개 필드는 기능으로 가려도 괜찮습니다. 그 필드를 아는 코드는 자기 기능을 `cfg`로 알 수 있는 `notice` 안에만 있습니다. `new`의 구조체 리터럴에서도 필드 하나에 `#[cfg]`를 붙였습니다. 비공개 필드와 접근자의 일반론은 [ch22](../ch22-minimize-visibility/README.md)에서 다룹니다.
- **기능에 딸린 메서드는 별도 트레이트로.** `Channel`의 필수 메서드는 어떤 빌드에서나 둘입니다. 첨부 파일을 보낼 수 있는 통로만 `AttachmentChannel`을 더 구현하고, 그 트레이트의 이름을 써야 하므로 구현하는 크레이트는 기능을 직접 켤 수밖에 없습니다. 기능을 모르는 `alerts`는 broken 판과 같은 `impl Channel for Sms`로 그대로 컴파일됩니다.

`fixed/`에서 `app`은 broken 판과 똑같이 기능을 켜지만 빌드되고, 모든 패키지가 `--no-default-features`와 `--all-features` 양쪽에서 clippy와 테스트를 통과합니다(3절). 다른 대안도 사본에서 확인했습니다.

- **늘 있는 필드.** 필드의 타입이 선택적 의존성에서 오지 않는다면 `pub attachments: Option<Vec<String>>`처럼 기능과 상관없이 늘 둡니다. `alerts`는 `attachments: None`을 적어야 하지만, 한 번 고치면 모든 빌드에서 컴파일됩니다. 기능이 꺼진 빌드에서는 그 값을 무시하거나 에러로 돌려줍니다.
- **`#[non_exhaustive]`.** 구조체에 붙이면 크레이트 밖의 구조체 리터럴이 **어느 빌드에서나** E0639(`cannot create non-exhaustive struct using struct expression`)가 되어, 처음부터 생성자를 쓰게 됩니다. 깨지는지가 다른 크레이트의 기능에 달려 있지 않다는 점이 중요합니다.
- **열거형 배리언트도 같습니다.** `#[cfg(feature = "…")]`로 가린 배리언트가 켜지면, 와일드카드 없이 모든 배리언트를 나열한 쓰는 쪽 `match`가 E0004(`non-exhaustive patterns`)로 깨집니다(단일 파일로 확인).

**따르지 않아도 되는 경우.**

- **기본 구현이 있고, 밖에서 재정의할 일이 없는 메서드.** 책이 부분적인 예외로 드는 경우입니다. `#[cfg(feature = "attachments")] fn max_attachments(&self) -> usize { 0 }`처럼 기본 구현을 주면 기능을 모르는 구현은 어느 빌드에서나 컴파일됩니다(사본에서 `cargo test -p alerts`와 `cargo test -p alerts --features notice/attachments` 모두 통과). 하지만 `Sms`가 기본값 0이 아닌 값을 주려고 재정의하면, 기능이 꺼진 빌드에서 E0407이 납니다. 재정의하려면 기능을 직접 켜야 하므로, 재정의가 필요 없는 메서드에만 씁니다.
- **기능이 타입이나 함수를 통째로 더할 때.** `AttachmentChannel`이나 `Message::attach`처럼 새 항목을 더하는 것은 더하기만 하는 변경입니다. 그 항목을 쓰는 크레이트는 기능을 직접 켜야 하고, 켜지 않았으면 그 크레이트 혼자의 빌드에서 바로 드러납니다.

### 2.5 지침 4: 기능 이름은 한 곳에서 관리하고 린트로 확인한다

**문제 상황.** 크레이트의 기능 목록은 생각보다 넓습니다. 책은 어떤 크레이트의 기능을 알려면 `[features]` 테이블뿐 아니라 선택적 의존성도 함께 봐야 하고, 기능 이름과 의존성 이름이 한 이름 공간을 쓰니 이름을 조심해서 고르라고 말합니다. 또 기능 이름을 코드에 잘못 적으면 그 조건은 조용히 늘 거짓이 됩니다.

**선택적 의존성은 같은 이름의 기능을 만듭니다.** `optional = true`인 의존성을 `[features]`에서 `dep:`으로 가리키지 않으면, Cargo는 의존성과 같은 이름의 기능을 암묵적으로 만듭니다. 2024 에디션의 작은 패키지(`dep = { path = …, optional = true }`, `[features] extra = []`)를 `cargo metadata --no-deps`로 보면 `features`에 `"extra": []`와 함께 `"dep": ["dep:dep"]`이 들어 있었고, `--features dep`으로 그 의존성을 켤 수 있었습니다. 내부 구현일 뿐인 의존성의 이름이 공개 기능이 되는 것입니다.

**비권장 코드.** 그 상태에서 같은 이름의 기능을 따로 정의하면(`[features] dep = []`) 매니페스트를 읽을 때 에러가 납니다.

```text
error: failed to parse manifest at `<경로>/Cargo.toml`

Caused by:
  optional dependency `dep` is not included in any feature
  Make sure that `dep:dep` is included in one of features in the [features] table.
```

**권장 코드: `dep:`으로 이름 공간을 나눕니다.** `[features]`에서 `"dep:weekday"`처럼 가리키면 암묵적 기능이 생기지 않습니다. `additive/datefmt`의 기능은 `default`와 `korean`뿐이고, `weekday`는 바깥에서 켤 수 없습니다.

```text
$ cargo build -p datefmt --features weekday
error: package `datefmt v0.1.0 (…/additive/datefmt)` does not have feature `weekday`

help: an optional dependency with that name exists, but the `features` table includes it with the "dep:" syntax so it does not have an implicit feature with that name
Dependency `weekday` would be enabled by these features:
	- `korean`
```

`dep:`을 쓰면 의존성과 이름이 같지만 뜻이 다른 기능도 만들 수 있습니다. `dep = []`과 `other = ["dep:dep"]`을 함께 두자 `--features dep`은 의존성을 끌어오지 않았고 `--features other`만 끌어왔습니다(`cargo tree -e features`로 확인).

**권장 코드: `unexpected_cfgs` 린트를 끄지 않습니다.** [`03_unexpected_cfgs.rs`](examples/03_unexpected_cfgs.rs)의 `v1`은 매니페스트의 `fast-hash`(하이픈)를 코드에 `fast_hash`(밑줄)로 적었습니다. 이 조건은 어느 빌드에서도 거짓이고, `--features fast-hash`로 실행해도 `v1::has_fast() = false`입니다. 컴파일 에러는 나지 않습니다. 그러나 2.1절에서 본 `--check-cfg` 덕분에 rustc가 오타를 압니다. 예제의 `#[expect]` 줄을 지우면 이렇게 경고합니다(같은 모양의 두 번째 경고는 줄임).

```text
warning: unexpected `cfg` condition value: `fast_hash`
  --> appd-a/ch26-feature-creep/examples/03_unexpected_cfgs.rs:24:11
   |
24 |     #[cfg(feature = "fast_hash")]
   |           ^^^^^^^^^^-----------
   |                     |
   |                     help: there is a expected value with a similar name: `"fast-hash"`
   |
   = note: expected values for `feature` are: `fast-hash`
   = help: consider adding `fast_hash` as a feature in `Cargo.toml`
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: requested on the command line with `-W unexpected-cfgs`
```

마지막 note는 이 챕터의 `Cargo.toml`이 `[lints.rust]`에서 이 린트를 설정했기 때문에 나옵니다. 그 설정이 없는 패키지에서는 `` `#[warn(unexpected_cfgs)]` on by default ``로 나왔습니다. 이 린트는 기본으로 켜져 있습니다.

**사용자 정의 cfg 이름은 `[lints.rust]`에 알립니다.** 예제의 `expensive_checks`는 기능이 아니라, 빌드하는 사람이 `RUSTFLAGS='--cfg expensive_checks'`로 거는 cfg 이름입니다. 알리지 않으면 `cfg!(expensive_checks)`마다 `` unexpected `cfg` condition name: `expensive_checks` ``가 나고, help가 해결 방법 셋(기능으로 바꾸기, 아래 설정, 빌드 스크립트의 `cargo::rustc-check-cfg`)을 제안합니다. 이 챕터의 `Cargo.toml`은 두 번째를 씁니다.

```toml
[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(expensive_checks)'] }
```

기능과 사용자 정의 cfg는 퍼지는 방식이 다릅니다. 기능은 의존성 선언을 따라 통합되므로 그래프의 아무 크레이트나 켤 수 있지만, `RUSTFLAGS`의 `--cfg`는 빌드하는 사람만 걸 수 있고 빌드되는 모든 크레이트에 함께 걸립니다(의존성이 있는 작은 프로젝트에서 `cargo build -v`로 두 크레이트의 rustc 호출에 모두 `--cfg expensive_checks`가 붙는 것을 확인). 느린 자체 검사나 검사 도구 전용 코드처럼 라이브러리 사용자가 켤 일이 아닌 설정에 맞습니다.

**`#[expect]`를 붙이는 자리.** 03은 오타를 보여 주려고 `v1` 모듈에 `#[expect(unexpected_cfgs, reason = "…")]`를 붙였습니다. 오타 난 `#[cfg]`와 같은 항목에 붙이면 경고를 막지 못했습니다. 조건이 거짓이라 그 항목이 `#[expect]`째 지워지기 때문입니다. 감싸는 모듈이나 함수에 붙입니다.

**따르지 않아도 되는 경우.** 빌드 스크립트가 cfg 값을 정하는 경우에는 `[lints.rust]` 대신 빌드 스크립트에서 `cargo::rustc-check-cfg`로 알립니다(경고의 help가 제안하는 셋째 방법). 빌드 스크립트는 Part 3에서 다룹니다.

### 2.6 지침 5: 기능을 늘리기 전에 조합 수를 센다

**문제 상황.** 기능 하나는 빌드를 둘로 나눕니다. 서로 독립인 기능이 N개면 가능한 빌드는 2^N가지이고, 기능 통합 때문에 그 가운데 무엇이 실제 사용자의 빌드가 될지 크레이트 작성자는 고를 수 없습니다. 2.2절에서 본 대로 `#[cfg]`로 지운 코드는 검사받지 않으므로, 한 조합에서만 컴파일되는 코드의 실수는 그 조합을 빌드하기 전까지 숨어 있습니다. 책은 CI에서 모든 조합을 검사하라고 권합니다.

| 패키지 | 기능 | 빌드 수 | 결과 |
| --- | --- | --- | --- |
| 이 챕터 패키지 | `fast-hash` | 2 | 둘 다 통과 |
| `exclusive_features/broken`의 `datefmt` | `iso`, `korean` | 4 | 기능 없음: 통과(그러나 `render`가 없음), 하나씩: 통과, 둘 다: 실패 |
| `exclusive_features/additive`의 `datefmt` | `korean` | 2 | 둘 다 통과 |
| `gated_api/fixed`의 `notice` | `attachments` | 2 | 둘 다 통과 |

broken 판을 additive 판으로 바꾸며 기능이 둘에서 하나로 줄었습니다. 의존성이 필요 없는 ISO 서식을 기능으로 두지 않았기 때문입니다. 기능은 의존성 그래프를 줄일 때(이 경우 `weekday`) 쓰는 도구이고, 작은 코드 조각마다 기능을 두면 조합만 늘어납니다. 책이 `no_std` 지원을 위한 `std`·`alloc` 기능을 흔한 예로 드는 것도 의존성(표준 라이브러리)을 덜어 내는 기능이기 때문입니다(`ch33-no-std`).

**비권장 코드.** 기본 기능으로만 검사합니다. Cargo Book의 "Feature combinations" 절에 따르면 테스트와 clippy 같은 도구는 따로 지정하지 않으면 기본 기능으로만 실행됩니다. 이 챕터의 예제를 기본 기능으로만 검사했다면 `04_required_features`는 한 번도 컴파일되지 않습니다.

**권장 코드.** 최소한 `--no-default-features`와 `--all-features` 양쪽을 검사하고, 기능이 적으면 모든 조합을 돌립니다. 이 챕터를 검증한 반복문입니다(zsh).

```sh
for f in "" "--no-default-features" "--all-features"; do
  cargo build -p er-ch26-feature-creep --examples ${=f}
  cargo clippy -p er-ch26-feature-creep --examples ${=f} -- -D warnings
done
```

`--all-features`가 통과하려면 모든 기능이 함께 켜질 수 있어야 합니다. broken 판의 `datefmt`처럼 배타적인 기능이 있으면 이 기본 검사부터 실패하므로, 지침 2를 지키는 것이 검사의 전제이기도 합니다. 기능이 많을 때는 `cargo-hack`이 조합을 대신 돌려 줍니다. 그 README에 따르면 `--each-feature`(기능 하나씩), `--feature-powerset`(모든 조합), `--depth`(한 번에 켤 기능 수 제한), `--mutually-exclusive-features` 같은 옵션이 있습니다(예: `cargo hack check --feature-powerset`). 이 저장소에는 설치돼 있지 않아 실행해 확인하지 못했습니다. CI에 넣는 구성은 `ch32-continuous-integration`에서 다룹니다.

**기능 전용 예제·테스트는 `required-features`로 표시합니다.** [`04_required_features.rs`](examples/04_required_features.rs)는 기능 전용 함수를 조건 없이 부릅니다. 대신 `Cargo.toml`에 다음을 적었습니다.

```toml
[[example]]
name = "04_required_features"
required-features = ["fast-hash"]
```

기능 없이 `cargo build --examples`를 실행하면 Cargo가 이 예제를 건너뛰고(`-v`로 01~03만 컴파일되는 것을 확인), `cargo run --example 04_required_features`는 필요한 기능을 알려 주며 멈춥니다(3절). 같은 효과를 노리고 파일 맨 위에 `#![cfg(feature = "fast-hash")]`를 붙이면, 기능이 꺼진 빌드에서 `main`까지 지워져 E0601(`` `main` function not found in crate … ``)로 실패합니다(사본에서 확인).

**따르지 않아도 되는 경우.** 모든 조합을 돌리는 비용이 크면 Cargo Book의 말대로 프로젝트가 전략을 고릅니다(기능 없이·기본·전부만 돌리기, 특정 조합만 돌리기, 모든 조합 돌리기). 늘 함께 쓰이는 기능이라면 한 기능이 다른 기능을 켜게 해서(`b = ["a"]`처럼 적기, 책의 각주에도 이런 예가 있음) "b만 켠 빌드"를 없앨 수 있습니다. 서로 독립인 기능 둘의 빌드 4가지가 3가지로 줄어듭니다. `cargo-hack`의 `--depth`로 한 번에 켜는 기능 수를 제한하는 방법도 있습니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0과 cargo 1.100.0에서 확인한 차이는 다음과 같습니다.

- **`unexpected_cfgs` 린트가 생겼습니다.** 로컬 문서의 릴리스 노트에 따르면 rustc 1.79.0에서 `--check-cfg`가 안정화되었고(컴파일러 쪽만), Cargo 1.80.0에서 Cargo가 늘 `--check-cfg`를 넘기게 되었으며 `[lints.rust]`의 `check-cfg` 설정이 더해졌습니다. 책의 예제에는 매니페스트의 기능 이름과 코드의 이름이 하이픈과 밑줄로 어긋난 곳이 있는데, 지금은 같은 종류의 오타(`fast-hash`와 `fast_hash`)를 예제 03처럼 경고로 알려 줍니다.
- **책 이후에 생긴 cfg 문법.** `#[cfg(true)]`·`#[cfg(false)]`(1.88)와 `cfg_select!`(1.95)는 책에 없습니다. 둘 다 로컬 문서의 릴리스 노트로 확인했고, 예제 01에서 씁니다.
- **선택적 의존성의 암묵적 기능과 `dep:`.** 책은 `dep:`을 각주로만 다룹니다. `dep:`은 1.60.0에서 안정화되어 책보다 먼저 있었고, 2024 에디션에서도 `dep:`을 쓰지 않은 선택적 의존성은 암묵적 기능을 만듭니다(2.5절). Cargo의 변경 기록에는 2024 에디션에서 암묵적 기능을 없애려던 변경을 되돌린 항목(#14630)이 있습니다. 기능 이름과 의존성 이름이 한 이름 공간을 쓴다는 책의 설명은 `dep:`을 쓰지 않을 때의 이야기이고, `dep:`을 쓰면 둘을 나눌 수 있습니다.
- **기능 리졸버.** 책은 리졸버 버전을 말하지 않고 그래프 전체의 합집합으로 설명합니다. 리졸버 1의 동작과 같고, 리졸버 2·3에서는 2.1절의 세 경우가 빠집니다([Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.3절). 일반 의존성끼리는 이 챕터의 프로젝트처럼 리졸버 3에서도 합쳐집니다.
- **책에 없는 지침과 내용.** 다음은 책의 Item 26이 명시적으로 권하지 않는 것입니다(2026-11-19에 온라인판에서 확인). 책은 `cfg`·`cfg_attr`와 툴체인이 정하는 값, `[features]`와 선택적 의존성, 이름 공간, 기능 통합, 더하기만 하는 기능, 공개 필드·트레이트 메서드, 2^N 조합과 CI, `std`·`alloc` 기능을 다룹니다.
  - 지침 1 전체(나머지 갈래, `cfg!`로 한 벌, `cfg_select!`, 가려진 코드가 검사받지 않음): The Reference의 Conditional compilation 절과 예제 01, 4.1절의 컴파일 결과가 근거입니다.
  - 지침 2의 "동작을 바꾸는 기능"(02)과 배타적인 기능의 대안(`compile_error!`, 패키지 나누기, 실행 중에 고르기, 우선순위): Cargo Book의 Feature unification 절과 Mutually exclusive features 절이 근거입니다.
  - 지침 3의 대안(비공개 필드와 생성자, 별도 트레이트, 늘 있는 필드, `#[non_exhaustive]`)과 열거형 배리언트: 프로젝트와 사본에서 직접 컴파일한 결과(E0639, E0004)가 근거입니다.
  - 지침 4의 `unexpected_cfgs`, `[lints.rust]`의 `check-cfg`, `#[expect]`를 붙이는 자리: rustc Book의 Checking conditional configurations 절과 Cargo Specifics 절, 예제 03의 실험이 근거입니다.
  - 지침 5의 `--all-features`, `required-features`, `cargo-hack`: Cargo Book의 Features 장(Command-line feature options, Required features, Feature combinations)과 `cargo-hack` README가 근거입니다.

### 2.8 기억할 것

- `#[cfg]`로 지운 코드는 타입 검사를 받지 않습니다. 플랫폼별 코드에는 나머지 갈래를 두고, 값만 다르면 `cfg!`로 한 벌만 씁니다.
- 기능은 그래프 어디선가 켜지면 모두에게 켜지고, `default-features = false`는 꺼짐을 보장하지 않습니다. 그러니 기능은 새 항목을 더하기만 하고, 있던 동작을 바꾸거나 다른 기능과 부딪히면 안 됩니다.
- 공개 필드와 트레이트의 필수 메서드를 기능으로 가리면, 그 기능을 모르는 쓰는 쪽이 남의 선택 때문에 깨집니다. 비공개 필드와 생성자, 기능 전용 트레이트로 바꿉니다.
- 선택적 의존성은 `dep:`으로 가리켜 내부 이름을 숨기고, `unexpected_cfgs` 경고는 끄지 않습니다.
- 기능 N개는 빌드 2^N가지입니다. 의존성을 덜어 낼 때만 기능을 만들고, `--no-default-features`와 `--all-features`는 늘 검사합니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 끝나고(04는 `--features fast-hash`가 필요), 기본 기능·`--no-default-features`·`--all-features` 세 가지로 빌드와 clippy(`-D warnings`)를 통과합니다. `projects/`의 네 워크스페이스는 각자의 디렉터리에서 실행합니다. cargo가 출력하는 절대 경로는 `…/`로 줄였습니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_cfg_forms.rs`](examples/01_cfg_forms.rs) | 1 | 플랫폼마다 한 벌, 나머지 없음(`v1`) | `cfg_select!`와 `_`, `cfg!`(`v2`), `cfg_attr` | 2.1, 2.2 |
| [`02_additive_features.rs`](examples/02_additive_features.rs) | 2 | 기능이 결과를 바꾸는 `v1::fingerprint` | `fingerprint_fast`를 더하는 `v2` | 2.3 |
| [`03_unexpected_cfgs.rs`](examples/03_unexpected_cfgs.rs) | 4 | 기능 이름 오타 `fast_hash` | 같은 이름, `[lints.rust]`에 알린 cfg | 2.5 |
| [`04_required_features.rs`](examples/04_required_features.rs) | 5 | (주석) 파일 전체를 `#![cfg]`로 가림 | `required-features` | 2.6 |
| [`projects/exclusive_features`](projects/exclusive_features) | 2 | `broken/`: 배타적인 기능 | `additive/`: 더하기만 하는 기능 | 2.3 |
| [`projects/gated_api`](projects/gated_api) | 3 | `broken/`: 가린 공개 필드·메서드 | `fixed/`: 비공개 필드, 별도 트레이트 | 2.4 |

### 01_cfg_forms — cfg의 네 가지 형태

```sh
cargo run -p er-ch26-feature-creep --example 01_cfg_forms
```

1번과 2번 구역은 플랫폼마다 다릅니다. 아래는 macOS(aarch64)의 결과입니다.

```text
1. 툴체인이 정해 주는 cfg 값 (플랫폼마다 다름)
  target_os = "macos"          true
  target_os = "linux"          false
  unix                         true
  windows                      false
  target_arch = "aarch64"      true
  target_pointer_width = "64"  true
  target_endian = "little"     true
  debug_assertions             true
  panic = "unwind"             true
2. 플랫폼마다 다른 값
  비권장 v1::config_dir() = ~/Library/Application Support
  권장 v2::config_dir()   = ~/Library/Application Support
  권장 v2::line_ending()  = "\n"
3. #[cfg(false)] 아래의 타입 에러는 빌드를 막지 않았습니다
4. cfg_attr: 디버그 빌드 true, 포인터 64비트
  Debug 구현이 있습니다: BuildInfo { debug: true, pointer_bits: 64 }
```

`--release`를 붙이면 `debug_assertions`가 `false`가 되고, 4번 구역이 `4. cfg_attr: 디버그 빌드 false, 포인터 64비트`와 `  Debug 구현이 없습니다`로 바뀝니다. 1번 구역의 `show!`는 조건을 글자 그대로 출력하려고 만든 작은 `macro_rules!` 매크로입니다([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)).

### 02_additive_features — 동작을 바꾸는 기능과 더하는 기능

```sh
cargo run -p er-ch26-feature-creep --example 02_additive_features
```

```text
이 빌드의 fast-hash 기능: false
1. 비권장: 기능이 같은 함수의 결과를 바꿈
  v1::fingerprint("사과") = 0xf2849d82
  저장해 둔 지문 3개 중 3개를 다시 찾음
2. 권장: fingerprint는 그대로 두고 fingerprint_fast를 더함
  v2::fingerprint("사과") = 0xf2849d82
  저장해 둔 지문 3개 중 3개를 다시 찾음
  v2::fingerprint_fast는 이 빌드에 없음 (--features fast-hash로 켬)
```

```sh
cargo run -p er-ch26-feature-creep --example 02_additive_features --features fast-hash
```

```text
이 빌드의 fast-hash 기능: true
1. 비권장: 기능이 같은 함수의 결과를 바꿈
  v1::fingerprint("사과") = 0x9a3156bd
  저장해 둔 지문 3개 중 0개를 다시 찾음
2. 권장: fingerprint는 그대로 두고 fingerprint_fast를 더함
  v2::fingerprint("사과") = 0xf2849d82
  저장해 둔 지문 3개 중 3개를 다시 찾음
  v2::fingerprint_fast("사과") = 0x9a3156bd
```

`v1`의 두 줄만 바뀌었습니다. `v2::fingerprint_fast`를 부르는 곳은 `#[cfg(feature = "fast-hash")]` 블록으로 가렸습니다. `if cfg!(…)`로 감싸면 4.1절의 에러가 납니다.

### 03_unexpected_cfgs — 기능 이름의 오타와 사용자 정의 cfg

```sh
cargo run -p er-ch26-feature-creep --example 03_unexpected_cfgs
```

```text
Cargo가 켠 기능 fast-hash: false
1. 비권장: 코드에 fast_hash(밑줄)라고 적음
  v1::has_fast() = false
2. 권장: Cargo.toml과 같은 fast-hash(하이픈)
  v2::has_fast() = false
3. 사용자 정의 cfg expensive_checks: false
  정렬된 목록 [3, 8, 15, 21, 42], 추가 검사 0번
```

`--features fast-hash`를 붙이면 `v1::has_fast()`는 그대로 `false`이고, `v2::has_fast() = true`와 `  v2::FAST_NAME = poly31`이 나옵니다. 사용자 정의 cfg는 `RUSTFLAGS`로 켭니다. `RUSTFLAGS`를 처음 바꾸면 Cargo가 의존성까지 모든 크레이트를 다시 컴파일합니다. 아래 출력은 기본 실행과 같은 1번과 2번 구역을 줄였습니다.

```sh
RUSTFLAGS='--cfg expensive_checks' cargo run -p er-ch26-feature-creep --example 03_unexpected_cfgs
```

```text
3. 사용자 정의 cfg expensive_checks: true
  정렬된 목록 [3, 8, 15, 21, 42], 추가 검사 5번
```

### 04_required_features — 기능이 켜졌을 때만 있는 예제

```sh
cargo run -p er-ch26-feature-creep --example 04_required_features --features fast-hash
```

```text
fast-hash 기능이 켜진 빌드에만 있는 예제입니다
  사과 → 0x9a3156bd
  배 → 0x0003882b
  포도 → 0x9c9d6016
```

`--features fast-hash` 없이 실행하면 종료 코드 101로 멈춥니다.

```text
error: target `04_required_features` in package `er-ch26-feature-creep` requires the features: `fast-hash`
Consider enabling them by passing, e.g., `--features="fast-hash"`
```

### projects/exclusive_features — 배타적인 기능과 더하기만 하는 기능

두 디렉터리가 각자의 워크스페이스 루트입니다. 구성은 같습니다. `datefmt`(라이브러리), 그것을 쓰는 `report`와 `invoice`, 둘 다 쓰는 `app`이고, `additive/`에는 선택적 의존성 `weekday`가 더 있습니다.

```sh
cd appd-a/ch26-feature-creep/projects/exclusive_features/broken
cargo test -p report      # 통과: test result: ok. 1 passed
cargo test -p invoice     # 통과: test result: ok. 1 passed
cargo build -p app        # 실패 (종료 코드 101)
```

```text
error[E0428]: the name `render` is defined multiple times
  --> datefmt/src/lib.rs:30:1
   |
24 | pub fn render(d: &Date) -> String {
   | --------------------------------- previous definition of the value `render` here
...
30 | pub fn render(d: &Date) -> String {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `render` redefined here
   |
   = note: `render` must be defined only once in the value namespace of this module

error: datefmt: `iso`와 `korean` 기능은 함께 켤 수 없습니다
  --> datefmt/src/lib.rs:20:1
   |
20 | compile_error!("datefmt: `iso`와 `korean` 기능은 함께 켤 수 없습니다");
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

broken 판을 여러 명령으로 빌드한 결과입니다(모두 `cargo build`).

| 인자 | 결과 |
| --- | --- |
| `-p datefmt` / `-p datefmt --no-default-features` / `-p datefmt --no-default-features --features korean` | 통과 |
| `-p datefmt --features korean` (기본 `iso`에 더해짐) / `-p datefmt --all-features` | 실패: E0428과 `compile_error!` |
| `-p report` / `-p invoice` (따로) | 통과 |
| `-p report -p invoice` / `-p app` / 인자 없음(워크스페이스 전체) | 실패: 같은 두 에러 |

```sh
cd ../additive
cargo run -p app
```

```text
[주간 보고] 2026-11-19
발행일: 2026년 11월 19일 (목)
```

additive 판은 기본 기능, `--no-default-features`, `--all-features` 세 가지로 `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`를 모두 통과합니다(단위 테스트 `weekday` 1개, `datefmt` 2개, `report` 1개, `invoice` 1개). `--no-default-features`를 줘도 `invoice`가 `korean`을 요청하므로 워크스페이스 전체에서는 `datefmt` 테스트가 2개입니다. `cargo test -p datefmt --no-default-features`로 따로 돌리면 `korean` 없이 1개가 통과합니다.

### projects/gated_api — 기능으로 가린 공개 API

```sh
cd appd-a/ch26-feature-creep/projects/gated_api/broken
cargo test -p alerts      # 통과: test result: ok. 1 passed
cargo build -p app        # 실패 (종료 코드 101): alerts에서 E0046, E0063 (2.4절)
```

```sh
cd ../fixed
cargo run -p app
```

```text
[SMS] 운영팀 ← 디스크 사용량 93%
[이메일] 팀장 ← 주간 보고 (첨부 2개)
```

fixed 판은 기본 기능, `--no-default-features`, `--all-features` 세 가지로 `cargo clippy --workspace --all-targets -- -D warnings`와 `cargo test --workspace`를 통과합니다. 워크스페이스 전체로 돌리면 `app`이 기능을 켜므로 `notice`의 단위 테스트가 늘 3개(`attach`, 첨부 파일 수 초과 검사 포함)이고, `cargo test -p notice --no-default-features`로 따로 돌리면 1개입니다. `alerts`는 1개입니다.

## 4. 자주 하는 실수

`compile_fail/`의 두 파일은 기능이 꺼진 빌드와 켜진 빌드를 `rustc`의 `--cfg`로 흉내 냅니다. `--cfg 'feature="x"'`는 Cargo가 `--features x`일 때 넘기는 인자와 같습니다(2.1절). 확인 명령은 각 파일의 머리 주석에 있습니다.

### 4.1 `if cfg!(…)` 안에서 기능 전용 함수를 부름 — E0425

[`compile_fail/e0425_cfg_macro_branch.rs`](compile_fail/e0425_cfg_macro_branch.rs)는 `#[cfg(feature = "fast-hash")]`로 정의한 `fingerprint_fast`를 `if cfg!(feature = "fast-hash") { … }` 안에서 부릅니다.

```sh
rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0425_cfg_macro_branch.rs
```

```text
error[E0425]: cannot find function `fingerprint_fast` in this scope
  --> compile_fail/e0425_cfg_macro_branch.rs:24:30
   |
24 |         println!("{:#010x}", fingerprint_fast("사과"));
   |                              ^^^^^^^^^^^^^^^^ not found in this scope
   |
note: found an item that was configured out
  --> compile_fail/e0425_cfg_macro_branch.rs:17:4
   |
16 | #[cfg(feature = "fast-hash")]
   |       --------------------- the item is gated behind the `fast-hash` feature
17 | fn fingerprint_fast(s: &str) -> u32 {
   |    ^^^^^^^^^^^^^^^^
```

**원인.** `cfg!`는 `false`라는 리터럴이 될 뿐, `if`의 첫 갈래를 지우지 않습니다. 기능이 꺼진 빌드에서도 그 갈래는 컴파일되고, 거기서 부르는 함수는 `#[cfg]`로 지워져 없습니다. note가 "기능 때문에 지워진 항목이 있다"고 정확히 짚어 줍니다. `--cfg 'feature="fast-hash"'`를 주면 컴파일됩니다.

**해결.** 부르는 쪽도 `#[cfg(feature = "fast-hash")]`로 가립니다. 02의 마지막 블록이 그 모양입니다. `cfg!`는 양쪽 갈래가 모든 빌드에서 컴파일될 때만 씁니다(2.2절).

### 4.2 공개 필드를 기능으로 가림 — E0560 / E0063

[`compile_fail/e0560_gated_pub_field.rs`](compile_fail/e0560_gated_pub_field.rs)는 `notice`, `alerts`, `reports`를 한 파일의 모듈로 흉내 냅니다. `alerts`는 필드를 적지 않았고 `reports`는 적었습니다.

```sh
rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0560_gated_pub_field.rs
rustc --edition 2024 --out-dir <임시 디렉터리> --cfg 'feature="attachments"' compile_fail/e0560_gated_pub_field.rs
```

```text
error[E0560]: struct `Message` has no field named `attachments`
  --> compile_fail/e0560_gated_pub_field.rs:46:13
   |
46 |             attachments: vec!["보고서.pdf".to_string()],
   |             ^^^^^^^^^^^ `Message` does not have this field
   |
   = note: all struct fields are already assigned
```

둘째 명령(기능을 켠 빌드)은 `alerts`의 리터럴(31행)에서 2.4절과 같은 E0063 `` missing field `attachments` in initializer of `Message` ``로 실패합니다.

**원인.** 기능이 꺼진 빌드에서는 필드를 적은 `reports`가, 켜진 빌드에서는 적지 않은 `alerts`가 깨집니다. 두 빌드 모두에서 컴파일되는 구조체 리터럴은 없습니다. 실제로는 `alerts`와 `reports`가 다른 크레이트라서 `notice`의 기능을 `cfg`로 물어볼 수도 없습니다.

**해결.** 라이브러리 쪽에서 고칩니다. 필드를 비공개로 하고 생성자를 주거나, 필드를 늘 두거나, `#[non_exhaustive]`로 처음부터 리터럴을 막습니다(2.4절, `projects/gated_api/fixed`).

### 4.3 그 밖의 실수

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| 서로 함께 켤 수 없는 기능 | 각자는 통과, 함께 쓰는 빌드에서 E0428(같은 이름 두 번 정의) | 이름이 다른 항목을 더하는 기능으로 | 2.3, `exclusive_features` |
| `default-features = false`면 기본 기능이 꺼진다고 믿음 | 다른 크레이트가 요청하면 켜짐(`cargo tree -f "{p} {f}"`) | 기능을 더하기만 하게 설계 | 2.1, 2.3 |
| 트레이트의 필수 메서드를 기능으로 가림 | 켜지면 E0046, 미리 구현하면 꺼진 빌드에서 E0407 | 기능 전용 트레이트로 분리 | 2.4, `gated_api` |
| 열거형 배리언트를 기능으로 가림 | 켜지면 쓰는 쪽 `match`가 E0004 | 배리언트를 늘 두거나 `#[non_exhaustive]` | 2.4 |
| 기능 이름 오타(`fast_hash`) | 에러 없이 늘 거짓, `` unexpected `cfg` condition value `` 경고 | 매니페스트와 같은 이름 | 2.5, 03 |
| 알리지 않은 사용자 정의 cfg | `` unexpected `cfg` condition name `` 경고 | `[lints.rust]`의 `check-cfg` | 2.5, 03 |
| `dep:`으로 가린 의존성 이름을 `--features`에 줌 | `` does not have feature `weekday` `` | 그 의존성을 켜는 기능 이름을 씀 | 2.5 |
| 예제 파일 전체를 `#![cfg(feature = …)]`로 가림 | 기능이 꺼진 빌드에서 E0601 | `[[example]]`의 `required-features` | 2.6, 04 |

## 5. 연습 문제

### 연습 1. 우선순위를 정하는 해법 시험하기

[`projects/exclusive_features/broken`](projects/exclusive_features/broken)의 `datefmt`에서 `compile_error!`를 지우고, 두 기능이 함께 켜지면 `korean`이 이기도록 `iso` 판의 조건을 고치세요. `cargo run -p app`과 `cargo test -p report -p invoice`의 결과를 보고, Cargo Book이 이 해법을 권하면서도 조심스러운 이유를 설명하세요.

<details>
<summary>힌트</summary>

`iso` 판을 `#[cfg(all(feature = "iso", not(feature = "korean")))]`로 바꾸면 빌드는 됩니다. 그러나 `app`은 `[주간 보고] 2026년 11월 19일`을 출력합니다. `report`는 ISO 모양을 기대했는데 아무 에러 없이 한국어 모양을 받았습니다. `cargo test -p report`는 혼자서 통과하지만 `cargo test -p report -p invoice`에서는 `report`의 테스트가 `left: "[주간 보고] 2026년 11월 19일"`, `right: "[주간 보고] 2026-11-19"`로 실패합니다. 컴파일 에러가 조용한 동작 변경으로 바뀐 것이라, 02의 `v1`과 같은 문제입니다.

</details>

### 연습 2. 공개 필드를 남긴 채 고치기

[`projects/gated_api/broken`](projects/gated_api/broken)을 `fixed/`와 다른 방법으로 고치세요. `Message`의 필드는 모두 공개로 두되 `attachments`를 기능과 상관없이 늘 있게 하고, `Channel::max_attachments`는 기능으로 가린 채 기본 구현을 주세요. `alerts`는 한 번만 고쳐서 `cargo test -p alerts`와 `cargo test -p alerts --features notice/attachments`가 모두 통과하고 `cargo run -p app`도 동작해야 합니다. 이 설계에 남는 약점은 무엇인가요?

<details>
<summary>힌트</summary>

필드를 `pub attachments: Option<Vec<String>>`로 바꾸고, `alerts`의 리터럴에 `attachments: None`을, `app`에는 `Some(vec![…])`와 `as_ref().map_or(0, Vec::len)`을 씁니다. 메서드는 `#[cfg(feature = "attachments")] fn max_attachments(&self) -> usize { 0 }`처럼 기본 구현을 줍니다. `--features notice/attachments`는 명령줄에서 의존성의 기능을 켜는 문법입니다. 약점은 둘입니다. 기능이 꺼진 빌드에서 `attachments`에 값을 넣어도 보낼 방법이 없으니 무시하거나 에러로 돌려줘야 하고, `Sms`가 `max_attachments`를 재정의하려면 기능을 직접 켜야 합니다(켜지 않고 재정의하면 E0407).

</details>

### 연습 3. 기능 하나 더하기와 조합 세기

[`projects/exclusive_features/additive`](projects/exclusive_features/additive)의 `datefmt`에 영어 서식 `english`(`"Nov 19, 2026 (Thu)"`)를 더하세요. 요일은 `weekday`에 영어 이름 함수를 더해 구합니다. `Cargo.toml`을 어떻게 적어야 하나요? 기능 조합은 몇 가지이고, 모두 검사하려면 어떤 명령을 실행하나요?

<details>
<summary>힌트</summary>

`english = ["dep:weekday"]`처럼 두 기능이 같은 선택적 의존성을 가리켜도 됩니다. 어느 쪽이 켜지든 `weekday`가 빌드됩니다. 새 함수와 테스트는 `#[cfg(feature = "english")]`로 가립니다. 기능이 `korean`, `english` 둘이므로 조합은 4가지입니다. `cargo test -p datefmt`에 `--no-default-features`, `--no-default-features --features korean`, `--no-default-features --features english`, `--all-features`를 차례로 주면 테스트가 1, 2, 2, 3개 통과합니다. 기본 기능(`korean`)은 넷 중 둘째와 같은 빌드입니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 26: Be wary of `feature` creep](https://effective-rust.com/features.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- Cargo Book, [Features](https://doc.rust-lang.org/cargo/reference/features.html): [Optional dependencies](https://doc.rust-lang.org/cargo/reference/features.html#optional-dependencies)(`dep:`), [Feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification), [Mutually exclusive features](https://doc.rust-lang.org/cargo/reference/features.html#mutually-exclusive-features), [Inspecting resolved features](https://doc.rust-lang.org/cargo/reference/features.html#inspecting-resolved-features), [Feature resolver version 2](https://doc.rust-lang.org/cargo/reference/features.html#feature-resolver-version-2), [Required features](https://doc.rust-lang.org/cargo/reference/features.html#required-features), [Feature combinations](https://doc.rust-lang.org/cargo/reference/features.html#feature-combinations)
- The Reference, [Conditional compilation](https://doc.rust-lang.org/reference/conditional-compilation.html): cfg 조건, 컴파일러가 정하는 값, `cfg`·`cfg_attr` 속성, `cfg!`·`cfg_select!` 매크로
- rustc Book, [Checking conditional configurations](https://doc.rust-lang.org/rustc/check-cfg.html)와 [Cargo Specifics](https://doc.rust-lang.org/rustc/check-cfg/cargo-specifics.html): `--check-cfg`, `unexpected_cfgs`, `[lints.rust]`의 `check-cfg`
- 로컬 문서의 릴리스 노트(`rustc --print sysroot` 아래 `share/doc/rust/html/releases.md`): 1.60.0(Cargo, namespaced and weak dependency features), 1.79.0(`--check-cfg`), 1.80.0(Cargo의 check-cfg), 1.88.0(`cfg_boolean_literals`), 1.95.0(`cfg_select!`)
- [`cargo-hack`](https://github.com/taiki-e/cargo-hack) README: `--each-feature`, `--feature-powerset`
