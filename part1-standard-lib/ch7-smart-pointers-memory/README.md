# ch7. 스마트 포인터와 메모리: 크기를 재고, 값을 옮기고, 내부 가변성을 고르기

[Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)에서 `Box<T>`로 재귀 타입을 만들고, `Deref`/`Drop`을 구현하고, `Rc<T>`와 `RefCell<T>`로 여러 소유자가 함께 바꾸는 값을 만들고, `Weak<T>`로 순환 참조를 끊었습니다. 이 챕터는 그 위에서 세 방향으로 넓힙니다.

- **재기**: 값이 메모리에서 몇 바이트를 차지하는지 `std::mem`의 함수로 직접 잽니다. Part 0 ch14가 "16바이트"라고만 말하고 넘어간 cons 리스트의 크기, [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)의 `Cell` 열거형이 `String`과 똑같이 24바이트인 이유를 여기서 설명합니다.
- **옮기기**: `&mut`로 빌린 자리에서 값을 꺼내는 `std::mem::swap`/`replace`/`take`를 배웁니다. [Part 1 ch2](../ch2-option-result/README.md)의 `Option::take`와 [Part 1 ch4](../ch4-collections/README.md)에서 미뤄 둔 "벡터를 통째로 꺼내고 빈 벡터 남기기"가 이것으로 풀립니다.
- **고르기**: `Box`와 `Rc`의 덜 알려진 메서드, 그리고 Part 0 ch14가 이름만 남긴 `Cell<T>`, `OnceCell<T>`, `LazyCell<T, F>`를 `RefCell<T>`과 나란히 놓고 "이럴 때 이것"을 정합니다.

이 챕터의 예제는 `unsafe` 코드를 쓰지 않습니다. 원시 메모리를 직접 다루는 함수는 이름과 다룰 챕터만 알려 줍니다.

- 패키지: `p1-ch7-smart-pointers-memory`
- 기준 문서: [`std::boxed`](https://doc.rust-lang.org/std/boxed/index.html), [`std::rc`](https://doc.rust-lang.org/std/rc/index.html), [`std::cell`](https://doc.rust-lang.org/std/cell/index.html), [`std::mem`](https://doc.rust-lang.org/std/mem/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024). 크기와 메모리 정렬 값은 64비트 환경(aarch64 macOS)에서 잰 것입니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `size_of`, `align_of`, `size_of_val`, `offset_of!`로 타입의 크기와 메모리 정렬, 필드 위치를 재고, 구조체의 크기를 "필드 합 + 패딩"으로 계산해 설명할 수 있다. 문서가 보장하는 크기와 이 툴체인에서 관찰한 결과를 구분할 수 있다.
2. 열거형의 크기를 태그와 니치 최적화로 설명할 수 있다. `Option<Box<T>>`가 `Box<T>`와 같은 8바이트인 이유, 배리언트가 셋인 열거형이 `String`과 같은 24바이트가 되는 이유, `&[T]`·`&str`·`&dyn Trait`가 16바이트인 이유를 설명할 수 있다.
3. `mem::swap`, `mem::replace`, `mem::take`로 `&mut` 뒤의 값을 꺼내거나 열거형 상태를 바꿀 수 있고, 그냥 꺼내면 E0507이 나는 이유를 설명할 수 있다.
4. `Box<[T]>`/`Box<str>`, 큰 배리언트를 `Box`로 감싸기, `Rc::ptr_eq`/`get_mut`/`make_mut`/`try_unwrap`/`into_inner`/`new_cyclic`, `Rc<str>`을 상황에 맞게 쓸 수 있다.
5. `Cell`, `RefCell`, `OnceCell`, `LazyCell`의 차이를 "안의 값을 가리키는 참조를 내주는가, 실행 중 검사가 필요한가"로 설명하고 상황에 맞게 고를 수 있다. `RefCell`의 대여가 생각보다 오래 살아 있는 경우를 알아보고 피할 수 있다.

## 2. 핵심 개념

2.1~2.4절은 크기와 메모리 배치, 2.5~2.6절은 `std::mem`의 함수, 2.7~2.8절은 `Box`와 `Rc`, 2.9~2.12절은 내부 가변성 도구를 다룹니다.

### 2.1 크기와 메모리 정렬

모든 타입에는 두 가지 수가 있습니다.

- **크기(size)**: 그 타입의 값 하나가 차지하는 바이트 수. `size_of::<T>()`로 잽니다.
- **메모리 정렬(alignment)**: 그 타입의 값이 놓일 수 있는 주소의 배수. `align_of::<T>()`로 잽니다. `u32`는 정렬이 4이므로 주소가 4의 배수인 곳에만 놓입니다.

메모리 정렬이 있는 이유는 CPU가 정렬된 주소에서 값을 읽고 쓰는 것이 빠르고, 일부 CPU는 정렬되지 않은 주소에서 아예 읽지 못하기 때문입니다. 컴파일러는 이 규칙을 지키려고 값 사이에 빈칸을 넣습니다.

`size_of`, `size_of_val`, `align_of`, `align_of_val`은 프렐류드에 있어 `use` 없이 씁니다([Part 1 ch1](../ch1-std-overview/README.md) 2.2절). [`01_size_align.rs`](examples/01_size_align.rs)의 1번 구역에서 잰 값입니다.

| 타입 | 크기 | 메모리 정렬 | 보장 여부 |
| --- | --- | --- | --- |
| `u8`, `bool` | 1 | 1 | 크기는 `size_of` 문서의 표가 보장 |
| `u16` | 2 | 2 | 〃 |
| `u32`, `char` | 4 | 4 | 〃 |
| `u64` | 8 | 8 | 〃 |
| `u128` | 16 | 16 | 크기만 보장. 정렬은 플랫폼에 따라 다를 수 있음 |
| `usize`, `&T` | 8 | 8 | 64비트 환경에서. 32비트 환경이면 4 |

`size_of::<T>()`는 타입만 보고 컴파일할 때 정해지는 상수입니다. 반면 **`size_of_val(&값)`** 은 참조가 가리키는 값의 크기를 잽니다. 동적 크기 타입([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.15절)도 잴 수 있어서, 같은 데이터라도 무엇을 재느냐에 따라 값이 다릅니다.

```rust
let scores: Vec<i32> = vec![90, 85, 77];
size_of_val(&scores)      // 24: Vec 핸들(포인터, 용량, 길이)
size_of_val(&scores[..])  // 12: 힙에 있는 원소 3개 (4바이트 × 3)
```

이 챕터에서는 `Vec`과 `String`처럼 힙 데이터를 가리키는 작은 구조체를 **핸들(handle)** 이라고 부릅니다. `size_of::<Vec<T>>()`는 원소가 몇 개든 핸들 크기인 24바이트입니다.

### 2.2 구조체: 패딩과 필드 순서

구조체의 메모리 정렬은 필드 중 가장 큰 메모리 정렬을 따르고, 크기는 그 정렬의 배수로 올림합니다. 필드 사이와 끝에 들어가는 빈칸을 **패딩(padding)** 이라고 합니다. 끝에 패딩을 두는 이유는 배열 때문입니다. `[Packet; 3]`의 원소는 빈틈없이 이어 붙으므로, 각 원소의 크기가 정렬의 배수여야 두 번째, 세 번째 원소도 올바른 주소에 놓입니다.

```rust
struct Packet { tag: u8, id: u32, flag: u8 }    // 필드 합 6바이트
struct Record { kind: u8, timestamp: u64, code: u16 } // 필드 합 11바이트
```

예제에서 잰 결과는 `Packet` 8바이트(정렬 4), `Record` 16바이트(정렬 8)입니다. 그런데 선언 순서대로 놓았다면 `Packet`은 12바이트여야 합니다.

```
선언 순서대로 놓는다면:  tag | 빈칸 3 | id id id id | flag | 빈칸 3   = 12바이트
실제 배치 (offset_of!):  id id id id | tag | flag | 빈칸 2           =  8바이트
```

**`offset_of!(타입, 필드)`** 매크로(`std::mem::offset_of`, 1.77.0부터)로 각 필드가 구조체 시작에서 몇 바이트 떨어져 있는지 재면, 이 툴체인에서는 `id@0, tag@4, flag@5`가 나옵니다. **Rust의 기본 표현(representation)에서는 컴파일러가 필드 순서를 자유롭게 바꿀 수 있고**, 보통 패딩이 줄어드는 쪽으로 바꿉니다. `Record`도 `timestamp@0, code@8, kind@10`으로 바뀌었습니다(8 + 2 + 1 = 11을 8의 배수로 올려 16바이트).

그래서 C 언어처럼 필드 순서를 손으로 맞출 필요가 거의 없습니다. 대신 **기본 표현의 필드 배치는 보장되지 않습니다.** 위 오프셋은 이 툴체인의 결과일 뿐이고 컴파일러 버전이 바뀌면 달라질 수 있으므로, 필드 순서에 기대는 코드를 쓰면 안 됩니다.

C 코드와 구조체를 주고받을 때처럼 배치를 고정해야 할 때는 `#[repr(C)]` 같은 표현 지정 속성을 붙입니다. `Packet`에 `#[repr(C)]`를 붙이면 선언 순서를 지켜 12바이트가 됩니다. 표현 지정과 메모리 배치 규칙은 Part 9 `ch7-type-system`과 Part 10 `ch2-data-layout`에서 다룹니다.

담을 정보가 없는 타입은 **크기가 0**입니다. `()`, 필드 없는 유닛 구조체, `[i32; 0]`이 그렇고, `[Marker; 1000]`처럼 1000개를 모아도 0바이트입니다. 크기가 0인 타입은 메모리를 차지하지 않으므로 "있다/없다"만 나타내는 표시로 비용 없이 쓸 수 있습니다.

→ [`examples/01_size_align.rs`](examples/01_size_align.rs)

### 2.3 열거형: 태그와 니치 최적화

열거형의 값은 한 번에 배리언트 하나만 담으므로, 크기는 **가장 큰 배리언트를 담을 공간 + 지금 어느 배리언트인지 적는 태그(tag)** 입니다(Part 0 ch14 2.3절). 표준 라이브러리는 이 태그 값을 **판별값(discriminant)** 이라고 부릅니다.

| 타입 | 크기 | 계산 |
| --- | --- | --- |
| `enum Direction { North, East, South, West }` | 1 | 데이터 없음 + 태그 1바이트 |
| `enum Shape { Dot, Circle(i32), Rect(i32, i32) }` | 12 | `Rect` 8 + 태그 (`i32` 정렬에 맞춰 4) |
| `Option<u32>` | 8 | `u32` 4 + 태그 (정렬 4) |
| `Option<u64>`, `Option<f64>` | 16 | 8 + 태그 (정렬 8) |

`Option<u32>`에 태그 칸이 따로 필요한 이유는 `u32`의 모든 비트 조합(0~4294967295)이 올바른 `u32` 값이기 때문입니다. `None`을 나타낼 남는 값이 없습니다.

#### 남는 값이 있으면 태그 칸이 필요 없습니다

어떤 타입은 결코 쓰지 않는 비트 조합을 가지고 있습니다. 이런 값을 **니치(niche)** 라고 부르고, 컴파일러가 니치에 배리언트 정보를 적어 태그 칸을 없애는 것을 **니치 최적화(niche optimization)** 라고 합니다.

- **`Box<T>`, `&T`, `&mut T`는 절대 0(null)이 아닙니다.** 그래서 "포인터 칸이 0이면 `None`"으로 정하면 됩니다. `Option<Box<i32>>`와 `Option<&i32>`는 8바이트로 `Box`, `&`와 같습니다. Part 0 ch14의 cons 리스트 `enum List { Cons(i32, Box<List>), Nil }`가 태그 칸 없이 16바이트(`i32` 4 + 빈칸 4 + `Box` 8)인 것도 같은 원리로, `Nil`은 `Box` 칸이 0인 값입니다.
- **`bool`은 0과 1만 씁니다.** 2~255가 니치이므로 `Option<bool>`도, `Option<Option<bool>>`도 1바이트입니다.
- **`char`는 0x10FFFF까지만 씁니다.** `Option<char>`는 4바이트입니다.
- **데이터 없는 열거형**(`Direction`)은 태그가 0~3만 쓰므로 `Option<Direction>`도 1바이트입니다.

이 중 **문서가 보장하는 것은 첫째뿐입니다.** [`std::option` 문서의 Representation 절](https://doc.rust-lang.org/std/option/index.html#representation)은 `Box<U>`, `&U`, `&mut U`, 함수 포인터, `num::NonZero*`, `ptr::NonNull<U>`에 대해 `Option<T>`가 `T`와 크기가 같다고 약속합니다. `bool`, `char` 등은 이 툴체인의 컴파일러가 해 준 최적화이며 보장되지는 않습니다. (`NonZero<u32>`처럼 0이 아님을 보장하는 정수 타입은 `ch14-ffi-any-misc`에서 다룹니다.)

#### `String`을 담은 열거형이 `String`과 같은 크기인 이유

Part 0 ch17의 `enum Cell { Int(i32), Float(f64), Text(String) }`은 24바이트로 `String`과 같았습니다. 배리언트가 셋인데 태그 칸이 없습니다. 예제 02는 같은 모양의 열거형을 `SheetCell`이라는 이름으로 잽니다(이 챕터의 `std::cell::Cell`과 헷갈리지 않도록 이름만 바꿨습니다).

`String`의 속은 `Vec<u8>`이고, `Vec`의 핸들은 (포인터, 용량, 길이) 세 칸입니다. 그런데 `Vec`의 용량은 `isize::MAX`를 넘을 수 없으므로, 용량 칸은 최상위 비트가 1인 값을 결코 갖지 않습니다. 현재 표준 라이브러리는 용량 칸의 타입을 "`0..=isize::MAX` 범위만 쓰는 정수"로 표시해 두었고(1.100.0의 `RawVecInner` 소스에서 확인), 그래서 용량 칸에는 니치가 2⁶³개 넘게 있습니다. 컴파일러는 다음과 같이 배치합니다.

- `Text(String)`: 세 칸을 그대로 씁니다. 용량 칸에는 올바른 용량이 들어 있습니다.
- `Int(i32)`, `Float(f64)`: 용량 칸에 니치 값(예: "용량이 2⁶³이면 `Int`")을 적고, 데이터는 용량 칸과 겹치지 않는 나머지 자리에 둡니다.

이 설명은 같은 예제의 비교 결과로 뒷받침됩니다.

| 타입 | 크기 | 왜 |
| --- | --- | --- |
| `Option<String>` | 24 | 니치 하나면 충분 |
| `SheetCell` (`Int`/`Float`/`Text`) | 24 | 니치 두 개 + 작은 데이터는 다른 칸에 |
| `Option<Option<String>>` | 24 | 니치가 여러 개라 `None` 두 종류를 모두 적을 수 있음 |
| `Option<Option<Box<i32>>>` | 16 | `Box`의 니치는 0 하나뿐이라, 바깥 `Option`은 태그 칸을 따로 둠 |
| `enum TwoTexts { Title(String), Body(String) }` | 32 | 두 배리언트 모두 용량 칸까지 써야 해서 니치를 쓸 수 없음 |

`Option<Option<String>>`이 24바이트라는 것은 니치가 적어도 두 개라는 뜻이므로, 니치가 포인터 칸(0 하나뿐)이 아니라 용량 칸에 있다는 것을 안전한 코드만으로 확인한 셈입니다. 이 배치는 모두 구현 세부이며 보장되지 않습니다.

#### `mem::discriminant`: 배리언트만 비교하기

열거형 두 값이 "같은 배리언트인가"만 알고 싶을 때가 있습니다. `==`는 필드까지 비교하고(그리고 `PartialEq`를 구현해야 쓸 수 있고), `match`는 배리언트 수만큼 갈래를 써야 합니다. **`std::mem::discriminant(&값)`** 은 필드를 무시하고 배리언트만 나타내는 `Discriminant` 값을 돌려주며, 이것끼리 `==`로 비교합니다.

```rust
mem::discriminant(&Event::Click { x: 10, y: 20 }) == mem::discriminant(&Event::Click { x: 0, y: 0 }) // true
mem::discriminant(&Event::Click { x: 10, y: 20 }) == mem::discriminant(&Event::Key('q'))            // false
```

→ [`examples/02_enum_niche.rs`](examples/02_enum_niche.rs)

### 2.4 얇은 포인터와 넓은 포인터

크기를 컴파일할 때 아는 값을 가리키는 포인터는 주소 한 칸이면 됩니다. 이것을 **얇은 포인터(thin pointer)** 라고 부릅니다. 동적 크기 타입을 가리키는 포인터는 주소와 함께 추가 정보를 한 칸 더 들고 다니는 **넓은 포인터(wide pointer)** 입니다(Part 0 ch19 2.15절). [`04_box_slices.rs`](examples/04_box_slices.rs)의 1번 구역에서 잰 값입니다.

| 포인터 | 크기 | 들고 있는 것 |
| --- | --- | --- |
| `&i32`, `Box<i32>`, `Rc<i32>` | 8 | 주소 |
| `&[i32]`, `Box<[i32]>` | 16 | 주소 + 원소 수 |
| `&str`, `Box<str>`, `Rc<str>` | 16 | 주소 + 바이트 수 |
| `&dyn Display`, `Box<dyn Display>` | 16 | 주소 + vtable 주소 ([Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)) |
| `Vec<i32>`, `String` | 24 | 주소 + 용량 + 길이 (포인터가 아니라 핸들 구조체) |

참조든 `Box`든 `Rc`든, 무엇을 가리키느냐가 크기를 정합니다. `Box<dyn Trait>`와 `Box<dyn Error>`(`ch8-errors-panic`)도 넓은 포인터라 16바이트입니다.

### 2.5 `std::mem`으로 값 옮기기: `swap`, `replace`, `take`

#### 왜 필요한가

`&mut`로 빌린 자리에서 값을 꺼내 옮길 수는 없습니다.

```rust
impl Shop {
    fn pay(&mut self, amount: u32) {
        self.order = match self.order {          // E0507
            Order::Cart(items) => Order::Paid { items, amount },
            other => other,
        };
    }
}
```

`match self.order`의 `Order::Cart(items)` 패턴은 안의 `Vec`을 `items`로 옮기려 합니다. 그런데 `self`는 빌린 것이라, 옮겨 가면 호출한 쪽의 `Shop`에 **비어 있는 구멍**이 남습니다. 바로 다음에 `self.order = ...`로 다시 채울 생각이라도, 그 사이에 패닉이 나면 구멍 난 `Shop`이 drop되면서 이미 옮겨 간 `Vec`을 또 해제하게 됩니다. 그래서 컴파일러는 빌린 자리를 잠시라도 비우는 일을 허락하지 않습니다(E0507, 4.1절).

해결책은 **꺼내는 순간 다른 값을 채워 넣는 것**입니다. `std::mem`의 세 함수가 이 일을 합니다. 셋 다 프렐류드에 없으므로 `use std::mem;`으로 가져와 `mem::swap`처럼 부릅니다([Part 1 ch1](../ch1-std-overview/README.md) 4.5절).

| 함수 | 시그니처 | 하는 일 |
| --- | --- | --- |
| `swap` | `fn swap<T>(x: &mut T, y: &mut T)` | 두 자리의 값을 맞바꿈 |
| `replace` | `fn replace<T>(dest: &mut T, src: T) -> T` | `src`를 넣고 예전 값을 돌려줌 |
| `take` | `fn take<T: Default>(dest: &mut T) -> T` | 기본값을 넣고 예전 값을 돌려줌 |

어느 함수도 값을 복제하지 않습니다. 비트를 옮길 뿐이고, 두 값 모두 drop되지 않습니다.

#### `replace`로 상태 바꾸기

위의 `pay`를 `mem::replace`로 고치면 다음과 같습니다.

```rust
fn pay(&mut self, amount: u32) {
    let current = mem::replace(&mut self.order, Order::Cart(Vec::new())); // 자리 표시 값을 넣고 꺼냄
    self.order = match current {                                          // current는 이제 소유한 값
        Order::Cart(items) => Order::Paid { items, amount },
        other => other,
    };
}
```

잠깐 들어가는 **자리 표시 값(placeholder)** 은 만들기 값싼 것을 고릅니다. `Vec::new()`와 `String::new()`는 힙을 할당하지 않으므로 거의 비용이 없습니다. [`03_mem_swap_replace_take.rs`](examples/03_mem_swap_replace_take.rs)의 3번 구역은 `Vec<String>` 하나가 한 번도 복제되지 않고 `Cart` → `Paid` → `Shipped`로 옮겨 다니는 것을 보여 줍니다. 상태를 열거형으로 두고 이렇게 전이시키는 방식은 Part 0 ch17에서 README 코드로만 소개한 "열거형 상태 패턴"을 실제로 쓸 때 필요한 도구입니다.

#### `take`: 꺼내고 기본값 남기기

`mem::take(&mut x)`는 `mem::replace(&mut x, Default::default())`와 같습니다. `Default`는 "그 타입의 기본값"을 정하는 트레이트로, 정수는 0, `String`은 빈 문자열, `Vec`은 빈 벡터, `Option`은 `None`입니다(`ch6-core-traits`에서 자세히 다룹니다).

```rust
fn flush(&mut self) -> Vec<String> {
    mem::take(&mut self.pending) // 벡터를 통째로 넘기고, 빈 벡터를 남깁니다.
}
```

- Part 1 ch4에서 미뤄 둔 "벡터 전체의 소유권을 꺼내면서 빈 벡터를 남기기"가 이것입니다. 힙 버퍼가 통째로 넘어가므로 원소를 하나도 복사하지 않습니다. 예제에서 꺼낸 쪽의 용량은 16 그대로이고 남은 쪽의 용량은 0입니다. 버퍼를 남겨 두고 재사용하고 싶다면 `drain(..)`을 씁니다(Part 1 ch4 2.6절).
- Part 1 ch2의 **`Option::take`는 `mem::take`의 `Option` 전용판**입니다. `mem::take(&mut slot)`과 `slot.take()`는 같은 일을 합니다. `Option`에는 메서드가 있으므로 보통 `slot.take()`로 씁니다.

clippy는 더 짧은 쪽이 있을 때 알려 줍니다. 이 툴체인에서 기본으로 켜진 lint입니다.

| 쓴 코드 | lint | 제안 |
| --- | --- | --- |
| `mem::replace(&mut v, Vec::new())` | `clippy::mem_replace_with_default` | `std::mem::take(&mut v)` |
| `mem::replace(&mut opt, None)` | `clippy::mem_replace_option_with_none` | `opt.take()` |
| `let t = x; x = y; y = t;` | `clippy::manual_swap` | `std::mem::swap(&mut x, &mut y);` |

#### `swap`: 맞바꾸기

`mem::swap`은 두 자리의 값을 맞바꿉니다. 화면에 보여 주는 버퍼와 다음 장면을 그리는 버퍼를 번갈아 쓰는 **이중 버퍼**가 대표적인 쓰임입니다. 두 `Vec`을 맞바꿀 때 바뀌는 것은 24바이트짜리 핸들뿐이고 원소는 제자리에 있습니다. 예제는 맞바꾼 뒤 `front`가 예전 `back`의 힙 버퍼를 그대로 쓰는지 주소를 비교해 `true`를 출력합니다(주소 값 자체는 실행마다 달라서 출력하지 않습니다).

같은 벡터의 두 원소를 맞바꿀 때는 `mem::swap(&mut v[0], &mut v[1])`이 아니라 슬라이스의 `v.swap(0, 1)`을 씁니다. 앞의 코드는 `v`를 가변으로 두 번 빌리므로 E0499입니다(4.6절).

### 2.6 `std::mem`의 나머지 함수들

[`std::mem` 문서](https://doc.rust-lang.org/std/mem/index.html)는 이 모듈의 함수를 몇 가족으로 나눕니다. 2.1~2.5절에서 다루지 않은 것을 정리합니다.

| 함수·타입 | 하는 일 | 이 챕터에서 |
| --- | --- | --- |
| `drop` | 값을 받아 버림 | Part 0 ch14 2.6절에서 다룸. 정의가 `fn drop<T>(_x: T) {}`뿐인 함수 |
| `needs_drop::<T>()` | `T`를 버릴 때 할 일(소멸자 실행, 힙 해제)이 있는가 | 예제 03의 5번 구역 |
| `discriminant` | 배리언트만 비교하는 값 | 2.3절 |
| `forget` | 값을 받아 **drop하지 않고** 잊어버림 | 이름만 |
| `DropGuard` | 값을 감싸 두었다가 버려질 때 클로저를 실행 (1.100.0에서 안정화) | 이름만 |
| `transmute`, `zeroed`, `MaybeUninit`, `ManuallyDrop` | 비트를 다른 타입으로 해석하거나 초기화되지 않은 메모리를 다룸 | 이름만 (Part 10) |

**`needs_drop`** 의 결과는 `i32`와 `(u8, &str)`이 `false`, `String`·`Vec<i32>`·`Drop`을 구현한 타입이 `true`입니다. 참조와 정수는 버릴 때 아무 일도 하지 않기 때문입니다. 원소를 직접 관리하는 컬렉션을 만들 때 "버릴 원소를 하나씩 돌 필요가 있는가"를 판단하는 데 씁니다.

**`mem::forget`은 `unsafe` 함수가 아닙니다.** 문서는 그 이유를 이렇게 설명합니다. Rust의 안전성 보장에는 "소멸자가 반드시 실행된다"가 들어 있지 않다. 이미 `Rc`로 순환 참조를 만들어(Part 0 ch14 2.10절) 안전한 코드만으로 누수를 일으킬 수 있으므로, `forget`을 안전한 함수로 두어도 보장이 약해지지 않는다. 누수와 안전성의 관계, `ManuallyDrop`은 Part 10 `ch7-obrm`에서 다룹니다.

나머지 `unsafe` 쪽 도구는 Part 10에서 다룹니다. `transmute`(비트를 그대로 다른 타입으로 해석)는 `ch5-type-conversions`, `MaybeUninit`과 `mem::zeroed`(초기화되지 않은 메모리)는 `ch6-uninitialized-memory`, `ManuallyDrop`은 `ch7-obrm`, C 코드와 소유권을 주고받을 때 쓰는 `Box::into_raw`/`Box::from_raw`는 `ch12-ffi-beneath-std`입니다.

→ [`examples/03_mem_swap_replace_take.rs`](examples/03_mem_swap_replace_take.rs)

### 2.7 `Box` 더 쓰기

#### `Box<[T]>`와 `Box<str>`: 더 늘지 않는 데이터

`Vec<T>`와 `String`은 늘어날 수 있도록 용량 칸과 여유 공간을 둡니다. 다 만든 뒤 더 이상 늘릴 일이 없다면 **`Box<[T]>`** 와 **`Box<str>`** 로 바꿔 둘 수 있습니다.

```rust
let mut v: Vec<i32> = Vec::with_capacity(100);
v.extend([10, 20, 30]);                       // 길이 3, 용량 100
let fixed: Box<[i32]> = v.into_boxed_slice(); // 남는 용량을 버리고 길이 3짜리 버퍼로
let back: Vec<i32> = fixed.into_vec();        // 다시 Vec으로 (길이 3, 용량 3)

let city: Box<str> = String::from("부산").into_boxed_str();
let country: Box<str> = Box::from("대한민국");
```

- 핸들이 24바이트에서 16바이트로 줄고, 힙 버퍼도 길이에 딱 맞습니다. 이름 1000개를 담는다면 핸들만 따져도 24000바이트가 16000바이트가 됩니다.
- **"이 데이터는 더 이상 늘지 않는다"는 의도가 타입에 드러납니다.** `Box<[T]>`에는 `push`가 없습니다. 원소를 고칠 수는 있습니다(`fixed[0] = 11`). `Box`가 데이터를 소유하기 때문입니다.
- 반복자에서 바로 모을 수도 있습니다: `(1..=5).map(|x| x * x).collect::<Box<[u32]>>()`.

#### 큰 값은 `Box`로

`Box`를 이동하면 포인터 한 칸만 복사된다는 것은 Part 0 ch14에서 확인했습니다. 열거형에서는 이 차이가 더 크게 드러납니다. 열거형의 크기는 가장 큰 배리언트가 정하므로, 데이터 없는 `Ping`도 1KB짜리 `Data`만큼 자리를 차지합니다.

```rust
enum Message { Ping, Data([u8; 1024]) }            // 1025바이트
enum BoxedMessage { Ping, Data(Box<[u8; 1024]>) }  //    8바이트
```

`Ping` 1000개를 `Vec`에 담으면 `Message`는 1025000바이트, `BoxedMessage`는 8000바이트입니다. `BoxedMessage`가 8바이트인 것은 2.3절의 니치 최적화 덕분입니다(`Ping`은 `Box` 칸이 0인 값). clippy는 이런 열거형에 `large_enum_variant` 경고(``large size difference between variants``)를 내고, `help: consider boxing the large fields ...`로 큰 필드를 `Box`로 감싸라고 제안합니다. 예제는 비교하려고 `#[expect]`로 막아 두었습니다.

#### 아주 큰 배열은 힙에 바로 만들기

`Box::new(값)`은 인자로 받은 값을 힙으로 옮기는 함수이므로, 값이 먼저 어딘가(보통 스택)에 만들어집니다. 그래서 디버그 빌드에서 `Box::new([7u8; 16 * 1024 * 1024])`(16MiB)를 실행하면 `has overflowed its stack`으로 비정상 종료합니다(macOS의 zsh에서 종료 코드 134. `--release`로 최적화하면 이 코드는 통과했지만, 그것에 기대면 안 됩니다). 큰 버퍼는 처음부터 힙에 만드는 `vec!`으로 만들고 `into_boxed_slice()`로 바꿉니다. 길이가 정해진 배열 타입이 필요하면 `try_into()`로 `Box<[u8; N]>`을 얻습니다. 예제 04의 5번 구역입니다.

`Box<dyn Trait>`는 Part 0 ch17에서, 고정된 위치가 필요한 값을 위한 `Box::pin`은 `ch13-async-primitives`에서 다룹니다.

→ [`examples/04_box_slices.rs`](examples/04_box_slices.rs)

### 2.8 `Rc`와 `Weak` 더 쓰기

`Rc`의 기능은 대부분 `Rc::strong_count(&a)`처럼 **연관 함수**로 부릅니다. `Rc`는 `Deref`로 안의 값의 메서드를 그대로 드러내므로, 이름이 겹치지 않게 하려는 것입니다(Part 0 ch14 2.7절).

#### `==`와 `Rc::ptr_eq`

`a == c`는 `Rc`가 가리키는 **값**을 비교하고, `Rc::ptr_eq(&a, &c)`는 **같은 할당을 가리키는가**를 비교합니다. 내용이 같은 문자열로 따로 만든 두 `Rc`는 `==`가 `true`, `ptr_eq`가 `false`입니다. "이 둘이 같은 객체인가"를 알고 싶을 때는 `ptr_eq`를 씁니다.

#### `Rc<str>`, `Rc<[T]>`

여러 곳이 같은 이름을 공유한다면 `Rc<String>`보다 `Rc<str>`이 낫습니다. `Rc<String>`은 `Rc` 할당 안에 `String` 핸들이 있고 글자는 또 다른 할당에 있어 두 번 따라가야 하지만, `Rc<str>`은 카운트와 글자를 한 할당에 담습니다. 대신 핸들이 넓은 포인터(16바이트)이고, 내용을 바꿀 수 없습니다. 어차피 `Rc`는 안의 값을 바꾸지 못하게 하므로(Part 0 ch14), 공유할 문자열에는 `Rc<str>`이 잘 맞습니다. `Rc::from("김작가")`, `vec![2, 3, 5, 7].into()`처럼 만듭니다.

#### `get_mut`과 `make_mut`: 혼자일 때만 바로 고치기

`Rc` 안의 값을 바꿀 수 없는 이유는 "다른 소유자가 보고 있을 수 있어서"입니다. 그렇다면 **다른 소유자가 없을 때**는 바꿔도 됩니다.

| 함수 | 혼자일 때 | 공유 중일 때 |
| --- | --- | --- |
| `Rc::get_mut(&mut rc)` | `Some(&mut T)` | `None` (`Weak`가 하나라도 있어도 `None`) |
| `Rc::make_mut(&mut rc)` | `&mut T` (제자리) | 안의 값을 **복제**해 새 할당으로 갈라선 뒤 `&mut T` |

`make_mut`처럼 "공유 중일 때만, 쓰기 직전에 복제하는" 방식을 **쓰기 시 복제(clone-on-write)** 라고 부릅니다. 읽기만 하는 동안은 복제 비용이 없고, 처음 고칠 때 한 번만 복제합니다. 예제 05에서 `reader`와 공유 중인 `config`에 `make_mut`으로 `push(5)`하면 `config`만 `[1, 2, 3, 4, 5]`가 되고 `reader`는 `[1, 2, 3, 4]`로 남으며, 둘의 카운트가 각각 1이 됩니다. 그 뒤 혼자가 된 `config`에 다시 `make_mut`하면 같은 할당을 그대로 씁니다. `Rc`가 아닌 참조에서 같은 생각을 구현한 `Cow`는 `ch6-core-traits`에서 다룹니다.

한 가지 주의할 점이 있습니다. 다른 `Rc`는 없고 `Weak`만 남아 있을 때 `make_mut`은 복제하지 않고 **`Weak`와의 연결을 끊습니다.** 예제에서 `make_mut` 뒤에 `watcher.upgrade()`가 `None`이 됩니다.

#### 안의 값을 꺼내기

| 함수 | 혼자일 때 | 공유 중일 때 |
| --- | --- | --- |
| `Rc::try_unwrap(rc)` | `Ok(T)` | `Err(rc)` — 넘긴 `Rc`를 그대로 돌려줌 |
| `Rc::into_inner(rc)` (1.70.0) | `Some(T)` | `None` — 그 `Rc`는 버림 |
| `Rc::unwrap_or_clone(rc)` (1.76.0) | `T` | 안의 값을 복제한 `T` |

`into_inner`는 "모든 복제본에 `into_inner`를 부르면 정확히 한 번만 값을 받는다"고 문서가 보장합니다. 여러 소유자가 차례로 손을 떼다가 마지막 소유자가 값을 넘겨받는 상황에 맞습니다. 세 함수 모두 `Rc`를 값으로 받으므로(`this: Rc<T>`) 부른 뒤에는 넘긴 변수를 더 쓸 수 없습니다. `try_unwrap`이 실패할 때 `Rc`를 `Err`에 담아 돌려주는 것도 그래서입니다.

#### `new_cyclic`: 자기 자신을 가리키는 `Weak`

`&self`만 받은 메서드 안에서 자기 자신의 `Rc`가 필요할 때가 있습니다(콜백 목록에 자신을 등록하는 경우 등). 자기 자신을 `Rc`로 품으면 strong 카운트가 절대 0이 되지 않아 누수이므로 `Weak`로 품어야 하는데, 값을 만들기 전에는 그 값을 가리키는 `Weak`를 얻을 수 없습니다. **`Rc::new_cyclic`** 은 할당을 먼저 해 두고 그곳을 가리키는 `Weak`를 클로저에 넘겨 줍니다.

```rust
Rc::new_cyclic(|me: &Weak<Widget>| Widget { name: name.to_string(), me: me.clone() })
```

클로저가 돌려준 값이 자리에 들어가야 `Rc`가 완성되므로, 클로저 안에서 `me.upgrade()`는 실패(`None`)합니다. 만들어진 뒤에는 `strong 1, weak 1`이고, `self.me.upgrade()`로 자신의 `Rc`를 얻습니다.

스레드 사이에서 공유할 때의 짝인 `Arc`도 같은 이름의 함수(`ptr_eq`, `make_mut`, `into_inner`, `new_cyclic` 등)를 가지고 있으며 `ch11-threads-sync`에서 다룹니다.

→ [`examples/05_rc_more.rs`](examples/05_rc_more.rs)

### 2.9 `Cell<T>`: 값을 통째로 넣고 빼기

[`std::cell` 문서](https://doc.rust-lang.org/std/cell/index.html)는 보통의 가변성을 **상속된 가변성(inherited mutability)** 이라고 부릅니다. `&mut`를 가진 쪽만 값을 바꿀 수 있고, 그 권리가 소유자에서 필드로 이어진다는 뜻입니다. 이와 달리 `&T`만 가지고도 안을 바꿀 수 있게 하는 것이 **내부 가변성(interior mutability)** 이고(Part 0 ch14 2.8절), 표준 라이브러리는 이를 위한 타입을 넷 제공합니다. `RefCell`은 그중 하나입니다.

**`Cell<T>`** 는 가장 단순한 방법을 씁니다. **안의 값을 가리키는 참조를 절대 내주지 않습니다.** 값을 복사해 주거나(`get`), 통째로 바꿔 넣을(`set`, `replace`) 뿐입니다.

| 메서드 | 조건 | 하는 일 |
| --- | --- | --- |
| `get(&self) -> T` | `T: Copy` | 안의 값을 복사해 돌려줌 |
| `set(&self, val: T)` | 모든 `T` | 새 값을 넣고 예전 값은 버림 |
| `update(&self, f)` (1.88.0) | `T: Copy` | `f(예전 값)`을 새 값으로 넣음 |
| `replace(&self, val: T) -> T` | 모든 `T` | 새 값을 넣고 예전 값을 돌려줌 |
| `take(&self) -> T` | `T: Default` | 기본값을 넣고 예전 값을 돌려줌 |
| `swap(&self, other: &Cell<T>)` | 모든 `T` | 두 `Cell`의 값을 맞바꿈 |
| `into_inner(self) -> T` | 모든 `T` | `Cell`을 소비하고 안의 값을 돌려줌 |
| `get_mut(&mut self) -> &mut T` | 모든 `T` | `&mut Cell`이 있을 때만. 참조를 줌 |

`replace`와 `take`가 2.5절의 `mem::replace`, `mem::take`와 이름과 동작이 같다는 점에 주목하세요. `Cell`은 "`&mut` 대신 `&`로 부르는 `mem` 함수"라고 볼 수 있습니다.

**왜 참조를 내주지 않으면 실행 중 검사가 필요 없을까요?** `RefCell`이 대여를 세는 이유는 누군가 안의 값을 가리키는 `&T`를 들고 있는 동안 다른 누가 그 값을 바꾸면 안 되기 때문입니다. `Cell`은 그런 참조를 아예 만들지 않으므로, 안의 값이 바뀌는 순간 그 값을 들여다보는 참조가 존재할 수 없습니다. `get`으로 받은 것은 복사본이라 원본이 바뀌어도 그대로입니다(`seen = 1, level = 5`). 그래서 `Cell<u32>`는 `u32`와 같은 4바이트이고 아무 실행 비용이 없습니다. 대여 카운트를 들고 있는 `RefCell<u32>`는 16바이트입니다.

`get`이 `T: Copy`일 때만 있는 것도 같은 이유입니다. `String`을 비트 그대로 복사하면 같은 힙 버퍼를 가리키는 `String`이 둘 생기고, 참조로 줄 수도 없습니다. 그래서 `Cell<String>`에 `get()`을 부르면 E0599입니다(4.2절). `Copy`가 아닌 값은 **꺼내서 고치고 다시 넣습니다.**

```rust
let mut current = title.take();       // 꺼내고 빈 String을 남김
current.push_str(" (검토 완료)");
title.set(current);                   // 다시 넣음
```

`Cell`이 가장 잘 맞는 곳은 `&self` 메서드 안의 작은 통계나 플래그입니다. 예제 06의 `Dictionary::lookup`은 `&self`를 받으면서 조회 수를 셉니다. 조회 수는 사전의 내용이 아니라 내부 사정이므로, 이것 때문에 `&mut self`를 요구하면 사전을 여러 곳에서 동시에 읽을 수 없게 됩니다. 또 `Rc<Cell<i32>>`를 캡처한 클로저는 `update`가 `&self`만 필요하므로 `FnMut`가 아니라 `Fn`이 됩니다([Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md)의 `Fn` 트레이트).

→ [`examples/06_cell.rs`](examples/06_cell.rs)

### 2.10 `RefCell<T>` 더 쓰기

`RefCell`은 `Cell`과 반대로 안의 값을 가리키는 참조를 내줍니다. 대신 지금 살아 있는 `Ref`/`RefMut`의 개수를 세서 대여 규칙을 실행 중에 검사합니다(Part 0 ch14 2.8절). 이 챕터에서는 `Ref`/`RefMut`를 **대여 가드(guard)** 라고 부릅니다. 가드가 살아 있는 동안 대여가 이어지고, 가드가 버려지면 대여가 끝납니다.

#### `try_borrow_mut`으로 재진입 감지하기

`RefCell`의 이중 대여는 대개 **재진입(reentrancy)** 에서 생깁니다. 어떤 메서드가 가드를 쥔 채로 콜백을 부르고, 그 콜백이 같은 객체의 다른 메서드를 다시 부르는 경우입니다. 예제 07의 게시판은 `for_each`가 글 목록의 `Ref`를 쥐고 콜백을 부르는데, 콜백이 같은 게시판에 답글을 쓰려 합니다. `post`가 `borrow_mut()` 대신 `try_borrow_mut()`을 쓰므로 패닉하지 않고 `Err`를 받습니다.

```text
글 올리기 실패 [공지에 답글]: RefCell already borrowed
```

#### 가드를 돌려주기: `Ref::map`

`RefCell` 안의 값 일부를 가리키는 `&str`을 메서드에서 돌려주려 하면 E0515입니다(4.3절). `borrow()`가 만든 가드가 함수 끝에서 버려져 대여가 끝나는데, 참조만 밖에 남으면 `RefCell`이 모르는 사이에 값이 바뀔 수 있기 때문입니다. 대여를 이어 가려면 **가드 자체를 돌려줍니다.** `Ref::map`은 가드를 "안의 일부를 가리키는 가드"로 바꿉니다.

```rust
fn name(&self) -> Ref<'_, str> {
    Ref::map(self.data.borrow(), |d| d.name.as_str())
}
fn visits_mut(&self) -> RefMut<'_, u32> {
    RefMut::map(self.data.borrow_mut(), |d| &mut d.visits)
}
```

돌려받은 `Ref<str>`은 `&str`처럼 쓰고, 그것이 살아 있는 동안은 `data` 전체가 빌려진 상태입니다. 예제에서 `name`이 살아 있을 때 `try_borrow_mut()`은 실패하고, 블록을 나온 뒤에는 성공합니다. `Ref::map`도 연관 함수로 부릅니다. `Ref`는 `Deref`로 안의 값의 메서드를 드러내므로 이름이 겹치지 않게 한 것입니다.

#### 가드 없이 통째로: `take`, `replace`, `replace_with`, `swap`

`RefCell`에도 `Cell`과 같은 이름의 메서드가 있습니다. `take()`(1.50.0)는 안의 값을 꺼내고 기본값을 남기며, `replace(새 값)`은 예전 값을, `replace_with(|old| ...)`는 예전 값의 `&mut`로 새 값을 만든 뒤 예전 값을 돌려줍니다. **돌려받는 것이 가드가 아니라 값**이므로 대여가 그 줄에서 끝납니다. 받은편지함을 통째로 꺼내 처리하는 동안 다른 코드가 받은편지함에 새 메일을 넣어도 안전합니다.

#### 함정: 조건식에서 만든 가드는 본문 끝까지 살아 있습니다

임시값은 보통 그 문장이 끝날 때 버려집니다. 그래서 `let len = log.borrow().len();`의 가드는 그 줄에서 사라집니다. 그런데 `match`, `if let`, `while let`의 **조건식(scrutinee)** 에서 만든 임시값은 본문이 끝날 때까지 살아 있습니다. 가드도 임시값이므로 같은 규칙을 따릅니다.

```rust
while let Some(task) = tasks.borrow_mut().pop_front() {
    // 여기서는 아직 RefMut가 살아 있습니다.
    tasks.borrow_mut().push_back("정리"); // 패닉: RefCell already borrowed
}
```

이 툴체인에서 확인한 결과를 정리하면 다음과 같습니다(예제 07의 4번 구역, `try_borrow_mut().is_ok()`로 관찰).

| 조건식에서 만든 가드가 | 살아 있는 범위 |
| --- | --- |
| `let x = cell.borrow().len();` | 그 `let` 문 끝까지 |
| `match cell.borrow().last() { .. }` | 모든 갈래가 끝날 때까지 (값을 `.cloned()`로 복사해도 마찬가지) |
| `while let Some(x) = cell.borrow_mut().pop() { .. }` | 매 반복의 본문이 끝날 때까지 |
| `if let Some(x) = cell.borrow().first() { .. } else { .. }` | `if` 본문 끝까지. 2024 에디션에서는 `else`에 들어가기 전에 버려짐(2021 에디션에서는 `else` 끝까지) |
| `println!("{} {}", cell.try_borrow_mut().is_ok(), cell.borrow().len())` | 문장 끝까지. 앞 인자의 `RefMut`가 살아 있어 뒤의 `borrow()`가 패닉 |

마지막 줄은 이 챕터의 예제를 작성하는 중에 실제로 만난 패닉입니다(`RefCell already mutably borrowed`). 해결책은 모두 같습니다. **가드가 필요 없는 값을 `let` 문으로 먼저 꺼내 둡니다.**

```rust
loop {
    let next = tasks.borrow_mut().pop_front(); // RefMut는 이 줄에서 버려짐
    let Some(task) = next else { break };
    // 이제 tasks.borrow_mut()을 마음껏 불러도 됩니다.
}
```

`if let` 규칙이 에디션마다 다른 이유는 Part 2 `ch4-rust-2024-language`에서 다룹니다.

#### `&mut`가 있으면 `get_mut`

`RefCell`을 `&mut`로 가지고 있다면 `get_mut()`으로 안의 값의 `&mut`를 바로 얻습니다. 컴파일러가 유일한 접근임을 보장하므로 대여 카운트를 건드리지 않습니다. `Cell::get_mut`, `OnceCell::get_mut`도 같은 생각입니다.

→ [`examples/07_refcell_more.rs`](examples/07_refcell_more.rs)

### 2.11 `OnceCell<T>`와 `LazyCell<T, F>`: 한 번만 채우기

**`OnceCell<T>`** (1.70.0)는 비어 있다가 **한 번만 채울 수 있는** 칸입니다. 한 번 채우면 `&mut`가 없는 한 다시 바꿀 수 없습니다.

| 메서드 | 하는 일 |
| --- | --- |
| `get(&self) -> Option<&T>` | 채워져 있으면 참조를 줌 |
| `set(&self, value: T) -> Result<(), T>` | 비어 있으면 채우고 `Ok(())`, 이미 찼으면 넣으려던 값을 `Err`로 돌려줌 |
| `get_or_init(&self, f) -> &T` | 비어 있으면 `f()`로 채운 뒤, 어느 쪽이든 참조를 줌 |
| `take(&mut self) -> Option<T>` | 비우고 값을 돌려줌 (`&mut` 필요) |

`OnceCell`은 `Cell`과 `RefCell`의 중간입니다. `Cell`과 달리 **안의 값을 가리키는 `&T`를 내주는데**, `RefCell`과 달리 **대여를 세지 않습니다.** 한 번 채운 값은 `&self`로는 다시 바뀌지 않으므로, 아무리 많은 `&T`가 동시에 살아 있어도 그 값이 바뀌거나 사라질 일이 없기 때문입니다. `take`가 `&mut self`를 요구하는 것도 같은 이유입니다. `&mut`를 가졌다는 것은 빌려 준 `&T`가 하나도 남아 있지 않다는 뜻입니다. 단, `get_or_init`에 넘긴 초기화 함수 안에서 같은 셀을 다시 초기화하려 하면 패닉한다고 문서에 적혀 있습니다.

이 성질 덕분에 `OnceCell`은 **`&self` 메서드에서 계산 결과를 캐시해 `&T`로 빌려 주는** 데 딱 맞습니다.

```rust
struct Article { body: String, summary: OnceCell<String> }

impl Article {
    fn summary(&self) -> &str {
        self.summary.get_or_init(|| { /* 비싼 계산 */ })
    }
}
```

Part 1 ch2의 `Report::summary`는 `Option::get_or_insert_with`로 같은 일을 했지만 `&mut self`가 필요했습니다. `RefCell<Option<String>>`으로 바꾸면 `&self`는 되지만 가드 때문에 `&str`을 바로 돌려줄 수 없습니다(2.10절). `OnceCell`은 `&self`를 받으면서 `&str`을 그대로 돌려줍니다. 예제 08에서 두 번 부른 결과 `s1`, `s2`는 같은 `String`을 가리키고, 요약을 만드는 클로저는 한 번만 불립니다.

**`LazyCell<T, F>`** (1.80.0)는 `OnceCell<T>`와 초기화 함수 `F`를 한데 묶은 것입니다. 만들 때 함수를 넘겨 두면, 처음 역참조할 때(`*lazy`, 메서드 호출, 인덱싱) 그 함수가 불려 값이 채워집니다.

```rust
let settings = LazyCell::new(|| vec![("theme", "dark"), ("lang", "ko")]);
LazyCell::get(&settings); // None — 들여다보기만 하고 초기화하지 않음 (1.94.0)
settings.len();           // 여기서 초기화 함수가 불림
```

`get_or_init`을 부를 때마다 같은 함수를 넘긴다면 `LazyCell`이 더 간단하고, 상황마다 다른 방법으로 채워야 한다면 `OnceCell`을 씁니다.

→ [`examples/08_once_lazy.rs`](examples/08_once_lazy.rs)

### 2.12 내부 가변성 도구 고르기

[`std::cell` 문서](https://doc.rust-lang.org/std/cell/index.html#when-to-choose-interior-mutability)는 내부 가변성을 "최후의 수단"이라고 부릅니다. `&mut`로 표현할 수 있으면 그것이 먼저입니다. 컴파일러가 모든 것을 검사해 주기 때문입니다. 그래도 내부 가변성이 필요한 대표적인 경우로 문서는 세 가지를 듭니다. `Rc`처럼 공유되어 `&`만 얻을 수 있는 값 안을 바꿔야 할 때, 논리적으로는 읽기인 메서드가 내부적으로 캐시나 통계를 바꿔야 할 때, 그리고 `&self`를 받는 `clone` 안에서 무언가를 바꿔야 할 때(`Rc`가 카운트를 `Cell`에 두는 이유)입니다.

| 타입 | 안의 값의 참조를 주는가 | 실행 중 검사 | 크기 (`T = u32`) | 이럴 때 |
| --- | --- | --- | --- | --- |
| `Cell<T>` | 주지 않음 (복사·교체만) | 없음 | 4 | 숫자, 플래그, 작은 `Copy` 값. 꺼내서 고치고 다시 넣어도 되는 값 |
| `RefCell<T>` | `Ref`/`RefMut` 가드로 줌 | 대여를 셈. 위반하면 패닉 | 16 | 컬렉션처럼 큰 값을 제자리에서 고쳐야 할 때 |
| `OnceCell<T>` | `&T`를 줌 | 없음 (한 번 채우면 안 바뀜) | 8 | 처음 한 번 계산해 두고 계속 읽는 값 |
| `LazyCell<T, F>` | 역참조로 `&T`를 줌 | 없음 | 함수에 따라 다름 | 초기화 방법이 하나로 정해진 지연 값 |

문서는 쓸 수 있다면 다른 셀 타입보다 `Cell`을 먼저 고르라고 권합니다. 실행 비용이 없고 패닉할 일도 없기 때문입니다.

이 넷은 모두 **단일 스레드 전용**입니다. 여러 스레드가 `&`로 함께 써도 안전하다는 표시인 `Sync`를 구현하지 않으므로 `static`에 둘 수 없습니다(4.4절). 스레드 사이에서 쓰는 짝은 다음과 같으며 모두 `ch11-threads-sync`에서 다룹니다.

| 단일 스레드 | 스레드 안전한 짝 |
| --- | --- |
| `Cell<u32>`, `Cell<bool>` 등 | 원자 타입 `AtomicU32`, `AtomicBool` 등 |
| `RefCell<T>` | `Mutex<T>`, `RwLock<T>` |
| `OnceCell<T>` | `OnceLock<T>` |
| `LazyCell<T, F>` | `LazyLock<T, F>` |
| `Rc<T>` | `Arc<T>` |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_size_align.rs`](examples/01_size_align.rs) | 원시 타입의 크기와 메모리 정렬, 구조체 패딩, `offset_of!`로 본 필드 순서, 크기가 0인 타입, `size_of_val` | 1 |
| [`examples/02_enum_niche.rs`](examples/02_enum_niche.rs) | 열거형의 태그, 니치 최적화(보장되는 것과 관찰한 것), `String`을 담은 열거형의 크기, `mem::discriminant` | 2 |
| [`examples/03_mem_swap_replace_take.rs`](examples/03_mem_swap_replace_take.rs) | 이중 버퍼 `swap`, `replace`로 상태 전이, `take`로 벡터 통째로 꺼내기, `needs_drop` | 3 |
| [`examples/04_box_slices.rs`](examples/04_box_slices.rs) | 얇은/넓은 포인터 크기, `Box<[T]>`, `Box<str>`, 큰 배리언트 `Box`로 감싸기, 큰 배열을 힙에 바로 만들기 | 2, 4 |
| [`examples/05_rc_more.rs`](examples/05_rc_more.rs) | `ptr_eq`, `Rc<str>`/`Rc<[T]>`, `get_mut`/`make_mut`, `try_unwrap`/`into_inner`/`unwrap_or_clone`, `new_cyclic` | 4 |
| [`examples/06_cell.rs`](examples/06_cell.rs) | `&self` 안에서 세기, `get`은 복사본, `Copy`가 아닌 값의 `replace`/`take`/`swap`, `Rc<Cell<T>>`와 `Fn` 클로저 | 5 |
| [`examples/07_refcell_more.rs`](examples/07_refcell_more.rs) | 재진입 감지, `Ref::map`/`RefMut::map`, 가드 없는 통째 연산, 조건식의 가드 함정, `get_mut` | 5 |
| [`examples/08_once_lazy.rs`](examples/08_once_lazy.rs) | `OnceCell`의 `set`/`get`/`get_or_init`, `&self` 캐시로 `&str` 돌려주기, `LazyCell` | 5 |
| [`compile_fail/`](compile_fail) | 이 챕터에서 자주 만나는 컴파일 에러 4개 (4절) | 3, 5 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 패닉 없이 끝나고 종료 코드는 0입니다. 주소 값은 실행마다 다르므로 어느 예제도 출력하지 않고, 같은 곳을 가리키는지는 비교 결과(`true`/`false`)로 보입니다. 크기와 오프셋은 64비트 환경, rustc 1.100.0에서 확인한 값입니다.

### 01_size_align — 크기와 메모리 정렬

```sh
cargo run -p p1-ch7-smart-pointers-memory --example 01_size_align
```

1번 구역(원시 타입)의 출력은 2.1절의 표와 같아 생략했습니다.

```text
== 2. 구조체의 크기 = 필드 합 + 패딩 ==
Packet       크기  8, 정렬  4
Record       크기 16, 정렬  8
[Packet; 3]  크기 24, 정렬  4

== 3. 기본 표현에서는 컴파일러가 필드 순서를 바꿉니다 ==
Packet: tag@4, id@0, flag@5
Record: kind@10, timestamp@0, code@8
(u8,u32,u8)  크기  8, 정렬  4

== 4. 크기가 0인 타입 ==
()           크기  0, 정렬  1
Marker       크기  0, 정렬  1
[Marker; 1000] 크기  0, 정렬  1
[i32; 0]     크기  0, 정렬  4

== 5. size_of_val: 핸들의 크기와 내용물의 크기 ==
Vec 핸들 24, 원소 슬라이스 12
String 핸들 24, 글자 6

필드 값: 1 7 0 / 2 1700000000 404, Marker 3개
```

3번 구역의 오프셋은 보장되지 않는 값입니다(2.2절). `[i32; 0]`은 크기가 0이어도 메모리 정렬은 `i32`를 따라 4입니다. 마지막 줄은 필드를 실제로 읽어 `dead_code` 경고를 피하려고 출력한 것입니다.

### 02_enum_niche — 열거형과 니치 최적화

```sh
cargo run -p p1-ch7-smart-pointers-memory --example 02_enum_niche
```

1번 구역(태그가 붙는 열거형)의 출력은 2.3절 첫 표와 같아 생략했습니다.

```text
== 2. 니치 최적화: 포인터는 0이 될 수 없습니다 (문서가 보장) ==
Box<i32>                    8바이트
Option<Box<i32>>            8바이트
&i32                        8바이트
Option<&i32>                8바이트
Option<&str>               16바이트
List (Cons/Nil)            16바이트

== 3. 니치 최적화: 값의 범위가 좁은 타입 (이 툴체인의 결과) ==
Option<bool>                1바이트
Option<Option<bool>>        1바이트
Option<char>                4바이트
Option<Direction>           1바이트

== 4. 큰 타입 안의 빈자리 (이 툴체인의 결과) ==
String                     24바이트
Option<String>             24바이트
SheetCell (Int/Float/Text) 24바이트
Option<Option<String>>     24바이트
Option<Option<Box<i32>>>   16바이트
TwoTexts                   32바이트

== 5. mem::discriminant: 필드는 무시하고 배리언트만 비교 ==
Click(10,20) 와 Click(0,0): 같은 배리언트? true
Click(10,20) 와 Key(q): 같은 배리언트? false
Key 이벤트만: ["Key(q)", "Key(w)"]
```

크기만 재는 열거형들은 값을 만들지 않으므로 `#[expect(dead_code, reason = ...)]`를 붙였습니다. 4번 구역의 표를 2.3절의 설명과 맞춰 읽어 보세요.

### 03_mem_swap_replace_take — `&mut` 뒤의 값 옮기기

```sh
cargo run -p p1-ch7-smart-pointers-memory --example 03_mem_swap_replace_take
```

```text
== 1. swap: 두 자리의 값을 맞바꾸기 ==
a = 오른쪽, b = 왼쪽
front = ["장면 2"], back = []
front가 예전 back의 힙 버퍼를 그대로 쓰는가? true

== 2. replace: 새 값을 넣고 예전 값을 돌려받기 ==
예전 = 대기, 지금 = 실행 중

== 3. replace로 상태 전이 (&mut self 뒤의 열거형) ==
장바구니 ["사과", "배"]
장바구니 ["사과", "배"]
결제 완료 ["사과", "배"], 5000원
배송 중 ["사과", "배"], 송장 A-1

== 4. take: 꺼내고 기본값 남기기 ==
꺼낸 것: ["시작", "완료"] (용량 16), 남은 것: [] (용량 0)
꺼낸 값: 42, "제목", Some(7)
남은 값: 0, "", None

== 5. needs_drop: 버릴 때 할 일이 있는 타입인가 ==
i32: false
(u8, &str): false
String: true
Vec<i32>: true
Noisy: true
[drop] Noisy
끝
```

3번 구역의 둘째 줄은 결제 전에 `ship`을 부른 결과입니다. `other => other` 갈래가 꺼냈던 장바구니를 그대로 되돌려 놓았습니다. 4번 구역의 용량은 이 툴체인의 결과입니다(`Vec::new()`가 할당하지 않는다는 것은 문서가 보장합니다).

### 04_box_slices — `Box`의 여러 모양

```sh
cargo run -p p1-ch7-smart-pointers-memory --example 04_box_slices
```

1번 구역(포인터 크기)의 출력은 2.4절의 표와 같아 생략했습니다.

```text
== 2. Box<[T]>: 더 늘지 않는 슬라이스를 소유하기 ==
Vec: 길이 3, 용량 100
Box<[i32]>: [11, 20, 30], 길이 3
다시 Vec: 길이 3, 용량 3
squares = [1, 4, 9, 16, 25]

== 3. Box<str>: 바뀌지 않는 문자열을 소유하기 ==
부산, 대한민국, 글자 바이트 6
핸들 1000개: String 24000바이트, Box<str> 16000바이트
부산광역시

== 4. 큰 배리언트는 Box로 감싸기 ==
Message            1025바이트
BoxedMessage          8바이트
Ping 1000개: Message 1025000바이트, BoxedMessage 8000바이트
페이로드 길이: 0, 0, 1024
Message::Data 길이: 1024

== 5. 아주 큰 배열은 힙에 바로 만들기 ==
Box<[u8; LEN]>: 원소 16777216개, 첫 값 0
```

`Message` 위의 `#[expect(clippy::large_enum_variant, ...)]`를 지우고 `cargo clippy -p p1-ch7-smart-pointers-memory --examples`를 실행하면 2.7절의 경고가 나옵니다.

### 05_rc_more — `Rc`의 여러 함수

```sh
cargo run -p p1-ch7-smart-pointers-memory --example 05_rc_more
```

```text
== 1. == 는 값을, Rc::ptr_eq는 가리키는 곳을 비교합니다 ==
a == c: true, ptr_eq(a, c): false
a == b: true, ptr_eq(a, b): true

== 2. Rc<str>, Rc<[T]>: 바뀌지 않는 문자열·슬라이스를 공유하기 ==
첫 책 - 김작가, 둘째 책 - 김작가
author의 strong_count = 3
Rc<String> 8바이트, Rc<str> 16바이트
primes = [2, 3, 5, 7], 합 = 17

== 3. get_mut과 make_mut: 혼자일 때만 바로 고칩니다 ==
혼자일 때 get_mut → [1, 2, 3, 4]
공유 중 get_mut은 None? true
config = [1, 2, 3, 4, 5], reader = [1, 2, 3, 4]
갈라선 뒤 ptr_eq: false, 카운트: 1 / 1
혼자일 때 make_mut은 같은 할당? true
Weak가 있을 때 get_mut은 None? true
make_mut 뒤 watcher.upgrade(): None

== 4. 안의 값을 꺼내기: try_unwrap / into_inner / unwrap_or_clone ==
try_unwrap(혼자) = Ok("혼자")
Err: 아직 2곳이 공유 중
into_inner(first) = None
into_inner(other) = Some("공유")
mine = ["초안", "수정"], keep = ["초안"]

== 5. weak_count와 new_cyclic: 자기 자신을 가리키는 Weak ==
  new_cyclic 안에서 upgrade 성공? false
확인 버튼: strong 1, weak 1
handle을 얻은 뒤: strong 2, 같은 위젯? true
```

3번 구역의 "같은 할당?"은 `Rc::as_ptr`(안의 값을 가리키는 원시 포인터)를 따라가지 않고 비교만 한 결과입니다.

### 06_cell — `Cell<T>`

```sh
cargo run -p p1-ch7-smart-pointers-memory --example 06_cell
```

```text
== 1. &self 메서드 안에서 세기: get / set / update ==
cell → Some("칸")
rust → None
borrow → Some("빌리다")
조회 3번, 실패 1번

== 2. get은 복사본을 줍니다 ==
seen = 1, level = 5
u32 4, Cell<u32> 4, RefCell<u32> 16

== 3. Copy가 아닌 값: replace / take / swap / into_inner ==
replace가 돌려준 예전 값: 초안
title = 2차 원고 (검토 완료)
tags_a = ["보류", "검토"], tags_b = ["긴급"]

== 4. &mut가 있으면 get_mut으로 바로 고칩니다 ==
score = 15

== 5. Rc<Cell<T>>: 여러 클로저가 같은 칸을 바꿉니다 ==
클릭 점수 = 12, strong_count = 3
```

1번 구역은 `reader_a`, `reader_b` 두 불변 참조로 같은 사전을 조회했는데 조회 수가 합쳐졌습니다. 5번 구역의 `strong_count = 3`은 `clicks` 자신과 두 클로저가 캡처한 복제본입니다.

### 07_refcell_more — `RefCell<T>` 깊이 쓰기

```sh
cargo run -p p1-ch7-smart-pointers-memory --example 07_refcell_more
```

```text
== 1. try_borrow_mut으로 재진입 감지 ==
글 올리기 실패 [공지에 답글]: RefCell already borrowed
글 목록: ["공지", "새 글"]

== 2. Ref::map / RefMut::map: 일부만 빌려 주기 ==
이름 = 러스티, 길이 = 9
name이 살아 있을 때 가변 대여 가능? false
블록을 나온 뒤 가변 대여 가능? true, visits = 1

== 3. take / replace / replace_with: 가드를 남기지 않는 통째 연산 ==
꺼낸 것 ["메일1", "메일2"], 남은 것 []
replace가 돌려준 값 []
before ["메일3"], now ["메일3", "메일4"]

== 4. 함정: 조건식에서 만든 대여는 본문 끝까지 살아 있습니다 ==
빨래: 본문에서 가변 대여 가능? false
청소: 본문에서 가변 대여 가능? false
처리: 빨래
처리: 청소
처리: 정리
기록 없음. 이 갈래에서 가변 대여 가능? false

== 5. &mut RefCell이 있으면 get_mut: 실행 중 검사 없이 ==
settings = [1, 2, 3]
```

4번 구역의 `while let` 본문에서 `tasks.borrow_mut().push_back(..)`을 부르도록 바꿔 보면, 첫 번째 할 일을 처리하는 도중 `RefCell already borrowed`로 패닉합니다(종료 코드 101). 아래쪽 `loop` 버전은 같은 일을 패닉 없이 해냅니다.

### 08_once_lazy — `OnceCell`과 `LazyCell`

```sh
cargo run -p p1-ch7-smart-pointers-memory --example 08_once_lazy
```

```text
== 1. OnceCell: get / set ==
처음 get: None
첫 set: Ok(())
둘째 set: Err("지영")
get: Some("민수")

== 2. get_or_init: &self 메서드에서 한 번만 계산하기 ==
  (요약을 만듭니다)
s1 = 러스트의 스마트…, s2 = 러스트의 스마트…
같은 곳을 가리키는가? true

== 3. &mut가 있으면 비우고 다시 채울 수 있습니다 ==
  (요약을 만듭니다)
새 요약 = Cell과 Re…

== 4. LazyCell: 처음 쓸 때 초기화 ==
아직 초기화 전: None
  (설정을 읽습니다)
항목 수 = 2
첫 항목 = ("theme", "dark")
초기화 뒤 값이 있는가? true
```

`(요약을 만듭니다)`가 `summary()`를 두 번 부른 2번 구역에서는 한 번, 본문을 바꾼 3번 구역에서 다시 한 번 찍힙니다. `(설정을 읽습니다)`는 `LazyCell::get`으로 들여다볼 때가 아니라 `settings.len()`을 부를 때 찍힙니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch7-smart-pointers-memory
rustc --edition 2024 compile_fail/e0507_move_state_behind_mut.rs --out-dir /tmp/rust-p1-ch7
```

### 4.1 `&mut self` 뒤의 열거형 상태를 그냥 꺼내기 — E0507

파일: [`compile_fail/e0507_move_state_behind_mut.rs`](compile_fail/e0507_move_state_behind_mut.rs)

```text
error[E0507]: cannot move out of `self.order` as enum variant `Cart` which is behind a mutable reference
  --> compile_fail/e0507_move_state_behind_mut.rs:28:28
   |
28 |         self.order = match self.order {
   |                            ^^^^^^^^^^
29 |             Order::Cart(items) => Order::Paid { items, amount },
   |                         ----- data moved here
30 |             other => other,
   |             ----- ...and here
   |
   = note: move occurs because these variables have types that don't implement the `Copy` trait
help: consider borrowing the pattern binding
   |
29 |             Order::Cart(ref items) => Order::Paid { items, amount },
   |                         +++
```

- **읽는 법**: `self.order`는 가변 참조(`&mut self`) 뒤에 있는데, 패턴의 `items`와 `other`가 그 안의 값을 옮겨 가려 한다(data moved here)는 뜻입니다. `help:`는 `other` 쪽에도 `ref other`를 한 번 더 제안합니다(인용에서는 생략).
- **원인**: 빌린 자리에서 값을 옮기면 빌려준 쪽에 구멍이 남습니다(2.5절). 바로 다음에 `self.order`를 다시 채우더라도 컴파일러는 그 사이를 허락하지 않습니다.
- **해결**: `help:`의 `ref items`를 따르면 `items`가 `&Vec<String>`이 되어 `Order::Paid { items, .. }`에 넣을 수 없으므로 E0308(`mismatched types`)로 바뀔 뿐입니다. 값을 빌리는 것이 아니라 정말 옮기려는 것이므로 `let current = mem::replace(&mut self.order, Order::Cart(Vec::new()));`로 자리 표시 값을 넣고 꺼낸 뒤 `match current`로 씁니다. `Order`가 `Default`를 구현했다면 `mem::take(&mut self.order)`도 됩니다. Part 1 ch2의 E0507(`&` 뒤의 `Option`에 `map`)이 "빌리기만 하면 되는" 경우였다면, 이것은 "정말 옮겨야 하는" 경우입니다.

### 4.2 `Copy`가 아닌 타입의 `Cell`에 `get()` — E0599

파일: [`compile_fail/e0599_cell_get_non_copy.rs`](compile_fail/e0599_cell_get_non_copy.rs)

```text
error[E0599]: the method `get` exists for struct `Cell<String>`, but its trait bounds were not satisfied
  --> compile_fail/e0599_cell_get_non_copy.rs:26:33
   |
26 |     let name = profile.nickname.get();
   |                                 ^^^
   |
   = note: the following trait bounds were not satisfied:
           `String: Copy`
```

- **읽는 법**: `get` 메서드는 있지만, `String: Copy`라는 조건을 만족하지 않아 쓸 수 없다는 뜻입니다. 문서에서 `get`은 `impl<T: Copy> Cell<T>` 블록에 있습니다.
- **원인**: `Cell`은 참조를 내주지 않으므로 `get`은 값을 복사해 줄 수밖에 없고, 비트 복사가 안전한 것은 `Copy` 타입뿐입니다(2.9절).
- **해결**: 무엇을 하려는지에 따라 고릅니다. 잠깐 꺼내 쓰고 되돌려 놓으려면 `take()`와 `set()`, 새 값으로 바꾸며 예전 값을 받으려면 `replace()`, 참조로 읽어야 한다면 `RefCell<String>`의 `borrow()`, 한 번 정하고 바꾸지 않는 값이라면 `OnceCell<String>`의 `get()`(`Option<&String>`)을 씁니다.

### 4.3 `RefCell` 안의 값을 가리키는 참조 돌려주기 — E0515

파일: [`compile_fail/e0515_return_ref_from_refcell.rs`](compile_fail/e0515_return_ref_from_refcell.rs)

```text
error[E0515]: cannot return value referencing temporary value
  --> compile_fail/e0515_return_ref_from_refcell.rs:26:9
   |
26 |         &self.songs.borrow()[0]
   |         ^-------------------^^^
   |         ||
   |         |temporary value created here
   |         returns a value referencing data owned by the current function
```

- **읽는 법**: `self.songs.borrow()`가 만든 임시값(`Ref` 가드)을 거쳐 얻은 참조를 돌려주려 한다는 뜻입니다. [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md)의 E0515는 지역 변수를 가리키는 참조였고, 여기서는 그 자리에 대여 가드가 있습니다.
- **원인**: 가드는 함수가 끝날 때 버려지고, 그 순간 `RefCell`은 "아무도 읽지 않는다"고 기록합니다. 참조만 밖에 남으면 `RefCell`이 모르는 사이에 `borrow_mut()`으로 값을 바꿀 수 있게 됩니다.
- **해결**: 가드를 돌려줍니다. `fn first(&self) -> Ref<'_, str> { Ref::map(self.songs.borrow(), |v| v[0].as_str()) }`(2.10절). 가드를 오래 쥐는 것이 부담스러우면 `self.songs.borrow()[0].clone()`으로 소유한 `String`을 돌려줍니다. 한 번 정하고 바꾸지 않는 값이라면 `OnceCell`로 `&str`을 그대로 돌려줄 수 있습니다(2.11절).

함수 안에서 같은 실수를 하면 E0716이 됩니다. `let first = songs.borrow().first();` 다음 줄에서 `first`를 쓰면 ``temporary value dropped while borrowed``와 함께 컴파일러가 `let binding = songs.borrow();`로 가드를 먼저 변수에 담으라고 제안합니다([Part 1 ch3](../ch3-strings-fmt/README.md)의 E0716과 같은 원리입니다).

### 4.4 `static`에 `OnceCell`(또는 `Cell`) 두기 — E0277

파일: [`compile_fail/e0277_static_oncecell.rs`](compile_fail/e0277_static_oncecell.rs)

```text
error[E0277]: `OnceCell<String>` cannot be shared between threads safely
  --> compile_fail/e0277_static_oncecell.rs:19:16
   |
19 | static CONFIG: OnceCell<String> = OnceCell::new();
   |                ^^^^^^^^^^^^^^^^ `OnceCell<String>` cannot be shared between threads safely
   |
   = help: the trait `Sync` is not implemented for `OnceCell<String>`
   = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::OnceLock` instead
   = note: shared static variables must have a type that implements `Sync`
```

- **읽는 법**: 마지막 `note:`가 핵심입니다. `static` 변수의 타입은 `Sync`를 구현해야 하는데 `OnceCell`은 그렇지 않다는 뜻입니다.
- **원인**: `static`([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md))은 프로그램 전체에 하나뿐이고 어느 스레드에서든 접근할 수 있습니다. 두 스레드가 동시에 `get_or_init`을 부르면 둘 다 "비어 있다"고 보고 동시에 쓰려 할 수 있으므로, 단일 스레드 전용인 셀 타입은 `static`에 둘 수 없습니다. `static COUNTER: Cell<u32>`도 같은 E0277이고, 이때 `note:`는 `std::sync::RwLock`이나 `std::sync::atomic::AtomicU32`를 제안합니다.
- **해결**: `note:`대로 `std::sync::OnceLock<String>`을 씁니다. 메서드 이름(`get_or_init`, `set`, `get`)이 같습니다. 전역 값이 꼭 필요한 것이 아니라면 `OnceCell`을 지역 변수나 구조체 필드에 둡니다. `OnceLock`, `LazyLock`, 원자 타입은 `ch11-threads-sync`에서 다룹니다.

### 4.5 실행 중 패닉: 대여 가드가 생각보다 오래 살아 있음

컴파일은 되지만 실행하면 `RefCell already borrowed`(또는 `RefCell already mutably borrowed`)로 패닉하고 종료 코드 101로 끝나는 경우입니다. 모두 이 툴체인에서 직접 실행해 확인했습니다.

| 코드 모양 | 왜 패닉하는가 | 바로잡기 |
| --- | --- | --- |
| `while let Some(t) = q.borrow_mut().pop_front() { q.borrow_mut().push_back(..) }` | 조건식의 `RefMut`가 본문 끝까지 살아 있음 | `let next = q.borrow_mut().pop_front();`로 먼저 꺼냄 (2.10절) |
| `match memo.borrow().get(&n) { Some(&v) => v, None => { /* 재귀 */ memo.borrow_mut().insert(..) } }` | `match` 조건식의 `Ref`가 `None` 갈래에서도 살아 있음 | `let cached = memo.borrow().get(&n).copied();`로 먼저 꺼냄 |
| `println!("{} {}", c.try_borrow_mut().is_ok(), c.borrow().len())` | 한 문장 안의 임시값은 문장 끝까지 살아 있음 | 값을 `let`으로 하나씩 꺼낸 뒤 출력 |
| `for_each`처럼 가드를 쥔 메서드가 부른 콜백이 같은 `RefCell`을 `borrow_mut()` | 재진입 | `try_borrow_mut()`으로 실패를 처리하거나, `take()`로 꺼낸 뒤 순회 |

### 4.6 그 밖의 실수

| 실수 | 결과 | 바로잡기 |
| --- | --- | --- |
| `static`이 안 되니 `const COUNTER: Cell<u32> = Cell::new(0);`으로 바꿈 | rustc 경고 `const_item_interior_mutations`: "each usage of a `const` item creates a new temporary". 쓸 때마다 새 `Cell`이 생겨 `set`이 아무 효과가 없음 | 전역 상태는 `ch11-threads-sync`의 원자 타입·`OnceLock`. 경고가 제안하는 `static`은 4.4절의 E0277이 됨 |
| `mem::swap(&mut v[0], &mut v[1])` | E0499(`cannot borrow v as mutable more than once at a time`) | `v.swap(0, 1)` |
| 디버그 빌드에서 `Box::new([0u8; 16 * 1024 * 1024])` | 실행 중 `has overflowed its stack` | `vec![0u8; N].into_boxed_slice()` (2.7절) |
| `mem::replace(&mut v, Vec::new())`, `mem::replace(&mut opt, None)` | clippy 경고 (2.5절) | `mem::take(&mut v)`, `opt.take()` |
| 기본 표현 구조체의 필드 오프셋이나 `bool`/`char`의 니치에 기대는 코드 | 지금은 동작해도 컴파일러가 바뀌면 깨질 수 있음 | 보장되는 것은 문서의 목록뿐 (2.2, 2.3절) |

## 5. 연습 문제

### 연습 1. 크기 예측하기

아래 타입의 크기를 **먼저 손으로 계산한 뒤** `size_of`로 확인하세요.

```rust
struct Sample { a: u8, b: u64, c: u8, d: u32 }
// Sample, Option<Sample>, (bool, u32), Option<(bool, u32)>
```

`Option<Sample>`과 `Option<(bool, u32)>` 중 어느 쪽이 원래 타입보다 커지는지, 왜 그런지 설명해 보세요.

<details>
<summary>힌트</summary>

- `Sample`의 필드 합은 1 + 8 + 1 + 4 = 14바이트, 메모리 정렬은 `u64`를 따라 8입니다. 컴파일러가 순서를 바꾸더라도 14를 8의 배수로 올리면 16입니다.
- `Option<Sample>`은 24바이트입니다. `u8`, `u64`, `u32`는 모든 비트 조합이 올바른 값이라 니치가 없습니다. **패딩 바이트는 니치가 아닙니다.** 그래서 태그 칸이 붙고, 정렬 8에 맞춰 8바이트가 늘어납니다.
- `(bool, u32)`는 8바이트이고 `Option<(bool, u32)>`도 8바이트입니다. `bool` 필드의 2~255가 니치로 쓰입니다. 이 결과는 이 툴체인에서 확인한 것이며 보장되지 않습니다.

</details>

### 연습 2. `&mut self`로 cons 리스트 고치기

Part 0 ch14의 `enum List { Cons(i32, Box<List>), Nil }`에 다음 메서드를 작성하세요.

- `fn push_front(&mut self, value: i32)`: 맨 앞에 칸을 하나 붙입니다.
- `fn pop_front(&mut self) -> Option<i32>`: 맨 앞 칸을 떼어 값을 돌려줍니다. 빈 리스트면 `None`입니다.

`Nil`에서 시작해 `push_front(3)`, `push_front(2)`, `push_front(1)` 뒤 디버그 형식으로 출력하면 `Cons(1, Cons(2, Cons(3, Nil)))`, 이어서 `pop_front()`는 `Some(1)`이어야 합니다.

<details>
<summary>힌트</summary>

- 둘 다 `self` 자리에 있는 리스트 전체를 꺼내야 합니다. `*self`를 그냥 옮기면 E0507이므로, `mem::replace(self, Nil)`로 `Nil`을 넣고 예전 리스트를 꺼냅니다. `Nil`은 자리 표시 값으로 딱 맞습니다. 힙을 할당하지 않습니다.
- `push_front`: `let old = mem::replace(self, Nil); *self = Cons(value, Box::new(old));`
- `pop_front`: `match mem::replace(self, Nil) { Cons(v, next) => { *self = *next; Some(v) } Nil => None }`. `*next`는 `Box<List>`를 역참조해 안의 `List`를 꺼냅니다(`Box`는 `*`로 안의 값을 옮겨 낼 수 있는 특별한 스마트 포인터입니다).

</details>

### 연습 3. 계산 결과를 기억하는 피보나치

`&self`만 받는 `fn get(&self, n: u64) -> u64`로 피보나치 수를 구하는 `Fib`를 만드세요. 계산한 결과는 `memo: RefCell<HashMap<u64, u64>>`에 기억하고, `get`이 몇 번 불렸는지는 `calls: Cell<u32>`로 셉니다. `get(50)`이 `12586269025`이고 호출 횟수가 수십 번 수준인지 확인하세요.

<details>
<summary>힌트</summary>

- 왜 `memo`는 `RefCell`이고 `calls`는 `Cell`인지 2.12절의 표로 설명해 보세요. `HashMap`은 제자리에서 고쳐야 하고, 호출 횟수는 `Copy`인 정수입니다.
- 함정: `match self.memo.borrow().get(&n) { Some(&v) => v, None => { 재귀 호출 후 self.memo.borrow_mut().insert(..) } }`로 쓰면 `None` 갈래에서도 `Ref`가 살아 있어 `RefCell already borrowed`로 패닉합니다(4.5절).
- `let cached = self.memo.borrow().get(&n).copied();`로 먼저 꺼내 가드를 끝내고, `if let Some(v) = cached { return v; }` 다음에 재귀로 계산해 `borrow_mut().insert(n, v)`로 넣습니다. 이렇게 쓰면 `get(50)`의 호출 횟수는 99번입니다.

</details>

### 연습 4. 값싼 되돌리기 기록

문서 편집기의 되돌리기 기록을 만드세요.

```rust
struct Doc {
    current: Rc<Vec<String>>,
    history: Vec<Rc<Vec<String>>>,
}
```

- `fn save(&mut self)`: 지금 상태를 `history`에 저장합니다. 줄을 복사하지 않아야 합니다.
- `fn edit(&mut self, line: &str)`: `current`에 줄을 하나 추가합니다. 저장해 둔 기록은 바뀌면 안 됩니다.

`edit("a")`, `edit("b")`, `save()`, `edit("c")`, `edit("d")`, `save()`, `edit("e")` 순서로 부른 뒤 `current`와 `history`를 출력하고, 실제로 `Vec`이 복제된 횟수를 세어 보세요.

<details>
<summary>힌트</summary>

- `save`는 `self.history.push(Rc::clone(&self.current))`입니다. 카운트만 늘어납니다.
- `edit`은 `Rc::make_mut(&mut self.current).push(line.to_string())`입니다. 방금 저장해서 공유 중이면 복제한 뒤 고치고, 혼자면 제자리에서 고칩니다(2.8절).
- 복제 횟수는 `make_mut`을 부르기 전에 `Rc::strong_count(&self.current) > 1`인지 보면 셀 수 있습니다. 위 순서에서는 `save` 직후의 `edit`에서만 복제가 일어나 2번입니다. 결과는 `current = ["a", "b", "c", "d", "e"]`, `history = [["a", "b"], ["a", "b", "c", "d"]]`입니다.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::mem`](https://doc.rust-lang.org/std/mem/index.html): 모듈 첫머리에서 함수들을 가족별로 나눕니다(2.6절). [`size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html)에는 원시 타입의 크기 표와 `#[repr(C)]` 구조체의 크기 계산법이 있습니다.
  - [`std::option`의 Representation 절](https://doc.rust-lang.org/std/option/index.html#representation): `Option<T>`가 `T`와 같은 크기임을 보장하는 타입 목록(2.3절).
  - [`std::boxed`](https://doc.rust-lang.org/std/boxed/index.html): 재귀 타입 예제와 "Memory layout" 절. `Option<Box<T>>`로 C의 null 가능 포인터를 나타내는 예가 있습니다.
  - [`std::rc`](https://doc.rust-lang.org/std/rc/index.html), [`Rc`](https://doc.rust-lang.org/std/rc/struct.Rc.html), [`Weak`](https://doc.rust-lang.org/std/rc/struct.Weak.html): `make_mut`, `get_mut`, `try_unwrap`, `into_inner`, `unwrap_or_clone`, `new_cyclic`의 설명과 예제(2.8절).
  - [`std::cell`](https://doc.rust-lang.org/std/cell/index.html): 네 셀 타입의 비교와 [When to choose interior mutability](https://doc.rust-lang.org/std/cell/index.html#when-to-choose-interior-mutability) 절(2.12절).
  - [`Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html), [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html), [`OnceCell`](https://doc.rust-lang.org/std/cell/struct.OnceCell.html), [`LazyCell`](https://doc.rust-lang.org/std/cell/struct.LazyCell.html): 메서드마다 붙은 바운드(`T: Copy`, `T: Default`)와 안정화 버전을 확인해 보세요.
- Rust By Example
  - [Box, stack and heap](https://doc.rust-lang.org/rust-by-example/std/box.html): 구조체를 `Box`에 담고 `mem::size_of_val`로 스택에 있는 값과 `Box`의 크기를 비교합니다(2.1절의 `size_of_val`).
  - [Rc](https://doc.rust-lang.org/rust-by-example/std/rc.html): `Rc::clone`에 따른 `strong_count` 변화와, `==`가 안의 값을 비교한다는 것을 보여 줍니다(2.8절의 `ptr_eq`와 비교해 보세요).
- 에러 설명: `rustc --explain E0507`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
