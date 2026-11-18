// 기대하는 에러: error[E0308]: mismatched types
// 핵심 메시지: expected `Option<i32>`, found `Option<Option<_>>`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0308_map_instead_of_and_then.rs --out-dir /tmp/rust-p1-ch2
//
// map의 클로저 바운드는 FnOnce(T) -> U이고, map은 클로저의 결과 U를 Some으로 쌉니다.
// 이 클로저는 parse().ok()로 이미 Option을 돌려주므로 U = Option<i32>가 되고, 결과는 Option<Option<i32>>입니다.
// 함수의 반환 타입 Option<i32>와 맞지 않습니다.
// 고치는 방법: 클로저가 Option을 돌려준다면 map 대신 and_then을 씁니다(바운드: FnOnce(T) -> Option<U>).
//   text.lines().next().and_then(|line| line.trim().parse().ok())
// 컴파일러는 끝에 ?를 붙이라고 제안합니다. 이 함수는 Option을 돌려주므로 그렇게 해도 컴파일되고 결과도 같지만,
// "바깥 Option이 None이면 함수에서 빠져나가고, 안쪽 Option을 그대로 돌려준다"는 우회로입니다.
// 단계를 잇는다는 뜻은 and_then이 더 직접 드러냅니다.

fn first_number(text: &str) -> Option<i32> {
    text.lines().next().map(|line| line.trim().parse().ok())
}

fn main() {
    println!("{:?}", first_number("42\n7"));
}
