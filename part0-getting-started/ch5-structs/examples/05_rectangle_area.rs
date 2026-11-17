// 이 예제가 보여 주는 것: 사각형 넓이를 구하는 프로그램을
//   "변수 두 개 → 튜플 → 구조체 → 메서드" 순서로 다듬으며, 구조체가 코드의 의미를 어떻게 드러내는지 봅니다.
// 실행: cargo run -p p0-ch5-structs --example 05_rectangle_area
//
// 단계마다 결과(1500)는 같습니다. 달라지는 것은 함수의 시그니처가 "무엇을 받는지"를 얼마나 잘 말해 주는가입니다.

// ── 1단계: 변수 두 개 ─────────────────────────────────────────────
// 시그니처만 보면 width와 height가 같은 사각형의 값이라는 사실이 드러나지 않습니다.
// 사각형이 여러 개면 width1, height1, width2, ...처럼 짝을 이름으로만 맞춰야 합니다.
fn area_from_values(width: u32, height: u32) -> u32 {
    width * height
}

// ── 2단계: 튜플 ──────────────────────────────────────────────────
// 값 하나로 묶였지만 원소에 이름이 없습니다. `.0`이 가로인지 세로인지는 기억에 의존해야 합니다.
// 넓이는 순서가 바뀌어도 같지만, 화면에 그리는 코드라면 가로·세로를 뒤바꾸는 버그가 생기기 쉽습니다.
fn area_from_tuple(dimensions: (u32, u32)) -> u32 {
    dimensions.0 * dimensions.1
}

// ── 3단계: 구조체 ────────────────────────────────────────────────
// 묶음 전체(Rectangle)와 각 값(width, height)에 모두 이름이 생겼습니다.
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

// 넓이를 구하는 데는 읽기만 하면 되므로 소유권을 가져오지 않고 불변 참조로 빌립니다(ch4).
// 참조를 통해서도 `rectangle.width`처럼 점으로 필드를 읽습니다. `(*rectangle).width`라고 쓸 필요가 없습니다.
fn area_from_struct(rectangle: &Rectangle) -> u32 {
    rectangle.width * rectangle.height
}

// ── 4단계: 메서드 ────────────────────────────────────────────────
// 3단계의 함수는 Rectangle에만 쓸 수 있는데도 Rectangle과 따로 떨어져 있습니다.
// `impl Rectangle` 블록 안에 넣으면 "Rectangle이 할 수 있는 일"로 묶입니다.
impl Rectangle {
    // 첫 매개변수 `&self`는 이 메서드를 호출한 인스턴스를 빌려 온 것입니다.
    // 3단계의 `rectangle: &Rectangle`과 같은 뜻을 짧게 쓴 것입니다.
    fn area(&self) -> u32 {
        self.width * self.height
    }

    // 메서드도 self 뒤에 매개변수를 더 받을 수 있습니다.
    // 다른 사각형은 읽기만 하므로 역시 불변 참조로 받습니다.
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

fn main() {
    println!("1단계: 변수 두 개");
    let width1 = 30;
    let height1 = 50;
    println!("    넓이: {}", area_from_values(width1, height1));
    // 출력: 넓이: 1500

    println!("2단계: 튜플");
    let rect1 = (30, 50);
    println!("    넓이: {}", area_from_tuple(rect1));
    // 출력: 넓이: 1500

    println!("3단계: 구조체");
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };
    // 호출하는 쪽에서도 `&`가 보이므로 rect1을 빌려주기만 한다는 것을 알 수 있습니다.
    println!("    넓이: {}", area_from_struct(&rect1));
    // 빌려주기만 했으므로 rect1은 여전히 main의 것입니다.
    println!("    rect1 = {rect1:?}");
    // 출력:
    //     넓이: 1500
    //     rect1 = Rectangle { width: 30, height: 50 }

    println!("4단계: 메서드");
    // `인스턴스.메서드()`로 호출합니다. `&rect1`을 직접 쓰지 않아도 Rust가 알아서 빌려 줍니다(06번 예제).
    println!("    넓이: {}", rect1.area());
    let rect2 = Rectangle {
        width: 10,
        height: 40,
    };
    let rect3 = Rectangle {
        width: 60,
        height: 45,
    };
    println!("    rect1에 rect2가 들어가는가? {}", rect1.can_hold(&rect2));
    println!("    rect1에 rect3가 들어가는가? {}", rect1.can_hold(&rect3));
    // 출력:
    //     넓이: 1500
    //     rect1에 rect2가 들어가는가? true
    //     rect1에 rect3가 들어가는가? false
}
