// 기대하는 에러: E0525 — expected a closure that implements the `Fn` trait,
//                          but this closure only implements `FnMut`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0525_fnmut_closure_to_fn_bound.rs --out-dir /tmp/er-ch2
//
// 받는 쪽(`run_job_strict`)은 단계 콜백을 차례로 한 번씩 부를 뿐인데 `Fn`을 요구했습니다.
// 부르는 쪽이 로그를 쌓는 클로저(`log`를 바꾸므로 `FnMut`)를 변수에 담아 두었다가 넘기면,
// 클로저의 종류는 정의할 때 이미 `FnMut`으로 정해졌으므로 E0525가 납니다.
// (같은 클로저를 인자 자리에 바로 쓰면 E0596이 나고, 받는 쪽을 `FnMut`으로 바꾸라는 도움말이
//  붙습니다. README 4.2절 참고.)
// 해결: 받는 쪽의 바운드를 `S: FnMut(usize, &str)`로 바꾸고 매개변수에 `mut`을 붙입니다
//       (examples/02_loosest_fn_bound.rs의 `run_job`).

fn run_job_strict<S>(steps: &[&str], on_step: S)
where
    S: Fn(usize, &str),
{
    for (i, step) in steps.iter().enumerate() {
        on_step(i + 1, step);
    }
}

fn main() {
    let mut log = Vec::new();
    let record = |i: usize, name: &str| log.push(format!("{i}. {name}"));
    run_job_strict(&["압축", "전송"], record); // E0525
    println!("{log:?}");
}
