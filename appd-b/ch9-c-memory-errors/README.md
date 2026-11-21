# ch9. C의 메모리 오류와 Rust

C와 C++로 일해 온 사람은 메모리 오류의 이름을 이미 압니다. 해제 후 사용, 이중 해제(double free), 댕글링 포인터(dangling pointer), 버퍼 오버플로(buffer overflow), 널 역참조(null pointer dereference), 반복자 무효화(iterator invalidation). 이 챕터는 그 오류를 하나씩 C·C++ 코드로 실제로 만들어 새니타이저(sanitizer)가 무엇을 보고하는지 보고, 같은 구조를 Rust로 옮기면 무엇이 언제 막는지 나란히 봅니다. 메모리 안전성의 뜻과 종류는 [ch2](../ch2-memory-safety/README.md) Q2.1·Q2.4에서 정리했으므로, 여기서는 오류마다 구체적인 코드에 집중합니다.

- 패키지: `faq-ch9-c-memory-errors`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 비교 코드를 확인한 컴파일러: Apple clang 21.0.0, macOS(aarch64). 다른 컴파일러(GCC, MSVC)에서는 옵션과 메시지가 다릅니다.
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q9.1](#q91-해제-후-사용use-after-free은-어떻게-막나요) | 해제 후 사용(use-after-free)은 어떻게 막나요? | 해제는 소유자만 하고, 해제한 뒤에는 그 값에 닿는 이름이 남지 않습니다. 참조는 값보다 오래 살 수 없습니다. 모두 컴파일 시간에 검사합니다. |
| [Q9.2](#q92-이중-해제는-어떻게-막나요) | 이중 해제는 어떻게 막나요? | 해제할 책임이 값마다 하나뿐이고 넘기면 함께 넘어가므로, 두 번 해제하는 코드는 컴파일되지 않습니다. |
| [Q9.3](#q93-지역-변수의-주소를-돌려주는-것-같은-댕글링-포인터는-어떻게-되나요) | 지역 변수의 주소를 돌려주는 것 같은 댕글링 포인터는 어떻게 되나요? | 지역 변수의 참조를 돌려주면 컴파일 에러입니다. 값 자체(소유권)를 돌려줍니다. |
| [Q9.4](#q94-버퍼-오버플로와-경계-밖-읽기는-어떻게-막나요) | 버퍼 오버플로와 경계 밖 읽기는 어떻게 막나요? | 배열과 슬라이스가 길이를 알고, 인덱싱은 실행 중에 경계를 검사해 패닉으로 멈춥니다. `get`은 `None`을 돌려줍니다. |
| [Q9.5](#q95-초기화하지-않은-메모리를-읽는-것은-어떻게-막나요) | 초기화하지 않은 메모리를 읽는 것은 어떻게 막나요? | 초기화하지 않은 변수를 읽으면 컴파일 에러이고, 안전한 API는 초기화하지 않은 메모리를 보여 주지 않습니다. |
| [Q9.6](#q96-널-포인터-역참조는-어떻게-되나요) | 널 포인터 역참조는 어떻게 되나요? | 참조는 널이 될 수 없고, 없을 수 있는 값은 `Option`이라 확인 없이는 쓸 수 없습니다. |
| [Q9.7](#q97-순회하는-중에-컨테이너를-고치는-반복자-무효화는-어떻게-되나요) | 순회하는 중에 컨테이너를 고치는 반복자 무효화는 어떻게 되나요? | 순회가 컨테이너를 빌린 동안에는 고칠 수 없어서 컴파일 에러입니다. 모았다가 나중에 고치거나 `retain`을 씁니다. |
| [Q9.8](#q98-c-문자열의-버그strcpy-널-종료-누락-서식-문자열는-어떻게-되나요) | C 문자열의 버그(`strcpy`, 널 종료 누락, 서식 문자열)는 어떻게 되나요? | 문자열이 길이를 알고 스스로 늘어나며 0 바이트로 끝을 표시하지 않습니다. 서식 문자열은 리터럴만 받습니다. |
| [Q9.9](#q99-malloc의-크기-계산과-void-형-변환-실수는-어떻게-되나요) | `malloc`의 크기 계산과 `void*` 형 변환 실수는 어떻게 되나요? | 할당 크기는 타입이 정하고 넘치면 패닉합니다. `void*` 같은 무엇으로든 바뀌는 포인터가 없습니다. |
| [Q9.10](#q910-메모리-누수는-막아-주나요) | 메모리 누수는 막아 주나요? | 아닙니다. 누수는 안전하다고 보며, 대부분은 스코프가 끝날 때 해제되어 덜 생길 뿐입니다(ch3 Q3.4). |
| [Q9.11](#q911-이-검사들은-실행-비용이-드나요) | 이 검사들은 실행 비용이 드나요? | 소유권과 대여 검사는 컴파일 시간이라 실행 비용이 없고, 경계 검사처럼 실행 중에 하는 검사에는 비용이 있습니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md): 소유권(ownership), 이동, 대여 규칙, 댕글링 참조(2.11절, 4.5절)
- [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md): 라이프타임(lifetime), 결과가 인자보다 오래 사용될 때(4.4절의 E0597), 함수 안에서 만든 값의 참조 반환(4.5절의 E0515)
- [Part 0 ch8](../../part0-getting-started/ch8-collections/README.md): 벡터, 요소를 빌린 채 `push`할 수 없는 이유(2.6절, 4.1절)
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.6절: `Drop`과 `std::mem::drop`
- [ch2](../ch2-memory-safety/README.md): 메모리 안전성의 종류(Q2.1), Rust가 언제 잡는가(Q2.4), 패닉도 안전한가(Q2.6)

비교 코드를 직접 돌려 보려면 C 컴파일러가 필요합니다. 돌리지 않아도 README의 보고 줄로 내용을 따라올 수 있습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 해제 후 사용·이중 해제·댕글링 포인터를 C 코드와 새니타이저 보고로 설명하고, Rust에서는 소유권과 대여 규칙이 어느 줄을 컴파일 에러로 만드는지 짚을 수 있다.
2. 버퍼 오버플로·`strcpy`·서식 문자열 같은 C 문자열 버그가 Rust의 슬라이스와 `String`, `println!`에서 왜 생기지 않는지 설명하고, C의 고정 크기 버퍼 작업을 Rust로 옮길 수 있다.
3. 널 역참조와 반복자 무효화를 Rust에서 어떻게 표현하거나 피하는지 설명하고, 순회하면서 컨테이너를 고치는 코드를 Rust답게 바꿔 쓸 수 있다.
4. 메모리 오류의 종류마다 C·C++에서 생기는 방식, Rust의 장치, 잡는 시점(컴파일·실행 중)을 표로 정리하고, 누수와 실행 비용처럼 Rust가 약속하지 않는 것을 구별할 수 있다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다. 이 챕터의 "다른 언어에서는"은 주로 C·C++입니다. 비교 코드의 실행 방법과 보고 전체는 3절에 모았습니다.

### Q9.1 해제 후 사용(use-after-free)은 어떻게 막나요?

**짧은 답** Rust에서 힙 메모리를 해제하는 것은 그 값의 소유자뿐이고, 해제는 소유자가 스코프를 벗어나거나 `drop`에 값을 넘길 때 일어납니다. 해제한 뒤에는 그 값을 가리키는 이름이 남지 않습니다. 값을 빌린 참조는 대여 검사기(borrow checker)가 값보다 오래 살 수 없게 막습니다. 그래서 해제한 메모리를 읽는 코드는 실행하기 전에 컴파일 에러가 됩니다.

**왜 그런가** C에서는 `malloc`이 돌려준 포인터를 여러 변수가 함께 들 수 있고, `free`는 그중 하나로 메모리를 돌려줄 뿐 나머지 포인터를 지우지 않습니다. [compare/01_use_after_free.c](compare/01_use_after_free.c)는 이름을 해제한 뒤 같은 포인터로 읽고, AddressSanitizer(ASan)가 `heap-use-after-free`를 보고합니다. Rust에서는 해제할 책임(소유권)이 값마다 하나이고 넘기면 함께 넘어갑니다. [01_one_owner_one_free](examples/01_one_owner_one_free.rs)의 1번은 버퍼를 세 함수에 차례로 넘깁니다.

```text
1. 넘겨 가며 쓰기
    [해제] 요청 버퍼 (16바이트)
  합계 112, 지금까지 해제 1번
```

해제는 마지막 소유자인 `checksum`이 끝날 때 한 번 일어나고, 그 뒤로 `buf`라는 이름은 쓸 수 없습니다. compare/01과 같은 구조, 곧 `drop(값)` 뒤에 그 값을 쓰는 코드가 E0382로 막힌다는 것은 Part 0 ch14 2.6절에서 봤습니다. 값을 빌려 둔 참조가 값보다 오래 살려 하면 E0597이 나고(Part 0 ch10 4.4절, 부록 A ch14), 빌려 준 값을 `drop`에 넘기면 E0505가 납니다(부록 A ch14).

**다른 언어에서는** ch2 Q2.3에서 본 Chromium 프로젝트의 분석에서는 심각도 높은 보안 버그의 약 70%가 메모리 안전성 문제였고, 그 절반이 해제 후 사용이었습니다. C++의 `unique_ptr`는 해제를 자동으로 하지만, `get()`으로 얻은 원시 포인터를 따로 들고 있다가 `reset()` 뒤에 쓰는 일은 막지 못합니다. 이 환경에서 그렇게 쓴 짧은 코드를 ASan으로 실행하니 `heap-use-after-free`가 보고됐습니다(모던 C++의 남은 문제는 [ch12](../ch12-modern-cpp-enough/README.md)). ASan은 이런 접근을 실행 중에 찾지만, 그 경로가 실제로 실행되어야 보고합니다.

**오해와 한계** `unsafe` 코드에서 원시 포인터(raw pointer)를 역참조하면 해제 후 사용을 만들 수 있고, 그것을 막는 책임은 `unsafe`를 쓴 사람에게 있습니다([ch3](../ch3-safety-limits/README.md) Q3.2). 또 해제 시점을 컴파일러가 정하는 대신, 여러 곳에서 함께 쓰는 값은 `Rc`처럼 실행 중에 참조 수를 세는 타입을 골라야 하는 설계 부담이 생깁니다([ch4](../ch4-memory-without-gc/README.md) Q4.7).

**더 읽기** Part 0 ch4 2.5·2.8절 / Part 0 ch10 4.4절 / [부록 A ch14](../../appd-a/ch14-lifetimes/README.md) / ch2 Q2.3

### Q9.2 이중 해제는 어떻게 막나요?

**짧은 답** 해제할 책임이 값마다 하나뿐이고, 값을 넘기면 책임도 함께 넘어가기 때문입니다. `drop(buffer)`를 두 번 부르면 두 번째 줄에서 "이미 옮겨 간 값을 쓴다"는 E0382가 납니다(4.1절). The Book 4.1절도 두 변수가 같은 힙 메모리를 가리키면 둘 다 해제하려는 이중 해제 버그가 생기므로, 대입 뒤에 원래 변수를 더는 유효하지 않게 본다고 설명합니다.

**왜 그런가** [compare/02_double_free.c](compare/02_double_free.c)는 같은 포인터를 두 번 `free`하고, ASan이 `attempting double-free`를 보고합니다. 실제 코드에서는 두 곳이 서로 "내가 해제한다"고 믿을 때 생깁니다. 01의 2·3번은 Rust에서 같은 상황을 어떻게 쓰는지 봅니다.

```text
2. clone하면 메모리도 해제도 둘
    [해제] 원본 (4바이트)
    [해제] 사본 (4바이트)
  지금까지 해제 3번
3. free(p); p = NULL; 대신 Option::take
    [해제] 캐시 (8바이트)
  slot은 비었나: true, 지금까지 해제 4번
```

둘 다 값을 써야 하면 `clone`으로 메모리를 따로 만들고, 그러면 해제도 따로 일어납니다. C에서 이중 해제를 피하려고 쓰는 `free(p); p = NULL;` 관용구는 `Option::take`에 해당합니다(`take` 자체는 [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md) 2.9절의 `drop(self.sender.take())`에서 봤습니다). 값을 꺼내 해제하면서 칸을 `None`으로 비우므로, 다시 꺼내도 해제할 것이 없습니다.

**다른 언어에서는** C++에서도 같은 원시 포인터를 두 `unique_ptr`에 넘기면 둘 다 해제하려 하므로 이중 해제가 됩니다(이 환경에서 ASan이 `attempting double-free`를 보고). Rust에서 원시 포인터로 소유자를 만드는 `Box::from_raw`는 `unsafe` 함수라서, 안전한 코드에서는 같은 일을 쓸 수 없습니다(E0133). `unsafe` 블록에서 두 번 부르면 Rust도 컴파일되고 이중 해제가 됩니다. 안전한 코드에서 소유자를 넘기는 길은 이동뿐이고, 이동은 원래 변수를 쓸 수 없게 하는 파괴적 이동(destructive move)이라 4.1절처럼 컴파일 시간에 걸립니다(C++의 이동과의 차이는 [ch11](../ch11-cpp-raii-move/README.md)).

**오해와 한계** `ManuallyDrop`처럼 해제를 직접 다루는 도구로 `unsafe` 코드를 쓰면 이중 해제를 만들 수 있습니다. 4.1절의 컴파일러는 `clone`을 제안하지만, 사본을 만들어 버리는 것은 이중 해제를 고치는 길이 아닙니다. 두 번째 `drop`이 왜 필요했는지(필요 없었는지)를 따져야 합니다.

**더 읽기** Part 0 ch4 2.5절, 4.1절 / Part 0 ch14 2.6절 / ch4 Q4.2

### Q9.3 지역 변수의 주소를 돌려주는 것 같은 댕글링 포인터는 어떻게 되나요?

**짧은 답** 지역 변수의 참조를 돌려주면 컴파일 에러입니다. 라이프타임을 적지 않았으면 E0106(Part 0 ch4 4.5절), 함수 안에서 만든 값의 참조를 돌려주려 하면 E0515(Part 0 ch10 4.5절)입니다. 해법은 참조 대신 값 자체(소유권)를 돌려주는 것입니다. The Book 4.2절은 컴파일러가 참조가 결코 댕글링 참조가 되지 않도록 보장한다고 설명합니다.

**왜 그런가** 함수가 끝나면 그 지역 변수의 스택 자리는 다음 호출이 다시 씁니다. [compare/03_return_stack_address.c](compare/03_return_stack_address.c)는 지역 변수의 주소를 포인터 변수를 거쳐 돌려주고, 부른 쪽이 그 주소를 읽습니다. Rust에서는 함수가 돌려주는 참조가 어떤 인자에서 빌린 것인지 라이프타임으로 정해져야 하고, 함수 안의 지역 변수는 그 후보가 될 수 없습니다.

**다른 언어에서는** clang은 `return &local;`처럼 바로 돌려주면 `-Wreturn-stack-address` 경고를 내지만, compare/03처럼 포인터 변수를 한 단계 거치면 경고하지 않습니다(이 환경에서 확인). ASan의 `stack-use-after-return` 검출은 Clang 문서에 따르면 Linux에서는 기본으로 켜져 있지만, 이 macOS 환경에서는 `ASAN_OPTIONS=detect_stack_use_after_return=1`을 주어야 보고했고 주지 않으면 아무 보고 없이 끝났습니다.

**오해와 한계** 값을 돌려주면 복사가 많아질 것 같지만, `String`이나 `Vec`을 돌려주는 것은 힙 버퍼를 가리키는 작은 값(포인터, 길이, 용량)을 옮기는 일입니다. 호출한 쪽이 버퍼를 다시 쓰고 싶으면 `&mut String`을 인자로 받아 채우는 방법도 있습니다.

**더 읽기** Part 0 ch4 2.11절, 4.5절 / Part 0 ch10 4.5절

### Q9.4 버퍼 오버플로와 경계 밖 읽기는 어떻게 막나요?

**짧은 답** Rust의 배열과 슬라이스는 길이를 알고, 읽기든 쓰기든 모든 접근이 그 길이 안에서만 일어납니다. 인덱싱(`v[i]`)은 실행 중에 경계를 검사(bounds check)해 넘으면 패닉으로 멈추고, `get(i)`는 `None`을 돌려줍니다. 어느 쪽이든 옆 메모리를 읽거나 쓰지 않습니다. 이 검사는 컴파일 시간이 아니라 실행 중에 합니다.

**왜 그런가** [compare/04_heap_buffer_overflow.c](compare/04_heap_buffer_overflow.c)는 원소 4개짜리 배열의 합을 구하며 `i <= 4`로 다섯 번 읽어, ASan이 `heap-buffer-overflow`와 `READ of size 4`를 보고합니다. 경계 밖 쓰기는 compare/07(Q9.8)이 보입니다. C의 포인터는 자기가 가리키는 블록의 길이를 모르므로 컴파일러가 검사할 수 없습니다. Rust의 슬라이스 `&[T]`는 시작 주소와 길이를 함께 담은 값이라 접근할 때마다 길이와 비교할 수 있습니다. 범위를 벗어난 `get`이 `None`을 돌려주는 것은 ch2 예제 01, 인덱싱이 패닉으로 멈추는 것은 ch2 예제 03에서 봤습니다. C의 고정 크기 버퍼에 복사하는 작업을 Rust로 옮기는 법은 Q9.8에서 봅니다.

**다른 언어에서는** C의 배열 인덱싱과 C++의 `operator[]`는 경계를 검사하지 않고, C++의 `at()`은 검사해 예외를 던집니다. 표준 라이브러리의 디버그 모드 같은 검사 장치는 ch2 Q2.4에서 정리했습니다. Rust는 검사하는 쪽이 기본이고, 검사하지 않는 `get_unchecked`는 `unsafe`입니다.

**오해와 한계** 경계를 넘는 접근이 패닉으로 바뀌는 것이지 사라지는 것은 아니므로, 서비스라면 그 패닉이 요청 하나를 멈추는 것도 장애입니다(ch2 Q2.6, ch3 Q3.7). 경계 검사에는 실행 비용이 있습니다(Q9.11).

**더 읽기** ch2 Q2.4·Q2.6, 예제 01·03 / Part 0 ch8 2.3절

### Q9.5 초기화하지 않은 메모리를 읽는 것은 어떻게 막나요?

**짧은 답** 초기화하지 않았을 수 있는 변수를 읽으면 컴파일 에러(E0381)입니다(ch2 4.1절). 힙 메모리도 안전한 API로는 초기화하지 않은 부분을 볼 수 없습니다. `vec![0; n]`은 0으로 채우고, `Vec::with_capacity(n)`은 공간만 잡을 뿐 `push`하기 전에는 그 자리에 닿을 수 없습니다. 초기화하지 않은 메모리를 직접 다루는 `MaybeUninit`은 읽을 때 `unsafe`가 필요합니다.

**왜 그런가** 초기화하지 않은 값은 그 자리에 남아 있던 아무 비트나 될 수 있고, C에서는 읽는 것 자체가 미정의 동작(undefined behavior)이 될 수 있습니다. Rust는 "모든 경로에서 대입된 뒤에만 읽는다"를 컴파일러가 흐름을 따라 검사합니다.

**다른 언어에서는** C의 `malloc`이 돌려준 메모리는 초기화되어 있지 않습니다. ASan은 초기화하지 않은 읽기를 찾지 않고, 그것을 찾는 MemorySanitizer는 Clang 문서에 따르면 Linux·NetBSD·FreeBSD에서만 지원되며, 이 환경의 Apple clang은 `-fsanitize=memory`를 `unsupported option`으로 거부했습니다. C에서 초기화하지 않은 변수를 읽는 코드는 미정의 동작을 다루는 [ch10](../ch10-undefined-behavior/README.md)에서 봅니다.

**오해와 한계** 초기화 검사는 보수적이라, 실제로는 모든 경로에서 대입되는데도 컴파일러가 증명하지 못하면 거부합니다. 그럴 때는 선언과 동시에 값을 주거나 `if`·`match`를 표현식으로 써서 한 번에 대입합니다.

**더 읽기** ch2 Q2.1, 4.1절 / [ch10](../ch10-undefined-behavior/README.md)

### Q9.6 널 포인터 역참조는 어떻게 되나요?

**짧은 답** Rust의 참조는 널이 될 수 없습니다. `std::ptr::null()`이 만드는 것은 참조가 아니라 원시 포인터라서 참조 자리에 넣으면 E0308이 납니다(4.2절). 없을 수 있는 값은 `Option<&T>`로 돌려주고, 쓰기 전에 `None`인 경우를 처리해야 컴파일됩니다([ch5](../ch5-no-null-no-exceptions/README.md) Q5.1).

**왜 그런가** [compare/05_null_deref.c](compare/05_null_deref.c)의 `find`는 못 찾으면 `NULL`을 돌려주는데, 부르는 쪽이 확인하지 않고 필드를 읽습니다. UndefinedBehaviorSanitizer(UBSan)가 `member access within null pointer`를 먼저 알리고, 이어서 ASan이 `SEGV on unknown address 0x000000000000`을 보고합니다. C에서는 "없음"과 "있음"이 같은 포인터 타입이라 확인을 빼먹어도 타입이 맞습니다. Rust에서는 둘이 `Option`의 두 배리언트라 타입이 다릅니다. `Option<&T>`는 널 포인터 최적화(null pointer optimization) 덕분에 포인터 하나와 같은 크기입니다([ch1](../ch1-why-rust/README.md) 예제 02).

**다른 언어에서는** UBSan이 이 줄을 `runtime error`로 보고한 것처럼 C에서 널 포인터 역참조는 미정의 동작입니다. 이 실행에서는 그 뒤에 ASan이 SEGV를 보고하며 멈췄지만, 미정의 동작에서 무엇이 일어날지는 언어가 정하지 않습니다([ch10](../ch10-undefined-behavior/README.md)).

**오해와 한계** 원시 포인터 `*const T`는 널일 수 있고, 역참조하려면 `unsafe`가 필요합니다. C 함수가 돌려준 포인터를 받는 FFI 코드에서는 널인지 확인하는 책임이 다시 프로그래머에게 옵니다.

**더 읽기** ch5 Q5.1·Q5.2 / [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.2절(원시 포인터)

### Q9.7 순회하는 중에 컨테이너를 고치는 반복자 무효화는 어떻게 되나요?

**짧은 답** `for x in &v`는 순회하는 동안 `v`를 빌리므로, 그 안에서 `v.push(…)`로 고치려 하면 E0502로 컴파일되지 않습니다(Part 0 ch8 4.1절, 맵에서 지우는 경우는 Part 1 ch4 4.4절). 같은 일은 더할 것을 모았다가 순회가 끝난 뒤 `extend`하거나, 인덱스로 순회하거나, 지우는 일이면 `retain`으로 씁니다.

**왜 그런가** [compare/06_iterator_invalidation.cpp](compare/06_iterator_invalidation.cpp)는 범위 기반 `for`로 `vector`를 돌며 `push_back`합니다. C++ 표준 초안(\[vector.modifiers\])은 `push_back`이 재할당을 일으키면 원소를 가리키던 모든 참조·포인터·반복자가 무효가 되고, 재할당이 없어도 넣은 자리 뒤와 끝 반복자는 무효가 된다고 정합니다. 범위 기반 `for`는 시작할 때 잡아 둔 끝 반복자와 비교하므로 compare/06은 용량과 상관없이 미정의 동작이고, 이 실행에서는 재할당이 일어나 ASan이 `heap-use-after-free`를 보고했습니다. Rust의 대여 규칙은 "빌린 동안에는 고칠 수 없다"이므로, 재할당이 실제로 일어나든 아니든 그럴 수 있는 코드를 받아들이지 않습니다. [03_modify_while_iterating](examples/03_modify_while_iterating.rs)은 같은 일을 세 가지로 씁니다.

```text
1. 모았다가 나중에 더하기
  [1, 2, 3, 4]
2. 인덱스로 순회하며 더하기
  [1, 2, 3, 4]
3. 조건에 맞는 것 지우기
  [1, 3, 5]
```

**다른 언어에서는** C++에서 같은 코드는 미리 `reserve`해 재할당이 일어나지 않으면 이 환경에서 겉으로 멀쩡히 돌았고 ASan도 보고하지 않았습니다. 그래도 무효가 된 끝 반복자와 비교하므로 여전히 미정의 동작이고, 원소 수나 용량에 따라 가끔만 터지는 버그가 됩니다. Java의 `ArrayList`는 순회 중에 구조가 바뀌면 반복자가 대개 `ConcurrentModificationException`을 던져 실행 중에 알립니다(ch4 Q4.5).

**오해와 한계** 대여 검사기는 컨테이너가 어떻게 바뀔지 따지지 않고 "빌린 동안에는 고칠 수 없다"로만 판단하므로, 재할당이 일어나지 않아 메모리 오류로 이어지지 않을 코드도 거부합니다. 2번처럼 인덱스로 순회하면 컴파일되지만, 순회 중에 넣은 원소를 같은 순회에서 볼지 말지는 프로그래머가 정해야 하는 논리 문제로 남습니다.

**더 읽기** Part 0 ch8 2.6절, 4.1절 / [Part 1 ch4](../../part1-standard-lib/ch4-collections/README.md) 예제 01, 4.4절 / ch4 Q4.5

### Q9.8 C 문자열의 버그(`strcpy`, 널 종료 누락, 서식 문자열)는 어떻게 되나요?

**짧은 답** Rust의 `String`과 `&str`은 길이를 함께 들고 다니므로 끝을 0 바이트로 표시하지 않고, `strlen`처럼 끝을 찾아 메모리를 훑을 일이 없습니다. `String`은 모자라면 스스로 늘어나서 정해진 칸을 넘어 쓰지 않습니다. `println!`의 서식 문자열은 컴파일 시간에 해석하는 리터럴만 받으므로, 사용자 입력이 서식 지정자로 해석되는 일이 없습니다(4.3절).

**왜 그런가** [compare/07_strcpy_overflow.c](compare/07_strcpy_overflow.c)는 8바이트 칸에 길이를 확인하지 않고 이름을 `strcpy`해서 ASan이 `stack-buffer-overflow`를 보고합니다. [02_buffers_and_strings](examples/02_buffers_and_strings.rs)는 같은 일을 Rust로 옮깁니다.

```text
1. 8바이트 칸에 이름 넣기
  8바이트 복사: "hana-kim"
  칸 밖 name.get(8): None
2. String은 길이를 알고 늘어남
  "hana-kim-long": 길이 13, 용량이 길이 이상인가: true
  "a\0b"의 길이: 3
3. 입력은 서식이 아니라 데이터
  입력: %s%s%n
```

1번은 길이를 먼저 비교해 들어갈 만큼만 복사합니다. `copy_from_slice`는 두 슬라이스의 길이가 다르면 덮어쓰는 대신 패닉하므로, 길이를 맞추지 않은 코드는 넘치지 않고 멈춥니다. 2번의 `"a\0b"`처럼 0 바이트도 그냥 한 바이트일 뿐이라 문자열을 끝내지 않습니다.

**다른 언어에서는** clang은 상수 문자열을 바로 `strcpy`해 넘칠 것이 확실하면 `-Wfortify-source` 경고를, `printf(user)`처럼 서식 자리에 리터럴이 아닌 값을 넣으면 `-Wformat-security` 경고를 기본으로 냈습니다(이 환경에서 확인). 둘 다 경고일 뿐이고, compare/07처럼 인자를 거치면 넘침을 컴파일할 때 알 수 없습니다.

**오해와 한계** 1번처럼 바이트 단위로 자르면 한글처럼 여러 바이트인 글자의 중간에서 잘릴 수 있습니다. 글자 경계에서 자르는 방법은 [Part 1 ch3](../../part1-standard-lib/ch3-strings-fmt/README.md) 예제 05에 있습니다. C 함수에 문자열을 넘길 때는 끝에 0 바이트를 붙이고 중간의 0 바이트를 막는 `CString`으로 바꿔야 합니다([Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 예제 03).

**더 읽기** ch7 Q7.5·Q7.8 / Part 1 ch3 예제 05 / Part 1 ch14 예제 03

### Q9.9 `malloc`의 크기 계산과 `void*` 형 변환 실수는 어떻게 되나요?

**짧은 답** Rust에서 할당 크기는 타입이 정합니다. `Box::new(값)`은 그 타입의 크기로, `vec![0u64; n]`과 `Vec::<T>::with_capacity(n)`은 원소 크기 × 개수로 바이트 수를 직접 계산하고, 뒤의 계산이 넘치면 `capacity overflow`로 패닉합니다(이 환경에서 확인). 무엇으로든 바뀌는 `void*` 같은 포인터가 없고, 제네릭이 타입을 끝까지 들고 갑니다.

**왜 그런가** C의 `malloc`은 바이트 수만 받고 `void*`를 돌려주며, C에서는 그 값을 어떤 객체 포인터에든 대입할 수 있습니다. 그래서 `int *p = malloc(sizeof(short));`처럼 크기를 잘못 적어도 이 환경의 clang은 `-Wall -Wextra`로도 경고하지 않았고, 실행하면 ASan이 `heap-buffer-overflow`를 보고했습니다. 개수와 크기를 곱하는 계산이 넘쳐 작은 블록을 잡는 버그와, 그것을 `checked_mul`로 막는 방법은 ch2 예제 04에서 봤습니다.

**다른 언어에서는** C++의 `new T[n]`과 `std::vector<T>`는 타입으로 크기를 정하므로 이 문제가 C보다 적습니다. Rust의 할당도 그쪽에 가깝고, 원시 포인터를 다른 타입으로 바꾸는 `as`는 쓸 수 있지만 그 포인터를 역참조하려면 `unsafe`가 필요합니다.

**오해와 한계** 할당 크기가 넘치지 않더라도 아주 큰 할당은 메모리가 모자라 실패할 수 있고, 기본 동작은 프로세스 중단입니다(ch5 Q5.3의 `handle_alloc_error`). 외부 입력으로 크기를 정한다면 상한을 먼저 검사합니다.

**더 읽기** ch2 예제 04 / ch5 Q5.3

### Q9.10 메모리 누수는 막아 주나요?

**짧은 답** 아닙니다. Rust는 누수를 메모리 안전성 위반으로 보지 않으며, `mem::forget`이나 `Rc` 순환 참조로 안전한 코드에서도 누수를 만들 수 있습니다(ch3 Q3.4). 다만 대부분의 값은 소유자의 스코프가 끝날 때 해제되므로, `free`를 빼먹어서 생기는 누수는 드뭅니다.

**왜 그런가** 해제를 빼먹는 것은 메모리를 잘못 읽거나 쓰지 않으므로 해제 후 사용·이중 해제와 위험의 종류가 다릅니다. ch3 Q3.4에 그 이유와 예제가 있습니다.

**다른 언어에서는** 누수를 찾는 LeakSanitizer에 대해 Clang 문서는 macOS에서 `ASAN_OPTIONS=detect_leaks=1`로 켤 수 있다고 적지만, 이 환경의 Apple clang 21.0.0(aarch64)에서는 `detect_leaks is not supported on this platform`이 나왔습니다. 문서와 실제가 다를 수 있으니 쓰는 도구에서 직접 확인합니다.

**오해와 한계** 오래 도는 서버에서는 순환 참조나 끝없이 커지는 캐시처럼 논리적인 누수가 여전히 문제가 됩니다. 이런 것은 `Weak`로 순환을 끊거나 크기에 상한을 두어 설계로 막습니다.

**더 읽기** ch3 Q3.4 / ch4 Q4.7

### Q9.11 이 검사들은 실행 비용이 드나요?

**짧은 답** 소유권과 대여 검사(Q9.1~Q9.3, Q9.7)는 컴파일 시간에 끝나므로 실행 비용이 없습니다. 경계 검사(Q9.4)나 `RefCell`의 대여 검사, `Rc`의 참조 수 관리처럼 실행 중에 하는 검사에는 비용이 있습니다. 얼마인지는 코드와 측정에 따라 다르므로 [ch14](../ch14-performance/README.md)에서 다룹니다.

**왜 그런가** 컴파일 시간 검사는 통과한 프로그램에 아무 코드도 더하지 않습니다. 반면 인덱스가 실행 중에야 정해지는 접근은 실행 중에 비교할 수밖에 없습니다.

**다른 언어에서는** C·C++의 새니타이저도 실행 중 검사이고, Clang 문서는 ASan의 일반적인 속도 저하를 2배로 적습니다. 그래서 새니타이저는 보통 테스트 빌드에서만 켭니다. Rust의 경계 검사는 릴리스 빌드(`-O`)에서도 켜져 있다는 점이 다릅니다(이 환경에서 `-O`로 컴파일해도 범위를 벗어난 인덱싱이 패닉했습니다).

**오해와 한계** 경계 검사 비용이 문제가 되는 반복문은 인덱스 대신 반복자를 쓰는 것처럼 검사가 필요 없는 모양으로 바꿀 수 있습니다. 측정 없이 "Rust는 느리다/빠르다"고 단정하지 않습니다.

**더 읽기** ch2 Q2.4 / [ch14](../ch14-performance/README.md)

### 오류별 정리

| 오류 종류 | C·C++에서 생기는 방식 | Rust의 장치 | 잡는 시점 |
| --- | --- | --- | --- |
| 해제 후 사용 | `free`·`delete` 뒤에도 포인터가 남음(compare/01) | 소유권(drop 뒤 사용은 E0382), 대여 검사(E0597, E0505) | 컴파일 |
| 이중 해제 | 같은 포인터를 두 곳에서 해제(compare/02) | 해제 책임이 하나, 이동(E0382, 4.1절) | 컴파일 |
| 댕글링 포인터 | 끝난 함수의 지역 변수 주소를 돌려줌(compare/03) | 라이프타임(E0106, E0515) | 컴파일 |
| 버퍼 오버플로·경계 밖 접근 | 길이를 모르는 포인터로 읽기(compare/04)·쓰기(compare/07) | 길이를 아는 슬라이스, 경계 검사 | 실행 중(패닉) |
| 초기화하지 않은 읽기 | `malloc` 메모리, 대입하지 않은 지역 변수 | 초기화 검사(E0381), 초기화를 강제하는 API | 컴파일 |
| 널 역참조 | `NULL`을 확인 없이 역참조(compare/05) | 널이 없는 참조, `Option`(E0308, 4.2절) | 컴파일 |
| 반복자 무효화 | 순회 중 재할당(compare/06) | 빌린 동안 고칠 수 없음(E0502) | 컴파일 |
| 문자열 버그 | 0 바이트 종료, `strcpy`, `printf(user)`(compare/07) | 길이를 아는 `String`, 리터럴 서식(4.3절) | 컴파일·실행 중 |
| 크기 계산 | `malloc(개수 * 크기)`, `void*` | 타입이 정하는 할당, `capacity overflow` 패닉 | 컴파일(타입)·실행 중(크기 넘침 패닉) |
| 누수 | `free`를 빼먹음 | 스코프 끝의 자동 해제. 막지는 않음 | 막지 않음 |

종류별 개념은 ch2 Q2.1·Q2.4의 표와 함께 보면 됩니다. 이 표는 그것을 구체적인 C 버그 이름으로 다시 나눈 것입니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_one_owner_one_free](examples/01_one_owner_one_free.rs) | 넘겨도 해제는 한 번, `clone`이면 둘, `free(p); p = NULL;` 대신 `Option::take` | Q9.1, Q9.2 |
| [02_buffers_and_strings](examples/02_buffers_and_strings.rs) | 고정 크기 칸에 들어갈 만큼만 복사, 늘어나는 `String`, 서식이 아닌 데이터로서의 입력 | Q9.4, Q9.8 |
| [03_modify_while_iterating](examples/03_modify_while_iterating.rs) | 순회하면서 더하기·지우기를 Rust답게 쓰는 세 방법 | Q9.7 |

### 01_one_owner_one_free — 해제 책임은 하나

```sh
cargo run -p faq-ch9-c-memory-errors --example 01_one_owner_one_free
```

```text
1. 넘겨 가며 쓰기
    [해제] 요청 버퍼 (16바이트)
  합계 112, 지금까지 해제 1번
2. clone하면 메모리도 해제도 둘
    [해제] 원본 (4바이트)
    [해제] 사본 (4바이트)
  지금까지 해제 3번
3. free(p); p = NULL; 대신 Option::take
    [해제] 캐시 (8바이트)
  slot은 비었나: true, 지금까지 해제 4번
```

- `Buffer`의 `Drop`이 "[해제]"를 출력하고 횟수를 셉니다. 해제가 몇 번, 언제 일어나는지 보는 장치입니다.
- 1번의 `fill`은 버퍼를 받아 고친 뒤 돌려주고(`buf`), `checksum`은 받아서 돌려주지 않습니다. 그래서 `checksum`이 끝날 때 해제됩니다.
- 합계 112는 16바이트를 모두 7로 채운 값(16 × 7)입니다.

### 02_buffers_and_strings — 넘치지 않는 버퍼와 문자열

```sh
cargo run -p faq-ch9-c-memory-errors --example 02_buffers_and_strings
```

```text
1. 8바이트 칸에 이름 넣기
  8바이트 복사: "hana-kim"
  칸 밖 name.get(8): None
2. String은 길이를 알고 늘어남
  "hana-kim-long": 길이 13, 용량이 길이 이상인가: true
  "a\0b"의 길이: 3
3. 입력은 서식이 아니라 데이터
  입력: %s%s%n
```

- 2번의 용량은 정확한 값 대신 "길이 이상인가"만 출력합니다. 늘어날 때 얼마나 잡을지는 표준 라이브러리가 정하고 문서가 값을 약속하지 않기 때문입니다.

### 03_modify_while_iterating — 순회하면서 고치기

```sh
cargo run -p faq-ch9-c-memory-errors --example 03_modify_while_iterating
```

```text
1. 모았다가 나중에 더하기
  [1, 2, 3, 4]
2. 인덱스로 순회하며 더하기
  [1, 2, 3, 4]
3. 조건에 맞는 것 지우기
  [1, 3, 5]
```

- 2번은 시작할 때의 길이(`len`)까지만 돌므로, 순회 중에 넣은 4는 같은 순회에서 보지 않습니다. `w[i]`를 읽는 빌림은 그 줄에서 끝나므로 같은 반복 안에서 `push`할 수 있습니다.

### C·C++ 비교 코드

저장소 루트에서 실행합니다. `cc`·`c++`의 `-o`는 출력 디렉터리를 만들지 않으므로 먼저 만듭니다. 파일마다 머리 주석에 같은 명령이 있습니다.

```sh
mkdir -p target/tmp/faq-ch9-c-memory-errors
cc -g -fsanitize=address,undefined -o target/tmp/faq-ch9-c-memory-errors/use_after_free appd-b/ch9-c-memory-errors/compare/01_use_after_free.c
target/tmp/faq-ch9-c-memory-errors/use_after_free
```

C++ 파일(06)은 `cc` 대신 `c++ -std=c++20`으로 컴파일하고, 03은 실행할 때 `ASAN_OPTIONS=detect_stack_use_after_return=1`을 앞에 붙입니다. 아래는 이 환경에서 새니타이저가 보고한 오류 종류 줄입니다(주소와 스택 추적은 생략). 모두 종료 상태 134로 끝났습니다. 새니타이저 없이 실행한 결과는 실행마다 달라질 수 있어 싣지 않습니다.

| 파일 | 질문 | 새니타이저가 보고한 오류 | Rust에서는 |
| --- | --- | --- | --- |
| [01_use_after_free.c](compare/01_use_after_free.c) | Q9.1 | `ERROR: AddressSanitizer: heap-use-after-free` | 예제 01 |
| [02_double_free.c](compare/02_double_free.c) | Q9.2 | `ERROR: AddressSanitizer: attempting double-free` | 4.1절(E0382) |
| [03_return_stack_address.c](compare/03_return_stack_address.c) | Q9.3 | `ERROR: AddressSanitizer: stack-use-after-return` | Part 0 ch4 4.5절, Part 0 ch10 4.5절 |
| [04_heap_buffer_overflow.c](compare/04_heap_buffer_overflow.c) | Q9.4 | `ERROR: AddressSanitizer: heap-buffer-overflow`, `READ of size 4` | ch2 예제 01·03 |
| [05_null_deref.c](compare/05_null_deref.c) | Q9.6 | `runtime error: member access within null pointer of type 'struct node'`, `ERROR: AddressSanitizer: SEGV on unknown address 0x000000000000` | 4.2절(E0308) |
| [06_iterator_invalidation.cpp](compare/06_iterator_invalidation.cpp) | Q9.7 | `ERROR: AddressSanitizer: heap-use-after-free` | 예제 03 |
| [07_strcpy_overflow.c](compare/07_strcpy_overflow.c) | Q9.8 | `ERROR: AddressSanitizer: stack-buffer-overflow`, `WRITE of size 14` | 예제 02 |

macOS의 Apple clang 새니타이저는 오류를 보고하면 `abort()`하도록 설정되어 있어 종료 상태가 134입니다([ch8](../ch8-concurrency-async/README.md) 3절에서 같은 설정을 봤습니다).

## 4. 자주 하는 실수

### 4.1 같은 값을 두 번 `drop` — E0382

파일: [`compile_fail/e0382_drop_twice.rs`](compile_fail/e0382_drop_twice.rs)

```text
error[E0382]: use of moved value: `buffer`
  --> compile_fail/e0382_drop_twice.rs:17:10
   |
15 |     let buffer = vec![0u8; 64];
   |         ------ move occurs because `buffer` has type `Vec<u8>`, which does not implement the `Copy` trait
16 |     drop(buffer); // 해제
   |          ------ value moved here
17 |     drop(buffer); // E0382: 이중 해제를 시도
   |          ^^^^^^ value used here after move
```

- **원인**: C의 이중 해제(compare/02)를 그대로 옮겼습니다. `drop(buffer)`는 `buffer`의 소유권을 받아 가는 보통 함수라서, 한 번 넘긴 `buffer`는 다시 쓸 수 없습니다.
- **해결**: 두 번째 `drop`을 지웁니다. 이어지는 `help`는 `buffer.clone()`을 제안하지만, 사본을 만들어 버리는 것은 이중 해제를 고치는 길이 아닙니다. 칸을 비우면서 해제하고 싶었다면 `Option`에 담고 `take`를 씁니다(예제 01의 3번).
- 덧붙임: `drop` 뒤에 값을 쓰면 E0382라는 것과 `drop`이 정확히 한 번 일어난다는 것은 Part 0 ch14 2.6절(가드의 경우 [부록 A ch11](../../appd-a/ch11-drop-raii/README.md) 4.3절)에서 봤습니다. 이 절은 두 번째 사용이 또 하나의 `drop`, 곧 C의 두 번째 `free`(compare/02)일 때도 같은 에러로 막힌다는 것과, `help`의 `clone`이 답이 아니라는 것을 더합니다.

### 4.2 참조 자리에 널 포인터 — E0308

파일: [`compile_fail/e0308_null_into_reference.rs`](compile_fail/e0308_null_into_reference.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_null_into_reference.rs:19:23
   |
19 |     let node: &Node = std::ptr::null(); // E0308
   |               -----   ^^^^^^^^^^^^^^^^ expected `&Node`, found `*const _`
   |               |
   |               expected due to this
   |
   = note: expected reference `&Node`
            found raw pointer `*const _`
```

- **원인**: C에서 "못 찾으면 `NULL`"을 쓰던 습관으로 참조에 널 포인터를 넣으려 했습니다. `&Node`는 언제나 살아 있는 값을 가리키는 타입이고, `std::ptr::null()`은 원시 포인터(`*const _`)를 만듭니다. 둘은 다른 타입이라 바뀌지 않습니다.
- **해결**: 없을 수 있는 값은 `Option<&Node>`로 표현하고, `match`나 `if let`으로 `None`을 처리합니다(연습 3).

### 4.3 사용자 입력을 서식 문자열로 — 에러 코드 없음

파일: [`compile_fail/format_string_from_input.rs`](compile_fail/format_string_from_input.rs)

```text
error: format argument must be a string literal
  --> compile_fail/format_string_from_input.rs:16:14
   |
16 |     println!(user_input); // 서식 문자열 자리에 입력을 넣음
   |              ^^^^^^^^^^
   |
help: you might be missing a string literal to format with
   |
16 |     println!("{}", user_input); // 서식 문자열 자리에 입력을 넣음
   |              +++++
```

- **원인**: C의 `printf(user)`처럼 실행 중에 만든 문자열을 서식 자리에 넣었습니다. C에서는 입력에 든 `%s`·`%n`이 서식 지정자로 해석되어 메모리를 읽거나 쓰는 취약점이 되지만, `println!`은 서식 문자열을 컴파일 시간에 해석하므로 리터럴만 받습니다.
- **해결**: `help`대로 `println!("{}", user_input);`처럼 입력을 인자로 넘깁니다. 입력은 데이터로만 출력됩니다(예제 02의 3번). 서식 문자열이 리터럴이어야 한다는 제약의 다른 면은 [ch7](../ch7-syntax-type-surprises/README.md) Q7.8에 있습니다.

## 5. 연습 문제

### 연습 1. 칸에 다시 넣으면

01의 3번 끝에 `slot = Some(Buffer::new("새 캐시", 2));`와 `slot = Some(Buffer::new("더 새 캐시", 3));`를 차례로 더하고, 각 줄 뒤에 안내 문구를 출력해 보세요. "새 캐시"와 "더 새 캐시"가 각각 언제 해제될지 예상한 뒤 확인합니다.

<details>
<summary>힌트</summary>

값이 든 변수에 새 값을 대입하면 옛 값은 그 자리에서 해제됩니다. 그래서 "새 캐시"는 두 번째 대입 줄에서 해제되고, "더 새 캐시"는 `main`이 끝날 때 해제됩니다. C에서 포인터 변수에 새 주소를 대입하면 옛 블록이 그대로 남아 누수가 되는 것과 다른 점입니다. 대입한 값을 읽지 않으면 그런 대입마다(두 줄 모두) `unused_assignments` 경고가 납니다.

</details>

### 연습 2. compare/04를 Rust로

compare/04의 합을 구하는 반복문을 Rust로 그대로 옮겨 `fn total(scores: &[i32]) -> i32` 안에서 `for i in 0..=scores.len()`으로 `sum += scores[i];`를 해 보세요. `vec![0, 10, 20, 30]`으로 실행해 결과를 확인한 뒤, 경계를 넘지 않는 모양으로 고칩니다.

<details>
<summary>힌트</summary>

옮긴 그대로 실행하면 ``index out of bounds: the len is 4 but the index is 4``라는 패닉으로 멈추고 종료 상태는 101입니다. 옆 메모리를 읽지 않고 멈춘다는 점이 C와 다릅니다(Q9.4). `scores.iter().sum()`이나 `for s in scores { sum += s; }`처럼 반복자로 쓰면 인덱스를 직접 다루지 않으므로 한 칸 넘침이 생길 자리가 없고, 합은 60입니다.

</details>

### 연습 3. compare/05를 Rust로

compare/05의 `find`를 `fn find(nodes: &[Node], id: u32) -> Option<&Node>`로 옮겨, 찾으면 값을, 못 찾으면 "3번 노드 없음"을 출력하게 해 보세요.

<details>
<summary>힌트</summary>

`Node`에 `id`와 `value` 필드를 두고, `nodes.iter().find(|n| n.id == id)`로 찾습니다. 부르는 쪽은 `match find(&nodes, 3) { Some(n) => …, None => println!("3번 노드 없음") }`로 처리합니다. `None`을 처리하지 않고 `.value`를 읽으려 하면 `Option`에는 그런 필드가 없다는 에러가 납니다(ch5 4.1절과 같은 이유).

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rust Programming Language: [4.1절 What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)(이중 해제), [4.2절 References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)(댕글링 참조), [8.1절 Storing Lists of Values with Vectors](https://doc.rust-lang.org/book/ch08-01-vectors.html), [15장 Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- 표준 라이브러리 문서: [`Option::take`](https://doc.rust-lang.org/std/option/enum.Option.html#method.take), [`slice::copy_from_slice`](https://doc.rust-lang.org/std/primitive.slice.html#method.copy_from_slice), [`std::ptr::null`](https://doc.rust-lang.org/std/ptr/fn.null.html), [`std::mem::MaybeUninit`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html)

C·C++과 도구

- Chromium: [Memory safety](https://www.chromium.org/Home/chromium-security/memory-safety/)(날짜 표기 없음)
- C++ 표준 초안: [\[vector.modifiers\]](https://eel.is/c++draft/vector.modifiers)
- Clang: [AddressSanitizer](https://clang.llvm.org/docs/AddressSanitizer.html), [MemorySanitizer](https://clang.llvm.org/docs/MemorySanitizer.html), [UndefinedBehaviorSanitizer](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html)
