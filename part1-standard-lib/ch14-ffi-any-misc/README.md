# ch14. FFI 문자열, `Any`, 수치 래퍼: `CString`/`CStr`, `OsString`, 다운캐스트, `NonZero`/`Wrapping`

Part 1의 마지막 챕터입니다. 서로 다른 세 주제를 다루지만, 모두 앞 챕터들이 "나중에 다룬다"고 미뤄 둔 것입니다.

- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md): `c"hello"` 리터럴과 `strlen`으로 맛만 본 **C 문자열 타입**
- [ch3](../ch3-strings-fmt/README.md)에서 다룬 `String`/`str` 밖에 있는 문자열 타입 가족의 나머지. [ch9](../ch9-io-fs-path/README.md)는 `Path`가 `OsStr` 위에 만들어졌다고 했고, [ch10](../ch10-env-process-time/README.md)은 `args_os`/`var_os`가 돌려주는 `OsString`의 메서드 세 개만 쓰고 타입 자체를 남겨 두었습니다.
- [ch8](../ch8-errors-panic/README.md): 패닉 페이로드에서 `downcast_ref::<&str>()`를 쓴 원리, 즉 **`Any`와 다운캐스트 일반**. [ch5](../ch5-iterators/README.md)가 한 문장으로 소개한 `std::any::type_name`
- [ch1](../ch1-std-overview/README.md): `checked_*`·`wrapping_*`·`saturating_*` 메서드로 고르던 오버플로 처리를 **타입으로** 고르는 `Wrapping`/`Saturating`. [ch7](../ch7-smart-pointers-memory/README.md)이 니치 최적화에서 이름만 남긴 `NonZero`

그래서 이 챕터는 세 부로 나뉩니다. **문자열 타입 가족**(2.2~2.5절), **타입을 실행 중에 묻기**(2.6~2.8절), **수치 래퍼**(2.9~2.11절)입니다.

- 패키지: `p1-ch14-ffi-any-misc`
- 기준 문서: [`std::ffi`](https://doc.rust-lang.org/std/ffi/index.html), [`std::any`](https://doc.rust-lang.org/std/any/index.html), [`std::num`](https://doc.rust-lang.org/std/num/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024), macOS (aarch64). 예제 02의 3·5번 구역은 유닉스 전용이고, 예제 04의 C 타입 별칭은 플랫폼마다 다릅니다.
- unsafe 코드는 예제 04에만 있습니다. 이 환경에는 nightly와 Miri가 없어 그 코드를 Miri로 검사하지 못했습니다([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.7절).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 문자열 타입 네 쌍(`String`/`str`, `OsString`/`OsStr`, `CString`/`CStr`, `PathBuf`/`Path`)이 각각 무엇을 보장하는지 설명하고, 서로 바꿀 때 어느 방향이 공짜이고 어느 방향이 실패하거나 손실이 생기는지 말할 수 있다.
2. `OsString`을 `to_str`·`to_string_lossy`·`into_string`으로 다루고, 유닉스에서 UTF-8이 아닌 값을 직접 만들어 세 방법의 결과 차이를 설명할 수 있다.
3. `CString::new`, `c"…"` 리터럴, `CStr::from_bytes_with_nul`/`from_bytes_until_nul`로 C 문자열을 만들고 `NulError`를 처리할 수 있다. C 함수 호출을 `&CStr`을 받는 안전한 함수로 감싸고, 임시 `CString`의 `as_ptr()`가 왜 위험한지 설명할 수 있다.
4. `TypeId`, `type_name`, `&dyn Any`의 `is`/`downcast_ref`/`downcast_mut`, `Box<dyn Any>`의 `downcast`를 쓸 수 있다. `'static`이 필요한 이유와 "상자 자체도 `Any`" 함정을 설명하고, `Any`와 열거형·트레이트 메서드 중 무엇을 쓸지 고를 수 있다.
5. 오버플로 처리를 메서드와 `Wrapping`/`Saturating` 타입 중 어느 쪽으로 고를지 판단할 수 있다. `NonZero`로 0을 타입에서 빼서 니치와 패닉 없는 나눗셈을 얻고, `ParseIntError::kind()`로 파싱 실패 이유를 구분할 수 있다.

## 2. 핵심 개념

### 2.1 이 챕터의 지도

| 부 | 절 | 타입 | 예제 |
| --- | --- | --- | --- |
| 문자열 타입 가족 | 2.2~2.5 | `OsString`/`OsStr`, `CString`/`CStr`, `c_char` 등 C 타입 별칭 | 01~04 |
| 타입을 실행 중에 묻기 | 2.6~2.8 | `Any`, `TypeId`, `type_name` | 05, 06 |
| 수치 래퍼 | 2.9~2.11 | `Wrapping`, `Saturating`, `NonZero`, `ParseIntError`/`IntErrorKind` | 07, 08 |

세 부는 서로 독립적이므로 어느 부를 먼저 읽어도 됩니다. 다만 2.4절의 `CString::from(Vec<NonZero<u8>>)`처럼 부 사이를 잇는 곳이 몇 군데 있습니다.

### 2.2 문자열 타입 가족의 지도

Rust 코드 안의 텍스트는 모두 `String`/`str`입니다. 그런데 프로그램 바깥(운영체제, C 라이브러리)은 Rust와 다른 규칙으로 문자열을 다룹니다. 표준 라이브러리는 바깥의 규칙마다 타입을 하나씩 두어, 경계를 넘을 때 무엇을 확인해야 하는지 타입으로 드러냅니다.

| 소유 / 빌림 | 보장하는 것 | 만나는 곳 |
| --- | --- | --- |
| `String` / `str` | 언제나 올바른 UTF-8. 길이를 따로 저장하므로 중간에 0 바이트가 있어도 됨 | Rust 코드의 모든 텍스트 |
| `OsString` / `OsStr` | 운영체제가 준 문자열을 **손실 없이** 담음. UTF-8이라는 보장은 없음 | `env::args_os`, `env::var_os`, 파일 이름 |
| `CString` / `CStr` | 끝에 0 바이트가 하나 있고 중간에는 없음. 인코딩은 보장하지 않음 | C 함수에 넘기거나 C 함수가 돌려준 문자열 |
| `PathBuf` / `Path` | `OsString`/`OsStr` 위에 경로 연산을 얹은 것 | 파일 시스템 API ([ch9](../ch9-io-fs-path/README.md)) |

**Rust 문자열과 C 문자열의 차이.** `std::ffi` 문서는 둘의 차이를 이렇게 정리합니다. Rust 문자열은 길이를 저장하므로 길이를 아는 데 O(1)이고, 중간에 `\0`이 있어도 문자열의 일부일 뿐입니다. C 문자열은 길이를 저장하지 않고 **0 바이트(NUL 종료 바이트)로 끝을 표시**합니다. 그래서 길이를 알려면 `strlen`이 0 바이트를 찾을 때까지 훑어야 하고(O(n)), 중간에 0 바이트가 있으면 C는 거기서 문자열이 끝났다고 읽습니다. 또 C 문자열은 UTF-8이라는 보장이 없습니다.

**소유한 것과 빌린 것은 짝을 이룹니다.** `String`이 역참조로 `str`이 되듯 `OsString`은 `OsStr`로, `CString`은 `CStr`로, `PathBuf`는 `Path`로 역참조됩니다. 그래서 함수는 빌린 타입(`&str`, `&OsStr`, `&CStr`, `&Path`)으로 받고, 소유한 값이 필요하면 `to_owned()`로 되돌립니다([`01_string_family.rs`](examples/01_string_family.rs)의 2번 구역).

**서로 바꾸기.** 규칙은 한 줄로 요약됩니다. **UTF-8에서 출발하면 공짜이고, UTF-8로 돌아오면 검사가 필요합니다.** 올바른 UTF-8은 운영체제 문자열로도 경로로도 언제나 올바르기 때문입니다. 예외는 C 문자열로 가는 길로, 0 바이트 규칙 때문에 검사와 복사가 필요합니다.

| 출발 → 도착 | 방법 | 실패하거나 손실이 생기는 경우 | 복사 |
| --- | --- | --- | --- |
| `&str` → `&OsStr`, `&Path` | `OsStr::new(s)`, `Path::new(s)` | 없음 | 없음 |
| `&OsStr` ↔ `&Path` | `Path::new(o)`, `p.as_os_str()` | 없음 | 없음 |
| `&str`, `String`, `Vec<u8>` → `CString` | `CString::new(x)` | 중간에 0 바이트가 있으면 `Err(NulError)` | 끝에 0 바이트를 붙일 버퍼 |
| `&OsStr`, `&Path` → `&str` | `to_str()` | UTF-8이 아니면 `None` | 없음 |
| `&CStr` → `&str` | `to_str()` | UTF-8이 아니면 `Err(Utf8Error)` | 없음 |
| `&OsStr`, `&Path`, `&CStr` → `Cow<str>` | `to_string_lossy()` | 깨진 바이트를 `�`(U+FFFD)로 바꿈(손실) | UTF-8이면 없음 |
| `OsString` → `String` | `into_string()` | `Err(원래 OsString)` | 없음 |
| `CString` → `String` | `into_string()` | `Err(IntoStringError)`, `into_cstring()`으로 되찾음 | 없음 |
| `OsStr` ↔ `CStr` | 직접 바꾸는 메서드 없음 | 유닉스는 바이트를 거침(2.3절), 이식성이 필요하면 `to_str()` 후 `CString::new` | 있음 |

예제 01의 3번 구역은 `OsStr::new(s)`와 `Path::new(s)`가 `s`와 **같은 주소의 바이트**를 가리킨다는 것을 비교로 확인합니다. 타입만 바꿔 볼 뿐 복사하지 않는다는 뜻입니다.

**출력하기.** 기본 형식(`{}`)을 구현한 것은 `String`/`str`뿐입니다. `Path`와 `OsStr`은 `display()`로 출력하고([ch9](../ch9-io-fs-path/README.md) 2.13절, 4.3절), `CStr`의 `display()`는 1.100.0에서 아직 nightly 전용이므로 `to_string_lossy()`를 씁니다. 디버그 형식(`{:?}`)은 넷 모두 구현하며, UTF-8이 아닌 바이트를 `\xE9`처럼 이스케이프해 보여 줍니다. 이 툴체인에서 `OsStr`은 대문자(`\xE9`), `CStr`은 소문자(`\xe9`)로 출력했습니다.

### 2.3 `OsString`과 `OsStr`

`OsString` 문서는 이 타입이 필요한 이유를 플랫폼별로 설명합니다. 유닉스에서 문자열은 흔히 **0이 아닌 아무 바이트의 나열**이고, Windows에서는 **0이 아닌 아무 16비트 값의 나열**(올바르면 UTF-16으로 해석)입니다. 반면 Rust 문자열은 언제나 올바른 UTF-8이어야 합니다. `OsString`은 둘 다를 손실 없이 담으면서, Rust 문자열에서는 비용 없이 만들 수 있게 설계되었습니다. 대신 0 바이트로 끝나지 않으므로 시스템 호출에 넘길 때는 `CStr`로 바꿔야 합니다.

**어디서 만나나.** 운영체제가 준 문자열을 돌려주는 API는 `String` 대신 `OsString`/`OsStr`을 씁니다. [`02_os_string.rs`](examples/02_os_string.rs)의 1번 구역이 세 곳을 보여 줍니다.

```rust
let pkg: Option<OsString> = env::var_os("CARGO_PKG_NAME");            // 환경 변수
let first_arg: Option<OsString> = env::args_os().next();              // 명령줄 인자
let name: Option<&OsStr> = Path::new("문서/회의록.md").file_name();    // 경로의 구성 요소
```

**UTF-8로 바꾸는 세 방법.** [ch10](../ch10-env-process-time/README.md) 2.2절에서 쓴 세 메서드를 비교하면 다음과 같습니다.

| 메서드 | 반환 타입 | UTF-8이 아니면 | 언제 쓰나 |
| --- | --- | --- | --- |
| `to_str()` | `Option<&str>` | `None` | 텍스트로 처리해야 하고, 아니면 에러로 다룰 때 |
| `to_string_lossy()` | `Cow<'_, str>` | 깨진 바이트를 `�`로 바꾼 `Cow::Owned` | 사람에게 보여 주거나 기록할 때 |
| `into_string()` | `Result<String, OsString>` | `Err(원래 값)`, 아무것도 잃지 않음 | 소유한 `String`이 필요할 때 |

**UTF-8이 아닌 값 직접 만들기(유닉스).** 유닉스에는 `OsStr`과 바이트를 서로 바꾸는 확장 트레이트 `std::os::unix::ffi::OsStrExt`(`from_bytes`, `as_bytes`)와 `OsStringExt`(`from_vec`, `into_vec`)가 있습니다. 예제 02의 3번 구역은 Latin-1로 저장된 옛 파일 이름을 흉내 내어 `b"caf\xE9.txt"`를 만듭니다. `0xE9`는 UTF-8에서 3바이트 글자의 첫 바이트인데 뒤가 이어지지 않으므로 올바르지 않습니다.

```text
3. UTF-8이 아닌 OsStr 만들기 (유닉스 기준)
    디버그 형식: "caf\xE9.txt"
    to_str():          None
    to_string_lossy(): "caf�.txt" (새로 할당했는가: true)
    display():         caf�.txt
    into_string(): Err("caf\xE9.txt")
    as_bytes():        [63, 61, 66, e9, 2e, 74, 78, 74]
    바이트 → OsString → 비교: true
```

`to_string_lossy()`와 `display()`의 결과에서는 `0xE9`가 사라졌지만 `OsStr` 자체는 원래 바이트를 그대로 들고 있습니다. 그래서 **운영체제에 다시 돌려줄 값(파일을 열거나 지울 경로)은 `OsString`/`PathBuf`로 들고 다니고, 보여 줄 때만 손실 변환**하는 것이 원칙입니다. 손실 변환한 문자열로 파일을 다시 찾으면 `caf�.txt`라는 다른 이름을 찾게 됩니다. Windows에는 같은 역할의 `std::os::windows::ffi`(`encode_wide`, `from_wide`)가 있습니다. 예제 02는 유닉스 전용 구역을 `#[cfg(unix)]`로 감싸 두었습니다.

**할 수 있는 일은 많지 않습니다.** `OsString`에는 `push`(이어 붙이기), `&str`과의 `==`, `eq_ignore_ascii_case`·`to_ascii_lowercase` 같은 ASCII 메서드, `len`, `display`(1.87.0)가 있습니다. `split`, `starts_with` 같은 문자열 처리는 1.100.0의 안정 버전에 없습니다. 예제 02의 4번 구역처럼 `to_str()`로 UTF-8인지 확인한 뒤 `&str`의 메서드를 쓰거나, `as_encoded_bytes()`(1.74.0)로 바이트를 꺼내 처리합니다. 문서에 따르면 이 바이트의 인코딩은 "UTF-8의 상위 집합"이라는 것만 정해져 있으므로, ASCII를 찾는 데는 써도 되지만 파일에 저장하거나 네트워크로 보내면 안 됩니다.

**`OsString` → `CString`.** 두 타입 사이에는 직접 바꾸는 메서드가 없습니다. 유닉스에서는 둘 다 "0이 아닌 바이트들"이므로 `CString::new(os.into_vec())`처럼 바이트를 거칩니다(예제 02의 5번 구역). 이때도 중간에 0 바이트가 있으면 `NulError`입니다.

### 2.4 `CString`과 `CStr`

`CString`은 "끝에 0 바이트가 하나 있고 중간에는 0 바이트가 없다"는 불변식을 지키는 소유한 C 문자열이고, `&CStr`은 그런 바이트를 빌린 것입니다. 인코딩은 보장하지 않습니다. [`03_c_string.rs`](examples/03_c_string.rs)가 만들기부터 되돌리기까지 순서대로 보여 줍니다.

**만들기: `CString::new`와 `NulError`.** `CString::new`는 `Into<Vec<u8>>`인 것(`&str`, `String`, `Vec<u8>`, `&[u8]`)을 받아 끝에 0 바이트를 붙입니다. 결과가 `Result<CString, NulError>`인 이유는 중간의 0 바이트 때문입니다.

```rust
let err = CString::new("이름\0비밀").unwrap_err();
println!("{err}");               // nul byte found in provided data at position: 6
println!("{}", err.nul_position()); // 6
let bytes = err.into_vec();      // 넘겼던 13바이트를 돌려받습니다
```

조용히 잘라서 넘기지 않고 만들 때 거절하는 이유는, C 쪽이 0 바이트까지만 읽기 때문입니다. `"이름\0비밀"`을 받은 C 함수는 `"이름"`만 봅니다. Rust 쪽에서 검사한 문자열과 C 쪽이 실제로 쓰는 문자열이 달라지면, 예를 들어 Rust에서 `".png"`로 끝나는지 확인한 파일 이름을 C가 그 앞에서 끊어 읽는 식의 문제가 생깁니다.

**`c"…"` 리터럴(1.77.0).** [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.4절에서 쓴 리터럴의 타입은 `&'static CStr`입니다. 컴파일러가 끝에 0 바이트를 붙이고, 중간에 `\0`을 쓰면 컴파일 에러(`null characters in C string literals are not supported`)로 거절합니다. 실행 중 검사가 필요 없으므로 고정된 문자열은 리터럴로 씁니다. 소유한 값이 필요하면 `c"…".to_owned()`나 `CString::from(c"…")`입니다.

**이미 있는 바이트에서 `&CStr` 빌리기.** 두 함수가 있습니다.

| 함수 | 조건 | 실패하면 |
| --- | --- | --- |
| `CStr::from_bytes_with_nul(&[u8])` | 마지막 바이트가 0이고 중간에는 0이 없음 | `NotNulTerminated`, `InteriorNul { position }` |
| `CStr::from_bytes_until_nul(&[u8])` (1.69.0) | 0 바이트가 하나 이상 있음. 첫 0 바이트까지가 문자열 | `FromBytesUntilNulError` |

`from_bytes_until_nul`은 C 구조체의 `char name[16]`처럼 **크기가 정해진 버퍼**를 읽을 때 꼭 맞습니다. 예제 03의 4번 구역은 16바이트 버퍼에 9바이트짜리 `"김철수"`를 넣고 나머지를 0으로 둔 뒤 `Ok("김철수")`를 읽어 냅니다. `from_bytes_with_nul(b"hi\0")`처럼 바이트 문자열 리터럴을 넘기면 Clippy가 `manual_c_str_literals`로 `c"hi"`를 쓰라고 알려 줍니다.

**Rust 쪽으로 되돌리기.**

| 메서드 | 결과 |
| --- | --- |
| `to_bytes()` / `to_bytes_with_nul()` | 0 바이트를 뺀 / 포함한 `&[u8]` |
| `count_bytes()` (1.79.0) | 0 바이트를 뺀 길이. C의 `strlen`과 같음 |
| `to_str()` | `Result<&str, Utf8Error>` |
| `to_string_lossy()` | `Cow<'_, str>` |
| `into_string()` (`CString`) | `Result<String, IntoStringError>`. 실패하면 `into_cstring()`으로 원래 값을 되찾음 |

**`NonZero<u8>`만 모은 `Vec`은 검사가 필요 없습니다.** `CString`은 `From<Vec<NonZero<u8>>>`(1.43.0)를 구현합니다. 원소마다 0이 아님이 보장되므로 중간의 0 바이트가 있을 수 없고, 그래서 `Result`가 아닌 `From`입니다(예제 03의 6번 구역). 타입이 불변식을 보장하면 검사가 사라진다는 것을 2.10절의 `NonZero`에서 다시 봅니다.

`&CStr`은 크기를 모르는 타입이고 지금은 넓은 포인터(이 환경에서 16바이트)지만, 문서는 메모리 배치를 보장하지 않으니 **FFI 함수의 시그니처에 `&CStr`을 쓰지 말고** `as_ptr()`로 얻은 `*const c_char`를 넘기라고 안내합니다.

### 2.5 C 함수에 넘기고 받기

[`04_calling_c.rs`](examples/04_calling_c.rs)는 C 표준 라이브러리의 `strlen`과 `getenv` 두 함수만 씁니다. 선언 방법(`unsafe extern "C"`)은 [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.4절과 같습니다.

```rust
unsafe extern "C" {
    fn strlen(s: *const c_char) -> usize;          // <string.h>
    fn getenv(name: *const c_char) -> *mut c_char; // <stdlib.h>, 없으면 NULL
}
```

**C 타입 별칭.** `std::ffi`의 `c_char`, `c_int`, `c_long` 등은 C 타입에 맞는 Rust 정수 타입의 별칭입니다. 어느 타입인지는 플랫폼이 정합니다. 특히 `c_char`(C의 `char`)는 부호가 있는지를 C 표준이 플랫폼에 맡기므로, 문서도 "플랫폼에 따라 `i8` 또는 `u8`"이라고만 말합니다. 이 환경(macOS aarch64)의 결과는 다음과 같았습니다.

```text
    c_char = i8 (최솟값 -128), c_int = i32, c_long = i64
```

그래서 `strlen("hello".as_ptr())`는 이 환경에서 `` expected `*const i8`, found `*const u8` ``(E0308)로 거부됩니다. 하지만 이것은 우연히 걸린 것입니다. `c_char`가 `u8`인 플랫폼이라면 이 코드는 컴파일되고, 0 바이트가 없는 `"hello"`의 끝을 넘어 읽는 미정의 동작이 됩니다. 타입이 다르다는 에러에 기대지 말고 **언제나 `CStr`을 거쳐** 포인터를 얻습니다.

**`&CStr`을 받는 안전한 래퍼.** `strlen`의 조건은 "0 바이트로 끝나는 살아 있는 버퍼"인데, `&CStr`이 바로 그것을 보장합니다. 그래서 매개변수를 `&CStr`로 받으면 래퍼는 `unsafe fn`일 필요가 없습니다([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)의 안전한 추상화).

```rust
fn c_strlen(s: &CStr) -> usize {
    // SAFETY: s.as_ptr()는 0 바이트로 끝나는 버퍼를 가리키고, 그 버퍼는 s를 빌린 동안
    // (이 호출이 끝날 때까지) 살아 있습니다. strlen은 읽기만 하고 포인터를 보관하지 않습니다.
    unsafe { strlen(s.as_ptr()) }
}
```

`&CString`은 역참조로 `&CStr`이 되므로 `c_strlen(c"hello")`와 `c_strlen(&owned)`가 모두 됩니다. `&str`을 넘기면 E0308입니다(4.1절).

**C 함수가 돌려준 포인터 읽기.** `getenv`는 환경 변수 값을 가리키는 포인터를 돌려주고, 없으면 NULL입니다. 받은 포인터는 세 단계로 다룹니다.

```rust
fn c_getenv(name: &CStr) -> Option<String> {
    let ptr = unsafe { getenv(name.as_ptr()) };   // SAFETY 주석은 예제 파일 참고
    if ptr.is_null() {
        return None;                               // ① NULL이면 여기서 끝
    }
    let value: &CStr = unsafe { CStr::from_ptr(ptr) }; // ② 0 바이트까지를 CStr로
    Some(value.to_string_lossy().into_owned())     // ③ 바로 복사해 소유한 값으로
}
```

- ① `CStr::from_ptr`에 NULL을 넘기면 미정의 동작입니다. C 함수의 문서가 NULL을 돌려줄 수 있다고 하면 먼저 확인합니다.
- ② `from_ptr`는 unsafe 함수입니다. 포인터가 0 바이트로 끝나는 유효한 메모리를 가리키고, 돌려받은 `&'a CStr`을 쓰는 동안 그 메모리가 바뀌지 않아야 합니다. 문서의 Caveat 절은 이 `'a`가 **쓰는 곳에 따라 추론될 뿐 아무것에도 묶여 있지 않다**고 경고합니다. 컴파일러는 이 참조가 언제까지 유효한지 모릅니다.
- ③ 그래서 언제까지 유효한지 모르는 포인터에서 얻은 값은 바로 복사합니다. `getenv`가 돌려준 버퍼는 누군가 환경을 바꾸면 무효가 될 수 있습니다. 2024 에디션에서 `env::set_var`가 unsafe 함수가 된 것도 이 때문입니다([ch10](../ch10-env-process-time/README.md) 2.4절).

예제 04의 4번 구역은 `CARGO_PKG_NAME`(cargo run이 실행하는 프로그램에 넣어 주는 환경 변수)을 `getenv`와 `env::var_os`로 각각 읽어 같은지 비교합니다. 이미 슬라이스를 가지고 있다면 unsafe인 `from_ptr` 대신 안전한 `from_bytes_until_nul`을 씁니다. 버퍼 안에 0 바이트가 없으면 끝을 넘어 읽는 대신 `Err`를 돌려주기 때문입니다(5번 구역).

**임시 `CString`의 `as_ptr()`.** `as_ptr()`가 돌려주는 원시 포인터에는 라이프타임이 없습니다. 그래서 대여 검사기는 `CString`이 버려진 뒤에 포인터를 쓰는 코드를 막지 못합니다. 가장 흔한 실수는 임시 `CString`에서 포인터만 꺼내 변수에 담는 것입니다.

```rust
let ptr = CString::new(word).unwrap().as_ptr(); // 이 문장이 끝나면 임시 CString이 버려집니다
let len = unsafe { strlen(ptr) };              // 해제된 메모리를 읽음: 미정의 동작
```

이 모양은 컴파일러가 `dangling_pointers_from_temporaries` 경고로 알려 줍니다(4.6절). 바르게 쓰는 방법은 둘입니다. `CString`을 변수에 묶어 포인터보다 오래 살게 하거나, `strlen(CString::new(word).unwrap().as_ptr())`처럼 한 표현식 안에서 만들고 바로 넘깁니다. 임시값은 그 표현식이 끝날 때까지 살아 있기 때문입니다(예제 04의 3번 구역). 다만 경고는 이 흔한 모양만 잡습니다. 블록 안에서 만든 `CString`의 포인터를 블록 밖의 변수에 담는 코드는 이 툴체인에서 아무 경고 없이 컴파일되었습니다.

이 챕터에서는 인자를 읽기만 하는 C 함수와 C가 돌려준 포인터를 읽는 것까지만 다룹니다. C 라이브러리를 직접 빌드해 링크하기, 콜백, 불투명 타입, C에서 Rust 부르기, `CString::into_raw`/`from_raw`로 소유권을 C에 넘겼다 되찾기는 Part 10 `ch12-ffi-beneath-std`에서, ABI와 링크 방식은 Part 9 `ch9-memory-panic-linkage`에서, `build.rs`로 C 코드를 컴파일하는 방법은 Part 3 `ch8-build-scripts`에서 다룹니다. 이 가운데 C 코드를 빌드해 링크하기, 불투명 포인터와 생성·해제 함수 쌍, C에서 Rust 부르기는 [부록 A ch34](../../appd-a/ch34-ffi-boundaries/README.md)에서 먼저 볼 수 있습니다.

### 2.6 `TypeId`와 `type_name`

여기부터는 두 번째 부입니다. [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)의 트레이트 객체는 "무슨 타입인지 몰라도 이 메서드가 있다"를 표현했습니다. 이번에는 반대로, 타입을 지운 값에게 **"너는 정확히 무슨 타입이냐"를 실행 중에 묻습니다.** 그 도구가 `std::any`입니다.

```rust
pub trait Any: 'static {
    fn type_id(&self) -> TypeId;
}
```

`Any`는 `'static`인 모든 타입에 자동으로 구현됩니다. 따라서 `i32`, `String`, `Vec<u8>`, 직접 만든 구조체는 모두 `Any`이고, 빌린 참조를 품은 타입(`&'a str` 등)은 `'static`일 때만 `Any`입니다(2.7절).

**`TypeId`** 는 타입마다 하나씩 있는 식별자입니다. `TypeId::of::<T>()`나 값의 `type_id()`로 얻고, `==`로 비교하거나 `HashMap`의 키로 씁니다([`05_any_downcast.rs`](examples/05_any_downcast.rs)의 1번 구역).

- 타입 별칭(`type Meters = u32`)은 같은 타입이므로 `TypeId`가 같고, 뉴타입(`struct UserId(u32)`)은 다른 타입이므로 다릅니다.
- 문서에 따르면 `TypeId`의 값, 해시, 순서는 Rust 버전마다 달라질 수 있습니다. 예제는 값을 출력하지 않고 같은지만 비교합니다. 파일에 저장하거나 다른 프로그램과 주고받지 않습니다.

**`type_name::<T>()`** (1.38.0)과 **`type_name_of_val(&v)`** (1.76.0)는 타입의 이름을 `&'static str`로 돌려줍니다. 문서는 **진단용**이라고 못 박습니다. 형식이 정해져 있지 않고, 컴파일러 버전에 따라 바뀔 수 있으며, 서로 다른 타입이 같은 이름을 가질 수도 있습니다. 이 툴체인의 결과는 다음과 같습니다.

```text
    alloc::vec::Vec<core::option::Option<&str>>
    f64
    dyn core::any::Any
```

경로가 `std::`가 아니라 정의된 곳(`alloc::`, `core::`)으로 나오는 것은 [ch5](../ch5-iterators/README.md)에서 본 것과 같습니다. 세 번째 줄처럼 트레이트 객체를 넘기면 안에 든 타입(`u8`)이 아니라 `dyn Any`라고만 답합니다. **프로그램의 동작은 `TypeId`로 정하고, `type_name`은 메시지와 디버깅에만 씁니다.**

### 2.7 `dyn Any`와 다운캐스트

`dyn Any`에는 안에 든 값의 구체 타입을 확인하고 꺼내는 메서드가 있습니다. 같은 메서드가 `dyn Any + Send`, `dyn Any + Send + Sync`에도 있습니다.

| 메서드 | 받는 것 | 돌려주는 것 |
| --- | --- | --- |
| `is::<T>()` | `&dyn Any` | `bool` |
| `downcast_ref::<T>()` | `&dyn Any` | `Option<&T>` |
| `downcast_mut::<T>()` | `&mut dyn Any` | `Option<&mut T>` |
| `downcast::<T>()` | `Box<dyn Any>` (소비) | `Result<Box<T>, Box<dyn Any>>`, 틀리면 원래 상자를 돌려받음 |

```rust
fn describe(value: &dyn Any) -> String {
    if let Some(n) = value.downcast_ref::<i32>() {
        format!("i32 {n}")
    } else if let Some(s) = value.downcast_ref::<String>() {
        format!("String {s:?} ({}바이트)", s.len())
    } else if let Some(s) = value.downcast_ref::<&str>() {
        format!("&str {s:?}")
    } else if value.is::<f64>() {
        String::from("f64 (타입만 확인)")
    } else {
        String::from("모르는 타입")
    }
}
```

**원리는 `TypeId` 비교입니다.** `downcast_ref::<T>()`는 안에 든 값의 `type_id()`와 `TypeId::of::<T>()`를 비교해, 같을 때만 포인터를 `&T`로 바꿔 돌려줍니다. 타입이 같다는 것을 확인했으므로 이 변환은 안전하고, 그래서 안전한 메서드입니다. 이 방식에서 두 가지가 따라 나옵니다.

- **정확히 같은 타입만 찾습니다.** `42u8`은 `i32`로 꺼낼 수 없고, `String`은 `&str`로 꺼낼 수 없습니다. 변환은 일어나지 않습니다. 또 문서가 밝히듯 "이 값이 어떤 트레이트를 구현하는가"는 물을 수 없습니다.
- **`'static`이어야 합니다.** `TypeId`에는 라이프타임이 들어 있지 않습니다. `&'a str`과 `&'static str`이 같은 `TypeId`를 가진다면, 잠깐 빌린 `&'a str`을 `dyn Any`에 넣었다가 `downcast_ref::<&'static str>()`로 꺼내 원래 값보다 오래 쓸 수 있게 됩니다. 이 구멍을 막으려고 `Any`는 `'static` 타입에만 구현됩니다. 지역 `String`을 빌린 조각을 `&dyn Any`로 넘기면 "`'static` 동안 빌려야 한다"는 E0597이 납니다(4.2절). [ch8](../ch8-errors-panic/README.md)의 `dyn Error + 'static`도 같은 이유였습니다.

**`Box<dyn Any>::downcast`** 는 상자째 소유권을 넘겨받습니다. 틀린 타입이면 `Err`에 원래 상자가 들어 있으므로 다른 타입으로 다시 시도할 수 있습니다(예제 05의 5번 구역). `Rc<dyn Any>`와 `Arc<dyn Any + Send + Sync>`에도 같은 모양의 `downcast`가 있습니다.

**"상자 자체도 `Any`" 함정.** `Box<dyn Any>`도 `'static` 타입이라 그 자체로 `Any`입니다. 그래서 두 가지 실수가 컴파일 에러 없이 일어납니다.

```rust
let boxed: Box<dyn Any> = Box::new(5u8);
boxed.type_id();    // Box<dyn Any>의 TypeId! (Clippy: type_id_on_box)
(*boxed).type_id(); // u8의 TypeId

let number: Box<dyn Any> = Box::new(42);
describe(&number);  // &Box<dyn Any>가 &dyn Any로 바뀌어 상자가 Any 값이 됨 → "모르는 타입"
describe(&*number); // 상자 안의 dyn Any를 빌림 → "i32 42"
```

`type_id`는 `Any` 트레이트의 메서드라서 메서드 탐색이 `Box` 자신의 구현을 먼저 찾습니다. 기본으로 켜진 Clippy의 `type_id_on_box`가 이것을 경고하므로, 예제 05는 일부러 보여 주려고 `#[expect]`로 막아 두었습니다. 반면 `downcast_ref`는 `dyn Any`에만 있는 고유 메서드라서 `boxed.downcast_ref::<u8>()`처럼 부르면 자동 역참조가 상자 안까지 들어가 바르게 동작합니다. 문제는 `&boxed`를 `&dyn Any` 자리에 **인자로 넘길 때**입니다. [ch8](../ch8-errors-panic/README.md) 4.6절의 페이로드 실수와 같은 원인입니다.

**패닉 페이로드도 같은 원리입니다.** `catch_unwind`와 `JoinHandle::join`이 돌려주는 페이로드의 타입은 `Box<dyn Any + Send>`입니다. `panic!("상수")`의 페이로드는 `&'static str`, 서식 인자가 있으면 `String`이므로 ch8의 `payload_text`는 두 타입으로 차례로 다운캐스트했습니다. 예제 05의 7번 구역은 패닉 없이 같은 타입의 값을 직접 만들어 같은 함수로 꺼냅니다. `dyn Error`의 `downcast_ref`(ch8 2.5절)는 `dyn Error`에 따로 있는 메서드지만, 같은 이유로 `'static`을 요구하는 실행 중 타입 확인입니다.

`downcast_unchecked_ref`처럼 타입 검사를 건너뛰는 unsafe 메서드도 문서에 보이지만 1.100.0에서는 nightly 전용입니다.

### 2.8 `Any`를 언제 쓰는가

다운캐스트는 강력하지만 컴파일러의 도움을 포기하는 일입니다. `describe`에 새 타입을 처리하는 갈래를 빠뜨려도 컴파일러는 아무 말도 하지 않고, 값은 조용히 `else` 갈래로 흘러갑니다. [`06_any_patterns.rs`](examples/06_any_patterns.rs)는 `Any`가 정당한 두 경우와 열거형이 더 나은 경우를 나란히 보여 줍니다.

**(1) 타입 맵: 담는 쪽이 타입을 모를 때.** 웹 프레임워크가 요청에 붙이는 확장 데이터처럼, 저장소를 만드는 쪽이 어떤 타입이 들어올지 미리 알 수 없는 경우가 있습니다. 이럴 때 `TypeId`를 키로 쓰는 맵을 만들고, 겉으로는 제네릭 메서드로 타입을 지정하게 합니다.

```rust
struct TypeMap {
    values: HashMap<TypeId, Box<dyn Any>>,
}

impl TypeMap {
    fn insert<T: Any>(&mut self, value: T) -> Option<T> {
        let old = self.values.insert(TypeId::of::<T>(), Box::new(value))?;
        old.downcast::<T>().ok().map(|boxed| *boxed)
    }

    fn get<T: Any>(&self) -> Option<&T> {
        self.values.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }
}
```

`TypeId::of::<T>()` 칸에는 `insert`가 `T`만 넣으므로 다운캐스트는 실패하지 않습니다. 이 불변식은 타입이 아니라 **구조체의 코드가** 지킵니다. 그래서 필드를 비공개로 두어 바깥에서 엉뚱한 값을 넣지 못하게 합니다. 쓰는 쪽은 `ext.get::<RequestId>()`처럼 타입으로 찾고, `RequestId` 안에 `u64`가 있어도 `get::<u64>()`는 `None`입니다.

**(2) 자기 트레이트 객체에서 특정 타입 골라내기: 트레이트 객체 업캐스트(1.86.0).** 위젯 목록을 `Vec<Box<dyn Widget>>`로 들고 있다가, 버튼에만 있는 필드가 필요해질 때가 있습니다. `Any`를 슈퍼트레이트로 두면 `&dyn Widget`을 `&dyn Any`로 **업캐스트**(하위 트레이트 객체를 슈퍼트레이트 객체로 바꾸기)한 뒤 다운캐스트할 수 있습니다.

```rust
trait Widget: Any {
    fn name(&self) -> String;
}

let total_clicks: u32 = widgets
    .iter()
    .filter_map(|w| {
        let any: &dyn Any = &**w; // &Box<dyn Widget> → &dyn Widget → &dyn Any
        any.downcast_ref::<Button>()
    })
    .map(|b| b.clicks)
    .sum();
```

`&**w`로 상자 안을 빌리는 것이 중요합니다. `w`(`&Box<dyn Widget>`)를 그대로 `&dyn Any`로 바꾸면 상자 자체가 `Any` 값이 되어 모두 실패합니다. `&mut dyn Widget` → `&mut dyn Any`, `Box<dyn Widget>` → `Box<dyn Any>`도 같은 방식으로 됩니다. 업캐스트는 1.86.0에서 안정화되었으므로, 그 전에 작성된 코드에서는 트레이트에 `fn as_any(&self) -> &dyn Any { self }` 메서드를 두고 구현마다 적는 모양을 자주 봅니다.

**(3) 종류가 정해져 있다면 열거형.** 위젯의 종류를 내가 다 알고 있다면 `enum WidgetKind { Button(Button), Slider(Slider) }`와 `match`가 낫습니다. 배리언트를 하나 추가하면 그 열거형을 `match`하는 모든 곳이 컴파일 에러가 되어 고칠 곳을 알려 줍니다(완전성 검사). 다운캐스트의 `else` 갈래는 그러지 못합니다.

| 상황 | 도구 | 컴파일러가 확인해 주는 것 |
| --- | --- | --- |
| 종류가 정해져 있고 내 코드가 다 앎 | 열거형 + `match` | 빠뜨린 종류 |
| 종류는 열려 있지만 할 일이 정해져 있음 | 트레이트 메서드(트레이트 객체) | 모든 타입이 그 메서드를 구현했는지 |
| 담는 쪽이 타입을 전혀 모름(패닉 페이로드, 타입 맵, 플러그인 데이터) | `Any` + 다운캐스트 | 거의 없음 |

`Any`는 표의 마지막 줄에만 씁니다. 위의 두 줄로 해결되는 문제를 `Any`로 풀면, 타입 검사를 실행 시간으로 미루는 대가만 치르게 됩니다.

### 2.9 `Wrapping`과 `Saturating`: 오버플로 처리를 타입으로 고르기

세 번째 부입니다. [ch1](../ch1-std-overview/README.md) 2.9절에서 오버플로를 처리하는 방법을 연산마다 메서드 이름으로 골랐습니다(`checked_add`, `wrapping_add`, `saturating_add`, `overflowing_add`). 같은 선택을 **값의 타입으로** 할 수도 있습니다.

```rust
pub struct Wrapping<T>(pub T);    // 넘치면 한 바퀴 돎 (1.0.0)
pub struct Saturating<T>(pub T);  // 최솟값·최댓값에서 멈춤 (1.74.0)
```

둘 다 정수 하나를 감싼 뉴타입이고, 문서는 메모리 배치가 감싼 정수와 같다고 보장합니다(`#[repr(transparent)]`). 안의 값은 튜플 필드 `.0`으로 꺼냅니다. 기본 연산자(`+`, `-`, `*`, `/`, `%`, 비트 연산, 시프트)를 그대로 쓰면 그 타입의 규칙으로 계산됩니다.

**메서드와 타입 비교.** 32비트 FNV-1a 해시를 두 방법으로 쓰면 차이가 보입니다([`07_wrapping_saturating.rs`](examples/07_wrapping_saturating.rs)의 2번 구역, 둘 다 `0x4f9f2cab`).

```rust
// 메서드: 넘칠 수 있는 연산마다 이름을 적어야 합니다.
hash ^= u32::from(b);
hash = hash.wrapping_mul(FNV_PRIME); // 그냥 *=를 쓰면 디버그 빌드에서 오버플로 패닉

// 타입: "넘치면 한 바퀴"가 이 값의 규칙이므로 연산자를 그대로 씁니다.
let mut hash = Wrapping(FNV_OFFSET);
hash ^= u32::from(b);
hash *= FNV_PRIME;
```

연산자가 디버그 빌드에서는 패닉하고 릴리스 빌드에서는 래핑한다는 것은 [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md)에서 봤습니다. `Wrapping`과 `Saturating`은 두 빌드에서 똑같이 동작합니다.

**타입이 정해 주는 것과 정해 주지 않는 것.**

- **감싼 값끼리 계산합니다.** `Wrapping<u8> + Wrapping<u8>`은 되지만 `Wrapping<u8> + u8`은 구현되어 있지 않아 E0277입니다(4.4절). 반면 복합 대입은 오른쪽에 감싸지 않은 정수를 받습니다(`w += 1u8`, `Wrapping`은 1.60.0부터). 그래서 `+=`는 되는데 `+`는 안 되는 비대칭이 생깁니다.
- **0으로 나누기는 여전히 패닉합니다.** 0으로 나눈 결과는 래핑으로도 포화로도 정할 수 없기 때문입니다. `Wrapping(5u8) / Wrapping(0)`과 `Saturating(5u8) / Saturating(0)`이 둘 다 패닉하는 것을 확인했습니다. 나눗셈이 섞이면 `checked_div`나 2.10절의 `NonZero`를 씁니다.
- **출력은 안의 값만 보여 줍니다.** 기본 형식뿐 아니라 디버그 형식도 `Wrapping(4)`가 아니라 `4`입니다.
- `Wrapping`의 메서드(`MIN`, `MAX`, `count_ones` 등)는 1.100.0에서 대부분 nightly 전용입니다. `Saturating`의 `MIN`/`MAX`는 안정입니다. 정수의 메서드가 필요하면 `.0`으로 꺼내 씁니다.
- 반복자의 합 `sum()`도 그 규칙을 따릅니다. 예제 07은 바이트를 `Wrapping<u8>`로 더해 256으로 나눈 나머지 체크섬을 만듭니다.

**어느 쪽을 고르나.**

| 상황 | 고르는 것 |
| --- | --- |
| 넘치는 것이 버그 | 기본 연산자(디버그 빌드에서 패닉으로 알림). 입력이 바깥에서 오면 `checked_*` |
| 넘치는 것을 알아야 함 | `checked_*`(`Option`), `overflowing_*`(`(값, bool)`) |
| 한 바퀴 도는 것이 정상인 값(해시, 체크섬, 순환 카운터) | 연산이 많으면 `Wrapping<T>`, 한두 곳이면 `wrapping_*` |
| 끝에서 멈추는 것이 정상인 값(체력, 음량, 통계 카운터) | 연산이 많으면 `Saturating<T>`, 한두 곳이면 `saturating_*` |

기준은 **"그 규칙이 연산의 성질인가, 값의 성질인가"** 입니다. 해시 값은 어느 연산에서든 래핑해야 하므로 값의 성질이고, 타입으로 표시하면 빠뜨릴 곳이 없습니다.

### 2.10 `NonZero`: 0이 아님을 타입으로

`NonZero<T>`는 "0이 아닌 정수"입니다. 제네릭 타입 `NonZero<T>`는 1.79.0에 안정화되었고, 그 전부터 있던 `NonZeroU32` 등은 이제 `NonZero<u32>`의 별칭입니다. `T`에는 표준 정수 타입만 올 수 있습니다(`T: ZeroablePrimitive`라는 불안정 트레이트가 막고 있어 직접 만든 타입에는 쓸 수 없습니다).

**만들 때 한 번 검사합니다.** `NonZero::new(n)`은 0이면 `None`인 `Option<NonZero<T>>`를 돌려줍니다. `NonZero::<u32>::try_from(0)`은 `Err(TryFromIntError(Zero))`입니다. 한 번 만든 뒤에는 `get()`으로 정수를 꺼낼 때 다시 검사하지 않습니다. 이것이 이 타입의 쓸모입니다. **0인지 확인하는 일을 값이 만들어지는 경계로 모으고, 그 안쪽 코드에서는 지웁니다.**

```rust
// const의 값은 컴파일할 때 계산되므로, 0을 넣으면 실행 전에 E0080으로 알게 됩니다(4.5절).
const WORKERS: NonZero<usize> = NonZero::new(4).unwrap();
```

검사를 건너뛰는 `NonZero::new_unchecked`는 0을 넣으면 미정의 동작인 unsafe 함수입니다. 이 챕터에서는 쓰지 않습니다.

**니치: `Option<NonZero<T>>`는 `T`와 크기가 같습니다.** [ch7](../ch7-smart-pointers-memory/README.md) 2.3절에서 `Option<Box<T>>`가 `Box<T>`와 같은 크기인 이유를 "결코 쓰지 않는 비트 조합(니치)에 `None`을 적는다"로 설명했습니다. `NonZero<T>`는 0을 쓰지 않으므로 `None`을 0으로 적으면 됩니다. `NonZero` 문서와 `std::option` 문서가 모두 이것을 **보장**합니다.

```text
    NonZero<u32> 4, Option<NonZero<u32>> 4, Option<u32> 8
    원소 1000개의 데이터 크기: Option<NonZero<u32>> 4000바이트, Option<u32> 8000바이트
```

`NonZero` 문서는 한 걸음 더 나아가 `Option<NonZero<T>>`가 FFI에서도 `T`와 호환된다고 보장합니다. C 함수가 "0이면 없음"으로 쓰는 정수를 Rust 쪽에서 `Option<NonZero<u32>>`로 받을 수 있다는 뜻입니다.

**0으로 나누기가 사라지는 나눗셈.** 부호 없는 정수에는 `u32 / NonZero<u32>`와 `u32 % NonZero<u32>`가 구현되어 있습니다(1.51.0). 문서대로 이 나눗셈에는 0 나누기 검사가 없고 패닉하지 않습니다.

```rust
fn average(total: u32, count: NonZero<u32>) -> u32 {
    total / count
}

match NonZero::new(scores.len() as u32) {
    Some(count) => println!("평균 = {}", average(scores.iter().sum(), count)),
    None => println!("점수가 없습니다"), // 0인 경우는 경계에서 한 번 처리
}
```

부호 있는 정수에는 이 구현이 없어서 `10i32 / NonZero<i32>`는 E0277(`` cannot divide `i32` by `NonZero<i32>` ``)입니다. 나누는 수가 0이 아니어도 `i32::MIN / -1`은 넘치기 때문입니다.

**산술은 결과도 0이 아님이 보장될 때만 있습니다.** `NonZero + NonZero`는 구현되어 있지 않습니다(E0369). 대신 `checked_add`(0이 아닌 값에 부호 없는 정수를 더한 결과는 넘치지 않는 한 0이 아님)는 `Option<NonZero<T>>`를, `saturating_add`는 `NonZero<T>`를 돌려줍니다(예제 08의 3번 구역).

**실제로 만나는 곳.** [ch11](../ch11-threads-sync/README.md)의 `thread::available_parallelism()`은 `io::Result<NonZero<usize>>`를 돌려줍니다. "동시에 돌릴 스레드 수"는 0일 수 없으므로 받는 쪽이 0 나누기를 걱정하지 않게 한 것입니다. 2.4절의 `CString::from(Vec<NonZero<u8>>)`도 같은 생각입니다.

### 2.11 정수 파싱 에러: `ParseIntError`와 `IntErrorKind`

`str::parse`로 정수를 읽다 실패하면 `ParseIntError`가 옵니다. [ch2](../ch2-option-result/README.md)에서 잠깐 본 `kind()`(1.55.0)로 이유를 `IntErrorKind` 열거형으로 꺼낼 수 있습니다. 예제 08의 4번 구역은 같은 입력들을 `NonZero<u32>`로 읽습니다.

```text
    "8" → Ok(8)
    "0" → Zero: 0은 안 됨 (number would be zero for non-zero type)
    "" → Empty: 빈 문자열 (cannot parse integer from empty string)
    "12a" → InvalidDigit: 숫자가 아닌 글자 (invalid digit found in string)
    "-1" → InvalidDigit: 숫자가 아닌 글자 (invalid digit found in string)
    "99999999999" → PosOverflow: 너무 큼 (number too large to fit in target type)
    "-200" as i8 → NegOverflow: 너무 작음
```

- `Zero`는 `NonZero`로 파싱할 때만 나옵니다. `NonZero`의 `FromStr`이 0을 거절하기 때문입니다.
- 부호 없는 타입에 `"-1"`을 넣으면 `NegOverflow`가 아니라 `InvalidDigit`입니다. `-` 자체를 받지 않기 때문입니다. `"+8"`은 `Ok(8)`이었습니다.
- 앞뒤 공백도 `InvalidDigit`입니다(`" 8"`). 표준 입력에서 읽은 줄은 `trim()`한 뒤 파싱합니다.
- `IntErrorKind`는 `#[non_exhaustive]`로 표시되어 있어서, 다른 크레이트(우리 코드)에서 `match`할 때 `_` 갈래가 반드시 필요합니다. 빠뜨리면 E0004(`` `IntErrorKind` is marked as non-exhaustive, so a wildcard `_` is necessary ``)입니다. 나중에 표준 라이브러리가 배리언트를 추가해도 우리 코드가 깨지지 않게 하려는 표시이며, 실제로 1.100.0 문서에는 nightly 전용 배리언트 `NotAPowerOfTwo`가 이미 보입니다.

### 2.12 이 챕터에서 다루지 않는 것

| 주제 | 한 줄 소개 | 다루는 곳 |
| --- | --- | --- |
| C 라이브러리 직접 링크, 콜백, 불투명 타입, C에서 Rust 부르기, `into_raw`/`from_raw` | Rust와 C가 소유권과 함수를 주고받는 방법 | Part 10 `ch12-ffi-beneath-std` |
| ABI와 링크 방식 | 함수를 부르는 약속과 라이브러리를 붙이는 방법 | Part 9 `ch9-memory-panic-linkage` |
| `build.rs`로 C 코드 컴파일 | 빌드 전에 실행되는 Rust 스크립트 | Part 3 `ch8-build-scripts` |
| `#[repr(transparent)]`, 타입 레이아웃 | 구조체가 메모리에 놓이는 방식을 정하는 속성 | Part 9 `ch7-type-system`, Part 10 `ch2-data-layout` |
| unsafe 코드를 Miri로 검사하기 | 미정의 동작을 실행 중에 찾는 도구 | Part 10 |
| `env::set_var`가 unsafe가 된 경위 | 2021 → 2024 에디션의 변화 | [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_string_family.rs`](examples/01_string_family.rs) | 같은 글자를 네 가족으로, 소유·빌림 짝, 공짜인 변환과 검사가 필요한 변환 | 1 |
| [`02_os_string.rs`](examples/02_os_string.rs) | `OsString`을 만나는 곳, UTF-8 변환 세 가지, UTF-8이 아닌 값(유닉스), `OsString` → `CString` | 1, 2 |
| [`03_c_string.rs`](examples/03_c_string.rs) | `CString::new`와 `NulError`, `c"…"`, `from_bytes_with_nul`/`until_nul`, Rust 문자열로 되돌리기 | 3 |
| [`04_calling_c.rs`](examples/04_calling_c.rs) | C 타입 별칭, `strlen`·`getenv`의 안전한 래퍼, 임시값과 `as_ptr`, `CStr::from_ptr` | 3 |
| [`05_any_downcast.rs`](examples/05_any_downcast.rs) | `TypeId`, `type_name`, `downcast_ref`/`mut`, `Box::downcast`, 상자 자체도 `Any`, 패닉 페이로드 | 4 |
| [`06_any_patterns.rs`](examples/06_any_patterns.rs) | 타입 맵, 트레이트 객체 업캐스트, 같은 일을 열거형으로 | 4 |
| [`07_wrapping_saturating.rs`](examples/07_wrapping_saturating.rs) | 메서드와 타입 비교, FNV-1a 해시와 체크섬, 포화 연산, 0 나누기 | 5 |
| [`08_nonzero.rs`](examples/08_nonzero.rs) | `NonZero` 만들기, 니치, 패닉 없는 나눗셈, `ParseIntError`/`IntErrorKind` | 5 |
| [`compile_fail/`](compile_fail) | 이 챕터의 타입에서 만나는 컴파일 에러 5개(4절) | 3, 4, 5 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 패닉 없이 종료 코드 0으로 곧바로 끝나고, 실행할 때마다 같은 출력이 나옵니다. 02와 04는 `cargo run`이 넣어 주는 환경 변수 `CARGO_PKG_NAME`을 읽으므로, 빌드된 실행 파일을 직접 실행하면 그 줄이 `None`으로 바뀝니다.

### 01_string_family — 문자열 타입 가족의 지도

```sh
cargo run -p p1-ch14-ffi-any-misc --example 01_string_family
```

```text
1. 같은 글자, 네 가족
    String   "메모.txt": 10바이트
    OsString "메모.txt": 10바이트
    CString  "메모.txt": 내용 10바이트, 0 바이트까지 11바이트
    PathBuf  "메모.txt": 10바이트
2. 소유한 것과 빌린 것은 짝을 이룹니다
    &str "메모.txt", &OsStr "메모.txt", &CStr "메모.txt", &Path "메모.txt"
    to_owned로 되돌린 값이 원래와 같은가: true
3. UTF-8에서 출발하는 변환은 검사가 필요 없습니다
    OsStr::new, Path::new가 같은 바이트를 가리키는가: true, true
    Path → OsStr: "메모.txt"
    "a\0b" → CString: Err(NulError(1, [97, 0, 98]))
4. UTF-8로 돌아오는 변환은 실패할 수 있습니다
    OsStr::to_str → Some("메모.txt")
    Path::to_str  → Some("메모.txt")
    CStr::to_str  → Ok("메모.txt")
    to_string_lossy → "메모.txt", "메모.txt", "메모.txt"
```

- **1번 구역**: 네 가족 모두 같은 10바이트를 담고, `CString`만 끝에 0 바이트 하나를 더 가집니다. `CString`만 만들 때 `Result`를 돌려줍니다.
- **4번 구역**: 같은 "UTF-8로 바꾸기"인데 `OsStr`·`Path`는 `Option`, `CStr`은 `Result<&str, Utf8Error>`를 돌려줍니다. `CStr` 쪽은 어디서 UTF-8이 깨졌는지(`valid_up_to`)도 알려 줍니다.

### 02_os_string — `OsString`과 `OsStr`

```sh
cargo run -p p1-ch14-ffi-any-misc --example 02_os_string
```

```text
1. OsString/OsStr를 만나는 곳
    var_os("CARGO_PKG_NAME")    = Some("p1-ch14-ffi-any-misc")
    var_os("CH14_SURELY_UNSET") = None
    args_os()의 0번 인자가 있는가: true
    file_name() = Some("회의록.md")
2. UTF-8이라면 세 가지 변환이 모두 성공합니다
    to_str()          = Some("회의록.md")
    to_string_lossy() = "회의록.md"
    into_string()     = Ok("회의록.md")
3. UTF-8이 아닌 OsStr 만들기 (유닉스 기준)
    …
4. OsString으로 할 수 있는 일은 많지 않습니다
    push로 이어 붙이기: "server-2026.LOG"
    == "server-2026.LOG": true, eq_ignore_ascii_case("SERVER-2026.log"): true
    to_str 후 ends_with: true, as_encoded_bytes 후 starts_with: true
5. OsString → CString (유닉스 기준)
    Ok("/tmp/caf\xe9.txt")
    Err(1)
```

- 출력 블록의 `…` 줄은 2.3절에 실은 3번 구역의 일곱 줄을 줄여 적은 것입니다.
- **3·5번 구역**: `std::os::unix::ffi`를 쓰므로 `#[cfg(unix)]`로 감쌌습니다. Windows에서는 두 구역 대신 "유닉스 전용 구역이라 건너뜁니다"가 출력됩니다(Windows에서는 실행해 보지 않았습니다).
- **5번 구역**: 실제 파일을 만들지 않고 경로 문자열만 바꿉니다. 두 번째 줄은 중간에 0 바이트가 있는 `OsString`을 `CString`으로 바꾸려다 1번 위치에서 거절된 것입니다.

### 03_c_string — `CString`과 `CStr`

```sh
cargo run -p p1-ch14-ffi-any-misc --example 03_c_string
```

```text
1. CString::new: 0 바이트를 붙인 새 버퍼를 만듭니다
    "안녕": to_bytes [236, 149, 136, 235, 133, 149], to_bytes_with_nul [236, 149, 136, 235, 133, 149, 0]
    "ok".count_bytes() = 2
2. 중간에 0 바이트가 있으면 NulError
    에러 메시지: nul byte found in provided data at position: 6
    nul_position(): 6
    돌려받은 바이트 수: 13
3. c"..." 리터럴은 &'static CStr입니다
    "리터럴": 9바이트 + 0 바이트
    to_owned() == 리터럴: true
4. 이미 있는 바이트에서 &CStr 빌리기
    Ok("hello")
    Err(NotNulTerminated)
    Err(InteriorNul { position: 2 })
    Ok("김철수")
    Err(FromBytesUntilNulError(()))
5. Rust 문자열로 되돌리기
    디버그 형식: "f\xffo"
    to_str():          Ok("안녕")
    to_str():          Err(1)
    to_string_lossy(): "f�o"
    into_string(): Err, 되찾은 값 "f\xffo"
6. 0이 아닌 바이트만 모았다면 검사가 필요 없습니다
    "abc"
```

- **2번 구역**: 위치 6은 바이트 위치입니다. `"이름"`이 6바이트이므로 0 바이트는 6번 바이트입니다. 돌려받은 13바이트는 `"이름"`(6) + `\0`(1) + `"비밀"`(6)입니다.
- **5번 구역**: 두 번째 `to_str()`의 `Err(1)`은 `Utf8Error::valid_up_to()`로, 앞의 1바이트(`f`)까지만 올바르다는 뜻입니다.

### 04_calling_c — C 함수에 넘기고 받기

```sh
cargo run -p p1-ch14-ffi-any-misc --example 04_calling_c
```

```text
1. C 타입 별칭 (이 환경 기준)
    c_char = i8 (최솟값 -128), c_int = i32, c_long = i64
2. &CStr을 받는 안전한 래퍼
    c_strlen(c"hello") = 5
    c_strlen(&owned) = 19, owned.count_bytes() = 19
3. as_ptr()의 포인터는 CString이 살아 있는 동안만 유효합니다
    (가) 변수에 묶음: 9, (나) 한 표현식 안: 9
4. C 함수가 돌려준 포인터 읽기: getenv
    getenv("CARGO_PKG_NAME") = Some("p1-ch14-ffi-any-misc")
    env::var_os와 같은가: true
    getenv("CH14_SURELY_UNSET") = None
5. 직접 가진 버퍼에 CStr::from_ptr 쓰기
    from_ptr: "abc"
    from_bytes_until_nul: Ok("abc")
```

- **1번 구역**: macOS aarch64에서 확인한 값입니다. 다른 플랫폼에서는 `c_char`가 `u8`이거나 `c_long`이 `i32`일 수 있습니다.
- **unsafe 블록의 위치**: `c_strlen` 안의 `strlen`, `c_getenv` 안의 `getenv`와 `CStr::from_ptr`, 3번 구역의 `strlen` 두 번, 5번 구역의 `CStr::from_ptr`입니다. 블록마다 왜 안전한지 `SAFETY:` 주석을 달았습니다.
- **5번 구역**: `from_ptr`는 첫 0 바이트(3번 위치)에서 멈추므로 뒤의 `"xyz"`는 읽지 않습니다.

### 05_any_downcast — `Any`와 다운캐스트

```sh
cargo run -p p1-ch14-ffi-any-misc --example 05_any_downcast
```

```text
1. TypeId: 타입마다 하나인 식별자
    7i32.type_id() == TypeId::of::<i32>(): true
    &str과 String: false
    별칭 Meters와 u32: true, 뉴타입 UserId와 u32: false
2. type_name: 사람이 읽는 타입 이름 (진단용)
    alloc::vec::Vec<core::option::Option<&str>>
    f64
    dyn core::any::Any
3. &dyn Any에서 구체 타입 꺼내기: is, downcast_ref
    i32 42
    String "러스트" (9바이트)
    &str "러스트"
    f64 (타입만 확인)
    모르는 타입
    모르는 타입
4. downcast_mut로 고치기
    u32였는가: true, String이었는가(u32로 꺼내기): false, hits = 10
5. Box<dyn Any>::downcast는 소유권째 꺼내고, 틀리면 상자를 돌려줍니다
    i32가 아니라서 상자를 돌려받았습니다
    String으로 꺼냄: 상자 안의 문자열
6. 상자 자체도 Any입니다
    boxed.type_id()가 Box<dyn Any>의 것인가: true, (*boxed).type_id()가 u8의 것인가: true
    describe(&number) = 모르는 타입, describe(&*number) = i32 42
7. 패닉 페이로드도 Box<dyn Any + Send>입니다
    &str 페이로드: 상수 메시지
    String 페이로드: 3번 시도 실패
    문자열이 아닌 페이로드
```

- **2번 구역**: `type_name`의 문자열은 툴체인에 따라 달라질 수 있습니다.
- **3번 구역**: 다섯 번째 값은 `42u8`, 여섯 번째는 `vec![1, 2]`(`Vec<i32>`)입니다. 둘 다 `describe`가 묻는 타입과 정확히 같지 않아 "모르는 타입"입니다.

### 06_any_patterns — `Any`를 쓰는 곳과 쓰지 않는 곳

```sh
cargo run -p p1-ch14-ffi-any-misc --example 06_any_patterns
```

```text
1. 타입 맵: 타입이 곧 키입니다
    Config.verbose = Some(true), RequestId = Some(7)
    get::<u64>() = None
    로그 = Some(["인증 통과", "권한 확인"])
    같은 타입을 다시 넣으면 옛 값을 돌려줍니다: Some(7) → Some(8)
2. dyn Widget → dyn Any 업캐스트로 특정 타입만 골라내기
    ["버튼 '저장'", "슬라이더 40", "버튼 '취소'"]
    버튼 클릭 합계: 3
    꺼낸 슬라이더 값: 100
3. 종류가 정해져 있다면 열거형이 낫습니다
    클릭 합계 3, 이름 ["저장", "슬라이더 40"]
```

- **2번 구역**: 슬라이더 값이 40에서 100으로 바뀐 것은 `&mut dyn Widget`을 `&mut dyn Any`로 업캐스트한 뒤 `downcast_mut`로 고쳤기 때문입니다. 마지막에는 `Box<dyn Widget>`을 `Box<dyn Any>`로 올려 소유권째 꺼냈습니다.

### 07_wrapping_saturating — 오버플로 처리를 타입으로

```sh
cargo run -p p1-ch14-ffi-any-misc --example 07_wrapping_saturating
```

```text
1. 메서드로 고르기: 연산마다 이름을 적습니다
    250u8 + 10 → checked None, wrapping 4, saturating 255, overflowing (4, true)
2. Wrapping<T>: 넘치면 한 바퀴 도는 것이 그 값의 규칙일 때
    Wrapping(250u8) + Wrapping(10) = 4 (디버그 형식 4)
    Wrapping(0u32) - Wrapping(1) = 4294967295
    FNV-1a("hello"): 메서드 0x4f9f2cab, Wrapping 0x4f9f2cab
    체크섬 = 0x15
3. Saturating<T>: 끝에 닿으면 멈추는 것이 그 값의 규칙일 때
    체력 30에서 50 피해: 0
    200씩 두 번 회복: 255
    -100i8 * 2 = -128, 100i8 + 100 = 127
4. 타입이 정해 주는 것과 정해 주지 않는 것
    Wrapping(255u8) += 1 → 0
    checked_div로 0 나누기 피하기: None
```

- **2번 구역**: 체크섬은 `0x10 + 0xF0 + 0x80 + 0x95 = 0x215`를 256으로 나눈 나머지 `0x15`입니다.

### 08_nonzero — `NonZero`와 정수 파싱 에러

```sh
cargo run -p p1-ch14-ffi-any-misc --example 08_nonzero
```

```text
1. 만들 때 한 번 검사합니다
    NonZero::new(5u32) = Some(5), NonZero::new(0u32) = None
    NonZero::<u32>::try_from(0) = Err(TryFromIntError(Zero))
    get() = -3, WORKERS = 4, NonZero::<u8>::MIN = 1
2. 니치: Option<NonZero<T>>는 T와 크기가 같습니다 (문서가 보장)
    …
3. 0으로 나누기가 사라지는 나눗셈
    평균 = 81
    빈 배열: 점수가 없습니다
    250 checked_add(5) = Some(255), checked_add(6) = None, saturating_add(100) = 255
4. 문자열에서 읽기: ParseIntError와 IntErrorKind
    …
```

- 출력 블록의 `…` 줄은 2.10절에 실은 2번 구역의 두 줄과 2.11절에 실은 4번 구역의 일곱 줄을 줄여 적은 것입니다.
- **1번 구역**: `NonZero`는 디버그 형식으로도 안의 값만 보여 주므로 `Some(NonZero(5))`가 아니라 `Some(5)`입니다.
- **3번 구역**: `(70 + 85 + 90) / 3 = 81`(정수 나눗셈)입니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch14-ffi-any-misc
rustc --edition 2024 compile_fail/e0308_str_as_cstr.rs --out-dir /tmp/rust-p1-ch14
```

에러 메시지 중 표준 라이브러리 소스 위치(`/rustc/0ef1bc38.../library/...`)를 가리키는 줄은 툴체인마다 해시가 달라서 아래 인용에서 생략했습니다.

### 4.1 `&CStr` 자리에 `&str` — E0308

파일: [`compile_fail/e0308_str_as_cstr.rs`](compile_fail/e0308_str_as_cstr.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_str_as_cstr.rs:21:29
   |
21 |     println!("{}", c_strlen("hello")); // → E0308
   |                    -------- ^^^^^^^ expected `&CStr`, found `&str`
   |                    |
   |                    arguments to this function are incorrect
   |
   = note: expected reference `&CStr`
              found reference `&'static str`
```

- **읽는 법**: `c_strlen`의 매개변수는 `&CStr`인데 `&str`을 넘겼다는 뜻입니다. `String`이 `&str`로 바뀌는 것 같은 자동 변환은 두 타입 사이에 없습니다.
- **원인**: `"hello"`는 끝에 0 바이트가 없고, 일반적으로 `&str`은 중간에 0 바이트가 있을 수도 있습니다. 둘 다 `CStr`의 불변식을 어기므로, 바꾸려면 검사(`CString::new`)나 복사가 필요합니다. 컴파일러가 몰래 해 줄 수 없는 일입니다.
- **해결**: 고정된 문자열이면 `c_strlen(c"hello")`, 실행 중에 만든 문자열이면 `c_strlen(&CString::new(text)?)`처럼 `CString`을 거칩니다(2.4절).

### 4.2 빌린 문자열 조각을 `&dyn Any`로 — E0597

파일: [`compile_fail/e0597_borrowed_str_as_any.rs`](compile_fail/e0597_borrowed_str_as_any.rs)

```text
error[E0597]: `line` does not live long enough
  --> compile_fail/e0597_borrowed_str_as_any.rs:18:23
   |
17 |     let line = String::from("이름=러스트");
   |         ---- binding `line` declared here
18 |     let name: &str = &line[7..]; // line을 빌린 조각
   |                       ^^^^ borrowed value does not live long enough
19 |     println!("{}", describe(&name)); // → E0597
   |                             ----- coercion requires that `line` is borrowed for `'static`
20 | }
   | - `line` dropped here while still borrowed
```

- **읽는 법**: 핵심은 19번 줄의 `` coercion requires that `line` is borrowed for `'static` ``입니다. `&name`(`&&str`)을 `&dyn Any`로 바꾸는 순간(coercion) `name`의 타입 `&str`이 `Any`여야 하고, `Any`는 `'static` 타입에만 구현되므로 `line`을 `'static` 동안 빌려야 한다는 요구가 생겼습니다.
- **원인**: `dyn Any`는 라이프타임을 기억하지 못하므로 `'static`이 아닌 참조를 받지 않습니다(2.7절). [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md)의 E0310(`execute`의 클로저가 스레드보다 먼저 사라질 수 있음)도 `'static`을 요구했지만, 그쪽은 "스레드가 언제까지 살지 모른다"는 이유였고 이쪽은 "다운캐스트가 라이프타임을 확인할 수 없다"는 이유입니다.
- **해결**: 소유한 값을 넘깁니다(`describe(&name.to_owned())`는 `String`이 되어 `"String"` 갈래로 갑니다). 값이 정말 `'static`이라면(`let name: &'static str = "러스트";`) 그대로 됩니다. 다운캐스트할 필요가 없는 함수라면 `&dyn Any` 대신 제네릭이나 `&str`으로 받습니다.

### 4.3 `Box<dyn Any>`에서 다운캐스트 없이 메서드 호출 — E0599

파일: [`compile_fail/e0599_method_on_box_dyn_any.rs`](compile_fail/e0599_method_on_box_dyn_any.rs)

```text
error[E0599]: the method `len` exists for struct `Box<(dyn Any + 'static)>`, but its trait bounds were not satisfied
  --> compile_fail/e0599_method_on_box_dyn_any.rs:14:26
   |
14 |     println!("{}", value.len()); // → E0599
   |                          ^^^ method cannot be called on `Box<(dyn Any + 'static)>` due to unsatisfied trait bounds
   |
   = note: the following trait bounds were not satisfied:
           `dyn Any: ExactSizeIterator`
           which is required by `Box<dyn Any>: ExactSizeIterator`
```

- **읽는 법**: 메시지가 엉뚱해 보입니다. 컴파일러는 `len`이라는 이름의 메서드를 찾다가 `Box<I>`의 `ExactSizeIterator` 구현(반복자를 담은 상자에도 `len`이 있음)을 발견했고, 그 조건(`dyn Any: ExactSizeIterator`)이 맞지 않는다고 알려 준 것입니다. 요점은 "`Box<dyn Any>`로는 `String`의 `len`을 부를 수 없다"입니다.
- **원인**: 안에 `String`을 넣었어도 타입은 `Box<dyn Any>`입니다. 컴파일러가 아는 것은 "어떤 `'static` 타입"뿐이고, 부를 수 있는 것은 `type_id`와 다운캐스트 메서드뿐입니다. 트레이트 객체에서는 그 트레이트의 메서드만 부를 수 있다는 [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)의 규칙 그대로입니다.
- **해결**: 먼저 꺼냅니다. `value.downcast_ref::<String>().map(|s| s.len())`은 `Some(9)`입니다. 처음부터 `String`인 줄 안다면 `Box<dyn Any>`로 지우지 말고 `String`으로 둡니다(2.8절).

### 4.4 `Wrapping<u8>`에 그냥 `u8`을 더함 — E0277

파일: [`compile_fail/e0277_wrapping_plus_u8.rs`](compile_fail/e0277_wrapping_plus_u8.rs)

```text
error[E0277]: cannot add `u8` to `Wrapping<u8>`
  --> compile_fail/e0277_wrapping_plus_u8.rs:14:21
   |
14 |     let next = hash + 10u8; // → E0277
   |                     ^ no implementation for `Wrapping<u8> + u8`
   |
   = help: the trait `Add<u8>` is not implemented for `Wrapping<u8>`
help: the following other types implement trait `Add<Rhs>`
   = note: `Wrapping<u8>` implements `Add`
   = note: `&Wrapping<u8>` implements `Add<Wrapping<u8>>`
   = note: `Wrapping<u8>` implements `Add<&Wrapping<u8>>`
   = note: `&Wrapping<u8>` implements `Add`
```

- **읽는 법**: `+`는 `Add` 트레이트입니다([ch6](../ch6-core-traits/README.md) 2.8절). `Wrapping<u8>`에는 `Add<u8>`이 없고, 있는 것은 아래 `note:` 네 줄, 즉 `Wrapping<u8>`(과 그 참조)끼리의 덧셈뿐입니다.
- **원인**: `Wrapping` 규칙을 쓰겠다는 선택을 값의 타입으로 표시하는 것이 이 타입의 목적이므로, 감싸지 않은 정수와의 연산은 일부러 두지 않았습니다. 다만 복합 대입은 오른쪽에 `u8`을 받으므로(`hash += 10u8`), `+=`는 되고 `+`는 안 되는 비대칭이 있습니다(2.9절).
- **해결**: `hash + Wrapping(10)`으로 감싸서 더합니다. 결과는 `4`입니다. 값을 바꾸는 중이라면 `hash += 10;`도 됩니다.

### 4.5 `const`에서 `NonZero::new(0).unwrap()` — E0080

파일: [`compile_fail/e0080_const_nonzero_zero.rs`](compile_fail/e0080_const_nonzero_zero.rs)

```text
error[E0080]: evaluation panicked: called `Option::unwrap()` on a `None` value
  --> compile_fail/e0080_const_nonzero_zero.rs:11:31
   |
11 | const WORKERS: NonZero<u32> = NonZero::new(0).unwrap(); // → E0080
   |                               ^^^^^^^^^^^^^^^^^^^^^^^^ evaluation of `WORKERS` failed here

note: erroneous constant encountered
  --> compile_fail/e0080_const_nonzero_zero.rs:14:20
   |
14 |     println!("작업자 {WORKERS}명");
   |                       ^^^^^^^
```

- **읽는 법**: `const`의 값을 계산(evaluation)하다가 패닉했다는 뜻입니다. 패닉 메시지는 실행 중에 `unwrap`이 실패했을 때와 같습니다. `note: erroneous constant encountered`는 그 상수를 쓰는 곳을 가리킬 뿐이며, 이 툴체인에서는 같은 `note`가 두 번 나왔습니다.
- **원인**: `const`의 값은 컴파일할 때 계산됩니다. `NonZero::new(0)`은 `None`이므로 `unwrap`이 컴파일 도중에 패닉하고, 그것이 컴파일 에러가 됩니다. 이 에러는 문제가 아니라 기능입니다. "0이 아님"을 프로그램을 실행하기 전에 확인받은 것입니다.
- **해결**: 0이 아닌 값을 넣습니다(`NonZero::new(4).unwrap()`, 예제 08). 값이 실행 중에 정해진다면 `const`가 아니라 `NonZero::new(n)`의 `None`을 그 자리에서 처리합니다(2.10절).

### 4.6 임시 `CString`의 `as_ptr()` — 경고 `dangling_pointers_from_temporaries`

이 실수는 컴파일 에러가 아니라 경고이므로 `compile_fail/`에 두지 않았습니다. 다음 코드를 `dangling.rs`로 저장해 컴파일하면 이렇게 나옵니다. 실행하면 미정의 동작이므로 실행하지 않습니다.

```rust
let word = String::from("러스트");
let ptr = CString::new(word).unwrap().as_ptr(); // 임시 CString이 이 문장 끝에서 버려집니다
// SAFETY: (틀림) ptr이 가리키는 CString은 이미 버려졌습니다.
let len = unsafe { strlen(ptr) };
```

```text
warning: this creates a dangling pointer because temporary `CString` is dropped at end of statement
 --> dangling.rs:9:43
  |
9 |     let ptr = CString::new(word).unwrap().as_ptr(); // 임시 CString이 이 문장 끝에서 버려집니다
  |               --------------------------- ^^^^^^ pointer created here
  |               |
  |               this `CString` is dropped at end of statement
  |
  = help: bind the `CString` to a variable such that it outlives the pointer returned by `as_ptr`
  = note: a dangling pointer is safe, but dereferencing one is undefined behavior
  = note: returning a pointer to a local variable will always result in a dangling pointer
  = note: for more information, see <https://doc.rust-lang.org/reference/destructors.html>
  = note: `#[warn(dangling_pointers_from_temporaries)]` on by default
```

- **읽는 법**: 아래쪽 밑줄(`this CString is dropped at end of statement`)이 임시값, 위쪽 `^^^^^^`가 포인터를 만든 곳입니다. 두 번째 `note:`가 핵심입니다. 댕글링 포인터를 **만드는 것**은 안전하지만, 그것을 **역참조하는 것**(여기서는 `strlen`이 읽는 것)이 미정의 동작입니다. 그래서 에러가 아니라 경고입니다.
- **원인**: `as_ptr()`의 반환 타입 `*const c_char`에는 라이프타임이 없어서, 대여 검사기는 포인터가 `CString`보다 오래 쓰이는 것을 막지 못합니다(2.5절).
- **해결**: `help:`대로 `let c_word = CString::new(word)?;`로 변수에 묶은 뒤 `c_word.as_ptr()`를 씁니다. 또는 `strlen(CString::new(word)?.as_ptr())`처럼 한 표현식 안에서 넘깁니다(예제 04의 3번 구역). 가장 좋은 방법은 포인터를 다루는 코드를 `&CStr`을 받는 래퍼 하나에 가두는 것입니다.

### 4.7 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| `let home: String = env::var_os("HOME").unwrap();` | E0308 `` expected `String`, found `OsString` `` | `.into_string()`(`Result`)이나 `.to_string_lossy().into_owned()`, 또는 `env::var`를 씀 (2.3절) |
| `println!("{}", c_string)` | E0277 `` `CString` doesn't implement `std::fmt::Display` `` | `c_string.to_string_lossy()` 또는 `{:?}`. 경로·`OsStr`도 같은 이유([ch9](../ch9-io-fs-path/README.md) 4.3절) |
| `strlen("hello".as_ptr())` | 이 환경에서는 E0308 `` expected `*const i8`, found `*const u8` ``. `c_char`가 `u8`인 플랫폼에서는 컴파일되어 미정의 동작 | `c"hello".as_ptr()` 또는 `CString` (2.5절) |
| `CStr::from_bytes_with_nul(b"hi\0")` | Clippy `manual_c_str_literals` | `c"hi"` (2.4절) |
| `c"a\0b"` | 에러 `null characters in C string literals are not supported` | 0 바이트로 나눠야 한다면 두 리터럴로 (2.4절) |
| `boxed.type_id()` (`Box<dyn Any>`) | Clippy `type_id_on_box`. 상자의 `TypeId`가 나옴 | `(*boxed).type_id()` (2.7절) |
| `describe(&boxed)` (`&dyn Any` 매개변수) | 컴파일되지만 다운캐스트가 모두 실패 | `&*boxed` (2.7절) |
| `value.downcast_ref::<str>()` | E0277 `` the size for values of type `str` cannot be known `` | 안에 든 타입 그대로 `downcast_ref::<String>()`이나 `::<&str>()` (2.7절) |
| `IntErrorKind`를 `_` 없이 `match` | E0004 `` pattern `&_` not covered `` | `_ =>` 갈래 추가 (2.11절) |
| `10i32 / NonZero::new(2i32).unwrap()` | E0277 `` cannot divide `i32` by `NonZero<i32>` `` | 부호 없는 타입에만 있음. 부호 있는 타입은 `checked_div` (2.10절) |
| `NonZero::new(1u32).unwrap() + NonZero::new(2u32).unwrap()` | E0369 `` cannot add `NonZero<u32>` to `NonZero<u32>` `` | `a.checked_add(b.get())` 또는 `a.saturating_add(b.get())` (2.10절) |

## 5. 연습 문제

### 연습 1. UTF-8이 아니어도 확장자 검사하기

`fn has_extension(name: &OsStr, ext: &str) -> bool`을 작성하세요. `name`이 `.`과 `ext`로 끝나면(ASCII 대소문자 무시) `true`입니다. 이름이 UTF-8이 아니어도 동작해야 합니다. 유닉스에서 `OsStr::from_bytes(b"caf\xE9.TXT")`와 `"txt"`로 `true`, `OsStr::new("메모.md")`와 `"txt"`로 `false`, `OsStr::new("txt")`와 `"txt"`로 `false`가 나오는지 확인하세요.

<details>
<summary>힌트</summary>

- `to_str()`을 쓰면 UTF-8이 아닌 이름에서 `None`이 되어 실패합니다. `name.as_encoded_bytes()`로 바이트를 꺼냅니다. 확장자처럼 ASCII인 부분을 찾는 데는 써도 된다고 2.3절에서 설명했습니다.
- 끝부분 비교는 `bytes.len() > ext.len()`인지 먼저 확인한 뒤 `bytes[bytes.len() - ext.len()..].eq_ignore_ascii_case(ext.as_bytes())`, 그 바로 앞 바이트가 `b'.'`인지 봅니다.
- 실제 프로그램이라면 `Path::new(name).extension()`(`Option<&OsStr>`)과 `eq_ignore_ascii_case`를 쓰는 편이 간단합니다. 두 방법의 결과를 비교해 보세요.

</details>

### 연습 2. `strcmp`의 안전한 래퍼

C 표준 라이브러리의 `int strcmp(const char *a, const char *b)`를 선언하고, `fn c_strcmp(a: &CStr, b: &CStr) -> std::cmp::Ordering`을 작성하세요. `strcmp`는 `a`가 작으면 음수, 같으면 0, 크면 양수를 돌려줍니다. `(c"apple", c"banana")`, `(c"same", c"same")`, `(c"abc", c"ab")`에 대해 `CStr`의 `cmp`(`CStr`은 `Ord`를 구현합니다)와 결과가 같은지 확인하세요.

<details>
<summary>힌트</summary>

- 선언은 `fn strcmp(a: *const c_char, b: *const c_char) -> c_int;`입니다. 반환값을 `Ordering`으로 바꿀 때는 `result.cmp(&0)`이 간단합니다. `strcmp`는 크기만 약속하고 정확한 값(예: -1)은 약속하지 않으므로 `== -1`처럼 비교하면 안 됩니다.
- `SAFETY:` 주석에는 "두 포인터 모두 0 바이트로 끝나는 살아 있는 버퍼이고, `strcmp`는 읽기만 하며 포인터를 보관하지 않는다"를 적습니다(2.5절의 `c_strlen`과 같은 논리).
- 이 환경에서는 `c"\xff"`와 `c"a"`처럼 128 이상의 바이트가 섞여도 두 결과가 같았습니다. C 표준은 `strcmp`가 바이트를 부호 없는 값으로 비교하도록 정하고 있기 때문입니다. `c_char`의 부호와는 상관없습니다.

</details>

### 연습 3. 타입 맵 넓히기

[`06_any_patterns.rs`](examples/06_any_patterns.rs)의 `TypeMap`에 세 메서드를 추가하세요.

- `fn contains<T: Any>(&self) -> bool`
- `fn remove<T: Any>(&mut self) -> Option<T>`
- `fn get_or_insert_with<T: Any>(&mut self, make: impl FnOnce() -> T) -> &mut T`

`get_or_insert_with::<Vec<String>>(Vec::new).push(..)`를 두 번 부른 뒤 `get::<Vec<String>>()`이 원소 두 개를 담고 있는지 확인하세요.

<details>
<summary>힌트</summary>

- `remove`는 `HashMap::remove`가 돌려준 `Box<dyn Any>`를 `insert`와 같은 방법(`downcast::<T>().ok().map(|b| *b)`)으로 꺼냅니다.
- `get_or_insert_with`는 `self.values.entry(TypeId::of::<T>()).or_insert_with(|| Box::new(make()))`로 칸을 얻은 뒤 `downcast_mut::<T>()`를 부릅니다. 이 칸에는 `T`만 들어가므로 실패하지 않습니다. `expect("TypeMap의 칸에는 그 타입만 들어갑니다")`처럼 불변식을 메시지로 적어 둡니다.
- `or_insert_with`의 결과는 `&mut Box<dyn Any>`입니다. 여기서 `downcast_mut`를 부르면 자동 역참조가 상자 안의 `dyn Any`를 찾아 줍니다(2.7절).

</details>

### 연습 4. 점수 평균 계산기

`fn parse_scores(line: &str) -> Result<Vec<u8>, String>`과 `fn average(scores: &[u8]) -> Option<u8>`을 작성하세요. `parse_scores`는 공백으로 나눈 각 조각을 `u8`로 읽고, 실패하면 `IntErrorKind`에 따라 `"'abc'는 숫자가 아닙니다"`, `"'300'은 너무 큽니다"` 같은 한국어 메시지를 돌려줍니다. `average`는 합을 `Saturating<u32>`로 더하지 말고(왜 안 되는지도 생각해 보세요) `u32`로 더한 뒤 개수를 `NonZero<u32>`로 바꿔 나눕니다. 빈 슬라이스는 `None`입니다.

<details>
<summary>힌트</summary>

- `match e.kind() { IntErrorKind::InvalidDigit => .., IntErrorKind::PosOverflow => .., _ => .. }`처럼 `_` 갈래를 둡니다(2.11절). `"-5"`는 `u8`에서 `InvalidDigit`입니다.
- `u8` 점수를 `u32`로 더하면 원소가 1600만 개를 넘기 전에는 넘치지 않습니다. 포화 연산으로 더하면 넘쳤을 때 조용히 틀린 평균이 나오므로, 평균에는 어울리지 않습니다. 넘침이 걱정되면 `checked_add`로 에러를 알립니다(2.9절의 선택표).
- `let count = NonZero::new(u32::try_from(scores.len()).ok()?)?;` 다음 `u8::try_from(total / count).ok()`처럼 쓰면 0 나누기 검사가 `NonZero::new` 한 곳에만 남습니다(2.10절).

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::ffi`](https://doc.rust-lang.org/std/ffi/index.html): Rust 문자열과 C 문자열의 차이(인코딩, 문자 크기, 0 바이트 종료, 중간의 0 바이트), `CString`/`CStr`과 `OsString`/`OsStr`의 역할, 플랫폼별 변환(On Unix, On Windows 절).
  - [`CString`](https://doc.rust-lang.org/std/ffi/struct.CString.html): `new`의 Errors 절, `as_ptr`의 WARNING(임시 `CString`의 댕글링 포인터 예제), `into_raw`/`from_raw`(Part 10에서 다룸).
  - [`CStr`](https://doc.rust-lang.org/std/ffi/struct.CStr.html): `from_ptr`의 Safety·Caveat 절, `from_bytes_with_nul`과 `from_bytes_until_nul`, "FFI 시그니처에 `CStr`을 쓰지 말라"는 설명.
  - [`OsString`](https://doc.rust-lang.org/std/ffi/struct.OsString.html), [`OsStr`](https://doc.rust-lang.org/std/ffi/struct.OsStr.html): 이 타입이 필요한 이유, `as_encoded_bytes`의 인코딩에 관한 주의.
  - [`std::any`](https://doc.rust-lang.org/std/any/index.html): "Smart pointers and `dyn Any`" 절(상자에 `type_id`를 부르는 함정), 값을 기록하는 예제.
  - [`Any`](https://doc.rust-lang.org/std/any/trait.Any.html), [`TypeId`](https://doc.rust-lang.org/std/any/struct.TypeId.html): 다운캐스트 메서드 목록, `TypeId`의 값·해시·순서가 버전마다 달라진다는 경고.
  - [`std::num`](https://doc.rust-lang.org/std/num/index.html), [`NonZero`](https://doc.rust-lang.org/std/num/struct.NonZero.html): Layout 절(니치와 FFI 호환 보장), `Div<NonZero<u32>> for u32`.
  - [`Wrapping`](https://doc.rust-lang.org/std/num/struct.Wrapping.html), [`Saturating`](https://doc.rust-lang.org/std/num/struct.Saturating.html): 왜 타입으로 고르는지(첫 단락), Trait Implementations 절의 연산자 목록.
- Rust By Example: [Foreign Function Interface](https://doc.rust-lang.org/rust-by-example/std_misc/ffi.html). `libm`의 복소수 함수 `csqrtf`, `ccosf`를 `#[link(name = "m")]`로 링크하고 안전한 래퍼 `cos`로 감쌉니다. 이 페이지의 `extern { .. }` 블록은 2024 에디션에서 `unsafe extern "C" { .. }`로 써야 합니다(Part 0 ch19 2.7절).
- [The Rustonomicon: Foreign Function Interface](https://doc.rust-lang.org/nomicon/ffi.html): snappy 압축 라이브러리를 링크하는 예로 안전한 인터페이스 만들기, 콜백, C에서 Rust 부르기, "nullable pointer optimization", 불투명 구조체를 설명합니다. 이 챕터가 미룬 내용으로, Part 10 `ch12-ffi-beneath-std`에서 읽습니다.
- 에러 설명: `rustc --explain E0597`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.

## Part 1을 마치며

Part 1에서는 표준 라이브러리를 모듈 단위로 읽으며, Part 0에서 맛본 타입들의 메서드와 원시 타입, 컬렉션, 반복자, 핵심 트레이트, 에러와 패닉, 입출력, 프로세스, 스레드, 네트워크, 그리고 이 챕터의 FFI 문자열·`Any`·수치 래퍼까지 넓혔습니다. 다음 Part 2(에디션)에서는 그동안 "2024 에디션에서 바뀌었다"고만 짚고 넘어간 `unsafe extern`, `static mut` 참조 금지, `env::set_var`의 unsafe화 같은 변화를 에디션이라는 제도와 함께 정리합니다. 전체 순서는 [로드맵](../../docs/roadmap.md)에서 확인할 수 있습니다.
