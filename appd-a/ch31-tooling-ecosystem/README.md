# ch31. 도구 생태계를 활용하기

Rust로 일하는 데 필요한 것은 컴파일러만이 아닙니다. 지금까지의 부록 챕터에서 Clippy, `cargo doc`, `cargo tree`, `criterion`을 실행했고, Miri, `cargo-deny`, `cargo-fuzz`, `cargo-expand`처럼 이 저장소에 설치하지 않아 소개만 한 도구도 여럿 만났습니다. 이 챕터는 그 도구들의 사용법을 다시 설명하지 않습니다. 대신 **어떤 문제에 어떤 도구를 꺼내는가, 도구를 들일 때 무엇을 따지는가, 들인 도구를 어디에 두는가**를 다룹니다. 책 전체에 나온 도구를 한 장의 도구 지도로 모으고, 외부 도구들이 공통으로 기대는 `cargo metadata`의 출력을 직접 읽어 "도구가 cargo의 출력을 읽어 판단하는" 작은 검사기를 만들어 봅니다.

- 원제: [Item 31: Take advantage of the tooling ecosystem](https://effective-rust.com/use-tools.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch31-tooling-ecosystem`
- 독립 프로젝트: `projects/meta_lint` (`cargo_metadata` 0.23.1, 2026-11-20에 `cargo add`로 받음)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), rustup 1.29.3

책의 5장(Tooling)에 속한 Item입니다. 다음 본편·부록 내용을 안다고 보고 진행합니다.

- [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md): `rustup`·`rustc`·`cargo`의 역할, `cargo check`와 `cargo build`, `rustup doc`
- [Part 1 ch10](../../part1-standard-lib/ch10-env-process-time/README.md) 2.8~2.10절: `Command`로 자식 프로세스를 실행하고 종료 상태와 출력을 받는 법(예제가 cargo를 다시 부름)
- [ch25](../ch25-dependency-graph/README.md): `cargo tree -d`/`-i`, `cargo update`, 쓰지 않는 의존성과 그래프 점검 도구
- [ch27](../ch27-document-public-interfaces/README.md), [ch29](../ch29-listen-to-clippy/README.md), [ch30](../ch30-beyond-unit-tests/README.md): `cargo doc`, Clippy, 테스트·벤치마크·퍼즈 테스트

CI에 도구를 올리는 구성은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서, Cargo 명령의 분류와 `cargo metadata` JSON 전체, 사용자 정의 하위 명령(`cargo-<이름>`) 만들기는 Part 3 `ch11-commands-tools`에서, `bindgen`은 [`ch35-bindgen`](../ch35-bindgen/README.md)에서 다룹니다. 이 챕터는 "무엇을 어디에 둘지"의 기준까지만 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 기본 도구 묶음과 Cargo 하위 명령(`check`, `fmt`, `clippy`, `doc`, `bench`, `update`, `tree`, `metadata`)이 각각 푸는 문제를 설명하고, 같은 도구의 고치는 모드와 검사 모드를 자리에 맞게 고를 수 있다.
2. `cargo metadata`의 출력 구조(`packages`, `workspace_members`, `resolve` 등)와 `--format-version`·`--no-deps`의 뜻을 설명하고, 사람이 읽는 출력 대신 이 출력을 읽는 검사 도구를 작성할 수 있다.
3. 도구 지도에서 문제에 맞는 도구와 그 도구를 다룬 챕터를 찾을 수 있다.
4. 외부 도구를 들이기 전에 유지 상태, nightly 필요 여부, 오탐, 속도, 결과의 재현성을 따지고, `cargo install`로 들일 때 버전과 잠금 파일을 고정할 수 있다.
5. 도구를 편집기, CI의 어느 작업, 손으로 가끔 중 어디에 둘지 도구의 성질을 들어 고를 수 있다.

## 2. 핵심 개념

2.1절에서 기본 도구 묶음을 정리하고, 2.2~2.6절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 지침 1·3·5의 뼈대는 책이 권하는 것이고(책이 굵게 강조한 문장은 지침 3과 5에 들어 있습니다), 지침 2·4와 각 지침에 더한 세부 기준은 Cargo 문서와 앞 챕터의 실행 결과를 근거로 더했습니다(2.7절). 이 Item은 책에도 코드가 없으므로, 이 챕터의 "코드"는 명령과 그 명령을 부르는 작은 프로그램입니다.

### 2.1 지도: 기본 도구 묶음

책은 어떤 Rust 개발 환경에나 있을 기본 도구로 다섯 가지를 듭니다. 이 컴퓨터에서 확인한 상태를 함께 적었습니다.

| 도구 | 하는 일 | 이 컴퓨터에서 |
| --- | --- | --- |
| `cargo` | 의존성 관리와 빌드, 하위 명령의 입구 | 1.100.0 |
| `rustup` | 툴체인과 구성 요소(component)의 설치·전환 | 1.29.3, 툴체인 1.100.0(활성)과 1.98.1 |
| 편집기 플러그인(rust-analyzer) | 정의로 이동, 자동 완성, 저장할 때 검사 | rustup 구성 요소로 받을 수 있으나 설치하지 않음 |
| Rust Playground | 브라우저에서 짧은 코드를 실행하고 링크로 공유 | 웹 서비스 |
| 표준 라이브러리 문서 | API와 안정화 버전 확인 | `rust-docs` 구성 요소로 설치됨, `rustup doc`으로 오프라인에서 열림 |

설치된 구성 요소는 설치를 일으키지 않는 조회 명령으로 봅니다. `rustup component list --installed`는 `cargo`, `clippy`, `rust-docs`, `rust-std`, `rustc`, `rustfmt` 여섯 개(각 이름 뒤에 `-aarch64-apple-darwin`이 붙음)를 보여 주었습니다. `rustup component list`(설치되지 않은 것까지)에는 `rust-analyzer`, `rust-src`, `llvm-tools` 등이 설치되지 않은 채로 나옵니다.

`cargo fmt`·`cargo clippy`는 Cargo에 들어 있는 명령이 아닙니다. Cargo는 모르는 하위 명령 `cargo foo`를 받으면 PATH에서 `cargo-foo` 실행 파일을 찾아 실행하고(Cargo Book, External tools 장의 Custom subcommands 절), rustup은 `~/.cargo/bin`에 `cargo-fmt`, `cargo-clippy`, `cargo-miri`, `rust-analyzer` 같은 **프록시**(활성 툴체인의 실제 도구로 넘겨 주는 얇은 실행 파일)를 둡니다. 그래서 `cargo --list`에는 `miri`도 보입니다. 하지만 1.100.0의 `rustup component list`에는 `miri`가 아예 없습니다. Miri는 nightly 툴체인의 구성 요소이기 때문입니다([ch16](../ch16-avoid-unsafe/README.md) 2.7절). **목록에 보이는 것과 쓸 수 있는 것은 다릅니다.** 쓸 수 있는지는 `rustup component list --installed`로 확인합니다.

조회할 때 조심할 점이 하나 있습니다. rustup 1.29는 설치되지 않은 툴체인을 이름으로 가리키기만 해도(`cargo +nightly …`, `rustup component list --toolchain nightly`) 그 툴체인을 내려받습니다. 이 저장소에서도 2026-11-19에 실제로 일어났습니다. 설치된 툴체인은 `rustup toolchain list`나 `rustup show`로 봅니다.

책은 각주에서, 중앙에서 툴체인을 관리하는 환경(그래서 `rustup`이 없음)이나 다른 빌드 시스템을 쓰는 환경(그래서 `cargo`가 없음)에서는 이 기본 묶음도 줄어들 수 있다고 덧붙입니다.

### 2.2 지침 1: Cargo에 있는 하위 명령부터 쓰고, CI에는 검사 모드를 둔다

**문제 상황.** 코드를 고치는 동안 컴파일 에러만 빨리 보고 싶고, 서식은 리뷰에서 다투고 싶지 않고, 서식이 어긋난 코드는 저장소에 들어오지 못하게 하고 싶습니다. 책은 이런 일에 `build`·`test`·`run` 말고도 Cargo가 이미 가진 하위 명령을 쓰라고 권합니다.

| 명령 | 푸는 문제 | 고치는가 | 이 저장소에서 다룬 곳 |
| --- | --- | --- | --- |
| `cargo check` | 기계어를 만들지 않고 컴파일 에러만 검사 | 검사만 | Part 0 ch1 |
| `cargo fmt` | 표준 서식으로 고침. `--check`는 검사만 하고 diff를 출력 | 둘 다 | Part 0 ch1, 예제 01 |
| `cargo clippy` | 린트. `--fix`는 기계적으로 고칠 수 있는 것만 고침 | 둘 다 | ch18, ch29 |
| `cargo doc` | 문서 생성과 rustdoc 린트 | 검사만 | ch27 |
| `cargo bench` | 벤치마크 실행(stable에서는 `criterion`과 함께) | 검사만 | ch30 |
| `cargo update` | `Cargo.lock`을 SemVer 범위 안에서 올림 | 바꿈 | ch21, ch25 |
| `cargo tree` | 의존성 그래프를 사람이 읽는 트리로 | 보기만 | ch25, ch26 |
| `cargo metadata` | 패키지와 그래프를 정해진 JSON으로 | 보기만 | 2.3절, 예제 02 |

`cargo check`가 무엇을 건너뛰는지는 산출물로 확인할 수 있습니다. 의존성 없는 작은 패키지에서 `cargo check`만 실행하면 `target/debug/`에 실행 파일이 생기지 않고, `--message-format json`으로 보면 산출물은 `.rmeta`(크레이트 메타데이터) 파일 하나이고 `"executable": null`이었습니다. 이어서 `cargo build`를 실행하자 목적 파일(`.o`)들과 실행 파일 `target/debug/<이름>`이 생겼습니다. 코드 생성과 링크를 건너뛰는 만큼 빠르므로, rust-analyzer도 기본으로 파일을 저장할 때마다 `cargo check`를 실행합니다(rust-analyzer 설정 문서: `checkOnSave` 기본 `true`, `check.command` 기본 `"check"`).

**비권장 코드.** CI에서 고치는 모드를 실행합니다. [`01_fmt_check_mode.rs`](examples/01_fmt_check_mode.rs)는 서식이 어긋난 임시 패키지에 `cargo fmt`를 실행합니다.

```text
1. 비권장: CI에서 고치는 모드(cargo fmt)를 실행
  종료 코드 0, 출력 0줄
  파일이 바뀜: true
```

아무 출력 없이 성공합니다. CI는 작업이 끝나면 고친 사본을 버리므로, 저장소에는 어긋난 코드가 그대로 남고 아무도 알지 못합니다.

**권장 코드.** CI에는 고치지 않고 검사만 하는 모드를 둡니다. 검사 모드는 문제를 출력하고 **종료 코드**로 실패를 알립니다. CI는 종료 코드만 보고 작업의 성패를 정하기 때문입니다.

```text
2. 권장: CI에서는 검사 모드(cargo fmt --check)
  종료 코드 1
    Diff in <임시>/src/lib.rs:1:
    -pub fn total(xs:&[u32])->u32{
    +pub fn total(xs: &[u32]) -> u32 {
```

고치는 모드는 사람이 있는 곳, 즉 로컬이나 편집기(저장할 때 서식 맞추기)에 둡니다. 다른 도구도 같은 짝을 가집니다. Clippy는 `cargo clippy -- -D warnings`나 `build.warnings = "deny"`가 검사 모드이고 `cargo clippy --fix`가 고치는 모드입니다([ch29](../ch29-listen-to-clippy/README.md) 2.2·2.4절). `cargo doc`은 `RUSTDOCFLAGS='-D warnings'`를 주어야 경고가 실패가 됩니다([ch27](../ch27-document-public-interfaces/README.md) 2.6절). 이 저장소가 챕터마다 돌리는 검증 명령(`cargo fmt --check`, `cargo clippy -- -D warnings`)도 모두 검사 모드입니다.

**책 이후에 생긴 내장 명령.** 책에 없는 하위 명령 두 가지도 알아 둡니다. `cargo info`(1.82)는 크레이트의 버전, `rust-version`, 라이선스, 저장소, 기능 목록을 보여 줍니다. 저장소 밖에서 `cargo info criterion --offline`을 실행하자 로컬 캐시에 있는 0.8.2의 정보(`rust-version: 1.86`, `license: Apache-2.0 OR MIT`, 기능 목록 등)가 나왔습니다. `--offline`이면 로컬 캐시에 있는 버전만 볼 수 있습니다. `cargo report future-incompatibilities`는 의존성이 받은 장래 호환성 경고를 다시 보여 주며, 보고서가 없으면 `error: no reports are currently available`(종료 코드 101)로 끝났습니다. 자세한 쓰임은 Part 3 `ch11-commands-tools`에서 다룹니다.

**따르지 않아도 되는 경우.** CI가 고친 결과를 커밋이나 PR로 되돌려 주도록 구성했다면 CI에서 고치는 모드를 돌릴 수도 있습니다. 다만 저장소에 쓰기 권한이 필요하고 그 구성은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)의 몫입니다. 또 실행 파일이 필요할 때(테스트, 실행, 배포)는 당연히 `cargo check`가 아니라 `cargo build`입니다.

### 2.3 지침 2: 도구는 사람이 읽는 출력이 아니라 `cargo metadata`를 읽는다

**문제 상황.** 워크스페이스의 멤버 중 게시할 수 있는 패키지는 게시 규칙을 지켜야 합니다. Cargo Book에 따르면 crates.io는 `license`나 `license-file` 중 하나를 요구하고(Manifest Format 장), crates.io 밖의 코드에 의존하는 패키지를 받지 않으므로 path 의존성에는 `version`도 적어야 합니다(Specifying Dependencies 장, dev-dependencies는 예외). [`projects/meta_lint/sample_ws`](projects/meta_lint/sample_ws/Cargo.toml)의 `ledger-report`는 둘 다 어겼지만 빌드와 clippy는 조용합니다. Cargo는 게시하려고 묶을 때가 되어서야 알려 줍니다. 저장소 밖의 사본에서 `cargo package -p ledger-report`를 실행하자 라이선스는 경고로만, path 의존성은 에러로 나왔습니다.

```text
warning: manifest has no license, license-file, documentation, homepage or repository
error: failed to verify manifest at `<경로>/ledger-report/Cargo.toml`

Caused by:
  all dependencies must have a version requirement specified when packaging.
  dependency `ledger-core` does not specify a version
```

이것을 변경마다 모든 멤버에 대해 검사하는 도구를 만들려고 합니다. 도구는 워크스페이스의 구조를 어디서 읽어야 할까요?

**비권장 코드.** 사람이 읽으라고 만든 출력이나 JSON의 글자를 긁습니다. [`02_read_metadata.rs`](examples/02_read_metadata.rs)의 4번 구역은 두 질문에 그렇게 답해 봅니다.

```text
  질문 A: 워크스페이스의 패키지는 몇 개인가?
    비권장: JSON 글자에서 "name": 을 센 수 = 9
    권장: packages 배열의 길이 = 3
  질문 B: ledger-cli가 직간접으로 의존하는 패키지는 몇 개인가?
      ledger-cli v0.1.0 (…/ledger-cli)
      ├── ledger-core v0.1.0 (…/ledger-core)
      └── ledger-report v0.1.0 (…/ledger-report)
          └── ledger-core v0.1.0 (…/ledger-core)
    비권장: 위 cargo tree 출력의 줄 수 - 1 = 3
    권장: resolve 그래프를 따라 센 수 = 2 (ledger-core, ledger-report)
```

`"name"` 키는 패키지만이 아니라 타깃과 의존성에도 있고, `cargo tree`는 여러 곳에서 쓰이는 패키지를 쓰인 곳마다 다시 그립니다. 사람에게는 편한 모양이 기계에게는 함정입니다. 산출물을 찾으려고 `target/` 디렉터리를 뒤지는 것도 같은 부류입니다. 1.100.0 릴리스 노트는 Cargo가 중간 산출물의 디렉터리 구조를 바꿨고, 그 구조는 안정된 인터페이스가 아니니 산출물 위치는 Cargo의 JSON 메시지로 찾으라고 적고 있습니다. 실제로 1.100.0에서 `cargo check`의 `.rmeta`는 `target/debug/build/<패키지>/<해시>/out/`에 있었습니다.

**권장 코드.** Cargo Book의 External tools 장은 도구 연동 수단으로 셋을 듭니다. `cargo metadata`(패키지 구조와 의존성), `--message-format json`(빌드 중의 진단과 산출물), 사용자 정의 하위 명령입니다. 그리고 `cargo metadata`의 형식은 안정적이고 버전이 붙어 있다고 밝힙니다. 예제 02의 2·3번 구역이 그 출력을 읽습니다(출력은 3절). 읽는 쪽이 알아야 할 것은 다음과 같습니다(`cargo help metadata`와 실행 결과로 확인).

| 키 | 내용 | `--no-deps`일 때 |
| --- | --- | --- |
| `packages` | 패키지마다 이름, 버전, ID, `dependencies`(매니페스트에 적은 요구 사항), `targets`, `features`, `publish`, `license`, `manifest_path` 등 | 워크스페이스 멤버만 |
| `workspace_members` / `workspace_default_members` | 멤버의 패키지 ID 목록 | 같음 |
| `resolve` | 해석된 의존성 그래프(`nodes`, `root`) | `null` |
| `target_directory` / `build_directory` | 산출물 디렉터리(절대 경로). `build_directory`는 문서에 불안정 표시 | 같음 |
| `version` | 출력 형식의 버전(지금은 1) | 같음 |
| `workspace_root` / `metadata` | 워크스페이스 루트, `[workspace.metadata]` 표 | 같음 |

- **`--format-version 1`을 늘 적습니다.** 빼면 ``warning: please specify `--format-version` flag explicitly to avoid compatibility problems``가 나옵니다(예제 02의 1번 구역). 같은 형식 버전 안에서도 새 필드와 열거형 같은 필드의 새 값은 호환되는 변경으로 칩니다(`cargo help metadata`의 Compatibility 절). 읽는 쪽은 모르는 필드를 무시하고 모르는 값에 대비해야 합니다.
- **`--no-deps`는 멤버의 매니페스트만 읽고 의존성을 해석하지 않습니다.** 매니페스트에 적힌 것(게시 규칙, 기능 목록)만 필요하면 이것으로 충분하고 빠릅니다. 버전 없이 적은 의존성의 `req`는 `"*"`로 나옵니다. 실제로 어떤 패키지가 이어지는지는 `--no-deps` 없이 실행해 `resolve.nodes`에서 봅니다.
- **패키지 ID는 열어 보지 않습니다.** 문서는 ID를 불투명한 식별자라고 하고, 실제로 sample_ws의 ID는 `path+file:///…/ledger-cli#0.1.0`처럼 이름이 빠진 모양이었습니다. ID로 `packages`에서 패키지를 찾아 이름을 읽습니다.
- **`resolve.nodes[].deps[].name`은 패키지 이름이 아닙니다.** 라이브러리 타깃 이름이라 `ledger_core`처럼 밑줄로 나옵니다(예제 02의 3번 구역). 패키지 이름은 `pkg`의 ID로 찾습니다.
- **경로는 절대 경로이고, `target_directory`는 설정을 따릅니다.** `CARGO_TARGET_DIR`을 주자 `target_directory`도 그 경로로 바뀌었습니다. 도구는 `target/`을 가정하지 말고 이 값을 읽습니다.
- **`metadata` 키에는 `[workspace.metadata]`(패키지는 `[package.metadata]`)의 내용이 그대로 실립니다.** ch25의 `cargo-udeps`·`cargo-machete`가 예외 목록을 `[package.metadata.…]`에서 읽는 것처럼, 도구는 자기 설정을 여기서 읽습니다(연습 1).

Rust로 도구를 만든다면 JSON을 직접 다루지 않고 `cargo_metadata` 크레이트로 타입을 통해 읽습니다. `cargo help metadata`와 External tools 장도 이 크레이트를 가리킵니다. [`projects/meta_lint`](projects/meta_lint/src/main.rs)의 핵심입니다.

```rust
fn is_publishable(p: &Package) -> bool {
    // None이면 제한 없음, Some(빈 목록)이면 publish = false
    !matches!(&p.publish, Some(registries) if registries.is_empty())
}
// ...
let path_without_version = dep.path.is_some() && dep.req == VersionReq::STAR;
if dep.kind != DependencyKind::Development && path_without_version { /* 위반 */ }
```

JSON의 `null`과 `[]`의 차이, `"*"`라는 글자, `"dev"`라는 문자열이 타입(`Option<Vec<String>>`, `semver::VersionReq`, `DependencyKind`)으로 바뀌어 있으므로 철자를 틀릴 곳이 줄어듭니다. 0.23.1 소스를 보면 `MetadataCommand::exec`는 `$CARGO`(없으면 PATH의 `cargo`)로 `cargo metadata --format-version 1`을 실행하고, `no_deps()`를 부르면 `--no-deps`를 붙입니다. `cargo fmt`도 이렇게 만들어진 도구입니다. rustfmt 저장소의 `src/cargo-fmt/main.rs`(2026-11-20 기준)는 `cargo_metadata::MetadataCommand`에 `no_deps()`를 걸어 포맷할 타깃을 찾습니다. 책은 같은 방식의 도구로 `cargo-udeps`와 `cargo-deny`를 듭니다.

**따르지 않아도 되는 경우.** 사람이 한 번 보고 끝낼 때는 `cargo tree`가 낫습니다([ch25](../ch25-dependency-graph/README.md)). 기계용 출력이 담지 않는 정보가 필요한 도구도 있습니다. `cargo-machete`는 컴파일하지 않고 소스를 검색하는 방식을 일부러 고른 도구입니다. 셸 한 줄짜리로 긁는 것도 나만 쓰는 일회성 확인이라면 괜찮지만, CI나 다른 사람이 쓰는 도구로 옮기는 순간 `cargo metadata`로 바꿉니다.

### 2.4 지침 3: 문제에서 출발해 도구를 찾는다

**문제 상황.** 책은 **있는 도구를 찾아보라**고 굵게 강조합니다. 어떤 도구 목록이든 주관적이고, 금방 낡고, 빠진 것이 있기 마련이라는 이유에서입니다. `cargo-<무엇>`이라는 이름의 도구 가운데에는 버려졌거나 맞지 않는 것도 있지만 딱 맞는 것도 있다고 덧붙입니다.

**비권장 습관.** 도구 이름부터 외우거나, 문제가 생겼을 때 손으로 같은 확인을 반복합니다. 매크로가 무엇으로 펼쳐지는지 머릿속으로 펼쳐 보거나, 중복 버전을 `Cargo.lock`에서 눈으로 찾는 식입니다.

**권장 습관.** 문제에서 출발해 아래 지도에서 도구와 그 도구를 다룬 챕터를 찾습니다. "이 저장소에서" 열은 이 저장소에서 실제로 실행했는지를 정직하게 적은 것입니다. 실행하지 않은 도구의 설명은 각 챕터가 도구의 문서에서 확인한 내용입니다. "책" 열은 Item 31이 그 도구를 이름으로 드는지이고, —인 행은 책에 없는 내용입니다. 버전과 지원 툴체인은 2026-11-20 기준입니다.

| 도구 | 푸는 문제 | 다룬 곳 | 종류 | stable | 이 저장소에서 | 책 |
| --- | --- | --- | --- | --- | --- | --- |
| `rustup` | 툴체인·구성 요소 설치와 전환 | Part 0 ch1, 2.1절 | 설치 도구 | — | 조회만 실행 | O |
| `cargo` | 빌드, 의존성, 하위 명령의 입구 | 모든 챕터 | 툴체인 기본 | O | 실행 | O |
| rust-analyzer | 편집기의 이동·자동 완성·저장할 때 검사 | Part 2 ch6, 2.6절 | rustup 구성 요소 | O | 실행 안 함(미설치) | O |
| Rust Playground | 짧은 코드 실행과 공유 | — | 웹 | — | 쓰지 않음 | O |
| 표준 라이브러리 문서 | API와 안정화 버전 확인 | 모든 챕터 | rustup 구성 요소 | O | 로컬 문서로 확인 | O |
| `cargo check` | 기계어 없이 컴파일 검사 | Part 0 ch1, 2.2절 | 내장 | O | 실행 | O |
| `cargo fmt` | 표준 서식 | Part 0 ch1, 예제 01 | rustup 구성 요소 | O | 실행 | O |
| `cargo clippy` | 린트, `--fix` | ch18, ch29 | rustup 구성 요소 | O | 실행 | O |
| `cargo doc` | 문서 생성, rustdoc 린트 | ch27 | 내장 | O | 실행 | O |
| `cargo test` | 단위·통합·문서 테스트 | Part 0 ch11, ch30 | 내장 | O | 실행 | O |
| `cargo bench` | 벤치마크 실행 | ch30 | 내장 | O(`criterion`과 함께) | 실행 | O |
| `cargo update` | SemVer 범위 안에서 올림 | ch21, ch25 | 내장 | O | 실행 | O |
| `cargo tree` | 그래프 보기(`-d`, `-i`, `-e`, `-f`) | ch25, ch26, 2.5절 | 내장 | O | 실행 | O |
| `cargo metadata` | 패키지·그래프를 정해진 JSON으로 | 2.3절, 예제 02 | 내장 | O | 실행 | O |
| `cargo fix` | 컴파일러 제안 자동 적용 | Part 2 ch6, ch29 | 내장 | O | 실행(저장소 밖 사본) | — |
| `cargo package --list` | 게시될 파일 확인 | ch30 | 내장 | O | 실행 | — |
| `cargo::unused_dependencies` | 쓰지 않는 의존성(빌드 중 경고) | ch25 | 내장 린트(1.100) | O | 실행 | — |
| `cargo info` | 버전·MSRV·기능·저장소 보기 | 2.2절 | 내장(1.82) | O | 실행(로컬 캐시) | — |
| `cargo report` | 장래 호환성 경고 다시 보기 | 2.2절 | 내장 | O | 실행(보고서 없음) | — |
| Miri | 미정의 동작 검출 | ch16 | rustup 구성 요소 | X(nightly) | 실행 안 함 | O |
| `cargo-semver-checks` | 공개 API의 SemVer 위반 | ch21 | `cargo install` | O(그때의 stable·beta) | 실행 안 함 | O |
| `cargo-deny` | 중복·금지 크레이트·라이선스·출처·권고 | ch25 | `cargo install` | O | 설정 예시만 | O |
| `cargo-udeps` | 쓰지 않는 의존성(컴파일해서) | ch25 | `cargo install` | X(실행은 nightly) | 실행 안 함 | O |
| `cargo-machete` | 쓰지 않는 의존성(소스 검색) | ch25 | `cargo install` | O | 실행 안 함 | — |
| `cargo-audit` | RustSec 권고 대조 | ch25 | `cargo install` | O | 실행 안 함 | — |
| `cargo-hack` | 기능 조합 검사 | ch26 | `cargo install` | O | 실행 안 함 | — |
| `cargo-expand` | 매크로를 펼친 코드 보기 | Part 0 ch19, ch28 | `cargo install` | 확인 못 함 | 실행 안 함 | O |
| `cargo-fuzz` | 퍼즈 테스트 | ch30 | `cargo install` | X(nightly) | 실행 안 함 | O |
| `cargo-tarpaulin` | 코드 커버리지 | 이 절 | `cargo install` | O | 실행 안 함 | O |
| 새니타이저 | 메모리 오류 검출 | ch30, Part 11 `ch2-compiler-flags` | rustc `-Z` 플래그 | X(nightly) | 실행 안 함 | — |
| `bindgen` | C 헤더에서 FFI 선언 생성 | [`ch35-bindgen`](../ch35-bindgen/README.md) | CLI·빌드 의존성 | 확인 못 함 | 실행 안 함 | O |
| `criterion` | 통계 벤치마크 | ch30 | 크레이트 0.8.2 | O | 실행 | O |
| `proptest`, Iai | 성질 기반 테스트, 명령어 수 세기 | ch30(이름만) | 크레이트 | 확인 못 함 | 쓰지 않음 | — |
| `syn` | 구문 트리, 절차적 매크로의 바탕 | ch28 | 크레이트 3.0.10 | O | 실행 | O |
| `cargo_metadata` | `cargo metadata`를 타입으로 | 2.3절, `projects/meta_lint` | 크레이트 0.23.1 | O | 실행 | O |
| Dependabot | 의존성 갱신 PR 자동 생성 | ch21, ch25 | GitHub 서비스 | — | 쓰지 않음 | O |
| Compiler Explorer | 컴파일된 기계어 보기(Godbolt) | ch30 | 웹 | — | 쓰지 않음 | O |

몇 행에 근거를 덧붙입니다. `cargo-semver-checks`는 README에 따르면 불안정한 rustdoc JSON에 기대므로 릴리스마다 그때의 stable과 beta를 지원하고 nightly는 "최선을 다하는" 수준입니다. 툴체인을 올리면 이 도구도 함께 올려야 한다는 뜻입니다. `cargo-deny`·`cargo-machete`·`cargo-audit`·`cargo-hack`·`cargo-tarpaulin`의 README에는 nightly 요구가 없고, `cargo-deny`는 최소 stable 버전 표시(1.88), `cargo-audit`은 1.74 이상, `cargo-hack`은 설치에 rustc 1.85 이상을 적어 두었습니다. `cargo-expand`의 README는 이 도구가 `cargo rustc --profile=check -- -Zunpretty=expanded`를 감싼 것이라고 설명하는데, stable rustc에 `-Zunpretty=expanded`를 직접 주면 ``error: the option `Z` is only accepted on the nightly compiler``로 거절됩니다. 도구가 stable에서 이것을 어떻게 처리하는지는 README에 없어 확인하지 못했습니다. `cargo-tarpaulin`은 책이 이름을 들지만 이 저장소의 어느 챕터도 다루지 않은 도구입니다. README에 따르면 Linux에서는 기본 엔진(ptrace)이 x86_64에서만 동작하고 macOS와 Windows에서는 LLVM 엔진이 기본입니다. 커버리지 계측 자체는 Part 5 `ch6-optimization-instrumentation`에서 다룹니다.

책은 더 높은 보증이 필요한 코드라면 형식 검증 도구도 고려하라고 권하며, 도구 이름 대신 Rust 자동 검증 도구를 정리한 글로 연결합니다. 이 챕터도 이름을 들지 않습니다.

**따르지 않아도 되는 경우.** 도구는 공짜가 아닙니다. 설치, 버전 관리, 설정 파일, CI 시간이 따라옵니다. 일 년에 한 번 하는 확인이라면 손으로 해도 됩니다. 그 판단이 다음 지침입니다.

### 2.5 지침 4: 들이기 전에 따지고, 들일 때는 버전과 잠금 파일을 고정한다

**문제 상황.** 쓰지 않는 의존성을 찾고 싶습니다. 지도에는 후보가 셋 있습니다. 어느 것을 들일까요?

| 따질 것 | `cargo::unused_dependencies` | `cargo-udeps` | `cargo-machete` |
| --- | --- | --- | --- |
| 설치 | 필요 없음(Cargo 1.100 내장) | `cargo install` | `cargo install` |
| 툴체인 | stable | 실행에 nightly | stable |
| 판단 방식 | 빌드하며 판단, 명령에 따라 결과가 다름 | 컴파일해서 판단 | 컴파일하지 않고 소스 검색 |
| 오탐 | dev-dependencies는 검사하지 않음 등 한계(ch25) | 예외를 `[package.metadata.cargo-udeps.ignore]`에 | README가 스스로 "빠르지만 부정확"하다고 밝힘 |
| CI 친화성 | 빌드 경고(수준을 `deny`로 올릴 수 있음) | nightly 작업이 필요 | 찾으면 종료 코드 1 |

모든 칸은 [ch25](../ch25-dependency-graph/README.md) 2.5절과 각 도구의 README에서 왔습니다. 이 비교에서 따진 항목을 일반화하면 다음과 같습니다.

1. **유지 상태.** 최근 릴리스와 저장소의 활동, 지원하는 툴체인을 봅니다. 저장소 주소는 크레이트라면 `cargo tree --depth 1 -f "{p} {r}"`(`{r}`은 repository, ch25가 쓴 `{l}`과 같은 자리 표시)로, 아직 쓰지 않는 크레이트라면 `cargo info`로 봅니다. `cargo tree -p er-ch4-idiomatic-errors --depth 1 -e dev -f "{p} {r}"`은 `anyhow v1.0.104 https://github.com/dtolnay/anyhow`와 `thiserror v2.0.21 https://github.com/dtolnay/thiserror`를 보여 주었습니다(`-e dev`는 ch4가 두 크레이트를 dev-dependency로 두었기 때문).
2. **nightly 필요 여부.** stable만 쓰는 팀이라면 nightly 도구는 별도 툴체인과 별도 CI 작업이 필요합니다.
3. **오탐.** 오탐이 잦으면 예외 표시가 쌓이고, 결국 경고를 아무도 읽지 않게 됩니다. 예외를 이유와 함께 적을 수단이 있는지 봅니다(ch29의 `#[expect]`, ch25의 `skip`·`ignore`).
4. **속도.** 컴파일이 필요한 도구는 빌드만큼 걸립니다. 저장할 때마다 돌릴지, 변경마다 CI에서 돌릴지가 여기서 갈립니다(2.6절).
5. **결과의 재현성.** 코드가 그대로여도 결과가 바뀌는 도구가 있습니다. 권고 데이터베이스가 갱신되는 `cargo-audit`, 측정값이 흔들리는 벤치마크(ch30)가 그렇습니다.

**비권장 코드.** 버전을 정하지 않고 설치합니다.

```sh
cargo install cargo-deny      # 그날의 최신 버전, 그날 다시 고른 의존성
```

**권장 코드.** 도구의 버전을 정하고, 도구가 게시될 때 시험한 의존성으로 빌드합니다.

```sh
cargo install cargo-deny@<버전>             # 또는 --version <버전>
cargo install cargo-deny@<버전> --locked    # 1.98.1 이하의 cargo에서는 --locked가 필요
```

잠금 파일 쪽은 1.100.0에서 바뀌었습니다. 1.100.0 릴리스 노트(로컬 문서의 `releases.md`)는 `cargo install`이 이제 `.crate`에 든 lockfile을 기본으로 쓰고, `cargo install`의 `--locked`는 아무 일도 하지 않게(no-op) 되었다고 적고 있습니다. 두 툴체인의 `cargo help install`(Dealing with the Lockfile 절)을 비교하면 차이가 분명합니다.

- 1.98.1: 기본으로 패키지에 든 `Cargo.lock`을 **무시**하고 의존성을 다시 고릅니다. `--locked`를 주어야 그 파일을 씁니다.
- 1.100.0: 기본으로 패키지에 든 `Cargo.lock`을 **씁니다**. `--locked`는 지금은 해석을 바꾸지 않고, `Cargo.lock`이 없는 패키지에 주면 경고만 냅니다.

crates.io에서 받은 패키지에는 실제로 `Cargo.lock`이 들어 있습니다. 이 챕터가 받은 `cargo_metadata-0.23.1`의 압축을 푼 로컬 레지스트리 캐시에도 `Cargo.lock`이 있었습니다. `cargo-deny`, `cargo-semver-checks`, `cargo-hack`, `cargo-tarpaulin`의 README는 모두 설치 명령에 `--locked`를 붙여 두었는데, 1.100.0부터는 이것이 기본 동작이 된 셈입니다. 그래도 CI 이미지의 cargo가 1.100.0보다 오래되었을 수 있으니 `--locked`는 붙여 두는 편이 안전합니다. 1.100.0 `cargo install --help`의 짧은 설명과 공통 옵션 설명은 여전히 `--locked`를 "Cargo.lock이 바뀌지 않음을 단언"하는 옵션으로 소개하므로, 동작은 위의 Dealing with the Lockfile 절을 기준으로 봅니다. 이 저장소에서는 `cargo install`을 실행하지 않았고(`cargo install --list`는 빈 출력), 위 내용은 문서로 확인한 것입니다.

버전을 고정하는 이유는 ch29 2.4절이 보여 준 것과 같습니다. 1.98.1의 Clippy는 1.100.0에서 생긴 린트를 몰라 `unknown lint`로 경고했습니다. 개발자와 CI가 서로 다른 도구 버전을 쓰면 "경고 0개"라는 기준부터 흔들립니다.

**따르지 않아도 되는 경우.** 내 컴퓨터에서 한 번 써 보는 것이라면 최신 버전으로 충분합니다. 여러 README가 미리 빌드한 실행 파일을 받는 `cargo binstall`을 대안으로 적어 두었지만, 이 저장소에서는 확인하지 않았습니다.

### 2.6 지침 5: 도구마다 둘 곳을 정한다

**문제 상황.** 책은 둘 곳에 대해 두 가지를 굵게 권합니다. **일회성이 아닌 쓸모가 있는 도구는 CI에 통합하고**(Item 32), 빠르고 오탐이 없는 도구라면 **편집기나 IDE에도 통합하라**는 것입니다. 그런데 도구를 CI에 넣는다고 할 때 "어느 작업에"까지 정해야 합니다.

**비권장 코드.** 성질을 보지 않고 모두 변경마다 도는 CI 작업에 넣습니다. [`03_tool_placement.rs`](examples/03_tool_placement.rs)는 도구 19개를 성질과 함께 데이터로 적고 그 결과를 셉니다.

```text
1. 비권장: 모두 변경마다 도는 CI 작업에 넣음
  stable CI에서 돌지 않음(nightly 필요): Miri, cargo-udeps, cargo-fuzz
  코드가 그대로여도 결과가 바뀜: cargo-audit, cargo bench (criterion)
  스스로 끝나지 않음: cargo-fuzz
  판정이 없거나 게시 때만 의미 있음: cargo-semver-checks, cargo package --list, cargo-expand, cargo tree -i, Compiler Explorer
```

nightly 도구는 stable 작업에서 실패하고, 결과가 바뀌는 도구는 관계없는 변경을 실패시키고, 끝나지 않는 퍼저는 제한 시간에 걸립니다. 판정이 없는 도구는 돌려도 읽는 사람이 없습니다.

**권장 코드.** 성질을 차례로 따져 둘 곳을 고릅니다. 예제의 `place` 함수가 그 순서입니다.

```rust
fn place(t: &Tool) -> Place {
    match t.purpose {
        Purpose::Investigate => return Place::ByHand,
        Purpose::Release => return Place::BeforeRelease,
        Purpose::Check => {}
    }
    if !t.ends { Place::TimeBoxed }
    else if !t.repeatable { Place::Scheduled }
    else if t.nightly { Place::NightlyJob }
    else if t.fast && !t.false_positives { Place::EditorAndCi }
    else { Place::EveryChange }
}
```

예제에 적은 성질과 그 근거입니다. 표에 없는 도구는 기본값(stable, 느림, 오탐 없음, 같은 결과, 스스로 끝남)입니다.

| 도구 | 기본값과 다른 성질 | 근거 |
| --- | --- | --- |
| `cargo fmt`, `cargo check`, `cargo clippy` | 빠름 | rust-analyzer가 저장할 때 돌리는 명령이 `cargo check`(설정으로 `clippy`) |
| clippy pedantic | 오탐 있음 | Clippy 문서, ch29의 `doc_markdown` 오탐 |
| `cargo-machete` | 빠름, 오탐 있음 | README "fast (yet imprecise)" |
| Miri, `cargo-udeps` | nightly | ch16, ch25 |
| `cargo-audit` | 결과가 바뀜 | 권고가 새로 붙음(ch25 2.5절) |
| `cargo bench`(criterion) | 결과가 바뀜 | CI 가상 머신의 측정값을 믿지 말라는 criterion FAQ(ch30) |
| `cargo-fuzz` | nightly, 끝나지 않음 | ch30 2.7절 |
| `cargo-semver-checks`, `cargo package --list` | 게시용 | ch21, ch30 |
| `cargo-expand`, `cargo tree -i`, Compiler Explorer | 판정이 없음 | ch28, ch25, ch30 |

결과(3절의 03 출력)는 이렇게 나뉩니다. 편집기와 CI 양쪽에는 `cargo fmt`·`cargo check`·`cargo clippy` 셋만 놓이고, 테스트·문서·pedantic·`cargo-deny`·`cargo-machete`·`cargo-hack`은 변경마다 도는 CI 작업에, Miri와 `cargo-udeps`는 nightly 작업에, `cargo-audit`과 벤치마크는 정기 예약 작업에, `cargo-fuzz`는 시간을 정한 별도 실행에 놓입니다.

편집기에는 빠르고 오탐 없는 도구만 둡니다. 저장할 때마다 틀린 경고가 뜨면 사람은 곧 경고를 무시하는 법을 익힙니다. 오탐이 있는 도구는 CI에 두되 예외를 이유와 함께 적게 합니다. CI에 둔 도구는 버전을 고정합니다(2.5절). 각 자리를 실제 워크플로로 만드는 일은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 합니다.

**따르지 않아도 되는 경우.** 성질은 프로젝트에 따라 달라집니다. 테스트가 몇 초면 끝나는 작은 크레이트라면 `cargo test`도 편집기에서 돌려도 됩니다. 직접 쓴 `unsafe`가 없는 크레이트는 Miri가 찾을 것이 거의 없으므로 nightly 작업을 따로 둘 이유도 작습니다(ch16). 혼자 하는 실험 프로젝트라면 편집기와 손만으로 충분합니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0과 cargo 1.100.0에서 확인한 차이는 다음과 같습니다.

- **`cargo install`이 패키지의 `Cargo.lock`을 기본으로 씁니다(1.100.0).** 책은 설치 방법을 다루지 않지만, 도구를 들이는 방식이 바뀐 것이라 2.5절에 적었습니다. 근거는 1.100.0 릴리스 노트와 두 툴체인의 `cargo help install`입니다.
- **쓰지 않는 의존성을 Cargo가 직접 경고합니다(1.100.0).** 책은 nightly가 필요한 `cargo-udeps`를 소개합니다. 동작과 한계는 ch25가 실행해 확인했습니다.
- **책 이후의 내장 명령과 변경.** `cargo info`(1.82)와 `cargo report`는 책에 없습니다. 1.100.0에서 중간 산출물의 디렉터리 구조가 바뀌었고, 릴리스 노트는 그 구조가 안정된 인터페이스가 아니라고 밝힙니다. `cargo metadata`의 출력에는 `build_directory` 키가 있고 문서는 이것을 불안정으로 표시합니다.
- **달라지지 않은 점.** Miri는 여전히 nightly 전용 구성 요소이고(1.100.0의 `rustup component list`에 없음), 책이 든 `cargo metadata`의 형식 버전은 여전히 1뿐입니다.
- **책에 없는 지침.** 다음은 책의 Item 31이 명시적으로 권하지 않는 것입니다(2026-11-20에 온라인판에서 확인). 책은 기본 도구 묶음, Cargo 하위 명령 목록, `cargo metadata`와 `cargo_metadata`·`cargo-udeps`·`cargo-deny`, `syn`과 `cargo-expand`, `cargo-tarpaulin`, 도구를 찾아보라는 권고, 형식 검증, CI와 편집기 통합, 기억할 도구 목록을 다룹니다.
  - 지침 1의 "CI에는 검사 모드, 사람 곁에는 고치는 모드"와 종료 코드: `cargo fmt --help`, 예제 01의 실행 결과, ch27·ch29의 `-D warnings`와 `build.warnings`가 근거입니다.
  - 지침 2 전체(사람용 출력 대신 `cargo metadata`, `--format-version`, `--no-deps`와 `resolve`, ID를 열어 보지 않기, `deps[].name`, `target_directory`): Cargo Book의 External tools 장, `cargo help metadata`, 1.100.0 릴리스 노트, 예제 02와 `projects/meta_lint`의 실행 결과가 근거입니다.
  - 지침 4의 따질 항목과 `cargo install`의 버전·잠금 파일 고정: 책은 버려졌거나 맞지 않는 도구가 있다고만 말합니다. 각 도구의 README, `cargo help install`(1.98.1과 1.100.0), ch25·ch29의 실행 결과가 근거입니다.
  - 지침 5에서 CI를 변경마다·nightly·정기·시간을 정해·게시 전으로 나눈 기준(nightly, 결과의 재현성, 끝나는지): ch16·ch25·ch30의 내용과 rust-analyzer 설정 문서가 근거입니다.
  - 지도에 넣은 `cargo fix`, `cargo package --list`, `cargo::unused_dependencies`, `cargo info`, `cargo report`, `cargo-machete`, `cargo-audit`, `cargo-hack`, 새니타이저, `proptest`, Iai는 책의 Item 31에 없는 도구입니다.

### 2.8 기억할 것

- Cargo에 있는 하위 명령부터 씁니다. 사람 곁(편집기, 로컬)에는 고치는 모드를, CI에는 종료 코드로 실패를 알리는 검사 모드를 둡니다.
- 도구는 사람이 읽는 출력이나 `target/`의 구조가 아니라 `cargo metadata --format-version 1`과 `--message-format json`을 읽습니다. Rust라면 `cargo_metadata`로 타입을 통해 읽습니다.
- 문제에서 출발해 도구를 찾고, 들이기 전에 유지 상태·nightly·오탐·속도·재현성을 따집니다.
- `cargo install`은 버전을 정해서 실행합니다. 1.100.0부터는 패키지의 `Cargo.lock`을 기본으로 쓰고, 그 전 cargo에서는 `--locked`가 필요합니다.
- 빠르고 오탐 없는 도구는 편집기와 CI에, nightly 도구는 별도 작업에, 결과가 바뀌는 도구는 정기 작업에, 끝나지 않는 도구는 시간을 정해 둡니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행합니다. 세 예제 모두 종료 코드 0으로 끝나고, 외부 크레이트 없이 std만 씁니다. 01과 02는 `cargo run`이 넣어 주는 `CARGO` 환경 변수로 cargo를 다시 부르되, 빌드를 일으키지 않는 명령(`cargo fmt`, `cargo metadata`, `cargo tree`)만 씁니다. 출력의 절대 경로는 예제가 `<임시>`나 `…`로 바꿔 출력합니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_fmt_check_mode.rs`](examples/01_fmt_check_mode.rs) | 1 | CI에서 `cargo fmt` | CI에서 `cargo fmt --check` | 2.2 |
| [`02_read_metadata.rs`](examples/02_read_metadata.rs) | 2 | JSON 글자, `cargo tree` 줄 수 | JSON 구조와 `resolve` 그래프 | 2.3 |
| [`03_tool_placement.rs`](examples/03_tool_placement.rs) | 5 | 모두 변경마다 CI | 성질을 보고 둘 곳 고르기 | 2.6 |
| [`projects/meta_lint`](projects/meta_lint/src/main.rs) | 2 | (sample_ws의 `ledger-report`가 규칙 위반) | `cargo_metadata`로 읽는 검사기 | 2.3 |

### 01_fmt_check_mode — 고치는 모드와 검사 모드

```sh
cargo run -p er-ch31-tooling-ecosystem --example 01_fmt_check_mode
```

`std::env::temp_dir()` 아래에 서식이 어긋난 패키지를 만들어 실행하고 지웁니다.

```text
1. 비권장: CI에서 고치는 모드(cargo fmt)를 실행
  종료 코드 0, 출력 0줄
  파일이 바뀜: true
  → CI의 사본만 고쳐지고 작업은 성공으로 끝남
2. 권장: CI에서는 검사 모드(cargo fmt --check)
  종료 코드 1
    Diff in <임시>/src/lib.rs:1:
    -pub fn total(xs:&[u32])->u32{
    +pub fn total(xs: &[u32]) -> u32 {
         xs.iter().sum()
     }

    Diff in <임시>/src/lib.rs:5:
    -pub fn  label( n:u32 )->String { format!("{n}개") }
    +pub fn label(n: u32) -> String {
    +    format!("{n}개")
    +}

  파일이 바뀜: false
3. 로컬(또는 편집기)에서 고친 뒤 다시 검사
  cargo fmt 종료 코드 0, cargo fmt --check 종료 코드 0
```

### 02_read_metadata — `cargo metadata`의 구조와 긁어 읽기의 함정

```sh
cargo run -p er-ch31-tooling-ecosystem --example 02_read_metadata
```

`projects/meta_lint/sample_ws`에 `cargo metadata`와 `cargo tree`를 `--offline`(과 `--locked`)으로 실행합니다. JSON은 예제 안의 작은 파서(180줄 남짓)로 읽습니다. std만 쓰려고 둔 것이고, 실제 도구라면 이 부분이 `cargo_metadata`로 바뀝니다. 4번 구역은 2.3절에 옮겼습니다.

```text
대상: projects/meta_lint/sample_ws/Cargo.toml

1. --format-version 없이 실행하면
  warning: please specify `--format-version` flag explicitly to avoid compatibility problems

2. cargo metadata --format-version 1 --no-deps
  최상위 키: packages, workspace_members, workspace_default_members, resolve, target_directory, build_directory, version, workspace_root, metadata
  version = 1, resolve = null
  workspace_members 3개, workspace_default_members 3개
  packages 3개 (이름순):
    ledger-cli 0.1.0  ledger-cli/Cargo.toml  publish: 막힘, license: null
      의존성 ledger-core  req "^0.1.0"  (normal, path)
      의존성 ledger-report  req "*"  (normal, path)
    ledger-core 0.1.0  ledger-core/Cargo.toml  publish: 제한 없음, license: "MIT OR Apache-2.0"
    ledger-report 0.1.0  ledger-report/Cargo.toml  publish: 제한 없음, license: null
      의존성 ledger-core  req "*"  (normal, path)

3. cargo metadata --format-version 1 (--no-deps 없이, 의존성 해석 포함)
  resolve의 키: nodes, root
  resolve.root = null (가상 매니페스트라 '현재 패키지'가 없음)
  resolve.nodes 3개:
    ledger-cli → ledger-core [normal, name=ledger_core], ledger-report [normal, name=ledger_report]
    ledger-core → (없음)
    ledger-report → ledger-core [normal, name=ledger_core]
```

sample_ws는 멤버를 적은 가상 매니페스트이고 `default-members`가 없으므로 기본 멤버도 셋 모두입니다. 의존성이 모두 path라서 `--no-deps` 없이 실행해도 `packages`는 3개 그대로이고, 네트워크에 접근하지 않습니다.

### 03_tool_placement — 도구를 둘 곳

```sh
cargo run -p er-ch31-tooling-ecosystem --example 03_tool_placement
```

도구를 실행하지 않고 성질만으로 분류하므로 설치되지 않은 도구도 다룹니다. 출력은 `도구 19개` 다음에 1번 구역(2.6절에 옮김)과 아래의 2번 구역이 이어집니다.

```text
2. 권장: 성질을 보고 둘 곳을 고름
  [편집기 + CI(변경마다)]
    cargo fmt (Part 0 ch1)
    cargo check (Part 0 ch1)
    cargo clippy (ch29)
  [CI(변경마다)]
    clippy pedantic (ch29)
    cargo test (ch30)
    cargo doc (-D warnings) (ch27)
    cargo deny check bans licenses (ch25)
    cargo-machete (ch25)
    cargo hack --feature-powerset (ch26)
  [CI(nightly 툴체인 작업)]
    Miri (ch16)
    cargo-udeps (ch25)
  [CI(정기 예약 작업)]
    cargo-audit (ch25)
    cargo bench (criterion) (ch30)
  [시간을 정해 따로]
    cargo-fuzz (ch30)
  [게시 전]
    cargo-semver-checks (ch21)
    cargo package --list (ch30)
  [필요할 때 손으로]
    cargo-expand (ch28)
    cargo tree -i (ch25)
    Compiler Explorer (ch30)
```

### projects/meta_lint — `cargo_metadata`로 만든 검사기

독립 워크스페이스입니다. 처음 빌드할 때 crates.io에서 `cargo_metadata` 0.23.1과 그 의존성(`serde`, `serde_json`, `semver`, `camino`, `cargo-platform`, `thiserror` 등)을 내려받습니다.

```sh
cd appd-a/ch31-tooling-ecosystem/projects/meta_lint
cargo run                  # sample_ws 검사, 위반이 있어 종료 코드 1
cargo run -- Cargo.toml    # 이 도구 자신 검사, 종료 코드 0
cargo test                 # test result: ok. 2 passed
```

```text
meta-lint: sample_ws/Cargo.toml (멤버 3개)
  ledger-cli     건너뜀 (publish = false)
  ledger-core    통과
  ledger-report  위반 2건
    - license와 license-file이 모두 없습니다
    - path 의존성 `ledger-core`에 version이 없습니다
위반 2건
```

`cargo run -- Cargo.toml`은 `meta-lint      건너뜀 (publish = false)`와 `위반 없음`을 출력합니다. `ledger-cli`에도 version 없는 path 의존성이 있지만 `publish = false`라 규칙을 적용하지 않습니다. 위반이 있으면 종료 코드 1로 끝나므로 그대로 CI의 검사 단계가 될 수 있습니다(지침 1). sample_ws 자체도 정상적인 워크스페이스입니다. `cd sample_ws && cargo run -p ledger-cli`는 `월급: 2500000원`, `월세: -700000원`, `항목 2개, 잔액 1800000원`을 출력합니다. 이 프로그램을 `cargo meta-lint`처럼 부르는 하위 명령으로 만드는 방법은 Part 3 `ch11-commands-tools`에서 다룹니다.

## 4. 자주 하는 실수

이 Item의 실수는 컴파일 에러가 아니라 도구를 쓰는 방식에서 나오므로 `compile_fail/`을 두지 않았습니다.

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| CI에서 `cargo fmt`(고치는 모드)를 실행 | 성공으로 끝나고 저장소의 코드는 그대로 | `cargo fmt --check` | 2.2, 01 |
| `cargo metadata`에 `--format-version`을 빠뜨림 | `please specify --format-version` 경고, 장래 형식 변경에 노출 | `--format-version 1` | 2.3, 02 |
| `cargo tree` 출력이나 JSON 글자를 긁어 셈 | 중복된 줄, 다른 뜻의 `"name"`까지 셈 | JSON 구조와 `resolve` 그래프를 따라 읽기 | 2.3, 02 |
| `resolve.nodes[].deps[].name`을 패키지 이름으로 씀 | `ledger_core`처럼 밑줄 이름 | `pkg` ID로 `packages`에서 찾기 | 2.3, 02 |
| 패키지 ID를 잘라 이름·버전을 얻음 | path 패키지의 ID에는 이름이 빠지기도 함 | ID는 불투명한 값으로 다루기 | 2.3, 02 |
| 산출물을 `target/`의 경로 규칙으로 찾음 | 1.100.0에서 구조가 바뀜, `CARGO_TARGET_DIR`이면 위치도 다름 | `target_directory`, `--message-format json` | 2.3 |
| `cargo --list`에 보이면 쓸 수 있다고 믿음 | `miri`는 프록시뿐 | `rustup component list --installed` | 2.1 |
| `cargo +nightly`나 `--toolchain nightly`로 "확인만" | rustup 1.29가 툴체인을 내려받음 | `rustup toolchain list` | 2.1 |
| 버전 없이 `cargo install` | 날마다 다른 도구와 의존성 | `이름@버전`, 1.98.1 이하에서는 `--locked`도 | 2.5 |
| 성질을 보지 않고 모든 도구를 변경마다 CI에서 돌림 | nightly 도구는 실패, 결과가 바뀌는 도구는 관계없는 변경을 실패시킴 | nightly 작업, 정기 예약 작업으로 나누기 | 2.6, 03 |

## 5. 연습 문제

### 연습 1. 도구의 설정을 매니페스트에서 읽기

[`projects/meta_lint`](projects/meta_lint/src/main.rs)가 sample_ws의 `Cargo.toml`에 적은 다음 설정을 읽어, 목록에 있는 패키지는 라이선스 규칙에서 빼도록 고치세요. path 의존성 규칙은 그대로 둡니다. `cargo run`의 결과와 테스트는 어떻게 바뀌어야 하나요?

```toml
[workspace.metadata.meta-lint]
allow-unlicensed = ["ledger-report"]
```

<details>
<summary>힌트</summary>

이 표는 `cargo metadata` 출력의 최상위 `metadata` 키에 실리고, `cargo_metadata`에서는 `Metadata::workspace_metadata`(타입은 `serde_json::Value`)입니다. `meta.workspace_metadata["meta-lint"]["allow-unlicensed"].as_array()`로 배열을 꺼내 `filter_map(|v| v.as_str())`로 이름 목록을 만들고, `check_package`에 넘겨 `p.name.as_ref()`가 목록에 있으면 라이선스 검사를 건너뜁니다. 값의 메서드만 쓰므로 `serde_json`을 의존성에 더하지 않아도 됩니다. 결과는 `ledger-report  위반 1건`, 마지막 줄 `위반 1건`, 종료 코드는 여전히 1입니다. 테스트의 `assert_eq!(problems.len(), 2)`도 1로 고쳐야 통과합니다(저장소 밖 사본에서 확인).

</details>

### 연습 2. `cargo tree`로 질문 B에 맞게 답하기

예제 02의 질문 B("ledger-cli가 직간접으로 의존하는 패키지는 몇 개인가?")에 `cargo tree`의 출력만으로 2라고 답하려면 어떤 옵션과 후처리가 필요할까요? 그렇게 맞춘 답이 여전히 깨지기 쉬운 이유를 두 가지 드세요.

<details>
<summary>힌트</summary>

`cargo tree -p ledger-cli --prefix none`은 트리 선 없이 한 줄에 패키지 하나를 출력하지만 `ledger-core`가 두 번 나옵니다. `| sort -u`로 중복을 지우면 3줄이 되고, 루트를 빼면 2입니다. 깨지기 쉬운 이유로는 줄에 들어 있는 절대 경로와 버전 표기에 기대는 점, dev-dependencies가 있으면 `-e` 옵션에 따라 줄이 달라지는 점(ch4처럼 `-e normal`과 `-e dev`의 결과가 다름), 같은 이름의 두 버전이 있으면 이름만으로는 셀 수 없는 점(ch25의 `이름@버전`), 출력 모양에 형식 버전 같은 약속이 없다는 점 등을 들 수 있습니다.

</details>

### 연습 3. 책이 든 도구 하나를 들여 보기

지도에서 이 저장소의 어느 챕터도 다루지 않은 `cargo-tarpaulin`을 지침 4의 다섯 항목으로 평가하고, 예제 03의 `tools()`에 한 줄로 더해 어느 자리에 놓이는지 확인하세요. 그 자리가 맞지 않는다고 생각되면 어떤 성질을 바꿔야 할까요?

<details>
<summary>힌트</summary>

2.4절에 옮긴 README의 내용(설치 명령에 `--locked`, nightly 요구 없음, Linux 기본 엔진은 x86_64 전용이고 macOS·Windows는 LLVM 엔진이 기본, LLVM 엔진은 0이 아닌 종료 코드로 끝난 테스트의 커버리지를 남기지 않음)에서 출발합니다. `tool("cargo-tarpaulin", "ch31", Check)`처럼 기본값으로 더하면 `[CI(변경마다)]`에 놓입니다. 커버리지 수치를 판정(통과/실패)에 쓰지 않고 추이만 본다면 목적은 `Check`보다 `Investigate`에 가깝고, 그러면 `[필요할 때 손으로]`로 옮겨 갑니다. 어느 쪽이든 CI 작업이 도는 운영체제와 CPU를 함께 따져야 합니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 31: Take advantage of the tooling ecosystem](https://effective-rust.com/use-tools.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- Cargo Book, [External tools](https://doc.rust-lang.org/cargo/reference/external-tools.html): `cargo metadata`, JSON 메시지, 사용자 정의 하위 명령, `CARGO` 환경 변수
- Cargo Book, [`cargo metadata`](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)(로컬에서는 `cargo help metadata`): 출력 형식, Compatibility, `--no-deps`, `--filter-platform`
- Cargo Book, [`cargo install`](https://doc.rust-lang.org/cargo/commands/cargo-install.html)(로컬에서는 `cargo help install`): Dealing with the Lockfile
- Cargo Book, [The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)(license와 license-file)과 [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)(path 의존성과 게시)
- 로컬 문서의 릴리스 노트(`rustc --print sysroot` 아래 `share/doc/rust/html/releases.md`): 1.82.0(`cargo info`), 1.100.0(Cargo의 린트, `cargo install`의 lockfile, 빌드 디렉터리 구조)
- [`cargo_metadata` 0.23.1 문서](https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/): `MetadataCommand`, `Metadata`, `Package`, `Dependency`
- rust-lang.org [Tools](https://www.rust-lang.org/tools): 편집기 지원, rustfmt, Clippy, Cargo Doc
- rust-analyzer [Configuration](https://rust-analyzer.github.io/book/configuration.html): `checkOnSave`, `check.command`
- 도구 README: [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks), [cargo-deny](https://github.com/EmbarkStudios/cargo-deny), [cargo-hack](https://github.com/taiki-e/cargo-hack), [cargo-machete](https://github.com/bnjbvr/cargo-machete), [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit), [cargo-expand](https://github.com/dtolnay/cargo-expand), [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) (모두 이 저장소에서는 실행하지 않음)
