// 기대하는 에러: E0308 — mismatched types (expected fn pointer, found closure)
// 핵심 메시지: closures can only be coerced to `fn` types if they do not capture any variables
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0308_capturing_closure_to_fn_pointer.rs --out-dir /tmp/er-ch2
//
// 받는 쪽(`first_alert_ptr`)이 판단 기준을 `fn(i32) -> bool`로 선언했습니다. `fn` 포인터에는
// 코드의 주소만 들어가므로, 실행 중에 계산한 `limit`을 캡처한 클로저는 넘길 수 없습니다.
// 에러는 부르는 쪽 줄에서 나지만, 고칠 곳은 받는 쪽의 시그니처입니다.
// 해결: 받는 쪽을 `is_alert: impl FnMut(i32) -> bool`로 바꿉니다(examples/01_fn_pointer_vs_fn_bound.rs).

fn first_alert_ptr(readings: &[i32], is_alert: fn(i32) -> bool) -> Option<usize> {
    readings.iter().position(|&t| is_alert(t))
}

fn main() {
    let readings = [6, 9, 12, 17, 15, 11];
    let limit = readings.iter().sum::<i32>() / readings.len() as i32 + 5;
    let found = first_alert_ptr(&readings, |t| t >= limit); // E0308
    println!("{found:?}");
}
