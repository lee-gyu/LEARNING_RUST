// 이 예제가 보여 주는 것: self를 받지 않는 연관 함수(생성자 관례 `new`), 타입 별칭 `Self`, 여러 개의 impl 블록
// 실행: cargo run -p p0-ch5-structs --example 07_associated_functions

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

// 첫 번째 impl 블록: 인스턴스를 만드는 연관 함수들
impl Rectangle {
    // impl 블록 안의 함수를 모두 연관 함수(associated function)라고 부르고,
    // 그중 첫 매개변수가 self인 것을 특별히 메서드라고 부릅니다.
    // new에는 self가 없으므로 인스턴스 없이 `Rectangle::new(...)`처럼 타입 이름으로 부릅니다.
    //
    // `Self`는 impl 대상 타입(여기서는 Rectangle)의 별칭입니다.
    // 반환 타입과 구조체 만드는 식 모두에 쓸 수 있습니다.
    fn new(width: u32, height: u32) -> Self {
        Self { width, height } // 매개변수 이름이 필드 이름과 같으므로 필드 초기화 축약을 씁니다.
    }

    // Rust에는 생성자 문법이 따로 없습니다. `new`도 관례로 쓰는 이름일 뿐, 특별한 키워드가 아닙니다.
    // 그래서 만드는 방법이 여러 가지라면 이름을 달리해 여러 개 둘 수 있습니다.
    fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

// 두 번째 impl 블록: 메서드들
// 한 타입에 impl 블록을 여러 개 둬도 됩니다. 모두 합쳐 하나의 Rectangle 기능이 됩니다.
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn is_square(&self) -> bool {
        self.width == self.height
    }

    // 메서드 안에서 같은 타입의 연관 함수를 부를 때도 `Self::`를 씁니다.
    // self는 읽기만 하고, 결과는 새 인스턴스로 돌려줍니다.
    fn scaled(&self, factor: u32) -> Self {
        Self::new(self.width * factor, self.height * factor)
    }
}

fn main() {
    println!("1. 연관 함수는 `타입::함수()`로 부릅니다");
    let rect = Rectangle::new(30, 50);
    // 지금까지 써 온 String::from("...")도 String 타입의 연관 함수입니다.
    let label = String::from("rect");
    println!("    {label} = {rect:?}");
    // 출력: rect = Rectangle { width: 30, height: 50 }

    println!("2. 생성자 역할의 연관 함수를 여러 개 둘 수 있습니다");
    let sq = Rectangle::square(3);
    println!("    sq = {sq:?}, 정사각형인가? {}", sq.is_square());
    // 출력: sq = Rectangle { width: 3, height: 3 }, 정사각형인가? true
    // 아래 줄의 주석을 풀면 E0599 에러가 납니다. square는 self가 없어 메서드처럼 점으로 부를 수 없습니다.
    // let wrong = sq.square(4);

    println!("3. 메서드가 Self::new로 새 인스턴스를 만들어 돌려줄 수 있습니다");
    let big = rect.scaled(2);
    println!("    big = {big:?}");
    println!("    rect는 그대로: {rect:?}"); // scaled는 &self만 빌렸으므로 rect를 계속 씁니다.
    // 출력:
    //     big = Rectangle { width: 60, height: 100 }
    //     rect는 그대로: Rectangle { width: 30, height: 50 }

    println!("4. 서로 다른 impl 블록의 함수도 같은 타입의 것입니다");
    // new·square는 첫 번째 블록에, area는 두 번째 블록에 있지만 사용하는 쪽에서는 구분되지 않습니다.
    println!(
        "    넓이: rect = {}, sq = {}, big = {}",
        rect.area(),
        sq.area(),
        big.area()
    );
    // 출력: 넓이: rect = 1500, sq = 9, big = 6000
}
