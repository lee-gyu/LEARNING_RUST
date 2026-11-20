# ch9. 에러 처리

프로그램은 언젠가 실패합니다. 읽으려던 파일이 없고, 사용자는 숫자 자리에 글자를 넣고, 코드에는 버그가 있습니다. 많은 언어는 이런 실패를 모두 예외(exception)라는 한 가지 장치로 다루지만, Rust에는 예외가 없습니다. 대신 실패를 두 종류로 나눠 서로 다른 도구를 씁니다. 버그처럼 더 진행하면 안 되는 상황은 `panic!`으로 프로그램을 멈추고, 파일이 없는 것처럼 일어날 수 있는 실패는 `Result<T, E>` 값으로 돌려줘 호출한 쪽이 처리하게 합니다.

앞 챕터들에서 이 챕터로 미뤄 둔 것이 많습니다. ch2의 `expect`와 `Result`, ch3·ch8에서 본 패닉 메시지와 "`get`으로 패닉 대신 `None` 받기", ch6의 `Ok`/`Err` 패턴, ch8의 `String::from_utf8`이 모두 여기서 하나로 이어집니다.

- 패키지: `p0-ch9-error-handling`
- 기준 문서: [The Book 9장 Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- 확인한 툴체인: rustc 1.100.0, Edition 2024, macOS (파일 에러 메시지는 운영체제마다 다를 수 있습니다). 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 복구할 수 없는 에러와 복구할 수 있는 에러를 구분하고, 패닉이 일어날 때 출력되는 메시지와 되감기, 종료 코드 101을 설명할 수 있으며, `RUST_BACKTRACE=1`로 얻은 백트레이스에서 패닉을 일으킨 내 코드의 위치를 찾을 수 있다.
2. `Result<T, E>`를 `match`로 처리하고 `ErrorKind`로 에러의 종류마다 다르게 대응하는 코드를 작성할 수 있으며, `unwrap`·`expect`·`unwrap_or`·`unwrap_or_else`가 각각 무엇을 하는지 설명할 수 있다.
3. 에러를 호출한 쪽으로 전파하는 함수를 `match`로 작성한 뒤 `?` 연산자로 줄일 수 있고, `?`를 쓸 수 있는 곳(함수의 반환 타입 조건, `Option`에서의 `?`)과 쓸 수 없는 곳을 설명할 수 있다.
4. `main`이 `Result<(), Box<dyn Error>>`를 돌려주도록 작성하고, `main`이 `Err`를 돌려줄 때의 출력과 종료 코드(1)를 패닉(101)과 구분할 수 있다.
5. 주어진 상황에서 패닉과 `Result` 중 무엇을 쓸지 근거를 들어 고르고, 생성자에서 값을 검증해 불변식을 지키는 타입(`Guess`)을 작성할 수 있다.

## 2. 핵심 개념

### 2.1 두 종류의 에러

| | 복구할 수 없는 에러 (unrecoverable) | 복구할 수 있는 에러 (recoverable) |
| --- | --- | --- |
| 예 | 배열 범위 밖 인덱스, 지켜져야 할 조건이 깨진 상태 | 파일이 없음, 숫자가 아닌 입력, 네트워크 끊김 |
| 원인 | 대부분 **버그** | 프로그램 바깥의 사정. 버그가 아니라 **일어날 수 있는 일** |
| Rust의 도구 | `panic!` | `Result<T, E>` |
| 결과 | 프로그램(스레드)이 멈춤 | 값으로 돌아오고, 호출한 쪽이 처리 방법을 정함 |

예외를 쓰는 언어에서는 함수 시그니처만 보고 그 함수가 실패할 수 있는지 알기 어렵습니다. Rust에서는 실패할 수 있는 함수의 반환 타입이 `Result`이므로 시그니처에 드러나고, 컴파일러는 그 값을 성공한 값처럼 쓰지 못하게 막습니다(4.4절). ch6에서 `Option`이 "값이 없을 수 있음"을 타입으로 드러내 null 문제를 해결한 것과 같은 발상입니다.

ch3과 ch8에서 이미 이 구분을 만났습니다. `v[i]`는 범위를 벗어나면 패닉하고, `v.get(i)`는 `None`을 돌려줍니다. 범위 밖 인덱스가 버그라면 인덱싱으로 즉시 멈추고, 사용자 입력처럼 일어날 수 있는 일이라면 `get`으로 받아 처리했습니다. 이 챕터의 주제는 결국 **"이 실패를 패닉으로 다룰까, 값으로 돌려줄까"** 를 고르는 일입니다(2.11절).

### 2.2 `panic!`: 프로그램을 멈추는 매크로

패닉은 두 가지 방법으로 일어납니다. 범위 밖 인덱싱처럼 패닉하는 코드를 실행하거나, `panic!` 매크로를 직접 호출하는 것입니다. `panic!`은 `println!`과 같은 서식 문자열을 받습니다.

```rust
fn ship(stock: u32, amount: u32) -> u32 {
    if amount > stock {
        panic!("출고 수량({amount})이 재고({stock})보다 많습니다");
    }
    stock - amount
}
```

[`01_panic.rs`](examples/01_panic.rs)에서 `ship(7, 20)`을 호출하면 다음이 표준 에러로 출력됩니다.

```
thread 'main' (6293600) panicked at part0-getting-started/ch9-error-handling/examples/01_panic.rs:40:9:
출고 수량(20)이 재고(7)보다 많습니다
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

- **첫 줄**: `main` 스레드가 패닉했고, 위치는 `01_panic.rs`의 40번 줄 9번 열입니다. 괄호 안의 숫자는 스레드 ID로, 실행할 때마다 달라집니다(스레드는 [ch15](../ch15-concurrency/README.md)에서 다룹니다).
- **둘째 줄**: `panic!`에 넘긴 메시지입니다.
- **셋째 줄**: 백트레이스를 보는 방법을 알려 줍니다(2.4절).

패닉으로 끝난 프로그램의 종료 코드는 **101**입니다. 패닉 메시지는 표준 에러(stderr)로, 예제의 `println!` 출력은 표준 출력(stdout)으로 나갑니다.

`unwrap`, `expect`, 인덱싱처럼 표준 라이브러리 안에서 일어나는 패닉도 위치는 그것을 **호출한 내 코드**로 나옵니다. ch3의 범위 밖 인덱스나 이 챕터 04번 예제의 `expect`가 그렇습니다. 하지만 다른 크레이트의 함수 깊숙한 곳에서 패닉하면 그 크레이트 코드의 위치가 나올 수도 있습니다. 이럴 때 2.4절의 백트레이스로 내 코드가 어디서 그 함수를 불렀는지 찾습니다.

명시적인 `panic!` 메시지는 원인을 바로 알려 준다는 장점도 있습니다. 위 코드에서 검사를 빼도 `stock - amount`가 `u32` 범위를 벗어나 패닉하지만(ch3의 오버플로), 그 메시지는 `attempt to subtract with overflow`일 뿐 무엇이 잘못됐는지는 알려 주지 않습니다.

### 2.3 되감기와 abort

패닉이 일어나면 Rust는 기본적으로 **되감기(unwinding)** 를 합니다. 호출 스택을 거꾸로 거슬러 올라가며, 각 함수가 가지고 있던 값을 drop해 정리한 뒤 프로그램을 끝냅니다. [`01_panic.rs`](examples/01_panic.rs)는 ch4의 관찰용 타입 `Noisy`로 이것을 보여 줍니다. 패닉 메시지 뒤에 `[drop]` 줄이 안쪽 함수부터 찍힙니다.

```
출고 수량(20)이 재고(7)보다 많습니다
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    [drop] ship의 출고 전표
    [drop] main의 장부
```

되감기는 할 일이 많고, 그 일을 하는 코드가 실행 파일에 들어갑니다. 그래서 정리하지 않고 즉시 끝내는 **abort**를 고를 수도 있습니다. 이때 프로그램이 쓰던 메모리는 운영체제가 회수합니다. 실행 파일을 최대한 작게 만들어야 할 때 쓰며, `Cargo.toml`의 프로필에 다음처럼 적습니다.

```toml
[profile.release]
panic = "abort"
```

- 워크스페이스에서는 프로필을 **루트 `Cargo.toml`에만** 적을 수 있습니다. 이 저장소처럼 챕터 패키지의 `Cargo.toml`에 적으면 Cargo가 `warning: profiles for the non root package will be ignored, specify profiles at the workspace root:`라고 경고하고 무시합니다.
- 01번 예제와 같은 코드를 abort로 빌드해 실행하면 패닉 메시지까지는 똑같이 나오지만 `[drop]` 줄은 나오지 않습니다. 종료 코드도 101이 아니라 abort 시그널에 따른 값(macOS의 zsh에서 134)입니다.

이 챕터에서는 개념만 알아 두면 됩니다. 패닉을 붙잡아 처리하는 방법(`catch_unwind`)과 패닉 훅은 [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)에서 다룹니다.

### 2.4 백트레이스 읽기

**백트레이스(backtrace)** 는 패닉이 일어난 지점까지 어떤 함수들이 차례로 호출됐는지를 보여 주는 목록입니다. 환경 변수 `RUST_BACKTRACE`를 `0`이 아닌 값으로 정하고 실행하면 출력됩니다.

```sh
RUST_BACKTRACE=1 cargo run -p p0-ch9-error-handling --example 01_panic
```

패닉 전의 1·2번 구역 출력은 3절과 같으므로 생략하고, 패닉 메시지부터 옮깁니다.

```
thread 'main' (6268255) panicked at part0-getting-started/ch9-error-handling/examples/01_panic.rs:40:9:
출고 수량(20)이 재고(7)보다 많습니다
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/std/src/panicking.rs:676:5
   1: core::panicking::panic_fmt
             at /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/core/src/panicking.rs:80:14
   2: 01_panic::ship
             at ./part0-getting-started/ch9-error-handling/examples/01_panic.rs:40:9
   3: 01_panic::main
             at ./part0-getting-started/ch9-error-handling/examples/01_panic.rs:57:17
   4: <fn() as core::ops::function::FnOnce<()>>::call_once
             at /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
    [drop] ship의 출고 전표
    [drop] main의 장부
```

**위에서부터 읽어 내려가다가, 내가 쓴 파일이 처음 나오는 줄**을 찾습니다.

- 0~1번은 패닉을 처리하는 표준 라이브러리 내부 함수입니다. 내 코드가 호출한 쪽, 즉 패닉이 일어난 뒤의 일이라 보통 건너뜁니다.
- 2번 `01_panic::ship`이 내 파일이 처음 나오는 줄입니다. `01_panic`은 예제 이름으로 만들어진 크레이트, `ship`은 함수 이름이고, 아랫줄 `at`이 정확한 위치(40번 줄 9번 열)입니다. 여기서부터 조사합니다.
- 3번 `01_panic::main`은 `ship`을 **호출한 쪽**입니다(57번 줄). 어떤 값으로 `ship`을 불렀는지는 여기서 확인합니다.
- 4번 아래는 `main`을 실행하기 위한 준비 코드입니다.

백트레이스의 모양은 운영체제와 Rust 버전에 따라 다르고, 스레드 ID와 `/rustc/` 뒤의 커밋 해시도 환경마다 다릅니다. `at` 줄에 파일 위치가 나오려면 **디버그 심벌**이 있어야 합니다. `cargo run`이나 `cargo build`처럼 `--release` 없이 빌드하면 디버그 심벌이 들어갑니다. 같은 예제를 `--release`로 실행하면 함수 이름만 나오고 `at` 줄은 나오지 않습니다.

### 2.5 `Result<T, E>`: 성공 또는 실패를 담는 열거형

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

- `T`는 성공했을 때 담을 값의 타입, `E`는 실패했을 때 담을 에러의 타입입니다. ch6의 `Option<T>`처럼 타입을 끼워 넣는 제네릭 문법이며 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 배웁니다.
- `Result`, `Ok`, `Err`는 `Option`, `Some`, `None`처럼 프렐류드에 들어 있어 `use` 없이 씁니다.
- 열거형이므로 ch6에서 배운 `match`, `if let`, `let...else`로 다룹니다. ch2 4단계에서 `parse`의 결과를 `Ok(num)`, `Err(_)`로 나눈 것이 바로 `Result`를 `match`한 것이었습니다.

실패할 수 있는 표준 라이브러리 함수는 `Result`를 돌려줍니다.

| 함수 | 반환 타입 | 실패하는 경우 |
| --- | --- | --- |
| `"42".parse::<i32>()` | `Result<i32, ParseIntError>` | 숫자가 아님, 빈 문자열, 범위를 넘음 |
| `String::from_utf8(bytes)` | `Result<String, FromUtf8Error>` | 바이트가 올바른 UTF-8이 아님 (ch8) |
| `File::open(path)` | `Result<File, io::Error>` | 파일이 없음, 권한이 없음 등 |

```rust
let result: Result<File, io::Error> = File::open(&path);
match result {
    Ok(file) => { /* file 사용 */ }
    Err(e) => println!("열지 못했습니다: {e}"),
}
```

에러 값은 기본 형식(`{}`)으로 출력하면 사람이 읽을 메시지가, 디버그 형식(`{:?}`)으로 출력하면 내부 구조가 나옵니다. 없는 파일을 열었을 때 macOS에서는 다음과 같습니다. 메시지와 번호는 운영체제가 정하므로 Windows에서는 다릅니다.

```
기본 형식:   No such file or directory (os error 2)
디버그 형식: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

전체 코드는 [`02_result_basics.rs`](examples/02_result_basics.rs)입니다.

#### `Result`는 버리면 경고가 납니다

`Result`를 받기만 하고 아무것도 하지 않으면 컴파일은 되지만 경고가 납니다. ch2에서 `read_line`의 `expect`를 뺐을 때 본 경고입니다. 아래는 `fs::remove_file("no_such_file.txt");` 한 줄만 쓴 코드를 컴파일한 결과입니다.

```
warning: unused `Result` that must be used
 --> unused.rs:3:5
  |
3 |     fs::remove_file("no_such_file.txt");
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: this `Result` may be an `Err` variant, which should be handled
  = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
help: use `let _ = ...` to ignore the resulting value
  |
3 |     let _ = fs::remove_file("no_such_file.txt");
  |     +++++++
```

실패할 수 있는 작업의 결과를 확인하지 않는 것은 대부분 실수이므로 `Result`에는 "반드시 사용해야 함(must use)" 표시가 붙어 있습니다. 에러가 아니라 경고인 이유는, 정말로 결과가 상관없는 경우도 있기 때문입니다. 그럴 때는 `help:`대로 `let _ = ...;`라고 써서 **일부러 버린다**는 뜻을 코드에 남깁니다. [`03_error_kind.rs`](examples/03_error_kind.rs)와 [`05_propagate.rs`](examples/05_propagate.rs)가 임시 파일을 지울 때 이렇게 씁니다. 지우지 못해도 예제의 결과에는 영향이 없기 때문입니다.

### 2.6 `ErrorKind`로 에러의 종류 나누기

파일을 열지 못한 이유는 여러 가지입니다. 이유에 따라 대응을 달리하려면 `io::Error`의 `kind()` 메서드로 에러의 종류를 꺼냅니다. `kind()`는 `io::ErrorKind` 열거형을 돌려주며, `NotFound`(없음), `PermissionDenied`(권한 없음), `AlreadyExists`(이미 있음), `NotADirectory`(디렉터리가 아님) 같은 배리언트가 있습니다.

```rust
use std::io::ErrorKind;

match File::open(path) {
    Ok(_file) => println!("있는 파일을 열었습니다"),
    Err(error) => match error.kind() {
        ErrorKind::NotFound => match File::create(path) {
            Ok(_file) => println!("파일이 없어서 새로 만들었습니다"),
            Err(e) => println!("파일이 없어서 만들려고 했지만 실패했습니다: {e}"),
        },
        other => println!("파일을 열지 못했습니다 [{other:?}]: {error}"),
    },
}
```

- 바깥 `match`는 성공인지 실패인지를, 안쪽 `match`는 실패했다면 어떤 종류인지를 나눕니다. 파일이 없는 경우만 `File::create`로 새로 만들고, 다른 종류의 실패는 알립니다. `File::create`는 파일이 없으면 만들고, 있으면 내용을 비운 채 엽니다.
- `ErrorKind`는 표준 라이브러리 문서에 "앞으로 배리언트가 늘어날 수 있다"고 표시된(`#[non_exhaustive]`) 열거형입니다. 그래서 다른 크레이트에서 `match`할 때는 **포괄 갈래(`other`나 `_`)가 반드시 있어야** 합니다. 빠뜨리면 E0004와 함께 ``note: `ErrorKind` is marked as non-exhaustive, so a wildcard `_` is necessary to match exhaustively``라는 설명이 나옵니다.

[`03_error_kind.rs`](examples/03_error_kind.rs)는 이 함수를 네 가지 경로로 호출해 각 갈래를 모두 지나가 봅니다. 덮어쓰지 않고 새 파일만 만드는 `File::create_new`가 이미 있는 파일에 대해 `AlreadyExists`로 실패하는 것도 확인합니다.

`match`가 세 겹으로 겹쳐 읽기 어렵다고 느꼈다면 맞게 본 것입니다. 이 모양은 다음 절의 `unwrap_or_else`와 클로저([ch13](../ch13-closures-iterators/README.md)), 2.8절의 `?`로 훨씬 짧아집니다.

### 2.7 `unwrap`과 `expect`: 실패하면 패닉하는 지름길

매번 `match`를 쓰는 대신, "실패하면 그냥 멈춘다"를 한 번에 쓰는 메서드가 있습니다.

| 메서드 | `Ok(v)` / `Some(v)`일 때 | `Err(e)` / `None`일 때 |
| --- | --- | --- |
| `unwrap()` | `v` | 패닉 |
| `expect("메시지")` | `v` | 주어진 메시지로 패닉 |
| `unwrap_or(기본값)` | `v` | 기본값 (패닉하지 않음) |
| `unwrap_or_else(클로저)` | `v` | 클로저가 계산한 값 (패닉하지 않음) |

`unwrap`과 `expect`가 패닉할 때의 메시지는 다음과 같습니다. 모두 실제로 실행해 확인한 것입니다.

| 코드 | 패닉 메시지 |
| --- | --- |
| `"abc".parse::<i32>().unwrap()` | ``called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }`` |
| `"abc".parse::<i32>().expect("숫자여야 합니다")` | `숫자여야 합니다: ParseIntError { kind: InvalidDigit }` |
| `v.get(5).unwrap()` (`Option`) | ``called `Option::unwrap()` on a `None` value`` |
| `v.get(5).expect("5번 요소가 있어야 합니다")` (`Option`) | `5번 요소가 있어야 합니다` |

`Result`의 `unwrap`/`expect`는 메시지 뒤에 에러 값을 디버그 형식으로 붙여 줍니다. `unwrap`은 "어딘가에서 `Err`였다"는 것만 알려 주지만, `expect`는 **왜 거기서 `Ok`여야 했는지**를 메시지로 남길 수 있습니다. 그래서 The Book은 실제로 쓰는 코드에서는 `unwrap`보다 `expect`를 권합니다. 메시지는 "~해야 합니다"처럼 기대한 조건을 적으면, 패닉했을 때 어떤 가정이 깨졌는지 바로 알 수 있습니다.

```rust
let settings = File::open(&path).expect("설정 파일 p0_ch9_04_settings.toml이 있어야 합니다");
```

```
설정 파일 p0_ch9_04_settings.toml이 있어야 합니다: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

`unwrap_or_else`에 넘기는 `|e| { ... }`는 **클로저**, 즉 그 자리에서 만드는 이름 없는 함수입니다. `Err`일 때만 에러 값 `e`를 받아 실행되고, 그 결과가 꺼낸 값을 대신합니다. 클로저는 [ch13](../ch13-closures-iterators/README.md)에서, `map`·`and_then`·`ok_or` 같은 더 많은 메서드는 [Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md)에서 다룹니다. 예제는 [`04_unwrap_expect.rs`](examples/04_unwrap_expect.rs)입니다.

### 2.8 에러 전파와 `?` 연산자

함수 안에서 일어난 에러를 그 자리에서 처리하지 않고, 함수의 반환값으로 **호출한 쪽에 넘기는 것**을 에러 전파(propagating)라고 합니다. 함수 안에서는 에러를 어떻게 처리해야 할지 모를 때가 많습니다. 사용자 이름 파일을 읽지 못했을 때, 어떤 프로그램은 손님으로 진행하고 어떤 프로그램은 종료해야 합니다. 그 판단은 호출한 쪽의 몫입니다.

[`05_propagate.rs`](examples/05_propagate.rs)는 "파일에서 사용자 이름 읽기"를 네 단계로 줄여 갑니다.

**1단계: `match`로 직접 전파합니다.** `Err`이면 `return Err(e)`로 함수를 즉시 끝냅니다.

```rust
fn read_username_match(path: &Path) -> Result<String, io::Error> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let mut username = String::new();
    match file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}
```

**2단계: `?`로 줄입니다.** `match`와 `return Err(e)`가 `?` 한 글자로 바뀝니다.

```rust
fn read_username_question(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut username = String::new();
    file.read_to_string(&mut username)?;
    Ok(username)
}
```

`식?`이 하는 일은 다음과 같습니다.

| `식`의 값 | `식?`의 결과 |
| --- | --- |
| `Ok(v)` | 식 전체의 값이 `v`가 되고, 다음 코드로 진행합니다. |
| `Err(e)` | 함수가 그 자리에서 `Err(e)`를 돌려주며 끝납니다. 이때 `e`를 함수의 에러 타입으로 **변환**합니다. |

변환은 `From`이라는 트레이트로 정의됩니다(트레이트는 ch10). 지금은 "`?`는 에러를 함수의 에러 타입으로 바꿀 수 있으면 바꿔서 돌려주고, 바꿀 수 없으면 컴파일 에러를 낸다"고 알아 두면 됩니다(4.3절). 1단계 코드에는 이런 변환이 없다는 점이 `?`와 다릅니다.

**3단계: `?`를 이어 씁니다.** `?`가 꺼낸 값에 바로 메서드를 부를 수 있습니다.

```rust
File::open(path)?.read_to_string(&mut username)?;
```

**4단계: 표준 라이브러리 함수를 씁니다.** 파일 전체를 `String`으로 읽는 일은 흔해서 `fs::read_to_string`이 이미 있습니다.

```rust
fn read_username_short(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}
```

네 함수는 같은 결과를 냅니다. 있는 파일에는 `Ok("러스타")`, 없는 파일에는 `Err(Os { code: 2, kind: NotFound, ... })`입니다.

#### `io::Result<T>`와 ch7의 이름 충돌

4단계의 `io::Result<String>`은 `Result<String, io::Error>`의 다른 이름(별칭)입니다. `std::io`의 함수들은 에러 타입이 모두 `io::Error`라서, 매번 적지 않도록 이 별칭을 둡니다. 비슷하게 `std::fmt::Result`는 `Result<(), fmt::Error>`의 별칭입니다(별칭을 만드는 `type` 문법은 [ch19](../ch19-advanced-features/README.md)).

[ch7](../ch7-modules/README.md)의 2.8절에서 "서로 다른 모듈의 같은 이름"을 다뤘는데, 이 둘이 대표적인 예입니다. 둘 다 이름이 `Result`라서 `use std::fmt::Result;`와 `use std::io::Result;`를 함께 쓰면 E0252(``the name `Result` is defined multiple times``)가 납니다. 게다가 둘 중 하나만 가져와도 그 파일에서 `Result`라는 이름이 프렐류드의 `Result<T, E>` 대신 그 별칭을 가리키게 됩니다. 그래서 관례적으로 `use std::io;`처럼 부모 모듈까지만 가져와 `io::Result<T>`, `fmt::Result`로 구분해 씁니다. ch7에서 배운 방법 1이 바로 이 관례입니다.

### 2.9 `?`를 쓸 수 있는 곳

`?`는 `Err`(또는 `None`)를 만나면 **함수에서 값을 돌려주며 빠져나가는** 연산자입니다. 그래서 그 값을 돌려줄 수 있는 함수 안에서만 쓸 수 있습니다.

| 함수의 반환 타입 | `?`를 쓸 수 있는 값 | 실패하면 돌려주는 것 |
| --- | --- | --- |
| `Result<T, E>` | `Result` | `Err(변환된 에러)` |
| `Option<T>` | `Option` | `None` |
| `()` (`main` 포함) | 쓸 수 없음 (4.1절) | |

`Option`에 쓰는 `?`는 `None`이면 함수에서 `None`을 돌려주고, `Some(v)`면 `v`를 꺼냅니다. ch6의 `let...else { return None; }`을 한 글자로 줄인 것입니다.

```rust
fn last_char_of_first_line(text: &str) -> Option<char> {
    text.lines().next()?.chars().last()
}
```

`lines()`는 줄을 하나씩 내주는 반복자, `next()`는 다음 줄을 `Option<&str>`로 꺼내는 메서드입니다(반복자는 ch13). 첫 줄이 없으면 `next()?`에서, 첫 줄이 비어 있으면 `last()`에서 `None`이 됩니다.

**`?`는 `Result`와 `Option`을 저절로 바꿔 주지 않습니다.** `Result`를 돌려주는 함수 안에서 `Option`에 `?`를 쓰거나, 그 반대로 쓰면 컴파일 에러입니다(4.2절). 섞어 써야 하면 직접 바꿉니다.

| 바꾸는 방향 | 메서드 | 결과 |
| --- | --- | --- |
| `Result` → `Option` | `result.ok()` | `Ok(v)` → `Some(v)`, `Err(_)` → `None` (에러 정보는 버려짐) |
| `Option` → `Result` | `option.ok_or(에러값)` | `Some(v)` → `Ok(v)`, `None` → `Err(에러값)` |

[`06_question_mark_option.rs`](examples/06_question_mark_option.rs)는 ch6의 `"port=8080"` 설정 줄을 `split_once('=')?`와 `parse().ok()?`로 처리합니다.

컴파일러 메시지에는 `` (or another type that implements `FromResidual`) ``라는 말이 덧붙는데, `Result`와 `Option` 말고도 `?`를 받는 타입을 만들 수 있다는 뜻입니다. 지금은 두 가지만 알면 충분합니다.

### 2.10 `main`이 `Result`를 돌려줄 때

지금까지의 `main`은 반환 타입이 `()`라서 `?`를 쓸 수 없었습니다. `main`도 `Result`를 돌려주게 만들 수 있습니다.

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let count = read_count(&good)?;
    println!("count = {count}");
    Ok(())
}
```

- 성공했을 때 돌려줄 의미 있는 값이 없으므로 `Ok`에 유닛 `()`를 담습니다. 마지막 줄의 `Ok(())`를 빠뜨리면 반환 타입이 맞지 않아 컴파일되지 않습니다.
- `Box<dyn Error>`는 **"어떤 종류의 에러든 담을 수 있는 타입"** 입니다. `?`가 `io::Error`든 `ParseIntError`든 이 타입으로 바꿔 주므로, 에러 타입이 여러 가지인 함수에서 편리합니다. `dyn`과 트레이트 객체는 [ch17](../ch17-trait-objects-oop/README.md)에서, 에러 타입을 직접 설계하고 `From`으로 변환하는 방법은 [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)에서 다룹니다.

`main`이 `Err(e)`를 돌려주면 Rust가 표준 에러에 `Error: `와 `e`의 디버그 형식을 출력하고, **종료 코드 1**로 프로그램을 끝냅니다.

```
Error: ParseIntError { kind: InvalidDigit }
```

| 끝나는 방식 | 종료 코드 (이 저장소에서 확인) |
| --- | --- |
| `main`이 정상적으로 끝남, `Ok(())` 반환 | 0 |
| `main`이 `Err`를 반환 | 1 |
| 패닉 (되감기) | 101 |
| 패닉 (`panic = "abort"`) | 시그널에 따라 다름 (macOS zsh에서 134) |

종료 코드는 셸 스크립트나 다른 프로그램이 "이 프로그램이 성공했는가"를 판단하는 근거입니다. 사용자에게 보여 줄 에러 메시지를 직접 다듬어 출력하고 원하는 종료 코드로 끝내는 방법은 [ch12](../ch12-io-project-minigrep/README.md)에서 다룹니다. 예제는 [`07_main_result.rs`](examples/07_main_result.rs)입니다.

### 2.11 패닉할까, `Result`를 돌려줄까

**기본은 `Result`입니다.** `Result`를 돌려주면 호출한 쪽이 재시도할지, 기본값을 쓸지, 멈출지를 고를 수 있습니다. 패닉하면 그 선택권을 호출한 쪽에서 빼앗게 됩니다. 반대로 `Result`를 받은 쪽은 언제든 `expect`로 패닉을 고를 수 있습니다.

**패닉이 맞는 것은 코드가 잘못된 상태(bad state)에 빠졌을 때입니다.** 가정, 보장, 약속(계약), 불변식이 깨진 상태를 말하며, 다음 조건에 해당하면 패닉을 고려합니다.

- 그 상태가 **예상하지 못한** 일입니다. 사용자가 형식을 틀리게 입력하는 것처럼 가끔 일어나는 일이 아닙니다.
- 이후의 코드가 **그 상태가 올바르다는 데 기대고** 있어서, 단계마다 다시 검사할 수 없습니다.
- 그 조건을 **타입으로 표현할 방법이 마땅치 않습니다.**

| 상황 | 고를 것 | 이유 |
| --- | --- | --- |
| 사용자 입력, 파일 내용, 네트워크 응답이 잘못됨 | `Result` | 일어날 수 있는 일입니다. 호출한 쪽이 다시 묻거나 안내합니다. |
| 함수의 약속(계약)을 호출한 쪽이 어김 | 패닉 | 호출한 쪽의 버그입니다. 고칠 곳은 호출한 코드입니다. |
| 계속 진행하면 위험함 (잘못된 메모리 읽기 등) | 패닉 | 표준 라이브러리가 범위 밖 인덱스에서 패닉하는 이유입니다(ch3). |
| 예제, 빠르게 만들어 보는 프로토타입 | `unwrap`/`expect` | 에러 처리는 나중에 채울 자리로 남겨 둡니다. 찾아서 고치기 쉽습니다. |
| 테스트 | `unwrap`/`expect` | 실패하면 테스트가 실패해야 합니다([ch11](../ch11-testing/README.md)). |
| 코드에 직접 적은 값처럼 **절대 실패할 수 없다고 내가 아는** 경우 | `expect` | 컴파일러는 모르지만 나는 압니다. 그 근거를 메시지에 적습니다. |

마지막 경우의 예가 04번 예제의 `"7".parse().expect("코드에 직접 적은 숫자 문자열이므로 항상 변환됩니다")`입니다. `parse`는 여전히 `Result`를 돌려주지만, 이 문자열은 바뀌지 않으므로 실패할 수 없습니다. 메시지에 근거를 적어 두면, 나중에 이 값을 사용자 입력으로 바꿀 때 이 `expect`도 고쳐야 한다는 것을 떠올릴 수 있습니다.

**타입으로 검사를 없앨 수도 있습니다.** 매개변수가 `Option`이 아닌 타입이면 값이 반드시 있으므로 "없을 때"를 검사할 필요가 없고, `u32`로 받으면 음수일 수 없습니다. 이 생각을 한 걸음 더 밀고 나간 것이 다음 절의 검증 타입입니다.

### 2.12 검증 타입: 생성자에서 한 번만 검사하기

ch2의 숫자 맞히기 게임은 입력이 숫자인지만 확인했을 뿐 1~100 사이인지는 확인하지 않았습니다. 범위를 확인하는 `if`를 추측값을 쓰는 모든 함수에 넣을 수도 있지만, 번거롭고 하나라도 빠뜨리기 쉽습니다. 대신 **1~100인 값만 담을 수 있는 타입**을 만듭니다.

```rust
mod guessing {
    pub struct Guess {
        value: i32, // 비공개 필드
    }

    impl Guess {
        pub fn new(value: i32) -> Guess {
            if !(1..=100).contains(&value) {
                panic!("Guess 값은 1~100이어야 합니다. 받은 값: {value}");
            }
            Guess { value }
        }

        pub fn value(&self) -> i32 {
            self.value
        }
    }
}
```

세 가지가 함께 불변식("모든 `Guess`의 값은 1~100")을 지킵니다.

1. **비공개 필드**: `value`에 `pub`이 없으므로 모듈 밖에서 `Guess { value: 500 }`처럼 직접 만들 수 없습니다. 시도하면 E0451(``field `value` of struct `Guess` is private``)입니다(ch7).
2. **검사하는 생성자**: `Guess`를 만드는 유일한 통로인 `new`가 범위를 검사합니다.
3. **게터만 있음**: `value()`로 읽을 수만 있고 값을 바꾸는 메서드가 없으므로, 만들어진 뒤에 범위를 벗어날 방법도 없습니다.

그 결과 `fn compare(guess: &Guess, secret: i32)`처럼 `Guess`를 받는 함수는 범위를 다시 검사하지 않습니다. 시그니처의 타입이 검사가 끝났음을 보장하기 때문입니다. `(1..=100).contains(&value)`는 "`value`가 1 이상 100 이하인가"입니다. `value < 1 || value > 100`이라고 써도 같지만 Clippy는 범위로 쓰기를 권합니다(`clippy::manual_range_contains`).

`new`가 범위 밖 값에 패닉하는 것은 2.11절의 "약속을 어긴 호출"에 해당하기 때문입니다. `Guess::new(200)`이라고 쓴 코드는 고쳐야 할 버그입니다. 하지만 사용자가 입력한 `"150"`은 버그가 아니라 늘 일어나는 일입니다. 그래서 [`08_guess.rs`](examples/08_guess.rs)에는 입력 문자열을 받아 `Result`를 돌려주는 `Guess::parse`도 함께 두었습니다.

```rust
pub fn parse(input: &str) -> Result<Guess, GuessError> {
    let value: i32 = match input.trim().parse() {
        Ok(n) => n,
        Err(_) => return Err(GuessError::NotANumber(input.to_string())),
    };
    if !(1..=100).contains(&value) {
        return Err(GuessError::OutOfRange(value));
    }
    Ok(Guess::new(value)) // 범위를 방금 확인했으므로 패닉하지 않습니다.
}
```

에러 타입 `GuessError`는 실패 이유를 배리언트로 나눈 열거형(ch6)이라, 호출한 쪽이 `match`로 이유마다 다른 안내를 할 수 있습니다. 같은 검사를 두고 **누가, 어떤 값으로 부르는가**에 따라 패닉과 `Result`가 갈린다는 점을 두 함수로 비교해 보세요.

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다. 출력에서 cargo가 찍는 `Compiling`, `Finished`, `Running` 줄은 생략했습니다. 파일을 다루는 예제는 `std::env::temp_dir()`이 돌려주는 임시 디렉터리에 `p0_ch9_`로 시작하는 파일을 만들고 끝날 때 지우므로, 어느 디렉터리에서 실행해도 결과가 같습니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_panic.rs`](examples/01_panic.rs) | `panic!`, 패닉 메시지, 되감기 중의 drop, 백트레이스 (**일부러 패닉, 종료 코드 101**) | 1 |
| [`02_result_basics.rs`](examples/02_result_basics.rs) | `Result`를 돌려주는 함수들(`parse`, `String::from_utf8`, `File::open`)과 `match` | 2 |
| [`03_error_kind.rs`](examples/03_error_kind.rs) | `ErrorKind`로 나누기: 없으면 만들기, `AlreadyExists` | 2 |
| [`04_unwrap_expect.rs`](examples/04_unwrap_expect.rs) | `unwrap`, `expect`, `unwrap_or`, `unwrap_or_else` (**일부러 패닉, 종료 코드 101**) | 2, 5 |
| [`05_propagate.rs`](examples/05_propagate.rs) | `match`로 전파 → `?` → `?` 연결 → `fs::read_to_string` | 3 |
| [`06_question_mark_option.rs`](examples/06_question_mark_option.rs) | `Option`에 쓰는 `?`, `let...else`와 비교, `ok()`로 바꿔 쓰기 | 3 |
| [`07_main_result.rs`](examples/07_main_result.rs) | `main`의 `Result` 반환, `Box<dyn Error>` (**일부러 `Err` 반환, 종료 코드 1**) | 4 |
| [`08_guess.rs`](examples/08_guess.rs) | 검증 타입 `Guess`, 패닉하는 `new`와 `Result`를 돌려주는 `parse` | 5 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 4개 (4절) | 2, 3, 4 |

### 01_panic — `panic!`과 되감기 (일부러 패닉)

`ship`은 재고보다 많이 출고하려 하면 패닉합니다. 1번 구역은 정상 호출이라 `ship`이 끝날 때 출고 전표가 drop되고, 2번 구역에서 패닉한 뒤에는 되감기가 `ship`과 `main`의 값을 차례로 drop합니다. **이 예제는 일부러 패닉하며 끝납니다.**

```sh
cargo run -p p0-ch9-error-handling --example 01_panic
echo $?   # 종료 코드 확인
```

```
1. 조건을 지키는 동안은 평소처럼 실행됩니다
    [drop] ship의 출고 전표
    남은 재고: 7
2. 조건을 어기면 panic!이 프로그램을 멈춥니다

thread 'main' (6293600) panicked at part0-getting-started/ch9-error-handling/examples/01_panic.rs:40:9:
출고 수량(20)이 재고(7)보다 많습니다
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    [drop] ship의 출고 전표
    [drop] main의 장부
101
```

`thread 'main'` 뒤 괄호 안의 스레드 ID는 실행할 때마다 달라집니다. 마지막 `101`은 `echo $?`가 출력한 종료 코드입니다. 패닉 메시지 세 줄은 표준 에러로, 나머지는 표준 출력으로 나가므로 `2>/dev/null`을 붙여 실행하면 프로그램 출력 중 패닉 메시지만 사라집니다(cargo의 `Compiling` 같은 줄도 표준 에러라서 함께 사라집니다). `RUST_BACKTRACE=1`을 붙여 실행한 결과와 읽는 법은 2.4절에 있습니다.

### 02_result_basics — `Result`를 돌려주는 함수들

1번 구역은 `Result<i32, ParseIntError>`라고 타입을 적어 `T`와 `E`가 무엇인지 보여 줍니다. 2번 구역은 `Result` 자체를 디버그 형식으로 출력합니다. 3번 구역은 ch8에서 미룬 `String::from_utf8`로, '녕'의 마지막 바이트가 빠진 바이트 벡터를 넣어 봅니다. 4번 구역의 에러 메시지는 운영체제에 따라 다릅니다.

```sh
cargo run -p p0-ch9-error-handling --example 02_result_basics
```

```
1. parse는 Result<i32, ParseIntError>를 돌려줍니다
    "42" → Ok, 값: 42
    "사십이" → Err, 이유: invalid digit found in string
    "" → Err, 이유: cannot parse integer from empty string
2. Result를 그대로 {:?}로 출력하면 배리언트와 안의 값이 함께 보입니다
    Ok(200)
    Err(ParseIntError { kind: PosOverflow })
3. String::from_utf8은 바이트가 올바른 UTF-8일 때만 String을 만듭니다
    Ok: 안녕
    Err: incomplete utf-8 byte sequence from index 3
4. File::open은 Result<File, io::Error>를 돌려줍니다
    기본 형식:   No such file or directory (os error 2)
    디버그 형식: Os { code: 2, kind: NotFound, message: "No such file or directory" }
    종류(kind):  NotFound
```

`from_utf8`의 에러 메시지는 3번 바이트에서 시작한 글자('녕')가 끝나기 전에 바이트가 끝났다는 뜻입니다.

### 03_error_kind — 파일 열기와 `ErrorKind`

2.6절의 `open_or_create`를 네 가지 경로로 호출합니다. 1·2번은 없는 파일을 만들고 다시 여는 정상 흐름, 3번은 없는 디렉터리 안의 경로라 `File::create`까지 실패하는 경우, 4번은 파일 이름 뒤에 `/child.txt`를 붙여 파일을 디렉터리처럼 쓴 경로라 `NotFound`가 아닌 에러가 나는 경우입니다. 5번은 `File::create_new`의 `AlreadyExists`입니다. 3·4번의 메시지와 4번의 에러 종류는 macOS에서 확인한 것으로, Windows에서는 다를 수 있습니다.

```sh
cargo run -p p0-ch9-error-handling --example 03_error_kind
```

```
1. 처음에는 파일이 없으므로 NotFound 갈래에서 새로 만듭니다
    파일이 없어서 새로 만들었습니다
2. 같은 경로를 다시 열면 이번에는 Ok입니다
    있는 파일을 열었습니다
3. 없는 디렉터리 안의 파일은 열 수도, 만들 수도 없습니다
    파일이 없어서 만들려고 했지만 실패했습니다: No such file or directory (os error 2)
4. NotFound가 아닌 에러는 포괄 패턴 갈래로 갑니다
    파일을 열지 못했습니다 [NotADirectory]: Not a directory (os error 20)
5. File::create_new는 이미 있는 파일을 덮어쓰지 않고 AlreadyExists로 실패합니다
    이미 있는 파일이라 만들지 않았습니다
```

### 04_unwrap_expect — `unwrap`, `expect`와 패닉하지 않는 대안 (일부러 패닉)

1·2번은 `Ok`에서 값을 꺼내고, 3·4번은 실패해도 패닉하지 않고 값을 정합니다. 5번은 없는 설정 파일을 `expect`로 열어 **일부러 패닉하며 끝납니다.** 패닉 메시지가 "`expect`에 넘긴 메시지: 에러의 디버그 형식" 모양이라는 점과, 위치가 표준 라이브러리 안이 아니라 `expect`를 호출한 48번 줄이라는 점을 확인하세요.

```sh
cargo run -p p0-ch9-error-handling --example 04_unwrap_expect
echo $?   # 종료 코드 확인
```

```
1. Ok이면 unwrap은 안의 값을 꺼냅니다
    a = 42
2. expect는 unwrap과 같되, 패닉할 때 보여 줄 메시지를 직접 정합니다
    b = 7
3. 실패하면 기본값을 쓰려면 unwrap_or를 씁니다 (패닉하지 않습니다)
    c = 0
4. 에러를 보고 할 일을 정하려면 unwrap_or_else를 씁니다 (패닉하지 않습니다)
    변환 실패(invalid digit found in string), 대신 -1을 씁니다
    d = -1
5. 프로그램이 동작하는 데 꼭 필요한 것이 없으면 expect로 멈춥니다

thread 'main' (6293710) panicked at part0-getting-started/ch9-error-handling/examples/04_unwrap_expect.rs:48:38:
설정 파일 p0_ch9_04_settings.toml이 있어야 합니다: Os { code: 2, kind: NotFound, message: "No such file or directory" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
101
```

스레드 ID는 실행할 때마다 달라지고, `Os { ... }` 부분은 운영체제에 따라 다릅니다.

### 05_propagate — `match`에서 `?`까지

2.8절의 네 함수를 있는 파일과 없는 파일로 각각 호출해 결과가 같다는 것을 확인합니다. 1단계 함수는 Clippy가 `?`로 바꾸라고 제안하는 코드(`clippy::question_mark`)지만, 비교하려고 일부러 남긴 것이라 `#[expect(clippy::question_mark)]`를 달았습니다. 5번 구역은 같은 에러를 호출한 쪽에서 "손님으로 진행"으로 처리합니다. 준비 단계의 `fs::write`는 실패하면 예제를 진행할 수 없으므로 `expect`를 씁니다.

```sh
cargo run -p p0-ch9-error-handling --example 05_propagate
```

```
1. match로 직접 전파
    있는 파일: Ok("러스타")
    없는 파일: Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
2. ? 연산자
    있는 파일: Ok("러스타")
    없는 파일: Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
3. ?를 이어서 쓰기
    있는 파일: Ok("러스타")
    없는 파일: Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
4. fs::read_to_string
    있는 파일: Ok("러스타")
    없는 파일: Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
5. 전파된 에러를 어떻게 처리할지는 호출한 쪽이 정합니다
    안녕하세요, 러스타님!
    사용자 이름을 읽지 못해 손님으로 진행합니다 (No such file or directory (os error 2))
```

### 06_question_mark_option — `Option`에 쓰는 `?`

1번 구역은 같은 함수를 `?` 버전과 ch6의 `let...else` 버전으로 나란히 실행합니다(`let...else` 버전에도 `#[expect(clippy::question_mark)]`를 달았습니다). 2번 구역은 슬라이스의 앞 두 요소를 `first()?`, `get(1)?`로 꺼냅니다. `first()`는 `get(0)`과 같은 일을 하는 메서드로, Clippy가 `get(0)` 대신 권합니다(`clippy::get_first`). 3번 구역의 `"port = 443"`은 `trim()` 덕분에 공백이 있어도 읽힙니다.

```sh
cargo run -p p0-ch9-error-handling --example 06_question_mark_option
```

```
1. 첫 줄의 마지막 글자
    "안녕하세요\n반갑습니다" → ? 버전: Some('요'), let...else 버전: Some('요')
    "" → ? 버전: None, let...else 버전: None
    "\n둘째 줄" → ? 버전: None, let...else 버전: None
2. 요소가 모자라면 첫 번째 None에서 멈춥니다
    [10, 20, 30] → Some(30)
    [10]         → None
3. Result는 ok()로 Option으로 바꾼 뒤 ?를 씁니다
    "port=8080" → Some(8080)
    "port = 443" → Some(443)
    "host=localhost" → None
    "port=abc" → None
    "# 주석" → None
```

3번 구역은 `"host=localhost"`(키가 다름), `"port=abc"`(숫자가 아님), `"# 주석"`(`=`가 없음)이 모두 `None`으로 나와서 이유를 구분할 수 없습니다. 이것이 `ok()`로 에러 정보를 버린 대가입니다. 이유를 알려야 한다면 `Result`를 돌려주는 함수로 만들어야 합니다(연습 2).

### 07_main_result — `main`이 `Result`를 돌려줄 때 (일부러 `Err` 반환)

`read_count`는 파일 읽기(`io::Error`)와 숫자 변환(`ParseIntError`) 두 곳에서 실패할 수 있고, 두 에러를 `Box<dyn Error>` 하나로 돌려줍니다. 3번 구역에서 `main`이 `?`로 `Err`를 돌려주며 **일부러 실패로 끝납니다.** `?`로 빠져나가기 전에 임시 파일을 지우려고, `read_count`의 결과를 변수에 담아 두었다가 정리를 마친 뒤에 `?`를 씁니다. `Result`도 값이므로 이렇게 나중에 처리할 수 있습니다.

```sh
cargo run -p p0-ch9-error-handling --example 07_main_result
echo $?   # 종료 코드 확인
```

```
1. 서로 다른 두 종류의 에러를 Box<dyn Error> 하나로 돌려줍니다
    good    → Ok(12)
    bad     → Err(ParseIntError { kind: InvalidDigit })
    missing → Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
2. main 안에서도 ?로 값을 꺼낼 수 있습니다
    count = 12
3. ?가 Err를 만나면 main이 그 Err를 돌려주며 끝납니다
Error: ParseIntError { kind: InvalidDigit }
1
```

`Error: ...` 줄은 표준 에러로 나갑니다. 종료 코드는 패닉의 101이 아니라 1입니다.

### 08_guess — 검증 타입 `Guess`

1번 구역은 사용자 입력을 흉내 낸 문자열들을 `Guess::parse`로 받습니다. `" 7 "`은 `trim()` 덕분에 7로 읽히고, `"-5"`는 `i32`로 읽은 뒤 범위 검사에서 걸립니다(ch2처럼 `u32`로 읽었다면 "숫자가 아님"이 됐을 것입니다). 2번 구역은 코드에 직접 적은 값을 `Guess::new`로 만듭니다.

```sh
cargo run -p p0-ch9-error-handling --example 08_guess
```

```
1. 사용자 입력은 parse로 받습니다. 잘못된 입력은 Err로 알려 다시 묻게 합니다
    50: 너무 큽니다
    7: 너무 작습니다
    "abc": 숫자가 아닙니다
    150: 1~100 사이로 입력하세요
    -5: 1~100 사이로 입력하세요
    42: 정답입니다!
2. 코드에 직접 적은 값은 new로 만듭니다
    Guess { value: 50 }
    50: 너무 큽니다
```

파일 끝의 주석 두 개를 하나씩 풀어 보세요. `let _wrong = Guess::new(200);`은 컴파일되지만 실행하면 `Guess 값은 1~100이어야 합니다. 받은 값: 200`으로 패닉하고 종료 코드 101로 끝납니다. `let _fake = Guess { value: 500 };`은 E0451(``field `value` of struct `Guess` is private``)로 컴파일되지 않습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch9-error-handling
rustc --edition 2024 compile_fail/e0277_question_mark_in_unit_main.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요. 에러 코드의 자세한 설명은 `rustc --explain E0277`처럼 확인할 수 있습니다.

이 챕터의 에러 셋은 같은 E0277이지만 첫 줄의 설명이 서로 다릅니다. E0277은 "이 타입에는 필요한 기능(트레이트)이 없다"는 넓은 뜻의 코드라서, **에러 코드보다 첫 줄의 문장**을 읽어야 무엇이 문제인지 알 수 있습니다.

### 4.1 `()`를 돌려주는 함수에서 `?` 사용 — E0277

파일: [`compile_fail/e0277_question_mark_in_unit_main.rs`](compile_fail/e0277_question_mark_in_unit_main.rs)

```
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
  --> compile_fail/e0277_question_mark_in_unit_main.rs:10:48
   |
 9 | fn main() {
   | --------- this function should return `Result` or `Option` to accept `?`
10 |     let greeting_file = File::open("hello.txt")?;
   |                                                ^ cannot use the `?` operator in a function that returns `()`
   |
help: consider adding return type
   |
 9 ~ fn main() -> Result<(), Box<dyn std::error::Error>> {
10 |     let greeting_file = File::open("hello.txt")?;
11 |
12 |     println!("{greeting_file:?}");
13 +     Ok(())
   |
```

- **원인**: `?`는 `Err`를 만나면 그 `Err`를 돌려주며 함수를 끝냅니다. 그런데 `main`의 반환 타입이 `()`라서 `Err`를 돌려줄 자리가 없습니다(2.9절).
- **해결**: `help:`가 정확한 답을 줍니다. `main`의 반환 타입을 `Result<(), Box<dyn std::error::Error>>`로 바꾸고 마지막에 `Ok(())`를 추가합니다(2.10절). 이렇게 고친 코드를 `hello.txt`가 없는 디렉터리에서 실행하면 `Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }`를 출력하고 종료 코드 1로 끝납니다. 이 자리에서 에러를 처리하고 싶다면 `?` 대신 `match`나 `expect`를 씁니다.
- Rustlings `errors3.rs`가 바로 이 문제입니다.

### 4.2 `Result`를 돌려주는 함수에서 `Option`에 `?` 사용 — E0277

파일: [`compile_fail/e0277_option_question_in_result_fn.rs`](compile_fail/e0277_option_question_in_result_fn.rs)

```
error[E0277]: the `?` operator can only be used on `Result`s, not `Option`s, in a function that returns `Result`
  --> compile_fail/e0277_option_question_in_result_fn.rs:11:41
   |
10 | fn first_number(text: &str) -> Result<i32, ParseIntError> {
   | --------------------------------------------------------- this function returns a `Result`
11 |     let first_line = text.lines().next()?;
   |                                         ^ use `.ok_or(...)?` to provide an error compatible with `Result<i32, ParseIntError>`
```

- **원인**: `next()`는 `Option<&str>`을 돌려줍니다. `None`일 때 함수가 돌려줄 `Err` 값이 정해져 있지 않으므로 `?`를 쓸 수 없습니다. `?`는 `Option`과 `Result`를 저절로 바꾸지 않습니다.
- **해결**: 컴파일러의 제안대로 `.ok_or(에러값)?`로 `None`을 `Err`로 바꿉니다. 다만 이 함수의 에러 타입 `ParseIntError`는 직접 만들 수 없는 타입이라, 이 경우에는 에러 타입을 `Box<dyn Error>`로 넓히는 것이 간단합니다. 그러면 문자열도 에러로 쓸 수 있습니다.

```rust
fn first_number(text: &str) -> Result<i32, Box<dyn Error>> {
    let first_line = text.lines().next().ok_or("빈 문자열입니다")?;
    let n: i32 = first_line.trim().parse()?;
    Ok(n)
}
// first_number("42\n7") → Ok(42), first_number("") → Err("빈 문자열입니다")
```

- **반대 방향**: `Option`을 돌려주는 함수에서 `Result`에 `?`를 쓰면 ``the `?` operator can only be used on `Option`s, not `Result`s, in a function that returns `Option` ``이 나고, 컴파일러가 `` use `.ok()?` ``를 제안합니다. [`06_question_mark_option.rs`](examples/06_question_mark_option.rs)의 `port_of`가 그렇게 고친 모양입니다.

### 4.3 에러 타입이 달라 `?`가 변환하지 못함 — E0277

파일: [`compile_fail/e0277_question_mark_error_conversion.rs`](compile_fail/e0277_question_mark_error_conversion.rs)

```
error[E0277]: `?` couldn't convert the error to `std::io::Error`
  --> compile_fail/e0277_question_mark_error_conversion.rs:13:41
   |
11 | fn read_count(path: &str) -> Result<u32, io::Error> {
   |                              ---------------------- expected `std::io::Error` because of this
12 |     let text = fs::read_to_string(path)?;
13 |     let count: u32 = text.trim().parse()?;
   |                                  -------^ the trait `From<ParseIntError>` is not implemented for `std::io::Error`
   |                                  |
   |                                  this can't be annotated with `?` because it has type `Result<_, ParseIntError>`
   |
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
```

(이 아래에 `io::Error`로 바꿀 수 있는 타입 목록이 이어집니다.)

- **읽는 법**: 함수의 에러 타입은 `io::Error`인데(`expected ... because of this`), `parse`가 돌려준 에러는 `ParseIntError`입니다. `note:`가 2.8절에서 말한 "`?`의 변환"을 설명합니다. `ParseIntError`를 `io::Error`로 바꾸는 방법(`From<ParseIntError>`)이 정의되어 있지 않아 변환할 수 없다는 뜻입니다.
- **원인**: 한 함수 안에서 서로 다른 종류의 에러에 `?`를 썼습니다. 12번 줄의 `fs::read_to_string`은 `io::Error`라서 문제가 없습니다.
- **해결**: 에러 타입을 두 에러를 모두 담을 수 있는 `Box<dyn Error>`로 바꿉니다(`use std::error::Error;`). [`07_main_result.rs`](examples/07_main_result.rs)의 `read_count`가 이렇게 고친 코드입니다. 에러의 종류를 호출한 쪽이 `match`로 구분해야 한다면 두 경우를 배리언트로 가진 에러 열거형을 직접 만들고 `From`을 구현하는데, 이 방법은 [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)에서 다룹니다.

### 4.4 `Result`를 성공한 값처럼 사용 — E0308

파일: [`compile_fail/e0308_result_as_value.rs`](compile_fail/e0308_result_as_value.rs)

```
error[E0308]: mismatched types
  --> compile_fail/e0308_result_as_value.rs:10:22
   |
10 |     let guess: u32 = input.trim().parse();
   |                ---   ^^^^^^^^^^^^^^^^^^^^ expected `u32`, found `Result<_, _>`
   |                |
   |                expected due to this
   |
   = note: expected type `u32`
              found enum `Result<_, _>`
help: consider using `Result::expect` to unwrap the `Result<_, _>` value, panicking if the value is a `Result::Err`
   |
10 |     let guess: u32 = input.trim().parse().expect("REASON");
   |                                          +++++++++++++++++
```

- **원인**: `parse`가 돌려주는 것은 `u32`가 아니라 `Result<u32, ParseIntError>`입니다. 실패했을 때 어떻게 할지 정하지 않고 `u32`처럼 쓸 수는 없습니다. ch6에서 `Option<i8>`을 `i8`처럼 더하지 못했던 것과 같습니다.
- **해결**: 실패했을 때 할 일을 정합니다. 2.11절의 기준대로, 실패할 수 없는 값이면 `help:`처럼 `expect("이유")`, 사용자 입력이면 `match`로 처리하거나 함수가 `Result`를 돌려주게 하고 `?`로 전파합니다. `help:`의 `"REASON"`은 자리만 채운 것이므로 실제 이유로 바꿔 씁니다.
- **같은 실수, 다른 모양**: `File::open("hello.txt").read_to_string(&mut s)`처럼 `Result`에 바로 `File`의 메서드를 부르면 E0599(``no method named `read_to_string` found for enum `Result<T, E>` in the current scope``)입니다. 컴파일러가 ``note: the method `read_to_string` exists on the type `File` ``라고 알려 주며 역시 `expect`를 제안합니다. 함수가 `Result`를 돌려준다면 `File::open("hello.txt")?.read_to_string(&mut s)`처럼 `?`를 끼우는 것이 3단계(2.8절)의 모양입니다.

### 4.5 그 밖의 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| `Result`를 받고 아무것도 하지 않음 | 경고 ``unused `Result` that must be used`` | 처리하거나, 정말 상관없으면 `let _ = ...;` (2.5절) |
| `ErrorKind`를 `match`하며 포괄 갈래를 빠뜨림 | E0004 ``non-exhaustive patterns: `_` not covered`` | `other =>`나 `_ =>` 갈래를 추가합니다(2.6절). |
| `use std::fmt::Result;`와 `use std::io::Result;`를 함께 씀 | E0252 ``the name `Result` is defined multiple times`` | `use std::{fmt, io};`로 가져와 `fmt::Result`, `io::Result`로 씁니다(2.8절). |
| 모듈 밖에서 `Guess { value: 500 }` | E0451 ``field `value` of struct `Guess` is private`` | `Guess::new`나 `Guess::parse`로 만듭니다(2.12절). |
| `fn main() -> Result<(), Box<dyn Error>>`의 끝에 `Ok(())`를 빠뜨림 | E0308 (``expected `Result<(), Box<dyn Error>>`, found `()` ``) | 함수 끝에 `Ok(())`를 씁니다. |
| 챕터 패키지의 `Cargo.toml`에 `[profile.release] panic = "abort"` | 경고 `profiles for the non root package will be ignored` | 프로필은 워크스페이스 루트 `Cargo.toml`에 적습니다(2.3절). |

## 5. 연습 문제

### 연습 1. 쉼표로 구분된 수의 합

`fn sum_csv(line: &str) -> Result<i32, ParseIntError>`를 작성하세요. `"10, 20, 30"`이면 `Ok(60)`을 돌려주고, 숫자가 아닌 조각이 하나라도 있으면 그 조각의 `parse` 에러를 그대로 돌려줍니다. `?`를 쓰고, `match`는 쓰지 마세요.

```rust
sum_csv("10, 20, 30") // Ok(60)
sum_csv("7")          // Ok(7)
sum_csv("1, 두, 3")   // Err(ParseIntError { kind: InvalidDigit })
```

<details>
<summary>힌트</summary>

`line.split(',')`는 `split_whitespace()`(ch8)처럼 쉼표를 기준으로 나눈 조각(`&str`)을 하나씩 내줍니다. `for part in line.split(',')` 안에서 `let n: i32 = part.trim().parse()?;`로 숫자를 꺼내 합계에 더하고, 반복이 끝나면 `Ok(sum)`을 돌려줍니다. `?`는 반복문 안에서도 함수 전체를 빠져나가므로, 첫 번째 잘못된 조각에서 바로 `Err`가 돌아옵니다. `ParseIntError`는 `use std::num::ParseIntError;`로 가져옵니다. 빈 문자열 `""`을 넣으면 어떻게 되는지도 예측해 보세요(조각 하나 `""`이 생기고, 빈 문자열은 숫자로 바꿀 수 없습니다).

</details>

### 연습 2. 설정 파일에서 포트 읽기

`fn read_port(path: &Path) -> Result<u16, Box<dyn Error>>`를 작성하세요. 파일 내용이 `port=8080` 한 줄이면 `Ok(8080)`을 돌려주고, 다음 경우에는 각각 이유가 드러나는 `Err`를 돌려줍니다. `main`에서 임시 디렉터리에 파일을 만들어 네 경우를 모두 확인하고, 끝나면 파일을 지우세요.

- 파일이 없음 → 파일을 읽을 때의 `io::Error`
- `=`가 없음 → `"'='가 없습니다"`
- 키가 `port`가 아님 → `"port 설정이 아닙니다"`
- 값이 `u16`이 아님 → `parse`의 에러

<details>
<summary>힌트</summary>

[`07_main_result.rs`](examples/07_main_result.rs)의 `read_count`에서 출발합니다. `fs::read_to_string(path)?`로 읽고, `text.trim().split_once('=')`가 돌려주는 `Option`은 4.2절처럼 `.ok_or("'='가 없습니다")?`로 `Result`로 바꿔 `?`를 씁니다. 에러 타입이 `Box<dyn Error>`이므로 문자열도 `?`가 에러로 바꿔 줍니다. 키가 다를 때는 `return Err("port 설정이 아닙니다".into());`처럼 씁니다. `.into()`는 문자열을 `Box<dyn Error>`로 바꾸는 메서드로, `?`가 하는 변환을 직접 부르는 것입니다([Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)). 06번 예제의 `port_of`와 달리 실패 이유가 모두 구분된다는 점을 확인하세요.

</details>

### 연습 3. 비어 있지 않은 이름 타입

검증 타입 `Name`을 만드세요. `Name::parse(input: &str) -> Result<Name, NameError>`는 앞뒤 공백을 뺀 이름이 비어 있으면 `NameError::Empty`, 20글자(`char` 기준)를 넘으면 `NameError::TooLong(글자 수)`를 돌려줍니다. 공백을 뺀 이름을 `String`으로 담고, 읽기 전용 게터 `fn as_str(&self) -> &str`을 제공하세요. 모듈 밖에서 검사를 건너뛰고 `Name`을 만들 수 없어야 합니다.

```
"  러스타  " → Ok, 이름: 러스타
"   "        → Err(Empty)
"가나다라마바사아자차카타파하가나다라마바사" → Err(TooLong(21))
```

<details>
<summary>힌트</summary>

[`08_guess.rs`](examples/08_guess.rs)의 `guessing` 모듈과 같은 구조로 `mod naming { pub struct Name { value: String } ... }`을 만듭니다. 필드에 `pub`을 붙이지 않는 것이 핵심입니다. 공백은 `input.trim()`으로 빼고, 비었는지는 `is_empty()`, 글자 수는 ch8에서 배운 `chars().count()`로 셉니다(`len()`은 바이트 수라서 한글 이름에서 틀립니다). 검사를 통과하면 `Ok(Name { value: trimmed.to_string() })`을 돌려줍니다. 게터는 `&self.value`를 돌려주면 `&String`이 `&str`로 자동 변환됩니다. 매개변수의 참조가 `&self` 하나뿐이라 라이프타임을 적지 않아도 됩니다(ch4의 `first_word`와 같습니다). `main`에서 에러를 `{e:?}`로만 출력하면 `TooLong`의 글자 수를 읽지 않은 것으로 쳐서 `dead_code` 경고(``field `0` is never read``)가 납니다. [`08_guess.rs`](examples/08_guess.rs)처럼 `Err(NameError::TooLong(n))` 패턴으로 꺼내 출력하세요.

</details>

### 연습 4. 패닉하는 `ship`을 `Result`로 바꾸기

[`01_panic.rs`](examples/01_panic.rs)의 `ship`은 "재고보다 많이 출고하는 일은 버그"라고 가정하고 패닉합니다. 이제 출고 수량이 고객의 주문에서 온다고 해 봅시다. 재고 부족은 더 이상 버그가 아니라 늘 일어나는 일입니다. `fn try_ship(stock: u32, amount: u32) -> Result<u32, String>`을 작성하고, 재고 10에서 주문 `[3, 20, 5, 4]`를 차례로 처리하세요. 처리할 수 없는 주문은 안내하고 건너뛰되, 프로그램은 끝까지 실행되어야 합니다.

```
3개 출고, 남은 재고 7
주문 거절: 재고(7)보다 많은 20개를 주문했습니다
5개 출고, 남은 재고 2
주문 거절: 재고(2)보다 많은 4개를 주문했습니다
```

<details>
<summary>힌트</summary>

`ship`의 `panic!(...)` 자리를 `return Err(format!(...));`으로, 마지막 `stock - amount`를 `Ok(stock - amount)`로 바꾸면 됩니다. ch3에서 배운 `checked_sub`를 쓰면 `stock.checked_sub(amount)`가 `Option<u32>`를 돌려주므로 `match`로 `Some`과 `None`을 나눠 `Ok`와 `Err`로 바꿀 수도 있습니다. `main`에서는 `let mut stock = 10;`을 두고 `for amount in [3, 20, 5, 4]` 안에서 `match try_ship(stock, amount)`로 성공하면 `stock`을 갱신하고, 실패하면 에러 메시지를 출력합니다. 같은 검사라도 **값이 어디서 오는가**에 따라 패닉과 `Result`가 갈린다는 2.11절의 기준을 떠올려 보세요.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [9. Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
  - [9.1 Unrecoverable Errors with `panic!`](https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html): 되감기와 abort, 백트레이스 읽기
  - [9.2 Recoverable Errors with `Result`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html): `match`와 `ErrorKind`, `unwrap`/`expect`, 에러 전파와 `?`, `main`의 `Result` 반환
  - [9.3 To `panic!` or Not to `panic!`](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html): 패닉과 `Result`의 선택 기준, 검증 타입 `Guess`. The Book은 `Guess`를 별도 모듈 파일(`src/guessing_game.rs`)에 두지만, 이 챕터는 한 파일 안의 인라인 모듈로 같은 구조를 만들었습니다.
- 표준 라이브러리 문서
  - [`std::result`](https://doc.rust-lang.org/std/result/index.html): "Results must be used" 절에 버린 `Result`가 경고인 이유가, "The question mark operator, `?`" 절에 `?`의 동작이 정리되어 있습니다.
  - [`std::io::ErrorKind`](https://doc.rust-lang.org/std/io/enum.ErrorKind.html): 에러 종류 전체 목록과, 배리언트가 늘어날 수 있어 포괄 갈래가 필요하다는 설명
  - [`panic!`](https://doc.rust-lang.org/std/macro.panic.html): `panic!`은 프로그램의 버그를 나타내고, 예상할 수 있는 실패는 `Result`로 다룬다는 설명
- Rust By Example: [Error handling](https://doc.rust-lang.org/rust-by-example/error.html) 장의 다음 절
  - [`panic`](https://doc.rust-lang.org/rust-by-example/error/panic.html), [`abort` & `unwind`](https://doc.rust-lang.org/rust-by-example/error/abort_unwind.html): 패닉과 두 가지 패닉 전략. 후자는 `rustc lemonade.rs -C panic=abort`처럼 명령줄에서 전략을 고르고, 코드 안에서 `cfg!(panic = "abort")`로 어느 전략인지 확인합니다(조건부 컴파일 `cfg`는 이 저장소의 뒤 파트에서 다룹니다).
  - [Unpacking options with `?`](https://doc.rust-lang.org/rust-by-example/error/option_unwrap/question_mark.html): 중첩된 `Option` 필드를 `?`로 따라가기. 예제의 `#[derive(Clone, Copy)]`는 `self.job?`이 값을 이동하지 않고 복사하게 하려고 붙인 것입니다.
  - [`Result`](https://doc.rust-lang.org/rust-by-example/error/result.html), [Early returns](https://doc.rust-lang.org/rust-by-example/error/result/early_returns.html), [Introducing `?`](https://doc.rust-lang.org/rust-by-example/error/result/enter_question_mark.html): 05번 예제의 1단계(`match`와 `return Err`)에서 2단계(`?`)로 가는 과정과 같은 흐름입니다. `try!`는 `?`가 생기기 전의 옛 방식이므로 읽고 넘어가면 됩니다.
  - [aliases for `Result`](https://doc.rust-lang.org/rust-by-example/error/result/result_alias.html): `io::Result` 같은 별칭을 직접 만드는 방법
  - [`Box`ing errors](https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/boxing_errors.html): 여러 에러 타입을 `Box<dyn Error>`로 묶기. 직접 만든 에러 타입에 `Display`와 `Error`를 구현하는 부분은 ch10의 트레이트와 [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)을 배운 뒤에 읽으면 좋습니다.
- Rustlings (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - [`13_error_handling`](https://github.com/rust-lang/rustlings/tree/main/exercises/13_error_handling)
    - `errors1.rs`: `Option<String>`을 돌려주던 함수를 실패 이유를 담는 `Result<String, String>`으로 바꾸기
    - `errors2.rs`: `parse`의 에러를 `?`로 전파하기
    - `errors3.rs`: `()`를 돌려주는 `main`에서 `?`를 쓴 컴파일 에러 고치기 (4.1절)
    - `errors4.rs`: 양수만 담는 타입의 생성자가 음수와 0에 `Err`를 돌려주게 하기 (2.12절의 `Guess::parse`와 같은 발상)
    - `errors5.rs`: `main`의 반환 타입 `Result<(), Box<dyn ???>>`의 `???` 채우기. 답은 2.10절에서 쓴 트레이트 이름입니다.
    - `errors6.rs`: `parse`의 에러를 직접 만든 에러 열거형의 배리언트로 감싸 돌려주기. 제공된 코드의 `map_err(ParsePosNonzeroError::Creation)`은 `Err` 안의 값을 그 배리언트로 감싸는 메서드입니다([Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md)). 빈 칸은 이 챕터의 `match`로 `Err(e) => return Err(ParsePosNonzeroError::ParseInt(e))`처럼 채워도 풀립니다.
  - `errors4.rs`~`errors6.rs`의 `x if x < 0 =>`처럼 패턴 뒤에 `if` 조건을 붙인 갈래(매치 가드)는 [ch18](../ch18-patterns/README.md)에서 다룹니다. "패턴이 맞고 조건도 참일 때만 이 갈래로 간다"고 읽으면 됩니다.
  - 연습 파일 아래쪽의 `#[cfg(test)]`, `#[test]`, `assert_eq!`은 정답을 확인하는 테스트([ch11](../ch11-testing/README.md))입니다.
- 에러 설명: `rustc --explain E0277`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
