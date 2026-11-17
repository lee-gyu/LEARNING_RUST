// 에러를 그 자리에서 처리하지 않고 호출한 쪽에 돌려주는 것을 "에러 전파"라고 합니다.
// 같은 일을 하는 함수를 네 단계로 줄여 가며, ? 연산자가 무엇을 대신해 주는지 확인합니다.
//   1단계: match로 직접 전파
//   2단계: ? 연산자
//   3단계: ?를 이어서 쓰기
//   4단계: 표준 라이브러리 함수 fs::read_to_string과 io::Result 별칭
//
// 실행: cargo run -p p0-ch9-error-handling --example 05_propagate
//
// 파일은 임시 디렉터리(std::env::temp_dir())에 만들고, 끝날 때 지웁니다.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

// 1단계: 실패할 수 있는 작업마다 match로 나누고, Err이면 return으로 호출한 쪽에 그대로 돌려줍니다.
// Clippy는 이 모양을 보면 ?를 쓰라고 제안합니다(clippy::question_mark).
// 2단계와 비교하려고 일부러 남긴 코드이므로 그 제안을 expect로 표시합니다.
#[expect(clippy::question_mark)]
fn read_username_match(path: &Path) -> Result<String, io::Error> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => return Err(e), // 함수를 즉시 끝내고 에러를 돌려줍니다.
    };

    let mut username = String::new();
    // read_to_string은 파일 내용을 username 뒤에 덧붙이고, 읽은 바이트 수를 Ok에 담아 돌려줍니다.
    match file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}

// 2단계: "Ok이면 값을 꺼내고, Err이면 그 에러로 return"을 ? 한 글자가 대신합니다.
fn read_username_question(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut username = String::new();
    file.read_to_string(&mut username)?;
    Ok(username) // 끝까지 왔으면 성공이므로 Ok로 감싸 돌려줍니다.
}

// 3단계: ?가 꺼낸 값에 바로 메서드를 이어 부를 수 있습니다.
fn read_username_chain(path: &Path) -> Result<String, io::Error> {
    let mut username = String::new();
    File::open(path)?.read_to_string(&mut username)?;
    Ok(username)
}

// 4단계: "파일을 열어 전부 String으로 읽기"는 흔한 일이라 표준 라이브러리에 함수가 있습니다.
// io::Result<String>은 Result<String, io::Error>의 다른 이름(별칭)입니다.
// std::io 안의 함수들은 에러 타입이 모두 io::Error라서 이 짧은 이름을 씁니다.
fn read_username_short(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

fn main() {
    let dir = std::env::temp_dir();
    let path = dir.join("p0_ch9_05_username.txt");
    let missing = dir.join("p0_ch9_05_missing.txt");

    // 예제를 준비하는 단계입니다. 임시 디렉터리에 쓰지 못하면 예제를 진행할 수 없으므로 expect로 멈춥니다.
    fs::write(&path, "러스타").expect("임시 디렉터리에 파일을 쓸 수 있어야 합니다");

    println!("1. match로 직접 전파");
    println!("    있는 파일: {:?}", read_username_match(&path));
    println!("    없는 파일: {:?}", read_username_match(&missing));

    println!("2. ? 연산자");
    println!("    있는 파일: {:?}", read_username_question(&path));
    println!("    없는 파일: {:?}", read_username_question(&missing));

    println!("3. ?를 이어서 쓰기");
    println!("    있는 파일: {:?}", read_username_chain(&path));
    println!("    없는 파일: {:?}", read_username_chain(&missing));

    println!("4. fs::read_to_string");
    println!("    있는 파일: {:?}", read_username_short(&path));
    println!("    없는 파일: {:?}", read_username_short(&missing));
    // 출력 (1~4 모두 같은 결과입니다. 에러 메시지는 macOS에서 확인한 것입니다):
    //     있는 파일: Ok("러스타")
    //     없는 파일: Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })

    println!("5. 전파된 에러를 어떻게 처리할지는 호출한 쪽이 정합니다");
    for target in [&path, &missing] {
        match read_username_question(target) {
            Ok(name) => println!("    안녕하세요, {name}님!"),
            Err(e) => println!("    사용자 이름을 읽지 못해 손님으로 진행합니다 ({e})"),
        }
    }
    // 출력:
    //     안녕하세요, 러스타님!
    //     사용자 이름을 읽지 못해 손님으로 진행합니다 (No such file or directory (os error 2))

    // 정리: 만든 파일을 지웁니다. 지우지 못해도 예제의 결과에는 영향이 없으므로 결과를 버립니다.
    let _ = fs::remove_file(&path);
}
