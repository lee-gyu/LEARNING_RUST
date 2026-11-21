// 부록 B ch15 · Q15.3 — 2단계 대여: 예전에는 거부되던 코드가 받아들여지는 예
//
// 대여 검사기는 보수적이지만, 받아들이는 범위는 넓어져 왔습니다. 그 하나가 2단계 대여(two-phase borrow)입니다.
// 컴파일러가 몰래 만드는 일부 &mut 대여는 먼저 "예약"만 해 두고(그동안은 불변 대여처럼 동작), 인자를 다 계산한 뒤에
// 가변 대여로 바꿉니다. 그래서 &mut로 빌리는 호출의 인자에서 같은 값을 읽을 수 있습니다.
// rustc 개발 안내서(Two-phase borrows)가 드는 세 경우를 각각 보입니다.
//   1. &mut self 메서드를 부를 때의 자동 참조(autoref): v.push(v.len())
//   2. 함수 인자로 넘기는 가변 재대여: mem::replace(r, …r.len()…)
//   3. 오버로드한 복합 대입 연산자: total += total
// 소스에 직접 적은 &mut는 2단계 대여가 아닙니다. 그 경계는 README의 연습 1에서 봅니다.
//
// 실행: cargo run -p faq-ch15-pain-points --example 01_two_phase_borrow

use std::mem;
use std::ops::AddAssign;

/// 복합 대입 연산자 += 를 직접 구현한 타입. Copy라서 오른쪽에 자기 자신을 값으로 넘길 수 있습니다.
#[derive(Clone, Copy, Debug)]
struct Meters(u32);

impl AddAssign for Meters {
    fn add_assign(&mut self, other: Meters) {
        self.0 += other.0;
    }
}

fn main() {
    println!("1. 메서드 호출의 자동 참조");
    let mut lengths = vec![3, 5];
    // push는 &mut self를 받습니다. &mut lengths를 예약해 두고 인자 lengths.len()을 먼저 계산합니다.
    lengths.push(lengths.len());
    println!("  {lengths:?}"); // 출력:   [3, 5, 2]

    println!("2. 함수 인자의 가변 재대여");
    let mut queue = vec![10, 20, 30];
    let r = &mut queue;
    // r을 넘기는 자리의 재대여(&mut *r)가 예약 상태인 동안 둘째 인자에서 r.len()을 읽습니다.
    let old = mem::replace(r, Vec::with_capacity(r.len()));
    println!("  옛 값: {old:?}, 지금 값: {queue:?}"); // 출력:   옛 값: [10, 20, 30], 지금 값: []

    println!("3. 오버로드한 복합 대입 연산자");
    let mut total = Meters(5);
    // total.add_assign(total)처럼 동작합니다. total의 가변 대여를 예약해 두고 오른쪽 total을 먼저 복사합니다.
    total += total;
    println!("  {total:?}"); // 출력:   Meters(10)
}
