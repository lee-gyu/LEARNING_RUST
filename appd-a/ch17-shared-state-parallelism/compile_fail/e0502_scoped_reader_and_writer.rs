// 기대하는 에러: E0502 — cannot borrow `occupied` as immutable because it is also borrowed as mutable
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0502_scoped_reader_and_writer.rs --out-dir /tmp/er-ch17
//
// 데이터 경쟁의 세 조건(같은 메모리, 적어도 하나는 쓰기, 동기화 없음) 중 "하나는 쓰고 하나는 읽는"
// 경우입니다. 스코프 스레드는 지역 변수를 빌려 쓰게 해 주지만, 두 스레드가 동시에 살아 있는 동안
// 한쪽이 &mut로 빌리면 다른 쪽은 &로도 빌릴 수 없습니다. 대여 규칙이 그대로 데이터 경쟁을 막습니다.
// 두 쪽이 모두 쓰는 경우(E0499)는 Part 1 ch11 4.1절에 있습니다.
// 해결: 쓰는 일과 읽는 일을 겹치지 않게 하거나(join한 뒤에 읽기), 정말 동시에 해야 한다면 값을
//       Mutex(값 하나라면 AtomicU32)로 감싸 &로 함께 씁니다.

use std::thread;

fn main() {
    let mut occupied = 12u32; // 지금 투숙 중인 객실 수

    thread::scope(|s| {
        s.spawn(|| occupied += 1); // 체크인: &mut occupied
        s.spawn(|| println!("현황판: 객실 {occupied}개 사용 중")); // E0502: &occupied
    });

    println!("마감: 객실 {occupied}개 사용 중");
}
