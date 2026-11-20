# ch5. 기능: `[features]`, 선택적 의존성, 기능 통합, `cfg(feature)`

크레이트 하나가 여러 일을 할 수 있어도, 쓰는 쪽은 그 가운데 일부만 필요한 경우가 많습니다. 기능(feature)은 크레이트가 "이 부분은 원하는 사람만 켜서 쓰라"고 나눠 두는 Cargo의 장치입니다. [ch2](../ch2-dependencies/README.md)에서 의존성 한 줄에 `features`와 `default-features`를 적을 수 있다고만 하고 넘어갔고, [ch3](../ch3-manifest-targets/README.md)에서는 `required-features` 키를 표에서만 보았습니다. 이 챕터는 기능을 정의하는 쪽과 켜는 쪽을 모두 다루고, 여러 패키지가 같은 크레이트의 기능을 서로 다르게 켤 때 무슨 일이 일어나는지(기능 통합)를 확인합니다.

- 패키지: `p3-ch5-features`
- 기준 문서: The Cargo Book [3.4 Features](https://doc.rust-lang.org/cargo/reference/features.html), [3.4.1 Features Examples](https://doc.rust-lang.org/cargo/reference/features-examples.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64)
- 실습 자료: `examples/` 2개, `compile_fail/` 1개, 독립 프로젝트 [`projects/formats`](projects/formats/Cargo.toml)(라이브러리 둘과 바이너리 둘의 워크스페이스). 외부 크레이트와 네트워크는 쓰지 않습니다.
- 이 챕터 패키지에도 기능(`pretty`, `stats`, `full`)이 있어 예제의 출력이 `--features` 옵션에 따라 달라집니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `[features]` 표에 기능을 정의하고(다른 기능을 켜는 기능, `default` 포함), 코드에서 `#[cfg(feature = "…")]`와 `cfg!(feature = "…")`로 확인하는 방법과 둘의 차이를 설명할 수 있다.
2. `--features`·`--no-default-features`·`--all-features`, 의존성 선언의 `features`·`default-features`, `required-features`로 어떤 기능을 켤지 고를 수 있다.
3. 선택적 의존성을 기능으로 켜고, `dep:`과 `?`가 무엇을 바꾸는지 설명할 수 있다.
4. 기능 통합 때문에 같은 크레이트가 빌드하는 방법에 따라 다른 기능으로 컴파일된다는 것을 `cargo tree -e features`로 확인하고, 기능을 "더하기만 하게" 설계해야 하는 이유를 설명할 수 있다.

## 2. 핵심 개념

### 2.1 기능이 푸는 문제

표를 마크다운, CSV, JSON으로 바꿔 주는 라이브러리를 생각해 봅니다. CSV만 필요한 프로그램도 JSON 코드와 JSON용 의존성까지 함께 컴파일해야 한다면 빌드가 느려지고 실행 파일이 커집니다. 그렇다고 형식마다 크레이트를 따로 나누면 관리할 것이 늘어납니다. 기능은 그 중간입니다. 크레이트는 하나로 두고, 코드와 의존성을 기능 단위로 묶어 쓰는 쪽이 고르게 합니다.

Cargo Book의 Features Examples가 드는 실제 쓰임은 다음과 같습니다.

| 쓰임 | 예 |
| --- | --- |
| 빌드 시간과 파일 크기 줄이기 | `syn`(파서의 일부만 켜기), `regex`(유니코드 표를 빼기) |
| 동작 넓히기 | `serde_json`의 `preserve_order`(키 순서를 지키는 맵, 선택적 의존성 `indexmap`을 켬) |
| `no_std` 지원 | `std` 기능을 기본으로 켜 두고, 끄면 표준 라이브러리 없이 빌드(`no_std`는 Part 8 `ch1-intro-no-std`와 [부록 A ch33](../../appd-a/ch33-no-std/README.md)에서 다룸) |
| 짝을 이루는 절차적 매크로 | `serde`의 `derive` 기능이 `serde_derive`를 켬 |
| nightly 전용·실험 API | 쓰는 사람이 직접 켜야 하는 기능으로 따로 둠 |

기능은 **컴파일할 때** 정해집니다. 꺼진 기능의 코드는 실행 파일에 아예 들어가지 않습니다. 실행할 때 고르는 옵션(명령줄 인자, 설정 파일)과 다른 점이고, 그래서 기능을 바꾸려면 다시 빌드해야 합니다.

### 2.2 기능을 정의하고 코드에서 쓰기

기능은 `Cargo.toml`의 `[features]` 표에 적습니다. 이 챕터 패키지의 표입니다.

```toml
[features]
default = ["pretty"]
pretty = []
stats = []
full = ["pretty", "stats"]
```

- `기능 이름 = [목록]`입니다. 목록은 그 기능을 켜면 **함께 켜지는 것**입니다. `pretty = []`처럼 빈 목록이면 자기 혼자만 켜집니다.
- `full = ["pretty", "stats"]`처럼 다른 기능을 묶은 기능을 만들 수 있습니다.
- `default`는 아무것도 지정하지 않았을 때 켜지는 기능입니다(2.3절).
- 이름에는 대부분의 글자와 숫자, `_`를 쓰고, 첫 글자 뒤에는 `-`, `+`, `.`도 쓸 수 있습니다. crates.io에 게시하려면 ASCII 영문·숫자와 `_`, `-`, `+`만 써야 합니다.

**Cargo는 켜진 기능을 `--cfg`로 넘깁니다.** 켜진 기능마다 `rustc`에 `--cfg 'feature="이름"'`을 붙입니다. [ch1](../ch1-getting-started/README.md) 2.1절에서 본 `cargo build -v`로 확인할 수 있습니다. `projects/formats`에서 `cargo build -v -p report-web`을 실행했을 때(이미 빌드했다면 `cargo clean` 뒤에) `textfmt`를 컴파일하는 줄의 일부입니다.

```text
--cfg 'feature="ascii"' --cfg 'feature="default"' --cfg 'feature="json"' --cfg 'feature="markdown"'
--check-cfg 'cfg(feature, values("ascii", "csv", "default", "full", "json", "markdown"))'
```

`--cfg`는 켜진 기능을, `--check-cfg`는 이 패키지에 **있는** 기능 이름 전체를 알려 줍니다. 두 번째 줄 덕분에 코드에서 기능 이름을 잘못 적으면 컴파일러가 경고합니다(4.2절).

**코드에서 확인하는 두 가지 방법.**

| 형태 | 하는 일 | 꺼진 쪽의 코드 |
| --- | --- | --- |
| `#[cfg(feature = "stats")]` | 붙은 항목(함수, 모듈, `impl`, 블록 등)을 조건이 참일 때만 남김 | 컴파일 결과에서 **사라짐** |
| `#[cfg(not(feature = "stats"))]` | 조건이 거짓일 때만 남김. 같은 이름의 대체판을 둘 때 씀 | 사라짐 |
| `cfg!(feature = "stats")` | 컴파일할 때 정해지는 `true`/`false` 값 | 사라지지 않음. `if`의 두 갈래가 모두 컴파일됨 |

`cfg`는 구성(configuration)의 줄임이고, 이렇게 조건에 따라 코드를 넣고 빼는 일을 조건부 컴파일(conditional compilation)이라고 합니다. `#[cfg(test)]`는 [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md)에서 이미 썼습니다. 테스트를 빌드할 때만 남기는 같은 장치이고, 조건만 `test`에서 `feature = "…"`로 바뀐 것입니다.

두 방법의 차이는 "꺼진 쪽이 컴파일되는가"입니다. `#[cfg]`로 지운 함수는 없는 것이 되므로 부르는 쪽도 같은 조건으로 가려야 합니다. `cfg!`는 값일 뿐이라 `if cfg!(feature = "stats") { average(…) }`처럼 써도 `average`가 없으면 에러가 납니다. `if`의 갈래도 컴파일은 되기 때문입니다. 예제 [`01_cfg_feature`](examples/01_cfg_feature.rs)는 함수를 `#[cfg]`로 가리고, 같은 이름의 `show` 함수 두 판을 `cfg`와 `cfg(not)`으로 나누고, `cfg!`로 상태를 출력합니다. 네 가지 조합으로 실행한 결과입니다.

| 실행 옵션 | 켜진 기능 | 출력 |
| --- | --- | --- |
| (없음) | `pretty`(기본) | 점수를 상자로 그림, "통계는 stats 기능을 켜야 계산합니다." |
| `--no-default-features` | 없음 | 점수를 그대로 한 줄로, 통계 없음 |
| `--features stats` | `pretty`, `stats` | 상자 + `평균 80.0, 최고 95` |
| `--all-features` | `pretty`, `stats`(`full`도) | 상자 + `평균 80.0, 최고 95` |

`--features stats`에서 `pretty`도 켜진 것은 `--features`가 기본 기능을 끄지 않고 **더하기** 때문입니다.

### 2.3 기능을 고르는 법

**`default`와 명령줄 옵션.** 기능은 기본으로 모두 꺼져 있고, `default`에 적은 것만 켜집니다. 직접 빌드하는 패키지의 기능은 명령줄에서 고릅니다.

| 옵션 | 하는 일 |
| --- | --- |
| `--features a,b` 또는 `--features "a b"` | 적은 기능을 켭니다. 공백으로 나눌 때는 셸이 나누지 않게 따옴표로 감쌉니다. |
| `--no-default-features` | `default`를 켜지 않습니다. `--features`와 함께 쓰면 "기본 없이 이것만"이 됩니다. |
| `--all-features` | 고른 패키지의 기능을 모두 켭니다. |

`projects/formats`의 라이브러리 `textfmt`(2.4절에서 자세히 봅니다)를 `cargo tree -p textfmt -f "{p} {f}" -e normal`로 확인한 결과입니다. `-f "{p} {f}"`는 패키지와 켜진 기능을 한 줄에 보여 달라는 서식이고, `-e normal`은 일반 의존성만 보이게 하는 옵션입니다(빌드·dev 의존성 제외. 이 프로젝트에는 둘 다 없어 빼도 결과가 같습니다).

| 옵션 | `textfmt`에 켜진 기능 |
| --- | --- |
| (없음) | `default,markdown` |
| `--no-default-features` | (없음) |
| `--features csv` | `csv,default,markdown` |
| `--no-default-features --features ascii` | `ascii` |
| `--features full` | `csv,default,full,json,markdown` |
| `--all-features` | `ascii,csv,default,full,json,markdown` |

**워크스페이스에서.** `--features`는 `-p`나 `--workspace`로 고른 패키지에 적용됩니다.

- `cargo build -p textfmt --features csv`처럼 패키지를 고르고 기능을 적는 것이 가장 분명합니다.
- 여러 패키지를 고르고 기능의 주인을 밝히려면 `패키지/기능`으로 적습니다. `cargo build -p report-cli -p textfmt --features textfmt/json`.
- 가상 워크스페이스의 루트에서 `-p` 없이 `--features csv`를 주면, 고른 패키지(전부) 가운데 그 기능이 있는 패키지에 적용됩니다. 아무 패키지에도 없는 이름이면 ``none of the selected packages contains this feature: yaml`` 에러입니다.
- 멤버 디렉터리(예: `crates/report-cli`)에서 그 멤버에 없는 기능을 주면 에러가 납니다(4.2절). 고른 패키지가 그 멤버 하나뿐이기 때문입니다.

**의존성의 기능 고르기.** 의존성으로 쓰는 크레이트의 기능은 의존성 선언에서 고릅니다. ch2 2.4절에서 미뤄 둔 두 키입니다.

```toml
# crates/report-cli/Cargo.toml
[dependencies]
textfmt = { workspace = true, default-features = false, features = ["csv"] }
```

- `features = ["csv"]`: `textfmt`의 `csv` 기능을 켭니다.
- `default-features = false`: `textfmt`의 `default`를 켜지 않습니다. 이 줄이 없으면 `markdown`도 켜집니다.

`report-cli`는 [ch4](../ch4-workspaces/README.md)의 방식대로 의존성을 `[workspace.dependencies]`에서 물려받으면서 기능을 덧붙였습니다. 2024 에디션 패키지에서는 멤버의 `default-features = false`가 워크스페이스 쪽 설정을 덮어씁니다(cargo 1.99부터. 에디션과 버전에 따른 차이는 [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.7절의 표).

**기본 기능은 신중하게.** `default-features = false`는 "나는 기본 기능이 필요 없다"는 뜻이지 "기본 기능을 꺼 달라"는 보장이 아닙니다. 같은 크레이트를 기본 기능과 함께 쓰는 패키지가 그래프 어딘가에 하나라도 있으면 켜집니다(2.5절). 그리고 기본 기능에서 무언가를 빼면 그 기능을 기대하던 사용자의 빌드가 깨질 수 있습니다. Cargo Book은 기본 기능을 정할 때 오래 유지할 수 있는 것만 넣으라고 권합니다. 어떤 변경이 호환되지 않는지는 Part 3 `ch10-publishing-semver`에서 다룹니다.

**`required-features`.** ch3 2.3절의 타깃 표 키입니다. 적은 기능이 켜지지 않으면 그 타깃을 빌드하지 않습니다. 이 챕터의 `Cargo.toml`에 다음이 있습니다.

```toml
[[example]]
name = "02_required_features"
required-features = ["stats"]
```

예제 [`02_required_features`](examples/02_required_features.rs)는 `#[cfg(feature = "stats")]`로 가린 `median` 함수를 **가리지 않은 채** 부릅니다. 이 파일은 `stats`가 켜진 빌드에서만 컴파일되므로 그래도 됩니다. 기능 없이 실행하면 Cargo가 빌드하기 전에 멈춥니다.

```text
error: target `02_required_features` in package `p3-ch5-features` requires the features: `stats`
Consider enabling them by passing, e.g., `--features="stats"`
```

`cargo build --examples`는 이 예제를 에러 없이 건너뜁니다. 그래서 기본 조합으로만 빌드하면 이 예제가 깨져도 모르고 지나갑니다(2.6절).

### 2.4 선택적 의존성

[`projects/formats`](projects/formats/Cargo.toml)의 라이브러리 `textfmt`는 형식마다 기능이 하나씩 있고, JSON 형식은 문자열을 JSON 리터럴로 바꿔 주는 다른 멤버 `jsonesc`에 맡깁니다. CSV만 쓰는 사람이 `jsonesc`까지 컴파일할 필요는 없으므로 `jsonesc`는 **선택적 의존성**(optional dependency)입니다.

```toml
# crates/textfmt/Cargo.toml
[dependencies]
jsonesc = { workspace = true, optional = true }

[features]
default = ["markdown"]
markdown = []
csv = []
json = ["dep:jsonesc"]
ascii = ["jsonesc?/unicode-escape"]
full = ["markdown", "csv", "json"]
```

`optional = true`인 의존성은 어떤 기능이 켜 줄 때만 빌드에 들어옵니다. `json`을 켜지 않은 빌드에는 `jsonesc`가 없습니다. 코드에서 `jsonesc`를 쓰는 곳도 `#[cfg(feature = "json")]`로 가려야 합니다.

**암묵적 기능과 `dep:`.** 선택적 의존성은 기본적으로 **같은 이름의 기능**을 하나 만듭니다. 이것을 암묵적 기능(implicit feature)이라고 합니다. `jsonesc = { …, optional = true }`만 있고 `[features]`에서 `dep:`을 쓰지 않으면 `jsonesc`라는 기능이 생겨 `--features jsonesc`로 켤 수 있습니다. 사본에서 `json = ["jsonesc"]`로 바꾸면 `jsonesc`가 기능 목록에 들어옵니다. 기능 하나만 켜서 확인하면 이렇게 나옵니다.

```console
$ cargo tree -p textfmt --no-default-features --features jsonesc -f "{p} {f}" -e normal
textfmt v0.1.0 (…/crates/textfmt) jsonesc
└── jsonesc v0.1.0 (…/crates/jsonesc)
```

`cargo build -v`의 `--check-cfg`에도 `values("ascii", "csv", "default", "full", "json", "jsonesc", "markdown")`처럼 `jsonesc`가 기능 이름으로 들어갑니다.

의존성 이름이 그대로 기능이 되면, 사용자가 그 기능을 켜도 의존성만 들어올 뿐 `textfmt`에는 아무 일도 일어나지 않습니다(`to_json`은 `json` 기능 뒤에 있으므로). 내부 사정이 공개 기능 이름으로 새어 나가는 셈입니다. `[features]` 어디에서든 `dep:jsonesc`라고 적으면 이 암묵적 기능이 사라지고, 의존성은 `dep:`으로 적은 기능(`json`)을 통해서만 켜집니다. 이때 `--features jsonesc`를 주면 Cargo가 이유와 대신 쓸 기능을 알려 줍니다(4.2절).

**의존성의 기능을 켜는 기능: `패키지/기능`과 `패키지?/기능`.** 기능 목록에 `"jsonesc/unicode-escape"`처럼 적으면 내 기능을 켤 때 의존성의 기능도 켜집니다. 그런데 의존성이 선택적이면 이 표기는 **의존성 자체도 켭니다.** `ascii`는 "JSON을 쓰는 사람이라면 한글을 `\uXXXX`로 바꿔 달라"는 기능인데, `?`가 없으면 JSON을 쓰지 않는 사람이 `ascii`만 켜도 `jsonesc`가 따라 들어옵니다. 사본에서 `?`를 빼고 확인한 결과입니다.

```console
$ cargo tree -p textfmt --no-default-features --features ascii -f "{p} {f}" -e normal
textfmt v0.1.0 (…/crates/textfmt) ascii
└── jsonesc v0.1.0 (…/crates/jsonesc) unicode-escape
```

`?`를 붙인 `"jsonesc?/unicode-escape"`는 "다른 무언가가 `jsonesc`를 이미 켰다면 그 기능도 켜라"는 뜻입니다. 저장소의 `textfmt`에서는 `ascii`만 켜면 `jsonesc`가 들어오지 않고(2.3절 표의 `ascii` 줄), `json`과 함께 켜야 들어옵니다.

```console
$ cargo tree -p textfmt --no-default-features --features ascii,json -f "{p} {f}" -e normal
textfmt v0.1.0 (…/crates/textfmt) ascii,json
└── jsonesc v0.1.0 (…/crates/jsonesc) unicode-escape
```

`dep:`과 `?`는 Rust 1.60에서 들어온 문법입니다.

| 표기 | 뜻 |
| --- | --- |
| `json = ["csv"]` | `json`을 켜면 내 기능 `csv`도 켬 |
| `json = ["dep:jsonesc"]` | `json`을 켜면 선택적 의존성 `jsonesc`를 빌드에 넣음. 암묵적 기능 `jsonesc`는 생기지 않음 |
| `x = ["jsonesc/unicode-escape"]` | `jsonesc`의 `unicode-escape`를 켬. `jsonesc`가 선택적이면 그것도 켬 |
| `x = ["jsonesc?/unicode-escape"]` | `jsonesc`가 이미 켜져 있을 때만 그 기능을 켬 |

선택적 의존성 말고 플랫폼에 따라 의존성을 넣고 빼는 방법(`[target.'cfg(…)'.dependencies]`)은 ch2 2.4절에서 보았습니다. 그쪽은 사용자가 고르는 것이 아니라 빌드하는 플랫폼이 정합니다.

### 2.5 기능 통합

**같은 크레이트는 한 빌드에서 한 번만, 켜진 기능을 모두 합쳐 컴파일합니다.** ch2 2.6절에서 호환되는 버전 요구 사항들이 버전 하나로 합쳐지는 것을 보았습니다. 기능도 마찬가지입니다. 여러 패키지가 `textfmt`의 기능을 서로 다르게 요청하면 Cargo는 요청된 기능의 **합집합**으로 `textfmt`를 한 번 컴파일합니다. 이것을 기능 통합(feature unification)이라고 합니다.

`projects/formats`의 두 바이너리는 `textfmt`를 이렇게 씁니다.

| 패키지 | 의존성 선언 | 요청한 기능 |
| --- | --- | --- |
| `report-cli` | `default-features = false, features = ["csv"]` | `csv` |
| `report-web` | `features = ["json", "ascii"]` | `markdown`(기본), `json`, `ascii` |

두 바이너리는 실행하면 `textfmt::enabled_formats()`로 `textfmt`에 실제로 켜진 기능을 출력합니다. 같은 `report-cli`를 두 가지 방법으로 빌드해 실행합니다.

```console
$ cargo run -q -p report-cli
textfmt에 켜진 기능: ["csv"]
도시,기온
서울,3
부산,9
$ cargo build -q --workspace
$ ./target/debug/report-cli
textfmt에 켜진 기능: ["markdown", "csv", "json", "ascii"]
도시,기온
서울,3
부산,9
```

소스는 같은데 결과가 다릅니다. `cargo run -p report-cli`는 `report-cli`와 그 의존성만 빌드하므로 `report-cli`가 요청한 `csv`만 켜집니다. `cargo build --workspace`는 두 바이너리를 **한 번에** 빌드하므로 `textfmt`도 한 번 빌드되고, 그때 두 바이너리의 요청이 합쳐집니다. `report-cli`는 `default-features = false`를 적었지만 `report-web`이 기본 기능을 켰으므로 `markdown`도 켜졌습니다. Cargo는 기능 조합이 다른 두 빌드의 결과를 `target/` 안에 따로 보관합니다. 그래서 두 명령을 번갈아 실행해도 다시 컴파일하지는 않고, `target/debug/report-cli` 자리에는 마지막에 실행한 명령의 결과가 놓입니다.

**누가 무엇을 켰는지 보기.** `cargo tree -e features`는 기능을 그래프의 노드로 보여 줍니다. `-i textfmt`로 거꾸로 보면 `textfmt`의 기능마다 그것을 켠 패키지가 나옵니다.

```console
$ cargo tree -e features -i textfmt --workspace
textfmt v0.1.0 (…/crates/textfmt)
├── textfmt feature "ascii"
│   └── report-web v0.1.0 (…/crates/report-web)
│       └── report-web feature "default" (command-line)
├── textfmt feature "csv"
│   └── report-cli v0.1.0 (…/crates/report-cli)
│       └── report-cli feature "default" (command-line)
├── textfmt feature "default" (command-line)
│   └── report-web v0.1.0 (…/crates/report-web) (*)
├── textfmt feature "json"
│   └── report-web v0.1.0 (…/crates/report-web) (*)
└── textfmt feature "markdown"
    └── textfmt feature "default" (command-line) (*)
```

`csv`는 `report-cli`가, 나머지는 `report-web`이 켰고, `markdown`은 `default`를 통해 켜졌습니다. `(command-line)`은 명령줄의 선택(여기서는 `--workspace`가 고른 패키지들의 기본 기능)에서 출발했다는 표시입니다. `-p report-cli`로 범위를 좁히면 `textfmt feature "csv"` 하나만 남습니다. 의존성마다 켜진 기능을 한 줄로 보려면 2.3절의 `cargo tree -f "{p} {f}"`를 씁니다. `--workspace`와 함께 쓰면 `textfmt` 줄이 `ascii,csv,default,json,markdown`으로 나옵니다.

`cargo tree -e features`의 출력에는 `jsonesc feature "default"`처럼 `[features]`에 적지 않은 `default`가 보이기도 합니다. 의존성을 기본 기능과 함께 쓰면(`default-features = false`가 없으면) Cargo는 그 의존성의 `default` 노드를 거쳐 잇습니다. 비어 있는 `default`이므로 켜지는 것은 없습니다.

테스트도 통합의 영향을 받습니다. `textfmt`의 단위 테스트 둘은 각각 `markdown`과 `csv` 기능 뒤에 있습니다. `cargo test -p textfmt`는 기본 기능만 켜므로 1개를 실행하고, `cargo test --workspace`는 합쳐진 기능으로 빌드하므로 2개를 실행합니다.

**왜 합치는가.** 같은 크레이트를 기능 조합마다 따로 컴파일하면, 한 프로그램 안에 `textfmt`가 둘 생기고 두 `textfmt::Table`은 다른 타입이 됩니다. 한쪽에서 만든 표를 다른 쪽 함수에 넘길 수 없습니다. 하나로 합쳐야 크레이트끼리 타입을 주고받을 수 있고, 컴파일도 한 번으로 끝납니다.

**그래서 기능은 더하기만 해야 합니다.** 통합이 일어나면 내가 요청하지 않은 기능도 켜질 수 있습니다. 어떤 기능을 켰을 때 다른 기능의 함수가 사라지거나 기존 코드가 컴파일되지 않게 되면, 아무 관계 없는 다른 패키지 하나가 그 기능을 켜는 순간 내 코드가 깨집니다. Cargo Book은 기능이 **가산적**(additive)이어야 한다고 말합니다. 기능을 켜도 기존 기능이 사라지면 안 되고, 어떤 조합으로 켜도 대체로 안전해야 하며, 기능을 켰다고 기존 사용이 깨지는(SemVer 호환이 아닌) 변경이 생기면 안 됩니다.

동작을 **바꾸는** 기능도 있습니다. 2.1절의 `serde_json` `preserve_order`가 그렇고, 이 챕터의 `ascii`도 그렇습니다. `ascii`가 켜지면 `to_json`의 출력이 한글 그대로에서 `\uXXXX`로 바뀌는데, 통합 때문에 같은 빌드에서 `to_json`을 부르는 **모든** 패키지의 출력이 바뀝니다. 사본에 `json`만 켜는 멤버를 하나 더하면, 그 멤버를 혼자 실행할 때는 `{"도시": "서울"}`이 나오고 `cargo build --workspace` 뒤에 실행하면 이스케이프된 문자열이 나옵니다. 이스케이프한 JSON도 읽는 쪽에서는 같은 값이라 기존 사용이 깨지지 않으므로 `ascii`는 받아들일 만한 기능입니다. 동작을 바꾸는 기능을 만들 때는 이렇게 "누가 켜든 다른 사용자가 깨지지 않는가"를 따져야 합니다.

가산적인 설계의 대표적인 예가 `no_std` 지원입니다.

- `no_std` 기능(켜면 표준 라이브러리를 뺌)을 두면 안 됩니다. 누군가 이 기능을 켜면 표준 라이브러리를 기대하던 다른 사용자의 빌드가 깨집니다.
- 대신 `std` 기능을 두고 기본으로 켭니다. 끄는 것은 각 사용자가 `default-features = false`로 하고, 켜는 쪽이 이기는 통합과 방향이 맞습니다.

함께 켜면 안 되는 기능(배타적인 기능, mutually exclusive features)이 꼭 필요하면 그 조합을 컴파일 에러로 막을 수 있습니다. 사본의 `textfmt`에 다음 두 줄을 넣고 `--features csv,json`으로 빌드했습니다.

```rust
#[cfg(all(feature = "csv", feature = "json"))]
compile_error!("csv 기능과 json 기능은 함께 켤 수 없습니다");
```

```text
error: csv 기능과 json 기능은 함께 켤 수 없습니다
```

`compile_error!`는 컴파일에 다다르면 적은 메시지로 에러를 내는 표준 매크로입니다. 하지만 이 방법은 문제를 알려 줄 뿐 풀지는 않습니다. 그래프의 서로 모르는 두 패키지가 각각 하나씩 켜면 둘 다 쓸 수 없게 됩니다. Cargo Book은 패키지를 나누거나, 둘 다 켜졌을 때 한쪽을 우선하거나, 실행할 때 고르게 하라고 권합니다. 배타적인 기능을 더하기만 하는 기능으로 고치는 과정은 [부록 A ch26](../../appd-a/ch26-feature-creep/README.md)에서 볼 수 있습니다.

**기능 리졸버 2가 합치지 않는 경우.** 2021 에디션부터 기본인 기능 리졸버(feature resolver) 2(2024 에디션의 리졸버 3도 기능에 대해서는 같습니다)는 다음 경우에 합치지 않습니다. 지금 빌드하지 않는 플랫폼 전용 의존성의 기능, 빌드 의존성(`[build-dependencies]`)과 절차적 매크로의 기능, 지금 빌드하지 않는 dev-dependencies의 기능입니다. 마지막 경우를 [Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.3절이 재현했습니다.

### 2.6 기능을 알리고 검사하기

**어떤 기능이 있는지 알리기.** 기능은 `Cargo.toml`에 이름만 있어 무엇을 하는지 알기 어렵습니다. Cargo Book은 크레이트 문서(`lib.rs` 맨 위의 `//!`)에 기능 목록을 적기를 권합니다. `textfmt`의 크레이트 문서에 기능마다 생기거나 달라지는 것을 표로 적어 두었습니다. 표에서 함수 이름에 문서 링크를 걸지 않은 것은, 링크한 함수가 꺼진 기능 뒤에 있으면 그 조합으로 문서를 만들 때 깨진 링크 경고가 나기 때문입니다. 사용자에게 켜라고 권하지 않을 기능(내부용 선택적 의존성 등)은 목록에서 뺍니다. docs.rs가 문서를 만들 때 켤 기능을 정하는 방법과 문서에 기능 표시를 붙이는 방법은 Part 4에서 다룹니다.

**조합을 모두 검사할 수는 없습니다.** 기능이 n개면 조합은 2ⁿ가지입니다. 그런데 `cargo test`, `cargo clippy`, `cargo doc`은 기본 기능으로만 실행합니다. 2.3절의 `02_required_features`처럼 기본 조합에서는 컴파일조차 되지 않는 코드가 생기므로, 적어도 다음 세 조합은 따로 확인합니다.

```sh
cargo clippy --all-targets                          # 기본 기능
cargo clippy --all-targets --no-default-features    # 아무것도 없이
cargo clippy --all-targets --all-features           # 전부
```

워크스페이스에서는 라이브러리 멤버를 `-p`로 골라 따로 검사합니다. 루트에서 `cargo clippy --workspace --no-default-features`를 실행해도, `report-web`처럼 의존성 선언으로 `textfmt`의 기본 기능을 요청하는 멤버가 있으면 통합 때문에 `textfmt`는 기본 기능이 켜진 채 검사됩니다. `textfmt`를 기본 기능 없이 검사하려면 `cargo clippy -p textfmt --all-targets --no-default-features`처럼 씁니다. 이렇게 검사해 보니 `textfmt`의 테스트 모듈에서 쓰이지 않는 함수 경고가 나서, 모듈을 `#[cfg(all(test, any(feature = "csv", feature = "markdown")))]`로 가렸습니다.

조합을 자동으로 돌려 주는 도구와 CI 구성은 Part 3 `ch9-testing-ci`에서 다룹니다.

**그 밖에.** 빌드 스크립트는 환경 변수 `CARGO_FEATURE_<이름>`(대문자, `-`는 `_`)으로 켜진 기능을 압니다(Part 3 `ch8-build-scripts`). 기능을 더하거나 빼는 것이 어떤 버전 변경에 해당하는지는 `ch10-publishing-semver`에서 다룹니다. crates.io는 새로 게시하는 크레이트의 기능을 300개까지로 제한합니다.

## 3. 예제 안내

저장소 루트에서 실행합니다.

| 예제 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_cfg_feature`](examples/01_cfg_feature.rs) | `#[cfg]`·`cfg(not)`·`cfg!`, 기능 조합에 따라 달라지는 출력 | 1, 2 |
| [`02_required_features`](examples/02_required_features.rs) | `required-features` | 2 |
| [`projects/formats`](projects/formats/Cargo.toml) | 의존성의 기능, 선택적 의존성, `dep:`·`?`, 기능 통합 | 2, 3, 4 |

### 01_cfg_feature — 기능 조합에 따라 달라지는 출력

```sh
cargo run -p p3-ch5-features --example 01_cfg_feature
cargo run -p p3-ch5-features --example 01_cfg_feature --no-default-features
cargo run -p p3-ch5-features --example 01_cfg_feature --features stats
```

기본 기능(`pretty`)으로 실행한 결과입니다.

```text
pretty 켜짐: true
stats  켜짐: false
+-------------------+
| 점수 72 95 88 61 84 |
+-------------------+
통계는 stats 기능을 켜야 계산합니다.
```

`--no-default-features`로 실행하면 `cfg(not(feature = "pretty"))` 쪽 `show`가 컴파일됩니다.

```text
pretty 켜짐: false
stats  켜짐: false
점수 72 95 88 61 84
통계는 stats 기능을 켜야 계산합니다.
```

`--features stats`(또는 `--all-features`, `--features full`)로 실행하면 `average`와 `best`가 컴파일되고 마지막 줄이 바뀝니다.

```text
pretty 켜짐: true
stats  켜짐: true
+-------------------+
| 점수 72 95 88 61 84 |
+-------------------+
평균 80.0, 최고 95
```

상자의 오른쪽 테두리가 어긋나 보이는 것은 `chars().count()`로 `char` 수를 셌는데, 한글은 한 글자가 `char` 하나이면서 터미널에서 두 칸을 차지하기 때문입니다. 화면 폭을 세는 기능은 표준 라이브러리에 없습니다([ch2](../ch2-dependencies/README.md)의 `frame`은 그래서 `unicode-width` 크레이트를 썼습니다).

### 02_required_features — 기능이 켜져야 빌드되는 예제

```sh
cargo run -p p3-ch5-features --example 02_required_features --features stats
```

```text
가운데 점수: 84
70점 이상: 4명 / 5명
```

`--features stats` 없이 실행하면 종료 코드 101로 2.3절의 에러가 나옵니다.

### projects/formats — 기능을 서로 다르게 켜는 워크스페이스

```text
formats/
├── Cargo.toml              가상 매니페스트, [workspace.dependencies]
└── crates/
    ├── jsonesc/            라이브러리: JSON 문자열 리터럴. 기능 unicode-escape
    ├── textfmt/            라이브러리: 표를 markdown·csv·json으로. 선택적 의존성 jsonesc
    ├── report-cli/         바이너리: textfmt의 csv만
    └── report-web/         바이너리: textfmt의 기본 + json + ascii
```

`report-cli`가 물려받은 의존성의 `default-features`를 `false`로 덮어쓰므로(2.3절) 이 프로젝트는 cargo 1.99 이상이 필요합니다. cargo 1.98.1에서는 매니페스트를 읽는 단계에서 ``error inheriting `textfmt` from workspace root manifest's `workspace.dependencies.textfmt` ``와 `` `default-features = false` cannot override workspace's `default-features` ``로 멈춥니다.

`formats` 디렉터리에서 실행합니다.

```sh
cd part3-cargo/ch5-features/projects/formats
```

| 명령 | 기대 결과 |
| --- | --- |
| `cargo run -q -p report-cli` | `textfmt에 켜진 기능: ["csv"]`와 CSV 세 줄 |
| `cargo run -q -p report-web` | `["markdown", "json", "ascii"]`, 한글이 `\uXXXX`로 바뀐 JSON, 마크다운 표 |
| `cargo build -q --workspace` 뒤 `./target/debug/report-cli` | `["markdown", "csv", "json", "ascii"]`와 CSV(2.5절) |
| `cargo tree -e features -i textfmt --workspace` | 2.5절의 출력 |
| `cargo tree -p textfmt -f "{p} {f}" -e normal` + 옵션 | 2.3절의 표 |
| `cargo test --workspace` | `textfmt` 단위 테스트 2개, `jsonesc` 문서 테스트 1개 통과 |
| `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` | 통과 |

`report-web`의 출력입니다.

```text
textfmt에 켜진 기능: ["markdown", "json", "ascii"]
[
  {"\ub3c4\uc2dc": "\uc11c\uc6b8", "\uae30\uc628": "3"},
  {"\ub3c4\uc2dc": "\ubd80\uc0b0", "\uae30\uc628": "9"}
]
| 도시 | 기온 |
| --- | --- |
| 서울 | 3 |
| 부산 | 9 |
```

`ascii`가 켜져 있고 `json`이 `jsonesc`를 들여왔으므로 `jsonesc?/unicode-escape`가 효과를 냅니다. `\ub3c4\uc2dc`는 "도시"입니다.

확인이 끝나면 `cargo clean`을 실행합니다. `Cargo.toml`을 고치는 실험(2.4절의 `dep:`·`?` 지우기, 2.5절의 `compile_error!`)과 연습 문제는 저장소 밖의 사본에서 합니다. 저장소 루트에서 복사합니다.

```sh
cp -R part3-cargo/ch5-features/projects/formats /tmp/formats
cd /tmp/formats
```

## 4. 자주 하는 실수

### 4.1 기능을 켜지 않고 가린 함수를 부름 — E0425, [`e0425_feature_not_enabled.rs`](compile_fail/e0425_feature_not_enabled.rs)

```sh
cd part3-cargo/ch5-features
rustc --edition 2024 --out-dir /tmp/p3-ch5 compile_fail/e0425_feature_not_enabled.rs
```

```text
error[E0425]: cannot find function `average` in this scope
  --> compile_fail/e0425_feature_not_enabled.rs:25:26
   |
25 |     println!("평균 {:.1}", average(&scores));
   |                            ^^^^^^^ not found in this scope
   |
note: found an item that was configured out
  --> compile_fail/e0425_feature_not_enabled.rs:19:4
   |
18 | #[cfg(feature = "stats")]
   |       ----------------- the item is gated behind the `stats` feature
19 | fn average(scores: &[u32]) -> f64 {
   |    ^^^^^^^
```

- **읽는 법**: 에러는 "`average`라는 함수가 없다"이지만, note가 "조건 때문에 빠진(configured out) 항목이 있다"며 그 항목이 `stats` 기능 뒤에 가려져 있다고 알려 줍니다.
- **원인**: `rustc`로 이 파일만 컴파일하면 `--cfg 'feature="stats"'`가 없으므로 기능이 꺼진 상태입니다. `rustc --edition 2024 --cfg 'feature="stats"' …`로 컴파일하면 Cargo가 기능을 켠 것과 같아 성공합니다.
- **해결**: 기능을 켭니다. 라이브러리의 기능이라면 의존성 선언의 `features`에 적습니다.

워크스페이스에서는 이 실수가 숨기도 합니다. 사본의 `report-cli`에서 `textfmt::to_markdown`을 부르게 하면, `cargo build -p report-cli`는 위와 같은 E0425(note는 `textfmt`의 `lib.rs`를 가리킴)로 실패하지만 `cargo build --workspace`는 **성공합니다**. `report-web`이 켠 `markdown`이 통합으로 `report-cli`에도 켜지기 때문입니다. 워크스페이스 전체로만 빌드해 보면 놓치므로, 패키지마다 쓰는 기능을 자기 의존성 선언에 모두 적고 `-p`로 따로 빌드해 확인합니다. 조건이 `if cfg!(…)`의 갈래인 경우의 같은 에러는 [부록 A ch26](../../appd-a/ch26-feature-creep/README.md) 4.1절에 있습니다.

### 4.2 그 밖의 에러와 경고

| 한 일 | 메시지 | 원인과 해결 |
| --- | --- | --- |
| `dep:`으로 감춘 의존성 이름을 기능으로 켬(`--features jsonesc`) | ``package `textfmt …` does not have feature `jsonesc` `` / ``help: an optional dependency with that name exists, but the `features` table includes it with the "dep:" syntax so it does not have an implicit feature with that name`` / ``Dependency `jsonesc` would be enabled by these features: `json` `` | 2.4절. 메시지가 알려 주는 기능(`json`)을 켭니다. |
| 없는 기능(`-p textfmt --features yaml`) | `the package 'textfmt' does not contain this feature: yaml` / `help: there is a similarly named feature: full` | 이름을 확인합니다. |
| 멤버 디렉터리에서 의존성의 기능을 줌(`crates/report-cli`에서 `--features csv`) | `the package 'report-cli' does not contain this feature: csv` / `help: package with the missing feature: textfmt` | `-p textfmt`를 고르거나 `textfmt/csv`로 적습니다. 늘 켜야 한다면 의존성 선언의 `features`에 적습니다. |
| 의존성 선언에 그 크레이트에 없는 기능을 적음(사본의 `textfmt`에서 `ascii`를 지운 채 빌드) | ``failed to select a version for `textfmt`.`` / ``package `report-web` depends on `textfmt` with feature `ascii` but `textfmt` does not have that feature.`` / `help: available features: csv, default, full, json, markdown` | 의존성의 기능 목록을 확인합니다. 해석 단계의 에러라 아무것도 빌드되지 않습니다. |
| 코드에서 기능 이름을 잘못 적음(`cfg!(feature = "acsii")`) | ``warning: unexpected `cfg` condition value: `acsii` `` / ``help: there is a expected value with a similar name: `"ascii"` `` / ``note: expected values for `feature` are: `ascii`, `csv`, `default`, `full`, `json`, and `markdown` `` | 2.2절의 `--check-cfg` 덕분에 잡힙니다. 경고뿐이고 조건은 늘 거짓이 되므로 꼭 고칩니다. |
| `required-features`가 있는 타깃을 기능 없이 실행 | ``target `02_required_features` in package `p3-ch5-features` requires the features: `stats` `` | 2.3절. `--features stats`를 줍니다. |
| 선택적 의존성의 암묵적 기능만 켬(`dep:` 없이 `--features jsonesc`) | ``warning: unused dependency `jsonesc` `` | 의존성은 들어왔지만 그것을 쓰는 코드(`json` 뒤)는 꺼져 있습니다. 공개할 기능만 남도록 `dep:`을 씁니다. |

## 5. 연습 문제

연습마다 새 사본에서 시작합니다. 앞 연습의 변경이 뒤 연습의 답을 바꾸지 않게 하려는 것입니다. 저장소 루트에서 실행합니다.

```sh
rm -rf /tmp/formats && cp -R part3-cargo/ch5-features/projects/formats /tmp/formats
cd /tmp/formats
```

### 연습 1. 형식 하나 더하기

`textfmt`에 탭으로 칸을 나누는 `tsv` 기능과 `pub fn to_tsv(table: &Table) -> String`을 더하고, `enabled_formats`에도 넣으세요. `report-cli`가 CSV 대신 TSV를 출력하게 바꾸고 `cargo run -q -p report-cli`로 `textfmt에 켜진 기능: ["tsv"]`가 나오는지 확인하세요. `full`에도 넣을지 정하고 이유를 적어 보세요.

<details>
<summary>힌트</summary>

`[features]`에 `tsv = []`, `lib.rs`에 `#[cfg(feature = "tsv")]`를 붙인 함수, `enabled_formats`에 `if cfg!(feature = "tsv")` 한 줄을 더합니다. `report-cli/Cargo.toml`의 `features = ["csv"]`를 `["tsv"]`로 바꿉니다. 기능 이름을 코드에서 잘못 적으면 4.2절의 `unexpected cfg` 경고가 알려 줍니다. `full`에 넣는 것은 "모든 형식"이라는 약속을 지키는 일이고, 넣어도 다른 사용자의 빌드를 깨지 않습니다(더하기만 하는 변경).

</details>

### 연습 2. 켜질 기능 맞히기

실행하기 전에 다음 명령이 `textfmt` 줄에 출력할 기능 목록을 적어 보세요. 그다음 실행해 맞춰 보세요.

1. `cargo tree -p report-cli -f "{p} {f}"`
2. `cargo tree -p report-web -f "{p} {f}"`
3. `cargo tree -p report-cli -p report-web -f "{p} {f}"`
4. `cargo tree -p textfmt --no-default-features --features full -f "{p} {f}"`

<details>
<summary>힌트</summary>

고른 패키지가 요청한 기능만 합쳐집니다. 1은 `csv`, 2는 `ascii,default,json,markdown`, 3은 둘을 합친 `ascii,csv,default,json,markdown`입니다. 4는 `--no-default-features`로 `default`를 껐지만 `full`이 `markdown`을 켜므로 `csv,full,json,markdown`입니다. 기능 목록은 이름순으로 정렬돼 나옵니다.

</details>

### 연습 3. 통합이 숨긴 실수 찾기

`report-cli/src/main.rs`의 마지막에 `println!("{}", textfmt::to_markdown(&table));`을 더하고, `cargo build --workspace`와 `cargo build -p report-cli`를 차례로 실행하세요. 두 결과가 다른 이유를 설명하고, `report-cli`가 혼자서도 빌드되도록 고치세요.

<details>
<summary>힌트</summary>

4.1절의 상황입니다. `--workspace`에서는 `report-web`이 켠 `markdown`이 통합으로 켜져 성공하고, `-p report-cli`에서는 꺼져 E0425가 납니다. `report-cli`의 의존성 선언을 `features = ["csv", "markdown"]`으로 고칩니다. 쓰는 기능은 남이 켜 주기를 기대하지 말고 자기 선언에 적습니다.

</details>

### 연습 4. `?`를 빼면

`textfmt/Cargo.toml`의 `ascii = ["jsonesc?/unicode-escape"]`에서 `?`를 지우고 `cargo tree -p textfmt --no-default-features --features ascii -f "{p} {f}" -e normal`을 실행하세요. 지우기 전과 무엇이 달라졌는지, 그것이 왜 `ascii` 기능의 의도와 맞지 않는지 설명하세요.

<details>
<summary>힌트</summary>

`?`가 없으면 `jsonesc`가 `unicode-escape`와 함께 트리에 나타납니다(2.4절). `ascii`는 "JSON을 쓸 때 한글을 이스케이프하라"는 기능인데, JSON을 쓰지 않는 사람이 켜도 쓰이지 않는 `jsonesc`를 컴파일하게 됩니다. `?`는 의존성을 켜는 일을 다른 기능(`json`)에 맡깁니다.

</details>

## 6. 참고 자료

- The Cargo Book
  - [3.4 Features](https://doc.rust-lang.org/cargo/reference/features.html) — `[features]`, `default`, 선택적 의존성과 `dep:`·`?`, 명령줄 옵션, 기능 통합, 리졸버 2, 기능 문서와 조합
  - [3.4.1 Features Examples](https://doc.rust-lang.org/cargo/reference/features-examples.html) — 실제 크레이트의 기능 쓰임
  - [3.3.3 Dependency Resolution의 Features 절](https://doc.rust-lang.org/cargo/reference/resolver.html#features) — 기능 통합과 리졸버 2의 예외
  - [cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html) — `-e features`, `-i`, `-f`
- 이 저장소의 관련 챕터
  - [ch2](../ch2-dependencies/README.md) — 의존성 선언, 버전이 하나로 합쳐지는 규칙
  - [ch3](../ch3-manifest-targets/README.md) — 타깃 표와 `required-features`
  - [ch4](../ch4-workspaces/README.md) — `[workspace.dependencies]` 상속
  - [Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.3절 — 기능 리졸버 2와 dev-dependencies
  - [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.7절 — 상속한 의존성의 `default-features`
  - [부록 A ch26](../../appd-a/ch26-feature-creep/README.md) — 기능을 설계하는 지침, 배타적인 기능 고치기
