//! 지침 5(이어서): 인덱스는 삭제 뒤에 어긋난다. 삭제를 설계에 넣는다. (세대 ID는 책에 없는 지침)
//!
//! 05의 인덱스판에서 사용자가 탈퇴합니다. `Vec::remove`로 지우면 뒤의 원소가 당겨져,
//! 방에 저장해 둔 번호가 다른 사람을 가리킵니다(비권장). 컴파일러도 clippy도 알려 주지 않습니다.
//! 칸을 지우지 않고 비워 두며 칸마다 세대(generation) 번호를 붙인 ID를 쓰면(권장),
//! 탈퇴한 사람의 옛 ID를 알아채고, 그 칸을 새 사용자가 다시 써도 헷갈리지 않습니다.
//!
//! 실행: cargo run -p er-ch15-borrow-checker --example 06_stale_index

#[derive(Debug)]
struct User {
    nickname: String,
}

fn user(nickname: &str) -> User {
    User {
        nickname: nickname.to_string(),
    }
}

// ---- 비권장: 05의 인덱스판 그대로, 탈퇴는 Vec::remove ----

fn plain_line(users: &[User], title: &str, members: &[usize]) -> String {
    // 05처럼 users[i]로 쓰면 범위를 벗어난 번호에서 패닉하므로 get으로 확인합니다.
    let names: Vec<String> = members
        .iter()
        .map(|&i| match users.get(i) {
            Some(u) => u.nickname.clone(),
            None => format!("(없는 번호 {i})"),
        })
        .collect();
    format!("{title}: {}", names.join(", "))
}

// ---- 권장: 칸을 비워 두고, 칸마다 세대를 셉니다 ----

/// 칸 번호와 그 칸의 몇 번째 주인인지를 함께 담은 ID. Copy라서 인덱스처럼 가볍습니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UserId {
    slot: usize,
    generation: u32,
}

struct Slot {
    generation: u32,
    user: Option<User>, // 탈퇴하면 None. 칸 자체는 지우지 않으므로 다른 칸의 번호가 바뀌지 않음
}

#[derive(Default)]
struct Directory {
    slots: Vec<Slot>,
    free: Vec<usize>, // 다시 쓸 수 있는 빈 칸
}

impl Directory {
    fn join(&mut self, nickname: &str) -> UserId {
        if let Some(slot) = self.free.pop() {
            let s = &mut self.slots[slot];
            s.user = Some(user(nickname));
            return UserId {
                slot,
                generation: s.generation,
            };
        }
        self.slots.push(Slot {
            generation: 0,
            user: Some(user(nickname)),
        });
        UserId {
            slot: self.slots.len() - 1,
            generation: 0,
        }
    }

    fn leave(&mut self, id: UserId) -> Option<User> {
        let s = self.slots.get_mut(id.slot)?;
        if s.generation != id.generation {
            return None; // 이미 다른 사람의 칸
        }
        let gone = s.user.take()?;
        s.generation += 1; // 이 칸을 가리키던 옛 ID는 이제 맞지 않음
        self.free.push(id.slot);
        Some(gone)
    }

    fn get(&self, id: UserId) -> Option<&User> {
        let s = self.slots.get(id.slot)?;
        if s.generation == id.generation {
            s.user.as_ref()
        } else {
            None
        }
    }

    fn line(&self, title: &str, members: &[UserId]) -> String {
        let names: Vec<&str> = members
            .iter()
            .map(|&id| match self.get(id) {
                Some(u) => u.nickname.as_str(),
                None => "(탈퇴한 사용자)",
            })
            .collect();
        format!("{title}: {}", names.join(", "))
    }
}

fn main() {
    println!("[비권장] Vec::remove로 탈퇴 처리한 인덱스판");
    let mut users = vec![user("하늘"), user("바다"), user("산들바람")];
    let hiking = vec![0, 2]; // 하늘, 산들바람
    let reading = vec![1, 0]; // 바다, 하늘
    users.remove(1); // 바다 탈퇴. 뒤의 산들바람이 1번으로 당겨짐
    println!("  바다 탈퇴 뒤");
    println!("    {}", plain_line(&users, "등산 모임", &hiking));
    println!("    {}", plain_line(&users, "독서 모임", &reading));
    // 출력:     등산 모임: 하늘, (없는 번호 2)   ← 산들바람이 사라짐
    // 출력:     독서 모임: 산들바람, 하늘       ← 들어온 적 없는 사람
    users.push(user("별")); // 새 사용자가 2번이 됨
    println!("  별 가입 뒤");
    println!("    {}", plain_line(&users, "등산 모임", &hiking));
    println!("    {}", plain_line(&users, "독서 모임", &reading));
    // 출력:     등산 모임: 하늘, 별             ← 새 사용자가 남의 방에
    // 출력:     독서 모임: 산들바람, 하늘

    println!("[권장] 세대를 붙인 ID");
    let mut dir = Directory::default();
    let sky = dir.join("하늘");
    let sea = dir.join("바다");
    let hill = dir.join("산들바람");
    let hiking = vec![sky, hill];
    let mut reading = vec![sea, sky];
    let left = dir.leave(sea).map(|u| u.nickname);
    println!("  {left:?} 탈퇴 뒤");
    println!("    {}", dir.line("등산 모임", &hiking));
    println!("    {}", dir.line("독서 모임", &reading));
    // 출력:   Some("바다") 탈퇴 뒤
    // 출력:     등산 모임: 하늘, 산들바람
    // 출력:     독서 모임: (탈퇴한 사용자), 하늘
    let star = dir.join("별");
    println!("  별 가입 뒤: 별의 ID {star:?}, 바다의 옛 ID {sea:?}");
    println!("    {}", dir.line("등산 모임", &hiking));
    println!("    {}", dir.line("독서 모임", &reading));
    // 출력:   별 가입 뒤: 별의 ID UserId { slot: 1, generation: 1 }, 바다의 옛 ID UserId { slot: 1, generation: 0 }
    // 출력:     (방 두 개는 위와 같음. 별이 바다의 칸을 다시 썼지만 옛 ID와 세대가 다름)
    println!(
        "  옛 ID로 다시 탈퇴 시도: {:?}",
        dir.leave(sea).map(|u| u.nickname)
    );
    // 출력:   옛 ID로 다시 탈퇴 시도: None
    reading.retain(|&id| dir.get(id).is_some()); // 방 명단 정리
    println!("    정리 뒤 {}", dir.line("독서 모임", &reading));
    // 출력:     정리 뒤 독서 모임: 하늘
}
