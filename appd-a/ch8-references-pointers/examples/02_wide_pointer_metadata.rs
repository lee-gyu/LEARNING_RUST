// 지침 2: 넓은 포인터가 들고 다니는 정보에 맡깁니다.
//
// - 슬라이스 `&[T]`는 주소와 함께 "길이"를 들고 다닙니다. 범위를 넘길 때 전체와 위치·개수를
//   따로 넘기지 말고, 그 범위의 슬라이스 하나를 넘깁니다.
// - 트레이트 객체 `&dyn Trait`는 주소와 함께 "vtable(메서드 포인터 표)"을 들고 다닙니다.
//   포인터 하나에는 추가 정보를 하나만 실을 수 있으므로, 길이와 vtable이 모두 필요하면
//   포인터가 두 겹이 됩니다.
//
// 실행: cargo run -p er-ch8-references-pointers --example 02_wide_pointer_metadata

use std::fmt::Display;
use std::ptr;

/// 비권장: 전체 기록과 범위를 따로 받습니다.
/// 둘째·셋째 인자는 둘 다 `usize`라서 "개수"인지 "끝 위치"인지 시그니처로는 알 수 없습니다.
fn rain_total_in(days: &[u32], start: usize, count: usize) -> u32 {
    days[start..start + count].iter().sum()
}

/// 권장: 범위를 슬라이스 하나로 받습니다. 몇 개인지는 슬라이스가 알고 있습니다.
fn rain_total(days: &[u32]) -> u32 {
    days.iter().sum()
}

/// 센서 오류값 999를 0으로 바꾸고 바꾼 개수를 돌려줍니다.
/// `&mut [u32]`로 받았으므로 이 함수는 넘겨받은 범위 밖을 읽을 수도, 고칠 수도 없습니다.
fn clear_errors(days: &mut [u32]) -> usize {
    let mut cleared = 0;
    for d in days.iter_mut().filter(|d| **d == 999) {
        *d = 0;
        cleared += 1;
    }
    cleared
}

trait Instrument {
    fn reading(&self) -> String;
}

struct RainGauge {
    mm: u32,
}

struct WindVane {
    degrees: u16,
    gusty: bool,
}

impl Instrument for RainGauge {
    fn reading(&self) -> String {
        format!("강수량 {}mm", self.mm)
    }
}

impl Instrument for WindVane {
    fn reading(&self) -> String {
        let gust = if self.gusty { ", 돌풍" } else { "" };
        format!("풍향 {}도{gust}", self.degrees)
    }
}

/// 섞인 장비 목록: 바깥 슬라이스가 길이를, 원소인 `&dyn Instrument`가 각각 vtable을 들고 다닙니다.
fn report(panel: &[&dyn Instrument]) -> Vec<String> {
    panel.iter().map(|item| item.reading()).collect()
}

fn main() {
    // 한 달(30일) 동안의 일별 강수량(mm). 999는 센서 오류값입니다.
    #[rustfmt::skip]
    let mut month: [u32; 30] = [
        0, 0, 3, 0, 12, 0, 0, 0, 5, 0,       // 1~10일
        18, 42, 37, 25, 9, 0, 0, 2, 0, 0,    // 11~20일 (11~15일이 장마)
        0, 4, 999, 0, 0, 999, 0, 7, 0, 1,    // 21~30일
    ];

    println!("1. 비권장: 범위를 위치와 개수로 따로 넘깁니다");
    // 부르는 쪽은 Rust의 범위 문법(10..15)처럼 둘째 인자를 "끝 위치"로 읽고 15를 넘겼습니다.
    // 받는 쪽은 "개수"로 읽어 15일치(인덱스 10..25)를 더합니다. 컴파일러는 둘 다 usize라 막지 못합니다.
    println!("    장마 기간 강수량: {}mm", rain_total_in(&month, 10, 15));
    // 출력:     장마 기간 강수량: 1136mm

    println!("2. 권장: 범위의 슬라이스를 넘깁니다");
    let rainy = &month[10..15];
    println!(
        "    장마 기간 강수량: {}mm ({}일치)",
        rain_total(rainy),
        rainy.len()
    );
    // 출력:     장마 기간 강수량: 131mm (5일치)
    // 슬라이스는 원본을 가리킬 뿐 복사하지 않습니다.
    println!(
        "    슬라이스가 원본의 11일째를 가리키는가: {}",
        ptr::addr_eq(rainy, &month[10])
    );
    // 출력:     슬라이스가 원본의 11일째를 가리키는가: true
    // 가변 슬라이스로 범위를 넘기면, 받는 쪽이 고칠 수 있는 곳도 그 범위로 제한됩니다.
    let cleared = clear_errors(&mut month[20..25]);
    println!(
        "    21~25일의 오류값 {cleared}개 정리, 범위 밖 26일째는 그대로: {}",
        month[25]
    );
    // 출력:     21~25일의 오류값 1개 정리, 범위 밖 26일째는 그대로: 999

    println!("3. 길이는 포인터 값의 일부입니다");
    let first_week = &month[..7];
    let whole = &month[..];
    // ptr::eq는 넓은 포인터의 주소와 추가 정보(여기서는 길이)를 모두 비교합니다.
    println!(
        "    주소가 같은가(addr_eq): {}, 포인터가 같은가(eq): {}",
        ptr::addr_eq(first_week, whole),
        ptr::eq(first_week, whole)
    );
    // 출력:     주소가 같은가(addr_eq): true, 포인터가 같은가(eq): false
    // split_at은 복사 없이 넓은 포인터 두 개를 새로 만듭니다.
    let (first_half, second_half) = whole.split_at(15);
    println!(
        "    split_at(15): 앞 {}일, 뒤 {}일, 뒤쪽이 16일째를 가리키는가: {}",
        first_half.len(),
        second_half.len(),
        ptr::addr_eq(second_half, &month[15])
    );
    // 출력:     split_at(15): 앞 15일, 뒤 15일, 뒤쪽이 16일째를 가리키는가: true

    println!("4. 트레이트 객체는 길이 대신 vtable을 들고 다닙니다 (64비트 기준)");
    let gauge = RainGauge { mm: 42 };
    let vane = WindVane {
        degrees: 270,
        gusty: true,
    };
    // 타입이 다른 값들을 섞어 넘기려면 "넓은 포인터들의 슬라이스"가 됩니다.
    let panel: [&dyn Instrument; 2] = [&gauge, &vane];
    println!("    {:?}", report(&panel));
    // 출력:     ["강수량 42mm", "풍향 270도, 돌풍"]
    println!(
        "    &[&dyn Instrument] 자체: {}바이트, 그것이 가리키는 원소들: {}바이트",
        size_of::<&[&dyn Instrument]>(),
        size_of_val(&panel[..])
    );
    // 출력:     &[&dyn Instrument] 자체: 16바이트, 그것이 가리키는 원소들: 32바이트
    // `&str`은 이미 길이를 실은 넓은 포인터라 vtable을 더 실을 수 없습니다.
    // 그래서 `let shown: &dyn Display = station;`은 E0277입니다(compile_fail/e0277_str_to_dyn_display.rs).
    // 한 겹 더 감싼 `&&str`은 얇은 포인터라 vtable을 실을 수 있습니다.
    let station: &str = "제주 관측소";
    let shown: &dyn Display = &station;
    println!(
        "    {shown}: &&str은 {}바이트, 그것을 담은 &dyn Display는 {}바이트",
        size_of::<&&str>(),
        size_of::<&dyn Display>()
    );
    // 출력:     제주 관측소: &&str은 8바이트, 그것을 담은 &dyn Display는 16바이트
}
