// 기대하는 에러: E0506
//   error[E0506]: cannot assign to `checklist` because it is borrowed
//
// 소유자가 할 수 있는 일(1): 빌려준 동안에는 소유자도 값을 덮어쓸 수 없습니다.
// `first`가 옛 목록의 원소를 가리키는데 목록을 새 Vec으로 바꾸면 옛 목록은 drop되므로,
// `first`는 해제된 메모리를 가리키게 됩니다.
//
// 확인: rustc --edition 2024 compile_fail/e0506_assign_while_borrowed.rs --out-dir /tmp/er-ch15

fn main() {
    let mut checklist = vec![String::from("여권"), String::from("충전기")];
    let first = &checklist[0]; // checklist를 빌림

    checklist = vec![String::from("우산")]; // E0506: 빌려준 값을 덮어씀

    println!("맨 위 항목: {first}"); // 빌린 것을 그 뒤에 씀
    println!("새 목록: {checklist:?}");
}
