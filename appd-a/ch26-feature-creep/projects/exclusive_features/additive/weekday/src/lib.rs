//! 그레고리력 날짜의 요일을 구하는 작은 크레이트.
//!
//! datefmt의 `korean` 기능이 켜졌을 때만 빌드되는 선택적 의존성입니다. 실제 프로젝트라면 지역화
//! 표나 달력 계산처럼 크고 무거운 의존성이 들어갈 자리를 흉내 냅니다.

const KOREAN_SHORT: [&str; 7] = ["일", "월", "화", "수", "목", "금", "토"];

/// 요일 번호(일요일 = 0)를 구합니다. 사카모토(Sakamoto)의 방법입니다.
pub fn index(year: u16, month: u8, day: u8) -> usize {
    const OFFSET: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    // 1월과 2월은 앞 해의 13월, 14월처럼 셉니다.
    let y = u32::from(year) - u32::from(month < 3);
    let m = usize::from(month - 1);
    let n = y + y / 4 - y / 100 + y / 400 + OFFSET[m] + u32::from(day);
    (n % 7) as usize
}

/// 요일의 한 글자 한국어 이름("목" 등)을 돌려줍니다.
pub fn korean_short(year: u16, month: u8, day: u8) -> &'static str {
    KOREAN_SHORT[index(year, month, day)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_days() {
        assert_eq!(korean_short(2026, 11, 19), "목");
        assert_eq!(korean_short(2024, 2, 29), "목");
        assert_eq!(korean_short(2000, 1, 1), "토");
    }
}
