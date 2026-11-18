// 이 예제가 보여 주는 것: BinaryHeap — 언제나 "가장 큰 것"을 O(log n)에 꺼내 주는 우선순위 큐.
//                        최대 힙의 기본 동작, Reverse로 만드는 최소 힙,
//                        derive(Ord)한 구조체로 만드는 작업 스케줄러, 가장 큰 k개(top-k) 고르기
// 실행: cargo run -p p1-ch4-collections --example 08_binary_heap
//
// "이것이 아니면 곤란한 상황": 요소가 계속 들어오고 나가는 중에 "지금 가장 중요한 것"을 반복해서 꺼낼 때입니다.
// 매번 정렬하면 O(n log n), 매번 최댓값을 찾아 훑으면 O(n)이 들지만,
// BinaryHeap은 넣기와 꺼내기가 O(log n), 들여다보기(peek)가 O(1)입니다.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

// 작업 하나. derive(Ord)는 필드를 선언 순서대로 비교하므로
// 1) priority가 큰 작업이 크고, 2) priority가 같으면 Reverse(seq) 덕분에 seq가 작은(먼저 들어온) 작업이 큽니다.
// BinaryHeap은 가장 큰 것부터 꺼내므로 "우선순위 높은 것 먼저, 같으면 먼저 온 것 먼저"가 됩니다.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Job {
    priority: u8,
    seq: Reverse<u32>,
    name: &'static str,
}

struct Scheduler {
    heap: BinaryHeap<Job>,
    next_seq: u32,
}

impl Scheduler {
    fn new() -> Self {
        Scheduler {
            heap: BinaryHeap::new(),
            next_seq: 0,
        }
    }

    fn submit(&mut self, priority: u8, name: &'static str) {
        self.heap.push(Job {
            priority,
            seq: Reverse(self.next_seq),
            name,
        });
        self.next_seq += 1;
    }

    fn run_next(&mut self) {
        match self.heap.pop() {
            Some(job) => println!(
                "실행: {} (우선순위 {}, 접수 {}번)",
                job.name, job.priority, job.seq.0
            ),
            None => println!("대기 중인 작업이 없습니다"),
        }
    }
}

fn main() {
    // 1. 최대 힙: pop은 항상 가장 큰 값을 꺼냅니다.
    println!("== 1. 최대 힙 ==");
    let mut heap = BinaryHeap::new();
    for n in [3, 9, 1, 7, 5] {
        heap.push(n);
    }
    println!("peek = {:?}, len = {}", heap.peek(), heap.len()); // 출력: peek = Some(9), len = 5
    // 내부 배열은 "부모가 자식보다 크다"는 조건만 지킬 뿐 정렬되어 있지 않습니다.
    // 아래 순서는 이 툴체인에서의 결과이며, 정렬되어 있지 않다는 것만 확인하면 됩니다.
    println!("내부 순서: {:?}", heap.as_slice()); // 출력: 내부 순서: [9, 7, 1, 3, 5]
    let mut popped = Vec::new();
    while let Some(top) = heap.pop() {
        popped.push(top);
    }
    println!("pop 순서: {popped:?}"); // 출력: pop 순서: [9, 7, 5, 3, 1]
    // 모두 꺼내 오름차순 Vec으로 받고 싶다면 into_sorted_vec을 씁니다.
    let sorted = BinaryHeap::from([4, 8, 2, 6]).into_sorted_vec();
    println!("into_sorted_vec: {sorted:?}"); // 출력: into_sorted_vec: [2, 4, 6, 8]

    // 2. 최소 힙: Reverse로 감싸면 비교가 뒤집혀 가장 작은 값이 먼저 나옵니다.
    println!("\n== 2. Reverse로 최소 힙 ==");
    let mut events = BinaryHeap::new();
    events.push(Reverse((930, "회의")));
    events.push(Reverse((815, "출근")));
    events.push(Reverse((1200, "점심")));
    events.push(Reverse((900, "메일 확인")));
    // 튜플은 앞 요소부터 비교하므로 시각(정수)이 이른 순서로 나옵니다. 패턴으로 Reverse를 벗깁니다.
    while let Some(Reverse((time, what))) = events.pop() {
        println!("{:02}:{:02} {what}", time / 100, time % 100);
    }
    // 출력:
    // 08:15 출근
    // 09:00 메일 확인
    // 09:30 회의
    // 12:00 점심

    // 3. 작업 스케줄러: 실행하는 도중에도 새 작업이 들어옵니다.
    //    한 번 정렬하고 끝나는 문제라면 sort로 충분하지만, 들어오고 나가는 일이 섞이면 힙이 알맞습니다.
    println!("\n== 3. 작업 스케줄러 ==");
    let mut scheduler = Scheduler::new();
    scheduler.submit(1, "로그 정리");
    scheduler.submit(3, "결제 처리");
    scheduler.submit(1, "캐시 갱신");
    scheduler.submit(2, "메일 발송");
    scheduler.run_next();
    scheduler.run_next();
    scheduler.submit(5, "장애 대응"); // 도중에 들어온 급한 작업
    scheduler.run_next();
    scheduler.run_next();
    scheduler.run_next();
    scheduler.run_next();
    // 출력:
    // 실행: 결제 처리 (우선순위 3, 접수 1번)
    // 실행: 메일 발송 (우선순위 2, 접수 3번)
    // 실행: 장애 대응 (우선순위 5, 접수 4번)
    // 실행: 로그 정리 (우선순위 1, 접수 0번)
    // 실행: 캐시 갱신 (우선순위 1, 접수 2번)
    // 대기 중인 작업이 없습니다

    // 4. 가장 큰 k개 고르기(top-k): 크기가 k인 최소 힙을 유지합니다.
    //    힙의 맨 위(peek)는 "지금까지의 상위 k개 중 가장 작은 값"이므로, 넘치면 그것을 버립니다.
    //    전체를 정렬하면 O(n log n)과 n개 분량의 메모리가 들지만, 이 방법은 O(n log k)와 k개 분량이면 됩니다.
    println!("\n== 4. 가장 큰 3개 고르기 ==");
    let scores = [72, 95, 68, 88, 91, 59, 98, 84];
    let k = 3;
    let mut top = BinaryHeap::with_capacity(k + 1);
    for s in scores {
        top.push(Reverse(s));
        if top.len() > k {
            top.pop(); // 가장 작은 값을 버립니다.
        }
    }
    // Reverse의 오름차순은 원래 값의 내림차순입니다.
    let best: Vec<u32> = top
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse(s)| s)
        .collect();
    println!("상위 {k}개: {best:?}"); // 출력: 상위 3개: [98, 95, 91]
}
