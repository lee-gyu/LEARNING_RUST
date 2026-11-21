# ch19. 고급 기능: unsafe부터 매크로까지

> 이 챕터는 The Book **20장**에 해당합니다. 이 저장소는 Book 14장(Cargo)을 Part 3으로 옮겼기 때문에 ch14부터 저장소 챕터 번호가 Book 장 번호보다 하나 작습니다.

지금까지 배운 것만으로도 대부분의 Rust 코드를 읽고 쓸 수 있습니다. 이 챕터는 **평소에는 잘 쓰지 않지만, 라이브러리 코드나 에러 메시지에서 만났을 때 알아볼 수 있어야 하는** 기능들을 훑어봅니다. 다섯 갈래로 나뉩니다.

- **unsafe Rust**: 컴파일러가 확인할 수 없는 일을 프로그래머가 책임지고 하는 방법입니다(2.1~2.7절).
- **고급 트레이트**: 연관 타입, 기본 제네릭 타입 매개변수와 연산자 오버로딩, 같은 이름의 메서드 구분하기, 슈퍼트레이트입니다(2.8~2.11절).
- **고급 타입**: 뉴타입 패턴, 타입 별칭, never 타입 `!`, 동적 크기 타입과 `Sized`입니다(2.12~2.15절).
- **고급 함수와 클로저**: 함수 포인터와 클로저 반환하기입니다(2.16~2.17절).
- **매크로**: `macro_rules!`로 만드는 선언적 매크로와, 코드를 받아 코드를 만드는 절차적 매크로입니다(2.18~2.20절).

앞 챕터에서 "`ch19-advanced-features`에서 다룬다"고 미뤄 둔 것들이 여기서 풀립니다. [ch13](../ch13-closures-iterators/README.md)의 `Iterator::Item`과 [ch14](../ch14-smart-pointers/README.md)의 `Deref::Target`(연관 타입), [ch8](../ch8-collections/README.md)의 `String + &str`(연산자 오버로딩), [ch10](../ch10-generics-traits-lifetimes/README.md)의 고아 규칙을 피하는 방법(뉴타입), [ch9](../ch9-error-handling/README.md)의 `io::Result<T>`(타입 별칭), [ch6](../ch6-enums-pattern-matching/README.md)의 `` expected `!` ``(never 타입), ch14의 `RefCell`과 [ch15](../ch15-concurrency/README.md)의 `Send`/`Sync` 뒤에 있는 `unsafe`입니다.

각 기능은 "무엇이고, 언제 필요하며, 어디서 더 깊이 다루는가"를 중심으로 설명합니다. 깊은 내용은 Part 9(The Reference)와 Part 10(The Rustonomicon)이 맡습니다.

- 패키지: `p0-ch19-advanced-features`
- 독립 프로젝트: `projects/hello_macro` (derive 매크로를 직접 만드는 세 크레이트 워크스페이스)
- 기준 문서: [The Book 20장 Advanced Features](https://doc.rust-lang.org/book/ch20-00-advanced-features.html)
- 사용 크레이트: `projects/hello_macro`에서만 `syn` 3.0.10, `quote` 1.0.47 (챕터 예제는 외부 크레이트를 쓰지 않습니다)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, Edition 2024, macOS(aarch64). 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `unsafe`가 대여 검사기를 끄는 것이 아니라 다섯 가지 능력만 더한다는 것을 설명할 수 있다. 원시 포인터를 만들고 역참조하고, unsafe 함수를 정의·호출하고, `split_at_mut`처럼 unsafe 코드를 안전한 함수로 감쌀 수 있다. `unsafe extern "C"`로 C 함수를 부르고, 가변 정적 변수와 unsafe 트레이트를 쓸 수 있으며, Edition 2024에서 바뀐 unsafe 문법을 알아보고 고칠 수 있다.
2. 연관 타입과 제네릭 트레이트의 차이를 설명할 수 있다. `Add`를 구현해 `Point + Point`와 서로 다른 타입의 덧셈을 만들 수 있고, 같은 이름의 메서드·연관 함수를 완전 정규화 문법(`<Type as Trait>::f`)으로 구분하고, 슈퍼트레이트를 정의할 수 있다.
3. 뉴타입으로 고아 규칙을 피하고 값을 헷갈리지 않게 만들 수 있으며, 타입 별칭과 뉴타입의 차이를 설명할 수 있다. never 타입 `!`이 `continue`·`panic!`·`loop`의 타입이라는 것과, 동적 크기 타입이 항상 포인터 뒤에 있어야 하는 이유, `?Sized`가 하는 일을 설명할 수 있다.
4. 함수 포인터 타입 `fn`과 `Fn` 트레이트들의 관계를 설명하고, 함수 이름과 열거형 생성자를 함수 자리에 넘길 수 있다. 클로저를 `impl Fn`과 `Box<dyn Fn>`으로 반환하고, 둘 중 무엇이 필요한지 판단할 수 있다.
5. 매크로와 함수의 차이를 설명하고, `macro_rules!`로 반복(`$(...),*`)을 쓰는 매크로를 작성할 수 있다. 절차적 매크로 세 종류(derive, 속성형, 함수형)를 구분하고, `syn`과 `quote`로 derive 매크로를 만들어 다른 크레이트에서 쓸 수 있다.

## 2. 핵심 개념

### 2.1 unsafe: 컴파일러 대신 프로그래머가 보증하는 코드

지금까지의 Rust 코드는 컴파일러가 메모리 안전성을 확인해 주었습니다. 그런데 컴파일러의 검사는 **보수적**입니다. 안전한 코드인데도 컴파일러가 증명하지 못하면 거부합니다(2.3절의 `split_at_mut`가 그 예입니다). 또 운영체제나 C 라이브러리를 부르는 일, 하드웨어를 직접 다루는 일은 본질적으로 컴파일러가 확인할 수 없습니다. 이런 일을 하려고 Rust는 `unsafe`라는 문을 열어 둡니다.

`unsafe` 블록(또는 `unsafe fn`) 안에서만 할 수 있는 일은 다섯 가지뿐입니다.

| unsafe 능력 | 이 챕터의 위치 |
| --- | --- |
| 원시 포인터 역참조 | 2.2절, 예제 01 |
| unsafe 함수·메서드 호출 (`extern` 함수 포함) | 2.3~2.4절, 예제 01·02 |
| 가변 정적 변수 읽기·쓰기 | 2.5절, 예제 02 |
| unsafe 트레이트 구현 | 2.6절, 예제 02 |
| 유니언(union) 필드 읽기 | 2.6절 (설명만) |

**`unsafe`는 대여 검사기를 끄지 않습니다.** 참조는 `unsafe` 블록 안에서도 똑같이 검사됩니다. 같은 벡터를 두 번 가변으로 빌리는 코드를 `unsafe`로 감싸도 E0499이고, 컴파일러는 "필요 없는 unsafe 블록"이라는 경고까지 덧붙입니다(핵심 줄만 옮겼습니다).

```text
warning: unnecessary `unsafe` block
 --> bc.rs:3:5
  |
3 |     unsafe {
  |     ^^^^^^ unnecessary `unsafe` block
...
error[E0499]: cannot borrow `v` as mutable more than once at a time
```

`unsafe`는 "이 블록 안에서는 위 다섯 가지 일도 해도 된다. 그 대신 그 일이 안전하다는 것은 내가 보증한다"는 표시입니다. 보증이 틀리면 **미정의 동작(undefined behavior)**, 즉 프로그램이 무엇을 할지 알 수 없는 상태가 됩니다. 그래서 다음 관례를 지킵니다.

- `unsafe` 블록은 **작게** 만듭니다. 메모리 문제가 생기면 찾아볼 곳이 그 블록들로 좁혀집니다.
- 블록 바로 위에 `// SAFETY:` 주석으로 **왜 안전한지**를 적습니다.
- `unsafe fn`의 문서 주석에는 `# Safety` 제목 아래에 **부르는 쪽이 지켜야 할 조건**을 적습니다.
- 가능하면 unsafe 코드를 **안전한 함수로 감싸** 바깥에는 안전한 API만 내놓습니다(2.3절).

### 2.2 원시 포인터

원시 포인터(raw pointer)는 `*const T`(읽기 전용)와 `*mut T`(쓰기도 가능) 두 가지입니다. 여기서 `*`는 역참조 연산자가 아니라 타입 이름의 일부입니다. 참조와 비교하면 다음과 같습니다.

| | 참조 `&T`, `&mut T` | 원시 포인터 `*const T`, `*mut T` |
| --- | --- | --- |
| 대여 규칙 | 검사함 (불변 여러 개 또는 가변 하나) | 검사하지 않음 (같은 곳에 `*const`와 `*mut`를 동시에 가질 수 있음) |
| 가리키는 곳 | 항상 유효한 값 | 해제된 메모리, 아무 주소, null일 수 있음 |
| 정리 | 소유권 규칙을 따름 | 자동으로 아무것도 하지 않음 |
| 만들기 / 역참조 | 둘 다 안전 | 만들기는 안전, 역참조는 `unsafe` |

```rust
let mut num = 5;
let r1 = &raw const num; // *const i32
let r2 = &raw mut num;   // *mut i32 — 참조였다면 r1과 동시에 만들 수 없습니다.

unsafe {
    // SAFETY: r2는 아직 살아 있는 num을 가리키고, 이 블록에서는 r2 하나로만 읽고 씁니다.
    *r2 += 1;
    println!("*r2 = {}", *r2); // *r2 = 6
}
```

`&raw const 위치`와 `&raw mut 위치`는 참조를 거치지 않고 원시 포인터를 바로 만드는 문법입니다. **만드는 것은 안전합니다.** 포인터 값을 만들거나 출력하거나 비교하는 것만으로는 메모리를 건드리지 않기 때문입니다. 위험한 것은 포인터를 따라가 읽고 쓰는 순간이고, 그래서 역참조만 `unsafe`입니다. `unsafe` 없이 `*r1`을 쓰면 E0133입니다(4.1절). `*const` 포인터로 쓰기(`*r1 = 7`)는 `unsafe` 안에서도 E0594(`` cannot assign to `*r1`, which is behind a `*const` pointer ``)입니다.

원시 포인터는 대여 규칙의 검사를 받지 않으므로, 같은 값을 가리키는 여러 포인터를 섞어 써도 되는지는 프로그래머가 따져야 합니다. 그 규칙(에일리어싱 규칙)은 Part 10 `ch3-ownership-lifetimes`에서 다룹니다. 이 챕터의 예제는 한 블록에서 포인터 하나로만 읽고 씁니다.

원시 포인터를 직접 쓸 일은 많지 않습니다. 주로 C 코드와 주고받을 때, 그리고 대여 검사기가 표현하지 못하는 자료 구조를 안전한 API 뒤에 숨겨 구현할 때 씁니다. [`01_unsafe_basics.rs`](examples/01_unsafe_basics.rs)의 1~2번 구역에서 실행해 볼 수 있습니다.

### 2.3 unsafe 함수와 안전한 추상화

**unsafe 함수**는 `unsafe fn`으로 선언한 함수로, "부르는 쪽이 지켜야 할 조건이 있는데 컴파일러가 확인할 수 없다"는 뜻입니다. 부르려면 `unsafe` 블록이 필요합니다.

```rust
/// # Safety
///
/// `ptr`에서 `offset`칸 떨어진 위치가 살아 있는 `i32` 배열 안이어야 합니다.
unsafe fn read_at(ptr: *const i32, offset: usize) -> i32 {
    // SAFETY: 위치가 배열 안이라는 것은 부르는 쪽이 보장합니다(# Safety).
    unsafe { *ptr.add(offset) }
}

// SAFETY: numbers의 길이가 3이므로 1번 위치는 배열 안입니다.
let second = unsafe { read_at(numbers.as_ptr(), 1) };
```

`ptr.add(offset)`은 포인터를 `offset`칸(바이트가 아니라 원소 단위) 뒤로 옮긴 새 포인터를 돌려주는 unsafe 메서드입니다. 표준 라이브러리에도 이런 함수가 있습니다. 슬라이스의 `get_unchecked(i)`는 범위 검사를 건너뛰는 대신, `i`가 범위 안이라는 것을 부르는 쪽이 보장해야 합니다.

**Edition 2024: `unsafe fn` 본문 안에서도 `unsafe` 블록이 필요합니다.** 2021 에디션까지는 `unsafe fn`의 본문 전체가 하나의 `unsafe` 블록처럼 취급되었습니다. 그러면 "부르는 쪽에 조건이 있다"는 표시와 "이 안에서 위험한 연산을 한다"는 표시가 섞여, 본문의 어느 줄이 위험한지 드러나지 않습니다. 2024 에디션에서는 본문 안의 unsafe 연산도 따로 감싸야 하고, 빠뜨리면 경고가 납니다(에러는 아닙니다). 다음은 2.5절의 `add_to_count` 본문에서 `unsafe { }`를 뺐을 때입니다(핵심 줄만 옮겼습니다).

```text
warning[E0133]: use of mutable static is unsafe and requires unsafe block
  --> ed2.rs:12:5
   |
12 |     COUNTER += inc;
   |     ^^^^^^^ use of mutable static
   |
...
note: an unsafe function restricts its caller, but its body is safe by default
...
   = note: `#[warn(unsafe_op_in_unsafe_fn)]` (part of `#[warn(rust_2024_compatibility)]`) on by default
```

`note:`가 이유를 요약합니다. "unsafe 함수는 부르는 쪽을 제약할 뿐, 본문은 기본적으로 안전한 코드다."

#### 안전한 추상화: `split_at_mut` 직접 만들기

슬라이스를 `mid` 위치에서 겹치지 않는 두 가변 슬라이스로 나누는 함수를 안전한 코드로만 쓰면 거부됩니다.

```rust
fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    assert!(mid <= len);
    (&mut values[..mid], &mut values[mid..]) // E0499
}
```

`[..mid]`와 `[mid..]`는 겹치지 않으므로 실제로는 안전합니다. 하지만 대여 검사기는 "같은 `values`를 두 번 가변으로 빌렸다"는 것까지만 알고, 두 범위가 겹치지 않는다는 사실은 알지 못합니다(4.2절). 이럴 때 원시 포인터로 직접 구현합니다.

```rust
use std::slice;

fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    let ptr = values.as_mut_ptr(); // 슬라이스 첫 원소의 *mut i32

    assert!(mid <= len); // 이 검사가 아래 unsafe 블록이 안전한 근거입니다.

    // SAFETY: mid <= len이므로 [0, mid)와 [mid, len)은 모두 values 안에 있고 서로 겹치지 않습니다.
    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}
```

`slice::from_raw_parts_mut(포인터, 길이)`는 포인터와 길이로 슬라이스를 만드는 unsafe 함수입니다. "포인터가 유효하고 그 길이만큼의 메모리가 그 타입의 값들"이라는 조건을 부르는 쪽이 보장해야 합니다. 여기서는 `assert!`가 그 조건을 확인한 뒤에만 `unsafe` 블록에 들어가므로 이 함수는 **어떤 인자로 불러도 미정의 동작이 생기지 않습니다.** 그래서 `split_at_mut` 자체는 `unsafe fn`이 아닌 보통 함수로 선언하고, 부르는 쪽은 `unsafe` 없이 씁니다. 이것이 **안전한 추상화(safe abstraction)** 입니다.

표준 라이브러리의 `split_at_mut`, ch8의 `Vec`, ch14의 `RefCell`, ch15의 `Mutex`가 모두 이렇게 안쪽의 unsafe 코드를 안전한 API로 감싸 둔 것입니다. 우리가 그것들을 `unsafe` 없이 쓸 수 있었던 이유입니다. 안전성 불변식을 설계하는 방법은 Part 10 `ch1-safe-and-unsafe`에서, `split_at_mut`를 대여 분할의 관점에서 다시 보는 것은 Part 10 `ch4-variance-dropck`에서 다룹니다.

### 2.4 extern으로 C 함수 부르기와 내보내기

다른 언어로 작성된 함수를 부르는 것을 **FFI(Foreign Function Interface)** 라고 부릅니다. `extern "C"` 블록에 C 함수의 시그니처를 선언하면 Rust에서 부를 수 있습니다. `"C"`는 함수를 부를 때 인자와 반환값을 어떻게 주고받을지 정한 약속(ABI) 중 C의 것을 쓴다는 뜻입니다.

```rust
use std::ffi::{c_char, c_double, c_int};

unsafe extern "C" {
    fn abs(input: c_int) -> c_int;            // <stdlib.h>
    fn strlen(s: *const c_char) -> usize;     // <string.h>
    safe fn sqrt(x: c_double) -> c_double;    // <math.h>
}
```

- **블록 앞의 `unsafe`** (Edition 2024에서 필수): 이 선언이 실제 C 함수와 맞는지 컴파일러는 확인할 수 없습니다. 이름이나 매개변수 타입을 잘못 적으면 미정의 동작입니다. 그래서 "선언이 맞다"는 보증을 프로그래머가 한다는 표시로 블록에 `unsafe`를 붙입니다. 빠뜨리면 `error: extern blocks must be unsafe`입니다.
- **블록 안의 함수는 기본적으로 unsafe 함수**입니다. C 함수는 Rust의 규칙을 지킨다는 보장이 없기 때문입니다. `strlen`은 끝에 0 바이트가 있는 문자열의 포인터를 받아야 하고, `abs`도 `abs(INT_MIN)`이 C에서 미정의 동작입니다.
- **`safe fn`** 으로 선언하면 `unsafe` 블록 없이 부를 수 있습니다. 어떤 인자에도 동작이 정의된 함수에만 붙입니다. `safe`를 붙인다고 함수가 안전해지는 것이 아니라 "안전하다"는 약속을 내가 하는 것입니다. 이 챕터는 음수에도 NaN을 돌려줄 뿐인 `sqrt`에 붙였습니다(The Book은 `abs`에 붙이는 예를 보여 주지만, 위의 이유로 여기서는 `abs`를 unsafe로 두었습니다).
- `c_int`, `c_char`, `c_double`은 C의 `int`, `char`, `double`에 맞는 Rust 타입의 별칭(2.13절)입니다. 플랫폼마다 C 타입의 크기가 다를 수 있어 이 별칭을 씁니다.
- `c"hello"`는 끝에 0 바이트가 붙은 C 문자열 리터럴이고, `.as_ptr()`로 `*const c_char`를 얻습니다. `strlen(c"안녕".as_ptr())`이 `6`인 것은 UTF-8로 한글 한 글자가 3바이트이기 때문입니다(ch8). C 문자열 타입은 [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md)에서 다룹니다.

표준 C 라이브러리는 Rust 프로그램에 기본으로 링크되므로(이 환경의 링커 명령에 `-lc`, `-lm`이 들어 있습니다) 따로 설정하지 않아도 됩니다.

#### Rust 함수를 C에 내보내기

반대로 C 코드가 부를 수 있는 Rust 함수를 만들 때는 `extern "C"`를 함수 앞에 붙이고 `#[unsafe(no_mangle)]` 속성을 답니다.

```rust
#[unsafe(no_mangle)]
pub extern "C" fn ch19_add(a: c_int, b: c_int) -> c_int {
    a + b
}
```

Rust 컴파일러는 같은 이름의 함수가 여러 모듈·크레이트에 있어도 구분할 수 있도록, 실행 파일 안의 함수 이름을 모듈 경로 등이 섞인 긴 이름으로 바꿉니다. 이것을 **이름 맹글링(name mangling)** 이라고 합니다. `no_mangle`은 이름을 그대로 두라는 뜻이라, C 쪽에서 `ch19_add`라는 이름으로 찾을 수 있습니다. 대신 다른 라이브러리의 같은 이름과 충돌할 수 있으므로 Edition 2024에서는 속성을 `unsafe(...)`로 감쌉니다. 그냥 `#[no_mangle]`로 쓰면 다음 에러입니다.

```text
error: unsafe attribute used without unsafe
 --> ed1.rs:5:3
  |
5 | #[no_mangle]
  |   ^^^^^^^^^ usage of unsafe attribute
  |
help: wrap the attribute in `unsafe(...)`
  |
5 | #[unsafe(no_mangle)]
  |   +++++++         +
```

예제 02는 C 프로그램 대신 같은 파일의 `mod c_side` 안에 `unsafe extern "C" { pub fn ch19_add(...); }`를 선언해, 이름만 보고 함수를 찾아 부르는 C 쪽 상황을 흉내 냅니다. `#[unsafe(no_mangle)]`를 지우면 링커가 이름을 찾지 못해 링크 에러가 납니다(macOS에서 확인한 핵심 줄).

```text
error: linking with `cc` failed: exit status: 1
  = note: Undefined symbols for architecture arm64:
            "_ch19_add", referenced from:
```

C 라이브러리를 직접 빌드해 링크하는 방법, 콜백과 불투명 타입은 Part 10 `ch12-ffi-beneath-std`에서, ABI와 링크 방식은 Part 9 `ch9-memory-panic-linkage`에서 다룹니다.

### 2.5 정적 변수와 가변 정적 변수

Rust의 전역 변수는 **정적 변수(static variable)** 입니다. 이름은 관례상 `SCREAMING_SNAKE_CASE`로 쓰고 타입을 반드시 적습니다.

```rust
static GREETING: &str = "안녕하세요"; // 불변 정적 변수: 읽기는 안전합니다.
static mut COUNTER: u32 = 0;         // 가변 정적 변수: 읽기도 쓰기도 unsafe입니다.
```

ch3의 상수(`const`)와 비슷해 보이지만, 상수는 쓰이는 곳마다 값이 복사되어 들어가고, 정적 변수는 프로그램 전체에서 **메모리 위치가 하나로 고정**되어 있습니다. 그래서 정적 변수는 `mut`일 수 있습니다. 그리고 바로 그 점 때문에 위험합니다. 여러 스레드가 같은 위치를 동시에 고치면 데이터 경쟁(ch15)이 일어나는데, 컴파일러는 어느 스레드가 언제 접근하는지 확인할 수 없습니다. 그래서 `static mut`에 접근하는 것은 unsafe입니다.

```rust
/// # Safety
///
/// 여러 스레드에서 동시에 부르면 데이터 경쟁이 일어납니다. 한 번에 한 스레드에서만 불러야 합니다.
unsafe fn add_to_count(inc: u32) {
    // SAFETY: 한 스레드에서만 부른다는 것은 부르는 쪽이 보장합니다.
    unsafe {
        COUNTER += inc;
    }
}
```

**Edition 2024: `static mut`의 참조를 만들 수 없습니다.** 참조는 "살아 있는 동안 값이 바뀌지 않는다(불변 참조)"거나 "나만 접근한다(가변 참조)"는 약속인데, 어디서든 고칠 수 있는 `static mut`의 참조는 그 약속을 지키기 어렵습니다. 2024 에디션에서는 `static_mut_refs` 린트가 기본으로 에러(deny)입니다(2021 에디션에서는 경고). `println!`은 인자의 참조를 몰래 만들기 때문에 `println!("{}", COUNTER)`도 여기에 걸립니다(핵심 줄만 옮겼습니다).

```text
error: creating a shared reference to mutable static
  --> ed2.rs:19:33
   |
19 |         println!("COUNTER: {}", COUNTER);
   |                                 ^^^^^^^ shared reference to mutable static
   |
   = note: shared references to mutable statics are dangerous; it's undefined behavior if the static is mutated or if a mutable reference is created for it while the shared reference lives
...
   = note: `#[deny(static_mut_refs)]` (part of `#[deny(rust_2024_compatibility)]`) on by default
```

`let r = &COUNTER;`처럼 직접 참조를 만들면 같은 에러에 `` help: use `&raw const` instead to create a raw pointer ``가 덧붙습니다.

참조 대신 **값을 복사해 오거나**(`let copied = unsafe { COUNTER };`), **원시 포인터를 만들어 읽습니다**(`let ptr = &raw const COUNTER;` 다음 `unsafe { *ptr }`). 원시 포인터를 만드는 것 자체는 안전하므로 `&raw const COUNTER`는 `unsafe` 밖에 써도 됩니다. The Book은 `*(&raw const COUNTER)`처럼 한 줄로 쓰는데, 이 모양은 Clippy가 `deref_addrof`(그냥 `COUNTER`와 같다)로 지적하므로 예제 02에서는 포인터를 변수에 담았습니다.

**실제로는 `static mut`를 피합니다.** 같은 일을 안전하게 하는 방법이 있기 때문입니다. ch15의 `Mutex`처럼 내부 가변성이 있는 동기화 타입을 **불변** `static`에 담으면 `unsafe` 없이 여러 곳에서 고칠 수 있습니다.

```rust
static LOG: Mutex<Vec<&str>> = Mutex::new(Vec::new());

LOG.lock().unwrap().push("첫 번째"); // unsafe가 필요 없습니다.
```

원자 타입(`AtomicU32` 등)과 처음 쓸 때 초기화되는 `OnceLock`/`LazyLock`도 이런 용도이며 [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md)에서 다룹니다.

### 2.6 unsafe 트레이트와 유니언

트레이트의 약속 중에 컴파일러가 확인할 수 없는 것이 있으면 `unsafe trait`으로 선언하고, 구현할 때 `unsafe impl`을 씁니다. `unsafe impl`은 "이 타입은 그 약속을 지킨다"는 보증입니다.

```rust
/// 모든 비트가 0인 값이 올바른 값인 타입입니다.
///
/// # Safety
///
/// 구현하는 타입은 메모리를 0으로 채운 값이 그 타입의 올바른 값이어야 합니다.
unsafe trait Zeroable {}

// SAFETY: 정수는 모든 비트가 0이면 0입니다.
unsafe impl Zeroable for u32 {}

fn zeroed<T: Zeroable>() -> T {
    // SAFETY: T가 Zeroable을 구현했다면 0으로 채운 값이 올바른 값입니다.
    unsafe { std::mem::zeroed() }
}
```

`std::mem::zeroed`는 메모리를 0으로 채운 값을 만드는 unsafe 함수입니다. 참조(`&T`)처럼 0(null)일 수 없는 타입에 쓰면 미정의 동작이므로, `Zeroable`을 구현한 타입에만 쓰도록 제한한 것입니다. 이 트레이트에는 메서드가 하나도 없지만, "구현했다"는 사실 자체가 약속이 됩니다. `zeroed`는 그 약속에 기대므로 보통 함수로 둘 수 있습니다.

표준 라이브러리의 대표적인 unsafe 트레이트가 ch15의 `Send`와 `Sync`입니다. 컴파일러는 필드가 모두 `Send`/`Sync`인 타입에 이 트레이트를 자동으로 구현하지만, 원시 포인터를 가진 타입처럼 자동으로 구현되지 않는 타입이 스레드 사이에서 안전하다면 프로그래머가 `unsafe impl Send for 타입 {}`으로 보증합니다(Part 10 `ch9-concurrency`).

다섯 번째 능력인 **유니언(union)** 은 구조체처럼 생겼지만 모든 필드가 같은 메모리를 나눠 쓰는 타입으로, 주로 C의 `union`과 주고받을 때 씁니다. 지금 어느 필드의 값이 들어 있는지 컴파일러가 알 수 없으므로 필드를 읽는 것이 unsafe입니다. Part 9 `ch3-crates-items`에서 다룹니다.

### 2.7 Edition 2024에서 달라진 unsafe 문법과 Miri

앞 절에서 본 Edition 2024의 변화를 모으면 다음과 같습니다. 모두 "위험한 곳을 코드에 더 분명히 드러낸다"는 같은 방향의 변화입니다. 2021 에디션 이전에 쓰인 코드나 자료에서는 왼쪽 모양을 자주 만나므로 둘 다 알아볼 수 있어야 합니다.

| 대상 | 2021 에디션까지 | 2024 에디션 | 2024에서 옛 모양을 쓰면 |
| --- | --- | --- | --- |
| `extern` 블록 | `extern "C" { ... }` | `unsafe extern "C" { ... }`, 안에 `safe fn` 가능 | 에러: `extern blocks must be unsafe` |
| `no_mangle` 속성 | `#[no_mangle]` | `#[unsafe(no_mangle)]` | 에러: `unsafe attribute used without unsafe` |
| `unsafe fn`의 본문 | 본문 전체가 unsafe 블록처럼 취급됨 | 본문 안의 unsafe 연산도 `unsafe { }`로 감쌈 | 경고: `unsafe_op_in_unsafe_fn` (E0133) |
| `static mut`의 참조 | 경고(`static_mut_refs`) | 에러(`static_mut_refs`가 deny) | 에러: `creating a shared reference to mutable static` |

위 표의 2021 열은 같은 파일을 `rustc --edition 2021`로 컴파일해 확인했습니다(앞의 세 가지는 아무 메시지 없이 컴파일되고, 마지막은 경고만 납니다). 에디션 사이의 차이와 자동 이전(`cargo fix --edition`)은 [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md)에서 자세히 다룹니다.

**Miri**: unsafe 코드가 정말 안전한지는 컴파일러가 확인해 주지 않습니다. Miri는 프로그램을 한 줄씩 해석하며 실행해서, 해제된 메모리 읽기나 에일리어싱 규칙 위반 같은 미정의 동작을 **실행 중에** 찾아 주는 공식 도구입니다. nightly 툴체인의 구성 요소라서 따로 설치해야 합니다.

```sh
rustup +nightly component add miri   # 설치 (nightly 툴체인이 필요합니다)
cargo +nightly miri run              # 프로그램을 Miri로 실행
cargo +nightly miri test             # 테스트를 Miri로 실행
```

Miri는 실제로 실행된 경로만 검사하므로, 문제를 보고하면 확실히 버그가 있지만 아무것도 보고하지 않았다고 버그가 없다는 뜻은 아닙니다. 이 챕터를 확인한 환경에는 nightly가 없어 Miri를 실행해 보지 않았습니다. Part 10에서 설치해 직접 씁니다.

### 2.8 연관 타입

**연관 타입(associated type)** 은 트레이트 안에 "타입이 들어갈 자리"를 만들어 두고, 구현하는 쪽이 그 자리를 채우게 하는 기능입니다. 이미 세 번 만났습니다.

| 트레이트 | 연관 타입 | 만난 곳 |
| --- | --- | --- |
| `Iterator` | `type Item` — 반복자가 내놓는 값의 타입 | ch13 |
| `Deref` | `type Target` — `*`로 따라가면 나오는 타입 | ch14 |
| `Future` | `type Output` — 끝나면 내놓는 값의 타입 | [ch16](../ch16-async/README.md) |

```rust
impl Iterator for Counter {
    type Item = u32; // 자리를 채웁니다.

    fn next(&mut self) -> Option<Self::Item> { ... } // Self::Item으로 그 자리를 가리킵니다.
}
```

**제네릭 트레이트와 무엇이 다를까요?** 같은 일을 `trait GenericIterator<T> { fn next_item(&mut self) -> Option<T>; }`처럼 제네릭으로도 정의할 수 있습니다. 차이는 **한 타입에 몇 번 구현할 수 있는가**입니다.

- 제네릭 트레이트는 타입 인자만 바꿔 `impl GenericIterator<u32> for Counter`와 `impl GenericIterator<String> for Counter`를 둘 다 둘 수 있습니다. 대신 쓰는 쪽에서 어느 구현인지 알려 줘야 합니다. 그냥 `counter.next_item()`이라고 쓰면 E0283입니다.

  ```text
  error[E0283]: type annotations needed for `Option<_>`
  note: multiple `impl`s satisfying `Counter: GenericIterator<_>` found
  ```

  그래서 `let x: Option<u32> = counter.next_item();`이나 `GenericIterator::<u32>::next_item(&mut counter)`처럼 타입을 적어야 합니다.
- 연관 타입을 쓰는 트레이트는 한 타입에 **하나만** 구현할 수 있습니다(`impl Iterator for Counter`는 하나뿐). 그래서 `Item`도 하나로 정해지고, 쓰는 쪽은 `counter.next()`만 쓰면 됩니다.

"이 타입의 반복자는 무엇을 내놓는가"처럼 **구현하는 타입이 정하면 하나로 정해지는 것**은 연관 타입으로, "무엇과 더하는가"처럼 **같은 타입이 여러 상대에 대해 구현할 수 있는 것**은 제네릭 매개변수로 둡니다. 다음 절의 `Add`가 두 가지를 함께 씁니다.

연관 타입에 조건을 걸 때는 ch13에서 본 것처럼 `impl Iterator<Item = u32>`, `I: Iterator<Item = u32>`라고 씁니다. 예제는 [`03_associated_types_operators.rs`](examples/03_associated_types_operators.rs)의 1~2번 구역입니다.

### 2.9 기본 제네릭 타입 매개변수와 연산자 오버로딩

Rust는 새 연산자를 만드는 것은 허용하지 않지만, `std::ops`에 있는 트레이트를 구현하면 기존 연산자를 내 타입에 쓸 수 있습니다. 이것을 **연산자 오버로딩(operator overloading)** 이라고 합니다. `a + b`는 `Add` 트레이트의 `a.add(b)`를 짧게 쓴 것입니다. `Add`의 정의는 다음과 같습니다.

```rust
trait Add<Rhs = Self> {
    type Output;

    fn add(self, rhs: Rhs) -> Self::Output;
}
```

- `Rhs`(right-hand side, 오른쪽 피연산자)는 제네릭 타입 매개변수입니다. `= Self`는 **기본 제네릭 타입 매개변수(default generic type parameter)** 로, `impl Add for Point`처럼 `Rhs`를 적지 않으면 `Self`(구현하는 타입 자신)로 정해집니다.
- `Output`(결과 타입)은 연관 타입입니다. `Point + Point`의 결과 타입은 하나로 정해지기 때문입니다.

```rust
impl Add for Point {             // = impl Add<Point> for Point
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Add<Meters> for Millimeters { // Rhs를 직접 정해 서로 다른 타입끼리 더합니다.
    type Output = Millimeters;

    fn add(self, other: Meters) -> Millimeters {
        Millimeters(self.0 + other.0 * 1000)
    }
}
```

`Millimeters(500) + Meters(2)`는 `Millimeters(2500)`입니다. 덧셈의 순서를 바꾼 `Meters(2) + Millimeters(500)`은 구현하지 않았으므로 E0369(`` cannot add `Millimeters` to `Meters` ``)입니다. 연산자 트레이트는 왼쪽 피연산자의 타입에 구현하므로, 순서마다 따로 구현해야 합니다.

**ch8의 `String + &str`이 이것이었습니다.** 표준 라이브러리에 `impl Add<&str> for String { type Output = String; ... }`이 있습니다. `add`가 `self`를 받으므로 왼쪽 `String`이 이동하고, `Rhs`가 `&str`이므로 오른쪽은 빌려서 넘깁니다. 예제 03의 5번 구역은 `+` 대신 `Add::add(hello, "하세요")`로 불러 같은 결과를 확인합니다.

기본 제네릭 타입 매개변수는 두 가지 경우에 씁니다. 하나는 대부분의 사용자가 신경 쓰지 않아도 되도록 흔한 경우를 기본값으로 정하는 것(`Add`의 `Rhs = Self`), 다른 하나는 이미 쓰이는 트레이트에 타입 매개변수를 추가하면서 기존 코드를 깨지 않는 것입니다.

| 연산자 | 트레이트 |
| --- | --- |
| `+` `-` `*` `/` `%` | `Add` `Sub` `Mul` `Div` `Rem` |
| 단항 `-`, `!` | `Neg`, `Not` |
| `+=` 등 | `AddAssign` 등 |
| `a[i]` | `Index`, `IndexMut` |
| `==`, `<` 등 | `PartialEq`, `PartialOrd` (`std::cmp`, ch10에서 derive) |

`std::ops`의 트레이트 전체와 비교·변환 트레이트는 [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)에서 다룹니다.

### 2.10 완전 정규화 문법: 같은 이름의 메서드 구분하기

서로 다른 트레이트가 같은 이름의 메서드를 가질 수 있고, 한 타입이 그 트레이트들을 모두 구현할 수도 있습니다. 타입 자신에게 같은 이름의 메서드가 있을 수도 있습니다.

```rust
impl Pilot for Human  { fn fly(&self) -> String { /* 기장입니다... */ } }
impl Wizard for Human { fn fly(&self) -> String { /* 날아올라라! */ } }
impl Human            { fn fly(&self) -> String { /* *팔을 힘차게 휘젓는다* */ } }
```

- `person.fly()`는 **타입 자신의 메서드**를 부릅니다.
- 트레이트의 것을 부르려면 ch10에서 본 `트레이트::메서드(&값)` 형태로 `Pilot::fly(&person)`, `Wizard::fly(&person)`라고 씁니다. 메서드는 `&person`의 타입을 보고 어느 타입의 구현인지 정할 수 있습니다.
- 타입 자신의 메서드가 없고 트레이트 두 개만 같은 이름을 가지면, `person.fly()`는 모호해서 E0034(`multiple applicable items in scope`)이고 컴파일러가 후보마다 `트레이트::메서드(&값)` 모양으로 고치라고 제안합니다.

**`self`가 없는 연관 함수는 다릅니다.** `Dog`에 타입 자신의 `baby_name()`과 `Animal` 트레이트의 `baby_name()`이 있을 때, `Animal::baby_name()`이라고 쓰면 인자가 없어 어느 타입의 구현을 부를지 알 수 없습니다(E0790, 4.3절). 이때 **완전 정규화 문법(fully qualified syntax)** 으로 타입과 트레이트를 모두 적습니다.

```rust
println!("{}", Dog::baby_name());              // 바둑이 (타입 자신의 연관 함수)
println!("{}", <Dog as Animal>::baby_name());  // 강아지 (Animal 트레이트의 구현)
```

일반형은 `<타입 as 트레이트>::함수(받는값, 인자...)`입니다. 메서드에도 쓸 수 있지만(`<Human as Wizard>::fly(&person)`), 컴파일러가 알아낼 수 있는 부분은 생략할 수 있으므로 모호할 때만 씁니다. 예제는 [`04_qualified_syntax_supertraits.rs`](examples/04_qualified_syntax_supertraits.rs)의 1~2번 구역입니다.

### 2.11 슈퍼트레이트

트레이트를 정의할 때 다른 트레이트의 기능이 필요하면, 그 트레이트를 구현한 타입만 내 트레이트를 구현할 수 있게 요구합니다. 요구되는 쪽을 **슈퍼트레이트(supertrait)** 라고 부릅니다.

```rust
trait OutlinePrint: fmt::Display {      // OutlinePrint를 구현하려면 Display도 구현해야 합니다.
    fn outline_print(&self) {
        let output = self.to_string();  // Display가 보장되므로 to_string()을 쓸 수 있습니다.
        // ... 별표 상자로 감싸 출력 ...
    }
}
```

`trait OutlinePrint: fmt::Display`는 트레이트 바운드(ch10)를 트레이트 자신에게 단 것입니다. 덕분에 기본 구현 안에서 `Display`가 주는 `to_string()`을 쓸 수 있고, `T: OutlinePrint` 바운드만 적은 함수에서도 `{}` 출력을 쓸 수 있습니다. `Display`를 구현하지 않은 타입에 `impl OutlinePrint for Point {}`를 쓰면 E0277입니다(4.5절).

다른 언어의 상속과 비슷해 보이지만, 슈퍼트레이트는 "이것도 구현해야 한다"는 **조건**일 뿐 구현을 물려주지 않습니다. `Display` 구현은 `Point`가 직접 작성해야 합니다. 예제 04의 3번 구역에서 별표 상자를 출력해 봅니다.

### 2.12 뉴타입 패턴

**뉴타입 패턴(newtype pattern)** 은 기존 타입 하나를 필드로 가진 튜플 구조체(ch5)를 만들어 새 타입으로 쓰는 것입니다. 이름은 Haskell에서 왔습니다. 감싸기만 하므로 실행 중 비용이 없습니다. 예제 05에서 `size_of::<UserId>()`와 `size_of::<u32>()`가 모두 `4`입니다. 세 가지 용도가 있습니다.

**1) 고아 규칙 피하기.** ch10에서 본 고아 규칙 때문에 `Display`(표준)를 `Vec<String>`(표준)에 직접 구현할 수 없습니다(E0117, 4.4절). `Vec<String>`을 감싼 `Wrapper`는 내 크레이트의 타입이므로 구현할 수 있습니다.

```rust
struct Wrapper(Vec<String>);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}
```

단점은 `Wrapper`가 `Vec`의 메서드를 물려받지 않는다는 것입니다. 필요한 메서드만 `self.0`에 넘기는 메서드를 직접 만들거나, ch14의 `Deref`를 구현해 `w.len()`, `w.first()`처럼 안쪽 타입의 메서드를 쓰게 할 수 있습니다(예제 05). 반대로 안쪽 타입의 메서드 일부를 숨기고 싶다면 `Deref`를 구현하지 않습니다.

**2) 값을 헷갈리지 않게 하기.** `UserId(u32)`와 `OrderId(u32)`는 속이 같아도 다른 타입입니다. `fn cancel_order(id: OrderId)`에 `UserId`를 넘기면 E0308(`` expected `OrderId`, found `UserId` ``)입니다. 단위(`Millimeters`와 `Meters`)를 구분할 때도 씁니다.

**3) 내부 구현 감추기.** ch9의 검증 타입 `Guess`처럼, 필드를 비공개로 두고 정해진 생성자와 메서드만 공개하면 안쪽 타입이 무엇인지 감추고 불변식을 지킬 수 있습니다.

### 2.13 타입 별칭

`type 새이름 = 기존타입;`은 **타입 별칭(type alias)** 을 만듭니다. 뉴타입과 달리 새 타입이 아니라 **같은 타입의 다른 이름**입니다.

```rust
type Kilometers = i32;

let distance: Kilometers = 5;
let extra: i32 = 3;
println!("{}", distance + extra); // 8 — Kilometers와 i32는 같은 타입이라 그대로 섞입니다.
```

그래서 별칭은 뉴타입처럼 실수를 막아 주지 못합니다. 주된 쓸모는 **긴 타입 이름 줄이기**입니다.

```rust
type Thunk = Box<dyn Fn() -> String + Send + 'static>; // 나중에 실행할 클로저

fn make_thunks() -> Vec<Thunk> { ... }
```

ch9의 `io::Result<T>`가 바로 별칭입니다. `std::io` 모듈에 `type Result<T> = std::result::Result<T, std::io::Error>;`가 있어서, 에러 타입이 늘 같은 `io` 함수들의 시그니처를 짧게 씁니다. 별칭이므로 `?`, `map_err` 등 `Result`의 기능을 그대로 쓸 수 있습니다. 예제 05의 4번 구역은 같은 방식으로 `type Result<T> = std::result::Result<T, ParseError>;`를 만들어 씁니다. 이렇게 자기 모듈의 `Result` 별칭을 만들면 그 모듈 안에서는 `Result`가 프렐류드의 `Result<T, E>` 대신 별칭을 가리킵니다(ch9에서 본 이름 가림과 같은 현상).

### 2.14 never 타입 `!`

ch6의 `let...else`에서 `` expected `!`, found `()` `` 에러를 보았습니다. `!`는 **never 타입**으로, 값이 하나도 없는 타입입니다. "이 표현식은 값을 만들지 않고 흐름을 끊는다"는 뜻입니다. 다음 표현식들의 타입이 `!`입니다.

| 표현식 | 흐름이 어떻게 끊기는가 |
| --- | --- |
| `continue`, `break`, `return` | 다음 반복이나 반복문 밖, 함수 밖으로 갑니다. |
| `panic!(...)` | 프로그램(스레드)이 멈춥니다. |
| `break` 없는 `loop` | 끝나지 않습니다. |
| `std::process::exit(code)` | 프로세스가 끝납니다. |

`!` 타입의 값은 만들어질 수 없으므로, 컴파일러는 `!`를 **어떤 타입으로든 바꿀 수 있습니다.** 그래서 `match`의 갈래나 `if`의 두 갈래가 같은 타입이어야 한다는 규칙과 부딪히지 않습니다.

```rust
let number: u32 = match text.parse() {
    Ok(n) => n,          // u32
    Err(_) => continue,  // ! → u32로 취급됩니다.
};
```

[ch2](../ch2-guessing-game/README.md) 숫자 맞히기 게임에서 `Err(_)` 갈래가 `continue`로 끝나던 `match`, [ch6](../ch6-enums-pattern-matching/README.md) `let...else`의 `else { return; }`이 모두 이 덕분에 컴파일됩니다. `{ println!(...); continue; }`처럼 흐름을 끊는 문장으로 끝나는 블록도 흐름을 끊는 것으로 취급되어 어떤 타입 자리에든 놓일 수 있습니다.

반환 타입이 `!`인 함수(`fn fail(message: &str) -> ! { panic!(...) }`)는 절대 돌아오지 않는다는 뜻이며, 이런 함수를 **발산 함수(diverging function)** 라고 부릅니다. 예제 06의 `my_unwrap`은 `Option::unwrap`을 단순하게 흉내 낸 함수로, `Some(v) => v` 갈래는 `T`, `None => fail(...)` 갈래는 `!`이므로 `match` 전체의 타입이 `T`가 됩니다.

`!`는 함수의 반환 타입 말고 다른 자리에도 타입으로 적을 수 있습니다. `let x: ! = panic!();`이나 `Vec<!>`(원소를 하나도 가질 수 없는 벡터)가 그 예입니다. 다만 이것은 never 타입이 안정화된 rustc 1.100.0부터입니다. 1.98.1까지는 반환 타입이 아닌 자리에 `!`를 쓰면 E0658(`` the `!` type is experimental ``)이었으므로, 오래된 툴체인이나 예전 글에서는 "반환 타입에만 쓸 수 있다"고 되어 있습니다. 타입을 정하지 못한 자리가 `!`로 정해지는 규칙(never 타입 폴백)은 [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.10절에서 다룹니다.

### 2.15 동적 크기 타입과 `Sized`

`str`(`&str`이 아니라 `str` 자체)은 길이가 값마다 다릅니다. `"hi"`는 2바이트, `"안녕하세요"`는 15바이트입니다. 이렇게 **크기를 실행 중에야 알 수 있는 타입**을 **동적 크기 타입(dynamically sized type, DST)** 이라고 부릅니다. `str`, 슬라이스 `[T]`, 트레이트 객체 `dyn Trait`([ch17](../ch17-trait-objects-oop/README.md))이 대표적입니다.

지역 변수와 함수 인자는 스택에 자리를 미리 잡아야 하므로 크기를 컴파일 시점에 알아야 합니다. 그래서 `let s: str = ...`은 E0277입니다(4.6절). **동적 크기 타입은 항상 포인터 뒤에 둡니다.** `&str`, `&[T]`, `Box<str>`, `Rc<str>`, `&dyn Trait`, `Box<dyn Trait>`가 그런 모양입니다. 이런 포인터는 주소와 함께 **추가 정보**(슬라이스·문자열은 길이, 트레이트 객체는 메서드 표의 주소)를 들고 있어서 크기가 보통 참조의 두 배입니다. 64비트 환경에서 확인한 값입니다.

| 타입 | 크기(바이트) | 들고 있는 것 |
| --- | --- | --- |
| `&i32` | 8 | 주소 |
| `&str`, `&[i32]`, `Box<str>` | 16 | 주소 + 길이 |
| `&dyn Display` | 16 | 주소 + 메서드 표 주소 |

**`Sized` 트레이트**는 "크기를 컴파일 시점에 아는 타입"이라는 표시입니다. 컴파일러가 자동으로 구현하며, 제네릭 함수의 타입 매개변수에는 **보이지 않는 `T: Sized` 바운드가 자동으로 붙습니다.** 그래서 `fn byte_size<T>(value: &T)`에 `"hello"`(타입 `&str`, 즉 `T = str`)를 넘기면 E0277이고, 컴파일러가 바운드를 풀라고 제안합니다.

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
note: required by an implicit `Sized` bound in `byte_size`
help: consider relaxing the implicit `Sized` restriction
  |
1 | fn byte_size<T: ?Sized>(value: &T) -> usize { std::mem::size_of_val(value) }
  |               ++++++++
```

`T: ?Sized`는 "`T`는 `Sized`일 수도 있고 아닐 수도 있다"는 뜻입니다. `T`의 크기를 모를 수 있으므로 `T`를 값으로 받지 못하고 `&T`처럼 포인터 뒤에 둡니다. 이렇게 하면 `&i32`, `&[i32]`, `&str`, `&dyn Display`를 모두 받습니다(예제 06의 3번 구역). `?`를 붙여 기본 바운드를 푸는 문법은 `Sized`에만 있습니다. `?Clone`처럼 쓰면 `` bound modifier `?` can only be applied to `Sized` `` 에러입니다.

타입 레이아웃과 동적 크기 타입의 자세한 규칙은 Part 9 `ch7-type-system`과 `ch8-special-types-names`, Part 10 `ch2-data-layout`에서 다룹니다.

### 2.16 함수 포인터

ch13에서 클로저 자리에 함수 이름을 넘길 수 있다고 했습니다. 함수는 **함수 포인터(function pointer)** 타입 `fn`(소문자)으로 바뀌어 전달됩니다.

```rust
fn add_one(x: i32) -> i32 { x + 1 }

fn do_twice(f: fn(i32) -> i32, arg: i32) -> i32 {
    f(arg) + f(arg)
}

do_twice(add_one, 5) // 12
```

`fn`과 `Fn`의 관계는 다음과 같습니다.

- `fn(i32) -> i32`는 **타입**입니다. 매개변수 타입 자리에 그대로 쓰고, 배열에 담을 수도 있습니다. `Fn`, `FnMut`, `FnOnce`는 **트레이트**라서 `impl Fn(i32) -> i32`나 제네릭 바운드로 씁니다.
- 함수 포인터는 세 `Fn` 트레이트를 모두 구현합니다. 그래서 `Fn`을 요구하는 자리에는 함수도 클로저도 넘길 수 있습니다.
- **아무것도 캡처하지 않는 클로저**는 함수 포인터로 바뀔 수 있습니다. 예제 07은 `|x| -x`를 함수들과 함께 `fn(i32) -> i32` 배열에 담습니다.
- **캡처하는 클로저**는 함수 포인터가 될 수 없습니다. 캡처한 값을 담을 자리가 함수 포인터에는 없기 때문입니다.

  ```text
  error[E0308]: mismatched types
     = note: expected fn pointer `fn(i32) -> i32`
                   found closure `{closure@t07.rs:12:29: 12:32}`
  note: closures can only be coerced to `fn` types if they do not capture any variables
  ```

그래서 함수를 받는 함수를 만들 때는 보통 `impl Fn(...)`이나 제네릭 바운드로 받아 함수와 클로저를 모두 받게 합니다. `fn` 타입만 받는 경우는 클로저가 없는 C 코드와 함수(콜백)를 주고받을 때처럼 특별한 경우입니다.

함수 이름을 넘기는 것은 반복자 어댑터에서 자주 봅니다.

```rust
numbers.iter().map(|n| n.to_string())   // 클로저
numbers.iter().map(ToString::to_string) // 함수 이름 — 결과가 같습니다.
```

`to_string`이라는 이름의 함수는 여러 곳에 있을 수 있으므로 `ToString::to_string`처럼 트레이트 이름을 붙여 가리킵니다(2.10절). **열거형 배리언트와 튜플 구조체의 이름도 함수**입니다. `Status::Value`는 `u32`를 받아 `Status`를 돌려주는 함수처럼 쓸 수 있어서 `(1..=3).map(Status::Value)`라고 쓸 수 있고, `Some`, `Meters`(튜플 구조체)도 마찬가지입니다. 클로저로 쓰든 함수 이름으로 쓰든 같은 코드로 컴파일되므로 읽기 쉬운 쪽을 고르면 됩니다.

### 2.17 클로저 반환하기

클로저 타입에는 이름이 없으므로, 클로저를 돌려주는 함수는 ch13에서 본 것처럼 반환 위치의 `impl Trait`를 씁니다.

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
```

그런데 `impl Fn`은 "구체 타입 **하나**를 숨긴 것"입니다(ch10). 모양이 같아도 클로저마다 타입이 다르므로, 갈래마다 다른 클로저를 돌려주면 E0308입니다.

```text
error[E0308]: `match` arms have incompatible types
  = note: no two closures, even if identical, have the same type
  = help: consider boxing your closure and/or using it as a trait object
help: you could change the return type to be a boxed trait object
  |
4 + fn make_op(kind: &str, n: i32) -> Box<dyn Fn(i32) -> i32> {
```

`impl Fn`을 반환하는 함수 두 개의 결과를 한 `Vec`에 담으려 할 때도 같은 이유로 `` expected opaque type, found a different opaque type ``이 납니다. 이럴 때는 `help:`대로 **트레이트 객체** `Box<dyn Fn(i32) -> i32>`([ch17](../ch17-trait-objects-oop/README.md))로 감싸 모든 클로저를 같은 타입으로 맞춥니다.

| 반환 타입 | 쓸 수 있는 경우 | 비용 |
| --- | --- | --- |
| `impl Fn(..) -> ..` | 언제나 같은 클로저 하나를 돌려줄 때 | 없음 (힙 할당·간접 호출 없음) |
| `Box<dyn Fn(..) -> ..>` | 경우마다 다른 클로저를 돌려주거나 여러 클로저를 한 컬렉션에 담을 때 | 힙 할당 + 실행 중에 호출할 함수를 찾음 |
| `fn(..) -> ..` | 아무것도 캡처하지 않는 클로저나 함수만 돌려줄 때 | 없음 (포인터 하나) |

예제 07의 `pick(negate: bool) -> fn(i32) -> i32`는 캡처 없는 클로저 `|x| -x`와 함수 `add_one` 중 하나를 돌려주므로 `fn` 타입으로 충분합니다.

### 2.18 매크로와 함수의 차이

매크로는 **코드를 만들어 내는 코드**입니다(메타프로그래밍). `println!`, `vec!`, `format!`, `assert_eq!`, 그리고 `#[derive(Debug)]`가 모두 매크로였습니다. 함수와는 다음이 다릅니다.

- **인자 개수가 자유롭습니다.** `println!("안녕")`과 `println!("{} {}", a, b)`처럼 같은 매크로를 인자 개수를 바꿔 부를 수 있습니다. Rust 함수는 매개변수 개수가 고정입니다.
- **컴파일러가 코드를 해석하기 전에 펼쳐집니다(expand).** 그래서 매크로는 구조체 정의나 `impl` 블록 같은 **항목**을 만들 수 있습니다. 함수는 실행 중에 불리므로 트레이트 구현을 만들 수 없습니다. `#[derive(Debug)]`가 `impl Debug for ...`를 만들어 낸 것이 이 능력입니다.
- **값이 아니라 코드를 받습니다.** 예제 08의 `show!(1 + 2 * 3)`은 `1 + 2 * 3 = 7`을 출력합니다. 함수였다면 `7`이라는 값만 받았을 것입니다.
- **정의가 읽고 고치기 어렵습니다.** 코드를 만드는 코드이기 때문입니다.
- **파일 안에서 쓰기 전에 정의되어 있어야 합니다.** 함수는 파일 어디에 정의해도 부를 수 있지만, `macro_rules!` 매크로는 정의보다 위에서 부르면 `` cannot find macro `my_macro` in this scope ``와 함께 `a macro with the same name exists, but it appears later`라는 안내가 나옵니다.

Rust의 매크로는 두 종류입니다.

| 종류 | 만드는 방법 | 예 |
| --- | --- | --- |
| 선언적 매크로 (declarative macro) | `macro_rules!`로 "이런 모양의 코드면 이렇게 바꾼다"는 규칙을 적습니다. | `vec!`, `println!` |
| 절차적 매크로 (procedural macro) | 코드(토큰)를 받아 코드를 돌려주는 **함수**를 별도 크레이트에 작성합니다. derive, 속성형, 함수형 세 가지가 있습니다. | `#[derive(Debug)]`, 외부 크레이트의 `#[derive(Serialize)]` |

매크로가 실제로 어떤 코드로 펼쳐지는지 보고 싶을 때는 서드파티 도구 `cargo expand`(`cargo install cargo-expand`로 설치)를 씁니다. 이 챕터에서는 설치하지 않고, 펼쳐진 모양을 손으로 적어 설명합니다.

### 2.19 `macro_rules!`: 선언적 매크로

`macro_rules!`는 `match`와 비슷합니다. 갈래마다 **패턴**과 **바꿀 코드**가 있고, 매크로에 넘긴 코드(값이 아니라 코드의 모양)를 위의 갈래부터 차례로 맞춰 봅니다. `vec!`을 흉내 낸 `my_vec!`입니다.

```rust
macro_rules! my_vec {
    () => {
        Vec::new()
    };
    ( $( $x:expr ),* $(,)? ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}
```

- **`$x:expr`**: 표현식 하나와 맞춰 보고, 맞으면 그 코드를 `$x`라는 이름으로 붙잡습니다. `expr` 자리에 올 수 있는 것을 **프래그먼트 지정자**라고 부르며, 이 챕터에서는 `expr`(표현식), `ident`(이름), `literal`(리터럴)을 씁니다. `ty`(타입), `tt`(토큰 하나) 등 나머지는 Part 9 `ch2-macros`에서 다룹니다.
- **`$( ... ),*`**: 괄호 안의 패턴이 쉼표로 구분되어 0번 이상 반복됩니다. `*` 대신 `+`는 1번 이상, `?`는 0번 또는 1번입니다. `$(,)?`는 끝에 쉼표가 있어도 되게 합니다.
- **바꿀 코드 안의 `$( ... )*`**: 패턴에서 붙잡은 `$x`마다 괄호 안의 코드를 한 번씩 만듭니다.

그래서 `my_vec![1, 2, 3]`은 다음 코드로 펼쳐집니다.

```rust
{
    let mut temp_vec = Vec::new();
    temp_vec.push(1);
    temp_vec.push(2);
    temp_vec.push(3);
    temp_vec
}
```

**펼쳐진 코드도 보통 코드와 똑같이 검사됩니다.** The Book의 `vec!` 예제에는 첫 번째 갈래(`()`)가 없습니다. 그대로 `my_vec![]`을 부르면 `push`가 하나도 없는 `let mut temp_vec`이 만들어져 `unused_mut` 경고가 나고, 경고의 위치는 매크로 정의 안을, 원인은 그 매크로를 부른 곳을 가리킵니다(`this warning originates in the macro my_vec`). 그래서 빈 호출만 받는 갈래를 위에 두었습니다. Clippy의 검사는 버전에 따라 다릅니다. Clippy 1.98.1은 펼쳐진 `Vec::new()` + `push` 모양을 보고 `vec![]`을 쓰라고 권했지만(`vec_init_then_push`), 1.100.0은 이 매크로가 펼친 코드에는 그 경고를 내지 않습니다. 그래서 예제에는 따로 표시를 달지 않았습니다. 매크로가 펼친 코드에서 나는 경고를 예상된 것으로 표시해야 할 때는, 매크로를 부른 곳에 `#[expect(lint 이름, reason = "...")]`를 답니다.

[`08_macro_rules.rs`](examples/08_macro_rules.rs)에는 이 밖에 세 가지 매크로가 더 있습니다.

- `max!`: 갈래가 둘이고, 두 번째 갈래가 자기 자신을 다시 부릅니다(재귀). 그래서 `max!(7)`, `max!(3, 9, 2)`처럼 인자 개수에 상관없이 동작합니다.
- `show!`: `stringify!($e)`로 받은 코드를 그대로 문자열로 만들어 값과 함께 출력합니다. `stringify!`는 표현식을 계산하지 않고 컴파일 시점에 그 모양을 문자열 리터럴로 바꾸는 내장 매크로입니다.
- `unit_type!(Meters, "m")`: `$name:ident`로 받은 이름으로 튜플 구조체와 `Display` 구현을 만듭니다. 함수로는 할 수 없는 일입니다. 2.12절의 뉴타입을 여러 개 만들 때 반복을 줄이는 방법이기도 합니다.

**다른 모듈·크레이트에서 쓰기**: `macro_rules!` 매크로는 정의된 위치부터 그 모듈 끝까지 보입니다. 모듈 안에서 정의한 매크로를 모듈 밖(뒤쪽)에서 쓰려면 `mod` 앞에 `#[macro_use]`를 붙이고, 다른 크레이트에서 쓰게 하려면 정의에 `#[macro_export]`를 붙여 크레이트 루트에 내보냅니다. 매크로 위생, 프래그먼트 지정자 전체, 디버깅 방법은 Part 9 `ch2-macros`에서 다룹니다.

### 2.20 절차적 매크로와 `projects/hello_macro`

**절차적 매크로(procedural macro)** 는 패턴 맞추기 대신 Rust 함수로 코드를 만듭니다. 컴파일러가 매크로를 붙인 코드를 **토큰 스트림(`TokenStream`)** 으로 넘기면, 함수가 그것을 읽고 새 토큰 스트림을 돌려줍니다. 컴파일 도중에 컴파일러가 불러 쓰는 코드라서 **`proc-macro = true`로 설정한 전용 크레이트**에 있어야 합니다. 세 종류가 있습니다.

| 종류 | 쓰는 모양 | 정의하는 함수의 모양 | 붙일 수 있는 곳 |
| --- | --- | --- | --- |
| derive 매크로 | `#[derive(HelloMacro)]` | `#[proc_macro_derive(HelloMacro)] pub fn f(input: TokenStream) -> TokenStream` | 구조체, 열거형, 유니언. 원래 항목 **뒤에 코드를 덧붙입니다**. |
| 속성형 매크로 | `#[route(GET, "/")] fn index() {}` | `#[proc_macro_attribute] pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream` | 함수 등 어떤 항목에나. `attr`는 `GET, "/"`, `item`은 붙인 항목 전체이며 **항목을 바꿔 놓을 수 있습니다**. |
| 함수형 매크로 | `sql!(SELECT * FROM posts)` | `#[proc_macro] pub fn sql(input: TokenStream) -> TokenStream` | 함수 호출처럼 쓰는 곳. `macro_rules!`로 표현하기 어려운 복잡한 해석이 필요할 때 씁니다. |

속성형의 `route`와 함수형의 `sql!`은 The Book이 웹 프레임워크와 SQL 검사를 예로 든 가상의 매크로입니다. 이 챕터에서는 derive 매크로를 직접 만듭니다.

#### `hello_macro`: derive 매크로 만들기

목표는 `#[derive(HelloMacro)]`만 붙이면 `HelloMacro` 트레이트가 구현되어 `Pancakes::hello_macro()`가 자기 타입 이름으로 인사하는 것입니다. 이름은 매크로가 알아내야 하므로 트레이트의 기본 구현으로는 만들 수 없고, 매크로가 필요합니다. 크레이트 세 개를 한 워크스페이스로 묶었습니다.

```text
projects/hello_macro/
├── Cargo.toml                # [workspace] members = 세 크레이트
├── hello_macro/              # 라이브러리 크레이트: trait HelloMacro { fn hello_macro(); }
├── hello_macro_derive/       # 절차적 매크로 크레이트: #[derive(HelloMacro)]
│   └── Cargo.toml            # [lib] proc-macro = true, syn·quote 의존
└── pancakes/                 # 바이너리 크레이트: 둘을 가져와 씁니다.
```

트레이트 크레이트와 매크로 크레이트를 나누는 이유는, 절차적 매크로 크레이트는 매크로 말고 다른 항목(트레이트 등)을 내보낼 수 없기 때문입니다. 이름은 관례에 따라 `트레이트크레이트_derive`로 짓습니다. The Book은 `hello_macro_derive`를 `hello_macro` 디렉터리 안에 두고 `pancakes`를 별도 프로젝트로 만들지만, 여기서는 세 크레이트를 나란히 둔 워크스페이스로 묶어 `cargo run` 한 번으로 결과를 볼 수 있게 했습니다.

매크로 크레이트의 핵심은 다음과 같습니다([`hello_macro_derive/src/lib.rs`](projects/hello_macro/hello_macro_derive/src/lib.rs)).

```rust
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();   // 토큰 → 구문 트리(syn::DeriveInput)
    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;                  // derive를 붙인 타입의 이름
    let generated = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("안녕하세요, 매크로! 제 이름은 {}입니다!", stringify!(#name));
            }
        }
    };
    generated.into()                        // proc_macro2::TokenStream → proc_macro::TokenStream
}
```

- `proc_macro`는 컴파일러와 함께 오는 크레이트로, `TokenStream` 타입을 제공합니다. `Cargo.toml`에 적지 않습니다.
- **`syn`** 은 토큰을 구문 트리로 해석합니다. `syn::DeriveInput`은 derive를 붙인 항목을 나타내며, `ident` 필드에 타입 이름이, `generics` 필드에 제네릭 매개변수가, `data` 필드에 필드나 배리언트 정보가 들어 있습니다.
- **`quote`** 의 `quote! { ... }`는 그 안의 Rust 코드를 토큰으로 바꿉니다. `#name` 자리에는 변수 `name`의 값(`Pancakes`)이 들어갑니다.
- `unwrap()`으로 해석 실패를 패닉으로 처리합니다. 절차적 매크로는 `Result`가 아니라 `TokenStream`을 돌려줘야 하므로, 실패를 알리는 방법이 패닉(컴파일 에러로 보고됨)이기 때문입니다. 실제 라이브러리는 `syn::parse_macro_input!` 등으로 더 친절한 에러를 만듭니다.

`pancakes`에서 `#[derive(HelloMacro)] struct Pancakes;`를 쓰면, 매크로가 돌려준 다음 코드가 `struct Pancakes;` 바로 뒤에 덧붙여진 것처럼 컴파일됩니다.

```rust
impl HelloMacro for Pancakes {
    fn hello_macro() {
        println!("안녕하세요, 매크로! 제 이름은 {}입니다!", stringify!(Pancakes));
    }
}
```

덧붙여진 코드는 **`pancakes`의 `main.rs` 안에서 해석됩니다.** 그래서 이 코드가 쓰는 `HelloMacro`라는 이름이 `main.rs`에서 트레이트를 가리켜야 하고, `main.rs`는 트레이트와 derive 매크로를 둘 다 가져옵니다. 두 `use`가 같은 이름 `HelloMacro`를 가져와도 충돌하지 않는 것은 매크로와 트레이트(타입)가 서로 다른 이름 공간에 있기 때문입니다. 트레이트를 가져오지 않았을 때의 에러는 4.7절에서 봅니다.

`syn`의 버전은 The Book이 쓰는 2.0이 아니라 이 챕터를 작성할 때 crates.io의 최신 버전인 3.0.10을 썼습니다. 위 코드에서 쓰는 API(`syn::parse`, `DeriveInput`의 `ident`)는 두 버전에서 같습니다.

## 3. 예제 안내

`examples/`의 명령은 저장소 루트에서 실행합니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_unsafe_basics.rs`](examples/01_unsafe_basics.rs) | 원시 포인터 만들기·역참조, unsafe 함수 정의·호출, `split_at_mut` 직접 구현 | 1 |
| [`02_ffi_static_unsafe_trait.rs`](examples/02_ffi_static_unsafe_trait.rs) | `unsafe extern "C"`와 `safe fn`, `#[unsafe(no_mangle)]`, `static`/`static mut`, unsafe 트레이트 | 1 |
| [`03_associated_types_operators.rs`](examples/03_associated_types_operators.rs) | 연관 타입과 제네릭 트레이트 비교, `Add` 구현(`Point + Point`, `Millimeters + Meters`), `String + &str` | 2 |
| [`04_qualified_syntax_supertraits.rs`](examples/04_qualified_syntax_supertraits.rs) | 같은 이름의 메서드·연관 함수와 완전 정규화 문법, 슈퍼트레이트 `OutlinePrint` | 2 |
| [`05_newtype_and_alias.rs`](examples/05_newtype_and_alias.rs) | 뉴타입으로 고아 규칙 피하기·값 구분하기, 타입 별칭(`Kilometers`, `Thunk`, `Result<T>`) | 2, 3 |
| [`06_never_and_sized.rs`](examples/06_never_and_sized.rs) | never 타입 `!`, 동적 크기 타입과 넓은 포인터의 크기, `?Sized` | 3 |
| [`07_fn_pointers_closures.rs`](examples/07_fn_pointers_closures.rs) | 함수 포인터 `fn`과 `Fn`, 함수 이름·생성자를 함수로 넘기기, `impl Fn`/`Box<dyn Fn>` 반환 | 4 |
| [`08_macro_rules.rs`](examples/08_macro_rules.rs) | `macro_rules!`의 반복·여러 갈래·재귀, `stringify!`, 항목을 만드는 매크로 | 5 |
| [`projects/hello_macro`](projects/hello_macro/) | derive 매크로 직접 만들기(트레이트·매크로·바이너리 세 크레이트) | 5 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 6개 (4절) | 1, 2, 3 |

### 01_unsafe_basics — 원시 포인터, unsafe 함수, 안전한 추상화

1번 구역의 주소는 **실행할 때마다 달라집니다.** 아래는 한 번 실행한 결과이고, 두 포인터가 같은 주소라는 것만 확인하면 됩니다. 4번 구역에서 직접 만든 `split_at_mut`과 표준 라이브러리의 `split_at_mut`을 차례로 써 봅니다.

```sh
cargo run -p p0-ch19-advanced-features --example 01_unsafe_basics
```

```text
1. 원시 포인터 만들기
    r1 = 0x16b31cf7c, r2 = 0x16b31cf7c
    같은 곳을 가리키는가? true
2. 원시 포인터 역참조
    *r2 = 6
    num = 6
    null인가? r1: false, nothing: true
3. unsafe 함수 호출
    read_at(first, 1) = 20
    numbers.get_unchecked(2) = 30
4. 직접 만든 split_at_mut
    left = [100, 2, 3], right = [400, 5, 6]
    values = [100, 2, 3, 400, 5, 6]
    표준 split_at_mut 사용 후 values = [100, 30, 3, 400, 5, 6]
```

`unsafe` 블록 하나를 지우고 실행해 보면 4.1절의 E0133을 볼 수 있습니다. `read_at` 본문의 `unsafe { }`만 지우면 에러가 아니라 `unsafe_op_in_unsafe_fn` 경고가 납니다(2.3절).

### 02_ffi_static_unsafe_trait — C 함수, 내보내기, 정적 변수, unsafe 트레이트

C 표준 라이브러리의 `abs`, `strlen`, `sqrt`를 부르고, 직접 내보낸 `ch19_add`를 C 쪽 선언(`c_side`)을 통해 다시 부릅니다. `#[unsafe(no_mangle)]`를 지우면 링크 에러가 나는 것도 확인해 보세요(2.4절).

```sh
cargo run -p p0-ch19-advanced-features --example 02_ffi_static_unsafe_trait
```

```text
1. C 함수 호출
    abs(-3) = 3
    strlen("hello") = 5, strlen("안녕") = 6
    sqrt(2.0) = 1.4142135623730951
2. C에 내보낸 함수
    ch19_add(1, 2) = 3
    c_side::ch19_add(2, 3) = 5
3. 정적 변수
    GREETING = 안녕하세요
    COUNTER = 7 (복사), 7 (원시 포인터)
    LOG = ["첫 번째", "두 번째"]
4. unsafe 트레이트
    zeroed::<u32>() = 0, zeroed::<[u8; 4]>() = [0, 0, 0, 0]
```

### 03_associated_types_operators — 연관 타입과 연산자 오버로딩

2번 구역에서 `let as_number: Option<u32>`의 타입 표기를 지우면 2.8절의 E0283을, 4번 구역에 `Meters(2) + Millimeters(500)`을 추가하면 2.9절의 E0369를 볼 수 있습니다.

```sh
cargo run -p p0-ch19-advanced-features --example 03_associated_types_operators
```

```text
1. 연관 타입: 타입 표기 없이 씁니다
    counter.next() = Some(1)
    total(Counter) = 6
2. 제네릭 트레이트: 어느 구현인지 알려 줘야 합니다
    Some(1), Some("#2"), Some(3)
3. Point + Point
    Point { x: 3, y: 3 }
    p + p == p.add(p)? true
4. Millimeters + Meters
    Millimeters(2500)
5. ch8의 String + &str도 Add입니다
    안녕하세요
```

2번 구역의 세 값이 `Some(1)`, `Some("#2")`, `Some(3)`인 것은 세 호출이 모두 같은 `counter`의 상태(`count`)를 한 칸씩 진행시키기 때문입니다. 구현만 다를 뿐 같은 값을 공유합니다.

### 04_qualified_syntax_supertraits — 완전 정규화 문법과 슈퍼트레이트

```sh
cargo run -p p0-ch19-advanced-features --example 04_qualified_syntax_supertraits
```

```text
1. 같은 이름의 메서드
    person.fly()         → *팔을 힘차게 휘젓는다*
    Pilot::fly(&person)  → 기장입니다. 곧 이륙하겠습니다.
    Wizard::fly(&person) → 날아올라라!
    <Human as Wizard>::fly(&person) → 날아올라라!
2. 같은 이름의 연관 함수
    Dog::baby_name()             → 바둑이
    <Dog as Animal>::baby_name() → 강아지
3. 슈퍼트레이트
    **********
    *        *
    * (1, 3) *
    *        *
    **********
    (1, 3)을(를) 상자에 담았습니다
```

상자의 너비는 `output.len()`, 즉 바이트 수로 계산합니다. 한글처럼 한 글자가 여러 바이트인 문자열을 넣으면 상자가 어긋납니다.

### 05_newtype_and_alias — 뉴타입과 타입 별칭

`cancel_order(order)`를 `cancel_order(user)`로 바꾸면 2.12절의 E0308을 볼 수 있습니다.

```sh
cargo run -p p0-ch19-advanced-features --example 05_newtype_and_alias
```

```text
1. 뉴타입으로 외부 타입에 외부 트레이트 구현하기
    w = [안녕, 세상]
    w.len() = 2, 첫 원소 = Some("안녕")
2. 뉴타입으로 값을 헷갈리지 않게 하기
    주문 1024번을 취소했습니다
    user = UserId(7), 안의 값 = 7
    size_of::<UserId>() = 4, size_of::<u32>() = 4
3. 타입 별칭은 같은 타입입니다
    distance + extra = 8
    나중에 실행 1
    나중에 실행 2
4. Result 별칭
    Ok(42)
    에러: 나이가 아닙니다: '마흔'
```

### 06_never_and_sized — never 타입과 동적 크기 타입

크기(바이트 수)는 64비트 환경에서 확인한 값입니다. `byte_size`의 `?Sized`를 지우면 `"hello"`, `slice`, `shown`을 넘기는 줄에서 2.15절의 E0277이 납니다.

```sh
cargo run -p p0-ch19-advanced-features --example 06_never_and_sized
```

```text
1. never 타입
    숫자만 더한 값: 40
    100 / 4 = 25
    my_unwrap(Some(7)) = 7
2. 동적 크기 타입은 참조(포인터) 뒤에 둡니다
    가리키는 str의 크기: short = 2, long = 15
    &i32: 8, &str: 16, &[i32]: 16, &dyn Display: 16, Box<str>: 16
3. ?Sized 바운드 덕분에 크기가 정해진 타입과 아닌 타입을 모두 받습니다
    byte_size(&5_i32)  = 4
    byte_size(slice)   = 12
    byte_size("hello") = 5
    byte_size(36.5°C)  = 8
```

`"스물"`은 숫자로 바꿀 수 없어 `continue`로 건너뛰었으므로 합은 `10 + 30 = 40`입니다. 마지막 줄의 `8`은 `&dyn Display`가 가리키는 `Celsius`(`f64` 하나)의 크기입니다. 트레이트 객체가 가리키는 값의 실제 크기는 실행 중에 메서드 표에서 읽어 옵니다.

### 07_fn_pointers_closures — 함수 포인터와 클로저 반환

```sh
cargo run -p p0-ch19-advanced-features --example 07_fn_pointers_closures
```

```text
1. 함수 포인터
    do_twice(add_one, 5) = 12
    add_one(10) = 11
    double(10) = 20
    negate(10) = -10
2. fn과 Fn 트레이트
    do_twice_generic(add_one, 5) = 12
    do_twice_generic(|x| x + bonus, 5) = 210
    ["1", "2", "3"] == ["1", "2", "3"]? true
3. 생성자도 함수입니다
    [Value(1), Value(2), Value(3), Stop]
    Value들의 합 = 6
    [Meters(5), Meters(10)], [Some(5), Some(10)]
4. 클로저 반환하기
    make_adder(10)(5) = 15
    [+3, *3, 그대로] 적용 결과 = [8, 15, 5]
    pick(true)(5) = -5, pick(false)(5) = 6
```

`do_twice(|x| x + bonus, 5)`를 추가하면 2.16절의 E0308을, `make_op`의 반환 타입을 `impl Fn(i32) -> i32`로 바꾸고 `Box::new(...)`를 벗기면 2.17절의 E0308을 볼 수 있습니다.

### 08_macro_rules — 선언적 매크로

```sh
cargo run -p p0-ch19-advanced-features --example 08_macro_rules
```

```text
1. my_vec!
    [] [1, 2, 3] ["가", "나"]
2. max!: 인자 개수가 정해져 있지 않습니다
    max!(7) = 7
    max!(3, 9, 2) = 9
    max!(1.5, 0.5) = 1.5
3. show!: 코드와 값을 함께 출력
    1 + 2 * 3 = 7
    numbers.len() = 3
    max!(4, 8) = 8
4. unit_type!이 만든 타입
    9.58초 동안 100m 달렸습니다
    초속 10.44m
```

`show!(max!(4, 8))`의 출력에서 `stringify!`가 안쪽 매크로를 펼치기 전의 모양(`max!(4, 8)`)을 그대로 문자열로 만든 것을 볼 수 있습니다. `my_vec!`의 첫 번째 갈래 `() => { Vec::new() };`를 지우면 2.19절의 `unused_mut` 경고를 볼 수 있습니다.

### projects/hello_macro — derive 매크로

독립 워크스페이스이므로 프로젝트 디렉터리로 이동해 실행합니다. 처음 빌드할 때 crates.io에서 `syn`, `quote`와 그 의존성을 내려받으므로 인터넷 연결이 필요합니다.

```sh
cd part0-getting-started/ch19-advanced-features/projects/hello_macro
cargo run
```

Cargo의 진행 메시지(`Compiling ...` 등) 뒤에 다음이 출력됩니다.

```text
안녕하세요, 매크로! 제 이름은 Pancakes입니다!
안녕하세요, 매크로! 제 이름은 Waffles입니다!
```

실행 파일이 `pancakes` 하나뿐이라 워크스페이스 루트에서 `cargo run`만으로 실행됩니다. 빌드 결과는 이 디렉터리의 `target/`에 생기므로, 다 본 뒤에는 `cargo clean`으로 지워도 됩니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch19-advanced-features
rustc --edition 2024 compile_fail/e0133_unsafe_without_block.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐서 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, `--out-dir`로 다른 디렉터리를 지정하거나 확인한 뒤 지우세요. 에러 코드의 설명은 `rustc --explain E0133`처럼 볼 수 있습니다.

### 4.1 unsafe 블록 없이 원시 포인터 역참조·unsafe 함수 호출 — E0133

파일: [`compile_fail/e0133_unsafe_without_block.rs`](compile_fail/e0133_unsafe_without_block.rs)

```text
error[E0133]: dereference of raw pointer is unsafe and requires unsafe block
  --> compile_fail/e0133_unsafe_without_block.rs:20:17
   |
20 |     let value = *r; // 원시 포인터 역참조
   |                 ^^ dereference of raw pointer
   |
   = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior

error[E0133]: call to unsafe function `read_first` is unsafe and requires unsafe block
  --> compile_fail/e0133_unsafe_without_block.rs:21:17
   |
21 |     let again = read_first(r); // unsafe 함수 호출
   |                 ^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
```

- **읽는 법**: 두 에러 모두 "이 연산은 unsafe라서 unsafe 블록이 필요하다"는 뜻입니다. 첫 번째 `note:`는 원시 포인터가 null이거나, 해제된 곳을 가리키거나(dangling), 정렬이 맞지 않을 수 있다고 이유를 설명합니다. 두 번째 `note:`는 그 함수의 문서(`# Safety`)를 읽으라고 안내합니다. 18번 줄의 `&raw const num`(포인터 만들기)은 에러가 아니라는 점도 확인하세요.
- **해결**: 그 연산만 `unsafe { }`로 감싸고, 바로 위에 `// SAFETY:` 주석으로 왜 안전한지 적습니다(`num`이 살아 있고, `read_first`의 조건을 만족한다). 블록을 크게 잡아 함수 전체를 감싸지 않습니다.
- 같은 E0133이 `static mut`에 접근할 때(`use of mutable static is unsafe`)와, ch12에서 본 2024 에디션의 `std::env::set_var`를 부를 때도 납니다.

### 4.2 안전한 코드로 쓴 `split_at_mut` — E0499

파일: [`compile_fail/e0499_safe_split_at_mut.rs`](compile_fail/e0499_safe_split_at_mut.rs)

```text
error[E0499]: cannot borrow `*values` as mutable more than once at a time
  --> compile_fail/e0499_safe_split_at_mut.rs:15:31
   |
10 | fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
   |                         - let's call the lifetime of this reference `'1`
...
15 |     (&mut values[..mid], &mut values[mid..])
   |     --------------------------^^^^^^--------
   |     |     |                   |
   |     |     |                   second mutable borrow occurs here
   |     |     first mutable borrow occurs here
   |     returning this value requires that `*values` is borrowed for `'1`
   |
   = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

- **원인**: 대여 검사기는 슬라이스의 **어느 부분**을 빌렸는지 구분하지 않습니다. `values[..mid]`와 `values[mid..]`가 겹치지 않는다는 것은 사람이 아는 사실일 뿐, 컴파일러가 보기에는 `*values`를 두 번 가변으로 빌린 것입니다(2.3절). 구조체의 서로 다른 필드는 따로 빌릴 수 있지만, 인덱스로 나눈 범위는 그렇지 않습니다.
- **해결**: `help:`대로 표준 라이브러리의 `values.split_at_mut(mid)`를 씁니다. 그런 함수가 없는 상황이라면 예제 01처럼 원시 포인터와 `slice::from_raw_parts_mut`로 구현하고, 경계 검사(`assert!`)로 안전성을 확보한 뒤 보통 함수로 감쌉니다.
- `unsafe` 블록으로 이 코드를 감싸도 에러는 그대로입니다. `unsafe`는 대여 검사기를 끄지 않습니다(2.1절).

### 4.3 트레이트 이름만으로 연관 함수 호출 — E0790

파일: [`compile_fail/e0790_ambiguous_associated_fn.rs`](compile_fail/e0790_ambiguous_associated_fn.rs)

```text
error[E0790]: cannot call associated function on trait without specifying the corresponding `impl` type
  --> compile_fail/e0790_ambiguous_associated_fn.rs:29:33
   |
10 |     fn baby_name() -> String;
   |     ------------------------- `Animal::baby_name` defined here
...
29 |     println!("새끼 개를 부르는 말: {}", Animal::baby_name());
   |                                         ^^^^^^^^^^^^^^^^^^^ cannot call associated function of trait
   |
help: use the fully-qualified path to the only available implementation
   |
29 |     println!("새끼 개를 부르는 말: {}", <Dog as Animal>::baby_name());
   |                                         +++++++       +
```

- **원인**: `baby_name`에는 `self`가 없으므로 인자를 보고 어느 타입의 구현인지 정할 수 없습니다(2.10절). 이 파일에는 `Animal`을 구현한 타입이 `Dog` 하나뿐이지만, 컴파일러는 그것만으로 고르지 않습니다. 나중에 `Cat`이 `Animal`을 구현하면 이 호출의 뜻이 바뀌어 버리기 때문입니다.
- **해결**: `help:`대로 `<Dog as Animal>::baby_name()`처럼 타입과 트레이트를 모두 적습니다. `Dog::baby_name()`은 타입 자신의 연관 함수(`바둑이`)를 부르므로 원하는 결과가 아닙니다.

### 4.4 외부 타입에 외부 트레이트 구현 — E0117

파일: [`compile_fail/e0117_orphan_rule.rs`](compile_fail/e0117_orphan_rule.rs)

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
  --> compile_fail/e0117_orphan_rule.rs:11:1
   |
11 | impl fmt::Display for Vec<String> {
   | ^^^^^^^^^^^^^^^^^^^^^^-----------
   |                       |
   |                       `Vec` is not defined in the current crate
   |
   = note: impl doesn't have any local type before any uncovered type parameters
   = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
   = note: define and implement a trait or new type instead
```

- **원인**: ch10 2.7절의 고아 규칙입니다. `Display`도 `Vec<String>`도 이 크레이트에서 정의하지 않았습니다. 서로 다른 크레이트가 같은 (트레이트, 타입) 쌍을 각자 구현하면 어느 것을 쓸지 정할 수 없게 되므로 막는 것입니다.
- **해결**: 마지막 `note:`의 "new type"이 2.12절의 뉴타입입니다. `struct Wrapper(Vec<String>);`를 만들어 `impl fmt::Display for Wrapper`로 구현합니다(예제 05). 내 트레이트를 새로 정의해 `Vec<String>`에 구현하는 것도 허용됩니다("define and implement a trait").

### 4.5 슈퍼트레이트를 구현하지 않은 타입 — E0277

파일: [`compile_fail/e0277_missing_supertrait.rs`](compile_fail/e0277_missing_supertrait.rs)

```text
error[E0277]: `Point` doesn't implement `std::fmt::Display`
  --> compile_fail/e0277_missing_supertrait.rs:26:23
   |
26 | impl OutlinePrint for Point {}
   |                       ^^^^^ unsatisfied trait bound
   |
help: the trait `std::fmt::Display` is not implemented for `Point`
  --> compile_fail/e0277_missing_supertrait.rs:21:1
   |
21 | struct Point {
   | ^^^^^^^^^^^^
note: required by a bound in `OutlinePrint`
  --> compile_fail/e0277_missing_supertrait.rs:11:21
   |
11 | trait OutlinePrint: fmt::Display {
   |                     ^^^^^^^^^^^^ required by this bound in `OutlinePrint`
```

(같은 이유로 `p.outline_print()`를 부르는 줄에서도 E0277이 한 번 더 납니다.)

- **읽는 법**: ch10에서 본 E0277과 같은 구조입니다. 첫 줄이 만족되지 않은 조건(`Point: Display`)이고, 마지막 `note:`가 그 조건이 `OutlinePrint`의 정의(`: fmt::Display`)에서 왔다고 알려 줍니다.
- **해결**: `impl fmt::Display for Point { ... }`를 작성합니다(예제 04). 슈퍼트레이트는 구현을 물려주지 않으므로 `Display` 구현은 직접 해야 합니다.

### 4.6 크기를 알 수 없는 타입의 지역 변수 — E0277

파일: [`compile_fail/e0277_unsized_local.rs`](compile_fail/e0277_unsized_local.rs)

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
  --> compile_fail/e0277_unsized_local.rs:10:9
   |
10 |     let owned: str = *greeting;
   |         ^^^^^ doesn't have a size known at compile-time
   |
   = help: the trait `Sized` is not implemented for `str`
   = note: all local variables must have a statically known size
help: consider borrowing here
   |
10 |     let owned: &str = *greeting;
   |                +
```

- **원인**: `*greeting`은 참조를 벗긴 `str` 값 자체이고, `str`은 동적 크기 타입이라 `Sized`가 아닙니다(2.15절). `note:`대로 지역 변수는 크기를 컴파일 시점에 알아야 합니다.
- **해결**: 포인터 뒤에 둡니다. 빌려 쓰려면 `let owned: &str = greeting;`, 소유하려면 `let owned: String = greeting.to_string();`이나 `Box<str>`를 씁니다. `help:`의 제안(`&str`에 `*greeting`을 그대로 대입)은 그대로 따르면 E0308이 나므로 `*`도 함께 지워야 합니다.
- The Book의 예처럼 `let s1: str = "Hello there!";`라고 쓰면 이 E0277과 함께 E0308(`` expected `str`, found `&str` ``)도 납니다. 문자열 리터럴의 타입이 `&str`이기 때문입니다.

### 4.7 derive 매크로를 쓰면서 트레이트를 가져오지 않음 — E0404

`compile_fail/`이 아니라 `projects/hello_macro`에서 재현합니다. `pancakes/src/main.rs`에서 `use hello_macro::HelloMacro;` 줄을 지우고 `cargo build`를 실행하면 다음 에러가 `#[derive(HelloMacro)]`마다 납니다.

```text
error[E0404]: expected trait, found derive macro `HelloMacro`
  --> pancakes/src/main.rs:11:10
   |
11 | #[derive(HelloMacro)]
   |          ^^^^^^^^^^ not a trait
   |
   = note: this error originates in the derive macro `HelloMacro` (in Nightly builds, run with -Z macro-backtrace for more info)
```

- **원인**: derive가 덧붙인 `impl HelloMacro for Pancakes`는 `main.rs` 안에서 해석됩니다(2.20절). 트레이트를 가져오지 않았으므로 `HelloMacro`라는 이름으로 찾을 수 있는 것은 derive 매크로뿐이고, 컴파일러는 "트레이트가 와야 할 자리에 derive 매크로가 있다"고 보고합니다. `note:`가 이 코드가 매크로에서 나왔다고 알려 줍니다.
- **해결**: 트레이트도 가져옵니다(`use hello_macro::HelloMacro;`). 매크로를 만드는 쪽에서 `quote!` 안에 `impl ::hello_macro::HelloMacro for #name`처럼 전체 경로를 쓰면 쓰는 쪽이 트레이트를 가져오지 않아도 구현은 만들어집니다. 다만 `Pancakes::hello_macro()`를 부르려면 어차피 트레이트가 스코프에 있어야 합니다(ch10 2.6절).

## 5. 연습 문제

### 연습 1. `Point`에 연산자 더 구현하기

[`03_associated_types_operators.rs`](examples/03_associated_types_operators.rs)의 `Point`에 다음 연산자를 구현하세요.

- `p - q` (`Sub`): `Point { x: 3, y: 3 } - Point { x: 1, y: 0 }`은 `Point { x: 2, y: 3 }`
- `-p` (`Neg`): `-Point { x: 1, y: 0 }`은 `Point { x: -1, y: 0 }`
- `p * 3` (`Mul<i32>`): `Point { x: 1, y: 2 } * 3`은 `Point { x: 3, y: 6 }`
- `3 * p`: 결과는 위와 같습니다. 이 구현은 고아 규칙에 걸릴까요?

<details>
<summary>힌트</summary>

모두 `use std::ops::{Mul, Neg, Sub};`로 가져와 `Add`와 같은 모양으로 구현합니다. `Neg`는 단항 연산자라서 `fn neg(self) -> Point`처럼 인자가 `self`뿐입니다. `p * 3`은 `impl Mul<i32> for Point`이고 `Rhs`가 `i32`입니다. `3 * p`는 왼쪽이 `i32`이므로 `impl Mul<Point> for i32 { type Output = Point; fn mul(self, p: Point) -> Point { p * self } }`로 구현합니다. `Mul`도 `i32`도 표준 라이브러리의 것이지만, 트레이트의 타입 인자 자리에 내 타입 `Point`가 있으므로 고아 규칙이 허용합니다(규칙의 정확한 조건은 4.4절 에러의 링크인 Reference의 orphan rules에 있습니다).

</details>

### 연습 2. 처음과 끝 원소를 동시에 가변으로 빌리기

슬라이스의 첫 원소와 마지막 원소를 동시에 고칠 수 있게 두 가변 참조를 돌려주는 함수를 작성하세요. 길이가 2보다 작으면 `None`입니다.

```rust
fn first_and_last_mut(values: &mut [i32]) -> Option<(&mut i32, &mut i32)>
```

`let mut v = [1, 2, 3, 4];`에 대해 돌려받은 두 참조를 `std::mem::swap(a, b)`로 바꾸면 `v`는 `[4, 2, 3, 1]`, `first_and_last_mut(&mut [7])`은 `None`이어야 합니다. 먼저 `(&mut values[0], &mut values[len - 1])`로 써서 어떤 에러가 나는지 확인한 뒤, 원시 포인터로 고치세요. 그다음 `unsafe` 없이 같은 일을 하는 방법도 찾아보세요.

<details>
<summary>힌트</summary>

인덱스 두 개로 빌리면 4.2절과 같은 E0499입니다. `let ptr = values.as_mut_ptr();`로 포인터를 얻은 뒤, 길이를 확인한 다음에만 `unsafe { Some((&mut *ptr, &mut *ptr.add(len - 1))) }`로 원시 포인터를 다시 참조로 만듭니다(`&mut *포인터`). `len >= 2`이면 두 위치가 서로 다르고 모두 슬라이스 안이라는 것을 `// SAFETY:` 주석으로 적으세요. 안전한 방법은 표준 라이브러리의 `split_first_mut`를 쓰는 것입니다. `let (first, rest) = values.split_first_mut()?; let last = rest.last_mut()?;`처럼 첫 원소와 나머지로 먼저 나누면 두 참조가 서로 다른 슬라이스에서 나오므로 대여 검사기가 받아들입니다. 이 함수들도 안쪽은 unsafe로 구현되어 있습니다.

</details>

### 연습 3. `hashmap!` 매크로

`HashMap`을 한 줄로 만드는 매크로를 작성하세요.

```rust
let scores: HashMap<&str, i32> = hashmap! { "철수" => 90, "영희" => 85 };
let empty: HashMap<&str, i32> = hashmap! {};
```

`scores["영희"]`는 `85`, `empty.len()`은 `0`이어야 하고, 끝에 쉼표가 있어도 동작해야 합니다.

<details>
<summary>힌트</summary>

`my_vec!`과 같은 구조입니다. 패턴은 `( $( $k:expr => $v:expr ),+ $(,)? )`이고, `=>`는 패턴 안에서 그대로 맞춰야 할 글자입니다. 바꿀 코드는 `let mut map = HashMap::new();` 뒤에 `$( map.insert($k, $v); )+`, 마지막에 `map`을 두고 전체를 `{ }` 블록으로 감쌉니다. 빈 호출은 `() => { HashMap::new() };` 갈래를 위에 따로 두어야 `unused_mut` 경고가 나지 않습니다. 매크로는 `hashmap! { }`, `hashmap!( )`, `hashmap![ ]` 중 어느 괄호로도 부를 수 있습니다.

</details>

### 연습 4. 제네릭 구조체에도 `#[derive(HelloMacro)]` 붙이기

`projects/hello_macro/pancakes/src/main.rs`에 다음을 추가하고 빌드해 보세요.

```rust
#[derive(HelloMacro)]
struct Stack<T> {
    items: Vec<T>,
}
```

E0107(`` missing generics for struct `Stack` ``)이 납니다. 매크로가 만든 `impl HelloMacro for Stack`에 `<T>`가 빠졌기 때문입니다. `hello_macro_derive`를 고쳐 `Stack::<i32>::hello_macro()`가 `안녕하세요, 매크로! 제 이름은 Stack입니다!`를 출력하게 하세요.

<details>
<summary>힌트</summary>

ch10의 `impl<T> Point<T>`처럼 `impl<T> HelloMacro for Stack<T>`를 만들어야 합니다. `syn::DeriveInput`의 `generics` 필드에 제네릭 정보가 있고, `let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();`로 세 조각을 얻을 수 있습니다. `quote!` 안을 `impl #impl_generics HelloMacro for #name #ty_generics #where_clause { ... }`로 바꾸면 됩니다. 제네릭이 없는 `Pancakes`에서는 세 조각이 모두 빈 토큰이 되므로 기존 동작은 그대로입니다. `items` 필드를 읽지 않으면 `dead_code` 경고가 나므로 `main`에서 한 번 읽어 주세요.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [20. Advanced Features](https://doc.rust-lang.org/book/ch20-00-advanced-features.html)
  - [20.1 Unsafe Rust](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html): 2.1~2.7절. 이 챕터는 `abs` 대신 `sqrt`를 `safe fn`으로 선언했고(2.4절), `*(&raw const COUNTER)` 대신 포인터를 변수에 담았습니다(2.5절).
  - [20.2 Advanced Traits](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html): 2.8~2.12절
  - [20.3 Advanced Types](https://doc.rust-lang.org/book/ch20-03-advanced-types.html): 2.12~2.15절
  - [20.4 Advanced Functions and Closures](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html): 2.16~2.17절
  - [20.5 Macros](https://doc.rust-lang.org/book/ch20-05-macros.html): 2.18~2.20절. The Book은 `syn` 2.0을 쓰고, 이 챕터는 3.0.10을 썼습니다.
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
  - [Unsafe Operations](https://doc.rust-lang.org/rust-by-example/unsafe.html): 원시 포인터 역참조와 `slice::from_raw_parts`. 이 절은 unsafe의 용도를 네 가지로 소개하며 유니언은 빼고 셉니다. 하위 절 [Inline assembly](https://doc.rust-lang.org/rust-by-example/unsafe/asm.html)는 Part 9 `ch10-unsafety-const-asm`의 내용입니다.
  - [Traits > Operator Overloading](https://doc.rust-lang.org/rust-by-example/trait/ops.html): `Foo + Bar`와 `Bar + Foo`를 따로 구현하는 예로, 2.9절의 "순서마다 따로 구현한다"를 보여 줍니다.
  - [Traits > Disambiguating overlapping traits](https://doc.rust-lang.org/rust-by-example/trait/disambiguating.html): 타입 자신의 메서드 없이 두 트레이트가 `get`을 가진 경우(2.10절의 E0034)와 `<Form as UsernameWidget>::get(&form)`.
  - [Traits > Supertraits](https://doc.rust-lang.org/rust-by-example/trait/supertraits.html): 슈퍼트레이트를 둘 가진 `CompSciStudent: Programmer + Student`. `&dyn CompSciStudent`는 [ch17](../ch17-trait-objects-oop/README.md)의 트레이트 객체입니다.
  - [Generics > New Type Idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html): `Miles`와 `Kilometers`로 단위를 구분하는 뉴타입, 구조 분해로 안쪽 값 꺼내기.
  - [Generics > Associated items > Associated types](https://doc.rust-lang.org/rust-by-example/generics/assoc_items/types.html): 앞 절 [The Problem](https://doc.rust-lang.org/rust-by-example/generics/assoc_items/the_problem.html)과 함께 읽으면 제네릭 트레이트를 연관 타입으로 바꿨을 때 바운드가 짧아지는 것을 볼 수 있습니다(2.8절).
  - [Types > Aliasing](https://doc.rust-lang.org/rust-by-example/types/alias.html): 타입 별칭(2.13절).
  - [Functions > Diverging functions](https://doc.rust-lang.org/rust-by-example/fn/diverging.html): `!`를 반환하는 함수와 `continue`의 타입(2.14절).
  - [Functions > Closures > Input functions](https://doc.rust-lang.org/rust-by-example/fn/closures/input_functions.html), [As output parameters](https://doc.rust-lang.org/rust-by-example/fn/closures/output_parameters.html): 클로저 자리에 함수 넘기기와 `impl Fn` 반환(2.16~2.17절).
  - [macro_rules!](https://doc.rust-lang.org/rust-by-example/macros.html): 하위 절 [Designators](https://doc.rust-lang.org/rust-by-example/macros/designators.html)(프래그먼트 지정자), [Overload](https://doc.rust-lang.org/rust-by-example/macros/overload.html)(여러 갈래), [Repeat](https://doc.rust-lang.org/rust-by-example/macros/repeat.html)(반복과 재귀), [Variadics](https://doc.rust-lang.org/rust-by-example/macros/variadics.html)가 2.19절에 해당합니다.
- Rustlings
  - [`21_macros`](https://github.com/rust-lang/rustlings/tree/main/exercises/21_macros): `macros1.rs`는 매크로 호출에 `!` 붙이기, `macros2.rs`는 정의를 호출보다 위로 옮기기(2.18절), `macros3.rs`는 모듈 안의 매크로를 밖에서 쓰기(2.19절의 `#[macro_use]`), `macros4.rs`는 갈래 사이의 구분자(`;`) 빠뜨린 것 고치기입니다.
  - Rustlings에는 unsafe나 고급 트레이트만 다루는 묶음은 없습니다. 이 챕터의 연습 문제로 대신합니다.
- Edition Guide (2.7절의 변화)
  - [Unsafe extern blocks](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html), [Unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html), [unsafe_op_in_unsafe_fn warning](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html), [Disallow references to static mut](https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html)
- 더 깊이
  - [The Rustonomicon](https://doc.rust-lang.org/nomicon/): unsafe Rust를 올바르게 쓰는 법. Part 10 전체가 이 문서를 다룹니다.
  - [The Little Book of Rust Macros](https://lukaswirth.dev/tlborm/): `macro_rules!`를 깊이 다루는 문서로, Rustlings `21_macros`의 README가 추천합니다.
  - [`syn` 3.0.10 문서](https://docs.rs/syn/3.0.10/syn/), [`quote` 1.0.47 문서](https://docs.rs/quote/1.0.47/quote/): `DeriveInput`의 필드와 `quote!`의 반복 문법(`#(...)*`).
  - [Miri](https://github.com/rust-lang/miri), [cargo-expand](https://github.com/dtolnay/cargo-expand): 2.7절과 2.18절에서 이름만 소개한 도구입니다.
- 에러 설명: `rustc --explain E0790`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
