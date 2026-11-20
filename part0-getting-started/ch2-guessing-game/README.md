# ch2. 숫자 맞히기 게임

컴퓨터가 1부터 100 사이의 수를 하나 정하면, 사용자가 맞힐 때까지 추측하는 게임을 네 단계에 걸쳐 만듭니다. 입력 받기, 변수, 외부 크레이트, `match`, 반복을 한 번씩 써 보는 맛보기 챕터입니다. 각 개념의 자세한 규칙은 뒤 챕터에서 다루므로, 여기서는 "이렇게 쓰면 이렇게 동작한다"를 손으로 익히는 데 집중합니다.

- 패키지 이름: `p0-ch2-guessing-game`
- 기준 문서: The Book 2장 "Programming a Guessing Game"
- 사용 크레이트: `rand` 0.10.4 (예제 전용이라 `[dev-dependencies]`에 있습니다)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 표준 입력에서 한 줄을 읽어 `String` 변수에 담는 코드를 작성하고, 그 변수에 `mut`가 필요한 이유를 설명할 수 있다.
2. `cargo add`로 외부 크레이트 `rand`를 추가해 정해진 범위의 난수를 만들고, `Cargo.toml`과 `Cargo.lock`이 각각 무엇을 기록하는지 설명할 수 있다.
3. `trim`과 `parse`로 문자열을 숫자로 바꾸고, `match`로 비교 결과에 따라 다른 코드를 실행할 수 있다.
4. `loop`, `break`, `continue`와 `match`로 잘못된 입력을 견디는 반복 프로그램을 작성할 수 있다.

| 학습 목표 | 예제 | 자주 하는 실수 |
| --- | --- | --- |
| 1. 입력 받기와 `mut` | [`01_read_input.rs`](examples/01_read_input.rs) | [`e0596_read_line_without_mut.rs`](compile_fail/e0596_read_line_without_mut.rs) |
| 2. 외부 크레이트로 난수 만들기 | [`02_secret_number.rs`](examples/02_secret_number.rs), [`Cargo.toml`](Cargo.toml), 2.3절의 직접 해 보기 | 4.4절 |
| 3. 숫자로 바꿔 비교하기 | [`03_compare.rs`](examples/03_compare.rs) | [`e0308_compare_string_number.rs`](compile_fail/e0308_compare_string_number.rs), [`e0282_parse_without_type.rs`](compile_fail/e0282_parse_without_type.rs) |
| 4. 반복과 잘못된 입력 처리 | [`04_complete_game.rs`](examples/04_complete_game.rs) | 5절 연습 3 |

## 2. 핵심 개념

### 2.1 변수는 기본이 불변: `let`과 `let mut`

`let`으로 만든 변수는 기본적으로 불변(immutable)입니다. 값을 바꾸려면 `mut`를 붙여 가변(mutable)으로 선언해야 합니다.

```rust
let apples = 5;      // 불변: 이후에 apples = 6; 같은 대입을 할 수 없습니다
let mut bananas = 5; // 가변: bananas += 1; 처럼 바꿀 수 있습니다
```

불변이 기본인 이유는 코드를 읽을 때 "이 값은 어디선가 바뀌지 않을까?"를 걱정하지 않게 하기 위해서입니다. 바뀌는 값에만 `mut`가 붙어 있으므로, `mut`가 없는 변수는 선언한 곳만 보면 됩니다. 바꾸지 않겠다고 해 놓고 바꾸려 하면 컴파일러가 막아 줍니다. 변수와 가변성은 `ch3-common-concepts`에서 자세히 다룹니다.

게임에서 입력을 담을 변수는 처음에 비어 있다가 입력으로 채워져야 하므로 가변이어야 합니다.

```rust
let mut guess = String::new();
```

`String`은 길이가 늘어날 수 있는 UTF-8 문자열 타입이고, `String::new()`는 빈 `String`을 만듭니다. `::`는 `new`가 `String` 타입에 딸린 함수, 즉 연관 함수(associated function)라는 뜻입니다. 연관 함수는 `ch5-structs`에서 다룹니다.

### 2.2 표준 입력 읽기: `read_line`과 `expect`

```rust
use std::io;

io::stdin()
    .read_line(&mut guess)
    .expect("입력을 읽지 못했습니다");
```

- `use std::io;`: `io`는 표준 라이브러리(`std`)의 입출력 모듈입니다. Rust는 모든 프로그램에 자주 쓰는 항목 일부(prelude)를 자동으로 가져오는데, `io`는 거기에 없으므로 `use`로 직접 가져옵니다.
- `io::stdin()`: 표준 입력(터미널 입력이나 파이프로 들어오는 입력)을 다루는 핸들을 돌려줍니다.
- `.read_line(&mut guess)`: 한 줄을 읽어 `guess` **뒤에 덧붙입니다**. 기존 내용을 지우지 않는다는 점이 중요합니다(5절 연습 3).
- `&mut guess`: `guess`를 잠시 빌려주되 고쳐 써도 된다는 뜻의 가변 참조(reference)입니다. 불변 변수에서는 가변 참조를 만들 수 없으므로 `guess`에 `mut`가 필요합니다. 참조와 대여(borrow)는 `ch4-ownership`에서 다룹니다.

`read_line`은 입력을 읽다가 실패할 수도 있으므로 결과를 `Result`라는 값으로 돌려줍니다. `Result`는 성공(`Ok`)과 실패(`Err`) 중 하나를 담는 타입입니다(자세한 내용은 `ch9-error-handling`에서 다룹니다). `expect`는 `Ok`이면 안에 든 값을 꺼내 주고, `Err`이면 전달한 메시지를 출력하며 프로그램을 멈춥니다. 이렇게 멈추는 것을 패닉(panic)이라고 합니다. `read_line`이 성공했을 때 꺼내 주는 값은 읽은 바이트 수입니다.

`expect`를 빼고 결과를 그냥 버리면 컴파일은 되지만 경고가 납니다. 실패할 수 있는 작업의 결과를 처리하지 않았다는 뜻입니다. 아래는 `.expect(...)`를 뺀 코드를 `unused.rs`라는 파일로 저장해 `rustc`로 컴파일한 결과의 앞부분입니다.

```
warning: unused `Result` that must be used
 --> unused.rs:5:5
  |
5 |     io::stdin().read_line(&mut guess);
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: this `Result` may be an `Err` variant, which should be handled
```

읽어 온 문자열에는 사용자가 누른 Enter, 즉 줄바꿈 문자 `\n`까지 들어 있습니다. 눈에 보이지 않으므로 `{guess:?}`처럼 디버그 형식으로 출력해 확인합니다. 디버그 형식은 문자열을 따옴표로 감싸고 특수 문자를 `\n`처럼 보여 줍니다.

```rust
println!("디버그 형식: {guess:?}"); // 디버그 형식: "42\n"
```

이 줄바꿈 문자 때문에 뒤에서 숫자로 바꾸기 전에 `trim`이 필요합니다(2.5절).

### 2.3 외부 크레이트 추가: `cargo add`, `Cargo.toml`, `Cargo.lock`

Rust 표준 라이브러리에는 난수 생성 기능이 없습니다. 대신 Rust 팀이 관리하는 `rand` 크레이트(crate)를 가져다 씁니다. 크레이트는 Rust의 컴파일 단위로, 실행 파일이 되는 바이너리 크레이트와 다른 프로그램에서 가져다 쓰는 라이브러리 크레이트가 있습니다. `rand`는 라이브러리 크레이트이고, 공개 크레이트는 [crates.io](https://crates.io/crates/rand)에 모여 있습니다.

**`cargo add`로 의존성 추가하기**

`cargo add 크레이트이름`은 crates.io에서 최신 버전을 찾아 `Cargo.toml`의 `[dependencies]`에 적어 줍니다. 같은 줄을 `Cargo.toml`에 직접 적어도 됩니다.

```toml
[dependencies]
rand = "0.10.4"
```

`"0.10.4"`는 정확히 0.10.4만 쓰라는 뜻이 아니라 "0.10.4 이상, 0.11.0 미만"이라는 버전 요구 사항입니다(`^0.10.4`의 줄임). Cargo는 시맨틱 버저닝(SemVer)을 따르며, 0.x 버전에서는 두 번째 숫자가 바뀌면 API가 호환되지 않을 수 있다고 봅니다. 그래서 0.10.x 안에서만 새 버전을 고릅니다.

이 저장소에서는 `rand`를 예제에서만 쓰므로 `[dev-dependencies]`에 넣었습니다. 예제(`examples/`)와 테스트에서만 쓰는 의존성을 적는 곳입니다. 이미 추가해 두었으므로 따로 실행할 필요는 없고, 추가할 때 쓴 명령은 다음과 같습니다.

```sh
cargo add --dev rand -p p0-ch2-guessing-game
```

그 결과 이 챕터의 [`Cargo.toml`](Cargo.toml)은 이렇게 되었습니다.

```toml
[dev-dependencies]
rand = "0.10.4"
```

**`Cargo.lock`이 하는 일**

`Cargo.toml`은 "0.10.4 이상 0.11 미만"이라는 범위만 적습니다. 실제로 어느 버전을 골랐는지는 `Cargo.lock`에 기록됩니다. `rand`가 내부에서 쓰는 크레이트(`rand_core`, `getrandom` 등)의 정확한 버전도 모두 함께 적힙니다. 한 번 기록되면 Cargo는 다시 버전을 고르지 않고 `Cargo.lock`의 버전을 그대로 씁니다. 그래서 몇 달 뒤에 빌드하거나 다른 사람이 빌드해도 같은 버전으로 빌드됩니다(재현 가능한 빌드). 실행 파일을 만드는 프로젝트는 보통 `Cargo.lock`을 버전 관리에 함께 넣습니다.

`Cargo.lock`은 `cargo add`를 쓰면 그 자리에서, `Cargo.toml`을 직접 고쳤다면 처음 빌드할 때 만들어집니다. 이 저장소처럼 여러 패키지를 묶은 워크스페이스에서는 저장소 루트의 `Cargo.lock` 하나를 모든 챕터가 함께 씁니다.

**`cargo update`로 새 버전 받기**

`Cargo.lock`에 고정된 버전을 올리고 싶을 때는 `cargo update`를 실행합니다. `Cargo.toml`의 요구 사항 범위 안에서 가장 새 버전을 다시 골라 `Cargo.lock`을 갱신합니다. 범위를 벗어나는 버전(예: 0.11.0)은 고르지 않으므로, 그런 버전을 쓰려면 `Cargo.toml`의 요구 사항을 직접 바꿔야 합니다.

**`cargo doc --open`으로 크레이트 문서 보기**

`cargo doc`은 프로젝트와 의존성의 API 문서를 로컬에서 HTML로 만들고, `--open`을 붙이면 브라우저로 엽니다. `rand`에 어떤 함수와 메서드가 있는지 인터넷 없이 찾아볼 수 있습니다.

- 직접 만든 프로젝트에서는 `cargo doc --open`을 실행하고, 왼쪽 사이드바의 크레이트 목록에서 `rand`를 고릅니다.
- 이 저장소에서는 `rand`가 dev-dependency이므로 예제 문서를 만들 때 함께 만들어집니다. `cargo doc -p p0-ch2-guessing-game --examples --open`을 실행하고 사이드바에서 `rand`를 고릅니다. 생성된 문서는 `target/doc/rand/index.html`에 있습니다. 참고로 이 저장소에서 `cargo doc -p rand`는 동작하지 않습니다. cargo 1.98.1에서는 cargo 내부 오류(panic)로 끝났고, 1.100.0에서는 저장소에 `rand`가 두 버전 들어 있어 ``specification `rand` is ambiguous`` 에러가 납니다(이름에 버전을 붙여 가리키는 방법은 [Part 3 ch2](../../part3-cargo/ch2-dependencies/README.md) 2.5절).

**직접 해 보기**

저장소 밖의 빈 디렉터리에서 연습용 프로젝트를 만들어 위 과정을 따라 해 봅니다. 아래 출력은 2026년 11월 기준이며, 그 뒤에 새 버전이 나오면 버전 번호가 다를 수 있습니다.

```sh
cargo new guessing_game
cd guessing_game
cargo add rand
```

```
    Updating crates.io index
      Adding rand v0.10.4 to dependencies
             Features:
             + alloc
             + std
             + std_rng
             + sys_rng
             + thread_rng
             - chacha
             - log
             - serde
             - simd_support
             - unbiased
    Updating crates.io index
     Locking 8 packages to latest Rust 1.98.1 compatible versions
```

`Features` 아래 `+`는 기본으로 켜진 기능, `-`는 꺼진 기능입니다. 뒤에서 쓸 `rand::rng()`는 기본으로 켜진 `thread_rng` 기능에 들어 있습니다. 마지막 줄의 "8 packages"는 `rand`와 `rand`가 의존하는 크레이트를 합친 수이고, 이 시점에 `Cargo.lock`이 생깁니다. `cargo build`를 실행하면 의존성을 내려받아(처음이면 `Downloaded` 줄이 보입니다) 컴파일합니다.

```
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling rand_core v0.10.1
   Compiling getrandom v0.4.3
   Compiling chacha20 v0.10.2
   Compiling rand v0.10.4
   Compiling guessing_game v0.1.0 (/경로/guessing_game)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.82s
```

`Cargo.lock`이 버전을 붙잡아 두는 모습과 `cargo update`가 그것을 갱신하는 모습도 확인해 봅니다. 먼저 요구 사항을 0.10.1로 낮추고, `Cargo.lock`을 일부러 0.10.1로 되돌립니다.

```sh
cargo add rand@0.10.1                  # Cargo.toml: rand = "0.10.1"
cargo update rand --precise 0.10.1     # Cargo.lock을 0.10.1로 고정
```

```
 Downgrading rand v0.10.4 -> v0.10.1
```

첫 명령 `cargo add rand@0.10.1`만으로는 `Cargo.lock`이 바뀌지 않습니다. 이미 고정된 0.10.4가 새 요구 사항 "0.10.1 이상 0.11 미만"도 만족하기 때문입니다. 첫 명령을 건너뛰고 요구 사항이 `"0.10.4"`인 채로 두 번째 명령을 실행하면, 0.10.1이 범위 밖이라 `failed to select a version for the requirement` 에러가 납니다.

두 명령을 실행한 뒤 `cargo build`를 하면 `Compiling rand v0.10.1`이 보입니다. 더 새로운 0.10.4가 있어도 `Cargo.lock`에 적힌 0.10.1을 쓰는 것입니다. 이제 `cargo update`를 실행합니다.

```
$ cargo update
    Updating crates.io index
     Locking 1 package to latest Rust 1.98.1 compatible version
    Updating rand v0.10.1 -> v0.10.4
```

의존성을 고르고 고정하는 방법은 [Part 3 ch2](../../part3-cargo/ch2-dependencies/README.md)에서 자세히 다룹니다.

### 2.4 난수 만들기: `rand::rng().random_range(1..=100)`

```rust
use rand::prelude::*;

let secret_number = rand::rng().random_range(1..=100);
```

- `use rand::prelude::*;`: `rand`에서 자주 쓰는 항목을 한꺼번에 가져옵니다. `*`는 "모두"라는 뜻입니다.
- `rand::rng()`: 현재 스레드 전용 난수 생성기를 돌려줍니다. 운영체제가 주는 무작위 값으로 초기화되므로 실행할 때마다 다른 수가 나옵니다.
- `random_range(1..=100)`: 범위 안의 수를 하나 뽑습니다. `1..=100`은 양 끝을 포함하는 범위이고, `1..100`이라고 쓰면 100은 빠집니다.

`random_range`는 `RngExt`라는 트레이트(trait)가 제공하는 메서드입니다. 트레이트는 여러 타입이 공유하는 메서드 묶음인데, 트레이트의 메서드를 부르려면 그 트레이트가 스코프에 있어야 합니다. `rand::prelude::*`가 `RngExt`를 가져오기 때문에 `random_range`를 부를 수 있습니다(`use rand::RngExt;`만 써도 됩니다). 트레이트는 `ch10-generics-traits-lifetimes`에서 다룹니다.

> **다른 자료와 달라진 점**: The Book 영문판은 `rand = "0.10.1"` 기준이고, 이 저장소는 그 뒤에 나온 0.10.4를 씁니다. 같은 0.10 계열이라 코드는 같습니다. 반면 한국어 번역본(doc.rust-kr.org)과 예전 글은 `rand` 0.8 기준이라 코드가 다릅니다.
>
> | `rand` 버전 | 가져오기 | 난수 만들기 |
> | --- | --- | --- |
> | 0.8 (한국어 번역본) | `use rand::Rng;` | `rand::thread_rng().gen_range(1..=100)` |
> | 0.9 | `use rand::Rng;` | `rand::rng().random_range(1..=100)` |
> | 0.10 (이 저장소, The Book 영문판) | `use rand::prelude::*;` 또는 `use rand::RngExt;` | `rand::rng().random_range(1..=100)` |
>
> 0.9에서는 `thread_rng`와 `gen_range`가 각각 `rng`와 `random_range`로 이름이 바뀌어 경고(deprecated)만 났지만, 0.10에서는 옛 이름이 없어져 컴파일 에러가 납니다(4.4절). 번역본을 따라 할 때 `cargo add rand`로 최신 버전을 받았다면 이 표의 0.10 줄대로 고쳐 씁니다.

### 2.5 문자열을 숫자로: `trim`, `parse`, 섀도잉

입력은 `String`이고 정답은 정수이므로 그대로는 비교할 수 없습니다(4.2절). Rust는 타입이 다른 두 값을 비교할 때 한쪽을 알아서 바꿔 주지 않습니다. 무엇을 어떤 타입으로 바꿔 비교하는지가 코드에 드러나야 의도하지 않은 변환으로 생기는 버그를 막을 수 있기 때문입니다. 그래서 문자열을 숫자로 직접 바꿔야 합니다.

```rust
let guess: u32 = guess.trim().parse().expect("숫자를 입력해야 합니다!");
```

- `trim()`: 앞뒤의 공백과 줄바꿈(`\n`, Windows에서는 `\r\n`)을 걷어 냅니다. `"50\n"`의 `\n`은 숫자가 아니므로 이것을 지우지 않으면 `parse`가 실패합니다.
- `parse()`: 문자열을 다른 타입으로 바꿉니다. 한 문자열을 `u32`, `i64`, `f64` 등 여러 타입으로 바꿀 수 있으므로 어느 타입인지 알려 줘야 합니다. `let guess: u32`처럼 변수 이름 뒤에 `: 타입`을 적는 것이 타입 표기(type annotation)이고, 이것을 빼면 E0282 에러가 납니다(4.3절). `u32`는 부호 없는 32비트 정수입니다.
- `parse()`도 실패할 수 있으므로 `Result`를 돌려줍니다. 3단계에서는 `expect`로 실패하면 멈추게 하고, 4단계에서는 `match`로 실패를 처리합니다.

같은 이름 `guess`로 `let`을 한 번 더 쓴 것을 섀도잉(shadowing)이라고 합니다. 새 변수가 이전 변수를 가리므로 이 줄부터 `guess`는 `u32`입니다. 타입을 바꾸면서 같은 이름을 계속 쓰고 싶을 때 `guess_str`, `guess_num`처럼 이름을 둘 만들지 않아도 됩니다. 섀도잉은 `ch3-common-concepts`에서 자세히 다룹니다.

Rust는 타입을 추론합니다. `guess`를 `u32`로 정하고 `secret_number`와 비교하면, 컴파일러는 `secret_number`도 `u32`여야 한다고 추론합니다. 이런 단서가 없으면 정수의 기본 타입인 `i32`가 됩니다.

### 2.6 `match`로 갈래 나누기

```rust
use std::cmp::Ordering;

match guess.cmp(&secret_number) {
    Ordering::Less => println!("너무 작아요!"),
    Ordering::Greater => println!("너무 커요!"),
    Ordering::Equal => println!("정답입니다!"),
}
```

`cmp`는 두 값을 비교해 `Ordering`을 돌려줍니다. `Ordering`은 `Less`, `Greater`, `Equal` 세 경우 중 하나인 열거형(enum)이고, 각 경우를 배리언트(variant)라고 부릅니다. 열거형은 `ch6-enums-pattern-matching`에서 다룹니다.

`match`는 값을 위에서부터 차례로 각 갈래(arm)의 패턴과 맞춰 보고, 처음 맞는 갈래의 코드만 실행합니다. `=>` 왼쪽이 패턴, 오른쪽이 실행할 코드입니다. 다른 언어의 `switch`와 비슷하지만 중요한 차이가 있습니다. 가능한 경우를 하나라도 빠뜨리면 컴파일되지 않습니다. 예를 들어 `Ordering::Equal` 갈래를 지우면 다음 에러가 납니다.

```
error[E0004]: non-exhaustive patterns: `std::cmp::Ordering::Equal` not covered
```

그래서 "정답일 때 처리를 깜빡하는" 실수를 컴파일 단계에서 잡을 수 있습니다.

### 2.7 반복과 잘못된 입력 처리: `loop`, `break`, `continue`

`loop`는 조건 없이 끝없이 반복합니다. `break`를 만나면 반복을 빠져나오고, `continue`를 만나면 이번 반복의 나머지를 건너뛰고 다음 반복으로 갑니다.

3단계까지는 숫자가 아닌 값을 넣으면 `expect` 때문에 프로그램이 멈췄습니다. 4단계에서는 `expect` 대신 `match`로 `parse`의 결과를 나눕니다.

```rust
let guess: u32 = match guess.trim().parse() {
    Ok(num) => num,
    Err(_) => {
        println!("숫자가 아닙니다: {:?}", guess.trim());
        continue;
    }
};
```

- `Ok(num)` 갈래: 성공했으면 안에 든 숫자를 `num`이라는 이름으로 꺼내고, 그 값이 `match` 전체의 값이 되어 `guess`에 들어갑니다. `match`는 값을 만들어 내는 표현식(expression)이기 때문에 이렇게 `let`의 오른쪽에 쓸 수 있습니다.
- `Err(_)` 갈래: 실패했으면 안내를 출력하고 `continue`로 다음 입력을 받으러 갑니다. `_`는 "안의 값은 쓰지 않는다"는 뜻의 패턴입니다. 새 `guess`(`u32`)는 이 `let` 문이 끝나야 생기므로, `match` 안에서 쓴 `guess.trim()`의 `guess`는 아직 입력 문자열입니다.

`expect`는 실패하면 멈추고, `match`는 실패했을 때 할 일을 직접 정합니다. 어느 쪽을 언제 고르는지는 `ch9-error-handling`에서 다룹니다.

**입력이 끝났을 때(EOF)**: The Book의 완성 코드는 입력이 끝나는 경우를 처리하지 않습니다. 파이프로 넣은 입력이 다 떨어지면 `read_line`은 0바이트를 읽고 `guess`는 빈 문자열이 됩니다. 빈 문자열은 숫자로 바꿀 수 없으니 `continue`하고, 다시 0바이트를 읽고, 이것을 끝없이 반복합니다. 이 저장소의 4단계 예제는 읽은 바이트 수가 0이면 게임을 끝냅니다. `if`는 `ch3-common-concepts`에서 다루지만 다른 언어의 `if`와 같다고 보면 됩니다.

```rust
let bytes = io::stdin()
    .read_line(&mut guess)
    .expect("입력을 읽지 못했습니다");

if bytes == 0 {
    println!("입력이 끝나 게임을 종료합니다. (정답: {secret_number})");
    break;
}
```

시도 횟수를 세는 `attempts`는 반복할 때마다 `attempts += 1`로 값이 바뀌므로 `let mut`로 선언했습니다. 반대로 `secret_number`는 한 번 정하면 바뀌지 않으므로 `mut`가 없습니다.

## 3. 예제 안내

예제는 The Book처럼 한 단계씩 기능을 더해 갑니다. 각 파일은 그 자체로 실행되는 완전한 프로그램입니다.

### 실행 방법

**대화형으로 실행하기**: 저장소 루트에서 실행하면 cargo의 빌드 메시지 뒤에 프로그램이 입력을 기다립니다. 숫자를 입력하고 Enter를 누릅니다.

```sh
cargo run -p p0-ch2-guessing-game --example 04_complete_game
```

반복하는 4단계에서 게임을 중간에 끝내려면 Ctrl+C로 프로그램을 중단하거나, macOS·Linux 터미널에서는 Ctrl+D로 입력의 끝(EOF)을 보냅니다.

**파이프로 입력 넣기**: 입력을 미리 정해 두고 한 번에 넣을 수 있습니다. `-q`는 cargo의 빌드 메시지를 숨기는 옵션입니다. 아래 명령은 macOS·Linux의 셸(bash, zsh) 기준입니다.

```sh
echo 42 | cargo run -q -p p0-ch2-guessing-game --example 01_read_input
printf '50\nabc\n' | cargo run -q -p p0-ch2-guessing-game --example 04_complete_game
seq 1 100 | cargo run -q -p p0-ch2-guessing-game --example 04_complete_game
```

파이프로 넣은 입력은 터미널에 따로 표시되지 않습니다. 대화형으로 실행할 때는 입력한 숫자가 프롬프트 다음 줄에 보이지만, 아래 출력 예시에는 그 줄이 없습니다.

**난수에 관한 주의**: 2~4단계의 정답은 실행할 때마다 바뀝니다. 아래 출력 예시의 정답 숫자, "너무 작아요!"/"너무 커요!" 중 어느 쪽이 나오는지, 몇 번 만에 맞히는지는 한 번 실행한 결과일 뿐이며 직접 실행하면 다르게 나옵니다.

### `01_read_input.rs` — 입력을 받아 그대로 출력

[`examples/01_read_input.rs`](examples/01_read_input.rs): `let mut`로 빈 `String`을 만들고 `read_line`으로 채웁니다. 읽은 문자열에 줄바꿈이 들어 있다는 것과 `read_line`이 읽은 바이트 수를 돌려준다는 것을 보여 줍니다.

```sh
echo 42 | cargo run -q -p p0-ch2-guessing-game --example 01_read_input
```

```
숫자를 맞혀 보세요!
추측한 숫자를 입력하세요.
입력한 값: 42

디버그 형식: "42\n"
읽은 바이트 수: 3
```

`입력한 값: 42` 다음의 빈 줄은 `guess` 끝의 `\n` 때문에 생깁니다. 바이트 수 3은 `4`, `2`, `\n`입니다. 한글은 한 글자가 UTF-8로 3바이트라서 `안녕`을 넣으면 `읽은 바이트 수: 7`이 나옵니다. 아무것도 넣지 않으면(`printf '' | ...`) `디버그 형식: ""`, `읽은 바이트 수: 0`이 나옵니다. 이 0이 4단계에서 입력의 끝을 알아내는 단서가 됩니다.

### `02_secret_number.rs` — 난수로 정답 만들기

[`examples/02_secret_number.rs`](examples/02_secret_number.rs): 1단계에 `rand`를 더해 1~100 사이의 정답을 만듭니다. 동작을 확인하려고 정답을 출력합니다. 아직 비교는 하지 않습니다.

```sh
echo 50 | cargo run -q -p p0-ch2-guessing-game --example 02_secret_number
```

```
숫자를 맞혀 보세요!
[개발용] 정답: 79
추측한 숫자를 입력하세요.
입력한 값: 50
```

여러 번 실행해 정답이 바뀌는 것을 확인합니다. 같은 명령을 세 번 실행했을 때 정답은 79, 1, 60이었습니다.

### `03_compare.rs` — 숫자로 바꿔 `match`로 비교

[`examples/03_compare.rs`](examples/03_compare.rs): 입력을 `trim`과 `parse`로 `u32`로 바꾸고(섀도잉), `cmp`와 `match`로 정답과 비교합니다. 기회는 한 번뿐입니다.

```sh
echo 50 | cargo run -q -p p0-ch2-guessing-game --example 03_compare
```

```
숫자를 맞혀 보세요!
[개발용] 정답: 68
추측한 숫자를 입력하세요.
입력한 값: 50
너무 작아요!
```

`echo '  50 '`처럼 앞뒤에 공백을 넣어도 `trim` 덕분에 `입력한 값: 50`으로 처리됩니다. 숫자가 아닌 값을 넣으면 `expect`가 패닉을 일으켜 프로그램이 종료 코드 101로 멈춥니다. 패닉 메시지에서 `thread 'main'` 뒤 괄호 안의 숫자는 실행마다 다릅니다.

```sh
echo abc | cargo run -q -p p0-ch2-guessing-game --example 03_compare
```

```
숫자를 맞혀 보세요!
[개발용] 정답: 8
추측한 숫자를 입력하세요.

thread 'main' (6078952) panicked at part0-getting-started/ch2-guessing-game/examples/03_compare.rs:48:43:
숫자를 입력해야 합니다!: ParseIntError { kind: InvalidDigit }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

`숫자를 입력해야 합니다!`는 `expect`에 넘긴 메시지이고, 그 뒤는 `parse`가 돌려준 에러입니다. `InvalidDigit`은 숫자가 아닌 문자가 섞였다는 뜻입니다. 빈 입력이면 `kind: Empty`가 나옵니다. `u32`는 부호가 없으므로 `-5`도 `InvalidDigit`입니다.

### `04_complete_game.rs` — 반복과 잘못된 입력 처리까지 갖춘 완성판

[`examples/04_complete_game.rs`](examples/04_complete_game.rs): `loop`로 맞힐 때까지 반복하고, 맞히면 `break`합니다. 숫자가 아닌 입력은 `match`로 걸러 `continue`하고, 입력이 끝나면(EOF) 게임을 끝냅니다. 정답은 더 이상 출력하지 않습니다.

숫자와 숫자가 아닌 값을 섞어 넣은 경우입니다.

```sh
printf '50\nabc\n' | cargo run -q -p p0-ch2-guessing-game --example 04_complete_game
```

```
숫자를 맞혀 보세요! (1~100)
추측한 숫자를 입력하세요.
입력한 값: 50
너무 커요!
추측한 숫자를 입력하세요.
숫자가 아닙니다: "abc"
추측한 숫자를 입력하세요.
입력이 끝나 게임을 종료합니다. (정답: 10)
```

`abc`는 패닉 없이 건너뛰고, 입력이 떨어지자 정답을 알려 주고 끝납니다. 정답이 마침 50이면 첫 입력에서 맞히고 끝납니다.

1부터 100까지 차례로 넣으면 언젠가는 반드시 맞힙니다. `seq 1 100`은 1부터 100까지 한 줄에 하나씩 출력하는 명령입니다. 출력의 마지막 부분은 다음과 같습니다.

```sh
seq 1 100 | cargo run -q -p p0-ch2-guessing-game --example 04_complete_game
```

```
추측한 숫자를 입력하세요.
입력한 값: 20
너무 작아요!
추측한 숫자를 입력하세요.
입력한 값: 21
정답입니다! 21번 만에 맞혔습니다.
```

1부터 하나씩 올라가므로 시도 횟수가 곧 정답과 같습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일 에러를 내는 코드입니다. Cargo의 빌드 대상이 아니므로 챕터 디렉터리에서 `rustc`로 직접 컴파일해 확인합니다. 아래 에러는 rustc 1.98.1에서 실제로 출력된 것입니다.

### 4.1 `mut` 없이 `read_line`에 넘기기 (E0596)

[`compile_fail/e0596_read_line_without_mut.rs`](compile_fail/e0596_read_line_without_mut.rs)

```sh
rustc --edition 2024 compile_fail/e0596_read_line_without_mut.rs
```

```
error[E0596]: cannot borrow `guess` as mutable, as it is not declared as mutable
  --> compile_fail/e0596_read_line_without_mut.rs:18:20
   |
18 |         .read_line(&mut guess)
   |                    ^^^^^^^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
12 |     let mut guess = String::new();
   |         +++
```

- **읽는 법**: "`guess`를 가변으로 빌릴 수 없다. 가변으로 선언되지 않았기 때문이다." `^^^^` 표시가 문제가 된 `&mut guess`를 가리키고, `help`가 고칠 위치(12번째 줄)와 넣을 글자(`+++` 아래의 `mut`)를 보여 줍니다.
- **원인**: `read_line`은 문자열에 입력을 덧붙여야 하므로 가변 참조 `&mut`를 요구합니다. 그런데 `guess`가 불변이라 가변 참조를 만들 수 없습니다.
- **해결**: `help`의 제안대로 `let mut guess = String::new();`로 선언합니다.

### 4.2 문자열과 숫자를 그대로 비교하기 (E0308)

[`compile_fail/e0308_compare_string_number.rs`](compile_fail/e0308_compare_string_number.rs)

```sh
rustc --edition 2024 compile_fail/e0308_compare_string_number.rs
```

```
error[E0308]: mismatched types
  --> compile_fail/e0308_compare_string_number.rs:24:21
   |
24 |     match guess.cmp(&secret_number) {
   |                 --- ^^^^^^^^^^^^^^ expected `&String`, found `&{integer}`
   |                 |
   |                 arguments to this method are incorrect
   |
   = note: expected reference `&String`
              found reference `&{integer}`
```

- **읽는 법**: "타입이 맞지 않는다. `&String`을 기대했는데 `&{integer}`가 왔다." `String`의 `cmp`는 같은 `String`끼리만 비교할 수 있으므로 인자로 `&String`을 기대합니다. `{integer}`는 "어떤 정수 타입인지 아직 정해지지 않은 정수"라는 뜻입니다.
- **원인**: `read_line`으로 받은 `guess`는 문자열인데, 숫자로 바꾸지 않고 정수와 비교했습니다.
- **해결**: 비교하기 전에 숫자로 바꿉니다(2.5절).

  ```rust
  let guess: u32 = guess.trim().parse().expect("숫자를 입력해야 합니다!");
  ```

### 4.3 `parse`에 타입을 알려 주지 않기 (E0282)

[`compile_fail/e0282_parse_without_type.rs`](compile_fail/e0282_parse_without_type.rs)

```sh
rustc --edition 2024 compile_fail/e0282_parse_without_type.rs
```

```
error[E0282]: type annotations needed
  --> compile_fail/e0282_parse_without_type.rs:23:9
   |
23 |     let guess = guess.trim().parse().expect("숫자를 입력해야 합니다!");
   |         ^^^^^
24 |
25 |     match guess.cmp(&secret_number) {
   |           ----- type must be known at this point
   |
help: consider giving `guess` an explicit type
   |
23 |     let guess: /* Type */ = guess.trim().parse().expect("숫자를 입력해야 합니다!");
   |              ++++++++++++
```

- **읽는 법**: "타입 표기가 필요하다." 25번째 줄의 `guess.cmp(...)`에서 메서드를 부르려면 그 시점에 `guess`의 타입이 정해져 있어야 하는데(`type must be known at this point`), 23번째 줄만으로는 알 수 없다는 뜻입니다.
- **원인**: `parse()`는 여러 타입으로 바꿀 수 있어서 결과 타입을 알려 주는 단서가 필요한데, 단서가 없습니다.
- **해결**: `help`의 `/* Type */` 자리에 원하는 타입을 적습니다. `let guess: u32 = ...`.

### 4.4 예전 `rand` 코드를 최신 `rand`로 빌드하기 (E0425, E0599)

외부 크레이트가 필요해 `compile_fail/`에 두지 않았습니다. 아래 에러는 2.3절의 연습용 프로젝트(`rand` 0.10.4)에서 확인한 것입니다.

한국어 번역본의 `rand` 0.8용 코드를 그대로 쓰면 다음 에러가 납니다.

```rust
use rand::Rng;
let secret_number = rand::thread_rng().gen_range(1..=100);
```

```
error[E0425]: cannot find function `thread_rng` in crate `rand`
 --> src/main.rs:4:31
  |
4 |     let secret_number = rand::thread_rng().gen_range(1..=100);
  |                               ^^^^^^^^^^ not found in `rand`
```

함수 이름만 `rng`/`random_range`로 고치고 `use rand::Rng;`를 그대로 두면 다른 에러가 납니다.

```
error[E0599]: no method named `random_range` found for struct `ThreadRng` in the current scope
...
    = help: items from traits can only be used if the trait is in scope
...
help: trait `RngExt` which provides `random_range` is implemented but not in scope; perhaps you want to import it
    |
  1 + use rand::RngExt;
```

- **원인**: `rand` 0.10에서는 `thread_rng`가 없어졌고, `random_range`는 `Rng`가 아니라 `RngExt` 트레이트가 제공합니다. 트레이트의 메서드는 그 트레이트를 `use`로 가져와야 부를 수 있습니다.
- **해결**: `use rand::prelude::*;`(또는 `use rand::RngExt;`)로 가져오고 `rand::rng().random_range(1..=100)`을 씁니다. 번역본의 코드를 그대로 쓰고 싶다면 `Cargo.toml`에 `rand = "0.8.5"`를 적어 0.8 계열을 쓰는 방법도 있습니다.

## 5. 연습 문제

모두 `examples/04_complete_game.rs`를 고쳐서 풉니다. 원본을 보존하고 싶다면 파일을 복사해 `examples/05_my_game.rs`처럼 새 이름으로 저장하고 `--example 05_my_game`으로 실행합니다.

**연습 1. `quit`으로 그만두기**

`quit`을 입력하면 "게임을 그만둡니다."를 출력하고 끝나게 만듭니다. `printf '50\nquit\n70\n' | ...`로 실행했을 때 `70`을 읽기 전에 끝나야 합니다.

<details>
<summary>힌트</summary>

숫자로 바꾸기 **전에** 검사해야 합니다. `parse`에 실패하면 `continue`로 넘어가 버리기 때문입니다. 문자열끼리는 `==`로 비교할 수 있으므로 `if guess.trim() == "quit" { ... break; }`를 `let guess: u32 = match ...` 줄 앞에 넣습니다. `trim()`을 빼면 `"quit\n"`과 비교하게 되어 같지 않습니다.

</details>

**연습 2. 기회는 일곱 번**

일곱 번 안에 맞히지 못하면 "기회를 다 썼습니다. (정답: …)"를 출력하고 끝나게 만듭니다. `printf '0\n0\n0\n0\n0\n0\n0\n0\n' | ...`처럼 여덟 번 입력해도 일곱 번째 입력 뒤에 끝나야 합니다.

<details>
<summary>힌트</summary>

시도 횟수는 이미 `attempts`가 세고 있습니다. 비교하는 `match`가 끝난 뒤(반복의 마지막)에 `if attempts == 7 { ...; break; }`를 넣습니다. 일곱 번째에 맞혔다면 `Ordering::Equal` 갈래의 `break`가 먼저 실행되므로 "기회를 다 썼습니다"는 나오지 않습니다. 숫자가 아닌 입력은 `attempts`를 늘리기 전에 `continue`하므로 기회를 쓰지 않습니다.

</details>

**연습 3. `String::new()`를 반복 밖으로 옮기면?**

`let mut guess = String::new();` 줄을 `loop {` 바로 위로 옮기고 `printf '50\n30\n' | ...`로 실행해 봅니다. 두 번째 입력이 어떻게 처리되는지 실행 전에 예상해 보고, 결과를 설명합니다. 그다음, 줄을 반복 밖에 둔 채로 바르게 동작하도록 고칩니다.

<details>
<summary>힌트</summary>

`read_line`은 기존 내용을 지우지 않고 **뒤에 덧붙입니다**(2.2절). 두 번째 입력 뒤 `guess`가 어떤 값인지는 `숫자가 아닙니다: {:?}` 출력에 그대로 드러납니다. 반복 안의 `let guess: u32 = ...`는 그 반복 안에서만 바깥의 `guess`를 가리므로, 다음 반복에서는 다시 바깥의 `String`을 쓰게 되어 컴파일은 됩니다. 고칠 때는 반복이 시작될 때마다 `guess.clear();`로 문자열을 비웁니다.

</details>

## 6. 참고 자료

- The Book 2장: [Programming a Guessing Game](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html) (영문, `rand` 0.10 기준)
- The Book 한국어 번역본 2장: [추리 게임](https://doc.rust-kr.org/ch02-00-guessing-game-tutorial.html) (`rand` 0.8 기준이므로 2.4절의 표를 참고합니다)
- `rand` 문서: [docs.rs/rand](https://docs.rs/rand/latest/rand/), [`RngExt` 트레이트](https://docs.rs/rand/latest/rand/trait.RngExt.html), [crates.io의 `rand`](https://crates.io/crates/rand)
- 표준 라이브러리 문서: [`std::io::Stdin`](https://doc.rust-lang.org/std/io/struct.Stdin.html) (`read_line` 포함)
- Cargo Book: [`cargo add`](https://doc.rust-lang.org/cargo/commands/cargo-add.html), [`cargo update`](https://doc.rust-lang.org/cargo/commands/cargo-update.html), [Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
- Rust By Example: 이 챕터 전체에 대응하는 절은 없으므로 개념별로 가까운 절을 안내합니다.
  - [Mutability](https://doc.rust-lang.org/rust-by-example/variable_bindings/mut.html): `let`과 `let mut`
  - [To and from Strings](https://doc.rust-lang.org/rust-by-example/conversion/string.html): "Parsing a String" 부분에서 `parse`와 타입 표기
  - [match](https://doc.rust-lang.org/rust-by-example/flow_control/match.html), [loop](https://doc.rust-lang.org/rust-by-example/flow_control/loop.html): `match`, `loop`, `break`, `continue`
  - [Dependencies](https://doc.rust-lang.org/rust-by-example/cargo/deps.html): `Cargo.toml`에 의존성 적기
- Rustlings: [`01_variables`](https://github.com/rust-lang/rustlings/tree/main/exercises/01_variables) (변수, `mut`, 섀도잉)
