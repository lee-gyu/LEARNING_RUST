// 컴파일 에러 예제: 필드가 모두 공개였던 구조체에 비공개 필드를 더한 새 버전(메이저 변경)
//
// 기대하는 에러: 에러 코드 없음 (두 에러 모두 E 코드가 붙지 않습니다)
// 핵심 메시지:   error: cannot construct `Size` with struct literal syntax due to private fields
//                error: pattern requires `..` due to inaccessible fields
//
// 확인 (챕터 디렉터리에서, 산출물은 임시 디렉터리로):
//   rustc --edition 2024 --out-dir ../../target/tmp/p3-ch10 compile_fail/new_private_field.rs
//
// 상황: 모듈 `size`가 라이브러리의 "1.1.0"이라고 생각합니다. 1.0.0에서는 width·height 둘 다
// 공개였고, 사용자 코드(main)는 1.0.0에 맞춰 리터럴로 만들고 `..` 없이 구조 분해했습니다.

mod size {
    pub struct Size {
        pub width: u32,
        pub height: u32,
        cached_area: u64, // 1.1.0에서 더한 비공개 필드
    }
    impl Size {
        pub fn new(width: u32, height: u32) -> Size {
            Size { width, height, cached_area: width as u64 * height as u64 }
        }
        pub fn area(&self) -> u64 {
            self.cached_area
        }
    }
}

fn main() {
    let s = size::Size { width: 3, height: 4 }; // 비공개 필드는 모듈(실제로는 라이브러리 크레이트) 밖에서 채울 수 없음
    let size::Size { width, height } = s; // 모듈 밖에서는 `..` 없이 모든 필드를 적을 수 없음
    println!("{width} x {height}");
}
