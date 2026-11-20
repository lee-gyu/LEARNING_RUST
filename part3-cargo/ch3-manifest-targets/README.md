# ch3. 매니페스트와 타깃: `Cargo.toml`의 표, 타깃 설정, `rust-version`, `[lints]`

[ch1](../ch1-getting-started/README.md)에서는 `Cargo.toml`에 거의 아무것도 적지 않고 파일의 위치만으로 타깃을 만들었고, [ch2](../ch2-dependencies/README.md)에서는 의존성 표를 채웠습니다. 이 챕터는 매니페스트(manifest)의 나머지를 봅니다. 어떤 표가 있고 각각 무엇을 정하는지, 관례와 다르게 타깃을 구성하려면 무엇을 적는지, 지원하는 Rust 버전과 린트 수준을 패키지 단위로 어떻게 선언하는지입니다.

- 패키지: `p3-ch3-manifest-targets`
- 기준 문서: The Cargo Book [3.1 The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html), [3.1.1 Cargo Targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html), [3.1.2 Rust Version](https://doc.rust-lang.org/cargo/reference/rust-version.html), [3.16 Lints](https://doc.rust-lang.org/cargo/reference/lints.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, clippy 1.100.0, macOS (aarch64)
- 실습 자료: `examples/` 2개, `compile_fail/` 2개, 독립 프로젝트 [`projects/unit-tools`](projects/unit-tools/Cargo.toml)(외부 크레이트 없음)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 매니페스트의 표를 "패키지 정보, 타깃, 의존성, 린트, 그 밖"으로 나눠 읽고, 처음 보는 키를 어느 문서에서 찾을지 알 수 있다.
2. `[lib]`·`[[bin]]`·`[[example]]`·`[[test]]` 표로 타깃의 이름과 경로를 관례와 다르게 정하고, `default-run`·`test`·`doctest`·`harness` 같은 설정으로 타깃의 동작을 바꿀 수 있다.
3. `rust-version`으로 패키지가 지원하는 가장 낮은 Rust 버전을 선언하고, 그 선언이 Cargo와 Clippy에서 어떤 효과를 내는지 확인할 수 있다.
4. `[lints]` 표로 rustc·Clippy·Cargo의 린트 수준을 패키지 단위로 정하고, 그룹과 개별 린트의 `priority`를 설명할 수 있다.

## 2. 핵심 개념

### 2.1 매니페스트의 지도

매니페스트는 TOML 형식의 표(table) 모음입니다. `[이름]`이 표 하나를 시작하고, 다음 `[…]`가 나올 때까지의 `키 = 값` 줄이 그 표에 속합니다. `[[이름]]`은 같은 이름으로 여러 번 쓸 수 있는 표(표 배열, array of tables)입니다.

예제 [`01_manifest_map`](examples/01_manifest_map.rs)은 이 챕터의 실습 패키지 [`projects/unit-tools`](projects/unit-tools/Cargo.toml)의 매니페스트를 읽어 표마다 종류와 키를 출력합니다.

```text
[package]  — 패키지 정보
    name, version, edition, rust-version, description, default-run, publish
[package.metadata.unit-tools]  — 다른 도구용 메타데이터(Cargo는 읽지 않음)
    default-precision
[lib]  — 타깃: 라이브러리(패키지에 하나)
    name
[[bin]]  — 타깃: 바이너리
    name, path, test
[[example]]  — 타깃: 예제
    name, path
[[test]]  — 타깃: 통합 테스트
    name, harness
[dependencies]  — 의존성
    (비어 있음)
[lints.rust]  — 린트 수준
    unsafe_code, missing_docs
[lints.clippy]  — 린트 수준
    pedantic, must_use_candidate
[lints.cargo]  — 린트 수준
    non_kebab_case_bins
[workspace]  — 워크스페이스
    (비어 있음)
```

매니페스트에 쓸 수 있는 표 전체와 이 저장소에서 다루는 곳입니다.

| 묶음 | 표 | 다루는 곳 |
| --- | --- | --- |
| 패키지 정보 | `[package]` | 2.2절, 2.4절 |
| 타깃 | `[lib]`, `[[bin]]`, `[[example]]`, `[[test]]`, `[[bench]]` | 2.3절 |
| 의존성 | `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, `[target.….dependencies]`, `[patch]` | [ch2](../ch2-dependencies/README.md) |
| 린트 | `[lints]` | 2.5절 |
| 기능 | `[features]` | Part 3 `ch5-features` |
| 빌드 설정 | `[profile.*]` | Part 3 `ch6-profiles-build-performance` |
| 워크스페이스 | `[workspace]` | Part 3 `ch4-workspaces` |
| 그 밖 | `[badges]`, `[hints]`, `[replace]` | 아래 설명 |

`[badges]`는 레지스트리에 보여 줄 배지를 적는 표이고, `[hints]`는 컴파일 방법에 대한 힌트를 적는 표인데 Cargo Book에 따르면 안정화된 힌트가 아직 없습니다. `[replace]`는 `[patch]`로 대체된 옛 표입니다(ch2 2.7절).

### 2.2 `[package]`: 패키지 정보

`cargo new`가 적어 주는 것은 `name`, `version`, `edition` 셋이지만 `[package]`에는 키가 스무 개 넘게 있습니다. 쓰임에 따라 나누면 다음과 같습니다.

| 쓰임 | 키 | 메모 |
| --- | --- | --- |
| 꼭 필요한 것 | `name` | Cargo가 요구하는 키는 이것 하나뿐입니다. |
| 컴파일에 영향 | `version`, `edition`, `rust-version` | `version`을 빼면 `0.0.0`이 됩니다. `edition`을 빼면 2015입니다([Part 2 ch1](../../part2-editions/ch1-editions-overview/README.md)). `rust-version`은 2.4절 |
| 타깃 고르기 | `default-run`, `autolib`, `autobins`, `autoexamples`, `autotests`, `autobenches` | 2.3절 |
| 게시할 때 쓰는 정보 | `description`, `license`, `license-file`, `repository`, `homepage`, `documentation`, `readme`, `keywords`, `categories`, `include`, `exclude`, `publish` | Part 3 `ch10-publishing-semver`. `authors`는 더는 권장하지 않는(deprecated) 키입니다. |
| 빌드 스크립트 | `build`, `links` | Part 3 `ch8-build-scripts` |
| 워크스페이스 | `workspace`, `resolver` | Part 3 `ch4-workspaces` |
| 다른 도구 | `metadata` | 아래 설명 |

`unit-tools`의 `[package]`입니다.

```toml
[package]
name = "unit-tools"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "섭씨·화씨, 킬로미터·마일을 바꾸는 작은 도구 모음"
default-run = "unit-tools"
publish = false
```

`publish = false`는 `cargo publish`가 이 패키지를 레지스트리에 올리지 못하게 막습니다. 게시할 생각이 없는 패키지(사내 도구, 이 저장소의 챕터 패키지)에 적어 두면 실수를 막을 수 있습니다.

**모르는 키는 경고합니다.** Cargo는 자기가 모르는 키를 만나면 무시하고 경고를 냅니다. 오타를 잡아 주는 장치입니다. `unit-tools`의 사본에서 `description`을 `desciption`으로 바꾸면 빌드는 되지만 다음이 나옵니다.

```text
warning: Cargo.toml: unused manifest key: package.desciption
```

값이 조용히 무시되므로 이 경고를 지나치면 "분명히 적었는데 적용이 안 된다"는 상황이 됩니다.

**`[package.metadata]`는 예외입니다.** Cargo는 이 표 아래를 읽지 않고 경고도 하지 않습니다. Cargo가 아닌 도구가 패키지별 설정을 두는 자리입니다. `unit-tools`에는 설명을 위해 만든 `[package.metadata.unit-tools]`가 들어 있습니다. 실제 예로 docs.rs가 문서를 만들 때 쓸 설정을 `[package.metadata.docs.rs]`에서 읽습니다.

### 2.3 타깃 표: 관례를 벗어날 때 적는 것

ch1에서 본 대로 Cargo는 정해진 위치의 파일을 타깃으로 알아봅니다. 타깃 표는 그 결과에 **더하거나 고치는** 수단입니다. `unit-tools`의 구성입니다.

```text
unit-tools/
├── Cargo.toml
├── src/
│   ├── lib.rs            라이브러리. [lib]가 이름만 units로 바꿈
│   ├── main.rs           바이너리 unit-tools (관례)
│   └── bin/km2mi.rs      바이너리 km2mi (관례)
├── tools/c2f.rs          바이너리 c2f — [[bin]]로 선언
├── demos/table.rs        예제 table — [[example]]로 선언
├── examples/quick.rs     예제 quick (관례)
└── tests/
    ├── convert.rs        통합 테스트 convert (관례)
    └── report.rs         통합 테스트 report (관례). [[test]]가 harness만 끔
```

```toml
[lib]
name = "units"

[[bin]]
name = "c2f"
path = "tools/c2f.rs"
test = false

[[example]]
name = "table"
path = "demos/table.rs"

[[test]]
name = "report"
harness = false
```

표를 쓰는 방식이 셋 보입니다.

- **관례 밖의 파일을 타깃으로 더합니다.** `tools/c2f.rs`와 `demos/table.rs`는 Cargo가 보는 위치가 아니므로 `name`과 `path`를 적어 선언합니다. `path`는 `Cargo.toml`이 있는 디렉터리를 기준으로 합니다.
- **관례가 찾은 타깃의 설정만 고칩니다.** `[[test]] name = "report"`에는 `path`가 없습니다. 이름이 `tests/report.rs`에서 찾은 타깃과 같으므로 그 타깃의 설정을 바꾸는 것입니다. `[lib]`도 위치는 `src/lib.rs` 그대로 두고 이름만 바꿉니다.
- **나머지는 그대로 관례를 따릅니다.** `[[bin]]`을 하나 적었다고 `src/main.rs`와 `src/bin/km2mi.rs`가 사라지지 않습니다. 선언한 타깃과 관례로 찾은 타깃이 합쳐집니다.

`[lib]`는 대괄호가 하나이고 나머지는 둘입니다. 라이브러리는 패키지에 하나뿐이라 보통 표이고, 바이너리·예제·테스트·벤치마크는 여럿일 수 있어 표 배열입니다.

합쳐진 결과는 Cargo에게 물어볼 수 있습니다. 예제 [`02_targets_from_cargo`](examples/02_targets_from_cargo.rs)가 `cargo metadata`의 출력에서 타깃 부분을 뽑아 보여 줍니다.

```text
default-run: unit-tools
rust-version: 1.85

kind     name        path               test  doctest  doc
lib      units       src/lib.rs         O     O        O
bin      c2f         tools/c2f.rs       -     -        O
bin      km2mi       src/bin/km2mi.rs   O     -        O
bin      unit-tools  src/main.rs        O     -        O
example  quick       examples/quick.rs  -     -        -
example  table       demos/table.rs     -     -        -
test     convert     tests/convert.rs   O     -        -
test     report      tests/report.rs    O     -        -
```

**타깃 표의 키.** 다섯 표가 같은 키를 씁니다.

| 키 | 뜻 | 기본값 |
| --- | --- | --- |
| `name` | 타깃의 이름. 실행 파일 이름이자, 라이브러리에서는 크레이트 이름 | 라이브러리는 패키지 이름(`-`는 `_`로), `src/main.rs`는 패키지 이름 그대로, 관례로 찾은 나머지는 파일(또는 디렉터리) 이름. 표로 적을 때는 `[lib]`가 아니면 꼭 적어야 함 |
| `path` | 크레이트 루트가 되는 소스 파일 | 이름에서 관례대로 짐작한 경로 |
| `test` | `cargo test`가 이 타깃의 테스트를 실행하는가 | 라이브러리·바이너리·통합 테스트는 `true`, 예제는 `false` |
| `doctest` | 문서 테스트를 실행하는가(라이브러리에만 의미가 있음) | 라이브러리는 `true` |
| `doc` | `cargo doc`이 이 타깃의 문서를 만드는가 | 라이브러리·바이너리는 `true` |
| `harness` | 테스트 러너(libtest)를 붙이는가 | `true` |
| `bench` | `cargo bench`의 대상인가 | 라이브러리·바이너리·벤치마크는 `true` |
| `crate-type` | 만들 크레이트의 종류. 라이브러리와 예제에만 적을 수 있음 | 라이브러리는 `["lib"]`, 예제는 `["bin"]` |
| `proc-macro` | 절차적 매크로 라이브러리인가(`[lib]`에만) | `false` |
| `required-features` | 이 타깃을 빌드하는 데 필요한 기능(`[lib]`에는 효과 없음) | 없음 |

위 출력의 `test`, `doctest`, `doc` 열이 이 표의 기본값 그대로입니다. `c2f`만 `test`가 `-`인 것은 `test = false`를 적었기 때문입니다. `proc-macro`는 [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)의 `hello_macro`에서 썼고, `required-features`는 Part 3 `ch5-features`에서, `bench`와 벤치마크의 `harness`는 `ch9-testing-ci`에서, `crate-type`의 종류(`cdylib`, `staticlib` 등)는 Part 5 `ch1-rustc-basics`에서 다룹니다.

**`[lib] name`: 크레이트 이름을 패키지 이름과 다르게.** 패키지 이름은 `unit-tools`이지만 코드에서는 `units::c_to_f`로 씁니다. 만들어지는 파일도 `libunits.rlib`이고, `cargo test`의 문서 테스트 줄도 `Doc-tests units`입니다. 다만 **다른 패키지가 이 패키지를 의존성으로 적을 때는 패키지 이름을 씁니다**(`unit-tools = "0.1"`). 이름이 둘이면 사용자가 헷갈리므로 꼭 필요할 때만 바꿉니다.

**`default-run`: `cargo run`이 실행할 바이너리.** 바이너리가 셋이라 ch1 4.1절의 에러가 날 상황이지만 `[package]`에 `default-run = "unit-tools"`를 적어 두어 `cargo run`만으로 실행됩니다.

```console
$ cargo run -q
unit-tools 0.1.0
  c2f [섭씨]    섭씨를 화씨로   (cargo run --bin c2f -- 36.5)
  km2mi [km]    킬로미터를 마일로 (cargo run --bin km2mi -- 10)
물이 끓는 온도: 212°F
$ cargo run -q --bin c2f -- 36.5
36.5°C = 97.7°F
```

**`test = false`: 테스트가 없는 타깃 건너뛰기.** `cargo test`는 `test`가 `true`인 타깃마다 테스트용 실행 파일을 따로 만듭니다. 바이너리에 단위 테스트가 없으면 그 빌드는 `running 0 tests`만 출력하는 낭비입니다. `unit-tools`의 `cargo test`에서 `Running unittests`는 `src/lib.rs`, `src/bin/km2mi.rs`, `src/main.rs` 세 줄이고 `tools/c2f.rs`는 없습니다.

**`harness = false`: 테스트 러너 없이 실행.** 평소 Cargo는 테스트 타깃을 `rustc --test`로 컴파일합니다. 그러면 컴파일러가 `#[test]` 함수를 모아 실행하고 결과를 집계하는 `main`을 대신 만들어 넣습니다. 이것이 테스트 러너(libtest harness)입니다. `harness = false`는 `--test`를 빼라는 뜻이라, 파일에 직접 쓴 `main`이 그대로 실행되고 종료 코드가 0이 아니면 실패로 칩니다. [`tests/report.rs`](projects/unit-tools/tests/report.rs)가 그런 테스트입니다.

```text
     Running tests/convert.rs (target/debug/…)

running 2 tests
(테스트 이름 두 줄 생략. 순서는 실행마다 다를 수 있습니다)

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/report.rs (target/debug/…)
통과 -40°C -> -40°F
통과 0°C -> 32°F
통과 100°C -> 212°F
```

`convert`는 테스트 러너의 형식으로, `report`는 자기가 출력한 대로 나옵니다. 자기만의 보고 형식이나 실행 방식이 필요한 테스트 도구와, stable에서 쓸 수 있는 벤치마크 도구가 이 설정을 요구합니다.

**자동 탐색(auto-discovery) 끄기.** `[package]`의 `autobins = false`는 바이너리를 관례로 찾지 않게 합니다(`autolib`, `autoexamples`, `autotests`, `autobenches`도 같습니다). 그러면 표로 선언한 타깃만 남습니다. `unit-tools`의 사본에 `autobins = false`를 넣으면 `src/main.rs`도 찾지 않게 되어 `default-run`이 가리킬 바이너리가 없다는 에러가 납니다(4.3절). Cargo Book이 드는 쓰임은 라이브러리에 `bin`이라는 모듈을 두고 싶을 때입니다. `src/bin/mod.rs`가 실행 파일로 취급되지 않게 하려면 자동 탐색을 꺼야 합니다. 그런 특별한 사정이 없으면 건드리지 않습니다.

### 2.4 `rust-version`: 지원하는 가장 낮은 Rust 버전

```toml
rust-version = "1.85"
```

`rust-version`은 "이 패키지는 이 버전 이상의 Rust로 컴파일할 수 있다"는 선언입니다. 최소 지원 Rust 버전(MSRV, minimum supported Rust version)이라고 부릅니다. 값은 `1.85`나 `1.85.0` 같은 버전 숫자만 적을 수 있고 `>=` 같은 연산자는 쓸 수 없습니다. [Part 2 ch1](../../part2-editions/ch1-editions-overview/README.md) 2.5절에서 에디션과 컴파일러 버전이 별개라는 것을 이 키로 확인했습니다. 여기서는 이 선언이 무엇을 바꾸는지 봅니다.

**효과 1. 더 낮은 툴체인에서는 Cargo가 먼저 막습니다.** `unit-tools`의 사본에서 값을 지금 툴체인보다 높은 `"1.200"`으로 바꾸고 빌드한 결과입니다.

```text
error: rustc 1.100.0 is not supported by the following packages:
  unit-tools@0.1.0 requires rustc 1.200
```

(같은 `requires` 줄이 타깃 수만큼 되풀이됩니다. 종료 코드는 101입니다.) 선언이 없으면 사용자는 낯선 문법 에러나 "그런 함수가 없다"는 에러를 만나고 원인을 스스로 짐작해야 합니다. 선언이 있으면 Cargo가 컴파일하기 전에 이유를 말해 줍니다. `--ignore-rust-version`을 주면 이 검사를 건너뛰고 빌드를 시도합니다.

**효과 2. 의존성을 고를 때 참고합니다.** 2024 에디션의 리졸버 3은 내 `rust-version`에서 쓸 수 있는 의존성 버전을 먼저 고르고([Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md)), `cargo add`도 그런 버전을 요구 사항으로 적습니다.

**효과 3. Clippy가 지켜지는지 확인합니다.** 선언만 하고 더 새로운 표준 라이브러리 기능을 쓰면 선언이 거짓이 됩니다. Clippy의 `incompatible_msrv` 린트(기본으로 warn)가 그것을 잡습니다. 사본의 `src/main.rs`에 1.100.0에서 안정화된 `bool::toggle`을 쓰는 두 줄을 넣고 `cargo clippy`를 실행한 결과입니다. 이 챕터의 경고·에러 인용에서는 `= help: for further information visit …` 같은 줄을 줄였습니다.

```text
warning: current MSRV (Minimum Supported Rust Version) is `1.85.0` but this item is stable since `1.100.0`
 --> src/main.rs:8:8
  |
8 |     on.toggle();
  |        ^^^^^^^^
  |
  = note: `#[warn(clippy::incompatible_msrv)]` on by default
```

이 린트가 보는 것은 표준 라이브러리 API의 안정화 버전입니다. 새 문법까지 확인하려면 실제로 그 버전의 툴체인으로 빌드해 봐야 하고, 그것은 CI의 일입니다(Part 3 `ch9-testing-ci`).

**어떤 값을 적을까.** 낮게 적을수록 더 많은 사용자가 쓸 수 있지만, 그만큼 새 기능을 쓰지 못합니다. Cargo Book은 정책을 정해 두기를 권합니다. 가장 단순한 정책은 "항상 최신 stable"이고, 그다음으로 단순한 방법은 옛 Rust를 지원하는 내 패키지의 이전 메이저·마이너 버전을 계속 관리해 주는 것입니다. 정책의 예로는 "N-2"(최신 버전과 그 앞의 두 릴리스까지 지원)를 듭니다. `rust-version`을 올리는 것은 사용자의 빌드를 막을 수 있는 변경이므로 Cargo Book은 마이너 버전을 올릴 만한 변경으로 봅니다. 값을 적었다면 효과 3과 CI로 지켜지는지 확인해야 합니다. 라이브러리 작성자의 지침은 [부록 A ch21](../../appd-a/ch21-semver/README.md) 2.7절에 있습니다.

### 2.5 `[lints]`: 린트 수준을 패키지 단위로

린트(lint)는 컴파일은 되지만 고칠 만한 코드를 알려 주는 검사이고, 린트마다 린트 수준(lint level)이 있습니다. 지금까지는 `#[expect(...)]`나 `#[allow(...)]` 같은 속성으로 코드의 한 곳에서 수준을 바꿨습니다. `[lints]` 표는 패키지 전체의 수준을 매니페스트 한 곳에서 정합니다.

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
must_use_candidate = "allow"

[lints.cargo]
non_kebab_case_bins = "warn"
```

**표의 이름이 도구입니다.** `[lints.도구]` 아래에 `린트 이름 = "수준"`을 적습니다. 린트 이름에서 `::` 앞부분이 도구이고, `::`가 없으면 `rust`입니다.

| 표 | 린트를 내는 도구 | 이름의 예 |
| --- | --- | --- |
| `[lints.rust]` | 컴파일러 `rustc` | `unsafe_code`, `missing_docs` |
| `[lints.clippy]` | Clippy(`cargo clippy`로 실행할 때만) | `clippy::pedantic` → `pedantic` |
| `[lints.rustdoc]` | `rustdoc`(`cargo doc`으로 실행할 때) | `rustdoc::broken_intra_doc_links` → `broken_intra_doc_links` |
| `[lints.cargo]` | Cargo 자신 | `cargo::unused_dependencies` → `unused_dependencies` |

**수준은 넷입니다.**

| 수준 | 뜻 |
| --- | --- |
| `allow` | 알리지 않음 |
| `warn` | 경고 |
| `deny` | 에러. 코드의 `#[allow]`로 풀 수 있음 |
| `forbid` | 에러. 코드에서 풀 수 없음 |

**왜 매니페스트에 적는가.** 같은 일을 크레이트 루트의 `#![warn(...)]`으로도 할 수 있지만, 타깃마다 크레이트 루트가 따로 있습니다. `unit-tools`라면 `src/lib.rs`, `src/main.rs`, `src/bin/km2mi.rs`, `tools/c2f.rs`와 예제·테스트 파일까지 여덟 곳에 같은 줄을 적어야 합니다. `[lints]`는 한 번 적으면 패키지의 모든 타깃에 적용됩니다. 그리고 의존성에는 적용되지 않습니다. 남의 크레이트가 내 기준에 맞지 않는다고 내 빌드가 실패하면 안 되기 때문입니다.

**Cargo는 이 표를 명령줄 옵션으로 바꿔 넘깁니다.** 사본의 `src/lib.rs`에 unsafe 블록을 넣으면 다음 에러가 납니다.

```text
error: usage of an `unsafe` block
  --> src/lib.rs:50:5
   |
50 |     unsafe { *p }
   |     ^^^^^^^^^^^^^
   |
   = note: requested on the command line with `-F unsafe-code`
```

마지막 note가 출처를 알려 줍니다. `[lints.rust] unsafe_code = "forbid"`가 `rustc -F unsafe-code`로 넘어간 것입니다. 문서 주석을 하나 지웠을 때의 `missing_docs` 경고에도 `requested on the command line with -W missing-docs`가 붙습니다. 린트 수준이 여러 곳(명령줄, 속성, 그룹)에서 정해질 때 어느 것이 이기는지는 Part 5 `ch3-lints`에서 다룹니다.

**그룹과 `priority`.** `clippy::pedantic`은 린트 하나가 아니라 여러 린트를 묶은 린트 그룹(lint group)입니다. 꽤 엄격하고 가끔 틀린 지적도 하는 린트를 모은 그룹이라 기본으로는 꺼져 있습니다. 그룹을 켜고 그 가운데 몇 개만 끄는 것이 흔한 구성인데, 이때 순서가 문제입니다. 그룹이 나중에 적용되면 개별 설정을 덮어 버립니다. TOML의 표에는 순서가 없으므로 Cargo는 `priority` 숫자로 순서를 정합니다. **숫자가 낮은 것이 먼저 적용되고, 높은 것이 그 위에 덮어씁니다.** 적지 않으면 0입니다.

```toml
pedantic = { level = "warn", priority = -1 }   # 먼저: 그룹 전체를 warn으로
must_use_candidate = "allow"                   # 나중(priority 0): 이 린트만 allow로
```

사본에서 `must_use_candidate` 줄을 지우면 값을 돌려주는 `pub fn` 셋(`c_to_f`, `f_to_c`, `km_to_mi`)에 경고가 나오고, note가 그룹에서 왔다는 것을 알려 줍니다.

```text
warning: this function could have a `#[must_use]` attribute
  --> src/lib.rs:11:8
   |
11 | pub fn c_to_f(celsius: f64) -> f64 {
   |        ^^^^^^
   |
   = note: `-W clippy::must-use-candidate` implied by `-W clippy::pedantic`
```

`priority`를 빼고 `pedantic = "warn"`이라고만 적으면 그룹과 개별 린트의 우선순위가 같아집니다. Cargo Book은 우선순위가 같은 항목끼리의 순서를 정하지 않으므로 어느 쪽이 이길지 기대할 수 없습니다. cargo 1.100.0에서 확인해 보니 린트 이름에 따라 결과가 달랐습니다. `must_use_candidate = "allow"`는 지켜졌지만, 같은 자리에 적은 `unreadable_literal = "allow"`는 그룹에 덮여 `1000000` 같은 리터럴에 경고가 났습니다. Clippy가 이런 매니페스트를 에러로 잡습니다(4.2절).

**Cargo 자신의 린트.** `[lints.cargo]`의 린트는 코드가 아니라 매니페스트를 검사합니다. [ch1](../ch1-getting-started/README.md) 2.4절에서 본 `unused dependency` 경고가 `cargo::unused_dependencies`(기본으로 warn)입니다. Cargo Book 3.16절의 목록에서 기본으로 켜진 것과 꺼진 것 몇 개를 고르면 다음과 같습니다.

| 린트 | 기본 수준 | 잡는 것 |
| --- | --- | --- |
| `unused_dependencies` | warn | 적어 놓고 쓰지 않는 의존성 |
| `unknown_lints` | warn | `[lints.cargo]`에 적은 모르는 린트 이름 |
| `missing_lints_inheritance` | warn | 워크스페이스에 `[workspace.lints]`가 있는데 패키지가 물려받지 않음(`ch4-workspaces`) |
| `non_kebab_case_bins` | allow | `kebab-case`가 아닌 바이너리 이름 |
| `non_snake_case_packages` | allow | `snake_case`가 아닌 패키지 이름 |

`unit-tools`는 꺼져 있는 `non_kebab_case_bins`를 warn으로 켰습니다. 사본에서 `src/bin/km2mi.rs`의 이름을 `km_to_mi.rs`로 바꾸면 Cargo가 알려 줍니다.

```text
warning: binary `km_to_mi` should have a kebab-case name
```

`[lints]`를 프로젝트의 규칙으로 운영하는 방법(어떤 그룹을 켜고, 예외를 어떻게 남기는가)은 [부록 A ch29](../../appd-a/ch29-listen-to-clippy/README.md)가 다룹니다. 워크스페이스의 여러 패키지가 같은 `[lints]`를 쓰게 하는 방법은 다음 챕터 `ch4-workspaces`에 있습니다.

## 3. 예제 안내

저장소 루트에서 실행합니다.

| 예제 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_manifest_map`](examples/01_manifest_map.rs) | 매니페스트의 표를 종류별로 | 1 |
| [`02_targets_from_cargo`](examples/02_targets_from_cargo.rs) | Cargo가 인식한 타깃과 설정 | 2 |
| [`projects/unit-tools`](projects/unit-tools/Cargo.toml) | 타깃 표, `default-run`, `rust-version`, `[lints]` | 2, 3, 4 |

### 01_manifest_map — 표의 종류

```sh
cargo run -p p3-ch3-manifest-targets --example 01_manifest_map
```

`projects/unit-tools/Cargo.toml`을 `include_str!`로 읽어 `[`로 시작하는 줄을 표의 시작으로 보고, 그 아래의 `키 = 값` 줄에서 키를 모읍니다. 출력은 2.1절에 실었습니다. TOML을 제대로 해석하는 것이 아니라 줄의 모양만 보는 단순한 도구입니다.

### 02_targets_from_cargo — Cargo가 본 타깃

```sh
cargo run -p p3-ch3-manifest-targets --example 02_targets_from_cargo
```

`cargo metadata --no-deps --format-version 1`을 자식 프로세스로 실행하고 JSON 출력에서 타깃 목록을 문자열 검색으로 꺼냅니다. 출력은 2.3절에 실었습니다. `kind`, `name`, `test`, `doctest`, `doc`은 JSON의 필드 이름 그대로이고, `path`는 절대 경로인 `src_path`에서 프로젝트 디렉터리 부분을 떼어 낸 것입니다. `cargo metadata`는 Part 3 `ch11-commands-tools`에서 다룹니다.

### projects/unit-tools — 타깃을 직접 구성한 패키지

```sh
cd part3-cargo/ch3-manifest-targets/projects/unit-tools
```

| 명령 | 기대 결과 |
| --- | --- |
| `cargo run -q` | 2.3절의 네 줄(`default-run` 덕분에 `--bin` 없이 실행) |
| `cargo run -q --bin c2f -- 36.5` | `36.5°C = 97.7°F` |
| `cargo run -q --bin c2f` | `100°C = 212.0°F` (인자가 없으면 100) |
| `cargo run -q --bin km2mi -- 10` | `10 km = 6.2 mi` |
| `cargo run -q --bin c2f -- abc` | `Error: "숫자가 아닙니다: abc"`, 종료 코드 1 |
| `cargo run -q --example table` | 아래 출력 |
| `cargo run -q --example quick` | `10 km = 6.21 mi` |
| `cargo test` | 단위 테스트 1개, `convert` 2개, `report`의 세 줄, 문서 테스트 1개. `Running` 줄은 다섯 번 |
| `cargo clippy --all-targets -- -D warnings` | 경고 없음(`pedantic` 그룹을 켠 상태) |

```text
섭씨   화씨
  -40  -40.0
    0   32.0
 36.5   97.7
  100  212.0
```

2.2~2.5절의 실험(키 오타, `rust-version` 올리기, unsafe 블록 넣기, 린트 줄 지우기)은 `Cargo.toml`과 소스를 고치므로 저장소 밖의 사본에서 합니다.

```sh
cp -R part3-cargo/ch3-manifest-targets/projects/unit-tools /tmp/unit-tools
cd /tmp/unit-tools
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 챕터 디렉터리에서 확인합니다.

```sh
cd part3-cargo/ch3-manifest-targets
rustc --edition 2024 --out-dir /tmp/p3-ch3 compile_fail/e0601_no_main_without_harness.rs
rustc --edition 2024 --out-dir /tmp/p3-ch3 compile_fail/forbid_unsafe_code.rs
```

### 4.1 `harness = false`인데 `main`이 없음 — E0601, [`e0601_no_main_without_harness.rs`](compile_fail/e0601_no_main_without_harness.rs)

```text
error[E0601]: `main` function not found in crate `e0601_no_main_without_harness`
  --> compile_fail/e0601_no_main_without_harness.rs:22:2
   |
22 | }
   |  ^ consider adding a `main` function to `compile_fail/e0601_no_main_without_harness.rs`
```

- **원인**: `harness = false`는 `rustc --test`의 `--test`를 빼라는 뜻입니다. 테스트 러너가 만들어 주던 `main`이 없어지므로 `#[test]` 함수만 있는 파일은 실행 파일이 될 수 없습니다. `rustc`로 이 파일을 그냥 컴파일하는 것이 같은 상황입니다(`--test`를 붙이면 컴파일됩니다). `unit-tools`의 `tests/report.rs`를 `#[test]` 함수 하나로 바꿔도 같은 에러가 나고, `[[bin]]`의 `path`가 `main`이 없는 파일을 가리킬 때도 마찬가지입니다.
- **해결**: `main`을 직접 쓰거나 `harness = false`를 지웁니다. 반대 방향의 실수가 더 조용합니다. `main`으로 쓴 테스트에서 `harness = false`를 지우면 에러 없이 `running 0 tests`가 되어 검사가 하나도 실행되지 않습니다(연습 3).

### 4.2 그룹과 개별 린트의 우선순위가 같음

`[lints.clippy]`에서 `priority`를 빼고 적은 경우입니다.

```toml
[lints.clippy]
pedantic = "warn"
must_use_candidate = "allow"
```

```text
error: lint group `pedantic` has the same priority (0) as a lint
  --> Cargo.toml:53:1
   |
53 | pedantic = "warn"
   | ^^^^^^^^   ------ has an implicit priority of 0
54 | must_use_candidate = "allow"
   | ------------------ has the same priority as this lint
   |
   = note: the order of the lints in the table is ignored by Cargo
   = note: `#[deny(clippy::lint_groups_priority)]` on by default
help: to have lints override the group, set `pedantic` to a lower priority
   |
53 - pedantic = "warn"
53 + pedantic = { level = "warn", priority = -1 }
```

- **원인**: 위에서 아래로 읽히기를 기대하고 적었지만 Cargo는 표 안의 순서를 보지 않습니다(`the order of the lints in the table is ignored by Cargo`). 우선순위가 같으면 린트 이름에 따라 그룹이 개별 설정을 덮기도 합니다(2.5절의 `unreadable_literal`). 이 검사는 Clippy의 `lint_groups_priority` 린트(기본으로 deny)라서 `cargo clippy`에서 나옵니다. `cargo build`는 이 검사를 하지 않습니다.
- **해결**: help대로 그룹의 `priority`를 낮춥니다.

### 4.3 `forbid`한 린트를 코드에서 어김 — [`forbid_unsafe_code.rs`](compile_fail/forbid_unsafe_code.rs)

```text
error: usage of an `unsafe` block
  --> compile_fail/forbid_unsafe_code.rs:21:18
   |
21 |     let copied = unsafe { *pointer };
   |                  ^^^^^^^^^^^^^^^^^^^
   |
note: the lint level is defined here
  --> compile_fail/forbid_unsafe_code.rs:15:11
   |
15 | #![forbid(unsafe_code)]
   |           ^^^^^^^^^^^
```

- **원인**: 린트를 `deny`나 `forbid`로 올리면 경고가 에러가 됩니다. 에러 코드가 없는 것은 컴파일러의 타입 검사가 아니라 린트가 낸 에러이기 때문입니다. 이 파일은 `rustc`만으로 재현하려고 `[lints.rust] unsafe_code = "forbid"`를 크레이트 속성으로 옮겨 적었습니다. Cargo로 빌드할 때는 note가 2.5절처럼 `requested on the command line with -F unsafe-code`로 나옵니다.
- **해결**: 코드를 고칩니다. `deny`였다면 이유를 적고 그 자리만 `#[expect(unsafe_code)]`로 풀 수 있지만, `forbid`는 풀 수 없습니다. "이 패키지에는 unsafe가 없다"를 약속하고 싶을 때 `forbid`를 고릅니다.

### 4.4 그 밖의 매니페스트 에러

| 한 일 | 메시지 | 원인과 해결 |
| --- | --- | --- |
| `[[bin]]`에 `name`이 없음 | `binary target bin.name is required` | `[lib]`를 뺀 타깃 표는 `name`이 필수입니다. |
| `[[example]]`에 `path` 없이 관례 밖의 파일을 기대 | ``can't find `table` example at `examples/table.rs` or `examples/table/main.rs`. Please specify example.path if you want to use a non-default path.`` | `path`가 없으면 이름으로 관례의 위치를 찾습니다. `path = "demos/table.rs"`를 적습니다. |
| `[lib] name = "unit-tools"` | `library target names cannot contain hyphens: unit-tools` | 라이브러리 이름은 크레이트 이름이라 `-`를 쓸 수 없습니다. |
| `default-run = "nope"` | ``default-run target `nope` not found`` | 있는 바이너리 이름을 적습니다. 이 상태에서는 매니페스트를 읽는 단계에서 실패하므로 `cargo run --bin`의 목록도 볼 수 없습니다. `default-run` 줄을 지운 뒤 목록을 확인합니다. |
| `autobins = false`를 넣었는데 `default-run`이 `src/main.rs`의 바이너리 | ``default-run target `unit-tools` not found`` | 자동 탐색을 끄면 `src/main.rs`도 찾지 않습니다. `[[bin]]`으로 선언합니다. |
| 키 이름 오타(`desciption`) | `unused manifest key: package.desciption` | 경고뿐이라 지나치기 쉽습니다. 값은 무시됩니다(2.2절). |
| `rust-version`이 툴체인보다 높음 | `rustc 1.100.0 is not supported by the following packages` | 툴체인을 올리거나, 시험 삼아 `--ignore-rust-version`으로 빌드합니다(2.4절). |

## 5. 연습 문제

3절의 명령으로 만든 사본(`/tmp/unit-tools`)에서 풉니다.

### 연습 1. 관례 밖의 바이너리 추가

화씨를 섭씨로 바꾸는 바이너리 `f2c`를 `tools/f2c.rs`에 추가하세요. `cargo run --bin f2c -- 212`가 `212°F = 100.0°C`를 출력하고, `cargo clippy --all-targets -- -D warnings`가 여전히 통과해야 합니다.

<details>
<summary>힌트</summary>

`tools/c2f.rs`를 복사해 `units::f_to_c`를 부르도록 고치고, `Cargo.toml`에 `[[bin]]` 표를 하나 더 적습니다(`name = "f2c"`, `path = "tools/f2c.rs"`). `[[bin]]`은 표 배열이라 여러 번 쓸 수 있습니다. 파일 맨 위에 `//!` 문서 주석이 없으면 `[lints.rust] missing_docs = "warn"` 때문에 경고가 납니다.

</details>

### 연습 2. 린트 하나 더 켜기

`[lints.clippy]`에 `unwrap_used = "deny"`를 추가하세요. 그다음 `src/bin/km2mi.rs`의 `?`를 `.unwrap()`으로 바꾸고 `cargo clippy`를 실행해 에러를 확인하세요. 에러의 마지막 note가 무엇을 말하는지, 이 린트가 `pedantic` 그룹에 들어 있지 않은데도 켜진 이유를 설명하세요.

<details>
<summary>힌트</summary>

`unwrap_used` 에러의 note는 `requested on the command line with -D clippy::unwrap-used`입니다. Cargo가 `[lints.clippy]`의 줄을 Clippy의 명령줄 옵션으로 넘겼다는 뜻입니다. `unwrap_used`는 일부러 고른 프로젝트만 켜는 restriction 그룹의 린트라 기본으로 꺼져 있고, 이름을 직접 적어서 켰습니다. 개별 린트이므로 `priority`는 적지 않아도 됩니다. 이 에러 앞에 `unnecessary_wraps` 경고도 하나 나옵니다. `?`가 없어져 `main`이 늘 `Ok(())`만 돌려주게 된 것을 `pedantic` 그룹이 알려 주는 것입니다.

</details>

### 연습 3. `harness`를 다시 켜면

`Cargo.toml`에서 `[[test]]` 표 세 줄을 지우고 `cargo test --test report`를 실행하세요. 테스트는 통과로 나오지만 무엇이 달라졌는지 출력에서 찾고, `tests/report.rs`를 테스트 러너에 맞게 고쳐 세 경우가 다시 검사되게 하세요.

<details>
<summary>힌트</summary>

`running 0 tests`가 나옵니다. 테스트 러너가 붙으면 러너가 만든 `main`이 실행되고 파일에 쓴 `main`은 불리지 않습니다. `#[test]` 함수가 하나도 없으니 검사가 0개입니다. `main`을 `#[test] fn boiling_and_freezing()`으로 바꾸고 `println!`과 `exit` 대신 `assert!`를 쓰면 `running 1 test`가 됩니다.

</details>

### 연습 4. `default-run` 바꾸기

`default-run`을 `c2f`로 바꾸고 `cargo run -q -- 20`의 출력을 확인하세요. 그다음 `default-run` 줄을 지우고 `cargo run`을 실행해, ch1 4.1절에서 본 에러가 돌아오는지 확인하세요.

<details>
<summary>힌트</summary>

`20°C = 68.0°F`가 나옵니다. `--` 뒤의 인자는 Cargo가 아니라 실행되는 바이너리에 전달됩니다. `default-run`이 없으면 ``error: `cargo run` could not determine which binary to run.``과 함께 `available binaries: c2f, km2mi, unit-tools`가 나옵니다.

</details>

## 6. 참고 자료

- The Cargo Book
  - [3.1 The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html) — 표와 키의 전체 목록, `[package]`의 각 필드, `[lints]`
  - [3.1.1 Cargo Targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html) — 타깃의 종류, 타깃 표의 키, 자동 탐색
  - [3.1.2 Rust Version](https://doc.rust-lang.org/cargo/reference/rust-version.html) — `rust-version`의 쓰임과 버전 정책
  - [3.16 Lints](https://doc.rust-lang.org/cargo/reference/lints.html) — Cargo 자신의 린트 목록
- 이 저장소의 관련 챕터
  - [ch1](../ch1-getting-started/README.md) 2.3절 — 관례가 정하는 타깃, 4.1절 — 바이너리가 여럿일 때의 `cargo run`
  - [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md) — 테스트 러너의 출력 읽기
  - [Part 2 ch1](../../part2-editions/ch1-editions-overview/README.md) 2.5절 — 에디션과 `rust-version`의 구분
  - [부록 A ch21](../../appd-a/ch21-semver/README.md) 2.7절 — MSRV를 약속으로 다루기
  - [부록 A ch29](../../appd-a/ch29-listen-to-clippy/README.md) — Clippy 그룹 고르기와 `[lints]` 운영
