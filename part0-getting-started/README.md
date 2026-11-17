# Part 0. 시작하기

Rust 언어의 기초 전체를 다루는 파트입니다. [The Rust Programming Language](https://doc.rust-lang.org/book/)(The Book)의 장 순서를 따르고, 챕터마다 대응하는 [Rust By Example](https://doc.rust-lang.org/rust-by-example/) 절과 [Rustlings](https://github.com/rust-lang/rustlings/) 연습 주제를 함께 안내합니다. 이후 모든 파트가 이 파트의 내용을 전제로 합니다.

The Book 14장(Cargo와 crates.io)은 Part 3에서 다룹니다.

## 학습 방법

챕터 번호 순서대로 진행합니다. 각 챕터의 `README.md`를 읽으면서 예제를 직접 실행해 봅니다.

```sh
cargo run -p <패키지 이름> --example <예제 이름>
```

패키지 이름과 예제 이름은 각 챕터 README의 "예제 안내"에 적혀 있습니다.

## 챕터 목차

| 챕터 | 주제 |
| --- | --- |
| [ch1-hello-cargo](ch1-hello-cargo/) | 설치(rustup/mise), `rustc`로 컴파일, `cargo new/build/run/check`, `println!` 서식 출력 |
| [ch2-guessing-game](ch2-guessing-game/) | 입력 받기, `let`/`mut`, 외부 크레이트(`rand`), `match`, 반복 — 숫자 맞히기 게임을 4단계로 만들기 |
| [ch3-common-concepts](ch3-common-concepts/) | 변수와 가변성, 섀도잉, 스칼라/복합 타입, 함수, 구문과 표현식, 제어 흐름 |
| [ch4-ownership](ch4-ownership/) | 소유권 규칙, 이동과 복사, 참조와 대여, 슬라이스 |
| [ch5-structs](ch5-structs/) | 구조체 정의와 갱신 문법, 튜플/유닛 구조체, `derive(Debug)`와 `dbg!`, 메서드와 연관 함수 |
| [ch6-enums-pattern-matching](ch6-enums-pattern-matching/) | 열거형, `Option<T>`, `match`, `if let`, `let...else` |
| [ch7-modules](ch7-modules/) | 패키지·크레이트·모듈, 경로, `pub`, `use`, 파일 분리 — 라이브러리 + 바이너리 프로젝트(`restaurant`) |
| [ch8-collections](ch8-collections/) | `Vec<T>`, `String`과 UTF-8, `HashMap<K, V>` — 단어 빈도수, 중앙값·피그 라틴 풀이 |
| [ch9-error-handling](ch9-error-handling/) | `panic!`, `Result<T, E>`, `?` 연산자, 패닉과 `Result`의 선택 기준, 검증 타입 |
| [ch10-generics-traits-lifetimes](ch10-generics-traits-lifetimes/) | 제네릭과 단형성화, 트레이트 정의·구현·바운드(`impl Trait`, `where`), `Display` 구현, 라이프타임 표기와 생략 규칙 |
| [ch11-testing](ch11-testing/) | `#[test]`, 단언 매크로, `should_panic`, 실행 옵션, 단위·통합·문서 테스트 — 라이브러리 프로젝트(`adder`) |
| [ch12-io-project-minigrep](ch12-io-project-minigrep/) | 인자 파싱, 파일 읽기, 모듈화, TDD, 환경 변수, 표준 에러 — `minigrep` 프로젝트 |
| [ch13-closures-iterators](ch13-closures-iterators/) | 클로저 캡처와 `Fn` 트레이트, `Iterator`와 어댑터, 성능 — `minigrep`을 반복자로 개선 |
| [ch14-smart-pointers](ch14-smart-pointers/) | `Box<T>`, `Deref`/`Drop`, `Rc<T>`, `RefCell<T>`와 내부 가변성, 순환 참조와 `Weak<T>` |
| [ch15-concurrency](ch15-concurrency/) | 스레드와 `join`, `move` 클로저, 채널(`mpsc`), `Mutex<T>`/`Arc<T>` 공유 카운터, `Send`/`Sync` |
| [ch16-async](ch16-async/) | `async`/`await`, 퓨처와 런타임, `join`·`select`, 비동기 채널, 스트림, `Pin`, 태스크와 스레드 (`trpl` 크레이트) |
| [ch17-trait-objects-oop](ch17-trait-objects-oop/) | 캡슐화, 트레이트 객체(`dyn Trait`)와 동적 디스패치, 상태 패턴(객체형·타입형) |
| [ch18-patterns](ch18-patterns/) | 패턴이 쓰이는 위치, 반박 가능성, 구조 분해·가드·`@` 바인딩·범위·무시·슬라이스 패턴 |
| [ch19-advanced-features](ch19-advanced-features/) | `unsafe` 기초, 연관 타입·연산자 오버로딩·슈퍼트레이트, 뉴타입·타입 별칭·never 타입, 함수 포인터, `macro_rules!`와 derive 매크로(`hello_macro`) |
| [ch20-web-server-project](ch20-web-server-project/) | TCP 연결과 HTTP 요청·응답, 스레드 풀, 우아한 종료 — 멀티스레드 웹 서버(`hello-server`) |

Part 0의 20개 챕터가 모두 작성됐습니다. 전체 계획은 [로드맵](../docs/roadmap.md)을, 진행 상황은 [PROGRESS.md](../PROGRESS.md)를 참고합니다.
