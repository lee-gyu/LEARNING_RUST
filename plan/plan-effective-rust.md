# 부록: Effective Rust 집필 계획

[Effective Rust](https://effective-rust.com/)(David Drysdale, O'Reilly 2024)의 35개 Item을 [로드맵](../docs/roadmap.md)의 본편(Part 0~11)과 별도인 **부록 파트**로 집필하기 위한 작업 계획입니다. 챕터 작성 방식은 본편과 같습니다. `/writing-guide` 스킬이 챕터마다 `tutor` 에이전트에게 위임하고, 메인 세션이 검증과 진척 관리를 맡습니다.

- 작성일: 2026-11-19
- 규모: 사전 작업 9건, 챕터 35개(Item 1개 = 챕터 1개), 마무리 1회
- 선수 과정: Part 0, Part 1 (둘 다 완료). 본편의 나머지 파트와는 순서가 무관합니다.
- 상태 표기: ⬜ 대기 · 🔄 작성 중 · ✅ 완료
- 2026-11-19 변경: 사용자 결정으로 부록 디렉터리 이름을 `appd-a/`로 정했습니다(처음 계획은 `appendix-effective-rust/`). 챕터 구조는 본편과 같습니다(디렉터리 + `README.md` + `examples/`).

## 1. 개요

### 1.1 기준 문서

책은 6개 장, 35개 Item으로 이뤄져 있습니다. 목차는 2026-11-19에 사이트에서 직접 확인했습니다.

| 책의 장 | Item | 부록 챕터 | 성격 |
| --- | --- | --- | --- |
| 1. Types | 1~9 | ch1~ch9 | 타입 시스템으로 설계하기 |
| 2. Traits | 10~13 | ch10~ch13 | 트레이트 다루기 |
| 3. Concepts | 14~20 | ch14~ch20 | Rust 설계의 핵심 개념 |
| 4. Dependencies | 21~26 | ch21~ch26 | 패키지 생태계와 의존성 |
| 5. Tooling | 27~32 | ch27~ch32 | 컴파일러 밖의 도구 |
| 6. Beyond Standard Rust | 33~35 | ch33~ch35 | `no_std`, FFI |

Preface, 각 장의 머리말, Afterword는 챕터로 만들지 않고 부록 README의 개요에 반영합니다.

### 1.2 본편과 다른 점

- **독자**: 책 스스로 "Rust 입문서 다음에 읽는 두 번째 책"이라고 밝힙니다. 부록의 독자는 Part 0과 Part 1을 마친 사람입니다. 문법과 API 사용법은 본편 챕터로 링크하고 반복하지 않습니다.
- **관점**: 본편은 "무엇이 있고 어떻게 쓰는가"를, 부록은 "여러 방법 중 무엇을 왜 고르는가"를 다룹니다. 챕터의 중심은 비권장 코드와 권장 코드의 비교, 그리고 지침을 따르지 않아도 되는 경우입니다.
- **범위 밖**: 책은 비동기를 다루지 않습니다. 부록도 다루지 않습니다.

### 1.3 저작권

온라인판은 **CC BY-NC-ND 4.0**(저작자 표시, 비영리, 변경 금지)입니다. 이 저장소는 GitHub Pages로 공개되므로 다음을 지킵니다.

- 본문을 번역하거나 발췌해 싣지 않습니다. 책의 코드 조각도 옮겨 오지 않습니다.
- 설명의 흐름과 예제는 새로 설계합니다. 책에서 가져오는 것은 "어떤 지침을 다루는가"라는 범위뿐입니다.
- Item 원제는 영어 그대로 인용하고 해당 페이지로 링크합니다. 부록 README에 저자, 책 제목, 라이선스를 밝힙니다.

### 1.4 책과 저장소의 버전 차이

책은 **2018 에디션, rustc 1.70** 기준이고 저장소는 **2024 에디션, rustc 1.100**입니다. 사실의 기준은 저장소 툴체인입니다. 책과 실제 동작이 다르면 실제 동작대로 쓰고 README에 "책과 달라진 점"으로 밝힙니다. 다음은 미리 알고 있는 후보이며, 해당 챕터를 쓸 때 직접 확인합니다.

| 챕터 | 확인할 것 |
| --- | --- |
| ch2, ch4, ch12 | 책의 "object safety"는 현재 용어로 "dyn 호환"입니다(`PROGRESS.md` 용어 메모). |
| ch4, ch33 | `Error` 트레이트를 `core::error::Error`로도 쓸 수 있습니다. 책은 `std` 전용으로 설명합니다. |
| ch14, ch15 | 책의 에러 메시지는 1.70 기준입니다. 2024 에디션의 임시값 범위 규칙과 let 체인이 대여 범위 예제에 영향을 주는지 봅니다. |
| ch16 | 책이 권하는 `once_cell` 크레이트의 기능 상당수가 표준 라이브러리(`OnceLock`, `LazyLock`)에 들어왔습니다. |
| ch19 | 책은 트레이트 객체 업캐스트를 "앞으로 들어올 기능"으로 설명합니다. 현재 툴체인에서 동작하는지 확인하고 그 결과대로 씁니다. |
| ch21, ch25 | 리졸버 3의 `rust-version` 인식 동작처럼 책 이후에 생긴 Cargo 기능을 반영합니다. |
| ch26 | `cfg` 이름과 값의 오타를 잡는 `unexpected_cfgs` lint가 생겼습니다. |
| ch29 | `#[expect]` 속성과 `Cargo.toml`의 `[lints]` 테이블은 책 이후에 생겼습니다. |
| ch34, ch35 | 2024 에디션에서는 `unsafe extern "C" { … }`, `#[unsafe(no_mangle)]`로 써야 합니다. `bindgen`이 만드는 코드가 2024 에디션에서 컴파일되는지 확인합니다. |

## 2. 구조와 규칙

로드맵 2절(저장소 구조와 규칙)을 그대로 따르고, 아래만 다릅니다.

### 2.1 디렉터리와 이름

```
LEARNING_RUST/
├── appd-a/
│   ├── README.md                       # 부록 개요, 출처와 라이선스, 챕터 목차(책의 장별로 묶음)
│   ├── ch1-express-data-structures/    # Item 1
│   │   ├── README.md
│   │   ├── Cargo.toml
│   │   ├── examples/
│   │   ├── compile_fail/               # (선택)
│   │   └── projects/                   # (선택)
│   ├── ch2-express-common-behavior/    # Item 2
│   └── ...                             # ch35까지
```

| 대상 | 규칙 | 예시 |
| --- | --- | --- |
| 부록 디렉터리 | `appd-{문자}` | `appd-a` |
| 챕터 디렉터리 | `ch{N}-{slug}`, **N은 Item 번호** | `ch14-lifetimes` |
| 패키지 이름 | `er-ch{N}-{slug}` | `er-ch14-lifetimes` |
| 다른 Item을 가리킬 때 | `ch14`(Item 14) | 본편 챕터는 `Part 1 ch8` |

챕터 번호와 Item 번호가 같으므로 책의 "Item 14 참고"를 그대로 `ch14`로 옮길 수 있습니다. 부록을 더 추가하게 되면 `appd-{문자}/` 디렉터리와 새 패키지 접두사를 하나 더 만듭니다.

### 2.2 README 구성

로드맵 2.4절의 여섯 항목을 유지하고 다음을 더합니다.

- 제목은 `# ch3. (한국어 제목)` 형식으로 쓰고, 첫머리에 원제(`Item 3: Prefer Option and Result transforms over explicit match expressions`)와 링크, 패키지 이름을 적습니다.
- **핵심 개념**은 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 풉니다. 책이 규칙이 아니라 이유가 붙은 지침을 주는 책이므로, 예외를 빼면 내용이 달라집니다.
- 핵심 개념의 마지막에 **기억할 것**을 3~6줄로 정리합니다.
- 책과 실제 동작이 다른 곳이 있으면 **책과 달라진 점** 소절을 둡니다.

### 2.3 예제

- 예제 파일 하나가 지침 하나를 맡고, 같은 파일 안에서 비권장 형태와 권장 형태를 함께 실행해 결과를 비교합니다.
- 비권장 코드가 컴파일되지 않는 경우에만 `compile_fail/`에 둡니다. 컴파일은 되지만 틀린 결과를 내는 코드는 예제로 두고 출력으로 보여 줍니다.
- 예제는 `cargo clippy -- -D warnings`를 통과해야 합니다. clippy가 잡는 비권장 코드는 `#[expect(clippy::…, reason = "…")]`를 붙이고, 속성을 지웠을 때 경고가 실제로 나는지 확인합니다.
- 분량: 보통 4~6개, 작은 Item(ch13, ch23, ch24, ch31)은 2~3개, 큰 Item(ch8, ch9, ch10, ch14, ch15, ch28, ch34)은 8개까지. README는 300~700줄을 기준으로 하고 큰 Item도 900줄을 넘기지 않습니다.
- 주 실습이 `projects/`인 챕터(4.4~4.6절의 대부분)도 `examples/*.rs`가 하나 이상 있어야 워크스페이스가 깨지지 않습니다.

## 3. 사전 작업

첫 챕터를 위임하기 전에 한 번 합니다. 루트 `Cargo.toml`과 `PROGRESS.md`를 고치므로 **다른 파트의 tutor가 돌고 있지 않을 때** 진행합니다(2026-11-19 현재 Part 2 ch1~ch4가 작성 중입니다).

| 상태 | 번호 | 작업 | 확인 방법 |
| --- | --- | --- | --- |
| ✅ | P1 | 툴체인 확인. mise를 거치지 않은 셸에서는 rustup 기본값 1.98.1이 잡힙니다(2026-11-19 확인). tutor에게 넘길 셸에서 1.100.0이 나오는지 봅니다. | `rustc --version`, `mise exec -- rustc --version` |
| ✅ | P2 | 루트 `Cargo.toml`: `members = ["part*/ch*", "appd-*/ch*"]` | 확인 결과: 글롭에 맞는 디렉터리가 하나도 없으면 `cargo metadata`가 실패합니다. 첫 챕터 디렉터리(자리표시 예제 포함)를 만든 뒤에 추가했습니다. |
| ✅ | P3 | `docs/roadmap.md`에 부록 추가: 1절 표 아래에 부록 행, 2.2절에 이 문서 2.1절의 이름 규칙, 3절 끝에 "부록. Effective Rust" 절(이 문서 4절의 표), 4절 진행 현황에 `부록. Effective Rust 0 / 35` 행. 본편 합계(109)는 그대로 두고 부록은 따로 셉니다. | 사이트의 `/roadmap/` 페이지 |
| ✅ | P4 | `PROGRESS.md`: 요약 표에 부록 행, "부록. Effective Rust" 표(열은 Part 0 표와 동일) 추가 | — |
| ✅ | P5 | `web/scripts/sync-content.mjs`: `partDirs()`가 `^part(\d+)-`만 찾습니다. 번호순 파트 뒤에 `appd-*` 디렉터리를 이름순으로 잇도록 고치고, 파일 머리 주석의 경로 표도 고칩니다. | `cd web && pnpm build` 후 사이드바에 부록 그룹 |
| ✅ | P6 | `.github/workflows/deploy.yml`: `paths`에 `'appd-*/**'` 추가 | — |
| ✅ | P7 | `.claude/skills/writing-guide/SKILL.md`: 부록 범위 표기(`/writing-guide appd-a ch1~ch9`), 디렉터리 준비 스크립트의 패키지 접두사(`p{n}-` → `er-`), 이 문서 6절을 위임 메시지에 넣는다는 안내를 추가 | 스킬을 ch1 하나로 시험 실행 |
| ✅ | P8 | `appd-a/README.md` 작성: 개요(책 소개, 본편과의 관계, 읽는 순서), 출처와 라이선스, 학습 방법, 책의 장별 챕터 목차(완료된 챕터만 행으로 추가) | — |
| ⬜ | P9 | 환경 준비물 승인(5절). ch1~ch10은 추가 도구가 필요 없어 승인 없이 진행했습니다(외부 크레이트는 ch4의 `thiserror`·`anyhow`뿐). ch16 이후를 시작하기 전에 승인받습니다. 승인하지 않은 도구가 필요한 부분은 "설명만 쓰고 검증하지 못했다고 밝힌다"는 조건으로 위임합니다. | — |

루트 `README.md`의 "구성" 표에는 첫 챕터가 완료될 때 부록 행을 추가합니다.

P3을 마친 뒤에는 **챕터 상태의 기준이 `docs/roadmap.md`와 `PROGRESS.md`** 입니다. 이 문서에서는 3절(사전 작업)과 7절(실행 순서)의 상태만 갱신합니다.

## 4. 챕터 계획

표의 열은 로드맵 3절과 같아서 P3에서 그대로 옮길 수 있습니다. 기준 문서 절의 주소는 `https://effective-rust.com/{이름}.html`입니다. 표 아래의 작성 메모는 위임 메시지의 "추가 요청"에 넣을 내용입니다.

### 4.1 Types — ch1~ch9

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch1-express-data-structures` | 기본 타입과 암묵적 변환이 없는 이유, 집합 타입, 열거형으로 무효한 상태를 표현할 수 없게 만들기, `Option`/`Result`로 부재와 실패 표현 | `bool` 인자를 열거형으로, 필드 조합을 데이터를 가진 열거형으로, 센티널 값(-1, 빈 문자열)을 `Option`으로 | Item 1 (`use-types`) |
| ⬜ | `ch2-express-common-behavior` | 함수와 메서드, 함수 포인터, 클로저와 `Fn`/`FnMut`/`FnOnce`, 트레이트 바운드와 트레이트 객체, 매개변수로 무엇을 받을지 고르는 기준 | 함수 포인터와 클로저 받기 비교, 가장 느슨한 `Fn*` 바운드 고르기, 마커 트레이트, 트레이트 객체로 받기 | Item 2 (`use-types-2`) |
| ⬜ | `ch3-option-result-transforms` | `match` 대신 `if let`·`?`·변환 메서드, 에러 타입 맞추기(`map_err`, `From`), `Option`↔`Result`, `as_ref`, 변환 메서드 고르는 순서 | 명시적 `match`를 단계별로 변환 메서드로 바꾸기, `?`와 에러 변환, 참조를 유지하는 변환 | Item 3 (`transform`) |
| ⬜ | `ch4-idiomatic-errors` | `Error` 트레이트의 요구 사항, 최소 에러 타입, 중첩 에러 열거형, 트레이트 객체 에러의 득실, 라이브러리와 애플리케이션의 선택 기준 | `String` 에러의 한계와 뉴타입, 중첩 에러 열거형과 `From`, `Box<dyn Error>`, `thiserror`판, `anyhow`판 | Item 4 (`errors`) |
| ⬜ | `ch5-type-conversions` | 사용자 정의 변환(`From`/`Into`/`TryFrom`), `as` 캐스트의 위험, 컴파일러가 해 주는 강제 변환 | `From`을 구현하고 `Into` 바운드로 받기, `as`의 잘림과 `try_from`, 강제 변환이 일어나는 자리 모음 | Item 5 (`casts`) |
| ⬜ | `ch6-newtype-pattern` | 단위 혼동 막기, 의미가 드러나는 인자 타입, 고아 규칙 우회, 뉴타입의 한계(트레이트를 다시 구현해야 함) | 단위 뉴타입, 외부 타입에 외부 트레이트 구현, 위임 구현의 비용 | Item 6 (`newtype`) |
| ⬜ | `ch7-builders` | 필드가 많은 구조체의 초기화 문제, `Default`와 구조체 갱신 문법의 한계, 빌더 패턴(소비형과 `&mut`형), derive로 빌더 만들기 | 큰 구조체 직접 초기화, `Default` + `..`, 소비형 빌더, `&mut self` 빌더, `derive_builder`(선택) | Item 7 (`builders`) |
| ⬜ | `ch8-references-pointers` | 참조와 포인터의 차이, 포인터 트레이트(`Deref`, `AsRef`, `Borrow` 등), 넓은 포인터(슬라이스, 트레이트 객체), 스마트 포인터 고르기 | 포인터 크기 비교, 슬라이스를 받는 시그니처, vtable이 붙은 포인터, `AsRef`/`Borrow` 바운드, 스마트 포인터 선택표를 따라가는 예제 | Item 8 (`references`) |
| ⬜ | `ch9-iterator-transforms` | 반복자 트레이트, 변환과 소비 메서드, `Result` 값 모으기, 반복문을 반복자 체인으로 바꾸기, 명시적 반복문이 더 나은 경우 | `for` 반복문을 단계별로 체인으로, `collect::<Result<_, _>>()`, 조기 탈출·복잡한 상태에서 반복문 유지 | Item 9 (`iterators`) |

작성 메모

| 챕터 | 본편 연결(반복하지 않을 것) | 주의·준비물 |
| --- | --- | --- |
| ch1 | Part 0 ch3·ch5·ch6 | `compile_fail` 후보: 정수 타입 불일치(E0308), 빠진 `match` 갈래(E0004) |
| ch2 | Part 0 ch10·ch13·ch17 | 제네릭과 트레이트 객체의 득실 비교는 ch12로 미룸 |
| ch3 | Part 1 ch2(메서드 목록과 선택표) | 여기서는 리팩터링 과정과 판단 기준만. clippy `manual_map`·`question_mark` 등은 `PROGRESS.md` 작업 방식 메모 참고 |
| ch4 | Part 1 ch8(직접 구현, `source` 체인) | dev-dependency: `thiserror`, `anyhow`. `anyhow`의 CLI 활용은 Part 7 `ch2-implementation-errors` |
| ch5 | Part 1 ch6(변환 트레이트 구현법) | 강제 변환 규칙 전체는 Part 9 `ch7-type-system`, Part 10 `ch5-type-conversions`로 미룸 |
| ch6 | Part 0 ch19(뉴타입 소개) | `compile_fail` 후보: 고아 규칙(E0117) |
| ch7 | Part 0 ch5, Part 1 ch6(`Default`) | `derive_builder`는 선택. 넣지 않으면 README에서 소개만 |
| ch8 | Part 0 ch14, Part 1 ch7 | 용어는 "넓은 포인터"(용어 메모). 스마트 포인터 API 설명은 본편으로 링크 |
| ch9 | Part 0 ch13, Part 1 ch5(어댑터 목록) | clippy가 잡는 반복문 모양은 작업 방식 메모 참고 |

### 4.2 Traits — ch10~ch13

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch10-standard-traits` | 공개 타입에 구현할 표준 트레이트 고르기(`Clone`, `Copy`, `Default`, `PartialEq`/`Eq`, `PartialOrd`/`Ord`, `Hash`, `Debug`/`Display`), derive할 수 없거나 하면 안 되는 경우, 직접 구현할 때의 일관성 규칙, 연산자 오버로드 | derive 가능 조건, `Copy`를 붙이면 안 되는 타입, `Eq`와 `Hash`가 어긋날 때 `HashMap`의 오동작, 부동소수점과 `PartialEq`, 트레이트 체크리스트를 적용한 타입 | Item 10 (`std-traits`) |
| ⬜ | `ch11-drop-raii` | RAII와 가드 타입, `Drop`의 제약(`&mut self`, 실패를 돌려줄 수 없음), `drop(x)`로 일찍 해제하기 | 조기 반환에서 해제를 빠뜨리는 수동 관리와 가드 비교, 직접 만든 가드, drop 순서 | Item 11 (`raii`) |
| ⬜ | `ch12-generics-vs-trait-objects` | 단형성화와 동적 디스패치의 비용(코드 크기, 속도, 컴파일 시간), 여러 트레이트 바운드, dyn 호환, 이종 컬렉션, 선택 기준 | 같은 함수의 제네릭판과 `dyn`판, 이종 컬렉션, dyn 호환이 아닌 트레이트와 `where Self: Sized` | Item 12 (`generics`) |
| ⬜ | `ch13-default-implementations` | 필수 메서드는 적게, 기본 구현은 넉넉하게. 기본 구현 재정의, 기본 메서드 추가가 호환 변경인 이유 | 필수 메서드 하나에 기본 메서드 여럿인 트레이트, 재정의로 더 빠른 구현 제공 | Item 13 (`default-impl`) |

작성 메모

| 챕터 | 본편 연결(반복하지 않을 것) | 주의·준비물 |
| --- | --- | --- |
| ch10 | Part 1 ch6(트레이트별 구현법과 규칙) | 여기서는 "공개 타입에 무엇을 구현할지"의 판단. 관련 clippy 기본 수준은 작업 방식 메모 참고 |
| ch11 | Part 0 ch14(`Drop`), Part 1 ch11(잠금 가드) | `compile_fail` 후보: `drop` 메서드 직접 호출(E0040). 누수와 안전성은 Part 10 `ch7-obrm`으로 미룸 |
| ch12 | Part 0 ch10·ch17 | `compile_fail` 후보: E0038. 바이너리 크기 관찰은 실행 환경마다 다르므로 값을 지어내지 않음 |
| ch13 | Part 0 ch10, Part 1 ch5(`Iterator`의 기본 메서드) | 작은 Item. 호환성 이야기는 ch21과 겹치지 않게 한 문단으로 |

### 4.3 Concepts — ch14~ch20

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch14-lifetimes` | 스택과 라이프타임, 라이프타임의 범위(마지막 사용까지), 여러 라이프타임 매개변수, 생략 규칙, `'static`, 힙에 있는 값의 라이프타임, 자료구조 안의 라이프타임, 익명 라이프타임 `'_` | 스택 프레임과 댕글링 참조, 마지막 사용 뒤에 풀리는 대여, 라이프타임 매개변수가 둘 필요한 함수, `&'static`과 `T: 'static`의 차이, 참조를 담은 구조체, `'_` | Item 14 (`lifetimes`) |
| ⬜ | `ch15-borrow-checker` | 대여 규칙과 소유자가 할 수 있는 일, 대여 검사기 에러를 푸는 방법(지역 리팩터링, 자료구조 설계, 스마트 포인터), 자기 참조 자료구조 | 대여 범위 줄이기, `mem::take`/`replace`와 `Option::take`, 참조 대신 인덱스, `Rc<RefCell<T>>`, 자기 참조 시도와 대안 | Item 15 (`borrows`) |
| ⬜ | `ch16-avoid-unsafe` | unsafe를 쓰기 전에 표준 라이브러리와 검증된 크레이트에서 안전한 대안 찾기, 꼭 써야 할 때의 지침(작게 감싸기, 안전성 주석, Miri, 테스트) | 직접 쓴 unsafe를 표준 라이브러리의 안전한 API로 바꾸기, 안전한 래퍼와 `// SAFETY:` 주석 | Item 16 (`unsafe`) |
| ⬜ | `ch17-shared-state-parallelism` | Rust가 막는 것(데이터 경쟁, `Send`/`Sync`)과 막지 못하는 것(교착 상태), 잠금 범위와 순서, 공유 상태 대신 채널 | 데이터 경쟁 시도가 컴파일되지 않는 모습, 잠금 순서가 엇갈린 교착 상태와 해결, 채널로 다시 설계 | Item 17 (`deadlock`) |
| ⬜ | `ch18-dont-panic` | 패닉이 숨어 있는 곳, 라이브러리가 패닉 대신 `Result`를 돌려줘야 하는 이유, `catch_unwind`의 한계, 패닉을 막는 도구 | 패닉하는 API를 `Result` API로, 숨은 패닉 지점(인덱싱, 산술, `RefCell`), `unwrap`을 금지하는 lint 설정 | Item 18 (`panic`) |
| ⬜ | `ch19-avoid-reflection` | Rust에 리플렉션이 없는 이유, `type_name`과 `Any`/`TypeId`의 한계, 트레이트 객체 업캐스트 | `type_name`의 한계, `dyn Any` 다운캐스트와 대안(열거형, 트레이트 메서드), 업캐스트 | Item 19 (`reflection`) |
| ⬜ | `ch20-avoid-over-optimizing` | 먼저 단순하게 쓰고 측정한 뒤 최적화하기, 자료구조와 할당, 복사 비용을 두려워하지 않기, 참조 대신 소유 타입이나 스마트 포인터로 설계 단순화 | 라이프타임이 번지는 구조체와 소유 타입 구조체 비교, `clone`과 `Rc`, 간단한 실행 시간 측정 | Item 20 (`optimize`) |

작성 메모

| 챕터 | 본편 연결(반복하지 않을 것) | 주의·준비물 |
| --- | --- | --- |
| ch14 | Part 0 ch4·ch10 | 가장 큰 Item 중 하나. `compile_fail` 후보: E0597, E0515, E0106. 변성과 HRTB는 Part 10 `ch3-ownership-lifetimes`·`ch4-variance-dropck`로 미룸 |
| ch15 | Part 0 ch4·ch14, Part 1 ch7 | `compile_fail` 후보: E0499, E0502, E0505. 책이 언급하는 `Pin`은 Part 1 ch13으로 링크만 |
| ch16 | Part 0 ch19 | 책에 코드가 없는 Item이라 예제를 새로 설계. Miri는 nightly 필요(5절). 없으면 설명만 쓰고 미검증이라고 밝힘. unsafe 규칙 자체는 Part 10으로 미룸 |
| ch17 | Part 0 ch15, Part 1 ch11 | 교착 상태 예제는 `try_lock`이나 제한 시간으로 관찰하고 스스로 끝나야 함. 스레드 출력 순서는 작업 방식 메모대로 처리. C++ 비교는 줄임 |
| ch18 | Part 0 ch9, Part 1 ch8(`catch_unwind`, 패닉 훅) | `panic = "abort"`는 워크스페이스 멤버의 `[profile]`이 무시되므로 `projects/`에서만 실습(`PROGRESS.md` 후속 조치) |
| ch19 | Part 1 ch14(`Any`) | 업캐스트는 1.4절 참고 |
| ch20 | Part 1 ch6(`Cow`)·ch7 | 시간 측정값은 실행마다 다름. 릴리스 빌드에서 반복문이 통째로 사라질 수 있음(작업 방식 메모) |

### 4.4 Dependencies — ch21~ch26

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch21-semver` | 시맨틱 버저닝의 약속, 크레이트 작성자가 지킬 것(무엇이 비호환 변경인가), 사용자가 알 것(버전 요구 사항, `Cargo.lock`, MSRV) | 라이브러리와 사용자 크레이트 한 쌍으로 호환 변경과 미묘한 비호환 변경 재현(`projects/`) | Item 21 (`semver`) |
| ⬜ | `ch22-minimize-visibility` | 가시성 단계(`pub`, `pub(crate)`, `pub(super)`, `pub(in …)`), 공개한 것은 되돌리기 어려움, 비공개 필드와 접근자 | 모듈 안에서 가시성 단계 비교, 비공개 필드 덕분에 내부 표현을 바꿔도 사용자 코드가 그대로인 예 | Item 22 (`visibility`) |
| ⬜ | `ch23-avoid-wildcard-imports` | 와일드카드 가져오기가 의존성 업데이트에서 이름 충돌을 일으키는 원리, 허용되는 예외(프렐류드, 테스트의 `use super::*`) | 의존 크레이트에 이름이 추가되면 깨지는 사용자 코드(`projects/`), 예외 사례 | Item 23 (`wildcard`) |
| ⬜ | `ch24-re-export-dependencies` | API에 드러난 의존성 타입과 버전 불일치, `pub use`로 의존성 재내보내기 | 같은 크레이트의 두 버전이 섞여 타입이 맞지 않는 상황과 재내보내기로 해결(`projects/`) | Item 24 (`re-export`) |
| ⬜ | `ch25-dependency-graph` | 버전 지정 방식, 중복 버전, `cargo tree`와 `cargo update`, 의존성 점검 도구, 무엇에 의존할지 판단하는 기준 | `cargo tree -d`/`-i` 읽기, 중복 버전을 만들고 해소(`projects/`), 점검 도구 설정 | Item 25 (`dep-graph`) |
| ⬜ | `ch26-feature-creep` | `cfg` 조건부 컴파일, 기능은 더하기만 해야 하는 이유(기능 통합), 공개 필드·트레이트 메서드를 기능으로 가리면 안 되는 이유, 조합 수와 CI | 가산적이지 않은 기능이 통합으로 깨지는 재현, 기능으로 가린 필드가 사용자 코드를 깨는 예(`projects/`) | Item 26 (`features`) |

작성 메모

| 챕터 | 본편 연결(반복하지 않을 것) | 주의·준비물 |
| --- | --- | --- |
| ch21 | Part 3 `ch10-publishing-semver`(미작성, 링크 없이 이름만) | 비호환 변경은 path 의존성으로 재현해 네트워크 없이 검증. `cargo-semver-checks`는 선택(5절) |
| ch22 | Part 0 ch7 | `compile_fail` 후보: E0603, E0616. `#[non_exhaustive]` 상세는 Part 9 `ch4-attributes` |
| ch23 | Part 0 ch7 | 작은 Item. 충돌 시 실제 에러 코드는 실행해서 확인 |
| ch24 | ch21 | 두 버전은 `package = "…"` 이름 바꾸기와 로컬 크레이트 둘로 흉내 내 네트워크 없이 재현 |
| ch25 | Part 3 `ch2-dependencies`(미작성) | 책에 코드가 거의 없는 Item. `cargo-deny`, `cargo-udeps`(nightly 필요)는 5절. 없으면 `cargo tree` 중심으로 |
| ch26 | Part 3 `ch5-features`, Part 5 `ch5-conditional-compilation`(미작성) | `--no-default-features`, `--all-features` 양쪽으로 검증 |

### 4.5 Tooling — ch27~ch32

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch27-document-public-interfaces` | 문서 주석에 쓸 것과 쓰지 말 것, `# Panics`·`# Errors`·`# Safety` 절, 항목 링크, 크레이트·모듈 문서, 문서 누락을 잡는 lint | 같은 라이브러리의 부실한 문서판과 고친 판(`projects/`), `missing_docs`와 깨진 링크 검출 | Item 27 (`documentation`) |
| ⬜ | `ch28-macros-judiciously` | 선언적 매크로와 절차적 매크로 3종, 매크로가 필요한 경우와 함수·제네릭으로 충분한 경우, 매크로의 단점 | 반복 코드를 `macro_rules!`로, 인자가 여러 번 평가되는 함정, 제네릭 함수로 대체, 간단한 derive 매크로(`projects/`) | Item 28 (`macros`) |
| ⬜ | `ch29-listen-to-clippy` | Clippy 경고를 고치는 이유, lint 그룹, 예외를 이유와 함께 표시하기, 프로젝트 단위 설정 | 경고가 나는 코드와 고친 코드, `#[expect(…, reason = "…")]`, `[lints]` 설정(`projects/`) | Item 29 (`clippy`) |
| ⬜ | `ch30-beyond-unit-tests` | 단위·통합·문서 테스트의 역할, 예제 프로그램, 벤치마크, 퍼즈 테스트, 테스트 조언 | 라이브러리 하나에 테스트 종류를 모두 갖추기(`projects/`): 단위, 통합, 문서 테스트, example, 벤치마크, 퍼즈 타깃(선택) | Item 30 (`testing`) |
| ⬜ | `ch31-tooling-ecosystem` | Cargo 기본 도구, `cargo metadata`를 바탕으로 한 도구들, 책 전체에 나온 도구 지도 | `cargo metadata` 출력 읽기, `cargo tree`, 도구별 용도와 사용 챕터를 묶은 표 | Item 31 (`use-tools`) |
| ⬜ | `ch32-continuous-integration` | CI에 넣을 단계(빌드, 테스트, 서식, lint, 문서, 기능 조합, MSRV, 의존성 점검), 원칙(로컬에서 재현, 경고는 실패, 도구 버전 고정) | 로컬에서 같은 검사를 돌리는 스크립트와 GitHub Actions 워크플로 예시(`projects/`) | Item 32 (`ci`) |

작성 메모

| 챕터 | 본편 연결(반복하지 않을 것) | 주의·준비물 |
| --- | --- | --- |
| ch27 | Part 0 ch11(문서 테스트), Part 4(미작성) | rustdoc 기능 전반은 Part 4로 미룸. 여기서는 무엇을 쓸지의 판단 |
| ch28 | Part 0 ch19(`macro_rules!`, derive 매크로 프로젝트) | `syn`·`quote` 버전은 Part 0 ch19 프로젝트와 맞춤. `cargo-expand`는 nightly 필요(5절). 프래그먼트·위생 상세는 Part 9 `ch2-macros` |
| ch29 | Part 5 `ch3-lints`(미작성) | `PROGRESS.md` 작업 방식 메모의 clippy 목록을 소재로 활용. "clippy가 잡는다"는 속성을 지우고 확인한 것만 |
| ch30 | Part 0 ch11, Part 3 `ch9-testing-ci`(미작성) | dev-dependency: `criterion`. `cargo-fuzz`는 nightly 필요(5절). 벤치마크 수치는 실행마다 다름 |
| ch31 | Part 3 `ch11-commands-tools`(미작성) | 책에 코드가 없는 Item. 설치하지 않은 도구는 용도만 소개하고 실행 결과를 지어내지 않음 |
| ch32 | Part 3 `ch9-testing-ci`(미작성), 이 저장소의 검증 명령 | 워크플로 예시는 챕터 디렉터리 안에 두어 실제로 실행되지 않게 함(`.github/`에 넣지 않음) |

### 4.6 Beyond Standard Rust — ch33~ch35

| 상태 | 챕터 | 학습 주제 | 예제 계획 | 기준 문서 절 |
| --- | --- | --- | --- | --- |
| ⬜ | `ch33-no-std` | `core`·`alloc`·`std`의 구분, 라이브러리를 `no_std` 호환으로 쓰기, `std` 기능 패턴, 실패할 수 있는 할당 | `no_std` 라이브러리와 `std` 기능(`projects/`), 호스트에서 테스트, `try_reserve` | Item 33 (`no-std`) |
| ⬜ | `ch34-ffi-boundaries` | Rust에서 C 함수 부르기(링크, 이름 맹글링, 타입 대응), C 데이터 다루기(`repr(C)`), 경계에서의 소유권과 라이프타임, C에서 Rust 부르기, 안전한 래퍼 | `build.rs`로 C 코드를 컴파일해 호출, `repr(C)` 구조체 주고받기, 불투명 포인터와 생성·해제 함수 쌍, C에서 부르는 Rust 함수(`projects/`) | Item 34 (`ffi`) |
| ⬜ | `ch35-bindgen` | 손으로 쓴 선언이 어긋날 때의 위험, 헤더에서 선언 생성하기, `-sys` 크레이트 관례, C++ 연동 도구 소개 | ch34의 C 라이브러리를 손으로 쓴 선언과 생성한 선언으로 비교(`projects/`) | Item 35 (`bindgen`) |

작성 메모

| 챕터 | 본편 연결(반복하지 않을 것) | 주의·준비물 |
| --- | --- | --- |
| ch33 | Part 1 ch1(`core`/`alloc`/`std`), Part 8 `ch1-intro-no-std`(미작성) | 실제 `no_std` 타깃 빌드 검증은 `thumbv7m-none-eabi` 타깃 필요(5절). 없으면 호스트 빌드만 하고 한계를 밝힘 |
| ch34 | Part 1 ch14(`CString`/`CStr`, C 함수 호출) | 큰 Item. build-dependency `cc`, C 컴파일러는 설치돼 있음. `PROGRESS.md` 후속 조치의 Part 10 `ch12-ffi-beneath-std` 인계 항목과 겹치지 않게 범위를 나눔 |
| ch35 | ch34(프로젝트를 복사해 이어 씀) | **ch34가 검증된 뒤 시작**. `bindgen`은 libclang이 필요. 빌드 스크립트 방식과 CLI 방식 중 검증 가능한 쪽으로 |

## 5. 준비물

**외부 크레이트** (챕터 `Cargo.toml`이나 `projects/`에 추가, 처음 빌드할 때 네트워크 필요)

| 챕터 | 크레이트 |
| --- | --- |
| ch4 | `thiserror`, `anyhow` |
| ch7 | `derive_builder` (선택) |
| ch28 | `syn`, `quote`, `proc-macro2` |
| ch30 | `criterion` |
| ch34 | `cc` |
| ch35 | `bindgen` |

**사용자 환경을 바꾸는 도구** (P9에서 승인받은 것만 설치합니다. 2026-11-19 현재 아래는 모두 설치돼 있지 않습니다.)

| 도구 | 쓰는 챕터 | 없을 때 |
| --- | --- | --- |
| nightly 툴체인 + Miri | ch16 | Miri 사용법은 설명만, 미검증 표시 |
| `cargo-fuzz` (nightly 필요) | ch30 | 퍼즈 타깃은 설명만 |
| `cargo-expand` (nightly 필요) | ch28 | 매크로 확장 결과는 싣지 않음 |
| `cargo-deny` | ch25, ch32 | 설정 파일 예시만 |
| `cargo-udeps` (nightly 필요) 또는 `cargo-machete` | ch25 | 소개만 |
| `cargo-semver-checks` | ch21 | 소개만 |
| `thumbv7m-none-eabi` 타깃 | ch33 | 호스트 빌드만 |
| libclang (`bindgen`용) | ch35 | Xcode 명령줄 도구의 것을 찾는지 먼저 확인 |

nightly와 Miri는 Part 10·11의 준비물이기도 하고, `PROGRESS.md` 후속 조치에 Miri로 다시 확인할 예제가 쌓여 있습니다. 이번에 설치하면 그 숙제도 함께 처리할 수 있습니다.

## 6. 위임 메시지에 넣을 공통 요청

`/writing-guide`의 위임 메시지 뼈대에 더해 모든 부록 챕터에 넣습니다. `tutor.md`는 독자를 "Rust를 처음 배우는 사람"으로 가정하므로 독자 항목은 꼭 넣습니다.

```
## 부록 공통
- 책이 권하는 것: (위임하기 전에 메인 세션이 책의 해당 Item 페이지에서 확인한 지침 요약을 여기에 적습니다.
  더해도 되는 판단은 따로 적고 "책에 없는 지침"으로 표시하게 합니다. 책의 예제에 나오는 이름과 시나리오도 적어 피하게 합니다.)
- 독자: Part 0과 Part 1을 마친 사람입니다. 문법과 API 사용법은 다시 설명하지 말고
  해당 본편 챕터를 가리킵니다. 이 챕터는 "여러 방법 중 무엇을 왜 고르는가"를 다룹니다.
- 저작권: 기준 문서는 CC BY-NC-ND 4.0입니다. 본문을 번역·발췌하지 않고 책의 코드도 옮기지
  않습니다. 설명과 예제는 새로 설계합니다. Item 원제는 영어 그대로 인용하고 링크합니다.
- README: 첫머리에 원제와 링크, 패키지 이름. 핵심 개념은 지침마다
  "문제 상황 → 비권장 코드 → 권장 코드 → 따르지 않아도 되는 경우" 순서.
  핵심 개념 끝에 "기억할 것" 3~6줄. 책과 실제 동작이 다르면 "책과 달라진 점" 소절.
- 출처 표기: 책의 Item 본문이 명시적으로 권하지 않는데 호출 메시지·std 문서·clippy를 근거로 더한 지침이 있으면
  "책과 달라진 점"에 "책에 없는 지침" 항목으로 어느 지침이 그런지와 근거를 밝히고 보고에도 적습니다.
- 예제: 파일 하나가 지침 하나. 같은 파일에서 비권장 형태와 권장 형태를 함께 실행해 비교합니다.
  컴파일되지 않는 비권장 코드만 compile_fail/에 둡니다.
- 버전: 책은 2018 에디션, rustc 1.70 기준입니다. 저장소 툴체인에서 확인한 것만 쓰고,
  책과 다른 점은 보고의 "레퍼런스와 실제 동작의 차이"에 적습니다.
- 참조 표기: 부록 챕터는 ch14(Item 14), 본편 챕터는 Part 1 ch8 형식.
  아직 없는 챕터는 링크 없이 코드 서식의 이름으로만 적습니다.
- 다른 언어(C++, Java, Go)와의 비교는 이해에 꼭 필요할 때만 씁니다. 비동기는 다루지 않습니다.
```

챕터별 추가 요청은 4절 작성 메모와 1.4절 표에서 해당 줄을 옮깁니다.

## 7. 실행 순서

묶음은 책의 장을 따릅니다. 묶음 안에서는 스킬 규칙대로 최대 4개를 동시에 띄우고 번호 순서로 빈자리를 채웁니다.

| 상태 | 묶음 | 범위 | 챕터 수 | 실행 | 비고 |
| --- | --- | --- | --- | --- | --- |
| ✅ | 0 | 사전 작업 P1~P8 | — | 메인 세션이 직접 | P9(환경 준비물 승인)는 묶음 3 전에 |
| ✅ | 1 | Types | 9 | `/writing-guide appd-a ch1~ch9` | 추가 도구 없이 가능 |
| ✅ | 2 | Traits | 4 | `… ch10~ch13` | 추가 도구 없이 가능 |
| ✅ | 3 | Concepts | 7 | `… ch14~ch20` | 2026-11-19 완료. nightly를 설치하지 않아 ch16의 unsafe 코드는 Miri로 확인하지 못함(`PROGRESS.md` 후속 조치) |
| ✅ | 4 | Dependencies | 6 | `… ch21~ch26` | 2026-11-19 완료. 추가 도구(`cargo-semver-checks`, `cargo-deny` 등)는 설치하지 않고 소개만 함(`PROGRESS.md` 후속 조치) |
| ⬜ | 5 | Tooling | 6 | `… ch27~ch32` | ch31은 ch27~ch30 뒤에, ch32는 마지막에(앞 챕터의 도구를 모음) |
| ⬜ | 6 | Beyond Standard Rust | 3 | `… ch33~ch35` | ch35는 ch34 검증 뒤에 |
| ⬜ | 7 | 마무리 | — | 메인 세션이 직접 | 8절 |

- 묶음 1~3은 Part 0·1만 전제로 하므로 사전 작업이 끝나면 바로 시작할 수 있습니다.
- 묶음 4~6은 아직 쓰지 않은 Part 3~5, 8, 10과 주제가 겹칩니다. 부록 챕터는 그 파트 없이도 읽을 수 있게 쓰고, 본편 쪽 챕터는 이름만 적습니다. 본편을 먼저 쓰게 되면 그때 링크로 바꿉니다.
- 묶음 사이에서 멈춰도 됩니다. 묶음 하나가 끝날 때마다 `cargo` 전체 검증과 사이트 빌드를 한 번 돌립니다.

## 8. 검증과 마무리

챕터별 검증은 `/writing-guide` 6절 그대로입니다(패키지 이름만 `er-ch…`). 부록에서 추가로 봅니다.

- README에 책 본문을 옮긴 문단이나 책의 코드와 같은 예제가 없는지 훑어봅니다.
- "책과 달라진 점"에 적힌 내용은 메인 세션이 한 번 더 실행해 확인합니다.
- `projects/` 실습이 많은 묶음 4~6은 프로젝트마다 `cargo clean`까지 확인하고, 루트 `Cargo.toml`이 바뀌지 않았는지 봅니다.

범위를 모두 마친 뒤:

- `cargo fmt --all --check`, `cargo build --workspace --examples`, `cargo clippy --workspace --examples -- -D warnings`, `(cd web && pnpm build)`
- 부록 README의 목차가 35행인지, 로드맵과 `PROGRESS.md`의 숫자가 맞는지 확인합니다.
- 부록 챕터끼리 링크 없이 이름만 적은 곳을 상대 링크로 바꿉니다(35개가 모두 생긴 뒤라 가능).
- `PROGRESS.md` "후속 조치"에 남깁니다: 본편 Part 3~5, 8, 10을 쓸 때 겹치는 부록 챕터(4절 작성 메모)를 서로 링크할 것.
- 새로 정한 용어(지침, 관용구 등)를 "용어 메모"에 옮깁니다.

## 9. 정해 둔 것과 정해야 할 것

이 계획이 기본값으로 정한 것입니다. 바꾸려면 사전 작업 전에 이 문서를 고칩니다.

| 항목 | 정한 값 | 다른 선택지 |
| --- | --- | --- |
| 챕터 단위 | Item 1개 = 챕터 1개(35개), 챕터 번호 = Item 번호 | 작은 Item을 합쳐 25개 안팎으로 줄이기(번호 대응이 깨짐) |
| 디렉터리와 패키지 | `appd-a/`, `er-ch{N}-{slug}` | `part12-…`로 본편에 편입(요청과 다름) |
| 진척 집계 | 본편 109와 별도로 `부록 0 / 35` | 합계에 포함해 144 |
| 분량 | 예제 4~6개, README 300~700줄 | 본편 수준(예제 8개 안팎, 600~1200줄) |

실행 전에 사용자가 정해야 하는 것은 두 가지입니다.

1. **환경 준비물 승인**(5절): 특히 nightly 툴체인. 승인하지 않아도 진행할 수 있지만 ch16, ch25, ch28, ch30의 일부가 "설명만"으로 남습니다.
2. **착수 시점**: Part 2를 마친 뒤 시작할지, 본편 사이 어느 지점에 끼워 넣을지. 묶음 1~3은 지금도 선수 과정이 충족됩니다.
