# Ch1. Hello, Cargo!

Rust를 설치하고, 컴파일러 `rustc`로 첫 프로그램을 직접 컴파일한 다음, 실제 개발에서 쓰는 빌드 도구 Cargo로 같은 일을 다시 해 봅니다. 마지막으로 앞으로 모든 챕터에서 쓰게 될 `println!`의 서식 출력을 익힙니다. The Book 1장에 해당합니다.

- 패키지 이름: `p0-ch1-hello-cargo`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024). 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `rustup`, `rustc`, `cargo`가 각각 무슨 일을 하는지 설명하고, mise나 rustup으로 설치한 툴체인을 버전 명령으로 확인할 수 있다.
2. `rustc`로 소스 파일 하나를 직접 컴파일해 실행 파일을 만들고 실행할 수 있다.
3. `cargo new`로 프로젝트를 만들고, 그 안의 `Cargo.toml`, `src/main.rs`, `Cargo.lock`, `target/`이 각각 무엇인지 설명할 수 있다.
4. `cargo build`, `cargo run`, `cargo check`, `cargo build --release`를 상황에 맞게 고르고, 이 저장소의 예제를 `cargo run -p <패키지> --example <이름>`으로 실행할 수 있다.
5. `println!`의 서식 문자열로 값을 원하는 위치·모양·표기법으로 출력하고, 출력 대상(표준 출력, 표준 에러, 문자열)을 고를 수 있다.

## 핵심 개념

### 1. 툴체인: `rustup`, `rustc`, `cargo`

Rust를 설치하면 여러 도구가 한 묶음으로 들어옵니다. 이 묶음을 툴체인(toolchain)이라고 부릅니다.

| 도구 | 하는 일 |
| --- | --- |
| `rustc` | 컴파일러입니다. `.rs` 소스 파일을 실행 파일로 바꿉니다. |
| `cargo` | 빌드 도구이자 패키지 관리자입니다. 프로젝트를 만들고, `rustc`를 알맞은 옵션으로 불러 빌드하고, 실행하고, 외부 라이브러리를 내려받습니다. |
| `rustfmt` | 코드 모양을 표준 스타일로 정리합니다. 보통 `cargo fmt`로 부릅니다. |
| `clippy` | 흔한 실수와 더 나은 작성법을 알려 주는 검사 도구입니다. `cargo clippy`로 부릅니다. |
| `rustup` | 툴체인 관리자입니다. 위 도구들을 설치하고, 버전을 바꾸고, 업데이트합니다. |

실제 개발에서는 `rustc`를 직접 부를 일이 거의 없고 `cargo`를 씁니다. 그래도 `rustc`를 한 번 직접 써 보면 Cargo가 뒤에서 무엇을 대신해 주는지 알 수 있습니다.

### 2. 설치

설치 방법은 두 가지입니다. 이 저장소는 mise를 쓰지만, 어느 쪽을 택하든 툴체인을 실제로 설치하는 것은 `rustup`입니다. mise도 내부적으로 rustup을 사용합니다.

> 이미 Rust가 설치돼 있다면 아래 "설치 확인"으로 바로 넘어가도 됩니다.

#### 방법 A: mise (이 저장소의 방식)

[mise](https://mise.jdx.dev/)는 Node.js, Python, Rust 같은 여러 언어 도구의 버전을 프로젝트마다 따로 관리해 주는 도구입니다. 저장소 루트의 `mise.toml`에 이 저장소가 쓸 Rust 버전이 적혀 있습니다.

```toml
[tools]
rust = "latest"
```

`"latest"`는 최신 안정(stable) 버전을 뜻합니다. `"1.98"`처럼 쓰면 1.98.x 중 최신 버전을, `"stable"`이나 `"nightly"`처럼 쓰면 해당 채널을 고릅니다.

mise로 준비하는 순서는 다음과 같습니다(macOS/Linux, zsh 기준).

```sh
# 1. mise 설치: ~/.local/bin/mise 에 설치됩니다.
curl -fsSL https://mise.run | sh

# 2. 셸이 시작될 때 mise를 활성화하도록 설정한 뒤, 새 터미널을 엽니다.
#    bash라면 `activate bash`와 ~/.bashrc 를 씁니다.
echo 'eval "$(~/.local/bin/mise activate zsh)"' >> "${ZDOTDIR:-$HOME}/.zshrc"

# 3. 저장소 루트에서 mise.toml에 적힌 도구(여기서는 Rust)를 설치합니다.
mise install
```

mise는 Rust를 직접 내려받지 않고 rustup에게 설치를 맡깁니다(rustup이 없으면 rustup부터 설치합니다). 그다음 환경 변수 `RUSTUP_TOOLCHAIN`을 설정해서, 이 저장소 안에서는 `cargo`와 `rustc`가 `mise.toml`이 고른 버전을 쓰게 합니다. 이렇게 하면 다른 프로젝트가 다른 Rust 버전을 써도 서로 영향을 주지 않습니다.

`mise install`을 실행한 프로젝트의 `mise.toml`은 신뢰하는(trusted) 설정으로 취급됩니다. `mise.toml`은 환경 변수를 바꾸거나 명령을 실행하게 만들 수도 있으므로, 처음 보는 저장소라면 내용을 먼저 읽어 본 뒤 실행합니다.

#### 방법 B: rustup 직접 설치

mise를 쓰지 않는다면 공식 설치 도구 rustup을 바로 설치합니다.

```sh
# macOS / Linux / WSL
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

이 명령은 rustup 설치 스크립트를 내려받아 실행하고, rustup이 최신 안정 버전 Rust를 설치합니다. 성공하면 `Rust is installed now. Great!`가 출력됩니다. 도구들은 `~/.cargo/bin`에 설치되며, 이 경로를 `PATH`에 추가하는 설정은 새 터미널을 열어야 적용될 수 있습니다.

Windows에서는 <https://www.rust-lang.org/tools/install>에서 `rustup-init.exe`를 내려받아 실행합니다. 이때 Visual Studio C++ Build Tools 설치를 요구할 수 있습니다.

rustup으로 설치했다면 다음 명령도 알아 둡니다.

| 명령 | 하는 일 |
| --- | --- |
| `rustup update` | 설치된 Rust를 최신 버전으로 업데이트합니다. |
| `rustup self uninstall` | Rust와 rustup을 삭제합니다. |
| `rustup doc` | 오프라인에서도 볼 수 있는 로컬 문서를 브라우저로 엽니다. |

#### 링커

`rustc`는 컴파일의 마지막 단계에서 링커(linker)라는 프로그램을 불러 컴파일 결과를 실행 파일 하나로 합칩니다. 대부분의 시스템에는 이미 링커가 있지만, 링커를 찾을 수 없다는 에러가 나면 C 컴파일러 도구를 설치합니다.

- macOS: `xcode-select --install`
- Ubuntu: `build-essential` 패키지 (다른 배포판은 GCC나 Clang)
- Windows: Visual Studio (C++ Build Tools)

#### 설치 확인

```console
$ rustc --version
rustc 1.100.0 (0ef1bc386 2026-11-09)
$ cargo --version
cargo 1.100.0 (3acc79c0c 2026-11-05)
```

`rustc` 뒤의 숫자가 버전이고, 괄호 안은 그 버전을 만든 커밋 해시와 날짜입니다. 숫자는 설치한 시점에 따라 다릅니다.

mise로 설치했다면 mise가 어떤 툴체인을 고르고 있는지도 확인할 수 있습니다. 아래 `<저장소 경로>`는 실제로는 저장소를 내려받은 위치로 표시됩니다.

```console
$ mise ls rust
rust  1.100.0 (symlink)  <저장소 경로>/mise.toml  latest
$ rustup show active-toolchain
1.100.0-aarch64-apple-darwin (overridden by environment variable RUSTUP_TOOLCHAIN)
```

두 번째 출력의 `overridden by environment variable RUSTUP_TOOLCHAIN`은 mise가 설정한 환경 변수 때문에 1.100.0이 선택됐다는 뜻입니다. `aarch64-apple-darwin`은 Apple Silicon macOS를 가리키며, 운영체제와 CPU에 따라 다르게 나옵니다.

### 3. 첫 프로그램과 `rustc`

Rust 소스 파일의 확장자는 `.rs`입니다. 가장 작은 Rust 프로그램은 다음과 같습니다.

```rust
fn main() {
    println!("Hello, world!");
}
```

- `fn main()`은 `main`이라는 함수를 정의합니다. 실행 파일로 만들어지는 Rust 프로그램은 언제나 `main` 함수에서 시작합니다. 괄호가 비어 있으니 매개변수(parameter)가 없고, 돌려주는 값도 없습니다. 함수 본문은 반드시 중괄호 `{ }`로 감쌉니다.
- `println!`은 함수가 아니라 매크로(macro)입니다. 이름 뒤의 `!`가 그 표시입니다. 왜 함수가 아니라 매크로인지는 7절에서 설명합니다.
- 줄 끝의 `;`는 이 작업이 끝났다는 표시입니다. Rust 코드는 대부분의 줄이 세미콜론으로 끝납니다.
- 들여쓰기는 공백 4칸이 표준 스타일입니다. `rustfmt`가 이 모양으로 맞춰 줍니다.

Rust는 **컴파일과 실행이 분리된** 언어입니다. 먼저 `rustc`로 실행 파일을 만들고, 그 실행 파일을 따로 실행합니다. 아무 디렉터리(저장소 밖이 좋습니다)에 위 코드를 `main.rs`로 저장하고 다음을 실행합니다.

```console
$ rustc --edition 2024 main.rs
$ ls
main  main.rs
$ ./main
Hello, world!
```

컴파일하면 소스 파일 옆에 실행 파일 `main`이 생깁니다. Windows에서는 `main.exe`와 디버깅 정보 파일 `main.pdb`가 생기고, `.\main`으로 실행합니다.

이렇게 미리 실행 파일을 만들어 두는 방식을 AOT(ahead-of-time) 컴파일이라고 합니다. Python이나 JavaScript는 실행할 때마다 인터프리터가 소스를 읽어야 하므로 실행하는 컴퓨터에도 인터프리터가 있어야 합니다. Rust 실행 파일은 Rust가 설치되지 않은 컴퓨터(같은 운영체제)에서도 그대로 실행됩니다. 대신 코드를 고칠 때마다 다시 컴파일해야 합니다.

**`--edition 2024`를 붙이는 이유**: 에디션(edition)은 Rust 문법의 판(版)으로, 몇 년마다 새 판이 나오며 일부 문법이 바뀝니다(Part 2에서 다룹니다). `rustc --help`에는 다음과 같이 적혀 있습니다.

```text
        --edition <2015|2018|2021|2024|future>
                        Specify which edition of the compiler to use when
                        compiling code. The default is 2015 and the latest
                        stable edition is 2024.
```

즉 `rustc`는 에디션을 적지 않으면 가장 오래된 2015 에디션으로 컴파일합니다. Hello World는 어느 에디션이든 똑같이 동작하지만, 최신 문법을 쓴 코드는 에러가 날 수 있습니다. Cargo는 `Cargo.toml`의 `edition` 값을 `rustc`에 넘겨 주므로(`cargo build -v`로 보면 `--edition=2024`가 붙어 있습니다) Cargo를 쓸 때는 신경 쓰지 않아도 됩니다.

### 4. Cargo로 프로젝트 만들기

`rustc`는 파일 하나를 컴파일하기에는 충분하지만, 파일이 여러 개가 되고 외부 라이브러리를 쓰기 시작하면 옵션을 일일이 챙기기 어렵습니다. Cargo는 정해진 디렉터리 구조와 설정 파일을 기준으로 이 일을 대신합니다. 그래서 처음 보는 Rust 프로젝트라도 내려받아 `cargo build` 한 번이면 빌드할 수 있습니다.

```console
$ cargo new hello_cargo
    Creating binary (application) `hello_cargo` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
```

`cargo new`는 다음 구조를 만듭니다.

```text
hello_cargo/
├── .git/          ← git 저장소 밖에서 만들 때만 생깁니다
├── .gitignore     ← git 저장소 밖에서 만들 때만 생깁니다 (내용: /target)
├── Cargo.toml
└── src/
    └── main.rs
```

이미 git 저장소인 디렉터리 안에서 `cargo new`를 실행하면 `.git`과 `.gitignore`를 만들지 않습니다. 저장소 안에 또 다른 저장소가 생기지 않게 하려는 동작입니다. `--vcs none`을 주면 어디서 실행하든 git 관련 파일을 만들지 않습니다.

**`Cargo.toml`** 은 Cargo의 설정 파일(매니페스트)이며 TOML 형식으로 씁니다.

```toml
[package]
name = "hello_cargo"
version = "0.1.0"
edition = "2024"

[dependencies]
```

- `[package]`: 이 패키지(package)의 정보를 적는 섹션입니다. `name`은 패키지 이름(실행 파일 이름이기도 합니다), `version`은 패키지 버전, `edition`은 사용할 Rust 에디션입니다.
- `[dependencies]`: 이 프로젝트가 사용하는 외부 코드 묶음을 적는 곳입니다. Rust에서는 이런 코드 묶음을 크레이트(crate)라고 부릅니다. 지금은 비어 있고, ch2에서 `rand` 크레이트를 추가해 봅니다.

**`src/main.rs`** 에는 Cargo가 미리 만들어 둔 Hello World가 들어 있습니다. Cargo는 소스 코드가 `src/` 안에 있다고 가정하고, 최상위 디렉터리에는 `Cargo.toml`, README, 라이선스처럼 코드가 아닌 파일을 둡니다.

이 챕터의 [`projects/hello_cargo`](projects/hello_cargo)는 `cargo new --vcs none hello_cargo`로 만든 프로젝트입니다. `Cargo.toml` 끝에 빈 `[workspace]` 테이블을 한 줄 덧붙인 것 말고는 `cargo new`가 만든 그대로입니다. 그 한 줄이 왜 필요한지는 "자주 하는 실수" 4번에서 설명합니다.

### 5. `cargo build`, `cargo run`, `cargo check`, `--release`

아래 출력은 `projects/hello_cargo`에서 실행한 실제 결과입니다. 괄호 안의 경로와 걸린 시간은 환경마다 다릅니다.

**`cargo build`** 는 프로젝트를 컴파일합니다. 실행 파일은 현재 디렉터리가 아니라 `target/debug/` 아래에 생깁니다.

```console
$ cargo build
   Compiling hello_cargo v0.1.0 (/…/projects/hello_cargo)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
$ ./target/debug/hello_cargo
Hello, world!
```

**`cargo run`** 은 필요하면 빌드한 뒤 바로 실행합니다. 소스가 바뀌지 않았으면 다시 컴파일하지 않으므로 `Compiling` 줄이 없습니다. 실행 파일 경로를 외울 필요가 없어서 보통은 이 명령을 씁니다.

```console
$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/hello_cargo`
Hello, world!
```

**`cargo check`** 는 컴파일 에러가 있는지만 검사하고 실행 파일은 만들지 않습니다. 실행 파일을 만드는 마지막 단계를 건너뛰므로 `cargo build`보다 빠릅니다. 코드를 쓰는 동안에는 `cargo check`로 자주 확인하고, 실제로 실행할 때 `cargo run`이나 `cargo build`를 쓰는 것이 일반적입니다.

```console
$ cargo check
    Checking hello_cargo v0.1.0 (/…/projects/hello_cargo)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
```

**`cargo build --release`** 는 최적화를 켜고 컴파일해 `target/release/`에 실행 파일을 만듭니다.

```console
$ cargo build --release
   Compiling hello_cargo v0.1.0 (/…/projects/hello_cargo)
    Finished `release` profile [optimized] target(s) in 0.07s
$ ./target/release/hello_cargo
Hello, world!
```

`Finished` 줄의 `dev`와 `release`는 빌드 설정 묶음인 프로필(profile)의 이름입니다(Part 3에서 자세히 다룹니다). 개발 중에는 코드를 고치고 다시 빌드하는 일이 잦으므로 `dev` 프로필은 최적화를 하지 않고(`unoptimized`) 디버깅 정보를 넣어(`debuginfo`) 빨리 컴파일합니다. 완성된 프로그램을 배포하거나 성능을 잴 때는 컴파일이 더 오래 걸리더라도 빠르게 실행되는 `release` 프로필을 씁니다.

| 명령 | 하는 일 | 실행 파일 위치 |
| --- | --- | --- |
| `cargo build` | 디버그 빌드 | `target/debug/hello_cargo` |
| `cargo run` | 필요하면 빌드한 뒤 실행 | `target/debug/hello_cargo` |
| `cargo check` | 컴파일 에러만 검사 | 만들지 않음 |
| `cargo build --release` | 최적화 빌드 | `target/release/hello_cargo` |

#### `target/`과 `Cargo.lock`

첫 빌드가 끝나면 디렉터리에 두 가지가 새로 생깁니다.

```console
$ ls
Cargo.lock  Cargo.toml  src  target
```

- **`target/`** 은 빌드 결과물이 모이는 곳입니다. 소스만 있으면 언제든 다시 만들 수 있으므로 버전 관리에 넣지 않습니다. `cargo new`가 만드는 `.gitignore`의 `/target`이 바로 그 설정입니다. 이 저장소에서는 루트 `.gitignore`의 `target/` 한 줄이 모든 하위 디렉터리의 `target/`을 무시합니다. 공간을 비우고 싶으면 `cargo clean`으로 지웁니다.
- **`Cargo.lock`** 은 빌드에 쓰인 의존성의 정확한 버전을 기록합니다. `Cargo.toml`에 적은 의존성 버전은 "이 버전과 호환되는 버전"이라는 범위로 해석되지만(ch2에서 다룹니다), `Cargo.lock`에는 실제로 고른 버전 하나가 적히므로 언제 어디서 빌드해도 같은 버전을 쓰게 됩니다. Cargo가 관리하는 파일이므로 직접 고치지 않습니다. `cargo new`가 만드는 `.gitignore`에는 `Cargo.lock`이 들어 있지 않아 기본적으로 버전 관리에 포함됩니다. 지금은 의존성이 없어서 자기 자신만 적혀 있습니다.

```toml
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "hello_cargo"
version = "0.1.0"
```

### 6. 이 저장소의 예제 실행하기

이 저장소에서는 챕터마다 예제를 다음 형태의 명령으로 실행합니다.

```sh
cargo run -p p0-ch1-hello-cargo --example 01_hello_world
```

| 부분 | 뜻 |
| --- | --- |
| `cargo run` | 빌드한 뒤 실행합니다. |
| `-p p0-ch1-hello-cargo` | `--package`의 줄임입니다. 저장소에는 챕터마다 패키지가 하나씩 있으므로 어느 패키지인지 고릅니다. 패키지 이름은 챕터 `Cargo.toml`의 `name`입니다. |
| `--example 01_hello_world` | 그 패키지의 `examples/01_hello_world.rs`를 실행합니다. 확장자 `.rs`는 빼고 씁니다. |
| `-q` (선택) | `--quiet`의 줄임입니다. `Compiling`, `Running` 같은 Cargo 메시지를 숨기고 프로그램 출력만 보여 줍니다. |

이 명령이 저장소 어디에서나 통하는 이유는 저장소 루트의 `Cargo.toml`이 모든 챕터 패키지를 하나로 묶은 워크스페이스(workspace)이기 때문입니다([Part 3 ch4](../../part3-cargo/ch4-workspaces/README.md)에서 다룹니다). 워크스페이스의 빌드 결과는 저장소 루트의 `target/` 하나에 모이고, 예제 실행 파일은 `target/debug/examples/` 아래에 생깁니다.

```console
$ cargo run -p p0-ch1-hello-cargo --example 01_hello_world
   Compiling p0-ch1-hello-cargo v0.0.0 (/…/part0-getting-started/ch1-hello-cargo)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/examples/01_hello_world`
Hello, world!
안녕하세요, Rust!
```

Cargo는 `examples/` 디렉터리의 `.rs` 파일을 저절로 예제(example)로 인식합니다. 예제 파일은 각자 `main` 함수를 가진 독립된 프로그램이라서 서로 영향을 주지 않습니다. 챕터 디렉터리 안에서 실행할 때는 현재 디렉터리의 패키지가 선택되므로 `-p`를 생략해도 됩니다.

```sh
cd part0-getting-started/ch1-hello-cargo
cargo run --example 01_hello_world
```

`projects/` 아래의 독립 프로젝트는 워크스페이스에 속하지 않으므로, 그 디렉터리로 들어가서 `cargo run`을 실행합니다.

### 7. `println!`과 서식 문자열

`println!`의 첫 번째 인자(argument)는 **서식 문자열(format string)** 입니다. 서식 문자열 안의 `{}`는 자리표시자(placeholder)로, 뒤에 오는 값으로 바뀝니다.

```rust
let year = 2015;
println!("Rust 1.0 출시: {}년", year); // 순서대로
println!("{0}, {1}, {0}", "가", "나");   // 번호로 (0부터)
println!("넓이: {area}", area = 3 * 4); // 이름으로
println!("{year}년");                   // 변수 이름을 직접
```

`let`은 값에 이름을 붙이는(바인딩, binding) 키워드입니다. ch2와 ch3에서 자세히 다룹니다.

#### 왜 함수가 아니라 매크로인가

Rust의 함수는 인자의 개수와 타입이 정해져 있어야 합니다. 그런데 출력할 값의 개수는 호출할 때마다 다릅니다. 매크로는 컴파일할 때 그 자리에 알맞은 코드를 만들어 넣는 기능이라 이런 제약이 없습니다(매크로 작성법은 ch19에서 다룹니다).

더 중요한 이점은 **서식 문자열을 컴파일 시점에 검사한다**는 것입니다. `{}`가 두 개인데 값이 하나뿐이거나, 출력할 수 없는 값을 넘기면 실행해 보기 전에 컴파일 에러가 납니다. C의 `printf`는 이런 실수를 해도 컴파일되고 실행 중에 엉뚱한 값을 출력하지만, Rust에서는 그런 프로그램이 아예 만들어지지 않습니다. "자주 하는 실수" 2, 3번에서 실제 에러를 확인합니다.

#### 서식 지정

자리표시자 안의 `:` 뒤에 서식 지정(format spec)을 붙여 출력 모양을 정합니다. 쓰는 것만 다음 순서로 적습니다.

```text
{인자:[채움][정렬][부호][#][0][너비][.정밀도][형식]}
```

| 서식 | 값 | 결과 | 뜻 |
| --- | --- | --- | --- |
| `{:6}` | `42` | `    42` | 최소 6칸. 숫자는 기본 오른쪽 정렬 |
| `{:6}` | `"ab"` | `ab    ` | 문자열은 기본 왼쪽 정렬 |
| `{:<6}` `{:^6}` `{:>6}` | `"ab"` | `ab    ` / `  ab  ` / `    ab` | 왼쪽 / 가운데 / 오른쪽 정렬 |
| `{:*^10}` | `"ab"` | `****ab****` | 빈칸을 `*`로 채움 |
| `{:05}` | `-42` | `-0042` | 숫자 앞을 0으로 채움 (부호가 먼저) |
| `{:+}` | `42` | `+42` | 양수에도 부호 표시 |
| `{:.2}` | `36.567` | `36.57` | 소수점 아래 2자리 (반올림) |
| `{:8.1}` | `36.567` | `    36.6` | 너비 8, 소수점 아래 1자리 |
| `{:>width$}` | `"ab"` (`width = 7`) | `     ab` | 너비를 변수로 지정 |

맨 끝의 형식 문자는 값을 **어떤 표기법으로** 보여 줄지 정합니다.

| 형식 | 이름 | 결과 예 |
| --- | --- | --- |
| (없음) | 기본 형식 (`Display`) | `255` → `255` |
| `?` | 디버그 형식 (`Debug`) | 문자열 `ab` → `"ab"` (따옴표가 붙음), 실수 `3.0` → `3.0` |
| `b` / `o` | 2진수 / 8진수 | `255` → `11111111` / `377` |
| `x` / `X` | 16진수 소문자 / 대문자 | `255` → `ff` / `FF` |
| `e` / `E` | 지수 표기 | `1234.5` → `1.2345e3` |
| `#` 붙이기 | 대체 형식 (진법 접두사) | `{:#x}` → `0xff`, `{:#b}` → `0b11111111` |

`{}`는 사람이 읽기 좋은 기본 형식, `{:?}`는 개발자가 값의 정확한 모양을 확인하기 위한 디버그 형식입니다. 이 둘은 각각 `Display`와 `Debug`라는 트레이트(trait)로 정해지는데, 트레이트는 ch10에서 다룹니다. 지금은 "배열처럼 `{}`로는 출력되지 않는 값도 `{:?}`로는 출력된다" 정도만 기억하면 됩니다. 직접 만든 타입을 `{:?}`로 출력하는 방법은 ch5에서 배웁니다.

#### 출력 매크로 가족

서식 문법은 모두 같고, 결과를 보내는 곳만 다릅니다.

| 매크로 | 보내는 곳 | 줄바꿈 |
| --- | --- | --- |
| `print!` | 표준 출력(stdout) | 붙이지 않음 |
| `println!` | 표준 출력(stdout) | 끝에 붙임 |
| `eprint!` | 표준 에러(stderr) | 붙이지 않음 |
| `eprintln!` | 표준 에러(stderr) | 끝에 붙임 |
| `format!` | 출력하지 않고 문자열(`String`)로 돌려줌 | - |

표준 출력과 표준 에러는 터미널에서는 똑같이 보이지만, 셸의 리다이렉션(`>`, `2>`)이나 파이프(`|`)로 나누면 서로 다른 곳으로 갑니다. 프로그램의 결과는 표준 출력으로, 진행 상황이나 에러 메시지는 표준 에러로 보내면 결과만 파일에 저장하거나 다른 프로그램에 넘길 수 있습니다.

## 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_hello_world.rs`](examples/01_hello_world.rs) | `main` 함수와 `println!`, `rustc` 직접 컴파일 | 2, 4 |
| [`examples/02_placeholders.rs`](examples/02_placeholders.rs) | 자리표시자에 값을 넣는 네 가지 방법, 중괄호 출력 | 5 |
| [`examples/03_width_and_precision.rs`](examples/03_width_and_precision.rs) | 너비·정렬·채움·정밀도·부호, 표 만들기, 한글 너비 주의점 | 5 |
| [`examples/04_number_formats.rs`](examples/04_number_formats.rs) | 진법·지수 표기, `#` 대체 형식, `{:?}` 디버그 형식 | 5 |
| [`examples/05_print_family.rs`](examples/05_print_family.rs) | `print!`/`eprintln!`/`format!`, 표준 출력과 표준 에러 | 5 |
| [`projects/hello_cargo`](projects/hello_cargo) | `cargo new`가 만드는 프로젝트, build/run/check/release | 3, 4 |
| [`compile_fail/`](compile_fail) | 서식 출력에서 자주 하는 컴파일 에러 | 5 |

학습 목표 1(설치 확인)은 위 "설치 확인"의 명령을 직접 실행해 보는 것으로 확인합니다.

### 01_hello_world — 가장 작은 프로그램

```sh
cargo run -q -p p0-ch1-hello-cargo --example 01_hello_world
```

```text
Hello, world!
안녕하세요, Rust!
```

Rust의 문자열은 UTF-8이므로 한글도 그대로 출력됩니다(문자열은 ch8에서 자세히 다룹니다).

같은 파일을 Cargo 없이 `rustc`로 직접 컴파일할 수도 있습니다. `rustc`는 기본적으로 현재 디렉터리에 실행 파일을 만들므로, 저장소에 실행 파일이 섞이지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다. 실행 파일 이름은 소스 파일 이름에서 확장자를 뺀 것입니다.

```console
$ cd part0-getting-started/ch1-hello-cargo
$ rustc --edition 2024 examples/01_hello_world.rs --out-dir /tmp/rust-ch1
$ /tmp/rust-ch1/01_hello_world
Hello, world!
안녕하세요, Rust!
```

### 02_placeholders — 자리표시자와 인자

```sh
cargo run -q -p p0-ch1-hello-cargo --example 02_placeholders
```

```text
Rust 1.0 출시일: 2015년 5월 15일
5 + 15 = 20
철수가 영희에게 인사하고, 영희가 철수에게 답합니다.
가로 3, 세로 4인 사각형의 넓이는 12입니다.
2015년 5월 15일
2015년의 다음 해는 2016년입니다.
자리표시자는 {} 모양이고, 값을 감싸면 {15}가 됩니다.

바로 위에 빈 줄이 있습니다.
```

볼 점:

- `{year}`처럼 변수 이름을 직접 쓰는 방식이 가장 간결하지만, `{}` 안에는 변수 이름만 쓸 수 있습니다. `year + 1` 같은 계산식은 인자로 넘깁니다.
- 이름 인자(`w = 3`)는 위치 인자보다 뒤에 와야 합니다. 순서를 바꾸면 `positional arguments cannot follow named arguments` 에러가 납니다.
- 중괄호 문자 자체는 `{{`, `}}`처럼 두 번 써서 출력합니다.

### 03_width_and_precision — 너비, 정렬, 정밀도

```sh
cargo run -q -p p0-ch1-hello-cargo --example 03_width_and_precision
```

```text
[    42] [ab    ]
[42]
[ab    ] [  ab  ] [    ab]
[****ab****] [----ab]
[00042] [-0042]
[+42] [-42]
36.567 → 36.6 → 36.57 → 36.56700
[    36.6] [36.6    ]
[     ab]
item    |  price
--------+-------
apple   |   1200
banana  |    350
total   |   1550
[apple ]
[사과    ]
```

볼 점:

- 너비는 **최소** 칸 수입니다. 값이 더 길면 잘리지 않습니다(`[42]`).
- 정밀도는 자릿수보다 길면 반올림하고, 짧으면 0을 덧붙입니다(`36.56700`).
- 마지막 두 줄은 한글 너비 문제입니다. 너비는 화면의 칸 수가 아니라 글자 수로 세므로, 한글 한 글자가 영문 두 글자 폭을 차지하는 터미널에서는 `[사과    ]`가 `[apple ]`보다 길게 보입니다. 한글이 섞인 표를 서식 너비만으로 정확히 맞추기는 어렵습니다.

### 04_number_formats — 표기법 바꾸기

```sh
cargo run -q -p p0-ch1-hello-cargo --example 04_number_formats
```

```text
10진수: 255
2진수: 11111111
8진수: 377
16진수: ff / FF
0b11111111 0o377 0xff
00000101
0b00000101
1.2345e3 1.2E-4
3 vs 3.0
[그가 "안녕"이라고 했다 ]
["그가 \"안녕\"이라고 했다 "]
[90, 85, 77]
```

볼 점:

- 형식 문자는 값을 바꾸지 않고 보여 주는 방법만 바꿉니다. `255`, `11111111`, `ff`는 모두 같은 값입니다.
- `{:#010b}`에서 접두사 `0b`도 너비 10칸에 포함됩니다.
- 실수 `3.0`은 `{}`로는 `3`, `{:?}`로는 `3.0`입니다. 문자열을 `{:?}`로 찍으면 따옴표가 붙어서 끝의 공백까지 눈에 보입니다.
- 배열 `[90, 85, 77]`은 `{:?}`로만 출력할 수 있습니다(배열은 ch3에서 다룹니다).

### 05_print_family — 출력 대상 고르기

```sh
cargo run -q -p p0-ch1-hello-cargo --example 05_print_family
```

```text
하나, 둘, 셋
[알림] 이 줄은 표준 에러(stderr)로 갑니다.
이 줄은 표준 출력(stdout)으로 갑니다.
만든 문자열: [7 × 6 = 42]
[   7 × 6 = 42   ]
```

터미널에서는 다섯 줄이 모두 보이지만, 표준 에러를 버리면(`2>/dev/null`) `[알림]` 줄이 사라지고, 표준 출력을 버리면(`1>/dev/null`) `[알림]` 줄만 남습니다.

```console
$ cargo run -q -p p0-ch1-hello-cargo --example 05_print_family 2>/dev/null
하나, 둘, 셋
이 줄은 표준 출력(stdout)으로 갑니다.
만든 문자열: [7 × 6 = 42]
[   7 × 6 = 42   ]
$ cargo run -q -p p0-ch1-hello-cargo --example 05_print_family 1>/dev/null
[알림] 이 줄은 표준 에러(stderr)로 갑니다.
```

### projects/hello_cargo — Cargo 프로젝트

워크스페이스와 분리된 독립 프로젝트이므로 그 디렉터리에서 실행합니다. 출력은 "핵심 개념" 5절에 실었습니다.

```sh
cd part0-getting-started/ch1-hello-cargo/projects/hello_cargo
cargo build
cargo run
cargo check
cargo build --release
```

## 자주 하는 실수

### 1. 매크로 이름 뒤의 `!`를 빠뜨림 (E0423)

[`compile_fail/e0423_println_without_bang.rs`](compile_fail/e0423_println_without_bang.rs)

```console
$ rustc --edition 2024 compile_fail/e0423_println_without_bang.rs --out-dir /tmp/rust-ch1
error[E0423]: cannot find function `println` in this scope
  --> compile_fail/e0423_println_without_bang.rs:14:5
   |
14 |     println("Hello, world!");
   |     ^^^^^^^ not found in this scope
   |
   = note: a macro named `println` exists in another namespace
help: use `!` to invoke the macro
   |
14 |     println!("Hello, world!");
   |            +
```

- **읽는 법**: 첫 줄의 `error[E0423]`이 에러 코드, 그 뒤가 요약입니다. "이 스코프에서 `println`이라는 함수를 찾을 수 없다"는 뜻이고, 그 아래 note가 같은 이름의 매크로는 있다고 알려 줍니다. `-->`는 파일:줄:칸 위치이고, `help:` 아래에 고친 코드가 `+` 표시와 함께 나옵니다.
- **원인**: `!`가 없으면 컴파일러는 `println`이라는 **함수**를 찾습니다. 그런 함수는 없고 같은 이름의 매크로만 있습니다.
- **해결**: `println!(...)`처럼 `!`를 붙입니다. 에러 코드가 있는 에러는 `rustc --explain E0423`으로 자세한 설명을 볼 수 있습니다.

### 2. `{}` 개수와 인자 개수가 다름 (에러 코드 없음)

[`compile_fail/format_args_mismatch.rs`](compile_fail/format_args_mismatch.rs)

```console
$ rustc --edition 2024 compile_fail/format_args_mismatch.rs --out-dir /tmp/rust-ch1
error: 2 positional arguments in format string, but there is 1 argument
  --> compile_fail/format_args_mismatch.rs:15:15
   |
15 |     println!("{}님, {}번째 방문을 환영합니다!", name);
   |               ^^    ^^                          ----
```

- **읽는 법**: "서식 문자열에는 위치 인자 자리가 2개인데 인자는 1개"라는 뜻입니다. `^^`가 자리표시자 두 개를, `----`가 넘긴 인자 하나를 가리킵니다. 서식 문자열 검사 에러에는 `E` 코드가 붙지 않습니다.
- **원인**: `println!`은 컴파일할 때 서식 문자열의 `{}`를 세어 인자 개수와 비교합니다.
- **해결**: 빠진 인자를 채우거나(`name, 3`) 자리표시자를 줄입니다. 반대로 인자가 남아도 에러입니다. `println!("{}", a, b);`는 `error: argument never used`로 거부됩니다.

### 3. 배열 같은 값을 `{}`로 출력함 (E0277)

다음 코드를 `arr.rs`의 `main` 함수 안에 넣고 컴파일하면 에러가 납니다.

```rust
let scores = [90, 85, 77];
println!("{scores}");
```

```text
error[E0277]: `[{integer}; 3]` doesn't implement `std::fmt::Display`
 --> arr.rs:3:15
  |
3 |     println!("{scores}");
  |               ^^^^^^^^ `[{integer}; 3]` cannot be formatted with the default formatter
  |
  = help: the trait `std::fmt::Display` is not implemented for `[{integer}; 3]`
  = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
```

- **원인**: `{}`는 사람이 읽기 좋은 기본 형식(`Display`)을 쓰는데, 배열은 "사람이 읽기 좋은 모양"이 하나로 정해져 있지 않아서 기본 형식이 없습니다. `[{integer}; 3]`은 "정수 3개짜리 배열"이라는 타입 표기입니다(ch3에서 다룹니다).
- **해결**: 컴파일러의 `note`대로 `{:?}`(디버그 형식)를 씁니다. `{:#?}`는 여러 줄로 보기 좋게 펼쳐 출력합니다.

### 4. 이 저장소 안에서 `cargo new`를 실행함

Cargo는 `cargo new`를 실행한 위치의 상위 디렉터리에서 워크스페이스 설정(이 저장소에서는 루트 `Cargo.toml`)을 찾으면, 새 프로젝트를 그 워크스페이스와 엮으려고 합니다. 그래서 이 저장소 안에서 실행하면 위치에 따라 두 가지 문제가 생깁니다.

**(가) 저장소 루트나 `part0-getting-started/`처럼 챕터 디렉터리 밖에서 실행한 경우**: Cargo가 루트 `Cargo.toml`의 `members` 목록에 새 프로젝트를 **자동으로 추가합니다**. 에러 없이 끝나기 때문에 루트 설정 파일이 바뀐 것을 놓치기 쉽습니다.

```text
    Creating binary (application) `hello_cargo` package
      Adding `hello_cargo` as member of workspace at `<저장소 경로>`
```

**(나) 챕터 디렉터리 안(예: `ch1-hello-cargo/` 아래)에서 실행한 경우**: 프로젝트는 만들어지지만 경고가 나오고, 그 프로젝트에서 `cargo build`를 실행하면 다음 에러로 빌드되지 않습니다.

```text
error: current package believes it's in a workspace when it's not:
...
Alternatively, to keep it out of the workspace, add the package to the `workspace.exclude` array, or add an empty `[workspace]` table to the package's manifest.
```

워크스페이스 안에 있으니 그 일부라고 판단했는데, 루트의 멤버 목록(`part*/ch*`)에는 그 경로가 없어서 "속한다고 생각하지만 멤버가 아니다"라는 모순이 생긴 것입니다.

- **해결**: 연습용 프로젝트는 저장소 밖(예: `~/projects`)에서 만드는 것이 가장 간단합니다. 저장소 안에 두어야 한다면 에러 메시지의 안내대로 새 프로젝트의 `Cargo.toml`에 빈 `[workspace]` 테이블을 추가해 독립 프로젝트로 만듭니다. `projects/hello_cargo`가 바로 그렇게 만들어져 있습니다.
- (가)처럼 루트 `Cargo.toml`이 바뀌었다면 `members`에 추가된 항목을 지워 원래대로 `members = ["part*/ch*"]`로 되돌립니다.

### 5. 설치했는데 `rustc`를 찾을 수 없음 (`command not found`)

- **원인**: 대부분 `PATH` 설정이 아직 적용되지 않은 경우입니다. rustup은 `~/.cargo/bin`을 `PATH`에 추가하도록 셸 설정 파일을 고치지만, 이미 열려 있던 터미널에는 반영되지 않습니다. mise를 쓴다면 셸 설정에 `mise activate` 줄이 빠졌을 수 있습니다.
- **해결**: 새 터미널을 열고 `echo $PATH`로 경로를 확인합니다. mise라면 활성화 설정을 확인하고, 활성화 없이 확인하려면 `mise exec -- rustc --version`을 실행해 봅니다.

### 6. `rustc`로 직접 컴파일할 때 `--edition`을 빠뜨림

`rustc`는 에디션을 적지 않으면 2015 에디션을 씁니다. 2018 에디션부터 생긴 문법을 쓴 파일을 그냥 `rustc file.rs`로 컴파일하면 이런 에러가 납니다.

```text
error[E0670]: `async fn` is not permitted in Rust 2015
...
  = help: pass `--edition 2024` to `rustc`
```

이 저장소의 파일을 `rustc`로 직접 컴파일할 때는 항상 `--edition 2024`를 붙입니다. Cargo를 쓰면 `Cargo.toml`의 `edition`이 자동으로 적용됩니다.

## 연습 문제

### 1. 나만의 Cargo 프로젝트

저장소 밖의 디렉터리에서 `cargo new intro`로 프로젝트를 만들고, `src/main.rs`를 고쳐 이름·나이·좋아하는 언어를 한 줄씩 출력하세요. 값은 `let`으로 이름을 붙인 뒤 `{변수이름}` 형태로 출력합니다. 그다음 `cargo check`, `cargo run`, `cargo build --release`를 차례로 실행하고, `target/` 아래에 어떤 디렉터리가 생겼는지 확인하세요.

<details>
<summary>힌트</summary>

- `let name = "페리스";`처럼 쓴 뒤 `println!("이름: {name}");`으로 출력합니다.
- 저장소 밖에서 만들면 `.git`과 `.gitignore`도 함께 생깁니다. `cat .gitignore`로 내용을 확인해 보세요.
- `cargo build --release`까지 실행하면 `target/` 아래에 `debug`와 `release`가 모두 생깁니다.

</details>

### 2. 영수증 출력

상품 세 개의 이름과 가격(소수점이 있는 실수)을 출력하는 영수증을 만드세요. 상품 이름은 10칸 왼쪽 정렬, 가격은 10칸 오른쪽 정렬에 소수점 아래 2자리로 맞추고, 머리글 아래와 합계 위에 `=`로 채운 구분선을 넣습니다. 합계는 세 가격을 더한 값을 인자로 넘겨 출력합니다.

```text
item           price
====================
coffee          4.50
bagel           3.25
salad          12.00
====================
total          19.75
```

<details>
<summary>힌트</summary>

- 이름은 `{:<10}`, 가격은 `{:>10.2}`처럼 정렬·너비·정밀도를 함께 씁니다.
- 구분선은 빈 문자열에 채움 문자를 쓰면 됩니다: `println!("{:=<20}", "");`
- 합계처럼 계산식은 `{}` 안에 쓸 수 없으므로 `println!("{:<10}{:>10.2}", "total", a + b + c);`처럼 인자로 넘깁니다.
- 상품 이름을 한글로 쓰면 열이 어긋나 보입니다. 이유는 예제 03의 마지막 부분을 참고하세요.

</details>

### 3. 진법 변환표

정수 하나(예: `42`)를 10진수, 2진수, 8진수, 16진수로 한 줄씩 출력하세요. 2진수는 접두사 `0b`를 포함해 10칸에 맞추고 빈자리를 0으로 채워 `0b00101010`처럼 출력하고, 16진수는 대문자로 `0x2A`처럼 출력합니다.

<details>
<summary>힌트</summary>

- 2진수: `#`(접두사)와 `0`(0 채움)과 너비 `10`을 함께 씁니다. 순서는 `{:#010b}`입니다.
- 16진수 대문자는 `X`입니다. `{:#X}`를 쓰면 접두사는 `0x`(소문자)로 붙습니다. 그 결과를 직접 확인해 보세요.

</details>

### 4. 컴파일러 메시지 읽기

`projects/hello_cargo`의 `src/main.rs`를 복사한 프로젝트(또는 연습 문제 1의 프로젝트)에서 다음을 하나씩 해 보고, 매번 `cargo check`로 컴파일러가 무엇이라고 하는지 읽어 보세요. 확인한 뒤에는 원래대로 되돌립니다.

1. `println!`의 `!`를 지운다.
2. `println!("Hello")` 줄을 하나 더 추가하되, 첫 줄의 세미콜론을 지운다.
3. 마지막 `}`를 지운다.
4. `main` 안에 줄이 하나뿐일 때 그 줄의 세미콜론을 지운다.

<details>
<summary>힌트</summary>

- 1번은 "자주 하는 실수" 1번과 같은 에러(E0423)입니다.
- 2번은 ``expected `;`, found `println` ``, 3번은 `this file contains an unclosed delimiter` 에러가 납니다. 각 에러의 `help:`나 화살표가 가리키는 위치를 확인하세요.
- 4번은 에러가 나지 않고 정상 실행됩니다. 함수의 마지막 줄은 세미콜론이 없으면 그 함수가 돌려주는 값이 되는데, 여기서는 그 값이 "아무 값도 없음"이라서 `main`과 맞기 때문입니다. 자세한 이유는 ch3의 구문(statement)과 표현식(expression)에서 배웁니다.

</details>

## 참고 자료

The Book (기준 문서)

- [1. Getting Started](https://doc.rust-lang.org/book/ch01-00-getting-started.html)
  - [1.1 Installation](https://doc.rust-lang.org/book/ch01-01-installation.html)
  - [1.2 Hello, World!](https://doc.rust-lang.org/book/ch01-02-hello-world.html)
  - [1.3 Hello, Cargo!](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html)

Rust By Example (대응 절)

- [Hello World](https://doc.rust-lang.org/rust-by-example/hello.html): 이 챕터의 예제 01과 같은 내용입니다. 끝의 Activity에서 `println!`을 한 줄 더 추가해 봅니다.
- [Formatted print](https://doc.rust-lang.org/rust-by-example/hello/print.html): 예제 02~05의 내용입니다. 위치·이름 인자, 너비와 채움을 다루고, Activity에 정밀도로 `Pi is roughly 3.142`를 출력하는 과제가 있습니다.
- [Formatting](https://doc.rust-lang.org/rust-by-example/hello/print/fmt.html): `{:X}`, `{:o}` 같은 형식 문자를 다룹니다. 이 절의 예제는 구조체와 트레이트 구현을 쓰므로 ch5 이후에 다시 보면 좋습니다.

Rustlings (대응 연습)

- [`exercises/00_intro`](https://github.com/rust-lang/rustlings/tree/main/exercises/00_intro): `intro1.rs`(Rustlings 사용법 안내)와 `intro2.rs`(`println!` 매크로 이름의 오타 고치기)가 이 챕터에 해당합니다. 설치와 시작 방법은 [Rustlings 설치 안내](https://rustlings.rust-lang.org/setup/)를 따릅니다(`cargo install rustlings` → `rustlings init` → `cd rustlings/` → `rustlings`).

설치 도구와 API 문서

- [Rust 설치 안내 (rust-lang.org)](https://www.rust-lang.org/tools/install)
- [mise 시작하기](https://mise.jdx.dev/getting-started.html), [mise의 Rust 지원](https://mise.jdx.dev/lang/rust.html)
- [`std::fmt` 모듈 문서](https://doc.rust-lang.org/std/fmt/): 서식 문자열 문법 전체와 형식 문자 목록이 있습니다.
- [Cargo FAQ](https://doc.rust-lang.org/cargo/faq.html): `Cargo.lock`을 버전 관리에 넣을지에 대한 설명이 있습니다.
