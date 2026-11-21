# ch11. C++의 RAII·이동과 Rust

C++ 개발자에게 Rust의 소유권은 낯설지 않습니다. RAII(자원 획득은 초기화, Resource Acquisition Is Initialization)도, `unique_ptr`도, `std::move`도 이미 씁니다. 그래서 질문은 "무엇이 같은가"보다 "무엇이 다른가"입니다. 이동 뒤 원본은 어떻게 되는가, 복사 생성자(copy constructor)와 Rule of three/five/zero는 어디로 갔는가, `const`와 `mutable`은, `optional`·`variant`·`string_view`는 무엇에 대응하는가. 이 챕터는 C++의 관용구를 하나씩 Rust로 옮겨 보고, 같은 이름 아래 무엇이 달라졌는지 답합니다.

- 패키지: `faq-ch11-cpp-raii-move`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 비교 코드를 확인한 컴파일러: Apple clang 21.0.0(C++20), macOS(aarch64). 정적 분석 확인에는 Homebrew LLVM 23.1.2의 `clang-tidy`를 썼습니다.
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q11.1](#q111-c에도-raii와-unique_ptrshared_ptr가-있는데-소유권은-무엇이-다른가요) | C++에도 RAII와 `unique_ptr`·`shared_ptr`가 있는데 소유권은 무엇이 다른가요? | 생각은 같지만, Rust는 모든 값에 소유권 규칙을 적용하고 빌림까지 컴파일러가 검사합니다. 지키지 않으면 컴파일되지 않습니다. |
| [Q11.2](#q112-rust의-이동은-stdmove와-무엇이-다른가요) | Rust의 이동은 `std::move`와 무엇이 다른가요? | Rust의 이동은 원본을 남기지 않는 파괴적 이동이라 원본을 쓸 수 없고 소멸자도 한 번입니다. `std::move`는 캐스트일 뿐이고 원본이 남습니다. |
| [Q11.3](#q113-복사-생성자가-없나요-값은-언제-복사되나요) | 복사 생성자가 없나요? 값은 언제 복사되나요? | 없습니다. `Copy` 타입만 대입할 때 비트가 복사되고, 그 밖의 복사는 `clone()`을 불러야 일어납니다. |
| [Q11.4](#q114-생성자와-소멸자-rule-of-threefivezero는-어떻게-되나요) | 생성자와 소멸자, Rule of three/five/zero는 어떻게 되나요? | 생성자는 관례적인 함수, 소멸자는 `Drop`입니다. 이동 생성자가 없고 복사가 저절로 생기지 않아, 대부분 Rule of zero로 끝납니다. |
| [Q11.5](#q115-const-정확성과-불변-기본은-어떻게-다른가요-mutable에-해당하는-것은-무엇인가요) | `const` 정확성과 불변 기본은 어떻게 다른가요? `mutable`에 해당하는 것은 무엇인가요? | 불변이 기본이고 `&mut`로 고칠 권한을 나눕니다. `mutable` 자리는 `Cell`·`RefCell` 같은 내부 가변성 타입입니다. |
| [Q11.6](#q116-unique_ptr-shared_ptr-weak_ptr-optional-variant-expected-string_view-span은-무엇에-대응하나요) | `unique_ptr`, `shared_ptr`, `weak_ptr`, `optional`, `variant`, `expected`, `string_view`, `span`은 무엇에 대응하나요? | 차례로 `Box`, `Rc`/`Arc`, `Weak`, `Option`, 열거형, `Result`, `&str`, `&[T]`입니다(2절 끝의 한눈에 보기 표). |
| [Q11.7](#q117-예외-안전성은-어떻게-되나요) | 예외 안전성은 어떻게 되나요? | 예외가 없고 패닉은 버그를 뜻하므로 부담이 줄지만, 패닉으로 되감기될 때를 생각하는 일은 남습니다(`unsafe` 코드에서는 필수). |
| [Q11.8](#q118-값-반환-최적화rvo와-out-매개변수는-어떻게-되나요) | 값 반환 최적화(RVO)와 out 매개변수는 어떻게 되나요? | 값을 그대로 돌려주는 것이 기본이고, 이동은 사용자 코드를 부르지 않는 바이트 복사입니다. out 매개변수 대신 튜플·`Option`·`Result`를 돌려줍니다. |
| [Q11.9](#q119-placement-new-이동할-수-없는-타입-자기를-가리키는-객체는-어떻게-하나요) | placement new, 이동할 수 없는 타입, 자기를 가리키는 객체는 어떻게 하나요? | 모든 타입은 바이트 복사로 옮겨질 수 있다고 보므로, 자기를 가리키는 값은 `Pin`으로 "옮기지 않는다"를 약속해 다룹니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md): 소유권(ownership), 이동, 복제와 복사(2.6절), 대여
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md): `Box`, `Drop`(2.6절), `Rc`, `RefCell`과 내부 가변성(2.8절), `Weak`
- [ch4](../ch4-memory-without-gc/README.md) Q4.2·Q4.3(이동과 `Copy`), [ch9](../ch9-c-memory-errors/README.md) Q9.2(이중 해제)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Rust의 파괴적 이동을 C++의 `std::move`·이동 생성자와 비교하고, 이동된 객체와 소멸자 호출 횟수가 어떻게 다른지 설명할 수 있다.
2. 복사 생성자, Rule of three/five/zero, `const`와 `mutable`이 Rust에서 `Copy`·`Clone`·`Drop`, 불변 기본과 내부 가변성으로 어떻게 바뀌는지 설명할 수 있다.
3. C++ 표준 타입(`unique_ptr`, `shared_ptr`, `optional`, `variant`, `expected`, `string_view`, `span`)을 쓴 함수 시그니처를 Rust로 옮길 수 있다.
4. 예외 안전성, RVO와 out 매개변수, 이동할 수 없는 타입에 대해 Rust가 무엇을 다르게 정했는지 설명할 수 있다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다. 이 챕터의 "다른 언어에서는"은 주로 C++입니다.

### Q11.1 C++에도 RAII와 `unique_ptr`·`shared_ptr`가 있는데 소유권은 무엇이 다른가요?

**짧은 답** "값의 수명에 자원의 수명을 묶는다"는 RAII의 생각은 같습니다. 다른 점은 두 가지입니다. Rust는 소유권 규칙을 스마트 포인터를 쓸 때만이 아니라 모든 값에 적용하고, 그 값을 빌린 참조가 값보다 오래 살지 않는지와 고치는 쪽이 하나뿐인지까지 컴파일러가 검사합니다. 규칙을 어기면 경고가 아니라 컴파일 에러입니다.

**왜 그런가** C++에서 `unique_ptr`는 "이 포인터가 유일한 소유자"라는 약속을 타입으로 적은 것이지만, `get()`으로 얻은 원시 포인터나 참조가 그보다 오래 살아도 컴파일러는 막지 않습니다([ch9](../ch9-c-memory-errors/README.md) Q9.1에서 `reset()` 뒤 사용을 ASan이 잡은 예). Rust에서는 참조도 소유권 체계 안에 있어서, 소유자가 사라진 뒤까지 살아남는 참조는 대여 검사기(borrow checker)가 거부합니다.

**다른 언어에서는** C++ Core Guidelines는 소유권을 `unique_ptr`·`shared_ptr` 같은 타입으로 드러내라고 권하지만, 지키는 것은 사람과 도구의 몫입니다. 이 챕터의 compare/01을 `clang-tidy`의 `bugprone-use-after-move` 검사로 돌리면 이동 뒤 사용을 경고하지만, 컴파일러(`-Wall -Wextra`)는 아무 말도 하지 않았습니다.

**오해와 한계** 모든 값에 규칙을 적용하므로, C++에서는 아무렇지 않게 쓰던 구조(서로를 가리키는 객체, 부모를 가리키는 자식)가 Rust에서는 `Rc`·`Weak`나 인덱스로 다시 짜야 할 때가 많습니다([ch4](../ch4-memory-without-gc/README.md) Q4.7, ch15 `ch15-pain-points`).

**더 읽기** Part 0 ch4 2.1~2.3절 / ch4 Q4.1·Q4.4

### Q11.2 Rust의 이동은 `std::move`와 무엇이 다른가요?

**짧은 답** C++ 표준 초안([forward])에 따르면 `std::move`는 인자를 rvalue 참조로 바꾸는 `static_cast`일 뿐이고, 실제 이동은 이동 생성자(move constructor)가 합니다. 이동 뒤에도 원본 객체는 살아 있어 쓸 수 있고 소멸자도 돕니다. Rust의 이동은 값을 그대로 옮기고 원본 변수를 쓸 수 없게 표시하는 파괴적 이동(destructive move)이라, 이동 생성자에 해당하는 코드가 없고 소멸자(`Drop`)도 값마다 한 번만 돕니다.

**왜 그런가** [compare/01_moved_from_object.cpp](compare/01_moved_from_object.cpp)는 이동 생성자에서 원본을 비워 두는 클래스를 이동하고, 이동 뒤 원본을 읽습니다. 이 환경에서 출력은 다음과 같았습니다.

```text
생성: 요청 버퍼
이동: 요청 버퍼
이동 뒤 a.name(): ""
소멸: 요청 버퍼
소멸: ""
```

사용자가 만든 생성자("생성")는 한 번이지만 이동 생성자가 객체 `b`를 하나 더 만들어, 이동된 원본 `a`까지 객체 둘 모두의 소멸자가 불렸습니다. [01_moves_and_drops](examples/01_moves_and_drops.rs)의 1번은 같은 일을 Rust로 합니다.

```text
1. let b = a;
    생성: 요청 버퍼
    b.name = 요청 버퍼
    소멸: 요청 버퍼
```

"이동" 출력이 없고 소멸은 한 번입니다. C++처럼 빈 값을 남겨 두는 이동이 필요하면 `std::mem::take`를 쓰고, 그러면 남은 빈 값도 따로 소멸합니다(2번).

```text
2. 빈 값을 남기는 이동: mem::take
    생성: 응답 버퍼
    a.name = "", b.name = 응답 버퍼
    소멸: 응답 버퍼
    소멸: ""
```

**다른 언어에서는** C++ 표준 초안([lib.types.movedfrom])은 표준 라이브러리 타입의 이동된 객체(moved-from object)를 "유효하지만 지정되지 않은 상태(valid but unspecified state)"로 둔다고 정합니다. 그래서 이동된 `std::string`을 읽는 코드는 컴파일되지만 무엇이 나올지 단정할 수 없습니다. `unique_ptr`처럼 이동된 뒤의 상태를 따로 정한 타입도 있습니다(이동 생성 뒤 원본의 `get()`은 `nullptr`).

**오해와 한계** 파괴적 이동은 "이동된 상태"를 설계할 필요를 없애는 대신, 조건에 따라 이동했다 안 했다 하는 코드에서는 컴파일러가 보수적으로 판단해 이후 사용을 막습니다. 그럴 때는 `Option`에 담고 `take`로 꺼내는 식으로 "비어 있을 수 있음"을 타입에 적습니다(mem::take·replace·swap은 [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 예제 03).

**더 읽기** Part 0 ch4 2.5절 / Part 1 ch7 예제 03 / ch4 Q4.2

### Q11.3 복사 생성자가 없나요? 값은 언제 복사되나요?

**짧은 답** 없습니다. 대입이나 인자 전달은 기본으로 이동이고, `Copy` 트레이트를 구현한 타입(정수, `bool`, `char`, `Copy` 필드만 가지고 `#[derive(Clone, Copy)]`를 붙인 구조체 등)만 대입할 때 비트가 복사되어 원본도 계속 쓸 수 있습니다. 그 밖의 깊은 복사는 `clone()`을 직접 불러야 일어납니다. The Book 4.1절은 Rust가 데이터의 "깊은" 복사를 결코 자동으로 만들지 않는다고 설명합니다.

**왜 그런가** 복사가 언제 일어나는지가 코드에 보이도록 하려는 것입니다. 01의 3번은 `clone`을 부르면 복제가 일어나고 소멸도 둘이 되며, `i32`는 대입이 복사라는 것을 봅니다.

```text
3. 복사는 명시적으로
    생성: 설정
    복제: 설정
    a.name = 설정, b.name = 설정
    x = 7, y = 7
    소멸: 설정
    소멸: 설정
```

표준 라이브러리의 `Copy` 문서는 복사와 이동의 차이가 대입 뒤에 원본을 쓸 수 있느냐뿐이고, 내부적으로는 둘 다 메모리의 비트를 복사할 수 있다고 적습니다.

**다른 언어에서는** C++에서 `T b = a;`는 복사 생성자를 부르고, 클래스에 따라 그것이 깊은 복사일 수도 얕은 복사일 수도 있습니다. 값으로 넘기는 인자도 복사 생성자를 거칩니다. 그래서 "이 줄에서 무슨 일이 일어나는가"를 알려면 클래스 정의를 봐야 합니다.

**오해와 한계** `Copy`는 바이트를 그대로 복사해도 되는 타입에만 구현할 수 있어서, `String`이나 `Vec`을 가진 구조체는 `Copy`가 될 수 없습니다(Part 1 ch6 4.4절의 E0204). `clone()`이 코드에 자주 보이는 것은 비용이 드러난 것일 뿐 나쁜 코드라는 뜻은 아닙니다([ch4](../ch4-memory-without-gc/README.md) Q4.6).

**더 읽기** Part 0 ch4 2.6절 / ch4 Q4.3·Q4.6 / [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 예제 08

### Q11.4 생성자와 소멸자, Rule of three/five/zero는 어떻게 되나요?

**짧은 답** 생성자는 언어 기능이 아니라 `new` 같은 이름의 연관 함수이고([ch6](../ch6-no-classes-inheritance/README.md) Q6.3), 소멸자는 `Drop` 트레이트입니다. Rust에는 이동 생성자가 없고 복사도 저절로 생기지 않으므로(Q11.3), 자원을 직접 다루는 타입만 `Drop`을 구현하고 복제가 필요하면 `Clone`을 구현합니다. 그 밖의 타입은 `String`·`Vec`·`Box` 같은 필드에 맡겨 아무것도 구현하지 않는 Rule of zero로 끝납니다.

**왜 그런가** C++의 Rule of three는 "소멸자를 직접 정의할 정도로 자원을 다루는 클래스라면, 컴파일러가 만든 복사 생성자는 아마 틀린다"는 경험에서 나왔습니다. [compare/02_rule_of_three.cpp](compare/02_rule_of_three.cpp)는 소멸자에서 `delete[]`하면서 복사 생성자를 정의하지 않아, 복사본과 원본이 같은 메모리를 두 번 해제합니다(ASan: `attempting double-free`). Rust에서 `Drop`을 구현한 타입은 `Copy`가 될 수 없고(Part 1 ch6 4.5절의 E0184), `Clone`은 직접 구현하거나 `derive`해야 생기므로 "몰래 생긴 얕은 복사"가 없습니다.

**다른 언어에서는** C++ Core Guidelines는 기본 연산을 정의하지 않아도 되면 정의하지 말고(C.20, Rule of zero), 복사·이동·소멸 가운데 하나를 정의하거나 지우면 모두 정의하거나 지우라고 권합니다(C.21). compare/02의 `int*`를 `std::vector<int>`로 바꾸면 Rule of zero가 되어 이 환경의 ASan도 보고하지 않았습니다(연습 3).

**오해와 한계** `Drop`은 값을 돌려줄 수 없어서 정리 중의 실패를 알릴 수 없습니다([ch5](../ch5-no-null-no-exceptions/README.md) Q5.6). 또 `Drop`을 구현한 타입에서는 필드를 옮겨 꺼낼 수 없습니다(4절 표의 E0509). C++에서 소멸자가 있는 객체의 멤버를 `std::move`로 빼 가던 코드는, 기본값이 있는 필드면 `mem::take(&mut self.필드)`로, 없으면 `Option` 필드와 `take`로 다시 씁니다(부록 A ch11 4.1절).

**더 읽기** Part 0 ch14 2.6절 / [부록 A ch11](../../appd-a/ch11-drop-raii/README.md) / Part 1 ch6 4.4·4.5절

### Q11.5 `const` 정확성과 불변 기본은 어떻게 다른가요? `mutable`에 해당하는 것은 무엇인가요?

**짧은 답** C++에서는 `const`를 붙여야 불변이고, Rust에서는 불변이 기본이며 바꿀 것에만 `mut`을 붙입니다. 메서드는 `&self`(읽기만)와 `&mut self`(고침)로 나뉘어 C++의 `const` 멤버 함수와 일반 멤버 함수에 대응합니다. `const` 멤버 함수 안에서도 고칠 수 있게 하는 `mutable` 멤버의 자리는 `Cell`·`RefCell` 같은 내부 가변성(interior mutability) 타입입니다.

**왜 그런가** C++ 표준 초안([dcl.stc])은 `mutable` 지정자가 객체가 `const`여도 그 멤버를 고칠 수 있게 한다고 정합니다. Rust에서 `&self` 메서드가 필드를 고치려 하면 컴파일되지 않고, 고쳐야 한다면 그 필드를 `Cell`이나 `RefCell`로 감싸 "이 필드는 공유된 채로 고칠 수 있다"를 타입에 적습니다. 같은 상황(`&self`인 `send`에서 기록을 쌓으려다 막히는 예)은 Part 0 ch14 2.8절에 있습니다.

**다른 언어에서는** C++의 `const` 멤버 함수 안에서도 포인터 멤버가 가리키는 대상은 고칠 수 있고(이 환경의 clang은 `-Wall -Wextra`로도 경고하지 않음), `const_cast`로 `const`를 벗길 수도 있습니다. 또 C++의 `const T&`는 "이 경로로는 고치지 않는다"는 약속일 뿐이라, 같은 객체를 다른 별칭이 고치면 const 참조로 읽던 값도 바뀝니다. 이 환경에서 `append_twice(s, s)`처럼 같은 문자열을 `std::string&`와 `const std::string&`로 넘기고 `dst`를 고치자 `src.size()`가 3에서 4로 바뀌었고, 경고도 새니타이저 보고도 없었습니다. 안전한 Rust에서는 불변 참조 `&T`가 살아 있는 동안 `&mut`를 만들 수 없으므로(E0502) 그동안 값이 바뀌지 않고, 그것으로 닿는 값 전체를 고칠 수 없습니다. 예외는 `UnsafeCell`을 안에 둔 타입(`Cell`, `RefCell`, `Mutex` 등)뿐입니다([ch4](../ch4-memory-without-gc/README.md) Q4.5, [ch10](../ch10-undefined-behavior/README.md) Q10.7).

**오해와 한계** `RefCell`은 대여 규칙 검사를 실행 중으로 미루므로, 규칙을 어기면 컴파일 에러 대신 패닉이 납니다(Part 0 ch14 4.6절). 여러 스레드에서 쓰려면 `Mutex`·원자 타입을 씁니다([ch8](../ch8-concurrency-async/README.md) Q8.3).

**더 읽기** Part 0 ch14 2.8절 / Part 1 ch7 예제 06·07 / [ch7](../ch7-syntax-type-surprises/README.md) Q7.1 / ch4 Q4.5 / ch10 Q10.7

### Q11.6 `unique_ptr`, `shared_ptr`, `weak_ptr`, `optional`, `variant`, `expected`, `string_view`, `span`은 무엇에 대응하나요?

**짧은 답** `unique_ptr<T>`는 `Box<T>`, `shared_ptr<T>`는 `Rc<T>`(한 스레드)나 `Arc<T>`(여러 스레드), `weak_ptr<T>`는 `Weak<T>`, `optional<T>`는 `Option<T>`, `variant<...>`는 열거형, `expected<T, E>`는 `Result<T, E>`, `string_view`는 `&str`, `span<T>`는 `&[T]`(`&mut [T]`)입니다. 이름만 다른 것이 아니라, 빌린 쪽(`&str`, `&[T]`)이 원본보다 오래 살지 못하도록 컴파일러가 검사한다는 점이 다릅니다.

**왜 그런가** [02_cpp_signatures](examples/02_cpp_signatures.rs)는 C++ 표준 타입을 쓴 함수 선언을 Rust로 옮겨 실행합니다.

```text
1. optional<string_view> → Option<&str>
  Some("hello"), None
2. variant → 열거형
  수 8080, 글자 "debug"
3. expected → Result
  Ok(443)
  Err("포트가 아님: \"99999\" (number too large to fit in target type)")
4. span<const int> → &[i32]
  전체 252, 앞의 둘 175
5. unique_ptr·shared_ptr 인자 → Box·&·Rc
  inspect: root
  adopt: 4
  share 안의 참조 수: 2
  share 뒤 참조 수: 1
```

소유권을 넘겨받는 `unique_ptr<Node>` 인자는 `Box<Node>`(보통은 그냥 `Node`), 관찰만 하는 `const Node&`는 `&Node`, 공유 소유를 하나 더 갖는 `shared_ptr` 인자는 `Rc`이며, `Rc::clone`으로 넘기면 함수 안에서 참조 수가 2가 되었다가 돌아오면 1로 줄어듭니다. `std::visit`로 `variant`의 대안을 고르던 자리는 `match`가 맡고, 빠뜨린 배리언트는 컴파일러가 알려 줍니다. C++에서 `string_view`나 `span`이 가리키는 원본이 먼저 사라지면 댕글링이 되는 문제는 [ch12](../ch12-modern-cpp-enough/README.md)에서 C++ 코드로 봅니다.

**다른 언어에서는** C++ 표준 초안([util.smartptr.shared.general])은 참조 수(`use_count`) 변경이 데이터 경쟁을 일으키는 수정이 아니라고 정해, 같은 대상을 가리키는 서로 다른 `shared_ptr` 객체를 여러 스레드에서 각각 복사·소멸해도 데이터 경쟁이 되지 않게 합니다. 반면 Rust는 참조 수를 원자적으로 세지 않는 `Rc`와 원자적으로 세는 `Arc`를 나누고, `Rc`를 다른 스레드로 보내면 컴파일 에러로 막습니다([Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 4.4절).

**오해와 한계** 대응표는 출발점일 뿐입니다. `Box`는 널이 될 수 없어 "비어 있을 수 있는 `unique_ptr`"은 `Option<Box<T>>`이고, `Rc`의 순환은 `Weak`로 끊어야 누수가 나지 않습니다(ch4 Q4.7).

**더 읽기** Part 0 ch14 / ch5 Q5.1·Q5.3 / ch6 Q6.8 / [부록 A ch8](../../appd-a/ch8-references-pointers/README.md)

### Q11.7 예외 안전성은 어떻게 되나요?

**짧은 답** Rust에는 예외가 없고, 실패는 `Result`로 돌려주며 패닉은 버그를 알리는 장치입니다(ch5 Q5.3·Q5.7). 그래서 "모든 줄에서 예외가 날 수 있다"를 전제로 강한 보장을 설계하는 부담이 크게 줄어듭니다. 다만 패닉도 스택을 되감으며(unwinding) `Drop`을 실행하므로, 되감기 도중에도 불변식이 깨지지 않게 하는 일은 남습니다. Nomicon은 이것을 두 단계로 나눕니다. `unsafe` 코드는 패닉이 나도 메모리 안전성을 깨지 않아야 하고(최소 보장), 안전한 코드는 할 수 있으면 상태를 올바르게 남기는 것이 좋다는 것입니다.

**왜 그런가** 이것을 흔히 예외 안전성(exception safety)이라 부르고, 패닉 때 스택을 거슬러 올라가며 값을 정리하는 것을 되감기라고 합니다. 패닉이 나도 메모리 안전성은 유지된다는 것은 [ch2](../ch2-memory-safety/README.md) Q2.6에서 봤습니다. 논리적인 상태가 반쯤 바뀐 채 남을 수 있는 경우를 위해 표준 라이브러리는 `Mutex`의 포이즈닝(poisoning) 같은 장치를 두고, `catch_unwind`는 `UnwindSafe`로 되감기 뒤에 써도 되는 값인지를 표시합니다([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 2.9절).

**다른 언어에서는** C++에서는 생성자·연산자·컨테이너 연산 어디서든 예외가 날 수 있어서, 함수마다 기본·강한·예외 없음(nothrow) 보장 가운데 무엇을 약속하는지 설계하고 문서로 밝힙니다. Rust의 이동은 사용자 코드를 부르지 않으므로 이동 도중에 실패할 일이 없습니다(Q11.8).

**오해와 한계** `panic = "abort"`로 빌드하면 되감기 없이 프로세스가 끝나므로 이 문제가 사라지는 대신 정리 코드도 돌지 않습니다([Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.3절). `unsafe` 코드에서 일시적으로 불변식을 깬 채 사용자 코드(클로저, 트레이트 메서드)를 부르면, 그 안의 패닉이 깨진 상태를 남길 수 있다는 점을 따져야 합니다.

**더 읽기** ch5 Q5.7 / Part 1 ch8 2.9절 / Nomicon "Exception Safety"

### Q11.8 값 반환 최적화(RVO)와 out 매개변수는 어떻게 되나요?

**짧은 답** Rust에서는 큰 값도 그대로 돌려주는 것이 기본 관용구입니다. 이동은 사용자 코드(이동 생성자)를 부르지 않는 바이트 복사이고, 표준 라이브러리의 `Copy` 문서도 이 복사가 때로 최적화로 사라진다고 적습니다. 여러 결과를 돌려주거나 실패를 알리려고 쓰던 out 매개변수(`bool parse(…, int& out)`) 대신 튜플, `Option`, `Result`를 돌려줍니다(02 예제의 `parse_port`).

**왜 그런가** C++에서 복사 생성자가 비싼 타입은 반환 비용을 줄이려고 RVO에 기대거나 out 매개변수를 썼습니다. Rust에서 값을 옮기는 비용은 그 값의 바이트 크기만큼의 복사이고(힙 버퍼를 가진 `String`·`Vec`은 포인터·길이·용량만 옮김), 그 과정에서 사용자가 정의한 코드가 돌지 않습니다. 그래서 "돌려줄까, 채워 줄까"를 반환 비용 때문에 고를 일이 줄어듭니다.

**다른 언어에서는** C++ 표준 초안([class.copy.elision])은 반환문 같은 경우에 객체 생성을 생략하는 복사 생략(copy elision)을 정합니다. Rust에는 이동 생성자가 없으므로 "생략되지 않으면 사용자 코드가 돈다"는 걱정 자체가 없습니다.

**오해와 한계** 바이트 복사가 생략될지는 최적화에 달려 있어 보장되지 않습니다. 아주 큰 배열 같은 값을 자주 옮기는 코드는 `Box`에 담거나 `&mut`로 넘겨 채우는 편이 나을 수 있지만, 그 판단은 측정으로 합니다(ch14 `ch14-performance`).

**더 읽기** Part 0 ch4 2.7절 / ch5 Q5.3 / ch14 `ch14-performance`

### Q11.9 placement new, 이동할 수 없는 타입, 자기를 가리키는 객체는 어떻게 하나요?

**짧은 답** Rust는 모든 값을 바이트 복사로 옮길 수 있다고 보고, 이동 생성자나 이동을 금지하는 선언이 없습니다. 그래서 자기 안의 다른 필드를 가리키는 값은 그대로 두면 옮기는 순간 깨집니다. 이런 값은 `Pin`으로 "이 값은 이제 제자리에서 옮기지 않는다"를 약속해 다루며, 주로 비동기의 퓨처가 이 문제를 겪습니다. 직접 쓸 일은 드물고, 자세한 것은 [Part 1 ch13](../../part1-standard-lib/ch13-async-primitives/README.md) 2.7절에 있습니다.

**왜 그런가** 표준 라이브러리의 `std::pin` 문서는 포인터가 가리키는 값이 메모리 위치에서 옮겨지지 않는다는 보장을 "고정(pinning)"이라 부르고, 자기 주소에 기대는 상태를 가진 타입을 위한 것이라고 설명합니다. 안전한 코드에서 자기 참조 구조체를 만들려 하면 대여 검사기가 거부합니다([부록 A ch15](../../appd-a/ch15-borrow-checker/README.md) 4.5절의 E0505).

**다른 언어에서는** C++에서는 이동·복사 생성자를 지워(`= delete`) 이동할 수 없는 타입을 만들고, 자기를 가리키는 객체는 이동 생성자에서 포인터를 고쳐 다시 잇습니다. placement new는 이미 잡은 메모리에 객체를 만드는 기능입니다. Rust에서 초기화하지 않은 메모리에 값을 만드는 일은 `MaybeUninit`과 `unsafe`로 합니다(ch9 Q9.5).

**오해와 한계** 자기 참조가 꼭 필요하면 보통은 인덱스로 가리키거나 데이터를 둘로 나눠 구조를 바꾸는 편이 `Pin`보다 쉽습니다(부록 A ch15). 퓨처를 직접 구현할 때가 아니면 `Pin`은 런타임과 `async` 문법이 대신 다룹니다.

**더 읽기** Part 1 ch13 2.7절 / 부록 A ch15 / Part 10 `ch1-safe-and-unsafe`

### 한눈에 보기

| C++ | Rust | 질문 |
| --- | --- | --- |
| RAII, 소멸자 | `Drop` | Q11.1, Q11.4 |
| `std::move`, 이동 생성자, 이동된 객체 | 파괴적 이동(원본을 쓸 수 없음), 빈 값을 남기려면 `mem::take` | Q11.2 |
| 복사 생성자 | `Copy`(비트를 복사해도 되는 타입), `Clone`(명시적 `clone()`) | Q11.3 |
| Rule of three/five/zero | 대부분 Rule of zero. 자원 타입만 `Drop`, 필요하면 `Clone` | Q11.4 |
| `const` 멤버 함수 / 일반 멤버 함수 | `&self` / `&mut self` | Q11.5 |
| `mutable` 멤버 | `Cell`, `RefCell`(한 스레드), `Mutex`·원자 타입(여러 스레드) | Q11.5 |
| `unique_ptr<T>` | `Box<T>`(비어 있을 수 있으면 `Option<Box<T>>`) | Q11.6 |
| `shared_ptr<T>`, `weak_ptr<T>` | `Rc<T>`·`Arc<T>`, `Weak<T>` | Q11.6 |
| `optional<T>` | `Option<T>` | Q11.6 |
| `variant<...>`, `std::visit` | 열거형, `match` | Q11.6 |
| `expected<T, E>` | `Result<T, E>` | Q11.6 |
| `string_view`, `span<T>` | `&str`, `&[T]`(라이프타임으로 검사) | Q11.6 |
| 예외, 예외 안전성 보장 | `Result`, 패닉(되감기 중 `Drop`), `unsafe`의 최소 보장 | Q11.7 |
| RVO, out 매개변수 | 값 반환(바이트 이동), 튜플·`Option`·`Result` 반환 | Q11.8 |
| 이동할 수 없는 타입, 자기 참조 | `Pin`, 구조를 바꿔 자기 참조를 피함 | Q11.9 |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_moves_and_drops](examples/01_moves_and_drops.rs) | 이동은 소멸 한 번, `mem::take`는 빈 값을 남겨 소멸 둘, `clone`은 명시적 | Q11.1~Q11.4 |
| [02_cpp_signatures](examples/02_cpp_signatures.rs) | `optional`·`variant`·`expected`·`span`, `unique_ptr`·`shared_ptr` 인자를 쓴 C++ 시그니처를 Rust로 | Q11.6, Q11.8 |

### 01_moves_and_drops — 이동과 소멸

```sh
cargo run -p faq-ch11-cpp-raii-move --example 01_moves_and_drops
```

```text
1. let b = a;
    생성: 요청 버퍼
    b.name = 요청 버퍼
    소멸: 요청 버퍼
2. 빈 값을 남기는 이동: mem::take
    생성: 응답 버퍼
    a.name = "", b.name = 응답 버퍼
    소멸: 응답 버퍼
    소멸: ""
3. 복사는 명시적으로
    생성: 설정
    복제: 설정
    a.name = 설정, b.name = 설정
    x = 7, y = 7
    소멸: 설정
    소멸: 설정
```

- 이동하면 해제가 한 번, `clone`하면 둘이라는 것은 [ch9](../ch9-c-memory-errors/README.md) 예제 01이 C의 `free` 관점에서 보였습니다. 이 예제가 더하는 것은 C++의 이동 생성자·이동된 객체와 소멸 횟수를 나란히 놓는 것과, `mem::take`가 남긴 빈 값도 따로 소멸한다는 점입니다.
- 구역마다 블록 `{ }`으로 감싸, 소멸이 그 구역 끝에서 일어나게 했습니다. 지역 변수는 선언의 역순으로 버려지므로 2번에서는 `b`가, 3번에서는 `b`(사본)가 먼저 소멸합니다.
- `mem::take`는 `Default`를 구현한 타입에만 쓸 수 있어서 `Buffer`에 `#[derive(Default)]`를 붙였습니다.
- `Clone`을 직접 구현해 "복제"를 출력하게 했습니다. 실제 코드에서는 보통 `#[derive(Clone)]`으로 충분합니다.

### 02_cpp_signatures — C++ 시그니처 옮기기

```sh
cargo run -p faq-ch11-cpp-raii-move --example 02_cpp_signatures
```

```text
1. optional<string_view> → Option<&str>
  Some("hello"), None
2. variant → 열거형
  수 8080, 글자 "debug"
3. expected → Result
  Ok(443)
  Err("포트가 아님: \"99999\" (number too large to fit in target type)")
4. span<const int> → &[i32]
  전체 252, 앞의 둘 175
5. unique_ptr·shared_ptr 인자 → Box·&·Rc
  inspect: root
  adopt: 4
  share 안의 참조 수: 2
  share 뒤 참조 수: 1
```

- 5번의 `adopt(node: Box<Node>)`는 `unique_ptr<Node>`를 값으로 받는 C++ 함수와 나란히 보이려고 `Box`로 받았습니다. Clippy는 이것을 `boxed_local`로 알리는데, Rust에서는 보통 `Node`를 값으로 받으면 되고 `Box`는 트레이트 객체나 재귀 타입처럼 힙에 둘 이유가 있을 때 씁니다. 그래서 예제는 `#[expect(clippy::boxed_local, …)]`를 붙였습니다. 관찰만 하는 함수는 `&Box<Node>`가 아니라 `&Node`를 받습니다(`inspect`).
- `first_word`가 돌려주는 `Option<&str>`은 인자 `s`에서 빌린 것이라, 라이프타임 생략 규칙(Part 0 ch10 2.15절)에 따라 `s`보다 오래 쓸 수 없습니다.
- `sum(&scores[..2])`처럼 슬라이스의 일부를 넘기는 것은 C++의 `std::span(scores).first(2)`에 해당합니다.

### C++ 비교 코드

저장소 루트에서 실행합니다. `c++`의 `-o`는 출력 디렉터리를 만들지 않으므로 먼저 `mkdir -p target/tmp/faq-ch11-cpp-raii-move`를 실행합니다. 파일마다 머리 주석에 명령이 있습니다.

| 파일 | 질문 | 명령(요지) | 이 환경에서 나온 것 |
| --- | --- | --- | --- |
| [01_moved_from_object.cpp](compare/01_moved_from_object.cpp) | Q11.2 | `c++ -std=c++20 -Wall -Wextra` 후 실행 | 경고 없음, 출력은 Q11.2(정해진 출력) |
| [02_rule_of_three.cpp](compare/02_rule_of_three.cpp) | Q11.4 | `c++ -std=c++20 -g -fsanitize=address,undefined` 후 실행 | `ERROR: AddressSanitizer: attempting double-free`, 종료 134 |

- 01은 원본을 빈 문자열로 만드는 이동 생성자를 직접 정의했으므로 출력이 정해져 있습니다(지정되지 않은 상태도 미정의 동작도 없음). 표준 라이브러리 타입을 이동한 뒤 읽었다면 그 값이 지정되지 않아 출력을 싣지 않았을 것입니다(Q11.2).
- 01을 `clang-tidy -checks='-*,bugprone-use-after-move' appd-b/ch11-cpp-raii-move/compare/01_moved_from_object.cpp -- -std=c++20`으로 검사하면 ``'a' used after it was moved [bugprone-use-after-move]`` 경고가 나왔습니다(Homebrew LLVM 23.1.2의 `clang-tidy`). 컴파일러와 달리 따로 돌려야 하는 정적 분석 도구입니다.

## 4. 자주 하는 실수

C++ 습관대로 쓰면 나는 에러는 대부분 앞 챕터에 `compile_fail` 파일로 이미 있어서, 이 챕터에는 새 파일을 두지 않았습니다. 대신 습관과 에러, 그것을 본 곳을 표로 모았습니다.

| C++ 습관 | Rust에서 나는 것 | 본 곳 |
| --- | --- | --- |
| `std::move` 뒤에 원본을 씀 | E0382 `borrow of moved value` | Part 0 ch4 4.1절 |
| 소멸자가 있는 객체에서 멤버를 `std::move`로 빼 감 | E0509 `cannot move out of type …, which implements the Drop trait` | 부록 A ch11 4.1절 |
| 소멸자도 정의하고 비트 복사도 허용하려 함 | E0184 `Copy`와 `Drop`을 함께 구현할 수 없음 | Part 1 ch6 4.5절 |
| `const` 멤버 함수에서 `mutable` 멤버를 고침 | `&self` 메서드에서 필드에 대입하면 E0594(``cannot assign to `self.n`, which is behind a `&` reference``, 1.100.0에서 확인), `&mut` 메서드를 부르면 E0596. `Cell`·`RefCell`로 해결 | E0596은 Part 0 ch14 2.8절 |
| 소멸자를 직접 부름 | E0040 `explicit use of destructor method` | Part 0 ch14 4.3절 |
| `const T&`에서 멤버를 이동해 감 | E0507 `cannot move out of … which is behind a shared reference` | [ch4](../ch4-memory-without-gc/README.md) 4.1절 |
| 지역 변수의 참조를 돌려줌 | E0515·E0106 | [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 4.5절, Part 0 ch4 4.5절 |
| 자기 멤버를 가리키는 포인터를 멤버로 둠 | E0505 등 | 부록 A ch15 4.5절 |

이 가운데 C++과 가장 크게 다른 것은 위의 두 줄입니다. 이동 뒤 원본을 쓰는 코드는 C++에서는 컴파일되고(compare/01), 소멸자가 있는 객체에서 멤버를 빼 가는 코드도 C++에서는 흔한 관용구입니다. Rust는 둘 다 컴파일 시간에 막고, 빼 가야 하면 기본값이 있는 필드는 `mem::take(&mut self.필드)`로, 없으면 `Option` 필드와 `take`로 쓰게 합니다.

## 5. 연습 문제

### 연습 1. 함수로 넘기면 언제 소멸하나

01의 1번 끝에 `fn consume(buf: Buffer) { println!("    consume 안: {}", buf.name); }`를 만들어 `consume(b);`를 부르고, 그 뒤에 `println!("    consume 뒤");`를 더해 보세요. "소멸"이 어느 줄 사이에 나올지 예상한 뒤 확인합니다.

<details>
<summary>힌트</summary>

`b`의 소유권이 `consume`의 매개변수 `buf`로 옮겨 가므로, 소멸은 `consume`이 끝날 때 일어납니다. 출력 순서는 "consume 안: 요청 버퍼", "소멸: 요청 버퍼", "consume 뒤"입니다. C++에서 값으로 넘기면 복사 생성자(또는 `std::move`를 쓰면 이동 생성자)가 불리고 원본도 나중에 따로 소멸하는 것과 다릅니다(Q11.2).

</details>

### 연습 2. `span`과 `optional`을 함께

02에 `fn average(xs: &[i32]) -> Option<f64>`를 더해, 빈 슬라이스면 `None`, 아니면 평균을 돌려주게 해 보세요. C++이라면 `std::optional<double> average(std::span<const int> xs)`인 함수입니다.

<details>
<summary>힌트</summary>

`if xs.is_empty() { None } else { Some(xs.iter().sum::<i32>() as f64 / xs.len() as f64) }`처럼 씁니다. `average(&[90, 85, 77])`은 `Some(84.0)`, `average(&[])`는 `None`입니다. 빈 경우를 `Option`으로 돌려주면 부르는 쪽이 0으로 나누는 일을 잊을 수 없습니다.

</details>

### 연습 3. compare/02를 Rule of zero로

compare/02에 `#include <vector>`를 더하고, `int *data_`를 `std::vector<int> data_`로 바꾸고 `size_` 멤버를 지운 뒤, 생성자를 `data_(size)`로, `size()`를 `data_.size()`로 고치고 소멸자를 지워 보세요. 같은 명령(ASan)으로 실행하면 어떻게 되는지 확인합니다.

<details>
<summary>힌트</summary>

`vector`가 자기 메모리를 직접 관리하므로 컴파일러가 만든 복사 생성자가 깊은 복사를 하고, 이 환경에서는 ASan 보고 없이 `4 4`를 출력하고 종료 상태 0으로 끝났습니다. 자원 관리를 그것을 맡은 타입에 넘기고 기본 연산을 정의하지 않는 Rule of zero(Core Guidelines C.20)이며, Rust에서 `String`·`Vec` 필드만 가진 구조체가 아무것도 구현하지 않아도 되는 것과 같은 생각입니다(Q11.4).

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rust Programming Language: [4.1절 What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html), [15장 Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- 표준 라이브러리 문서: [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html), [`std::mem::take`](https://doc.rust-lang.org/std/mem/fn.take.html), [`std::pin`](https://doc.rust-lang.org/std/pin/index.html)
- The Rustonomicon: [Exception Safety](https://doc.rust-lang.org/nomicon/exception-safety.html)

C++

- C++ 표준 초안: [\[forward\]](https://eel.is/c++draft/forward)(`std::move`), [\[lib.types.movedfrom\]](https://eel.is/c++draft/lib.types.movedfrom), [\[unique.ptr.single.ctor\]](https://eel.is/c++draft/unique.ptr.single.ctor), [\[dcl.stc\]](https://eel.is/c++draft/dcl.stc)(`mutable`), [\[class.copy.elision\]](https://eel.is/c++draft/class.copy.elision), [\[util.smartptr.shared.general\]](https://eel.is/c++draft/util.smartptr.shared.general)
- C++ Core Guidelines: [C.20](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines#rc-zero), [C.21](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines#rc-five)
- clang-tidy: [bugprone-use-after-move](https://clang.llvm.org/extra/clang-tidy/checks/bugprone/use-after-move.html)
- 같은 주제를 다룬 자료: Brown 대학교 [C++ to Rust Phrasebook](https://cel.cs.brown.edu/crp/)(이 챕터의 질문 범위를 정할 때 목차를 참고했고, 본문과 코드는 옮기지 않았습니다)
