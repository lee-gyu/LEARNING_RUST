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

책의 장 구분을 따라 묶었습니다. 작성된 챕터부터 추가합니다.

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

전체 계획은 [로드맵](../docs/roadmap.md)을, 진행 상황은 [PROGRESS.md](../PROGRESS.md)를 참고합니다.
