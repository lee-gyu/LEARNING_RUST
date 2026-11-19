//! 지침 5: 여러 곳이 같은 데이터를 가리켜야 하면 한 곳이 소유하고, 나머지는 인덱스로 가리킨다.
//!
//! 채팅 서비스의 사용자와 채팅방입니다. 한 사용자가 여러 방에 들어가 있습니다.
//! 방이 `&User`를 담으면 닉네임을 바꾸는 순간 E0502이므로, 흔히 두 가지로 우회합니다.
//! 방마다 사용자를 복제해 두는 판(비권장)과, 사용자 목록이 소유하고 방은 번호만 담는 판(권장)에서
//! 같은 닉네임 변경을 실행해 방에 보이는 이름을 비교합니다.
//!
//! 실행: cargo run -p er-ch15-borrow-checker --example 05_clone_or_index

// 출발점(컴파일되지 않음, E0502):
//
//     struct RefRoom<'a> { title: &'static str, members: Vec<&'a User> }
//
//     let hiking = RefRoom { title: "등산 모임", members: vec![&users[0], &users[2]] };
//     users[2].nickname = String::from("산들바람"); // users가 빌려진 동안 고침
//     show(&hiking);                                  // 빌린 것을 그 뒤에 씀
//
// 방이 살아 있는 동안 사용자 목록은 아무도 고칠 수 없습니다. 게다가 RefRoom<'a>는
// 사용자 목록과 한 구조체에 함께 담을 수도 없습니다(자기 참조, 08 예제).

#[derive(Debug, Clone)]
struct User {
    nickname: String,
}

fn user(nickname: &str) -> User {
    User {
        nickname: nickname.to_string(),
    }
}

/// 비권장: 방마다 `User` 사본을 담습니다. 라이프타임은 사라지지만 사본은 원본과 따로 놉니다.
struct CopyRoom {
    title: &'static str,
    members: Vec<User>,
}

/// 권장: 방은 사용자 목록의 인덱스만 담습니다. 라이프타임도 사본도 없습니다.
struct IndexRoom {
    title: &'static str,
    members: Vec<usize>,
}

/// 인덱스판에서는 사용자와 방을 한 구조체에 함께 담을 수 있습니다.
struct Chat {
    users: Vec<User>,
    rooms: Vec<IndexRoom>,
}

impl Chat {
    fn rename(&mut self, index: usize, nickname: &str) {
        self.users[index].nickname = nickname.to_string(); // 한 곳만 고치면 됨
    }

    fn room_line(&self, room: &IndexRoom) -> String {
        let names: Vec<&str> = room
            .members
            .iter()
            .map(|&i| self.users[i].nickname.as_str())
            .collect();
        format!("{}: {}", room.title, names.join(", "))
    }
}

fn main() {
    // 0: 하늘, 1: 바다, 2: 산
    let users = vec![user("하늘"), user("바다"), user("산")];

    println!("[비권장] 복제판: 방마다 User 사본");
    let mut master = users.clone();
    let rooms = [
        CopyRoom {
            title: "등산 모임",
            members: vec![master[0].clone(), master[2].clone()],
        },
        CopyRoom {
            title: "독서 모임",
            members: vec![master[1].clone(), master[0].clone()],
        },
    ];
    master[2].nickname = String::from("산들바람"); // 원본만 바뀜
    println!("  사용자 목록: {:?}", nicknames(&master));
    for room in &rooms {
        println!("  {}: {}", room.title, nicknames(&room.members).join(", "));
    }
    // 출력:   사용자 목록: ["하늘", "바다", "산들바람"]
    // 출력:   등산 모임: 하늘, 산      ← 사본은 옛 닉네임 그대로
    // 출력:   독서 모임: 바다, 하늘
    let copies: usize = rooms.iter().map(|r| r.members.len()).sum();
    println!("  방에 든 사본: {copies}개 (닉네임이 바뀌면 모두 찾아 고쳐야 함)");
    // 출력:   방에 든 사본: 4개 (닉네임이 바뀌면 모두 찾아 고쳐야 함)

    println!("[권장] 인덱스판: 사용자 목록이 소유하고 방은 번호만");
    let mut chat = Chat {
        users,
        rooms: vec![
            IndexRoom {
                title: "등산 모임",
                members: vec![0, 2],
            },
            IndexRoom {
                title: "독서 모임",
                members: vec![1, 0],
            },
        ],
    };
    chat.rename(2, "산들바람");
    println!("  사용자 목록: {:?}", nicknames(&chat.users));
    for room in &chat.rooms {
        println!("  {}", chat.room_line(room));
    }
    // 출력:   사용자 목록: ["하늘", "바다", "산들바람"]
    // 출력:   등산 모임: 하늘, 산들바람
    // 출력:   독서 모임: 바다, 하늘
}

fn nicknames(users: &[User]) -> Vec<&str> {
    users.iter().map(|u| u.nickname.as_str()).collect()
}
