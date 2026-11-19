// 지침 2: 변환은 `From`으로 구현하고, 변환할 수 있는 값을 받는 매개변수는 `Into` 바운드로 받습니다.
//
// 색을 칠하는 함수를 만듭니다. 함수가 다루는 색은 `Rgb`이지만, 부르는 쪽은 색을 `[u8; 3]` 배열,
// 회색조 값 `Gray`, 옛 크레이트의 `legacy::Cmy`처럼 여러 모양으로 가지고 있습니다.
// 받는 쪽  비권장: 매개변수를 `Rgb`로만 받아, 부르는 쪽이 호출마다 변환을 적게 합니다.
//          권장:   `impl Into<Rgb>`로 받아, 변환은 함수 안에서 한 번 합니다.
// 구현하는 쪽 비권장: `Into`만 구현합니다(`legacy::Cmy`). `Rgb::from(..)`과 `From` 바운드에서 빠집니다.
//             권장:   `From`을 구현합니다(`[u8; 3]`, `Gray`). `Into`는 포괄 구현으로 따라옵니다.
//
// 실행: cargo run -p er-ch5-type-conversions --example 02_from_impl_into_bound

#[derive(Debug, Clone, Copy)]
struct Rgb(u8, u8, u8);

// 권장 구현: 받는 쪽 타입(`Rgb`)에 `From<원래 타입>`을 구현합니다.
impl From<[u8; 3]> for Rgb {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Rgb(r, g, b)
    }
}

struct Gray(u8);

impl From<Gray> for Rgb {
    fn from(Gray(level): Gray) -> Self {
        Rgb(level, level, level)
    }
}

// Rust 1.41 이전에 만든 크레이트를 흉내 냅니다. 그때는 고아 규칙 때문에 다른 크레이트의 타입으로
// 가는 `From`을 구현할 수 없어서 `Into`를 직접 구현하곤 했습니다. 지금은 그럴 이유가 없습니다.
mod legacy {
    use super::Rgb;

    /// 인쇄용 색 표기(C, M, Y)입니다. RGB와 정확히 맞바꿀 수 있어 손실 없는 변환입니다.
    pub struct Cmy(pub u8, pub u8, pub u8);

    // 비권장 구현: clippy가 기본 설정에서 `from_over_into`로 경고합니다.
    #[expect(
        clippy::from_over_into,
        reason = "Into만 구현했을 때 무엇을 잃는지 보여 주는 비권장 구현"
    )]
    impl Into<Rgb> for Cmy {
        fn into(self) -> Rgb {
            Rgb(255 - self.0, 255 - self.1, 255 - self.2)
        }
    }
}

/// 비권장(받는 쪽): `Rgb`만 받습니다. 권장 함수 `fill`도 본문에서 이 함수를 부릅니다.
fn fill_exact(color: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2)
}

/// 권장(받는 쪽): `Rgb`로 바꿀 수 있는 것이면 무엇이든 받습니다.
/// 제네릭 부분은 `into()` 한 줄뿐이고 실제 일은 제네릭이 아닌 `fill_exact`가 합니다.
/// 부르는 타입마다 복제되는 코드(단형성화)가 그만큼 작아집니다.
fn fill(color: impl Into<Rgb>) -> String {
    fill_exact(color.into())
}

/// 비권장(받는 쪽 바운드): `Into` 대신 `From`을 바운드로 적었습니다.
/// `Into`만 구현한 타입은 넘길 수 없습니다(compile_fail/e0277_into_only_no_from.rs).
fn fill_from<T>(color: T) -> String
where
    Rgb: From<T>,
{
    fill_exact(Rgb::from(color))
}

fn main() {
    println!("1. 비권장: Rgb만 받는 함수 — 부르는 쪽이 호출마다 변환합니다");
    let a = fill_exact(Rgb::from([255, 136, 0]));
    let b = fill_exact(Gray(200).into());
    let c = fill_exact(legacy::Cmy(0, 255, 255).into());
    println!("    {a} {b} {c}"); // 출력:     #ff8800 #c8c8c8 #ff0000

    println!("2. 권장: impl Into<Rgb>로 받는 함수 — 가진 값을 그대로 넘깁니다");
    let a = fill([255, 136, 0]);
    let b = fill(Gray(200));
    let c = fill(legacy::Cmy(0, 255, 255)); // Into만 구현한 타입도 받습니다.
    let d = fill(Rgb(0, 0, 0)); // 반사 구현(From<T> for T) 덕분에 Rgb 자신도 받습니다.
    println!("    {a} {b} {c} {d}"); // 출력:     #ff8800 #c8c8c8 #ff0000 #000000

    println!("3. Into만 구현한 타입은 From으로 쓰는 곳에서 빠집니다");
    // `From`을 구현한 타입은 `Rgb::from`을 함수처럼 넘길 수 있습니다.
    let grays: Vec<Rgb> = [Gray(0), Gray(255)].into_iter().map(Rgb::from).collect();
    // `Into`만 구현한 타입은 `Rgb::from`이 없으므로 `Into::into`를 넘기고 목적 타입을 따로 적습니다.
    let inks: Vec<Rgb> = [legacy::Cmy(255, 255, 0), legacy::Cmy(0, 0, 0)]
        .into_iter()
        .map(Into::into)
        .collect();
    println!("    {grays:?}"); // 출력:     [Rgb(0, 0, 0), Rgb(255, 255, 255)]
    println!("    {inks:?}"); // 출력:     [Rgb(0, 0, 255), Rgb(255, 255, 255)]
    // `From` 바운드 함수는 `From`을 구현한 타입만 받습니다.
    // `fill_from(legacy::Cmy(0, 255, 255))`는 E0277입니다.
    println!("    fill_from(Gray(128)) = {}", fill_from(Gray(128))); // 출력:     fill_from(Gray(128)) = #808080
}
