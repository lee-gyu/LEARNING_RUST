# ch2. 메모리 안전성

Rust를 소개하는 글은 거의 언제나 "메모리 안전하다"는 말로 시작합니다. 그런데 메모리 안전성이 정확히 무엇을 막는다는 뜻인지, 그것이 왜 보안과 이어지는지, 가비지 컬렉터(garbage collector, GC)를 쓰는 Java나 Go도 메모리 안전하다는데 무엇이 다른지는 잘 설명되지 않습니다. 이 챕터는 메모리 안전성을 다섯 종류로 나눠 정의하고, 종류마다 Rust가 무엇으로 언제 막는지를 예제와 컴파일 에러로 확인합니다. 메모리 버그가 취약점이 되는 과정과 "취약점의 70%"라는 숫자를 읽는 법도 1차 자료로 답합니다.

- 패키지: `faq-ch2-memory-safety`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q2.1](#q21-메모리-안전성이-정확히-무엇인가요) | 메모리 안전성이 정확히 무엇인가요? | 메모리에 닿을 때 언제나 살아 있고, 초기화되었고, 타입이 맞는 객체의 범위 안에만 닿는다는 성질입니다. 보통 다섯 종류로 나눕니다. |
| [Q2.2](#q22-메모리-버그가-왜-보안-취약점이-되나요) | 메모리 버그가 왜 보안 취약점이 되나요? | 자기 것이 아닌 메모리를 읽으면 비밀이 새고, 쓰면 프로그램의 제어 데이터가 바뀌어 공격자의 코드가 실행될 수 있기 때문입니다. |
| [Q2.3](#q23-취약점의-70가-메모리-문제라는-숫자는-어디서-나왔나요) | "취약점의 70%가 메모리 문제"라는 숫자는 어디서 나왔나요? | Microsoft와 Chromium이 자기 C/C++ 코드의 보안 취약점을 센 비율입니다. 모든 버그, 모든 언어의 비율이 아닙니다. |
| [Q2.4](#q24-rust는-메모리-오류를-어떻게-막나요-언제-잡나요) | Rust는 메모리 오류를 어떻게 막나요? 언제 잡나요? | 시간·초기화·타입·데이터 경쟁은 주로 컴파일 시간에, 범위는 대부분 실행 중 경계 검사(bounds check)로 막습니다. |
| [Q2.5](#q25-java-go-python-c도-메모리-안전한데-rust는-무엇이-다른가요) | Java, Go, Python, C#도 메모리 안전한데 Rust는 무엇이 다른가요? | GC 없이 컴파일 시간에 막고, 데이터 경쟁과 null 같은 실행 중 오류도 타입으로 미리 막습니다. |
| [Q2.6](#q26-패닉으로-프로그램이-죽는-것도-안전한-건가요) | 패닉으로 프로그램이 죽는 것도 안전한 건가요? | 메모리 안전성의 뜻에서는 안전합니다. 잘못된 메모리에 닿기 전에 멈추기 때문입니다. 다만 서비스가 멈추는 것은 별개의 문제입니다. |
| [Q2.7](#q27-메모리-안전성-말고-보안에-도움이-되는-것은-무엇인가요) | 메모리 안전성 말고 보안에 도움이 되는 것은 무엇인가요? | 정수 변환과 오버플로(overflow)를 드러내는 산술, null 없는 `Option`, 모든 경우를 다루게 하는 `match` 같은 장치입니다. |
| [Q2.8](#q28-실제로-취약점이-줄었나요) | 실제로 취약점이 줄었나요? | Android는 메모리 안전성 문제의 비율이 2019년 76%에서 2024년 24%로 줄었다고 보고했습니다. 한 조직의 자기 보고이고 비율과 수를 구별해 읽어야 합니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 2.8~2.11절, 4절: 참조와 대여 규칙, 댕글링 참조(dangling reference), 대여 검사기(borrow checker) 에러
- [Part 0 ch8](../../part0-getting-started/ch8-collections/README.md) 2.3·2.6절, 4.5절: 인덱싱과 `get`, 요소를 빌린 채 `push`할 수 없는 이유, 범위 밖 인덱스와 글자 경계의 패닉
- [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.2·2.3·2.11절: `panic!`, 되감기(unwinding), 패닉과 `Result`의 선택
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.7·2.8절: `Rc`의 참조 카운팅, `RefCell`의 실행 중 대여 검사
- [ch1](../ch1-why-rust/README.md) Q1.1·Q1.3: Rust가 풀려는 문제, 조직들이 옮기는 이유의 근거 목록

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 메모리 안전성을 공간·시간·타입·초기화·데이터 경쟁의 다섯 종류로 나눠 설명하고, 종류마다 Rust가 막는 장치와 그 시점(컴파일 시간 또는 실행 중)을 말할 수 있다.
2. 경계 밖 읽기가 비밀 유출로 이어지는 과정을 설명하고, 같은 실수를 Rust로 옮기면 `get`은 거부하고 인덱싱은 패닉으로 멈추는 것을 예제로 확인할 수 있다.
3. 크기 계산의 오버플로, 음수 길이, 비트 재해석처럼 메모리 버그로 이어지는 실수를 Rust가 어떻게 드러내는지 보일 수 있다.
4. "취약점의 70%" 같은 통계가 무엇을 센 숫자인지 설명하고, GC 언어의 메모리 안전성과 Rust의 차이를 들 수 있다.

학습 목표 1~3은 예제와 `compile_fail/`이 뒷받침하고, 4는 2절의 1차 자료로 답합니다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다.

### Q2.1 메모리 안전성이 정확히 무엇인가요?

**짧은 답** 메모리 안전성(memory safety)은 프로그램이 메모리를 읽고 쓸 때 언제나 **살아 있고, 초기화되었고, 그 타입의 값이 들어 있는 객체의, 할당된 범위 안**에만 닿는다는 성질입니다. Google의 2024년 보고서 *Secure by Design: Google's Perspective on Memory Safety*는 이를 어기는 버그를 공간, 시간, 타입, 초기화, 데이터 경쟁의 다섯 종류로 나눕니다.

**왜 그런가** 종류마다 일어나는 일과 막는 방법이 달라서 나눠 보는 것이 쓸모 있습니다. 표의 "Rust에서" 열은 Q2.4에서 자세히 봅니다.

| 종류 | 뜻 | 대표적인 버그 | Rust에서 | 확인할 곳 |
| --- | --- | --- | --- | --- |
| 공간적 안전성(spatial safety) | 객체에 할당된 범위 밖에 닿지 않음 | 버퍼 오버플로(buffer overflow), 경계 밖 읽기·쓰기 | 실행 중 경계 검사(패닉), `get`은 `None` | 예제 01, 02, 03 |
| 시간적 안전성(temporal safety) | 객체의 수명 밖에서 닿지 않음 | 해제 후 사용(use after free), 끝난 함수의 지역 변수 가리키기 | 컴파일 시간 대여 검사 | [Part 0 ch8](../../part0-getting-started/ch8-collections/README.md) 2.6·4.1절(E0502), [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 4.5절(E0106) |
| 타입 안전성(type safety) | 그 타입의 값이 들어 있지 않은 메모리를 그 타입으로 읽지 않음 | 잘못된 포인터 캐스트 | 안전한 코드에서는 메모리를 다른 타입으로 다시 읽을 수 없음 | `compile_fail/e0605_…`, 예제 04 |
| 초기화 안전성(initialization safety) | 초기화하기 전에 읽지 않음 | 초기화하지 않은 변수·버퍼 읽기 | 컴파일 시간 검사 | `compile_fail/e0381_…` |
| 데이터 경쟁 안전성(data-race safety) | 동기화 없이 여러 스레드가 같은 값을 읽고 쓰지 않음 | 반쯤 고친 값을 다른 스레드가 읽음 | 컴파일 시간 대여 규칙과 `Send`/`Sync` | [ch1](../ch1-why-rust/README.md) 예제 03, ch8 `ch8-concurrency-async` |

같은 보고서는 데이터 경쟁을 "엄밀한 메모리 안전성"의 범위에서 일단 뺍니다. 메모리 안전성과 일부만 겹치는 별개의 종류라는 이유이고, 예로 Java는 데이터 경쟁을 막아 주지 않지만 데이터 경쟁이 힙을 망가뜨리지는 않는다는 점을 듭니다. Rust는 안전한 코드에서 데이터 경쟁까지 막으므로, 이 챕터는 다섯 종류를 모두 다룹니다.

**오해와 한계** 메모리 안전성은 "버그가 없다"가 아닙니다. 계산이 틀리거나, 비밀을 담은 데이터를 엉뚱한 사람에게 보내는 논리 오류(logic error)는 메모리 안전성과 상관없이 생깁니다(예제 02의 3번, ch3 `ch3-safety-limits`). 메모리 누수(memory leak)도 메모리 안전성 위반이 아닙니다(ch3).

**더 읽기** Part 0 ch4 2.1절 / ch9 `ch9-c-memory-errors`(종류마다 C 코드와 나란히 보기) / Q2.4

### Q2.2 메모리 버그가 왜 보안 취약점이 되나요?

**짧은 답** 메모리 버그는 프로그램이 자기 것이 아닌 메모리를 읽거나 쓰게 만들고, 그 자리에 무엇이 있는지는 공격자가 영향을 줄 수 있는 경우가 많기 때문입니다. 범위 밖을 **읽으면** 옆에 있던 비밀(암호 키, 개인 정보, 메모리 주소)이 새고, 범위 밖이나 해제된 메모리에 **쓰면** 반환 주소 같은 제어 데이터가 바뀌어 공격자가 고른 코드가 실행될 수 있습니다.

**왜 그런가** MITRE의 약점 분류(CWE)는 대표적인 메모리 버그의 결과를 이렇게 적습니다.

| 약점 | 정의(요약) | 적힌 결과의 예 |
| --- | --- | --- |
| CWE-125 경계 밖 읽기(out-of-bounds read) | 버퍼의 끝 뒤나 시작 앞을 읽음 | 암호 키, 개인 정보, 메모리 주소 같은 비밀 값을 얻음 |
| CWE-787 경계 밖 쓰기(out-of-bounds write) | 버퍼의 끝 뒤나 시작 앞에 씀 | 반환 주소 같은 제어 데이터를 바꿔 예상하지 못한 코드를 실행 |
| CWE-416 해제 후 사용(use after free) | 해제된 메모리를 다시 쓰거나 가리킴 | 해제된 메모리를 읽어 정보가 새거나, 임의 코드 실행으로 이어짐 |

대표적인 사건이 2014년의 Heartbleed(CVE-2014-0160)입니다. 미국 국립취약점데이터베이스(NVD)의 설명에 따르면, OpenSSL 1.0.1부터 1.0.1g 직전까지의 TLS·DTLS 구현이 Heartbeat 확장 패킷을 제대로 처리하지 못해, 원격 공격자가 조작한 패킷으로 버퍼 경계 밖을 읽게 만들어 프로세스 메모리의 민감한 정보(개인 키 포함)를 얻을 수 있었습니다. 예를 들어 요청이 "내용이 64바이트"라고 주장하면 실제 내용이 4바이트여도 64바이트를 복사해 보낸 것입니다. 길이 필드는 16비트였습니다(RFC 6520의 `payload_length`).

[02_trusted_length](examples/02_trusted_length.rs)는 같은 모양의 응답 함수를 Rust로 씁니다. 비밀 키를 요청 내용 **바로 뒤**에 붙여 두고 요청 부분만 가리키는 슬라이스를 넘기면, 길이를 부풀린 요청은 거부됩니다.

```text
2. 길이를 부풀린 요청
  길이 64: 거부 (주장한 길이 64 > 실제 내용 4)
```

슬라이스는 주소와 함께 길이를 들고 다니므로(ch1 Q1.1), 메모리상 바로 옆에 무엇이 있든 슬라이스의 길이 밖은 읽을 수 없습니다. `get`은 범위가 맞지 않으면 `None`을 돌려주고, 인덱싱이었다면 패닉으로 멈춥니다(Q2.6).

**오해와 한계** Rust가 지키는 경계는 "넘겨받은 슬라이스"입니다. 02의 3번처럼 비밀까지 들어 있는 슬라이스를 넘기면 비밀이 그대로 나갑니다. 이것은 메모리 오류가 아니라 논리 오류라서 컴파일러도 경계 검사도 막지 않습니다.

```text
3. 슬라이스를 잘못 정한 경우
  길이 19: 응답 "pingSECRET-KEY-1234"
```

**더 읽기** ch9 `ch9-c-memory-errors`(경계 밖 읽기·쓰기를 C 코드로 재현) / Q2.6

### Q2.3 "취약점의 70%가 메모리 문제"라는 숫자는 어디서 나왔나요?

**짧은 답** 널리 인용되는 70%는 Microsoft(2019년, 해마다 CVE를 부여하는 취약점의 약 70%)와 Chromium 프로젝트(2015년 이후 심각도 높음·치명 보안 버그 912건의 약 70%)가 **자기 제품의 보안 취약점**을 센 비율입니다. 두 코드베이스 모두 C/C++가 중심입니다. 모든 버그의 비율도, 모든 언어로 쓴 프로그램의 비율도 아닙니다.

**왜 그런가** 숫자를 읽을 때는 분자와 분모가 무엇인지부터 봅니다.

| 출처 | 분모(무엇을 셌나) | 분자 | 결과 |
| --- | --- | --- | --- |
| Microsoft MSRC (2019-07-16) | Microsoft가 해마다 CVE를 부여한 취약점 | 그중 메모리 안전성 문제 | 약 70% |
| Chromium (날짜 표기 없음) | 2015년 이후 안정 채널에 영향을 준 심각도 높음·치명 보안 버그 912건 | 그중 메모리 안전성 문제 | 약 70%, 그 절반이 해제 후 사용 |
| Google Android (2024-09-25) | 해마다 Android 취약점 | 그중 메모리 안전성 문제 | 2019년 76%, 2024년 24% |

비슷한 그림은 특정 회사 밖의 자료에서도 보입니다. MITRE가 공개된 취약점(CVE) 자료로 만든 2025 CWE Top 25에서 경계 밖 쓰기(CWE-787)는 5위, 해제 후 사용(CWE-416)은 7위, 경계 밖 읽기(CWE-125)는 8위, 널 포인터 역참조(CWE-476)는 13위입니다. 이 목록은 언어를 가리지 않고 센 것이라 비율로 바꿔 말할 수는 없지만, 메모리 버그가 위험한 약점의 윗자리를 차지한다는 점은 같습니다.

**오해와 한계**

- 비율은 수가 아닙니다. Android의 비율이 줄어든 것은 메모리 안전성 문제가 줄어든 것과 함께 다른 종류의 문제의 몫이 커진 결과일 수 있습니다(Q2.8).
- 분모가 무엇인지에 따라 숫자가 달라집니다. Chromium은 심각도가 높음·치명인 버그만 셌고, MSRC는 CVE가 부여된 취약점을 셌습니다. 심각도 낮은 버그까지 세면 비율은 달라질 수 있습니다.
- 분모가 "C/C++ 중심 제품의 보안 취약점"이므로, Java나 Go로 쓴 서비스에 그대로 옮겨 적용할 수 없습니다(ch1 Q1.3).
- CWE Top 25는 2024-06-01부터 2025-06-01 사이에 공개된 CVE 레코드 39,080건을 근본 원인(CWE)으로 분류해 만든 목록입니다. 회사 한 곳의 자기 집계가 아니라는 점에서 앞의 세 숫자를 보완하지만, 언어별로 나눈 자료는 아닙니다.

**더 읽기** ch1 Q1.3 / Q2.8

### Q2.4 Rust는 메모리 오류를 어떻게 막나요? 언제 잡나요?

**짧은 답** 종류마다 장치가 다릅니다. 시간적 안전성(소유권(ownership)과 대여 검사기), 초기화 안전성, 타입 안전성, 데이터 경쟁은 **컴파일 시간**에 막습니다. 공간적 안전성은 인덱스 값이 실행 중에야 정해지는 경우가 많으므로 대부분 **실행 중 경계 검사**로 막고, 반복자처럼 인덱스를 쓰지 않는 방법으로 검사할 일 자체를 줄입니다. 소유자가 여럿인 경우처럼 컴파일 시간 규칙으로 표현할 수 없는 곳은 `Rc`·`RefCell` 같은 실행 중 장치로 대신합니다. 이 보장은 안전한(safe) Rust 코드에 해당합니다.

**왜 그런가**

| 장치 | 막는 종류 | 시점 | 어기면 | 이 저장소에서 |
| --- | --- | --- | --- | --- |
| 소유권, 대여 규칙, 라이프타임(lifetime) | 시간적, 데이터 경쟁 | 컴파일 | E0382, E0499, E0502, E0106, E0597, E0515 등 | Part 0 ch4 4절, [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 4.4·4.5절 |
| 확정 초기화 검사(definite initialization) | 초기화 | 컴파일 | E0381 | 4.1절 |
| 참조를 다른 타입으로 캐스트할 수 없음, `transmute`는 `unsafe` | 타입 | 컴파일 | E0605, E0133 | 4.2절, [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 4절 |
| 참조는 널이 될 수 없고 `Option`으로 부재를 표시 | 널 역참조 | 컴파일 | E0308 등 | ch5 `ch5-no-null-no-exceptions` |
| `Send`/`Sync` | 데이터 경쟁 | 컴파일 | E0277 | [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 4절 |
| 경계 검사 | 공간적 | 실행 중 | 패닉, 또는 `get`이 `None` | 예제 01, 03 |
| `Rc`/`Arc`의 참조 카운트, `RefCell`의 대여 검사 | 시간적(소유자가 여럿), 대여 규칙 | 실행 중 | `RefCell`은 패닉 | Part 0 ch14 2.7·2.8절, 4.6절 |

[01_spatial_checks](examples/01_spatial_checks.rs)는 Part 0 ch8에서 본 인덱싱과 `get`의 동작을 공간적 안전성의 눈으로 다시 봅니다. 범위가 길이를 조금이라도 넘으면 앞부분만 잘라 주지도 않고 통째로 거부하고, 문자열은 바이트 범위가 글자 경계에 맞지 않아도 거부합니다. 잘린 바이트가 `&str`이 되면 다른 코드가 올바른 UTF-8이라고 믿고 읽게 되기 때문입니다.

공간적 안전성을 주로 실행 중에 검사하는 이유는 간단합니다. 02의 "주장한 길이"처럼 인덱스가 네트워크나 파일에서 들어오면 컴파일러가 그 값을 알 수 없습니다. 대신 검사가 필요 없다는 것을 컴파일러가 증명할 수 있으면 최적화 단계에서 검사를 지우기도 합니다([Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.15절). 반복자로 훑으면 처음부터 끝까지만 가므로 범위를 벗어날 인덱스가 아예 생기지 않습니다.

표의 마지막 줄은 컴파일 시간 규칙의 빈틈을 채우는 실행 중 장치입니다. 대여 검사기는 순환하는 참조 그래프 같은 구조를 허용하지 않으므로, 표준 라이브러리는 참조 카운팅으로 실행 중 비용을 치르며 이런 구조를 안전하게 만드는 API를 둡니다(Google 보고서). 비용이 들고, `RefCell`은 규칙을 어기면 컴파일 에러가 아니라 패닉으로 알립니다.

이 표의 보장에는 조건이 있습니다. Rustonomicon은 안전한 Rust만 쓰면 타입 안전성과 메모리 안전성을 걱정할 일이 없고 댕글링 포인터, 해제 후 사용 같은 미정의 동작(undefined behavior)을 겪지 않는다고 설명합니다. 거꾸로 말하면 `unsafe` 블록 안에서는 그 보장을 프로그래머가 대신 지켜야 합니다(ch3 `ch3-safety-limits`).

**다른 언어에서는** C의 배열 인덱싱과 C++의 기본 인덱싱(`operator[]`)은 경계를 검사하지 않습니다. C++에는 검사하는 `at()`이 있고, 표준 라이브러리에도 검사 장치가 있습니다. GCC의 표준 라이브러리(libstdc++)는 최적화하지 않은 빌드에서 문자열 경계 검사 같은 전제 조건 검사를 기본으로 켜고, 디버그 모드(`_GLIBCXX_DEBUG`)로 빌드하면 일부 반복자 오류(예: 이미 파괴된 컨테이너를 가리키는 반복자의 역참조)까지 잡습니다. 다만 최적화한 빌드에서는 기본으로 꺼져 있고, 시간적 안전성 전체를 보장하지는 않습니다. Google 보고서는 C++가 시간적 안전성을 포함한 엄밀한 메모리 안전성을 갖추는 쪽으로 발전할 현실적인 길이 없다고 봅니다. 같은 종류의 버그를 찾으려면 새니타이저(sanitizer) 같은 도구를 켜고 테스트에서 그 경로를 실제로 실행해야 합니다(ch9 `ch9-c-memory-errors`, ch12 `ch12-modern-cpp-enough`).

**오해와 한계** 컴파일 시간 검사는 보수적입니다. 실제로는 안전한 코드도 규칙으로 증명할 수 없으면 거부하므로, 코드를 다시 짜야 할 때가 있습니다(ch15 `ch15-pain-points`). 실행 중 경계 검사에는 비용이 있습니다(ch14 `ch14-performance`).

**더 읽기** Part 0 ch4 4절, ch8 4.5절 / ch3, ch9 / Q2.6

### Q2.5 Java, Go, Python, C#도 메모리 안전한데 Rust는 무엇이 다른가요?

**짧은 답** 메모리 오류를 막는다는 점은 같습니다. 다른 점은 세 가지입니다. Rust는 시간적 안전성을 GC 없이 컴파일 시간에 얻고, 데이터 경쟁까지 컴파일 시간에 막으며, null 역참조처럼 GC 언어에서 실행 중 예외로 나타나는 오류도 타입으로 미리 막습니다. Google 보고서는 Rust를 GC나 모든 곳에 적용하는 참조 카운팅 같은 실행 중 장치 없이, 넓은 범위의 코드에서 시간적 안전성을 주는 유일한 성숙한 언어로 꼽습니다.

**왜 그런가**

| | GC 언어(Java, C#, Go, Python) | Rust |
| --- | --- | --- |
| 공간적 | 인덱스마다 실행 중 검사, 범위를 벗어나면 예외나 패닉 | 같음(패닉, 또는 `get`이 `None`) |
| 시간적 | 닿을 수 있는 객체는 GC가 해제하지 않음 | 컴파일 시간 대여 검사. GC 없음 |
| 데이터 경쟁 | 막지 않음. Java는 경쟁이 있어도 힙이 망가지지 않지만, Go는 여러 워드로 된 값의 경쟁이 메모리 손상으로 이어질 수 있다고 문서에 적음 | 안전한 코드에서는 컴파일 에러 |
| null | 실행 중 예외(Java `NullPointerException` 등). C#은 nullable 참조 형식으로 컴파일 시간 경고를 냄 | 참조는 널이 될 수 없고, 없을 수 있는 값은 `Option`(컴파일 에러) |

공간적 안전성은 GC 언어도 Rust와 같은 방식으로 지킵니다. Go 명세는 범위를 벗어난 인덱스가 실행 중에 패닉을 일으킨다고, Java 언어 명세는 `ArrayIndexOutOfBoundsException`을 던진다고, Python 문서는 `IndexError`를, .NET 문서는 `IndexOutOfRangeException`을 낸다고 적습니다. 차이는 나머지 줄에 있습니다.

- **시간적 안전성의 비용**: GC는 실행 중에 돌며 시간과 메모리를 쓰고, 해제 시점을 프로그래머가 정하지 않습니다(ch1 Q1.1, ch4 `ch4-memory-without-gc`).
- **데이터 경쟁**: Java의 경우는 앞의 Google 보고서가, Go의 경우는 Go 메모리 모델 문서가 밝힌 내용입니다. 어느 쪽이든 틀린 값이 나오는 것을 컴파일러가 미리 막지는 않습니다(ch8 `ch8-concurrency-async`).
- **null**: Java 문서는 `NullPointerException`을 "객체가 필요한 곳에 null을 쓰려 할 때 던진다"고 설명합니다. 메모리를 망가뜨리지는 않지만 실행 중에야 드러납니다. C#의 nullable 참조 형식(`string?`)은 이 문제를 컴파일 시간에 잡으려는 장치로, 최근 .NET 템플릿은 기본으로 켭니다. 다만 Microsoft 문서가 밝히듯 컴파일러의 경고일 뿐 실행 동작은 그대로이고, `!` 연산자로 경고를 무시할 수 있습니다. Rust에서는 `Option<T>`를 `T`처럼 쓰는 코드가 경고가 아니라 타입 에러입니다(ch5 `ch5-no-null-no-exceptions`).

**오해와 한계** GC 언어로 충분히 안전하고 생산적인 경우가 많습니다(ch1 Q1.6). 또 어느 언어든 메모리 안전하지 않은 코드를 부르는 부분은 그 언어의 보장 밖에 있습니다. Google 보고서에 따르면 Java·Kotlin은 JNI, Go는 `unsafe` 패키지, Rust는 `unsafe` 블록으로 그런 코드의 경계가 분명히 드러납니다. 경계를 표시하는 것 자체는 Rust만의 특징이 아닙니다(ch3, ch17 `ch17-interop-migration`).

**더 읽기** ch1 Q1.1, Q1.8 / ch4, ch5, ch8

### Q2.6 패닉으로 프로그램이 죽는 것도 안전한 건가요?

**짧은 답** 메모리 안전성의 뜻에서는 안전합니다. 패닉은 잘못된 메모리에 닿기 **전에** 멈추고, 스택을 되감으며(unwinding) 지역 값들을 정리한 뒤 프로그램을 끝냅니다. 위험한 것은 C의 미정의 동작처럼 망가진 상태로 **계속 실행되는** 쪽입니다. 다만 서비스가 멈추는 것 자체는 가용성의 문제이므로, 외부 입력 때문에 생기는 실패는 패닉보다 `Result`로 거부하는 편이 낫습니다.

**왜 그런가** [03_panic_stops](examples/03_panic_stops.rs)는 02의 응답 함수를 `get` 대신 인덱싱으로 쓰고 요청 세 개를 차례로 처리합니다.

```text
요청 1 처리 시작
  응답 "ping"
  연결 1 정리
요청 2 처리 시작
  연결 2 정리
```

```text
thread 'main' (스레드 번호) panicked at appd-b/ch2-memory-safety/examples/03_panic_stops.rs:27:13:
range end index 64 out of range for slice of length 4
```

두 번째 요청에서 범위 검사가 실패해 패닉하고, 프로그램은 종료 코드 101로 끝납니다. 응답은 한 바이트도 나가지 않았고, 세 번째 요청은 처리되지 않았습니다. "연결 2 정리"가 출력된 것은 되감기 중에 `Connection`이 버려지며 `Drop`이 실행되었기 때문입니다(Part 0 ch9 2.3절). The Book도 계속 실행하면 안전하지 않거나 해로울 수 있는 경우에는 `panic!`이 최선일 수 있다고 설명합니다.

**다른 언어에서는** GC 언어의 처리되지 않은 예외도 비슷하게 멈춥니다. C에서는 같은 실수가 대개 멈추지 않고 옆 메모리를 복사해 응답으로 보냅니다(Q2.2의 Heartbleed).

**오해와 한계**

- 공격자가 패닉을 일부러 일으킬 수 있다면 서버를 멈추게 하는 서비스 거부(denial of service)가 됩니다(ch3 `ch3-safety-limits`). 그래서 외부 입력을 다루는 코드는 02처럼 `get`과 `Result`로 거부하는 편이 낫습니다(연습 2, [부록 A ch18](../../appd-a/ch18-dont-panic/README.md)).
- `panic = "abort"`로 빌드하면 되감지 않고 바로 끝나므로 `Drop`이 실행되지 않습니다(Part 0 ch9 2.3절).

**더 읽기** Part 0 ch9 2.2·2.3·2.11절 / 부록 A ch18 / ch3

### Q2.7 메모리 안전성 말고 보안에 도움이 되는 것은 무엇인가요?

**짧은 답** 메모리 버그로 이어지기 쉬운 실수를 일찍 드러내는 장치들입니다. 실패할 수 있는 정수 변환과 오버플로를 `Option`·`Result`로 알리는 산술, 비트 재해석 대신 뜻이 정해진 변환 함수, null 대신 `Option`, 버리면 경고가 나는 `Result`, 모든 경우를 다루게 하는 `match`가 그렇습니다.

**왜 그런가** C에서 버퍼 오버플로로 이어지는 흔한 길 하나는 크기 계산입니다. CWE-190(정수 오버플로)은 이 약점이 메모리 할당이나 복사의 크기·위치를 정할 때 보안에 치명적이 될 수 있다고 적습니다. [04_size_and_conversion](examples/04_size_and_conversion.rs)은 이런 계산을 Rust로 해 봅니다.

```text
1. 크기 곱셈의 오버플로
  1,000개 × 16바이트 = Some(16000)
  (usize::MAX / 8 + 1)개 × 16바이트 = None
2. 음수 길이
  usize::try_from(-1) = Err(TryFromIntError(NegOverflow))
```

- `checked_mul`은 오버플로를 `None`으로 알립니다. C의 부호 없는 곱셈은 오버플로가 나면 조용히 작은 값으로 돌아가고(래핑), 그 작은 크기로 할당한 버퍼에 큰 데이터를 쓰게 됩니다.
- 부호 있는 수를 길이(`usize`)로 바꾸는 변환은 저절로 일어나지 않고, `try_from`은 음수를 거부합니다.
- 잘라 내는 변환(`300_i32 as u8`은 44)은 `as`로 코드에 적어야만 일어납니다.
- 비트를 다른 타입으로 읽을 때는 `f32::from_bits`, `u32::from_le_bytes`처럼 뜻이 정해진 함수를 씁니다. 참조를 다른 타입으로 캐스트하는 코드는 컴파일되지 않습니다(4.2절).

`Option`, `Result`, `match`는 ch5 `ch5-no-null-no-exceptions`와 ch7 `ch7-syntax-type-surprises`에서 다룹니다.

**오해와 한계** 그냥 `*`나 `+`로 계산하면 디버그 빌드는 오버플로가 나면 패닉하지만 릴리스 빌드는 기본적으로 래핑합니다(ch3). `as`는 여전히 조용히 잘라 냅니다. 그리고 인증을 빠뜨리거나 권한을 잘못 검사하는 논리 오류는 어떤 타입 검사로도 막지 못합니다.

**더 읽기** ch3, ch5, ch7, ch10 `ch10-undefined-behavior`

### Q2.8 실제로 취약점이 줄었나요?

**짧은 답** Google Android 팀은 2019년부터 Android 취약점 가운데 메모리 안전성 문제의 비율을 해마다 공개해 왔습니다. 그 비율은 2019년 76%에서 2024년 24%로 줄었고, 2025년 자료에서 처음으로 20% 아래가 되었다고 보고했습니다. 비율만이 아니라 수도 줄고 있다고 밝혔습니다. 다만 한 조직의 자기 보고이고, 같은 기간에 기존 C/C++ 코드를 더 안전하게 만드는 노력도 함께 했다고 적었습니다.

**왜 그런가** Android 팀의 2024년 글은 감소의 원인을 새 코드를 메모리 안전한 언어로 쓴 데서 찾습니다. 취약점은 코드가 오래될수록 지수적으로 줄어들므로(반감기), 새로 쓰는 코드가 메모리 안전하면 오래된 C/C++ 코드를 다시 쓰지 않아도 메모리 안전성 취약점이 빠르게 줄어든다는 설명입니다. 2025년 글은 Rust 코드의 메모리 안전성 취약점 밀도가 Android의 C/C++ 코드보다 1000배 넘게 낮다고 보고했습니다. 이 숫자는 Android 플랫폼의 Rust 약 500만 줄에서 찾은 잠재적 취약점 1건(공개 전에 고침)으로 낸 보수적 추정치(100만 줄당 0.2건)를 C/C++의 과거 자료(100만 줄당 1,000건 가까이)와 비교한 것입니다.

**오해와 한계**

- 비율과 수를 구별해야 합니다. 2024년 글의 각주는 2023년에 메모리 안전성 문제의 **비율**은 계속 줄었지만 **수**는 조금 늘었다고 밝힙니다. 그해 취약점 보고가 유난히 많았기 때문이라고 설명합니다.
- 2024년의 24%는 글을 쓴 9월 시점의 값입니다. 같은 글의 차트에서 2024년 메모리 안전성 취약점 수(36건)는 한 해 전체로 외삽한 값이고, 9월 보안 게시판 시점에는 27건이었습니다.
- 같은 글은 결과가 시뮬레이션보다 좋았던 까닭이 메모리 안전하지 않은 기존 코드를 더 안전하게 만드는 노력 때문일 수도 있다고 덧붙입니다. 언어만 바꾼 효과를 따로 잰 실험이 아닙니다.
- "1000배"는 Rust 쪽 표본이 1건이라 오차가 큰 추정입니다. Android 팀도 이를 보수적 추정이라고 부르며, 크기 차이(자릿수)를 보여 주는 숫자로 읽는 편이 맞습니다. 그 1건이 `unsafe` Rust의 버퍼 오버플로(CVE-2025-48530)였다는 점은 ch3에서 다룹니다.

**더 읽기** ch1 Q1.3, Q1.7 / ch3, ch17

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_spatial_checks](examples/01_spatial_checks.rs) | 배열·범위·문자열에서 범위를 벗어난 `get`이 `None`을 돌려줌, 반복자로 훑기 | Q2.1, Q2.4 |
| [02_trusted_length](examples/02_trusted_length.rs) | 주장한 길이를 믿는 응답 함수: 슬라이스 밖은 거부, 슬라이스를 잘못 정하면 비밀이 나감 | Q2.2, Q2.4 |
| [03_panic_stops](examples/03_panic_stops.rs) | 인덱싱이 범위 검사에서 패닉하고, 되감기 중에 `Drop`이 실행됨 (종료 코드 101) | Q2.6 |
| [04_size_and_conversion](examples/04_size_and_conversion.rs) | 크기 곱셈의 오버플로, 음수 길이, 잘라 내는 변환, 비트 재해석 | Q2.7 |

### 01_spatial_checks — 범위를 벗어난 접근

```sh
cargo run -p faq-ch2-memory-safety --example 01_spatial_checks
```

```text
1. 인덱스 하나
  scores.get(2)  = Some(91)
  scores.get(5)  = None
2. 범위
  scores.get(1..3)  = Some([85, 91])
  scores.get(3..10) = None
3. 문자열: 바이트 범위가 글자 경계에 맞아야 함
  바이트 길이 6, 글자 수 2
  name.get(0..3) = Some("한")
  name.get(0..1) = None
4. 인덱스 없이 훑기
  합계 400, 최고점 Some(91)
```

- 길이 5인 배열의 인덱스 5와 범위 `3..10`은 모두 배열 밖이므로 `None`입니다. 범위는 앞부분만 잘라 주지 않고 통째로 거부합니다.
- "한"은 UTF-8로 3바이트이므로 `0..1`은 글자 중간에서 끊기는 범위입니다. 같은 범위를 인덱싱(`&name[0..1]`)으로 자르면 패닉합니다(Part 0 ch8 4.5절).

### 02_trusted_length — 주장한 길이를 믿는 버그

```sh
cargo run -p faq-ch2-memory-safety --example 02_trusted_length
```

```text
1. 정상 요청
  길이 4: 응답 "ping"
2. 길이를 부풀린 요청
  길이 64: 거부 (주장한 길이 64 > 실제 내용 4)
3. 슬라이스를 잘못 정한 경우
  길이 19: 응답 "pingSECRET-KEY-1234"
```

- 요청은 길이 2바이트(빅 엔디언)와 내용으로 이뤄집니다. `split_first_chunk::<2>()`는 앞의 2바이트를 배열로, 나머지를 슬라이스로 나눠 주고, 요청이 2바이트보다 짧으면 `None`입니다.
- `buffer_with_secret`은 요청 바로 뒤에 비밀 키를 붙인 버퍼를 만듭니다. 1·2번은 요청 부분(`&memory[..end]`)만 넘기고, 3번은 실수로 버퍼 전체를 넘깁니다.

### 03_panic_stops — 패닉은 범위 밖을 읽기 전에 멈춘다

```sh
cargo run -p faq-ch2-memory-safety --example 03_panic_stops
```

일부러 패닉으로 끝나며 종료 코드는 101입니다. 표준 출력과 표준 에러는 Q2.6에 실었습니다. 표준 에러는 빈 줄로 시작하고, 그다음 줄의 괄호 안 스레드 번호는 실행마다 다르며, 끝에 `note: run with RUST_BACKTRACE=1 ...` 줄이 붙습니다.

- `heartbeat_reply`는 02와 같은 일을 인덱싱으로 합니다. 두 번째 요청의 `&payload[..claimed]`에서 `claimed`(64)가 `payload`의 길이(4)보다 커서 패닉합니다.
- `Connection`은 버려질 때 "정리"를 출력하는 타입입니다. 패닉 뒤에도 "연결 2 정리"가 나오는 것으로 되감기 중에 지역 값이 버려진다는 것을 확인할 수 있습니다.

### 04_size_and_conversion — 정수 실수를 드러내기

```sh
cargo run -p faq-ch2-memory-safety --example 04_size_and_conversion
```

```text
1. 크기 곱셈의 오버플로
  1,000개 × 16바이트 = Some(16000)
  (usize::MAX / 8 + 1)개 × 16바이트 = None
2. 음수 길이
  usize::try_from(-1) = Err(TryFromIntError(NegOverflow))
3. 잘라 내는 변환은 코드에 보이게
  300_i32 as u8 = 44
  u8::try_from(300) = Err(TryFromIntError(PosOverflow))
4. 비트를 다른 타입으로 읽기
  f32::from_bits(0x3f80_0000) = 1
  u32::from_le_bytes([1, 0, 0, 0]) = 1
```

- `usize::MAX / 8 + 1`에 16을 곱하면 `usize`의 범위를 넘으므로 `checked_mul`이 `None`을 돌려줍니다.
- `TryFromIntError`의 디버그 형식은 음수 때문이면 `NegOverflow`, 너무 커서면 `PosOverflow`를 보여 줍니다.
- `0x3f80_0000`은 IEEE 754 단정밀도에서 1.0의 비트 패턴입니다.

## 4. 자주 하는 실수

이 챕터의 `compile_fail/`은 메모리 안전성의 종류 가운데 초기화와 타입을 맡습니다. 시간적 안전성의 컴파일 에러는 앞 챕터에 있습니다. 요소를 빌린 채 `push`하면 재할당으로 예전 공간이 해제되므로 이를 막는 [Part 0 ch8](../../part0-getting-started/ch8-collections/README.md) 4.1절(E0502)이 해제 후 사용의 모양이고, 함수 안에서 만든 값의 참조를 돌려주는 [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 4.5절(E0106)이 끝난 함수의 지역 변수를 가리키는 모양입니다. 같은 버그를 C 코드와 나란히 놓은 비교는 ch9 `ch9-c-memory-errors`에 있습니다.

### 4.1 일부 경로에서만 초기화한 변수 — E0381

파일: [`compile_fail/e0381_possibly_uninitialized.rs`](compile_fail/e0381_possibly_uninitialized.rs)

```text
error[E0381]: used binding `limit` is possibly-uninitialized
  --> compile_fail/e0381_possibly_uninitialized.rs:22:19
   |
18 |     let limit: usize;
   |         ----- binding declared here but left uninitialized
19 |     if verbose {
   |        ------- if this `if` condition is `false`, `limit` is not initialized
20 |         limit = 100;
21 |     }
   |      - an `else` arm might be missing here, initializing `limit`
22 |     println!("최대 {limit}줄까지 출력합니다"); // E0381
   |                     ^^^^^ `limit` used here but it is possibly-uninitialized
   |
   = note: when checking initialization, the compiler describes possible control-flow paths without evaluating whether branch conditions can actually have the values shown
```

- **원인**: `verbose`가 거짓인 경로에서는 `limit`에 아무것도 들어가지 않습니다. C라면 값이 정해지지 않은 변수를 읽게 되어 그 자리의 메모리에 남아 있던 데이터가 나올 수 있습니다. 컴파일러는 조건이 실제로 거짓이 될 수 있는지 따지지 않고, 그런 경로가 코드에 있는지만 봅니다.
- **해결**: 모든 경로에서 값을 넣습니다. `let limit = if verbose { 100 } else { 20 };`처럼 `if`를 표현식으로 쓰면 빠뜨릴 수 없습니다.

### 4.2 참조를 다른 타입으로 캐스트 — E0605

파일: [`compile_fail/e0605_reference_cast.rs`](compile_fail/e0605_reference_cast.rs)

```text
error[E0605]: non-primitive cast: `&u32` as `&f32`
  --> compile_fail/e0605_reference_cast.rs:19:23
   |
19 |     let value: &f32 = &bits as &f32; // E0605
   |                       ^^^^^^^^^^^^^ an `as` expression can only be used to convert between primitive types or to coerce to a specific trait object
   |
   = note: casting reference expression `&bits` because `&` binds tighter than `as`
```

- **원인**: C에서 `*(float *)&bits`처럼 포인터를 다른 타입의 포인터로 캐스트하면 그 메모리를 새 타입으로 읽게 됩니다. 그 비트가 새 타입의 올바른 값인지는 아무도 확인하지 않으므로, Q2.1 표의 "잘못된 포인터 캐스트"가 이것입니다. Rust의 `as`는 숫자 사이처럼 원시 타입끼리의 변환에만 쓸 수 있고 참조의 타입을 바꾸지 못합니다. 원시 포인터를 거치면(`&bits as *const u32 as *const f32`) 캐스트 자체는 안전한 코드에서 컴파일되지만, 그 포인터를 따라가 값을 읽는 것은 `unsafe`라서 `unsafe` 블록 없이 쓰면 E0133(``dereference of raw pointer is unsafe and requires unsafe block``)이 납니다(Part 0 ch19 2.2절, 4.1절).
- **해결**: 뜻이 정해진 안전한 변환 함수를 씁니다. `f32::from_bits(bits)`(예제 04)는 어떤 `u32`든 올바른 `f32`가 되므로 안전합니다.
- **`transmute`로 우회하면**: `std::mem::transmute`는 크기만 맞으면 어떤 타입으로든 비트를 다시 읽게 해 주는 `unsafe` 함수라서, `unsafe` 블록 없이 부르면 [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 4.1절에서 본 E0133이 납니다. Rust 레퍼런스는 `bool`이 0이나 1이 아니거나 참조가 널인 값을 만드는 것을 미정의 동작으로 꼽으므로, 그런 값이 나오지 않게 하는 책임이 프로그래머에게 넘어옵니다. `u32`를 `f32`로 바꾸는 이 경우에는 `unsafe { std::mem::transmute(bits) }`로 감싸도 1.100.0의 컴파일러가 `warning: unnecessary transmute`와 함께 `f32::from_bits(bits)`로 바꾸라고 알려 줍니다.

## 5. 연습 문제

### 연습 1. 길이가 정확히 맞아야 하는 요청

02의 `heartbeat_reply`는 주장한 길이가 실제 내용보다 **짧으면** 앞부분만 돌려줍니다. 길이가 정확히 같을 때만 받고, 다르면 "주장한 길이 2 ≠ 실제 내용 4" 같은 에러를 돌려주게 고쳐 보세요.

<details>
<summary>힌트</summary>

`payload.get(..claimed)` 대신 `if claimed != payload.len() { return Err(...); }`로 먼저 확인한 뒤 `Ok(payload)`를 돌려주면 됩니다. 요청 `buffer_with_secret(2, b"ping")`의 앞부분(`&memory[..end]`)으로 시험해 보세요.

</details>

### 연습 2. 패닉 대신 거부하고 계속 처리하기

03의 `heartbeat_reply`를 02처럼 `Result`를 돌려주게 바꾸고, `handle`이 에러를 출력한 뒤 다음 요청으로 넘어가게 해 보세요. 세 요청이 모두 처리되고 "모든 요청 처리 끝"까지 출력되며 종료 코드가 0이 되면 됩니다.

<details>
<summary>힌트</summary>

헤더는 `request.split_first_chunk::<2>()`로, 내용은 `payload.get(..claimed)`로 자르고 각각 `ok_or`로 `Result`를 만듭니다. `handle`에서는 `match heartbeat_reply(request) { Ok(reply) => …, Err(e) => println!("  거부: {e}") }`처럼 처리합니다. "연결 N 정리"는 여전히 요청마다 한 번씩 나옵니다.

</details>

### 연습 3. 크기를 검사하는 할당 함수

04의 `buffer_size`를 써서 `fn alloc_records(count: usize, record_size: usize) -> Result<Vec<u8>, String>`을 만들어 보세요. 전체 크기에서 오버플로가 나면 에러를, 아니면 그 크기의 0으로 채운 `Vec`을 돌려줍니다. `alloc_records(4, 8)`과 `alloc_records(usize::MAX, 2)`로 시험해 보세요.

<details>
<summary>힌트</summary>

`buffer_size(count, record_size).ok_or("크기 오버플로")?`으로 크기를 얻고 `Ok(vec![0u8; size])`를 돌려줍니다. 함수의 에러 타입은 `String`이지만 `?`가 `From`으로 `&str`을 `String`으로 바꿔 주므로 그대로 통과합니다(Part 0 ch9 2.8절). 앞의 호출은 `Ok`(길이 32), 뒤의 호출은 `Err("크기 오버플로")`이 됩니다. 오버플로가 나지 않더라도 아주 큰 크기는 할당이 실패할 수 있으니 시험에는 작은 값을 씁니다.

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

메모리 안전성의 정의와 Rust

- [Secure by Design: Google's Perspective on Memory Safety](https://research.google/pubs/secure-by-design-googles-perspective-on-memory-safety/) (Alex Rebert, Christoph Kern, Google, 2024-03-04): 다섯 종류의 분류, Java와 데이터 경쟁, 언어별 안전성 개요
- [The Rustonomicon: Meet Safe and Unsafe](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html)
- [The Rust Reference: Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html)
- [The Rust Programming Language 4장: Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html), [9.3절: To panic! or Not to panic!](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)

취약점과 통계

- [CWE-125 Out-of-bounds Read](https://cwe.mitre.org/data/definitions/125.html), [CWE-787 Out-of-bounds Write](https://cwe.mitre.org/data/definitions/787.html), [CWE-416 Use After Free](https://cwe.mitre.org/data/definitions/416.html), [CWE-190 Integer Overflow or Wraparound](https://cwe.mitre.org/data/definitions/190.html) (MITRE)
- [2025 CWE Top 25 Most Dangerous Software Weaknesses](https://cwe.mitre.org/top25/archive/2025/2025_cwe_top25.html) (MITRE, 2025-12-15 갱신), [Methodology](https://cwe.mitre.org/top25/archive/2025/2025_methodology.html)
- [RFC 6520: TLS and DTLS Heartbeat Extension](https://www.rfc-editor.org/rfc/rfc6520) (2012-02)
- [CVE-2014-0160](https://nvd.nist.gov/vuln/detail/CVE-2014-0160) (NVD, 2014-04-07 공개)
- [A proactive approach to more secure code](https://www.microsoft.com/en-us/msrc/blog/2019/07/a-proactive-approach-to-more-secure-code) (Microsoft MSRC, 2019-07-16)
- [Memory safety](https://www.chromium.org/Home/chromium-security/memory-safety/) (Chromium 프로젝트, 날짜 표기 없음)
- [Eliminating Memory Safety Vulnerabilities at the Source](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html) (Google 보안 블로그, 2024-09-25)
- [Rust in Android: move fast and fix things](https://blog.google/security/rust-in-android-move-fast-fix-things/) (Google, 2025-11-13)

다른 언어

- [The Go Programming Language Specification: Index expressions](https://go.dev/ref/spec#Index_expressions), [The Go Memory Model](https://go.dev/ref/mem)
- [Java Language Specification 15.10.4: Run-Time Evaluation of Array Access Expressions](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.10.4), [`NullPointerException`](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/NullPointerException.html)
- [Python `IndexError`](https://docs.python.org/3/library/exceptions.html#IndexError)
- [C# Nullable reference types](https://learn.microsoft.com/en-us/dotnet/csharp/nullable-references)
- [.NET `IndexOutOfRangeException`](https://learn.microsoft.com/en-us/dotnet/api/system.indexoutofrangeexception)
- [libstdc++ Debug Mode](https://gcc.gnu.org/onlinedocs/libstdc++/manual/debug_mode.html): 디버그 모드의 안전한 반복자, [Macros](https://gcc.gnu.org/onlinedocs/libstdc++/manual/using_macros.html): `_GLIBCXX_ASSERTIONS`와 `_GLIBCXX_DEBUG`의 기본값
