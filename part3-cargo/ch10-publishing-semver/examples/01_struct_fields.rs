//! SemVer: 구조체에 필드를 더하는 변경이 언제 호환되고 언제 깨지는지 봅니다.
//!
//! 실행: cargo run -q -p p3-ch10-publishing-semver --example 01_struct_fields
//!
//! 가상의 라이브러리 `size`의 버전을 모듈로 흉내 냅니다(실제로는 크레이트의 버전).
//!   open_v1_0   : 1.0.0. 필드가 모두 공개입니다. 사용자는 구조체 리터럴로 만들고 구조 분해합니다.
//!   open_v1_1   : 1.1.0이라고 내고 싶지만, 비공개 필드를 더하면 그런 사용자 코드가 깨집니다
//!                 → compile_fail/new_private_field.rs. Cargo Book은 메이저 변경으로 분류합니다.
//!   closed_v1_0 : 1.0.0. 처음부터 비공개 필드를 하나 두고 생성자를 줍니다.
//!   closed_v1_1 : 1.1.0. 비공개 필드를 하나 더 더해도 사용자 코드는 그대로 컴파일됩니다(마이너).

mod open_v1_0 {
    pub struct Size {
        pub width: u32,
        pub height: u32,
    }
}

mod closed_v1_0 {
    pub struct Size {
        pub width: u32,
        pub height: u32,
        _private: (),
    }
    impl Size {
        pub fn new(width: u32, height: u32) -> Size {
            Size {
                width,
                height,
                _private: (),
            }
        }
    }
}

mod closed_v1_1 {
    pub struct Size {
        pub width: u32,
        pub height: u32,
        cached_area: u64, // 1.1.0에서 더한 비공개 필드
    }
    impl Size {
        pub fn new(width: u32, height: u32) -> Size {
            Size {
                width,
                height,
                cached_area: width as u64 * height as u64,
            }
        }
        pub fn area(&self) -> u64 {
            self.cached_area
        }
    }
}

fn main() {
    // 필드가 모두 공개인 판: 사용자는 리터럴로 만들고, `..` 없이 구조 분해할 수 있습니다.
    // 이 두 가지가 "필드를 더하면 깨지는" 사용 방식입니다.
    let s = open_v1_0::Size {
        width: 3,
        height: 4,
    };
    let open_v1_0::Size { width, height } = s;
    println!("open 1.0.0  : {width} x {height}");

    // 처음부터 비공개 필드가 있는 판: 사용자는 생성자로만 만들 수 있고,
    // 구조 분해에는 반드시 `..`가 필요합니다. 그래서 필드를 더해도 사용자 코드가 그대로입니다.
    let a = closed_v1_0::Size::new(3, 4);
    let closed_v1_0::Size { width, height, .. } = a;
    println!("closed 1.0.0: {width} x {height}");

    let b = closed_v1_1::Size::new(3, 4);
    let closed_v1_1::Size { width, height, .. } = &b;
    println!("closed 1.1.0: {width} x {height}, 넓이 {}", b.area());

    // 출력: open 1.0.0  : 3 x 4
    // 출력: closed 1.0.0: 3 x 4
    // 출력: closed 1.1.0: 3 x 4, 넓이 12
}
