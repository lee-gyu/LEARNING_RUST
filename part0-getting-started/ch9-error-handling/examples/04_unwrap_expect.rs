// unwrap과 expect는 Ok이면 안의 값을 꺼내고, Err이면 패닉합니다.
// 패닉하지 않고 값을 정하는 unwrap_or, unwrap_or_else와 비교합니다.
//
// 주의: 이 예제는 5번 구역에서 **일부러 패닉하며 끝납니다**(종료 코드 101).
// expect에 넘긴 메시지와 에러 값이 패닉 메시지에 어떻게 나오는지 보기 위해서입니다.
//
// 실행: cargo run -p p0-ch9-error-handling --example 04_unwrap_expect
// 종료 코드 확인: 위 명령 바로 뒤에 echo $?

use std::fs::File;

fn main() {
    println!("1. Ok이면 unwrap은 안의 값을 꺼냅니다");
    let a: i32 = "42".parse().unwrap();
    println!("    a = {a}");
    // 출력: a = 42
    // "abc"였다면 다음 메시지로 패닉합니다(에러 값은 디버그 형식으로 붙습니다).
    //     called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }

    println!("2. expect는 unwrap과 같되, 패닉할 때 보여 줄 메시지를 직접 정합니다");
    // 메시지에는 "왜 여기서는 Ok여야 하는가"를 적습니다. 패닉했을 때 무엇이 깨졌는지 바로 알 수 있습니다.
    let b: i32 = "7"
        .parse()
        .expect("코드에 직접 적은 숫자 문자열이므로 항상 변환됩니다");
    println!("    b = {b}");
    // 출력: b = 7

    println!("3. 실패하면 기본값을 쓰려면 unwrap_or를 씁니다 (패닉하지 않습니다)");
    let c: i32 = "abc".parse().unwrap_or(0);
    println!("    c = {c}");
    // 출력: c = 0

    println!("4. 에러를 보고 할 일을 정하려면 unwrap_or_else를 씁니다 (패닉하지 않습니다)");
    // |e| { ... }는 클로저(이름 없는 함수)입니다. Err일 때만 에러 값 e를 받아 실행되고,
    // 그 결과가 꺼낸 값을 대신합니다. 클로저는 ch13에서 배웁니다.
    let d: i32 = "abc".parse().unwrap_or_else(|e| {
        println!("    변환 실패({e}), 대신 -1을 씁니다");
        -1
    });
    println!("    d = {d}");
    // 출력:
    //     변환 실패(invalid digit found in string), 대신 -1을 씁니다
    //     d = -1

    println!("5. 프로그램이 동작하는 데 꼭 필요한 것이 없으면 expect로 멈춥니다");
    // 임시 디렉터리에 아무도 만들지 않는 파일 이름이므로 File::open은 Err를 돌려줍니다.
    let path = std::env::temp_dir().join("p0_ch9_04_settings.toml");
    let settings = File::open(&path).expect("설정 파일 p0_ch9_04_settings.toml이 있어야 합니다");
    println!("    이 줄은 실행되지 않습니다: {settings:?}");
    // 출력 (패닉 메시지는 표준 에러로 나갑니다):
    //
    //     thread 'main' (스레드 ID) panicked at part0-getting-started/ch9-error-handling/examples/04_unwrap_expect.rs:48:38:
    //     설정 파일 p0_ch9_04_settings.toml이 있어야 합니다: Os { code: 2, kind: NotFound, message: "No such file or directory" }
    //     note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    //
    // "expect에 넘긴 메시지: 에러의 디버그 형식" 모양입니다. 스레드 ID는 실행할 때마다 달라지고,
    // 에러 메시지는 운영체제에 따라 다릅니다(macOS에서 확인).
}
