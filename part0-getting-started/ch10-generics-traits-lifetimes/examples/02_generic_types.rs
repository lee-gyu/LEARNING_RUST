// 이 예제가 보여 주는 것: 제네릭 구조체·열거형·메서드, 특정 타입에만 구현하는 impl, 단형성화(monomorphization)
// 실행: cargo run -p p0-ch10-generics-traits-lifetimes --example 02_generic_types

// 타입 매개변수 T 하나로 두 필드의 타입을 정합니다. 그래서 x와 y는 반드시 같은 타입이어야 합니다.
#[derive(Debug)]
struct Point<T> {
    x: T,
    y: T,
}

// 필드마다 다른 타입을 허용하려면 타입 매개변수를 두 개 둡니다. HashMap<K, V>가 이런 모양입니다.
#[derive(Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V,
}

// 열거형도 제네릭으로 만들 수 있습니다. 표준 라이브러리의 Option<T>도 이렇게 정의되어 있습니다.
#[derive(Debug)]
enum Slot<T> {
    Filled(T),
    Empty,
}

// impl 바로 뒤의 <T>는 "이 블록의 T는 타입 매개변수다"라는 선언입니다.
// 이것이 없으면 컴파일러는 Point<T>의 T를 `T`라는 이름의 구체 타입으로 찾으려 합니다.
// 이 블록의 메서드는 T가 무엇이든 모든 Point<T>에 생깁니다.
impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }

    fn y(&self) -> &T {
        &self.y
    }
}

// impl 뒤에 <T>가 없고 Point<f32>라고 구체 타입을 적었습니다.
// 이 메서드는 Point<f32>에만 생기고, Point<i32> 같은 다른 Point에는 없습니다.
// powi(제곱)와 sqrt(제곱근)는 실수 타입에만 있는 메서드라서, 아무 T에나 구현할 수 없습니다.
impl Point<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

impl<K, V> KeyValue<K, V> {
    // 메서드는 구조체와 별개로 자기만의 타입 매개변수를 가질 수 있습니다.
    // K, V는 구조체의 것이라 impl 뒤에, V2는 이 메서드만 쓰므로 메서드 이름 뒤에 선언합니다.
    // 결과는 값의 타입이 V2로 바뀐 "다른 타입"의 KeyValue입니다.
    fn with_value<V2>(self, value: V2) -> KeyValue<K, V2> {
        KeyValue {
            key: self.key,
            value,
        }
    }
}

impl<T> Slot<T> {
    // Option의 unwrap_or와 같은 일을 하는 메서드입니다. 값이 있으면 그 값을, 없으면 기본값을 돌려줍니다.
    fn get_or(self, default: T) -> T {
        match self {
            Slot::Filled(value) => value,
            Slot::Empty => default,
        }
    }
}

fn main() {
    println!("1. 제네릭 구조체는 담는 타입만 바꿔 여러 번 쓸 수 있습니다");
    let integer = Point { x: 5, y: 10 }; // Point<i32>
    let float = Point { x: 1.0, y: 4.0 }; // Point<f64>
    println!("    integer = {integer:?}, float = {float:?}");
    println!(
        "    integer.x() = {}, float.y() = {}",
        integer.x(),
        float.y()
    );
    // 출력:
    //     integer = Point { x: 5, y: 10 }, float = Point { x: 1.0, y: 4.0 }
    //     integer.x() = 5, float.y() = 4
    // 아래 줄의 주석을 풀면 E0308(expected integer, found floating-point number)이 납니다.
    // x가 5라서 T가 정수로 정해졌는데, y에 실수를 넣었기 때문입니다.
    // let mixed = Point { x: 5, y: 4.0 };

    println!("2. 타입 매개변수가 두 개면 필드마다 다른 타입을 담을 수 있습니다");
    let entry = KeyValue {
        key: "점수",
        value: 95,
    }; // KeyValue<&str, i32>
    println!("    {} = {}", entry.key, entry.value);
    // 출력: 점수 = 95

    println!("3. Point<f32>에만 있는 메서드");
    let p = Point { x: 3.0_f32, y: 4.0 }; // 3.0_f32처럼 접미사를 붙여 T를 f32로 정합니다.
    println!("    원점에서의 거리: {}", p.distance_from_origin());
    // 출력: 원점에서의 거리: 5
    // 아래 줄의 주석을 풀면 E0599(no method named `distance_from_origin` found for struct `Point<{integer}>`)가
    // 납니다. {integer}는 "아직 어떤 정수 타입인지 정해지지 않은 정수"라는 뜻이고, 메시지의
    // note: the method was found for `Point<f32>`가 이 메서드는 Point<f32>에만 있다고 알려 줍니다.
    // println!("{}", integer.distance_from_origin());

    println!("4. 메서드가 자기만의 타입 매개변수를 가질 수 있습니다");
    let converted = entry.with_value("A등급"); // KeyValue<&str, i32> → KeyValue<&str, &str>
    println!("    {converted:?}");
    // 출력: KeyValue { key: "점수", value: "A등급" }

    println!("5. 제네릭 열거형: Option<T>와 같은 모양을 직접 만들 수 있습니다");
    let filled: Slot<i32> = Slot::Filled(7);
    let empty: Slot<i32> = Slot::Empty; // 값이 없으면 T를 추론할 수 없으므로 타입을 적습니다.
    println!("    {filled:?}, {empty:?}");
    println!("    get_or(0): {}, {}", filled.get_or(0), empty.get_or(0));
    // ch8의 get(i)가 돌려주던 Option<&i32>도 같은 모양의 제네릭 열거형입니다(T = &i32).
    let scores = [90, 85];
    println!(
        "    표준 Option의 unwrap_or: {}, {}",
        scores.get(1).unwrap_or(&0),
        scores.get(5).unwrap_or(&0)
    );
    // 출력:
    //     Filled(7), Empty
    //     get_or(0): 7, 0
    //     표준 Option의 unwrap_or: 85, 0

    println!("6. 단형성화: 쓰인 타입마다 별도의 구체 타입이 만들어집니다");
    // 컴파일러는 Point<T>를 Point<u8>, Point<f64>처럼 실제로 쓰인 타입마다 따로 만들어 냅니다.
    // 그래서 같은 정의에서 나왔어도 크기가 다릅니다. 실행 중에 "T가 무엇인지" 확인하는 비용은 없습니다.
    // size_of::<타입>()의 ::<> 는 함수에 타입 인자를 직접 적는 문법입니다(ch5에서도 썼습니다).
    println!("    Point<u8>:  {}바이트", size_of::<Point<u8>>());
    println!("    Point<f64>: {}바이트", size_of::<Point<f64>>());
    println!("    Slot<u8>:   {}바이트", size_of::<Slot<u8>>());
    println!("    Slot<u64>:  {}바이트", size_of::<Slot<u64>>());
    // 출력:
    //     Point<u8>:  2바이트
    //     Point<f64>: 16바이트
    //     Slot<u8>:   2바이트
    //     Slot<u64>:  16바이트
}
