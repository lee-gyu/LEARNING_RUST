// 기대하는 에러: E0277 — the type `&mut [i32; 2]` may not be safely transferred across an unwind boundary
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_catch_unwind_mut_ref.rs --out-dir /tmp/rust-p1-ch8
//
// catch_unwind는 클로저에 UnwindSafe를 요구합니다. 이 클로저는 accounts를 &mut로 캡처하는데,
// &mut T는 UnwindSafe가 아닙니다. 클로저가 출금과 입금 사이에서 패닉하면, 반쯤 바뀐 accounts를
// catch_unwind 뒤에서 그대로 보게 되기 때문입니다.
// 해결: 패닉 뒤의 상태를 직접 확인하겠다는 뜻으로 AssertUnwindSafe로 감쌉니다.
//       panic::catch_unwind(AssertUnwindSafe(|| { ... }))  (examples/06_catch_unwind.rs의 3번 구역)

use std::panic;

fn main() {
    let mut accounts = [600, 400];
    let result = panic::catch_unwind(|| {
        accounts[0] -= 300;
        accounts[1] += 300;
    });
    println!("{result:?} {accounts:?}");
}
