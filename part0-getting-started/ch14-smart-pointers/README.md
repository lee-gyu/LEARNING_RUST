# ch14. 스마트 포인터

지금까지 값에는 소유자가 하나뿐이었고, 대여 규칙은 컴파일러가 모두 검사했습니다. 이 규칙만으로는 표현하기 어려운 구조가 있습니다. 자기 자신을 품는 재귀 타입, 여러 곳이 함께 소유해야 하는 값, `&self`로만 접근할 수 있는데 안을 바꿔야 하는 값, 서로를 가리키는 트리의 부모와 자식이 그런 예입니다. 이 챕터에서는 표준 라이브러리의 **스마트 포인터(smart pointer)** 로 이런 구조를 만들면서도 메모리 안전성을 지키는 방법을 배웁니다.

이 저장소의 `ch14`는 The Book의 **15장**에 해당합니다. Book 14장(Cargo와 crates.io)을 Part 3으로 옮겼기 때문에 이 챕터부터 저장소 챕터 번호와 Book 장 번호가 하나씩 어긋납니다.

- 패키지: `p0-ch14-smart-pointers`
- 기준 문서: [The Book 15장 Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- 확인한 툴체인: rustc 1.98.1, Edition 2024

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 스마트 포인터가 참조와 어떻게 다른지(데이터를 소유하고 `Deref`·`Drop`을 구현하는 구조체) 설명할 수 있다. `Box<T>`로 값을 힙에 두고, 재귀 타입에 `Box`가 필요한 이유를 크기 계산으로 설명할 수 있다.
2. 직접 만든 타입에 `Deref`와 `DerefMut`를 구현할 수 있고, `*` 연산자와 역참조 강제 변환이 어떤 단계를 거쳐 일어나는지 설명할 수 있다.
3. `Drop`을 구현해 정리 코드를 넣고 값이 버려지는 순서를 예측할 수 있다. `.drop()`을 직접 부를 수 없는 이유를 설명하고 `std::mem::drop`으로 값을 일찍 버릴 수 있다.
4. `Rc<T>`로 값 하나에 소유자를 여럿 두고, `RefCell<T>`로 대여 규칙을 실행 시점에 검사하는 내부 가변성을 쓸 수 있다. `Box`·`Rc`·`RefCell` 중 상황에 맞는 것을 고를 수 있다.
5. `Rc`로 만든 순환 참조가 메모리 누수를 일으키는 과정을 카운트로 설명할 수 있고, `Weak<T>`로 부모를 가리키는 트리를 만들 수 있다.

## 2. 핵심 개념

2.1~2.3절은 `Box<T>`, 2.4~2.5절은 `Deref`, 2.6절은 `Drop`, 2.7~2.9절은 `Rc<T>`와 `RefCell<T>`, 2.10~2.11절은 순환 참조와 `Weak<T>`를 다룹니다.

### 2.1 스마트 포인터란

**포인터(pointer)** 는 메모리 주소를 담고 있어서 다른 곳의 데이터를 가리키는 값을 통틀어 부르는 말입니다. Rust에서 가장 흔한 포인터는 ch4에서 배운 **참조** `&`입니다. 참조는 데이터를 빌리기만 하고, 가리키는 일 말고는 아무 기능도 없으며 그만큼 비용도 없습니다.

**스마트 포인터** 는 포인터처럼 동작하면서 추가 정보와 기능을 가진 데이터 구조입니다. C++에서 시작된 개념으로, Rust의 스마트 포인터는 보통 다음과 같은 **구조체**입니다.

- **가리키는 데이터를 소유합니다.** 참조는 빌리기만 하지만, 스마트 포인터는 대부분 데이터의 소유자입니다.
- **`Deref` 트레이트를 구현합니다.** 그래서 `*`로 따라가거나 참조가 필요한 자리에 넘길 수 있어, 참조처럼 쓸 수 있습니다(2.4절).
- **`Drop` 트레이트를 구현합니다.** 그래서 버려질 때 힙 메모리 해제 같은 정리 작업을 스스로 합니다(2.6절).

사실 이미 스마트 포인터를 써 왔습니다. **`String`과 `Vec<T>`** 입니다. 둘 다 힙에 있는 데이터를 소유하고, 길이와 용량이라는 추가 정보를 가지며, `Deref`를 구현해 `&String`이 `&str` 자리에, `&Vec<T>`가 `&[T]` 자리에 들어가고(2.5절), `Drop`을 구현해 버려질 때 힙 메모리를 돌려줍니다. `String`은 여기에 "내용이 항상 올바른 UTF-8"이라는 보장까지 더합니다.

| | 참조 `&T` | 스마트 포인터 (`Box<T>`, `String` 등) |
| --- | --- | --- |
| 데이터 | 빌림 | 대개 소유 |
| 정체 | 언어에 내장된 주소 값 | `Deref`·`Drop`을 구현한 구조체 |
| 추가 기능 | 없음 | 타입마다 다름 (힙 할당, 참조 카운트, 실행 중 대여 검사 등) |
| 버려질 때 | 아무 일 없음 | `Drop` 구현이 정리 작업을 함 |

이 챕터에서 다루는 표준 라이브러리 스마트 포인터는 다음과 같습니다.

| 타입 | 한 줄 요약 | 절 |
| --- | --- | --- |
| `Box<T>` | 값을 힙에 두고 소유하는 가장 단순한 스마트 포인터 | 2.2~2.3 |
| `Rc<T>` | 참조 카운트로 값 하나에 소유자를 여럿 둠 | 2.7 |
| `RefCell<T>` (`Ref<T>`, `RefMut<T>`) | 대여 규칙을 실행 시점에 검사해, 불변인 겉모습 안의 값을 바꿀 수 있게 함 | 2.8~2.9 |
| `Weak<T>` | 값을 소유하지 않고 가리키기만 하는 `Rc`의 짝 | 2.11 |

### 2.2 `Box<T>`: 값을 힙에 두기

`Box::new(값)`은 값을 힙에 옮겨 두고, 그 위치를 가리키는 `Box`를 돌려줍니다. 스택에는 포인터 한 칸만 남습니다.

```rust
{
    let b = Box::new(5);
    println!("b = {b}");    // b = 5. 출력하면 안의 값이 나옵니다.
    println!("{}", *b + 1); // 6. 계산할 때는 *로 따라가 꺼냅니다.
} // b가 버려지면 Box의 Drop 구현이 힙의 5를 해제합니다.
```

```
  스택 (b: Box<i32>)          힙
  +-----+                    +---+
  | ptr |------------------> | 5 |
  +-----+                    +---+
```

`Box` 자체의 크기는 가리키는 데이터와 상관없이 포인터 한 칸(64비트 환경에서 8바이트)입니다. `Box<i32>`도, 1만 바이트짜리 배열을 담은 `Box<[u8; 10000]>`도 8바이트이고, 참조 `&i32`와 크기가 같습니다. 참조와 `Box`의 차이는 크기가 아니라 **소유하는가**에 있습니다.

`Box`는 힙 할당과 간접 참조(indirection, 값을 직접 담지 않고 가리키기만 하는 것) 말고는 특별한 기능이 없고, 그만큼 추가 비용도 없습니다. `Box`가 필요한 대표적인 상황은 세 가지입니다.

1. **컴파일 시점에 크기를 알 수 없는 타입**을, 정확한 크기가 필요한 자리에 써야 할 때. 재귀 타입이 대표적입니다(2.3절).
2. **큰 데이터의 소유권을 복사 없이 넘기고 싶을 때.** 큰 배열을 그대로 이동하면 스택에서 배열 전체가 복사될 수 있지만, `Box`에 담아 두면 포인터 한 칸만 복사되고 데이터는 힙의 제자리에 있습니다. [`01_box_basics.rs`](examples/01_box_basics.rs)는 이동 전후의 힙 주소가 같다는 것을 `true`로 확인합니다.
3. **구체적인 타입이 아니라 "어떤 트레이트를 구현했는가"만 중요할 때.** ch9에서 본 `Box<dyn Error>`가 이 경우입니다. `dyn Trait`(트레이트 객체)는 `ch17-trait-objects-oop`에서 배웁니다.

정수 하나를 `Box`에 담는 것은 보통 의미가 없습니다. 값은 기본적으로 스택에 두는 편이 빠르고 간단하며, `Box`는 위 세 가지처럼 이유가 있을 때 씁니다.

### 2.3 재귀 타입과 cons 리스트

**재귀 타입(recursive type)** 은 자기 자신과 같은 타입의 값을 일부로 품는 타입입니다. 대표 예로 Lisp에서 온 **cons 리스트**를 만들어 봅니다. 각 칸이 (값, 다음 칸) 쌍이고, 마지막 칸은 `Nil`입니다. `1, 2, 3`은 `Cons(1, Cons(2, Cons(3, Nil)))`이 됩니다. (실제로 목록이 필요할 때는 `Vec<T>`가 거의 항상 더 낫습니다. cons 리스트는 재귀 타입을 설명하기 좋은 예일 뿐입니다.)

가장 자연스러워 보이는 정의는 컴파일되지 않습니다.

```rust
enum List {
    Cons(i32, List),   // E0072: recursive type `List` has infinite size
    Nil,
}
```

#### 열거형의 크기는 어떻게 정해지나

Rust는 값을 스택에 두려면 그 타입의 크기를 컴파일할 때 알아야 합니다. 열거형의 값은 한 번에 배리언트 하나만 담으므로, 크기는 **가장 큰 배리언트를 담을 공간 + 어느 배리언트인지 적는 태그(tag)** 입니다.

```rust
enum Shape {
    Dot,             // 0바이트
    Circle(i32),     // 4바이트
    Rect(i32, i32),  // 8바이트  ← 가장 큼
}
// 8바이트 + 태그 = 12바이트 (태그는 i32에 맞춰 4바이트 단위로 자리를 잡습니다)
```

같은 계산을 위의 `List`에 적용하면 끝나지 않습니다.

```
size(List) = size(Cons)
           = size(i32) + size(List)
           = size(i32) + size(i32) + size(List)
           = size(i32) + size(i32) + size(i32) + size(List)
           = ...  (영원히 끝나지 않음)
```

#### `Box`가 해결책인 이유

`Box<List>`는 무엇을 가리키든 포인터 한 칸이므로 크기가 정해져 있습니다. 다음 칸을 `List`로 직접 담지 않고 `Box<List>`로 가리키게 하면, 계산이 한 단계에서 끝납니다.

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}

let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
```

```
size(List) = size(Cons) = i32(4) + Box<List>(8) + 빈칸(4) = 16바이트
```

빈칸 4바이트는 정렬(alignment) 때문에 생깁니다. 8바이트짜리 포인터는 8의 배수 위치에 놓여야 하므로 `i32` 뒤를 비워 둡니다. 태그 칸이 따로 없는 것은 `Box`가 절대 주소 0(null)을 갖지 않는다는 점을 이용해, 컴파일러가 "포인터 자리가 0이면 `Nil`"로 구분하기 때문입니다. 이런 메모리 배치의 세부는 Part 1 `ch7-smart-pointers-memory`에서 다시 다룹니다.

리스트가 길어져도 `List` 값 하나의 크기는 16바이트 그대로입니다. 아래 그림에서 맨 왼쪽 칸(변수 `list`)만 스택에 있고, 나머지 칸은 모두 힙에 있습니다.

```
  list
  +---+-----+         +---+-----+          +---+-----+          +-----+
  | 1 | ptr |-------> | 2 | ptr |--------> | 3 | ptr |--------> | Nil |
  +---+-----+         +---+-----+          +---+-----+          +-----+
```

재귀 타입은 재귀 함수로 다루는 것이 자연스럽습니다. [`02_cons_list.rs`](examples/02_cons_list.rs)의 `sum`은 `Cons(value, rest)`를 만나면 `value + sum(rest)`를, `Nil`을 만나면 0을 돌려줍니다. 여기서 `rest`의 타입은 `&Box<List>`인데 `sum`은 `&List`를 받습니다. 그래도 컴파일되는 이유가 다음 절의 주제입니다.

### 2.4 `Deref`: 내 타입에 `*` 쓰기

참조는 `*`(역참조 연산자)로 따라가 가리키는 값을 꺼냅니다. `Box`도 똑같이 쓸 수 있습니다.

```rust
let x = 5;
let y = &x;
let b = Box::new(x);
assert_eq!(5, *y); // y는 &i32이므로 *로 따라가야 i32와 비교할 수 있습니다.
assert_eq!(5, *b); // Box도 같은 방식입니다.
```

`Box`처럼 값 하나를 감싸는 `MyBox<T>`를 직접 만들어 보면, `*`가 저절로 되는 것이 아니라는 것을 알 수 있습니다. (진짜 `Box`와 달리 이 `MyBox`는 값을 힙이 아니라 자기 안에 둡니다. 여기서는 `Deref`만 보면 되므로 상관없습니다.)

```rust
struct MyBox<T>(T);

let y = MyBox::new(5);
assert_eq!(5, *y); // E0614: type `MyBox<{integer}>` cannot be dereferenced
```

컴파일러는 `&` 참조를 따라가는 법만 압니다. 다른 타입에 `*`를 쓰려면 **`std::ops::Deref` 트레이트**로 "따라가면 무엇이 나오는지"를 알려 줘야 합니다.

```rust
use std::ops::Deref;

impl<T> Deref for MyBox<T> {
    type Target = T;              // *로 따라가면 나오는 타입

    fn deref(&self) -> &Self::Target {
        &self.0                    // 안의 값을 가리키는 참조
    }
}
```

- `type Target = T;`는 트레이트 안에서 쓸 타입 이름을 정하는 문법입니다. 연관 타입(associated type)이라고 부르며 `ch19-advanced-features`에서 자세히 다룹니다.
- 이제 `*y`라고 쓰면 컴파일러는 이것을 **`*(y.deref())`** 로 바꿉니다. `deref`가 안의 값의 참조 `&i32`를 돌려주고, 바깥의 평범한 `*`가 그 참조를 따라갑니다. 이 치환은 `*` 하나당 한 번만 일어나므로 끝없이 반복되지 않습니다.
- `deref`가 값이 아니라 **참조**를 돌려주는 이유는 소유권입니다. 값을 돌려주면 안의 `T`가 `MyBox` 밖으로 이동해 버립니다. `*y`로 값을 들여다볼 때마다 `y`가 비어 버리면 안 됩니다.

### 2.5 역참조 강제 변환

**역참조 강제 변환(deref coercion)** 은 `Deref`를 구현한 타입의 참조를, 그 타입의 `Target`의 참조로 컴파일러가 자동으로 바꿔 주는 기능입니다. 함수와 메서드의 인자에서 타입이 맞지 않을 때 일어나고, 필요하면 여러 번 이어서 일어납니다.

```rust
fn hello(name: &str) {
    println!("Hello, {name}!");
}

let m = MyBox::new(String::from("Rust"));
hello(&m);
```

`hello`는 `&str`을 받는데 `&MyBox<String>`을 넘겼습니다. 컴파일러는 다음 순서로 타입을 맞춥니다.

```
&MyBox<String>  --MyBox의 deref-->  &String  --String의 deref-->  &str
```

강제 변환이 없다면 `hello(&(*m)[..])`처럼 써야 합니다. `(*m)`으로 `MyBox<String>`을 따라가 `String`을 얻고, `&...[..]`로 전체 슬라이스를 만드는 식입니다. 읽기도 쓰기도 번거롭습니다. `deref`를 몇 번 부를지는 컴파일할 때 정해지므로 **실행 비용은 없습니다.**

이미 이 기능을 여러 번 만났습니다.

- ch4에서 `&String`을 `&str` 매개변수에 넘겼을 때: `String`이 `Deref<Target = str>`을 구현합니다.
- ch8에서 `&Vec<T>`를 `&[T]` 자리에 넘겼을 때: `Vec<T>`가 `Deref<Target = [T]>`를 구현합니다.
- 2.3절의 `sum(rest)`: `&Box<List>`가 `&List`로 바뀝니다.

메서드 호출에서도 같은 일이 일어납니다. `m.len()`에서 `MyBox`에는 `len`이 없지만, 컴파일러가 [ch5 2.10절의 자동 참조·역참조](../ch5-structs/README.md)를 하면서 `deref`를 따라가 `String`의 `len`을 찾아 부릅니다.

#### `DerefMut`와 세 가지 규칙

`Deref`가 `&`에 대한 `*`를 정한다면, **`DerefMut`** 는 `&mut`에 대한 `*`를 정합니다. `DerefMut`를 구현하면 `*counter += 10`처럼 안의 값을 바꿀 수 있습니다. 역참조 강제 변환은 다음 세 경우에 일어납니다.

| | 바꾸기 전 | 바꾼 뒤 | 조건 |
| --- | --- | --- | --- |
| 1 | `&T` | `&U` | `T: Deref<Target = U>` |
| 2 | `&mut T` | `&mut U` | `T: DerefMut<Target = U>` |
| 3 | `&mut T` | `&U` | `T: Deref<Target = U>` |

세 번째 규칙은 가변 참조를 불변 참조로 바꿔도 된다는 뜻입니다. 반대로 **불변 참조를 가변 참조로 바꾸는 일은 없습니다.** 가변 참조는 그 데이터의 유일한 참조이므로 불변 참조로 낮춰도 대여 규칙이 깨지지 않습니다. 하지만 불변 참조는 다른 불변 참조와 함께 있을 수 있으므로, 이것을 가변 참조로 올리면 "가변 참조는 하나뿐"이라는 규칙이 깨질 수 있습니다. 실제로 `&mut String`을 받는 함수에 `&MyBox<String>`을 넘기면 E0308(`mismatched types`)로 거부되고, 메시지는 원인을 `types differ in mutability`(가변성이 다른 타입)라고 짚습니다.

마찬가지로 `DerefMut`를 구현하지 않은 타입을 통해 안의 값을 바꾸려 하면 E0596과 함께 ``trait `DerefMut` is required to modify through a dereference``라는 도움말이 나옵니다. 2.7절의 `Rc`가 바로 이 경우입니다.

### 2.6 `Drop`: 버려질 때 실행할 코드

ch4부터 관찰용으로 써 온 `Noisy`의 `impl Drop`을 이제 제대로 읽어 봅니다. **`Drop` 트레이트**는 값이 스코프를 벗어날 때 실행할 코드를 정합니다. 메서드는 `drop` 하나이고 `&mut self`를 받습니다.

```rust
struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("[drop] {}", self.data);
    }
}
```

스마트 포인터는 거의 언제나 `Drop`을 구현합니다. `Box`는 힙 메모리를 해제하고, 다음 절의 `Rc`는 참조 카운트를 줄입니다. 파일이나 네트워크 연결, 잠금(lock)을 닫는 데도 씁니다. 이 정리 코드는 **Rust가 알아서 부르므로**, 쓰는 쪽은 정리를 잊을 수도 없고 같은 정리를 여러 곳에 복사할 필요도 없습니다.

#### drop 순서

- 같은 스코프의 변수는 **만든 역순**으로 버려집니다(ch4에서 본 것과 같습니다).
- 구조체 값이 버려질 때는 **구조체 자신의 `drop`이 먼저** 실행되고, 그다음 **필드가 선언 순서대로** 버려집니다. 그래서 `drop` 안에서 아직 필드를 읽을 수 있습니다.
- `Box`가 버려지면 **안의 값이 먼저 drop되고**, 그 뒤에 힙 메모리가 해제됩니다.

[`04_drop.rs`](examples/04_drop.rs)에서 세 가지를 모두 출력으로 확인합니다.

#### `.drop()`을 직접 부를 수 없는 이유

값을 일찍 정리하고 싶어서 `c.drop()`을 부르면 컴파일되지 않습니다(E0040, 4.3절). 직접 부른 뒤에도 Rust는 스코프 끝에서 `c`의 `drop`을 **또** 부를 것이고, 그러면 같은 값을 두 번 정리하는 이중 해제가 됩니다. 에러 메시지의 destructor(소멸자)는 값을 정리하는 함수를 부르는 일반적인 이름으로, 값을 만드는 생성자(constructor)의 반대말입니다. Rust의 소멸자가 `drop`입니다.

#### `std::mem::drop`으로 일찍 버리기

자동 drop을 끌 수는 없습니다. 대신 **`std::mem::drop` 함수**에 값을 넘깁니다. 프렐류드에 들어 있어 `use` 없이 `drop(c)`로 씁니다.

```rust
let c = CustomSmartPointer { data: String::from("some data") };
drop(c);       // 여기서 [drop] some data가 출력됩니다.
// println!("{}", c.data); // E0382: c는 drop 함수로 이동했습니다.
```

이 함수에는 마법이 없습니다. 표준 라이브러리 문서에 적힌 정의가 말 그대로 `pub fn drop<T>(_x: T) {}`입니다. 값을 인자로 받아 소유권을 가져온 뒤 아무것도 하지 않고 끝나면, 매개변수 `_x`가 스코프를 벗어나며 버려집니다. ch4의 "함수로 이동한 값은 그 함수가 끝날 때 drop된다"를 그대로 이용한 것입니다. 그리고 `c`는 이미 이동했으므로 스코프 끝에서 다시 drop되지 않습니다. **소유권이 "drop은 정확히 한 번"을 보장합니다.**

일찍 버리는 것이 쓸모 있는 대표적인 경우는 빌린 것을 일찍 돌려줄 때입니다. 2.8절의 `RefCell`에서 대여를 일찍 끝낼 때, `ch15-concurrency`에서 잠금을 일찍 풀 때 씁니다.

### 2.7 `Rc<T>`: 소유자가 여럿인 값

대부분의 값은 누가 소유하는지가 분명합니다. 그런데 그래프에서 여러 간선이 같은 노드를 가리키는 것처럼, 값 하나를 **여러 곳이 함께 소유해야** 하는 경우가 있습니다. 그 노드는 자신을 가리키는 간선이 하나도 남지 않았을 때 정리되어야 합니다.

cons 리스트로 보면, 리스트 `a`(5, 10)를 꼬리로 공유하는 두 리스트 `b`(3으로 시작), `c`(4로 시작)를 만들고 싶습니다.

```
  b: 3 --+
         +--> a: 5 -> 10 -> Nil
  c: 4 --+
```

`Box`로는 안 됩니다. `Box`는 안의 값을 소유하므로 `Box::new(a)`로 `a`가 `b` 안으로 이동하고, `c`를 만들 때는 이미 `a`가 없습니다(E0382, 4.4절). 참조로 바꾸면 라이프타임을 적어야 하고 "모든 원소가 리스트보다 오래 산다"는 조건이 생기는데, 그런 조건을 만족하지 못하는 상황도 많습니다.

**`Rc<T>`(reference counting, 참조 카운팅)** 는 값과 함께 **"지금 소유자가 몇인지"(strong count)** 를 힙에 두는 스마트 포인터입니다.

```rust
use std::rc::Rc; // 프렐류드에 없으므로 가져옵니다.

enum List {
    Cons(i32, Rc<List>),
    Nil,
}

let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil))))); // 카운트 1
let b = Cons(3, Rc::clone(&a));                              // 카운트 2
let c = Cons(4, Rc::clone(&a));                              // 카운트 3
```

- **`Rc::clone(&a)`는 깊은 복사가 아닙니다.** 데이터는 그대로 두고 카운트만 1 늘린 새 `Rc`를 만듭니다. 그래서 값쌉니다. `a.clone()`도 같은 일을 하지만, 데이터를 통째로 복제하는 비싼 `clone`과 눈으로 구분하려고 `Rc::clone(&a)`로 쓰는 것이 관례입니다. 성능 문제를 찾을 때 `Rc::clone`은 건너뛰고 나머지 `clone`만 살펴보면 됩니다.
- **`Rc` 하나가 버려질 때마다 `Rc`의 `Drop` 구현이 카운트를 1 줄입니다.** 카운트가 0이 되는 순간 안의 값이 drop됩니다. TV를 처음 들어온 사람이 켜고, 마지막으로 나가는 사람이 끄는 것과 같습니다. 누가 마지막일지 컴파일할 때 알 수 없으므로 실행 중에 세는 것입니다. (누가 마지막인지 컴파일할 때 알 수 있다면, 그쪽을 소유자로 정하면 되므로 `Rc`가 필요 없습니다.)
- **`Rc::strong_count(&a)`** 로 현재 카운트를 볼 수 있습니다. `a.strong_count()`처럼 메서드로 부르지 않고 연관 함수로 부르는 것은, `Rc`가 `Deref`로 안의 값의 메서드를 그대로 노출하기 때문입니다. `Rc` 자신의 기능을 연관 함수로 두어 안의 타입의 메서드와 이름이 겹치지 않게 했습니다.

[`05_rc_shared_list.rs`](examples/05_rc_shared_list.rs)는 카운트가 1 → 2 → 3으로 늘었다가, `c`가 스코프를 벗어나면 2로 주는 것을 출력합니다. `Rc::ptr_eq`로 두 `Rc`가 같은 값을 가리킨다는 것(`true`)도 확인합니다.

`Rc<T>`에는 두 가지 제약이 있습니다.

1. **안의 값을 읽기만 할 수 있습니다.** `Rc`는 `&T`만 내주고 `DerefMut`를 구현하지 않습니다. 여러 소유자가 저마다 값을 바꿀 수 있다면 "가변 참조는 하나뿐"이라는 대여 규칙이 깨지기 때문입니다. `Rc<String>`에 `push_str`을 부르면 E0596(``cannot borrow data in an `Rc` as mutable``)으로 거부됩니다(4.5절). 바꿔야 한다면 2.9절의 `Rc<RefCell<T>>`를 씁니다.
2. **단일 스레드 전용입니다.** `Rc`는 카운트를 늘리고 줄이는 동작을 스레드 사이에서 안전하게 맞추지 않습니다. 대신 그만큼 빠릅니다. 다른 스레드로 넘기려 하면 컴파일러가 막습니다(``error[E0277]: `Rc<i32>` cannot be sent between threads safely``). 스레드 사이에서 공유할 때는 `ch15-concurrency`의 `Arc<T>`를 씁니다.

### 2.8 `RefCell<T>`와 내부 가변성

ch4의 대여 규칙을 다시 떠올려 봅니다.

1. 어느 시점에든 가변 참조 하나 **또는** 불변 참조 여러 개 중 한쪽만 존재할 수 있다.
2. 참조는 항상 유효해야 한다.

참조와 `Box<T>`는 이 규칙을 **컴파일 시점에** 검사합니다. 어기면 컴파일 에러입니다. **`RefCell<T>`** 는 같은 규칙을 **실행 시점에** 검사합니다. 어기면 패닉합니다.

왜 실행 시점 검사가 필요할까요? 컴파일러의 분석은 보수적입니다. 코드만 보고 안전하다고 **증명할 수 없으면** 실제로는 안전한 프로그램도 거부합니다. 잘못된 프로그램을 받아들이는 것보다 올바른 프로그램을 거부하는 쪽이 낫기 때문입니다. 대부분은 이 방식이 옳지만, 프로그래머는 규칙을 지킨다고 확신하는데 컴파일러가 확인할 수 없는 경우가 있습니다. 그때 `RefCell<T>`로 검사를 실행 시점으로 미룹니다.

이렇게 **불변 참조만 가지고도 안의 데이터를 바꿀 수 있게 하는 설계**를 **내부 가변성(interior mutability)** 이라고 부릅니다. `RefCell<T>`는 내부적으로 `unsafe` 코드(`ch19-advanced-features`)를 쓰지만, 그것을 대여 규칙을 지키는 안전한 API로 감싸 두었으므로 쓰는 쪽은 안전한 Rust만 씁니다.

```rust
use std::cell::RefCell;

let notes = RefCell::new(vec![String::from("첫 메모")]); // mut가 없습니다.
notes.borrow_mut().push(String::from("둘째 메모"));        // 그래도 바꿀 수 있습니다.
println!("{:?}", notes.borrow());
```

| 메서드 | 돌려주는 것 | 뜻 |
| --- | --- | --- |
| `borrow()` | `Ref<T>` | 불변 대여. `&T`처럼 씀 |
| `borrow_mut()` | `RefMut<T>` | 가변 대여. `&mut T`처럼 씀 |
| `try_borrow()` | `Result<Ref<T>, BorrowError>` | 대여할 수 없으면 패닉 대신 `Err` |
| `try_borrow_mut()` | `Result<RefMut<T>, BorrowMutError>` | 대여할 수 없으면 패닉 대신 `Err` |

`Ref<T>`와 `RefMut<T>`도 `Deref`를 구현한 스마트 포인터입니다. `RefCell`은 지금 살아 있는 `Ref`와 `RefMut`의 개수를 셉니다. `borrow()`를 부를 때마다 불변 대여 수가 1 늘고, `Ref`가 버려지면 1 줄어듭니다. 그래서 **`Ref`/`RefMut`가 버려지는 순간 대여가 끝납니다.** `notes.borrow_mut().push(...)`처럼 한 문장 안에서 쓰고 버린 `RefMut`는 그 문장이 끝날 때 대여도 끝나고, 변수에 담아 둔 `Ref`는 그 변수가 버려질 때(또는 `drop`으로 일찍 버릴 때) 끝납니다.

규칙을 어기는 코드는 **컴파일은 되지만 실행 중에 패닉합니다.**

```rust
let mut first = notes.borrow_mut();
let mut second = notes.borrow_mut(); // first가 살아 있으므로 여기서 패닉
```

```
thread 'main' (6405104) panicked at part0-getting-started/ch14-smart-pointers/examples/06_refcell.rs:87:32:
RefCell already borrowed
```

괄호 안의 스레드 ID는 실행할 때마다 달라집니다. 패닉 위치가 `RefCell` 내부가 아니라 `borrow_mut`를 부른 내 코드로 나오므로 어디서 규칙을 어겼는지 바로 찾을 수 있습니다. 패닉 대신 상황을 확인하고 싶다면 `try_borrow_mut()`를 씁니다. 읽는 중이면 `Err`가 돌아오고, 그 에러를 출력하면 `RefCell already borrowed`가, 쓰는 중에 `try_borrow()`를 부르면 `RefCell already mutably borrowed`가 나옵니다. [`06_refcell.rs`](examples/06_refcell.rs)는 평소에는 `try_` 메서드로 위반을 관찰하고, `-- panic` 인자를 줄 때만 실제로 패닉합니다.

실행 시점 검사의 대가는 두 가지입니다. 실수가 개발 중이 아니라 실행 중에야, 운이 나쁘면 배포한 뒤에야 드러날 수 있습니다. 그리고 대여 수를 세는 만큼 약간의 실행 비용이 듭니다. 그래서 `RefCell`은 컴파일 시점 검사로 표현할 수 없을 때만 씁니다.

#### 내부 가변성이 필요한 실제 예: 테스트용 가짜 객체

다음 트레이트의 `send`는 `&self`를 받습니다. 메시지를 보내는 일은 보내는 쪽의 상태를 바꾸지 않는다고 설계했기 때문입니다.

```rust
pub trait Messenger {
    fn send(&self, msg: &str);
}
```

사용량을 추적하다가 한도에 가까워지면 이 메신저로 알림을 보내는 `LimitTracker`를 테스트하려고 합니다. 진짜로 메일을 보낼 수는 없으니, 보내지 않고 **무엇을 보냈는지 기록만 하는 가짜 객체(mock object)** 를 만듭니다. 그런데 기록 칸을 그냥 `Vec<String>`으로 두면, `&self`로는 `push`할 수 없어서 컴파일되지 않습니다.

```
error[E0596]: cannot borrow `self.sent_messages` as mutable, as it is behind a `&` reference
```

컴파일러는 트레이트와 구현의 `&self`를 모두 `&mut self`로 바꾸라고 제안하지만, 테스트 편의를 위해 실제 설계를 바꾸는 것은 옳지 않습니다. 기록 칸을 `RefCell<Vec<String>>`으로 감싸면 `&self`인 채로 `self.sent_messages.borrow_mut().push(...)`로 기록할 수 있습니다. 바깥에서 보기에는 여전히 불변이고, 기록이라는 내부 사정만 바뀝니다. 전체 코드와 테스트는 [`07_mock_messenger.rs`](examples/07_mock_messenger.rs)에 있습니다.

### 2.9 `Rc<RefCell<T>>`와 선택 기준

`Rc<T>`는 소유자를 여럿 두지만 읽기만 할 수 있고, `RefCell<T>`는 안을 바꿀 수 있지만 소유자는 하나입니다. 둘을 겹친 **`Rc<RefCell<T>>`** 는 **여러 소유자가 함께 가지면서 바꿀 수도 있는 값**이 됩니다.

```rust
let value = Rc::new(RefCell::new(5));
let a = Rc::new(Cons(Rc::clone(&value), Rc::new(Nil)));
let b = Cons(Rc::new(RefCell::new(3)), Rc::clone(&a));
let c = Cons(Rc::new(RefCell::new(4)), Rc::clone(&a));

*value.borrow_mut() += 10; // a, b, c가 모두 15를 보게 됩니다.
```

`value.borrow_mut()`에서 `value`는 `Rc<RefCell<i32>>`입니다. 메서드 호출의 자동 역참조로 `Rc`를 지나 `RefCell`의 `borrow_mut`가 불리고, 돌려받은 `RefMut<i32>`를 `*`로 따라가 값을 바꿉니다.

세 타입의 선택 기준을 정리하면 다음과 같습니다.

| | `Box<T>` | `Rc<T>` | `RefCell<T>` |
| --- | --- | --- | --- |
| 소유자 수 | 하나 | 여럿 | 하나 |
| 대여 검사 시점 | 컴파일 시점 | 컴파일 시점 | 실행 시점 |
| 안의 값 수정 | `&mut`로 가능 | 불가 (`&T`만 내줌) | `borrow_mut()`로 가능 (`RefCell`이 불변이어도) |
| 규칙을 어기면 | 컴파일 에러 | 컴파일 에러 | 패닉 |
| 스레드 사이 공유용 짝 | (해당 없음) | `Arc<T>` (ch15) | `Mutex<T>` (ch15) |
| 이 챕터 예제 | 01, 02 | 05 | 06, 07 |

- 값 하나를 힙에 두거나 재귀 타입을 만들 때: `Box<T>`
- 누가 마지막까지 쓸지 모르는 값을 여럿이 읽을 때: `Rc<T>`
- `&self`밖에 없는데 안을 바꿔야 할 때: `RefCell<T>`
- 여럿이 함께 가지면서 바꿔야 할 때: `Rc<RefCell<T>>`

표준 라이브러리에는 내부 가변성을 주는 타입이 더 있습니다. 값을 통째로 넣고 빼는 `Cell<T>`, 한 번만 초기화하는 `OnceCell<T>`는 Part 1 `ch7-smart-pointers-memory`에서 다룹니다.

### 2.10 순환 참조와 메모리 누수

Rust의 메모리 안전성 보장은 **메모리 누수(memory leak)까지 막아 주지는 않습니다.** `Rc<T>`와 `RefCell<T>`를 함께 쓰면, 서로를 가리키는 **순환 참조(reference cycle)** 를 만들 수 있습니다. 순환 안의 값은 카운트가 절대 0이 되지 않아 drop되지 않습니다.

꼬리를 나중에 바꿀 수 있도록 `RefCell<Rc<List>>`로 둔 cons 리스트로 순환을 만들어 봅니다.

```rust
enum List {
    Cons(i32, RefCell<Rc<List>>),
    Nil,
}

let a = Rc::new(Cons(5, RefCell::new(Rc::new(Nil))));
let b = Rc::new(Cons(10, RefCell::new(Rc::clone(&a)))); // b → a
if let Some(link) = a.tail() {
    *link.borrow_mut() = Rc::clone(&b);                 // a → b. 순환 완성
}
```

```
  a ---> [Cons(5, tail)] ---tail---> [Cons(10, tail)] <--- b
           ^                                  |
           +---------------tail---------------+
```

이때 `a`와 `b`의 카운트는 각각 2입니다(변수 하나 + 상대의 꼬리(tail) 하나). 스코프가 끝나면 다음과 같이 됩니다.

| 단계 | `a`의 카운트 | `b`의 카운트 | 결과 |
| --- | --- | --- | --- |
| 순환을 만든 직후 | 2 | 2 | |
| 변수 `b`가 버려짐 | 2 | 1 | `b`의 값은 `a`의 꼬리가 붙잡고 있어 drop되지 않음 |
| 변수 `a`가 버려짐 | 1 | 1 | `a`의 값은 `b`의 꼬리가 붙잡고 있어 drop되지 않음 |

두 값은 서로를 붙잡은 채 아무도 접근할 수 없는 상태로 남습니다. 프로그램이 끝나면 운영체제가 메모리를 회수하지만, 그 전까지는 누구도 해제할 수 없습니다. [`08_cycle_and_weak.rs`](examples/08_cycle_and_weak.rs)는 `List`에 `Drop`을 구현해 두고, 순환이 없을 때는 `[drop] Cons(..)`가 찍히지만 순환을 만들면 블록이 끝나도 찍히지 않는 것을 비교합니다.

**순환을 출력하면 안 됩니다.** 디버그 형식 출력은 꼬리를 따라가며 내용을 출력하므로, 순환이 있으면 `Cons(10, ... Cons(5, ... Cons(10, ...`을 끝없이 출력하다 스택이 넘칩니다. 예제에는 이 줄을 주석으로만 남겨 두었습니다. 따로 실행해 보면 다음과 같이 비정상 종료합니다(macOS의 zsh에서 종료 코드 134).

```
thread 'main' (6388585) has overflowed its stack
fatal runtime error: stack overflow, aborting
```

컴파일러는 순환을 잡아 주지 않습니다. 설계할 때 **"어느 쪽이 어느 쪽을 소유하는가"** 를 정하고, 소유하지 않는 방향의 연결은 다음 절의 `Weak<T>`로 만들어 순환을 피합니다.

### 2.11 `Weak<T>`: 소유하지 않는 연결

`Rc::downgrade(&rc)`는 **`Weak<T>`** 를 돌려줍니다. `Weak`는 값을 가리키기만 하고 소유하지 않는 포인터입니다.

| | `Rc<T>` (강한 참조) | `Weak<T>` (약한 참조) |
| --- | --- | --- |
| 만드는 법 | `Rc::new`, `Rc::clone(&rc)` | `Rc::downgrade(&rc)`, `Weak::new()` |
| 늘리는 카운트 | `Rc::strong_count` | `Rc::weak_count` |
| 값을 살려 두는가 | 예. strong이 0이 되어야 drop | 아니요. weak가 남아 있어도 drop됨 |
| 안의 값 쓰기 | `Deref`로 바로 | `upgrade()`로 `Option<Rc<T>>`를 얻은 뒤 |

`Weak`가 가리키던 값은 이미 drop되었을 수 있으므로, 쓰기 전에 반드시 **`upgrade()`** 로 확인해야 합니다. 값이 살아 있으면 `Some(Rc<T>)`, 이미 사라졌으면 `None`입니다. `Option`으로 돌려주니 두 경우를 모두 처리하도록 컴파일러가 강제하고, 댕글링 포인터가 생길 수 없습니다.

#### 부모를 `Weak`로 가리키는 트리

자식 목록을 가진 노드로 트리를 만듭니다. 자식이 부모를 알 수 있도록 부모 칸도 둡니다.

```rust
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,      // 자식 → 부모: 소유하지 않음
    children: RefCell<Vec<Rc<Node>>>, // 부모 → 자식: 소유함
}
```

- **부모 → 자식은 `Rc`입니다.** 부모가 사라지면 자식도 사라져야 하므로 부모가 자식을 소유합니다. 자식 노드는 변수로도 붙잡아 둘 수 있어야 하므로 `Box`가 아니라 `Rc`입니다.
- **자식 → 부모는 `Weak`입니다.** 자식이 사라진다고 부모가 사라지면 안 되므로 자식은 부모를 소유하지 않습니다. 여기까지 `Rc`로 두면 2.10절의 순환이 됩니다.
- **둘 다 `RefCell`로 감쌌습니다.** 노드를 만든 뒤에 부모를 정하거나 자식을 추가하려면, `Rc` 안에 있어 불변인 노드의 칸을 바꿔야 하기 때문입니다.

`leaf`(값 3)를 만들고, 블록 안에서 `branch`(값 5)의 자식으로 넣은 뒤 블록을 벗어나면 카운트가 이렇게 바뀝니다.

| 시점 | `leaf` strong / weak | `branch` strong / weak | `leaf`의 부모 (`upgrade`) |
| --- | --- | --- | --- |
| `leaf`만 만든 뒤 | 1 / 0 | (없음) | `None` |
| `branch`의 자식이 된 뒤 | 2 / 0 | 1 / 1 | `Some(5)` |
| `branch`의 블록이 끝난 뒤 | 1 / 0 | (drop됨) | `None` |

`branch`의 weak 카운트가 1이어도 strong이 0이 되는 순간 `branch`는 drop되고, `branch`가 소유하던 `leaf`의 `Rc` 하나도 함께 버려져 `leaf`의 strong이 2에서 1로 돌아옵니다. 서로를 가리키지만 순환 소유가 아니므로 누수가 없습니다. 디버그 형식으로 출력할 때도 `Weak`는 `(Weak)`로만 표시되고 따라가지 않으므로 끝없이 출력되지 않습니다.

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다. 아래 출력에서 `Compiling`, `Finished`, `Running` 같은 cargo의 진행 줄은 뺐습니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_box_basics.rs`](examples/01_box_basics.rs) | `Box::new`, `Box`의 크기, 이동할 때 포인터만 복사됨, `String`·`Vec`도 스마트 포인터 | 1 |
| [`02_cons_list.rs`](examples/02_cons_list.rs) | `Box`로 만든 cons 리스트, 열거형의 크기 계산, 재귀 함수 | 1 |
| [`03_deref_mybox.rs`](examples/03_deref_mybox.rs) | `MyBox<T>`에 `Deref`/`DerefMut` 구현, `*(y.deref())`, 역참조 강제 변환 | 2 |
| [`04_drop.rs`](examples/04_drop.rs) | `Drop` 구현, drop 순서(변수·구조체 필드·`Box`), `std::mem::drop` | 3 |
| [`05_rc_shared_list.rs`](examples/05_rc_shared_list.rs) | `Rc`로 꼬리를 공유하는 리스트, `strong_count` 변화, `Rc::clone`은 깊은 복사가 아님 | 4 |
| [`06_refcell.rs`](examples/06_refcell.rs) | `borrow`/`borrow_mut`, `try_borrow_mut`로 위반 관찰, `Rc<RefCell<T>>` (**`-- panic`을 주면 일부러 패닉, 종료 코드 101**) | 4 |
| [`07_mock_messenger.rs`](examples/07_mock_messenger.rs) | `&self` 트레이트 메서드 안에서 기록하는 가짜 객체, `cargo test`로 실행하는 테스트 | 4 |
| [`08_cycle_and_weak.rs`](examples/08_cycle_and_weak.rs) | 순환 참조로 인한 누수 관찰, `Weak`의 `downgrade`/`upgrade`, 부모를 `Weak`로 가리키는 트리 | 5 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 5개 (4절) | 1~4 |

### 01_box_basics — `Box`와 스마트 포인터

`Box`의 크기가 가리키는 데이터와 상관없이 8바이트라는 것, 이동 전후의 힙 주소가 같다는 것을 확인합니다. 힙 주소 값 자체는 실행할 때마다 달라지므로 비교 결과만 출력합니다. 크기는 64비트 환경 기준입니다.

```sh
cargo run -p p0-ch14-smart-pointers --example 01_box_basics
```

```
1. Box::new는 값을 힙에 두고, 그 값을 소유하는 Box를 돌려줍니다
    b = 5
    *b + 1 = 6
2. Box 자체의 크기는 가리키는 데이터와 상관없이 포인터 한 칸입니다
    i32:              4바이트
    Box<i32>:         8바이트
    [u8; 10000]:      10000바이트
    Box<[u8; 10000]>: 8바이트
    &i32:             8바이트
3. Box를 이동하면 포인터만 복사되고, 힙의 데이터는 그 자리에 있습니다
    이동 후에도 같은 힙 주소를 가리키는가? true
    first = 7, len = 10000
4. String과 Vec<T>도 힙 데이터를 소유하는 스마트 포인터입니다
    String: 24바이트, Vec<i32>: 24바이트
    s의 글자 6바이트, v의 원소 3개는 힙에 있습니다
5. main이 끝나면 b, s, v가 버려지면서 각자 소유한 힙 메모리를 해제합니다
```

### 02_cons_list — 재귀 타입

`Box`로 cons 리스트를 만들고 재귀 함수로 합계를 구합니다. 3·4번 구역이 2.3절의 크기 계산을 숫자로 보여 줍니다. 원소가 3개인 리스트와 빈 리스트의 크기가 똑같이 16바이트라는 점에 주목하세요. 5번 구역의 주석 처리된 `println!`을 풀면 E0382가 납니다.

```sh
cargo run -p p0-ch14-smart-pointers --example 02_cons_list
```

```
1. Box로 다음 칸을 감싸 리스트를 만듭니다
    Cons(1, Cons(2, Cons(3, Nil)))
    1 -> 2 -> 3 -> Nil
2. 재귀 타입은 재귀 함수로 다룹니다
    합계 = 6
3. 열거형의 크기 = 가장 큰 배리언트 + 태그
    Shape: 12바이트
4. List의 크기 계산은 Box 덕분에 유한한 값으로 끝납니다
    Box<List>: 8바이트
    List: 16바이트
    원소 3개짜리: 16바이트, 빈 리스트: 16바이트
5. Box는 안의 값을 소유하므로, 리스트를 Box에 넣으면 소유권이 이동합니다
    0 -> 1 -> 2 -> 3 -> Nil
```

### 03_deref_mybox — `Deref`와 역참조 강제 변환

3번 구역은 `*m`과 `*(m.deref())`가 같은 값이라는 것을, 4번 구역은 `hello(&name)`과 강제 변환 없이 쓴 `hello(&(*name)[..])`가 같은 결과라는 것을 보여 줍니다. 5번 구역의 `greet_then_shout`는 `&mut MyBox<String>` 하나를 `&str`을 받는 함수(규칙 3)와 `&mut String`을 받는 함수(규칙 2)에 차례로 넘깁니다. `impl Deref` 블록을 지워 보면 E0614가 납니다(4.2절).

```sh
cargo run -p p0-ch14-smart-pointers --example 03_deref_mybox
```

```
1. 참조는 `*`로 따라가 가리키는 값을 꺼냅니다
    x == *y ? true
2. Box<T>도 참조처럼 `*`로 따라갈 수 있습니다
    x == *b ? true
3. Deref를 구현한 MyBox도 `*`로 따라갈 수 있습니다
    *m = 5
    *(m.deref()) = 5
4. 역참조 강제 변환: &MyBox<String>을 &str 자리에 넘길 수 있습니다
    Hello, Rust!
    Hello, Rust!
    name.len() = 4
    as_str = 문자열, as_slice = [10, 20, 30]
5. DerefMut: `*`로 안의 값을 바꾸고, &mut 강제 변환도 일어납니다
    *counter = 10
    Hello, hey!
    *word = hey!
```

### 04_drop — `Drop`과 drop 순서

`[drop]` 줄이 찍히는 위치를 코드의 `}`, `drop(c)`, `my_drop(e)`와 나란히 놓고 보세요. 2번 구역은 구조체 자신의 `drop`이 필드보다 먼저 실행된다는 것을, 5번 구역은 직접 만든 `my_drop`이 `std::mem::drop`과 똑같이 동작한다는 것을 보여 줍니다. 4번 구역의 `// c.drop();` 주석을 풀면 E0040이 납니다(4.3절).

```sh
cargo run -p p0-ch14-smart-pointers --example 04_drop
```

```
1. 스코프가 끝나면 Rust가 drop을 자동으로 부릅니다 (만든 역순)
    CustomSmartPointer 두 개를 만들었습니다
    [drop] other stuff
    [drop] my stuff
2. 구조체는 자기 drop을 먼저 실행하고, 필드를 선언 순서대로 버립니다
    [drop] Pair(first, second) 시작
    [drop] first
    [drop] second
3. Box가 버려지면 안의 값도 drop되고, 그 뒤에 힙 메모리가 해제됩니다
    Box를 만들었습니다
    [drop] 상자 속 값
4. std::mem::drop으로 스코프가 끝나기 전에 버립니다
    CustomSmartPointer를 만들었습니다
    [drop] some data
    main이 끝나기 전에 버렸습니다
5. drop 함수가 하는 일은 소유권을 가져가는 것뿐입니다
    [drop] my_drop으로 버린 값
    my_drop 호출이 끝났습니다
6. main이 끝납니다
    [drop] 마지막 값
```

### 05_rc_shared_list — `Rc`와 참조 카운트

1번 구역의 카운트 변화(1 → 2 → 3 → 2)가 2.7절의 그림에 대응합니다. 2번 구역은 `Rc::clone`한 두 `Rc`가 같은 값을 가리키고(`true`), 안의 `String`을 복제한 것은 다른 힙 데이터를 가리킨다(`false`)는 것을 비교합니다. 3번 구역에서는 첫 번째 `drop`으로는 아무것도 버려지지 않고, 마지막 소유자를 버릴 때 `[drop] 공유 자원`이 찍힙니다.

```sh
cargo run -p p0-ch14-smart-pointers --example 05_rc_shared_list
```

```
1. 리스트 a를 b와 c가 함께 소유합니다
    a를 만든 뒤 카운트 = 1
    b를 만든 뒤 카운트 = 2
    c를 만든 뒤 카운트 = 3
    b = 3 -> 5 -> 10 -> Nil
    c = 4 -> 5 -> 10 -> Nil
    c가 사라진 뒤 카운트 = 2
2. Rc::clone은 깊은 복사가 아니라 카운트 증가입니다
    같은 값을 가리키는가? true
    String 복제본은 같은 글자를 가리키는가? false
    text의 카운트 = 2
3. 마지막 소유자가 사라질 때 비로소 값이 drop됩니다
    카운트 = 2
    first를 버린 뒤 카운트 = 1
    [drop] 공유 자원
    second까지 버렸습니다
4. Rc는 안의 값을 읽기만 할 수 있습니다
    text = 공유되는 문자열, 길이 = 22
```

### 06_refcell — 실행 시점 대여 검사

인자 없이 실행하면 `try_borrow_mut`/`try_borrow`로 규칙 위반을 패닉 없이 관찰하고 정상 종료합니다. 4번 구역은 `Rc<RefCell<i32>>` 하나를 바꾸면 그 값을 공유하는 `a`, `b`, `c`가 모두 15를 보게 되는 것을 보여 줍니다. 디버그 형식에서 `Rc`는 드러나지 않고 `RefCell`은 `RefCell { value: .. }`로 보입니다.

```sh
cargo run -p p0-ch14-smart-pointers --example 06_refcell
```

```
1. RefCell 안의 값은 바인딩이 불변이어도 바꿀 수 있습니다
    notes = ["첫 메모", "둘째 메모"]
2. 대여 규칙은 같습니다: 불변 대여는 여러 개, 가변 대여는 하나만
    r1.len() = 2, r2[0] = 첫 메모
    w.len() = 3
3. 규칙 위반은 실행 중에 드러납니다 (try_ 메서드로 패닉 없이 확인)
    읽는 중에 가변 대여 → 실패: RefCell already borrowed
    reader를 버린 뒤 가변 대여 → 성공, 길이 4
    쓰는 중에 불변 대여 → 실패: RefCell already mutably borrowed
4. Rc<RefCell<T>>: 여러 소유자가 같은 값을 바꿀 수 있습니다
    a = 15 -> Nil
    b = 3 -> 15 -> Nil
    c = 4 -> 15 -> Nil
    b = Cons(RefCell { value: 3 }, Cons(RefCell { value: 15 }, Nil))
5. (건너뜀) `-- panic`을 붙여 실행하면 가변 대여를 두 번 해서 패닉합니다
```

`-- panic`을 붙이면 5번 구역에서 **일부러 패닉하며 끝납니다.** 1~4번 구역의 출력은 위와 같으므로 생략했습니다.

```sh
cargo run -p p0-ch14-smart-pointers --example 06_refcell -- panic
echo $?   # 종료 코드 확인
```

```
...
5. 가변 대여를 두 번 하면 컴파일은 되지만 실행 중에 패닉합니다

thread 'main' (6405104) panicked at part0-getting-started/ch14-smart-pointers/examples/06_refcell.rs:87:32:
RefCell already borrowed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
101
```

스레드 ID는 실행할 때마다 달라집니다. 87번 줄은 두 번째 `borrow_mut()`를 부른 곳입니다.

### 07_mock_messenger — 내부 가변성으로 만든 가짜 객체

`main`은 화면에 출력하는 실제 메신저와 기록만 하는 가짜 메신저를 차례로 써 봅니다. The Book은 가짜 메신저를 `#[cfg(test)] mod tests` 안에 두지만, 이 예제는 `main`에서도 관찰하려고 밖에 두었습니다.

```sh
cargo run -p p0-ch14-smart-pointers --example 07_mock_messenger
```

```
1. 실제 메신저: 사용량에 따라 알림을 보냅니다
    사용량 50/100
    사용량 80/100
    [알림] 경고: 할당량의 75% 이상을 사용했습니다!
    사용량 95/100
    [알림] 긴급 경고: 할당량의 90% 이상을 사용했습니다!
    사용량 120/100
    [알림] 에러: 할당량을 넘었습니다!
2. 가짜 메신저: &self인 send 안에서도 RefCell 덕분에 기록이 남습니다
    기록된 메시지: 2개
    - 경고: 할당량의 75% 이상을 사용했습니다!
    - 긴급 경고: 할당량의 90% 이상을 사용했습니다!
```

같은 파일의 테스트 3개는 `cargo test`에 `--example`을 붙여 실행합니다(`ch11-testing`에서 본 방법입니다). 테스트는 병렬로 실행되므로 결과 줄의 순서는 실행마다 다를 수 있습니다.

```sh
cargo test -p p0-ch14-smart-pointers --example 07_mock_messenger
```

```
running 3 tests
test tests::it_sends_an_error_when_over_quota ... ok
test tests::it_sends_nothing_under_75_percent ... ok
test tests::it_sends_an_over_75_percent_warning_message ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 08_cycle_and_weak — 순환 참조와 `Weak`

1번 구역(순환 없음)에서는 블록이 끝날 때 `[drop] Cons(10)`, `[drop] Cons(5)`가 찍히지만, 2번 구역(순환)에서는 하나도 찍히지 않습니다. 3번 구역은 `Weak`가 값을 살려 두지 않는다는 것을, 4~5번 구역은 2.11절의 카운트 표를 출력으로 보여 줍니다. 2번 구역 안의 주석 처리된 `println!`은 스택 오버플로를 일으키므로 풀지 마세요(2.10절).

```sh
cargo run -p p0-ch14-smart-pointers --example 08_cycle_and_weak
```

```
1. 비교용: 순환이 없으면 스코프가 끝날 때 정상적으로 drop됩니다
    a의 카운트 = 2
    b의 카운트 = 1
    [drop] Cons(10)
    [drop] Cons(5)
2. Rc로 순환을 만들면 스코프가 끝나도 drop되지 않습니다 (메모리 누수)
    b의 다음 칸 = Some(RefCell { value: Cons(5, RefCell { value: Nil }) })
    순환을 만든 뒤 a의 카운트 = 2
    순환을 만든 뒤 b의 카운트 = 2
    블록이 끝났지만 [drop] Cons(..) 줄이 찍히지 않았습니다
3. Weak는 값을 가리키기만 하고 살려 두지는 않습니다
    owner: strong = 1, weak = 1
    upgrade → Some(Node(1))
    owner: strong = 2, weak = 1
    [drop] Node(1)
    owner를 버린 뒤 upgrade → None
4. 트리: 자식은 부모를 Weak로 가리킵니다
    leaf의 부모 = None
    leaf: strong = 1, weak = 0
    leaf의 부모 = Some(5)
    branch의 자식 수 = 1
    branch: strong = 1, weak = 1
    leaf: strong = 2, weak = 0
    leaf = Node { value: 3, parent: RefCell { value: (Weak) }, children: RefCell { value: [] } }
5. 부모가 사라지면 자식의 Weak는 None이 됩니다
    [drop] Node(5)
    leaf의 부모 = None
    leaf: strong = 1, weak = 0
6. main이 끝나면 남은 leaf도 drop됩니다
    [drop] Node(3)
```

1번 구역에서 `Cons(10)`이 먼저 drop되는 것은 변수 `b`가 나중에 선언되어 먼저 버려지기 때문입니다. `Cons(10)`이 drop되면서 그 안의 꼬리(`a`를 가리키는 `Rc`)가 버려져 `a`의 카운트가 1이 되고, 이어서 변수 `a`가 버려지면 0이 되어 `Cons(5)`가 drop됩니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch14-smart-pointers
rustc --edition 2024 compile_fail/e0072_recursive_type.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요. 에러 코드의 자세한 설명은 `rustc --explain E0072`처럼 볼 수 있습니다.

### 4.1 `Box` 없는 재귀 타입 — E0072

파일: [`compile_fail/e0072_recursive_type.rs`](compile_fail/e0072_recursive_type.rs)

```
error[E0072]: recursive type `List` has infinite size
 --> compile_fail/e0072_recursive_type.rs:7:1
  |
7 | enum List {
  | ^^^^^^^^^
8 |     Cons(i32, List),
  |               ---- recursive without indirection
  |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
  |
8 |     Cons(i32, Box<List>),
  |               ++++    +
```

- **원인**: `List`가 `List`를 직접 품고 있어(recursive without indirection), 크기를 계산하려면 자기 자신의 크기가 필요합니다(2.3절).
- **해결**: `help:`가 말하는 간접 참조(indirection)를 넣습니다. 제안대로 `Box<List>`로 바꾸고, 값을 만들 때도 `Cons(1, Box::new(...))`처럼 감쌉니다. `Rc`나 `&`도 후보로 나오지만, 리스트가 다음 칸을 혼자 소유한다면 `Box`가 맞습니다.

### 4.2 `Deref` 없는 타입에 `*` 사용 — E0614

파일: [`compile_fail/e0614_deref_without_trait.rs`](compile_fail/e0614_deref_without_trait.rs)

```
error[E0614]: type `MyBox<{integer}>` cannot be dereferenced
  --> compile_fail/e0614_deref_without_trait.rs:20:19
   |
20 |     assert_eq!(5, *y);
   |                   ^^ can't be dereferenced
```

- **원인**: 컴파일러가 `*`로 따라갈 수 있는 것은 참조와 `Deref`를 구현한 타입뿐입니다. `MyBox`는 이름만 Box일 뿐 아무 트레이트도 구현하지 않았습니다. `{integer}`는 "아직 구체적인 정수 타입이 정해지지 않은 정수"라는 뜻입니다.
- **해결**: `impl<T> Deref for MyBox<T>`를 추가합니다(2.4절). 안의 값을 바꾸는 `*y = 6` 같은 코드까지 쓰려면 `DerefMut`도 구현합니다.

### 4.3 `drop` 메서드를 직접 호출 — E0040

파일: [`compile_fail/e0040_explicit_drop_call.rs`](compile_fail/e0040_explicit_drop_call.rs)

```
error[E0040]: explicit use of destructor method
  --> compile_fail/e0040_explicit_drop_call.rs:23:7
   |
23 |     c.drop();
   |       ^^^^ explicit destructor calls not allowed
   |
help: consider using `drop` function
   |
23 -     c.drop();
23 +     drop(c);
   |
```

- **원인**: `Drop::drop`은 Rust만 부를 수 있습니다. 직접 부르면 스코프 끝에서 한 번 더 불려 이중 해제가 되기 때문입니다(2.6절).
- **해결**: `help:`대로 `drop(c)`, 즉 `std::mem::drop` 함수를 씁니다. 메서드 호출 `c.drop()`과 함수 호출 `drop(c)`는 모양이 비슷하지만 전혀 다릅니다. 함수는 `c`의 소유권을 가져가므로 이후에 `c`를 쓰거나 다시 drop할 수 없습니다.

### 4.4 `Box` 리스트를 두 곳에서 소유 — E0382

파일: [`compile_fail/e0382_box_list_shared.rs`](compile_fail/e0382_box_list_shared.rs)

```
error[E0382]: use of moved value: `a`
  --> compile_fail/e0382_box_list_shared.rs:18:31
   |
16 |     let a = Cons(5, Box::new(Cons(10, Box::new(Nil))));
   |         - move occurs because `a` has type `List`, which does not implement the `Copy` trait
17 |     let _b = Cons(3, Box::new(a));
   |                               - value moved here
18 |     let _c = Cons(4, Box::new(a));
   |                               ^ value used here after move
   |
note: if `List` implemented `Clone`, you could clone the value
```

- **원인**: `Box::new(a)`로 `a`가 `_b`의 `Box` 안으로 이동했습니다. `Box`는 소유자가 하나뿐인 포인터이므로 같은 값을 두 번째 `Box`에 넣을 수 없습니다.
- **해결**: `note:`는 `List`에 `Clone`을 구현해 복제하라고 하지만, 그러면 꼬리 리스트가 두 벌 생겨 "공유"가 아니게 됩니다. 꼬리를 정말 함께 쓰려면 `Box`를 `Rc`로 바꾸고 `Rc::clone(&a)`를 넘깁니다(2.7절, [`05_rc_shared_list.rs`](examples/05_rc_shared_list.rs)).

### 4.5 `Rc` 안의 값을 수정 — E0596

파일: [`compile_fail/e0596_mutate_inside_rc.rs`](compile_fail/e0596_mutate_inside_rc.rs)

```
error[E0596]: cannot borrow data in an `Rc` as mutable
  --> compile_fail/e0596_mutate_inside_rc.rs:14:5
   |
14 |     shared.push_str(", world");
   |     ^^^^^^ cannot borrow as mutable
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<String>`
```

- **원인**: `push_str`은 `&mut String`이 필요한데, `Rc`는 `Deref`만 구현하고 `DerefMut`는 구현하지 않습니다(`help:`가 정확히 이 점을 짚습니다). `shared`를 `let mut`로 바꿔도 같은 에러가 납니다. 문제는 변수의 가변성이 아니라, 다른 소유자(`other`)가 있는 값을 한쪽에서 바꾸려 한 것이기 때문입니다.
- **해결**: 값을 `Rc<RefCell<String>>`으로 감싸고 `shared.borrow_mut().push_str(", world")`로 바꿉니다(2.9절). 그러면 `other`로 읽어도 바뀐 값이 보입니다.

### 4.6 `RefCell`을 동시에 두 번 가변 대여 — 실행 중 패닉

이것은 컴파일 에러가 아닙니다. `RefCell`은 대여 규칙을 실행 시점에 검사하므로, 규칙을 어긴 코드도 컴파일은 되고 **실행하다 패닉합니다**(`RefCell already borrowed`, 종료 코드 101). 출력은 3절의 `06_refcell -- panic`에 있습니다.

- **원인**: 앞서 받은 `RefMut`(또는 `Ref`)가 아직 살아 있는데 `borrow_mut()`를 또 불렀습니다. 대여는 `Ref`/`RefMut`가 버려질 때 끝나므로, 변수에 담아 둔 대여는 그 변수의 스코프 끝까지 이어집니다.
- **해결**: 대여를 짧게 씁니다. `notes.borrow_mut().push(...)`처럼 한 문장 안에서 쓰고 버리거나, 블록으로 스코프를 나누거나, 다 쓴 대여를 `drop(guard)`로 일찍 버립니다. 실패할 수 있는 상황이라면 `try_borrow_mut()`로 `Result`를 받아 처리합니다.

## 5. 연습 문제

### 연습 1. 슬라이스로 cons 리스트 만들기

[`02_cons_list.rs`](examples/02_cons_list.rs)의 `List`에 대해 `fn from_slice(values: &[i32]) -> List`를 작성하세요. `from_slice(&[1, 2, 3])`은 `Cons(1, Cons(2, Cons(3, Nil)))`을, `from_slice(&[])`는 `Nil`을 돌려줘야 합니다. 디버그 형식으로 출력해 확인하세요.

<details>
<summary>힌트</summary>

리스트는 뒤에서부터 만들면 쉽습니다. `let mut list = Nil;`로 시작해 `for value in values.iter().rev()`로 뒤에서부터 돌면서 `list = Cons(*value, Box::new(list));`로 앞에 한 칸씩 붙입니다. 대입할 때 이전 `list`가 새 `Box` 안으로 이동하므로 복사는 일어나지 않습니다. 재귀로 풀고 싶다면 `values`가 비었으면 `Nil`, 아니면 `Cons(values[0], Box::new(from_slice(&values[1..])))`를 돌려주면 됩니다.

</details>

### 연습 2. drop 순서 예측하기

[`04_drop.rs`](examples/04_drop.rs)의 `main`을 아래 코드로 바꾸고, 실행하기 **전에** 출력 순서를 예측해 보세요.

```rust
fn main() {
    let a = custom("a");
    let _pair = Pair {
        first: custom("p1"),
        second: custom("p2"),
    };
    let boxed = Box::new(custom("box"));
    drop(a);
    {
        let _inner = custom("inner");
        let _moved = boxed;
    }
    println!("    main 끝");
}
```

<details>
<summary>힌트</summary>

`a`는 `drop(a)`에서 바로 버려집니다. 블록 안에서는 나중에 선언된 `_moved`가 `_inner`보다 먼저 버려지는데, `_moved`는 `boxed`에서 이동해 온 `Box`이므로 이때 `box`가 drop됩니다. `a`와 `boxed`는 이미 이동했으므로 `main` 끝에서 다시 drop되지 않고, 남은 `_pair`만 `Pair`의 `drop` → `p1` → `p2` 순서로 버려집니다. 출력은 `[drop] a`, `[drop] box`, `[drop] inner`, `main 끝`, `[drop] Pair(p1, p2) 시작`, `[drop] p1`, `[drop] p2` 순서입니다.

</details>

### 연습 3. 공유된 값을 모두 두 배로

[`06_refcell.rs`](examples/06_refcell.rs)의 4번 구역에 이어서, 리스트의 모든 칸의 값을 두 배로 만드는 `fn double_all(list: &List)`를 작성하세요. 그리고 `double_all(&b)`를 부른 뒤 `b`와 `c`를 출력해 보세요. `c`에는 `double_all`을 부르지 않았는데 `c`의 출력도 바뀌는 칸이 있습니다. 어느 칸이고, 왜 그런가요?

<details>
<summary>힌트</summary>

`if let Cons(value, rest) = list`로 칸을 꺼내 `*value.borrow_mut() *= 2;`로 값을 바꾸고, `double_all(rest)`로 다음 칸에 재귀합니다. `rest`는 `&Rc<List>`이지만 역참조 강제 변환으로 `&List`가 됩니다. `double_all(&b)` 뒤에 `b`는 `6 -> 30 -> Nil`, `c`는 `4 -> 30 -> Nil`이 됩니다. `b`와 `c`의 꼬리는 같은 `a`이고, `a`의 값은 같은 `RefCell` 하나이기 때문입니다. `Rc::clone`이 깊은 복사가 아니라는 것을 다시 확인할 수 있습니다.

</details>

### 연습 4. 트리의 깊이 구하기

[`08_cycle_and_weak.rs`](examples/08_cycle_and_weak.rs)의 `Node`에 대해 다음 두 함수를 작성하세요.

- `fn add_child(parent: &Rc<Node>, child: Rc<Node>)`: `child`의 부모를 `parent`로 정하고, `parent`의 자식 목록에 `child`를 넣습니다.
- `fn depth(node: &Rc<Node>) -> usize`: 부모를 따라 올라가며 뿌리까지 몇 단계인지 셉니다. 뿌리는 0입니다.

`root`(1) → `mid`(2) → `leaf`(3)를 만들고 각각의 깊이가 0, 1, 2인지 확인하세요. 그다음 `drop(root)`를 하고 `depth(&leaf)`를 다시 구하면 얼마가 나올지 예측해 보세요.

<details>
<summary>힌트</summary>

`add_child`는 `*child.parent.borrow_mut() = Rc::downgrade(parent);` 다음에 `parent.children.borrow_mut().push(child);`입니다. `depth`는 `let mut current = Rc::clone(node);`에서 시작해, `let parent = current.parent.borrow().upgrade();`로 부모를 얻고 `Some(p)`이면 깊이를 1 늘리고 `current = p`, `None`이면 멈추는 `loop`로 씁니다. `upgrade()` 결과를 `let`으로 먼저 받아 두는 이유는, `current.parent.borrow()`가 만든 `Ref`를 그 문장에서 끝내야 `current`에 새 값을 대입할 수 있기 때문입니다. `match current.parent.borrow().upgrade() { ... }`처럼 바로 `match`에 넣으면 `Ref`가 `match`가 끝날 때까지 살아 있어서 E0506(``cannot assign to `current` because it is borrowed``)이 납니다. `drop(root)` 뒤에는 `root`를 가리키던 것이 `mid`의 `Weak`뿐이므로 `root`가 drop되고, `mid`의 부모가 `None`이 되어 `depth(&leaf)`는 1이 됩니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [15. Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
  - [15.1 Using `Box<T>` to Point to Data on the Heap](https://doc.rust-lang.org/book/ch15-01-box.html)
  - [15.2 Treating Smart Pointers Like Regular References with `Deref`](https://doc.rust-lang.org/book/ch15-02-deref.html)
  - [15.3 Running Code on Cleanup with the `Drop` Trait](https://doc.rust-lang.org/book/ch15-03-drop.html)
  - [15.4 `Rc<T>`, the Reference Counted Smart Pointer](https://doc.rust-lang.org/book/ch15-04-rc.html)
  - [15.5 `RefCell<T>` and the Interior Mutability Pattern](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
  - [15.6 Reference Cycles Can Leak Memory](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)
- Rust By Example
  - [Std library types > Box, stack and heap](https://doc.rust-lang.org/rust-by-example/std/box.html): `Box`로 구조체를 힙에 두고 `size_of_val`로 크기를 비교합니다.
  - [Std library types > Rc](https://doc.rust-lang.org/rust-by-example/std/rc.html): `Rc::clone`과 `strong_count`의 변화를 따라갑니다.
  - [Traits > Drop](https://doc.rust-lang.org/rust-by-example/trait/drop.html): 중첩 스코프의 drop 순서, `drop`으로 일찍 버리기, 임시 파일을 지우는 실용적인 `Drop` 구현이 나옵니다.
- Rustlings: [`19_smart_pointers`](https://github.com/rust-lang/rustlings/tree/main/exercises/19_smart_pointers) (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - `smart_pointers1.rs`: `Box`로 cons 리스트 만들기
  - `smart_pointers2.rs`: `Rc`로 태양을 여러 행성이 공유하기
  - `smart_pointers3.rs`는 스레드와 `Arc`(`ch15-concurrency`), `smart_pointers4.rs`는 `Cow`(Part 1 `ch6-core-traits`)를 다루므로 해당 챕터를 마친 뒤에 풉니다.
- 표준 라이브러리 문서
  - [`std::rc`](https://doc.rust-lang.org/std/rc/index.html): `Rc`와 `Weak`, 순환을 `Weak`로 끊는 예
  - [`std::cell`](https://doc.rust-lang.org/std/cell/index.html): `Cell`, `RefCell`, `OnceCell`과 내부 가변성을 언제 쓰는지
  - [`std::mem::drop`](https://doc.rust-lang.org/std/mem/fn.drop.html): 정의와 `RefCell` 대여를 일찍 끝내는 예
