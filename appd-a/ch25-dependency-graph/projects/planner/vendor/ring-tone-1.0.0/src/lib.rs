//! 연습용 ring-tone 1.0.0: chime-sys 0.2를 써서 알림음을 낸다고 칩니다.

/// 알림음 이름과 링크한 chime-sys 사본의 버전을 돌려줍니다.
pub fn tone() -> String {
    format!("딩동 (chime-sys {})", chime_sys::VERSION)
}
