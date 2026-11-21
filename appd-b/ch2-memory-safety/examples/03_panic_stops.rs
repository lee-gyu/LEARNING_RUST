// Q2.6 — 범위를 벗어난 인덱싱은 패닉으로 멈춘다
//
// 02와 같은 응답 함수를 `get` 대신 인덱싱(`[..claimed]`)으로 씁니다. 요청 세 개를 차례로
// 처리하는데, 두 번째 요청이 길이를 부풀렸습니다. 인덱싱은 범위를 검사하다 패닉을 내고,
// 패닉은 스택을 되감으며(unwinding) 지역 변수를 버린 뒤 프로그램을 끝냅니다.
//
// 이 예제는 일부러 패닉으로 끝납니다. 종료 코드는 101입니다.
// 표준 에러의 패닉 메시지는 빈 줄로 시작하고, 그다음 줄의 괄호 안 스레드 번호는 실행마다 다릅니다.
//
// 실행: cargo run -p faq-ch2-memory-safety --example 03_panic_stops

/// 요청 하나를 처리하는 동안 살아 있는 연결. 버려질 때 정리 메시지를 출력합니다.
struct Connection {
    id: u32,
}

impl Drop for Connection {
    fn drop(&mut self) {
        println!("  연결 {} 정리", self.id);
    }
}

/// 02의 `heartbeat_reply`와 같지만 범위를 `get` 대신 인덱싱으로 자릅니다.
fn heartbeat_reply(request: &[u8]) -> &[u8] {
    let claimed = usize::from(u16::from_be_bytes([request[0], request[1]]));
    let payload = &request[2..];
    &payload[..claimed] // claimed가 payload.len()보다 크면 여기서 패닉
}

fn handle(id: u32, request: &[u8]) {
    let _conn = Connection { id };
    println!("요청 {id} 처리 시작");
    let reply = heartbeat_reply(request);
    println!("  응답 {:?}", String::from_utf8_lossy(reply));
}

fn main() {
    let requests: [&[u8]; 3] = [b"\x00\x04ping", b"\x00\x40ping", b"\x00\x04pong"];
    for (i, request) in (1..).zip(requests) {
        handle(i, request);
    }
    println!("모든 요청 처리 끝"); // 두 번째 요청에서 패닉하므로 실행되지 않습니다.

    // 표준 출력:
    // 요청 1 처리 시작
    //   응답 "ping"
    //   연결 1 정리
    // 요청 2 처리 시작
    //   연결 2 정리
    //
    // 표준 에러(빈 줄로 시작하고, 스레드 번호는 실행마다 다름):
    //
    // thread 'main' (스레드 번호) panicked at appd-b/ch2-memory-safety/examples/03_panic_stops.rs:27:13:
    // range end index 64 out of range for slice of length 4
    // note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
}
