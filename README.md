# LEARNING_RUST

한국어 사용자를 위한 Rust 학습 저장소입니다. [rust-lang.org/learn](https://rust-lang.org/learn/)의 공식 학습 자료를 순서대로 따라가며, 챕터마다 설명 문서(`README.md`)와 직접 실행해 볼 수 있는 실습 예제(`.rs`)를 제공합니다.

- 전체 계획과 저장소 규칙: [docs/roadmap.md](docs/roadmap.md)
- 진척 현황: [PROGRESS.md](PROGRESS.md)

## 준비

Rust 툴체인(stable, Edition 2024)이 필요합니다. [mise](https://mise.jdx.dev/)를 쓴다면 저장소 루트에서 다음을 실행합니다.

```sh
mise install
```

mise를 쓰지 않는다면 [rustup](https://rustup.rs/)으로 설치합니다.

## 학습 방법

파트와 챕터를 번호 순서대로 진행합니다. 챕터 디렉터리의 `README.md`를 읽고, 안내에 따라 예제를 실행합니다.

```sh
# 챕터의 예제 하나 실행
cargo run -p p0-ch1-hello-cargo --example 01_hello_world

# 일부러 컴파일 에러를 내는 코드 확인 (챕터 디렉터리에서)
rustc --edition 2024 compile_fail/<파일>.rs
```

`projects/` 아래의 독립 프로젝트는 해당 디렉터리로 이동해서 `cargo run`, `cargo test` 등을 실행합니다.

## 구성

| 파트 | 디렉터리 | 기준 문서 |
| --- | --- | --- |
| Part 0. 시작하기 | [part0-getting-started/](part0-getting-started/) | The Book, Rust By Example, Rustlings |

나머지 파트(Part 1~11)는 [로드맵](docs/roadmap.md)의 계획에 따라 순서대로 추가됩니다.
