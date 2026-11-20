// `tests/`의 파일은 파일마다 따로 컴파일되는 통합 테스트입니다(Part 0 ch11).
//
// 실행: cargo test --test counts

use text_stat::{SAMPLE, count, longest_line};

#[test]
fn sample_counts() {
    let c = count(SAMPLE);
    assert_eq!((c.lines, c.words), (4, 16));
    // 마지막 줄의 조합형 "한글"은 `char` 6개, 자소 클러스터 2개입니다.
    assert_eq!(c.chars - c.graphemes, 4);
}

#[test]
fn longest_line_of_sample() {
    assert_eq!(
        longest_line(SAMPLE),
        Some("cargo run 은 빌드한 뒤 바로 실행합니다.")
    );
}
