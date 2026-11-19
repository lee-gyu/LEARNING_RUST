// 기대하는 에러: E0277 — the trait bound `Rgb: From<Cmy>` is not satisfied
// 핵심 메시지: the trait `From<Cmy>` is not implemented for `Rgb`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_into_only_no_from.rs --out-dir /tmp/er-ch5
//
// `Cmy`는 `Into<Rgb>`만 구현했습니다(Rust 1.41 이전에 흔하던 방식). `From`을 구현하면 `Into`가
// 포괄 구현으로 따라오지만, 반대 방향의 포괄 구현은 없습니다. 그래서 `Rgb: From<T>`를 바운드로
// 요구하는 함수에는 `Cmy`를 넘길 수 없습니다.
// 해결: 구현하는 쪽은 `impl From<Cmy> for Rgb`로 바꿉니다(clippy `from_over_into`의 제안과 같음).
//       받는 쪽은 바운드를 `T: Into<Rgb>`로 바꿉니다(examples/02_from_impl_into_bound.rs의 `fill`).
//       둘 중 하나만 고쳐도 컴파일되지만, 고칠 수 있다면 둘 다 고칩니다.

#[derive(Debug)]
struct Rgb(u8, u8, u8);

impl From<[u8; 3]> for Rgb {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Rgb(r, g, b)
    }
}

struct Cmy(u8, u8, u8);

#[allow(clippy::from_over_into)]
impl Into<Rgb> for Cmy {
    fn into(self) -> Rgb {
        Rgb(255 - self.0, 255 - self.1, 255 - self.2)
    }
}

fn fill_from<T>(color: T) -> Rgb
where
    Rgb: From<T>,
{
    Rgb::from(color)
}

fn main() {
    println!("{:?}", fill_from([255, 136, 0])); // From<[u8; 3]>이 있으므로 됩니다.
    println!("{:?}", fill_from(Cmy(0, 255, 255))); // E0277
}
