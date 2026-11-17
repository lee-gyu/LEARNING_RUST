// 기대하는 에러: error[E0106]: missing lifetime specifier
// 핵심 메시지: this function's return type contains a borrowed value, but the signature
//             does not say whether it is borrowed from `query` or `contents`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0106_search_missing_lifetime.rs --out-dir /tmp/rust-ch12
//
// 반환 타입 Vec<&str> 안의 참조가 query와 contents 중 어느 쪽에서 빌려 온 것인지
// 시그니처에 나와 있지 않습니다. 참조 매개변수가 둘이라 라이프타임 생략 규칙으로도
// 정할 수 없으므로 컴파일러가 거부합니다.
// 고치는 방법: 반환값이 contents의 일부이므로 contents와 반환값만 'a로 묶습니다.
//   pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str>

pub fn search(query: &str, contents: &str) -> Vec<&str> {
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }

    results
}

fn main() {
    let contents = "\
Rust:
safe, fast, productive.
Pick three.";

    println!("{:?}", search("duct", contents));
}
