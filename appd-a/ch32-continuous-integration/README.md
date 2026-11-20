# ch32. CI 시스템을 갖추기

ch25부터 ch33까지 여러 챕터가 "이 검사는 CI에서 돌린다"고 말하고 구성을 이 챕터로 미뤘습니다. 이 챕터는 그 검사들을 한 줄로 세웁니다. 도구의 사용법은 각 챕터로 연결하고, 여기서는 **어떤 검사를 어느 주기에 두는가, 로컬과 CI가 같은 명령을 돌리게 하려면 어떻게 구성하는가, 실패를 어떻게 다루는가**를 다룹니다. 작은 라이브러리 `pollcount`에 검사 스크립트 `ci.sh`와 GitHub Actions 워크플로를 두고, 사본에 문제를 하나씩 넣어 어느 단계가 어떤 종료 코드로 잡는지 확인합니다.

- 원제: [Item 32: Set up a continuous integration (CI) system](https://effective-rust.com/ci.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch32-continuous-integration`
- 독립 프로젝트: `projects/pollcount` (외부 크레이트 없음)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), MSRV 검사에 rustc 1.98.1, rustup 1.29.3
- **워크플로 [`ci/github-workflow.yml`](projects/pollcount/ci/github-workflow.yml)은 GitHub에서 실행해 확인하지 못했습니다.** YAML 문법만 `ruby -ryaml`로 확인했습니다. `cargo-deny`와 `cargo-hack`을 쓰는 단계, `thumbv6m-none-eabi` 타깃 빌드도 이 저장소에 도구와 타깃이 없어 실행해 확인하지 못했습니다.

책의 5장(Tooling)을 마무리하는 Item입니다. 다음 본편·부록 내용을 안다고 보고 진행합니다.

- [ch31](../ch31-tooling-ecosystem/README.md) 2.2·2.6절: 검사 모드와 고치는 모드, 도구를 둘 곳(편집기, 변경마다 도는 CI 작업, nightly 툴체인 작업, 정기 예약 작업, 시간을 정해 따로, 게시 전)
- [ch29](../ch29-listen-to-clippy/README.md) 2.4절: 경고 0개 유지, `-D warnings`와 `build.warnings`, Clippy 버전
- [ch30](../ch30-beyond-unit-tests/README.md) 2.9절: 테스트 종류별 명령, 벤치마크와 퍼즈 테스트를 돌리는 때
- [ch27](../ch27-document-public-interfaces/README.md) 2.6절: rustdoc 린트는 `cargo doc`을 실행하는 곳에서만 걸림
- [ch26](../ch26-feature-creep/README.md) 2.6절과 [ch33](../ch33-no-std/README.md) 2.5절: 기능 조합, `std` 없는 타깃 빌드
- [ch21](../ch21-semver/README.md) 2.7절과 [ch25](../ch25-dependency-graph/README.md) 2.4·2.5절: MSRV, `Cargo.lock`, 의존성 점검 도구
- [Part 1 ch10](../../part1-standard-lib/ch10-env-process-time/README.md): 예제가 `Command`로 cargo와 bash를 실행하고 종료 상태를 받음

GitHub Actions 문법 전반과 Cargo 쪽 테스트 명령의 상세는 Part 3 `ch9-testing-ci`에서, 크로스 컴파일은 Part 5 `ch4-targets-cross-compilation`에서, 코드 커버리지는 Part 5 `ch6-optimization-instrumentation`에서, 게시 절차는 Part 3 `ch10-publishing-semver`에서 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. CI에 넣을 단계(잠금 파일, 서식, Clippy, 모든 종류의 테스트, 예제 실행, 문서, 기능 조합, MSRV, 의존성 점검, 프로젝트 관례)를 나열하고, 각 단계를 "문제가 있으면 0이 아닌 종료 코드로 끝나는 명령"으로 작성할 수 있다.
2. 경고를 실패로 만드는 세 방법(`RUSTFLAGS=-Dwarnings`, `cargo clippy -- -D warnings`, `build.warnings`)의 차이를 설명하고, `rust-toolchain.toml`로 툴체인을 고정하는 이유와 우선순위를 설명할 수 있다.
3. 로컬과 CI가 같은 명령을 돌리는 검사 스크립트를 작성하고, 종료 코드가 사라지는 셸 모양(파이프, `|| true`, 반복문)과 설치되지 않은 도구를 다루는 규칙을 적용할 수 있다.
4. 검사를 주기(편집기, 리뷰 요청마다, 메인 병합마다, 정기 예약)에 나눠 GitHub Actions 워크플로로 옮기고, 기능 조합과 MSRV를 매트릭스로 구성할 수 있다.
5. 공개 CI에서 토큰 권한을 읽기로 좁히고 외부 액션을 커밋 SHA로 고정하는 이유를 설명할 수 있다.

## 2. 핵심 개념

2.1절에서 단계와 주기를 한 장의 지도로 모으고, 2.2~2.6절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 지침 1·2·4·5의 뼈대(단계 목록, 툴체인 고정, Clippy를 경고 실패로, 사람의 시간을 아끼는 주기 나누기, 예외 없는 실패 처리, 공개 CI의 보안)는 책이 권하는 것입니다. 지침 3의 스크립트 설계와 각 지침에 더한 세부(`build.warnings`, `--locked`, 종료 코드의 함정 등)는 Cargo·GitHub 문서와 이 챕터의 실행 결과를 근거로 더했습니다(2.7절). 책에도 코드가 없는 Item이므로, 이 챕터의 "코드"는 명령과 스크립트와 워크플로입니다.

### 2.1 지도: CI 단계와 주기

`projects/pollcount/ci.sh`의 단계와, 각 단계를 둔 주기입니다. "주기"의 낱말은 ch31의 자리를 따르되, ch31의 "변경마다"를 책처럼 **리뷰 요청마다**(PR을 열거나 커밋을 더할 때)와 **메인 병합마다**로 나눴습니다.

| 단계 | 실패로 끝나는 명령 | 잡는 것 | 다룬 곳 | 주기 |
| --- | --- | --- | --- | --- |
| `lockfile` | `cargo metadata --locked --format-version 1` | `Cargo.lock`이 매니페스트와 어긋남 | ch25, 2.2절 | 리뷰 요청마다 |
| `fmt` | `cargo fmt --check` | 서식 | ch31 | 편집기 + 리뷰 요청마다 |
| `conventions` | 셸 검사 | 프로젝트만의 관례 | 2.4절 | 리뷰 요청마다 |
| `clippy` | `cargo clippy --all-targets --all-features` | 린트, 경고, MSRV보다 새 API | ch29, ch21 | 편집기 + 리뷰 요청마다 |
| `test` | `cargo test --all-targets` + `cargo test --doc` | 단위·통합·문서 테스트 | ch30 | 리뷰 요청마다, 메인 병합마다 모든 OS |
| `examples` | 예제마다 `cargo run --example` | 예제 프로그램 | ch30 | 위와 같음 |
| `docs` | `cargo doc --no-deps --all-features` | 깨진 문서 링크 등 rustdoc 린트 | ch27 | 리뷰 요청마다 |
| `features` | `cargo clippy --no-default-features [--features f]` | 한 조합에서만 나는 에러·경고 | ch26, 예제 03 | 리뷰 요청마다 |
| `msrv` | `cargo +1.98.1 check` | MSRV보다 새 API와 언어 기능 | ch21 | 리뷰 요청마다 |
| `deny` | `cargo deny check bans licenses sources` | 중복 버전, 라이선스, 출처 | ch25 | 리뷰 요청마다 |
| `advisories` | `cargo deny check advisories` | 보안 권고 | ch25 | 정기 예약 |
| `hack` | `cargo hack check --feature-powerset` | 모든 기능 조합 | ch26 | 정기 예약 |
| (워크플로) | `cargo update` 뒤 `test` | 최신 의존성에서 깨짐 | ch25 | 정기 예약, 실패해도 막지 않음 |

넣지 않은 것도 있습니다. 벤치마크와 퍼즈 테스트는 정규 작업에 두지 않고(2.5절), Miri는 `pollcount`에 `unsafe`가 없어 둘 이유가 작습니다([ch16](../ch16-avoid-unsafe/README.md), ch31 2.6절). `pollcount`는 `std` 라이브러리라 `std` 없는 타깃 빌드도 없습니다. 이 단계가 필요한 크레이트의 구성은 예제 03의 2번 구역과 ch33 2.5절에 있습니다.

책은 상황에 따라 더할 검사로 넷을 더 듭니다. 라이브러리라면 `Cargo.lock` 없이도 빌드해 보고(사용자는 이 파일을 무시하므로, 워크플로의 `latest-deps` 작업), 플랫폼별 `cfg` 코드가 있으면 그 플랫폼에서 빌드하고(`test-all-os` 작업), 버전 관리하는 파일 가운데 기계가 만든 것이 있으면 다시 만들어 차이를 보고([`ch35-bindgen`](../ch35-bindgen/README.md)이 만든 선언이 그런 예), 공개 프로젝트라면 실수로 커밋한 토큰이나 키를 훑으라고 합니다(커밋 전 Git 훅도 함께). 뒤의 둘은 `pollcount`에 해당하지 않아 넣지 않았습니다. 다른 프로젝트의 CI를 참고하라는 권고도 있어서, 이 챕터의 워크플로는 Cargo 저장소의 `main.yml`에서 몇 가지 모양(액션의 SHA 고정, 결과를 모으는 작업)을 빌려 왔습니다.

### 2.2 지침 1: 단계는 "실패로 끝나는 명령"으로, 빠짐없이 적는다

**문제 상황.** CI는 명령의 출력이 아니라 종료 코드만 보고 성패를 정합니다(ch31 2.2절). 검사처럼 보이지만 실패하지 않는 명령이나, 검사하는 범위가 생각보다 좁은 명령을 넣으면 초록 빌드가 아무것도 보장하지 않습니다.

**비권장 코드.** 다음 명령은 모두 이 챕터에서 문제가 있는 코드에 실행해도 종료 코드 0으로 끝났습니다.

```sh
cargo clippy                # 경고가 있어도 0 (예제 01)
cargo doc --no-deps         # 깨진 문서 링크 경고가 있어도 0
cargo test --all-targets    # 문서 테스트를 돌리지 않음 (출력에 Doc-tests 구역이 없음)
cargo test --examples       # 예제 프로그램의 main을 실행하지 않음 (running 0 tests)
cargo clippy -- -D warnings # 기본 기능 조합 하나만 검사 (json만 켠 조합의 에러를 못 봄)
cargo +1.100.0 build        # MSRV(1.98)보다 새 언어 기능을 써도 통과
```

**권장 코드.** `ci.sh`는 단계마다 실패로 끝나는 명령을 둡니다. 경고는 스크립트 첫머리의 `CARGO_BUILD_WARNINGS=deny` 한 줄로 모든 cargo 명령에서 실패가 되고(2.3절), 테스트는 `--doc`을 따로 돌리고, 예제는 하나씩 실행합니다.

```sh
step_test() {
  run cargo test --locked --all-targets --all-features &&
    run cargo test --locked --doc --all-features
}
```

단계가 정말 그 문제를 잡는지는 문제를 넣어 보기 전에는 알 수 없습니다. 책이 권하는 대로 **검사를 먼저 더하고 빨강을 본 뒤에** 믿기로 하고, 저장소 밖의 사본에 문제를 하나씩 넣어 `sh ci.sh --keep-going`을 실행했습니다. 문제가 없는 사본은 종료 코드 0, 문제를 넣은 사본은 모두 종료 코드 1이었습니다.

| 넣은 문제 | 실패한 단계 (종료 코드) | 기본 실행이 멈춘 곳 |
| --- | --- | --- |
| 서식 어긋남(`new`를 한 줄로) | fmt (1) | fmt |
| 쓰지 않는 변수 | clippy, test, examples, features (모두 101) | clippy |
| 깨진 문서 링크(`` [`Votes::to_tsv`] ``) | docs (101) | docs |
| 기능 조합 하나에서만 나는 에러 | features (101) | features |
| MSRV보다 새 API(`str::trim_prefix`, 1.100.0) | clippy, features, msrv (모두 101) | clippy |
| MSRV보다 새 언어 기능(`type Err = !;`, 1.100.0) | msrv (101) | msrv |
| 실패하는 문서 테스트 | test (101) | test |
| 버전을 올리고 `Cargo.lock`을 갱신하지 않음 | lockfile, clippy, test, examples, docs, features, msrv (모두 101) | lockfile |
| 버전을 올리고 `CHANGELOG.md`에 절을 쓰지 않음 | conventions (1) | conventions |

"기본 실행이 멈춘 곳"은 `--keep-going` 없이 실행했을 때 첫 실패에서 멈추는 단계입니다. 표에서 읽을 것은 넷입니다.

- **문제마다 잡는 단계가 다릅니다.** 문서 테스트는 `test` 단계의 두 번째 명령만, 기능 조합의 에러는 `features`만, 새 언어 기능은 `msrv`만 잡았습니다. 단계 하나를 빼면 그 종류의 문제는 통과합니다. 문서 테스트를 넣은 사본에서 `cargo test --all-targets`는 `test result: ok`를 세 번 출력했고, 이어진 `--doc`이 `left: [("배", 2), ("귤", 1)]`, `right: [("배", 2), ("사과", 1)]`로 실패했습니다.
- **기능 조합의 에러는 한 조합에서만 납니다.** `use std::fmt::Write as _;`를 `#[cfg(any(feature = "csv", feature = "json"))]` 대신 `#[cfg(feature = "csv")]`로 가리자, `--no-default-features --features json`만 `` error[E0599]: cannot write into `std::string::String` ``으로 실패하고 기본 조합과 `--all-features`는 통과했습니다. 반대로 가리지 않으면 기능 없는 조합에서 `` unused import: `std::fmt::Write as _` `` 경고가 납니다. 둘 다 그 조합을 빌드해야만 보입니다.
- **Clippy는 MSRV의 일부만 압니다.** `str::trim_prefix`는 Clippy의 `incompatible_msrv`가 `` current MSRV (Minimum Supported Rust Version) is `1.98.0` but this item is stable since `1.100.0` ``로 잡았습니다. 1.100.0에서 안정화된 never 타입(`!`)을 쓴 코드는 1.100.0의 Clippy와 테스트를 모두 통과했고, 1.98.1로 빌드해야 `` error[E0658]: the `!` type is experimental ``가 났습니다. ch21이 말한 "언어 기능까지 확인하려면 그 버전으로 실제로 빌드한다"가 이것입니다.
- **잠금 파일은 가장 먼저, 그리고 모든 명령에서 확인합니다.** 버전만 올린 사본에서 `--locked` 없는 `cargo clippy`를 먼저 실행하자, Cargo가 아무 말 없이 `Cargo.lock`의 버전 줄을 고쳤고 그 뒤의 `cargo check --locked`는 통과했습니다. 그래서 `ci.sh`는 `lockfile` 단계를 맨 앞에 두고, 모든 cargo 명령에 `--locked`를 붙였습니다. 실패할 때의 메시지는 `` error: cannot update the lock file …/Cargo.lock because --locked was passed to prevent this ``입니다. `cargo metadata --no-deps --locked`는 의존성을 해석하지 않아 통과하므로 `--no-deps`를 붙이지 않습니다.

문서 단계에는 기능도 맞춰야 합니다. `pollcount`의 크레이트 문서는 `json` 기능에만 있는 메서드로 링크하므로(`` [`Votes::to_json`] ``), 기본 기능으로 `cargo doc`을 실행하면 `` warning: unresolved link to `Votes::to_json` ``이 납니다. docs.rs는 `Cargo.toml`의 `[package.metadata.docs.rs]`대로 문서를 만드는데 `pollcount`는 여기에 `all-features = true`를 적었으므로, `docs` 단계도 `--all-features`로 맞췄습니다.

워크스페이스라면 `--workspace`도 붙입니다. `-p` 없이 실행한 cargo 명령은 `default-members`만 검사합니다. ch29의 `projects/lint_config`에서 `CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets`는 종료 코드 0이었고, `--workspace`를 붙이자 `default-members`에서 빠진 `before-strict`의 경고 18개 때문에 종료 코드 101이 되었습니다(빌드 디렉터리는 임시 디렉터리로 돌려 실행). `pollcount`는 패키지가 하나라서 붙이지 않았습니다.

**지침을 따르지 않아도 되는 경우.** 라이브러리가 `Cargo.lock`을 커밋하지 않기로 했다면(ch25 2.4절이 드는 선택지) `lockfile` 단계와 `--locked`는 뺍니다. 사용자에게 바이너리만 배포하고 사용자가 직접 컴파일하지 않는 프로그램이라면 MSRV를 약속할 일이 없으니 `msrv` 단계도 필요 없습니다. 기능이 없는 크레이트에는 `features` 단계가 없습니다.

### 2.3 지침 2: 경고는 실패로, 툴체인과 도구는 고정한다

**문제 상황.** ch29는 경고 0개를 지켜야 새 경고가 보인다고 했습니다. CI가 경고를 실패로 다뤄야 0개가 지켜집니다. 방법이 여럿이고 부작용이 다르며, 같은 코드도 컴파일러 버전이 다르면 경고 목록이 다릅니다.

**비권장 코드.** [`01_warnings_as_errors.rs`](examples/01_warnings_as_errors.rs)는 경고가 하나 나는 임시 패키지에 명령을 차례로 실행합니다(출력 전체는 3절).

| 명령 | 종료 코드 | 다시 컴파일 | 걸리는 범위 |
| --- | --- | --- | --- |
| `cargo check` | 0 | 처음만 | 경고로 끝남 |
| `RUSTFLAGS=-Dwarnings cargo check` | 101 | 처음 바꿀 때 예 | 의존성까지 모든 크레이트에 rustc 플래그로 |
| `cargo clippy -- -D warnings` | 101 | 인자가 바뀔 때마다 예 | Clippy 실행에만 |
| `CARGO_BUILD_WARNINGS=deny cargo check` | 101 | 아니요 | 이 패키지(로컬 패키지)의 린트 경고 |

`RUSTFLAGS`는 rustc에 그대로 넘어가므로 다른 컴파일로 칩니다. cargo 1.100.0은 `RUSTFLAGS` 값마다 산출물을 따로 두어서(빌드 디렉터리에 해시가 다른 디렉터리 둘이 생김) 되돌릴 때는 다시 컴파일하지 않았지만, 처음 바꿀 때 전부 다시 컴파일하고 디스크를 두 벌 씁니다. 책의 방식인 `cargo clippy -- -D warnings`는 `--` 뒤의 인자가 바뀔 때마다 그 크레이트를 다시 검사했습니다. 로컬의 `cargo clippy`와 CI 스크립트의 `cargo clippy -- -D warnings`를 번갈아 돌리면 매번 다시 검사합니다.

**권장 코드: `build.warnings = "deny"`.** Cargo 1.97에서 안정화된 설정이고, Cargo Book의 Continuous Integration 장도 "Checking for Warnings" 절에서 이것을 권합니다. 컴파일 결과는 그대로 쓰고 판정만 바꾸므로 다시 컴파일하지 않습니다. `ci.sh`는 첫머리에서 한 번 내보냅니다.

```sh
CARGO_BUILD_WARNINGS=deny
export CARGO_BUILD_WARNINGS
```

이 한 줄이 걸리는 범위를 확인했습니다. Clippy와 `cargo check`(예제 01), `cargo test`와 `cargo run`의 빌드(2.2절 표의 "쓰지 않는 변수"), rustdoc 린트(같은 표의 깨진 문서 링크: `` error: warnings are denied by `build.warnings` configuration ``), Cargo 자신의 린트인 `cargo::unused_dependencies`(path 의존성 하나를 쓰지 않는 임시 패키지에서 종료 코드 101)까지 실패가 되었습니다. 그래서 ch27의 `RUSTDOCFLAGS='-D warnings'`를 따로 줄 필요가 없었습니다. 로컬 문서의 Cargo 설정 장에 따르면 린트가 아닌 경고와 의존성 쪽의 경고는 바뀌지 않습니다. 개발자가 평소에도 같은 판정을 원하면 `.cargo/config.toml`에 `[build] warnings = "deny"`를 둘 수 있지만, 실험 중인 코드까지 막히므로 `pollcount`는 스크립트에서만 켰습니다.

**경고는 고정한 툴체인 하나에서만 판정합니다.** `ci.sh`의 `msrv` 단계는 `CARGO_BUILD_WARNINGS=warn`으로 되돌린 하위 셸에서 1.98.1을 부릅니다. ch29에서 1.98.1의 Clippy는 1.100.0에서 생긴 린트를 몰라 `unknown lint`로 경고했고, 1.100.0에는 기본으로 켜진 새 린트가 다섯 개 생겼습니다. Cargo Book도 경고에는 호환성 보장이 약해 새 툴체인이 CI를 깨뜨릴 수 있다며 툴체인 고정을 권합니다.

**툴체인은 `rust-toolchain.toml`로 고정합니다.** 책이 굵게 권하는 것입니다. `projects/pollcount/rust-toolchain.toml`의 내용입니다.

```toml
[toolchain]
channel = "1.100.0"
components = ["clippy", "rustfmt"]
```

셸의 `RUSTUP_TOOLCHAIN`을 지운 하위 셸(`env -u RUSTUP_TOOLCHAIN`)에서 어떤 툴체인이 잡히는지 확인했습니다. 이 컴퓨터의 rustup 기본값은 1.98.1입니다.

| 실행한 곳 | `RUSTUP_TOOLCHAIN` | 잡힌 툴체인 | `rustup show active-toolchain`이 밝힌 이유 |
| --- | --- | --- | --- |
| 프로젝트 밖(임시 디렉터리) | 없음 | 1.98.1 | `(default)` |
| `projects/pollcount` | 없음 | 1.100.0 | `` overridden by '…/projects/pollcount/rust-toolchain.toml' `` |
| `projects/pollcount/src` | 없음 | 1.100.0 | (위 디렉터리로 올라가며 파일을 찾음) |
| `projects/pollcount` | `1.98.1` | 1.98.1 | `overridden by environment variable RUSTUP_TOOLCHAIN` |
| `projects/pollcount`에서 `cargo +1.98.1` | 없음 | cargo 1.98.1 | (`+` 표기가 가장 우선) |

rustup 문서(Overrides)의 우선순위가 그대로 보입니다. `+툴체인` 표기, `RUSTUP_TOOLCHAIN`, `rustup override`로 정한 디렉터리 설정, `rust-toolchain.toml`, 기본 툴체인 순입니다. 그래서 `ci.sh`의 `msrv` 단계는 `+1.98.1`로 파일을 넘어서고, 셸에 `RUSTUP_TOOLCHAIN`을 고정해 둔 개발자는 파일이 무시된다는 것을 알아야 합니다. 이 파일에는 주의할 점이 하나 있습니다. rustup 1.29는 이름으로 가리킨 툴체인이 없으면 내려받으므로(ch31 2.1절), 이 파일에 적은 툴체인이나 구성 요소가 없을 때도 내려받습니다(이 저장소에서는 설치를 일으키지 않으려고 실행해 보지 않았습니다). CI에서는 바라는 동작이고, 워크플로는 `rustup toolchain install`을 인자 없이 실행해 이 파일의 툴체인을 미리 설치합니다(1.29.3의 `rustup toolchain install --help`: "or by default the active toolchain"). 책의 각주처럼 nightly 전용 기능에 기대는 프로젝트라면 이 파일이 그 사실을 드러내 줍니다.

툴체인을 올리는 일은 그 줄만 바꾸는 변경으로 따로 만듭니다. 새 컴파일러가 낸 새 경고(ch29의 `mismatched_bit_width_type`)가 그 변경 하나에서만 보이고, 기능 변경과 섞이지 않습니다. 외부 도구도 같은 이유로 버전을 고정합니다(`cargo install --locked cargo-deny@0.22.0`, ch31 2.5절). `ci.sh`의 `DENY_VERSION`·`HACK_VERSION`과 워크플로의 설치 줄이 그 버전이고, 둘은 손으로 맞춥니다.

**지침을 따르지 않아도 되는 경우.** 다음 컴파일러에서 깨지는지 미리 보려는 작업은 일부러 고정하지 않습니다. Cargo Book의 첫 예처럼 stable·beta·nightly를 매트릭스로 돌리되, 실패해도 전체를 막지 않게 둡니다(GitLab 예의 `allow_failure: true`, GitHub의 `continue-on-error: true`). Cargo 1.97보다 오래된 툴체인에서 경고를 판정해야 한다면 `build.warnings`가 없으므로 `-D warnings`를 씁니다.

### 2.4 지침 3: 로컬과 CI가 같은 스크립트를 돌리고, 실패는 종료 코드로 전한다

**문제 상황.** 책은 CI의 검사를 자동화와 상관없이 손으로도 쉽게 돌릴 수 있게 하라고 권하고, CI 스크립트가 환경 준비 절차의 문서 노릇을 한다고 말합니다. 명령이 워크플로 YAML에만 있으면 실패를 재현하려고 YAML을 셸 명령으로 옮겨 적어야 하고, 시간이 지나면 둘이 어긋납니다. 셸에는 실패를 조용히 삼키는 모양도 있습니다.

**비권장 코드.** 검사 명령을 워크플로의 `run`에 흩어 두고, 로그를 남기려고 파이프를 씁니다.

```yaml
- run: cargo test --all-features 2>&1 | tee test.log
- run: for f in csv json; do cargo clippy --no-default-features --features $f; done
```

[`02_step_runner.rs`](examples/02_step_runner.rs)의 1번 구역은 이런 모양을 bash로 실제로 실행합니다. GitHub Actions는 `run`의 `shell`을 적지 않으면 `bash -e {0}`으로, `shell: bash`라고 적으면 `bash --noprofile --norc -eo pipefail {0}`으로 실행합니다(워크플로 문법 문서).

| 모양 | 종료 코드 |
| --- | --- |
| 종료 코드 101로 끝나는 명령 `\| tee`, `bash -e` | 0 |
| 같은 파이프, `bash -eo pipefail` | 101 |
| 실패한 명령 `\|\| true` | 0 |
| `-e` 없는 스크립트의 반복문, 첫 반복 실패 | 0 |
| 반복문 안에서 `\|\| exit` | 101 |

파이프의 종료 코드는 마지막 명령(`tee`)의 것이고, `-e` 없이 도는 반복문의 결과는 마지막 반복의 결과입니다. 로그에 `FAILED`가 찍혀 있어도 CI는 초록으로 표시합니다.

**권장 코드: 검사 명령은 스크립트 한 곳에.** [`projects/pollcount/ci.sh`](projects/pollcount/ci.sh)를 로컬에서도 워크플로에서도 `sh ci.sh <단계…>`로 부릅니다. 설계에서 고른 것은 다음과 같습니다.

- **POSIX sh로 씁니다.** macOS의 `sh`(bash 3.2)와 Ubuntu의 `sh`인 dash에서 모두 돌려 봤습니다(이 컴퓨터의 `/bin/dash`). 이 dash는 `set -o pipefail`을 `Illegal option`으로 거부했으므로, 판정에 쓰는 명령은 파이프로 잇지 않습니다.
- **단계는 함수 하나, 명령은 `run`으로 보여 주고 실행합니다.** 로그의 `$ cargo …` 줄을 복사하면 같은 명령을 손으로 돌릴 수 있습니다.
- **`set -e`에 기대지 않습니다.** bash 3.2와 dash 모두 `set -e`를 켜도 `f || …`처럼 부른 함수 안에서는 실패한 명령 뒤를 계속 실행했습니다. 단계 안에서는 `&&`와 `|| return`으로 잇고, 단계의 종료 코드는 `$?`로 받습니다.
- **기본은 첫 실패에서 멈추고, `--keep-going`은 끝까지 돌고 요약합니다.** 02의 2번 구역이 차이를 셉니다. 문제 셋(서식, 문서 테스트, 문서 링크)이 든 변경은 첫 실패에서 멈추면 초록이 될 때까지 CI를 4번, 끝까지 돌면 2번 기다리게 했습니다. 로컬에서는 빠른 답을 원하므로 멈추고, CI에서는 사람이 기다리는 횟수를 줄이려고 끝까지 돕니다.
- **설치되지 않은 도구는 로컬에서 건너뛰고, CI에서는 실패로 다룹니다.** GitHub Actions는 `CI=true`를 늘 넣어 줍니다(변수 문서). 로컬에서 모든 사람에게 모든 도구를 요구하지 않으면서도, CI의 도구 설치가 깨진 날 아무것도 검사하지 않는 초록 빌드가 이어지는 것을 막습니다(02의 3번 구역). `rustup`이 이름만으로도 툴체인을 내려받으므로 MSRV 툴체인은 `rustup toolchain list`로만 확인합니다.
- **종료 코드 규칙.** 모두 통과하면 0, 하나라도 실패하면 1, 사용법이 틀리면 2입니다. 단계의 종료 코드는 `[실패] docs (종료 코드 101)`처럼 따로 출력합니다.

```text
$ CI=true sh ci.sh --keep-going deny hack fmt
==> deny
    cargo-deny 없음: CI에서는 건너뛸 수 없습니다 (준비: cargo install --locked cargo-deny@0.22.0)
[실패] deny (종료 코드 1)
==> hack
    cargo-hack 없음: CI에서는 건너뛸 수 없습니다 (준비: cargo install --locked cargo-hack@0.6.46)
[실패] hack (종료 코드 1)
==> fmt
    $ cargo fmt --check
==> 요약
  실패: deny(1) hack(1)
```

(첫 줄의 `모드: CI, rustc 1.100.0 …`은 줄였습니다.) 같은 명령을 `CI` 없이 실행하면 두 단계는 `건너뜀`이 되고 종료 코드는 0입니다.

**프로젝트만의 관례는 짧은 셸 검사로.** 책은 Cargo가 모르는 관례를 간단한 스크립트로 검사하라고 권합니다. `pollcount`의 관례는 "`Cargo.toml`의 `version`과 같은 `## <버전>` 절이 `CHANGELOG.md`에 있어야 한다"입니다. 어떤 린트도 변경 기록 파일을 읽지 않으므로 `grep -F -x` 한 줄로 검사합니다. 반대로 린트가 있는 관례라면 린트를 씁니다. 셸의 `grep`은 주석과 문자열까지 걸리지만 린트는 코드만 봅니다(연습 1). 책은 이런 관례의 예로 "패닉을 일으키는 호출에는 표시 주석을 단다"를 드는데, [ch18](../ch18-dont-panic/README.md) 2.6절의 restriction 린트(`unwrap_used`, `indexing_slicing` 등)를 `Cargo.toml`의 `[lints.clippy]`에 켜 두면 `clippy` 단계가 그대로 검사합니다. `ci.sh`가 `build.warnings`를 켜 두었으므로 `"warn"`으로 켜도 CI에서는 실패가 됩니다. 검사가 길어지면 ch31의 [`meta_lint`](../ch31-tooling-ecosystem/projects/meta_lint/src/main.rs)처럼 Rust 프로그램으로 쓰고 위반이 있을 때 종료 코드 1을 돌려주게 해서 단계 하나로 붙입니다.

**지침을 따르지 않아도 되는 경우.** 명령이 `cargo build`와 `cargo test` 둘뿐인 작은 프로젝트라면 Cargo Book의 예처럼 YAML에 바로 적어도 재현에 문제가 없습니다. 셸이 없는 환경까지 같은 스크립트로 돌려야 한다면 스크립트 자체를 Rust 프로그램으로 쓰는 편이 낫습니다. 워크플로는 Windows에서도 `shell: bash`(Git Bash)로 `ci.sh`를 부르게 했지만 실행해 확인하지 못했습니다.

### 2.5 지침 4: 검사를 주기별로 나누고, 실패는 예외 없이 고친다

**문제 상황.** 책은 CI의 핵심 원칙으로 **사람의 시간을 낭비하지 않는다**를 듭니다. 리뷰를 요청한 뒤에야 느리게 도는 CI, 가끔 실패하는 테스트, 의미 없는 대기가 낭비입니다. 모든 검사를 한 곳에 몰면 느려지고, 드물게 돌리면 늦게 압니다.

**비권장 코드.** 모든 조합을 변경마다 돌립니다. [`03_feature_matrix.rs`](examples/03_feature_matrix.rs)는 기능 조합에 운영체제 셋과 툴체인 둘을 곱하면 `pollcount`도 24개, 독립 기능이 다섯인 크레이트는 192개의 빌드가 된다고 셉니다. 반대로 메인 병합 때만 돌리면 리뷰어가 승인한 뒤에야 빨강을 봅니다.

**권장 코드.** 책의 주기 나누기를 ch31의 자리와 워크플로의 작업으로 옮긴 표입니다.

| 주기(책) | ch31의 자리 | 워크플로 작업 | 돌리는 것 |
| --- | --- | --- | --- |
| 개발 환경 | 편집기 | (없음) | 저장할 때 `rustfmt`, rust-analyzer의 검사 |
| 리뷰 요청마다 | 변경마다 도는 CI 작업 | `quick`, `checks` | `ci.sh`의 lockfile~docs, 매트릭스로 features·msrv·deny |
| 메인 병합마다 | 변경마다 도는 CI 작업 | `test-all-os` | ubuntu·macos·windows에서 test·examples |
| 정기 예약(매주) | 정기 예약 작업 | `scheduled`, `latest-deps` | advisories·hack, `cargo update` 뒤 test |
| 계속 | 시간을 정해 따로 | (없음) | 퍼즈 테스트 |
| — | nightly 툴체인 작업 | (없음) | Miri(`unsafe`가 생기면) |
| — | 게시 전 | (없음) | `cargo-semver-checks`, `cargo package --list` |

- **리뷰 요청마다는 빠르고 결정적인 것만.** `quick`이 한 환경에서 빠른 단계를 `--keep-going`으로 돌리고, 서로 독립인 features·msrv·deny는 `checks` 매트릭스로 나눠 동시에 돕니다. 매트릭스 항목마다 준비물(`setup`)을 함께 적었고, `fail-fast: false`로 하나가 실패해도 나머지를 끝까지 돌립니다(기본값은 `true`). 기능 조합은 "없음 + 하나씩 + 전부"로 줄인 집합만 돌립니다. `pollcount`처럼 기능이 둘이면 이것이 곧 모든 조합(4개)이고, 다섯이면 32개 대신 7개입니다.
- **결과가 바뀌는 검사는 정기 예약으로.** 보안 권고는 코드가 그대로여도 새로 붙고(ch25), 최신 의존성은 남이 게시한 버전에 따라 깨집니다. Cargo Book의 예를 따라 `latest-deps`는 `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS: allow`로 `rust-version` 때문에 새 버전이 걸러지지 않게 하고, `continue-on-error: true`로 실패해도 실행 전체를 실패로 만들지 않습니다. 모든 기능 조합(`cargo hack --feature-powerset`)도 여기에 둡니다. Cargo Book은 오래 손대지 않은 저장소의 예약 작업을 CI 서비스가 끌 수 있다고 경고합니다.
- **벤치마크와 퍼즈 테스트는 정규 작업에 넣지 않습니다.** 책도 공유 CI 머신은 측정값이 시끄러워 믿을 만한 벤치마크에는 전용 환경이 필요하다고 하고, ch30에서는 손대지 않은 코드가 `regressed`로 판정되기도 했습니다. 측정은 이전 결과와 비교할 때 쓸모가 있으므로, 둔다면 전용 머신의 예약 작업에서 비교만 합니다. 퍼즈 테스트는 끝이 없으므로 시간을 정해 따로 돌립니다(ch30 2.7절). 커버리지(`cargo-tarpaulin`)도 책이 드는 측정 단계이지만 이 챕터에서는 다루지 않습니다.
- **`std` 없는 타깃이 있다면 매트릭스에 한 줄.** 예제 03의 2번 구역처럼 `std`를 켜지 않은 조합을 `rustup target add thumbv6m-none-eabi`를 준비물로 둔 항목에서 `cargo build --target thumbv6m-none-eabi`로 빌드합니다(ch33 2.5절, 실행해 확인하지 못함). 타깃을 설치할 수 없는 환경이라면 ch33의 검사용 크레이트(`#[panic_handler]`를 정의한 `#![no_std]` 크레이트, `projects/leak_check/nostd-check`)를 호스트에서 빌드하는 단계가 보조 확인이 됩니다.

**CI를 리뷰에 연결합니다.** 책은 리뷰어가 초록 빌드를 보고 사소한 것 대신 의미에 집중하게 하라고 합니다. 워크플로의 마지막 작업 `ci-ok`는 앞 작업들의 결과를 모아 하나라도 `failure`나 `cancelled`면 실패합니다. 브랜치 보호 규칙의 필수 검사로는 이 작업 하나만 걸면 되고, 매트릭스 항목이 늘어도 규칙을 고칠 필요가 없습니다. 건너뛴 작업은 성공으로 치므로, 앞 작업이 실패해도 이 작업이 돌도록 `if: ${{ !cancelled() }}`를 씁니다(Cargo 저장소의 `conclusion` 작업과 같은 모양). `concurrency`의 `cancel-in-progress: true`는 같은 PR에 새 커밋이 오면 앞의 실행을 취소합니다. 사람이 보는 것은 마지막 결과뿐입니다.

**실패를 다루는 법.**

- **예외를 두지 않습니다.** 책은 받아들인 실패가 새 실패를 가린다고 말합니다. "원래 빨간 작업"이 하나 있으면 다른 실패도 빨강 속에 섞입니다. 이 워크플로에서 실패를 허용하는 작업은 남의 변경 때문에 깨질 수 있는 `latest-deps` 하나뿐입니다.
- **불안정한(flaky) 테스트는 원인을 고칩니다.** 책은 이것을 엔지니어의 시간을 가장 성가시게 낭비하는 것으로 들며, 다시 돌리기로 덮지 말고 원인을 찾아 고치라고 합니다. 느리고 드물게 실패하는 테스트는 따로 떼어 정기 예약 작업으로 옮깁니다.
- **과정의 문제는 검사를 먼저 더합니다.** 책은 문제를 고치기 전에 그 문제를 잡는 CI 단계를 먼저 더해 빨강에서 초록으로 바뀌는 것을 보라고 권합니다(ch30의 재현 테스트와 같은 생각). 2.2절의 표가 그렇게 만든 것입니다.

**지침을 따르지 않아도 되는 경우.** 혼자 하는 작은 프로젝트라면 푸시마다 도는 작업 하나로 충분합니다. 주기를 나누는 것은 검사가 느려지거나 결과가 흔들리기 시작할 때입니다.

### 2.6 지침 5: 공개 CI는 권한을 좁히고 외부 코드를 고정한다

**문제 상황.** 책은 공개 CI가 악용과 공격의 대상이 된다고 경고합니다. 암호화폐 채굴, 접근 토큰 탈취, 공급망 공격입니다. CI는 저장소에 쓸 수 있는 토큰과 비밀을 들고 남의 코드(PR, 외부 액션, 내려받은 도구)를 실행하는 곳입니다.

**비권장 코드.**

```yaml
on: pull_request_target        # 포크의 PR도 쓰기 권한과 비밀을 가진 채 돈다
permissions: write-all
steps:
  - uses: actions/checkout@main   # 브랜치: 누가 밀어 넣든 다음 실행에 들어온다
  - run: echo "${{ github.event.pull_request.title }}"   # PR 제목이 스크립트가 된다
```

**권장 코드.** `pollcount`의 워크플로가 따른 것입니다. 근거는 GitHub의 보안 문서(Secure use reference)와 워크플로 문법 문서입니다.

- **`permissions: contents: read`.** 토큰은 필요한 최소 권한으로 둡니다. 하나라도 적으면 적지 않은 권한은 `none`이 됩니다. 쓰기가 필요하면 그 권한만 더합니다. 이 저장소의 사이트 배포 워크플로는 `contents: read`에 배포용 `pages: write`와 `id-token: write`만 더했습니다.
- **`pull_request`를 씁니다.** GitHub 문서는 `pull_request_target`과 `workflow_run`이 쓰기 권한과 비밀을 가지므로 믿을 수 없는 PR에 쓰지 말라고 합니다. 포크에서 온 PR의 워크플로는 쓰기 권한을 가진 관리자의 수동 승인을 받게 저장소·조직 단위로 설정할 수 있습니다. 책이 말한 "새 기여자는 손으로 실행"이 이것입니다.
- **외부 액션은 커밋 SHA로 고정합니다.** GitHub 문서는 태그는 옮기거나 지울 수 있으니 SHA가 가장 안전하다고 하고, 태그는 만든 이를 믿을 때만 쓰라고 합니다. `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1`의 SHA는 2026-11-20에 `git ls-remote`로 `v7.0.1` 태그에서 확인했고, Cargo 저장소의 워크플로도 같은 SHA를 씁니다. 주석의 버전은 사람이 읽으라고 붙입니다.
- **`persist-credentials: false`.** `actions/checkout`은 기본으로 토큰을 git 설정에 남깁니다(기본값 `true`). 뒤 단계가 푸시하지 않으므로 남기지 않습니다.
- **도구는 버전을 정해 설치합니다.** `cargo install --locked cargo-deny@0.22.0`처럼 적습니다(2.3절).
- **믿을 수 없는 값은 스크립트에 끼워 넣지 않습니다.** PR 제목 같은 값을 `run`에 `${{ … }}`로 넣으면 그 글자가 셸 스크립트의 일부가 됩니다. GitHub 문서는 환경 변수로 받아 쓰라고 권합니다. `pollcount`의 워크플로는 이런 값을 쓰지 않습니다.
- **`timeout-minutes`를 정합니다.** 기본값은 360분입니다. 멈춘 작업이 러너를 오래 붙잡지 않게 15~30분으로 줄였습니다.

**지침을 따르지 않아도 되는 경우.** 믿는 사람만 기여하는 비공개 저장소이거나, GitHub이 직접 관리하는 액션처럼 만든 이를 믿는다면 메이저 태그(`@v7`)로도 충분하다고 판단할 수 있습니다. 이 저장소의 사이트 배포 워크플로도 `actions/checkout@v7`처럼 메이저 태그로 씁니다. 대신 태그가 옮겨지면 다음 실행에 새 코드가 들어온다는 것을 받아들이는 것입니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc·cargo 1.100.0과 2026-11-20의 GitHub 문서에서 확인한 차이는 다음과 같습니다.

- **경고를 실패로 만드는 권장 방법이 바뀌었습니다.** 책은 `cargo clippy -- -Dwarnings`를 듭니다. Cargo 1.97의 `build.warnings`는 다시 컴파일하지 않고 Clippy·빌드·테스트·rustdoc·Cargo 린트에 한꺼번에 걸립니다(2.3절, 예제 01). Cargo Book의 Continuous Integration 장도 이것을 권합니다.
- **`cargo test`의 범위.** 책은 `cargo test`가 단위·통합·문서 테스트를 돌린다고 합니다. 옵션 없는 `cargo test`는 그렇지만, `--all-targets`를 주면 문서 테스트가 빠지고 `--examples`는 예제의 `main`을 실행하지 않습니다(ch30, 2.2절).
- **의존성 점검의 일부가 Cargo에 들어왔습니다.** 책은 nightly가 필요한 `cargo-udeps`를 듭니다. 1.100.0의 `cargo::unused_dependencies` 경고는 `build.warnings = "deny"`로 실패가 됩니다(ch25, 2.3절).
- **MSRV의 일부는 Clippy가 잡습니다.** `incompatible_msrv`는 표준 라이브러리 API만 보고, 1.100.0에서 안정화된 never 타입 같은 언어 기능은 MSRV 툴체인으로 빌드해야 잡혔습니다(2.2절).
- **`cargo install`은 1.100.0부터 패키지의 `Cargo.lock`을 기본으로 씁니다**(ch31 2.5절). 오래된 cargo를 쓰는 CI 이미지를 생각해 `--locked`는 그대로 붙였습니다.
- **GitHub Actions.** 책 시점 이후 `actions/checkout`은 v7(2026-11-20, v7.0.1)까지 나왔습니다. GitHub 문서는 SHA 고정을 가장 안전한 방법으로 권하고, `CI` 변수를 늘 `true`로 넣어 줍니다.
- **확인하지 못한 것.** 책의 예인 `thumbv6m-none-eabi` 빌드, `cargo-deny`·`cargo-hack`·`cargo-tarpaulin`·`cargo-udeps`, 워크플로 자체는 이 저장소에서 실행하지 못했습니다.
- **책에 없는 지침과 내용.** 다음은 책의 Item 32가 명시적으로 권하지 않는 것입니다(2026-11-20에 온라인판에서 확인). 책은 빌드와 테스트, 툴체인 고정, 기능 조합·`no_std`·MSRV·모든 테스트 종류, Clippy·`cargo doc`·`cargo-udeps`·`cargo-deny`·`cargo fmt --check`, 커버리지와 벤치마크, 상황에 따른 검사(`Cargo.lock` 없이 빌드, 생성된 파일, 플랫폼, 비밀), 관례 검사 스크립트, 사람의 시간·불안정한 테스트·주기 나누기·리뷰 연결·예외 없음·검사 먼저 더하기, 공개 CI와 보안을 다룹니다.
  - 지침 1의 `--locked`와 `lockfile` 단계를 맨 앞에 두기, 문서 단계의 기능을 docs.rs에 맞추기, 워크스페이스에서 `--workspace`: 2.2절의 실험(ch29 프로젝트 포함)과 `cargo help metadata`, `pollcount`의 `[package.metadata.docs.rs]`가 근거입니다.
  - 지침 2의 `build.warnings`와 `RUSTFLAGS`·`-D warnings`의 비교, MSRV 툴체인에서는 경고를 판정하지 않기: Cargo 설정 장, Cargo Book의 CI 장, 예제 01이 근거입니다.
  - 지침 3 전체(스크립트 한 곳, 종료 코드가 사라지는 셸 모양, `set -e`에 기대지 않기, 첫 실패에서 멈춤과 끝까지 돌기, 설치되지 않은 도구 규칙), 관례 가운데 린트가 있는 것은 린트로(`dbg_macro`, ch18의 restriction 린트): 책은 손으로 돌릴 수 있게 하라고만 하고, 관례 검사의 예로는 셸 스크립트를 듭니다. GitHub의 워크플로 문법·변수 문서와 예제 02, bash·dash의 실행 결과, 연습 1의 사본 실험이 근거입니다.
  - 지침 4의 주기를 ch31의 자리와 워크플로 작업으로 옮긴 표, 매트릭스와 `fail-fast`, `ci-ok` 작업, `concurrency`, `latest-deps`의 `continue-on-error`: GitHub 문서, Cargo Book의 CI 장, Cargo 저장소의 워크플로가 근거입니다. `cargo-hack`은 책에 없는 도구입니다.
  - 지침 5의 `permissions`, `pull_request_target` 주의, `persist-credentials`, 믿을 수 없는 값의 주입, `timeout-minutes`: 책은 자동 실행 제한, 외부 스크립트 고정, 읽기 이상의 권한 감시를 권합니다. 세부는 GitHub 보안 문서와 `actions/checkout` README가 근거입니다.

### 2.8 기억할 것

- CI는 종료 코드만 봅니다. 단계마다 문제가 있으면 실패로 끝나는 명령을 두고, 문제를 넣어 빨강을 본 뒤에 그 단계를 믿습니다.
- 경고는 `build.warnings = "deny"`로 실패가 됩니다. 판정은 `rust-toolchain.toml`로 고정한 툴체인 하나에서 하고, 툴체인과 도구를 올리는 일은 따로 된 변경으로 만듭니다.
- 검사 명령은 스크립트 한 곳에 두고 로컬과 CI가 함께 부릅니다. 파이프, `|| true`, `-e` 없는 반복문은 실패를 삼킵니다.
- 빠르고 결정적인 검사는 리뷰 요청마다, 모든 환경은 메인 병합마다, 결과가 바뀌는 검사는 정기 예약으로 둡니다. 실패에는 예외를 두지 않습니다.
- 공개 CI의 토큰은 읽기로 좁히고, 외부 액션은 SHA로, 도구는 버전으로 고정합니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행합니다. 세 예제 모두 종료 코드 0으로 끝나고 std만 씁니다. 01은 `cargo run`이 넣어 주는 `CARGO`로 임시 디렉터리의 작은 패키지에 cargo를 다시 부르고(1초 안팎), 02는 bash를 부르고, 03은 계산만 합니다. `projects/pollcount`는 그 디렉터리에서 실행합니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_warnings_as_errors.rs`](examples/01_warnings_as_errors.rs) | 2 | 경고를 둠, `RUSTFLAGS`, `-- -D warnings` | `build.warnings = "deny"` | 2.3 |
| [`02_step_runner.rs`](examples/02_step_runner.rs) | 3 | 파이프·`\|\| true`·반복문, 도구가 없으면 조용히 통과 | `pipefail`, `\|\| exit`, 로컬은 건너뛰고 CI는 실패 | 2.4 |
| [`03_feature_matrix.rs`](examples/03_feature_matrix.rs) | 1, 4 | 모든 조합을 변경마다 | 줄인 집합, 주기 나누기 | 2.2, 2.5 |
| [`projects/pollcount`](projects/pollcount/ci.sh) | 1~5 | (사본에 넣은 문제 아홉) | `ci.sh`, 워크플로, `rust-toolchain.toml` | 2.2~2.6 |

### 01_warnings_as_errors — 경고를 실패로 만드는 세 가지 방법

```sh
cargo run -p er-ch32-continuous-integration --example 01_warnings_as_errors
```

```text
1. 비권장: 경고를 그대로 둠
  cargo check
    종료 코드 0, 다시 컴파일 예
    | warning: unused variable: `unused`
  cargo check (한 번 더)
    종료 코드 0, 다시 컴파일 아니요
    | warning: unused variable: `unused`
2. RUSTFLAGS=-Dwarnings: 실패하지만 처음 바꿀 때 다시 컴파일
  RUSTFLAGS=-Dwarnings cargo check
    종료 코드 101, 다시 컴파일 예
    | error: unused variable: `unused`
  cargo check (RUSTFLAGS 없이 다시)
    종료 코드 0, 다시 컴파일 아니요
    | warning: unused variable: `unused`
3. cargo clippy -- -D warnings: Clippy에만 걸리고 인자가 바뀌면 다시 검사
  cargo clippy
    종료 코드 0, 다시 컴파일 예
    | warning: unused variable: `unused`
  cargo clippy -- -D warnings
    종료 코드 101, 다시 컴파일 예
    | error: unused variable: `unused`
  cargo clippy (인자 없이 다시)
    종료 코드 0, 다시 컴파일 예
    | warning: unused variable: `unused`
4. 권장(Cargo 1.97+): build.warnings = "deny"
  CARGO_BUILD_WARNINGS=deny cargo check
    종료 코드 101, 다시 컴파일 아니요
    | warning: unused variable: `unused`
    | error: warnings are denied by `build.warnings` configuration
  CARGO_BUILD_WARNINGS=deny cargo clippy
    종료 코드 101, 다시 컴파일 아니요
    | warning: unused variable: `unused`
    | error: warnings are denied by `build.warnings` configuration
```

"다시 컴파일"은 cargo의 표준 에러에 `Checking warn-demo` 줄이 있었는지입니다. 두 번째 `cargo check`의 경고는 Cargo가 저장해 두었다가 다시 보여 준 것입니다. 예제는 바깥 환경의 `RUSTFLAGS`·`CARGO_BUILD_WARNINGS`를 지우고 빌드 디렉터리를 임시 패키지 안에 둔 채 실행합니다.

### 02_step_runner — 실패를 종료 코드로, 빠짐없이

```sh
cargo run -p er-ch32-continuous-integration --example 02_step_runner
```

```text
1. 실패한 명령의 종료 코드가 사라지는 곳
  비권장: 파이프, shell 미지정(bash -e)  → 종료 코드 0
  권장:   파이프, shell: bash(-eo pipefail)  → 종료 코드 101
  비권장: 실패를 || true로 덮음  → 종료 코드 0
  권장:   덮지 않음  → 종료 코드 101
  비권장: -e 없는 스크립트의 반복문  → 종료 코드 0
  권장:   반복문 안에서 || exit  → 종료 코드 101
2. 한 변경에 문제가 셋(서식, 문서 테스트, 문서 링크)일 때
  첫 실패에서 멈춤 (ci.sh의 기본)
    1회차: 단계 2개 실행, 실패 fmt(1) (종료 코드 1)
    2회차: 단계 5개 실행, 실패 test(101) (종료 코드 1)
    3회차: 단계 7개 실행, 실패 docs(101) (종료 코드 1)
    4회차: 단계 8개 실행, 모두 통과 (종료 코드 0)
    → 초록이 될 때까지 CI를 4번 기다림
  끝까지 돌고 요약 (ci.sh --keep-going, 또는 단계마다 따로 도는 CI 작업)
    1회차: 단계 8개 실행, 실패 fmt(1) test(101) docs(101) (종료 코드 1)
    2회차: 단계 8개 실행, 모두 통과 (종료 코드 0)
    → 초록이 될 때까지 CI를 2번 기다림
3. cargo-deny가 설치되지 않은 환경에서 deny 단계
  비권장: 조용히 통과, 로컬: 통과로 처리, 종료 코드 0
  비권장: 조용히 통과, CI: 통과로 처리, 종료 코드 0
  권장:   ci.sh의 규칙, 로컬: 건너뜀(로그에 남김), 종료 코드 0
  권장:   ci.sh의 규칙, CI: 실패, 종료 코드 1
```

1번 구역만 실제로 bash를 실행합니다. 2·3번 구역은 2.2절 표에서 가져온 단계 결과로 규칙만 시험합니다.

### 03_feature_matrix — 검사할 기능 조합 세기

```sh
cargo run -p er-ch32-continuous-integration --example 03_feature_matrix
```

```text
1. 조합 수: 부분집합, 실제로 다른 빌드, 줄인 집합
projects/pollcount: 기능 2개, 부분집합 4개, 실제로 다른 빌드 4개, 줄인 집합 4개
    cargo clippy --all-targets --no-default-features
    cargo clippy --all-targets --no-default-features --features csv
    cargo clippy --all-targets --no-default-features --features json
    cargo clippy --all-targets --all-features
독립 기능 다섯: 기능 5개, 부분집합 32개, 실제로 다른 빌드 32개, 줄인 집합 7개
    cargo clippy --all-targets --no-default-features
    cargo clippy --all-targets --no-default-features --features a
    cargo clippy --all-targets --no-default-features --features b
    cargo clippy --all-targets --no-default-features --features c
    cargo clippy --all-targets --no-default-features --features d
    cargo clippy --all-targets --no-default-features --features e
    cargo clippy --all-targets --all-features
std/alloc 계층: 기능 3개, 부분집합 8개, 실제로 다른 빌드 5개, 줄인 집합 5개
    cargo clippy --all-targets --no-default-features
    cargo clippy --all-targets --no-default-features --features alloc,std
    cargo clippy --all-targets --no-default-features --features alloc
    cargo clippy --all-targets --no-default-features --features alloc,json
    cargo clippy --all-targets --all-features
2. std 없는 타깃에서 빌드해야 하는 조합 (std/alloc 계층)
    cargo build --target thumbv6m-none-eabi --no-default-features
    cargo build --target thumbv6m-none-eabi --no-default-features --features alloc
    cargo build --target thumbv6m-none-eabi --no-default-features --features alloc,json
3. CI 작업 수: 기능 조합 × 운영체제 × 툴체인
  projects/pollcount: 모든 조합 4 × 3 × 2 = 24개, 변경마다(줄인 집합, ubuntu, 고정 툴체인) 4개 + MSRV 1개
  독립 기능 다섯: 모든 조합 32 × 3 × 2 = 192개, 변경마다(줄인 집합, ubuntu, 고정 툴체인) 7개 + MSRV 1개
```

"std/alloc 계층"은 `std = ["alloc"]`, `json = ["alloc"]`처럼 한 기능이 다른 기능을 켜는 패키지입니다(ch26 2.6절, ch33). 8개 부분집합 가운데 3개는 다른 것과 같은 빌드라서 실제로 다른 빌드는 5개입니다. 이 패키지에서는 줄인 집합이 곧 모든 빌드입니다.

### projects/pollcount — 검사 스크립트와 워크플로

`pollcount`는 항목 개수를 세는 작은 라이브러리입니다. 기능 `csv`(기본)와 `json`, `rust-version = "1.98"`, 예제 프로그램 `survey`를 갖췄고, 모든 기능을 켜면 문서 테스트 5개, 단위 테스트 3개, 통합 테스트 3개가 돕니다. 독립 워크스페이스이고 의존성이 없어 네트워크 없이 빌드됩니다.

| 파일 | 내용 |
| --- | --- |
| [`ci.sh`](projects/pollcount/ci.sh) | 검사 스크립트(단계 12개) |
| [`ci/github-workflow.yml`](projects/pollcount/ci/github-workflow.yml) | GitHub Actions 워크플로 예시. 실제로는 저장소 루트의 `.github/workflows/`에 두어야 실행됨 |
| [`rust-toolchain.toml`](projects/pollcount/rust-toolchain.toml) | 툴체인 1.100.0 고정 |
| [`deny.toml`](projects/pollcount/deny.toml) | `cargo-deny` 설정(실행해 확인하지 못함) |
| [`CHANGELOG.md`](projects/pollcount/CHANGELOG.md) | `conventions` 단계가 읽는 변경 기록 |

```sh
cd appd-a/ch32-continuous-integration/projects/pollcount
sh ci.sh                  # 모든 단계, 종료 코드 0 (도구 없는 세 단계는 건너뜀)
sh ci.sh --keep-going     # 실패해도 끝까지
sh ci.sh test docs        # 고른 단계만
CI=true sh ci.sh          # CI 모드: 이 저장소에서는 deny 단계에서 실패 (종료 코드 1)
```

`sh ci.sh`의 출력에서 단계 머리줄과 명령, 테스트 결과만 남긴 것입니다. 이 컴퓨터에서 `cargo clean` 뒤 처음 실행해도 3초 안에 끝났습니다.

```text
모드: 로컬, rustc 1.100.0 (0ef1bc386 2026-11-09), cargo 1.100.0 (3acc79c0c 2026-11-05)
==> lockfile
    $ cargo metadata --locked --format-version 1 > /dev/null
==> fmt
    $ cargo fmt --check
==> conventions
    CHANGELOG.md에 '## 0.1.0' 절이 있습니다
==> clippy
    $ cargo clippy --locked --all-targets --all-features
==> test
    $ cargo test --locked --all-targets --all-features
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …
    $ cargo test --locked --doc --all-features
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …
==> examples
    $ cargo run --locked --quiet --all-features --example survey
==> docs
    $ cargo doc --locked --no-deps --all-features
==> features
    $ cargo clippy --locked --all-targets --no-default-features
    $ cargo clippy --locked --all-targets --no-default-features --features csv
    $ cargo clippy --locked --all-targets --no-default-features --features json
==> msrv
    $ cargo +1.98.1 check --locked --all-targets --all-features
==> deny
    cargo-deny 없음: 건너뜀 (설치: cargo install --locked cargo-deny@0.22.0)
==> advisories
    cargo-deny 없음: 건너뜀 (설치: cargo install --locked cargo-deny@0.22.0)
==> hack
    cargo-hack 없음: 건너뜀 (설치: cargo install --locked cargo-hack@0.6.46)
==> 요약
  건너뜀: deny advisories hack
  모든 단계 통과
```

테스트 결과의 첫 세 줄은 단위 테스트, 통합 테스트, 예제의 테스트 하네스(0개)이고, `finished in` 뒤의 시간은 실행마다 달라 줄였습니다. `examples` 단계는 예제의 출력(`응답 9개, 서로 다른 답 4개`, `1위 사과: 4표` …, CSV와 JSON)을 그대로 보여 줍니다. `msrv` 단계는 1.98.1이 설치되어 있지 않으면 로컬에서는 `툴체인 1.98.1 없음: 건너뜀`이 됩니다. 첫 실패에서 멈추는 기본 실행은 2.2절의 MSRV API 사본에서 이렇게 끝났습니다.

```text
[실패] clippy (종료 코드 101)
==> 요약
  실행하지 않음: test examples docs features msrv deny advisories hack
  실패: clippy(101)
```

**워크플로.** [`ci/github-workflow.yml`](projects/pollcount/ci/github-workflow.yml)은 2.5절 표의 작업 여섯(`quick`, `checks`, `test-all-os`, `scheduled`, `latest-deps`, `ci-ok`)으로 되어 있고, 모든 검사는 `sh ci.sh <단계…>`로 부릅니다. 워크플로에 남은 것은 환경 준비(`rustup toolchain install`, MSRV 툴체인, `cargo install`)와 "언제 무엇을"뿐입니다. 이 저장소에서는 프로젝트가 하위 디렉터리에 있어 `defaults.run.working-directory`를 적었습니다. `ruby -ryaml`로 읽으면 맨 위의 `on:` 키가 `true`로 읽힙니다. Ruby의 YAML 1.1 파서가 `on`을 참·거짓 값으로 읽기 때문이고, GitHub 문서의 예도 모두 따옴표 없이 `on:`으로 씁니다.

## 4. 자주 하는 실수

이 Item의 실수는 컴파일 에러가 아니라 검사를 구성하는 방식에서 나오므로 `compile_fail/`을 두지 않았습니다.

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| `cargo test --all-targets`만 돌림 | 실패하는 문서 테스트가 통과 | `cargo test --doc`을 따로 | 2.2 |
| `cargo test --examples`로 예제를 "실행" | `main`이 돌지 않음(`running 0 tests`) | 예제마다 `cargo run --example` | 2.2 |
| 기본 기능으로만 검사 | 한 조합에서만 나는 E0599, 쓰지 않는 가져오기 경고 | 없음 + 하나씩 + 전부 | 2.2, 03 |
| MSRV를 Clippy로만 확인 | 새 언어 기능(`!` 타입)이 통과 | MSRV 툴체인으로 `cargo check` | 2.2 |
| `--locked` 없는 명령을 먼저 실행 | `Cargo.lock`이 조용히 고쳐지고 뒤의 검사가 통과 | `lockfile` 단계를 맨 앞에, 모든 명령에 `--locked` | 2.2 |
| 기본 기능으로 `cargo doc` | 기능 전용 항목으로 가는 링크가 `unresolved link` | docs.rs와 같은 기능(`--all-features`) | 2.2 |
| 워크스페이스에서 `-p`·`--workspace` 없이 검사 | `default-members`에서 빠진 멤버의 경고가 통과 | `--workspace` | 2.2 |
| CI에서 `RUSTFLAGS=-Dwarnings` | 의존성까지 다시 컴파일, 산출물이 두 벌 | `CARGO_BUILD_WARNINGS=deny` | 2.3 |
| MSRV 툴체인에서도 경고를 실패로 | 버전마다 다른 경고(새 린트 이름에 대한 `unknown lint` 등)로 실패 | 경고는 고정한 툴체인에서만 판정 | 2.3 |
| `cargo +1.98`처럼 설치되지 않은 이름을 씀 | rustup이 그 툴체인을 내려받음 | `rustup toolchain list`로 확인한 이름만 | 2.4 |
| `cargo test \| tee log`를 `shell` 없이 | 테스트가 실패해도 단계는 성공 | `shell: bash`(pipefail), 스크립트에서는 파이프로 판정하지 않기 | 2.4, 02 |
| `-e` 없는 스크립트에서 반복문으로 조합을 돎 | 마지막 조합의 결과만 남음 | 반복문 안에서 `\|\| exit`(또는 `\|\| return`) | 2.4, 02 |
| CI에서 없는 도구를 건너뜀 | 아무것도 검사하지 않는 초록 빌드 | `CI=true`면 실패로 | 2.4, 02 |
| 외부 액션을 `@main`이나 태그로 | 남이 옮긴 코드가 다음 실행에 들어옴 | 커밋 SHA와 버전 주석 | 2.6 |

## 5. 연습 문제

### 연습 1. 셸 검사와 린트 중에서 고르기

`pollcount`에 "`src/`에 `dbg!`가 남아 있으면 안 된다"는 관례를 더하려고 합니다. `ci.sh`의 `conventions` 단계에 `grep`으로 넣는 방법과 Clippy 린트로 넣는 방법을 각각 해 보고, 어느 쪽이 나은지 이유를 드세요.

<details>
<summary>힌트</summary>

Clippy에는 `dbg_macro` 린트가 있지만 restriction 그룹이라 기본으로 꺼져 있습니다(`clippy-driver -W help`에서 `clippy::dbg-macro  allow`). `Cargo.toml`에 `[lints.clippy]`와 `dbg_macro = "warn"`을 더하면 `clippy` 단계가 `` warning: the `dbg!` macro is intended as a debugging tool `` 다음 `` error: warnings are denied by `build.warnings` configuration ``으로 실패합니다(종료 코드 101, 사본에서 확인). `ci.sh`가 `build.warnings`를 켜 두었으므로 `"deny"`가 아니라 `"warn"`이면 충분하고, 개발 중에는 경고로만 보입니다. `grep -rn 'dbg!' src`는 주석이나 문서의 `dbg!`까지 잡고, 매크로 이름을 바꿔 쓴 경우는 못 잡습니다. 린트가 있는 관례는 린트로, 어떤 린트도 볼 수 없는 것(변경 기록 파일, 저장소 구조)만 셸 검사로 둡니다.

</details>

### 연습 2. 기능 하나 더하기

`pollcount`에 탭으로 나눈 출력 `to_tsv`를 기능 `tsv`로 더하세요. `ci.sh`와 워크플로에서 고쳐야 할 곳이 있나요? 기능 조합은 몇 개가 되고, `features` 단계와 `clippy` 단계는 cargo를 몇 번 부르나요?

<details>
<summary>힌트</summary>

`ci.sh`의 `feature_names`는 `Cargo.toml`의 `[features]` 표에서 이름을 읽으므로 스크립트와 워크플로는 고칠 곳이 없습니다. 문서의 기능 목록과 `` [`Votes::to_tsv`] `` 링크는 더해야 하고, `to_tsv`가 `write!`를 쓴다면 `use std::fmt::Write as _;`의 `cfg(any(…))`에 `feature = "tsv"`도 넣어야 합니다(빠뜨리면 2.2절 표의 "기능 조합 하나에서만 나는 에러"가 `--features tsv`에서 납니다). 모든 조합은 2^3 = 8개이고, `features` 단계는 기능 없음과 하나씩으로 4번, `clippy` 단계가 `--all-features`로 1번, 합쳐 5번입니다. 예제 03의 `POLLCOUNT`에 `feature("tsv", &[])`를 더하면 같은 수(부분집합 8개, 줄인 집합 5개)가 나옵니다.

</details>

### 연습 3. MSRV 올리기

`pollcount`의 MSRV를 1.99로 올리기로 했습니다. 바꿔야 할 곳을 모두 찾고, 하나씩 빠뜨리면 어느 검사가 알려 주는지 예측하세요.

<details>
<summary>힌트</summary>

세 곳입니다. `Cargo.toml`의 `rust-version`, `ci.sh`의 `MSRV_TOOLCHAIN`, 워크플로 `checks` 매트릭스의 `setup`(1.99.x 툴체인 설치)입니다. `rust-version`만 바꾸면 `msrv` 단계가 `Cargo.toml의 rust-version(1.99)과 MSRV_TOOLCHAIN(1.98.1)이 다릅니다`를 출력하고 종료 코드 1로 실패합니다(사본에서 확인). 스크립트까지 바꾸고 워크플로를 빠뜨리면 CI의 `msrv` 작업에서 그 툴체인이 설치되지 않아 `CI에서는 건너뛸 수 없습니다`로 실패합니다. 로컬에서 새 툴체인을 설치하지 않았다면 `msrv` 단계는 건너뜀으로 끝납니다. `rust-version`을 올리면 Clippy의 `incompatible_msrv` 기준도 함께 올라가므로, 그 사이에 안정화된 API를 쓸 수 있게 됩니다. 변경 기록에 MSRV를 올렸다고 적는 것도 잊지 않습니다(ch21 2.7절).

</details>

## 6. 참고 자료

- 기준 문서: [Item 32: Set up a continuous integration (CI) system](https://effective-rust.com/ci.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- Cargo Book, [Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html): GitHub Actions 예, Verifying Latest Dependencies(`CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS`), Verifying `rust-version`, Checking for Warnings
- Cargo Book, [Configuration](https://doc.rust-lang.org/cargo/reference/config.html)의 `build.warnings`(로컬 문서에서 확인)
- rustup 문서, [Overrides](https://rust-lang.github.io/rustup/overrides.html): 툴체인을 고르는 우선순위, `rust-toolchain.toml`의 키
- GitHub Docs, [Workflow syntax for GitHub Actions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax): `on.schedule`, `permissions`, `concurrency`, `strategy`, `continue-on-error`, `timeout-minutes`, 기본 셸
- GitHub Docs, [Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use): SHA 고정, 토큰 권한, `pull_request_target`, 스크립트 주입
- GitHub Docs, [Variables reference](https://docs.github.com/en/actions/reference/workflows-and-actions/variables)(`CI`)와 [Approving workflow runs from forks](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/approve-runs-from-forks)
- [`actions/checkout`](https://github.com/actions/checkout) README: `persist-credentials`
- Cargo 저장소의 워크플로 [`main.yml`](https://github.com/rust-lang/cargo/blob/master/.github/workflows/main.yml): SHA 고정, `conclusion` 작업, `cargo hack check --rust-version … --locked`
