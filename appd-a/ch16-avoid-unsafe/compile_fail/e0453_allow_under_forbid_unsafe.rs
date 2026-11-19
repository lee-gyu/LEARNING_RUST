// 기대하는 에러: E0453 (에러 코드 없는 린트 에러 하나가 함께 나옵니다)
//   error[E0453]: allow(unsafe_code) incompatible with previous forbid
//   error: usage of an `unsafe` block
//
// #![forbid(unsafe_code)]는 크레이트 안의 어느 곳도 다시 허락할 수 없는 금지입니다. 그래서 모듈에
// 붙인 #[allow(unsafe_code)]가 E0453으로 거부되고, unsafe 블록도 여전히 에러입니다.
// 한 모듈에만 unsafe를 허락하려면 forbid 대신 deny를 씁니다(예제 04의 모양).
// unsafe가 하나도 필요 없는 크레이트라면 forbid가 맞습니다. 그때는 이 모듈의 unsafe를 지웁니다.
//
// 확인: rustc --edition 2024 compile_fail/e0453_allow_under_forbid_unsafe.rs --out-dir /tmp/er-ch16

#![forbid(unsafe_code)]

#[allow(unsafe_code)] // E0453
mod station {
    /// 줄의 앞 3바이트(열차 종류)를 돌려줍니다.
    pub fn kind(line: &[u8]) -> Option<&str> {
        let head = line.get(..3)?;
        if !head.is_ascii() {
            return None;
        }
        // SAFETY: 바로 위에서 head가 ASCII임을 확인했고, ASCII 바이트열은 올바른 UTF-8입니다.
        Some(unsafe { std::str::from_utf8_unchecked(head) })
    }
}

fn main() {
    println!("{:?}", station::kind(b"KTX0101"));
}
