// 기대하는 에러: E0267
// 핵심 메시지: `break` inside of a closure
//
// 확인: rustc --edition 2024 compile_fail/e0267_break_inside_closure.rs --out-dir <임시 디렉터리>
//
// 점검표를 for 반복문으로 돌다가 실패한 항목에서 break하던 코드를 for_each로 옮겼습니다.
// 클로저 본문은 반복문의 본문이 아니라 따로 불리는 함수이므로, 그 안에는 끝낼
// 반복문이 없습니다. continue도 같은 에러입니다.

fn main() {
    let checks = [("전원", true), ("통신", true), ("연료 압력", false), ("항법 장치", true)];
    let mut passed = Vec::new();

    checks.iter().for_each(|&(name, ok)| {
        if !ok {
            break;
        }
        passed.push(name);
    });

    println!("통과: {passed:?}");
}

// 고치는 방법 (예제 05):
// - for 반복문으로 되돌립니다: for &(name, ok) in &checks { if !ok { break; } passed.push(name); }
// - 부수 효과가 없다면 어댑터가 멈추게 합니다:
//   let passed: Vec<_> = checks.iter().take_while(|(_, ok)| *ok).map(|(name, _)| *name).collect();
// - return으로 바꾸면 컴파일은 되지만 continue처럼 동작해서 결과가 틀립니다(예제 05의 2번 구역).
