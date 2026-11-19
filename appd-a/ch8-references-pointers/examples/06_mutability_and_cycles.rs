// 지침 6: 소유 방식을 정한 다음, 필요할 때만 내부 가변성과 Weak를 더합니다.
//
//   ③ 공유한 값을 고쳐야 하는가? → 고칠 칸에만 내부 가변성을 둡니다.
//      값 전체를 RefCell로 감싸면, 누군가 읽는 동안에는 관계없는 칸도 고칠 수 없습니다.
//   ④ 순환이 생기는가? → 소유 방향을 한쪽으로 정하고, 거꾸로 가리키는 쪽을 Weak로 둡니다.
//
// 예제 05의 두 질문(소유자가 하나인가, 스레드를 넘는가)에 이어지는 예제입니다.
// 모두 한 스레드 안의 상황이라 Rc·Cell·RefCell을 씁니다.
//
// 실행: cargo run -p er-ch8-references-pointers --example 06_mutability_and_cycles

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

// ---------- ③ 내부 가변성을 어디에 둘까 ----------

/// 비권장 쪽에서 쓰는 카드: 목록 전체를 RefCell로 감쌉니다.
struct Card {
    title: &'static str,
    likes: u32,
}

/// 권장 쪽에서 쓰는 카드: 바뀌는 칸(좋아요 수)만 Cell입니다.
struct LiveCard {
    title: &'static str,
    likes: Cell<u32>,
}

// ---------- ④ 순환이 생기는가 ----------

/// 사진을 소유하는 앨범. 사진 목록은 앨범을 만든 뒤에 채우므로 RefCell에 둡니다.
struct Album<P> {
    name: &'static str,
    photos: RefCell<Vec<Rc<P>>>,
}

impl<P> Drop for Album<P> {
    fn drop(&mut self) {
        println!("    [앨범 '{}' 해제]", self.name);
    }
}

/// 비권장: 사진이 자기 앨범을 Rc로 가리킵니다. 앨범 → 사진 → 앨범으로 순환이 생깁니다.
struct StrongPhoto {
    album: Rc<Album<StrongPhoto>>,
}

/// 권장: 사진은 앨범을 Weak로 가리킵니다. 앨범을 살려 두지 않습니다.
struct WeakPhoto {
    album: Weak<Album<WeakPhoto>>,
}

fn main() {
    println!("1. 비권장: 목록 전체를 RefCell로 감싼 Rc<RefCell<Vec<Card>>>");
    let gallery = Rc::new(RefCell::new(vec![
        Card {
            title: "등대",
            likes: 0,
        },
        Card {
            title: "노을",
            likes: 0,
        },
    ]));
    // 좋아요 버튼의 처리기도 갤러리를 함께 소유합니다(소유자가 여럿 → Rc).
    let handler = Rc::clone(&gallery);
    let like = move |i: usize| -> Result<u32, String> {
        let mut cards = handler.try_borrow_mut().map_err(|e| e.to_string())?;
        cards[i].likes += 1;
        Ok(cards[i].likes)
    };
    {
        // 화면을 그리는 동안 목록을 빌리고 있습니다. 그 사이에 "노을"의 좋아요가 눌립니다.
        let cards = gallery.borrow();
        let titles: Vec<&str> = cards.iter().map(|c| c.title).collect();
        println!("    그리는 중 {titles:?}, 좋아요 결과: {:?}", like(1));
    }
    // 출력:     그리는 중 ["등대", "노을"], 좋아요 결과: Err("RefCell already borrowed")
    println!("    다 그린 뒤 좋아요 결과: {:?}", like(1));
    // 출력:     다 그린 뒤 좋아요 결과: Ok(1)

    println!("2. 권장: 바뀌는 칸만 Cell로 둔 Rc<Vec<LiveCard>>");
    let gallery = Rc::new(vec![
        LiveCard {
            title: "등대",
            likes: Cell::new(0),
        },
        LiveCard {
            title: "노을",
            likes: Cell::new(0),
        },
    ]);
    let handler = Rc::clone(&gallery);
    let like = move |i: usize| -> u32 {
        handler[i].likes.update(|n| n + 1);
        handler[i].likes.get()
    };
    {
        // 목록을 빌려 읽는 동안에도 좋아요 칸은 바꿀 수 있습니다. 실행 중 대여 검사도 없습니다.
        let titles: Vec<&str> = gallery.iter().map(|c| c.title).collect();
        println!("    그리는 중 {titles:?}, 좋아요 결과: {}", like(1));
    }
    // 출력:     그리는 중 ["등대", "노을"], 좋아요 결과: 1

    println!("3. 비권장: 사진이 앨범을 Rc로 가리키면 순환이 생깁니다");
    let album = Rc::new(Album {
        name: "여행",
        photos: RefCell::new(Vec::new()),
    });
    let photo = Rc::new(StrongPhoto {
        album: Rc::clone(&album),
    });
    println!("    사진이 속한 앨범: {:?}", photo.album.name);
    // 출력:     사진이 속한 앨범: "여행"
    album.photos.borrow_mut().push(photo);
    let watch = Rc::downgrade(&album); // 관찰용. 앨범을 살려 두지 않습니다.
    drop(album); // 앨범 핸들을 버려도 사진이 앨범을 붙잡고 있습니다.
    println!(
        "    핸들을 버린 뒤: 앨범을 붙잡은 강한 참조 {}개, 아직 살아 있는가: {}",
        watch.strong_count(),
        watch.upgrade().is_some()
    );
    // 출력:     핸들을 버린 뒤: 앨범을 붙잡은 강한 참조 1개, 아직 살아 있는가: true
    // 이 앨범과 사진은 프로그램이 끝날 때까지 해제되지 않습니다(메모리 누수). 해제 메시지도 나오지 않습니다.

    println!("4. 권장: 거꾸로 가리키는 쪽(사진 → 앨범)을 Weak로 둡니다");
    let album = Rc::new(Album {
        name: "가족",
        photos: RefCell::new(Vec::new()),
    });
    let photo = Rc::new(WeakPhoto {
        album: Rc::downgrade(&album),
    });
    album.photos.borrow_mut().push(Rc::clone(&photo));
    println!(
        "    사진이 속한 앨범: {:?}",
        photo.album.upgrade().map(|a| a.name)
    );
    // 출력:     사진이 속한 앨범: Some("가족")
    drop(album);
    // 출력:     [앨범 '가족' 해제]
    println!(
        "    핸들을 버린 뒤 사진이 속한 앨범: {:?}",
        photo.album.upgrade().map(|a| a.name)
    );
    // 출력:     핸들을 버린 뒤 사진이 속한 앨범: None
}
