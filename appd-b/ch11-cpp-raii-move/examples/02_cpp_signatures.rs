// Q11.6 — C++ 표준 타입을 쓴 함수 시그니처를 Rust로 옮기기
//
// 아래 C++ 선언을 Rust로 옮겨 실행합니다. 괄호 안이 대응하는 Rust 타입입니다.
//
//   std::optional<std::string_view> first_word(std::string_view s);   // Option<&str>
//   std::variant<long, std::string> parse_setting(std::string_view s); // 열거형
//   std::expected<int, std::string> parse_port(std::string_view s);   // Result<u16, String> (C++23)
//   int sum(std::span<const int> xs);                                  // &[i32]
//   std::size_t adopt(std::unique_ptr<Node> node);                     // Box<Node> (소유권을 넘겨받음)
//   const std::string& inspect(const Node& node);                      // &Node → &str (빌려서 봄)
//   long share(std::shared_ptr<std::string> config);                   // Rc<String>
//
// string_view와 span이 가리키는 원본이 먼저 사라지면 C++에서는 댕글링이 되지만(ch12 `ch12-modern-cpp-enough`),
// Rust의 &str과 &[i32]는 라이프타임으로 원본보다 오래 살 수 없게 검사됩니다. Option·열거형·Result 자체는 Part 0
// ch6·ch9에서 배웠고, 여기서는 C++ 타입과의 대응만 봅니다.
//
// 실행: cargo run -p faq-ch11-cpp-raii-move --example 02_cpp_signatures

use std::rc::Rc;

/// std::variant<long, std::string>에 해당하는 열거형
enum Setting {
    Number(i64),
    Text(String),
}

fn first_word(s: &str) -> Option<&str> {
    s.split_whitespace().next()
}

fn parse_setting(s: &str) -> Setting {
    match s.parse::<i64>() {
        Ok(n) => Setting::Number(n),
        Err(_) => Setting::Text(s.to_string()),
    }
}

/// std::visit 자리: match가 모든 배리언트를 다뤘는지 컴파일러가 검사합니다.
fn describe(setting: &Setting) -> String {
    match setting {
        Setting::Number(n) => format!("수 {n}"),
        Setting::Text(t) => format!("글자 {t:?}"),
    }
}

fn parse_port(s: &str) -> Result<u16, String> {
    s.parse::<u16>()
        .map_err(|e| format!("포트가 아님: {s:?} ({e})"))
}

fn sum(xs: &[i32]) -> i32 {
    xs.iter().sum()
}

struct Node {
    name: String,
}

#[expect(
    clippy::boxed_local,
    reason = "C++의 unique_ptr<Node> 인자와 나란히 보이려고 Box로 받음. 보통은 Node로 받으면 됨"
)]
fn adopt(node: Box<Node>) -> usize {
    node.name.len()
} // 넘겨받은 Box가 여기서 해제됩니다.

fn inspect(node: &Node) -> &str {
    &node.name
}

fn share(config: Rc<String>) -> usize {
    Rc::strong_count(&config)
}

fn main() {
    println!("1. optional<string_view> → Option<&str>");
    println!(
        "  {:?}, {:?}",
        first_word("  hello rust "),
        first_word("   ")
    );
    // 출력:   Some("hello"), None

    println!("2. variant → 열거형");
    let (a, b) = (parse_setting("8080"), parse_setting("debug"));
    println!("  {}, {}", describe(&a), describe(&b));
    // 출력:   수 8080, 글자 "debug"

    println!("3. expected → Result");
    println!("  {:?}", parse_port("443"));
    println!("  {:?}", parse_port("99999"));
    // 출력:   Ok(443)
    // 출력:   Err("포트가 아님: \"99999\" (number too large to fit in target type)")

    println!("4. span<const int> → &[i32]");
    let scores = vec![90, 85, 77];
    println!("  전체 {}, 앞의 둘 {}", sum(&scores), sum(&scores[..2]));
    // 출력:   전체 252, 앞의 둘 175

    println!("5. unique_ptr·shared_ptr 인자 → Box·&·Rc");
    let root = Box::new(Node {
        name: String::from("root"),
    });
    println!("  inspect: {}", inspect(&root)); // 빌려서 봄. root는 그대로
    println!("  adopt: {}", adopt(root)); // 소유권을 넘김. 이후 root는 쓸 수 없음
    let config = Rc::new(String::from("설정"));
    println!("  share 안의 참조 수: {}", share(Rc::clone(&config)));
    println!("  share 뒤 참조 수: {}", Rc::strong_count(&config));
    // 출력:   inspect: root
    // 출력:   adopt: 4
    // 출력:   share 안의 참조 수: 2
    // 출력:   share 뒤 참조 수: 1
}
