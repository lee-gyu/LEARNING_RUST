# Part 3. Cargo

[The Cargo Book](https://doc.rust-lang.org/cargo/index.html)을 따라 Rust의 패키지 관리자이자 빌드 시스템인 Cargo를 다루는 파트입니다. Part 0부터 모든 예제를 Cargo로 실행해 왔지만, 지금까지는 `cargo run`과 `cargo add` 정도만 썼습니다. 이 파트에서는 패키지를 만들고, 의존성을 고르고, 매니페스트와 워크스페이스를 구성하고, 빌드를 설정하고, 테스트하고 게시하는 데까지 Cargo가 하는 일을 처음부터 끝까지 살펴봅니다.

[Part 0](../part0-getting-started/)의 내용을 전제로 합니다. [Part 1](../part1-standard-lib/)과 [Part 2](../part2-editions/)는 필요한 곳에서 링크로 가리킵니다.

## 학습 방법

챕터 번호 순서대로 진행합니다. 각 챕터의 `README.md`를 읽으면서 예제를 직접 실행해 봅니다.

```sh
cargo run -p <패키지 이름> --example <예제 이름>
```

패키지 이름과 예제 이름은 각 챕터 README의 "예제 안내"에 적혀 있습니다.

이 파트의 실습은 대부분 챕터의 `projects/` 아래에 있는 독립 프로젝트입니다. Cargo 자체를 다루려면 `Cargo.toml`을 여러 모양으로 만들어 봐야 하는데, 이 저장소의 워크스페이스 안에서는 그럴 수 없기 때문입니다. 독립 프로젝트는 그 디렉터리로 이동해서 실행하고, 고쳐 보는 실습은 저장소 밖으로 복사한 사본에서 합니다. 실행 방법은 각 챕터 README에 적혀 있습니다. crates.io에서 크레이트를 내려받는 프로젝트는 처음 빌드할 때 네트워크가 필요합니다.

## 챕터 목차

| 챕터 | 주제 |
| --- | --- |
| [ch1-getting-started](ch1-getting-started/) | Cargo가 `rustc` 위에서 하는 일, `cargo new`/`cargo init`, 패키지 레이아웃 관례와 타깃 고르기, `Cargo.toml`과 `Cargo.lock`(`--locked`), 이미 있는 패키지 빌드하기, Cargo home — 관례대로 놓은 패키지(`text-stat`) |
| [ch2-dependencies](ch2-dependencies/) | 버전 요구 사항 문법(기본·캐럿, 틸드, 와일드카드, 비교, 프리릴리스), crates.io·git·path 세 출처와 잠금 파일의 `source`, 의존성 표의 종류와 이름 바꾸기, `cargo add`·`remove`·`update`·`tree`, 의존성 해석 규칙과 해석 실패 읽기, `[patch]` — 세 출처를 쓰는 패키지(`three-sources`), 연습용 레지스트리(`resolver-lab`) |
| [ch3-manifest-targets](ch3-manifest-targets/) | 매니페스트의 표 지도, `[package]`의 키와 모르는 키 경고·`[package.metadata]`, 타깃 표(`[lib]`·`[[bin]]`·`[[example]]`·`[[test]]`)와 `default-run`·`test`·`harness`·자동 탐색, `rust-version`의 효과, `[lints]`의 수준과 `priority`·Cargo 자신의 린트 — 타깃을 직접 구성한 패키지(`unit-tools`) |
| [ch4-workspaces](ch4-workspaces/) | 워크스페이스가 함께 쓰는 것(`Cargo.lock`·`target/`·명령), 루트 패키지형과 가상 매니페스트, `members`·`exclude`·`default-members`와 `-p`·`--workspace`, 멤버 간 path 의존과 다시 컴파일되는 범위, `[workspace.package]`·`[workspace.dependencies]`·`[workspace.lints]` 상속, 루트에서만 읽는 설정 — 패키지 셋으로 나눈 프로그램(`bookshelf`) |
| [ch5-features](ch5-features/) | `[features]`와 `default`, `#[cfg(feature)]`와 `cfg!`, `--features`·`--no-default-features`·`--all-features`와 의존성의 `features`·`default-features`, `required-features`, 선택적 의존성과 `dep:`·`?`, 기능 통합과 `cargo tree -e features`, 가산적인 기능 설계 — 기능을 서로 다르게 켜는 워크스페이스(`formats`) |
| [ch6-profiles-build-performance](ch6-profiles-build-performance/) | 내장 프로필과 명령별 선택, 프로필 설정(`opt-level`·`debug`·`strip`·`debug-assertions`·`overflow-checks`·`lto`·`codegen-units`·`panic`·`incremental`), `inherits`와 `--profile`, 패키지별 재정의와 `build-override`·제네릭의 한계, target 디렉터리와 build 디렉터리, `.d` 파일과 `Dirty`, `--timings` 보고서, 빌드 시간 단축 방법 — 프로필만 바꿔 크기와 속도를 비교하는 프로젝트(`collatz`) |
| [ch7-configuration-env](ch7-configuration-env/) | 설정 파일을 찾는 곳과 합치는 규칙, 설정 파일·`CARGO_<키>` 환경 변수·`--config`의 우선순위, `[alias]`, `[env]`·`build.rustflags`·`[term]` 등 설정 키, Cargo가 읽는 환경 변수, Cargo가 크레이트에 알려 주는 환경 변수와 `env!`·`option_env!`·`std::env::var` — 설정 파일 둘을 겹쳐 쓰는 프로젝트(`configured`) |
| [ch8-build-scripts](ch8-build-scripts/) | 빌드 스크립트의 한살이와 입력(환경 변수)·출력(`OUT_DIR`, 지시문), 데이터 파일로 코드 생성과 `include!`, `cargo::` 지시문과 `rerun-if-changed`, `-vv`로 출력 보기, `cc`로 C 라이브러리 링크, `links`와 `DEP_<links>_<키>`, `-sys` 관례와 빌드 스크립트 재정의 — 챕터 패키지의 `build.rs`, C 라이브러리를 감싼 프로젝트(`fletcher`) |

전체 계획은 [로드맵](../docs/roadmap.md)을, 진행 상황은 [PROGRESS.md](../PROGRESS.md)를 참고합니다.
