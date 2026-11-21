// Q9.1 · Q9.2 — 해제할 책임은 값 하나에 하나뿐이다
//
// C에서는 malloc으로 받은 포인터를 여러 변수가 나눠 가질 수 있고, 누가 free할지는 약속(주석, 문서)으로
// 정합니다. 약속이 어긋나면 이미 해제한 메모리를 쓰거나(compare/01_use_after_free.c) 두 번 해제합니다
// (compare/02_double_free.c). Rust에서는 해제할 책임(소유권)이 값마다 정확히 하나이고, 값을 넘기면
// 책임도 함께 넘어갑니다. 넘겨준 쪽은 그 값을 더 쓸 수 없으므로 해제한 뒤 쓰거나 두 번 해제할 길이 없습니다.
//
//   1. 버퍼를 세 함수에 차례로 넘겨도 해제는 마지막 소유자에서 한 번
//   2. 둘 다 써야 하면 clone: 메모리도 둘, 해제도 둘
//   3. C의 "free(p); p = NULL;" 관용구 → Option::take: 비우면서 해제하고, 다시 비워도 아무 일 없음
//
// 해제가 일어나는 순간은 Drop에서 "해제"를 출력해 확인합니다(Drop 자체는 Part 0 ch14 2.6절).
//
// 실행: cargo run -p faq-ch9-c-memory-errors --example 01_one_owner_one_free

use std::sync::atomic::{AtomicUsize, Ordering};

static FREED: AtomicUsize = AtomicUsize::new(0);

struct Buffer {
    name: &'static str,
    data: Vec<u8>,
}

impl Buffer {
    fn new(name: &'static str, size: usize) -> Buffer {
        Buffer {
            name,
            data: vec![0; size],
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        FREED.fetch_add(1, Ordering::SeqCst);
        println!("    [해제] {} ({}바이트)", self.name, self.data.len());
    }
}

fn freed() -> usize {
    FREED.load(Ordering::SeqCst)
}

fn fill(mut buf: Buffer) -> Buffer {
    buf.data.fill(7);
    buf // 소유권을 돌려줌
}

fn checksum(buf: Buffer) -> u32 {
    buf.data.iter().map(|&b| u32::from(b)).sum()
} // buf의 마지막 소유자는 이 함수이므로 여기서 해제됩니다.

fn main() {
    println!("1. 넘겨 가며 쓰기");
    let buf = Buffer::new("요청 버퍼", 16);
    let buf = fill(buf); // 넘겼다가 돌려받음
    let sum = checksum(buf); // 다시 넘김. 이 줄 뒤로 buf는 쓸 수 없음
    println!("  합계 {sum}, 지금까지 해제 {}번", freed());
    // 출력:     [해제] 요청 버퍼 (16바이트)
    // 출력:   합계 112, 지금까지 해제 1번

    println!("2. clone하면 메모리도 해제도 둘");
    let original = Buffer::new("원본", 4);
    let copy = Buffer {
        name: "사본",
        data: original.data.clone(), // 내용을 새 메모리에 복제
    };
    drop(original);
    drop(copy);
    println!("  지금까지 해제 {}번", freed());
    // 출력:     [해제] 원본 (4바이트)
    // 출력:     [해제] 사본 (4바이트)
    // 출력:   지금까지 해제 3번

    println!("3. free(p); p = NULL; 대신 Option::take");
    let mut slot = Some(Buffer::new("캐시", 8));
    drop(slot.take()); // 꺼내서 해제하고, slot은 None이 됨
    drop(slot.take()); // 이미 None이라 아무것도 해제하지 않음
    println!(
        "  slot은 비었나: {}, 지금까지 해제 {}번",
        slot.is_none(),
        freed()
    );
    // 출력:     [해제] 캐시 (8바이트)
    // 출력:   slot은 비었나: true, 지금까지 해제 4번
}
