# ch8. 빌드 스크립트: `build.rs`, `cargo::` 지시문, 코드 생성, 네이티브 라이브러리

Cargo는 `rustc`로 Rust 코드를 컴파일하는 일은 잘하지만, 그 앞에 필요한 다른 일까지 알지는 못합니다. C로 쓴 라이브러리를 먼저 컴파일해야 하거나, 데이터 파일이나 명세에서 Rust 코드를 만들어야 하거나, 빌드하는 컴퓨터와 대상 플랫폼을 보고 설정을 바꿔야 하는 경우입니다. 빌드 스크립트(build script)는 이런 일을 하는 작은 Rust 프로그램입니다. 패키지 디렉터리에 `build.rs`를 두면, Cargo가 패키지를 컴파일하기 직전에 그것을 컴파일해 실행하고, 그 프로그램이 출력한 지시문(instruction)을 따라 패키지를 컴파일합니다.

앞 챕터들이 이 챕터로 미뤄 둔 것이 여럿 있습니다. [ch2](../ch2-dependencies/README.md) 2.4절의 `[build-dependencies]`, [ch3](../ch3-manifest-targets/README.md) 2.2절 표의 `build`·`links` 키, [ch5](../ch5-features/README.md) 2.6절의 `CARGO_FEATURE_<이름>`, [ch6](../ch6-profiles-build-performance/README.md) 2.4절의 "빌드 스크립트는 최적화하지 않는다", [ch7](../ch7-configuration-env/README.md) 2.7절에서 이름만 본 빌드 스크립트용 환경 변수, [Part 2 ch6](../../part2-editions/ch6-migration-practice/README.md)에서 `cargo fix`가 고치지 못한 "빌드 스크립트가 만든 코드"입니다.

- 패키지: `p3-ch8-build-scripts` (이 챕터 패키지에도 `build.rs`가 있습니다)
- 기준 문서: The Cargo Book [3.8 Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html), [3.8.1 Build Script Examples](https://doc.rust-lang.org/cargo/reference/build-script-examples.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, Apple clang 21.0.0, macOS (aarch64)
- 실습 자료: 챕터의 [`build.rs`](build.rs)와 [`data/`](data/units.txt), `examples/` 2개, 독립 프로젝트 [`projects/fletcher`](projects/fletcher/Cargo.toml)(C 라이브러리를 감싼 `-sys` 패키지와 그 위의 안전한 패키지). `compile_fail/`은 없습니다. 이 챕터의 실수는 빌드 스크립트나 Cargo가 내는 에러라서 4절에 메시지를 표로 모았습니다.
- 외부 크레이트: `projects/fletcher`가 빌드 의존성으로 `cc` 1.11.3을 씁니다(그 아래 `find-msvc-tools`, `shlex`). 처음 빌드할 때 crates.io에서 내려받습니다. C 컴파일러가 있어야 합니다(macOS는 Xcode Command Line Tools의 `cc`).
- 챕터 패키지에 기능 `extra-units`가 있어 예제 01의 출력이 `--features`에 따라 달라집니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 빌드 스크립트가 언제 컴파일되고 실행되는지, 무엇을 입력으로 받고(환경 변수, 현재 디렉터리) 어디에 결과를 두는지(`OUT_DIR`) 설명할 수 있다.
2. 빌드 스크립트로 데이터 파일에서 Rust 코드를 만들어 `include!`로 가져다 쓰고, `rerun-if-changed`로 다시 실행할 때를 정할 수 있다.
3. `cargo::` 지시문(`rustc-env`, `rustc-cfg`·`rustc-check-cfg`, `rustc-link-lib`·`rustc-link-search`, `warning`·`error`, `metadata`)이 각각 무엇을 바꾸는지 설명하고, `-vv`와 build 디렉터리에서 빌드 스크립트의 출력을 확인할 수 있다.
4. `[build-dependencies]`의 `cc`로 C 코드를 컴파일해 링크하고, `links` 키와 `DEP_<links>_<키>`로 `-sys` 패키지가 의존하는 패키지에 정보를 넘기는 방법을 설명할 수 있다.

## 2. 핵심 개념

### 2.1 빌드 스크립트의 한살이

패키지 디렉터리(매니페스트 옆)에 `build.rs`가 있으면 Cargo는 패키지를 빌드할 때마다 다음을 합니다.

1. `build.rs`를 실행 파일로 **컴파일**합니다(이미 컴파일했고 바뀐 것이 없으면 건너뜀).
2. 그 실행 파일을 **실행**합니다(다시 실행할 조건에 해당할 때만, 2.4절). 빌드 스크립트는 파일을 만들거나 다른 프로그램(C 컴파일러 등)을 부르는 등 필요한 일을 하고, 표준 출력에 `cargo::`로 시작하는 지시문을 써서 Cargo에게 알립니다.
3. 빌드 스크립트가 성공(종료 코드 0)하면 그 지시문을 반영해 **패키지를 컴파일**합니다. 0이 아니면 빌드를 멈추고 빌드 스크립트의 출력을 보여 줍니다.

패키지의 모든 타깃(라이브러리, 바이너리, 예제, 테스트)이 이 빌드 스크립트의 결과를 함께 씁니다. 1단계(컴파일)와 2단계(실행)는 ch6 2.6절의 `--timings` 보고서에서 따로 된 단위(unit)로 나타납니다. 이름 뒤에 `build-script`가 붙은 것이 컴파일이고 `build-script (run)`이 붙은 것이 실행이며, 실행 단위가 주황색 막대입니다. 이 챕터 패키지는 라이브러리 없이 예제만 있지만, 예제를 컴파일하기 전에 `build.rs`가 먼저 실행됩니다.

빌드 스크립트는 Rust 프로그램이라 Cargo 자체를 고치지 않고도 무엇이든 할 수 있습니다. Cargo Book은 Cargo가 이런 일(C 빌드, 파서 생성)을 잘하는 다른 도구를 대신하려는 것이 아니라, 빌드 스크립트로 그 도구와 연결한다고 설명합니다. 3.8.1절은 자주 쓰는 빌드 의존성으로 C·C++ 컴파일러를 부르는 `cc`, C 헤더로 Rust 선언을 만드는 `bindgen`([부록 A ch35](../../appd-a/ch35-bindgen/README.md)), 시스템 라이브러리를 찾는 `pkg-config`, `cmake`를 실행하는 `cmake`, 컴파일러 버전을 보고 조건을 정하는 `autocfg`·`rustc_version`·`version_check`를 듭니다.

**이름과 끄기.** 매니페스트의 `build` 키([ch3](../ch3-manifest-targets/README.md))로 빌드 스크립트 파일을 바꿀 수 있습니다. `build = "tools/gen.rs"`처럼 다른 경로를 적거나, `build = false`로 `build.rs`가 있어도 쓰지 않게 합니다. 이 챕터 패키지의 사본에서 `build = false`를 적자 예제가 `OUT_DIR`을 찾지 못하는 컴파일 에러가 났습니다(4절).

**빌드 의존성.** 빌드 스크립트가 쓰는 크레이트는 `[dependencies]`가 아니라 `[build-dependencies]`에 적습니다. 빌드 스크립트는 패키지보다 먼저 실행되므로 `[dependencies]`의 크레이트는 아직 빌드되지 않았기 때문입니다. 반대로 `[build-dependencies]`의 크레이트는 패키지 코드에서 쓸 수 없습니다. 빌드 스크립트와 빌드 의존성은 ch6 2.4절의 `build-override` 기본값 때문에 `release`에서도 최적화하지 않고 빌드됩니다. 프로필의 `panic = "abort"`도 빌드 스크립트에는 걸리지 않습니다(ch6 2.2절). 사본의 `[profile.dev]`에 `panic = "abort"`를 적고 빌드 스크립트를 패닉시켜도 종료 상태는 abort의 134가 아니라 되감기의 101이었습니다.

### 2.2 입력과 출력

**입력은 환경 변수와 현재 디렉터리입니다.** 빌드 스크립트의 현재 디렉터리는 패키지 디렉터리라서 `fs::read_to_string("data/units.txt")`처럼 패키지의 파일을 상대 경로로 읽습니다. 나머지 정보는 Cargo가 환경 변수로 넣어 줍니다.

| 환경 변수 | 값 |
| --- | --- |
| `OUT_DIR` | 결과를 둘 디렉터리. 패키지마다 따로 있고 build 디렉터리 안에 있음 |
| `TARGET`, `HOST` | 빌드 대상 플랫폼, 빌드 스크립트가 실행되는 플랫폼(타깃 이름) |
| `PROFILE` | `release`를 물려받은 프로필이면 `release`, 아니면 `debug`. Cargo Book은 이 값보다 아래 둘을 쓰라고 권함 |
| `OPT_LEVEL`, `DEBUG` | 지금 프로필의 `opt-level`, 디버그 정보를 만드는지(`true`·`false`) |
| `CARGO_FEATURE_<이름>` | 켜진 기능마다 하나(대문자, `-`는 `_`). 값이 아니라 있는지를 봄 |
| `CARGO_CFG_<이름>` | 대상 플랫폼의 cfg 값(`CARGO_CFG_TARGET_OS=macos` 등) |
| `CARGO_MANIFEST_DIR`, `CARGO_PKG_*` 등 | ch7 2.7절의 변수(크레이트를 컴파일할 때와 같은 이름) |
| `DEP_<links>_<키>` | 의존하는 `links` 패키지가 넘긴 정보(2.5절) |
| `RUSTC`, `NUM_JOBS`, `CARGO_ENCODED_RUSTFLAGS` 등 | Cargo가 쓰는 컴파일러, 병렬 작업 수, `rustc` 옵션 |

이 값들은 빌드 스크립트가 **실행될 때** 들어오므로 `env!`가 아니라 `std::env::var`로 읽습니다. 빌드 스크립트를 컴파일하는 시점에는 아직 없기 때문입니다. Cargo 설정의 `[env]`(ch7 2.5절)로 정한 환경 변수도 빌드 스크립트에 들어갑니다. 사본의 `build.rs`에서 `FOO`를 읽어 `cargo::warning`으로 출력하게 하고 `cargo build --examples --config 'env.FOO="from-config"'`로 빌드하자 `FOO=Some("from-config")`가 나왔습니다.

**대상 플랫폼은 `cfg!`로 보지 않습니다.** 빌드 스크립트는 빌드하는 컴퓨터(호스트)용으로 컴파일되어 호스트에서 실행됩니다. 그래서 빌드 스크립트 안의 `cfg!(target_os = "windows")`는 호스트를 가리킵니다. 다른 플랫폼용으로 크로스 컴파일할 때는 둘이 달라지므로, 대상 플랫폼은 `CARGO_CFG_TARGET_OS` 같은 환경 변수로 봅니다(크로스 컴파일은 Part 5).

**출력은 두 가지입니다.**

- **`OUT_DIR`에 쓴 파일.** 만든 소스 코드, 컴파일한 라이브러리 같은 결과물입니다. 빌드 스크립트는 `OUT_DIR` 밖의 파일을 고치면 안 됩니다. 패키지가 crates.io에서 받은 의존성일 때 그 소스는 고칠 수 없는 것이 전제이기 때문입니다. Cargo는 다시 실행할 때 `OUT_DIR`을 비우지 않으므로 지난번 파일이 남아 있을 수 있습니다.
- **표준 출력의 `cargo::` 지시문.** Cargo가 읽는 줄입니다. 그 밖의 줄은 무시합니다(2.4절).

평소 빌드에서는 빌드 스크립트의 출력이 보이지 않습니다. `-vv`(아주 자세히)를 주면 빌드 스크립트가 실행될 때 그 출력이 `[패키지 버전]`을 앞에 붙여 나타납니다. 출력은 build 디렉터리에도 남습니다. Cargo Book은 그 위치를 `target/debug/build/<pkg>/output`처럼 적으면서 설정에 따라 다를 수 있다고 하는데, 1.100.0에서는 `target/debug/build/<패키지>/<해시>/run/` 아래의 `stdout`·`stderr`였고, `OUT_DIR`은 같은 해시 아래의 `out/`이었습니다(ch6 2.5절의 구조).

### 2.3 코드 생성

빌드 스크립트의 대표적인 쓰임은 빌드할 때 Rust 코드를 만드는 코드 생성(code generation)입니다. 이 챕터 패키지의 [`build.rs`](build.rs)는 단위 표 [`data/units.txt`](data/units.txt)를 읽어 `OUT_DIR/units.rs`를 만듭니다.

```text
# 단위 이름과 미터로 바꾸는 배수. 빌드 스크립트가 이 파일로 Rust 코드를 만듭니다.
mm 0.001
cm 0.01
m 1
km 1000
```

만들어진 `units.rs`는 이렇습니다(기본 기능으로 빌드했을 때).

```rust
// 빌드 스크립트가 data/*.txt로 만든 파일입니다. 고치지 마세요.

pub const UNITS: &[(&str, f64)] = &[
    ("mm", 0.001_f64),
    ("cm", 0.01_f64),
    ("m", 1_f64),
    ("km", 1000_f64),
];

pub fn factor(unit: &str) -> Option<f64> {
    match unit {
        "mm" => Some(0.001_f64),
        "cm" => Some(0.01_f64),
        "m" => Some(1_f64),
        "km" => Some(1000_f64),
        _ => None,
    }
}
```

예제는 이 파일을 `include!`로 가져옵니다.

```rust
include!(concat!(env!("OUT_DIR"), "/units.rs"));
```

`env!("OUT_DIR")`은 빌드 스크립트가 있는 패키지를 컴파일할 때 Cargo가 넣어 주는 값이고, `concat!`이 경로를 이어 붙이고, `include!`가 그 파일의 내용을 이 자리에 붙여 넣은 것처럼 컴파일합니다. 실행할 때 파일을 읽는 것이 아니라서 표가 실행 파일 안에 들어갑니다. 데이터를 손으로 Rust 코드로 옮겨 적는 대신 데이터 파일을 하나의 원본으로 두는 것이 코드 생성의 이점입니다. 표가 커지거나(유니코드 표, 명령어 집합) 다른 형식의 명세(프로토콜 정의, C 헤더)에서 코드를 만들 때 특히 그렇습니다.

`build.rs`는 기능 `extra-units`가 켜졌는지를 `CARGO_FEATURE_EXTRA_UNITS`로 확인해 [`data/extra_units.txt`](data/extra_units.txt)도 읽습니다. 기능에 따라 만들어지는 코드가 달라지는 것입니다.

**만든 코드를 고치려면 생성기를 고칩니다.** `OUT_DIR`의 파일은 빌드 결과물이라 빌드 스크립트가 다시 실행되면 덮어쓰입니다. [Part 2 ch6](../../part2-editions/ch6-migration-practice/README.md)에서 `cargo fix --edition`이 `out/gen.rs`를 고쳤지만 에디션을 올리자 빌드 스크립트가 옛 코드를 다시 만든 것이 그 예입니다. 만든 파일을 저장소에 커밋해 소스처럼 다루고 싶다면, 빌드 스크립트로 만들지 말고 "커밋한 파일이 생성 결과와 같은지" 확인하는 테스트를 두라고 Cargo Book은 권합니다(부록 A ch35가 `bindgen` 결과로 이 방법을 보였습니다).

### 2.4 `cargo::` 지시문

빌드 스크립트가 표준 출력에 쓰는 `cargo::이름=값` 줄이 지시문입니다. 이 챕터에서 쓰는 것과 나머지를 나누면 다음과 같습니다.

| 지시문 | 하는 일 | 이 챕터에서 |
| --- | --- | --- |
| `rerun-if-changed=경로` | 그 파일(디렉터리면 그 안 전체)이 바뀌었을 때만 빌드 스크립트를 다시 실행 | 챕터 `build.rs`, `fletcher-sys` |
| `rerun-if-env-changed=이름` | 그 환경 변수가 바뀌었을 때 다시 실행 | `cc`가 출력함 |
| `rustc-env=이름=값` | 패키지를 컴파일할 때 환경 변수를 넣음. 코드는 `env!`로 읽음 | 예제 02, `fletcher` |
| `rustc-cfg=이름` 또는 `이름="값"` | `--cfg`를 넘겨 `#[cfg(이름)]`을 켬 | `fletcher` |
| `rustc-check-cfg=cfg(…)` | 직접 지은 cfg 이름을 알려 `unexpected_cfgs` 경고를 막음(1.80부터 반영) | `fletcher` |
| `rustc-link-lib=[종류=]이름` | 네이티브 라이브러리를 링크(`-l`) | `cc`가 출력함 |
| `rustc-link-search=[종류=]경로` | 라이브러리를 찾을 경로를 더함(`-L`) | `cc`가 출력함 |
| `warning=메시지` | 빌드 스크립트가 끝난 뒤 경고를 보여 줌. 빌드는 계속 | 챕터 `build.rs`(중복 단위) |
| `error=메시지` | 에러를 보여 주고 빌드를 실패시킴(1.84부터 반영) | 챕터 `build.rs`(형식이 틀린 줄) |
| `metadata=키=값` | `links` 패키지가 직접 의존하는 패키지의 빌드 스크립트에 정보를 넘김 | `fletcher-sys` |
| `rustc-link-arg…`, `rustc-flags` | 링커 옵션, `-l`·`-L` 묶음 | 표만 |

- `cargo::` 두 콜론 문법은 1.77부터입니다. 그 전에는 `cargo:키=값` 한 콜론을 썼고, 지금도 읽습니다. 3절에서 보듯 `cc` 1.11.3은 한 콜론 문법으로 출력합니다.
- `rustc-env`로 넣은 값은 `cargo run`·`cargo test`로 실행할 때도 들어온다고 Cargo Book은 적고, 1.100.0에서도 그랬습니다(`OUT_DIR`도 마찬가지). 하지만 실행 파일을 Cargo 없이 실행하면 없으므로, Cargo Book은 `env!`로 컴파일할 때 읽으라고 권합니다.
- `rustc-cfg`의 이름은 ch7 2.2절의 `--cfg from_config`와 같은 것입니다. Cargo Book은 `rustc-check-cfg`를 `rustc-cfg`와 가까이 두라고 권합니다. 켤 수 있는 cfg는 **켜지 않을 때도** 모두 알려 둬야 하기 때문입니다.
- `warning`은 path 의존성(지금 작업 중인 패키지)에서만 보입니다. crates.io에서 받은 크레이트의 경고는 빌드가 실패하지 않는 한 `-vv`에서만 보입니다.
- 지시문의 순서가 `rustc`에 넘기는 옵션의 순서가 되고, 그것이 링커에 넘기는 순서에 영향을 줄 수 있습니다. 라이브러리끼리 의존할 때는 순서에 주의합니다.

**다시 실행할 때.** `rerun-if-…` 지시문을 하나도 출력하지 않으면 Cargo는 **패키지 안의 어느 파일이 바뀌어도** 빌드 스크립트를 다시 실행합니다. 안전하지만 대개 불필요하게 자주 실행됩니다. 하나라도 출력하면 그것들에 해당할 때만 다시 실행합니다. 이 챕터 패키지에는 라이브러리·바이너리가 없어서 `cargo build`만으로는 아무것도 빌드하지 않고 빌드 스크립트도 실행하지 않습니다(`Finished` 한 줄만 나옴). 그래서 아래 실험은 `--examples`를 붙여 예제까지 빌드합니다. 파일이 바뀌었는지는 내용이 아니라 수정 시각(mtime)으로 판단하므로, 내용을 그대로 두고 `touch`만 해도 바뀐 것으로 봅니다. 이 챕터 패키지의 사본에서 확인한 결과입니다.

| `build.rs` | `touch README.md` 뒤 `cargo build -v --examples` |
| --- | --- |
| `rerun-if-changed` 지시문 없음 | `Running …/build_script_build`(다시 실행) |
| `rerun-if-changed=build.rs`, `=data/units.txt` | ``Fresh p3-ch8-build-scripts`` (다시 실행하지 않음) |

입력 파일이 없는 빌드 스크립트라면 `cargo::rerun-if-changed=build.rs` 한 줄로 "빌드 스크립트 자신이 바뀔 때만"을 나타냅니다. 빌드 스크립트의 소스가 바뀌면 지시문과 상관없이 다시 컴파일되고 실행되므로, 입력 파일을 적었다면 `build.rs` 줄은 없어도 같습니다(Cargo Book은 이때 불필요하다고 적음). 이 챕터의 `build.rs`는 감시 대상을 한눈에 보이려고 함께 적었습니다. 또 코드가 `env!`·`option_env!`로 읽는 환경 변수는 Cargo가 알아서 추적하므로(ch7 2.7절) `rerun-if-env-changed`가 필요 없습니다.

### 2.5 네이티브 라이브러리 링크와 `-sys` 패키지

C 라이브러리를 Rust에서 쓰려면 세 가지가 필요합니다. 함수의 선언(`unsafe extern "C"` 블록, [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)), 라이브러리의 기계어(컴파일된 `.a`나 시스템의 공유 라이브러리), 그리고 링커에게 그 라이브러리를 알려 주는 일입니다. 뒤의 둘을 빌드 스크립트가 합니다. 정적 라이브러리(static library, `.a`)는 링크할 때 기계어가 실행 파일 안에 복사되고, 공유 라이브러리(shared library, `.so`·`.dylib`·`.dll`)는 실행할 때 운영체제가 불러옵니다. 링크 방식의 자세한 내용은 Part 9 `ch9-memory-panic-linkage`에서 다룹니다.

**`cc` 크레이트.** `projects/fletcher`의 [`fletcher-sys/build.rs`](projects/fletcher/fletcher-sys/build.rs)는 C 소스를 이렇게 컴파일합니다.

```rust
println!("cargo::rerun-if-changed=csrc/fletcher.c");
println!("cargo::rerun-if-changed=csrc/fletcher.h");
cc::Build::new().file("csrc/fletcher.c").compile("fletcher");
```

`compile("fletcher")`는 플랫폼에 맞는 C 컴파일러(macOS·Linux는 `cc`, Windows는 MSVC 등)로 `OUT_DIR`에 정적 라이브러리 `libfletcher.a`를 만들고, `rustc-link-lib=static=fletcher`와 `rustc-link-search=native=<OUT_DIR>`을 출력합니다. `static=`은 정적 라이브러리로 링크하라는 종류이고, `native=`는 그 경로가 네이티브 라이브러리를 찾을 곳이라는 종류입니다(2.4절 표의 `[종류=]`). `TARGET`, `OPT_LEVEL`, `DEBUG`를 보고 컴파일러 옵션도 맞춥니다. Cargo Book 3.8.1절은 같은 일을 `gcc`와 `ar`을 직접 불러 하는 판도 보여 주는데, 그 판은 `gcc`가 없는 플랫폼과 크로스 컴파일에서 깨집니다. `cc`는 C 파일의 `rerun-if-changed`를 대신 출력해 주지 않으므로 직접 적습니다.

**`links` 키.** `fletcher-sys`의 매니페스트에 `links = "fletcher"`가 있습니다. 이 패키지가 네이티브 라이브러리 `libfletcher`를 링크한다는 선언입니다.

- **한 빌드에 한 패키지만** 같은 `links` 값을 가질 수 있습니다. 두 패키지가 같은 C 라이브러리를 각자 링크하면 같은 심볼이 두 번 들어가기 때문입니다. 두 버전이 함께 들어오려 하면 의존성 해석 단계에서 거절됩니다([부록 A ch25](../../appd-a/ch25-dependency-graph/README.md)).
- `links`가 있는 패키지는 빌드 스크립트가 있어야 하고, 그 빌드 스크립트가 `rustc-link-lib`로 라이브러리를 링크해야 합니다.
- 빌드 스크립트가 출력한 `cargo::metadata=키=값`은 이 패키지에 **직접** 의존하는 패키지의 빌드 스크립트에 환경 변수 `DEP_<links 값>_<키>`(대문자)로 들어갑니다. 전이 의존성에는 가지 않습니다.

`fletcher-sys`는 헤더의 `#define FLETCHER_VERSION 2`를 읽어 `cargo::metadata=version=2`를 출력하고, 그 위의 `fletcher` 패키지의 [`build.rs`](projects/fletcher/fletcher/build.rs)는 그것을 `DEP_FLETCHER_VERSION`으로 받아 cfg를 켭니다.

```rust
println!("cargo::rustc-check-cfg=cfg(fletcher_v2)");
let version: u32 = env::var("DEP_FLETCHER_VERSION")…;
if version >= 2 {
    println!("cargo::rustc-cfg=fletcher_v2");
}
```

`fletcher`의 `lib.rs`는 C 라이브러리 버전 2에서 생긴 함수를 감싼 `verify`를 `#[cfg(fletcher_v2)]` 뒤에 둡니다. 링크한 C 라이브러리가 그 함수를 갖고 있을 때만 Rust 쪽 API가 생기는 것입니다. Cargo Book 3.8.1절의 `openssl` 크레이트가 같은 방법으로 OpenSSL의 여러 버전을 지원합니다.

**`-sys` 관례.** 네이티브 라이브러리 `libfoo`를 링크하는 패키지는 흔히 `foo-sys`라고 이름 짓고, 두 가지만 합니다. 라이브러리를 링크하는 것(시스템에서 찾거나 소스에서 빌드), 그리고 C의 타입과 함수를 그대로 선언하는 것입니다. 안전한 고수준 API는 `-sys`가 붙지 않은 별도 패키지(`git2`와 `libgit2-sys`처럼)가 만듭니다. 이렇게 나누면 여러 패키지가 같은 `foo-sys`에 의존해 `links` 규칙을 지키고, 라이브러리를 찾는 방법을 한곳에 모으고, 사용자가 설정으로 빌드 스크립트를 재정의할 수 있습니다. `projects/fletcher`가 이 모양입니다.

**빌드 스크립트 재정의(overriding build scripts).** `links`가 있는 패키지는 Cargo 설정 파일(ch7)의 `[target.<타깃>.<links 값>]` 표에 `rustc-link-lib`, `rustc-link-search`, 메타데이터 값을 미리 적어 빌드 스크립트를 **컴파일하지도 실행하지도 않게** 할 수 있습니다. 시스템에 이미 설치된 라이브러리를 쓰게 하려는 장치입니다. `projects/fletcher`의 사본에서 미리 만든 `libfletcher.a`를 `prebuilt/`에 두고 `.cargo/config.toml`에 다음을 적자, `cargo build -v`에서 `cc`와 `fletcher-sys`의 빌드 스크립트가 컴파일되지 않았고 `fletcher`의 빌드 스크립트는 설정의 `version = "1"`을 받아 `C 라이브러리 버전: 1`이 출력되었습니다.

```toml
[target.aarch64-apple-darwin.fletcher]
rustc-link-lib = ["static=fletcher"]
rustc-link-search = ["/절대/경로/prebuilt"]
version = "1"
```

FFI의 설계 지침(선언을 헤더와 맞추기, 할당과 해제, 패닉 막기)은 [부록 A ch34](../../appd-a/ch34-ffi-boundaries/README.md)가 다룹니다. `projects/fletcher`의 `unsafe` 블록은 빌린 슬라이스의 포인터와 길이를 넘기기만 하고, 블록마다 `// SAFETY:` 주석으로 전제를 적었습니다. C 코드를 거치는 호출은 Miri로 검증할 수 없어 검증하지 않았습니다.

**병렬 작업.** Cargo는 동시에 돌리는 작업 수를 `--jobs`(기본은 논리 CPU 수)로 제한하는데, 빌드 스크립트 하나는 그 가운데 한 자리(job slot)를 받아 실행됩니다. 여러 CPU를 쓰려면 `jobserver` 크레이트로 Cargo와 조율해야 하고, `cc`의 `parallel` 기능이 그렇게 C 파일 여럿을 동시에 컴파일합니다.

## 3. 예제 안내

예제는 저장소 루트에서 실행합니다. 예제를 처음 컴파일할 때 챕터의 `build.rs`가 먼저 컴파일·실행됩니다.

### 01_generated_units — 빌드 스크립트가 만든 코드 쓰기

[`examples/01_generated_units.rs`](examples/01_generated_units.rs)는 2.3절의 `units.rs`를 `include!`로 가져와 단위를 바꿉니다.

```console
$ cargo run -q -p p3-ch8-build-scripts --example 01_generated_units
단위 4개: mm, cm, m, km
5 km = 5000 m
250 cm = 2.5 m
1 mi → km: 모르는 단위
$ cargo run -q -p p3-ch8-build-scripts --example 01_generated_units --features extra-units
단위 7개: mm, cm, m, km, in, ft, mi
5 km = 5000 m
250 cm = 2.5 m
1 mi = 1.609344 km
```

예제 소스는 한 글자도 다르지 않습니다. 기능이 켜지자 빌드 스크립트가 `extra_units.txt`까지 읽어 다른 `units.rs`를 만들었습니다. 기능 조합마다 build 디렉터리에 해시가 다른 결과가 따로 남습니다(ch6 2.5절).

빌드 스크립트의 출력은 `-vv`로 봅니다. 아래는 `touch part3-cargo/ch8-build-scripts/build.rs`로 다시 실행되게 한 뒤 `cargo build -vv -p p3-ch8-build-scripts --examples`를 실행한 출력에서 `[p3-ch8-build-scripts 0.0.0]`으로 시작하는 줄만 고른 것입니다.

```text
[p3-ch8-build-scripts 0.0.0] cargo::rerun-if-changed=build.rs
[p3-ch8-build-scripts 0.0.0] cargo::rerun-if-changed=data/units.txt
[p3-ch8-build-scripts 0.0.0] cargo::rustc-env=CH8_TARGET=aarch64-apple-darwin
[p3-ch8-build-scripts 0.0.0] cargo::rustc-env=CH8_HOST=aarch64-apple-darwin
[p3-ch8-build-scripts 0.0.0] cargo::rustc-env=CH8_PROFILE=debug
[p3-ch8-build-scripts 0.0.0] cargo::rustc-env=CH8_OPT_LEVEL=0
[p3-ch8-build-scripts 0.0.0] cargo::rustc-env=CH8_DEBUG=true
```

### 02_build_info — 빌드 스크립트가 받은 값을 넘겨받기

[`examples/02_build_info.rs`](examples/02_build_info.rs)는 `build.rs`가 `cargo::rustc-env=CH8_<이름>=…`으로 넘긴 값을 `env!`로 읽어 출력합니다.

```console
$ cargo run -q -p p3-ch8-build-scripts --example 02_build_info
TARGET    (빌드 대상): aarch64-apple-darwin
HOST      (빌드한 곳): aarch64-apple-darwin
PROFILE:   debug
OPT_LEVEL: 0
DEBUG:     true
크레이트에서 option_env!("TARGET"): None
$ cargo run -q -p p3-ch8-build-scripts --example 02_build_info --release
TARGET    (빌드 대상): aarch64-apple-darwin
HOST      (빌드한 곳): aarch64-apple-darwin
PROFILE:   release
OPT_LEVEL: 3
DEBUG:     false
크레이트에서 option_env!("TARGET"): None
```

- `TARGET`과 `HOST`가 같은 것은 이 컴퓨터용으로 빌드했기 때문입니다. 다른 컴퓨터에서는 그 컴퓨터의 타깃 이름이 나옵니다.
- `TARGET` 같은 변수는 빌드 스크립트에만 들어가고 크레이트를 컴파일할 때는 없으므로, 크레이트가 그 값을 쓰려면 빌드 스크립트가 `rustc-env`로 넘겨야 합니다. 마지막 줄의 `None`이 그것을 보여 줍니다.
- `DEBUG`는 프로필의 `debug` 설정을 따릅니다. `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only cargo run … --release`로 실행하면 `DEBUG: true`가 됩니다.

### projects/fletcher — C 라이브러리를 감싼 `-sys` 패키지

Fletcher-16 체크섬을 C로 구현한 작은 라이브러리를 빌드해 링크하는 가상 워크스페이스입니다([ch4](../ch4-workspaces/README.md)).

| 파일 | 내용 |
| --- | --- |
| [`Cargo.toml`](projects/fletcher/Cargo.toml) | 멤버 둘의 가상 매니페스트 |
| [`fletcher-sys/Cargo.toml`](projects/fletcher/fletcher-sys/Cargo.toml) | `links = "fletcher"`, `[build-dependencies] cc` |
| [`fletcher-sys/csrc/fletcher.h`](projects/fletcher/fletcher-sys/csrc/fletcher.h), [`fletcher.c`](projects/fletcher/fletcher-sys/csrc/fletcher.c) | C 라이브러리. `FLETCHER_VERSION 2`, `fletcher16`, 버전 2에서 생긴 `fletcher16_verify` |
| [`fletcher-sys/build.rs`](projects/fletcher/fletcher-sys/build.rs) | `cc`로 컴파일, 헤더의 버전을 `cargo::metadata`로 넘김 |
| [`fletcher-sys/src/lib.rs`](projects/fletcher/fletcher-sys/src/lib.rs) | C 함수의 선언만 |
| [`fletcher/build.rs`](projects/fletcher/fletcher/build.rs) | `DEP_FLETCHER_VERSION`을 받아 `fletcher_v2` cfg와 `FLETCHER_C_VERSION` 환경 변수를 넘김 |
| [`fletcher/src/lib.rs`](projects/fletcher/fletcher/src/lib.rs) | 안전한 `checksum`, cfg 뒤의 `verify`, 단위 테스트 |
| [`fletcher/src/main.rs`](projects/fletcher/fletcher/src/main.rs) | 인자 문자열의 체크섬을 출력 |

프로젝트 디렉터리에서 실행합니다.

```console
$ cargo build --locked
   Compiling shlex v2.0.1
   Compiling find-msvc-tools v0.1.15
   Compiling fletcher v0.1.0 (…/fletcher/fletcher)
   Compiling cc v1.11.3
   Compiling fletcher-sys v0.1.0 (…/fletcher/fletcher-sys)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.99s
$ cargo run -q --locked
C 라이브러리 버전: 2
fletcher16("abcde") = 51440 (0xC8F0)
C로 다시 검증: true
```

처음 빌드할 때는 `cc`를 내려받는 줄이 앞에 더 나오고, 병렬로 컴파일하므로 줄의 순서는 실행마다 조금 다를 수 있습니다. `fletcher-sys`보다 `fletcher`의 `Compiling` 줄이 먼저 나오는 것은 `fletcher`의 빌드 스크립트를 컴파일하는 단계이기 때문입니다. 빌드 스크립트를 **실행**하는 것은 `fletcher-sys`의 빌드 스크립트가 끝나 `DEP_FLETCHER_VERSION`이 정해진 뒤입니다. `cargo build -vv`로 두 빌드 스크립트의 출력을 보면 이렇습니다(`cc`가 출력한 줄은 `cargo:rerun-if-env-changed=CC`처럼 환경 변수를 확인하는 줄이 60줄 넘게 이어지므로 가운데를 줄였습니다).

```text
[fletcher-sys 0.1.0] cargo::rerun-if-changed=csrc/fletcher.c
[fletcher-sys 0.1.0] cargo::rerun-if-changed=csrc/fletcher.h
[fletcher-sys 0.1.0] cargo:rerun-if-env-changed=CC_FORCE_DISABLE
[fletcher-sys 0.1.0] CC_FORCE_DISABLE = None
…
[fletcher-sys 0.1.0] cargo:rustc-link-lib=static=fletcher
[fletcher-sys 0.1.0] cargo:rustc-link-search=native=…/fletcher/target/debug/build/fletcher-sys/<해시>/out
[fletcher-sys 0.1.0] cargo::metadata=version=2
[fletcher 0.1.0] cargo::rerun-if-changed=build.rs
[fletcher 0.1.0] cargo::rustc-check-cfg=cfg(fletcher_v2)
[fletcher 0.1.0] cargo::rustc-cfg=fletcher_v2
[fletcher 0.1.0] cargo::rustc-env=FLETCHER_C_VERSION=2
```

- `CC_FORCE_DISABLE = None` 같은 줄은 `cc`가 남긴 기록이고, `cargo:`로 시작하지 않으므로 Cargo는 무시합니다.
- `cc`는 C 컴파일러를 정하는 환경 변수(`CC`, `CFLAGS`, `AR` 등)마다 `rerun-if-env-changed`를 출력합니다. 컴파일러를 바꾸면 C 코드도 다시 컴파일되게 하려는 것입니다.

헤더를 고치면 다시 실행되는 범위는 `-v`로 볼 수 있습니다.

```console
$ touch fletcher-sys/csrc/fletcher.h
$ cargo build -v --locked
…
       Dirty fletcher-sys v0.1.0 (…/fletcher/fletcher-sys): the file `fletcher-sys/csrc/fletcher.h` has changed (…)
   Compiling fletcher-sys v0.1.0 (…/fletcher/fletcher-sys)
     Running `…/fletcher/target/debug/build/fletcher-sys/<해시>/out/build_script_build`
       Dirty fletcher v0.1.0 (…/fletcher/fletcher): the dependency `fletcher-sys` was rebuilt
   Compiling fletcher v0.1.0 (…/fletcher/fletcher)
     Running `…/fletcher/target/debug/build/fletcher/<해시>/out/build_script_build`
…
```

`fletcher-sys`의 빌드 스크립트가 다시 실행되자, 그 메타데이터를 받는 `fletcher`의 빌드 스크립트도 다시 실행되었습니다.

| 명령 | 기대 결과 |
| --- | --- |
| `cargo fmt --all --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings` | 경고 없음 |
| `cargo test --locked --workspace` | `fletcher`의 단위 테스트 2개 통과 |
| `cargo run -q --locked -- 안녕` | `fletcher16("안녕") = 18 (0x0012)` |
| `cargo build -vv --locked`(처음 빌드할 때) | 위의 빌드 스크립트 출력 |
| `cargo clean` | `target/` 삭제(실습을 마친 뒤) |

## 4. 자주 하는 실수

모두 이 챕터 패키지나 `projects/fletcher`를 저장소 밖에 복사한 사본에서 확인했습니다. 경로와 시각은 줄였습니다.

| 상황 | 메시지 | 해결 |
| --- | --- | --- |
| 빌드 스크립트가 패닉함(여기서는 `data/units.txt`를 지움) | ``error: failed to run custom build command for `p3-ch8-build-scripts v0.0.0 (…)` `` / `process didn't exit successfully: … (exit status: 101)` 아래에 `--- stdout`, `--- stderr`와 패닉 메시지 | 빌드 스크립트의 출력이 그대로 나오므로 `--- stderr`의 패닉 위치와 메시지를 읽습니다. `RUST_BACKTRACE=1`로 백트레이스를 봐도 빌드 스크립트는 디버그 정보 없이 빌드되어 줄 위치가 없습니다. `[profile.dev.build-override] debug = true`(ch6 2.4절)를 주면 `at ./build.rs:27:64` 같은 위치가 붙습니다. 워크스페이스 멤버라 프로필을 적을 수 없는 이 챕터 패키지에서는 환경 변수 `CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=true`로 같은 설정을 줍니다(ch7 2.3절). |
| 빌드 스크립트가 `cargo::error`를 출력함(형식이 틀린 줄) | ``error: p3-ch8-build-scripts@0.0.0: data/units.txt:7: `이름 배수` 형식이 아닙니다: bad-line`` / `error: build script logged errors` | 메시지가 가리키는 입력을 고칩니다. 종료 상태는 101입니다. |
| 빌드 스크립트가 `cargo::warning`을 출력함(중복 단위) | ``warning: p3-ch8-build-scripts@0.0.0: data/units.txt:6: 단위 `km`이 이미 있어 무시합니다`` | 빌드는 성공합니다. 빌드 스크립트가 다시 실행되지 않은 빌드(`Fresh`)에서도 Cargo가 저장해 둔 경고를 다시 보여 주므로, 입력을 고칠 때까지 계속 보입니다. |
| `build = false`인데 코드가 `OUT_DIR`을 씀 | ``error: environment variable `OUT_DIR` not defined at compile time`` / ``help: `OUT_DIR` may not be available for the current Cargo target`` | 빌드 스크립트가 있는 패키지에서만 `OUT_DIR`이 있습니다. `build` 키를 확인합니다. |
| `rerun-if-changed` 지시문이 없음 | (메시지 없음) | 패키지의 어느 파일이 바뀌어도 빌드 스크립트가 다시 실행됩니다(2.4절). 읽는 입력 파일을 모두 적습니다(입력이 없으면 `build.rs`). |
| 입력 파일을 `rerun-if-changed`에 빠뜨림 | (메시지 없음) | 그 파일을 고쳐도 빌드 스크립트가 다시 실행되지 않아 옛 결과가 남습니다. 읽는 파일은 모두 적습니다(연습 2). |
| 빌드 스크립트에서 `cfg!(target_os = …)`로 대상 플랫폼을 판단함 | (메시지 없음) | 호스트를 가리킵니다. `CARGO_CFG_TARGET_OS`를 읽습니다(2.2절). |
| `rustc-cfg`로 켠 이름을 `rustc-check-cfg`로 알리지 않음 | ``warning: unexpected `cfg` condition name: `fletcher_v2` `` | `cargo::rustc-check-cfg=cfg(fletcher_v2)`를 출력합니다(연습 3). |
| 빌드 스크립트가 `OUT_DIR` 밖(패키지 디렉터리)에 파일을 씀 | (메시지 없음) | 빌드할 때는 메시지가 없지만, crates.io에서 받은 의존성이면 소스를 고치는 셈이라 문제가 됩니다. Cargo Book은 패키징할 때 이런 빌드 스크립트를 허용하지 않는다고 적습니다(`cargo package`는 Part 3 `ch10-publishing-semver`). `OUT_DIR`에만 씁니다. |

## 5. 연습 문제

연습마다 챕터 패키지나 `projects/fletcher`를 저장소 밖에 새로 복사해 시작합니다. 챕터 패키지를 복사하면 매니페스트의 `edition.workspace = true`, `publish.workspace = true`를 `edition = "2024"`, `publish = false`로 바꾸고 빈 `[workspace]` 표를 더합니다. 이 챕터 패키지에는 예제만 있어 `cargo build`만으로는 빌드 스크립트가 실행되지 않으므로 사본에서는 `cargo build --examples`(자세히 보려면 `-v`)로 빌드합니다. 빌드 스크립트의 경고는 `-q`를 주면 보이지 않습니다.

### 연습 1. 단위 하나 더하기

챕터 패키지의 사본에서 `data/units.txt`에 `nmi 1852`(해리)를 더하고, 예제 01에 `(2.0, "nmi", "km")` 변환을 더하세요. Rust 코드에 단위 표를 손으로 적지 않아야 합니다. 그다음 `km 999`처럼 이미 있는 단위를 한 줄 더하고 `cargo build --examples`로 빌드하면 어떤 일이 일어나는지, 아무것도 고치지 않고 `cargo build -v --examples`로 다시 빌드해도 같은 메시지가 보이는지 확인하세요.

<details>
<summary>힌트</summary>

데이터 파일만 고치면 `rerun-if-changed=data/units.txt` 때문에 빌드 스크립트가 다시 실행되어 `units.rs`에 `nmi`가 생깁니다. 중복 줄은 `cargo::warning`으로 경고만 내고 먼저 나온 값을 씁니다. 아무것도 고치지 않고 다시 빌드하면 빌드 스크립트는 실행되지 않지만(`-v`로 보면 `Fresh`), Cargo가 지난번 실행의 경고를 저장해 두었다가 다시 보여 줍니다. 컴파일러 경고를 다시 보여 주는 것과 같습니다. 예제를 `cargo run -q`로 실행하면 `-q`가 이 경고를 숨기므로 보이지 않습니다.

</details>

### 연습 2. 다시 실행되지 않는 빌드 스크립트

챕터 패키지의 사본에서 `build.rs`의 `println!("cargo::rerun-if-changed=data/units.txt");` 줄만 지우고 `cargo build --examples`로 한 번 빌드한 뒤, `data/units.txt`의 `km` 배수를 `1000`에서 `1001`로 바꾸고 예제 01을 실행하세요. 결과를 예상한 뒤 확인하고, 이유를 설명하세요.

<details>
<summary>힌트</summary>

`rerun-if-changed=build.rs`가 남아 있으므로 Cargo는 `build.rs`가 바뀔 때만 빌드 스크립트를 다시 실행합니다. `data/units.txt`는 감시 목록에 없어 빌드 스크립트가 다시 실행되지 않고, 예제는 옛 `units.rs`로 `5 km = 5000 m`을 출력합니다. 지시문이 하나도 없을 때는 패키지의 모든 파일을 감시하므로 오히려 다시 실행됩니다. 빌드 스크립트가 읽는 파일은 모두 적어야 합니다.

</details>

### 연습 3. C 라이브러리 버전 1과 링크하기

`projects/fletcher`의 사본에서 `fletcher-sys/csrc/fletcher.h`의 버전을 `1`로 바꾸고, `fletcher16_verify`의 선언(헤더)과 정의(`fletcher.c`)를 지운 뒤 `cargo run`을 실행하세요. 출력이 어떻게 바뀌나요? 이어서 `fletcher/build.rs`의 `cargo::rustc-check-cfg=…` 줄을 지우고 다시 빌드하면 무엇이 나오나요?

<details>
<summary>힌트</summary>

`fletcher-sys`가 `version=1`을 넘기므로 `fletcher`의 빌드 스크립트가 `fletcher_v2`를 켜지 않고, `verify`와 `main.rs`의 검증 줄이 컴파일되지 않아 `C로 다시 검증: (버전 2 미만이라 없음)`이 나옵니다. C 쪽에 함수가 없어도 Rust 쪽에서 부르지 않으므로 링크 에러도 없습니다. `rustc-check-cfg`를 지우면 `#[cfg(fletcher_v2)]`가 있는 곳마다 ``unexpected `cfg` condition name: `fletcher_v2` `` 경고가 납니다.

</details>

### 연습 4. 메타데이터 하나 더 넘기기

`fletcher/build.rs`가 C 라이브러리의 버전과 함께 `fletcher-sys`의 C 소스 파일 이름 목록(`fletcher.c`)도 넘기게 하려고 합니다. `fletcher-sys`와 `fletcher` 중 어느 빌드 스크립트에 무엇을 더해야 하는지 쓰고, 직접 해서 `fletcher`의 `main.rs`가 `C 소스: fletcher.c`를 출력하게 하세요.

<details>
<summary>힌트</summary>

`fletcher-sys/build.rs`에 `println!("cargo::metadata=sources=fletcher.c");`를 더하면 `fletcher`의 빌드 스크립트에 `DEP_FLETCHER_SOURCES`로 들어옵니다. `fletcher/build.rs`가 그 값을 `cargo::rustc-env=FLETCHER_C_SOURCES=…`로 넘기고, `main.rs`가 `env!("FLETCHER_C_SOURCES")`로 읽습니다. 메타데이터는 직접 의존하는 패키지의 빌드 스크립트까지만 가고, 크레이트 코드까지 가려면 `rustc-env`를 한 번 더 거쳐야 합니다.

</details>

## 6. 참고 자료

- The Cargo Book: [3.8 Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html), [3.8.1 Build Script Examples](https://doc.rust-lang.org/cargo/reference/build-script-examples.html), [3.7 Environment Variables — 빌드 스크립트용](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts)
- crates.io의 [`cc`](https://crates.io/crates/cc)
- 표준 라이브러리: [`include!`](https://doc.rust-lang.org/std/macro.include.html), [`concat!`](https://doc.rust-lang.org/std/macro.concat.html)
