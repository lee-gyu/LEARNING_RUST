// 이 예제가 보여 주는 것: 클로저 문법(함수와 나란히 비교), 환경 캡처, 타입 추론과 "한 번 정해진 타입은 고정",
//                        클로저는 정의할 때가 아니라 호출할 때 실행된다는 것
// 실행: cargo run -p p0-ch13-closures-iterators --example 01_closure_syntax

// 비교용 함수입니다. 함수는 매개변수와 반환값의 타입을 반드시 적어야 합니다.
// 함수의 시그니처는 다른 코드와 맺는 약속이라서, 본문을 보지 않고도 쓰는 법을 알 수 있어야 하기 때문입니다.
fn add_one_v1(x: u32) -> u32 {
    x + 1
}

fn main() {
    // 1. 같은 일을 하는 함수와 클로저
    println!("== 1. 함수와 클로저 ==");

    // 매개변수를 ( ) 대신 | |로 감쌉니다. 타입을 모두 적으면 함수와 거의 같은 모양입니다.
    let add_one_v2 = |x: u32| -> u32 { x + 1 };
    // 타입 표기와 중괄호를 모두 생략한 모양입니다. 본문이 표현식 하나면 중괄호가 필요 없습니다.
    // 타입은 아래에서 add_one_v4(5)로 부르는 것을 보고 컴파일러가 추론합니다.
    let add_one_v4 = |x| x + 1;
    // 본문이 여러 줄이면 중괄호 블록을 씁니다. 함수처럼 마지막 표현식이 반환값입니다.
    let add_one_verbose = |x: u32| {
        let result = x + 1;
        println!("  (add_one_verbose 안에서 {x} + 1 = {result})");
        result
    };

    println!("add_one_v1(5) = {}", add_one_v1(5)); // 출력: add_one_v1(5) = 6
    println!("add_one_v2(5) = {}", add_one_v2(5)); // 출력: add_one_v2(5) = 6
    println!("add_one_v4(5) = {}", add_one_v4(5)); // 출력: add_one_v4(5) = 6
    println!("add_one_verbose(5) = {}", add_one_verbose(5));
    // 출력:   (add_one_verbose 안에서 5 + 1 = 6)
    // 출력: add_one_verbose(5) = 6
    // 인자를 먼저 계산하므로 클로저 안의 println!이 바깥 println!보다 먼저 찍힙니다.

    // 2. 클로저는 주변 변수(환경)를 캡처합니다
    println!("\n== 2. 환경 캡처 ==");
    let base = 10;
    // 이 클로저는 매개변수가 아닌 base를 본문에서 씁니다. 클로저가 정의된 스코프의 변수를
    // 붙잡아(capture) 두었다가 나중에 호출될 때 씁니다.
    let add_base = |x: i32| x + base;
    println!("add_base(1) = {}", add_base(1)); // 출력: add_base(1) = 11
    // 함수는 이렇게 할 수 없습니다. main 안에 fn add_base(x: i32) -> i32 { x + base }를 만들면
    // error[E0434]: can't capture dynamic environment in a fn item 으로 거부됩니다.

    // 3. 클로저는 호출할 때 실행됩니다
    println!("\n== 3. 정의와 호출 ==");
    let say_hello = || println!("  클로저 본문이 실행되었습니다"); // 매개변수가 없으면 ||
    println!("클로저를 정의했습니다(아직 본문은 실행되지 않음)");
    say_hello();
    say_hello();
    // 출력: 클로저를 정의했습니다(아직 본문은 실행되지 않음)
    // 출력:   클로저 본문이 실행되었습니다
    // 출력:   클로저 본문이 실행되었습니다

    // 4. 타입은 처음 쓰인 대로 하나로 정해집니다
    println!("\n== 4. 한 번 정해진 타입은 고정 ==");
    let identity = |x| x;
    let s = identity(String::from("hello")); // 이 호출로 x의 타입과 반환 타입이 String으로 정해집니다.
    println!("identity(String) = {s}"); // 출력: identity(String) = hello
    // 이어서 identity(5)를 부르면 error[E0308]: mismatched types (expected `String`, found integer)입니다.
    // 클로저는 제네릭 함수가 아니라 타입이 하나로 정해진 함수입니다(compile_fail/e0308_closure_type_fixed.rs).
    // 정수에도 쓰고 싶다면 클로저를 하나 더 만듭니다.
    let identity_num = |x| x;
    println!("identity_num(5) = {}", identity_num(5)); // 출력: identity_num(5) = 5
}
