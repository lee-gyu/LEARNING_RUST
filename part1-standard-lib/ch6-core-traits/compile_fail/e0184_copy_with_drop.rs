// 기대하는 에러: error[E0184]: the trait `Copy` cannot be implemented for this type; the type has a destructor
// 핵심 메시지: `Copy` not allowed on types with destructors
//
// Drop을 구현했다는 것은 "버려질 때 할 정리 작업이 있다"는 뜻입니다. Copy 값은 대입할 때마다
// 몰래 복사본이 생기므로, 복사본마다 정리 작업(여기서는 표 반납)이 실행되어 한 번만 해야 할 일을 여러 번 하게 됩니다.
// 해결: Copy를 빼고 Clone만 남깁니다. 복제가 일어나는 곳이 .clone()으로 드러납니다.
//
// 확인: rustc --edition 2024 compile_fail/e0184_copy_with_drop.rs --out-dir <임시 디렉터리>

#[derive(Clone, Copy)]
struct Ticket {
    id: u32,
}

impl Drop for Ticket {
    fn drop(&mut self) {
        println!("{}번 표 반납", self.id);
    }
}

fn main() {
    let t = Ticket { id: 7 };
    let u = t;
    println!("{} {}", t.id, u.id);
}
