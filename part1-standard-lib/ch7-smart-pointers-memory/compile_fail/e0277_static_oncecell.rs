// 기대하는 에러: error[E0277]: `OnceCell<String>` cannot be shared between threads safely
// 핵심 메시지: help: the trait `Sync` is not implemented for `OnceCell<String>`
//             note: if you want to do aliasing and mutation between multiple threads, use `std::sync::OnceLock` instead
//             note: shared static variables must have a type that implements `Sync`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_static_oncecell.rs --out-dir /tmp/rust-p1-ch7
//
// static 변수(Part 0 ch19)는 프로그램 전체에서 하나뿐이고, 어느 스레드에서든 접근할 수 있습니다.
// 그래서 static의 타입은 "여러 스레드가 &로 함께 써도 안전하다"는 표시인 Sync를 구현해야 합니다.
// Cell, RefCell, OnceCell, LazyCell은 단일 스레드 전용이라 Sync가 아닙니다. 두 스레드가 동시에 get_or_init을
// 부르면 둘 다 "비어 있다"고 보고 동시에 값을 쓰려 할 수 있기 때문입니다. static COUNTER: Cell<u32>도 같은 E0277입니다.
// 고치는 방법: note:가 말하는 std::sync::OnceLock<String>을 씁니다. 메서드(get_or_init, set, get)가 같고,
// 여러 스레드가 동시에 불러도 초기화가 한 번만 일어나도록 맞춰 줍니다. 처음 쓸 때 초기화하는 전역 값은 LazyLock입니다.
// 둘 다 ch11-threads-sync에서 다룹니다. 전역일 필요가 없다면 지역 변수나 구조체 필드에 OnceCell을 둡니다.

use std::cell::OnceCell;

static CONFIG: OnceCell<String> = OnceCell::new();

fn main() {
    let config = CONFIG.get_or_init(|| String::from("기본 설정"));
    println!("{config}");
}
