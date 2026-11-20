//! 실패할 수 있는 할당: `try_reserve`/`try_reserve_exact`는 할당 실패를 `Result`로 돌려줍니다.
//! 같은 일을 하는 `reserve`는 실패를 알릴 길이 없어 패닉합니다.
//!
//! 실행: cargo run -p er-ch33-no-std --example 04_try_reserve
//!
//! 실패를 결정적으로 일으키려고 용량 오버플로(필요한 바이트 수가 `isize::MAX`를 넘는 요청)를 씁니다.
//! 할당기가 실제로 메모리를 거절하는 경우는 05에서 흉내 냅니다.

use std::collections::TryReserveError;

/// 패킷 머리에 적힌 샘플 수만큼 버퍼를 미리 늘립니다. 그 수는 밖에서 온 값이라 터무니없이 클 수 있습니다.
fn prepare(buf: &mut Vec<u16>, announced: usize) -> Result<(), TryReserveError> {
    buf.try_reserve_exact(announced)
}

/// 바이트 쌍을 샘플로 바꿔 붙입니다. 필요한 용량을 먼저 확보해 두고, 그 안에서만 `push`합니다.
/// 실패할 수 있는 `push`가 stable에 없어서(1.100.0에서 `push_within_capacity`는 nightly 전용) 쓰는 우회입니다.
fn append_samples(buf: &mut Vec<u16>, bytes: &[u8]) -> Result<(), TryReserveError> {
    buf.try_reserve_exact(bytes.len() / 2)?;
    let (pairs, _rest) = bytes.as_chunks::<2>(); // 홀수 바이트가 남으면 버립니다
    for pair in pairs {
        buf.push(u16::from_le_bytes(*pair)); // 용량 안이므로 재할당하지 않음
    }
    Ok(())
}

fn main() {
    // ── 1. 성공하면 용량이 늘어납니다 ───────────────────────────────────────
    let mut buf: Vec<u16> = vec![100, 101, 102];
    let ok = prepare(&mut buf, 5);
    // 문서가 약속하는 것은 `capacity >= len + additional`뿐입니다. 8은 이 툴체인에서 본 값입니다.
    println!(
        "1. prepare(5) = {ok:?}, len {}, capacity {}",
        buf.len(),
        buf.capacity()
    ); // 출력: 1. prepare(5) = Ok(()), len 3, capacity 8

    // ── 2. 실패하면 Err를 돌려주고 내용은 그대로 둡니다 ─────────────────────────
    // u16 `usize::MAX / 2`개는 `isize::MAX` 바이트를 넘으므로 할당기에 묻기 전에 실패합니다.
    match prepare(&mut buf, usize::MAX / 2) {
        Ok(()) => println!("2. 예상과 달리 성공"),
        Err(e) => {
            println!("2. Err: {e}"); // 출력: 2. Err: memory allocation failed because the computed capacity exceeded the collection's maximum
            println!("   Debug: {e:?}"); // 출력:    Debug: TryReserveError { kind: CapacityOverflow }
        }
    }
    println!("   버퍼는 그대로: {buf:?}, capacity {}", buf.capacity()); // 출력:    버퍼는 그대로: [100, 101, 102], capacity 8

    // ── 3. 실패하지 않는 판은 같은 요청에서 패닉합니다 ─────────────────────────
    // 기본 패닉 훅의 메시지에는 실행마다 다른 스레드 번호가 들어가므로 훅을 잠시 비우고 페이로드만 봅니다.
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(|| {
        let mut v: Vec<u16> = vec![100, 101, 102];
        v.reserve(usize::MAX / 2);
    });
    let _ = std::panic::take_hook();
    match caught {
        Ok(()) => println!("3. 예상과 달리 패닉하지 않음"),
        Err(payload) => println!(
            "3. reserve는 패닉: {:?}",
            payload
                .downcast_ref::<&str>()
                .copied()
                .unwrap_or("(문자열이 아닌 페이로드)")
        ), // 출력: 3. reserve는 패닉: "capacity overflow"
    }

    // ── 4. 미리 확보하고 그 안에서만 push ─────────────────────────────────────
    let mut samples: Vec<u16> = Vec::new();
    let packet = [0x10, 0x00, 0x20, 0x00, 0x30, 0x00, 0x40, 0x00];
    match append_samples(&mut samples, &packet) {
        Ok(()) => println!("4. {samples:?}, capacity {}", samples.capacity()), // 출력: 4. [16, 32, 48, 64], capacity 4
        Err(e) => println!("4. Err: {e}"),
    }
}
