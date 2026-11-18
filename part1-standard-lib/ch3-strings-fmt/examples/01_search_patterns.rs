// 이 예제가 보여 주는 것: 문자열 안에서 찾기(contains, starts_with, ends_with, find, rfind,
//                        matches, match_indices)와, 검색 패턴 자리에 넣을 수 있는 것들
//                        (char, &str, &String, char 배열, 클로저, char::is_* 함수)
// 실행: cargo run -p p1-ch3-strings-fmt --example 01_search_patterns
//
// 찾기 메서드는 모두 &self를 받고 원본을 바꾸지 않습니다. 돌려주는 것은 bool, 바이트 위치(usize),
// 또는 원본을 빌린 조각(&str)을 하나씩 내주는 반복자이므로 새 String을 만들지 않습니다.

fn main() {
    let line = "2025-03-14 ERROR [db] 연결 실패: timeout=30s";

    // 1. 들어 있는가: bool
    println!("== 1. 들어 있는가: bool ==");
    println!("contains(\"ERROR\")   = {}", line.contains("ERROR")); // 출력: contains("ERROR")   = true
    println!("starts_with(\"2025\") = {}", line.starts_with("2025")); // 출력: starts_with("2025") = true
    println!("ends_with(\"ms\")     = {}", line.ends_with("ms")); // 출력: ends_with("ms")     = false

    // 2. 어디에 있는가: Option<usize>
    // 돌려주는 위치는 "몇 번째 글자"가 아니라 "몇 번째 바이트"입니다. 한글은 글자마다 3바이트이므로
    // "실패" 앞에는 글자가 25개 있지만, 그중 한글 두 글자가 6바이트를 차지해서 위치는 29입니다.
    println!("\n== 2. 어디에 있는가: Option<usize> (바이트 위치) ==");
    println!("find(\"실패\") = {:?}", line.find("실패")); // 출력: find("실패") = Some(29)
    println!("find(\"WARN\") = {:?}", line.find("WARN")); // 출력: find("WARN") = None
    println!("find('-')    = {:?}", line.find('-')); // 출력: find('-')    = Some(4)
    println!("rfind('-')   = {:?}", line.rfind('-')); // 출력: rfind('-')   = Some(7)

    // find가 돌려준 위치는 언제나 글자 경계이므로 그 위치로 잘라도 패닉하지 않습니다.
    // ':'은 1바이트 글자라서 콜론 다음 위치는 i + 1입니다(일반적으로는 i + c.len_utf8()).
    if let Some(i) = line.find(':') {
        println!("콜론 앞: {:?}", &line[..i]); // 출력: 콜론 앞: "2025-03-14 ERROR [db] 연결 실패"
        println!("콜론 뒤: {:?}", &line[i + 1..]); // 출력: 콜론 뒤: " timeout=30s"
    }

    // 3. 모두 찾기: matches는 찾은 조각(&str)을, match_indices는 (바이트 위치, 조각)을 내주는 반복자입니다.
    // 반복자는 게으르므로 count()나 collect()로 소비해야 실제로 찾습니다(Part 0 ch13).
    println!("\n== 3. 모두 찾기: matches, match_indices ==");
    let dash_count = line.matches('-').count();
    println!("'-'의 개수 = {dash_count}"); // 출력: '-'의 개수 = 2
    let dashes: Vec<(usize, &str)> = line.match_indices('-').collect();
    println!("'-'의 위치 = {dashes:?}"); // 출력: '-'의 위치 = [(4, "-"), (7, "-")]

    // 4. 검색 패턴: 위 메서드들의 매개변수 타입은 P: Pattern이라는 제네릭입니다.
    // Pattern을 구현한 타입이면 무엇이든 넣을 수 있습니다.
    println!("\n== 4. 검색 패턴 자리에 올 수 있는 것 ==");

    // char 하나
    println!("char        '['   → {:?}", line.find('[')); // 출력: char        '['   → Some(17)
    // &str: 부분 문자열 전체가 일치해야 합니다.
    println!("&str        \"db\"  → {:?}", line.find("db")); // 출력: &str        "db"  → Some(18)
    // &String도 됩니다. String을 그대로(이동) 넣는 것이 아니라 빌려서 넣습니다.
    let needle = String::from("timeout");
    println!("&String     {needle:?} → {:?}", line.find(&needle)); // 출력: &String     "timeout" → Some(37)
    // char 배열: 배열 안의 글자 중 "아무거나" 하나와 일치하면 됩니다.
    println!("[char; 2]   ['[', ':'] → {:?}", line.find(['[', ':'])); // 출력: [char; 2]   ['[', ':'] → Some(17)
    // 클로저: char를 받아 bool을 돌려주는 클로저가 true인 글자를 찾습니다.
    // 매개변수 타입(c: char)을 적어 줘야 컴파일러가 어느 Pattern 구현인지 고를 수 있습니다.
    let first_upper = line.find(|c: char| c.is_ascii_uppercase());
    println!("클로저      대문자 → {first_upper:?}"); // 출력: 클로저      대문자 → Some(11)
    // char::is_* 함수도 "char를 받아 bool을 돌려주는 것"이므로 이름만 넘기면 됩니다.
    println!(
        "함수        char::is_whitespace → {:?}",
        line.find(char::is_whitespace)
    ); // 출력: 함수        char::is_whitespace → Some(10)

    // 같은 검색 패턴을 matches에 넣으면 조건에 맞는 글자들을 하나씩 꺼낼 수 있습니다.
    // &str 조각들을 collect로 이어 붙여 String 하나로 모읍니다.
    let digits: String = line.matches(|c: char| c.is_ascii_digit()).collect();
    println!("숫자만 모으기 = {digits}"); // 출력: 숫자만 모으기 = 2025031430
    let hangul_count = line.matches(|c: char| ('가'..='힣').contains(&c)).count();
    println!("한글 음절 개수 = {hangul_count}"); // 출력: 한글 음절 개수 = 4

    // 5. 검색은 대소문자를 구분합니다. 무시하려면 양쪽을 같은 대소문자로 바꿔서 비교합니다.
    // to_lowercase()는 새 String을 만들므로(03 예제), 짧은 ASCII 비교라면 할당이 없는
    // eq_ignore_ascii_case가 낫습니다.
    println!("\n== 5. 대소문자 ==");
    println!("contains(\"error\") = {}", line.contains("error")); // 출력: contains("error") = false
    println!(
        "to_lowercase().contains(\"error\") = {}",
        line.to_lowercase().contains("error")
    ); // 출력: to_lowercase().contains("error") = true
    let level = &line[11..16];
    println!(
        "{level:?}.eq_ignore_ascii_case(\"error\") = {}",
        level.eq_ignore_ascii_case("error")
    ); // 출력: "ERROR".eq_ignore_ascii_case("error") = true
}
