# ch1. 표준 라이브러리 둘러보기와 API 문서 읽기

Part 0에서는 언어를 배우면서 `Vec`, `String`, `Option` 같은 표준 라이브러리의 타입을 필요한 만큼만 썼습니다. Part 1은 표준 라이브러리 자체를 모듈 단위로 읽어 나가는 파트이고, 이 챕터는 그 출발점입니다. 세 가지를 다룹니다.

- **표준 라이브러리의 구조**: 흔히 `std`라고 부르는 것은 `core`, `alloc`, `std` 세 크레이트가 겹친 것입니다. 그리고 `use` 없이 쓸 수 있는 이름의 모음인 프렐류드(prelude)가 있습니다.
- **API 문서 읽는 법**: 타입 페이지의 구성, 메서드 시그니처(signature), 안정화 버전 같은 표시, 검색을 익힙니다. Part 1의 다른 챕터는 "API 문서에서 메서드를 찾아 시그니처를 읽는다"는 것을 전제로 쓰였으므로, 여기서 읽는 법을 먼저 익혀 둡니다.
- **원시 타입의 메서드**: [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md)에서 정수, 실수, `char`, `bool`, 튜플, 배열이라는 타입을 배웠습니다. 이 챕터에서는 그 타입들에 붙은 메서드와 연관 상수(associated constant)를 문서에서 계열별로 알아봅니다.

- 패키지: `p1-ch1-std-overview`
- 기준 문서: [std 크레이트 첫 페이지](https://doc.rust-lang.org/std/index.html), 원시 타입 페이지([`i32`](https://doc.rust-lang.org/std/primitive.i32.html), [`f64`](https://doc.rust-lang.org/std/primitive.f64.html), [`char`](https://doc.rust-lang.org/std/primitive.char.html), [`bool`](https://doc.rust-lang.org/std/primitive.bool.html), [`array`](https://doc.rust-lang.org/std/primitive.array.html), [`slice`](https://doc.rust-lang.org/std/primitive.slice.html), [`tuple`](https://doc.rust-lang.org/std/primitive.tuple.html)), [`std::prelude`](https://doc.rust-lang.org/std/prelude/index.html)
- 확인한 툴체인: rustc 1.98.1, cargo 1.98.1 (Edition 2024). 문서 화면은 같은 버전의 로컬 문서로 확인했습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `core`·`alloc`·`std`가 각각 무엇을 전제하는지와 왜 나뉘어 있는지 설명할 수 있고, `core::option::Option`과 `std::option::Option`처럼 같은 타입이 여러 경로로 보이는 이유를 설명할 수 있다.
2. `std::prelude` 문서에서 2024 에디션 프렐류드의 구성을 확인하고, 어떤 이름을 `use` 없이 쓸 수 있고 어떤 이름은 가져와야 하는지 판단할 수 있다.
3. API 문서 타입 페이지의 구성(Implementations, Methods from Deref, Trait/Auto Trait/Blanket Implementations)과 표시(안정화 버전, `const`, Deprecated, nightly 전용)를 읽고, 이름 검색과 타입 시그니처 검색으로 원하는 항목을 찾을 수 있다.
4. 메서드 시그니처에서 `self`/`&self`/`&mut self`, 제네릭과 `where`, `impl Trait`, `Option`/`Result` 반환이 뜻하는 바를 읽고, 그대로 호출하는 코드를 작성할 수 있다.
5. 정수·실수·`char`·`bool`·배열·슬라이스·튜플의 메서드를 계열(검사 연산, 포화 연산, 반올림, 분류, 꺼내기 등)로 알아보고 상황에 맞는 것을 골라 쓸 수 있다.

## 2. 핵심 개념

2.1~2.2절은 표준 라이브러리의 구조, 2.3~2.8절은 API 문서 읽는 법, 2.9~2.12절은 원시 타입(primitive type)별 메서드를 다룹니다. 원시 타입 절은 메서드를 모두 나열하지 않고, 문서에서 계열을 알아보는 데 초점을 둡니다.

### 2.1 `core`, `alloc`, `std`: 표준 라이브러리의 세 층

표준 라이브러리는 세 크레이트로 되어 있고, 위층이 아래층을 포함합니다.

| 크레이트 | 전제하는 것 | 들어 있는 것(예) |
| --- | --- | --- |
| `core` | 아무것도 없음 | 원시 타입의 메서드 대부분, `Option`, `Result`, `Iterator`, `Clone`, `Copy`, `mem::swap`, `time::Duration` |
| `alloc` | 힙 할당기 | `Box`, `Vec`, `String`, `Rc`, `Arc`, `BTreeMap`, `VecDeque` |
| `std` | 운영체제 | `fs`, `io`, `net`, `thread`, `env`, `process`, `HashMap`, 그리고 `core`와 `alloc`의 항목 전체 |

**왜 나눴을까요?** 마이크로컨트롤러, 운영체제 커널, 부트로더처럼 운영체제가 없는 환경에서는 파일도 스레드도 없고, 힙 할당기조차 없을 수 있습니다. 표준 라이브러리가 한 덩어리였다면 이런 환경에서는 `Option` 하나 쓰려 해도 표준 라이브러리 전체를 포기해야 합니다. 세 층으로 나누면 환경이 허락하는 만큼 쓸 수 있습니다. 할당기가 없으면 `core`만, 할당기가 있으면 `core`와 `alloc`을 씁니다. `core` 문서는 자신을 "의존성이 없는 기반(dependency-free foundation)"이라고 소개하며 힙 할당도, 동시성도, 입출력도 모른다고 설명합니다. `std` 없이 `core`/`alloc`만 쓰는 `#![no_std]` 프로그램은 Part 8 `ch1-intro-no-std`에서 직접 만들어 봅니다.

**우리가 쓰는 보통 프로그램은 `std` 하나만 보면 됩니다.** `std`가 `core`와 `alloc`의 항목을 재내보내기(re-export)하기 때문입니다([Part 0 ch7](../../part0-getting-started/ch7-modules/README.md)의 `pub use`). 그래서 같은 타입이 여러 경로로 보입니다. `std::option::Option`은 `core::option::Option`을 다시 내보낸 것이고, 둘은 이름만 다른 같은 타입입니다. 예제 01은 이것을 `TypeId`(타입마다 유일한 값)로 확인하고, 진단용 함수 `type_name`으로 실제로 정의된 크레이트를 엿봅니다.

```rust
let same = TypeId::of::<core::option::Option<i32>>() == TypeId::of::<std::option::Option<i32>>();
println!("{same}");                         // true
println!("{}", type_name::<Vec<i32>>());    // alloc::vec::Vec<i32>
```

`type_name`의 문서에는 "돌려주는 문자열의 정확한 내용과 형식은 정해져 있지 않고, 컴파일러 버전에 따라 바뀔 수 있다"고 적혀 있습니다. 디버깅할 때 구경하는 용도로만 씁니다.

**경로로 직접 쓸 때의 차이.** `std::`와 `core::`는 어디서나 바로 쓸 수 있습니다. 두 크레이트가 외부 프렐류드(extern prelude)라는 이름 목록에 항상 들어 있기 때문입니다. `alloc`은 그렇지 않아서 `extern crate alloc;` 선언이 있어야 `alloc::vec::Vec`처럼 쓸 수 있습니다. 선언 없이 쓰면 컴파일러가 알려 줍니다.

```text
error[E0433]: cannot find module or crate `alloc` in this scope
  = help: add `extern crate alloc` to use the `alloc` crate
```

**문서에서도 층이 보입니다.** `core` 크레이트의 문서(<https://doc.rust-lang.org/core/>)에도 `f64`, `slice` 같은 원시 타입 페이지가 있는데, `std` 판보다 메서드가 적습니다.

- `f64`: `std` 판 페이지에는 `impl f64` 블록이 두 개 있습니다. `sqrt`, `powi`, `floor`, `round`, `sin` 같은 메서드가 든 블록은 `std`에서 추가한 것이라 `core` 판 페이지에는 없습니다(1.98.1 문서 기준). 대상 플랫폼에 따라 수학 라이브러리 호출로 구현될 수 있는 연산들입니다. `abs`, `is_nan`, `total_cmp`, `clamp`는 `core`에 있습니다.
- `slice`: 정렬 `sort`는 보조 버퍼를 할당하므로 `alloc`에서 추가됩니다. 문서는 `sort_unstable`이 "보조 메모리를 할당하지 않는다"고 설명하며, 이쪽은 `core`에 있습니다. `to_vec`(`Vec`을 만듦), `concat`, `join`도 `alloc`에서 옵니다.

→ [`examples/01_std_layers.rs`](examples/01_std_layers.rs)

### 2.2 프렐류드: `use` 없이 쓸 수 있는 이름들

**프렐류드(prelude)** 는 모든 모듈에 자동으로 가져와지는 이름의 목록입니다. `std::prelude` 문서는 그 목적을 이렇게 설명합니다. 쓰는 것마다 일일이 가져와야 한다면 너무 장황하지만, 쓰지도 않는 것을 잔뜩 가져오는 것도 좋지 않으니 균형을 잡아야 한다. 그래서 프렐류드는 거의 모든 프로그램이 쓰는 것, 특히 트레이트 위주로 작게 유지됩니다.

어떤 목록을 쓸지는 크레이트의 에디션이 정합니다. 이 저장소는 2024 에디션이므로 `std::prelude::rust_2024`입니다. 문서([`std::prelude`](https://doc.rust-lang.org/std/prelude/index.html))에 적힌 내용을 정리하면 다음과 같습니다.

| 분류 | 항목 | 들어온 에디션 |
| --- | --- | --- |
| 타입과 배리언트 | `Option`/`Some`/`None`, `Result`/`Ok`/`Err`, `Box`, `String`, `Vec` | 모든 에디션 |
| 마커 트레이트 | `Copy`, `Send`, `Sized`, `Sync`, `Unpin` | 모든 에디션 |
| 클로저 트레이트 | `Fn`, `FnMut`, `FnOnce`와 `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce` | 모든 에디션 |
| 정리 | `Drop` 트레이트, `drop` 함수 | 모든 에디션 |
| 크기와 정렬 | `size_of`, `size_of_val`, `align_of`, `align_of_val` 함수 | 모든 에디션 |
| 복제와 비교 | `Clone`, `ToOwned`, `ToString`, `PartialEq`, `PartialOrd`, `Eq`, `Ord` | 모든 에디션 |
| 변환과 기본값 | `AsRef`, `AsMut`, `Into`, `From`, `Default` | 모든 에디션 |
| 반복자 | `Iterator`, `Extend`, `IntoIterator`, `DoubleEndedIterator`, `ExactSizeIterator` | 모든 에디션 |
| 매크로 | 표준 매크로 대부분(`println!`, `vec!`, `format!`, `assert_eq!`, `matches!` 등)과 `derive` 매크로 | 모든 에디션 |
| 변환과 반복자 | `TryFrom`, `TryInto`, `FromIterator` | 2021부터 |
| 비동기 | `Future`, `IntoFuture` | 2024부터 |

이 표에서 두 가지를 기억해 둡니다.

- **트레이트가 프렐류드에 있어서 메서드를 바로 부를 수 있습니다.** 트레이트의 메서드는 그 트레이트가 스코프에 있어야 부를 수 있는데([Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md)), `42.to_string()`, `x.clone()`, `v.iter().sum()`의 `sum`, `u8::try_from(300)`이 `use` 없이 되는 것은 `ToString`, `Clone`, `Iterator`, `TryFrom`이 프렐류드에 있기 때문입니다.
- **같은 모듈이라도 일부만 들어 있습니다.** `std::mem`에서 `drop`, `size_of` 등은 프렐류드에 있지만 `swap`, `replace`, `take`는 없습니다(4.5절의 E0425). `Debug`도 마찬가지입니다. 프렐류드에 든 것은 `#[derive(Debug)]`에 쓰는 derive 매크로뿐이고 같은 이름의 트레이트는 아닙니다. 그래서 `use` 없이 `impl Debug for Point`라고 쓰면 ``error[E0404]: expected trait, found derive macro `Debug` ``가 납니다. 트레이트는 `use std::fmt::Debug;`나 `std::fmt::Debug` 경로로 씁니다.

프렐류드에 없어서 자주 가져오게 되는 것들입니다.

| 항목 | 가져오는 경로 |
| --- | --- |
| `HashMap`, `HashSet`, `BTreeMap`, `VecDeque` | `std::collections` |
| `Ordering`(`cmp`가 돌려주는 값) | `std::cmp::Ordering` |
| `Display`, `Debug` 트레이트, `Formatter` | `std::fmt` |
| `swap`, `replace`, `take` | `std::mem` |
| `Rc`, `RefCell` | `std::rc`, `std::cell` |
| `Read`, `Write`, `BufRead` | `std::io` |

`cmp` 메서드 자체는 프렐류드의 `Ord` 트레이트에 있으므로 바로 부를 수 있지만, 돌려받은 `Ordering::Less` 같은 배리언트 이름을 쓰려면 `Ordering`을 가져와야 합니다(예제 02의 5번 구역).

**다른 프렐류드.** 표준 라이브러리에는 `std::io::prelude`처럼 자주 함께 쓰는 항목을 모아 둔 모듈이 더 있고, 외부 크레이트도 자기 프렐류드를 만듭니다([Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md)의 `use rand::prelude::*;`). 문서는 이것들이 "자동으로 가져와지지 않으며 직접 가져와야 한다"는 점에서 "그 프렐류드"와 다르다고 설명합니다. 에디션마다 프렐류드가 달라진 배경과 영향은 Part 2 `ch3-rust-2021`, `ch5-rust-2024-ecosystem`에서 다룹니다.

→ [`examples/02_prelude.rs`](examples/02_prelude.rs)

### 2.3 문서는 어디에 있고, 어느 버전인가

API 문서는 두 곳에서 볼 수 있습니다.

- **온라인**: <https://doc.rust-lang.org/std/>. 최신 안정판 기준입니다. 2026년 11월 18일에 열어 보았을 때 1.100.0 문서였습니다.
- **로컬**: `rustup doc --std`를 실행하면 지금 쓰는 툴체인과 같은 버전의 문서가 브라우저에서 열립니다. 인터넷 없이도 볼 수 있습니다. 브라우저를 열지 않고 경로만 확인하려면 `--path`를 붙입니다.

```console
$ rustup doc --std --path
/Users/<사용자>/.rustup/toolchains/1.98.1-aarch64-apple-darwin/share/doc/rust/html/std/index.html
```

로컬 문서는 `rust-docs` 컴포넌트에 들어 있습니다. `rustup component list --installed`의 목록에 `rust-docs-…`가 있으면 쓸 수 있습니다.

**버전이 다르면 문서와 컴파일러가 어긋납니다.** 온라인 1.100.0 문서의 `bool` 페이지에는 1.100.0에 추가된 `toggle` 메서드가 있지만, 이 저장소의 1.98.1에서 그 메서드를 부르면 컴파일되지 않습니다.

```text
error[E0599]: no method named `toggle` found for type `bool` in the current scope
```

반대로 같은 페이지의 `ok_or`는 1.98.0에 추가되었으므로 1.98.1에서 쓸 수 있습니다(예제 06). 문서 왼쪽 위의 크레이트 이름 옆에 문서 버전이 나오므로(로컬 문서는 `std 1.98.1`), `rustc --version`과 비교해 봅니다. 메서드 오른쪽의 안정화 버전 읽는 법은 2.6절에서 봅니다.

### 2.4 크레이트 페이지와 모듈 페이지

[std 첫 페이지](https://doc.rust-lang.org/std/index.html)는 크레이트 페이지입니다. 위쪽에 표준 라이브러리 소개와 "이 문서를 읽는 법"이 있고, 그 아래에 **Primitive Types**(원시 타입), **Modules**(모듈), **Macros**(매크로), **Keywords**(키워드) 목록이 이어집니다. 목록의 이름 옆에 `Experimental`(nightly 전용), `Deprecation planned`(사용 중단 예정), `Deprecated`(사용 중단) 같은 딱지가 붙기도 합니다. 모듈 페이지(예: `std::collections`)도 같은 모양으로, 설명 뒤에 그 모듈의 Structs, Enums, Traits, Functions, Constants 목록이 나옵니다.

**원시 타입 페이지와 같은 이름의 모듈 페이지는 다릅니다.** 처음에 가장 헷갈리는 부분입니다.

| 페이지 | 주소 | 들어 있는 것 |
| --- | --- | --- |
| 원시 타입 `i32` | `std/primitive.i32.html` | `i32`의 연관 상수(`MIN`, `MAX`, `BITS`), 모든 메서드, 트레이트 구현 |
| 모듈 `std::i32` | `std/i32/index.html` | 예전 방식의 상수 `std::i32::MIN`, `std::i32::MAX`뿐. 모듈 전체가 "Redundant constants module", Deprecation planned |
| 원시 타입 `char` | `std/primitive.char.html` | `char`의 모든 메서드 |
| 모듈 `std::char` | `std/char/index.html` | 메서드가 돌려주는 타입(`ToUppercase` 같은 반복자, 에러 타입)과 예전 방식의 함수·상수 |
| 원시 타입 `f64` | `std/primitive.f64.html` | 연관 상수(`EPSILON`, `NAN`, `MAX` 등)와 메서드 |
| 모듈 `std::f64` | `std/f64/index.html` | 수학 상수 하위 모듈 `consts`(`PI`, `E`, `TAU` 등)와 예전 방식의 상수 |

std 첫 페이지도 이 차이를 설명합니다. `char` 원시 타입 페이지는 문자에 부를 수 있는 모든 메서드를 보여 주므로 "매우 유용하고", `std::char` 모듈 페이지는 그 메서드들이 만드는 반복자와 에러 타입을 설명하므로 "가끔 유용하다"고 합니다. 메서드를 찾을 때는 원시 타입 페이지로 갑니다. 단, 원주율 같은 수학 상수는 `std::f64::consts` 모듈에 있습니다.

### 2.5 타입 페이지 읽기

[`i32`](https://doc.rust-lang.org/std/primitive.i32.html)와 [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) 페이지를 열어 놓고 읽으면 좋습니다.

**맨 위.** 제목(`Primitive Type i32`, `Struct Vec`)과 경로를 복사하는 `Copy item path` 버튼이 있고, 오른쪽에 안정화 버전과 소스 코드로 가는 `Source` 링크가 있습니다. 도구 막대의 `Summary` 버튼을 누르면 긴 설명을 접고 항목 이름만 보여 줍니다(다시 누르면 `Show all`). 구조체 페이지에는 정의가 나옵니다. `Vec`은 `pub struct Vec<T, A = Global> where A: Allocator`입니다. `A = Global`은 기본값이 정해진 타입 매개변수입니다. 할당기를 바꾸는 `Allocator` 트레이트는 nightly 전용이므로, 평소에는 `A`를 지우고 `Vec<T>`로 읽으면 됩니다.

**본문과 사이드바의 절.** 페이지 본문은 설명 다음에 아래 절들이 이어지고, 왼쪽 사이드바에 같은 이름의 목차가 있습니다(1.98.1 문서에서 확인).

| 절 | 들어 있는 것 | 예 |
| --- | --- | --- |
| Associated Constants | `타입::이름`으로 쓰는 연관 상수 | `i32::MAX`, `f64::EPSILON` |
| Associated Functions | `self`가 없는 연관 함수 | `i32::from_str_radix`, `Vec::new` |
| Methods | `self`를 받는 메서드 | `i32::pow`, `Vec::push` |
| Methods from Deref<Target=[T]> | `Deref`로 이어진 타입의 메서드 | `Vec`에서 부르는 `first`, `windows` |
| Trait Implementations | 이 타입이 구현한 트레이트 | `impl Add for i32`, `impl From<u8> for i32` |
| Auto Trait Implementations | 컴파일러가 자동으로 정하는 트레이트 | `Send`, `Sync`, `Unpin` |
| Blanket Implementations | 조건만 맞으면 모든 타입에 붙는 포괄 구현 | `impl<T> ToString for T where T: Display` |

위의 세 절은 본문에서 **Implementations**라는 제목 아래 함께 나옵니다. 타입에 직접 붙은 고유 구현(`impl i32 { ... }`)입니다.

**Implementations: `impl` 블록이 여러 개입니다.** `Vec` 페이지에는 `impl<T> Vec<T>`, `impl<T, A> Vec<T, A> where A: Allocator`, `impl<T, A> Vec<T, A> where T: Clone, A: Allocator` 같은 블록이 차례로 나옵니다. `where T: Clone` 블록에 든 `resize`, `extend_from_slice`는 원소가 `Clone`일 때만 생기는 메서드입니다. [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.11절의 "바운드를 이용한 조건부 구현"을 표준 라이브러리가 그대로 쓰는 것입니다. 메서드를 찾았는데 호출이 안 되면, 그 메서드가 든 블록의 `where` 조건을 확인합니다.

**Methods from Deref: `Vec`과 `String`의 메서드 대부분은 여기에 있습니다.** `Vec<T>`는 `Deref<Target = [T]>`를, `String`은 `Deref<Target = str>`를 구현합니다. 그래서 역참조 강제 변환([Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.5절)으로 슬라이스와 `str`의 메서드를 그대로 부를 수 있습니다. 문서는 이 메서드들을 `Vec` 페이지의 **Methods from Deref<Target = [T]>** 절(`#deref-methods-%5BT%5D`)과 `String` 페이지의 **Methods from Deref<Target = str>** 절(`#deref-methods-str`)에 옮겨 보여 줍니다. `v.first()`, `v.windows(2)`, `v.contains(&3)`은 모두 이 절에 있는 슬라이스의 메서드입니다. `Vec` 고유의 `len`과 슬라이스의 `len`이 두 절에 각각 나오는 것처럼 겹치는 이름도 있습니다. std 첫 페이지도 "`String`과 `Vec<T>`의 많은 메서드 호출은 사실 `str`과 `[T]`의 메서드 호출"이라고 안내합니다.

배열은 사정이 조금 다릅니다. 배열은 `Deref`를 구현하지 않고, 컴파일러가 배열을 슬라이스로 바꿔(coerce) 슬라이스의 메서드를 부릅니다. 배열 문서는 이것이 "배열을 다루는 API의 대부분"이라고 적고 있으므로, 배열의 메서드를 찾을 때는 [`slice`](https://doc.rust-lang.org/std/primitive.slice.html) 페이지를 봅니다.

**Trait Implementations: 연산자, 변환, 비교, 출력이 여기서 옵니다.** 1.98.1 문서의 `i32` 페이지에는 이 절의 항목이 수백 개 있습니다. 전부 읽을 필요는 없고 무엇이 있는지, 무엇이 없는지를 봅니다.

- 연산자: `Add`, `Sub`, `BitAnd`, `Shl` 등. `a + b`, `a & b`, `a << 2`가 되는 이유입니다. `bool` 페이지에 `BitAnd`, `Not`이 있어서 `bool`에 `&`, `!`를 쓸 수 있습니다.
- 변환: `From<u8> for i32`(손실 없는 변환), `TryFrom<i64> for i32`(실패할 수 있는 변환), `FromStr`(`parse`). 이 절에는 `impl From<i32> for f64`처럼 이 타입이 다른 타입으로 변환되는 구현도 함께 나옵니다.
- 비교와 해시: `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`. **없는 것도 정보입니다.** `f64` 페이지에는 `Eq`와 `Ord`가 없습니다. NaN 때문에(2.10절) 실수는 전체 순서가 정해지지 않기 때문이고, 그래서 `f64`는 `Ord`를 요구하는 곳(예: 슬라이스의 `sort`, `ch4-collections`)에 그대로 쓸 수 없습니다.
- 출력: `Display`, `Debug`, `LowerHex`(`{:x}`) 등.

**Auto Trait Implementations.** `Send`, `Sync`([Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md)), `Unpin`([Part 0 ch16](../../part0-getting-started/ch16-async/README.md)), `UnwindSafe`, `RefUnwindSafe`처럼 타입의 구성 요소를 보고 컴파일러가 자동으로 구현 여부를 정하는 트레이트입니다. 스레드 사이에 보낼 수 있는지 확인할 때 봅니다. 목록의 나머지(`Freeze`, `UnsafeUnpin`)는 지금은 넘어가도 됩니다.

**Blanket Implementations.** `impl<T> From<T> for T`, `impl<T, U> Into<U> for T where U: From<T>`, `impl<T> ToString for T where T: Display + ?Sized` 같은 포괄 구현입니다([Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.11절). 거의 모든 타입 페이지에 비슷한 목록이 붙습니다. `i32`에 `to_string()`이 있는 이유를 찾을 때 여기를 봅니다.

**ⓘ 표시.** 두 곳에 나옵니다.

- 반환 타입 옆: `pub fn to_uppercase(self) -> ToUppercase ⓘ`처럼 붙으면, 마우스를 올렸을 때 "Notable traits for ToUppercase"와 함께 `impl Iterator for ToUppercase`가 나옵니다. 돌려받는 값이 반복자라는 뜻입니다. 슬라이스의 `iter`, `windows`, `chunks`에도 붙어 있습니다.
- 코드 예제 옆: "This example panics"(패닉하는 예제), "This example deliberately fails to compile"(일부러 컴파일되지 않는 예제), "This example runs with edition 2018" 같은 설명이 나옵니다. 예제 오른쪽 위의 실행 버튼(`Run code`)은 그 코드를 Rust Playground에서 엽니다.

메서드 설명 안의 **§Panics** 절도 꼭 읽습니다. 예를 들어 슬라이스의 `split_at`에는 "Panics if mid > len"과 함께 패닉하지 않는 대안 `split_at_checked`가 적혀 있습니다.

### 2.6 표시 읽기: 안정화 버전, `const`, Deprecated, nightly 전용

메서드 제목 오른쪽과 아래에 붙는 표시입니다(문서에 실제로 붙은 예).

| 표시 | 예 | 뜻 |
| --- | --- | --- |
| `1.60.0` | `i32::abs_diff` | Rust 1.60.0부터 안정(stable)입니다. 이보다 오래된 툴체인에서는 없습니다. |
| `1.0.0 (const: 1.50.0)` | `u32::pow` | 1.0.0부터 안정이고, 1.50.0부터 `const` 문맥에서도 쓸 수 있습니다. |
| `1.62.0 (const: unstable)` | `bool::then_some` | 안정이지만 `const` 문맥에서는 아직 못 씁니다. |
| 👎 Deprecated since 1.3.0: renamed to join | 슬라이스의 `connect` | 쓰지 말라는 표시입니다. 쓰면 경고가 납니다. |
| 👎 Deprecating in a future version | `i32::min_value`, `std::i32::MAX` | 앞으로 Deprecated가 될 예정입니다. 목록에서는 `Deprecation planned`로 보입니다. |
| 🔬 This is a nightly-only experimental API. (int_roundings #88581) | `i32::div_ceil` | nightly 툴체인에서만 쓸 수 있는 실험 API입니다. 괄호 안은 기능 이름과 추적 이슈 번호입니다. 목록에서는 `Experimental`로 보입니다. |

**`const`.** `const` 표시가 있는 메서드는 상수의 값을 계산하는 데 쓸 수 있습니다. 예제 03은 `const KIB: u32 = 2u32.pow(10);`을 씁니다. `f64::sqrt`처럼 `const`가 아닌 메서드를 상수에 쓰면 거부됩니다.

```text
error[E0015]: cannot call non-const method `f64::<impl f64>::sqrt` in constants
```

**Deprecated.** 1.98.1에서 슬라이스의 `connect`를 쓰면 경고와 함께 바꿀 이름을 알려 줍니다. 반면 "Deprecating in a future version"인 `std::i32::MAX`는 지금은 경고가 나지 않지만(1.98.1에서 확인), 새 코드에서는 문서가 권하는 `i32::MAX`를 씁니다.

```text
warning: use of deprecated method `std::slice::<impl [T]>::connect`: renamed to join
```

**nightly 전용(Experimental).** 문서에는 안정판에서 쓸 수 없는 API도 함께 실려 있습니다. 안정판 툴체인에서 쓰면 E0658입니다.

```text
error[E0658]: use of unstable library feature `int_roundings`
  = note: see issue #88581 <https://github.com/rust-lang/rust/issues/88581> for more information
```

같은 이름이라도 타입마다 상태가 다를 수 있습니다. `u32::div_ceil`은 1.73.0부터 안정이지만 `i32::div_ceil`은 아직 nightly 전용입니다. 그래서 메서드는 "그 타입의 페이지"에서 확인해야 합니다.

### 2.7 메서드 시그니처 읽기

시그니처는 메서드가 무엇을 받고 무엇을 돌려주는지, 호출하는 쪽이 무엇을 준비해야 하는지 알려 주는 약속입니다. 예제 03은 아래 시그니처들을 문서에서 그대로 옮겨 와 주석으로 달고 호출합니다.

**첫 매개변수 `self`의 모양**

| 시그니처 | 뜻 | 호출하는 쪽 |
| --- | --- | --- |
| `pub const fn to_ascii_uppercase(&self) -> char` | 빌려서 읽기만 합니다 | 결과를 새 값으로 받고, 원래 값은 그대로입니다 |
| `pub const fn make_ascii_uppercase(&mut self)` | 그 자리에서 바꿉니다 | 변수가 `let mut`여야 합니다. 반환값이 없습니다 |
| `pub fn map<F, U>(self, f: F) -> [U; N]` | 값을 통째로 받습니다 | `Copy` 타입이면 복사되고, 아니면 소유권이 이동합니다 |
| `pub const fn from_str_radix(src: &str, radix: u32) -> …` | `self`가 없습니다 | 연관 함수이므로 `i32::from_str_radix(..)`로 부릅니다 |

`i32`처럼 `Copy`인 타입은 `self` 메서드를 불러도 원래 변수를 계속 씁니다. `[String; 2]`처럼 `Copy`가 아닌 값에 `self` 메서드(배열의 `map`)를 부르면 그 뒤로 원래 변수를 쓸 수 없습니다(E0382). 메서드는 첫 매개변수가 `self`인 연관 함수이므로 `2_i32.pow(10)`을 `i32::pow(2, 10)`으로 써도 같습니다. 시그니처와 모양이 똑같아서, 무엇이 `self` 자리에 들어가는지 헷갈릴 때 도움이 됩니다.

**제네릭과 `where` 절.** `where` 절은 "이 메서드를 쓰려면 타입이 만족해야 하는 조건"입니다.

```rust
pub fn contains(&self, x: &T) -> bool where T: PartialEq
pub fn fill(&mut self, value: T) where T: Clone
pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[T]>>::Output>
where I: SliceIndex<[T]>
```

- `contains`는 값이 아니라 참조 `&T`를 받습니다(`nums.contains(&30)`). 원소끼리 `==`로 비교해야 하므로 `T: PartialEq`를 요구합니다.
- `fill`은 같은 값을 모든 칸에 넣으려면 여러 번 복제해야 하므로 `T: Clone`을 요구합니다.
- `get`은 복잡해 보이지만 하나씩 읽으면 됩니다. `I`는 "슬라이스의 인덱스로 쓸 수 있는 타입"이고, `<I as SliceIndex<[T]>>::Output`은 "`I`를 슬라이스 인덱스로 볼 때의 연관 타입 `Output`"입니다([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)의 완전 정규화 문법). `usize`를 넘기면 `Output`이 원소 `T`이고, 범위를 넘기면 부분 슬라이스 `[T]`입니다. 그래서 `nums.get(1)`은 `Option<&i32>`, `nums.get(1..3)`은 `Option<&[i32]>`를 돌려줍니다.

**`impl Trait` 매개변수.** `Vec::pop_if`(1.86.0)는 `pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T>`입니다. `impl FnOnce(&mut T) -> bool`은 "`&mut T`를 받아 `bool`을 돌려주는 클로저면 무엇이든"이라는 뜻입니다([Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md)). 표준 라이브러리는 같은 뜻을 이름 있는 타입 매개변수로 더 자주 씁니다. `bool::then`의 `pub fn then<T, F>(self, f: F) -> Option<T> where F: FnOnce() -> T`가 그 모양입니다.

**반환 타입 `Option`과 `Result`.** 반환 타입만 보고도 실패 방식을 알 수 있습니다.

- `Option<T>`: 실패할 수 있지만 이유를 따로 알려 줄 필요가 없을 때입니다. `checked_div(0)`은 `None`, 범위 밖의 `get`도 `None`입니다.
- `Result<T, E>`: 왜 실패했는지 에러 값으로 알려 줄 때입니다. `i32::from_str_radix("1fz", 16)`은 `Err(ParseIntError { kind: InvalidDigit })`입니다.
- 둘 다 아니면 실패하지 않거나, 실패할 때 패닉합니다. 패닉 조건은 §Panics 절에 있습니다.

이름에도 규칙이 있습니다. `checked_*`와 `*_checked`는 `Option`을, `try_*`는 `Result`를 돌려주는 경우가 많고, 같은 일을 하는 접두사 없는 메서드는 패닉할 수 있습니다(`split_at`과 `split_at_checked`). 이 챕터에서는 돌려받은 `Option`을 Part 0에서 배운 `match`, `if let`, `unwrap_or`로만 다룹니다. `map`, `and_then`, `ok_or` 같은 `Option`/`Result`의 콤비네이터(combinator)는 `ch2-option-result`에서 다룹니다.

**라이프타임 `'_`.** 슬라이스의 `pub fn iter(&self) -> Iter<'_, T>`에서 `'_`는 생략된 라이프타임입니다. 돌려받은 반복자가 `self`를 빌리고 있다는 뜻이므로, 반복자를 쓰는 동안 원본을 바꿀 수 없습니다([Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md)의 생략 규칙).

→ [`examples/03_reading_signatures.rs`](examples/03_reading_signatures.rs)

### 2.8 문서 검색

문서 페이지 맨 위에 검색 창이 있습니다. 페이지 아무 곳에서나 `S`나 `/`를 누르면 검색 창으로 가고, `?`를 누르면 단축키와 검색 요령(Search Tricks)이 나옵니다.

**이름 검색.** 이름이나 경로를 입력합니다. rustdoc Book의 설명에 따르면 다음과 같이 동작합니다.

- `::` 대신 공백을 써도 됩니다. `vec new`와 `vec::new`는 모두 `std::vec::Vec::new`를 찾습니다.
- 오타를 어느 정도 허용합니다(`HahsMap`). 정확히 그 이름만 찾으려면 따옴표로 감쌉니다(`"string"`).
- 앞에 종류를 붙여 좁힙니다. `fn:`, `mod:`, `struct:`, `enum:`, `trait:`, `type:`, `macro:`, `constant:`를 쓸 수 있고, `println!`처럼 느낌표를 붙이면 매크로를 찾습니다.
- 결과는 "In Names", "In Parameters", "In Return Types" 탭으로 나뉘고, 검색 창 아래의 "Results in [std]" 선택 상자로 크레이트를 바꿀 수 있습니다.

**타입 시그니처 검색.** 이름을 모르지만 "무엇을 넣으면 무엇이 나오는지"를 알 때 씁니다. 매개변수 타입을 쉼표로 나열하고 `->` 뒤에 반환 타입을 적습니다. rustdoc Book이 드는 예입니다.

| 검색어 | 결과에 나오는 것 |
| --- | --- |
| `usize -> vec` | `slice::repeat`, `Vec::with_capacity` |
| `vec, vec -> bool` | `Vec::eq` |
| `option<T>, fnonce -> option<U>` | `Option::map`, `Option::and_then` |
| `option -> default` | `Option::unwrap_or_default` |

알아 둘 규칙입니다.

- 매개변수 순서는 상관없고, `self`도 보통 매개변수처럼 칩니다.
- 참조, `Box`, `Rc`, `Arc`, `Option`, `Result` 같은 감싸는 타입은 생략해도 찾습니다. 그래서 `char -> u32`라고만 써도 `Option<u32>`를 돌려주는 메서드가 후보에 들어갈 수 있습니다.
- `[u8]`, `[]`처럼 대괄호로 슬라이스와 배열을 찾습니다(`-> [u8]`, `[] -> Option`).
- **`Deref`는 무시합니다.** `Vec`에서 부르는 슬라이스 메서드를 찾으려면 `vec`이 아니라 `[T]`나 `[]`로 검색합니다.

rustdoc Book은 타입 기반 검색이 아직 실험적이고 버그가 있을 수 있다고 밝히고 있습니다. 결과의 순서와 구성은 문서 버전에 따라 달라질 수 있으니, 결과 목록에서 시그니처를 보고 고릅니다.

### 2.9 정수: 연관 상수와 메서드 계열

정수 페이지(`i8`~`i128`, `u8`~`u128`, `isize`, `usize`)는 구성이 같으므로 하나를 익히면 나머지도 읽을 수 있습니다. 연관 상수 `MIN`, `MAX`, `BITS`가 맨 위에 있습니다. `usize::BITS`는 64비트 환경에서 64입니다. `std::i32::MAX` 같은 모듈 상수는 예전 방식이므로 쓰지 않습니다(2.4절).

**오버플로 처리 계열.** Part 0 ch3에서 `*_add` 네 가지를 봤습니다. 같은 계열이 뺄셈, 곱셈, 나눗셈, 나머지, 거듭제곱, 부호 반전, 절댓값, 시프트에도 있고, 문서에서도 `checked_add`, `strict_add`, `checked_add_unsigned`…처럼 계열별로 나란히 나옵니다.

| 계열 | 예 (`3u8`에서 5를 뺌) | 반환 타입 | 오버플로가 나면 |
| --- | --- | --- | --- |
| 검사 연산 `checked_*` | `3u8.checked_sub(5)` | `Option<T>` | `None` |
| 래핑 `wrapping_*` | `3u8.wrapping_sub(5)` | `T` | 한 바퀴 돌아 `254` |
| 포화 연산 `saturating_*` | `3u8.saturating_sub(5)` | `T` | 최솟값/최댓값에서 멈춰 `0` |
| `overflowing_*` | `3u8.overflowing_sub(5)` | `(T, bool)` | `(254, true)` |
| `strict_*` (1.91.0) | `x.strict_add(1)` | `T` | 빌드 모드와 상관없이 항상 패닉 |

`checked_div(0)`은 0으로 나누기도 `None`으로 알려 줍니다. `i32::MIN.checked_abs()`는 `None`입니다. -2147483648의 절댓값은 `i32`의 최댓값 2147483647보다 크기 때문입니다. 이럴 때 `unsigned_abs()`는 결과를 `u32`로 돌려주므로 담을 수 있습니다. `unchecked_*` 계열도 보이는데, `unsafe` 함수라서 오버플로가 나면 미정의 동작입니다. 안전한 코드에서는 쓸 일이 없습니다.

**그 밖의 계열**

| 계열 | 메서드 | 볼 점 |
| --- | --- | --- |
| 수학 | `pow`, `abs`, `signum`, `abs_diff`, `isqrt`, `ilog10`, `midpoint` | `pow`의 지수는 항상 `u32`입니다(4.4절). `abs_diff`는 부호 없는 타입으로 돌려주므로 빼다가 넘칠 걱정이 없습니다. |
| 유클리드 나눗셈 | `rem_euclid`, `div_euclid` | `%`는 나머지의 부호가 왼쪽을 따르지만(`-30 % 24`는 `-6`), `rem_euclid`는 항상 0 이상입니다(`18`). |
| 부호 없는 정수 전용 | `is_power_of_two`, `next_power_of_two`, `is_multiple_of`, `div_ceil` | `i32` 페이지에는 없거나 nightly 전용입니다. |
| 비트 | `count_ones`, `leading_zeros`, `trailing_zeros`, `rotate_left`, `reverse_bits`, `swap_bytes` | `<<`는 밀려난 비트를 버리고, `rotate_left`는 반대쪽으로 돌려 넣습니다. |
| 바이트 | `to_be_bytes`, `to_le_bytes`, `to_ne_bytes`, `from_be_bytes` 등 | be는 빅 엔디언, le는 리틀 엔디언입니다. `ne`(네이티브)는 CPU에 따라 결과가 다릅니다. |
| 문자열 | `from_str_radix` | 2~36진수 문자열을 읽습니다. 10진수는 `parse`(`ch3-strings-fmt`)를 씁니다. |

**`as` 변환은 조용히 잘라 냅니다.** 넓은 타입을 좁은 타입으로 `as` 변환하면 넘치는 위쪽 비트를 버립니다. `300_i32 as u8`은 `44`, `-1_i32 as u8`은 `255`이고 에러도 패닉도 없습니다. 값이 바뀌면 안 되는 변환에는 `u8::try_from(300_i32)`처럼 `Result`를 돌려주는 `try_from`을 씁니다(`Err(TryFromIntError(PosOverflow))`). `From`/`TryFrom` 트레이트 자체는 `ch6-core-traits`에서 다룹니다.

→ [`examples/04_integers.rs`](examples/04_integers.rs)

### 2.10 실수: NaN, 비교, 반올림

**NaN은 자기 자신과도 같지 않습니다.** `(-1.0_f64).sqrt()`나 `0.0 / 0.0`처럼 수학적으로 정해지지 않는 계산은 NaN(Not a Number)이 됩니다. IEEE 754 표준이 "NaN은 어떤 값과도 같지 않다"고 정했기 때문에 `nan == nan`은 `false`이고, NaN인지는 `is_nan()`으로 확인합니다. 이 때문에 `f64`는 전체 순서가 정해지지 않아 `PartialOrd`만 구현하고 `Ord`, `Eq`는 구현하지 않습니다(2.5절의 "없는 것도 정보").

- `partial_cmp`는 비교할 수 없으면 `None`을 돌려줍니다(`1.0.partial_cmp(&nan)`).
- `total_cmp`(`-> Ordering`)는 IEEE 754의 totalOrder에 따라 모든 값에 순서를 정해 줍니다. 순서는 음의 NaN < -inf < 음수 < -0.0 < 0.0 < 양수 < inf < 양의 NaN이고, `==`에서는 같은 `-0.0`과 `0.0`도 구별합니다. 그런데 산술 연산이 돌려주는 NaN의 부호는 정해져 있지 않습니다(`f32` 페이지의 "NaN bit patterns" 절: "The result has a non-deterministic sign"). 그래서 NaN이 맨 앞에 올지 맨 뒤에 올지에 기대는 코드는 쓰지 않습니다.
- `max`, `min`은 한쪽이 NaN이면 다른 쪽을 돌려줍니다.

**근삿값 비교.** `0.1 + 0.2 == 0.3`은 `false`입니다. 차이의 절댓값이 충분히 작은지로 비교합니다. `f64::EPSILON`(약 2.2e-16)은 "1.0과 그다음으로 표현할 수 있는 수의 차이"이므로 1 근처의 값을 비교할 때 기준으로 쓸 수 있고, 값이 크면 허용 오차도 값의 크기에 맞춰 키워야 합니다.

**반올림 계열.** 결과는 모두 `f64`이고, 정수가 필요하면 그 뒤에 `as`로 바꿉니다. 예제 05의 출력입니다.

| x | `floor` | `ceil` | `round` | `round_ties_even` | `trunc` | `fract` |
| --- | --- | --- | --- | --- | --- | --- |
| 2.5 | 2 | 3 | 3 | 2 | 2 | 0.5 |
| -2.5 | -3 | -2 | -3 | -2 | -2 | -0.5 |
| 2.7 | 2 | 3 | 3 | 3 | 2 | 0.7 |

`floor`는 음의 무한대 쪽, `ceil`은 양의 무한대 쪽, `trunc`는 0 쪽으로 자릅니다. `round`는 0.5를 0에서 먼 쪽으로, `round_ties_even`은 짝수 쪽으로 보냅니다.

**수학 상수는 모듈에 있습니다.** `PI`, `E`, `TAU`, `FRAC_PI_2` 등은 원시 타입 페이지가 아니라 `std::f64::consts` 모듈에 있습니다(`use std::f64::consts;` 뒤 `consts::PI`). `3.14159` 같은 근삿값을 직접 쓰면 Clippy의 `approx_constant`가 "approximate value of `f64::consts::PI` found"라며 막습니다.

**실수 → 정수 `as`.** 소수점 아래는 0 쪽으로 버리고(`-3.99 as i32`는 `-3`), 범위를 넘으면 그 타입의 최댓값·최솟값에 붙으며(`1e10 as i32`는 `2147483647`, `-1.0 as u8`은 `0`), NaN은 `0`이 됩니다. 정수끼리의 `as`와 달리 잘라 내지 않고 끝값에 붙는다는 점이 다릅니다.

→ [`examples/05_floats.rs`](examples/05_floats.rs)

### 2.11 `char`와 `bool`

**`char`의 메서드 계열**

| 계열 | 메서드 | 볼 점 |
| --- | --- | --- |
| 분류 | `is_alphabetic`, `is_numeric`, `is_alphanumeric`, `is_whitespace`, `is_uppercase` | 유니코드 전체가 기준입니다. `'가'.is_alphabetic()`은 `true`입니다. |
| ASCII 분류 | `is_ascii`, `is_ascii_alphabetic`, `is_ascii_digit`, `is_ascii_hexdigit` | ASCII 범위만 봅니다. 아라비아-인도 숫자 `'٣'`은 `is_numeric`이지만 `is_ascii_digit`은 아닙니다. |
| 대소문자 | `to_ascii_uppercase`, `to_uppercase`, `to_lowercase` | `to_ascii_uppercase`는 `char`를, `to_uppercase`는 반복자 `ToUppercase`를 돌려줍니다. |
| 숫자 | `to_digit(radix)`, `char::from_digit(num, radix)` | 그 진법의 숫자가 아니면 `None`입니다. |
| 코드 포인트 | `as u32`, `char::from_u32` | `char` → `u32`는 항상 되지만 반대는 `Option`입니다. |
| 인코딩 | `len_utf8`, `len_utf16`, `encode_utf8` | `len_utf8`은 UTF-8로 저장할 때의 바이트 수(1~4)입니다. |

**왜 `to_uppercase`는 반복자를 돌려줄까요?** 유니코드에서는 대문자로 바꾸면 글자 수가 달라지는 문자가 있기 때문입니다. 독일어 `'ß'`는 대문자로 `"SS"` 두 글자가 됩니다. 시그니처 옆의 ⓘ가 이것이 반복자라고 알려 줍니다. 다행히 `ToUppercase`는 `Display`도 구현하므로(`std::char::ToUppercase` 페이지의 Trait Implementations) `println!("{}", c.to_uppercase())`로 바로 출력할 수 있습니다. 한글처럼 대소문자가 없는 문자는 `is_uppercase()`와 `is_lowercase()`가 모두 `false`입니다.

**정수에서 `char`로.** `char`는 유니코드 스칼라 값만 담을 수 있으므로, 서로게이트 영역(0xD800~0xDFFF)이나 0x10FFFF보다 큰 수는 `char`가 될 수 없습니다. 그래서 `char::from_u32`는 `Option<char>`를 돌려주고, `u32 as char`는 컴파일 에러입니다(4.2절). 0~255는 모두 올바른 문자이므로 `u8 as char`만은 허용됩니다. 문자열 안의 `char`를 다루는 일(`chars()`, 검색 패턴)은 `ch3-strings-fmt`에서 다룹니다.

**`bool`의 메서드.** `bool` 페이지의 Implementations에는 메서드가 네 개뿐입니다(1.98.1).

| 메서드 | 시그니처 | 볼 점 |
| --- | --- | --- |
| `then_some` (1.62.0) | `fn then_some<T>(self, t: T) -> Option<T>` | 인자를 먼저 계산합니다(즉시 평가). |
| `then` (1.50.0) | `fn then<T, F>(self, f: F) -> Option<T> where F: FnOnce() -> T` | `true`일 때만 클로저를 부릅니다(게으른 평가). |
| `ok_or` (1.98.0) | `fn ok_or<E>(self, err: E) -> Result<(), E>` | `false`면 `Err(err)`입니다. |
| `ok_or_else` (1.98.0) | `fn ok_or_else<E, F>(self, f: F) -> Result<(), E> where F: FnOnce() -> E` | `ok_or`의 게으른 판입니다. |

`then_some`과 `then`의 차이는 시그니처에 그대로 드러납니다. `then_some`은 값 `t: T`를 받으므로 호출하기 전에 값이 이미 계산되어 있어야 하고, `then`은 클로저 `f: F`를 받으므로 필요할 때만 부를 수 있습니다. 문서도 "함수 호출 결과를 넘긴다면 게으르게 평가하는 `then`을 권한다"고 적고 있습니다. 예제 06에서 `false.then_some(make_value(..))`는 `false`인데도 `make_value`를 부르고, `false.then(|| make_value(..))`는 부르지 않습니다.

그 밖에 `bool`은 `&`, `|`, `^`, `!` 연산자를 씁니다(Trait Implementations의 `BitAnd`, `BitOr`, `BitXor`, `Not`). `&&`와 달리 `&`는 양쪽을 모두 계산합니다. `true as i32`는 `1`입니다.

→ [`examples/06_char_bool.rs`](examples/06_char_bool.rs)

### 2.12 배열, 슬라이스, 튜플

**배열의 메서드 대부분은 슬라이스의 메서드입니다.** 2.5절에서 본 대로 배열은 슬라이스로 바뀌어 슬라이스의 메서드를 부릅니다. 배열 `[T; N]` 자체의 메서드는 `map`(같은 길이의 새 배열), `as_slice`, `each_ref` 등 몇 개뿐입니다. 슬라이스 페이지의 기본 메서드를 계열로 묶으면 이렇습니다.

| 계열 | 메서드 | 볼 점 |
| --- | --- | --- |
| 크기 | `len`, `is_empty` | |
| 꺼내기 | `first`, `last`, `get`, `split_first`, `split_last` | 범위 밖이거나 비어 있으면 패닉 대신 `None`입니다. |
| 나누기 | `split_at`, `split_at_checked` | `split_at`은 `mid > len`이면 패닉, `split_at_checked`(1.80.0)는 `None`입니다. |
| 묶어 보기 | `windows(n)`, `chunks(n)`, `chunks_exact(n)` | `windows`는 한 칸씩 밀며 겹치게, `chunks`는 겹치지 않게 자릅니다. 반복자를 돌려줍니다. |
| 찾기 | `contains`, `starts_with`, `ends_with` | 참조를 받고, `T: PartialEq`가 필요합니다. |
| 제자리 수정 | `swap`, `reverse`, `fill`, `rotate_left` | `&mut self`이므로 `mut` 변수에서만 부릅니다. `cards[1..4].fill(0)`처럼 부분 슬라이스에도 씁니다. |
| 반복 | `iter`, `iter_mut` | `iter().rev()`로 거꾸로 돌고, `iter().sum()`으로 더합니다([Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md)). |

슬라이스의 정렬·이진 탐색·`dedup`·`retain`과 `Vec`의 용량 관리는 `ch4-collections`에서, `concat`/`join`은 `ch3-strings-fmt`에서, 반복자 어댑터 전반은 `ch5-iterators`에서 다룹니다.

배열과 슬라이스는 원소가 비교 가능하면 `==`, `<`로 비교할 수 있고, 앞에서부터 처음 다른 원소로 결과가 정해집니다(`[1, 2, 9] < [1, 3, 0]`은 `true`). 원소가 `Copy`면 배열도 `Copy`라서 대입하면 통째로 복사됩니다.

→ [`examples/07_arrays_slices.rs`](examples/07_arrays_slices.rs)

**튜플에는 고유 메서드가 없습니다.** [`tuple`](https://doc.rust-lang.org/std/primitive.tuple.html) 페이지에는 Implementations 절이 없고 Trait Implementations만 있습니다. 튜플로 하는 일은 `.0` 같은 필드 접근, 패턴 분해, 그리고 트레이트가 주는 기능입니다. 문서의 "Trait implementations" 설명에서 두 가지를 읽을 수 있습니다.

- `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Debug`, `Default`, `Hash`, `From<[T; N]>`은 **원소 12개 이하**의 튜플에만 구현되어 있습니다. 문서는 이것을 "타입 시스템의 일시적인 제약" 때문이라고 설명합니다. 원소가 13개인 튜플을 `{:?}`로 출력하면 ``error[E0277]: `(…)` doesn't implement `Debug` ``가 납니다. 이렇게 큰 묶음은 구조체([Part 0 ch5](../../part0-getting-started/ch5-structs/README.md))로 만드는 편이 낫습니다. `Clone`, `Copy`, `Send`, `Sync` 등은 컴파일러가 만들어 주므로 길이 제한이 없습니다.
- 비교는 **사전식(lexicographic)** 입니다. 앞 원소부터 비교하다가 처음 다른 원소에서 결과가 정해집니다. `(1, "z") < (2, "a")`는 첫 원소에서, `(2, "a") < (2, "b")`는 둘째 원소에서 결정됩니다. 그래서 `(1, 98, 1)` 같은 버전 번호를 그대로 비교할 수 있습니다.

같은 타입으로 된 튜플과 배열은 `into()`로 서로 바꿀 수 있습니다(원소 1~12개). 유닛 `()`는 값이 하나뿐인 타입으로, 유닛 페이지에도 고유 메서드는 없고 `Debug`, `PartialEq`, `Default` 같은 트레이트 구현만 있습니다. `Result<(), E>`처럼 "성공했지만 돌려줄 값이 없음"을 나타낼 때 자주 봅니다.

→ [`examples/08_tuples_unit.rs`](examples/08_tuples_unit.rs)

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_std_layers.rs`](examples/01_std_layers.rs) | 같은 타입의 여러 경로, `type_name`으로 본 정의 위치, 층마다 다른 메서드 | 1 |
| [`examples/02_prelude.rs`](examples/02_prelude.rs) | 프렐류드의 타입·트레이트·함수·매크로, 가져와야 하는 항목 | 2 |
| [`examples/03_reading_signatures.rs`](examples/03_reading_signatures.rs) | `self`의 모양, `where`, `impl Trait`, `Option`/`Result` 반환, Methods from Deref, `const fn` | 3, 4 |
| [`examples/04_integers.rs`](examples/04_integers.rs) | 연관 상수, 오버플로 처리 계열, 수학·비트·바이트·진법 메서드, `as`의 잘림 | 5 |
| [`examples/05_floats.rs`](examples/05_floats.rs) | NaN과 비교, `total_cmp`, 근삿값 비교, 반올림 계열, `consts`, 실수 → 정수 `as` | 5 |
| [`examples/06_char_bool.rs`](examples/06_char_bool.rs) | `char`의 분류·대소문자·숫자·코드 포인트·UTF-8 길이, `bool`의 `then_some`/`then`/`ok_or` | 4, 5 |
| [`examples/07_arrays_slices.rs`](examples/07_arrays_slices.rs) | 배열 → 슬라이스, 꺼내기·나누기·묶어 보기·찾기·제자리 수정 | 5 |
| [`examples/08_tuples_unit.rs`](examples/08_tuples_unit.rs) | 튜플의 사전식 비교, `Default`, 배열과의 변환, 12개 제한, 유닛 | 3, 5 |
| [`compile_fail/`](compile_fail) | 원시 타입과 프렐류드에서 자주 만나는 컴파일 에러 | 2, 4, 5 |

예제 명령은 저장소 루트에서 실행한다고 보고 썼습니다. 모든 예제는 패닉 없이 끝납니다(종료 코드 0).

### 01_std_layers — `core`, `alloc`, `std`

```console
$ cargo run -q -p p1-ch1-std-overview --example 01_std_layers
describe(from_core) = 값 있음
core Option == std Option ? true
alloc Vec == std Vec ? true
core::option::Option<i32>
alloc::vec::Vec<i32>
alloc::string::String
std::collections::hash::map::HashMap<alloc::string::String, i32>
2_i32.pow(10) = 1024, (-2.7_f64).abs() = 2.7
2.0_f64.sqrt() = 1.4142135623730951
sort = [1, 2, 3], sort_unstable = [1, 2, 3]
지금 스레드 이름: Some("main")
```

볼 점:

- `core` 경로로 만든 값을 `std` 경로로 적은 매개변수에 그대로 넘깁니다. 같은 타입이 아니었다면 E0308입니다.
- `type_name`의 출력은 rustc 1.98.1 기준입니다. 형식이 보장되지 않으므로 버전에 따라 다르게 보일 수 있습니다.
- `HashMap`이 `std`에 정의된 것은 기본 해시 함수의 시드를 실행 환경(운영체제)이 주는 난수로 만들기 때문입니다(`HashMap` 문서의 HashDoS 설명).
- 파일 맨 위의 `extern crate alloc;`을 지우면 `alloc::vec::Vec`에서 E0433이 납니다.

### 02_prelude — 프렐류드에 있는 것과 없는 것

```console
$ cargo run -q -p p1-ch1-std-overview --example 02_prelude
Some(3) Err("숫자가 아님") 7 []
42 42 6 0 true
Ok(200) Err(TryFromIntError(PosOverflow))
size_of::<u64>() = 8
swap 뒤: a = 2, b = 1
[1, 2, 3]
Some(7)
3 < 5
{1, 2, 3}
```

볼 점:

- 파일 위쪽의 `use`는 세 줄(`Ordering`, `HashMap`, `mem`)뿐입니다. 나머지는 모두 프렐류드 덕분에 바로 씁니다.
- `size_of`와 `drop`은 프렐류드에 있지만 같은 `std::mem` 모듈의 `swap`은 `mem::swap`으로 부릅니다.
- `std::collections::BTreeSet::from(..)`처럼 한 번만 쓰는 항목은 `use` 없이 전체 경로로 써도 됩니다.

### 03_reading_signatures — 시그니처 그대로 호출하기

```console
$ cargo run -q -p p1-ch1-std-overview --example 03_reading_signatures
c = r, upper = R
d = R
n.abs() = 5, n = -5
lengths = [6, 5]
1024 1024
contains(&30) = true
Some(20) Some([20, 30]) None
grid = [7, 7, 7, 7, 7]
popped = Some(4), stack = [1, 2, 3]
msg = Some("원소 3개")
checked_div(0) = None
에러: invalid digit found in string
first = Some(3), last = Some(5)
v.contains(&4) = true
s.len() = 5
FERRIS
KIB = 1024
```

볼 점:

- 각 호출 위에 문서의 시그니처를 주석으로 옮겨 두었습니다. 문서를 함께 열어 같은 시그니처를 찾아보세요.
- `nums.get(1)`과 `nums.get(1..3)`은 같은 메서드인데 돌려주는 타입이 `Option<&i32>`와 `Option<&[i32]>`로 다릅니다(2.7절의 `SliceIndex`).
- 주석 처리된 `println!("{names:?}")`를 풀면 E0382입니다. 배열의 `map`이 `self`를 받아 `names`를 가져갔기 때문입니다.
- `name.to_uppercase()`는 `String`이 아니라 `str`의 메서드입니다. `String` 페이지의 Methods from Deref<Target = str> 절에서 찾을 수 있습니다.

### 04_integers — 정수의 연관 상수와 메서드 계열

```console
$ cargo run -q -p p1-ch1-std-overview --example 04_integers
u8: 0 ~ 255
i64::MAX = 9223372036854775807
u128::MAX = 340282366920938463463374607431768211455
i32::BITS = 32, usize::BITS = 64
checked_sub     = None
wrapping_sub    = 254
saturating_sub  = 0
overflowing_sub = (254, true)
None
2147483648
None None
2147483647
합계가 u32 범위를 넘었습니다
total = 4294967295
0
pow = 81, signum = -1
abs_diff = 247, 8
-30 % 24 = -6
(-30).rem_euclid(24) = 18
(-30).div_euclid(24) = -2
isqrt(99) = 9
ilog10(12345) = 4
midpoint = 254
true 4
flags = 10110000
count_ones = 3, leading_zeros = 0, trailing_zeros = 4
rotate_left(2) = 11000010
<< 2           = 11000000
reverse_bits   = 00001101
to_be_bytes = [12, 34, 56, 78]
to_le_bytes = [78, 56, 34, 12]
from_le_bytes = 0x12345678
Ok(255)
Ok(10)
Err(ParseIntError { kind: InvalidDigit })
300 as u8 = 44
-1 as u8 = 255
3_000_000_000 as i32 = -1294967296
Ok(44) Err(TryFromIntError(PosOverflow))
```

볼 점:

- `usize::BITS = 64`는 64비트 환경 기준입니다. 32비트 환경에서는 32입니다.
- `checked_mul`이 돌려준 `Option`을 `match`로 꺼내 `None`일 때 대체값을 정합니다. `unwrap_or(0)`도 같은 일을 짧게 합니다.
- `0시에서 30시간 전`처럼 음수의 나머지가 필요할 때 `%`와 `rem_euclid`의 차이가 드러납니다.
- `{:08b}`, `{:02x?}`, `{:#x}`는 각각 2진수 8자리, 16진수 디버그 형식, `0x`가 붙은 16진수로 출력하는 서식입니다. 서식 문법은 `ch3-strings-fmt`에서 다룹니다.

### 05_floats — NaN, 비교, 반올림

```console
$ cargo run -q -p p1-ch1-std-overview --example 05_floats
f64::MAX = 1.7976931348623157e308
f64::EPSILON = 2.220446049250313e-16
inf -inf
PI = 3.141592653589793, TAU = 6.283185307179586
nan = NaN, nan == nan ? false
nan.is_nan() = true
Some(Less) None
Less Less true
max = 3, min = 2
0.1 + 0.2 == 0.3 ? false
차이 < EPSILON ? true
 2.5: floor  2 ceil  3 round  3 round_ties_even  2 trunc  2 fract 0.5
-2.5: floor -3 ceil -2 round -3 round_ties_even -2 trunc -2 fract -0.5
 2.7: floor  2 ceil  3 round  3 round_ties_even  3 trunc  2 fract 0.7
sqrt = 1.4142135623730951, powi = 1024, powf = 2
hypot = 5, abs = 1.5
clamp = 1
sin(PI/2) = 1
3 -3 2147483647
0 0
```

볼 점:

- `nan == nan`은 Clippy가 `eq_op`로 막는 코드입니다. 일부러 보여 주는 비교라서 `#[expect(clippy::eq_op, reason = "…")]`를 붙였습니다.
- `let sum: f64 = 0.1 + 0.2;`에서 `: f64`를 지우면 `.abs()`를 부르는 줄에서 E0689가 납니다(4.1절).
- `Less Less true` 줄은 `total_cmp`가 `-0.0`을 `0.0`보다 작다고 보지만 `==`는 둘을 같다고 본다는 것을 보여 줍니다.
- `{:e}`는 지수 표기, `{:>2}`는 오른쪽 정렬 서식입니다(`ch3-strings-fmt`).

### 06_char_bool — `char`와 `bool`의 메서드

```console
$ cargo run -q -p p1-ch1-std-overview --example 06_char_bool
'a': alphabetic true  ascii_alphabetic true  numeric false ascii_digit false whitespace false
'Z': alphabetic true  ascii_alphabetic true  numeric false ascii_digit false whitespace false
'가': alphabetic true  ascii_alphabetic false numeric false ascii_digit false whitespace false
'7': alphabetic false ascii_alphabetic false numeric true  ascii_digit true  whitespace false
'٣': alphabetic false ascii_alphabetic false numeric true  ascii_digit false whitespace false
' ': alphabetic false ascii_alphabetic false numeric false ascii_digit false whitespace true
Q ß
Q SS
'ß'.to_uppercase()가 내는 글자 수 = 2
false
Some(7) Some(15) None
Some('b') None
'가' as u32 = 44032
Some('각')
None
A
a → 1바이트
é → 2바이트
가 → 3바이트
🦀 → 4바이트
char::MAX = '\u{10ffff}'
a = None, b = None, c = Some(200), 호출 횟수 = 2
half(10) = 5
half(7) = None
Err("미성년자")
1 0
```

볼 점:

- `'٣'` 줄은 터미널에 따라 글자가 오른쪽에서 왼쪽으로 표시되어 줄 모양이 흐트러져 보일 수 있습니다.
- `'가'`(U+AC00, 44032) 다음 코드 포인트가 `'각'`입니다. 한글 음절은 유니코드에서 초성·중성·종성 순서로 차례대로 배치되어 있습니다.
- 호출 횟수가 2인 것은 `then_some` 한 번과 `true.then` 한 번입니다. `false.then`은 클로저를 부르지 않았습니다.
- `ok_or`는 1.98.0에 추가된 메서드라서 그보다 오래된 툴체인에서는 E0599로 컴파일되지 않습니다.

### 07_arrays_slices — 배열과 슬라이스의 기본 메서드

```console
$ cargo run -q -p p1-ch1-std-overview --example 07_arrays_slices
doubled = [42, 46, 38, 50, 56, 44, 40]
len = 7
Some(21) Some(20) None
Some(25) None
head = 21, rest = [23, 19, 25, 28, 22, 20]
평일 = [21, 23, 19, 25, 28], 주말 = [22, 20]
None
19 → 25: 크게 올랐습니다
chunk = [21, 23, 19]
chunk = [25, 28, 22]
chunk = [20]
contains = true
starts_with = true
ends_with = true
swap(0, 4) → [5, 2, 3, 4, 1]
reverse   → [1, 4, 3, 2, 5]
fill      → [1, 0, 0, 0, 5]
countdown = [3, 2, 1]
평균 = 22.6
false true
original = [0, 0, 0], copy = [9, 0, 0]
```

볼 점:

- `windows(2)`의 각 창 `w`는 길이 2인 슬라이스라서 `w[0]`, `w[1]`로 인접한 두 날을 비교합니다.
- `chunks(3)`의 마지막 조각은 원소가 하나뿐입니다. 길이가 꼭 맞는 조각만 돌고 남은 원소는 따로 다루려면 `chunks_exact`를 씁니다. 남은 원소는 `chunks_exact`가 돌려준 반복자의 `remainder()`로 얻습니다.
- `cards[1..4].fill(0)`처럼 `&mut self` 메서드를 부분 슬라이스에 바로 쓸 수 있습니다.

### 08_tuples_unit — 튜플과 유닛

```console
$ cargo run -q -p p1-ch1-std-overview --example 08_tuples_unit
x = 3, y = -2, point.0 = 3
a = 2, b = 1
true
true
Greater
(1, 98, 1)
(0, false, "")
[255, 128, 0] (10, 20, 30)
(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
Ok(()) Err("이름이 비어 있음")
```

볼 점:

- `current.cmp(&required)`와 `current.max(required)`는 튜플의 `Ord` 구현이 주는 메서드입니다. `Ord`가 프렐류드에 있으므로 바로 부릅니다.
- 주석 처리된 13개짜리 튜플의 두 줄을 풀면 E0277이 납니다(2.12절).
- `Ok(())`의 `()`가 유닛입니다. 유닛이 `Debug`를 구현하므로 출력할 수 있습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch1-std-overview
rustc --edition 2024 compile_fail/e0689_ambiguous_float_method.rs --out-dir /tmp/rust-p1-ch1
```

에러 메시지 안의 `/rustc/48a229ce…/library/…` 경로는 표준 라이브러리 소스의 위치이며, 해시 값은 툴체인마다 다릅니다.

### 4.1 타입이 정해지지 않은 숫자에 메서드 호출 — E0689

파일: [`compile_fail/e0689_ambiguous_float_method.rs`](compile_fail/e0689_ambiguous_float_method.rs)

```text
error[E0689]: can't call method `sqrt` on ambiguous numeric type `{float}`
  --> compile_fail/e0689_ambiguous_float_method.rs:11:20
   |
11 |     let root = 2.0.sqrt();
   |                    ^^^^
   |
help: you must specify a concrete type for this numeric value, like `f32`
   |
11 |     let root = 2.0_f32.sqrt();
   |                   ++++

error[E0689]: can't call method `powi` on ambiguous numeric type `{float}`
  --> compile_fail/e0689_ambiguous_float_method.rs:15:22
   |
15 |     println!("{}", x.powi(2));
   |                      ^^^^
   |
help: you must specify a type for this binding, like `f32`
   |
14 |     let x: f32 = 2.0;
   |          +++++
```

- **읽는 법**: `{float}`는 "실수 리터럴인데 아직 `f32`인지 `f64`인지 정해지지 않은 타입"이라는 뜻입니다. 정수 리터럴이면 `{integer}`로 나옵니다.
- **원인**: 메서드는 타입마다 따로 정의되어 있으므로(`f32::sqrt`와 `f64::sqrt`), 메서드를 부르는 시점에 타입이 정해져 있어야 어느 쪽을 부를지 알 수 있습니다. 리터럴은 끝까지 단서가 없으면 `f64`가 되지만, 그 결정은 메서드 호출을 해석하는 시점보다 늦습니다. 예제 05를 처음 작성할 때도 `let sum = 0.1 + 0.2;` 뒤에 `(sum - 0.3).abs()`를 불러 이 에러가 났습니다.
- **해결**: 리터럴에 접미사를 붙이거나(`2.0_f64.sqrt()`) 변수에 타입을 적습니다(`let x: f64 = 2.0;`). 컴파일러는 `f32`를 예로 들지만 특별한 이유가 없으면 `f64`를 씁니다.

### 4.2 `u32`를 `as`로 `char`로 변환 — E0604

파일: [`compile_fail/e0604_u32_as_char.rs`](compile_fail/e0604_u32_as_char.rs)

```text
error[E0604]: only `u8` can be cast as `char`, not `u32`
  --> compile_fail/e0604_u32_as_char.rs:12:13
   |
12 |     let c = code as char;
   |             ^^^^^^^^^^^^ invalid cast
   |
help: consider using `char::from_u32` instead
   |
12 -     let c = code as char;
12 +     let c = char::from_u32(code);
   |
```

- **읽는 법**: "`u8`만 `char`로 캐스트할 수 있다"는 뜻이고, `help:`가 대신 쓸 함수를 알려 줍니다.
- **원인**: `as`는 실패할 수 없는 변환입니다. `u32`의 값 중에는 `char`가 될 수 없는 것(0xD800~0xDFFF, 0x10FFFF 초과)이 있으므로 허용하지 않습니다. `u8`의 0~255는 모두 올바른 문자라서 허용됩니다.
- **해결**: `char::from_u32(code)`를 쓰고 돌려받은 `Option<char>`를 처리합니다. `help:`의 제안대로만 바꾸면 `c`가 `Option<char>`가 되므로 `println!("{c}")`에서 다시 에러가 납니다. `match`나 `if let Some(c) = char::from_u32(code)`로 꺼냅니다(예제 06의 4번 구역).

### 4.3 슬라이스를 `i32`로 인덱싱 — E0277

파일: [`compile_fail/e0277_slice_index_i32.rs`](compile_fail/e0277_slice_index_i32.rs)

```text
error[E0277]: the type `[{integer}]` cannot be indexed by `i32`
  --> compile_fail/e0277_slice_index_i32.rs:14:27
   |
14 |     println!("{}", scores[i]);
   |                           ^ slice indices are of type `usize` or ranges of `usize`
   |
   = help: the trait `SliceIndex<[{integer}]>` is not implemented for `i32`
help: `usize` implements trait `SliceIndex<T>`
   ...
   = note: required for `[{integer}]` to implement `Index<i32>`
   = note: 1 redundant requirement hidden
   = note: required for `[{integer}; 3]` to implement `Index<i32>`
```

- **읽는 법**: 밑줄 옆의 설명 "slice indices are of type `usize` or ranges of `usize`"가 핵심입니다. `help:`의 `SliceIndex`는 2.7절에서 `get`의 시그니처로 읽은 그 트레이트입니다. `note:`는 배열 인덱싱이 슬라이스 인덱싱으로 이어진다는 것을 거슬러 올라가며 보여 줍니다. (중간의 소스 위치 줄은 줄였습니다.)
- **원인**: 인덱스로 쓸 수 있는 타입은 `usize`와 `usize`의 범위뿐입니다. `i32`는 음수일 수 있으므로 Rust는 인덱스로 쓰기 전에 직접 변환하게 합니다. 이것은 [Part 0 ch8](../../part0-getting-started/ch8-collections/README.md)의 "문자열을 정수로 인덱싱"과는 다른 상황입니다. 그쪽은 인덱스 타입이 아니라 `String`이 정수 인덱싱을 아예 지원하지 않는 것이었습니다.
- **해결**: 인덱스 변수를 처음부터 `usize`로 선언합니다. 다른 계산 때문에 `i32`여야 한다면 `usize::try_from(i)`로 바꾸고 음수인 경우를 처리합니다. 음수가 아님을 확신할 때만 `i as usize`를 씁니다. 음수를 `as usize`로 바꾸면 아주 큰 수가 되어(2.9절의 잘림) 결국 범위 밖 패닉이 납니다.

### 4.4 `pow`의 지수에 `usize`를 넘김 — E0308

파일: [`compile_fail/e0308_pow_exponent_usize.rs`](compile_fail/e0308_pow_exponent_usize.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_pow_exponent_usize.rs:14:28
   |
14 |     let scale = 10_i32.pow(digits.len());
   |                        --- ^^^^^^^^^^^^ expected `u32`, found `usize`
   |                        |
   |                        arguments to this method are incorrect
   |
note: method defined here
  --> /rustc/48a229ceaefd4985c50990b14116b6d856af0985/library/core/src/num/int_macros.rs:3136:21
   ...
help: you can convert a `usize` to a `u32` and panic if the converted value doesn't fit
   |
14 |     let scale = 10_i32.pow(digits.len().try_into().unwrap());
   |                                        ++++++++++++++++++++
```

- **읽는 법**: `pow`의 인자로 `u32`가 와야 하는데 `usize`가 왔다는 뜻입니다. `note: method defined here`가 가리키는 경로가 `library/core/…`라는 것도 볼 수 있습니다. `pow`는 `core`에 정의되어 있습니다(2.1절).
- **원인**: 문서의 시그니처는 `pub const fn pow(self, exp: u32) -> i32`입니다. 밑이 어떤 정수 타입이든 지수 `exp`는 항상 `u32`입니다. `len()`은 `usize`를 돌려주고, Rust는 정수 타입 사이를 자동으로 변환하지 않습니다. `rotate_left`, `checked_shl` 같은 비트 메서드도 `u32`를 받으므로 같은 에러를 자주 만납니다.
- **해결**: `help:`처럼 `.try_into().unwrap()`으로 바꾸면 넘칠 때 패닉합니다. 실패를 직접 처리하려면 `u32::try_from(digits.len())`의 `Result`를 `match`로 다룹니다. 길이가 `u32` 범위를 넘지 않는다고 확신하면 `digits.len() as u32`도 됩니다. 어느 쪽이든 "시그니처의 매개변수 타입을 먼저 확인한다"는 습관이 이 에러를 줄입니다.

### 4.5 프렐류드에 없는 함수를 `use` 없이 사용 — E0425

파일: [`compile_fail/e0425_swap_not_in_prelude.rs`](compile_fail/e0425_swap_not_in_prelude.rs)

```text
error[E0425]: cannot find function `swap` in this scope
  --> compile_fail/e0425_swap_not_in_prelude.rs:14:5
   |
14 |     swap(&mut a, &mut b); // 프렐류드에 없으므로 에러입니다.
   |     ^^^^ not found in this scope
   |
help: consider importing one of these functions
   |
10 + use std::mem::swap;
   |
10 + use std::ptr::swap;
   |
```

- **읽는 법**: 이 스코프에서 `swap`이라는 함수를 찾지 못했다는 뜻입니다. 바로 위 줄의 `size_of::<i32>()`는 에러가 아닙니다. `help:`는 같은 이름의 후보 두 개를 보여 줍니다.
- **원인**: `std::mem`에서 프렐류드에 든 것은 `drop`, `size_of`, `size_of_val`, `align_of`, `align_of_val`뿐입니다(2.2절). 같은 모듈의 `swap`, `replace`, `take`는 가져와야 합니다.
- **해결**: `use std::mem;` 뒤 `mem::swap(&mut a, &mut b)`로 부릅니다(예제 02). `use std::mem::swap;`도 되지만, 함수는 부모 모듈까지만 가져와 출처가 보이게 쓰는 것이 관례입니다([Part 0 ch7](../../part0-getting-started/ch7-modules/README.md)). `help:`의 두 후보 중 `std::ptr::swap`은 원시 포인터를 다루는 `unsafe` 함수이므로 고르지 않습니다.

## 5. 연습 문제

### 연습 1. 문서에서 답 찾기

코드를 실행하지 말고 API 문서(로컬 문서 `rustup doc --std` 또는 온라인 문서)만 보고 답하세요.

1. `u16`이 담을 수 있는 가장 큰 값은 얼마인가요?
2. `i64::checked_neg`는 언제 `None`을 돌려주나요?
3. `char::is_ascii_hexdigit`는 몇 버전부터 안정되었고, 몇 버전부터 `const` 문맥에서 쓸 수 있나요?
4. `Vec<T>` 페이지에서 `windows`는 어느 절에 있나요? 왜 그 절에 있나요?
5. `f64`는 `Eq`를 구현하나요? `bool`은 `Ord`를 구현하나요?

<details>
<summary>힌트</summary>

- 1은 `u16` 원시 타입 페이지의 Associated Constants, 2는 `i64` 페이지에서 `checked_neg`를 찾아 설명 첫 줄을 읽습니다. `checked_neg`처럼 이름을 알면 검색 창에 `i64 checked_neg`라고 입력하는 것이 빠릅니다.
- 3은 메서드 제목 오른쪽의 `1.x.0 (const: 1.y.0)` 표시를 읽습니다(2.6절).
- 4는 사이드바의 절 목록에서 찾습니다. `Vec`이 어떤 `Deref`를 구현하는지 떠올려 보세요(2.5절).
- 5는 각 타입 페이지의 Trait Implementations에서 `impl Eq for …`, `impl Ord for …`가 있는지 봅니다. 없다면 왜 없는지 2.10절에서 확인합니다.
- 1.98.1 문서 기준 답: 65535 / `self == MIN`일 때 / 1.24.0, const 1.47.0 / Methods from Deref<Target = [T]> / `f64`는 `Eq` 없음, `bool`은 `Ord` 있음.

</details>

### 연습 2. 넘치지 않는 퍼센트 계산

`fn percent(part: u32, total: u32) -> Option<u32>`를 작성하세요. `part * 100 / total`을 계산하되, 곱셈이 `u32` 범위를 넘거나 `total`이 0이면 패닉하지 말고 `None`을 돌려줍니다. `percent(1, 4)`는 `Some(25)`, `percent(1, 0)`과 `percent(u32::MAX, 1)`은 `None`이어야 합니다.

<details>
<summary>힌트</summary>

- 정수 페이지에서 곱셈과 나눗셈의 검사 연산을 찾습니다. 둘 다 `Option`을 돌려줍니다.
- 첫 번째 결과를 `match`로 꺼내 `Some(scaled)`일 때 `scaled.checked_div(total)`을 돌려주고, `None`일 때 `None`을 돌려줍니다. `checked_div`는 0으로 나누면 `None`이므로 `total == 0`을 따로 검사할 필요가 없습니다.
- 두 `Option`을 이어 붙이는 더 짧은 방법(`and_then`)은 `ch2-option-result`에서 배웁니다.

</details>

### 연습 3. 색상 코드 읽기

`fn parse_color(hex: &str) -> Option<(u8, u8, u8)>`를 작성하세요. `"#ff8000"`은 `Some((255, 128, 0))`, `"#FFFFFF"`는 `Some((255, 255, 255))`이고, `#`으로 시작하지 않거나(`"ff8000"`) 16진수가 아닌 글자가 있으면(`"#ggg000"`) `None`입니다.

<details>
<summary>힌트</summary>

- 먼저 길이가 7이고 첫 바이트가 `b'#'`인지 확인합니다. `hex.len()`과 `hex.as_bytes()[0]`을 쓸 수 있습니다. 조건이 맞지 않으면 `return None;`.
- `&hex[1..]`(Part 0 ch4의 문자열 슬라이스)을 `u32::from_str_radix(.., 16)`으로 읽고, `Result`를 `match`로 처리합니다.
- 읽은 `u32`에 `to_be_bytes()`를 부르면 `[0, r, g, b]` 모양의 배열이 나옵니다. `let [_, r, g, b] = n.to_be_bytes();`처럼 배열 패턴으로 풀 수 있습니다.

</details>

### 연습 4. 2주 기온 분석

`let temps = [21, 23, 19, 25, 28, 22, 20, 18, 17, 24, 26, 27, 21, 19];`(14일치)에서 (1) 주별 평균 기온을 소수 첫째 자리까지, (2) 전날보다 가장 많이 오른 날이 며칠째이고 몇 도 올랐는지를 출력하세요. 기대 출력은 `1주차 평균 22.6`, `2주차 평균 21.7`, `가장 많이 오른 날: 10일째 (+7)`입니다.

<details>
<summary>힌트</summary>

- (1)은 `chunks(7)`로 일주일씩 자릅니다. 각 조각의 합은 `week.iter().sum::<i32>()`, 평균은 `as f64`로 바꿔 나눕니다. 몇 번째 주인지 필요하면 `chunks(7).enumerate()`를 씁니다.
- (2)는 `windows(2)`로 인접한 두 날을 봅니다. 창 `w`에서 `w[1] - w[0]`이 지금까지의 최댓값보다 크면 기록합니다. `windows(2).enumerate()`의 인덱스 `i`는 창의 첫 날(0부터)이므로, 오른 날은 `i + 2`일째입니다.

</details>

## 6. 참고 자료

- 기준 문서
  - [std 크레이트 첫 페이지](https://doc.rust-lang.org/std/index.html): "How to read this documentation"과 "What is in the standard library documentation?"에 원시 타입 페이지와 같은 이름 모듈의 차이, `Vec`/`String`의 메서드가 `[T]`/`str`에 있다는 안내가 있습니다(2.4~2.5절).
  - [`std::prelude`](https://doc.rust-lang.org/std/prelude/index.html): 프렐류드의 목적과 에디션별 구성입니다(2.2절의 표). 실제 목록은 하위 모듈 [`rust_2024`](https://doc.rust-lang.org/std/prelude/rust_2024/index.html)와 [`v1`](https://doc.rust-lang.org/std/prelude/v1/index.html)의 Re-exports에 있습니다.
  - [`core`](https://doc.rust-lang.org/core/index.html), [`alloc`](https://doc.rust-lang.org/alloc/index.html) 크레이트 문서: 각 크레이트가 무엇을 전제하는지 첫 단락에서 설명합니다(2.1절).
  - 원시 타입 페이지: [`i32`](https://doc.rust-lang.org/std/primitive.i32.html), [`f64`](https://doc.rust-lang.org/std/primitive.f64.html), [`char`](https://doc.rust-lang.org/std/primitive.char.html), [`bool`](https://doc.rust-lang.org/std/primitive.bool.html), [`array`](https://doc.rust-lang.org/std/primitive.array.html), [`slice`](https://doc.rust-lang.org/std/primitive.slice.html), [`tuple`](https://doc.rust-lang.org/std/primitive.tuple.html), [`unit`](https://doc.rust-lang.org/std/primitive.unit.html). 같은 이름의 모듈 [`std::i32`](https://doc.rust-lang.org/std/i32/index.html), [`std::char`](https://doc.rust-lang.org/std/char/index.html), [`std::f64::consts`](https://doc.rust-lang.org/std/f64/consts/index.html)와 비교해 보세요.
  - [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html#deref-methods-%5BT%5D), [`String`](https://doc.rust-lang.org/std/string/struct.String.html#deref-methods-str): 링크가 Methods from Deref 절로 바로 이동합니다.
  - [`std::any::type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html): 반환 문자열이 보장되지 않는다는 주의 사항이 있습니다(예제 01).
- rustdoc Book
  - [How to read rustdoc output](https://doc.rust-lang.org/rustdoc/how-to-read-rustdoc.html): 문서 화면의 구성(항목 문서, 사이드바, 검색 창, 단축키)을 설명합니다.
  - [Rustdoc search](https://doc.rust-lang.org/rustdoc/read-documentation/search.html): 이름 검색, 타입 시그니처 검색, 생략할 수 있는 감싸는 타입, 종류 필터를 자세히 설명합니다(2.8절의 표가 여기서 나왔습니다).
- The Rust Reference
  - [Preludes](https://doc.rust-lang.org/reference/names/preludes.html): 에디션별로 어떤 프렐류드 모듈이 쓰이는지, `core`와 `std`는 외부 프렐류드에 들어 있지만 `alloc`은 `extern crate`가 필요하다는 규칙이 있습니다(2.1~2.2절).
- Rust By Example
  - [Primitives](https://doc.rust-lang.org/rust-by-example/primitives.html): 원시 타입 목록과 타입 표기·접미사·추론을 정리합니다. 하위 절 [Literals and operators](https://doc.rust-lang.org/rust-by-example/primitives/literals.html), [Tuples](https://doc.rust-lang.org/rust-by-example/primitives/tuples.html), [Arrays and Slices](https://doc.rust-lang.org/rust-by-example/primitives/array.html)는 Part 0 ch3의 복습이면서, Tuples 절의 긴 튜플 출력 예제는 2.12절의 12개 제한과 이어집니다.
- Rustlings
  - [`04_primitive_types`](https://github.com/rust-lang/rustlings/tree/main/exercises/04_primitive_types): `bool`, `char`(`is_alphabetic`, `is_numeric`), 배열, 슬라이스, 튜플 분해와 인덱싱을 연습합니다. Part 0 수준의 복습입니다.
