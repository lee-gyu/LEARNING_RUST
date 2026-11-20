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

전체 계획은 [로드맵](../docs/roadmap.md)을, 진행 상황은 [PROGRESS.md](../PROGRESS.md)를 참고합니다.
