// 같은 소스 파일을 e2018(2018 에디션)과 e2021(2021 에디션) 두 패키지가 함께 컴파일합니다.
// 두 에디션 모두에서 컴파일되지만, 여섯 구역 모두 출력이 달라집니다.
//
// 실행(projects/same_code 디렉터리에서):
//   cargo run -q -p e2018
//   cargo run -q -p e2021

// 2018 에디션으로 컴파일할 때, 이 프로그램이 일부러 2018의 동작을 보여 주느라 생기는 경고만 막습니다.
// - array_into_iter: 1번 구역의 arr.into_iter()가 2021에서 뜻이 바뀐다는 경고
// - clippy::into_iter_on_ref: 같은 줄이 사실상 .iter()라는 Clippy 경고
// - non_fmt_panics: 3번 구역의 panic!이 2021에서 서식 처리된다는 경고
// 2021 에디션에서는 이 린트들이 아무것도 찾지 못합니다.
#![allow(array_into_iter, non_fmt_panics, clippy::into_iter_on_ref)]

use std::any::Any;
use std::panic;

/// 버려질 때 이름을 출력해서 drop 시점을 보여 줍니다.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    drop: {}", self.0);
    }
}

/// TryInto가 프렐류드에 들어오기 전에 만든 사용자 트레이트라고 생각합니다.
trait Clamp {
    fn try_into(&self) -> Result<u8, String>;
}

impl Clamp for i32 {
    fn try_into(&self) -> Result<u8, String> {
        Ok((*self).clamp(0, 255) as u8)
    }
}

/// 받은 토큰 트리의 개수를 셉니다.
macro_rules! count_tt {
    () => { 0usize };
    ($first:tt $($rest:tt)*) => { 1usize + count_tt!($($rest)*) };
}

/// 어느 규칙이 입력을 받았는지 알려 줍니다. macros2018 크레이트의 classify!와 정의가 같습니다.
macro_rules! classify {
    ($p:pat) => {
        "규칙 1: 패턴 하나"
    };
    ($($t:tt)*) => {
        "규칙 2: 패턴이 아닌 토큰들"
    };
}

/// 반복자가 내놓는 항목의 타입 이름입니다.
fn item_type<I: Iterator>(_: &I) -> &'static str {
    std::any::type_name::<I::Item>()
}

/// f를 실행해 패닉을 붙잡고, 페이로드가 문자열이면 그 내용을 돌려줍니다.
fn panic_message(f: impl FnOnce() + panic::UnwindSafe) -> String {
    let payload: Box<dyn Any + Send> = panic::catch_unwind(f).expect_err("패닉해야 합니다");
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        format!("{s:?}")
    } else if let Some(s) = payload.downcast_ref::<String>() {
        format!("{s:?}")
    } else {
        String::from("(문자열이 아닌 페이로드)")
    }
}

fn main() {
    // e2018 또는 e2021입니다. cargo가 컴파일할 때 패키지 이름을 넣어 줍니다.
    println!("[{}]", env!("CARGO_PKG_NAME"));

    println!("1. 배열에 .into_iter()를 부르면");
    let arr = [1, 2, 3];
    println!("    항목 타입: {}", item_type(&arr.into_iter()));

    println!("2. 클로저가 t.0만 옮길 때의 drop 순서");
    {
        let t = (Noisy("t.0"), Noisy("t.1"));
        let consume = || drop(t.0);
        consume();
        println!("    consume() 호출 끝");
    }

    println!("3. 인자가 하나뿐인 panic!의 메시지");
    panic::set_hook(Box::new(|_| {})); // 표준 에러에 패닉 메시지를 쓰지 않습니다.
    let name = "러스트";
    println!("    (name = {name})");
    let with_name = panic_message(|| panic!("{name} 설정 없음"));
    let with_braces = panic_message(|| panic!("중괄호 {{}}"));
    let _ = panic::take_hook(); // 기본 훅으로 되돌립니다.
    println!("    panic!(\"{{name}} 설정 없음\") → {with_name}");
    println!("    panic!(\"중괄호 {{{{}}}}\")    → {with_braces}");

    println!("4. 프렐류드의 TryInto와 이름이 겹치는 메서드");
    let x: i32 = 300;
    let by_method: Result<u8, _> = x.try_into();
    println!("    x.try_into()        = {by_method:?}");
    println!("    Clamp::try_into(&x) = {:?}", Clamp::try_into(&x));

    println!("5. 접두사 모양의 토큰 수");
    println!(
        "    c\"hi\" → {}개, 'r#a → {}개",
        count_tt!(c"hi"),
        count_tt!('r#a)
    );

    println!("6. 매크로의 $p:pat에 1 | 2를 넘기면");
    println!("    이 크레이트의 classify!   → {}", classify!(1 | 2));
    println!(
        "    macros2018::classify!    → {}",
        macros2018::classify!(1 | 2)
    );
}
