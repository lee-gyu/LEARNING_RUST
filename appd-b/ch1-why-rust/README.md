# ch1. 왜 Rust인가

다른 언어를 써 온 사람이 Rust를 처음 볼 때 가장 먼저 묻는 것은 문법이 아니라 "왜"입니다. 이미 잘 쓰는 언어가 있는데 왜 배우기 어렵다는 언어를 또 봐야 하는지, 왜 운영체제와 브라우저를 만드는 조직들이 Rust 이야기를 하는지, 어디에 맞고 어디에 맞지 않는지. 이 챕터는 이 질문들에 근거를 붙여 답합니다. 언어의 동작은 예제 셋으로 직접 확인하고, 조직들이 옮기는 이유와 사용 현황은 보안 보고서·기술 블로그·설문 같은 1차 자료로 답합니다.

- 패키지: `faq-ch1-why-rust`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 자료 확인일: 2026-11-21. 통계와 사례는 시간이 지나면 낡으므로 6절의 원문을 읽는 시점에 다시 확인하기를 권합니다.

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q1.1](#q11-rust는-어떤-문제를-풀려고-만든-언어인가요) | Rust는 어떤 문제를 풀려고 만든 언어인가요? | C/C++의 성능·제어와 GC 언어의 메모리 안전성을 함께 얻는 것. 수단은 컴파일러가 검사하는 소유권 규칙입니다. |
| [Q1.2](#q12-시스템-프로그래밍-언어라는데-웹-백엔드나-cli-도구에도-쓰나요) | "시스템 프로그래밍 언어"라는데 웹 백엔드나 CLI 도구에도 쓰나요? | 씁니다. "시스템"은 내려갈 수 있는 바닥을 말할 뿐 위쪽을 막지 않습니다. |
| [Q1.3](#q13-왜-여러-조직이-rust로-옮기나요) | 왜 여러 조직이 Rust로 옮기나요? | 가장 큰 이유는 C/C++ 코드의 메모리 안전성 취약점입니다. 여기에 GC가 없어서 얻는 자원·지연 시간 이점과 생산성 결과가 더해집니다. |
| [Q1.4](#q14-실제로-어디에서-쓰이나요) | 실제로 어디에서 쓰이나요? | Linux 커널, Android, Cloudflare의 프록시, Discord의 서비스 같은 곳에서 씁니다. |
| [Q1.5](#q15-인기가-많다는데-얼마나-쓰이나요) | 인기가 많다는데 얼마나 쓰이나요? | "써 본 사람이 계속 쓰고 싶어 하는 비율"이 가장 높은 언어와 "많이 쓰는 언어"는 다릅니다. Rust의 사용률은 2026년 설문에서 18%입니다. |
| [Q1.6](#q16-rust가-맞지-않는-곳은-어디인가요) | Rust가 맞지 않는 곳은 어디인가요? | 빨리 결과를 보는 일이 중요하고 성능·자원 제약이 약한 곳. 컴파일 시간과 학습 비용도 따라옵니다. |
| [Q1.7](#q17-기존-코드를-전부-다시-써야-하나요) | 기존 코드를 전부 다시 써야 하나요? | 아닙니다. 새 코드부터 메모리 안전한 언어로 쓰는 것만으로도 효과가 있었다는 보고가 있습니다. |
| [Q1.8](#q18-go-c와는-어떤-관계이고-무엇을-고르나요) | Go, C++와는 어떤 관계이고 무엇을 고르나요? | 셋 다 네이티브 코드로 컴파일되지만 메모리를 맡기는 곳이 GC, 프로그래머, 컴파일러로 다릅니다. |

다음 챕터의 내용을 안다고 보고 진행합니다. 이 챕터는 그 내용을 다시 설명하지 않고 "그래서 왜 중요한가"에 답합니다.

- [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 2.1절: 메모리를 돌려주는 세 가지 방식(수동 관리, GC, 소유권)과 소유권 규칙이 막는 버그
- [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.15절: 비용 없는 추상화(zero-cost abstraction)
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.6절: `Drop`
- [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 2.1·2.7절: 겁 없는 동시성, `Mutex<T>`

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Rust가 풀려는 문제를 C/C++와 GC 언어(Java, Go, C#, Python 등)의 맞바꿈에 견주어 설명하고, 메모리 해제 시점·안전장치의 크기·스레드 공유 검사를 예제로 확인할 수 있다.
2. 조직들이 Rust로 옮기는 이유를 보안·자원·생산성으로 나누고, 근거마다 누가 언제 무엇을 센 숫자인지 함께 설명할 수 있다.
3. 설문의 "가장 사랑받는 언어"와 사용률을 구분해 읽고, Rust가 실제로 쓰이는 분야를 예로 들 수 있다.
4. Rust가 맞지 않는 경우와 Rust가 치르는 비용을 들고, Go·C++와 비교해 언어를 고르는 기준을 말할 수 있다.

학습 목표 1은 예제 01~03이 뒷받침합니다. 2~4는 실행해서 확인할 수 있는 성질이 아니므로 2절에 적은 1차 자료로 답합니다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다. 짧은 답만 읽어도 답이 되도록 썼습니다.

### Q1.1 Rust는 어떤 문제를 풀려고 만든 언어인가요?

**짧은 답** 오랫동안 맞바꿔야 했던 두 가지, 즉 C/C++ 수준의 성능·제어와 GC 언어 수준의 메모리 안전성(memory safety)을 함께 얻으려는 언어입니다. 수단은 컴파일러가 검사하는 소유권(ownership) 규칙입니다. 메모리를 언제 돌려줄지를 컴파일할 때 정하므로 가비지 컬렉터(garbage collector, GC)가 필요 없고, 같은 규칙이 스레드 사이의 데이터 경쟁(data race)도 막습니다. 공식 사이트(rust-lang.org)는 이것을 성능(Performance), 신뢰성(Reliability), 생산성(Productivity) 세 가지로 소개합니다.

**왜 그런가** 프로그램이 빌려 쓴 메모리는 언젠가 돌려줘야 하고, 언어마다 이 일을 맡기는 곳이 다릅니다(Part 0 ch4 2.1절). 맡기는 곳에 따라 얻는 것과 잃는 것이 갈립니다.

| | C/C++ | GC 언어 (Java, Go, C#, JavaScript 등) | Rust |
| --- | --- | --- | --- |
| 메모리를 돌려주는 주체 | 프로그래머(`free`, `delete`, 스마트 포인터) | 런타임의 GC | 컴파일러가 정한 시점(값이 스코프를 벗어날 때) |
| 해제 후 사용·이중 해제 | 생길 수 있음(미정의 동작) | 생기지 않음 | 안전한 코드에서는 컴파일 에러 |
| 실행 중 추가 비용 | 없음 | GC가 도는 시간과 메모리, 런타임 | GC와 런타임은 없음. 경계 검사 같은 검사는 있음 |
| 해제되는 때 | 코드에 적힌 때 | GC가 정함 | 코드의 구조(스코프)로 정해짐 |
| 데이터 경쟁 | 생길 수 있음 | 생길 수 있음(결과는 언어마다 다름, Q1.8) | 안전한 코드에서는 컴파일 에러 |

표의 해제 후 사용(use after free)은 이미 돌려준 메모리를 쓰는 것, 이중 해제(double free)는 같은 메모리를 두 번 돌려주는 것이고, 미정의 동작(undefined behavior)은 언어가 결과를 정하지 않아 무슨 일이든 일어날 수 있는 상태입니다. 경계 검사(bounds check)는 인덱스가 배열 안에 있는지 실행 중에 확인하는 것입니다.

Rust는 표의 오른쪽 열을 세 가지 약속으로 지킵니다. 이 챕터의 예제가 하나씩 보여 줍니다.

- **GC 없이, 정해진 때 해제합니다.** [01_freed_without_gc](examples/01_freed_without_gc.rs)는 1 MB 버퍼를 네 번 읽어 처리하면서 살아 있는 버퍼 수를 셉니다. 버퍼는 처리 함수가 끝날 때마다, 오류로 일찍 돌아가는 길에서도 그 자리에서 해제되므로 살아 있는 버퍼는 언제나 최대 1개입니다. 프로그램 어디에도 `free`가 없고 GC도 없습니다.
- **안전장치에 실행 중 비용을 거의 치르지 않습니다(비용 없는 추상화).** [02_safety_without_overhead](examples/02_safety_without_overhead.rs)는 "값이 없을 수 있다"(`Option<&u8>`)나 "이 수는 미터다"(`Meters(f64)`) 같은 정보를 타입에 담아도 값의 크기가 그대로이고, 반복자 체인이 힙에 아무것도 만들지 않는 16바이트짜리 값임을 `size_of`로 보여 줍니다. 비용이 드는 곳도 함께 보여 줍니다. 슬라이스 참조는 경계 검사에 쓸 길이를 들고 다니므로 주소만 있는 참조보다 8바이트 큽니다.
- **스레드 사이의 공유를 컴파일러가 검사합니다.** [03_checked_thread_sharing](examples/03_checked_thread_sharing.rs)은 스레드 네 개가 원본 텍스트의 서로 다른 조각을 복사 없이 빌려 읽고, 합계 하나는 `Mutex`로 보호해 함께 고칩니다. 같은 합계를 `Mutex` 없이 여러 스레드가 빌려 고치는 코드는 컴파일되지 않습니다(E0499).

이 설계는 2015년 5월 15일의 1.0 발표로 안정 버전이 되었고, 발표문은 그 뒤로 기존 코드를 깨는 변경을 대부분 하지 않겠다고 약속했습니다. 그 약속을 지키면서 언어를 바꾸는 장치가 에디션입니다([Part 2](../../part2-editions/)).

**다른 언어에서는** GC 언어도 메모리 안전하지만, 해제 시점을 GC가 정하므로 메모리를 다 쓴 객체가 다음 수거 때까지 남습니다. Python은 조금 다릅니다. 기본 구현은 참조 카운팅(reference counting)으로 객체를 해제하고, GC는 참조 카운팅이 거두지 못하는 순환 참조(reference cycle)를 위한 보조 장치입니다(Python `gc` 모듈 문서). C++는 RAII(자원 획득은 초기화, 자원을 객체의 수명에 묶어 소멸자가 해제하는 방식)와 스마트 포인터로 Rust와 비슷한 해제 시점을 얻을 수 있지만, 그 규칙을 지키는지 검사하는 것은 프로그래머의 몫입니다([ch11](../ch11-cpp-raii-move/README.md), [ch12](../ch12-modern-cpp-enough/README.md)). 파일·잠금 같은 메모리 밖의 자원을 정리하는 방식(`try`/`finally`, `defer`, `with`)과의 비교는 [ch5](../ch5-no-null-no-exceptions/README.md)에서 합니다.

**오해와 한계**

- "안전하다"는 말의 범위는 정해져 있습니다. 안전한(safe) Rust 코드에는 메모리 오류와 데이터 경쟁이 없다는 뜻이지, 버그가 없다는 뜻이 아닙니다. 논리 오류, 메모리 누수(memory leak), 교착 상태(deadlock)는 막지 않습니다([ch3](../ch3-safety-limits/README.md)).
- 모든 검사가 컴파일 시간에 끝나는 것은 아닙니다. 배열 인덱싱의 경계 검사처럼 실행 중에 하는 검사가 있고, 비용도 있습니다(02의 [3], [ch14](../ch14-performance/README.md)).
- 이 설계의 값은 배우는 비용과 컴파일 시간으로 치릅니다(Q1.6).

**더 읽기** Part 0 ch4 2.1절, Part 0 ch13 2.15절, Part 0 ch15 2.1절 / [ch2](../ch2-memory-safety/README.md)(메모리 안전성의 뜻), [ch4](../ch4-memory-without-gc/README.md)(GC 없는 메모리 관리), [ch8](../ch8-concurrency-async/README.md)(데이터 경쟁) / Q1.8

### Q1.2 "시스템 프로그래밍 언어"라는데 웹 백엔드나 CLI 도구에도 쓰나요?

**짧은 답** 씁니다. "시스템 프로그래밍 언어"라는 말은 Rust가 운영체제, 임베디드 장치처럼 런타임을 둘 수 없는 바닥까지 내려갈 수 있다는 뜻일 뿐, 위쪽 응용을 막지 않습니다. 공식 사이트가 내세우는 분야도 명령줄 도구, WebAssembly, 네트워크 서비스, 임베디드 네 가지입니다.

**왜 그런가** Rust 프로그램은 GC나 가상 머신 같은 런타임 없이 기계어로 컴파일되므로 바닥까지 내려갈 수 있고, 표준 라이브러리와 crates.io의 크레이트로 위쪽 응용도 만듭니다. 이 저장소에서도 이미 둘 다 만들었습니다.

| 분야 | 이 저장소에서 | 덧붙일 것 |
| --- | --- | --- |
| 명령줄 도구 | [Part 0 ch12](../../part0-getting-started/ch12-io-project-minigrep/README.md)의 `minigrep`, Part 7 `part7-cli`(미작성) | 실행 파일 하나로 배포하기 쉽습니다(아래) |
| 네트워크 서비스 | [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md)의 멀티스레드 웹 서버, [Part 1 ch12](../../part1-standard-lib/ch12-net/README.md) | 비동기 런타임(tokio 등)은 표준 라이브러리 밖에 있습니다([ch8](../ch8-concurrency-async/README.md)) |
| 임베디드 | [부록 A ch33](../../appd-a/ch33-no-std/README.md)의 `no_std`, Part 8 `part8-embedded`(미작성) | 표준 라이브러리 없이 `core`만으로 씁니다 |
| WebAssembly | 이 저장소는 다루지 않습니다 | — |

배포가 쉬운 이유는 표준 라이브러리를 포함한 크레이트가 실행 파일 안에 정적으로 링크되기 때문입니다. macOS에서 이 챕터의 예제 실행 파일을 `otool -L target/debug/examples/01_freed_without_gc`로 보면, 동적으로 링크하는 라이브러리는 시스템 라이브러리 `/usr/lib/libSystem.B.dylib` 하나뿐입니다. 대상 컴퓨터에 언어 런타임(JVM, 인터프리터)을 따로 설치할 필요가 없습니다.

**오해와 한계** 쓸 수 있다는 것과 그 분야에서 가장 쉬운 선택이라는 것은 다릅니다. 웹 백엔드는 비동기 런타임과 프레임워크를 고르는 일부터 배워야 하고, 분야에 따라 다른 언어보다 라이브러리가 적을 수 있습니다(Q1.6, [ch16](../ch16-tooling-ecosystem/README.md)).

**더 읽기** Part 0 ch12, ch20 / Q1.4, Q1.6

### Q1.3 왜 여러 조직이 Rust로 옮기나요?

**짧은 답** 가장 큰 이유는 보안입니다. 큰 C/C++ 코드베이스에서 심각한 취약점의 70% 안팎이 메모리 안전성 문제였다는 보고가 거듭 나왔고, 새 코드를 메모리 안전한 언어로 쓰자 그 비율이 실제로 줄었다는 결과가 나왔습니다. 여기에 GC가 없어서 얻는 자원·지연 시간의 이점과, 컴파일러가 잡아 주는 만큼 리뷰와 롤백이 줄었다는 생산성 결과가 더해집니다. 미국 정부 기관들도 메모리 안전한 언어로 옮기라고 권고합니다.

**왜 그런가** 근거를 이유별로 나눕니다. 숫자마다 누가, 언제, 무엇을 센 것인지를 함께 적었습니다.

**1. 보안: 메모리 안전성 취약점**

| 근거 | 발표 | 무엇을 센 숫자인가 |
| --- | --- | --- |
| Microsoft 보안 대응 센터(MSRC) | 2019-07-16 | Microsoft가 해마다 CVE(공개된 보안 취약점마다 붙는 식별 번호)를 부여하는 취약점의 약 70%가 메모리 안전성 문제 |
| Chromium 프로젝트 | 날짜 표기 없음 | 2015년 이후 안정 채널에 영향을 준 심각도 높음·치명 보안 버그 912건 가운데 약 70%가 메모리 안전성 문제, 그 절반이 해제 후 사용 |
| Google Android 팀 | 2024-09-25 | Android 취약점 가운데 메모리 안전성 문제의 비율이 2019년 76%에서 2024년 24%로 줄었음. 기존 코드를 다시 쓰지 않고 새 코드를 메모리 안전한 언어로 쓴 결과라고 설명 |
| Google Android 팀 | 2025-11-13 | 2025년 자료에서 메모리 안전성 취약점이 처음으로 전체의 20% 아래로 내려감. Rust 코드의 메모리 안전성 취약점 밀도는 Android의 C/C++ 코드보다 1000배 넘게 낮음 |

메모리 버그가 왜 취약점이 되는지, "70%"를 어떻게 읽어야 하는지는 [ch2](../ch2-memory-safety/README.md)에서 자세히 다룹니다.

정부 기관의 권고도 같은 방향입니다. 미국 백악관 국가사이버국장실(ONCD)은 2024년 2월 26일 보고서 *Back to the Building Blocks*를 내며, 메모리 안전한 언어를 쓰면 취약점의 한 부류 전체를 막을 수 있다고 주장했습니다. 미국 사이버보안·인프라보안청(CISA)은 2023년 12월 *The Case for Memory Safe Roadmaps*에서 소프트웨어 제조사에 메모리 안전 로드맵을 세우라고 권고했고, 2025년 6월 24일에는 국가안보국(NSA)과 함께 메모리 안전한 언어 도입의 장애물과 해법을 다룬 지침을 냈습니다. 이 권고들은 특정 언어를 고르라고 하지 않고 메모리 안전한 언어 전반을 권합니다. 커널, 브라우저, 임베디드처럼 GC나 런타임을 두기 어려운 자리에서 C/C++를 대신할 메모리 안전한 언어를 찾을 때 Rust가 후보가 됩니다.

**2. 자원과 지연 시간: GC가 없어서**

| 근거 | 발표 | 내용 |
| --- | --- | --- |
| Discord | 2020-02-04 | Go로 쓴 "Read States" 서비스에서 몇 분마다 지연 시간이 튀었고, 원인을 Go의 메모리 모델과 GC로 진단했습니다. Rust로 다시 쓴 판에서는 지연 시간이 튀지 않았다고 보고했습니다. |
| Cloudflare | 2022-09-14 | NGINX 기반 프록시를 Rust로 만든 Pingora로 바꾼 뒤, 같은 트래픽에서 CPU를 약 70%, 메모리를 약 67% 덜 썼다고 보고했습니다. Rust를 고른 이유로 C가 할 수 있는 일을 성능을 잃지 않고 메모리 안전하게 할 수 있다는 점을 들었습니다. |

**3. 생산성: 컴파일러가 잡는 만큼 덜 고친다**

Google Android 팀의 2025년 글은 중간 크기 이상의 변경에서 Rust 변경의 롤백률이 C++보다 약 4배 낮고, Rust 변경이 코드 리뷰에 쓰는 시간은 C++보다 약 25% 짧다고 보고했습니다.

**오해와 한계**

- "70%"는 C/C++로 쓴 큰 코드베이스에서 보안 취약점(CVE, 심각도 높은 버그)을 센 비율입니다. 프로그램 버그 전체의 비율이 아니고, Java나 Go로 쓴 프로젝트에 그대로 옮겨 적용할 수 있는 숫자도 아닙니다.
- 위 숫자는 모두 각 조직이 자기 코드를 센 결과입니다.
- 다시 쓴 사례의 개선이 모두 언어 덕은 아닙니다. Cloudflare는 옛 프록시가 NGINX 위에서 Lua 코드를 돌리던 구조였다고 밝히며, CPU·메모리 절감이 Lua 대신 Rust 코드를 쓴 효율과 함께, 헤더를 복사하지 않고 요청 사이의 데이터를 스레드끼리 나누는 새 구조에서 왔다고 설명합니다. 지연 시간 단축은 코드가 빨라서가 아니라 스레드 사이에 연결을 공유하는 구조 덕분이라고 따로 밝힙니다. Discord도 자료구조를 `BTreeMap`으로 바꾸는 등의 최적화와 비동기 런타임 업그레이드를 함께 적었습니다. 두 글 모두 "같은 설계를 언어만 바꿔" 잰 결과가 아닙니다.
- 메모리 안전성 취약점이 줄어도 다른 종류의 취약점(논리 오류, 인증·권한 실수)은 남습니다. Android 팀도 Rust로 쓴 코드에서 버퍼 오버플로(CVE-2025-48530)가 나왔다가 공개 릴리스 전에 잡힌 일을 함께 보고했습니다([ch3](../ch3-safety-limits/README.md) Q3.8).

**더 읽기** [ch2](../ch2-memory-safety/README.md), [ch3](../ch3-safety-limits/README.md), [ch14](../ch14-performance/README.md)(성능 비교를 읽는 법), [ch17](../ch17-interop-migration/README.md)(도입 비용)

### Q1.4 실제로 어디에서 쓰이나요?

**짧은 답** 운영체제 커널(Linux), 모바일 운영체제(Android), 인터넷 인프라(Cloudflare의 프록시), 대규모 서비스(Discord)처럼 성능과 안정성이 함께 필요한 곳에서 씁니다. 2025년 12월 Linux 커널 메인테이너들은 커널의 Rust 지원이 더는 실험이 아니라고 합의했습니다.

**왜 그런가** 자료로 확인한 사례만 적습니다.

| 어디에서 | 무엇을 | 근거 |
| --- | --- | --- |
| Linux 커널 | 2025 Maintainers Summit에서 "Rust 실험"을 성공으로 끝내고, 커널의 Rust를 계속 이어 갈 핵심 부분으로 보기로 합의 | LWN 기사(2025-12-10), Miguel Ojeda의 패치 "rust: conclude the Rust experiment"(2025-12-13) |
| Android | 새 코드를 Rust 등 메모리 안전한 언어로 쓰는 방향으로 전환 | Google 보안 블로그(2024-09-25, 2025-11-13) |
| Cloudflare | 인터넷 트래픽을 중계하는 프록시 Pingora | Cloudflare 블로그(2022-09-14) |
| Discord | 메시지 읽음 상태를 관리하는 서비스 | Discord 블로그(2020-02-04) |

공통점이 있습니다. 모두 오래 돌아야 하고, 요청이 많고, 메모리 버그가 곧 보안 사고나 장애로 이어지는 코드입니다. 반대로 이 사례들이 Rust가 그 조직의 주 언어라는 뜻은 아닙니다. Android의 전환도 기존 C/C++ 코드를 두고 새 코드부터 바꾼 것입니다(Q1.7).

**오해와 한계** "누가 쓴다" 목록은 쉽게 낡고, 일부 팀이 일부 구성 요소에 쓰는 것과 전사 표준으로 쓰는 것을 구별하지 않습니다. 도입을 판단할 때는 사례의 이름보다 그 조직이 어떤 문제를 풀려고 했는지(Q1.3)를 보는 편이 쓸모 있습니다.

**더 읽기** Q1.3, Q1.7

### Q1.5 인기가 많다는데 얼마나 쓰이나요?

**짧은 답** "가장 사랑받는 언어"와 "많이 쓰는 언어"는 다른 숫자입니다. Stack Overflow 개발자 설문에서 Rust는 써 본 사람이 계속 쓰고 싶어 하는 비율(admired)이 2025년 72%로 1위였고 2026년에도 67%로 최상위권이었지만, 지난 1년 동안 Rust로 개발했다는 응답자는 2025년 14.8%, 2026년 18.0%로 JavaScript·Python·Java보다 훨씬 적습니다.

**왜 그런가** 두 숫자는 분모가 다릅니다. 설문 페이지의 정의에 따르면 admired는 지난 1년 동안 그 기술을 쓴 사람 가운데 계속 쓰고 싶다는 사람의 비율이고, 사용률은 언어 문항에 답한 사람 가운데 그 언어로 개발했다는 사람의 비율입니다.

| | 2025 | 2026 |
| --- | --- | --- |
| 설문 응답 수 | 49,009 | 30,903 |
| 언어 문항 응답 수(사용률의 분모) | 31,771 | 14,099 |
| Rust admired | 72% (1위. 다음은 Gleam 70%, Elixir 66%, Zig 64%) | 67% (Rust보다 높은 언어는 사용자가 수십 명 이하인 Elm 94%(17명 중 16명), Odin 82%(34명 중 28명)이고, 다음은 Elixir 62%, Zig 58%) |
| Rust 사용률(언어 문항 응답자 가운데) | 14.8% | 18.0% |
| 비교: JavaScript / Python / Java / C++ / Go 사용률 | 66% / 57.9% / 29.4% / 23.5% / 16.4% | 62.0% / 58.0% / 27.6% / 24.3% / 16.5% |

2026년 설문에서 Rust를 쓰고 싶다는 응답(desired)은 26%였습니다. 설문 응답 수는 각 연도의 Methodology 페이지가 밝힌, 결과에 쓰인 응답의 수입니다. 언어 문항에 답한 사람은 그보다 적으므로(2025년은 페이지에 적힌 응답 수, 2026년은 데이터 페이지의 `n` 값), 사용률은 언어 문항 응답자 가운데의 비율입니다. 2026년 admired 순위는 표본이 아주 작은 언어를 함께 놓으면 Rust가 1위가 아니므로, 몇 명이 답한 숫자인지 함께 봐야 합니다.

**오해와 한계**

- admired는 이미 Rust를 쓰는 사람, 즉 스스로 Rust를 고른 사람들 안에서의 비율입니다. 높은 admired는 "써 본 사람은 만족한다"는 뜻이지 "많이 쓴다"는 뜻이 아닙니다.
- 응답자는 Stack Overflow 설문에 응한 개발자입니다. 업계 전체를 무작위로 뽑은 표본이 아니므로 비율을 업계 전체의 비율로 읽지 않습니다.
- Rust 커뮤니티가 따로 하는 State of Rust 설문(2025년 7,156명 응답)은 이미 Rust를 쓰는 사람들의 경험을 묻는 설문이라 사용 현황의 근거가 아닙니다. 대신 쓰는 사람들이 꼽은 불편(Q1.6)을 보기에 좋습니다.
- 인기는 품질의 근거가 아닙니다. 고를 때는 Q1.6과 Q1.8의 기준을 봅니다.

**더 읽기** Q1.6, Q1.8

### Q1.6 Rust가 맞지 않는 곳은 어디인가요?

**짧은 답** 결과를 빨리 보는 것이 무엇보다 중요하고 성능·자원 제약이 약한 일(일회성 스크립트, 데이터 탐색, 빠른 프로토타입)에는 GC 언어가 더 빠른 길일 때가 많습니다. 어디에 쓰든 컴파일 시간, 배우는 비용, 팀의 경험, 필요한 라이브러리가 있는지를 함께 따져야 합니다.

**왜 그런가** Rust가 주는 것(GC 없는 메모리 안전성, 예측할 수 있는 성능, 컴파일 시간의 검사)은 공짜가 아닙니다.

| 비용 | 내용 | 더 읽기 |
| --- | --- | --- |
| 컴파일 시간과 디스크 | 2025 State of Rust 설문 결과에서도 느린 컴파일과 저장 공간 사용이 여전히 상위의 불편으로 꼽혔습니다. 컴파일러가 검사하고 최적화하는 일이 많고, 제네릭을 쓰인 타입마다 따로 컴파일(단형성화, monomorphization)하기 때문입니다. | [Part 3 ch6](../../part3-cargo/ch6-profiles-build-performance/README.md), [부록 A ch12](../../appd-a/ch12-generics-vs-trait-objects/README.md), [ch15](../ch15-pain-points/README.md) |
| 배우는 비용 | 소유권과 라이프타임은 다른 언어에 대응하는 개념이 없어 처음에 시간이 듭니다. 같은 설문에서 언어가 점점 복잡해진다는 걱정도 이어졌습니다. | [ch4](../ch4-memory-without-gc/README.md), [ch15](../ch15-pain-points/README.md) |
| 설계를 먼저 정해야 함 | 누가 데이터를 소유하고 누가 빌리는지를 코드에 적어야 하므로, 구조를 자주 뒤엎는 탐색 단계에서는 고칠 곳이 늘어납니다. | [ch15](../ch15-pain-points/README.md) |
| 라이브러리 | 분야에 따라 다른 언어보다 성숙한 라이브러리가 적을 수 있습니다. 필요한 크레이트가 있는지 먼저 확인합니다. | [ch16](../ch16-tooling-ecosystem/README.md) |
| 기존 코드와의 연결 | 큰 C++·Java 코드베이스와 섞으면 경계를 만드는 비용이 듭니다. | [ch17](../ch17-interop-migration/README.md) |

그래서 다음 조건이 많이 겹칠수록 Rust의 비용이 이득보다 커지기 쉽습니다(이 저장소의 정리이며, 자료에서 온 규칙은 아닙니다): 수명이 짧은 코드, 성능과 메모리가 병목이 아닌 일, 메모리 버그의 피해가 작은 일, Rust 경험자가 없는 팀, 필요한 라이브러리가 다른 언어에만 있는 경우.

**오해와 한계** "맞지 않는다"는 "못 쓴다"가 아닙니다. 작은 도구를 Rust로 쓰는 사람도 많고, 익숙해지면 프로토타입 속도도 달라집니다. 다만 Rust를 처음 도입하는 팀이라면 비용이 가장 크게 느껴지는 곳에서 시작하지 않는 편이 낫습니다(Q1.7).

**더 읽기** [ch15](../ch15-pain-points/README.md), [ch16](../ch16-tooling-ecosystem/README.md) / Q1.7, Q1.8

### Q1.7 기존 코드를 전부 다시 써야 하나요?

**짧은 답** 아닙니다. Google Android 팀은 기존 C/C++ 코드를 그대로 두고 새 코드를 메모리 안전한 언어로 쓰는 것만으로 메모리 안전성 취약점의 비율이 크게 줄었다고 보고했습니다. Rust는 C와 함수 단위로 이어 붙일 수 있어서 경계를 정해 조금씩 들일 수 있습니다.

**왜 그런가** Android 팀의 2024년 글은 그 근거로, 취약점이 코드가 오래될수록 지수적으로 줄어든다는(반감기가 있다는) 관찰을 듭니다. 취약점은 주로 새로 쓰거나 최근에 고친 코드에서 나오므로, 새 코드가 메모리 안전하면 오래된 코드를 다시 쓰지 않아도 전체 위험이 빠르게 줄어든다는 것입니다. 기술적으로는 Rust 함수를 C에서 부르고 C 함수를 Rust에서 부를 수 있습니다([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.4절, [부록 A ch34](../../appd-a/ch34-ffi-boundaries/README.md)).

**오해와 한계** 경계에는 비용이 있습니다. C와의 경계는 컴파일러가 검사하지 못하는 `unsafe` 코드이고, C++와는 C만큼 직접 이어지지 않습니다. 어디부터 옮길지, 어떤 도구를 쓸지는 [ch17](../ch17-interop-migration/README.md) Q17.7·Q17.2~Q17.4에서 다룹니다.

**더 읽기** [ch17](../ch17-interop-migration/README.md) Q17.1·Q17.7, 부록 A ch34

### Q1.8 Go, C++와는 어떤 관계이고 무엇을 고르나요?

**짧은 답** 셋 다 네이티브 실행 파일로 컴파일되는 언어지만, 메모리를 맡기는 곳이 다릅니다. Go는 GC에, C++는 프로그래머에게, Rust는 컴파일러의 소유권 검사에 맡깁니다. Go는 단순한 언어와 빠른 빌드로 쉽게 만들고 유지하는 쪽, C++는 수십 년의 코드와 생태계 위에서 최대한의 제어를 주는 쪽, Rust는 GC 없이 제어와 안전을 함께 얻는 대신 배우는 비용과 컴파일 시간을 치르는 쪽입니다.

**왜 그런가**

| | Go | C++ | Rust |
| --- | --- | --- | --- |
| 메모리 관리 | GC | 수동 관리, RAII, 스마트 포인터 | 소유권(컴파일 시간), 필요하면 `Rc`/`Arc` |
| 메모리 안전성 | 안전. 다만 데이터 경쟁이 있으면 메모리가 손상될 수 있다고 Go 메모리 모델 문서가 밝힘 | 프로그래머의 책임 | 안전한 코드에서 보장 |
| 데이터 경쟁을 찾는 방법 | 실행 중 도구(`-race`). 실행된 경로만 찾음 | 실행 중 도구(ThreadSanitizer 등) | 안전한 코드에서는 컴파일 에러(03) |
| 설계가 앞세우는 것 | 단순함과 빠른 빌드(Go FAQ) | 기존 코드와의 호환, 제어(이 저장소의 요약) | 안전성과 제어를 함께(이 저장소의 요약) |

표에서 Go에 관한 내용은 Go 공식 문서에서 확인했습니다. Go 메모리 모델 문서는 여러 워드로 된 값(인터페이스, 슬라이스, 문자열 등)에 대한 경쟁이 대부분의 Go 구현에서 임의의 메모리 손상으로 이어질 수 있다고 적고, 경쟁 검출기 문서는 검출기가 실행 중에 실제로 일어난 경쟁만 찾으며 메모리를 5~10배, 실행 시간을 2~20배 더 쓸 수 있다고 적습니다. Rust에서는 03 예제의 합계를 `Mutex` 없이 여러 스레드가 빌려 고치는 코드가 실행해 보기도 전에 컴파일되지 않습니다.

고를 때 물어볼 것은 다음과 같습니다(이 저장소의 정리입니다).

| 상황 | 먼저 검토할 언어 |
| --- | --- |
| 지연 시간이 고르게 낮아야 하거나 메모리가 빠듯함, 메모리 버그가 곧 보안 사고 | Rust |
| 팀 전체가 빨리 읽고 고쳐야 하는 네트워크 서비스, GC 멈춤이 문제 되지 않음 | Go |
| 큰 C++ 코드베이스나 C++ 전용 라이브러리에 깊게 묶여 있음 | C++를 유지하면서 새 구성 요소부터 Rust를 검토([ch12](../ch12-modern-cpp-enough/README.md), [ch17](../ch17-interop-migration/README.md)) |
| 커널, 임베디드, 브라우저 엔진처럼 GC를 둘 수 없음 | Rust 또는 C/C++. 메모리 안전성이 필요하면 Rust |

**오해와 한계** 언어 비교 표는 출발점일 뿐입니다. 실제 결정은 팀이 무엇을 잘 아는지, 어떤 라이브러리가 필요한지, 기존 코드와 어떻게 이어야 하는지에 더 크게 좌우됩니다. C++와의 더 자세한 비교는 [ch11](../ch11-cpp-raii-move/README.md)와 [ch12](../ch12-modern-cpp-enough/README.md)에서, Go를 포함한 동시성 모델의 비교는 [ch8](../ch8-concurrency-async/README.md)에서 합니다.

**더 읽기** [ch8](../ch8-concurrency-async/README.md), [ch11](../ch11-cpp-raii-move/README.md), [ch12](../ch12-modern-cpp-enough/README.md), [ch17](../ch17-interop-migration/README.md) / Q1.6

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_freed_without_gc](examples/01_freed_without_gc.rs) | 버퍼가 함수가 끝날 때마다, 일찍 돌아가는 길에서도 해제되어 살아 있는 버퍼가 최대 1개 | Q1.1, Q1.8 |
| [02_safety_without_overhead](examples/02_safety_without_overhead.rs) | `Option<&u8>`, `Option<Box<u64>>`, 뉴타입, 반복자 체인의 크기와 슬라이스 참조가 들고 다니는 길이 | Q1.1, Q1.8 |
| [03_checked_thread_sharing](examples/03_checked_thread_sharing.rs) | 스레드가 원본의 조각을 빌려 읽고, 함께 고치는 합계는 `Mutex`로 보호 | Q1.1, Q1.8 |

### 01_freed_without_gc — GC 없이, 정해진 때 해제

```sh
cargo run -p faq-ch1-why-rust --example 01_freed_without_gc
```

```text
  배치 1 처리 중: 살아 있는 버퍼 1개
  배치 1 해제 (1000000 바이트)
배치 1 합계 1000000, 살아 있는 버퍼 0개
  배치 2 처리 중: 살아 있는 버퍼 1개
  배치 2 해제 (1000000 바이트)
배치 2 합계 1000000, 살아 있는 버퍼 0개
  배치 3 처리 중: 살아 있는 버퍼 1개
  배치 3 해제 (1000000 바이트)
배치 3 실패: 배치 3의 형식이 잘못됨, 살아 있는 버퍼 0개
  배치 4 처리 중: 살아 있는 버퍼 1개
  배치 4 해제 (1000000 바이트)
배치 4 합계 1000000, 살아 있는 버퍼 0개
동시에 살아 있던 버퍼는 최대 1개
```

- `Batch`는 힙에 있는 `Vec<u8>`을 소유하고, `Drop`을 구현해 버려질 때 "해제"를 출력합니다. 살아 있는 버퍼 수는 원자 타입(atomic type)으로 만든 전역 카운터로 셉니다. 원자 타입은 여러 곳에서 안전하게 고칠 수 있는 정수로, Part 1 ch11에서 다룹니다.
- "해제" 줄이 `main`의 "합계" 줄보다 먼저 나옵니다. `process`가 반환값을 계산한 뒤 끝나면서 지역 변수 `batch`가 버려지기 때문입니다.
- 배치 3은 오류로 일찍 돌아가지만 출력은 다른 배치와 같습니다. 어느 길로 함수를 빠져나가든 `batch`는 버려집니다. C라면 `return`마다 `free`를 빠뜨리지 않았는지 사람이 확인해야 하는 자리입니다.
- 추적 GC를 쓰는 언어였다면 다 쓴 버퍼가 다음 수거 때까지 남을 수 있어, 같은 순간에 살아 있는 버퍼 수를 이렇게 정확히 말할 수 없습니다.

### 02_safety_without_overhead — 안전장치의 크기

```sh
cargo run -p faq-ch1-why-rust --example 02_safety_without_overhead
```

```text
[1] 널이 될 수 있음을 타입으로 드러내도 크기는 그대로
  &u8         8바이트  →  Option<&u8>         8바이트
  Box<u64>    8바이트  →  Option<Box<u64>>    8바이트
[2] 단위를 타입으로 구분해도 크기는 그대로
  f64         8바이트  →  Meters(f64)         8바이트
[3] 경계 검사에 쓰는 길이는 참조가 함께 들고 다님
  &u8         8바이트  →  &[u8]              16바이트
[4] 반복자 체인은 힙에 아무것도 만들지 않는 작은 값
  filter + map 체인의 크기: 16바이트
  체인의 합 190, 인덱스 반복문의 합 190
  거리 1.5m
```

- [1]: 참조와 `Box`는 0번지를 가리킬 수 없으므로, 컴파일러가 모든 비트가 0인 값을 `None`의 표시로 씁니다. 표준 라이브러리 문서(`std::option`의 "Representation")가 보장하는 성질이며 널 포인터 최적화(null pointer optimization)라고 부릅니다. C의 널 포인터와 크기가 같지만, `Option<&u8>`를 `&u8`처럼 바로 쓸 수는 없고 `match`·`if let`·`unwrap`(`None`이면 패닉)처럼 `None`을 다루는 길을 거쳐야 합니다.
- [2]: `Meters`는 기존 타입 하나를 감싸 새 타입을 만드는 뉴타입(newtype, Part 0 ch19 2.12절)입니다. `Meters`와 `f64`를 섞어 더하는 실수는 컴파일 에러가 되지만, 실행 중에는 그냥 `f64`입니다.
- [3]: 슬라이스 참조 `&[u8]`는 주소와 길이를 함께 가집니다. C에서 포인터와 길이를 따로 넘기던 것을 한 값으로 묶은 것이고, 인덱싱할 때 이 길이로 경계를 검사합니다. 비용이 드는 곳입니다.
- [4]: 반복자 체인은 원래 배열을 가리키는 반복자(주소 두 개)와 캡처한 것이 없어 크기가 0인 클로저 둘로 이뤄집니다. 인덱스 반복문(Clippy의 `needless_range_loop` 경고를 `#[expect]`로 표시한 함수)과 결과가 같습니다. 속도까지 같은지는 최적화를 켠 빌드에서 재야 합니다(Part 0 ch13 2.15절).
- 크기는 64비트 플랫폼 기준입니다. 32비트 플랫폼에서는 포인터가 4바이트라 숫자가 달라집니다.

### 03_checked_thread_sharing — 스레드 공유의 검사

```sh
cargo run -p faq-ch1-why-rust --example 03_checked_thread_sharing
```

```text
1. 읽기만 하는 공유: 조각을 빌려 읽고 결과를 돌려줌
  조각별 단어 수 [3, 7, 11, 15], 합계 36
  스레드가 끝난 뒤에도 원본은 그대로: 8줄
2. 함께 고치는 공유: 합계 하나를 Mutex로 보호
  합계 36
```

- `thread::scope`는 스코프가 끝나기 전에 안에서 만든 스레드가 모두 끝난다고 보장합니다. 그래서 스레드가 지역 변수 `lines`를 `Arc`나 복사 없이 빌릴 수 있습니다. Part 0 ch15 2.11절이 [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md)로 넘긴 내용입니다.
- `chunks(2)`는 원본을 복사하지 않고 두 줄씩 가리키는 슬라이스를 만듭니다. 스레드마다 다른 조각을 읽기만 하므로 동기화가 필요 없습니다.
- 결과는 스레드가 돌려준 값을 `main`에서 모아 출력하므로, 스레드가 끝나는 순서와 상관없이 출력은 실행마다 같습니다.
- 합계를 `Mutex` 없이 고치면 어떻게 되는지는 파일 끝의 주석에 적었습니다. 빌려서 고치면 E0499(``cannot borrow `total` as mutable more than once at a time``)로 컴파일되지 않고(같은 에러를 내는 파일은 [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 4.1절의 `compile_fail/e0499_scoped_threads_mut_borrow.rs`), `move` 클로저로 고치면 `usize` 값이 스레드마다 복사되어 합계가 0으로 남습니다(컴파일러가 ``value captured by `total` is never read`` 경고를 냅니다). 어느 쪽이든 두 스레드가 같은 메모리를 동시에 고치는 데이터 경쟁은 생기지 않습니다. 자세한 것은 [ch8](../ch8-concurrency-async/README.md)에서 다룹니다.

## 4. 자주 하는 실수

이 챕터의 실수는 컴파일 에러가 아니라 Rust를 평가하는 방식에서 나오므로 `compile_fail/`을 두지 않았습니다. 컴파일러가 거부하는 코드는 [ch2](../ch2-memory-safety/README.md)(메모리)와 [ch8](../ch8-concurrency-async/README.md)(스레드)에서 봅니다.

| 실수 | 왜 틀렸나 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| 디버그 빌드로 다른 언어와 속도를 비교 | 디버그 빌드는 최적화를 끄고 오버플로 검사 등을 켭니다 | `--release`로 빌드해 비교합니다 | Part 0 ch13 2.15절, [ch14](../ch14-performance/README.md) |
| "가장 사랑받는 언어"를 "가장 많이 쓰는 언어"로 읽음 | admired는 Rust를 쓰는 사람 안에서의 비율입니다 | 사용률을 따로 봅니다 | Q1.5 |
| "취약점의 70%"를 모든 프로젝트에 적용 | C/C++ 대형 코드베이스의 보안 취약점을 센 숫자입니다 | 누가 무엇을 셌는지 확인합니다 | Q1.3, [ch2](../ch2-memory-safety/README.md) |
| "Rust로 쓰면 버그가 없다"고 기대 | 보장하는 것은 안전한 코드의 메모리 안전성과 데이터 경쟁 없음입니다 | 테스트와 리뷰는 그대로 필요합니다 | Q1.1, [ch3](../ch3-safety-limits/README.md) |
| 다시 쓴 사례의 개선을 모두 언어 덕으로 봄 | 사례 대부분이 구조도 함께 바꿨습니다 | 무엇을 함께 바꿨는지 원문에서 확인합니다 | Q1.3 |
| 도입을 "전부 다시 쓰기"로 생각 | 새 코드부터 들이는 방법이 있습니다 | 경계를 정해 조금씩 들입니다 | Q1.7, [ch17](../ch17-interop-migration/README.md) |
| 03의 합계를 `move` 클로저로 고치고 경고를 무시 | `Copy` 값은 스레드마다 복사되어 원래 변수는 그대로입니다 | 경고를 읽고 `Mutex`나 반환값으로 모읍니다 | 03 |

## 5. 연습 문제

### 연습 1. 버퍼를 모아 두면

01의 `main`에서 `process`가 `Batch`를 버리지 않고 돌려주게 바꾼 뒤 `Vec<Batch>`에 모아 두면, "살아 있는 버퍼" 수와 "해제" 줄이 나오는 위치가 어떻게 바뀔지 먼저 예상하고 실행해 확인해 보세요.

<details>
<summary>힌트</summary>

`process`의 반환 타입을 `Result<Batch, String>`으로 바꾸고 `main`에서 `let mut kept = Vec::new();`에 `push`합니다. 값이 `Vec`으로 이동하면 소유자가 `Vec`이 되므로, 해제는 `kept`가 버려지는 `main`의 끝에서 한꺼번에 일어납니다. 오류로 끝난 배치 3만 `process` 안에서 해제되므로, 살아 있는 버퍼는 최대 3개가 됩니다.

</details>

### 연습 2. `Option`이 커지는 경우

02에 `Option<f64>`와 `Option<Meters>`의 크기를 출력하는 줄을 더해 보세요. 결과가 `Option<&u8>`와 다른 이유를 설명해 보세요.

<details>
<summary>힌트</summary>

둘 다 16바이트가 나옵니다. `f64`는 모든 비트 패턴이 유효한 값(0.0을 포함)이라 `None`을 표시할 남는 값이 없으므로, 어느 쪽인지 표시하는 공간이 따로 붙습니다. 참조는 0번지를 가리킬 수 없어서 남는 값이 있었습니다.

</details>

### 연습 3. 우리 팀이라면

지금 일하는 팀의 서비스나 도구 하나를 골라 "Rust로 새로 쓰거나 옮길 가치가 있는가"를 동료에게 설명한다고 해 보세요. Q1.3의 세 가지 이유와 Q1.6의 비용 표를 기준으로, 판단에 필요한 질문 다섯 개를 적어 보세요.

<details>
<summary>힌트</summary>

예: 이 코드가 C/C++인가, 메모리 버그가 보안 사고로 이어지는가, GC 멈춤이나 메모리 사용이 문제인 적이 있었는가, 이 코드는 얼마나 오래 쓰이는가, 필요한 라이브러리가 crates.io에 있는가, 팀에 Rust를 아는 사람이 있는가, 기존 코드와 어떤 경계로 이어야 하는가. 답이 "예"인 질문이 Q1.3 쪽에 많을수록, "아니요"인 질문이 Q1.6 쪽에 많을수록 검토할 가치가 커집니다.

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

공식 문서

- [rust-lang.org](https://www.rust-lang.org/): "Why Rust?"(성능, 신뢰성, 생산성)와 "Build it in Rust"(명령줄, WebAssembly, 네트워킹, 임베디드)
- [Announcing Rust 1.0](https://blog.rust-lang.org/2015/05/15/Rust-1.0/) (Rust 블로그, 2015-05-15)
- [The Rust Programming Language 4장: Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html), [16장: Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [`std::option`의 Representation](https://doc.rust-lang.org/std/option/index.html#representation): `Option<&T>`, `Option<Box<T>>`의 크기 보장

보안

- [A proactive approach to more secure code](https://www.microsoft.com/en-us/msrc/blog/2019/07/a-proactive-approach-to-more-secure-code) (Microsoft MSRC, 2019-07-16)
- [Memory safety](https://www.chromium.org/Home/chromium-security/memory-safety/) (Chromium 프로젝트, 날짜 표기 없음)
- [Eliminating Memory Safety Vulnerabilities at the Source](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html) (Google 보안 블로그, 2024-09-25)
- [Rust in Android: move fast and fix things](https://blog.google/security/rust-in-android-move-fast-fix-things/) (Google, 2025-11-13)
- [Press Release: Future Software Should Be Memory Safe](https://bidenwhitehouse.archives.gov/oncd/briefing-room/2024/02/26/press-release-technical-report/) (미국 ONCD, 2024-02-26. 보고서 *Back to the Building Blocks*)
- [The Case for Memory Safe Roadmaps](https://www.cisa.gov/resources-tools/resources/case-memory-safe-roadmaps) (미국 CISA, 2023-12-06)
- [Memory Safe Languages: Reducing Vulnerabilities in Modern Software Development](https://www.cisa.gov/resources-tools/resources/memory-safe-languages-reducing-vulnerabilities-modern-software-development) (CISA·NSA, 2025-06-24)

도입 사례

- [Why Discord is switching from Go to Rust](https://discord.com/blog/why-discord-is-switching-from-go-to-rust) (Discord, 2020-02-04)
- [How we built Pingora, the proxy that connects Cloudflare to the Internet](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/) (Cloudflare, 2022-09-14)
- [The (successful) end of the kernel Rust experiment](https://lwn.net/Articles/1049831/) (LWN, 2025-12-10), [rust: conclude the Rust experiment](https://lwn.net/Articles/1050308/) (Miguel Ojeda의 패치, 2025-12-13)

설문

- [Stack Overflow Developer Survey 2025: Technology](https://survey.stackoverflow.co/2025/technology), [Methodology](https://survey.stackoverflow.co/2025/methodology) (발행일 표기 없음)
- [Stack Overflow Developer Survey 2026: 언어 사용률](https://survey.stackoverflow.co/2026/technology/data/languages), [Admired and Desired](https://survey.stackoverflow.co/2026/technology/data/da-language), [Methodology](https://survey.stackoverflow.co/2026/methodology) (발행일 표기 없음, 조사 기간 2026-06-23~08-05)
- [2025 State of Rust Survey Results](https://blog.rust-lang.org/2026/03/02/2025-State-Of-Rust-Survey-results) (Rust 블로그, 2026-03-02)

다른 언어

- [Go FAQ](https://go.dev/doc/faq), [The Go Memory Model](https://go.dev/ref/mem), [Data Race Detector](https://go.dev/doc/articles/race_detector)
- [Python `gc` 모듈](https://docs.python.org/3/library/gc.html): GC가 참조 카운팅을 보조한다는 설명
