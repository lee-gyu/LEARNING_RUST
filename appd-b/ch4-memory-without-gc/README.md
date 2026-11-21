# ch4. GC 없는 메모리 관리

Java, C#, Go, Python, JavaScript처럼 가비지 컬렉터(garbage collector, GC)가 있는 언어를 써 온 사람에게 Rust의 소유권(ownership)은 가장 낯선 규칙입니다. 변수를 넘겼더니 원래 변수를 쓸 수 없고, 같은 값을 두 곳에서 고치려 하면 컴파일러가 막고, 구조체에 참조를 넣으려 하면 라이프타임(lifetime)을 적으라고 합니다. 이 챕터는 소유권 규칙을 다시 가르치지 않습니다(그것은 Part 0 ch4의 일입니다). 대신 GC 언어의 습관으로 Rust를 쓸 때 부딪히는 질문에 "왜 그렇게 만들었고, 내가 쓰던 언어에서는 어땠는가"로 답합니다.

- 패키지: `faq-ch4-memory-without-gc`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q4.1](#q41-gc도-없고-free도-부르지-않는데-메모리는-누가-언제-해제하나요) | GC도 없고 `free`도 부르지 않는데 메모리는 누가 언제 해제하나요? | 값의 소유자가 스코프를 벗어날 때 컴파일러가 넣어 둔 해제 코드가 실행됩니다. |
| [Q4.2](#q42-변수를-다른-변수에-넣거나-함수에-넘겼더니-왜-더-쓸-수-없나요) | 변수를 다른 변수에 넣거나 함수에 넘겼더니 왜 더 쓸 수 없나요? | 대입과 인자 전달이 소유권을 옮기기 때문입니다. 소유자가 하나여야 해제도 한 번만 일어납니다. |
| [Q4.3](#q43-어떤-타입은-넘긴-뒤에도-쓸-수-있는데-기준이-무엇인가요) | 어떤 타입은 넘긴 뒤에도 쓸 수 있는데 기준이 무엇인가요? | `Copy`를 구현한 타입입니다. 힙 자원을 소유하지 않아 비트 복사만으로 충분한 타입만 `Copy`가 될 수 있습니다. |
| [Q4.4](#q44-참조-t는-c의-포인터-java의-참조와-무엇이-다른가요) | 참조 `&T`는 C의 포인터, Java의 참조와 무엇이 다른가요? | 언제나 살아 있는 값을 가리키고 널이 될 수 없으며, 값을 소유하지 않아 해제와 상관없습니다. |
| [Q4.5](#q45-왜-가변-참조는-한-번에-하나만-만들-수-있나요) | 왜 가변 참조는 한 번에 하나만 만들 수 있나요? | 고치는 동안 다른 쪽이 옛 상태를 보거나 해제된 메모리를 읽는 일을 컴파일 시간에 막기 위해서입니다. |
| [Q4.6](#q46-clone을-많이-쓰면-나쁜-코드인가요) | `clone()`을 많이 쓰면 나쁜 코드인가요? | 꼭 그렇지 않습니다. 비용이 코드에 드러날 뿐입니다. 큰 데이터를 자주 복제하면 공유나 빌림을 검토합니다. |
| [Q4.7](#q47-rc와-arc는-gc가-아닌가요-순환-참조는-어떻게-되나요) | `Rc`와 `Arc`는 GC가 아닌가요? 순환 참조(reference cycle)는 어떻게 되나요? | 참조 카운팅이라는 좁은 의미의 자동 해제입니다. 추적 GC와 달리 순환 참조는 거두지 못하므로 `Weak`로 끊습니다. |
| [Q4.8](#q48-라이프타임-a는-무엇이고-왜-직접-써야-하나요) | 라이프타임 `'a`는 무엇이고 왜 직접 써야 하나요? | 참조가 어떤 값보다 오래 살 수 없는지를 적는 표시입니다. 컴파일러가 짐작할 수 없는 경우에만 적습니다. |
| [Q4.9](#q49-gc가-없어서-얻는-것과-잃는-것은-무엇인가요) | GC가 없어서 얻는 것과 잃는 것은 무엇인가요? | GC 없이 얻는 시간적 안전성과 정해진 해제 시점을 얻고, 코드가 안전함을 미리 보여야 하는 수고와 순환 구조의 불편을 치릅니다. |
| [Q4.10](#q410-값을-스택에-둘지-힙에-둘지-직접-골라야-하나요) | 값을 스택에 둘지 힙에 둘지 직접 골라야 하나요? | 기본은 그 자리(스택이나 바깥 값의 안)이고, 힙은 `Box`·`Vec`·`String`처럼 그렇다고 적을 때만 씁니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md): 소유권 규칙, 이동·복사·복제, 참조와 대여 규칙, 댕글링 참조
- [Part 0 ch8](../../part0-getting-started/ch8-collections/README.md) 2.6절: 요소의 참조를 쥔 채 `push`할 수 없는 이유
- [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.12~2.15절: 라이프타임 표기, 구조체의 라이프타임, 생략 규칙. [Part 0 ch5](../../part0-getting-started/ch5-structs/README.md) 4.5절: 참조 필드에 라이프타임이 없을 때의 E0106
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md): `Box`, `Drop`, `Rc`, `RefCell`, 순환 참조와 `Weak`
- [ch1](../ch1-why-rust/README.md) Q1.1(GC 없이 정해진 때 해제), [ch3](../ch3-safety-limits/README.md) Q3.4(누수는 안전하다)

로드맵의 예제 계획(drop 시점, 이동과 복사, 빌려주기, `Rc`의 참조 수와 `Weak`, `clone` 비용)은 위 챕터들의 예제에 이미 있으므로 본문에서 인용하고, 이 챕터의 예제는 GC 언어와 비교하는 새 시연 셋으로 바꿨습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. GC 언어에서 "같은 객체를 두 변수가 가리키는" 코드를 Rust의 이동·빌림·복제·`Rc<RefCell<T>>` 가운데 하나로 옮기고, 각각의 비용과 의미의 차이를 설명할 수 있다.
2. 가변 참조가 하나뿐이어야 하는 이유를 해제 후 사용과 연결해 설명하고, 같은 값을 동시에 `&mut`와 `&`로 넘기는 코드를 고칠 수 있다.
3. Rust의 값이 기본으로 그 자리에 놓인다는 것을 크기와 메모리 배치(layout)로 확인하고, Java·C#·Go의 객체 배치와 비교할 수 있다.
4. `Rc`/`Arc`와 추적 GC의 차이, 라이프타임 표기가 필요한 경우, GC가 없어서 얻고 잃는 것을 설명할 수 있다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다.

### Q4.1 GC도 없고 `free`도 부르지 않는데 메모리는 누가 언제 해제하나요?

**짧은 답** 값마다 소유자(owner)가 하나 있고, 소유자가 스코프를 벗어나면 그 값이 버려지면서(drop) 메모리가 해제됩니다. 해제 코드는 컴파일러가 그 자리에 넣어 둡니다. 그래서 프로그래머가 `free`를 부르지도, 실행 중에 GC가 돌지도 않습니다. The Book은 소유권이 Rust가 가비지 컬렉터 없이 메모리 안전성을 보장하게 해 준다고 설명합니다.

**왜 그런가** 해제 시점을 정하는 규칙이 컴파일러가 확인할 수 있을 만큼 단순하기 때문입니다. 소유자는 하나이고(Q4.2), 소유자의 스코프가 끝나는 곳은 코드에 보입니다. 해제가 실제로 그 자리에서 일어나는 것은 [ch1](../ch1-why-rust/README.md) 예제 01(살아 있는 버퍼 수가 최대 1개)과 Part 0 ch4 예제 01, ch14 예제 04에서 출력으로 확인했습니다.

**다른 언어에서는** Java·C#·Go·JavaScript의 추적(tracing) GC는 실행 중에 닿을 수 없는 객체를 찾아 해제하므로, 해제 시점은 GC가 정합니다. Python의 기본 구현은 참조 카운팅으로 해제하고 GC로 순환 참조를 거둡니다(ch1 Q1.1). C는 `free`, C++는 `delete`나 RAII로 해제하며, Rust의 방식은 C++의 RAII를 언어 전체의 기본 규칙으로 삼고 컴파일러가 검사하게 만든 것에 가깝습니다(ch11 `ch11-cpp-raii-move`).

**오해와 한계** 메모리 말고도 파일·소켓·잠금이 같은 규칙으로 정리됩니다(ch5 `ch5-no-null-no-exceptions` Q5.6). 반대로 `mem::forget`이나 `Rc` 순환 참조처럼 소멸자가 실행되지 않는 길도 있습니다([ch3](../ch3-safety-limits/README.md) Q3.4).

**더 읽기** Part 0 ch4 2.1·2.4절 / ch1 Q1.1 / Q4.7

### Q4.2 변수를 다른 변수에 넣거나 함수에 넘겼더니 왜 더 쓸 수 없나요?

**짧은 답** 힙 자원을 소유한 값(`String`, `Vec`, 그런 값을 담은 구조체)은 대입하거나 함수에 넘기면 소유권이 옮겨 갑니다(move). 소유자가 둘이면 둘 다 스코프를 벗어날 때 같은 메모리를 두 번 해제하는 이중 해제(double free)가 되므로, 옮긴 뒤의 원래 변수는 쓸 수 없게 합니다. 같은 값을 여러 곳에서 쓰고 싶으면 빌리거나(대여, borrow: `&`), 복제하거나(`clone`), 함께 소유하는(`Rc`) 방법을 고릅니다.

**왜 그런가** GC 언어의 `b = a`는 "같은 객체를 가리키는 이름을 하나 더 만든다"는 한 가지 뜻이지만, Rust에는 그 한 가지 동작이 없습니다. 하려는 일에 따라 다른 도구를 씁니다. [01_java_style_sharing](examples/01_java_style_sharing.rs)이 같은 계좌를 네 가지 방법으로 다룹니다.

```text
1. 이동: let b = a;
  b: 하준 100원
2. 빌려 주기: deposit(&mut account, 50)
  account: 서아 150원
3. 복제: let mut copy = original.clone();
  original 100원, copy 150원
4. Rc<RefCell<Account>>: 함께 소유하고 함께 고치기
  shared로 읽은 잔액 150원, 소유자 수 2
```

| 하려는 일 | Rust의 도구 | 비용 |
| --- | --- | --- |
| 값을 다른 곳에 넘기고 더는 안 씀 | 이동 | 스택의 몇 바이트 복사(힙은 그대로) |
| 잠깐 읽거나 고치게 함 | `&T`, `&mut T` | 없음(주소 하나. 슬라이스와 `&str`은 길이까지 두 칸) |
| 서로 독립된 사본이 필요 | `clone()` | 힙 데이터까지 새로 할당하고 복사 |
| 여러 곳이 함께 소유하고 함께 고침 | `Rc<RefCell<T>>`(스레드 사이는 `Arc<Mutex<T>>`) | 참조 수 세기와 실행 중 대여 검사 |

**다른 언어에서는** Java·C#·Python·JavaScript에서 객체를 담은 변수는 참조이고, 대입은 참조를 복사합니다. 위 표의 네 번째 줄이 그 언어들의 기본 동작이고, Rust에서는 그것을 특별히 고를 때만 씁니다. C++는 `auto b = a;`(초기화)면 복사 생성자, `b = a;`(대입)면 복사 대입 연산자가 돌아 기본으로 사본을 만들고, `std::move`로 이동을 요청합니다(ch11 `ch11-cpp-raii-move`).

**오해와 한계** GC 언어의 습관대로 모든 곳에 `Rc<RefCell<T>>`를 쓰면 컴파일은 되지만, 대여 규칙 위반이 실행 중 패닉으로 미뤄지고 비용도 듭니다. 데이터의 주인을 먼저 정하고, 나머지는 빌리는 설계가 기본입니다([부록 A ch15](../../appd-a/ch15-borrow-checker/README.md)). 불변 참조로 받은 구조체에서 필드를 그대로 꺼내 오는 Java식 코드가 E0507로 거부되는 것은 4.1절에서 봅니다.

**더 읽기** Part 0 ch4 2.5~2.7절 / Q4.6, Q4.7

### Q4.3 어떤 타입은 넘긴 뒤에도 쓸 수 있는데 기준이 무엇인가요?

**짧은 답** `Copy` 트레이트를 구현한 타입입니다. 정수, 실수, `bool`, `char`, 불변 참조 `&T`, 그리고 그런 것만 담은 튜플·배열·구조체가 `Copy`가 될 수 있습니다. 대입하면 비트를 그대로 복사하므로 원래 변수도 계속 쓸 수 있습니다. 직접 만든 구조체는 필드가 모두 `Copy`여도 `#[derive(Clone, Copy)]`를 붙여야만 `Copy`가 되고, 붙이지 않으면 이동합니다. 힙 자원을 소유하거나 `Drop`을 구현한 타입은 `Copy`가 될 수 없습니다.

**왜 그런가** 비트 복사만으로 독립된 사본이 되는 타입만 `Copy`가 될 수 있습니다. `String`을 비트 복사하면 같은 힙 버퍼를 가리키는 값이 둘 생기고, 둘 다 해제하려 하므로 이중 해제가 됩니다(Part 0 ch4 2.6절). 그래서 직접 만든 구조체도 필드가 모두 `Copy`일 때만 `#[derive(Clone, Copy)]`를 붙일 수 있고, `String` 필드가 하나라도 있으면 컴파일러가 거부합니다(E0204). `Drop`을 구현한 타입에 `Copy`를 붙이는 것도 거부됩니다(E0184). 두 경우 모두 [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)의 `compile_fail/e0204_copy_with_string_field.rs`, `e0184_copy_with_drop.rs`에 있습니다. 예제 03의 `Point`(`f64` 둘)는 `Copy`라서 `Vec`에서 꺼내 쓸 때 복사됩니다.

**다른 언어에서는** C#의 값 형식(struct)은 대입할 때 복사된다는 점에서 `Copy` 타입과 비슷합니다. 다만 C# 문서가 밝히듯 값 형식 안에 참조 형식의 필드가 있으면 그 참조만 복사되어, 복사본과 원본이 같은 객체에 닿습니다. Rust는 힙 자원을 소유한 필드가 있으면 `Copy` 자체를 허용하지 않고, 복제하려면 `clone()`을 명시하게 합니다. 또 C#의 struct는 선언만으로 복사되지만, Rust의 구조체는 `Copy`를 직접 붙이지 않으면 필드가 모두 `f64`여도 대입할 때 이동합니다.

**오해와 한계** `Copy`가 아닌 타입도 `clone()`으로 명시적으로 복제할 수 있습니다. 반대로 `Copy`를 붙이면 큰 배열 필드도 대입할 때마다 코드에 보이지 않는 복사가 일어나고, 공개 타입에 한 번 붙인 `Copy`는 나중에 떼기 어렵습니다([부록 A ch10](../../appd-a/ch10-standard-traits/README.md)).

**더 읽기** Part 0 ch4 2.6절 / Part 1 ch6

### Q4.4 참조 `&T`는 C의 포인터, Java의 참조와 무엇이 다른가요?

**짧은 답** Rust의 참조(reference)는 언제나 살아 있는 값을 가리키고, 널이 될 수 없으며, 값을 소유하지 않습니다. C 포인터와 달리 댕글링이나 널이 될 수 없고, Java 참조와 달리 가리키는 값을 살려 두지 않습니다. 그래서 참조가 값보다 오래 살지 못하게 컴파일러가 검사합니다.

**왜 그런가** The Book은 참조가 포인터와 달리 그 참조가 사는 동안 특정 타입의 올바른 값을 가리킨다고 보장한다고 설명합니다.

| | C 포인터 | Java 참조 | Rust `&T` |
| --- | --- | --- | --- |
| 널이 될 수 있나 | 예 | 예(Java 언어 명세: 참조 값에는 아무 객체도 가리키지 않는 null 참조가 있음) | 아니요(없을 수 있으면 `Option<&T>`) |
| 가리키는 값을 살려 두나 | 아니요(댕글링 가능) | 예(닿을 수 있는 객체는 GC가 해제하지 않음) | 아니요. 대신 값보다 오래 살 수 없음을 컴파일러가 검사 |
| 고칠 수 있나 | `const`가 없으면 | 객체가 허용하면 언제나 | 가변 참조(mutable reference) `&mut T`일 때만, 한 번에 하나 |
| 산술 | 가능 | 불가 | 참조로는 불가. 원시 포인터로는 가능(`wrapping_add`는 안전, `add`·`offset`과 역참조는 `unsafe`) |

두 번째 줄의 차이가 가장 큽니다. Java에서 메서드가 받은 참조를 필드에 저장해 두면 그 객체는 계속 살아 있습니다. Rust의 참조는 값을 살려 두지 못하므로, 값을 오래 붙잡아 두려면 소유하거나(필드 타입을 `String`으로) `Rc`로 함께 소유해야 합니다(Q4.8).

**오해와 한계** 참조는 값을 소유하지 않으므로, 참조를 통해 값의 일부를 가져갈 수도 없습니다. 불변 참조 `&Account`에서 `String` 필드를 꺼내 오면 E0507이 납니다(4.1절). 읽기만 하려면 `&account.owner`처럼 빌리고, 따로 가져야 하면 복제합니다.

**더 읽기** Part 0 ch4 2.8·2.11절 / ch2 Q2.5

### Q4.5 왜 가변 참조는 한 번에 하나만 만들 수 있나요?

**짧은 답** 값을 고치는 동안 다른 쪽이 그 값을 보고 있으면, 반쯤 바뀐 상태를 보거나 고치는 쪽이 옮기고 해제한 메모리를 읽게 됩니다. "고치는 쪽이 있으면 다른 참조는 없다"는 규칙 하나로 이런 해제 후 사용(use after free)과 데이터 경쟁(data race)을 컴파일 시간에 막습니다. The Book도 이 제한 덕분에 데이터 경쟁을 컴파일 시간에 막을 수 있다고 설명합니다.

**왜 그런가** `Vec`에 원소를 넣다가 용량이 모자라면 더 큰 버퍼로 원소를 옮기고 옛 버퍼를 해제할 수 있습니다(Part 0 ch8 2.6절, 예제 03). 할당기가 제자리에서 늘려 주면 옮기지 않기도 하지만, 어느 쪽일지는 실행 중에야 정해지므로 컴파일러는 옮길 수 있다고 보고 검사합니다. [02_exclusive_mut](examples/02_exclusive_mut.rs)는 이 일이 함수 인자에서 일어나는 경우를 봅니다.

```text
2. 같은 벡터를 자기 뒤에 덧붙이기
  extend_from_within: [1, 2, 3, 1, 2, 3]
  사본을 덧붙이기: [1, 2, 3, 1, 2, 3]
```

`append_copy(dst: &mut Vec<i32>, src: &[i32])`에 같은 벡터를 `append_copy(&mut v, &v)`로 넘기면, 함수 안의 `push`가 버퍼를 옮기는 순간 `src`가 해제된 옛 버퍼를 가리키게 됩니다. 컴파일러는 이 호출을 E0502로 거부합니다(4.2절). 같은 벡터를 자기 뒤에 덧붙이려면 그 일을 안전하게 해 주는 `extend_from_within`을 쓰거나 사본을 먼저 만듭니다. 이 규칙 덕분에 `&mut` 매개변수를 받은 함수는 "이 함수가 도는 동안 아무도 이 값을 보지 않는다"고 믿을 수 있습니다.

**다른 언어에서는** GC 언어에서는 같은 객체를 여러 참조가 언제든 고칠 수 있고, 고치는 도중의 상태를 다른 쪽이 보는 문제는 프로그래머가 피해야 합니다. Java의 `ArrayList`는 반복자를 만든 뒤 리스트의 구조가 바뀌면 그 반복자가 `ConcurrentModificationException`을 던지는(fail-fast) 방식으로 일부를 실행 중에 잡습니다(Java API 문서). C++에서는 같은 상황이 미정의 동작(undefined behavior)이 됩니다(ch9 `ch9-c-memory-errors`의 반복자 무효화).

**오해와 한계** 이 규칙은 보수적이라, 서로 다른 필드나 슬라이스의 겹치지 않는 부분을 동시에 고치는 것처럼 실제로는 안전한 코드도 거부할 때가 있습니다. `split_at_mut`처럼 겹치지 않음을 보증하는 API나 필드를 나눠 빌리는 방법으로 풉니다([부록 A ch15](../../appd-a/ch15-borrow-checker/README.md), ch15 `ch15-pain-points`).

**더 읽기** Part 0 ch4 2.9절, 4.2절(가변 참조 둘, E0499), ch8 2.6절 / ch8 `ch8-concurrency-async`(스레드 사이의 같은 규칙)

### Q4.6 `clone()`을 많이 쓰면 나쁜 코드인가요?

**짧은 답** 꼭 그렇지 않습니다. `clone()`은 GC 언어에서 보이지 않던 복사 비용을 코드에 드러낼 뿐입니다. 작은 값이나 드물게 복제하는 값이라면 `clone()`이 가장 단순한 답일 때가 많습니다. 큰 데이터를 반복문 안에서 자꾸 복제한다면 빌리거나(`&T`) 함께 소유하는(`Rc`) 쪽을 검토합니다.

**왜 그런가** `String`의 `clone()`은 힙에 새 버퍼를 할당하고 글자를 복사하지만(Part 0 ch4 예제 02의 `as_ptr` 비교), `Rc::clone()`은 참조 수만 하나 늘립니다(Part 0 ch14 예제 05). 무엇이 비싼지가 타입으로 보이므로, 측정한 뒤 바꿔도 늦지 않습니다. 부록 A ch20은 라이프타임이 번지는 구조체를 소유 타입으로 바꾸고, 복사를 지우기 전에 세어 보라고 권합니다.

**다른 언어에서는** Java·Python에서는 대입이 참조 복사라서 복제를 거의 의식하지 않고, 사본이 필요하면 프로그래머가 따로 만듭니다. Rust는 공유가 기본이 아니라서 "사본이 필요한가, 공유가 필요한가"를 처음부터 고르게 합니다.

**오해와 한계** 대여 검사기(borrow checker)의 에러를 없애려고 아무 생각 없이 `clone()`을 넣으면, 원래 같은 값을 고쳐야 하는 코드가 사본을 고치는 버그가 됩니다(예제 01의 3번처럼 원본은 그대로 남습니다). 복제가 의미상 맞는지 먼저 봅니다.

**더 읽기** [부록 A ch20](../../appd-a/ch20-avoid-over-optimizing/README.md) / ch14 `ch14-performance`

### Q4.7 `Rc`와 `Arc`는 GC가 아닌가요? 순환 참조는 어떻게 되나요?

**짧은 답** 참조 카운팅(reference counting)이라는 좁은 의미의 자동 해제입니다. 마지막 소유자가 사라지는 순간 그 자리에서 해제되고, 따로 도는 수거기는 없습니다. 추적 GC와 달리 서로를 가리키는 순환 참조는 거두지 못하므로, 한쪽을 `Weak`로 바꿔 순환을 끊습니다. `Arc`는 참조 수를 원자적으로 세어 스레드 사이에서 쓸 수 있게 한 판입니다.

**왜 그런가** 참조 수는 `Rc::clone`과 버려질 때만 바뀌므로 해제 시점이 여전히 정해져 있습니다. 대신 A가 B를, B가 A를 붙잡으면 둘의 참조 수가 0이 되지 않아 영원히 남습니다. The Book은 이런 누수가 메모리 안전성 위반은 아니라고 설명하고([ch3](../ch3-safety-limits/README.md) Q3.4, 예제 01의 3번), Part 0 ch14 2.11절은 부모를 가리키는 쪽을 `Weak`로 두는 방법을 보입니다.

**다른 언어에서는** Python의 기본 구현은 Rust의 `Rc`처럼 참조 카운팅으로 객체를 해제하고, 그 위에 순환 참조를 거두는 GC를 따로 둡니다(Python `gc` 모듈 문서). Rust에는 그 순환 수거기가 없다는 것이 차이입니다. Java·C#·Go의 추적 GC는 순환이 있어도 닿을 수 없으면 거둡니다.

**오해와 한계** `Rc`를 쓰면 "GC 언어처럼 편하게" 쓸 수 있을 것 같지만, 참조 수 세기 비용이 있고 순환을 프로그래머가 끊어야 하며 고치려면 `RefCell`이 더 필요합니다. 그래프처럼 순환이 본질인 자료구조는 인덱스로 가리키는 방법이 더 나을 때가 많습니다(ch15 `ch15-pain-points`, [부록 A ch15](../../appd-a/ch15-borrow-checker/README.md)).

**더 읽기** Part 0 ch14 2.7·2.10·2.11절 / ch3 Q3.4

### Q4.8 라이프타임 `'a`는 무엇이고 왜 직접 써야 하나요?

**짧은 답** 라이프타임 표기(lifetime annotation)는 "이 참조는 저 값보다 오래 살 수 없다"는 관계를 적는 표시입니다. 참조가 값을 살려 두지 못하는(Q4.4) Rust에서, 컴파일러는 모든 참조가 가리키는 값보다 먼저 끝나는지 확인해야 합니다. 대부분은 생략 규칙(elision rules)으로 짐작하므로 적을 필요가 없고, 함수가 참조를 여러 개 받아 하나를 돌려주거나 구조체가 참조를 담을 때처럼 짐작할 수 없는 경우에만 적습니다.

**왜 그런가** 함수 시그니처만 보고도 호출하는 쪽을 검사할 수 있게 하려는 것입니다. `fn longest(a: &str, b: &str) -> &str`의 반환값이 `a`에서 왔는지 `b`에서 왔는지 시그니처에 없으면, 호출하는 쪽은 결과를 언제까지 써도 되는지 알 수 없습니다(Part 0 ch10의 E0106). 구조체도 마찬가지로, `struct Parser { input: &str }`는 `Parser`가 `input`이 가리키는 문자열보다 오래 살 수 없다는 관계를 `Parser<'a>`로 적어야 합니다(Part 0 ch5 4.5절의 E0106).

**다른 언어에서는** GC 언어에는 이 개념이 필요 없습니다. 참조가 객체를 살려 두므로 "객체가 참조보다 먼저 사라지는" 일이 없기 때문입니다. C·C++에서는 같은 관계가 문서나 프로그래머의 머릿속에만 있고, 어기면 댕글링 포인터가 됩니다.

**오해와 한계** GC 언어에서 온 사람은 구조체에 참조를 넣고 라이프타임을 붙이는 쪽으로 가기 쉽지만, 오래 쓰는 구조체는 데이터를 소유하게 하는 편(필드를 `String`, `Vec<T>`로)이 대개 더 단순합니다. 라이프타임은 짧게 쓰고 버리는 뷰(view) 타입에 어울립니다([부록 A ch14](../../appd-a/ch14-lifetimes/README.md), [ch20](../../appd-a/ch20-avoid-over-optimizing/README.md)).

**더 읽기** Part 0 ch10, ch5 4.5절 / 부록 A ch14

### Q4.9 GC가 없어서 얻는 것과 잃는 것은 무엇인가요?

**짧은 답** 얻는 것은 GC 같은 실행 중 장치 없이 얻는 시간적 안전성(해제 후 사용·이중 해제 방지), 정해진 해제 시점, GC 멈춤이 없는 고른 지연 시간, 메모리 사용량의 예측 가능성입니다. 경계 검사 같은 실행 중 검사는 남습니다(ch2 Q2.4). 잃는 것은 코드가 안전함을 미리 보여야 하는 수고, 순환하는 자료구조를 만들 때의 불편, 컴파일러가 거부하는 안전한 코드를 고쳐 쓰는 수고입니다.

**왜 그런가**

| 얻는 것 | 근거 |
| --- | --- |
| 실행 중 장치 없는 시간적 안전성 | Google 보고서는 Rust를 GC나 모든 곳에 적용하는 참조 카운팅 같은 실행 중 장치 없이 넓은 범위의 코드에서 시간적 안전성을 주는 유일한 성숙한 언어로 꼽습니다(ch2 Q2.5) |
| 정해진 해제 시점 | ch1 예제 01: 살아 있는 버퍼가 언제나 최대 1개 |
| GC 멈춤 없음 | Discord는 Go로 쓴 서비스의 주기적인 지연 시간 급증을 Go의 메모리 모델과 GC로 진단하고 Rust로 다시 썼습니다(ch1 Q1.3) |
| 메모리 배치를 고를 수 있음 | 예제 03: 값이 그 자리에 놓이고 힙은 고를 때만 씀(Q4.10) |

| 잃는 것 | 근거 |
| --- | --- |
| 안전함을 미리 보여야 하는 수고 | Google 보고서는 라이프타임과 대여 검사기가 넓은 범위의 코드에서 실행 비용 없이 컴파일 시간에 안전을 보장하는 대신, 코드가 안전함을 보이려면 프로그래머가 미리 더 많은 노력을 들여야 한다고 씁니다. 정적 타입이 동적 타입보다 미리 드는 노력이 많은 것과 비슷하다는 비유도 덧붙입니다 |
| 순환하는 자료구조의 불편 | 같은 Google 보고서는 대여 검사기의 제약이 순환 참조 그래프 같은 구조를 막고, 표준 라이브러리가 참조 카운팅으로 실행 중 비용을 치르며 이를 허용한다고 씁니다 |
| 보수적인 검사 | 안전한 코드도 규칙으로 증명할 수 없으면 거부(Q4.5, ch15 `ch15-pain-points`) |

**오해와 한계** GC가 나쁘다는 뜻이 아닙니다. 지연 시간과 메모리 사용이 중요하지 않은 프로그램에서는 GC 언어가 더 생산적인 선택일 수 있습니다(ch1 Q1.6). 또 해제 시점이 정해진 대신, 큰 자료구조가 버려지는 자리에서는 그 안의 값을 해제하는 비용이 한꺼번에 듭니다.

**더 읽기** ch1 Q1.1, Q1.6 / ch2 Q2.5 / ch14 `ch14-performance`

### Q4.10 값을 스택에 둘지 힙에 둘지 직접 골라야 하나요?

**짧은 답** 기본은 값이 놓인 그 자리입니다. 지역 변수는 스택에, 구조체의 필드나 `Vec`의 원소는 바깥 값의 안에 그대로 담깁니다. 힙은 `Box`, `Vec`, `String`, `Rc`처럼 힙을 쓰는 타입을 고를 때만 씁니다. 크기가 실행 중에 정해지거나, 재귀 타입이거나, 트레이트 객체이거나, 아주 큰 값이라 옮길 때 복사를 줄이고 싶을 때 힙을 고릅니다.

**왜 그런가** The Book은 스택에 둘 데이터는 컴파일할 때 크기가 정해져 있어야 하고, 크기를 모르거나 바뀔 수 있는 데이터는 힙에 둔다고 설명합니다. [03_values_inline](examples/03_values_inline.rs)은 이 기본이 메모리 배치에 어떻게 나타나는지 봅니다.

```text
1. 구조체 안의 구조체는 그 자리에 담긴다
  Point 16바이트, Segment 32바이트
2. Vec<Point>: 원소가 힙에 나란히 붙어 있다
  이웃한 원소의 간격 16바이트
3. Vec<Box<Point>>: 원소마다 따로 할당한 값을 가리킨다
  원소 하나의 크기 8바이트, 이웃한 원소의 간격 8바이트
```

`Segment`는 `Point` 둘을 가리키는 주소가 아니라 `Point` 둘 자체이고, `Vec<Point>`는 버퍼 하나에 `Point`가 16바이트씩 이어져 있습니다. `Vec<Box<Point>>`는 버퍼에 주소만 이어져 있고 `Point`는 각자 따로 할당됩니다.

**다른 언어에서는**

- Java 언어 명세는 참조 값이 객체를 가리키는 포인터라고 정합니다. 객체를 담은 배열이나 `ArrayList`의 원소는 참조이므로, 위의 `Vec<Box<Point>>`에 가까운 배치가 됩니다.
- C#은 값 형식(struct)과 참조 형식(class)을 나눕니다. 값 형식의 변수는 인스턴스를 직접 담으므로 struct 배열은 `Vec<Point>`에, class 배열은 `Vec<Box<Point>>`에 가깝습니다.
- Go FAQ는 변수가 스택에 있는지 힙에 있는지는 언어의 의미와 상관없고, 컴파일러가 함수가 끝난 뒤 참조되지 않음을 증명하지 못하면 GC 힙에 둔다고 설명합니다. Rust는 이것을 컴파일러가 알아서 정하지 않고 타입으로 고르게 합니다.

**오해와 한계** "스택이 빠르니 힙을 피하라"는 규칙은 아닙니다. 아주 큰 배열을 스택에 두면 스택이 넘칠 수 있고(디버그 빌드에서는 `Box::new`에 넘기는 큰 배열도 먼저 스택에 만들어져 넘칠 수 있다는 것을 [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.7절이 보입니다), 크기가 큰 값을 자주 옮기면 그만큼 복사됩니다. 대부분의 코드는 `Vec`과 `String`을 쓰고, 특별한 이유가 있을 때만 `Box`를 고르면 충분합니다(Part 0 ch14 2.2절).

**더 읽기** Part 0 ch14 2.2절 / [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md)(크기와 정렬)

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_java_style_sharing](examples/01_java_style_sharing.rs) | 같은 계좌를 이동·빌려 주기·복제·`Rc<RefCell<T>>` 네 가지로 다루고 결과 비교 | Q4.2, Q4.4, Q4.7 |
| [02_exclusive_mut](examples/02_exclusive_mut.rs) | 다른 벡터를 덧붙이기와, 같은 벡터를 자기 뒤에 덧붙이는 안전한 방법 두 가지 | Q4.5 |
| [03_values_inline](examples/03_values_inline.rs) | 구조체 안의 구조체, `Vec<Point>`와 `Vec<Box<Point>>`의 크기와 원소 간격 | Q4.10, Q4.9 |

### 01_java_style_sharing — 네 가지 방법

```sh
cargo run -p faq-ch4-memory-without-gc --example 01_java_style_sharing
```

```text
1. 이동: let b = a;
  b: 하준 100원
2. 빌려 주기: deposit(&mut account, 50)
  account: 서아 150원
3. 복제: let mut copy = original.clone();
  original 100원, copy 150원
4. Rc<RefCell<Account>>: 함께 소유하고 함께 고치기
  shared로 읽은 잔액 150원, 소유자 수 2
```

- 1번에서 `let b = a;` 뒤에 `a`를 쓰면 E0382입니다(Part 0 ch4 4.1절). 이 예제는 그 에러를 다시 보이지 않고 쓰지 않는 것으로 둡니다.
- 3번은 사본을 고쳤으므로 원본은 100원 그대로입니다. 4번은 `other`로 고친 것이 `shared`로도 보입니다. Java의 대입과 같은 결과는 4번뿐입니다.

### 02_exclusive_mut — 같은 벡터를 동시에 고치고 읽기

```sh
cargo run -p faq-ch4-memory-without-gc --example 02_exclusive_mut
```

```text
1. 다른 벡터를 덧붙이기
  [1, 2, 3, 4, 5]
2. 같은 벡터를 자기 뒤에 덧붙이기
  extend_from_within: [1, 2, 3, 1, 2, 3]
  사본을 덧붙이기: [1, 2, 3, 1, 2, 3]
```

- `push`가 용량을 늘리며 원소를 옮길 수 있다는 것은 Part 0 ch8 예제 03의 2번에서 봤으므로 다시 보이지 않습니다. 이 예제가 더하는 것은 같은 문제가 함수 인자(`dst`와 `src`)에서 생길 때와 그 해결입니다.
- `extend_from_within(..)`은 주어진 범위의 원소를 복제해 벡터 끝에 붙입니다. 범위 `..`는 전체입니다.

### 03_values_inline — 값은 그 자리에

```sh
cargo run -p faq-ch4-memory-without-gc --example 03_values_inline
```

```text
1. 구조체 안의 구조체는 그 자리에 담긴다
  Point 16바이트, Segment 32바이트
2. Vec<Point>: 원소가 힙에 나란히 붙어 있다
  이웃한 원소의 간격 16바이트
3. Vec<Box<Point>>: 원소마다 따로 할당한 값을 가리킨다
  원소 하나의 크기 8바이트, 이웃한 원소의 간격 8바이트
  (0, 0)에서 (3, 4)까지의 길이 5
```

- 크기와 간격은 64비트 플랫폼 기준입니다. 주소 값 자체는 실행마다 다르므로, 두 원소 주소의 차이만 출력합니다. `(&items[1] as *const T).addr()`은 참조를 원시 포인터로 바꿔 주소를 정수로 얻는 것으로, 역참조하지 않으므로 안전한 코드입니다.
- 마지막 줄은 `Vec`과 `Box` 안의 `Point`를 복사해 와 `Segment`를 만든 결과입니다. `Point`가 `Copy`라서 원래 `Vec`의 원소는 그대로 남습니다(Q4.3).

## 4. 자주 하는 실수

### 4.1 참조로 받은 구조체에서 필드를 꺼내 옴 — E0507

파일: [`compile_fail/e0507_move_field_out_of_reference.rs`](compile_fail/e0507_move_field_out_of_reference.rs)

```text
error[E0507]: cannot move out of `account.owner` which is behind a shared reference
  --> compile_fail/e0507_move_field_out_of_reference.rs:21:16
   |
21 |     let name = account.owner; // E0507
   |                ^^^^^^^^^^^^^ move occurs because `account.owner` has type `String`, which does not implement the `Copy` trait
   |
help: consider borrowing here
   |
21 |     let name = &account.owner; // E0507
   |                +
help: consider cloning the value if the performance cost is acceptable
   |
21 |     let name = account.owner.clone(); // E0507
   |                             ++++++++
```

- **원인**: 메시지의 "shared reference"는 이 저장소에서 부르는 불변 참조 `&T`입니다. 빌린 값에서 소유권을 옮겨 가려 할 때의 E0507은 [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md) 4.4절(`Drop`에서 `&mut`로 빌린 `JoinHandle`을 `join`)에도 나왔습니다. 여기서는 가장 단순한 모양, 곧 불변 참조로 받은 구조체에서 `let`으로 필드를 꺼내는 경우를 Java 습관과 연결해 봅니다. Java의 `String name = account.owner;`는 참조를 하나 더 만들 뿐이지만, Rust의 `let name = account.owner;`는 `String`의 소유권을 옮기는 코드입니다. `account`는 빌린 것(`&Account`)이라 그 안의 필드를 가져가면 빌려 준 쪽의 값에 구멍이 남습니다.
- **해결**: 컴파일러의 두 `help`가 답입니다. 읽기만 하면 `&account.owner`로 빌리고, 따로 가져야 하면 `account.owner.clone()`으로 복제합니다(Q4.6).

### 4.2 같은 값을 `&mut`와 `&`로 동시에 넘김 — E0502

파일: [`compile_fail/e0502_dst_src_alias.rs`](compile_fail/e0502_dst_src_alias.rs)

```text
error[E0502]: cannot borrow `v` as immutable because it is also borrowed as mutable
  --> compile_fail/e0502_dst_src_alias.rs:23:25
   |
23 |     append_copy(&mut v, &v); // E0502
   |     ----------- ------  ^^ immutable borrow occurs here
   |     |           |
   |     |           mutable borrow occurs here
   |     mutable borrow later used by call
```

- **원인**: 함수가 도는 동안 `dst`로 고치고 `src`로 읽는데, 둘이 같은 벡터입니다. `push`가 버퍼를 옮기면 `src`가 해제된 옛 버퍼를 가리키게 되므로(Q4.5), 가변 참조와 불변 참조가 동시에 살아 있는 것을 거부합니다. Part 0 ch8 4.1절의 "요소를 빌린 채 `push`"가 한 함수 안의 일이었다면, 이것은 함수 경계를 넘는 같은 문제입니다.
- **해결**: `v.extend_from_within(..)`처럼 그 일을 안전하게 해 주는 메서드를 쓰거나, 사본을 먼저 만들어 넘깁니다(예제 02).

## 5. 연습 문제

### 연습 1. Java 코드 옮기기

다음 Java 코드와 같은 결과(`a.balance`와 `b.balance`가 모두 150)를 내는 Rust 코드를 예제 01의 네 가지 방법 가운데 하나로 써 보세요. 왜 그 방법이어야 하는지도 설명합니다.

```java
Account a = new Account("하준", 100);
Account b = a;
b.balance += 50;
System.out.println(a.balance + " " + b.balance);
```

<details>
<summary>힌트</summary>

`a`와 `b` 두 이름으로 같은 계좌를 고치고 읽어야 하므로 4번(`Rc<RefCell<Account>>`)입니다. `let a = Rc::new(RefCell::new(open("하준"))); let b = Rc::clone(&a); b.borrow_mut().balance += 50;` 뒤에 `a.borrow().balance`와 `b.borrow().balance`를 출력합니다. 다만 실제 Rust 코드라면 "두 이름이 꼭 필요한가"를 먼저 묻고, 한 소유자가 고치고 다른 쪽은 빌리는 설계(2번)로 바꿀 수 있는지 봅니다.

</details>

### 연습 2. 앞의 원소를 뒤에 덧붙이기

예제 02의 `append_copy`를 써서, 벡터 `[1, 2, 3, 4]`의 앞 두 원소를 뒤에 덧붙여 `[1, 2, 3, 4, 1, 2]`를 만들어 보세요. `extend_from_within`을 쓰는 방법과 사본을 쓰는 방법을 각각 해 봅니다.

<details>
<summary>힌트</summary>

`v.extend_from_within(..2)`가 가장 짧습니다. 사본을 쓴다면 `let head = v[..2].to_vec(); append_copy(&mut v, &head);`처럼 앞부분만 복사해 둡니다. `append_copy(&mut v, &v[..2])`는 4.2절과 같은 이유로 E0502입니다.

</details>

### 연습 3. 크기 예상하기

예제 03에 `struct Polygon { points: Vec<Point> }`과 `struct Triangle { points: [Point; 3] }`을 더하고, 두 구조체의 크기를 먼저 예상한 뒤 `size_of`로 확인해 보세요. 점의 수가 다른 다각형을 담을 수 있는 것은 어느 쪽인지도 설명합니다.

<details>
<summary>힌트</summary>

`Polygon`은 `Vec` 하나(주소, 용량, 길이)라서 24바이트이고, 점들은 힙에 있습니다. `Triangle`은 `Point` 셋을 그 자리에 담으므로 48바이트입니다. 점의 수가 실행 중에 정해지는 다각형은 크기가 바뀌므로 힙을 쓰는 `Polygon`만 담을 수 있습니다(Q4.10의 The Book 설명).

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

- [The Rust Programming Language 4장: Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html): 4.1절 스택과 힙, 4.2절 참조와 대여
- [The Rust Programming Language 10.3절: Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [The Rust Programming Language 15장: Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html), [15.6절: Reference Cycles Can Leak Memory](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)
- [`Vec::extend_from_within`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.extend_from_within)
- [Secure by Design: Google's Perspective on Memory Safety](https://research.google/pubs/secure-by-design-googles-perspective-on-memory-safety/) (Google, 2024-03-04): Rust의 시간적 안전성, 개발 시점의 비용, 순환 구조와 참조 카운팅
- [Why Discord is switching from Go to Rust](https://discord.com/blog/why-discord-is-switching-from-go-to-rust) (Discord, 2020-02-04)

다른 언어

- [Java Language Specification 4.3.1: Objects](https://docs.oracle.com/javase/specs/jls/se21/html/jls-4.html#jls-4.3.1), [`ArrayList`](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/ArrayList.html)(fail-fast 반복자)
- [C# Value types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-types)
- [Go FAQ: How do I know whether a variable is allocated on the heap or the stack?](https://go.dev/doc/faq#stack_or_heap)
- [Python `gc` 모듈](https://docs.python.org/3/library/gc.html)
