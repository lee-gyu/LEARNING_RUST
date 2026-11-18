// 이 예제가 보여 주는 것: 나누기(split 계열, split_once, split_whitespace, lines)와
//                        다듬기(trim 계열, strip_prefix/strip_suffix). 결과가 모두 원본을 빌린 &str이라는 것
// 실행: cargo run -p p1-ch3-strings-fmt --example 02_split_trim
//
// 나누기·다듬기 메서드는 문자열을 복사하지 않습니다. 원본의 "어디부터 어디까지"를 가리키는
// &str(또는 그런 &str을 내주는 반복자)을 돌려줄 뿐입니다. 그래서 빠르지만, 결과를 쓰는 동안
// 원본이 살아 있어야 합니다.

fn main() {
    // 1. split: 구분자가 나올 때마다 자릅니다.
    println!("== 1. split 계열 ==");
    // 구분자가 연달아 나오면 그 사이의 빈 조각("")도 그대로 나옵니다.
    let csv = "사과,배,,포도";
    let parts: Vec<&str> = csv.split(',').collect();
    println!("split(',')            = {parts:?}"); // 출력: split(',')            = ["사과", "배", "", "포도"]
    // 구분자로 시작하면 맨 앞에 빈 조각이 생깁니다. rsplit은 뒤에서부터 나눕니다.
    let path = "/usr/local/bin";
    let forward: Vec<&str> = path.split('/').collect();
    let backward: Vec<&str> = path.rsplit('/').collect();
    println!("split('/')            = {forward:?}"); // 출력: split('/')            = ["", "usr", "local", "bin"]
    println!("rsplit('/')           = {backward:?}"); // 출력: rsplit('/')           = ["bin", "local", "usr", ""]
    // split_terminator는 맨 끝의 구분자를 "끝 표시"로 보고 마지막 빈 조각을 만들지 않습니다.
    let stmts = "a=1;b=2;";
    let with_empty: Vec<&str> = stmts.split(';').collect();
    let terminated: Vec<&str> = stmts.split_terminator(';').collect();
    println!("split(';')            = {with_empty:?}"); // 출력: split(';')            = ["a=1", "b=2", ""]
    println!("split_terminator(';') = {terminated:?}"); // 출력: split_terminator(';') = ["a=1", "b=2"]
    // splitn(n, ..)은 최대 n조각까지만 나눕니다. 마지막 조각에는 나머지가 통째로 들어갑니다.
    let setting = "url=https://a.b/?x=1";
    let two: Vec<&str> = setting.splitn(2, '=').collect();
    println!("splitn(2, '=')        = {two:?}"); // 출력: splitn(2, '=')        = ["url", "https://a.b/?x=1"]
    // 검색 패턴은 01 예제와 같습니다. char 배열을 넣으면 여러 구분자로 한 번에 나눕니다.
    let mixed: Vec<&str> = "하나, 둘;셋".split([',', ';']).collect();
    println!("split([',', ';'])     = {mixed:?}"); // 출력: split([',', ';'])     = ["하나", " 둘", "셋"]

    // 2. split_once: 처음 나오는 구분자 하나로 두 조각을 만듭니다. 없으면 None입니다.
    // rsplit_once는 마지막 구분자를 기준으로 합니다. 확장자 떼기에 알맞습니다.
    println!("\n== 2. split_once, rsplit_once ==");
    println!("{:?}", setting.split_once('=')); // 출력: Some(("url", "https://a.b/?x=1"))
    println!("{:?}", "archive.tar.gz".rsplit_once('.')); // 출력: Some(("archive.tar", "gz"))
    println!("{:?}", "README".rsplit_once('.')); // 출력: None

    // 3. 공백과 줄 단위
    println!("\n== 3. split_whitespace, lines ==");
    let words = "  하나   둘\t셋\n넷  ";
    // split(' ')은 공백 하나하나를 구분자로 보므로 빈 조각이 많이 생기고, 탭과 줄바꿈은 자르지 못합니다.
    let by_space: Vec<&str> = words.split(' ').collect();
    println!("split(' ')         = {by_space:?}"); // 출력: split(' ')         = ["", "", "하나", "", "", "둘\t셋\n넷", "", ""]
    // split_whitespace는 연달아 있는 공백 문자(스페이스, 탭, 줄바꿈 등)를 하나로 보고, 빈 조각을 버립니다.
    let by_ws: Vec<&str> = words.split_whitespace().collect();
    println!("split_whitespace() = {by_ws:?}"); // 출력: split_whitespace() = ["하나", "둘", "셋", "넷"]
    // lines는 \n과 \r\n을 모두 줄 끝으로 보고, 줄 끝 문자는 떼고 돌려줍니다. 빈 줄은 ""로 나옵니다.
    let text = "첫 줄\r\n둘째 줄\n\n넷째 줄\n";
    let lines: Vec<&str> = text.lines().collect();
    println!("lines()            = {lines:?}"); // 출력: lines()            = ["첫 줄", "둘째 줄", "", "넷째 줄"]

    // 4. 다듬기: 앞뒤만 걷어 냅니다. 가운데는 건드리지 않습니다.
    println!("\n== 4. trim 계열 ==");
    let padded = " \t 안녕 하세요 \n";
    println!("trim()       = {:?}", padded.trim()); // 출력: trim()       = "안녕 하세요"
    println!("trim_start() = {:?}", padded.trim_start()); // 출력: trim_start() = "안녕 하세요 \n"
    println!("trim_end()   = {:?}", padded.trim_end()); // 출력: trim_end()   = " \t 안녕 하세요"
    // *_matches는 검색 패턴과 일치하는 글자를 "반복해서" 걷어 냅니다.
    println!("{:?}", "**중요**".trim_matches('*')); // 출력: "중요"
    println!("{:?}", "000120".trim_start_matches('0')); // 출력: "120"
    println!(
        "{:?}",
        "report_2024".trim_end_matches(|c: char| c.is_ascii_digit())
    ); // 출력: "report_"

    // 5. strip_prefix/strip_suffix: 접두사·접미사를 "정확히 한 번" 떼고 Option으로 알려 줍니다.
    // 없으면 None이므로 "떼었는지"를 구분할 수 있습니다. trim_start_matches는 몇 번이든 떼고 결과만 줍니다.
    println!("\n== 5. strip_prefix, strip_suffix ==");
    println!("{:?}", "v1.2.3".strip_prefix('v')); // 출력: Some("1.2.3")
    println!("{:?}", "1.2.3".strip_prefix('v')); // 출력: None
    println!("{:?}", "vv1".strip_prefix('v')); // 출력: Some("v1")
    println!("{:?}", "vv1".trim_start_matches('v')); // 출력: "1"
    println!("{:?}", "main.rs".strip_suffix(".rs")); // 출력: Some("main")
    // 접두사가 있으면 떼고, 없으면 원래대로 쓰는 흔한 모양입니다(unwrap_or는 ch2-option-result).
    let tag = "#rust";
    println!("{:?}", tag.strip_prefix('#').unwrap_or(tag)); // 출력: "rust"

    // 6. 결과는 원본을 빌린 &str입니다.
    // substr_range는 조각이 원본의 몇 번째 바이트부터 몇 번째 바이트까지를 가리키는지 알려 줍니다
    // (Rust 1.98에서 안정화). 문자열을 다시 검색하는 것이 아니라 두 &str의 메모리 위치를 비교하므로,
    // Some이 나왔다는 것은 key와 value가 line의 바이트를 그대로 가리킨다는 증거입니다.
    println!("\n== 6. 조각은 원본을 가리킵니다 ==");
    let line = String::from("name = 페리스");
    if let Some((key, value)) = line.split_once('=') {
        let (key, value) = (key.trim(), value.trim());
        println!("key = {key:?}, value = {value:?}"); // 출력: key = "name", value = "페리스"
        println!("key는 line의 {:?}", line.substr_range(key)); // 출력: key는 line의 Some(0..4)
        println!("value는 line의 {:?}", line.substr_range(value)); // 출력: value는 line의 Some(7..16)
        // 내용이 같아도 다른 곳에 있는 문자열은 line의 일부가 아닙니다.
        println!("\"name\" 리터럴은 {:?}", line.substr_range("name")); // 출력: "name" 리터럴은 None
        // 여기서 line을 바꾸거나 버리면 key, value가 가리키는 곳이 사라지므로 대여 검사기가 막습니다.
        // 조각을 원본보다 오래 보관해야 한다면 key.to_string()처럼 소유한 String으로 복사합니다.
    }
}
