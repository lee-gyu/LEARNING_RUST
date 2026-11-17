// 기대하는 에러: E0502 — cannot borrow `s` as mutable because it is also borrowed as immutable
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0502_mutable_while_immutable.rs
//
// r1, r2(불변 참조)가 println!에서 다시 쓰이므로, r3(가변 참조)를 만드는 시점에 불변 대여가 아직 살아 있습니다.
// 읽는 쪽은 값이 그대로라고 믿고 있는데 다른 쪽이 그 값을 바꾸면 안 되기 때문입니다.

fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    let r3 = &mut s;

    println!("{r1}, {r2}, and {r3}");
}
