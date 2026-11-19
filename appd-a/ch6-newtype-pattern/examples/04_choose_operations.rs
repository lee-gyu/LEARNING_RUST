// 지침: 뉴타입에는 뜻이 있는 연산만 골라 열어 줍니다. Deref로 안쪽 타입을 통째로 열면
//       단위를 구별하려고 만든 뉴타입이 다시 그냥 f64처럼 쓰입니다.
// 비권장: impl Deref<Target = f64> for DegreesDeref   (f64의 메서드가 모두 열림)
// 권장:   Degrees + Degrees, Degrees -> Radians 변환, Radians의 sin/cos만 엶
//
// 드론이 x축에서 반시계 방향으로 30도를 향해 있다가, 45도와 60도를 차례로 돌고 10m 전진합니다.
//
// 실행: cargo run -p er-ch6-newtype-pattern --example 04_choose_operations

use std::ops::{Add, Deref};

const START_DEG: f64 = 30.0;
const TURNS_DEG: [f64; 2] = [45.0, 60.0];
const DISTANCE_M: f64 = 10.0;

// ---- 비권장 ----------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct DegreesDeref(f64);

impl Add for DegreesDeref {
    type Output = DegreesDeref;
    fn add(self, rhs: DegreesDeref) -> DegreesDeref {
        DegreesDeref(self.0 + rhs.0)
    }
}

// `.0`을 쓰기 귀찮아서 안쪽 f64를 통째로 열었습니다.
// 이제 메서드 탐색이 DegreesDeref에서 못 찾은 메서드를 f64에서 찾으므로,
// `heading.cos()`가 "도를 라디안으로 읽는" f64::cos로 조용히 연결됩니다.
impl Deref for DegreesDeref {
    type Target = f64;
    fn deref(&self) -> &f64 {
        &self.0
    }
}

// ---- 권장 ------------------------------------------------------------------

// derive는 "안쪽 값끼리 비교·복사하는 것이 이 단위에서도 뜻이 있는가"로 고릅니다.
// 같은 단위의 각도끼리 크기를 비교하는 것은 뜻이 있으므로 PartialEq, PartialOrd를 붙입니다.
// (f64가 Eq, Ord, Hash를 구현하지 않으므로 그 셋은 derive할 수 없습니다.)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Degrees(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Radians(f64);

// 도 + 도 = 도. 뜻이 있는 조합 하나만 구현합니다.
// Degrees + Radians, Degrees * Degrees는 구현하지 않았으므로 컴파일러가 막습니다.
impl Add for Degrees {
    type Output = Degrees;
    fn add(self, rhs: Degrees) -> Degrees {
        Degrees(self.0 + rhs.0)
    }
}

impl Degrees {
    fn to_radians(self) -> Radians {
        Radians(self.0.to_radians())
    }
}

// 삼각함수는 라디안에만 있습니다. Degrees에서 `.cos()`를 부르면 E0599로 멈추므로
// 변환을 빠뜨릴 수 없습니다(compile_fail/e0599_inner_method_on_newtype.rs).
impl Radians {
    fn cos(self) -> f64 {
        self.0.cos()
    }
    fn sin(self) -> f64 {
        self.0.sin()
    }
}

fn main() {
    let [first, second] = TURNS_DEG;
    println!("{START_DEG}도에서 출발, {first}도와 {second}도를 차례로 돌고 {DISTANCE_M}m 전진");
    // 출력: 30도에서 출발, 45도와 60도를 차례로 돌고 10m 전진

    println!("[비권장] Deref<Target = f64>");
    let mut heading = DegreesDeref(START_DEG);
    for turn in TURNS_DEG {
        heading = heading + DegreesDeref(turn);
    }
    // DegreesDeref에는 cos, sin이 없지만 Deref 덕분에 f64::cos, f64::sin이 불립니다.
    let (dx, dy) = (DISTANCE_M * heading.cos(), DISTANCE_M * heading.sin());
    println!("  방향 {heading:?}, 이동 ({dx:.2}, {dy:.2})");
    // 출력:   방향 DegreesDeref(135.0), 이동 (-9.96, 0.88)
    // 변환하는 메서드를 불러도 결과는 맨 f64라서, 단위 정보가 다시 사라집니다.
    let r = heading.to_radians();
    println!("  heading.to_radians() = {r:.4} (타입은 f64)");
    // 출력:   heading.to_radians() = 2.3562 (타입은 f64)

    println!("[권장] 같은 단위의 덧셈, 단위 변환, 라디안의 삼각함수만 엶");
    let mut heading = Degrees(START_DEG);
    for turn in TURNS_DEG {
        heading = heading + Degrees(turn);
    }
    let rad = heading.to_radians();
    let (dx, dy) = (DISTANCE_M * rad.cos(), DISTANCE_M * rad.sin());
    println!("  방향 {heading:?}, 이동 ({dx:.2}, {dy:.2})");
    // 출력:   방향 Degrees(135.0), 이동 (-7.07, 7.07)
    // derive한 PartialOrd 덕분에 같은 단위끼리는 비교할 수 있습니다.
    println!("  직각보다 큰 방향인가: {}", heading > Degrees(90.0));
    // 출력:   직각보다 큰 방향인가: true
}
