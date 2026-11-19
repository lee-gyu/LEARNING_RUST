// 기대하는 에러: E0502 — cannot borrow `server.notes` as immutable because it is also borrowed as mutable
//                핵심 줄: mutable borrow might be used here, when `sync` is dropped and runs
//                         the `Drop` code for type `AutoSync`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0502_drop_guard_keeps_borrow.rs --out-dir /tmp/er-ch11
//
// sync는 server를 &mut로 빌린 가드입니다. 보통의 대여는 마지막으로 쓴 곳에서 끝나지만,
// Drop을 구현한 타입은 스코프 끝에서 drop(&mut self)가 그 대여를 다시 씁니다. 그래서 sync를
// 마지막으로 쓴 뒤에도 스코프가 끝날 때까지 server를 읽을 수 없습니다.
// (impl Drop을 지우면 이 파일은 컴파일됩니다.)
// 해결: 가드를 블록에 넣어 범위를 좁히거나(examples/03_fallible_release.rs의 1번 구역),
//       server를 읽기 전에 drop(sync)를 부릅니다.

struct Server {
    notes: Vec<String>,
}

struct AutoSync<'a> {
    server: &'a mut Server,
    pending: Vec<String>,
}

impl Drop for AutoSync<'_> {
    fn drop(&mut self) {
        self.server.notes.append(&mut self.pending);
    }
}

fn main() {
    let mut server = Server { notes: Vec::new() };
    let mut sync = AutoSync {
        server: &mut server,
        pending: Vec::new(),
    };
    sync.pending.push(String::from("우유 사기"));
    println!("서버에 저장된 메모 {}개", server.notes.len()); // E0502
}
