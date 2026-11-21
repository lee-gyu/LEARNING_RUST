# ch4. 2024 에디션의 언어 변경

Part 2의 네 번째 챕터입니다. [ch1](../ch1-editions-overview/README.md)에서 에디션이 무엇인지 보았고, [ch2](../ch2-rust-2015-2018/README.md)과 [ch3](../ch3-rust-2021/README.md)에서 앞선 에디션들의 변경을 보았습니다. 이 챕터는 2024 에디션(공식 이름 "Rust 2024", rustc 1.85.0과 함께 나왔습니다)이 **언어**에서 바꾼 것을 Edition Guide 5.1절의 순서대로 모두 다룹니다.

이 저장소는 처음부터 2024 에디션으로 써 왔습니다. 그래서 이 챕터의 방향은 앞 챕터들과 반대입니다. 지금까지 당연하게 써 온 규칙이 2021 에디션에서는 어땠는지 보고, 2021 에디션 코드를 읽거나 옮길 때 **무엇이 컴파일되지 않고, 무엇이 조용히 다르게 동작하는지** 직접 확인합니다. 앞 챕터들이 이 챕터로 미뤄 둔 것도 여기서 정리합니다.

- [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.10절의 반환 위치 `impl Trait`가 인자의 라이프타임과 어떻게 묶이는가(2.3절)
- [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.10절이 확인해 둔 "`if let`의 대여 가드는 2024 에디션에서 else 전에 버려진다"는 규칙의 이유(2.4절)
- [Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.2절의 let 체인이 2024 에디션에서만 되는 이유와 `while` 조건의 함정(2.5절)
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.4·2.5·2.7절이 2024 문법으로만 가르친 `unsafe extern`, `#[unsafe(no_mangle)]`, `static mut`의 2021 모양(2.7~2.9절)과 2.14절의 never 타입(2.10절)

- 패키지: `p2-ch4-rust-2024-language`
- 기준 문서: [Edition Guide 5.1 Rust 2024 → Language](https://doc.rust-lang.org/edition-guide/rust-2024/language.html)와 그 하위 절 14개
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64). 2021 에디션의 동작은 같은 툴체인에서 `--edition 2021`(또는 `edition = "2021"`)로 컴파일해 확인했습니다. never 타입 폴백의 옛 동작만 예전 툴체인 rustc 1.98.1로 확인했습니다(2.10절).
- 독립 프로젝트: [`projects/edition_compare`](projects/edition_compare/Cargo.toml). 같은 소스 파일을 2021 에디션 패키지와 2024 에디션 패키지가 함께 컴파일합니다(2.2절).
- unsafe 코드는 예제 05, `projects/edition_compare/shared/new_syntax.rs`, `compile_fail/unsafe_missing_on_ffi.rs`에 있습니다. 블록마다 `SAFETY:` 주석을 달았지만, 이 환경에는 nightly와 Miri가 없어 Miri로 검사하지 못했습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 반환 위치의 `impl Trait`(RPIT)가 라이프타임을 "캡처한다"는 것이 부르는 쪽에서 어떤 대여로 나타나는지 설명하고, 2021과 2024 에디션의 캡처 규칙 차이 때문에 생기는 E0700·E0502를 정밀 캡처 `use<..>`로 고칠 수 있다.
2. `if let` 조건식과 블록의 마지막 표현식에서 만든 임시값이 두 에디션에서 언제 버려지는지 실행 결과로 설명하고, 그 차이 때문에 한쪽에서만 컴파일되거나 다르게 동작하는 코드를 알아보고 고칠 수 있다. `match`는 왜 그대로인지 말할 수 있다.
3. let 체인이 2024 에디션에서만 되는 이유를 설명하고, `while` 조건의 let 체인이 원소를 하나 더 꺼내는 함정을 피해 작성할 수 있다.
4. `unsafe extern` 블록과 `safe`/`unsafe` 항목, unsafe 속성, unsafe 함수 안의 unsafe 블록, `static mut` 참조 금지를 2021 코드에 적용해 고칠 수 있다. 이 문법들이 2021 에디션에서도 된다는 것을 이용해 에디션을 올리기 전에 미리 고칠 수 있다.
5. never 타입 폴백이 무엇인지 설명하고 rustc 1.100.0의 실제 동작(에디션과 상관없이 `!`)에 맞게 폴백에 기대던 코드를 고칠 수 있다. `gen` 예약 키워드, 패턴의 `mut`/`ref`/`&` 제한, `expr` 프래그먼트 지정자, 예약 문법 때문에 나는 에러를 알아보고 고칠 수 있다.

## 2. 핵심 개념

### 2.1 이 챕터의 지도

Edition Guide 5.1절의 14개 항목입니다. "2021 코드에 생기는 일"은 2021 에디션 코드를 그대로 2024 에디션으로 컴파일했을 때의 결과입니다. "옮기기 전 린트"는 2021 에디션에서 켜면 바뀔 곳을 미리 알려 주는 린트(lint)로, 모두 에디션 호환성 린트 그룹 `rust-2024-compatibility`에 들어 있습니다(`rustc --edition 2021 -W help`로 확인). `cargo fix --edition`이 이 린트 그룹으로 코드를 자동 수정하는 절차는 [Part 2 ch6](../ch6-migration-practice/README.md)에서 다룹니다. 표의 "ch1 4.4절"처럼 적은 것은 [Part 2 ch1](../ch1-editions-overview/README.md) README의 절이고, 거기에 같은 에러를 내는 `compile_fail/` 파일이 있습니다.

| 절 | 항목 | 2021 코드에 생기는 일 | 옮기기 전 린트 (2021 기본 수준) | 확인하는 곳 |
| --- | --- | --- | --- | --- |
| 2.3 | RPIT 라이프타임 캡처 | 대여 에러가 날 수 있음. 반대로 2021의 E0700은 사라짐 | `impl_trait_overcaptures` (allow) | 예제 01, 4.1·4.4절 |
| 2.4 | `if let` 임시값 범위 | 컴파일은 되지만 임시값이 버려지는 시점이 달라짐 | `if_let_rescope` (allow) | 예제 02, 프로젝트 |
| 2.5 | let 체인 | 영향 없음(2024에서 새로 되는 문법) | 없음 | 예제 03, ch1 4.4절 |
| 2.6 | 마지막 표현식 임시값 범위 | 버려지는 순서가 달라짐. 컴파일 결과가 바뀌는 경우도 있음 | `tail_expr_drop_order` (allow) | 예제 04, 프로젝트, 4.2·4.3절 |
| 2.12 | 패턴의 `mut`/`ref`/`&` 제한 | 에러 | `rust_2024_incompatible_pat` (allow) | 예제 07 |
| 2.7 | `unsafe extern` 블록 | 에러 | `missing_unsafe_on_extern` (allow) | 예제 05, 4.5절 |
| 2.8 | unsafe 속성 | 에러 | `unsafe_attr_outside_unsafe` (allow) | 예제 05, 4.5절 |
| 2.12 | `unsafe_op_in_unsafe_fn` 경고 | 경고 | `unsafe_op_in_unsafe_fn` (allow) | 예제 05 |
| 2.9 | `static mut` 참조 금지 | 에러 | `static_mut_refs` (**warn**) | 예제 05 |
| 2.10 | never 타입 폴백 | **rustc 1.100.0에서는 차이 없음**(두 에디션 모두 `!`) | 없음(1.100.0에서 제거됨) | 예제 06, 프로젝트, 4.7절 |
| 2.12 | `expr` 프래그먼트 지정자 | 컴파일은 되지만 매크로가 고르는 규칙이 달라질 수 있음 | `edition_2024_expr_fragment_specifier` (allow) | 예제 07, 프로젝트 |
| 2.12 | 빠진 프래그먼트 지정자 | 1.89부터 모든 에디션에서 에러 | 없음 | 2.12절 |
| 2.11 | `gen` 키워드 | 에러 | `keyword_idents_2024` (allow) | 예제 07, ch1 4.2절 |
| 2.12 | 예약 문법 `#"..."#`, `##` | 에러 | `rust_2024_guarded_string_incompatible_syntax` (allow) | 예제 07, 4.6절 |

표를 보면 2024 에디션의 변경은 세 종류입니다. **2021 코드가 컴파일되지 않는 것**(에러)은 컴파일러가 알려 주므로 오히려 쉽습니다. 어려운 것은 **컴파일은 되는데 동작이 달라지는 것**(임시값 범위, `expr` 지정자)입니다. 마지막으로 let 체인처럼 **2024에서 새로 되는 것**이 있습니다. 각 절은 "2021에서는 → 2024에서는 → 왜 바꿨나 → 옮길 때" 순서로 설명합니다.

### 2.2 같은 코드를 두 에디션으로 컴파일하는 방법

이 챕터의 실습은 같은 코드를 두 에디션으로 컴파일해 결과를 비교하는 것입니다. 방법은 두 가지입니다.

**파일 하나라면 `rustc --edition`.** `compile_fail/`의 파일과 이 README의 짧은 코드 조각은 이렇게 확인했습니다. `--out-dir`은 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않게 하려고 붙입니다.

```sh
cd part2-editions/ch4-rust-2024-language
rustc --edition 2021 compile_fail/e0502_rpit_captures_borrow.rs --out-dir /tmp/rust-p2-ch4   # 통과
rustc --edition 2024 compile_fail/e0502_rpit_captures_borrow.rs --out-dir /tmp/rust-p2-ch4   # E0502
```

**실행해서 비교하려면 `projects/edition_compare`.** 작은 워크스페이스에 패키지 두 개(`e2021`, `e2024`)가 있고, 둘 다 `shared/`의 같은 소스 파일을 `[[bin]]`의 `path`로 가리킵니다. 패키지마다 `edition`이 다르므로 **같은 파일이 두 에디션으로 컴파일**됩니다.

```toml
# e2021/Cargo.toml (e2024/Cargo.toml은 2021이 2024로 바뀐 것만 다릅니다)
[package]
name = "e2021"
edition = "2021"

[[bin]]
name = "if_let_scope_2021"
path = "../shared/if_let_scope.rs"
```

최상위 `Cargo.toml`은 `[package]` 없이 `[workspace]`만 있는 가상 매니페스트입니다. 패키지를 둘로 나눈 것은 `[[bin]]`마다 `edition`을 적는 타깃별 키가 1.100.0에서 deprecated 경고를 내기 때문입니다([ch1](../ch1-editions-overview/README.md) 2.6절). 실행 파일 이름 끝에 `_2021`/`_2024`를 붙인 것은, 두 패키지의 실행 파일 이름이 같으면 Cargo가 `output filename collision` 경고를 내기 때문입니다(같은 `target/debug/`에 같은 이름의 파일을 만들게 되므로). 프로젝트 디렉터리에서 `cargo run -q --bin if_let_scope_2021`과 `cargo run -q --bin if_let_scope_2024`처럼 실행합니다. 실행 파일 목록과 출력은 3절 끝에 있습니다.

### 2.3 RPIT 라이프타임 캡처

[Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.10절에서 반환 타입에 `impl Trait`를 쓰면 실제 타입을 숨긴 채 "그 트레이트를 구현한 어떤 타입"을 돌려준다고 배웠습니다. 이것을 **반환 위치의 `impl Trait`(return-position impl Trait, RPIT)** 라고 부릅니다. 부르는 쪽이 보는 `impl Iterator<Item = usize>`를 **불투명 타입(opaque type)**, 함수 안의 실제 타입(예: `Range<usize>`)을 **숨은 타입(hidden type)** 이라고 합니다.

**캡처한다는 것.** 불투명 타입이 어떤 제네릭 매개변수(타입 매개변수와 라이프타임 매개변수)를 "쓸 수 있는가"를 캡처(capture)라고 합니다. 라이프타임 `'a`를 캡처한 불투명 타입은 `'a`에 묶입니다. 부르는 쪽에서는 **반환된 값이 살아 있는 동안 `'a`로 빌린 인자를 계속 빌리고 있다**고 취급됩니다. 부르는 쪽은 숨은 타입을 볼 수 없으므로, 숨은 타입이 실제로 그 인자를 빌리는지와 상관없이 시그니처만 보고 판단합니다.

```rust
fn positions(numbers: &[i32]) -> impl Iterator<Item = usize> {
    0..numbers.len() // 숨은 타입 Range<usize>는 numbers를 빌리지 않습니다.
}

let mut scores = vec![70, 85];
let it = positions(&scores);
scores.push(90); // 2021: 통과, 2024: E0502
println!("{:?}", it.collect::<Vec<_>>());
```

**2021에서는.** 타입 매개변수는 언제나 캡처했지만, 라이프타임 매개변수는 **불투명 타입의 바운드에 이름이 나올 때만** 캡처했습니다(일반 함수와 고유 구현의 메서드). 위 `positions`의 바운드에는 라이프타임이 없으므로 아무것도 캡처하지 않고, `it`이 살아 있어도 `scores`를 고칠 수 있었습니다. 대신 정말로 인자를 빌리는 반복자를 돌려주면 에러였습니다.

```rust
fn evens(numbers: &[i32]) -> impl Iterator<Item = i32> {
    numbers.iter().copied().filter(|n| n % 2 == 0) // 숨은 타입이 numbers를 빌림
}
```

2021에서는 E0700 `` hidden type for `impl Iterator<Item = i32>` captures lifetime that does not appear in bounds ``입니다(4.1절). "숨은 타입이 바운드에 나오지 않는 라이프타임을 캡처한다"는 뜻입니다. 2021 코드에서는 이 에러를 피하려고 `-> impl Iterator<Item = i32> + '_`처럼 바운드에 라이프타임을 억지로 적는 방법(Edition Guide는 outlives trick이라고 부릅니다)을 흔히 썼습니다.

**2024에서는.** `use<..>`(아래)를 적지 않으면 **스코프에 있는 제네릭 매개변수를 라이프타임까지 모두 캡처**합니다. 그래서 `evens`는 아무것도 덧붙이지 않아도 컴파일되고, `positions`는 `numbers`를 캡처하므로 위 호출이 E0502가 됩니다. 2024 컴파일러는 이 에러에 "Rust 2024가 캡처 규칙을 바꿨기 때문일 수 있다"는 note와 `use<>`를 붙이라는 help를 덧붙여 줍니다(4.4절).

**왜 바꿨나.** Edition Guide가 드는 이유는 일관성입니다. 트레이트 구현 안의 메서드, 트레이트 정의 안의 RPIT, `async fn`이 돌려주는 퓨처는 원래 어느 에디션에서든 모든 라이프타임을 캡처했고, 일반 함수만 달랐습니다. 또 2021의 기본값은 가장 흔한 경우(인자를 빌리는 반복자를 돌려주기)에서 E0700을 냈고, 그것을 피하려던 `+ 'a` 방법은 타입 매개변수에 `T: 'a` 같은 불필요한 제약까지 요구해 정확하지 않았습니다. 2024는 흔한 경우를 기본값으로 삼고, 캡처하고 싶지 않은 드문 경우는 `use<..>`로 정확히 적게 했습니다.

**정밀 캡처 `use<..>`.** 불투명 타입의 바운드에 `use<..>`를 적으면 캡처할 매개변수를 직접 고릅니다. 이것을 **정밀 캡처(precise capturing)** 라고 부릅니다. Edition Guide에 따르면 rustc 1.82부터 **모든 에디션**에서 쓸 수 있고, 적으면 에디션별 기본 규칙 대신 이 목록이 쓰입니다([`01_rpit_capture.rs`](examples/01_rpit_capture.rs)).

| 쓰는 모양 | 뜻 | 예제 01 |
| --- | --- | --- |
| (안 적음) | 2024: 라이프타임까지 모두 캡처. 2021: 타입 매개변수와 바운드에 나온 라이프타임만 | 1·2번 구역 |
| `+ use<>` | 아무것도 캡처하지 않음(2021 기본 동작과 같음) | 3번 구역 |
| `+ use<'a>` | 라이프타임 `'a`만 캡처하고 다른 라이프타임은 캡처하지 않음 | 4번 구역 |
| `+ use<T>` | 타입 매개변수 `T`만 캡처 | 5번 구역 |
| `+ use<'_, T>` | 생략한 라이프타임과 `T`를 캡처 | |

규칙이 두 가지 있습니다. 첫째, 이 툴체인에서는 **타입 매개변수를 목록에서 뺄 수 없습니다.** `fn first_n<T: Clone>(items: &[T]) -> impl Iterator<Item = T> + use<>`는 두 에디션 모두에서 `` `impl Trait` must mention all type parameters in scope in `use<...>` `` 에러입니다. 둘째, 목록에서 뺀 라이프타임을 숨은 타입이 실제로 쓰면 두 에디션 모두 E0700이고, 컴파일러가 `` add `'_` to the `use<...>` bound `` 같은 help를 줍니다.

**옮길 때.** 2021 코드의 RPIT를 그대로 두면 2024에서 캡처가 늘어날 수 있습니다. `impl_trait_overcaptures` 린트를 켜면 그런 곳을 2021 에디션에서 미리 알려 줍니다(기본 수준은 allow).

```sh
rustc --edition 2021 -W impl_trait_overcaptures compile_fail/e0502_rpit_captures_borrow.rs --out-dir /tmp/rust-p2-ch4
```

```text
warning: `impl Iterator<Item = usize>` will capture more lifetimes than possibly intended in edition 2024
  --> compile_fail/e0502_rpit_captures_borrow.rs:16:34
...
note: specifically, this lifetime is in scope but not mentioned in the type's bounds
...
help: use the precise capturing `use<...>` syntax to make the captures explicit
   |
16 | fn positions(numbers: &[i32]) -> impl Iterator<Item = usize> + use<> {
   |                                                              +++++++
```

린트가 제안하는 `+ use<>`를 붙이면 두 에디션에서 뜻이 같아집니다. 다만 Edition Guide는 (인자 위치 `impl Trait`가 있어 자동 수정이 안 되는 경우를 설명하며) `use<..>`를 붙이는 대신 캡처를 그대로 허용하는 것도 고려해 보라고 합니다. 나중에 숨은 타입이 그 라이프타임을 쓰게 바꿀 여지가 남기 때문입니다. 부르는 쪽에서 실제로 에러가 나는지가 판단 기준입니다. 반대로 2021 코드의 `+ '_` 같은 방법은 2024에서도 그대로 동작하지만 자동으로 지워 주지는 않으므로, 손으로 지우거나 `use<..>`로 바꿉니다.

### 2.4 `if let` 조건식의 임시값 범위

[Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.10절에서 "`match`, `if let`, `while let`의 조건식(scrutinee)에서 만든 임시값은 본문이 끝날 때까지 살아 있다"고 보았습니다. 그 표의 `if let` 줄에만 에디션 차이가 있었습니다.

**2021에서는.** `if let`의 조건식에서 만든 임시값은 `match`와 같은 규칙을 따라 **`else` 블록까지 포함한 `if let` 전체**가 끝날 때 버려졌습니다.

**2024에서는.** then 블록이 끝나거나 **`else` 블록에 들어가기 전에** 버려집니다. then 블록에서는 그대로 살아 있습니다. 패턴이 묶은 값(`Some(first)`의 `first`)이 그 임시값을 빌리고 있을 수 있기 때문입니다. `else`에는 패턴이 묶은 값이 없으므로 임시값을 더 들고 있을 이유가 없습니다.

`projects/edition_compare`의 `if_let_scope`는 버려질 때 이름을 출력하는 `Noisy` 값으로 이 차이를 보여 줍니다. 같은 코드의 1번 구역 출력입니다.

```text
[2021 에디션]                          [2024 에디션]
1. 패턴이 맞지 않아 else로 갈 때         1. 패턴이 맞지 않아 else로 갈 때
    else 블록                              [drop] 조건식의 임시값
    [drop] 조건식의 임시값                  else 블록
    if let 다음 문장                        if let 다음 문장
```

**왜 바꿨나.** 임시값이 대여 가드나 잠금일 때 `else`에서 같은 대상을 다시 빌리면 문제가 됩니다. Edition Guide의 예는 읽기 잠금으로 값을 확인하고, 없으면 `else`에서 쓰기 잠금을 얻는 코드입니다.

```rust
if let Some(x) = *value.read().unwrap() {
    println!("value is {x}");
} else {
    let mut v = value.write().unwrap(); // 2021: 읽기 잠금이 아직 잡혀 있음
    // ...
}
```

2021에서는 `else`에서도 조건식의 읽기 잠금이 살아 있으므로, `write()`가 자기 자신이 쥔 잠금을 기다리게 됩니다. Edition Guide는 이것을 교착 상태라고 설명하고, `RwLock::write` 문서는 현재 스레드가 이미 잠금을 쥐고 있으면 패닉할 수도 있다고 적고 있습니다. 어느 쪽이든 버그입니다. 프로젝트의 4·5번 구역은 실제로 멈추지 않도록 `try_borrow_mut()`과 `try_write()`로 관찰합니다.

| `if_let_scope`의 구역 | 2021 에디션 | 2024 에디션 |
| --- | --- | --- |
| 4. `RefCell`: else에서 `try_borrow_mut()` | 실패(조건식의 `Ref`가 살아 있음) | 성공 |
| 5. `RwLock`: else에서 `try_write()` | 실패: `try_lock failed because the operation would block` | 성공 |

[`02_if_let_scope.rs`](examples/02_if_let_scope.rs)는 이 규칙 덕분에 2024에서 되는 코드를 보여 줍니다. `if let Some(&cached) = memo.borrow().get(&n) { .. } else { .. memo.borrow_mut().insert(..) }` 모양의 메모이제이션이 그대로 동작합니다. 같은 함수를 2021 에디션으로 컴파일해 실행하면 `RefCell already borrowed`로 패닉하는 것을 확인했습니다. Part 1 ch7 4.5절이 `match`로 쓴 같은 모양에서 만난 패닉입니다.

**`match`는 왜 그대로인가.** `match`의 조건식에서 만든 임시값은 어느 에디션에서든 `match` 전체가 끝날 때까지 삽니다. `match`에는 "else"에 해당하는 특별한 갈래가 없고, 어느 갈래든 패턴으로 조건식 안의 값을 빌릴 수 있기 때문입니다. 예제 02의 3번 구역에서 빌린 값이 없는 `None` 갈래에서도 가드가 살아 있는 것을 확인할 수 있습니다. 그래서 Edition Guide는 "`if let`을 `match`로 바꾸면 언제나 2021의 동작이 유지된다"고 설명합니다.

**옮길 때.** 대부분의 코드는 고칠 것이 없습니다. 2024에서는 임시값이 더 일찍 버려질 뿐이고, 그 덕분에 위 버그가 사라집니다. 확인할 것은 `else` 블록이 **조건식의 임시값이 아직 살아 있기를 기대하는** 코드입니다. 예를 들어 잠금을 쥔 채로 `else`의 작업을 해야 다른 스레드가 끼어들지 못하는 코드라면, 2024에서는 잠금이 먼저 풀립니다. `if_let_rescope` 린트(기본 수준 allow)를 켜면 2021 에디션에서 그런 후보를 보여 주고, 2021 동작을 지키는 `match`로 바꾸자고 제안합니다. `projects/edition_compare`에서 `rustc --edition 2021 -W if_let_rescope shared/if_let_scope.rs --out-dir /tmp/rust-p2-ch4`로 확인한 핵심 줄입니다.

```text
warning: `if let` assigns a shorter lifetime since Edition 2024
  --> shared/if_let_scope.rs:32:8
...
   |                      this value has a significant drop implementation which may observe a major change in drop order and requires your discretion
...
help: a `match` with a single arm can preserve the drop order up to Edition 2021
```

두 가지를 조심합니다. 첫째, Edition Guide의 말대로 제안을 그대로 받아들여 `match`로 바꾸면 위의 읽기·쓰기 잠금 버그까지 그대로 남습니다. 바뀐 곳마다 어느 동작이 필요한지 판단합니다. 둘째, 이 툴체인에서 이 린트는 1·2번(`Noisy(..).find(..)`, 임시값 자신의 메서드)과 5번(`*flag.read().unwrap()`, 역참조) 구역은 알려 줬지만 **4번 구역 `cache.borrow().first()`는 알려 주지 않았습니다.** `RefCell`이라서가 아니라 조건식의 모양 때문입니다. 가드를 `Deref`로 거쳐 메서드(`first`, `len`, `get`)를 부르는 모양은 경고가 나지 않았고, 같은 `RefCell`이라도 `*c.borrow()`(역참조)나 `c.borrow()[0]`(색인)은 경고가 났습니다. `Mutex`도 마찬가지로 `m.lock().unwrap().get(&k)`는 조용하고 `*m.lock().unwrap()`, `m.lock().unwrap()[0]`은 경고했습니다. 린트가 모든 차이를 잡지는 않으므로, 대여 가드와 잠금을 조건식에서 만드는 `if let`은 직접 훑어봅니다. 모양별 정리는 [ch6](../ch6-migration-practice/README.md) 2.7절에 있습니다.

### 2.5 let 체인

[Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.2절에서 `if`와 `while`의 조건에서 `let` 패턴과 `bool` 조건을 `&&`로 잇는 let 체인(let chains)을 배웠습니다.

**2021에서는.** `let`은 `if`나 `while` 바로 뒤에 하나만 올 수 있었습니다(`if let`, `while let`). let 체인을 쓰면 `let`이 나오는 자리마다 `let chains are only allowed in Rust 2024 or later` 에러가 납니다. 에러 전문과 읽는 법은 [ch1](../ch1-editions-overview/README.md) 4.4절(`compile_fail/let_chains_2021.rs`)에 있습니다.

**2024에서는.** `let`을 하나 이상, `bool` 조건과 섞어 `&&`로 이을 수 있습니다. `||`로는 이을 수 없습니다(Part 0 ch18 참고).

**왜 2024 전용인가.** 새 문법을 추가할 뿐인데 에디션을 나눈 이유를 Edition Guide는 "`if let` 임시값 범위 변경이 필요하기 때문"이라고 설명합니다. 체인에서는 고리마다 임시값이 생기고, 중간 고리가 실패하면 그때까지 만든 임시값을 정리한 뒤 `else`로 가야 합니다. 2.4절의 2024 규칙(else 전에 버림)이 있어야 이 순서가 깔끔하게 정해집니다. [`03_let_chains.rs`](examples/03_let_chains.rs)의 2번 구역에서 둘째 고리가 실패하면, 만든 순서의 반대로(둘째, 첫째) 임시값을 모두 버리고 나서 `else`에 들어가는 것을 볼 수 있습니다(3절의 출력).

**`while` 조건의 함정.** 조건의 고리는 왼쪽부터 차례로 평가되고, 앞 고리의 부수 효과는 뒤 고리가 실패해도 되돌려지지 않습니다.

```rust
let mut stack = vec![1, 2, 3, 4, 5];
while let Some(top) = stack.pop()
    && top > 2
{
    println!("    처리: {top}");
}
println!("    남은 스택: {stack:?}"); // [1] — 2는 꺼냈지만 처리하지 않고 사라짐
```

`top`이 2일 때 `top > 2`가 거짓이 되어 반복이 끝나지만, `pop()`은 이미 2를 꺼낸 뒤입니다. 꺼낸 값은 패턴 변수 `top`에만 있다가 그대로 버려집니다. 꺼내기 전에 `last()`로 확인하면 됩니다(예제 03의 4번 구역, 남은 스택 `[1, 2]`). 표준 라이브러리의 `Vec::pop_if`(1.86.0)처럼 조건을 확인한 뒤에만 꺼내는 메서드를 쓰는 방법도 있습니다.

**옮길 때.** 2021 코드에는 let 체인이 없으므로 고칠 것이 없습니다. 2024로 옮긴 뒤 중첩된 `if let`을 체인으로 펴는 것은 선택입니다. 펼 때는 위 함정처럼 조건 안에 부수 효과가 있는 고리가 없는지 봅니다.

### 2.6 마지막 표현식의 임시값 범위

블록의 마지막 표현식(tail expression)은 세미콜론 없이 블록의 값이 되는 표현식입니다. 함수 본문의 마지막 줄도 그렇습니다.

**2021에서는.** 마지막 표현식에서 만든 임시값이 **블록을 넘어 바깥 문장이 끝날 때까지** 살아 있었고, 그래서 그 블록의 지역 변수보다 **나중에** 버려졌습니다. 그 결과 다음 코드가 E0597이었습니다(4.2절).

```rust
fn count_names() -> usize {
    let names = RefCell::new(vec!["가", "나", "다"]);
    names.borrow().len() // 2021: E0597, 2024: 통과
}
```

`names.borrow()`가 만든 `Ref` 가드가 `names`보다 나중에 버려지는데, 가드는 버려질 때 `names`의 대여 개수를 고치므로 이미 사라진 `names`를 건드리게 됩니다. `len()`은 `usize`를 돌려줄 뿐이라 대여가 끝난 것처럼 보이는데도 거부되니, 무척 당황스러운 에러였습니다.

**2024에서는.** 마지막 표현식의 임시값은 그 표현식을 평가한 직후, **그 블록의 지역 변수보다 먼저** 버려집니다. 프로젝트의 `tail_expr_scope`로 순서를 비교합니다.

```text
[2021 에디션]                          [2024 에디션]
1. 함수 본문의 마지막 표현식             1. 함수 본문의 마지막 표현식
    [drop] 지역 변수                        [drop] 마지막 표현식의 임시값
    [drop] 마지막 표현식의 임시값            [drop] 지역 변수
    돌려받은 값: 12                         돌려받은 값: 12
```

**왜 바꿨나.** 위의 `RefCell` 예처럼 "블록이 끝났는데 블록 안의 값이 아직 빌려져 있다"는 2021의 규칙은 직관에 어긋났습니다. Edition Guide는 이 동작이 자주 놀라움을 준다고 설명합니다.

**범위가 좁아져서 생기는 에러.** 반대로 2021에서 블록 밖까지 살던 임시값에 기대던 코드는 2024에서 컴파일되지 않습니다(4.3절).

```rust
let len = { &String::from("1234") }.len(); // 2021: 통과, 2024: E0716
```

블록이 `&임시 String`을 내보내고 블록 밖의 `.len()`이 그것을 쓰는데, 2024에서는 블록이 끝나는 순간 `String`이 버려집니다. 블록의 결과를 `let`으로 묶으면 됩니다(예제 04의 3번 구역).

```rust
let text = { &String::from("1234") };
let len = text.len();
```

`let 변수 = &임시값;` 모양에서는 임시값의 수명이 변수만큼 늘어나는데, 이것을 **임시값 수명 연장(temporary lifetime extension)** 이라고 합니다. 블록의 마지막 표현식이 `&임시값`일 때도 이 규칙이 적용되어 `String`이 `text`와 함께 삽니다. 임시값 수명 규칙 전체는 Part 9 `ch5-statements-expressions`에서 다룹니다.

**옮길 때.** 버려지는 순서만 바뀌는 경우는 컴파일러가 아무 말도 하지 않습니다. 대부분은 상관없지만, 잠금 해제나 메시지 전송처럼 `Drop`에 부수 효과가 있으면 순서가 의미를 가질 수 있습니다. `tail_expr_drop_order` 린트(기본 수준 allow)가 2021 에디션에서 그런 곳을 알려 줍니다. 자동 수정은 없습니다. `projects/edition_compare`에서 `rustc --edition 2021 -W tail_expr_drop_order shared/tail_expr_scope.rs --out-dir /tmp/rust-p2-ch4`로 확인한 핵심 줄입니다.

```text
warning: relative drop order changing in Rust 2024
  --> shared/tail_expr_scope.rs:28:5
...
26 |     let _local = Noisy("지역 변수");
...
   |         `_local` will be dropped later as of Edition 2024
...
28 |     Noisy("마지막 표현식의 임시값").char_count()
...
   |     up until Edition 2021 `#1` is dropped last but will be dropped earlier in Edition 2024
```

### 2.7 `unsafe extern` 블록

[Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.4절에서 C 함수를 `unsafe extern "C" { .. }`로 선언했습니다.

**2021에서는.** 블록 앞에 `unsafe`가 없는 `extern "C" { .. }`였습니다. 2021 에디션은 이 모양을 아무 메시지 없이 받아들입니다. Rust By Example의 FFI 페이지처럼 오래된 자료는 대부분 이 모양입니다.

**2024에서는.** 블록 앞에 `unsafe`가 없으면 `extern blocks must be unsafe` 에러입니다(4.5절). 그리고 블록 안의 항목마다 `safe`나 `unsafe`를 붙일 수 있습니다.

```rust
unsafe extern "C" {
    safe fn sqrt(x: c_double) -> c_double;         // 부를 때 unsafe 블록이 필요 없음
    unsafe fn strlen(s: *const c_char) -> usize;   // 부를 때 unsafe 블록 필요
    fn abs(x: c_int) -> c_int;                     // 표시가 없으면 unsafe fn
}
```

**왜 바꿨나.** 컴파일러는 선언이 실제 C 함수와 맞는지(이름, 매개변수 타입, 반환 타입) 확인할 수 없습니다. 틀리면 미정의 동작입니다. 그 책임이 **선언을 쓴 사람**에게 있다는 것을 블록의 `unsafe`로 드러냅니다. 2021에서는 부르는 곳의 `unsafe { }`만 있었기 때문에, 위험이 선언에 있다는 것이 코드에 보이지 않았습니다. `safe fn`은 "어떤 인자로 불러도 안전하다"는 보증을 선언하는 쪽에서 한 번 하고, 부르는 곳마다 `unsafe` 블록을 쓰지 않게 해 줍니다.

**옮길 때.** 블록 앞에 `unsafe`를 붙이면 끝입니다. Edition Guide에 따르면 rustc 1.82부터 모든 에디션에서 `unsafe extern`을 쓸 수 있으므로, 2021 에디션 코드에서 먼저 고쳐 둘 수 있습니다(`projects/edition_compare/shared/new_syntax.rs`가 두 에디션에서 같은 출력을 냅니다). 다만 `safe`/`unsafe` 표시는 블록에 `unsafe`가 있을 때만 쓸 수 있습니다. `unsafe` 없는 블록 안에 `safe fn`을 쓰면 두 에디션 모두 `` items in `extern` blocks without an `unsafe` qualifier cannot have safety qualifiers `` 에러입니다. 자동 수정(`missing_unsafe_on_extern` 린트)은 `unsafe`를 붙여 줄 뿐 선언이 맞는지는 확인하지 않으므로, 선언은 사람이 다시 확인합니다.

### 2.8 unsafe 속성

**2021에서는.** `#[no_mangle]`, `#[export_name = "..."]`, `#[link_section = "..."]`를 그냥 썼습니다.

**2024에서는.** 이 세 속성은 `#[unsafe(no_mangle)]`처럼 `unsafe(...)`로 감싸야 합니다. 감싸지 않으면 `unsafe attribute used without unsafe` 에러이고, 컴파일러가 `` wrap the attribute in `unsafe(...)` ``라는 help를 줍니다(4.5절).

**왜 바꿨나.** 세 속성은 링커가 보는 **심볼(symbol)** 의 이름이나 위치를 정합니다. 링크되는 모든 라이브러리가 심볼 이름 공간 하나를 함께 쓰므로, 이름이 겹치면 안전한 코드만으로도 프로그램이 망가질 수 있습니다. Edition Guide는 `#[export_name = "malloc"]`을 붙인 안전한 함수 하나 때문에 대부분의 유닉스 계열 플랫폼에서 프로그램이 죽는 예를 듭니다. 보통 함수는 이름 맹글링(Part 0 ch19 2.4절)으로 이름이 겹치지 않지만, 이 속성들은 그 보호를 끕니다. 그래서 "이 이름이 겹치지 않는다"는 보증을 `unsafe(...)`와 `SAFETY:` 주석으로 남깁니다.

```rust
// SAFETY: 이 프로그램에 `p2ch4_renamed`라는 이름의 다른 심볼은 없습니다.
#[unsafe(export_name = "p2ch4_renamed")]
pub extern "C" fn answer() -> c_int {
    42
}
```

[`05_unsafe_changes.rs`](examples/05_unsafe_changes.rs)의 2번 구역은 이렇게 정한 심볼 이름만 알고 `unsafe extern` 블록에 선언해 다시 불러서, 이름이 실제로 그렇게 정해졌다는 것을 확인합니다. `link_section`은 섹션 이름 형식이 플랫폼마다 달라서 예제에 넣지 않았습니다. 이 환경(macOS)에서 `#[link_section = "__DATA,__ch4demo"]`도 2024에서는 같은 에러가 나고, `#[unsafe(link_section = "__DATA,__ch4demo")]`로 감싸면 두 에디션에서 컴파일되는 것을 확인했습니다.

**옮길 때.** 감싸기만 하면 됩니다. unsafe 속성 문법도 rustc 1.82부터 모든 에디션에서 쓸 수 있습니다(Edition Guide). 감쌀 때 `SAFETY:` 주석으로 이름이 겹치지 않는 이유를 적어 둡니다.

### 2.9 `static mut` 참조 금지

[Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.5절에서 2024 에디션은 `static mut`의 참조를 만들지 못하게 한다고 배웠습니다. 여기서는 에디션별 수준과 대안을 정리합니다.

**2021에서는 경고, 2024에서는 에러.** 둘 다 `static_mut_refs` 린트이고, 기본 수준만 다릅니다(2021은 warn, 2024는 deny). 다음 코드를 `static_mut.rs`로 저장해 두 에디션으로 컴파일했습니다.

```rust
static mut LOG: Vec<&str> = Vec::new();
static mut COUNTER: u32 = 0;

fn main() {
    unsafe {
        COUNTER += 1; // 값을 읽고 쓰기만 함: 참조가 아님
        let copied = COUNTER; // 값 복사: 참조가 아님
        LOG.push("시작"); // push(&mut self)가 &mut LOG를 몰래 만듦
        println!("{copied} {}", LOG.len()); // len(&self)가 &LOG를 몰래 만듦
    }
}
```

```text
error: creating a mutable reference to mutable static
 --> static_mut.rs:8:9
...
  = note: mutable references to mutable statics are dangerous; it's undefined behavior if any other pointer to the static is used or if any other reference is created for the static while the mutable reference lives
...
  = note: `#[deny(static_mut_refs)]` (part of `#[deny(rust_2024_compatibility)]`) on by default

error: creating a shared reference to mutable static
 --> static_mut.rs:9:33
```

2021 에디션에서는 같은 두 곳이 `warning:`으로 나오고, 마지막 note가 `` `#[warn(static_mut_refs)]` (part of `#[warn(rust_2024_compatibility)]`) on by default ``로 바뀝니다. `COUNTER += 1`과 `let copied = COUNTER`는 두 에디션 모두 문제없습니다. **값을 읽고 쓰는 것**이 아니라 **참조를 만드는 것**이 금지 대상입니다. 메서드 호출(`LOG.push`, `LOG.len`)과 `println!`처럼 `&`가 보이지 않아도 참조를 만드는 곳이 걸린다는 점을 기억합니다.

**왜 바꿨나.** Edition Guide는 `static mut`의 참조를 만드는 것만으로, 그 참조를 읽거나 쓰지 않아도 "가변 참조는 하나, 또는 불변 참조는 여럿" 규칙을 어기면 즉시 미정의 동작이라고 설명합니다. 그리고 `static mut`에 대해 그 규칙이 지켜지는지 확인하려면 프로그램 전체(재진입, 여러 스레드)를 따져야 해서 무척 어렵습니다.

**대안.** 예제 05의 4번 구역이 세 가지를 보여 줍니다.

| 방법 | 쓰는 곳 | unsafe |
| --- | --- | --- |
| `AtomicU32` 같은 원자 타입을 `mut` 없는 `static`에 ([Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 2.8절) | 정수·불리언 하나 | 필요 없음 |
| `Mutex`/`RwLock`을 `mut` 없는 `static`에 | 더 복잡한 값 | 필요 없음 |
| `OnceLock`/`LazyLock` (Part 1 ch11 2.9절) | 한 번만 초기화하는 전역 값 | 필요 없음 |
| `&raw const X` / `&raw mut X`로 원시 포인터만 다루기 | C 함수에 포인터를 넘기는 등 `static mut`를 꼭 써야 할 때 | 역참조에 필요 |

`&raw mut TICKS`처럼 원시 포인터를 **만드는 것**은 안전한 코드이고, 그 포인터로 읽고 쓰는 것만 `unsafe` 블록 안에서 합니다. 참조를 거치지 않으므로 `static_mut_refs`에 걸리지 않습니다. 다만 Edition Guide가 강조하듯 원시 포인터로 바꿔도 동시에 접근하지 않게 하는 책임은 그대로 프로그래머에게 있습니다.

**옮길 때.** 자동 수정이 없습니다. 2021 에디션에서 이미 경고가 나므로 경고가 나는 곳을 위 표의 방법으로 하나씩 바꿉니다. `&raw const`/`&raw mut`과 원자 타입·`Mutex`는 모두 2021 에디션에서도 쓸 수 있습니다.

### 2.10 never 타입 폴백

[Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.14절에서 never 타입 `!`가 `continue`, `return`, `panic!()`처럼 흐름을 끊는 표현식의 타입이고, 어떤 타입으로든 바뀔 수 있다고 배웠습니다. 예제 06의 1번 구역에서 `Err(_) => continue`가 `u32` 자리에 들어가는 것이 그 예입니다.

> Part 0 ch19 2.14절은 "`!`를 타입으로 직접 적을 수 있는 곳은 함수의 반환 타입뿐"이라고 설명했습니다. 그것은 rustc 1.98.1 기준이었습니다. rustc 1.100.0의 표준 라이브러리 문서는 never 타입을 "Stable since Rust version 1.100.0"으로 표시하고, 실제로 `Result<u32, !>`, `Vec<!>`를 쓸 수 있습니다(예제 06의 4번 구역). `std::convert::Infallible`은 `!`와 같은 타입입니다.

**폴백이란.** 컴파일러는 `!` 타입의 값이 다른 타입 자리에 오면 "아무 타입으로나 바꾸는" 변환을 끼워 넣습니다. 그런데 바꿀 타입을 정해 줄 단서가 전혀 없을 때가 있습니다.

```rust
fn return_type_of<R, F: FnOnce() -> R>(_f: F) -> &'static str {
    std::any::type_name::<R>()
}

return_type_of(|| panic!()) // R은 무엇인가?
```

클로저의 반환 타입 `R`을 정해 줄 단서가 `panic!()`(타입 `!`)밖에 없습니다. 이렇게 추론이 막히면 에러를 내는 대신 컴파일러가 정해진 타입 하나를 고르는데, 이 타입을 **폴백 타입(fallback type)** 이라고 하고 이 규칙을 **never 타입 폴백**이라고 부릅니다.

**원래 계획: 2021은 `()`, 2024는 `!`.** 오랫동안 폴백 타입은 `()`였습니다. 그래서 `!`가 아무 이유 없이 `()`로 바뀌는 일이 생겼고, Edition Guide는 이것이 혼란을 주고 `!` 타입의 안정화를 막아 왔다고 설명합니다. 2024 에디션은 폴백을 `!`로 바꿨습니다. 이 저장소가 예전에 쓰던 rustc 1.98.1에서 위 코드는 2021 에디션일 때 `()`, 2024 에디션일 때 `!`를 출력했습니다.

**rustc 1.100.0의 실제 동작: 에디션과 상관없이 `!`.** 1.100.0의 Edition Guide에는 "이 절의 변경은 Rust 1.100부터 더 이상 에디션별 변경이 아니다. 모든 에디션에서 폴백이 `!`이고, 마이그레이션용 린트 `dependency_on_unit_never_type_fallback`도 제거됐다"는 안내가 붙어 있습니다. [ch1](../ch1-editions-overview/README.md) 2.4절이 "모든 에디션에 적용된 드문 예외"로 소개한 것이 이 변경입니다. 실제로 `projects/edition_compare`의 `never_fallback_2021`과 `never_fallback_2024`는 같은 출력을 냅니다. 위 코드는 두 에디션 모두 `R = !`이고, 단서가 있을 때(한 갈래가 `u8`인 `if`)는 `R = u8`, 직접 지정하면(`return_type_of::<(), _>(..)`) `R = ()`입니다.

```text
1. 반환 타입을 정할 단서가 없는 클로저
    || panic!()          → R = !
```

**폴백에 기대던 코드.** 폴백이 `()`라서 우연히 컴파일되던 코드는 이제 두 에디션 모두에서 실패합니다. Edition Guide가 드는 세 가지 모양을 1.100.0에서 2021과 2024로 컴파일해 보았고, 모두 같은 E0277이었습니다.

| 모양 | 에러 | 고치는 방법 |
| --- | --- | --- |
| `f()?;` (`f<T: Default>() -> Result<T, E>`, 결과를 버림) | `` the trait bound `!: Default` is not satisfied `` | `f::<()>()?;` 또는 `let () = f()?;` |
| `run(\|\| panic!())` (`run<R: Unit>(f: impl FnOnce() -> R)`, `Unit`은 `()`에만 구현) | `` the trait bound `!: Unit` is not satisfied `` | 클로저 반환 타입 적기: `\|\| -> () { panic!() }` |
| `if c { Default::default() } else { return };` | `` the trait bound `!: Default` is not satisfied `` | `<() as Default>::default()` |

첫 줄이 4.7절의 [`compile_fail/e0277_never_type_fallback.rs`](compile_fail/e0277_never_type_fallback.rs)입니다. 에러에는 `` help: you might have intended to use the type `()` here instead ``가 붙어 고칠 방향을 알려 줍니다. 예제 06의 3번 구역이 고친 모양(`load::<()>(true)?;`와 `let n: u32 = load(true)?;`)을 실행합니다.

**옮길 때.** 1.100.0 이후의 툴체인에서는 에디션을 올리는 것과 상관없이 고쳐야 합니다. 원리는 하나입니다. **폴백이 쓰이지 않도록 타입을 직접 적습니다.**

### 2.11 `gen` 키워드

**2021에서는.** `gen`은 평범한 식별자였습니다. 함수, 메서드, 변수 이름으로 쓸 수 있었습니다.

**2024에서는.** `gen`은 **예약 키워드**입니다. 이름으로 쓰면 `` expected identifier, found reserved keyword `gen` `` 에러이고, 정의하는 곳과 부르는 곳 모두에서 납니다(에러 전문은 [ch1](../ch1-editions-overview/README.md) 4.2절의 `compile_fail/reserved_keyword_gen.rs`). raw 식별자 `r#gen`으로 쓰면 됩니다(raw 식별자는 [Part 2 ch2](../ch2-rust-2015-2018/README.md)에서 배웠습니다).

```rust
impl Counter {
    fn r#gen(&mut self) -> u32 { .. }
}
let r#gen = counter.r#gen();
println!("{gen}"); // 서식 문자열 안에서는 r# 없이 씁니다
```

서식 문자열 안에서는 반대로 `{r#gen}`이 에러입니다(`invalid format string: raw identifiers are not supported`). 컴파일러는 `` identifiers in format strings can be keywords and don't need to be prefixed with `r#` ``라고 알려 줍니다. 예제 07의 1번 구역이 이 모양으로 출력합니다.

**왜 예약했나.** 반복자를 쉽게 만드는 `gen` 블록을 앞으로 도입하기 위해서입니다(Edition Guide). 키워드를 미리 예약해 두면 다음 에디션을 기다리지 않고 `gen` 블록을 안정화할 수 있습니다. rustc 1.100.0에서 `gen { yield 1; }`은 2024 에디션에서도 `error[E0658]: gen blocks are experimental`입니다. 2024 에디션이 바꾼 것은 **키워드 예약뿐**이고, `gen` 블록 자체는 Part 11에서 다룹니다.

**실제 사례.** 난수 크레이트 `rand`가 이 변경 때문에 메서드 이름을 바꿨습니다. `rand` 0.9.0의 CHANGELOG에는 "Rename fn `Rng::gen` to `random` to avoid conflict with the new `gen` keyword in Rust 2024"라는 항목이 있고, `gen_range`도 같은 버전에서 `random_range`로 바뀌었습니다. [Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md)에서 쓴 `rand::rng().random_range(1..=100)`이 바뀐 뒤의 이름입니다. 0.9에서는 옛 이름을 `r#gen`이라는 deprecated 별칭으로 남겨 두었고, Part 0 ch2가 쓰는 0.10.4에는 그 별칭이 없습니다(로컬에 받아 둔 각 버전의 소스로 확인).

**옮길 때.** `keyword_idents_2024` 린트(2021 기본 수준 allow)가 `gen` 식별자를 찾아 `r#gen`으로 고치자고 합니다. ch1의 파일에 이 린트를 켜서 2021로 컴파일하면 다음과 같습니다(이 챕터 디렉터리에서 실행).

```text
$ rustc --edition 2021 -W keyword_idents_2024 ../ch1-editions-overview/compile_fail/reserved_keyword_gen.rs --out-dir /tmp/rust-p2-ch4
warning: `gen` is a keyword in the 2024 edition
  --> ../ch1-editions-overview/compile_fail/reserved_keyword_gen.rs:15:4
...
   |    ^^^ help: you can use a raw identifier to stay compatible: `r#gen`
...
   = warning: this is accepted in the current edition (Rust 2021) but is a hard error in Rust 2024!
```

라이브러리라면 `r#gen`으로 바꾸는 것만으로는 쓰는 사람이 불편하므로, `rand`처럼 새 이름을 짓고 옛 이름은 deprecated로 남기는 방법도 생각합니다.

### 2.12 그 밖의 변경

나머지 다섯 항목입니다. 모두 이 툴체인에서 확인한 코드 조각과 함께 짧게 정리합니다.

#### 패턴의 `mut`/`ref`/`&` 제한 (Match ergonomics reservations)

참조를 `&` 없는 패턴으로 맞추면 안쪽 바인딩이 자동으로 참조가 됩니다(기본 바인딩 모드, [Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.6절). 2021에서는 그 안에 `mut`을 쓰면 그 바인딩만 값으로 복사·이동하는 이상한 동작이 있었습니다. `mut`이 타입까지 바꾸는 셈입니다.

```rust
let pair = &(1, 2);
let (x, mut y) = pair; // 2021: x는 &i32, y는 i32(복사) → 출력 "1 12"
y += 10;
```

2024에서는 이렇게 암묵적으로 빌리는 패턴 안에서 `mut`, `ref`, `ref mut`, `&`, `&mut`을 쓰면 에러입니다. 규칙을 나중에 더 일관되게 바꿀 자리를 남겨 두려는 것이라고 Edition Guide는 설명합니다. 위 코드를 `patterns.rs`의 `main`에 넣어 2024로 컴파일한 결과입니다.

```text
error: cannot mutably bind by value within an implicitly-borrowing pattern
 --> patterns.rs:3:13
...
  |             ^^^ `mut` binding modifier not allowed when implicitly borrowing
...
help: match on the reference with a reference pattern and borrow explicitly using a variable binding mode
  |
3 |     let &(ref x, mut y) = pair; // 2021: x는 &i32, y는 i32(복사)
```

고치는 방법은 help대로 바깥 `&`부터 모두 적은 **완전히 명시한 패턴(fully explicit pattern)** 으로 바꾸는 것입니다. 이 모양은 두 에디션에서 같은 뜻입니다(예제 07의 2번 구역). `ref`만 쓸데없이 붙은 경우(`cannot explicitly borrow within an implicitly-borrowing pattern`, Part 0 ch18 2.6절에서 본 에러)는 `ref`를 지우면 됩니다. `rust_2024_incompatible_pat` 린트가 자동으로 고쳐 줍니다. 바인딩 모드 규칙의 상세는 Part 9 `ch6-patterns`에서 다룹니다.

#### `unsafe_op_in_unsafe_fn` 경고

2021에서는 `unsafe fn`의 본문 전체가 `unsafe` 블록처럼 취급되어, 원시 포인터 역참조 같은 unsafe 연산을 아무 표시 없이 쓸 수 있었습니다. 2024에서는 `unsafe_op_in_unsafe_fn` 린트가 기본으로 경고합니다(2021은 allow). 이 툴체인의 경고 문구입니다.

```text
warning[E0133]: dereference of raw pointer is unsafe and requires unsafe block
...
note: an unsafe function restricts its caller, but its body is safe by default
...
   = note: `#[warn(unsafe_op_in_unsafe_fn)]` (part of `#[warn(rust_2024_compatibility)]`) on by default
```

`unsafe fn`의 `unsafe`는 "부르는 쪽이 조건을 지켜야 한다"는 뜻과 "본문에서 unsafe 연산을 해도 된다"는 뜻을 함께 가지고 있었는데, 두 번째 역할이 너무 위험하다고 판단해 나눈 것입니다(Edition Guide). 이제 본문 안에서도 unsafe 연산마다 `unsafe { }`와 `SAFETY:` 주석을 답니다. 예제 05의 3번 구역에 두 모양이 있고, 옛 모양에는 경고가 실제로 난다는 것을 `#[expect(unsafe_op_in_unsafe_fn)]`으로 확인합니다. `unsafe fn` 안의 `unsafe` 블록은 2021에서도 경고 없이 컴파일됩니다.

#### `expr` 프래그먼트 지정자

`macro_rules!`의 `$e:expr`는 2021까지 `const { .. }` 블록(1.79에서 추가)과 `_` 표현식(1.59에서 추가)을 받지 않았습니다. 새 문법을 기존 지정자가 받게 하면 기존 매크로가 다른 규칙을 고르게 될 수 있어서, 에디션 경계까지 미룬 것입니다. 2024에서는 `expr`가 둘 다 받고, 옛 동작이 필요하면 새 지정자 `expr_2021`을 씁니다. `projects/edition_compare`의 `macro_expr`가 같은 매크로를 두 에디션으로 비교합니다.

```text
[2021 에디션]                                [2024 에디션]
1. $e:expr                                    1. $e:expr
    1 + 1           → 1번 규칙($e:expr)            1 + 1           → 1번 규칙($e:expr)
    const { 1 + 1 } → 2번 규칙(const 블록)         const { 1 + 1 } → 1번 규칙($e:expr)
    _               → 3번 규칙(밑줄)               _               → 1번 규칙($e:expr)
```

컴파일 에러 없이 **고르는 규칙이 조용히 달라지는** 변경이라는 점이 중요합니다. `edition_2024_expr_fragment_specifier` 린트의 자동 수정은 모든 `expr`를 `expr_2021`로 바꿔 동작을 보존합니다. Edition Guide는 대부분의 매크로에는 새 `expr`가 더 낫다며, 충돌하는 규칙이 없는지 확인한 뒤 되돌리라고 권합니다. `expr_2021`은 2021 에디션에서도 쓸 수 있습니다(프로젝트의 2번 구역). 매크로 프래그먼트의 상세는 Part 9 `ch2-macros`에서 다룹니다.

#### 빠진 프래그먼트 지정자

매크로 규칙에 `$name`처럼 지정자 없이 메타변수를 쓰면, 그 규칙이 쓰이지 않더라도 에러입니다. Edition Guide에 따르면 처음에는 2024 에디션에서만 에러였다가 rustc 1.89부터 모든 에디션에서 에러가 됐습니다. 이 툴체인에서 두 에디션 모두 같은 에러를 냈습니다.

```text
error: missing fragment specifier
 --> fragment.rs:3:6
  |
3 |     ($name) => {}; // 프래그먼트 지정자가 없음
  |      ^^^^^
  |
  = note: fragment specifiers must be provided
```

자동 수정은 없습니다. 쓰이지 않는 규칙이라면 지우고, 쓰는 규칙이라면 `$name:ident`처럼 지정자를 붙입니다.

#### 예약 문법 `#"..."#`와 `##`

2021 에디션은 `ident##"foo"##`처럼 접두사가 붙은 모양을 예약했습니다([Part 2 ch3](../ch3-rust-2021/README.md)). 2024는 접두사 없는 `#"foo"#`(앞에 `#`가 하나 이상 붙은 문자열 리터럴)와, 공백 없이 이어진 `#` 두 개 이상(`##`)을 앞으로의 문법을 위해 예약합니다. 2021에서는 `#"foo"#`가 토큰 세 개(`#`, `"foo"`, `#`)였으므로 매크로 입력으로 쓸 수 있었지만, 2024에서는 `invalid string literal`과 `reserved multi-hash token is forbidden` 에러입니다(4.6절). help대로 공백을 넣어 `# "foo" #`, `# # #`로 쓰면 예전처럼 토큰 세 개가 됩니다. raw 문자열 `r#"foo"#`는 영향이 없습니다(예제 07의 4번 구역). `rust_2024_guarded_string_incompatible_syntax` 린트가 공백을 넣어 줍니다.

### 2.13 이 챕터에서 다루지 않는 것

| 주제 | 한 줄 소개 | 다루는 곳 |
| --- | --- | --- |
| 에디션의 개념, 에디션이 다른 크레이트 함께 쓰기 | 에디션 제도 자체 | [Part 2 ch1](../ch1-editions-overview/README.md) |
| 2024의 표준 라이브러리·Cargo·rustdoc·rustfmt 변경 | `set_var`의 unsafe화, 프렐류드의 `Future`/`IntoFuture`, `Box<[T]>`의 `IntoIterator`, 스타일 에디션 등 | [Part 2 ch5](../ch5-rust-2024-ecosystem/README.md) |
| `cargo fix --edition`과 에디션 호환성 린트 그룹(`rust-2024-compatibility`)으로 옮기는 절차 | 이 챕터의 린트들을 한 번에 적용하기 | [Part 2 ch6](../ch6-migration-practice/README.md) |
| 기본 바인딩 모드 규칙 상세 | 패턴이 참조를 맞출 때의 규칙 | Part 9 `ch6-patterns` |
| 매크로 프래그먼트 지정자 전체 | `expr`, `tt`, `pat_param` 등 | Part 9 `ch2-macros` |
| 임시값 수명 규칙 전체 | 임시값 범위, 수명 연장, 승격 | Part 9 `ch5-statements-expressions` |
| `gen` 블록 | 반복자를 만드는 불안정 기능 | Part 11 |
| unsafe와 FFI 깊이 보기, Miri | 원시 포인터 규칙, C와 소유권 주고받기 | Part 10 |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_rpit_capture.rs`](examples/01_rpit_capture.rs) | 2024의 RPIT 캡처, `use<>`·`use<'a>`·`use<T>` | 1 |
| [`02_if_let_scope.rs`](examples/02_if_let_scope.rs) | `if let` 임시값이 else 전에 버려짐, 메모이제이션, `match`와의 차이, 읽기 잠금 후 쓰기 잠금 | 2 |
| [`03_let_chains.rs`](examples/03_let_chains.rs) | let 체인, 고리마다의 임시값 정리 순서, `while` 조건의 함정과 고친 모양 | 3 |
| [`04_tail_expr_scope.rs`](examples/04_tail_expr_scope.rs) | 마지막 표현식의 임시값이 지역 변수보다 먼저 버려짐, 지역 `RefCell`, 범위가 좁아져 고친 코드 | 2 |
| [`05_unsafe_changes.rs`](examples/05_unsafe_changes.rs) | `unsafe extern`과 `safe`/`unsafe` 항목, unsafe 속성, unsafe 함수 안의 unsafe 블록, `static mut` 대안 | 4 |
| [`06_never_type.rs`](examples/06_never_type.rs) | `!`의 변환, 폴백 타입, 폴백에 기대던 코드 고치기, 1.100.0의 `!` 타입 | 5 |
| [`07_other_changes.rs`](examples/07_other_changes.rs) | `r#gen`, 완전히 명시한 패턴, `expr`/`expr_2021`, 예약 문법 | 5 |
| [`projects/edition_compare`](projects/edition_compare/Cargo.toml) | 같은 소스 파일 다섯 개를 2021·2024 에디션으로 각각 실행 | 2, 4, 5 |
| [`compile_fail/`](compile_fail) | 한쪽 에디션에서만(또는 1.100.0에서 두 에디션 모두) 컴파일되지 않는 코드 7개(4절) | 1~5 |

예제는 모두 저장소 루트에서 실행하고, 이 저장소의 기본 에디션인 2024 에디션으로 컴파일됩니다. 모두 패닉 없이 종료 코드 0으로 곧바로 끝나고, 실행할 때마다 같은 출력이 나옵니다.

### 01_rpit_capture — RPIT 캡처와 `use<..>`

```sh
cargo run -p p2-ch4-rust-2024-language --example 01_rpit_capture
```

```text
1. 인자를 빌리는 반복자는 그대로 됩니다
    evens = [2, 4, 6]
2. 캡처했으므로, 반복자를 다 쓴 뒤에야 원본을 고칠 수 있습니다
    positions = [0, 1], scores = [70, 85, 90]
3. use<>로 캡처하지 않으면 반복자가 살아 있어도 원본을 고칠 수 있습니다
    positions_detached = [0, 1, 2], scores = [70, 85, 90, 95]
4. use<'a>: 두 인자 중 items만 캡처합니다
    - 사과
    - 배
    prefix는 이제 "- >"
5. use<T>: 타입 매개변수는 적고, 라이프타임은 뺍니다
    echo = ["안녕", "안녕"], words = []
```

- **2번 구역**: 반복자를 `collect`로 다 쓴 다음에 `push`합니다. 순서를 바꾸면 4.4절의 E0502입니다.
- **3번 구역**: 반복자는 만들 때의 길이(3)를 기억하므로 나중에 넣은 95의 위치(3)는 나오지 않습니다.
- **4번 구역**: `prefixed<'a>(items: &'a [String], prefix: &str)`는 `prefix`를 복사해 두므로 `use<'a>`로 `items`만 캡처합니다. `use<'a>`를 지우면 2024에서는 `prefix`까지 캡처해 `prefix.push('>')`가 E0502이고, 2021에서는 바운드에 `'a`가 나오지 않아 E0700(`` help: add a `use<...>` bound to explicitly capture `'a` ``)입니다. 어느 에디션에서든 `use<'a>`가 답입니다.

### 02_if_let_scope — `if let` 임시값 범위

```sh
cargo run -p p2-ch4-rust-2024-language --example 02_if_let_scope
```

```text
1. 임시값이 버려지는 시점
    [drop] 조건식의 임시값
    else 블록
2. else에서 같은 RefCell을 다시 빌리기: 메모이제이션
    fib(50) = 12586269025
    메모에 저장된 값: 49개
3. then 블록과 match에서는 가드가 끝까지 살아 있습니다
    if let의 then 블록: 첫 줄 "시작", 가변 대여 가능? false
    match의 None 갈래: 가변 대여 가능? false
4. 읽기 잠금으로 확인하고, 없으면 else에서 쓰기 잠금
    else에서 기본값을 썼습니다
    설정: Some("기본값")
```

- **2번 구역**: `fib`의 `if let` 조건식에서 `memo.borrow()`로 빌리고, `else`에서 재귀 호출 뒤 `memo.borrow_mut()`으로 넣습니다. 2부터 50까지 49개가 저장됩니다. 2021 에디션이라면 `RefCell already borrowed`로 패닉합니다(같은 함수를 2021로 컴파일해 확인).
- **4번 구역**: 2024에서는 `write()`를 그냥 불러도 됩니다. 2021이라면 이 줄에서 멈추거나 패닉할 수 있으므로, 2021과 비교할 때는 `try_write()`로 관찰하는 프로젝트의 `if_let_scope`를 씁니다.

### 03_let_chains — let 체인과 `while` 조건의 함정

```sh
cargo run -p p2-ch4-rust-2024-language --example 03_let_chains
```

```text
1. let과 bool 조건을 &&로 잇기
    "3, 4" → Some((3, 4))
    "3, 넷" → None
    "5, 4" → None
    "34" → None
2. 고리가 실패하면, 그때까지 만든 임시값을 모두 버린 뒤 else로 갑니다
    평가: 첫째 고리의 임시값
    평가: 둘째 고리의 임시값
    [drop] 둘째 고리의 임시값
    [drop] 첫째 고리의 임시값
    else 블록
3. while 조건의 함정: 조건이 거짓이 되는 순간에도 pop은 이미 일어났습니다
    처리: 5
    처리: 4
    처리: 3
    남은 스택: [1]
4. 고친 모양: 꺼내기 전에 last()로 먼저 확인합니다
    처리: 5
    처리: 4
    처리: 3
    남은 스택: [1, 2]
```

- **1번 구역**: `parse_pair`는 `let` 세 개와 `x <= y` 조건을 잇습니다. `"3, 넷"`은 둘째 `parse`에서, `"5, 4"`는 마지막 `bool` 조건에서, `"34"`는 쉼표가 없어 첫 고리에서 실패합니다.

### 04_tail_expr_scope — 마지막 표현식의 임시값 범위

```sh
cargo run -p p2-ch4-rust-2024-language --example 04_tail_expr_scope
```

```text
1. 마지막 표현식의 임시값이 지역 변수보다 먼저 버려집니다
    [drop] 마지막 표현식의 임시값
    [drop] 지역 변수
    돌려받은 값: 12
2. 그래서 지역 RefCell을 빌린 가드를 마지막 표현식에 둘 수 있습니다
    local_refcell_len() = 3
3. 범위가 좁아져서 고쳐야 하는 코드
    text = "1234", len = 4
```

- 1번 구역의 12는 `"마지막 표현식의 임시값"`의 글자 수(공백 포함)이고, 2번 구역의 `local_refcell_len`은 4.2절의 E0597 파일과 같은 모양입니다.

### 05_unsafe_changes — unsafe가 드러나야 하는 곳

```sh
cargo run -p p2-ch4-rust-2024-language --example 05_unsafe_changes
```

```text
1. unsafe extern 블록
    sqrt(2.25) = 1.5 (safe fn이라 unsafe 블록 없이 부름)
    strlen(c"Rust 2024") = 9, abs(-7) = 7
2. unsafe 속성으로 정한 심볼 이름으로 부르기
    p2ch4_triple(5) = 15, p2ch4_renamed() = 42
3. unsafe 함수 안의 unsafe 블록
    first = 10, last = 30
4. static mut 대신 쓰는 방법들
    &raw mut: 2, AtomicU32: 2, Mutex: ["시작", "끝"]
```

- **unsafe 블록의 위치**: `main`의 `strlen`·`abs` 호출, `read_i32` 두 번을 부르는 블록, `RAW_COUNTER`를 고치는 블록, 그리고 `read_i32` 안의 블록입니다. `read_i32_old_style`은 일부러 블록 없이 썼습니다. 블록마다 왜 안전한지 `SAFETY:` 주석을 달았습니다.
- **2번 구역**: `mod c_side`의 `unsafe extern` 블록은 심볼 이름만 알고 함수를 선언합니다. `export_name`으로 정한 `p2ch4_renamed`가 Rust 쪽 이름 `answer`와 다른데도 불린다는 것이 요점입니다. 이 선언은 위에서 정의한 함수와 시그니처가 같다는 것을 우리가 알기 때문에 `safe fn`으로 적었습니다.

### 06_never_type — never 타입과 폴백

```sh
cargo run -p p2-ch4-rust-2024-language --example 06_never_type
```

```text
1. 흐름을 끊는 표현식(타입 !)은 어떤 타입 자리에든 들어갑니다
    "42" → 42
    "사십이"는 숫자가 아니라 건너뜁니다
2. 폴백: 타입을 정할 단서가 없을 때 컴파일러가 고르는 타입
    || panic!()        → R = !
    한 갈래가 u8인 if  → R = u8
3. 폴백에 기대던 코드는 타입을 직접 적어서 고칩니다
    load_twice() = Ok(1)
4. rustc 1.100.0부터 !를 타입으로 직접 쓸 수 있습니다
    halve(9) = 4
    Infallible과 !는 같은 타입인가? true
    size_of::<!>() = 0, Vec<!>의 길이 = 0
```

- **2번 구역**: 클로저를 부르지 않고 타입만 보므로 `panic!()`은 실행되지 않습니다. 두 번째 줄의 조건 `flag`는 컴파일러가 미리 알 수 없게 명령줄 인자 개수로 만들었습니다.
- **4번 구역**: `halve`의 반환 타입 `Result<u32, !>`에서 `Err`는 만들어질 수 없으므로 `let Ok(half) = halve(9);`가 반박 불가능한 패턴으로 받아들여집니다.

### 07_other_changes — 그 밖의 변경

```sh
cargo run -p p2-ch4-rust-2024-language --example 07_other_changes
```

```text
1. gen 키워드
    counter.r#gen() = 1, counter.next_value() = 2
2. 참조를 맞추는 패턴 안에서는 mut/ref/&를 쓸 수 없습니다
    기본 바인딩 모드: 러스트, 2024
    완전히 명시한 패턴: 러스트, 2027
3. 매크로의 expr 프래그먼트 지정자
    expr      + const 블록 → 1번 규칙($e:expr)
    expr_2021 + const 블록 → 2번 규칙(const 블록)
4. 예약 문법: #"..."#와 ##
    r#"foo"#   → 토큰 1개
    # "foo" #  → 토큰 3개
    # # #      → 토큰 3개
```

### projects/edition_compare — 같은 코드, 두 에디션

```sh
cd part2-editions/ch4-rust-2024-language/projects/edition_compare
cargo run -q --bin if_let_scope_2021      # 이름 끝을 _2024로 바꿔 같은 파일의 2024판을 실행
```

| 소스 파일 (`shared/`) | 실행 파일 | 두 에디션의 출력 |
| --- | --- | --- |
| [`if_let_scope.rs`](projects/edition_compare/shared/if_let_scope.rs) | `if_let_scope_2021`, `_2024` | 다름(1·4·5번 구역) |
| [`tail_expr_scope.rs`](projects/edition_compare/shared/tail_expr_scope.rs) | `tail_expr_scope_2021`, `_2024` | 다름(1·2번 구역의 drop 순서) |
| [`macro_expr.rs`](projects/edition_compare/shared/macro_expr.rs) | `macro_expr_2021`, `_2024` | 다름(1번 구역) |
| [`never_fallback.rs`](projects/edition_compare/shared/never_fallback.rs) | `never_fallback_2021`, `_2024` | 같음(rustc 1.100.0) |
| [`new_syntax.rs`](projects/edition_compare/shared/new_syntax.rs) | `new_syntax_2021`, `_2024` | 같음(2024 문법이 2021에서도 됨) |

모두 종료 코드 0입니다. 이 프로젝트는 루트 워크스페이스와 분리된 워크스페이스이므로 이 디렉터리에서 `cargo build`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`가 경고 없이 통과합니다. 2021 에디션 쪽에서도 경고가 나지 않으므로 `#[allow]`로 막은 경고는 없습니다. bash나 zsh라면 `diff <(cargo run -q --bin if_let_scope_2021) <(cargo run -q --bin if_let_scope_2024)`로 다른 줄만 볼 수도 있습니다.

`if_let_scope_2024`의 출력입니다. `if_let_scope_2021`은 1번 구역의 `else 블록`과 `[drop]` 두 줄의 순서가 바뀌고(2.4절의 나란한 출력), 4·5번 구역이 `else에서 가변 대여 실패: 조건식의 Ref가 아직 살아 있습니다`, `캐시: []`, `else에서 쓰기 잠금 실패: try_lock failed because the operation would block`, `값: None`으로 바뀝니다.

```text
1. 패턴이 맞지 않아 else로 갈 때
    [drop] 조건식의 임시값
    else 블록
    if let 다음 문장
2. 패턴이 맞아 then으로 갈 때 (두 에디션이 같음)
    then 블록: 7
    [drop] 조건식의 임시값
3. match (두 에디션이 같음)
    None 갈래
    [drop] 조건식의 임시값
4. RefCell: 비어 있으면 else에서 채우기
    else에서 가변 대여 성공: 캐시에 1을 넣었습니다
    캐시: [1]
5. RwLock: 값이 없으면 else에서 쓰기
    else에서 쓰기 잠금 성공
    값: Some(true)
```

`tail_expr_scope`는 1번 구역(함수 본문)과 2번 구역(블록)에서 `[drop]` 두 줄의 순서만 다르고(1번 구역은 2.6절에 나란히 실었습니다), 3번 구역(`let` 문의 임시값)은 두 에디션이 같습니다. `macro_expr`의 2번 구역(`expr_2021`)은 두 에디션 모두 `1번 규칙($e:expr_2021)`, `2번 규칙(const 블록)`, `3번 규칙(밑줄)`입니다. 1번 구역은 2.12절에 실었습니다. `never_fallback`의 출력은 2.10절에 실었습니다. `new_syntax`의 출력은 두 에디션 모두 다음과 같습니다.

```text
1. unsafe extern: sqrt(2.25) = 1.5
   strlen(c"edition") = 7, abs(-3) = 3
2. unsafe 속성: p2ch4_double(21) = 42
3. unsafe fn 안의 unsafe 블록: 10
4. &raw mut으로 static mut 고치기: 1
5. r#gen() = 4
6. use<>: [0, 1, 2], items = [1, 2, 3, 4]
7. 완전히 명시한 패턴: 러스트 95
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 한쪽 에디션에서 일부러 컴파일되지 않게 만든 코드입니다. 파일 맨 위 주석에 **어느 에디션에서 실패하고 어느 에디션에서 통과하는지**와 확인 명령이 적혀 있습니다. 2024에서는 통과하고 2021에서만 실패하는 파일은 이름 끝에 `_2021`을 붙였습니다. 챕터 디렉터리에서 실행합니다.

```sh
cd part2-editions/ch4-rust-2024-language
rustc --edition 2024 compile_fail/e0502_rpit_captures_borrow.rs --out-dir /tmp/rust-p2-ch4
```

| 파일 | 2021 에디션 | 2024 에디션 |
| --- | --- | --- |
| `e0700_rpit_hidden_lifetime_2021.rs` | 실패(E0700) | 통과 |
| `e0597_tail_expr_borrow_2021.rs` | 실패(E0597) | 통과 |
| `e0716_tail_expr_narrowed.rs` | 통과 | 실패(E0716) |
| `e0502_rpit_captures_borrow.rs` | 통과 | 실패(E0502) |
| `unsafe_missing_on_ffi.rs` | 통과 | 실패 |
| `reserved_guarded_string.rs` | 통과 | 실패 |
| `e0277_never_type_fallback.rs` | 실패(E0277) | 실패(E0277) |

통과하는 쪽은 모두 경고 없이 컴파일됩니다. 2021에서 쓴 let 체인과 2024에서 `gen`을 이름으로 쓴 코드는 [ch1](../ch1-editions-overview/README.md)의 4.4절(`let_chains_2021.rs`)과 4.2절(`reserved_keyword_gen.rs`)에 있습니다.

### 4.1 2021 에디션에서 인자를 빌리는 반복자를 RPIT로 돌려줌 — E0700

파일: [`compile_fail/e0700_rpit_hidden_lifetime_2021.rs`](compile_fail/e0700_rpit_hidden_lifetime_2021.rs) (2021에서 실패)

```text
error[E0700]: hidden type for `impl Iterator<Item = i32>` captures lifetime that does not appear in bounds
  --> compile_fail/e0700_rpit_hidden_lifetime_2021.rs:16:5
   |
15 | fn evens(numbers: &[i32]) -> impl Iterator<Item = i32> {
   |                              ------------------------- opaque type defined here
16 |     numbers.iter().copied().filter(|n| n % 2 == 0) // → 2021에서 E0700
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: hidden type `Filter<Copied<Iter<'_, i32>>, {closure@...}>` captures lifetime `'_`
```

- **읽는 법**: "숨은 타입(hidden type)이 바운드에 나오지 않는 라이프타임을 캡처한다"는 뜻입니다. `note:`의 `Iter<'_, i32>`가 `numbers`를 빌리는 슬라이스 반복자이고, `'_`가 그 대여의 라이프타임입니다. `{closure@...}`는 클로저 타입을 줄여 표시한 것입니다.
- **원인**: 2021에서 RPIT는 바운드에 이름이 나오는 라이프타임만 캡처하는데, 반환값은 실제로 `numbers`를 빌립니다(2.3절). 2024는 자동으로 캡처하므로 통과합니다.
- **해결**: 2024 에디션으로 올리면 그대로 됩니다. 2021에서는 `-> impl Iterator<Item = i32> + use<'_>`(두 에디션에서 같은 뜻) 또는 예전 방식인 `+ '_`를 붙입니다. 두 모양 모두 두 에디션에서 컴파일되는 것을 확인했습니다.

### 4.2 2021 에디션에서 마지막 표현식의 대여 가드 — E0597

파일: [`compile_fail/e0597_tail_expr_borrow_2021.rs`](compile_fail/e0597_tail_expr_borrow_2021.rs) (2021에서 실패)

```text
error[E0597]: `names` does not live long enough
  --> compile_fail/e0597_tail_expr_borrow_2021.rs:19:5
...
19 |     names.borrow().len() // → 2021에서 E0597
   |     ^^^^^---------
   |     |
   |     borrowed value does not live long enough
   |     a temporary with access to the borrow is created here ...
20 | }
   | -
   | |
   | `names` dropped here while still borrowed
   | ... and the borrow might be used here, when that temporary is dropped and runs the destructor for type `Ref<'_, Vec<&str>>`
   |
   = note: the temporary is part of an expression at the end of a block;
           consider forcing this temporary to be dropped sooner, before the block's local variables are dropped
help: for example, you could save the expression's value in a new local variable `x` and then make `x` be the expression at the end of the block
```

- **읽는 법**: 위쪽 밑줄(`a temporary with access to the borrow is created here`)이 `names.borrow()`가 만든 임시 `Ref` 가드이고, 20번 줄의 `}`에서 `names`가 먼저 버려진 뒤 그 가드가 버려지며 `names`를 건드린다는 뜻입니다. `note:`가 원인을 정확히 짚습니다. "임시값이 블록의 마지막 표현식에 있다."
- **원인**: 2021에서는 마지막 표현식의 임시값이 지역 변수보다 나중에 버려집니다(2.6절).
- **해결**: 2024 에디션에서는 그대로 컴파일됩니다(예제 04의 2번 구역). 2021에서는 help대로 값을 지역 변수에 먼저 담습니다(`let n = names.borrow().len(); n`). 이 모양은 두 에디션 모두에서 컴파일되는 것을 확인했습니다.

### 4.3 2024 에디션에서 블록 밖으로 내보낸 임시값의 참조 — E0716

파일: [`compile_fail/e0716_tail_expr_narrowed.rs`](compile_fail/e0716_tail_expr_narrowed.rs) (2024에서 실패)

```text
error[E0716]: temporary value dropped while borrowed
  --> compile_fail/e0716_tail_expr_narrowed.rs:16:18
   |
16 |     let len = { &String::from("1234") }.len(); // → 2024에서 E0716
   |                  ^^^^^^^^^^^^^^^^^^^-   --- borrow later used by call
   |                  |                  |
   |                  |                  temporary value is freed at the end of this statement
   |                  creates a temporary value which is freed while still in use
   |
   = note: consider using a `let` binding to create a longer lived value
```

- **읽는 법**: 밑줄 친 `String::from("1234")`가 임시값이고, `-` 표시(블록을 닫는 `}` 바로 앞)에서 버려지는데 그 뒤의 `.len()`이 아직 그것을 빌린다는 뜻입니다. 레이블은 "end of this statement"라고 하지만 가리키는 위치는 블록의 끝입니다.
- **원인**: 2024에서는 마지막 표현식의 임시값이 블록이 끝날 때 버려집니다. 2021에서는 바깥 문장 끝까지 살아 있어서 통과했습니다(2.6절).
- **해결**: note대로 `let`으로 묶습니다. `let text = { &String::from("1234") }; let len = text.len();`은 임시값 수명 연장 덕분에 두 에디션에서 컴파일됩니다(예제 04의 3번 구역).

### 4.4 RPIT가 캡처한 대여 — E0502

파일: [`compile_fail/e0502_rpit_captures_borrow.rs`](compile_fail/e0502_rpit_captures_borrow.rs) (2024에서 실패)

```text
error[E0502]: cannot borrow `scores` as mutable because it is also borrowed as immutable
  --> compile_fail/e0502_rpit_captures_borrow.rs:23:5
   |
22 |     let it = positions(&scores);
   |                        ------- immutable borrow occurs here
23 |     scores.push(90); // → 2024에서 E0502
   |     ^^^^^^^^^^^^^^^ mutable borrow occurs here
24 |     println!("{:?}", it.collect::<Vec<_>>());
   |                      -- immutable borrow later used here
   |
note: this call may capture more lifetimes than intended, because Rust 2024 has adjusted the `impl Trait` lifetime capture rules
  --> compile_fail/e0502_rpit_captures_borrow.rs:22:14
...
help: use the precise capturing `use<...>` syntax to make the captures explicit
   |
16 | fn positions(numbers: &[i32]) -> impl Iterator<Item = usize> + use<> {
   |                                                              +++++++
```

- **읽는 법**: 에러 자체는 [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md)부터 보아 온 E0502입니다. 새로운 것은 `note:`입니다. `positions(&scores)`의 반환값이 `scores`의 대여를 들고 있는 이유가 2024의 캡처 규칙이라고 알려 줍니다.
- **원인**: 2024에서는 `positions`의 불투명 타입이 인자의 라이프타임을 캡처하므로, `it`이 살아 있는 동안 `scores`를 빌린 것으로 취급됩니다. 숨은 타입 `Range<usize>`가 실제로 빌리지 않는다는 것은 부르는 쪽에서 보이지 않습니다(2.3절).
- **해결**: 반환값이 인자를 빌리지 않는다면 help대로 `+ use<>`를 붙입니다(두 에디션에서 같은 뜻). 정말 빌린다면 반복자를 다 쓴 뒤에 `push`합니다.

### 4.5 `unsafe` 없는 `extern` 블록과 `#[no_mangle]`

파일: [`compile_fail/unsafe_missing_on_ffi.rs`](compile_fail/unsafe_missing_on_ffi.rs) (2024에서 실패)

```text
error: extern blocks must be unsafe
  --> compile_fail/unsafe_missing_on_ffi.rs:19:1
   |
19 |   extern "C" {
   |   ^
   |   |
   |  _help: needs `unsafe` before the extern keyword: `unsafe`
...
error: unsafe attribute used without unsafe
  --> compile_fail/unsafe_missing_on_ffi.rs:23:3
   |
23 | #[no_mangle] // → 2024에서 "unsafe attribute used without unsafe"
   |   ^^^^^^^^^ usage of unsafe attribute
   |
help: wrap the attribute in `unsafe(...)`
```

- **읽는 법**: 둘 다 E 코드가 없는 에러이고, help가 고칠 자리를 정확히 보여 줍니다. 첫 에러의 help는 블록 시작 위치를 가리키며 `extern` 앞에 `unsafe`를 넣으라는 뜻입니다.
- **원인**: 2024에서는 선언이 맞는지, 심볼 이름이 겹치지 않는지에 대한 보증을 `unsafe`로 드러내야 합니다(2.7·2.8절). 2021에서는 이 파일이 경고 하나 없이 컴파일됩니다.
- **해결**: `unsafe extern "C" { .. }`, `#[unsafe(no_mangle)]`. 두 모양 모두 2021에서도 컴파일되므로 에디션을 올리기 전에 고쳐 둘 수 있습니다. `export_name`, `link_section`도 같습니다.

### 4.6 매크로 입력에 쓴 `#"..."#`와 `###` — 예약 문법

파일: [`compile_fail/reserved_guarded_string.rs`](compile_fail/reserved_guarded_string.rs) (2024에서 실패)

```text
error: invalid string literal
  --> compile_fail/reserved_guarded_string.rs:27:33
   |
27 |     println!("{}", token_count!(#"foo"#)); // → 2024에서 에러
   |                                 ^^^^^^^
   |
   = note: unprefixed guarded string literals are reserved for future use since Rust 2024
help: consider inserting whitespace here
   |
27 |     println!("{}", token_count!(# "foo"#)); // → 2024에서 에러
   |                                  +

error: reserved multi-hash token is forbidden
  --> compile_fail/reserved_guarded_string.rs:28:33
...
   = note: sequences of two or more # are reserved for future use since Rust 2024
```

- **읽는 법**: E 코드가 없는 에러입니다. 첫 에러의 "guarded string literal"은 `#`로 둘러싼 문자열 리터럴을 말하고, 두 번째 에러는 `#`가 공백 없이 두 개 이상 이어진 토큰을 말합니다. 둘 다 note가 "2024부터 앞으로를 위해 예약됨"이라고 알려 줍니다.
- **원인**: 2021에서는 둘 다 `#`와 문자열로 나뉘는 토큰 세 개였고, 이 파일은 `3`을 두 번 출력합니다. 2024는 이 모양들을 예약했습니다(2.12절).
- **해결**: help대로 공백을 넣습니다. `# "foo" #`, `# # #`는 두 에디션에서 토큰 세 개입니다(예제 07의 4번 구역). raw 문자열 `r#"foo"#`는 영향이 없습니다.

### 4.7 never 타입 폴백에 기댄 `?` — E0277

파일: [`compile_fail/e0277_never_type_fallback.rs`](compile_fail/e0277_never_type_fallback.rs) (rustc 1.100.0에서는 두 에디션 모두 실패)

```text
error[E0277]: the trait bound `!: Default` is not satisfied
  --> compile_fail/e0277_never_type_fallback.rs:22:5
   |
22 |     load()?; // → E0277
   |     ^^^^^^ the trait `Default` is not implemented for `!`
   |
   = note: this error might have been caused by changes to Rust's type-inference algorithm (see issue #148922 <https://github.com/rust-lang/rust/issues/148922> for more information)
   = help: you might have intended to use the type `()` here instead
note: required by a bound in `load`
```

- **읽는 법**: `load`의 `T`가 `!`로 정해졌는데 `!`는 `Default`가 아니라는 뜻입니다. 코드 어디에도 `!`를 적지 않았으므로, `!`가 나왔다면 폴백을 의심합니다. 첫 `note:`가 바로 그 이야기입니다(타입 추론 규칙이 바뀐 탓일 수 있다).
- **원인**: `load()?;`는 결과를 버리므로 `T`를 정할 단서가 없고, 폴백 타입 `!`가 쓰였습니다(2.10절). rustc 1.98.1의 2021 에디션에서는 폴백이 `()`라 대신 deny 린트 `` this function depends on never type fallback being `()` ``(장래 호환성 경고)가 났습니다. 1.100.0에서는 그 린트가 없어지고 두 에디션 모두 이 E0277입니다.
- **해결**: help대로 `()`를 직접 적습니다. `load::<()>()?;` 또는 `let () = load()?;`. 두 모양 모두 두 에디션에서 컴파일되는 것을 확인했습니다.

### 4.8 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| 2024에서 `println!("{}", COUNTER)`(`static mut`) | 에러 `creating a shared reference to mutable static`. 2021은 경고 | 값을 먼저 복사(`let c = unsafe { COUNTER };`)하거나 원자 타입으로 (2.9절) |
| `unsafe` 없는 `extern` 블록 안에 `safe fn` | 두 에디션 모두 에러 `` items in `extern` blocks without an `unsafe` qualifier cannot have safety qualifiers `` | 블록에 `unsafe`를 붙임 (2.7절) |
| `unsafe extern` 블록의 표시 없는 함수를 그냥 부름 | E0133 `` call to unsafe function `abs` is unsafe and requires unsafe block `` (2021은 `... requires unsafe function or block`) | `unsafe { }`로 감싸거나, 정말 안전하면 선언을 `safe fn`으로 (2.7절) |
| `fn f<T>(x: &[T]) -> impl Iterator<Item = T> + use<>` | 두 에디션 모두 에러 `` `impl Trait` must mention all type parameters in scope in `use<...>` `` | `use<T>`처럼 타입 매개변수를 적음 (2.3절) |
| `println!("{r#gen}")` | 두 에디션 모두 에러 `invalid format string: raw identifiers are not supported` | `{gen}` (2.11절) |
| 2024에서 `let (x, mut y) = &pair;` | 에러 `cannot mutably bind by value within an implicitly-borrowing pattern` | `let &(ref x, mut y) = &pair;` (2.12절) |
| 2024에서 `fn gen()`, `x.gen()` | 에러 `` expected identifier, found reserved keyword `gen` `` | `r#gen` (2.11절, [ch1](../ch1-editions-overview/README.md) 4.2절) |
| `if_let_rescope`의 `match` 제안을 그대로 받아들임 | 2021의 읽기 잠금 후 쓰기 잠금 버그가 그대로 남음 | 바뀐 곳마다 필요한 동작을 판단 (2.4절) |

## 5. 연습 문제

### 연습 1. 2021 코드를 두 에디션에서 컴파일되게 고치기

다음은 2021 에디션 코드입니다. 2021에서는 `static_mut_refs` 경고 하나와 함께 컴파일되고 `abs = 5, calls = 1`, `id = 1, indexes = [0], words = ["가", "나"]`를 출력합니다. `rustc --edition 2021`과 `--edition 2024` **양쪽에서 경고 없이** 컴파일되고 같은 출력을 내도록 고치세요.

```rust
use std::ffi::c_int;

extern "C" {
    fn abs(x: c_int) -> c_int;
}
static mut CALLS: u32 = 0;

struct Ids {
    next: u32,
}
impl Ids {
    fn gen(&mut self) -> u32 {
        self.next += 1;
        self.next
    }
}

fn indexes(words: &[String]) -> impl Iterator<Item = usize> {
    0..words.len()
}

fn main() {
    let mut ids = Ids { next: 0 };
    let id = ids.gen();
    unsafe {
        CALLS += 1;
        println!("abs = {}, calls = {}", abs(-5), CALLS);
    }
    let mut words = vec![String::from("가")];
    let it = indexes(&words);
    words.push(String::from("나"));
    println!("id = {id}, indexes = {:?}, words = {words:?}", it.collect::<Vec<_>>());
}
```

<details>
<summary>힌트</summary>

- 2024로 처음 컴파일하면 `gen` 두 곳, `extern` 블록, E0502(`it`과 `push`)가 나옵니다. 이것을 고치고 다시 컴파일하면 그제야 `static mut` 에러가 하나 더 나옵니다. 컴파일러는 앞 단계에서 에러가 나면 멈추므로, 고치고 다시 컴파일하는 것을 반복합니다.
- 바꿀 곳: `unsafe extern "C"`, `fn r#gen`과 `ids.r#gen()`, `-> impl Iterator<Item = usize> + use<>`.
- `CALLS`는 `static CALLS: AtomicU32 = AtomicU32::new(0);`으로 바꾸고 `CALLS.fetch_add(1, Ordering::Relaxed) + 1`로 새 값을 얻으면 `unsafe` 블록에는 `abs` 호출만 남습니다. `SAFETY:` 주석도 답니다.

</details>

### 연습 2. 에디션에 기대지 않는 "없으면 채우기"

다음 함수는 2024 에디션에서는 동작하지만, 2021 에디션에서는 `else`의 `write()`에서 멈추거나 패닉할 수 있습니다. **어느 에디션에서든 같은 동작**을 하도록 고치고, `cache`가 비어 있을 때와 채워져 있을 때 두 번 불러 `"기본값 기본값"`과 `Some("기본값")`이 나오는지 확인하세요. 고친 코드를 2021 에디션으로 실행해도 멈추지 않아야 합니다.

```rust
fn get_or_init(cache: &RwLock<Option<String>>) -> String {
    if let Some(v) = cache.read().unwrap().as_ref() {
        v.clone()
    } else {
        let v = String::from("기본값");
        *cache.write().unwrap() = Some(v.clone());
        v
    }
}
```

<details>
<summary>힌트</summary>

- `let` 문의 임시값은 두 에디션 모두 그 문장 끝에서 버려집니다. 읽기를 `let current = cache.read().unwrap().clone();`로 따로 떼면 읽기 잠금이 그 줄에서 풀립니다.
- 그다음 `if let Some(v) = current { return v; }`로 돌려주고, 아래에서 쓰기 잠금을 얻습니다.
- 여러 스레드가 동시에 부른다면, 읽기 잠금을 풀고 쓰기 잠금을 얻는 사이에 다른 스레드가 먼저 채울 수 있습니다. 쓰기 잠금 안에서 다시 확인하려면 `cache.write().unwrap().get_or_insert_with(..).clone()`을, 한 번만 초기화하는 값이라면 `OnceLock`(Part 1 ch11 2.9절)을 생각해 보세요.

</details>

### 연습 3. 필요한 것만 캡처하는 `tokens`

`fn tokens<'t>(text: &'t str, seps: &str) -> impl Iterator<Item = &'t str>`를 작성하세요. `seps`에 든 문자 중 하나로 `text`를 나누고 빈 조각은 건너뜁니다. 부르는 쪽에서 반복자가 살아 있는 동안 `seps`를 고칠 수 있어야 합니다.

```rust
let text = String::from("사과,배;;포도");
let mut seps = String::from(",;");
let it = tokens(&text, &seps);
seps.push(' ');
println!("{:?} {seps:?}", it.collect::<Vec<_>>()); // ["사과", "배", "포도"] ",; "
```

<details>
<summary>힌트</summary>

- 반복자는 `text`만 빌려야 하므로 `seps`는 `let seps: Vec<char> = seps.chars().collect();`로 복사해 `move` 클로저에 넘깁니다. `text.split(move |c: char| seps.contains(&c)).filter(|t| !t.is_empty())`.
- 2024에서는 `+ use<'t>`를 붙여야 `seps.push`가 E0502가 되지 않습니다(예제 01의 4번 구역).
- 같은 코드를 `use<'t>` 없이 2021로 컴파일하면 통과합니다. 2021에서도 바운드에 이름이 나오는 라이프타임(`Item = &'t str`의 `'t`)은 캡처하기 때문입니다. 2.3절의 2021 규칙과 맞춰 보세요.

</details>

### 연습 4. 큐의 `while` 함정 고치기

`VecDeque::from([5, 4, 0, 3])`에서 앞에서부터 0보다 큰 작업만 처리하고, 처음으로 0이 나오면 멈추되 **0과 그 뒤의 작업은 큐에 남기세요**. `while let Some(job) = queue.pop_front() && job > 0`으로 쓰면 남는 것이 `[3]`뿐인 것을 먼저 확인하고, `[0, 3]`이 남도록 고치세요.

<details>
<summary>힌트</summary>

- 예제 03의 4번 구역처럼 `while let Some(&job) = queue.front() && job > 0 { queue.pop_front(); .. }`로 꺼내기 전에 확인합니다.
- `VecDeque::pop_front_if`(1.93.0)를 쓰면 `while let Some(job) = queue.pop_front_if(|j| *j > 0)`처럼 let 체인 없이도 됩니다. 두 방법 모두 남은 큐가 `[0, 3]`인지 확인하세요.

</details>

## 6. 참고 자료

- Edition Guide 5. Rust 2024 (기준 문서)
  - [Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html): RFC 3501, 안정화 버전 1.85.0. [Language](https://doc.rust-lang.org/edition-guide/rust-2024/language.html)가 5.1절의 목차입니다.
  - [RPIT lifetime capture rules](https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html): 캡처의 정의, 에디션별 규칙, `use<..>`, `Captures` 방법과 outlives 방법에서 옮기기, 인자 위치 `impl Trait`가 있을 때의 수동 수정.
  - [`if let` temporary scope](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-if-let-scope.html): `RwLock` 교착 상태 예, `match`로 바꾸면 2021 동작이 유지된다는 설명.
  - [`let` chains in `if` and `while`](https://doc.rust-lang.org/edition-guide/rust-2024/let-chains.html): `if let` 임시값 범위 변경에 기대기 때문에 에디션으로 나뉜다는 설명.
  - [Tail expression temporary scope](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-tail-expr-scope.html): `c.borrow().len()`의 2021 E0597과 범위가 좁아져 실패하는 예.
  - [Match ergonomics reservations](https://doc.rust-lang.org/edition-guide/rust-2024/match-ergonomics.html), [Unsafe `extern` blocks](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html), [Unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html), [`unsafe_op_in_unsafe_fn` warning](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html)
  - [Disallow references to static mut](https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html): 원자 타입, `Mutex`/`RwLock`, `OnceLock`/`LazyLock`, `no_std`의 한 번 초기화, 원시 포인터, `UnsafeCell` 감싸기 등 대안을 코드와 함께 설명합니다.
  - [Never type fallback change](https://doc.rust-lang.org/edition-guide/rust-2024/never-type-fallback.html): 1.100.0판에는 "Rust 1.100부터 더 이상 에디션별 변경이 아니다"라는 안내가 맨 위에 있습니다. 온라인 판은 최신 툴체인 기준이므로 내용이 다를 수 있습니다.
  - [Macro fragment specifiers](https://doc.rust-lang.org/edition-guide/rust-2024/macro-fragment-specifiers.html), [Missing macro fragment specifiers](https://doc.rust-lang.org/edition-guide/rust-2024/missing-macro-fragment-specifiers.html), [`gen` keyword](https://doc.rust-lang.org/edition-guide/rust-2024/gen-keyword.html), [Reserved syntax](https://doc.rust-lang.org/edition-guide/rust-2024/reserved-syntax.html)
- The Rust Reference
  - [Destructors → Temporary scopes](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes): 임시값 범위의 정확한 목록과 "2024 Edition differences" 상자. `match`의 조건식은 임시값 범위가 아니라는 note가 있습니다.
  - [Impl trait type → Capturing](https://doc.rust-lang.org/reference/types/impl-trait.html#capturing): 자동 캡처와 정밀 캡처 규칙.
- 표준 라이브러리 문서: [never 타입 `!`](https://doc.rust-lang.org/std/primitive.never.html)(1.100.0에서 안정화, Never-to-any coercion 절), [`RwLock::write`](https://doc.rust-lang.org/std/sync/struct.RwLock.html#method.write)(Panics 절).
- 로컬 문서: 위 Edition Guide와 Reference는 툴체인과 함께 설치되어 있습니다. `rustup doc --edition-guide`처럼 열거나 `$(rustc --print sysroot)/share/doc/rust/html/edition-guide/`를 엽니다. 이 챕터는 1.100.0과 함께 설치된 판으로 확인했습니다.
