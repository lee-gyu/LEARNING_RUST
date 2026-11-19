// 기대하는 에러: E0499
// 핵심 메시지: cannot borrow `notes` as mutable more than once at a time
//
// 확인: rustc --edition 2024 compile_fail/e0499_two_closures_same_state.rs --out-dir <임시 디렉터리>
//
// 포스기 기록을 읽으면서 건너뛴 줄의 사유를 notes 하나에 남기려고, 체인의 두 단계
// (filter와 filter_map)에서 각각 notes.push를 불렀습니다. 어댑터는 클로저를 필드로
// 들고 있다가 collect가 돌 때 번갈아 부르므로, 두 클로저가 notes를 가변으로 빌린 채
// 동시에 살아 있어야 합니다.

fn main() {
    let lines = ["128", "", "75", "칠십"];
    let mut notes: Vec<String> = Vec::new();

    let cups: Vec<u32> = lines
        .iter()
        .filter(|line| {
            if line.is_empty() {
                notes.push("빈 줄".to_string());
                return false;
            }
            true
        })
        .filter_map(|line| match line.parse() {
            Ok(n) => Some(n),
            Err(_) => {
                notes.push(format!("숫자가 아님: {line}"));
                None
            }
        })
        .collect();

    println!("{cups:?} {notes:?}");
}

// 고치는 방법 (예제 06):
// - 상태를 여러 곳에서 고쳐야 하면 for 반복문과 match 하나로 씁니다. 본문 한 곳에서
//   cups.push와 notes.push를 하므로 대여가 겹치지 않습니다.
// - 체인을 유지하려면 상태를 고치는 단계를 하나로 합칩니다(filter_map 하나가 빈 줄도 처리).
