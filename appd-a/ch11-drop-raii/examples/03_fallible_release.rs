// 지침 3: 실패할 수 있는 해제는 Result를 돌려주는 메서드로 따로 두고, Drop은 안전망으로 남깁니다.
//
// 메모 앱이 편집한 메모를 모아 두었다가 서버에 한꺼번에 올립니다. 서버에는 할당량이 있어서
// 넘치면 올리기가 실패하고, 그때는 하나도 저장되지 않습니다.
//
//   1. 비권장: Drop에서만 올림 — drop은 값을 돌려주지 못하므로 실패가 조용히 사라집니다.
//   2. 비권장: finish(self)를 만들었지만 E0509를 피하려고 clone — finish가 끝날 때 Drop이 한 번 더 올립니다.
//   3. 권장: finish(mut self) -> Result로 올리고 mem::take로 비워 둡니다.
//      Drop은 finish를 부르지 못한 경우(조기 반환, ?, 패닉)를 위한 안전망입니다.
//
// 실행: cargo run -p er-ch11-drop-raii --example 03_fallible_release

use std::mem;

struct Server {
    quota: usize,
    notes: Vec<String>,
}

#[derive(Debug)]
struct QuotaExceeded {
    requested: usize,
    available: usize,
}

impl Server {
    fn new(quota: usize) -> Server {
        Server {
            quota,
            notes: Vec::new(),
        }
    }

    /// 모두 올리거나, 하나도 올리지 않습니다.
    fn upload(&mut self, notes: &[String]) -> Result<(), QuotaExceeded> {
        let available = self.quota - self.notes.len();
        if notes.len() > available {
            return Err(QuotaExceeded {
                requested: notes.len(),
                available,
            });
        }
        self.notes.extend_from_slice(notes);
        Ok(())
    }
}

const EDITS: [&str; 3] = ["우유 사기", "치과 예약", "택배 찾기"];

// ---------- 1. 비권장: Drop에서만 올림 ----------

struct AutoSync<'a> {
    server: &'a mut Server,
    pending: Vec<String>,
}

impl AutoSync<'_> {
    fn edit(&mut self, note: &str) {
        self.pending.push(note.to_string());
    }
}

impl Drop for AutoSync<'_> {
    fn drop(&mut self) {
        // drop은 ()만 돌려줍니다. 실패를 알릴 길이 없으니 결과를 버릴 수밖에 없습니다.
        let _ = self.server.upload(&self.pending);
    }
}

// ---------- 2. 비권장: finish(self)에서 필드를 clone ----------

struct CloneSync<'a> {
    server: &'a mut Server,
    pending: Vec<String>,
}

impl CloneSync<'_> {
    fn edit(&mut self, note: &str) {
        self.pending.push(note.to_string());
    }

    fn finish(self) -> Result<usize, QuotaExceeded> {
        // let pending = self.pending; 는 E0509입니다(Drop을 구현한 타입에서 필드를 옮길 수 없음).
        // 컴파일러의 help 중 하나인 clone()을 따르면 컴파일은 됩니다.
        let pending = self.pending.clone();
        self.server.upload(&pending)?;
        Ok(pending.len())
    } // self가 여기서 버려지며 Drop이 원본 pending을 한 번 더 올립니다.
}

impl Drop for CloneSync<'_> {
    fn drop(&mut self) {
        let _ = self.server.upload(&self.pending);
    }
}

// ---------- 3. 권장: Result를 돌려주는 finish + 안전망 Drop ----------

struct NoteSync<'a> {
    server: &'a mut Server,
    pending: Vec<String>,
}

/// 올리지 못한 메모를 함께 돌려주는 에러. 부르는 쪽이 보관하거나 다시 시도할 수 있습니다.
#[derive(Debug)]
struct SyncError {
    reason: QuotaExceeded,
    unsent: Vec<String>,
}

impl<'a> NoteSync<'a> {
    fn new(server: &'a mut Server) -> NoteSync<'a> {
        NoteSync {
            server,
            pending: Vec::new(),
        }
    }

    fn edit(&mut self, note: &str) {
        self.pending.push(note.to_string());
    }

    /// 모은 메모를 올리고 결과를 돌려줍니다. 이 메서드가 "해제"의 정식 경로입니다.
    fn finish(mut self) -> Result<usize, SyncError> {
        // pending을 꺼내고 빈 Vec을 남깁니다. 이 뒤에 실행되는 Drop은 할 일이 없습니다.
        let pending = mem::take(&mut self.pending);
        match self.server.upload(&pending) {
            Ok(()) => Ok(pending.len()),
            Err(reason) => Err(SyncError {
                reason,
                unsent: pending,
            }),
        }
    }
}

impl Drop for NoteSync<'_> {
    fn drop(&mut self) {
        if self.pending.is_empty() {
            return; // finish를 불렀거나 올릴 것이 없음
        }
        // finish 없이 버려졌습니다. 최선을 다해 올려 보지만 결과를 돌려줄 수는 없습니다.
        // 실제 코드라면 로그를 남기는 자리입니다. 이 예제는 관찰을 위해 출력합니다.
        let outcome = match self.server.upload(&self.pending) {
            Ok(()) => "성공",
            Err(_) => "실패(에러는 여기서 사라짐)",
        };
        println!(
            "    [안전망] finish 없이 버려짐: {}개 올리기 {outcome}",
            self.pending.len()
        );
    }
}

/// 입력 줄을 메모로 받아 올립니다. 빈 줄을 만나면 ?로 일찍 반환합니다.
fn sync_lines(server: &mut Server, lines: &[&str]) -> Result<usize, String> {
    let mut sync = NoteSync::new(server);
    for (i, line) in lines.iter().enumerate() {
        let note = Some(line.trim())
            .filter(|s| !s.is_empty())
            .ok_or(format!("{}번째 줄이 비어 있음", i + 1))?; // finish 없이 반환
        sync.edit(note);
    }
    sync.finish().map_err(|e| format!("{e:?}"))
}

fn main() {
    println!("1. 비권장: Drop에서만 올림 (할당량 2개)");
    let mut server = Server::new(2);
    {
        let mut sync = AutoSync {
            server: &mut server,
            pending: Vec::new(),
        };
        for note in EDITS {
            sync.edit(note);
        }
    } // 여기서 Drop이 올리기를 시도하고 실패합니다. 블록이 끝나야 server를 다시 읽을 수 있습니다.
    println!(
        "    편집 {}개, 서버에 저장된 메모 {}개 — 실패를 알려 준 곳이 없음",
        EDITS.len(),
        server.notes.len()
    );
    // 출력:     편집 3개, 서버에 저장된 메모 0개 — 실패를 알려 준 곳이 없음

    println!("2. 비권장: E0509를 clone으로 피한 finish (할당량 10개)");
    let mut server = Server::new(10);
    let mut sync = CloneSync {
        server: &mut server,
        pending: Vec::new(),
    };
    for note in EDITS {
        sync.edit(note);
    }
    let result = sync.finish();
    println!(
        "    finish 결과 {result:?}, 서버에 저장된 메모 {}개",
        server.notes.len()
    );
    // 출력:     finish 결과 Ok(3), 서버에 저장된 메모 6개

    println!("3. 권장: Result를 돌려주는 finish, Drop은 안전망");
    let mut server = Server::new(10);
    let mut sync = NoteSync::new(&mut server);
    for note in EDITS {
        sync.edit(note);
    }
    println!("    할당량 10개: finish → {:?}", sync.finish());
    // 출력:     할당량 10개: finish → Ok(3)
    println!("    서버에 저장된 메모 {}개", server.notes.len());
    // 출력:     서버에 저장된 메모 3개

    let mut server = Server::new(2);
    let mut sync = NoteSync::new(&mut server);
    for note in EDITS {
        sync.edit(note);
    }
    let mut kept_on_device: Vec<String> = Vec::new();
    match sync.finish() {
        Ok(n) => println!("    할당량 2개: {n}개 올림"),
        Err(e) => {
            println!(
                "    할당량 2개: 실패 (요청 {}개, 남은 자리 {}개)",
                e.reason.requested, e.reason.available
            );
            kept_on_device.extend(e.unsent); // 에러가 돌려준 메모를 기기에 보관합니다.
        }
    }
    // 출력:     할당량 2개: 실패 (요청 3개, 남은 자리 2개)
    println!("    기기에 보관한 메모 {kept_on_device:?}");
    // 출력:     기기에 보관한 메모 ["우유 사기", "치과 예약", "택배 찾기"]

    let mut server = Server::new(10);
    let result = sync_lines(&mut server, &["우유 사기", "치과 예약", "  ", "택배 찾기"]);
    // 출력:     [안전망] finish 없이 버려짐: 2개 올리기 성공
    println!("    ?로 일찍 반환: {result:?}");
    // 출력:     ?로 일찍 반환: Err("3번째 줄이 비어 있음")
    println!("    서버에 저장된 메모 {:?}", server.notes);
    // 출력:     서버에 저장된 메모 ["우유 사기", "치과 예약"]
}
