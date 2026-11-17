// Rust의 모든 항목(함수, 구조체, 열거형, 상수, 모듈)은 기본적으로 비공개(private)입니다.
// `pub`을 붙인 항목만 부모 모듈 바깥에서 쓸 수 있고, 반대로 자식 모듈은 조상의 비공개 항목을 볼 수 있습니다.
//
// 실행: cargo run -p p0-ch7-modules --example 03_privacy
//
// 이 파일의 모듈 트리 (pub이 붙은 것만 표시):
//
// crate
//  ├── kitchen                       ← pub 아님. 하지만 main과 같은 곳(크레이트 루트)에 있어서 main은 쓸 수 있음
//  │   ├── SAUCE_PER_SERVING         ← 비공개 상수
//  │   ├── pub make_pasta
//  │   ├── prepare_sauce             ← 비공개 함수
//  │   └── pub staff                 ← pub 모듈
//  │       ├── pub check_recipe
//  │       └── private_memo          ← pub 모듈 안이라도 pub이 없으면 비공개
//  └── main

mod kitchen {
    // 비공개 상수: 소스 양은 주방의 내부 사정이므로 바깥에 알리지 않습니다.
    const SAUCE_PER_SERVING: u32 = 120;

    // 바깥에 공개하는 기능은 "파스타를 만든다" 하나뿐입니다.
    pub fn make_pasta(servings: u32) {
        // 같은 모듈 안에서는 비공개 항목을 자유롭게 씁니다.
        let sauce = prepare_sauce(servings);
        println!("  파스타 {servings}인분 완성 (소스 {sauce}ml 사용)");
    }

    // 비공개 함수: 바깥에서는 이 함수가 있는지조차 알 필요가 없습니다.
    // 나중에 소스 만드는 방법을 바꿔도 make_pasta만 그대로 두면 바깥 코드는 영향을 받지 않습니다.
    fn prepare_sauce(servings: u32) -> u32 {
        servings * SAUCE_PER_SERVING
    }

    // 모듈에 pub을 붙이면 "이 모듈의 이름"을 바깥에서 부를 수 있게 될 뿐입니다.
    // 모듈 안의 항목은 하나하나 따로 pub을 붙여야 공개됩니다.
    pub mod staff {
        pub fn check_recipe() {
            // 자식 모듈은 조상 모듈의 비공개 항목을 쓸 수 있습니다.
            // 주방 직원(staff)은 주방(kitchen)의 내부 사정을 알아도 되기 때문입니다.
            println!("  1인분 소스 양: {}ml", super::SAUCE_PER_SERVING);
            println!("  2인분 소스 양: {}ml", super::prepare_sauce(2));
            private_memo();
        }

        // pub mod 안에 있어도 pub이 없으면 staff 바깥에서는 부를 수 없습니다.
        fn private_memo() {
            println!("  (메모: 이 레시피는 주방 밖으로 알려지지 않습니다)");
        }
    }
}

fn main() {
    println!("1. 공개 함수 호출");
    // kitchen에는 pub이 없지만 main과 kitchen은 둘 다 크레이트 루트에 정의된 형제라서 쓸 수 있습니다.
    // make_pasta에는 pub이 있으므로 kitchen 바깥인 main에서 부를 수 있습니다.
    kitchen::make_pasta(3); // 출력:   파스타 3인분 완성 (소스 360ml 사용)

    println!("2. 자식 모듈은 조상의 비공개 항목을 볼 수 있습니다");
    kitchen::staff::check_recipe();
    // 출력:   1인분 소스 양: 120ml
    // 출력:   2인분 소스 양: 240ml
    // 출력:   (메모: 이 레시피는 주방 밖으로 알려지지 않습니다)

    // 아래 줄들의 주석을 풀면 컴파일 에러(E0603)가 납니다. 부모(여기서는 크레이트 루트)는
    // 자식 모듈의 비공개 항목을 볼 수 없습니다. compile_fail/e0603_private_function.rs 참고.
    // kitchen::prepare_sauce(1);
    // kitchen::staff::private_memo();
    // println!("{}", kitchen::SAUCE_PER_SERVING);
}
