# ch1. Cargo 시작하기

Part 3의 첫 챕터입니다. [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md)에서 `cargo new`, `cargo build`, `cargo run`을 써 보았고, 그 뒤로 이 저장소의 모든 예제를 Cargo로 실행해 왔습니다. 이 챕터는 그동안 "그렇게 하면 된다"로 넘어간 것들을 정리합니다. Cargo가 `rustc` 위에서 무슨 일을 하는지, 파일을 어디에 두면 무엇이 되는지, `Cargo.toml`과 `Cargo.lock`은 왜 둘인지, 내려받은 크레이트는 어디에 있는지를 직접 확인합니다.

- 패키지: `p3-ch1-getting-started`
- 기준 문서: The Cargo Book [1. Getting Started](https://doc.rust-lang.org/cargo/getting-started/index.html), [2.1 Why Cargo Exists](https://doc.rust-lang.org/cargo/guide/why-cargo-exists.html), [2.2 Creating a New Package](https://doc.rust-lang.org/cargo/guide/creating-a-new-project.html), [2.3 Working on an Existing Package](https://doc.rust-lang.org/cargo/guide/working-on-an-existing-project.html), [2.5 Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html), [2.6 Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html), [2.10 Cargo Home](https://doc.rust-lang.org/cargo/guide/cargo-home.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64)
- 실습 자료: `examples/` 2개, `compile_fail/` 2개, 독립 프로젝트 [`projects/text-stat`](projects/text-stat/Cargo.toml)

> `projects/text-stat`은 crates.io의 `unicode-segmentation` 크레이트에 의존합니다. 처음 빌드할 때 네트워크가 필요합니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Cargo가 사람 대신 해 주는 일 네 가지를 설명하고, `cargo build -v`의 출력에서 Cargo가 만든 `rustc` 명령을 읽을 수 있다.
2. `cargo new`와 `cargo init`으로 바이너리 패키지와 라이브러리 패키지를 만들고, 패키지 이름으로 쓸 수 없는 이름을 구별할 수 있다.
3. 패키지 레이아웃 관례에 따라 파일을 두면 어떤 타깃이 되는지 설명하고, `--bin`·`--example`·`--test`로 타깃을 골라 빌드·실행할 수 있다.
4. `Cargo.toml`과 `Cargo.lock`의 역할을 구분하고, `--locked`로 잠금 파일이 바뀌지 않게 빌드할 수 있다.
5. Cargo home의 구성 요소를 설명하고, 내려받은 크레이트의 압축 파일과 소스가 어디에 있는지 찾을 수 있다.

## 2. 핵심 개념

### 2.1 Cargo가 하는 일

Rust 컴파일러는 `rustc`입니다. 파일 하나짜리 프로그램은 `rustc hello.rs`로 충분합니다. 그런데 크레이트가 둘만 되어도 사람이 챙길 것이 생깁니다. 예제 [`01_rustc_by_hand`](examples/01_rustc_by_hand.rs)는 라이브러리 `greet`와 그것을 쓰는 실행 파일 `app`을 `rustc`만으로 빌드합니다.

```text
== 1. 실행 파일만 컴파일해 보기 ==
$ rustc --edition 2024 app.rs
성공: false
error[E0432]: unresolved import `greet`

== 2. 라이브러리를 먼저 컴파일 ==
$ rustc --edition 2024 --crate-type lib greet.rs
성공: true
libgreet.rlib 생성: true

== 3. 라이브러리의 위치를 알려 주고 실행 파일 컴파일 ==
$ rustc --edition 2024 app.rs --extern greet=libgreet.rlib
성공: true
```

`rustc`는 한 번에 크레이트 하나만 컴파일하고, 다른 크레이트가 어디 있는지 스스로 찾지 않습니다. 그래서 사람이 세 가지를 정했습니다.

- **순서**: `greet`를 먼저, `app`을 나중에 컴파일합니다.
- **종류**: `greet`는 `--crate-type lib`로 라이브러리 파일(`libgreet.rlib`)을 만듭니다.
- **연결**: `--extern greet=libgreet.rlib`로 "`greet`라는 크레이트는 이 파일"이라고 알려 줍니다.

크레이트가 수십 개이고 그 대부분이 인터넷에서 받아 온 것이라면, 이 일을 손으로 할 수 없습니다. 받아 온 크레이트가 다시 다른 크레이트에 의존하고, 버전도 맞춰야 하기 때문입니다. Cargo Book은 Cargo가 이 문제를 풀기 위해 하는 일을 네 가지로 정리합니다.

| Cargo가 하는 일 | 이 챕터에서 보는 곳 |
| --- | --- |
| 패키지 정보를 담는 파일 두 개(`Cargo.toml`, `Cargo.lock`)를 둔다 | 2.4절 |
| 의존성을 내려받아 빌드한다 | 2.5절, 2.6절 |
| 알맞은 인자로 `rustc`를 부른다 | 바로 아래 |
| 패키지를 다루기 쉽게 하는 관례를 정한다 | 2.2절, 2.3절 |

세 번째 일은 `-v`(`--verbose`)로 볼 수 있습니다. `projects/text-stat`에서 `cargo build -v`를 실행하면 Cargo가 만든 `rustc` 명령이 `Running` 줄에 나옵니다. 아래는 길어서 `…`로 줄인 것입니다.

```text
   Compiling unicode-segmentation v1.13.3
     Running `…/rustc --crate-name unicode_segmentation --edition=2018 ~/.cargo/registry/src/…/unicode-segmentation-1.13.3/src/lib.rs … --crate-type lib …`
   Compiling text-stat v0.1.0 (…/projects/text-stat)
     Running `…/rustc --crate-name text_stat --edition=2024 src/lib.rs … --crate-type lib … --extern unicode_segmentation=…/libunicode_segmentation-….rmeta …`
     Running `…/rustc --crate-name longest_line --edition=2024 src/bin/longest-line.rs … --crate-type bin … --extern text_stat=…/libtext_stat-….rlib --extern unicode_segmentation=…`
     Running `…/rustc --crate-name top_words --edition=2024 src/bin/top-words/main.rs … --crate-type bin … --extern text_stat=…`
     Running `…/rustc --crate-name text_stat --edition=2024 src/main.rs … --crate-type bin … --extern text_stat=…`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.39s
```

손으로 정했던 세 가지가 모두 들어 있습니다. 의존받는 `unicode-segmentation`이 먼저 컴파일되고(순서), 라이브러리와 바이너리가 `--crate-type`으로 나뉘며(종류), `--extern`이 앞에서 만든 파일을 가리킵니다(연결). 바이너리 세 개는 서로 의존하지 않아 동시에 컴파일되므로 세 줄의 순서는 실행마다 다를 수 있습니다. `--edition`이 크레이트마다 다른 것도 보입니다. 의존성은 자기 `Cargo.toml`에 적힌 에디션으로 컴파일됩니다([Part 2 ch1](../../part2-editions/ch1-editions-overview/README.md)).

`rustc`의 옵션은 Part 5 `ch1-rustc-basics`에서 다룹니다. 여기서는 "Cargo는 `rustc`를 대신하는 것이 아니라 `rustc`를 부르는 도구"라는 점만 기억하면 됩니다. Cargo Book의 표현으로는, Cargo로 빌드하는 프로젝트 하나를 빌드할 줄 알면 모든 프로젝트를 빌드할 줄 아는 셈입니다. 어느 프로젝트든 `cargo build`이기 때문입니다.

Cargo의 설치는 Part 0 ch1에서 다뤘습니다. `rustup`으로 Rust를 설치하면 `cargo`가 함께 설치됩니다.

### 2.2 패키지 만들기: `cargo new`와 `cargo init`

패키지(package)는 `Cargo.toml` 하나가 다스리는 단위입니다. `cargo new 이름`은 디렉터리를 새로 만들고 그 안에 패키지를 만듭니다.

| 명령 | 만드는 것 | 소스 파일 |
| --- | --- | --- |
| `cargo new 이름` 또는 `cargo new --bin 이름` | 바이너리 패키지(기본값) | `src/main.rs` (Hello World) |
| `cargo new --lib 이름` | 라이브러리 패키지 | `src/lib.rs` (`add` 함수와 테스트 하나) |
| `cargo init` | 지금 있는 디렉터리를 패키지로 | 위와 같음. `--lib`도 받음 |

바이너리 패키지는 Part 0 ch1에서 만들어 보았으므로 여기서는 `--lib`을 봅니다. 저장소 밖의 빈 디렉터리에서 실행한 결과입니다.

```console
$ cargo new --lib hello_lib
    Creating library `hello_lib` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
```

```text
hello_lib/
├── .git/
├── .gitignore      내용: /target
├── Cargo.toml
└── src/
    └── lib.rs
```

`Cargo.toml`은 바이너리 패키지와 글자 하나 다르지 않습니다.

```toml
[package]
name = "hello_lib"
version = "0.1.0"
edition = "2024"

[dependencies]
```

"이 패키지는 라이브러리"라는 설정이 어디에도 없습니다. 차이는 `src/` 안의 파일 이름뿐입니다. `src/lib.rs`가 있으면 라이브러리가 있는 패키지이고, `src/main.rs`가 있으면 바이너리가 있는 패키지입니다. 이것이 Cargo의 방식인 **설정보다 관례**(convention over configuration)입니다. 관례를 따르면 `Cargo.toml`에 적을 것이 줄고, 처음 보는 패키지에서도 어디를 열어야 할지 압니다. 관례의 전체 목록은 2.3절에서 봅니다.

`cargo new`에서 알아 둘 옵션은 다음과 같습니다(`cargo help new`).

| 옵션 | 뜻 |
| --- | --- |
| `--name 이름` | 패키지 이름을 디렉터리 이름과 다르게 정합니다. 기본값은 디렉터리 이름입니다. |
| `--vcs none` | 버전 관리 저장소를 만들지 않습니다. 기본값은 `git`이고, 이미 저장소 안이면 `none`입니다. |
| `--edition 2021` | 에디션을 정합니다. 기본값은 2024입니다. |

`cargo init`은 이미 있는 디렉터리에서 씁니다. 있는 소스 파일은 덮어쓰지 않습니다. `src/lib.rs`가 이미 있는 디렉터리에서 실행하면 `Cargo.toml`만 만들고 `src/main.rs`는 만들지 않았습니다(이때도 출력은 `Creating binary (application) package`라서 헷갈릴 수 있습니다). git 저장소에 `.gitignore`가 이미 있으면 그 끝에 `# Added by cargo`와 `/target`을 덧붙입니다. `cargo new`에 이미 있는 디렉터리 이름을 주면 에러와 함께 `cargo init`을 쓰라고 알려 줍니다.

```text
error: destination `…/hello_bin` already exists

Use `cargo init` to initialize the directory
```

**패키지 이름의 규칙.** 패키지 이름은 크레이트 이름이 되고, 크레이트 이름은 코드에서 `use 이름::…`으로 쓰입니다. 그래서 Rust 식별자가 될 수 없는 이름은 거절됩니다. 다음은 `cargo new`에 준 이름과 실제로 받은 에러입니다.

| 준 이름 | 에러 |
| --- | --- |
| `2048` | ``invalid character `2` in package name: `2048`, the name cannot start with a digit`` |
| `self` | ``invalid package name `self`: it is a Rust keyword`` |
| `test` | ``invalid package name `test`: it conflicts with Rust's built-in test library`` |
| `my.pkg` | ``invalid character `.` in package name: `my.pkg`, characters must be Unicode XID characters (numbers, `-`, `_`, or most letters)`` |

`-`는 예외입니다. 식별자에는 쓸 수 없지만 패키지 이름에는 쓸 수 있고, Cargo가 크레이트 이름을 만들 때 `_`로 바꿉니다. 이 챕터의 실습 패키지 `text-stat`의 라이브러리 크레이트는 `text_stat`입니다(2.3절).

### 2.3 패키지 레이아웃: 파일의 위치가 타깃을 정한다

Cargo가 빌드하는 대상 하나하나를 **타깃(target)** 이라고 합니다. 라이브러리, 바이너리, 예제, 통합 테스트, 벤치마크가 타깃입니다. 타깃 하나는 크레이트 하나로 컴파일됩니다. Cargo는 정해진 위치의 파일을 타깃으로 알아봅니다.

[`projects/text-stat`](projects/text-stat/Cargo.toml)은 글의 줄·낱말·글자 수를 세는 패키지이고, 관례가 정하는 위치를 거의 다 씁니다.

```text
text-stat/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── lib.rs                 라이브러리 text_stat
│   ├── main.rs                바이너리 text-stat (패키지 이름과 같은 이름)
│   └── bin/
│       ├── longest-line.rs    바이너리 longest-line (파일 이름)
│       └── top-words/         바이너리 top-words (디렉터리 이름)
│           ├── main.rs
│           └── rank.rs        top-words만 쓰는 모듈
├── examples/
│   └── quick.rs               예제 quick
└── tests/
    └── counts.rs              통합 테스트 counts
```

| 파일 위치 | 타깃 종류 | 타깃 이름 | 개수 |
| --- | --- | --- | --- |
| `src/lib.rs` | 라이브러리 | 패키지 이름(`-`는 `_`로) | 패키지에 하나까지 |
| `src/main.rs` | 바이너리 | 패키지 이름 | 하나 |
| `src/bin/이름.rs` | 바이너리 | 파일 이름 | 여럿 |
| `src/bin/이름/main.rs` | 바이너리(여러 파일) | 디렉터리 이름 | 여럿 |
| `examples/이름.rs`, `examples/이름/main.rs` | 예제 | 파일 또는 디렉터리 이름 | 여럿 |
| `tests/이름.rs`, `tests/이름/main.rs` | 통합 테스트 | 파일 또는 디렉터리 이름 | 여럿 |
| `benches/이름.rs`, `benches/이름/main.rs` | 벤치마크 | 파일 또는 디렉터리 이름 | 여럿 |

이 저장소의 챕터 패키지가 `examples/01_….rs` 파일만으로 이뤄진 것도 이 표의 다섯째 줄 덕분입니다. `tests/`는 [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md)에서 썼고, `benches/`는 [ch9](../ch9-testing-ci/README.md)에서 다룹니다. 관례와 다른 위치나 이름을 쓰려면 `Cargo.toml`에 타깃을 직접 적어야 하며, 이것은 [ch3](../ch3-manifest-targets/README.md)의 주제입니다.

**여러 파일로 된 타깃.** 타깃 하나가 여러 파일로 이뤄지면 `이름/main.rs`와 모듈 파일을 한 디렉터리에 둡니다. `top-words/main.rs`는 `mod rank;`로 같은 디렉터리의 `rank.rs`를 자기 모듈로 선언합니다. `src/bin/` 바로 아래에 `rank.rs`를 두면 안 됩니다. 그 자리의 `.rs` 파일은 모두 바이너리로 취급되어 `rank`라는 실행 파일을 만들려다 `main` 함수가 없다는 에러가 납니다.

**이름을 짓는 관례.** Cargo Book은 바이너리, 예제, 벤치마크, 통합 테스트의 이름은 `kebab-case`(`longest-line`)로, 그 안의 모듈은 Rust 관례대로 `snake_case`로 짓기를 권합니다. 실행 파일 이름은 `longest-line` 그대로이고, 크레이트 이름만 `longest_line`으로 바뀝니다(2.1절의 `--crate-name longest_line`). 이 저장소의 예제 이름(`01_rustc_by_hand`)이 `snake_case`인 것은 번호순으로 늘어놓으려고 저장소가 따로 정한 규칙입니다.

**라이브러리와 바이너리는 다른 크레이트.** `cargo build`가 `text-stat` 패키지에서 만드는 크레이트는 넷입니다(라이브러리 하나, 바이너리 셋). 바이너리 크레이트는 같은 패키지의 라이브러리 크레이트를 외부 크레이트처럼 이름으로 가져옵니다.

```rust
// src/main.rs
use text_stat::{Counts, count, load};
```

Cargo가 2.1절의 `--extern text_stat=…`을 붙여 주기 때문에 `Cargo.toml`에 따로 적지 않아도 됩니다. 바이너리 셋이 함께 쓰는 코드(`load`, `count`)는 라이브러리에 두고, 바이너리 하나만 쓰는 코드(`rank`)는 그 바이너리의 모듈로 둡니다. [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.12절에서 본 "얇은 `main.rs` + 라이브러리" 구조가 바이너리 여럿으로 늘어난 것입니다.

**타깃 고르기.** 타깃이 여럿이면 명령에 어느 타깃인지 알려 줍니다. `projects/text-stat` 디렉터리에서 실행합니다.

```console
$ cargo run --bin text-stat
줄 4 / 낱말 16 / char 79 / 자소 클러스터 75
$ cargo run --bin longest-line
가장 긴 줄(27자): cargo run 은 빌드한 뒤 바로 실행합니다.
$ cargo run --bin top-words
2번: cargo
1번: Cargo는
1번: Rust의
$ cargo run --example quick
Counts { lines: 2, words: 3, chars: 13, graphemes: 13 }
```

(`Compiling`, `Finished`, `Running` 줄은 생략했습니다. `-q`를 붙이면 실제로도 나오지 않습니다.)

이름을 빼고 옵션만 주면 Cargo가 찾아낸 타깃의 목록을 보여 줍니다. 관례가 내 파일을 어떻게 읽었는지 확인하는 가장 빠른 방법입니다.

```console
$ cargo run --bin
error: "--bin" takes one argument.
Available binaries:
    longest-line
    text-stat
    top-words

$ cargo test --test
error: "--test" takes one argument.
Available test targets:
    counts

```

| 옵션 | 고르는 타깃 | 쓸 수 있는 명령의 예 |
| --- | --- | --- |
| `--lib` | 라이브러리 | `cargo build --lib`, `cargo test --lib` |
| `--bin 이름` | 바이너리 하나 | `cargo run --bin 이름`, `cargo build --bin 이름` |
| `--example 이름` | 예제 하나 | `cargo run --example 이름` |
| `--test 이름` | 통합 테스트 하나 | `cargo test --test 이름` |

옵션 없는 `cargo build`는 라이브러리와 모든 바이너리를 빌드합니다. 예제와 테스트는 빌드하지 않습니다(`--examples`, `--tests`, `--all-targets`로 포함합니다). `cargo test`는 라이브러리의 단위 테스트, 바이너리마다의 단위 테스트, 통합 테스트, 문서 테스트를 차례로 실행합니다. `text-stat`에서는 `Running` 줄이 다섯 번, `Doc-tests` 줄이 한 번 나옵니다.

**빌드 결과가 놓이는 곳.** `cargo build` 뒤의 `target/debug/`입니다.

```console
$ ls target/debug
build            libtext_stat.d     longest-line.d  top-words
examples         libtext_stat.rlib  text-stat       top-words.d
incremental      longest-line       text-stat.d
```

실행 파일 셋과 라이브러리 파일(`libtext_stat.rlib`)이 있습니다. `.d` 파일은 그 산출물이 어느 소스 파일로 만들어졌는지 적은 목록입니다. `build/`에는 크레이트마다의 컴파일 결과가 들어 있습니다. 2.1절의 `--extern`이 가리키던 `…/target/debug/build/<패키지>/<해시>/out/` 아래의 파일이 그것이고, 위 목록의 실행 파일과 `.rlib`은 거기서 꺼내 놓은 것입니다. `incremental/`은 `rustc`가 다음 컴파일을 빠르게 하려고 남기는 캐시입니다. 예제의 실행 파일은 `cargo build --examples`나 `cargo run --example`을 실행하면 `target/debug/examples/` 아래에 생깁니다. `target/` 안의 구조는 [ch6](../ch6-profiles-build-performance/README.md)에서 다시 봅니다.

### 2.4 `Cargo.toml`과 `Cargo.lock`

[Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md) 2.3절에서 `Cargo.toml`의 `rand = "0.10.4"`는 범위이고 실제 버전은 `Cargo.lock`에 적힌다는 것을 보았습니다. 두 파일의 역할을 나란히 놓으면 이렇습니다.

| | `Cargo.toml` — 매니페스트(manifest) | `Cargo.lock` — 잠금 파일(lockfile) |
| --- | --- | --- |
| 누가 쓰나 | 사람(또는 `cargo add`) | Cargo. 손으로 고치지 않음 |
| 무엇을 적나 | 의존성을 **넓게**: 어떤 크레이트의 어느 범위 | 의존성을 **정확하게**: 고른 버전, 출처, 체크섬 |
| 답하는 질문 | "이 패키지는 무엇이 필요한가" | "지난번에 실제로 무엇으로 빌드했는가" |

**왜 둘로 나눴는가.** 파일이 하나뿐이라고 해 봅니다. 범위만 적으면, 오늘 내가 빌드한 버전과 다음 달 동료가 빌드한 버전이 달라질 수 있습니다. 그 사이에 범위 안의 새 버전이 게시되기 때문입니다. 같은 커밋인데 빌드 결과가 다르면 버그의 원인을 찾기 어렵습니다. 반대로 정확한 버전만 적으면 빌드는 같아지지만, 새 버전으로 올릴 때마다 사람이 버전 번호를 하나하나 고쳐야 하고 "어디까지 올려도 되는가"라는 정보가 사라집니다. 그래서 사람은 범위를 적고(`Cargo.toml`), Cargo가 그 범위에서 고른 결과를 따로 기록합니다(`Cargo.lock`). 올리고 싶을 때는 `cargo update`가 범위 안에서 다시 골라 잠금 파일을 고칩니다.

**잠금 파일 읽기.** `text-stat`의 `Cargo.toml`에는 의존성이 한 줄입니다.

```toml
[dependencies]
unicode-segmentation = "1.13.3"
```

[`Cargo.lock`](projects/text-stat/Cargo.lock)에는 같은 의존성이 이렇게 적혀 있습니다.

```toml
version = 4

[[package]]
name = "text-stat"
version = "0.1.0"
dependencies = [
 "unicode-segmentation",
]

[[package]]
name = "unicode-segmentation"
version = "1.13.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6f5d3c3b1bf09027a88a6bc961fc00497d651009560b5463668dc81b0fa87a8"
```

- 맨 위의 `version = 4`는 잠금 파일 형식의 버전입니다. 패키지의 버전과는 관계가 없습니다.
- `[[package]]`는 빌드에 들어가는 패키지마다 하나씩 있습니다. 자기 자신(`text-stat`)도 들어 있고 `dependencies`로 서로의 관계가 적힙니다.
- `source`는 받아 온 곳입니다. 레지스트리(registry)는 크레이트를 게시하고 내려받는 저장소이고, Cargo가 기본으로 쓰는 레지스트리가 crates.io입니다(게시는 [ch10](../ch10-publishing-semver/README.md)). `registry+…crates.io-index`는 crates.io에서 받았다는 뜻입니다. 내 디스크에 있는 `text-stat`에는 `source`가 없습니다.
- `checksum`은 내려받은 `.crate` 파일의 SHA-256 값입니다. 2.6절에서 찾을 파일에 `shasum -a 256`을 실행하면 같은 값이 나옵니다. 누군가 같은 버전 번호로 다른 내용을 내려보내면 Cargo가 알아챕니다.

**잠금 파일은 언제 바뀌는가.** 잠금 파일에 적힌 버전이 `Cargo.toml`의 요구 사항을 만족하는 동안에는 Cargo가 다시 고르지 않습니다. Cargo가 의존성의 버전을 다시 고르는 것은 잠금 파일이 없을 때, 의존성이 추가·삭제됐을 때, 요구 사항이 잠긴 버전을 벗어났을 때, 그리고 `cargo update`를 실행했을 때입니다. 잠금 파일에는 패키지 자신도 적혀 있으므로 `[package]`의 `version`을 바꿔도 그 줄이 바뀝니다.

`--locked`는 "잠금 파일을 고쳐야 하는 상황이면 고치지 말고 실패하라"는 옵션입니다. `text-stat`을 저장소 밖으로 복사한 사본에서 확인한 결과입니다.

| 한 일 | `cargo build --locked` | 옵션 없는 `cargo build` |
| --- | --- | --- |
| 아무것도 바꾸지 않음 | 성공 | 성공. 잠금 파일 그대로 |
| 요구 사항을 `"1.12"`로 낮춤(1.12.0 이상 2.0.0 미만) | 성공(잠긴 1.13.3이 여전히 범위 안) | 성공. 잠금 파일 그대로 |
| `Cargo.lock`을 지움 | 실패(종료 코드 101) | 성공. 잠금 파일을 새로 만듦 |
| `[dependencies]`에 `unicode-width = "0.2"`를 추가 | 실패(종료 코드 101) | 성공. 잠금 파일에 패키지를 추가 |

실패할 때의 메시지입니다(경로는 줄였습니다).

```text
error: cannot create the lock file …/Cargo.lock because --locked was passed to prevent this
help: to generate the lock file without accessing the network, remove the --locked flag and use --offline instead.
```

```text
error: cannot update the lock file …/Cargo.lock because --locked was passed to prevent this
help: to generate the lock file without accessing the network, remove the --locked flag and use --offline instead.
```

옵션 없이 빌드하면 Cargo는 조용히 잠금 파일을 고칩니다. 그래서 "커밋된 잠금 파일 그대로 빌드되는지"를 확인해야 하는 곳(CI, 배포용 빌드)에서는 `--locked`를 붙입니다. 의존성을 추가한 마지막 줄에서 옵션 없이 빌드하면 다음이 나왔습니다. `unicode-width`의 버전은 2026년 11월 기준이며, 그 뒤에 새 버전이 나오면 다릅니다.

```text
    Updating crates.io index
     Locking 1 package to highest Rust 1.100.0 compatible version
      Adding unicode-width v0.2.3
   Compiling unicode-width v0.2.3
   Compiling text-stat v0.1.0 (…)
warning: unused dependency `unicode-width`
```

마지막 경고는 `Cargo.toml`에 적어 놓고 코드에서 쓰지 않은 의존성을 Cargo 1.100.0이 알려 주는 것입니다.

**잠금 파일을 버전 관리에 넣을까.** Cargo Book의 답은 "잘 모르겠으면 넣는다"입니다. `cargo new`가 만드는 `.gitignore`에는 `/target`뿐이라 `Cargo.lock`은 기본으로 커밋 대상입니다. 잠금 파일이 저장소에 있으면 과거 커밋을 그때와 같은 의존성으로 빌드할 수 있고, CI가 내 변경 때문이 아니라 바깥에서 새 버전이 게시된 탓에 실패하는 일이 없습니다.

한 가지 알아 둘 한계가 있습니다. 잠금 파일은 **그 패키지를 직접 빌드할 때만** 쓰입니다. 내 라이브러리를 의존성으로 쓰는 다른 패키지는 내 `Cargo.lock`을 보지 않고 내 `Cargo.toml`의 범위만 봅니다. 그래서 라이브러리는 잠금 파일을 커밋했더라도 `Cargo.toml`의 범위를 정확히 적어야 합니다. 범위를 적는 문법과 Cargo가 버전을 고르는 규칙은 다음 챕터 [ch2](../ch2-dependencies/README.md)에서 다룹니다.

### 2.5 이미 있는 패키지에서 작업하기

다른 사람이 만든 패키지를 받아 빌드하는 순서는 어느 패키지든 같습니다.

```sh
git clone <저장소 주소>
cd <디렉터리>
cargo build
```

`cargo build`는 `Cargo.lock`에 적힌 의존성을 내려받고, 의존성과 패키지를 순서대로 빌드합니다. 이 학습 저장소가 바로 그런 예입니다. 저장소를 받아 Part 0 ch2의 예제(`cargo run -p p0-ch2-guessing-game --example …`)를 처음 실행하면 `Downloaded rand v0.10.4` 같은 줄이 지나간 뒤 예제가 실행됩니다. Cargo는 고른 패키지에 필요한 의존성만 내려받으므로, 의존성이 없는 챕터의 예제를 실행할 때는 `Downloaded` 줄이 나오지 않습니다.

내려받기와 빌드를 따로 하고 싶을 때 쓰는 명령과 옵션입니다.

| 명령·옵션 | 하는 일 |
| --- | --- |
| `cargo fetch` | 의존성을 내려받기만 하고 빌드하지 않습니다. |
| `--offline` | 네트워크를 쓰지 않습니다. 이미 내려받은 것만으로 빌드합니다. |
| `--locked` | 잠금 파일을 고치지 않습니다(2.4절). |

비행기를 타기 전에 `cargo fetch`를 해 두면 `cargo build --offline`으로 빌드할 수 있습니다. 내려받은 것이 어디에 있는지는 2.6절에서 봅니다.

Cargo는 현재 디렉터리에 `Cargo.toml`이 없으면 부모 디렉터리로 올라가며 찾습니다. 그래서 패키지의 `src/` 안에서 `cargo build`를 실행해도 됩니다. 끝까지 찾지 못하면 다음 에러가 납니다.

```text
error: could not find `Cargo.toml` in `…` or any parent directory
```

### 2.6 Cargo home

`target/`이 "이 패키지의 빌드 결과"를 두는 곳이라면, **Cargo home**은 "이 컴퓨터의 모든 패키지가 함께 쓰는 것"을 두는 곳입니다. 내려받은 크레이트와 `cargo install`로 설치한 실행 파일이 여기에 있습니다. `cargo install`은 crates.io에 게시된 프로그램을 내려받아 빌드하고 실행 파일을 `bin/`에 넣는 명령입니다([ch10](../ch10-publishing-semver/README.md)). 위치는 환경 변수 `CARGO_HOME`이 정하고, 없으면 홈 디렉터리의 `.cargo`입니다.

| 경로 | 내용 |
| --- | --- |
| `config.toml` | 모든 패키지에 적용되는 Cargo 설정([ch7](../ch7-configuration-env/README.md)) |
| `credentials.toml` | `cargo login`으로 저장한 레지스트리 토큰. 남에게 보이면 안 되는 파일 |
| `.crates.toml`, `.crates2.json` | `cargo install`로 설치한 크레이트의 기록. 손으로 고치지 않음 |
| `bin/` | `cargo install`과 `rustup`이 넣은 실행 파일. `PATH`에 넣어 둘 디렉터리 |
| `registry/index/` | 레지스트리의 색인(index). 어떤 크레이트의 어떤 버전이 있고 무엇에 의존하는지 |
| `registry/cache/` | 내려받은 크레이트의 압축 파일(`이름-버전.crate`) |
| `registry/src/` | `.crate`를 풀어 둔 소스. `rustc`가 읽는 곳 |
| `git/db/` | git 의존성의 저장소 사본 |
| `git/checkouts/` | 그 사본에서 꺼낸 특정 커밋의 소스 |

Cargo Book은 이 내부 구조가 안정화된 것이 아니라 언제든 바뀔 수 있다고 밝힙니다. 스크립트가 이 경로에 기대면 안 됩니다. 실제로 책은 `registry/index`를 git 저장소라고 설명하지만, cargo 1.100.0의 기본값인 sparse 프로토콜은 필요한 크레이트의 색인 항목만 HTTPS로 받아 `registry/index/…/.cache/`에 파일로 둡니다(`registries.crates-io.protocol`의 기본값이 `"sparse"`).

**빈 Cargo home이 채워지는 모습.** `CARGO_HOME`을 빈 임시 디렉터리로 바꾸고 `text-stat`의 사본에서 `cargo fetch`를 실행하면 의존성 하나가 어디에 무엇을 만드는지 그대로 보입니다.

```console
$ CARGO_HOME=/tmp/fresh-home cargo fetch --locked
    Updating crates.io index
 Downloading crates ...
  Downloaded unicode-segmentation v1.13.3
```

```text
/tmp/fresh-home/
├── .global-cache
├── .package-cache
├── .package-cache-mutate
└── registry/
    ├── CACHEDIR.TAG
    ├── cache/index.crates.io-1949cf8c6b5b557f/
    │   └── unicode-segmentation-1.13.3.crate
    ├── index/index.crates.io-1949cf8c6b5b557f/
    │   ├── .cache/
    │   └── config.json
    └── src/index.crates.io-1949cf8c6b5b557f/
        └── unicode-segmentation-1.13.3/
```

`cache/`의 `.crate` 파일(약 110 KiB)이 내려받은 원본이고, `src/`의 디렉터리가 그것을 푼 것입니다. 2.1절의 `rustc` 명령이 읽던 `~/.cargo/registry/src/…/unicode-segmentation-1.13.3/src/lib.rs`가 이 자리입니다. `index.crates.io-…`는 레지스트리마다 하나씩 생기는 디렉터리입니다. 표에 없는 `.global-cache` 같은 파일은 Cargo가 내부적으로 쓰는 것으로, 문서가 설명하지 않습니다. 이어서 `CARGO_HOME=/tmp/fresh-home cargo build --offline`을 실행하면 네트워크 없이 빌드됩니다.

예제 [`02_cargo_home`](examples/02_cargo_home.rs)은 지금 쓰는 Cargo home에 위 표의 항목이 있는지 보여 줍니다.

**지워도 되는가.** Cargo Book에 따르면 캐시의 어느 부분을 지워도 Cargo가 필요할 때 다시 풀거나 다시 내려받습니다. `cargo clean`은 패키지의 `target/`만 지우고 Cargo home은 건드리지 않습니다. 그리고 Cargo는 오래 쓰지 않은 캐시를 스스로 지웁니다. 설정 문서의 `[cache]` 절에 따르면 네트워크에서 다시 받아야 하는 파일은 3개월, 네트워크 없이 다시 만들 수 있는 파일(`.crate`를 푼 소스 등)은 1개월 동안 쓰이지 않으면 `cargo build` 같은 명령을 실행할 때 지워집니다.

CI에서 Cargo home을 캐시할 때는 통째로 넣지 않습니다. `registry/cache/`와 `registry/src/`에는 같은 소스가 압축본과 푼 것으로 두 번 들어 있기 때문입니다. Cargo Book이 권하는 목록은 `.crates.toml`, `.crates2.json`, `bin/`, `registry/index/`, `registry/cache/`, `git/db/`입니다. CI는 [ch9](../ch9-testing-ci/README.md)에서 다룹니다.

## 3. 예제 안내

저장소 루트에서 실행합니다.

| 예제 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_rustc_by_hand`](examples/01_rustc_by_hand.rs) | Cargo 없이 `rustc`로 라이브러리와 실행 파일 빌드 | 1 |
| [`02_cargo_home`](examples/02_cargo_home.rs) | Cargo home의 구성 요소 | 5 |
| [`projects/text-stat`](projects/text-stat/Cargo.toml) | 패키지 레이아웃, 타깃 고르기, 잠금 파일 | 1, 2, 3, 4 |

### 01_rustc_by_hand — Cargo가 대신 해 주는 일

```sh
cargo run -p p3-ch1-getting-started --example 01_rustc_by_hand
```

임시 디렉터리에 `greet.rs`(라이브러리)와 `app.rs`(실행 파일)를 쓰고 `rustc`를 세 번 실행합니다. 첫 번째는 일부러 실패시킵니다. 예제 자체의 종료 코드는 0입니다.

```text
== 1. 실행 파일만 컴파일해 보기 ==
$ rustc --edition 2024 app.rs
성공: false
error[E0432]: unresolved import `greet`

== 2. 라이브러리를 먼저 컴파일 ==
$ rustc --edition 2024 --crate-type lib greet.rs
성공: true
libgreet.rlib 생성: true

== 3. 라이브러리의 위치를 알려 주고 실행 파일 컴파일 ==
$ rustc --edition 2024 app.rs --extern greet=libgreet.rlib
성공: true

== 4. 실행 ==
$ ./app
안녕하세요, Cargo 없이 빌드한 프로그램!

크레이트 2개에 명령 2개와 순서 1가지를 손으로 챙겼습니다.
Cargo는 Cargo.toml과 디렉터리 관례를 읽어 이 명령들을 대신 만듭니다.
```

### 02_cargo_home — Cargo home 둘러보기

```sh
cargo run -p p3-ch1-getting-started --example 02_cargo_home
```

**출력은 컴퓨터마다 다릅니다.** 아래는 이 문서를 쓴 컴퓨터의 결과입니다. `config.toml`을 만든 적이 없고, git 의존성을 받은 적이 없어서 "없음"으로 나옵니다. `.crate` 파일의 수는 지금까지 내려받은 크레이트의 수입니다.

```text
Cargo home을 찾은 방법: 환경 변수 CARGO_HOME

== 파일 ==
없음   config.toml        모든 패키지에 적용되는 Cargo 설정
없음   credentials.toml   cargo login으로 저장한 레지스트리 토큰
있음   .crates.toml       cargo install로 설치한 크레이트 기록
있음   .crates2.json      cargo install로 설치한 크레이트 기록

== 디렉터리 ==
있음   bin                cargo install과 rustup이 넣은 실행 파일
있음   registry/index     레지스트리 색인(어떤 크레이트의 어떤 버전이 있는가)
있음   registry/cache     내려받은 .crate 압축 파일
있음   registry/src       .crate를 풀어 둔 소스(rustc가 읽는 곳)
없음   git/db             git 의존성의 저장소 사본
없음   git/checkouts      git 의존성에서 꺼낸 특정 커밋의 소스

== 레지스트리 캐시 ==
index.crates.io-1949cf8c6b5b557f: .crate 파일 282개
  $CARGO_HOME/registry/cache/index.crates.io-1949cf8c6b5b557f/unicode-segmentation-1.13.3.crate
  $CARGO_HOME/registry/src/index.crates.io-1949cf8c6b5b557f/unicode-segmentation-1.13.3/ (있음)
```

`projects/text-stat`을 한 번도 빌드하지 않았다면 마지막 두 줄 대신 `unicode-segmentation-*.crate 없음`이 나옵니다. 그 디렉터리에서 `cargo fetch`를 실행한 뒤 다시 실행해 봅니다. 첫 줄이 "환경 변수 CARGO_HOME"인 것은 `rustup`이 설치한 `cargo` 명령이 실제 `cargo`를 실행하면서 이 환경 변수를 넣어 주기 때문입니다. 툴체인 디렉터리의 `cargo`를 직접 실행하면 변수가 없어 "홈 디렉터리의 .cargo"가 나옵니다.

### projects/text-stat — 관례대로 놓은 패키지

저장소의 워크스페이스에 속하지 않는 독립 프로젝트입니다. 그 디렉터리로 들어가서 실행합니다.

```sh
cd part3-cargo/ch1-getting-started/projects/text-stat
```

| 명령 | 기대 결과 |
| --- | --- |
| `cargo run --bin text-stat` | `줄 4 / 낱말 16 / char 79 / 자소 클러스터 75` |
| `cargo run --bin text-stat -- Cargo.toml` | `줄 11 / 낱말 39 / char 222 / 자소 클러스터 222` |
| `cargo run --bin longest-line` | `가장 긴 줄(27자): cargo run 은 빌드한 뒤 바로 실행합니다.` |
| `cargo run --bin top-words` | `2번: cargo`, `1번: Cargo는`, `1번: Rust의` 세 줄 |
| `cargo run --example quick` | `Counts { lines: 2, words: 3, chars: 13, graphemes: 13 }` |
| `cargo test` | 단위 테스트 2개, 통합 테스트 2개, 문서 테스트 1개 통과 |
| `cargo run` | 실패(종료 코드 101). 4.1절의 에러 |
| `cargo build -v` | 2.1절의 `Running` 줄(이미 빌드했다면 `cargo clean` 뒤에) |

인자 없이 실행하면 라이브러리에 들어 있는 예시 글을 셉니다. 예시 글의 마지막 줄은 조합형으로 적은 "한글"이라 `char`는 6개, 자소 클러스터는 2개입니다. 그래서 첫 줄의 두 숫자가 4만큼 다릅니다([Part 0 ch8](../../part0-getting-started/ch8-collections/README.md)에서 본 자소 클러스터를 세려고 `unicode-segmentation` 크레이트를 씁니다). `Cargo.toml` 끝의 빈 `[workspace]` 테이블은 이 학습 저장소 안에 두려고 넣은 것이며, 저장소 밖에서 만든 패키지에는 필요 없습니다.

확인이 끝나면 `cargo clean`으로 `target/`을 지웁니다.

## 4. 자주 하는 실수

Cargo 챕터의 실수는 대부분 `rustc`가 아니라 Cargo가 내는 에러입니다. 코드가 틀려서 나는 두 가지만 `compile_fail/`에 두었고, 나머지는 `text-stat`의 사본에서 재현한 메시지를 실었습니다. `compile_fail/`의 파일은 챕터 디렉터리에서 다음과 같이 확인합니다.

```sh
cd part3-cargo/ch1-getting-started
rustc --edition 2024 --out-dir /tmp/p3-ch1 compile_fail/hyphen_in_crate_name.rs
rustc --edition 2024 --out-dir /tmp/p3-ch1 compile_fail/e0432_crate_is_not_the_library.rs
```

### 4.1 바이너리가 여럿인데 `cargo run`만 실행

```text
error: `cargo run` could not determine which binary to run. Use the `--bin` option to specify a binary, or the `default-run` manifest key.
available binaries: longest-line, text-stat, top-words
```

- **원인**: `cargo run`은 바이너리 하나를 실행하는 명령인데 후보가 셋입니다. `src/main.rs`가 있다고 해서 그것이 기본이 되지는 않습니다.
- **해결**: `cargo run --bin text-stat`처럼 이름을 줍니다. 메시지가 말하는 `default-run` 키는 [ch3](../ch3-manifest-targets/README.md)에서 다룹니다.

### 4.2 패키지 이름의 `-`를 코드에 그대로 씀 — [`hyphen_in_crate_name.rs`](compile_fail/hyphen_in_crate_name.rs)

```text
error: expected one of `::`, `;`, or `as`, found `-`
  --> compile_fail/hyphen_in_crate_name.rs:13:9
   |
13 | use text-stat::count;
   |         ^ expected one of `::`, `;`, or `as`
```

- **원인**: `-`는 식별자에 쓸 수 없는 글자라서 컴파일러가 `text` 다음에 온 `-`를 이해하지 못합니다. 에러 코드가 없는 문법 에러입니다.
- **해결**: `use text_stat::count;`로 씁니다. 패키지 이름과 의존성 이름은 `Cargo.toml`에서는 `-` 그대로, 코드에서는 `_`입니다(`unicode-segmentation` → `unicode_segmentation`).

### 4.3 `main.rs`에서 라이브러리를 `crate::`로 가져옴 — E0432, [`e0432_crate_is_not_the_library.rs`](compile_fail/e0432_crate_is_not_the_library.rs)

```text
error[E0432]: unresolved import `crate::count`
  --> compile_fail/e0432_crate_is_not_the_library.rs:16:5
   |
16 | use crate::count;
   |     ^^^^^^^-----
   |            |
   |            no `count` in the root
```

- **원인**: `src/lib.rs`와 `src/main.rs`는 서로 다른 크레이트입니다. `crate::`는 지금 컴파일하는 크레이트의 루트이므로 `main.rs` 안에서는 바이너리 크레이트 자신을 가리키고, 거기에는 `count`가 없습니다. `text-stat`의 `src/main.rs`에서 `use text_stat::`를 `use crate::`로 바꾸면 같은 에러가 `Counts`, `count`, `load` 세 이름에 대해 납니다.
- **해결**: 라이브러리 크레이트의 이름으로 가져옵니다(`use text_stat::count;`). [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.12절에서 `crate::eat_at_restaurant()`처럼 경로로 직접 불렀을 때는 E0425였고, `use`로 가져오면 같은 원인이 E0432로 나옵니다. `main.rs`에 `mod`로 라이브러리의 모듈 파일을 다시 선언하는 실수는 [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 4.6절에 있습니다.

### 4.4 그 밖의 Cargo 에러

| 한 일 | 메시지 | 원인과 해결 |
| --- | --- | --- |
| `src/main.rs`가 있는데 `src/bin/text-stat.rs`를 추가 | `found duplicate binary name text-stat, but all binary targets must have a unique name` | `src/main.rs`의 바이너리 이름이 패키지 이름이라 겹칩니다. 파일 이름을 바꿉니다. |
| `Cargo.toml`만 있고 `src/`가 없음 | `no targets specified in the manifest` / `either src/lib.rs, src/main.rs, a [lib] section, or [[bin]] section must be present` | 타깃이 하나도 없는 패키지는 빌드할 수 없습니다. `src/lib.rs`나 `src/main.rs`를 만듭니다. |
| 패키지 밖에서 `cargo build` | ``could not find `Cargo.toml` in `…` or any parent directory`` | 패키지 디렉터리로 이동합니다(2.5절). |
| 잠금 파일을 고쳐야 하는데 `--locked` | `cannot update the lock file … because --locked was passed to prevent this` | 의도한 변경이면 `--locked` 없이 한 번 빌드해 잠금 파일을 갱신하고 커밋합니다(2.4절). |
| 이 저장소 안에서 `cargo new` | 루트 `Cargo.toml`의 `members`가 조용히 바뀌거나, 빌드할 때 ``current package believes it's in a workspace when it's not`` | [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md) "자주 하는 실수" 4번. 저장소 밖에서 만듭니다. |

## 5. 연습 문제

연습은 저장소 밖에서 합니다. `projects/text-stat`을 고치는 문제는 먼저 사본을 만듭니다.

```sh
cp -R part3-cargo/ch1-getting-started/projects/text-stat /tmp/text-stat
cd /tmp/text-stat
```

### 연습 1. 바이너리 하나 더

사본에 글의 `char` 수만 출력하는 바이너리 `char-count`를 추가하세요. `Cargo.toml`은 고치지 않습니다. `cargo run --bin`의 목록에 `char-count`가 나타나고 `cargo run --bin char-count`가 `79`를 출력하면 됩니다.

<details>
<summary>힌트</summary>

`src/bin/char-count.rs` 파일 하나면 됩니다. `text_stat::load(None)`으로 예시 글을 얻고 `text_stat::count(&text).chars`를 출력합니다. `main`이 `std::io::Result<()>`를 돌려주게 하면 `?`를 쓸 수 있습니다.

</details>

### 연습 2. 라이브러리 패키지에서 시작하기

`cargo new --lib temp-conv`로 패키지를 만들고, `src/lib.rs`에 섭씨를 화씨로 바꾸는 `pub fn c_to_f(c: f64) -> f64`를 쓰세요. 그다음 `Cargo.toml`을 고치지 않고 이 함수를 불러 `100°C = 212°F`를 출력하는 실행 파일을 같은 패키지에 추가해 `cargo run`으로 실행하세요.

<details>
<summary>힌트</summary>

`src/main.rs`를 만들면 패키지 이름과 같은 바이너리가 생깁니다. 바이너리가 하나뿐이므로 `cargo run`에 `--bin`이 필요 없습니다. 라이브러리 크레이트의 이름은 `temp-conv`가 아니라 `temp_conv`입니다. 화씨 = 섭씨 × 9 / 5 + 32.

</details>

### 연습 3. 잠금 파일 되살리기

사본에서 `Cargo.lock`을 지운 뒤 `cargo build --locked`와 `cargo build`를 차례로 실행하세요. 두 명령의 결과가 왜 다른지 설명하고, 새로 생긴 `Cargo.lock`을 저장소의 것과 비교하세요(`diff`). 내용이 같다면 왜 같고, 다르다면 무엇이 달라졌는지 적어 보세요.

<details>
<summary>힌트</summary>

`--locked`는 잠금 파일을 만들거나 고쳐야 하는 상황에서 실패합니다. 옵션이 없으면 Cargo가 `Cargo.toml`의 범위(`"1.13.3"`, 곧 1.13.3 이상 2.0.0 미만)에서 버전을 다시 고릅니다. 그 사이에 `unicode-segmentation`의 새 버전이 게시됐다면 `version`과 `checksum`이 달라집니다. 이것이 잠금 파일을 커밋하는 이유입니다.

</details>

### 연습 4. 빈 Cargo home 채우기

2.6절의 실험을 직접 해 보세요. `CARGO_HOME`을 빈 임시 디렉터리로 지정하고 사본에서 `cargo fetch`를 실행한 뒤, 그 디렉터리에서 `.crate` 파일과 풀린 소스의 `Cargo.toml`을 찾으세요. `.crate` 파일에 `shasum -a 256`(Linux는 `sha256sum`)을 실행한 값이 `Cargo.lock`의 `checksum`과 같은지 확인하세요.

<details>
<summary>힌트</summary>

`CARGO_HOME=/tmp/fresh-home cargo fetch`처럼 명령 앞에 변수를 붙이면 그 명령에만 적용됩니다. 파일은 `find /tmp/fresh-home -name '*.crate'`로 찾습니다. 풀린 소스는 `registry/src/` 아래 같은 이름의 디렉터리에 있습니다. 실험이 끝나면 임시 디렉터리를 지웁니다.

</details>

## 6. 참고 자료

- The Cargo Book
  - [1.1 Installation](https://doc.rust-lang.org/cargo/getting-started/installation.html), [1.2 First Steps with Cargo](https://doc.rust-lang.org/cargo/getting-started/first-steps.html)
  - [2.1 Why Cargo Exists](https://doc.rust-lang.org/cargo/guide/why-cargo-exists.html) — Cargo가 하는 네 가지 일
  - [2.2 Creating a New Package](https://doc.rust-lang.org/cargo/guide/creating-a-new-project.html), [2.3 Working on an Existing Package](https://doc.rust-lang.org/cargo/guide/working-on-an-existing-project.html)
  - [2.5 Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html) — 2.3절의 관례
  - [2.6 Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html), [FAQ: Why have Cargo.lock in version control?](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control)
  - [2.10 Cargo Home](https://doc.rust-lang.org/cargo/guide/cargo-home.html)
  - [cargo new](https://doc.rust-lang.org/cargo/commands/cargo-new.html), [cargo init](https://doc.rust-lang.org/cargo/commands/cargo-init.html), [설정의 `[cache]` 절](https://doc.rust-lang.org/cargo/reference/config.html#cache)
- 이 저장소의 관련 챕터
  - [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md) — 설치, `cargo new`·`build`·`run`·`check`
  - [Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md) 2.3절 — `cargo add`, `cargo update`
  - [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) — 패키지와 크레이트, 라이브러리와 바이너리를 함께 가진 패키지
  - [부록 A ch25](../../appd-a/ch25-dependency-graph/README.md) — 잠금 파일을 커밋하고 정해 둔 때에 올리는 지침
- 온라인 문서는 최신 stable 기준입니다. 이 챕터는 툴체인과 함께 설치된 1.100.0의 Cargo Book(`rustc --print sysroot` 아래 `share/doc/rust/html/cargo/`)으로 확인했습니다.
