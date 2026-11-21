//! 라이브러리를 바깥(사용자) 자리에서 쓰는 통합 테스트입니다.

use linestat::{Stats, count};

#[test]
fn counts_a_small_text() {
    let text = "first line\nsecond line here\n";
    assert_eq!(
        count(text),
        Stats {
            lines: 2,
            words: 5,
            chars: 28
        }
    );
}
