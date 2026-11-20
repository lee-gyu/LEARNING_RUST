# 부록 A. Effective Rust

[Effective Rust](https://effective-rust.com/)(David Drysdale 지음, O'Reilly, 2024)의 35개 Item을 하나씩 따라가며 **여러 방법 중 무엇을 왜 고르는가**를 익히는 부록입니다. 본편(Part 0~11)이 Rust에 무엇이 있고 어떻게 쓰는지를 다룬다면, 이 부록은 이미 아는 도구들 사이에서 판단하는 기준을 다룹니다. 챕터마다 비권장 코드와 권장 코드를 나란히 실행해 비교하고, 지침을 따르지 않아도 되는 경우를 함께 짚습니다.

[Part 0](../part0-getting-started/)과 [Part 1](../part1-standard-lib/)의 내용을 전제로 합니다. 문법과 API 사용법은 다시 설명하지 않고 해당 본편 챕터로 연결합니다. 본편의 나머지 파트와는 순서가 무관하므로, Part 1을 마친 뒤라면 언제든 읽을 수 있습니다.

## 출처와 라이선스

- 기준 문서: David Drysdale, *Effective Rust: 35 Specific Ways to Improve Your Rust Code* (O'Reilly Media, 2024). 온라인판은 <https://effective-rust.com/>에서 읽을 수 있습니다.
- 온라인판의 라이선스는 [CC BY-NC-ND 4.0](https://creativecommons.org/licenses/by-nc-nd/4.0/)(저작자 표시, 비영리, 변경 금지)입니다. 이 부록은 책의 본문을 번역하거나 발췌하지 않고, 책의 코드도 옮겨 오지 않습니다. 책에서 가져온 것은 "어떤 지침을 다루는가"라는 범위와 Item의 원제뿐이며, 설명과 예제는 모두 새로 작성했습니다.
- 챕터의 지침은 해당 Item이 다루는 범위를 따르지만, 본편과 잇기 위해 표준 라이브러리 문서나 clippy를 근거로 더한 지침도 있습니다. 책에 없는 지침은 각 챕터의 "책과 달라진 점"에 "책에 없는 지침"으로 밝혔습니다.
- 책은 2018 에디션과 rustc 1.70을 기준으로 쓰였고, 이 저장소는 2024 에디션과 rustc 1.100을 씁니다. 책의 설명과 현재 툴체인의 동작이 다른 곳은 각 챕터의 "책과 달라진 점"에 적었습니다.
- 책은 비동기를 다루지 않으며, 이 부록도 다루지 않습니다.

## 학습 방법

챕터 번호는 책의 Item 번호와 같습니다(`ch4` = Item 4). 번호 순서대로 읽는 것을 권하지만, Item끼리 독립적이어서 필요한 챕터만 골라 읽어도 됩니다. 각 챕터의 `README.md`를 읽으면서 예제를 직접 실행해 봅니다.

```sh
cargo run -p <패키지 이름> --example <예제 이름>
```

패키지 이름은 `er-ch{N}-{slug}` 꼴이며, 예제 이름과 함께 각 챕터 README의 "예제 안내"에 적혀 있습니다. 예제 파일 하나는 지침 하나를 맡고, 같은 파일 안에서 비권장 형태와 권장 형태를 함께 실행해 결과를 비교합니다.

챕터 README의 "핵심 개념"은 지침마다 문제 상황, 비권장 코드, 권장 코드, 지침을 따르지 않아도 되는 경우의 순서로 풀고, 마지막에 "기억할 것"으로 정리합니다.

## 챕터 목차

책의 장 구분을 따라 묶었습니다. 35개 Item이 모두 들어 있습니다.

### Types (책 1장, Item 1~9)

| 챕터 | Item | 주제 |
| --- | --- | --- |
| [ch1-express-data-structures](ch1-express-data-structures/) | Item 1 | 범위에 맞는 정수 타입과 경계에서의 변환, 이름 붙인 필드, `bool` 매개변수 대신 열거형, 필드 조합 대신 데이터를 가진 열거형, 센티널 값 대신 `Option`, 에러 코드 대신 `Result` |
| [ch2-express-common-behavior](ch2-express-common-behavior/) | Item 2 | 동작을 받는 매개변수 고르기: `fn` 포인터보다 `Fn*` 바운드, 가장 느슨한 `Fn`/`FnMut`/`FnOnce`, 구체 타입 대신 트레이트, 마커 트레이트, 트레이트 객체로 받기 |
| [ch3-option-result-transforms](ch3-option-result-transforms/) | Item 3 | `match`를 `if let`·`let...else`·`?`·변환 메서드로 한 단계씩 바꾸기, `Option`과 `Result` 중 고르기, `map_err`와 `From`으로 에러 타입 맞추기, `as_ref`/`as_deref`로 빌려서 변환하기, `match`가 더 나은 곳 |
| [ch4-idiomatic-errors](ch4-idiomatic-errors/) | Item 4 | 에러 타입의 다섯 모양과 고르는 기준: `String` 대신 `Error`를 구현한 뉴타입, 중첩 에러 열거형과 `From`, `Box<dyn Error>`의 득실과 절충형, `thiserror`로 반복 줄이기, 라이브러리는 구체 타입·애플리케이션은 `anyhow` |
| [ch5-type-conversions](ch5-type-conversions/) | Item 5 | 실패할 수 있는 변환은 `TryFrom`, `From`을 구현하고 `Into` 바운드로 받기, 좁히는 변환은 `as` 대신 `try_from`, 넓히는 변환도 `from`, 강제 변환이 일어나는 자리와 일어나지 않는 자리 |
| [ch6-newtype-pattern](ch6-newtype-pattern/) | Item 6 | 단위는 별칭이 아니라 뉴타입으로, 같은 타입의 인자 구별하기, 생성자에서 값의 범위 검증하기, 열어 줄 연산 고르기(`Deref`를 피하는 이유), 고아 규칙 우회, 위임 구현의 비용 |
| [ch7-builders](ch7-builders/) | Item 7 | 위치 인자 생성자의 문제, `Default`와 구조체 갱신 문법을 쓸 수 있는 경우와 한계, 필수 값·비공개 필드·필드 사이 규칙이 있을 때의 빌더와 `build() -> Result`, 소비형 빌더와 `&mut` 빌더 고르기, `derive_builder` |
| [ch8-references-pointers](ch8-references-pointers/) | Item 8 | 읽기만 하는 매개변수는 `&[T]`·`&str`·`&T`로 받기, 넓은 포인터가 들고 다니는 정보, `AsRef`와 `Borrow` 바운드 고르기, `Borrow`의 비교·해시 약속, 스마트 포인터 고르는 순서(소유자 수, 스레드, 내부 가변성, 순환과 `Weak`) |
| [ch9-iterator-transforms](ch9-iterator-transforms/) | Item 9 | 인덱스 반복문을 한 단계씩 체인으로, 누적 변수를 소비 메서드로, 받는 쪽을 `impl IntoIterator`로 열기, 반복문 안의 `?`를 옮기는 선택지(`collect`·`filter_map`·`partition`·`try_fold`), `for_each`보다 `for`, 반복문에서 멈출 곳 |

### Traits (책 2장, Item 10~13)

| 챕터 | Item | 주제 |
| --- | --- | --- |
| [ch10-standard-traits](ch10-standard-traits/) | Item 10 | 공개 타입의 트레이트 체크리스트: derive가 타입의 뜻과 맞는지, `Copy`를 붙일 타입, `PartialEq`와 `Hash`를 같은 기준으로, `f64` 필드와 `total_cmp`, `Default`·`Debug`·`Display` 판단, 연산자 오버로딩의 통념 |
| [ch11-drop-raii](ch11-drop-raii/) | Item 11 | 자원을 가진 타입의 `Drop`과 가드(수동 짝과 비교), 가드의 범위를 블록과 `drop`으로 좁히기, 실패할 수 있는 해제는 `Result`를 돌려주는 메서드로, drop 순서와 가드 설계 |
| [ch12-generics-vs-trait-objects](ch12-generics-vs-trait-objects/) | Item 12 | 기본은 제네릭, 단형성화 비용과 얇은 제네릭 껍데기, 바운드 여럿과 조합 트레이트·업캐스트, 이종 컬렉션(열린 집합은 트레이트 객체, 닫힌 집합은 열거형), dyn 호환 설계와 `where Self: Sized` |
| [ch13-default-implementations](ch13-default-implementations/) | Item 13 | 필수 메서드는 최소로 두고 편의 메서드는 기본 구현으로, 더 나은 방법을 아는 구현의 재정의와 그 약속, 메서드에만 붙이는 바운드, 기본 메서드를 나중에 더할 때의 호환성 |

### Concepts (책 3장, Item 14~20)

| 챕터 | Item | 주제 |
| --- | --- | --- |
| [ch14-lifetimes](ch14-lifetimes/) | Item 14 | 스택 프레임과 소유 사슬, 값·참조(NLL)·임시값의 라이프타임, 라이프타임 매개변수를 출처대로 묶거나 나누기, 생략 규칙 읽기와 `'_`, `'static` 참조의 출처와 `T: 'static`, 뷰 타입과 소유 타입 고르기 |
| [ch15-borrow-checker](ch15-borrow-checker/) | Item 15 | 대여 검사기 에러를 싼 것부터 푸는 순서: 대여 범위 줄이기, 체인 쪼개기와 `Cow`, `take`·`replace`로 바꿔 끼우기, 나눠 빌리기, 한 곳이 소유하고 인덱스로 가리키기(세대 ID), `Rc<RefCell<T>>`와 `Weak`, 자기 참조 구조체의 대안 |
| [ch16-avoid-unsafe](ch16-avoid-unsafe/) | Item 16 | `unsafe`가 옮기는 책임, 쓰기 전에 표준 라이브러리의 안전한 API부터 찾기(경계 검사, 바이트 읽기, 배열 만들기), 없으면 검증된 크레이트, 피할 수 없는 `unsafe`는 안전한 래퍼 한 곳에, 전제 조건을 적는 안전성 주석과 작은 블록·린트, 테스트와 Miri |
| [ch17-shared-state-parallelism](ch17-shared-state-parallelism/) | Item 17 | Rust가 막는 것과 막지 못하는 것, 함께 맞아야 하는 데이터는 잠금 하나로, 잠금은 도우미 메서드 안에서만(쥔 채 클로저를 부르거나 가드를 돌려주지 않기), 공유 상태 대신 채널, 잠금 순서는 마지막 수단 |
| [ch18-dont-panic](ch18-dont-panic/) | Item 18 | 라이브러리는 패닉 대신 `Result`, 패닉판은 실패를 돌려주는 판과 짝으로, 숨은 패닉 지점을 검사 연산으로, `catch_unwind`를 예외 처리로 쓰지 않기(`panic_strategy`), 린트로 막기(`panic_free_lib`) |
| [ch19-avoid-reflection](ch19-avoid-reflection/) | Item 19 | `type_name`은 진단에만, 타입별 처리는 다운캐스트 사슬 대신 트레이트 메서드로, "이 기능도 있나?"는 트레이트로 묻기, 다운캐스트가 필요하면 `Any` 슈퍼트레이트와 업캐스트, 필드를 훑는 코드는 derive에 |
| [ch20-avoid-over-optimizing](ch20-avoid-over-optimizing/) | Item 20 | 상태로 보관하는 타입은 데이터를 소유하기, 보이는 복사를 지우기 전에 세기, 큰 데이터는 라이프타임 대신 `Rc`/`Arc`로 공유 소유, 바꾸기 전에 재고 측정이 가리키는 곳만 바꾸기 |

### Dependencies (책 4장, Item 21~26)

| 챕터 | Item | 주제 |
| --- | --- | --- |
| [ch21-semver](ch21-semver/) | Item 21 | 버전 번호의 약속과 0.x 규칙, 요구 사항은 캐럿으로(`version_requirements`로 Cargo의 선택 확인), 더하기처럼 보여도 깨는 변경(포괄 구현, 제네릭 기본 메서드, 바운드 조이기, 구현 떼기), `derive(Ord)` 순서 같은 동작 변경, `#[non_exhaustive]`는 처음부터(`non_exhaustive`), deprecated로 예고한 뒤 지우고 타입으로 드러내기, MSRV와 `rust-version` |
| [ch22-minimize-visibility](ch22-minimize-visibility/) | Item 22 | 공개 범위는 약속의 범위, 가시성 단계를 쓰는 곳까지만 열기(`pub(super)`·`pub(in …)`·`pub(crate)`), 필드는 비공개로 두고 메서드로 열어 내부 표현 바꾸기, 핸들의 필드 감추기, 하위 모듈은 감추고 `pub use`로 공개 경로 정하기, `dead_code`·`unreachable_pub`·`private_interfaces`로 점검 |
| [ch23-avoid-wildcard-imports](ch23-avoid-wildcard-imports/) | Item 23 | 글롭으로 들어온 이름이 지는 경우와 이기는 경우(같은 스코프, 함수 본문, 프렐류드), 새 트레이트가 부르는 E0034와 조용한 호출 전환, 통제하지 않는 크레이트는 명시적으로 가져오기, 써도 되는 글롭(테스트의 `use super::*`, `pub use`, 프렐류드), 글롭을 남길 때의 `=` 버전 고정(`glob_upgrade`) |
| [ch24-re-export-dependencies](ch24-re-export-dependencies/) | Item 24 | 같은 크레이트의 두 버전이 한 그래프에 들어오면 이름이 같아도 다른 타입, API에 타입이 드러나는 의존성은 `pub use`로 재내보내기(타입만 또는 크레이트 전체), 라이브러리를 고칠 수 없을 때의 앱 쪽 대처(`package` 이름 바꾸기), 드러내기 전에 뉴타입으로 감출 수 있는지 따지기(`two_versions`) |
| [ch25-dependency-graph](ch25-dependency-graph/) | Item 25 | Cargo가 버전을 고르는 규칙, 버전 고정 대신 캐럿 요구 사항과 `Cargo.lock`, 중복 버전을 `cargo tree -d`·`-i`로 찾아 줄이기(`planner`), `Cargo.lock`을 커밋하고 `cargo update`로 정해 둔 때에 올리기, 점검 도구(`cargo::unused_dependencies`, `cargo-deny` 설정 예시), 의존성을 들이기 전에 비용 따지기(`real_graph`) |
| [ch26-feature-creep](ch26-feature-creep/) | Item 26 | `cfg` 속성과 `cfg!`·`cfg_select!`, 기능은 더하기만(기능 통합으로 깨지는 배타적인 기능 `exclusive_features`), 공개 필드와 트레이트 메서드를 기능으로 가리지 않기(`gated_api`), 기능 이름의 오타를 잡는 `unexpected_cfgs`와 `check-cfg`, 기능을 늘리기 전에 조합 수 세기 |

### Tooling (책 5장, Item 27~32)

| 챕터 | Item | 주제 |
| --- | --- | --- |
| [ch27-document-public-interfaces](ch27-document-public-interfaces/) | Item 27 | 시그니처가 말하는 것은 되풀이하지 않고 코드가 말하지 못하는 것 쓰기, 타입으로 말할 수 있는 조건은 타입에, 실패 조건은 `# Errors`·`# Panics`·`# Safety` 절에, 크레이트 문서와 문서 테스트로 검사되는 예제, 항목 링크와 재내보낸 항목이 문서에 보이는 모양, 문서 린트의 한계와 렌더링된 문서 읽기(`doc_quality`) |
| [ch28-macros-judiciously](ch28-macros-judiciously/) | Item 28 | 함수나 제네릭으로 충분하면 매크로를 쓰지 않기(추상화 사다리), 서로 맞아야 하는 코드와 부른 곳의 정보는 매크로로, 인자는 한 번만 평가, 숨은 제어 흐름과 숨은 참조 넣지 않기, 서식은 `format_args!`에, 내보내는 매크로는 `$crate`로, 필드·배리언트마다 만드는 코드는 derive로(`tabular`), 매크로의 비용 |
| [ch29-listen-to-clippy](ch29-listen-to-clippy/) | Item 29 | 경고마다 설명을 읽고 고칠지·리팩터링할지·예외로 둘지 고르기, 예외는 가장 좁은 범위에 이유와 함께 `#[expect]`로, 경고 0개 유지(새 린트와 `build.warnings`), 꺼진 린트 목록도 읽고 골라 켜기(pedantic·restriction), 린트 설정은 `[lints]`·`[workspace.lints]`·`clippy.toml`로 프로젝트 단위로(`lint_config`), 책의 Item별 린트 표 |
| [ch30-beyond-unit-tests](ch30-beyond-unit-tests/) | Item 30 | 테스트 종류마다 보는 자리(변경 실험 표), 단위 테스트는 타입이 보장하지 않는 것을, 의존성의 동작은 조기 경보 테스트로, 통합 테스트와 문서 테스트로 사용자 자리에서 보기, 예제 프로그램은 `?`로 쓰고 실행까지, 벤치마크와 기준선 비교(`criterion`), 퍼즈 테스트를 돌릴 코드와 때, 버그는 재현 테스트부터(`tinytlv`) |
| [ch31-tooling-ecosystem](ch31-tooling-ecosystem/) | Item 31 | 기본 도구 묶음, Cargo에 있는 하위 명령부터 쓰고 CI에는 검사 모드, 도구는 사람용 출력이 아니라 `cargo metadata`를 읽기(`meta_lint`), 문제에서 출발해 도구 찾기(책 전체의 도구 지도), 들이기 전에 따지고 버전과 잠금 파일 고정, 도구마다 둘 곳 정하기(편집기·변경마다·정기·게시 전) |
| [ch32-continuous-integration](ch32-continuous-integration/) | Item 32 | 단계는 실패로 끝나는 명령으로 빠짐없이(`--locked`, 문서 테스트, 예제 실행, 기능 조합, MSRV), 경고는 실패로 하고 툴체인과 도구는 고정(`build.warnings`, `rust-toolchain.toml`), 로컬과 CI가 같은 스크립트를 돌리고 실패는 종료 코드로(`pollcount`의 `ci.sh`), 검사를 주기별로 나누기(워크플로 예시), 공개 CI의 권한과 외부 코드 고정 |

### Beyond Standard Rust (책 6장, Item 33~35)

| 챕터 | Item | 주제 |
| --- | --- | --- |
| [ch33-no-std](ch33-no-std/) | Item 33 | `std::` 대신 `core::`·`alloc::` 경로로 쓰기(같은 타입, std 전용 항목 표), `std`·`alloc` 기능을 더하기만 하는 기능으로 나누기(`tinyhex`), 호스트 테스트와 기능 조합 검사, std 없는 타깃 검사와 호스트 보조 검사(`leak_check`), 실패할 수 있는 할당(`try_reserve`, 예산 할당기) |
| [ch34-ffi-boundaries](ch34-ffi-boundaries/) | Item 34 | 선언은 헤더와 정확히 맞추고 크기가 정해진 정수 타입 쓰기, 링크는 빌드 스크립트에 맡기고 `links` 적기, `#[repr(C)]`와 `#[repr(transparent)]`, 할당과 해제는 경계의 같은 쪽에서, C 자원은 `Drop` 래퍼에 담고 unsafe 가두기(`tally_ffi`), C에 내보내는 함수의 접두사와 포인터 검사(`textstat_export`), 패닉이 경계를 넘지 못하게 막기 |
| [ch35-bindgen](ch35-bindgen/) | Item 35 | 헤더가 바뀌면 누가 알려 주는가(미정의 동작과 잘못 해석한 값), 사소한 것을 넘는 선언은 헤더에서 생성하기, 생성 범위를 접두사로 좁히고 생성 결과 읽어 보기, `-sys` 크레이트와 그 위의 안전한 크레이트, 커밋해 둔 생성 결과를 새로 생성한 것과 비교하는 검사(`tally_bindings`), C++은 `cxx` |

전체 계획은 [로드맵](../docs/roadmap.md)을, 진행 상황은 [PROGRESS.md](../PROGRESS.md)를 참고합니다.
