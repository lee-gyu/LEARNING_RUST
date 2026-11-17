# 진척 현황

[로드맵](docs/roadmap.md)의 챕터별 작업 상태를 기록합니다. 챕터 하나가 끝날 때마다 이 문서와 로드맵의 상태 칸을 함께 갱신합니다.

- 마지막 갱신: 2026-11-18
- 툴체인: rustc 1.98.1, cargo 1.98.1 (Edition 2024)

상태 표기: ⬜ 대기 · 🔄 작성 중 · ✅ 완료(검증 통과) · ⚠️ 부분 완료(비고 참고)

## 요약

| 구분 | 완료 / 전체 |
| --- | --- |
| 사전 작업 | 3 / 3 |
| Part 0. 시작하기 | 20 / 20 |
| Part 1~11 | 0 / 89 |

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

## Part 1~11

아직 시작하지 않았습니다. 챕터 목록은 [로드맵 3절](docs/roadmap.md#3-파트별-챕터-계획)을 따르며, 파트를 시작할 때 이 문서에 표를 추가합니다.

## 작업 방식 메모

- 챕터는 `tutor` 서브 에이전트(`.claude/agents/tutor.md`)가 하나씩 작성하고, 메인 세션이 보고를 받아 이 문서·로드맵·파트 README를 갱신합니다.
- 워크스페이스 `members = ["part*/ch*"]` 때문에 `Cargo.toml`이 없거나 빌드 타깃이 하나도 없는 챕터 디렉터리가 있으면 **모든 챕터의 cargo 명령이 실패**합니다. 여러 챕터를 동시에 작성할 때는 메인 세션이 `Cargo.toml`과 자리표시 예제(`examples/00_placeholder.rs`)를 갖춘 디렉터리를 먼저 만들어 둡니다. 에이전트마다 `CARGO_TARGET_DIR`을 따로 줘서 빌드 디렉터리도 나눕니다.
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
- 스레드 출력 순서를 고정하려면 스레드 안에서 출력하지 말고 결과를 돌려받아 `main`에서 출력합니다. 공유된 `Mutex`를 여러 스레드에서 디버그 형식으로 출력하면 `"<locked>"`가 섞여 출력이 비결정적이 됩니다.

## 후속 조치

챕터 작성 중 발견한, 다른 파일에 반영해야 할 사항을 적습니다.

- ch2: 이 워크스페이스에서 `cargo doc -p rand`는 cargo 1.98.1 내부 패닉으로 끝납니다(`rand`가 dev-dependency뿐이라 생기는 문제로 보임). README에는 대안 `cargo doc -p p0-ch2-guessing-game --examples --open`을 안내했습니다. Part 3 `ch2-dependencies` 작성 때 다시 확인합니다.
- ch9: 워크스페이스 멤버 패키지의 `[profile]`은 무시되고 경고가 납니다(`profiles for the non root package will be ignored`). 프로필은 루트 `Cargo.toml`에만 둘 수 있습니다. Part 3 `ch6-profiles-build-performance` 작성 때 반영합니다.
- ch12: 2024 에디션에서는 `std::env::set_var`가 `unsafe`입니다(그냥 부르면 E0133). Part 2 `ch5-rust-2024-ecosystem`에서 다룹니다.
- ch11: 에디션 2024의 문서 테스트 병합(`all doctests ran in …; merged doctests compilation took …` 줄)은 Part 2 `ch5-rust-2024-ecosystem`에서, `cargo test 이름`이 문서 테스트를 건너뛰는 동작은 Part 3 `ch9-testing-ci`에서 다시 다룹니다.
- ch18: `while let Some(top) = stack.pop() && top > 2`는 조건이 거짓이 되는 순간에도 원소 하나를 이미 꺼냅니다. Part 2 `ch4-rust-2024-language`에서 let 체인의 함정으로 다룰 만합니다.
- ch19: unsafe 예제는 Miri로 확인하지 못했습니다. Part 10을 시작해 nightly와 Miri를 준비하면 `ch19-advanced-features`의 `01`, `02` 예제를 `cargo +nightly miri run`으로 다시 확인합니다.
- Part 0 전체: 챕터를 동시에 작성했기 때문에, README가 그때 아직 없던 챕터를 링크 없이 코드 서식의 이름(`ch10-generics-traits-lifetimes`)으로만 가리키는 곳이 있습니다. 내용에는 문제가 없고, 상대 링크로 바꾸는 것은 선택 사항입니다.
- Part 0 전체: 예제 수(챕터당 4~13개)와 README 분량(600~1280줄)이 `tutor` 문서의 기준(보통 3~6개)보다 많은 챕터가 있습니다. 줄일지는 읽어 보고 정합니다.

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
