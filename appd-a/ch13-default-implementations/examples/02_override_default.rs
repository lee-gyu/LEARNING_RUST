// 지침 2: 기본 메서드는 말 그대로 "기본"입니다. 구현하는 타입이 더 나은 방법을 알면 재정의합니다.
//         (책에 없는 지침) 재정의한 메서드는 기본 구현과 같은 답을 내야 하고, 그것을 확인합니다.
// 비교: 1000m부터 1000m마다 놓인 거리 표지 42개
//   기본 구현에 맡긴 판:  count, last, next_from이 at을 처음부터 차례로 부릅니다.
//   재정의한 판:          간격을 아는 타입이 계산으로 바로 답합니다(at 호출 0번).
//   잘못 재정의한 판:     빠르지만 표지 바로 위에서 기본 구현과 다른 답을 냅니다.
//
// 일의 양은 시간 대신 at이 불린 횟수로 셉니다. 횟수는 실행마다 같습니다.
//
// 실행: cargo run -p er-ch13-default-implementations --example 02_override_default

use std::cell::Cell;

trait Checkpoints {
    /// index번째 지점의 위치(출발점에서 m). 지점이 없으면 None.
    /// index가 커질수록 위치도 커져야 하고, 중간에 None이 끼면 안 됩니다.
    fn at(&self, index: usize) -> Option<u32>;

    /// 지점 수. at을 None이 나올 때까지 부릅니다.
    fn count(&self) -> usize {
        (0..).map_while(|i| self.at(i)).count()
    }
    /// 마지막 지점의 위치. count를 구한 뒤 at을 한 번 더 부릅니다.
    fn last(&self) -> Option<u32> {
        self.count().checked_sub(1).and_then(|i| self.at(i))
    }
    /// pos에서 앞으로 만날 첫 지점(pos에 있는 지점도 포함). 앞에서부터 차례로 봅니다.
    fn next_from(&self, pos: u32) -> Option<u32> {
        (0..).map_while(|i| self.at(i)).find(|&p| p >= pos)
    }
}

/// 같은 간격으로 놓인 지점: first부터 every마다 n곳.
/// get이 불린 횟수를 세어 둡니다(Cell은 &self로도 값을 바꾸기 위해 씀).
struct Spacing {
    first: u32,
    every: u32,
    n: usize,
    calls: Cell<usize>,
}

impl Spacing {
    fn new(first: u32, every: u32, n: usize) -> Spacing {
        Spacing {
            first,
            every,
            n,
            calls: Cell::new(0),
        }
    }
    /// at이 쓰는 조회. 부를 때마다 횟수를 셉니다.
    fn get(&self, index: usize) -> Option<u32> {
        self.calls.set(self.calls.get() + 1);
        if index < self.n {
            Some(self.position(index))
        } else {
            None
        }
    }
    /// 범위 검사도, 횟수 세기도 없는 계산. 재정의한 메서드가 씁니다.
    fn position(&self, index: usize) -> u32 {
        self.first + self.every * index as u32
    }
    /// 지금까지 센 횟수를 돌려주고 0으로 되돌립니다.
    fn take_calls(&self) -> usize {
        self.calls.replace(0)
    }
}

// 1) 필수 메서드만 구현하고 나머지는 기본 구현에 맡긴 판
struct PlainMarkers(Spacing);

impl Checkpoints for PlainMarkers {
    fn at(&self, index: usize) -> Option<u32> {
        self.0.get(index)
    }
}

// 2) 간격을 아는 타입이 세 메서드를 계산으로 재정의한 판
struct FastMarkers(Spacing);

impl Checkpoints for FastMarkers {
    fn at(&self, index: usize) -> Option<u32> {
        self.0.get(index)
    }
    fn count(&self) -> usize {
        self.0.n
    }
    fn last(&self) -> Option<u32> {
        self.0.n.checked_sub(1).map(|i| self.0.position(i))
    }
    fn next_from(&self, pos: u32) -> Option<u32> {
        let s = &self.0;
        // pos 이상인 첫 지점의 번호 = (pos - first) / every를 올림한 값
        let index = if pos <= s.first {
            0
        } else {
            (pos - s.first).div_ceil(s.every) as usize
        };
        if index < s.n {
            Some(s.position(index))
        } else {
            None
        }
    }
}

// 3) 빠르지만 잘못 재정의한 판: 올림 대신 "내림 + 1"을 썼습니다.
//    나누어떨어질 때(표지 바로 위)만 하나를 건너뛰므로, 몇 개 위치를 손으로 시험해서는 놓치기 쉽습니다.
struct SloppyMarkers(Spacing);

impl Checkpoints for SloppyMarkers {
    fn at(&self, index: usize) -> Option<u32> {
        self.0.get(index)
    }
    fn next_from(&self, pos: u32) -> Option<u32> {
        let s = &self.0;
        let index = if pos <= s.first {
            0
        } else {
            ((pos - s.first) / s.every + 1) as usize
        };
        if index < s.n {
            Some(s.position(index))
        } else {
            None
        }
    }
}

/// 재정의한 구현이 기본 구현과 같은 답을 내는지 0m부터 45000m까지 100m 간격으로 비교합니다.
/// reference에는 next_from을 재정의하지 않은(기본 구현을 쓰는) 값을 넘깁니다.
fn mismatches(reference: &impl Checkpoints, candidate: &impl Checkpoints) -> Vec<u32> {
    (0..=45_000)
        .step_by(100)
        .filter(|&pos| reference.next_from(pos) != candidate.next_from(pos))
        .collect()
}

fn report(label: &str, value: String, calls: usize) {
    println!("  {label} = {value} (at 호출 {calls}번)");
}

fn main() {
    let plain = PlainMarkers(Spacing::new(1000, 1000, 42));
    let fast = FastMarkers(Spacing::new(1000, 1000, 42));

    println!("1. 기본 구현에 맡긴 판: at을 처음부터 차례로 부릅니다");
    let v = plain.count();
    report("count()", format!("{v}"), plain.0.take_calls()); // 출력:   count() = 42 (at 호출 43번)
    let v = plain.last();
    report("last()", format!("{v:?}"), plain.0.take_calls()); // 출력:   last() = Some(42000) (at 호출 44번)
    let v = plain.next_from(37_500);
    report("next_from(37500)", format!("{v:?}"), plain.0.take_calls()); // 출력:   next_from(37500) = Some(38000) (at 호출 38번)

    println!("2. 재정의한 판: 같은 답을 계산으로 바로 냅니다");
    let v = fast.count();
    report("count()", format!("{v}"), fast.0.take_calls()); // 출력:   count() = 42 (at 호출 0번)
    let v = fast.last();
    report("last()", format!("{v:?}"), fast.0.take_calls()); // 출력:   last() = Some(42000) (at 호출 0번)
    let v = fast.next_from(37_500);
    report("next_from(37500)", format!("{v:?}"), fast.0.take_calls()); // 출력:   next_from(37500) = Some(38000) (at 호출 0번)

    println!("3. 재정의한 next_from이 기본 구현과 같은 답을 내는지 451곳에서 비교합니다");
    let sloppy = SloppyMarkers(Spacing::new(1000, 1000, 42));
    let ok = mismatches(&plain, &fast);
    println!("  재정의한 판: 다른 곳 {}곳", ok.len()); // 출력:   재정의한 판: 다른 곳 0곳
    let bad = mismatches(&plain, &sloppy);
    println!(
        "  잘못 재정의한 판: 다른 곳 {}곳, 처음 세 곳 {:?}",
        bad.len(),
        &bad[..3]
    ); // 출력:   잘못 재정의한 판: 다른 곳 41곳, 처음 세 곳 [2000, 3000, 4000]
    println!(
        "  next_from(2000): 기본 구현 {:?}, 잘못 재정의한 판 {:?}",
        plain.next_from(2000),
        sloppy.next_from(2000)
    ); // 출력:   next_from(2000): 기본 구현 Some(2000), 잘못 재정의한 판 Some(3000)
}
