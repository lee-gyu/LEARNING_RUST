//! 힙이 제한된 환경을 흉내 냅니다. 바이트 예산을 넘는 요청을 거절하는 할당기를 만들어 `Vec`에 주면,
//! 할당 실패가 결정적으로 일어납니다. 실패할 수 있는 API(`try_reserve`)는 `Err`를 돌려주고,
//! 실패하지 않는 API(`push`)는 프로세스를 중단시킵니다.
//!
//! 실행: cargo run -p er-ch33-no-std --example 05_limited_heap
//!
//! `Allocator` 트레이트와 `Vec::new_in`은 1.100.0에서 안정화되었습니다(책 시점에는 nightly 전용).
//! 중단은 프로세스를 끝내므로, 이 예제는 자기 자신을 자식 프로세스로 다시 실행해 중단을 관찰합니다.
//! 3번 구역의 종료 상태 표시(`signal: 6 (SIGABRT)`)는 macOS(유닉스)에서 본 것입니다.
//!
//! unsafe가 두 곳 있습니다. 둘 다 받은 요청을 `Global`(전역 할당기)에 그대로 넘기기만 합니다.
//! Miri로 검사하지 못했습니다(이 저장소에 nightly가 없음).

use std::alloc::{AllocError, Allocator, Global, Layout};
use std::cell::Cell;
use std::process::Command;
use std::ptr::NonNull;

/// 남은 바이트 수를 세다가, 예산을 넘는 요청은 `AllocError`로 거절하는 할당기입니다.
struct Budget {
    left: Cell<usize>,
}

impl Budget {
    fn new(bytes: usize) -> Self {
        Budget {
            left: Cell::new(bytes),
        }
    }
}

// SAFETY: 블록을 만들고 해제하는 일은 모두 `Global`이 합니다. `Global`이 돌려준 블록을 요청한 크기로
// 잘라서 돌려주므로, 해제할 때 받는 `layout`은 `Global`에 요청했던 것과 같습니다. 블록은 해제될
// 때까지 유효하고, 이 할당기는 블록의 내용을 읽거나 쓰지 않으며, 두 메서드는 패닉하지 않습니다
// (뺄셈은 확인 뒤에만 하고, 덧셈은 앞서 뺀 크기를 되돌리는 것뿐입니다). 크기를 바꾸는 메서드
// (`grow`, `shrink`)는 기본 구현을 쓰며, 기본 구현은 이 두 메서드를 거칩니다.
unsafe impl Allocator for Budget {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let size = layout.size();
        if size > self.left.get() {
            return Err(AllocError);
        }
        let block = Global.allocate(layout)?;
        self.left.set(self.left.get() - size);
        Ok(NonNull::slice_from_raw_parts(block.cast::<u8>(), size))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.left.set(self.left.get() + layout.size());
        // SAFETY: 부르는 쪽은 `ptr`이 이 할당기에서 받은 블록이고 `layout`이 그 블록에 맞는다고
        // 보장합니다. 그 블록은 `allocate`가 같은 `layout`으로 `Global`에서 받은 것입니다.
        unsafe { Global.deallocate(ptr, layout) }
    }
}

const CHILD_ARG: &str = "--abort-child";

fn main() {
    if std::env::args().nth(1).as_deref() == Some(CHILD_ARG) {
        abort_child();
        return;
    }

    // ── 1. 예산 안에서는 보통의 Vec처럼 동작합니다 ─────────────────────────────
    let budget = Budget::new(64); // 64바이트 = u32 16개
    let mut log: Vec<u32, &Budget> = Vec::new_in(&budget);
    let r = log.try_reserve_exact(8);
    println!(
        "1. try_reserve_exact(8) = {r:?}, 남은 예산 {}바이트",
        budget.left.get()
    ); // 출력: 1. try_reserve_exact(8) = Ok(()), 남은 예산 32바이트

    // 확보한 용량 안에서 push하면 할당기를 부르지 않으므로 예산이 줄지 않습니다.
    for reading in [21, 22, 23] {
        log.push(reading);
    }
    println!(
        "   push 3번 뒤 남은 예산 {}바이트, {log:?}",
        budget.left.get()
    ); // 출력:    push 3번 뒤 남은 예산 32바이트, [21, 22, 23]

    // ── 2. 예산을 넘는 요청은 Err로 돌아옵니다 ────────────────────────────────
    match log.try_reserve_exact(100) {
        Ok(()) => println!("2. 예상과 달리 성공"),
        Err(e) => {
            println!("2. Err: {e}"); // 출력: 2. Err: memory allocation failed because the memory allocator returned an error
            println!("   Debug: {e:?}"); // 출력:    Debug: TryReserveError { kind: AllocError { layout: Layout { size: 412, align: 4 (1 << 2) }, non_exhaustive: () } }
        }
    }
    // 요청한 크기는 (len 3 + 100) × 4바이트 = 412바이트입니다. 실패해도 내용과 용량은 그대로입니다.
    println!("   {log:?}, capacity {}", log.capacity()); // 출력:    [21, 22, 23], capacity 8
    drop(log);
    println!("   Vec을 버린 뒤 남은 예산 {}바이트", budget.left.get()); // 출력:    Vec을 버린 뒤 남은 예산 64바이트

    // ── 3. 실패하지 않는 API가 실패하면: 프로세스 중단 ─────────────────────────
    // 자식 프로세스는 같은 예산에서 `push`만으로 용량을 넘깁니다. std를 링크한 프로그램에서는
    // 할당 실패가 패닉이 아니라 "메시지 출력 후 중단"이라서 `catch_unwind`로도 붙잡을 수 없습니다.
    let exe = std::env::current_exe().expect("실행 파일 경로");
    let out = Command::new(exe)
        .arg(CHILD_ARG)
        .env("RUST_BACKTRACE", "0")
        .output()
        .expect("자식 프로세스 실행");
    println!("3. 자식의 종료 상태: {}", out.status); // 출력: 3. 자식의 종료 상태: signal: 6 (SIGABRT)
    println!(
        "   성공? {} / 종료 코드 {:?}",
        out.status.success(),
        out.status.code()
    ); // 출력:    성공? false / 종료 코드 None
    println!(
        "   자식의 표준 출력: {:?}",
        String::from_utf8_lossy(&out.stdout)
    ); // 출력:    자식의 표준 출력: "push 8번 성공\n"
    for line in String::from_utf8_lossy(&out.stderr).lines() {
        println!("   자식의 표준 에러: {line}");
    }
    // 출력:    자식의 표준 에러: memory allocation of 64 bytes failed
    // 출력:    자식의 표준 에러: note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
}

/// 자식 프로세스: 64바이트 예산에서 u32를 계속 push합니다. 용량 8을 넘을 때 16개(64바이트)짜리
/// 새 블록을 요청하는데, 옛 블록(32바이트)이 아직 예산을 쓰고 있어 거절됩니다.
fn abort_child() {
    let budget = Budget::new(64);
    let mut log: Vec<u32, &Budget> = Vec::new_in(&budget);
    for i in 0..8 {
        log.push(i);
    }
    println!("push {}번 성공", log.len());
    log.push(8); // 여기서 중단됩니다.
    println!("이 줄은 출력되지 않습니다");
}
