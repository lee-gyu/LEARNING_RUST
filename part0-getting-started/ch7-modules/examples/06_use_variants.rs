// `use`의 여러 형태: 이름 충돌을 피하는 두 방법(부모 모듈, `as` 별칭), 중첩 경로, 글롭(`*`).
// 형태마다 함수를 하나씩 두고 그 안에 use를 썼습니다. 함수 안의 use는 그 함수에서만 유효하므로
// 함수끼리 같은 이름을 가져와도 서로 부딪히지 않습니다.
//
// 실행: cargo run -p p0-ch7-modules --example 06_use_variants

mod menu {
    // lunch와 dinner에 이름이 같은 열거형 Course가 있습니다.
    // 모듈이 달라서 정의할 때는 문제가 없지만, 둘 다 `Course`라는 이름으로 가져오면 충돌합니다.
    pub mod lunch {
        #[derive(Debug)]
        pub enum Course {
            Bibimbap,
            Kimbap,
        }
    }

    pub mod dinner {
        #[derive(Debug)]
        pub enum Course {
            Steak,
            Pasta,
        }
    }

    pub mod drinks {
        pub fn coffee() {
            println!("  커피를 내립니다 (원두: {})", secret_blend());
        }

        pub fn tea() {
            println!("  차를 우립니다");
        }

        // 비공개 함수는 글롭으로도 가져와지지 않습니다. 글롭은 "공개 항목 전부"를 가져옵니다.
        fn secret_blend() -> String {
            String::from("하우스 블렌드")
        }
    }
}

fn by_parent_module() {
    // 해결 1: 타입까지 가져오지 않고 부모 모듈까지만 가져와서, 쓸 때 모듈 이름으로 구분합니다.
    use crate::menu::dinner;
    use crate::menu::lunch;

    let a = lunch::Course::Bibimbap;
    let b = dinner::Course::Steak;
    println!("  점심: {a:?} / 저녁: {b:?}");
}

fn by_alias() {
    // 해결 2: `as`로 이 스코프에서 쓸 새 이름(별칭)을 붙입니다. 원래 타입은 그대로이고 이름만 다릅니다.
    use crate::menu::dinner::Course as DinnerCourse;
    use crate::menu::lunch::Course as LunchCourse;

    let a = LunchCourse::Kimbap;
    let b = DinnerCourse::Pasta;
    println!("  점심: {a:?} / 저녁: {b:?}");
}

fn nested_paths() {
    // 앞부분이 같은 경로 여러 개는 `{ }`로 묶어 한 줄로 씁니다. 아래 한 줄은 다음 세 줄과 같습니다.
    //   use crate::menu::dinner;
    //   use crate::menu::drinks;
    //   use crate::menu::lunch;
    use crate::menu::{dinner, drinks, lunch};
    // `self`는 "공통 부분 그 자체"를 뜻합니다. 아래 한 줄은 다음 두 줄과 같습니다.
    //   use std::cmp;
    //   use std::cmp::Ordering;
    use std::cmp::{self, Ordering};

    let a = lunch::Course::Kimbap;
    let b = dinner::Course::Steak;
    println!("  점심: {a:?} / 저녁: {b:?}");
    drinks::tea();

    let bigger = cmp::max(3, 7);
    let order = 3.cmp(&7);
    println!("  cmp::max(3, 7) = {bigger}, 3과 7을 비교하면 {order:?}");
    if let Ordering::Less = order {
        println!("  3이 7보다 작습니다");
    }
}

fn glob() {
    // `*`(글롭)는 그 모듈의 공개 항목을 전부 가져옵니다. coffee와 tea를 경로 없이 부를 수 있습니다.
    // 편하지만, 코드만 보고는 coffee가 어디서 왔는지 알기 어렵습니다. 그래서 꼭 필요한 곳에만 씁니다.
    use crate::menu::drinks::*;

    coffee();
    tea();
    // secret_blend();  // 비공개라 글롭으로 가져와지지 않습니다. 주석을 풀면 E0425(찾을 수 없음) 에러가 납니다.
}

fn main() {
    println!("1. 이름이 같은 두 타입을 부모 모듈로 구분하기");
    by_parent_module(); // 출력:   점심: Bibimbap / 저녁: Steak

    println!("2. as로 별칭 붙이기");
    by_alias(); // 출력:   점심: Kimbap / 저녁: Pasta

    println!("3. 중첩 경로로 use 묶기");
    nested_paths();
    // 출력:   점심: Kimbap / 저녁: Steak
    // 출력:   차를 우립니다
    // 출력:   cmp::max(3, 7) = 7, 3과 7을 비교하면 Less
    // 출력:   3이 7보다 작습니다

    println!("4. 글롭(*)으로 공개 항목 전부 가져오기");
    glob();
    // 출력:   커피를 내립니다 (원두: 하우스 블렌드)
    // 출력:   차를 우립니다
}
