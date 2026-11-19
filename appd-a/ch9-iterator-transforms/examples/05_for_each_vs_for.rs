//! 예제 05: 부수 효과가 본문이면 `for_each`보다 `for`
//!
//! 실행: cargo run -q -p er-ch9-iterator-transforms --example 05_for_each_vs_for
//!
//! 로켓 발사 전 점검표를 차례로 확인하며 통과한 항목을 기록합니다.
//! 값을 만들지 않고 "일을 하는" 본문은 for_each로 옮겨도 얻는 것이 없고,
//! 흐름을 바꿔야 하는 순간 클로저의 경계 때문에 틀린 코드가 되기 쉽습니다.

/// 점검 항목 하나.
struct Check {
    name: &'static str,
    passed: bool,
}

// ---- 1. 부수 효과만 낼 때 ---------------------------------------------------

fn log_all_for_each(checks: &[Check], report: &mut Vec<String>) {
    checks.iter().for_each(|c| {
        let mark = if c.passed { "통과" } else { "실패" };
        report.push(format!("{}: {mark}", c.name));
    });
}

fn log_all_for(checks: &[Check], report: &mut Vec<String>) {
    for c in checks {
        let mark = if c.passed { "통과" } else { "실패" };
        report.push(format!("{}: {mark}", c.name));
    }
}

// ---- 2. 실패하면 멈춰야 할 때 -----------------------------------------------

// 권장: for 안의 return은 함수를 끝냅니다.
fn countdown_for(checks: &[Check], report: &mut Vec<&'static str>) -> Result<(), &'static str> {
    for c in checks {
        if !c.passed {
            return Err(c.name);
        }
        report.push(c.name);
    }
    Ok(())
}

// 비권장: 위의 본문을 그대로 for_each로 옮겼습니다. 컴파일은 되지만
// 클로저 안의 return은 클로저 한 번의 호출만 끝냅니다. 즉 continue처럼 동작해서
// 실패한 뒤에도 다음 항목을 계속 확인하고, 함수는 실패를 알릴 방법이 없습니다.
fn countdown_for_each(
    checks: &[Check],
    report: &mut Vec<&'static str>,
) -> Result<(), &'static str> {
    checks.iter().for_each(|c| {
        if !c.passed {
            return; // 바깥 함수가 아니라 이 클로저에서 빠져나갑니다
        }
        report.push(c.name);
    });
    Ok(())
}

// 체인으로 쓰고 싶다면 try_for_each: 클로저가 Err를 돌려주면 순회가 멈추고
// 그 Err가 try_for_each의 결과가 됩니다. 결과는 for판과 같지만 Ok(())를
// 직접 돌려줘야 하는 등 for판보다 적을 것이 많습니다.
fn countdown_try_for_each(
    checks: &[Check],
    report: &mut Vec<&'static str>,
) -> Result<(), &'static str> {
    checks.iter().try_for_each(|c| {
        if !c.passed {
            return Err(c.name); // 클로저가 Err를 돌려줌 → try_for_each가 멈춤
        }
        report.push(c.name);
        Ok(())
    })
}

fn check(name: &'static str, passed: bool) -> Check {
    Check { name, passed }
}

fn main() {
    let checks = [
        check("전원", true),
        check("통신", true),
        check("연료 압력", false),
        check("항법 장치", true),
        check("발사대 고정", true),
    ];

    println!("== 1. 부수 효과만 낼 때: 결과는 같습니다 ==");
    let (mut a, mut b) = (Vec::new(), Vec::new());
    log_all_for_each(&checks, &mut a);
    log_all_for(&checks, &mut b);
    println!("{:?} … (for_each판과 for판 일치: {})", &b[..2], a == b);
    // 출력: ["전원: 통과", "통신: 통과"] … (for_each판과 for판 일치: true)

    println!();
    println!("== 2. 실패하면 멈춰야 할 때 ==");
    type Countdown = fn(&[Check], &mut Vec<&'static str>) -> Result<(), &'static str>;
    let versions: [(&str, Countdown); 3] = [
        ("for판", countdown_for),
        ("for_each판", countdown_for_each),
        ("try_for_each판", countdown_try_for_each),
    ];
    for (label, countdown) in versions {
        let mut passed = Vec::new();
        let result = countdown(&checks, &mut passed);
        println!("{label}: {result:?}, 통과 기록 {passed:?}");
    }
    // 출력: for판: Err("연료 압력"), 통과 기록 ["전원", "통신"]
    // 출력: for_each판: Ok(()), 통과 기록 ["전원", "통신", "항법 장치", "발사대 고정"]
    // 출력: try_for_each판: Err("연료 압력"), 통과 기록 ["전원", "통신"]
}
