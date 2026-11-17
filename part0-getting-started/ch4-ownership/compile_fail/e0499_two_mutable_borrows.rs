// 기대하는 에러: E0499 — cannot borrow `s` as mutable more than once at a time
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0499_two_mutable_borrows.rs
//
// r1과 r2의 사용 구간이 println!까지 겹치므로, 같은 값에 대한 가변 참조가 동시에 두 개 존재합니다.
// 한 값을 동시에 두 곳에서 바꿀 수 있으면 데이터 경쟁이 생길 수 있어 금지됩니다.

fn main() {
    let mut s = String::from("hello");

    let r1 = &mut s;
    let r2 = &mut s;

    println!("{r1}, {r2}");
}
