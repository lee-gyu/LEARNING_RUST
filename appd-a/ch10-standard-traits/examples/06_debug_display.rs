// 지침: Debug는 거의 모든 공개 타입에 구현합니다. 다만 비밀이 든 타입은 derive하지 않고
//       그 타입의 Debug를 손으로 써서 가립니다. Display는 사용자에게 보여 줄 표현이
//       하나로 분명할 때만 구현하고, 표현이 여럿이면 이름 붙은 메서드로 고르게 합니다.
// 비권장: 카드 번호를 String 필드로 두고 derive(Debug) (로그에 번호가 그대로 남음)
//         바이트 크기에 Display를 구현하면서 단위 체계 하나를 골라 버림
// 권장:   CardNumber 타입의 Debug에서 가림, ByteSize는 si()/iec()로 표현을 고르게 함
//
// 실행: cargo run -p er-ch10-standard-traits --example 06_debug_display

use std::fmt;

// ---- 비밀이 든 값의 Debug ------------------------------------------------------

// 비권장: derive(Debug)는 모든 필드를 출력합니다. {:?}로 남긴 로그에 카드 번호가 그대로 남습니다.
#[derive(Debug)]
struct RawPayment {
    order: u32,
    card: String,
}

// 권장: 비밀을 담는 타입을 따로 두고, 그 타입의 Debug에서 가립니다.
pub struct CardNumber(String);

impl CardNumber {
    pub fn new(digits: &str) -> CardNumber {
        CardNumber(digits.to_string())
    }
    fn last4(&self) -> &str {
        let len = self.0.len();
        self.0.get(len.saturating_sub(4)..).unwrap_or("")
    }
}

impl fmt::Debug for CardNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 도우미를 쓰면 {:#?}의 여러 줄 출력도 저절로 맞춰집니다.
        f.debug_tuple("CardNumber")
            .field(&format_args!("****-{}", self.last4()))
            .finish()
    }
}

// 결제 타입은 그대로 derive합니다. derive(Debug)는 필드마다 그 필드 타입의 Debug를 부르므로
// card 필드는 CardNumber가 정한 대로 가려져 나옵니다.
#[derive(Debug)]
pub struct Payment {
    pub order: u32,
    pub card: CardNumber,
}

// ---- 사용자에게 보여 줄 표현 ---------------------------------------------------

const KB: f64 = 1000.0;
const KIB: f64 = 1024.0;

fn human(bytes: u64, base: f64, units: [&str; 4]) -> String {
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= base && unit < units.len() - 1 {
        value /= base;
        unit += 1;
    }
    format!("{value:.1} {}", units[unit])
}

// 비권장: 크기를 보여 주는 방식은 여럿(1000 단위, 1024 단위, 바이트 그대로)인데
// Display 하나에 1024 단위를 골라 넣었습니다. 사용자는 다른 표현을 고를 수 없고,
// 고아 규칙 때문에 이 타입에 자기 Display를 구현할 수도 없습니다.
pub struct FixedSize(pub u64);

impl fmt::Display for FixedSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", human(self.0, KIB, ["B", "KiB", "MiB", "GiB"]))
    }
}

// 권장: Display를 구현하지 않고, 표현마다 이름 붙은 메서드가 Display를 구현한 도우미를 돌려줍니다.
// 표준 라이브러리의 Path::display()와 같은 모양입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteSize(pub u64);

pub struct Si(u64);
pub struct Iec(u64);

impl ByteSize {
    /// 1000 단위(kB, MB): 저장 장치 용량 표기에 흔히 씁니다.
    pub fn si(self) -> Si {
        Si(self.0)
    }
    /// 1024 단위(KiB, MiB): 메모리 크기 표기에 흔히 씁니다.
    pub fn iec(self) -> Iec {
        Iec(self.0)
    }
}

impl fmt::Display for Si {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", human(self.0, KB, ["B", "kB", "MB", "GB"]))
    }
}

impl fmt::Display for Iec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", human(self.0, KIB, ["B", "KiB", "MiB", "GiB"]))
    }
}

fn main() {
    println!("[비권장] 카드 번호가 든 구조체에 derive(Debug)");
    let raw = RawPayment {
        order: 1042,
        card: String::from("1234-5678-9012-3456"),
    };
    println!("  로그: {raw:?}");
    // 출력:   로그: RawPayment { order: 1042, card: "1234-5678-9012-3456" }
    println!("  주문 {}, 카드 끝자리 {}", raw.order, &raw.card[15..]);
    // 출력:   주문 1042, 카드 끝자리 3456

    println!("[권장] 비밀을 담는 타입의 Debug에서 가림");
    let payment = Payment {
        order: 1042,
        card: CardNumber::new("1234-5678-9012-3456"),
    };
    println!("  로그: {payment:?}");
    // 출력:   로그: Payment { order: 1042, card: CardNumber(****-3456) }
    println!("  여러 줄 로그:\n{payment:#?}");
    // 출력:   여러 줄 로그:
    // 출력: Payment {
    // 출력:     order: 1042,
    // 출력:     card: CardNumber(
    // 출력:         ****-3456,
    // 출력:     ),
    // 출력: }

    let size = 1_572_864;
    println!("[비권장] Display가 단위 체계 하나를 골라 버림");
    println!("  {}", FixedSize(size));
    // 출력:   1.5 MiB

    println!("[권장] 표현을 이름으로 고름");
    let size = ByteSize(size);
    println!(
        "  si: {}, iec: {}, 바이트: {}",
        size.si(),
        size.iec(),
        size.0
    );
    // 출력:   si: 1.6 MB, iec: 1.5 MiB, 바이트: 1572864
}
