//! 지침 3: 핸들(무엇을 가리킬 자격을 담은 값)은 필드를 감춰, 라이브러리만 만들고 고치게 한다.
//! (책에 없는 지침. 근거는 Rust API 가이드라인 C-STRUCT-PRIVATE와 ch15의 세대 ID입니다.)
//!
//! ch15 예제 06의 세대 ID 명부를 라이브러리 모듈로 옮깁니다. 칸 번호와 세대 번호를 담은
//! `UserId`를 받은 사람만 그 사용자를 찾을 수 있어야 하고, 탈퇴한 사람의 옛 ID로는
//! 그 칸을 새로 쓰는 사람을 찾을 수 없어야 합니다.
//! - 비권장 `roster_open`: `UserId`의 필드와 명부의 칸이 `pub`입니다. 사용자 코드가
//!   (1) 옛 ID의 세대를 고쳐 새 ID를 만들고(위조), (2) 칸의 세대를 직접 되돌려,
//!   탈퇴한 바다의 ID로 새 사용자 별을 찾습니다. 별의 진짜 ID는 오히려 쓸 수 없게 됩니다.
//! - 권장 `roster`: 필드가 비공개입니다. `UserId`는 `join`만 만들고 세대는 `leave`만 올립니다.
//!   사용자 코드는 ID를 복사하고 비교할 수만 있습니다. 위 두 시도는 모듈 밖에서 컴파일되지 않습니다
//!   (구조체 리터럴은 E0451, 필드 쓰기는 E0616).
//!
//! 실행: cargo run -p er-ch22-minimize-visibility --example 03_opaque_handle

// ---- 비권장: 핸들과 칸을 모두 공개 ----

pub mod roster_open {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct UserId {
        pub slot: usize,
        pub generation: u32,
    }

    pub struct Slot {
        pub generation: u32,
        pub nickname: Option<String>,
    }

    #[derive(Default)]
    pub struct Roster {
        pub slots: Vec<Slot>,
        pub free: Vec<usize>,
    }

    impl Roster {
        pub fn join(&mut self, nickname: &str) -> UserId {
            let nickname = Some(nickname.to_string());
            if let Some(slot) = self.free.pop() {
                let s = &mut self.slots[slot];
                s.nickname = nickname;
                UserId {
                    slot,
                    generation: s.generation,
                }
            } else {
                self.slots.push(Slot {
                    generation: 0,
                    nickname,
                });
                UserId {
                    slot: self.slots.len() - 1,
                    generation: 0,
                }
            }
        }

        pub fn leave(&mut self, id: UserId) -> bool {
            match self.slots.get_mut(id.slot) {
                Some(s) if s.generation == id.generation && s.nickname.is_some() => {
                    s.nickname = None;
                    s.generation += 1;
                    self.free.push(id.slot);
                    true
                }
                _ => false,
            }
        }

        pub fn get(&self, id: UserId) -> Option<&str> {
            let s = self.slots.get(id.slot)?;
            if s.generation == id.generation {
                s.nickname.as_deref()
            } else {
                None
            }
        }
    }
}

// ---- 권장: 핸들과 칸을 감춤 ----

pub mod roster {
    /// 명부가 발급한 사용자 ID. 필드가 비공개라서 모듈 밖에서는 만들 수도, 고칠 수도 없습니다.
    /// 내부 표현(칸 번호 + 세대 번호)도 약속이 아니므로 나중에 바꿀 수 있습니다.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct UserId {
        slot: usize,
        generation: u32,
    }

    // 칸은 명부의 내부 사정이라 타입 자체를 공개하지 않습니다.
    struct Slot {
        generation: u32,
        nickname: Option<String>,
    }

    /// 불변식: `UserId`의 세대가 칸의 세대와 같을 때만 그 칸의 사용자를 가리킨다.
    /// 칸의 세대를 바꾸는 코드는 `leave` 하나뿐이다.
    #[derive(Default)]
    pub struct Roster {
        slots: Vec<Slot>,
        free: Vec<usize>,
    }

    impl Roster {
        pub fn join(&mut self, nickname: &str) -> UserId {
            let nickname = Some(nickname.to_string());
            if let Some(slot) = self.free.pop() {
                let s = &mut self.slots[slot];
                s.nickname = nickname;
                UserId {
                    slot,
                    generation: s.generation,
                }
            } else {
                self.slots.push(Slot {
                    generation: 0,
                    nickname,
                });
                UserId {
                    slot: self.slots.len() - 1,
                    generation: 0,
                }
            }
        }

        pub fn leave(&mut self, id: UserId) -> bool {
            match self.slots.get_mut(id.slot) {
                Some(s) if s.generation == id.generation && s.nickname.is_some() => {
                    s.nickname = None;
                    s.generation += 1;
                    self.free.push(id.slot);
                    true
                }
                _ => false,
            }
        }

        pub fn get(&self, id: UserId) -> Option<&str> {
            let s = self.slots.get(id.slot)?;
            if s.generation == id.generation {
                s.nickname.as_deref()
            } else {
                None
            }
        }
    }
}

fn main() {
    println!("1. 비권장: UserId의 필드와 명부의 칸이 pub");
    let mut r = roster_open::Roster::default();
    let sea = r.join("바다");
    r.join("하늘");
    r.leave(sea); // 바다가 탈퇴. 칸 0의 세대가 1이 됩니다.
    let star = r.join("별"); // 별이 칸 0을 다시 씁니다(세대 1).
    println!("    바다의 옛 ID로 조회: {:?}", r.get(sea)); // 출력: None
    println!("    별의 ID로 조회: {:?}", r.get(star)); // 출력: Some("별")

    // (1) 위조: "세대가 안 맞으니 하나 올려서 다시 찾자"
    let forged = roster_open::UserId {
        slot: sea.slot,
        generation: sea.generation + 1,
    };
    let got = r.get(forged);
    println!("    옛 ID의 세대를 고쳐 만든 ID로 조회: {got:?}");

    // (2) 내부 고치기: 칸의 세대를 바다가 받았던 값으로 되돌림
    r.slots[sea.slot].generation = sea.generation;
    let got = r.get(sea);
    println!("    칸의 세대를 되돌린 뒤 바다의 옛 ID로 조회: {got:?}");
    println!("    같은 때 별의 ID로 조회: {:?}", r.get(star));

    println!("2. 권장: 필드를 감춘 UserId와 명부");
    let mut r = roster::Roster::default();
    let sea = r.join("바다");
    r.join("하늘");
    r.leave(sea);
    let star = r.join("별");
    println!("    바다의 옛 ID로 조회: {:?}", r.get(sea)); // 출력: None
    println!("    별의 ID로 조회: {:?}", r.get(star)); // 출력: Some("별")
    println!("    옛 ID로 탈퇴 시도: {}", r.leave(sea)); // 출력: false
    let same = sea == star; // Copy와 PartialEq는 derive해 두었습니다.
    println!("    ID끼리 비교는 가능: 바다의 옛 ID == 별의 ID → {same}");
    // roster::UserId { slot: 0, generation: 1 }; // E0451: fields `slot` and `generation` of struct `roster::UserId` are private
    // let _ = sea.generation; // E0616: field `generation` of struct `roster::UserId` is private
}
