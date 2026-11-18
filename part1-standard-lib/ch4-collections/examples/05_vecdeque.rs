// 이 예제가 보여 주는 것: VecDeque — 양쪽 끝에서 넣고 빼는 일이 모두 빠른 덱(deque).
//                        큐(먼저 들어온 것이 먼저 나감), 최근 N개 유지, 인덱스 접근과 링 버퍼,
//                        Vec::remove(0)으로 큐를 흉내 냈을 때와의 시간 비교
// 실행: cargo run -p p1-ch4-collections --example 05_vecdeque
//
// "이것이 아니면 곤란한 상황": 앞에서 꺼내고 뒤에 넣는 일을 반복할 때입니다.
// Vec에서 앞을 꺼내는 remove(0)은 나머지를 모두 한 칸씩 당기므로 O(n)이지만,
// VecDeque의 pop_front는 O(1)입니다.

use std::collections::VecDeque;
use std::time::Instant;

fn main() {
    // 1. 큐: push_back으로 줄 서고, pop_front로 앞사람부터 처리합니다.
    println!("== 1. 큐 (push_back / pop_front) ==");
    let mut waiting: VecDeque<&str> = VecDeque::new();
    waiting.push_back("1번 손님");
    waiting.push_back("2번 손님");
    waiting.push_back("3번 손님");
    // 맨 앞과 맨 뒤는 꺼내지 않고 들여다볼 수도 있습니다. 비어 있을 수 있으므로 Option입니다.
    println!("맨 앞: {:?}, 맨 뒤: {:?}", waiting.front(), waiting.back()); // 출력: 맨 앞: Some("1번 손님"), 맨 뒤: Some("3번 손님")
    while let Some(customer) = waiting.pop_front() {
        println!("{customer} 처리, 남은 대기 {}명", waiting.len());
    }
    // 출력:
    // 1번 손님 처리, 남은 대기 2명
    // 2번 손님 처리, 남은 대기 1명
    // 3번 손님 처리, 남은 대기 0명

    // 2. 양쪽 끝 모두: 앞에 끼워 넣기(push_front), 뒤에서 빼기(pop_back), 돌리기(rotate_left)
    println!("\n== 2. 양쪽 끝 다루기 ==");
    let mut line = VecDeque::from(["가온", "나래", "다솜"]);
    line.push_front("응급 환자"); // 맨 앞에 넣어도 다른 요소를 밀지 않습니다.
    println!("앞에 추가: {line:?}"); // 출력: 앞에 추가: ["응급 환자", "가온", "나래", "다솜"]
    let left = line.pop_back(); // 맨 뒤 사람이 기다리다 떠났습니다.
    println!("뒤에서 빼기: {left:?}, {line:?}"); // 출력: 뒤에서 빼기: Some("다솜"), ["응급 환자", "가온", "나래"]
    // 당번 순서 돌리기: rotate_left(1)은 맨 앞 요소를 맨 뒤로 보냅니다.
    let mut duty = VecDeque::from(["월", "화", "수", "목"]);
    duty.rotate_left(1);
    println!("rotate_left(1): {duty:?}"); // 출력: rotate_left(1): ["화", "수", "목", "월"]

    // 3. 최근 N개만 유지하기: 뒤에 넣고, 넘치면 앞에서 버립니다.
    println!("\n== 3. 최근 3개의 평균 (이동 평균) ==");
    let temps = [21.0, 23.5, 22.0, 26.5, 25.0];
    let mut recent: VecDeque<f64> = VecDeque::with_capacity(3);
    for t in temps {
        recent.push_back(t);
        if recent.len() > 3 {
            recent.pop_front();
        }
        let avg = recent.iter().sum::<f64>() / recent.len() as f64;
        println!("{t:>4.1} 입력 → 최근 {recent:?}, 평균 {avg:.2}");
    }
    // 출력:
    // 21.0 입력 → 최근 [21.0], 평균 21.00
    // 23.5 입력 → 최근 [21.0, 23.5], 평균 22.25
    // 22.0 입력 → 최근 [21.0, 23.5, 22.0], 평균 22.17
    // 26.5 입력 → 최근 [23.5, 22.0, 26.5], 평균 24.00
    // 25.0 입력 → 최근 [22.0, 26.5, 25.0], 평균 24.50

    // 4. 인덱스 접근과 링 버퍼
    // VecDeque는 고정된 크기의 배열을 고리처럼 이어 쓰는 링 버퍼(ring buffer)입니다.
    // 앞에서 빼고 뒤에 넣다 보면 요소가 배열 끝에서 처음으로 넘어가 메모리상 두 조각이 됩니다.
    println!("\n== 4. 인덱스 접근과 링 버퍼 ==");
    let mut ring: VecDeque<i32> = VecDeque::with_capacity(4);
    ring.extend([10, 20, 30]);
    ring.pop_front();
    ring.pop_front();
    ring.extend([40, 50, 60]);
    // 인덱스는 언제나 "논리적인 앞"에서부터 셉니다. 메모리 배치와 상관없습니다.
    println!(
        "ring = {ring:?}, ring[0] = {}, get(9) = {:?}",
        ring[0],
        ring.get(9)
    );
    // 출력: ring = [30, 40, 50, 60], ring[0] = 30, get(9) = None
    // as_slices는 메모리상의 두 조각을 그대로 보여 줍니다. 어떻게 나뉘는지는 구현에 달린 일이며,
    // 아래 값은 이 툴체인에서의 결과입니다.
    println!("as_slices = {:?}", ring.as_slices()); // 출력: as_slices = ([30, 40], [50, 60])
    // 그래서 VecDeque에는 슬라이스 메서드인 sort가 없습니다(compile_fail/e0599_vecdeque_sort.rs).
    // make_contiguous()로 요소를 한 조각으로 모으면 &mut [T]를 얻어 정렬할 수 있습니다.
    ring.push_front(45);
    ring.make_contiguous().sort();
    println!("정렬 후 = {ring:?}, as_slices = {:?}", ring.as_slices());
    // 출력: 정렬 후 = [30, 40, 45, 50, 60], as_slices = ([30, 40, 45, 50, 60], [])

    // 5. Vec::remove(0) 대 VecDeque::pop_front
    // 같은 개수의 요소를 앞에서부터 모두 꺼내는 시간을 잽니다. 시간은 컴퓨터와 실행할 때마다 달라집니다.
    // 요소 수가 2배가 되면 Vec 쪽은 약 4배(O(n)짜리를 n번: O(n²)), VecDeque 쪽은 약 2배(O(n))로 늘어나는 경향을 보세요.
    println!("\n== 5. 앞에서 꺼내기: Vec::remove(0) 대 VecDeque::pop_front ==");
    for n in [25_000u64, 50_000] {
        let mut v: Vec<u64> = (0..n).collect();
        let start = Instant::now();
        let mut sum_v = 0;
        while !v.is_empty() {
            sum_v += v.remove(0);
        }
        let vec_time = start.elapsed();

        let mut d: VecDeque<u64> = (0..n).collect();
        let start = Instant::now();
        let mut sum_d = 0;
        while let Some(x) = d.pop_front() {
            sum_d += x;
        }
        let deque_time = start.elapsed();

        println!(
            "n = {n}: Vec {vec_time:.1?}, VecDeque {deque_time:.1?}, 합계 일치: {}",
            sum_v == sum_d
        );
    }
    // 출력 (시간은 실행마다 다릅니다):
    // n = 25000: Vec 30.5ms, VecDeque 108.3µs, 합계 일치: true
    // n = 50000: Vec 128.6ms, VecDeque 215.9µs, 합계 일치: true
}
