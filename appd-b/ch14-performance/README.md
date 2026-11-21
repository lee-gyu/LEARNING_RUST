# ch14. 성능: 빠르다는 말을 읽는 법

"Rust는 C만큼 빠르다"는 말은 자주 듣지만, 무엇을 어떻게 재서 나온 말인지는 잘 따지지 않습니다. 이 챕터는 Rust 프로그램의 속도와 크기를 정하는 것이 무엇인지, 느릴 때는 어디를 먼저 보는지, 벤치마크와 도입 사례를 어떻게 읽는지 답합니다. 다른 언어와의 속도 수치는 직접 만들지 않습니다. 대신 컴파일러가 실제로 만든 코드와 할당 횟수처럼 이 환경에서 다시 확인할 수 있는 것을 봅니다.

- 패키지: `faq-ch14-performance`
- 확인한 툴체인: rustc 1.100.0(LLVM 23.1.5), cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 비교 코드를 확인한 컴파일러: Apple clang 21.0.0(C17), macOS(aarch64)
- 자료 확인일: 2026-11-21
- Miri가 없어 예제 02의 `unsafe` 블록(시스템 할당기에 그대로 넘기는 코드)은 Miri로 검사하지 못했습니다.

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q14.1](#q141-rust는-정말-cc만큼-빠른가요) | Rust는 정말 C/C++만큼 빠른가요? | 같은 LLVM 백엔드로 기계어를 만들고 GC가 없으므로 같은 수준에 있습니다. 경계 검사 같은 비용이 있는 대신, `&mut`의 별칭 정보처럼 C가 `restrict` 없이는 주지 못하는 정보를 줍니다. |
| [Q14.2](#q142-비용-없는-추상화는-무슨-뜻인가요) | "비용 없는 추상화"는 무슨 뜻인가요? | 쓰지 않는 기능에는 비용을 내지 않고, 쓰는 기능은 손으로 짠 코드보다 느리지 않게 만든다는 원칙입니다. 반복자와 제네릭이 대표적이고, `dyn`이나 `Rc`처럼 비용이 드러나는 추상화도 있습니다. |
| [Q14.3](#q143-경계-검사와-오버플로-검사는-느리지-않나요-끌-수-있나요) | 경계 검사와 오버플로 검사는 느리지 않나요? 끌 수 있나요? | 오버플로 검사는 릴리스 빌드에서 기본으로 꺼집니다. 경계 검사는 끄는 옵션이 없지만, 컴파일러가 증명할 수 있으면 지우므로 반복자나 슬라이스 표현으로 바꾸는 것이 먼저입니다. |
| [Q14.4](#q144-왜-제-rust-프로그램이-느린가요) | 왜 제 Rust 프로그램이 느린가요? | 가장 흔한 원인은 디버그 빌드로 잰 것입니다. 그다음은 줄마다 비우는 표준 출력, 반복문 안의 할당과 복제, 버퍼 없는 입출력입니다. |
| [Q14.5](#q145-런타임이-있나요-gc-멈춤이나-시작-지연은-없나요) | 런타임이 있나요? GC 멈춤이나 시작 지연은 없나요? | GC와 가상 머신이 없어 GC 멈춤과 JIT 예열이 없습니다. 표준 라이브러리가 넣는 작은 시작 코드와 패닉·스택 오버플로 처리만 있고, 비동기 런타임은 표준 라이브러리 밖에 있습니다. |
| [Q14.6](#q146-go나-java보다-얼마나-빠른가요-벤치마크를-읽는-법) | Go나 Java보다 얼마나 빠른가요? (벤치마크를 읽는 법) | 한 숫자로 답할 수 없습니다. 빌드 설정, 같은 알고리즘인지, 무엇을 쟀는지, 구조도 함께 바꿨는지를 확인하고 읽어야 합니다. |
| [Q14.7](#q147-바이너리가-왜-크고-어떻게-줄이나요) | 바이너리가 왜 크고 어떻게 줄이나요? | 표준 라이브러리와 의존성을 실행 파일에 정적으로 넣고, 제네릭을 타입마다 펼치기 때문입니다. 프로필의 `opt-level`, `lto`, `strip`, `panic`, `codegen-units`로 줄입니다. |
| [Q14.8](#q148-hashmap은-왜-기본-해시-함수가-느린-편인가요) | `HashMap`은 왜 기본 해시 함수가 느린 편인가요? | 기본값(현재 SipHash 1-3)이 HashDoS 공격을 막도록 고른 것이기 때문입니다. 공격을 걱정하지 않아도 되면 맵마다 다른 해시 함수로 바꿀 수 있습니다. |
| [Q14.9](#q149-꼬리-호출-최적화가-있나요) | 꼬리 호출 최적화가 있나요? | 보장되지 않습니다. 최적화가 반복문으로 바꿔 주는 경우가 있을 뿐이라, 깊이가 입력에 따라 커지는 재귀는 반복문으로 씁니다. |
| [Q14.10](#q1410-메모리는-얼마나-쓰나요) | 메모리는 얼마나 쓰나요? | 값의 크기와 배치가 타입으로 정해지고 GC를 위한 여유분이 없어 예측하기 쉽습니다. 대신 `Vec`의 남는 용량, 할당 횟수처럼 코드가 정하는 부분은 직접 챙겨야 합니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.15절: 반복자와 반복문의 성능
- [Part 3 ch6](../../part3-cargo/ch6-profiles-build-performance/README.md): 프로필, 실행 파일 크기와 실행 시간 비교
- [부록 A ch20](../../appd-a/ch20-avoid-over-optimizing/README.md): 복사를 세고, 릴리스 빌드와 `black_box`로 재는 법
- [ch1](../ch1-why-rust/README.md) Q1.3: Discord·Cloudflare 사례를 읽는 법

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Rust와 C가 같은 백엔드에서 어떻게 다른 정보를 주는지(`&mut`의 `noalias`와 C의 `restrict`) LLVM IR로 확인하고, 성능 비교를 읽을 때 따질 것을 말할 수 있다.
2. 느린 Rust 프로그램의 흔한 원인을 찾고, 전역 할당기로 할당 횟수를 세어 반복문 안의 할당을 줄일 수 있다.
3. 꼬리 호출 최적화가 보장되지 않는다는 것과, `Drop`이 있는 값이 꼬리 호출을 막는 이유를 설명하고 깊은 재귀를 반복문으로 바꿀 수 있다.
4. 런타임, 바이너리 크기, 메모리 사용량, `HashMap`의 해시 함수에서 Rust가 무엇을 기본값으로 정했고 어떻게 바꾸는지 말할 수 있다.

## 2. 핵심 개념

### Q14.1 Rust는 정말 C/C++만큼 빠른가요?

**짧은 답** 같은 수준에 있습니다. rustc와 clang은 둘 다 LLVM이라는 같은 컴파일러 백엔드(최적화와 기계어 생성을 맡는 부분)로 기계어를 만들고, Rust에는 GC도 가상 머신도 없습니다. 차이는 각 언어가 컴파일러에 주는 정보와 넣는 검사에서 생깁니다. Rust는 경계 검사(bounds check) 같은 비용을 더하는 대신, `&mut`가 다른 참조와 겹치지 않는다는(별칭(aliasing)이 없다는) 정보를 모든 함수에 줍니다. C는 프로그래머가 `restrict`를 붙여야 같은 정보를 줍니다.

**왜 그런가** 이 툴체인의 `rustc -vV`는 `LLVM version: 23.1.5`를 출력합니다. 같은 백엔드라면 같은 정보를 받았을 때 같은 코드가 나옵니다. 컴파일러가 LLVM에 넘기는 코드를 IR(중간 표현, intermediate representation)이라 하고, `--emit=llvm-ir`로 볼 수 있습니다. 아래에서 `load`는 메모리에서 읽기, `store`는 쓰기, `shl … 1`은 왼쪽으로 1비트 밀기(두 배)입니다. [03_noalias](examples/03_noalias.rs)의 `add_twice(a: &mut i32, b: &i32)`는 `*a`에 `*b`를 두 번 더합니다. `rustc -C opt-level=3 --emit=llvm-ir`로 본 IR입니다(메타데이터 생략).

```text
define dso_local void @add_twice(ptr noalias … %a, ptr noalias … readonly … %b) …
  %_3 = load i32, ptr %b, align 4
  %0 = load i32, ptr %a, align 4
  %reass.add = shl i32 %_3, 1
  %1 = add i32 %0, %reass.add
  store i32 %1, ptr %a, align 4
```

두 매개변수에 `noalias`가 붙었고, `*b`를 한 번 읽어 두 배(`shl … 1`)로 만든 뒤 한 번 더하고 한 번 씁니다. C로 같은 함수를 쓴 [compare/02_aliasing.c](compare/02_aliasing.c)를 `cc -O3 -S -emit-llvm`으로 보면, `restrict`가 없는 `add_twice`는 `*b`를 두 번 읽고 `*a`에 두 번 씁니다. `*a`에 쓴 값이 `*b`를 바꿨을 수도 있기 때문입니다. `restrict`를 붙인 `add_twice_restrict`는 Rust와 같은 모양(load 한 번, `shl`, store 한 번)이 됩니다. `&mut`가 겹치지 않는다는 규칙은 [ch10](../ch10-undefined-behavior/README.md) Q10.7에서 봤고, 여기서는 그 규칙이 실제로 코드 생성에 쓰이는 모습을 본 것입니다.

**다른 언어에서는** C의 `restrict`는 프로그래머의 약속이라, 한쪽 `restrict` 포인터로 고친 객체를 다른 포인터로 읽거나 쓰면 미정의 동작입니다(이 함수에 같은 변수를 두 번 넘기면 그렇게 됩니다). C++ 표준에는 `restrict`가 없어, 이 환경의 `c++ -std=c++20`은 `restrict`를 `error: expected ')'`로 거부하고 컴파일러 확장 `__restrict`는 받았습니다. Rust에서는 안전한 코드가 겹치는 `&mut`를 만들 수 없으므로 같은 정보를 위험 없이 줍니다. 반대로 C에는 Rust가 기본으로 넣는 경계 검사가 없습니다.

**오해와 한계** IR 한 조각이 "Rust가 더 빠르다"는 뜻은 아닙니다. 실제 프로그램의 속도는 알고리즘, 자료구조, 할당, 입출력이 대부분을 정하고, 같은 일을 하는 C와 Rust 코드는 보통 비슷한 범위에 있습니다. 속도를 비교하려면 같은 알고리즘을 릴리스 빌드로 재야 합니다(Q14.6).

**더 읽기** ch10 Q10.7 / Q14.3 / Q14.6

### Q14.2 "비용 없는 추상화"는 무슨 뜻인가요?

**짧은 답** 비용 없는 추상화(zero-cost abstraction)는 추상화를 쓰지 않으면 그 비용을 내지 않고, 쓰면 같은 일을 손으로 짠 코드보다 느리지 않게 컴파일된다는 원칙입니다. 반복자 체인이 손으로 쓴 반복문과 같은 코드로 바뀌고, 제네릭이 타입마다 펼쳐져 실행 중에 타입을 따지지 않는 것이 대표적입니다. 모든 추상화가 공짜는 아닙니다. `dyn Trait`의 간접 호출, `Rc`의 참조 수 세기, `RefCell`의 실행 중 대여 검사에는 비용이 있습니다.

**왜 그런가** The Book 13.4절은 반복자로 쓴 검색과 `for` 반복문으로 쓴 검색을 비교하며 이 원칙을 소개하고, 그 내용은 Part 0 ch13 2.15절에서 봤습니다. 제네릭이 타입마다 코드를 만드는 단형성화는 [ch6](../ch6-no-classes-inheritance/README.md) Q6.6에서, 그 대가(컴파일 시간과 실행 파일 크기)와 트레이트 객체를 고르는 기준은 [부록 A ch12](../../appd-a/ch12-generics-vs-trait-objects/README.md)에서 봤습니다. `Option<&T>`가 포인터 하나 크기라 "없음"을 표현하는 데 공간을 더 쓰지 않는 것도 같은 원칙입니다([Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 예제 02).

**다른 언어에서는** 이 생각은 C++의 zero-overhead 원칙에서 왔고(The Book 13.4절은 C++를 만든 Bjarne Stroustrup의 설명을 인용함), C++의 템플릿과 인라인 함수도 같은 원칙을 따릅니다. 인터페이스나 가상 함수처럼 실행 중에 호출 대상을 고르는 추상화는 어느 언어에서나 간접 호출 비용이 있고, Rust에서는 그것이 `dyn`으로 코드에 드러납니다.

**오해와 한계** "비용 없음"은 실행 시간에 대한 말입니다. 단형성화는 컴파일 시간과 실행 파일 크기를 늘리고(Q14.7), 최적화를 끈 디버그 빌드에서는 반복자 체인이 손으로 쓴 반복문보다 훨씬 느릴 수 있습니다. 원칙은 최적화가 켜져 있을 때의 이야기입니다.

**더 읽기** Part 0 ch13 2.15절 / ch6 Q6.6 / 부록 A ch12

### Q14.3 경계 검사와 오버플로 검사는 느리지 않나요? 끌 수 있나요?

**짧은 답** 정수 오버플로 검사는 릴리스 프로필에서 기본으로 꺼져 있습니다(`overflow-checks = false`, 그때 오버플로는 래핑). 배열·슬라이스의 경계 검사는 끄는 컴파일러 옵션이 없습니다. 대신 컴파일러가 인덱스가 범위 안이라고 증명할 수 있으면 검사를 지우므로, 반복자나 `chunks`·`windows` 같은 슬라이스 메서드로 쓰는 것이 먼저입니다. 그래도 남은 검사가 측정으로 문제가 되면 `get_unchecked` 같은 `unsafe` 메서드를 씁니다.

**왜 그런가** 디버그와 릴리스에서 오버플로가 어떻게 다르게 처리되는지는 [ch3](../ch3-safety-limits/README.md) Q3.6에서, 프로필의 `overflow-checks` 설정은 Part 3 ch6에서 봤습니다. 경계 검사가 실제로 남는지는 어셈블리로 확인할 수 있습니다. [부록 A ch16](../../appd-a/ch16-avoid-unsafe/README.md) 2.3절은 같은 계산을 인덱싱으로 쓴 판과 `array_windows`·`as_chunks`로 쓴 판을 `--emit asm`으로 비교했고, 손으로 계산한 인덱스(`alts[g * GROUP + k]`)에만 `panic_bounds_check` 호출이 남았습니다. rustc의 코드 생성 옵션(The rustc book의 Codegen Options)에는 경계 검사를 끄는 옵션이 없습니다.

**다른 언어에서는** C와 C++의 `operator[]`는 경계를 검사하지 않고, C++의 `at()`이나 표준 라이브러리 하드닝([ch12](../ch12-modern-cpp-enough/README.md) Q12.4)으로 검사를 켭니다. Go와 Java는 Rust처럼 항상 검사합니다(Go 명세는 범위 밖 인덱스를 실행 중 패닉으로, Java 언어 명세는 `ArrayIndexOutOfBoundsException`으로 정함). 다만 Go의 gc 컴파일러에는 경계 검사를 끄는 옵션(`go tool compile`의 `-B`, `go build -gcflags=-B`)이 있습니다.

**오해와 한계** 검사 비용은 대개 작지만 0은 아닙니다. 촘촘한 반복문에서 남은 검사는 벡터화(여러 원소를 한 명령으로 처리하는 SIMD 코드로 바꾸는 최적화)를 막기도 합니다. 다만 "검사가 느릴 것"이라는 짐작으로 `unsafe`를 쓰기 전에, 검사가 실제로 남는지 보고 측정해야 합니다(부록 A ch16·ch20). 오버플로 검사를 릴리스에서도 켜고 싶으면 프로필에 `overflow-checks = true`를 둡니다.

**더 읽기** ch3 Q3.6 / [ch9](../ch9-c-memory-errors/README.md) Q9.11 / 부록 A ch16 2.3절 / Part 3 ch6

### Q14.4 왜 제 Rust 프로그램이 느린가요?

**짧은 답** 가장 흔한 원인은 디버그 빌드로 실행한 것입니다(`cargo run`은 기본이 디버그). 그다음은 줄마다 비우는 표준 출력에 많이 쓰기, 반복문 안의 할당과 `clone()`, 버퍼 없는 파일 읽기·쓰기입니다. 원인은 짐작하지 말고 릴리스 빌드에서 재고, 할당처럼 셀 수 있는 것은 세어 봅니다.

**왜 그런가** 흔한 원인과 확인하는 곳을 모았습니다.

| 원인 | 왜 느린가 | 확인하는 곳 |
| --- | --- | --- |
| 디버그 빌드 | 최적화가 꺼지고 오버플로 검사 등이 켜집니다 | `--release`로 다시 잽니다(ch1 4절, Part 0 ch13 2.15절) |
| `println!`을 많이 부름 | 표준 출력은 터미널에 연결되면 줄 단위로 비우고(std `Stdout` 문서), 호출마다 잠금을 잡습니다 | `stdout().lock()`을 `BufWriter`로 감쌉니다([Part 1 ch9](../../part1-standard-lib/ch9-io-fs-path/README.md) 예제 03) |
| 반복문 안의 할당 | 할당은 함수 호출보다 비쌉니다 | 할당을 세어 봅니다(아래, 예제 02) |
| 불필요한 `clone()` | 복제마다 할당과 복사가 일어날 수 있습니다 | 복사를 세어 봅니다(부록 A ch20 예제 02) |
| 버퍼 없는 파일 입출력 | 작은 읽기·쓰기마다 시스템 호출을 합니다 | `BufReader`·`BufWriter`(Part 1 ch9) |
| 정수 키가 많은 `HashMap` | 기본 해시 함수는 작은 키에서 가장 빠른 것이 아닙니다 | Q14.8 |

[02_count_allocations](examples/02_count_allocations.rs)는 전역 할당기(global allocator, 프로그램의 모든 힙 할당을 맡는 할당기)를 감싸 할당과 재할당을 셉니다.

```text
1. Vec에 1000개 넣기
  Vec::new: 할당 1, 재할당 8
  len 1000, capacity 1024
  Vec::with_capacity: 할당 1, 재할당 0
2. 문자열 100조각 잇기
  + format!: 할당 101, 재할당 6
  write!: 할당 1, 재할당 6
  두 문자열이 같은가: true
```

용량을 모르고 시작한 `Vec`은 커질 때마다 재할당했고, `with_capacity`로 미리 잡으면 한 번으로 끝났습니다. 반복문 안의 `format!`은 반복마다 새 `String`을 만들어 할당이 100번 더 일어났고, 같은 버퍼에 `write!`로 이어 쓰면 그 할당이 사라졌습니다(결과 문자열은 같음).

**다른 언어에서는** GC 언어에서도 반복문 안의 할당은 비용이고(GC의 일이 늘어남), Java의 `StringBuilder`나 Go의 `strings.Builder`가 위의 `write!`와 같은 역할을 합니다. 차이는 Rust에서는 할당이 일어나는 자리가 `String::new`, `format!`, `clone()`, `collect()`처럼 코드에서 비교적 잘 보인다는 점입니다.

**오해와 한계** 할당 횟수와 재할당 횟수는 이 툴체인의 값입니다. `Vec`이 얼마씩 커지는지는 문서가 정하지 않으므로 바뀔 수 있습니다. 횟수가 줄었다고 꼭 빨라지는 것은 아니니, 문제가 되는 곳인지는 재서 판단합니다(부록 A ch20). 이 예제의 할당기는 `unsafe impl GlobalAlloc`이지만 시스템 할당기에 그대로 넘기기만 하므로, `unsafe`가 맡는 책임(ch3 Q3.2)은 넘긴 함수들의 계약을 그대로 지키는 것입니다.

**더 읽기** ch1 4절 / Part 1 ch9 / 부록 A ch20 / Reference "The Rust runtime"(`#[global_allocator]`)

### Q14.5 런타임이 있나요? GC 멈춤이나 시작 지연은 없나요?

**짧은 답** GC도 가상 머신도 없으므로 GC 멈춤과 JIT 예열이 없습니다. 표준 라이브러리가 `main` 앞뒤에 작은 일(인자 준비, 패닉 처리, 스택 오버플로 감지 등)을 넣을 뿐이고, 실행 파일은 바로 기계어로 시작합니다. 비동기 코드를 돌리는 런타임(tokio 등)은 표준 라이브러리 밖에 있어, 쓰는 프로그램만 그 비용을 냅니다.

**왜 그런가** Reference의 "The Rust runtime" 절이 다루는 것은 전역 할당기(`#[global_allocator]`) 같은 몇 가지 속성뿐입니다. 표준 라이브러리가 넣는 장치의 흔적은 예제에서도 보입니다. [01_recursion_depth](examples/01_recursion_depth.rs)에서 스택 오버플로(stack overflow)가 났을 때 `thread 'recursion' (번호) has overflowed its stack`과 `fatal runtime error: stack overflow`를 출력한 것은, 스택 끝의 보호 영역(guard page)에 닿아 시그널이 나자 표준 라이브러리가 미리 설치해 둔 시그널 처리기가 그것이 스택 오버플로임을 알아보고 출력한 결과입니다. GC 멈춤이 없다는 것이 실제 서비스에서 어떤 차이를 냈는지는 Discord 사례로 [ch4](../ch4-memory-without-gc/README.md) Q4.9와 ch1 Q1.3에서 봤습니다.

**다른 언어에서는** Java와 C#은 가상 머신과 JIT, GC가, Go는 GC와 고루틴 스케줄러가 실행 파일과 함께 돕니다. 그 덕에 얻는 편의(메모리 관리, 동적 최적화)가 있고, 대가로 시작 시간, 메모리 여유분, GC 멈춤이 생깁니다. 멈춤의 길이는 GC의 종류와 설정에 따라 크게 다르므로 차이의 크기는 서비스마다 다릅니다.

**오해와 한계** GC가 없다고 지연 시간이 저절로 고르지는 않습니다. 큰 자료구조를 한꺼번에 버리면 그 자리에서 해제 비용이 몰리고(ch4 Q4.9), 할당기와 운영체제의 동작도 지연에 영향을 줍니다. 비동기 런타임을 쓰면 그 런타임의 스케줄링도 성능의 일부가 됩니다([ch8](../ch8-concurrency-async/README.md)).

**더 읽기** ch4 Q4.9 / ch8 / ch1 Q1.3

### Q14.6 Go나 Java보다 얼마나 빠른가요? (벤치마크를 읽는 법)

**짧은 답** 한 숫자로 답할 수 없습니다. 같은 일을 같은 알고리즘으로 했는지, 릴리스 빌드인지, JIT 예열과 GC 설정을 어떻게 했는지, 무엇을(처리량, 지연 시간, 메모리) 쟀는지에 따라 결과가 크게 달라집니다. 이 챕터가 직접 언어 사이의 속도 수치를 만들지 않는 이유입니다. 숫자를 읽을 때는 그 숫자가 무엇을 비교했는지부터 확인합니다.

**왜 그런가** 벤치마크나 사례를 읽을 때 따질 것입니다.

| 확인할 것 | 왜 |
| --- | --- |
| 빌드 설정(Rust는 `--release`인가) | 디버그 빌드는 몇 배 느릴 수 있습니다(Q14.4) |
| 같은 알고리즘·자료구조인가 | 언어보다 알고리즘 차이가 더 큽니다 |
| 무엇을 쟀나(처리량(throughput), 지연 시간의 꼬리, 메모리) | 지연 시간의 꼬리는 가장 느린 몇 %의 요청이 걸린 시간(예: p99, 100번 가운데 가장 느린 1번)입니다. GC 멈춤 같은 차이는 평균보다 여기서 드러납니다 |
| 측정 방법(반복, 예열, 최적화로 지워진 계산) | 릴리스 빌드는 쓰이지 않는 계산을 지웁니다(부록 A ch20의 `black_box`) |
| 구조도 함께 바꿨나 | 다시 쓴 사례는 대개 설계도 바꿨습니다 |

마지막 항목은 ch1 Q1.3에서 자세히 봤습니다. Cloudflare는 Pingora로 바꾼 뒤 같은 트래픽에서 CPU를 약 70%, 메모리를 약 67% 덜 썼다고 보고했지만(2022-09-14), 같은 글에서 그 절감이 Lua 대신 Rust를 쓴 효율과 함께 새 구조에서 왔다고 설명합니다. Discord도 자료구조 변경과 런타임 업그레이드를 함께 적었습니다(2020-02-04).

**다른 언어에서는** JIT을 쓰는 Java는 실행 초기와 예열 뒤의 속도가 다르고, GC 설정에 따라 처리량과 지연 시간이 바뀝니다. Go도 GC의 목표 설정에 따라 메모리와 CPU의 균형이 달라집니다. 이런 언어와의 비교 숫자는 그 설정과 함께 읽어야 합니다.

**오해와 한계** "Rust로 바꾸면 몇 배 빨라진다"는 기대는 대부분의 서비스에 맞지 않습니다. 시간이 데이터베이스나 네트워크 대기에 쓰이는 프로그램은 언어를 바꿔도 크게 빨라지지 않습니다. Rust의 이점이 분명한 곳은 CPU와 메모리를 많이 쓰거나, 지연 시간의 꼬리가 중요하거나, 자원이 좁은 곳입니다(ch1 Q1.6).

**더 읽기** ch1 Q1.3·Q1.6 / 부록 A ch20 / Part 3 ch6

### Q14.7 바이너리가 왜 크고 어떻게 줄이나요?

**짧은 답** Rust 실행 파일은 표준 라이브러리와 의존 크레이트를 실행 파일 안에 정적으로 넣고, 제네릭을 타입마다 펼치며, 패닉과 서식 출력 장치를 함께 담기 때문에 C보다 큽니다. 크기는 프로필의 `opt-level = "s"`·`"z"`, `lto`, `codegen-units = 1`, `strip`, `panic = "abort"`로 줄이고, 의존성과 제네릭 사용을 줄이는 것도 효과가 있습니다.

**왜 그런가** 이 환경에서 이름이 `hw`인 새 패키지의 hello world(`println!("hello");`)를 `cargo build --release`로 만든 실행 파일은 435,824바이트였고(패키지 이름에 따라 수십 바이트 달라짐), C hello world(`puts`, `cc -O2`)는 33,432바이트였습니다. `otool -L`로 보면 둘 다 동적으로는 시스템 라이브러리(`libSystem.B.dylib`) 하나만 링크합니다. C는 `puts`를 시스템의 C 라이브러리에서 빌려 쓰고, Rust는 표준 라이브러리를 실행 파일에 담는 차이입니다. 프로필 설정마다 크기가 어떻게 바뀌는지는 [Part 3 ch6](../../part3-cargo/ch6-profiles-build-performance/README.md)에서 같은 소스를 여러 프로필로 빌드해 비교했습니다.

**다른 언어에서는** Go도 런타임과 표준 라이브러리를 실행 파일에 정적으로 넣습니다. 이 환경에서 Go 1.27.1로 만든 hello world(`fmt.Println`, 모듈로 빌드)는 2,429,714바이트였습니다. Java는 JVM이 따로 있어 `.jar`는 작지만 실행에는 JVM이 필요합니다. C와 C++는 표준 라이브러리를 시스템에서 동적으로 빌려 쓰는 것이 보통입니다.

**오해와 한계** 크기를 줄이는 설정은 대가가 있습니다. `strip`은 백트레이스의 함수 이름을, `panic = "abort"`는 패닉 되감기와 `catch_unwind`를, `opt-level = "z"`는 속도를 내줄 수 있습니다(Part 3 ch6). 수백 KB의 차이는 서버와 데스크톱에서는 대개 문제가 되지 않고, 임베디드나 WebAssembly에서 중요해집니다(`no_std`는 [부록 A ch33](../../appd-a/ch33-no-std/README.md)).

**더 읽기** Part 3 ch6 / 부록 A ch12·ch33

### Q14.8 `HashMap`은 왜 기본 해시 함수가 느린 편인가요?

**짧은 답** 기본 해시 함수(현재 SipHash 1-3)는 속도보다 안전을 골랐습니다. 공격자가 같은 해시를 내는 키를 대량으로 보내 해시 맵을 느리게 만드는 공격(HashDoS)을 막도록 맵마다 무작위 키로 초기화됩니다. 정수 같은 작은 키에서는 더 빠른 해시 함수가 있으므로, 입력을 공격자가 고를 수 없는 곳이면 맵마다 해시 함수를 바꿀 수 있습니다.

**왜 그런가** std의 `HashMap` 문서는 기본 알고리즘이 현재 SipHash 1-3이고 언제든 바뀔 수 있으며, 중간 크기의 키에서는 경쟁력 있지만 정수 같은 작은 키나 긴 문자열에서는 다른 알고리즘이 더 빠르고, 그런 알고리즘은 대개 HashDoS를 막지 못한다고 적습니다. 기본 해시 함수의 뜻과 순회 순서가 실행마다 다른 이유는 [Part 0 ch8](../../part0-getting-started/ch8-collections/README.md)과 [Part 1 ch4](../../part1-standard-lib/ch4-collections/README.md) 2.11절에서, `with_hasher`로 FNV 해셔를 끼우는 방법은 [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)에서 봤습니다.

**다른 언어에서는** C++의 `std::unordered_map`은 기본 해시로 `std::hash`를 쓰고, 그 알고리즘은 표준 라이브러리 구현이 정합니다. 해시 맵을 노린 공격을 막을지는 언어와 구현마다 다르므로, 외부 입력을 키로 받는 맵이라면 쓰는 언어의 기본 해시가 무엇인지 확인해야 합니다.

**오해와 한계** 해시 함수를 바꿀 때는 그 맵의 키를 외부 입력이 정하지 않는지 확인해야 합니다. 웹 요청의 헤더나 매개변수를 키로 쓰는 맵은 기본값을 두는 편이 안전합니다. crates.io의 빠른 해셔(예: `rustc-hash`, `ahash`)는 쓰기 쉽지만 의존성이 하나 늘어납니다(ch16 `ch16-tooling-ecosystem`).

**더 읽기** Part 0 ch8 / Part 1 ch4 2.11절 / Part 1 ch6

### Q14.9 꼬리 호출 최적화가 있나요?

**짧은 답** 꼬리 호출(tail call) 최적화는 언어가 보장하지 않습니다. 최적화를 켜면 LLVM이 꼬리 재귀(tail recursion)를 반복문으로 바꿔 주는 경우가 많지만, 디버그 빌드에서는 바꾸지 않고, `Drop`이 있는 값이 호출 뒤까지 살아 있으면 릴리스에서도 바꿀 수 없습니다. 깊이가 입력에 따라 커지는 재귀는 반복문으로 쓰는 것이 안전합니다. 꼬리 호출을 보장하는 `become` 키워드는 예약만 되어 있고, 그 기능(`explicit_tail_calls`)은 nightly에만 있습니다.

**왜 그런가** 디버그에서는 스택 오버플로로 끝나고 최적화 빌드에서는 컴파일러가 반복문으로 바꿔 끝까지 간 재귀는 [ch3](../ch3-safety-limits/README.md) Q3.7에서 봤습니다. 여기서는 `Drop`이 그 최적화를 막는 경우와 반복문 판을 더합니다. [01_recursion_depth](examples/01_recursion_depth.rs)는 스택이 256 KiB인 스레드에서 100만 번 재귀합니다. 인자에 따라 함수를 고르고, 디버그와 릴리스에서 각각 실행한 결과입니다.

| 인자 | 디버그(`cargo run`) | 릴리스(`--release`) |
| --- | --- | --- |
| `tail`(꼬리 재귀) | 스택 오버플로, 종료 상태 134 | `깊은 재귀(100만 번): 1000000` |
| `drop`(꼬리 재귀 + `Drop` 값) | 스택 오버플로, 종료 상태 134 | 스택 오버플로, 종료 상태 134 |
| `loop`(반복문) | `깊은 재귀(100만 번): 1000000` | `깊은 재귀(100만 번): 1000000` |

`drop`의 `count_with_drop`은 겉보기에 마지막 줄이 자기 호출이지만, 지역 변수 `_frame`(`Drop`을 구현한 `Frame`)은 그 호출이 돌아온 뒤에 drop되어야 합니다. `Frame`의 drop은 전역 카운터를 올리는 부수 효과가 있어 최적화가 없앨 수 없고, drop이 호출 뒤에 남으므로 호출이 진짜 꼬리 위치가 아닙니다. 그래서 최적화도 프레임을 없애지 못합니다. 스택 오버플로가 나면 스레드 하나가 아니라 프로세스 전체가 끝납니다.

**다른 언어에서는** C도 꼬리 호출을 반복문으로 바꾸는 것은 최적화가 하는 일입니다. [compare/01_deep_recursion.c](compare/01_deep_recursion.c)의 같은 꼬리 재귀(1000만 번)는 `cc -O2`에서 `10000000`을 출력했지만 `-O0`에서는 출력 없이 종료 상태 139(SIGSEGV)로 끝났고, ASan을 켜면 `ERROR: AddressSanitizer: stack-overflow`를 보고했습니다. Rust와 같은 결과인 것은 둘이 같은 LLVM 최적화를 쓰기 때문입니다.

**오해와 한계** 릴리스에서 `tail`이 끝까지 간 것은 이 컴파일러가 이번에 그렇게 최적화했기 때문이지 약속이 아닙니다. 코드를 조금 바꾸거나(예: `Drop` 값 추가) 디버그로 빌드하면 바로 넘칩니다. 지역 값의 주소를 밖으로 내보내는 코드도 이 최적화를 막습니다. 이 환경의 릴리스 빌드에서는 `count_tail` 안에 `let x = std::hint::black_box(5u64);` 한 줄만 더해도(`Drop`이 없는 `Copy` 값) 스택 오버플로로 끝났습니다. 트리처럼 깊이가 입력에 달린 자료구조를 재귀로 처리할 때는 명시적인 스택(`Vec`)을 쓰는 반복문을 고려합니다.

**더 읽기** Reference "Keywords"(예약 키워드 `become`) / The Unstable Book `explicit_tail_calls` / [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md)(스레드 스택 크기)

### Q14.10 메모리는 얼마나 쓰나요?

**짧은 답** 값의 크기와 배치가 타입으로 정해지고, GC가 다음 수거까지 쓰는 여유 메모리가 없으므로 사용량을 예측하기 쉽습니다. 대신 컬렉션의 남는 용량, 할당 횟수, 큰 값을 스택에 둘지 힙에 둘지처럼 코드가 정하는 부분은 직접 챙깁니다.

**왜 그런가** 02의 1번에서 1000개를 넣은 `Vec<u32>`는 `len 1000, capacity 1024`였습니다. 용량 24개분(96바이트)은 할당했지만 쓰지 않는 메모리입니다. 용량이 넉넉히 늘어나는 덕에 재할당이 8번으로 끝났지만, 다 채운 뒤 메모리를 줄이고 싶으면 `shrink_to_fit`을 부릅니다(용량 관리는 Part 1 ch4 2.7절). 타입의 크기와 정렬, `Option`이 빈 값을 표현하려고 공간을 더 쓰지 않는 경우(니치)는 [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 예제 01·02에서, 값을 스택과 힙 가운데 어디에 두는지는 ch4 Q4.10에서 봤습니다.

**다른 언어에서는** GC 언어는 수거 사이에 쌓이는 쓰레기만큼 살아 있는 데이터보다 많은 힙을 둡니다. 예를 들어 Go의 `runtime/debug.SetGCPercent` 문서는 이전 수거 뒤 살아남은 데이터 대비 새로 할당한 데이터의 비율이 설정값(기본 100, 환경 변수 `GOGC`)에 이르면 수거가 시작된다고 적습니다. 설정으로 메모리와 CPU 사이의 균형을 고르는 것이고, Rust에는 그런 여유분이 없습니다.

**오해와 한계** Rust 프로그램도 메모리를 많이 쓸 수 있습니다. `Rc` 순환은 해제되지 않고(ch3 Q3.4), 큰 `Vec`의 남는 용량이나 오래 사는 캐시는 그대로 메모리입니다. 실제 사용량은 운영체제 도구나 할당기 통계로 재야 합니다.

**더 읽기** Part 1 ch7 / ch4 Q4.9·Q4.10 / ch3 Q3.4

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_recursion_depth](examples/01_recursion_depth.rs) | 꼬리 재귀, `Drop` 값이 있는 재귀, 반복문을 깊이 100만에서 디버그·릴리스로 실행 | Q14.9, Q14.5 |
| [02_count_allocations](examples/02_count_allocations.rs) | 전역 할당기로 `Vec`의 재할당과 반복문 안 `format!`의 할당 세기 | Q14.4, Q14.10 |
| [03_noalias](examples/03_noalias.rs) | `&mut`·`&` 매개변수가 LLVM IR에서 `noalias`가 되는 것 | Q14.1 |

### 01_recursion_depth — 꼬리 호출은 보장되지 않는다

```sh
cargo run -p faq-ch14-performance --example 01_recursion_depth
```

```text
얕은 재귀(1000번)
  tail: 1000
  drop: 1000 (Frame drop 1001번)
  loop: 1000
```

- 인자 없이 실행하면 깊이 1000이라 어느 빌드에서나 끝까지 갑니다. `drop` 줄의 `Frame drop 1001번`은 호출 1001번(깊이 0~1000)마다 `Frame`이 한 번씩 drop된 수입니다.
- 깊이 100만은 `cargo run -p faq-ch14-performance --example 01_recursion_depth -- tail`과 `cargo run --release -p faq-ch14-performance --example 01_recursion_depth -- tail`처럼 실행합니다(`tail` 자리에 `drop`, `loop`). `--release`는 `--` 앞에 둡니다. 뒤에 두면 프로그램 인자가 되어 디버그 빌드가 실행됩니다(결과는 Q14.9의 표).
- 스택 오버플로가 난 실행은 표준 에러에 `thread 'recursion' (번호) has overflowed its stack`과 `fatal runtime error: stack overflow, aborting`을 출력하고 종료 상태 134로 끝납니다. 괄호 안의 번호는 스레드 식별자라 실행마다 다릅니다.
- 넘치는 지점을 빨리, 매번 같게 보려고 스택을 256 KiB로 정한 스레드에서 재귀합니다(`thread::Builder::stack_size`, Part 1 ch11). 메인 스레드는 스택이 더 커서 넘치는 깊이가 다릅니다.
- `Frame`의 drop이 전역 카운터를 올리는 것은 최적화가 drop을 지우지 못하게 하려는 것입니다. 아무 일도 하지 않는 값이나 쓰이지 않는 `String`은 최적화가 통째로 지워 버려 `Drop`의 효과를 볼 수 없습니다.

### 02_count_allocations — 할당 세기

```sh
cargo run -p faq-ch14-performance --example 02_count_allocations
```

```text
1. Vec에 1000개 넣기
  Vec::new: 할당 1, 재할당 8
  len 1000, capacity 1024
  Vec::with_capacity: 할당 1, 재할당 0
2. 문자열 100조각 잇기
  + format!: 할당 101, 재할당 6
  write!: 할당 1, 재할당 6
  두 문자열이 같은가: true
```

- `#[global_allocator]`로 프로그램 전체의 할당기를 바꿉니다. `Counting`은 시스템 할당기(`System`)에 그대로 넘기고 원자적 카운터만 올립니다.
- 횟수는 세는 구간 앞뒤의 차이로 냅니다. 구간 안에서는 출력하지 않습니다(표준 출력도 처음 쓸 때 버퍼를 할당할 수 있음).
- `joined + &format!(…)`은 `joined`의 버퍼를 이어 쓰므로 그 재할당은 `write!` 쪽과 같은 6번이고, 차이는 반복마다 `format!`이 만든 임시 `String` 100개입니다.
- 디버그와 릴리스에서 같은 횟수였습니다.

### 03_noalias — `&mut`가 주는 정보

```sh
cargo run -p faq-ch14-performance --example 03_noalias
```

```text
total = 16
```

- LLVM IR을 보는 명령은 파일 머리 주석에 있고, 결과는 Q14.1에 옮겼습니다.
- `add_twice`에 `#[unsafe(no_mangle)]`을 붙인 것은 C 쪽 함수처럼 다른 파일에서도 부를 수 있는 함수로 두기 위해서입니다. 크레이트 안에서만 쓰이는 함수라면 LLVM이 `&i32`를 값으로 바꿔 넘기는 등 더 고쳐 버려 C와 비교하기 어려워집니다(`no_mangle`은 [ch13](../ch13-headers-build-templates/README.md) Q13.6).

### C 비교 코드

저장소 루트에서 실행합니다. `cc`의 `-o`는 출력 디렉터리를 만들지 않으므로 먼저 `mkdir -p target/tmp/faq-ch14-performance`를 실행합니다. 파일마다 머리 주석에 명령이 있습니다.

| 파일 | 질문 | 명령(요지) | 이 환경에서 나온 것 |
| --- | --- | --- | --- |
| [01_deep_recursion.c](compare/01_deep_recursion.c) | Q14.9 | `cc -std=c17 -Wall -Wextra -O2` / `-O0` / `-O0 -g -fsanitize=address,undefined` 후 실행 | `-O2`: `10000000`, 종료 0. `-O0`: 출력 없이 종료 139. ASan: `ERROR: AddressSanitizer: stack-overflow`, 종료 134 |
| [02_aliasing.c](compare/02_aliasing.c) | Q14.1 | `cc -std=c17 -Wall -Wextra -O3 -S -emit-llvm` 후 IR 보기 | `add_twice`는 `*b`를 두 번 읽고 두 번 씀. `add_twice_restrict`는 한 번 읽고 `shl … 1`, 한 번 씀 |

- 01은 미정의 동작이 아니라 스택이라는 자원이 모자란 경우라, 끝까지 간 `-O2`의 출력은 정해진 값입니다.
- 02는 실행하지 않고 컴파일러가 만든 IR만 봅니다.

## 4. 자주 하는 실수

이 챕터의 실수는 컴파일 에러가 아니라 성능을 재고 판단하는 방식에서 나오므로 `compile_fail/`을 두지 않았습니다.

| 실수 | 왜 틀렸나 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| `cargo run`(디버그)으로 속도를 잼 | 최적화가 꺼져 있고 검사가 더 켜져 있습니다 | `cargo run --release`로 잽니다 | Q14.4, ch1 4절 |
| 깊은 재귀를 릴리스에서 통과했다고 믿음 | 꼬리 호출 최적화는 보장이 아닙니다 | 깊이가 입력에 달리면 반복문으로 씁니다 | Q14.9 |
| 경계 검사가 느릴 거라며 `get_unchecked`부터 씀 | 검사가 남는지, 느린지 확인하지 않았습니다 | 반복자·슬라이스 표현으로 바꾸고 재 봅니다 | Q14.3, 부록 A ch16 |
| 반복문 안에서 `format!`으로 문자열을 이음 | 반복마다 새 `String`을 할당합니다 | 한 버퍼에 `write!`·`push_str`로 씁니다 | Q14.4 |
| 외부 입력을 키로 쓰는 맵의 해시 함수를 빠른 것으로 바꿈 | HashDoS 방어가 사라집니다 | 그런 맵은 기본 해시 함수를 둡니다 | Q14.8 |
| 다시 쓴 사례의 개선을 모두 언어 덕으로 봄 | 대개 구조도 함께 바꿨습니다 | 원문에서 무엇을 함께 바꿨는지 봅니다 | Q14.6, ch1 Q1.3 |

## 5. 연습 문제

### 연습 1. 넘치지 않게 고치기

`examples/01_recursion_depth.rs`의 `count_with_drop`에서 `let _frame = Frame;`을 `let frame = Frame; drop(frame);`으로 바꿔 재귀 호출 전에 버리면, `-- drop`이 디버그와 릴리스에서 각각 어떻게 될까요? 해 본 뒤, 어느 빌드에서나 끝까지 가게 고쳐 보세요.

<details>
<summary>힌트</summary>

이 환경에서는 릴리스가 `깊은 재귀(100만 번): 1000000`으로 끝까지 갔고, 디버그는 여전히 종료 상태 134로 넘쳤습니다. drop이 호출 앞으로 옮겨져 호출이 다시 꼬리 위치가 되었기 때문입니다. 하지만 디버그에서는 최적화가 없으니 소용이 없습니다. 어느 빌드에서나 끝까지 가게 하는 확실한 방법은 `count_loop`처럼 반복문으로 바꾸고, 반복마다 `Frame`을 만들고 버리는 것입니다.

</details>

### 연습 2. `collect`도 세어 보기

`examples/02_count_allocations.rs`에 3번 구역을 더해, `(0..1000).map(|i| i as u32).collect::<Vec<u32>>()`가 할당과 재할당을 몇 번 하는지 세어 보세요. 1번의 두 결과 가운데 어느 쪽과 같을까요?

<details>
<summary>힌트</summary>

이 환경에서는 할당 1, 재할당 0으로 `with_capacity` 쪽과 같았습니다. 범위(`0..1000`)에 `map`을 붙인 반복자는 남은 개수를 정확히 알려 주므로(`size_hint`), `collect`가 처음부터 그만큼 잡습니다. `filter`를 끼우면 개수를 미리 알 수 없어 결과가 달라집니다. `filter(|i| i % 2 == 0)`을 넣으면 이 환경에서는 할당 1, 재할당 7이었고 500개에 용량 512였습니다.

</details>

### 연습 3. C에 `restrict`를 붙이면 무엇을 약속하나

`compare/02_aliasing.c`에 `int x = 1; add_twice_restrict(&x, &x);`처럼 같은 변수를 두 번 넘기는 `main`을 더하면 어떤 일이 생길까요? Rust에서 `add_twice(&mut x, &x)`를 쓰면 어떻게 될까요?

<details>
<summary>힌트</summary>

C에서는 `restrict`의 약속을 어긴 것이라 미정의 동작입니다. 컴파일러는 경고하지 않을 수 있고, 최적화된 코드는 `*b`를 한 번만 읽으므로 `restrict` 없는 판과 다른 값을 낼 수 있습니다(미정의 동작이므로 출력에 기대면 안 됩니다). Rust에서는 `x`를 `&mut`로 빌린 동안 `&x`를 만들 수 없어 E0502로 컴파일되지 않습니다([ch4](../ch4-memory-without-gc/README.md) 4.2절).

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rust Programming Language: [13.4절 Comparing Performance: Loops vs. Iterators](https://doc.rust-lang.org/book/ch13-04-performance.html)
- The Rust Reference: [The Rust runtime](https://doc.rust-lang.org/reference/runtime.html), [Keywords](https://doc.rust-lang.org/reference/keywords.html)(예약 키워드)
- 표준 라이브러리 문서: [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)(기본 해시 함수), [`Stdout`](https://doc.rust-lang.org/std/io/struct.Stdout.html)(줄 버퍼링), [`GlobalAlloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html), [`thread::Builder::stack_size`](https://doc.rust-lang.org/std/thread/struct.Builder.html#method.stack_size)
- The Cargo Book: [Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- The rustc book: [Codegen Options](https://doc.rust-lang.org/rustc/codegen-options/index.html)
- The Unstable Book: [explicit_tail_calls](https://doc.rust-lang.org/unstable-book/language-features/explicit-tail-calls.html)

다른 언어

- The Go Programming Language Specification: [Index expressions](https://go.dev/ref/spec#Index_expressions)(범위 밖 인덱스는 실행 중 패닉)
- Go 표준 라이브러리: [`runtime/debug.SetGCPercent`](https://pkg.go.dev/runtime/debug#SetGCPercent)
- The Java Language Specification(SE 21): [15.10.4 Run-Time Evaluation of Array Access Expressions](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.10.4)

도입 사례(ch1 Q1.3에서 확인한 자료)

- Cloudflare, [How we built Pingora, the proxy that connects Cloudflare to the Internet](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/) (2022-09-14)
- Discord, [Why Discord is switching from Go to Rust](https://discord.com/blog/why-discord-is-switching-from-go-to-rust) (2020-02-04)
