// 부록 B ch14 · Q14.4, Q14.10 — 할당을 세어 보기
//
// 느린 Rust 프로그램의 흔한 원인 가운데 하나는 눈에 보이지 않는 힙 할당입니다. 이 예제는 전역 할당기
// (#[global_allocator])를 시스템 할당기를 감싼 것으로 바꿔, 할당(alloc)과 재할당(realloc)이 몇 번 일어났는지 셉니다.
// GlobalAlloc 트레이트를 구현하려면 unsafe가 필요하지만, 여기서는 System에 그대로 넘기고 횟수만 셉니다
// (unsafe의 책임은 부록 B ch3, 할당기 자체는 Part 10에서 다룹니다).
//
// 1. 용량을 미리 잡지 않은 Vec은 커질 때마다 재할당합니다. 남는 용량(capacity - len)도 메모리입니다.
// 2. 루프 안의 format!은 반복마다 새 String을 만듭니다. 같은 버퍼에 write!로 이어 쓰면 그 할당이 사라집니다.
// 아래 횟수는 이 툴체인(1.100.0)의 값입니다. Vec이 얼마씩 커지는지는 문서가 정하지 않으므로 다른 버전에서는
// 다를 수 있습니다. 횟수를 세는 구간 안에서는 출력하지 않습니다(출력도 버퍼를 할당할 수 있기 때문).
//
// 실행: cargo run -p faq-ch14-performance --example 02_count_allocations

use std::alloc::{GlobalAlloc, Layout, System};
use std::fmt::Write;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

struct Counting;

static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static REALLOCS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: 모든 요청을 System에 그대로 넘기고, 원자적 카운터만 올립니다.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        // SAFETY: GlobalAlloc::alloc의 계약을 받은 그대로 System에 넘깁니다.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: ptr은 같은 layout으로 System에서 받은 것입니다.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        REALLOCS.fetch_add(1, Relaxed);
        // SAFETY: GlobalAlloc::realloc의 계약을 받은 그대로 System에 넘깁니다.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// 지금까지의 (할당, 재할당) 횟수
fn counts() -> (usize, usize) {
    (ALLOCS.load(Relaxed), REALLOCS.load(Relaxed))
}

/// before 이후에 늘어난 (할당, 재할당) 횟수
fn since(before: (usize, usize)) -> (usize, usize) {
    let now = counts();
    (now.0 - before.0, now.1 - before.1)
}

fn main() {
    let n = black_box(1000);

    let before = counts();
    let mut grown = Vec::new();
    for i in 0..n {
        grown.push(i as u32);
    }
    let push_counts = since(before);

    let before = counts();
    let mut reserved = Vec::with_capacity(n);
    for i in 0..n {
        reserved.push(i as u32);
    }
    let reserved_counts = since(before);

    let before = counts();
    let mut joined = String::new();
    for i in 0..100 {
        joined = joined + &format!("{i},");
    }
    let format_counts = since(before);

    let before = counts();
    let mut written = String::new();
    for i in 0..100 {
        write!(written, "{i},").unwrap();
    }
    let write_counts = since(before);

    println!("1. Vec에 1000개 넣기");
    println!(
        "  Vec::new: 할당 {}, 재할당 {}",
        push_counts.0, push_counts.1
    ); // 출력:   Vec::new: 할당 1, 재할당 8
    println!("  len {}, capacity {}", grown.len(), grown.capacity()); // 출력:   len 1000, capacity 1024
    println!(
        "  Vec::with_capacity: 할당 {}, 재할당 {}",
        reserved_counts.0, reserved_counts.1
    ); // 출력:   Vec::with_capacity: 할당 1, 재할당 0
    println!("2. 문자열 100조각 잇기");
    println!(
        "  + format!: 할당 {}, 재할당 {}",
        format_counts.0, format_counts.1
    ); // 출력:   + format!: 할당 101, 재할당 6
    println!(
        "  write!: 할당 {}, 재할당 {}",
        write_counts.0, write_counts.1
    ); // 출력:   write!: 할당 1, 재할당 6
    println!("  두 문자열이 같은가: {}", joined == written); // 출력:   두 문자열이 같은가: true
}
