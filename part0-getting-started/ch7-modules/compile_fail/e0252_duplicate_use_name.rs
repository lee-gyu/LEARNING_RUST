// 기대하는 에러: E0252 — the name `Course` is defined multiple times
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0252_duplicate_use_name.rs --out-dir /tmp/rust-ch7
//
// 서로 다른 모듈에 있는 같은 이름의 타입 두 개를 같은 스코프로 가져왔습니다.
// 한 스코프에는 같은 이름이 둘 있을 수 없으므로, main의 `Course`가 점심 메뉴인지 저녁 메뉴인지 정할 수 없습니다.
// (두 번째 use가 쓰이지 않았다는 unused_imports 경고도 함께 나옵니다.)

mod menu {
    pub mod lunch {
        pub enum Course {
            Special, // 점심 특선
        }
    }

    pub mod dinner {
        pub enum Course {
            Special, // 저녁 특선
        }
    }
}

use crate::menu::dinner::Course;
use crate::menu::lunch::Course;

fn main() {
    let _order = Course::Special;
}
