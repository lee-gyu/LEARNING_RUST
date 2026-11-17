// 함수에 인자를 넘기거나 반환값을 받을 때도 대입과 똑같이 소유권이 이동합니다.
//
// 실행: cargo run -p p0-ch4-ownership --example 03_ownership_and_functions

// 01번 예제와 같은 관찰용 타입입니다. 버려질 때 `[drop] 이름`을 출력합니다.
// 구조체(struct)는 ch5에서, `Drop` 트레이트는 ch14에서 배웁니다.
struct Noisy {
    name: String,
}

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.name);
    }
}

// 매개변수 some_string이 인자로 들어온 String의 새 소유자가 됩니다.
fn takes_ownership(some_string: String) {
    println!("    takes_ownership: {some_string}");
} // ← some_string의 스코프가 끝나므로 힙 메모리가 여기서 해제됩니다.

// i32는 Copy 타입이므로 인자로 넘길 때 값이 복사됩니다.
fn makes_copy(some_integer: i32) {
    println!("    makes_copy: {some_integer}");
}

// Noisy 값의 소유권을 받아서, 함수가 끝날 때 버립니다.
fn consume(value: Noisy) {
    println!("    consume: {} 받음", value.name);
} // ← value가 여기서 drop됩니다. main으로 돌아가기 전입니다.

// 함수 안에서 만든 String을 반환하면 소유권이 호출한 쪽으로 넘어갑니다.
fn gives_ownership() -> String {
    String::from("yours")
}

// 받은 소유권을 그대로 돌려줍니다.
fn takes_and_gives_back(a_string: String) -> String {
    a_string
}

// 길이만 알고 싶어도 String을 넘기면 소유권이 넘어가 버립니다.
// 그래서 원래 값을 튜플에 담아 다시 돌려줘야 합니다.
fn calculate_length(s: String) -> (String, usize) {
    let length = s.len();
    (s, length)
}

fn main() {
    println!("1. String을 인자로 넘기면 소유권이 함수로 이동합니다");
    let s = String::from("hello");
    takes_ownership(s);
    // println!("{s}"); // ← 주석을 풀면 E0382: borrow of moved value: `s`
    // 출력:
    //     takes_ownership: hello

    println!("2. Copy 타입은 인자로 넘겨도 원래 변수를 계속 쓸 수 있습니다");
    let x = 5;
    makes_copy(x);
    println!("    main: x는 여전히 {x}");
    // 출력:
    //     makes_copy: 5
    //     main: x는 여전히 5

    println!("3. 함수로 이동한 값은 그 함수가 끝날 때 drop됩니다");
    let n = Noisy {
        name: String::from("n"),
    };
    consume(n);
    println!("    main: consume 호출이 끝남");
    // 출력:
    //     consume: n 받음
    //     [drop] n
    //     main: consume 호출이 끝남
    // [drop] n이 main의 다음 줄보다 먼저 찍힙니다. 새 소유자인 consume이 끝났기 때문입니다.

    println!("4. 반환값으로 소유권을 넘겨받을 수 있습니다");
    let s1 = gives_ownership();
    let s2 = String::from("hello");
    let s3 = takes_and_gives_back(s2); // s2 → 함수 → s3 순서로 소유권이 옮겨 갑니다.
    println!("    s1 = {s1}, s3 = {s3}");
    // 출력:
    //     s1 = yours, s3 = hello

    println!("5. 소유권을 넘겼다가 돌려받는 방식은 번거롭습니다");
    let s4 = String::from("hello");
    let (s4, len) = calculate_length(s4); // 돌려받은 String을 같은 이름으로 섀도잉합니다.
    println!("    '{s4}'의 길이는 {len}입니다");
    // 출력:
    //     'hello'의 길이는 5입니다
    // 값을 잠깐 쓰기만 할 때는 소유권을 넘기지 않는 방법, 즉 참조가 필요합니다(04번 예제).
}
