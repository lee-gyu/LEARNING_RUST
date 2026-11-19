// 기대하는 에러: E0599 — no method named `cos` found for struct `Degrees` in the current scope
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0599_inner_method_on_newtype.rs --out-dir /tmp/er-ch6
//
// 뉴타입은 안쪽 타입(f64)의 메서드를 물려받지 않습니다. 이 에러는 "도를 그대로 삼각함수에
// 넣으려 했다"는 실수를 잡은 것이므로 반가운 에러입니다.
// 주의: 컴파일러의 `help:`는 `heading.0.cos()`를 제안합니다. 그대로 따르면 컴파일은 되지만
//       135도를 135라디안으로 계산하는, 바로 그 버그가 됩니다.
// 해결: 단위를 바꾼 뒤 라디안의 메서드를 부릅니다. `heading.to_radians().cos()`
//       (examples/04_choose_operations.rs의 권장 판).

#[derive(Debug, Clone, Copy)]
struct Degrees(f64);

#[derive(Debug, Clone, Copy)]
struct Radians(f64);

impl Degrees {
    fn to_radians(self) -> Radians {
        Radians(self.0.to_radians())
    }
}

impl Radians {
    fn cos(self) -> f64 {
        self.0.cos()
    }
}

fn main() {
    let heading = Degrees(135.0);
    let dx = 10.0 * heading.cos(); // E0599
    let ok = 10.0 * heading.to_radians().cos();
    println!("{dx} {ok}");
}
