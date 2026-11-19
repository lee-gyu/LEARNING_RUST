// 기대 에러: E0308 mismatched types
//   expected `&Vec<u32>`, found `&[u32; 7]`
//
// 받는 쪽이 `&Vec<u32>`로 선언하면, 배열을 가진 부르는 쪽은 그대로 넘길 수 없습니다.
// 에러는 부르는 쪽 줄에서 나지만, note가 가리키는 받는 쪽 매개변수가 원인입니다.
// 부르는 쪽에서 `&week.to_vec()`으로 복사해 넘기면 컴파일되지만 할당과 복사가 생깁니다.
// 받는 쪽을 고칠 수 있다면 매개변수를 `&[u32]`로 바꾸는 것이 맞는 해결입니다(예제 01의 `average`).
//
// 확인: rustc --edition 2024 compile_fail/e0308_vec_param_rejects_array.rs --out-dir /tmp/er-ch8

fn average_vec(days: &Vec<u32>) -> u32 {
    days.iter().sum::<u32>() / days.len() as u32
}

fn main() {
    let week: [u32; 7] = [6200, 8400, 7100, 9900, 5300, 12000, 7600];
    println!("{}", average_vec(&week)); // E0308
}
