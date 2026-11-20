# ch35. FFI 선언은 손으로 쓰지 말고 bindgen으로 생성하기

[ch34](../ch34-ffi-boundaries/README.md)에서는 C 헤더의 선언을 Rust로 손으로 옮겼고, 그 선언이 C와 어긋나도 컴파일러와 링커가 알려 주지 않는다는 것을 보았습니다. 이 챕터는 그 어긋남을 실제로 재현합니다. ch34의 C 라이브러리를 다음 판으로 올리고, 손으로 쓴 선언이 조용히 틀린 값을 내는 동안 헤더에서 생성한 선언은 저절로 맞거나 래퍼를 컴파일 에러로 멈추는 것을 견줍니다. 다루는 판단은 셋입니다. 언제 손으로 쓰고 언제 생성하는가, 생성은 빌드할 때 할 것인가 결과를 커밋해 둘 것인가, 생성한 선언을 어떻게 감싸는가입니다.

- 원제: [Item 35: Prefer `bindgen` to manual FFI mappings](https://effective-rust.com/bindgen.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch35-bindgen`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), `bindgen` 0.73.2, `cc` 1.11.2, Apple clang 21.0.0(C 컴파일러), macOS aarch64

> **이 챕터의 `unsafe` 코드는 Miri로 확인하지 못했습니다.** 이 환경에는 nightly 툴체인이 없고, Miri는 C 코드를 실행하지도 못합니다. ch34와 같은 방식으로 모든 `unsafe` 블록에 `// SAFETY:` 주석을 달고, 그 전제 조건을 바로 위 코드나 헤더의 약속이 대도록 썼습니다. 선언이 C와 어긋난 빌드 가운데 미정의 동작이 되는 것(인자 수가 다른 호출)은 실행하지 않습니다.

**준비 사항.** `examples/`와 `compile_fail/`은 툴체인만 있으면 됩니다. `projects/tally_bindings`는 C 컴파일러가 필요하고(`cc` 크레이트가 부름), `bindgen`을 쓰는 빌드는 libclang도 필요합니다.

| 무엇을 빌드하나 | 필요한 것 |
| --- | --- |
| `handwritten/`, `generated/`의 기본 빌드(커밋해 둔 생성 결과) | C 컴파일러. crates.io에서 `cc`를 받음 |
| `generated/`에서 `--features tally-sys/bindgen` | C 컴파일러, libclang. crates.io에서 `bindgen`과 그 의존성을 받음 |

libclang은 bindgen 문서의 요구 사항 절대로 설치합니다(macOS는 Homebrew의 `llvm`, Debian 계열은 `libclang-dev`, Fedora는 `clang-devel`). macOS는 Xcode 명령줄 도구에도 `/Library/Developer/CommandLineTools/usr/lib/libclang.dylib`가 들어 있습니다. 이 환경에는 그것과 Homebrew LLVM 23.1.2가 함께 있었는데, `LIBCLANG_PATH` 없이 실행하자 bindgen이 Homebrew 쪽을 골랐습니다. bindgen이 쓰는 `clang-sys` 1.9.1의 소스를 보면 `LIBCLANG_PATH`가 있으면 그 경로만 보고, 없으면 `llvm-config`, Xcode, 알려진 디렉터리(명령줄 도구 포함)를 뒤져 가장 높은 버전을 고릅니다. `LIBCLANG_PATH=/Library/Developer/CommandLineTools/usr/lib`를 주면 Apple clang 21.0.0을 썼고, 이 챕터의 생성 결과는 두 경우가 글자 하나 다르지 않았습니다. 어느 것도 찾지 못하면 빌드 스크립트가 ``set the `LIBCLANG_PATH` environment variable``을 권하는 메시지와 함께 실패합니다(`clang-sys` 소스로 확인, 실행하지는 않음).

책의 6장(Beyond Standard Rust)을 마무리하는 Item입니다. 다음 내용을 안다고 보고 진행합니다.

- [ch34](../ch34-ffi-boundaries/README.md) 전체: 선언과 정의, `#[repr(C)]`, `cc`와 `links`, 할당한 쪽이 해제, `safe fn` 판단, 안전한 래퍼. 이 챕터는 그 기본을 다시 설명하지 않습니다.
- [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.7~2.8절: `unsafe extern` 블록과 `safe fn`/`unsafe fn` 항목
- [ch26](../ch26-feature-creep/README.md): 기능과 `dep:`, [ch25](../ch25-dependency-graph/README.md) 2.3절: `links` 충돌, [ch29](../ch29-listen-to-clippy/README.md) 2.3절: `#[expect]`와 `#[allow]`

빌드 스크립트 전반은 Part 3 `ch8-build-scripts`에서, FFI의 깊은 내용(콜백, 불투명 타입)은 Part 10 `ch12-ffi-beneath-std`에서, 레이아웃 규칙은 Part 10 `ch2-data-layout`에서, CI 구성은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. C 헤더가 바뀔 때 손으로 쓴 선언이 어긋나는 방식을 들고, 그중 미정의 동작이 되는 것과 값을 잘못 해석하는 것, 레이아웃 검사가 잡는 것과 놓치는 것을 구분해 설명할 수 있다.
2. 빌드 스크립트에서 `bindgen`으로 헤더의 선언을 생성하고, allowlist와 생성 옵션을 근거를 들어 고를 수 있다. 생성 결과가 손으로 쓴 선언과 어떻게 다르고 래퍼의 어느 줄이 바뀌는지 설명할 수 있다.
3. `-sys` 크레이트와 안전한 래퍼 크레이트로 층을 나누고, `links`와 `DEP_<이름>_<키>` 환경 변수로 빌드 스크립트 사이에 정보를 넘길 수 있다.
4. 생성 결과를 빌드할 때 만들지 커밋해 둘지 득실을 따져 고르고, 커밋해 둔 결과가 헤더와 어긋나면 실패하는 검사를 만들 수 있다.

## 2. 핵심 개념

2.1절은 헤더가 바뀌었을 때 무슨 일이 일어나는지 정리하는 **이해** 부분입니다. 2.2~2.6절이 지침이고, 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 씁니다. 지침 1~5의 줄기(생성하기, allowlist로 좁히기, `-sys`와 래퍼로 나누기, 커밋한 결과를 CI에서 비교하기, C++에는 `cxx`)는 책이 권하는 것이고, 더한 판단 기준 가운데 책에 없는 것은 2.7절에 밝혔습니다.

### 2.1 지도: 헤더가 바뀌면 누가 알려 주는가

[`projects/tally_bindings`](projects/tally_bindings/csrc/v2/tally.h)에는 ch34의 히스토그램 라이브러리 `tally`가 세 판으로 들어 있습니다. 빌드 스크립트가 환경 변수 `TALLY_C_VERSION`(기본 1)으로 판을 고르며, "시스템의 C 라이브러리가 새 판으로 올라간 상황"을 흉내 냅니다.

| 판 | 바뀐 것 | C 프로그램(다시 컴파일) | 손으로 쓴 Rust 선언(v1 기준) | 실행하면 |
| --- | --- | --- | --- | --- |
| v1 | ch34 그대로 | — | 맞음 | 맞는 값 |
| v2 | `TALLY_NAME_MAX` 31→15 | 저절로 맞음 | 경고 없이 빌드 | 낡은 상수 |
| v2 | 상태 코드 `TALLY_ERR_EMPTY_NAME = -2` 추가, `NAME_TOO_LONG` -2→-3 | 저절로 맞음 | 경고 없이 빌드 | 상태 코드를 잘못 해석 |
| v2 | `tally_summary`에서 `max`를 `min` 앞으로 | 저절로 맞음 | 경고 없이 빌드 | 두 값이 뒤바뀌어 읽힘 |
| v3 | `tally_record`에 `uint32_t weight` 인자 추가 | C 컴파일러가 에러 | 경고 없이 빌드 | **미정의 동작**(실행하지 않음) |

C 프로그램은 바뀐 헤더로 다시 컴파일하는 것만으로 맞춰집니다. 이름(`TALLY_NAME_MAX`, `s.min`)으로 쓴 코드는 새 값과 새 오프셋을 그대로 따라가고, 인자가 바뀐 호출은 `too few arguments to function call, expected 3, have 2`로 거부됩니다(v3 헤더로 작은 C 파일을 컴파일해 확인). 손으로 쓴 Rust 선언은 헤더를 다시 읽지 않으므로 다섯 줄 모두에서 경고 없이 빌드됩니다.

**미정의 동작과 잘못 해석한 값은 다릅니다.** 이 챕터가 실행하는 어긋남과 실행하지 않는 어긋남의 경계입니다.

- **상수의 값이 바뀜(v2의 앞 두 줄).** 함수의 시그니처는 그대로이므로 선언은 정의와 맞습니다. Rust는 C가 돌려준 `i32`를 낡은 상수와 비교할 뿐이라, 결과가 틀려도 미정의 동작은 아닙니다. ch34가 C의 enum을 Rust 열거형이 아니라 정수로 받은 덕분이기도 합니다.
- **같은 타입의 두 필드가 자리를 바꿈(v2의 셋째 줄).** C의 호출 규약은 필드 이름을 보지 않습니다. Rust 쪽 정의와 v2 헤더의 정의는 필드 타입이 순서까지 같아(`u64` 셋, `i64` 둘) 이름 말고는 구별되지 않고, 값이 다른 이름으로 읽힐 뿐입니다. 타입이 다른 필드끼리 자리를 바꿨다면 이야기가 다릅니다. 표준 라이브러리의 `fn` 타입 문서(ABI compatibility 절)는 `i32`와 `u32`조차 호출 규약이 같다고 보장하지 않으며, 이 챕터는 그런 어긋남을 실행하지 않습니다.
- **인자 수나 폭, 구조체 크기가 바뀜(v3).** 같은 문서는 인자 수가 같아야 한다고 못 박습니다. 인자 2개로 선언한 함수로 인자 3개짜리 C 함수를 부르면 미정의 동작입니다. 데모 프로그램은 이런 빌드를 실행하지 않고 종료 코드 2로 멈춥니다.

**레이아웃 검사는 일부만 봅니다.** bindgen은 구조체마다 크기, 정렬, 필드 오프셋을 확인하는 코드를 함께 생성합니다. [`01_layout_checks.rs`](examples/01_layout_checks.rs)는 "바뀐 헤더에서 잰 숫자로 낡은 정의를 검사한다면"을 가정해 어긋남 다섯 가지를 견줍니다. 필드가 자리를 바꾸거나(1), 늘거나(4), 앞쪽 필드가 넓어지면(5) 걸리지만, 크기가 같은 다른 타입으로 바뀌거나(2) 줄어든 폭이 패딩에 가려지면(3) 모든 검사를 통과합니다. 3번처럼 C가 `uint16_t`로 쓰고 Rust가 `u32`로 읽으면 초기화되지 않은 패딩 바이트를 읽게 되어 그 자체로 미정의 동작입니다. 게다가 bindgen의 레이아웃 검사는 **같은 헤더에서 만든 정의**를 검사합니다([`02_const_layout_asserts.rs`](examples/02_const_layout_asserts.rs)). 생성할 때의 C 배치와 Rust의 `#[repr(C)]` 배치가 같은지 확인할 뿐, 그 뒤에 헤더가 바뀌었는지는 모릅니다.

### 2.2 지침 1: 사소한 것을 넘는 FFI 선언은 헤더에서 생성한다

**문제 상황.** C 라이브러리가 v2로 올라갔습니다. 함수 시그니처는 하나도 바뀌지 않았고 C 사용자는 다시 컴파일만 하면 됩니다. Rust 쪽은 무엇을 해야 할까요?

**비권장 코드.** ch34의 [`src/ffi.rs`](projects/tally_bindings/handwritten/src/ffi.rs)를 그대로 둡니다. 헤더의 선언을 바로 위 주석에 적어 대조하기 쉽게 만든 좋은 손 선언이지만, 대조는 사람이 할 때만 일어납니다. `handwritten/`에서 `TALLY_C_VERSION=2 cargo run`을 실행하면 경고 하나 없이 빌드되어 다음을 출력합니다(3절에 전체 출력).

| 데모 항목 | v1 (모든 판이 같음) | v2, 손으로 쓴 선언 | v2, 빌드 때 생성한 선언 |
| --- | --- | --- | --- |
| `NAME_MAX` | 31 | 31 | 15 |
| Rust가 읽은 `range` | `Some((-3, 250))` | `Some((250, -3))` | `Some((-3, 250))` |
| `NAME_MAX`에 맞춰 자른 이름 | 31바이트, `Ok(())` | 31바이트, `Err(Status(-3))` | 14바이트, `Ok(())` |
| 빈 이름 | `Ok(())` | `Err(NameTooLong { len: 0, max: 31 })` | `Err(Status(-2))` |
| 60바이트 이름 | `NameTooLong { len: 60, max: 31 }` | `Status(-3)` | `NameTooLong { len: 60, max: 15 }` |

손으로 쓴 판은 같은 실행에서 C가 직접 쓴 표 머리(`min=-3 max=250`)와 다른 최솟값·최댓값을 보고하고, 0바이트 이름이 "너무 길다"고 말합니다. 래퍼의 단위 테스트 8개 가운데 2개(`summary_counts_out_of_range`, `name_errors`)가 v2에서 실패하므로 테스트가 일부는 잡아 주지만, 테스트가 확인한 값만입니다. 테스트가 기대값으로 같은 `NAME_MAX`를 쓰므로, 상태 코드가 그대로이고 `NAME_MAX`만 바뀌었다면 실패하는 테스트가 없었을 것입니다.

**권장 코드.** 빌드 스크립트에서 `bindgen`으로 같은 헤더의 선언을 생성하고, `OUT_DIR`의 결과를 `include!`로 끌어옵니다. [`generated/tally-sys/build.rs`](projects/tally_bindings/generated/tally-sys/build.rs)의 핵심입니다.

```rust
let bindings = bindgen::Builder::default()
    .header(header.to_str().expect("헤더 경로가 UTF-8이어야 합니다"))
    .allowlist_function("tally_.*")
    .allowlist_type("tally_.*")
    .allowlist_var("TALLY_.*")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .generate()
    .expect("bindgen이 헤더에서 선언을 생성하지 못했습니다");
bindings.write_to_file(out.join("bindings.rs")).expect("…");
```

```rust
// tally-sys/src/lib.rs (bindgen 기능을 켰을 때)
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

`CargoCallbacks`는 헤더와 헤더가 포함한 파일(`stdint.h` 등)마다 `cargo:rerun-if-changed`를 출력해, 헤더가 바뀌면 다시 생성하게 합니다. 같은 v2에서 이 선언 위의 래퍼는 위 표의 오른쪽 열처럼 맞는 값을 냅니다. 새로 생긴 상태 코드 -2는 래퍼가 아직 모르므로 `Status(-2)`로 남는데, 이것은 거짓 해석이 아니라 "모른다"는 정직한 답이고, 래퍼에 배리언트를 더하는 일은 사람의 몫입니다(연습 1). v3에서는 다시 생성한 선언이 인자 3개가 되어 래퍼가 컴파일되지 않습니다.

```text
error[E0061]: this function takes 3 arguments but 2 arguments were supplied
   --> tally/src/lib.rs:169:31
    |
169 |         let status = unsafe { ffi::tally_record(self.raw.as_ptr(), sample) };
    |                               ^^^^^^^^^^^^^^^^^--------------------------- argument #3 of type `u32` is missing
    |
note: function defined here
   --> …/target/debug/build/tally-sys/<해시>/out/bindings.rs:64:12
    |
 64 |     pub fn tally_record(t: *mut tally, sample: i64, weight: u32) -> i32;
```

같은 상황에서 손으로 쓴 판은 `Compiling tally … Finished`로 조용히 빌드됩니다. 책이 이 지침의 근거로 드는 비대칭이 이것입니다. C 선언이 C 정의와 어긋나면 C 컴파일러가 알려 주지만, 손으로 쓴 Rust 선언이 C 선언과 어긋나는 것은 아무도 알려 주지 않습니다. Rust 선언을 C 헤더에서 만들면 C 컴파일러가 보는 것과 같은 원본을 보게 됩니다.

**따르지 않아도 되는 경우.**

- **사소한 선언.** 책의 표현대로 "사소한 것을 넘는" 선언이 대상입니다. ch34 예제 05의 `hypot` 같은 함수 한두 개는 생성 도구를 들이는 비용이 더 큽니다. C 표준 라이브러리의 선언은 이미 `libc` 크레이트가 모아 두었습니다(ch34 2.3절).
- **bindgen이 옮기지 못하는 부분.** 시험해 보니 함수 모양 매크로(`#define TALLY_SQUARE(x) …`)와 `static inline` 함수는 아무것도 생성되지 않았습니다. 그런 API는 손으로 감싸거나, `static` 함수를 부르는 C 파일을 생성해 주는 `wrap_static_fns` 옵션을 검토합니다(생성된 C 파일도 컴파일해 링크해야 함).
- 손으로 쓴 선언을 꼭 유지해야 한다면, 실제 C 라이브러리를 부르는 테스트를 선언마다 두어 어긋남의 일부라도 잡습니다. 위에서 본 대로 테스트가 확인하는 값까지만입니다.

### 2.3 지침 2: 생성 범위는 접두사로 좁히고, 생성 결과를 읽어 본다

**문제 상황.** 생성한 선언은 손으로 쓴 선언과 똑같은 모양일까요? 그리고 헤더 하나가 `<stdint.h>`를 포함하는 순간, 생성기는 무엇까지 옮길까요?

**생성 결과.** bindgen 0.73.2가 v1 헤더에서 만든 [`committed_bindings.rs`](projects/tally_bindings/generated/tally-sys/src/committed_bindings.rs)의 일부입니다(79줄).

```rust
/* automatically generated by rust-bindgen 0.73.2 */

pub const TALLY_MAX_BUCKETS: u32 = 32;
pub const TALLY_NAME_MAX: u32 = 31;
pub const tally_status_TALLY_OK: tally_status = 0;
pub const tally_status_TALLY_ERR_NULL: tally_status = -1;
pub const tally_status_TALLY_ERR_NAME_TOO_LONG: tally_status = -2;
pub type tally_status = ::std::os::raw::c_int;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tally_config {
    pub low: i64,
    pub bucket_width: u32,
    pub bucket_count: u32,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of tally_config"][::std::mem::size_of::<tally_config>() - 16usize];
    ["Alignment of tally_config"][::std::mem::align_of::<tally_config>() - 8usize];
    // …필드 오프셋 세 줄
};
// …tally_summary와 그 레이아웃 검사
#[repr(C)]
#[derive(Debug)]
pub struct tally {
    _unused: [u8; 0],
}
unsafe extern "C" {
    pub fn tally_default_config() -> tally_config;
}
// …함수마다 unsafe extern "C" 블록 하나씩, 모두 9개
```

[`03_compare_declarations.rs`](examples/03_compare_declarations.rs)가 두 파일을 읽어 항목별로 견준 결과를 표로 옮깁니다.

| 항목 | 손으로 쓴 선언(ch34) | 생성한 선언 | 래퍼에 미친 영향 |
| --- | --- | --- | --- |
| 함수 9개의 시그니처 | — | 모두 같음 | 없음 |
| `safe` 표시 | `safe fn` 2개, `unsafe fn` 7개 | 표시 없음(모두 unsafe) | `unsafe` 블록 2개와 안전성 주석이 늘어남 |
| `#define` 정수 | `u32`, `usize`로 골라 적음 | 모두 `u32` | `as usize` 한 곳(빼면 E0308, 4.1절) |
| enum 값 | `TALLY_OK: i32` | `tally_status_TALLY_OK: tally_status`(`c_int`의 별칭) | `match` 패턴의 이름 |
| 구조체 필드·파생 | `Debug, Clone, Copy` | 같은 필드, `Debug, Copy, Clone` | 없음 |
| 불투명 타입 | `_opaque: [u8; 0]` | `_unused: [u8; 0]`, `Debug` 파생 | 없음 |
| 레이아웃 검사 | 없음 | 크기 2, 정렬 2, 오프셋 8 | 없음(컴파일 때 확인) |

래퍼([`generated/tally/src/lib.rs`](projects/tally_bindings/generated/tally/src/lib.rs))에서 바뀐 곳에는 `[생성판]` 표시를 달았고 모두 6곳입니다. 책은 생성 결과가 Item 34에서 손으로 쓴 선언과 같다고 하지만, 2024 에디션에서 손으로 쓴 선언에는 사람이 더한 판단(`safe fn`, 쓰기 좋은 정수 타입)이 들어 있고 생성기는 그것을 알 수 없습니다. 그 판단은 이제 래퍼의 `unsafe` 블록과 `// SAFETY:` 주석으로 옮겨 갑니다. 0.73.2의 Builder에는 `safe fn`을 생성하는 옵션이 없습니다(소스로 확인).

**비권장 코드: 범위를 좁히지 않음.** allowlist 없이 같은 헤더를 생성하면 79줄이 271줄이 되고, `pub const __DARWIN_C_LEVEL: u32 = 900000;` 같은 macOS 내부 매크로가 175개 항목 사이에 섞입니다. 생성 결과가 플랫폼에 묶일 뿐 아니라, Homebrew LLVM 23.1.2와 Apple clang 21.0.0의 결과가 한 줄(`__has_bounds_safety_attributes`) 달랐습니다. allowlist를 둔 결과는 두 libclang에서 같았습니다.

**권장 코드: 접두사로 좁힙니다.** ch34의 헤더가 공개 이름에 `tally_`/`TALLY_` 접두사를 붙인 것이 여기서 쓸모가 있습니다. 책이 쓰는 CLI의 `--allowlist-function`·`--allowlist-type`에 해당하는 Builder 메서드에 변수용 `allowlist_var`를 더했습니다. 패턴은 이름 전체와 맞춰 봅니다(0.73.2 소스에서 `^(패턴)$`로 감쌈). allowlist에 걸린 항목이 쓰는 타입은 저절로 따라 들어옵니다. 시험 삼아 `allowlist_function`만 남기자 구조체 셋은 함수 시그니처를 따라 생성됐지만, 시그니처에 쓰이지 않는 `tally_status`의 상수와 `#define` 둘은 빠졌습니다. 타입과 변수도 따로 고르는 이유입니다.

**옵션은 기본값에서 출발합니다.** 이 챕터의 `tally-sys`는 allowlist와 `CargoCallbacks` 말고는 기본값을 씁니다. 판단에 필요한 옵션만 정리합니다.

| 옵션 | 0.73.2의 기본 | 바꿀 때 |
| --- | --- | --- |
| `rust_target` | 이 bindgen이 아는 가장 새 stable(1.82) | 1.82 미만으로 낮추면 `unsafe` 없는 `extern "C" {`를 생성 |
| `rust_edition` | 대상 판이 지원하는 가장 새 에디션 | 이 헤더에서는 2021로 바꿔도 결과가 같았음 |
| enum 표현 | 상수(`tally_status_TALLY_OK`) | `constified_enum_module`(모듈 안의 상수), `newtype_enum`(뉴타입). `rustified_enum`은 C가 목록 밖의 값을 주면 미정의 동작이라고 문서가 경고 |
| `layout_tests` | 켬(`const _` 블록) | 끄면 책의 `--no-layout-tests`와 같음 |
| `derive_*` | `Debug`·`Copy`(`Clone`) 켬 | `derive_default`, `derive_eq` 등을 켬 |
| `use_core` | `::std::os::raw::c_char` | `::core::ffi::c_char`. `no_std` 크레이트([ch33](../ch33-no-std/README.md))에 필요 |
| `generate_cstr` | 끔(문자열 매크로는 `&[u8; N]`) | `&CStr`(`c"…"`). 문서는 앞으로 기본으로 바뀐다고 예고 |
| `wrap_unsafe_ops` | 끔 | 비트필드 접근자처럼 본문이 있는 `unsafe fn`을 생성할 때. 이 헤더는 해당 없음 |
| `ParseCallbacks::int_macro` | `#define` 정수는 `u32`/`i32` | 매크로마다 타입을 고름(래퍼의 `as usize`를 없앨 수 있음) |

**2024 에디션에서 컴파일되는가.** 기본 설정의 0.73.2는 `unsafe extern "C"` 블록을 생성하므로 그대로 컴파일됩니다. 대상 판을 1.81로 낮춘 결과(`rust_target(RustTarget::stable(81, 0)?)`)와, 시험 삼아 받아 본 bindgen 0.69.5의 기본 결과는 `extern "C" {`였고, 2024 에디션으로 컴파일하면 블록마다 `error: extern blocks must be unsafe`가 나서 에러 9개로 실패했습니다(같은 파일을 2021 에디션으로는 컴파일할 수 있음). 2024 에디션 크레이트에서는 대상 판을 1.82 아래로 내리지 말고, 옛 bindgen이라면 생성 결과의 첫 블록을 확인합니다.

**린트 억제는 `#[allow]`로.** 생성된 이름이 Rust 규칙과 달라 `non_upper_case_globals`(enum 상수 셋)와 `non_camel_case_types`(`tally_status` 별칭)가 경고합니다. [ch29](../ch29-listen-to-clippy/README.md)는 이유를 남긴 `#[expect]`를 권했지만, 생성 코드에는 `#[allow]`가 맞습니다. 무엇이 생성될지는 헤더와 옵션이 정하기 때문입니다. bindgen 튜토리얼이 권하는 `non_snake_case`를 `#[expect]`로 적으면 이 헤더에서는 `this lint expectation is unfulfilled`가 났고, enum 표현을 `constified_enum_module`로 바꾸자 나머지 두 린트도 같은 경고를 냈습니다.

**따르지 않아도 되는 경우.** 헤더 전체가 그 라이브러리의 API이고 접두사가 없다면 `allowlist_file`로 파일 단위로 고르거나, 범위를 좁히지 않는 대신 생성 결과를 플랫폼마다 따로 다룹니다.

### 2.4 지침 3: `-sys` 크레이트에는 생성한 선언만 두고, 안전한 API는 그 위 크레이트가 만든다

**문제 상황.** 생성한 선언은 모두 `unsafe`이고 이름도 C 그대로입니다. 이것을 어디에 두고, 누가 감쌀까요?

**Cargo의 관례.** Cargo Book의 Build Scripts 장 "`*-sys` Packages" 절에 따르면, 이름이 `foo-sys`인 패키지는 네이티브 라이브러리 `libfoo`를 링크하고, `libfoo`의 타입과 함수의 **선언**을 제공하되 더 높은 수준의 추상화는 제공하지 않습니다. 그 위에 안전한 API를 주는 `-sys` 없는 짝 패키지를 두는 것이 흔한 모양이라고 소개합니다. 책도 같은 층 나누기를 권합니다. `unsafe`가 `-sys` 크레이트에, 그것을 감싸는 판단이 래퍼 크레이트에 모입니다([ch16](../ch16-avoid-unsafe/README.md)).

**비권장 코드: 같은 라이브러리를 링크하는 패키지가 둘.** 각 크레이트가 C 라이브러리를 직접 컴파일해 링크하면, 한 그래프에 들어온 순간 Cargo가 거부합니다. 이 챕터의 손으로 쓴 판(`links = "tally"`)과 `tally-sys`(`links = "tally"`)를 한 워크스페이스에 넣어 보았습니다(서로 의존하지 않아도).

```text
error: Attempting to resolve a dependency with more than one crate with links=tally.
This will not build as is. Consider rebuilding the .lock file.
```

`links` 값은 네이티브 라이브러리 하나에 패키지 하나만 허용합니다. 같은 심볼이 두 벌 링크되는 것을 막는 장치이고, 그래서 손으로 쓴 판은 [`handwritten/`](projects/tally_bindings/handwritten/Cargo.toml)이라는 별도 워크스페이스에 두었습니다. 이 규칙이 "모두가 같은 `-sys` 크레이트 하나에 의존하라"는 관례의 이유입니다.

**권장 코드.** [`generated/`](projects/tally_bindings/generated/Cargo.toml) 워크스페이스는 두 층입니다.

| 패키지 | 담는 것 | `unsafe` |
| --- | --- | --- |
| `tally-sys` | `links = "tally"`, `build.rs`(C 컴파일 + 선언 생성), 생성한 선언 | 선언 자체(부르려면 `unsafe`) |
| `tally` | ch34의 안전한 래퍼를 옮긴 것, `tally-sys`에 의존 | 래퍼 메서드 안의 블록 11개 |

`tally-sys`의 빌드 스크립트는 `cargo::metadata`로 값 셋을 내보냅니다. Cargo Book에 따르면 이 값은 **직접 의존하는** 패키지의 빌드 스크립트에 `DEP_<links 값>_<키>`(키는 대문자) 환경 변수로 전해집니다. `tally`의 [`build.rs`](projects/tally_bindings/generated/tally/build.rs)가 그것을 받아 데모 출력에 쓸 판 번호를 정합니다. `cargo build -vv -p tally`로 본 환경 변수입니다(경로는 줄임).

```text
DEP_TALLY_C_VERSION=1
DEP_TALLY_DECLARATIONS=committed
DEP_TALLY_INCLUDE=…/csrc/v1
```

ch34가 남긴 `cargo::metadata=include=…`가 여기서 `DEP_TALLY_INCLUDE`가 됩니다. 이 값은 같은 헤더로 C 코드를 함께 컴파일하는 크레이트가 `cc::Build::include`에 넘길 때 쓰는 것이고(Cargo Book의 "Using another `sys` crate" 예), `links`가 있는 패키지는 사용자가 `.cargo/config.toml`에서 빌드 스크립트를 미리 만든 라이브러리로 바꿔 끼울 수도 있습니다(Overriding Build Scripts).

**따르지 않아도 되는 경우.** 한 크레이트 안에서만 쓰고 다른 크레이트가 같은 라이브러리를 링크할 일이 없다면, ch34의 `tally_ffi`처럼 비공개 `ffi` 모듈과 래퍼를 한 크레이트에 두어도 됩니다. 라이브러리를 게시하거나 같은 C 라이브러리를 쓰는 크레이트가 여럿이 될 때 나눕니다.

### 2.5 지침 4: 생성 결과를 커밋해 둔다면 새로 생성한 결과와 비교하는 검사를 둔다

**문제 상황.** 빌드할 때마다 생성하면 늘 헤더와 맞지만, 이 크레이트를 쓰는 모든 사람이 libclang을 갖춰야 합니다. 생성 결과를 저장소에 커밋해 두면 그 부담이 사라지지만, 헤더가 바뀌면 손으로 쓴 선언과 똑같이 낡습니다.

| | 빌드할 때 생성 | 커밋해 둔 결과를 씀 |
| --- | --- | --- |
| 쓰는 쪽이 갖출 것 | C 컴파일러 + libclang | C 컴파일러 |
| `tally`의 빌드 그래프(자신 포함) | 패키지 26개(`bindgen`, `clang-sys`, `syn`, `regex` 등) | 패키지 5개 |
| 헤더와 맞는가 | 늘 맞음 | 검사가 있어야 맞음 |
| 생성 결과를 검토하기 | `OUT_DIR`을 찾아 봐야 함 | 커밋의 diff로 보임 |
| 플랫폼 | 빌드하는 플랫폼의 결과 | 생성한 플랫폼의 결과 |

`tally-sys`는 두 방식을 기능 하나로 고르게 했습니다. 기본은 커밋해 둔 `src/committed_bindings.rs`를 쓰고, `bindgen` 기능(`bindgen = ["dep:bindgen"]`, 선택적 빌드 의존성)을 켜면 빌드할 때 생성합니다. 기능을 켜도 꺼도 API가 같아야 하므로 이 기능은 둘이 맞을 때만 "더하기만 하는" 기능입니다([ch26](../ch26-feature-creep/README.md) 2.3절). 그래서 아래 검사가 필요합니다.

**비권장 코드: 커밋하고 끝.** `generated/`에서 `TALLY_C_VERSION=2 cargo run -p tally`를 실행하면, 생성한 선언인데도 손으로 쓴 판과 한 글자도 다르지 않은 틀린 출력이 나옵니다. 커밋해 둔 결과는 v1 헤더에서 만든 것이기 때문입니다. 그 안의 레이아웃 검사도 v1의 숫자를 담고 있어 아무것도 잡지 못합니다.

**권장 코드: 새로 생성해 비교하는 테스트.** 책은 생성 결과를 커밋해 둔다면 CI에서 새로 생성한 결과와 다를 때 실패하게 하라고 권합니다. [`tally-sys/src/lib.rs`](projects/tally_bindings/generated/tally-sys/src/lib.rs)의 `freshness` 테스트가 cargo만으로 그 일을 합니다. `bindgen` 기능을 켠 테스트 빌드에서 두 파일을 `include_str!`로 읽어 비교하고, 다르면 처음 다른 줄을 보여 주며 실패합니다.

```sh
cd appd-a/ch35-bindgen/projects/tally_bindings/generated
cargo test -p tally-sys --features bindgen                     # 통과: 1 passed
TALLY_C_VERSION=2 cargo test -p tally-sys --features bindgen   # 실패(종료 코드 101)
```

```text
커밋해 둔 생성 결과가 헤더에서 새로 생성한 결과와 다릅니다(4행)
  커밋: pub const TALLY_NAME_MAX: u32 = 31;
  새로: pub const TALLY_NAME_MAX: u32 = 15;
검토한 뒤 TALLY_BLESS=1을 붙여 다시 실행하면 새 결과로 덮어씁니다.
```

`TALLY_BLESS=1`을 붙이면 테스트가 커밋할 파일을 새 결과로 덮어씁니다. 그 전에 diff로 무엇이 바뀌었는지 읽는 것이 이 방식의 장점입니다. v1과 v2의 차이는 2.1절의 표가 그대로 Rust로 옮겨진 모양입니다.

```diff
-pub const TALLY_NAME_MAX: u32 = 31;
+pub const TALLY_NAME_MAX: u32 = 15;
-pub const tally_status_TALLY_ERR_NAME_TOO_LONG: tally_status = -2;
+pub const tally_status_TALLY_ERR_EMPTY_NAME: tally_status = -2;
+pub const tally_status_TALLY_ERR_NAME_TOO_LONG: tally_status = -3;
 …(tally_summary에서 min이 max 뒤로)
-    ["Offset of field: tally_summary::min"][::std::mem::offset_of!(tally_summary, min) - 24usize];
-    ["Offset of field: tally_summary::max"][::std::mem::offset_of!(tally_summary, max) - 32usize];
+    ["Offset of field: tally_summary::max"][::std::mem::offset_of!(tally_summary, max) - 24usize];
+    ["Offset of field: tally_summary::min"][::std::mem::offset_of!(tally_summary, min) - 32usize];
```

이 테스트는 헤더 말고도 세 가지에 반응합니다. 첫 줄에 bindgen의 버전이 적히므로 bindgen을 올리면 실패하고, allowlist를 빼면 libclang 버전에 따라 결과가 달라지며(2.3절), 플랫폼마다 숫자가 다릅니다. libclang에 `--target=i686-unknown-linux-gnu`를 주어 생성해 보니 두 구조체의 정렬이 8이 아니라 4로 나왔습니다. 이 저장소에서 커밋한 결과를 그 플랫폼에서 컴파일하면 레이아웃 검사가 걸릴 것입니다(타깃이 설치돼 있지 않아 확인하지 못함). 커밋한 결과는 플랫폼마다 따로 두거나, 검사가 걸리는 것을 "다시 생성하라"는 신호로 씁니다. CI에서 이 테스트를 어느 플랫폼에서 돌릴지는 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 다룹니다.

**따르지 않아도 되는 경우.** 빌드할 때만 생성한다면 비교할 스냅숏이 없으므로 이 검사가 필요 없습니다. 쓰는 쪽이 모두 libclang을 갖춘 사내 프로젝트처럼 빌드 의존성이 부담이 아니라면 그쪽이 더 단순합니다.

### 2.6 지침 5: C++ 라이브러리라면 `cxx`를 고려한다

**문제 상황.** bindgen은 C++ 헤더도 읽지만 일부만 옮깁니다. bindgen 문서의 C++ 절에 따르면 메서드·오버로딩·상속은 대체로 되고 생성자·소멸자는 바인딩은 생기되 자동으로 불리지 않으며, 인라인 함수와 템플릿 함수, 템플릿 특수화는 옮기지 못하고, bindgen 인터페이스를 넘는 예외는 미정의 동작입니다.

**비권장 코드.** C++ 클래스 API를 bindgen으로 옮겨 `-sys` 크레이트에 담고, 생성자 호출과 소멸, 예외를 래퍼가 손으로 맞추는 것입니다. 빠뜨린 소멸자 호출이나 넘어온 예외는 컴파일러가 잡지 못합니다.

**권장 코드.** 책은 더 나은(그래도 제한적인) 연동으로 `cxx` 크레이트를 권합니다. cxx.rs에 따르면 `#[cxx::bridge]` 모듈 하나에 Rust가 내보낼 항목(`extern "Rust"`)과 C++에서 가져올 항목(`unsafe extern "C++"`)을 적으면, 두 생성기가 그 선언에서 Rust 쪽과 C++ 쪽 코드를 함께 만들고 빌드 중에 확인되는 정적 단언을 더합니다. C++ 헤더에서 Rust를 만드는 bindgen과 달리, 공통 스키마에서 양쪽을 만드는 방식입니다. 이 챕터는 C++ 코드를 다루지 않아 실행해 보지 않았습니다(`cxx` 1.0.205, 2026-11-20 crates.io 기준).

책에 없는 도구 둘도 이름만 적습니다(버전은 2026-11-20 crates.io 기준, 실행하지 않음). `autocxx` 0.31.0은 기존 C++ 헤더를 bindgen으로 읽어 cxx의 다리로 옮겨 주고, `cbindgen` 0.29.4는 반대 방향으로 Rust 라이브러리의 C 인터페이스에서 C/C++ 헤더를 만듭니다. ch34의 `textstat_export`처럼 C에서 부를 Rust 함수를 내보낼 때 손으로 쓴 `textstat.h`를 대신할 수 있습니다.

**따르지 않아도 되는 경우.** C++ 라이브러리가 `extern "C"`로 감싼 C API를 따로 제공한다면, 그 헤더를 C처럼 bindgen으로 옮기는 것이 가장 단순합니다. ch34의 `tally.h`도 `#ifdef __cplusplus` 블록으로 C++에서 쓸 수 있게 되어 있습니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70, `rust-bindgen` 0.59.2의 생성 결과를 기준으로 합니다. 이 챕터에서 확인한 차이는 다음과 같습니다.

- **레이아웃 검사가 컴파일 때 평가됩니다.** 책의 각주는 기본 생성 결과에 레이아웃을 확인하는 `#[test]` 코드가 들어간다고 설명하고, bindgen 튜토리얼도 `cargo test`로 `bindgen_test_layout_*` 테스트를 돌리는 모습을 보여 줍니다. 받아 본 0.69.5도 그 모양이었지만, 0.73.2는 `offset_of!`(1.77)를 쓰는 `const _` 블록을 생성해 `cargo build`만으로 확인합니다(예제 02, 4.3절).
- **`unsafe extern` 블록.** 0.73.2의 기본 결과는 2024 에디션에서 그대로 컴파일되지만, 대상 판을 1.82 아래로 낮추거나 0.69.5를 쓰면 `extern "C" {`가 생성되어 컴파일되지 않습니다(2.3절).
- **생성 결과는 손으로 쓴 선언과 "같지" 않습니다.** 책은 같다고 하지만, 2024 에디션의 `safe fn`, `usize`로 고른 상수, enum 상수의 이름, 레이아웃 검사에서 달랐습니다(2.3절의 표). 함수 시그니처와 구조체 필드는 같았습니다.
- **CLI 대신 빌드 스크립트.** 책은 `bindgen` CLI로 `-o`에 적은 파일을 만들어 커밋하고 `include!`합니다. CLI는 설치하지 않았고, bindgen 튜토리얼대로 Builder를 빌드 스크립트에서 불러 `OUT_DIR/bindings.rs`를 `include!`했습니다. 책의 플래그는 `--allowlist-function`→`allowlist_function`, `--allowlist-type`→`allowlist_type`, `--no-layout-tests`→`layout_tests(false)`, `-o`→`write_to_file`에 해당합니다.
- **책에 없는 지침.** 다음은 책의 Item 35가 명시적으로 권하지 않는 내용입니다(2026-11-20에 온라인판에서 확인). 책은 손으로 쓴 선언의 위험, bindgen의 CLI와 빌드 스크립트 실행, `include!`, 사소한 것을 넘는 선언의 생성, CI에서의 비교, allowlist·blocklist, `-sys`와 래퍼의 층 나누기, C++과 `cxx`, `--no-layout-tests` 각주를 다룹니다.
  - 2.1절의 미정의 동작과 잘못 해석한 값의 구분, 레이아웃 검사가 보는 범위: 표준 라이브러리 `fn` 문서의 ABI compatibility 절과 예제 01·02가 근거입니다.
  - 2.2절의 "bindgen이 옮기지 못하는 부분"(함수 모양 매크로, `static inline`)과 테스트로 일부를 잡는 대안: 직접 생성해 본 결과와 v2에서 실패한 테스트가 근거입니다.
  - 2.3절의 allowlist가 결과를 환경에 덜 묶는다는 점, 옵션 표, 2024 에디션 확인, 생성 코드에는 `#[allow]`: bindgen 0.73.2의 소스와 Builder 문서, 두 libclang과 0.69.5로 생성해 본 결과가 근거입니다.
  - 2.4절의 `links` 거부, `DEP_<이름>_<키>`, 빌드 스크립트 바꿔 끼우기: Cargo Book의 Build Scripts 장(`*-sys` Packages, The `links` Manifest Key, Overriding Build Scripts)이 근거입니다(책은 `-sys` 크레이트와 래퍼 크레이트의 층 나누기만 말함).
  - 2.5절의 득실 표, 기능으로 두 방식을 고르는 구성, 덮어쓰기(`TALLY_BLESS`), 플랫폼·libclang에 따라 달라지는 결과: 이 챕터의 프로젝트에서 확인한 것입니다.
  - 2.6절의 `autocxx`와 `cbindgen`: 각 프로젝트의 문서가 근거입니다.

### 2.8 기억할 것

- 손으로 쓴 선언은 헤더가 바뀌어도 조용히 컴파일됩니다. 상수와 필드 이름의 어긋남은 틀린 값으로, 인자 수의 어긋남은 미정의 동작으로 나타납니다. 레이아웃 검사는 그 일부만 봅니다.
- 사소한 것을 넘는 선언은 C 컴파일러와 같은 원본(헤더)에서 생성합니다. 생성 범위는 라이브러리의 접두사로 좁히고, 생성 결과를 한 번은 읽어 래퍼가 바뀔 곳을 확인합니다.
- `-sys` 크레이트는 링크와 생성한 선언만 맡고(`links`는 라이브러리당 하나), 안전한 API와 `safe` 판단은 그 위 크레이트가 맡습니다. 생성 코드의 린트는 `#[allow]`로 끕니다.
- 생성 결과를 커밋해 두면 쓰는 쪽에 libclang이 필요 없지만 낡을 수 있습니다. 새로 생성한 결과와 비교하는 검사를 함께 둡니다.
- C++이면 bindgen의 한계를 확인하고 `cxx`를 검토합니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행하고, C 코드나 외부 크레이트 없이 종료 코드 0으로 끝납니다. 숫자는 64비트 플랫폼 기준입니다. `projects/tally_bindings`의 두 워크스페이스는 각자의 디렉터리에서 실행합니다.

| 파일 | 지침 | 보여 주는 것 | 관련 절 |
| --- | --- | --- | --- |
| [`01_layout_checks.rs`](examples/01_layout_checks.rs) | 1 | 어긋남 다섯 가지와 레이아웃 검사가 잡는 것·놓치는 것 | 2.1 |
| [`02_const_layout_asserts.rs`](examples/02_const_layout_asserts.rs) | 1, 2 | bindgen이 생성하는 `const` 검사와 손으로 쓰는 `assert!` 모양 | 2.1, 2.3 |
| [`03_compare_declarations.rs`](examples/03_compare_declarations.rs) | 2 | 손으로 쓴 선언과 생성한 선언을 항목별로 대조 | 2.3 |
| [`projects/tally_bindings`](projects/tally_bindings/demo/drift.rs) | 1~4 | C 세 판 × 손으로 쓴 선언·커밋한 결과·빌드 때 생성 | 2.2~2.5 |

### 01_layout_checks — 레이아웃 검사가 보는 범위

```sh
cargo run -p er-ch35-bindgen --example 01_layout_checks
```

```text
1. 같은 타입의 두 필드가 자리를 바꿈 (min과 max)
   헤더: 크기 40, 정렬 8, 오프셋 count=0 below=8 above=16 max=24 min=32
   낡음: 크기 40, 정렬 8, 오프셋 count=0 below=8 above=16 min=24 max=32
   → 실패하는 검사: 오프셋(max, min) — 잡힘
2. 크기가 같은 다른 타입으로 바뀜 (u32 → f32)
   헤더: 크기 16, 정렬 8, 오프셋 low=0 bucket_width=8 bucket_count=12
   낡음: 크기 16, 정렬 8, 오프셋 low=0 bucket_width=8 bucket_count=12
   → 크기·정렬·오프셋 검사를 모두 통과: 놓침
3. 끝 필드가 좁아졌지만 패딩에 가려짐 (u32 → u16)
   헤더: 크기 16, 정렬 8, 오프셋 low=0 bucket_width=8 bucket_count=12
   낡음: 크기 16, 정렬 8, 오프셋 low=0 bucket_width=8 bucket_count=12
   → 크기·정렬·오프셋 검사를 모두 통과: 놓침
4. 끝에 필드가 더해짐 (total 추가)
   헤더: 크기 48, 정렬 8, 오프셋 count=0 below=8 above=16 min=24 max=32 total=40
   낡음: 크기 40, 정렬 8, 오프셋 count=0 below=8 above=16 min=24 max=32
   → 실패하는 검사: 크기 — 잡힘
5. 앞쪽 필드가 넓어짐 (u32 → u64)
   헤더: 크기 24, 정렬 8, 오프셋 low=0 bucket_width=8 bucket_count=16
   낡음: 크기 16, 정렬 8, 오프셋 low=0 bucket_width=8 bucket_count=12
   → 실패하는 검사: 크기, 오프셋(bucket_count) — 잡힘
레이아웃 검사가 보는 것: 크기, 정렬, 이름으로 찾은 오프셋
보지 않는 것: 필드의 타입과 폭, #define·enum 상수의 값, 함수 시그니처
```

### 02_const_layout_asserts — 컴파일 때 평가되는 레이아웃 검사

```sh
cargo run -p er-ch35-bindgen --example 02_const_layout_asserts
```

```text
tally_config의 레이아웃 검사 (모두 컴파일할 때 통과함)
   크기: Rust 16, 헤더 16
   정렬: Rust 8, 헤더 8
   low의 오프셋: Rust 0, 헤더 0
   bucket_width의 오프셋: Rust 8, 헤더 8
   bucket_count의 오프셋: Rust 12, 헤더 12
   값: low=-5 bucket_width=10 bucket_count=4
검사가 확인하는 것: Rust의 #[repr(C)] 배치가 생성할 때 잰 C의 배치와 같은가
확인하지 않는 것: 그 숫자를 잰 뒤에 헤더가 바뀌었는가
```

파일 안의 첫 `const _` 블록은 bindgen 0.73.2의 생성 결과를 옮긴 것입니다. 값이 같으면 `크기 - 16`이 0이 되어 원소 하나짜리 배열의 0번을 읽고, 다르면 범위 밖 인덱스나 뺄셈 오버플로가 되어 상수 평가가 실패합니다. 둘째 블록은 손으로 쓸 때 읽기 쉬운 `assert!` 모양이고(상수 평가 중의 패닉은 1.57부터), 실패하면 `error[E0080]: evaluation panicked: <메시지>`가 납니다(사본으로 확인).

### 03_compare_declarations — 손으로 쓴 선언과 생성한 선언 대조

```sh
cargo run -p er-ch35-bindgen --example 03_compare_declarations
```

```text
1. 함수 (손으로 쓴 판 9개, 생성판 9개)
   tally_default_config   시그니처 같음, 손: safe fn   생성: 표시 없음
   tally_new              시그니처 같음, 손: safe fn   생성: 표시 없음
   tally_free             시그니처 같음, 손: unsafe fn 생성: 표시 없음
   …(tally_record부터 tally_string_free까지 여섯 줄이 같은 모양)
2. 상수 (손으로 쓴 판 5개)
   손: TALLY_MAX_BUCKETS: u32 = 32
       생성: TALLY_MAX_BUCKETS: u32 = 32
   손: TALLY_NAME_MAX: usize = 31
       생성: TALLY_NAME_MAX: u32 = 31
   손: TALLY_OK: i32 = 0
       생성: tally_status_TALLY_OK: tally_status = 0
   …(TALLY_ERR_NULL, TALLY_ERR_NAME_TOO_LONG도 같은 모양)
3. 구조체
   tally_config: 필드 같음 (pub low: i64, pub bucket_width: u32, pub bucket_count: u32)
       derive 손: [Debug, Clone, Copy] 생성: [Debug, Copy, Clone]
   tally_summary: 필드 같음 (pub count: u64, pub below: u64, pub above: u64, pub min: i64, pub max: i64)
       derive 손: [Debug, Clone, Copy] 생성: [Debug, Copy, Clone]
   tally: 필드 다름
       손: _opaque: [u8; 0]
       생성: _unused: [u8; 0]
       derive 손: [] 생성: [Debug]
4. 생성판에만 있는 것
   pub type tally_status = c_int;
   레이아웃 검사: 크기 2개, 정렬 2개, 필드 오프셋 8개
   extern 블록: 손으로 쓴 판 1개, 생성판 9개
5. 그 위의 래퍼
   unsafe 블록: 손으로 쓴 판 위 9개, 생성판 위 11개
   생성판 래퍼에서 [생성판] 표시를 단 곳: 6곳
```

출력에서 줄인 두 곳은 실제로는 여섯 줄과 네 줄입니다. 예제는 [ch25](../ch25-dependency-graph/README.md)의 예제가 `Cargo.lock`을 읽은 것처럼 `projects/`의 파일을 `include_str!`로 읽으므로, 그 파일을 고치면 출력도 바뀝니다. 함수마다 extern 블록이 따로인 것은 `merge_extern_blocks(true)`로 합칠 수 있습니다.

### projects/tally_bindings — C 세 판과 선언 세 가지

`csrc/v1`~`v3`이 C 라이브러리의 세 판이고, [`demo/drift.rs`](projects/tally_bindings/demo/drift.rs)는 두 워크스페이스가 함께 쓰는 데모 프로그램입니다(`[[bin]] path`로 같은 파일을 빌드). 두 래퍼는 이름(`tally`)과 공개 API가 같아 같은 데모가 양쪽에서 컴파일됩니다.

```sh
cd appd-a/ch35-bindgen/projects/tally_bindings/handwritten
cargo run -q                       # 손으로 쓴 선언, C v1
TALLY_C_VERSION=2 cargo run -q     # 손으로 쓴 선언, C v2
```

```text
[C 소스 v1 · 손으로 쓴 선언]
1. 헤더의 #define을 옮긴 상수
   MAX_BUCKETS = 32, NAME_MAX = 31
2. C가 값으로 돌려준 요약 (표본 12개)
   Rust가 읽은 값: Summary { count: 12, below: 1, above: 2, range: Some((-3, 250)) }
   C가 쓴 표 머리: [tally] n=12 below=1 above=2
3. 이름을 NAME_MAX 바이트에 맞춰 잘라 넣기
   "응답 시간(밀리초, 하루" (31바이트) → Ok(())
4. C의 상태 코드를 Result로
   빈 이름: Ok(())
   60바이트 이름: Err(NameTooLong { len: 60, max: 31 })
   Display: 이름이 60바이트로 최대 31바이트를 넘습니다
```

```text
[C 소스 v2 · 손으로 쓴 선언]
1. 헤더의 #define을 옮긴 상수
   MAX_BUCKETS = 32, NAME_MAX = 31
2. C가 값으로 돌려준 요약 (표본 12개)
   Rust가 읽은 값: Summary { count: 12, below: 1, above: 2, range: Some((250, -3)) }
   C가 쓴 표 머리: [tally] n=12 below=1 above=2 min=-3 max=250
3. 이름을 NAME_MAX 바이트에 맞춰 잘라 넣기
   "응답 시간(밀리초, 하루" (31바이트) → Err(Status(-3))
4. C의 상태 코드를 Result로
   빈 이름: Err(NameTooLong { len: 0, max: 31 })
   60바이트 이름: Err(Status(-3))
   Display: 래퍼가 모르는 상태 코드 -3
```

`TALLY_C_VERSION=3 cargo build`는 경고 없이 끝나고, `TALLY_C_VERSION=3 cargo run -q`는 데모가 `선언이 v3 헤더와 맞지 않습니다(tally_record의 인자 수가 다름).`와 `실행하면 미정의 동작이므로 여기서 멈춥니다.`를 표준 에러에 출력하고 종료 코드 2로 멈춥니다. `cargo test`는 단위 테스트 8개와 문서 테스트 1개를 통과합니다(`TALLY_C_VERSION=2`이면 단위 테스트 2개가 실패).

```sh
cd ../generated
cargo run -q -p tally                                        # 커밋해 둔 생성 결과
cargo run -q -p tally --features tally-sys/bindgen           # 빌드 때 생성(libclang 필요)
TALLY_C_VERSION=2 cargo run -q -p tally --features tally-sys/bindgen
```

v1에서는 두 명령 모두 첫 줄의 꼬리표(`커밋해 둔 생성 결과`, `빌드 때 생성한 선언`)만 다르고 나머지는 손으로 쓴 판의 v1 출력과 같습니다. `TALLY_C_VERSION=2 cargo run -q -p tally`(커밋해 둔 결과)는 꼬리표 말고는 손으로 쓴 판의 v2 출력과 같습니다. 셋째 명령의 출력입니다.

```text
[C 소스 v2 · 빌드 때 생성한 선언]
1. 헤더의 #define을 옮긴 상수
   MAX_BUCKETS = 32, NAME_MAX = 15
2. C가 값으로 돌려준 요약 (표본 12개)
   Rust가 읽은 값: Summary { count: 12, below: 1, above: 2, range: Some((-3, 250)) }
   C가 쓴 표 머리: [tally] n=12 below=1 above=2 min=-3 max=250
3. 이름을 NAME_MAX 바이트에 맞춰 잘라 넣기
   "응답 시간(" (14바이트) → Ok(())
4. C의 상태 코드를 Result로
   빈 이름: Err(Status(-2))
   60바이트 이름: Err(NameTooLong { len: 60, max: 15 })
   Display: 이름이 60바이트로 최대 15바이트를 넘습니다
```

`TALLY_C_VERSION=3 cargo build -p tally --features tally-sys/bindgen`은 2.2절의 E0061로 실패하고(종료 코드 101), `TALLY_C_VERSION=3 cargo run -q -p tally`(커밋해 둔 결과)는 손으로 쓴 판처럼 조용히 빌드된 뒤 데모가 종료 코드 2로 멈춥니다. `cargo test`는 `tally`의 단위 테스트 8개와 문서 테스트 1개를, `cargo test --features tally-sys/bindgen`은 여기에 `tally-sys`의 `freshness` 테스트 1개를 더 통과합니다. 첫 빌드에서 `bindgen` 기능을 켜면 의존성 21개를 더 컴파일합니다.

## 4. 자주 하는 실수

`compile_fail/`의 세 파일은 생성한 선언이나 레이아웃 검사를 한 파일 안의 모듈로 흉내 냈습니다. 각 파일 머리의 `rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/<파일>`로 확인합니다.

### 4.1 생성한 `#define` 상수를 손으로 쓸 때의 타입으로 씀 — E0308

[`compile_fail/e0308_define_is_u32.rs`](compile_fail/e0308_define_is_u32.rs)는 손으로 쓴 판의 래퍼 줄 `pub const NAME_MAX: usize = ffi::TALLY_NAME_MAX;`를 생성한 상수 위로 그대로 옮겼습니다.

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_define_is_u32.rs:18:29
   |
18 | pub const NAME_MAX: usize = ffi::TALLY_NAME_MAX;
   |                     -----   ^^^^^^^^^^^^^^^^^^^ expected `usize`, found `u32`
   |                     |
   |                     expected because of the type of the constant
```

**원인.** bindgen은 `#define`의 정수를 기본으로 `u32`로 옮깁니다(시험해 보니 음수는 `i32`, `u32`를 넘는 값은 `u64`). 손으로 쓸 때 사람이 고른 `usize`는 헤더에 없는 정보입니다.

**해결.** 래퍼에서 `ffi::TALLY_NAME_MAX as usize`로 바꿉니다(`u32`에서 `usize`로의 변환은 이 저장소가 다루는 32·64비트 플랫폼에서 값을 잃지 않음). 매크로가 많다면 `ParseCallbacks::int_macro`로 생성할 타입을 고릅니다.

### 4.2 다시 생성한 선언이 래퍼를 멈춤 — E0061

[`compile_fail/e0061_regenerated_signature.rs`](compile_fail/e0061_regenerated_signature.rs)는 v3 헤더에서 생성한 `tally_record` 선언과 v2까지의 래퍼 메서드를 한 파일에 담았습니다.

```text
error[E0061]: this function takes 3 arguments but 2 arguments were supplied
  --> compile_fail/e0061_regenerated_signature.rs:26:14
   |
26 |     unsafe { ffi::tally_record(raw, sample) }
   |              ^^^^^^^^^^^^^^^^^------------- argument #3 of type `u32` is missing
   |
note: function defined here
  --> compile_fail/e0061_regenerated_signature.rs:19:16
   |
19 |         pub fn tally_record(t: *mut tally, sample: i64, weight: u32) -> i32;
   |                ^^^^^^^^^^^^                             ------
help: provide the argument
```

**원인.** 이 에러는 실수라기보다 bindgen이 해 주는 일입니다. C의 시그니처가 바뀌었다는 사실이 컴파일 에러로 바뀌었습니다. 같은 변화가 손으로 쓴 선언에서는 아무 신호 없이 미정의 동작이 됩니다.

**해결.** help가 제안하는 `/* u32 */` 자리에 아무 값이나 넣지 말고, 새 인자의 뜻을 헤더에서 읽고 래퍼의 API를 정합니다(연습 2).

### 4.3 낡은 정의를 새 헤더의 레이아웃 검사로 확인 — E0080

[`compile_fail/e0080_layout_assert_stale.rs`](compile_fail/e0080_layout_assert_stale.rs)는 v1에 맞춘 `tally_summary` 정의에 bindgen이 v2 헤더에서 생성한 레이아웃 검사를 붙였습니다.

```text
error[E0080]: index out of bounds: the length is 1 but the index is 8
  --> compile_fail/e0080_layout_assert_stale.rs:33:5
   |
33 |     ["Offset of field: tally_summary::max"][::std::mem::offset_of!(tally_summary, max) - 24usize];
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ evaluation of `_` failed here
```

**읽는 법.** 배열의 문자열이 어느 검사인지 알려 주고, 인덱스 8은 실제 오프셋 32와 헤더의 24의 차이입니다. 값이 헤더보다 작은 경우는 `attempt to compute … which would overflow`로 나옵니다.

**원인과 해결.** 숫자와 정의의 출처가 다릅니다. 정의를 같은 헤더에서 다시 생성하면 사라집니다. 반대로 정의와 검사를 함께 커밋해 두고 둘 다 낡으면 이 에러조차 나지 않으므로, 2.5절의 비교 검사가 필요합니다.

### 4.4 그 밖의 실수

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| libclang을 찾지 못함 | 빌드 스크립트가 ``set the `LIBCLANG_PATH` environment variable…``로 실패 | libclang 설치, `LIBCLANG_PATH` | 준비 사항 |
| allowlist 없이 생성 | 79줄이 271줄, 플랫폼 내부 매크로, libclang마다 다른 결과 | 접두사로 `allowlist_*` | 2.3 |
| 대상 판 1.82 미만, 옛 bindgen | 2024 에디션에서 `extern blocks must be unsafe` | 기본 대상 판, 새 bindgen | 2.3 |
| 생성 코드에 `#[expect(…)]` | 생성 결과가 바뀌면 `unfulfilled_lint_expectations` | `#[allow(…)]` | 2.3 |
| C가 돌려주는 값을 `rustified_enum`으로 받음 | 목록 밖의 값이면 미정의 동작 | 상수나 뉴타입 표현 | 2.3, ch34 2.4 |
| 생성한 선언의 함수를 그냥 부름 | E0133(생성 결과에는 `safe` 표시가 없음) | `unsafe` 블록과 안전성 주석 | Part 2 ch4 4.8절 |
| 같은 라이브러리를 링크하는 패키지가 둘 | ``more than one crate with links=tally`` | `-sys` 크레이트 하나에 의존 | 2.4 |
| 생성 결과를 커밋하고 검사하지 않음 | 헤더가 바뀌면 손으로 쓴 판처럼 틀린 값 | 비교 테스트, CI | 2.5 |

## 5. 연습 문제

### 연습 1. 생성판 래퍼를 v2로 옮기기

`generated/`를 C v2에 맞춰 옮기세요. 커밋해 둔 결과를 v2로 갱신하고, 빈 이름을 뜻하는 `TallyError::EmptyName`을 더해 데모의 "빈 이름" 줄이 `Err(EmptyName)`이 되게 합니다. 순서는 어떻게 되고, 갱신한 뒤 v1로 빌드하면 무슨 일이 일어나나요?

<details>
<summary>힌트</summary>

`TALLY_C_VERSION=2 TALLY_BLESS=1 cargo test -p tally-sys --features bindgen`으로 `committed_bindings.rs`를 갱신하고 `git diff`로 2.5절의 diff를 확인합니다. 래퍼의 `set_name`에 `ffi::tally_status_TALLY_ERR_EMPTY_NAME => Err(TallyError::EmptyName)` 갈래를, `Display`와 `status_name`에도 한 줄씩 더합니다. 이제 C v1로 빌드하면(`cargo build -p tally --features tally-sys/bindgen`) 그 상수가 생성되지 않아 `match` 패턴에서 E0531(`cannot find unit struct, unit variant or constant`)로 실패하고, v1 소스와 v2 결과를 함께 쓰는 기본 빌드(`cargo run -q -p tally`)는 `range`가 뒤바뀌고 60바이트 이름을 `Err(EmptyName)`으로 보고하는 틀린 값을 냅니다. 래퍼는 한 판의 헤더에 맞춰 쓰는 것이고, 빌드 스크립트가 C 소스의 판을 고정해야 합니다.

</details>

### 연습 2. v3의 `weight` 인자 받아들이기

`TALLY_C_VERSION=3`에서 `generated/`가 빌드되도록 래퍼를 고치세요. 기존 `record(sample)`은 그대로 두고 `record_weighted(sample, weight)`를 더합니다. 데모의 v3 안전장치는 이 빌드를 막나요?

<details>
<summary>힌트</summary>

`record`는 `ffi::tally_record(self.raw.as_ptr(), sample, 1)`을, `record_weighted`는 받은 `weight`(`u32`)를 넘깁니다. 헤더에 따르면 `weight`가 0이면 아무것도 기록하지 않으므로, 그 사실을 문서 주석에 옮깁니다. 데모는 `DECLARATIONS`가 `Generated`인 빌드는 멈추지 않으므로 `TALLY_C_VERSION=3 cargo run -q -p tally --features tally-sys/bindgen`이 실행됩니다. 첫 줄(`[C 소스 v3 · 빌드 때 생성한 선언]`) 말고는 v2를 빌드 때 생성한 출력과 같습니다(`record`가 `weight` 1로 부름). 커밋해 둔 결과가 v1인 기본 빌드는 이제 E0061로 실패합니다. 갱신하지 않은 스냅숏이 이번에는 컴파일 에러로 드러나는 것입니다.

</details>

### 연습 3. enum 표현 바꾸기

`tally-sys/build.rs`의 Builder에 `.constified_enum_module("tally_status")`를 더하고 `--features bindgen`으로 빌드하세요. 래퍼의 어느 줄이 바뀌어야 하나요? `tally-sys`의 `#![allow(…)]`를 `#![expect(…)]`로 바꾸면 어떻게 되나요? `freshness` 테스트는요?

<details>
<summary>힌트</summary>

상수가 `pub mod tally_status { pub type Type = c_int; pub const TALLY_OK: Type = 0; … }`로 바뀌므로 래퍼의 `ffi::tally_status_TALLY_OK`를 모두 `ffi::tally_status::TALLY_OK`로 고칩니다(`match` 패턴에도 경로를 쓸 수 있음). 이 표현에서는 Rust 이름 규칙에 어긋나는 이름이 생성되지 않으므로, `#![expect(non_camel_case_types, non_upper_case_globals)]`는 두 린트 모두 `this lint expectation is unfulfilled`를 냅니다. 커밋해 둔 결과와도 달라지므로 `freshness` 테스트가 실패하고, 옵션을 바꾼 것도 덮어쓰고 커밋할 일이 됩니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 35: Prefer `bindgen` to manual FFI mappings](https://effective-rust.com/bindgen.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- The bindgen User Guide: [Requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)(libclang 설치), 튜토리얼의 [build.rs](https://rust-lang.github.io/rust-bindgen/tutorial-3.html)와 [`include!`](https://rust-lang.github.io/rust-bindgen/tutorial-4.html), [Allowlisting](https://rust-lang.github.io/rust-bindgen/allowlisting.html), [Generating Bindings to C++](https://rust-lang.github.io/rust-bindgen/cpp.html)
- `bindgen` 0.73.2의 Builder 문서와 소스(`~/.cargo/registry` 아래 `bindgen-0.73.2/options/mod.rs`, `features.rs`): 옵션의 기본값, `unsafe extern` 블록을 생성하는 대상 판
- The Cargo Book, [Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html): `*-sys` Packages, The `links` Manifest Key, `cargo::metadata`와 `DEP_<links>_<KEY>`, Overriding Build Scripts
- 표준 라이브러리 문서, [`fn` 타입의 ABI compatibility](https://doc.rust-lang.org/std/primitive.fn.html#abi-compatibility): 인자 수와 타입이 맞아야 하는 이유
- [CXX](https://cxx.rs/): `#[cxx::bridge]`, `extern "Rust"`, `unsafe extern "C++"`
- [`autocxx` 문서](https://docs.rs/autocxx/latest/autocxx/), [`cbindgen`](https://github.com/mozilla/cbindgen) README
- 로컬 문서의 릴리스 노트(`rustc --print sysroot` 아래 `share/doc/rust/html/releases.md`): 1.57.0(상수 평가 중의 패닉), 1.77.0(`mem::offset_of!`), 1.82.0(`unsafe extern`)

## 부록 A를 마치며

부록 A는 이미 아는 도구 사이에서 고르는 기준을 책의 여섯 묶음을 따라 정리했습니다. Types(ch1~ch9)에서는 타입으로 뜻과 불변식을 드러내는 법을, Traits(ch10~ch13)에서는 표준 트레이트와 제네릭·트레이트 객체를 고르는 기준을 다뤘습니다. Concepts(ch14~ch20)에서는 라이프타임과 대여 검사기, `unsafe`, 공유 상태, 패닉, 리플렉션, 최적화를 언제 어디까지 쓸지 따졌습니다. Dependencies(ch21~ch26)는 시맨틱 버전과 가시성, 의존성 그래프와 기능이 쓰는 쪽에 미치는 영향을, Tooling(ch27~ch32)은 문서, 매크로, Clippy, 테스트, 도구 생태계, CI로 판단을 자동화하는 법을 다뤘습니다. 마지막 Beyond Standard Rust(ch33~ch35)는 `no_std`, FFI 경계, 이 챕터의 생성한 선언처럼 표준 라이브러리 바깥과 만나는 곳을 다뤘습니다.

여기서 이름만 짚고 넘어간 내용은 본편에서 이어집니다. 빌드 스크립트와 기능, 워크스페이스는 Part 3 `part3-cargo`에서, 문서와 문서 테스트는 Part 4 `part4-rustdoc`에서, 린트와 조건부 컴파일은 Part 5 `part5-rustc`에서, `no_std` 타깃의 실제 빌드는 Part 8 `part8-embedded`에서 다룹니다. 언어의 규칙 자체는 Part 9 `part9-reference`가, 레이아웃과 FFI처럼 `unsafe` 아래의 세계는 Part 10 `part10-nomicon`이 맡습니다. 전체 순서는 [로드맵](../../docs/roadmap.md)에서 확인할 수 있습니다.
