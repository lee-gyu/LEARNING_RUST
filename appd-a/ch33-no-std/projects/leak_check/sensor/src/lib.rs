//! 센서 읽기 값을 다루는 `no_std` 라이브러리입니다. std가 필요한 집계만 std 기능 뒤에 있습니다.

#![no_std]

#[cfg(test)]
extern crate std;

/// 센서 한 개의 읽기 값
#[derive(Debug, Clone, Copy)]
pub struct Reading {
    pub sensor: u32,
    pub raw: [u8; 4],
}

impl Reading {
    /// 원시 값을 16진수로 표시합니다. tinyhex의 core API만 쓰므로 할당하지 않습니다.
    pub fn raw_hex(&self) -> tinyhex::Hex<'_> {
        tinyhex::Hex(&self.raw)
    }
}

/// 읽기 값에 나온 서로 다른 센서의 수입니다. stdful에 맡기므로 std 기능에만 있습니다.
#[cfg(feature = "std")]
pub fn distinct_sensors(readings: &[Reading]) -> usize {
    stdful::count_distinct(readings.iter().map(|r| r.sensor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::format;

    const READINGS: [Reading; 3] = [
        Reading {
            sensor: 7,
            raw: [0x00, 0x00, 0x01, 0x2c],
        },
        Reading {
            sensor: 9,
            raw: [0xff, 0xff, 0xff, 0xf6],
        },
        Reading {
            sensor: 7,
            raw: [0x00, 0x00, 0x01, 0x2d],
        },
    ];

    #[test]
    fn raw_hex_without_allocation() {
        assert_eq!(format!("{}", READINGS[1].raw_hex()), "fffffff6");
    }

    #[cfg(feature = "std")]
    #[test]
    fn distinct_with_std() {
        assert_eq!(distinct_sensors(&READINGS), 2);
    }
}
