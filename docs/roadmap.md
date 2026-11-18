# Rust 학습 로드맵

[rust-lang.org/learn](https://rust-lang.org/learn/)에 정리된 공식 학습 자료를 순서대로 따라가며, 챕터마다 **설명 문서(README.md)** 와 **실습 예제(.rs)** 를 작성하기 위한 전체 계획입니다.

- 기준 툴체인: Rust 1.98 (stable), Edition 2024 — `mise.toml`의 `rust = "latest"`
- 전체 규모: 12개 파트, 109개 챕터
- 작업 단위: **챕터 1개 = 작업 1회**. 완료하면 이 문서의 상태 칸을 ⬜ → ✅ 로 바꿉니다.

## 1. 전체 구성

공식 사이트의 4개 단계를 그대로 파트 순서로 사용합니다.

| 단계 (공식 사이트 구분) | 파트 | 디렉터리 | 기준 문서 | 챕터 수 |
| --- | --- | --- | --- | --- |
| Get started | Part 0 | `part0-getting-started/` | [The Book](https://doc.rust-lang.org/book/), [Rust By Example](https://doc.rust-lang.org/rust-by-example/), [Rustlings](https://github.com/rust-lang/rustlings/) | 20 |
| Core documentation | Part 1 | `part1-standard-lib/` | [The standard library](https://doc.rust-lang.org/std/index.html) | 14 |
| | Part 2 | `part2-editions/` | [Edition Guide](https://doc.rust-lang.org/edition-guide/index.html) | 6 |
| | Part 3 | `part3-cargo/` | [Cargo Book](https://doc.rust-lang.org/cargo/index.html) | 11 |
| | Part 4 | `part4-rustdoc/` | [rustdoc Book](https://doc.rust-lang.org/rustdoc/index.html) | 4 |
| | Part 5 | `part5-rustc/` | [rustc Book](https://doc.rust-lang.org/rustc/index.html) | 7 |
| | Part 6 | `part6-error-index/` | [Compiler Error Index](https://doc.rust-lang.org/error-index.html) | 6 |
| Application domain | Part 7 | `part7-cli/` | [Command Line Book](https://rust-cli.github.io/book/index.html) | 7 |
| | Part 8 | `part8-embedded/` | [Embedded Book](https://doc.rust-lang.org/embedded-book) | 8 |
| Master Rust | Part 9 | `part9-reference/` | [The Reference](https://doc.rust-lang.org/reference/index.html) | 10 |
| | Part 10 | `part10-nomicon/` | [The Rustonomicon](https://doc.rust-lang.org/nomicon/index.html) | 12 |
| | Part 11 | `part11-unstable/` | [The Unstable Book](https://doc.rust-lang.org/nightly/unstable-book/index.html) | 4 |

**학습 순서**: Part 0 → 11 순서대로 진행합니다. Part 0은 이후 모든 파트의 선수 과정이며, Part 7·8(응용 분야)은 서로 독립적이라 순서를 바꿔도 됩니다. Part 10은 Part 9의 ch7·ch10을, Part 11은 nightly 툴체인을 전제로 합니다.

## 2. 저장소 구조와 규칙

### 2.1 디렉터리 구조

```
LEARNING_RUST/
├── Cargo.toml                      # 워크스페이스 루트 (members = ["part*/ch*"])
├── docs/roadmap.md                 # 이 문서
├── part0-getting-started/
│   ├── README.md                   # 파트 개요 + 챕터 목차
│   ├── ch1-hello-cargo/
│   │   ├── README.md               # 설명 문서
│   │   ├── Cargo.toml              # 챕터 = 워크스페이스 멤버 패키지
│   │   ├── examples/               # 실습 예제 (.rs) — 파일 하나가 실행 단위 하나
│   │   │   ├── 01_hello_world.rs
│   │   │   └── 02_....rs
│   │   ├── compile_fail/           # (선택) 일부러 컴파일 에러를 내는 코드
│   │   └── projects/               # (선택) 독립된 Cargo 프로젝트가 필요한 실습
│   └── ch2-guessing-game/
├── part1-standard-lib/
│   └── ch1-...
└── ...
```

### 2.2 이름 규칙

| 대상 | 규칙 | 예시 |
| --- | --- | --- |
| 파트 디렉터리 | `part{N}-{slug}` | `part3-cargo` |
| 챕터 디렉터리 | `ch{N}-{slug}` (kebab-case) | `ch4-ownership` |
| 패키지 이름 | `p{파트}-ch{챕터}-{slug}` | `p0-ch4-ownership` |
| 예제 파일 | `examples/{NN}_{snake_case}.rs` | `examples/02_borrowing.rs` |
| 컴파일 에러 예제 | `compile_fail/{snake_case}.rs` | `compile_fail/e0382_use_after_move.rs` |
| 독립 프로젝트 | `projects/{name}/` | `projects/minigrep/` |

### 2.3 예제 작성 규칙

- **기본 형태**: `examples/*.rs` 단일 파일. 각 파일은 `fn main()`을 가지며 다른 예제에 의존하지 않습니다.
- **실행**: `cargo run -p p0-ch4-ownership --example 02_borrowing`
- **주석은 한국어**로 작성하고, 출력 결과를 `// 출력: ...` 형태로 남깁니다.
- **외부 크레이트**가 필요하면 해당 챕터의 `Cargo.toml`에 `[dev-dependencies]`로 추가합니다.
- **컴파일 에러를 보여주는 코드**는 `compile_fail/`에 둡니다. Cargo가 빌드 대상으로 인식하지 않으므로 워크스페이스 빌드를 깨지 않습니다. 확인 방법: `rustc --edition 2024 compile_fail/e0382_use_after_move.rs`
- **독립 프로젝트** (`projects/*`)는 여러 파일로 된 프로그램, 다른 에디션/타깃, Cargo 자체 실습에 사용합니다. 자신의 `Cargo.toml`에 빈 `[workspace]` 테이블을 넣어 루트 워크스페이스와 분리합니다.

챕터 `Cargo.toml` 기본형:

```toml
[package]
name = "p0-ch4-ownership"
version = "0.0.0"
edition.workspace = true
publish.workspace = true
```

### 2.4 설명 문서(README.md) 구성

모든 챕터 README는 아래 순서를 따릅니다.

1. **학습 목표** — 이 챕터를 마치면 할 수 있는 것 3~5개
2. **핵심 개념** — 개념별 설명과 짧은 코드 조각
3. **예제 안내** — `examples/` 파일별 설명, 실행 명령, 기대 출력
4. **자주 하는 실수** — 대표 컴파일 에러와 해결 방법 (`compile_fail/` 연결)
5. **연습 문제** — 직접 풀어볼 과제 2~4개
6. **참고 자료** — 기준 문서의 해당 절 링크

### 2.5 챕터 작업 절차

1. 이 문서에서 다음 ⬜ 챕터를 고릅니다.
2. "기준 문서 절"에 적힌 공식 문서를 읽고 내용을 확인합니다. (현재 버전 기준으로 사실 확인)
3. 챕터 디렉터리와 `Cargo.toml`을 만듭니다.
4. 예제를 작성하고 하나씩 실행해 출력을 확인합니다.
5. README.md를 작성합니다.
6. 검증합니다.
   ```sh
   cargo build -p <패키지> --examples
   cargo clippy -p <패키지> --examples
   cargo fmt -p <패키지> --check
   ```
7. 이 문서의 상태를 ✅ 로 바꾸고, 파트 README 목차에 챕터를 추가합니다.

### 2.6 사전 작업 (첫 챕터 전에 1회)

| 상태 | 작업 |
| --- | --- |
| ✅ | 루트 `Cargo.toml` 작성 (`[workspace]`, `resolver = "3"`, `members = ["part*/ch*"]`, `[workspace.package]`에 `edition = "2024"`, `publish = false`) |
| ✅ | `.gitignore` 작성 (`target/`) |
| ✅ | 루트 `README.md` 작성 (저장소 소개, 로드맵 링크, 실행 방법) |

파트별 추가 준비물은 각 파트의 "준비" 항목에 적었습니다.

---

## 3. 파트별 챕터 계획

### Part 0. 시작하기 — `part0-getting-started/`

언어의 기초 전체를 다룹니다. The Book의 장 순서를 따르고, 각 챕터 README에 대응하는 Rust By Example 절과 Rustlings 연습 주제를 함께 안내합니다. The Book 14장(Cargo와 crates.io)은 Part 3에서 다룹니다.

- 준비: ch16은 비동기 런타임 크레이트(`trpl` 또는 `tokio`)가 필요합니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch1-hello-cargo` | 설치(rustup/mise), `rustc`로 컴파일, `cargo new/build/run/check` | Hello World, 서식 출력 `println!` | Book 1 |
| ✅ | `ch2-guessing-game` | 입력 받기, `let`/`mut`, 외부 크레이트(`rand`), `match`, 반복 | 단계별 숫자 맞히기 게임 (4단계) | Book 2 |
| ✅ | `ch3-common-concepts` | 변수와 가변성, 섀도잉, 스칼라/복합 타입, 함수, 표현식과 구문, 제어 흐름 | 변수, 타입, 함수, `if`/`loop`/`while`/`for`, 피보나치·온도 변환 | Book 3 |
| ✅ | `ch4-ownership` | 소유권 규칙, 이동과 복사, 참조와 대여, 슬라이스 | 이동/복제, 불변·가변 대여, 문자열 슬라이스, 대여 규칙 위반(`compile_fail`) | Book 4 |
| ✅ | `ch5-structs` | 구조체 정의, 튜플 구조체, 메서드와 연관 함수, `derive(Debug)` | 구조체 생성/갱신 문법, 사각형 넓이 프로그램, `impl` 블록 | Book 5 |
| ✅ | `ch6-enums-pattern-matching` | 열거형, `Option<T>`, `match`, `if let`, `let...else` | 데이터를 가진 열거형, `Option` 처리, 패턴 바인딩 | Book 6 |
| ✅ | `ch7-modules` | 패키지/크레이트/모듈, 경로, `pub`, `use`, 파일 분리 | 인라인 모듈 예제 + 파일 분리 프로젝트(`projects/restaurant`) | Book 7 |
| ✅ | `ch8-collections` | `Vec<T>`, `String`과 UTF-8, `HashMap<K, V>` | 벡터 순회/수정, 문자열 결합·인덱싱, 단어 빈도수, 연습(중앙값·피그 라틴) | Book 8 |
| ✅ | `ch9-error-handling` | `panic!`, `Result<T, E>`, `?` 연산자, 패닉과 `Result`의 선택 기준 | 파일 열기 에러 처리, 에러 전파, 검증 타입(`Guess`) | Book 9 |
| ✅ | `ch10-generics-traits-lifetimes` | 제네릭, 트레이트 정의/구현, 트레이트 바운드, 라이프타임 명시 | 제네릭 `largest`, `Summary` 트레이트, `impl Trait`, `longest` 라이프타임 | Book 10 |
| ✅ | `ch11-testing` | `#[test]`, 단언 매크로, `should_panic`, 테스트 실행 옵션, 단위/통합 테스트 | 테스트가 포함된 라이브러리 프로젝트(`projects/adder`) | Book 11 |
| ✅ | `ch12-io-project-minigrep` | 인자 파싱, 파일 읽기, 모듈화, TDD, 환경 변수, 표준 에러 | 단계별 `minigrep` 프로젝트(`projects/minigrep`) | Book 12 |
| ✅ | `ch13-closures-iterators` | 클로저 캡처와 `Fn` 트레이트, `Iterator`, 어댑터, 성능 | 클로저 캡처 방식 3종, 반복자 어댑터 체인, minigrep 개선 | Book 13 |
| ✅ | `ch14-smart-pointers` | `Box<T>`, `Deref`, `Drop`, `Rc<T>`, `RefCell<T>`, 순환 참조와 `Weak<T>` | cons 리스트, 직접 만든 `MyBox`, 내부 가변성, 트리 구조 | Book 15 |
| ✅ | `ch15-concurrency` | 스레드, `move` 클로저, 채널, `Mutex<T>`/`Arc<T>`, `Send`/`Sync` | 스레드 생성·join, 메시지 전달, 공유 카운터 | Book 16 |
| ✅ | `ch16-async` | `async`/`await`, 퓨처, 동시 실행, 스트림, `Pin`, 태스크와 스레드 | 비동기 함수, `join`/`race`, 채널, 스트림 처리 | Book 17 |
| ✅ | `ch17-trait-objects-oop` | 캡슐화, 트레이트 객체(`dyn Trait`), 상태 패턴 | GUI 컴포넌트 예제, 블로그 상태 패턴(객체형/타입형 2가지) | Book 18 |
| ✅ | `ch18-patterns` | 패턴이 쓰이는 위치, 반박 가능성, 패턴 문법 전체 | 구조 분해, 가드, `@` 바인딩, 범위·무시 패턴 | Book 19 |
| ✅ | `ch19-advanced-features` | `unsafe` 기초, 연관 타입, 연산자 오버로딩, 뉴타입, 함수 포인터, 매크로 | 원시 포인터, `Add` 구현, `macro_rules!`, derive 매크로(`projects/hello_macro`) | Book 20 |
| ✅ | `ch20-web-server-project` | TCP 연결 처리, 스레드 풀, 우아한 종료 | 멀티스레드 웹 서버(`projects/hello-server`) | Book 21 |

### Part 1. 표준 라이브러리 — `part1-standard-lib/`

`std` API 문서를 모듈 단위로 읽고 실제로 사용해 봅니다. Part 0에서 맛본 타입들의 메서드를 넓고 깊게 익히는 것이 목표입니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch1-std-overview` | `core`/`alloc`/`std` 구분, prelude, 원시 타입 메서드, API 문서 읽는 법 | 정수·실수·`char`·`bool`·배열·슬라이스·튜플 메서드 | std 첫 페이지, primitive types, `std::prelude` |
| ✅ | `ch2-option-result` | `Option`/`Result` 콤비네이터, 상호 변환, `?`와의 관계 | `map`/`and_then`/`unwrap_or_else`/`ok_or`, `transpose`, 수집(`collect`) | `std::option`, `std::result` |
| ✅ | `ch3-strings-fmt` | `String`/`&str`/`char`, 서식 문법, `Display`/`Debug`, `FromStr` | 문자열 검색·분할·변환, 서식 지정자, `Display` 구현, 파싱 | `std::string`, `std::str`, `std::char`, `std::fmt` |
| ✅ | `ch4-collections` | `Vec`, `VecDeque`, `HashMap`/`BTreeMap`, `HashSet`/`BTreeSet`, `BinaryHeap`, 선택 기준 | 컬렉션별 주요 연산, entry API, 정렬·이진 탐색, 우선순위 큐 | `std::collections`, `std::vec`, `std::slice` |
| ⬜ | `ch5-iterators` | `Iterator`/`IntoIterator`/`FromIterator`, 어댑터와 소비자, 반복자 구현 | 어댑터 모음, `fold`/`scan`/`zip`/`chain`/`peekable`, 직접 만든 반복자 | `std::iter` |
| ⬜ | `ch6-core-traits` | 변환(`From`/`Into`/`TryFrom`/`AsRef`), 연산자, 비교, 해시, `Default`, `Clone`/`Copy`, `Borrow`/`Cow` | 트레이트별 구현 예제, `Cow`로 불필요한 복사 줄이기 | `std::convert`, `std::ops`, `std::cmp`, `std::hash`, `std::default`, `std::borrow`, `std::marker` |
| ⬜ | `ch7-smart-pointers-memory` | `Box`, `Rc`/`Weak`, `Cell`/`RefCell`/`OnceCell`, `std::mem` 함수 | `mem::swap`/`take`/`replace`, 내부 가변성 패턴, 크기·정렬 확인 | `std::boxed`, `std::rc`, `std::cell`, `std::mem` |
| ⬜ | `ch8-errors-panic` | `Error` 트레이트, 에러 타입 설계, `source` 체인, `Box<dyn Error>`, 패닉 훅 | 사용자 정의 에러, 에러 변환(`From`), `catch_unwind`, 백트레이스 | `std::error`, `std::panic`, `std::backtrace` |
| ⬜ | `ch9-io-fs-path` | `Read`/`Write`/`BufRead`/`Seek`, 표준 입출력, 파일·디렉터리, `Path`/`PathBuf` | 줄 단위 읽기, 버퍼링 쓰기, 디렉터리 순회, 경로 조작 | `std::io`, `std::fs`, `std::path` |
| ⬜ | `ch10-env-process-time` | 인자·환경 변수, 자식 프로세스, 종료 코드, `Instant`/`Duration`/`SystemTime` | `Command` 실행·파이프, 실행 시간 측정 | `std::env`, `std::process`, `std::time` |
| ⬜ | `ch11-threads-sync` | 스레드와 스코프 스레드, `Mutex`/`RwLock`/`Condvar`, `Arc`, `OnceLock`/`LazyLock`, 채널, 원자 타입 | 스코프 스레드, 생산자-소비자, 전역 지연 초기화, `AtomicUsize` 카운터 | `std::thread`, `std::sync`, `std::sync::mpsc`, `std::sync::atomic` |
| ⬜ | `ch12-net` | TCP/UDP 소켓, 주소 타입 | 에코 서버·클라이언트, UDP 송수신 | `std::net` |
| ⬜ | `ch13-async-primitives` | `Future` 트레이트, `Poll`, `Context`/`Waker`, `Pin` | 직접 구현한 퓨처, 최소 실행기(`block_on`) 만들기 | `std::future`, `std::task`, `std::pin` |
| ⬜ | `ch14-ffi-any-misc` | `CString`/`CStr`, `OsString`, `Any`와 다운캐스팅, 수치 타입(`NonZero`, `Wrapping`) | C 문자열 변환, `dyn Any` 다운캐스트, 오버플로 처리 방식 | `std::ffi`, `std::any`, `std::num` |

### Part 2. 에디션 — `part2-editions/`

에디션이 무엇이고 각 에디션에서 무엇이 바뀌었는지를 비교 예제로 확인합니다.

- 준비: 에디션별 차이를 보여주는 예제는 `projects/` 아래에 에디션을 다르게 지정한 독립 프로젝트로 만듭니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-editions-overview` | 에디션의 목적과 호환성 보장, `edition` 필드, 에디션 간 크레이트 혼용 | 에디션이 다른 두 크레이트를 함께 링크하는 프로젝트 | Edition Guide 1, 1.1 |
| ⬜ | `ch2-rust-2015-2018` | 2015 기준선, 경로·모듈 시스템 변경, 익명 트레이트 매개변수, 새 키워드(`async`/`await`/`dyn`), raw 식별자 | 같은 코드의 2015판/2018판 비교 | Edition Guide 2, 3 |
| ⬜ | `ch3-rust-2021` | prelude 추가, 기능 리졸버 v2, 배열 `IntoIterator`, 클로저 분리 캡처, 패닉 매크로 일관성, 예약 문법, C 문자열 리터럴 | 2018판/2021판 동작 차이 비교 | Edition Guide 4 |
| ⬜ | `ch4-rust-2024-language` | RPIT 라이프타임 캡처, `if let`·꼬리 표현식 임시값 범위, let 체인, `unsafe extern`, unsafe 속성, `static mut` 참조 금지, never 타입 폴백, `gen` 키워드 | 2021판/2024판 동작 차이 비교 | Edition Guide 5.1 |
| ⬜ | `ch5-rust-2024-ecosystem` | prelude 변경, `Box<[T]>` `IntoIterator`, 새로 unsafe가 된 함수, rust-version 인식 리졸버, rustdoc 통합 테스트, rustfmt 스타일 에디션 | 표준 라이브러리·Cargo·rustfmt 변경 확인 | Edition Guide 5.2~5.5 |
| ⬜ | `ch6-migration-practice` | `cargo fix --edition`, 수동 수정이 필요한 경우, 고급 마이그레이션 | 2018 프로젝트를 2021 → 2024로 단계별 이전 | Edition Guide 1.2, 1.3 |

### Part 3. Cargo — `part3-cargo/`

패키지 관리자이자 빌드 시스템인 Cargo를 다룹니다. 대부분의 실습이 `projects/` 아래의 독립 프로젝트입니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-getting-started` | Cargo의 역할, 패키지 생성, 패키지 레이아웃, `Cargo.toml`과 `Cargo.lock`, Cargo home | 바이너리/라이브러리 패키지 생성과 디렉터리 관례 | Cargo 1, 2.1~2.3, 2.5, 2.6, 2.10 |
| ⬜ | `ch2-dependencies` | 버전 요구사항 문법, crates.io·git·path 의존성, `cargo add/update/tree`, `[patch]`, 의존성 해석 | 세 가지 출처의 의존성 사용, 버전 충돌 관찰 | Cargo 2.4, 3.3 |
| ⬜ | `ch3-manifest-targets` | 매니페스트 전체 구조, 타깃(lib/bin/example/test/bench), `rust-version`, `[lints]` | 여러 바이너리와 예제를 가진 패키지 | Cargo 3.1, 3.16 |
| ⬜ | `ch4-workspaces` | 워크스페이스, 가상 매니페스트, 설정 상속, 멤버 간 의존 | 라이브러리 2개 + 바이너리 1개 워크스페이스 | Cargo 3.2 |
| ⬜ | `ch5-features` | 기능 플래그, 선택적 의존성, 기능 통합, `cfg(feature)` | 기능으로 구현을 바꾸는 라이브러리 | Cargo 3.4 |
| ⬜ | `ch6-profiles-build-performance` | 프로필(dev/release/사용자 정의), 빌드 캐시, 빌드 시간 분석과 단축 | 프로필별 바이너리 크기·속도 비교, `cargo build --timings` | Cargo 2.11, 3.5, 3.9, 3.15 |
| ⬜ | `ch7-configuration-env` | `.cargo/config.toml`, 별칭, Cargo가 읽고 설정하는 환경 변수 | 별칭 정의, `env!`/`CARGO_PKG_*` 사용 | Cargo 3.6, 3.7 |
| ⬜ | `ch8-build-scripts` | `build.rs`, `cargo::` 지시문, 코드 생성, 네이티브 라이브러리 링크 | 빌드 시 코드 생성, C 코드 컴파일·링크(`cc`) | Cargo 3.8 |
| ⬜ | `ch9-testing-ci` | `cargo test`/`bench`, 테스트 종류별 위치, CI 구성 | 단위·통합·문서 테스트 프로젝트, GitHub Actions 워크플로 | Cargo 2.7, 2.8 |
| ⬜ | `ch10-publishing-semver` | 게시 절차, `cargo package`, 레지스트리, SemVer 호환성 규칙, `cargo install` | `cargo publish --dry-run`, 호환/비호환 변경 사례 | Cargo 2.9, 3.12, 3.13 |
| ⬜ | `ch11-commands-tools` | 명령어 분류, `cargo metadata`, 사용자 정의 하위 명령, 외부 도구 연동, future-incompat 리포트 | `cargo-hello` 하위 명령 만들기, 메타데이터 JSON 읽기 | Cargo 3.10, 3.11, 3.14, 4 |

### Part 4. rustdoc — `part4-rustdoc/`

크레이트 문서를 작성하고 생성하는 방법을 다룹니다. 실습 대상은 `projects/` 아래의 문서화용 라이브러리입니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-rustdoc-basics` | rustdoc의 역할, `cargo doc`, 명령줄 인자, 생성된 문서 읽기와 검색 | 작은 라이브러리 문서 생성 | rustdoc 1~3 |
| ⬜ | `ch2-writing-documentation` | `///`와 `//!`, 무엇을 문서화할지, `#[doc]` 속성, 재내보내기, 이름으로 항목 링크 | 크레이트·모듈·항목 수준 문서, 인트라 문서 링크 | rustdoc 4.1~4.4 |
| ⬜ | `ch3-doc-tests` | 문서 테스트, 코드 블록 속성(`ignore`/`no_run`/`should_panic`/`compile_fail`), 숨김 줄 | 문서 테스트가 포함된 라이브러리 | rustdoc 4.5 |
| ⬜ | `ch4-lints-advanced` | rustdoc 린트, scraped examples, `cfg(doc)`, 문서 별칭 | 깨진 링크 검출, 플랫폼별 문서 | rustdoc 5~7 |

### Part 5. rustc — `part5-rustc/`

Cargo 없이 컴파일러를 직접 다루며 옵션과 동작을 이해합니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-rustc-basics` | rustc의 역할, 주요 인자(`--crate-type`, `--emit`, `--extern`, `--print`) | 수동으로 라이브러리와 바이너리 링크, 중간 산출물(LLVM IR, asm) 출력 | rustc 1, 2, 2.1 |
| ⬜ | `ch2-codegen-options` | `-C` 옵션(`opt-level`, `debuginfo`, `lto`, `panic`, `codegen-units`, `target-cpu`) | 옵션별 바이너리 크기·성능 비교 | rustc 2.2 |
| ⬜ | `ch3-lints` | 린트 수준(`allow`/`expect`/`warn`/`deny`/`forbid`), 린트 그룹, 주요 린트 | 린트 수준 조정, 명령줄·속성·Cargo `[lints]` 비교 | rustc 4 |
| ⬜ | `ch4-targets-cross-compilation` | 타깃 트리플, 내장 타깃, 플랫폼 지원 등급, 사용자 정의 타깃 | 다른 타깃으로 크로스 컴파일 | rustc 7, 16 |
| ⬜ | `ch5-conditional-compilation` | `--cfg`, `--check-cfg`, Cargo에서의 동작 | 사용자 정의 cfg와 오타 검출 | rustc 11 |
| ⬜ | `ch6-optimization-instrumentation` | PGO, 계측 기반 코드 커버리지, 링커 플러그인 LTO | 커버리지 리포트 생성, PGO 적용 전후 비교 | rustc 8~10 |
| ⬜ | `ch7-output-internals` | JSON 진단 출력, 테스트 하네스, 심벌 맹글링, 소스 경로 재매핑, 익스플로잇 완화 | JSON 진단 파싱, 맹글링된 심벌 확인 | rustc 5, 6, 12~14 |

### Part 6. 컴파일러 에러 색인 — `part6-error-index/`

자주 만나는 에러를 분류별로 **재현 → 원인 이해 → 수정** 순서로 익힙니다. 에러를 내는 코드는 `compile_fail/E0xxx_*.rs`, 고친 코드는 `examples/E0xxx_fixed.rs`로 짝을 맞춥니다.

| 상태 | 챕터 | 학습 주제 | 다룰 에러 코드 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-reading-diagnostics` | 진단 메시지 구조, 에러 코드 체계, `rustc --explain`, 제안 자동 적용 | (개요) | Error Index 첫 페이지 |
| ⬜ | `ch2-ownership-borrowing` | 이동과 대여 관련 에러 | E0382, E0499, E0502, E0505, E0506, E0507, E0596, E0597, E0716, E0373, E0384 | 각 코드 항목 |
| ⬜ | `ch3-lifetimes` | 라이프타임 관련 에러 | E0106, E0261, E0310, E0515, E0521, E0621 | 각 코드 항목 |
| ⬜ | `ch4-types-traits` | 타입 불일치와 트레이트 관련 에러 | E0308, E0277, E0282, E0599, E0369, E0038, E0046, E0117, E0119, E0207 | 각 코드 항목 |
| ⬜ | `ch5-names-visibility` | 이름 해석과 가시성 에러 | E0412, E0425, E0432, E0433, E0423, E0428, E0583, E0603, E0616 | 각 코드 항목 |
| ⬜ | `ch6-patterns-unsafe-misc` | 패턴, unsafe, const, async, 불안정 기능 에러 | E0004, E0005, E0015, E0133, E0658, E0728, E0752 | 각 코드 항목 |

### Part 7. 명령줄 애플리케이션 — `part7-cli/`

파일에서 패턴을 찾는 도구 `grrs`를 만들며 CLI 개발 전 과정을 익힙니다. 챕터마다 `projects/grrs`를 단계별로 발전시킵니다.

- 준비: `clap`, `anyhow`, `indicatif`, `log`, `assert_cmd`, `predicates`, `assert_fs` 등 외부 크레이트를 사용합니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-setup-arguments` | 프로젝트 설정, `std::env::args`, `clap` derive로 인자 파싱 | 수동 파싱판과 `clap`판 | CLI Book 1.1, 1.2 |
| ⬜ | `ch2-implementation-errors` | 첫 구현, `Result`와 `?`, `anyhow` 컨텍스트 | 파일 읽기와 패턴 검색, 친절한 에러 메시지 | CLI Book 1.3, 1.4 |
| ⬜ | `ch3-output` | stdout/stderr 구분, 출력 버퍼링, 진행 표시줄, 로깅 | `BufWriter`, `indicatif`, `log`/`env_logger` | CLI Book 1.5 |
| ⬜ | `ch4-testing` | 테스트하기 쉬운 구조, 단위 테스트, 바이너리 통합 테스트 | `assert_cmd` 통합 테스트, 임시 파일 | CLI Book 1.6 |
| ⬜ | `ch5-packaging-distribution` | `cargo install`, `cargo publish`, 바이너리 배포, CI 릴리스 | 릴리스 빌드와 배포 워크플로 | CLI Book 1.7 |
| ⬜ | `ch6-signals-config-exit-codes` | 시그널 처리(Ctrl+C), 설정 파일, 종료 코드 | `ctrlc`/`signal-hook`, `confy`, `ExitCode` | CLI Book 2.1~2.3 |
| ⬜ | `ch7-humans-machines-docs` | 사람을 위한 출력(패닉 메시지), 기계를 위한 출력(파이프, JSON), man 페이지 생성 | `human-panic`, `IsTerminal`, `serde_json`, `clap_mangen` | CLI Book 2.4~2.6 |

### Part 8. 임베디드 — `part8-embedded/`

ARM Cortex-M 대상의 `no_std` 프로그래밍을 QEMU 에뮬레이션으로 실습합니다. 실제 보드(STM32F3DISCOVERY)가 필요한 내용은 설명만 다룹니다.

- 준비: `rustup target add thumbv7m-none-eabi`, `qemu-system-arm`(설치됨), `cargo-binutils`, `gdb`. 모든 실습은 타깃이 달라 `projects/` 아래 독립 프로젝트로 만듭니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-intro-no-std` | 하드웨어 개요, `no_std`와 `core`, 도구 설치와 확인 | 최소 `no_std` 크레이트, 환경 점검 스크립트 | Embedded 1 |
| ⬜ | `ch2-qemu-semihosting` | `cortex-m-quickstart`, 메모리 맵(`memory.x`), QEMU 실행, 세미호스팅, 디버깅 | QEMU에서 Hello 출력, GDB 연결 | Embedded 2.1, 2.2, 2.4 |
| ⬜ | `ch3-registers-panic-interrupts` | 메모리 맵 레지스터(PAC/HAL 계층), 패닉 처리기, 예외, 인터럽트 | SysTick 예외 처리기, 패닉 동작 선택 | Embedded 2.3, 2.5~2.7 |
| ⬜ | `ch4-peripherals` | 주변장치 접근, 대여 검사기 활용, 싱글턴 | 레지스터 구조체, `take()` 싱글턴 패턴 | Embedded 3 |
| ⬜ | `ch5-static-guarantees` | 타입 상태 프로그래밍, 상태 기계로서의 주변장치, 설계 계약, 무비용 추상화 | GPIO 핀 타입 상태 (호스트에서 실행 가능한 예제) | Embedded 4 |
| ⬜ | `ch6-portability-concurrency` | `embedded-hal`, 임계 구역, 원자 타입, `Mutex`, RTIC 개요 | HAL 트레이트 기반 드라이버, 인터럽트와 공유 상태 | Embedded 5, 6 |
| ⬜ | `ch7-collections-design-patterns` | `heapless`, `alloc` 사용, HAL 설계 지침 | 고정 용량 컬렉션, 전역 할당자 | Embedded 7, 8 |
| ⬜ | `ch8-c-interop-optimizations` | C 개발자를 위한 대응표, C↔Rust 연동, 속도/크기 최적화, 수학 함수 | C에서 Rust 호출·Rust에서 C 호출, 크기 최적화 비교 | Embedded 9~11 |

### Part 9. 레퍼런스 — `part9-reference/`

언어 명세에 가까운 The Reference를 읽으며 Part 0에서 직관으로 넘어간 규칙들을 정확히 이해합니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-lexical-structure` | 문법 표기법, 토큰, 키워드 분류, 식별자, 리터럴, 주석 | 리터럴 종류 전체, raw 식별자·문자열 | Reference 1, 2 |
| ⬜ | `ch2-macros` | 선언적 매크로 상세(프래그먼트, 반복, 위생), 절차적 매크로 3종 | `macro_rules!` 심화, derive·속성·함수형 매크로(`projects/`) | Reference 3 |
| ⬜ | `ch3-crates-items` | 크레이트와 소스 파일, 조건부 컴파일, 아이템 전체(유니언, `const`/`static`, extern 블록, 연관 아이템) | 아이템 종류별 예제, `cfg`/`cfg_attr` | Reference 4~6 |
| ⬜ | `ch4-attributes` | 속성 분류: 테스트, derive, 진단, 코드 생성, 한계, 타입 시스템 | `#[inline]`, `#[must_use]`, `#[deprecated]`, `#[non_exhaustive]`, `#[track_caller]` | Reference 7 |
| ⬜ | `ch5-statements-expressions` | 구문과 표현식, 장소/값 표현식, 임시값 수명, 연산자 우선순위, 평가 순서 | 임시값 수명 연장, 블록 표현식, 클로저 표현식 | Reference 8 |
| ⬜ | `ch6-patterns` | 패턴 문법 전체, 바인딩 모드, 반박 가능성 | 기본 바인딩 모드, 슬라이스·rest 패턴 | Reference 9 |
| ⬜ | `ch7-type-system` | 타입 종류, DST, 타입 레이아웃, 내부 가변성, 서브타이핑과 변성, 강제 변환, 소멸자, 라이프타임 생략 | `size_of`/`align_of`, `repr`, 강제 변환 지점, drop 순서 | Reference 10 |
| ⬜ | `ch8-special-types-names` | 특수 타입과 트레이트(`Sized`, `Copy`, `Drop`, `Send`), 네임스페이스, 스코프, prelude, 경로, 이름 해석, 가시성 | 네임스페이스 충돌, `pub(crate)`/`pub(in path)` | Reference 11, 12 |
| ⬜ | `ch9-memory-panic-linkage` | 메모리 모델, 변수, 패닉, 링크 방식과 크레이트 타입, ABI, 런타임 | `extern "C"`, `#[no_mangle]`, `panic=abort`와 unwind | Reference 13~15, 19, 20 |
| ⬜ | `ch10-unsafety-const-asm` | `unsafe` 키워드의 의미, 미정의 동작 목록, 상수 평가(`const fn`), 인라인 어셈블리 | `const fn`과 const 블록, `asm!` 기초 | Reference 16~18 |

### Part 10. Rustonomicon — `part10-nomicon/`

unsafe Rust를 올바르게 쓰기 위한 지식을 다룹니다.

- 준비: 미정의 동작 검출에 Miri를 사용합니다 (`rustup +nightly component add miri`, `cargo +nightly miri run`).

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-safe-and-unsafe` | safe/unsafe의 경계, unsafe가 할 수 있는 일, 안전성 불변식 | 안전한 추상화로 감싼 unsafe 코드 | Nomicon 1 |
| ⬜ | `ch2-data-layout` | `repr(Rust)`, DST·ZST·빈 타입, `repr(C)`/`transparent`/`packed`/`align` | 레이아웃 출력과 비교, 니치 최적화 | Nomicon 2 |
| ⬜ | `ch3-ownership-lifetimes` | 참조와 에일리어싱, 라이프타임 추론, 한계, 생략 규칙, 무제한 라이프타임, HRTB | 라이프타임 풀어쓰기, 대여 검사기 한계 사례 | Nomicon 3.1~3.7 |
| ⬜ | `ch4-variance-dropck` | 서브타이핑과 변성, drop 검사, `PhantomData`, 대여 분할 | 변성 표 확인 예제, `split_at_mut` 구현 | Nomicon 3.8~3.11 |
| ⬜ | `ch5-type-conversions` | 강제 변환, 점 연산자 해석, 캐스트, `transmute` | 메서드 탐색 순서, 안전한 `transmute` 대안 | Nomicon 4 |
| ⬜ | `ch6-uninitialized-memory` | 검사되는 초기화, drop 플래그, `MaybeUninit` | 배열 부분 초기화 | Nomicon 5 |
| ⬜ | `ch7-obrm` | 생성자, 소멸자, 누수와 안전성(`mem::forget`) | 누수에 안전한 가드 설계 | Nomicon 6 |
| ⬜ | `ch8-unwinding` | 되감기, 예외 안전성, 포이즈닝 | 패닉에도 불변식을 지키는 코드 | Nomicon 7 |
| ⬜ | `ch9-concurrency` | 데이터 경쟁과 경쟁 조건, `Send`/`Sync` 수동 구현, 원자 연산과 메모리 순서 | 스핀락 구현 | Nomicon 8 |
| ⬜ | `ch10-implementing-vec` | `Vec<T>` 직접 구현: 레이아웃, 할당, push/pop, `Deref`, `IntoIter`, `RawVec`, `Drain`, ZST 처리 | 단계별 `MyVec` (`projects/my-vec`) | Nomicon 9 |
| ⬜ | `ch11-implementing-arc-mutex` | `Arc<T>` 직접 구현: 레이아웃, 복제, 해제 | `MyArc` (`projects/my-arc`) | Nomicon 10 |
| ⬜ | `ch12-ffi-beneath-std` | FFI: C 함수 호출, 콜백, 불투명 타입, `#[panic_handler]` | libc 함수 호출, C에서 쓰는 Rust 라이브러리 | Nomicon 11, 12 |

### Part 11. 불안정 기능 — `part11-unstable/`

nightly 전용 기능을 사용하는 방법과 대표 기능을 살펴봅니다. 불안정 기능은 수시로 바뀌므로 **다룰 기능 목록은 각 챕터를 작성하는 시점에 Unstable Book에서 확정**합니다.

- 준비: nightly 툴체인. 각 실습은 `rust-toolchain.toml`로 nightly를 지정한 `projects/` 독립 프로젝트로 만듭니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-nightly-toolchain` | 릴리스 채널, 기능 게이트(`#![feature]`), 추적 이슈, 안정화 과정, `rust-toolchain.toml` | nightly 프로젝트 구성, stable에서의 E0658 확인 | Unstable Book 소개 |
| ⬜ | `ch2-compiler-flags` | `-Z` 플래그: 새니타이저, 매크로 확장 출력 등 | AddressSanitizer로 메모리 오류 검출 | Compiler flags |
| ⬜ | `ch3-language-features` | 대표 언어 기능 4~6개 (작성 시점에 선정) | 기능별 예제 | Language features |
| ⬜ | `ch4-library-features` | 대표 라이브러리 기능 4~6개 (작성 시점에 선정) | 기능별 예제 | Library features |

---

## 4. 진행 현황

| 파트 | 완료 / 전체 |
| --- | --- |
| 사전 작업 | 3 / 3 |
| Part 0. 시작하기 | 20 / 20 |
| Part 1. 표준 라이브러리 | 4 / 14 |
| Part 2. 에디션 | 0 / 6 |
| Part 3. Cargo | 0 / 11 |
| Part 4. rustdoc | 0 / 4 |
| Part 5. rustc | 0 / 7 |
| Part 6. 컴파일러 에러 색인 | 0 / 6 |
| Part 7. 명령줄 애플리케이션 | 0 / 7 |
| Part 8. 임베디드 | 0 / 8 |
| Part 9. 레퍼런스 | 0 / 10 |
| Part 10. Rustonomicon | 0 / 12 |
| Part 11. 불안정 기능 | 0 / 4 |
| **합계** | **24 / 109 챕터** |
