# 진척 현황

[로드맵](docs/roadmap.md)의 챕터별 작업 상태를 기록합니다. 챕터 하나가 끝날 때마다 이 문서와 로드맵의 상태 칸을 함께 갱신합니다.

- 마지막 갱신: 2026-11-19 (부록 A ch1~ch15 추가)
- 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024). 2026-11-18에 1.98.1에서 올렸습니다. Part 0과 Part 1 ch1~ch4의 본문과 예제 출력은 1.98.1에서 작성·검증했고, 1.100.0에서는 아래 "툴체인 1.100.0 확인"의 범위만 다시 확인했습니다.

상태 표기: ⬜ 대기 · 🔄 작성 중 · ✅ 완료(검증 통과) · ⚠️ 부분 완료(비고 참고)

## 요약

| 구분 | 완료 / 전체 |
| --- | --- |
| 사전 작업 | 3 / 3 |
| Part 0. 시작하기 | 20 / 20 |
| Part 1. 표준 라이브러리 | 14 / 14 |
| Part 2. 에디션 | 6 / 6 |
| Part 3~11 | 0 / 69 |
| 부록 A. Effective Rust (본편 합계와 별도) | 19 / 35 |

## 사전 작업

| 상태 | 작업 | 완료일 |
| --- | --- | --- |
| ✅ | 루트 `Cargo.toml` (워크스페이스) | 2026-11-17 |
| ✅ | `.gitignore` | 2026-11-17 |
| ✅ | 루트 `README.md` | 2026-11-17 |

## Part 0. 시작하기 — `part0-getting-started/`

| 상태 | 챕터 | 산출물 | 검증 | 완료일 | 비고 |
| --- | --- | --- | --- | --- | --- |
| ✅ | `ch1-hello-cargo` | 예제 5 / 에러 2 / 프로젝트 1 | 통과 | 2026-11-17 | `projects/hello_cargo`. `rustc` 명령에는 `--edition 2024`를 붙임(기본 에디션이 2015). 서식 인자 개수 불일치 에러는 E 코드가 없어 메시지로 확인 |
| ✅ | `ch2-guessing-game` | 예제 4 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-17 | `rand` 0.10.4(dev-dependency). 완성판은 파이프 검증을 위해 EOF에서 종료 |
| ✅ | `ch3-common-concepts` | 예제 13 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-17 | 예제가 기준(3~6개)보다 많음. `04`, `06`은 표준 입력을 받고 특정 값에서 일부러 패닉(종료 코드 101). `static`, 블록 주석, `overflow-checks`는 다루지 않음 |
| ✅ | `ch4-ownership` | 예제 7 / 에러 6 / 프로젝트 0 | 통과 | 2026-11-17 | drop 시점 관찰용으로 구조체와 `impl Drop`을 미리 사용(ch5, ch14 안내). 부분 이동, `std::mem::drop`은 다루지 않음 |
| ✅ | `ch5-structs` | 예제 7 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-17 | `Display` 직접 구현과 참조 필드의 라이프타임은 ch10으로 미룸. `derive(Clone)`은 E0382 해결책으로만 언급 |
| ✅ | `ch6-enums-pattern-matching` | 예제 7 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-17 | `let...else`의 `else`는 `return`뿐 아니라 흐름을 끊으면 된다고 실제 동작대로 설명. `ref`, `..`, 가드 등은 ch18로 미룸 |
| ✅ | `ch7-modules` | 예제 7 / 에러 4 / 프로젝트 1 | 통과 | 2026-11-17 | `projects/restaurant`(lib + bin). E0451·E0583·E0432·E0761은 README 본문에서 인용. `pub(crate)`는 참고 자료에서만 소개 |
| ✅ | `ch8-collections` | 예제 11 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-17 | 예제가 기준보다 많음. 패닉 예제 없이 `get`으로 같은 상황을 관찰. The Book 연습 풀이 2개(중앙값·최빈값, 피그 라틴) 포함 |
| ✅ | `ch9-error-handling` | 예제 8 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-17 | `01`·`04`는 일부러 패닉(종료 코드 101), `07`은 일부러 `Err` 반환(종료 코드 1). 파일 에러 메시지는 macOS 기준. 사용자 정의 에러 타입은 Part 1 ch8로 미룸 |
| ✅ | `ch10-generics-traits-lifetimes` | 예제 9 / 에러 6 / 프로젝트 0 | 통과 | 2026-11-17 | 세 주제를 담아 README가 1144줄. 라이프타임 그림은 Book과 달리 NLL 기준(참조가 쓰이는 구간)으로 그림. 트레이트 객체·연관 타입·`T: 'static`은 다루지 않음 |
| ✅ | `ch11-testing` | 예제 4 / 에러 2 / 프로젝트 1 | 통과 | 2026-11-17 | `projects/adder`(lib + 얇은 bin, 단위·통합·문서 테스트). 예제 안의 실패 데모 테스트는 `#[ignore]`로 막아 둠 — 예제에 `--ignored`를 주면 일부러 실패함. README 1253줄 |
| ✅ | `ch12-io-project-minigrep` | 예제 4 / 에러 3 / 프로젝트 1 | 통과 | 2026-11-17 | `projects/minigrep`(lib + bin, 테스트 5개). Book과 달리 `Config`·`run`도 `lib.rs`에 둠. 클로저 대신 `match`/`if let Err` 사용(ch13에서 개선 예고). PowerShell 명령은 미검증 |
| ✅ | `ch13-closures-iterators` | 예제 8 / 에러 5 / 프로젝트 1 | 통과 | 2026-11-18 | `projects/minigrep`은 ch12 판을 복사해 반복자·클로저로 개선(테스트 6개). `08`은 성능 측정이라 시간 출력이 실행마다 다름. README 1202줄 |
| ✅ | `ch14-smart-pointers` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-17 | `06`은 `-- panic` 인자를 줄 때만 일부러 패닉(종료 코드 101). `07`은 `cargo test --example`로 테스트 3개 실행. `Cell`/`OnceCell`, 니치 최적화는 Part 1 ch7로 미룸 |
| ✅ | `ch15-concurrency` | 예제 7 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-17 | `02`는 일부러 스레드 하나를 패닉시킴(프로그램 종료 코드는 0). `06`의 3번 구역은 일부러 틀린 결과를 냄(값이 실행마다 다름). 교착 상태는 설명만. 스코프 스레드·`RwLock`·원자 타입은 Part 1 ch11로 미룸 |
| ✅ | `ch16-async` | 예제 8 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-18 | `trpl` 0.3.0(dev-dependency, 크레이트 약 135개를 끌어옴). Book의 웹 요청 예제는 타이머로 대체(외부 네트워크 사용 안 함). 시간은 100ms 단위로 반올림해 출력. macOS 외 환경은 미확인 |
| ✅ | `ch17-trait-objects-oop` | 예제 6 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-18 | Book과 달리 `Draw::draw`가 `String`을 돌려주고 `AveragedCollection`에 `new()`를 추가함. 열거형 상태 패턴은 README 코드로만 소개. dyn 호환성은 현재 컴파일러 용어("dyn compatible")로 설명 |
| ✅ | `ch18-patterns` | 예제 8 / 에러 6 / 프로젝트 0 | 통과 | 2026-11-18 | let 체인, 가드 자리의 `if let`, `a..b` 범위 패턴 등 Book에 없는 문법은 1.98.1에서 확인해 실음. `..`를 두 번 쓴 에러는 E 코드가 없어 메시지로 확인. Edition 2024 바인딩 모드 규칙 상세는 Part 9 ch6으로 미룸. README 1283줄 |
| ✅ | `ch19-advanced-features` | 예제 8 / 에러 6 / 프로젝트 1 | 통과 | 2026-11-18 | `projects/hello_macro`(크레이트 3개 워크스페이스, `syn` 3.0.10·`quote` 1.0.47). unsafe 코드는 Miri로 검증하지 못함(nightly 미설치) — 원시 포인터 예제는 보수적으로 작성. `01`의 주소 출력은 실행마다 다름. README 1272줄 |
| ✅ | `ch20-web-server-project` | 예제 4 / 에러 4 / 프로젝트 1 | 통과 | 2026-11-18 | `projects/hello-server`(lib + bin, 테스트 7개 + 문서 테스트 1개). 127.0.0.1:7878에서 연결 3개(인자로 변경 가능)를 처리하고 우아하게 종료. Book과 달리 `build`가 `Result` 반환, `Drop`에 `drain` 사용. 실제 브라우저와 macOS 외 OS는 미확인 |

- 산출물: 예제 / 컴파일 에러 예제 / 독립 프로젝트 개수. Part 0 합계는 예제 143 / 에러 86 / 프로젝트 7입니다.
- 검증: `cargo fmt --check`, `cargo build --examples`, `cargo clippy -- -D warnings`, 예제 실행, `compile_fail` 에러 코드 확인. 챕터를 쓴 에이전트의 검증과 별도로 메인 세션이 한 번 더 실행했습니다.
- Part 0 전체 확인(2026-11-18): 저장소 루트에서 `cargo fmt --all --check`, `cargo build --workspace --examples`, `cargo clippy --workspace --examples -- -D warnings`가 경고 없이 통과했고, `web/`의 사이트 빌드도 깨진 링크 경고 없이 통과했습니다.
- 모든 검증은 macOS(aarch64)에서만 했습니다. 다른 운영체제에서는 파일 에러 메시지, 종료 코드, 타이머에 의존하는 출력이 다를 수 있습니다.

## Part 1. 표준 라이브러리 — `part1-standard-lib/`

| 상태 | 챕터 | 산출물 | 검증 | 완료일 | 비고 |
| --- | --- | --- | --- | --- | --- |
| ✅ | `ch1-std-overview` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-18 | 패닉하는 예제 없음. 문서 화면 설명은 로컬 1.98.1 문서 기준(온라인 문서는 1.100.0) — 버전이 어긋나는 예로 `bool::toggle`을 실음. 문서 검색 결과는 브라우저가 필요해 직접 확인하지 못함. `bool::ok_or`(1.98) 사용. `usize::BITS` 등은 64비트 환경 기준. `arithmetic_overflow`는 Part 0 ch3에 있어 에러 예제에서 뺌. README 896줄 |
| ✅ | `ch2-option-result` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-18 | 패닉하는 예제 없음. README가 1077줄로 목표(600~900줄)보다 김. `collect` 타입 추론 실패(E0283)는 Part 0 ch13에 있어 `Result` 원소를 `Vec<T>`로 `collect`하는 E0277로 대체. 최근 안정화된 `map_or_default`(1.98)·`Result::flatten`(1.89)·`is_none_or`(1.82)·`take_if`(1.80) 포함. `unused_must_use`는 README 표로만 다룸 |
| ✅ | `ch3-strings-fmt` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-18 | `06`의 `{:p}` 주소는 실행마다 다름. `str::substr_range`(1.98)·`floor_char_boundary`(1.91)처럼 최근 안정화된 API를 써서 오래된 툴체인에서는 `02`·`05`가 컴파일되지 않음. `parse()` 타입 추론 실패(E0282)는 Part 0 ch2의 에러 예제로 연결. 서식 문자열 에러 4종은 README 표로만 실음. README 975줄 |
| ✅ | `ch4-collections` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-18 | `05`의 시간 측정 두 줄만 실행마다 다름. 불안정 정렬 결과(`02`), `as_slices`(`05`), 힙 내부 순서(`08`), `capacity`(`01`)는 이 툴체인의 구현 결과라 툴체인을 올리면 다시 확인. `BinaryHeap`에 `Ord` 없는 타입을 넣는 에러는 README 본문에만 인용. `extract_if`·`get_disjoint_mut` 등 최근 안정화된 API 포함. README가 1060줄로 목표(700~1000줄)보다 김 |
| ✅ | `ch5-iterators` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-18 | 1.100.0에서 작성·검증. 패닉하는 예제 없음. `type_name` 출력(`01`·`03`·`07`)은 툴체인에 따라, `size_of_val` 값(`03`)은 플랫폼에 따라 달라질 수 있음. `std::iter::chain`(1.91)·`repeat_n`(1.82)·`is_sorted`(1.82) 등 최근 안정화된 API 사용. `sum` 추론 실패(E0283)와 `FromIterator` 없는 타입으로 `collect`(E0277)는 앞 챕터와 겹쳐 README 표로만 실음. 에러 예제 두 개는 E0599가 함께 나옴. README 1000줄(출력 블록은 핵심 구역만) |
| ✅ | `ch6-core-traits` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-18 | 1.100.0에서 작성·검증. 패닉하는 예제 없음. 해시 값과 주소는 출력하지 않고 같은지만 출력. `PartialEq` 없는 `==`(E0369)는 Part 0 ch11에 있어 뺌. `derive(Eq)`·`derive(Ord)` 누락 에러와 clippy가 잡는 실수 8가지는 README 표로만 실음. Rustlings 연습은 현재 이름(`conversions1`~`5`)으로 안내. `Infallible`은 1.100.0에서 `!`의 별칭이라 실제 동작대로 설명. README 941줄 |
| ✅ | `ch7-smart-pointers-memory` | 예제 8 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-18 | 1.100.0에서 작성·검증. 패닉하는 예제와 unsafe 코드 없음. 크기·오프셋은 64비트·1.100.0 기준이고, `01`의 필드 오프셋과 `02`의 3·4번 구역(문서가 보장하지 않는 니치)은 툴체인을 올리면 다시 확인. `static`에 두는 예는 `Cell` 대신 `OnceCell`로 만듦. E0716·E0499는 README 본문에만 인용. README 1006줄(`01`·`02`·`04`의 출력 블록은 일부 생략) |
| ✅ | `ch8-errors-panic` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-18 | 1.100.0에서 작성·검증. `06`·`07`은 일부러 패닉하지만 안에서 붙잡아 종료 코드 0(표준 에러에 실행마다 다른 스레드 ID가 나옴). `08`은 `RUST_LIB_BACKTRACE` 값에 따라 출력이 달라 실행 명령에 환경 변수를 명시. `payload_as_str`(1.91)·`PanicHookInfo`(1.81) 사용. `Display` 없는 `impl Error`(E0277)는 Part 0 ch19와 겹쳐 README 표로만 실음. README 934줄 |
| ✅ | `ch9-io-fs-path` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-19 | 1.100.0에서 작성·검증. 패닉하는 예제 없음. `04`는 표준 입력을 읽음(파이프로 넣어 실행, 입력이 없으면 요약만 표준 에러에 출력하고 종료 코드 0). `02`에 `#[cfg(test)]` 테스트 2개(`cargo test --example`). 파일은 임시 디렉터리에 만들고 지움 — `06`의 임시 경로 두 줄은 실행마다 다름. 운영체제 에러 번호와 메시지는 macOS 기준. `use std::io::Write` 누락 대신 `fmt::Write`로 파일에 `writeln!`하는 E0599를 실음. 터미널 여부 확인에 `IsTerminal` 사용. README 961줄 |
| ✅ | `ch10-env-process-time` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-18 | 1.100.0에서 작성·검증. 모든 예제가 종료 코드 0 — 0이 아닌 코드·`process::exit`·패닉은 자식 쪽에서만 일어남. `02`·`03`·`04`·`08`은 예제가 자기 자신을 자식으로 다시 실행하고, `03`의 6번·`04`의 5번 구역만 유닉스 명령(`sh`·`echo`·`sort`·`uniq`)을 씀. `06`의 "걸린 시간" 한 줄만 실행마다 다름. `08`은 약 1초 걸림. `Duration`에 실수를 곱하는 에러는 지침의 E0277이 아니라 E0308. 자기 환경은 바꾸지 않음(`set_var` 미사용). Linux·Windows 동작은 미확인. README 957줄 |
| ✅ | `ch11-threads-sync` | 예제 8 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-18 | 1.100.0에서 작성·검증. unsafe 코드 없음. `01`·`02`·`03`은 일부러 스레드를 패닉시키지만 종료 코드 0(표준 에러에 실행마다 다른 스레드 ID). `07`의 "load 후 store" 줄은 일부러 만든 경쟁 조건이라 실행마다 다름. `02`의 `available_parallelism` 값과 스레드 생성 실패 메시지는 환경에 따라 다름. `fetch_update`가 1.99부터 deprecated라 `try_update`(1.95) 사용. `Arc` 안의 값 수정(E0596)은 Part 0 ch14와 겹쳐 뺌. README 957줄 |
| ✅ | `ch12-net` | 예제 7 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-18 | 1.100.0에서 작성·검증. 모든 소켓은 `127.0.0.1`의 포트 0(운영체제가 고름)에서만 열고 포트 번호는 출력하지 않음. 패닉하는 예제 없음. `03`은 교착 상태를 보이려고 64 MiB를 주고받아 메모리를 수백 MiB 쓰고 약 1초 걸림. 제한 시간 초과가 `WouldBlock`으로 나오는 것, `ConnectionRefused`, `localhost` 해석 결과, UDP 최대 크기 9216바이트는 macOS 기준. `recv_from`에 `String`을 넘기는 E0308은 README 표로만 실음. README 838줄 |
| ✅ | `ch13-async-primitives` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-19 | 1.100.0에서 작성·검증. 외부 크레이트와 unsafe 코드 없이 표준 라이브러리만으로 퓨처·`block_on`·작은 실행기를 구현. 패닉하는 예제와 표준 에러 출력 없음. 시간과 바쁜 폴링 횟수는 비교 결과로만 출력해 실행마다 출력이 같음. `Send`가 아닌 값을 await 너머로 들고 가는 에러는 E 코드가 없어 메시지로 확인. 재귀 `async fn`(E0733)을 에러 예제로 추가. 타이머 스레드는 `join`하지 않지만 예제가 끝까지 기다림. README 824줄 |
| ✅ | `ch14-ffi-any-misc` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-19 | 1.100.0에서 작성·검증. 패닉하는 예제 없음. unsafe 코드는 `04_calling_c`에만 있고(`strlen`·`getenv` 호출과 `CStr::from_ptr`, 블록마다 `SAFETY` 주석) **Miri로는 검증하지 못함**(nightly 미설치). `02`의 3·5번 구역은 유닉스 전용(`cfg(unix)`). C 타입 별칭은 macOS aarch64 기준(`c_char`는 `i8`). 임시 `CString`의 `as_ptr`는 에러가 아니라 경고(`dangling_pointers_from_temporaries`)라 README에 경고문만 인용. 트레이트 객체 업캐스트(1.86) 사용. README 956줄 |

- Part 1 전체 확인(2026-11-19, rustc 1.100.0): 저장소 루트에서 `cargo fmt --all --check`, `cargo build --workspace --examples`, `cargo clippy --workspace --examples -- -D warnings`가 경고 없이 통과했고, `compile_fail` 파일 68개가 모두 기대한 에러로 실패하며, `web/`의 사이트 빌드도 깨진 링크 경고 없이 통과했습니다. ch1~ch4는 1.98.1에서, ch5~ch14는 1.100.0에서 작성·검증했습니다. Part 1 합계는 예제 111 / 에러 68 / 프로젝트 0입니다.
- 툴체인 1.100.0 확인(2026-11-18): 저장소 루트에서 `cargo fmt --all --check`, `cargo build --workspace --examples`, `cargo clippy --workspace --examples -- -D warnings`가 통과했습니다. `compile_fail` 파일 106개가 모두 파일 이름의 에러 코드로 실패하고, Part 1 예제 32개의 출력은 1.98.1 때와 같습니다(실행마다 다른 줄 제외). Part 0 예제의 출력, `projects/` 아래 독립 프로젝트, README에 인용한 에러 메시지 문구는 다시 확인하지 않았습니다.
- Clippy 1.100.0은 매크로가 펼친 코드에 `vec_init_then_push`를 내지 않습니다. Part 0 ch19 `08_macro_rules`의 `#[expect]` 두 개가 충족되지 않아 지우고 README 설명을 고쳤습니다.

## Part 2. 에디션 — `part2-editions/`

| 상태 | 챕터 | 산출물 | 검증 | 완료일 | 비고 |
| --- | --- | --- | --- | --- | --- |
| ✅ | `ch1-editions-overview` | 예제 2 / 에러 4 / 프로젝트 1 | 통과 | 2026-11-19 | `projects/edition_mix`(2015·2021·2024 크레이트 3개 워크스페이스, 1.100.0에 안정화된 `bool::toggle`을 써서 rustc 1.100.0 이상 필요). `02`는 `rustc`를 자식 프로세스로 실행해 에디션별 O/X 표를 출력(버전 줄은 툴체인마다 다름). 에러 예제는 에디션에 따라 결과가 달라 파일 머리말에 에디션별 결과를 적음 — 이름 끝의 `_2015`·`_2021`은 그 에디션에서만 실패한다는 뜻. 로컬 Edition Guide의 `rust-next/`는 리다이렉트뿐이라 다음 에디션 내용은 싣지 못함. `edition = "future"`는 nightly가 없어 미확인. 에디션과 버전의 차이를 보이려고 이미 설치돼 있던 1.98.1을 비교에 사용(2.5절). README 719줄 |
| ✅ | `ch2-rust-2015-2018` | 예제 3 / 에러 6 / 프로젝트 1 | 통과 | 2026-11-19 | `projects/scoreboard`(2015 라이브러리 `scorekit` + 같은 프로그램의 2015판·2018판, 두 판의 출력이 같음). `app2015`는 옛 관용구를 보이려고 린트 4개(`deprecated`·`anonymous_parameters`·`bare_trait_objects`·`ellipsis_inclusive_range_patterns`)를 `#[allow]`로 막음. Edition Guide와 달리 1.100.0에서는 2015에서도 `foo.rs` + `foo/bar.rs`가 되고 `extern crate` 없이도 식 안의 경로는 통함(막히는 것은 `use`와 매크로) — 실제 동작대로 설명. `tyvar_behind_raw_pointer`의 에러 코드는 E0282. `extern crate` 누락 에러는 단일 파일로 만들 수 없어 README 본문에만 인용. `cargo install`의 `--path` 요구는 실행하지 않아 미확인. README 902줄 |
| ✅ | `ch3-rust-2021` | 예제 6 / 에러 6 / 프로젝트 2 | 통과 | 2026-11-19 | `projects/same_code`(같은 소스 파일을 2018·2021 두 패키지가 컴파일, 여섯 구역의 출력이 서로 다름)와 `projects/feature_resolver`(리졸버 1·2의 기능 통합 차이). 기능 리졸버 실습은 빌드 스크립트를 아직 배우지 않아 build-dependency 대신 dev-dependency로 재현. `same_code`의 공유 소스는 2018 동작을 보이느라 `array_into_iter`·`non_fmt_panics`·`clippy::into_iter_on_ref`를 `#![allow]`로 막음. 패닉은 모두 안에서 붙잡아 종료 코드 0. Edition Guide와 다른 실제 동작(raw 라이프타임을 띄우는 위치는 `'r #a`, 페이로드가 문자열이 아닌 패닉의 메시지 형식, `&self` 메서드와 `TryInto`가 겹치면 에러 없이 결과가 바뀜)은 실제대로 설명. E0783과 2018 전용 에러 두 가지는 README 표로만 실음. README 942줄 |
| ✅ | `ch4-rust-2024-language` | 예제 7 / 에러 7 / 프로젝트 1 | 통과 | 2026-11-19 | `projects/edition_compare`(같은 소스 5개를 2021·2024 두 패키지가 컴파일, 실행 파일 10개). **never 타입 폴백은 1.100.0에서 에디션에 따른 차이가 아님** — 두 에디션 모두 폴백 타입이 `!`이고 에러 예제도 둘 다 E0277(1.98.1까지는 2021이 `()`), README 2.10절에 두 동작을 함께 적음. unsafe 코드는 예제 `05`·프로젝트 `new_syntax.rs`·에러 예제 하나에 있고 **Miri로는 검증하지 못함**(nightly 미설치). 교착 상태는 일으키지 않고 `try_write`·`try_borrow_mut`으로 관찰. `gen` 블록은 nightly가 없어 설명만. `link_section`은 플랫폼마다 형식이 달라 README 설명만(macOS 기준). 패턴의 `mut`/`ref`/`&` 제한·`static mut` 참조·빠진 프래그먼트 지정자의 에러는 README에만 인용. README가 1095줄로 목표(1000줄)보다 김 |
| ✅ | `ch5-rust-2024-ecosystem` | 예제 4 / 에러 5 / 프로젝트 2 | 통과 | 2026-11-19 | `projects/ecosystem_compare`(같은 소스와 문서 테스트를 2021·2024 두 패키지로 비교, 문서 테스트 8개)와 `projects/msrv_resolver`(리졸버 3 실습). **`msrv_resolver`는 crates.io에서 `smol_str`을 내려받음** — 고른 버전(0.3.2)은 2026-11-19 기준이며 `Cargo.lock`이 고정. `03`은 자기 환경을 `main` 첫머리에서만 바꾸고(unsafe, `SAFETY` 주석) 자기 자신을 자식으로 다시 실행, 4번 구역(`pre_exec`)과 에러 예제 `before_exec`는 유닉스 전용. `04`는 `rustfmt`를 자식 프로세스로 실행. unsafe 코드는 **Miri로 검증하지 못함**. Edition Guide와 다른 실제 동작: 프렐류드의 `poll` 충돌은 E0061이거나 에러 없이 호출 대상만 바뀜, `set_var` 자동 수정 주석은 `FIXME`, 상속 `default-features` 거부는 1.85~1.98의 동작(1.99부터는 덮어씀), 문서 테스트는 병합돼도 테스트마다 별도 프로세스로 실행. Cargo 매니페스트 에러는 README 표로만 실음. README 860줄 |
| ✅ | `ch6-migration-practice` | 예제 2 / 에러 3 / 프로젝트 1 | 통과 | 2026-11-19 | `projects/migration`(가계부 프로젝트의 단계별 스냅숏 `step0-2018`·`step1-2021`·`step2-2024`, 각각 통합 테스트 12개 + 문서 테스트 2개). `cargo fix --edition` 직후의 상태는 서식 검사를 통과하지 않아 스냅숏 대신 `diffs/*.diff` 두 개로 실음 — 스냅숏 사본에서 `cargo fix --edition --all-features`를 다시 돌려 두 diff와 같음을 확인. `step0-2018`은 옛 모양을 남기느라 린트 5개를 `#[allow]`로 막음. 세 스냅숏의 `cargo run` 출력은 2018·2021이 같고 2024는 임시값 범위 변경으로 두 줄의 순서가 다름(본문에서 설명). `cargo fix`는 저장소 밖 사본에서만 실행. Edition Guide와 다른 실제 동작: 옵션은 `--force-warn`(Guide의 `--force-warns`는 없는 옵션), 타깃별 `edition` 키는 deprecated, `set_var` 자동 수정 주석은 `FIXME`, `--edition-idioms`는 step0에서 0개를 고침, `if_let_rescope`는 가드에 `Deref`를 거쳐 메서드를 부르는 모양을 놓침. 미확인: 수정 실패 시 되돌림(back out) 메시지, `--target`으로 다른 플랫폼 코드 옮기기(타깃 미설치), nightly의 다음 에디션. unsafe 코드(`02`의 `set_var`/`remove_var`, 에러 예제 2개)는 **Miri로 검증하지 못함**. README 907줄 |

- Part 2 전체 확인(2026-11-19, rustc 1.100.0): 저장소 루트에서 `cargo fmt --all --check`, `cargo build --workspace --examples`, `cargo clippy --workspace --examples -- -D warnings`가 경고 없이 통과했고, `compile_fail` 파일 31개가 머리말에 적힌 에디션별 결과대로 실패·통과하며, `projects/` 아래 독립 프로젝트는 각 디렉터리에서 `cargo fmt --check`, `cargo clippy -- -D warnings`, 실행(테스트가 있으면 `cargo test`)을 확인했습니다. `web/`의 사이트 빌드도 깨진 링크 경고 없이 통과했습니다. Part 2 합계는 예제 24 / 에러 31 / 프로젝트 8입니다(ch3·ch5는 프로젝트 2개, ch6의 프로젝트는 스냅숏 3벌).
- Part 2는 모두 1.100.0에서 작성·검증했습니다. 에디션에 따른 동작 차이는 툴체인이 올라가면 달라질 수 있습니다(예: never 타입 폴백은 1.100부터 모든 에디션에 적용). nightly가 필요한 내용(`gen` 블록, `edition = "future"`)은 확인하지 못했습니다.

## Part 3~11

아직 시작하지 않았습니다. 챕터 목록은 [로드맵 3절](docs/roadmap.md#3-파트별-챕터-계획)을 따르며, 파트를 시작할 때 이 문서에 표를 추가합니다.

## 부록 A. Effective Rust — `appd-a/`

[Effective Rust](https://effective-rust.com/)의 Item 1개를 챕터 1개로 정리합니다(챕터 번호 = Item 번호, 패키지 이름 `er-ch{N}-{slug}`). 계획은 [로드맵 3절의 부록 A](docs/roadmap.md#3-파트별-챕터-계획)를 따릅니다. ch21~ch35의 행은 그 범위를 시작할 때 추가합니다.

| 상태 | 챕터 | 산출물 | 검증 | 완료일 | 비고 |
| --- | --- | --- | --- | --- | --- |
| ✅ | `ch1-express-data-structures` | 예제 6 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | 계획의 예제 3개를 6개로 늘림(정수 타입 고르기, 이름 붙인 필드, 에러 코드 대신 `Result` 추가). 패닉하는 예제와 외부 크레이트 없음. 책과 달라진 점은 컴파일러 출력의 표시 방식뿐(E0308의 `swap these arguments`, E0004의 `note`). 타입 크기는 macOS aarch64·1.100.0 기준이며 보장되지 않는다고 밝힘. README 682줄. 책에 없는 지침: 2.1절(범위에 맞는 정수 타입 고르기)(README "책과 달라진 점"에 밝힘) |
| ✅ | `ch2-express-common-behavior` | 예제 5 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | 계획의 예제 4개에 `03_trait_over_concrete`(구체 타입 대신 트레이트로 받기)를 더함. 패닉하는 예제와 외부 크레이트 없음. 책과 달라진 점: 책의 "object safety"는 dyn 호환, 함수 포인터의 `==` 비교에는 `unpredictable_function_pointer_comparisons` 경고가 남(`ptr::fn_addr_eq` 안내). 마커 트레이트 바운드 위반(E0277)은 README 본문에만 인용. README 565줄 |
| ✅ | `ch3-option-result-transforms` | 예제 5 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | 계획의 예제 3종에 `02_option_or_result`(반환 타입 고르기)와 `05_where_to_stop`(`if let`·`match`가 나은 곳)을 더함. 패닉하는 예제와 외부 크레이트 없음. clippy가 잡는 비권장 코드 6곳은 `#[expect(…, reason)]`로 표시(`manual_map`·`collapsible_if`·`question_mark`·`single_match`·`option_map_unit_fn`, rustc `unused_must_use`). 책과 달라진 점: E0507에 `help:`가 붙지만 `?`·`unwrap_or`에는 `clone()`만 제안, 2024 에디션에서는 `collapsible_if`가 let 체인을 제안. `unwrap`/`expect`는 다루지 않고 ch18로 미룸. README 695줄. 책에 없는 지침: 2.5절(`if let`·`match`가 나은 곳)(README "책과 달라진 점"에 밝힘) |
| ✅ | `ch4-idiomatic-errors` | 예제 5 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | dev-dependency `thiserror` 2.0.21·`anyhow` 1.0.104(루트 `Cargo.lock`에 `syn` 3.0.10 등 함께 추가). 일부러 낸 `Err`는 `main` 안에서 처리해 모든 예제가 종료 코드 0. 책과 달라진 점: `Error`는 1.81.0부터 `core::error::Error`로도 쓸 수 있음, 책의 "object safety"는 dyn 호환, `thiserror` 2는 `std` 기능을 끄면 `no_std`에서도 빌드됨(직접 확인). `Display` 없는 `impl Error`와 포괄 `From` 충돌(E0119)은 Part 1 ch8과 겹쳐 링크로 대체. `anyhow`의 `context`는 Part 7 `ch2-implementation-errors`로 미룸. 타입 크기는 64비트 기준. README 692줄 |
| ✅ | `ch5-type-conversions` | 예제 5 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | 계획의 예제 3종을 5개로 나눔(`TryFrom` 대 대체값으로 덮는 `From`, 좁히기와 넓히기를 따로). 패닉하는 예제와 외부 크레이트 없음. `as` 관련 clippy 린트는 pedantic·restriction이라 기본으로 꺼져 있다고 실제대로 설명. 책과 달라진 점: `Infallible`은 `!`의 별칭, 트레이트 객체 업캐스트(1.86)가 강제 변환에 추가됨, `cast_signed`/`cast_unsigned`(1.87) 안정화, E0277 도움말 목록이 다름. `Into`만 구현한 타입에 `T::from`을 부르면 대상에 다른 `From`이 없을 때 E0277이 아니라 E0308. 크기는 64비트 기준. README 635줄. 책에 없는 지침: 지침 3의 `try_from` 권고(README "책과 달라진 점"에 밝힘) |
| ✅ | `ch6-newtype-pattern` | 예제 6 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | 계획의 예제 3종에 ch1이 넘긴 내용(같은 타입 인자 구별, 생성자에서 범위 검증)과 열어 줄 연산 고르기(`Deref`로 통째로 열면 안 되는 이유)를 더함. 패닉하는 예제와 외부 크레이트 없음(`derive_more`는 이름만 소개). 책과 달라진 점: E0117의 `note:`가 늘어남. E0599의 `help:`가 제안하는 `.0.cos()`는 버그를 되살리는 제안이라고 설명. E0117 예제(`Display for Duration`)는 Part 0 ch19와 트레이트가 같고 타입과 동기가 다름. README 661줄. 책에 없는 지침: 2.3절(범위 검증 생성자), 2.4절(열어 줄 연산 고르기)(README "책과 달라진 점"에 밝힘) |
| ✅ | `ch7-builders` | 예제 5 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | dev-dependency `derive_builder` 0.20.2(루트 `Cargo.lock`에 `darling` 0.20.11 계열 함께 추가). 패닉하는 예제 없음, `build()`의 실패는 `Result`로 받아 출력. 책과 동작이 다른 곳은 없음. `&mut` 빌더를 임시값에서 시작한 E0716은 모든 에디션에서 같고, 체인을 블록의 마지막 표현식에 둔 모양만 2024 에디션에서 새로 E0716(직접 확인). E0063은 Part 0 ch5와 겹쳐 링크로 대체. `Command`는 실행하지 않고 설명의 근거로만 사용. 지침 5의 비권장 코드(기본값을 고려하지 않는 검증 함수)는 README에만 실음. README 662줄. 책에 없는 지침: 위치 인자 생성자의 문제, 비공개 필드, `build() -> Result` 검증(README "책과 달라진 점"에 밝힘) |
| ✅ | `ch8-references-pointers` | 예제 6 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-19 | 스마트 포인터 고르기를 지침 둘(소유자 수·스레드 / 내부 가변성·순환)로 나눔. **지침 1(`&Vec<T>` 대신 `&[T]`)과 지침 4(`Borrow`의 비교·해시 약속)는 책의 Item 8에 명시된 내용이 아니라 std 문서와 clippy `ptr_arg`를 근거로 더한 것**(README 2.8절에 밝힘). 패닉하는 예제·외부 크레이트·unsafe 없음(원시 포인터는 주소 비교에만 사용). 주소와 해시 값은 출력하지 않고 비교 결과만 출력 — 예제마다 25번 실행해 출력이 같음을 확인. 크기는 64비트 기준. `HashMap::get`에 `Borrow` 없는 키를 넘기면 지침의 E0277이 아니라 E0308. 책과 달라진 점: 용어 "fat pointer" → 넓은 포인터, `ptr::addr_eq`(1.76) 안정화, `size_of`가 프렐류드에 있음. 연습 3의 `BTreeMap` 결과는 이 툴체인의 구현 결과. README 692줄 |
| ✅ | `ch9-iterator-transforms` | 예제 6 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | 계획의 예제 3종을 6개로 나눔. 패닉하는 예제와 외부 크레이트 없음(`itertools`의 `partition_result`는 이름만 소개). 시간 측정은 하지 않고 ch20으로 미룸. clippy가 잡는 비권장 코드는 `#[expect(…, reason)]`로 표시(`needless_range_loop`·`unnecessary_fold`·`manual_find`·`explicit_counter_loop`). `05`는 `for_each` 안의 `return`이 경고 없이 `continue`처럼 동작해 틀린 결과를 내는 것을 일부러 보임. 책과 달라진 점: 책이 소개하는 `try_find`는 1.100.0에서도 nightly 전용(E0658). 책에 없는 지침: 2.3절(`impl IntoIterator`로 받기), 2.4절의 `partition`·`try_fold` 선택 기준(README "책과 달라진 점"에 밝힘). README 770줄 |
| ✅ | `ch10-standard-traits` | 예제 8 / 에러 4 / 프로젝트 0 | 통과 | 2026-11-19 | 계획의 예제 5종을 8개로 늘림(`Default`, `Debug`/`Display`, 연산자 오버로딩에 예제를 하나씩). 패닉하는 예제와 외부 크레이트 없음. `03`은 출력이 실행마다 같도록 시드를 고정한 해셔를 씀(기본 해셔에서는 어긋난 키의 조회 결과가 실행마다 다름) — `01`·`03`·`04`·`08`을 25번씩 실행해 출력이 같음을 확인. clippy `derived_hash_with_manual_eq`(deny)·`new_without_default`와 rustc `missing_debug_implementations`·`missing_copy_implementations`는 `#[expect]`로 표시. E0204·E0184는 Part 1 ch6과 겹쳐 README 표로만 실음. 책과 실제 동작이 다른 곳은 없음. 책에 없는 지침: 제네릭 타입의 derive 바운드, `total_cmp`와 정수 키, `Default` 판단 전체, 표현이 여럿일 때의 이름 붙은 메서드, `+=`와 `+`의 일치, 고아 규칙과 `Send`/`Sync` 확인(README 2.10절에 근거와 함께 밝힘). README 786줄 |
| ✅ | `ch11-drop-raii` | 예제 4 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | 외부 크레이트와 unsafe 없음. `01`은 패닉 경로를 보이려고 일부러 세 번 패닉하지만 `catch_unwind`로 안에서 붙잡아 종료 코드 0(표준 에러에 실행마다 다른 스레드 번호가 든 패닉 메시지가 나옴). 표준 출력은 예제마다 25번 실행해 같음을 확인. 교착 상태는 일으키지 않고 스코프 스레드의 `try_lock`으로 관찰. `03`의 clone판은 일부러 메모를 두 번 올리는 틀린 결과를 냄. 1.100.0에서 안정화된 `std::mem::DropGuard`를 사용(오래된 툴체인에서는 `01`이 컴파일되지 않음). E0040은 Part 0 ch14와 겹쳐 링크로 대체. 책과 달라진 점: `DropGuard` 안정화, `Drop` 문서에 nightly 전용 `pin_drop`이 함께 보임, E0040의 `help:`가 줄 단위 diff로 바뀜. 책에 없는 지침: `Drop`이 필요 없는 경우와 그 비용, 이름 없이 버린 가드와 `#[must_use]`, `Drop`을 안전망으로 두는 설계, 지침 4(drop 순서) 전체(README 2.6절에 밝힘). 누수는 한 문단만 두고 Part 10 `ch7-obrm`으로 미룸. README 587줄 |
| ✅ | `ch12-generics-vs-trait-objects` | 예제 5 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | 계획의 예제 3종에 `02_thin_generic_shell`(얇은 제네릭 껍데기)과 `03_combined_bounds`(바운드 조합과 업캐스트)를 더함. 패닉하는 예제·외부 크레이트·unsafe 없음. 단형성화는 `type_name`과 디버그 빌드의 `nm -C` 심볼 수로 보임 — 심볼 수는 macOS aarch64·1.100.0·디버그 빌드 기준이고 릴리스 빌드에서는 인라인돼 셀 수 없음(Linux·Windows 미확인). 위임 메시지가 제안한 "제네릭 함수 안의 `static`은 인스턴스마다 따로"는 사실이 아니어서(하나뿐) 쓰지 않고 함정으로 적음. `sized_method_on_dyn` 에러 예제는 에러 코드가 없어 메시지로 확인. 책과 달라진 점: 업캐스트(1.86) 안정화, 책의 "object safety"는 dyn 호환, `impl Trait` 매개변수의 터보피시는 E0107. 책에 없는 지침: 지침 2(얇은 껍데기) 전체, 지침 4의 열거형 선택지, 지침 5의 제네릭 메서드 `where Self: Sized`·`?Sized` 함수(README 2.7절에 밝힘). README 614줄 |
| ✅ | `ch13-default-implementations` | 예제 3 / 에러 2 / 프로젝트 0 | 통과 | 2026-11-19 | 작은 Item이라 예제 3개. 패닉하는 예제·외부 크레이트·unsafe 없음. 일의 양은 시간이 아니라 호출 횟수로 비교. 기본 메서드 추가의 호환성은 한 문단과 에러 예제(E0034)로만 다루고 규칙 전체는 `ch21-semver`로 미룸. E0046은 README 본문과 표에만 인용. 책과 달라진 점: `Iterator`의 기본 메서드는 1.100.0 로컬 문서 기준 75개(책은 50개 이상), 책 각주의 "고유 메서드가 이긴다"는 수신자 모양이 같을 때만 성립(`&self` 트레이트 메서드가 `&mut self` 고유 메서드보다 먼저 잡힘 — 직접 확인). `ExactSizeIterator::is_empty`는 여전히 nightly 전용. 책에 없는 지침: 재정의는 기본 구현과 같은 답을 내야 한다는 약속과 비교 검사, 슈퍼트레이트 대신 메서드에만 바운드를 붙이는 기준, 4.3절의 `Read` 설명(README 2.6절에 밝힘). README 439줄 |
| ✅ | `ch14-lifetimes` | 예제 7 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-19 | 큰 Item. 외부 크레이트·unsafe·패닉하는 예제 없음. 이해가 목적인 소절은 "잘못된 짐작으로 쓴 코드 → 맞는 모형" 틀로 씀. `05`·`06`은 `Box::leak`으로 일부러 메모리를 누수시킴(안전한 코드, 주석과 출력에 밝힘). 스레드 결과는 `main`에서 출력해 실행마다 같음(`01`·`02`·`05`·`06`을 25번씩 확인). 주소는 출력하지 않음. rustc `mismatched_lifetime_syntaxes`는 `#[expect]`로 표시. 책과 달라진 점: 반환 타입 경로에 숨은 라이프타임을 `mismatched_lifetime_syntaxes`(1.89부터 기본 경고)가 잡음, 2024 에디션에서 블록 마지막 표현식의 임시값이 일찍 drop됨, 임시값 수명 연장이 `Some(&temp)`까지 넓어짐, E0106에 소유한 값을 돌려주라는 도움말 추가, `String::leak`(1.72). 책에 없는 지침: `&'static T`와 `T: 'static`의 구분, 2024 에디션의 RPIT 캡처, 뷰 타입과 소유 타입을 고르는 기준(README 2.9절에 밝힘). README 756줄 |
| ✅ | `ch15-borrow-checker` | 예제 8 / 에러 5 / 프로젝트 0 | 통과 | 2026-11-19 | 큰 Item. 외부 크레이트·unsafe·패닉하는 예제 없음(`RefCell`의 대여 실패는 `try_borrow_mut`으로 관찰). 비권장 코드는 "에러를 없애려고 쓴 임시방편"(통째 복제, 사본이 어긋나는 복제판, 삭제 뒤 어긋나는 인덱스, 방이 `Rc`를 쥐어 탈퇴자가 남는 판)으로 구성해 틀린 결과를 출력으로 보임. 책의 "shared reference"는 본편 표기대로 "불변 참조"로 씀. 라이프타임·임시값 바인드는 ch14, `Drop` 가드와 E0509는 ch11 링크로 넘김. `slotmap`·`ouroboros`·`self_cell`·`yoke`는 이름과 버전만 소개(crates.io에서 확인). 책과 달라진 점: E0505·E0507에 복제하라는 `help:`가 붙음, `get_disjoint_mut`(1.86) 안정화, 조건부로 `&mut`를 돌려주는 함수는 1.100.0에서도 E0499(책은 다루지 않음). 책에 없는 지침: 조건부 `&mut` 반환의 해결, `take` 계열 고르는 표, 지침 4(나눠 빌리기) 전체, 세대 ID, `RefCell` 대신 데이터로 돌려주기(README 2.10절에 밝힘). README 888줄 |
| ⬜ | `ch16-avoid-unsafe` | | | | |
| ✅ | `ch17-shared-state-parallelism` | 예제 5 / 에러 2 / 프로젝트 0 | 통과 | 2026-11-19 | 외부 크레이트·unsafe·패닉하는 예제 없음. **`01`·`03`의 비권장 판은 실제로 교착 상태가 됨** — 멈춘 스레드는 `main`이 제한 시간으로 관찰한 뒤 프로세스 종료와 함께 사라지고 예제는 종료 코드 0(`01`은 약 1.4초, `03`은 약 0.6초 걸림). `02`는 일부러 두 표가 어긋나는 틀린 결과를 냄. 예제마다 25번, 동시에 8개씩도 실행해 출력이 같음을 확인. 시점 맞추기는 `Barrier` 대신 제한 시간이 있는 `Condvar` 기다림을 씀(`Barrier`는 잠금 하나 판에서 그 자체로 교착 상태를 만듦). 본편에 있는 `Send`/`Sync` 에러는 표에서 링크만. ThreadSanitizer는 nightly가 필요해 실행하지 못함. `no_deadlocks` 1.3.2·`parking_lot` 0.13.0은 이름과 버전만 소개. 책과 달라진 점: 상태 타입이 `Clone`이면 E0382에 복제하라는 help가 붙음(따르면 공유 상태가 갈라짐), 2024 에디션에서 `if let` 조건식의 가드가 `else` 전에 풀림. 책에 없는 지침: `RwLock`·원자 타입 선택, 잠금을 쥔 채 기다리지 않기, 채널판의 비용, 순위를 검사하는 `RankedMutex`(README 2.6절에 밝힘). README 667줄 |
| ✅ | `ch18-dont-panic` | 예제 4 / 에러 0 / 프로젝트 2 | 통과 | 2026-11-19 | 외부 크레이트와 unsafe 없음. 모든 `examples/`는 종료 코드 0 — `01`·`04`는 일부러 패닉하지만 안에서 붙잡으며 표준 에러에 실행마다 다른 스레드 번호가 든 메시지가 나옴(`02`·`03`은 패닉 훅을 바꿔 표준 에러가 비어 있음). 표준 출력은 25번 실행해 같음. `04`는 `catch_unwind` 뒤 상태가 어긋나는 틀린 결과를 일부러 보임. **`projects/panic_strategy`는 일부러 중단됨**: 기본 실행은 종료 코드 0, `cargo run --profile abort`·`-- extern-c`·`-- drop-panic`은 종료 상태 134(macOS zsh 기준, 메인 세션이 다시 실행해 확인). `projects/panic_free_lib`는 clippy restriction 린트를 `[lints.clippy]`와 `clippy.toml`로 켠 라이브러리(단위 테스트 3 + 문서 테스트 1). `compile_fail/`은 후보가 모두 본편과 겹쳐 만들지 않고 4절을 실행 중 패닉 표로 구성. `no-panic` 0.1.37은 이름만 소개(실행하지 않음). 책과 달라진 점: `extern "C"` 함수 밖으로 나가는 패닉은 1.81부터 중단됨, 책이 패닉판의 예로 든 `from_utf8_unchecked`는 패닉이 아니라 미정의 동작(unsafe fn)이라 다른 예로 바꿈, `[lints]`·`#[expect]`·`strict_*` 등 책 이후의 도구. 책에 없는 지침: `#[track_caller]`와 `expect` 메시지 관례, 책이 들지 않은 숨은 패닉 지점, "실패할 수 있는 일을 먼저", clippy restriction 린트 설정(README 2.7절에 밝힘). README 746줄 |
| ✅ | `ch19-avoid-reflection` | 예제 5 / 에러 3 / 프로젝트 0 | 통과 | 2026-11-19 | 외부 크레이트·unsafe·패닉하는 예제 없음. 비권장 판(`type_name` 문자열 분기, 다운캐스트 사슬)은 새 타입을 더했을 때 틀린 합계를 내는 것을 출력으로 보임. `type_name` 출력은 이 툴체인의 결과이며 문서가 보장하지 않는다고 밝힘. `TypeId` 값과 주소는 출력하지 않음. clippy `type_id_on_box`는 `#[expect]`로 표시. 책과 달라진 점: 업캐스트는 1.86.0에서 안정화(책은 1.76 예정), `trait T: Any`의 `&dyn T`를 `&dyn Any`로 업캐스트해 다운캐스트할 수 있어 `as_any` 우회가 필요 없음, `type_name_of_val`(1.76) 안정화, `TypeId::of`가 `const fn`(1.91), `try_as_dyn` 등은 nightly 전용. 책에 없는 지침: 열거형판, 선택적 메서드와 기능 조회 메서드, `Any`가 정당한 경우 목록, `as_any` 대신 업캐스트, 접근 함수 표(README 2.7절에 밝힘). README 625줄 |
| ✅ | `ch20-avoid-over-optimizing` | 예제 4 / 에러 2 / 프로젝트 0 | 통과 | 2026-11-19 | 외부 크레이트·unsafe·패닉하는 예제 없음. 복사는 시간이 아니라 횟수와 바이트로 먼저 셈. **`04`의 경과 시간 세 줄(소유한 판·빌린 판·경계판)은 실행마다 다름** — README에는 이 환경에서 디버그·릴리스로 한 번씩 실행한 결과를 싣고 그렇게 밝힘(메인 세션이 다시 재도 같은 범위였고, 나머지 줄은 8번 실행해 같음). 차이가 작은 빌린 판과 경계판은 실행마다 순서가 바뀜. 반복문이 통째로 지워지는 함정은 이 챕터의 코드에서 재현되지 않아 본편의 관찰로 연결. 앞 챕터가 미룬 측정(ch9 체인 대 반복문, ch12 제네릭 대 `dyn`)은 연습 문제로 처리. 스레드로 넘기기·함수에서 돌려주기 에러는 ch14와 겹쳐 README 표로만 실음. 책과 달라진 점: `Rc::unwrap_or_clone`(1.76) 안정화, 컬렉션에 참조를 모으다 난 E0597에 note와 help가 붙음, 분리 캡처 때문에 빌리지 않은 필드만 쓰는 클로저는 2021 에디션부터 스레드로 넘길 수 있음. 책에 없는 지침: 판단의 순서 표와 세기 단계, 타입별 `clone()` 비용 표, 측정 방법과 경계판(README 2.6절에 밝힘). README 549줄 |

- 산출물 합계(ch1~ch15): 예제 84 / 에러 50 / 프로젝트 0(ch1~ch10이 57 / 32, ch11~ch15가 27 / 18). 외부 크레이트는 ch4의 `thiserror`·`anyhow`, ch7의 `derive_builder`뿐입니다(dev-dependency).
- ch1~ch10 확인(2026-11-19, rustc 1.100.0): 챕터를 쓴 에이전트의 검증과 별도로 메인 세션이 챕터마다 `cargo fmt --check`, `cargo build --examples`, `cargo clippy --examples -- -D warnings`, 예제 실행(57개 모두 종료 코드 0, 출력이 README와 일치), `compile_fail` 32개의 에러 코드, README 상대 링크를 다시 확인했습니다. 저장소 루트에서 `cargo fmt --all --check`, `cargo build --workspace --examples`, `cargo clippy --workspace --examples -- -D warnings`가 통과했고, `web/`의 사이트 빌드도 깨진 링크 경고 없이 통과했습니다. 모든 검증은 macOS(aarch64)에서만 했습니다.
- ch11~ch15 확인(2026-11-19, rustc 1.100.0): ch1~ch10과 같은 방법으로 챕터마다 다시 확인했습니다(예제 27개 모두 종료 코드 0이고 출력이 README와 일치, `compile_fail` 18개의 에러 코드, README 상대 링크). 실행마다 달라질 수 있는 예제는 25번씩 실행해 표준 출력이 같음을 확인했습니다. 저장소 루트의 `cargo fmt --all --check`, `cargo build --workspace --examples`, `cargo clippy --workspace --examples -- -D warnings`와 `web/`의 사이트 빌드가 깨진 링크 경고 없이 통과했습니다. 이 범위부터는 위임하기 전에 책의 해당 Item이 권하는 내용을 먼저 확인해 위임 메시지에 적고, 더한 판단은 "책에 없는 지침"으로 표시하게 했습니다(ch11~ch15 모두 해당 항목이 있음).
- 책과의 대조(2026-11-19): 챕터마다 책의 해당 Item 페이지에서 코드에 쓰인 타입·함수 이름을 뽑아 챕터에 같은 이름이 없는지 확인했고, "책과 달라진 점"에 적힌 현재 툴체인 쪽 동작은 메인 세션이 다시 실행해 확인했습니다. 책에 없는 지침은 챕터 README의 "책과 달라진 점"에 "책에 없는 지침"으로 밝혔습니다(ch2·ch4는 해당 없음). 이 대조는 책 페이지를 요약 모델로 읽은 결과입니다.
- 사전 작업(2026-11-19): 루트 `Cargo.toml`의 `members`에 `"appd-*/ch*"` 추가, 사이트 스크립트(`web/scripts/sync-content.mjs`)와 배포 워크플로가 `appd-*` 디렉터리를 읽도록 수정, 로드맵에 부록 A 절 추가.

## 작업 방식 메모

- 챕터는 `tutor` 서브 에이전트(`.claude/agents/tutor.md`)가 하나씩 작성하고, 메인 세션이 보고를 받아 이 문서·로드맵·파트 README를 갱신합니다.
- 워크스페이스 `members = ["part*/ch*", "appd-*/ch*"]` 때문에 `Cargo.toml`이 없거나 빌드 타깃이 하나도 없는 챕터 디렉터리가 있으면 **모든 챕터의 cargo 명령이 실패**합니다. 여러 챕터를 동시에 작성할 때는 메인 세션이 `Cargo.toml`과 자리표시 예제(`examples/00_placeholder.rs`)를 갖춘 디렉터리를 먼저 만들어 둡니다. 에이전트마다 `CARGO_TARGET_DIR`을 따로 줘서 빌드 디렉터리도 나눕니다.
- 챕터 README와 소스 파일은 `web/scripts/sync-content.mjs`가 사이트 페이지로 변환합니다. 상대 링크가 가리키는 파일이 없으면 `[sync-content] … 링크 대상이 없습니다` 경고가 나오므로, 챕터를 추가한 뒤 `web/`에서 `pnpm build`로 확인합니다.
- 저장소 루트나 파트 디렉터리에서 `cargo new`를 실행하면 Cargo가 루트 `Cargo.toml`의 `members`에 새 경로를 조용히 추가합니다. `projects/` 아래 독립 프로젝트는 저장소 밖 임시 디렉터리에서 만든 뒤 복사하고 빈 `[workspace]` 테이블을 추가합니다.
- 예제 작성 시 clippy 주의: 서식 지정 없는 문자열 리터럴 인자는 `clippy::print_literal`, `3.141592` 같은 원주율 근삿값은 `clippy::approx_constant`(기본 deny)에 걸립니다. `x: x`처럼 쓴 필드는 `clippy::redundant_field_names`에 걸립니다.
- `derive(Debug)`로 출력하기만 한 필드는 읽은 것으로 치지 않아 `dead_code` 경고가 납니다. 예제에서 필드를 직접 읽어야 합니다.
- 그 밖에 clippy가 기본으로 잡는 것: `manual_map`, `single_match`(갈래 하나가 `_ => ()`인 `match`), `question_mark`(`let Some(..) = .. else { return None; }`), `useless_vec`, `vec_init_then_push`(`let` 문에 붙인 `#[expect]`로는 막을 수 없음), `for_kv_map`, `needless_range_loop`, `get_first`, `manual_range_contains`, `unnecessary_literal_unwrap`, `explicit_auto_deref`, `unnecessary_sort_by`, `unnecessary_lazy_evaluations`, `while_let_loop`, `while_let_on_iterator`, `manual_async_fn`, `new_ret_no_self`, `redundant_pattern_matching`, `manual_range_patterns`, `deref_addrof`(`*(&raw const X)` 모양). `question_mark`는 `match x { Ok(v) => v, Err(e) => return Err(e) }` 모양에도 걸립니다.
- 매크로가 펼친 코드의 lint는 매크로를 부른 곳에 `#[expect]`를 붙여야 막힙니다. rustfmt는 `|x| { x + 1 }`을 `|x| x + 1`로 바꿉니다.
- 다른 챕터를 가리킬 때는 "8장"이 아니라 `ch8` 형식으로 씁니다. "N장"은 The Book의 장을 가리킬 때만 씁니다(저장소 챕터 번호와 Book 장 번호는 ch14부터 다릅니다).
- `ch8-collections`의 `examples/07_string_utf8.rs` 출력 주석 한 줄과 README의 07 출력 블록 한 줄에는 일부러 조합형(NFD) 한글 바이트가 들어 있습니다. 편집 도구가 완성형(NFC)으로 바꿔 저장할 수 있으므로 이 두 파일은 `perl` 등으로 바이트를 보존하며 고칩니다.
- 여러 에이전트가 같은 임시 디렉터리를 쓰므로, 챕터마다 전용 하위 디렉터리를 정해 줍니다.
- 예제 안의 `#[cfg(test)]` 코드는 `cargo clippy --examples`로 검사되지 않습니다. `cargo clippy -p <패키지> --examples -- --test -D warnings`를 쓰되, 인자만 바꾸면 다시 검사하지 않으므로 먼저 소스를 `touch`합니다.
- `ch11-testing` 예제에는 일부러 실패하는 테스트가 `#[ignore]`로 들어 있습니다. 워크스페이스 전체에 `cargo test -- --include-ignored`를 돌리면 실패하는 것이 정상입니다.
- `println!` 호출 문에 붙인 `#[expect]`는 무시되고 `unused_attributes` 경고가 납니다. `let s = format!(..)` 같은 `let` 문에 붙입니다. 서식 관련으로 기본 켜진 lint: `clippy::unused_format_specs`(`format_args!` 결과에 너비 지정), rustc `invalid_from_utf8`(잘못된 UTF-8 리터럴을 `from_utf8`에 넘김).
- `Option`/`Result` 관련으로 clippy가 기본으로 잡는 것: `map_flatten`(`.map(..).flatten()`을 이어 쓸 때만), `manual_ok_err`, `option_map_unit_fn`, `unnecessary_unwrap`, `map_or(false, ..)` 모양(→ `is_some_and`). `manual_map`은 `Option`의 `match`에만 걸리고 `Result`에는 걸리지 않습니다. `unwrap_or(함수호출())`은 경고가 없습니다(`or_fun_call`은 기본으로 꺼져 있음). rustc의 `for_loops_over_fallibles`는 `for x in 옵션값` 모양에 경고합니다.
- 온라인 std 문서는 최신 stable 기준이라 저장소 툴체인보다 앞설 수 있습니다. API가 있는지와 안정화 버전은 `rustc --print sysroot` 아래 `share/doc/rust/html`의 로컬 문서로 확인합니다.
- `HashMap` 키에 `Eq`/`Hash`가 없을 때의 에러 코드는 쓰는 모양에 따라 다릅니다. 변수에 타입을 적으면 `insert`에서 E0599, 타입이 `insert` 호출에서 추론되면 E0277이 나옵니다. clippy `map_entry`는 `contains_key` + `insert` + `get_mut` 모양을 잡지 않습니다.
- `f64`의 `floor`/`round`/`sqrt` 등은 1.98.1에서 `std` 쪽 `impl` 블록에만 있습니다(`core`에는 `abs` 등 일부만). 산술 연산이 만든 NaN의 부호는 비결정적이라 NaN이 들어간 `total_cmp` 출력은 예제에 쓰지 않습니다. 프렐류드의 `Debug`는 derive 매크로뿐이라 `use` 없이 `impl Debug for X`를 쓰면 E0404입니다.
- `#[expect(lint)]`는 기본으로 꺼진 lint(예: nursery 그룹의 `clippy::or_fun_call`)도 그 범위에서 켜서 충족시킵니다. `#[expect]`가 통과한다고 해서 기본 설정에서 나는 경고라는 뜻이 아니므로, "clippy가 잡는다"고 쓰기 전에 속성을 지우고 경고가 실제로 나는지 확인합니다. 기본 수준은 `clippy-driver -W help`로 볼 수 있습니다.
- 변환·비교·해시 관련 clippy 기본 수준: `derive_ord_xor_partial_ord`와 `derived_hash_with_manual_eq`는 deny, `non_canonical_partial_ord_impl`·`non_canonical_clone_impl`·`from_over_into`·`unnecessary_fallible_conversions`·`manual_clamp`·`derivable_impls`·`new_without_default`는 warn입니다. `ptr_arg`는 `&Cow<_>` 매개변수에도, `cast_nan_to_int`는 `f64::NAN as i32`에, `cmp_owned`는 `String::from("x") == "x"`에, `useless_vec`는 `sort`만 하는 `vec![..]`에 걸립니다. `question_mark`는 `Err(e) => return Err(From::from(e))` 모양에는 걸리지 않습니다.
- `RefCell` 예제 주의: 한 `println!` 안에 `try_borrow_mut()`과 `borrow()`를 함께 쓰면 임시 가드가 문장 끝까지 살아 있어 패닉합니다. `const`에 `Cell`을 두면 rustc `const_item_interior_mutations` 경고가 납니다. 디버그 빌드에서 `Box::new([0u8; 16 MiB])`는 스택 오버플로로 끝나므로 `vec!`에서 `into_boxed_slice`로 만듭니다.
- 패닉 예제 주의: `panic!("{}", "상수")`처럼 서식 인자가 상수뿐이면 페이로드가 `String`이 아니라 `&str`입니다. 기본 훅의 `note: run with RUST_BACKTRACE=1` 줄은 처음 한 번만 나옵니다. Clippy 1.100.0은 `[(&str, fn() -> i32); 4]` 같은 타입에 `type_complexity`를 냅니다.
- zsh는 `$var`를 단어로 나누지 않습니다. `env $combo cmd`처럼 변수 하나에 환경 변수 할당 여러 개를 담아 넘기면 잘못된 결과가 나오므로 `${=var}`를 씁니다.
- 반복자 관련으로 clippy가 기본으로 잡는 것: `unnecessary_fold`, `filter_next`, `map_flatten`, `iter_cloned_collect`, `while_let_on_iterator`. `impl Iterator`를 돌려받아 `.rev()`를 붙이면 E0599가 아니라 E0277입니다. `type_name`은 `core::iter::adapters::map::Map`처럼 정의된 위치를 출력하므로 문서의 `std::iter::Map`과 경로가 다릅니다.
- 동기화 예제 주의: `Condvar` 하나를 `Mutex` 둘과 쓰면 macOS에서 실행 중 패닉합니다(`attempted to use a condition variable with two mutexes`). 원자 타입의 `fetch_update`는 1.99부터 deprecated라 `try_update`/`update`를 씁니다. 스코프 스레드에 넘기는 클로저가 `&F`를 캡처하면 `F: Sync`가 필요하므로 `move`로 옮깁니다. 스레드의 진행 시점을 맞춰 `try_lock` 관찰을 결정적으로 만들 때는 `Barrier`가 유용합니다.
- 잠금 관련 lint의 기본 수준: clippy `mutex_atomic`·`mutex_integer`는 꺼져 있습니다(restriction). 켜져 있는 것은 rustc `let_underscore_lock`·`invalid_atomic_ordering`(deny), clippy `mut_mutex_lock`·`readonly_write_lock`·`missing_const_for_thread_local`·`declare_interior_mutable_const`(warn)입니다.
- 소켓 예제 주의: clippy `unused_io_amount`는 기본 deny라 `read`/`write`의 반환값을 버리면 에러입니다(`read_zero_byte_vec`는 꺼져 있음). macOS에서는 읽기 제한 시간 초과가 `TimedOut`이 아니라 `WouldBlock`으로 나오므로 둘을 함께 봅니다. 포트는 0으로 `bind`하고 `local_addr()`로 알아냅니다.
- 프로세스 예제 주의: 이식성 있게 자식 프로세스를 보이려면 예제가 `current_exe()`에 "자식 모드" 인자를 붙여 자기 자신을 다시 실행합니다. clippy `zombie_processes`(warn)는 `spawn` 뒤 `wait`하지 않는 코드를 잡지만, 필드를 `unwrap()`으로 옮겨 쓴 경우는 놓칩니다. `Duration * 1.5`는 E0277이 아니라 E0308입니다(`Mul<u32>`뿐이라서). 릴리스 빌드에서는 `black_box`를 써도 반복문이 통째로 최적화될 수 있습니다.
- 입출력 예제 주의: `println!`은 `BrokenPipe`에서 패닉하므로 예제 출력을 `| head`로 자르면 패닉이 보입니다. 입출력 관련 clippy 기본 수준: `unbuffered_bytes`(버퍼 없는 리더의 `bytes()`), `join_absolute_paths`, `write_literal`은 warn입니다. `io::Error::new`/`other`로 감싼 에러는 `Display`로는 안쪽 메시지를 보여 주지만 `source()`는 안쪽 에러를 돌려주지 않습니다. macOS에서는 파이프로 보낸 표준 출력도 줄 단위로 비워지지만 문서가 약속하는 동작은 아닙니다.
- 여러 에이전트가 동시에 빌드할 때 `cargo run -q`가 잠금을 기다리며 아무 메시지 없이 오래 멈춘 것처럼 보인 일이 있었습니다(원인 미확인). 표준 입력을 읽는 예제는 `< /dev/null`이나 파이프를 명시하고 제한 시간을 걸어 실행합니다.
- 비동기 예제 주의: `Send`가 아닌 값을 await 너머로 들고 간 퓨처를 `Send`를 요구하는 곳에 넘기면 E0277이 아니라 에러 코드 없는 `future cannot be sent between threads safely`가 납니다. await 전에 `drop(rc)`를 불러도 같은 에러가 나므로 블록으로 스코프를 끝냅니다. 관련 clippy 기본 수준: `async_yields_async`는 deny, `manual_noop_waker`·`waker_clone_wake`·`let_underscore_future`·`redundant_async_block`·`manual_async_fn`은 warn, `unused_async`·`future_not_send`·`large_futures`는 꺼져 있습니다.
- FFI·`Any` 예제 주의: `c_char`의 부호는 플랫폼마다 다릅니다(macOS aarch64에서는 `i8`). `Wrapping`·`Saturating`·`NonZero`의 디버그 형식은 감싼 모양 없이 안의 값만 출력합니다. `Wrapping<u8> + u8`은 E0277이지만 `+=`는 됩니다. `Box<dyn Any>`에 `type_id()`를 부르면 상자 자체의 타입 ID가 나오고 clippy `type_id_on_box`가 잡습니다. `getenv`로 읽을 결정적인 값이 필요하면 `cargo run`이 넣어 주는 `CARGO_PKG_NAME`을 씁니다.
- 스레드 출력 순서를 고정하려면 스레드 안에서 출력하지 말고 결과를 돌려받아 `main`에서 출력합니다. 공유된 `Mutex`를 여러 스레드에서 디버그 형식으로 출력하면 `"<locked>"`가 섞여 출력이 비결정적이 됩니다.
- 셸 환경에 `RUSTUP_TOOLCHAIN`이 옛 버전으로 고정돼 있으면 `mise.toml`의 설정과 달리 옛 툴체인이 잡힙니다. 작업 전에 `rustc --version`을 확인하고, 다르면 명령마다 `RUSTUP_TOOLCHAIN=<버전>`을 붙입니다.
- Part 2의 에디션 비교 방식: `examples/`는 2024로만 컴파일하고, 다른 에디션으로 컴파일할 코드는 `projects/` 아래 독립 프로젝트에 둡니다. 에디션별 패키지가 `[[bin]] path = "../shared/x.rs"`로 패키지 밖의 같은 소스 파일을 공유해도 build·clippy·fmt가 동작합니다. 타깃별 `edition` 키는 1.100.0에서 deprecated 경고가 납니다. 워크스페이스의 두 패키지에 같은 이름의 bin이 있으면 `output filename collision` 경고가 나므로 이름에 에디션을 붙입니다. 가상 매니페스트에는 `resolver`를 직접 적습니다.
- 여러 에디션으로 컴파일하는 공유 소스에서 에디션에 따라 나기도 하고 안 나기도 하는 경고는 `#[expect]`가 아니라 `#[allow]`로 막습니다(`#[expect]`는 경고가 안 나는 쪽에서 unfulfilled 경고가 납니다).
- Part 2부터 `compile_fail/` 파일은 에디션에 따라 결과가 다를 수 있습니다. 파일 머리말에 에디션별 결과를 적고, 이름 끝의 `_2015`·`_2018`·`_2021`은 그 에디션(과 그 이전)에서만 실패한다는 뜻입니다. 검증할 때는 `rustc --edition`을 바꿔 가며 모두 컴파일합니다.
- 에디션 관련 사실(1.100.0): never 타입 폴백 변경은 1.100부터 모든 에디션에 적용됩니다. `keyword_idents_2018`·`keyword_idents_2024`의 기본 수준은 allow입니다. `rustfmt`를 직접 실행하면 2015로 읽고, `cargo fmt`는 크레이트의 에디션을 넘깁니다. Cargo 프로젝트에서 에디션 호환성 린트 그룹을 켜 보려면 `cargo rustc -q -p <패키지> -- -W rust-2021-compatibility`를 씁니다. 서식 문자열 안에서는 `{r#gen}`이 에러이므로 `{gen}`으로 씁니다.
- Part 2에서 clippy가 잡은 것: `iter_next_slice`(배열에 `.iter().next()`), `into_iter_on_ref`, `never_loop`(기본 deny, 한 번만 도는 `for`), `needless_lifetimes`. 기본 패닉 훅의 표준 에러 출력은 파이프로 받으면 빈 줄로 시작합니다.
- 도구와 에디션(1.100.0): `cargo fmt`는 두 패키지가 함께 쓰는 소스 파일을 멤버 목록에서 먼저 나오는 패키지의 에디션으로 한 번만 서식합니다. 2024 에디션의 문서 테스트는 한 바이너리로 병합되지만, 중첩 `include_str!`를 쓰는 테스트가 있으면 메시지 없이 전부 따로 컴파일되고(`standalone_crate`를 붙이면 나머지가 다시 묶임) `-- --show-output`을 줘도 병합하지 않습니다. rustfmt의 한 줄 `if`/`else`는 한글 문자열의 폭(글자당 2칸) 때문에 폭 제한에 쉽게 걸립니다.
- zsh에서는 `"$e:PASS"`처럼 변수 바로 뒤에 `:P`가 오면 경로 수식자로 해석됩니다. `"${e}:PASS"`로 씁니다.
- `cargo fix` 주의: 추적하지 않는 파일(`Cargo.lock` 포함)이 있어도 작업 트리가 더럽다고 보고 멈춥니다. `cargo fix --edition`은 `--force-warn`으로 린트 그룹을 켜므로 `#[allow]`로 막은 코드도 고치지만, `-W`와 `#![warn(..)]`은 `#[allow]`에 집니다. `Fixed` 줄의 순서는 실행마다 다릅니다. 저장소 안에서는 실행하지 않고 저장소 밖 사본에서만 실행합니다.
- zsh에서 `echo ===`는 `=` 확장 때문에 실패합니다(`echo '==='`로 씁니다). 테스트 출력의 스레드 ID와 테스트가 끝나는 순서는 실행마다 다릅니다.

- 부록(`appd-*`) 주의: 워크스페이스의 `"appd-*/ch*"` 글롭은 맞는 디렉터리가 하나도 없으면 cargo가 `failed to load manifest for workspace member`로 실패합니다. 첫 챕터 디렉터리를 만든 뒤에 `members`에 넣습니다. 부록 챕터의 패키지 이름은 `er-ch{N}-{slug}`입니다.
- 받는 쪽 바운드와 에러 코드(부록 ch2): `Fn` 바운드에 맞지 않는 클로저는 쓰는 위치에 따라 코드가 다릅니다. 변수에 담아 넘기면 E0525, 인자 자리에 바로 쓰면 바꾸는 클로저는 E0596, 옮기는 클로저는 E0507입니다. `?Sized`가 빠진 제네릭에 `&dyn T`를 넘기면 E0277이고, 첫 번째 help(`.as_ref()`를 지우라는 것)를 따르면 E0308이 납니다. 함수 포인터의 `==` 비교는 rustc `unpredictable_function_pointer_comparisons` 경고가 납니다(`std::ptr::fn_addr_eq` 사용).
- 타입 설계 관련 clippy 기본 수준(부록 ch1): `fn_params_excessive_bools`·`struct_excessive_bools`·`cast_possible_truncation`·`cast_sign_loss`는 pedantic이라 꺼져 있습니다. `i64 as u16`은 기본 설정에서 경고가 없습니다. 1.100.0의 `TryFromIntError` 표시 문구는 `number too large to fit in target type` / `number too small to fit in target type`입니다. rustfmt는 한글이 든 긴 `println!`을 여러 줄로 펼치므로 출력용 도우미 함수로 줄입니다.
- `Option`/`Result` 리팩터링 관련(부록 ch3): clippy `question_mark`는 `Err(e) => return Err(e.into())`를 잡지만, 배리언트로 감싸는 갈래·`None => return Err(..)`·`Err(_) => return None`은 잡지 않습니다. 겹친 `if let`은 `collapsible_if`가 let 체인으로 합치라고 제안하고, `match o { Some(v) => v, None => 0 }`은 `manual_unwrap_or_default`가 잡습니다. 수동 `and_then` 모양과 가드가 있는 `match`는 잡지 않습니다. 빌린 `Option`을 옮기는 E0507의 help는 `map`에는 `as_ref()`/`as_mut()`/`clone()`을, `?`와 `unwrap_or`에는 `clone()`만 제안합니다. `?` 앞의 `map_err(|e| e.into())`는 E0283(또는 E0282), 클로저 안의 `continue`는 E0267입니다. 식 문장(`f();`)에 붙인 `#[expect(unused_must_use)]`는 동작합니다.
- 에러 크레이트(부록 ch4, `thiserror` 2.0.21·`anyhow` 1.0.104): `thiserror`는 `Debug`를 만들지 않습니다(직접 derive). `anyhow`의 `{:?}`는 원인이 하나면 번호 없이, 둘 이상이면 `0:`, `1:` 번호를 붙입니다. `thiserror-impl`이 `syn` 3.x를 끌어옵니다. 챕터를 쓴 에이전트는 cargo 1.100.0이 쓰지 않는 의존성에 `unused_dependencies` 경고를 냈다고 보고했습니다(메인 세션은 확인하지 않음).
- 변환 관련(부록 ch5): `as` 관련 clippy 린트(`cast_possible_truncation`·`cast_sign_loss`·`cast_possible_wrap`·`cast_lossless`)는 pedantic, `as_conversions`는 restriction이라 기본으로 꺼져 있고, `cast_nan_to_int`·`cast_abs_to_unsigned`·`unnecessary_fallible_conversions`·`from_over_into`는 경고합니다. `impl Into<u64>` 매개변수에 정수 리터럴을 넘기면 리터럴이 `i32`로 정해져 E0277입니다. `Into`만 구현한 타입에 `T::from(x)`를 부르면 대상 타입에 다른 `From` 구현이 없을 때는 E0308, 있으면 E0277입니다. `From`은 const 트레이트가 아니라 `const`에서 `u64::from`을 부르면 E0658입니다. 정수의 `truncate`·`widen`·`saturating_cast`는 1.100.0에서 nightly 전용(E0658)이고 `cast_signed`/`cast_unsigned`(1.87)는 안정화됐습니다.
- 뉴타입 관련(부록 ch6): 비공개 필드를 가진 튜플 구조체를 모듈 밖에서 만들거나 구조 분해하면 E0603(이름 붙은 필드는 E0451)입니다. 뉴타입에 안쪽 타입의 메서드를 부른 E0599의 help는 `.0.method()`를 제안합니다. `Add` 구현이 다른 타입에 대해서만 있으면 E0369가 아니라 E0308이고, 아예 없으면 E0369입니다. `iter().sum::<NewType>()`은 E0277입니다. `write!(f, "{}", self.0)`로 `Display`를 위임하면 너비 같은 서식 지정이 사라지므로 `fmt::Display::fmt(&self.0, f)`로 위임합니다.
- 빌더 관련(부록 ch7, `derive_builder` 0.20.2): clippy `too_many_arguments`는 인자 8개부터 warn입니다. `&mut` 빌더를 임시값에서 시작해 변수에 담는 E0716은 모든 에디션에서 같고, 체인을 블록의 마지막 표현식에 둔 모양(`{ B::new().x() }.build()`)만 2024 에디션에서 새로 E0716입니다. 소비형 빌더의 E0382는 세터 인자에 `format!`이 있으면 `Clone`을 구현하라는(따르면 안 되는) 제안으로 바뀝니다. `derive_builder`의 기본은 `&mut` 패턴이고, 검증 함수에서는 기본값에 맡긴 필드가 `None`입니다. `type_name_of_val`에 `&mut T`를 그대로 넘기면 `&T`로 강제 변환돼 `&mut`이 보이지 않습니다. 필드 기본값 문법(`struct S { t: u32 = 30 }`)은 1.100.0에서 E0658입니다.
- 포인터 관련(부록 ch8): clippy `ptr_arg`는 본문이 `capacity()`나 `push`를 쓰는 `&Vec`/`&mut Vec` 매개변수와 `&Rc<T>`에는 경고하지 않습니다. `vec_box`·`box_collection`·`redundant_allocation`은 `let` 문의 타입 표기에는 경고하지 않고 필드와 시그니처에만 경고합니다. `size_of_val(&참조변수)`는 `size_of_ref`에 걸립니다. `HashMap<K, _>::get("문자열")`은 `K: Borrow<str>`가 없으면 E0277이 아니라 E0308입니다. 넓은 원시 포인터의 `==`는 rustc `ambiguous_wide_pointer_comparisons` 경고가 납니다(`ptr::addr_eq` 사용). `Eq`와 `Hash`가 어긋난 키의 조회 결과는 기본 해셔에서 실행마다 다르므로, 예제 출력에 쓰려면 `BuildHasherDefault<DefaultHasher>`로 시드를 고정합니다. rust-src가 없어도 표준 라이브러리 소스는 로컬 문서의 `share/doc/rust/html/src/`에서 볼 수 있습니다.
- 반복 관련(부록 ch9): clippy 기본 warn은 `needless_range_loop`·`unnecessary_fold`·`manual_find`·`explicit_counter_loop`·`manual_try_fold`·`map_flatten`이고, `needless_for_each`·`needless_collect`·`explicit_iter_loop`는 꺼져 있습니다. `needless_range_loop`는 인덱스로 이웃 원소나 다른 배열의 다른 위치를 읽으면 경고하지 않습니다. `for_each` 안의 `return`은 경고 없이 `continue`처럼 동작하고, `break`/`continue`는 E0267입니다. `try_find`·`try_collect`·`map_windows`는 1.100.0에서 nightly 전용(E0658)입니다.
- 표준 트레이트 관련(부록 ch10): 예제 바이너리라도 `pub mod` 안의 `pub` 타입에는 rustc `missing_debug_implementations`·`missing_copy_implementations`(기본 allow)와 clippy `new_without_default`가 걸립니다. `#[default]` 없는 열거형의 `derive(Default)`는 E0665, `derive(Eq)`를 `f64` 필드 둘에 붙이면 E0277이 첫 필드에서만 나고 `Hash`는 필드마다 납니다. clippy `eq_op`(deny)는 `x == x`를 잡습니다. `should_implement_trait`·`inherent_to_string`·`suspicious_arithmetic_impl`·`suspicious_op_assign_impl`은 warn, `derive_partial_eq_without_eq`는 allow입니다.
- 제네릭·트레이트 객체 관련(부록 ch12): **제네릭 함수 안의 `static`은 타입 인자마다 따로가 아니라 하나뿐입니다**(단형성화를 보이는 용도로 쓸 수 없음). 단형성화는 `type_name`과 디버그 빌드의 `nm -C <실행 파일> | grep <함수 이름>`으로 봅니다(1.100.0은 v0 맹글링, 릴리스 빌드는 인라인돼 셀 수 없음). 제네릭 함수 안의 클로저는 바깥 타입 매개변수마다 다른 타입이 되어 그 클로저를 받는 `filter_map`·`collect` 등도 다시 단형성화됩니다. `dyn A + B`는 E0225(`dyn A + Send + Sync`는 됨), `trait X: Clone`을 `dyn`으로 쓰면 E0038, `where Self: Sized` 메서드를 트레이트 객체로 부르면 에러 코드 없는 에러, `impl Trait` 매개변수에 터보피시를 쓰면 E0107, 값으로 받는 `self` 메서드를 `Box<dyn T>`에서 부르면 E0161입니다.
- 기본 구현 관련(부록 ch13): 1.100.0의 `io::Read`는 `read`와 nightly 전용 `read_buf`가 서로를 기본 구현으로 부르고, `impl Read for X {}`는 `E0046 missing one of: read_buf, read`입니다. 사용자 트레이트에서 서로 부르는 두 기본 메서드를 둘 다 구현하지 않아도 rustc와 clippy는 경고하지 않습니다(실행하면 무한 재귀). 메서드 탐색에서 `&self` 트레이트 메서드는 `&mut self` 고유 메서드보다 먼저 잡힙니다. `ExactSizeIterator::is_empty`를 재정의하면 E0658입니다. clippy `manual_range_contains`는 리터럴 경계에만 경고합니다.
- `Drop`·가드 관련(부록 ch11): `std::mem::DropGuard`가 1.100.0에서 안정화됐습니다(`new`·`dismiss`, `new`에 `#[must_use]`). `impl Drop for X {}`는 E0046, `Drop` 타입에서 필드를 옮겨 꺼내면 E0509(help는 `&self.field`와 `.clone()`을 제안), `Drop`을 구현한 가드는 스코프 끝까지 대여를 붙잡아 E0502가 납니다. rustc 기본 수준: `let_underscore_lock`은 deny(`unwrap` 없는 `let _ = m.lock();`도 걸림), `let_underscore_drop`은 allow, `dropping_references`는 warn. clippy `significant_drop_tightening`·`significant_drop_in_scrutinee`·`mem_forget`은 allow, `drop_non_drop`은 warn. 가드를 `Option`/`Result`에 감싸 돌려주면 타입의 `#[must_use]`가 경고를 내지 않으므로 메서드에 붙입니다. `unused_must_use`의 help가 제안하는 `let _ = …`은 가드에서는 바로 버려지는 함정입니다. 기본 패닉 훅 메시지에는 실행마다 다른 스레드 번호가 나옵니다.
- 라이프타임 관련(부록 ch14): rustc `mismatched_lifetime_syntaxes`(1.89부터 기본 warn)는 반환 타입 경로에 숨은 라이프타임(`fn f(s: &str) -> View`)을 잡고 `'_`를 제안합니다. 입력에만 숨은 경우는 `elided_lifetimes_in_paths`(allow)만 잡습니다. clippy `needless_lifetimes`(warn)는 `fn f<'a>(s: &'a str) -> &'a str`만 잡고, 구조체를 돌려주는 모양은 pedantic `elidable_lifetime_names`만 잡습니다. 임시값 수명 연장은 `Some(&temp)` 같은 튜플 구조체 생성자 안까지 됩니다(1.89). `const fn` 호출 결과는 상수 승격되지 않습니다(`&u32::max(1, 2)`를 `&'static`으로 돌려주면 E0515). `&str` 매개변수를 `thread::spawn`에 넘기면 E0521, 입력 참조 없는 함수의 E0106은 소유한 값을 돌려주라는 help가 붙지만 그대로 따르면 E0308입니다. E0505의 help(`&x.clone()`)를 따르면 E0716이 날 수 있습니다. 2024 에디션에서 `fn f(&self) -> impl Iterator<Item = &'a str>`는 `&self`까지 캡처하므로 `+ use<'a>`가 필요할 수 있습니다.
- 대여 검사기 관련(부록 ch15): 조건부로 `&mut`를 돌려주는 함수(찾으면 돌려주고 없으면 넣기)는 1.100.0에서도 E0499입니다(인덱스나 entry API로 풂). 대여가 살아 있는 동안 소유자가 덮어쓰면 E0506, 읽으면 E0503, 옮기면 E0505입니다. E0505·E0507에는 복제하라는 help가 붙고(`Clone`이 없으면 note로), E0507의 "빌리라"는 제안을 따르면 E0308이 날 수 있습니다. 고치는 메서드의 인자 안에서 빌린 값을 마지막으로 쓰는 코드는 컴파일됩니다(2단계 대여). `for` 안에서 `&mut self` 메서드를 부른 E0499에는 인덱스 반복문을 쓰라는 도움말이 붙습니다. clippy `redundant_clone`은 allow(nursery), `mem_replace_option_with_some`·`mem_replace_option_with_none`·`mem_replace_with_default`는 warn입니다. 복제본을 돌며 고치는 `for mut x in self.v.clone()`은 clippy가 잡지 않습니다. `get_disjoint_mut`은 1.86에서 안정화됐습니다.
- 한 에이전트가 예제를 `touch`한 뒤 `cargo build`/`clippy`가 너무 빨리 끝나 재검사 여부가 불분명했다고 보고했습니다(원인 미확인). 의심스러우면 `cargo clean -p <패키지>` 뒤에 다시 확인합니다.
- 교착 상태 예제(부록 ch17): 같은 스레드가 `Mutex::lock`을 다시 부르면 멈춥니다(문서는 "패닉하거나 교착 상태"라고만 정함). 멈출 수 있는 스레드는 스코프 스레드로 만들면 안 되고(scope가 영원히 `join`을 기다림) `spawn` + `Arc`로 만들어 `main`이 `recv_timeout`으로 관찰합니다. 멈춘 스레드는 프로세스 종료와 함께 사라집니다. `Barrier`는 잠금 하나 판에서 그 자체로 교착 상태를 만들므로, 여러 판에 같은 관찰 장치를 쓰려면 제한 시간이 있는 `Condvar` 기다림을 씁니다. 2024 에디션에서는 `if let … = m.lock()…`의 가드가 `else` 전에 풀리지만 `match`에서는 끝까지 삽니다. `MutexGuard`를 거쳐 두 필드를 나눠 빌리면 E0499이므로 `let l = &mut *guard;`로 한 번 역참조합니다. `Mutex::set`/`replace`/`get_cloned`, `ReentrantLock`은 nightly 전용입니다. clippy `significant_drop_tightening`은 nursery라 꺼져 있고 켜면 잡음이 많습니다.
- 패닉 예제(부록 ch18): `extern "C"` 함수 밖으로 나가는 패닉은 1.81부터 중단되고(`panic in a function that cannot unwind`), 이 중단과 이중 패닉의 중단은 `RUST_BACKTRACE=0`이어도 백트레이스를 출력합니다. `panic = "abort"`인 프로그램의 종료 상태는 macOS에서 134입니다. 사용자 정의 프로필(`[profile.abort] inherits = "dev"`, `cargo run --profile abort`)은 독립 프로젝트에서만 동작합니다. `Index::index`는 트레이트 선언에 `#[track_caller]`가 있어 직접 만든 구현도 패닉 위치가 부른 줄이 됩니다. clippy restriction 린트: `indexing_slicing`은 상수 인덱스와 `str` 슬라이싱을 잡지 않고(`string_slice`가 잡음), `arithmetic_side_effects`는 0이 아닌 상수로 나누는 연산에는 경고하지 않습니다. `clippy.toml`의 `allow-unwrap-in-tests` 등으로 테스트 코드를 허용할 수 있습니다. `String::from_utf8_unchecked`는 패닉판이 아니라 `unsafe fn`입니다.
- `Any` 관련(부록 ch19): `downcast_ref`/`is`는 `dyn Any`의 고유 메서드라 `trait T: Any`여도 `&dyn T`에서 바로 부르면 E0599입니다. 먼저 `&dyn Any`로 업캐스트합니다(1.86부터 가능, `as_any` 우회가 필요 없음). 제네릭 함수 안의 `TypeId::of::<T>()`는 E0310, 관계없는 트레이트 객체로의 `as` 캐스트는 E0605입니다. `type_name_of_val`은 1.76, `TypeId::of`의 `const fn`은 1.91부터이고 `try_as_dyn`은 nightly 전용입니다. else-if 다운캐스트 사슬의 마지막 갈래가 `Some(..)`/`None`이면 clippy `manual_map`에 걸립니다.
- 측정 예제(부록 ch20): 시간 측정은 다른 에이전트가 동시에 빌드하면 크게 흔들리고, 차이가 작은 두 판은 실행마다 순서가 바뀝니다. 출력에 "어느 쪽이 빠르다"를 단정하는 줄을 두지 않고, 실행마다 달라지는 줄을 시간 줄로만 한정합니다(검증할 때는 그 줄을 빼고 비교). clippy `clone_on_ref_ptr`(restriction)·`redundant_clone`(nursery)은 꺼져 있습니다. 빌린 값을 스레드로 넘기는 에러를 재현할 때 클로저가 빌리지 않은 필드 하나만 쓰면 분리 캡처 때문에 2021 에디션부터 컴파일됩니다. 빌린 값을 구조체 필드에 보관하다 난 E0597에는 도움말이 없고 컬렉션에 모으면 note와 help가 붙습니다. `Rc::unwrap_or_clone`은 1.76부터입니다.
- rustup 1.29는 설치되지 않은 툴체인을 이름으로 가리키는 명령을 실행하면 그 툴체인을 자동으로 내려받습니다. 조회처럼 보이는 `rustup component list --toolchain nightly`도 nightly(약 1.4GB)를 설치합니다(2026-11-19에 실제로 일어나 제거함). 설치 여부는 `rustup toolchain list`로만 확인하고, `--toolchain`·`rustup run`·`cargo +<이름>`은 목록에 있는 툴체인에만 씁니다.
- 이 환경의 안전 검사는 `rm -rf "$변수/…"` 꼴의 삭제를 막습니다. 리터럴 경로를 적어 지웁니다.

## 후속 조치

챕터 작성 중 발견한, 다른 파일에 반영해야 할 사항을 적습니다.

- ch2: 이 워크스페이스에서 `cargo doc -p rand`는 cargo 1.98.1 내부 패닉으로 끝납니다(`rand`가 dev-dependency뿐이라 생기는 문제로 보임). README에는 대안 `cargo doc -p p0-ch2-guessing-game --examples --open`을 안내했습니다. Part 3 `ch2-dependencies` 작성 때 다시 확인합니다.
- ch9: 워크스페이스 멤버 패키지의 `[profile]`은 무시되고 경고가 납니다(`profiles for the non root package will be ignored`). 프로필은 루트 `Cargo.toml`에만 둘 수 있습니다. Part 3 `ch6-profiles-build-performance` 작성 때 반영합니다.
- ch11: `cargo test 이름`이 문서 테스트를 건너뛰는 동작은 Part 3 `ch9-testing-ci`에서 다시 다룹니다.
- ch19: unsafe 예제는 Miri로 확인하지 못했습니다. Part 10을 시작해 nightly와 Miri를 준비하면 `ch19-advanced-features`의 `01`, `02` 예제를 `cargo +nightly miri run`으로 다시 확인합니다.
- Part 0 ch19: README 2.14절의 "`!`를 타입으로 직접 적을 수 있는 곳은 함수의 반환 타입뿐이고 `let x: ! = panic!();`은 E0658"이라는 설명이 1.100.0에서는 맞지 않습니다. never 타입이 안정화되어 `let x: ! = …`과 `Vec<!>`가 컴파일됩니다(1.98.1에서는 E0658). 해당 문단을 고칩니다. Part 2 ch4 2.10절이 1.100.0 기준의 never 타입과 폴백을 보충했습니다.
- Part 1 ch14: Part 10에서 nightly와 Miri를 준비하면 `examples/04_calling_c.rs`의 unsafe 블록(29·35·41·76·79·103행)을 `cargo +nightly miri run`으로 확인합니다. Miri가 `strlen`·`getenv` 같은 외부 C 함수를 지원하는지도 그때 함께 확인합니다.
- Part 10 `ch12-ffi-beneath-std`에 넘길 것: `CString::into_raw`/`from_raw`, `CStr::from_ptr`이 돌려주는 참조의 라이프타임이 아무것에도 묶여 있지 않다는 점, `Option<NonZero<T>>`의 FFI 호환 보장.
- Part 0 ch17: README 2.8절의 "`Box<dyn Error>`는 안에 무엇이 들었는지 타입으로 알 수 없다"는 설명은 다운캐스트로 실행 중에 물어볼 수 있다는 점에서 조금 부정확합니다. Part 1 ch8 2.5절이 보충했고, ch17 원문을 고칠지는 정합니다.
- Part 10 `ch2-data-layout`에 넘길 것: `Packet { u8, u32, u8 }`은 기본 표현에서 8바이트, `#[repr(C)]`에서 12바이트이고, `Option<Cell<bool>>`은 2바이트입니다(1.100.0에서 확인).
- Part 1 ch1: README가 "툴체인은 1.98.1, 온라인 문서는 1.100.0"인 상황을 문서 버전이 어긋나는 예로 들고 `bool::toggle`이 E0599라고 설명합니다. 툴체인을 1.100.0으로 올린 뒤에는 `toggle`이 컴파일되므로, 이 예와 로컬 문서 버전 표기(`std 1.98.1`)를 현재 툴체인에 맞게 고칩니다.
- 전체: 챕터 README 머리말의 "확인한 툴체인: rustc 1.98.1"은 작성 당시 기준입니다. 1.100.0에서 예제 출력과 에러 메시지까지 다시 확인한 챕터부터 표기를 바꿉니다.
- Part 1 ch12: `03_split_read_write`는 교착 상태를 확실히 보이려고 64 MiB를 주고받아 메모리를 수백 MiB 씁니다. 부담스러우면 크기를 줄일지 정합니다.
- Part 0 ch8: README 2.5절과 `05_vec_drop` 예제 설명이 "벡터가 drop되면 요소가 0번부터 차례로 drop된다"고 단정합니다. `Vec` 문서의 Guarantees 절은 요소의 drop 순서를 보장하지 않는다고 적고 있으므로(현재 구현은 앞에서부터), "현재 구현의 동작이며 보장되지 않음"이라는 단서를 붙일지 정합니다.
- Part 1 ch2·ch4: README가 목표 분량을 넘었습니다(1077줄, 1060줄). 예제 출력과 에러 인용이 대부분이며, 줄일지는 읽어 보고 정합니다.
- Part 1 전체: 여러 챕터를 동시에 작성해서, 같은 묶음의 챕터(ch1~ch4끼리, ch5~ch8끼리, ch9~ch14 일부)와 뒤 챕터는 링크 없이 코드 서식의 이름으로만 가리키는 곳이 있습니다. 상대 링크로 바꾸는 것은 선택 사항입니다.
- Part 0 전체: 챕터를 동시에 작성했기 때문에, README가 그때 아직 없던 챕터를 링크 없이 코드 서식의 이름(`ch10-generics-traits-lifetimes`)으로만 가리키는 곳이 있습니다. 내용에는 문제가 없고, 상대 링크로 바꾸는 것은 선택 사항입니다.
- Part 0 전체: 예제 수(챕터당 4~13개)와 README 분량(600~1280줄)이 `tutor` 문서의 기준(보통 3~6개)보다 많은 챕터가 있습니다. 줄일지는 읽어 보고 정합니다.
- Part 2에서 반영한 약속: let 체인의 `while let … pop() && …` 함정(Part 0 ch18), `if let` 임시값 범위의 2021/2024 차이(Part 1 ch7), `extern { … }` → `unsafe extern`(Part 1 ch14)은 Part 2 ch4가 다뤘습니다. `set_var`의 unsafe화(Part 0 ch12, Part 1 ch10·ch14)와 문서 테스트 병합(Part 0 ch11)은 Part 2 ch5가 다뤘습니다. Part 0 ch18 README 97행과 Part 1 ch7 README 431행은 Part 2 `ch4-rust-2024-language`를 링크 없이 이름으로만 가리키므로, 상대 링크로 바꿀 수 있습니다(선택 사항).
- Part 2 ch4: Part 10에서 nightly와 Miri를 준비하면 `examples/05_unsafe_changes.rs`와 `projects/edition_compare/shared/new_syntax.rs`의 unsafe 블록을 `cargo +nightly miri run`으로 확인합니다.
- Part 2 ch4: README가 1095줄로 목표(1000줄)를 넘었습니다. 14개 항목의 에러 인용이 대부분이며, 줄일지는 읽어 보고 정합니다.
- Part 2 전체: ch1~ch4를 동시에 작성해서, 서로를 링크 없이 코드 서식의 이름으로만 가리키는 곳이 있습니다. 상대 링크로 바꾸는 것은 선택 사항입니다.
- Part 2 ch5: Part 10에서 Miri를 준비하면 `examples/03_env_unsafe.rs`와 `projects/ecosystem_compare/shared/main.rs`의 unsafe 블록을 확인합니다. `projects/msrv_resolver`의 결과(`smol_str` 0.3.2와 0.3.7)는 crates.io의 2026-11-19 상태에 기대므로, Part 3 `ch2-dependencies`를 쓸 때 다시 확인합니다.
- Part 2 ch6: Part 10에서 Miri를 준비하면 `examples/02_fix_leftovers_2024.rs`의 unsafe 블록을 확인합니다. 린트 수준의 우선순위(`--force-warn`, `-W`, `#[allow]`)와 `[lints]` 테이블은 Part 5 `ch3-lints`로, 에디션과 무관한 `cargo fix`는 Part 6 `ch1-reading-diagnostics`로, 빌드 스크립트가 만든 코드의 마이그레이션은 Part 3 `ch8-build-scripts`로 미뤘습니다.

- 부록 A ch1~ch10: 동시에 작성해서 부록 챕터끼리는 링크 없이 코드 서식의 이름으로만 가리키는 곳이 있습니다. 35개가 모두 생긴 뒤 상대 링크로 바꿉니다(`ch5-type-conversions`는 Part 10에도 같은 이름이 있으므로 기계적으로 바꾸지 않습니다).
- Part 1 ch8: README 2.4절 끝이 "반복을 줄여 주는 크레이트는 Part 7 `ch2-implementation-errors`에서 다룬다"고 적고 있습니다. `thiserror`·`anyhow`의 기본은 부록 A ch4가 다루므로 함께 가리키도록 고칠지 정합니다.
- 부록 A ch4 → `ch33-no-std`: `#![no_std]` 라이브러리에서 `thiserror`를 `default-features = false`로 쓰는 것, `core::error::Error`(1.81.0)를 이어받습니다. → Part 7 `ch2-implementation-errors`: `anyhow`의 `context`와 종료 코드.
- 부록 A ch1 → `ch21-semver`: 공개 열거형에 배리언트를 추가하는 호환성 문제와 `#[non_exhaustive]`. ch2 → `ch21-semver`: 공개 함수의 바운드를 조이면 부르는 쪽이 깨진다는 점. ch3 → `ch18-dont-panic`: `unwrap`/`expect`.

- 부록 A ch5 → `ch12-generics-vs-trait-objects`: "제네릭 부분은 `into()` 한 줄, 실제 일은 제네릭이 아닌 함수"로 단형성화 비용을 줄이는 모양, `where Self: Sized`로 제네릭 메서드를 dyn 호환 트레이트에 남기기. → `ch19-avoid-reflection`: 업캐스트의 쓰임. → `ch29-listen-to-clippy`: pedantic cast 린트를 프로젝트 단위로 켜기. ch6 → `ch34-ffi-boundaries`·Part 10 `ch2-data-layout`: `#[repr(transparent)]`.

- 부록 A 전체: 위임 메시지의 "추가 요청"에 넣은 판단 기준이 그대로 챕터의 지침이 되어, ch1·ch3·ch5~ch10에 책에 없는 지침이 들어갔습니다(각 README에 밝힘). ch11 이후를 위임할 때는 먼저 책의 해당 Item이 무엇을 권하는지 확인하고, 더할 판단 기준은 "책에 없는 지침"으로 표시하게 합니다. 이미 들어간 지침을 둘지 뺄지는 정합니다.
- 부록 A ch7 → `ch21-semver`: 공개 구조체에 필드를 추가하면 사용자의 구조체 리터럴이 깨지는 문제. → `ch22-minimize-visibility`: 필드를 비공개로 두는 이유. → Part 8 `ch5-static-guarantees`: 타입 상태 빌더. ch8 → `ch14-lifetimes`·`ch15-borrow-checker`: 슬라이스 보관과 라이프타임, 인덱스로 가리키기. ch9 → `ch20-avoid-over-optimizing`: 성능 측정, `ch15-borrow-checker`: 두 클로저가 같은 상태를 고칠 때의 구조 변경. ch10 → `ch21-semver`: 트레이트 구현을 더하거나 뗄 때의 호환성과 `derive(Ord)`의 필드 순서, `ch17-shared-state-parallelism`: `unsafe impl Send/Sync`.
- 부록 A ch10: README가 `ch9-iterator-transforms`를 링크 없이 이름으로만 가리킵니다(동시에 작성). 상대 링크로 바꿀 수 있습니다(선택 사항).

- 부록 A ch11~ch15: 동시에 작성해 서로를 이름으로만 가리키는 곳이 있습니다(ch13 → `ch12-generics-vs-trait-objects`, ch8·ch9·ch14 → `ch15-borrow-checker`, ch2·ch5·ch8·ch9 → `ch12-…`). 35개가 모두 생긴 뒤 상대 링크로 바꿉니다.
- 부록 A ch11 → `ch17-shared-state-parallelism`: 잠금 범위를 너무 좁혀 한 번에 끝나야 할 일을 쪼개는 문제, 잠금 순서와 교착 상태. → `ch18-dont-panic`·Part 10 `ch8-unwinding`: 이중 패닉, `catch_unwind`. → Part 10 `ch7-obrm`: 소멸자가 실행되지 않는 경우. ch12 → `ch19-avoid-reflection`: 업캐스트의 쓰임과 다운캐스트, `ch20-avoid-over-optimizing`: 릴리스 빌드에서 얇은 껍데기가 줄이는 크기의 측정, `ch21-semver`: 공개 트레이트에 dyn 호환이 아닌 메서드를 더하는 문제. ch13 → `ch21-semver`: 호환되지 않는 변경의 전체 규칙. ch14 → `ch17-…`: 스레드와 `'static`, Part 10: 변성·HRTB·drop 검사. ch15 → `ch16-avoid-unsafe`: 자기 참조를 `unsafe`로 구현하는 일과 원시 포인터, `ch17-…`: `Arc<Mutex<T>>`, `ch18-…`: `RefCell` 패닉, `ch22-minimize-visibility`: 비공개 필드로 불변식 지키기.
- 부록 A ch15: README가 소개하는 크레이트 버전(`slotmap` 1.1.1, `ouroboros` 0.18.5, `self_cell` 1.3.0, `yoke` 0.8.4)은 2026-11-19의 crates.io 기준입니다.

## 용어 메모

챕터 사이에 표기를 맞추기 위해 정한 용어를 적습니다.

| 영어 | 표기 |
| --- | --- |
| ownership / move / borrow / reference | 소유권 / 이동 / 대여 / 참조 |
| lifetime / trait / generic | 라이프타임 / 트레이트 / 제네릭 |
| statement / expression | 구문 / 표현식 |
| mutable / immutable / shadowing | 가변 / 불변 / 섀도잉 |
| binding / scope | 바인딩 / 스코프 |
| parameter / argument | 매개변수 / 인자 |
| crate / package / module | 크레이트 / 패키지 / 모듈 |
| struct / enum / method / associated function | 구조체 / 열거형 / 메서드 / 연관 함수 |
| closure / iterator | 클로저 / 반복자 |
| match arm / variant / pattern | 갈래 / 배리언트 / 패턴 |
| type annotation / dependency | 타입 표기 / 의존성 |
| owner / copy / clone | 소유자 / 복사 / 복제 |
| dangling reference / borrow checker / data race | 댕글링 참조 / 대여 검사기 / 데이터 경쟁 |
| double free / use after free | 이중 해제 / 해제 후 사용 |
| dereference / deref coercion | 역참조 / 역참조 강제 변환 |
| drop | "버려지다(drop)", "drop된다" |
| toolchain / workspace / edition / profile / manifest | 툴체인 / 워크스페이스 / 에디션 / 프로필 / 매니페스트 |
| format string / placeholder / format spec | 서식 문자열 / 자리표시자 / 서식 지정 |
| `Display` `{}` / `Debug` `{:?}` | 기본 형식 / 디버그 형식 |
| constant / literal / destructuring / unit | 상수 / 리터럴 / 구조 분해 / 유닛 |
| overflow / wrapping / tail expression / label / range | 오버플로 / 래핑 / 마지막 표현식 / 레이블 / 범위 |
| debug build / release build / exit code | 디버그 빌드 / 릴리스 빌드 / 종료 코드 |
| field / instance / attribute / getter | 필드 / 인스턴스 / 속성 / 게터 |
| field init shorthand / struct update syntax / partial move | 필드 초기화 축약 / 구조체 갱신 문법 / 부분 이동 |
| tuple struct / unit-like struct | 튜플 구조체 / 유닛 구조체 |
| automatic referencing and dereferencing | 자동 참조·역참조 |
| module tree / crate root / binary crate / library crate | 모듈 트리 / 크레이트 루트 / 바이너리 크레이트 / 라이브러리 크레이트 |
| path / absolute path / relative path | 경로 / 절대 경로 / 상대 경로 |
| private / public / re-export / encapsulation | 비공개 / 공개 / 재내보내기 / 캡슐화 |
| nested path / glob / alias / namespace | 중첩 경로 / 글롭 / 별칭 / 이름 공간 |
| item / prelude | 항목 / 프렐류드 |
| catch-all pattern / wildcard `_` | 포괄 패턴 / `_` 패턴(와일드카드) |
| exhaustiveness checking / refutable / irrefutable | 완전성 검사 / 반박 가능한 / 반박 불가능한 |
| happy path / diverge | 행복한 경로 / 흐름을 끊다 |
| collection / vector / hash map / key / value | 컬렉션 / 벡터 / 해시 맵 / 키 / 값 |
| length / capacity / reallocation | 길이 / 용량 / 재할당 |
| grapheme cluster / code point | 자소 클러스터 / 코드 포인트 |
| unrecoverable / recoverable error | 복구할 수 없는 / 복구할 수 있는 에러 |
| unwinding / backtrace / propagating errors | 되감기 / 백트레이스 / 에러 전파 |
| invariant / validation type / type alias | 불변식 / 검증 타입 / 별칭 |
| command line argument / environment variable / redirection | 명령줄 인자 / 환경 변수 / 리다이렉션 |
| separation of concerns / TDD | 관심사 분리 / 테스트 주도 개발 |
| type parameter / concrete type / monomorphization | 타입 매개변수 / 구체 타입 / 단형성화 |
| trait bound / default implementation / override | 트레이트 바운드 / 기본 구현 / 재정의 |
| orphan rule / coherence / blanket implementation | 고아 규칙 / 일관성 / 포괄 구현 |
| lifetime annotation / lifetime parameter / elision rules | 라이프타임 표기 / 라이프타임 매개변수 / 생략 규칙 |
| trait object | 트레이트 객체 |
| test runner / assertion | 테스트 러너 / 단언 |
| unit / integration / doc test | 단위 테스트 / 통합 테스트 / 문서 테스트 |
| capture (test output) | 가로채기 |
| smart pointer / indirection / recursive type | 스마트 포인터 / 간접 참조 / 재귀 타입 |
| destructor / reference counting / strong·weak reference | 소멸자 / 참조 카운팅 / 강한·약한 참조 |
| interior mutability / mock object | 내부 가변성 / 가짜 객체 |
| reference cycle / memory leak / associated type | 순환 참조 / 메모리 누수 / 연관 타입 |
| concurrency / parallelism / thread | 동시성 / 병렬성 / 스레드 |
| message passing / channel / transmitter / receiver | 메시지 전달 / 채널 / 송신자 / 수신자 |
| shared state / mutex / lock / poisoning | 공유 상태 / 뮤텍스 / 잠금 / 포이즈닝 |
| race condition / deadlock / marker trait | 경쟁 조건 / 교착 상태 / 마커 트레이트 |
| capture / iterator adapter / consuming adapter | 캡처 / 어댑터 / 소비 메서드 |
| lazy / turbofish / zero-cost abstraction | 게으르다 / 터보피시 / 비용 없는 추상화 |
| future / task / runtime / executor / stream | 퓨처 / 태스크 / 런타임 / 실행기 / 스트림 |
| await point / cooperative multitasking / starvation / blocking | await 지점 / 협력적 멀티태스킹 / 굶주림 / 블로킹 |
| pin / self-referential / state machine | 고정 / 자기 참조 / 상태 기계 |
| worker thread / extension trait / cancel | 작업자 스레드 / 확장 트레이트 / 취소 |
| static / dynamic dispatch / dyn compatible | 정적 디스패치 / 동적 디스패치 / dyn 호환 |
| state pattern / polymorphism / inheritance | 상태 패턴 / 다형성 / 상속 |
| default binding mode / match guard / let chains | 기본 바인딩 모드 / 매치 가드 / let 체인 |
| range pattern / slice pattern / rest `..` | 범위 패턴 / 슬라이스 패턴 / 나머지 |
| raw pointer / undefined behavior / safe abstraction | 원시 포인터 / 미정의 동작 / 안전한 추상화 |
| operator overloading / fully qualified syntax / supertrait | 연산자 오버로딩 / 완전 정규화 문법 / 슈퍼트레이트 |
| newtype pattern / never type / DST / wide pointer / function pointer | 뉴타입 패턴 / never 타입 / 동적 크기 타입 / 넓은 포인터 / 함수 포인터 |
| declarative / procedural macro / token stream / expand | 선언적 / 절차적 매크로 / 토큰 스트림 / 펼쳐지다 |
| thread pool / graceful shutdown / worker / job / queue | 스레드 풀 / 우아한 종료 / 작업자 / 작업 / 대기열 |
| request line / status line / header / body | 요청 줄 / 상태 줄 / 헤더 / 본문 |
| temporary (value) / bind | 임시값 / 바인드 |
| stdout / stderr | 표준 출력 / 표준 에러 |
| signature / primitive type / combinator | 시그니처 / 원시 타입 / 콤비네이터 |
| pattern (`str` 메서드의 인자) / parsing / round-trip | 검색 패턴 / 파싱 / 왕복 |
| formatting trait / alternate form `#` / helper (`debug_struct` 등) | 서식 트레이트 / 대체 형식 / 도우미 |
| allocation / in-place mutation / replacement character | 할당 / 제자리 수정 / 대체 문자 |
| eager evaluation / short-circuit | 즉시 평가 / 단락 평가 |
| nested `Option<Option<T>>` / `flatten` | 두 겹 / 한 겹 벗긴다 |
| binary search / insertion point / priority queue | 이진 탐색 / 삽입 위치 / 우선순위 큐 |
| stable / unstable sort / total order / lexicographic | 안정 정렬 / 불안정 정렬 / 전순서 / 사전식 |
| deque / ring buffer / set / binary heap / max-heap / min-heap | 덱 / 링 버퍼 / 집합 / 이진 힙 / 최대 힙 / 최소 힙 |
| union / intersection / difference / symmetric difference | 합집합 / 교집합 / 차집합 / 대칭 차집합 |
| subset / superset / amortized / expected cost | 부분집합 / 상위집합 / 분할 상환 / 기대 비용 |
| allocator / seed / cursor | 할당기 / 시드 / 커서 |
| conversion / fallible / infallible / lossless | 변환 / 실패할 수 있는 / 실패하지 않는 / 손실 없는 |
| partial order / equivalence relation / logic error | 부분 순서 / 동치 관계 / 논리 오류 |
| hasher / `BuildHasher` / prefix-free | 해셔 / 해셔 공장 / 경계가 섞이지 않게(prefix-free) |
| clone-on-write / reflexive impl / discriminant | 쓰기 시 복제 / 반사 구현 / 판별값 |
| alignment / padding / layout / representation | 메모리 정렬 / 패딩 / 메모리 배치 / 표현(기본 표현, 표현 지정) |
| niche / tag / thin pointer / zero-sized type | 니치 / 태그 / 얇은 포인터 / 크기가 0인 타입 |
| handle (`JoinHandle`, 표준 입출력·파일·소켓의 핸들, `Vec`·`String`의 포인터·용량·길이 구조체) / placeholder value | 핸들 / 자리 표시 값 |
| guard (`Ref`/`RefMut`) / reentrancy / scrutinee | 대여 가드 / 재진입 / 조건식 |
| panic hook / payload / unwind safety / unwind boundary | 패닉 훅 / 페이로드 / 되감기 안전성 / 되감기 경계 |
| `source` chain / downcast / wrap / error report | `source` 체인 / 다운캐스트 / 감싸다 / 에러 보고서 |
| borrowed / owned (`Cow`, `Borrow`의 문맥) | 빌린 / 소유한 ("빌림과 소유") |
| scoped thread / condition variable / spurious wakeup | 스코프 스레드 / 조건 변수 / 가짜 깨어남 |
| atomic type / memory ordering / compare-and-swap | 원자 타입 / 메모리 순서 / 비교 후 교환 |
| bounded channel / rendezvous channel / backpressure | 용량이 정해진 채널 / 랑데부 채널 / 배압 |
| thread-local / lazy initialization / reader-writer lock | 스레드 지역 변수 / 지연 초기화 / 읽기-쓰기 잠금 |
| snapshot / busy waiting / work stealing | 스냅숏 / 바쁜 대기 / 작업 훔치기 |
| socket address / loopback / listener / datagram / ephemeral port | 소켓 주소 / 루프백 / 리스너 / 데이터그램 / 임시 포트 |
| byte stream / framing / length prefix / delimiter | 바이트 스트림 / 메시지 경계 나누기(프레이밍) / 길이 접두사 / 구분자 |
| timeout / name resolution / retransmission / non-blocking | 제한 시간 / 이름 해석 / 재전송 / 논블로킹 |
| child / parent process / pipe / exit status / builder | 자식 프로세스 / 부모 프로세스 / 파이프 / 종료 상태 / 빌더 |
| monotonic clock / system clock / Unix time / epoch | 단조 시계 / 시스템 시계 / 유닉스 시간 / 기준 시각 |
| elapsed time / deadline / polling / zombie / reap | 경과 시간 / 마감 시점 / 폴링 / 좀비 / 거둬들이다 |
| reader / writer / buffering / flush / line buffering | 리더 / 라이터 / 버퍼링 / 플러시(버퍼 비우기) / 줄 버퍼링 |
| system call / EOF / seek / metadata | 시스템 호출 / EOF(입력의 끝) / 위치 옮기기 / 메타데이터 |
| path component / separator / extension / symbolic link / canonical path | 구성 요소 / 구분자 / 확장자 / 심볼릭 링크 / 정규 경로 |
| poll / waker / wake / pinned / task queue / busy polling | 폴링 / 웨이커 / 깨우다 / 고정된 / 태스크 대기열 / 바쁜 폴링 |
| auto trait / pin projection / oneshot channel | 자동 트레이트 / 고정 투영 / 일회용 채널(oneshot) |
| C string / NUL terminator / interior NUL | C 문자열 / NUL 종료 바이트(0 바이트) / 중간의 0 바이트 |
| OS string (`OsString`/`OsStr`) / upcast / type map | 운영체제 문자열 / 업캐스트 / 타입 맵 |
| fold / helper trait (`DoubleEndedIterator` 등) / iterator struct (`Map`, `Drain` 등) | 접기 / 보조 트레이트 / 반복자 구조체 |
| associated constant / inherent impl / extern prelude | 연관 상수 / 고유 구현 / 외부 프렐류드 |
| checked / saturating 연산 / Euclidean division / endian | 검사 연산 / 포화 연산 / 유클리드 나눗셈 / 엔디언 |
| since (문서의 버전 표시) / nightly-only experimental API / badge | 안정화 버전 / nightly 전용 실험 API / 표시 |
| API 문서의 절 이름 (Implementations, Methods from Deref, Trait Implementations 등) | 영어 그대로 |
| backward compatibility / breaking change / migration | 하위 호환성 / 호환되지 않는 변경 / 마이그레이션 |
| lint / lint group / lint level | 린트 / 린트 그룹 / 린트 수준 (Part 2부터. Part 0·1의 일부 챕터와 위 작업 방식 메모는 lint로 썼음) |
| edition compatibility lint group (`rust-2021-compatibility`) / idiom lint group (`rust_2018_idioms`) | 에디션 호환성 린트 그룹 / 관용구 린트 그룹 |
| strict / reserved / weak keyword / raw identifier | 엄격한 키워드 / 예약 키워드 / 약한 키워드 / raw 식별자 |
| baseline (2015 에디션) / stability without stagnation | 기준선 / 정체 없는 안정성 |
| virtual manifest / sysroot crate / inference variable | 가상 매니페스트 / sysroot 크레이트 / 추론 변수 |
| reserved syntax / prefix / token tree | 예약 문법 / 접두사 / 토큰 트리 |
| feature resolver / feature unification | 기능 리졸버(기능 리졸버 1/2) / 기능 통합 |
| disjoint capture / dummy let (`let _ = &x;`) / method lookup | 분리 캡처 / 더미 let / 메서드 탐색 |
| temporary scope / temporary lifetime extension | 임시값 범위 / 임시값 수명 연장 |
| RPIT / opaque type / hidden type / precise capturing (`use<..>`) | 반환 위치의 `impl Trait`(RPIT) / 불투명 타입 / 숨은 타입 / 정밀 캡처 |
| never type fallback / fallback type | never 타입 폴백 / 폴백 타입 |
| fragment specifier / fully explicit pattern / symbol | 프래그먼트 지정자 / 완전히 명시한 패턴 / 심볼 |
| deprecated | 처음 한 번 "더는 권장하지 않는(deprecated)"으로 풀어 쓰고 이후 deprecated |
| future-compatibility warning / automatic fix / suggestion | 장래 호환성 경고 / 자동 수정 / 제안 |
| 특정 에디션 | "2024 에디션" 꼴 (Part 2부터. 앞 파트의 "Edition 2024"와 같은 뜻) |
| merged doctests / standalone | 문서 테스트 병합(묶음 컴파일, 묶음 바이너리) / 따로 컴파일 |
| rust-version aware resolver / MSRV / yank | 리졸버 3(rust-version 인식 리졸버) / 최소 지원 Rust 버전(MSRV) / 게시 취소(yank) |
| style edition / version sort / optional dependency | 스타일 에디션 / 버전 정렬 / 선택적 의존성 |
| async-signal-safe | 비동기 시그널 안전 |
| (마이그레이션) 자동 수정 뒤 정리 / 고치지 않고 경고만 보기 | 다듬기 / 미리 보기 |
| idiom / working tree / staged | 관용구 / 작업 트리 / 스테이지에 올린 |
| (부록 A) Item / guideline / idiom | Item(영어 그대로, "Item 4") / 지침 / 관용구 |
| (부록 A) README의 고정 표현 | 비권장 코드 / 권장 코드 / 지침을 따르지 않아도 되는 경우 / 기억할 것 / 책과 달라진 점 |
| (부록 A) 같은 코드의 여러 버전 / 변환 메서드를 이어 쓴 것 | "~판"(`match`판, 손으로 쓴 판, `thiserror` 판) / 체인 |
| caller / callee (시그니처 설계의 문맥) | 부르는 쪽 / 받는 쪽 (에러를 돌려받는 문맥에서는 본편처럼 "호출한 쪽"도 씀) |
| function body / method receiver / fn item | 본문 / 수신자 / 함수 아이템 |
| loosest bound / idempotent | 가장 느슨한 바운드 / 멱등 |
| widening / narrowing conversion / boundary | 넓히는 변환 / 좁히는 변환 / 경계(바깥 입력을 안쪽 타입으로 바꾸는 지점) |
| product type / sum type / algebraic data type | 곱 타입 / 합 타입 / 대수적 데이터 타입 |
| sentinel value / invalid state / red flag | 센티널 값 / 무효한 상태 / 경고 신호 |
| transform (`Option`/`Result`의 메서드) | 변환 메서드 |
| nested error / minimal error / type erasure | 중첩 에러 / 최소 에러 / 종류를 지우다 |
| coercion site / sign extension | 강제 변환 자리 / 부호 확장 |
| unit newtype / validated newtype / delegating impl / accessor | 단위 뉴타입 / 검증 뉴타입 / 위임 구현 / 읽기용 접근자 |
| setter / options struct / typestate builder | 세터 / 옵션 구조체(필수 값을 뺀 선택 값 묶음 + `Default`) / 타입 상태 빌더 |
| consuming builder / non-consuming builder | 소비형 빌더 / `&mut` 빌더 |
| vtable / (wide pointer) metadata | vtable(처음 한 번 "메서드 포인터 표"로 풀어 씀) / 메타데이터 |
| accumulator / side effect / early exit / bounds check | 누적 변수 / 부수 효과 / 탈출 조건 / 경계 검사 |
| checklist (공개 타입의 트레이트 판단표) / normalize / (Copy의) copy | 체크리스트 / 정규화 / 사본 |
| 트레이트 구현 사이에 맞아야 하는 성질(`Eq`와 `Hash` 등) | "서로 맞아야 한다", "같은 기준"으로 풀어 씀("일관성"은 coherence에만 씀) |
| heterogeneous collection / combination trait / open·closed set | 이종 컬렉션(처음 한 번 풀어 씀) / 조합 트레이트 / 열린 집합·닫힌 집합 |
| thin generic shell / inner function / symbol / mangling | 얇은 껍데기 / 안쪽 함수 / 심볼 / 맹글링 |
| required method / provided method / implementor / user (of a trait) | 필수 메서드 / 기본 메서드 / 구현하는 쪽 / 쓰는 쪽 |
| RAII / guard (일반 개념) / release method / safety net / double panic | RAII(처음 한 번 "자원 획득은 초기화"로 풀어 씀) / 가드 / 해제 메서드 / 안전망 / 이중 패닉 |
| chain of ownership / view type / non-lexical lifetimes / anonymous lifetime / constant promotion | 소유 사슬 / 뷰 타입 / 마지막 사용에서 끝나는 대여(NLL) / 익명 라이프타임 / 상수 승격 |
| `&'static T` / `T: 'static` | 영원히 유효한 참조 / 짧은 대여를 품지 않은 타입 |
| shared reference (`&T`) | 불변 참조(본편 표기. 책의 "shared reference"는 처음 한 번만 병기) |
| two-phase borrow / disjoint borrow / generation (index) | 2단계 대여 / 나눠 빌리기 / 세대 ID·세대 번호 |
| `take`·`replace`로 `&mut` 뒤의 값을 꺼내는 일 | 바꿔 끼우기 |
| critical section / lock inversion / lock ordering (hierarchy) / rank | 임계 구역 / 잠금 순서 뒤집힘 / 잠금 순서 / 순위 |
| (함께 맞아야 하는 데이터가 서로 맞지 않는 상태) / helper method / snapshot(복사해 둔 값) | 어긋난 상태 / 도우미 메서드 / 사본 |
| abort / exception safety / hidden panic / restriction group | 중단(abort) / 예외 안전성 / 숨은 패닉 지점 / restriction 그룹(처음 한 번 "일부러 고른 프로젝트만 켜는 린트") |
| panicking version / fallible version (같은 기능의 두 판) | 패닉판 / 실패를 돌려주는 판(`Result`판·`Option`판) |
| reflection / RTTI / downcast chain / capability query method / optional method | 리플렉션 / 실행 중 타입 정보(RTTI) / 다운캐스트 사슬 / 기능 조회 메서드 / 선택적 메서드 |
| zero-copy / premature optimization / benchmark / hot path / median | 복사 없는(zero-copy) / 섣부른 최적화 / 벤치마크 / 뜨거운 경로 / 가운데 값 |
| count (결정적인 횟수·바이트) / measure (경과 시간) | 세다 / 재다 |
