# ch11. 자동화된 테스트 작성하기

컴파일러는 타입과 대여 규칙을 검사하지만, 코드가 **의도한 일을 하는지**는 알지 못합니다. 이 챕터에서는 그 빈틈을 메우는 자동화된 테스트(automated test)를 다룹니다. 앞부분은 워크스페이스 안의 예제 파일로 테스트 함수와 단언 매크로, 패닉 검사, `Result`를 반환하는 테스트, 실행 옵션을 하나씩 실행해 보고, 마지막에는 라이브러리 프로젝트 `projects/adder`로 단위 테스트, 통합 테스트, 문서 테스트를 함께 갖춘 패키지를 만들어 봅니다.

- 패키지: `p0-ch11-testing`
- 기준 문서: [The Book 11장 Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024). 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `#[test]` 함수의 구조를 설명하고, `cargo test` 출력에서 통과·실패·무시·걸러진 테스트 수와 단위·통합·문서 테스트 구역을 읽을 수 있다.
2. `assert!`, `assert_eq!`, `assert_ne!`와 사용자 정의 실패 메시지로 단언을 작성하고, `assert_eq!`가 `PartialEq`와 `Debug`를 요구하는 이유를 설명할 수 있다.
3. `#[should_panic(expected = "...")]`로 패닉을 검사하고, `Result<(), E>`를 반환하는 테스트에서 `?`를 쓸 수 있다.
4. `--` 구분자의 의미를 설명하고, 이름 필터, `#[ignore]`와 `--ignored`/`--include-ignored`, `--show-output`, `--test-threads`로 테스트 실행 방식을 바꿀 수 있다.
5. `#[cfg(test)] mod tests`의 단위 테스트(비공개 함수 포함)와 `tests/` 디렉터리의 통합 테스트(공용 모듈 `tests/common/mod.rs`)로 테스트를 구성하고, 바이너리 크레이트만 있는 패키지에서 통합 테스트를 쓸 수 없는 이유를 설명할 수 있다.

## 2. 핵심 개념

### 2.1 테스트가 필요한 이유

"인자에 2를 더해 돌려준다"는 함수 `add_two`를 생각해 봅니다.

```rust
fn add_two(a: i32) -> i32 {
    a + 10 // 실수: 2가 아니라 10을 더함
}
```

이 코드는 아무 경고 없이 컴파일됩니다. 매개변수와 반환값의 타입이 맞고 대여 규칙도 어기지 않았기 때문입니다. 컴파일러는 "2를 더해야 한다"는 의도를 모릅니다. 의도는 사람이 코드로 적어 줘야 하고, 그것이 테스트입니다.

```rust
#[test]
fn add_two_adds_two() {
    assert_eq!(add_two(2), 4); // 위의 잘못된 구현에서는 12 != 4라서 실패합니다.
}
```

테스트의 가치는 한 번 확인하고 끝나는 데 있지 않습니다. 코드를 고칠 때마다 `cargo test` 한 번으로 **이미 맞게 동작하던 부분이 여전히 맞는지** 다시 확인할 수 있습니다. 다만 테스트는 "확인한 경우에는 맞다"는 것만 보여 줍니다. 버그가 없다는 증명은 아니므로, 경계값(0, 최댓값, 빈 목록 등)과 실패해야 하는 입력도 테스트에 넣습니다.

### 2.2 테스트 함수의 구조

테스트는 `#[test]` 속성(attribute)을 붙인 평범한 함수입니다. 본문은 보통 세 단계로 이뤄집니다.

1. **준비**: 테스트에 필요한 값이나 상태를 만듭니다.
2. **실행**: 테스트할 코드를 부릅니다.
3. **단언(assertion)**: 결과가 기대와 같은지 확인합니다. 다르면 패닉합니다.

`cargo new --lib adder`로 라이브러리를 만들면 Cargo가 테스트의 틀을 함께 만들어 줍니다.

```rust
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2); // 준비와 실행
        assert_eq!(result, 4);  // 단언
    }
}
```

`#[cfg(test)]`, `mod tests`, `use super::*`는 2.8절에서 설명합니다. 지금은 "테스트 함수를 모아 두는 모듈"이라고 보면 됩니다.

`cargo test`를 실행하면 Cargo는 코드를 **테스트 모드로 컴파일한 별도의 실행 파일**을 만들어 실행합니다. 이 실행 파일의 `main`은 우리가 쓴 `main`이 아니라 테스트 러너(test runner)입니다. 테스트 러너는 `#[test]`가 붙은 함수를 모두 찾아 실행하고 결과를 보고합니다.

- 테스트 함수가 패닉 없이 끝나면 **통과(ok)**, 패닉하면 **실패(FAILED)** 입니다. 단언 매크로는 조건이 맞지 않을 때 패닉하는 매크로일 뿐입니다.
- 테스트마다 **별도의 스레드**에서 실행됩니다. 그래서 한 테스트가 패닉해도 그 스레드만 끝나고 다른 테스트는 계속 실행됩니다. 실패 메시지의 `thread 'tests::...'`가 테스트 이름을 딴 그 스레드입니다. 스레드 자체는 [ch15](../ch15-concurrency/README.md)에서 다룹니다.
- 우리가 쓴 `main`은 실행되지 않습니다.

**이 저장소의 예제에서 테스트 실행하기**: 이 챕터의 `examples/*.rs`는 `main`과 `#[cfg(test)] mod tests`를 함께 가집니다. `cargo run`은 `main`을, `cargo test --example`은 테스트를 실행합니다.

```sh
cargo run  -p p0-ch11-testing --example 01_assertions   # main 실행
cargo test -p p0-ch11-testing --example 01_assertions   # 이 예제의 테스트 실행
cargo test -p p0-ch11-testing --examples                # 모든 예제의 테스트 실행
```

`--example`이나 `--examples` 없이 `cargo test -p p0-ch11-testing`만 실행하면 아무 테스트도 실행되지 않습니다. Cargo는 예제 안의 테스트를 따로 지정했을 때만 실행하기 때문입니다.

### 2.3 `cargo test` 출력 읽기

[`examples/01_assertions.rs`](examples/01_assertions.rs)의 테스트를 실행한 결과입니다.

```text
   Compiling p0-ch11-testing v0.0.0 (…/part0-getting-started/ch11-testing)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running unittests examples/01_assertions.rs (target/debug/examples/01_assertions-<해시>)

running 10 tests
test tests::failure_demos::assert_eq_fails ... ignored, 실패 데모
test tests::failure_demos::assert_eq_with_message_fails ... ignored, 실패 데모
test tests::failure_demos::assert_fails ... ignored, 실패 데모
test tests::failure_demos::custom_message_fails ... ignored, 실패 데모
test tests::add_two_adds_two ... ok
test tests::add_two_changes_value ... ok
test tests::larger_can_hold_smaller ... ok
test tests::greeting_contains_name ... ok
test tests::smaller_cannot_hold_larger ... ok
test tests::square_has_same_sides ... ok

test result: ok. 6 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

위에서부터 읽습니다.

- **``Finished `test` profile``**: 테스트는 `test` 프로필로 빌드합니다. 디버그 빌드처럼 최적화하지 않은 빌드입니다.
- **`Running unittests 파일 (실행 파일 경로)`**: 테스트 러너 실행 파일 하나를 실행한다는 뜻입니다. 이 줄부터 `test result:` 줄까지가 한 **구역**입니다. 실행 파일 이름 끝의 해시는 환경마다 다릅니다.
- **`test 이름 ... 결과`**: 테스트 하나의 결과입니다. 이름에는 모듈 경로가 붙습니다(`tests::failure_demos::assert_fails`). 결과는 `ok`(통과), `FAILED`(실패), `ignored`(무시, 2.7절)이고, `#[ignore = "이유"]`로 이유를 적었으면 `ignored, 이유`처럼 함께 나옵니다. `#[should_panic]` 테스트(2.5절)에는 이름 뒤에 `- should panic`이 붙습니다.
- **줄의 순서**: 테스트가 여러 스레드에서 동시에 실행되므로 `ok` 줄은 끝난 순서대로 나오며, 실행할 때마다 달라질 수 있습니다. 무시된 테스트는 실행하지 않으므로 대개 앞쪽에 나옵니다.
- **`test result:` 요약 줄**:

  | 항목 | 뜻 |
  | --- | --- |
  | `ok.` / `FAILED.` | 이 구역 전체의 결과입니다. 하나라도 실패하면 `FAILED.`입니다. |
  | `passed` / `failed` | 통과한 테스트 수와 실패한 테스트 수입니다. |
  | `ignored` | `#[ignore]`가 붙어 건너뛴 테스트 수입니다. |
  | `measured` | 성능 측정(벤치마크) 수입니다. 벤치마크는 Part 3 `ch9-testing-ci`에서 다룹니다. |
  | `filtered out` | 이름 필터 등으로 이번 실행에서 걸러진 테스트 수입니다(2.7절). |
  | `finished in` | 이 구역의 실행 시간입니다. 실행마다 다릅니다. |

**실패했을 때의 출력**: 같은 예제의 무시된 테스트를 `-- --ignored`로 실행하면(2.7절) 일부러 틀리게 적은 테스트 4개가 실패합니다. 일부를 옮깁니다.

```text
running 4 tests
test tests::failure_demos::assert_fails ... FAILED
test tests::failure_demos::assert_eq_with_message_fails ... FAILED
test tests::failure_demos::assert_eq_fails ... FAILED
test tests::failure_demos::custom_message_fails ... FAILED

failures:

---- tests::failure_demos::assert_fails stdout ----

thread 'tests::failure_demos::assert_fails' (6306546) panicked at part0-getting-started/ch11-testing/examples/01_assertions.rs:151:13:
assertion failed: smaller.can_hold(&larger)

(… 다른 세 테스트의 실패 내용 …)

failures:
    tests::failure_demos::assert_eq_fails
    tests::failure_demos::assert_eq_with_message_fails
    tests::failure_demos::assert_fails
    tests::failure_demos::custom_message_fails

test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p p0-ch11-testing --example 01_assertions`
```

- 첫 번째 `failures:` 아래에 실패한 테스트마다 `---- 이름 stdout ----` 구획이 생깁니다. 그 테스트가 출력한 내용(2.7절)과 패닉 메시지가 들어 있습니다.
- `panicked at 파일:줄:열:`이 패닉한 위치, 즉 실패한 단언의 위치입니다. 괄호 안의 숫자는 스레드 번호이며 실행마다 다릅니다.
- 실패 구획들의 순서도 실행마다 다를 수 있습니다. ``note: run with `RUST_BACKTRACE=1` …`` 안내는 처음 패닉한 테스트의 구획에 한 번만 붙습니다.
- 두 번째 `failures:` 아래는 실패한 테스트 이름의 목록입니다. 이 목록은 이름 순서입니다.
- 마지막 `error:` 줄은 이 구역만 다시 실행하려면 어떤 옵션을 주면 되는지 알려 줍니다. 테스트가 하나라도 실패하면 `cargo test`는 종료 코드 101로 끝납니다.

**구역이 여러 개인 경우**: 라이브러리, 바이너리, 통합 테스트, 문서 테스트를 모두 가진 패키지에서는 테스트 러너 실행 파일이 여러 개 만들어지고, 구역도 그만큼 생깁니다. `projects/adder`의 출력을 3절에서 읽어 봅니다.

### 2.4 단언 매크로: `assert!`, `assert_eq!`, `assert_ne!`

| 매크로 | 통과 조건 | 실패 메시지 |
| --- | --- | --- |
| `assert!(조건)` | 조건이 `true` | ``assertion failed: 조건식`` (식을 그대로 보여 주고, 값은 보여 주지 않음) |
| `assert_eq!(왼쪽, 오른쪽)` | `왼쪽 == 오른쪽` | ``assertion `left == right` failed``와 두 값 |
| `assert_ne!(왼쪽, 오른쪽)` | `왼쪽 != 오른쪽` | ``assertion `left != right` failed``와 두 값 |

`assert!`는 `bool`을 돌려주는 함수를 검사할 때 알맞습니다. `false`여야 하는 결과는 `!`로 뒤집어 `assert!(!smaller.can_hold(&larger))`처럼 씁니다.

두 값이 같은지 볼 때는 `assert!(a == b)`보다 `assert_eq!(a, b)`가 낫습니다. 실패했을 때 두 값을 모두 보여 주기 때문입니다.

```text
assertion `left == right` failed
  left: Rectangle { width: 3, height: 3 }
 right: Rectangle { width: 4, height: 4 }
```

Rust는 두 인자를 "기댓값"과 "실제값"으로 구분하지 않고 그냥 `left`, `right`라고 부릅니다. 그래서 `assert_eq!(add_two(2), 4)`와 `assert_eq!(4, add_two(2))`는 똑같이 동작합니다. 어느 순서로 쓰든 한 프로젝트 안에서 일관되게 쓰면 됩니다. 이 챕터는 `assert_eq!(실제, 기대)` 순서로 씁니다.

`assert_ne!`는 정확한 결과는 모르지만 "적어도 이 값은 아니어야 한다"는 것을 확인할 때 씁니다. 예를 들어 `assert_ne!(add_two(5), 5)`는 "값이 바뀌었다"는 것만 확인합니다.

**사용자 정의 실패 메시지**: 필수 인자 뒤에 인자를 더 주면 `format!`과 같은 규칙으로 실패 메시지를 만듭니다. 메시지는 실패했을 때만 만들어집니다.

```rust
let result = greeting("영희");
assert!(
    result.contains("영희"),
    "인사말에 이름이 없습니다. 실제 값: `{result}`"
);
```

- `assert!`에 준 메시지는 기본 메시지(``assertion failed: ...``) **대신** 출력됩니다. `assert!`는 값을 보여 주지 않으므로, 이렇게 실제 값을 메시지에 넣어 주면 원인을 찾기 쉽습니다.
- `assert_eq!`, `assert_ne!`에 준 메시지는 기본 메시지 **뒤에** 붙고, 두 값도 그대로 출력됩니다.

```text
assertion `left == right` failed: add_two(2)의 결과가 틀렸습니다
  left: 4
 right: 5
```

**`assert_eq!`가 `PartialEq`와 `Debug`를 요구하는 이유**: `assert_eq!(a, b)`는 안에서 `a == b`로 비교하고, 실패하면 두 값을 `{:?}`(디버그 형식)로 출력합니다. 그래서 비교할 타입은 `==`를 쓰게 해 주는 `PartialEq` 트레이트와, 디버그 형식으로 출력하게 해 주는 `Debug` 트레이트를 모두 구현해야 합니다. 정수와 `String`은 이미 구현하고 있고, `Vec<T>`나 `Option<T>`는 안에 담긴 `T`가 구현하고 있으면 함께 구현됩니다. 직접 만든 구조체나 열거형은 `derive`로 구현을 만들어 붙입니다.

```rust
#[derive(Debug, PartialEq)]
struct Rectangle {
    width: u32,
    height: u32,
}
```

둘 중 하나라도 빠지면 테스트가 **실행되기 전에, 컴파일 단계에서** 거부됩니다. 실패할 때 값을 보여 줄 수 없는 비교는 아예 허락하지 않는 것입니다. 4.1절과 4.2절에서 실제 에러를 봅니다. `derive`가 무엇을 만들어 주는지는 ch5에서 `Debug`로 처음 보았고, 트레이트 자체는 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 다룹니다.

### 2.5 패닉을 검사하는 `#[should_panic]`

잘못된 입력을 받으면 패닉하는 것이 **올바른 동작**인 코드도 있습니다. 다음 `Guess`는 1~100 사이의 값만 담도록 `new`에서 범위를 검사합니다([ch9](../ch9-error-handling/README.md)의 검증 타입과 같은 생각입니다).

```rust
impl Guess {
    fn new(value: i32) -> Guess {
        if value < 1 {
            panic!("추측 값은 1 이상이어야 합니다. 받은 값: {value}");
        } else if value > 100 {
            panic!("추측 값은 100 이하여야 합니다. 받은 값: {value}");
        }

        Guess { value }
    }
}
```

"범위를 벗어나면 패닉한다"는 약속을 테스트하려면 `#[test]` 아래에 `#[should_panic]`을 붙입니다. 이 테스트는 **본문이 패닉하면 통과, 패닉 없이 끝나면 실패**합니다.

```rust
#[test]
#[should_panic]
fn greater_than_100() {
    Guess::new(200);
}
```

`#[should_panic]`만으로는 부족할 때가 많습니다. **어떤 이유로든** 패닉하기만 하면 통과하므로, 엉뚱한 곳에서 난 패닉도 통과시키기 때문입니다. 예를 들어 `new`의 두 메시지가 실수로 뒤바뀌어도 이 테스트는 계속 통과합니다(연습 2). 그래서 보통 `expected`로 패닉 메시지에 들어 있어야 할 문자열을 함께 적습니다.

```rust
#[test]
#[should_panic(expected = "100 이하여야 합니다")]
fn greater_than_100_with_message() {
    Guess::new(200);
}
```

`expected`의 문자열은 패닉 메시지의 **일부**이기만 하면 됩니다. 받은 값처럼 바뀌는 부분은 빼고, 이 패닉을 다른 패닉과 구별할 수 있을 만큼만 적습니다.

`should_panic` 테스트는 두 가지 경우에 실패합니다. [`examples/02_should_panic.rs`](examples/02_should_panic.rs)의 실패 데모 출력입니다.

```text
---- tests::failure_demos::does_not_panic stdout ----
note: test did not panic as expected at part0-getting-started/ch11-testing/examples/02_should_panic.rs:79:12
---- tests::failure_demos::wrong_panic_message stdout ----

thread 'tests::failure_demos::wrong_panic_message' (6267253) panicked at part0-getting-started/ch11-testing/examples/02_should_panic.rs:20:13:
추측 값은 100 이하여야 합니다. 받은 값: 200
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
note: panic did not contain expected string
      panic message: "추측 값은 100 이하여야 합니다. 받은 값: 200"
 expected substring: "1 이상이어야 합니다"
```

- **패닉하지 않음**: ``test did not panic as expected``
- **메시지가 다름**: ``panic did not contain expected string``과 함께 실제 메시지(`panic message`)와 기대한 문자열(`expected substring`)을 나란히 보여 줍니다.

`should_panic` 테스트가 통과할 때 나는 패닉의 메시지는 테스트 러너가 가로채므로 화면에 나오지 않습니다.

### 2.6 `Result<T, E>`를 반환하는 테스트

테스트 함수는 `()` 대신 `Result<(), E>`를 반환할 수도 있습니다. `Ok(())`를 반환하면 통과, `Err`를 반환하면 실패입니다. 가장 큰 장점은 본문에서 `?` 연산자(ch9)를 쓸 수 있다는 것입니다.

```rust
use std::num::ParseIntError;

fn parse_and_double(text: &str) -> Result<i32, ParseIntError> {
    let number: i32 = text.trim().parse()?;
    Ok(number * 2)
}

#[test]
fn doubles_number() -> Result<(), ParseIntError> {
    let doubled = parse_and_double(" 21 ")?; // Err면 여기서 테스트가 Err를 반환하며 끝남
    assert_eq!(doubled, 42);
    Ok(())
}
```

`?` 대신 `unwrap()`을 써도 테스트는 됩니다(실패하면 패닉하므로). `Result`를 반환하면 중간 단계가 여러 개일 때 `?`로 짧게 쓸 수 있고, 테스트 함수의 모양이 실제 코드의 함수와 비슷해집니다.

알아 둘 점:

- **실패 출력에 위치가 없습니다.** `Err`를 반환해 실패하면 테스트 러너는 `Err` 안의 값을 디버그 형식으로 `Error: ...`처럼 출력할 뿐, 어느 줄의 `?`에서 멈췄는지는 알려 주지 않습니다.

  ```text
  ---- tests::failure_demos::question_mark_returns_err stdout ----
  Error: ParseIntError { kind: InvalidDigit }
  ```

  에러 타입이 `String`이면 `Error: "두 값이 다릅니다"`처럼 따옴표가 붙어 나옵니다. 디버그 형식으로 출력하기 때문입니다. 같은 이유로 `E`는 `Debug`를 구현해야 하며, 그렇지 않으면 ``error[E0277]: `MyErr` doesn't implement `Debug` ``로 컴파일되지 않습니다.
- **"`Err`가 나와야 한다"를 검사할 때는 `?`를 쓰지 않습니다.** `?`는 `Err`를 만나면 테스트를 실패로 끝내기 때문입니다. 결과를 받아서 `assert!(result.is_err())`로 확인합니다.
- **`#[should_panic]`과 함께 쓸 수 없습니다.** `Result`를 반환하는 함수에 `#[should_panic]`을 붙이면 컴파일 에러입니다(4.6절).

### 2.7 테스트 실행 제어

#### `--` 앞은 Cargo에게, 뒤는 테스트 러너에게

`cargo test` 명령줄에는 두 프로그램에게 주는 옵션이 섞여 있습니다.

```text
cargo test [Cargo 옵션] [이름 필터] -- [테스트 러너 옵션]
```

- `--` **앞**의 옵션은 Cargo가 받습니다. 무엇을 빌드하고 어떤 테스트 러너 실행 파일을 실행할지 정합니다(`-p`, `--example`, `--lib`, `--test` 등).
- `--` **뒤**의 옵션은 Cargo가 손대지 않고, 빌드한 테스트 러너 실행 파일에 그대로 넘깁니다. 테스트를 어떻게 실행할지 정합니다(`--test-threads`, `--show-output`, `--ignored` 등).

옵션 목록도 따로 봅니다. `cargo test --help`는 Cargo의 옵션을, `cargo test -- --help`는 테스트 러너의 옵션을 보여 줍니다. 테스트 러너 옵션을 `--` 없이 쓰면 Cargo가 모르는 옵션이라며 거부합니다(4.5절).

자주 쓰는 옵션을 정리하면 다음과 같습니다. 모두 이 챕터의 예제나 `projects/adder`에서 확인했습니다.

| 하고 싶은 일 | 명령 | 받는 쪽 |
| --- | --- | --- |
| 이름에 문자열이 들어간 테스트만 실행 | `cargo test add` | Cargo |
| 여러 문자열 중 하나라도 들어간 테스트 실행 | `cargo test -- add_works internal` | 테스트 러너 |
| 이름이 정확히 같은 테스트만 실행 | `cargo test -- --exact tests::one_hundred` | 테스트 러너 |
| 문자열이 들어간 테스트 빼고 실행 | `cargo test -- --skip shared_file` | 테스트 러너 |
| 무시된 테스트만 실행 | `cargo test -- --ignored` | 테스트 러너 |
| 무시된 테스트까지 모두 실행 | `cargo test -- --include-ignored` | 테스트 러너 |
| 통과한 테스트의 출력도 보기 | `cargo test -- --show-output` | 테스트 러너 |
| 출력을 가로채지 않고 바로 내보내기 | `cargo test -- --no-capture` | 테스트 러너 |
| 테스트 스레드 수 정하기 | `cargo test -- --test-threads=1` | 테스트 러너 |
| 실행하지 않고 테스트 목록만 보기 | `cargo test -- --list` | 테스트 러너 |
| 라이브러리의 단위 테스트만 | `cargo test --lib` | Cargo |
| 통합 테스트 파일 하나만 | `cargo test --test parse_test` | Cargo |
| 문서 테스트만 | `cargo test --doc` | Cargo |
| 예제 하나(또는 전부)의 테스트 | `cargo test --example 01_assertions` (`--examples`) | Cargo |
| 한 구역이 실패해도 나머지 구역 계속 실행 | `cargo test --no-fail-fast` | Cargo |

#### 이름으로 걸러 실행하기

테스트가 많아지면 지금 고치는 부분의 테스트만 실행하고 싶어집니다. `cargo test` 뒤에 문자열을 하나 주면 **이름에 그 문자열이 들어 있는** 테스트만 실행하고, 나머지는 `filtered out`으로 셉니다.

- 테스트 이름에는 모듈 경로가 포함되므로 `cargo test shared_file`처럼 모듈 이름으로 거르면 그 모듈의 테스트가 모두 실행됩니다.
- Cargo는 이름 필터를 **하나만** 받습니다. `cargo test add_works internal`은 ``error: unexpected argument 'internal' found``로 거부됩니다. 여러 개를 주려면 `cargo test -- add_works internal`처럼 `--` 뒤에 씁니다. 테스트 러너는 여러 필터를 받고, 그중 하나라도 들어 있는 테스트를 실행합니다.
- `cargo test 이름`처럼 `--` 앞에 이름을 주면 이 툴체인에서는 **문서 테스트 구역이 아예 실행되지 않습니다.** 문서 테스트까지 이름으로 거르려면 `cargo test -- 이름`처럼 `--` 뒤에 씁니다.

#### `#[ignore]`로 평소에는 건너뛰기

실행에 오래 걸리는 테스트는 `#[test]` 아래에 `#[ignore]`를 붙여 평소 실행에서 뺍니다. `#[ignore = "이유"]`로 이유를 적어 두면 출력에 함께 나옵니다.

```rust
#[test]
#[ignore = "오래 걸림"]
fn expensive_test() {
    std::thread::sleep(std::time::Duration::from_secs(1)); // 1초 동안 멈춤
}
```

- `cargo test -- --ignored`: 무시된 테스트**만** 실행합니다. 나머지는 `filtered out`으로 셉니다.
- `cargo test -- --include-ignored`: 무시된 테스트를 **포함해** 모두 실행합니다.

평소에는 빠르게 `cargo test`만 돌리고, 시간이 있을 때(예: 작업을 마치기 전) 무시된 테스트까지 돌리는 식으로 씁니다. 이 챕터의 예제 01~03은 실패 출력을 보여 주기 위해 일부러 틀리게 적은 테스트에도 `#[ignore = "실패 데모"]`를 붙여 두었습니다. 그 예제들에서 `-- --ignored`로 실행하면 실패하는 것이 정상입니다.

#### 출력 캡처와 `--show-output`

테스트 러너는 테스트 안에서 `println!`으로 출력한 내용을 화면에 바로 내보내지 않고 가로채(capture) 둡니다. 테스트가 동시에 실행되면서 출력이 뒤섞이지 않게 하고, 통과한 테스트의 출력으로 결과 화면이 지저분해지지 않게 하기 위해서입니다.

- 테스트가 **실패하면** 가로챈 출력을 실패 메시지와 함께 `---- 이름 stdout ----` 구획에 보여 줍니다.
- 테스트가 **통과하면** 가로챈 출력을 버립니다. `-- --show-output`을 주면 통과한 테스트의 출력도 `successes:` 구역에 모아서 보여 줍니다.
- `-- --no-capture`를 주면 가로채지 않고 실행 중에 바로 출력합니다. 테스트가 동시에 실행되면 출력이 섞일 수 있으므로 `--test-threads=1`과 함께 쓰면 읽기 쉽습니다.

#### 병렬 실행과 `--test-threads`

테스트 러너는 기본적으로 여러 테스트를 **여러 스레드에서 동시에** 실행합니다. 그래서 빨리 끝나지만, 테스트끼리 **공유하는 상태**가 있으면 문제가 됩니다. 같은 파일, 현재 작업 디렉터리, 환경 변수처럼 테스트 바깥에 있는 것이 공유 상태입니다.

예제 04의 `shared_file` 모듈은 이 문제를 일부러 만듭니다. 두 테스트가 같은 파일에 각각 `A`와 `B`를 쓰고, 0.2초 기다렸다가 다시 읽어 자기가 쓴 값인지 확인합니다. 동시에 실행되면 기다리는 사이에 다른 테스트가 파일을 덮어쓰므로, 코드에는 아무 잘못이 없는데도 둘 중 하나가 실패합니다. 어느 쪽이 실패할지는 실행할 때마다 다릅니다.

해결 방법은 두 가지입니다.

1. **테스트가 서로 공유하지 않게 고칩니다.** 예를 들어 테스트마다 다른 파일 이름을 씁니다. 근본적인 해결책이므로 가능하면 이쪽을 택합니다.
2. **`-- --test-threads=1`로 한 번에 하나씩 실행합니다.** 스레드를 하나만 쓰면 테스트가 차례로 실행되므로 간섭이 없습니다. 대신 전체 실행 시간이 길어집니다. 이때 테스트는 이름 순서로 실행됩니다.

### 2.8 테스트 구성: 단위 테스트와 통합 테스트

Rust 커뮤니티는 테스트를 크게 두 가지로 나눕니다.

| | 단위 테스트(unit test) | 통합 테스트(integration test) |
| --- | --- | --- |
| 위치 | `src/` 안, 테스트할 코드와 **같은 파일**의 `tests` 모듈 | 패키지 최상위의 `tests/` 디렉터리 |
| 크레이트 | 테스트할 코드와 **같은 크레이트** | 파일마다 **별도의 크레이트** |
| 쓸 수 있는 것 | 비공개 항목까지 모두 | 라이브러리의 **공개 API만** |
| 목적 | 작은 단위(함수, 모듈)를 따로 떼어 정확히 검사 | 여러 부분이 함께 맞게 동작하는지, 바깥 사용자 입장에서 검사 |

둘은 서로를 대신하지 않습니다. 부품 하나하나가 맞는지(단위)와 부품을 조립한 결과가 맞는지(통합)는 다른 질문이기 때문입니다.

#### 단위 테스트: `#[cfg(test)] mod tests`

```rust
pub fn add_two(a: u64) -> u64 {
    internal_adder(a, 2)
}

fn internal_adder(left: u64, right: u64) -> u64 { // 비공개
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        assert_eq!(internal_adder(2, 2), 4);
    }
}
```

- **`#[cfg(test)]`**: `cfg`는 configuration(구성)의 줄임말입니다. `#[cfg(test)]`가 붙은 항목은 **테스트 모드로 컴파일할 때만** 컴파일되고, `cargo build`로 만드는 결과물에는 들어가지 않습니다. 테스트 코드 때문에 컴파일 시간이 늘거나 결과물이 커지지 않게 하는 것입니다. 실제로 `projects/adder` 복사본의 `tests` 모듈 안에 타입 에러(`let x: u64 = "문자열";`)를 넣어 보면 `cargo build`는 성공하고 `cargo test`만 E0308로 실패합니다. 모듈 안의 `#[test]`가 없는 도우미 함수도 함께 빠집니다.
- **`mod tests`**: 이름이 `tests`인 것은 관례일 뿐 특별한 규칙은 없습니다. ch7에서 배운 평범한 자식 모듈입니다.
- **`use super::*`**: `tests`는 자식 모듈이므로 부모 모듈의 항목을 쓰려면 경로를 적거나 `use`로 가져와야 합니다. ch7에서 미뤄 둔 글롭(`*`)이 여기서 쓰입니다. 부모의 항목을 한꺼번에 가져오는 것입니다. 빠뜨리면 E0425가 납니다(4.4절).
- **비공개 함수 테스트**: ch7의 규칙대로 자식 모듈은 조상 모듈의 비공개 항목을 쓸 수 있습니다. 그래서 `tests` 모듈은 `pub`이 없는 `internal_adder`도 부를 수 있습니다. 비공개 함수를 테스트해야 하는지는 의견이 갈리지만, Rust는 어느 쪽도 막지 않습니다.

#### 통합 테스트: `tests/` 디렉터리

패키지 최상위(`src/` 옆)에 `tests/` 디렉터리를 만들면 Cargo가 그 안의 `.rs` 파일을 **하나하나 별도의 크레이트**로 컴파일해 통합 테스트로 실행합니다.

```rust
// tests/integration_test.rs
use adder::{add_two, sum};

mod common;

#[test]
fn it_adds_two() {
    assert_eq!(add_two(2), 4);
}
```

- 통합 테스트 파일은 별도의 크레이트이므로 라이브러리를 **크레이트 이름**(`adder`)으로 가져옵니다. ch7에서 `src/main.rs`가 같은 패키지의 라이브러리를 쓰던 것과 같습니다. 따라서 공개 API만 쓸 수 있습니다(4.7절).
- `tests/` 디렉터리는 `cargo test`를 실행할 때만 컴파일하므로 `#[cfg(test)]`를 붙이지 않습니다. 실제로 통합 테스트 파일에 타입 에러를 넣어도 `cargo build`는 성공합니다.
- 파일마다 크레이트가 따로이므로 `cargo test` 출력에 **파일마다 구역이 하나씩** 생기고, `cargo test --test 파일이름`(확장자 `.rs` 없이)으로 그 파일의 테스트만 실행할 수 있습니다.

#### 통합 테스트의 공용 모듈: `tests/common/mod.rs`

여러 통합 테스트 파일에서 같은 준비 코드를 쓰고 싶을 때가 있습니다. 이 도우미를 `tests/common.rs`에 두면, Cargo는 `tests/` 바로 아래의 파일이므로 이것도 통합 테스트 크레이트로 보고 테스트가 하나도 없는 구역을 출력에 만듭니다.

```text
     Running tests/common.rs (target/debug/deps/common-037633121a60a010)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

대신 `tests/common/mod.rs`에 둡니다. Cargo는 **`tests/`의 하위 디렉터리 안의 파일은 통합 테스트 크레이트로 취급하지 않습니다.** 각 테스트 파일에서 `mod common;`으로 선언하면, ch7에서 본 옛 방식(`mod.rs`)의 모듈 파일 규칙에 따라 `tests/common/mod.rs`를 찾습니다. 새 방식(`tests/common.rs`)을 쓰면 위와 같은 빈 구역이 생기므로, 여기서만은 ch7의 권장과 달리 옛 방식을 씁니다.

`mod common;`을 선언한 테스트 파일마다 `common` 모듈이 **따로 한 벌씩** 컴파일된다는 점도 기억해 둡니다. 어떤 테스트 파일이 `common`의 함수를 쓰지 않으면, 그 테스트 크레이트에서는 그 함수가 쓰이지 않은 것이므로 `dead_code` 경고가 납니다(4.8절).

#### 바이너리 크레이트와 통합 테스트

`src/main.rs`만 있고 `src/lib.rs`가 없는 패키지에서는 `tests/`의 통합 테스트가 `main.rs`의 함수를 `use`로 가져올 수 없습니다. 다른 크레이트가 가져다 쓸 수 있게 항목을 내보내는 것은 **라이브러리 크레이트**뿐이기 때문입니다. `main.rs`의 함수에 `pub`을 붙여도 마찬가지입니다(4.8절).

그래서 실행 파일을 만드는 Rust 프로젝트는 보통 ch7의 `restaurant`처럼 구성합니다.

- 실제 기능은 `src/lib.rs`(라이브러리)에 두고, 통합 테스트로 검사합니다.
- `src/main.rs`(바이너리)는 라이브러리를 불러 실행을 시작할 만큼만 짧게 둡니다. `main.rs`에 남은 코드가 거의 없으므로, 라이브러리가 맞게 동작하면 바이너리도 맞게 동작한다고 볼 수 있습니다.

[ch12](../ch12-io-project-minigrep/README.md)에서 이 구성으로 명령줄 프로그램을 만들며 테스트를 먼저 작성하는 방식을 연습합니다.

#### 문서 테스트 맛보기

`///`로 시작하는 주석은 문서 주석(documentation comment)입니다. `cargo doc`이 만드는 API 문서에 들어가며, 문서 주석 안의 ```` ``` ```` 코드 블록은 `cargo test`가 **문서 테스트(doc test)** 로 실제로 컴파일하고 실행합니다.

```rust
/// 두 수를 더합니다.
///
/// ```
/// let sum = adder::add(2, 3);
/// assert_eq!(sum, 5);
/// ```
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
```

문서 테스트도 통합 테스트처럼 라이브러리 바깥의 코드로 취급되므로 `adder::add`처럼 크레이트 이름으로 부릅니다. 문서에 실린 예제가 실제 코드와 어긋나면 테스트가 실패하므로, 문서가 낡는 것을 막아 줍니다. `cargo test` 출력의 `Doc-tests adder` 구역이 문서 테스트의 결과입니다. 문서 테스트의 자세한 문법은 Part 4 `ch3-doc-tests`에서 다룹니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_assertions.rs`](examples/01_assertions.rs) | 테스트 함수의 구조, `assert!`/`assert_eq!`/`assert_ne!`, 사용자 정의 메시지, `derive(Debug, PartialEq)`, 실패 출력 | 1, 2 |
| [`examples/02_should_panic.rs`](examples/02_should_panic.rs) | `#[should_panic]`, `expected`, `should_panic` 테스트가 실패하는 두 경우 | 3 |
| [`examples/03_result_tests.rs`](examples/03_result_tests.rs) | `Result<(), E>`를 반환하는 테스트와 `?`, `is_err()`로 에러 검사, `Err` 반환 시의 실패 출력 | 3 |
| [`examples/04_running_options.rs`](examples/04_running_options.rs) | 출력 캡처와 `--show-output`, 이름 필터, `#[ignore]`, 공유 상태와 `--test-threads` | 4 |
| [`projects/adder`](projects/adder) | 단위 테스트(비공개 함수 포함), 통합 테스트 두 파일과 공용 모듈, 문서 테스트, 얇은 `main.rs` | 1, 4, 5 |
| [`compile_fail/`](compile_fail) | `PartialEq`나 `Debug`가 없는 타입을 `assert_eq!`로 비교 | 2 |

아래 출력에서 `Compiling`, `Finished` 줄은 대부분 뺐습니다. 실행 파일 이름의 해시, 스레드 번호, `finished in` 시간, 그리고 병렬로 실행된 테스트의 결과 줄 순서는 실행마다 다를 수 있습니다.

### 01_assertions — 단언 매크로

```sh
cargo run -q -p p0-ch11-testing --example 01_assertions
```

```text
larger.can_hold(&smaller) = true
smaller.can_hold(&larger) = false
add_two(2) = 4
Rectangle { width: 3, height: 3 }
안녕하세요, 영희님!
```

`main`은 테스트할 함수들이 무엇을 돌려주는지 보여 줍니다. 테스트는 다음 명령으로 실행합니다. 출력은 2.3절에 실었습니다(통과 6, 무시 4).

```sh
cargo test -p p0-ch11-testing --example 01_assertions
```

일부러 틀리게 적은 무시된 테스트 4개를 실행해 실패 메시지를 봅니다.

```sh
cargo test -p p0-ch11-testing --example 01_assertions -- --ignored
```

```text
---- tests::failure_demos::assert_fails stdout ----

thread 'tests::failure_demos::assert_fails' (6306546) panicked at part0-getting-started/ch11-testing/examples/01_assertions.rs:151:13:
assertion failed: smaller.can_hold(&larger)

---- tests::failure_demos::assert_eq_with_message_fails stdout ----

thread 'tests::failure_demos::assert_eq_with_message_fails' (6306545) panicked at part0-getting-started/ch11-testing/examples/01_assertions.rs:177:13:
assertion `left == right` failed: add_two(2)의 결과가 틀렸습니다
  left: 4
 right: 5

---- tests::failure_demos::assert_eq_fails stdout ----

thread 'tests::failure_demos::assert_eq_fails' (6306544) panicked at part0-getting-started/ch11-testing/examples/01_assertions.rs:158:13:
assertion `left == right` failed
  left: Rectangle { width: 3, height: 3 }
 right: Rectangle { width: 4, height: 4 }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::failure_demos::custom_message_fails stdout ----

thread 'tests::failure_demos::custom_message_fails' (6306547) panicked at part0-getting-started/ch11-testing/examples/01_assertions.rs:166:13:
인사말에 이름이 없습니다. 실제 값: `안녕하세요, 영희님!`
```

볼 점:

- `assert!`는 식(`smaller.can_hold(&larger)`)만 보여 주고, `assert_eq!`는 두 값을 보여 줍니다.
- `Rectangle`의 값이 `Rectangle { width: 3, height: 3 }`로 나오는 것은 `#[derive(Debug)]` 덕분입니다.
- `assert!`에 준 메시지는 기본 메시지를 대신하고, `assert_eq!`에 준 메시지는 기본 메시지 뒤에 붙습니다.
- 맨 끝의 요약은 `test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 6 filtered out`입니다. `--ignored`로 실행하면 무시되지 않았던 6개가 `filtered out`으로 세어집니다.

### 02_should_panic — 패닉 검사

```sh
cargo run -q -p p0-ch11-testing --example 02_should_panic
```

```text
만든 값: 50
```

```sh
cargo test -p p0-ch11-testing --example 02_should_panic
```

```text
running 6 tests
test tests::failure_demos::does_not_panic ... ignored, 실패 데모
test tests::failure_demos::wrong_panic_message ... ignored, 실패 데모
test tests::accepts_value_in_range ... ok
test tests::less_than_1_with_message - should panic ... ok
test tests::greater_than_100_with_message - should panic ... ok
test tests::greater_than_100 - should panic ... ok

test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

볼 점:

- `should_panic` 테스트에는 이름 뒤에 `- should panic`이 붙습니다. 세 테스트 모두 `Guess::new` 안에서 패닉했지만, 기대한 패닉이므로 메시지가 화면에 나오지 않습니다.
- `accepts_value_in_range`처럼 패닉하지 **않아야** 하는 경우도 함께 검사합니다. 그래야 "아무 값에나 패닉하는" 잘못된 구현을 잡을 수 있습니다.
- `cargo test -p p0-ch11-testing --example 02_should_panic -- --ignored`로 실행한 실패 데모의 출력은 2.5절에 실었습니다.

### 03_result_tests — `Result`를 반환하는 테스트

```sh
cargo run -q -p p0-ch11-testing --example 03_result_tests
```

```text
Ok(42)
Err(ParseIntError { kind: InvalidDigit })
```

```sh
cargo test -p p0-ch11-testing --example 03_result_tests
```

```text
running 4 tests
test tests::failure_demos::question_mark_returns_err ... ignored, 실패 데모
test tests::doubles_negative_number ... ok
test tests::doubles_number ... ok
test tests::rejects_text ... ok

test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```sh
cargo test -p p0-ch11-testing --example 03_result_tests -- --ignored
```

```text
running 1 test
test tests::failure_demos::question_mark_returns_err ... FAILED

failures:

---- tests::failure_demos::question_mark_returns_err stdout ----
Error: ParseIntError { kind: InvalidDigit }


failures:
    tests::failure_demos::question_mark_returns_err

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

볼 점:

- `doubles_number`는 `?`를 쓰고, `doubles_negative_number`는 `Result<(), String>`을 반환하며 `Ok`/`Err`를 직접 만듭니다. 둘 다 `Ok(())`로 끝나면 통과입니다.
- `rejects_text`는 "에러가 나와야 한다"를 `assert!(result.is_err(), ...)`로 확인합니다.
- 실패 출력에는 `panicked at 파일:줄`이 없습니다. 2.3절의 단언 실패와 비교해 보세요. `Err` 반환으로 실패하면 위치 대신 `Err` 안의 값만 나옵니다.

### 04_running_options — 실행 옵션

```sh
cargo run -q -p p0-ch11-testing --example 04_running_options
```

```text
add_two 호출: a = 100
결과: 102
```

기본 실행에서는 통과한 테스트의 `println!` 출력이 보이지 않습니다.

```sh
cargo test -p p0-ch11-testing --example 04_running_options
```

```text
running 6 tests
test tests::expensive_test ... ignored, 오래 걸림
test tests::shared_file::writes_a ... ignored, 병렬 실행 데모
test tests::shared_file::writes_b ... ignored, 병렬 실행 데모
test tests::add_three_and_two ... ok
test tests::add_two_and_two ... ok
test tests::one_hundred ... ok

test result: ok. 3 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**`--show-output`**: 통과한 테스트의 출력을 `successes:` 구역에 모아 보여 줍니다(위와 같은 결과 줄은 뺐습니다).

```sh
cargo test -p p0-ch11-testing --example 04_running_options -- --show-output
```

```text
successes:

---- tests::add_three_and_two stdout ----
add_two 호출: a = 3

---- tests::add_two_and_two stdout ----
add_two 호출: a = 2

---- tests::one_hundred stdout ----
add_two 호출: a = 100


successes:
    tests::add_three_and_two
    tests::add_two_and_two
    tests::one_hundred
```

**이름 필터**: 하나의 이름 전체를 주거나, 여러 이름에 공통으로 들어간 일부를 줍니다.

```sh
cargo test -p p0-ch11-testing --example 04_running_options one_hundred
```

```text
running 1 test
test tests::one_hundred ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s
```

```sh
cargo test -p p0-ch11-testing --example 04_running_options add
```

```text
running 2 tests
test tests::add_three_and_two ... ok
test tests::add_two_and_two ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

**`#[ignore]`와 `--ignored`**: 이름 필터 `expensive`와 함께 주면 무시된 테스트 중 `expensive_test`만 실행합니다. 1초 동안 멈추므로 `finished in`이 1초쯤 됩니다.

```sh
cargo test -p p0-ch11-testing --example 04_running_options expensive -- --ignored
```

```text
running 1 test
test tests::expensive_test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 1.01s
```

`--include-ignored`와 `--skip`을 함께 주면 "무시된 것까지 모두, 단 공유 상태 데모는 빼고" 실행할 수 있습니다.

```sh
cargo test -p p0-ch11-testing --example 04_running_options -- --include-ignored --skip shared_file
```

```text
running 4 tests
test tests::add_three_and_two ... ok
test tests::add_two_and_two ... ok
test tests::one_hundred ... ok
test tests::expensive_test ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 1.00s
```

**공유 상태와 `--test-threads`**: `shared_file` 모듈의 두 테스트를 병렬로(기본값) 실행하면 둘 중 하나가 실패합니다.

```sh
cargo test -p p0-ch11-testing --example 04_running_options shared_file -- --ignored
```

```text
running 2 tests
test tests::shared_file::writes_b ... ok
test tests::shared_file::writes_a ... FAILED

failures:

---- tests::shared_file::writes_a stdout ----
파일에 쓴 값: A
파일에서 읽은 값: B

thread 'tests::shared_file::writes_a' (6306865) panicked at part0-getting-started/ch11-testing/examples/04_running_options.rs:83:13:
assertion `left == right` failed
  left: "B"
 right: "A"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::shared_file::writes_a

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.21s
```

같은 테스트를 스레드 하나로 실행하면 둘 다 통과합니다.

```sh
cargo test -p p0-ch11-testing --example 04_running_options shared_file -- --ignored --test-threads=1
```

```text
running 2 tests
test tests::shared_file::writes_a ... ok
test tests::shared_file::writes_b ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.41s
```

볼 점:

- 병렬 실행에서는 `writes_a`가 실패할 때도, `writes_b`가 실패할 때도 있습니다. 실패한 테스트의 구획에 그 테스트가 출력한 `파일에 쓴 값`, `파일에서 읽은 값`이 나옵니다. 가로챈 출력은 이렇게 실패한 테스트에서 원인을 찾는 데 쓰입니다.
- 스레드 하나로 실행하면 두 테스트가 차례로 0.2초씩 기다리므로, 병렬 실행 때보다 `finished in` 시간이 늘어납니다.
- 이 예제를 `-- --include-ignored`만 주고 실행하면 공유 상태 데모도 병렬로 실행되어 하나가 실패합니다. 위처럼 `--skip shared_file`을 함께 줍니다.

### projects/adder — 단위·통합·문서 테스트를 갖춘 라이브러리

`cargo new --lib adder`로 만든 라이브러리에 바이너리(`src/main.rs`)와 통합 테스트(`tests/`)를 더한 프로젝트입니다. 워크스페이스와 분리된 독립 프로젝트이므로 그 디렉터리에서 실행합니다.

```text
projects/adder/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── lib.rs                 라이브러리 크레이트: 함수들 + 단위 테스트(mod tests) + 문서 테스트(add의 문서 주석)
│   └── main.rs                바이너리 크레이트: 라이브러리를 불러 결과를 출력하기만 함
└── tests/
    ├── common/
    │   └── mod.rs             통합 테스트용 공용 도우미 (테스트 크레이트가 아님)
    ├── integration_test.rs    통합 테스트 크레이트 1
    └── parse_test.rs          통합 테스트 크레이트 2
```

라이브러리의 공개 API는 `add`, `add_two`, `sum`(목록의 합, 오버플로가 나면 패닉), `parse_sum`(`"1 + 2 + 3"` 같은 식을 계산, 숫자가 아니면 `Err`)이고, `internal_adder`는 비공개입니다.

```sh
cd part0-getting-started/ch11-testing/projects/adder
cargo run
```

```text
add_two(3) = 5
10 + 20 + 30 = 60
10 + 스물: 계산할 수 없습니다 (invalid digit found in string)
```

```sh
cargo test
```

```text
   Compiling adder v0.1.0 (…/part0-getting-started/ch11-testing/projects/adder)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/lib.rs (target/debug/deps/adder-aa0091fdd9af81e8)

running 7 tests
test tests::add_two_many_times ... ignored, 오래 걸림: cargo test -- --ignored로 실행
test tests::add_works ... ok
test tests::internal ... ok
test tests::parse_sum_rejects_text ... ok
test tests::parse_sum_works ... ok
test tests::sum_of_empty_is_zero ... ok
test tests::sum_overflow_panics - should panic ... ok

test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/adder-e0ee12466f32ba86)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/integration_test.rs (target/debug/deps/integration_test-bea9c4588e930380)

running 2 tests
test it_adds_two ... ok
test sum_one_to_ten ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/parse_test.rs (target/debug/deps/parse_test-470a43239ea6f219)

running 2 tests
test parse_sum_matches_sum ... ok
test parse_sum_rejects_missing_number ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests adder

running 1 test
test src/lib.rs - add (line 12) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.38s; merged doctests compilation took 0.13s
```

볼 점:

- 구역이 다섯 개입니다. 크레이트(테스트 러너 실행 파일)마다 하나씩입니다.

  | 구역 | 크레이트 | 내용 |
  | --- | --- | --- |
  | `Running unittests src/lib.rs` | 라이브러리 | `src/lib.rs`의 `tests` 모듈(단위 테스트) |
  | `Running unittests src/main.rs` | 바이너리 | `main.rs`에 테스트가 없으므로 0개 |
  | `Running tests/integration_test.rs` | 통합 테스트 1 | 이름에 모듈 경로가 없습니다(`it_adds_two`). 파일 자체가 크레이트 루트이기 때문입니다. |
  | `Running tests/parse_test.rs` | 통합 테스트 2 | 파일마다 구역이 따로 생깁니다. |
  | `Doc-tests adder` | 문서 테스트 | `src/lib.rs`의 12번째 줄에서 시작하는 코드 블록(`add`의 문서 주석) |

- `tests/common/mod.rs`의 구역은 없습니다. 하위 디렉터리 안의 파일이기 때문입니다.
- 비공개 함수 `internal_adder`는 단위 테스트(`tests::internal`)에서만 검사합니다. 통합 테스트에서는 부를 수 없습니다(4.7절).
- 마지막 `all doctests ran in …; merged doctests compilation took …` 줄은 에디션 2024의 rustdoc이 문서 테스트를 하나로 묶어 컴파일하기 때문에 붙습니다([Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md)). 시간은 실행마다 다릅니다.
- 실행 파일 이름 끝의 해시(`adder-aa0091fdd9af81e8` 등)는 환경마다 다를 수 있습니다.

**원하는 구역이나 테스트만 실행하기**

```sh
cargo test --test parse_test     # tests/parse_test.rs만
```

```text
     Running tests/parse_test.rs (target/debug/deps/parse_test-470a43239ea6f219)

running 2 tests
test parse_sum_rejects_missing_number ... ok
test parse_sum_matches_sum ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```sh
cargo test parse_sum             # 이름에 parse_sum이 들어간 테스트만, 모든 구역에서
```

이 명령은 라이브러리 구역에서 2개(`5 filtered out`), `parse_test` 구역에서 2개를 실행하고, `integration_test` 구역은 `running 0 tests`(`2 filtered out`)가 됩니다. 이름을 `--` 앞에 주었으므로 문서 테스트 구역은 실행되지 않습니다(2.7절).

```sh
cargo test -- --ignored          # 무시된 테스트만
```

라이브러리 구역에서 `tests::add_two_many_times ... ok`(`6 filtered out`)만 실행되고, 나머지 구역은 모두 `running 0 tests`입니다. 이 테스트는 `add_two`를 5천만 번 부르므로 다른 테스트보다 오래 걸립니다. `cargo test -- --include-ignored`로 실행하면 모든 구역이 통과합니다(라이브러리 구역 `7 passed`).

**실패를 재현해 보기**: 프로젝트를 저장소 밖으로 복사한 뒤, 복사본에서 비공개 함수에 버그를 넣어 봅니다.

```sh
cp -R part0-getting-started/ch11-testing/projects/adder /tmp/adder-try
cd /tmp/adder-try
```

`src/lib.rs`의 `internal_adder` 본문 `left + right`(28번째 줄)를 `left + right + 1`로 바꾸고 `cargo test`를 실행합니다.

```text
     Running unittests src/lib.rs (target/debug/deps/adder-aa0091fdd9af81e8)

running 7 tests
test tests::add_two_many_times ... ignored, 오래 걸림: cargo test -- --ignored로 실행
test tests::add_works ... ok
test tests::parse_sum_works ... ok
test tests::parse_sum_rejects_text ... ok
test tests::sum_of_empty_is_zero ... ok
test tests::internal ... FAILED
test tests::sum_overflow_panics - should panic ... ok

failures:

---- tests::internal stdout ----

thread 'tests::internal' (6283931) panicked at src/lib.rs:68:9:
assertion `left == right` failed
  left: 5
 right: 4
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::internal

test result: FAILED. 5 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

- 라이브러리 구역이 실패하자 **뒤의 구역(바이너리, 통합 테스트, 문서 테스트)은 실행되지 않았습니다.** Cargo는 한 구역이 실패하면 거기서 멈춥니다.
- `cargo test --no-fail-fast`로 실행하면 나머지 구역도 모두 실행합니다. 이번 버그는 `add_two`를 통해 통합 테스트 `it_adds_two`(`tests/integration_test.rs:12:5`)도 실패시키고, 마지막에 실패한 구역을 모아 보여 줍니다.

  ```text
  error: 2 targets failed:
      `--lib`
      `--test integration_test`
  ```

- ``error: test failed, to rerun pass `--lib` ``는 실패한 구역만 다시 실행하려면 `cargo test --lib`을 쓰라는 안내입니다.

## 4. 자주 하는 실수

4.1절과 4.2절은 `compile_fail/`의 파일로 재현합니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일합니다. 코드를 고쳐 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part0-getting-started/ch11-testing
rustc --edition 2024 compile_fail/e0369_assert_eq_without_partial_eq.rs --out-dir /tmp/rust-ch11
```

나머지는 테스트 모드에서만 드러나거나 여러 파일이 필요한 실수라서 `projects/adder`의 복사본(3절 끝의 `/tmp/adder-try`)에서 재현합니다. 하나를 확인한 뒤에는 고친 곳을 되돌리고 다음으로 넘어갑니다. 복사본에서 재현한 출력의 줄 번호는 코드를 어디에 어떻게 추가했는지에 따라 조금 다를 수 있습니다.

### 4.1 `PartialEq`가 없는 타입을 `assert_eq!`로 비교 — E0369

파일: [`compile_fail/e0369_assert_eq_without_partial_eq.rs`](compile_fail/e0369_assert_eq_without_partial_eq.rs)

```text
error[E0369]: binary operation `==` cannot be applied to type `Point`
  --> compile_fail/e0369_assert_eq_without_partial_eq.rs:19:5
   |
19 |     assert_eq!(origin(), Point { x: 0, y: 0 });
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |     |
   |     Point
   |     Point
   |
note: an implementation of `PartialEq` might be missing for `Point`
  --> compile_fail/e0369_assert_eq_without_partial_eq.rs:9:1
   |
 9 | struct Point {
   | ^^^^^^^^^^^^ must implement `PartialEq`
help: consider annotating `Point` with `#[derive(PartialEq)]`
```

- **읽는 법**: `assert_eq!`를 썼는데 에러는 `==`(binary operation)에 대한 것입니다. `assert_eq!`가 안에서 `==`로 비교하기 때문입니다. 밑줄 아래의 `Point`, `Point`는 비교하려던 두 값의 타입입니다.
- **원인**: `Point`에 `#[derive(Debug)]`만 있고 `PartialEq`가 없습니다.
- **해결**: `help:`대로 `PartialEq`를 derive합니다. 이미 `#[derive(Debug)]`가 있으므로 `#[derive(Debug, PartialEq)]`처럼 한 줄에 합쳐 씁니다.

### 4.2 `Debug`가 없는 타입을 `assert_eq!`로 비교 — E0277

파일: [`compile_fail/e0277_assert_eq_without_debug.rs`](compile_fail/e0277_assert_eq_without_debug.rs)

```text
error[E0277]: `Point` doesn't implement `Debug`
  --> compile_fail/e0277_assert_eq_without_debug.rs:19:5
   |
19 |     assert_eq!(origin(), Point { x: 0, y: 0 });
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Debug` is not implemented for `Point`
   |
help: consider annotating `Point` with `#[derive(Debug)]`
```

- **읽는 법**: 같은 에러가 두 번 나옵니다. 실패했을 때 출력할 값이 왼쪽과 오른쪽 두 개이기 때문입니다.
- **원인**: `Point`는 `PartialEq`가 있어 비교는 할 수 있지만, 실패했을 때 `left:`, `right:`로 값을 보여 줄 `Debug`가 없습니다. 비교가 실제로 실패하는지와 상관없이 컴파일 단계에서 거부됩니다.
- **해결**: `#[derive(Debug, PartialEq)]`처럼 `Debug`도 derive합니다. `assert!(origin() == Point { x: 0, y: 0 })`로 바꾸면 `Debug` 없이도 컴파일되지만, 실패했을 때 값을 볼 수 없으므로 권하지 않습니다.

### 4.3 `#[test]`를 빠뜨림 — 테스트가 조용히 빠짐

복사본의 `src/lib.rs`에서 `fn internal()` 위의 `#[test]` 한 줄을 지우고 `cargo test --lib`을 실행합니다.

```text
warning: function `internal` is never used
  --> src/lib.rs:66:8
   |
66 |     fn internal() {
   |        ^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
...
running 6 tests
...
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

- **원인**: `tests` 모듈 안에 있다고 테스트가 되는 것이 아닙니다. 테스트 러너는 `#[test]`가 붙은 함수만 찾습니다. `#[test]`가 없는 함수는 테스트에서 쓰는 도우미 함수로 취급되므로, 아무도 부르지 않으면 `dead_code` 경고만 남습니다.
- **위험한 점**: 에러가 아니라 경고이고 결과도 `ok.`라서 놓치기 쉽습니다. 이 함수 안의 단언이 틀렸더라도 실행되지 않으므로 아무도 모릅니다. 테스트 수(`running 6 tests`)가 줄었는지 확인하는 습관이 도움이 됩니다.
- **해결**: 테스트 함수마다 `#[test]`를 붙입니다.

### 4.4 `use super::*`를 빠뜨림 — E0425

복사본의 `src/lib.rs`에서 `tests` 모듈의 `use super::*;` 줄을 지우고 `cargo test --lib`을 실행하면 같은 종류의 에러가 8개 납니다. 첫 번째만 옮깁니다.

```text
error[E0425]: cannot find function `add` in this scope
  --> src/lib.rs:61:20
   |
61 |         assert_eq!(add(2, 2), 4);
   |                    ^^^ not found in this scope
   |
help: consider importing this function
   |
59 +     use crate::add;
   |
```

- **원인**: `tests`는 자식 모듈이라 별개의 스코프입니다. 부모 모듈에 정의된 `add`라는 이름이 자동으로 보이지 않습니다(ch7의 `use` 스코프 규칙).
- **해결**: `tests` 모듈 맨 위에 `use super::*;`를 둡니다. 컴파일러가 제안하는 `use crate::add;`처럼 하나씩 가져와도 되지만, 테스트 모듈에서는 글롭으로 한꺼번에 가져오는 것이 관례입니다.
- 참고로 반대로 `#[cfg(test)]`를 빠뜨리면 `cargo build`에서 ``warning: unused import: `super::*` ``와 함께 ``help: if this is a test module, consider adding a `#[cfg(test)]` to the containing module``이 나옵니다. 일반 빌드에서는 `#[test]` 함수가 빠지므로 `use`가 쓰이지 않기 때문입니다.

### 4.5 테스트 러너 옵션을 `--` 없이 씀

```sh
cargo test --test-threads=1
```

```text
error: unexpected argument '--test-threads' found

  tip: a similar argument exists: '--test'
  tip: to pass '--test-threads' as a value, use '-- --test-threads'

Usage: cargo test --test [<NAME>] [TESTNAME] [-- [ARGS]...]
```

- **원인**: `--test-threads`는 테스트 러너의 옵션인데 `--` 앞에 써서 Cargo가 받았습니다. Cargo에는 그런 옵션이 없습니다. `--show-output`, `--ignored`도 똑같이 거부됩니다.
- **해결**: 두 번째 `tip:`대로 `cargo test -- --test-threads=1`처럼 `--` 뒤에 씁니다. 첫 번째 `tip:`이 권하는 `--test`는 전혀 다른 옵션(통합 테스트 파일 고르기)이므로 따르지 않습니다.

### 4.6 `Result`를 반환하는 테스트에 `#[should_panic]`

복사본의 `src/lib.rs`에서 `parse_sum_works` 위의 `#[test]` 아래에 `#[should_panic]`을 한 줄 추가하고 `cargo test --lib`을 실행합니다.

```text
error: functions using `#[should_panic]` must return `()`
  --> src/lib.rs:84:5
   |
84 | /     fn parse_sum_works() -> Result<(), ParseIntError> {
85 | |         assert_eq!(parse_sum("1 + 2 + 3")?, 6);
86 | |         Ok(())
87 | |     }
   | |_____^
```

- **원인**: `#[should_panic]`은 "패닉했는가"로 판정하고, `Result`를 반환하는 테스트는 "`Err`를 반환했는가"로 판정합니다. 두 판정 방식을 한 함수에 섞을 수 없습니다. 이 에러에는 에러 코드가 없고, 테스트 모드로 컴파일할 때만 나므로 `cargo build`로는 보이지 않습니다.
- **해결**: 패닉을 검사하려면 반환 타입을 없애고 `?` 대신 `unwrap()`을 씁니다. 에러를 검사하려면 `#[should_panic]`을 빼고 `assert!(result.is_err())`를 씁니다(2.6절).

### 4.7 통합 테스트에서 비공개 함수 사용 — E0603

복사본의 `tests/integration_test.rs` 끝에 다음 테스트를 추가하고 `cargo test --test integration_test`를 실행합니다.

```rust
#[test]
fn internal() {
    assert_eq!(adder::internal_adder(2, 2), 4);
}
```

```text
error[E0603]: function `internal_adder` is private
  --> tests/integration_test.rs:23:23
   |
23 |     assert_eq!(adder::internal_adder(2, 2), 4);
   |                       ^^^^^^^^^^^^^^ private function
   |
note: the function `internal_adder` is defined here
  --> src/lib.rs:27:1
   |
27 | fn internal_adder(left: u64, right: u64) -> u64 {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

- **원인**: 통합 테스트 파일은 별도의 크레이트라서, ch7에서 본 공개 규칙에 따라 라이브러리의 공개 항목만 쓸 수 있습니다. 같은 패키지 안에 있다고 특별 대우를 받지 않습니다.
- **해결**: 비공개 함수는 `src/lib.rs`의 `tests` 모듈(단위 테스트)에서 검사합니다. 통합 테스트는 그 비공개 함수를 쓰는 공개 함수(`add_two`)를 통해 간접적으로 검사합니다. 테스트하려고 함수를 `pub`으로 바꾸는 것은 공개 API를 넓히는 일이므로 피합니다.

### 4.8 통합 테스트 구성의 실수

**바이너리 크레이트만 있는 패키지 — E0432.** `cargo new calc`로 만든 바이너리 패키지의 `src/main.rs`에 `pub fn add_two`를 두고, `tests/integration_test.rs`에서 `use calc::add_two;`로 가져오면 다음 에러가 납니다.

```text
error[E0432]: unresolved import `calc`
 --> tests/integration_test.rs:1:5
  |
1 | use calc::add_two;
  |     ^^^^ use of unresolved module or unlinked crate `calc`
  |
  = help: if you wanted to use a crate named `calc`, use `cargo add calc` to add it to your `Cargo.toml`
```

- **원인**: 통합 테스트에서 패키지 이름으로 가져올 수 있는 것은 라이브러리 크레이트(`src/lib.rs`)뿐입니다. 이 패키지에는 라이브러리 크레이트가 없으므로 `calc`라는 크레이트를 찾지 못합니다. `pub`을 붙여도 바이너리 크레이트의 항목은 내보내지지 않습니다.
- **주의**: `help:`의 `cargo add calc`를 따르면 안 됩니다. 그 명령은 crates.io에서 이름이 `calc`인 **남의 크레이트**를 의존성으로 추가합니다.
- **해결**: 기능을 `src/lib.rs`로 옮기고 `main.rs`는 `use calc::add_two;`로 라이브러리를 쓰게 합니다(2.8절, `projects/adder`의 구성).

**공용 모듈을 `tests/common.rs`로 만듦.** 테스트가 하나도 없는 `Running tests/common.rs` 구역이 생깁니다(2.8절의 출력). `tests/common/mod.rs`로 옮깁니다.

**공용 모듈의 함수를 일부 테스트 파일에서만 씀 — `dead_code` 경고.** 복사본의 `tests/common/mod.rs`에 `pub fn zeros(n: u64) -> Vec<u64>`를 추가하고 새 파일 `tests/sum_test.rs`에서만 쓰면, `cargo test`에서 다음 경고가 납니다.

```text
warning: function `zeros` is never used
  --> tests/common/mod.rs:19:8
...
warning: `adder` (test "integration_test") generated 1 warning
warning: `adder` (test "parse_test") generated 1 warning (1 duplicate)
```

- **원인**: `common` 모듈은 `mod common;`을 선언한 테스트 크레이트마다 따로 컴파일됩니다. `integration_test`와 `parse_test` 크레이트에서는 `zeros`를 쓰지 않으므로 그 크레이트 입장에서는 죽은 코드입니다. 경고 끝의 크레이트 이름이 어느 테스트 크레이트에서 난 경고인지 알려 줍니다.
- **해결**: 일부 테스트 파일에서만 쓰는 도우미에는 `#[allow(dead_code)]`를 붙여 경고를 끕니다. 공용 모듈에 둘 만큼 여러 파일에서 쓰는 함수가 아니라면 그 테스트 파일 안으로 옮기는 것도 방법입니다.

## 5. 연습 문제

### 연습 1. 윤년 판별 함수와 테스트

저장소 밖에서 `cargo new --lib leap`으로 라이브러리를 만들고, `pub fn is_leap_year(year: u32) -> bool`을 작성하세요. 4로 나누어떨어지는 해는 윤년이지만, 100으로 나누어떨어지는 해는 평년이고, 400으로 나누어떨어지는 해는 다시 윤년입니다. 그다음 `tests` 모듈에 2024년(윤년), 2023년(평년), 1900년(평년), 2000년(윤년)을 검사하는 테스트를 하나씩 작성하세요. 실패하면 연도가 메시지에 나와야 합니다. 다 되면 `cargo test 1900`으로 1900년 테스트만 실행해 보세요.

<details>
<summary>힌트</summary>

- 윤년이어야 하면 `assert!(is_leap_year(2024), ...)`, 평년이어야 하면 `assert!(!is_leap_year(1900), ...)`처럼 `!`로 뒤집습니다. 메시지는 `"{year}년은 평년이어야 합니다"`처럼 씁니다.
- 테스트 이름은 숫자로 시작할 수 없으므로 `year_1900_is_not_leap`처럼 짓습니다. 이름 필터는 이름의 **일부**와 비교하므로 `cargo test 1900`이 이 테스트를 고릅니다.
- `cargo new --lib`이 만든 `it_works` 테스트와 `add` 함수는 지워도 됩니다.
- 일부러 조건 하나(예: 100으로 나누어떨어지는 경우)를 빼고 실행해서 어떤 테스트가 어떤 메시지로 실패하는지도 확인해 보세요.

</details>

### 연습 2. `expected`가 잡아내는 버그

[`examples/02_should_panic.rs`](examples/02_should_panic.rs)의 `Guess::new`에서 두 `panic!`의 메시지를 서로 바꿔 보세요(1보다 작을 때 "100 이하여야 합니다", 100보다 클 때 "1 이상이어야 합니다"가 나오게). 실행하기 전에 네 개의 일반 테스트 중 어떤 것이 실패할지 예측한 다음, `cargo test -p p0-ch11-testing --example 02_should_panic`으로 확인하세요. 확인한 뒤에는 원래대로 되돌립니다.

<details>
<summary>힌트</summary>

- `expected`가 있는 `greater_than_100_with_message`와 `less_than_1_with_message`는 실패하고, `expected`가 없는 `greater_than_100`은 여전히 통과합니다. `Guess::new(200)`은 바뀐 뒤에도 패닉은 하기 때문입니다.
- 결과 요약은 `2 passed; 2 failed; 2 ignored`가 됩니다. `#[should_panic]`만으로는 "엉뚱한 메시지로 패닉하는" 버그를 잡지 못한다는 것이 이 연습의 요점입니다.

</details>

### 연습 3. 통합 테스트 파일 추가하기

`projects/adder`의 복사본에 `tests/sum_test.rs`를 추가하고, `common::one_to(100)`의 합이 5050인지 검사하는 테스트를 작성하세요. `cargo test --test sum_test`로 이 파일의 테스트만 실행한 뒤, `cargo test` 전체 출력에서 새 구역이 어디에 생겼는지 확인하세요.

<details>
<summary>힌트</summary>

- 파일 맨 위에 `use adder::sum;`과 `mod common;`을 씁니다. 다른 통합 테스트 파일과 같은 모양입니다.
- 새 구역 `Running tests/sum_test.rs`는 `tests/parse_test.rs` 구역 다음, `Doc-tests adder` 구역 앞에 생깁니다. 통합 테스트 구역이 파일 이름 순서(`integration_test`, `parse_test`, `sum_test`)로 나옵니다.
- 이어서 4.8절처럼 `common`에 `sum_test.rs`에서만 쓰는 함수를 추가해 보세요. `cargo test --test sum_test`만 실행하면 경고가 나지 않고, `cargo test`를 실행하면 다른 테스트 크레이트에서 경고가 납니다. 왜 그런지 설명해 보세요.

</details>

### 연습 4. 에러의 내용까지 검사하기

`projects/adder`의 단위 테스트 `parse_sum_rejects_text`는 에러가 났다는 것만 확인합니다. 에러 메시지가 정확히 `invalid digit found in string`인지까지 확인하는 테스트를 복사본에 추가하세요. 또 `parse_sum("1 + 2 +")`의 에러 메시지가 무엇인지 테스트로 확인하세요.

<details>
<summary>힌트</summary>

- ch6의 `let...else`로 `Err` 안의 값을 꺼냅니다. `let Err(e) = parse_sum("1 + 둘") else { panic!("에러가 나와야 합니다"); };`
- `ParseIntError`의 기본 형식(`Display`) 문자열은 `e.to_string()`으로 얻습니다. `assert_eq!(e.to_string(), "invalid digit found in string");`
- 빈 문자열을 숫자로 바꾸려다 난 에러의 메시지는 `cannot parse integer from empty string`입니다. 마지막 `+` 뒤가 빈 문자열이기 때문입니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [11. Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
  - [11.1 How to Write Tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html): 단언 매크로, `should_panic`, `Result`를 반환하는 테스트. `Rectangle::can_hold`, `greeting`, `Guess` 예제가 이 절에서 왔습니다.
  - [11.2 Controlling How Tests Are Run](https://doc.rust-lang.org/book/ch11-02-running-tests.html): `--test-threads`, `--show-output`, 이름 필터, `#[ignore]`
  - [11.3 Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html): 단위 테스트와 통합 테스트, `tests/common/mod.rs`, 바이너리 크레이트의 통합 테스트
- Rust By Example: [Testing](https://doc.rust-lang.org/rust-by-example/testing.html) 장
  - [Unit testing](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html): 이 챕터의 2.4~2.7절을 짧은 예제로 다시 보여 줍니다. `expected`를 줄여 쓴 `#[should_panic = "..."]` 형태도 나옵니다(이 툴체인에서 `expected =`와 똑같이 동작하는 것을 확인했습니다).
  - [Documentation testing](https://doc.rust-lang.org/rust-by-example/testing/doc_testing.html): 문서 테스트를 미리 보고 싶다면 읽어 봅니다. 자세한 내용은 Part 4 `ch3-doc-tests`에서 다룹니다.
  - [Integration testing](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html): `tests/` 디렉터리와 `tests/common/mod.rs`
- Rustlings (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - [`17_tests`](https://github.com/rust-lang/rustlings/tree/main/exercises/17_tests): `tests1.rs`는 `tests` 모듈에 바깥 모듈의 `is_even`을 가져와(글롭을 써도 된다는 안내가 있습니다) `assert!`로 검사하기, `tests2.rs`는 `assert_eq!`로 2의 거듭제곱 함수 검사하기, `tests3.rs`는 너비나 높이가 0 이하이면 패닉하는 `Rectangle::new`를 음수 값으로 불러 `#[should_panic]`으로 검사하기입니다. `tests2.rs`의 `1 << n`은 비트 이동 연산으로, 파일의 주석대로 "2의 n제곱"과 같다고 보고 읽으면 됩니다.
- rustc Book: [Tests](https://doc.rust-lang.org/rustc/tests/index.html): 테스트 러너(`--` 뒤) 옵션 전체의 설명입니다. `cargo test -- --help`로도 같은 목록을 볼 수 있습니다. 이 문서에 따르면 `--nocapture`는 `--no-capture`의 예전 이름입니다.
- 에러 설명: `rustc --explain E0369`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
