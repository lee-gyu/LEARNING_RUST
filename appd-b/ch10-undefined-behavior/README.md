# ch10. 미정의 동작

C와 C++의 미정의 동작(undefined behavior)은 "결과가 틀린다"가 아니라 "언어가 이 프로그램에 아무 약속도 하지 않는다"는 뜻입니다. 부호 있는 정수의 오버플로(overflow), 폭을 넘는 시프트, 0으로 나누기, 초기화하지 않은 변수 읽기, 순서가 정해지지 않은 두 수정이 모두 여기에 듭니다(다만 C++26부터 초기화하지 않은 지역 변수를 읽는 것은 미정의 동작이 아니라 오류 동작입니다. [ch12](../ch12-modern-cpp-enough/README.md) Q12.4). 이 챕터는 그런 경우를 C·C++ 코드로 만들어 새니타이저(sanitizer)인 UndefinedBehaviorSanitizer(UBSan)와 컴파일러 경고가 무엇을 알리는지 보고, Rust에서는 같은 연산이 어떻게 정해져 있는지, "안전한 Rust에는 미정의 동작이 없다"는 말이 정확히 무엇을 뜻하는지 봅니다.

- 패키지: `faq-ch10-undefined-behavior`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 비교 코드를 확인한 컴파일러: Apple clang 21.0.0, macOS(aarch64). 다른 컴파일러에서는 옵션과 메시지가 다릅니다.
- 쓸 수 없던 도구: Miri(설치하지 않음). `unsafe` 코드의 미정의 동작을 실행으로 보이는 예제는 두지 않고, Reference의 규칙으로 설명합니다(Q10.9).
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q10.1](#q101-미정의-동작이-무엇이고-왜-틀린-결과보다-위험한가요) | 미정의 동작이 무엇이고 왜 "틀린 결과"보다 위험한가요? | 언어가 그 프로그램 전체의 동작을 정하지 않는다는 뜻입니다. 컴파일러가 "일어나지 않는다"고 가정하고 코드를 바꿀 수 있어 버그가 엉뚱한 곳에서 드러납니다. |
| [Q10.2](#q102-안전한-rust에는-미정의-동작이-없다는-말은-무슨-뜻인가요) | "안전한 Rust에는 미정의 동작이 없다"는 말은 무슨 뜻인가요? | `unsafe`를 쓰지 않은 코드로는 Reference가 정한 미정의 동작을 만들 수 없다는 뜻입니다. 버그나 패닉이 없다는 뜻은 아닙니다. |
| [Q10.3](#q103-부호-있는-정수의-오버플로는-어떻게-다른가요) | 부호 있는 정수의 오버플로는 어떻게 다른가요? | C에서는 미정의 동작이고, Rust에서는 디버그 빌드는 패닉·릴리스 빌드는 래핑으로 정해진 에러입니다. |
| [Q10.4](#q104-암묵적-변환-정수-승격-부호-있는-수와-없는-수의-비교-문제는-어떻게-되나요) | 암묵적 변환, 정수 승격, 부호 있는 수와 없는 수의 비교 문제는 어떻게 되나요? | 자동 변환이 없어서 부호가 다른 비교는 컴파일 에러이고, 정수는 자기 타입 그대로 계산됩니다. |
| [Q10.5](#q105-0으로-나누기-폭을-넘는-시프트-switch의-fallthrough-값을-돌려주지-않고-끝나는-함수는-어떻게-되나요) | 0으로 나누기, 폭을 넘는 시프트, `switch`의 fallthrough, 값을 돌려주지 않고 끝나는 함수는 어떻게 되나요? | 나누기는 패닉, 시프트는 디버그 패닉·릴리스 래핑으로 정해져 있고, fallthrough와 빠진 반환은 문법·타입 검사로 생기지 않습니다. |
| [Q10.6](#q106-i--i--1-같은-평가-순서-문제와-전역-객체의-초기화-순서-문제는-어떻게-되나요) | `i = i++ + 1` 같은 평가 순서 문제와 전역 객체의 초기화 순서 문제는 어떻게 되나요? | 피연산자는 왼쪽에서 오른쪽으로 평가된다고 정해져 있고, `static`의 초기값은 컴파일 시간에 계산됩니다. |
| [Q10.7](#q107-엄격한-별칭-규칙strict-aliasing과-restrict에-해당하는-것은-무엇인가요) | 엄격한 별칭 규칙(strict aliasing)과 `restrict`에 해당하는 것은 무엇인가요? | `&mut T`가 `restrict`처럼 "다른 누구도 이 메모리에 닿지 않는다"를 보장하고, 대여 검사기가 안전한 코드에서 그것을 지킵니다. |
| [Q10.8](#q108-bool이나-열거형에-잘못된-값이-들어가는-것-유니언으로-타입을-바꿔-읽는-것은-어떻게-되나요) | `bool`이나 열거형에 잘못된 값이 들어가는 것, 유니언으로 타입을 바꿔 읽는 것은 어떻게 되나요? | 안전한 코드로는 잘못된 값을 만들 수 없고, 정수를 열거형으로 바꾸는 `as`는 없습니다. 비트로 읽기는 `from_bits` 같은 안전한 함수로 합니다. |
| [Q10.9](#q109-unsafe-rust에서는-미정의-동작이-가능한가요-c보다-쉬운가요-어려운가요) | `unsafe` Rust에서는 미정의 동작이 가능한가요? C보다 쉬운가요, 어려운가요? | 가능합니다. 규칙 일부는 C보다 엄격하지만, 미정의 동작이 생길 수 있는 곳이 `unsafe` 블록으로 좁혀집니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md): 정수 타입과 정수 오버플로(4절, 예제 04)
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.1~2.3절: `unsafe`, 원시 포인터, 안전한 추상화
- [ch3](../ch3-safety-limits/README.md) Q3.2(`unsafe`와 C), Q3.6(Rust의 정수 오버플로)
- [ch9](../ch9-c-memory-errors/README.md): C·C++ 비교 코드와 새니타이저 읽는 법

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 미정의 동작이 "틀린 결과"와 어떻게 다른지 설명하고, "안전한 Rust에는 미정의 동작이 없다"가 약속하는 것과 약속하지 않는 것을 구별할 수 있다.
2. C에서 미정의 동작인 부호 있는 오버플로·폭을 넘는 시프트·0으로 나누기·`INT_MIN / -1`이 Rust에서 어떻게 정해져 있는지 설명하고, 원하는 결과를 `checked_*`·`wrapping_*` 메서드로 고를 수 있다.
3. 정수 승격과 부호가 다른 비교, 평가 순서, 전역 초기화 순서에 대해 C와 Rust의 규칙을 비교하고, 같은 일을 Rust에서 명시적으로 쓸 수 있다.
4. `&mut`의 별칭 보장, 잘못된 값(`bool`, 열거형), 비트로 다른 타입 읽기에서 안전한 Rust가 무엇을 막는지와, `unsafe`에서는 다시 프로그래머의 책임이 된다는 것을 설명할 수 있다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다. 이 챕터의 "다른 언어에서는"은 주로 C·C++입니다. 비교 코드의 실행 방법과 보고 전체는 3절에 모았습니다.

### Q10.1 미정의 동작이 무엇이고 왜 "틀린 결과"보다 위험한가요?

**짧은 답** 미정의 동작은 언어 명세가 "이런 프로그램의 동작은 정하지 않는다"고 한 경우입니다. 틀린 값 하나가 나오는 것으로 끝나지 않을 수 있습니다. 컴파일러는 미정의 동작이 일어나지 않는다고 가정하고 최적화하므로, 그 가정이 깨지면 검사가 사라지거나 엉뚱한 곳에서 이상한 동작이 나타납니다. Rust Reference도 미정의 동작이 프로그램 전체에 영향을 준다고 적습니다.

**왜 그런가** "틀린 결과"라면 그 값을 쓰는 곳만 보면 됩니다. 미정의 동작은 그렇지 않습니다. 예를 들어 이 환경의 clang은 `int bigger(int x) { return x + 1 > x; }`를 `cc -O2 -S`로 컴파일하면 인자와 상관없이 1을 돌려주는 코드(`mov w0, #1`)로 바꿨고, `-O0`에서는 실제로 비교했습니다. `x`가 `INT_MAX`이면 수학적으로는 거짓이어야 할 자리지만, 부호 있는 정수의 오버플로는 일어나지 않는다고 가정해도 되기 때문입니다. 이런 식으로 오버플로를 검사하려고 쓴 코드 자체가 사라질 수 있습니다. 그래서 같은 코드가 최적화 수준이나 컴파일러 버전에 따라 다르게 동작합니다. 이 챕터의 비교 코드가 새니타이저 없이 실행한 출력을 싣지 않는 것도 그 때문입니다.

**다른 언어에서는** C에서 초기화하지 않은 지역 변수를 읽는 [compare/05_uninit_read.c](compare/05_uninit_read.c)는 clang이 `-Wall`을 켜야 `-Wsometimes-uninitialized` 경고를 내고, 실행 중에는 이 환경의 어떤 새니타이저도 보고하지 않았습니다(ASan과 UBSan은 이것을 검사하지 않고, 검사하는 MemorySanitizer는 이 Apple clang에서 쓸 수 없음, [ch9](../ch9-c-memory-errors/README.md) Q9.5). Java는 지역 변수를 값이 확실히 대입되기 전에 읽으면 컴파일 에러로 막고(Java 언어 명세 16장, definite assignment), 정수 오버플로는 래핑으로 정해 두었습니다(ch3 Q3.6). Rust의 E0381([ch2](../ch2-memory-safety/README.md) 4.1절)과 같은 쪽입니다.

**오해와 한계** 미정의 동작이 있는 프로그램이 "지금은 잘 돈다"는 것은 아무것도 보장하지 않습니다. 다음 빌드에서 다르게 돌 수 있습니다. Rust도 `unsafe` 코드나 FFI로 부른 C 코드에서 미정의 동작이 생기면 프로그램 전체가 영향을 받습니다(Q10.9).

**더 읽기** ch2 Q2.2 / ch9 Q9.5 / Part 9 `ch10-unsafety-const-asm`

### Q10.2 "안전한 Rust에는 미정의 동작이 없다"는 말은 무슨 뜻인가요?

**짧은 답** `unsafe`를 쓰지 않은 코드로는 Rust Reference의 "Behavior considered undefined" 목록에 있는 일(데이터 경쟁, 댕글링 포인터 역참조, 별칭 규칙 위반, 잘못된 값(invalid value) 만들기 등)을 할 수 없다는 뜻입니다. 단, 그 코드가 부르는 `unsafe` 코드(표준 라이브러리 포함)가 올바르다는 전제가 붙습니다. 버그·패닉·누수·교착 상태가 없다는 뜻은 아닙니다.

**왜 그런가** Reference는 따로 "Behavior not considered unsafe"를 두어 교착 상태, 메모리와 자원의 누수, 소멸자 없이 종료, 포인터 누출로 무작위 주소 드러내기, 정수 오버플로, 논리 오류의 여섯 가지를 안전한 코드에서도 생길 수 있는 일로 분류합니다. 이것들은 결과가 틀릴 수는 있어도 메모리를 망가뜨리지 않고 언어의 약속을 깨지 않습니다. 반면 미정의 동작 목록에 있는 일은 `unsafe` 블록 안에서만 할 수 있는 연산(원시 포인터 역참조, `unsafe` 함수 호출 등)을 거쳐야 생깁니다(Nomicon "What Unsafe Rust Can Do").

**다른 언어에서는** C와 C++에는 이런 경계가 없어서, 어느 줄에서든 미정의 동작이 생길 수 있습니다. C++ 표준의 안전성 작업은 [ch12](../ch12-modern-cpp-enough/README.md)에서 봅니다.

**오해와 한계** 이 보장은 컴파일러와 `unsafe` 코드에 버그가 없다는 전제 위에 있습니다([ch3](../ch3-safety-limits/README.md) Q3.3·Q3.10). 또 "미정의 동작이 없다"가 "패닉이 없다"는 뜻은 아니어서, 정해진 방식으로 멈추는 것도 서비스에는 장애일 수 있습니다(ch3 Q3.7).

**더 읽기** ch3 Q3.1~Q3.4 / Part 10 `ch1-safe-and-unsafe`

### Q10.3 부호 있는 정수의 오버플로는 어떻게 다른가요?

**짧은 답** C에서 부호 있는 정수의 오버플로는 미정의 동작이고, 부호 없는 정수의 오버플로는 정의된 래핑(wrapping)입니다. Rust에서는 둘 다 미정의 동작이 아닙니다. Reference는 산술 오버플로를 프로그래머의 에러로 보고, 디버그 빌드는 패닉, 그 밖의 빌드는 패닉이나 래핑 가운데 구현이 고르게 합니다(지금의 릴리스 빌드는 래핑). 의도한 래핑은 `wrapping_add` 같은 메서드로 적습니다.

**왜 그런가** [compare/01_signed_overflow.c](compare/01_signed_overflow.c)는 `INT_MAX + 1`을 계산하고, UBSan이 `signed integer overflow: 2147483647 + 1 cannot be represented in type 'int'`를 보고합니다. UBSan 문서는 부호 없는 정수의 오버플로에 대해서는 "미정의 동작은 아니지만 흔히 의도하지 않은 것"이라고 따로 적습니다. Rust가 오버플로를 미정의 동작으로 두지 않는 이유는 그것이 메모리 안전성을 깨지 않게 하려는 것이고, 디버그 빌드에서 검사하는 것은 실수를 일찍 찾으려는 것입니다. Rust의 오버플로 규칙과 메서드 네 종류는 Part 0 ch3 예제 04, 그것이 메모리 안전성과 어떻게 이어지는지는 ch3 Q3.6에서 봤습니다.

**다른 언어에서는** Java와 Go는 정수 오버플로를 래핑으로 정해 두었습니다(ch3 Q3.6). C에서는 `-fwrapv` 같은 컴파일러 옵션으로 래핑을 강제할 수 있지만 언어의 기본은 아닙니다(UBSan 문서는 `-fwrapv`를 켜도 검사가 들어간다고 적습니다).

**오해와 한계** 릴리스 빌드의 래핑은 미정의 동작이 아닐 뿐, 틀린 값이라는 점은 같습니다. 크기나 길이처럼 오버플로가 나면 안 되는 계산은 `checked_*`로 쓰고, 릴리스에서도 검사하려면 프로필의 `overflow-checks`를 켭니다(ch3 Q3.6).

**더 읽기** Part 0 ch3 4절, 예제 04 / ch3 Q3.6, 예제 03

### Q10.4 암묵적 변환, 정수 승격, 부호 있는 수와 없는 수의 비교 문제는 어떻게 되나요?

**짧은 답** Rust에는 수 타입 사이의 암묵적 변환이 없습니다. 그래서 `i32`와 `u32`를 바로 비교하면 컴파일 에러(E0308)이고(4.1절), `u8`끼리의 덧셈은 C처럼 `int`로 승격(integer promotion)되지 않고 `u8` 범위에서 계산됩니다. 어떤 변환이 필요한지는 `i64::from`, `as`, `try_from` 가운데 골라 적습니다.

**왜 그런가** C에서는 미정의 동작이 아니어도 결과가 예상과 다르게 정해져 있는 경우가 많습니다. [compare/04_sign_compare.c](compare/04_sign_compare.c)는 이 환경에서 다음을 출력했습니다.

```text
-1 < 1u: 0
200 + 100 (unsigned char): 300
```

`int`와 `unsigned`를 비교하면 `int` 쪽이 `unsigned`로 바뀌어 -1이 아주 큰 수가 되고, `unsigned char`끼리는 먼저 `int`로 승격되어 300이 됩니다. clang은 첫 비교에 `-Wsign-compare` 경고를 내지만 `-Wextra`를 켜야 나옵니다. [02_conversions_and_bits](examples/02_conversions_and_bits.rs)는 같은 일을 Rust로 씁니다.

```text
1. 정수 승격 대신 명시적 확장
  200u8.checked_add(100) = None
  u16::from(a) + u16::from(b) = 300
2. 부호가 다른 수의 비교
  i64로 넓혀 비교: true
  as u32로 C처럼: false (balance as u32 = 4294967295)
  u32::try_from(balance) = Err(TryFromIntError(NegOverflow))
```

둘 다 담을 수 있는 `i64`로 넓히면 수학적으로 맞는 답이 나오고, `as`로 바꾸면 C와 같은 답이 나옵니다. 어느 쪽인지가 코드에 드러납니다. `try_from`의 `NegOverflow`와 `as`가 값을 조용히 바꾸는 것은 ch2 예제 04에서 봤고, 여기서는 그것을 C의 승격·부호 비교 규칙과 나란히 놓았습니다.

**다른 언어에서는** Java는 부호 없는 정수 타입이 16비트 `char` 하나뿐이고 연산 전에 `int`로 승격되므로 이 비교 문제가 거의 없고([ch7](../ch7-syntax-type-surprises/README.md) Q7.3), Go는 Rust처럼 암묵적 수 변환이 없습니다(ch7 Q7.4).

**오해와 한계** `as`는 여전히 값을 조용히 바꿀 수 있습니다(-1 → 4294967295). 바뀌면 안 되는 변환은 `try_from`으로 실패를 받습니다([부록 A ch5](../../appd-a/ch5-type-conversions/README.md) 2.4절). 변환을 적는 수고는 ch7 Q7.4에서 본 비용과 같습니다.

**더 읽기** ch7 Q7.3·Q7.4 / [부록 A ch1](../../appd-a/ch1-express-data-structures/README.md) 4.1절, 부록 A ch5 2.4·2.5절

### Q10.5 0으로 나누기, 폭을 넘는 시프트, `switch`의 fallthrough, 값을 돌려주지 않고 끝나는 함수는 어떻게 되나요?

**짧은 답** 넷 다 C나 C++에서는 미정의 동작이거나(앞의 둘과 마지막) 흔한 버그(fallthrough)지만, Rust에서는 모두 정해져 있습니다.

- 0으로 나누기와 `i32::MIN / -1`: 빌드와 상관없이 패닉합니다. Reference는 정수의 0으로 나누기가 패닉한다고 따로 정하고, `MIN / -1`의 오버플로 검사는 `-C overflow-checks`를 꺼도 남는다고 적습니다.
- 폭 이상의 시프트: 실행 중에 정해진 `n = 32`로 `1u32 << n`을 하면 디버그 빌드는 패닉했고, 이 환경의 릴리스 빌드는 시프트 양을 폭으로 나눈 나머지만큼 밀어 1이 됐습니다. `1u32 << 32`처럼 값이 상수면 `arithmetic_overflow` 린트(기본 deny)가, `10 / 0`이면 `unconditional_panic` 린트(기본 deny)가 컴파일 에러로 막습니다.
- fallthrough: `match`의 갈래는 다음 갈래로 넘어가지 않습니다(ch7 Q7.9).
- 값을 돌려주지 않는 경로: 반환 타입의 값을 내지 않는 경로가 있으면 컴파일 에러입니다.

**왜 그런가** UBSan은 [compare/02_shift_width.c](compare/02_shift_width.c)에서 `shift exponent 32 is too large for 32-bit type 'unsigned int'`를, [compare/03_division.c](compare/03_division.c)에서 `division of -2147483648 by -1 cannot be represented in type 'int'`와 `division by zero`를 보고했습니다. [compare/07_missing_return.cpp](compare/07_missing_return.cpp)는 C++에서 `return` 없이 함수 끝에 닿아 `execution reached the end of a value-returning function without returning a value`가 보고됐습니다. [01_defined_arithmetic](examples/01_defined_arithmetic.rs)는 Rust에서 결과를 고르는 메서드를 봅니다.

```text
1. 시프트
  1u32.checked_shl(31) = Some(2147483648)
  1u32.checked_shl(32) = None
  1u32.wrapping_shl(32) = 1
  1u32.wrapping_shl(33) = 2
2. 나눗셈
  10i32.checked_div(0) = None
  i32::MIN.checked_div(-1) = None
  i32::MIN.wrapping_div(-1) = -2147483648
  i32::MIN.overflowing_div(-1) = (-2147483648, true)
```

`wrapping_shl`은 시프트 양을 폭으로 나눈 나머지(32 → 0, 33 → 1)만큼 밉니다. 폭 이상이면 0을 원하면 `unbounded_shl`, 빌드와 상관없이 언제나 패닉하길 원하면 `strict_shl`이 있습니다(이 환경에서 `1u32.unbounded_shl(32)`는 0). 0으로 나누기에는 래핑할 결과가 없으므로 `wrapping_div(0)`도 패닉합니다(이 환경에서 확인). `checked_div(0)` 자체는 [Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md) 예제 03에서 봤고, 여기서 더하는 것은 `MIN / -1`과 시프트입니다.

**다른 언어에서는** clang은 값을 돌려주지 않는 경로에 `-Wreturn-type` 경고를 기본으로 내지만 경고일 뿐입니다. C·C++의 콜론 형식 `switch`가 다음 레이블로 넘어가는 것은 미정의 동작은 아니지만, Java 언어 명세도 미묘한 버그의 원인이 될 수 있다고 적습니다(ch7 Q7.9).

**오해와 한계** 0으로 나누기 패닉은 디버그와 릴리스 모두에서 일어나므로, 외부 입력으로 나누는 코드는 `checked_div`로 쓰거나 0을 먼저 검사합니다. 패닉을 `catch_unwind`로 잡는 것은 흐름 제어용이 아닙니다([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 예제 06, [ch5](../ch5-no-null-no-exceptions/README.md) Q5.7).

**더 읽기** ch7 Q7.9 / Part 1 ch8 예제 06 / Part 0 ch3의 "자주 하는 실수" 3

### Q10.6 `i = i++ + 1` 같은 평가 순서 문제와 전역 객체의 초기화 순서 문제는 어떻게 되나요?

**짧은 답** Rust Reference는 피연산자를 소스에 적힌 대로 왼쪽에서 오른쪽으로 평가한다고 평가 순서(evaluation order)를 정합니다. 대입식(과 원시 타입끼리의 복합 대입)은 대입할 값을 먼저, 대입받을 자리를 나중에 평가한다고 따로 정합니다. `++`가 없어 `i = i++ + 1` 같은 식은 쓸 수 없습니다. `static`의 초기값은 컴파일 시간에 계산되는 상수 식(constant expression)이라서, 실행 중에 "어느 전역이 먼저 초기화되나"를 따질 일이 없습니다.

**왜 그런가** [03_evaluation_order](examples/03_evaluation_order.rs)는 평가되는 순간 이름을 출력하는 함수로 순서를 봅니다.

```text
1. 함수 인자와 이항 연산
    첫째 인자
    둘째 인자
    왼쪽 피연산자
    오른쪽 피연산자
  (1, 2), 30
2. 대입
    값
    자리(인덱스)
  [0, 7, 0]
3. 컴파일 시간에 정해진 static
  SQUARES = [0, 1, 4, 9]
```

`SQUARES`의 초기값은 `const fn`(컴파일 시간에도 실행할 수 있는 함수, Part 9 `ch10-unsafety-const-asm`에서 다룸)을 불러 만든 배열이고, Reference에 따르면 `static`의 초기값은 컴파일 시간에 평가되는 상수 식입니다. 실행 중에 만들어야 하는 전역은 처음 쓸 때 한 번만 초기화하는 `LazyLock`으로 만듭니다([Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 예제 08).

**다른 언어에서는** [compare/06_unsequenced.c](compare/06_unsequenced.c)의 `i = i++ + 1`은 한 식에서 `i`를 두 번 고치는데 그 순서를 C가 정하지 않아 미정의 동작이고, clang이 `multiple unsequenced modifications to 'i' [-Wunsequenced]` 경고를 기본으로 냈습니다. C++ 표준 초안([basic.start.dynamic])은 같은 번역 단위(translation unit) 안에서 정의 순서를 따르는 경우가 아니면 두 전역 변수의 동적 초기화(dynamic initialization)가 어느 쪽이 먼저인지 정해지지 않은(indeterminately sequenced) 관계가 된다고 정합니다. 그래서 다른 파일에 있는 전역끼리 초기화 코드에서 서로를 쓰면, 아직 초기화되지 않은 쪽을 읽는 문제가 생길 수 있습니다.

**오해와 한계** 평가 순서가 정해져 있다고 부작용에 기대는 코드가 읽기 좋은 것은 아닙니다. 대입식처럼 오른쪽을 먼저 평가하는 예외도 있으므로, 순서가 중요한 부작용은 문장을 나눠 씁니다. `static`의 초기값에 `const`가 아닌 함수를 부르면 E0015로 막힙니다(Part 1 ch11 4.4절).

**더 읽기** Reference "Evaluation order of operands" / Part 1 ch11 2.9절, 4.4절 / [ch8](../ch8-concurrency-async/README.md) Q8.10

### Q10.7 엄격한 별칭 규칙(strict aliasing)과 `restrict`에 해당하는 것은 무엇인가요?

**짧은 답** Rust의 `&mut T`는 C의 `restrict`보다 강한 약속입니다. Reference가 정리한 별칭 규칙에 따르면 `&mut T`가 살아 있는 동안 그 메모리는 그 참조에서 나온 것 말고는 어떤 포인터도 읽거나 쓰지 않고, `&T`가 가리키는 메모리는 그동안 바뀌지 않습니다(`UnsafeCell` 안은 예외). 안전한 코드에서는 대여 검사기가 이것을 지키므로 프로그래머가 별칭을 직접 따질 일이 없습니다.

**왜 그런가** 같은 값을 `&mut`와 `&`로 동시에 넘기려 하면 E0502로 막히는 것은 [ch4](../ch4-memory-without-gc/README.md) 4.2절(`append_copy(&mut v, &v)`)에서 봤습니다. 컴파일러가 이 약속을 믿을 수 있으면 "이 쓰기가 저 읽기에 영향을 주지 않는다"고 보고 최적화할 수 있습니다. C의 `restrict`는 C99에 들어온 한정자로, GCC 문서(Restricting Pointer Aliasing)는 그것을 별칭되지 않음을 나타내는 표시로 설명합니다. 다만 `restrict`는 프로그래머가 골라 붙이는 것이고, Rust의 `&mut`는 그 약속을 모든 가변 참조가 타입으로 들고 다닙니다.

**다른 언어에서는** GCC 문서는 `-fstrict-aliasing`(최적화 수준에 따라 켜짐)이 C·C++에서 "한 타입의 객체를 호환되지 않는 다른 타입의 식으로 접근하지 않는다"고 가정하게 한다고 설명합니다. 그래서 `float`로 쓴 메모리를 `int*`로 읽는 코드는 그 가정을 깨고, 최적화된 빌드에서 예상과 다르게 동작할 수 있습니다. Rust의 별칭 규칙은 타입이 아니라 참조의 대여 관계로 정해지고, 다른 타입으로 읽는 일은 Q10.8의 방법으로 합니다.

**오해와 한계** Reference는 정확한 별칭 규칙이 아직 다 정해지지 않았다고 적습니다. 그래서 원시 포인터와 참조를 섞어 쓰는 `unsafe` 코드는 지금 어디까지 허용되는지 판단하기 어렵고, Miri 같은 도구로 실행해 확인하는 방법이 있습니다(Q10.9, Part 0 ch19 2.7절).

**더 읽기** ch4 Q4.5, 4.2절 / Reference "Behavior considered undefined" / Part 10 `ch1-safe-and-unsafe`

### Q10.8 `bool`이나 열거형에 잘못된 값이 들어가는 것, 유니언으로 타입을 바꿔 읽는 것은 어떻게 되나요?

**짧은 답** Reference는 `bool`이 0이나 1이 아닌 값, 열거형이 올바르지 않은 판별값(discriminant)을 갖는 것 같은 "잘못된 값을 만드는 것"을 미정의 동작으로 정합니다. 안전한 코드에는 그런 값을 만들 길이 없습니다. 정수를 열거형으로 바꾸는 `as` 변환은 없고(4.2절의 E0605), `match`나 `TryFrom`으로 바꿉니다. 실수의 비트를 정수로 읽는 일은 `f32::to_bits`·`from_bits` 같은 안전한 함수로 합니다.

**왜 그런가** 02의 3번은 실수의 비트를 읽고 비트에서 실수를 만듭니다.

```text
3. 비트로 다른 타입 읽기
  1.0f32.to_bits() = 0x3f800000
  f32::from_bits(0x4049_0fdb) = 3.1415927
```

같은 일을 `unsafe { std::mem::transmute::<u32, f32>(bits) }`로도 할 수 있지만, 이 환경의 rustc는 `unnecessary_transmutes` 경고와 함께 `f32::from_bits`를 쓰라고 제안했습니다([ch2](../ch2-memory-safety/README.md) 4.2절이 확인한 사실). 유니언의 필드를 읽는 것도 `unsafe`가 필요합니다(Part 0 ch19 2.6절). 참조를 다른 타입의 참조로 바꾸는 `&bits as &f32`는 E0605로 막힙니다(ch2 4.2절). 원시 포인터를 거치면(`&bits as *const u32 as *const f32`) 캐스트는 컴파일되지만 역참조는 `unsafe`(E0133)이고, 정렬과 값의 올바름은 프로그래머의 책임이 됩니다. `from_bits`로 비트에서 실수를 만드는 것은 ch2 예제 04에서도 봤습니다.

**다른 언어에서는** C의 열거형은 정수 타입과 같아서 `enum direction d = 7;`처럼 어느 열거자에도 해당하지 않는 값을 넣어도 미정의 동작이 아니고, 이 환경의 clang은 `-Wall -Wextra`로도 경고하지 않았습니다. C++에서는 열거형이 표현할 수 있는 범위 밖의 값을 읽는 것이 미정의 동작이고, UBSan의 `-fsanitize=enum`이 C++ 코드(`static_cast<direction>(7)`)에서 ``load of value 7, which is not a valid value for type 'direction'``을 보고했습니다(C 코드에서는 보고 없음).

**오해와 한계** `unsafe`에서 `transmute`로 정수를 `bool`이나 열거형으로 바꾸면 잘못된 값을 만들 수 있고, 그 순간 미정의 동작입니다. 외부에서 받은 바이트를 열거형으로 해석할 때는 언제나 검사를 거칩니다.

**더 읽기** ch2 4.2절 / Part 0 ch19 2.6절 / [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)(`TryFrom`)

### Q10.9 `unsafe` Rust에서는 미정의 동작이 가능한가요? C보다 쉬운가요, 어려운가요?

**짧은 답** 가능합니다. Nomicon은 `unsafe`가 풀어 주는 일을 다섯 가지(원시 포인터 역참조, `unsafe` 함수 호출, `unsafe` 트레이트 구현, 가변 정적 변수 접근, 유니언 필드 접근)로 꼽고, 이것들을 잘못 쓰면 미정의 동작이 된다고 설명합니다. 일부 규칙(특히 `&mut`의 별칭 규칙)은 C보다 엄격해서 `unsafe` 코드를 바르게 쓰기가 C보다 까다로운 면이 있지만, 미정의 동작이 생길 수 있는 곳이 `unsafe` 블록으로 좁혀져 살펴볼 범위가 작습니다.

**왜 그런가** 안전한 코드가 지키던 규칙(참조는 살아 있고, 정렬되고, 올바른 값을 가리키며, 별칭 규칙을 지킨다)을 `unsafe` 코드에서는 프로그래머가 직접 지켜야 합니다. Reference의 미정의 동작 목록이 그 규칙의 목록이지만, Reference는 이 목록이 완전하지 않고 늘거나 줄 수 있으며 `unsafe` 코드에서 무엇이 허용되는지에 대한 형식 모델이 아직 없다고 경고합니다. 이 저장소의 툴체인에는 Miri가 없어 `unsafe` 코드의 미정의 동작을 실행으로 보이는 예제는 두지 않았습니다. Miri는 nightly 툴체인에서 `unsafe` 코드를 해석하며 이런 위반을 찾는 도구입니다(Part 0 ch19 2.7절).

**다른 언어에서는** C에서는 모든 코드가 `unsafe` 블록 안에 있는 것과 같습니다. 그래서 어느 줄에서든 미정의 동작이 생길 수 있고, 새니타이저로 실행해 본 경로만 확인됩니다. Rust의 `unsafe`는 그 범위를 표시하고 안전한 API로 감싸게 해서, 검토할 코드를 줄입니다(ch3 Q3.2).

**오해와 한계** `unsafe` 블록이 작다고 영향도 작은 것은 아닙니다. Reference가 말하듯 미정의 동작은 프로그램 전체에 영향을 주므로, `unsafe` 한 줄의 실수가 안전한 코드에서 증상으로 나타날 수 있습니다. 표준 라이브러리 밖의 `unsafe` 코드를 피하는 지침은 [부록 A ch16](../../appd-a/ch16-avoid-unsafe/README.md)에 있습니다.

**더 읽기** ch3 Q3.2·Q3.3 / Part 0 ch19 2.1~2.3·2.7절 / 부록 A ch16 / Part 9 `ch10-unsafety-const-asm`, Part 10 `ch1-safe-and-unsafe`

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_defined_arithmetic](examples/01_defined_arithmetic.rs) | C에서 미정의 동작인 시프트·나눗셈을 Rust 메서드로 결과를 골라 계산 | Q10.5 |
| [02_conversions_and_bits](examples/02_conversions_and_bits.rs) | 정수 승격 대신 명시적 확장, 부호가 다른 비교의 세 방법, 실수의 비트 읽기 | Q10.4, Q10.8 |
| [03_evaluation_order](examples/03_evaluation_order.rs) | 인자·이항 연산·대입의 평가 순서, 컴파일 시간에 정해지는 `static` | Q10.6 |

### 01_defined_arithmetic — 정해진 산술

```sh
cargo run -p faq-ch10-undefined-behavior --example 01_defined_arithmetic
```

```text
1. 시프트
  1u32.checked_shl(31) = Some(2147483648)
  1u32.checked_shl(32) = None
  1u32.wrapping_shl(32) = 1
  1u32.wrapping_shl(33) = 2
2. 나눗셈
  10i32.checked_div(0) = None
  i32::MIN.checked_div(-1) = None
  i32::MIN.wrapping_div(-1) = -2147483648
  i32::MIN.overflowing_div(-1) = (-2147483648, true)
```

- `overflowing_div`는 래핑한 결과와 오버플로가 났는지를 함께 돌려줍니다.
- 연산자(`<<`, `/`)를 그대로 쓴 경우의 동작은 값을 `std::hint::black_box`로 감싸 실행 중에 정해지게 한 코드를 디버그(`rustc`)와 릴리스(`rustc -O`)로 각각 컴파일해 확인했습니다. 디버그는 시프트에서 ``attempt to shift left with overflow``, 나눗셈에서 ``attempt to divide by zero``·``attempt to divide with overflow`` 패닉을 냈고, 릴리스는 시프트만 1을 내고 나눗셈 둘은 같은 패닉을 냈습니다.

### 02_conversions_and_bits — 명시적 변환과 비트

```sh
cargo run -p faq-ch10-undefined-behavior --example 02_conversions_and_bits
```

```text
1. 정수 승격 대신 명시적 확장
  200u8.checked_add(100) = None
  u16::from(a) + u16::from(b) = 300
2. 부호가 다른 수의 비교
  i64로 넓혀 비교: true
  as u32로 C처럼: false (balance as u32 = 4294967295)
  u32::try_from(balance) = Err(TryFromIntError(NegOverflow))
3. 비트로 다른 타입 읽기
  1.0f32.to_bits() = 0x3f800000
  f32::from_bits(0x4049_0fdb) = 3.1415927
```

- `TryFromIntError`의 디버그 형식에 든 `NegOverflow`는 음수라서 실패했다는 뜻입니다.
- `{:#010x}`는 `0x`를 붙인 16진수를 열 글자 폭으로 출력합니다.

### 03_evaluation_order — 평가 순서

```sh
cargo run -p faq-ch10-undefined-behavior --example 03_evaluation_order
```

```text
1. 함수 인자와 이항 연산
    첫째 인자
    둘째 인자
    왼쪽 피연산자
    오른쪽 피연산자
  (1, 2), 30
2. 대입
    값
    자리(인덱스)
  [0, 7, 0]
3. 컴파일 시간에 정해진 static
  SQUARES = [0, 1, 4, 9]
```

- `squares`는 `const fn`이라 반복문을 `for` 대신 `while`로 썼습니다. `const fn` 안에서 범위에 `for`를 쓰면 E0658(``cannot use `for` loop on `std::ops::Range<usize>` in constant functions``)이 납니다.

### C·C++ 비교 코드

저장소 루트에서 실행합니다. `cc`·`c++`의 `-o`는 출력 디렉터리를 만들지 않으므로 먼저 `mkdir -p target/tmp/faq-ch10-undefined-behavior`를 실행합니다. 파일마다 머리 주석에 명령이 있습니다. 새니타이저 없이 실행한 결과는 미정의 동작의 결과라 싣지 않습니다(04는 결과가 정해져 있어 실었습니다).

| 파일 | 질문 | 명령(요지) | 이 환경에서 나온 것 |
| --- | --- | --- | --- |
| [01_signed_overflow.c](compare/01_signed_overflow.c) | Q10.3 | `cc -g -fsanitize=undefined` 후 실행 | `runtime error: signed integer overflow: 2147483647 + 1 cannot be represented in type 'int'`, 종료 0 |
| [02_shift_width.c](compare/02_shift_width.c) | Q10.5 | 같음 | `runtime error: shift exponent 32 is too large for 32-bit type 'unsigned int'`, 종료 0 |
| [03_division.c](compare/03_division.c) | Q10.5 | 같음 | `runtime error: division of -2147483648 by -1 cannot be represented in type 'int'`, `runtime error: division by zero`, 종료 0 |
| [04_sign_compare.c](compare/04_sign_compare.c) | Q10.4 | `cc -Wall -Wextra` 후 실행 | 경고 `comparison of integers of different signs: 'int' and 'unsigned int' [-Wsign-compare]`, 출력은 Q10.4 |
| [05_uninit_read.c](compare/05_uninit_read.c) | Q10.1 | `cc -Wall`(컴파일만) | 경고 `variable 'total' is used uninitialized whenever 'if' condition is false [-Wsometimes-uninitialized]` |
| [06_unsequenced.c](compare/06_unsequenced.c) | Q10.6 | `cc`(컴파일만) | 경고 `multiple unsequenced modifications to 'i' [-Wunsequenced]` |
| [07_missing_return.cpp](compare/07_missing_return.cpp) | Q10.5 | `c++ -std=c++20 -g -fsanitize=undefined` 후 실행 | 경고 `non-void function does not return a value in all control paths [-Wreturn-type]`, `runtime error: execution reached the end of a value-returning function without returning a value`, 종료 134 |

- UBSan 문서에 따르면 대부분의 검사는 오류를 보고한 뒤 실행을 계속하므로 01~03은 종료 상태가 0입니다. 07의 검사는 이 환경에서 보고 뒤 멈춰 134로 끝났습니다.
- 04~06의 경고 가운데 `-Wsign-compare`는 `-Wextra`, `-Wsometimes-uninitialized`는 `-Wall`을 켜야 나왔고, `-Wunsequenced`와 `-Wreturn-type`은 옵션 없이도 나왔습니다.

## 4. 자주 하는 실수

### 4.1 부호가 다른 정수를 바로 비교 — E0308

파일: [`compile_fail/e0308_compare_signed_unsigned.rs`](compile_fail/e0308_compare_signed_unsigned.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_compare_signed_unsigned.rs:19:18
   |
19 |     if balance < limit {
   |        -------   ^^^^^ expected `i32`, found `u32`
   |        |
   |        expected because this is `i32`
   |
help: you can convert a `u32` to an `i32` and panic if the converted value doesn't fit
   |
19 |     if balance < limit.try_into().unwrap() {
   |                       ++++++++++++++++++++
```

- **원인**: C처럼 `int`와 `unsigned`에 해당하는 `i32`와 `u32`를 바로 비교했습니다. 정수 타입 사이의 비교 연산자는 같은 타입끼리만 구현돼 있고, 어느 쪽으로 바꿀지 정해 주지 않습니다.
- **해결**: 둘 다 담을 수 있는 `i64`로 넓혀 `i64::from(balance) < i64::from(limit)`으로 씁니다. 컴파일러의 `help`는 `limit`를 `i32`로 바꾸고 안 들어가면 패닉하는 방법을 제안하는데, `limit`가 `i32::MAX`보다 크면 패닉하므로 넓히는 쪽이 낫습니다.
- 덧붙임: 정수 타입이 다른 값을 함수 인자나 반환 자리에 쓰는 E0308은 부록 A ch1 4.1절에서 봤습니다. 이 절은 비교 연산자에서도 같고, C의 부호 변환(compare/04)이 일어날 자리라는 점을 더합니다.

### 4.2 정수를 열거형으로 `as` 변환 — E0605

파일: [`compile_fail/e0605_int_to_enum.rs`](compile_fail/e0605_int_to_enum.rs)

```text
error[E0605]: non-primitive cast: `u8` as `Direction`
  --> compile_fail/e0605_int_to_enum.rs:26:15
   |
26 |     let dir = code as Direction; // E0605
   |               ^^^^^^^^^^^^^^^^^ an `as` expression can only be used to convert between primitive types or to coerce to a specific trait object
```

- **원인**: C에서 정수를 `enum` 변수에 넣듯 `u8`을 `as`로 열거형으로 바꾸려 했습니다. 열거형은 정해진 배리언트 가운데 하나만 값이 될 수 있고, 4나 255 같은 정수는 그 어느 것도 아니므로 `as`가 허용되지 않습니다. 반대 방향인 `Direction::West as u8`은 언제나 올바른 값이라 됩니다.
- **해결**: `match code { 0 => Some(Direction::North), …, _ => None }`처럼 해당하지 않는 값을 처리하는 변환을 씁니다. 자주 쓰면 `TryFrom<u8>`을 구현합니다(연습 2).
- 덧붙임: 참조를 다른 타입의 참조로 바꾸는 E0605는 ch2 4.2절, 뉴타입 구조체를 `as`로 바꾸려는 E0605는 [부록 A ch5](../../appd-a/ch5-type-conversions/README.md) 2.1절에서 봤습니다. 이 절의 상황은 정수에서 열거형으로 가는 변환입니다.

## 5. 연습 문제

### 연습 1. n번째 비트

`fn bit(n: u32) -> u32`를 만들어, `u32`에서 n번째 비트만 1인 값을 돌려주되 `n`이 32 이상이면 0을 돌려주게 해 보세요. `bit(3)`, `bit(31)`, `bit(32)`, `bit(40)`을 출력합니다.

<details>
<summary>힌트</summary>

`1u32.checked_shl(n).unwrap_or(0)`이나 `1u32.unbounded_shl(n)`으로 씁니다. 결과는 8, 2147483648, 0, 0입니다. `1u32.wrapping_shl(n)`을 쓰면 `bit(32)`가 1, `bit(40)`이 256이 되어 원하는 답이 아닙니다(Q10.5). C의 `1u << n`은 `n`이 32 이상이면 compare/02처럼 미정의 동작입니다.

</details>

### 연습 2. 바이트를 방향으로

4.2절의 `Direction`에 `impl TryFrom<u8> for Direction`을 구현해, 0~3은 해당 배리언트를, 그 밖의 값은 `Err(값)`을 돌려주게 해 보세요. `Direction::try_from(2)`와 `Direction::try_from(7)`을 출력합니다.

<details>
<summary>힌트</summary>

`type Error = u8;`로 정하고 `fn try_from(code: u8) -> Result<Self, Self::Error>` 안에서 `match code { 0 => Ok(Direction::North), 1 => Ok(Direction::East), 2 => Ok(Direction::South), 3 => Ok(Direction::West), other => Err(other) }`로 씁니다. 출력은 `Ok(South)`와 `Err(7)`입니다. `TryFrom`은 프렐류드에 있어 `use` 없이 씁니다. 같은 모양의 구현은 부록 A ch5 2.2절에서 봤습니다.

</details>

### 연습 3. 대입 순서 확인하기

03의 2번에 `v[log("자리", 0)] += log("더할 값", 5);`를 더하고 출력 순서를 확인해 보세요. 일반 대입과 같은 순서일지 예상한 뒤 실행합니다.

<details>
<summary>힌트</summary>

복합 대입은 Reference가 따로 정합니다. 두 피연산자가 모두 원시 타입이면 더할 값을 먼저 평가하므로, `usize` 원소의 `+=`에서는 "더할 값", "자리" 순서로 출력됩니다. 원시 타입이 아니면(연산자 트레이트를 구현한 사용자 타입) 왼쪽부터 평가합니다. 이런 차이에 기대는 코드는 읽기 어려우므로 문장을 나눠 쓰는 편이 낫습니다(Q10.6).

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rust Reference: [Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html), [Behavior not considered unsafe](https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html), [Evaluation order of operands](https://doc.rust-lang.org/reference/expressions.html#evaluation-order-of-operands), [Operator expressions — Overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow), [Assignment expressions](https://doc.rust-lang.org/reference/expressions/operator-expr.html#assignment-expressions), [Static items](https://doc.rust-lang.org/reference/items/static-items.html)
- The Rustonomicon: [What Unsafe Rust Can Do](https://doc.rust-lang.org/nomicon/what-unsafe-does.html)
- 표준 라이브러리 문서: [`i32::checked_div`](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_div), [`u32::wrapping_shl`](https://doc.rust-lang.org/std/primitive.u32.html#method.wrapping_shl), [`f32::from_bits`](https://doc.rust-lang.org/std/primitive.f32.html#method.from_bits), [`TryFrom`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html)

C·C++과 도구

- Clang: [UndefinedBehaviorSanitizer](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html), [MemorySanitizer](https://clang.llvm.org/docs/MemorySanitizer.html)
- GCC: [Optimize Options](https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html)(`-fstrict-aliasing`), [Restricting Pointer Aliasing](https://gcc.gnu.org/onlinedocs/gcc/Restricted-Pointers.html)
- C++ 표준 초안: [\[basic.start.dynamic\]](https://eel.is/c++draft/basic.start.dynamic)
- Java: Java Language Specification(SE 21) [16장 Definite Assignment](https://docs.oracle.com/javase/specs/jls/se21/html/jls-16.html)
