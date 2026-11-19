// 기대하는 에러: E0503
//   error[E0503]: cannot use `weight` because it was mutably borrowed
//
// 소유자가 할 수 있는 일(2): 가변으로 빌려준 동안에는 소유자도 값을 읽을 수 없습니다.
// 불변 참조가 살아 있을 때는 소유자가 읽어도 되지만, 가변 참조가 살아 있으면
// 그 참조를 거쳐 값이 언제 바뀔지 모르기 때문입니다.
// 같은 자리에서 println!처럼 값을 빌려 읽으면 E0503이 아니라 E0502가 납니다.
//
// 확인: rustc --edition 2024 compile_fail/e0503_use_while_mutably_borrowed.rs --out-dir /tmp/er-ch15

const LIMIT_GRAMS: u32 = 7000;

fn main() {
    let mut weight: u32 = 6800;
    let scale = &mut weight; // weight를 가변으로 빌림

    let over = weight > LIMIT_GRAMS; // E0503: 가변으로 빌려준 값을 소유자가 읽음

    *scale += 500; // 가변 참조를 그 뒤에 씀
    println!("초과 여부: {over}, 무게: {weight}g");
}
