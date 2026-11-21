// Q2.2 · Q2.4 — 요청이 주장하는 길이를 믿는 버그
//
// 요청은 [길이 2바이트(빅 엔디언), 내용…] 모양이고, 서버는 내용을 "주장한 길이만큼" 그대로
// 돌려줍니다. 2014년의 Heartbleed(OpenSSL, CVE-2014-0160)가 이 모양의 버그였습니다. C 구현은
// 주장한 길이만큼 `memcpy`하면서 실제 내용 뒤의 메모리(개인 키 등)까지 복사해 보냈습니다.
//
// 이 예제에서는 비밀 키를 요청 내용 바로 뒤에 붙여 두고, 요청 부분만 가리키는 슬라이스를 넘깁니다.
//
//   1. 정상 요청: 길이 4, 내용 "ping"
//   2. 길이를 부풀린 요청: 길이 64, 내용 "ping" → 슬라이스 밖이라 거부
//   3. 슬라이스를 잘못 정한 경우: 비밀 키까지 포함한 슬라이스를 넘기면 비밀이 그대로 나감
//
// 실행: cargo run -p faq-ch2-memory-safety --example 02_trusted_length

/// 요청 내용을 주장한 길이만큼 돌려줍니다. 길이가 내용보다 길면 거부합니다.
fn heartbeat_reply(request: &[u8]) -> Result<&[u8], String> {
    let (len_bytes, payload) = request
        .split_first_chunk::<2>()
        .ok_or("헤더가 2바이트보다 짧음")?;
    // 2바이트를 빅 엔디언 정수로 읽습니다(`from_be_bytes`는 Part 1 ch1에서 다루는 바이트 변환).
    let claimed = usize::from(u16::from_be_bytes(*len_bytes));
    // `get`은 범위가 슬라이스 안에 있을 때만 `Some`을 돌려줍니다.
    payload
        .get(..claimed)
        .ok_or_else(|| format!("주장한 길이 {claimed} > 실제 내용 {}", payload.len()))
}

/// 길이 헤더와 내용으로 요청을 만들고, 그 바로 뒤에 비밀 키를 붙인 버퍼를 돌려줍니다.
/// 두 번째 값은 요청 부분의 끝 위치입니다.
fn buffer_with_secret(claimed: u16, content: &[u8]) -> (Vec<u8>, usize) {
    let mut memory = Vec::new();
    memory.extend_from_slice(&claimed.to_be_bytes());
    memory.extend_from_slice(content);
    let request_end = memory.len();
    memory.extend_from_slice(b"SECRET-KEY-1234"); // 같은 버퍼 안, 요청 바로 뒤
    (memory, request_end)
}

// `from_utf8_lossy`는 바이트를 문자열로 바꿔 출력합니다. 올바른 UTF-8이 아닌 부분은 대체 문자(�)가 됩니다.
fn show(label: &str, result: Result<&[u8], String>) {
    match result {
        Ok(reply) => println!("  {label}: 응답 {:?}", String::from_utf8_lossy(reply)),
        Err(e) => println!("  {label}: 거부 ({e})"),
    }
}

fn main() {
    println!("1. 정상 요청");
    let (memory, end) = buffer_with_secret(4, b"ping");
    show("길이 4", heartbeat_reply(&memory[..end]));
    // 출력:   길이 4: 응답 "ping"

    println!("2. 길이를 부풀린 요청");
    let (memory, end) = buffer_with_secret(64, b"ping");
    show("길이 64", heartbeat_reply(&memory[..end]));
    // 출력:   길이 64: 거부 (주장한 길이 64 > 실제 내용 4)
    // 비밀 키는 메모리상 바로 뒤에 있지만, 함수가 받은 슬라이스는 요청 부분의 길이만 압니다.
    // 슬라이스의 길이를 넘는 읽기는 `get`이면 `None`, 인덱싱이면 패닉이 되어(03) 일어나지 않습니다.

    println!("3. 슬라이스를 잘못 정한 경우");
    let (memory, _end) = buffer_with_secret(19, b"ping");
    show("길이 19", heartbeat_reply(&memory)); // 실수: 요청이 아니라 버퍼 전체를 넘김
    // 출력:   길이 19: 응답 "pingSECRET-KEY-1234"
    // Rust가 지키는 경계는 "넘겨받은 슬라이스"입니다. 비밀까지 들어 있는 슬라이스를 넘긴 것은
    // 메모리 오류가 아니라 논리 오류이므로 컴파일러도 실행 중 검사도 막지 않습니다(ch3).
}
