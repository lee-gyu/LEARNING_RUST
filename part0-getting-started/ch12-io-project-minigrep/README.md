# ch12. 입출력 프로젝트: 명령줄 프로그램 minigrep 만들기

지금까지 배운 것을 모아 실제로 쓸 수 있는 작은 명령줄 프로그램을 만듭니다. 파일에서 검색어가 들어 있는 줄을 찾아 출력하는 `grep`의 간단한 버전, `minigrep`입니다. 명령줄 인자(command line argument)를 받고, 파일을 읽고, 코드를 라이브러리와 바이너리로 나누고, 테스트를 먼저 쓰며 기능을 더하고, 환경 변수(environment variable)로 동작을 바꾸고, 에러 메시지를 표준 에러로 보내는 과정을 한 단계씩 밟습니다.

이 챕터의 중심은 **"왜 이렇게 바꾸는가"** 입니다. 처음에는 일단 동작하는 코드를 쓰고, 그 코드의 문제를 직접 관찰한 다음 고칩니다. 앞 단계(인자 받기부터 에러 처리까지)는 `examples/`에 단계마다 파일 하나로 두었고, 완성판은 독립 프로젝트 `projects/minigrep`에 있습니다.

- 패키지: `p0-ch12-io-project-minigrep`
- 독립 프로젝트: `projects/minigrep`
- 기준 문서: [The Book 12장 An I/O Project: Building a Command Line Program](https://doc.rust-lang.org/book/ch12-00-an-io-project.html)
- 확인한 툴체인: rustc 1.98.1, cargo 1.98.1 (Edition 2024)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `std::env::args`로 명령줄 인자를 벡터에 모으고, `cargo run`에 `--`를 붙여 프로그램에 인자를 넘길 수 있다.
2. `fs::read_to_string`으로 파일을 읽고, 인자 부족이나 파일 없음 같은 사용자 실수를 패닉 대신 `Result`와 종료 코드로 알리도록 코드를 리팩터링할 수 있다.
3. 바이너리 프로젝트의 관심사 분리 지침에 따라 로직을 `src/lib.rs`로 옮기고 `src/main.rs`를 얇게 유지하는 이유를 설명할 수 있다.
4. "실패하는 테스트 → 통과하는 최소 구현 → 리팩터링" 순서(TDD)로 검색 함수를 작성하고, `search`의 반환값에 라이프타임 `'a`가 필요한 이유를 설명할 수 있다.
5. 환경 변수로 프로그램의 동작을 바꾸고, 에러 메시지를 `eprintln!`으로 표준 에러에 보내 검색 결과와 섞이지 않게 할 수 있다.

## 2. 핵심 개념

### 2.1 만들 프로그램과 진행 순서

완성된 `minigrep`은 이렇게 동작합니다.

```console
$ minigrep frog poem.txt
How public, like a frog
```

첫 번째 인자는 검색어, 두 번째 인자는 파일 경로입니다. 파일을 읽어 검색어가 들어 있는 줄만 출력합니다. 진짜 `grep`은 정규 표현식(regular expression)으로 검색하지만 `minigrep`은 문자열이 들어 있는지만 봅니다. Rust로 만든 실전용 검색 도구로는 [ripgrep](https://github.com/BurntSushi/ripgrep)이 있습니다.

이 챕터는 다음 순서로 진행합니다. `examples/`의 파일은 같은 프로그램을 단계마다 찍어 둔 사진과 같습니다. 파일마다 완전한 프로그램이므로 두 파일을 나란히 놓고 비교하면 그 단계에서 무엇이 바뀌었는지 보입니다.

| 단계 | 파일 | 하는 일 | The Book |
| --- | --- | --- | --- |
| 1 | [`examples/01_args.rs`](examples/01_args.rs) | 명령줄 인자를 벡터로 모으기 | 12.1 |
| 2 | [`examples/02_read_file.rs`](examples/02_read_file.rs) | 인자를 변수에 담고 파일 읽기 (검사 없는 초안) | 12.2 |
| 3 | [`examples/03_config.rs`](examples/03_config.rs) | 설정 값을 `Config`로 묶고, 인자 부족을 `Result`로 알리기 | 12.3 |
| 4 | [`examples/04_run.rs`](examples/04_run.rs) | 로직을 `run`으로 빼고, 파일 에러도 `Result`로 돌려주기 | 12.3 |
| 5 | [`projects/minigrep`](projects/minigrep) | 로직을 `lib.rs`로 옮기기 | 12.3 |
| 6 | 〃 | 테스트를 먼저 쓰고 `search` 구현하기 (TDD) | 12.4 |
| 7 | 〃 | 환경 변수 `IGNORE_CASE`로 대소문자 무시하기 | 12.5 |
| 8 | 〃 | 에러 메시지를 표준 에러로 보내기 | 12.6 |

앞 챕터에서 배운 것이 이렇게 쓰입니다: 소유권과 복제(ch4), 구조체와 `impl`(ch5), `match`와 `if let`(ch6), 라이브러리와 바이너리를 함께 가진 패키지(ch7), `Vec`과 `String`(ch8), `Result`·`?`·`Box<dyn Error>`(`ch9-error-handling`), 라이프타임(`ch10-generics-traits-lifetimes`), `#[test]`와 `cargo test`(`ch11-testing`).

### 2.2 명령줄 인자 받기: `std::env::args`

```rust
use std::env;

let args: Vec<String> = env::args().collect();
```

- `env::args()`는 프로그램이 받은 인자를 하나씩 꺼내 주는 **반복자(iterator)** 를 돌려줍니다. 반복자는 `ch13-closures-iterators`에서 자세히 다루고, 지금은 "값을 차례로 꺼내 주는 것"으로 보면 됩니다.
- `collect()`는 반복자가 내놓는 값을 모아 컬렉션 하나로 만듭니다. 벡터, 해시 맵 등 여러 컬렉션을 만들 수 있어서 무엇을 만들지 스스로 정하지 못하므로 `Vec<String>`이라고 타입을 적어 줍니다.
- **0번 자리는 실행 파일 경로**입니다. 사용자가 넘긴 인자는 1번부터 들어 있습니다. 그래서 인자 없이 실행해도 길이는 1입니다. 0번 값은 실행한 위치에 따라 달라집니다(3절 예제 01).
- 인자는 셸이 공백으로 나눠 넘깁니다. 공백이 든 값을 인자 하나로 넘기려면 따옴표로 감쌉니다. 한글 인자도 그대로 `String`이 됩니다.
- 인자 중 하나라도 유효한 유니코드가 아니면 `env::args()`는 패닉합니다. zsh나 bash에서 `$'\xff'`로 UTF-8이 아닌 바이트를 넘겨 보면 다음처럼 끝납니다(종료 코드 101). 패닉 위치가 표준 라이브러리 소스로 나오는 것은 패닉이 `env::args()` 안에서 났기 때문입니다. 그런 인자까지 받아야 하는 프로그램은 `env::args_os()`를 씁니다. 이 함수는 `String` 대신 다루기 조금 더 번거로운 `OsString`을 돌려줍니다.

  ```console
  $ cargo run -q -p p0-ch12-io-project-minigrep --example 01_args -- $'\xff'

  thread 'main' (6290064) panicked at /rustc/48a229ceaefd4985c50990b14116b6d856af0985/library/std/src/env.rs:878:51:
  called `Result::unwrap()` on an `Err` value: "\xFF"
  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
  ```

**`cargo run`으로 인자 넘기기와 `--`**: 이 저장소의 예제는 다음처럼 실행합니다.

```sh
cargo run -q -p p0-ch12-io-project-minigrep --example 01_args -- needle haystack
```

`--`는 "여기서부터는 cargo의 옵션이 아니라 실행할 프로그램에 넘길 인자"라는 표시입니다. `needle`처럼 `-`로 시작하지 않는 값은 `--` 없이 써도 프로그램에 전달되지만, `-i`처럼 옵션 모양의 값은 cargo가 자기 옵션으로 해석하려다 에러를 냅니다(4.4절). 그래서 프로그램에 줄 인자 앞에는 항상 `--`를 붙이는 습관을 들입니다. 빌드된 실행 파일을 직접 실행할 때(`./target/debug/minigrep to poem.txt`)는 cargo를 거치지 않으므로 `--`가 필요 없습니다.

### 2.3 파일 읽기: `fs::read_to_string`

```rust
use std::fs;

let contents = fs::read_to_string(file_path).expect("파일을 읽을 수 있어야 합니다");
```

`fs::read_to_string`은 파일을 열어 내용 전체를 `String`으로 읽어 오며, 결과 타입은 `std::io::Result<String>`(`Result<String, std::io::Error>`)입니다. 파일이 없거나, 읽을 권한이 없거나, 내용이 UTF-8이 아니면 `Err`가 됩니다(4.6절).

**상대 경로는 현재 디렉터리 기준입니다.** `poem.txt`라고 넘기면 소스 파일이나 실행 파일이 있는 곳이 아니라, 프로그램을 **실행한 디렉터리**에서 `poem.txt`를 찾습니다. 이 README의 예제 명령은 모두 저장소 루트에서 실행한다고 보고 `part0-getting-started/ch12-io-project-minigrep/poem.txt`처럼 루트 기준 경로를 넘깁니다. 다른 디렉터리에서 실행한다면 그 위치에서 본 경로나 절대 경로를 넘기면 됩니다. 예제 코드는 경로를 인자로만 받으므로 어느 디렉터리에서 실행해도 같은 방식으로 동작합니다.

예제 02는 인자를 `&args[1]`, `&args[2]`로 꺼내 파일을 읽는 첫 초안입니다. 정상적으로 쓰면 잘 동작하지만, 잘못 쓰면 패닉으로 끝납니다(3절 예제 02).

### 2.4 리팩터링 ①: 설정 값을 `Config`로 묶기

예제 02에는 문제가 네 가지 있습니다.

1. **`main`이 여러 일을 합니다.** 인자 해석과 파일 읽기를 모두 합니다. 기능이 늘수록 `main`이 길어져 읽기도, 테스트하기도 어려워집니다.
2. **설정 값과 로직 값이 섞여 있습니다.** `query`와 `file_path`는 "어떻게 실행할지"를 정하는 설정이고, `contents`는 로직이 다루는 데이터입니다. 이름만 보고는 이 구분이 드러나지 않습니다.
3. **파일 읽기 에러가 뭉뚱그려집니다.** `expect`의 메시지 하나가 파일 없음, 권한 없음 등 모든 경우에 똑같이 붙습니다.
4. **인자 에러가 사용자에게 불친절합니다.** 인자를 빠뜨리면 `index out of bounds: the len is 1 but the index is 1`이라는 패닉이 납니다. 프로그래머에게는 의미가 있어도 사용자는 무엇을 잘못했는지 알 수 없습니다.

코드가 아직 작을 때 고쳐 두는 편이 쉽습니다. 먼저 설정 값 두 개를 구조체 하나로 묶습니다.

```rust
struct Config {
    query: String,
    file_path: String,
}
```

**왜 `&str`이 아니라 `String`이고, 왜 `clone`하는가?** `args` 벡터가 인자 `String`들의 소유자(owner)입니다. `Config`를 만드는 함수는 `args`를 `&[String]`으로 빌려 오기만 하므로, 그 안의 `String`을 꺼내 `Config`로 옮길(move) 수 없습니다. 옮기면 원래 소유자인 벡터에 빈자리가 생기기 때문입니다. 이렇게 쓰면 컴파일러가 E0508로 거부합니다(4.2절).

```rust
Config { query: args[1], file_path: args[2] } // E0508: cannot move out of type `[String]`
```

방법은 두 가지입니다. `Config`의 필드를 `&str` 참조로 만들면 복사는 없지만, `Config`가 `args`보다 오래 살 수 없다는 라이프타임 관계를 구조체에 적어야 합니다. `args[1].clone()`으로 복제하면 메모리와 시간이 조금 더 들지만 `Config`가 자기 값을 소유하므로 라이프타임을 신경 쓸 필요가 없습니다. 인자 두 개를 실행할 때 한 번 복사하는 비용은 무시할 만하므로, 이 프로젝트는 단순한 쪽인 `clone`을 택합니다. 처음부터 완벽하게 최적화하려 하기보다 먼저 동작하는 코드를 만드는 것이 낫습니다. 복제 없이 처리하는 방법은 `ch13-closures-iterators`에서 반복자로 다시 다룹니다.

### 2.5 리팩터링 ②: 패닉 대신 `Result`로 알리기

`Config`를 만드는 연관 함수에서 인자 개수를 먼저 확인합니다.

```rust
impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("인자가 부족합니다");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        Ok(Config { query, file_path })
    }
}
```

- **왜 `panic!`이 아니라 `Result`인가?** 패닉은 "프로그램에 버그가 있다"는 신호입니다. 인자를 빠뜨리는 것은 버그가 아니라 사용자가 충분히 저지를 수 있는 실수이므로, 실패를 값(`Err`)으로 돌려주고 어떻게 알릴지는 호출한 쪽이 정하게 합니다(`ch9-error-handling`의 "패닉과 `Result`의 선택 기준").
- **왜 `new`가 아니라 `build`인가?** Rust에서는 `new`라는 이름의 생성자는 실패하지 않는다고 기대하는 관례가 있습니다. 실패할 수 있으므로 다른 이름을 붙입니다.
- **에러 타입 `&'static str`**: 문자열 리터럴의 타입입니다. 리터럴은 프로그램 바이너리에 들어 있어 프로그램이 끝날 때까지 유효하므로 라이프타임이 `'static`입니다.

`main`은 `match`로 결과를 나눕니다.

```rust
let config = match Config::build(&args) {
    Ok(config) => config,
    Err(err) => {
        println!("인자 문제: {err}");
        println!("사용법: minigrep <검색어> <파일 경로>");
        process::exit(1);
    }
};
```

`std::process::exit`는 프로그램을 그 자리에서 끝내고, 인자로 준 숫자를 **종료 코드(exit code)** 로 운영체제에 돌려줍니다. 종료 코드는 셸이나 이 프로그램을 실행한 다른 프로그램이 성공 여부를 판단하는 수단입니다. 셸에서는 `echo $?`로 직전 명령의 종료 코드를 볼 수 있습니다.

| 종료 코드 | 뜻 | 이 챕터에서 나오는 곳 |
| --- | --- | --- |
| 0 | 성공 | `main`이 정상적으로 끝남 |
| 1 | 실패 (0이 아닌 값은 모두 실패라는 관례) | `process::exit(1)` |
| 101 | 패닉 | 예제 02의 인덱스 패닉, `expect` 실패 |

`cargo run`은 실행한 프로그램의 종료 코드를 그대로 돌려주므로 `cargo run ...; echo $?`로 확인할 수 있습니다.

`process::exit`의 반환 타입은 "절대 돌아오지 않음"을 뜻하는 `!`입니다. 그래서 `Err` 갈래가 `Config` 값을 만들지 않아도 `match` 전체의 타입이 `Config`로 맞습니다. ch6에서 `let...else`의 `else`가 흐름을 끊어야 한다고 했던 것과 같은 원리입니다. 단, `process::exit`는 즉시 끝나므로 아직 살아 있는 값이 drop되지 않습니다. 메모리는 프로세스가 끝나면 운영체제가 회수하므로 지금처럼 `main`에서 끝낼 때는 문제가 없지만, drop될 때 마무리 작업을 해야 하는 값이 살아 있다면 주의해야 합니다.

> **The Book의 `unwrap_or_else`**: The Book은 같은 일을 `Config::build(&args).unwrap_or_else(|err| { ... })`로 씁니다. `|err| { ... }`는 이름 없는 함수인 **클로저(closure)** 로, `unwrap_or_else`는 값이 `Ok`면 안의 값을 꺼내고 `Err`일 때만 클로저에 에러 값을 넘겨 실행합니다. 위의 `match`와 같은 동작입니다. 클로저는 `ch13-closures-iterators`에서 배우며, 거기서 이 프로젝트를 반복자와 클로저로 개선합니다.

**`main`이 `Result`를 돌려주면 되지 않을까?** `ch9-error-handling`에서처럼 `fn main() -> Result<(), &'static str>`로 쓰고 `?`를 써도 종료 코드 1로 끝나기는 합니다. 하지만 에러가 디버그 형식으로 `Error: "인자가 부족합니다"`처럼 따옴표째 출력되고, 사용법 같은 안내를 덧붙일 수 없습니다. 사용자에게 보여 줄 메시지를 직접 정하고 싶어서 `match`와 `process::exit`를 씁니다.

### 2.6 리팩터링 ③: 로직을 `run`으로 빼기

`main`에 남은 파일 읽기와 출력을 `run` 함수로 옮깁니다. 파일 읽기 실패도 `expect`로 패닉을 내는 대신 `?`로 호출한 쪽에 돌려줍니다.

```rust
use std::error::Error;

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;
    println!("파일 내용:\n{contents}");
    Ok(())
}
```

- 성공했을 때 돌려줄 값이 없으므로 `Ok` 쪽은 유닛 타입 `()`이고, 마지막에 `Ok(())`를 돌려줍니다. "부수 효과(출력)를 위해 불렀고 돌려줄 값은 없다"는 뜻입니다.
- `Box<dyn Error>`는 "`Error` 트레이트를 구현한 타입이면 무엇이든"이라는 뜻의 에러 타입입니다. `?`는 `io::Error`를 이 타입으로 바꿔서 돌려줍니다. 에러 타입을 `&'static str`로 정하면 `io::Error`를 `&str`로 바꿀 방법이 없어 E0277이 납니다(4.3절). `dyn`과 트레이트 객체는 `ch17-trait-objects-oop`에서 자세히 다룹니다.

`main`에서는 `if let`으로 `Err`인 경우만 골라냅니다. 성공하면 꺼낼 값이 `()`뿐이므로 `match`로 두 갈래를 다 쓸 필요가 없습니다.

```rust
if let Err(e) = run(config) {
    println!("실행 오류: {e}");
    process::exit(1);
}
```

`Box<dyn Error>`를 `{}`로 출력하면 안에 든 원래 에러의 메시지가 나옵니다. 파일이 없으면 `실행 오류: No such file or directory (os error 2)`가 출력됩니다. `expect`의 패닉 메시지(`Os { code: 2, kind: NotFound, ... }`)보다 사용자가 읽기 좋습니다.

`run(config);`처럼 결과를 무시하면 컴파일러가 경고합니다. `Result`에는 "반드시 사용해야 하는 값"이라는 표시가 붙어 있어서, 에러를 그냥 흘려보내는 실수를 알려 줍니다. 다음은 예제 04의 `if let` 블록을 `run(config);` 한 줄로 바꿨을 때의 경고입니다.

```text
warning: unused `Result` that must be used
  --> examples/04_run.rs:66:5
   |
66 |     run(config);
   |     ^^^^^^^^^^^
   |
   = note: this `Result` may be an `Err` variant, which should be handled
   = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
help: use `let _ = ...` to ignore the resulting value
   |
66 |     let _ = run(config);
   |     +++++++
```

`help:`가 제안하는 `let _ =`는 "일부러 무시한다"는 표시일 뿐 에러를 처리하지 않습니다. 여기서는 `if let`으로 처리하는 것이 맞습니다.

### 2.7 관심사 분리: `main.rs`는 얇게, 로직은 `lib.rs`로

`main` 하나에 모든 일을 맡기는 문제는 바이너리 프로젝트에서 흔합니다. Rust 커뮤니티는 이런 **관심사 분리(separation of concerns)** 지침을 따릅니다.

- 프로그램을 `src/main.rs`와 `src/lib.rs`로 나누고, 프로그램의 로직은 `src/lib.rs`에 둡니다.
- 인자 해석이 아주 단순하면 `main.rs`에 둬도 되지만, 복잡해지면 함수나 타입으로 빼냅니다.
- 그 결과 `main`에 남는 일은 다음뿐입니다.
  - 인자 값으로 인자 해석 로직을 부르기
  - 그 밖의 설정 준비하기
  - `lib.rs`의 `run` 함수 부르기
  - `run`이 에러를 돌려주면 처리하기

**왜 나누는가?**

- **테스트하기 쉬워집니다.** `main`은 인자를 받지도 값을 돌려주지도 않고, `env::args()`를 읽고 `process::exit`로 끝나 버리므로 테스트에서 부르기 어렵습니다. 반면 `Config::build(&args)`나 `search(query, contents)`처럼 값을 받아 값을 돌려주는 함수는 `#[test]`에서 바로 부르고 결과를 비교할 수 있습니다.
- **통합 테스트는 라이브러리 크레이트만 쓸 수 있습니다.** `ch11-testing`에서 본 `tests/` 디렉터리의 통합 테스트는 `use`로 라이브러리 크레이트의 공개 항목을 가져와 씁니다. 바이너리 크레이트(`src/main.rs`)의 함수는 가져올 수 없습니다. 중요한 기능을 라이브러리에 두면 통합 테스트로 검사할 수 있고, 남은 `main.rs`는 눈으로 확인할 수 있을 만큼 작아집니다.
- **다시 쓸 수 있습니다.** 라이브러리에 둔 `search`는 다른 프로그램에서도 가져다 쓸 수 있습니다.

`projects/minigrep`은 이렇게 나눕니다.

```text
projects/minigrep/
├── Cargo.toml
├── Cargo.lock
├── poem.txt       영어 검색 대상 (The Book과 같은 Emily Dickinson의 시)
├── poem_ko.txt    한글 검색 대상 (윤동주, 「서시」)
└── src/
    ├── main.rs    바이너리 크레이트: 인자 모으기, 에러 알리기, 종료 코드
    └── lib.rs     라이브러리 크레이트: Config, run, search, search_case_insensitive, 테스트
```

`src/main.rs` 전체는 다음과 같습니다(주석은 뺐습니다).

```rust
use std::env;
use std::process;

use minigrep::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = match Config::build(&args) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("인자 문제: {err}");
            eprintln!("사용법: minigrep <검색어> <파일 경로>");
            eprintln!("(대소문자를 무시하려면 환경 변수 IGNORE_CASE를 설정합니다)");
            process::exit(1);
        }
    };

    if let Err(e) = minigrep::run(config) {
        eprintln!("실행 오류: {e}");
        process::exit(1);
    }
}
```

[ch7](../ch7-modules/)에서 본 것처럼 바이너리 크레이트는 같은 패키지의 라이브러리 크레이트를 **패키지 이름**(`minigrep::`)으로 가리키고, 라이브러리의 공개(`pub`) 항목만 쓸 수 있습니다. 그래서 `lib.rs`의 `Config`와 그 필드, `build`, `run`, 두 검색 함수에 `pub`을 붙였습니다. `eprintln!`은 2.10절에서 설명합니다.

> **The Book과 다른 점**: 현재 The Book(12.3절)의 코드는 `search` 함수만 `lib.rs`로 옮기고 `Config`와 `run`은 `main.rs`에 남겨 둡니다. 이 저장소는 위 지침대로 `Config`와 `run`까지 `lib.rs`로 옮겼습니다. 그 덕분에 `Config::build`의 인자 검사도 라이브러리 테스트로 확인할 수 있습니다(3절의 `build_needs_query_and_file_path`). 어느 쪽이든 `main.rs`가 로직 없이 얇다는 점은 같습니다.

### 2.8 테스트 주도 개발로 `search` 만들기

**테스트 주도 개발(TDD, test-driven development)** 은 다음 순서를 되풀이하는 방법입니다.

1. 실패하는 테스트를 쓰고, 실행해서 **예상한 이유로** 실패하는지 확인합니다.
2. 그 테스트를 통과할 만큼만 코드를 쓰거나 고칩니다.
3. 방금 쓴 코드를 리팩터링하고, 테스트가 계속 통과하는지 확인합니다.

테스트를 먼저 쓰면 "무엇이 되면 완성인가"가 코드로 정해지고, 리팩터링할 때 기능이 망가지지 않았다는 확인을 테스트가 대신해 줍니다. 아래 출력은 `projects/minigrep`을 저장소 밖에 복사해 각 단계를 재현한 실제 결과입니다. 줄 번호는 그 단계의 `lib.rs` 내용에 따라 다르므로 완성판과는 다릅니다.

**① 실패하는 테스트 쓰기**

`lib.rs` 끝에 테스트 모듈을 추가합니다. 세 줄 중 `duct`가 들어 있는 한 줄만 돌려받아야 합니다.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }
}
```

여는 따옴표 뒤의 `\`는 문자열 맨 앞에 개행이 들어가지 않게 합니다. 테스트가 컴파일되려면 `search`가 있어야 하므로, 항상 빈 벡터를 돌려주는 빈 껍데기를 먼저 만듭니다. 이때 라이프타임을 빼고 쓰면 컴파일러가 거부합니다.

```rust
pub fn search(query: &str, contents: &str) -> Vec<&str> {
    vec![]
}
```

```text
error[E0106]: missing lifetime specifier
  --> src/lib.rs:32:51
   |
32 | pub fn search(query: &str, contents: &str) -> Vec<&str> {
   |                      ----            ----         ^ expected named lifetime parameter
   |
   = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `query` or `contents`
help: consider introducing a named lifetime parameter
   |
32 | pub fn search<'a>(query: &'a str, contents: &'a str) -> Vec<&'a str> {
   |              ++++         ++                 ++              ++

For more information about this error, try `rustc --explain E0106`.
error: could not compile `minigrep` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `minigrep` (lib test) due to 1 previous error
```

**왜 라이프타임이 필요한가?** `search`가 돌려주는 `Vec<&str>` 안의 `&str`은 새로 만든 문자열이 아니라 `contents`의 일부를 가리키는 슬라이스입니다. 반환값이 참조이면 컴파일러는 "그 참조가 어느 매개변수에서 빌려 온 것인지"를 알아야, 호출한 쪽에서 결과를 쓰는 동안 원본이 살아 있는지 검사할 수 있습니다. 참조 매개변수가 하나뿐이면 라이프타임 생략 규칙이 알아서 이어 주지만, `query`와 `contents` 두 개라서 정할 수 없습니다(`ch10-generics-traits-lifetimes`의 `longest`와 같은 상황). 그래서 다음처럼 **`contents`와 반환값만** 같은 라이프타임 `'a`로 묶습니다.

```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    vec![]
}
```

이 시그니처는 "반환된 슬라이스들은 `contents`가 살아 있는 동안 유효하다"는 약속입니다. 컴파일러의 `help:`처럼 `query`에도 `'a`를 붙여도 컴파일은 되지만, 반환값이 `query`와는 상관없는데도 `query`까지 결과만큼 오래 살아야 한다는 과한 제약이 생깁니다. 예를 들어 블록 안에서 만든 검색어로 검색하고 블록 밖에서 결과를 쓰면, `query`에 `'a`를 붙인 버전은 ``error[E0597]: `query` does not live long enough``로 거부되고 `contents`에만 붙인 버전은 통과합니다.

라이프타임을 고치면 컴파일은 되고, 테스트는 **예상한 이유로** 실패합니다. 기대한 값은 한 줄짜리 벡터인데 빈 벡터가 돌아왔습니다.

```console
$ cargo test
```

```text
running 1 test
test tests::one_result ... FAILED

failures:

---- tests::one_result stdout ----

thread 'tests::one_result' (6281652) panicked at src/lib.rs:48:9:
assertion `left == right` failed
  left: ["safe, fast, productive."]
 right: []
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::one_result

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

이 출력 앞에는 `query`와 `contents`를 쓰지 않았다는 `warning: unused variable` 경고 두 개가 먼저 나옵니다. 아직 빈 껍데기이므로 예상한 경고입니다. 괄호 안의 숫자는 스레드 ID로, 실행할 때마다 달라집니다.

**② 통과할 만큼 구현하기**

```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }

    results
}
```

- `lines()`는 문자열을 줄 단위로 나눠 하나씩 꺼내 줍니다. 각 줄은 `contents`를 가리키는 `&str`이고, 줄 끝의 개행 문자는 들어 있지 않습니다.
- `contains`는 그 줄에 검색어가 들어 있는지 `bool`로 알려 줍니다.
- 결과 벡터에 넣는 `line`이 `contents`의 일부이므로 반환 타입 `Vec<&'a str>`과 맞습니다.

```text
running 1 test
test tests::one_result ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**③ 리팩터링**: 지금 코드는 충분히 짧아서 더 고칠 것이 없습니다. 이 반복문은 `ch13-closures-iterators`에서 반복자 어댑터로 더 짧게 바꿉니다. 이제 `run`이 `search`를 써서 찾은 줄만 출력하게 합니다.

```rust
for line in search(&config.query, &contents) {
    println!("{line}");
}
```

### 2.9 환경 변수로 대소문자 무시하기

대소문자를 무시하고 검색하는 기능을 추가합니다. 이 설정은 명령줄 인자 대신 **환경 변수** `IGNORE_CASE`로 켭니다. 환경 변수는 셸 세션에 한 번 설정해 두면 그 뒤에 실행하는 프로그램들이 모두 읽을 수 있으므로, 매번 인자로 넘기기 번거로운 "사용자 취향" 같은 설정에 알맞습니다.

**다시 TDD로 시작합니다.** 기존 테스트 이름을 `case_sensitive`로 바꾸고 `Duct tape.` 줄을 추가해 대문자 `D`는 찾지 않는다는 것도 확인하게 합니다. 그리고 새 테스트 `case_insensitive`를 씁니다. 이 테스트는 검색어 `rUsT`로 `Rust:`와 `Trust me.` 두 줄을 찾아야 합니다.

```rust
#[test]
fn case_insensitive() {
    let query = "rUsT";
    let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

    assert_eq!(
        vec!["Rust:", "Trust me."],
        search_case_insensitive(query, contents)
    );
}
```

아직 함수가 없으므로 `cargo test`는 컴파일 단계에서 실패합니다. 테스트 모듈은 `#[cfg(test)]`라서 테스트 빌드에서만 컴파일되므로, 에러 끝에 `(lib test)`라고 나옵니다.

```text
error[E0425]: cannot find function `search_case_insensitive` in this scope
  --> src/lib.rs:71:13
   |
71 |             search_case_insensitive(query, contents)
   |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `minigrep` (lib test) due to 1 previous error
```

빈 벡터를 돌려주는 껍데기를 만들면 `case_sensitive`는 통과하고 `case_insensitive`만 예상대로 실패합니다(출력의 앞부분만 옮겼습니다).

```text
running 2 tests
test tests::case_sensitive ... ok
test tests::case_insensitive ... FAILED

failures:

---- tests::case_insensitive stdout ----

thread 'tests::case_insensitive' (6283555) panicked at src/lib.rs:73:9:
assertion `left == right` failed
  left: ["Rust:", "Trust me."]
 right: []
```

통과할 만큼의 첫 구현은 줄과 검색어를 모두 소문자로 바꿔 비교하는 것입니다.

```rust
for line in contents.lines() {
    if line.to_lowercase().contains(&query.to_lowercase()) {
        results.push(line);
    }
}
```

```text
running 2 tests
test tests::case_insensitive ... ok
test tests::case_sensitive ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

테스트는 여러 스레드에서 동시에 실행되므로 `test ... ok` 줄의 순서는 실행할 때마다 바뀔 수 있습니다.

이제 리팩터링할 차례입니다. 이 코드는 **줄마다 검색어를 다시 소문자로 바꿉니다.** 검색어는 바뀌지 않으므로 반복문 밖에서 한 번만 바꾸면 됩니다. 테스트가 있으니 고친 뒤 `cargo test`만 다시 돌려 보면 동작이 그대로인지 확인됩니다(결과는 위와 같이 `2 passed`).

```rust
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }

    results
}
```

- `to_lowercase()`는 원래 문자열을 고치지 않고 소문자로 바꾼 **새 `String`** 을 만듭니다. 그래서 같은 이름 `query`로 섀도잉한 뒤로 `query`의 타입은 `&str`이 아니라 `String`입니다.
- `contains`는 `&str` 같은 패턴을 받으므로 `String`인 `query`를 `&query`로 빌려 넘깁니다.
- 결과에는 소문자로 바꾼 줄이 아니라 원래 `line`을 넣습니다. 그래서 출력은 원문 그대로입니다.
- `to_lowercase()`는 영어 이외의 문자도 유니코드 규칙에 따라 바꾸지만, The Book의 말처럼 모든 언어에서 100% 정확하지는 않습니다. 한글에는 대소문자가 없으므로 결과가 바뀌지 않습니다.

**`Config`에 설정 추가하기**: `Config`에 `ignore_case: bool` 필드를 넣고, `build`에서 환경 변수를 읽습니다. `run`은 이 값에 따라 두 검색 함수 중 하나를 고릅니다.

```rust
let ignore_case = env::var("IGNORE_CASE").is_ok();
```

```rust
let results = if config.ignore_case {
    search_case_insensitive(&config.query, &contents)
} else {
    search(&config.query, &contents)
};
```

`env::var`는 환경 변수가 있으면 `Ok(값)`, 없으면 `Err`를 담은 `Result<String, VarError>`를 돌려줍니다. 여기서는 값이 무엇인지는 보지 않고 **설정돼 있는지만** 따지므로 `is_ok()`로 `bool`을 얻습니다.

**실행해 보기** (`projects/minigrep` 디렉터리에서):

```console
$ cargo run -q -- to poem.txt
Are you nobody, too?
How dreary to be somebody!
$ IGNORE_CASE=1 cargo run -q -- to poem.txt
Are you nobody, too?
How dreary to be somebody!
To tell your name the livelong day
To an admiring bog!
```

- `IGNORE_CASE=1 cargo run ...`처럼 명령 앞에 `이름=값`을 쓰면 **그 명령 하나에만** 환경 변수가 설정됩니다. 다음 명령에는 남지 않습니다.
- 셸 세션 전체에 설정하려면 `export IGNORE_CASE=1`, 해제하려면 `unset IGNORE_CASE`를 씁니다.
- Windows PowerShell에서는 `$Env:IGNORE_CASE=1; cargo run -- to poem.txt`로 설정하고 `Remove-Item Env:IGNORE_CASE`로 해제한다고 The Book이 안내합니다. 이 설정은 셸 세션이 끝날 때까지 유지됩니다(이 README는 macOS의 zsh에서만 확인했습니다).
- `is_ok()`는 값을 보지 않으므로 `IGNORE_CASE=0`이나 빈 값(`IGNORE_CASE=`)으로 설정해도 대소문자를 무시합니다. 직접 실행해 보면 두 경우 모두 위와 같은 네 줄이 나옵니다. 값까지 해석하는 방법은 연습 문제 3에서 다룹니다.

### 2.10 에러 메시지는 표준 에러로

[ch1](../ch1-hello-cargo/)에서 본 것처럼 프로그램의 출력에는 두 갈래가 있습니다. **표준 출력(stdout)** 은 프로그램의 결과를, **표준 에러(stderr)** 는 에러와 진행 상황 메시지를 위한 곳입니다. 터미널에서는 둘 다 화면에 보이지만, 셸의 리다이렉션(redirection) `>`는 표준 출력만 파일로 보냅니다.

예제 04까지는 에러 메시지도 `println!`(표준 출력)으로 냈습니다. 그 상태에서 결과를 파일로 저장하려고 `>`를 쓰면 문제가 보입니다(저장소 루트에서 실행).

```console
$ cargo run -q -p p0-ch12-io-project-minigrep --example 04_run > output.txt
$ echo $?
1
$ cat output.txt
인자 문제: 인자가 부족합니다
사용법: minigrep <검색어> <파일 경로>
$ rm output.txt
```

화면에는 아무것도 나오지 않아서 사용자는 왜 실패했는지 모르고, 결과가 들어 있어야 할 파일에는 에러 메시지가 들어갔습니다. 그래서 `projects/minigrep`의 `main.rs`는 에러 메시지를 `eprintln!`으로 냅니다. `eprintln!`은 `println!`과 서식 문법이 같고 표준 에러로 출력합니다. 검색 결과는 프로그램의 진짜 출력이므로 `run`에서 그대로 `println!`을 씁니다.

```console
$ cargo run -q > output.txt
인자 문제: 인자가 부족합니다
사용법: minigrep <검색어> <파일 경로>
(대소문자를 무시하려면 환경 변수 IGNORE_CASE를 설정합니다)
$ cat output.txt
$ cargo run -q -- to poem.txt > output.txt
$ cat output.txt
Are you nobody, too?
How dreary to be somebody!
$ rm output.txt
```

(`projects/minigrep` 디렉터리에서 실행했습니다.) 이제 에러는 화면에 보이고 파일은 비어 있으며, 정상 실행의 결과만 파일에 들어갑니다. 반대로 표준 에러만 파일로 보내려면 `2>`를 씁니다. `-q`를 빼고 실행하면 cargo가 출력하는 `Finished`, `Running` 줄도 화면에 보이는데, cargo 역시 자기 메시지를 표준 에러로 내기 때문입니다.

리다이렉션 실습으로 만든 `output.txt`는 저장소에 남기지 않도록 확인한 뒤 `rm`으로 지웁니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_args.rs`](examples/01_args.rs) | `env::args().collect()`, 0번 인자, `--`로 인자 넘기기 | 1 |
| [`examples/02_read_file.rs`](examples/02_read_file.rs) | `fs::read_to_string`, 검사 없는 초안의 패닉 두 가지 | 1, 2 |
| [`examples/03_config.rs`](examples/03_config.rs) | `Config`와 `Config::build`, `Result`로 인자 부족 알리기, `process::exit`와 종료 코드 | 2 |
| [`examples/04_run.rs`](examples/04_run.rs) | `run`과 `Box<dyn Error>`, `?`, `if let Err`, 표준 출력에 낸 에러 메시지의 문제 | 2, 5 |
| [`projects/minigrep`](projects/minigrep) | `lib.rs`/`main.rs` 분리, 테스트, `search`, 환경 변수, `eprintln!` | 3, 4, 5 |
| [`compile_fail/`](compile_fail) | 이 프로젝트에서 실제로 만나는 컴파일 에러 | 2, 4 |
| [`poem.txt`](poem.txt) | 예제 02~04가 읽는 검색 대상 파일 | - |

예제 명령은 모두 **저장소 루트**에서 실행한다고 보고 썼습니다. 예제 01~04는 같은 프로그램의 단계별 모습이라서 앞부분이 비슷합니다. 무엇이 바뀌었는지는 `diff part0-getting-started/ch12-io-project-minigrep/examples/03_config.rs part0-getting-started/ch12-io-project-minigrep/examples/04_run.rs`처럼 비교해 보면 잘 보입니다.

### 01_args — 명령줄 인자 모으기

```console
$ cargo run -q -p p0-ch12-io-project-minigrep --example 01_args -- needle haystack
인자 개수: 3
args[0] = "target/debug/examples/01_args"
사용자가 넘긴 인자: ["needle", "haystack"]
$ cargo run -q -p p0-ch12-io-project-minigrep --example 01_args
인자 개수: 1
args[0] = "target/debug/examples/01_args"
사용자가 넘긴 인자: []
$ cargo run -q -p p0-ch12-io-project-minigrep --example 01_args -- "죽는 날까지" 하늘을 우러러
인자 개수: 4
args[0] = "target/debug/examples/01_args"
사용자가 넘긴 인자: ["죽는 날까지", "하늘을", "우러러"]
```

볼 점:

- 인자 없이 실행해도 길이는 1이고, `&args[1..]`은 빈 슬라이스가 되어 패닉이 나지 않습니다.
- `args[0]`은 실행 파일 경로입니다. cargo는 실행 파일이 현재 디렉터리 아래에 있으면 현재 디렉터리 기준 상대 경로로 실행하므로 저장소 루트에서는 `target/debug/examples/01_args`가 나옵니다. 챕터 디렉터리에서 `cargo run -q --example 01_args -- needle`로 실행하면 `"<저장소 경로>/target/debug/examples/01_args"`처럼 절대 경로가 나옵니다. 0번 값은 이처럼 실행 방법에 따라 달라지므로 프로그램의 동작을 여기에 기대지 않습니다.
- 따옴표로 감싼 `"죽는 날까지"`는 공백이 있어도 인자 하나입니다.

### 02_read_file — 파일 읽기 (검사 없는 초안)

```console
$ cargo run -q -p p0-ch12-io-project-minigrep --example 02_read_file -- the part0-getting-started/ch12-io-project-minigrep/poem.txt
검색어: the
파일: part0-getting-started/ch12-io-project-minigrep/poem.txt
파일 내용:
I'm nobody! Who are you?
Are you nobody, too?
Then there's a pair of us - don't tell!
They'd banish us, you know.

How dreary to be somebody!
How public, like a frog
To tell your name the livelong day
To an admiring bog!

```

마지막 빈 줄은 `poem.txt`가 개행으로 끝나는 데다 `println!`이 개행을 하나 더 붙이기 때문입니다. 검색어 `the`는 아직 쓰지 않습니다.

잘못 실행하면 두 가지 패닉이 납니다. 둘 다 종료 코드는 101입니다.

```console
$ cargo run -q -p p0-ch12-io-project-minigrep --example 02_read_file

thread 'main' (6268674) panicked at part0-getting-started/ch12-io-project-minigrep/examples/02_read_file.rs:19:22:
index out of bounds: the len is 1 but the index is 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
$ echo $?
101
$ cargo run -q -p p0-ch12-io-project-minigrep --example 02_read_file -- the poem.txt
검색어: the
파일: poem.txt

thread 'main' (6268716) panicked at part0-getting-started/ch12-io-project-minigrep/examples/02_read_file.rs:30:50:
파일을 읽을 수 있어야 합니다: Os { code: 2, kind: NotFound, message: "No such file or directory" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
$ echo $?
101
```

볼 점:

- 첫 번째는 인자가 없어서 `args[1]`이 범위를 벗어난 경우입니다. 사용자는 이 메시지만으로 무엇을 잘못했는지 알기 어렵습니다(2.4절의 문제 4).
- 두 번째는 저장소 루트에서 `poem.txt`라고만 넘긴 경우입니다. 파일은 챕터 디렉터리에 있지만 프로그램은 **현재 디렉터리**(저장소 루트)에서 찾았습니다. `expect`의 메시지 뒤에 디버그 형식의 `io::Error`가 붙습니다.
- 괄호 안의 스레드 ID는 실행할 때마다 달라집니다.

### 03_config — `Config`와 인자 에러 처리

정상 실행의 출력은 예제 02와 같습니다.

```sh
cargo run -q -p p0-ch12-io-project-minigrep --example 03_config -- the part0-getting-started/ch12-io-project-minigrep/poem.txt
```

인자를 빠뜨리면 이제 패닉 대신 사용법을 알리고 종료 코드 1로 끝납니다. 인자를 하나만 줘도 같습니다.

```console
$ cargo run -q -p p0-ch12-io-project-minigrep --example 03_config
인자 문제: 인자가 부족합니다
사용법: minigrep <검색어> <파일 경로>
$ echo $?
1
$ cargo run -q -p p0-ch12-io-project-minigrep --example 03_config -- the
인자 문제: 인자가 부족합니다
사용법: minigrep <검색어> <파일 경로>
```

볼 점:

- 인자 검사가 `Config::build` 안으로 들어가고, `main`은 결과에 따라 알리기만 합니다.
- 종료 코드가 101(패닉)이 아니라 1입니다. 프로그램의 버그가 아니라 사용자의 실수로 실패했다는 것을 구분해서 알립니다.
- 파일 읽기는 아직 `expect`이므로 없는 파일을 주면 예제 02와 같은 패닉이 납니다.

### 04_run — `run`과 파일 에러 처리

정상 실행의 출력은 예제 02와 같습니다. 없는 파일을 주면 패닉 대신 에러 메시지를 출력하고 종료 코드 1로 끝납니다.

```console
$ cargo run -q -p p0-ch12-io-project-minigrep --example 04_run -- the no_such_file.txt
검색어: the
파일: no_such_file.txt
실행 오류: No such file or directory (os error 2)
$ echo $?
1
```

볼 점:

- `실행 오류:` 뒤는 `Box<dyn Error>` 안에 든 `io::Error`의 메시지입니다.
- 에러 메시지를 아직 `println!`으로 내므로, `> output.txt`로 실행하면 메시지가 화면 대신 파일로 들어갑니다(2.10절).

### projects/minigrep — 완성판

워크스페이스와 분리된 독립 프로젝트이므로 그 디렉터리에서 실행합니다. 검색 대상 파일 `poem.txt`, `poem_ko.txt`가 프로젝트 디렉터리에 있으므로 파일 이름만 넘기면 됩니다.

```sh
cd part0-getting-started/ch12-io-project-minigrep/projects/minigrep
```

**테스트**: 2.8~2.9절에서 만든 테스트 두 개에, 한글 검색 테스트(`hangul`)와 `Config::build`의 인자 검사 테스트 두 개를 더해 모두 다섯 개입니다. 테스트는 여러 스레드에서 동시에 실행되므로 `test ... ok` 줄의 순서는 실행할 때마다 달라질 수 있습니다.

```console
$ cargo test
```

```text
     Running unittests src/lib.rs (target/debug/deps/minigrep-d4a0709254e5231c)

running 5 tests
test tests::build_needs_query_and_file_path ... ok
test tests::case_insensitive ... ok
test tests::build_reads_query_and_file_path ... ok
test tests::case_sensitive ... ok
test tests::hangul ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/minigrep-bafc4b0da472ca4e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests minigrep

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(맨 앞의 `Compiling`, `Finished` 줄은 뺐습니다.) `src/main.rs`에는 테스트가 없으므로 `0 tests`입니다. 검사할 로직이 모두 라이브러리에 있기 때문입니다.

**검색**: 한 줄, 여러 줄, 없는 경우입니다. 찾은 줄이 없으면 아무것도 출력하지 않습니다.

```console
$ cargo run -q -- frog poem.txt
How public, like a frog
$ cargo run -q -- body poem.txt
I'm nobody! Who are you?
Are you nobody, too?
How dreary to be somebody!
$ cargo run -q -- monomorphization poem.txt
$ echo $?
0
```

**한글 검색**: 한글도 UTF-8 문자열이므로 같은 방식으로 찾습니다. 한글에는 대소문자가 없으므로 `IGNORE_CASE`를 설정해도 결과가 같습니다.

```console
$ cargo run -q -- 바람 poem_ko.txt
잎새에 이는 바람에도
오늘 밤에도 별이 바람에 스치운다.
$ cargo run -q -- 별 poem_ko.txt
별을 노래하는 마음으로
오늘 밤에도 별이 바람에 스치운다.
$ IGNORE_CASE=1 cargo run -q -- 바람 poem_ko.txt
잎새에 이는 바람에도
오늘 밤에도 별이 바람에 스치운다.
```

**에러와 종료 코드**: 에러 메시지는 표준 에러로 나갑니다.

```console
$ cargo run -q
인자 문제: 인자가 부족합니다
사용법: minigrep <검색어> <파일 경로>
(대소문자를 무시하려면 환경 변수 IGNORE_CASE를 설정합니다)
$ echo $?
1
$ cargo run -q -- to nofile.txt
실행 오류: No such file or directory (os error 2)
$ echo $?
1
```

**실행 파일을 직접 실행하기**: `cargo build`가 만든 실행 파일은 cargo를 거치지 않으므로 `--` 없이 인자를 넘깁니다.

```console
$ cargo build -q
$ ./target/debug/minigrep frog poem.txt
How public, like a frog
$ IGNORE_CASE=1 ./target/debug/minigrep to poem.txt
Are you nobody, too?
How dreary to be somebody!
To tell your name the livelong day
To an admiring bog!
```

대소문자 무시(`IGNORE_CASE`)와 리다이렉션(`> output.txt`)의 실행 결과는 2.9절과 2.10절에 실었습니다. 실습이 끝나면 `cargo clean`으로 `target/`을 지울 수 있습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 코드를 고쳐 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part0-getting-started/ch12-io-project-minigrep
rustc --edition 2024 compile_fail/e0106_search_missing_lifetime.rs --out-dir /tmp/rust-ch12
```

### 4.1 `search`의 라이프타임을 빠뜨림 — E0106

파일: [`compile_fail/e0106_search_missing_lifetime.rs`](compile_fail/e0106_search_missing_lifetime.rs)

```text
error[E0106]: missing lifetime specifier
  --> compile_fail/e0106_search_missing_lifetime.rs:14:51
   |
14 | pub fn search(query: &str, contents: &str) -> Vec<&str> {
   |                      ----            ----         ^ expected named lifetime parameter
   |
   = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `query` or `contents`
help: consider introducing a named lifetime parameter
   |
14 | pub fn search<'a>(query: &'a str, contents: &'a str) -> Vec<&'a str> {
   |              ++++         ++                 ++              ++
```

- **읽는 법**: `^`가 반환 타입 안의 `&`를 가리키며 "이름 붙은 라이프타임 매개변수가 필요하다"고 합니다. `----` 두 개는 후보인 참조 매개변수 `query`와 `contents`입니다. `= help:`는 "반환값이 빌린 값을 담고 있는데, `query`와 `contents` 중 어디서 빌렸는지 시그니처에 없다"는 뜻입니다.
- **원인**: 참조 매개변수가 두 개라서 라이프타임 생략 규칙으로 반환값의 라이프타임을 정할 수 없습니다.
- **해결**: 반환값이 실제로 빌려 오는 곳은 `contents`이므로 `pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str>`로 씁니다. 컴파일러가 제안한 대로 `query`에도 `'a`를 붙이면 컴파일은 되지만 필요 없는 제약이 생깁니다(2.8절).

### 4.2 `args[1]`을 `Config`로 옮기려 함 — E0508

파일: [`compile_fail/e0508_move_out_of_args.rs`](compile_fail/e0508_move_out_of_args.rs)

```text
error[E0508]: cannot move out of type `[String]`, a non-copy slice
  --> compile_fail/e0508_move_out_of_args.rs:26:20
   |
26 |             query: args[1],
   |                    ^^^^^^^
   |                    |
   |                    cannot move out of here
   |                    move occurs because `args[_]` has type `String`, which does not implement the `Copy` trait
   |
help: consider cloning the value if the performance cost is acceptable
   |
26 |             query: args[1].clone(),
   |                           ++++++++
```

같은 에러가 `file_path: args[2]` 줄에도 한 번 더 나옵니다.

- **읽는 법**: "복사할 수 없는 슬라이스 `[String]`에서 값을 옮겨 갈 수 없다"는 뜻입니다. `String`은 `Copy` 트레이트를 구현하지 않으므로 `args[1]`을 다른 곳에 넣는 것은 복사가 아니라 이동입니다.
- **원인**: `build`는 `args`를 `&[String]`으로 빌려 왔을 뿐 소유하지 않습니다. 빌려 온 것에서 소유권을 가져갈 수는 없습니다. 만약 허용된다면 `main`의 벡터에 빈자리가 생겨, 나중에 그 자리를 읽거나 벡터를 drop할 때 문제가 됩니다.
- **해결**: 컴파일러의 `help:`대로 `args[1].clone()`으로 복제본을 넣습니다(2.4절). 같은 실수를 슬라이스가 아니라 벡터에 하면, 예를 들어 `main`에서 `let query = args[1];`이라고 쓰면 에러 코드만 다른 ``error[E0507]: cannot move out of index of `Vec<String>` ``이 납니다. 그때는 `help:`가 `&args[1]`(빌려 오기)도 함께 제안합니다.

### 4.3 `run`의 에러 타입을 `&'static str`로 정함 — E0277

파일: [`compile_fail/e0277_question_mark_error_type.rs`](compile_fail/e0277_question_mark_error_type.rs)

```text
error[E0277]: `?` couldn't convert the error to `&str`
  --> compile_fail/e0277_question_mark_error_type.rs:22:56
   |
21 | fn run(config: Config) -> Result<(), &'static str> {
   |                           ------------------------ expected `&str` because of this
22 |     let contents = fs::read_to_string(config.file_path)?;
   |                    ------------------------------------^ the trait `From<std::io::Error>` is not implemented for `&str`
   |                    |
   |                    this can't be annotated with `?` because it has type `Result<_, std::io::Error>`
   |
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
```

(이 뒤에 `String`이 구현한 `From` 목록을 보여 주는 `= help:`가 이어지지만 이 상황과는 관계가 없습니다.)

- **읽는 법**: `?`가 `io::Error`를 함수의 에러 타입 `&str`로 바꾸지 못했다는 뜻입니다. `note:`가 설명하듯 `?`는 에러를 돌려주기 전에 `From` 트레이트로 함수의 에러 타입으로 변환하는데, `io::Error`에서 `&str`로 가는 변환은 정의돼 있지 않습니다.
- **원인**: `Config::build`와 모양을 맞추려고 `run`의 에러 타입도 `&'static str`로 정했지만, `run` 안에서 실패할 수 있는 작업(`fs::read_to_string`)의 에러 타입은 `io::Error`입니다.
- **해결**: `run`의 반환 타입을 `Result<(), Box<dyn Error>>`로 바꿉니다. `Error` 트레이트를 구현한 에러는 무엇이든 `Box<dyn Error>`로 변환될 수 있습니다(2.6절).

### 4.4 `--` 없이 옵션 모양의 인자를 넘김

프로그램에 `-i` 같은 인자를 넘기려고 `--`를 빠뜨리면 cargo가 그것을 자기 옵션으로 해석합니다.

```console
$ cargo run -q -p p0-ch12-io-project-minigrep --example 01_args -i needle
error: unexpected argument '-i' found

  tip: to pass '-i' as a value, use '-- -i'

Usage: cargo run [OPTIONS] [ARGS]...

For more information, try '--help'.
$ cargo run -q -p p0-ch12-io-project-minigrep --example 01_args -- -i needle
인자 개수: 3
args[0] = "target/debug/examples/01_args"
사용자가 넘긴 인자: ["-i", "needle"]
```

- **원인**: `cargo run` 뒤에 쓴 것 중 `-`로 시작하는 것은 cargo의 옵션으로 해석됩니다. `-i`라는 cargo 옵션은 없으므로 에러가 납니다(종료 코드 1). 이 에러는 프로그램이 실행되기도 전에 cargo가 낸 것입니다.
- **해결**: cargo의 `tip:`대로 프로그램에 줄 인자 앞에 `--`를 붙입니다.

### 4.5 상대 경로를 엉뚱한 디렉터리 기준으로 넘김

저장소 루트에서 `-- the poem.txt`처럼 파일 이름만 넘기면, 파일이 챕터 디렉터리에 있어도 찾지 못합니다. 예제 02는 `expect` 때문에 패닉하고(3절), 예제 04와 `projects/minigrep`은 `실행 오류: No such file or directory (os error 2)`를 출력하고 종료 코드 1로 끝납니다.

- **원인**: 상대 경로는 소스 파일이나 실행 파일의 위치가 아니라 **프로그램을 실행한 현재 디렉터리**를 기준으로 해석됩니다.
- **해결**: 현재 디렉터리에서 본 경로(`part0-getting-started/ch12-io-project-minigrep/poem.txt`)나 절대 경로를 넘기거나, 파일이 있는 디렉터리로 이동해서 실행합니다. `pwd`로 현재 디렉터리를 확인해 보세요.

### 4.6 UTF-8이 아닌 한글 파일을 읽음

오래된 한글 텍스트 파일은 EUC-KR(CP949)로 저장된 경우가 있습니다. `poem_ko.txt`를 `iconv -f UTF-8 -t EUC-KR poem_ko.txt > poem_euckr.txt`로 변환해 검색해 보면 다음처럼 실패합니다.

```console
$ cargo run -q -- 바람 poem_euckr.txt
실행 오류: stream did not contain valid UTF-8
```

- **원인**: Rust의 `String`은 항상 유효한 UTF-8이어야 합니다(ch8). `fs::read_to_string`은 내용이 UTF-8이 아니면 `String`을 만들지 않고 `Err`를 돌려줍니다. `run`이 이 에러를 `?`로 돌려준 덕분에 패닉 없이 메시지로 알 수 있습니다.
- **해결**: 파일을 UTF-8로 변환해서(`iconv -f EUC-KR -t UTF-8 ...`) 검색합니다. 실습 뒤에는 만든 `poem_euckr.txt`를 지웁니다.

## 5. 연습 문제

연습은 `projects/minigrep`을 저장소 밖으로 복사한 뒤 그 안에서 합니다(`cp -R part0-getting-started/ch12-io-project-minigrep/projects/minigrep /tmp/minigrep-try`). 기능을 더할 때는 이 챕터에서 한 것처럼 **테스트를 먼저** 써 보세요.

### 연습 1. 줄 번호와 함께 출력하기

찾은 줄 앞에 줄 번호를 붙여 출력하세요. `cargo run -q -- frog poem.txt`의 결과가 `7: How public, like a frog`가 되어야 합니다(빈 줄도 한 줄로 셉니다). 먼저 `search`처럼 줄 번호와 줄을 함께 돌려주는 함수의 테스트를 쓰고 시작합니다.

<details>
<summary>힌트</summary>

- 반환 타입은 튜플의 벡터 `Vec<(usize, &'a str)>`로 할 수 있습니다. 줄은 여전히 `contents`의 일부이므로 라이프타임 `'a`가 그대로 필요합니다.
- 테스트는 `assert_eq!(vec![(2, "safe, fast, productive.")], search_with_numbers("duct", contents));`처럼 씁니다.
- 반복문 밖에 `let mut number = 0;`을 두고, 줄마다 1씩 늘린 뒤 일치하면 `(number, line)`을 넣습니다. `ch13-closures-iterators`에서 배우는 `enumerate`를 쓰면 번호를 직접 세지 않아도 됩니다.
- 출력은 `run`에서 `println!("{number}: {line}")`처럼 바꿉니다.

</details>

### 연습 2. 명령줄 옵션이 환경 변수보다 우선하게 하기

`minigrep -i to poem.txt`는 대소문자를 무시하고, `minigrep -s to poem.txt`는 `IGNORE_CASE`가 설정돼 있어도 대소문자를 구분하게 하세요. 옵션이 없으면 지금처럼 환경 변수를 따릅니다. `Config::build`에 대한 테스트도 추가하세요.

<details>
<summary>힌트</summary>

- `args[1]`이 `"-i"`나 `"-s"`인지 먼저 확인합니다. 옵션이 있으면 검색어와 파일 경로가 한 칸씩 밀리므로 인자 개수 검사도 달라집니다.
- 옵션이 있으면 그 값으로 `ignore_case`를 정하고, 없을 때만 `env::var("IGNORE_CASE").is_ok()`를 씁니다.
- `-s`를 준 경우의 테스트는 환경 변수와 상관없이 결과가 정해지므로 언제 실행해도 안정적입니다.
- cargo로 실행할 때는 `cargo run -q -- -i to poem.txt`처럼 `--`가 꼭 필요합니다(4.4절). `IGNORE_CASE=1 cargo run -q -- -s to poem.txt`의 결과가 두 줄이면 성공입니다.

</details>

### 연습 3. `IGNORE_CASE=0`이면 끄기

지금은 `IGNORE_CASE=0`도 "설정됨"으로 보고 대소문자를 무시합니다. 값이 `0`, `false`, 빈 문자열이면 대소문자를 구분하고, 그 밖의 값이면 무시하게 바꾸세요.

<details>
<summary>힌트</summary>

- `is_ok()` 대신 `match env::var("IGNORE_CASE") { Ok(value) => ..., Err(_) => false }`로 값을 꺼내 비교합니다.
- 값을 해석하는 부분을 `fn parse_ignore_case(value: &str) -> bool` 같은 별도 함수로 빼면, 환경 변수를 건드리지 않고 `parse_ignore_case("0")`처럼 테스트할 수 있습니다.
- 테스트 안에서 환경 변수를 바꾸는 `std::env::set_var`는 2024 에디션에서 `unsafe` 함수라서 그냥 부르면 ``error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block``가 납니다. 값을 해석하는 함수를 따로 두는 쪽이 간단합니다.

</details>

### 연습 4. `grep`처럼 종료 코드로 결과 알리기

`grep`은 관례적으로 찾은 줄이 있으면 0, 없으면 1, 에러가 나면 2로 끝납니다(`grep monomorphization poem.txt; echo $?`로 확인해 보세요). `minigrep`도 그렇게 바꾸세요. 인자 문제와 파일 읽기 실패는 2입니다.

<details>
<summary>힌트</summary>

- `run`이 찾은 줄의 개수를 돌려주게 합니다: `Result<usize, Box<dyn Error>>`, 마지막은 `Ok(results.len())`.
- `main`에서는 `match minigrep::run(config) { Ok(0) => process::exit(1), Ok(_) => {}, Err(e) => { ... process::exit(2); } }`처럼 나눕니다. `Ok(0)`은 리터럴 패턴입니다.
- 찾은 줄이 없을 때는 아무것도 출력하지 않는 것이 `grep`의 동작입니다. 메시지를 내고 싶다면 표준 출력을 더럽히지 않도록 `eprintln!`을 씁니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [12. An I/O Project: Building a Command Line Program](https://doc.rust-lang.org/book/ch12-00-an-io-project.html)
  - [12.1 Accepting Command Line Arguments](https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html)
  - [12.2 Reading a File](https://doc.rust-lang.org/book/ch12-02-reading-a-file.html): `poem.txt`의 출처입니다.
  - [12.3 Refactoring to Improve Modularity and Error Handling](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html): "Separation of Concerns for Binary Projects"가 2.7절의 지침입니다.
  - [12.4 Adding Functionality with Test-Driven Development](https://doc.rust-lang.org/book/ch12-04-testing-the-librarys-functionality.html)
  - [12.5 Working with Environment Variables](https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html)
  - [12.6 Redirecting Errors to Standard Error](https://doc.rust-lang.org/book/ch12-06-writing-to-stderr-instead-of-stdout.html)
  - [11.3 Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html): "Integration Tests for Binary Crates"에서 로직을 `src/lib.rs`에 두는 이유를 통합 테스트 관점에서 설명합니다.
- Rust By Example
  - [Program arguments](https://doc.rust-lang.org/rust-by-example/std_misc/arg.html): `env::args()`로 0번 인자(실행 경로)와 나머지 인자를 출력하는 예제 01과 같은 내용입니다. 본격적인 명령줄 프로그램에는 `clap` 같은 크레이트를 쓴다는 안내도 있습니다.
  - [Argument parsing](https://doc.rust-lang.org/rust-by-example/std_misc/arg/matching.html): `match args.len()`으로 인자 개수에 따라 동작을 나누고, 잘못된 입력에는 `eprintln!`으로 사용법을 알리는 예제입니다. 연습 2를 풀 때 참고가 됩니다.
  - [File I/O](https://doc.rust-lang.org/rust-by-example/std_misc/file.html)와 [open](https://doc.rust-lang.org/rust-by-example/std_misc/file/open.html): `File::open`과 `read_to_string`으로 파일을 읽고, 실패를 `match`로 처리합니다. `fs::read_to_string`이 한 번에 해 주는 일을 두 단계로 나눠 볼 수 있습니다.
  - [read_lines](https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html): 파일 전체를 읽는 대신 `BufReader`로 한 줄씩 읽는 방법입니다. 반복자 어댑터와 제네릭을 쓰므로 `ch13-closures-iterators` 뒤에 읽으면 좋습니다.
- Rustlings: 명령줄 인자, 파일, 환경 변수를 다루는 연습이 없어 이 챕터에서는 안내하지 않습니다.
- API 문서
  - [`std::env::args`](https://doc.rust-lang.org/std/env/fn.args.html): 0번 인자는 관례상 실행 파일 경로일 뿐 무엇이든 될 수 있다는 점과, 유니코드가 아닌 인자에서 패닉한다는 점이 적혀 있습니다.
  - [`std::env::var`](https://doc.rust-lang.org/std/env/fn.var.html): 변수가 없을 때(`NotPresent`)와 값이 유니코드가 아닐 때(`NotUnicode`)의 에러를 설명합니다.
  - [`std::fs::read_to_string`](https://doc.rust-lang.org/std/fs/fn.read_to_string.html): 파일이 없거나 UTF-8이 아니면 에러를 돌려준다고 적혀 있습니다.
  - [`std::process::exit`](https://doc.rust-lang.org/std/process/fn.exit.html): 반환 타입이 `!`이고 소멸자(drop)가 실행되지 않는다는 주의 사항이 있습니다.
  - [`eprintln!`](https://doc.rust-lang.org/std/macro.eprintln.html): 에러와 진행 상황 메시지에만 쓰고, 프로그램의 주 출력에는 `println!`을 쓰라고 안내합니다.
