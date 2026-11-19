// 기대하는 에러: E0521
// 핵심 메시지: borrowed data escapes outside of function
//              argument requires that `'1` must outlive `'static`
//
// thread::spawn은 클로저에 F: 'static을 요구합니다. 클로저가 매개변수 text(&str)를 품으면
// 클로저의 타입에 text의 라이프타임 '1이 들어가는데, '1은 부르는 쪽이 정하는 구간이라
// 'static보다 짧을 수 있습니다. 그래서 빌린 데이터가 함수 밖(새 스레드)으로 빠져나간다고 거부합니다.
// 매개변수를 &'static str로 바꾸면 컴파일되지만 부르는 쪽에 누수를 떠넘깁니다.
// String을 받거나(소유), 스코프 스레드를 쓰는 것이 해결입니다(examples/06_static_bound.rs).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0521_spawn_borrowed_param.rs --out-dir /tmp/er-ch14

use std::thread;

fn count_sentences(text: &str) -> usize {
    text.chars().filter(|c| matches!(c, '.' | '?' | '!')).count()
}

fn spawn_count_borrowed(text: &str) -> thread::JoinHandle<usize> {
    thread::spawn(move || count_sentences(text)) // E0521
}

fn main() {
    let draft = String::from("비가 왔다. 우산이 없었다!");
    let handle = spawn_count_borrowed(&draft);
    println!("문장 수: {}", handle.join().unwrap_or(0));
}
