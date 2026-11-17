// Result<T, E>는 성공(Ok)과 실패(Err) 중 하나를 담는 열거형입니다.
// 실패할 수 있는 함수는 Result를 돌려주고, 호출한 쪽은 match로 두 경우를 모두 처리합니다.
//
// 실행: cargo run -p p0-ch9-error-handling --example 02_result_basics

use std::fs::File;
use std::io;
use std::num::ParseIntError;

fn main() {
    println!("1. parse는 Result<i32, ParseIntError>를 돌려줍니다");
    for input in ["42", "사십이", ""] {
        // 반환 타입을 적어 보면 성공했을 때의 값(T = i32)과 실패했을 때의 값(E = ParseIntError)이 보입니다.
        let result: Result<i32, ParseIntError> = input.parse();
        match result {
            Ok(n) => println!("    {input:?} → Ok, 값: {n}"),
            Err(e) => println!("    {input:?} → Err, 이유: {e}"),
        }
    }
    // 출력:
    //     "42" → Ok, 값: 42
    //     "사십이" → Err, 이유: invalid digit found in string
    //     "" → Err, 이유: cannot parse integer from empty string

    println!("2. Result를 그대로 {{:?}}로 출력하면 배리언트와 안의 값이 함께 보입니다");
    println!("    {:?}", "200".parse::<u8>());
    println!("    {:?}", "300".parse::<u8>());
    // 출력:
    //     Ok(200)
    //     Err(ParseIntError { kind: PosOverflow })
    // u8은 0~255만 담을 수 있으므로 300은 "양수 쪽으로 넘침(PosOverflow)"입니다.

    println!("3. String::from_utf8은 바이트가 올바른 UTF-8일 때만 String을 만듭니다");
    let good = vec![236, 149, 136, 235, 133, 149]; // "안녕"의 UTF-8 바이트 (ch8)
    let broken = vec![236, 149, 136, 235, 133]; // '녕'의 마지막 바이트가 빠졌습니다.
    for bytes in [good, broken] {
        // from_utf8은 Vec<u8>의 소유권을 받습니다. 성공하면 같은 버퍼를 그대로 String으로 씁니다.
        match String::from_utf8(bytes) {
            Ok(text) => println!("    Ok: {text}"),
            Err(e) => println!("    Err: {e}"),
        }
    }
    // 출력:
    //     Ok: 안녕
    //     Err: incomplete utf-8 byte sequence from index 3

    println!("4. File::open은 Result<File, io::Error>를 돌려줍니다");
    // 임시 디렉터리 안의, 아무도 만들지 않는 파일 이름입니다. 어디서 실행해도 결과가 같습니다.
    let path = std::env::temp_dir().join("p0_ch9_02_missing.txt");
    let result: Result<File, io::Error> = File::open(&path);
    match result {
        Ok(_) => println!("    파일을 열었습니다"),
        Err(e) => {
            println!("    기본 형식:   {e}");
            println!("    디버그 형식: {e:?}");
            println!("    종류(kind):  {:?}", e.kind());
        }
    }
    // 출력 (macOS에서 확인. 메시지와 번호는 운영체제가 정하므로 Windows에서는 다릅니다):
    //     기본 형식:   No such file or directory (os error 2)
    //     디버그 형식: Os { code: 2, kind: NotFound, message: "No such file or directory" }
    //     종류(kind):  NotFound
}
