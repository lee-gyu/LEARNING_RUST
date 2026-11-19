//! 지침 1: `unsafe`를 쓰기 전에 표준 라이브러리에서 찾는다 — 바이트를 수와 문자열로 읽을 때.
//!
//! 블루투스 체중계가 보내는 패킷을 읽습니다. 기기 설명서에 적힌 형식은 다음과 같습니다.
//!
//! | 위치 | 내용 |
//! | --- | --- |
//! | 0..2 | 측정 번호, u16 리틀 엔디언 |
//! | 2..4 | 체중, u16 리틀 엔디언, 0.1kg 단위 |
//! | 4.. | 사용자 이름, UTF-8 |
//!
//! C에서 하던 대로 포인터를 `u16`으로 바꿔 읽고 `from_utf8_unchecked`로 이름을 만든 판(비권장)과
//! `from_le_bytes`·`str::from_utf8`로 읽은 판(권장)을 같은 패킷 네 개에 실행합니다.
//! 비권장 판도 건전하게 쓰려니 이름 검사를 ASCII로 좁힐 수밖에 없었고, 그 때문에 한글 이름을 읽지 못합니다.
//! 이 파일의 `unsafe` 블록은 Miri로 검사하지 못했습니다(nightly 툴체인을 설치하지 않음).
//!
//! 실행: cargo run -p er-ch16-avoid-unsafe --example 02_bytes_to_values

use std::str::Utf8Error;

/// 체중계 패킷 네 개. 앞 4바이트가 번호와 체중이고, 나머지가 이름입니다.
/// B의 이름 `\xEB\xAF\xBC\xEC\xA7\x80`은 "민지"의 UTF-8이고, C의 이름에는 UTF-8이 아닌 0xFF가
/// 끼어 있으며, D는 너무 짧습니다.
const PACKETS: [(&str, &[u8]); 4] = [
    ("A", b"\x01\x02\xD4\x02minji"),
    ("B", b"\x02\x02\x8F\x02\xEB\xAF\xBC\xEC\xA7\x80"),
    ("C", b"\x03\x02\x21\x03Ju\xFFn"),
    ("D", b"\x04"),
];

/// 비권장: C 습관 그대로 포인터와 unchecked 함수로 읽습니다.
mod pointer_cast {
    /// at부터 2바이트를 **이 기계의 바이트 순서로** u16으로 읽습니다.
    pub fn read_u16(bytes: &[u8], at: usize) -> Option<u16> {
        let field = bytes.get(at..at.checked_add(2)?)?;
        // SAFETY: field는 정확히 2바이트이므로 u16 하나를 읽어도 범위를 넘지 않습니다.
        // 바이트 슬라이스는 u16의 메모리 정렬(2)을 보장하지 않으므로 read가 아니라 read_unaligned를 씁니다.
        // u16은 어떤 비트 패턴이든 올바른 값입니다.
        Some(unsafe { field.as_ptr().cast::<u16>().read_unaligned() })
    }

    /// 이름 바이트를 &str로. from_utf8_unchecked의 전제 조건(올바른 UTF-8)을 직접 확인하려면
    /// UTF-8 검사기를 새로 써야 하므로, 확인하기 쉬운 ASCII만 받기로 했습니다.
    pub fn read_name(bytes: &[u8]) -> Option<&str> {
        if !bytes.is_ascii() {
            return None;
        }
        // SAFETY: 바로 위에서 모든 바이트가 ASCII임을 확인했고, ASCII 바이트열은 올바른 UTF-8입니다.
        Some(unsafe { std::str::from_utf8_unchecked(bytes) })
    }
}

/// 권장: 표준 라이브러리의 안전한 변환만 씁니다.
mod std_api {
    use std::str::Utf8Error;

    /// at부터 2바이트를 **리틀 엔디언으로** u16으로 읽습니다.
    pub fn read_u16_le(bytes: &[u8], at: usize) -> Option<u16> {
        // first_chunk(1.77.0)는 앞의 N바이트를 &[u8; N]으로 줍니다. 모자라면 None.
        let field = bytes.get(at..)?.first_chunk::<2>()?;
        Some(u16::from_le_bytes(*field))
    }

    pub fn read_name(bytes: &[u8]) -> Result<&str, Utf8Error> {
        std::str::from_utf8(bytes)
    }
}

fn kg(tenths: u16) -> String {
    format!("{}.{}kg", tenths / 10, tenths % 10)
}

fn show_pointer_cast(label: &str, packet: &[u8]) {
    let (Some(id), Some(weight), Some(name_bytes)) = (
        pointer_cast::read_u16(packet, 0),
        pointer_cast::read_u16(packet, 2),
        packet.get(4..),
    ) else {
        println!("    패킷 {label} → 너무 짧음");
        return;
    };
    match pointer_cast::read_name(name_bytes) {
        Some(name) => println!("    패킷 {label} → #{id}, {}, 이름 {name:?}", kg(weight)),
        None => println!(
            "    패킷 {label} → #{id}, {}, 이름을 읽지 못함(ASCII가 아님)",
            kg(weight)
        ),
    }
}

fn show_std_api(label: &str, packet: &[u8]) {
    let (Some(id), Some(weight), Some(name_bytes)) = (
        std_api::read_u16_le(packet, 0),
        std_api::read_u16_le(packet, 2),
        packet.get(4..),
    ) else {
        println!("    패킷 {label} → 너무 짧음");
        return;
    };
    let name: Result<&str, Utf8Error> = std_api::read_name(name_bytes);
    match name {
        Ok(name) => println!("    패킷 {label} → #{id}, {}, 이름 {name:?}", kg(weight)),
        Err(e) => println!("    패킷 {label} → #{id}, {}, 이름 오류: {e}", kg(weight)),
    }
}

fn main() {
    println!("1. 비권장: 포인터로 읽기 + ASCII 확인 뒤 from_utf8_unchecked");
    for (label, packet) in PACKETS {
        show_pointer_cast(label, packet);
    }
    // 출력:     패킷 A → #513, 72.4kg, 이름 "minji"
    // 출력:     패킷 B → #514, 65.5kg, 이름을 읽지 못함(ASCII가 아님)
    // 출력:     패킷 C → #515, 80.1kg, 이름을 읽지 못함(ASCII가 아님)
    // 출력:     패킷 D → 너무 짧음

    println!("2. 권장: from_le_bytes + str::from_utf8");
    for (label, packet) in PACKETS {
        show_std_api(label, packet);
    }
    // 출력:     패킷 A → #513, 72.4kg, 이름 "minji"
    // 출력:     패킷 B → #514, 65.5kg, 이름 "민지"
    // 출력:     패킷 C → #515, 80.1kg, 이름 오류: invalid utf-8 sequence of 1 bytes from index 2
    // 출력:     패킷 D → 너무 짧음

    println!("3. 바이트 순서");
    // 이 줄은 실행하는 기계에 따라 다릅니다. 확인한 기계(aarch64 macOS)는 little입니다.
    let native = if cfg!(target_endian = "little") {
        "little"
    } else {
        "big"
    };
    println!("    이 기계의 바이트 순서: {native}"); // 출력:     이 기계의 바이트 순서: little
    let weight_bytes = [0xD4, 0x02];
    let le = u16::from_le_bytes(weight_bytes);
    let be = u16::from_be_bytes(weight_bytes);
    println!("    체중 바이트 {weight_bytes:02X?}: from_le_bytes → {le}, from_be_bytes → {be}");
    // 출력:     체중 바이트 [D4, 02]: from_le_bytes → 724, from_be_bytes → 54274
    println!(
        "    빅 엔디언 기계라면 포인터 판은 {} 대신 {}을 읽습니다",
        kg(le),
        kg(be)
    );
    // 출력:     빅 엔디언 기계라면 포인터 판은 72.4kg 대신 5427.4kg을 읽습니다
}
