# ch13. 함수형 언어의 특징: 클로저와 반복자

Rust는 함수를 값처럼 다루는 함수형 프로그래밍의 아이디어를 여럿 받아들였습니다. 이 챕터는 그중 가장 자주 쓰는 두 가지를 다룹니다.

- **클로저(closure)**: 변수에 저장하거나 다른 함수에 넘길 수 있는 이름 없는 함수입니다. 함수와 달리 자신이 정의된 곳의 변수를 붙잡아(캡처) 쓸 수 있습니다.
- **반복자(iterator)**: 값의 나열을 하나씩 차례로 꺼내 주는 것입니다. `map`, `filter` 같은 메서드를 이어 붙여 "무엇을 할지"만 적으면, 인덱스를 세고 끝을 확인하는 일은 반복자가 맡습니다.

두 가지 모두 이미 써 봤습니다. [ch9](../ch9-error-handling/README.md)의 `unwrap_or_else(|e| ...)`에 넘긴 `|e| ...`가 클로저이고, [ch3](../ch3-common-concepts/README.md)의 `(1..4).rev()`, [ch4](../ch4-ownership/README.md)의 `bytes.iter().enumerate()`, [ch8](../ch8-collections/README.md)의 `chars()`, [ch12](../ch12-io-project-minigrep/README.md)의 `env::args().collect()`가 반복자입니다. 이 챕터에서 그 동작 원리를 배우고, 마지막에 ch12의 `minigrep`을 반복자와 클로저로 개선합니다.

- 패키지: `p0-ch13-closures-iterators`
- 독립 프로젝트: `projects/minigrep` (ch12 판을 복사해 개선한 것)
- 기준 문서: [The Book 13장 Functional Language Features: Iterators and Closures](https://doc.rust-lang.org/book/ch13-00-functional-features.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024). 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 클로저를 정의하고 호출할 수 있으며, 함수와의 차이(환경 캡처, 생략할 수 있는 타입 표기, 처음 쓰인 타입으로 고정되는 추론)를 설명할 수 있다.
2. 클로저 본문을 보고 환경을 불변 대여·가변 대여·소유권 이동 중 어느 방식으로 캡처하는지 판단할 수 있고, `move`가 필요한 경우와 캡처 때문에 생기는 대여 에러(E0502, E0382)를 고칠 수 있다.
3. `FnOnce`·`FnMut`·`Fn`이 "본문이 캡처한 값을 어떻게 다루는가"로 정해진다는 것을 설명하고, `impl Fn(..)`이나 제네릭 바운드로 클로저를 받는 함수를 작성할 수 있다. `unwrap_or_else`가 `FnOnce`를, `sort_by_key`가 `FnMut`을 요구하는 이유를 설명할 수 있다.
4. `Iterator` 트레이트의 `next`, 반복자가 게으르다는 것, `for` 문과의 관계, `iter`/`iter_mut`/`into_iter`의 차이를 설명하고, 직접 만든 타입에 `Iterator`를 구현할 수 있다.
5. 어댑터(`map`, `filter`, `enumerate`, `zip`, `rev`, `take`, `skip`)와 소비 메서드(`sum`, `count`, `collect` 등)로 `for` 반복문을 반복자 체인으로 바꿔 쓸 수 있고(`minigrep` 개선), 그렇게 바꿔도 느려지지 않는 이유(비용 없는 추상화)를 설명할 수 있다.

## 2. 핵심 개념

2.1~2.7절은 클로저, 2.8~2.13절은 반복자, 2.14절은 `minigrep` 개선, 2.15절은 성능을 다룹니다.

### 2.1 클로저: 그 자리에서 만드는 이름 없는 함수

함수와 같은 일을 하는 클로저를 나란히 놓으면 이렇습니다(The Book의 예시).

```rust
fn  add_one_v1   (x: u32) -> u32 { x + 1 }   // 함수
let add_one_v2 = |x: u32| -> u32 { x + 1 };  // 타입을 모두 적은 클로저
let add_one_v3 = |x|             { x + 1 };  // 타입 표기 생략
let add_one_v4 = |x|               x + 1  ;  // 본문이 표현식 하나면 중괄호도 생략
```

- 매개변수를 `( )` 대신 `| |`로 감쌉니다. 매개변수가 없으면 `||`입니다.
- 클로저는 `let`으로 변수에 담고, 그 변수 이름 뒤에 괄호를 붙여 함수처럼 부릅니다.
- `rustfmt`(`cargo fmt`)는 `|x| { x + 1 }`처럼 중괄호 안에 표현식 하나뿐인 클로저를 `|x| x + 1`로 바꿔 놓습니다. 그래서 예제 01에는 v2, v4 모양과 본문이 여러 줄인 클로저를 실었습니다.

**함수는 타입을 꼭 적는데 클로저는 왜 생략해도 될까요?** 함수의 시그니처는 다른 코드와 맺는 약속입니다. 함수를 부르는 사람이 본문을 읽지 않고도 무엇을 넘기고 무엇을 받는지 알아야 하므로 타입을 명시합니다. 클로저는 대개 짧고, 정의한 바로 그 근처에서만 쓰입니다. 그 좁은 범위 안에서 어떻게 쓰이는지 보고 컴파일러가 타입을 추론할 수 있으므로 생략을 허용합니다.

**클로저는 환경을 캡처합니다.** 이것이 함수와의 가장 큰 차이입니다. 아래 `add_base`는 매개변수가 아닌 `base`를 본문에서 씁니다. 클로저가 정의된 곳의 변수를 붙잡아 두었다가 나중에 호출될 때 쓰는 것을 **캡처(capture)** 라고 부릅니다.

```rust
let base = 10;
let add_base = |x: i32| x + base;
println!("add_base(1) = {}", add_base(1)); // add_base(1) = 11
```

같은 일을 `main` 안에 `fn add_base(x: i32) -> i32 { x + base }`로 쓰면 컴파일러가 거부하고, 클로저를 쓰라고 안내합니다(핵심 줄만 옮겼습니다).

```text
error[E0434]: can't capture dynamic environment in a fn item
  = help: use the `|| { ... }` closure form instead
```

**클로저는 호출할 때 실행됩니다.** 정의하는 순간에는 본문이 실행되지 않습니다. 그래서 "나중에, 필요할 때만 실행할 코드"를 다른 함수에 넘기는 데 씁니다. ch9의 `unwrap_or_else`가 그 예입니다. `Err`일 때만 클로저를 부르므로, 기본값을 만드는 데 비용이 들거나 에러 메시지를 출력하는 것처럼 부수 효과가 있는 일도 필요할 때만 일어납니다. 예제는 [`01_closure_syntax.rs`](examples/01_closure_syntax.rs)입니다.

### 2.2 타입 추론: 처음 쓰인 타입으로 고정됩니다

타입 표기가 없는 클로저도 매개변수와 반환값마다 **구체적인 타입 하나**를 갖습니다. 컴파일러는 클로저가 처음 쓰인 모습을 보고 그 타입을 정합니다.

```rust
let identity = |x| x;
let s = identity(String::from("hello")); // 여기서 x와 반환값이 String으로 정해집니다.
let n = identity(5);                     // E0308: expected `String`, found integer
```

클로저는 여러 타입을 받는 제네릭 함수가 아닙니다. `let v = Vec::new();`가 나중에 넣는 값을 보고 원소 타입을 정하는 것과 같은 방식입니다. 에러 메시지와 해결 방법은 4.1절에 있습니다.

**클로저마다 고유한 타입이 있습니다.** 클로저의 타입은 컴파일러가 만들어 붙이는 이름 없는 타입이라 코드에 적을 수 없습니다. 모양이 완전히 같은 클로저 두 개도 서로 다른 타입입니다. `let mut f = |x: i32| x + 1;`을 만든 뒤 `f = |x: i32| x + 1;`로 다시 대입하면 E0308과 함께 이런 설명이 나옵니다.

```text
  = note: no two closures, even if identical, have the same type
```

그래서 클로저를 받거나 돌려주는 함수는 타입 이름 대신 "이 트레이트를 구현한 어떤 타입"이라고 적습니다. 그 트레이트가 2.5절의 `Fn` 트레이트입니다.

### 2.3 환경을 캡처하는 세 가지 방식

함수가 매개변수를 받는 방식이 세 가지(불변 참조, 가변 참조, 소유권)이듯, 클로저가 환경을 캡처하는 방식도 세 가지입니다. 어느 방식을 쓸지는 우리가 고르지 않습니다. **컴파일러가 본문이 그 변수로 무엇을 하는지 보고, 그 일을 할 수 있는 가장 약한 방식을 변수마다 고릅니다.**

| 본문이 변수로 하는 일 | 캡처 방식 | 클로저가 살아 있는 동안 원래 변수는 |
| --- | --- | --- |
| 읽기만 함 | 불변 대여 `&T` | 읽을 수 있고, 바꿀 수 없음 |
| 바꿈 (`push`, `+=` 등) | 가변 대여 `&mut T` | 읽을 수도 바꿀 수도 없음 |
| 밖으로 옮김 (반환, 다른 변수에 대입 등) | 소유권 이동 `T` | 정의한 순간부터 쓸 수 없음 |

여기서 "클로저가 살아 있는 동안"은 클로저를 정의한 곳부터 **마지막으로 사용한 곳까지**입니다. 캡처는 특별한 규칙이 아니라 평범한 대여와 이동이라서, [ch4](../ch4-ownership/README.md)의 대여 규칙과 "참조는 마지막으로 사용되는 곳까지 유효하다"는 원칙이 그대로 적용됩니다.

```rust
let mut list = vec![1, 2, 3];
let mut borrows_mutably = || list.push(7); // 여기서 가변 대여가 시작됩니다.
// println!("{list:?}");                   // 여기서 읽으면 E0502
borrows_mutably();                         // 가변 대여의 마지막 사용
println!("{list:?}");                      // 대여가 끝났으므로 괜찮습니다: [1, 2, 3, 7]
```

- `borrows_mutably`는 `let r = &mut list;`로 만든 참조를 품고 있는 것과 같습니다. 그 사이에 `list`를 읽으려 하면 "가변 대여가 있는 동안 다른 대여는 없어야 한다"는 규칙에 걸립니다(4.2절).
- 가변 대여를 캡처한 클로저는 변수에도 `mut`이 필요합니다. 호출할 때마다 클로저가 품은 가변 참조를 써서 상태를 바꾸기 때문입니다. 예를 들어 `let inc = || count += 1;`처럼 `mut` 없이 만들고 `inc()`를 부르면 ``error[E0596]: cannot borrow `inc` as mutable, as it is not declared as mutable``과 함께 ``calling `inc` requires mutable binding due to mutable borrow of `count` ``라는 설명이 나옵니다.
- 본문이 `|| name`처럼 캡처한 값을 밖으로 내보내면, 빌린 것으로는 내보낼 수 없으므로 클로저가 정의되는 순간 `name`의 소유권이 클로저 안으로 옮겨 갑니다.

세 경우 모두 [`02_capture_modes.rs`](examples/02_capture_modes.rs)에서 실행해 볼 수 있습니다.

### 2.4 `move`: 소유권을 강제로 가져가기

본문이 읽기만 해도 값을 클로저 안으로 옮기고 싶을 때는 매개변수 목록 앞에 `move`를 붙입니다.

```rust
let list = vec![1, 2, 3];
let print_list = move || println!("{list:?}");
print_list();
// 여기서 list를 쓰면 E0382: borrow of moved value (4.3절)
```

**언제 필요한가?** 클로저가 자신이 빌린 변수보다 **오래 살아야 할 때**입니다. 대표적인 경우가 클로저를 함수 밖으로 돌려줄 때입니다.

```rust
fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}
```

`count`는 `make_counter`의 지역 변수라서 함수가 끝나면 사라집니다. 클로저가 `count`를 빌리기만 한다면 돌려준 클로저는 사라진 값을 가리키게 됩니다(댕글링 참조). `move`를 빼면 컴파일러가 이를 잡아내고 `move`를 붙이라고 제안합니다(핵심 줄만 옮겼습니다).

```text
error[E0373]: closure may outlive the current function, but it borrows `count`, which is owned by the current function
note: closure is returned here
help: to force the closure to take ownership of `count` (and any other referenced variables), use the `move` keyword
```

반환 타입의 `impl FnMut() -> u32`는 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 본 반환 위치의 `impl Trait`입니다. 클로저 타입에는 이름이 없으므로 이 문법이 꼭 필요합니다. `make_counter()`를 두 번 부르면 클로저마다 자기 `count`를 소유하므로 두 카운터는 서로 영향을 주지 않습니다. 다른 스레드에 클로저를 넘길 때도 같은 이유로 `move`가 필요하며, 이것은 [ch15](../ch15-concurrency/README.md)에서 다룹니다.

**`Copy` 타입을 `move`하면 복사본이 들어갑니다.** `i32` 같은 `Copy` 타입은 이동 대신 복사되므로, `move` 뒤에도 원래 변수를 쓸 수 있습니다. 대신 클로저 안에서 바꾼 값은 클로저 안의 복사본이라 바깥 변수에는 반영되지 않습니다. 예제 02의 5번 구역에서 클로저를 두 번 불러 `2`가 나온 뒤에도 바깥의 `count`는 `0`입니다.

**`move`는 "값이 어떻게 들어오는가"만 정합니다.** 위의 `print_list`는 `list`를 소유하지만 본문은 읽기만 하므로 여러 번 부를 수 있습니다. 몇 번 부를 수 있는지는 다음 절의 `Fn` 트레이트가 정하고, 그것은 본문이 캡처한 값을 어떻게 다루는지에 달려 있습니다.

### 2.5 `Fn` 트레이트 세 가지

클로저가 캡처한 값을 본문에서 어떻게 다루는지에 따라, 컴파일러는 클로저에 다음 트레이트를 자동으로 구현합니다.

| 본문이 캡처한 값을 | 구현하는 트레이트 | 부를 수 있는 횟수 |
| --- | --- | --- |
| 밖으로 옮김 | `FnOnce` | 한 번 |
| 바꾸지만 옮기지는 않음 | `FnMut`, `FnOnce` | 여러 번 (부를 때 클로저를 가변으로 빌림) |
| 읽기만 함, 또는 아무것도 캡처하지 않음 | `Fn`, `FnMut`, `FnOnce` | 여러 번 |

- **`FnOnce`**: "한 번은 부를 수 있다"는 뜻이라 모든 클로저가 구현합니다. 캡처한 값을 밖으로 옮기는 클로저는 이것만 구현합니다. 한 번 부르고 나면 옮길 값이 남아 있지 않기 때문입니다. `|| { let owned = name; ... }`처럼 `name`을 옮기는 클로저를 두 번 부르면 E0382와 함께 ``closure cannot be invoked more than once because it moves the variable `name` out of its environment``라는 설명이 나옵니다.
- **`FnMut`**: 여러 번 부를 수 있지만 부를 때마다 캡처한 값을 바꿀 수 있습니다. 그래서 부르는 동안 클로저를 가변으로 빌려야 하고, 클로저 변수나 매개변수에 `mut`이 필요합니다.
- **`Fn`**: 여러 번 불러도 아무것도 바꾸지 않습니다. 같은 클로저를 여러 곳에서 동시에 불러도 안전하다는 보장이 됩니다(동시성은 [ch15](../ch15-concurrency/README.md)).

세 트레이트는 포함 관계입니다. `Fn`을 구현하면 `FnMut`도, `FnMut`을 구현하면 `FnOnce`도 구현합니다. 그래서 `Fn` 클로저는 `FnMut`이나 `FnOnce`를 요구하는 자리 어디에나 넘길 수 있습니다. 환경을 캡처하지 않는 보통 함수도 세 트레이트를 모두 구현하므로, 클로저 자리에 함수 이름을 넘길 수 있습니다(예제 03의 `call_twice(say_hello)`).

**2.3절의 캡처 방식과 헷갈리지 마세요.** 캡처 방식은 "값이 클로저 안으로 어떻게 들어오는가", `Fn` 트레이트는 "본문이 그 값을 어떻게 다루는가"입니다. `move` 클로저도 본문이 읽기만 하면 `Fn`입니다.

### 2.6 클로저를 받는 함수 작성하기

클로저의 타입에는 이름이 없으므로, 클로저를 받는 함수는 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 배운 트레이트 바운드로 "어떤 `Fn` 트레이트를 구현한 타입"을 요구합니다. 트레이트 이름 뒤에 함수처럼 매개변수 타입과 반환 타입을 적습니다(`Fn(i32) -> i32`, `FnOnce() -> T`, 반환값이 없으면 `Fn()`).

```rust
fn call_once(f: impl FnOnce()) {            // impl Trait 문법
    f();
}

fn call_twice_mut<F: FnMut()>(mut f: F) {   // 제네릭 바운드 문법. FnMut을 부르려면 mut이 필요
    f();
    f();
}

fn call_twice<F>(f: F)                      // where 절
where
    F: Fn(),
{
    f();
    f();
}
```

어떤 클로저를 어느 함수에 넘길 수 있는지는 다음과 같습니다([`03_fn_traits.rs`](examples/03_fn_traits.rs)).

| 넘기는 클로저 | `impl FnOnce()` | `F: FnMut()` | `F: Fn()` |
| --- | --- | --- | --- |
| 읽기만 함: `\|\| println!("{greeting}")` | O | O | O |
| 바꿈: `\|\| count += 1` | O | O | X (E0594) |
| 옮김: `\|\| { let taken = name; ... }` | O | X (E0507) | X |

`Fn`을 요구하는 자리에 값을 바꾸는 클로저를 넘기면, 컴파일러는 받는 쪽의 바운드를 고치라고 알려 줍니다(핵심 줄만 옮겼습니다).

```text
error[E0594]: cannot assign to `count`, as it is a captured variable in a `Fn` closure
1 | fn call_twice(f: impl Fn()) {
  |                  --------- change this to accept `FnMut` instead of `Fn`
```

**어떤 바운드를 고를까?** 함수를 만드는 쪽은 자기가 클로저를 어떻게 부를지에 맞춰 **필요한 만큼만** 요구합니다.

- 한 번만(또는 많아야 한 번) 부른다면 `FnOnce`. 모든 클로저가 구현하므로 호출하는 쪽이 가장 자유롭습니다.
- 여러 번 부르고, 클로저가 상태를 바꿔도 괜찮다면 `FnMut`.
- 여러 번 부르되 클로저가 아무것도 바꾸면 안 된다면 `Fn`.

### 2.7 표준 라이브러리가 클로저를 받는 방법

**`unwrap_or_else`는 `FnOnce`를 요구합니다.** `Option<T>`의 정의는 이렇습니다(The Book에서 인용).

```rust
impl<T> Option<T> {
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Some(x) => x,
            None => f(),
        }
    }
}
```

`F: FnOnce() -> T`는 "인자 없이 한 번 불러서 `T`를 돌려주는 것"이라는 뜻입니다. `None`일 때만, 그것도 한 번만 부르므로 가장 느슨한 `FnOnce`로 충분하고, 덕분에 캡처한 `String`을 그대로 내보내는 클로저도 받을 수 있습니다(예제 03의 5번 구역). `Result<T, E>`의 `unwrap_or_else`는 `FnOnce(E) -> T`를 요구해서 에러 값을 클로저 인자로 넘겨 줍니다. ch9와 ch12에서 쓴 `|err| { ... }`가 이것입니다.

기본값이 이미 만들어져 있는 단순한 값이라면 클로저 없이 `unwrap_or(값)`을 씁니다. `unwrap_or_else(|| 0)`처럼 쓰면 clippy가 ``unnecessary closure used to substitute value for `Option::None` ``이라며 `unwrap_or`를 쓰라고 제안합니다. 클로저는 기본값을 만드는 일 자체를 미루고 싶을 때 씁니다.

**`sort_by_key`와 `sort_by`는 `FnMut`을 요구합니다.** [ch8](../ch8-collections/README.md)에서는 `sort()`로 원소의 기본 순서대로 정렬했습니다. 다른 기준으로 정렬하려면 클로저로 기준을 넘깁니다.

- `sort_by_key(|r| r.width)`: 원소마다 정렬 기준 값(키)을 돌려주는 클로저를 받습니다. 바운드는 `F: FnMut(&T) -> K`(`K`는 비교할 수 있는 타입)입니다.
- `sort_by(|a, b| ...)`: 두 원소를 받아 순서(`Ordering`)를 돌려주는 클로저를 받습니다. 바운드는 `F: FnMut(&T, &T) -> Ordering`입니다. [ch2](../ch2-guessing-game/README.md)에서 본 `cmp`가 `Ordering`을 돌려주며, `b.height.cmp(&a.height)`처럼 자리를 바꾸면 내림차순이 됩니다. `then`은 앞의 비교 결과가 `Equal`일 때만 뒤의 비교를 쓰므로 "높이 오름차순, 같으면 너비 내림차순" 같은 기준도 만들 수 있습니다.

정렬하는 동안 클로저를 원소마다 여러 번 부르므로 `FnOnce`로는 안 되고, 상태를 바꾸는 클로저까지 받아 주려고 `Fn`보다 느슨한 `FnMut`을 요구합니다. 그래서 바깥의 카운터를 늘리는 클로저는 넘길 수 있지만(예제 03의 6번 구역), 캡처한 값을 밖으로 옮기는 클로저는 E0507로 거부됩니다(4.4절).

### 2.8 반복자와 `Iterator` 트레이트

반복자는 표준 라이브러리의 `Iterator` 트레이트를 구현한 타입입니다. 핵심은 다음과 같습니다.

```rust
pub trait Iterator {
    type Item;                                // 이 반복자가 내놓는 값의 타입

    fn next(&mut self) -> Option<Self::Item>; // 다음 값을 Some으로, 다 꺼냈으면 None

    // 기본 구현이 있는 메서드들(map, filter, sum, collect 등)
}
```

- `type Item`은 "이 반복자가 내놓는 값의 타입"을 구현하는 쪽이 정하는 자리입니다. 이런 것을 연관 타입(associated type)이라 부르며 자세한 내용은 [ch19](../ch19-advanced-features/README.md)에서 다룹니다.
- 구현하는 쪽이 만들어야 하는 메서드는 `next` 하나뿐입니다. `map`, `filter`, `sum` 같은 수십 개의 메서드는 모두 `next`를 써서 만든 기본 구현(ch10)으로 따라옵니다.
- `next`는 `&mut self`를 받습니다. 반복자는 "지금 어디까지 꺼냈는지"를 상태로 들고 있고, `next`를 부를 때마다 그 상태가 한 칸 나아가기 때문입니다. 그래서 `next`를 직접 부르려면 반복자 변수에 `mut`이 필요합니다.

```rust
let v1 = vec![1, 2, 3];
let mut v1_iter = v1.iter();
println!("{:?}", v1_iter.next()); // Some(1)
println!("{:?}", v1_iter.next()); // Some(2)
println!("{:?}", v1_iter.next()); // Some(3)
println!("{:?}", v1_iter.next()); // None
```

`iter()`가 내놓는 값은 원소의 참조(`&i32`)입니다. 디버그 형식은 참조를 따라가 값을 보여 주므로 `Some(1)`로 출력됩니다.

**앞 챕터에서 이미 쓴 반복자**: 그때는 "메서드 호출"로만 썼던 코드들을 이제 읽을 수 있습니다.

| 챕터 | 코드 | 이제 보이는 것 |
| --- | --- | --- |
| ch3 | `for n in (1..4).rev()` | 범위는 반복자이고, `rev`는 거꾸로 내놓는 어댑터입니다(2.12절). |
| ch3, ch4 | `weekdays.iter().enumerate()`, `bytes.iter().enumerate()` | `iter()`로 만든 반복자에 `enumerate`가 인덱스를 붙여 `(인덱스, 원소의 참조)`를 내놓습니다. ch4의 `for (i, &item) in ...`에서 `&item`은 그 참조를 벗기는 패턴입니다. |
| ch8 | `s.chars().count()`, `chars().nth(n)` | `chars()`는 `char`를 내놓는 반복자입니다. `count`는 끝까지 꺼내며 세는 소비 메서드이고, `nth(n)`은 앞의 `n`개를 꺼내 버린 뒤 그다음 값을 돌려줍니다. 둘 다 앞에서부터 하나씩 꺼내므로 문자열이 길수록 오래 걸립니다. |
| ch8, ch9 | `chars().next()`, `lines().next()` | 반복자의 첫 값을 `Option`으로 꺼냅니다. |
| ch8, ch12 | `iter_mut()`, `env::args().collect()` | 가변 참조를 내놓는 반복자(2.11절), 반복자의 값을 컬렉션으로 모으는 소비 메서드(2.12절)입니다. |

### 2.9 반복자는 게으릅니다

반복자는 **게으릅니다(lazy)**. 소비하는 메서드를 부르기 전에는 아무 일도 하지 않습니다. `map`은 "원소마다 클로저를 적용하는 새 반복자"를 만들 뿐, 그 자리에서 클로저를 부르지 않습니다.

```rust
let doubled = numbers.iter().map(|n| {
    println!("  map이 {n}을(를) 처리");
    n * 2
});
println!("map을 호출했습니다(아직 아무 처리도 하지 않음)");
for d in doubled {
    println!("for가 {d}을(를) 받음");
}
```

출력을 보면 `map을 호출했습니다`가 먼저 나오고, 그 뒤로 "map 처리 → for 받음"이 원소 하나씩 번갈아 나옵니다(3절 예제 04). 반복자 체인은 원소 하나를 처음부터 끝까지 흘려보낸 뒤 다음 원소로 넘어갑니다. 그래서 중간 결과를 담는 벡터가 따로 생기지 않고, `take`처럼 필요한 만큼 받으면 그 뒤는 계산하지 않습니다. 1부터 끝없이 세는 `(1..)`도 `take(5)`를 붙이면 다섯 개만 계산하고 끝납니다.

어댑터만 만들고 소비하지 않으면 아무 일도 일어나지 않으므로 컴파일러가 경고합니다. 새 프로젝트의 `src/main.rs`에 다음을 쓰고 `cargo build`를 실행하면 이렇게 나옵니다.

```rust
fn main() {
    let v1 = vec![1, 2, 3];

    v1.iter().map(|x| x + 1);
}
```

```text
warning: unused `Map` that must be used
 --> src/main.rs:4:5
  |
4 |     v1.iter().map(|x| x + 1);
  |     ^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: iterators are lazy and do nothing unless consumed
  = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
help: use `let _ = ...` to ignore the resulting value
  |
4 |     let _ = v1.iter().map(|x| x + 1);
  |     +++++++
```

에러가 아니라 경고라서 프로그램은 빌드되지만, 클로저는 한 번도 실행되지 않습니다. `help:`의 `let _ =`는 "일부러 버린다"는 표시일 뿐이고 여전히 아무 일도 하지 않습니다. 결과가 필요하면 `collect` 같은 소비 메서드를 붙이고, 원소마다 부수 효과만 내고 싶다면 `for` 문을 씁니다.

### 2.10 `for` 문은 반복자 위에서 동작합니다

`for x in 값`은 `값.into_iter()`로 반복자를 얻은 뒤, `next`가 `None`을 돌려줄 때까지 부르는 문법입니다. 예제 04의 `manual_for`는 `for` 문이 하는 일을 `loop`와 `match`로 풀어 쓴 것입니다.

```rust
let mut iter = words.iter();
loop {
    match iter.next() {
        Some(word) => println!("  손으로 쓴 반복: {word}"),
        None => break,
    }
}
```

`for` 문에서는 반복자 변수에 `mut`을 붙이지 않았습니다. `for`가 반복자의 소유권을 가져가 안에서 가변으로 쓰기 때문입니다. (clippy는 이 `loop`를 `while let`으로 줄여 쓰라고 제안합니다. `while let`은 `if let`을 조건이 맞는 동안 되풀이하는 문법으로 [ch18](../ch18-patterns/README.md)에서 다룹니다.)

### 2.11 `iter`, `iter_mut`, `into_iter`

컬렉션에서 반복자를 만드는 메서드는 세 가지이고, 함수가 값을 받는 세 가지 방식과 대응합니다.

| 메서드 | 내놓는 값 | 원래 컬렉션은 | 같은 뜻의 `for` |
| --- | --- | --- | --- |
| `v.iter()` | 불변 참조 `&T` | 그대로 남음 | `for x in &v` |
| `v.iter_mut()` | 가변 참조 `&mut T` | 원소가 그 자리에서 바뀔 수 있음 | `for x in &mut v` |
| `v.into_iter()` | 값 자체 `T` | 이동되어 더 쓸 수 없음 | `for x in v` |

```rust
for score in scores.iter_mut() {
    *score += 5; // 참조가 가리키는 정수를 바꾸려면 역참조 *가 필요합니다(ch4).
}

let mut members: Vec<String> = Vec::new();
for name in names.into_iter() {
    members.push(name); // String 자체를 받으므로 복제 없이 옮겨 담습니다.
}
// 이제 names를 쓰면 E0382
```

`for x in &v`가 `v.iter()`와 같은 이유는, `for`가 부르는 `into_iter()`가 무엇에 부르느냐에 따라 다르게 동작하기 때문입니다. `Vec`에 부르면 원소를, `&Vec`에 부르면 참조를, `&mut Vec`에 부르면 가변 참조를 내놓습니다. 이 동작을 정하는 `IntoIterator` 트레이트는 [Part 1 ch5](../../part1-standard-lib/ch5-iterators/README.md)에서 다룹니다. 배열도 마찬가지라서, 2021 에디션부터는 `[a, b, c].into_iter()`가 원소 값을 내놓습니다. 예제는 [`05_iter_ownership.rs`](examples/05_iter_ownership.rs)입니다.

### 2.12 어댑터와 소비 메서드

`Iterator`의 기본 메서드는 크게 두 종류입니다.

- **어댑터(iterator adapter)**: 반복자를 받아 다른 반복자를 돌려줍니다. 게으르므로 그것만으로는 아무 일도 하지 않습니다.
- **소비 메서드(consuming adapter)**: 안에서 `next`를 불러 반복자를 다 써 버리고 결과값 하나를 돌려줍니다. `self`를 받아 반복자의 소유권을 가져가므로, 부른 뒤에는 그 반복자를 다시 쓸 수 없습니다(4.5절).

그래서 반복자 체인은 **반복자 만들기 → 어댑터 여러 개 → 소비 메서드 하나** 모양이 됩니다.

| 어댑터 | 하는 일 | 예 (`(1..=6)`에 적용) |
| --- | --- | --- |
| `map(f)` | 원소마다 `f`를 적용한 값을 내놓음 | `map(\|n\| n * 10)` → 10, 20, …, 60 |
| `filter(f)` | `f`가 `true`인 원소만 내놓음 | `filter(\|n\| n % 2 == 0)` → 2, 4, 6 |
| `rev()` | 뒤에서부터 내놓음 | 6, 5, 4, 3, 2, 1 |
| `take(n)` | 앞의 `n`개만 | `take(3)` → 1, 2, 3 |
| `skip(n)` | 앞의 `n`개를 건너뜀 | `skip(4)` → 5, 6 |
| `enumerate()` | `(인덱스, 원소)` 튜플 | `(0, 1)`, `(1, 2)`, … |
| `zip(other)` | 두 반복자에서 하나씩 꺼내 `(왼쪽, 오른쪽)` 튜플. 짧은 쪽이 끝나면 끝남 | |

| 소비 메서드 | 돌려주는 것 |
| --- | --- |
| `sum()` | 모두 더한 값 |
| `count()` | 원소 개수 |
| `max()`, `min()` | 가장 큰 값, 가장 작은 값을 `Option`으로 (비어 있으면 `None`) |
| `collect()` | 원소를 모은 컬렉션(`Vec`, `String` 등) |

**`filter`의 클로저는 항목의 참조를 받습니다.** `filter`는 항목을 가져가지 않고 들여다보기만 한 뒤 통과시킬지 정하므로, 클로저에 `&Item`을 넘깁니다. 범위 `(1..=6)`의 항목은 `i32`라서 클로저는 `&i32`를 받습니다. 산술 연산자는 정수의 참조에도 쓸 수 있어서 `n % 2`처럼 그대로 계산할 수 있습니다. 하지만 `scores.iter()`처럼 항목 자체가 참조(`&i32`)인 반복자에서는 클로저가 `&&i32`를 받으므로 `**s >= 80`처럼 두 번 따라가야 합니다. 구조체의 필드에 접근할 때는 자동 역참조 덕분에 `s.score`라고만 써도 됩니다.

**`collect`와 `sum`에는 결과 타입이 필요합니다.** 둘 다 여러 타입의 결과를 만들 수 있어서 무엇을 만들지 스스로 정하지 못합니다. 타입을 알려 주지 않으면 E0283입니다(`let v2 = v1.iter().map(|x| x + 1).collect();`를 컴파일한 결과에서 핵심 줄만 옮겼습니다).

```text
error[E0283]: type annotations needed
3 |     let v2 = v1.iter().map(|x| x + 1).collect();
  |         ^^                            ------- type must be known at this point
help: consider giving `v2` an explicit type
  |
3 |     let v2: Vec<_> = v1.iter().map(|x| x + 1).collect();
```

타입은 변수에 적거나(`let v2: Vec<i32> = ...`), 함수의 반환 타입으로 정해지게 하거나, 메서드 이름 뒤에 `::<>`로 붙여(`collect::<Vec<_>>()`, 이 모양을 터보피시(turbofish)라고 부릅니다) 알려 줍니다. `Vec<_>`의 `_`는 "원소 타입은 추론해 달라"는 뜻입니다.

**환경을 캡처하는 클로저와 `filter`**: 클로저가 바깥 값을 쓸 수 있다는 점이 반복자와 만나면 힘을 발휘합니다. 아래 함수는 매개변수 `cutoff`를 `filter`의 클로저가 캡처하므로, 같은 함수가 `cutoff`에 따라 다르게 걸러 냅니다.

```rust
fn passed_names(students: &[Student], cutoff: u32) -> Vec<&str> {
    students
        .iter()
        .filter(|s| s.score >= cutoff)
        .map(|s| s.name.as_str())
        .collect()
}
```

**반복문을 체인으로**: 예제 06의 4번 구역은 "1부터 10까지 중 짝수의 제곱의 합"을 두 가지로 씁니다.

```rust
let mut loop_total = 0;
for n in 1..=10 {
    if n % 2 == 0 {
        loop_total += n * n;
    }
}

let chain_total: i32 = (1..=10).filter(|n| n % 2 == 0).map(|n| n * n).sum();
```

체인은 "걸러서(`filter`) 바꾸고(`map`) 더한다(`sum`)"를 그대로 읽을 수 있고, 결과를 쌓아 둘 가변 변수가 필요 없습니다. 반복을 어떻게 돌릴지가 아니라 무엇을 할지가 드러나기 때문에, The Book은 대부분의 Rust 프로그래머가 반복자 스타일을 선호한다고 소개합니다. 예제는 [`06_adapters.rs`](examples/06_adapters.rs)입니다.

### 2.13 직접 반복자 만들기

`Iterator`를 직접 구현하려면 `type Item`을 정하고 `next`를 만들면 됩니다. 반복자는 "지금 어디까지 왔는지"를 기억해야 하므로 그 상태를 필드로 둡니다.

```rust
struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}
```

`next` 하나만 구현했지만 `for` 문도, 모든 어댑터와 소비 메서드도 바로 쓸 수 있습니다.

```rust
let sum: u32 = Counter::new()
    .zip(Counter::new().skip(1))  // (1,2) (2,3) (3,4) (4,5)
    .map(|(a, b)| a * b)          // 2, 6, 12, 20
    .filter(|x| x % 3 == 0)       // 6, 12
    .sum();                       // 18
```

단, `rev`처럼 추가 조건이 붙은 메서드는 예외입니다. `rev`는 뒤에서부터도 꺼낼 수 있는 반복자(`DoubleEndedIterator`)에만 쓸 수 있어서, `Counter`에 부르면 ``error[E0277]: the trait bound `Counter: DoubleEndedIterator` is not satisfied``입니다.

**반복자를 돌려주는 함수**: 어댑터를 엮어 만든 반복자의 타입에는 클로저 타입이 들어 있어 이름을 쓸 수 없습니다. 그래서 반환 위치의 `impl Trait`로 "`u32`를 내놓는 어떤 반복자"라고 적습니다. `Item = u32`는 연관 타입 `Item`이 `u32`라는 뜻입니다.

```rust
fn multiples_of(n: u32) -> impl Iterator<Item = u32> {
    (1..).map(move |i| i * n)
}
```

돌려준 반복자는 함수가 끝난 뒤에 쓰이므로, 클로저가 매개변수 `n`을 빌리면 2.4절과 같은 E0373이 납니다. `move`로 `n`을 클로저 안에 복사해 넣습니다. 예제는 [`07_custom_iterator.rs`](examples/07_custom_iterator.rs)입니다.

### 2.14 `minigrep` 개선하기

ch12에서 만든 `minigrep`을 `projects/minigrep`으로 복사해 세 군데를 고쳤습니다. 프로그램의 동작은 그대로이고, 코드가 짧아지고 불필요한 복제가 사라졌습니다.

**① `main`: 인자 반복자를 그대로 넘기고 `unwrap_or_else` 쓰기**

ch12 판:

```rust
let args: Vec<String> = env::args().collect();

let config = match Config::build(&args) {
    Ok(config) => config,
    Err(err) => {
        eprintln!("인자 문제: {err}");
        // ... 사용법 안내 ...
        process::exit(1);
    }
};
```

ch13 판:

```rust
let config = Config::build(env::args()).unwrap_or_else(|err| {
    eprintln!("인자 문제: {err}");
    // ... 사용법 안내 ...
    process::exit(1);
});
```

- `env::args()`는 반복자를 돌려줍니다. 벡터에 모아 슬라이스를 빌려주는 대신, 반복자의 소유권을 `Config::build`에 그대로 넘깁니다.
- `Result::unwrap_or_else`는 `Ok`면 안의 `Config`를 꺼내고, `Err`일 때만 클로저에 에러 값을 넘겨 부릅니다. `match`와 같은 동작입니다.
- 클로저는 `Config`를 돌려주지 않고 `process::exit(1)`로 끝납니다. `process::exit`의 반환 타입은 "절대 돌아오지 않음"을 뜻하는 `!`이므로(ch12), "`Config`를 돌려주는 `FnOnce`" 자리에 맞습니다.

**② `Config::build`: 슬라이스와 `clone` 대신 반복자와 `next`**

ch12 판:

```rust
pub fn build(args: &[String]) -> Result<Config, &'static str> {
    if args.len() < 3 {
        return Err("인자가 부족합니다");
    }

    let query = args[1].clone();
    let file_path = args[2].clone();
    // ...
```

ch13 판:

```rust
pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
    args.next(); // 첫 값은 실행 파일 경로라서 버립니다.

    let Some(query) = args.next() else {
        return Err("검색어가 없습니다");
    };

    let Some(file_path) = args.next() else {
        return Err("파일 경로가 없습니다");
    };
    // ...
```

- **`clone`이 사라진 이유**: ch12 판은 `args`를 `&[String]`으로 빌려 왔기 때문에 그 안의 `String`을 `Config`로 옮길 수 없어 복제했습니다(ch12의 E0508). 반복자의 `next`는 `String` 자체를 돌려주므로 소유권을 그대로 `Config`로 옮길 수 있습니다.
- **`impl Iterator<Item = String>`**: "`String`을 내놓는 반복자라면 무엇이든"이라는 뜻입니다. `main`은 `env::args()`를, 테스트는 배열로 만든 반복자를 넘길 수 있습니다.
- **`mut args`**: `next`가 반복자의 상태를 바꾸므로 매개변수에 `mut`을 붙입니다.
- **에러가 구체적으로**: 인자를 하나씩 꺼내므로 무엇이 빠졌는지 따로 알릴 수 있습니다. `let ... else`는 [ch6](../ch6-enums-pattern-matching/README.md)에서 배운 문법이며, The Book은 같은 일을 `match`로 씁니다.

The Book처럼 시그니처만 먼저 바꾸고 본문을 그대로 두면 컴파일되지 않습니다. 반복자에는 길이도 인덱스도 없고, `next`로 앞에서부터 꺼낼 수만 있기 때문입니다. 아래는 프로젝트를 저장소 밖에 복사해 그 중간 상태를 재현한 결과입니다(일부 `help:`는 줄였습니다).

```text
error[E0599]: no method named `len` found for type parameter `impl Iterator<Item = String>` in the current scope
  --> src/lib.rs:24:17
   |
23 |     pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
   |                            ---------------------------- method `len` not found for this type parameter
24 |         if args.len() < 3 {
   |                 ^^^

error[E0608]: cannot index into a value of type `impl Iterator<Item = String>`
  --> src/lib.rs:29:25
   |
29 |         let query = args[1].clone();
   |                         ^^^
```

**③ `search`, `search_case_insensitive`: `for`와 `push` 대신 `filter`와 `collect`**

ch12 판:

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

ch13 판:

```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    contents
        .lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}
```

- `filter`의 클로저가 바깥의 `query`를 캡처합니다. 결과를 쌓아 둘 가변 벡터 `results`가 사라졌습니다. The Book은 가변 상태가 줄면 나중에 검색을 병렬로 처리하도록 바꾸기도 쉬워진다고 설명합니다.
- `collect`가 무엇을 만들지는 함수의 반환 타입 `Vec<&'a str>`로 정해집니다.
- `filter`는 항목을 바꾸지 않고 통과시킬지만 정하므로, `search_case_insensitive`의 결과에도 소문자로 바꾼 줄이 아니라 원래 줄이 들어갑니다.
- The Book은 `search_case_insensitive`를 독자의 몫으로 남겨 두었고, 이 프로젝트는 두 함수를 모두 바꿨습니다.

**테스트는 리팩터링의 안전망입니다.** 코드를 바꾸기 전과 후에 같은 테스트가 통과하면 동작이 그대로라는 근거가 됩니다. ch12의 테스트 5개는 그대로 두었고, `build`의 테스트 두 개만 인자를 넘기는 부분을 `&args`에서 `args.into_iter()`로 바꿨습니다. `build`의 인터페이스가 슬라이스에서 반복자로 바뀌었기 때문이며, 확인하는 내용(`assert!`)은 같습니다. 여기에 "무엇이 빠졌는지 알려 주는가"를 확인하는 테스트 `build_reports_missing_argument`를 하나 더했습니다. 이 테스트는 `["minigrep", "duct"].into_iter().map(String::from)`처럼 만든 반복자를 넘기는데, 반복자라면 무엇이든 받는 시그니처 덕분에 가능합니다. 실행 결과는 3절에 있습니다.

### 2.15 성능: 비용 없는 추상화

반복자 체인은 손으로 쓴 `for` 반복문보다 느릴 것 같아 보입니다. 클로저를 넘기고, 어댑터마다 반복자를 감싸기 때문입니다. 하지만 The Book의 벤치마크(셜록 홈스 소설 전체에서 `the` 검색)에서 `for` 판과 반복자 판의 시간은 거의 같았습니다.

이것을 **비용 없는 추상화(zero-cost abstraction)** 라고 부릅니다. 추상화를 써도 실행 중에 추가 비용이 없다는 뜻입니다. The Book은 C++를 설계한 Bjarne Stroustrup의 말을 인용합니다.

> In general, C++ implementations obey the zero-overhead principle: What you don't use, you don't pay for. And further: What you do use, you couldn't hand code any better.
>
> (쓰지 않는 기능에는 비용을 치르지 않는다. 그리고 쓰는 기능은 손으로 짠 코드보다 더 잘 만들 수 없다.)

**왜 비용이 없을까?**

- **클로저는 캡처한 값을 담은 이름 없는 구조체입니다.** 힙에 할당하지 않고, 호출은 실행 중에 무엇을 부를지 찾는 과정 없이 정해진 함수를 바로 부르는 것입니다.
- **어댑터는 클로저의 타입을 제네릭 매개변수로 받습니다.** `filter`는 `filter<P>(self, predicate: P)` 모양의 제네릭 메서드라서, [ch10](../ch10-generics-traits-lifetimes/README.md)에서 배운 단형성화(monomorphization)로 클로저마다 전용 코드가 만들어집니다. 그러면 컴파일러가 클로저 본문을 호출하는 자리에 펼쳐 넣을(인라이닝) 수 있고, 체인 전체가 하나의 반복문처럼 최적화됩니다.
- The Book은 반복문 펼치기(loop unrolling)나 배열 접근의 범위 검사 제거 같은 최적화도 적용되어, 반복자 코드가 손으로 쓴 것과 같은 기계어로 컴파일되는 경우가 많다고 설명합니다.

**직접 재 보기**: [`08_performance.rs`](examples/08_performance.rs)는 같은 일을 하는 `for` 판과 반복자 판의 시간을 잽니다. 두 가지로 실행해 비교해 보세요.

```sh
cargo run -p p0-ch13-closures-iterators --example 08_performance             # 디버그 빌드
cargo run -p p0-ch13-closures-iterators --example 08_performance --release   # 릴리스 빌드
```

시간은 컴퓨터와 실행할 때의 상황에 따라 매번 달라지므로 이 문서에는 숫자를 적지 않습니다. 이 문서를 쓸 때 확인한 경향은 다음과 같습니다.

- **릴리스 빌드**에서는 두 판의 시간이 비슷했습니다.
- **디버그 빌드**에서는 숫자 합계 쪽에서 반복자 판이 눈에 띄게 느렸습니다. 디버그 빌드는 최적화를 끄므로 어댑터와 클로저의 함수 호출이 펼쳐지지 않고 그대로 남기 때문입니다. "비용 없음"은 최적화가 켜진 빌드에서의 이야기이므로, 성능은 항상 `--release`로 잽니다.

예제의 `std::hint::black_box`는 "이 값이 무엇인지 컴파일러가 안다고 가정하지 말라"는 표시입니다. 최적화가 켜지면 컴파일러는 결과를 쓰지 않는 계산을 지워 버리기도 합니다. 실제로 이 예제를 처음 쓸 때 결과를 `black_box`로 감싸지 않았더니, 릴리스 빌드에서 숫자 합계 시간이 `0ns`로 나왔습니다. 재려던 계산이 사라진 것입니다. 그래서 입력과 결과를 모두 `black_box`에 통과시킵니다. 표준 라이브러리 문서도 벤치마크에서 이렇게 쓰라고 안내합니다(다만 최적화를 막는 것은 "최선을 다하는" 수준이지 보장이 아닙니다).

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_closure_syntax.rs`](examples/01_closure_syntax.rs) | 클로저 문법, 환경 캡처, 호출 시점, 한 번 정해진 타입 | 1 |
| [`examples/02_capture_modes.rs`](examples/02_capture_modes.rs) | 캡처 방식 3종과 대여 규칙, `move`, `Copy` 타입과 `move`, 클로저 반환 | 2 |
| [`examples/03_fn_traits.rs`](examples/03_fn_traits.rs) | `FnOnce`/`FnMut`/`Fn`, 클로저를 받는 함수, `unwrap_or_else`, `sort_by_key`, `sort_by` | 3 |
| [`examples/04_iterator_next.rs`](examples/04_iterator_next.rs) | `next`, 게으름, `for`와 `next`의 관계 | 4 |
| [`examples/05_iter_ownership.rs`](examples/05_iter_ownership.rs) | `iter`/`iter_mut`/`into_iter`와 `for`의 세 모양 | 4 |
| [`examples/06_adapters.rs`](examples/06_adapters.rs) | 어댑터와 소비 메서드, 환경을 캡처하는 `filter`, 반복문을 체인으로 | 5 |
| [`examples/07_custom_iterator.rs`](examples/07_custom_iterator.rs) | 직접 구현한 `Iterator`, `impl Iterator` 반환 | 4, 5 |
| [`examples/08_performance.rs`](examples/08_performance.rs) | `for`와 반복자의 시간 비교, 디버그와 릴리스 빌드 | 5 |
| [`projects/minigrep`](projects/minigrep) | ch12의 `minigrep`을 반복자와 클로저로 개선 | 3, 5 |
| [`compile_fail/`](compile_fail) | 클로저와 반복자에서 자주 만나는 컴파일 에러 | 1~4 |

예제 명령은 저장소 루트에서 실행한다고 보고 썼습니다.

### 01_closure_syntax — 클로저 문법

```console
$ cargo run -q -p p0-ch13-closures-iterators --example 01_closure_syntax
== 1. 함수와 클로저 ==
add_one_v1(5) = 6
add_one_v2(5) = 6
add_one_v4(5) = 6
  (add_one_verbose 안에서 5 + 1 = 6)
add_one_verbose(5) = 6

== 2. 환경 캡처 ==
add_base(1) = 11

== 3. 정의와 호출 ==
클로저를 정의했습니다(아직 본문은 실행되지 않음)
  클로저 본문이 실행되었습니다
  클로저 본문이 실행되었습니다

== 4. 한 번 정해진 타입은 고정 ==
identity(String) = hello
identity_num(5) = 5
```

볼 점:

- `add_one_verbose(5)`를 출력하는 줄에서 클로저 안의 `println!`이 먼저 찍힙니다. `println!`의 인자를 계산하려고 클로저를 먼저 부르기 때문입니다.
- 3번 구역에서 정의 직후에는 아무것도 출력되지 않고, 부를 때마다 본문이 실행됩니다.
- 4번 구역에서 `identity`를 정수로 부르지 않고 `identity_num`을 따로 만든 이유는 4.1절의 에러 때문입니다.

### 02_capture_modes — 캡처 방식과 `move`

```console
$ cargo run -q -p p0-ch13-closures-iterators --example 02_capture_modes
== 1. 불변 대여 ==
정의 전: [1, 2, 3]
호출 전: [1, 2, 3]
클로저 안: [1, 2, 3]
호출 후: [1, 2, 3]

== 2. 가변 대여 ==
호출 후: [1, 2, 3, 7, 7]

== 3. 소유권 이동 ==
새 소유자: 페리스

== 4. move ==
move 클로저 안: [1, 2, 3]
move 클로저 안: [1, 2, 3]

== 5. move와 Copy 타입 ==
increment() = 1
increment() = 2
바깥의 count = 0

== 6. 클로저 반환 ==
a: 1
a: 2
b: 1
```

볼 점:

- 1번 구역은 불변 대여라서 클로저를 정의한 뒤에도, 부르기 전에도 `list`를 읽을 수 있습니다.
- 2번 구역은 가변 대여라서 정의와 마지막 호출 사이에 `list`를 읽는 코드가 없습니다. 두 번 불러 `7`이 두 번 들어갔습니다.
- 4번 구역의 `move` 클로저는 `list`를 소유하지만 읽기만 하므로 두 번 부를 수 있습니다.
- 5번 구역에서 클로저 안의 `count`는 바깥 `count`의 복사본입니다.
- 6번 구역의 두 카운터는 서로 다른 `count`를 소유합니다.

### 03_fn_traits — `Fn` 트레이트와 클로저를 받는 함수

```console
$ cargo run -q -p p0-ch13-closures-iterators --example 03_fn_traits
== 1. 읽기만 하는 클로저 (Fn) ==
  안녕하세요
  안녕하세요
  안녕하세요
  안녕하세요
  안녕하세요

== 2. 캡처한 값을 바꾸는 클로저 (FnMut) ==
count = 3

== 3. 캡처한 값을 내보내는 클로저 (FnOnce) ==
  가져간 값: 페리스

== 4. 함수 이름 넘기기 ==
  함수 say_hello 실행
  함수 say_hello 실행

== 5. unwrap_or_else ==
Some일 때: 러스타시안
  (기본값 클로저 실행)
None일 때: 손님
  파싱 실패: invalid digit found in string
number = 0

== 6. sort_by_key, sort_by ==
너비 오름차순: [Rectangle { width: 2, height: 5 }, Rectangle { width: 3, height: 5 }, Rectangle { width: 7, height: 12 }, Rectangle { width: 10, height: 1 }]
키 클로저 호출 횟수: 12
높이 오름차순, 같으면 너비 내림차순: [Rectangle { width: 10, height: 1 }, Rectangle { width: 3, height: 5 }, Rectangle { width: 2, height: 5 }, Rectangle { width: 7, height: 12 }]
```

볼 점:

- 1번 구역의 `안녕하세요` 다섯 줄은 `call_twice`(2번), `call_twice_mut`(2번), `call_once`(1번)에서 나온 것입니다. 이 클로저는 `&greeting`만 들고 있어서 통째로 복사(`Copy`)되므로, 같은 클로저를 세 함수에 차례로 넘길 수 있습니다.
- 2번 구역의 `count`가 3인 것은 `call_twice_mut`에서 두 번, `call_once`에서 한 번 늘었기 때문입니다.
- 5번 구역에서 `Some`일 때는 `(기본값 클로저 실행)`이 나오지 않습니다. 클로저가 불리지 않았다는 뜻입니다.
- 6번 구역의 키 클로저 호출 횟수(원소 4개에 12번)는 표준 라이브러리의 정렬 구현에 달린 값이라 Rust 버전에 따라 달라질 수 있습니다. 높이가 5로 같은 두 사각형은 `then`으로 정한 두 번째 기준(너비 내림차순)에 따라 3이 2보다 앞에 옵니다.

### 04_iterator_next — `next`, 게으름, `for`

```console
$ cargo run -q -p p0-ch13-closures-iterators --example 04_iterator_next
== 1. next ==
Some(1)
Some(2)
Some(3)
None
v1은 그대로: [1, 2, 3]
Some('러') Some('스')

== 2. 게으름 ==
map을 호출했습니다(아직 아무 처리도 하지 않음)
  map이 1을(를) 처리
for가 2을(를) 받음
  map이 2을(를) 처리
for가 4을(를) 받음
  map이 3을(를) 처리
for가 6을(를) 받음
처음 다섯 제곱수: [1, 4, 9, 16, 25]

== 3. for와 next ==
  for 문: 하나
  for 문: 둘
  for 문: 셋
  손으로 쓴 반복: 하나
  손으로 쓴 반복: 둘
  손으로 쓴 반복: 셋
```

볼 점:

- 네 번째 `next`는 `None`입니다. 반복자가 끝났다는 신호입니다.
- 2번 구역에서 "map 처리"와 "for 받음"이 번갈아 나옵니다. `map`이 원소를 한꺼번에 처리해 두는 것이 아니라, `for`가 하나를 요청할 때마다 하나씩 처리합니다.
- 끝이 없는 `(1..)`도 `take(5)` 덕분에 다섯 개만 계산하고 끝납니다.

### 05_iter_ownership — `iter`, `iter_mut`, `into_iter`

```console
$ cargo run -q -p p0-ch13-closures-iterators --example 05_iter_ownership
== 1. iter() → &T ==
  안녕, 가람
  안녕, 나래
  안녕, 다온
names는 그대로: ["가람", "나래", "다온"]

== 2. iter_mut() → &mut T ==
5점씩 올린 점수: [75, 90, 97]

== 3. into_iter() → T ==
옮겨 담은 이름: ["가람", "나래", "다온"]

== 4. for x in &v / &mut v / v ==
&mut로 고친 뒤: ["rust!", "crab!"]
  소유한 값: rust!
  소유한 값: crab!
```

볼 점:

- 반복문 안의 `let _check: &String = name;` 같은 줄은 반복 변수의 타입을 확인하려고 넣었습니다. 타입이 다르면 컴파일되지 않으므로, 예제가 빌드된다는 것이 표의 내용이 맞다는 증거입니다. 직접 `&String`을 `String`으로 바꿔 컴파일해 보세요.
- 3번 구역 뒤에서 `names`를 쓰면 E0382입니다. `into_iter()`가 `names`의 소유권을 가져갔습니다.

### 06_adapters — 어댑터와 소비 메서드

```console
$ cargo run -q -p p0-ch13-closures-iterators --example 06_adapters
== 1. 어댑터 ==
map(n * 10):    [10, 20, 30, 40, 50, 60]
filter(짝수):   [2, 4, 6]
rev():          [6, 5, 4, 3, 2, 1]
take(3):        [1, 2, 3]
skip(4):        [5, 6]
enumerate():    [(0, "월"), (1, "화"), (2, "수")]
zip():          [("가람", 88), ("나래", 72), ("다온", 95)]

== 2. 소비 메서드 ==
sum:   315
count: 80점 이상 2명
max:   Some(95)
min:   None
collect(String): 요세하녕안

== 3. 환경을 캡처하는 filter ==
80점 이상: ["가람", "다온"]
90점 이상: ["다온"]

== 4. for 반복문 vs 어댑터 체인 ==
for 반복문: 220
어댑터 체인: 220
```

볼 점:

- `zip`은 이름 3개와 점수 4개를 짝지었으므로 짧은 쪽에 맞춰 3쌍입니다. `sum`은 점수 4개를 모두 더한 값입니다.
- 빈 배열의 `min()`은 `None`입니다. 그래서 `max`와 `min`은 `Option`을 돌려줍니다.
- 문자를 `collect`로 모으면 `String`이 됩니다. `chars().rev()`로 글자 단위로 뒤집었으므로 한글도 깨지지 않습니다(ch8의 UTF-8).

### 07_custom_iterator — 직접 만든 반복자

```console
$ cargo run -q -p p0-ch13-closures-iterators --example 07_custom_iterator
== 1. next ==
next 여섯 번: Some(1) Some(2) Some(3) Some(4) Some(5) None

== 2. for ==
for로 받은 값: 1 2 3 4 5

== 3. 어댑터 ==
합: 18
짝수만: [2, 4]

== 4. impl Iterator ==
3의 배수 다섯 개: [3, 6, 9, 12, 15]
```

볼 점:

- `Counter`에는 `next`만 구현했는데 `zip`, `skip`, `map`, `filter`, `sum`, `collect`를 모두 썼습니다.
- `multiples_of(3)`은 끝이 없는 반복자지만 `take(5)`로 다섯 개만 꺼냈습니다.

### 08_performance — 반복문과 반복자의 성능

```console
$ cargo run -q -p p0-ch13-closures-iterators --example 08_performance
디버그 빌드로 실행했습니다(최적화 꺼짐).

[1] "the" 검색: 180000줄 중에서
  for 반복문: <시간>
  반복자    : <시간>
  찾은 줄 수: 40000 / 40000

[2] 짝수의 제곱의 합: 수 2000000개
  for 반복문: <시간>
  반복자    : <시간>
  결과: 1333331333334000000 / 1333331333334000000
```

`<시간>` 자리에는 `21.4ms`나 `375µs`처럼 실행할 때마다 다른 값이 나옵니다. 각 측정은 다섯 번 재서 가장 빠른 값을 보여 줍니다. `--release`를 붙이면 첫 줄이 `릴리스 빌드로 실행했습니다(최적화 켜짐).`으로 바뀝니다. 두 빌드의 시간을 비교해 보세요(2.15절). 두 판의 결과(찾은 줄 수, 합계)가 같다는 것도 확인합니다. 빠르더라도 결과가 다르면 비교할 의미가 없기 때문입니다.

### projects/minigrep — 반복자와 클로저로 개선한 `minigrep`

워크스페이스와 분리된 독립 프로젝트이므로 그 디렉터리에서 실행합니다.

```sh
cd part0-getting-started/ch13-closures-iterators/projects/minigrep
```

**테스트**: ch12의 테스트 5개와 새 테스트 1개가 모두 통과합니다. 테스트는 여러 스레드에서 동시에 실행되므로 `test ... ok` 줄의 순서는 실행할 때마다 달라질 수 있습니다.

```console
$ cargo test
```

```text
running 6 tests
test tests::build_needs_query_and_file_path ... ok
test tests::build_reports_missing_argument ... ok
test tests::build_reads_query_and_file_path ... ok
test tests::case_sensitive ... ok
test tests::case_insensitive ... ok
test tests::hangul ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(앞뒤의 `Compiling`, `Running` 줄과 `src/main.rs`·문서 테스트의 `0 tests` 결과는 뺐습니다.)

**검색**: 결과는 ch12 판과 같습니다.

```console
$ cargo run -q -- frog poem.txt
How public, like a frog
$ cargo run -q -- body poem.txt
I'm nobody! Who are you?
Are you nobody, too?
How dreary to be somebody!
$ IGNORE_CASE=1 cargo run -q -- to poem.txt
Are you nobody, too?
How dreary to be somebody!
To tell your name the livelong day
To an admiring bog!
$ cargo run -q -- 바람 poem_ko.txt
잎새에 이는 바람에도
오늘 밤에도 별이 바람에 스치운다.
```

**에러**: 무엇이 빠졌는지에 따라 메시지가 달라졌습니다. ch12 판은 두 경우 모두 `인자가 부족합니다`였습니다. 종료 코드는 그대로 1입니다.

```console
$ cargo run -q
인자 문제: 검색어가 없습니다
사용법: minigrep <검색어> <파일 경로>
(대소문자를 무시하려면 환경 변수 IGNORE_CASE를 설정합니다)
$ cargo run -q -- frog
인자 문제: 파일 경로가 없습니다
사용법: minigrep <검색어> <파일 경로>
(대소문자를 무시하려면 환경 변수 IGNORE_CASE를 설정합니다)
$ echo $?
1
$ cargo run -q -- to nofile.txt
실행 오류: No such file or directory (os error 2)
```

ch12 판과 나란히 비교하려면 저장소 루트에서 `diff -u part0-getting-started/ch12-io-project-minigrep/projects/minigrep/src/lib.rs part0-getting-started/ch13-closures-iterators/projects/minigrep/src/lib.rs`를 실행해 보세요. 실습이 끝나면 `cargo clean`으로 `target/`을 지울 수 있습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part0-getting-started/ch13-closures-iterators
rustc --edition 2024 compile_fail/e0308_closure_type_fixed.rs --out-dir /tmp/rust-ch13
```

### 4.1 같은 클로저를 다른 타입으로 호출 — E0308

파일: [`compile_fail/e0308_closure_type_fixed.rs`](compile_fail/e0308_closure_type_fixed.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_closure_type_fixed.rs:17:22
   |
17 |     let n = identity(5);
   |             -------- ^ expected `String`, found integer
   |             |
   |             arguments to this function are incorrect
   |
note: expected because the closure was earlier called with an argument of type `String`
  --> compile_fail/e0308_closure_type_fixed.rs:16:22
   |
16 |     let s = identity(String::from("hello"));
   |             -------- ^^^^^^^^^^^^^^^^^^^^^ expected because this argument is of type `String`
   |             |
   |             in this closure call
note: closure parameter defined here
  --> compile_fail/e0308_closure_type_fixed.rs:14:21
   |
14 |     let identity = |x| x;
   |                     ^
help: try using a conversion method
   |
17 |     let n = identity(5.to_string());
   |                       ++++++++++++
```

- **읽는 법**: `expected String, found integer`는 "`String`이 와야 하는데 정수가 왔다"는 뜻입니다. 첫 번째 `note:`가 그 이유를 짚습니다. 클로저가 앞에서 `String`으로 불렸기 때문입니다. 두 번째 `note:`는 타입이 정해진 매개변수 `x`를 가리킵니다.
- **원인**: 타입 표기가 없는 클로저는 처음 호출된 모습으로 매개변수와 반환값의 타입을 하나로 정합니다(2.2절).
- **해결**: 타입마다 클로저를 따로 만듭니다(예제 01의 `identity_num`). 정말로 여러 타입을 받는 하나의 코드가 필요하면 제네릭 함수(ch10)로 만듭니다. 컴파일러의 `help:`는 `5.to_string()`으로 바꾸라고 제안하지만, 이것은 정수를 `String`으로 바꿔 타입을 맞추는 것이라 원래 의도(정수를 그대로 돌려받기)와는 다릅니다.

### 4.2 가변 대여를 캡처한 클로저가 살아 있는데 원본을 읽음 — E0502

파일: [`compile_fail/e0502_mutable_capture_alive.rs`](compile_fail/e0502_mutable_capture_alive.rs)

```text
error[E0502]: cannot borrow `list` as immutable because it is also borrowed as mutable
  --> compile_fail/e0502_mutable_capture_alive.rs:18:22
   |
16 |     let mut borrows_mutably = || list.push(7);
   |                               -- ---- first borrow occurs due to use of `list` in closure
   |                               |
   |                               mutable borrow occurs here
17 |
18 |     println!("호출 전: {list:?}");
   |                         ^^^^ immutable borrow occurs here
19 |     borrows_mutably();
   |     --------------- mutable borrow later used here
```

- **읽는 법**: 표시가 세 곳입니다. 16행의 `||`에서 가변 대여가 시작되었고(`list`를 클로저 안에서 쓰기 때문이라고 `----`가 덧붙입니다), 18행에서 불변 대여를 하려 했으며, 19행에서 그 가변 대여가 아직 쓰이고 있습니다.
- **원인**: `list.push(7)`을 하는 클로저는 `list`의 가변 참조를 캡처합니다. 그 대여는 클로저를 정의한 곳부터 마지막으로 부르는 곳까지 이어지고, 그 사이에는 다른 대여가 있을 수 없습니다(ch4의 대여 규칙). 이 규칙이 없다면 `push`가 벡터를 재할당하는 동안 다른 곳에서 예전 메모리를 읽을 수 있습니다.
- **해결**: 클로저를 쓰기 전에 출력하거나, 클로저의 마지막 호출 뒤로 출력을 옮깁니다. 예제 02의 2번 구역이 고친 모습입니다.

### 4.3 `move` 클로저로 옮긴 값을 다시 사용 — E0382

파일: [`compile_fail/e0382_use_after_move_closure.rs`](compile_fail/e0382_use_after_move_closure.rs)

```text
error[E0382]: borrow of moved value: `list`
  --> compile_fail/e0382_use_after_move_closure.rs:19:23
   |
14 |     let list = vec![1, 2, 3];
   |         ---- move occurs because `list` has type `Vec<i32>`, which does not implement the `Copy` trait
15 |
16 |     let print_list = move || println!("클로저 안: {list:?}");
   |                      -------                       ---- variable moved due to use in closure
   |                      |
   |                      value moved into closure here
...
19 |     println!("클로저 밖: {list:?}");
   |                           ^^^^ value borrowed here after move
   |
help: consider cloning the value before moving it into the closure
   |
16 ~     let value = list.clone();
17 ~     let print_list = move || println!("클로저 안: {value:?}");
   |
```

- **읽는 법**: `value moved into closure here`가 `move ||`를, `variable moved due to use in closure`가 본문의 `list`를 가리킵니다. 그 뒤 19행에서 옮겨 간 값을 빌리려 했다는 뜻입니다.
- **원인**: `move`가 `list`의 소유권을 클로저 안으로 옮겼습니다. `Vec`은 `Copy`가 아니므로 원래 변수는 더 쓸 수 없습니다(ch4의 이동).
- **해결**: 클로저가 소유할 필요가 없다면 `move`를 빼서 빌리게 합니다. 클로저를 함수 밖으로 돌려주는 경우처럼 꼭 소유해야 한다면, 컴파일러의 `help:`대로 복제본을 만들어 옮겨 넣습니다.

### 4.4 `sort_by_key`에 캡처한 값을 밖으로 옮기는 클로저를 넘김 — E0507

파일: [`compile_fail/e0507_move_out_in_fnmut.rs`](compile_fail/e0507_move_out_in_fnmut.rs)

```text
error[E0507]: cannot move out of `value`, a captured variable in an `FnMut` closure
  --> compile_fail/e0507_move_out_in_fnmut.rs:41:30
   |
38 |     let value = String::from("closure called");
   |         -----   ------------------------------ move occurs because `value` has type `String`, which does not implement the `Copy` trait
   |         |
   |         captured outer variable
39 |
40 |     list.sort_by_key(|r| {
   |                      --- captured by this `FnMut` closure
41 |         sort_operations.push(value);
   |                              ^^^^^ `value` is moved here
   |
help: `Fn` and `FnMut` closures require captured values to be able to be consumed multiple times, but `FnOnce` closures may consume them only once
  --> /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/alloc/src/slice.rs:249:11
help: consider cloning the value if the performance cost is acceptable
   |
41 |         sort_operations.push(value.clone());
   |                                   ++++++++
```

- **읽는 법**: "`FnMut` 클로저가 캡처한 변수 `value`를 밖으로 옮길 수 없다"는 뜻입니다. 첫 번째 `help:`가 핵심입니다. `Fn`과 `FnMut` 클로저는 여러 번 불려도 캡처한 값을 쓸 수 있어야 하는데, 값을 옮기는 일은 한 번만 할 수 있습니다. 이 `help:`가 가리키는 표준 라이브러리 소스 위치는 `sort_by_key`의 `FnMut` 바운드이며, 경로의 해시 값은 툴체인마다 다릅니다.
- **원인**: 정렬하면서 클로저가 몇 번 불리는지 세려고 `value`를 벡터에 넣었습니다. 이 클로저는 캡처한 값을 밖으로 옮기므로 `FnOnce`만 구현하는데, `sort_by_key`는 원소마다 여러 번 부르므로 `FnMut`을 요구합니다(2.7절).
- **해결**: 값을 옮기지 않도록 본문을 바꿉니다. 호출 횟수를 세려면 바깥의 정수 카운터를 `+= 1` 하면 됩니다(예제 03의 6번 구역). 두 번째 `help:`처럼 `value.clone()`을 넣어도 컴파일은 되지만, 호출할 때마다 문자열을 복제해 쌓는 것이라 "횟수 세기"라는 목적에는 맞지 않습니다.

### 4.5 다 쓴 반복자를 다시 사용 — E0382

파일: [`compile_fail/e0382_consumed_iterator.rs`](compile_fail/e0382_consumed_iterator.rs)

```text
error[E0382]: use of moved value: `v1_iter`
  --> compile_fail/e0382_consumed_iterator.rs:18:17
   |
16 |     let v1_iter = v1.iter();
   |         ------- move occurs because `v1_iter` has type `std::slice::Iter<'_, i32>`, which does not implement the `Copy` trait
17 |     let total: i32 = v1_iter.sum();
   |                              ----- `v1_iter` moved due to this method call
18 |     let count = v1_iter.count();
   |                 ^^^^^^^ value used here after move
   |
note: `std::iter::Iterator::sum` takes ownership of the receiver `self`, which moves `v1_iter`
```

- **읽는 법**: `note:`가 원인을 직접 말합니다. `sum`이 `self`를 받아 `v1_iter`의 소유권을 가져갔습니다. 16행의 설명에서 `v1.iter()`가 만든 반복자의 실제 타입이 `std::slice::Iter<'_, i32>`라는 것도 볼 수 있습니다.
- **원인**: 소비 메서드는 반복자를 끝까지 써 버리므로, 소유권을 가져가 다시 쓰지 못하게 합니다. 설령 다시 쓸 수 있더라도 이미 끝까지 간 반복자라 더 내놓을 값이 없습니다.
- **해결**: 결과가 여러 개 필요하면 원본에서 반복자를 새로 만듭니다(`v1.iter().count()`). 컴파일러가 제안하는 `v1_iter.clone().sum()`도 동작하지만, 원본에서 새로 만드는 쪽이 의도가 분명합니다.

### 4.6 어댑터만 만들고 소비하지 않음 — 경고 `unused_must_use`

`v1.iter().map(|x| x + 1);`처럼 어댑터를 만들기만 하고 소비하지 않으면 에러가 아니라 경고(``unused `Map` that must be used``)가 납니다. 빌드는 되지만 클로저는 한 번도 실행되지 않습니다. 경고 전문과 해결 방법은 2.9절에 있습니다. 경고를 무시하고 넘어가면 "분명히 `map`으로 처리했는데 아무 일도 일어나지 않는" 버그가 됩니다.

## 5. 연습 문제

### 연습 1. 정렬 기준 바꾸기

학생 목록(`name: String`, `score: u32`)을 점수 내림차순으로, 점수가 같으면 이름 가나다순으로 정렬하세요. 그다음 같은 목록을 이름순으로 다시 정렬하면서 비교 클로저가 몇 번 불렸는지 세어 출력하세요.

<details>
<summary>힌트</summary>

- 점수 내림차순은 `b.score.cmp(&a.score)`, 이름 오름차순은 `a.name.cmp(&b.name)`입니다. 두 비교를 `then`으로 이으면 앞의 결과가 같을 때만 뒤의 결과를 씁니다.
- 횟수를 세려면 `let mut count = 0;`을 두고 `sort_by(|a, b| { count += 1; a.name.cmp(&b.name) })`처럼 씁니다. 이 클로저는 `count`를 바꾸므로 `FnMut`이고, `sort_by`는 `FnMut`을 받으므로 됩니다.
- 횟수는 정렬 알고리즘에 따라 정해지므로 원소 수나 처음 순서를 바꾸면 달라집니다.

</details>

### 연습 2. 피보나치 반복자

0, 1, 1, 2, 3, 5, 8, …을 끝없이 내놓는 `Fibonacci` 반복자를 직접 구현하세요. 그리고 어댑터를 써서 (1) 처음 10개 `[0, 1, 1, 2, 3, 5, 8, 13, 21, 34]`, (2) 짝수인 피보나치 수 처음 5개 `[0, 2, 8, 34, 144]`를 벡터로 모으세요.

<details>
<summary>힌트</summary>

- 상태로 "현재 값"과 "다음 값" 두 필드(`curr`, `next`)를 둡니다. `next` 메서드는 지금의 `curr`를 돌려줄 값으로 기억해 두고, 두 필드를 한 칸씩 당긴 뒤 `Some(기억한 값)`을 돌려줍니다.
- 끝이 없는 반복자이므로 `next`는 항상 `Some`을 돌려줍니다. 반드시 `take`로 개수를 정한 뒤 `collect`하세요. 그렇지 않으면 `u64`가 넘칠 때까지 계속 계산하다 디버그 빌드에서 오버플로 패닉이 납니다.
- (2)는 `filter(|n| n % 2 == 0).take(5)` 순서입니다. `take(5).filter(...)`로 쓰면 결과가 어떻게 달라지는지도 확인해 보세요.

</details>

### 연습 3. `minigrep`에 줄 번호 붙이기

[ch12](../ch12-io-project-minigrep/README.md)의 연습 1을 반복자로 풀어 봅니다. `projects/minigrep`을 저장소 밖으로 복사한 뒤(`cp -R part0-getting-started/ch13-closures-iterators/projects/minigrep /tmp/minigrep-ch13`), 찾은 줄 앞에 줄 번호를 붙여 출력하세요. `cargo run -q -- frog poem.txt`의 결과가 `7: How public, like a frog`가 되어야 합니다. 테스트를 먼저 쓰고 시작합니다.

<details>
<summary>힌트</summary>

- 반환 타입은 `Vec<(usize, &'a str)>`입니다.
- `contents.lines().enumerate()`는 `(0부터 시작하는 인덱스, 줄)`을 내놓습니다. `filter`로 거른 뒤 `map(|(i, line)| (i + 1, line))`으로 1부터 시작하는 번호로 바꿉니다.
- `filter`의 클로저는 튜플의 참조를 받으므로 `|(_, line)| line.contains(query)`처럼 패턴으로 꺼낼 수 있습니다.

</details>

### 연습 4. `search`가 벡터 대신 반복자를 돌려주게 하기

`search`와 `search_case_insensitive`의 반환 타입을 `Vec<&'a str>`에서 `impl Iterator<Item = &'a str>`로 바꾸고 `collect()`를 지워 보세요. 결과를 벡터에 모으지 않으므로 찾은 줄을 바로바로 출력할 수 있습니다. 컴파일 에러를 하나씩 고쳐 `cargo test`가 다시 통과하게 만드세요.

<details>
<summary>힌트</summary>

- 먼저 `filter`의 클로저에서 E0373(`closure may outlive the current function, but it borrows query`)이 납니다. 돌려준 반복자는 함수가 끝난 뒤에 쓰이는데 클로저가 매개변수 `query`를 빌리고 있기 때문입니다. 2.4절처럼 `move`를 붙입니다.
- 다음으로 `run`의 `if`/`else`에서 E0308이 납니다. 두 함수가 돌려주는 `impl Iterator`는 서로 다른 타입이라 한 변수에 담을 수 없습니다([ch10](../ch10-generics-traits-lifetimes/README.md)에서 본 반환 위치 `impl Trait`의 제약과 같습니다). 각 갈래 안에서 바로 `for line in search(...) { println!("{line}"); }`로 출력하면 됩니다.
- 테스트의 비교는 `search(query, contents).collect::<Vec<_>>()`처럼 모은 뒤에 합니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [13. Functional Language Features: Iterators and Closures](https://doc.rust-lang.org/book/ch13-00-functional-features.html)
  - [13.1 Closures](https://doc.rust-lang.org/book/ch13-01-closures.html): 셔츠 나눠 주기 예제(`unwrap_or_else`), 캡처 방식 3종, `Fn` 트레이트, `sort_by_key`의 E0507이 나옵니다.
  - [13.2 Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html): `Iterator` 트레이트와 `next`, 소비 메서드와 어댑터, 신발 크기로 거르는 `filter` 예제입니다.
  - [13.3 Improving Our I/O Project](https://doc.rust-lang.org/book/ch13-03-improving-our-io-project.html): 2.14절의 `minigrep` 개선과 같은 내용입니다.
  - [13.4 Performance in Loops vs. Iterators](https://doc.rust-lang.org/book/ch13-04-performance.html): 벤치마크와 비용 없는 추상화를 설명합니다.
- Rust By Example
  - [Closures](https://doc.rust-lang.org/rust-by-example/fn/closures.html): 타입을 적은 클로저와 생략한 클로저, 함수는 바깥 변수를 쓸 수 없다는 것을 보여 줍니다.
  - [Capturing](https://doc.rust-lang.org/rust-by-example/fn/closures/capture.html): 캡처 방식 3종과 `move`. 2.3~2.4절과 같은 내용을 다른 예제로 봅니다.
  - [As input parameters](https://doc.rust-lang.org/rust-by-example/fn/closures/input_parameters.html): `Fn`/`FnMut`/`FnOnce` 바운드로 클로저를 받는 함수. "컴파일러는 변수마다 가장 덜 제한적인 방식으로 캡처한다"는 설명이 있습니다.
  - [Type anonymity](https://doc.rust-lang.org/rust-by-example/fn/closures/anonymity.html): 클로저 타입에 이름이 없어서 클로저를 받는 함수가 제네릭이어야 하는 이유입니다(2.2절).
  - [As output parameters](https://doc.rust-lang.org/rust-by-example/fn/closures/output_parameters.html): `impl Fn`으로 클로저를 돌려줄 때 `move`가 필요한 이유입니다(2.4절).
  - [Iterator::any](https://doc.rust-lang.org/rust-by-example/fn/closures/closure_examples/iter_any.html): 표준 라이브러리 메서드가 `FnMut` 바운드로 클로저를 받는 예이고, `iter`와 `into_iter`에서 클로저가 받는 값이 어떻게 다른지 보여 줍니다.
  - [Iterators](https://doc.rust-lang.org/rust-by-example/trait/iter.html): `Fibonacci` 반복자를 직접 구현하고 `take`, `skip`을 씁니다. 연습 2를 푼 뒤에 비교해 보세요.
  - [impl Trait](https://doc.rust-lang.org/rust-by-example/trait/impl_trait.html): 반복자와 클로저를 `impl Trait`로 돌려주는 예(`combine_vecs`, `make_adder_function`, `double_positives`)가 있습니다. ch10에서 미뤄 둔 부분을 이제 읽을 수 있습니다.
- Rustlings
  - [`18_iterators`](https://github.com/rust-lang/rustlings/tree/main/exercises/18_iterators): `iterators1.rs`~`iterators5.rs`. `next`, `map`, `collect`, 해시 맵 값의 반복 등을 연습합니다. 클로저만 따로 다루는 연습 디렉터리는 없습니다.
- API 문서
  - [`std::iter::Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html): 필수 메서드 `next`와 기본 구현 메서드 전체 목록입니다. 이 챕터에서 다루지 않은 어댑터(`fold`, `chain`, `peekable` 등)는 [Part 1 ch5](../../part1-standard-lib/ch5-iterators/README.md)에서 다룹니다.
  - [`std::ops::FnMut`](https://doc.rust-lang.org/std/ops/trait.FnMut.html): `FnOnce`, `Fn`과의 포함 관계와 바운드를 고르는 기준이 적혀 있습니다. [`FnOnce`](https://doc.rust-lang.org/std/ops/trait.FnOnce.html), [`Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html)도 함께 보세요.
  - [`move` 키워드](https://doc.rust-lang.org/std/keyword.move.html): `move` 클로저도 본문이 하는 일에 따라 `Fn`이나 `FnMut`을 구현할 수 있다고 설명합니다.
  - [`Option::unwrap_or_else`](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or_else), [`slice::sort_by_key`](https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by_key): 2.7절의 바운드를 확인할 수 있습니다.
  - [`std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html): 벤치마크에서 입력과 결과를 감싸라는 안내와, 최적화를 막는 것이 보장이 아니라는 주의 사항이 있습니다.
