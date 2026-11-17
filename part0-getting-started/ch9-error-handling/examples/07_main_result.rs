// main도 Result를 돌려줄 수 있습니다. 그러면 main 안에서 ?를 쓸 수 있고,
// main이 Err를 돌려주면 에러를 출력하고 종료 코드 1로 끝납니다(패닉의 101과 다릅니다).
//
// 주의: 이 예제는 3번 구역에서 main이 **일부러 Err를 돌려주며 끝납니다**(종료 코드 1).
//
// 실행: cargo run -p p0-ch9-error-handling --example 07_main_result
// 종료 코드 확인: 위 명령 바로 뒤에 echo $?
//
// 파일은 임시 디렉터리(std::env::temp_dir())에 만들고, 끝나기 전에 지웁니다.

use std::error::Error;
use std::fs;
use std::path::Path;

// 파일에서 개수를 읽습니다. 실패할 수 있는 곳이 두 군데이고, 에러 타입도 서로 다릅니다.
//   fs::read_to_string → io::Error
//   parse              → ParseIntError
// 반환 타입의 에러 자리를 io::Error로 정하면 ParseIntError에 ?를 쓸 수 없습니다(README 4.3절).
// Box<dyn Error>는 "어떤 종류의 에러든 담을 수 있는 타입"이라서 두 에러를 모두 받아 줍니다.
// ?가 각 에러를 Box<dyn Error>로 바꿔 돌려줍니다. dyn과 트레이트 객체는 ch17에서 배웁니다.
fn read_count(path: &Path) -> Result<u32, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let count: u32 = text.trim().parse()?;
    Ok(count)
}

// 반환 타입이 Result<(), Box<dyn Error>>인 main입니다.
// 성공하면 돌려줄 의미 있는 값이 없으므로 Ok에 유닛 ()를 담습니다.
fn main() -> Result<(), Box<dyn Error>> {
    let dir = std::env::temp_dir();
    let good = dir.join("p0_ch9_07_good.txt");
    let bad = dir.join("p0_ch9_07_bad.txt");
    let missing = dir.join("p0_ch9_07_missing.txt");

    // main이 Result를 돌려주므로, 준비 단계의 실패도 expect 대신 ?로 main 밖에 알릴 수 있습니다.
    fs::write(&good, "12\n")?;
    fs::write(&bad, "열두 개\n")?;

    println!("1. 서로 다른 두 종류의 에러를 Box<dyn Error> 하나로 돌려줍니다");
    println!("    good    → {:?}", read_count(&good));
    println!("    bad     → {:?}", read_count(&bad));
    println!("    missing → {:?}", read_count(&missing));
    // 출력 (missing의 메시지는 macOS에서 확인한 것입니다):
    //     good    → Ok(12)
    //     bad     → Err(ParseIntError { kind: InvalidDigit })
    //     missing → Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })

    println!("2. main 안에서도 ?로 값을 꺼낼 수 있습니다");
    let count = read_count(&good)?;
    println!("    count = {count}");
    // 출력: count = 12

    // Result는 값이므로 변수에 담아 두었다가 나중에 ?를 써도 됩니다.
    // ?로 main을 빠져나가기 전에 임시 파일부터 지우려고 이렇게 순서를 잡았습니다.
    let result = read_count(&bad);
    fs::remove_file(&good)?;
    fs::remove_file(&bad)?;

    println!("3. ?가 Err를 만나면 main이 그 Err를 돌려주며 끝납니다");
    let count = result?;
    println!("    이 줄은 실행되지 않습니다: {count}");
    Ok(())
    // 출력 (표준 에러):
    //     Error: ParseIntError { kind: InvalidDigit }
    // main이 Err(e)를 돌려주면 Rust가 "Error: "와 e의 디버그 형식을 출력하고 종료 코드 1로 끝냅니다.
}
