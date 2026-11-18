// 기대하는 에러: error[E0515]: cannot return value referencing temporary value
// 핵심 메시지: temporary value created here / returns a value referencing data owned by the current function
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0515_return_ref_from_refcell.rs --out-dir /tmp/rust-p1-ch7
//
// self.songs.borrow()는 Ref<Vec<String>>이라는 임시값을 만들고, [0]과 &는 그 Ref를 거쳐 안의 String을 가리킵니다.
// RefCell은 이 Ref가 살아 있는 동안만 "누가 읽는 중"이라고 셉니다. 함수가 끝나면 Ref가 버려져 대여가 끝나므로,
// 그 뒤에도 남는 &str을 돌려주면 RefCell이 모르는 사이에 누군가 borrow_mut()으로 Vec을 바꿀 수 있게 됩니다.
// 그래서 컴파일러는 임시값(Ref)을 가리키는 참조를 밖으로 내보내지 못하게 합니다.
// 고치는 방법:
//   - 대여를 돌려주는 쪽이 들고 있게 합니다: fn first(&self) -> Ref<'_, str> {
//         Ref::map(self.songs.borrow(), |v| v[0].as_str())
//     }  (std::cell::Ref를 가져와야 합니다. examples/07_refcell_more.rs)
//   - 복제해서 소유한 값을 돌려줍니다: fn first(&self) -> String { self.songs.borrow()[0].clone() }
//   - 한 번 정하고 바꾸지 않는 값이라면 OnceCell을 써서 &str을 그대로 돌려줄 수 있습니다(examples/08_once_lazy.rs).

use std::cell::RefCell;

struct Playlist {
    songs: RefCell<Vec<String>>,
}

impl Playlist {
    fn first(&self) -> &str {
        &self.songs.borrow()[0]
    }
}

fn main() {
    let list = Playlist {
        songs: RefCell::new(vec![String::from("봄날")]),
    };
    println!("{}", list.first());
}
