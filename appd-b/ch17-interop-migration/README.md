# ch17. 기존 코드와 함께 쓰기

Rust를 들이려는 팀이 가진 것은 빈 저장소가 아니라 수십만 줄의 C/C++ 코드와 그것을 빌드하는 시스템입니다. 이 챕터는 "다시 써야 하나", "어떻게 잇나", "어디부터, 얼마의 비용으로"에 답합니다. FFI를 쓰는 방법 자체는 Part 1 ch14와 부록 A ch34·ch35에서 실습했으므로 이 챕터는 그것을 가리키고, 함께 쓰기로 정했을 때 판단할 것에 집중합니다.

- 패키지: `faq-ch17-interop-migration`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), Apple clang 21.0.0, macOS(aarch64)
- 자료 확인일: 2026-11-21
- 이름과 용도만 적고 설치하지 않은 도구: `cxx`, autocxx, Crubit, PyO3, napi-rs, `jni`, UniFFI, `wasm-bindgen`, `libloading`, `cbindgen`, C2Rust, Corrosion, rules_rust, Meson. 크레이트 버전은 확인일의 crates.io 기준입니다.

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q17.1](#q171-기존-cc-코드를-버리고-다시-써야-하나요) | 기존 C/C++ 코드를 버리고 다시 써야 하나요? | 아닙니다. Android 팀은 기존 코드를 두고 새 코드를 메모리 안전한 언어로 쓰며, 다시 쓰기보다 언어 사이를 잇는 일에 투자하라고 권합니다. 다시 쓰면 같은 일을 하는 함수도 동작이 달라질 수 있습니다. |
| [Q17.2](#q172-rust에서-c-라이브러리를-어떻게-부르고-c에서-rust를-어떻게-부르나요) | Rust에서 C 라이브러리를 어떻게 부르고, C에서 Rust를 어떻게 부르나요? | 둘 다 C ABI로 만납니다. Rust는 `unsafe extern "C"` 블록에 C 함수를 선언해 부르고, C에는 `#[unsafe(no_mangle)] extern "C"` 함수를 정적 라이브러리(`staticlib`)나 동적 라이브러리(`cdylib`)로 내보냅니다. |
| [Q17.3](#q173-c와는-어떻게-연동하나요) | C++와는 어떻게 연동하나요? | C++ 함수는 맹글링된 이름으로 일부 부를 수 있지만 그 이름은 컴파일러마다 다르고, 템플릿·인라인 함수는 심볼이 없거나 실체화되지 않아 부를 수 없으며, 생성자·소멸자·예외는 손으로 맞춰야 합니다. 보통은 C++ 쪽에 `extern "C"` 함수로 감싼 층을 두거나, `cxx`·autocxx 같은 도구로 다리를 만듭니다. |
| [Q17.4](#q174-python-nodejs-java-c-go에서-rust-코드를-부를-수-있나요) | Python, Node.js, Java, C#, Go에서 Rust 코드를 부를 수 있나요? | 부를 수 있습니다. C#·Go·Java(FFM API)·Python(`ctypes`)은 C ABI로 내보낸 함수를 바로 부르고, Node.js·Java(JNI)·Python 확장 모듈처럼 정해진 규약으로 함수를 등록해야 하는 곳에는 napi-rs·`jni`·PyO3 같은 크레이트를 씁니다. |
| [Q17.5](#q175-경계의-unsafe는-안전성을-얼마나-깎나요) | 경계의 `unsafe`는 안전성을 얼마나 깎나요? | 경계 너머의 C 코드는 Rust의 보장 밖이고, 경계 코드는 사람이 확인해야 합니다. 그 코드를 작게 가두고 안전한 API로 감싸면 나머지 Rust 코드는 계속 컴파일러의 검사를 받습니다. |
| [Q17.6](#q176-c-코드를-자동으로-rust로-바꿀-수-있나요) | C 코드를 자동으로 Rust로 바꿀 수 있나요? | C2Rust는 C를 `unsafe` Rust로 옮겨 주지만 그 결과를 안전한 Rust로 바꾸는 일은 따로 남습니다. 안전한 Rust로의 자동 번역은 DARPA TRACTOR 같은 연구 단계입니다. |
| [Q17.7](#q177-어디부터-옮기는-것이-좋은가요) | 어디부터 옮기는 것이 좋은가요? | 새 코드부터입니다. 기존 코드라면 신뢰할 수 없는 입력을 다루고, 인터페이스가 좁고, 테스트로 동작을 대조할 수 있는 부분부터 옮깁니다. |
| [Q17.8](#q178-팀에-도입할-때-드는-비용은-무엇인가요) | 팀에 도입할 때 드는 비용은 무엇인가요? | 학습 시간, 느린 빌드, 두 번째 툴체인과 빌드 단계, 경계 코드와 바인딩의 유지, 두 언어를 읽는 리뷰입니다. Google은 사내 조사에서 학습 기간이 경험상 다른 언어와 비슷했다고 밝혔습니다. |
| [Q17.9](#q179-기존-빌드-시스템cmake-bazel에-rust를-넣을-수-있나요) | 기존 빌드 시스템(CMake, Bazel)에 Rust를 넣을 수 있나요? | 넣을 수 있습니다. 가장 단순하게는 Cargo로 정적 라이브러리를 만들어 기존 빌드가 링크하고, CMake에는 Corrosion, Bazel에는 rules_rust, Meson에는 자체 지원이 있습니다. |
| [Q17.10](#q1710-도입을-판단할-때-무엇을-물어야-하나요) | 도입을 판단할 때 무엇을 물어야 하나요? | 풀려는 문제가 무엇인지, 옮길 부분이 입력과 경계 면에서 알맞은지, 동작을 대조할 테스트가 있는지, 대상 플랫폼과 라이브러리, 팀의 학습과 빌드에 들일 여력이 있는지 묻고, GC 언어나 C++ 하드닝 같은 대안과 비교합니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md): `unsafe`, `extern` 블록, C에서 부를 Rust 함수
- [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md): `CString`·`CStr`, C 함수에 넘기고 받기
- [Part 3 ch8](../../part3-cargo/ch8-build-scripts/README.md): 빌드 스크립트, 네이티브 라이브러리 링크와 `-sys` 패키지
- [부록 A ch34](../../appd-a/ch34-ffi-boundaries/README.md), [부록 A ch35](../../appd-a/ch35-bindgen/README.md): FFI 경계를 통제하는 지침, bindgen
- [ch1](../ch1-why-rust/README.md) Q1.7, [ch3](../ch3-safety-limits/README.md), [ch16](../ch16-tooling-ecosystem/README.md) Q16.9: 다시 쓰지 않아도 되는 이유, `unsafe`의 범위, ABI

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 기존 코드를 다시 쓰지 않고 새 코드와 경계부터 Rust를 들이는 근거와, 다시 쓰거나 번역할 때 동작이 달라지는 위험을 설명할 수 있다.
2. Rust와 C가 서로 부르는 방법과, C++·다른 언어와 잇는 도구가 어디까지 해 주는지 말할 수 있다.
3. 경계의 `unsafe`가 보장하지 못하는 것과 그것을 작게 가두는 방법을 설명할 수 있다.
4. C 프로그램의 함수 하나를 Rust 정적 라이브러리로 바꿔 끼우고, 원래 구현과 출력을 대조할 수 있다.
5. 옮길 곳을 고르는 기준, 도입 비용, 빌드 시스템 통합, 판단할 때 물을 것을 말할 수 있다.

## 2. 핵심 개념

### Q17.1 기존 C/C++ 코드를 버리고 다시 써야 하나요?

**짧은 답** 아닙니다. [ch1](../ch1-why-rust/README.md) Q1.7에서 봤듯이 Google Android 팀은 기존 C/C++ 코드를 그대로 두고 새 코드를 메모리 안전한 언어로 쓰는 방식으로 메모리 안전성 취약점의 비율을 줄였습니다. 같은 팀은 메모리 안전하지 않은 기존 코드를 모두 버리거나 다시 쓸 필요가 없고, 대신 언어 사이의 상호 운용(interoperability)을 안전하고 편하게 만드는 데 투자하라고 권합니다. 다시 쓰기에는 비용과 위험이 따릅니다. 같은 일을 하는 것처럼 보이는 함수도 입력에 따라 동작이 다르고, 원래 구현의 버그까지 프로그램 동작의 일부가 되어 있기 때문입니다.

**왜 그런가** Android 팀의 2024-09-25 글은 메모리 안전성 취약점의 비율이 6년 동안 76%에서 24%로 줄었다고 적고, 그 이유로 취약점이 새 코드나 최근에 고친 코드에 몰려 있고 코드가 오래될수록 지수적으로 줄어든다는 관찰을 듭니다(ch1 Q1.7). 같은 글은 "모든 기존 코드를 버리거나 다시 쓸 필요가 없다"고 결론짓고, Rust ↔ C++, Rust ↔ Kotlin의 상호 운용과 그 도구(Crubit, autocxx)에 투자하고 있다고 밝힙니다.

다시 쓰기의 위험은 이 챕터의 두 실습에서 볼 수 있습니다. [01_strtol_vs_parse](examples/01_strtol_vs_parse.rs)는 C의 `strtol`과 Rust의 `str::parse::<i64>`가 같은 입력에서 다르게 동작하는 것을 보입니다. `strtol`은 앞의 공백을 건너뛰고, 숫자가 아닌 글자 앞까지만 읽고, 범위를 넘으면 최댓값이나 최솟값으로 잘라 돌려주지만, `parse`는 그 셋 모두 에러입니다.

```text
  " 42"                    strtol: 42 (3바이트 읽음) / parse: Err(InvalidDigit)
  "42abc"                  strtol: 42 (2바이트 읽음) / parse: Err(InvalidDigit)
  "9223372036854775808"    strtol: 9223372036854775807 (19바이트 읽음) / parse: Err(PosOverflow)
```

[projects/version_swap](projects/version_swap/Cargo.toml)에서는 C 프로그램의 함수 하나를 Rust 구현으로 바꿔 끼우자 출력 13줄 가운데 한 줄이 달라졌습니다. 원래 C 구현이 오버플로를 검사하지 않아 `4294967296.0.0`을 `0.0.0`으로 읽고 있었기 때문입니다(Q17.7).

**다른 언어에서는** 점진적인 전환은 다른 언어에서도 기본 전략입니다. Kotlin 문서는 Kotlin이 Java와의 상호 운용을 염두에 두고 설계되어 기존 Java 코드를 자연스럽게 부를 수 있다고 적고, TypeScript 안내서의 "Migrating from JavaScript"는 `allowJs` 옵션으로 JavaScript 파일을 그대로 입력으로 받으며 옮겨 가는 방법을 설명합니다. Rust와 C도 함수 단위로 섞을 수 있다는 점은 같지만, 두 언어가 같은 런타임을 나누지 않으므로 경계가 C ABI 수준으로 낮고 그 경계는 `unsafe`입니다(Q17.5).

**오해와 한계** 다시 쓰지 않는다는 것은 기존 코드의 취약점이 그대로 남는다는 뜻이기도 합니다. Android의 결과는 새 코드가 위험의 대부분이라는 관찰에 기대므로, 신뢰할 수 없는 입력을 다루는 오래된 부품처럼 위험이 큰 곳은 따로 옮길지 따져야 합니다(Q17.7). 상호 운용에도 경계 코드와 빌드라는 비용이 듭니다(Q17.5, Q17.8).

**더 읽기** ch1 Q1.7 / Q17.7 / Google 보안 블로그 "Eliminating Memory Safety Vulnerabilities at the Source"

### Q17.2 Rust에서 C 라이브러리를 어떻게 부르고, C에서 Rust를 어떻게 부르나요?

**짧은 답** 두 방향 모두 C ABI로 만납니다([ch16](../ch16-tooling-ecosystem/README.md) Q16.9). Rust에서 C를 부를 때는 `unsafe extern "C"` 블록에 C 함수를 선언하고 `unsafe` 블록 안에서 부르며, 링크할 라이브러리는 빌드 스크립트가 Cargo에 알립니다. C에서 Rust를 부를 때는 `#[unsafe(no_mangle)] pub extern "C" fn`으로 함수를 내보내고 `#[repr(C)]` 타입을 쓰며, 크레이트를 정적 라이브러리(`staticlib`, `.a`)나 동적 라이브러리(`cdylib`)로 빌드해 C 쪽 링커에 넘깁니다. 손으로 쓰는 선언은 bindgen(C 헤더 → Rust 선언)과 `cbindgen`(Rust → C 헤더)이 대신할 수 있습니다.

**왜 그런가** 예제 01은 C 표준 라이브러리의 `strtol`을 선언하고 안전한 래퍼 `c_strtol`로 감싸 부릅니다. 이 환경에서는 표준 라이브러리가 이미 C 라이브러리와 링크하므로 빌드 스크립트가 필요 없습니다. C 문자열을 넘기고 받는 방법은 Part 1 ch14 2.5절과 예제 `04_calling_c`에서, 직접 만든 C 라이브러리를 감싸는 방법은 부록 A ch34의 `tally_ffi`와 Part 3 ch8의 `fletcher`에서 봤습니다.

반대 방향은 `version_swap`이 보입니다. `src/lib.rs`의 `semver_parse`가 C 헤더 `csrc/semver.h`와 같은 시그니처로 내보내지고, `Cargo.toml`의 `[lib] crate-type = ["staticlib"]` 때문에 `cargo build --release`가 `target/release/libsemver_rs.a`를 만듭니다. C 프로그램은 `cc … csrc/main.c target/release/libsemver_rs.a`로 링크합니다. 정적 라이브러리가 기대는 시스템 라이브러리는 `cargo rustc --release -- --print native-static-libs`로 볼 수 있고, 이 환경에서는 `-lSystem -lc -lm`이라 `cc`가 기본으로 링크하는 것으로 충분했습니다. 부록 A ch34의 `textstat_export`는 Cargo만으로 확인하려고 C 코드를 Rust 실행 파일 안에 링크했는데, `version_swap`은 실제 C 프로그램처럼 C의 `main`이 Rust 라이브러리를 링크합니다.

**다른 언어에서는** 다른 언어도 C 함수를 부르는 장치를 둡니다. Go는 cgo, Python은 표준 라이브러리의 `ctypes`(공유 라이브러리의 함수를 부르는 "foreign function library", 배포판에 따라 빠질 수 있는 선택 모듈), Java는 Java 22에서 정식이 된 Foreign Function & Memory API(JEP 454는 이 API로 JNI의 취약함과 위험 없이 네이티브 라이브러리를 부른다고 소개), C#은 P/Invoke를 씁니다. 이 언어들은 런타임이 관리하는 객체를 C에 넘길 때의 규칙을 따로 둡니다(Q17.5). Rust는 GC가 없고 C와 같은 호출 규약(calling convention)으로 함수를 내보낼 수 있어, C 프로그램 입장에서는 Rust 라이브러리가 C 라이브러리와 같은 모양입니다.

**오해와 한계** C ABI로는 C가 표현할 수 있는 것만 넘깁니다. 문자열, 소유권, 라이프타임, 스레드 규칙은 타입이 아니라 헤더의 주석과 약속으로 전해지므로 경계 코드가 맞춰야 합니다(부록 A ch34). `extern "C"` 함수 밖으로 나가려는 Rust 패닉은 1.81부터 프로세스를 중단시키고(부록 A ch34 2.8절), The Rust Reference는 되감기(unwinding)를 허용하지 않는 프레임을 지나 되감는 것을 미정의 동작(undefined behavior)으로 꼽습니다. C++ 예외가 Rust를 지나가야 하면 `"C-unwind"` ABI를 씁니다. 또 `c_long`처럼 C 타입의 크기가 플랫폼마다 다르므로(예제 01의 머리 주석), 한 플랫폼에서 맞던 선언이 다른 플랫폼에서 틀릴 수 있습니다.

**더 읽기** Part 0 ch19 / Part 1 ch14 2.5절 / Part 3 ch8 / 부록 A ch34·ch35 / The Rustonomicon "FFI"

### Q17.3 C++와는 어떻게 연동하나요?

**짧은 답** Rust는 C++의 맹글링 규칙과 클래스, 템플릿, 예외를 알지 못합니다. 맹글링된 이름을 `#[link_name]`에 적으면 C++ 함수 일부를 부를 수 있지만, 그 이름은 컴파일러와 ABI마다 다르고, 템플릿·인라인 함수는 심볼이 없거나 실체화되지 않아 부를 수 없으며, 생성자·소멸자·예외는 손으로 맞춰야 합니다. 그래서 보통 쓰는 방법은 셋입니다. C++ 쪽에 `extern "C"` 함수로 감싼 층을 두고 C처럼 부르거나, `cxx`로 두 언어가 함께 쓸 타입과 함수를 한곳에 적어 양쪽 바인딩을 만들거나, autocxx처럼 C++ 헤더를 읽어 그 다리를 만들어 주는 도구를 씁니다. 어느 쪽이든 넘길 수 있는 타입은 정해진 범위로 줄어듭니다.

**왜 그런가** [compare/01_cpp_mangling.cpp](compare/01_cpp_mangling.cpp)는 이름이 같은 C++ 함수 둘(`scale(int)`, `scale(double)`)과 `extern "C"` 함수 하나를 정의합니다. 빌드한 실행 파일을 `nm`으로 보면 C++ 함수는 매개변수 타입을 덧붙인 이름(맹글링, mangling)으로, `extern "C"` 함수는 C의 이름 그대로 들어 있습니다.

```text
00000001000008d4 T __Z5scaled
0000000100000868 T __Z5scalei
00000001000008f0 T _scale_c
```

Rust의 `extern "C"` 선언은 적은 이름을 C의 이름으로 찾습니다. 그래서 `scale_c`는 그 이름으로 바로 부를 수 있고, 오버로드된 `scale`은 맹글링된 이름을 `#[link_name = "_Z5scalei"]`처럼 적어야 부를 수 있습니다(이 환경에서 같은 두 함수를 C++ 정적 라이브러리로 만들어 Rust에서 부르자 `scale(21)`은 42, `scale(1.25)`는 2.5였습니다). 앞의 `_`는 macOS가 C 이름에 붙이는 접두사이고, Rust가 링크할 때 붙이므로 `link_name`에는 적지 않습니다. bindgen이 C++ 헤더에서 만드는 선언이 바로 이 방식이고, 메서드·오버로딩은 대체로 되지만 인라인 함수와 템플릿 함수는 심볼이 없거나 실체화되지 않아 옮기지 못합니다. 맹글링 규칙은 C++ 컴파일러와 플랫폼의 ABI가 정하므로, 이렇게 만든 선언은 그 조합에 묶입니다. `cxx`의 `#[cxx::bridge]`, autocxx, bindgen의 C++ 지원 범위는 [부록 A ch35](../../appd-a/ch35-bindgen/README.md) 2.6절에서 봤습니다. Android 팀은 2024년 글에서 Rust ↔ C++ 상호 운용 도구로 Crubit과 autocxx를 만들고 있다고 밝혔습니다(Q17.1).

**다른 언어에서는** Swift는 컴파일러 차원에서 C++ 상호 운용을 지원하지만, Swift 문서 "Mixing Swift and C++"는 이 기능이 계속 발전 중이고 지금은 언어 기능의 일부만 지원한다고 적습니다. C++와 다른 언어를 잇는 일은 대개 지원 범위를 정해 두고 그 안에서만 하는 일입니다.

**오해와 한계** C 층으로 감쌀 때는 C++ 예외를 그 층에서 잡아 에러 코드로 바꾸고, C++ 객체는 불투명 포인터와 해제 함수로 넘기고, 템플릿은 구체적인 타입으로 실체화한 함수만 내보냅니다. 감싸는 코드가 C++ API의 크기만큼 늘어나므로, 경계를 넘는 호출이 많은 곳(작은 메서드를 자주 부르는 객체 등)보다 덩어리가 큰 기능 단위로 경계를 긋는 편이 유지하기 쉽습니다.

**더 읽기** 부록 A ch35 2.6절 / [ch12](../ch12-modern-cpp-enough/README.md) / CXX(cxx.rs)

### Q17.4 Python, Node.js, Java, C#, Go에서 Rust 코드를 부를 수 있나요?

**짧은 답** 부를 수 있습니다. 길은 둘입니다. 첫째, 임의의 C ABI 함수를 부르는 장치(Python `ctypes`, Java FFM API, C# P/Invoke, Go cgo)가 있는 언어는 Rust가 내보낸 `extern "C"` 함수를 바로 부릅니다. `ctypes`·FFM API·P/Invoke는 동적 라이브러리(`cdylib`)를 불러오고, cgo는 `#cgo LDFLAGS`로 링크할 라이브러리를 지정합니다. 둘째, 네이티브 확장 API(CPython C API, Node.js의 Node-API, Java의 JNI)는 확장 쪽이 그 API의 규약대로 함수를 등록해야 하므로, 그냥 `extern "C"`로 내보낸 함수는 부를 수 없습니다. 이 규약을 대신 맞춰 주는 크레이트가 PyO3(Python), napi-rs(Node-API), `jni`(JNI)입니다. 여러 언어의 바인딩을 생성하는 UniFFI, 웹의 JavaScript와 WebAssembly를 잇는 `wasm-bindgen`도 있습니다.

**왜 그런가** C ABI는 거의 모든 언어가 부를 수 있는 공통 경계입니다(ch16 Q16.9). crates.io의 설명은 다음과 같습니다(이 챕터는 설치하지 않았습니다).

| 대상 | 크레이트 | crates.io 설명 |
| --- | --- | --- |
| Python | `pyo3` 0.30.1 | Bindings to Python interpreter |
| Node.js | `napi` 3.16.2 | N-API bindings |
| Java(JVM, Android) | `jni` 0.22.4 | Rust bindings to the JNI |
| Kotlin·Swift·Python 등 | `uniffi` 0.32.2 | a multi-language bindings generator for rust |
| 웹(JavaScript) | `wasm-bindgen` 0.2.130 | Easy support for interacting between JS and Rust. |
| Rust 플러그인을 실행 중에 불러오기 | `libloading` 0.9.1 | Bindings around the platform's dynamic library loading primitives with greatly improved memory safety. |

Node.js 문서는 Node-API가 JavaScript 엔진과 독립적이고 Node.js 버전 사이에서 ABI가 안정적이라, 한 메이저 버전용으로 컴파일한 애드온을 다시 컴파일하지 않고 뒤 버전에서 쓸 수 있다고 적습니다. Node-API는 네이티브 애드온을 만드는 API이고, Java의 JNI 명세는 네이티브 메서드를 `Java_`로 시작하는 이름이나 `RegisterNatives`로 찾으며 첫 인자로 JNI 인터페이스 포인터를 넘긴다고 정합니다. 그래서 이 두 곳에는 그 규약에 맞춘 함수가 필요합니다. C#과 Go는 위 표처럼 정해진 크레이트 없이, Rust가 C ABI로 내보낸 함수를 P/Invoke와 cgo로 부르는 방식으로 이을 수 있습니다. ch16 Q16.9에서 본 것처럼 Rust로 만든 플러그인끼리도 실행 중에 불러온다면 같은 C ABI 규칙을 따라야 합니다.

**다른 언어에서는** 이 언어들끼리도 같은 방법을 씁니다. Python의 C 확장, Node.js의 네이티브 애드온, Java의 JNI 라이브러리는 대개 C나 C++로 쓰였고, Rust는 그 C 자리에 들어가는 것입니다.

**오해와 한계** 경계를 넘을 때마다 데이터를 C 타입으로 바꾸거나 복사해야 하고, 상대 언어의 런타임 규칙을 지켜야 합니다. 예를 들어 Go의 cgo 문서는 C 코드가 Go 포인터의 사본을 호출이 끝난 뒤까지 쥐고 있으면 안 된다고(고정하지 않았다면) 적습니다. 또 네이티브 확장은 플랫폼마다 따로 빌드해 배포해야 하므로, 순수한 Python·JavaScript 패키지보다 배포가 복잡해집니다.

**더 읽기** ch16 Q16.9·Q16.12 / Python `ctypes` 문서 / Node.js "Node-API" / JEP 454

### Q17.5 경계의 `unsafe`는 안전성을 얼마나 깎나요?

**짧은 답** 경계 너머의 C 코드는 Rust의 보장 밖에 있고, 같은 프로세스 안이라 C의 메모리 버그는 Rust 쪽 메모리도 망가뜨릴 수 있습니다. 경계 코드의 `unsafe` 블록은 포인터가 유효한지, 길이와 라이프타임이 맞는지 같은 전제 조건을 컴파일러 대신 사람이 확인해야 합니다. 그래도 `unsafe`를 작은 모듈에 가두고 안전한 API로 감싸면, 그 API를 쓰는 나머지 Rust 코드는 계속 컴파일러의 검사를 받습니다. 깎이는 범위는 경계 코드를 얼마나 작고 분명하게 만드느냐에 달렸습니다.

**왜 그런가** Chromium의 보안 문서 "The Rule Of 2"는 Rust의 `unsafe` 같은 장치가 있다고 해서 그 주변 코드의 메모리 안전성이 사라지지는 않지만 `unsafe`를 어떻게 쓰느냐가 결정적이라고 적고, `unsafe`가 작고(Small), 안전한 API 뒤에 감춰지고(Encapsulated), 전제 조건을 주석으로 밝히기(Documented)를 요구합니다. `version_swap`의 `src/lib.rs`는 그 모양입니다. 형식을 판단하는 `parse`는 안전한 Rust이고 단위 테스트를 받으며, `unsafe`는 C에서 받은 포인터를 읽고 쓰는 `semver_parse` 안의 두 블록뿐이고, 블록마다 `SAFETY:` 주석으로 전제 조건을 적었습니다. 예제 01의 `c_strtol`도 같은 모양입니다.

전제 조건이 없는 C 함수는 `unsafe extern` 블록 안에서 `safe fn`으로 선언해 `unsafe` 없이 부를 수 있고, 그렇게 선언하지 않은 함수를 `unsafe` 블록 밖에서 부르면 E0133입니다. 어떤 함수를 `safe fn`으로 선언할 수 있는지는 부록 A ch34 예제 `05_safe_fn_judgment`에서 봤습니다. `unsafe`가 허락하는 연산과 그 안에서도 유지되는 검사는 [ch3](../ch3-safety-limits/README.md) Q3.2에서 봤습니다.

**다른 언어에서는** [ch2](../ch2-memory-safety/README.md) Q2.5에서 봤듯이 Java·Kotlin의 JNI, Go의 `unsafe` 패키지도 메모리 안전하지 않은 코드의 경계를 드러냅니다. Go의 cgo 문서는 포인터를 넘기는 규칙을 실행 중에 검사하지만(`GODEBUG=cgocheck`), `unsafe` 패키지로 그 검사를 피할 수 있고 C 코드가 무엇을 하든 막을 수 없다고 적습니다. Python `ctypes` 문서도 이 모듈이 Python의 안전장치를 우회해 프로세스의 메모리에 낮은 수준으로 접근하며, 잘못 쓰면 데이터와 객체를 망가뜨리거나 프로세스를 중단시킬 수 있다고 경고합니다.

**오해와 한계** Rust로 감쌌다고 C 라이브러리 안의 버그가 고쳐지지는 않습니다. 래퍼가 확인할 수 있는 것은 NULL, 길이, 상태 코드처럼 경계에서 보이는 것뿐이고, C가 넘긴 포인터가 정말 유효한지는 확인할 수 없습니다. Rust 코드의 메모리 안전성 취약점이 `unsafe`를 쓰는 곳에서 나온 사례는 ch3 Q3.8에서 봤습니다.

**더 읽기** ch3 Q3.2·Q3.8 / ch2 Q2.5 / 부록 A ch34 / Chromium "The Rule Of 2"

### Q17.6 C 코드를 자동으로 Rust로 바꿀 수 있나요?

**짧은 답** C를 Rust로 옮겨 주는 도구는 있지만, 안전하고 관용적인 Rust를 자동으로 얻는 단계는 아닙니다. C2Rust의 `c2rust transpile`은 C99 코드를 원래 C와 거의 같은 모양의 `unsafe` Rust로 옮기고, 그 결과를 안전한 Rust로 바꾸는 일은 사람이나 다른 도구에 남깁니다. 미국 DARPA의 TRACTOR는 숙련된 Rust 개발자 수준의 자동 번역을 목표로 하는 연구 프로그램입니다.

**왜 그런가** C2Rust의 README에 따르면 번역기는 기능 보존을 우선하고, 안전한 Rust로 바꾸는 나머지 일은 사람, 대규모 언어 모델(LLM), 결정적인 도구 또는 그 조합이 맡을 수 있다고 적습니다. LLM을 쓰는 후처리도 오류를 들일 수 있어 튼튼한 테스트 모음과 함께 쓰라고 권하고, 번역한 코드를 원래 코드와 대조하는 방법도 안내합니다. DARPA의 TRACTOR 페이지는 오래된 C 코드를 Rust로 자동 번역하되 숙련된 Rust 개발자가 만들 품질과 스타일을 얻는 것을 목표로 하고, 정적·동적 분석과 LLM 같은 기계 학습 기법을 함께 쓴다고 소개합니다.

번역에서 원래 동작을 지키는 일이 왜 까다로운지는 예제 01의 2번 구역에서 볼 수 있습니다. `strtol`을 안전한 Rust로 옮긴 첫 판은 공백 판단에 `u8::is_ascii_whitespace`를 썼는데, 이 함수는 C의 `isspace`와 달리 세로 탭(U+000B)을 공백으로 보지 않습니다(표준 라이브러리 문서의 경고). 입력마다 C의 결과와 대조하자 그 차이가 드러났습니다.

```text
2. strtol을 옮긴 Rust 함수를 C의 결과와 대조
  첫 판(is_ascii_whitespace): 9개 중 8개 같음, 다른 입력: "\u{b}42"
  고친 판(C의 isspace 목록): 9개 중 9개 같음, 다른 입력: 없음
```

**다른 언어에서는** Kotlin의 IntelliJ IDEA 플러그인에는 Java 파일을 Kotlin으로 바꾸는 변환기(J2K)가 있고, Kotlin 문서는 이 변환기가 완벽하지 않아 손으로 고칠 일이 생긴다고 적습니다. 같은 런타임과 비슷한 타입 체계를 가진 두 언어 사이에서도 자동 변환은 출발점입니다.

**오해와 한계** 이 챕터는 C2Rust와 TRACTOR를 실행해 보지 않았고, 두 프로젝트가 스스로 밝힌 범위만 옮겼습니다. 자동 번역이 기능을 보존할수록 원래 코드의 버그(`version_swap`의 오버플로 같은)도 함께 옮겨지므로, 번역 뒤에 그 동작을 유지할지 고칠지는 사람이 정해야 합니다.

**더 읽기** C2Rust README / DARPA TRACTOR / Q17.7

### Q17.7 어디부터 옮기는 것이 좋은가요?

**짧은 답** 새 코드부터입니다(Q17.1). 기존 코드를 옮긴다면 세 가지를 함께 봅니다. 신뢰할 수 없는 입력을 다루는가(파서, 디코더, 네트워크 프로토콜), 인터페이스가 좁은가(함수 몇 개와 단순한 타입으로 경계를 그을 수 있는 잎(leaf) 모듈), 테스트로 원래 구현과 동작을 대조할 수 있는가입니다. 옮긴 뒤에는 같은 입력에서 원래 구현과 출력을 비교하고, 달라진 곳마다 고칠지 맞출지 정합니다.

**왜 그런가** Chromium의 "The Rule Of 2"는 신뢰할 수 없는 입력, 안전하지 않은 구현 언어, 높은 권한 셋 가운데 둘까지만 고르라고 합니다. 셋이 겹친 코드의 버그가 심각한 취약점이 되기 때문입니다. 신뢰할 수 없는 입력을 다루는 부품을 메모리 안전한 언어로 옮기면 그 가운데 하나가 빠집니다.

`version_swap`은 좁은 인터페이스의 잎 함수 하나를 옮기는 과정입니다. 헤더 `semver.h`는 그대로 두고, C 프로그램 `main.c`도 그대로 둔 채 링크하는 구현만 바꿉니다. `run.sh`가 두 실행 파일에 같은 입력 13줄을 넣고 출력을 `diff`로 비교합니다.

```text
== 3. 두 출력의 차이(diff)
12c12
< "4294967296.0.0" -> 0.0.0
---
> "4294967296.0.0" -> 형식 오류
```

C 구현은 각 자리를 `uint32_t`로 쌓으며 오버플로를 검사하지 않아 2^32를 0으로 읽었고(부호 없는 정수라 미정의 동작은 아님), Rust 구현은 `u32`로 파싱하다 범위를 넘으면 형식 오류로 봅니다. 대조가 찾아낸 차이는 둘 중 하나로 처리합니다. 원래 동작이 버그라면 Rust 쪽 동작을 새 사양으로 정해 문서와 테스트에 남기고, 다른 코드가 그 동작에 기대고 있다면 Rust 쪽을 원래 동작에 맞춥니다(연습 1).

**다른 언어에서는** 이 기준은 Rust만의 것이 아닙니다. "The Rule Of 2"는 메모리 안전한 언어로 Go, Rust, Python, Java, JavaScript, Kotlin, Swift를 꼽습니다. 기존 C/C++ 프로그램 안에서 함수 하나를 바꿔 끼우는 방식이 가능한 것은 Rust가 런타임 없이 C와 같은 프로세스에서 함수 단위로 이어지기 때문입니다(Q17.2).

**오해와 한계** 잎 모듈부터 옮기면 위험이 작은 대신 효과도 작을 수 있습니다. 경계를 넘는 호출이 잦은 곳을 쪼개 옮기면 변환 비용과 경계 코드가 늘어나므로, 경계는 데이터가 한 번에 오가는 곳에 긋습니다. 대조할 입력은 실제 입력과 경계값(빈 문자열, 최댓값, 공백 문자 등)을 섞어야 차이가 드러납니다.

**더 읽기** Q17.1·Q17.6 / Chromium "The Rule Of 2" / 부록 A ch34

### Q17.8 팀에 도입할 때 드는 비용은 무엇인가요?

**짧은 답** 학습 시간, 느린 빌드, 빌드와 CI에 더해지는 두 번째 툴체인, 경계 코드와 바인딩의 유지(헤더가 바뀌면 함께 고침), 두 언어를 모두 읽는 리뷰어가 비용입니다. Google은 2022년에 Rust 코드를 작성해 커밋한 1,000명 이상의 사내 개발자를 바탕으로, 학습 기간은 경험상 다른 언어를 들일 때와 비슷했고 가장 큰 불만은 빌드 속도였다고 밝혔습니다. 반대쪽 저울에는 Android 팀이 측정한 변경의 안정성과 리뷰 시간이 있습니다.

**왜 그런가** Google 오픈소스 블로그의 "Rust fact vs. fiction"(2023-06-27)에 따르면, 응답자의 3분의 2 이상이 Rust를 배우기 시작한 지 두 달 안에 Rust 코드베이스에 기여할 자신이 생겼고, 3분의 1은 두 달 안에, 절반 이상은 넉 달 안에 다른 언어만큼 생산적이 되었습니다. 같은 글은 느린 빌드가 가장 많이 꼽힌 어려움이었고 속도가 괜찮다고 답한 사람은 40%를 조금 넘었다고 적습니다. Android 팀의 2025-11-13 글은 Rust 변경이 C++ 변경보다 코드 리뷰에 쓰는 시간이 약 25% 짧고, 중간 이상 크기의 변경에서는 롤백률이 약 4배 낮다고 보고합니다.

빌드 쪽 비용은 `version_swap`에서도 보입니다. 같은 `main.c`를 C 구현과 링크한 실행 파일은 33,736바이트, Rust 정적 라이브러리와 링크한 실행 파일은 약 1.47 MB였습니다(릴리스 빌드, 심볼을 지우지 않음). macOS에서는 실행 파일에 정적 라이브러리의 절대 경로가 디버그 정보로 기록되므로, 이 크기는 빌드한 디렉터리에 따라 수십~수백 바이트씩 달라집니다. The Rust Reference에 따르면 `staticlib`은 크레이트의 코드와 그것이 기대는 모든 상위 의존성을 담는 정적 라이브러리라, 표준 라이브러리까지 들어 있는 `libsemver_rs.a`(18,933,608바이트)에서 쓰는 부분이 실행 파일에 링크됩니다. 크기를 줄이는 방법은 [ch14](../ch14-performance/README.md) Q14.7에서 봤습니다. 컴파일 시간을 줄이는 방법은 [ch15](../ch15-pain-points/README.md) Q15.6에, 배우기 어렵다는 말의 정체는 Q15.1에 있습니다.

**다른 언어에서는** 어느 언어를 들이든 교육과 도구 비용은 듭니다. Google의 같은 글은 이 학습 기간이 사내외에서 개발자가 다른 언어를 들일 때 본 기간과 비슷했다고, 일화적인 관찰(anecdotally)이라는 단서를 달아 적습니다. 미국 CISA와 NSA의 합동 지침 "Memory Safe Languages: Reducing Vulnerabilities in Modern Software Development"(2025-06-24)도 메모리 안전한 언어를 들일 때의 주요 장애물과 해법을 다루며, 언어 차원의 보호, 라이브러리 지원, 도구, 개발자 교육을 권고 항목으로 듭니다(CISA 페이지의 소개 기준).

**오해와 한계** Google의 수치는 사내 교육 자료와 지원이 있는 한 회사의 자기 보고이고 2022년 자료이므로, 다른 조직에 그대로 옮길 수는 없습니다. Android의 롤백률과 리뷰 시간도 그 조직의 측정입니다. 비용은 대부분 처음에 크고 시간이 지나며 줄어드는 종류라, 도입 규모가 작으면 고정 비용(빌드 통합, 교육)의 비중이 커집니다.

**더 읽기** ch15 Q15.1·Q15.6 / ch14 Q14.7 / Google "Rust fact vs. fiction" / Android "Rust in Android: move fast and fix things"

### Q17.9 기존 빌드 시스템(CMake, Bazel)에 Rust를 넣을 수 있나요?

**짧은 답** 넣을 수 있습니다. 가장 단순한 방법은 Cargo로 정적 라이브러리를 만들고 기존 빌드가 그 `.a`를 링크하는 것입니다(`version_swap`의 `run.sh`). CMake에는 `Cargo.toml`의 라이브러리와 실행 파일을 CMake 타깃으로 가져오는 Corrosion이, Bazel에는 Rust 규칙 모음인 rules_rust가 있고, Meson은 Rust를 지원 언어로 다룹니다. 반대로 Rust 프로젝트 안에서 C 코드를 함께 빌드할 때는 빌드 스크립트를 씁니다(Part 3 ch8).

**왜 그런가** `run.sh`는 두 단계입니다. `cargo build --release`가 `libsemver_rs.a`를 만들고, `cc`가 그것을 C 소스와 함께 링크합니다. 어떤 빌드 시스템이든 이 두 단계를 자기 규칙으로 표현하면 됩니다. 덧붙일 시스템 라이브러리는 `--print native-static-libs`가 알려 줍니다(Q17.2). Corrosion의 README는 이 도구가 기존 CMake 프로젝트에 Rust를 통합하며 Cargo 매니페스트에서 실행 파일, 정적 라이브러리, 동적 라이브러리를 자동으로 가져온다고 소개합니다. rules_rust 저장소의 설명은 "Rust rules for Bazel"이고, Meson 문서에는 "Using Rust with Meson" 장이 있습니다. 이 챕터는 셋 다 설치하지 않았습니다.

**다른 언어에서는** [ch13](../ch13-headers-build-templates/README.md) Q13.4에서 봤듯이 C/C++의 빌드와 패키지 관리는 표준 밖의 도구가 맡고, Rust는 Cargo가 한 도구로 맡습니다. 그래서 C/C++ 빌드에 Rust를 넣을 때는 Cargo를 기존 빌드 안에서 부르거나, 빌드 시스템이 `rustc`를 부르는 규칙을 갖추는 두 갈래가 생깁니다.

**오해와 한계** 의존성 관리가 둘이 됩니다. crates.io의 의존성과 `Cargo.lock`을 기존 빌드의 재현성 정책(오프라인 빌드, 버전 고정)과 어떻게 맞출지 정해야 합니다(ch16 Q16.3). 크로스 컴파일하는 프로젝트라면 Rust 타깃과 C 툴체인의 타깃을 함께 맞춰야 합니다. 이 챕터의 `run.sh`는 macOS의 기본 링크 설정에 기대므로, 다른 플랫폼에서는 `--print native-static-libs`의 결과를 링크 명령에 더해야 할 수 있습니다.

**더 읽기** ch13 Q13.4 / Part 3 ch8 / Corrosion / rules_rust / Meson "Rust"

### Q17.10 도입을 판단할 때 무엇을 물어야 하나요?

**짧은 답** 다섯 묶음을 묻습니다. 풀려는 문제가 무엇인가(메모리 안전성 취약점, 성능, 동시성 버그), 옮길 부분이 알맞은가(신뢰할 수 없는 입력, 좁은 경계), 동작을 대조할 테스트가 있는가, 대상 플랫폼과 필요한 라이브러리가 갖춰져 있는가, 팀이 학습과 빌드 통합에 시간을 들일 수 있는가입니다. 그리고 GC 언어나 C++ 하드닝 같은 대안과 비교합니다.

**왜 그런가** 이 질문들은 앞의 답을 판단 순서로 모은 것입니다.

| 묻는 것 | 볼 곳 |
| --- | --- |
| 풀려는 문제가 Rust가 막는 종류인가, 막지 않는 종류(논리 버그, 교착 상태)인가 | ch2, ch3 |
| 옮길 부분이 신뢰할 수 없는 입력을 다루고 경계가 좁은가 | Q17.7 |
| 원래 구현과 출력을 대조할 테스트와 입력이 있는가 | Q17.6, Q17.7 |
| 대상 플랫폼의 지원 등급과 필요한 크레이트의 상태는 어떤가 | ch16 Q16.3·Q16.12 |
| 경계를 C로 그을 수 있는가, C++·다른 언어라면 도구가 감당하는가 | Q17.2~Q17.4 |
| 학습, 빌드 시간, 빌드 통합, 리뷰에 들일 여력이 있는가 | Q17.8, Q17.9 |
| GC 언어나 C++ 하드닝으로 충분하지 않은가 | ch1 Q1.6, ch12 Q12.4 |

**다른 언어에서는** Go나 Java 같은 다른 메모리 안전한 언어를 들일 때도 같은 질문을 합니다. Rust를 고르는 쪽으로 기우는 것은 GC 없이 기존 C/C++와 같은 프로세스에서 함수 단위로 섞어야 할 때(Q17.2, Q17.7)이고, 그런 제약이 없다면 GC 언어가 더 싼 경우가 많습니다(ch1 Q1.6).

**오해와 한계** 이 표는 결정을 대신하지 않습니다. 이 챕터가 인용한 Android와 Google의 수치는 큰 조직이 오랜 기간 측정한 것이라, 작은 팀이 시험적으로 들일 때의 비용과 효과는 직접 작게 시작해 재 보는 수밖에 없습니다.

**더 읽기** ch1 Q1.6 / ch2·ch3 / ch16 Q16.12 / Q17.7·Q17.8

### 한눈에 보기

| 질문 | Rust의 답 | 비용과 한계 |
| --- | --- | --- |
| 다시 쓰기 | 새 코드부터, 상호 운용에 투자(Android) | 기존 코드의 위험은 남음, 다시 쓰면 동작이 달라짐 |
| C와 잇기 | `unsafe extern "C"`, `#[unsafe(no_mangle)] extern "C"`, `staticlib`·`cdylib` | 경계는 C 타입과 약속뿐 |
| C++와 잇기 | `extern "C"` 층, `cxx`, autocxx | 넘길 수 있는 타입이 제한됨 |
| 다른 언어 | C ABI + 각 언어의 FFI, PyO3·napi-rs·`jni`·UniFFI | 변환 비용, 플랫폼별 배포 |
| 경계의 `unsafe` | 작게 가두고 안전한 API로 감쌈 | C 코드의 버그는 그대로 |
| 자동 번역 | C2Rust(`unsafe` Rust), TRACTOR(연구) | 안전한 Rust로의 전환은 남는 일 |
| 어디부터 | 새 코드, 신뢰할 수 없는 입력, 좁은 경계, 대조할 테스트 | 원래 동작과의 차이를 정해야 함 |
| 비용 | 학습, 빌드 시간, 두 번째 툴체인, 바인딩 유지 | 조직마다 다름 |
| 빌드 통합 | 정적 라이브러리 링크, Corrosion, rules_rust, Meson | 의존성 관리가 둘 |

## 3. 예제 안내

로드맵의 예제 계획 가운데 "libc 함수 호출"은 Part 1 ch14 예제 `04_calling_c`가 `strlen`·`getenv`로 이미 보였으므로, 이 챕터는 C 함수와 Rust 함수의 동작 차이를 보이는 쪽으로 바꿨습니다. "C 프로그램의 함수 하나를 Rust 구현으로 바꿔 끼우기"는 `projects/version_swap`입니다. Q17.3의 C++ 링크 이름은 `compare/`의 비교 코드로 확인합니다.

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_strtol_vs_parse](examples/01_strtol_vs_parse.rs) | C의 `strtol`과 Rust의 `parse::<i64>`의 차이, `strtol`을 옮긴 Rust 함수를 C와 대조 | Q17.1·Q17.2·Q17.6 |
| [projects/version_swap](projects/version_swap/Cargo.toml) | C 프로그램의 함수 하나를 Rust 정적 라이브러리로 바꿔 끼우고 출력 대조 | Q17.2·Q17.5·Q17.7·Q17.9 |
| [compare/01_cpp_mangling.cpp](compare/01_cpp_mangling.cpp) | C++ 함수와 `extern "C"` 함수의 링크 이름 | Q17.3 |

### 01_strtol_vs_parse — 같은 일을 하는 C 함수와 Rust 함수

```sh
cargo run -p faq-ch17-interop-migration --example 01_strtol_vs_parse
```

```text
1. strtol과 parse::<i64>
  "42"                     strtol: 42 (2바이트 읽음) / parse: Ok(42)
  " 42"                    strtol: 42 (3바이트 읽음) / parse: Err(InvalidDigit)
  "+42"                    strtol: 42 (3바이트 읽음) / parse: Ok(42)
  "42abc"                  strtol: 42 (2바이트 읽음) / parse: Err(InvalidDigit)
  "abc"                    strtol: 0 (0바이트 읽음) / parse: Err(InvalidDigit)
  ""                       strtol: 0 (0바이트 읽음) / parse: Err(Empty)
  "9223372036854775808"    strtol: 9223372036854775807 (19바이트 읽음) / parse: Err(PosOverflow)
  "-9223372036854775809"   strtol: -9223372036854775808 (20바이트 읽음) / parse: Err(NegOverflow)
  "\u{b}42"                strtol: 42 (3바이트 읽음) / parse: Err(InvalidDigit)
2. strtol을 옮긴 Rust 함수를 C의 결과와 대조
  첫 판(is_ascii_whitespace): 9개 중 8개 같음, 다른 입력: "\u{b}42"
  고친 판(C의 isspace 목록): 9개 중 9개 같음, 다른 입력: 없음
```

- `strtol`은 `unsafe extern "C"` 블록에 선언했고, 원시 포인터를 받아 그 너머를 읽고 쓰므로 `safe fn`으로 선언할 수 없습니다. `c_strtol`이 두 `unsafe` 블록에 `SAFETY:` 주석을 달아 감쌉니다.
- 범위를 넘는 두 입력에서 `strtol`은 숫자를 모두 읽은 뒤(19·20바이트) 값을 `long`의 최댓값과 최솟값으로 잘랐습니다. macOS의 `man strtol`은 이때 `errno`를 `ERANGE`로 둔다고 적습니다. 이 예제는 `errno`를 읽지 않습니다.
- `c_long`은 이 환경에서 `i64`입니다. `c_long`이 32비트인 플랫폼(Windows 등)에서는 범위를 넘는 입력의 결과가 다르고, 예제의 `#[allow(clippy::useless_conversion)]`도 그 차이 때문에 둡니다.
- `parse::<i64>`가 받아들이지 않는 입력에서 `strtol`이 값을 돌려주는 것은 오류가 아니라 정해진 동작입니다. 옮길 때 문제가 되는 것은 어느 쪽이 맞느냐가 아니라 둘이 다르다는 사실입니다.

### projects/version_swap — C 프로그램의 함수 하나를 Rust로 바꿔 끼우기

`csrc/semver.h`가 `semver_parse` 하나를 선언하고, 원래 구현 `csrc/semver.c`와 Rust 구현 `src/lib.rs`가 같은 선언을 따릅니다. C 프로그램 `csrc/main.c`는 표준 입력의 줄마다 `semver_parse`를 부르고 결과를 출력하며, 어느 구현과 링크하든 그대로입니다. 외부 크레이트가 없어 네트워크 없이 빌드됩니다.

| 디렉터리 | 명령 | 기대 결과 |
| --- | --- | --- |
| `projects/version_swap` | `./run.sh` | 1번 구역에 C 구현의 출력 13줄, 3번 구역의 `diff`에 12번째 줄 하나만 다름. 종료 코드 0 |
| `projects/version_swap` | `cargo test` | 단위 테스트 4개 통과 |
| `projects/version_swap` | `cargo rustc --release -q -- --print native-static-libs` | `note: native-static-libs: -lSystem -lc -lm`(이 환경) |

```sh
cd appd-b/ch17-interop-migration/projects/version_swap
./run.sh
```

```text
== 1. 원래 프로그램: main.c + semver.c
"1.2.3" -> 1.2.3
"0.10.0" -> 0.10.0
"10.20.30" -> 10.20.30
"1.2" -> 형식 오류
"1.2.3.4" -> 형식 오류
"1..3" -> 형식 오류
"v1.2.3" -> 형식 오류
" 1.2.3" -> 형식 오류
"+1.2.3" -> 형식 오류
"1.2.3-beta" -> 형식 오류
"4294967295.0.0" -> 4294967295.0.0
"4294967296.0.0" -> 0.0.0
"" -> 형식 오류
== 2. 바꿔 끼운 프로그램: main.c + Rust 정적 라이브러리
== 3. 두 출력의 차이(diff)
12c12
< "4294967296.0.0" -> 0.0.0
---
> "4294967296.0.0" -> 형식 오류
```

- `run.sh`는 `cc -std=c11 -Wall -Wextra`로 두 실행 파일을 만들고(`target/app_c`, `target/app_rust`), 출력을 `target/out_c.txt`·`target/out_rust.txt`에 남깁니다. `diff`가 다른 줄을 찾아도 스크립트는 0으로 끝납니다.
- Rust 구현의 `parse`는 `u32::from_str`가 앞의 `+`를 받아들이므로(`"+1".parse::<u32>()`는 `Ok(1)`) 숫자만 있는지 먼저 봅니다. 그래서 `"+1.2.3"`에서도 두 구현이 같습니다.
- 두 실행 파일을 `-fsanitize=address,undefined`로 다시 빌드해 같은 입력을 넣었을 때 새니타이저의 보고는 없었습니다.
- `#[unsafe(no_mangle)]`을 빼고 빌드하면 Rust가 이름을 맹글링해 C 링커가 함수를 찾지 못합니다(4절).
- 확인이 끝나면 `cargo clean`으로 `target/`을 지웁니다.

### compare/01_cpp_mangling.cpp — C++의 링크 이름

| 파일 | 질문 | 명령 | 결과 |
| --- | --- | --- | --- |
| [01_cpp_mangling.cpp](compare/01_cpp_mangling.cpp) | Q17.3 | 파일 머리의 `c++ -std=c++20 -fsanitize=address,undefined` 빌드 후 실행, `nm … \| grep scale` | 실행 출력 `scale(2) = 4, scale(1.5) = 3.0, scale_c(3) = 6`, 새니타이저 보고 없음. `nm`에 `__Z5scaled`, `__Z5scalei`, `_scale_c` |

확인한 컴파일러는 Apple clang 21.0.0(arm64-apple-darwin27.2.0)이고, `nm`의 주소는 빌드마다 다를 수 있습니다. `c++filt`에 넘기면 `__Z5scalei`가 `scale(int)`로 풀립니다.

## 4. 자주 하는 실수

이 챕터의 실수는 컴파일러가 거부하는 코드보다 링크와 동작에서 드러나므로 `compile_fail/`을 두지 않았습니다. `unsafe` 블록 없이 부른 `extern` 함수(E0133), 원시 포인터를 담은 래퍼를 다른 스레드로 보내기(E0277)는 부록 A ch34 4절에 있습니다.

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| 같은 일을 하는 Rust 함수로 바꾸면 동작도 같다고 믿음 | 공백, 남은 글자, 오버플로에서 결과가 다름 | 원래 구현과 같은 입력으로 출력을 대조합니다 | Q17.6, 예제 01 |
| 원래 구현의 버그를 그대로 둘지 정하지 않고 바꿈 | 다른 코드가 기대던 동작이 조용히 바뀜 | 차이마다 고칠지 맞출지 정하고 테스트와 문서에 남깁니다 | Q17.7, `version_swap` |
| 내보낼 함수에 `#[unsafe(no_mangle)]`을 빠뜨림 | 링크 에러 `Undefined symbols for architecture arm64: "_semver_parse"` | `#[unsafe(no_mangle)] pub extern "C" fn`으로 내보냅니다 | Q17.2, 부록 A ch34 4.4절 |
| 헤더의 구조체와 `#[repr(C)]` 구조체의 필드 순서·타입이 다름 | 컴파일과 링크는 되고 값이 틀림 | 선언을 생성하거나 레이아웃을 검사합니다 | 부록 A ch35 |
| Rust 패닉이나 C++ 예외가 경계를 넘게 둠 | 패닉은 프로세스 중단, C++ 예외가 `"C"` 함수를 지나 되감기면 미정의 동작 | 경계에서 잡아 에러 코드로 바꿉니다 | Q17.2, 부록 A ch34 2.8절 |
| 정적 라이브러리가 기대는 시스템 라이브러리를 빠뜨림 | 다른 플랫폼에서 링크 에러 | `--print native-static-libs`의 결과를 링크 명령에 더합니다 | Q17.9 |
| 번역 도구의 출력을 안전한 Rust로 여김 | C2Rust의 출력은 `unsafe` | 안전한 Rust로 바꾸는 일을 계획에 넣습니다 | Q17.6 |

## 5. 연습 문제

### 연습 1. 원래 동작에 맞추기

`version_swap`의 Rust 구현을 C 구현처럼 범위를 넘으면 2^32로 나눈 나머지가 되게 바꾸어 `./run.sh`의 `diff`가 "차이 없음"이 되게 해 보세요. 그다음 `cargo test`를 돌리면 무엇이 실패하고, 그것은 무엇을 뜻할까요?

<details>
<summary>힌트</summary>

`parse` 안의 `piece.parse().ok()?` 대신 자리마다 `value = value.wrapping_mul(10).wrapping_add(u32::from(b - b'0'))`로 쌓습니다(`wrapping_*`는 Part 1 ch14 2.9절). 그러면 두 출력이 같아지고, 단위 테스트 `u32를_넘으면_형식_오류`가 실패합니다. 테스트가 정해 둔 사양("범위를 넘으면 형식 오류")과 원래 C의 동작이 다르다는 뜻이므로, 어느 쪽을 사양으로 삼을지 정하고 테스트를 그에 맞춰야 합니다. 대개는 원래 동작이 버그인지, 그 동작에 기대는 코드가 있는지를 보고 정합니다.

</details>

### 연습 2. 세 번째 판의 공백

예제 01의 `inputs`에 `"\u{a0}42"`(줄 바꿈 없는 공백 NBSP 뒤에 42)를 더하면 `strtol`과 두 판은 무엇을 돌려줄까요? 공백을 건너뛰는 데 `str::trim_start`를 쓰는 세 번째 판을 만든다면 결과는 어떻게 될까요?

<details>
<summary>힌트</summary>

NBSP는 UTF-8로 `0xC2 0xA0` 두 바이트입니다. C 로캘의 `isspace`는 둘 다 공백으로 보지 않으므로 `strtol`은 0바이트를 읽고 0을 돌려주고, 두 판도 같은 결과입니다. `str::trim_start`는 유니코드의 `White_Space` 속성으로 공백을 판단해 NBSP를 지우므로 세 번째 판은 42를 돌려주고, 대조에서 이 입력이 다르게 나옵니다. "공백"의 정의가 함수마다 다르다는 점은 표준 라이브러리의 `u8::is_ascii_whitespace` 문서도 경고합니다.

</details>

### 연습 3. 동료에게 설명해 보기

팀에서 "C로 된 이미지 디코더를 Rust로 다시 쓰자"는 제안이 나왔다고 합시다. Q17.10의 표를 따라, 판단에 필요한 질문과 지금 답할 수 있는 것, 더 알아봐야 할 것을 나눈 메모를 써 보세요.

<details>
<summary>힌트</summary>

이미지 디코더는 신뢰할 수 없는 입력을 다루는 전형적인 부품이라 Q17.7의 기준에 잘 맞습니다. 그래도 물을 것이 남습니다. 디코더의 인터페이스가 "바이트 버퍼를 받아 픽셀 버퍼를 돌려줌"처럼 좁은지, 원래 디코더의 출력과 대조할 시험 이미지 모음이 있는지, 대상 플랫폼의 등급(ch16 Q16.12), 빌드 통합 방법(Q17.9), 팀의 학습 시간(Q17.8)입니다. 전부 다시 쓰기보다 새 형식의 디코더부터 Rust로 쓰는 선택지(Q17.1)도 메모에 넣습니다.

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rustonomicon: [Foreign Function Interface](https://doc.rust-lang.org/nomicon/ffi.html)
- The Rust Reference: [Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html)(되감기를 허용하지 않는 프레임), [Panic](https://doc.rust-lang.org/reference/panic.html)(`"C-unwind"`), [Linkage](https://doc.rust-lang.org/reference/linkage.html)(`staticlib`, `cdylib`)
- 표준 라이브러리 문서: [`u8::is_ascii_whitespace`](https://doc.rust-lang.org/std/primitive.u8.html#method.is_ascii_whitespace)(세로 탭에 대한 경고)
- [CXX](https://cxx.rs/)
- crates.io: [pyo3](https://crates.io/crates/pyo3), [napi](https://crates.io/crates/napi), [jni](https://crates.io/crates/jni), [uniffi](https://crates.io/crates/uniffi), [wasm-bindgen](https://crates.io/crates/wasm-bindgen), [libloading](https://crates.io/crates/libloading), [cbindgen](https://crates.io/crates/cbindgen)

전환 사례와 도구

- Google 보안 블로그: [Eliminating Memory Safety Vulnerabilities at the Source](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html)(2024-09-25)
- Google: [Rust in Android: move fast and fix things](https://blog.google/security/rust-in-android-move-fast-fix-things/)(2025-11-13)
- Google 오픈소스 블로그: [Rust fact vs. fiction: 5 Insights from Google's Rust journey in 2022](https://opensource.googleblog.com/2023/06/rust-fact-vs-fiction-5-insights-from-googles-rust-journey-2022.html)(2023-06-27)
- Chromium: [The Rule Of 2](https://chromium.googlesource.com/chromium/src/+/main/docs/security/rule-of-2.md)
- CISA·NSA: [Memory Safe Languages: Reducing Vulnerabilities in Modern Software Development](https://www.cisa.gov/resources-tools/resources/memory-safe-languages-reducing-vulnerabilities-modern-software-development)(2025-06-24)
- [C2Rust](https://github.com/immunant/c2rust) README
- DARPA: [TRACTOR: Translating All C to Rust](https://www.darpa.mil/research/programs/translating-all-c-to-rust)
- [Corrosion](https://github.com/corrosion-rs/corrosion), [rules_rust](https://github.com/bazelbuild/rules_rust), Meson: [Using Rust with Meson](https://mesonbuild.com/Rust.html)

다른 언어

- Kotlin: [Calling Java from Kotlin](https://kotlinlang.org/docs/java-interop.html), [Adding Kotlin to a Java project](https://kotlinlang.org/docs/mixing-java-kotlin-intellij.html)(J2K 변환기)
- TypeScript: [Migrating from JavaScript](https://www.typescriptlang.org/docs/handbook/migrating-from-javascript.html)
- Swift: [Mixing Swift and C++](https://www.swift.org/documentation/cxx-interop/)
- Python: [ctypes](https://docs.python.org/3/library/ctypes.html)
- Go: [cgo](https://pkg.go.dev/cmd/cgo)
- Java: [JEP 454: Foreign Function & Memory API](https://openjdk.org/jeps/454)
- .NET: [Platform Invoke (P/Invoke)](https://learn.microsoft.com/en-us/dotnet/standard/native-interop/pinvoke)
- Node.js: [Node-API](https://nodejs.org/api/n-api.html)
