// 기대하는 에러: generic parameters may not be used in const operations (에러 코드 없음)
// 핵심 메시지: cannot perform const operation using `N`
//              help: const parameters may only be used as standalone arguments here, i.e. `N`
//
// 배열 끝에 원소 하나를 붙여 길이가 N + 1인 배열을 돌려주려 했습니다. C++에서는
// `template <std::size_t N> std::array<int, N + 1> push_one(std::array<int, N>, int)`처럼 쓸 수 있지만,
// Rust 안정판의 const 제네릭은 타입 자리에서 N을 그대로만 쓸 수 있고 N으로 계산한 식은 쓰지 못합니다.
// 이 제한을 푸는 기능(generic_const_exprs)은 이 툴체인에서 nightly 전용이고, 안정판에서
// #![feature]를 쓰면 E0554입니다.
// 고치는 방법: 결과 길이를 두 번째 const 매개변수 M으로 부르는 쪽이 정하게 하고 본문에서 M == N + 1을 확인하거나,
// 길이가 실행 중에 정해지는 Vec<i32>를 돌려줍니다. 확인을 인라인 const 블록(const { assert!(…) })으로 쓰면
// 컴파일 시간에 잡지만, 그 에러(E0080)는 단형성화할 때 나므로 cargo build에서는 나고 cargo check로는 드러나지 않습니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch13-headers-build-templates compile_fail/const_generic_expression.rs

fn push_one<const N: usize>(xs: [i32; N], x: i32) -> [i32; N + 1] {
    std::array::from_fn(|i| if i < N { xs[i] } else { x })
}

fn main() {
    println!("{:?}", push_one([1, 2], 3));
}
