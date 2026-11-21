# ch10. 환경·프로세스·시간: 인자와 환경 변수, 종료 코드, 자식 프로세스, `Instant`/`Duration`/`SystemTime`

지금까지의 프로그램은 거의 혼자서 동작했습니다. 이 챕터는 프로그램을 **운영체제 위의 프로세스(process)** 로 보고, 프로세스가 바깥 세계와 주고받는 것을 다룹니다. 앞 챕터들이 여기로 미뤄 둔 일도 함께 처리합니다.

- [Part 0 ch12](../../part0-getting-started/ch12-io-project-minigrep/README.md): `env::args`와 `env::var`의 기초 위에서 `args_os`, `var_os`, `VarError`의 두 경우, 현재 디렉터리와 실행 파일 경로로 넓힙니다.
- [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)와 [Part 1 ch8](../ch8-errors-panic/README.md): `main`이 `Result`를 돌려줄 때 종료 코드가 1이 되는 원리(`Termination` 트레이트), 원하는 종료 코드를 고르는 `ExitCode`, 그리고 `process::exit`가 소멸자를 실행하지 않는다는 점을 다룹니다.
- [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md): 맛만 본 `Instant::now()`와 `elapsed()`로 시간을 재는 방법과, 시간의 길이 `Duration`, 시스템 시계 `SystemTime`을 다룹니다.

챕터는 네 부분으로 나뉩니다. 프로세스가 **받는 것**(2.2~2.5절), 프로세스가 **돌려주는 것**(2.6~2.7절), 다른 프로그램을 **자식 프로세스로 실행하기**(2.8~2.10절), 그리고 **시간**(2.11~2.14절)입니다. 마지막 2.14절은 시간과 자식 프로세스를 합쳐 제한 시간이 있는 기다리기를 만듭니다.

- 패키지: `p1-ch10-env-process-time`
- 기준 문서: [`std::env`](https://doc.rust-lang.org/std/env/index.html), [`std::process`](https://doc.rust-lang.org/std/process/index.html), [`std::time`](https://doc.rust-lang.org/std/time/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024), macOS (aarch64). 셸 명령과 `sort`·`uniq`·`sh` 같은 유틸리티, 시그널 관련 출력은 macOS/Linux 기준입니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `env::args`와 `env::args_os`, `env::var`·`var_os`·`vars_os`의 차이를 설명하고, 환경 변수가 "없음", "비어 있음", "UTF-8이 아님"인 경우를 `VarError`로 구분해 처리하는 코드를 작성할 수 있다. 2024 에디션에서 `env::set_var`가 `unsafe`인 이유를 설명하고 `Command::env`로 대신할 수 있다.
2. 종료 코드의 관례를 설명하고, `main`이 `()`·`Result`·`ExitCode`를 돌려줄 때 각각 어떤 종료 코드가 되는지(`Termination` 트레이트)와 `process::exit`가 소멸자를 실행하지 않아 생기는 문제를 설명하며, 상황에 맞는 끝내는 방법을 고를 수 있다.
3. `Command`로 인자·환경 변수·현재 디렉터리를 정해 자식 프로세스를 실행하고, `status`·`output`·`spawn`의 차이와 "실행하지 못함(`Err`)"과 "자식이 실패함(`ExitStatus`)"의 차이를 설명할 수 있다. 인자가 셸을 거치지 않는다는 점이 무엇을 막고 무엇을 못 하게 하는지 설명할 수 있다.
4. `Stdio::piped`로 자식의 표준 입력에 쓰고 표준 출력을 읽으며, 두 자식을 파이프로 이을 수 있다. 입력을 닫지 않거나 큰 데이터를 한 스레드에서 주고받을 때 생기는 교착 상태를 설명하고 피할 수 있으며, `wait`·`try_wait`·`kill`로 자식을 남김없이 거둬들일 수 있다.
5. `Duration`을 만들고 단위를 바꾸고 넘치지 않게 계산할 수 있으며, `Instant`(단조 시계)와 `SystemTime`(시스템 시계) 중 무엇을 쓸지 근거를 들어 고르고, 실행 시간 측정과 제한 시간을 구현할 수 있다.

## 2. 핵심 개념

### 2.1 프로세스의 입구와 출구

운영체제는 프로그램을 실행할 때 프로세스 하나를 만들고, 그 프로세스에 몇 가지를 건네줍니다. 프로세스가 끝나면 부모 프로세스(parent process), 즉 그 프로그램을 실행한 쪽(셸이나 다른 프로그램)이 결과를 받아 갑니다.

| 방향 | 무엇 | 읽거나 정하는 방법 | 다루는 곳 |
| --- | --- | --- | --- |
| 받는 것 | 명령줄 인자 | `env::args`, `env::args_os` | 2.2절 |
| 받는 것 | 환경 변수 | `env::var`, `env::var_os`, `env::vars_os` | 2.3~2.4절 |
| 받는 것 | 현재 디렉터리 | `env::current_dir` | 2.5절 |
| 받는 것 | 표준 입력 | `io::stdin` | [ch9](../ch9-io-fs-path/README.md) |
| 돌려주는 것 | 표준 출력, 표준 에러 | `io::stdout`, `io::stderr` | [ch9](../ch9-io-fs-path/README.md) |
| 돌려주는 것 | 종료 코드 | `main`의 반환값, `process::exit` | 2.6~2.7절 |

이 표를 부모 쪽에서 보면 자식 프로세스(child process)를 실행하는 일이 됩니다. `std::process::Command`는 "자식에게 무엇을 건넬지"를 정하는 도구이고(2.8절), 자식이 돌려준 것은 `ExitStatus`와 파이프로 받습니다(2.9~2.10절). 표준 입출력 자체와 `Read`/`Write`는 [ch9](../ch9-io-fs-path/README.md)에서 배웠습니다.

### 2.2 명령줄 인자: `args`와 `args_os`

[Part 0 ch12](../../part0-getting-started/ch12-io-project-minigrep/README.md)에서 `env::args()`로 인자를 `Vec<String>`에 모았습니다. 같은 일을 하는 함수가 하나 더 있습니다.

| 함수 | 원소 타입 | UTF-8이 아닌 인자를 만나면 |
| --- | --- | --- |
| `env::args()` | `String` | 반복하는 도중에 **패닉** |
| `env::args_os()` | `OsString` | 그대로 돌려줌 |

`OsString`은 "운영체제가 쓰는 문자열"입니다. 유닉스에서 인자와 파일 이름은 아무 바이트의 나열일 수 있어서 UTF-8이라는 보장이 없습니다. 이 챕터에서는 `to_str()`(UTF-8이면 `Some(&str)`), `into_string()`(UTF-8이면 `Ok(String)`, 아니면 `Err(원래 값)`), `to_string_lossy()`(깨진 바이트를 `�`로 바꿈) 세 메서드만 쓰고, 타입 자체는 [ch14](../ch14-ffi-any-misc/README.md)에서 다룹니다.

```rust
let raw_args: Vec<_> = env::args_os().collect();
for (i, arg) in raw_args.iter().enumerate().skip(1) {
    match arg.to_str() {
        Some(text) => println!("    {i}번: UTF-8 {text:?}"),
        None => println!("    {i}번: UTF-8 아님 {arg:?} → 손실 변환 {:?}", arg.to_string_lossy()),
    }
}
```

어느 쪽을 쓸지는 인자가 무엇이냐에 달려 있습니다. 검색어나 숫자처럼 **텍스트**인 인자는 `args()`로 충분합니다. **파일 경로**를 인자로 받는 프로그램은 사용자의 파일 이름이 UTF-8이 아닐 수도 있으므로 `args_os()`로 받아 `PathBuf::from(arg)`로 바로 경로를 만드는 편이 안전합니다.

**0번 인자.** 문서는 0번 인자가 "관례상 실행 파일 경로이지만 아무 문자열로나 정할 수 있고, 존재하지 않는 경로일 수도 있다"고 설명합니다. 실행 방법에 따라 상대·절대 경로도 달라지므로(Part 0 ch12의 예제 01) 프로그램의 동작을 0번 인자에 기대지 않습니다. 실행 파일의 위치가 필요하면 `env::current_exe()`를 씁니다(2.5절).

**와일드카드는 셸이 풉니다.** `cargo run -- *.rs`라고 치면 프로그램은 `*.rs`가 아니라 셸이 펼친 파일 이름 목록을 받습니다. 문서에 따르면 유닉스의 셸은 따옴표 없는 `*`, `?`를 펼치지만 Windows에서는 그대로 전달됩니다. 이 차이는 2.8절에서 자식에게 인자를 넘길 때 다시 중요해집니다.

복잡한 옵션(`-v`, `--output=파일` 등)을 손으로 파싱하는 일은 금방 번거로워집니다. 인자 파싱 크레이트(`clap`)는 Part 7 `ch1-setup-arguments`에서 다룹니다.

### 2.3 환경 변수: `var`, `var_os`, `vars_os`

환경 변수(environment variable)는 부모 프로세스가 자식에게 물려주는 `이름=값` 목록입니다. 셸에서 `이름=값 명령`으로 그 명령에만 설정하거나 `export`로 세션 전체에 설정하는 방법은 Part 0 ch12에서 봤습니다.

| 함수 | 돌려주는 것 | 쓰는 곳 |
| --- | --- | --- |
| `env::var(이름)` | `Result<String, VarError>` | 값을 텍스트로 쓸 때 |
| `env::var_os(이름)` | `Option<OsString>` | 있는지만 볼 때, 경로처럼 UTF-8이 아닐 수 있는 값 |
| `env::vars()` | `(String, String)` 반복자 | 모든 값이 UTF-8이라고 확신할 때만 (아니면 패닉) |
| `env::vars_os()` | `(OsString, OsString)` 반복자 | 전체를 훑을 때 |

`var`의 에러 타입 `VarError`는 배리언트가 둘인 열거형입니다.

```rust
pub enum VarError {
    NotPresent,           // 설정되지 않음 (이름에 '='나 NUL이 들어 있어도 이것)
    NotUnicode(OsString), // 설정됐지만 UTF-8이 아님. 원래 값을 돌려줌
}
```

Part 0 ch12의 `env::var("IGNORE_CASE").is_ok()`는 "설정됐는가"만 봤습니다. 설정 값을 읽을 때는 세 상황을 구분해야 하는 경우가 많습니다. [`01_args_env.rs`](examples/01_args_env.rs)의 `workers_from_env`가 자주 쓰는 모양입니다.

```rust
fn workers_from_env() -> Result<u32, String> {
    match env::var("P1CH10_WORKERS") {
        Err(VarError::NotPresent) => Ok(4), // 없으면 기본값
        Err(VarError::NotUnicode(_)) => Err("P1CH10_WORKERS가 UTF-8이 아닙니다".to_string()),
        Ok(text) => text
            .trim()
            .parse()
            .map_err(|e| format!("P1CH10_WORKERS={text:?}를 읽지 못했습니다: {e}")),
    }
}
```

`P1CH10_WORKERS=` 처럼 **빈 값**으로 설정하면 `Ok("")`가 되어 숫자 변환에서 실패합니다. "없음"과 "비어 있음"을 같게 볼지는 프로그램이 정할 일이므로, 같게 보려면 `Ok(text) if text.is_empty()` 갈래를 따로 둡니다. `vars()`가 UTF-8이 아닌 값에서 패닉한다는 것은 실제로 확인했습니다(4.6절). 예제 01은 그래서 `vars_os()`로 개수만 셉니다.

Cargo가 빌드할 때 설정하는 환경 변수(`CARGO_PKG_VERSION` 등)와 컴파일 시점에 값을 읽는 `env!` 매크로는 [Part 3 ch7](../../part3-cargo/ch7-configuration-env/README.md)에서 다룹니다. 설정 파일까지 아우르는 설정 관리는 Part 7 `ch6-signals-config-exit-codes`에서 다룹니다.

### 2.4 환경을 바꾸는 함수는 왜 `unsafe`인가

`env::set_var`와 `env::remove_var`는 **지금 프로세스의** 환경을 바꿉니다. 2024 에디션에서 두 함수는 `unsafe` 함수이고, `unsafe` 블록 없이 부르면 E0133입니다(4.2절).

이유는 문서의 Safety 절에 있습니다. 유닉스 계열 운영체제는 환경을 스레드 안전하게 읽는 방법을 제공하지 않습니다. 한 스레드가 환경을 바꾸는 동안 다른 스레드가 읽으면 미정의 동작(Part 0 ch19)이 될 수 있습니다. 문제는 "누가 환경을 읽는가"를 프로그램이 다 알 수 없다는 점입니다. C 라이브러리의 많은 함수가 몰래 환경을 읽고, 표준 라이브러리도 `std::net`의 주소 조회처럼 이 모듈을 거치지 않고 읽을 수 있습니다. 그래서 문서는 **단일 스레드 프로그램에서는 안전하지만, 여러 스레드를 쓰는 프로그램에서 안전한 방법은 아예 부르지 않는 것뿐**이라고 결론짓습니다(Windows에서는 늘 안전합니다). 컴파일러가 이 조건을 검사할 수 없으므로 "확인은 프로그래머의 몫"이라는 표시로 `unsafe`가 붙었습니다.

그렇다면 환경 변수를 바꾸고 싶을 때는 어떻게 할까요? 대부분은 **자식 프로세스에게 넘기기 위해서**입니다. 문서도 그럴 때는 `Command::env`를 쓰라고 안내합니다. 이 메서드는 자식의 환경만 정하고 지금 프로세스는 건드리지 않으므로 `unsafe`가 필요 없습니다(2.8절, 예제 03의 2번 구역). 설정 값을 함수에 넘기고 싶을 때는 환경 변수를 거치지 말고 인자로 넘깁니다. Part 0 ch12 연습 3의 힌트처럼, 값을 해석하는 함수를 따로 두면 테스트에서도 환경을 바꿀 필요가 없습니다.

이 함수가 2021 에디션까지는 안전한 함수였다가 바뀐 경위는 [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md)에서 다룹니다.

### 2.5 현재 디렉터리, 실행 파일, 플랫폼 상수

| 함수·상수 | 뜻 |
| --- | --- |
| `env::current_dir()` | 현재 작업 디렉터리. **프로그램을 실행한 곳**이며, 상대 경로는 이곳을 기준으로 풀립니다. `io::Result<PathBuf>` |
| `env::current_exe()` | 지금 실행 중인 **실행 파일의 경로**. `io::Result<PathBuf>` |
| `env::temp_dir()` | 임시 디렉터리 경로(Part 0 ch9의 예제에서 사용) |
| `env::consts::OS`, `FAMILY`, `ARCH` | 컴파일 대상 플랫폼. 이 환경에서는 `"macos"`, `"unix"`, `"aarch64"` |
| `env::consts::EXE_SUFFIX`, `DLL_EXTENSION` | 실행 파일 확장자(`""`, Windows는 `".exe"`), 공유 라이브러리 확장자(`"dylib"`) |

`current_dir`와 `current_exe`는 서로 관계가 없습니다. `cargo run`을 저장소 루트에서 하든 챕터 디렉터리에서 하든 실행 파일은 `target/debug/examples/` 아래에 있지만, 현재 디렉터리는 명령을 친 곳입니다. 프로그램 옆에 둔 데이터 파일을 찾으려면 `current_exe()`의 부모 디렉터리를, 사용자가 넘긴 상대 경로를 풀려면 `current_dir()`를 기준으로 삼습니다. 문서는 `current_exe`의 결과가 심볼릭 링크를 따라갔는지가 플랫폼마다 다르고, 권한이 높은 프로그램에서는 보안 판단에 쓰지 말라고 경고합니다.

`env::consts`의 값은 실행 중에 알아낸 것이 아니라 **컴파일할 때 정해진 상수**입니다. 코드 자체를 플랫폼마다 다르게 하려면 `#[cfg(unix)]` 같은 조건부 컴파일을 쓰고, 이 챕터의 예제도 유닉스 명령을 쓰는 구역을 그렇게 감쌌습니다.

`env::set_current_dir`로 현재 디렉터리를 바꿀 수도 있지만, 프로세스 전체에 영향을 주는 전역 상태라서 여러 스레드가 상대 경로를 쓰는 중이라면 결과가 엉킵니다. 자식만 다른 디렉터리에서 실행하려면 `Command::current_dir`를 씁니다(2.8절).

### 2.6 종료 코드와 `main`의 반환 타입

**종료 코드(exit code)** 는 프로세스가 끝나면서 부모에게 남기는 정수입니다. 관례는 단순합니다.

- **0은 성공, 0이 아닌 값은 실패**입니다. 셸의 `명령1 && 명령2`(성공하면 다음 실행), `||`, `if 명령; then`이 모두 이 관례로 동작합니다.
- 0이 아닌 값 각각의 뜻은 프로그램마다 정합니다. `grep`은 "찾음 0, 못 찾음 1, 에러 2"를 씁니다(Part 0 ch12 연습 4). 셸이 정한 값도 있어서, 명령을 찾지 못하면 127, 시그널로 끝나면 128+시그널 번호(SIGKILL이면 137)를 보여 줍니다(zsh에서 확인).
- 유닉스에서 부모에게 전달되는 것은 **하위 8비트(0~255)** 뿐입니다(2.7절).

Rust 프로그램의 종료 코드는 `main`의 **반환값**이 정합니다. `main`은 아무 타입이나 돌려줄 수 없고, `std::process::Termination` 트레이트를 구현한 타입만 돌려줄 수 있습니다.

```rust
pub trait Termination {
    fn report(self) -> ExitCode;
}
```

`main`이 끝나면 표준 라이브러리가 반환값의 `report()`를 불러 `ExitCode`를 얻고, 그 값으로 프로세스를 끝냅니다. 표준 라이브러리가 구현해 둔 타입은 네 가지입니다.

| `main`의 반환 타입 | 종료 코드 (이 환경에서 확인) |
| --- | --- |
| `()` | 0 |
| `ExitCode` | 담긴 값 (`SUCCESS` 0, `FAILURE` 1, `ExitCode::from(n)`은 n) |
| `Result<T, E>` (`T: Termination`, `E: Debug`) | `Ok(t)`면 `t.report()`, `Err(e)`면 표준 에러에 `Error: {e:?}`를 쓰고 1 |
| `!` (돌아오지 않음) | 해당 없음 |

여기서 Part 0 ch9의 "`main`이 `Err`를 돌려주면 종료 코드 1"이 나옵니다. `Result`의 `report()`가 에러를 디버그 형식으로 출력하고 `FAILURE`를 돌려주는 것입니다. `T: Termination`이므로 `Result<ExitCode, E>`도 됩니다. [`02_exit_code.rs`](examples/02_exit_code.rs)는 표준 라이브러리 대신 `report()`를 직접 불러 이 표를 확인합니다. C처럼 `fn main() -> i32`라고 쓰면 `i32`가 `Termination`이 아니라서 E0277입니다(4.1절).

**`ExitCode`** 는 1.61.0에서 안정화된 타입입니다. 만드는 방법은 `ExitCode::SUCCESS`, `ExitCode::FAILURE`, `ExitCode::from(u8)` 셋뿐입니다. `i32`가 아니라 `u8`만 받는 것은 대부분의 플랫폼이 지원하는 범위의 값만 만들 수 있게 하려는 것입니다. 문서는 흔치 않은 플랫폼과의 호환을 위해 안에 든 숫자를 꺼내는 방법도 두지 않았다고 설명합니다. 비교(`==`)만 됩니다.

**어떤 모양을 고를까.** Part 0 ch12의 minigrep은 사용자에게 보여 줄 메시지를 직접 정하려고 `match`와 `process::exit(1)`을 썼습니다. 지금은 같은 일을 `ExitCode`로 할 수 있고, 이 챕터의 예제 03·04·08이 이 모양입니다.

```rust
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("에러: {e}"); // 사용자에게 보여 줄 문장은 내가 정합니다
            ExitCode::FAILURE       // 실패 종류에 따라 ExitCode::from(2) 등도 가능
        }
    }
}
```

| 상황 | 고를 것 |
| --- | --- |
| 실패할 일이 없다 | `fn main()` |
| 실패를 개발자용 디버그 형식으로 보여 줘도 된다, 종료 코드는 1이면 된다 | `fn main() -> Result<(), E>` |
| 사용자용 메시지를 직접 쓰거나, 실패 종류마다 종료 코드를 다르게 하고 싶다 | `fn main() -> ExitCode` + `run()` 함수 |
| `main`에 돌아갈 수 없는 깊은 곳에서 즉시 끝내야 한다 | `process::exit` (2.7절의 주의를 읽고) |

`ExitCode`는 지금 프로세스가 **돌려줄** 값이고, 자식 프로세스가 **돌려준** 결과는 `ExitStatus`입니다(2.9절). 문서는 두 타입을 일부러 나눴다고 설명합니다. 자식이 시그널로 끝난 것 같은 결과는 지금 프로세스가 그대로 재현할 수 없기 때문입니다.

### 2.7 `process::exit`: 그 자리에서 끝내기

```rust
pub fn exit(code: i32) -> !
```

`process::exit`는 돌아오지 않는 함수(`!`)이고, 부른 자리에서 프로세스를 끝냅니다. 문서는 이때 **현재 스레드와 다른 모든 스레드의 스택에 있는 소멸자가 실행되지 않는다**고 분명히 적습니다. Part 0 ch12에서 "아직 살아 있는 값이 drop되지 않는다"고만 하고 넘어간 부분이 실제로 어떤 결과를 낳는지 예제 02의 2번 구역에서 확인합니다.

```rust
"exit3" => {
    let _guard = Cleanup("exit");                  // drop될 때 "[drop] exit 정리"를 출력
    let mut out = BufWriter::new(io::stdout());
    writeln!(out, "BufWriter에 쓴 줄").unwrap();   // 아직 BufWriter의 버퍼에만 있음
    process::exit(3)
}
```

같은 코드를 `process::exit(3)` 대신 `ExitCode::from(3)`을 돌려주며 끝내면 표준 출력에 두 줄이 나옵니다. `process::exit`로 끝내면 아무것도 나오지 않습니다. `BufWriter`는 drop될 때 버퍼를 비우는데([ch9](../ch9-io-fs-path/README.md)), 그 drop이 실행되지 않아 **써 둔 출력이 사라진 것**입니다. 파일에 쓰던 `BufWriter`였다면 파일 끝부분이 잘렸을 것입니다. 참고로 이 툴체인에서 확인한 바로는 `print!`처럼 표준 라이브러리 자체의 표준 출력 버퍼에 남은 내용은 `process::exit`에서도 출력됐습니다. 사라지는 것은 내가 만든 값의 drop에 맡긴 일입니다.

그래서 문서는 `process::exit`를 "더 실행할 소멸자가 없는 곳"에서만 부르고, 되도록 `main`에서 `ExitCode`나 `Result`를 돌려주라고 권합니다. 패닉(종료 코드 101)과 달리 되감기도 없으므로 `catch_unwind`(Part 1 ch8)로도 막을 수 없습니다.

**8비트 잘림.** 인자는 `i32`이지만 유닉스에서 부모가 받는 것은 하위 8비트입니다. 예제 02의 3번 구역에서 `process::exit(256)`은 종료 코드 0(성공!)으로, `process::exit(-1)`은 255로 보였습니다. 문서의 예로는 같은 `exit(0x0100)`이 Windows에서는 256입니다. `ExitCode::from`이 `u8`만 받는 것은 이런 실수를 막기 위해서입니다.

### 2.8 `Command`: 자식 프로세스 만들기

`std::process::Command`는 "어떤 프로그램을, 무엇을 건네서 실행할지"를 모으는 빌더(builder)입니다. `Command::new(프로그램)`의 기본 설정은 다음과 같습니다.

- 인자 없음
- 부모의 환경 변수와 현재 디렉터리를 물려받음
- 표준 입출력은 실행 방법에 따라 다름(2.9절)

| 메서드 | 하는 일 |
| --- | --- |
| `arg(a)`, `args([a, b])` | 인자를 하나씩 / 여러 개 추가 |
| `env(k, v)`, `envs(목록)` | 자식의 환경 변수를 추가하거나 덮어씀 |
| `env_remove(k)` | 물려받을 환경 변수 하나를 뺌 |
| `env_clear()` | 아무것도 물려받지 않음 |
| `current_dir(경로)` | 자식의 현재 디렉터리 |
| `stdin`, `stdout`, `stderr` | 표준 입출력 연결(2.10절) |

빌더 메서드는 모두 `&mut self`를 받아 `&mut Command`를 돌려주므로 사슬처럼 이어 쓸 수 있습니다. 한 표현식 안에서 `Command::new(..).arg(..).output()`처럼 끝까지 쓰면 문제가 없지만, `let cmd = Command::new("echo").arg("hi");`처럼 중간 결과를 변수에 담으면 `Command` 값 자체는 문장 끝에서 버려지는 임시값이라 E0716이 납니다(4.6절). 설정을 나눠 쓸 때는 `let mut cmd = Command::new(..); cmd.arg(..);`처럼 먼저 변수에 담습니다. 예제의 `child()` 함수가 이 모양입니다. 한 번 만든 `Command`는 여러 번 실행할 수 있습니다.

**인자는 셸을 거치지 않습니다.** 터미널에서 `echo $HOME | tr a-z A-Z`를 치면 셸이 `$HOME`을 값으로 바꾸고, `|`로 두 프로그램을 잇고, 공백으로 인자를 나눕니다. `Command`에는 이 셸이 없습니다. 문서의 표현대로 인자는 **글자 그대로** 프로그램에 전달되고, 따옴표, 이스케이프, 단어 나누기, 와일드카드, 변수 치환이 일어나지 않습니다. 예제 03의 1번 구역에서 자식은 `"*.rs"`, `"$HOME"`, `"a; echo 주입"`, `"x | y"`를 받은 그대로 출력합니다.

이 성질의 좋은 면은 **셸 인젝션(shell injection)이 없다**는 것입니다. 사용자가 입력한 파일 이름이 `a; rm -rf ~`라도 `Command::new("cat").arg(name)`은 그런 이름의 파일을 찾을 뿐 명령을 실행하지 않습니다. 반대로 말하면 **셸 문법은 동작하지 않습니다.**

- `Command::new("ls -l")`은 `ls -l`이라는 이름의 프로그램을 찾다가 `NotFound`로 실패합니다. 프로그램과 인자를 나눠 `Command::new("ls").arg("-l")`로 씁니다.
- `.arg("-l /tmp")`는 공백이 든 인자 **하나**입니다. macOS의 `ls`는 공백 문자를 옵션 글자로 읽어 `ls: invalid option -- `(마지막 글자가 공백)를 출력하고 실패합니다. `.args(["-l", "/tmp"])`로 나눕니다. Clippy가 이 모양을 `suspicious_command_arg_space`로 경고합니다(4.6절).
- 파이프(`|`)는 2.10절처럼 Rust 코드로 잇습니다. 꼭 셸이 필요하면 `Command::new("sh").arg("-c").arg(스크립트)`로 셸을 직접 실행합니다. 이때 스크립트 문자열에 사용자 입력을 `format!`으로 끼워 넣으면 인젝션이 되살아나므로, 값은 스크립트 뒤의 인자로 넘기고 스크립트 안에서 `"$1"`로 받습니다(예제 03의 6번 구역).

Windows에서는 인자가 한 줄의 명령줄 문자열로 합쳐져 전달되고 받는 프로그램이 다시 나누므로, `cmd.exe`나 `.bat` 파일에 신뢰할 수 없는 인자를 넘길 때 특히 조심해야 한다고 문서가 경고합니다.

**프로그램 찾기.** 프로그램 이름이 절대 경로가 아니면 `PATH` 환경 변수에서 찾습니다. 문서에 따르면 유닉스에서는 **자식의 환경**의 `PATH`로 찾으므로, `env_clear()`나 `env_remove("PATH")`를 했다면 부모의 `PATH`가 아니라 운영체제의 기본값(보통 `/bin:/usr/bin`)으로 찾습니다. 환경을 비울 때는 절대 경로를 쓰거나 `PATH`를 직접 넣어 줍니다. 이 챕터의 예제는 자식으로 `env::current_exe()`(절대 경로)를 실행하므로 이 문제가 없습니다.

**환경 변수 넘기기.** 예제 03의 2번 구역은 같은 자식을 네 가지 환경으로 실행합니다. `env_clear()`를 하면 자식이 본 환경 변수가 0개였습니다(macOS에서 확인). 마지막 줄에서 부모의 `P1CH10_COLOR`는 여전히 `Err(NotPresent)`입니다. `Command::env`는 자식 몫만 바꾸기 때문입니다.

`Command`를 디버그 형식으로 출력하면 셸 명령처럼 보이는 문자열이 나옵니다. 이 환경에서 `Command::new("echo").arg("a b").arg("*").env("X", "1").current_dir("/tmp")`는 `cd "/tmp" && X="1" "echo" "a b" "*"`로 출력됐습니다. 문서는 이 출력이 실행 가능한 셸 명령이라고 보장하지 않는다고 적고 있으니 로그용으로만 씁니다.

### 2.9 실행하는 세 가지: `status`, `output`, `spawn`

| 메서드 | 기다리나 | 돌려주는 것 | 표준 입력 기본값 | 표준 출력·에러 기본값 |
| --- | --- | --- | --- | --- |
| `status()` | 끝날 때까지 | `io::Result<ExitStatus>` | 물려받음(inherit) | 물려받음 |
| `output()` | 끝날 때까지 | `io::Result<Output>` | 빈 입력 | 파이프로 모음(piped) |
| `spawn()` | 기다리지 않음 | `io::Result<Child>` | 물려받음 | 물려받음 |

- **`status`**: 자식이 부모의 터미널을 그대로 씁니다. 자식의 출력은 부모를 거치지 않고 화면으로 갑니다. 컴파일러나 편집기처럼 사용자와 직접 상호작용할 프로그램을 실행할 때 씁니다.
- **`output`**: 자식의 출력이 필요할 때 씁니다. `Output`은 `status: ExitStatus`, `stdout: Vec<u8>`, `stderr: Vec<u8>` 세 필드를 가진 구조체입니다. 출력이 **바이트**라는 점에 주의합니다. 문자열로 쓰려면 `String::from_utf8`(UTF-8이 아니면 에러)이나 `String::from_utf8_lossy`(깨진 바이트를 `�`로)로 바꿉니다(4.4절). 표준 입력은 빈 입력이라서 자식이 읽으면 바로 끝을 만납니다(예제 04의 3번 구역).
- **`spawn`**: 자식을 시작하자마자 `Child`를 돌려줍니다. 기다리는 동안 다른 일을 하거나, 자식과 파이프로 대화하거나, 제한 시간을 걸 때 씁니다. 대신 **기다리는 것도 내 책임**입니다(2.14절).

**`ExitStatus`** 는 끝난 자식의 종료 상태(exit status)입니다. `success()`는 종료 코드가 0일 때만 `true`이고, `code()`는 종료 코드를 `Option<i32>`로 돌려줍니다. 유닉스에서 시그널로 끝난 프로세스는 종료 코드가 없어 `None`입니다(예제 08). `Display`로 출력하면 `exit status: 3`이나 `signal: 9 (SIGKILL)`처럼 나오며, 문서는 실패한 프로세스를 보고할 때 이 `Display`를 쓰라고 권합니다.

**`Err`와 실패한 자식은 다릅니다.** `status`·`output`·`spawn`이 `Err(io::Error)`를 돌려주는 것은 **자식을 실행하지 못했을 때**뿐입니다. 프로그램이 없거나(`NotFound`), 실행 권한이 없거나, 프로세스를 더 만들 수 없는 경우입니다. 자식이 실행된 뒤 0이 아닌 코드로 끝나거나 시그널로 죽는 것은 `Ok`이고, 결과는 `ExitStatus`에 담깁니다. 그래서 `?` 하나로는 자식의 실패를 잡지 못합니다. 예제 03의 5번 구역처럼 `status.success()`를 따로 확인해야 합니다. (`ExitStatus`를 `Result`로 바꿔 주는 `exit_ok()`는 아직 nightly 전용 실험 API입니다.)

```rust
let status = child("fail").status()?;          // 실행은 됐으므로 ?를 통과
if !status.success() {
    return Err(io::Error::other(format!("자식이 실패했습니다: {status}")));
}
```

### 2.10 `Stdio`와 파이프

**파이프(pipe)** 는 운영체제가 제공하는 한 방향 통로입니다. 한쪽 끝에 쓴 바이트를 다른 쪽 끝에서 읽습니다. `Command`의 `stdin`·`stdout`·`stderr` 메서드에 `Stdio`를 넘겨 자식의 표준 입출력을 어디에 연결할지 정합니다.

| 값 | 뜻 |
| --- | --- |
| `Stdio::inherit()` | 부모의 것을 그대로 씀(터미널 등) |
| `Stdio::piped()` | 새 파이프를 만들고 부모 쪽 끝을 `Child`에 넣어 둠 |
| `Stdio::null()` | 입력은 빈 입력, 출력은 버림 |
| `Stdio::from(child.stdout)` 등 | 다른 자식의 파이프 끝이나 파일(`File`)에 연결 |

`piped()`로 설정하면 `spawn()`이 돌려준 `Child`의 공개 필드 `stdin: Option<ChildStdin>`, `stdout: Option<ChildStdout>`, `stderr: Option<ChildStderr>`에 부모 쪽 끝이 들어 있습니다. `ChildStdin`은 `Write`를, `ChildStdout`과 `ChildStderr`는 `Read`를 구현하므로 [ch9](../ch9-io-fs-path/README.md)에서 배운 메서드를 그대로 씁니다. 꺼낼 때는 **`take()`** 를 씁니다. `child.stdout.unwrap()`은 필드를 `child` 밖으로 옮기는 부분 이동이라 그 뒤로 `child.wait()`를 부를 수 없습니다(4.5절). `take()`는 필드 자리에 `None`을 남깁니다(Part 1 ch2).

[`04_pipes.rs`](examples/04_pipes.rs)의 1번 구역이 기본 모양입니다.

```rust
let mut upper = child("upper").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
let mut stdin = upper.stdin.take().expect("stdin을 piped로 설정했습니다");
stdin.write_all("hello\nrust\n안녕\n".as_bytes())?;
drop(stdin); // 쓰는 쪽을 닫아야 자식이 입력의 끝(EOF)을 봅니다
let output = upper.wait_with_output()?; // 남은 출력을 다 읽고 자식을 기다림
```

**입력을 닫지 않으면 멈춥니다.** 자식이 표준 입력을 끝까지 읽는 프로그램이라면 "입력이 끝났다"는 신호가 와야 일을 마칩니다. 그 신호는 쓰는 쪽 끝이 **모두 닫혔을 때** 옵니다. 위 코드에서 `drop(stdin)`을 빼면 자식은 입력을 계속 기다리고, 부모는 `wait_with_output`에서 자식을 계속 기다립니다. 실제로 `cat`으로 시험하면 프로그램이 멈춘 채로 끝나지 않았습니다. `wait`와 `wait_with_output`은 `Child` 안에 남아 있는 `stdin`은 기다리기 전에 닫아 주지만, `take()`로 꺼낸 것은 내 변수에 있으므로 내가 닫아야 합니다.

**큰 데이터는 다른 스레드에서 씁니다.** 파이프에는 운영체제가 정한 크기의 버퍼가 있고, 가득 차면 쓰는 쪽이 멈춥니다. 부모가 1MB를 모두 쓴 다음에 출력을 읽으려 하면 이렇게 됩니다.

1. 자식은 입력을 읽는 대로 출력에 쓰는데, 아무도 읽지 않으니 출력 파이프가 곧 가득 찹니다.
2. 자식은 출력에 쓰다가 멈추고, 그래서 입력을 더 읽지 않습니다.
3. 입력 파이프도 가득 차서 부모의 `write_all`이 멈춥니다. 서로를 기다리는 교착 상태(deadlock)입니다.

이 환경에서 1,000,000바이트를 `cat`에 이렇게 보내 보니 `write_all`에서 멈췄습니다. `std::process` 모듈 문서가 권하는 해결은 쓰기를 다른 스레드에 맡겨 **쓰기와 읽기를 동시에** 하는 것입니다(예제 04의 2번 구역). 스레드는 Part 0 ch15에서 배운 `thread::spawn`을 씁니다.

```rust
let mut stdin = echo.stdin.take().expect("stdin을 piped로 설정했습니다");
let writer = thread::spawn(move || stdin.write_all(data.as_bytes())); // 끝나면 stdin이 drop되어 닫힘
let output = echo.wait_with_output()?;                                 // 동시에 출력을 읽음
writer.join().expect("쓰기 스레드가 패닉하지 않아야 합니다")?;
```

**두 자식을 잇기.** 셸의 `numbers | sum`은 앞 자식의 표준 출력 끝을 뒤 자식의 표준 입력으로 넘기면 됩니다. `ChildStdout`은 `Stdio`로 바꿀 수 있습니다(`impl From<ChildStdout> for Stdio`). 데이터는 두 자식 사이를 곧바로 흐르고 부모는 만지지 않습니다.

```rust
let mut producer = child("numbers").stdout(Stdio::piped()).spawn()?;
let pipe = producer.stdout.take().expect("stdout을 piped로 설정했습니다");
let consumer = child("sum").stdin(Stdio::from(pipe)).stdout(Stdio::piped()).spawn()?;
let result = consumer.wait_with_output()?;
producer.wait()?; // 앞쪽 자식도 거둬들입니다
```

예제 04의 5번 구역은 같은 방법으로 유닉스의 `sort`와 `uniq`를 잇습니다.

### 2.11 `Duration`: 시간의 길이

여기부터는 시간입니다. `std::time`에는 타입이 셋 있습니다. **`Duration`은 길이**(얼마나), **`Instant`와 `SystemTime`은 시점**(언제)입니다. 시점끼리 빼면 `Duration`이 되고, 시점에 `Duration`을 더하면 시점이 됩니다.

`Duration`은 초(`u64`)와 나노초(`u32`, 0~999,999,999)로 이루어진 음수가 없는 값입니다. [`05_duration.rs`](examples/05_duration.rs)가 주요 연산을 모두 보여 줍니다.

| 하고 싶은 일 | 방법 |
| --- | --- |
| 만들기 | `Duration::new(초, 나노초)`, `from_secs`/`from_millis`/`from_micros`/`from_nanos`, `from_mins`/`from_hours`(1.91.0), `from_secs_f64`, 상수 `ZERO`·`MAX` |
| 단위로 꺼내기 | `as_secs()`(`u64`, 소수점 아래 버림), `subsec_millis()` 등(1초 미만 부분), `as_millis()`/`as_micros()`/`as_nanos()`(`u128`), `as_secs_f64()` |
| 더하기·빼기·비교 | `+`, `-`, `<`, `==`, `max`, 반복자의 `sum()` |
| 정수배 | `d * 3`, `d / 3` (`u32`만) |
| 실수배·비율 | `mul_f64(1.5)`, `div_f64`, `div_duration_f64(다른 d)`(1.80.0) |
| 차이의 크기 | `abs_diff`(1.81.0) |

**넘치는 연산.** `Duration`에는 음수가 없으므로 `300ms - 1s`는 `overflow when subtracting durations`로 패닉합니다. 정수 타입의 검사 연산·포화 연산(Part 1 ch1)과 같은 이름의 메서드가 있습니다. `checked_sub`·`checked_add`·`checked_mul`은 넘치면, `checked_div`는 0으로 나누면 `None`, `saturating_sub`·`saturating_add`·`saturating_mul`은 0이나 `MAX`에서 멈춥니다. `from_secs_f64`는 음수·NaN·너무 큰 값에서 패닉하므로, 설정 파일 등 바깥에서 받은 실수는 `try_from_secs_f64`로 바꿉니다.

**실수 곱셈이 연산자가 아닌 이유.** `Duration * 1.5`는 컴파일되지 않습니다(4.3절). 실수 곱셈은 나노초 아래를 어떻게 반올림할지, 음수나 NaN이 오면 어떻게 할지를 정해야 하는 연산이라, 그 규칙을 문서에 적은 이름 있는 메서드(`mul_f64`)로만 제공합니다.

**출력.** 문서에 따르면 `Duration`은 사람이 읽기 좋은 형식이 여러 가지라서 **일부러 `Display`를 구현하지 않았습니다.** 디버그 형식(`{:?}`)은 알맞은 단위를 골라 줍니다(`250ms`, `1.5µs`, `3725.042s`). 정밀도(`{:.2?}`)도 지정할 수 있습니다. `µs`는 ASCII가 아니므로 출력이 ASCII만 받는 곳으로 간다면 직접 형식을 만듭니다. 예제 05의 `hms` 함수는 `01:02:05.042` 모양을 만듭니다.

### 2.12 `Instant`: 경과 시간을 재는 단조 시계

`Instant`는 **단조 시계(monotonic clock)** 의 한 시점입니다. 단조 시계는 뒤로 가지 않는 시계로, 문서는 "플랫폼 버그가 없다면 나중에 잰 `Instant`는 먼저 잰 것보다 작지 않다"고 보장합니다. 대신 이 값은 달력의 시각과 아무 관계가 없습니다. "몇 초"를 꺼내는 메서드도 없고, 할 수 있는 일은 두 `Instant`의 **차이**를 구하거나 비교하는 것뿐입니다. 그래서 `Instant`는 **경과 시간(elapsed time)을 재는 전용 도구**입니다.

```rust
let start = Instant::now();
let value = work();
let took = start.elapsed();          // Instant::now() - start 와 같음
```

[`06_instant.rs`](examples/06_instant.rs)에서 확인하는 성질은 다음과 같습니다.

- `thread::sleep(d)`는 **적어도** `d`만큼 잡니다. 운영체제의 스케줄링 때문에 더 길어질 수는 있어도 짧아지지는 않으므로 `elapsed() >= d`가 늘 성립합니다.
- 두 `Instant`는 빼거나(`later - earlier`) `duration_since`로 차이를 구합니다. 순서를 거꾸로 넣으면 지금은 `0ns`로 멈추지만(문서는 앞으로 다시 패닉하게 될 수도 있다고 경고합니다), `checked_duration_since`는 `None`을 돌려줘 실수를 드러냅니다.
- `Instant + Duration`으로 미래의 시점을 만들 수 있습니다. 이것으로 **마감 시점(deadline)** 을 정합니다.

**실행 시간을 잴 때 주의할 점.** 예제 06의 2번 구역은 1부터 백만까지 제곱의 합을 구하는 데 걸린 시간을 잽니다. 이 환경의 디버그 빌드에서는 실행할 때마다 9~17ms 사이였고, `--release`로 실행하면 40ns 안팎(`41.0ns`, `42.0ns`)이었습니다. 백만 번 반복을 실제로 돌았다면 나올 수 없는 시간이므로, 최적화기가 반복문을 훨씬 빠른 계산으로 바꿔 버린 것입니다. 두 가지를 기억해 둡니다.

- 성능은 **릴리스 빌드에서** 잽니다. 디버그 빌드의 시간은 최적화 전의 시간이라 실제 성능을 대표하지 않습니다.
- 최적화기는 결과를 쓰지 않는 계산을 지우거나, 입력이 상수면 컴파일할 때 미리 계산할 수 있습니다. `std::hint::black_box(값)`은 "이 값이 어떻게 쓰일지 모른다"고 컴파일러에게 알려 그런 최적화를 막는 힌트입니다. 문서는 벤치마크에 쓰라고 권하면서도, 막아 주는 정도는 플랫폼에 따라 다른 "최선의 노력"일 뿐이라고 적고 있습니다. 예제는 입력에 `black_box`를 씌워 컴파일 시점의 계산은 막았지만, 반복문을 다른 계산으로 바꾸는 것까지 막지는 않습니다. 무엇을 재고 있는지 늘 확인해야 합니다.

**제한 시간(timeout).** "1초 안에 끝내라"는 "시작 시점 + 1초"라는 마감 시점을 정해 두고 비교하는 것으로 구현합니다. 예제 06의 4번 구역의 `retry_until`이 그 모양입니다.

```rust
let deadline = Instant::now() + limit;
loop {
    attempt += 1;
    if let Ok(value) = flaky(attempt, succeed_at) {
        return Ok((value, attempt));
    }
    if Instant::now() >= deadline {
        return Err(attempt);
    }
}
```

마감 시점을 한 번 정해 두면 몇 번을 돌든 "남은 시간"은 `deadline.saturating_duration_since(Instant::now())`로 언제든 계산할 수 있습니다. 채널의 `recv_timeout`처럼 제한 시간을 받는 함수에 이 남은 시간을 넘기는 방법은 [ch11](../ch11-threads-sync/README.md)에서, 소켓의 제한 시간은 [ch12](../ch12-net/README.md)에서 다룹니다.

### 2.13 `SystemTime`: 시스템 시계

`SystemTime`은 운영체제의 **시스템 시계(system clock)**, 흔히 벽시계(wall clock)라고 부르는 시계의 시각입니다. 파일의 수정 시각을 돌려주는 `fs::Metadata::modified`도 이 타입을 쓰고, 로그에 남기는 시각이나 다른 프로그램·서버와 주고받는 시각도 이 시계를 씁니다.

`SystemTime`도 안을 직접 들여다볼 수 없습니다. 기준 시각(epoch)으로 쓰는 것이 `UNIX_EPOCH`, 즉 1970-01-01 00:00:00 UTC입니다. 이 기준 시각부터 흐른 초를 **유닉스 시간(Unix time)** 이라고 하며, 많은 프로토콜과 파일 형식이 이 값을 씁니다.

```rust
let secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(); // 유닉스 시간
let billion = UNIX_EPOCH + Duration::from_secs(1_000_000_000);      // 정해진 시각 만들기
```

**`duration_since`가 `Result`인 이유.** 시스템 시계는 **뒤로 갈 수 있습니다.** 사용자가 시계를 바꾸거나, 네트워크 시간 동기화가 시계를 조정하면 나중에 잰 `SystemTime`이 먼저 잰 것보다 앞설 수 있습니다. 문서는 "파일 두 개를 차례로 저장했는데 두 번째 파일의 시각이 더 이를 수 있다"는 예를 듭니다. 그래서 `duration_since`와 `elapsed`는 순서가 거꾸로면 `Err(SystemTimeError)`를 돌려주고, 에러의 `duration()`은 얼마나 거꾸로였는지 알려 줍니다. 순서가 논리적으로 맞더라도 이 `Err`는 처리해야 합니다. `Instant`의 차이는 `Duration`을 바로 돌려주는 것과 대조적입니다.

**달력과 시간대는 없습니다.** 표준 라이브러리는 "기준 시각부터 몇 초"까지만 제공하고, 연·월·일로 바꾸는 달력 계산, 시간대(time zone), 날짜 문자열의 서식과 파싱은 제공하지 않습니다. 시간대 규칙은 나라마다 정하고 때때로 바뀌는 데이터라서, 언어와 함께 고정해 두기보다 따로 갱신하는 크레이트에 맡기는 편이 낫습니다. 예제 07은 공개된 알고리즘으로 UTC 날짜만 직접 계산해 보지만(`2001-09-09 01:46:40 UTC`), 실제 프로그램에서는 날짜·시간 크레이트를 씁니다.

**`Instant`냐 `SystemTime`이냐.**

| | `Instant` | `SystemTime` |
| --- | --- | --- |
| 시계 | 단조 시계 (뒤로 가지 않음) | 시스템 시계 (조정되면 뒤로 갈 수 있음) |
| 두 시점의 차이 | `Duration` (거꾸로면 0, `checked_`는 `None`) | `Result<Duration, SystemTimeError>` |
| 바깥 세계의 시각과의 관계 | 없음 (프로세스 안에서만 의미) | `UNIX_EPOCH` 기준 시각 |
| 쓰는 곳 | 실행 시간 측정, 제한 시간, 재시도 간격 | 타임스탬프 기록·저장, 파일 시각, 다른 프로그램과 시각 주고받기 |

기준은 "**얼마나 걸렸나**를 재면 `Instant`, **언제였나**를 기록하면 `SystemTime`"입니다. 두 타입은 서로 빼거나 비교할 수 없습니다(4.6절).

### 2.14 자식 기다리기와 제한 시간: `wait`, `try_wait`, `kill`

`spawn`으로 만든 `Child`는 이런 메서드로 다룹니다.

| 메서드 | 하는 일 |
| --- | --- |
| `wait(&mut self)` | 끝날 때까지 멈춰 기다리고 `ExitStatus`를 돌려줌. 기다리기 전에 `Child` 안의 `stdin`을 닫음. 다시 불러도 같은 결과 |
| `try_wait(&mut self)` | 기다리지 않음. 끝났으면 `Ok(Some(상태))`, 아직이면 `Ok(None)` |
| `wait_with_output(self)` | 남은 `stdout`·`stderr`를 다 읽고 기다림. `Child`를 소비함 |
| `kill(&mut self)` | 강제로 끝냄(유닉스에서는 SIGKILL). 이미 끝났으면 `Ok(())` |
| `id(&self)` | 운영체제가 준 프로세스 ID(PID) |

**좀비 프로세스.** 문서의 Warning 절은 이렇게 설명합니다. 끝난 자식이라도 부모가 `wait`로 거둬들이기(reap) 전까지는 운영체제가 자원을 풀지 않고 **좀비(zombie)** 로 남겨 둡니다. 좀비가 쌓이면 프로세스 ID 같은 전역 자원이 바닥날 수 있습니다. 그리고 **`Child`에는 `Drop` 구현이 없습니다.** `Child`가 스코프를 벗어나도 자식은 계속 실행되고, 아무도 거둬들이지 않습니다. 그래서 `spawn`했다면 어떤 경로로 끝나든 `wait`(또는 `try_wait`가 `Some`을 돌려줄 때까지, 또는 `wait_with_output`)를 불러야 합니다. Clippy는 `spawn`한 `Child`를 기다리지 않는 코드를 `zombie_processes` 경고로 잡아 주지만, 모든 모양을 잡지는 못합니다(4.6절, 연습 2). `kill`도 끝내기만 할 뿐 거둬들이지 않으므로 그 뒤에 `wait`가 필요합니다.

**제한 시간이 있는 기다리기.** 표준 라이브러리의 `Child`에는 제한 시간을 받는 `wait`가 없습니다. `try_wait`와 마감 시점(2.12절)을 합쳐 직접 만듭니다. [`08_child_timeout.rs`](examples/08_child_timeout.rs)의 `wait_timeout`입니다.

```rust
fn wait_timeout(child: &mut Child, limit: Duration) -> io::Result<Option<ExitStatus>> {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));     // 제때 끝남
        }
        if Instant::now() >= deadline {
            return Ok(None);             // 시간 초과: 어떻게 할지는 부른 쪽이 정함
        }
        thread::sleep(Duration::from_millis(10)); // 너무 자주 묻지 않도록 쉬면서 확인(폴링)
    }
}

// 부르는 쪽: 시간을 넘기면 kill한 뒤 반드시 wait로 거둬들입니다.
let status = match wait_timeout(&mut slow, Duration::from_millis(200))? {
    Some(status) => status,
    None => {
        slow.kill()?;
        slow.wait()?
    }
};
```

이렇게 주기적으로 상태를 물어보는 방식을 폴링(polling)이라고 합니다. 쉬는 간격이 길면 CPU를 덜 쓰는 대신 끝난 것을 늦게 알아채고, 짧으면 그 반대입니다. `kill`은 자식에게 정리할 기회를 주지 않고 끝냅니다(예제 08에서 종료 상태는 `signal: 9 (SIGKILL)`, `code()`는 `None`). 자식에게 먼저 "정리하고 끝내라"는 시그널을 보내는 부드러운 종료는 Part 7 `ch6-signals-config-exit-codes`에서 다룹니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_args_env.rs`](examples/01_args_env.rs) | `args_os`/`args`, `var`/`var_os`/`vars_os`, `VarError`, 환경 변수로 설정 받기, `current_dir`/`current_exe`, `env::consts` | 1 |
| [`examples/02_exit_code.rs`](examples/02_exit_code.rs) | `main`의 반환 타입별 종료 코드(`Termination::report`), `process::exit`와 소멸자, 8비트 잘림 | 2 |
| [`examples/03_command.rs`](examples/03_command.rs) | `Command` 빌더, 셸 없는 인자 전달, `env`/`env_remove`/`env_clear`, `current_dir`, `status`/`output`/`spawn`, 실행 실패와 자식의 실패, `sh -c` | 3 |
| [`examples/04_pipes.rs`](examples/04_pipes.rs) | `Stdio::piped`로 쓰고 읽기, 스레드로 교착 상태 피하기, `Stdio::null`, 두 자식 잇기, `sort \| uniq` | 4 |
| [`examples/05_duration.rs`](examples/05_duration.rs) | `Duration` 만들기·단위 변환·연산·넘치는 연산·출력 | 5 |
| [`examples/06_instant.rs`](examples/06_instant.rs) | `Instant`로 경과 시간과 실행 시간 재기, 시점의 차이, 마감 시점까지 재시도 | 5 |
| [`examples/07_system_time.rs`](examples/07_system_time.rs) | `SystemTime`과 `UNIX_EPOCH`, `Result`인 차이, UTC 날짜 직접 계산 | 5 |
| [`examples/08_child_timeout.rs`](examples/08_child_timeout.rs) | `try_wait`, 자식의 실행 시간, 제한 시간과 `kill`, 거둬들이기 | 4, 5 |
| [`compile_fail/`](compile_fail) | 이 챕터에서 자주 만나는 컴파일 에러 5개 (4절) | 1~5 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 끝납니다. 예제 02·03·04·08은 **자기 자신을 다시 실행**해 자식 프로세스로 씁니다. `env::current_exe()`로 얻은 실행 파일을 `child <모드>` 인자와 함께 실행하면 자식 모드로 동작하므로, 설치된 프로그램에 기대지 않고 어느 운영체제에서나 같은 결과가 나옵니다. 0이 아닌 종료 코드와 `process::exit`는 모두 자식 쪽에서 일어나고, 모든 자식은 `wait`로 거둬들입니다. 예제 03의 6번 구역과 04의 5번 구역만 유닉스 명령(`echo`, `sh`, `tr`, `sort`, `uniq`)을 쓰며 `#[cfg(unix)]`로 감쌌습니다. 경과 시간은 실행마다 다르므로 대부분 "50ms 이상인가" 같은 비교 결과로 출력합니다.

### 01_args_env — 프로세스가 받는 것

```sh
cargo run -p p1-ch10-env-process-time --example 01_args_env
```

```text
1. env::args_os: 인자를 검사 없이 OsString으로 받습니다
    개수(0번 포함): 1
2. env::args: String으로 받지만 UTF-8이 아닌 인자가 있으면 패닉합니다
    0번 인자의 파일 이름: Some("01_args_env")
    1번부터: []
3. 환경 변수 읽기
    P1CH10_GREETING: 설정되지 않음 (NotPresent)
    var_os("P1CH10_GREETING").is_some(): false
    "A=B": 설정되지 않음 (NotPresent)
    PATH 항목이 하나 이상: true
    환경 변수가 하나 이상: true
    작업자 수 설정: Ok(4)
4. 현재 디렉터리와 실행 파일
    current_dir는 절대 경로: true
    current_dir에 Cargo.toml이 있음: true
    current_exe 파일 이름: Some("01_args_env")
    current_exe가 든 디렉터리 이름: Some("examples")
5. 컴파일 대상 플랫폼 (env::consts)
    OS = "macos", FAMILY = "unix", ARCH = "aarch64"
    EXE_SUFFIX = "", DLL_EXTENSION = "dylib"
```

경로와 환경 변수 값은 컴퓨터마다 다르므로 예제는 파일 이름이나 "있는가"만 출력합니다. 5번 구역은 macOS(aarch64)의 값입니다. 인자와 환경 변수를 바꿔 실행하면 1~3번 구역이 이렇게 달라집니다(zsh나 bash의 `$'\xff'`는 UTF-8이 아닌 바이트 하나를 만듭니다).

```console
$ cargo run -q -p p1-ch10-env-process-time --example 01_args_env -- 하나 "둘 셋" $'\xff'
1. env::args_os: 인자를 검사 없이 OsString으로 받습니다
    개수(0번 포함): 4
    1번: UTF-8 "하나"
    2번: UTF-8 "둘 셋"
    3번: UTF-8 아님 "\xFF" → 손실 변환 "�"
2. env::args: String으로 받지만 UTF-8이 아닌 인자가 있으면 패닉합니다
    UTF-8이 아닌 인자가 있어 env::args()를 부르지 않습니다
$ P1CH10_GREETING=안녕 P1CH10_WORKERS=8 cargo run -q -p p1-ch10-env-process-time --example 01_args_env | grep -E 'GREETING:|작업자'
    P1CH10_GREETING: 설정됨, 값 "안녕"
    작업자 수 설정: Ok(8)
$ P1CH10_GREETING= P1CH10_WORKERS=many cargo run -q -p p1-ch10-env-process-time --example 01_args_env | grep -E 'GREETING:|작업자'
    P1CH10_GREETING: 설정됨, 빈 문자열
    작업자 수 설정: Err("P1CH10_WORKERS=\"many\"를 읽지 못했습니다: invalid digit found in string")
$ P1CH10_GREETING=$'\xff' cargo run -q -p p1-ch10-env-process-time --example 01_args_env | grep 'GREETING:'
    P1CH10_GREETING: UTF-8이 아님 (NotUnicode), 원래 값 "\xFF"
```

첫 번째 실행에서 2번 구역은 `env::args()`를 부르지 않았습니다. 불렀다면 Part 0 ch12에서 본 것처럼 패닉했을 것입니다. 디버그 형식의 `"\xFF"`는 UTF-8이 아닌 바이트를 보여 주는 `OsString`의 표기입니다.

### 02_exit_code — 프로세스가 돌려주는 것

```sh
cargo run -p p1-ch10-env-process-time --example 02_exit_code
```

```text
1. main이 돌려준 값과 종료 코드
    ()                       → 종료 코드 0
    Ok::<(), &str>(())       → 종료 코드 0
    Err::<(), &str>("..")    → 종료 코드 1, 표준 에러 "Error: \"port 설정이 없습니다\""
    Ok::<ExitCode, &str>(2)  → 종료 코드 2
    ExitCode::from(42)       → 종료 코드 42
    ExitCode::FAILURE        → 종료 코드 1
    panic!("..")             → 종료 코드 101, 표준 에러 "thread 'main' (…) panicked at …"
2. 돌려주며 끝내기와 process::exit의 차이
    return ExitCode::from(3) → 종료 코드 3, 표준 출력 "BufWriter에 쓴 줄\n[drop] return 정리"
    process::exit(3)         → 종료 코드 3
3. 유닉스에서는 하위 8비트만 전달됩니다
    process::exit(256) → 종료 코드 0
    process::exit(-1)  → 종료 코드 255
4. ExitCode는 비교만 할 수 있습니다
    ExitCode::default() == SUCCESS: true
    ExitCode::from(1) == FAILURE: true
```

- 1번 구역의 자식은 모드마다 `main`이 그 값을 돌려준 것처럼 `report()`를 직접 부릅니다. 패닉 메시지에는 실행마다 다른 스레드 ID가 들어 있어서 예제가 앞부분만 남겼습니다.
- 2번 구역에서 `return` 쪽은 `BufWriter`가 먼저, `_guard`가 나중에 drop되었습니다(선언의 역순). `process::exit` 쪽은 둘 다 실행되지 않았습니다.
- 자식 하나를 셸에서 직접 실행해 종료 코드를 볼 수도 있습니다. `cargo run`은 실행한 프로그램의 종료 코드를 그대로 돌려줍니다.

```console
$ cargo run -q -p p1-ch10-env-process-time --example 02_exit_code -- child code42; echo $?
42
$ cargo run -q -p p1-ch10-env-process-time --example 02_exit_code -- child err; echo $?
Error: "port 설정이 없습니다"
1
```

### 03_command — 자식 프로세스 실행하기

```sh
cargo run -p p1-ch10-env-process-time --example 03_command
```

```text
1. 인자는 한 개씩, 셸 해석 없이 전달됩니다
    [자식] 받은 인자 5개: ["공백 있는 인자", "*.rs", "$HOME", "a; echo 주입", "x | y"]
    get_args(): ["child", "args", "공백 있는 인자", "*.rs", "$HOME", "a; echo 주입", "x | y"]
2. 환경 변수는 물려받고, 자식 몫만 바꿀 수 있습니다
    그대로:      [자식] P1CH10_COLOR=(없음), PATH 있음=true, 환경 변수 0개=false
    env 추가:    [자식] P1CH10_COLOR=blue, PATH 있음=true, 환경 변수 0개=false
    env_remove:  [자식] P1CH10_COLOR=(없음), PATH 있음=false, 환경 변수 0개=false
    env_clear:   [자식] P1CH10_COLOR=(없음), PATH 있음=false, 환경 변수 0개=true
    부모의 P1CH10_COLOR: Err(NotPresent)
3. current_dir로 자식의 현재 디렉터리를 정합니다
    [자식] 현재 디렉터리 이름: "p1_ch10_03_workdir"
4. 실행하는 세 가지 방법
  [status]
[자식] 표준 출력으로 한 줄
[자식] 표준 에러로 한 줄
    종료 상태: exit status: 0, success() = true
  [output]
    status = exit status: 0
    stdout = "[자식] 표준 출력으로 한 줄\n"
    stderr = "[자식] 표준 에러로 한 줄\n"
  [spawn]
    spawn이 돌아옴, 자식 PID가 있음: true
    wait 결과: exit status: 3, code() = Some(3)
5. 실행 실패(Err)와 자식의 실패(Ok + 0이 아닌 종료 상태)는 다릅니다
    실행하지 못함: kind = NotFound, No such file or directory (os error 2)
    실행됨: success() = false, code() = Some(3)
6. (macOS/Linux) 셸 문법은 셸에게 맡겨야 해석됩니다
    echo 직접 실행: $HOME | tr a-z A-Z
    sh -c 실행: HELLO; ECHO INJECTED
```

- 4번 구역의 `[status]` 아래 두 줄은 들여쓰기가 없습니다. 자식이 부모와 같은 표준 출력·표준 에러에 직접 썼기 때문입니다. 두 번째 줄은 표준 에러이므로 `> 파일`로 리다이렉션해도 화면에 남습니다. `[output]`에서는 같은 두 줄이 부모의 변수에 모였습니다.
- 3번 구역은 `env::temp_dir()` 아래에 `p1_ch10_03_workdir` 디렉터리를 만들었다가 끝날 때 지웁니다.
- 6번 구역의 `sh -c` 스크립트는 `printf '%s\n' "$1" | tr a-z A-Z`이고, `"hello; echo injected"`는 `$1`로 넘긴 **데이터**라서 `;` 뒤가 명령으로 실행되지 않고 대문자로 바뀌기만 했습니다.

### 04_pipes — 파이프로 주고받기

```sh
cargo run -p p1-ch10-env-process-time --example 04_pipes
```

```text
1. 자식의 표준 입력에 쓰고, 표준 출력을 읽습니다
[자식] 3줄, 18바이트 받음
    받은 출력: "HELLO\nRUST\n안녕\n"
2. 큰 데이터는 다른 스레드에서 씁니다
    보낸 988890바이트, 받은 988890바이트
3. Stdio::null과 output()의 표준 입력
    stdin·stderr를 null로: 출력 ""
    output()의 표준 에러: "[자식] 0줄, 0바이트 받음\n"
4. 두 자식을 파이프로 잇습니다 (numbers | sum)
    합계 15
    앞쪽: exit status: 0, 뒤쪽: exit status: 0
5. (macOS/Linux) sort | uniq
    "apple\nbanana\ncherry\n"
```

- 1번 구역의 `[자식] ...` 줄은 표준 에러를 `piped`로 설정하지 않아 자식이 물려받은 표준 에러에 직접 쓴 것입니다. `"안녕\n"`이 7바이트(한글 한 글자 3바이트)라 합계가 18바이트입니다. `to_uppercase`는 한글을 바꾸지 않습니다.
- 3번 구역의 첫 줄은 표준 입력과 표준 에러를 모두 `null`로 해서 입력은 비고 자식의 알림은 버려졌습니다. 둘째 줄은 `output()`의 표준 입력이 빈 입력이라는 확인입니다.

### 05_duration — 시간의 길이

```sh
cargo run -p p1-ch10-env-process-time --example 05_duration
```

```text
1. 만들기
    new(1, 500_000_000) == from_millis(1500): true
    from_mins(90) == from_hours(1) + from_secs(1800): true
    from_secs_f64(0.25) = 250ms
    ZERO = 0ns, default() = 0ns
    MAX = 18446744073709551615.999999999s
2. 단위로 꺼내기
    as_secs = 2, subsec_millis = 750
    as_millis = 2750, as_nanos = 2750000000
    as_secs_f64 = 2.75
3. 연산
    합계 = 3.23s
    평균 = 1.076666666s
    가장 긴 구간 = 1.2s
    1.2s와 980ms의 차이 = 220ms
    250ms × 1.5 = 375ms
    45초는 1분의 75%
4. 넘칠 수 있는 연산
    300ms.checked_sub(1s) = None
    300ms.saturating_sub(1s) = 0ns
    MAX.checked_add(1ns) = None
    MAX.checked_mul(2) = None
    try_from_secs_f64(-1.0) = Err(TryFromFloatSecsError { kind: Negative })
    try_from_secs_f64(NaN) 에러: cannot convert float seconds to Duration: value is either too big or NaN
5. 출력
         1.5µs |   1.50µs | 00:00:00.000
         2.5ms |   2.50ms | 00:00:00.002
        1.001s |    1.00s | 00:00:01.001
     3725.042s | 3725.04s | 01:02:05.042
```

3번 구역의 평균 `1.076666666s`처럼 정수 나눗셈은 나노초 아래를 버립니다. 5번 구역의 세 열은 각각 `{:?}`, `{:.2?}`, 직접 만든 `hms` 함수입니다.

### 06_instant — 경과 시간 재기

```sh
cargo run -p p1-ch10-env-process-time --example 06_instant
```

```text
1. now()와 elapsed()
    50ms 이상 지남: true
    1초 미만: true
2. 실행 시간 재기
    결과 = 333333833333500000
    걸린 시간 = 17.3ms
    1초 미만: true
3. 두 시점의 차이
    later > earlier: true
    later - earlier >= 10ms: true
    earlier.duration_since(later) = 0ns
    earlier.checked_duration_since(later) = None
4. 마감 시점까지 다시 시도하기
    1초 안에, 3번째에 성공하는 작업: Ok(("성공", 3))
    90ms 이상 걸림: true
    100ms 안에 끝나지 않는 작업: 성공 = false
    100ms 이상 지나서 포기: true
```

`걸린 시간` 줄만 실행마다 다릅니다(이 환경의 디버그 빌드에서 9~17ms, 릴리스 빌드에서 40ns 안팎. 2.12절). 4번 구역의 두 번째 작업은 시도마다 30ms가 걸리므로 100ms 동안 서너 번 시도하지만, 정확한 횟수는 잠든 시간의 오차에 따라 달라질 수 있어 출력하지 않습니다.

### 07_system_time — 시스템 시계

```sh
cargo run -p p1-ch10-env-process-time --example 07_system_time
```

```text
1. 지금의 유닉스 시간
    1_700_000_000초 이후: true
2. 정해진 시각 만들기
    UNIX_EPOCH  → Some("1970-01-01 00:00:00 UTC")
    10억 초     → Some("2001-09-09 01:46:40 UTC")
    i32::MAX 초 → Some("2038-01-19 03:14:07 UTC")
    SystemTime { tv_sec: 1000000000, tv_nsec: 0 }
3. 두 시각의 차이는 Result입니다
    billion.duration_since(UNIX_EPOCH) = Ok(1000000000s)
    거꾸로: second time provided was later than self (차이 1000000000s)
    내일 시각의 elapsed() 성공: false
    연달아 잰 두 시각의 차이 1초 미만: true
4. 지금 날짜 (UTC)
    "YYYY-MM-DD hh:mm:ss UTC" 모양: true
```

- 2번 구역의 `i32::MAX` 초는 초를 부호 있는 32비트 정수로 세던 시스템이 넘치는 시각("2038년 문제")입니다. `SystemTime`의 디버그 형식은 플랫폼의 내부 표현이라 유닉스 계열에서의 모양입니다.
- `civil_from_days` 함수는 이 문서를 쓰면서 1970년부터 약 550년 동안의 날짜를 다른 구현(파이썬의 `datetime`)과 비교해 같다는 것을 확인했습니다. 연습 삼아 읽어 볼 수는 있지만 외울 필요는 없습니다.

### 08_child_timeout — 기다리기와 제한 시간

```sh
cargo run -p p1-ch10-env-process-time --example 08_child_timeout
```

```text
1. try_wait는 기다리지 않습니다
    바로 물어본 결과: None
    끝난 뒤의 결과: exit status: 0
    150ms 이상 지남: true
    다시 wait해도 같음: true
2. 자식 프로세스의 실행 시간 재기
    exit status: 0, 200ms 이상: true, 2초 미만: true
3. 제한 시간을 넘긴 자식은 kill로 끝냅니다
    50ms 자식, 제한 1초: 제때 끝남, exit status: 0
    5초 자식, 제한 200ms: 시간 초과 → kill
    signal: 9 (SIGKILL), success() = false, code() = None
    signal() = Some(9)
    5초를 다 기다리지 않음: true
    끝난 자식에게 kill: Ok(())
```

예제 전체는 1초 정도 걸립니다. `signal()`은 유닉스 전용 확장 트레이트 `std::os::unix::process::ExitStatusExt`의 메서드라 `#[cfg(unix)]` 안에서 불렀습니다. 2번 구역의 시간에는 자식이 잠든 200ms 외에 프로세스를 만들고 끝내는 비용도 들어 있습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch10-env-process-time
rustc --edition 2024 compile_fail/e0277_main_returns_i32.rs --out-dir /tmp/rust-p1-ch10
```

### 4.1 `main`에서 정수를 돌려줌 — E0277

파일: [`compile_fail/e0277_main_returns_i32.rs`](compile_fail/e0277_main_returns_i32.rs)

```text
error[E0277]: `main` has invalid return type `i32`
  --> compile_fail/e0277_main_returns_i32.rs:11:14
   |
11 | fn main() -> i32 {
   |              ^^^ `main` can only return types that implement `Termination`
   |
   = help: consider using `()`, or a `Result`
```

- **읽는 법**: `main`은 `Termination`을 구현한 타입만 돌려줄 수 있는데 `i32`는 아니라는 뜻입니다.
- **원인**: C의 `int main()`처럼 반환값이 곧 종료 코드가 되리라 기대했습니다. Rust는 플랫폼마다 쓸 수 있는 범위가 다른 `i32`를 허락하지 않습니다(2.6절).
- **해결**: `help:`는 `()`나 `Result`를 권하지만, 종료 코드를 직접 고르고 싶다면 `fn main() -> ExitCode`로 바꾸고 `return ExitCode::from(2);`, 마지막에 `ExitCode::SUCCESS`를 씁니다. 이렇게 고친 프로그램은 인자 없이 실행하면 사용법을 출력하고 종료 코드 2로 끝납니다.

### 4.2 2024 에디션에서 `unsafe` 없이 `env::set_var` — E0133

파일: [`compile_fail/e0133_set_var_in_2024.rs`](compile_fail/e0133_set_var_in_2024.rs)

```text
error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block
  --> compile_fail/e0133_set_var_in_2024.rs:16:5
   |
16 |     env::set_var("P1CH10_COLOR", "blue");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
```

- **읽는 법**: `unsafe` 함수를 `unsafe` 블록 밖에서 불렀다는, [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)에서 본 것과 같은 에러입니다. 다른 점은 이 함수가 **에디션에 따라** 달라진다는 것입니다. 같은 파일을 `--edition 2021`로 컴파일하면 에러 없이 통과합니다(경고를 켜는 `-W deprecated-safe-2024`를 주면 "2024에서는 에러"라는 경고가 나옵니다). `note:`는 문서의 Safety 절을 읽으라고 안내합니다.
- **원인**: 자식 프로세스에 환경 변수를 넘기려고 내 프로세스의 환경을 먼저 바꿨습니다. 여러 스레드가 있는 프로그램에서 환경을 바꾸는 것은 안전하게 할 방법이 없습니다(2.4절).
- **해결**: `unsafe` 블록으로 감싸는 것이 아니라 설계를 바꿉니다. 자식에게 넘길 값이면 `Command::new("printenv").arg("P1CH10_COLOR").env("P1CH10_COLOR", "blue")`처럼 자식 몫만 정합니다. 고친 코드는 `blue`와 `exit status: 0`을 출력합니다.

### 4.3 `Duration`에 실수를 곱함 — E0308

파일: [`compile_fail/e0308_duration_times_float.rs`](compile_fail/e0308_duration_times_float.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_duration_times_float.rs:17:25
   |
17 |         delay = delay * 1.5;
   |                         ^^^ expected `u32`, found floating-point number
```

- **읽는 법**: `*`의 오른쪽에 `u32`가 와야 하는데 실수가 왔다는 뜻입니다. "곱할 수 없다"(E0277)가 아니라 "타입이 다르다"(E0308)로 나오는 것은, `Duration`을 왼쪽에 둔 곱셈 구현이 `impl Mul<u32> for Duration` 하나뿐이라 컴파일러가 오른쪽을 `u32`로 정해 버렸기 때문입니다. 변수를 `let x: f64 = 1.5;`로 만들어 곱해도 ``expected `u32`, found `f64` ``로 같은 에러입니다.
- **원인**: 재시도 간격을 1.5배씩 늘리려고 실수를 곱했습니다. 실수배는 연산자로 제공되지 않습니다(2.11절).
- **해결**: `delay = delay.mul_f64(1.5);` 고친 코드는 100ms, 150ms, 225ms를 차례로 출력합니다. 정수배라면 `delay * 2`가 됩니다.

### 4.4 자식의 출력(`Vec<u8>`)을 `&str`처럼 씀 — E0308

파일: [`compile_fail/e0308_stdout_bytes_as_str.rs`](compile_fail/e0308_stdout_bytes_as_str.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_stdout_bytes_as_str.rs:19:29
   |
19 |     let words = count_words(&output.stdout);
   |                 ----------- ^^^^^^^^^^^^^^ expected `&str`, found `&Vec<u8>`
   |                 |
   |                 arguments to this function are incorrect
   |
   = note: expected reference `&str`
              found reference `&Vec<u8>`
```

- **읽는 법**: `&str`을 받는 함수에 `&Vec<u8>`을 넘겼다는 뜻입니다.
- **원인**: `Output`의 `stdout`은 바이트입니다. 프로세스의 출력이 UTF-8이라는 보장은 없으므로(이미지를 출력하는 프로그램도 있습니다) 표준 라이브러리가 문자열로 바꿔 주지 않습니다(2.9절).
- **해결**: UTF-8이 아니면 에러로 다루려면 `String::from_utf8(output.stdout)`(→ `Result<String, FromUtf8Error>`), 깨진 바이트를 `�`로 바꿔도 된다면 `count_words(&String::from_utf8_lossy(&output.stdout))`를 씁니다. 고친 코드는 `단어 3개`를 출력합니다.

### 4.5 `child.stdout`을 `unwrap()`으로 꺼낸 뒤 `wait` — E0382

파일: [`compile_fail/e0382_child_stdout_partial_move.rs`](compile_fail/e0382_child_stdout_partial_move.rs)

```text
error[E0382]: borrow of partially moved value: `child`
  --> compile_fail/e0382_child_stdout_partial_move.rs:22:18
   |
19 |     let mut stdout = child.stdout.unwrap();
   |                                   -------- `child.stdout` partially moved due to this method call
...
22 |     let status = child.wait().expect("기다리기");
   |                  ^^^^^ value borrowed here after partial move
   |
note: `std::option::Option::<T>::unwrap` takes ownership of the receiver `self`, which moves `child.stdout`
   = note: partial move occurs because `child.stdout` has type `Option<ChildStdout>`, which does not implement the `Copy` trait
```

`note:` 아래에 표준 라이브러리 소스 위치(`/rustc/0ef1bc38.../library/core/src/option.rs`)가 한 줄 더 나오고, `help:`가 `.as_ref()`, `.as_mut()`, `.clone()` 세 가지를 제안합니다. 인용에서는 생략했습니다.

- **읽는 법**: `child`의 일부(`stdout` 필드)가 이미 옮겨졌으므로 `child` 전체를 빌리는 `wait`를 부를 수 없다는 뜻입니다. Part 0 ch5에서 본 부분 이동(partial move)입니다.
- **원인**: `Option::unwrap`은 `self`를 받으므로 `child.stdout`을 `child` 밖으로 옮깁니다. `wait`는 `&mut self`가 필요한데 일부가 빠진 값은 빌릴 수 없습니다.
- **해결**: `child.stdout.take().unwrap()`으로 꺼냅니다. `take()`는 `&mut self`로 값을 꺼내고 자리에 `None`을 남기므로 `child`는 온전합니다(2.10절). `help:`가 제안하는 `as_mut()`도 컴파일되지만, 그때 `ChildStdout`은 여전히 `child` 안에 있으므로 다른 자식의 입력으로 넘기는 일(`Stdio::from`)에는 쓸 수 없습니다. Rust By Example의 Pipes 예제처럼 `child.stdout.unwrap()`을 써도 컴파일되는 경우는 그 뒤로 `child`의 메서드를 부르지 않을 때뿐이고, 그러면 자식을 거둬들이지 못합니다(연습 2).

### 4.6 그 밖의 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| `Command::new("ls").arg(5)` | E0277 ``the trait bound `{integer}: AsRef<OsStr>` is not satisfied`` | 인자는 문자열이어야 합니다. `.arg(5.to_string())`이나 `.arg(format!("{n}"))` |
| `let cmd = Command::new("echo").arg("hi");` 뒤에 `cmd.status()` | E0716 `temporary value dropped while borrowed` | `arg`는 `&mut Command`를 돌려줄 뿐입니다. `let mut cmd = Command::new("echo"); cmd.arg("hi");`처럼 먼저 변수에 담습니다(2.8절). |
| `instant - system_time` | E0277 ``cannot subtract `SystemTime` from `Instant` `` | 두 시계는 섞을 수 없습니다. 측정에는 `Instant`끼리, 기록에는 `SystemTime`끼리 씁니다(2.13절). |
| `ExitCode::from(300)` | 에러 ``literal out of range for `u8` `` (기본으로 deny인 lint `overflowing_literals`) | `ExitCode`는 0~255만 담습니다. 이 범위를 넘는 값을 쓰지 않습니다(2.6절). |
| `Command::new("ls -l")` | 실행 중 `Err`, `NotFound` | 프로그램 이름과 인자를 나눕니다. `Command::new("ls").arg("-l")` |
| `.arg("-l /tmp")` | Clippy `suspicious_command_arg_space`(기본 warn), 실행하면 macOS의 `ls`가 공백을 잘못된 옵션이라며 실패 | `.args(["-l", "/tmp"])`. 인자는 셸이 나눠 주지 않습니다(2.8절). |
| `spawn()`한 `Child`를 `wait` 없이 버림 | Clippy `zombie_processes`(기본 warn): ``spawned process is never `wait()`ed on``. 단, 필드를 옮겨 쓴 경우처럼 잡지 못하는 모양도 있음(연습 2) | 모든 경로에서 `wait`·`try_wait`·`wait_with_output` 중 하나로 거둬들입니다(2.14절). |
| `take()`로 꺼낸 `stdin`을 닫지 않고 `wait_with_output` | 끝나지 않고 멈춤 | `drop(stdin)`으로 닫거나, 쓰기를 맡긴 스레드가 끝나면서 닫히게 합니다(2.10절). |
| 큰 입력을 다 쓴 뒤에 출력을 읽음 | `write_all`에서 멈춤(교착 상태) | 쓰기를 다른 스레드에 맡깁니다(2.10절). |
| `env::vars()`로 환경 전체를 훑음 | 값 하나라도 UTF-8이 아니면 패닉 ``called `Result::unwrap()` on an `Err` value: "\xFF"`` | `env::vars_os()`를 씁니다(2.3절). |
| 성능을 디버그 빌드에서 잼 | 실제와 동떨어진 시간 | `--release`로 재고, `black_box`로 계산이 지워지지 않게 합니다(2.12절). |

## 5. 연습 문제

### 연습 1. `timeit`: 명령을 실행하고 시간을 재는 프로그램

`examples/09_timeit.rs`를 만들어, 1번 이후의 인자를 명령과 인자로 실행하고 걸린 시간을 표준 에러에 출력한 뒤 **자식의 종료 코드를 그대로 자기 종료 코드로** 돌려주게 하세요. 명령을 찾지 못하면 127, 인자가 없으면 사용법을 출력하고 2로 끝냅니다(macOS/Linux 기준). 시간은 실행마다 조금씩 다릅니다.

```console
$ cargo run -q -p p1-ch10-env-process-time --example 09_timeit -- sleep 0.3; echo $?
[timeit] sleep: 0.31s, exit status: 0
0
$ cargo run -q -p p1-ch10-env-process-time --example 09_timeit -- sh -c "exit 7"; echo $?
[timeit] sh: 0.01s, exit status: 7
7
$ cargo run -q -p p1-ch10-env-process-time --example 09_timeit -- no-such-cmd; echo $?
[timeit] no-such-cmd: 실행하지 못함 (No such file or directory (os error 2))
127
```

<details>
<summary>힌트</summary>

- 명령은 `env::args_os().skip(1)`로 받으면 UTF-8이 아닌 인자도 그대로 넘길 수 있습니다. `let mut parts = env::args_os().skip(1);` 뒤에 `parts.next()`가 프로그램, 나머지는 `Command::args(parts)`로 넘깁니다.
- 자식이 사용자와 직접 입출력하도록 `status()`를 씁니다(2.9절). 시간은 `Instant::now()`와 `elapsed()`, 출력은 `{:.2}`와 `as_secs_f64()`.
- `main`은 `ExitCode`를 돌려줍니다. 자식의 `status.code()`가 `Some(c)`면 `ExitCode::from(c as u8)`, 시그널로 끝나 `None`이면 `ExitCode::FAILURE`로 정합니다. `status()`가 `Err(e)`를 돌려줬고 `e.kind() == io::ErrorKind::NotFound`면 127입니다. `as u8`은 하위 8비트를 남기므로 유닉스에서 부모가 본 값과 같습니다.

</details>

### 연습 2. Rust By Example의 Pipes 예제 고치기

[Rust By Example의 Pipes](https://doc.rust-lang.org/rust-by-example/std_misc/process/pipe.html) 예제는 `wc`의 표준 입력에 문장을 쓰고 출력을 읽습니다. 이 예제는 `process.stdin.unwrap()`과 `process.stdout.unwrap()`으로 필드를 옮겨 쓰고, 자식을 `wait`하지 않습니다. 이 챕터에서 배운 대로 다음을 고치세요.

1. `take()`로 꺼내고, 쓴 뒤에는 `drop`으로 닫습니다.
2. `wait_with_output()`으로 출력을 받고 자식을 거둬들인 뒤, `status.success()`가 아니면 에러로 알립니다.
3. `wc` 대신 이 챕터의 방식대로 자기 자신을 자식으로 실행해 줄·단어·바이트 수를 세게 하면, 운영체제와 상관없이 동작합니다.

<details>
<summary>힌트</summary>

- 원래 코드를 Clippy로 검사해 보면 1.100.0에서는 `zombie_processes` 경고가 나지 않습니다. 필드를 옮겨 쓰고 `process` 자체는 다시 쓰지 않는 모양을 lint가 알아보지 못하기 때문으로 보입니다. lint가 조용하다고 자식을 거둬들인 것은 아니라는 예입니다.
- 줄·단어·바이트 수는 자식 모드에서 `io::stdin().read_to_string(&mut text)` 뒤 `text.lines().count()`, `text.split_whitespace().count()`, `text.len()`으로 셉니다.
- 예제 04의 1번 구역이 거의 같은 모양입니다.

</details>

### 연습 3. "3분 전"처럼 보여 주기

파일 목록 프로그램이 수정 시각을 사람이 읽기 좋게 보여 주도록 `fn ago(then: SystemTime, now: SystemTime) -> String`을 만드세요. 차이가 1분 미만이면 `"방금"`, 1시간 미만이면 `"N분 전"`, 하루 미만이면 `"N시간 전"`, 그 이상이면 `"N일 전"`입니다. `then`이 `now`보다 나중이면(시계가 조정된 경우) `"미래"`를 돌려줍니다. 테스트하기 쉽도록 `now`를 인자로 받습니다.

```rust
let now = UNIX_EPOCH + Duration::from_secs(1_000_000_000);
assert_eq!(ago(now - Duration::from_secs(30), now), "방금");
assert_eq!(ago(now - Duration::from_secs(185), now), "3분 전");
assert_eq!(ago(now - Duration::from_hours(5), now), "5시간 전");
assert_eq!(ago(now + Duration::from_secs(10), now), "미래");
```

<details>
<summary>힌트</summary>

- `now.duration_since(then)`이 `Err`면 `"미래"`입니다(2.13절). `Ok(d)`면 `d.as_secs()`를 60, 3600, 86400과 비교합니다.
- `SystemTime - Duration`은 `SystemTime`입니다. `Duration::from_hours`는 1.91.0부터 있습니다.
- 실제 파일의 수정 시각은 `fs::metadata(경로)?.modified()?`로 얻습니다. 이것을 `ago(modified, SystemTime::now())`에 넘겨 보세요.

</details>

### 연습 4. 세 자식을 잇는 파이프라인

예제 04의 자식 모드에 "짝수만 통과시키는" `evens` 모드를 추가하고, `numbers | evens | sum`을 Rust 코드로 이으세요. `numbers`가 1부터 10까지 출력하도록 바꾸면 결과는 `합계 30`입니다. 세 자식의 종료 상태를 모두 확인합니다.

<details>
<summary>힌트</summary>

- 가운데 자식은 `stdin(Stdio::from(앞쪽.stdout.take()...))`과 `stdout(Stdio::piped())`를 둘 다 설정합니다. 그 `stdout`을 다시 `take()`해 마지막 자식의 입력으로 넘깁니다.
- 마지막 자식은 `wait_with_output()`, 앞의 두 자식은 `wait()`로 거둬들입니다. 거둬들이는 순서는 상관없지만 하나라도 빠뜨리면 좀비가 남습니다(2.14절).
- `evens` 모드는 `io::stdin().lock().lines()`로 한 줄씩 읽어 `n % 2 == 0`일 때만 `println!`합니다.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::env`](https://doc.rust-lang.org/std/env/index.html): 이 모듈의 함수 목록. 이름이 `_os`로 끝나는 함수는 `OsString`을, 아닌 함수는 `String`을 돌려준다는 규칙이 첫머리에 있습니다.
  - [`env::set_var`](https://doc.rust-lang.org/std/env/fn.set_var.html): 2.4절의 Safety 절 전문과 `Command::env`를 쓰라는 안내.
  - [`std::process`](https://doc.rust-lang.org/std/process/index.html): 파이프로 두 명령을 잇는 예와, 교착 상태를 피하려고 다른 스레드에서 쓰는 예(Handling I/O 절), Windows의 인자 처리 주의.
  - [`Command`](https://doc.rust-lang.org/std/process/struct.Command.html): 빌더 메서드마다 기본값, `PATH` 검색 규칙(`new`의 Platform-specific behavior 절), 인자가 셸을 거치지 않는다는 설명(`arg`).
  - [`Child`](https://doc.rust-lang.org/std/process/struct.Child.html): 좀비 프로세스와 `Drop`이 없다는 Warning 절, `wait`·`try_wait`·`kill`.
  - [`ExitCode`](https://doc.rust-lang.org/std/process/struct.ExitCode.html), [`Termination`](https://doc.rust-lang.org/std/process/trait.Termination.html), [`process::exit`](https://doc.rust-lang.org/std/process/fn.exit.html): 2.6~2.7절. `ExitCode`와 `ExitStatus`를 나눈 이유, 소멸자가 실행되지 않는다는 설명, 8비트 잘림.
  - [`std::time`](https://doc.rust-lang.org/std/time/index.html), [`Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html): `Display`가 없는 이유(Formatting Duration values 절).
  - [`Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html): 단조성 보장과 그 한계(Monotonicity 절), 플랫폼별로 쓰는 시스템 호출.
  - [`SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html): 시스템 시계가 단조적이지 않아 차이가 `Result`라는 설명, `UNIX_EPOCH`.
- Rust By Example (Std misc 장)
  - [Program arguments](https://doc.rust-lang.org/rust-by-example/std_misc/arg.html): `env::args`의 기본 사용과, 본격적인 명령줄 프로그램에는 `clap` 같은 크레이트를 쓴다는 안내.
  - [Child processes](https://doc.rust-lang.org/rust-by-example/std_misc/process.html): `output()`과 `success()`, `from_utf8_lossy`로 결과를 보는 예. 페이지의 권유대로 `rustc`에 잘못된 옵션을 줘서 실패 갈래도 확인해 보세요.
  - [Pipes](https://doc.rust-lang.org/rust-by-example/std_misc/process/pipe.html): 입력을 닫아야 자식이 처리를 시작한다는 설명. 연습 2에서 이 예제를 고칩니다.
  - [Wait](https://doc.rust-lang.org/rust-by-example/std_misc/process/wait.html): `spawn` 뒤 `wait`로 자식이 끝나기를 기다리는 가장 작은 예.
- 에러 설명: `rustc --explain E0133`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
