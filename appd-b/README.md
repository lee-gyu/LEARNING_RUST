# 부록 B. Rust FAQ

C/C++, Java, C#, Go, Python, JavaScript/TypeScript를 써 온 프로그래머가 Rust를 처음 볼 때 묻는 질문에 답하는 부록입니다. "왜 보안에 강하다고 하는가", "왜 여러 조직이 Rust로 옮기는가", "GC도 예외도 클래스도 없으면 그 자리를 무엇이 채우는가", "C/C++의 고질적인 문제를 정말 풀었는가" 같은 질문을 주제별로 묶었습니다. 본편(Part 0~11)이 Rust에 무엇이 있고 어떻게 쓰는지를, [부록 A](../appd-a/)가 여러 방법 중 무엇을 왜 고르는지를 다룬다면, 이 부록은 **왜 그렇게 만들었고, 내가 쓰던 언어와 무엇이 다른가**를 다룹니다.

[Part 0](../part0-getting-started/)의 내용을 전제로 합니다. 문법과 API 사용법은 다시 설명하지 않고 해당 본편 챕터로 연결합니다. Part 0을 마쳤다면 본편의 나머지 파트와 상관없이 언제든 읽을 수 있습니다.

## 읽는 법

챕터 하나가 주제 하나를 맡고, 챕터 안의 질문마다 `Q4.2`(ch4의 두 번째 질문)처럼 번호가 붙어 있습니다. 챕터 README의 첫머리에 질문과 한 줄 답을 모은 표가 있으므로, 급하면 그 표만 읽어도 됩니다. 질문마다 다음 순서로 답합니다.

1. **짧은 답**: 그 문단만 읽어도 답이 되는 두세 문장
2. **왜 그런가**: Rust가 그렇게 설계한 이유와 막으려는 문제, 확인할 수 있는 예제
3. **다른 언어에서는**: 독자가 아는 언어의 동작과 나란히 놓기
4. **오해와 한계**: 흔한 오해, Rust가 치르는 비용, 예외
5. **더 읽기**: 본편과 부록 A의 관련 챕터, 관련 질문 번호

챕터는 네 묶음으로 나뉩니다. 순서대로 읽기를 권하지만, 궁금한 묶음부터 읽어도 됩니다.

| 묶음 | 챕터 | 이런 독자에게 |
| --- | --- | --- |
| 왜 Rust인가 | ch1~ch3 | 도입을 검토하거나 Rust가 왜 화제인지 궁금한 사람 |
| 다른 언어와 무엇이 다른가 | ch4~ch8 | Java·C#·Go·Python·JS를 주로 써 온 사람 |
| C/C++의 문제를 어떻게 풀었나 | ch9~ch13 | C/C++를 주로 써 온 사람 |
| 실무에서 묻는 것 | ch14~ch17 | 성능, 학습 비용, 생태계, 기존 코드와의 연동이 궁금한 사람 |

예제는 본편과 같은 방식으로 실행합니다.

```sh
cargo run -p <패키지 이름> --example <예제 이름>
```

패키지 이름은 `faq-ch{N}-{slug}` 꼴이며, 예제 이름과 함께 각 챕터 README의 "예제 안내"에 적혀 있습니다.

## 근거를 다루는 원칙

- **언어의 동작**은 이 저장소의 툴체인(rustc 1.100.0, 2024 에디션)에서 직접 실행해 확인한 것만 씁니다.
- **통계, 도입 사례, 표준화 현황**은 1차 자료(보안 보고서, 회사의 기술 블로그, 정부 기관의 지침, 설문 결과)를 열어 확인한 것만 씁니다. 수치를 적을 때는 누가, 언제, 무엇을 센 숫자인지를 함께 밝히고, 챕터의 "참고 자료"에 자료마다 발행일과 확인일을 적습니다. 이런 사실은 시간이 지나면 낡으므로, 읽는 시점에 원문을 다시 확인하기를 권합니다.
- **C/C++와의 비교 코드**는 챕터의 `compare/` 디렉터리에 두고, 새니타이저를 켠 Apple clang으로 macOS(aarch64)에서 컴파일·실행해 확인했습니다. 미정의 동작이 있는 프로그램은 실행할 때마다, 컴파일러마다 결과가 다르므로 새니타이저가 보고한 오류의 종류만 인용합니다.
- **그 밖의 언어**(Java, C#, Go, Python, JavaScript/TypeScript)는 README 안의 짧은 조각으로만 싣고, 그 언어의 공식 문서를 근거로 링크합니다.
- 질문을 고를 때 참고한 자료(보관된 공식 FAQ, 다른 언어 사용자를 위한 교재들)의 본문과 코드는 옮기지 않았습니다. 설명과 예제는 모두 새로 작성했습니다.

## 챕터 목차

완료된 챕터만 행으로 추가합니다. 전체 계획은 [로드맵](../docs/roadmap.md)의 "부록 B"에 있습니다.

| 챕터 | 주제 |
| --- | --- |
| [ch1-why-rust](ch1-why-rust/) | Rust가 풀려는 문제(GC 없는 메모리 안전성, 비용 없는 추상화, 검사되는 스레드 공유), 조직들이 옮기는 이유와 근거(보안 보고서, 도입 사례), 쓰이는 분야와 사용 현황, 맞지 않는 곳, 기존 코드를 다시 써야 하는지, Go·C++와 고르는 기준 |
| [ch2-memory-safety](ch2-memory-safety/) | 메모리 안전성의 다섯 종류(공간·시간·타입·초기화·데이터 경쟁)와 Rust가 막는 장치와 시점, 메모리 버그가 취약점이 되는 과정(Heartbleed 모양의 버그), "70%"를 읽는 법, GC 언어와의 차이, 패닉의 안전성, 정수 실수를 드러내는 장치, Android의 감소 자료를 읽는 법 |
| [ch3-safety-limits](ch3-safety-limits/) | Rust가 보장하지 않는 것(논리 오류, 메모리 누수, 교착 상태와 경쟁 조건, 정수 오버플로, 패닉과 스택 오버플로로 인한 중단), `unsafe`의 범위와 안전한 API 안에 가두기, 표준 라이브러리의 검증, Rust 코드의 CVE 사례, 공급망과 컴파일러의 건전성 버그 |
| [ch4-memory-without-gc](ch4-memory-without-gc/) | GC 없는 메모리 관리: Java식 공유를 이동·빌림·복제·`Rc<RefCell<T>>`로 옮기기, `Copy`의 기준, 참조와 C 포인터·Java 참조의 차이, 가변 참조가 하나뿐인 이유, `clone`의 비용, `Rc`/`Arc`와 추적 GC, 라이프타임 표기, GC가 없어서 얻고 잃는 것, 값이 그 자리에 놓이는 메모리 배치 |
| [ch5-no-null-no-exceptions](ch5-no-null-no-exceptions/) | null도 예외도 없이: Kotlin·TS의 null 연산자를 `Option`으로 옮기기, Java `Optional`·Kotlin `T?`와의 차이, `Result`와 checked exception·Go `error`, `?`, `finally`·`defer`·`with` 자리의 `Drop`(파일·잠금), 패닉과 예외, `unwrap`의 자리, 에러 타입 고르기, 스택 추적 |
| [ch6-no-classes-inheritance](ch6-no-classes-inheritance/) | 클래스도 상속도 없이: 구조체와 `impl`, 생성자 관례, Java 계층을 조합·위임·기본 메서드로 옮기기, 트레이트와 Java·Go 인터페이스, 제네릭과 Java 타입 소거·C++ 템플릿·Go, 오버로딩·기본 인자·가변 인자의 대안, 열거형과 트레이트 객체로 옮기는 계층, 리플렉션 대신 매크로, 모듈 단위 공개 범위 |
| [ch7-syntax-type-surprises](ch7-syntax-type-surprises/) | 문법과 타입에서 낯선 것: 불변 기본과 섀도잉, 표현식과 `++`·삼항 연산자, 정수의 폭과 `usize`, 암묵적 변환이 없는 수 타입, `String`과 `&str`, 언어마다 다른 문자열 길이와 `s[0]`, 시그니처의 타입 표기, 매크로 `!`, 완전한 `match`, 실수와 `Eq`·`Hash` |

## 질문 색인

완료된 챕터의 질문을 번호순으로 모았습니다.

| 번호 | 질문 |
| --- | --- |
| [Q1.1](ch1-why-rust/) | Rust는 어떤 문제를 풀려고 만든 언어인가요? |
| [Q1.2](ch1-why-rust/) | "시스템 프로그래밍 언어"라는데 웹 백엔드나 CLI 도구에도 쓰나요? |
| [Q1.3](ch1-why-rust/) | 왜 여러 조직이 Rust로 옮기나요? |
| [Q1.4](ch1-why-rust/) | 실제로 어디에서 쓰이나요? |
| [Q1.5](ch1-why-rust/) | 인기가 많다는데 얼마나 쓰이나요? |
| [Q1.6](ch1-why-rust/) | Rust가 맞지 않는 곳은 어디인가요? |
| [Q1.7](ch1-why-rust/) | 기존 코드를 전부 다시 써야 하나요? |
| [Q1.8](ch1-why-rust/) | Go, C++와는 어떤 관계이고 무엇을 고르나요? |
| [Q2.1](ch2-memory-safety/) | 메모리 안전성이 정확히 무엇인가요? |
| [Q2.2](ch2-memory-safety/) | 메모리 버그가 왜 보안 취약점이 되나요? |
| [Q2.3](ch2-memory-safety/) | "취약점의 70%가 메모리 문제"라는 숫자는 어디서 나왔나요? |
| [Q2.4](ch2-memory-safety/) | Rust는 메모리 오류를 어떻게 막나요? 언제 잡나요? |
| [Q2.5](ch2-memory-safety/) | Java, Go, Python, C#도 메모리 안전한데 Rust는 무엇이 다른가요? |
| [Q2.6](ch2-memory-safety/) | 패닉으로 프로그램이 죽는 것도 안전한 건가요? |
| [Q2.7](ch2-memory-safety/) | 메모리 안전성 말고 보안에 도움이 되는 것은 무엇인가요? |
| [Q2.8](ch2-memory-safety/) | 실제로 취약점이 줄었나요? |
| [Q3.1](ch3-safety-limits/) | Rust로 쓰면 버그가 없어지나요? |
| [Q3.2](ch3-safety-limits/) | `unsafe`가 있으면 결국 C와 같지 않나요? |
| [Q3.3](ch3-safety-limits/) | 표준 라이브러리도 `unsafe`를 쓰는데 믿을 수 있나요? |
| [Q3.4](ch3-safety-limits/) | 메모리 누수는 왜 막지 않나요? |
| [Q3.5](ch3-safety-limits/) | 교착 상태와 경쟁 조건은 막아 주나요? |
| [Q3.6](ch3-safety-limits/) | 정수 오버플로는 어떻게 되나요? |
| [Q3.7](ch3-safety-limits/) | 패닉은 서비스 거부 공격이 될 수 있지 않나요? |
| [Q3.8](ch3-safety-limits/) | Rust로 쓴 코드에서도 CVE가 나오나요? |
| [Q3.9](ch3-safety-limits/) | 의존하는 크레이트가 많은데 공급망은 안전한가요? |
| [Q3.10](ch3-safety-limits/) | 컴파일러에 버그가 있으면 보장이 깨지나요? |
| [Q4.1](ch4-memory-without-gc/) | GC도 없고 `free`도 부르지 않는데 메모리는 누가 언제 해제하나요? |
| [Q4.2](ch4-memory-without-gc/) | 변수를 다른 변수에 넣거나 함수에 넘겼더니 왜 더 쓸 수 없나요? |
| [Q4.3](ch4-memory-without-gc/) | 어떤 타입은 넘긴 뒤에도 쓸 수 있는데 기준이 무엇인가요? |
| [Q4.4](ch4-memory-without-gc/) | 참조 `&T`는 C의 포인터, Java의 참조와 무엇이 다른가요? |
| [Q4.5](ch4-memory-without-gc/) | 왜 가변 참조는 한 번에 하나만 만들 수 있나요? |
| [Q4.6](ch4-memory-without-gc/) | `clone()`을 많이 쓰면 나쁜 코드인가요? |
| [Q4.7](ch4-memory-without-gc/) | `Rc`와 `Arc`는 GC가 아닌가요? 순환 참조는 어떻게 되나요? |
| [Q4.8](ch4-memory-without-gc/) | 라이프타임 `'a`는 무엇이고 왜 직접 써야 하나요? |
| [Q4.9](ch4-memory-without-gc/) | GC가 없어서 얻는 것과 잃는 것은 무엇인가요? |
| [Q4.10](ch4-memory-without-gc/) | 값을 스택에 둘지 힙에 둘지 직접 골라야 하나요? |
| [Q5.1](ch5-no-null-no-exceptions/) | `null`이 없으면 "값이 없음"을 어떻게 표현하나요? |
| [Q5.2](ch5-no-null-no-exceptions/) | `Option`은 Java의 `Optional`, Kotlin의 `T?`, TypeScript의 `T \| undefined`와 무엇이 다른가요? |
| [Q5.3](ch5-no-null-no-exceptions/) | 예외가 없는데 에러는 어떻게 알리고 처리하나요? |
| [Q5.4](ch5-no-null-no-exceptions/) | `Result`는 Java의 checked exception, Go의 `if err != nil`과 무엇이 다른가요? |
| [Q5.5](ch5-no-null-no-exceptions/) | `?` 연산자는 무엇을 하나요? |
| [Q5.6](ch5-no-null-no-exceptions/) | `try`/`catch`/`finally`에 해당하는 것은 무엇인가요? |
| [Q5.7](ch5-no-null-no-exceptions/) | 패닉은 예외인가요? 잡을 수 있나요? |
| [Q5.8](ch5-no-null-no-exceptions/) | `unwrap()`은 써도 되나요? |
| [Q5.9](ch5-no-null-no-exceptions/) | 에러 타입을 매번 직접 만들어야 하나요? |
| [Q5.10](ch5-no-null-no-exceptions/) | 에러가 난 위치(스택 추적)는 어떻게 보나요? |
| [Q6.1](ch6-no-classes-inheritance/) | Rust는 객체 지향 언어인가요? |
| [Q6.2](ch6-no-classes-inheritance/) | 클래스가 없으면 데이터와 메서드를 어떻게 묶나요? |
| [Q6.3](ch6-no-classes-inheritance/) | 생성자가 없나요? |
| [Q6.4](ch6-no-classes-inheritance/) | 상속이 없는데 코드 재사용과 다형성은 어떻게 하나요? |
| [Q6.5](ch6-no-classes-inheritance/) | 트레이트는 Java·C#의 인터페이스, Go의 인터페이스와 무엇이 다른가요? |
| [Q6.6](ch6-no-classes-inheritance/) | 제네릭은 Java(타입 소거), C++(템플릿), Go와 어떻게 다른가요? |
| [Q6.7](ch6-no-classes-inheritance/) | 함수 오버로딩, 기본 인자, 가변 인자가 없나요? 대신 무엇을 쓰나요? |
| [Q6.8](ch6-no-classes-inheritance/) | `instanceof`와 다운캐스트, 추상 클래스 계층은 어떻게 옮기나요? |
| [Q6.9](ch6-no-classes-inheritance/) | 리플렉션, 어노테이션, 데코레이터에 해당하는 것은 무엇인가요? |
| [Q6.10](ch6-no-classes-inheritance/) | `private`, `protected`는 어떻게 되나요? |
| [Q7.1](ch7-syntax-type-surprises/) | 변수가 왜 기본으로 불변인가요? 같은 이름으로 `let`을 다시 쓰는 것은 왜 허용하나요? |
| [Q7.2](ch7-syntax-type-surprises/) | 세미콜론이 없는 마지막 줄은 무엇인가요? 삼항 연산자와 `++`는 왜 없나요? |
| [Q7.3](ch7-syntax-type-surprises/) | 정수 타입이 왜 이렇게 많고 `usize`는 무엇인가요? |
| [Q7.4](ch7-syntax-type-surprises/) | `i32` 값을 `i64` 변수에 왜 그냥 넣지 못하나요? |
| [Q7.5](ch7-syntax-type-surprises/) | 문자열 타입이 왜 `String`과 `&str` 둘인가요? 무엇을 쓰나요? |
| [Q7.6](ch7-syntax-type-surprises/) | 문자열을 왜 `s[0]`으로 읽을 수 없나요? |
| [Q7.7](ch7-syntax-type-surprises/) | 타입 추론이 되는데 함수 시그니처에는 왜 타입을 다 써야 하나요? |
| [Q7.8](ch7-syntax-type-surprises/) | `println!`의 `!`는 무엇이고 왜 함수가 아닌가요? |
| [Q7.9](ch7-syntax-type-surprises/) | `match`는 왜 모든 경우를 다뤄야 하나요? `switch`와 무엇이 다른가요? |
| [Q7.10](ch7-syntax-type-surprises/) | 실수는 왜 `HashMap`의 키로 쓰거나 바로 정렬할 수 없나요? |

전체 계획은 [로드맵](../docs/roadmap.md)을, 진행 상황은 [PROGRESS.md](../PROGRESS.md)를 참고합니다.
