// Deref 트레이트로 직접 만든 MyBox<T>가 참조처럼 동작하게 만듭니다.
// - `*y`는 컴파일러가 `*(y.deref())`로 바꿔 실행합니다.
// - 역참조 강제 변환(deref coercion): 함수 인자의 타입이 맞지 않으면 컴파일러가 deref를
//   필요한 만큼 이어 불러 &MyBox<String> → &String → &str처럼 바꿔 줍니다.
// - DerefMut를 구현하면 `*`로 안의 값을 바꿀 수 있고, &mut 강제 변환도 일어납니다.
//
// 실행: cargo run -p p0-ch14-smart-pointers --example 03_deref_mybox

use std::ops::{Deref, DerefMut};

// Box<T>처럼 값 하나를 감싸는 튜플 구조체입니다.
// 진짜 Box와 달리 값을 힙이 아니라 MyBox 안(스택)에 둡니다. 여기서는 Deref만 보면 되므로 상관없습니다.
struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

// Deref를 구현하면 `*`가 MyBox에도 동작합니다. 이 구현을 지우면 E0614가 납니다
// (compile_fail/e0614_deref_without_trait.rs).
impl<T> Deref for MyBox<T> {
    // `*`로 따라갔을 때 나오는 타입입니다. 트레이트 안에서 쓰는 타입 이름을 정해 두는 문법입니다.
    type Target = T;

    // 값이 아니라 참조를 돌려줍니다. 값을 돌려주면 안의 T가 MyBox 밖으로 이동해 버리기 때문입니다.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// DerefMut는 Deref를 구현한 타입에만 구현할 수 있습니다. Target은 Deref의 것을 그대로 씁니다.
impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// 문자열 슬라이스를 받는 함수입니다. &MyBox<String>을 넘겨도 호출됩니다(4번 구역).
fn hello(name: &str) {
    println!("    Hello, {name}!");
}

// String을 바꾸는 함수입니다. &mut MyBox<String>을 넘기면 &mut String으로 바뀝니다(5번 구역).
fn shout(text: &mut String) {
    text.push('!');
}

// &mut MyBox<String>을 받아, &str을 받는 hello에 그대로 넘깁니다.
// 가변 참조는 불변 참조로 강제 변환될 수 있습니다(반대는 안 됩니다).
fn greet_then_shout(name: &mut MyBox<String>) {
    hello(name); // &mut MyBox<String> → &String → &str
    shout(name); // &mut MyBox<String> → &mut String
}

fn main() {
    println!("1. 참조는 `*`로 따라가 가리키는 값을 꺼냅니다");
    let x = 5;
    let y = &x;
    // x는 i32, y는 &i32로 타입이 다르므로 비교하려면 *y로 따라가야 합니다.
    println!("    x == *y ? {}", x == *y); // 출력: x == *y ? true

    println!("2. Box<T>도 참조처럼 `*`로 따라갈 수 있습니다");
    let b = Box::new(x);
    println!("    x == *b ? {}", x == *b); // 출력: x == *b ? true

    println!("3. Deref를 구현한 MyBox도 `*`로 따라갈 수 있습니다");
    let m = MyBox::new(x);
    // 컴파일러는 *m을 *(m.deref())로 바꿉니다. deref가 &i32를 돌려주고, 바깥 *가 그 참조를 따라갑니다.
    println!("    *m = {}", *m); // 출력: *m = 5
    println!("    *(m.deref()) = {}", *(m.deref())); // 출력: *(m.deref()) = 5

    println!("4. 역참조 강제 변환: &MyBox<String>을 &str 자리에 넘길 수 있습니다");
    let name = MyBox::new(String::from("Rust"));
    // &MyBox<String> --deref--> &String --deref--> &str. 몇 번 부를지는 컴파일할 때 정해집니다.
    hello(&name); // 출력: Hello, Rust!
    // 강제 변환이 없다면 이렇게 써야 합니다. (*name)이 String, [..]이 전체 슬라이스입니다.
    hello(&(*name)[..]); // 출력: Hello, Rust!
    // 메서드 호출도 같습니다. MyBox에는 len이 없으므로 deref를 따라가 String의 len이 불립니다.
    println!("    name.len() = {}", name.len()); // 출력: name.len() = 4
    // String과 Vec<T>도 Deref를 구현합니다(Target이 각각 str, [T]).
    // 그래서 타입을 적은 let에서도 &String → &str, &Vec<i32> → &[i32] 강제 변환이 일어납니다.
    let owned = String::from("문자열");
    let numbers = vec![10, 20, 30];
    let as_str: &str = &owned;
    let as_slice: &[i32] = &numbers;
    println!("    as_str = {as_str}, as_slice = {as_slice:?}"); // 출력: as_str = 문자열, as_slice = [10, 20, 30]

    println!("5. DerefMut: `*`로 안의 값을 바꾸고, &mut 강제 변환도 일어납니다");
    let mut counter = MyBox::new(0);
    *counter += 10; // *(counter.deref_mut()) += 10
    println!("    *counter = {}", *counter); // 출력: *counter = 10
    let mut word = MyBox::new(String::from("hey"));
    greet_then_shout(&mut word); // 출력: Hello, hey!
    println!("    *word = {}", *word); // 출력: *word = hey!
}
