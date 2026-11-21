# ch12. 모던 C++로 충분하지 않은가

C++ 개발자가 Rust를 처음 볼 때 가장 먼저 드는 생각은 "그건 이미 모던 C++로 해결했다"입니다. `unique_ptr`와 `shared_ptr`로 해제를 맡기고, Core Guidelines를 따르고, AddressSanitizer와 정적 분석(static analysis)을 CI에 넣고, C++26은 표준 라이브러리 하드닝(hardening)까지 들였습니다. 이 챕터는 그 도구들을 이 환경에서 직접 돌려 보고, 무엇을 잡고 무엇이 남는지, Rust가 그 남는 부분에서 무엇을 다르게 하는지 답합니다. C++가 더 나은 점도 함께 적습니다.

- 패키지: `faq-ch12-modern-cpp-enough`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 비교 코드를 확인한 컴파일러: Apple clang 21.0.0(C++20)과 그 libc++, macOS(aarch64). 정적 분석 확인에는 Homebrew LLVM 23.1.2의 `clang-tidy`를 썼습니다.
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q12.1](#q121-c를-조심해서-잘-쓰고-있는데-rust가-무엇을-더-주나요) | C++를 조심해서 잘 쓰고 있는데 Rust가 무엇을 더 주나요? | 조심은 사람이 매번 하는 일입니다. Rust는 수명과 공유·변경에 관한 규칙을 컴파일러가 모든 코드에 대해 검사해, "찾아야 하는 버그"를 "컴파일되지 않는 코드"로 바꿉니다. |
| [Q12.2](#q122-스마트-포인터와-core-guidelines를-지키면-메모리-안전하지-않나요) | 스마트 포인터와 Core Guidelines를 지키면 메모리 안전하지 않나요? | 해제를 잊는 문제는 해결되지만, 소유하지 않는 접근(뷰, 반복자, 원시 포인터, 캡처한 `this`)이 소유자보다 오래 쓰이는 문제와 소유를 잘못 나누는 문제는 남습니다. |
| [Q12.3](#q123-addresssanitizer-정적-분석-퍼징으로-잡으면-되지-않나요) | AddressSanitizer, 정적 분석, 퍼징으로 잡으면 되지 않나요? | 꼭 쓸 도구지만, 새니타이저(sanitizer)는 실행된 경로에서 실제로 일어난 오류만, 정적 분석은 아는 패턴만 잡습니다. 이 환경에서 넷 다 놓친 미정의 동작이 있었습니다. |
| [Q12.4](#q124-c-표준도-안전성을-강화하고-있지-않나요) | C++ 표준도 안전성을 강화하고 있지 않나요? | 그렇습니다. C++26에 표준 라이브러리 하드닝과 "초기화하지 않은 지역 변수 읽기는 미정의 동작이 아님"이 들어갔고, Profiles는 C++29를 목표로 작업 중입니다. 하드닝은 경계 검사를 잡지만 수명 문제는 잡지 않았습니다. |
| [Q12.5](#q125-안전이-기본값이라는-것이-왜-중요한가요) | 안전이 "기본값"이라는 것이 왜 중요한가요? | 켜야 하는 검사는 빠지는 곳이 생깁니다. Rust는 검사가 기본으로 켜져 있고, 컴파일러가 확인할 수 없는 연산은 `unsafe`로 표시한 자리에서만 할 수 있어 검토를 그곳에 모을 수 있습니다. |
| [Q12.6](#q126-반대로-c가-더-나은-점은-무엇인가요) | 반대로 C++가 더 나은 점은 무엇인가요? | 쌓인 코드와 라이브러리, 여러 구현과 ISO 표준, 더 강력한 컴파일 시간 프로그래밍, C++ 코드와 직접 섞기, 다시 컴파일만 해서 얻는 개선입니다. |
| [Q12.7](#q127-c는-어떤가요-커널이나-임베디드에서-c-대신-쓸-만한가요) | C는 어떤가요? 커널이나 임베디드에서 C 대신 쓸 만한가요? | 쓰입니다. Linux 커널은 Rust 지원을 실험 단계에서 끝냈고, 운영체제 없는 타깃은 `no_std`로 지원합니다. 다만 그런 타깃은 Tier 1에 없고 "빌드 보장"(Tier 2)이나 그 아래 등급이며, C 코드와의 경계는 `unsafe`입니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [ch9](../ch9-c-memory-errors/README.md): C의 메모리 오류와 새니타이저(특히 Q9.1 해제 후 사용, Q9.7 반복자 무효화)
- [ch11](../ch11-cpp-raii-move/README.md): C++의 RAII·이동과 Rust의 대응(Q11.2 이동, Q11.6 표준 타입 대응)
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md): `Rc`, `Weak`, `RefCell`
- [부록 A ch14](../../appd-a/ch14-lifetimes/README.md): 라이프타임과 `'static` 바운드(2.7절, 4.4절)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 스마트 포인터·뷰·람다만 쓴 모던 C++ 코드에 남는 수명 버그를 예로 들고, 같은 구조가 Rust에서 어떤 컴파일 에러가 되는지 설명할 수 있다.
2. `std::function`과 달리 구조체 필드나 함수 시그니처에 쓴 `Box<dyn FnMut()>`가 기본으로 `'static`을 요구한다는 것을 설명하고, 콜백이 상태를 소유하게(`Rc`) 하거나 살아 있는지 묻게(`Weak`) 고칠 수 있다.
3. 새니타이저, 정적 분석, 표준 라이브러리 하드닝이 각각 무엇을 잡고 무엇을 놓치는지 이 환경에서 확인한 결과로 설명할 수 있다.
4. C++26의 안전성 작업 현황과 "안전이 기본값"이라는 말의 뜻을 설명하고, C++가 더 나은 점과 C 대신 Rust를 쓸 때의 비용을 말할 수 있다.

## 2. 핵심 개념

### Q12.1 C++를 조심해서 잘 쓰고 있는데 Rust가 무엇을 더 주나요?

**짧은 답** 조심해서 쓰는 것은 사람이 매번, 모든 변경에서 해야 하는 일입니다. Rust는 그 조심 가운데 "참조가 가리키는 값보다 오래 쓰이지 않는가", "공유하는 동안 고치지 않는가"를 컴파일러가 모든 코드에 대해 검사합니다. 리뷰와 도구가 찾아야 했던 버그가 컴파일되지 않는 코드가 됩니다. 대가로 규칙에 맞게 설계를 바꿔야 하고, 컴파일러가 증명하지 못하는 올바른 코드도 거부됩니다.

**왜 그런가** 이 챕터의 `compare/` 파일 01·03·04는 원시 `new`/`delete` 없이 스마트 포인터, `string_view`, 람다만 쓴 코드입니다. 셋 다 `-Wall -Wextra`로 경고 없이 컴파일되고, AddressSanitizer(ASan)를 켜고 실행해야 `heap-use-after-free`가 보입니다. 같은 구조를 Rust로 쓰면 모두 컴파일 에러입니다.

| C++ 코드 | 컴파일(`-Wall -Wextra`) | ASan을 켜고 실행 | 같은 구조의 Rust |
| --- | --- | --- | --- |
| 함수를 거친 `string_view`가 임시 `string`을 가리킴(01) | 경고 없음 | `heap-use-after-free` | E0716([Part 1 ch3](../../part1-standard-lib/ch3-strings-fmt/README.md) 4.2절) |
| `[this]`를 캡처한 콜백이 객체보다 오래 삶(03) | 경고 없음 | `heap-use-after-free` | E0597(이 챕터 4절) |
| `get()`으로 얻은 포인터를 `reset()` 뒤에 씀(04) | 경고 없음 | `heap-use-after-free` | E0505(부록 A ch14 4.1절) |

C++에서는 "이 코드를 실행했을 때 그 경로가 지나가야" 버그가 드러나고, Rust에서는 그 경로를 실행하지 않아도 컴파일 단계에서 드러납니다.

**다른 언어에서는** "조심하면 된다"가 큰 코드에서 어떻게 되는지는 [ch2](../ch2-memory-safety/README.md) Q2.3의 숫자가 보여 줍니다. Microsoft와 Chromium이 C/C++가 중심인 자기 제품의 보안 취약점을 세었더니 약 70%가 메모리 안전성 문제였습니다. 숙련된 개발자가 많고 도구를 갖춘 조직의 결과입니다.

**오해와 한계** Rust가 컴파일 시간에 막는 것은 안전한 코드의 메모리 오류와 데이터 경쟁뿐이고, 논리 오류는 그대로입니다([ch3](../ch3-safety-limits/README.md) Q3.1). `unsafe` 안에서는 C++와 같은 책임을 사람이 집니다. 또 C++를 조심해서 써 온 경험은 Rust에서도 그대로 쓸모 있습니다. 소유자와 수명을 먼저 생각하는 습관이 대여 검사기(borrow checker)의 에러를 빨리 읽게 해 줍니다(ch11).

**더 읽기** ch2 Q2.3·Q2.8 / ch3 Q3.1 / ch9 Q9.1 / ch11 Q11.1

### Q12.2 스마트 포인터와 Core Guidelines를 지키면 메모리 안전하지 않나요?

**짧은 답** 해제를 잊는 문제는 스마트 포인터로 해결되지만, 소유하지 않는 접근의 수명은 남습니다. 뷰(`string_view`, `span`), 반복자, 원시 포인터와 참조, 람다가 캡처한 `this`가 그렇습니다. 같은 포인터를 두 소유자에게 넘기는 것처럼 소유를 잘못 나누는 코드도 컴파일됩니다. Core Guidelines도 원시 포인터와 참조를 "소유하지 않는" 용도로 쓰라고 하므로, 그것들이 소유자보다 오래 쓰이지 않게 하는 일은 지침의 Lifetime 프로필과 도구에 맡겨집니다. Rust는 이 수명을 타입에 적고 컴파일러가 검사합니다.

**왜 그런가** `compare/`의 세 조각은 모두 소유는 올바릅니다. 해제를 빠뜨리거나 두 번 하는 곳이 없습니다. 문제는 소유하지 않는 쪽입니다.

- 01: `first_word(make_line())`이 돌려준 `string_view`가 문장 끝에서 소멸한 임시 `string`의 버퍼를 가리킵니다. clang은 한 줄로 쓴 `std::string_view w = make_line();`에는 `-Wdangling-gsl` 경고를 내지만, 함수를 한 번 거치면 경고하지 않았습니다. `first_word`의 매개변수에 `[[clang::lifetimebound]]`를 붙여 "반환값이 이 인자를 가리킨다"고 알려 주면 그때는 경고했습니다. Rust에서는 `fn first_word(s: &str) -> &str`의 라이프타임 생략 규칙이 이 정보를 기본으로 적습니다.
- 03: `std::function`은 담은 람다가 무엇을 가리키는지 타입에 남기지 않습니다. 그래서 `[this]`를 캡처한 람다를 `Button`에 등록하고 `Counter`를 먼저 해제해도 컴파일러가 막을 근거가 없습니다. Rust에서는 구조체 필드나 함수 시그니처에 쓴 `Box<dyn FnMut()>`가 `Box<dyn FnMut() + 'static>`과 같으므로(Reference "Default trait object lifetimes". 함수 본문의 `let` 타입 표기에서는 라이프타임이 추론됨), 빌린 것을 품은 클로저를 넣으려 하면 등록하는 줄에서 E0597이 납니다.
- 04: Core Guidelines의 R.3은 원시 포인터(`T*`)를 소유하지 않는 포인터로 쓰라고 합니다. `node.get()`으로 얻은 포인터를 관찰용으로 드는 것은 지침에 맞지만, 그 포인터가 `node.reset()` 뒤에 쓰여도 막는 것은 없습니다.

03의 Rust 판을 컴파일되게 고치는 방법은 [01_callbacks_own_state](examples/01_callbacks_own_state.rs)에 있습니다. 콜백이 상태를 빌리지 않고 소유하게 합니다.

```text
1. Rc를 옮겨 콜백이 공동 소유
  블록 안의 참조 수: 2
  클릭 수: 1
  클릭 수: 2
2. Weak를 옮겨 살아 있는지 묻기
  클릭 수: 1
  Counter가 이미 사라져 무시
```

**다른 언어에서는** Core Guidelines의 Lifetime 프로필(Pro.lifetime)은 이미 해제된 객체를 가리키는 포인터를 역참조하지 않게 하려는 규칙 묶음입니다. 지침은 이 프로필이 스타일 규칙, 정적 분석, 라이브러리 지원을 함께 써서 완전히 강제될 때 효과가 있다고 적습니다. 지침만으로는 강제되지 않는다는 뜻입니다. C++에서 03을 올바르게 고치는 방법 가운데 하나는 `weak_ptr`을 캡처하는 것이고(소멸자에서 등록을 해제하는 방법도 있음), 예제의 2번(`Weak`)이 그 관용구에 해당합니다. 차이는 C++에서는 `[this]`도 컴파일된다는 점입니다.

**오해와 한계** 스마트 포인터도 소유를 잘못 나누는 코드는 막지 못합니다. 같은 원시 포인터를 `unique_ptr` 둘에 넘기면 컴파일되고, 이 환경에서 ASan이 `attempting double-free`를 보고했습니다(ch9 Q9.2). Rust에서 원시 포인터로 소유자를 만드는 `Box::from_raw`는 `unsafe` 함수라 안전한 코드에서는 쓸 수 없습니다. 한편 Rust의 해법에도 비용이 있습니다. `Rc`는 참조 수를 세고, `RefCell`은 대여 규칙을 실행 중에 검사하므로 잘못 쓰면 컴파일 대신 실행 중에 패닉합니다(Part 0 ch14). `Rc`끼리 서로 가리키면 누수가 납니다(ch3 Q3.4). 그리고 C++에서는 컴파일되던 구조(콜백이 등록한 쪽의 지역 객체를 빌림)를 Rust에서는 처음부터 다르게 설계해야 합니다. 이동한 뒤 원본을 쓰는 "이동 후 사용"은 ch11 Q11.2에서, 순회 중 `push_back`은 ch9 Q9.7에서 봤습니다.

**더 읽기** ch9 Q9.1·Q9.7 / ch11 Q11.2·Q11.6 / 부록 A ch14 2.7절 / Part 0 ch14

### Q12.3 AddressSanitizer, 정적 분석, 퍼징으로 잡으면 되지 않나요?

**짧은 답** 꼭 써야 할 도구입니다. 다만 새니타이저는 실행된 경로에서 실제로 잘못된 메모리에 닿은 경우만 잡고, 정적 분석은 아는 패턴만 잡습니다. 퍼징(fuzzing)은 실행되는 경로를 넓혀 줄 뿐 이 성질을 바꾸지 않습니다. 이 환경에서 [compare/02](compare/02_reserve_hides_bug.cpp)는 표준상 미정의 동작인데 ASan, UBSan, 하드닝, clang-tidy가 모두 조용했습니다. Rust의 대여 검사는 실행 여부와 상관없이 컴파일되는 모든 코드에 대해 일어납니다.

**왜 그런가** 이 챕터의 C++ 코드를 도구마다 돌린 결과입니다. "보고 없음"은 그 도구가 이 문제(수명·반복자·경계)를 알리지 않았다는 뜻이고, 코드가 올바르다는 뜻이 아닙니다.

| 코드 | ASan+UBSan 실행 | `clang-tidy`(`bugprone-*`, `clang-analyzer-*`, `cppcoreguidelines-*`) | libc++ 하드닝(DEBUG) 실행 |
| --- | --- | --- | --- |
| 01 `string_view` | `heap-use-after-free` | 보고 없음(한 줄로 쓰면 `bugprone-dangling-handle`) | 보고 없음 |
| 02 `reserve` 뒤 순회 중 `push_back` | 보고 없음 | 보고 없음 | 보고 없음 |
| 03 `[this]` 캡처 | `heap-use-after-free` | 보고 없음 | 보고 없음 |
| 04 `get()` 뒤 `reset()` | `heap-use-after-free` | 보고 없음 | 보고 없음 |
| 05 경계 밖 인덱스 | (돌리지 않음) | (돌리지 않음) | `libc++ Hardening assertion … vector[] index out of bounds` |

`clang-tidy`는 수명과 상관없는 경고(예외가 `main` 밖으로 나갈 수 있음, C 가변 인자 함수 사용 등)만 냈습니다. 02는 ch9의 [compare/06](../ch9-c-memory-errors/compare/06_iterator_invalidation.cpp)에 `reserve` 한 줄을 더한 것입니다. ch9 Q9.7은 이때 ASan이 보고하지 않는다는 것까지 적었고, 여기서는 UBSan, 하드닝, `clang-tidy`도 조용하다는 것을 더합니다. 재할당이 없으니 해제된 메모리에 닿지 않아 ASan이 볼 것이 없지만, C++ 표준 초안([vector.modifiers])은 재할당이 없어도 끝 반복자를 무효로 보므로 범위 기반 `for`가 잡아 둔 끝 반복자와 비교하는 순간 미정의 동작입니다. 용량이라는 실행 중의 값에 따라 도구가 보는 것이 달라지고, 테스트 입력이 그 값을 바꾸면 결과도 바뀝니다. 같은 구조의 Rust 코드는 용량과 상관없이 E0502로 컴파일되지 않습니다([Part 0 ch8](../../part0-getting-started/ch8-collections/README.md) 4.1절).

새니타이저에는 실행 비용도 있습니다. Clang 문서는 ASan의 보통 속도 저하를 2배로 적고, ASan 런타임이 보안을 고려해 만든 것이 아니므로 배포용 실행 파일에 링크하지 말라고 합니다. 그래서 새니타이저는 테스트와 퍼징에서 쓰고, 배포된 프로그램은 보호하지 않습니다.

**다른 언어에서는** 실행된 경로만 본다는 한계는 Go의 경쟁 검출기도 같습니다([ch1](../ch1-why-rust/README.md) Q1.8, [ch8](../ch8-concurrency-async/README.md)). Rust에서도 실행 중 도구가 필요한 곳이 있습니다. `unsafe` 코드는 Miri나 새니타이저로 확인해야 하고([부록 A ch16](../../appd-a/ch16-avoid-unsafe/README.md)), 논리 오류와 패닉은 테스트와 퍼징으로 찾습니다([부록 A ch30](../../appd-a/ch30-beyond-unit-tests/README.md)).

**오해와 한계** 컴파일 시간 검사가 도구를 대신하지는 않습니다. Rust 프로젝트도 `unsafe`를 쓰는 부분, C 라이브러리와의 경계, 입력 파서에는 퍼징과 새니타이저를 씁니다. 차이는 그 도구가 필요한 범위입니다. C++에서는 모든 코드가 대상이고, Rust에서는 `unsafe`와 그것을 감싼 모듈로 좁혀집니다(Q12.5).

**더 읽기** ch9 Q9.7 / [ch10](../ch10-undefined-behavior/README.md) Q10.1 / 부록 A ch16·ch30 / ch8

### Q12.4 C++ 표준도 안전성을 강화하고 있지 않나요?

**짧은 답** 강화하고 있습니다. 2026년 3월 기술 작업을 마친 C++26에는 표준 라이브러리 하드닝(경계 같은 전제 조건을 실행 중에 검사)과, 초기화하지 않은 지역 변수를 읽는 것을 미정의 동작에서 빼는 변경이 들어갔습니다. Profiles는 C++26에 들어가지 않았고, 2026년 6월 회의에서 C++29를 목표로 명세를 만들기로 했습니다. 다만 이 환경에서 하드닝은 경계 밖 인덱스를 잡았지만 이 챕터의 수명 문제 넷은 잡지 않았습니다.

**왜 그런가** 위원회 의장을 지낸(2002~2025) Herb Sutter의 회의 보고와 표준 초안에서 확인한 내용입니다.

| 작업 | 현황 | 근거 |
| --- | --- | --- |
| 표준 라이브러리 하드닝(P3471) | C++26에 채택 | 2025-02 Hagenberg 회의 보고 |
| 초기화하지 않은 지역 변수 읽기 | C++26에서 미정의 동작이 아님. 초안은 그 바이트가 "오류 값(erroneous value)"을 가지고, 그 값을 만들어 내는 평가는 오류 동작(erroneous behavior)이라고 정함. 다만 동적 할당한 객체와 `[[indeterminate]]`를 붙인 변수는 여전히 미정(indeterminate) 값이라 읽으면 미정의 동작 | 2026-03 London Croydon 회의 보고, 초안 [basic.indet] |
| C++26 전체 | 기술 작업 완료, 국제 승인 투표용 최종 문서를 만드는 중. ISO가 곧 발행할 예정이라고만 적고 연도는 밝히지 않음 | 2026-03 London Croydon 회의 보고 |
| Profiles | C++26에 채택되지 않음. 안전·보안 연구 그룹(SG23)이 C++29를 목표로 명세를 만들기로 했고, 그러지 못하면 C++29와 함께 백서로 낼 수 있다고 함 | 2025-02 Hagenberg, 2026-06 Brno 회의 보고 |

표준 초안 [structure.specifications]는 하드닝된 전제 조건(hardened precondition)을 어기면 하드닝된 구현에서는 계약 검사가 프로그램을 끝내는 방식으로 평가되고, 하드닝되지 않은 구현에서는 미정의 동작이라고 정합니다. 이 환경의 libc++는 같은 검사를 `_LIBCPP_HARDENING_MODE` 매크로로 이미 제공합니다. [compare/05](compare/05_hardened_bounds.cpp)를 모드별로 컴파일해 실행한 결과입니다.

| 모드 | 결과 |
| --- | --- |
| `NONE` | 경계 밖 읽기, 미정의 동작(출력은 싣지 않음) |
| `FAST`, `EXTENSIVE` | 메시지 없이 멈춤, 종료 상태 133(SIGTRAP) |
| `DEBUG` | `libc++ Hardening assertion __n < size() failed: vector[] index out of bounds`, 종료 상태 134 |

같은 `DEBUG` 모드로 compare/01~04를 컴파일해 실행하면 넷 다 아무 보고 없이 종료 상태 0이었습니다. 하드닝은 함수의 전제 조건(인덱스 < 크기)을 검사하지만, 뷰나 포인터가 가리키는 객체가 아직 살아 있는지는 검사하지 않습니다.

**다른 언어에서는** Rust에서는 두 변경에 해당하는 것이 처음부터 기본입니다. 인덱싱은 언제나 경계를 검사하고(ch9 Q9.4), 초기화하지 않은 변수를 읽으면 컴파일 에러입니다(ch9 Q9.5). C++ 안에 Rust와 비슷한 대여 검사를 들이자는 제안 [P3390R0 "Safe C++"](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2024/p3390r0.html)(2024-09-11, 대상 SG23)도 있습니다. 이 제안의 현재 상태는 1차 자료로 확인하지 못해 적지 않습니다.

**오해와 한계** 하드닝의 큰 장점은 기존 코드를 고치지 않고 다시 컴파일만 해서 얻는다는 것입니다. Rust로 다시 쓰는 것보다 훨씬 싼 개선이고, C++ 코드가 많은 조직에는 현실적인 첫걸음입니다. 대가는 검사의 실행 비용입니다. Profiles가 C++29에 들어가면 수명 검사가 어디까지 될지는 이 챕터를 쓰는 시점에 정해지지 않았으므로 예상하지 않습니다. 위 현황은 2026-11-21에 확인한 것이고, 2026년 11월 Búzios 회의 이후의 변화는 반영하지 않았습니다.

**더 읽기** ch9 Q9.4·Q9.5 / ch10 Q10.1 / Q12.6

### Q12.5 안전이 "기본값"이라는 것이 왜 중요한가요?

**짧은 답** 같은 검사라도 켜야 하는 것과 꺼야 하는 것은 결과가 다릅니다. 켜야 하는 검사는 빠지는 파일, 빠지는 빌드, 빠지는 경로가 생깁니다. Rust에서는 대여 검사와 경계 검사가 기본으로 켜져 있고, 컴파일러가 확인할 수 없는 연산(원시 포인터 역참조, `unsafe` 함수 호출 등)은 `unsafe`라고 적은 자리에서만 할 수 있습니다. 그 자리가 코드에 남아 검토와 도구를 그곳에 모을 수 있습니다(대여 검사는 `unsafe` 안에서도 그대로, ch3 Q3.2). C++에서는 안전한 쪽(`at()`, 하드닝, 새니타이저, `clang-tidy`, 지침)을 골라서 켜야 합니다.

**왜 그런가** 이 챕터의 C++ 코드는 모두 아무 옵션 없이도, `-Wall -Wextra`를 켜도 경고 없이 컴파일됐습니다. 버그를 보려면 새니타이저, 하드닝 매크로, `clang-tidy` 검사 목록을 따로 골라야 했고, 그렇게 골라도 02는 아무것도 잡지 못했습니다. libc++ 문서도 벤더가 정하는 하드닝 모드의 기본값은 `none`(검사 없음)이고 어느 수준을 켰는지는 벤더에게 확인하라고 적는데, 이 환경의 libc++도 매크로 없이 컴파일하면 `NONE` 모드였습니다. 다만 GCC의 libstdc++는 최적화하지 않은 빌드에서 일부 전제 조건 검사를 기본으로 켭니다(ch2 Q2.4). Rust에서는 같은 구조가 기본 설정으로 컴파일되지 않습니다. `unsafe` 블록으로 감싸도 compile_fail의 E0597은 그대로이고(대여 검사는 꺼지지 않음), 피해 가려면 참조 대신 원시 포인터를 쓰고 그것을 `unsafe` 안에서 역참조해야 합니다. 크레이트 맨 위에 `#![forbid(unsafe_code)]`를 두면 "이 크레이트에는 `unsafe`가 없다"는 것까지 컴파일러가 확인합니다(ch3 4.2절).

C++26도 초기화하지 않은 지역 변수에서는 같은 설계를 택했습니다. 기본은 미정의 동작이 아닌 오류 동작이고, 미정 값을 원하면 `[[indeterminate]]`를 붙여 명시적으로 빠져나갑니다(Q12.4). `unsafe`를 쓴 자리가 표시된다는 것은 검토 범위가 좁아진다는 뜻입니다. `unsafe` 블록과 그것을 감싼 모듈만 사람이 깊이 보면 되고, 나머지 코드의 메모리 안전성은 컴파일러가 맡습니다(부록 A ch16). ch2 Q2.8의 Android 보고도 같은 방향입니다. Android 팀은 메모리 안전성 취약점의 비율이 줄어든 원인을 새 코드를 메모리 안전한 언어로 쓴 데서 찾았습니다.

**다른 언어에서는** Java, Go, C# 같은 GC 언어도 메모리 안전성이 기본값입니다(ch2 Q2.5). Rust의 차이는 그 기본값을 GC 없이 컴파일 시간 검사로 얻는다는 점입니다.

**오해와 한계** 기본값이 안전해도 `unsafe` 안의 코드와, 그것을 감싼 안전한 API가 건전한지는 사람의 책임입니다(ch3 Q3.2·Q3.3). 기본값에는 비용도 있습니다. 경계 검사는 실행 중에 비용이 들고(ch9 Q9.11), 대여 검사기는 증명하지 못하는 올바른 코드를 거부합니다([ch4](../ch4-memory-without-gc/README.md)).

**더 읽기** ch2 Q2.5·Q2.8 / ch3 Q3.2, 4.2절 / ch9 Q9.11 / 부록 A ch16

### Q12.6 반대로 C++가 더 나은 점은 무엇인가요?

**짧은 답** 여러 가지입니다. 수십 년 동안 쌓인 코드, 라이브러리, 도구와 개발자가 있습니다. 여러 독립 구현과 ISO 표준이 있습니다. 템플릿 특수화와 가변 인자 템플릿처럼 더 강력한 컴파일 시간 프로그래밍을 씁니다. 기존 C++ 코드와 직접 섞어 쓸 수 있습니다. 자기 참조나 침습적(intrusive) 자료 구조를 그대로 표현합니다. 하드닝처럼 다시 컴파일만 해서 얻는 개선도 있습니다.

**왜 그런가**

| C++가 나은 점 | Rust의 현재 상태 | 더 읽기 |
| --- | --- | --- |
| 기존 코드와 라이브러리, 개발자 | 새로 쓰거나 FFI로 연결해야 함 | ch17 `ch17-interop-migration` |
| C++ 코드와 직접 섞기 | C ABI(`extern "C"`)를 거치거나 `cxx` 같은 도구를 씀 | [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md), [부록 A ch34](../../appd-a/ch34-ffi-boundaries/README.md) |
| 템플릿 특수화 | 안정판에서 쓸 수 없음. `#![feature(specialization)]`은 안정판에서 E0554 | [ch6](../ch6-no-classes-inheritance/README.md) Q6.6 |
| 가변 인자 템플릿 | 가변 인자 제네릭이 없음. 표준 라이브러리도 `PartialEq` 등 여러 트레이트를 원소 12개 이하의 튜플에만 구현(`tuple` 문서) | ch6 Q6.7, [ch13](../ch13-headers-build-templates/README.md) |
| 자기 참조, 침습적 자료 구조 | `Pin`, `Rc`/`Weak`, 또는 `unsafe`가 필요 | ch11 Q11.9, [부록 A ch15](../../appd-a/ch15-borrow-checker/README.md) |
| 여러 독립 구현과 ISO 표준 | 주 구현은 `rustc` 하나. 다른 구현은 초기 단계 | ch16 `ch16-tooling-ecosystem` |
| 다시 컴파일만 해서 얻는 안전성 개선 | 다시 써야 얻음 | Q12.4 |

C++의 템플릿은 본문 가운데 `T`에 기대는 부분을 쓰이는 자리에서 실제 타입으로 검사하므로 제약을 미리 적지 않아도 되고(C++20 concepts로 적는 것은 선택, [ch13](../ch13-headers-build-templates/README.md) Q13.5), Rust의 제네릭은 본문에서 쓰는 능력을 모두 트레이트 바운드로 적어야 합니다(ch6 Q6.6). 에러가 정의 쪽에서 일찍 나는 대신 표현할 수 있는 것이 그만큼 좁습니다. 가변 인자가 없는 것은 ch6 Q6.7에서 봤습니다.

**다른 언어에서는** 같은 질문을 Rust 쪽에서 보면, Rust가 맞지 않는 곳에 대한 ch1 Q1.6의 답과 이어집니다. 빨리 결과를 봐야 하거나, 기존 생태계를 벗어날 수 없거나, 팀이 새 언어를 배울 여유가 없는 곳입니다.

**오해와 한계** "더 나은 점"은 상황에 따라 무게가 다릅니다. 새 코드의 메모리 안전성을 가장 앞에 두는 조직은 위의 점들을 비용으로 받아들이고, 거대한 C++ 코드와 생태계 위에서 일하는 팀에게는 이 점들이 결정적입니다. 둘 중 하나를 고르기보다 새 구성 요소부터 Rust로 쓰고 경계를 FFI로 잇는 방식도 있습니다(ch1 Q1.7).

**더 읽기** ch1 Q1.6·Q1.7·Q1.8 / ch6 Q6.6·Q6.7 / ch11 Q11.9

### Q12.7 C는 어떤가요? 커널이나 임베디드에서 C 대신 쓸 만한가요?

**짧은 답** 쓰이고 있습니다. Linux 커널 메인테이너들은 2025년 12월 커널의 Rust 지원을 실험 단계에서 끝내기로 합의했고(ch1 Q1.4), 운영체제 없이 도는 타깃은 표준 라이브러리 없이 `core`만 쓰는 `no_std`로 지원합니다. 다만 대상 아키텍처가 Rust의 타깃 목록에 있어야 하고, 운영체제 없는 타깃은 모두 "빌드 보장"(Tier 2)이나 그 아래 등급이며, C 코드와의 경계는 `unsafe`입니다.

**왜 그런가** `rustc` 문서의 Platform Support 페이지는 타깃을 세 등급으로 나눕니다. Tier 1은 "동작 보장"으로 바뀔 때마다 빌드와 테스트를 자동으로 돌리고, Tier 2는 "빌드 보장"으로 표준 라이브러리(또는 `core`만)의 공식 빌드를 내지만 테스트는 보장하지 않으며, Tier 3는 코드는 있지만 공식 빌드와 자동 검사가 없습니다. 이 툴체인(1.100.0)에 딸린 문서에서 이름에 `-none`이 들어간 운영체제 없는 타깃을 세어 보면 Tier 1은 없고, Tier 2(호스트 도구 없음)가 33개, Tier 3가 32개입니다. 예를 들어 Armv7E-M(하드웨어 부동소수점)용 `thumbv7em-none-eabihf`는 Tier 2이고 `no_std` 개발만 지원한다고 표시되어 있습니다.

`no_std` 실행 파일은 패닉했을 때 할 일을 직접 정해야 합니다. 이 환경에서 `#![no_std]`와 `#![no_main]`만 둔 파일을 실행 파일로 컴파일하면 첫 에러로 `` `#[panic_handler]` function required, but not found ``가, 이어서 `unwinding panics are not supported without std`가 났습니다. `no_std` 라이브러리를 쓰는 법은 부록 A ch33에, 임베디드 실습은 Part 8 `ch1-intro-no-std`에서 다룹니다.

**다른 언어에서는** C는 커널과 펌웨어에서 오래 써 온 언어이고, 언어 사이를 잇는 ABI의 기준이기도 합니다. Rust의 FFI도 `extern "C"`로 C ABI를 씁니다(Part 1 ch14). C의 메모리 오류를 Rust가 어떻게 막는지는 ch9에서 봤습니다.

**오해와 한계** 기존 C 코드를 두고 새 구성 요소부터 Rust로 쓰면(ch1 Q1.7의 Android처럼) C와 Rust의 경계가 생깁니다. 경계의 C 함수는 `unsafe extern` 블록으로 선언하고 대부분의 호출에 `unsafe` 블록이 필요하며([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)), 바인딩을 유지하는 비용이 듭니다(부록 A ch34, [부록 A ch35](../../appd-a/ch35-bindgen/README.md)). 대상 아키텍처가 Rust 타깃 목록에 없거나 Tier 3라면 C를 쓰는 편이 현실적입니다.

**더 읽기** ch1 Q1.4 / ch9 / [부록 A ch33](../../appd-a/ch33-no-std/README.md)·ch34 / Part 1 ch14 / Part 8 `ch1-intro-no-std`

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_callbacks_own_state](examples/01_callbacks_own_state.rs) | `'static` 콜백이 상태를 `Rc`로 소유하거나 `Weak`로 살아 있는지 묻기 | Q12.2 |

### 01_callbacks_own_state — 콜백이 상태를 소유하게 하기

```sh
cargo run -p faq-ch12-modern-cpp-enough --example 01_callbacks_own_state
```

```text
1. Rc를 옮겨 콜백이 공동 소유
  블록 안의 참조 수: 2
  클릭 수: 1
  클릭 수: 2
2. Weak를 옮겨 살아 있는지 묻기
  클릭 수: 1
  Counter가 이미 사라져 무시
```

- `Rc`·`Weak`·`RefCell` 자체와, 소유자를 버린 뒤 `upgrade`가 `None`이 되는 것은 [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 예제 08(트리의 부모 링크)에서 봤습니다. 이 예제가 더하는 것은 `'static` 콜백을 등록하는 자리에서 두 방법 가운데 무엇을 고르는가입니다.
- 1번은 콜백이 `Counter`를 함께 소유합니다. 블록이 끝나 `counter` 변수가 사라져도 `Button`이 든 클로저가 `Rc`를 가지고 있어 `Counter`가 살아 있습니다. 콜백이 대상의 수명을 늘려도 될 때 씁니다.
- 2번은 콜백이 `Weak`만 가집니다. `Counter`의 수명은 원래 소유자가 정하고, 콜백은 부를 때마다 `upgrade`로 살아 있는지 묻습니다. C++에서 `weak_ptr`을 캡처하는 관용구와 같습니다.
- `Button`의 필드와 `on_click`의 매개변수에 쓴 `Box<dyn FnMut()>`는 `Box<dyn FnMut() + 'static>`과 같으므로, 등록하는 클로저는 빌린 것을 품을 수 없습니다. 함수 본문에서 `let mut v: Vec<Box<dyn FnMut()>> = Vec::new();`처럼 쓴 타입 표기에서는 라이프타임이 추론되므로, 그런 지역 목록에는 지역 변수를 빌린 클로저도 넣을 수 있습니다.

### C++ 비교 코드

저장소 루트에서 실행합니다. `c++`의 `-o`는 출력 디렉터리를 만들지 않으므로 먼저 `mkdir -p target/tmp/faq-ch12-modern-cpp-enough`를 실행합니다. 파일마다 머리 주석에 명령이 있습니다.

| 파일 | 질문 | 명령(요지) | 이 환경에서 나온 것 |
| --- | --- | --- | --- |
| [01_string_view_temporary.cpp](compare/01_string_view_temporary.cpp) | Q12.2 | `c++ -std=c++20 -g -fsanitize=address,undefined` 후 실행 | `ERROR: AddressSanitizer: heap-use-after-free`, 종료 134 |
| [02_reserve_hides_bug.cpp](compare/02_reserve_hides_bug.cpp) | Q12.3 | 같음 | 보고 없음, 종료 0(그래도 미정의 동작이라 출력은 싣지 않음) |
| [03_lambda_captures_this.cpp](compare/03_lambda_captures_this.cpp) | Q12.2 | 같음 | `ERROR: AddressSanitizer: heap-use-after-free`, 종료 134 |
| [04_unique_ptr_get_reset.cpp](compare/04_unique_ptr_get_reset.cpp) | Q12.2 | 같음 | `ERROR: AddressSanitizer: heap-use-after-free`, 종료 134 |
| [05_hardened_bounds.cpp](compare/05_hardened_bounds.cpp) | Q12.4 | `c++ -std=c++20 -D_LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_DEBUG` 후 인자 없이 실행 | `libc++ Hardening assertion __n < size() failed: vector[] index out of bounds`, 종료 134 |

- 01~05는 `c++ -std=c++20 -Wall -Wextra`로 컴파일해도 경고가 없었습니다.
- `clang-tidy`는 `clang-tidy --checks='-*,bugprone-*,clang-analyzer-*,cppcoreguidelines-*' <파일> -- -std=c++20 -isysroot "$(xcrun --show-sdk-path)"`로 돌렸습니다(Homebrew LLVM의 `clang-tidy`가 macOS SDK의 헤더를 찾도록 `-isysroot`를 줌). 01~04에서 수명 문제를 보고한 검사는 없었습니다.
- 05의 다른 모드(`NONE`, `FAST`, `EXTENSIVE`)와, `DEBUG` 모드로 01~04를 돌린 결과는 Q12.4에 있습니다.

## 4. 자주 하는 실수

### 4.1 지역 값을 빌린 클로저를 `'static` 콜백으로 등록 — E0597

파일: [`compile_fail/e0597_callback_borrows_local.rs`](compile_fail/e0597_callback_borrows_local.rs)

```text
error[E0597]: `counter` does not live long enough
  --> compile_fail/e0597_callback_borrows_local.rs:53:34
   |
52 |         let mut counter = Counter::default();
   |             ----------- binding `counter` declared here
53 |         button.on_click(Box::new(counter.handler())); // E0597
   |                         ---------^^^^^^^-----------
   |                         |        |
   |                         |        borrowed value does not live long enough
   |                         coercion requires that `counter` is borrowed for `'static`
54 |     } // counter가 여기서 drop됩니다.
   |     - `counter` dropped here while still borrowed
   |
   = note: due to object lifetime defaults, `Box<dyn FnMut()>` actually means `Box<(dyn FnMut() + 'static)>`
```

- **읽는 법**: `counter.handler()`가 돌려준 클로저는 `counter`를 빌립니다. 그 클로저를 `Box<dyn FnMut()>`로 강제 변환(coercion)하는 자리에서 "`counter`를 `'static` 동안 빌려야 한다"는 요구가 생겼습니다. 코드 어디에도 `'static`을 적지 않았는데, 마지막 `note`가 그 이유를 알려 줍니다. 기본 객체 라이프타임(object lifetime default) 규칙 때문에 `Box<dyn FnMut()>`가 실제로는 `Box<(dyn FnMut() + 'static)>`이라는 것입니다. 블록 끝에서 drop되는 `counter`는 그 요구를 채울 수 없습니다.
- **원인**: C++의 [compare/03](compare/03_lambda_captures_this.cpp)과 같은 구조입니다. C++의 `std::function`은 담은 람다가 무엇을 가리키는지 잊지만, 필드나 시그니처에 쓴 Rust의 `Box<dyn FnMut()>`는 `'static`이라는 라이프타임을 타입에 가지고 있습니다. `'static` 바운드가 빌린 것을 품지 않기를 요구한다는 원리는 부록 A ch14 2.7절과 4.4절(스레드에 넘기는 경우, E0521)에서 봤습니다. 숨은 `'static` 요구로 E0597이 나는 것은 Part 1 ch14 4.2절(`&dyn Any`)과도 같은 교훈입니다. 여기서는 그 요구가 콜백 목록(필드)과 `on_click` 매개변수의 타입에 쓴 트레이트 객체의 기본 라이프타임에서 조용히 생기고, 에러는 매개변수로 강제 변환하는 자리를 가리킨다는 점이 다릅니다.
- **해결**: 콜백이 상태를 소유하게 합니다. `Rc<RefCell<_>>`을 `move`로 옮기거나, 대상의 수명을 늘리면 안 되면 `Weak`를 옮깁니다([01_callbacks_own_state](examples/01_callbacks_own_state.rs)). 콜백 목록이 대상보다 먼저 사라진다는 것을 보장할 수 있으면 `Button<'a>`처럼 라이프타임 매개변수를 두는 방법도 있습니다(연습 1).

### 4.2 C++의 다른 습관과 이미 본 에러

이 챕터의 다른 C++ 조각에 해당하는 Rust 에러는 앞 챕터에 이미 있습니다.

| C++에서 하던 것 | Rust에서 나는 에러 | 본 곳 |
| --- | --- | --- |
| 임시 객체를 가리키는 `string_view`를 계속 씀(compare/01) | E0716 | Part 1 ch3 4.2절 |
| 순회 중 `push_back`(compare/02, ch9 compare/06) | E0502 | Part 0 ch8 4.1절, ch9 Q9.7 |
| `get()`으로 얻은 포인터를 소유자 해제 뒤에 씀(compare/04) | E0505 | 부록 A ch14 4.1절 |
| 지역 변수를 참조로 캡처한 람다를 돌려줌 | E0373 | [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.4절 |
| `std::move`한 객체를 다시 씀 | E0382 | ch11 Q11.2, [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 4.1절 |

## 5. 연습 문제

### 연습 1. `'static`이 아니어도 되는 콜백 목록

`compile_fail/e0597_callback_borrows_local.rs`에서 블록을 없애고 `counter`를 `button`보다 먼저 선언하면 컴파일될까요? 안 된다면, `Button`에 라이프타임 매개변수 `'a`를 두어 컴파일되게 고쳐 보세요.

<details>
<summary>힌트</summary>

블록을 없애도 같은 E0597이 납니다. `counter`가 `button`보다 오래 살아도 `main`의 지역 변수는 `'static`이 아니기 때문입니다. `struct Button<'a> { handlers: Vec<Box<dyn FnMut() + 'a>> }`, `impl<'a> Button<'a>`, `fn on_click(&mut self, f: Box<dyn FnMut() + 'a>)`로 바꾸면 컴파일되고 `clicks: 1`이 출력됩니다. 대신 `button`이 살아 있는 동안 `counter`는 가변으로 빌려진 상태라 다른 곳에서 쓸 수 없습니다. 지역 변수는 선언의 역순으로 drop되므로 `counter`를 먼저 선언해야 합니다.

</details>

### 연습 2. C++ 쪽을 고치기

`compare/01_string_view_temporary.cpp`를 두 가지로 고쳐 보세요. (1) `make_line()`의 결과를 `std::string line`에 담은 뒤 `first_word(line)`을 부르고 ASan으로 실행합니다. (2) 원래 코드에서 `first_word`의 매개변수에 `[[clang::lifetimebound]]`를 붙이고 `-Wall -Wextra`로 컴파일합니다.

<details>
<summary>힌트</summary>

(1)은 ASan 보고 없이 `request-handler`를 출력하고 종료 상태 0입니다. 소유자 `line`이 `w`보다 오래 살기 때문입니다. (2)는 `-Wdangling-gsl` 경고(``object backing the pointer will be destroyed at the end of the full-expression``)가 납니다. C++에서도 수명 정보를 적으면 컴파일러가 도울 수 있지만, 적는 것은 선택이고 적지 않은 함수는 검사되지 않습니다. Rust의 라이프타임 생략 규칙은 같은 정보를 기본으로 적습니다.

</details>

### 연습 3. `Counter`는 언제 해제되나

`examples/01_callbacks_own_state.rs`의 `Counter`에 `Drop`을 구현해 `Counter 해제(클릭 N번)`을 출력하게 하고 실행해 보세요. 1번 구역의 `Counter`는 언제 해제될까요?

<details>
<summary>힌트</summary>

2번 구역의 `Counter`는 블록 끝에서 해제되어 `Counter가 이미 사라져 무시`보다 먼저 `Counter 해제(클릭 1번)`이 나옵니다. 1번 구역의 `Counter`는 `main`이 끝날 때 맨 마지막에 `Counter 해제(클릭 2번)`으로 해제됩니다. 2번 구역의 `let mut button`은 첫 `button`을 가릴 뿐(섀도잉) drop하지 않으므로, 첫 `button`과 그 안의 `Rc`는 `main` 끝까지 살아 있습니다.

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rust Reference: [Default trait object lifetimes](https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes)
- 표준 라이브러리 문서: [`std::rc::Weak`](https://doc.rust-lang.org/std/rc/struct.Weak.html), [`tuple`](https://doc.rust-lang.org/std/primitive.tuple.html)(트레이트 구현이 원소 12개 이하로 제한됨)
- The rustc book: [Platform Support](https://doc.rust-lang.org/rustc/platform-support.html)
- Linux 커널 문서: [Rust](https://docs.kernel.org/rust/index.html)

C++

- 표준화 현황(위원회 의장을 지낸 Herb Sutter의 회의 보고): [2025-02 Hagenberg](https://herbsutter.com/2025/02/17/trip-report-february-2025-iso-c-standards-meeting-hagenberg-austria/)(게시 2025-02-17), [2026-03 London Croydon](https://herbsutter.com/2026/03/29/c26-is-done-trip-report-march-2026-iso-c-standards-meeting-london-croydon-uk/)(게시 2026-03-29), [2026-06 Brno](https://herbsutter.com/2026/06/13/brno-trip-report/)(게시 2026-06-13)
- C++ 표준 초안: [\[structure.specifications\]](https://eel.is/c++draft/structure.specifications)(하드닝된 전제 조건), [\[basic.indet\]](https://eel.is/c++draft/basic.indet)(오류 값), [\[vector.modifiers\]](https://eel.is/c++draft/vector.modifiers)
- WG21 제안: [P3390R0 Safe C++](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2024/p3390r0.html)(2024-09-11)
- C++ Core Guidelines: [R.3](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines#rr-ptr), [Pro.lifetime](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines#ss-lifetime)
- Clang: [AddressSanitizer](https://clang.llvm.org/docs/AddressSanitizer.html)(속도 저하, 배포용 실행 파일에 링크하지 말 것), [`lifetimebound` 속성](https://clang.llvm.org/docs/AttributeReference.html#lifetimebound)
- clang-tidy: [bugprone-dangling-handle](https://clang.llvm.org/extra/clang-tidy/checks/bugprone/dangling-handle.html)
- libc++: [Hardening Modes](https://libcxx.llvm.org/Hardening.html)(모드 이름, 기본값 `none`, 실패했을 때의 동작)
