// 이 예제가 보여 주는 것: Hash/Hasher/BuildHasher. 규칙 "a == b이면 hash(a) == hash(b)"를
//                        손으로 구현할 때 지키는 법(대소문자를 무시하는 키), 어기면 생기는 일,
//                        with_hasher로 해시 함수를 바꾸는 법
// 실행: cargo run -p p1-ch6-core-traits --example 07_hash
//
// 역할 분담:
//   Hash        — 값이 "어떤 바이트들을 해셔에 넣을지" 정합니다. (키 타입이 구현)
//   Hasher      — 넣은 바이트들로 u64 해시를 계산합니다. (해시 알고리즘이 구현)
//   BuildHasher — 키마다 새 Hasher를 만들어 줍니다. (HashMap의 세 번째 타입 매개변수 S)
// 해시 값 자체는 실행마다(RandomState) 또는 버전마다 달라질 수 있어서 출력하지 않고, "같은가"만 출력합니다.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher, RandomState};

// ---------- 1. 대소문자를 무시하는 태그: Eq와 Hash를 같은 기준으로 ----------
#[derive(Debug)]
struct Tag(String);

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Eq for Tag {}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // eq가 "Rust"와 "rust"를 같다고 하므로, 해시에도 소문자로 바꾼 바이트를 넣어야 합니다.
        // 길이를 먼저 넣는 것은 여러 값을 이어 해시할 때 경계가 섞이지 않게 하기 위해서입니다.
        state.write_usize(self.0.len());
        for b in self.0.bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
    }
}

// ---------- 2. 규칙을 어긴 타입: derive(Hash) + 손으로 쓴 PartialEq ----------
#[expect(
    clippy::derived_hash_with_manual_eq,
    reason = "규칙을 어기면 무슨 일이 생기는지 보여 주려고 일부러 섞었습니다"
)]
#[derive(Debug, Hash)] // 해시는 대소문자를 구분하는 원래 문자열로 계산됩니다.
struct BadTag(String);

impl PartialEq for BadTag {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0) // 비교는 대소문자를 무시합니다.
    }
}

impl Eq for BadTag {}

// ---------- 3. 직접 만든 해시 함수: FNV-1a ----------
// 알고리즘이 짧아 예로 들기 좋습니다. 대신 HashDoS 공격을 막지 못하므로
// 외부 입력을 키로 쓰는 맵에는 기본값(RandomState, SipHash 1-3)을 그대로 두는 것이 안전합니다.
struct Fnv1a(u64);

impl Default for Fnv1a {
    fn default() -> Self {
        Fnv1a(0xcbf2_9ce4_8422_2325) // FNV-1a의 시작값(offset basis)
    }
}

impl Hasher for Fnv1a {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3); // FNV prime
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

// Default를 구현한 Hasher는 BuildHasherDefault로 감싸면 BuildHasher가 됩니다.
type FnvBuildHasher = BuildHasherDefault<Fnv1a>;

fn main() {
    let state = RandomState::new(); // 해시 값을 비교할 때 쓸 BuildHasher

    // 1. derive(PartialEq, Eq, Hash)는 모든 필드를 같은 방식으로 다루므로 규칙이 저절로 지켜집니다.
    println!("== 1. derive한 Hash ==");
    #[derive(PartialEq, Eq, Hash)]
    struct Point {
        x: i32,
        y: i32,
    }
    let p = Point { x: 1, y: 2 };
    let q = Point { x: 1, y: 2 };
    println!(
        "p == q? {}, 해시 같음? {}",
        p == q,
        state.hash_one(&p) == state.hash_one(&q)
    ); // 출력: p == q? true, 해시 같음? true

    // 2. 같은 기준으로 손으로 구현한 Tag: 집합이 대소문자만 다른 태그를 하나로 셉니다.
    println!("\n== 2. 대소문자를 무시하는 키 ==");
    let input = ["Rust", "rust", "WASM", "CLI", "wasm", "RUST"];
    let tags: HashSet<Tag> = input.iter().map(|s| Tag(s.to_string())).collect();
    let mut kept: Vec<&str> = tags.iter().map(|t| t.0.as_str()).collect();
    kept.sort(); // 집합의 순회 순서는 실행마다 다르므로 정렬해서 출력합니다.
    println!("입력 {}개 → 서로 다른 태그 {}개", input.len(), tags.len()); // 출력: 입력 6개 → 서로 다른 태그 3개
    // 이미 같은 값이 있으면 insert는 원래 값을 그대로 두므로, 처음 나온 표기가 남습니다.
    println!("{kept:?}"); // 출력: ["CLI", "Rust", "WASM"]
    println!("wAsM이 있나? {}", tags.contains(&Tag("wAsM".to_string()))); // 출력: wAsM이 있나? true

    // 3. 규칙을 어긴 BadTag: ==는 같다고 하는데 해시가 다릅니다.
    //    HashSet은 해시로 찾을 자리를 정하므로, "rust"를 찾으러 "Rust"와 다른 자리로 갑니다.
    //    그래서 contains가 false를 내거나 같은 태그가 두 번 들어갈 수 있고, 그 결과는 실행마다 달라질 수 있습니다.
    println!("\n== 3. 규칙을 어기면 ==");
    let a = BadTag("Rust".to_string());
    let b = BadTag("rust".to_string());
    println!("a == b? {}", a == b); // 출력: a == b? true
    println!("해시 같음? {}", state.hash_one(&a) == state.hash_one(&b)); // 출력: 해시 같음? false

    // 4. 해시에 넣는 바이트열은 경계가 구분되어야 합니다(prefix-free).
    //    &str의 Hash는 끝에 구분 바이트를 하나 더 넣어 ("ab", "c")와 ("a", "bc")를 구별합니다.
    println!("\n== 4. 경계가 섞이지 않게 ==");
    let same = state.hash_one(("ab", "c")) == state.hash_one(("a", "bc"));
    println!("(\"ab\", \"c\")와 (\"a\", \"bc\")의 해시가 같은가? {same}"); // 출력: ("ab", "c")와 ("a", "bc")의 해시가 같은가? false

    // 5. with_hasher: HashMap<K, V, S>의 S를 바꿉니다. 사용법(insert, get)은 그대로입니다.
    println!("\n== 5. with_hasher로 해시 함수 바꾸기 ==");
    let mut stock: HashMap<&str, u32, FnvBuildHasher> =
        HashMap::with_hasher(FnvBuildHasher::default());
    stock.insert("사과", 12);
    stock.insert("배", 5);
    *stock.entry("사과").or_default() += 3;
    println!("사과 {:?}, 귤 {:?}", stock.get("사과"), stock.get("귤")); // 출력: 사과 Some(15), 귤 None

    // RandomState는 맵마다 시드가 달라 같은 내용이어도 순회 순서가 다를 수 있지만,
    // 시드가 없는 FNV는 같은 순서로 넣은 두 맵의 순회 순서가 같습니다.
    let names = ["가온", "나래", "다솜", "라온", "마루", "바다"];
    let mut m1: HashMap<&str, usize, FnvBuildHasher> = HashMap::default();
    let mut m2: HashMap<&str, usize, FnvBuildHasher> = HashMap::default();
    for (i, n) in names.iter().enumerate() {
        m1.insert(n, i);
        m2.insert(n, i);
    }
    let same_order = m1.keys().eq(m2.keys());
    println!("FNV 맵 두 개의 순회 순서가 같은가? {same_order}"); // 출력: FNV 맵 두 개의 순회 순서가 같은가? true
}
