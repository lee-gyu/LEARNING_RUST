# ch7. 문법과 타입에서 낯선 것

Rust 코드를 처음 읽는 사람은 큰 개념보다 작은 문법에서 먼저 걸립니다. 변수는 왜 바꿀 수 없는가, 마지막 줄의 세미콜론은 왜 빠졌는가, `i32`를 `i64`에 왜 그냥 못 넣는가, 문자열은 왜 둘이고 `s[0]`은 왜 안 되는가. 이 챕터는 그런 작은 질문을 하나씩 짧게 답합니다. 문법 자체는 Part 0 ch3·ch6·ch8에서 배웠으므로 다시 가르치지 않고, 독자가 쓰던 언어와 무엇이 다르고 왜 그렇게 정했는지를 봅니다.

- 패키지: `faq-ch7-syntax-type-surprises`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q7.1](#q71-변수가-왜-기본으로-불변인가요-같은-이름으로-let을-다시-쓰는-것은-왜-허용하나요) | 변수가 왜 기본으로 불변인가요? 같은 이름으로 `let`을 다시 쓰는 것은 왜 허용하나요? | 바뀌는 값을 `mut`으로 드러내려는 설계입니다. 같은 이름의 `let`은 값을 바꾸는 것이 아니라 새 변수를 만드는 섀도잉입니다. |
| [Q7.2](#q72-세미콜론이-없는-마지막-줄은-무엇인가요-삼항-연산자와-는-왜-없나요) | 세미콜론이 없는 마지막 줄은 무엇인가요? 삼항 연산자와 `++`는 왜 없나요? | 블록의 값이 되는 표현식입니다. `if`가 값을 내므로 삼항 연산자가 필요 없고, 평가 순서에 기대는 버그를 낳는 `++` 대신 `+= 1`을 씁니다. |
| [Q7.3](#q73-정수-타입이-왜-이렇게-많고-usize는-무엇인가요) | 정수 타입이 왜 이렇게 많고 `usize`는 무엇인가요? | 폭과 부호를 타입으로 고르기 때문입니다. `usize`는 포인터와 같은 폭의 부호 없는 정수로, 길이와 인덱스에 씁니다. |
| [Q7.4](#q74-i32-값을-i64-변수에-왜-그냥-넣지-못하나요) | `i32` 값을 `i64` 변수에 왜 그냥 넣지 못하나요? | 타입이 바뀌는 곳을 코드에 드러내려고 암묵적 변환을 두지 않았습니다. 손실 없는 변환은 `i64::from`, 실패할 수 있으면 `try_from`으로 적습니다. |
| [Q7.5](#q75-문자열-타입이-왜-string과-str-둘인가요-무엇을-쓰나요) | 문자열 타입이 왜 `String`과 `&str` 둘인가요? 무엇을 쓰나요? | 소유하는 문자열과 빌린 문자열입니다. 매개변수는 `&str`, 직접 가져야 하는 필드와 새로 만든 문자열의 반환은 `String`이 기본입니다. |
| [Q7.6](#q76-문자열을-왜-s0으로-읽을-수-없나요) | 문자열을 왜 `s[0]`으로 읽을 수 없나요? | `str`은 UTF-8 바이트라서 "0번째"가 바이트인지 글자인지 정해지지 않기 때문입니다. 무엇으로 셀지 메서드로 고릅니다. |
| [Q7.7](#q77-타입-추론이-되는데-함수-시그니처에는-왜-타입을-다-써야-하나요) | 타입 추론이 되는데 함수 시그니처에는 왜 타입을 다 써야 하나요? | 추론은 함수 본문 안에서만 하고, 시그니처는 함수 사이의 약속으로 직접 적게 한 의도적인 설계입니다. |
| [Q7.8](#q78-println의-는-무엇이고-왜-함수가-아닌가요) | `println!`의 `!`는 무엇이고 왜 함수가 아닌가요? | `!`는 매크로 호출 표시입니다. 인자 개수가 정해지지 않았고 서식 문자열을 컴파일 시간에 검사하려면 함수로는 안 됩니다. |
| [Q7.9](#q79-match는-왜-모든-경우를-다뤄야-하나요-switch와-무엇이-다른가요) | `match`는 왜 모든 경우를 다뤄야 하나요? `switch`와 무엇이 다른가요? | 빠뜨린 경우를 컴파일러가 잡게 하려는 것입니다. 전통적인 콜론 형식의 `switch`와 달리 다음 갈래로 넘어가지 않고 값을 내며, 형식이 하나라 언제나 빠짐없어야 합니다. |
| [Q7.10](#q710-실수는-왜-hashmap의-키로-쓰거나-바로-정렬할-수-없나요) | 실수는 왜 `HashMap`의 키로 쓰거나 바로 정렬할 수 없나요? | `NaN`이 자기 자신과도 같지 않아서 `f64`에는 `Eq`·`Ord`가 없고 `Hash`도 구현되어 있지 않기 때문입니다. 정렬은 `total_cmp`로 합니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md): 변수와 가변성, 섀도잉, 데이터 타입, 정수 오버플로(overflow), 함수, 구문과 표현식
- [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 2.12절: 문자열 슬라이스 `&str`
- [Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md): `match`와 완전성 검사(exhaustiveness checking, 2.8절)
- [Part 0 ch8](../../part0-getting-started/ch8-collections/README.md) 2.8~2.10절: `String`과 `&str`, `s[0]`이 안 되는 이유

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 불변 기본과 섀도잉, 표현식 기반 문법을 Java의 `final`, JavaScript의 `const`, 삼항 연산자, `++`와 비교해 설명할 수 있다.
2. 정수 타입의 폭과 암묵적 변환이 없는 이유를 Java·Go·Python·JavaScript의 수 타입과 비교하고, 값의 범위에 맞는 타입을 고를 수 있다.
3. 같은 문자열의 길이와 "첫 글자"가 Go·Java·JavaScript·Python에서 달라지는 까닭을 UTF-8·UTF-16·코드 포인트(code point)로 설명하고, Rust에서 각각을 구할 수 있다.
4. 시그니처의 타입 표기, 매크로 `!`, 완전한 `match`, 실수의 비교 문제가 어떤 실수를 막으려는 설계인지 설명할 수 있다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다. 질문이 작은 만큼 소절도 짧게 씁니다.

### Q7.1 변수가 왜 기본으로 불변인가요? 같은 이름으로 `let`을 다시 쓰는 것은 왜 허용하나요?

**짧은 답** 바뀌는 값과 바뀌지 않는 값을 코드에서 구별하려는 설계입니다. 변수는 기본으로 불변(immutable)이고, 가변(mutable)으로 쓸 것에만 `mut`을 붙입니다. `let`으로 만든 변수는 다시 대입할 수 없습니다. 같은 이름으로 `let`을 다시 쓰는 것은 값을 바꾸는 것이 아니라 같은 이름의 새 변수를 만드는 섀도잉(shadowing)이라서, 타입이 달라도 됩니다.

**왜 그런가** The Book 3.1절은 기본 불변을 Rust가 안전성과 쉬운 동시성을 살리도록 이끄는 여러 장치 중 하나로 설명합니다. 불변은 변수뿐 아니라 그 변수로 닿는 값 전체에 걸립니다. `let v = vec![1, 2]`이면 `v`에 다시 대입할 수도, `v.push(3)`으로 내용을 고칠 수도 없습니다([Part 0 ch5](../../part0-getting-started/ch5-structs/README.md) 4.4절의 E0596과 같은 이유). 다만 불변은 값이 아니라 변수(바인딩)의 성질이라서, `let mut w = v;`로 값을 가변 변수에 옮기면 `w.push(3)`은 됩니다. 섀도잉은 `let input = input.trim();`, `let count: u32 = count.parse()?;`처럼 "같은 것을 가공한 새 값"에 이름을 다시 쓰고 싶을 때 씁니다.

**다른 언어에서는** Java 언어 명세는 `final` 변수가 객체를 가리키면 그 객체의 상태는 바뀔 수 있고 변수만 같은 객체를 계속 가리킨다고 정합니다(4.12.4). MDN도 JavaScript의 `const`가 다시 대입할 수 없는 참조일 뿐 값이 불변이라는 뜻은 아니라고 설명합니다. 그래서 `final List`나 `const arr`에는 원소를 더할 수 있지만, Rust의 불변 변수로는 그렇게 할 수 없습니다. 또 Java는 지역 변수의 스코프 안에서 같은 이름의 지역 변수를 새로 선언하면 컴파일 에러로 막습니다(6.4). Rust의 섀도잉은 이 점이 반대입니다.

**오해와 한계** 불변이 절대적인 것은 아닙니다. `Cell`·`RefCell`·`Mutex` 같은 타입은 불변 참조를 통해서도 안의 값을 바꾸게 해 주며(내부 가변성(interior mutability), [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)), 그런 타입이 있다는 것은 시그니처에 드러납니다. 섀도잉은 같은 스코프에서 이름이 가려지므로, 긴 함수에서 남용하면 어느 값을 가리키는지 헷갈릴 수 있습니다.

**더 읽기** Part 0 ch3(변수와 가변성, 섀도잉, '자주 하는 실수' 1의 E0384)

### Q7.2 세미콜론이 없는 마지막 줄은 무엇인가요? 삼항 연산자와 `++`는 왜 없나요?

**짧은 답** 세미콜론이 없는 마지막 줄은 그 블록의 값이 되는 표현식(expression)입니다. 함수 본문이면 반환값이 됩니다. `if`도 값을 내는 표현식이라서 `let x = if c { a } else { b };`로 쓰면 되므로 삼항 연산자 `?:`가 따로 없습니다. `++`와 `--`도 없고 `count += 1`로 씁니다. 예전 공식 Rust FAQ는 증감 연산자가 평가 순서를 알아야 해서 C·C++에서 미묘한 버그와 미정의 동작으로 이어지곤 하고, `x += 1`은 조금 길 뿐 모호하지 않다는 이유를 들었습니다.

**왜 그런가** Rust에서는 블록, `if`, `match`, `loop`(`break 값`)가 모두 값을 냅니다(The Book 3.5절의 "Using if in a let Statement"). 세미콜론을 붙이면 그 표현식은 값을 버리는 구문(statement)이 되고 블록의 값은 `()`가 됩니다. 그래서 마지막 줄에 습관적으로 세미콜론을 붙이면 반환 타입과 맞지 않는다는 에러가 납니다(Part 0 ch3 '자주 하는 실수' 3의 E0308). `count++`라고 쓰면 컴파일러가 Rust에는 그런 연산자가 없다며 `+= 1`을 제안합니다(4.1절).

**다른 언어에서는** Go도 삼항 연산자가 없습니다. Go FAQ는 그 연산자가 읽기 어려운 복잡한 식을 만드는 데 너무 자주 쓰였다는 이유를 들고, `++`와 `--`는 식이 아니라 문장으로만 두어 `f(i++)` 같은 식의 평가 순서 문제를 없앴다고 설명합니다. Rust는 `++` 자체를 두지 않았고, `if`를 표현식으로 만들어 삼항 연산자의 자리를 메웠습니다. 위의 예전 Rust FAQ는 지금 공식 사이트에서 내려갔고, 보관된(archived) 저장소 rust-lang/prev.rust-lang.org의 `en-US/faq.md`(마지막 수정 2018-06-22)에 남아 있습니다.

**오해와 한계** 마지막 줄의 세미콜론 하나로 뜻이 바뀌므로 처음에는 실수하기 쉽습니다. 다만 그 실수는 대부분 타입 에러로 바로 드러납니다. 함수 중간에서 일찍 돌아갈 때는 여전히 `return`을 씁니다.

**더 읽기** Part 0 ch3(구문과 표현식, 제어 흐름) / Part 0 ch3 예제 08~10

### Q7.3 정수 타입이 왜 이렇게 많고 `usize`는 무엇인가요?

**짧은 답** Rust는 정수의 폭(8·16·32·64·128비트)과 부호 유무를 타입으로 고르게 합니다. 값이 얼마나 커질지, 음수가 필요한지를 타입이 말해 주고, 메모리 배치도 정확히 정해집니다. `usize`(와 `isize`)는 그 컴퓨터의 포인터와 같은 폭의 정수로, 컬렉션의 길이와 인덱스에 씁니다. 타입을 적지 않은 정수 리터럴은 `i32`가 됩니다.

**왜 그런가** The Book 3.2절에 따르면 `isize`·`usize`의 폭은 컴퓨터의 아키텍처를 따라 64비트 환경에서 64비트, 32비트 환경에서 32비트이고, Rust 레퍼런스는 포인터와 같은 폭이라고 정하고, 그 폭이 최소 16비트라고만 보장합니다(64·32비트만 있는 것은 아님). 메모리 주소를 셀 수 있을 만큼의 폭이므로 길이와 인덱스에 맞습니다. 16비트 환경도 고려하므로 `u32`에서 `usize`로 가는 `From`이 없다는 것은 Part 1 ch6 예제 02에서 봤습니다. [02_integer_widths](examples/02_integer_widths.rs)는 폭이 정해져 있다는 것이 무엇을 뜻하는지 봅니다.

```text
1. 타입을 적지 않은 정수는 i32
  크기: 4바이트, i32::MAX = 2147483647
  i32: 20억 + 20억 = None
  i64: 20억 + 20억 = Some(4000000000)
2. n!에서 처음 오버플로가 나는 n
  u32: 13!, u64: 21!, u128: 35!
```

정수 오버플로가 나면 디버그 빌드에서는 패닉하고, `--release` 빌드에서는 검사 없이 래핑(wrapping)합니다(The Book 3.2절). 오버플로가 날 수 있는 연산은 `checked_*` 같은 메서드로 명시합니다(Part 0 ch3 예제 04).

**다른 언어에서는**

- Python 문서는 정수의 정밀도에 제한이 없다고 적습니다. 02의 2번처럼 Rust에서 `u128`도 오버플로가 나는 35!을 Python은 그대로 계산합니다.
- JavaScript의 `Number`는 IEEE 754 64비트 실수이고, 그보다 큰 정수는 언어에 내장된 `BigInt`로 다룹니다(MDN). 2^53을 넘는 정수는 실수로 바꿀 때 값이 바뀔 수 있다는 것을 Part 1 ch6 예제 02에서 봤습니다.
- Java의 정수 타입 `byte`·`short`·`int`·`long`은 모두 부호가 있고, 부호 없는 정수 타입은 16비트 `char`뿐입니다(Java 언어 명세 4.2.1).
- Go 명세는 `int`·`uint`의 폭을 구현에 따라 32비트나 64비트로 정합니다. Rust의 `isize`·`usize`와 비슷한 자리입니다.

**오해와 한계** 타입을 고르는 수고가 들고, 서로 다른 정수 타입끼리는 바로 섞어 쓸 수 없어서(Q7.4) `as`나 `from`이 늘어납니다. `u128`보다 큰 수가 필요하면 임의 정밀도 정수는 외부 크레이트를 써야 합니다. Java는 `BigInteger`, Go는 `math/big`을 표준 라이브러리에 두고, JavaScript는 `BigInt`를 언어에 내장한 것과 다른 점입니다.

**더 읽기** Part 0 ch3(데이터 타입, 4절 정수 오버플로, 예제 03) / [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.9절 / [부록 A ch1](../../appd-a/ch1-express-data-structures/README.md) 2.1절

### Q7.4 `i32` 값을 `i64` 변수에 왜 그냥 넣지 못하나요?

**짧은 답** Rust에는 수 타입 사이의 암묵적 변환이 없습니다. 타입이 바뀌는 곳과 값이 잘릴 수 있는 곳을 코드에 드러내려는 것입니다. 값이 바뀌지 않는 넓히는 변환(widening conversion)도 `i64::from(x)`나 `x.into()`로 적어야 합니다. 값이 잘릴 수 있는 좁히는 변환(narrowing conversion)은 `i32::try_from(big)`으로 실패를 `Result`로 받거나, 잘려도 되는 경우에만 `as`를 씁니다.

**왜 그런가** 변환을 적게 하면 "여기서 타입이 바뀐다"는 것이 코드에 보이고, 잘릴 수 있는 변환과 그렇지 않은 변환이 메서드 이름으로 구별됩니다. 정수 타입이 다른 값을 그대로 넘기는 E0308은 부록 A ch1 4.1절에, `from`·`try_from`·`as`를 고르는 기준은 [부록 A ch5](../../appd-a/ch5-type-conversions/README.md) 2.4·2.5절에 있습니다.

**다른 언어에서는** Java 언어 명세는 `int`에서 `long`으로 가는 것 같은 넓히는 변환을 대입할 때 저절로 하도록 허용합니다(5.1.2, 5.2). Go는 Rust처럼 암묵적인 수 변환이 없고, Go FAQ는 C의 자동 변환이 주는 편리함보다 "어떤 식이 부호가 없는지, 얼마나 큰지, 오버플로가 나는지"를 헷갈리게 하는 비용이 크다고 설명합니다.

**오해와 한계** 넓히는 변환까지 적어야 하므로 코드가 길어지고, 귀찮아서 `as`로 통일하면 좁히는 변환에서 값이 조용히 잘리는 문제를 다시 들여오게 됩니다(부록 A ch5 2.4절).

**더 읽기** 부록 A ch1 2.1절 / 부록 A ch5 / [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)(`From`, `TryFrom`, `as`)

### Q7.5 문자열 타입이 왜 `String`과 `&str` 둘인가요? 무엇을 쓰나요?

**짧은 답** `String`은 힙에 있는 문자열을 소유하고 늘리거나 고칠 수 있는 타입이고, `&str`은 어딘가에 있는 UTF-8 문자열을 빌려 보는 참조입니다. 함수의 매개변수는 `&str`로 받으면 `&String`(`&s`)도 문자열 리터럴도 넘길 수 있고, 구조체 필드나 새로 만든 문자열을 돌려줄 때는 `String`을 씁니다.

**왜 그런가** 소유권 규칙(Part 0 ch4) 때문에 "누가 이 메모리를 해제하는가"가 타입에 드러나야 합니다. `String`은 해제할 책임을 지고, `&str`은 책임 없이 빌립니다. 문자열 리터럴 `"abc"`는 실행 파일 안에 있는 문자열을 빌린 `&'static str`입니다. 빌리는 쪽을 따로 두면 부분 문자열을 복사 없이 넘길 수 있습니다(`&s[0..3]`).

**다른 언어에서는** Java API 문서는 `String`이 만든 뒤 바꿀 수 없는 상수라고 하고, 바꿀 수 있는 문자열은 `StringBuilder` 같은 별도 타입으로 둡니다. Go 명세도 문자열은 불변이라고 정합니다. Rust의 `&str`이 이 불변 문자열에 가깝고, `String`은 `StringBuilder`의 자리에 있습니다. 차이는 Rust에서는 GC가 아니라 소유권이 문자열 메모리를 관리한다는 것입니다([ch4](../ch4-memory-without-gc/README.md) Q4.1).

**오해와 한계** 두 타입 사이를 오가는 `to_string()`, `as_str()`, `&` 같은 변환이 코드에 자주 보이는 것이 비용입니다. 구조체에 `&str` 필드를 두려면 라이프타임을 적어야 하므로, 처음에는 필드를 `String`으로 두는 편이 쉽습니다(Part 0 ch5 4.5절의 E0106).

**더 읽기** Part 0 ch4 2.12절 / Part 0 ch8 2.8·2.9절 / [Part 1 ch3](../../part1-standard-lib/ch3-strings-fmt/README.md)

### Q7.6 문자열을 왜 `s[0]`으로 읽을 수 없나요?

**짧은 답** Rust의 `str`은 UTF-8 바이트의 나열이라서, "0번째"가 바이트인지 글자인지 정해지지 않습니다. 그래서 정수 인덱스를 막고(Part 0 ch8 4.2절의 E0277), 무엇으로 셀지를 메서드로 고르게 합니다. 바이트는 `s.as_bytes()[0]`, 첫 글자는 `s.chars().next()`, 바이트 범위로 자르기는 `&s[0..3]`이나 실패를 `Option`으로 받는 `s.get(0..3)`입니다.

**왜 그런가** 언어마다 문자열을 바이트, 코드 단위(code unit), 코드 포인트 가운데 무엇의 나열로 보는지가 다르고, 그래서 같은 문자열의 길이와 `s[0]`이 다릅니다. [01_string_lengths](examples/01_string_lengths.rs)는 Rust에서 세 가지 기준으로 세어 봅니다.

```text
1. 세 가지 길이
  Rust: 바이트 4 / UTF-16 4 / char 4
  한글: 바이트 6 / UTF-16 2 / char 2
  🦀: 바이트 4 / UTF-16 2 / char 1
  e + U+0301: 바이트 3 / UTF-16 2 / char 2
2. "🦀"의 첫 단위를 꺼내면
  첫 바이트(Go의 s[0]): 0xf0
  첫 UTF-16 단위(Java의 charAt(0), JS의 s[0]): 0xd83e
  첫 char(Python의 s[0]): '🦀'
```

**다른 언어에서는**

| 언어 | 길이와 `s[0]`의 단위 | 근거 |
| --- | --- | --- |
| Go | 바이트 | Go 명세: `len`은 바이트 수, `s[i]`는 i번째 바이트 |
| Java | UTF-16 코드 단위 | Java API 문서: `length()`는 유니코드 코드 단위 수, `charAt`은 `char` 값 |
| JavaScript | UTF-16 코드 단위 | MDN: `length`는 UTF-16 코드 단위 수 |
| Python | 유니코드 코드 포인트 | Python 문서: 문자열은 유니코드 코드 포인트의 불변 나열 |
| Rust | 고름: 바이트(`len`), UTF-16(`encode_utf16`), `char`(`chars`) | std 문서 |

Java·JavaScript에서 `"🦀"`의 첫 단위는 01의 2번처럼 짝이 있어야 뜻이 있는 서로게이트(surrogate)의 절반입니다. Rust는 `s[0]`을 막아서, 어떤 단위를 원하는지 정하지 않은 채 이런 값을 꺼내는 일을 없앴습니다.

**오해와 한계** `chars()`도 "사람이 보는 글자"를 세지는 않습니다. 01의 `e + U+0301`은 e 뒤에 결합 부호(combining mark)를 붙여 화면에서 é 한 글자로 보이지만 `char`는 2개입니다. 표준 라이브러리 문서는 `char`가 유니코드 스칼라 값(Unicode scalar value)이라 생각하는 "글자"와 다를 수 있고, 자소 클러스터(grapheme cluster) 단위로 나누는 기능은 표준 라이브러리에 없으니 crates.io를 보라고 적습니다. 또 `chars().nth(i)`는 앞에서부터 세므로 i에 비례하는 시간이 듭니다.

**더 읽기** Part 0 ch8 2.10절, 예제 07 / Part 1 ch3(글자 경계를 지키며 자르기, 예제 05)

### Q7.7 타입 추론이 되는데 함수 시그니처에는 왜 타입을 다 써야 하나요?

**짧은 답** Rust의 타입 추론(type inference)은 함수 본문 안에서만 일합니다. 함수의 매개변수와 반환 타입은 직접 적어야 하며, The Book은 이것을 의도적인 설계라고 설명합니다. 시그니처에 타입이 있으면 본문 안의 다른 곳에서는 타입을 거의 적지 않아도 되고, 컴파일러가 더 도움이 되는 에러 메시지를 낼 수 있다는 이유입니다.

**왜 그런가** 시그니처를 함수 사이의 약속으로 고정하면, 함수 본문을 고쳐도 그 함수를 부르는 코드의 타입은 바뀌지 않습니다. 본문 안에서는 추론이 뒤에 나오는 쓰임새까지 봅니다. `let mut names = Vec::new();` 다음 줄의 `names.push("러스트")`를 보고 `Vec<&str>`로 정하는 식입니다(Part 0 ch8 2.2절). 반환 타입을 `_`로 두고 컴파일러에게 맡기려 하면 E0121로 막히는데, 컴파일러는 본문을 보고 맞는 타입을 제안합니다(4.2절).

**다른 언어에서는** TypeScript 핸드북은 반환 타입 표기가 보통 필요 없고 return 문으로 추론한다고 설명하면서도, 문서화나 실수로 인한 변경을 막으려고 반환 타입을 명시하는 코드베이스도 있다고 덧붙입니다. Kotlin 문서는 식 하나로 된 함수의 반환 타입은 추론하지만, 블록 본문의 함수는 흐름이 복잡해 읽는 사람에게 반환 타입이 분명하지 않을 수 있어 직접 적게 한다고 설명합니다. Java의 `var`는 지역 변수에만 쓰며, OpenJDK의 지역 변수 타입 추론 지침은 `var`와 다이아몬드(`new PriorityQueue<>()`)를 함께 쓰면 `PriorityQueue<Object>`로 추론되니 조심하라고 적습니다. Rust의 추론은 뒤의 쓰임새까지 보므로 이런 경우에도 원소 타입을 찾아냅니다.

**오해와 한계** 반환 타입이 길거나 이름을 붙일 수 없는 타입(클로저, 반복자 사슬)이면 적기 어렵습니다. 그럴 때는 `impl Iterator<Item = u32>`처럼 "이 트레이트를 구현한 어떤 타입"으로 적습니다([Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.10절). 클로저는 함수와 달리 매개변수 타입을 추론합니다([Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md)).

**더 읽기** Part 0 ch3(함수) / Part 0 ch10 2.10절

### Q7.8 `println!`의 `!`는 무엇이고 왜 함수가 아닌가요?

**짧은 답** 이름 뒤의 `!`는 함수가 아니라 매크로(macro)를 부른다는 표시입니다. `println!`은 인자 개수가 정해져 있지 않고(`println!("안녕")`, `println!("{} {}", a, b)`), 서식 문자열(format string)의 `{}`와 인자가 맞는지를 컴파일 시간에 검사해야 합니다. 함수는 시그니처에 매개변수의 개수와 타입을 정해야 하므로 이 일을 할 수 없습니다.

**왜 그런가** The Book의 매크로 절은 함수 시그니처는 매개변수의 개수와 타입을 선언해야 하지만 매크로는 개수가 정해지지 않은 매개변수를 받을 수 있고, 컴파일러가 코드의 뜻을 해석하기 전에 펼쳐진다고 설명합니다. 그래서 `{}`가 둘인데 인자를 하나만 주면 실행하기 전에 컴파일 에러가 납니다(Part 0 ch1의 `format_args_mismatch.rs`). 인자가 남아도 `argument never used` 에러입니다.

**다른 언어에서는** Java API 문서에 따르면 `Formatter`(`String.format`)는 서식 지정자가 가리키는 인자가 없으면 실행 중에 `MissingFormatArgumentException`을 던지고, 남는 인자는 무시합니다. Rust는 두 경우를 모두 컴파일 시간에 잡습니다.

**오해와 한계** 컴파일 시간에 검사하는 대가로 서식 문자열은 리터럴이어야 합니다. `let t = "{}"; println!(t, 1);`은 `format argument must be a string literal` 에러이므로, Java의 `String.format(template, …)`처럼 실행 중에 고른 템플릿을 표준 매크로로 쓸 수는 없습니다. 또 매크로는 펼쳐진 코드에서 에러가 나면 메시지가 읽기 어려울 수 있고, 직접 만들기는 함수보다 어렵습니다. 그래서 함수로 되는 일은 함수로 쓰는 것이 기본입니다. `vec!`, `format!`, `assert!`도 같은 이유로 매크로입니다.

**더 읽기** [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md)(서식 문자열 검사) / [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.18·2.19절

### Q7.9 `match`는 왜 모든 경우를 다뤄야 하나요? `switch`와 무엇이 다른가요?

**짧은 답** 빠뜨린 경우를 컴파일러가 잡게 하려는 것입니다. `match`는 가능한 모든 값을 갈래(arm)로 다뤄야 컴파일되고(E0004), 나머지를 한꺼번에 받으려면 `_` 갈래를 둡니다. C·Java의 전통적인 콜론 형식 `switch`와 달리 한 갈래가 끝나면 다음 갈래로 넘어가지 않고(`break`가 필요 없음), `match` 전체가 값을 내는 표현식입니다. 형식이 하나뿐이라 언제나 빠짐없어야 합니다.

**왜 그런가** 열거형에 배리언트를 더하면, 그 열거형을 다루면서 `_` 갈래가 없는 모든 `match`가 새 배리언트를 다룰 때까지 컴파일되지 않습니다. 고칠 곳을 컴파일러가 찾아 주는 셈입니다(부록 A ch1 4.3절). 갈래 하나에 여러 값을 묶을 때는 `1 | 2 =>`처럼 `|`를 씁니다.

**다른 언어에서는** Java 언어 명세는 콜론 형식의 `switch`에서 일치한 레이블 뒤의 문장이 다음 레이블의 문장까지 이어서 실행되며(C·C++처럼 "넘어간다"), 이것이 미묘한 버그의 원인이 될 수 있어 `break`나 화살표 형식의 switch 규칙(`case X ->`)을 쓰라고 적습니다(14.11.3). 화살표 형식은 넘어가지 않고, 값을 내는 `switch` 식은 빠짐없지 않으면 컴파일 에러입니다(15.28.1). 그래서 요즘 Java의 `switch` 식은 Rust의 `match`에 가깝고, `sealed` 타입의 하위 타입을 모두 다룬 경우도 빠짐없는 것으로 인정합니다([ch6](../ch6-no-classes-inheritance/README.md) Q6.8). 남는 차이는 Java는 형식(콜론·화살표, 문·식)에 따라 넘어감과 완전성 요구가 달라지고, Rust의 `match`는 형식이 하나라 언제나 빠짐없어야 하는 표현식이라는 것입니다. Go 명세의 `switch`는 `fallthrough`를 적었을 때만 다음 절로 넘어갑니다.

**오해와 한계** `_` 갈래를 두면 완전성 검사는 통과하지만, 나중에 더한 배리언트도 `_`로 조용히 흘러갑니다. 배리언트가 늘 때 알림을 받고 싶으면 `_` 대신 배리언트를 모두 적습니다.

**더 읽기** Part 0 ch6 2.4·2.8절, 4.1절 / 부록 A ch1 4.3절 / [Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md)

### Q7.10 실수는 왜 `HashMap`의 키로 쓰거나 바로 정렬할 수 없나요?

**짧은 답** IEEE 754의 `NaN`은 자기 자신과도 같지 않습니다. 그래서 `f64`는 "같은 값은 자기 자신과 같다"를 약속하는 `Eq`와 모든 값에 순서를 매기는 `Ord`를 구현하지 않고 `PartialEq`·`PartialOrd`만 구현합니다. 같은 값은 같은 해시를 내야 하는 `Hash`도 구현되어 있지 않습니다. 예를 들어 비트를 그대로 해시하면, `==`로는 같은 `0.0`과 `-0.0`이 서로 다른 해시를 냅니다(두 값의 비트가 다름). `HashMap`의 키는 `Eq + Hash`를, `sort()`는 `Ord`를 요구하므로 `f64`를 바로 쓸 수 없습니다. 정렬은 `sort_by(f64::total_cmp)`로 하고, 키가 필요하면 정수 표현으로 바꾼 타입을 씁니다.

**왜 그런가** 표준 라이브러리의 `Eq` 문서는 `f32`·`f64`가 `NaN != NaN` 때문에 `PartialEq`만 구현하고 `Eq`는 구현하지 않는다고 적습니다. 키가 자기 자신과 같지 않으면 넣은 값을 다시 찾을 수 없고, 순서가 없는 값이 섞이면 정렬 결과를 믿을 수 없습니다. `total_cmp`는 IEEE 754의 totalOrder 규칙으로 `NaN`까지 포함한 모든 값에 순서를 매깁니다(Part 1 ch4).

**다른 언어에서는** MDN에 따르면 JavaScript의 `Map`은 키를 SameValueZero 알고리즘으로 비교해서 `NaN`을 `NaN`과 같은 키로 봅니다(`NaN !== NaN`인데도). 언어가 키 비교 규칙을 따로 정해 실수 키를 받아 주는 방식입니다. Rust는 그런 특별 규칙을 두지 않고, 어떤 기준으로 비교할지를 사용자가 정하게 합니다.

**오해와 한계** 실수를 정수 표현(`to_bits()`나 정해진 배율로 곱한 정수)으로 바꾼 키는 `-0.0`과 `0.0`을 다른 키로 보거나 정밀도를 잃는 등 그 나름의 선택을 강요합니다. 어떤 값을 같은 키로 볼지를 먼저 정해야 합니다. `f64` 필드가 있는 타입의 지침과 예제는 [부록 A ch10](../../appd-a/ch10-standard-traits/README.md) 2.5절과 예제 04에 있습니다.

**더 읽기** Part 1 ch1 2.10절 / [Part 1 ch4](../../part1-standard-lib/ch4-collections/README.md)(`total_cmp`로 정렬, `e0277_sort_f64.rs`) / 부록 A ch10 2.5절

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_string_lengths](examples/01_string_lengths.rs) | 같은 문자열을 바이트·UTF-16 코드 단위·`char`로 세기, Go·Java·JS·Python의 `s[0]`에 해당하는 값 | Q7.6 |
| [02_integer_widths](examples/02_integer_widths.rs) | 타입을 적지 않은 정수는 `i32`, 계승에서 오버플로가 나는 지점으로 본 `u32`·`u64`·`u128`의 폭(연습 2가 Q7.4) | Q7.3 |

이 챕터의 다른 질문은 Part 0·1과 부록 A에 이미 있는 예제로 답하므로, 해당 소절의 "더 읽기"를 따라가면 됩니다.

### 01_string_lengths — 언어마다 다른 문자열의 길이

```sh
cargo run -p faq-ch7-syntax-type-surprises --example 01_string_lengths
```

```text
1. 세 가지 길이
  Rust: 바이트 4 / UTF-16 4 / char 4
  한글: 바이트 6 / UTF-16 2 / char 2
  🦀: 바이트 4 / UTF-16 2 / char 1
  e + U+0301: 바이트 3 / UTF-16 2 / char 2
2. "🦀"의 첫 단위를 꺼내면
  첫 바이트(Go의 s[0]): 0xf0
  첫 UTF-16 단위(Java의 charAt(0), JS의 s[0]): 0xd83e
  첫 char(Python의 s[0]): '🦀'
```

- 한글 한 글자는 UTF-8로 3바이트, UTF-16으로 1단위입니다. 🦀처럼 U+FFFF보다 큰 글자는 UTF-8로 4바이트, UTF-16으로 2단위(서로게이트 쌍)입니다.
- `{:#04x}`와 `{:#06x}`는 `0x`를 붙인 16진수를 정해진 폭으로 출력합니다(Part 1 ch3).

### 02_integer_widths — 정수의 폭

```sh
cargo run -p faq-ch7-syntax-type-surprises --example 02_integer_widths
```

```text
1. 타입을 적지 않은 정수는 i32
  크기: 4바이트, i32::MAX = 2147483647
  i32: 20억 + 20억 = None
  i64: 20억 + 20억 = Some(4000000000)
2. n!에서 처음 오버플로가 나는 n
  u32: 13!, u64: 21!, u128: 35!
```

- 타입을 적지 않으면 `i32`라는 것과 `MIN`·`MAX` 상수(`i8`, `u32`)는 Part 0 ch3 예제 03에서도 봤습니다. 이 예제는 `size_of_val`로 크기를 직접 보고, `i32::MAX`의 값과 20억 + 20억이라는 구체적인 경계를 더합니다.
- `let a = 2_000_000_000;`의 `a`는 다른 곳에서 타입이 정해지지 않으므로 `i32`가 됩니다. 이렇게 타입이 정해지지 않은 리터럴에 바로 `a.checked_add(a)`처럼 메서드를 부르면 E0689가 납니다(Part 1 ch1 4.1절). 그래서 1번은 리터럴 끝에 `_i32`, `_i64`를 붙여 타입을 정했습니다.
- `first_overflow`는 `u32::checked_mul` 같은 함수를 인자로 받아 세 타입에 같은 반복문을 씁니다. `checked_mul`이 오버플로로 `None`을 돌려주는 첫 n을 돌려줍니다.

## 4. 자주 하는 실수

### 4.1 `count++`로 1을 더함 — 에러 코드 없음

파일: [`compile_fail/no_postfix_increment.rs`](compile_fail/no_postfix_increment.rs)

```text
error: Rust has no postfix increment operator
  --> compile_fail/no_postfix_increment.rs:17:14
   |
17 |         count++;
   |              ^^ not a valid postfix operator
   |
help: use `+= 1` instead
   |
17 -         count++;
17 +         count += 1;
   |
```

- **원인**: C·Java·JavaScript의 습관대로 `count++`를 썼습니다. Rust에는 `++`와 `--` 연산자가 없습니다(Q7.2). 이 에러에는 에러 코드가 없습니다(코드를 읽는 구문 분석 단계에서 납니다).
- **해결**: 컴파일러의 `help`대로 `count += 1;`로 씁니다. `++count`처럼 앞에 붙여도 같은 이유로 컴파일되지 않습니다.

### 4.2 반환 타입을 `_`로 두고 추론을 기대함 — E0121

파일: [`compile_fail/e0121_return_type_placeholder.rs`](compile_fail/e0121_return_type_placeholder.rs)

```text
error[E0121]: the placeholder `_` is not allowed within types on item signatures for return types
  --> compile_fail/e0121_return_type_placeholder.rs:13:29
   |
13 | fn total(prices: &[u32]) -> _ {
   |                             ^ not allowed in type signatures
   |
help: replace with the correct return type
   |
13 - fn total(prices: &[u32]) -> _ {
13 + fn total(prices: &[u32]) -> u32 {
   |
```

- **원인**: TypeScript처럼 반환 타입을 본문에서 추론하기를 기대하고 `-> _`로 썼습니다. `_`는 `let v: Vec<_> = …`처럼 함수 본문 안에서는 "여기는 추론해 달라"는 뜻으로 쓸 수 있지만, 함수 같은 항목(item)의 시그니처에는 쓸 수 없습니다(Q7.7).
- **해결**: 컴파일러가 본문을 보고 제안한 `-> u32`를 적습니다. 반환 타입을 적기 어려운 반복자나 클로저라면 `impl Trait`으로 적습니다.

## 5. 연습 문제

### 연습 1. Java가 세는 길이

Java에서 `"한🦀".length()`와 `"한🦀".charAt(1)`이 무엇을 돌려줄지, 01의 방식으로 Rust에서 구해 보세요. 같은 문자열을 Python의 `len`과 Go의 `len`으로 세면 각각 얼마일지도 함께 출력합니다.

<details>
<summary>힌트</summary>

Java의 `length()`는 `s.encode_utf16().count()`(3), `charAt(1)`은 `s.encode_utf16().nth(1)`(서로게이트의 앞쪽 절반 `0xd83e`)에 해당합니다. Python의 `len`은 `s.chars().count()`(2), Go의 `len`은 `s.len()`(7)입니다. 한글 한 글자가 UTF-16 1단위인 데 비해 🦀은 2단위라서, Java에서 두 번째 글자를 `charAt(1)`로 꺼내면 반쪽짜리 값이 나옵니다(Q7.6).

</details>

### 연습 2. `i64`로 바꾸어 더하기

02에 `fn add_wide(a: i32, b: i32) -> i64`를 만들어, 두 `i32`를 오버플로 없이 더해 보세요. `add_wide(2_000_000_000, 2_000_000_000)`이 `4000000000`이어야 합니다.

<details>
<summary>힌트</summary>

`i64::from(a) + i64::from(b)`로 먼저 넓힌 뒤 더합니다. `(a + b) as i64`로 쓰면 `i32`끼리 더하는 단계에서 이미 오버플로가 나므로(디버그 빌드에서 패닉) 소용이 없습니다. `a as i64 + b as i64`도 결과는 같지만, 넓히는 변환에는 값이 바뀌지 않는 `from`을 쓰는 것이 부록 A ch5 2.5절의 지침입니다.

</details>

### 연습 3. 컴파일러가 제안한 타입 확인하기

`compile_fail/e0121_return_type_placeholder.rs`의 본문을 `prices.iter().map(|&p| u64::from(p)).sum::<u64>()`로 바꾸고 다시 컴파일해, 컴파일러가 제안하는 반환 타입이 어떻게 바뀌는지 확인해 보세요.

<details>
<summary>힌트</summary>

`help: replace with the correct return type`의 제안이 `u64`로 바뀝니다. 컴파일러는 본문의 타입을 알고 있지만, 시그니처를 대신 정해 주지는 않고 제안만 합니다(Q7.7).

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rust Programming Language: [3.1절 Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html), [3.2절 Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html), [3.3절 Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html), [3.5절 Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html), [20.5절 Macros](https://doc.rust-lang.org/book/ch20-05-macros.html)
- The Rust Reference: [Numeric types](https://doc.rust-lang.org/reference/types/numeric.html)(`usize`는 최소 16비트)
- 예전 공식 Rust FAQ(보관됨): [Why doesn't Rust have increment and decrement operators?](https://github.com/rust-lang/prev.rust-lang.org/blob/master/en-US/faq.md) — 보관된 저장소 rust-lang/prev.rust-lang.org의 `en-US/faq.md`, 마지막 수정 2018-06-22
- 표준 라이브러리 문서: [`str::chars`](https://doc.rust-lang.org/std/primitive.str.html#method.chars), [`str::encode_utf16`](https://doc.rust-lang.org/std/primitive.str.html#method.encode_utf16), [`Eq`](https://doc.rust-lang.org/std/cmp/trait.Eq.html), [`f64::total_cmp`](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp)

다른 언어

- Java: Java Language Specification(SE 21) [4.2.1](https://docs.oracle.com/javase/specs/jls/se21/html/jls-4.html#jls-4.2.1), [4.12.4](https://docs.oracle.com/javase/specs/jls/se21/html/jls-4.html#jls-4.12.4), [5.1.2](https://docs.oracle.com/javase/specs/jls/se21/html/jls-5.html#jls-5.1.2), [5.2](https://docs.oracle.com/javase/specs/jls/se21/html/jls-5.html#jls-5.2), [6.4](https://docs.oracle.com/javase/specs/jls/se21/html/jls-6.html#jls-6.4), [14.11.3](https://docs.oracle.com/javase/specs/jls/se21/html/jls-14.html#jls-14.11.3), [15.28.1](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.28.1). API 문서: [`String`](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/String.html), [`Formatter`](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Formatter.html), [`BigInteger`](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/math/BigInteger.html). OpenJDK: [Local Variable Type Inference: Style Guidelines](https://openjdk.org/projects/amber/guides/lvti-style-guide)
- JavaScript(MDN): [`const`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const), [`String.prototype.length`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/length), [`Number`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number), [`BigInt`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt), [`Map`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map)
- TypeScript: [Everyday Types — Return Type Annotations](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html)
- Kotlin: [Functions — Return types](https://kotlinlang.org/docs/functions.html)
- Python: [Built-in Types](https://docs.python.org/3/library/stdtypes.html)(정수의 정밀도, 문자열은 코드 포인트의 나열)
- Go: [FAQ — Why does Go not have the ?: operator?](https://go.dev/doc/faq#Does_Go_have_a_ternary_form), [Why are ++ and -- statements?](https://go.dev/doc/faq#inc_dec), [Why does Go not provide implicit numeric conversions?](https://go.dev/doc/faq#conversions), 명세의 [Numeric types](https://go.dev/ref/spec#Numeric_types), [String types](https://go.dev/ref/spec#String_types), [Switch statements](https://go.dev/ref/spec#Switch_statements), [`math/big`](https://pkg.go.dev/math/big)
