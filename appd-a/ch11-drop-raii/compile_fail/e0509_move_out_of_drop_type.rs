// 기대하는 에러: E0509 — cannot move out of type `NoteSync<'_>`, which implements the `Drop` trait
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0509_move_out_of_drop_type.rs --out-dir /tmp/er-ch11
//
// finish(self)가 self를 받았으니 필드를 옮겨 꺼내도 될 것 같지만, Drop을 구현한 타입은 필드를
// 옮길 수 없습니다. finish가 끝나면 self에 대해 Drop::drop(&mut self)가 실행되는데, 그때
// 필드가 비어 있으면 drop이 옮겨진 값을 보게 되기 때문입니다.
// 주의: 컴파일러의 help는 &self.pending(빌리기)과 self.pending.clone()(복제)을 제안합니다.
//       clone을 따르면 finish가 올린 메모를 Drop이 한 번 더 올립니다
//       (examples/03_fallible_release.rs의 2번 구역).
// 해결: mem::take(&mut self.pending)로 꺼내고 빈 Vec을 남깁니다(fn finish(mut self)).
//       기본값이 없는 필드는 Option<T>에 담고 take()로 꺼냅니다(examples/01_manual_vs_guard.rs의 PooledConn).

struct Server {
    notes: Vec<String>,
}

struct NoteSync<'a> {
    server: &'a mut Server,
    pending: Vec<String>,
}

impl NoteSync<'_> {
    fn finish(self) -> usize {
        let pending = self.pending; // E0509
        let count = pending.len();
        self.server.notes.extend(pending);
        count
    }
}

impl Drop for NoteSync<'_> {
    fn drop(&mut self) {
        // finish 없이 버려졌을 때의 안전망
        self.server.notes.append(&mut self.pending);
    }
}

fn main() {
    let mut server = Server { notes: Vec::new() };
    let sync = NoteSync {
        server: &mut server,
        pending: vec![String::from("우유 사기")],
    };
    println!("{}개 올림", sync.finish());
}
