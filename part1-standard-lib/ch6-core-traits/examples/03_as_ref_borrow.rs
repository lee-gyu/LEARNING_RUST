// 이 예제가 보여 주는 것: AsRef/AsMut로 여러 타입을 받는 함수, ?Sized와 함께 쓰기,
//                        Borrow 덕분에 HashMap<String, V>를 &str로 찾을 수 있는 이유,
//                        직접 만든 키 타입에 Borrow를 구현해도 되는 경우와 안 되는 경우
// 실행: cargo run -p p1-ch6-core-traits --example 03_as_ref_borrow
//
// AsRef<T>와 Borrow<T>는 모두 "&self에서 &T를 싸게 얻는다"는 같은 모양의 메서드를 가집니다.
// 차이는 약속입니다. Borrow<T>는 "빌린 &T의 Eq·Ord·Hash 결과가 원래 값과 똑같다"고 약속하고,
// AsRef<T>는 아무것도 약속하지 않습니다. 그래서 해시 맵의 조회처럼 그 약속에 기대는 곳은 Borrow를 씁니다.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{BuildHasher, RandomState};

// AsRef<str>: &str, String, &String 등 "&str을 꺼낼 수 있는 것"을 모두 받습니다.
fn initial<S: AsRef<str>>(name: S) -> Option<char> {
    name.as_ref().chars().next()
}

// 참조로 받을 때는 ?Sized를 붙여야 S = str(크기를 모르는 타입)도 받습니다(Part 0 ch19).
fn shout<S: AsRef<str> + ?Sized>(text: &S) -> String {
    text.as_ref().to_uppercase() + "!"
}

// AsRef<[u32]>: Vec, 배열, 슬라이스를 모두 받습니다.
fn total<T: AsRef<[u32]>>(prices: T) -> u32 {
    prices.as_ref().iter().sum()
}

// AsMut<[u8]>: 가변 슬라이스를 꺼낼 수 있는 것을 받아 내용을 고칩니다.
fn wipe<T: AsMut<[u8]>>(buffer: &mut T) {
    for b in buffer.as_mut() {
        *b = 0;
    }
}

// 필드가 둘인 타입. 주소 문자열을 꺼내 쓰도록 AsRef<str>는 구현하지만 Borrow<str>는 구현하지 않습니다.
// derive(Hash)는 두 필드를 모두 해시하므로, Email의 해시는 주소 문자열 하나의 해시와 다르기 때문입니다.
#[derive(Debug, PartialEq, Eq, Hash)]
struct Email {
    address: String,
    verified: bool,
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.address
    }
}

// 문자열 하나를 감싼 키 타입. derive(Hash, Eq)가 안쪽 String의 결과를 그대로 쓰므로
// UserId의 비교·해시는 str과 같습니다. 그래서 Borrow<str>를 구현해도 약속이 지켜집니다.
#[derive(Debug, PartialEq, Eq, Hash)]
struct UserId(String);

impl Borrow<str> for UserId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

fn main() {
    // 1. AsRef<str>: 호출하는 쪽이 무엇을 가지고 있든 그대로 넘깁니다.
    println!("== 1. AsRef<str> ==");
    let owned = String::from("지영");
    let email = Email {
        address: String::from("minsu@example.com"),
        verified: true,
    };
    println!("{:?}", initial("민수")); // 출력: Some('민')
    println!("{:?}", initial(&owned)); // 출력: Some('지')
    println!("{:?}", initial(owned)); // 출력: Some('지')  (String을 넘기면 함수가 소유하고 끝나면 버립니다)
    println!("{:?}", initial(&email)); // 출력: Some('m')  (&Email도 AsRef<str>: 참조에 대한 포괄 구현)
    println!("인증됨? {}", email.verified); // 출력: 인증됨? true

    // 2. ?Sized 덕분에 &str을 그대로 넘길 수 있습니다(S = str).
    println!("\n== 2. ?Sized와 함께 ==");
    let name = String::from("rust");
    println!("{} {}", shout("hello"), shout(&name)); // 출력: HELLO! RUST!

    // 3. AsRef<[u32]>와 AsMut<[u8]>
    println!("\n== 3. 슬라이스로 보기 ==");
    let cart = vec![1200, 3500, 800];
    let fixed = [1000, 2000];
    println!("{} {} {}", total(&cart), total(fixed), total(&cart[1..])); // 출력: 5500 3000 4300
    let mut secret = vec![7_u8, 4, 2, 9];
    let mut pin = [1_u8, 2, 3, 4];
    wipe(&mut secret);
    wipe(&mut pin);
    println!("{secret:?} {pin:?}"); // 출력: [0, 0, 0, 0] [0, 0, 0, 0]

    // 4. HashMap<String, u32>를 &str로 찾기.
    //    get의 시그니처: fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V> where K: Borrow<Q>, Q: Hash + Eq
    //    K = String, Q = str이고 String: Borrow<str>이므로, 찾을 때 String을 새로 만들지 않아도 됩니다.
    println!("\n== 4. Borrow와 해시 맵 조회 ==");
    let mut scores: HashMap<String, u32> = HashMap::new();
    scores.insert(String::from("민수"), 90);
    scores.insert(String::from("지영"), 85);
    let who: &str = "지영";
    println!("{who}: {:?}", scores.get(who)); // 출력: 지영: Some(85)

    // 5. 직접 만든 키 UserId도 Borrow<str> 덕분에 &str로 찾을 수 있습니다.
    println!("\n== 5. 직접 만든 키와 Borrow ==");
    let mut last_login: HashMap<UserId, &str> = HashMap::new();
    last_login.insert(UserId(String::from("u-001")), "11-17");
    last_login.insert(UserId(String::from("u-002")), "11-18");
    println!("u-002: {:?}", last_login.get("u-002")); // 출력: u-002: Some("11-18")

    // 6. Borrow가 약속하는 것: 빌린 값의 해시가 원래 값의 해시와 같아야 합니다.
    //    해시 값은 실행마다 달라지므로(RandomState), 값 대신 "같은가"만 출력합니다.
    println!("\n== 6. Borrow의 약속 확인 ==");
    let state = RandomState::new();
    let id = UserId(String::from("u-001"));
    let same_id = state.hash_one(&id) == state.hash_one("u-001");
    println!("hash(UserId) == hash(str)? {same_id}"); // 출력: hash(UserId) == hash(str)? true
    let same_email = state.hash_one(&email) == state.hash_one(email.as_ref());
    println!("hash(Email) == hash(주소)? {same_email}"); // 출력: hash(Email) == hash(주소)? false
    // Email에 Borrow<str>를 구현했다면, 맵은 Email의 해시로 자리를 정해 넣고
    // 주소 문자열의 해시로 찾으므로 엉뚱한 자리를 뒤지게 됩니다. AsRef만 구현한 이유입니다.
}
