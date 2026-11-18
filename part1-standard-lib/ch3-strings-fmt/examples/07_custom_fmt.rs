// 이 예제가 보여 주는 것: 직접 구현한 Display가 너비·정렬을 무시하는 이유와 고치는 방법
//                        (Formatter의 pad, width, precision, alternate, 안쪽 값에 그대로 넘기기),
//                        다른 서식 트레이트(LowerHex) 구현, Debug를 도우미(debug_struct, debug_list)로 직접 구현하기
// 실행: cargo run -p p1-ch3-strings-fmt --example 07_custom_fmt
//
// {:>10}의 너비·정렬 같은 서식 지정은 Formatter(f)에 담겨 fmt 메서드로 전달됩니다.
// 그 값을 읽고 적용하는 것은 fmt를 구현한 사람의 몫입니다. 읽지 않으면 그냥 무시됩니다.

use std::fmt;

// 1. write!만 쓴 Display: Part 0 ch10에서 본 모양입니다.
struct NaiveVersion(u32, u32, u32);

impl fmt::Display for NaiveVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 이 write!는 "v{}.{}.{}"라는 새 서식 문자열로 f에 씁니다.
        // 그 안의 {}에는 너비가 없으므로, 바깥에서 요청한 {:>10}은 어디에도 쓰이지 않습니다.
        write!(f, "v{}.{}.{}", self.0, self.1, self.2)
    }
}

// 2. pad로 고친 Display
struct Version(u32, u32, u32);

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 서식 지정이 없으면 임시 String을 만들 필요가 없으므로 바로 씁니다.
        if f.width().is_none() && f.precision().is_none() {
            return write!(f, "v{}.{}.{}", self.0, self.1, self.2);
        }
        // 완성된 문자열을 먼저 만든 뒤, pad가 채움·정렬·너비·정밀도(최대 글자 수)를 적용해 씁니다.
        let text = format!("v{}.{}.{}", self.0, self.1, self.2);
        f.pad(&text)
    }
}

// 3. 안쪽 값에 Formatter를 그대로 넘기기: 감싼 값의 Display::fmt를 같은 f로 부르면
// 너비·정밀도·부호 같은 서식 지정이 모두 안쪽 값에 적용됩니다.
struct Meters(f64);

impl fmt::Display for Meters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)?; // self.0.fmt(f)라고 써도 같습니다.
        f.write_str(" m") // 서식 없이 그대로 덧붙입니다.
    }
}

// 4. 다른 서식 트레이트 구현: {:x}는 LowerHex 트레이트를 부릅니다.
// alternate()는 # 플래그가 붙었는지 알려 줍니다. 여기서는 {:#x}일 때 CSS처럼 '#'을 붙이기로 정했습니다.
struct Rgb(u8, u8, u8);

impl fmt::LowerHex for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = if f.alternate() { "#" } else { "" };
        let text = format!("{prefix}{:02x}{:02x}{:02x}", self.0, self.1, self.2);
        f.pad(&text)
    }
}

// 5. Debug를 직접 구현하기: 비밀번호처럼 디버그 출력에 드러나면 안 되는 필드가 있을 때
// derive(Debug)는 모든 필드를 출력하므로, 도우미로 보여 줄 필드만 고릅니다.
struct Account {
    id: u32,
    owner: String,
    password: String,
}

impl Account {
    fn check(&self, input: &str) -> bool {
        self.password == input
    }
}

impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Account")
            .field("id", &self.id)
            .field("owner", &self.owner)
            .finish_non_exhaustive() // 생략한 필드가 있다는 뜻으로 ".."을 붙입니다.
    }
}

// 감싼 타입을 목록처럼 보이게: debug_list에 원소를 넘깁니다.
struct Playlist(Vec<String>);

impl fmt::Debug for Playlist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

fn main() {
    // 1, 2. write!만 쓴 구현과 pad를 쓴 구현
    println!("== 1. write!만 쓰면 서식 지정이 무시됩니다 ==");
    let naive = NaiveVersion(1, 98, 1);
    let version = Version(1, 98, 1);
    println!("[{naive:>10}] [{naive:-^12}]"); // 출력: [v1.98.1] [v1.98.1]
    println!("[{version:>10}] [{version:-^12}]"); // 출력: [   v1.98.1] [--v1.98.1---]
    println!("[{version}] [{version:.3}]"); // 출력: [v1.98.1] [v1.]  ← pad는 정밀도를 최대 글자 수로 봅니다.

    println!("\n== 2. 표로 비교 ==");
    let rows = [("rustc", (1, 98, 1)), ("clippy", (0, 1, 98))];
    for (tool, (a, b, c)) in rows {
        println!("{tool:<8}|{:>10}|", NaiveVersion(a, b, c));
    }
    // 출력:
    // rustc   |v1.98.1|
    // clippy  |v0.1.98|
    for (tool, (a, b, c)) in rows {
        println!("{tool:<8}|{:>10}|", Version(a, b, c));
    }
    // 출력:
    // rustc   |   v1.98.1|
    // clippy  |   v0.1.98|

    // 3. 안쪽 값에 넘기기
    println!("\n== 3. 안쪽 값에 Formatter 넘기기 ==");
    let d = Meters(12.345);
    println!("[{d}] [{d:.1}] [{d:+.2}]"); // 출력: [12.345 m] [12.3 m] [+12.35 m]
    // 너비도 안쪽 숫자에만 적용됩니다. " m"까지 포함해 정렬하려면 2번처럼 pad를 써야 합니다.
    println!("[{d:>8.1}]"); // 출력: [    12.3 m]

    // 4. LowerHex 구현
    println!("\n== 4. LowerHex 구현 ==");
    let orange = Rgb(255, 128, 0);
    println!("{orange:x} {orange:#x} [{orange:>10x}]"); // 출력: ff8000 #ff8000 [    ff8000]
    // UpperHex는 구현하지 않았으므로 {orange:X}는 컴파일 에러(E0277)입니다.

    // 5. Debug 직접 구현
    println!("\n== 5. Debug 도우미 ==");
    let account = Account {
        id: 7,
        owner: String::from("페리스"),
        password: String::from("hunter2"),
    };
    println!("{account:?}"); // 출력: Account { id: 7, owner: "페리스", .. }
    // 도우미를 쓰면 {:#?}의 여러 줄 출력도 따로 구현하지 않아도 됩니다.
    println!("{account:#?}");
    // 출력:
    // Account {
    //     id: 7,
    //     owner: "페리스",
    //     ..
    // }
    println!("비밀번호 확인: {}", account.check("hunter2")); // 출력: 비밀번호 확인: true
    let list = Playlist(vec![String::from("산책"), String::from("Coffee Break")]);
    println!("{list:?}"); // 출력: ["산책", "Coffee Break"]
}
