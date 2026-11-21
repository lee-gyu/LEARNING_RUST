// 부록 B ch13 · Q13.5 — const 제네릭: C++의 값 템플릿 매개변수에 해당하는 것
//
// C++의 `template <typename T, std::size_t N> class Stack`처럼 타입 대신 값(크기 등)을 매개변수로 받는 일은
// Rust의 const 제네릭 `<T, const N: usize>`가 맡습니다. N은 컴파일 시간에 정해지는 값이라 배열 길이에 쓸 수 있고,
// 함수 본문에서는 보통의 값처럼 읽을 수 있습니다. N이 다르면 다른 타입이므로 크기가 다른 Stack끼리 섞이지 않습니다.
// 한계: 안정판에서는 N을 그대로만 쓸 수 있고 `N + 1` 같은 식을 타입 자리에 쓰지 못합니다
// (compile_fail/const_generic_expression.rs). C++ 템플릿은 그런 식을 쓸 수 있습니다.
//
// 실행: cargo run -p faq-ch13-headers-build-templates --example 02_const_generics

use std::mem::size_of;

/// 길이가 타입에 들어 있는 배열을 받습니다. 호출할 때 N을 적지 않아도 인자에서 추론합니다.
fn average<const N: usize>(xs: [f64; N]) -> f64 {
    xs.iter().sum::<f64>() / N as f64
}

/// 용량이 타입에 들어 있는 스택. N을 적지 않으면 기본값 4를 씁니다.
struct Stack<T, const N: usize = 4> {
    items: [Option<T>; N],
    len: usize,
}

impl<T, const N: usize> Stack<T, N> {
    fn new() -> Self {
        Stack {
            // [None; N]은 원소 타입이 Copy여야 해서(T: Copy가 아니면 E0277) 쓸 수 없습니다. from_fn은 인덱스마다
            // 클로저를 불러 원소를 하나씩 만듭니다(부록 A ch16 예제 03, compile_fail/e0277_array_repeat_non_copy.rs).
            items: std::array::from_fn(|_| None),
            len: 0,
        }
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            return Err(value); // 가득 차면 값을 돌려줍니다.
        }
        self.items[self.len] = Some(value);
        self.len += 1;
        Ok(())
    }

    fn capacity(&self) -> usize {
        N
    }
}

fn main() {
    println!("1. 배열 길이를 받는 함수");
    println!("  {}", average([1.0, 2.0, 3.0])); // 출력:   2
    println!("  {}", average([10.0; 4])); // 출력:   10

    println!("2. 용량이 타입에 들어 있는 스택");
    let mut small: Stack<&str, 2> = Stack::new();
    println!("  {:?}", small.push("a")); // 출력:   Ok(())
    println!("  {:?}", small.push("b")); // 출력:   Ok(())
    println!("  {:?}", small.push("c")); // 출력:   Err("c")
    let default_sized: Stack<u8> = Stack::new();
    println!("  기본 용량: {}", default_sized.capacity()); // 출력:   기본 용량: 4

    println!("3. N이 다르면 다른 타입이고 크기도 다름(이 환경 aarch64)");
    println!("  Stack<u32, 2>: {}바이트", size_of::<Stack<u32, 2>>()); // 출력:   Stack<u32, 2>: 24바이트
    println!("  Stack<u32, 8>: {}바이트", size_of::<Stack<u32, 8>>()); // 출력:   Stack<u32, 8>: 72바이트
}
