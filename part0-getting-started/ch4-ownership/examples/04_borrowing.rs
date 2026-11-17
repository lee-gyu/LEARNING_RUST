// 참조(reference)로 값을 빌려주면(borrow) 소유권을 넘기지 않고도 함수가 값을 쓸 수 있습니다.
// 불변 참조 `&T`는 읽기만, 가변 참조 `&mut T`는 읽기와 쓰기를 허락합니다.
//
// 실행: cargo run -p p0-ch4-ownership --example 04_borrowing

// 03번 예제의 calculate_length를 참조로 바꾼 버전입니다.
// 이제 String을 돌려줄 필요가 없으므로 반환 타입이 usize 하나로 줄었습니다.
//
// Clippy는 `&String` 매개변수를 보면 `&str`을 쓰라고 경고합니다(clippy::ptr_arg).
// `&str`은 06번 예제에서 배우므로, 여기서는 그 경고가 날 것을 알고 일부러 `&String`을 씁니다.
#[expect(clippy::ptr_arg, reason = "&str은 06번 예제에서 배웁니다")]
fn calculate_length(s: &String) -> usize {
    s.len()
} // s는 참조일 뿐 소유자가 아닙니다. 그래서 여기서 String이 해제되지 않습니다.

// `&mut String`을 받으므로 빌려 온 String을 바꿀 수 있습니다.
fn change(some_string: &mut String) {
    // 메서드를 호출할 때(`.`)는 컴파일러가 참조를 알아서 따라가 줍니다.
    some_string.push_str(", world");
}

// 참조가 가리키는 정수 자체를 바꾸려면 `*`(역참조 연산자)로 참조를 따라가야 합니다.
fn add_one(n: &mut i32) {
    *n += 1;
}

fn main() {
    println!("1. 불변 참조(&)로 빌려주면 소유권은 그대로 남습니다");
    let s1 = String::from("hello");
    let len = calculate_length(&s1); // 호출하는 쪽에서도 `&`가 보이므로 "빌려준다"는 사실이 드러납니다.
    println!("    '{s1}'의 길이는 {len}입니다"); // 출력: 'hello'의 길이는 5입니다
    // s1을 넘기지 않고 빌려줬기 때문에 호출 뒤에도 s1을 그대로 쓸 수 있습니다.

    println!("2. 불변 참조는 동시에 여러 개 있어도 됩니다");
    let r1 = &s1;
    let r2 = &s1;
    println!("    r1 = {r1}, r2 = {r2}, s1 = {s1}"); // 출력: r1 = hello, r2 = hello, s1 = hello
    // 모두 읽기만 하므로, 누가 먼저 읽든 결과가 달라지지 않습니다.

    println!("3. 가변 참조(&mut)로 빌려주면 빌린 쪽이 값을 바꿀 수 있습니다");
    let mut s2 = String::from("hello"); // 변수 자체가 mut이어야 &mut로 빌려줄 수 있습니다.
    change(&mut s2);
    println!("    s2 = {s2}"); // 출력: s2 = hello, world

    println!("4. 가변 참조는 겹치지 않게 차례로 만들면 몇 번이든 쓸 수 있습니다");
    let mut count = 0;
    add_one(&mut count); // 이 가변 참조는 함수 호출이 끝나면 더 쓰이지 않습니다.
    add_one(&mut count); // 그래서 새 가변 참조를 또 만들 수 있습니다.
    println!("    count = {count}"); // 출력: count = 2

    println!("5. 참조를 담은 변수도 일반 값처럼 출력하고 메서드를 호출할 수 있습니다");
    let r3 = &s2;
    println!("    r3 = {r3}, 길이 = {}", r3.len()); // 출력: r3 = hello, world, 길이 = 12
    // 정수의 참조는 `*`로 값을 꺼내 계산합니다.
    let rc = &count;
    println!("    *rc + 10 = {}", *rc + 10); // 출력: *rc + 10 = 12
}
