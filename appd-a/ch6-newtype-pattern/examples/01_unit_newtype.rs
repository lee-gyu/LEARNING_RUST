// 지침: 단위가 다른 값은 타입 별칭(`type`)이 아니라 뉴타입으로 구별합니다.
// 비권장: type Degrees = f64; type Radians = f64;   (이름만 다르고 같은 타입)
// 권장:   struct Degrees(f64); struct Radians(f64);  (서로 다른 타입)
//
// 경사로의 높이를 "길이 × sin(경사각)"으로 계산합니다. 도면은 경사각을 도(degree)로 적고,
// `f64::sin`은 라디안(radian)을 받습니다.
//
// 실행: cargo run -p er-ch6-newtype-pattern --example 01_unit_newtype

const RAMP_LENGTH_M: f64 = 12.0;
const SLOPES_FROM_DRAWING: [f64; 2] = [3.0, 5.0]; // 도면에 적힌 경사각(도)

// ---- 비권장 ----------------------------------------------------------------

mod alias {
    // 별칭은 새 타입을 만들지 않습니다. Degrees, Radians, f64는 컴파일러에게 모두 같은 타입입니다.
    pub type Degrees = f64;
    pub type Radians = f64;

    // 시그니처는 "라디안을 달라"고 적어 두었지만, 그 말은 사람만 읽을 수 있습니다.
    pub fn ramp_height(length_m: f64, slope: Radians) -> f64 {
        length_m * slope.sin()
    }
}

// ---- 권장 ------------------------------------------------------------------

mod newtype {
    // 필드 하나짜리 튜플 구조체입니다. 안쪽은 똑같이 f64이지만 둘은 서로 다른 타입입니다.
    #[derive(Debug, Clone, Copy)]
    pub struct Degrees(pub f64);

    #[derive(Debug, Clone, Copy)]
    pub struct Radians(pub f64);

    impl Degrees {
        // 단위를 바꾸는 길은 이 메서드 하나뿐입니다. 변환이 코드에 이름으로 드러납니다.
        pub fn to_radians(self) -> Radians {
            Radians(self.0.to_radians())
        }
    }

    // 이제 "라디안을 달라"는 말을 컴파일러도 읽습니다.
    // Degrees를 넘기면 E0308입니다(compile_fail/e0308_mixed_angle_units.rs).
    pub fn ramp_height(length_m: f64, slope: Radians) -> f64 {
        length_m * slope.0.sin()
    }
}

fn main() {
    println!("경사로 길이 {RAMP_LENGTH_M}m");

    println!("[비권장] 타입 별칭");
    for deg in SLOPES_FROM_DRAWING {
        let slope: alias::Degrees = deg;
        // 변환을 빠뜨렸지만 Degrees와 Radians가 같은 f64라서 그대로 컴파일됩니다.
        let h = alias::ramp_height(RAMP_LENGTH_M, slope);
        println!("  {deg}도 -> 높이 {h:.2}m");
    }
    // 출력:
    //   3도 -> 높이 1.69m
    //   5도 -> 높이 -11.51m

    println!("[권장] 뉴타입");
    for deg in SLOPES_FROM_DRAWING {
        let slope = newtype::Degrees(deg);
        // newtype::ramp_height(RAMP_LENGTH_M, slope)는 컴파일되지 않으므로
        // 변환을 빠뜨릴 수 없습니다.
        let h = newtype::ramp_height(RAMP_LENGTH_M, slope.to_radians());
        println!("  {:?} -> 높이 {h:.2}m", slope);
    }
    // 출력:
    //   Degrees(3.0) -> 높이 0.63m
    //   Degrees(5.0) -> 높이 1.05m

    // 별칭에는 "이 값의 단위"가 남지 않습니다. 디버그 형식으로 찍어도 그냥 수입니다.
    let a: alias::Degrees = 5.0;
    let b: alias::Radians = a; // 아무 변환 없이 대입됩니다.
    println!("별칭: a = {a:?}, b = {b:?}");
    // 출력: 별칭: a = 5.0, b = 5.0
}
