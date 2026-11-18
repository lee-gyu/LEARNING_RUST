// 기대하는 에러: error[E0599]: the method `get` exists for struct `Cell<String>`, but its trait bounds were not satisfied
// 핵심 메시지: note: the following trait bounds were not satisfied: `String: Copy`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0599_cell_get_non_copy.rs --out-dir /tmp/rust-p1-ch7
//
// Cell::get은 impl<T: Copy> Cell<T> 블록에 있습니다. 안의 값을 비트 그대로 복사해 돌려주기 때문입니다.
// String을 그렇게 복사하면 같은 힙 버퍼를 가리키는 String이 둘이 되어 이중 해제가 됩니다.
// 그렇다고 &String을 돌려줄 수도 없습니다. Cell은 안을 가리키는 참조를 내주지 않는 대신 대여 검사를 하지 않는 타입이기 때문입니다.
// 고치는 방법(무엇을 하려는지에 따라):
//   - 값을 잠깐 꺼내 쓰고 되돌려 놓기: let name = profile.nickname.take(); ... profile.nickname.set(name);
//   - 새 값으로 바꾸며 예전 값 받기: let old = profile.nickname.replace(String::from("새 이름"));
//   - 읽기 위한 참조가 필요하면 RefCell<String>으로 바꾸고 borrow()를 씁니다(examples/07_refcell_more.rs).
//   - 값이 한 번 정해지고 바뀌지 않는다면 OnceCell<String>의 get()이 &String을 줍니다(examples/08_once_lazy.rs).

use std::cell::Cell;

struct Profile {
    nickname: Cell<String>,
}

fn main() {
    let profile = Profile {
        nickname: Cell::new(String::from("러스티")),
    };
    let name = profile.nickname.get();
    println!("{name}");
}
