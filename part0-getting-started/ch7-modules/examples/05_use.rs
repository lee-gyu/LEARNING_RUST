// `use`는 긴 경로에 짧은 이름(바로 가기)을 만들어 줍니다.
// 이 예제는 `use`의 관용적인 쓰임새와 "use는 선언한 스코프 안에서만 유효하다"는 규칙을 보여 줍니다.
//
// 실행: cargo run -p p0-ch7-modules --example 05_use

mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist(name: &str) {
            println!("  대기 명단에 추가: {name}");
        }
    }
}

mod back_of_house {
    #[derive(Debug)]
    pub enum Appetizer {
        Soup,
        Salad,
    }

    impl Appetizer {
        pub fn price(&self) -> u32 {
            match self {
                Appetizer::Soup => 4000,
                Appetizer::Salad => 5500,
            }
        }
    }
}

// 크레이트 루트에 쓴 use입니다. 이 파일의 맨 바깥 스코프(크레이트 루트)에서만 유효합니다.
//
// 관용 1: 함수는 "부모 모듈"까지만 가져옵니다.
//   hosting::add_to_waitlist()처럼 부르면, 호출하는 곳만 보고도 이 함수가 다른 모듈에 있다는 것을 알 수 있습니다.
use crate::front_of_house::hosting;
// 관용 2: 구조체·열거형 같은 타입은 "그 타입까지" 전체 경로로 가져옵니다.
//   그래서 Appetizer::Soup처럼 타입 이름부터 씁니다.
use crate::back_of_house::Appetizer;
// 표준 라이브러리(std)도 다른 크레이트이므로 크레이트 이름 std로 시작하는 경로를 씁니다.
// std::cmp::max는 함수이므로 관용대로 부모 모듈 cmp까지만 가져옵니다.
use std::cmp;

// 자식 모듈은 크레이트 루트와 다른 스코프입니다. 위의 use는 여기까지 미치지 않습니다.
mod customer {
    // 방법 1: 이 모듈 안에서 따로 use를 씁니다.
    use crate::front_of_house::hosting;

    pub fn arrive(name: &str) {
        hosting::add_to_waitlist(name);
    }
}

mod takeout {
    // 방법 2: 부모(크레이트 루트)에 use로 만들어 둔 이름을 super::로 가리킵니다.
    // use로 만든 이름도 그 스코프의 항목처럼 경로로 찾을 수 있습니다.
    pub fn call(name: &str) {
        super::hosting::add_to_waitlist(name);
    }
}

fn compare_prices(a: Appetizer, b: Appetizer) {
    // 함수 본문 안에도 use를 쓸 수 있습니다. 이 use는 이 함수 안에서만 유효합니다.
    // Ordering은 열거형이므로 관용대로 타입까지 가져옵니다(ch2의 숫자 맞히기 게임에서 쓴 그 Ordering입니다).
    use std::cmp::Ordering;

    let verdict = match a.price().cmp(&b.price()) {
        Ordering::Less => "더 쌉니다",
        Ordering::Greater => "더 비쌉니다",
        Ordering::Equal => "가격이 같습니다",
    };
    println!("  {a:?}는 {b:?}보다 {verdict}");
}

fn main() {
    println!("1. 함수는 부모 모듈 이름과 함께 부릅니다");
    hosting::add_to_waitlist("김민준"); // 출력:   대기 명단에 추가: 김민준

    println!("2. 타입은 전체 경로로 가져와 타입 이름부터 씁니다");
    let soup = Appetizer::Soup;
    println!("  {soup:?}: {}원", soup.price()); // 출력:   Soup: 4000원

    println!("3. 표준 라이브러리 함수도 같은 관용을 따릅니다");
    let higher = cmp::max(Appetizer::Soup.price(), Appetizer::Salad.price());
    println!("  더 비싼 쪽: {higher}원"); // 출력:   더 비싼 쪽: 5500원

    println!("4. 자식 모듈에서는 자기 use를 쓰거나 super::로 가리킵니다");
    customer::arrive("이서연"); // 출력:   대기 명단에 추가: 이서연
    takeout::call("박지호"); // 출력:   대기 명단에 추가: 박지호

    println!("5. 함수 안의 use는 그 함수 안에서만 유효합니다");
    compare_prices(Appetizer::Soup, Appetizer::Salad); // 출력:   Soup는 Salad보다 더 쌉니다
    // main에서는 Ordering을 가져온 적이 없으므로 여기서 Ordering::Less라고 쓰면
    // E0433(경로를 찾을 수 없음) 에러가 납니다. std::cmp::Ordering::Less처럼 전체 경로로 써야 합니다.
}
