// 대입할 때 일어나는 세 가지 일, 이동(move)·복사(copy)·복제(clone)를 비교합니다.
//
// 실행: cargo run -p p0-ch4-ownership --example 02_move_copy_clone

fn main() {
    println!("1. 정수는 대입하면 복사됩니다 (Copy 타입)");
    let x = 5;
    let y = x; // 4바이트짜리 값을 그대로 한 번 더 만듭니다. 비용이 거의 없습니다.
    println!("    x = {x}, y = {y}"); // 출력: x = 5, y = 5
    // 두 변수가 각자 자기 값을 가지므로 둘 다 쓸 수 있습니다.

    println!("2. String 값 자체는 (포인터, 길이, 용량) 세 칸짜리 작은 묶음입니다");
    // `size_of::<String>()`은 String 값이 스택에서 차지하는 바이트 수를 알려 줍니다.
    // 64비트 환경에서는 8바이트 x 3칸 = 24바이트입니다. 글자 내용은 이 안에 없고 힙에 있습니다.
    println!("    String의 크기: {}바이트", size_of::<String>()); // 출력: String의 크기: 24바이트

    println!("3. String은 대입하면 소유권이 이동합니다");
    let s1 = String::from("hello");
    // `as_ptr()`은 String이 가리키는 힙 데이터의 시작 주소입니다.
    // 주소 값 자체는 실행할 때마다 달라지므로, 출력하지 않고 비교에만 씁니다.
    let heap_of_s1 = s1.as_ptr();
    let s2 = s1; // 스택의 세 칸(24바이트)만 복사하고, s1은 무효가 됩니다.
    // println!("{s1}"); // ← 주석을 풀면 E0382: borrow of moved value: `s1`
    println!("    s2 = {s2}"); // 출력: s2 = hello
    println!(
        "    s2가 s1과 같은 힙 데이터를 가리키는가? {}",
        s2.as_ptr() == heap_of_s1
    ); // 출력: s2가 s1과 같은 힙 데이터를 가리키는가? true
    // 힙의 "hello"는 그대로 있고 주인만 s1에서 s2로 바뀌었습니다.
    // s1까지 유효하다면 스코프가 끝날 때 같은 힙 메모리를 두 번 해제(이중 해제)하게 됩니다.

    println!("4. clone()은 힙 데이터까지 통째로 복제합니다");
    let s3 = s2.clone(); // 새 힙 공간을 할당하고 글자를 복사합니다. 데이터가 크면 비쌉니다.
    println!("    s2 = {s2}, s3 = {s3}"); // 출력: s2 = hello, s3 = hello
    println!(
        "    s3가 s2와 같은 힙 데이터를 가리키는가? {}",
        s3.as_ptr() == s2.as_ptr()
    ); // 출력: s3가 s2와 같은 힙 데이터를 가리키는가? false
    // 각자 자기 힙 데이터를 소유하므로 둘 다 쓸 수 있고, 나중에 각자 해제합니다.

    println!("5. 값이 이동해 나간 변수도 새 값을 대입하면 다시 쓸 수 있습니다");
    let mut s4 = String::from("first");
    let s5 = s4; // s4는 무효가 되었지만,
    s4 = String::from("second"); // 새 값의 소유자가 되면 다시 유효합니다.
    println!("    s4 = {s4}, s5 = {s5}"); // 출력: s4 = second, s5 = first

    println!("6. Copy 타입으로만 이뤄진 튜플·배열은 통째로 복사됩니다");
    // `{:?}`는 튜플·배열을 디버그 형식으로 출력합니다(ch5에서 자세히 다룹니다).
    let t1 = (1, 2.5, 'a', true);
    let t2 = t1;
    println!("    t1 = {t1:?}, t2 = {t2:?}"); // 출력: t1 = (1, 2.5, 'a', true), t2 = (1, 2.5, 'a', true)
    let a1 = [1, 2, 3];
    let a2 = a1;
    println!("    a1 = {a1:?}, a2 = {a2:?}"); // 출력: a1 = [1, 2, 3], a2 = [1, 2, 3]

    println!("7. String이 하나라도 섞인 튜플은 이동합니다");
    let pair1 = (1, String::from("one"));
    let pair2 = pair1; // 튜플 전체가 이동합니다. pair1.0만 따로 쓰는 것도 불가능합니다.
    // println!("{}", pair1.0); // ← 주석을 풀면 E0382: borrow of moved value: `pair1`
    println!("    pair2 = {pair2:?}"); // 출력: pair2 = (1, "one")
}
