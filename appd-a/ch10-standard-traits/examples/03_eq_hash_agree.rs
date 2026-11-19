// 지침: PartialEq를 손으로 구현했다면 Hash도 같은 기준으로 손으로 구현합니다.
//       "a == b이면 hash(a) == hash(b)"가 깨지면 HashSet/HashMap이 넣은 값을 찾지 못합니다.
// 비권장: 같음은 id로만 판단(손으로 쓴 PartialEq), 해시는 derive(모든 필드)
// 권장:   PartialEq와 Hash 모두 id만 봄
//
// 즐겨찾기한 카페를 HashSet에 넣어 두었는데, 다시 불러온 같은 카페는 리뷰 수가 늘어 있습니다.
//
// 실행: cargo run -p er-ch10-standard-traits --example 03_eq_hash_agree
//
// 결과가 실행마다 같도록 시드가 고정된 해셔(DefaultHasher::default)로 집합을 만듭니다.
// 기본 해셔(RandomState)는 집합마다 시드가 달라서, 비권장 판의 조회 결과가 실행마다 달라질 수 있습니다.

use std::collections::HashSet;
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hash};

type FixedState = BuildHasherDefault<DefaultHasher>;

// ---- 비권장 ------------------------------------------------------------------

mod naive {
    #[derive(Debug, Clone)]
    // derive(Hash)는 모든 필드(id, name, reviews)를 해셔에 넣습니다.
    // clippy의 derived_hash_with_manual_eq(기본 deny)가 이 조합을 에러로 막으므로,
    // 무슨 일이 생기는지 보이려고 예외로 표시했습니다.
    #[expect(
        clippy::derived_hash_with_manual_eq,
        reason = "PartialEq와 Hash가 어긋나면 생기는 일을 보이려고 일부러 섞음"
    )]
    #[derive(Hash)]
    pub struct Cafe {
        pub id: u32,
        pub name: &'static str,
        pub reviews: u32,
    }

    // 같은 카페인지는 id로만 판단합니다. 리뷰 수는 계속 바뀌는 상태일 뿐입니다.
    impl PartialEq for Cafe {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl Eq for Cafe {}
}

// ---- 권장 --------------------------------------------------------------------

mod fixed {
    use std::hash::{Hash, Hasher};

    #[derive(Debug, Clone)]
    pub struct Cafe {
        pub id: u32,
        pub name: &'static str,
        pub reviews: u32,
    }

    impl PartialEq for Cafe {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl Eq for Cafe {}

    // eq가 보는 것(id)만 해셔에 넣습니다. eq가 무시하는 필드를 넣으면 약속이 깨집니다.
    impl Hash for Cafe {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.id.hash(state);
        }
    }
}

fn same_hash<T: Hash>(a: &T, b: &T) -> bool {
    let state = FixedState::default();
    state.hash_one(a) == state.hash_one(b)
}

fn main() {
    println!("[비권장] PartialEq는 id만, Hash는 모든 필드");
    let saved = naive::Cafe {
        id: 31,
        name: "골목 커피",
        reviews: 128,
    };
    let mut favorites: HashSet<naive::Cafe, FixedState> = HashSet::default();
    favorites.insert(saved.clone());
    let fresh = naive::Cafe {
        reviews: 131,
        ..saved.clone()
    };
    println!(
        "  저장한 값: {} 리뷰 {}개, 다시 불러온 값: {} 리뷰 {}개",
        saved.name, saved.reviews, fresh.name, fresh.reviews
    );
    // 출력:   저장한 값: 골목 커피 리뷰 128개, 다시 불러온 값: 골목 커피 리뷰 131개
    println!("  saved == fresh? {}", saved == fresh);
    // 출력:   saved == fresh? true
    println!("  해시가 같은가? {}", same_hash(&saved, &fresh));
    // 출력:   해시가 같은가? false
    println!("  즐겨찾기에 있나? {}", favorites.contains(&fresh));
    // 출력:   즐겨찾기에 있나? false
    favorites.insert(fresh);
    println!("  없다길래 넣었더니 즐겨찾기 {}개", favorites.len());
    // 출력:   없다길래 넣었더니 즐겨찾기 2개

    println!("[권장] PartialEq와 Hash 모두 id만");
    let saved = fixed::Cafe {
        id: 31,
        name: "골목 커피",
        reviews: 128,
    };
    let mut favorites: HashSet<fixed::Cafe, FixedState> = HashSet::default();
    favorites.insert(saved.clone());
    let fresh = fixed::Cafe {
        reviews: 131,
        ..saved.clone()
    };
    println!("  saved == fresh? {}", saved == fresh);
    // 출력:   saved == fresh? true
    println!("  해시가 같은가? {}", same_hash(&saved, &fresh));
    // 출력:   해시가 같은가? true
    println!("  즐겨찾기에 있나? {}", favorites.contains(&fresh));
    // 출력:   즐겨찾기에 있나? true
    // 이미 같은 값이 있으면 insert는 false를 돌려주고 원래 값을 그대로 둡니다.
    let added = favorites.insert(fresh);
    println!("  다시 넣기: {added}, 즐겨찾기 {}개", favorites.len());
    // 출력:   다시 넣기: false, 즐겨찾기 1개
    if let Some(kept) = favorites.iter().next() {
        println!("  집합에 남은 값: {} 리뷰 {}개", kept.name, kept.reviews);
    }
    // 출력:   집합에 남은 값: 골목 커피 리뷰 128개
}
