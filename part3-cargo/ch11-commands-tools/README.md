# ch11. Cargo 명령과 외부 도구

Part 3의 마지막 챕터입니다. 지금까지 쓴 Cargo 명령을 Cargo Book의 분류로 정리하고, 의존성 그래프의 패키지를 정확히 가리키는 패키지 ID 지정(Package ID Specification)을 배웁니다. 이어서 편집기나 다른 빌드 시스템 같은 외부 도구(external tools)가 Cargo와 함께 일하는 세 가지 방법(`cargo metadata`, `--message-format=json`, 사용자 정의 하위 명령(custom subcommand))을 보고, 하위 명령 `cargo hello`를 직접 만듭니다. 의존성이 앞으로의 컴파일러에서 깨질 수 있음을 알려 주는 장래 비호환 보고서(future-incompat report)도 다룹니다.

- 패키지: `p3-ch11-commands-tools`
- 기준 문서: The Cargo Book [3.10 Package ID Specifications](https://doc.rust-lang.org/cargo/reference/pkgid-spec.html), [3.11 External Tools](https://doc.rust-lang.org/cargo/reference/external-tools.html), [3.14 Future incompat report](https://doc.rust-lang.org/cargo/reference/future-incompat-report.html), [4. Cargo Commands](https://doc.rust-lang.org/cargo/commands/index.html)(특히 [cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html))
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64)
- 실습 자료: `examples/` 3개, 독립 프로젝트 [`projects/cargo-hello`](projects/cargo-hello/Cargo.toml)(사용자 정의 하위 명령). `compile_fail/`은 없습니다.
- 외부 크레이트: `projects/cargo-hello`가 JSON을 읽으려고 `serde_json` 1.0.154를 씁니다(그 아래 `serde_core`, `itoa`, `memchr` 등). 처음 빌드할 때 crates.io에서 내려받습니다.
- 하위 명령은 Cargo home의 `bin/`에 설치하지 않고, 빌드한 디렉터리를 그 명령에만 `PATH`에 더해 실행합니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Cargo 명령을 Cargo Book의 여섯 갈래(일반·빌드·매니페스트·패키지·게시·보고)로 나누고, 이 파트에서 각 명령을 어디서 썼는지 찾을 수 있다.
2. 패키지 ID 지정의 문법으로 의존성 그래프의 패키지 하나를 가리키고, `specification … is ambiguous` 에러를 풀 수 있다.
3. 외부 도구가 Cargo와 연동하는 세 방법을 설명하고, `cargo metadata`의 JSON에서 워크스페이스 멤버·타깃·의존성 그래프를 읽을 수 있다.
4. 사용자 정의 하위 명령 `cargo-<이름>`을 만들어 `cargo <이름>`으로 실행하고, Cargo가 넘기는 인자와 `CARGO` 환경 변수를 쓸 수 있다.
5. 장래 비호환 보고서가 무엇을 알려 주는지 설명하고, 연습용 레지스트리로 그 보고서를 만들어 볼 수 있다.

## 2. 핵심 개념

### 2.1 Cargo 명령의 분류

Cargo Book 4장은 명령을 여섯 갈래로 나눕니다(그 밖에 더는 쓰지 않거나 없어진 명령 `read-manifest`, `git-checkout`, `verify-project`를 따로 모아 둡니다). `cargo help <명령>`이 명령마다의 설명을, `cargo --list`가 설치된 명령과 별칭 목록을 보여 줍니다([ch7](../ch7-configuration-env/README.md) 2.4절).

| 갈래 | 명령 | 이 저장소에서 |
| --- | --- | --- |
| 일반 | `cargo`, `help`, `version` | |
| 빌드 | `build`, `check`, `run`, `test`, `bench`, `clean`, `doc`, `fetch`, `fix`, `rustc`, `rustdoc`, `clippy`, `fmt`, `miri` | `build`·`run`·`check` [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md), `test`·`bench` [ch9](../ch9-testing-ci/README.md), `clean`·빌드 캐시 [ch6](../ch6-profiles-build-performance/README.md), `fetch` [ch1](../ch1-getting-started/README.md) 2.6절, `fix` [Part 2 ch6](../../part2-editions/ch6-migration-practice/README.md), `doc`·`rustdoc`는 Part 4, `rustc`는 Part 5 |
| 매니페스트 | `add`, `remove`, `update`, `tree`, `generate-lockfile`, `info`, `locate-project`, `metadata`, `pkgid`, `vendor` | `add`·`remove`·`update`·`tree` [ch2](../ch2-dependencies/README.md) 2.5절, `info` [ch10](../ch10-publishing-semver/README.md) 2.2절, `metadata`·`pkgid`는 이 챕터 |
| 패키지 | `new`, `init`, `install`, `uninstall`, `search` | `new`·`init` [ch1](../ch1-getting-started/README.md) 2.2절, `install`·`uninstall` [ch10](../ch10-publishing-semver/README.md) 2.8절 |
| 게시 | `login`, `logout`, `owner`, `package`, `publish`, `yank` | [ch10](../ch10-publishing-semver/README.md) |
| 보고 | `report`, `report future-incompatibilities` | 2.6절 |

- `clippy`·`fmt`·`miri`는 Cargo Book이 빌드 명령 목록에 함께 싣지만, Cargo 안에 들어 있는 명령이 아니라 rustup이 설치하는 별도 실행 파일(`cargo-clippy` 등)입니다. 2.5절의 사용자 정의 하위 명령과 같은 방식으로 실행됩니다([부록 A ch31](../../appd-a/ch31-tooling-ecosystem/README.md) 2.1절). `miri`는 이 저장소에 설치돼 있지 않습니다.
- `cargo locate-project`는 지금 디렉터리에서 쓰일 매니페스트의 경로를 JSON(`{"root":"…/Cargo.toml"}`)으로, `--workspace`를 주면 워크스페이스 루트의 매니페스트를 알려 줍니다. 셸 스크립트가 프로젝트 루트를 찾을 때 씁니다.
- `cargo vendor`는 의존성의 소스를 프로젝트 안의 디렉터리로 복사하고, 그것을 쓰게 할 소스 교체 설정을 출력합니다. [ch2](../ch2-dependencies/README.md) 2.6절의 연습용 레지스트리가 쓴 디렉터리 소스(`directory = "vendor"`)가 그 결과물의 형식입니다.

### 2.2 패키지 ID 지정

`-p 이름`, `cargo update 이름`, `cargo tree -i 이름`처럼 많은 명령이 의존성 그래프의 패키지 **하나**를 가리켜야 합니다. 보통은 이름만으로 충분하지만, 같은 이름의 패키지가 그래프에 둘 이상이면 이름만으로는 모호합니다. 패키지 ID 지정(Package ID Specification)은 패키지 하나를 유일하게 가리키는 문자열의 문법입니다.

```text
spec := 이름 [ ("@" | ":") 버전 ]
      | [ 종류 "+" ] 프로토콜 "://" 호스트와 경로 [ "?" 질의 ] [ "#" ( 이름 [@버전] | 버전 ) ]
종류 := "registry" | "git" | "path"
```

| 지정 | 가리키는 것 |
| --- | --- |
| `rand` | 이름이 `rand`인 패키지(그래프에 하나뿐일 때) |
| `rand@0.8` | `rand` 0.8.x |
| `rand@0.10.4` | `rand` 0.10.4 |
| `https://github.com/rust-lang/crates.io-index#rand@0.10.4` | crates.io 색인의 `rand` 0.10.4 |
| `registry+https://github.com/rust-lang/crates.io-index#rand@0.10.4` | 위와 같음. 출처의 종류까지 적은 완전한 형태 |
| `path+file:///…/foo#1.1.8` | 디스크의 경로에 있는 `foo` 1.1.8. 경로의 마지막 디렉터리 이름이 패키지 이름과 같으면 이름을 생략하고 `#버전`만 씀 |

이름 뒤에 `:`를 써도 되므로 [ch6](../ch6-profiles-build-performance/README.md) 2.4절의 `[profile.dev.package."foo:2.1.0"]`도 같은 문법입니다. Cargo가 출력하는 패키지 ID(`cargo metadata`의 `id`, `cargo pkgid`의 결과)는 늘 완전한 형태입니다.

이 저장소의 워크스페이스에는 `rand`가 두 버전 들어 있습니다. Part 0 ch2가 쓰는 0.10.4와, Part 0이 쓰는 `trpl`이 끌어온 0.8.8입니다. 그래서 `-p rand`는 모호하다는 에러가 나고, Cargo가 고를 수 있는 지정을 알려 줍니다(예제 01, [ch2](../ch2-dependencies/README.md) 2.5절에서 본 그 에러). 이런 에러는 버전을 붙인 지정으로 풉니다.

### 2.3 `cargo metadata`: 패키지 구조를 JSON으로

Cargo Book 3.11절은 외부 도구가 Cargo와 연동하는 방법으로 셋을 듭니다. 패키지 구조를 JSON으로 알려 주는 `cargo metadata`, 빌드 중에 일어난 일을 JSON으로 알려 주는 `--message-format=json`(2.4절), 그리고 사용자 정의 하위 명령(2.5절)입니다. 편집기의 Rust 지원(rust-analyzer 등)도 이 정보로 프로젝트를 이해합니다.

`cargo metadata`는 워크스페이스의 멤버와 의존성, 타깃, 해석된 의존성 그래프를 JSON 하나로 출력합니다. [ch3](../ch3-manifest-targets/README.md)과 [ch4](../ch4-workspaces/README.md)의 예제 02가 이것을 자식 프로세스로 실행해 문자열 검색으로 읽은 적이 있습니다.

- **`--format-version 1`을 꼭 줍니다.** 출력 형식은 안정적이고 버전이 붙어 있습니다. 앞으로 형식이 바뀌어도 내 도구가 깨지지 않도록 기대하는 버전을 명시합니다. 빼면 ``warning: please specify `--format-version` flag explicitly to avoid compatibility problems``가 표준 에러로 나옵니다.
- **`--no-deps`.** 멤버의 정보만 출력하고 의존성을 해석하지 않습니다. 빠르고 네트워크도 쓰지 않습니다. 이때 `resolve`는 `null`입니다.

1.100.0의 출력에서 자주 쓰는 필드입니다.

| 필드 | 내용 |
| --- | --- |
| `version` | 출력 형식의 버전(`1`) |
| `workspace_root`, `target_directory`, `build_directory` | 워크스페이스 루트, target·build 디렉터리의 절대 경로([ch6](../ch6-profiles-build-performance/README.md) 2.5절). `build_directory`는 문서에 불안정(unstable)으로 표시됨 |
| `workspace_members`, `workspace_default_members` | 멤버와 기본 멤버의 패키지 ID 목록 |
| `packages[]` | 패키지마다 `id`, `name`, `version`, `manifest_path`, 선언한 `dependencies`, `features`, `targets`, 게시 정보(`license`, `description` …), `rust_version`, `edition` |
| `packages[].targets[]` | 타깃마다 `kind`(`["lib"]`, `["bin"]`, `["example"]` …), `name`, `src_path`, `edition`, `test`, `doctest` |
| `resolve.nodes[]` | 해석이 끝난 그래프의 노드. 노드마다 `id`, 실제로 고른 직접 의존성의 ID 목록 `dependencies`, 켜진 `features` |

`packages[].dependencies`는 매니페스트에 **적은** 의존성(요구 사항, 종류, 선택적 여부)이고, `resolve.nodes[].dependencies`는 해석 결과 **고른** 패키지입니다. `resolve`에는 기본으로 **모든 플랫폼**의 의존성이 들어갑니다. 그래서 3절의 `cargo hello --deps`가 세는 패키지 수(12개)가 이 컴퓨터용으로 실제 빌드되는 것을 보여 주는 `cargo tree`(6개)보다 많고, `--filter-platform aarch64-apple-darwin`처럼 타깃을 주면 같아집니다. 같은 형식 버전 안에서도 필드가 더해질 수 있으므로 도구는 모르는 필드를 무시해야 합니다.

Rust로 이 JSON을 읽을 때 쓸 수 있는 크레이트로 Cargo Book은 `cargo_metadata`를 소개합니다. `projects/cargo-hello`는 의존성을 줄이려고 `serde_json`의 `Value`로 직접 읽었습니다. `Value`는 JSON 값 하나(null, 불리언, 수, 문자열, 배열, 객체)를 나타내는 열거형입니다. `v["키"]`와 `v[0]`은 그 키나 위치가 없으면 패닉하지 않고 `Value::Null`을 돌려주고, `as_str()`·`as_array()`·`as_object()`는 그 종류의 값일 때만 `Some`을 돌려줍니다. 그래서 `--no-deps`로 `resolve`가 `null`이어도 `meta["resolve"]["nodes"]`가 패닉하지 않습니다.

### 2.4 `--message-format=json`: 빌드 중에 일어난 일

빌드 명령에 `--message-format=json`을 주면 Cargo는 사람이 읽는 출력 대신, 일어난 일을 **한 줄에 JSON 객체 하나씩** 표준 출력에 씁니다. 줄마다의 `reason` 필드가 메시지의 종류입니다.

| `reason` | 내용 |
| --- | --- |
| `compiler-message` | 컴파일러의 경고·에러. `message` 안에 `rustc`의 JSON 진단이 들어 있고, `message.rendered`는 사람이 읽는 형태 |
| `compiler-artifact` | 만든 산출물. 타깃, 파일 경로, 실행 파일 경로(`executable`), 다시 쓰지 않고 그대로 둔 것인지(`fresh`) |
| `build-script-executed` | 빌드 스크립트의 실행 결과(링크할 라이브러리, cfg, 환경 변수, `OUT_DIR`)([ch8](../ch8-build-scripts/README.md)) |
| `build-finished` | 빌드가 끝났고 성공했는지(`success`) |

`Checking …`, `Finished …` 같은 진행 메시지는 그대로 표준 에러로 나갑니다. 이 형식은 Cargo와 `rustc`의 출력만 바꿉니다. `cargo run --message-format=json`으로 실행한 프로그램이 출력하는 내용이나 절차적 매크로의 출력은 JSON이 아니므로, Cargo Book은 `{`로 시작하는 줄만 JSON으로 파싱하는 방법을 제안합니다. 패키지에 관한 메시지의 `package_id`는 2.2절의 패키지 ID입니다(`build-finished`에는 없습니다).

### 2.5 사용자 정의 하위 명령

Cargo는 자신을 고치지 않고도 새 하위 명령(사용자 정의 하위 명령)을 더할 수 있게 되어 있습니다. `cargo hello`처럼 Cargo가 모르는 하위 명령을 받으면, Cargo는 실행 파일 `cargo-hello`를 찾아 실행합니다.

- **찾는 곳.** `PATH`의 디렉터리에서 찾습니다. 다만 Cargo home의 `bin/`(예: `~/.cargo/bin`)이 `PATH`에 없으면 그것을 맨 앞에 더해 먼저 봅니다. rustup으로 설치한 환경에서는 보통 `~/.cargo/bin`이 `PATH`에 있으므로 `PATH`에 적힌 순서가 그대로 우선순위입니다. 3절의 `PATH="$PWD/target/debug:$PATH" cargo hello`가 이미 설치된 같은 이름의 하위 명령보다 앞서는 것도 이 때문입니다. `cargo install`로 설치한 하위 명령([ch10](../ch10-publishing-semver/README.md) 2.8절)은 그 `bin/`에 들어가므로 바로 쓰입니다. 찾지 못하면 ``error: no such command: `hello` ``입니다.
- **넘기는 인자.** 첫째 인자는 늘 그렇듯 실행 파일 이름이고, **둘째 인자는 하위 명령의 이름**(`hello`)이며, 그 뒤에 사용자가 준 인자가 그대로 붙습니다. 그래서 하위 명령은 둘째 인자를 걷어 내고 나머지를 처리합니다. 실행 파일을 직접 실행하면 둘째 인자가 오지 않습니다.
- **도움말.** `cargo help hello`는 `cargo-hello hello --help`를 실행합니다. 셋째 인자가 `--help`이면 도움말을 출력하도록 만듭니다.
- **Cargo를 다시 부르기.** Cargo는 하위 명령을 실행할 때 환경 변수 `CARGO`에 자신의 경로를 넣어 줍니다([ch7](../ch7-configuration-env/README.md) 2.7절). 하위 명령은 이것으로 같은 cargo를 불러 `cargo metadata` 같은 정보를 얻습니다. `cargo` 크레이트를 라이브러리로 링크할 수도 있지만, 그 API는 안정적이지 않고 링크한 버전이 사용자의 cargo와 다를 수 있어 Cargo Book은 명령줄 인터페이스를 쓰라고 권합니다.

**별칭과 무엇이 다른가.** [ch7](../ch7-configuration-env/README.md) 2.4절의 별칭은 다른 cargo 명령으로 펼쳐질 뿐입니다. 하위 명령은 임의의 프로그램이라 파일을 읽고 쓰거나, Cargo를 여러 번 부르고 결과를 엮거나, 다른 도구를 실행할 수 있습니다. 내장 명령은 별칭으로 덮을 수 없었는데, 하위 명령도 마찬가지로 `cargo-build`라는 실행 파일을 만들어도 `cargo build`는 내장 명령이 실행됩니다. rustup 구성 요소인 `cargo fmt`·`cargo clippy`가 이 방식으로 실행되고, crates.io의 많은 도구(`cargo-hack`, `cargo-semver-checks` …)도 이 방식으로 배포됩니다.

`projects/cargo-hello`의 [`src/main.rs`](projects/cargo-hello/src/main.rs)는 이 규칙대로 둘째 인자를 걷어 내고, `--help`에 도움말을 출력하고, `CARGO`로 `cargo metadata --format-version 1`을 실행해 결과를 요약합니다.

### 2.6 장래 비호환 보고서

`rustc`에는 지금은 받아들이지만 앞으로의 버전에서 에러가 될 코드를 알리는 린트가 있습니다. 이런 경고를 장래 호환성 경고(future-compatibility warning)라고 부르고, `future_incompatible` 린트 그룹에 모여 있습니다. 예를 들어 표현식 자리에서 부르는 매크로가 `true;`처럼 세미콜론으로 끝나면 `semicolon_in_expressions_from_macros` 린트가 걸립니다. 내가 쓴 크레이트에서는 기본 수준이 deny라서 컴파일 에러가 납니다(핵심 줄만 옮겼습니다).

```text
error: trailing semicolon in macro used in expression position
  = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
  = note: `#[deny(semicolon_in_expressions_from_macros)]` (part of `#[deny(future_incompatible)]`) on by default
```

그런데 crates.io나 git에서 받은 의존성(경로 의존성 제외)은 린트가 꺼진 채(`--cap-lints allow`, `-vv`일 때는 `warn`) 컴파일되어 빌드를 막지 않고 경고도 보이지 않으므로, 의존성이 미래의 컴파일러에서 빌드되지 않게 될 것을 모르고 지나치기 쉽습니다. 그래서 Cargo는 **모든 의존성**에서 이런 경고를 모아, 빌드 끝에 짧은 알림을 냅니다. 예제 03이 연습용 레지스트리(디렉터리 소스, [ch2](../ch2-dependencies/README.md) 2.6절)의 의존성 `olddep`에 위 코드를 넣어 만든 알림입니다.

```text
warning: the following packages contain code that will be rejected by a future version of Rust: olddep v0.1.0
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
```

빌드는 성공(종료 상태 0)합니다. 전체 보고서는 `cargo report future-incompatibilities --id <번호>`나, 빌드를 `--future-incompat-report`와 함께 다시 실행해 봅니다. 보고서는 경고의 원문과 함께 할 수 있는 일 두 가지(의존성의 개발자에게 알리거나 고치기, `[patch]`로 고친 사본을 쓰기, [ch2](../ch2-dependencies/README.md) 2.7절)를 알려 줍니다. 보고서는 `target/.future-incompat-report.json`에 저장됩니다. 설정 파일의 `[future-incompat-report]`에서 `frequency = "never"`(환경 변수 `CARGO_FUTURE_INCOMPAT_REPORT_FREQUENCY=never`)로 알림을 끌 수 있습니다(기본은 `"always"`).

이 저장소의 실제 의존성에는 지금 그런 경고가 없어서, 저장소 루트에서 `cargo report future-incompatibilities`를 실행하면 `error: no reports are currently available`과 함께 종료 상태 101로 끝납니다.

## 3. 예제 안내

예제는 저장소 루트에서 실행합니다.

### 01_pkgid_specs — 모호한 이름과 정확한 지정

[`examples/01_pkgid_specs.rs`](examples/01_pkgid_specs.rs)는 저장소 루트의 워크스페이스에 대해 `cargo pkgid <지정>`을 여러 번 실행합니다. `cargo pkgid`는 지정이 가리키는 패키지의 완전한 ID를 출력하는 명령입니다. 빌드도 네트워크도 쓰지 않습니다.

```console
$ cargo run -q -p p3-ch11-commands-tools --example 01_pkgid_specs
$ cargo pkgid p3-ch11-commands-tools
  path+file://…/part3-cargo/ch11-commands-tools#p3-ch11-commands-tools@0.0.0
$ cargo pkgid rand
  error: specification `rand` is ambiguous
  help: re-run this command with one of the following specifications
    rand@0.8.8
    rand@0.10.4
$ cargo pkgid rand@0.8
  registry+https://github.com/rust-lang/crates.io-index#rand@0.8.8
$ cargo pkgid rand@0.10.4
  registry+https://github.com/rust-lang/crates.io-index#rand@0.10.4
$ cargo pkgid https://github.com/rust-lang/crates.io-index#rand@0.10.4
  registry+https://github.com/rust-lang/crates.io-index#rand@0.10.4
$ cargo pkgid rand@0.9
  error: package ID specification `rand@0.9` did not match any packages

  help: a package with a similar name exists: `rand`
```

첫 줄 다음의 `$ cargo pkgid …` 줄은 예제가 출력한 것입니다. 경로 패키지의 ID에는 디스크의 경로가 들어가고, 디렉터리 이름(`ch11-commands-tools`)이 패키지 이름(`p3-ch11-commands-tools`)과 다르므로 `#p3-ch11-commands-tools@0.0.0`처럼 이름도 붙습니다. `projects/cargo-hello`처럼 둘이 같으면 `…/cargo-hello#0.1.0`으로 버전만 붙습니다. 0.8.8이 어디서 들어왔는지는 연습 2에서 `cargo tree -i`로 찾습니다.

### 02_message_format — 빌드 메시지를 JSON으로 받기

[`examples/02_message_format.rs`](examples/02_message_format.rs)는 임시 디렉터리에 경고가 하나 나는 작은 패키지를 만들고, 자식 프로세스의 cargo로 `cargo check --message-format=json`을 실행해 줄마다 `reason`을 읽습니다. 끝나면 임시 디렉터리를 지웁니다.

```console
$ cargo run -q -p p3-ch11-commands-tools --example 02_message_format
compiler-message: warning: unused variable: `unused`
compiler-artifact: msgdemo (fresh: false)
build-finished: 성공 true
표준 에러의 줄 수: 2
```

경고는 표준 출력의 JSON(`compiler-message`)으로만 나오고, 표준 에러에는 `Checking`과 `Finished` 두 줄만 남았습니다. 이 예제는 짧게 쓰려고 JSON을 문자열 검색으로 읽습니다. 진단의 `level`·`message` 같은 키는 안쪽의 `children`에도 있어서 이런 방법으로는 엉뚱한 값을 꺼내기 쉽습니다. 문자열 값을 가진 `rendered`는 바깥 진단에만 있으므로(`children`의 `rendered`는 `null`) 예제는 `"rendered":"`를 찾습니다. 실제 도구는 JSON 파서로 읽습니다.

### 03_future_incompat — 장래 비호환 보고서 만들어 보기

[`examples/03_future_incompat.rs`](examples/03_future_incompat.rs)는 임시 디렉터리에 연습용 레지스트리와 그것을 쓰는 패키지 `app`을 만들고(2.6절), `app` 안에서 자식 프로세스의 cargo로 `cargo check`와 `cargo report future-incompatibilities`를 실행해 핵심 줄만 출력합니다. 설정 파일은 실행한 디렉터리에서 찾으므로([ch7](../ch7-configuration-env/README.md) 2.2절) `--manifest-path`가 아니라 작업 디렉터리를 `app`으로 정해 실행합니다. 네트워크는 쓰지 않고, 끝나면 임시 디렉터리를 지웁니다.

```console
$ cargo run -q -p p3-ch11-commands-tools --example 03_future_incompat
cargo check 종료 상태: 0
warning: the following packages contain code that will be rejected by a future version of Rust: olddep v0.1.0
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
cargo report future-incompatibilities 종료 상태: 0
The package `olddep v0.1.0` currently triggers the following future incompatibility lints:
> warning: trailing semicolon in macro used in expression position
>   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
```

내 크레이트였다면 에러였을 코드가 의존성에서는 린트가 꺼져(`--cap-lints allow`) 빌드가 성공하고 경고도 보이지 않으며, 그 대신 Cargo가 알림과 보고서로 알려 줍니다. 보고서의 `>`로 시작하는 줄이 의존성을 컴파일할 때 나온 경고의 원문입니다.

### projects/cargo-hello — 사용자 정의 하위 명령

`cargo metadata`를 읽어 워크스페이스를 요약하는 하위 명령입니다. 패키지 이름이 `cargo-hello`라서 바이너리 이름도 `cargo-hello`입니다.

| 파일 | 담은 것 |
| --- | --- |
| [`Cargo.toml`](projects/cargo-hello/Cargo.toml) | `serde_json` 의존성, 빈 `[workspace]` |
| [`src/main.rs`](projects/cargo-hello/src/main.rs) | 인자 처리(둘째 인자 걷어 내기, `--help`, `--manifest-path`, `--deps`), `CARGO`로 `cargo metadata` 실행, JSON 요약 |

프로젝트 디렉터리에서 실행합니다. 설치하지 않고 쓰려고 빌드한 디렉터리를 그 명령에만 `PATH`의 앞에 둡니다.

```console
$ cargo build -q --locked
$ ./target/debug/cargo-hello
받은 인자: []
CARGO 환경 변수: 없음
워크스페이스 루트: …/cargo-hello
target 디렉터리: …/target
멤버 1개
  cargo-hello 0.1.0  [bin:cargo-hello]
$ PATH="$PWD/target/debug:$PATH" cargo hello
받은 인자: ["hello"]
CARGO 환경 변수: 있음
워크스페이스 루트: …/cargo-hello
target 디렉터리: …/target
멤버 1개
  cargo-hello 0.1.0  [bin:cargo-hello]
```

직접 실행하면 인자가 비어 있고 `CARGO`도 없습니다. `cargo hello`로 부르면 둘째 인자로 `hello`가 오고 `CARGO`가 들어옵니다. 다른 프로젝트를 가리키고 의존성 그래프까지 보려면 `--manifest-path`와 `--deps`를 줍니다. [ch4](../ch4-workspaces/README.md)의 `bookshelf`입니다.

```console
$ PATH="$PWD/target/debug:$PATH" cargo hello --manifest-path ../../../ch4-workspaces/projects/bookshelf/Cargo.toml --deps
받은 인자: ["hello", "--manifest-path", "../../../ch4-workspaces/projects/bookshelf/Cargo.toml", "--deps"]
CARGO 환경 변수: 있음
워크스페이스 루트: …/bookshelf
target 디렉터리: …/target
멤버 3개
  shelf-cli 0.3.0  [bin:shelf]
  shelf-core 0.3.0  [lib:shelf_core]
  shelf-report 0.3.0  [lib:shelf_report]
그래프의 패키지 4개
  shelf-cli@0.3.0 → shelf-core@0.3.0, shelf-report@0.3.0, unicode-width@0.2.3
  shelf-core@0.3.0 → (없음)
  shelf-report@0.3.0 → shelf-core@0.3.0, unicode-width@0.2.3
```

| 명령 | 기대 결과 |
| --- | --- |
| `cargo fmt --check`, `cargo clippy --locked -- -D warnings` | 경고 없음 |
| `PATH="$PWD/target/debug:$PATH" cargo help hello` | `받은 인자: ["hello", "--help"]` 다음에 도움말 |
| `cargo hello`(`PATH`에 더하지 않고) | ``error: no such command: `hello` `` |
| `PATH="$PWD/target/debug:$PATH" cargo hello --deps` | 멤버 요약 뒤 `그래프의 패키지 …개`, `cargo-hello@0.1.0 → serde_json@1.0.154` |
| `cargo clean` | `target/` 삭제(실습을 마친 뒤) |

## 4. 자주 하는 실수

| 상황 | 메시지 | 해결 |
| --- | --- | --- |
| 같은 이름의 패키지가 그래프에 둘 이상인데 이름만 줌 | ``error: specification `rand` is ambiguous`` 뒤에 고를 수 있는 지정 목록 | `rand@0.10.4`처럼 버전을 붙입니다(2.2절). |
| 그래프에 없는 버전을 지정 | ``error: package ID specification `rand@0.9` did not match any packages`` | `cargo tree`나 예제 01로 실제 버전을 확인합니다. |
| `cargo metadata`에 `--format-version`을 주지 않음 | ``warning: please specify `--format-version` flag explicitly to avoid compatibility problems`` | `--format-version 1`을 명시합니다. |
| 하위 명령 실행 파일이 Cargo home의 `bin/`이나 `PATH`에 없음 | ``error: no such command: `hello` `` | `cargo install --path …`로 설치하거나 `PATH`에 더합니다. |
| 하위 명령이 둘째 인자 `hello`를 걷어 내지 않음 | (메시지 없음) `hello`를 사용자 인자로 잘못 처리 | `args[1]`이 하위 명령 이름이면 걷어 냅니다(2.5절). |
| 하위 명령이 `PATH`의 `cargo`를 부름 | (메시지 없음) `PATH`에 없는 cargo(절대 경로로 부른 툴체인의 cargo 등)로 불렸을 때 다른 cargo가 실행될 수 있음. rustup의 `cargo +1.98.1 hello`는 자식 프로세스에 툴체인을 물려주므로 같은 cargo가 됨 | 환경 변수 `CARGO`를 씁니다. |
| `--message-format=json` 출력을 문자열 검색으로 읽음 | (메시지 없음) 안쪽 객체의 같은 키를 잘못 꺼냄 | JSON 파서(`serde_json`, `cargo_metadata`)로 읽습니다(예제 02). |

## 5. 연습 문제

연습 1·3은 `projects/cargo-hello`를 저장소 밖에 복사해 시작합니다.

### 연습 1. 기능 목록 보여 주기

`cargo hello`가 멤버마다 정의된 기능(`[features]`)의 이름도 출력하게 고치세요. [ch5](../ch5-features/README.md)의 `projects/formats`에 대해 실행하면 `textfmt`의 기능이 보여야 합니다.

<details>
<summary>힌트</summary>

`packages[].features`는 기능 이름을 키로, 그 기능이 켜는 것의 목록을 값으로 하는 JSON 객체입니다. `pkg["features"].as_object()`로 키를 모아 출력합니다. `--manifest-path`에는 복사한 위치에서 `formats`의 `Cargo.toml`까지의 경로를 줍니다. `--no-deps`여도 멤버의 기능은 나옵니다.

</details>

### 연습 2. 0.8.8은 누가 끌어왔나

저장소 루트에서 `rand` 0.8.8이 어떤 경로로 워크스페이스에 들어왔는지 찾으세요. 2.2절의 패키지 ID 지정을 쓰세요.

<details>
<summary>힌트</summary>

`cargo tree -i rand@0.8.8`입니다(`-i`는 그 패키지에 의존하는 쪽으로 거꾸로 보여 줍니다). `rand v0.8.8 ← phf_generator ← phf_codegen ← (빌드 의존성) selectors ← scraper ← trpl ← (dev-dependencies) …`처럼 Part 0의 챕터가 쓰는 `trpl`에서 왔습니다. `-i rand`만 주면 2.2절의 모호성 에러가 납니다. `-e normal`을 주면 빌드·개발 의존성 경로가 빠져 트리 대신 `warning: nothing to print.`와 `--target all`을 써 보라는 안내가 나옵니다.

</details>

### 연습 3. 하위 명령이 받는 인자 확인하기

`PATH="$PWD/target/debug:$PATH" cargo hello -- --deps`와 `PATH="$PWD/target/debug:$PATH" cargo hello --deps --unknown`을 실행해 `받은 인자:` 줄을 비교하세요. Cargo는 하위 명령의 인자를 처리하나요?

<details>
<summary>힌트</summary>

Cargo는 하위 명령 이름 뒤의 인자를 처리하지 않고 그대로 넘깁니다. 첫째 명령은 `["hello", "--", "--deps"]`, 둘째는 `["hello", "--deps", "--unknown"]`을 받습니다. `--`나 모르는 옵션을 어떻게 다룰지는 하위 명령이 정합니다. `cargo-hello`는 `--deps`가 어디 있든 찾으므로 둘 다 의존성 그래프를 출력하고, `--unknown`은 무시합니다.

</details>

## 6. 참고 자료

- The Cargo Book: [3.10 Package ID Specifications](https://doc.rust-lang.org/cargo/reference/pkgid-spec.html), [3.11 External Tools](https://doc.rust-lang.org/cargo/reference/external-tools.html), [3.14 Future incompat report](https://doc.rust-lang.org/cargo/reference/future-incompat-report.html), [4. Cargo Commands](https://doc.rust-lang.org/cargo/commands/index.html), [cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html), [cargo pkgid](https://doc.rust-lang.org/cargo/commands/cargo-pkgid.html)
- crates.io의 [`cargo_metadata`](https://crates.io/crates/cargo_metadata) (Cargo Book 3.11절이 소개하는 크레이트)
