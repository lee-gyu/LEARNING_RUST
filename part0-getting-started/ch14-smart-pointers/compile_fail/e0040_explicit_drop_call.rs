// 기대하는 에러: E0040 explicit use of destructor method
// 원인: Drop의 drop 메서드를 직접 불렀습니다. 그래도 Rust는 main이 끝날 때 c를 또 drop하므로
//       같은 값을 두 번 정리(이중 해제)하게 됩니다. 그래서 직접 호출 자체를 막습니다.
// 해결: std::mem::drop 함수에 값을 넘깁니다. drop(c);  (examples/04_drop.rs)
//
// 확인: rustc --edition 2024 compile_fail/e0040_explicit_drop_call.rs

struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("[drop] {}", self.data);
    }
}

fn main() {
    let c = CustomSmartPointer {
        data: String::from("some data"),
    };
    println!("CustomSmartPointer를 만들었습니다");
    c.drop();
    println!("main이 끝나기 전에 버렸습니다");
}
