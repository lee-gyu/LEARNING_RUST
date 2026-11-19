// 2021 에디션부터 panic!은 println!처럼 첫 인자를 언제나 서식 문자열로 처리합니다.
// 패닉을 catch_unwind로 붙잡아 페이로드(패닉과 함께 전달되는 값)의 타입과 내용을 확인합니다.
// 각 구역 주석의 "2018에서는"은 같은 코드를 2018 에디션으로 컴파일했을 때의 결과입니다.
//
// 실행: cargo run -p p2-ch3-rust-2021 --example 04_panic_format

use std::any::Any;
use std::panic;

/// 문자열이 아닌 페이로드의 예로 쓸 에러 코드입니다.
struct ErrorCode(u16);

/// 페이로드를 꺼내 "타입 내용" 꼴의 문자열로 만듭니다.
fn describe(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        format!("&str {s:?}")
    } else if let Some(s) = payload.downcast_ref::<String>() {
        format!("String {s:?}")
    } else if let Some(code) = payload.downcast_ref::<ErrorCode>() {
        format!("ErrorCode({})", code.0)
    } else {
        String::from("알 수 없는 타입")
    }
}

/// f를 실행해 패닉을 붙잡고 페이로드를 설명합니다.
fn catch(f: impl FnOnce() + panic::UnwindSafe) -> String {
    let payload = panic::catch_unwind(f).expect_err("이 예제의 클로저는 모두 패닉합니다");
    describe(payload.as_ref())
}

fn check_ascii(name: &str) {
    // assert!의 두 번째 인자부터도 panic!과 같은 규칙으로 서식 처리됩니다.
    assert!(name.is_ascii(), "{name}은(는) ASCII가 아닙니다");
}

fn main() {
    // 기본 훅이 표준 에러에 패닉 메시지를 쓰지 않도록, 아무것도 하지 않는 훅으로 바꿉니다.
    panic::set_hook(Box::new(|_| {}));
    let name = "러스트";

    println!("1. 서식 문자열 안의 변수 이름이 값으로 바뀝니다");
    println!("    {}", catch(|| panic!("{name} 설정을 찾을 수 없습니다")));
    // 출력:     String "러스트 설정을 찾을 수 없습니다"
    // 2018에서는: 인자가 하나뿐이면 서식 처리를 하지 않으므로 &str "{name} 설정을 찾을 수 없습니다"

    println!("2. 중괄호를 글자로 쓰려면 {{{{ }}}}로 씁니다");
    println!(
        "    {}",
        catch(|| panic!("빈 자리표시자 {{}}는 글자입니다"))
    );
    // 출력:     &str "빈 자리표시자 {}는 글자입니다"
    // 서식 인자가 없어 컴파일할 때 문자열이 정해지므로 페이로드가 String이 아니라 &str입니다.
    // 2018에서는: &str "빈 자리표시자 {{}}는 글자입니다" (중괄호 두 개가 그대로 남음)

    println!("3. 이미 만든 문자열을 그대로 쓰려면 \"{{}}\"를 앞에 둡니다");
    let template = String::from("{key}는 치환하지 않습니다");
    println!("    {}", catch(move || panic!("{}", template)));
    // 출력:     String "{key}는 치환하지 않습니다"
    // panic!(template)처럼 문자열 리터럴이 아닌 값을 하나만 넘기면 2021부터는 컴파일 에러입니다.

    println!("4. 문자열이 아닌 값은 panic_any로 넘깁니다");
    println!("    {}", catch(|| panic::panic_any(ErrorCode(404))));
    // 출력:     ErrorCode(404)
    // 2018에서는 panic!(ErrorCode(404))로도 됐지만, 2021부터는 panic_any만 됩니다.

    println!("5. assert!의 메시지도 서식 문자열입니다");
    println!("    {}", catch(|| check_ascii(name)));
    // 출력:     String "러스트은(는) ASCII가 아닙니다"
    // 2018에서는: &str "{name}은(는) ASCII가 아닙니다"

    // 기본 훅으로 되돌립니다.
    let _ = panic::take_hook();
}
