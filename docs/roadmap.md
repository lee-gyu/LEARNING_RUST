# Rust 학습 로드맵

[rust-lang.org/learn](https://rust-lang.org/learn/)에 정리된 공식 학습 자료를 순서대로 따라가며, 챕터마다 **설명 문서(README.md)** 와 **실습 예제(.rs)** 를 작성하기 위한 전체 계획입니다.

- 기준 툴체인: Rust 1.100 (stable), Edition 2024 — `mise.toml`의 `rust = "latest"`
- 전체 규모: 12개 파트, 109개 챕터. 본편과 별도로 부록 A(35개 챕터)와 부록 B(17개 챕터)가 있습니다.
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

본편과 별도로, 공식 학습 자료 밖의 책이나 주제를 같은 방식으로 정리하는 **부록**이 있습니다. 부록의 챕터는 본편의 합계(109)에 넣지 않고 따로 셉니다.

| 부록 | 디렉터리 | 기준 문서 | 챕터 수 |
| --- | --- | --- | --- |
| 부록 A | `appd-a/` | [Effective Rust](https://effective-rust.com/) (David Drysdale, O'Reilly 2024) | 35 |
| 부록 B | `appd-b/` | 다른 언어를 써 온 프로그래머가 자주 묻는 질문. 공식 문서와 챕터마다 밝힌 1차 자료(보안 보고서, 도입 사례, 설문) | 17 |

**학습 순서**: Part 0 → 11 순서대로 진행합니다. Part 0은 이후 모든 파트의 선수 과정이며, Part 7·8(응용 분야)은 서로 독립적이라 순서를 바꿔도 됩니다. Part 10은 Part 9의 ch7·ch10을, Part 11은 nightly 툴체인을 전제로 합니다. 부록 A는 Part 0과 Part 1만, 부록 B는 Part 0만 전제로 하며, 본편의 나머지 파트와는 순서가 무관합니다.

## 2. 저장소 구조와 규칙

### 2.1 디렉터리 구조

```
LEARNING_RUST/
├── Cargo.toml                      # 워크스페이스 루트 (members = ["part*/ch*", "appd-*/ch*"])
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
├── ...
├── appd-a/                         # 부록 A. 구조는 파트와 같습니다
│   ├── README.md
│   └── ch1-express-data-structures/
└── appd-b/                         # 부록 B. 구조는 파트와 같습니다
    ├── README.md
    └── ch1-why-rust/
        └── compare/                # (선택) 비교용 C/C++ 코드. Cargo가 빌드하지 않습니다
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
| 부록 디렉터리 | `appd-{문자}` | `appd-a` |
| 부록 A의 챕터 디렉터리 | `ch{N}-{slug}`, N은 책의 Item 번호 | `ch14-lifetimes` |
| 부록 A의 패키지 이름 | `er-ch{N}-{slug}` | `er-ch14-lifetimes` |
| 부록 B의 챕터 디렉터리 | `ch{N}-{slug}` | `ch9-c-memory-errors` |
| 부록 B의 패키지 이름 | `faq-ch{N}-{slug}` | `faq-ch9-c-memory-errors` |
| 부록 B의 질문 번호 | `Q{챕터}.{순서}` | `Q9.4` |
| 부록 B의 비교 코드 | `compare/{NN}_{snake_case}.c` 또는 `.cpp` | `compare/01_use_after_free.c` |

부록 A 안에서는 다른 챕터를 `ch14`(Item 14)처럼, 부록 B 안에서는 `ch9`·`Q9.4`처럼 쓰고, 본편 챕터는 `Part 1 ch8`처럼 파트를 붙여 구분합니다. 다른 부록의 챕터는 `부록 A ch14`처럼 부록 이름을 붙입니다.

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
| ✅ | 루트 `Cargo.toml` 작성 (`[workspace]`, `resolver = "3"`, `members = ["part*/ch*"]`(부록을 시작하며 `"appd-*/ch*"` 추가), `[workspace.package]`에 `edition = "2024"`, `publish = false`) |
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
| ✅ | `ch5-iterators` | `Iterator`/`IntoIterator`/`FromIterator`, 어댑터와 소비자, 반복자 구현 | 어댑터 모음, `fold`/`scan`/`zip`/`chain`/`peekable`, 직접 만든 반복자 | `std::iter` |
| ✅ | `ch6-core-traits` | 변환(`From`/`Into`/`TryFrom`/`AsRef`), 연산자, 비교, 해시, `Default`, `Clone`/`Copy`, `Borrow`/`Cow` | 트레이트별 구현 예제, `Cow`로 불필요한 복사 줄이기 | `std::convert`, `std::ops`, `std::cmp`, `std::hash`, `std::default`, `std::borrow`, `std::marker` |
| ✅ | `ch7-smart-pointers-memory` | `Box`, `Rc`/`Weak`, `Cell`/`RefCell`/`OnceCell`, `std::mem` 함수 | `mem::swap`/`take`/`replace`, 내부 가변성 패턴, 크기·정렬 확인 | `std::boxed`, `std::rc`, `std::cell`, `std::mem` |
| ✅ | `ch8-errors-panic` | `Error` 트레이트, 에러 타입 설계, `source` 체인, `Box<dyn Error>`, 패닉 훅 | 사용자 정의 에러, 에러 변환(`From`), `catch_unwind`, 백트레이스 | `std::error`, `std::panic`, `std::backtrace` |
| ✅ | `ch9-io-fs-path` | `Read`/`Write`/`BufRead`/`Seek`, 표준 입출력, 파일·디렉터리, `Path`/`PathBuf` | 줄 단위 읽기, 버퍼링 쓰기, 디렉터리 순회, 경로 조작 | `std::io`, `std::fs`, `std::path` |
| ✅ | `ch10-env-process-time` | 인자·환경 변수, 자식 프로세스, 종료 코드, `Instant`/`Duration`/`SystemTime` | `Command` 실행·파이프, 실행 시간 측정 | `std::env`, `std::process`, `std::time` |
| ✅ | `ch11-threads-sync` | 스레드와 스코프 스레드, `Mutex`/`RwLock`/`Condvar`, `Arc`, `OnceLock`/`LazyLock`, 채널, 원자 타입 | 스코프 스레드, 생산자-소비자, 전역 지연 초기화, `AtomicUsize` 카운터 | `std::thread`, `std::sync`, `std::sync::mpsc`, `std::sync::atomic` |
| ✅ | `ch12-net` | TCP/UDP 소켓, 주소 타입 | 에코 서버·클라이언트, UDP 송수신 | `std::net` |
| ✅ | `ch13-async-primitives` | `Future` 트레이트, `Poll`, `Context`/`Waker`, `Pin` | 직접 구현한 퓨처, 최소 실행기(`block_on`) 만들기 | `std::future`, `std::task`, `std::pin` |
| ✅ | `ch14-ffi-any-misc` | `CString`/`CStr`, `OsString`, `Any`와 다운캐스팅, 수치 타입(`NonZero`, `Wrapping`) | C 문자열 변환, `dyn Any` 다운캐스트, 오버플로 처리 방식 | `std::ffi`, `std::any`, `std::num` |

### Part 2. 에디션 — `part2-editions/`

에디션이 무엇이고 각 에디션에서 무엇이 바뀌었는지를 비교 예제로 확인합니다.

- 준비: 에디션별 차이를 보여주는 예제는 `projects/` 아래에 에디션을 다르게 지정한 독립 프로젝트로 만듭니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch1-editions-overview` | 에디션의 목적과 호환성 보장, `edition` 필드, 에디션 간 크레이트 혼용 | 에디션이 다른 두 크레이트를 함께 링크하는 프로젝트 | Edition Guide 1, 1.1 |
| ✅ | `ch2-rust-2015-2018` | 2015 기준선, 경로·모듈 시스템 변경, 익명 트레이트 매개변수, 새 키워드(`async`/`await`/`dyn`), raw 식별자 | 같은 코드의 2015판/2018판 비교 | Edition Guide 2, 3 |
| ✅ | `ch3-rust-2021` | prelude 추가, 기능 리졸버 v2, 배열 `IntoIterator`, 클로저 분리 캡처, 패닉 매크로 일관성, 예약 문법, C 문자열 리터럴 | 2018판/2021판 동작 차이 비교 | Edition Guide 4 |
| ✅ | `ch4-rust-2024-language` | RPIT 라이프타임 캡처, `if let`·꼬리 표현식 임시값 범위, let 체인, `unsafe extern`, unsafe 속성, `static mut` 참조 금지, never 타입 폴백, `gen` 키워드 | 2021판/2024판 동작 차이 비교 | Edition Guide 5.1 |
| ✅ | `ch5-rust-2024-ecosystem` | prelude 변경, `Box<[T]>` `IntoIterator`, 새로 unsafe가 된 함수, rust-version 인식 리졸버, rustdoc 통합 테스트, rustfmt 스타일 에디션 | 표준 라이브러리·Cargo·rustfmt 변경 확인 | Edition Guide 5.2~5.5 |
| ✅ | `ch6-migration-practice` | `cargo fix --edition`, 수동 수정이 필요한 경우, 고급 마이그레이션 | 2018 프로젝트를 2021 → 2024로 단계별 이전 | Edition Guide 1.2, 1.3 |

### Part 3. Cargo — `part3-cargo/`

패키지 관리자이자 빌드 시스템인 Cargo를 다룹니다. 대부분의 실습이 `projects/` 아래의 독립 프로젝트입니다.

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch1-getting-started` | Cargo의 역할, 패키지 생성, 패키지 레이아웃, `Cargo.toml`과 `Cargo.lock`, Cargo home | 바이너리/라이브러리 패키지 생성과 디렉터리 관례 | Cargo 1, 2.1~2.3, 2.5, 2.6, 2.10 |
| ✅ | `ch2-dependencies` | 버전 요구사항 문법, crates.io·git·path 의존성, `cargo add/update/tree`, `[patch]`, 의존성 해석 | 세 가지 출처의 의존성 사용, 버전 충돌 관찰 | Cargo 2.4, 3.3 |
| ✅ | `ch3-manifest-targets` | 매니페스트 전체 구조, 타깃(lib/bin/example/test/bench), `rust-version`, `[lints]` | 여러 바이너리와 예제를 가진 패키지 | Cargo 3.1, 3.16 |
| ✅ | `ch4-workspaces` | 워크스페이스, 가상 매니페스트, 설정 상속, 멤버 간 의존 | 라이브러리 2개 + 바이너리 1개 워크스페이스 | Cargo 3.2 |
| ✅ | `ch5-features` | 기능 플래그, 선택적 의존성, 기능 통합, `cfg(feature)` | 기능으로 구현을 바꾸는 라이브러리 | Cargo 3.4 |
| ✅ | `ch6-profiles-build-performance` | 프로필(dev/release/사용자 정의), 빌드 캐시, 빌드 시간 분석과 단축 | 프로필별 바이너리 크기·속도 비교, `cargo build --timings` | Cargo 2.11, 3.5, 3.9, 3.15 |
| ✅ | `ch7-configuration-env` | `.cargo/config.toml`, 별칭, Cargo가 읽고 설정하는 환경 변수 | 별칭 정의, `env!`/`CARGO_PKG_*` 사용 | Cargo 3.6, 3.7 |
| ✅ | `ch8-build-scripts` | `build.rs`, `cargo::` 지시문, 코드 생성, 네이티브 라이브러리 링크 | 빌드 시 코드 생성, C 코드 컴파일·링크(`cc`) | Cargo 3.8 |
| ✅ | `ch9-testing-ci` | `cargo test`/`bench`, 테스트 종류별 위치, CI 구성 | 단위·통합·문서 테스트 프로젝트, GitHub Actions 워크플로 | Cargo 2.7, 2.8 |
| ✅ | `ch10-publishing-semver` | 게시 절차, `cargo package`, 레지스트리, SemVer 호환성 규칙, `cargo install` | `cargo publish --dry-run`, 호환/비호환 변경 사례 | Cargo 2.9, 3.12, 3.13 |
| ✅ | `ch11-commands-tools` | 명령어 분류, `cargo metadata`, 사용자 정의 하위 명령, 외부 도구 연동, future-incompat 리포트 | `cargo-hello` 하위 명령 만들기, 메타데이터 JSON 읽기 | Cargo 3.10, 3.11, 3.14, 4 |

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

### 부록 A. Effective Rust — `appd-a/`

[Effective Rust](https://effective-rust.com/)(David Drysdale, O'Reilly 2024)의 35개 Item을 Item 1개 = 챕터 1개로 정리합니다. 챕터 번호는 책의 Item 번호와 같습니다. 본편이 "무엇이 있고 어떻게 쓰는가"를 다룬다면, 부록은 "여러 방법 중 무엇을 왜 고르는가"를 다룹니다. 챕터의 중심은 비권장 코드와 권장 코드의 비교, 그리고 지침을 따르지 않아도 되는 경우입니다.

- 선수 과정: Part 0, Part 1. 문법과 API 사용법은 본편 챕터로 링크하고 반복하지 않습니다.
- 저작권: 온라인판은 CC BY-NC-ND 4.0입니다. 본문을 번역·발췌하지 않고 책의 코드도 옮기지 않습니다. 설명과 예제는 새로 설계하고, Item 원제만 영어 그대로 인용해 링크합니다.
- 버전: 책은 2018 에디션, rustc 1.70 기준입니다. 실제 동작과 다른 곳은 챕터 README의 "책과 달라진 점"에 적습니다.
- README는 2.4절의 여섯 항목을 유지하되, 핵심 개념을 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 풀고 "기억할 것"으로 마칩니다.
- 준비: ch4는 `thiserror`·`anyhow`, ch28은 `syn`·`quote`, ch30은 `criterion`, ch34는 `cc`, ch35는 `bindgen`(libclang)을 씁니다. ch16(Miri)·ch25·ch28·ch30의 일부는 nightly 툴체인이나 추가 도구가 필요합니다.
- 기준 문서 절의 주소는 `https://effective-rust.com/{이름}.html`입니다.

#### Types — ch1~ch9

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch1-express-data-structures` | 기본 타입과 암묵적 변환이 없는 이유, 집합 타입, 열거형으로 무효한 상태를 표현할 수 없게 만들기, `Option`/`Result`로 부재와 실패 표현 | `bool` 인자를 열거형으로, 필드 조합을 데이터를 가진 열거형으로, 센티널 값(-1, 빈 문자열)을 `Option`으로 | Item 1 (`use-types`) |
| ✅ | `ch2-express-common-behavior` | 함수와 메서드, 함수 포인터, 클로저와 `Fn`/`FnMut`/`FnOnce`, 트레이트 바운드와 트레이트 객체, 매개변수로 무엇을 받을지 고르는 기준 | 함수 포인터와 클로저 받기 비교, 가장 느슨한 `Fn*` 바운드 고르기, 마커 트레이트, 트레이트 객체로 받기 | Item 2 (`use-types-2`) |
| ✅ | `ch3-option-result-transforms` | `match` 대신 `if let`·`?`·변환 메서드, 에러 타입 맞추기(`map_err`, `From`), `Option`↔`Result`, `as_ref`, 변환 메서드 고르는 순서 | 명시적 `match`를 단계별로 변환 메서드로 바꾸기, `?`와 에러 변환, 참조를 유지하는 변환 | Item 3 (`transform`) |
| ✅ | `ch4-idiomatic-errors` | `Error` 트레이트의 요구 사항, 최소 에러 타입, 중첩 에러 열거형, 트레이트 객체 에러의 득실, 라이브러리와 애플리케이션의 선택 기준 | `String` 에러의 한계와 뉴타입, 중첩 에러 열거형과 `From`, `Box<dyn Error>`, `thiserror`판, `anyhow`판 | Item 4 (`errors`) |
| ✅ | `ch5-type-conversions` | 사용자 정의 변환(`From`/`Into`/`TryFrom`), `as` 캐스트의 위험, 컴파일러가 해 주는 강제 변환 | `From`을 구현하고 `Into` 바운드로 받기, `as`의 잘림과 `try_from`, 강제 변환이 일어나는 자리 모음 | Item 5 (`casts`) |
| ✅ | `ch6-newtype-pattern` | 단위 혼동 막기, 의미가 드러나는 인자 타입, 고아 규칙 우회, 뉴타입의 한계(트레이트를 다시 구현해야 함) | 단위 뉴타입, 외부 타입에 외부 트레이트 구현, 위임 구현의 비용 | Item 6 (`newtype`) |
| ✅ | `ch7-builders` | 필드가 많은 구조체의 초기화 문제, `Default`와 구조체 갱신 문법의 한계, 빌더 패턴(소비형과 `&mut`형), derive로 빌더 만들기 | 큰 구조체 직접 초기화, `Default` + `..`, 소비형 빌더, `&mut self` 빌더, `derive_builder`(선택) | Item 7 (`builders`) |
| ✅ | `ch8-references-pointers` | 참조와 포인터의 차이, 포인터 트레이트(`Deref`, `AsRef`, `Borrow` 등), 넓은 포인터(슬라이스, 트레이트 객체), 스마트 포인터 고르기 | 포인터 크기 비교, 슬라이스를 받는 시그니처, vtable이 붙은 포인터, `AsRef`/`Borrow` 바운드, 스마트 포인터 선택표를 따라가는 예제 | Item 8 (`references`) |
| ✅ | `ch9-iterator-transforms` | 반복자 트레이트, 변환과 소비 메서드, `Result` 값 모으기, 반복문을 반복자 체인으로 바꾸기, 명시적 반복문이 더 나은 경우 | `for` 반복문을 단계별로 체인으로, `collect::<Result<_, _>>()`, 조기 탈출·복잡한 상태에서 반복문 유지 | Item 9 (`iterators`) |

#### Traits — ch10~ch13

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch10-standard-traits` | 공개 타입에 구현할 표준 트레이트 고르기(`Clone`, `Copy`, `Default`, `PartialEq`/`Eq`, `PartialOrd`/`Ord`, `Hash`, `Debug`/`Display`), derive할 수 없거나 하면 안 되는 경우, 직접 구현할 때의 일관성 규칙, 연산자 오버로드 | derive 가능 조건, `Copy`를 붙이면 안 되는 타입, `Eq`와 `Hash`가 어긋날 때 `HashMap`의 오동작, 부동소수점과 `PartialEq`, 트레이트 체크리스트를 적용한 타입 | Item 10 (`std-traits`) |
| ✅ | `ch11-drop-raii` | RAII와 가드 타입, `Drop`의 제약(`&mut self`, 실패를 돌려줄 수 없음), `drop(x)`로 일찍 해제하기 | 조기 반환에서 해제를 빠뜨리는 수동 관리와 가드 비교, 직접 만든 가드, drop 순서 | Item 11 (`raii`) |
| ✅ | `ch12-generics-vs-trait-objects` | 단형성화와 동적 디스패치의 비용(코드 크기, 속도, 컴파일 시간), 여러 트레이트 바운드, dyn 호환, 이종 컬렉션, 선택 기준 | 같은 함수의 제네릭판과 `dyn`판, 이종 컬렉션, dyn 호환이 아닌 트레이트와 `where Self: Sized` | Item 12 (`generics`) |
| ✅ | `ch13-default-implementations` | 필수 메서드는 적게, 기본 구현은 넉넉하게. 기본 구현 재정의, 기본 메서드 추가가 호환 변경인 이유 | 필수 메서드 하나에 기본 메서드 여럿인 트레이트, 재정의로 더 빠른 구현 제공 | Item 13 (`default-impl`) |

#### Concepts — ch14~ch20

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch14-lifetimes` | 스택과 라이프타임, 라이프타임의 범위(마지막 사용까지), 여러 라이프타임 매개변수, 생략 규칙, `'static`, 힙에 있는 값의 라이프타임, 자료구조 안의 라이프타임, 익명 라이프타임 `'_` | 스택 프레임과 댕글링 참조, 마지막 사용 뒤에 풀리는 대여, 라이프타임 매개변수가 둘 필요한 함수, `&'static`과 `T: 'static`의 차이, 참조를 담은 구조체, `'_` | Item 14 (`lifetimes`) |
| ✅ | `ch15-borrow-checker` | 대여 규칙과 소유자가 할 수 있는 일, 대여 검사기 에러를 푸는 방법(지역 리팩터링, 자료구조 설계, 스마트 포인터), 자기 참조 자료구조 | 대여 범위 줄이기, `mem::take`/`replace`와 `Option::take`, 참조 대신 인덱스, `Rc<RefCell<T>>`, 자기 참조 시도와 대안 | Item 15 (`borrows`) |
| ✅ | `ch16-avoid-unsafe` | unsafe를 쓰기 전에 표준 라이브러리와 검증된 크레이트에서 안전한 대안 찾기, 꼭 써야 할 때의 지침(작게 감싸기, 안전성 주석, Miri, 테스트) | 직접 쓴 unsafe를 표준 라이브러리의 안전한 API로 바꾸기, 안전한 래퍼와 `// SAFETY:` 주석 | Item 16 (`unsafe`) |
| ✅ | `ch17-shared-state-parallelism` | Rust가 막는 것(데이터 경쟁, `Send`/`Sync`)과 막지 못하는 것(교착 상태), 잠금 범위와 순서, 공유 상태 대신 채널 | 데이터 경쟁 시도가 컴파일되지 않는 모습, 잠금 순서가 엇갈린 교착 상태와 해결, 채널로 다시 설계 | Item 17 (`deadlock`) |
| ✅ | `ch18-dont-panic` | 패닉이 숨어 있는 곳, 라이브러리가 패닉 대신 `Result`를 돌려줘야 하는 이유, `catch_unwind`의 한계, 패닉을 막는 도구 | 패닉하는 API를 `Result` API로, 숨은 패닉 지점(인덱싱, 산술, `RefCell`), `unwrap`을 금지하는 lint 설정 | Item 18 (`panic`) |
| ✅ | `ch19-avoid-reflection` | Rust에 리플렉션이 없는 이유, `type_name`과 `Any`/`TypeId`의 한계, 트레이트 객체 업캐스트 | `type_name`의 한계, `dyn Any` 다운캐스트와 대안(열거형, 트레이트 메서드), 업캐스트 | Item 19 (`reflection`) |
| ✅ | `ch20-avoid-over-optimizing` | 먼저 단순하게 쓰고 측정한 뒤 최적화하기, 자료구조와 할당, 복사 비용을 두려워하지 않기, 참조 대신 소유 타입이나 스마트 포인터로 설계 단순화 | 라이프타임이 번지는 구조체와 소유 타입 구조체 비교, `clone`과 `Rc`, 간단한 실행 시간 측정 | Item 20 (`optimize`) |

#### Dependencies — ch21~ch26

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch21-semver` | 시맨틱 버저닝의 약속, 크레이트 작성자가 지킬 것(무엇이 비호환 변경인가), 사용자가 알 것(버전 요구 사항, `Cargo.lock`, MSRV) | 라이브러리와 사용자 크레이트 한 쌍으로 호환 변경과 미묘한 비호환 변경 재현(`projects/`) | Item 21 (`semver`) |
| ✅ | `ch22-minimize-visibility` | 가시성 단계(`pub`, `pub(crate)`, `pub(super)`, `pub(in …)`), 공개한 것은 되돌리기 어려움, 비공개 필드와 접근자 | 모듈 안에서 가시성 단계 비교, 비공개 필드 덕분에 내부 표현을 바꿔도 사용자 코드가 그대로인 예 | Item 22 (`visibility`) |
| ✅ | `ch23-avoid-wildcard-imports` | 와일드카드 가져오기가 의존성 업데이트에서 이름 충돌을 일으키는 원리, 허용되는 예외(프렐류드, 테스트의 `use super::*`) | 의존 크레이트에 이름이 추가되면 깨지는 사용자 코드(`projects/`), 예외 사례 | Item 23 (`wildcard`) |
| ✅ | `ch24-re-export-dependencies` | API에 드러난 의존성 타입과 버전 불일치, `pub use`로 의존성 재내보내기 | 같은 크레이트의 두 버전이 섞여 타입이 맞지 않는 상황과 재내보내기로 해결(`projects/`) | Item 24 (`re-export`) |
| ✅ | `ch25-dependency-graph` | 버전 지정 방식, 중복 버전, `cargo tree`와 `cargo update`, 의존성 점검 도구, 무엇에 의존할지 판단하는 기준 | `cargo tree -d`/`-i` 읽기, 중복 버전을 만들고 해소(`projects/`), 점검 도구 설정 | Item 25 (`dep-graph`) |
| ✅ | `ch26-feature-creep` | `cfg` 조건부 컴파일, 기능은 더하기만 해야 하는 이유(기능 통합), 공개 필드·트레이트 메서드를 기능으로 가리면 안 되는 이유, 조합 수와 CI | 가산적이지 않은 기능이 통합으로 깨지는 재현, 기능으로 가린 필드가 사용자 코드를 깨는 예(`projects/`) | Item 26 (`features`) |

#### Tooling — ch27~ch32

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch27-document-public-interfaces` | 문서 주석에 쓸 것과 쓰지 말 것, `# Panics`·`# Errors`·`# Safety` 절, 항목 링크, 크레이트·모듈 문서, 문서 누락을 잡는 lint | 같은 라이브러리의 부실한 문서판과 고친 판(`projects/`), `missing_docs`와 깨진 링크 검출 | Item 27 (`documentation`) |
| ✅ | `ch28-macros-judiciously` | 선언적 매크로와 절차적 매크로 3종, 매크로가 필요한 경우와 함수·제네릭으로 충분한 경우, 매크로의 단점 | 반복 코드를 `macro_rules!`로, 인자가 여러 번 평가되는 함정, 제네릭 함수로 대체, 간단한 derive 매크로(`projects/`) | Item 28 (`macros`) |
| ✅ | `ch29-listen-to-clippy` | Clippy 경고를 고치는 이유, lint 그룹, 예외를 이유와 함께 표시하기, 프로젝트 단위 설정 | 경고가 나는 코드와 고친 코드, `#[expect(…, reason = "…")]`, `[lints]` 설정(`projects/`) | Item 29 (`clippy`) |
| ✅ | `ch30-beyond-unit-tests` | 단위·통합·문서 테스트의 역할, 예제 프로그램, 벤치마크, 퍼즈 테스트, 테스트 조언 | 라이브러리 하나에 테스트 종류를 모두 갖추기(`projects/`): 단위, 통합, 문서 테스트, example, 벤치마크, 퍼즈 타깃(선택) | Item 30 (`testing`) |
| ✅ | `ch31-tooling-ecosystem` | Cargo 기본 도구, `cargo metadata`를 바탕으로 한 도구들, 책 전체에 나온 도구 지도 | `cargo metadata` 출력 읽기, `cargo tree`, 도구별 용도와 사용 챕터를 묶은 표 | Item 31 (`use-tools`) |
| ✅ | `ch32-continuous-integration` | CI에 넣을 단계(빌드, 테스트, 서식, lint, 문서, 기능 조합, MSRV, 의존성 점검), 원칙(로컬에서 재현, 경고는 실패, 도구 버전 고정) | 로컬에서 같은 검사를 돌리는 스크립트와 GitHub Actions 워크플로 예시(`projects/`) | Item 32 (`ci`) |

#### Beyond Standard Rust — ch33~ch35

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ✅ | `ch33-no-std` | `core`·`alloc`·`std`의 구분, 라이브러리를 `no_std` 호환으로 쓰기, `std` 기능 패턴, 실패할 수 있는 할당 | `no_std` 라이브러리와 `std` 기능(`projects/`), 호스트에서 테스트, `try_reserve` | Item 33 (`no-std`) |
| ✅ | `ch34-ffi-boundaries` | Rust에서 C 함수 부르기(링크, 이름 맹글링, 타입 대응), C 데이터 다루기(`repr(C)`), 경계에서의 소유권과 라이프타임, C에서 Rust 부르기, 안전한 래퍼 | `build.rs`로 C 코드를 컴파일해 호출, `repr(C)` 구조체 주고받기, 불투명 포인터와 생성·해제 함수 쌍, C에서 부르는 Rust 함수(`projects/`) | Item 34 (`ffi`) |
| ✅ | `ch35-bindgen` | 손으로 쓴 선언이 어긋날 때의 위험, 헤더에서 선언 생성하기, `-sys` 크레이트 관례, C++ 연동 도구 소개 | ch34의 C 라이브러리를 손으로 쓴 선언과 생성한 선언으로 비교(`projects/`) | Item 35 (`bindgen`) |

### 부록 B. Rust FAQ — `appd-b/`

C/C++, Java, C#, Go, Python, JavaScript/TypeScript를 써 온 프로그래머가 Rust를 처음 볼 때 묻는 질문에 답합니다. 본편이 "무엇이 있고 어떻게 쓰는가"를, 부록 A가 "여러 방법 중 무엇을 왜 고르는가"를 다룬다면, 부록 B는 "왜 그렇게 만들었고, 내가 쓰던 언어와 무엇이 다른가"를 다룹니다. 챕터 하나가 주제 하나를 맡고, 그 안의 질문 7~12개에 `Q4.2`(ch4의 두 번째 질문)처럼 번호를 붙입니다.

- 선수 과정: Part 0. 문법과 API 사용법은 본편 챕터로 링크하고 반복하지 않습니다.
- README는 2.4절의 여섯 항목을 유지하되, 첫머리에 질문 표(번호, 질문, 한 줄 답)를 두고, 핵심 개념을 질문마다 "짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기" 순서로 풉니다.
- 근거: 언어 동작은 저장소 툴체인에서 실행한 것만 씁니다. 통계·도입 사례·표준화 현황은 1차 자료를 열어 확인한 것만 쓰고, 누가 언제 무엇을 센 숫자인지와 발행일·확인일을 밝힙니다.
- 비교 코드: C/C++ 코드는 챕터의 `compare/`에 두고 새니타이저를 켜서 컴파일·실행해 확인합니다. 그 밖의 언어는 README 안의 짧은 조각으로만 싣습니다.
- 준비: 비교 코드에는 C/C++ 컴파일러(Apple clang)를, ch8은 `trpl`을 씁니다. 그 밖의 추가 도구는 쓰지 않습니다.
- 질문 목록, 챕터별 작성 메모, 근거 자료의 확인 상태는 `plan/plan-rust-faq.md`에 있습니다.

#### 왜 Rust인가 — ch1~ch3

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 근거 자료 |
| --- | --- | --- | --- | --- |
| ✅ | `ch1-why-rust` | Rust가 풀려는 문제(안전성·속도·동시성을 함께), 쓰이는 분야, 조직들이 옮기는 이유(보안·자원·생산성), 사용 현황, 맞지 않는 곳 | 세 가지 약속 맛보기: GC 없이 정해진 때 해제, 비용 없는 추상화, 검사되는 스레드 공유 | 보안 보고서(Microsoft, Chromium, Android), 정부 권고(CISA, ONCD), 도입 사례(Discord, Cloudflare, Linux 커널), 개발자 설문 |
| ✅ | `ch2-memory-safety` | 메모리 안전성의 뜻(공간적·시간적), 메모리 버그가 취약점이 되는 이유, "70%"의 출처, Rust가 막는 방법(컴파일 시간과 실행 시간), GC 언어와의 차이, 메모리 밖의 안전장치 | 경계 밖 접근이 패닉으로 멈춤, 해제 후 사용과 초기화하지 않은 변수가 컴파일되지 않음, 명시적 변환 | 보안 보고서, Book 4장 |
| ✅ | `ch3-safety-limits` | Rust가 보장하지 않는 것: 논리 버그, 누수, 교착 상태, 오버플로, 패닉. `unsafe`의 범위와 책임, Rust 코드의 취약점, 공급망 | 안전한 코드로 만든 누수, 교착 상태 관찰, 오버플로의 디버그·릴리스 동작, 컴파일되는 논리 버그, `unsafe` 안에서도 유지되는 검사 | Reference "Behavior not considered unsafe", Rustonomicon, Android 보안 블로그 |

#### 다른 언어와 무엇이 다른가 — ch4~ch8

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 근거 자료 |
| --- | --- | --- | --- | --- |
| ✅ | `ch4-memory-without-gc` | GC도 `free`도 없는 메모리 관리: 소유권과 스코프, 이동과 `Copy`, 참조와 대여 규칙, `clone`의 비용, `Rc`/`Arc`와 GC의 차이, 라이프타임 표기 | drop 시점 출력, 이동과 복사, 넘기는 대신 빌려주기, `Rc`의 참조 수와 `Weak`, `clone` 비용 | Book 4·10·15장 |
| ✅ | `ch5-no-null-no-exceptions` | `null` 대신 `Option`, 예외 대신 `Result`와 `?`, `finally` 대신 `Drop`, 패닉과 예외의 차이, `unwrap`의 자리, 에러 타입 | `Option`으로 바꾼 널 검사, `Result`와 `?` 전파, `Drop`으로 정리, 패닉과 `Result` 비교 | Book 6·9장 |
| ✅ | `ch6-no-classes-inheritance` | 클래스·상속 없는 설계: 구조체와 `impl`, 생성자 관례, 조합과 트레이트, 인터페이스와의 차이, 제네릭 구현 방식 비교, 오버로딩·기본 인자의 대안, 열거형으로 옮기는 타입 계층 | `new`와 `Default`, 조합, 남의 타입에 트레이트 구현, 정적·동적 디스패치, 계층 대신 열거형, 오버로딩 대안 | Book 5·10·18장 |
| ✅ | `ch7-syntax-type-surprises` | 문법과 타입에서 낯선 것: 불변 기본과 섀도잉, 표현식 기반 문법, 암묵적 변환이 없는 수 타입, `String`과 `&str`, UTF-8과 인덱싱, 시그니처의 타입 표기, 매크로 `!`, 완전한 `match` | 불변과 섀도잉, 표현식, 수 변환, 문자열 두 타입, UTF-8 인덱싱, 완전성 검사 | Book 3·6·8장 |
| ✅ | `ch8-concurrency-async` | 데이터 경쟁을 컴파일 시간에 막는 방법(`Send`/`Sync`), 데이터를 감싸는 `Mutex`, 다른 언어의 동시성 모델과 비교, 표준 라이브러리에 비동기 런타임이 없는 이유, 게으른 퓨처, 전역 상태 | 거부되는 데이터 경쟁과 고친 판, 데이터를 소유한 뮤텍스, 채널, 스코프 스레드, `await` 전에는 돌지 않는 퓨처, 전역 상태 | Book 16·17장 |

#### C/C++의 문제를 어떻게 풀었나 — ch9~ch13

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 근거 자료 |
| --- | --- | --- | --- | --- |
| ✅ | `ch9-c-memory-errors` | 메모리 오류를 종류별로: 해제 후 사용, 이중 해제, 댕글링 포인터, 버퍼 오버플로, 초기화하지 않은 메모리, 널 역참조, 반복자 무효화, 문자열 버그. 오류마다 Rust의 장치와 잡는 시점 | 오류 종류마다 C 비교 코드와 Rust 해법 한 쌍 | Chromium 보안 보고서, Book 4·8·15장 |
| ✅ | `ch10-undefined-behavior` | 미정의 동작의 뜻과 위험, "안전한 Rust에는 미정의 동작이 없다"의 의미, 정수 오버플로·암묵적 변환·평가 순서·별칭 규칙의 차이, `unsafe` Rust의 미정의 동작 | 정의된 오버플로, 변환이 필요한 비교, 0으로 나누기와 시프트, 평가 순서, 배타적 `&mut` | Reference "Behavior considered undefined" |
| ⬜ | `ch11-cpp-raii-move` | C++의 RAII·스마트 포인터·이동 의미론과 비교: 파괴적 이동, 명시적 복제, `Drop`과 Rule of three/five/zero, `const`와 불변 기본, 표준 타입 대응 | 이동 뒤 사용 불가, 명시적 `clone`, drop 순서, 스마트 포인터 대응, 공유와 변경의 배타성, `variant`와 열거형 | Book 4·15장 |
| ⬜ | `ch12-modern-cpp-enough` | "모던 C++와 도구로 충분하지 않은가": 스마트 포인터가 남기는 문제, 새니타이저와 정적 분석의 범위, C++ 표준의 안전성 작업, 안전이 기본값이라는 것의 의미, C++가 더 나은 점 | 모던 C++로 써도 새니타이저에 걸리는 조각과 같은 구조의 Rust 컴파일 에러 | C++ 표준화 문서, Linux 커널 |
| ⬜ | `ch13-headers-build-templates` | 헤더·전처리기·빌드 시스템·패키지 관리의 대응: 모듈과 가시성, `cfg`, 위생적인 매크로, Cargo, 정의 시점에 검사하는 제네릭과 템플릿의 차이 | 헤더 없는 모듈, 매크로 위생, 정의 시점 검사, `cfg`, const 제네릭 | Book 7·10·20장, Cargo Book |

#### 실무에서 묻는 것 — ch14~ch17

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 근거 자료 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch14-performance` | C/C++와의 성능 비교를 읽는 법, 비용 없는 추상화, 경계 검사와 오버플로 검사의 비용, 느린 Rust 프로그램의 흔한 원인, 런타임과 GC 멈춤, 바이너리 크기 | 반복자와 반복문, `Option`의 크기, 경계 검사를 피하는 모양, 버퍼링, 디버그·릴리스 실행 비교 | 도입 사례(Discord, Cloudflare), Book 13장, Cargo Book |
| ⬜ | `ch15-pain-points` | 어렵다고 하는 이유: 대여 검사기와 싸우는 전형, 컴파일러가 거부하는 올바른 코드, 연결 리스트·그래프·자기 참조, 컴파일 시간과 `target/` 크기, 디버깅, 버려야 할 습관 | 대여 에러 전형 세 가지와 해법, 인덱스로 만든 그래프, `Rc<RefCell<T>>` 트리, 보수적 거부와 우회, entry API | State of Rust 설문, Book 4·15장 |
| ⬜ | `ch16-tooling-ecosystem` | Cargo와 다른 언어의 패키지 관리자 비교, 작은 표준 라이브러리, 크레이트 고르기와 공급망, 릴리스 주기·안정성 보장·에디션, 명세와 컴파일러 구현, ABI, 관리 주체, 지원 플랫폼 | 표준 라이브러리만으로 되는 일의 범위, 의존성 하나를 가진 프로젝트의 `Cargo.lock`과 `cargo tree`(`projects/`) | Rust 블로그(명세), gccrs, Cargo Book, Edition Guide |
| ⬜ | `ch17-interop-migration` | 기존 코드와 함께 쓰기: C 호출과 C에서 호출되기, C++ 연동의 한계, 다른 언어에서 Rust 부르기, 자동 번역의 현황, 어디부터 옮길지, 도입 비용과 판단 기준 | libc 함수 호출, C 프로그램의 함수 하나를 Rust 구현으로 바꿔 끼우기(`projects/`) | Android 보안 블로그, CISA·NSA 지침, DARPA TRACTOR, Rustonomicon FFI |

---

## 4. 진행 현황

| 파트 | 완료 / 전체 |
| --- | --- |
| 사전 작업 | 3 / 3 |
| Part 0. 시작하기 | 20 / 20 |
| Part 1. 표준 라이브러리 | 14 / 14 |
| Part 2. 에디션 | 6 / 6 |
| Part 3. Cargo | 11 / 11 |
| Part 4. rustdoc | 0 / 4 |
| Part 5. rustc | 0 / 7 |
| Part 6. 컴파일러 에러 색인 | 0 / 6 |
| Part 7. 명령줄 애플리케이션 | 0 / 7 |
| Part 8. 임베디드 | 0 / 8 |
| Part 9. 레퍼런스 | 0 / 10 |
| Part 10. Rustonomicon | 0 / 12 |
| Part 11. 불안정 기능 | 0 / 4 |
| **합계** | **51 / 109 챕터** |
| 부록 A. Effective Rust (합계와 별도) | 35 / 35 |
| 부록 B. Rust FAQ (합계와 별도) | 10 / 17 |
