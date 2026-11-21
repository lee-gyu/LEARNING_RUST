# ch5. null도 예외도 없이

Rust에는 `null`이 없고 예외(exception)도 없습니다. Java, C#, Kotlin, TypeScript, Python, Go를 써 온 사람이라면 곧바로 묻습니다. 값이 없을 때는 무엇을 돌려주는가, 실패는 어떻게 알리는가, `try`/`catch`/`finally`는 무엇으로 대신하는가, 패닉은 결국 예외가 아닌가. 이 챕터는 `Option`과 `Result`의 사용법을 다시 가르치지 않습니다(Part 0 ch6·ch9, Part 1 ch2·ch8의 일입니다). 대신 독자가 쓰던 언어의 null 처리와 예외 처리를 Rust의 무엇에 대응시키면 되는지, 그 설계가 무엇을 얻고 무엇을 치르는지에 답합니다.

- 패키지: `faq-ch5-no-null-no-exceptions`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q5.1](#q51-null이-없으면-값이-없음을-어떻게-표현하나요) | `null`이 없으면 "값이 없음"을 어떻게 표현하나요? | `Option<T>`의 `None`으로 표현합니다. 없을 수도 있다는 것이 타입에 드러납니다. |
| [Q5.2](#q52-option은-java의-optional-kotlin의-t-typescript의-t--undefined와-무엇이-다른가요) | `Option`은 Java의 `Optional`, Kotlin의 `T?`, TypeScript의 `T \| undefined`와 무엇이 다른가요? | 생각은 같고, 확인 없이 쓰면 컴파일 에러인 것도 Kotlin·TypeScript(`strictNullChecks`)와 같습니다. 다른 점은 `Option`이 평범한 열거형이고, 검사를 비켜 갈 구멍이 없다는 것입니다. |
| [Q5.3](#q53-예외가-없는데-에러는-어떻게-알리고-처리하나요) | 예외가 없는데 에러는 어떻게 알리고 처리하나요? | 복구할 수 있는 에러(recoverable error)는 `Result<T, E>`로 돌려주고, 복구할 수 없는 에러(unrecoverable error)인 버그는 패닉으로 멈춥니다. |
| [Q5.4](#q54-result는-java의-checked-exception-go의-if-err--nil과-무엇이-다른가요) | `Result`는 Java의 checked exception, Go의 `if err != nil`과 무엇이 다른가요? | 실패가 시그니처에 드러나는 점은 checked exception과, 에러가 값인 점은 Go와 같습니다. 에러 전파(propagating errors)는 `?`로 자리마다 보이고, 버린 `Result`에는 경고가 납니다(Java보다 약하고 Go보다 강한 강제). |
| [Q5.5](#q55--연산자는-무엇을-하나요) | `?` 연산자는 무엇을 하나요? | `Err`나 `None`이면 그 자리에서 함수를 끝내며 돌려주고, 아니면 안의 값을 꺼냅니다. 필요하면 에러 타입도 바꿉니다. |
| [Q5.6](#q56-trycatchfinally에-해당하는-것은-무엇인가요) | `try`/`catch`/`finally`에 해당하는 것은 무엇인가요? | `catch`는 `match`나 `?`, `finally`는 `Drop`입니다. 파일·잠금 같은 자원도 그 값이 스코프를 벗어나는 모든 길에서 정리됩니다. |
| [Q5.7](#q57-패닉은-예외인가요-잡을-수-있나요) | 패닉은 예외인가요? 잡을 수 있나요? | 예외처럼 스택을 되감지만(unwinding) 버그를 알리는 장치입니다. `catch_unwind`로 잡을 수는 있어도 흐름 제어에 쓰라고 만든 것이 아닙니다. |
| [Q5.8](#q58-unwrap은-써도-되나요) | `unwrap()`은 써도 되나요? | 실패할 리 없음을 아는 곳, 예제와 테스트에서는 씁니다. 실패할 수 있는 곳에서는 처리하거나 `?`로 넘깁니다. |
| [Q5.9](#q59-에러-타입을-매번-직접-만들어야-하나요) | 에러 타입을 매번 직접 만들어야 하나요? | 아닙니다. 애플리케이션은 `Box<dyn Error>`나 `anyhow`로 충분하고, 라이브러리는 열거형을 만들되 `thiserror`로 반복을 줄입니다. |
| [Q5.10](#q510-에러가-난-위치스택-추적는-어떻게-보나요) | 에러가 난 위치(스택 추적)는 어떻게 보나요? | 패닉은 `RUST_BACKTRACE=1`로 봅니다. `Result`의 에러 값은 기본으로 스택 추적(stack trace)을 담지 않으므로, 필요하면 직접 담습니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md): `Option<T>`, `match`, `if let`, `let...else`
- [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md): `panic!`과 되감기, `Result<T, E>`, `unwrap`과 `expect`, `?`로 에러 전파(예제 05), 백트레이스 읽기(2.4절)
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.6절: `Drop`과 drop 순서
- [ch2](../ch2-memory-safety/README.md) Q2.6(패닉은 메모리 안전하게 멈춘다), [ch4](../ch4-memory-without-gc/README.md) Q4.1(소유자가 스코프를 벗어날 때 해제)

`Option`과 `Result`의 메서드 목록은 [Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md)에, 에러 타입 설계와 `catch_unwind`·백트레이스는 [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)에 있습니다. 둘은 이 부록의 선수 과정이 아니므로 "더 읽기"로만 가리킵니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Kotlin·TypeScript의 null 안전 연산자(`?.`, `?:`, `!!`)와 null 검사를 `Option`의 메서드와 문법으로 옮기고, Java의 `Optional`·Kotlin의 `T?`와 다른 점을 설명할 수 있다.
2. 예외 대신 `Result`를 돌려주는 설계를 Java의 checked exception, Go의 `if err != nil`과 비교하고, 에러 값을 보고 다시 시도하는 코드를 작성할 수 있다.
3. `try`/`finally`, `defer`, `with`, `using` 자리에 `Drop`이 온다는 것을 정상 종료·`?`로 인한 조기 반환·정리되는 단위(Rust는 블록, Go의 `defer`는 함수)로 확인할 수 있다.
4. 패닉과 예외의 차이, `unwrap`을 써도 되는 곳, 에러 타입을 고르는 기준, 스택 추적을 보는 방법을 설명할 수 있다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다.

### Q5.1 `null`이 없으면 "값이 없음"을 어떻게 표현하나요?

**짧은 답** 값이 없을 수 있으면 타입을 `Option<T>`로 하고, 없을 때는 `None`, 있을 때는 `Some(값)`을 씁니다. 참조와 일반 타입은 언제나 값이 있으므로(ch2 Q2.4), "없을 수도 있다"는 것이 `Option`이라는 타입으로 드러납니다. 안의 값을 쓰려면 없는 경우를 어떻게 할지 먼저 정해야 합니다.

**왜 그런가** null의 문제는 어떤 참조든 null일 수 있는데 그것이 타입에 보이지 않는다는 데 있습니다. Rust는 "없을 수도 있음"을 별도의 타입으로 만들어, 확인이 필요한 곳과 필요 없는 곳을 컴파일러가 구별하게 합니다. `Option<&T>`나 `Option<Box<T>>`는 널 포인터 최적화(null pointer optimization) 덕분에 포인터 하나와 같은 크기라서 이 구별에 실행 중 비용도 없습니다([ch1](../ch1-why-rust/README.md) 예제 02).

[01_null_safety_operators](examples/01_null_safety_operators.rs)는 회원 → 주소 → 도시를 따라가는 조회를 여러 방법으로 씁니다.

```text
1. ?. 사슬과 ?: 기본값
  회원 1: 부산
  회원 2: 미등록
  회원 3: 미등록
```

회원 2는 주소가 없고 회원 3은 회원 자체가 없지만, 두 경우 모두 사슬 어딘가에서 `None`이 되어 기본값 "미등록"으로 바뀝니다.

**오해와 한계** `Option`을 쓰면 확인을 빼먹을 수 없는 대신, 확인하는 코드를 쓰는 수고가 있습니다. 그 수고를 줄이는 메서드(`map`, `and_then`, `unwrap_or`)와 `?`가 있습니다(Q5.2, Q5.5).

**더 읽기** Part 0 ch6 2.7절 / Part 1 ch2 / [부록 A ch1](../../appd-a/ch1-express-data-structures/README.md)(센티널 값(sentinel value) 대신 `Option`)

### Q5.2 `Option`은 Java의 `Optional`, Kotlin의 `T?`, TypeScript의 `T | undefined`와 무엇이 다른가요?

**짧은 답** "값이 없을 수 있음을 타입에 적는다"는 생각은 같고, 확인하지 않고 쓰면 컴파일 에러가 나는 것도 Kotlin, `strictNullChecks`를 켠 TypeScript와 같습니다. 다른 점은 둘입니다. Rust의 `Option`은 언어에 붙은 특별한 문법이 아니라 표준 라이브러리의 평범한 열거형이라서, `?.`·`?:` 같은 전용 연산자 대신 메서드와 `match`로 다룹니다. 그리고 안전한 Rust에서는 참조가 null일 수 없으므로(ch2 Q2.4), 검사를 끄는 설정(TypeScript)도, `Optional` 변수 자체가 null인 경우(Java)도, 다른 언어에서 들어온 값이 검사를 비켜 가는 구멍(Kotlin의 플랫폼 타입)도 없습니다.

**왜 그런가** 01 예제는 Kotlin·TypeScript의 연산자를 Rust로 옮긴 대응표를 실행해 봅니다.

| Kotlin / TypeScript | Rust | 01 예제 |
| --- | --- | --- |
| `user?.address` | `user.and_then(\|u\| u.address.as_ref())`, 함수 안이라면 `?` | 1·2번 |
| `address?.city` | `address.map(\|a\| a.city.as_str())` | 1번 |
| `city ?: "미등록"` / `city ?? "미등록"` | `.unwrap_or("미등록")` | 1번 |
| `city!!` / `city!` | `.expect("이유")`, `.unwrap()` (없으면 패닉) | 4번 |
| `if (user != null) { … }` | `if let Some(user) = … { … }` | 3번 |

표의 메서드는 Part 1 ch2에서 자세히 다루므로 여기서는 뜻만 적습니다. `map`은 클로저가 보통 값을 돌려줄 때, `and_then`은 클로저가 다시 `Option`을 돌려줄 때 씁니다. 01 예제에서 주소(`Option<Address>`)로 가는 단계는 `and_then`, 도시 이름(`&str`)으로 가는 단계는 `map`인 까닭입니다. Kotlin은 두 경우를 모두 `?.` 하나로 씁니다. 4.1절과 연습 1에 나오는 `map_or(기본값, f)`는 `map(f).unwrap_or(기본값)`을 한 번에 쓴 것이고, `as_deref()`는 `Option<String>`을 빌려 `Option<&str>`로 바꿉니다.

| | Java `Optional<T>` | Kotlin `T?` | TypeScript `T \| undefined` | Rust `Option<T>` |
| --- | --- | --- | --- | --- |
| 무엇인가 | 라이브러리 클래스 | 타입 시스템의 nullable 타입 | 합집합 타입(union type) | 표준 라이브러리의 열거형 |
| `null`로 빠져나갈 길 | 있음. `Optional` 변수 자체가 null일 수 있음(문서는 그러면 안 된다고 적음) | 없음. 다만 Java 코드와 섞을 때의 플랫폼 타입(platform type: Java에서 온 값이라 Kotlin이 null 여부를 모르는 타입)이나, 초기화 중에 `this`가 새어 나가는 경우 NPE가 날 수 있음(Kotlin 문서) | `strictNullChecks`를 끄면 검사하지 않음 | 없음 |
| 확인하지 않고 쓰면 | 컴파일됨(`get()`은 실행 중 예외) | 컴파일 에러 | `strictNullChecks`를 켜면 컴파일 에러 | 컴파일 에러(4.1절의 E0599) |
| "확신한다"는 표시 | `get()`, `orElseThrow()` | `!!`(null이면 NPE) | `!`(실행 동작은 바꾸지 않음) | `unwrap()`, `expect()`(없으면 패닉) |

- Java API 문서는 `Optional`을 "결과 없음"을 나타낼 분명한 필요가 있는 메서드의 반환 타입으로 쓰라고 하고, `Optional` 타입의 변수는 결코 null이면 안 된다고 적습니다. 언어가 막아 주는 것이 아니라 관례라는 뜻입니다.
- Kotlin 문서는 `!!`가 값을 nullable이 아닌 타입으로 바꾸고, 값이 null이면 NPE가 난다고 설명합니다.
- TypeScript 핸드북은 `strictNullChecks`를 켜야 null·undefined를 확인하게 하며, `!` 단언(assertion)은 실행 동작을 바꾸지 않는다고 설명합니다.
- C#의 nullable 참조 형식은 같은 경우를 컴파일 에러가 아니라 경고로 알립니다(ch2 Q2.5).

**오해와 한계** Kotlin의 `?.`처럼 짧은 연산자가 없어서 처음에는 장황하게 느껴질 수 있습니다. 대신 `?`가 함수 안에서 같은 일을 짧게 해 주고(01의 2번), 메서드 체인은 길어도 각 단계가 무엇을 하는지 이름으로 보입니다. Kotlin에서 `?.`를 쓸 자리에 Rust의 `.`를 쓰면 4.1절의 에러가 납니다. 또 C 코드에서 받은 원시 포인터(raw pointer, `*const T`)는 null일 수 있고, 그것을 참조로 바꾸는 일은 `unsafe` 코드가 책임집니다(Part 0 ch19).

**더 읽기** Part 1 ch2 / [부록 A ch3](../../appd-a/ch3-option-result-transforms/README.md)(`match`를 변환 메서드로 바꾸기)

### Q5.3 예외가 없는데 에러는 어떻게 알리고 처리하나요?

**짧은 답** 두 가지로 나눕니다. 파일이 없거나 입력이 틀린 것처럼 **복구할 수 있는 실패**는 함수가 `Result<T, E>`를 돌려 알리고, 부르는 쪽이 `match`나 `?`로 처리합니다. 인덱스가 범위를 벗어나는 것처럼 **버그를 뜻하는 실패**는 패닉으로 멈춥니다. The Book도 Rust에는 예외가 없고, 복구할 수 있는 에러에는 `Result`를, 복구할 수 없는 에러에는 `panic!`을 쓴다고 설명합니다.

**왜 그런가** unchecked 예외(Java의 `RuntimeException`, C#·Python·JavaScript의 예외)는 "어느 함수가 어떤 실패를 할 수 있는가"를 시그니처에 보이지 않게 하고, 실패가 호출 스택의 어디에서 잡힐지도 코드만 보고 알기 어렵게 합니다. `Result`는 실패를 반환값으로 만들어, 실패할 수 있는 함수인지가 시그니처에 보이고 처리하는 곳도 부르는 자리에 보입니다. [02_errors_are_values](examples/02_errors_are_values.rs)는 에러가 값이라서 할 수 있는 일을 봅니다.

```text
1. 에러 값을 보고 다시 시도하기
  1번째 시도: 시간 초과, 다시 시도
  2번째 시도: 시간 초과, 다시 시도
  결과: Ok(8080)
2. 다시 시도할 수 없는 에러는 바로 돌려줌
  결과: Err(NotFound)
```

`fetch_with_retry`는 결과를 `match`로 보고, 시간 초과(`Err(FetchError::Timeout)`)면 다시 시도하고 그 밖의 에러는 그대로 돌려줍니다. 예외라면 `try`/`catch`를 반복문 안에 넣고 예외 타입으로 갈라야 하는 일이, 값에 대한 `match` 한 번이 됩니다.

**오해와 한계** 모든 실패가 `Result`인 것은 아닙니다. 메모리 할당 실패처럼 대부분의 프로그램이 복구할 수 없는 실패는 표준 라이브러리 문서(`handle_alloc_error`)에 따르면 기본으로 표준 에러에 메시지를 출력하고 프로세스를 중단(abort)합니다. 같은 문서는 앞으로의 버전에서 기본 동작이 패닉으로 바뀔 수 있다고도 적습니다. 무엇을 `Result`로 하고 무엇을 패닉으로 할지는 Part 0 ch9 2.11절과 [부록 A ch18](../../appd-a/ch18-dont-panic/README.md)의 기준을 따릅니다.

**더 읽기** Part 0 ch9 2.5·2.11절 / 부록 A ch18

### Q5.4 `Result`는 Java의 checked exception, Go의 `if err != nil`과 무엇이 다른가요?

**짧은 답** 실패가 시그니처에 드러난다는 점은 Java의 checked exception과, 에러가 그냥 값이라 변수에 담고 비교할 수 있다는 점은 Go의 `error`와 같습니다. Java와 다른 점은 전파가 자동이 아니라 `?`로 자리마다 보인다는 것, 그리고 강제하는 방식입니다. 성공한 값을 꺼내 쓰려면 실패를 다룰 수밖에 없지만, 성공한 값이 필요 없어 `Result`를 통째로 버리면(예: `Result<(), E>`) Java처럼 컴파일 에러가 나는 것이 아니라 경고만 납니다. Go와 다른 점은 버린 에러에 경고가 나고, `?` 한 글자로 짧게 전파한다는 것입니다.

**왜 그런가**

| | Java checked exception | Go `error` | Rust `Result` |
| --- | --- | --- | --- |
| 시그니처에 드러나나 | `throws`로 드러남 | 반환값으로 드러남 | 반환 타입으로 드러남 |
| 처리를 강제하나 | 컴파일 시간 검사(unchecked 예외는 제외) | 강제하지 않음 | 값을 꺼내려면 다뤄야 함. 통째로 버리면 경고(`#[must_use]`) |
| 전파 | 자동(호출 스택을 따라 올라감) | `if err != nil { return err }` | `?` |
| 에러를 값으로 다룸 | 잡은 뒤에야 | 언제나 | 언제나 |

- Java 언어 명세는 예외 처리기가 있는지를 컴파일 시간에 검사해 처리되지 않는 예외를 줄이도록 설계했다고 설명합니다. 다만 `RuntimeException`과 `Error`의 하위 클래스는 unchecked라서 이 검사를 받지 않습니다.
- Go 블로그의 "Errors are values"(Rob Pike, 2015-01-12)는 에러가 값이므로 프로그래밍할 수 있다고 설명합니다. 02 예제의 다시 시도하기가 그런 예입니다.
- Go 명세는 함수 호출을 문장으로 쓸 수 있게 하므로(Expression statements), 에러를 돌려주는 함수를 불러 놓고 결과를 버려도 컴파일됩니다.
- Go라면 `if err != nil { return err }`를 쓰는 자리를 Rust는 `?` 하나로 씁니다(Q5.5). `match`로 전파한 판과 `?`로 전파한 판을 나란히 둔 예는 Part 0 ch9 예제 05에 있습니다.

`Result`를 버리는 코드는 기본으로 경고만 나고(Part 0 ch9 2.5절에서 본 경고), `#![deny(unused_must_use)]`로 에러로 올릴 수 있습니다(4.2절). 표준 라이브러리 문서는 `Result`에 `#[must_use]`가 붙어 있어, 값을 무시하면 컴파일러가 경고한다고 설명합니다.

**오해와 한계** `Result`는 실패의 종류를 타입으로 적으므로, 서로 다른 에러 타입을 돌려주는 함수를 이어 부르려면 에러 타입을 맞추는 일이 생깁니다(Q5.9). Java에서 새 checked exception을 던지게 되면 호출 사슬의 `throws` 절을 함께 고쳐야 하는 것과 비슷한 비용이며, Rust는 `From` 변환과 `?`로 이 비용을 줄입니다.

**더 읽기** Part 0 ch9 2.8절 / Q5.5, Q5.9

### Q5.5 `?` 연산자는 무엇을 하나요?

**짧은 답** `Result`나 `Option` 뒤에 붙여, 값이 `Err`/`None`이면 그 자리에서 함수를 끝내며 그것을 돌려주고, `Ok`/`Some`이면 안의 값을 꺼냅니다. `Err`를 돌려줄 때는 함수의 에러 타입으로 바꿀 수 있으면(`From`) 바꿔서 돌려줍니다. Go의 `if err != nil { return err }`를 한 글자로 줄인 것에 가깝습니다.

**왜 그런가** 01의 2번(`city_question`)이 `Option`에 `?`를 씁니다(`Option`에 쓰는 `?`의 기본은 Part 0 ch9 예제 06). 실패하면 일찍 돌아간다는 것이 `?`라는 표시로 줄마다 보이므로, 예외처럼 어디서 빠져나가는지 모르는 일도 없고 Go처럼 같은 세 줄을 반복할 일도 없습니다. `?`가 무엇을 대신하는지는 Part 0 ch9 예제 05가 `match`판과 나란히 보였습니다.

**다른 언어에서는** Kotlin의 `?.`와 Rust의 `?`는 모양이 비슷하지만 하는 일이 다릅니다. Kotlin의 `?.`는 그 표현식만 null로 만들고 계속 진행하지만, Rust의 `?`는 함수 전체를 끝냅니다. Kotlin의 `?.`에 해당하는 Rust 코드는 `map`·`and_then`입니다(Q5.2의 표).

**오해와 한계** `?`는 함수의 반환 타입과 같은 종류에만 씁니다. `Result`를 돌려주는 함수 안에서는 `Result`에, `Option`을 돌려주는 함수 안에서는 `Option`에만 쓸 수 있고, `()`를 돌려주는 함수에서는 쓸 수 없습니다(Part 0 ch9 2.9절, 4.1·4.2절의 E0277). `main`에서 쓰려면 `main`이 `Result`를 돌려주게 합니다(Part 0 ch9 2.10절).

**더 읽기** Part 0 ch9 2.8~2.10절, 예제 05·06 / 부록 A ch3

### Q5.6 `try`/`catch`/`finally`에 해당하는 것은 무엇인가요?

**짧은 답** `catch`에 해당하는 것은 `Result`를 받는 `match`나 `if let`이고, 처리하지 않고 넘기는 것은 `?`입니다. `finally`에 해당하는 것은 `Drop`입니다. 파일(`File`)이나 잠금 가드(`MutexGuard`)처럼 정리가 필요한 표준 라이브러리 타입은 버려질 때 정리 코드가 실행되므로(`MutexGuard`는 자기 `Drop`이, `File`은 안에 든 파일 핸들의 `Drop`이), 그 값이 스코프를 벗어나는 모든 길(정상 종료, `?`로 인한 조기 반환, 패닉으로 인한 되감기)에서 파일이 닫히고 잠금이 풀립니다. 닫거나 푸는 코드는 따로 쓰지 않습니다.

**왜 그런가** 정리 코드를 "정리할 값"에 붙여 두면 정리를 빼먹을 길이 없습니다. 표준 라이브러리 문서는 `File`이 스코프를 벗어나면 자동으로 닫히고, `MutexGuard`가 버려지면 잠금이 풀린다고 적습니다. 네트워크 연결(`TcpStream`)도 버려지면 닫힙니다(Part 0 ch20). [03_cleanup_like_finally](examples/03_cleanup_like_finally.rs)는 잠금으로 세 경우를 봅니다.

```text
1. 정상적으로 끝남
    잠근 동안 다시 잠글 수 있나: false
  결과: Ok(1), 돌아온 뒤 잠글 수 있나: true
2. ?로 일찍 돌아감
    잠근 동안 다시 잠글 수 있나: false
  결과: Err("크기가 숫자가 아님: \"열\""), 돌아온 뒤 잠글 수 있나: true
3. 반복문 안에서 잠그면 한 바퀴마다 풀림
  20 넣음: 기록 2개
  30 넣음: 기록 3개
  반복문 뒤 기록: [10, 20, 30]
```

1·2번 모두 함수 안에서는 다시 잠글 수 없었고(`false`), 돌아온 뒤에는 잠글 수 있습니다(`true`). 2번은 `?`에서 일찍 돌아갔는데도 잠금이 풀렸습니다. 3번은 반복문 한 바퀴가 끝날 때마다 가드가 버려지므로 다음 바퀴의 `lock`이 돌아옵니다. 표준 라이브러리 문서에 따르면 이미 잠근 스레드가 다시 `lock`을 부르면 돌아오지 않습니다(패닉하거나 교착 상태(deadlock)에 빠질 수 있음). 일찍 돌아가도 값이 버려지는 것 자체는 [ch1](../ch1-why-rust/README.md) 예제 01이 메모리로 보였고, 패닉으로 되감는 길에서도 정리된다는 것은 [ch2](../ch2-memory-safety/README.md) 예제 03에서 봤습니다.

**다른 언어에서는**

| 언어 | 정리 구문 | 정리가 붙는 곳 | 파일·잠금의 예 |
| --- | --- | --- | --- |
| Java | try-with-resources, `try`/`finally` | 블록. try-with-resources는 자원을 자동으로, 초기화한 역순으로 닫음(Java 언어 명세 14.20.3) | `try (var r = Files.newBufferedReader(p)) { … }`, `lock.lock(); try { … } finally { lock.unlock(); }` |
| C# | `using` | 블록이나 선언. 예외가 나도 `Dispose`가 불림 | `using var r = File.OpenText(p);` |
| Python | `with` | 블록. `__enter__`가 성공했으면 `__exit__`가 언제나 불림 | `with open(p) as f:`, `with lock:` |
| Go | `defer` | 함수. 함수가 돌아갈 때 등록한 역순으로 실행(Go 명세) | `defer f.Close()`, `defer mu.Unlock()` |
| Rust | `Drop` | 값. 값이 스코프를 벗어날 때(선언의 역순) | `let f = File::open(p)?;`, `let g = m.lock().unwrap();` |

Java의 try-with-resources, C#의 `using`, Python의 `with`는 정리를 "자원 객체"에 맡긴다는 점에서 Rust의 `Drop`과 가깝습니다. 차이는 그 구문을 쓰는 것을 프로그래머가 기억해야 한다는 데 있습니다. Rust에서는 값을 만들기만 하면 정리가 따라옵니다. 여럿이면 만든 순서의 반대로 정리되는 것(Part 0 ch14 예제 04)은 Go의 `defer`, Java의 try-with-resources와 같습니다.

Go의 `defer`와 다른 점은 정리가 붙는 단위입니다. Go 명세에 따르면 `defer`는 둘러싼 함수가 돌아갈 때 실행되므로, 반복문 안에서 `defer mu.Unlock()`을 쓰면 함수가 끝날 때까지 잠금이 풀리지 않습니다. 그리고 `sync.Mutex` 문서에 따르면 이미 잠긴 뮤텍스의 `Lock`은 풀릴 때까지 기다립니다. Rust의 가드는 03의 3번처럼 블록이 끝날 때 버려집니다.

**오해와 한계** `Drop`은 값을 돌려줄 수 없으므로, 정리 자체가 실패할 수 있으면 그 실패를 알릴 길이 없습니다. `File` 문서도 닫을 때 생긴 에러는 `Drop` 구현이 무시한다며, 그 에러를 직접 다뤄야 하면 `sync_all`을 부르라고 적습니다. 그래서 정리가 실패할 수 있는 타입은 `Result`를 돌려주는 메서드를 따로 두고 `Drop`은 안전망으로 씁니다([부록 A ch11](../../appd-a/ch11-drop-raii/README.md) 예제 03).

값을 `let _ = …`로 받으면 그 자리에서 바로 버려지므로, `with` 블록처럼 끝까지 쥐고 있을 값에는 이름을 붙입니다. 표준 라이브러리의 잠금이면 rustc의 `let_underscore_lock` 린트(기본 deny)가 컴파일을 거부하지만, 직접 만든 타입은 아무 말 없이 버려집니다(부록 A ch11). 또 `mem::forget`이나 `process::exit`([ch3](../ch3-safety-limits/README.md) Q3.4), `panic = "abort"`로 인한 중단(Part 0 ch9 2.3절)에서는 `Drop`이 실행되지 않습니다.

**더 읽기** Part 0 ch14 2.6절 / Part 0 ch15 2.7절(잠금 가드) / 부록 A ch11 / ch4 Q4.1

### Q5.7 패닉은 예외인가요? 잡을 수 있나요?

**짧은 답** 겉모습은 예외와 비슷합니다. 기본 설정에서는 스택을 되감으며 값들을 정리하고, `std::panic::catch_unwind`로 잡을 수도 있습니다. 그러나 패닉은 "프로그램에 버그가 있다"를 알리는 장치이지 실패를 처리하는 흐름이 아닙니다. 표준 라이브러리 문서도 `catch_unwind`를 일반적인 try/catch 수단으로 쓰지 말라고 하고, 모든 패닉을 잡는다고 보장하지 않는다고 적습니다.

**왜 그런가** `panic = "abort"`로 빌드하면 패닉은 되감지 않고 프로세스를 바로 끝내므로 `catch_unwind`가 아무것도 잡지 못합니다(Part 0 ch9 2.3절). 라이브러리가 패닉을 흐름 제어에 쓰면, 그 라이브러리를 abort 설정으로 빌드하는 사용자의 프로그램은 그대로 죽습니다. 그래서 정상적으로 일어날 수 있는 실패는 `Result`로 알리는 것이 Rust의 관례입니다.

**다른 언어에서는** 버그를 알리는 쓰임새로는 Java의 unchecked 예외나 Go의 `panic`이 Rust의 패닉에 가깝습니다. 범위를 벗어난 인덱스가 좋은 예로, Java는 `ArrayIndexOutOfBoundsException`을 던지고(Java 언어 명세 15.10.4), Go는 실행 중 패닉을 일으킵니다(Go 명세의 Index expressions). 다만 Java에서는 그 예외도 보통의 `try`/`catch`로 잡을 수 있습니다. Go는 명세에 따라 `panic`을 `recover`로 다룰 수 있지만, 보통의 에러는 `error` 값으로 다룹니다(Q5.4).

**오해와 한계** `catch_unwind`가 쓸모 있는 곳도 있습니다. 스레드 풀이나 서버가 요청 하나의 패닉 때문에 전체가 죽지 않게 경계에서 붙잡거나, C 코드가 부르는 Rust 함수에서 패닉을 에러 코드로 바꾸는 경우입니다(표준 라이브러리 문서에 따르면 `extern "C"`로 정의한 함수에서 패닉하면 프로세스가 중단되므로, 중단 대신 다르게 처리하고 싶을 때 씁니다). 자세한 것은 Part 1 ch8 2.9절과 [부록 A ch18](../../appd-a/ch18-dont-panic/README.md) 예제 04에 있습니다.

**더 읽기** Part 0 ch9 2.2·2.3절 / Part 1 ch8 2.9절 / 부록 A ch18 / ch2 Q2.6

### Q5.8 `unwrap()`은 써도 되나요?

**짧은 답** 씁니다. 다만 "여기서 실패하면 버그다"라고 말할 수 있는 곳에서만 씁니다. 예제, 프로토타입, 테스트, 그리고 앞의 코드가 이미 값을 보장한 곳이 그렇습니다. 그때도 `unwrap()`보다 이유를 적는 `expect("…")`가 낫습니다. 입력이나 외부 상태 때문에 실패할 수 있는 곳에서는 처리하거나 `?`로 넘깁니다.

**왜 그런가** `unwrap`은 Kotlin의 `!!`, TypeScript의 `!`와 같은 자리에 있습니다(Q5.2). 01의 4번은 시작할 때 넣어 둔 회원 1을 `expect("회원 1은 시작할 때 넣어 두었음")`로 꺼냅니다. 만약 이 가정이 깨지면 패닉 메시지에 그 이유가 나오므로 무엇이 틀렸는지 바로 압니다. 외부 입력에서 온 값에 `unwrap`을 쓰면 공격자가 패닉을 일으켜 서비스를 멈추게 할 수 있습니다([ch3](../ch3-safety-limits/README.md) Q3.7).

**오해와 한계** "`unwrap`은 나쁘다"고 기계적으로 피하면 오히려 의미 없는 기본값(`unwrap_or(0)`)으로 버그를 숨기게 됩니다. 실패가 정말 일어날 수 없는지, 일어난다면 어떻게 해야 하는지를 먼저 정합니다. Clippy의 `unwrap_used` 같은 린트로 라이브러리 코드에서 `unwrap`을 금지하는 방법은 부록 A ch18에 있습니다.

**더 읽기** Part 0 ch9 2.7·2.11절 / 부록 A ch18

### Q5.9 에러 타입을 매번 직접 만들어야 하나요?

**짧은 답** 아닙니다. 애플리케이션의 `main` 근처처럼 에러를 사람에게 보여 주기만 하면 되는 곳은 `Box<dyn Error>`나 `anyhow` 크레이트로 어떤 에러든 담습니다. 부르는 쪽이 에러의 종류를 보고 다르게 처리해야 하는 라이브러리는 에러 열거형을 만들되, `thiserror` 크레이트로 반복 코드를 줄입니다.

**왜 그런가** 에러 타입은 부르는 쪽에 주는 정보의 양을 정합니다. 02 예제의 `FetchError`처럼 열거형이면 부르는 쪽이 "시간 초과면 다시 시도"처럼 종류를 보고 갈 수 있고, `Box<dyn Error>`면 메시지를 보여 주는 것 말고는 하기 어렵습니다. 그래서 라이브러리는 구체적인 열거형, 애플리케이션은 담기 쉬운 타입이라는 나눔이 관례가 되었습니다. `thiserror`와 `anyhow`는 이 저장소의 선수 과정에서 쓰지 않으므로 이 챕터에서는 소개만 하고, 두 크레이트를 쓴 예제는 [부록 A ch4](../../appd-a/ch4-idiomatic-errors/README.md)에 있습니다.

**다른 언어에서는** Java·Python에서 예외 클래스 계층을 만드는 일과 같은 고민입니다. Go는 에러의 종류를 `errors.New`로 만든 센티널 값이나 직접 만든 에러 타입으로 나누고, 각각 `errors.Is`(문서의 예: `errors.Is(err, fs.ErrExist)`)와 `errors.As`로 살핍니다. `fmt.Errorf`의 `%w`로 맥락을 덧붙여 감싸도 두 함수가 안쪽의 원래 에러를 찾아 줍니다(`errors` 패키지 문서). Rust의 에러 열거형은 이 둘을 한 타입의 배리언트로 모은 것에 가깝습니다.

**오해와 한계** 에러 타입을 너무 일찍 세밀하게 만들면 바뀔 때마다 고칠 곳이 늘어납니다. 처음에는 `Box<dyn Error>`로 시작해, 부르는 쪽이 종류를 구별할 필요가 생길 때 열거형으로 바꿔도 늦지 않습니다.

**더 읽기** Part 0 ch9 2.10절 / Part 1 ch8 2.3~2.6절 / 부록 A ch4

### Q5.10 에러가 난 위치(스택 추적)는 어떻게 보나요?

**짧은 답** 패닉은 환경 변수 `RUST_BACKTRACE=1`을 켜고 실행하면 스택 추적을 출력합니다. Rust에서는 이것을 백트레이스(backtrace)라고 부릅니다. 반면 `Result`의 에러 값은 기본으로 스택 추적을 담지 않습니다. 에러가 어디서 만들어졌는지 알아야 하면 `std::backtrace::Backtrace`를 에러 값에 담거나, 그 일을 해 주는 `anyhow`를 씁니다.

**왜 그런가** 에러가 그냥 값이라서(Q5.4), 만들 때 스택을 기록하는 비용을 늘 치를 이유가 없기 때문입니다. 표준 라이브러리 문서에 따르면 `Backtrace::capture()`는 `RUST_BACKTRACE`나 `RUST_LIB_BACKTRACE`가 켜져 있을 때만 실제로 스택을 기록하고, 꺼져 있으면 아무 일도 하지 않습니다. 기록하는 데 메모리와 시간이 들기 때문에 환경 변수로 켜고 끄게 한 것입니다. 패닉의 백트레이스를 읽는 법은 Part 0 ch9 2.4절에, 에러 값에 백트레이스를 담는 예는 Part 1 ch8 2.10절에 있습니다.

**다른 언어에서는** Java의 예외 객체는 만들어질 때 그 스레드의 실행 스택을 기록하고(Java API 문서의 `Throwable`), Python의 예외는 `__traceback__` 속성에 트레이스백을 담습니다. Go의 `errors.New`가 만드는 에러는 텍스트 메시지만 담습니다. Rust의 `Result`는 Go 쪽에 가깝고, 필요할 때 Java처럼 기록하게 할 수 있습니다.

**오해와 한계** 스택 추적이 없어도 에러가 어디를 거쳐 왔는지는 `source()` 체인으로 알릴 수 있습니다. "설정을 읽다가 → 파일을 열다가 → 파일이 없음"처럼 원인을 감싸 올리는 방식이며, Part 1 ch8 2.7절이 다룹니다.

**더 읽기** Part 0 ch9 2.4절 / Part 1 ch8 2.7·2.10절

### 한눈에 보기

| 다른 언어 | Rust | 질문 |
| --- | --- | --- |
| `null`, `T?`, `Optional<T>` | `Option<T>`의 `None` | Q5.1, Q5.2 |
| `?.` | `map`·`and_then`, 함수 안에서는 `?` | Q5.2, Q5.5 |
| `?:`, `??`, `orElse` | `unwrap_or` | Q5.2 |
| `!!`, `!`, `Optional.get()` | `expect`, `unwrap` | Q5.2, Q5.8 |
| `throw` | `return Err(…)` | Q5.3 |
| `throws` 절 | 반환 타입 `Result<T, E>` | Q5.4 |
| `catch` | `match`, `if let` | Q5.6 |
| 잡지 않고 위로 넘기기 | `?` | Q5.5 |
| `finally`, `defer`, `with`, `using` | `Drop` | Q5.6 |
| 버그를 알리는 unchecked 예외, Go `panic` | `panic!` | Q5.7 |
| 최상위 `catch`, Go `recover` | 경계의 `catch_unwind` | Q5.7 |
| `printStackTrace`, 트레이스백 | `RUST_BACKTRACE=1`, `Backtrace` | Q5.10 |
| 예외 클래스 계층 | 에러 열거형, `Box<dyn Error>` | Q5.9 |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_null_safety_operators](examples/01_null_safety_operators.rs) | Kotlin·TS의 `?.`, `?:`, `!!`, null 검사를 `and_then`·`map`, `unwrap_or`, `expect`, `if let`, `?`로 옮기기 | Q5.1, Q5.2, Q5.5, Q5.8 |
| [02_errors_are_values](examples/02_errors_are_values.rs) | 에러 값을 보고 다시 시도하기, 다시 시도할 수 없는 에러는 바로 돌려주기 | Q5.3, Q5.4, Q5.9 |
| [03_cleanup_like_finally](examples/03_cleanup_like_finally.rs) | 잠금 가드가 정상 종료와 `?`로 인한 조기 반환에서 모두 풀림, 반복문에서는 한 바퀴마다 풀림 | Q5.6 |

### 01_null_safety_operators — null 연산자를 Option으로

```sh
cargo run -p faq-ch5-no-null-no-exceptions --example 01_null_safety_operators
```

```text
1. ?. 사슬과 ?: 기본값
  회원 1: 부산
  회원 2: 미등록
  회원 3: 미등록
2. 같은 일을 ?로
  회원 1: Some("부산")
  회원 2: None
  회원 3: None
3. if (user != null) → if let
  서아님은 주소 등록 여부: false
4. !! → expect (없을 리 없을 때만)
  하준
```

- `db.get(&id)`는 `HashMap`에 키가 없으면 `None`을 돌려줍니다. Java의 `Map.get`이 null을 돌려주는 자리입니다.
- `u.address.as_ref()`는 `&Option<Address>`를 `Option<&Address>`로 바꿔, 주소를 빌린 채로 사슬을 이어 가게 합니다(Part 1 ch2 예제 05).
- 1번과 2번은 같은 조회를 메서드 체인(`city_chain`)과 `?`(`city_question`)로 쓴 것입니다.

### 02_errors_are_values — 에러 값으로 할 수 있는 일

```sh
cargo run -p faq-ch5-no-null-no-exceptions --example 02_errors_are_values
```

```text
1. 에러 값을 보고 다시 시도하기
  1번째 시도: 시간 초과, 다시 시도
  2번째 시도: 시간 초과, 다시 시도
  결과: Ok(8080)
2. 다시 시도할 수 없는 에러는 바로 돌려줌
  결과: Err(NotFound)
```

- `fetch_config`는 실제 네트워크 대신 시도 횟수로 결과를 정하므로 출력이 실행마다 같습니다.
- `Err(FetchError::Timeout) if attempt < max`는 매치 가드(match guard, Part 0 ch18)로, 시간 초과이면서 아직 다시 시도할 수 있을 때만 그 갈래(match arm)로 갑니다.

### 03_cleanup_like_finally — finally 자리의 Drop

```sh
cargo run -p faq-ch5-no-null-no-exceptions --example 03_cleanup_like_finally
```

출력은 [Q5.6](#q56-trycatchfinally에-해당하는-것은-무엇인가요)에 실었습니다.

- `log.try_lock()`은 기다리지 않고 잠금을 시도해, 이미 잠겨 있으면 `Err`를 돌려줍니다. `is_ok()`가 `false`면 잠겨 있다는 뜻입니다. 블록 끝이나 `drop(guard)`에서 가드가 풀리는 것은 Part 0 ch15 예제 05가 이미 보였고, 이 예제가 더하는 것은 `?`로 일찍 돌아가는 길과 Go `defer`와의 정리 단위 차이입니다.
- `println!` 안의 `log.try_lock()`이 성공해 만든 가드는 그 문장이 끝날 때 버려지므로, 다음 줄의 `lock`을 막지 않습니다.
- `record`는 `lock`의 `Err`를 `unwrap`으로 넘겼습니다. `lock`은 다른 스레드가 잠금을 쥔 채 패닉한 경우(포이즈닝(poisoning), Part 0 ch15 2.7절)에만 `Err`를 돌려주는데, 이 예제는 스레드 하나로 돌아 그런 일이 없기 때문입니다.

## 4. 자주 하는 실수

### 4.1 없을 수도 있는 값에 바로 메서드를 부름 — E0599

파일: [`compile_fail/e0599_method_on_option.rs`](compile_fail/e0599_method_on_option.rs)

```text
error[E0599]: no method named `to_uppercase` found for enum `Option<T>` in the current scope
  --> compile_fail/e0599_method_on_option.rs:28:29
   |
28 |     println!("{}", nickname.to_uppercase()); // E0599
   |                             ^^^^^^^^^^^^ method not found in `Option<String>`
   |
note: the method `to_uppercase` exists on the type `String`
  --> /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/alloc/src/str.rs:656:4
help: consider using `Option::expect` to unwrap the `String` value, panicking if the value is an `Option::None`
   |
28 |     println!("{}", nickname.expect("REASON").to_uppercase()); // E0599
   |                            +++++++++++++++++
```

- **원인**: Kotlin의 `nickname?.uppercase()`를 옮기며 `?.` 자리에 `.`만 썼습니다. `nickname`은 `String`이 아니라 `Option<String>`이고, `Option`에는 `to_uppercase`가 없습니다. 컴파일러는 그 메서드가 안의 `String`에 있다고 알려 줍니다(`note`). `Option<T>`를 `T`처럼 더하면 E0277이 나는 것은 Part 0 ch6 4.2절에서 봤고, 메서드를 부르면 이렇게 E0599가 됩니다.
- **해결**: 컴파일러의 `help`는 `expect`를 제안하지만, 그것은 Kotlin의 `!!`에 해당합니다(없으면 패닉). `?.`와 같은 뜻으로 쓰려면 `nickname.as_deref().map(str::to_uppercase)`(결과는 `Option<String>`), 기본값까지 정하려면 `map_or(String::new(), str::to_uppercase)`를 씁니다.
- 덧붙임: 같은 실험을 `len()`으로 하면 이 에러가 아니라 E0624(``method `len` is private``)가 납니다. 1.100.0의 `Option`에 바깥에서 쓸 수 없는 같은 이름의 메서드가 있기 때문입니다.

### 4.2 `Result`를 쓰지 않고 버림 — `unused_must_use` 린트

파일: [`compile_fail/unused_must_use_result.rs`](compile_fail/unused_must_use_result.rs)

```text
error: unused `Result` that must be used
  --> compile_fail/unused_must_use_result.rs:25:5
   |
25 |     save(""); // 실패했는지 확인하지 않고 버림
   |     ^^^^^^^^
   |
   = note: this `Result` may be an `Err` variant, which should be handled
note: the lint level is defined here
  --> compile_fail/unused_must_use_result.rs:14:9
   |
14 | #![deny(unused_must_use)]
   |         ^^^^^^^^^^^^^^^
help: use `let _ = ...` to ignore the resulting value
   |
25 |     let _ = save(""); // 실패했는지 확인하지 않고 버림
   |     +++++++
```

- **원인**: `save`가 실패를 `Result`로 알리는데, 부르는 쪽이 확인하지 않고 버렸습니다. Java의 빈 `catch` 블록이나 Go에서 `err`를 확인하지 않는 것과 같은 실수입니다. 기본 경고는 Part 0 ch9 2.5절에서 봤고, 이 파일은 맨 위의 `#![deny(unused_must_use)]`로 그 경고를 에러로 올렸습니다. 린트 에러라서 에러 코드는 없습니다.
- **해결**: 실패를 처리하거나(`if let Err(e) = save("") { … }`), 부르는 쪽에 넘기거나(`save("")?`), 정말 무시해도 되면 `let _ = save("");`로 무시한다는 것을 코드에 적습니다. 컴파일러의 `help`는 마지막 방법을 제안하지만, 무시해도 되는지 먼저 따져야 합니다.

## 5. 연습 문제

### 연습 1. 도시 이름의 길이

01의 `city_chain`을 써서, 회원의 도시 이름 글자 수를 돌려주는 `fn city_len(db: &HashMap<u32, User>, id: u32) -> usize`를 만들어 보세요. 도시가 없으면 0을 돌려줍니다. Kotlin이라면 `city?.length ?: 0`인 자리입니다.

<details>
<summary>힌트</summary>

`city_chain(db, id).map_or(0, |c| c.chars().count())`처럼 `map_or`로 `?.`와 `?:`를 한 번에 씁니다. 바이트 수가 아니라 글자 수가 필요하므로 `len()` 대신 `chars().count()`를 씁니다(Part 0 ch8). 회원 1이면 2("부산"), 회원 2·3이면 0입니다.

</details>

### 연습 2. 다시 시도 횟수를 결과에 담기

02의 `fetch_with_retry`를 고쳐, 성공하면 값과 함께 몇 번째 시도에서 성공했는지(`Ok((8080, 3))`)를 돌려주게 해 보세요. 에러일 때는 그대로 `Err`를 돌려줍니다.

<details>
<summary>힌트</summary>

반환 타입을 `Result<(u32, u32), FetchError>`로 바꾸고, 반복문의 `other => return other` 갈래를 `Ok(value) => return Ok((value, attempt))`와 `Err(e) => return Err(e)`로 나눕니다. `main`은 고치지 않아도 됩니다.

</details>

### 연습 3. 잠금을 일찍 풀기

03의 `record`에서 기록을 덧붙인 뒤에도 잠금이 필요 없는 일이 더 이어진다고 해 봅시다. 기록 수를 구한 직후 잠금을 풀도록 고치고, 그다음 줄에서 `log.try_lock().is_ok()`를 출력해 `true`인지 확인해 보세요. Go라면 `defer` 대신 `mu.Unlock()`을 직접 부르는 자리입니다.

<details>
<summary>힌트</summary>

`let count = entries.len();`로 기록 수를 먼저 꺼내 두고 `drop(entries);`로 가드를 일찍 버립니다(Part 0 ch14 2.6절의 `std::mem::drop`). 그 뒤의 `println!`에서 `try_lock().is_ok()`는 `true`이고, 함수는 `Ok(count)`를 돌려줍니다. `drop(entries)` 뒤에 `entries`를 다시 쓰면 E0382가 납니다. 반대로 `let _ = log.lock().unwrap();`처럼 이름 없이 잠그면 rustc가 컴파일을 거부합니다(Q5.6). `drop(guard)`로 일찍 푸는 것은 Part 0 ch15 예제 05와 같은 연습으로, 여기서는 Go의 `defer`를 쓰지 않는 자리와 짝지어 봅니다.

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- [The Rust Programming Language 6.1절: The `Option` Enum and Its Advantages Over Null Values](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum-and-its-advantages-over-null-values)
- [The Rust Programming Language 9장: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- 표준 라이브러리 문서: [`std::option`](https://doc.rust-lang.org/std/option/index.html#representation)(Representation), [`std::result`](https://doc.rust-lang.org/std/result/index.html)(`#[must_use]`), [`std::alloc::handle_alloc_error`](https://doc.rust-lang.org/std/alloc/fn.handle_alloc_error.html), [`std::panic::catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html), [`std::backtrace::Backtrace`](https://doc.rust-lang.org/std/backtrace/struct.Backtrace.html)
- 표준 라이브러리 문서(자원): [`std::fs::File`](https://doc.rust-lang.org/std/fs/struct.File.html), [`std::sync::MutexGuard`](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html), [`Mutex::lock`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock), [`std::net::TcpStream`](https://doc.rust-lang.org/std/net/struct.TcpStream.html)

다른 언어

- Java: [`Optional`](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Optional.html), [`Throwable`](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Throwable.html), [Java Language Specification 11장: Exceptions](https://docs.oracle.com/javase/specs/jls/se21/html/jls-11.html), [14.20.3: try-with-resources](https://docs.oracle.com/javase/specs/jls/se21/html/jls-14.html#jls-14.20.3), [15.10.4: Run-Time Evaluation of Array Access Expressions](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.10.4), [`Lock`](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/concurrent/locks/Lock.html)
- Kotlin: [Null safety](https://kotlinlang.org/docs/null-safety.html)
- TypeScript: [Everyday Types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html)(`strictNullChecks`, `!`), [TypeScript 3.7](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-7.html)(`?.`, `??`)
- Go: [Errors are values](https://go.dev/blog/errors-are-values) (Rob Pike, 2015-01-12), [`errors` 패키지](https://pkg.go.dev/errors), [명세: Defer statements](https://go.dev/ref/spec#Defer_statements), [Handling panics](https://go.dev/ref/spec#Handling_panics), [Expression statements](https://go.dev/ref/spec#Expression_statements), [Index expressions](https://go.dev/ref/spec#Index_expressions), [`sync.Mutex`](https://pkg.go.dev/sync#Mutex.Lock)
- Python: [The `with` statement](https://docs.python.org/3/reference/compound_stmts.html#the-with-statement), [Built-in Exceptions](https://docs.python.org/3/library/exceptions.html)(`__traceback__`), [`threading`: Using locks, conditions, and semaphores in the `with` statement](https://docs.python.org/3/library/threading.html#with-locks)
- C#: [The `using` statement](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/using)
