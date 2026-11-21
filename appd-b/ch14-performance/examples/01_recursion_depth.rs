// 부록 B ch14 · Q14.9 — 꼬리 호출 최적화는 보장되지 않는다
//
// count_tail은 마지막에 자기 자신을 부르는 꼬리 재귀(tail recursion)입니다. 최적화를 켜면 LLVM이 이것을 반복문으로
// 바꿔 스택을 쓰지 않지만, 언어가 보장하는 일은 아닙니다. 최적화를 끈 디버그 빌드에서는 호출마다 스택 프레임이
// 쌓입니다.
// count_with_drop은 겉보기에 똑같은 꼬리 호출이지만, Drop을 구현한 지역 값(Frame)이 호출이 끝날 때까지 살아 있습니다.
// 그 값의 drop은 재귀 호출이 돌아온 뒤에 실행되어야 하므로 호출이 꼬리 위치가 아니게 되고, 릴리스에서도 프레임이
// 쌓입니다. Frame의 drop은 전역 카운터를 올리는 부수 효과가 있어 최적화가 없앨 수 없습니다.
//
// 실행(인자 없음, 얕은 깊이라 어느 빌드에서나 종료 상태 0):
//   cargo run -p faq-ch14-performance --example 01_recursion_depth
// 깊은 재귀(스택 256 KiB인 스레드에서 100만 번). 디버그와 릴리스를 각각 실행합니다:
//   cargo run -p faq-ch14-performance --example 01_recursion_depth -- tail
//   cargo run --release -p faq-ch14-performance --example 01_recursion_depth -- tail
//   (tail 자리에 drop, loop를 넣어 같은 두 명령을 실행합니다. --release는 -- 앞에 둡니다.
//    뒤에 두면 프로그램 인자가 되어 디버그 빌드가 실행됩니다)
// 스택 오버플로가 나면 "thread 'recursion' (번호) has overflowed its stack"을 출력하고 프로세스 전체가 종료 상태 134로
// 끝납니다(번호는 실행마다 다름). 어느 조합이 넘치는지는 README Q14.9의 표에 있습니다.

use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::thread;

static DROPS: AtomicUsize = AtomicUsize::new(0);

/// drop될 때 전역 카운터를 올리는 값. 이 부수 효과 때문에 최적화가 drop을 지울 수 없습니다.
struct Frame;

impl Drop for Frame {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Relaxed);
    }
}

fn count_tail(n: u64, acc: u64) -> u64 {
    if n == 0 {
        acc
    } else {
        count_tail(n - 1, acc + 1)
    }
}

fn count_with_drop(n: u64, acc: u64) -> u64 {
    let _frame = Frame; // 재귀 호출이 돌아온 뒤, 이 함수가 끝날 때 drop됨
    if n == 0 {
        acc
    } else {
        count_with_drop(n - 1, acc + 1)
    }
}

fn count_loop(mut n: u64) -> u64 {
    let mut acc = 0;
    while n > 0 {
        n -= 1;
        acc += 1;
    }
    acc
}

fn main() {
    let which = std::env::args().nth(1);
    let Some(which) = which else {
        let n = black_box(1_000);
        println!("얕은 재귀(1000번)");
        println!("  tail: {}", count_tail(n, 0)); // 출력:   tail: 1000
        let result = count_with_drop(n, 0);
        println!("  drop: {result} (Frame drop {}번)", DROPS.load(Relaxed)); // 출력:   drop: 1000 (Frame drop 1001번)
        println!("  loop: {}", count_loop(n)); // 출력:   loop: 1000
        return;
    };

    let n = black_box(1_000_000);
    // 넘치는 지점을 빨리, 매번 같게 보려고 스택이 작은 스레드에서 돌립니다(메인 스레드의 스택은 더 큽니다).
    let handle = thread::Builder::new()
        .name(String::from("recursion"))
        .stack_size(256 * 1024)
        .spawn(move || match which.as_str() {
            "tail" => count_tail(n, 0),
            "drop" => count_with_drop(n, 0),
            _ => count_loop(n),
        })
        .unwrap();
    println!("깊은 재귀(100만 번): {}", handle.join().unwrap());
}
