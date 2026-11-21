//! 지침 2: 구조체의 필드는 비공개로 두고 메서드로 연다. 그래야 내부 표현을 바꿀 수 있다.
//!
//! 재생 큐 `PlayQueue`의 첫 판은 곡을 `Vec`에 담습니다. 맨 앞 곡을 꺼내는 `Vec::remove(0)`은
//! 남은 곡을 모두 한 칸씩 당기므로, 둘째 판은 앞에서 꺼내기가 싼 `VecDeque`로 바꾸려 합니다.
//! - 비권장 `queue_pub_v1`: 필드가 `pub`입니다. 사용자 코드가 `songs.push`, `songs.remove(0)`을
//!   직접 씁니다. 필드 타입을 `VecDeque`로 바꾸면 이 사용자 코드가 세 군데에서 컴파일되지 않습니다
//!   (E0308 두 번, E0599 한 번. README 2.3절).
//! - 권장 `queue_v1` → `queue_v2`: 필드는 비공개이고 메서드 네 개만 공개합니다. 내부 표현을
//!   `Vec`에서 `VecDeque`로 바꿔도 사용자 코드(`listen_v1`, `listen_v2`)는 `use` 줄만 다르고
//!   본문이 같습니다.
//!
//! 실행: cargo run -p er-ch22-minimize-visibility --example 02_private_fields

const SETLIST: [&str; 4] = ["아침 산책", "비 오는 오후", "퇴근길", "한밤의 라디오"];

// ---- 비권장: 필드를 공개한 첫 판 ----

pub mod queue_pub_v1 {
    pub struct PlayQueue {
        pub songs: Vec<String>, // 표현(Vec)이 곧 공개 API가 됩니다.
    }
}

// ---- 권장: 필드를 감춘 첫 판(Vec)과 둘째 판(VecDeque) ----

pub mod queue_v1 {
    #[derive(Default)]
    pub struct PlayQueue {
        songs: Vec<String>,
    }

    impl PlayQueue {
        pub fn new() -> PlayQueue {
            PlayQueue::default()
        }

        pub fn add(&mut self, title: &str) {
            self.songs.push(title.to_string());
        }

        /// 맨 앞 곡을 꺼냅니다. `Vec::remove(0)`은 남은 원소를 모두 앞으로 옮깁니다.
        pub fn play_next(&mut self) -> Option<String> {
            if self.songs.is_empty() {
                None
            } else {
                Some(self.songs.remove(0))
            }
        }

        /// 대기 중인 곡 제목. 반환 타입이 `Vec`의 반복자라고 약속하지 않으므로 표현을 바꿀 수 있습니다.
        pub fn upcoming(&self) -> impl Iterator<Item = &str> {
            self.songs.iter().map(String::as_str)
        }
    }
}

pub mod queue_v2 {
    use std::collections::VecDeque;

    #[derive(Default)]
    pub struct PlayQueue {
        songs: VecDeque<String>, // 표현만 바꿨습니다. 공개 메서드의 시그니처는 그대로입니다.
    }

    impl PlayQueue {
        pub fn new() -> PlayQueue {
            PlayQueue::default()
        }

        pub fn add(&mut self, title: &str) {
            self.songs.push_back(title.to_string());
        }

        /// `VecDeque::pop_front`는 남은 원소를 옮기지 않습니다.
        pub fn play_next(&mut self) -> Option<String> {
            self.songs.pop_front()
        }

        pub fn upcoming(&self) -> impl Iterator<Item = &str> {
            self.songs.iter().map(String::as_str)
        }
    }
}

// ---- 사용자 코드 ----

/// 공개 필드에 기댄 사용자 코드. `Vec`의 메서드를 직접 부릅니다.
fn listen_pub_v1() -> String {
    use queue_pub_v1::PlayQueue;

    let mut q = PlayQueue { songs: Vec::new() };
    for title in SETLIST {
        q.songs.push(title.to_string());
    }
    let first = q.songs.remove(0);
    let second = q.songs.remove(0);
    q.songs.push("주말 드라이브".to_string());
    format!("재생: {first} → {second} / 대기: {}", q.songs.join(", "))
}

/// 공개 메서드만 쓰는 사용자 코드(첫 판).
fn listen_v1() -> String {
    use queue_v1::PlayQueue;

    let mut q = PlayQueue::new();
    for title in SETLIST {
        q.add(title);
    }
    let first = q.play_next().unwrap_or_default();
    let second = q.play_next().unwrap_or_default();
    q.add("주말 드라이브");
    let upcoming: Vec<&str> = q.upcoming().collect();
    format!("재생: {first} → {second} / 대기: {}", upcoming.join(", "))
}

/// 공개 메서드만 쓰는 사용자 코드(둘째 판). `use` 줄 말고는 `listen_v1`과 같습니다.
fn listen_v2() -> String {
    use queue_v2::PlayQueue;

    let mut q = PlayQueue::new();
    for title in SETLIST {
        q.add(title);
    }
    let first = q.play_next().unwrap_or_default();
    let second = q.play_next().unwrap_or_default();
    q.add("주말 드라이브");
    let upcoming: Vec<&str> = q.upcoming().collect();
    format!("재생: {first} → {second} / 대기: {}", upcoming.join(", "))
}

fn main() {
    println!("1. 비권장: 필드가 pub인 첫 판(Vec)");
    // 결과는 같지만, 이 사용자 코드는 필드 타입이 Vec이라는 사실에 기대고 있습니다.
    println!("    {}", listen_pub_v1());

    println!("2. 권장: 필드를 감추고 메서드로 연다");
    let v1 = listen_v1();
    let v2 = listen_v2();
    println!("    첫 판(Vec):         {v1}");
    println!("    둘째 판(VecDeque):  {v2}");
    println!("    두 판의 결과가 같음: {}", v1 == v2); // 출력: 두 판의 결과가 같음: true
}
