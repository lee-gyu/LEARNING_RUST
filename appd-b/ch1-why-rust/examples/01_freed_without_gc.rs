// Q1.1 · Q1.8 — GC 없이, 정해진 때 해제되는 메모리
//
// 1 MB짜리 버퍼를 하나씩 읽어 처리하는 일을 네 번 반복합니다. 버퍼마다 "지금 살아 있는
// 버퍼 수"를 세어, 해제가 언제 일어나는지 출력으로 확인합니다.
//
// 실행: cargo run -p faq-ch1-why-rust --example 01_freed_without_gc

use std::sync::atomic::{AtomicUsize, Ordering};

// 지금 살아 있는 버퍼 수와 그 최댓값. 어디서든 고칠 수 있는 전역 카운터가 필요해서
// 원자 타입(Part 1 ch11)을 썼습니다. 이 예제는 스레드 하나로 돌므로 그 이상의 뜻은 없습니다.
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn live() -> usize {
    LIVE.load(Ordering::SeqCst)
}

/// 처리할 데이터 한 덩어리. 힙에 있는 `Vec<u8>`을 소유합니다.
struct Batch {
    id: u32,
    data: Vec<u8>,
}

impl Batch {
    fn load(id: u32, size: usize) -> Batch {
        let now = LIVE.fetch_add(1, Ordering::SeqCst) + 1;
        PEAK.fetch_max(now, Ordering::SeqCst);
        Batch {
            id,
            data: vec![1; size],
        }
    }
}

// 값이 버려질 때(drop) 실행됩니다. `Vec`의 힙 메모리는 이 함수가 끝난 직후 해제됩니다.
impl Drop for Batch {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
        println!("  배치 {} 해제 ({} 바이트)", self.id, self.data.len());
    }
}

fn process(id: u32) -> Result<u64, String> {
    let batch = Batch::load(id, 1_000_000);
    println!("  배치 {id} 처리 중: 살아 있는 버퍼 {}개", live());

    if id == 3 {
        // 일찍 돌아가는 길에서도 `batch`는 이 함수의 스코프를 벗어나며 버려집니다.
        // C라면 이 `return` 앞에 `free`를 빠뜨리지 않았는지 사람이 확인해야 합니다.
        return Err(format!("배치 {id}의 형식이 잘못됨"));
    }

    let sum = batch.data.iter().map(|&b| u64::from(b)).sum();
    Ok(sum)
    // 여기서 `batch`가 버려집니다. 반환값은 이미 계산해 두었으므로 쓰는 데 문제가 없습니다.
}

fn main() {
    for id in 1..=4 {
        match process(id) {
            Ok(sum) => println!("배치 {id} 합계 {sum}, 살아 있는 버퍼 {}개", live()),
            Err(e) => println!("배치 {id} 실패: {e}, 살아 있는 버퍼 {}개", live()),
        }
    }
    println!(
        "동시에 살아 있던 버퍼는 최대 {}개",
        PEAK.load(Ordering::SeqCst)
    );

    // 출력:
    //   배치 1 처리 중: 살아 있는 버퍼 1개
    //   배치 1 해제 (1000000 바이트)
    // 배치 1 합계 1000000, 살아 있는 버퍼 0개
    //   배치 2 처리 중: 살아 있는 버퍼 1개
    //   배치 2 해제 (1000000 바이트)
    // 배치 2 합계 1000000, 살아 있는 버퍼 0개
    //   배치 3 처리 중: 살아 있는 버퍼 1개
    //   배치 3 해제 (1000000 바이트)
    // 배치 3 실패: 배치 3의 형식이 잘못됨, 살아 있는 버퍼 0개
    //   배치 4 처리 중: 살아 있는 버퍼 1개
    //   배치 4 해제 (1000000 바이트)
    // 배치 4 합계 1000000, 살아 있는 버퍼 0개
    // 동시에 살아 있던 버퍼는 최대 1개
}
