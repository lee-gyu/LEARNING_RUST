//! &'static T와 T: 'static의 차이: "영원히 유효한 참조"와 "짧은 대여를 품지 않은 타입"은 다른 요구입니다.
//!
//! 실행: cargo run -p er-ch14-lifetimes --example 06_static_bound
//!
//! thread::spawn은 클로저와 결과에 T: 'static을 요구합니다(새 스레드가 언제 끝날지 모르기 때문).
//! 이 바운드는 "값이 프로그램 끝까지 산다"가 아니라 "값 안에 'static보다 짧은 참조가 없다"는 뜻이라서,
//! 실행 중에 만든 String도 만족합니다. 스레드 안에서는 출력하지 않고 결과를 돌려받아 main에서 출력합니다.

use std::sync::Arc;
use std::thread;

fn count_sentences(text: &str) -> usize {
    text.chars()
        .filter(|c| matches!(c, '.' | '?' | '!'))
        .count()
}

/// 비권장: spawn의 'static 바운드를 맞추려고 매개변수를 &'static str로 좁혔습니다.
/// 리터럴이 아닌 원고를 넘기려면 부르는 쪽이 누수시키는 수밖에 없습니다.
fn spawn_count_static(text: &'static str) -> thread::JoinHandle<usize> {
    thread::spawn(move || count_sentences(text))
}

/// 권장: 값을 소유한 채 넘겨받습니다. String은 빌린 것이 없으므로 'static 바운드를 만족합니다.
fn spawn_count(text: String) -> thread::JoinHandle<usize> {
    thread::spawn(move || count_sentences(&text))
}

/// T: 'static 바운드를 직접 건 판: &'static str, String, Arc<str>이 모두 들어갑니다.
/// 지역 String을 빌린 &str은 들어가지 않습니다(compile_fail/e0521_spawn_borrowed_param.rs).
fn spawn_count_any<T>(text: T) -> thread::JoinHandle<usize>
where
    T: AsRef<str> + Send + 'static,
{
    thread::spawn(move || count_sentences(text.as_ref()))
}

fn main() {
    // 실행 중에 만든 원고들입니다(파일에서 읽었다고 생각합니다). 리터럴이 아니므로 'static이 아닙니다.
    let drafts: Vec<String> = ["비가 왔다. 우산이 없었다!", "왜 하필 오늘일까? 뛰었다."]
        .iter()
        .map(|s| s.to_string())
        .collect();

    println!("1. 비권장: &'static str을 요구하면 부르는 쪽이 누수시켜야 합니다");
    let mut leaked_bytes = 0;
    let mut handles = Vec::new();
    for draft in &drafts {
        let forever: &'static str = Box::leak(draft.clone().into_boxed_str());
        leaked_bytes += forever.len();
        handles.push(spawn_count_static(forever));
    }
    let counts: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap_or(0)).collect();
    println!("    문장 수 {counts:?}, 되찾을 수 없게 된 바이트 {leaked_bytes}");
    // 출력:     문장 수 [2, 2], 되찾을 수 없게 된 바이트 70

    println!("2. 권장: 소유한 String을 넘기면 스레드가 끝날 때 drop됩니다");
    let handles: Vec<_> = drafts.iter().map(|d| spawn_count(d.clone())).collect();
    let counts: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap_or(0)).collect();
    println!("    문장 수 {counts:?}, 누수 0바이트");
    // 출력:     문장 수 [2, 2], 누수 0바이트

    println!("3. T: 'static은 '오래 산다'가 아니라 '짧은 대여가 없다'입니다");
    let literal = spawn_count_any("리터럴도 됩니다."); // &'static str
    let owned = spawn_count_any(drafts[0].clone()); // String
    let shared: Arc<str> = Arc::from(drafts[1].as_str());
    let from_arc = spawn_count_any(Arc::clone(&shared)); // Arc<str>
    let results = [literal, owned, from_arc].map(|h| h.join().unwrap_or(0));
    println!("    문장 수 {results:?}");
    // 스레드로 옮겨 간 Arc는 스레드가 끝나며 drop되어 소유자가 다시 하나뿐입니다.
    println!(
        "    스레드가 끝난 뒤 Arc의 소유자 수: {}",
        Arc::strong_count(&shared)
    );
    // 출력:
    //     문장 수 [1, 2, 2]
    //     스레드가 끝난 뒤 Arc의 소유자 수: 1

    println!("4. 기다릴 수 있다면 'static 자체가 필요 없습니다(스코프 스레드)");
    let counts: Vec<usize> = thread::scope(|s| {
        let handles: Vec<_> = drafts
            .iter()
            .map(|d| s.spawn(move || count_sentences(d)))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap_or(0)).collect()
    });
    println!("    문장 수 {counts:?}, 복사 0번");
    // 출력:     문장 수 [2, 2], 복사 0번
}
