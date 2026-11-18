// std::panic::catch_unwind로 되감기 중인 패닉을 붙잡습니다.
//
//   1) 클로저가 패닉하지 않으면 Ok(반환값), 패닉하면 Err(페이로드)를 돌려줍니다.
//      페이로드는 panic!에 넘긴 값으로, 타입은 Box<dyn Any + Send>입니다.
//   2) 붙잡아도 패닉 훅은 이미 실행된 뒤라서 기본 훅의 메시지가 표준 에러에 출력됩니다.
//   3) 캡처한 &mut는 되감기 경계를 그냥 넘을 수 없습니다. AssertUnwindSafe로 "괜찮다"고 직접 약속해야 하며,
//      실제로 반쯤 바뀐 상태가 남을 수 있습니다.
//   4) resume_unwind는 붙잡은 패닉을 패닉 훅을 거치지 않고 다시 일으킵니다.
//
// 실행: cargo run -p p1-ch8-errors-panic --example 06_catch_unwind
//
// 이 예제는 패닉을 모두 안에서 붙잡으므로 종료 코드 0으로 끝납니다.
// 표준 에러에 나오는 패닉 메시지의 스레드 ID(괄호 안 숫자)는 실행할 때마다 달라집니다.

use std::any::Any;
use std::panic::{self, AssertUnwindSafe};

// 페이로드를 사람이 읽을 문자열로 바꿉니다.
// panic!("고정 문자열")은 &'static str을, 서식 인자가 있는 panic!과 표준 라이브러리의 패닉 대부분은
// String을 페이로드로 씁니다. dyn Any는 "어떤 타입이든"이라는 뜻이고, downcast_ref::<T>()는
// "안에 든 것이 T이면 Some(&T)"입니다(Any 자체는 ch14-ffi-any-misc에서 다룹니다).
fn payload_text(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        format!("&str 페이로드: {s}")
    } else if let Some(s) = payload.downcast_ref::<String>() {
        format!("String 페이로드: {s}")
    } else {
        "문자열이 아닌 페이로드".to_string()
    }
}

fn divide(a: i32, b: i32) -> i32 {
    a / b // b가 0이면 패닉합니다.
}

fn speed_of(mode: &str) -> i32 {
    match mode {
        "slow" => 1,
        "fast" => 10,
        // 호출하는 코드가 정해진 모드만 넘긴다는 약속을 어기면 패닉합니다.
        other => panic!("알 수 없는 모드: {other}"),
    }
}

// 입금 후 잔액이 한도를 넘으면 패닉합니다.
fn deposit(balance: &mut i32, amount: i32) {
    assert!(
        *balance + amount <= 600,
        "입금 후 잔액이 한도 600을 넘습니다"
    );
    *balance += amount;
}

// (작업 이름, 작업). 캡처가 없는 클로저는 함수 포인터 fn() -> i32로 바꿀 수 있으므로(Part 0 ch19)
// 클로저마다 타입이 달라도 한 배열에 담을 수 있습니다.
type Job = (&'static str, fn() -> i32);

fn main() {
    println!("1. 패닉이 없으면 Ok(클로저의 반환값)입니다");
    let result = panic::catch_unwind(|| divide(84, 2));
    println!("    {result:?}");
    // 출력: Ok(42)

    println!("2. 작업마다 경계를 두면 한 작업의 패닉이 다른 작업을 멈추지 않습니다");
    let jobs: [Job; 4] = [
        ("정상 나눗셈", || divide(10, 2)),
        ("0으로 나눔", || divide(10, 0)),
        // black_box는 컴파일러가 값을 미리 계산하지 못하게 막습니다. [1, 2, 3][7]이라고 쓰면
        // 컴파일 중에 범위 밖임을 알아채고 unconditional_panic 에러로 거부합니다.
        ("범위 밖 인덱스", || {
            [1, 2, 3][std::hint::black_box(7)]
        }),
        ("직접 panic!", || speed_of("turbo")),
    ];
    let mut failed = 0;
    for (name, job) in jobs {
        match panic::catch_unwind(job) {
            Ok(v) => println!("    [{name}] 성공: {v}"),
            Err(payload) => {
                failed += 1;
                // payload는 Box<dyn Any + Send>입니다. &*payload로 Box 안의 값을 빌려 넘깁니다.
                // &payload를 넘기면 Box 자체가 Any로 취급되어 다운캐스트가 모두 실패합니다.
                println!("    [{name}] 패닉을 붙잡음 → {}", payload_text(&*payload));
            }
        }
    }
    println!(
        "    작업 {}개 중 {failed}개가 패닉했지만 main은 계속 실행됩니다",
        jobs.len()
    );
    // 출력 (표준 출력):
    //     [정상 나눗셈] 성공: 5
    //     [0으로 나눔] 패닉을 붙잡음 → &str 페이로드: attempt to divide by zero
    //     [범위 밖 인덱스] 패닉을 붙잡음 → String 페이로드: index out of bounds: the len is 3 but the index is 7
    //     [직접 panic!] 패닉을 붙잡음 → String 페이로드: 알 수 없는 모드: turbo
    //     작업 4개 중 3개가 패닉했지만 main은 계속 실행됩니다
    // 패닉할 때마다 기본 패닉 훅이 붙잡히기 전에 실행되어 표준 에러에 메시지를 남깁니다(README 3절).

    println!("3. &mut를 캡처하려면 AssertUnwindSafe로 직접 약속해야 합니다");
    // 두 계좌 사이의 이체입니다. "두 잔액의 합은 늘 1000"이 지켜져야 할 불변식입니다.
    let mut accounts = [600, 400];
    // catch_unwind(|| { accounts[0] -= 300; ... })라고 쓰면 컴파일되지 않습니다
    // (compile_fail/e0277_catch_unwind_mut_ref.rs). 클로저가 &mut를 캡처하기 때문입니다.
    // AssertUnwindSafe로 감싸면 컴파일되지만, 패닉 뒤의 상태를 확인할 책임은 우리에게 넘어옵니다.
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        accounts[0] -= 300; // 출금은 끝났는데
        deposit(&mut accounts[1], 300); // 입금에서 패닉하면
    }));
    println!(
        "    결과: {}",
        if result.is_err() { "패닉" } else { "성공" }
    );
    println!("    잔액 {accounts:?}, 합계 {}", accounts[0] + accounts[1]);
    // 출력:
    //     결과: 패닉
    //     잔액 [300, 400], 합계 700
    // 불변식이 깨진 상태를 패닉 뒤에 그대로 보게 됐습니다. UnwindSafe가 막으려는 것이 바로 이것입니다.

    println!("4. resume_unwind는 패닉 훅을 거치지 않고 패닉을 다시 일으킵니다");
    let outer = panic::catch_unwind(|| {
        let inner = panic::catch_unwind(|| -> i32 { panic!("안쪽 작업 실패") });
        if let Err(payload) = inner {
            println!(
                "    안쪽에서 붙잡아 기록만 하고 다시 던집니다: {}",
                payload_text(&*payload)
            );
            panic::resume_unwind(payload); // 같은 페이로드로 다시 되감기를 시작합니다.
        }
    });
    if let Err(payload) = outer {
        println!("    바깥에서 다시 붙잡음: {}", payload_text(&*payload));
    }
    // 출력 (표준 출력):
    //     안쪽에서 붙잡아 기록만 하고 다시 던집니다: &str 페이로드: 안쪽 작업 실패
    //     바깥에서 다시 붙잡음: &str 페이로드: 안쪽 작업 실패
    // 표준 에러에는 "안쪽 작업 실패" 패닉 메시지가 한 번만 나옵니다. resume_unwind는 훅을 부르지 않습니다.
}
