# ch17. 트레이트 객체와 객체 지향

Java, C#, Python 같은 언어에서 프로그램을 클래스와 상속으로 설계해 왔다면, Rust에 클래스도 상속도 없다는 사실이 처음에는 막막하게 느껴집니다. 이 챕터에서는 객체 지향 프로그래밍(object-oriented programming, OOP)이 흔히 내세우는 특징이 Rust에서는 각각 무엇으로 바뀌는지 확인합니다. 그 중심에 있는 것이 **트레이트 객체(trait object)** `dyn Trait`입니다. [ch9](../ch9-error-handling/README.md)부터 써 온 `Box<dyn Error>`와 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 막혔던 "갈래마다 다른 타입 반환"도 이 챕터에서 풀립니다. 마지막으로 객체 지향의 대표적인 설계 패턴인 상태 패턴을 트레이트 객체로 한 번, Rust다운 방식인 타입으로 한 번 구현해 비교합니다.

이 저장소의 `ch17`은 The Book의 **18장**에 해당합니다. Book 14장을 Part 3으로 옮겼기 때문에 저장소 챕터 번호가 Book 장 번호보다 하나 작습니다.

- 패키지: `p0-ch17-trait-objects-oop`
- 기준 문서: [The Book 18장 Object-Oriented Programming Features](https://doc.rust-lang.org/book/ch18-00-oop.html)
- 확인한 툴체인: rustc 1.98.1, Edition 2024

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 객체 지향의 세 특징(데이터와 동작을 묶은 객체, 캡슐화, 상속)이 Rust에서 각각 무엇에 해당하는지 설명할 수 있다. 비공개 필드와 공개 메서드로 불변식을 지키는 타입(`AveragedCollection`)을 작성하고, 상속 없이 기본 구현으로 코드를 재사용할 수 있다.
2. `Vec<Box<dyn Trait>>`로 서로 다른 타입의 값을 한 컬렉션에 담는 코드를 작성하고, 제네릭과 트레이트 바운드로 만든 버전(`GenericScreen<T: Draw>`)과 무엇이 다른지 설명할 수 있다.
3. 정적 디스패치와 동적 디스패치의 차이와 비용을 설명하고, `&dyn Trait`과 `Box<dyn Trait>` 중 맞는 것을 고를 수 있다. 반환 위치의 `Box<dyn Trait>`과 `Box<dyn Error>`가 무엇인지 설명할 수 있다.
4. 열거형, 제네릭, 트레이트 객체 중 무엇을 쓸지 근거를 들어 고를 수 있고, 트레이트 객체로 만들 수 없는 트레이트를 E0038 메시지로 알아보고 고칠 수 있다.
5. 블로그 글의 상태 흐름을 트레이트 객체로 구현한 상태 패턴(`Option<Box<dyn State>>`, `take()`, `self: Box<Self>`)과 상태를 타입으로 표현한 구현으로 각각 작성하고, 두 방식의 장단점을 비교할 수 있다.

## 2. 핵심 개념

2.1~2.3절은 객체 지향의 특징과 캡슐화, 2.4~2.8절은 트레이트 객체, 2.9~2.10절은 트레이트 객체의 한계와 대안, 2.11~2.13절은 상태 패턴을 다룹니다.

### 2.1 Rust는 객체 지향 언어인가

객체 지향 언어가 무엇인지에 대해서는 합의된 정의가 없습니다. 흔히 꼽는 세 가지 특징을 기준으로 보면 다음과 같습니다.

| 특징 | 뜻 | Rust에서는 |
| --- | --- | --- |
| 객체 | 데이터와 그 데이터를 다루는 동작(메서드)을 함께 묶은 것 | 구조체·열거형(데이터)과 `impl` 블록(메서드). 이름은 객체가 아니지만 하는 일은 같습니다. |
| 캡슐화(encapsulation) | 구현 세부를 숨기고 공개된 창구로만 다루게 하는 것 | `pub`을 붙인 것만 모듈 밖에 공개됩니다(ch7). 필드를 비공개로 두고 메서드만 공개합니다(2.2절). |
| 상속(inheritance) | 부모 타입의 데이터와 동작을 자식 타입이 다시 정의하지 않고 물려받는 것 | **없습니다.** 상속을 쓰려던 이유에 따라 다른 도구를 씁니다(2.3절). |

"Rust는 객체 지향 언어인가"에 대한 답은 어느 정의를 따르느냐에 따라 달라집니다. 중요한 것은 이름이 아니라, 다른 언어에서 객체 지향으로 풀던 문제를 Rust에서는 어떤 도구로 푸는지 아는 것입니다.

### 2.2 캡슐화: 불변식을 지키는 `AveragedCollection`

정수 목록과 그 평균을 함께 들고 있는 타입을 생각해 봅니다. 평균을 필요할 때마다 계산하지 않고 필드에 저장해 두면(캐시) 읽기가 빠릅니다. 대신 **"`average`는 언제나 `list`의 평균"** 이라는 불변식(invariant)을 지켜야 합니다. `list`가 바뀌었는데 `average`를 갱신하지 않으면 틀린 값이 남기 때문입니다.

```rust
mod cached {
    #[derive(Default)]
    pub struct AveragedCollection {
        list: Vec<i32>,   // pub이 없으므로 모듈 밖에서는 읽지도 쓰지도 못합니다.
        average: f64,
    }

    impl AveragedCollection {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add(&mut self, value: i32) {
            self.list.push(value);
            self.update_average(); // list가 바뀌는 곳마다 평균도 갱신합니다.
        }

        pub fn remove(&mut self) -> Option<i32> {
            let value = self.list.pop()?;
            self.update_average();
            Some(value)
        }

        pub fn average(&self) -> f64 {  // 읽기만 허용하는 게터
            self.average
        }

        fn update_average(&mut self) { /* 비공개 도우미 */ }
    }
}
```

- 구조체는 `pub`이지만 필드는 비공개입니다. 바깥 코드가 `list`를 바꾸는 방법은 `add`와 `remove`뿐이고, 둘 다 `update_average`를 부르므로 불변식이 깨질 틈이 없습니다. 불변식을 지킬 책임이 이 `impl` 블록 안으로 모입니다.
- 필드가 비공개라 모듈 밖에서는 `AveragedCollection { list: ..., average: ... }`으로 값을 만들 수도 없습니다. 그래서 `new` 같은 공개 생성 함수가 반드시 필요합니다(The Book의 코드에는 생략되어 있습니다). 여기서는 `#[derive(Default)]`가 만든 `default()`(빈 `Vec`과 `0.0`)를 그대로 씁니다.
- 바깥에서 `scores.list.push(100)`처럼 필드에 손대면 E0616으로 거부됩니다(4.4절). ch9의 검증 타입 `Guess`와 같은 원리입니다.

**왜 이렇게까지 하는가?** [`01_encapsulation.rs`](examples/01_encapsulation.rs)의 3번 구역은 필드를 모두 공개한 `LeakyCollection`으로 무엇이 잘못될 수 있는지 보여 줍니다. `leaky.list.push(0)`으로 값을 직접 넣으면 실제 평균은 51.67인데 `average`에는 77.50이 남고, `leaky.average = 100.0`처럼 평균을 마음대로 덮어써도 막을 방법이 없습니다.

캡슐화의 두 번째 이득은 **내부 표현을 자유롭게 바꿀 수 있다**는 것입니다. 같은 예제의 `on_demand` 모듈은 평균을 저장하지 않고 합계(`total`)만 들고 있다가 `average()`를 부를 때 계산합니다. 공개 메서드의 시그니처가 같으므로 4번 구역의 두 블록은 `use` 줄만 다르고 나머지 코드가 글자 하나까지 같으며, 결과도 같습니다. 만약 `list`가 공개되어 바깥 코드가 `list.push`를 쓰고 있었다면, 표현을 바꾸는 순간 `total`이 어긋나므로 그 코드를 모두 찾아 고쳐야 했을 것입니다.

> 빈 목록의 평균: `0 / 0`을 `f64`로 계산하면 에러가 아니라 `NaN`(숫자가 아님)이 됩니다. 예제에서는 목록이 비면 평균을 `0.0`으로 정했습니다. 이런 "빈 경우의 약속"도 공개 API의 일부이므로, 내부 표현을 바꿀 때 함께 지켜야 합니다.

### 2.3 상속 대신: 코드 재사용과 다형성

상속을 쓰는 이유는 크게 두 가지입니다. Rust는 각각을 다른 도구로 해결합니다.

**1. 코드 재사용 → 트레이트의 기본 구현.** 부모 클래스에 구현한 메서드를 자식 클래스가 물려받듯이, 트레이트에 기본 구현을 두면 구현하는 타입이 아무것도 쓰지 않아도 그 메서드를 얻습니다. 필요하면 같은 시그니처로 다시 작성해 재정의합니다(ch10 2.5절).

```rust
pub trait Draw {
    fn draw(&self) -> String;     // 각 타입이 반드시 구현

    fn focusable(&self) -> bool { // 기본 구현: 대부분의 컴포넌트는 포커스를 받지 않습니다.
        false
    }
}
```

[`02_gui_trait_objects.rs`](examples/02_gui_trait_objects.rs)에서 `Label`은 `focusable`을 쓰지 않아 기본 구현(`false`)을 쓰고, `Button`과 `SelectBox`는 `true`를 돌려주도록 재정의합니다. 다만 상속과 달리 트레이트는 **필드를 물려줄 수 없고**, 재정의한 메서드 안에서 기본 구현을 불러 쓸 수도 없습니다.

**2. 다형성(polymorphism) → 제네릭과 트레이트 객체.** 다형성은 "여러 타입의 값을 같은 코드로 다룰 수 있는 것"입니다. 상속을 쓰는 언어에서는 자식 클래스의 값을 부모 클래스 자리에 넣는 방식으로 다형성을 얻습니다. Rust에서는 두 가지 방법이 있습니다.

- **제네릭 + 트레이트 바운드**(ch10): `fn notify<T: Summary>(item: &T)`. 어떤 타입이 올지는 컴파일할 때 정해집니다.
- **트레이트 객체**(이 챕터): `fn notify(item: &dyn Summary)`. 어떤 타입이 올지는 실행 중에 정해집니다.

Rust가 상속을 넣지 않은 이유도 있습니다. 상속은 자식이 부모의 모든 것을 물려받으므로 필요 이상으로 많은 코드를 공유하기 쉽고, 자식에게는 의미가 없는 부모의 메서드까지 부를 수 있게 됩니다. 트레이트는 필요한 동작만 골라 묶고, 한 타입이 여러 트레이트를 구현할 수 있어서 이런 문제가 적습니다.

### 2.4 트레이트 객체 `dyn Trait`

ch8에서 벡터는 한 가지 타입만 담을 수 있다고 했고, 여러 종류를 담으려면 열거형으로 감쌌습니다. 그런데 **담을 타입의 목록을 미리 정할 수 없는** 경우가 있습니다. GUI 라이브러리를 만든다고 해 봅시다. 라이브러리는 `Button`과 `Label`을 제공하지만, 라이브러리를 쓰는 사람은 `SelectBox`처럼 라이브러리 작성자가 모르는 컴포넌트를 새로 만들어 함께 화면에 올리고 싶어 합니다. 열거형에는 라이브러리 작성자가 아는 배리언트만 넣을 수 있으므로 이 요구를 채울 수 없습니다.

라이브러리가 알아야 하는 것은 각 컴포넌트의 구체 타입이 아니라 "`draw`를 부를 수 있다"는 사실뿐입니다. 이것을 그대로 타입으로 쓴 것이 트레이트 객체입니다.

```rust
pub struct Screen {
    pub components: Vec<Box<dyn Draw>>,
}

impl Screen {
    pub fn run(&self) {
        for component in self.components.iter() {
            println!("{}", component.draw()); // 실제 타입이 무엇이든 Draw의 메서드는 부를 수 있습니다.
        }
    }
}
```

- `dyn Draw`는 "`Draw`를 구현한 **어떤** 타입의 값"을 뜻합니다. 실제 타입은 실행 중에 정해지고, 타입 정보로는 남지 않습니다.
- 트레이트 객체는 언제나 **포인터 뒤에** 둡니다. `Box<dyn Draw>`(소유)나 `&dyn Draw`(대여)처럼 씁니다. 이유는 2.7절에서 봅니다.
- `Box::new(Button { .. })`는 `Box<Button>`이지만, `Box<dyn Draw>`가 필요한 자리에 넣으면 컴파일러가 `Box<dyn Draw>`로 바꿔 줍니다. 이때 `Button`이 `Draw`를 구현했는지 **컴파일할 때** 검사합니다. 구현하지 않은 `String`을 넣으면 E0277입니다(4.1절).
- 2021 에디션부터 `dyn`은 생략할 수 없습니다. `Vec<Box<Draw>>`라고 쓰면 E0782(`expected a type, found a trait`)와 함께 `dyn`을 붙이라는 `help:`가 나옵니다.

`Screen`을 쓰는 쪽은 서로 다른 세 타입을 한 벡터에 담습니다.

```rust
let screen = Screen {
    components: vec![
        Box::new(Label { text: String::from("배송 방법을 고르세요") }),
        Box::new(SelectBox { width: 75, height: 10, options: vec![/* ... */] }),
        Box::new(Button { width: 50, height: 10, label: String::from("OK") }),
    ],
};
screen.run();
```

`SelectBox`는 `gui` 모듈 밖, 즉 라이브러리를 쓰는 쪽에서 정의한 타입입니다. 라이브러리 코드는 한 글자도 고치지 않았는데 `Screen`에 들어갑니다. 이것이 트레이트 객체가 주는 **확장성**입니다.

값의 실제 타입은 몰라도 그 값이 할 줄 아는 일(메서드)만 보고 다루는 방식은 동적 타입 언어의 덕 타이핑(duck typing, "오리처럼 걷고 오리처럼 꽥꽥거리면 오리다")과 비슷합니다. 차이는 Rust가 "정말 그 메서드가 있는가"를 컴파일할 때 확인한다는 점입니다. 실행 중에 "메서드가 없다"는 에러가 날 일이 없습니다.

트레이트 객체로는 **트레이트에 있는 메서드만** 쓸 수 있습니다. 첫 원소가 `Label`이라는 사실은 `Box<dyn Draw>` 타입에 남아 있지 않으므로 `screen.components[0].text`는 E0609(``no field `text` on type `Box<dyn Draw>` ``)입니다. ch10에서 `&impl Summary`로 받은 값의 필드를 쓸 수 없던 것과 같습니다. 또한 다른 언어의 객체와 달리 트레이트 객체에 데이터를 더할 수는 없습니다. 트레이트 객체의 목적은 오직 "공통 동작으로 여러 타입을 묶는 것"입니다.

[`02_gui_trait_objects.rs`](examples/02_gui_trait_objects.rs)의 3번 구역은 한 걸음 더 나아가, `"label:로그인이 필요합니다,button:로그인,button:취소"`라는 문자열을 읽고 **실행 중에** 어떤 타입의 컴포넌트를 만들지 정합니다.

```rust
let component: Box<dyn Draw> = match kind {
    "button" => Box::new(Button { /* ... */ }),
    _ => Box::new(Label { /* ... */ }),
};
```

여기서 타입 표기 `: Box<dyn Draw>`를 지우면 첫 갈래가 타입을 `Box<Button>`으로 정해 버려서 E0308(`` `match` arms have incompatible types ``, ``expected `Box<Button>`, found `Box<Label>` ``)이 납니다. 각 갈래의 `Box`를 `Box<dyn Draw>`로 바꾸려면 컴파일러가 목표 타입을 알아야 합니다.

### 2.5 제네릭 버전과 비교: 한 가지 타입만 담는 `GenericScreen<T>`

같은 일을 ch10의 제네릭과 트레이트 바운드로 써 볼 수도 있습니다.

```rust
pub struct GenericScreen<T: Draw> {
    pub components: Vec<T>,
}

impl<T: Draw> GenericScreen<T> {
    pub fn run(&self) {
        for component in self.components.iter() {
            println!("{}", component.draw());
        }
    }
}
```

겉보기에는 비슷하지만 뜻이 다릅니다. 타입 매개변수 `T`는 `GenericScreen` 값 하나에 대해 **한 가지 구체 타입**으로 정해집니다. 버튼만 담은 `GenericScreen<Button>`은 만들 수 있지만, 여기에 `SelectBox`를 섞으면 E0308(``expected `Button`, found `SelectBox` ``)입니다.

| | `Screen` (`Vec<Box<dyn Draw>>`) | `GenericScreen<T: Draw>` (`Vec<T>`) |
| --- | --- | --- |
| 한 화면에 담을 수 있는 타입 | `Draw`를 구현한 여러 타입을 섞어서 | 한 가지 타입만 |
| 어느 `draw`를 부를지 정하는 때 | 실행 중 (동적 디스패치) | 컴파일할 때 (정적 디스패치) |
| 원소 저장 방식 | 원소마다 힙에 `Box` 하나 | `Vec` 안에 값을 그대로 |

담을 값이 언제나 한 가지 타입이라면 제네릭 쪽이 낫습니다. 단형성화(ch10 2.4절) 덕분에 실행 비용이 없기 때문입니다. 여러 타입을 섞어야 할 때 트레이트 객체를 씁니다.

### 2.6 정적 디스패치와 동적 디스패치

메서드 호출 `shape.area()`를 만났을 때, 실제로 어느 타입의 `area` 함수로 갈지 정하는 일을 **디스패치(dispatch)** 라고 합니다.

- **정적 디스패치(static dispatch)**: 컴파일할 때 정합니다. 제네릭 함수 `describe_static<T: Shape>`는 쓰인 타입마다 한 벌씩 만들어지고(단형성화), `describe_static::<Circle>` 안의 `shape.area()`는 처음부터 `Circle`의 `area`를 부르도록 정해져 있습니다.
- **동적 디스패치(dynamic dispatch)**: 실행 중에 정합니다. `describe_dynamic(shape: &dyn Shape)`는 한 벌뿐이고, `shape.area()`를 만나면 `shape`에 함께 붙어 있는 표를 보고 부를 함수를 찾습니다.

이 표를 **vtable**(virtual method table)이라고 부릅니다. 컴파일러는 `&Circle`을 `&dyn Shape`로 바꿀 때 "`Circle`용 vtable"의 주소를 함께 붙입니다. 그래서 트레이트 객체를 가리키는 포인터는 보통 포인터의 두 배 크기입니다.

| `&dyn Shape`의 칸 | 내용 |
| --- | --- |
| 첫째 칸 (8바이트) | 값(`Circle`)이 있는 곳의 주소 |
| 둘째 칸 (8바이트) | `Circle`용 vtable의 주소. vtable에는 `name`, `area`에 해당하는 `Circle`의 함수 주소와, 값의 크기·drop 방법 같은 정보가 들어 있습니다. |

[`03_dynamic_dispatch.rs`](examples/03_dynamic_dispatch.rs)의 2번 구역에서 `&Circle`과 `Box<Circle>`은 8바이트, `&dyn Shape`와 `Box<dyn Shape>`는 16바이트로 확인됩니다(64비트 환경). 3번 구역에서는 크기가 같은 두 `Box<dyn Shape>`가 가리키는 값의 크기를 `size_of_val`로 재면 원은 8바이트, 사각형은 16바이트로 다르게 나옵니다. 실제 크기가 vtable에 적혀 있기 때문입니다.

두 방식의 비용을 정리하면 다음과 같습니다.

| | 정적 디스패치 (`T: Shape`, `impl Shape`) | 동적 디스패치 (`dyn Shape`) |
| --- | --- | --- |
| 부를 함수가 정해지는 때 | 컴파일할 때 | 실행 중 |
| 호출 비용 | 함수를 바로 부름. 컴파일러가 본문을 호출 자리에 펼쳐 넣는 최적화(인라인)도 가능 | vtable에서 함수 주소를 읽어 간접 호출. 인라인 불가 |
| 만들어지는 코드 | 쓰인 타입마다 한 벌 (컴파일 시간과 실행 파일 크기 증가) | 한 벌 |
| 섞어 담기 | 불가 | 가능 |

동적 디스패치의 호출 비용은 대부분의 프로그램에서 문제가 되지 않을 만큼 작습니다. 다만 아주 작은 메서드를 반복문 안에서 수없이 부르는 코드라면 인라인을 못 하는 손해가 드러날 수 있습니다. "기본은 제네릭, 여러 타입을 섞어야 할 때 트레이트 객체"로 기억하면 됩니다.

### 2.7 `&dyn Trait`과 `Box<dyn Trait>`

`dyn Shape` 자체는 **크기가 정해지지 않은 타입**입니다. 원일 수도(8바이트) 사각형일 수도(16바이트) 있기 때문입니다. Rust는 변수, 함수 인자, `Vec`의 원소처럼 값을 직접 두는 자리에는 크기를 컴파일할 때 알아야 하므로, `Vec<dyn Shape>`나 `fn f(shape: dyn Shape)`는 E0277(``the size for values of type `dyn Shape` cannot be known at compilation time``)입니다. 포인터는 가리키는 값과 상관없이 크기가 일정하므로, 트레이트 객체는 언제나 포인터 뒤에 둡니다. ch14에서 재귀 타입을 `Box`로 감싼 것과 같은 이유입니다.

| | `&dyn Shape` | `Box<dyn Shape>` |
| --- | --- | --- |
| 값과의 관계 | 빌림 | 소유 |
| 힙 할당 | 없음 (값은 원래 있던 곳에) | 값마다 한 번 |
| 쓸 수 있는 기간 | 빌려 온 값이 살아 있는 동안 | 소유자가 살아 있는 동안 |
| 어울리는 곳 | 함수 인자 (`fn describe(shape: &dyn Shape)`), 잠깐 모아 보기 | 구조체 필드, 함수 반환값, 오래 보관하는 컬렉션 |

```rust
// 스택의 값을 빌려 힙 할당 없이 트레이트 객체 배열을 만듭니다.
let borrowed: [&dyn Shape; 2] = [&circle, &rect];

// 값을 소유하므로 함수 밖으로 돌려줄 수 있습니다.
fn make_shapes() -> Vec<Box<dyn Shape>> {
    vec![Box::new(Circle { radius: 1.0 }), Box::new(Rectangle { width: 2.0, height: 3.0 })]
}
```

`make_shapes`가 `Vec<&dyn Shape>`를 돌려줄 수는 없습니다. 함수 안에서 만든 값의 참조는 함수가 끝나면 댕글링 참조가 되기 때문입니다(ch10 4.5절).

`Box<dyn Shape>`가 들어 있는 벡터의 원소를 `&dyn Shape`를 받는 함수에 넘길 때는 주의할 점이 있습니다. `describe_dynamic(&shapes[1])`은 역참조 강제 변환(ch14)이 일어날 것 같지만, 실제로는 컴파일러가 `Box<dyn Shape>` 자체를 트레이트 객체로 만들려다가 E0277(``the trait `Shape` is not implemented for `Box<dyn Shape>` ``)을 냅니다. `shapes[1].as_ref()`처럼 `as_ref()`로 `Box` 안의 값을 빌려 넘깁니다.

> 컴파일러 메시지에서 `dyn Shape + 'static`이나 `Box<(dyn State + 'static)>`처럼 `'static`이 붙은 표기를 볼 수 있습니다. `Box<dyn Trait>`은 따로 적지 않으면 "안에 든 값이 짧게 사는 참조를 품고 있지 않다"(`'static`, ch10 2.17절)고 가정하기 때문입니다. 이 챕터의 예제에서는 신경 쓰지 않아도 됩니다.

### 2.8 앞 챕터에서 미뤄 둔 두 가지

#### 반환 위치의 `Box<dyn Trait>`

ch10 4.6절에서 `-> impl Summary`인 함수가 갈래에 따라 `NewsArticle`과 `SocialPost`를 돌려주려다 E0308로 막혔습니다. `impl Summary`는 이름만 숨긴 **구체 타입 하나**였기 때문입니다. 그때 컴파일러가 제안한 해결책이 `Box<dyn Summary>`입니다.

```rust
fn returns_summarizable(switch: bool) -> Box<dyn Summary> {
    if switch {
        Box::new(NewsArticle { headline: String::from("러스트 2024 에디션 공개") })
    } else {
        Box::new(SocialPost { username: String::from("ferris") })
    }
}
```

`Box<dyn Summary>`는 "`Summary`를 구현한 어떤 타입의 값을 담은 `Box`"라는 **한 가지 타입**이므로, 두 갈래의 `Box<NewsArticle>`과 `Box<SocialPost>`가 모두 이 타입으로 바뀝니다. 반환 타입이 이미 적혀 있어서 2.4절의 `match`처럼 따로 타입을 표기하지 않아도 됩니다.

| 반환 타입 | 돌려줄 수 있는 것 | 비용 |
| --- | --- | --- |
| `-> impl Summary` | `Summary`를 구현한 타입 **하나** (함수가 정함) | 없음 (정적 디스패치) |
| `-> Box<dyn Summary>` | `Summary`를 구현한 타입 **아무거나**, 실행 중에 골라서 | 힙 할당 + 동적 디스패치 |

#### `Box<dyn Error>`의 정체

ch9와 ch12에서 "어떤 에러든 담을 수 있는 타입"이라고만 했던 `Box<dyn Error>`도 이제 읽을 수 있습니다. `std::error::Error`는 표준 라이브러리의 **트레이트**이고, `Box<dyn Error>`는 "`Error`를 구현한 어떤 타입의 값을 담은 `Box`", 즉 `Box<dyn Draw>`와 똑같은 트레이트 객체입니다.

```rust
fn parse_port(text: &str) -> Result<u16, Box<dyn Error>> {
    let port: u16 = text.trim().parse()?;          // ParseIntError → Box<dyn Error>
    if port < 1024 {
        return Err(Box::new(ReservedPort(port)));  // 직접 만든 에러 → Box<dyn Error>
    }
    Ok(port)
}
```

- `?`는 `ParseIntError`를 `Box`에 담아 `Box<dyn Error>`로 바꿔 돌려줍니다. 이 변환은 `From` 트레이트가 하며 Part 1 `ch6-core-traits`에서 다룹니다. ch9에서 쓴 `"메시지".into()`도 같은 변환입니다.
- 직접 만든 `ReservedPort`도 `Error`를 구현하면 `Box::new`로 감싸 같은 자리에 넣을 수 있습니다. `Error`는 `Debug`와 `Display`를 먼저 구현한 타입에만 구현할 수 있는데(이런 관계를 슈퍼트레이트라고 하며 `ch19-advanced-features`에서 다룹니다), `Error`의 메서드는 모두 기본 구현이 있으므로 `impl Error for ReservedPort {}`처럼 빈 블록이면 충분합니다.
- `println!("{e}")`는 `Box` 안에 든 실제 에러 타입의 `Display` 구현을 부릅니다. 이것도 vtable을 거친 동적 디스패치입니다. [`04_box_dyn_and_enum.rs`](examples/04_box_dyn_and_enum.rs)의 2번 구역에서 `"abc"`는 `ParseIntError`의 메시지를, `"80"`은 `ReservedPort`의 메시지를 출력합니다.

에러 타입마다 다르게 처리해야 한다면 `Box<dyn Error>`로는 부족합니다. 안에 무엇이 들었는지 타입으로 알 수 없기 때문입니다. 그럴 때는 에러 종류를 배리언트로 가진 열거형을 직접 만듭니다(Part 1 `ch8-errors-panic`). 이것도 다음 절의 "트레이트 객체냐 열거형이냐"와 같은 선택입니다.

### 2.9 dyn 호환성: 트레이트 객체로 만들 수 없는 트레이트

모든 트레이트를 `dyn`으로 쓸 수 있는 것은 아닙니다. 트레이트 객체로 메서드를 부르려면 vtable에 **메서드마다 함수 하나**를 적어 둘 수 있어야 하는데, 그렇게 할 수 없는 메서드가 있습니다. 트레이트 객체로 쓸 수 있는 트레이트를 **dyn 호환(dyn compatible)** 이라고 부릅니다. (예전에는 object safety라고 불렀으므로 오래된 글에서는 이 이름을 볼 수 있습니다.)

현재 컴파일러는 이렇게 알려 줍니다.

```text
error[E0038]: the trait `Shape` is not dyn compatible
...
note: for a trait to be dyn compatible it needs to allow building a vtable
...
13 |     fn scaled(&self, factor: f64) -> Self;
   |                                      ^^^^ ...because method `scaled` references the `Self` type in its return type
```

자주 만나는 경우는 두 가지입니다.

| 메서드 모양 | dyn 호환이 아닌 이유 |
| --- | --- |
| `Self`를 돌려줌 `fn scaled(&self, f: f64) -> Self` | `dyn Shape` 뒤의 실제 타입을 모르는 호출자는 돌려받을 값의 크기를 알 수 없습니다. |
| 타입 매개변수가 있음 `fn log<T: Display>(&self, value: T)` | 제네릭 메서드는 `T`마다 한 벌씩 만들어지므로(단형성화), vtable에 칸 하나로 적을 수 없습니다. 이때 메시지는 ``...because method `log` has generic type parameters``입니다. |

표준 라이브러리의 `Clone`도 `clone(&self) -> Self`를 가지므로 `Box<dyn Clone>`은 E0038(`` the trait `Clone` is not dyn compatible ``)입니다. 이 밖의 규칙은 [The Reference의 Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility)에 정리되어 있습니다.

고치는 방법은 세 가지입니다.

1. **그 메서드만 트레이트 객체에서 빼기**: 메서드에 `where Self: Sized`를 붙이면 "크기를 아는 구체 타입에서만 부를 수 있는 메서드"가 되어 vtable에서 빠지고, 트레이트 전체는 dyn 호환이 됩니다. [`03_dynamic_dispatch.rs`](examples/03_dynamic_dispatch.rs)의 `Shape`가 이렇게 되어 있습니다. `circle.scaled(2.0)`은 되지만 `shapes[0].scaled(2.0)`은 `` the `scaled` method cannot be invoked on a trait object ``로 거부됩니다(에러 코드가 없는 메시지입니다).
   ```rust
   fn scaled(&self, factor: f64) -> Self
   where
       Self: Sized;
   ```
2. **메서드를 다른 트레이트로 옮기기**: 컴파일러의 첫 번째 `help:`(``consider moving `scaled` to another trait``)입니다.
3. **열거형으로 바꾸기**: 구현한 타입이 여럿이면 컴파일러가 `consider defining an enum where each variant holds one of these types`라고 제안합니다. 타입의 목록이 정해져 있다면 트레이트 객체 대신 열거형을 쓰라는 뜻이며, 다음 절의 주제입니다.

### 2.10 열거형, 제네릭, 트레이트 객체 중 무엇을 쓸까

"여러 타입을 같은 코드로 다룬다"는 문제를 푸는 도구가 이제 셋입니다. [`04_box_dyn_and_enum.rs`](examples/04_box_dyn_and_enum.rs)의 3번 구역은 ch8의 `Cell` 열거형과 같은 일을 트레이트 객체로도 해 봅니다.

```rust
// 열거형: 담을 수 있는 종류를 모두 나열합니다.
enum Cell { Int(i32), Float(f64), Text(String) }

// 트레이트 객체: 칸이 갖춰야 할 동작만 정합니다.
trait CellValue {
    fn render(&self) -> String;
    fn number(&self) -> Option<f64> { None }
}
impl CellValue for i32 { /* ... */ }
impl CellValue for f64 { /* ... */ }
impl CellValue for String { /* ... */ }
impl CellValue for bool { /* 나중에 추가. 위의 코드는 한 줄도 고치지 않습니다. */ }
```

두 방식은 "무엇을 추가하기 쉬운가"가 정반대입니다.

| | 열거형 | 트레이트 객체 |
| --- | --- | --- |
| 새 **종류** 추가 (`bool` 칸) | 배리언트를 추가하고 **모든** `match`를 고쳐야 함 | 새 타입에 `impl`만 추가. 다른 크레이트에서도 가능 |
| 새 **동작** 추가 (`is_empty` 같은 메서드) | `match` 하나짜리 메서드만 추가 | 트레이트에 메서드를 추가하고 **모든** `impl`을 고쳐야 함 (기본 구현이 있으면 덜함) |
| 빠뜨린 경우 검사 | `match`의 완전성 검사가 잡아 줌 | 각 타입이 트레이트를 구현했는지만 검사 |
| 값을 꺼내 쓰기 | `match`로 배리언트 안의 값을 꺼낼 수 있음 | 트레이트의 메서드로만 다룰 수 있음 |
| 저장과 호출 | 값을 그대로 저장, `match`로 분기 | 값마다 힙 할당, vtable로 간접 호출 |

선택 기준을 정리하면 다음과 같습니다.

- **종류가 정해져 있고 내가 모두 안다** → 열거형. ch8의 스프레드시트 칸, ch6의 주문 상태 `OrderStatus`처럼 목록이 닫혀 있는 경우입니다. 2.9절에서 dyn 호환이 아니라서 막힌 경우에도 대안이 됩니다.
- **한 번에 한 가지 타입만 다룬다** → 제네릭. 섞어 담을 필요가 없다면 실행 비용이 가장 적습니다.
- **종류를 미리 다 알 수 없거나, 쓰는 사람이 새 타입을 추가해야 한다** → 트레이트 객체. GUI 컴포넌트, 플러그인, 에러(`Box<dyn Error>`)처럼 목록이 열려 있는 경우입니다.

### 2.11 상태 패턴 1: 트레이트 객체로 상태 표현하기

**상태 패턴(state pattern)** 은 값이 가질 수 있는 상태마다 **상태 객체**를 하나씩 두고, 상태에 따라 달라지는 동작을 그 상태 객체에게 맡기는 객체 지향 설계 패턴입니다. 값 자신은 "지금 상태 객체"만 들고 있다가 일을 넘기고, 상태 객체가 다음 상태로 바꿀지를 정합니다.

예로 블로그 글의 작업 흐름을 만듭니다.

1. 새 글은 **초안**으로 시작합니다.
2. 초안을 다 쓰면 **검토**를 요청합니다.
3. 검토 중인 글이 승인되면 **게시**됩니다.
4. 게시된 글만 내용을 보여 주고, 그 전에는 빈 문자열을 돌려줍니다. 순서를 어긴 요청(초안을 바로 승인 등)은 아무 효과가 없습니다.

```rust
pub struct Post {
    state: Option<Box<dyn State>>,  // 지금 상태 객체
    content: String,
}

trait State {
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>) -> Box<dyn State>;
    fn content<'a>(&self, _post: &'a Post) -> &'a str { "" } // 기본 구현: 내용을 보여 주지 않음
}

struct Draft;          // 각 상태는 State를 구현한 구조체
struct PendingReview;
struct Published;      // content를 재정의해 &post.content를 돌려주는 유일한 상태
```

세 가지 장치를 하나씩 봅니다.

**`self: Box<Self>`**: 지금까지 본 `self`, `&self`, `&mut self`(ch5)와 달리, "이 메서드는 `Box`에 담긴 값에서만 부를 수 있고, 그 `Box`의 소유권을 가져간다"는 뜻입니다. 상태를 바꾸는 메서드는 옛 상태를 소비하고 새 상태를 돌려줍니다. 바뀔 필요가 없으면 받은 `self`를 그대로 돌려줍니다.

```rust
impl State for Draft {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        Box::new(PendingReview)  // 옛 Draft는 여기서 소비되고 버려집니다.
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        self                     // 초안은 승인할 수 없으므로 그대로
    }
}
```

**`Option`과 `take()`**: `Post::request_review`는 `&mut self`를 받으므로 `self.state`의 소유권을 그냥 가져갈 수 없습니다. 상태가 `Box<dyn State>`였다면 `self.state = self.state.request_review();`는 E0507(``cannot move out of `self.state` which is behind a mutable reference``)입니다. 꺼내 가는 동안 필드가 빈 채로 남는 것을 Rust가 허락하지 않기 때문입니다. 그래서 상태를 `Option`에 담고, `take()`로 꺼내면서 그 자리에 `None`을 남깁니다.

```rust
pub fn request_review(&mut self) {
    if let Some(s) = self.state.take() {     // 꺼내고 None을 남김
        self.state = Some(s.request_review()) // 새 상태로 다시 채움
    }
}
```

`take()`한 뒤 바로 다시 채우므로 메서드가 끝났을 때 `state`는 언제나 `Some`입니다. 그래서 `content`에서는 `self.state.as_ref().unwrap()`을 써도 패닉이 나지 않습니다. 이것도 `state` 필드를 비공개로 둔 덕분에 지킬 수 있는 불변식입니다(`as_ref()`는 `Option<Box<dyn State>>`를 빌려 `Option<&Box<dyn State>>`로 바꿉니다).

**상태에게 일을 맡기는 `content`**: `Post::content`는 "지금 내용을 보여 줘도 되는가"를 스스로 판단하지 않고 상태에게 자신을 넘겨 묻습니다. 기본 구현은 `""`이고 `Published`만 재정의합니다. 반환하는 참조가 `post`에서 빌린 것이므로 라이프타임 `'a`로 둘을 잇습니다(ch10).

[`05_blog_state_objects.rs`](examples/05_blog_state_objects.rs)의 `main`은 처음부터 끝까지 `Post` 타입의 변수 `post` 하나만 다룹니다. 상태가 몇 개인지, 어떻게 바뀌는지는 `blog` 모듈 안에 숨어 있습니다. `State` 트레이트와 상태 구조체에 `pub`이 없으므로 바깥에서는 보이지도 않습니다.

### 2.12 상태 패턴 2: 상태를 타입으로 표현하기

2.11절의 구현은 다른 객체 지향 언어에서 쓰는 방식을 그대로 옮긴 것입니다. 그런데 Rust의 타입 시스템과 소유권을 이용하면 **잘못된 순서의 호출을 아예 컴파일되지 않게** 만들 수 있습니다. 상태마다 별도의 **타입**을 두고, 그 상태에서 할 수 있는 메서드만 붙이는 것입니다.

```rust
pub struct DraftPost { content: String }          // add_text, request_review만 있음
pub struct PendingReviewPost { content: String }  // approve만 있음
pub struct Post { content: String }               // content만 있음

impl Post {
    pub fn new() -> DraftPost { /* ... */ }  // 새 글은 언제나 초안
    pub fn content(&self) -> &str { &self.content }
}

impl DraftPost {
    pub fn add_text(&mut self, text: &str) { self.content.push_str(text); }
    pub fn request_review(self) -> PendingReviewPost {
        PendingReviewPost { content: self.content }
    }
}

impl PendingReviewPost {
    pub fn approve(self) -> Post {
        Post { content: self.content }
    }
}
```

- **초안에는 `content` 메서드가 없습니다.** `post.content()`는 빈 문자열을 돌려주는 대신 E0599로 컴파일되지 않습니다(4.3절). 초안의 `approve()`, 검토 중인 글의 `add_text()`도 마찬가지로 E0599입니다.
- **상태를 바꾸는 메서드는 `self`를 값으로 받습니다.** 호출하면 옛 값이 소비되므로(ch4의 이동), 이미 검토를 요청한 초안에 글을 더 쓰려고 하면 E0382(``borrow of moved value: `draft` ``)입니다. 내용 `String`은 복사 없이 새 값으로 옮겨집니다.
- **쓰는 쪽은 돌려받은 값을 다시 받아야 합니다.** 그래서 `main`은 섀도잉(ch3)으로 같은 이름을 다시 씁니다.
  ```rust
  let mut post = Post::new();          // DraftPost
  post.add_text("오늘 점심은 샐러드를 먹었다");
  let post = post.request_review();    // PendingReviewPost
  let post = post.approve();           // Post
  println!("{}", post.content());
  ```
- **필드가 비공개라 `Post`를 직접 만들 수 없습니다.** 게시된 `Post`를 얻는 유일한 길이 `approve`이므로, `-> Post`를 돌려주는 함수는 반드시 모든 단계를 거쳐야 합니다. 캡슐화와 타입이 함께 규칙을 지킵니다.

`Post::new()`가 `Post`가 아닌 `DraftPost`를 돌려주면 Clippy가 `new_ret_no_self` 경고를 냅니다. `new`는 보통 자기 타입을 돌려주기 때문입니다. 여기서는 "글은 언제나 초안으로 시작한다"는 의도된 설계라서, [`06_blog_state_types.rs`](examples/06_blog_state_types.rs)에서는 `#[expect(clippy::new_ret_no_self, reason = "...")]`로 이유를 적어 두었습니다. 이름을 `Post::draft()`처럼 바꾸는 것도 방법입니다.

### 2.13 두 방식의 비교, 그리고 열거형

| | 트레이트 객체 (05) | 타입 (06) |
| --- | --- | --- |
| 잘못된 순서의 호출 | 컴파일됨. 실행 중에 조용히 무시 | 컴파일 에러 (E0599) |
| 쓰는 쪽의 코드 | 변수 하나(`Post`)로 메서드만 부름. 상태 전이가 `Post` 안에 캡슐화됨 | 단계마다 돌려받은 값을 다시 받아야 함 (`let post = post.approve();`) |
| 새 상태 추가 | 구조체 하나와 `impl State` 추가. 다만 앞뒤 상태의 전이 코드도 고쳐야 함 | 타입 하나와 메서드 추가. 앞뒤 타입의 메서드도 고쳐야 함 |
| "아무 상태의 글"을 한 곳에 담기 | `Vec<Post>`로 바로 가능 | 타입이 다르므로 그대로는 불가 (열거형이나 트레이트 객체가 또 필요) |
| 실행 비용 | 상태가 바뀔 때마다 힙 할당, 동적 디스패치, `unwrap` 검사 | 없음 |

트레이트 객체 방식에는 The Book이 지적하는 단점이 두 가지 더 있습니다.

- **상태끼리 서로를 압니다.** `PendingReview`가 `Published`를 직접 만들어 돌려주므로, 둘 사이에 "예약됨" 상태를 끼워 넣으려면 `PendingReview`도 고쳐야 합니다.
- **중복이 생깁니다.** "바뀌지 않으면 `self`를 돌려준다"는 같은 코드가 여러 상태에 반복됩니다. 이것을 `State`의 기본 구현으로 옮길 수도 없습니다. 기본 구현 안의 `Self`는 크기를 모르는 타입일 수도 있어서, `Box<Self>`를 `Box<dyn State>`로 바꿀 수 없기 때문입니다. 실제로 기본 구현에 `self`를 쓰면 E0277(``the size for values of type `Self` cannot be known at compilation time``)입니다.

**세 번째 선택지: 열거형.** 상태를 열거형 배리언트로 표현할 수도 있습니다. 상태가 데이터를 갖지 않으므로 `Copy`를 derive해 두면 `match`로 다음 상태를 고르기 쉽습니다.

```rust
#[derive(Clone, Copy)]
enum State {
    Draft,
    PendingReview,
    Published,
}

pub struct Post {
    state: State,
    content: String,
}

impl Post {
    pub fn approve(&mut self) {
        self.state = match self.state {
            State::PendingReview => State::Published,
            other => other, // 다른 상태에서는 그대로 둡니다.
        };
    }

    pub fn content(&self) -> &str {
        match self.state {
            State::Published => &self.content,
            State::Draft | State::PendingReview => "",
        }
    }
}
```

힙 할당도 `Option`도 필요 없고 상태 목록이 한눈에 보입니다. 대신 상태에 따라 달라지는 메서드마다 `match`가 하나씩 생기고, 상태를 추가하면 그 `match`를 모두 고쳐야 합니다(2.10절의 표와 같은 이야기입니다). 잘못된 순서의 호출이 실행 중에 무시된다는 점은 트레이트 객체 방식과 같습니다.

어느 것이 정답이라기보다, **규칙을 컴파일러가 검사하게 할 수 있다면 타입으로**, 상태가 적고 단순하면 **열거형으로**, 상태별 동작이 크고 다양해서 각자의 구조체로 나누는 편이 읽기 좋다면 **트레이트 객체로** 고릅니다. 다른 언어의 객체 지향 패턴을 Rust에서 그대로 쓸 수는 있지만, 소유권과 타입 시스템을 쓰면 더 나은 설계가 있는 경우가 많습니다.

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_encapsulation.rs`](examples/01_encapsulation.rs) | `AveragedCollection`으로 불변식 지키기, 필드를 공개했을 때의 문제, 내부 표현 교체 | 1 |
| [`02_gui_trait_objects.rs`](examples/02_gui_trait_objects.rs) | `Vec<Box<dyn Draw>>` GUI, 라이브러리 밖에서 만든 타입, 기본 구현, 실행 중 타입 선택, `GenericScreen<T>` | 1, 2 |
| [`03_dynamic_dispatch.rs`](examples/03_dynamic_dispatch.rs) | 정적·동적 디스패치, 포인터 크기, `&dyn`과 `Box<dyn>`, `where Self: Sized` | 3, 4 |
| [`04_box_dyn_and_enum.rs`](examples/04_box_dyn_and_enum.rs) | 반환 위치 `Box<dyn Summary>`, `Box<dyn Error>`, ch8 `Cell` 열거형과 트레이트 객체 비교 | 3, 4 |
| [`05_blog_state_objects.rs`](examples/05_blog_state_objects.rs) | 트레이트 객체로 만든 상태 패턴, `Option::take`, `self: Box<Self>` | 5 |
| [`06_blog_state_types.rs`](examples/06_blog_state_types.rs) | 상태를 타입으로 표현한 블로그 글, 섀도잉으로 바뀌는 타입 | 5 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 4개 (4절) | 1, 2, 4, 5 |

### 01_encapsulation — 캡슐화와 불변식

`cached`, `on_demand`, `leaky` 세 모듈이 같은 일을 하는 타입을 하나씩 정의합니다. 2번 구역은 값을 모두 꺼낸 뒤 빈 목록에서 `remove()`가 `None`을 돌려주고 평균이 `0.00`이 되는 것까지 확인합니다. 3번 구역의 `77.50`은 갱신되지 않고 남은 옛 평균입니다(실제 평균은 51.67). 4번 구역의 두 블록은 `use` 줄 말고는 같은 코드입니다.

```sh
cargo run -p p0-ch17-trait-objects-oop --example 01_encapsulation
```

```text
1. 객체 = 데이터(필드) + 동작(메서드)
    값 3개, 평균 81.67
2. 공개 메서드로만 바꿀 수 있으므로 평균이 언제나 맞습니다
    remove() = Some(90), 남은 값 2개, 평균 77.50
    모두 꺼낸 뒤: remove() = None, 평균 0.00
3. 필드가 공개되어 있으면 바깥 코드가 불변식을 깰 수 있습니다
    list = [70, 85, 0], average = 77.50
    average를 덮어쓴 뒤: 100.00
4. 내부 표현이 달라도 공개 API가 같으면 쓰는 코드는 그대로입니다
    cached:    꺼낸 값 Some(8), 남은 값 2개, 평균 3.50
    on_demand: 꺼낸 값 Some(8), 남은 값 2개, 평균 3.50
```

### 02_gui_trait_objects — 트레이트 객체로 만든 GUI

`gui` 모듈이 라이브러리 역할을 하고, 그 아래의 `SelectBox`와 `main`이 라이브러리를 쓰는 사용자 코드입니다. 실제로 화면에 그리는 대신 `draw`가 그린 모양을 문자열로 돌려줍니다. 줄 앞의 `*`는 `focusable()`이 `true`인 컴포넌트입니다. `Label`은 기본 구현을 그대로 써서 `*`가 없습니다. 3번 구역은 문자열 `layout`을 읽어 실행 중에 컴포넌트의 타입을 고르고, 4번 구역은 같은 타입만 담는 제네릭 버전입니다.

```sh
cargo run -p p0-ch17-trait-objects-oop --example 02_gui_trait_objects
```

```text
1. 서로 다른 세 타입을 Vec<Box<dyn Draw>> 하나에 담아 그립니다 (*는 포커스 가능)
      배송 방법을 고르세요
    * <택배 | 퀵 | 방문 수령> (75x10)
    * [ OK ] (50x10)
2. 트레이트 객체로는 트레이트의 메서드만 부를 수 있습니다
    컴포넌트 3개 중 포커스를 받는 것 2개
3. 어떤 타입을 담을지 실행 중에 데이터를 보고 정할 수 있습니다
      로그인이 필요합니다
    * [ 로그인 ] (40x10)
    * [ 취소 ] (40x10)
4. 제네릭 버전 GenericScreen<T>는 한 가지 타입만 담습니다
    [ 예 ] (30x10)
    [ 아니요 ] (30x10)
```

### 03_dynamic_dispatch — 정적·동적 디스패치와 두 포인터

1번 구역은 같은 본문의 `describe_static`과 `describe_dynamic`이 같은 결과를 낸다는 것을, 2번 구역은 트레이트 객체 포인터가 두 칸(16바이트)이라는 것을 보여 줍니다. 크기는 64비트 환경에서 확인한 값입니다. `Shape` 트레이트의 `scaled`에서 `where Self: Sized`를 지우면 E0038이 납니다(4.2절).

```sh
cargo run -p p0-ch17-trait-objects-oop --example 03_dynamic_dispatch
```

```text
1. 같은 본문, 다른 디스패치: 결과는 같습니다
    정적: 반지름 1인 원: 넓이 3.14
    정적: 2x3 사각형: 넓이 6.00
    동적: 반지름 1인 원: 넓이 3.14
    동적: 2x3 사각형: 넓이 6.00
2. 트레이트 객체 포인터는 두 칸입니다 (64비트 환경)
    &Circle:         8바이트
    &dyn Shape:      16바이트
    Box<Circle>:     8바이트
    Box<dyn Shape>:  16바이트
3. 포인터의 크기는 같아도, 가리키는 값의 크기는 타입마다 다릅니다
    반지름 1인 원: 포인터 16바이트, 값 8바이트
    2x3 사각형: 포인터 16바이트, 값 16바이트
4. &dyn은 빌려서, Box<dyn>은 소유해서 가리킵니다
    빌린 도형 넓이 합계: 9.14
    소유한 도형 넓이 합계: 9.14
    2x3 사각형: 넓이 6.00
5. `where Self: Sized` 메서드는 구체 타입에서만 부를 수 있습니다
    반지름 2인 원: 넓이 12.57
```

### 04_box_dyn_and_enum — `Box<dyn Trait>`의 쓰임새와 열거형

1번 구역은 ch10 4.6절에서 E0308로 막혔던 `returns_summarizable`을 `Box<dyn Summary>`로 고친 것입니다. 2번 구역의 `{input:>5}`는 입력을 5칸에 오른쪽 정렬하는 서식 지정입니다. `ParseIntError`의 메시지는 표준 라이브러리가 정한 영어 문장입니다. 3번 구역의 `Cell` 크기 24바이트는 64비트 환경 값이며, 배리언트를 구분하는 정보가 `String` 안의 쓰지 않는 자리에 들어가서 `String`과 같은 크기가 됩니다(그 원리는 Part 1 `ch7-smart-pointers-memory`에서 다룹니다).

```sh
cargo run -p p0-ch17-trait-objects-oop --example 04_box_dyn_and_enum
```

```text
1. 반환 타입 Box<dyn Summary>: 갈래마다 다른 타입을 돌려줄 수 있습니다
    switch = true: [기사] 러스트 2024 에디션 공개
    switch = false: [글] @ferris
2. Box<dyn Error>에는 Error를 구현한 어떤 에러든 담깁니다
     8080 → 포트 8080
      abc → 에러: invalid digit found in string
    70000 → 에러: number too large to fit in target type
       80 → 에러: 80번 포트는 시스템용으로 예약되어 있습니다
    Box<dyn Error>의 크기: 16바이트
3. 같은 행을 열거형과 트레이트 객체로 담아 봅니다
    열거형:        ["사과", "3", "1200.5"], 숫자 합계 1203.5
    트레이트 객체: ["사과", "3", "1200.5", "예"], 숫자 합계 1203.5
    원소 하나의 크기: Cell 24바이트, Box<dyn CellValue> 16바이트
```

### 05_blog_state_objects — 트레이트 객체로 만든 상태 패턴

1번 구역은 `Option::take`의 동작을 따로 보여 줍니다. 3번 구역의 `post.approve()`는 컴파일되지만 초안의 `approve`가 자기 자신을 돌려줄 뿐이라 상태가 그대로입니다. 06과 비교할 때 이 줄을 눈여겨보세요. 예제에는 상태 이름을 출력하려고 The Book에 없는 `state_name`/`name` 메서드를 더했습니다.

```sh
cargo run -p p0-ch17-trait-objects-oop --example 05_blog_state_objects
```

```text
1. Option::take는 값을 꺼내고 그 자리에 None을 남깁니다
    taken = Some("상태"), slot = None
2. 새 글은 초안이고, 초안의 내용은 보이지 않습니다
    상태: 초안, content() = ""
3. 초안을 바로 승인하려 해도 아무 일도 일어나지 않습니다
    상태: 초안, content() = ""
4. 검토를 요청하면 검토 중이 됩니다
    상태: 검토 중, content() = ""
5. 승인되면 게시되고, 이제야 내용이 보입니다
    상태: 게시됨, content() = "오늘 점심은 샐러드를 먹었다"
6. main은 처음부터 끝까지 같은 변수 post 하나만 다룹니다
    상태: 게시됨
```

### 06_blog_state_types — 상태를 타입으로 표현한 블로그 글

`short_type_name`은 변수의 타입 이름을 출력해, 같은 이름 `post`의 타입이 단계마다 바뀌는 것을 보여 주는 도우미 함수입니다. 안에서 쓰는 `std::any::type_name`은 디버깅용이라 정확한 표기가 컴파일러 버전에 따라 달라질 수 있으므로 프로그램의 동작을 여기에 기대면 안 됩니다. 주석 처리된 호출(`post.content()`, `post.approve()`, `post.add_text(...)`)의 주석을 풀면 E0599를 직접 볼 수 있습니다.

```sh
cargo run -p p0-ch17-trait-objects-oop --example 06_blog_state_types
```

```text
1. Post::new()는 Post가 아니라 DraftPost를 돌려줍니다
    post의 타입: DraftPost
2. 검토를 요청하면 PendingReviewPost가 됩니다
    post의 타입: PendingReviewPost
3. 승인하면 비로소 content()가 있는 Post가 됩니다
    post의 타입: Post
    content() = "오늘 점심은 샐러드를 먹었다"
4. 단계를 거치지 않고는 Post를 얻을 수 없습니다
    Post: 내일은 비빔밥
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch17-trait-objects-oop
rustc --edition 2024 compile_fail/e0277_not_draw.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요. 에러 코드의 자세한 설명은 `rustc --explain E0038`처럼 볼 수 있습니다.

### 4.1 트레이트를 구현하지 않은 타입을 트레이트 객체로 — E0277

파일: [`compile_fail/e0277_not_draw.rs`](compile_fail/e0277_not_draw.rs)

```text
error[E0277]: the trait bound `String: Draw` is not satisfied
  --> compile_fail/e0277_not_draw.rs:45:13
   |
45 |             Box::new(String::from("Hi")), // String은 Draw를 구현하지 않았습니다.
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Draw` is not implemented for `String`
   |
help: the trait `Draw` is implemented for `Button`
  --> compile_fail/e0277_not_draw.rs:30:5
   |
30 |     impl Draw for Button {
   |     ^^^^^^^^^^^^^^^^^^^^
   = note: required for the cast from `Box<String>` to `Box<dyn Draw>`
```

- **읽는 법**: 마지막 `note:`가 핵심입니다. `Box<String>`을 `Box<dyn Draw>`로 바꾸는(cast) 데 `String: Draw`가 필요한데, 그 구현이 없다는 뜻입니다. ch10 4.2절의 E0277과 같은 에러이며, 바운드 대신 트레이트 객체로의 변환이 조건을 요구했다는 점만 다릅니다.
- **원인**: `Vec<Box<dyn Draw>>`에는 `Draw`를 구현한 타입의 값만 들어갈 수 있습니다. 동적 타입 언어라면 실행 중에 `draw`를 부르는 순간 실패했을 코드를 Rust는 컴파일할 때 거부합니다.
- **해결**: 의도한 값이 글자 한 줄이었다면 `Label`처럼 `Draw`를 구현한 타입으로 감싸 넣습니다. `String` 자체를 컴포넌트로 쓰고 싶다면 `impl Draw for String { ... }`을 작성합니다(내 트레이트이므로 고아 규칙이 허용합니다).

### 4.2 dyn 호환이 아닌 트레이트로 트레이트 객체 만들기 — E0038

파일: [`compile_fail/e0038_dyn_incompatible.rs`](compile_fail/e0038_dyn_incompatible.rs)

```text
error[E0038]: the trait `Shape` is not dyn compatible
  --> compile_fail/e0038_dyn_incompatible.rs:51:29
   |
51 |     let shapes: Vec<Box<dyn Shape>> = vec![
   |                             ^^^^^ `Shape` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> compile_fail/e0038_dyn_incompatible.rs:13:38
   |
11 | trait Shape {
   |       ----- this trait is not dyn compatible...
12 |     fn area(&self) -> f64;
13 |     fn scaled(&self, factor: f64) -> Self;
   |                                      ^^^^ ...because method `scaled` references the `Self` type in its return type
   = help: consider moving `scaled` to another trait
   = help: the following types implement `Shape`:
             Circle
             Rectangle
           consider defining an enum where each variant holds one of these types,
           implementing `Shape` for this new enum and using it instead
```

- **읽는 법**: 첫 줄은 "어디서" 트레이트 객체를 만들려 했는지, `note:` 아래의 `...because` 줄은 "왜" 안 되는지(문제가 된 메서드와 이유)를 가리킵니다. `this trait is not dyn compatible...`과 `...because`가 한 문장으로 이어집니다.
- **원인**: `scaled`가 `Self`를 돌려주므로 vtable을 만들 수 없습니다(2.9절). 에러가 난 곳은 `Vec<Box<dyn Shape>>`이지만, 고칠 곳은 트레이트 정의입니다.
- **해결**: `fn scaled(&self, factor: f64) -> Self where Self: Sized;`로 트레이트 객체에서는 부를 수 없는 메서드로 표시합니다([`03_dynamic_dispatch.rs`](examples/03_dynamic_dispatch.rs)). `help:`처럼 `scaled`를 다른 트레이트로 옮기거나, 도형의 종류가 정해져 있다면 열거형으로 바꾸는 것도 방법입니다. 구현한 타입이 하나뿐이면 `help:` 대신 ``only type `Square` implements `Shape`; consider using it directly instead``처럼 트레이트 객체 없이 그 타입을 쓰라고 제안합니다.

### 4.3 초안의 내용을 읽으려 함 (타입형 상태 패턴) — E0599

파일: [`compile_fail/e0599_draft_content.rs`](compile_fail/e0599_draft_content.rs)

```text
error[E0599]: no method named `content` found for struct `DraftPost` in the current scope
  --> compile_fail/e0599_draft_content.rs:60:25
   |
14 |     pub struct DraftPost {
   |     -------------------- method `content` not found for this struct
...
60 |     println!("{}", post.content()); // post는 아직 DraftPost입니다.
   |                         ^^^^^^^ private field, not a method
```

- **읽는 법**: `post`의 타입이 `DraftPost`라는 것을 첫 줄에서 확인합니다. 아래의 `private field, not a method`는 "`DraftPost`에 `content`라는 이름의 비공개 **필드**는 있지만 **메서드**는 없다"는 뜻입니다. 필드를 공개하라는 뜻이 아닙니다.
- **원인**: 타입형 설계에서는 `content()`가 게시된 `Post`에만 있습니다. 객체형([`05_blog_state_objects.rs`](examples/05_blog_state_objects.rs))이라면 빈 문자열이 나왔을 호출이 여기서는 컴파일 에러가 됩니다. 이것이 타입형 설계가 노린 효과입니다.
- **해결**: `let post = post.request_review();`, `let post = post.approve();`로 단계를 거쳐 `Post`를 얻은 뒤 `content()`를 부릅니다.

### 4.4 캡슐화된 비공개 필드에 직접 접근 — E0616

파일: [`compile_fail/e0616_private_field.rs`](compile_fail/e0616_private_field.rs)

```text
error[E0616]: field `list` of struct `AveragedCollection` is private
  --> compile_fail/e0616_private_field.rs:44:12
   |
44 |     scores.list.push(100); // add를 거치지 않고 list를 직접 바꾸려 합니다.
   |            ^^^^ private field
```

- **원인**: `list`에 `pub`이 없어서 `stats` 모듈 밖에서는 보이지 않습니다(ch7 4.2절과 같은 에러). 이 제약 덕분에 `average`를 갱신하지 않은 채 `list`만 바뀌는 일이 생기지 않습니다.
- **해결**: 공개 메서드 `scores.add(100)`을 씁니다. 에러를 없애려고 필드에 `pub`을 붙이는 것은 캡슐화를 버리는 것입니다([`01_encapsulation.rs`](examples/01_encapsulation.rs)의 `LeakyCollection`이 그 결과를 보여 줍니다). 비공개 메서드 `update_average`를 바깥에서 부르려 하면 비슷한 E0624(``method `update_average` is private``)가 납니다.

### 4.5 그 밖에 이 챕터에서 만나는 에러

| 상황 | 에러 | 해결 |
| --- | --- | --- |
| `Vec<Box<Draw>>`처럼 `dyn`을 빠뜨림 | E0782 (`expected a type, found a trait`) | `Box<dyn Draw>`로 씁니다(2.4절). |
| 갈래마다 다른 `Box`를 만드는 `match`에 타입 표기가 없음 | E0308 (`` `match` arms have incompatible types ``) | `let c: Box<dyn Draw> = match ...`로 목표 타입을 적습니다(2.4절). |
| 트레이트 객체로 실제 타입의 필드 사용 | E0609 (``no field `text` on type `Box<dyn Draw>` ``) | 필요한 동작을 트레이트 메서드로 만듭니다. |
| `GenericScreen<Button>`에 다른 타입 섞기 | E0308 (``expected `Button`, found `SelectBox` ``) | 섞어야 한다면 트레이트 객체 버전을 씁니다(2.5절). |
| `Vec<dyn Shape>`, `fn f(s: dyn Shape)` | E0277 (``the size for values of type `dyn Shape` cannot be known at compilation time``) | `Box<dyn Shape>`나 `&dyn Shape`처럼 포인터 뒤에 둡니다(2.7절). |
| `&dyn Shape` 자리에 `&Box<dyn Shape>` 넘기기 | E0277 (``the trait `Shape` is not implemented for `Box<dyn Shape>` ``) | `boxed.as_ref()`로 넘깁니다(2.7절). |
| `where Self: Sized` 메서드를 트레이트 객체로 호출 | 코드 없음 (`` the `scaled` method cannot be invoked on a trait object ``) | 구체 타입의 값에서 부릅니다(2.9절). |
| 상태를 `Option` 없이 `Box<dyn State>`로 두고 `&mut self`에서 옮기기 | E0507 (``cannot move out of `self.state` which is behind a mutable reference``) | `Option`에 담고 `take()`로 꺼냅니다(2.11절). |
| 타입형 상태 패턴에서 소비된 옛 값을 다시 사용 | E0382 (``borrow of moved value: `draft` ``) | 메서드가 돌려준 새 값을 씁니다(2.12절). |

## 5. 연습 문제

### 연습 1. 새 컴포넌트 `TextField`

[`02_gui_trait_objects.rs`](examples/02_gui_trait_objects.rs)의 사용자 코드 쪽(`gui` 모듈 밖)에 `TextField { width: u32, placeholder: String }`를 정의하고 `Draw`를 구현하세요. `draw`는 `[ 이메일을 입력하세요 ] (60칸)`처럼 그리고, 포커스를 받아야 합니다. 1번 구역의 `screen`에 `TextField`를 넣어 실행했을 때 줄 앞에 `*`가 붙는지 확인하세요. 그다음 4번 구역의 `GenericScreen`에도 `TextField`를 넣어 보고, 어떤 에러가 나는지 예측한 뒤 확인하세요.

<details>
<summary>힌트</summary>

`SelectBox`와 같은 모양으로 `impl Draw for TextField { fn draw(&self) -> String { format!("[ {} ] ({}칸)", self.placeholder, self.width) } fn focusable(&self) -> bool { true } }`를 작성합니다. `focusable`을 재정의하지 않으면 기본 구현이 `false`를 돌려주므로 `*`가 붙지 않습니다. `gui` 모듈은 고칠 필요가 없습니다. `GenericScreen`의 `components`는 이미 `Button`으로 정해진 `Vec<T>`이므로 E0308(``expected `Button`, found `TextField` ``)이 납니다(2.5절).

</details>

### 연습 2. 검토에서 반려하기

[`05_blog_state_objects.rs`](examples/05_blog_state_objects.rs)에 `reject` 메서드를 추가하세요. 검토 중인 글을 반려하면 초안으로 돌아가고, 초안이나 게시된 글을 반려하면 아무 일도 일어나지 않습니다. `main`에서 "검토 요청 → 반려 → 상태: 초안"을 출력해 확인하세요.

<details>
<summary>힌트</summary>

`State` 트레이트에 `fn reject(self: Box<Self>) -> Box<dyn State>;`를 추가하면, 컴파일러가 세 상태 모두에 구현을 요구합니다(E0046). `PendingReview`는 `Box::new(Draft)`를, 나머지 둘은 `self`를 돌려줍니다. `Post`에는 `request_review`와 똑같은 모양으로 `pub fn reject(&mut self) { if let Some(s) = self.state.take() { self.state = Some(s.reject()) } }`를 추가합니다. 새 동작 하나를 위해 트레이트와 모든 상태를 고쳐야 했다는 점을 2.10절의 표와 비교해 보세요.

</details>

### 연습 3. 두 번 승인받아야 게시

연습 2에 이어, 검토 중인 글은 `approve`를 **두 번** 받아야 게시되도록 바꾸세요. 한 번 승인한 뒤의 상태는 여전히 "검토 중"이고 `content()`는 빈 문자열이어야 합니다. 반려되면 승인 횟수는 처음부터 다시 셉니다.

<details>
<summary>힌트</summary>

상태 구조체도 필드를 가질 수 있습니다. `struct PendingReview { approvals: u32 }`로 바꾸고, `Draft::request_review`는 `Box::new(PendingReview { approvals: 0 })`을 돌려줍니다. `PendingReview::approve`에서 `self.approvals + 1`이 2 이상이면 `Box::new(Published)`를, 아니면 `Box::new(PendingReview { approvals: self.approvals + 1 })`을 돌려줍니다. `self: Box<Self>`로 받은 `self`에서도 `self.approvals`처럼 필드를 읽을 수 있습니다. 반려하면 새 `Draft`가 되므로 다음 검토 요청 때 횟수가 자연스럽게 0부터 시작합니다. `Post`의 공개 메서드와 `main`의 호출 코드는 바꿀 필요가 없다는 점도 확인하세요.

</details>

### 연습 4. 타입형 설계에서 반려하기

[`06_blog_state_types.rs`](examples/06_blog_state_types.rs)에 반려 기능을 추가하세요. 반려된 글은 다시 고칠 수 있어야 합니다. `main`에서 "초고 작성 → 검토 요청 → 반려 → `, 수정본` 추가 → 검토 요청 → 승인"을 거쳐 `content()`가 `초고, 수정본`을 출력하는지 확인하세요. 게시된 `Post`나 `DraftPost`에서 `reject`를 부르면 어떻게 되나요?

<details>
<summary>힌트</summary>

`impl PendingReviewPost`에 `pub fn reject(self) -> DraftPost { DraftPost { content: self.content } }`를 추가하면 됩니다. 연습 2와 달리 다른 타입은 고치지 않습니다. 반려 뒤에 글을 고치려면 가변 바인딩이 필요하므로 `let mut post = post.reject();`로 받습니다. `reject`는 `PendingReviewPost`에만 있으므로 다른 상태에서 부르면 실행 중에 무시되는 대신 E0599로 컴파일되지 않습니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [18. Object-Oriented Programming Features](https://doc.rust-lang.org/book/ch18-00-oop.html)
  - [18.1 Characteristics of Object-Oriented Languages](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html): 2.1~2.3절. `AveragedCollection`, 상속과 다형성
  - [18.2 Using Trait Objects to Abstract over Shared Behavior](https://doc.rust-lang.org/book/ch18-02-trait-objects.html): 2.4~2.6절. GUI 예제, 정적·동적 디스패치
  - [18.3 Implementing an Object-Oriented Design Pattern](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html): 2.11~2.13절. 블로그 글 상태 패턴 두 가지와 장단점. 절 중간에 연습 2·3과 같은 과제 제안이 있습니다.
- Rust By Example
  - [Traits > Returning Traits with `dyn`](https://doc.rust-lang.org/rust-by-example/trait/dyn.html): 함수가 트레이트를 그대로 반환할 수 없는 이유와 `Box<dyn Animal>`을 돌려주는 예. 2.8절의 `returns_summarizable`과 같은 내용입니다.
- Rustlings: 트레이트 객체만 다루는 주제는 없습니다. ch9에서 풀었던 [`13_error_handling/errors5.rs`](https://github.com/rust-lang/rustlings/blob/main/exercises/13_error_handling/errors5.rs)를 다시 열어, 주석이 `Box<dyn ???>`를 "???를 할 줄 아는 무엇이든"이라고 설명한 부분을 2.8절의 설명과 비교해 보면 좋습니다.
- The Reference
  - [Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html): `dyn Trait` 문법, 크기가 정해지지 않은 타입이라 포인터 뒤에 둔다는 점, vtable의 구성
  - [Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility): 트레이트가 dyn 호환이 되는 조건 전체 목록(2.9절)
- 에러 설명: `rustc --explain E0038`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
