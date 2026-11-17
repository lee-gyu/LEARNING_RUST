// 대여 규칙: "불변 참조 여러 개" 또는 "가변 참조 하나" 중 하나만 동시에 존재할 수 있습니다.
// 여기서 "동시에"를 판단하는 기준은 변수의 스코프가 아니라 참조가 마지막으로 사용되는 지점입니다.
// 아래 코드는 모두 컴파일되고 실행됩니다.
//
// 실행: cargo run -p p0-ch4-ownership --example 05_borrow_scope

fn main() {
    let mut s = String::from("hello");

    println!("1. 불변 참조의 마지막 사용 뒤에는 가변 참조를 만들 수 있습니다");
    let r1 = &s;
    let r2 = &s;
    println!("    r1 = {r1}, r2 = {r2}"); // ← r1, r2가 마지막으로 사용되는 곳. 불변 대여가 여기서 끝납니다.
    // 변수 r1, r2는 main이 끝날 때까지 "이름"으로는 남아 있지만, 대여는 이미 끝났습니다.
    let r3 = &mut s; // 그래서 가변 참조를 만들어도 겹치는 대여가 없습니다.
    r3.push_str(", world");
    println!("    r3 = {r3}"); // ← r3의 마지막 사용
    // 출력:
    //     r1 = hello, r2 = hello
    //     r3 = hello, world
    // 위 r3 줄 아래에서 r1을 한 번 더 출력하면 불변 대여가 r3와 겹치므로 E0502가 납니다.

    println!("2. 가변 참조가 끝나면 원래 변수로 다시 읽고 쓸 수 있습니다");
    println!("    s = {s}");
    s.push('!');
    println!("    s = {s}");
    // 출력:
    //     s = hello, world
    //     s = hello, world!

    println!("3. 가변 참조도 사용 구간이 겹치지 않으면 차례로 여러 번 만들 수 있습니다");
    let a = &mut s;
    a.push('?'); // ← a의 마지막 사용
    let b = &mut s;
    b.push('?'); // ← b의 마지막 사용
    println!("    s = {s}");
    // 출력:
    //     s = hello, world!??

    println!("4. 중괄호로 스코프를 나눠 대여 구간을 눈에 보이게 할 수도 있습니다");
    {
        let inner = &mut s;
        inner.clear();
        inner.push_str("new");
    } // inner의 스코프가 끝났으므로 이 아래에서 대여가 겹칠 수 없습니다.
    let r4 = &s;
    println!("    r4 = {r4}");
    // 출력:
    //     r4 = new

    println!("5. 참조를 오래 들고 있을 필요가 없으면, 필요한 값만 복사해 둡니다");
    let len_before = s.len(); // usize(Copy 타입)를 받아 두므로 s를 계속 빌리고 있지 않습니다.
    s.push_str(" value");
    println!("    길이: {len_before} → {}", s.len());
    // 출력:
    //     길이: 3 → 9
}
