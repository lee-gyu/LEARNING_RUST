# Part 2. 에디션

[The Rust Edition Guide](https://doc.rust-lang.org/edition-guide/index.html)를 따라 에디션(edition)이 무엇이고 각 에디션에서 무엇이 바뀌었는지를 비교 예제로 확인하는 파트입니다. Part 0과 Part 1에서 "2021 에디션부터", "2024 에디션에서 바뀌었다"고만 짚고 넘어간 변화들을 에디션이라는 제도와 함께 정리하고, 오래된 코드를 읽고 최신 에디션으로 옮길 수 있게 되는 것이 목표입니다.

[Part 0](../part0-getting-started/)과 [Part 1](../part1-standard-lib/)의 내용을 전제로 합니다.

## 학습 방법

챕터 번호 순서대로 진행합니다. 각 챕터의 `README.md`를 읽으면서 예제를 직접 실행해 봅니다.

```sh
cargo run -p <패키지 이름> --example <예제 이름>
```

패키지 이름과 예제 이름은 각 챕터 README의 "예제 안내"에 적혀 있습니다. `examples/`의 예제는 이 저장소의 기본 에디션(2024)으로 컴파일됩니다. 다른 에디션으로 컴파일해야 하는 비교 코드는 챕터의 `projects/` 아래에 에디션을 따로 지정한 독립 프로젝트로 들어 있으며, 그 디렉터리로 이동해서 실행합니다. 실행 방법은 각 챕터 README에 적혀 있습니다.

## 챕터 목차

| 챕터 | 주제 |
| --- | --- |
| [ch1-editions-overview](ch1-editions-overview/) | 에디션의 목적과 하위 호환성, 에디션이 바꾸는 것과 바꾸지 않는 것, 에디션과 컴파일러 버전(`rust-version`)의 구분, `edition` 필드와 `rustc --edition`, 에디션이 다른 크레이트 섞어 쓰기(`edition_mix`), 매크로와 에디션 |
| [ch2-rust-2015-2018](ch2-rust-2015-2018/) | 2015 기준선의 옛 문법 읽기(`extern crate`, `mod.rs`, `try!`, 익명 트레이트 매개변수, `dyn` 없는 트레이트 객체, `...` 패턴), 2018의 경로·모듈 시스템 변경, 새 키워드(`async`/`await`/`dyn`/`try`), raw 식별자 — 같은 프로그램의 2015판과 2018판(`scoreboard`) |
| [ch3-rust-2021](ch3-rust-2021/) | 2018 → 2021의 변화: 프렐류드 추가(`TryFrom`/`TryInto`/`FromIterator`), 기능 리졸버 2, 배열 `IntoIterator`, 클로저 분리 캡처, `panic!` 일관성, 예약된 접두사와 C 문자열·raw 라이프타임, 에러로 승격된 경고, `:pat` — 같은 소스를 두 에디션으로 비교(`same_code`), 기능 통합 비교(`feature_resolver`) |
| [ch4-rust-2024-language](ch4-rust-2024-language/) | 2021 → 2024의 언어 변화: RPIT 라이프타임 캡처와 `use<..>`, `if let`·마지막 표현식의 임시값 범위, let 체인, `unsafe extern`과 unsafe 속성, `static mut` 참조 금지, never 타입 폴백, `gen` 예약, 매크로 프래그먼트·예약 문법 — 같은 소스를 두 에디션으로 비교(`edition_compare`) |
| [ch5-rust-2024-ecosystem](ch5-rust-2024-ecosystem/) | 언어 밖의 2024 변화: 네 도구가 에디션을 읽는 곳, 프렐류드의 `Future`/`IntoFuture`, `Box<[T]>`의 `IntoIterator`, unsafe가 된 `set_var`/`remove_var`/`before_exec`, 리졸버 3과 Cargo 키 이름, 문서 테스트 병합, rustfmt 스타일 에디션 — 두 에디션 비교(`ecosystem_compare`), 리졸버 실습(`msrv_resolver`) |
| [ch6-migration-practice](ch6-migration-practice/) | `cargo fix --edition`의 절차와 원리, 자동 수정 뒤의 다듬기, 수동 수정이 필요한 경우(자동 수정이 없는 린트, 린트가 놓치는 동작 변화, 매크로·문서 테스트·꺼진 `cfg`), 고급 마이그레이션 — 2018 프로젝트를 2021, 2024로 단계별로 옮기기(`migration`) |

Part 2의 6개 챕터가 모두 작성됐습니다. 전체 계획은 [로드맵](../docs/roadmap.md)을, 진행 상황은 [PROGRESS.md](../PROGRESS.md)를 참고합니다.
