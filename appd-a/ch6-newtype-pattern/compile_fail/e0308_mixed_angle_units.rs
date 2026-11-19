// 기대하는 에러: E0308 — mismatched types (두 번)
//   1) expected `Radians`, found `Degrees`  (도를 라디안 매개변수에 넘김)
//   2) expected `Degrees`, found `Radians`  (도에 라디안을 더함)
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0308_mixed_angle_units.rs --out-dir /tmp/er-ch6
//
// 타입 별칭(`type Degrees = f64;`)이었다면 두 줄 모두 컴파일되고 틀린 값을 냈을 실수입니다
// (examples/01_unit_newtype.rs의 비권장 판). 뉴타입은 서로 다른 타입이므로 컴파일러가 멈춥니다.
// 해결: 경계에서 `slope.to_radians()`로 단위를 바꿔 넘깁니다. 도에 더할 값은 처음부터 도로 만듭니다.

use std::ops::Add;

#[derive(Debug, Clone, Copy)]
struct Degrees(f64);

#[derive(Debug, Clone, Copy)]
struct Radians(f64);

impl Degrees {
    fn to_radians(self) -> Radians {
        Radians(self.0.to_radians())
    }
}

// 같은 단위끼리의 덧셈만 구현했습니다(Rhs의 기본값이 Self).
impl Add for Degrees {
    type Output = Degrees;
    fn add(self, rhs: Degrees) -> Degrees {
        Degrees(self.0 + rhs.0)
    }
}

fn ramp_height(length_m: f64, slope: Radians) -> f64 {
    length_m * slope.0.sin()
}

fn main() {
    let slope = Degrees(5.0);
    let h = ramp_height(12.0, slope); // E0308: 변환을 빠뜨림
    let tilt = slope.to_radians();
    let total = slope + tilt; // E0308: 도와 라디안을 더함
    println!("{h} {total:?}");
}
