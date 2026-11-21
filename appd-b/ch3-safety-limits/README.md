# ch3. Rust가 보장하지 않는 것

[ch2](../ch2-memory-safety/README.md)에서 Rust가 막는 것을 봤다면, 이 챕터는 그 반대편을 봅니다. "Rust로 쓰면 안전하다"는 말을 들은 사람이 곧이어 묻는 것은 대개 이렇습니다. `unsafe`가 있는데 결국 C와 같지 않은가, 표준 라이브러리도 `unsafe`를 쓰는데 믿을 수 있는가, Rust 코드에서도 보안 취약점이 나오는가. 이 챕터는 Rust가 **보장하지 않는 것**(논리 오류, 메모리 누수, 교착 상태와 경쟁 조건, 정수 오버플로, 패닉으로 인한 중단)을 예제로 확인하고, `unsafe`의 범위와 Rust 코드의 실제 취약점 사례, 공급망(supply chain)과 컴파일러 버그를 1차 자료로 답합니다.

- 패키지: `faq-ch3-safety-limits`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 자료 확인일: 2026-11-21
- Miri가 없어 예제 05와 `compile_fail/forbid_unsafe_code.rs`의 `unsafe` 블록은 Miri로 검사하지 못했습니다(`compile_fail/forbid_unsafe_code.rs`는 컴파일되지 않는 파일이라 실행할 일도 없습니다). 블록마다 전제 조건을 바로 위 코드가 확인하고 `// SAFETY:` 주석으로 적었습니다.

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q3.1](#q31-rust로-쓰면-버그가-없어지나요) | Rust로 쓰면 버그가 없어지나요? | 아닙니다. 막는 것은 안전한 코드의 메모리 오류와 데이터 경쟁입니다. 논리 오류는 그대로 생깁니다. |
| [Q3.2](#q32-unsafe가-있으면-결국-c와-같지-않나요) | `unsafe`가 있으면 결국 C와 같지 않나요? | 다릅니다. `unsafe`가 푸는 것은 다섯 가지 일뿐이고, 그 코드가 어디 있는지 드러나며, 크레이트 단위로 금지할 수도 있습니다. |
| [Q3.3](#q33-표준-라이브러리도-unsafe를-쓰는데-믿을-수-있나요) | 표준 라이브러리도 `unsafe`를 쓰는데 믿을 수 있나요? | 많이 검증되지만 완벽하지 않습니다. 표준 라이브러리에서 `unsafe`를 쓰는 코드에서도 취약점이 나온 적이 있습니다. |
| [Q3.4](#q34-메모리-누수는-왜-막지-않나요) | 메모리 누수는 왜 막지 않나요? | 누수는 메모리 안전성 위반이 아니고, 모든 누수를 막으려면 쓸모 있는 기능까지 막아야 하기 때문입니다. |
| [Q3.5](#q35-교착-상태와-경쟁-조건은-막아-주나요) | 교착 상태와 경쟁 조건은 막아 주나요? | 막지 않습니다. 데이터 경쟁만 막습니다. |
| [Q3.6](#q36-정수-오버플로는-어떻게-되나요) | 정수 오버플로는 어떻게 되나요? | 디버그 빌드는 패닉, 릴리스 빌드는 기본으로 래핑합니다. 어느 쪽이든 미정의 동작은 아닙니다. |
| [Q3.7](#q37-패닉은-서비스-거부-공격이-될-수-있지-않나요) | 패닉은 서비스 거부 공격이 될 수 있지 않나요? | 될 수 있습니다. 외부 입력으로 생기는 실패는 `Result`로 거부하고 입력의 크기와 깊이를 제한합니다. |
| [Q3.8](#q38-rust로-쓴-코드에서도-cve가-나오나요) | Rust로 쓴 코드에서도 CVE가 나오나요? | 나옵니다. 메모리 오류는 `unsafe`를 쓰는 코드(모듈)에서, 그 밖의 취약점은 안전한 코드에서도 나옵니다. |
| [Q3.9](#q39-의존하는-크레이트가-많은데-공급망은-안전한가요) | 의존하는 크레이트가 많은데 공급망은 안전한가요? | 언어가 보장하는 영역이 아닙니다. crates.io에도 악성 크레이트가 올라온 적이 있습니다. |
| [Q3.10](#q310-컴파일러에-버그가-있으면-보장이-깨지나요) | 컴파일러에 버그가 있으면 보장이 깨지나요? | 깨질 수 있습니다. 알려진 건전성(soundness) 버그는 공개적으로 추적되고 있습니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md) 4절, 예제 04: 정수 오버플로의 디버그 패닉과 릴리스 래핑, 오버플로를 다루는 메서드 4종, 값을 미리 알면 컴파일 에러
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.6·2.10·2.11절: `Drop`, 순환 참조와 메모리 누수, `Weak<T>`
- [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 2.7·2.10절, 예제 05: `Mutex<T>`와 `try_lock`, 컴파일러가 막아 주지 못하는 것(경쟁 조건, 교착 상태)
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.1~2.3절: `unsafe`가 할 수 있는 일, 원시 포인터, 안전한 추상화
- [ch2](../ch2-memory-safety/README.md): 메모리 안전성의 다섯 종류, 패닉의 안전성(Q2.6), 정수 실수를 드러내는 장치(Q2.7)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Rust가 안전하다고 보지만 막지는 않는 것(누수, 교착 상태, 경쟁 조건, 정수 오버플로, 논리 오류)을 들고, 각각이 안전한 코드에서 컴파일되는 것을 예제로 보일 수 있다.
2. `unsafe`가 허용하는 일과 허용하지 않는 일을 구분하고, `unsafe`를 안전한 API 안에 가두는 방법과 크레이트 단위로 금지하는 방법을 설명할 수 있다.
3. 정수 오버플로가 빌드에 따라 어떻게 다르게 동작하는지 설명하고, 뜻을 밝힌 산술 메서드를 고를 수 있다.
4. Rust 코드에서 나온 취약점 사례를 메모리 오류와 그 밖의 오류로 나눠 설명하고, 공급망과 컴파일러 버그라는 언어 밖의 위험을 들 수 있다.

학습 목표 1~3은 예제와 `compile_fail/`이 뒷받침하고, 4는 2절의 1차 자료로 답합니다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다.

### Q3.1 Rust로 쓰면 버그가 없어지나요?

**짧은 답** 아닙니다. Rust가 보장하는 것은 안전한(safe) 코드에 메모리 오류와 데이터 경쟁이 없다는 것까지입니다. 계산이 틀리거나, 검사를 빠뜨리거나, 입력을 믿고 위험한 경로를 만드는 논리 오류(logic error)는 Rust에서도 그대로 컴파일됩니다.

**왜 그런가** Rust 레퍼런스의 "Behavior not considered unsafe"는 안전한 코드에서 일어날 수 있지만 미정의 동작(undefined behavior)은 아닌 것으로 교착 상태, 메모리와 자원의 누수, 소멸자를 부르지 않고 종료하기, 포인터 누출로 무작위 주소 드러내기, 정수 오버플로, 논리 오류를 듭니다. Rustonomicon도 교착 상태, 경쟁 조건, 메모리 누수, 정수 오버플로, 프로그램 중단(abort)을 "바람직하지 않지만 안전한" 일로 꼽고, 마지막에 "운영 데이터베이스 지우기"를 덧붙입니다. 메모리 안전성은 잘못된 메모리에 닿지 않는다는 약속이지 프로그램이 옳다는 약속이 아닙니다.

[04_logic_bug_compiles](examples/04_logic_bug_compiles.rs)는 파일 서버가 요청 경로를 기준 디렉터리 아래로 바꾸는 코드입니다.

```text
요청 "../../etc/passwd"
  [비권장] /srv/files/../../etc/passwd
  [권장]   거부: "../../etc/passwd"
요청 "/etc/passwd"
  [비권장] /etc/passwd
  [권장]   거부: "/etc/passwd"
```

`Path::join`에 요청을 그대로 넘기면 `..`로 기준 디렉터리 밖으로 나가고, 절대 경로는 기준 경로를 통째로 바꿉니다(`join`의 문서가 밝힌 동작입니다). MITRE가 CWE-22로 분류하는 경로 순회(path traversal) 취약점이지만, 메모리를 잘못 다루는 곳은 없으므로 컴파일러도 경계 검사도 막지 않습니다. 권장 판은 경로의 구성 요소가 모두 보통 이름일 때만 받습니다.

**오해와 한계** 타입을 잘 설계하면 일부 논리 오류도 컴파일 에러로 바꿀 수 있습니다(단위를 구분하는 뉴타입, 상태를 열거형으로 표현하기 등. [부록 A ch1](../../appd-a/ch1-express-data-structures/README.md)·[ch6](../../appd-a/ch6-newtype-pattern/README.md)). 그래도 "무엇이 옳은가"는 결국 프로그래머가 정하므로 테스트와 리뷰는 그대로 필요합니다.

**더 읽기** ch2 Q2.1 / Q3.8 / 부록 A ch1·ch6

### Q3.2 `unsafe`가 있으면 결국 C와 같지 않나요?

**짧은 답** 다릅니다. `unsafe`는 다섯 가지 일만 허용하고, 대여 검사와 타입 검사는 `unsafe` 블록 안에서도 그대로 합니다. 또 `unsafe` 코드는 키워드로 드러나서 검토할 곳이 좁혀지고, 크레이트 단위로 아예 금지할 수도 있습니다. C에서는 프로그램 전체가 `unsafe` 블록 안에 있는 것과 같습니다.

**왜 그런가** Rustonomicon에 따르면 `unsafe`가 허용하는 것은 원시 포인터 역참조, unsafe 함수 호출(C 함수 포함), unsafe 트레이트 구현, 가변 정적 변수 접근, 유니언 필드 접근의 다섯 가지입니다(Part 0 ch19 2.1절). Rust 레퍼런스는 이를 더 잘게 나눠 `target_feature`가 붙은 함수 호출, `extern` 블록 선언, unsafe 속성까지 따로 들지만, 어느 쪽이든 `unsafe`가 여는 문은 정해진 몇 가지뿐입니다. 그 밖의 규칙은 꺼지지 않습니다. Part 0 ch19 2.1절에서 본 대로, `unsafe` 블록 안에서 같은 `Vec`을 두 번 가변으로 빌리는 코드는 블록 밖에서와 똑같이 E0499로 거부됩니다. [`compile_fail/e0499_unsafe_keeps_borrowck.rs`](compile_fail/e0499_unsafe_keeps_borrowck.rs)는 그 상황을 파일로 재현합니다(4.1절).

`unsafe`를 쓰는 관용적인 모양은 **안전한 API 안에 가두는 것**입니다. [05_unsafe_boundary](examples/05_unsafe_boundary.rs)의 두 부분이 그 예입니다.

```text
1. split_at_mut: 안전한 함수 안의 unsafe
  [40, 20, 30, 40]
2. 직접 만든 안전한 래퍼
  middle([1, 2, 3]) = Some(2)
  middle([]) = None
```

- 표준 라이브러리의 `split_at_mut`은 한 `Vec`의 겹치지 않는 두 부분을 동시에 가변으로 빌려 줍니다. 대여 검사기는 두 부분이 겹치지 않는다는 것을 증명하지 못하므로 안에서는 `unsafe`로 구현되어 있지만(Part 0 ch19 2.3절), 부르는 쪽은 안전한 함수로 씁니다.
- `middle`은 경계 검사를 건너뛰는 `get_unchecked`를 쓰되, 바로 위에서 슬라이스가 비어 있지 않음을 확인하고 그 이유를 `// SAFETY:` 주석으로 적습니다. 부르는 쪽은 어떤 슬라이스를 넘겨도 범위 밖을 읽을 수 없고, 검토할 곳은 이 함수 하나입니다.

`unsafe`가 아예 필요 없는 크레이트는 맨 위에 `#![forbid(unsafe_code)]`를 둡니다. 그러면 크레이트 안의 `unsafe`가 모두 컴파일 에러가 되고(4.2절), 안쪽에서 `#[allow]`로 다시 풀 수도 없습니다.

**다른 언어에서는** ch2 Q2.5에서 봤듯이 Java·Kotlin은 JNI, Go는 `unsafe` 패키지로 메모리 안전하지 않은 코드의 경계가 드러납니다. Rust의 차이는 그 경계 밖에서 GC 없이 메모리 안전성을 얻는다는 데 있습니다. C와 C++에는 이런 경계가 없어서, 메모리 오류가 어디서든 생길 수 있다고 보고 전체를 검토해야 합니다.

**오해와 한계** `unsafe` 블록 안의 실수는 C의 실수와 똑같이 미정의 동작이 됩니다. 그리고 `unsafe` 블록이 지켜야 하는 조건은 블록 바깥의 안전한 코드에도 기댈 수 있습니다. 예를 들어 `middle`의 `if values.is_empty()` 검사를 누가 지우면 블록 안을 고치지 않아도 미정의 동작이 생깁니다. Q3.3의 CVE-2018-1000657이 실제로 이런 모양이었습니다. 그래서 검토 범위는 `unsafe` 블록이 아니라 그 블록의 전제 조건에 영향을 주는 코드 전체, 보통은 그 블록을 둘러싼 모듈입니다(Rustonomicon "Working with Unsafe", [부록 A ch16](../../appd-a/ch16-avoid-unsafe/README.md)).

**더 읽기** Part 0 ch19 2.1~2.3절 / 부록 A ch16 / ch10 `ch10-undefined-behavior`

### Q3.3 표준 라이브러리도 `unsafe`를 쓰는데 믿을 수 있나요?

**짧은 답** 대부분의 코드가 기대는 만큼 많이 검증되지만, 완벽하다는 보장은 없습니다. 표준 라이브러리의 테스트는 미정의 동작 검출기인 Miri로도 정기적으로 돌고, `unsafe` 코드를 형식 검증하려는 연구와 프로젝트도 이어지고 있습니다. 그래도 표준 라이브러리에서 `unsafe`를 쓰는 코드에서 메모리 안전성 취약점이 나온 적이 있습니다.

**왜 그런가** `Vec`, `String`, `HashMap`, `Rc`, `Mutex` 같은 타입은 메모리와 운영체제를 직접 다뤄야 하므로 안에서 `unsafe`를 씁니다. 1.100.0에 함께 들어 있는 표준 라이브러리 소스에서 `Vec`(alloc/vec/mod.rs)과 `Rc`(alloc/rc.rs)를 정의한 파일만 봐도 `unsafe`라는 낱말이 각각 100번 넘게 나옵니다. 이 코드를 확인하는 장치는 여러 겹입니다.

| 장치 | 내용 | 출처 |
| --- | --- | --- |
| Miri로 테스트 | `miri-test-libstd` 저장소가 libcore·liballoc·libstd의 테스트를 Miri로 돌리고, CI가 매일 밤 최신 nightly로 실행합니다(Miri가 지원하지 않는 플랫폼 API에 기대는 일부 테스트는 빠짐). | rust-lang/miri-test-libstd |
| 형식 검증(formal verification) 연구 | RustBelt 프로젝트(POPL 2018 논문 *RustBelt: Securing the Foundations of the Rust Programming Language*)는 `unsafe` 코드를 안전하게 감쌌는지 검증하는 첫 형식 도구를 목표로 했습니다. | plv.mpi-sws.org/rustbelt |
| 표준 라이브러리 검증 프로젝트 | "Verify the Rust Standard Library"는 표준 라이브러리 구현이 안전한지 자동으로 검증하는 것을 목표로, 과제와 상금을 건 공모 형태로 진행됩니다. | model-checking.github.io/verify-rust-std |

그래도 버그는 나옵니다. CVE-2018-1000657은 `VecDeque::reserve`의 버퍼 오버플로로, Rust 1.3.0부터 1.22.0 직전까지 영향을 주었고 NVD는 임의 코드 실행으로 이어질 수 있다고 적었습니다(CVSS 3.0 7.8). NVD가 가리키는 수정(PR #44802, "Fix capacity comparison in reserve")이 고친 줄은 `unsafe` 블록 안이 아니라 그 바로 위의 용량 비교였습니다. 안전한 코드의 비교가 틀려 크기를 늘리지 않았는데도 `unsafe` 메서드(`handle_cap_increase`)를 불러 내부 상태를 망가뜨린 것입니다. Q3.2의 "검토 범위는 블록이 아니라 그 블록을 둘러싼 모듈"이 실제로 어떤 뜻인지 보여 주는 사례입니다.

**오해와 한계** 표준 라이브러리를 믿는 것은 C 프로그래머가 libc를, Java 프로그래머가 JVM을 믿는 것과 같은 종류의 신뢰입니다. 다른 점은 그 신뢰가 필요한 코드가 `unsafe`로 표시된 작은 부분이고, 그 부분을 위에 적은 도구들로 따로 검사할 수 있다는 것입니다.

**더 읽기** Q3.8, Q3.10 / 부록 A ch16

### Q3.4 메모리 누수는 왜 막지 않나요?

**짧은 답** 메모리 누수(memory leak)는 메모리 안전성 위반이 아니기 때문입니다. 해제되지 않은 메모리를 잘못 읽거나 쓰는 일은 생기지 않습니다. 또 모든 누수를 컴파일 시간에 막으려면 순환하는 자료구조나 프로그램이 끝날 때까지 두는 값처럼 쓸모 있는 기능까지 막아야 합니다.

**왜 그런가** The Book(15.6절)은 Rust의 메모리 안전성 보장이 누수를 어렵게 만들 뿐 불가능하게 하지는 않으며, 누수를 완전히 막는 것은 Rust의 보장에 들어 있지 않다고 설명합니다. 표준 라이브러리 문서도 `mem::forget`이 `unsafe`가 아닌 이유로 Rust의 안전성 보장이 "소멸자가 언제나 실행된다"는 것을 포함하지 않는다는 점을 듭니다. `Rc` 순환 참조나 `process::exit`로도 소멸자를 건너뛸 수 있으므로 `forget`을 안전한 코드에 허용해도 보장이 달라지지 않는다는 설명입니다.

[01_leaks_are_safe](examples/01_leaks_are_safe.rs)는 버려질 때 이름을 출력하는 값을 세 가지 방법으로 잃어버립니다.

```text
1. mem::forget
  (forget 뒤에 아무것도 출력되지 않음)
2. Box::leak
  새어 나간 값을 계속 쓸 수 있음: 새어 나간 값
3. Rc 순환 참조
  A의 참조 수 2, B의 참조 수 2
  (스코프가 끝났지만 A도 B도 버려지지 않음)
```

세 방법 모두 `unsafe` 없이 컴파일되고 어느 값의 소멸자도 실행되지 않습니다. `Box::leak`은 프로그램이 끝날 때까지 쓸 설정 값처럼 일부러 해제하지 않는 메모리에 쓰라고 만든 함수입니다(표준 라이브러리 문서). 순환 참조는 Part 0 ch14 2.10·2.11절처럼 한쪽을 `Weak`로 바꿔 끊습니다.

**다른 언어에서는** GC 언어도 누수를 막지 못합니다. GC는 닿을 수 있는 객체를 거두지 않으므로(ch2 Q2.5), 더는 쓰지 않는 객체를 전역 컬렉션이나 캐시가 계속 붙잡고 있으면 그 객체는 남습니다. C에서는 `free`를 빠뜨리는 것만으로 누수가 생깁니다. C++는 스마트 포인터로 대부분의 누수를 막지만, `shared_ptr`끼리 서로를 가리키면 `Rc` 순환 참조처럼 새고 `weak_ptr`로 끊는다는 점까지 Rust와 같습니다.

**오해와 한계** 누수가 안전하다는 것은 "괜찮다"는 뜻이 아닙니다. 오래 도는 서버에서 누수는 결국 메모리를 다 써 버립니다. 또 소멸자가 실행되지 않으면 파일이 닫히지 않거나 잠금이 풀리지 않는 것처럼 메모리 밖의 자원도 새므로, 소멸자에 기대는 정리 코드(RAII)는 "보통은 실행된다"까지만 믿을 수 있습니다([부록 A ch11](../../appd-a/ch11-drop-raii/README.md)).

**더 읽기** Part 0 ch14 2.10·2.11절 / 부록 A ch11 / Part 10 `ch7-obrm`

### Q3.5 교착 상태와 경쟁 조건은 막아 주나요?

**짧은 답** 막지 않습니다. Rust가 컴파일 시간에 막는 것은 데이터 경쟁(data race), 즉 여러 스레드가 동기화 없이 같은 메모리를 읽고 쓰는 것뿐입니다. 서로 잠금을 기다리며 멈추는 교착 상태(deadlock)와, 실행 순서에 따라 결과가 틀려지는 경쟁 조건(race condition)은 안전한 코드에서도 생깁니다.

**왜 그런가** 둘 다 메모리를 잘못 다루는 문제가 아니라 프로그램의 논리 문제이기 때문입니다. Rust 레퍼런스는 교착 상태를, Rustonomicon은 교착 상태와 경쟁 조건을 안전하지만 바람직하지 않은 일로 꼽습니다(Q3.1). Part 0 ch15 2.10절에서 본 두 버그를 [02_deadlock_and_race](examples/02_deadlock_and_race.rs)가 매번 같은 결과로 재현합니다.

```text
1. 잠금을 쥔 채 같은 Mutex를 다시 잠그기
  지금은 잠글 수 없음 (lock()이었다면 이 호출은 돌아오지 않음)
  잠금을 푼 뒤 다시 잠금: true
2. 잔액 100원에서 두 스레드가 70원씩 출금
  [비권장] 확인과 출금을 따로 잠금: 성공 2건, 잔액 -40원
  [권장] 한 번의 잠금 안에서 확인과 출금: 성공 1건, 잔액 30원
```

- **교착 상태**: 잠금을 쥔 채 같은 `Mutex`를 다시 잠그는 일은 잠금을 잡는 도우미 함수를 잠금 안에서 부를 때 흔히 생깁니다. 표준 라이브러리 문서는 이때 `lock()`이 돌아오지 않는다(패닉하거나 멈출 수 있다)고 적으므로, 예제는 Part 0 ch15 예제 05처럼 `try_lock()`으로 관찰만 합니다. 두 스레드가 잠금 둘을 반대 순서로 잡아 서로를 기다리는 경우는 [부록 A ch17](../../appd-a/ch17-shared-state-parallelism/README.md) 예제 01에서 볼 수 있습니다.
- **경쟁 조건**: 비권장 판은 잔액을 확인하는 잠금과 출금하는 잠금이 따로라서, 두 스레드가 모두 "잔액 100원이니 충분하다"고 확인한 뒤 각자 출금해 잔액이 -40원이 됩니다. Part 0 ch15 예제 06의 "읽고 다시 잠가 쓰기"와 같은 구조이지만, 이 예제는 `Barrier`로 두 스레드가 모두 확인을 마친 뒤 출금하게 해서 실행할 때마다 같은 결과가 나옵니다. 실제 프로그램에서는 운 나쁜 실행에서만 나타납니다. 모든 접근이 `Mutex`를 거치므로 데이터 경쟁은 없고 컴파일러도 경고하지 않습니다. 권장 판처럼 확인과 출금을 한 번의 잠금 안에서 하면 고쳐집니다.

**다른 언어에서는** 교착 상태와 경쟁 조건은 어느 언어에서나 같은 방식으로 생깁니다. Rust가 덜어 주는 것은 그 아래층의 데이터 경쟁뿐입니다([ch8](../ch8-concurrency-async/README.md)).

**오해와 한계** Rust의 API 설계가 실수를 줄여 주는 부분은 있습니다. `Mutex`가 데이터를 감싸므로 잠그지 않고 데이터에 닿을 수 없고, 잠금 가드가 스코프를 벗어나면 자동으로 풀립니다. 하지만 "무엇을 한 번의 잠금으로 묶어야 하는가"는 여전히 설계의 문제입니다.

**더 읽기** Part 0 ch15 2.10절 / 부록 A ch17 / ch8

### Q3.6 정수 오버플로는 어떻게 되나요?

**짧은 답** 빌드 설정에 따라 다릅니다. 디버그 빌드는 오버플로(overflow)가 나면 패닉하고, 릴리스 빌드는 기본으로 2의 보수 래핑(wrapping)을 합니다. 어느 쪽이든 결과가 정해져 있어 미정의 동작은 아닙니다. 래핑을 원하면 `wrapping_add`, 실패로 다루려면 `checked_add`처럼 뜻을 밝힌 메서드를 씁니다.

**왜 그런가** Rust 레퍼런스는 산술 오버플로를 프로그래머의 실수로 보고, 디버그 단언이 켜진 빌드(최적화하지 않은 빌드 등)에서는 오버플로를 검사해 패닉해야 하며, 그 밖의 빌드에서는 패닉하거나 2의 보수로 래핑할 수 있다고 정합니다. Cargo의 기본 프로필은 `dev`에서 `overflow-checks = true`, `release`에서 `false`입니다([Part 3 ch6](../../part3-cargo/ch6-profiles-build-performance/README.md)). 이 규칙과 오버플로를 다루는 메서드 4종(`checked_*`, `wrapping_*`, `saturating_*`, `overflowing_*`), 값을 미리 알면 컴파일 에러가 되는 것은 Part 0 ch3 4절과 예제 04에서 봤습니다. 여기서는 릴리스 빌드에서 래핑된 값이 **어디까지 가는지**를 봅니다. [ch2](../ch2-memory-safety/README.md) 예제 04가 `checked_mul`로 본 크기 계산을 그냥 `+`로 했을 때의 뒷이야기입니다. [03_integer_overflow](examples/03_integer_overflow.rs)는 패킷 길이를 헤더 200바이트 + 본문 100바이트로 계산하는데, 길이 필드가 `u8`이라 300을 담지 못합니다.

```text
1. 권장: checked_add로 길이 계산
  길이가 u8을 넘으므로 요청 거부
2. 비권장: 그냥 +로 길이 계산
  계산한 길이 44
  44바이트 버퍼에 300바이트를 담을 자리가 없어 거부
```

위는 릴리스 빌드(`--release`)의 출력입니다. 래핑된 길이 44로 버퍼를 만들었지만, `get_mut`이 범위를 검사하므로 300바이트를 버퍼 밖에 쓰지 않고 거부합니다. C였다면 44바이트 버퍼에 300바이트를 복사해 힙을 망가뜨렸을 자리입니다. 디버그 빌드에서는 `2.`의 덧셈에서 `attempt to add with overflow` 패닉으로 멈추고(종료 코드 101), 릴리스 빌드라도 `CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true`로 오버플로 검사를 켜면 같은 자리에서 멈춥니다. 정리하면 Rust에서 래핑은 메모리 오류가 아니라 "틀린 길이"라는 논리 오류로 남고, 그 오류를 덧셈에서 잡을지는 빌드 설정이 정합니다.

**다른 언어에서는** C와 C++에서 부호 있는 정수의 오버플로는 미정의 동작이고, 부호 없는 정수는 래핑합니다(ch10 `ch10-undefined-behavior`). Java 언어 명세는 정수 덧셈이 오버플로하면 결과가 수학적 합의 낮은 비트라고 정하고, Go 명세도 부호 없는 정수는 2ⁿ으로 나눈 나머지, 부호 있는 정수는 표현에 따라 정해진 값이 되며 패닉하지 않는다고 정합니다. C#은 기본으로 검사하지 않는 문맥이라 래핑하지만, 상수 식의 오버플로는 컴파일 에러이고 `checked` 문맥이나 컴파일러 옵션으로 실행 중 예외를 켤 수 있습니다. Python의 정수는 크기 제한이 없습니다. Rust의 차이는 디버그 빌드에서 실수를 패닉으로 드러낸다는 점입니다.

**오해와 한계** 릴리스 빌드의 래핑은 "조용히 틀린 값"입니다. ch2 Q2.7처럼 크기 계산에 쓰는 값이라면 `checked_*`로 계산하거나, 프로필에서 `overflow-checks = true`를 켜 릴리스에서도 검사합니다(검사 비용은 ch14 `ch14-performance`).

**더 읽기** ch2 Q2.7 / Part 3 ch6 / ch10

### Q3.7 패닉은 서비스 거부 공격이 될 수 있지 않나요?

**짧은 답** 될 수 있습니다. 패닉은 메모리 안전성을 지키며 멈추지만(ch2 Q2.6), 공격자가 일부러 패닉을 일으킬 수 있다면 서버를 멈추게 하는 서비스 거부(denial of service)가 됩니다. 외부 입력 때문에 생기는 실패는 `Result`로 거부하고, 입력의 크기와 중첩 깊이에 한도를 둡니다.

**왜 그런가** 외부 입력이 프로그램을 멈추게 하는 길은 여러 가지입니다.

| 길 | 예 | 막는 법 |
| --- | --- | --- |
| 패닉 | 입력이 정한 인덱스로 인덱싱, `unwrap` | `get`과 `Result`로 거부(ch2 예제 02, 연습 2) |
| 디버그 빌드의 오버플로 | 입력이 정한 수를 더하기 | `checked_*` (Q3.6) |
| 스택 오버플로 | 입력의 중첩 깊이만큼 재귀 | 깊이에 한도 두기, 재귀 대신 반복 |
| 메모리 고갈 | 입력이 주장한 크기로 미리 할당 | 크기에 한도 두기, 실제로 읽은 만큼만 늘리기 |

스택 오버플로는 패닉과 다르게 끝납니다. 이 저장소의 툴체인에서 다음처럼 `[`가 100만 개인 입력의 깊이를 재귀로 세는 함수를 디버그 빌드로 돌려 보았습니다.

```rust
fn depth(input: &[u8]) -> usize {
    match input.first() {
        Some(b'[') => 1 + depth(&input[1..]),
        _ => 0,
    }
}
// main에서: println!("{}", depth(&vec![b'['; 1_000_000]));
```

그러면 표준 에러에 `thread 'main' (스레드 번호) has overflowed its stack`과 `fatal runtime error: stack overflow, aborting`이 나오고 종료 코드 134(SIGABRT)로 끝납니다. 되감기 없이 프로세스가 바로 중단되므로 소멸자도 실행되지 않습니다. (같은 함수를 최적화해 빌드하면 컴파일러가 재귀를 반복문으로 바꿔 끝까지 실행되었으므로, 최적화에 기대지 말고 깊이 한도를 두어야 합니다.)

**다른 언어에서는** GC 언어에서도 처리되지 않은 예외, 깊은 재귀, 과도한 할당은 같은 공격면이 됩니다. RustSec 권고 데이터베이스가 서비스 거부(denial-of-service)를 따로 분류해 둘 만큼 Rust 크레이트에서도 흔한 종류입니다.

**오해와 한계** 패닉을 스레드나 요청 단위로 가두는 방법(`catch_unwind`, 비동기 런타임의 태스크 격리)도 있지만, 예외 처리처럼 쓰라고 만든 장치가 아니고 abort로 끝나는 경우는 막지 못합니다([부록 A ch18](../../appd-a/ch18-dont-panic/README.md)).

**더 읽기** ch2 Q2.6 / 부록 A ch18

### Q3.8 Rust로 쓴 코드에서도 CVE가 나오나요?

**짧은 답** 나옵니다. 메모리 안전성 취약점은 `unsafe`를 쓰는 코드(모듈)에서 나오고, 명령 주입·경쟁 조건 같은 그 밖의 취약점은 안전한 코드에서도 나옵니다. 표준 라이브러리도 예외가 아닙니다.

**왜 그런가** 1차 자료로 확인한 사례를 종류별로 나눕니다.

| 사례 | 어디서 | 종류 | 고친 판 |
| --- | --- | --- | --- |
| CVE-2018-1000657 | 표준 라이브러리 `VecDeque::reserve` | 버퍼 오버플로(메모리 오류). `unsafe` 호출 바로 위의 안전한 비교가 틀림 | 1.22.0 |
| CVE-2022-21658 | 표준 라이브러리 `fs::remove_dir_all` | 심볼릭 링크를 따라가게 만드는 경쟁 조건(확인 시점과 사용 시점의 차이) | 1.58.1 |
| CVE-2024-24576 | 표준 라이브러리 `process::Command`(Windows에서 배치 파일 실행) | 인자 이스케이프 실패로 인한 명령 주입. Rust 팀이 "critical"로 평가 | 1.77.2 |
| CVE-2025-48530 | Android의 CrabbyAVIF(Rust) | 선형 버퍼 오버플로(메모리 오류). Android 팀이 "Rust로 쓴 코드의 첫 메모리 안전성 취약점"이 될 뻔했다고 부름. 공개 릴리스 전에 잡힘 | — |

- 메모리 오류인 두 건(2018, 2025)은 안전한 Rust의 보장(ch2 Q2.4) 밖에서 나온 것입니다. 2018년 건은 `unsafe` 호출의 전제 조건을 안전한 코드가 깨뜨린 경우였습니다(Q3.3). 2025년 건에 대해 Android 팀의 글은 그 코드가 `unsafe`였는지 밝히지 않습니다. 사후 분석의 교훈으로는 Scudo 강화 할당기와 함께 "Unsafe Review and Training"을 들고, `unsafe` 코드 교육 과정을 더하겠다고 씁니다. 같은 글은 이 1건을 근거로 Rust 코드의 메모리 안전성 취약점 밀도를 C/C++보다 1000배 넘게 낮게 추정했습니다(ch2 Q2.8).
- 나머지 두 건은 메모리 오류가 아닙니다. `remove_dir_all`은 "심볼릭 링크인지 확인"과 "지우기" 사이에 공격자가 링크를 바꿔 끼울 수 있었던 경쟁 조건이고(Q3.5), `Command`는 Windows 배치 파일에 넘기는 인자를 제대로 이스케이프하지 못한 것입니다(Q3.1의 경로 순회와 같은 논리 오류).

크레이트의 취약점은 RustSec 권고 데이터베이스에 모입니다. 분류는 열 가지로, 메모리 손상(memory-corruption)과 메모리 노출(memory-exposure)뿐 아니라 코드 실행, 암호 실패, 서비스 거부, 파일 노출, 형식 주입, 권한 상승, 스레드 안전성, 악성(malicious)이 있습니다.

**오해와 한계** CVE의 수만으로 언어를 비교하기는 어렵습니다. 코드의 양, 쓰이는 곳, 얼마나 열심히 찾는지에 따라 보고되는 수가 달라지기 때문입니다. 위 사례는 "어떤 종류의 취약점이 남는가"를 보는 데 쓰는 것이 맞습니다.

**더 읽기** Q3.1, Q3.3, Q3.5 / ch2 Q2.8

### Q3.9 의존하는 크레이트가 많은데 공급망은 안전한가요?

**짧은 답** 공급망(supply chain), 즉 내가 가져다 쓰는 남의 코드의 안전은 언어가 보장하는 영역이 아닙니다. 크레이트는 안전한 Rust만으로도 파일을 지우거나 비밀을 밖으로 보낼 수 있고, 빌드 스크립트와 절차적 매크로는 빌드하는 순간에 코드를 실행합니다. crates.io에도 악성 크레이트가 올라온 적이 있습니다.

**왜 그런가** 메모리 안전성은 "코드가 메모리를 잘못 다루지 않는다"는 약속일 뿐 "코드가 나쁜 일을 하지 않는다"는 약속이 아닙니다. 2025년 9월 24일 crates.io 팀은 악성 크레이트 `faster_log`와 `async_println`을 지웠다고 발표했습니다. 두 크레이트는 정상적인 로깅 크레이트의 소스와 문서를 비슷한 이름으로 베낀 것이었고, 그 크레이트에 의존하는 프로젝트를 실행하거나 테스트할 때 로그 파일에서 암호화폐 개인 키를 찾아 외부 서버로 보냈습니다. 발표문은 빌드할 때는 악성 코드를 실행하지 않았다고 밝힙니다. 그러나 빌드할 때 코드를 실행하는 길도 있습니다. 빌드 스크립트(`build.rs`)와 절차적 매크로는 크레이트를 빌드하는 순간에 실행되므로([Part 3 ch8](../../part3-cargo/ch8-build-scripts/README.md)), 의존성을 추가하는 것은 그 코드를 내 컴퓨터와 CI에서 실행하는 것과 같습니다.

**다른 언어에서는** npm, PyPI, Maven Central 같은 다른 패키지 저장소도 같은 종류의 위험을 안고 있습니다. 공급망 위험은 언어가 아니라 패키지 생태계의 문제입니다.

**오해와 한계** `Cargo.lock`으로 버전을 고정하고, 의존성의 수를 줄이고, RustSec 데이터베이스를 확인하는 도구로 점검하는 방법이 있습니다. 크레이트를 고르고 믿는 방법은 ch16 `ch16-tooling-ecosystem`(Q16.3)에서 다룹니다.

**더 읽기** ch16 / [부록 A ch25](../../appd-a/ch25-dependency-graph/README.md)

### Q3.10 컴파일러에 버그가 있으면 보장이 깨지나요?

**짧은 답** 깨질 수 있습니다. Rust의 보장은 컴파일러가 규칙을 올바르게 검사한다는 전제 위에 있으므로, 타입 검사에 구멍(건전성(soundness) 버그)이 있으면 안전한 코드로도 미정의 동작을 만들 수 있습니다. 이런 버그는 rust-lang/rust 저장소에 `I-unsound` 라벨로 공개 추적됩니다.

**왜 그런가** `I-unsound` 라벨은 "건전성 구멍(가장 나쁜 종류의 버그)"이라는 설명이 붙어 있고, 2026-11-21에 열린 이슈가 178개였습니다. 오래된 예로 이슈 #25860("Implied bounds on nested references + variance = soundness hole")은 2015년 5월 28일에 열려 지금도 열려 있습니다. 따라서 "안전한 코드라면 미정의 동작이 없다"는 말에는 "컴파일러에 건전성 버그가 없다면"이라는 조건이 붙습니다.

**다른 언어에서는** 어느 언어의 보장도 그 언어의 구현(컴파일러, 런타임)이 올바르다는 전제 위에 있습니다.

**오해와 한계** 건전성 버그가 있다는 것과 실제 취약점이 된다는 것은 다른 문제입니다. Q3.8의 사례는 모두 컴파일러가 아니라 라이브러리 코드의 버그였습니다.

**더 읽기** Q3.3, Q3.8 / Part 10 `ch1-safe-and-unsafe`

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_leaks_are_safe](examples/01_leaks_are_safe.rs) | `mem::forget`, `Box::leak`, `Rc` 순환 참조로 만든 누수. 소멸자가 실행되지 않음 | Q3.4 |
| [02_deadlock_and_race](examples/02_deadlock_and_race.rs) | 잠금을 쥔 채 다시 잠그기(`try_lock`으로 관찰), 확인과 출금을 따로 잠근 경쟁 조건 | Q3.5 |
| [03_integer_overflow](examples/03_integer_overflow.rs) | `checked_add`로 거부하기와, 그냥 `+`로 계산한 길이가 빌드마다 어디서 멈추는지 (디버그 빌드는 종료 코드 101) | Q3.6 |
| [04_logic_bug_compiles](examples/04_logic_bug_compiles.rs) | `Path::join`에 요청을 그대로 넘기는 경로 순회 버그와 고친 판 | Q3.1, Q3.8 |
| [05_unsafe_boundary](examples/05_unsafe_boundary.rs) | 표준 라이브러리의 `split_at_mut`, 전제 조건을 확인하는 작은 안전한 래퍼 | Q3.2, Q3.3 |

### 01_leaks_are_safe — 안전한 코드로 만든 누수

```sh
cargo run -p faq-ch3-safety-limits --example 01_leaks_are_safe
```

```text
0. 비교: 보통은 스코프가 끝날 때 버려짐
  보통 값 버려짐
1. mem::forget
  (forget 뒤에 아무것도 출력되지 않음)
2. Box::leak
  새어 나간 값을 계속 쓸 수 있음: 새어 나간 값
3. Rc 순환 참조
  A의 참조 수 2, B의 참조 수 2
  (스코프가 끝났지만 A도 B도 버려지지 않음)
끝: 잊은 값, 새어 나간 값, A, B는 프로그램이 끝날 때까지 버려지지 않습니다
```

- 0번은 비교용입니다. 스코프가 끝나면 "버려짐"이 출력됩니다.
- 3번에서 A와 B는 `next` 필드로 서로를 가리키므로 참조 수가 2입니다. 지역 변수 `a`, `b`가 사라지면 참조 수는 1이 될 뿐 0이 되지 않습니다. `next`가 `RefCell`인 것은 만든 뒤에 연결을 바꾸기 위해서입니다(Part 0 ch14 2.8절).
- 프로그램이 끝나면 운영체제가 프로세스의 메모리를 통째로 거두지만, 그때도 소멸자는 실행되지 않습니다.

### 02_deadlock_and_race — 컴파일되는 동시성 버그

```sh
cargo run -p faq-ch3-safety-limits --example 02_deadlock_and_race
```

```text
1. 잠금을 쥔 채 같은 Mutex를 다시 잠그기
  지금은 잠글 수 없음 (lock()이었다면 이 호출은 돌아오지 않음)
  잠금을 푼 뒤 다시 잠금: true
2. 잔액 100원에서 두 스레드가 70원씩 출금
  [비권장] 확인과 출금을 따로 잠금: 성공 2건, 잔액 -40원
  [권장] 한 번의 잠금 안에서 확인과 출금: 성공 1건, 잔액 30원
```

- `try_lock`은 잠금을 기다리지 않고, 다른 쪽이 쥐고 있으면 `Err(TryLockError::WouldBlock)`을 돌려줍니다. 잠금 가드를 `drop`으로 버린 뒤에는 다시 잠글 수 있습니다.
- `Barrier::new(2)`는 두 스레드가 모두 `wait()`에 이를 때까지 기다리게 합니다. 비권장 판은 두 스레드가 모두 확인을 마친 뒤에 출금하도록, 권장 판은 두 스레드가 동시에 출발하도록 씁니다. 권장 판에서 어느 스레드가 성공하는지는 실행마다 다를 수 있으므로 성공 건수만 출력합니다. 출력은 실행마다 같습니다.

### 03_integer_overflow — 래핑된 길이가 어디까지 가는가

```sh
cargo run -p faq-ch3-safety-limits --example 03_integer_overflow
cargo run -p faq-ch3-safety-limits --example 03_integer_overflow --release
CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true cargo run -p faq-ch3-safety-limits --example 03_integer_overflow --release
```

두 번째 명령(릴리스 빌드)은 종료 코드 0으로 끝나고, 출력은 Q3.6에 실었습니다. 나머지 두 명령은 `2. 비권장: 그냥 +로 길이 계산`까지 같은 줄을 출력한 뒤 덧셈에서 패닉하고 종료 코드 101로 끝납니다. 표준 에러는 빈 줄로 시작하고, 이어서 다음이 나옵니다(괄호 안의 스레드 번호는 실행마다 다름).

```text
thread 'main' (스레드 번호) panicked at appd-b/ch3-safety-limits/examples/03_integer_overflow.rs:39:15:
attempt to add with overflow
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

- 세 번째 명령의 환경 변수는 루트 워크스페이스의 `release` 프로필에서 `overflow-checks` 하나만 켭니다. 워크스페이스 멤버(챕터 패키지)의 `Cargo.toml`에 적은 `[profile]`은 무시되므로 환경 변수를 씁니다(Part 3 ch6). 실제 프로젝트라면 루트 `Cargo.toml`에 `[profile.release] overflow-checks = true`로 적습니다.
- 값을 `black_box`로 감싼 것은 컴파일러가 값을 미리 알지 못하게 하기 위해서입니다. 값을 알면 Part 0 ch3 4절에서 본 `this arithmetic operation will overflow` 에러로 빌드가 멈춥니다.
- `get_mut(..payload.len())`은 버퍼의 앞 300바이트를 가리키는 가변 슬라이스를 돌려주고, 버퍼가 그보다 짧으면 `None`입니다.

### 04_logic_bug_compiles — 메모리 안전한 보안 버그

```sh
cargo run -p faq-ch3-safety-limits --example 04_logic_bug_compiles
```

```text
요청 "2026/report.txt"
  [비권장] /srv/files/2026/report.txt
  [권장]   /srv/files/2026/report.txt
요청 "../../etc/passwd"
  [비권장] /srv/files/../../etc/passwd
  [권장]   거부: "../../etc/passwd"
요청 "/etc/passwd"
  [비권장] /etc/passwd
  [권장]   거부: "/etc/passwd"
```

- 경로를 만들기만 하고 파일 시스템에는 닿지 않습니다. 출력은 macOS와 Linux 기준이며, Windows에서는 경로 구분자와 `join`의 규칙이 달라 결과가 다릅니다.
- `Path::components()`는 경로를 루트(`/`), 현재 디렉터리(`.`), 상위 디렉터리(`..`), 보통 이름으로 나눠 줍니다. 중간에 낀 `.`과 겹친 `/`는 정리해 없애므로, `.`은 경로 맨 앞에 올 때만 구성 요소로 나옵니다. 권장 판은 보통 이름(`Component::Normal`)만 있을 때 받습니다. `matches!(값, 패턴)`은 값이 패턴에 맞으면 `true`를 돌려주는 매크로입니다. 이 검사는 문자열 수준의 검사라, 기준 디렉터리 안에 다른 곳을 가리키는 심볼릭 링크가 있는 경우까지 막지는 않습니다.

### 05_unsafe_boundary — 안전한 API 안의 `unsafe`

```sh
cargo run -p faq-ch3-safety-limits --example 05_unsafe_boundary
```

```text
1. split_at_mut: 안전한 함수 안의 unsafe
  [40, 20, 30, 40]
2. 직접 만든 안전한 래퍼
  middle([1, 2, 3]) = Some(2)
  middle([]) = None
```

- `split_at_mut(2)`는 `[10, 20]`과 `[30, 40]`을 가리키는 가변 슬라이스 둘을 돌려주므로 `front[0] += back[0]`이 첫 원소를 40으로 바꿉니다.
- `middle`의 `unsafe` 블록은 Miri로 검사하지 못했습니다. 전제 조건(비어 있지 않음)은 바로 위의 `if`가 확인합니다. 같은 일은 `values.get(values.len() / 2).copied()`처럼 안전한 코드로도 할 수 있고, 실제 코드라면 그쪽을 써야 합니다(부록 A ch16). 여기서는 `unsafe`를 가두는 모양을 보이려고 일부러 썼습니다.

## 4. 자주 하는 실수

### 4.1 `unsafe`가 대여 검사를 끈다고 생각함 — E0499

파일: [`compile_fail/e0499_unsafe_keeps_borrowck.rs`](compile_fail/e0499_unsafe_keeps_borrowck.rs)

```text
warning: unnecessary `unsafe` block
  --> compile_fail/e0499_unsafe_keeps_borrowck.rs:18:5
   |
18 |     unsafe {
   |     ^^^^^^ unnecessary `unsafe` block
   |
   = note: `#[warn(unused_unsafe)]` (part of `#[warn(unused)]`) on by default

error[E0499]: cannot borrow `scores` as mutable more than once at a time
  --> compile_fail/e0499_unsafe_keeps_borrowck.rs:20:25
   |
19 |         let first = &mut scores[0];
   |                          ------ first mutable borrow occurs here
20 |         let last = &mut scores[2]; // E0499
   |                         ^^^^^^ second mutable borrow occurs here
21 |         *first += *last;
   |         --------------- first borrow later used here
   |
   = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

- **원인**: `unsafe` 블록 안에서도 대여 규칙은 그대로입니다. Part 0 ch19 2.1절이 인용한 것과 같은 상황을 파일로 둔 것입니다. 블록 안에서 `unsafe`가 허용하는 일을 하나도 하지 않으므로, 컴파일러는 블록이 필요 없다는 경고까지 냅니다.
- **해결**: `unsafe`를 지우고, 컴파일러의 `help`대로 겹치지 않는 두 부분을 주는 안전한 API `split_at_mut`을 씁니다(예제 05).

### 4.2 `unsafe`를 금지한 크레이트에서 `unsafe` 사용 — `unsafe_code` 린트

파일: [`compile_fail/forbid_unsafe_code.rs`](compile_fail/forbid_unsafe_code.rs)

```text
error: usage of an `unsafe` block
  --> compile_fail/forbid_unsafe_code.rs:19:13
   |
19 |     let v = unsafe { *values.get_unchecked(1) };
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
note: the lint level is defined here
  --> compile_fail/forbid_unsafe_code.rs:14:11
   |
14 | #![forbid(unsafe_code)]
   |           ^^^^^^^^^^^
```

- **원인**: 크레이트 맨 위의 `#![forbid(unsafe_code)]`가 `unsafe` 블록을 에러로 만듭니다. 린트 에러라서 에러 코드는 없습니다.
- **해결**: 안전한 API(`values[1]`, `values.get(1)`)를 씁니다. `unsafe`가 꼭 필요하면 그 코드를 작은 모듈이나 크레이트로 떼어 내 그곳만 허용하고 검토합니다. 패키지 전체(모든 타깃)에 걸 때는 `Cargo.toml`의 `[lints.rust]`에 `unsafe_code = "forbid"`로 적을 수도 있습니다. 같은 상황의 에러는 [Part 3 ch3](../../part3-cargo/ch3-manifest-targets/README.md) 4.3절에도 있고, `forbid` 아래에서 `#[allow]`로 풀려 할 때의 에러(E0453)는 [부록 A ch16](../../appd-a/ch16-avoid-unsafe/README.md) 4.2절에 있습니다.

## 5. 연습 문제

### 연습 1. 순환 참조를 `Weak`로 끊기

01의 3번에서 B → A 연결을 `Weak`로 바꿔, 스코프가 끝날 때 A와 B가 모두 버려지게 해 보세요. B → A 연결을 어떤 타입의 필드에 두어야 하는지, 출력에 "A 버려짐", "B 버려짐"이 어떤 순서로 나오는지 예상하고 확인합니다.

<details>
<summary>힌트</summary>

A는 B를 강하게, B는 A를 약하게 가리키면 됩니다. 두 방향의 타입이 달라지므로 `Tracked`에 `back: RefCell<Option<Weak<Tracked>>>` 같은 필드를 하나 더 두고 `Rc::downgrade(&a)`를 넣는 방법이 간단합니다(Part 0 ch14 2.11절). 지역 변수는 선언의 역순으로 버려지므로 `b`가 먼저 사라지지만, B는 A가 강하게 붙잡고 있어 아직 살아 있습니다. 이어서 `a`가 사라지면 A의 참조 수가 0이 되어 "A 버려짐"이 먼저 나오고, A의 필드가 정리되며 B의 참조 수도 0이 되어 "B 버려짐"이 이어집니다.

</details>

### 연습 2. 도우미 함수 안의 다시 잠그기

02의 1번처럼 잠금을 쥔 채 다시 잠그는 일은 보통 함수 경계를 넘어 일어납니다. 안에서 `stock.lock()`으로 재고를 늘리는 `fn restock(stock: &Mutex<i32>)`을, 이미 `stock.lock()`으로 가드를 쥔 상태에서 부르면 어떻게 되는지 설명해 보세요(실행하지는 마세요. 표준 라이브러리 문서대로 돌아오지 않을 수 있습니다). 그다음 도우미가 잠그지 않아도 되게 `fn restock(stock: &mut i32)`로 바꾸고, 가드를 쥔 채 이 도우미로 재고를 10에서 15로 늘린 뒤 값을 출력하는 프로그램을 만들어 실행해 보세요.

<details>
<summary>힌트</summary>

처음 판의 `restock` 안의 `lock()`은 같은 스레드가 이미 쥔 잠금을 다시 잡으려 하므로 돌아오지 않습니다. 고친 판은 잠금을 부르는 쪽에서만 잡습니다. `let mut guard = stock.lock().unwrap();`로 가드를 가변 바인딩에 담고 `restock(&mut *guard)`로 안의 값을 빌려 넘깁니다(가드를 `let guard`로 담으면 `&mut *guard`가 E0596으로 거부됩니다). 도우미 안에서는 `*stock += 5;`만 하면 됩니다. 부록 A ch17의 "잠금은 도우미 메서드 안에서만" 지침도 같은 문제를 다룹니다.

</details>

### 연습 3. 경로 검사를 더 엄격하게

04의 `resolve_checked`에 두 가지를 더해 보세요. 빈 문자열은 거부하고, 경로가 `.`으로 시작하는 이름(숨김 파일, 예: `.env`)을 포함하면 거부합니다. `"2026/report.txt"`, `""`, `"config/.env"`로 시험합니다.

<details>
<summary>힌트</summary>

빈 문자열은 `components()`가 아무것도 내지 않아 `all`이 `true`가 되므로 따로 확인해야 합니다(`requested.is_empty()`). 숨김 이름은 `Component::Normal(name)`의 `name`(`&OsStr`)을 `name.to_str()`으로 바꾼 뒤 `starts_with('.')`로 확인합니다. `matches!` 대신 `match c { Component::Normal(name) => …, _ => false }`로 쓰면 이름을 꺼낼 수 있습니다.

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust의 보장과 그 범위

- [The Rust Reference: Behavior not considered unsafe](https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html): 교착 상태, 누수, 정수 오버플로, 논리 오류
- [The Rust Reference: Unsafety](https://doc.rust-lang.org/reference/unsafety.html)
- [The Rustonomicon: What Unsafe Rust Can Do](https://doc.rust-lang.org/nomicon/what-unsafe-does.html)
- [The Rust Programming Language 15.6절: Reference Cycles Can Leak Memory](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)
- 표준 라이브러리 문서: [`mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html)(Safety 절), [`Box::leak`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak), [`Mutex::lock`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock), [`Path::join`](https://doc.rust-lang.org/std/path/struct.Path.html#method.join)
- [The Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html): `overflow-checks`의 기본값
- [The Rustonomicon: Working with Unsafe](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)

표준 라이브러리 검증

- [rust-lang/miri-test-libstd](https://github.com/rust-lang/miri-test-libstd)
- [RustBelt](https://plv.mpi-sws.org/rustbelt/) (Ralf Jung 외, POPL 2018)
- [Verify the Rust Standard Library](https://model-checking.github.io/verify-rust-std/)

취약점 사례

- [CVE-2018-1000657](https://nvd.nist.gov/vuln/detail/CVE-2018-1000657) (NVD, 2018-08-20 공개), 수정 [rust-lang/rust PR #44802](https://github.com/rust-lang/rust/pull/44802)
- [Security advisory for the standard library (CVE-2022-21658)](https://blog.rust-lang.org/2022/01/20/cve-2022-21658/) (Rust 블로그, 2022-01-20)
- [Security advisory for the standard library (CVE-2024-24576)](https://blog.rust-lang.org/2024/04/09/cve-2024-24576/) (Rust 블로그, 2024-04-09)
- [Rust in Android: move fast and fix things](https://blog.google/security/rust-in-android-move-fast-fix-things/) (Google, 2025-11-13): CVE-2025-48530
- [CWE-22: Improper Limitation of a Pathname to a Restricted Directory](https://cwe.mitre.org/data/definitions/22.html) (MITRE)
- [RustSec Advisory Database](https://rustsec.org/), [분류 목록](https://rustsec.org/categories/)

공급망과 컴파일러

- [crates.io: Malicious crates faster_log and async_println](https://blog.rust-lang.org/2025/09/24/crates.io-malicious-crates-fasterlog-and-asyncprintln/) (Rust 블로그, 2025-09-24)
- [rust-lang/rust 이슈 #25860](https://github.com/rust-lang/rust/issues/25860), [`I-unsound` 라벨](https://github.com/rust-lang/rust/labels/I-unsound)

다른 언어

- [Java Language Specification 15.18.2](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.18.2): 정수 덧셈의 오버플로
- [The Go Programming Language Specification: Integer overflow](https://go.dev/ref/spec#Integer_overflow)
- [C# checked and unchecked statements](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/checked-and-unchecked)
- [Python: Numeric Types](https://docs.python.org/3/library/stdtypes.html#numeric-types-int-float-complex)
