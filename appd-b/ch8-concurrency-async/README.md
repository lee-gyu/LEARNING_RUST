# ch8. 동시성과 비동기

Rust는 "겁 없는 동시성(fearless concurrency)"을 내세웁니다. Java, Go, C++, Python, JavaScript로 동시성 코드를 써 온 사람은 그 말이 정확히 무엇을 약속하는지 묻습니다. 데이터 경쟁은 어떻게 컴파일 시간에 막는가, `Mutex`는 왜 데이터를 감싸는가, 고루틴 같은 가벼운 스레드는 있는가, 비동기 런타임(runtime)은 왜 표준 라이브러리에 없는가, `async`는 JavaScript와 무엇이 다른가. 이 챕터는 스레드·`Mutex`·`async`의 사용법을 다시 가르치지 않습니다(Part 0 ch15·ch16, Part 1 ch11·ch13의 일입니다). 대신 다른 언어의 동시성 모델과 비교해 Rust가 무엇을 막고 무엇은 막지 못하는지에 답합니다.

- 패키지: `faq-ch8-concurrency-async`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 비교 코드를 확인한 컴파일러: Apple clang 21.0.0, macOS(aarch64)
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q8.1](#q81-겁-없는-동시성은-무슨-뜻인가요) | "겁 없는 동시성"은 무슨 뜻인가요? | 동시성 버그 가운데 데이터 경쟁 같은 많은 수를 실행 전에 컴파일 에러로 바꾼다는 뜻입니다. 모든 동시성 버그를 막는다는 뜻은 아닙니다. |
| [Q8.2](#q82-데이터-경쟁을-컴파일-시간에-어떻게-막나요) | 데이터 경쟁을 컴파일 시간에 어떻게 막나요? | 가변 참조는 하나뿐이라는 대여 규칙(borrowing rules)과, 스레드 사이에 보내거나 함께 써도 되는 타입을 나타내는 `Send`·`Sync`로 막습니다. |
| [Q8.3](#q83-mutex가-왜-데이터를-감싸나요-c의-stdmutex-java의-synchronized-go의-syncmutex와-비교) | `Mutex`가 왜 데이터를 감싸나요? (C++의 `std::mutex`, Java의 `synchronized`, Go의 `sync.Mutex`와 비교) | 잠그지 않고는 데이터에 닿을 수 없게 하려는 것입니다. 다른 언어의 잠금은 데이터와 따로 있어 잠그는 것을 빠뜨려도 컴파일됩니다. |
| [Q8.4](#q84-python의-gil-go의-race-detector-cc의-threadsanitizer와-무엇이-다른가요) | Python의 GIL, Go의 race detector, C/C++의 ThreadSanitizer와 무엇이 다른가요? | 세 도구는 실행 중에 일어난 경쟁을 찾거나(검출기) 인터프리터를 한 번에 한 스레드만 돌게 합니다(GIL). Rust는 실행하지 않은 경로까지 컴파일할 때 검사합니다. |
| [Q8.5](#q85-go의-고루틴-같은-가벼운-스레드가-있나요) | Go의 고루틴 같은 가벼운 스레드가 있나요? | 표준 라이브러리의 스레드는 운영체제 스레드입니다. 가벼운 실행 단위는 비동기 런타임의 태스크가 맡습니다. |
| [Q8.6](#q86-비동기-런타임이-왜-표준-라이브러리에-없나요-tokio는-무엇인가요) | 비동기 런타임이 왜 표준 라이브러리에 없나요? tokio는 무엇인가요? | 쓰임새마다 알맞은 런타임이 달라서 고르게 했습니다. tokio는 가장 널리 쓰이는 비동기 런타임 크레이트입니다. |
| [Q8.7](#q87-rust의-async는-javascript의-asyncpromise와-무엇이-다른가요) | Rust의 `async`는 JavaScript의 `async`/Promise와 무엇이 다른가요? | Rust의 퓨처는 게을러서 `.await`하거나 런타임에 넘기기 전에는 본문을 한 줄도 실행하지 않습니다. |
| [Q8.8](#q88-일반-함수에서-async-함수를-그냥-부를-수-없는-이유는-무엇인가요) | 일반 함수에서 `async` 함수를 그냥 부를 수 없는 이유는 무엇인가요? | 부르면 결과가 아니라 퓨처를 받고, `.await`는 `async` 안에서만 쓸 수 있기 때문입니다. 일반 함수에서는 런타임에 퓨처를 넘깁니다. |
| [Q8.9](#q89-스레드와-async-중-무엇을-고르나요) | 스레드와 `async` 중 무엇을 고르나요? | 계산이 많은 일은 스레드, 기다림이 많은 일은 `async`가 기본이고, 둘을 함께 쓰는 경우도 많습니다. |
| [Q8.10](#q810-전역-가변-상태나-싱글턴은-어떻게-만드나요) | 전역 가변 상태나 싱글턴은 어떻게 만드나요? | `static`에 원자 타입(atomic type)이나 `Mutex`를 두고, 실행 중에 만드는 값은 `OnceLock`·`LazyLock`으로 한 번만 초기화합니다. `static mut`는 피합니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md): 스레드, `move` 클로저, 채널, `Mutex<T>`(2.7절), `Arc<T>`, `Send`와 `Sync`(2.9절), 컴파일러가 막아 주지 못하는 것(2.10절)
- [Part 0 ch16](../../part0-getting-started/ch16-async/README.md): 퓨처와 `async`·`.await`, 게으른 퓨처(2.3절), 런타임과 `block_on`(2.4절), 태스크, 퓨처·태스크·스레드(2.12절)
- [ch3](../ch3-safety-limits/README.md) Q3.5(교착 상태와 경쟁 조건)

스코프 스레드(scoped thread)·원자 타입·`OnceLock`은 [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md), 퓨처가 실행되는 원리는 [Part 1 ch13](../../part1-standard-lib/ch13-async-primitives/README.md), 잠금을 설계하는 지침은 [부록 A ch17](../../appd-a/ch17-shared-state-parallelism/README.md)에 있습니다. 이 부록의 선수 과정은 아니므로 "더 읽기"로만 가리킵니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. "겁 없는 동시성"이 약속하는 것(데이터 경쟁이 없음)과 약속하지 않는 것(경쟁 조건, 교착 상태, 틀린 계산)을 구별할 수 있다.
2. `Mutex`가 데이터를 소유하는 설계를 C의 `pthread_mutex_t`, C++의 `std::mutex`, Java의 `synchronized`, Go의 `sync.Mutex`와 비교하고, ThreadSanitizer·Go 경쟁 검출기(race detector)·Python GIL이 하는 일과 구별할 수 있다.
3. Rust의 스레드·태스크를 Go의 고루틴과, Rust의 퓨처를 JavaScript의 Promise와 비교하고, 비동기 런타임이 표준 라이브러리에 없는 이유를 설명할 수 있다.
4. 일반 함수와 `async` 함수 사이를 잇는 방법, 스레드와 `async`를 고르는 기준, 전역 상태를 만드는 방법을 설명할 수 있다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다.

### Q8.1 "겁 없는 동시성"은 무슨 뜻인가요?

**짧은 답** 동시성 버그의 많은 수를 실행 중의 미묘한 버그가 아니라 컴파일 에러로 만든다는 뜻입니다. The Book 16장은 소유권과 타입 검사 덕분에 많은 동시성 에러가 실행 중이 아니라 컴파일 시간에 드러나고, 그래서 제품에 나간 뒤가 아니라 코드를 쓰는 동안 고칠 수 있다는 점을 이렇게 부른다고 설명합니다. 대표가 데이터 경쟁(data race)입니다. 모든 동시성 버그를 막는다는 뜻은 아닙니다.

**왜 그런가** 데이터 경쟁은 두 스레드가 같은 메모리에 동시에 접근하고 그중 하나가 쓰기일 때 생기며, 실행할 때마다 결과가 달라서 재현하고 고치기 어렵습니다. Rust는 메모리 안전성을 위해 이미 갖춘 규칙(소유권, 대여 규칙)을 스레드에도 그대로 적용해 이것을 막습니다(Q8.2). 같은 규칙이 한 스레드 안의 메모리 오류와 여러 스레드 사이의 데이터 경쟁을 함께 막는 셈입니다([ch2](../ch2-memory-safety/README.md)의 데이터 경쟁 안전성).

**다른 언어에서는** Go 문서의 Data Race Detector 글은 데이터 경쟁을 동시성 시스템에서 가장 흔하고 디버그하기 어려운 버그 가운데 하나로 꼽습니다. 다른 언어들은 이것을 경쟁 검출기 같은 실행 중 도구나 규율로 다루고(Q8.4), Rust는 타입 시스템으로 다룹니다.

**오해와 한계** "겁 없음"은 경쟁 조건(race condition)과 교착 상태(deadlock)까지 막는다는 뜻이 아닙니다. 모든 접근이 `Mutex` 안에서 일어나도 "읽고-더하고-쓰기"를 잠금 두 번으로 나누면 결과가 틀리고, 잠금 순서가 엇갈리면 멈춥니다(Part 0 ch15 2.10절, ch3 Q3.5). 데이터 경쟁이 없어도 계산이 틀릴 수 있다는 예는 Q8.2의 예제 01입니다.

**더 읽기** Part 0 ch15 2.1·2.10절 / ch3 Q3.5 / ch2

### Q8.2 데이터 경쟁을 컴파일 시간에 어떻게 막나요?

**짧은 답** 두 가지 장치로 막습니다. 첫째, 대여 규칙이 "가변 참조는 하나뿐이거나, 불변 참조만 여럿"을 강제하므로 두 스레드가 같은 값을 동시에 고칠 수 없습니다. 둘째, 스레드로 보내도 되는 타입은 `Send`, 여러 스레드가 함께 참조해도 되는 타입은 `Sync`라는 마커 트레이트(marker trait)로 표시됩니다. `thread::spawn`은 넘기는 클로저가 `Send`이기를 요구하고, `&T`는 `T: Sync`일 때만 `Send`입니다. 그래서 `Rc`를 다른 스레드로 보내거나(`Send`가 아님) `RefCell`을 여러 스레드가 함께 참조하면(`Sync`가 아님) 컴파일 에러가 납니다(Part 0 ch15 4.4·4.5절).

**왜 그런가** 스레드 둘이 같은 지역 변수를 가변으로 빌려 고치려 하면 E0499로 막힙니다(Part 1 ch11 4.1절). C로 같은 구조를 쓰면 컴파일은 되고, ThreadSanitizer를 켜고 실행해야 경쟁이 보고됩니다([compare/01_data_race_counter.c](compare/01_data_race_counter.c), 3절). 함께 고쳐야 하는 값은 `Mutex`나 원자 타입처럼 `Sync`인 타입에 담아 불변 참조로 함께 씁니다.

그러나 데이터 경쟁이 없다는 것이 계산이 맞다는 뜻은 아닙니다. [01_race_free_not_bug_free](examples/01_race_free_not_bug_free.rs)는 단어 수를 스레드 셋이 나눠 세어 더합니다.

```text
1. move 클로저로 고친 합계
  합계: 0
2. 원자 타입을 빌려 함께 더한 합계
  합계: 7
```

1번은 `move` 클로저가 합계 `total`을 가져가는데, `usize`는 `Copy`라서 스레드마다 사본이 생기고 각자 제 사본만 고칩니다. 원래의 `total`은 아무도 고치지 않아 0으로 남습니다. 데이터 경쟁은 없으므로 컴파일되고, 컴파일러는 ``value captured by `total` is never read``와 ``value assigned to `total` is never read`` 두 경고(`unused_assignments`)로만 알려 줍니다. 예제는 경고 없이 빌드되도록 `#[expect(unused_assignments, …)]`로 이 경고가 난다는 것을 적어 두었습니다. 속성을 지우고 빌드하면 두 경고가 보입니다. 2번은 원자 타입 하나를 모든 스레드가 빌려 함께 고칩니다.

**다른 언어에서는** Java 언어 명세는 happens-before 관계로 순서가 정해지지 않은 두 충돌 접근이 있으면 그 프로그램에 데이터 경쟁이 있다고 정의하며(17.4.5), 그런 프로그램도 컴파일하고 실행합니다. Go 메모리 모델(memory model) 문서도 여러 고루틴이 동시에 접근하는 데이터를 고치는 프로그램은 그 접근을 채널이나 `sync`·`sync/atomic`의 도구로 직렬화해야 한다고 적습니다. 두 언어 모두 데이터 경쟁을 막는 일은 프로그래머의 몫이고, Rust는 그 규칙을 컴파일러가 검사합니다.

**오해와 한계** 1번처럼 사본을 고치는 실수, 경쟁 조건, 교착 상태는 막지 못합니다. 또 `unsafe` 코드에서 원시 포인터로 같은 메모리를 고치거나 `unsafe impl Send`를 잘못 붙이면 이 보장이 깨질 수 있으며, 그 책임은 `unsafe`를 쓴 사람에게 있습니다(ch3). `Copy` 값의 사본을 고치는 함정은 한 스레드 안에서도 생기는데, 부록 A ch10 2.3절이 구조체 필드로 같은 일을 보입니다.

**더 읽기** Part 0 ch15 2.4·2.9절, 4.4·4.5절 / Part 1 ch11 2.2·2.8절, 4.1절 / [부록 A ch10](../../appd-a/ch10-standard-traits/README.md) 2.3절

### Q8.3 `Mutex`가 왜 데이터를 감싸나요? (C++의 `std::mutex`, Java의 `synchronized`, Go의 `sync.Mutex`와 비교)

**짧은 답** 잠그지 않고는 데이터에 닿을 수 없게 하려는 것입니다. Rust의 `Mutex<T>`는 보호할 값 `T`를 소유하고, 안의 값에는 `lock()`이 돌려주는 가드(guard, `MutexGuard`)를 거쳐야만 닿습니다. 가드가 버려지면 잠금이 풀리므로 푸는 것도 빠뜨릴 수 없습니다([ch5](../ch5-no-null-no-exceptions/README.md) Q5.6). 다른 언어의 잠금은 보호할 데이터와 따로 있어서, 잠그는 것을 빠뜨려도 컴파일됩니다.

**왜 그런가** "이 데이터는 이 잠금으로 보호한다"는 약속을 주석이 아니라 타입으로 적은 것입니다. `Mutex<u64>`에 든 값을 그냥 `+= 1`로 고치려 하면 `Mutex<u64>`에는 그런 연산이 없다는 E0368이 납니다(4.1절). 여러 필드가 함께 바뀌어야 하면 그 필드들을 구조체 하나로 묶어 `Mutex` 하나에 넣으므로, 잠금 하나가 무엇을 지키는지도 분명해집니다(부록 A ch17).

**다른 언어에서는**

| 언어 | 잠금 | 데이터와의 관계 |
| --- | --- | --- |
| C | `pthread_mutex_t` | 별개의 변수. compare/01은 한 스레드만 잠그고 다른 스레드는 빠뜨려도 컴파일됨 |
| C++ | `std::mutex` | 표준 초안의 `mutex` 클래스는 잠금 연산이 `lock`·`try_lock`·`unlock`뿐이고 보호할 데이터를 담지 않음 |
| Java | `synchronized` | 모든 객체에 딸린 모니터를 잠금(Java 언어 명세 17.1). 다른 코드가 같은 필드를 `synchronized` 없이 읽고 써도 컴파일됨 |
| Go | `sync.Mutex` | 데이터와 별개의 값. `Lock`은 잠금만 얻음(`sync` 문서) |
| Rust | `Mutex<T>` | 데이터를 소유. 가드를 거쳐야만 접근 |

C·C++에도 잠그지 않은 접근을 컴파일 시간에 찾는 도구가 있습니다. Clang의 스레드 안전성 분석(Thread Safety Analysis)은 문서에서 C++ 언어 확장으로 소개하지만 C에서도 동작하며, 데이터에 `guarded_by` 같은 속성을 달고 `-Wthread-safety`로 컴파일하면 잠그지 않은 쓰기를 경고합니다. 이 환경의 Apple clang 21.0.0에서 compare/01과 같은 카운터에 속성을 달아 보니 ``writing variable 'counter' requires holding mutex 'counter_lock' exclusively``라는 경고가 났습니다. 다만 속성을 직접 달아야 켜지는 선택 기능이고 결과는 에러가 아니라 경고입니다.

**오해와 한계** `Mutex`가 데이터를 감싸도 잠금을 너무 오래 쥐거나, 잠금 둘을 엇갈린 순서로 잡아 교착 상태에 빠지는 일은 막지 못합니다(부록 A ch17). 또 다른 스레드가 잠금을 쥔 채 패닉하면 그 `Mutex`는 포이즈닝(poisoning)되어 `lock()`이 `Err`를 돌려주므로 처리할 일이 하나 늘어납니다(Part 0 ch15 2.7절).

**더 읽기** Part 0 ch15 2.7·2.8절 / Part 1 ch11 2.4절 / 부록 A ch17

### Q8.4 Python의 GIL, Go의 race detector, C/C++의 ThreadSanitizer와 무엇이 다른가요?

**짧은 답** Go의 경쟁 검출기와 C/C++의 ThreadSanitizer(TSan)는 프로그램을 실행하면서 실제로 일어난 데이터 경쟁을 찾는 검출기입니다. Python의 GIL(global interpreter lock)은 CPython이 한 번에 한 스레드만 바이트코드를 실행하게 하는 장치입니다. Rust는 실행하지 않고 컴파일할 때 모든 경로를 검사해, 데이터 경쟁이 생길 수 있는 코드를 처음부터 받아들이지 않습니다.

**왜 그런가** 검출기는 실행된 경로만 봅니다. Go 문서는 경쟁 검출기가 실행 중에 일어난 경쟁만 찾으므로 실행되지 않은 코드 경로의 경쟁은 찾지 못한다고 적습니다. compare/01을 TSan으로 실행하면 이 환경에서는 세 번 모두 경쟁이 보고됐지만, 그것은 경쟁이 실제로 일어난 실행이기 때문입니다. Rust의 검사는 타입 검사라서 실행 여부와 상관없고, 실행 중 비용도 없습니다.

**다른 언어에서는**

| 도구 | 언제 | 무엇을 | 비용(각 문서) |
| --- | --- | --- | --- |
| Go 경쟁 검출기 | 실행 중(`-race`로 빌드) | 실행된 경로에서 일어난 데이터 경쟁 | 보통의 프로그램에서 메모리가 5~10배, 실행 시간이 2~20배 늘 수 있음 |
| ThreadSanitizer | 실행 중(`-fsanitize=thread`로 빌드) | 실행된 경로에서 일어난 데이터 경쟁 | 보통 5~15배 느려지고 메모리 5~10배 |
| Python GIL | GIL이 있는 빌드에서는 실행 중 항상 | 한 번에 한 스레드만 바이트코드 실행 | 여러 코어를 쓰는 병렬성(parallelism)을 상당 부분 잃음(Python 용어집) |
| Rust | 컴파일 시간 | 데이터 경쟁이 생길 수 있는 코드 | 실행 중 비용 없음. 대신 표현할 수 없는 코드가 생김 |

Python 용어집은 GIL이 `dict` 같은 내장 타입을 동시 접근에 대해 암묵적으로 안전하게 만들어 구현을 단순하게 하지만, 여러 프로세서가 주는 병렬성의 상당 부분을 희생한다고 설명하고, Python 3.13부터는 `--disable-gil` 빌드 설정으로 GIL을 끌 수 있다고 적습니다(PEP 703).

**오해와 한계** 컴파일 시간 검사는 "확실히 안전하다고 증명할 수 있는 코드"만 받아들이므로, 실제로는 경쟁이 없는데도 거부되는 코드가 있습니다. 그럴 때는 스코프 스레드(Part 1 ch11 2.2절)나 `Arc`·`Mutex`로 구조를 바꾸거나, 마지막 수단으로 `unsafe`를 씁니다. 또 Rust 프로그램이 부르는 C 라이브러리 안의 경쟁은 Rust의 검사 밖이므로 TSan 같은 도구가 여전히 쓸모 있습니다.

**더 읽기** 3절의 compare/01 / Part 1 ch11 2.2절 / [ch15](../ch15-pain-points/README.md)

### Q8.5 Go의 고루틴 같은 가벼운 스레드가 있나요?

**짧은 답** 표준 라이브러리의 `std::thread`는 운영체제 스레드입니다. 표준 라이브러리 문서는 실행 중인 Rust 프로그램을 각자 스택과 지역 상태를 가진 운영체제의 네이티브 스레드들의 모음으로 설명합니다. 고루틴처럼 가벼운 실행 단위는 비동기 런타임의 태스크(task)가 맡습니다. 런타임이 많은 태스크를 적은 수의 운영체제 스레드에 나눠 실행합니다.

**왜 그런가** 운영체제 스레드는 스레드마다 스택 메모리를 따로 잡으므로 수만 개를 만들기 부담스럽습니다. 태스크는 런타임이 관리하는 퓨처라서 훨씬 가볍고, Part 0 ch16 예제 08은 태스크 만 개를 만들어 봅니다. 다만 태스크는 `.await` 지점에서만 다른 태스크에 차례를 넘기므로, 오래 걸리는 계산을 `.await` 없이 돌리면 같은 작업자 스레드(worker thread)의 다른 태스크가 기다립니다(Part 0 ch16 2.9절).

**다른 언어에서는** Go FAQ는 고루틴을 독립적으로 실행되는 함수(코루틴, coroutine)를 스레드 집합에 나눠 싣는 방식으로 설명합니다. 고루틴 하나가 블로킹(blocking) 시스템 호출에서 멈추면 런타임이 같은 운영체제 스레드의 다른 고루틴을 다른 스레드로 옮기고, 프로그래머는 이것을 보지 못한다고 적습니다. 또 고루틴의 스택은 몇 킬로바이트로 시작해 필요하면 런타임이 늘리고 줄인다고 설명합니다. Rust의 태스크는 비슷한 자리에 있지만, 언어가 아니라 고른 런타임이 제공하고, 차례를 넘기는 지점이 `.await`로 코드에 보인다는 점이 다릅니다.

**오해와 한계** Go에서는 모든 코드가 고루틴 위에서 돌아 블로킹 호출도 런타임이 처리해 주지만, Rust의 `async` 코드 안에서 블로킹 함수(파일 읽기, `thread::sleep`, 무거운 계산)를 그대로 부르면 그 작업자 스레드가 멈춥니다. 그런 일은 런타임이 제공하는 블로킹 전용 스레드나 별도 스레드로 보냅니다.

**더 읽기** Part 0 ch16 2.6·2.9·2.12절, 예제 08

### Q8.6 비동기 런타임이 왜 표준 라이브러리에 없나요? tokio는 무엇인가요?

**짧은 답** 쓰임새마다 알맞은 런타임이 다르기 때문입니다. The Book 17.1절은 비동기를 지원하는 언어 대부분이 런타임을 함께 넣지만 Rust는 그러지 않았고, 서로 다른 장단점을 고른 여러 런타임이 있다고 설명합니다. 많은 코어와 메모리를 가진 웹 서버와, 코어 하나에 메모리가 적고 힙 할당도 못 하는 마이크로컨트롤러는 필요한 것이 다르다는 예를 듭니다. tokio는 그런 런타임 가운데 하나로, The Book은 특히 웹 애플리케이션에서 가장 널리 쓰이는 비동기 런타임이라고 소개합니다.

**왜 그런가** 언어와 표준 라이브러리는 `async`/`.await` 문법과 `Future` 트레이트처럼 런타임들이 함께 쓸 바탕만 둡니다. 퓨처를 언제 다시 폴링(polling)할지 정하고, 타이머와 네트워크 이벤트를 기다리고, 태스크를 스레드에 나눠 싣는 일은 런타임의 몫입니다. 그 일을 직접 만들어 보는 것이 Part 1 ch13이고, Part 0 ch16이 쓴 `trpl`은 안에서 tokio를 씁니다.

**다른 언어에서는** MDN의 실행 모델 문서에 따르면 JavaScript는 엔진과 호스트 환경이 함께 실행을 맡고, 비동기 프로그래밍을 가능하게 하는 작업 큐(job queue, HTML에서는 이벤트 루프(event loop)라고 부름)가 그 실행 모델에 들어 있습니다. Go는 고루틴을 스레드에 나눠 싣는 일을 언어 런타임이 합니다(Q8.5). 두 언어 모두 프로그램이 런타임을 따로 고를 일이 없습니다.

**오해와 한계** 런타임을 고를 수 있다는 것은 고르는 수고가 있다는 뜻이기도 합니다. 라이브러리가 특정 런타임(예: tokio)의 타이머나 네트워크 타입을 쓰면 그 런타임 위에서만 돌기도 해서, 생태계가 사실상 한 런타임에 모이는 경향이 생깁니다. 처음에는 가장 많이 쓰이는 런타임을 고르는 것이 무난합니다.

**더 읽기** Part 0 ch16 2.4·2.13절 / Part 1 ch13 2.5·2.10절

### Q8.7 Rust의 `async`는 JavaScript의 `async`/Promise와 무엇이 다른가요?

**짧은 답** 문법은 비슷하지만 실행 시점이 다릅니다. JavaScript의 `async` 함수는 부르는 즉시 첫 `await`까지 실행되고 Promise를 돌려줍니다. Rust의 `async` 함수를 부르면 본문을 한 줄도 실행하지 않은 퓨처(future)를 돌려주고, 그 퓨처를 `.await`하거나 런타임에 넘겨야 그제야 실행됩니다. The Book도 Rust의 퓨처를 게으르다(lazy)고 설명합니다.

**왜 그런가** 퓨처가 게으르면 만들기만 하고 쓰지 않은 작업은 아무 비용도 들지 않고, 런타임이 언제 실행할지 정할 수 있습니다. 대신 `.await`를 빠뜨리면 그 작업은 끝내 실행되지 않으므로, 컴파일러는 버려진 퓨처에 ``unused implementer of `Future` that must be used`` 경고를 내고, ``futures do nothing unless you `.await` or poll them``이라고 덧붙입니다. 퓨처가 게으르다는 것은 Part 0 ch16 예제 01이 출력으로 보였습니다.

**다른 언어에서는** MDN은 JavaScript의 `async` 함수 본문이 `await`로 나뉘며, 첫 `await`까지의 코드는 동기적으로 실행되고 `await`가 없는 `async` 함수는 끝까지 동기적으로 실행된다고 설명합니다. 그래서 JavaScript에서는 `fetchData()`만 부르고 `await`하지 않아도 요청이 시작되지만, Rust에서는 시작되지 않습니다.

**오해와 한계** JavaScript처럼 "부르면 바로 시작"하고 결과는 나중에 받고 싶으면, 런타임의 태스크로 만들어(`spawn`) 따로 돌게 합니다(Part 0 ch16 2.6절). 또 Rust의 `async` 함수는 결과 타입을 그대로 쓸 수 없어서, 일반 함수에서 결과를 바로 쓰려 하면 4.2절의 에러가 납니다(Q8.8).

**더 읽기** Part 0 ch16 2.2·2.3절, 예제 01, 4.5절(버려진 퓨처의 경고) / Part 1 ch13 2.2절

### Q8.8 일반 함수에서 `async` 함수를 그냥 부를 수 없는 이유는 무엇인가요?

**짧은 답** 부르는 것 자체는 됩니다. 다만 결과가 아니라 퓨처를 받으므로 그 값을 결과처럼 쓸 수 없고(4.2절), 퓨처를 기다리는 `.await`는 `async` 함수나 블록 안에서만 쓸 수 있습니다(E0728). 그래서 일반 함수에서는 런타임의 `block_on` 같은 함수에 퓨처를 넘겨, 그 스레드를 막고 결과를 기다립니다. `main`이 `async`일 수 없고 런타임을 시작하는 자리인 것도 같은 이유입니다(Part 0 ch16 2.4절).

**왜 그런가** `.await`는 그 자리에서 함수를 멈췄다가 나중에 이어 가야 하는데, 그럴 수 있는 것은 퓨처(상태 기계)로 컴파일되는 `async` 함수와 블록뿐입니다. 일반 함수는 런타임 안에서 불려도 멈출 수 없으므로 `.await`를 쓰지 못하고(Part 0 ch16 4.1절), 결과가 필요하면 `block_on`처럼 스레드를 막고 기다리는 함수에 퓨처를 넘깁니다. `block_on`이 런타임을 세우고 퓨처를 끝까지 돌립니다. 4.2절의 컴파일러는 `.await`를 붙이라고 제안하지만, 일반 함수인 `main`에서 그대로 따르면 E0728이 납니다(Part 0 ch16 4.1절).

**다른 언어에서는** MDN에 따르면 JavaScript의 `await`도 `async` 함수 안이나 모듈의 최상위에서만 쓸 수 있어서, 같은 종류의 제약이 있습니다. 다른 점은 JavaScript의 `async` 함수는 부르기만 해도 실행되고(Q8.7) 이벤트 루프가 실행 모델에 들어 있다는 것입니다(Q8.6). Rust에서는 그 자리를 맡을 런타임을 누군가 직접 세워야 하므로, 일반 함수와 `async` 함수 사이의 다리(`block_on`)가 코드에 드러납니다.

**오해와 한계** `.await`를 쓰려면 부르는 쪽도 `async`여야 해서 `async`가 호출 사슬을 따라 위로 번지는 것(함수의 "색"이 나뉘는 문제, function coloring)이 비용입니다. 또 `async` 함수 안에서 다시 `block_on`을 부르면 안 됩니다. 이 환경에서 `trpl::block_on` 안의 `async` 함수가 다시 `trpl::block_on`을 부르게 해 보니(trpl 0.3.0, 안의 tokio 1.54.1), tokio가 "Cannot start a runtime from within a runtime"이라는 메시지와 함께 패닉했습니다. `async` 함수 안에서는 `block_on` 대신 `.await`를 씁니다.

**더 읽기** Part 0 ch16 2.4절, 4.1절 / Part 1 ch13 2.5절(최소 실행기 `block_on`)

### Q8.9 스레드와 `async` 중 무엇을 고르나요?

**짧은 답** The Book 17.6절의 기준을 따르면, 데이터를 나눠 따로 처리하는 것처럼 계산이 많은(CPU 중심) 일은 스레드가, 여러 곳에서 언제 올지 모르는 메시지를 다루는 것처럼 기다림이 많은(I/O 중심) 일은 `async`가 낫습니다. 둘 다 필요하면 함께 씁니다. The Book도 많은 경우 선택지는 "스레드 또는 `async`"가 아니라 "스레드와 `async`"라고 적습니다.

**왜 그런가** 스레드는 운영체제가 코어에 나눠 실행하므로 계산을 병렬로 돌리기 좋지만 하나하나가 무겁습니다. `async` 태스크는 가벼워 수만 개의 연결을 기다리기 좋지만, `.await` 없이 계산을 오래 하면 다른 태스크를 막습니다(Q8.5). Part 0 ch16 예제 08은 무거운 계산은 스레드에 맡기고 결과를 비동기 채널로 받는 방법을 보입니다.

**다른 언어에서는** Go는 고루틴 하나로 두 경우를 모두 다루고 런타임이 블로킹을 처리해 줍니다(Q8.5). 브라우저의 JavaScript는 무거운 계산을 Web Worker로 메인 실행 스레드와 다른 백그라운드 스레드에서 돌립니다(MDN). Rust는 두 도구를 따로 두고 고르게 합니다.

**오해와 한계** 서버라면 처음부터 `async` 런타임을 쓰는 경우가 많지만, 동시 연결이 적은 도구나 일괄 처리 프로그램은 스레드(또는 스코프 스레드)만으로 더 단순하게 쓸 수 있습니다. 비동기는 런타임 선택(Q8.6)과 함수 색 문제(Q8.8)라는 비용을 함께 가져옵니다.

**더 읽기** Part 0 ch16 2.12절, 예제 08 / Part 1 ch11

### Q8.10 전역 가변 상태나 싱글턴은 어떻게 만드나요?

**짧은 답** `static`에 스레드 안전한 타입을 둡니다. 싱글턴(singleton)도 같은 방법으로 만듭니다. 정수나 `bool` 하나면 원자 타입(`static VISITS: AtomicU64`), 그보다 큰 값이면 `static`의 `Mutex`(`Mutex::new`는 `const fn`), 실행 중에 만들어야 하는 값(설정, 표)은 `OnceLock`이나 `LazyLock`으로 처음 쓸 때 한 번만 초기화합니다. `static mut`는 읽고 쓸 때마다 `unsafe`가 필요하고, 2024 에디션은 그 참조를 만드는 코드를 기본으로 거부합니다(`static_mut_refs`).

**왜 그런가** `static`은 모든 스레드가 함께 보는 값이므로 `Sync`여야 합니다. 그래서 "전역 가변 상태"도 데이터 경쟁을 막는 같은 규칙(Q8.2)을 따릅니다. 여러 스레드가 동시에 처음 접근해도 `OnceLock`·`LazyLock`의 초기화는 한 번만 일어납니다(Part 1 ch11 예제 08). `static`의 초기값에서 `HashMap::new()`처럼 `const`가 아닌 함수를 부르면 E0015로 막히는 것도 같은 맥락입니다(Part 1 ch11 4.4절).

**다른 언어에서는** Java의 `static` 필드나 싱글턴 객체는 여러 스레드가 함께 고쳐도 컴파일되며, 동기화는 프로그래머가 맞춥니다(Q8.3). Go의 패키지 수준 변수도 마찬가지입니다. Rust에서 전역은 `Sync` 타입이어야 하므로 동기화 방법을 타입으로 고르게 됩니다.

**오해와 한계** 전역 상태는 Rust에서도 테스트와 추론을 어렵게 하므로, 가능하면 값을 인자로 넘기거나 `Arc`로 공유하는 편이 낫습니다. `static mut`를 2024 에디션으로 옮길 때 생기는 에러와 대안은 [Part 2 ch6](../../part2-editions/ch6-migration-practice/README.md) 4.2절에 있습니다.

**더 읽기** Part 1 ch11 2.8·2.9절, 예제 07·08 / [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.9절, Part 2 ch6 4.2절 / [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.5절

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_race_free_not_bug_free](examples/01_race_free_not_bug_free.rs) | `move` 클로저가 합계의 사본만 고쳐 0이 남는 코드(데이터 경쟁은 없음)와, 원자 타입을 빌려 함께 더한 코드 | Q8.1, Q8.2 |
| [compare/01_data_race_counter.c](compare/01_data_race_counter.c) | 한 스레드가 잠금을 빠뜨린 C 코드와 ThreadSanitizer의 보고 | Q8.2, Q8.3, Q8.4 |

이 챕터의 다른 질문은 Part 0 ch15·ch16, Part 1 ch11에 이미 있는 예제로 답하므로, 해당 소절의 "더 읽기"를 따라가면 됩니다.

### 01_race_free_not_bug_free — 데이터 경쟁이 없어도 틀릴 수 있음

```sh
cargo run -p faq-ch8-concurrency-async --example 01_race_free_not_bug_free
```

```text
1. move 클로저로 고친 합계
  합계: 0
2. 원자 타입을 빌려 함께 더한 합계
  합계: 7
```

- 세 줄의 단어 수는 2, 1, 4개로 합계가 7입니다. 스레드는 순서와 상관없이 더하므로 출력은 실행마다 같습니다.
- 2번의 클로저는 `move`가 없어도 컴파일됩니다. `total`은 스코프 밖에 있어 스레드보다 오래 살고, `line`은 `&str`이라 클로저가 `line` 변수가 아니라 그것이 가리키는 문자열을 빌리기 때문입니다. 반대로 반복문 안에서 만든 값 변수(`let n = count_words(line);`)를 `move` 없이 쓰면, 그 변수가 반복 한 번이 끝날 때 사라지므로 E0373이 납니다.
- 원자 타입의 `Ordering::Relaxed`는 순서 보장이 가장 약한 방식으로, 합계처럼 값 하나만 세는 데 충분합니다(Part 1 ch11 2.8절).

### compare/01_data_race_counter.c — C의 데이터 경쟁과 ThreadSanitizer

저장소 루트에서 실행합니다. `cc`의 `-o`는 출력 디렉터리를 만들지 않으므로 먼저 만듭니다.

```sh
mkdir -p target/tmp/faq-ch8-concurrency-async
cc -g -fsanitize=thread -o target/tmp/faq-ch8-concurrency-async/data_race appd-b/ch8-concurrency-async/compare/01_data_race_counter.c
target/tmp/faq-ch8-concurrency-async/data_race
```

ThreadSanitizer가 보고하는 오류 종류는 이 줄입니다(뒤에 붙는 프로세스 번호와 그 아래의 주소·스택 추적은 생략).

```text
WARNING: ThreadSanitizer: data race
```

- 두 스레드가 전역 카운터를 10만 번씩 올리는데, `add_locked`는 잠그고 `add_forgot_lock`은 잠그지 않습니다. 잠금이 데이터와 따로 있어서 C 컴파일러는 이것을 막지 않습니다(Q8.3).
- 마지막 합계 줄(`counter = …`)은 실행마다 달라질 수 있어 싣지 않습니다. 결과가 정해지지 않는다는 것 자체가 데이터 경쟁의 문제입니다.
- TSan은 보고한 뒤에도 프로그램을 끝까지 실행하므로(`halt_on_error`가 `false`) 합계 줄도 출력되고, 이 환경에서는 프로그램이 끝날 때 `abort()`하도록 설정되어 있어(`TSAN_OPTIONS=help=1`의 `abort_on_error`가 `true`) 종료 상태가 134입니다. `TSAN_OPTIONS=abort_on_error=0`이면 `exitcode`의 기본값인 66으로 끝납니다. 다른 운영체제나 설정에서는 다를 수 있습니다.
- 같은 구조를 Rust로 쓰면 컴파일되지 않습니다. 두 스레드가 지역 변수를 가변으로 빌리면 E0499(Part 1 ch11 4.1절), `Mutex`에 든 값을 잠그지 않고 고치면 E0368(4.1절)입니다.

## 4. 자주 하는 실수

### 4.1 `Mutex`에 든 값을 잠그지 않고 고침 — E0368

파일: [`compile_fail/e0368_mutex_used_as_value.rs`](compile_fail/e0368_mutex_used_as_value.rs)

```text
error[E0368]: binary assignment operation `+=` cannot be applied to type `std::sync::Mutex<u64>`
  --> compile_fail/e0368_mutex_used_as_value.rs:21:13
   |
21 |             visits += 1; // E0368
   |             ------^^^^^
   |             |
   |             cannot use `+=` on type `std::sync::Mutex<u64>`
   |
note: `std::sync::Mutex<u64>` does not implement `AddAssign<{integer}>`
```

- **원인**: Go의 `mu`와 `count`, Java의 `synchronized`와 필드처럼 잠금과 데이터를 따로 두는 습관대로 `Mutex`에 든 값을 바로 고치려 했습니다. `Mutex<u64>`는 `u64`가 아니라 `u64`를 담은 상자라서 `+=` 같은 연산이 없습니다.
- **해결**: `*visits.lock().unwrap() += 1;`처럼 잠근 뒤 가드를 거쳐 고칩니다. 값 하나를 세는 것이 전부라면 `AtomicU64`의 `fetch_add`가 더 가볍습니다(01 예제의 2번).

### 4.2 `async` 함수의 결과를 일반 함수에서 바로 씀 — E0599

파일: [`compile_fail/e0599_future_used_as_value.rs`](compile_fail/e0599_future_used_as_value.rs)

```text
error[E0599]: no method named `len` found for opaque type `impl Future<Output = String>` in the current scope
  --> compile_fail/e0599_future_used_as_value.rs:22:33
   |
22 |     println!("제목 길이: {}", title.len()); // E0599
   |                                     ^^^ method not found in `impl Future<Output = String>`
   |
help: consider `await`ing on the `Future` and calling the method on its `Output`
   |
22 |     println!("제목 길이: {}", title.await.len()); // E0599
   |                                     ++++++
```

- **원인**: JavaScript에서 `async` 함수가 Promise를 돌려주듯, Rust의 `async fn fetch_title`도 `String`이 아니라 퓨처(`impl Future<Output = String>`)를 돌려줍니다. 그 퓨처는 아직 본문을 실행하지도 않았습니다(Q8.7).
- **해결**: 컴파일러는 `.await`를 제안하지만, `main`은 `async`가 아니므로 그대로 따르면 E0728입니다(Part 0 ch16 4.1절). 일반 함수에서는 런타임에 퓨처를 넘겨 결과를 받습니다(`trpl::block_on(fetch_title(1))`, Part 0 ch16 2.4절). 이미 `async` 함수 안이라면 `.await`를 붙입니다.

## 5. 연습 문제

### 연습 1. `move`를 지우면

01의 `total_with_move`에서 클로저의 `move`를 지우고 컴파일해 보세요. 어떤 에러가 나는지 예상한 뒤 확인하고, `Mutex<usize>`를 써서 합계 7이 나오게 고쳐 봅니다.

<details>
<summary>힌트</summary>

`move`를 지우면 세 스레드가 같은 `total`을 동시에 가변으로 빌리므로 E0499(``cannot borrow `total` as mutable more than once at a time``)가 납니다. `let total = Mutex::new(0);`으로 바꾸고 클로저 안에서 `*total.lock().unwrap() += count_words(line);`로 고친 뒤, 마지막에 `total.into_inner().unwrap()`을 돌려줍니다. 이제 `unused_assignments` 경고가 나지 않으므로 `#[expect(...)]`도 지워야 합니다(남겨 두면 충족되지 않은 기대 경고가 납니다).

</details>

### 연습 2. 잠금을 채워 넣기

compare/01의 `add_forgot_lock`에도 `add_locked`처럼 잠금을 넣고, 3절의 명령(먼저 `mkdir -p`)으로 ThreadSanitizer를 켜고 실행해 보세요. 보고가 사라지고 합계가 기대값과 같아지는지 확인합니다.

<details>
<summary>힌트</summary>

`counter++;` 앞뒤에 `pthread_mutex_lock(&counter_lock);`과 `pthread_mutex_unlock(&counter_lock);`을 넣습니다. 이 환경에서는 TSan 보고 없이 `counter = 200000 (기대값 200000)`을 출력하고 종료 상태 0으로 끝났습니다. C에서는 이 두 줄을 넣는 것을 기억해야 하지만, Rust의 `Mutex<T>`는 잠그지 않으면 데이터에 닿을 수 없습니다(Q8.3).

</details>

### 연습 3. 전역 방문 수

`static VISITS: AtomicU64 = AtomicU64::new(0);`과 `fn visit()`을 만들어, 스코프 스레드 4개가 각각 `visit()`을 250번 부른 뒤 방문 수를 출력해 보세요. `unsafe` 없이 1000이 나와야 합니다.

<details>
<summary>힌트</summary>

`visit` 안에서 `VISITS.fetch_add(1, Ordering::Relaxed);`를 부르고, 마지막에 `VISITS.load(Ordering::Relaxed)`를 출력합니다. `AtomicU64::new`는 `const fn`이라 `static`의 초기값으로 쓸 수 있고, 원자 타입은 `Sync`라서 여러 스레드가 함께 고쳐도 됩니다(Q8.10). 같은 일을 `static mut VISITS: u64`로 하면 읽고 쓸 때마다 `unsafe`가 필요합니다.

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rust Programming Language: [16장 Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html), [16.4절 Send and Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html), [17.1절 Futures and the Async Syntax](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html), [17.6절 Futures, Tasks, and Threads](https://doc.rust-lang.org/book/ch17-06-futures-tasks-threads.html)
- 표준 라이브러리 문서: [`std::thread`](https://doc.rust-lang.org/std/thread/index.html), [`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html), [`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html), [`std::sync::LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html)

다른 언어와 도구

- C++: 표준 초안 [\[thread.mutex.class\]](https://eel.is/c++draft/thread.mutex.class)
- Java: Java Language Specification(SE 21) [17.1 Synchronization](https://docs.oracle.com/javase/specs/jls/se21/html/jls-17.html#jls-17.1), [17.4.5 Happens-before Order](https://docs.oracle.com/javase/specs/jls/se21/html/jls-17.html#jls-17.4.5)
- Go: [Data Race Detector](https://go.dev/doc/articles/race_detector), [The Go Memory Model](https://go.dev/ref/mem), [`sync.Mutex`](https://pkg.go.dev/sync#Mutex), [FAQ — Why goroutines instead of threads?](https://go.dev/doc/faq#goroutines)
- Clang: [ThreadSanitizer](https://clang.llvm.org/docs/ThreadSanitizer.html), [Thread Safety Analysis](https://clang.llvm.org/docs/ThreadSafetyAnalysis.html)
- Python: [용어집 — global interpreter lock](https://docs.python.org/3/glossary.html#term-global-interpreter-lock)
- JavaScript(MDN): [`async function`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function), [JavaScript execution model](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Execution_model), [Web Workers API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API), [`await`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await)
