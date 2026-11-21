//! 인자로 받은 문자열의 Fletcher-16 체크섬을 C 라이브러리로 구합니다.
//!
//! 실행: cargo run -q -- [문자열]     (생략하면 "abcde")

fn main() {
    let text = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "abcde".to_string());
    let sum = fletcher::checksum(text.as_bytes());
    println!("C 라이브러리 버전: {}", fletcher::C_VERSION);
    println!("fletcher16({text:?}) = {sum} (0x{sum:04X})");

    // fletcher_v2는 fletcher의 빌드 스크립트가 켠 cfg입니다. 이 바이너리도 같은 패키지라 보입니다.
    #[cfg(fletcher_v2)]
    println!("C로 다시 검증: {}", fletcher::verify(text.as_bytes(), sum));
    #[cfg(not(fletcher_v2))]
    println!("C로 다시 검증: (버전 2 미만이라 없음)");
}
