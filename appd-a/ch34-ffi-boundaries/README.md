# ch34. FFI 경계를 넘는 것을 통제하기

Rust 코드가 C 라이브러리를 부르거나 C 프로그램이 Rust 함수를 부르는 곳을 FFI 경계(foreign function interface boundary)라고 합니다. 경계 안쪽에서는 컴파일러가 타입, 소유권, 라이프타임, `Send`/`Sync`를 확인해 주지만, 경계를 넘는 순간 그 확인은 모두 사라집니다. 링커는 함수 이름만 맞춰 보고, C 컴파일러는 Rust의 규칙을 모릅니다. 그래서 FFI 코드는 자동으로 `unsafe`이고, [ch16](../ch16-avoid-unsafe/README.md)의 "unsafe를 직접 쓰지 말라"는 조언을 그대로 따를 수 없습니다. 이 챕터는 그 대신 쓸 수 있는 판단을 다룹니다. 경계를 넘는 것(이름, 정수, 구조체, 포인터, 해제 책임, 패닉)을 무엇으로 정할지, 그리고 넘어온 위험을 어떻게 한 곳에 가둘지입니다. 작은 C 라이브러리를 직접 컴파일해 링크하는 프로젝트와, C 코드가 부르는 Rust 함수를 내보내는 프로젝트로 확인합니다.

- 원제: [Item 34: Control what crosses FFI boundaries](https://effective-rust.com/ffi.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch34-ffi-boundaries`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), Apple clang 21.0.0(macOS aarch64), `cc` 크레이트 1.11.2

> **이 챕터의 `unsafe` 코드는 Miri로 확인하지 못했습니다.** 이 환경에는 nightly 툴체인이 없고, Miri는 C 코드를 실행하지도 못합니다. 모든 `unsafe` 블록에 `// SAFETY:` 주석으로 근거를 적고, 그 전제 조건을 바로 위의 코드(NULL 검사, 길이 비교)나 함수의 `# Safety` 절이 대도록 썼습니다. 이중 해제, 해제 후 사용, 어긋난 선언으로 부르는 일처럼 미정의 동작(undefined behavior)이 되는 코드는 실행하지 않고 설명만 합니다.

책의 6장(Beyond Standard Rust)에 있는 Item입니다. 다음 내용을 안다고 보고 진행합니다.

- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.4절: `extern`으로 C 함수 부르기와 내보내기, 이름 맹글링
- [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 2.4~2.5절: `CString`/`CStr`, `c_char`의 부호, `&CStr`을 받는 안전한 래퍼, 임시 `CString`의 `as_ptr()` 함정
- [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.7~2.8절: `unsafe extern` 블록, `safe fn`/`unsafe fn` 항목, `#[unsafe(no_mangle)]`
- [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 2.9절, [ch18](../ch18-dont-panic/README.md) 2.5절: `catch_unwind`와 그 한계
- [ch16](../ch16-avoid-unsafe/README.md) 2.5~2.6절: 안전한 래퍼와 안전성 주석, [ch11](../ch11-drop-raii/README.md) 2.2절: `Drop`으로 자원 해제

앞 챕터가 넘긴 것도 받습니다. [ch6](../ch6-newtype-pattern/README.md)의 `#[repr(transparent)]`(2.4절), [ch25](../ch25-dependency-graph/README.md)의 `links` 키를 적는 이유(2.3절), ch16의 "FFI 경계의 unsafe를 래퍼에 가두기"와 [ch17](../ch17-shared-state-parallelism/README.md)의 "C 코드는 `Send`/`Sync` 검사를 받지 않는다"(2.6절), ch18의 FFI 경계를 넘는 패닉(2.8절)입니다.

다음은 뒤로 미룹니다. `CString::into_raw`/`from_raw`의 상세, `CStr::from_ptr`이 돌려주는 참조의 라이프타임, `Option<NonZero<T>>`의 FFI 호환 보장, 콜백과 불투명 타입 선언의 세부는 Part 10 `ch12-ffi-beneath-std`에서, 레이아웃 규칙 전체는 Part 10 `ch2-data-layout`에서, ABI와 링크 방식·크레이트 타입은 Part 9 `ch9-memory-panic-linkage`에서, 빌드 스크립트 전반은 Part 3 `ch8-build-scripts`에서 다룹니다. 헤더에서 선언을 생성하는 일과 손으로 쓴 선언이 어긋났을 때의 재현은 [`ch35-bindgen`](../ch35-bindgen/README.md)이 이 챕터의 `projects/tally_ffi`를 가져가서 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. FFI 호출에서 링커와 rustc가 각각 무엇을 확인하고 무엇을 확인하지 못하는지(이름만 맞추는 링커, 맹글링, `improper_ctypes`) 설명하고, 크기가 정해진 정수 타입으로 C 선언을 옮길 수 있다.
2. C 라이브러리를 빌드 스크립트와 `cc` 크레이트로 컴파일해 링크하고, `#[link]` 속성과 빌드 스크립트 중 무엇을 쓸지, `links` 키를 왜 적는지 설명할 수 있다.
3. `#[repr(C)]`와 `#[repr(transparent)]`로 경계를 넘는 타입의 배치를 고정하고, C의 enum 값을 미정의 동작 없이 받을 수 있다.
4. 할당과 해제를 경계의 같은 쪽에 두고, C 자원을 `Drop`과 `Result`를 갖춘 안전한 래퍼에 가두며, `safe fn` 선언과 래퍼의 `Send`/`Sync`를 근거를 들어 정할 수 있다.
5. C에서 부를 Rust 함수를 접두사, `#[unsafe(no_mangle)]`, NULL 검사, `catch_unwind`를 갖춰 내보낼 수 있다.

## 2. 핵심 개념

2.1절은 경계에서 실제로 무슨 일이 일어나는지 정리하는 **이해** 부분입니다. 2.2~2.8절이 지침이고, 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 씁니다. 지침 1~7의 굵은 줄기는 책이 권하는 것이고, 각 지침에 더한 판단 기준 중 책에 없는 것은 2.9절에 "책에 없는 지침"으로 밝혔습니다.

### 2.1 지도: 경계를 넘는 것과 확인하는 쪽

C는 프로그래밍 언어 사이의 최소 공통분모입니다. Rust와 다른 언어가 만나는 곳은 거의 언제나 C의 규칙(호출 규약, 구조체 배치, 0 바이트로 끝나는 문자열)을 따릅니다. 경계를 넘는 것과 그것을 누가 확인하는지를 표로 정리하면 이렇습니다.

| 경계를 넘는 것 | 확인하는 쪽 | 다루는 곳 |
| --- | --- | --- |
| 함수와 전역 변수의 이름(심볼) | 링커. 그 이름이 있는지만 봄 | 2.1, 2.3, 2.7 |
| 인자와 반환값의 타입 | 없음. rustc는 C에 대응하는 타입이 있는지만 봄(`improper_ctypes`) | 2.2 |
| 구조체의 메모리 배치 | 없음. `#[repr(C)]`로 C와 같은 규칙을 고름 | 2.4 |
| 누가 해제하는가, 포인터가 언제까지 유효한가 | 없음. 헤더의 문서뿐 | 2.5, 2.6 |
| 어느 스레드에서 써도 되는가 | 없음. 래퍼의 `Send`/`Sync`로 옮겨 적음 | 2.6 |
| 패닉(되감기) | rustc. `extern "C"` 경계에서 프로세스를 중단 | 2.8 |

**링커는 이름만 봅니다.** C 함수의 심볼은 함수 이름 그대로이고, 이 환경(macOS)의 목적 파일 형식은 앞에 `_`를 붙입니다. 타입 정보는 없습니다. 반면 Rust와 C++는 같은 이름의 함수가 여럿 있어도 겹치지 않게 경로와 타입을 심볼에 넣는 이름 맹글링(name mangling)을 합니다. 책이 쓰는 `nm`과 `c++filt`로 C++ 쪽을 보면 차이가 분명합니다(같은 이름의 C++ 함수 둘을 컴파일해 확인).

```text
$ nm cpp.o                     # 주소 열은 지웠습니다
T __Z12tally_recordP5tallyd
T __Z12tally_recordP5tallyx
$ nm cpp.o | c++filt
tally_record(tally*, double)
tally_record(tally*, long long)
```

C 심볼 `_tally_record`에는 이런 정보가 없으므로, Rust 쪽 선언의 매개변수가 C와 달라도 링커는 그대로 연결합니다. 어긋남은 실행 중에야 잘못된 값이나 망가진 메모리로 드러납니다(2.2절).

**선언과 정의의 심볼.** 다음 파일을 `rustc --edition 2024 --crate-type lib --crate-name probe --emit obj -o probe.o probe.rs`로 컴파일하고 `nm probe.o`로 보았습니다.

```rust
unsafe extern "C" {
    safe fn hypot(x: f64, y: f64) -> f64; // 선언: 본문은 C 라이브러리에 있음
}
pub extern "C" fn probe_plain(x: f64) -> f64 { hypot(x, 4.0) }
#[unsafe(no_mangle)]
pub extern "C" fn probe_exported(x: f64) -> f64 { hypot(x, 4.0) }
```

```text
T __RNvCs…_5probe11probe_plain   # 정의, 속성 없음: 맹글링됨 (…은 크레이트 해시)
U _hypot                         # 선언: 맨 이름으로 "다른 곳에 있다(U)"고 요청
T _probe_exported                # 정의, #[unsafe(no_mangle)]: 맨 이름
```

책은 "`extern "C"` 표시가 함수를 자동으로 `#[no_mangle]`로 만든다"고 적는데, 이것은 **`extern` 블록 안의 선언**에만 맞습니다. Rust에서 **정의하는** `extern "C" fn`은 호출 규약만 C로 바뀌고 이름은 여전히 맹글링되므로, C가 이름으로 찾게 하려면 `#[unsafe(no_mangle)]`이 필요합니다(책도 "C에서 Rust 부르기" 절에서는 그렇게 씁니다). `projects/tally_ffi`의 rlib에서도 `nm -u target/debug/libtally.rlib`가 Rust 쪽 선언이 요청하는 `_tally_new`, `_tally_free` 같은 맨 이름 9개를 보여 주고, 실행 파일에서 래퍼의 `Tally::new`는 `__RNvMs2_Cs…_5tallyNtB5_5Tally3new`처럼 맹글링된 이름입니다. 1.100.0의 Rust 심볼은 `_R`로 시작하는 v0 형식(이 환경에서는 앞에 `_`가 하나 더 붙음)이라 `c++filt`로는 풀리지 않습니다. 책의 각주가 소개하는 `rustfilt`는 이 환경에 설치돼 있지 않아 실행하지 못했습니다.

**2024 에디션의 표시.** `extern` 블록은 `unsafe extern "C" { … }`로 쓰고, 그 안의 항목에 `safe fn`/`unsafe fn`을 붙일 수 있으며(표시가 없으면 `unsafe fn`), `no_mangle`·`export_name`·`link_section`은 `#[unsafe(…)]`로 감쌉니다. 문법은 [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.7~2.8절에서 배웠습니다. 이 챕터는 그 표시를 **어디에** 붙일지를 판단합니다(2.6절).

### 2.2 지침 1: 선언은 헤더와 정확히 맞추고, 크기가 정해진 정수 타입을 쓴다

**문제 상황.** C 헤더의 함수를 부르려면 Rust 쪽에 같은 시그니처의 선언을 적어야 합니다. 그런데 C의 `int`, `long`은 크기가 플랫폼마다 다르고, 선언이 C와 어긋나도 2.1절대로 아무도 알려 주지 않습니다.

**비권장 코드.** 파일 크기를 돌려주는 C 함수가 `long file_size(const char *path);`이고, Rust 선언도 `fn file_size(path: *const c_char) -> c_long;`입니다. 선언은 헤더와 맞지만, [`01_sized_types.rs`](examples/01_sized_types.rs)의 1번 구역에서 보듯 이 환경의 `c_long`은 `i64`(8바이트)이고, 표준 라이브러리 문서에 따르면 Windows에서는 `i32`입니다. 2번 구역은 5 GiB가 32비트 `long`에 담기지 않는다는 것을 `i32::try_from`으로 흉내 냅니다. 게다가 `c_long`은 별칭일 뿐이라, 이 환경에서 `c_long` 값을 `i64`처럼 쓰는 코드는 경고 없이 컴파일되고 다른 플랫폼에서야 깨집니다.

**권장 코드.** C 쪽 헤더부터 `<stdint.h>`의 크기가 정해진 타입으로 쓰고, Rust 선언은 같은 크기의 Rust 정수로 옮깁니다. [`projects/tally_ffi`](projects/tally_ffi/src/ffi.rs)의 선언 모듈은 헤더의 각 선언을 바로 위 주석에 그대로 적어 나란히 대조할 수 있게 했습니다.

```rust
// int32_t tally_record(tally *t, int64_t sample);
pub unsafe fn tally_record(t: *mut tally, sample: i64) -> i32;

// tally_summary tally_summarize(const tally *t);
pub unsafe fn tally_summarize(t: *const tally) -> tally_summary;
```

C의 `const` 포인터는 `*const`로, `const`가 없는 포인터는 `*mut`로 옮깁니다. 이렇게 맞춰도 확인하는 것은 사람입니다. 이 프로젝트의 사본에서 선언의 `sample`을 `i32`로 잘못 옮기고 래퍼의 호출도 그에 맞춘 경우와, `tally_summary`의 필드 순서만 바꾼 경우 모두 `cargo build`와 `cargo clippy`가 아무 경고 없이 통과했습니다(실행하지는 않았습니다). 그래서 선언은 한 파일에 모으고, 손으로 맞추는 대신 헤더에서 생성하는 것이 낫습니다([`ch35-bindgen`](../ch35-bindgen/README.md)).

**rustc가 잡아 주는 것.** 선언에 C에 대응하는 타입이 없는 Rust 타입을 쓰면 `improper_ctypes`(정의 쪽은 `improper_ctypes_definitions`) 경고가 납니다. 01의 `rejected` 모듈은 `&str`, `String`, 튜플, `#[repr(C)]` 없는 구조체, `char`, `&[u8]`을 선언에 썼고, `#[expect]`를 지우면 여섯 개 모두 경고가 납니다.

```text
warning: `extern` block uses type `str`, which is not FFI-safe
  --> 01_sized_types.rs:30:29
   |
30 |         pub fn takes_str(s: &str); // str: 길이를 함께 가진 넓은 포인터
   |                             ^^^^ not FFI-safe
   |
   = help: consider using `*const u8` and a length instead
   = note: string slices have no C equivalent
   = note: `#[warn(improper_ctypes)]` on by default
```

이 린트는 "C에 있을 수 없는 타입"만 잡습니다. `c_long`과 `i32`가 바뀐 것, 필드 순서, `*const`와 `*mut`, `safe fn`이 맞는지는 모릅니다. 같은 크레이트 안에 같은 심볼의 선언이 둘 있고 서로 다르면 `clashing_extern_declarations`가 경고하지만([`05_safe_fn_judgment.rs`](examples/05_safe_fn_judgment.rs)), C와 Rust 사이의 차이는 여전히 보이지 않습니다.

**따르지 않아도 되는 경우.** 바꿀 수 없는 기존 헤더가 `int`, `long`, `size_t`를 쓴다면 Rust 선언도 `c_int`, `c_long`, `usize`로 **그 헤더에 맞춥니다**. 목표는 "C와 정확히 같게"이고, 크기가 정해진 타입은 새로 설계하는 인터페이스에서 그 목표를 쉽게 만들어 주는 수단입니다. 버퍼 길이처럼 C에서 관례적으로 `size_t`를 쓰는 자리는 `usize`로 받습니다(`projects/textstat_export`의 `buf_len`).

### 2.3 지침 2: 링크는 빌드 스크립트에 맡기고 `links`를 적는다

**문제 상황.** 선언만으로는 함수의 본문이 생기지 않습니다. 링커에게 어느 라이브러리를 어디서 찾을지 알려 줘야 합니다. 방법은 `extern` 블록에 붙이는 `#[link(name = "…")]` 속성과, 빌드 스크립트(`build.rs`)가 출력하는 `cargo::rustc-link-lib` 지시문 두 가지입니다.

**비권장 코드.** 미리 만든 `libtally.a`를 쓰려고 선언 블록에 `#[link(name = "tally", kind = "static")]`만 붙였습니다(`tally_ffi`의 사본에서 `build.rs`를 지우고 확인).

```text
error: could not find native static library `tally`, perhaps an -L flag is missing?
```

라이브러리가 표준 위치에 없으면 속성만으로는 찾을 수 없습니다. `RUSTFLAGS="-L native=<디렉터리>" cargo build`로 빌드하는 사람이 위치를 알려 주면 빌드되지만, 그 지식이 크레이트 밖에 있게 됩니다.

**권장 코드.** 빌드 스크립트가 C 소스를 컴파일하고 링크 지시문을 출력합니다. [`projects/tally_ffi/build.rs`](projects/tally_ffi/build.rs)의 핵심입니다.

```rust
println!("cargo::rerun-if-changed=csrc/tally.c");
println!("cargo::rerun-if-changed=csrc/tally.h");
cc::Build::new()
    .file(csrc.join("tally.c"))
    .include(&csrc)
    .warnings_into_errors(true)
    .compile("tally");
println!("cargo::metadata=include={}", csrc.display());
```

`cc` 크레이트는 이 플랫폼의 C 컴파일러를 불러 `OUT_DIR`에 `libtally.a`를 만들고 링크 지시문을 출력합니다. `cargo build -vv`로 본 출력의 일부입니다(경로는 줄임).

```text
[tally 0.1.0] cargo::rerun-if-changed=csrc/tally.c
[tally 0.1.0] cargo::rerun-if-changed=csrc/tally.h
[tally 0.1.0] cargo:rerun-if-env-changed=CC_FORCE_DISABLE
…(CC, CFLAGS, AR 등 환경 변수에 대한 rerun-if-env-changed 줄이 모두 35개)
[tally 0.1.0] cargo:rustc-link-lib=static=tally
[tally 0.1.0] cargo:rustc-link-search=native=…/target/debug/build/tally/<해시>/out
[tally 0.1.0] cargo::metadata=include=…/csrc
```

- 책은 콜론 하나인 `cargo:rustc-link-lib=…` 형태를 씁니다. Cargo Book에 따르면 콜론 둘인 `cargo::` 형태는 Cargo 1.77부터이고, 그보다 옛 Cargo를 지원하려면 `cargo:` 형태를 씁니다. `cc` 1.11.2는 지금도 `cargo:` 형태를 출력하고, 두 형태는 한 빌드 스크립트 안에 섞여도 됩니다.
- `cc`가 출력하는 `rerun-if-env-changed`가 하나라도 있으면 Cargo는 "패키지의 아무 파일이나 바뀌면 다시 실행"하는 기본 동작을 멈춥니다. C 소스를 고쳐도 다시 컴파일되지 않으므로 `rerun-if-changed`로 C 파일을 직접 알려 줍니다.
- `warnings_into_errors(true)`는 C 컴파일러의 경고(`cc`가 붙이는 `-Wall -Wextra`)를 빌드 실패로 바꿉니다. C 쪽 실수도 Rust의 경고만큼 진지하게 다루기 위해서입니다.

**`links` 키를 적는 이유.** `tally_ffi`의 `Cargo.toml`에는 `links = "tally"`가 있습니다. Cargo Book이 드는 이유는 셋입니다. 첫째, 같은 네이티브 라이브러리를 링크하는 패키지가 그래프에 둘 들어오면 같은 심볼이 두 벌 생기므로 Cargo가 해석 단계에서 거부합니다([ch25](../ch25-dependency-graph/README.md) 2.3절의 충돌이 이것입니다). 둘째, `cargo::metadata=include=…`가 이 패키지에 직접 의존하는 패키지의 빌드 스크립트에 `DEP_TALLY_INCLUDE` 환경 변수로 전해집니다. 셋째, 사용자가 `.cargo/config.toml`에서 빌드 스크립트를 미리 만든 라이브러리로 바꿔 끼울 수 있습니다. `links`를 적고 빌드 스크립트를 지우면 ``package specifies that it links to `tally` but does not have a custom build script``로 매니페스트를 읽지 못합니다.

**따르지 않아도 되는 경우.** 책은 빌드 스크립트 쪽이 환경을 살필 수 있어 더 유연하다고만 하고, 속성을 금하지는 않습니다. 이름과 위치가 모든 대상 플랫폼에서 같은 시스템 라이브러리라면 `#[link(name = "…")]` 한 줄이 가장 간단합니다(macOS의 `#[link(name = "CoreFoundation", kind = "framework")]` 등). C 표준 라이브러리처럼 표준 라이브러리가 이미 링크하는 것은 아무것도 적지 않아도 됩니다. 예제 05의 `hypot`과 `toupper`가 그렇습니다. C 표준 라이브러리의 선언은 손으로 쓰는 대신 `libc` 크레이트가 모아 두었다고 책의 각주가 소개합니다(이 챕터는 쓰지 않았습니다).

### 2.4 지침 3: FFI에 쓰는 타입에는 `#[repr(C)]`를, 뉴타입에는 `#[repr(transparent)]`를 붙인다

**문제 상황.** C와 구조체를 값으로 주고받거나 포인터 너머의 구조체를 함께 읽으려면, 양쪽이 필드를 같은 오프셋에 두어야 합니다.

**비권장 코드.** [`02_repr_c_layout.rs`](examples/02_repr_c_layout.rs)는 C 헤더의 `struct reading { uint8_t flags; int64_t value; uint16_t channel; }`을 `#[repr]` 없이 옮겼습니다. 기본 표현(`repr(Rust)`)에서는 컴파일러가 필드 순서를 바꿔 크기를 16바이트로 줄였고, `flags`가 10번 바이트에 있습니다. C는 0번 바이트를 `flags`로 읽습니다. 기본 표현의 배치는 언어가 약속하지 않으므로 컴파일러 버전에 따라 달라질 수도 있습니다.

**권장 코드.** `#[repr(C)]`는 "선언 순서대로, 각 필드의 메모리 정렬에 맞춰" C 컴파일러와 같은 규칙으로 배치합니다. 02에서는 24바이트에 `flags=0 value=8 channel=16`이었습니다. `tally_ffi`의 `tally_config`와 `tally_summary`는 C가 값으로 받고 돌려주는 구조체라서 `#[repr(C)]`입니다. rustc의 `non_camel_case_types` 린트는 `#[repr(C)]` 타입에는 경고하지 않으므로, C 이름(`tally_config`)을 그대로 써서 헤더와 대조하기 쉽게 했습니다.

**뉴타입은 `#[repr(transparent)]`.** [ch6](../ch6-newtype-pattern/README.md)의 단위 뉴타입을 경계에 쓰고 싶다면 `#[repr(transparent)]`를 붙입니다. Reference에 따르면 이 표현은 "크기가 0이 아닌 필드 하나와 같은 배치, 같은 호출 규약"을 약속하므로, C 헤더에는 그냥 `double`로 적으면 됩니다. 02의 `CelsiusPlain(f64)`와 `Celsius(f64)`는 이 환경에서 크기가 같고 결과도 같지만, `#[repr]` 없는 쪽에는 `improper_ctypes_definitions` 경고가 납니다(`#[expect]`를 지우고 확인).

```text
warning: `extern` fn uses type `CelsiusPlain`, which is not FFI-safe
  --> 02_repr_c_layout.rs:36:31
   |
36 | extern "C" fn warmer_plain(t: CelsiusPlain) -> CelsiusPlain {
   |                               ^^^^^^^^^^^^ not FFI-safe
   |
   = help: consider adding a `#[repr(C)]` or `#[repr(transparent)]` attribute to this struct
   = note: this struct has unspecified layout
```

`#[repr(C)]`를 붙인 뉴타입은 C의 "필드 하나인 구조체"가 되어 `double`과는 호출 규약이 다를 수 있습니다. 필드가 둘 이상이면 `#[repr(transparent)]`는 E0690으로 거부됩니다(4.2절).

**C의 enum은 정수로 받습니다.** C 함수가 돌려준 상태 값을 Rust 열거형으로 바로 받으면, C가 목록에 없는 값을 돌려주는 순간 올바르지 않은 값이 생겨 미정의 동작입니다. 02의 3번 구역은 `i32`로 받아 `TryFrom`으로 바꾸고, 모르는 값 `7`을 `Err(7)`로 돌려줍니다. C enum 타입 자체의 크기는 C 컴파일러가 정하므로, `tally.h`는 enum을 값의 이름으로만 쓰고 함수 시그니처는 `int32_t`로 적었습니다. Rust 쪽 `ffi.rs`도 `pub const TALLY_OK: i32 = 0;`처럼 정수 상수로 옮겼습니다.

**따르지 않아도 되는 경우.** 배치가 경계를 넘지 않는 타입에는 필요 없습니다. C에 불투명 포인터로만 보이는 Rust 타입(`projects/textstat_export`의 `Counter`)은 C가 필드를 읽지 않으므로 기본 표현으로 둡니다. rustc도 `extern "C" fn` 정의의 `*mut Counter`나 `Box<Counter>`에는 경고하지 않았습니다. 반대로 `extern` 블록의 **선언**에서는 `#[repr]` 없는 타입의 포인터(`*const Plain`)에도 경고가 났으므로, C의 불투명 타입을 들여올 때는 `tally_ffi`처럼 크기가 0인 `#[repr(C)]` 구조체로 이름만 만듭니다. 책은 C 구조체를 직접 다뤄도 안전할 때 래퍼가 `AsRef`/`AsMut`으로 그 구조체를 열어 줄 수 있다고 하는데, 이 챕터는 `#[repr(C)]` 타입을 `ffi` 모듈에 두고 바깥에는 `Config`, `Summary` 같은 Rust 타입으로 옮겨 보였습니다.

### 2.5 지침 4: 할당과 해제는 경계의 같은 쪽에서 한다

**문제 상황.** 원시 포인터는 라이프타임이 없는 주소일 뿐입니다. 참조를 원시 포인터로 바꾸는 순간 대여 검사기의 보호가 사라지고, C 코드는 해제된 메모리를 쓰거나, `const`를 벗겨 고치거나, 여러 스레드에서 동시에 쓰거나, Rust가 할당한 메모리를 C의 `free()`로 돌려줄 수 있습니다. 마지막 경우는 같은 프로세스 안에서도 두 쪽의 할당기가 같다는 보장이 없어 미정의 동작입니다.

**비권장 코드.** [`03_alloc_free_same_side.rs`](examples/03_alloc_free_same_side.rs)의 `v1::session_new`는 `Box`를 만들고 그 주소만 돌려줍니다. `Box`는 함수 끝에서 버려지므로, 출력의 해제 메시지가 C 쪽이 포인터를 받기도 전에 나옵니다. 돌려받은 포인터는 NULL이 아니지만 해제된 메모리를 가리킵니다. 반대 방향의 실수(C가 `malloc`한 문자열을 Rust의 `String`이나 `Box`로 바꿔 Rust의 할당기로 해제하는 것)도 같은 종류입니다.

**권장 코드.** 만드는 함수와 해제하는 함수를 같은 쪽이 짝지어 제공합니다.

| 할당 | 해제 | 어디서 |
| --- | --- | --- |
| C `tally_new` | C `tally_free` | `tally_ffi`. Rust는 `Drop`에서 `tally_free`를 부름 |
| C `tally_render` | C `tally_string_free` | `tally_ffi`. 표 문자열을 복사한 뒤 C의 해제 함수로 |
| Rust `Box::into_raw` | Rust `Box::from_raw` | 03의 `v2`, `textstat_export`의 `textstat_counter_new`/`_free` |
| C의 버퍼 | C | `textstat_describe`. Rust는 C가 준 버퍼에 쓰기만 함 |

`Box::into_raw`는 소유권을 원시 포인터로 옮겨 Rust의 자동 해제에서 빼냅니다. 03의 2번 구역에서 `session_free`가 `Box::from_raw`로 되돌려야 비로소 해제 메시지가 나옵니다. 3번 구역처럼 C가 해제 함수를 부르지 않으면 Rust는 대신 해제해 주지 않습니다(누수). C 쪽의 이중 해제도 막을 수 없으니 헤더에 "한 번만 해제"를 적습니다.

**문자열은 방향마다 정합니다.** 책은 소유한 문자열은 `CString`, 빌린 문자열은 `CStr`로 다루라고 권합니다. 두 타입의 API는 [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 2.4~2.5절에서 배웠으므로, 여기서는 방향별로 누가 해제하는지만 정리합니다.

| 방향 | 메모리의 주인 | Rust 쪽 모양 | 예 |
| --- | --- | --- | --- |
| Rust → C, 호출 동안 빌려줌 | Rust의 `CString` | `c_name.as_ptr()`. C는 복사만 함 | `Tally::set_name` |
| C → Rust, 빌려줌 | C, 다음 `tally_set_name`/`tally_free`까지 | `&self`에 묶은 `&CStr` | `Tally::name` |
| C → Rust, 넘겨줌 | C가 할당, C의 해제 함수로 | `Drop`에서 `tally_string_free` | `Tally::render` |
| Rust → C, 써 줌 | C의 버퍼 | 원시 포인터로 쓰기 | `textstat_describe` |

Rust가 할당한 문자열을 C에 넘기고 짝지은 해제 함수로 되찾는 방법(`CString::into_raw`/`from_raw`)도 있지만, 상세는 Part 10 `ch12-ffi-beneath-std`로 미룹니다. `textstat_describe`는 C의 버퍼가 초기화돼 있지 않을 수 있으므로 `&mut [u8]`을 만들지 않고 `ptr::copy_nonoverlapping`과 `write`로 원시 포인터에 직접 씁니다. 초기화되지 않은 메모리에서 `&mut [u8]`을 만드는 것 자체가 미정의 동작이기 때문입니다. C가 결과를 받으려고 넘기는 출력 매개변수(`textstat_counts *out` 등)도 같은 이유로 참조로 바꾸지 않고 원시 포인터로 씁니다(2.7절).

**따르지 않아도 되는 경우.** C 라이브러리가 "호출자가 `free()`로 해제하라"고 문서화한 경우(POSIX의 `strdup`, `getline` 등)에는 그 문서대로 C 표준 라이브러리의 `free`를 부릅니다. 이것도 "C가 할당한 것은 C의 해제 함수로"라는 같은 원칙을 지키는 것입니다. 정적 데이터처럼 아무도 해제하지 않는 메모리(예제 06의 `er34_stats_default`, 07의 `LABELS`)는 짝지을 해제 함수가 필요 없습니다.

### 2.6 지침 5: C 자원은 `Drop` 래퍼에 담고, unsafe는 그 모듈에 가둔다

**문제 상황.** 원시 포인터를 직접 들고 다니면 모든 사용처가 `unsafe` 블록이 되고, 해제를 빠뜨리는 경로가 생깁니다.

**비권장 코드.** [`04_drop_wrapper.rs`](examples/04_drop_wrapper.rs)의 `fill_manual`은 C 흉내 라이브러리의 계량기를 열고, 더하다가 용량을 넘으면 `return Err(…)`로 빠져나갑니다. 그 경로에서 `meter_close`를 건너뛰어 열린 핸들이 1로 남습니다. 성공 경로만 시험했다면 보이지 않는 누수입니다.

**권장 코드.** 핸들을 소유하는 구조체를 만들고, 만들 때 NULL을 `Result`로 바꾸고, 버려질 때 `Drop`에서 해제합니다. 04의 `Meter`는 같은 `?` 경로에서도 핸들을 남기지 않았습니다. 진짜 C 라이브러리를 감싼 판은 [`projects/tally_ffi/src/lib.rs`](projects/tally_ffi/src/lib.rs)입니다.

```rust
pub struct Tally {
    raw: NonNull<ffi::tally>,
}

impl Tally {
    pub fn new(config: Config) -> Result<Tally, TallyError> {
        let raw = ffi::tally_new(config.into()); // safe fn
        NonNull::new(raw).map(|raw| Tally { raw }).ok_or(TallyError::CreateFailed)
    }
}

impl Drop for Tally {
    fn drop(&mut self) {
        // SAFETY: self.raw는 tally_new가 돌려준 포인터이고, 해제하는 곳은 여기 한 곳뿐입니다.
        unsafe { ffi::tally_free(self.raw.as_ptr()) }
    }
}
```

선언은 비공개 `ffi` 모듈에, `unsafe` 블록은 `lib.rs`의 래퍼 메서드에만 있고, 이 크레이트를 쓰는 `main.rs`에는 `unsafe`가 한 줄도 없습니다. 책이 권하는 "unsafe한 FFI를 안전한 Rust 안에 가둔다"가 이 모양이고, 문제가 생기면 볼 곳이 래퍼 한 파일로 좁혀집니다([ch16](../ch16-avoid-unsafe/README.md) 2.5절). 이 래퍼에는 판단이 세 가지 더 들어 있습니다.

**`safe fn`으로 선언할 것인가.** `safe fn`은 "어떤 인자로 불러도 미정의 동작이 없다"는 보증을 선언 한 곳에서 하는 것이고, 컴파일러는 그 보증이 맞는지 확인하지 않습니다. 다음 조건을 모두 만족할 때만 씁니다.

1. 포인터를 받지 않거나, 받은 포인터가 무엇이든(NULL 포함) C 쪽이 안전하게 처리한다.
2. 모든 인자 값에 대해 C 쪽 동작이 정의돼 있다.
3. 전역 상태나 스레드 문제가 없다(`rand`, `strtok`, `setlocale`처럼 숨은 상태를 고치는 함수는 아님).
4. C 구현이 문서대로 동작한다고 믿을 수 있다.

[`05_safe_fn_judgment.rs`](examples/05_safe_fn_judgment.rs)의 `hypot`은 NaN과 무한대를 포함한 어떤 `f64` 쌍에도 결과가 정의돼 있어 `safe fn`입니다. `toupper`는 C 표준이 "EOF이거나 `unsigned char`로 나타낼 수 있는 값"만 허용하므로 `unsafe fn`으로 선언하고, `u8`을 받는 래퍼가 그 조건을 타입으로 보장합니다. 같은 `toupper`를 `safe fn`으로 선언한 `careless` 모듈은 컴파일되고, `toupper(1000)`도 `unsafe` 없이 쓸 수 있게 됩니다. `tally_ffi`에서는 인자가 없는 `tally_default_config`와, 설정을 C가 검사해 NULL을 돌려주는 `tally_new`만 `safe fn`이고 나머지는 포인터를 받으므로 `unsafe fn`입니다.

**C의 라이프타임 약속을 대여 규칙으로 옮깁니다.** `tally_name`이 돌려준 포인터는 "다음 `tally_set_name`이나 `tally_free`까지" 유효합니다. 래퍼의 `name(&self) -> &CStr`은 돌려주는 참조를 `&self`에 묶었고, 버퍼를 무효로 만드는 두 함수는 `&mut self`나 소유권이 있어야 부를 수 있습니다. 그래서 그 약속을 어기는 코드는 컴파일되지 않습니다. `name`의 문서 테스트(`compile_fail`)가 이것을 확인하고, 같은 코드를 예제로 컴파일하면 다음 에러입니다.

```text
error[E0502]: cannot borrow `t` as mutable because it is also borrowed as immutable
 --> examples/doccheck.rs:4:5
  |
3 |     let old = t.name();
  |               - immutable borrow occurs here
4 |     t.set_name("새 이름").unwrap(); // old가 아직 쓰이므로 &mut 대여를 만들 수 없습니다
  |     ^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("{old:?}");
  |                --- immutable borrow later used here
```

stable 툴체인의 문서 테스트는 `compile_fail,E0502`처럼 에러 코드를 적으면 그 블록을 일반 텍스트로 취급하므로(rustdoc 문서의 Unstable features), 에러 코드 없이 `compile_fail`만 적고 코드는 따로 확인했습니다.

**`Send`와 `Sync`는 헤더를 읽고 정합니다.** C 코드는 Rust의 `Send`/`Sync` 검사를 받지 않습니다. 원시 포인터 필드는 두 자동 트레이트를 모두 막으므로 래퍼의 기본값은 "어느 스레드로도 못 보냄"이고, 이것이 안전한 기본값입니다(4.1절). `tally.h`는 "전역 상태가 없고, tally 하나를 다른 스레드로 넘겨 써도 되지만, 같은 tally를 여러 스레드에서 동시에 부르면 안 된다(읽기 함수끼리도)"고 약속합니다. 앞부분은 `Send`, 뒷부분은 `Sync`가 아님에 해당하므로 `unsafe impl Send for Tally {}`만 `// SAFETY:` 주석과 함께 적었습니다. `main.rs`의 6번 구역이 tally를 스레드로 옮겨 쓰고 돌려받습니다. 헤더가 스레드에 대해 아무 말도 하지 않는다면 둘 다 구현하지 않습니다.

**따르지 않아도 되는 경우.** `-sys` 크레이트처럼 선언 자체를 공개하는 것이 목적인 크레이트는 래퍼를 두지 않고, 그 위의 별도 크레이트가 안전한 API를 제공합니다(Cargo Book의 `-sys` 관례, [`ch35-bindgen`](../ch35-bindgen/README.md)). 한 곳에서 한 번만 부르는 함수라면 [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md)의 `c_strlen`처럼 함수 하나짜리 래퍼로 충분합니다.

### 2.7 지침 6: C에 내보내는 함수는 접두사를 붙이고, 받은 포인터를 믿지 않는다

**문제 상황.** C 프로그램이 Rust 함수를 부르게 하려면 `pub extern "C" fn`에 `#[unsafe(no_mangle)]`을 붙여 맨 이름으로 내보냅니다. 그 이름은 프로그램 전체가 함께 쓰는 전역 이름 공간에 올라갑니다. 그리고 C는 무엇이든 넘길 수 있습니다. NULL도, 해제된 포인터도 됩니다.

**비권장 코드.** [`06_exported_null_checks.rs`](examples/06_exported_null_checks.rs)의 `v1::mean`은 접두사 없는 이름에, 받은 포인터를 `&*stats`로 바로 참조로 바꿉니다. NULL이 오면 미정의 동작이고(실행하지 않음), 표본이 0개면 `NaN`이 조용히 나갑니다. `v1::default_stats`는 지역 변수의 주소를 돌려줍니다. 이름이 겹치는 문제는 [`compile_fail/runtime_symbol_open.rs`](compile_fail/runtime_symbol_open.rs)가 보여 줍니다(4.3절).

**권장 코드.** [`projects/textstat_export/src/textstat.rs`](projects/textstat_export/src/textstat.rs)의 모양입니다.

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn textstat_count(text: *const c_char, out: *mut Counts) -> i32 {
    guard(|| {
        if text.is_null() || out.is_null() {
            return TEXTSTAT_ERR_NULL;
        }
        // … text를 CStr로 읽어 세고, 성공하면:
        // SAFETY: out이 NULL이 아님을 위에서 확인했고, Counts 하나를 쓸 수 있는 정렬된
        // 메모리라는 것이 전제 조건입니다. write는 원래 내용을 읽거나 버리지 않습니다.
        unsafe { out.write(counts) };
        TEXTSTAT_OK
    })
}
```

- **접두사.** 내보내는 이름은 모두 `textstat_`로 시작합니다. 헤더의 상수(`TEXTSTAT_OK`)와 타입(`textstat_counts`)도 같은 접두사를 씁니다. 1.98부터 rustc는 표준 라이브러리가 쓰는 일부 심볼(`open`, `close`, `read`, `write`, `free`, `memcpy` 등)을 내보내면 알려 주지만, `init`, `new`, `destroy` 같은 흔한 이름은 아무 경고 없이 다른 라이브러리와 겹칩니다.
- **읽을 포인터는 `as_ref()`로, 출력 매개변수는 `is_null()`과 `write`로.** 원시 포인터의 `as_ref()`/`as_mut()`은 NULL을 `None`으로 바꿔 줍니다. `&*p`는 NULL에서도 참조를 만들어 버립니다. 다만 `as_ref()`도 NULL이 아닐 때는 참조를 만들고, `core::ptr` 문서의 "Pointer to reference conversion" 절은 참조로 바꿀 포인터가 `T`의 올바른 값을 가리켜야 한다고 적습니다. C 호출자는 결과를 받을 변수를 초기화하지 않고 주소만 넘기는 일이 흔하므로, 쓰기만 하는 출력 매개변수(`textstat_count`의 `out`)는 참조로 바꾸지 않고 `is_null()`로 확인한 뒤 `out.write(값)`으로 씁니다. 2.5절의 `textstat_describe`가 버퍼에 원시 포인터로 쓰는 것과 같은 이유입니다. 읽을 값이 있는 입력 포인터(06의 `stats`)와 핸들(`counter`)에만 `as_ref()`/`as_mut()`을 쓰고, `# Safety` 절에 "초기화된 …를 가리켜야" 또는 "초기화돼 있지 않아도 된다"를 구분해 적습니다. 06의 `v2::er34_stats_mean`은 NULL과 빈 입력을 각각 `-1`, `-2`로 돌려줍니다. NULL이 아닌 잘못된 포인터는 여전히 걸러낼 수 없으므로 그 조건도 `# Safety` 절에 적습니다.
- **`unsafe extern "C" fn`.** 포인터 인자를 역참조하는 `pub` 함수가 `unsafe`가 아니면 clippy의 `not_unsafe_ptr_arg_deref`(기본 deny)가 에러를 냅니다(직접 확인). Rust 안에서 부르는 사람에게도 "조건이 있다"는 것을 드러내고, C 쪽에서는 아무것도 달라지지 않습니다.
- **스택 대신 힙이나 정적 데이터.** 1.91부터 rustc의 `dangling_pointers_from_locals`가 06의 `&raw const local`을 돌려주는 모양을 경고했습니다. 다만 지역 배열에 `as_ptr()`를 부른 모양은 경고하지 않았으므로 린트에 기대지 않습니다. 힙에 두고 넘길 때 `Box`를 그대로 두면 2.5절의 `v1`이 되므로 `Box::into_raw`를 씁니다.

[`projects/textstat_export`](projects/textstat_export/src/main.rs)에서 C 코드(`csrc/driver.c`)가 이 함수들을 이름으로 부릅니다. 실행 파일과 C 정적 라이브러리의 심볼은 다음과 같았습니다(주소 열은 지움).

```text
$ nm -g target/debug/textstat-export | grep -E 'textstat_|c_program'
T _c_program_main
T _textstat_count
T _textstat_counter_feed
…(_textstat_counter_finish, _free, _new, _describe, _version)
$ nm -u target/debug/build/textstat-export/<해시>/out/libtextstat_driver.a | grep textstat
_textstat_count
…(같은 일곱 이름)
```

`-g`는 전역 심볼만 보여 줍니다. 빼면 `Counter::feed` 같은 Rust 함수의 맹글링된 지역 심볼(`t __RNvM…15textstat_export8textstat…`)도 이름에 `textstat_`가 들어 있어 함께 나옵니다.

사본에서 `textstat_version`의 `#[unsafe(no_mangle)]`을 지우자 rustc는 ``function `textstat_version` is never used`` 경고를 냈고(맹글링된 이름으로는 아무도 찾지 않으므로), 링크가 실패했습니다.

```text
Undefined symbols for architecture arm64:
  "_textstat_version", referenced from:
      _c_program_main in libtextstat_driver.a[2](…-driver.o)
ld: symbol(s) not found for architecture arm64
```

**따르지 않아도 되는 경우.** C에 함수 포인터로만 넘기는 콜백은 링커가 이름으로 찾지 않으므로 `no_mangle`과 접두사가 필요 없습니다(콜백의 세부는 Part 10 `ch12-ffi-beneath-std`). 같은 실행 파일 안에서 Rust가 부르는 `extern "C" fn`도 마찬가지라서, 예제 03·06·07은 속성 없이 두었습니다.

### 2.8 지침 7: 패닉이 FFI 경계를 넘지 못하게 막는다

**문제 상황.** Rust의 패닉은 스택을 되감으며(unwind) 올라갑니다. C 프레임은 이 되감기를 모릅니다. 책은 패닉이 FFI 경계를 넘으면 "항상 미정의 동작"이라고 적었습니다.

**지금의 동작.** 1.81부터는 `extern "C"` 함수 밖으로 나가려는 패닉이 그 자리에서 프로세스를 중단시킵니다(릴리스 노트의 "Abort on uncaught panics in `extern "C"` functions"). [ch18](../ch18-dont-panic/README.md)의 [`projects/panic_strategy`](../ch18-dont-panic/projects/panic_strategy/src/main.rs)가 `catch_unwind` 안에서 불러도 `panic in a function that cannot unwind`와 함께 중단되는 것을 보였습니다. 미정의 동작은 아니게 되었지만, C 프로그램 입장에서는 라이브러리 호출 하나가 프로세스 전체를 끝내는 것입니다.

**비권장 코드.** [`07_catch_unwind_boundary.rs`](examples/07_catch_unwind_boundary.rs)의 `v1::level_label`은 본문을 감싸지 않았습니다. 안쪽 `label_of`에는 0..=3만 들어온다고 믿은 버그가 있어 `9`를 넣으면 인덱스 패닉이 납니다. 이 예제는 그 입력을 `v1`에 넣지 않습니다.

**권장 코드.** 내보내는 함수의 본문 전체를 `catch_unwind`로 감싸 패닉을 에러 코드로 바꿉니다.

```rust
fn ffi_guard(body: impl FnOnce() -> i32 + UnwindSafe) -> i32 {
    panic::catch_unwind(body).unwrap_or(ER34_ERR_PANIC)
}
```

07의 `v2::er34_level_label(9)`는 `-99`를 돌려주고 C 쪽 출력 변수를 건드리지 않았습니다. `textstat_export`에서도 C가 `finish` 뒤에 `feed`를 부르자 안쪽 `Counter::feed`의 `assert!`가 패닉했고, C는 `TEXTSTAT_ERR_PANIC`을 받은 뒤 다음 줄을 계속 실행했습니다. `catch_unwind` 문서도 "패닉하면 중단해도 된다면 `extern "C"`가 이미 그렇게 해 주므로 `catch_unwind`가 필요 없고, 더 부드럽게 처리할 때 쓰라"고 설명합니다. 주의할 점이 둘 있습니다.

- **원시 포인터는 `UnwindSafe` 검사를 통과합니다.** 클로저가 `&mut Counter`를 캡처하면 컴파일러가 `AssertUnwindSafe`를 요구하지만([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 4.3절), `*mut Counter`를 캡처하고 안에서 참조로 바꾸면 요구하지 않습니다. 패닉 뒤의 상태가 어긋나지 않는지는 직접 따져야 합니다. `Counter::feed`는 상태를 바꾸기 전에 검사하므로 붙잡힌 뒤에도 핸들이 멀쩡합니다([ch18](../ch18-dont-panic/README.md) 2.5절).
- **패닉 메시지는 경계를 넘지 못합니다.** 기본 패닉 훅이 표준 에러에 메시지를 출력할 뿐, C가 받는 것은 `-99`뿐입니다. 메시지가 필요하면 스레드 지역 변수에 담아 두고 C가 버퍼로 가져가게 하는 함수를 따로 둡니다(연습 2).

**`extern "C-unwind"`.** 1.71에서 안정화된 이 ABI는 경계를 넘는 되감기를 허용합니다. 07의 3번 구역에서 `extern "C-unwind" fn`의 패닉은 부르는 쪽(Rust)의 `catch_unwind`까지 되감겨 `Err`로 붙잡혔습니다. Reference의 표에 따르면 `panic = "unwind"`일 때 되감기가 `"C"` 같은 되감지 않는 ABI의 경계에 닿으면 중단이고, C++ 예외가 `"C"`로 선언한 함수를 통해 Rust로 들어오면 미정의 동작입니다. 예외 처리를 켜고 컴파일한 C++처럼 부르는 쪽이 되감기를 처리할 수 있을 때만 `"C-unwind"`를 씁니다. 상세는 Part 9 `ch9-memory-panic-linkage`에서 다룹니다.

**따르지 않아도 되는 경우.** 패닉하면 프로세스를 끝내는 것이 맞는 프로그램이라면 `extern "C"`의 중단을 그대로 받아들여도 미정의 동작은 없습니다. `panic = "abort"` 프로필로 빌드하는 프로그램에서는 `catch_unwind`가 아무것도 붙잡지 못하므로 패닉하지 않는 코드(`Result`)가 유일한 방법입니다(ch18 2.5절).

### 2.9 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0, cargo 1.100.0에서 확인한 차이는 다음과 같습니다(버전은 로컬 문서의 릴리스 노트로 확인).

- **`unsafe extern` 블록과 unsafe 속성(1.82).** 책의 `extern "C" { … }`와 `#[no_mangle]`은 2024 에디션에서 에러이고, `unsafe extern "C" { … }`와 `#[unsafe(no_mangle)]`로 씁니다. 블록 안의 항목에 `safe fn`을 붙일 수 있게 되어, 2.6절의 "어디에 unsafe를 둘 것인가"라는 판단이 생겼습니다.
- **C 타입 별칭의 위치(1.64).** 책의 `std::os::raw::c_int` 대신 `core::ffi::c_int`(`std::ffi`에도 있음)를 씁니다. `no_std`에서도 쓸 수 있습니다.
- **`extern "C"` 밖으로 나가는 패닉(1.81).** 책의 "항상 미정의 동작"은 이제 "프로세스 중단"입니다. 책의 각주가 언급한 `"C-unwind"`(1.71)는 07에서 직접 확인했습니다.
- **빌드 스크립트 지시문(Cargo 1.77).** 책의 `cargo:rustc-link-lib` 형태는 지금도 동작하지만 Cargo Book은 `cargo::` 형태를 기본으로 설명합니다.
- **새 린트.** `dangling_pointers_from_locals`(1.91), `invalid_runtime_symbol_definitions`(deny)와 `suspicious_runtime_symbol_definitions`(1.98)는 책 이후에 생겼습니다. 포인터를 받아 역참조하는 안전한 `pub extern "C" fn`은 지금 clippy `not_unsafe_ptr_arg_deref`(기본 deny)에 걸리므로, 이 챕터의 내보내는 함수는 모두 `unsafe extern "C" fn`입니다.
- **`extern "C"`와 `no_mangle`.** 책의 "`extern "C"`가 자동으로 `#[no_mangle]`로 만든다"는 문장은 `extern` 블록의 선언에만 맞습니다. Rust에서 정의한 `extern "C" fn`은 속성이 없으면 맹글링됩니다(2.1절의 `nm`). 책도 Rust 함수를 내보낼 때는 `#[no_mangle]`을 붙이므로 내용이 다르지는 않지만, 문장만 읽으면 오해하기 쉽습니다.
- **책에 없는 지침.** 다음은 책의 Item 34가 명시적으로 권하지 않는 내용입니다(2026-11-20에 온라인판에서 확인). 책은 C 함수 선언과 링크(`#[link]`, `cargo:rustc-link-lib`, `rustc-link-search`), 크기가 정해진 타입, 이름 맹글링과 `nm`·`c++filt`, `#[repr(C)]`, `CString`/`CStr`과 `const`, 할당과 해제를 같은 쪽에서, `Drop` 래퍼와 `Result`·`AsRef`/`AsMut`, unsafe를 안전한 코드에 가두기, `#[no_mangle]`과 접두사, NULL 검사와 `Box::into_raw`/`from_raw`, 패닉 막기를 다룹니다. `cc` 크레이트와 `catch_unwind`는 이름으로 들지 않습니다.
  - 2.2절의 `improper_ctypes`·`clashing_extern_declarations`: rustc의 실제 경고(예제 01, 05)가 근거입니다.
  - 2.3절의 `cc` 크레이트, `rerun-if-changed`, `warnings_into_errors`, `links`의 세 가지 쓰임: Cargo Book의 Build Scripts 장(Outputs of the Build Script, The `links` Manifest Key)과 `cc` 1.11.2 문서, `cargo build -vv` 출력이 근거입니다(책은 `links`를 각주로만 언급).
  - 2.4절의 `#[repr(transparent)]`와 "C의 enum은 정수로 받는다": Reference의 Type layout 장과 Behavior considered undefined(올바르지 않은 값 만들기), 예제 02가 근거입니다.
  - 2.5절의 "C가 준 버퍼에 원시 포인터로 쓰기": `slice::from_raw_parts_mut`와 `ptr::copy_nonoverlapping` 문서가 근거입니다.
  - 2.6절의 `safe fn` 판단 기준, 라이프타임을 `&self`에 묶기, `Send`/`Sync` 판단: Edition Guide의 Unsafe extern blocks 절, C 표준의 `toupper` 조건, ch17이 넘긴 약속이 근거입니다.
  - 2.7절의 "읽을 포인터는 `as_ref()`, 출력 매개변수는 `is_null()` + `write`" 구분: 책은 `&*p` 대신 `as_ref()`를 쓰라고까지만 권합니다. `core::ptr` 문서의 Pointer to reference conversion 절(참조로 바꿀 포인터는 올바른 값을 가리켜야 함)과 `<*mut T>::write` 문서가 근거입니다.
  - 2.8절의 `catch_unwind`로 붙잡아 에러 코드로 바꾸기와 `"C-unwind"`의 쓰임: `std::panic::catch_unwind` 문서와 Reference의 Functions(Unwinding)·Panic 장이 근거입니다.

### 2.10 기억할 것

- 링커는 이름만 맞춥니다. 선언은 헤더와 한 글자씩 대조하고, 새 인터페이스에는 `<stdint.h>`의 크기가 정해진 타입을 씁니다. rustc는 C에 없는 타입만 잡아 줍니다.
- 링크는 빌드 스크립트(`cc`)에 맡기고 C 파일을 `rerun-if-changed`로 알립니다. 네이티브 라이브러리를 링크하는 패키지에는 `links`를 적습니다.
- 경계를 넘는 구조체는 `#[repr(C)]`, 뉴타입은 `#[repr(transparent)]`, C의 enum은 정수로 받습니다.
- 할당한 쪽이 해제합니다. C 자원은 `Drop` 래퍼에 담고, NULL과 상태 코드는 `Result`로, C의 라이프타임과 스레드 약속은 `&self`와 `Send`/`Sync`로 옮깁니다.
- 내보내는 함수에는 접두사와 `#[unsafe(no_mangle)]`을 붙이고, 읽을 포인터는 `as_ref()`로, 출력 매개변수는 `is_null()` 뒤 `write`로 다루고, 본문은 `catch_unwind`로 감쌉니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행하고, 모두 C 코드 없이 동작하며 종료 코드 0으로 끝납니다. 예제의 `#[expect]`는 지우면 실제로 경고가 나는 것만 붙였습니다. `projects/`의 두 프로젝트는 C 컴파일러가 필요하고, 각자의 디렉터리에서 실행합니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_sized_types.rs`](examples/01_sized_types.rs) | 1 | `c_long`, C에 없는 타입의 선언 | `int64_t`/`i64`, 대신 쓸 타입 | 2.2 |
| [`02_repr_c_layout.rs`](examples/02_repr_c_layout.rs) | 3 | 기본 표현의 구조체와 뉴타입 | `#[repr(C)]`, `#[repr(transparent)]`, enum은 `TryFrom` | 2.4 |
| [`03_alloc_free_same_side.rs`](examples/03_alloc_free_same_side.rs) | 4 | `Box`를 버리고 주소만 돌려줌 | `Box::into_raw`/`from_raw` 짝 | 2.5 |
| [`04_drop_wrapper.rs`](examples/04_drop_wrapper.rs) | 5 | 직접 열고 닫기(실패 경로에서 누수) | `Drop` 래퍼와 `Result` | 2.6 |
| [`05_safe_fn_judgment.rs`](examples/05_safe_fn_judgment.rs) | 5 | 조건이 있는 함수를 `safe fn`으로 | `safe fn hypot`, `unsafe fn toupper` + 래퍼 | 2.6 |
| [`06_exported_null_checks.rs`](examples/06_exported_null_checks.rs) | 6 | `&*p`, 지역 변수의 주소 | 접두사, `as_ref()`와 `write`, 정적 데이터 | 2.7 |
| [`07_catch_unwind_boundary.rs`](examples/07_catch_unwind_boundary.rs) | 7 | 감싸지 않은 `extern "C"` | `catch_unwind`, `extern "C-unwind"` | 2.8 |
| [`projects/tally_ffi`](projects/tally_ffi/src/lib.rs) | 1~5 | — | C 라이브러리 + 선언 모듈 + 안전한 래퍼 | 2.2~2.6 |
| [`projects/textstat_export`](projects/textstat_export/src/textstat.rs) | 4, 6, 7 | — | C가 부르는 Rust 함수 | 2.5, 2.7, 2.8 |

### 01_sized_types — C 타입 별칭과 경계에 둘 수 없는 타입

```sh
cargo run -p er-ch34-ffi-boundaries --example 01_sized_types
```

1번 구역은 플랫폼마다 다릅니다. 아래는 macOS aarch64의 결과입니다.

```text
1. C 타입 별칭이 이 환경에서 가리키는 Rust 타입
   c_char     → i8   1바이트
   c_short    → i16  2바이트
   c_int      → i32  4바이트
   c_long     → i64  8바이트
   c_longlong → i64  8바이트
   size_t     → usize 8바이트
2. 5 GiB 파일 크기를 경계 너머로 돌려받는다면
   long이 32비트인 플랫폼: Err(TryFromIntError(PosOverflow))
   int64_t / i64: 5368709120
3. C에 없는 타입 대신 쓰는 것 (rejected 모듈의 선언은 컴파일러가 경고함)
   &str       → *const c_char(0 바이트로 끝남) 또는 *const u8 + 길이
   String     → *mut c_char + 짝지은 해제 함수, 또는 호출자 버퍼
   (i32, i32) → #[repr(C)] 구조체
   Point      → #[repr(C)] Point
   char       → u32
   &[u8]      → *const u8 + 길이
```

### 02_repr_c_layout — 배치, 뉴타입, C의 enum

```sh
cargo run -p er-ch34-ffi-boundaries --example 02_repr_c_layout
```

```text
1. 같은 필드, 다른 표현
   기본 표현:  크기 16, 정렬 8, 오프셋 flags=10 value=0 channel=8
   #[repr(C)]: 크기 24, 정렬 8, 오프셋 flags=0 value=8 channel=16
2. 뉴타입: 크기는 같아도 약속이 다름
   CelsiusPlain 8바이트, Celsius 8바이트, f64 8바이트
   warmer_plain(20.0) = 21.5, warmer(20.0) = 21.5
3. C의 enum 값을 Rust 열거형으로 바꾸기
   1 → Ok(Running)
   2 → Ok(Fault)
   7 → Err(7)
```

기본 표현의 오프셋은 이 툴체인의 결과이고 약속된 값이 아닙니다.

### 03_alloc_free_same_side — 할당한 쪽이 해제한다

```sh
cargo run -p er-ch34-ffi-boundaries --example 03_alloc_free_same_side
```

```text
1. 비권장: Box를 버리고 주소만 돌려줌
      (Session 'v1-7' 해제, 조회 0번)
   돌려받은 포인터가 NULL인가: false
   살아 있는 Session: 0
2. 권장: Box::into_raw로 넘기고, 같은 라이브러리의 해제 함수로 되찾음
   만든 직후 살아 있는 Session: 1
   조회 2번 뒤 살아 있는 Session: 1
      (Session 'v2-8' 해제, 조회 2번)
   session_free 뒤 살아 있는 Session: 0
3. 해제 함수를 잊으면 Rust는 대신 해제해 주지 않음
   해제 함수를 부르지 않으면 살아 있는 Session: 1
      (Session 'v2-9' 해제, 조회 0번)
```

### 04_drop_wrapper — 수동 해제와 `Drop` 래퍼

```sh
cargo run -p er-ch34-ffi-boundaries --example 04_drop_wrapper
```

```text
1. 비권장: 직접 열고 닫기
   용량 200: Ok(120)
   열린 핸들: 0
   용량 100: Err(Full { amount: 40 })
   열린 핸들: 1
2. 권장: Drop 래퍼
   용량 200: Ok(120)
   용량 100: Err(Full { amount: 40 })
   Display: 40을(를) 더하면 용량을 넘습니다
   새로 남은 핸들: 0
3. NULL을 Result로
   Meter::open(0): Some(OpenFailed)
```

### 05_safe_fn_judgment — `safe fn`으로 선언할 수 있는 C 함수

```sh
cargo run -p er-ch34-ffi-boundaries --example 05_safe_fn_judgment
```

```text
1. safe fn: 어떤 인자에도 정의된 결과
   hypot(3, 4) = 5
   hypot(inf, NaN) = inf
   hypot(NaN, 1) = NaN
2. unsafe fn + 타입으로 조건을 보장하는 래퍼
   "ffi-guard �"
   "FFI-GUARD �"
3. 비권장: 조건이 있는 함수를 safe fn으로 선언
   careless::toupper('q') = Some('Q')
```

`toupper`의 결과는 로캘에 따라 다릅니다. Rust 프로그램은 `setlocale`을 부르지 않으므로 "C" 로캘이고, `0xE9`는 글자로 취급되지 않아 그대로 돌아옵니다.

### 06_exported_null_checks — 내보내는 함수와 받은 포인터

```sh
cargo run -p er-ch34-ffi-boundaries --example 06_exported_null_checks
```

```text
1. 비권장: 받은 포인터를 그대로 믿음
   mean(&stats) = 62.5
   mean(&empty) = NaN
2. 권장: NULL과 빈 입력을 상태 코드로
   er34_stats_mean(&stats, &out) = 0
   er34_stats_mean(NULL, &out) = -1
   er34_stats_mean(&stats, NULL) = -1
   er34_stats_mean(&empty, &out) = -2
   out = 62.5
   (ER34_OK = 0, ER34_ERR_NULL = -1, ER34_ERR_EMPTY = -2)
3. 돌려주는 포인터가 가리키는 곳
   v1::default_stats()가 NULL인가: false
   v2::er34_stats_default().samples = 0
```

### 07_catch_unwind_boundary — 패닉을 경계 안에서 붙잡기

```sh
cargo run -p er-ch34-ffi-boundaries --example 07_catch_unwind_boundary
```

표준 출력입니다.

```text
1. 비권장: 감싸지 않은 extern "C" 함수 (정상 입력만)
   level_label(2) = 0, "mid"
2. 권장: catch_unwind로 감싼 함수
   er34_level_label(2) = 0, "mid"
   er34_level_label(9) = -99, "(없음)"
   er34_level_label(1, NULL) = -1
3. extern "C-unwind": 패닉이 경계를 넘어 부르는 쪽으로 되감김
   level_label_unwind(3): Ok("high")
   level_label_unwind(9): 부르는 쪽에서 붙잡음 = true
```

표준 에러에는 기본 패닉 훅의 메시지가 두 번 나옵니다. 스레드 번호는 실행마다 다르고, `note:` 줄은 처음 한 번만 나옵니다.

```text
thread 'main' (스레드 번호) panicked at appd-a/ch34-ffi-boundaries/examples/07_catch_unwind_boundary.rs:22:5:
index out of bounds: the len is 4 but the index is 9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### projects/tally_ffi — C 라이브러리를 감싼 안전한 래퍼

정수 표본을 구간별로 세는 작은 C 히스토그램 라이브러리 `tally`(`csrc/tally.h`, `csrc/tally.c`)를 `build.rs`가 `cc`로 컴파일해 링크합니다. `src/ffi.rs`가 손으로 옮긴 선언을, `src/lib.rs`가 안전한 래퍼를, `src/main.rs`가 그 래퍼만 쓰는 프로그램을 담습니다.

```sh
cd appd-a/ch34-ffi-boundaries/projects/tally_ffi
cargo run -q
```

```text
1. C가 값으로 돌려준 기본 설정
   Config { low: 0, bucket_width: 10, bucket_count: 10 }
2. 응답 시간 12개를 기록
   Summary { count: 12, below: 1, above: 2, range: Some((-3, 250)) }
3. 이름: 소유한 문자열을 넘기고(CString), 빌린 문자열을 받음(&CStr)
   바꾸기 전: "tally"
   바꾼 뒤:   Ok("응답 시간(ms)")
4. C가 할당한 표 → String으로 복사 → tally_string_free로 해제
[응답 시간(ms)] n=12 below=1 above=2
  [    0,    50) #### 4
  [   50,   100) ### 3
  [  100,   150) # 1
  [  150,   200) # 1
5. NULL과 상태 코드를 Result로
   너비 0인 설정: Some(CreateFailed)
   긴 이름: Err(NameTooLong { len: 47, max: 31 })
   중간의 0 바이트: Err(InteriorNul { position: 6 })
   Display: 이름의 6번 위치에 0 바이트가 있습니다
   이름은 그대로: Ok("응답 시간(ms)")
6. Send: 다른 스레드로 옮겨 쓰고 돌려받기
   스레드에서 하나 더 기록한 뒤 count = 13
```

`cargo test`는 단위 테스트 8개(잘못된 설정 네 가지, 이름 오류, 표 내용, 스레드 이동 등)와 `name`의 `compile_fail` 문서 테스트 1개를 통과합니다. `[    0,    50)` 같은 표는 C의 `printf` 형식으로 만든 것이고, 이름 안의 한글은 Rust가 넘긴 UTF-8 바이트를 C가 그대로 복사했다가 돌려준 것입니다.

### projects/textstat_export — C 프로그램이 부르는 Rust 함수

`src/textstat.rs`가 UTF-8 텍스트의 바이트·글자·단어·줄 수를 세는 함수를 `textstat_` 접두사로 내보내고, `csrc/textstat.h`가 그 C 선언을 담습니다. 실제 C 프로그램이라면 Rust 쪽을 정적 라이브러리로 빌드해 C의 `main`과 링크하지만, cargo만으로 확인하려고 `build.rs`가 C 코드(`csrc/driver.c`)를 이 실행 파일에 함께 링크하고 Rust의 `main`은 C 쪽 진입점 `c_program_main` 하나만 부릅니다. 그 뒤로 출력하는 것은 모두 C 코드입니다.

```sh
cd appd-a/ch34-ffi-boundaries/projects/textstat_export
cargo run -q
```

```text
[C] textstat_version() = 1
[C] 1. 텍스트 전체를 한 번에 세기
    "hello, 러스트\n둘째 줄" -> bytes=27 chars=15 words=4 lines=2
[C] 2. 잘못된 인자
    text = NULL -> 에러 -1
    out = NULL -> 에러 -1
    잘못된 UTF-8 -> 에러 -2
[C] 3. Rust가 만든 핸들에 조각으로 나눠 넣기
    feed 결과: 0 0 0
    finish -> bytes=13 chars=9 words=2 lines=1
[C] 4. 사용 규칙 위반: finish 뒤에 feed
    feed 결과: -99 (TEXTSTAT_ERR_PANIC = -99)
[C] 5. 문자열 받기: C의 버퍼에 Rust가 씀
    필요한 길이 35, 버퍼 64바이트: "13바이트, 9글자, 2단어, 1줄"
    필요한 길이 35, 버퍼 8바이트: ""
```

3번 구역에서 "녕"의 UTF-8 세 바이트가 두 조각에 걸쳐 들어왔지만 `Counter`가 잘린 바이트를 다음 조각까지 들고 있다가 이어 셉니다. 표준 에러에는 4번 구역의 패닉 메시지(`thread 'main' (스레드 번호) panicked at src/textstat.rs:63:9:`와 `finish 뒤에는 feed할 수 없습니다`)가 나옵니다. C의 표준 출력 버퍼는 Rust와 따로 있어서, 터미널에서 두 출력이 섞이는 위치는 버퍼링에 따라 다를 수 있습니다. `cargo test`는 내보낸 함수를 C처럼 원시 포인터로 부르는 단위 테스트 4개를 통과합니다.

## 4. 자주 하는 실수

`compile_fail/`의 세 파일은 각 파일 머리의 명령으로 확인합니다. Part 2 ch4가 이미 보인 `unsafe` 없는 `extern` 블록, `unsafe(…)` 없는 `#[no_mangle]`, `unsafe` 블록 없이 부른 `extern` 함수(E0133)는 4.4절의 표로만 정리합니다.

### 4.1 원시 포인터를 담은 래퍼를 다른 스레드로 — E0277

[`compile_fail/e0277_raw_handle_not_send.rs`](compile_fail/e0277_raw_handle_not_send.rs)는 `NonNull<tally>` 필드 하나를 가진 `Tally`를 `thread::spawn`에 옮깁니다.

```sh
rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0277_raw_handle_not_send.rs
```

```text
error[E0277]: `NonNull<tally>` cannot be sent between threads safely
  --> compile_fail/e0277_raw_handle_not_send.rs:42:32
   |
42 |     let worker = thread::spawn(move || t.is_open());
   |                  ------------- -------^^^^^^^^^^^^
   |                  |             |
   |                  |             `NonNull<tally>` cannot be sent between threads safely
…
note: required because it appears within the type `Tally`
  --> compile_fail/e0277_raw_handle_not_send.rs:24:5
   |
23 | pub struct Tally {
24 |     raw: NonNull<tally>,
   |     ^^^^^^^^^^^^^^^^^^^ required by this field
```

**원인.** 원시 포인터(`*mut T`, `*const T`, `NonNull<T>`)는 `Send`도 `Sync`도 아니고, 필드 하나가 막으면 구조체 전체가 막힙니다. 컴파일러는 포인터 너머의 C 코드가 스레드에 대해 무엇을 약속하는지 모르므로 막는 쪽을 고릅니다.

**해결.** 에러를 없애려고 `unsafe impl Send`부터 적지 않습니다. C 라이브러리의 문서를 읽고, 핸들을 다른 스레드로 넘겨 써도 된다고 약속할 때만 `// SAFETY:` 주석과 함께 `unsafe impl Send`를 적습니다. 동시 접근까지 허용할 때만 `Sync`도 적습니다(2.6절, `projects/tally_ffi/src/lib.rs`).

### 4.2 필드가 둘인 타입에 `#[repr(transparent)]` — E0690

[`compile_fail/e0690_transparent_two_fields.rs`](compile_fail/e0690_transparent_two_fields.rs)는 범위를 나타내는 `Range(f64, f64)`에 `#[repr(transparent)]`를 붙였습니다.

```text
error[E0690]: transparent struct needs at most one non-trivial field, but has 2
  --> compile_fail/e0690_transparent_two_fields.rs:14:1
   |
14 | pub struct Range(pub f64, pub f64);
   | ^^^^^^^^^^^^^^^^ -------  ------- this field has non-zero size
   | |                |
   | |                this field has non-zero size
   | needs at most one non-trivial field, but has 2
```

**원인.** `#[repr(transparent)]`는 "안쪽 필드 하나와 같은 배치와 호출 규약"이라는 약속이라, 크기가 0이 아닌 필드가 둘이면 어느 쪽과 같아야 할지 정할 수 없습니다. 크기가 0이고 메모리 정렬이 1인 필드(`PhantomData` 등)는 몇 개든 더할 수 있습니다.

**해결.** 필드가 여럿인 타입은 `#[repr(C)]`를 붙여 C 쪽에서도 구조체로 받습니다. 그렇게 고친 사본은 컴파일되어 `2.5`를 출력했습니다.

### 4.3 접두사 없이 `open`/`close`를 내보냄 — 린트 에러

[`compile_fail/runtime_symbol_open.rs`](compile_fail/runtime_symbol_open.rs)는 로그 라이브러리의 핸들을 여닫는 함수를 `#[unsafe(no_mangle)]`과 함께 `open`, `close`로 내보냅니다.

```text
error: invalid definition of the runtime `open` symbol used by the standard library
  --> compile_fail/runtime_symbol_open.rs:26:1
   |
26 | pub unsafe extern "C" fn open(path: *const c_char) -> *mut Logger {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: expected `unsafe extern "C" fn(*const i8, i32, ...) -> i32` (for the current target)
           found    `unsafe extern "C" fn(*const i8) -> *mut Logger`
   = help: either fix the signature or remove any attributes like `#[unsafe(no_mangle)]`, `#[unsafe(export_name = "open")]`, or `#[link_name = "open"]`
   = note: `#[deny(invalid_runtime_symbol_definitions)]` on by default
```

`close`에도 같은 에러가 하나 더 납니다.

**원인.** `no_mangle`은 이름을 프로그램 전체가 함께 쓰는 이름 공간에 그대로 올리므로, C 라이브러리의 `open(2)`·`close(2)`와 같은 심볼이 됩니다. 표준 라이브러리도 파일을 열 때 이 심볼을 부릅니다. 1.98의 이 린트는 인자 수나 반환값 유무가 다르면 에러이고, 시그니처가 맞아 보여도(`close(fd: i32) -> i32`) `suspicious_runtime_symbol_definitions` 경고를 냅니다. 목록은 표준 라이브러리가 쓰는 일부 심볼뿐이라, 실험해 보니 `init`, `new`, `destroy`, `printf`, `rand` 같은 이름은 아무 경고 없이 내보내졌습니다.

**해결.** 접두사를 붙입니다(`applog_open`, `applog_close`). 그렇게 고친 사본은 경고 없이 컴파일되었습니다.

### 4.4 그 밖의 실수

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| 선언의 정수 크기나 필드 순서가 C와 다름 | 경고 없이 빌드되고 실행 중에 잘못된 값 | 헤더와 대조, 선언을 한 파일에 모음, [`ch35-bindgen`](../ch35-bindgen/README.md) | 2.2 |
| 선언에 `&str`, `String`, 튜플, `char`를 씀 | `improper_ctypes` 경고 | `*const c_char`, `#[repr(C)]` 구조체, `u32` | 2.2, 01 |
| `#[link(name = …)]`만 쓰고 위치를 안 알림 | `could not find native static library` | 빌드 스크립트에서 `cc` 또는 `rustc-link-search` | 2.3 |
| `links`를 적고 빌드 스크립트가 없음 | ``links to `tally` but does not have a custom build script`` | `build.rs`를 둠 | 2.3 |
| C 파일을 `rerun-if-changed`로 알리지 않음 | C를 고쳐도 다시 컴파일되지 않음 | `cargo::rerun-if-changed=csrc/…` | 2.3 |
| `#[repr]` 없는 구조체·뉴타입을 경계에 씀 | `improper_ctypes(_definitions)` 경고, C와 다른 배치 | `#[repr(C)]`, `#[repr(transparent)]` | 2.4, 02 |
| C가 돌려준 정수를 Rust 열거형으로 바로 받음 | 목록에 없는 값이 오면 미정의 동작 | 정수로 받아 `TryFrom` | 2.4, 02 |
| `Box`를 버리고 주소만 돌려줌 | 해제 후 사용 | `Box::into_raw`와 짝지은 해제 함수 | 2.5, 03 |
| C가 할당한 메모리를 Rust가 해제(또는 반대) | 할당기가 달라 미정의 동작 | 같은 쪽의 해제 함수 | 2.5 |
| 조건이 있는 C 함수를 `safe fn`으로 선언 | 경고 없음, 안전한 코드가 미정의 동작을 부름 | `unsafe fn` + 조건을 타입으로 보장하는 래퍼 | 2.6, 05 |
| `unsafe` 없는 `extern "C" { }` / `#[no_mangle]` (2024) | `extern blocks must be unsafe` / `unsafe attribute used without unsafe` | `unsafe extern "C"`, `#[unsafe(no_mangle)]` | Part 2 ch4 4.5절 |
| `unsafe extern` 블록의 표시 없는 함수를 그냥 부름 | E0133 | `unsafe { }`, 정말 안전하면 `safe fn` | Part 2 ch4 4.8절 |
| 내보낼 함수에 `#[unsafe(no_mangle)]`을 빠뜨림 | ``function … is never used`` 경고, 링크 시 `Undefined symbols` | `#[unsafe(no_mangle)]` | 2.7 |
| 포인터 인자를 역참조하는 `pub extern "C" fn` | clippy `not_unsafe_ptr_arg_deref`(deny) | `pub unsafe extern "C" fn` + `# Safety` | 2.7 |
| 받은 포인터를 `&*p`로 바꿈 | NULL이면 미정의 동작 | 읽을 포인터는 `p.as_ref()`/`as_mut()` | 2.7, 06 |
| 출력 매개변수를 `as_mut()`으로 참조로 바꿔 씀 | 경고 없음. C가 초기화하지 않은 변수를 넘기면 미정의 동작 | `is_null()` 확인 뒤 `out.write(값)` | 2.7, 06, 07 |
| 지역 변수의 주소를 돌려줌 | `dangling_pointers_from_locals` 경고(지역 배열의 `as_ptr()`는 경고 없음) | 힙(`Box::into_raw`)이나 정적 데이터 | 2.7, 06 |
| 내보내는 함수 안에서 패닉 | `extern "C"`면 프로세스 중단 | `catch_unwind`로 에러 코드로 | 2.8, 07 |

## 5. 연습 문제

### 연습 1. C 라이브러리에 함수 하나 더하기

[`projects/tally_ffi`](projects/tally_ffi/csrc/tally.h)에 기록을 모두 지우는 `int32_t tally_reset(tally *t);`(이름은 그대로 둠)를 더하세요. C 헤더와 소스, `src/ffi.rs`의 선언, 래퍼의 메서드, 단위 테스트를 차례로 고칩니다. 선언은 `safe fn`이어야 할까요? 래퍼 메서드는 `&self`와 `&mut self` 중 무엇을 받아야 할까요?

<details>
<summary>힌트</summary>

C 쪽은 `summary`와 `buckets`를 0으로 만들고 이름은 그대로 둡니다(`memset(&t->summary, 0, sizeof t->summary)` 등). 포인터를 받으므로 선언은 `pub unsafe fn tally_reset(t: *mut tally) -> i32;`입니다. NULL이 아니어도 해제된 포인터라면 미정의 동작이니 `safe fn`이 될 수 없습니다. C가 tally를 고치므로 래퍼는 `pub fn reset(&mut self)`이고, `record`처럼 상태 코드를 `debug_assert_eq!`로 확인합니다. 헤더를 고친 뒤 `cargo build`가 C 파일을 다시 컴파일하는지도 보세요. `build.rs`의 `rerun-if-changed` 두 줄 덕분입니다.

</details>

### 연습 2. 패닉 메시지를 C에 전하기

[`projects/textstat_export`](projects/textstat_export/src/textstat.rs)의 `guard`가 패닉을 붙잡으면 메시지를 버립니다. 마지막 패닉 메시지를 C가 가져갈 수 있도록 `int32_t textstat_last_error(char *buf, size_t buf_len);`을 더하세요. `driver.c`의 4번 구역에서 이 함수로 메시지를 출력해 봅니다.

<details>
<summary>힌트</summary>

`thread_local! { static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) }; }`을 두고, `guard`의 `Err(payload)` 갈래에서 `payload.downcast_ref::<&str>()`과 `downcast_ref::<String>()`으로 메시지를 꺼내 저장합니다([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 2.9절). 버퍼에 쓰는 부분은 `textstat_describe`를 그대로 따르면 할당과 해제가 모두 C 쪽에 남습니다. 스레드 지역 변수를 쓰는 이유는 C 프로그램의 다른 스레드가 같은 함수를 불러도 메시지가 섞이지 않게 하려는 것입니다. 이 함수 자체도 `guard`로 감싸세요.

</details>

### 연습 3. 헤더의 스레드 규칙을 `Send`/`Sync`로 옮기기

다음 세 문장 각각에 대해 `Tally` 래퍼가 `Send`와 `Sync`를 구현해야 하는지 정하세요. 그리고 `projects/tally_ffi`의 사본에서 `unsafe impl Send`를 지우고 `main.rs`의 6번 구역을 빌드해 어떤 에러가 나는지, `std::thread::scope`로 `&Tally`를 두 스레드에 나눠 주는 코드를 더하면 지금 판에서 어떤 에러가 나는지 확인하세요.

1. "tally는 만든 스레드에서만 써야 한다(스레드마다 다른 내부 캐시를 씀)."
2. "tally를 다른 스레드로 넘겨 써도 되지만, 동시에 부르면 안 된다." (지금 헤더)
3. "읽기 함수(`const tally *`를 받는 함수)는 여러 스레드에서 동시에 불러도 된다."

<details>
<summary>힌트</summary>

1은 둘 다 구현하지 않습니다(기본값). 2는 `Send`만, 3은 2에 더해 `Sync`까지입니다. 3에서 `Sync`가 맞으려면 `&self`를 받는 래퍼 메서드가 모두 `const tally *` 함수만 불러야 합니다. `record`가 `&mut self`인 것이 여기서 의미를 가집니다. `unsafe impl Send`를 지우면 6번 구역의 `thread::spawn`이 4.1절과 같은 E0277(`` `NonNull<tally::ffi::tally>` cannot be sent between threads safely ``)로 실패합니다. 지금 판에서 `&Tally`를 스코프 스레드 둘에 나눠 주면 `Sync`가 없어 E0277(`` `NonNull<tally::ffi::tally>` cannot be shared between threads safely ``)입니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 34: Control what crosses FFI boundaries](https://effective-rust.com/ffi.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- The Rustonomicon, [Foreign Function Interface](https://doc.rust-lang.org/nomicon/ffi.html): 불투명 구조체 표현, 콜백, 되감기
- The Reference: [External blocks](https://doc.rust-lang.org/reference/items/external-blocks.html)(`safe`/`unsafe` 항목, `#[link]`), [Application binary interface](https://doc.rust-lang.org/reference/abi.html)(`no_mangle`), [Type layout](https://doc.rust-lang.org/reference/type-layout.html)(`repr(C)`, `repr(transparent)`), [Functions](https://doc.rust-lang.org/reference/items/functions.html)(Extern function qualifier, Unwinding), [Panic](https://doc.rust-lang.org/reference/panic.html)(Unwinding across FFI boundaries)
- The Cargo Book, [Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html): Outputs of the Build Script(`cargo::rustc-link-lib`, `cargo::rustc-link-search`, MSRV 1.77), The `links` Manifest Key, `*-sys` Packages
- [`cc` 1.11.2 문서](https://docs.rs/cc/1.11.2/cc/): `Build::new().file(..).compile(..)`과 출력하는 링크 지시문
- Edition Guide: [Unsafe extern blocks](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html), [Unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html)
- 표준 라이브러리 문서: [`core::ffi`](https://doc.rust-lang.org/core/ffi/index.html)(`c_int`, `c_long` 등), [`std::panic::catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html), [`Box::into_raw`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.into_raw)
- 로컬 문서의 릴리스 노트(`rustc --print sysroot` 아래 `share/doc/rust/html/releases.md`): 1.64.0(`core::ffi`의 C 타입), 1.71.0(`extern "C-unwind"`), 1.77.0(Cargo의 `cargo::` 지시문), 1.81.0(`extern "C"` 함수의 패닉 중단), 1.82.0(`unsafe extern`, unsafe 속성), 1.91.0(`dangling_pointers_from_locals`), 1.98.0(런타임 심볼 정의 린트)
