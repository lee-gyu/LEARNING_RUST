// 지침 2: 가드가 사는 범위를 블록으로 좁힙니다. 너무 길어도, 너무 짧아도 문제입니다.
//
// 게임 순위표에 점수를 등록하고 "몇 위가 됐다"는 알림 문구를 만듭니다. 알림 만들기는 순위표가
// 필요 없는 느린 일(실제로는 메시지 전송)이라고 가정합니다.
//
//   1. 비권장: 잠금 가드를 쥔 채 알림까지 만듦 → 그동안 다른 스레드는 순위표를 잠그지 못함
//   2. 권장: 블록 안에서 순위표를 고치고 필요한 값(순위)만 꺼냄 → 블록이 끝나면 잠금이 풀림
//   3. 권장: 블록이 어색하면 drop(guard)로 그 자리에서 풀기
//   4. 너무 짧은 범위: 직접 만든 동결 가드를 이름 없이 버리면 그 자리에서 풀림
//
// "다른 스레드가 잠글 수 있는가"는 스코프 스레드에서 try_lock으로 관찰하고, 결과는 main에서
// 출력합니다. 스레드를 바로 join하므로 출력은 실행마다 같습니다.
//
// 실행: cargo run -p er-ch11-drop-raii --example 02_narrow_guard_scope

use std::cmp::Reverse;
use std::sync::{Mutex, PoisonError};
use std::thread;

struct Board {
    scores: Vec<(&'static str, u32)>,
    frozen: bool,
}

struct Leaderboard {
    inner: Mutex<Board>,
}

impl Leaderboard {
    fn new() -> Leaderboard {
        let scores = vec![("민지", 900), ("서준", 750), ("하린", 600)];
        Leaderboard {
            inner: Mutex::new(Board {
                scores,
                frozen: false,
            }),
        }
    }

    /// 지금 이 순간 다른 스레드가 순위표를 잠글 수 있는지 봅니다(기다리지 않는 try_lock).
    fn other_thread_can_lock(&self) -> bool {
        thread::scope(|s| s.spawn(|| self.inner.try_lock().is_ok()).join().unwrap())
    }

    /// 점수를 넣고 정렬한 뒤 순위(1부터)를 돌려줍니다. 가드를 쥔 쪽이 부릅니다.
    fn insert(board: &mut Board, name: &'static str, score: u32) -> usize {
        board.scores.push((name, score));
        board.scores.sort_by_key(|&(_, score)| Reverse(score));
        board.scores.iter().position(|&(n, _)| n == name).unwrap() + 1
    }

    /// 동결 가드를 돌려줍니다. 가드가 살아 있는 동안 submit은 거절됩니다.
    fn freeze(&self) -> Frozen<'_> {
        self.inner.lock().unwrap().frozen = true;
        Frozen { board: self }
    }

    /// 늦게 도착한 점수 제출. 동결 중이면 받지 않습니다.
    fn submit(&self, name: &'static str, score: u32) -> bool {
        let mut board = self.inner.lock().unwrap();
        if board.frozen {
            return false;
        }
        board.scores.push((name, score));
        true
    }
}

/// 알림 문구 만들기. 순위표가 필요 없는 일입니다.
fn announce(name: &str, rank: usize) -> String {
    format!("{name} 님이 {rank}위가 되었습니다")
}

/// 비권장: 가드가 함수 끝까지 살아 있어 알림을 만드는 동안에도 잠겨 있습니다.
fn record_holding(lb: &Leaderboard, name: &'static str, score: u32) -> (String, bool) {
    let mut board = lb.inner.lock().unwrap();
    let rank = Leaderboard::insert(&mut board, name, score);
    // ---- 여기부터는 순위표가 필요 없는 일 ----
    let others_can_lock = lb.other_thread_can_lock();
    (announce(name, rank), others_can_lock)
} // board(가드)는 여기서야 버려집니다.

/// 권장: 순위표를 고치는 일만 블록 안에서 하고, 블록의 값으로 순위만 꺼냅니다.
fn record_scoped(lb: &Leaderboard, name: &'static str, score: u32) -> (String, bool) {
    let rank = {
        let mut board = lb.inner.lock().unwrap();
        Leaderboard::insert(&mut board, name, score)
    }; // 가드는 여기서 버려집니다.
    let others_can_lock = lb.other_thread_can_lock();
    (announce(name, rank), others_can_lock)
}

/// 권장: 블록으로 감싸기 어색할 때는 다 쓴 자리에서 drop(guard)를 부릅니다.
fn record_with_drop(lb: &Leaderboard, name: &'static str, score: u32) -> (String, bool) {
    let mut board = lb.inner.lock().unwrap();
    let rank = Leaderboard::insert(&mut board, name, score);
    let top = board.scores[0].0;
    drop(board); // 여기서 잠금이 풀립니다. 이 뒤에 board를 쓰면 E0382입니다.
    let others_can_lock = lb.other_thread_can_lock();
    let message = format!("{} (1위 {top})", announce(name, rank));
    (message, others_can_lock)
}

/// 동결 가드. Drop에서 동결을 풉니다.
#[must_use = "버리면 동결이 바로 풀립니다. `let _frozen = ...`처럼 이름을 붙이세요"]
struct Frozen<'a> {
    board: &'a Leaderboard,
}

impl Drop for Frozen<'_> {
    fn drop(&mut self) {
        // drop에서는 패닉하지 않도록 포이즈닝된 잠금도 그대로 씁니다.
        let inner = &self.board.inner;
        inner.lock().unwrap_or_else(PoisonError::into_inner).frozen = false;
    }
}

fn print_record(label: &str, (message, others_can_lock): (String, bool)) {
    println!("    {label}: {message}");
    println!("    알림을 만드는 동안 다른 스레드가 잠글 수 있었나: {others_can_lock}");
}

fn main() {
    println!("1. 비권장: 잠금 가드를 쥔 채 알림까지 만듦");
    let lb = Leaderboard::new();
    print_record("기록", record_holding(&lb, "도윤", 800));
    // 출력:     기록: 도윤 님이 2위가 되었습니다
    // 출력:     알림을 만드는 동안 다른 스레드가 잠글 수 있었나: false

    println!("2. 권장: 블록으로 가드의 범위를 좁힘");
    let lb = Leaderboard::new();
    print_record("기록", record_scoped(&lb, "도윤", 800));
    // 출력:     기록: 도윤 님이 2위가 되었습니다
    // 출력:     알림을 만드는 동안 다른 스레드가 잠글 수 있었나: true

    println!("3. 권장: drop(guard)로 다 쓴 자리에서 풂");
    let lb = Leaderboard::new();
    print_record("기록", record_with_drop(&lb, "도윤", 800));
    // 출력:     기록: 도윤 님이 2위가 되었습니다 (1위 민지)
    // 출력:     알림을 만드는 동안 다른 스레드가 잠글 수 있었나: true

    println!("4. 너무 짧은 범위: 시즌 정산 중에는 늦은 제출을 막아야 합니다");
    let lb = Leaderboard::new();
    {
        #[expect(unused_must_use, reason = "가드를 문장으로 버리는 실수")]
        lb.freeze(); // 문장 끝에서 바로 버려집니다.
        let accepted = lb.submit("늦은 제출", 999);
        println!("    lb.freeze();              → 정산 중 늦은 제출을 받음: {accepted}");
        // 출력:     lb.freeze();              → 정산 중 늦은 제출을 받음: true
    }
    let lb = Leaderboard::new();
    {
        let _ = lb.freeze(); // `_`는 값을 묶지 않으므로 역시 바로 버려집니다. 경고도 없습니다.
        let accepted = lb.submit("늦은 제출", 999);
        println!("    let _ = lb.freeze();      → 정산 중 늦은 제출을 받음: {accepted}");
        // 출력:     let _ = lb.freeze();      → 정산 중 늦은 제출을 받음: true
    }
    let lb = Leaderboard::new();
    {
        let _frozen = lb.freeze(); // 이름이 있는 변수는 블록 끝까지 삽니다.
        let accepted = lb.submit("늦은 제출", 999);
        println!("    let _frozen = lb.freeze(); → 정산 중 늦은 제출을 받음: {accepted}");
        // 출력:     let _frozen = lb.freeze(); → 정산 중 늦은 제출을 받음: false
    } // _frozen이 여기서 버려지며 동결이 풀립니다.
    let accepted = lb.submit("다음 시즌", 100);
    println!("    정산이 끝난 뒤의 제출을 받음: {accepted}");
    // 출력:     정산이 끝난 뒤의 제출을 받음: true
}
