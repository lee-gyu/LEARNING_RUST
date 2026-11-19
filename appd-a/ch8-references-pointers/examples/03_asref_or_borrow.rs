// 지침 3: 제네릭 매개변수의 바운드는 함수가 무엇에 기대는지 보고 `AsRef`와 `Borrow` 중에 고릅니다.
//
// - 참조를 얻어 들여다보기만 한다면 `AsRef<T>`.
//   "싸게 &T로 볼 수 있다"는 것만 약속하므로 한 타입이 여러 대상에 구현할 수 있고
//   (`String`은 str, [u8], OsStr, Path로 보입니다), `&T`에 대한 포괄 구현 덕분에 참조도 받습니다.
// - 같은 타입을 소유한 형태로도 빌린 형태로도 받고 싶다면 `Borrow<T>`.
//   모든 `T`와 `&T`가 `Borrow<T>`를 구현합니다. 대신 "빌린 형태가 원래 값과 똑같이 비교·해시된다"는
//   약속이 붙어 있어서, 다른 타입으로 보이게 하는 구현은 드뭅니다.
//
// 실행: cargo run -p er-ch8-references-pointers --example 03_asref_or_borrow

use std::borrow::Borrow;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

// ---------- 들여다보기만 하는 함수 ----------

/// 비권장: 바이트를 읽기만 하는데 `Borrow<[u8]>`를 요구합니다.
fn checksum_borrow<B: Borrow<[u8]>>(data: B) -> u8 {
    data.borrow().iter().fold(0, |acc, b| acc.wrapping_add(*b))
}

/// 권장: 바이트로 볼 수 있기만 하면 됩니다.
fn checksum<B: AsRef<[u8]>>(data: B) -> u8 {
    data.as_ref().iter().fold(0, |acc, b| acc.wrapping_add(*b))
}

/// 고정된 키로 해시값을 계산합니다. `DefaultHasher::new()`는 실행할 때마다 같은 결과를 내지만,
/// 그 값 자체는 Rust 버전에 따라 바뀔 수 있으므로 "같은가"만 비교합니다.
fn hash_of<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

// ---------- 같은 타입을 여러 형태로 받는 함수 ----------

struct Track {
    title: &'static str,
    secs: u32,
}

/// 비권장: `AsRef<Track>`은 `Track` 자신과 `&Track`에는 구현되어 있지 않습니다.
/// (`AsRef`에는 `impl<T> AsRef<T> for T` 같은 반사 구현이 없습니다.)
fn total_secs_as_ref<T: AsRef<Track>>(list: &[T]) -> u32 {
    list.iter().map(|t| t.as_ref().secs).sum()
}

/// 권장: `Track`, `&Track`, `Rc<Track>`, `Box<Track>`가 모두 `Borrow<Track>`입니다.
fn total_secs<T: Borrow<Track>>(list: &[T]) -> u32 {
    list.iter().map(|t| t.borrow().secs).sum()
}

fn main() {
    let text = "점검";
    let owned = String::from("rust");
    let bytes: Vec<u8> = vec![10, 20, 30];
    let array = [0xFF_u8; 4];

    println!("1. 비권장: Borrow<[u8]>로 받는 함수");
    // 그대로 넘길 수 있는 것은 [u8]을 "같은 키로" 빌려줄 수 있는 타입뿐입니다.
    println!(
        "    그대로: Vec<u8> {}, [u8; 4] {}, &[u8] {}",
        checksum_borrow(bytes.clone()),
        checksum_borrow(array),
        checksum_borrow(&bytes[1..])
    );
    // 출력:     그대로: Vec<u8> 60, [u8; 4] 252, &[u8] 50
    // &str, String, &Vec<u8>은 E0277이라 부르는 쪽이 바이트 슬라이스로 바꿔서 넘겨야 합니다.
    println!(
        "    바꿔서: &str {}, String {}, &Vec<u8> {}",
        checksum_borrow(text.as_bytes()),
        checksum_borrow(owned.as_bytes()),
        checksum_borrow(bytes.as_slice())
    );
    // 출력:     바꿔서: &str 56, String 206, &Vec<u8> 60

    println!("2. 권장: AsRef<[u8]>로 받는 함수");
    // 결과는 같고, 일곱 가지를 모두 그대로 넘깁니다.
    println!(
        "    &str {}, String {}, &String {}, Vec<u8> {}",
        checksum(text),
        checksum(owned.clone()),
        checksum(&owned),
        checksum(bytes.clone())
    );
    // 출력:     &str 56, String 206, &String 206, Vec<u8> 60
    println!(
        "    &Vec<u8> {}, [u8; 4] {}, &[u8] {}",
        checksum(&bytes),
        checksum(array),
        checksum(&bytes[1..])
    );
    // 출력:     &Vec<u8> 60, [u8; 4] 252, &[u8] 50

    println!("3. String이 Borrow<[u8]>를 구현하지 않는 이유");
    // 바이트는 같아도 str과 [u8]은 해시가 다릅니다. Borrow는 "해시가 같다"고 약속해야 하므로
    // String은 [u8]로 "보일" 수는 있지만(AsRef), [u8]과 "같은 키"는 아닙니다(Borrow 불가).
    println!(
        "    \"ok\".as_bytes() == b\"ok\": {}, 해시가 같은가: {}",
        "ok".as_bytes() == b"ok",
        hash_of("ok") == hash_of(&b"ok"[..])
    );
    // 출력:     "ok".as_bytes() == b"ok": true, 해시가 같은가: false
    println!(
        "    String과 str의 해시가 같은가: {} (그래서 String: Borrow<str>)",
        hash_of(&owned) == hash_of("rust")
    );
    // 출력:     String과 str의 해시가 같은가: true (그래서 String: Borrow<str>)

    println!("4. 같은 타입을 소유한 형태로도 빌린 형태로도 받으려면 Borrow<T>");
    let library = vec![
        Track {
            title: "새벽 산책",
            secs: 214,
        },
        Track {
            title: "비 오는 오후",
            secs: 177,
        },
        Track {
            title: "퇴근길",
            secs: 245,
        },
    ];
    // 재생 목록은 라이브러리의 곡을 빌려 가리킵니다.
    let playlist: Vec<&Track> = vec![&library[0], &library[1]];
    // 공유 목록은 Rc로 곡을 함께 소유합니다.
    let shared: Vec<Rc<Track>> = vec![
        Rc::new(Track {
            title: "주말 아침",
            secs: 198,
        }),
        Rc::new(Track {
            title: "한밤의 라디오",
            secs: 314,
        }),
    ];
    let titles: Vec<&str> = playlist.iter().map(|t| t.title).collect();
    println!("    재생 목록: {titles:?}");
    // 출력:     재생 목록: ["새벽 산책", "비 오는 오후"]
    // AsRef 판은 Vec<Rc<Track>>만 받습니다. &library와 &playlist는 E0277입니다
    // (compile_fail/e0277_asref_not_reflexive.rs).
    println!(
        "    AsRef<Track> 판: 공유 목록 {}초",
        total_secs_as_ref(&shared)
    );
    // 출력:     AsRef<Track> 판: 공유 목록 512초
    println!(
        "    Borrow<Track> 판: 라이브러리 {}초, 재생 목록 {}초, 공유 목록 {}초",
        total_secs(&library),
        total_secs(&playlist),
        total_secs(&shared)
    );
    // 출력:     Borrow<Track> 판: 라이브러리 636초, 재생 목록 391초, 공유 목록 512초
}
