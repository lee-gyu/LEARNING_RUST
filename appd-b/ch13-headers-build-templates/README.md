# ch13. 헤더·빌드·템플릿과 Rust

C와 C++ 개발자가 Rust 프로젝트를 처음 열면 빠진 것이 눈에 띕니다. 헤더 파일이 없고, `#include`와 `#ifdef`가 없고, Makefile이나 `CMakeLists.txt`도 없습니다. 이 챕터는 C/C++에서 소스 바깥의 도구, 즉 전처리기(preprocessor), 빌드 시스템, 링커, 패키지 관리자가 하던 일을 Rust에서는 무엇이 맡는지, 템플릿과 제네릭은 무엇이 다른지 답합니다. 문법은 다시 가르치지 않고 본편과 부록 A를 가리킵니다.

- 패키지: `faq-ch13-headers-build-templates`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 비교 코드를 확인한 컴파일러: Apple clang 21.0.0(C17, C++20), macOS(aarch64)
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q13.1](#q131-헤더-파일이-없나요-선언과-정의를-나누지-않나요) | 헤더 파일이 없나요? 선언과 정의를 나누지 않나요? | 없습니다. 크레이트 전체가 한 번에 컴파일되므로 함수·타입·모듈은 적은 순서와 상관없이 서로를 가리키고, 다른 크레이트에는 컴파일러가 만든 메타데이터로 인터페이스가 전달됩니다. |
| [Q13.2](#q132-include-define-ifdef는-무엇으로-대신하나요) | `#include`, `#define`, `#ifdef`는 무엇으로 대신하나요? | `mod`와 `use`, `const`와 함수(필요하면 `macro_rules!`), `#[cfg]`와 `cfg!`가 맡습니다. 컴파일 전에 토큰을 바꿔 끼우는 전처리 단계가 따로 없습니다. |
| [Q13.3](#q133-rust의-매크로는-c의-매크로와-무엇이-다른가요) | Rust의 매크로는 C의 매크로와 무엇이 다른가요? | 인자를 `expr` 같은 문법 단위로 받아 우선순위가 지켜지고, 지역 변수에 위생(hygiene) 규칙이 적용되며, 펼친 결과가 보통 코드처럼 타입 검사됩니다. 다만 인자를 두 번 쓰면 두 번 평가되는 것은 같습니다. |
| [Q13.4](#q134-make나-cmake-별도의-패키지-관리자-없이-어떻게-빌드하나요) | Make나 CMake, 별도의 패키지 관리자 없이 어떻게 빌드하나요? | Cargo가 빌드, 의존성 내려받기와 버전 고정, 테스트, 문서를 한 도구로 맡습니다. C 코드를 함께 빌드할 때는 빌드 스크립트를 씁니다. |
| [Q13.5](#q135-템플릿과-제네릭은-무엇이-다른가요) | 템플릿과 제네릭은 무엇이 다른가요? | 제네릭 본문은 정의할 때 트레이트 바운드만으로 검사되어 에러가 일찍, 짧게 납니다. 대신 특수화(specialization)와 가변 인자 제네릭이 없고 const 제네릭(const generics)으로 계산한 식을 쓰지 못하는 등 표현력이 좁습니다. |
| [Q13.6](#q136-odr-위반-링크-순서-같은-문제는-어떻게-되나요) | ODR 위반, 링크 순서 같은 문제는 어떻게 되나요? | 항목은 크레이트와 모듈 경로로 구별되어 같은 이름이 부딪치지 않고, 링크는 Cargo가 정합니다. 심볼 이름을 직접 정하는 `#[unsafe(no_mangle)]`·`export_name`에서는 충돌이 생길 수 있는데, 같은 크레이트 안이면 컴파일 에러, 크레이트 사이면 링커 경고만 나고 한쪽 정의가 쓰입니다(macOS에서 확인). |
| [Q13.7](#q137-네임스페이스-friend-익명-네임스페이스와-static-함수에-해당하는-것은-무엇인가요) | 네임스페이스, `friend`, 익명 네임스페이스와 `static` 함수에 해당하는 것은 무엇인가요? | 모듈이 네임스페이스이고, 비공개가 기본이라 익명 네임스페이스·`static` 함수가 따로 필요 없으며, 같은 모듈 안에서는 비공개 필드를 볼 수 있어 `friend` 자리를 모듈이 맡습니다. |
| [Q13.8](#q138-컴파일-단위는-무엇인가요) | 컴파일 단위는 무엇인가요? | 크레이트입니다. 소스 파일마다 따로 컴파일되는 C++의 번역 단위(translation unit)와 달리, 크레이트 안에서는 컴파일러가 코드를 코드 생성 단위로 나눠 병렬로 처리합니다. |
| [Q13.9](#q139-테스트-문서-포매터-린터를-따로-골라야-하나요) | 테스트, 문서, 포매터, 린터를 따로 골라야 하나요? | 기본은 고르지 않아도 됩니다. `cargo test`, `cargo doc`, `cargo fmt`, `cargo clippy`가 공식 도구이고, 더 필요한 것만 크레이트나 도구로 더합니다. |
| [Q13.10](#q1310-플랫폼별-코드와-조건부-컴파일은-어떻게-하나요) | 플랫폼별 코드와 조건부 컴파일은 어떻게 하나요? | `#[cfg(target_os = "linux")]` 같은 속성과 `cfg!` 매크로, `Cargo.toml`의 플랫폼별 의존성과 기능으로 합니다. Cargo가 알려 준 이름이 아니면 경고하므로 오타가 드러납니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md): 패키지, 크레이트, 모듈, `use`, 가시성
- [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md): 제네릭, 트레이트 바운드
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.18~2.20절: 매크로
- [ch6](../ch6-no-classes-inheritance/README.md) Q6.6·Q6.7·Q6.10: 제네릭과 C++ 템플릿, 가변 인자, 가시성

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 헤더, 전방 선언(forward declaration), 번역 단위가 Rust에서 왜 필요 없는지 크레이트라는 컴파일 단위로 설명할 수 있다.
2. C 전처리기의 `#include`·`#define`·`#ifdef`와 함수형 매크로가 일으키는 문제(중복 평가, 이름 가로채기, 오타)를 예로 들고, Rust에서 무엇이 그 자리를 맡는지 말할 수 있다.
3. 템플릿과 제네릭의 차이(검사 시점, 에러 위치, concept과 트레이트 바운드)를 설명하고, const 제네릭으로 C++의 값 템플릿 매개변수를 옮기며 그 한계를 말할 수 있다.
4. ODR, 링크, 네임스페이스, 빌드·테스트·포매터 도구가 Rust에서 어떻게 정해져 있는지 C/C++와 비교해 설명할 수 있다.

## 2. 핵심 개념

### Q13.1 헤더 파일이 없나요? 선언과 정의를 나누지 않나요?

**짧은 답** 없습니다. Rust의 컴파일 단위는 크레이트이고, 컴파일러는 크레이트 하나를 이루는 모든 파일을 한 번에 봅니다. 그래서 함수, 타입, 모듈은 적은 순서와 상관없이 서로를 가리킬 수 있고 전방 선언이 필요 없습니다. 다른 크레이트가 쓸 인터페이스는 사람이 헤더로 따로 적지 않고, 컴파일러가 컴파일한 크레이트에 메타데이터로 담아 넘깁니다. 대가로 크레이트 하나가 커지면 그만큼 한 번에 다시 컴파일할 양이 커집니다.

**왜 그런가** C와 C++는 소스 파일 하나(와 `#include`한 헤더)를 따로 컴파일하므로, 다른 파일의 함수를 부르려면 그 선언을 먼저 보아야 합니다. 그 선언을 모아 둔 것이 헤더입니다. Rust Reference는 크레이트를 컴파일과 링크의 단위라고 정하고, 컴파일 한 번이 크레이트 하나를 처리해 실행 파일이나 라이브러리 하나를 만든다고 설명합니다. [01_no_forward_declarations](examples/01_no_forward_declarations.rs)는 아래에 정의한 함수, 서로를 부르는 함수, 아직 정의하지 않은 타입, 아래에 적은 모듈을 먼저 씁니다. 첫째는 Part 0 ch3 예제 07에서 이미 봤고, 나머지가 이 예제가 더하는 것입니다.

```text
1. 아래에 정의한 함수를 위에서 부름
  안녕, 헤더
2. 서로를 부르는 함수
  is_even(10) = true, is_odd(7) = true
3. 아직 정의하지 않은 타입을 필드로 씀
  개발: 1명, 첫 사람 민지
  첫 사람의 소속 있음? false
4. 아래에 정의한 모듈의 함수
  넓이 = 12
```

다른 파일에 둔 모듈은 `mod 이름;` 한 줄로 크레이트에 넣고 경로로 가리킵니다(Part 0 ch7). 공개할 것과 숨길 것은 헤더에 무엇을 적느냐가 아니라 `pub`으로 정합니다.

**다른 언어에서는** C++20은 모듈을 표준에 넣었습니다(표준 초안 [module.unit]: 모듈 선언을 담은 번역 단위). 헤더 대신 `import`로 인터페이스를 가져오는 방식이라 방향은 Rust와 비슷하지만, 기존 헤더 기반 코드와 함께 쓰이므로 헤더가 사라지지는 않았습니다.

**오해와 한계** 순서와 상관없다는 규칙에는 예외가 있습니다. `macro_rules!` 매크로를 이름만으로 부를 때는 정의한 줄보다 아래에서만 쓸 수 있어, 위에서 부르면 `` cannot find macro `hello` in this scope ``와 함께 정의를 앞으로 옮기라는 안내가 나옵니다(이 환경에서 확인). Q13.2가 `#define` 대신 `macro_rules!`를 쓸 수 있다고 했으므로 실제로 부딪히는 차이입니다. 그리고 헤더가 없다고 인터페이스와 구현을 나눌 수 없는 것은 아닙니다. 공개 API는 `pub` 항목과 그 문서(`cargo doc`)가 보여 주고, 구현을 감추는 것은 비공개 항목이 맡습니다([부록 A ch22](../../appd-a/ch22-minimize-visibility/README.md)). 다만 크레이트 하나가 한 번에 컴파일되므로, 큰 프로젝트는 크레이트를 나눠 다시 컴파일할 범위를 줄입니다. 컴파일 시간 문제는 [ch15](../ch15-pain-points/README.md)(Q15.6)에서 다룹니다.

**더 읽기** Part 0 ch7 / Q13.8 / 부록 A ch22

### Q13.2 `#include`, `#define`, `#ifdef`는 무엇으로 대신하나요?

**짧은 답** 컴파일 전에 토큰을 바꿔 끼우는 전처리 단계가 따로 없습니다. `#include`는 `mod`(파일을 크레이트에 넣기)와 `use`(이름 가져오기)가, 값을 정의하는 `#define`은 `const`·`static`이, 함수처럼 쓰는 `#define`은 함수나 제네릭(정 필요하면 `macro_rules!`)이, `#ifdef`는 `#[cfg]` 속성과 `cfg!` 매크로가 맡습니다. 인클루드 가드나 `#pragma once`도 필요 없습니다.

**왜 그런가** 전처리기는 컴파일러가 코드를 해석하기 전에 전처리 토큰을 바꿔 끼웁니다. 그래서 매크로는 타입도 범위도 모르고, 헤더를 두 번 넣으면 같은 정의가 두 번 생깁니다. Rust에서는 모든 대체물이 언어의 일부입니다. `const`는 타입이 있는 값이고, `use`는 이름을 가져올 뿐 내용을 복사하지 않으며, `#[cfg]`는 조건이 거짓인 항목을 컴파일에서 빼되 그 판단에 쓸 이름과 값을 Cargo와 컴파일러가 압니다(Q13.10).

| C/C++ | Rust | 더 읽기 |
| --- | --- | --- |
| `#include "x.h"` | `mod x;`(파일을 크레이트에 넣음), `use x::Item;`(이름을 가져옴) | Part 0 ch7 |
| 생성한 코드를 글자 그대로 넣는 `#include` | `include!("파일")` | Part 3 ch8 |
| `#define LIMIT 10` | `const LIMIT: u32 = 10;` | [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md) |
| `#define MAX(a, b) …` | 함수(`std::cmp::max`), 제네릭 함수, `macro_rules!` | Q13.3 |
| `#ifdef`, `#if defined(...)` | `#[cfg(...)]`, `cfg!(...)`, `#[cfg_attr(...)]` | Q13.10, 부록 A ch26 |
| `#if` / `#elif` / `#else` | `cfg_select!`(처음 참인 갈래만 남김) | 부록 A ch26 2.2절 |
| 인클루드 가드, `#pragma once` | 필요 없음 | |

**다른 언어에서는** C++도 `constexpr` 변수와 함수, 템플릿, 모듈로 전처리기에 기대는 일을 줄여 왔습니다. 다만 조건부 컴파일(conditional compilation)과 헤더 포함은 여전히 전처리기가 맡습니다.

**오해와 한계** `#[cfg]`로 뺀 코드는 컴파일되지 않으므로, 다른 플랫폼에서만 켜지는 코드의 타입 에러는 그 플랫폼에서 빌드하기 전까지 드러나지 않습니다. 이 점은 `#ifdef`와 같습니다. 반대로 `cfg!`는 양쪽 갈래를 모두 컴파일하므로 이 문제가 없지만, 그 플랫폼에 없는 API는 쓸 수 없습니다([부록 A ch26](../../appd-a/ch26-feature-creep/README.md) 2.2절).

**더 읽기** Part 0 ch7 / 부록 A ch26 / Q13.10

### Q13.3 Rust의 매크로는 C의 매크로와 무엇이 다른가요?

**짧은 답** C 매크로는 인자의 전처리 토큰을 문법 구조와 상관없이 본문의 자리에 그대로 바꿔 끼웁니다(그래서 괄호가 필요합니다). Rust 매크로는 토큰 트리를 받되 `$a:expr`처럼 문법 단위로 받으므로 묶음이 유지되고, 펼친 결과가 보통의 코드처럼 해석·타입 검사됩니다. `macro_rules!`는 지역 변수에 대해 위생적이라 매크로 안의 변수가 부르는 쪽의 같은 이름과 섞이지 않습니다. 다만 본문에 인자를 두 번 쓰면 두 번 평가되는 것은 Rust 매크로도 같으므로, 값이 필요한 곳에서는 매크로보다 함수를 씁니다.

**왜 그런가** 먼저 우선순위입니다. 이 환경에서 `#define DOUBLE(a) a * 2`의 `DOUBLE(1 + 2)`는 `1 + 2 * 2`로 펼쳐져 5였고, 같은 모양의 `macro_rules! double { ($a:expr) => { $a * 2 } }`의 `double!(1 + 2)`는 6이었습니다. 한편 C 전처리기도 글자가 아니라 토큰 단위로 바꾸므로, `#define A 1`이 식별자 `ABC`나 문자열 `"A"` 안의 `A`를 바꾸지는 않습니다. C 매크로의 대표적인 문제 둘을 [compare/](compare/)에서 실행했습니다. 둘 다 미정의 동작이 없어 출력이 정해져 있습니다.

- [01_macro_double_eval.c](compare/01_macro_double_eval.c): `MAX(i++, j)`가 `((i++) > (j) ? (i++) : (j))`로 펼쳐져 `i++`가 두 번 실행됩니다. 출력은 `m = 6, i = 7`이고, `-Wall -Wextra`로도 경고가 없었습니다.
- [02_macro_name_capture.c](compare/02_macro_name_capture.c): 반복 횟수를 세는 매크로의 지역 변수 `count`가, 부르는 쪽이 넘긴 같은 이름의 변수를 가로챕니다. 7을 두 번 찍어야 할 자리에서 `count: 1`, `count: 0`을 찍었습니다. 컴파일러의 단서는 `-Wall`이 낸 ``unused variable 'count' [-Wunused-variable]`` 하나였습니다.

Rust의 `macro_rules!`에서는 이런 가로채기가 일어나지 않습니다(연습 3에서 직접 확인합니다). The Reference는 `macro_rules!`의 위생을 "혼합 위생(mixed-site hygiene)"이라 부르며, 지역 변수와 레이블은 정의한 곳에서, 함수·타입 같은 나머지 이름은 부른 곳에서 찾습니다. 이 규칙은 [부록 A ch28](../../appd-a/ch28-macros-judiciously/README.md) 2.1절에, 매크로 안의 변수가 부르는 쪽의 같은 이름 변수와 섞이지 않는 예는 2.4절(예제 04)에 있습니다.

**다른 언어에서는** C++ Core Guidelines ES.31은 상수나 "함수"에 매크로를 쓰지 말라고 하고, C++에서는 그 자리를 `constexpr` 변수, `inline`·`constexpr` 함수, 템플릿이 맡습니다. C에서는 `static inline` 함수가 같은 역할을 합니다. Rust의 선택도 같은 방향입니다. `max`는 매크로가 아니라 함수(`std::cmp::max`, `Ord::max`)라서 인자를 한 번만 평가합니다.

**오해와 한계** 위생은 지역 변수에만 적용되므로, 매크로 본문이 가리키는 함수·타입 이름은 부른 곳에서 찾습니다(부록 A ch28 4.2절). 그리고 `macro_rules!` 본문에 `$x`를 두 번 쓰면 C의 `MAX`와 똑같이 두 번 평가됩니다(부록 A ch28 예제 04). 매크로는 함수로 안 되는 일(가변 개수의 인자, 새 문법, 항목 생성)에만 쓰는 편이 좋습니다.

**더 읽기** Part 0 ch19 2.18~2.20절 / 부록 A ch28 / Part 9 `ch2-macros`

### Q13.4 Make나 CMake, 별도의 패키지 관리자 없이 어떻게 빌드하나요?

**짧은 답** Cargo가 합니다. `Cargo.toml`에 패키지와 의존성을 적으면 `cargo build`가 의존성을 내려받고, 버전을 `Cargo.lock`에 고정하고, 크레이트마다 올바른 순서로 `rustc`를 부릅니다. 테스트(`cargo test`), 문서(`cargo doc`), 벤치마크와 게시도 같은 도구입니다. C 라이브러리를 함께 빌드하거나 링크할 때는 빌드 스크립트(`build.rs`)로 Cargo에 알립니다.

**왜 그런가** C/C++에서는 소스 파일마다 컴파일 명령을 만들고, 헤더 경로와 라이브러리 경로를 적고, 링크 순서를 맞추는 일을 빌드 시스템이 맡고, 외부 라이브러리를 구하는 일은 다시 별도의 도구가 맡습니다. Rust에서는 크레이트가 컴파일 단위이자 배포 단위라서, Cargo는 크레이트 사이의 의존 관계만 알면 무엇을 어떤 순서로 컴파일할지 정할 수 있습니다. Cargo의 각 기능은 Part 3에서 실습했습니다.

| 하는 일 | Cargo | 더 읽기 |
| --- | --- | --- |
| 프로젝트 만들기와 빌드 | `cargo new`, `cargo build`, `cargo run` | [Part 3 ch1](../../part3-cargo/ch1-getting-started/README.md) |
| 의존성 내려받기와 버전 고정 | `[dependencies]`, `Cargo.lock` | [Part 3 ch2](../../part3-cargo/ch2-dependencies/README.md) |
| 여러 패키지를 한 저장소에서 | 워크스페이스 | [Part 3 ch4](../../part3-cargo/ch4-workspaces/README.md) |
| 선택 기능 | `[features]` | [Part 3 ch5](../../part3-cargo/ch5-features/README.md) |
| 최적화 수준 등 빌드 설정 | 프로필 | [Part 3 ch6](../../part3-cargo/ch6-profiles-build-performance/README.md) |
| C 코드 빌드, 시스템 라이브러리 링크 | `build.rs`(`cargo::rustc-link-lib` 등) | [Part 3 ch8](../../part3-cargo/ch8-build-scripts/README.md) |

**다른 언어에서는** C와 C++의 표준은 언어와 표준 라이브러리를 정하고, 빌드와 패키지 관리는 표준 밖의 도구(Make, CMake, 여러 패키지 관리자)가 맡습니다. 그래서 프로젝트마다 도구 조합이 다르고, 그 대신 프로젝트에 맞는 도구를 고를 수 있습니다.

**오해와 한계** Cargo가 모든 빌드를 대신하지는 않습니다. C/C++와 섞인 큰 프로젝트, 여러 언어를 함께 빌드하는 저장소에서는 Cargo를 다른 빌드 시스템 안에서 부르거나, 빌드 스크립트가 복잡해집니다. 기존 C/C++ 빌드와 함께 쓰는 문제는 [ch17](../ch17-interop-migration/README.md) Q17.9에서, 생태계와 공급망은 [ch16](../ch16-tooling-ecosystem/README.md)에서 다룹니다.

**더 읽기** Part 3 ch1~ch8 / [부록 A ch25](../../appd-a/ch25-dependency-graph/README.md)

### Q13.5 템플릿과 제네릭은 무엇이 다른가요?

**짧은 답** 둘 다 쓰인 타입마다 코드를 따로 만들지만(단형성화, monomorphization), 검사하는 때가 다릅니다. C++ 템플릿은 본문 가운데 `T`에 기대는 부분을 쓰는 자리에서 실제 타입으로 검사하고, Rust 제네릭은 정의할 때 트레이트 바운드만으로 본문을 검사합니다. 그래서 Rust의 에러는 일찍, 정의나 호출 한 곳을 가리키며 납니다. 대가로 표현력이 좁습니다. 특수화가 없고, const 제네릭으로 계산한 식(`N + 1`)을 타입에 쓰지 못하며, 가변 인자 제네릭이 없습니다.

**왜 그런가** 이 질문의 기본 비교(단형성화, 바운드를 적어야 하는 이유, 제약을 만족시키는 방식)는 ch6 Q6.6에 있습니다. 여기서는 에러가 나는 자리를 [compare/03](compare/03_template_checked_late.cpp)으로 봅니다.

- 제약 없는 템플릿 `total_size`는 `std::string`으로만 쓰면 문제없이 컴파일되고, `int`로 쓰는 줄을 더해야(`-DUSE_INT`) 에러가 납니다. 에러는 템플릿 본문의 `t.size()` 줄을 가리키고(``member reference base type 'const int' is not a structure or union``), 쓴 자리는 `note: in instantiation of function template specialization 'total_size<int>' requested here`로 따라 나옵니다.
- C++20 concept `HasSize`로 제약을 적은 `total_size_checked`는 부른 줄에서 `no matching function for call to 'total_size_checked'`로 거부되고, `constraints not satisfied`, `'int' does not satisfy 'HasSize'`가 이유를 말합니다.

Rust는 기본으로 두 번째 방식입니다. 바운드 없는 `T`로 연산자나 메서드를 쓰면 아무도 부르지 않아도 정의에서 에러이고(연산자는 Part 0 ch10 4.1절의 E0369, 메서드는 E0599), 바운드를 못 채운 타입으로 부르면 부른 자리에서 E0277입니다(Part 0 ch10 4.2절). 예외는 const 평가입니다. 인라인 const 블록(`const { … }`, 블록 안을 컴파일 시간에 평가하는 식. Part 9 `ch10-unsafety-const-asm`에서 다룸)의 `assert!`처럼 const 매개변수에 기대는 검사는 단형성화할 때 쓰인 곳에서 에러(E0080)가 나므로, C++의 인스턴스화 에러와 같은 모양입니다(연습 2). C++의 concept은 구조적이라 조건만 맞으면 어떤 타입이든 만족하지만, Rust의 트레이트는 `impl`로 밝힌 타입만 만족합니다.

값을 템플릿 매개변수로 받는 C++의 `template <std::size_t N>`은 const 제네릭이 맡습니다. [02_const_generics](examples/02_const_generics.rs)입니다.

```text
1. 배열 길이를 받는 함수
  2
  10
2. 용량이 타입에 들어 있는 스택
  Ok(())
  Ok(())
  Err("c")
  기본 용량: 4
3. N이 다르면 다른 타입이고 크기도 다름(이 환경 aarch64)
  Stack<u32, 2>: 24바이트
  Stack<u32, 8>: 72바이트
```

**다른 언어에서는** C++ 템플릿이 더 많은 것을 표현합니다. 특수화는 Rust에서 nightly 기능이라 안정판에서 쓰면 E0554입니다([ch12](../ch12-modern-cpp-enough/README.md) Q12.6). 값 매개변수로 계산한 식(`std::array<int, N + 1>`)은 4.1절의 에러가 됩니다. 값 매개변수의 타입도 Rust는 정수·`bool`·`char`뿐이라 `&'static str`을 쓰면 `` the only supported types are integers, `bool`, and `char` ``라는 note와 함께 거부되지만, C++20은 클래스 타입도 받습니다.

C++의 가변 인자 템플릿(variadic template, `template <typename... Ts>`)에 해당하는 것은 Rust에 없고, The Unstable Book에도 가변 인자 제네릭 기능은 없습니다(C 가변 인자 함수용 `c_variadic` 계열만 있음). 그래서 원소 수가 다른 튜플마다 구현이 필요하면 `macro_rules!`의 `$(…),*` 반복으로 `impl`을 개수마다 만들어 냅니다. 표준 라이브러리도 `PartialEq` 같은 트레이트를 원소 12개 이하의 튜플에만 구현합니다(`tuple` 문서는 이를 타입 시스템의 일시적 제약 때문이라고 적음). 비용은 정해진 최대 개수와 개수마다 늘어나는 `impl`입니다. 함수의 가변 인자를 슬라이스나 매크로로 대신하는 방법은 ch6 Q6.7에 있습니다.

**오해와 한계** 정의 시점 검사는 에러를 일찍 내는 대신, 본문에서 쓰는 능력을 모두 바운드로 적어야 하는 수고가 듭니다. 바운드가 길어지면 `where` 절이나 트레이트를 묶은 보조 트레이트로 정리합니다. 단형성화의 비용(컴파일 시간, 실행 파일 크기)은 템플릿과 같고, 줄이는 방법은 부록 A ch12에 있습니다.

**더 읽기** ch6 Q6.6·Q6.7 / ch12 Q12.6 / Part 0 ch10 / [부록 A ch12](../../appd-a/ch12-generics-vs-trait-objects/README.md)

### Q13.6 ODR 위반, 링크 순서 같은 문제는 어떻게 되나요?

**짧은 답** Rust의 항목은 크레이트와 모듈 경로로 이름이 정해지고, 컴파일된 심볼 이름에도 그 정보가 들어가므로 다른 곳의 같은 이름이 부딪치지 않습니다. 같은 항목을 두 번 정의하는 일도 없습니다(정의가 한 곳뿐). 크레이트를 링크하는 순서는 Cargo와 `rustc`가 의존 관계로 정합니다. 심볼 이름을 그대로 내보내는 `#[unsafe(no_mangle)]`·`#[unsafe(export_name = "…")]` 함수는 예외라 충돌할 수 있습니다. 같은 크레이트 안의 충돌은 컴파일러가 알리지만, 크레이트 사이의 충돌은 링커 경고만 내고 한쪽 정의가 쓰입니다(macOS의 링커에서 확인).

**왜 그런가** C++의 단일 정의 규칙(ODR)은 `inline` 함수처럼 여러 번역 단위에 정의가 있어도 되는 것들에 "모든 정의가 같아야 한다"를 요구하고, 어기면 프로그램이 잘못된 것이지만 진단은 요구하지 않습니다(표준 초안 [basic.def.odr]). [compare/05_odr_main.cpp](compare/05_odr_main.cpp)와 [05_odr_other.cpp](compare/05_odr_other.cpp)는 `inline int limit()`을 두 파일에서 다르게 정의했습니다. 이 환경에서는 경고 없이 링크되었고, ASan·UBSan도(`ASAN_OPTIONS=detect_odr_violation=2`를 주어도), `-flto -O2` 링크도 아무것도 알리지 않았습니다. 잘못된 프로그램이므로 실행 출력은 싣지 않습니다.

Rust에는 헤더로 같은 정의를 여러 번 복사하는 구조가 없어 이런 위반이 생길 자리가 없고, 보통 함수의 심볼 이름에는 크레이트와 경로 정보가 들어가 겹치지 않습니다(이름 맹글링(name mangling), Part 0 ch19 2.4절). 같은 크레이트 안의 서로 다른 모듈에 `#[unsafe(no_mangle)] pub extern "C" fn answer()`를 두 개 두면 컴파일되지 않습니다(4.2절). 그러나 라이브러리 크레이트와 그것을 쓰는 실행 파일 크레이트가 각각 같은 이름을 내보내면, 이 환경에서는 `warning: linker stderr: ld: duplicate symbol '_answer'`라는 링커 경고만 나고 빌드가 성공했으며 한쪽 정의만 쓰였습니다. 컴파일러가 이런 크레이트 밖(C 라이브러리 포함)의 충돌을 모두 잡을 수 없기 때문에 2024 에디션은 이 속성들을 `unsafe(...)`로 감싸게 했습니다([Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.8절).

**다른 언어에서는** C++20 모듈은 이 문제를 일부 줄입니다. 같은 초안 [basic.def.odr]은 정의가 이름 있는 모듈에 붙어 있고 앞선 정의가 보이는 경우에는 진단을 요구합니다. 헤더 기반 코드에는 해당하지 않습니다. 링크할 라이브러리와 순서는 C/C++에서는 빌드 시스템이 정하고, Rust에서는 크레이트 사이를 Cargo가, 시스템 라이브러리는 빌드 스크립트가 `cargo::rustc-link-lib`로 알려 Cargo가 `rustc`에 넘깁니다(Part 3 ch8).

**오해와 한계** 같은 크레이트의 호환되지 않는 두 버전(예: 1.x와 2.x)이 의존성 그래프에 함께 들어오면 Cargo는 둘 다 컴파일해 넣습니다. 이름이 같아도 서로 다른 타입이라, 한쪽 버전의 값을 다른 쪽 버전을 기대하는 함수에 넘기면 컴파일 에러가 납니다(부록 A ch25). ODR처럼 조용히 섞이지는 않지만, 같은 이름의 타입이 맞지 않는다는 에러가 처음에는 낯설 수 있습니다.

**더 읽기** Part 2 ch4 2.8절 / Part 3 ch8 / 부록 A ch25

### Q13.7 네임스페이스, `friend`, 익명 네임스페이스와 `static` 함수에 해당하는 것은 무엇인가요?

**짧은 답** 모듈이 네임스페이스이고, `use`가 `using`입니다. 모든 항목은 기본으로 비공개라 그 모듈(과 하위 모듈) 밖에서 보이지 않으므로, 파일 밖으로 내보내지 않으려고 쓰던 익명 네임스페이스나 `static` 함수가 따로 필요 없습니다. 같은 모듈 안의 코드는 다른 타입의 비공개 필드도 볼 수 있어 `friend`가 하던 일을 모듈 경계가 맡고, 범위를 넓히려면 `pub(crate)`, `pub(super)`, `pub(in 경로)`를 씁니다.

**왜 그런가** C++의 접근 제어는 클래스 단위이고 파일 밖 공개 여부는 링키지(linkage, 외부·내부)로 정합니다. Rust는 둘 다 모듈 단위의 가시성 하나로 정합니다. 같은 모듈의 다른 타입이 비공개 필드를 읽는 예는 ch6 Q6.10에, 가시성 단계와 그것을 좁게 유지하는 지침은 부록 A ch22에 있습니다.

| C++ | Rust |
| --- | --- |
| `namespace net { … }` | `mod net { … }` 또는 `net.rs` 파일 |
| `using net::Socket;` | `use net::Socket;` |
| 익명 네임스페이스, 파일 범위 `static` 함수 | `pub` 없는 항목(기본) |
| `friend class Manager;` | 같은 모듈에 두기, 또는 `pub(super)`·`pub(crate)` |
| `private`·`protected`·`public` 멤버 | 비공개(기본)·(없음)·`pub` 필드 |

**다른 언어에서는** C++의 `friend`는 한 클래스가 특정 클래스나 함수에게만 비공개 멤버를 여는 방식이라 범위를 아주 좁게 정할 수 있습니다. Rust에는 "이 타입에게만" 여는 방법이 없어, 그런 관계의 타입은 같은 모듈에 둡니다.

**오해와 한계** 모듈 단위의 비공개는 같은 모듈의 모든 코드가 비공개 필드를 고칠 수 있다는 뜻이기도 합니다. 불변식을 지켜야 하는 타입은 작은 모듈에 따로 두어, 필드를 만질 수 있는 코드의 범위를 좁힙니다(부록 A ch22, [부록 A ch16](../../appd-a/ch16-avoid-unsafe/README.md)의 안전한 래퍼).

**더 읽기** ch6 Q6.10 / Part 0 ch7 / 부록 A ch22

### Q13.8 컴파일 단위는 무엇인가요?

**짧은 답** 크레이트입니다. C++의 번역 단위는 소스 파일 하나와 `#include`로 들인 헤더에서 나온 것이고 파일마다 따로 컴파일되지만, Rust에서는 크레이트 전체가 한 번의 `rustc` 호출로 컴파일됩니다. 대신 크레이트 안에서 컴파일러가 코드를 여러 코드 생성 단위(codegen unit)로 나눠 병렬로 처리하고, 바뀐 부분만 다시 처리하는 증분 컴파일(incremental compilation)을 씁니다.

**왜 그런가** C++ 표준 초안 [lex.separate]는 소스 파일과 `#include`로 들인 헤더·소스 파일을 합친 것을 전처리 번역 단위(preprocessing translation unit)로 정하고, 번역 단위는 따로 번역된 뒤 링크될 수 있다고 적습니다. 그것을 토큰으로 바꾼 것이 번역 단위입니다([lex.phases]). Rust Reference는 크레이트를 컴파일과 링크의 단위로 정하고, 컴파일러는 언제나 소스 파일 하나(크레이트 루트)를 입력으로 받아 크레이트 하나를 만든다고 적습니다. 크레이트 안의 병렬성은 `codegen-units`(Part 3 ch6은 "조각"이라 부름)와 `incremental` 프로필 설정으로 조절합니다(Part 3 ch6). 기본값은 `dev` 프로필에서 `codegen-units = 256`, `incremental = true`이고, `release`에서 `16`, `false`입니다.

**다른 언어에서는** C++에서는 파일 하나를 고치면 그 파일과 그 헤더를 포함한 파일들을 다시 컴파일합니다. 헤더를 고치면 다시 컴파일할 파일이 많아지는 이유입니다. Rust에서는 크레이트 안의 무엇을 고쳐도 그 크레이트가 다시 컴파일 대상이 되고, 증분 컴파일이 그 안에서 다시 할 일을 줄입니다.

**오해와 한계** 크레이트가 단위라는 것은 크레이트 하나가 너무 커지면 작은 수정에도 다시 컴파일할 양이 많다는 뜻입니다. 큰 프로젝트를 워크스페이스의 여러 크레이트로 나누는 이유 가운데 하나입니다(Part 3 ch4). 또 코드 생성 단위가 많을수록 병렬로 빨리 컴파일되지만 단위 사이의 최적화 기회는 줄어듭니다(Part 3 ch6).

**더 읽기** Q13.1 / Part 3 ch4·ch6 / [ch15](../ch15-pain-points/README.md)

### Q13.9 테스트, 문서, 포매터, 린터를 따로 골라야 하나요?

**짧은 답** 기본은 고르지 않아도 됩니다. 단위 테스트와 통합 테스트, 문서 테스트는 `cargo test`, API 문서는 `cargo doc`(rustdoc), 포매터는 `cargo fmt`(rustfmt), 린터는 `cargo clippy`가 공식 도구이고, 이 환경에서도 툴체인과 함께 설치되어 있습니다. 그래서 Rust 프로젝트는 대부분 같은 명령과 같은 코드 모양을 씁니다. 성질 기반 테스트, 퍼징, 벤치마크 통계처럼 더 필요한 것은 크레이트나 도구로 더합니다.

**왜 그런가** 테스트 프레임워크가 언어(`#[test]`)와 Cargo에 들어 있고, 문서 주석의 코드 예제가 테스트로 실행됩니다. 포매터와 린터의 설정도 프로젝트 파일(`rustfmt.toml`, `Cargo.toml`의 `[lints]`)에 둡니다. 각 도구는 [Part 3 ch9](../../part3-cargo/ch9-testing-ci/README.md)(테스트와 CI), [Part 3 ch11](../../part3-cargo/ch11-commands-tools/README.md)(명령과 도구), [부록 A ch29](../../appd-a/ch29-listen-to-clippy/README.md)(Clippy), [부록 A ch31](../../appd-a/ch31-tooling-ecosystem/README.md)(도구 지도)에서 다뤘습니다.

**다른 언어에서는** C/C++에서는 테스트 프레임워크, 문서 생성기, 포매터, 정적 분석기를 프로젝트마다 고르고 빌드에 연결합니다. 이 환경에 있는 Homebrew LLVM의 `clang-format`과 `clang-tidy`가 그런 도구의 예이고, [ch11](../ch11-cpp-raii-move/README.md)·ch12에서 `clang-tidy`를 썼습니다. 고를 수 있다는 것은 장점이기도 해서, 오래된 코드베이스는 자기에게 맞는 조합을 이미 갖추고 있습니다.

**오해와 한계** 공식 도구가 있다고 충분한 것은 아닙니다. `cargo test`는 테스트를 실행할 뿐 커버리지를 재지 않고, Clippy의 기본 린트 묶음은 일부 검사만 켭니다(부록 A ch29). 퍼징, 커버리지, 의존성 감사 같은 도구는 따로 설치해야 합니다([부록 A ch30](../../appd-a/ch30-beyond-unit-tests/README.md)·ch31).

**더 읽기** Part 3 ch9·ch11 / 부록 A ch29·ch30·ch31

### Q13.10 플랫폼별 코드와 조건부 컴파일은 어떻게 하나요?

**짧은 답** `#[cfg(target_os = "linux")]`처럼 항목에 붙이는 속성, 식에서 쓰는 `cfg!(...)`, 속성을 조건부로 붙이는 `#[cfg_attr(...)]`로 합니다. 플랫폼마다 다른 의존성은 `Cargo.toml`의 `[target.'cfg(...)'.dependencies]`에, 선택 기능은 `[features]`에 적습니다. Cargo는 알려진 이름과 값을 컴파일러에 넘기므로, 이름이나 값의 오타는 `unexpected_cfgs` 경고가 됩니다.

**왜 그런가** C 전처리기는 `#ifdef`의 이름이 어딘가에서 정의될지 모르므로, 오타를 내도 알릴 근거가 없습니다. [compare/04](compare/04_ifdef_typo.c)는 `__APPLE__`을 `__APLE__`로 잘못 적었고, macOS에서도 `그 밖의 플랫폼 경로`를 출력했습니다. `-Wall -Wextra -Wundef`를 켜도 경고가 없었습니다. Rust에서는 Cargo가 알려진 `cfg` 이름과 값을 컴파일러에 넘기므로, Cargo로 빌드할 때 `target_os = "macoss"` 같은 오타나 `Cargo.toml`에 없는 기능 이름이 경고됩니다(기능 이름의 예는 부록 A ch26 예제 03). `--check-cfg` 없이 `rustc`만 부르면 이 검사가 일어나지 않습니다.

이 환경에서 `rustc --print cfg`가 출력한 값 가운데 일부는 `target_os="macos"`, `target_arch="aarch64"`, `target_family="unix"`, `target_endian="little"`, `debug_assertions`입니다. `cfg` 조건을 쓰는 여러 형태와 `#[cfg]`와 `cfg!`의 차이는 부록 A ch26 2.2절에, 플랫폼별 의존성은 Part 3 ch2에, 기능은 Part 3 ch5에 있습니다.

**다른 언어에서는** C/C++에서는 컴파일러가 미리 정의하는 매크로(`__APPLE__`, `__linux__`, `_WIN32` 등)와 빌드 시스템이 넘기는 `-D` 옵션으로 조건부 컴파일을 합니다. 이름의 목록이 컴파일러마다 다르고, 확인은 사람이 합니다.

**오해와 한계** `#[cfg]`로 뺀 갈래는 그 플랫폼에서 빌드하기 전까지 컴파일되지 않으므로, 여러 플랫폼을 지원하려면 CI에서 플랫폼마다 빌드해야 합니다. 기능 조합이 늘면 확인할 조합도 늘어납니다(부록 A ch26 2.6절).

**더 읽기** 부록 A ch26 / Part 3 ch2·ch5 / Q13.2

### 한눈에 보기

| C/C++ | Rust | 질문 |
| --- | --- | --- |
| 헤더와 전방 선언 | 없음. 크레이트 안의 함수·타입·모듈은 순서와 상관없이 서로를 가리킴(`macro_rules!`는 예외) | Q13.1 |
| `#include` | `mod`, `use` | Q13.2 |
| `#define` 상수 | `const`, `static` | Q13.2 |
| 함수형 매크로 | 함수, 제네릭, `macro_rules!`(위생적, 문법 단위로 인자를 받음) | Q13.3 |
| Make, CMake, 패키지 관리자 | Cargo(`Cargo.toml`, `Cargo.lock`, `build.rs`) | Q13.4 |
| 템플릿 | 제네릭(정의할 때 검사) | Q13.5 |
| concept | 트레이트 바운드(`impl`로 밝혀야 만족) | Q13.5 |
| 값 템플릿 매개변수 `template <std::size_t N>` | const 제네릭 `<const N: usize>`(계산한 식은 안정판에서 불가) | Q13.5 |
| 특수화 | 안정판에 없음(nightly 기능) | Q13.5 |
| 가변 인자 템플릿 | 없음(`macro_rules!`로 개수마다 `impl`을 만듦) | Q13.5 |
| ODR, 링크 순서 | 경로로 구별되는 항목, Cargo가 정하는 링크 | Q13.6 |
| `namespace`, `using` | 모듈, `use` | Q13.7 |
| 익명 네임스페이스, `static` 함수 | 비공개 항목(기본) | Q13.7 |
| `friend` | 같은 모듈, `pub(super)`·`pub(crate)` | Q13.7 |
| 번역 단위 | 크레이트(안에서 코드 생성 단위로 나뉨) | Q13.8 |
| 테스트·문서·포매터·린터를 골라 연결 | `cargo test`·`cargo doc`·`cargo fmt`·`cargo clippy` | Q13.9 |
| `#ifdef`, 미리 정의된 매크로 | `#[cfg]`, `cfg!`, `#[cfg_attr]`, `cfg_select!`, 플랫폼별 의존성, 기능 | Q13.10 |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_no_forward_declarations](examples/01_no_forward_declarations.rs) | 아래에 정의한 함수·타입·모듈을 먼저 쓰기, 서로를 부르는 함수 | Q13.1 |
| [02_const_generics](examples/02_const_generics.rs) | 배열 길이를 받는 함수, 용량을 타입에 둔 스택과 기본값, N마다 다른 타입 | Q13.5 |

### 01_no_forward_declarations — 전방 선언 없이 쓰기

```sh
cargo run -p faq-ch13-headers-build-templates --example 01_no_forward_declarations
```

```text
1. 아래에 정의한 함수를 위에서 부름
  안녕, 헤더
2. 서로를 부르는 함수
  is_even(10) = true, is_odd(7) = true
3. 아직 정의하지 않은 타입을 필드로 씀
  개발: 1명, 첫 사람 민지
  첫 사람의 소속 있음? false
4. 아래에 정의한 모듈의 함수
  넓이 = 12
```

- 3번의 `Department`와 `Employee`는 서로를 가리킵니다. C++라면 한쪽을 `struct Department;`로 먼저 선언합니다. Rust는 선언이 필요 없습니다. `Department`가 `Employee`를 `Vec`(원소를 힙에 둠)으로 들고 있어 두 타입의 크기가 정해지므로 `Box`도 필요 없습니다. 간접 참조 없이 서로를 직접 품으면 크기가 무한해져 E0072(``recursive types `Department` and `Employee` have infinite size``)가 납니다(재귀 타입과 `Box`는 [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)).
- 4번의 `use geometry::area;`는 파일 맨 위에 있고 `mod geometry`는 맨 아래에 있습니다. `use`도 순서와 상관없습니다.

### 02_const_generics — const 제네릭

```sh
cargo run -p faq-ch13-headers-build-templates --example 02_const_generics
```

```text
1. 배열 길이를 받는 함수
  2
  10
2. 용량이 타입에 들어 있는 스택
  Ok(())
  Ok(())
  Err("c")
  기본 용량: 4
3. N이 다르면 다른 타입이고 크기도 다름(이 환경 aarch64)
  Stack<u32, 2>: 24바이트
  Stack<u32, 8>: 72바이트
```

- `average([1.0, 2.0, 3.0])`처럼 부를 때 `N`을 적지 않아도 인자의 배열 길이에서 추론합니다. 본문에서 `N`은 보통의 `usize` 값처럼 읽힙니다.
- `Stack<T, const N: usize = 4>`처럼 const 매개변수에도 기본값을 줄 수 있어, `Stack<u8>`은 용량 4입니다.
- 3번의 크기는 원소 `Option<u32>`(8바이트) N개와 `len`(8바이트)을 더한 값입니다. 크기는 타입 배치에 따라 정해지므로 다른 플랫폼에서는 다를 수 있습니다.

### C/C++ 비교 코드

저장소 루트에서 실행합니다. `cc`·`c++`의 `-o`는 출력 디렉터리를 만들지 않으므로 먼저 `mkdir -p target/tmp/faq-ch13-headers-build-templates`를 실행합니다. 파일마다 머리 주석에 명령이 있습니다.

| 파일 | 질문 | 명령(요지) | 이 환경에서 나온 것 |
| --- | --- | --- | --- |
| [01_macro_double_eval.c](compare/01_macro_double_eval.c) | Q13.3 | `cc -std=c17 -Wall -Wextra -g -fsanitize=address,undefined` 후 실행 | 경고·보고 없음, `m = 6, i = 7` |
| [02_macro_name_capture.c](compare/02_macro_name_capture.c) | Q13.3 | 같음 | 경고 `unused variable 'count'` 하나, 보고 없음, `value: 7` 두 번, `count: 1`, `count: 0` |
| [03_template_checked_late.cpp](compare/03_template_checked_late.cpp) | Q13.5 | `c++ -std=c++20 -Wall -Wextra -fsyntax-only`, `-DUSE_INT`, `-DUSE_INT_CHECKED` | 기본은 에러 없음. 두 옵션의 에러는 Q13.5 |
| [04_ifdef_typo.c](compare/04_ifdef_typo.c) | Q13.10 | `cc -std=c17 -Wall -Wextra -Wundef -g -fsanitize=address,undefined` 후 실행 | 경고·보고 없음, `그 밖의 플랫폼 경로` |
| [05_odr_main.cpp](compare/05_odr_main.cpp), [05_odr_other.cpp](compare/05_odr_other.cpp) | Q13.6 | 두 파일을 `c++ -std=c++20 -Wall -Wextra -g -fsanitize=address,undefined`로 함께 링크 후 실행 | 경고·보고 없음, 종료 0(잘못된 프로그램이라 출력은 싣지 않음) |

- 01·02·04는 미정의 동작이 없어 출력이 정해져 있으므로 출력을 적었습니다. 새니타이저는 미정의 동작이 없다는 것을 확인하려고 켰습니다.

## 4. 자주 하는 실수

### 4.1 const 매개변수로 계산한 길이를 타입에 씀 — const 연산 제한

파일: [`compile_fail/const_generic_expression.rs`](compile_fail/const_generic_expression.rs)

```text
error: generic parameters may not be used in const operations
  --> compile_fail/const_generic_expression.rs:17:60
   |
17 | fn push_one<const N: usize>(xs: [i32; N], x: i32) -> [i32; N + 1] {
   |                                                            ^ cannot perform const operation using `N`
   |
   = help: const parameters may only be used as standalone arguments here, i.e. `N`
```

- **읽는 법**: 타입 자리(`[i32; N + 1]`)에서는 const 매개변수 `N`을 "그대로만" 쓸 수 있다는 뜻입니다. 에러 코드가 없는 에러입니다.
- **원인**: C++에서는 `std::array<int, N + 1>`처럼 값 매개변수로 계산한 식을 타입에 쓸 수 있지만, Rust 안정판의 const 제네릭은 그렇지 않습니다. 이 제한을 푸는 `generic_const_exprs`는 The Unstable Book에 있는 nightly 전용 기능이고, 안정판에서 `#![feature(...)]`를 쓰면 E0554입니다.
- **해결**: 결과 길이를 두 번째 const 매개변수 `M`으로 받고 본문에서 `M == N + 1`을 확인하거나(부르는 쪽이 `let out: [i32; 3] = push_one([1, 2], 3);`처럼 길이를 정함), 길이가 실행 중에 정해지는 `Vec<i32>`를 돌려줍니다(연습 2). 확인을 인라인 const 블록 `const { assert!(M == N + 1, "…") };`로 쓰면 컴파일 시간에 잡습니다. 다만 그 에러(E0080)는 단형성화할 때 나므로 `cargo build`에서는 나고 `cargo check`로는 드러나지 않습니다. 비용은 부르는 쪽이 길이를 적어야 하고, 에러가 정의가 아니라 쓰인 곳에서 늦게 난다는 것입니다.

### 4.2 같은 크레이트에서 `no_mangle` 이름이 겹침 — 심볼 중복

파일: [`compile_fail/no_mangle_symbol_clash.rs`](compile_fail/no_mangle_symbol_clash.rs)

```text
error: symbol `answer` is already defined
  --> compile_fail/no_mangle_symbol_clash.rs:23:5
   |
23 |     pub extern "C" fn answer() -> i32 {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

- **읽는 법**: 같은 심볼 이름이 이미 정의되어 있다는 뜻입니다. 두 번째 정의를 가리키며, 에러 코드는 없습니다.
- **원인**: `#[unsafe(no_mangle)]`은 이름 맹글링을 끄고 함수 이름을 그대로 심볼 이름으로 내보냅니다. 모듈 `sensor`와 `display`가 같은 이름 `answer`를 내보내 한 크레이트 안에서 부딪쳤습니다. 크레이트 사이라면 이 에러 대신 링커 경고만 나고 한쪽 정의가 쓰입니다(Q13.6).
- **해결**: 내보낼 이름을 겹치지 않게 짓습니다(라이브러리 이름을 접두사로 붙이는 등). C에서 부를 필요가 없으면 `no_mangle`을 빼고 보통 함수로 둡니다.

### 4.3 C/C++의 다른 습관과 이미 본 에러

이 챕터의 다른 주제에 해당하는 Rust 에러는 앞 챕터에 이미 있습니다.

| C/C++에서 하던 것 | Rust에서 나는 에러·경고 | 본 곳 |
| --- | --- | --- |
| 제약 없는 템플릿처럼 바운드 없는 `T`로 연산 | E0369 | Part 0 ch10 4.1절 |
| concept을 못 채운 타입으로 부름 | E0277 | Part 0 ch10 4.2절 |
| 매크로 안에서 부르는 쪽의 지역 변수를 고침 | E0425(위생) | 부록 A ch28 4.1절 |
| `#ifdef`의 이름 오타 | `unexpected_cfgs` 경고 | 부록 A ch26 예제 03 |
| 템플릿 특수화 | E0554(nightly 기능) | ch12 Q12.6 |

## 5. 연습 문제

### 연습 1. 용량 0인 스택

`examples/02_const_generics.rs`에서 `Stack<u8, 0>`을 만들고 `push(1)`의 결과와 `size_of::<Stack<u8, 0>>()`를 출력해 보세요. 컴파일될까요?

<details>
<summary>힌트</summary>

컴파일되고, `push`는 처음부터 `Err(1)`을 돌려줍니다(`self.len == N`이 바로 참). 이 환경에서 크기는 `len` 필드 하나뿐인 8바이트였습니다. 길이 0인 배열도 타입이고, C++의 `std::array<T, 0>`처럼 쓸 수 있습니다.

</details>

### 연습 2. 길이를 부르는 쪽이 정하게 고치기

`compile_fail/const_generic_expression.rs`를 `fn push_one<const N: usize, const M: usize>(xs: [i32; N], x: i32) -> [i32; M]`로 바꿔 컴파일되게 고치세요. 본문 첫 줄에서 `M == N + 1`을 확인합니다. 그다음 부르는 쪽에서 결과 타입을 `[i32; 4]`로 잘못 적으면 어떻게 될까요?

<details>
<summary>힌트</summary>

`assert_eq!(M, N + 1, "결과 길이는 N + 1이어야 함");`를 첫 줄에 두고 `std::array::from_fn(|i| if i < N { xs[i] } else { x })`를 돌려주면, `let out: [i32; 3] = push_one([1, 2], 3);`는 `[1, 2, 3]`을 출력합니다. `[i32; 4]`로 적으면 컴파일은 되지만 실행 중에 `` assertion `left == right` failed: 결과 길이는 N + 1이어야 함 ``으로 패닉합니다. 첫 줄을 `const { assert!(M == N + 1, "결과 길이는 N + 1이어야 함") };`로 바꾸면 `[i32; 4]`인 호출이 컴파일 에러 `error[E0080]: evaluation panicked: 결과 길이는 N + 1이어야 함`이 되고, `the above error was encountered while instantiating`으로 시작하는 note가 어느 호출인지 알려 줍니다. 이 에러는 `cargo check`로는 나지 않고 `cargo build`에서 납니다.

</details>

### 연습 3. 같은 매크로를 Rust로

`compare/02_macro_name_capture.c`의 `PRINT_TWICE`를 `macro_rules! print_twice`로 옮겨 보세요. 본문에서 `let mut count = 2;`로 세고, `let count = 7; print_twice!("count", count);`로 부르면 무엇이 출력될까요?

<details>
<summary>힌트</summary>

`count: 7`이 두 번 출력됩니다. 매크로 본문의 `count`와 부르는 쪽이 넘긴 `count`는 이름이 같아도 위생 때문에 다른 변수로 취급됩니다. 본문은 `{ let mut count = 2; while count > 0 { count -= 1; println!("{}: {}", $label, $x); } }`처럼 씁니다(`$label:expr, $x:expr`). `$x`가 반복문 안에 있으므로 인자는 두 번 평가됩니다. 부작용이 있는 식을 넘기면 그 효과도 두 번 일어납니다(Q13.3의 한계).

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rust Reference: [Crates and source files](https://doc.rust-lang.org/reference/crates-and-source-files.html), [Conditional compilation](https://doc.rust-lang.org/reference/conditional-compilation.html), [Macros By Example: Hygiene](https://doc.rust-lang.org/reference/macros-by-example.html#hygiene)
- The Rust Programming Language: [7장 Packages, Crates, and Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html), [10장 Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html), [20.5절 Macros](https://doc.rust-lang.org/book/ch20-05-macros.html)
- The Cargo Book: [Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html), [Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html), [Specifying Dependencies: Platform specific dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies)
- The Unstable Book: [generic_const_exprs](https://doc.rust-lang.org/unstable-book/language-features/generic-const-exprs.html)

C/C++

- C++ 표준 초안: [\[lex.separate\]](https://eel.is/c++draft/lex.separate)(전처리 번역 단위), [\[lex.phases\]](https://eel.is/c++draft/lex.phases)(번역 단위), [\[module.unit\]](https://eel.is/c++draft/module.unit)(모듈), [\[basic.def.odr\]](https://eel.is/c++draft/basic.def.odr)(단일 정의 규칙)
- C++ Core Guidelines: [ES.31](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines#res-macros2)
