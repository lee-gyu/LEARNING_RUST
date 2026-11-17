// String을 만들고 이어 붙이는 방법을 보여 줍니다.
// `push_str`, `push`, `+` 연산자, `format!`이 각각 인자의 소유권을 어떻게 다루는지에 주목하세요.
//
// 실행: cargo run -p p0-ch8-collections --example 06_string_build

fn main() {
    println!("1. String::from과 to_string은 같은 String을 만듭니다");
    let a = String::from("안녕하세요");
    let b = "안녕하세요".to_string(); // 문자열 리터럴(&str)에서 String을 만듭니다.
    println!("    a = {a}, b = {b}, a == b: {}", a == b);
    // 출력: a = 안녕하세요, b = 안녕하세요, a == b: true
    // to_string은 {}로 출력할 수 있는 값이면 무엇에나 쓸 수 있습니다.
    let n = 42.to_string();
    println!("    n = {n}, 길이 = {}", n.len()); // 출력: n = 42, 길이 = 2

    println!("2. String은 소유하는 문자열, &str은 빌려 보는 문자열입니다");
    let owned = String::from("러스트 문자열");
    let whole: &str = &owned; // &String은 &str 자리에 자동으로 바뀌어 들어갑니다(원리는 ch14).
    let part: &str = &owned[..9]; // 한글 3글자 = 9바이트(ch4의 슬라이스)
    let literal: &str = "리터럴도 &str"; // 문자열 리터럴은 프로그램에 고정된 &str입니다.
    println!("    whole = {whole}, part = {part}, literal = {literal}");
    // 출력: whole = 러스트 문자열, part = 러스트, literal = 리터럴도 &str

    println!("3. push_str은 &str을, push는 char 하나를 덧붙입니다");
    let mut greeting = String::new(); // 빈 String
    let name = "지영";
    greeting.push_str(name);
    greeting.push_str("님, 반가워요");
    greeting.push('!'); // char는 작은따옴표
    // push_str은 인자를 빌리기만 하므로 name을 계속 쓸 수 있습니다.
    println!("    greeting = {greeting}, name = {name}");
    // 출력: greeting = 지영님, 반가워요!, name = 지영

    println!("4. + 연산자는 왼쪽 String의 소유권을 가져갑니다");
    let s1 = String::from("안녕, ");
    let s2 = String::from("세상!");
    // + 는 대략 `fn add(self, s: &str) -> String` 모양의 메서드를 부릅니다.
    // - 왼쪽(self)은 &가 없으므로 s1의 소유권이 이동합니다. s1의 버퍼에 s2의 내용을 덧붙여 돌려줍니다.
    // - 오른쪽은 &str이어야 하므로 &s2라고 씁니다. s2는 빌려주기만 하므로 계속 쓸 수 있습니다.
    let s3 = s1 + &s2;
    println!("    s3 = {s3}, s2 = {s2}"); // 출력: s3 = 안녕, 세상!, s2 = 세상!
    // s1은 이동했으므로 여기서 쓰면 E0382입니다(compile_fail/e0382_string_add_moves.rs).

    println!("5. 여러 개를 이을 때 +는 읽기 어렵고, format!이 편합니다");
    let rock = String::from("가위");
    let paper = String::from("바위");
    let scissors = String::from("보");
    let joined = rock + "-" + &paper + "-" + &scissors; // rock은 여기서 이동합니다.
    println!("    +로 이은 결과: {joined}"); // 출력: +로 이은 결과: 가위-바위-보

    // format!은 println!과 같은 서식 문자열을 쓰지만, 출력하는 대신 String을 만들어 돌려줍니다.
    // 인자를 빌리기만 하므로 어느 변수의 소유권도 가져가지 않습니다.
    let again = format!("{paper}-{scissors}-{joined}");
    println!("    format! 결과: {again}"); // 출력: format! 결과: 바위-보-가위-바위-보
    println!("    paper, scissors, joined 모두 그대로: {paper}, {scissors}, {joined}");
    // 출력: paper, scissors, joined 모두 그대로: 바위, 보, 가위-바위-보
}
