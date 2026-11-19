// never 타입 `!`와 never 타입 폴백(fallback)을 보여 줍니다.
//
// 실행: cargo run -p p2-ch4-rust-2024-language --example 06_never_type
//
// 폴백 타입은 원래 `()`였고, 2024 에디션에서 `!`로 바뀌었습니다. 그런데 rustc 1.100.0부터는
// 모든 에디션에서 `!`입니다. 그래서 이 예제의 출력은 2021 에디션으로 컴파일해도 같습니다
// (projects/edition_compare의 never_fallback_2021 / never_fallback_2024로 확인).

use std::any::{TypeId, type_name};
use std::convert::Infallible;

/// 클로저 `f`의 반환 타입 `R`이 무엇으로 정해졌는지 이름으로 알려 줍니다.
/// 클로저를 부르지는 않으므로 안의 `panic!()`은 실행되지 않습니다.
fn return_type_of<R, F: FnOnce() -> R>(_f: F) -> &'static str {
    type_name::<R>()
}

/// 실패할 수 있는 읽기 함수라고 칩니다. 성공하면 `T`의 기본값을 돌려줍니다.
fn load<T: Default>(ok: bool) -> Result<T, String> {
    if ok {
        Ok(T::default())
    } else {
        Err(String::from("읽기 실패"))
    }
}

/// 폴백에 기대던 코드를 고친 모양입니다.
fn load_twice() -> Result<u32, String> {
    // `load(true)?;`라고만 쓰면 T를 정할 단서가 없습니다. 폴백이 `()`이던 시절에는 T = ()로
    // 컴파일됐지만, 지금은 T = !가 되어 `!: Default`가 아니라는 E0277이 납니다
    // (compile_fail/e0277_never_type_fallback.rs).
    load::<()>(true)?; // 고친 방법 1: T를 직접 적습니다.
    let n: u32 = load(true)?; // 고친 방법 2: 결과를 받는 쪽의 타입이 T를 정해 줍니다.
    Ok(n + 1)
}

/// 절대 실패하지 않는 연산입니다. 에러 타입이 `!`이므로 `Err`는 만들어질 수 없습니다.
fn halve(x: u32) -> Result<u32, !> {
    Ok(x / 2)
}

fn main() {
    println!("1. 흐름을 끊는 표현식(타입 !)은 어떤 타입 자리에든 들어갑니다");
    for text in ["42", "사십이"] {
        let n: u32 = match text.parse() {
            Ok(n) => n,
            // `continue`의 타입은 `!`이고, u32 자리에 들어갑니다.
            Err(_) => {
                println!("    {text:?}는 숫자가 아니라 건너뜁니다");
                continue;
            }
        };
        println!("    {text:?} → {n}");
    }
    // 출력:     "42" → 42
    // 출력:     "사십이"는 숫자가 아니라 건너뜁니다

    println!("2. 폴백: 타입을 정할 단서가 없을 때 컴파일러가 고르는 타입");
    println!(
        "    || panic!()        → R = {}",
        return_type_of(|| panic!())
    );
    let flag = std::env::args().count() > 100; // 실행할 때만 알 수 있는 조건
    println!(
        "    한 갈래가 u8인 if  → R = {}",
        return_type_of(move || if flag { 1u8 } else { panic!() })
    );
    // 출력:     || panic!()        → R = !
    // 출력:     한 갈래가 u8인 if  → R = u8

    println!("3. 폴백에 기대던 코드는 타입을 직접 적어서 고칩니다");
    println!("    load_twice() = {:?}", load_twice());
    // 출력:     load_twice() = Ok(1)

    println!("4. rustc 1.100.0부터 !를 타입으로 직접 쓸 수 있습니다");
    let Ok(half) = halve(9); // Err(!)는 있을 수 없으므로 반박 불가능한 패턴입니다.
    println!("    halve(9) = {half}");
    println!(
        "    Infallible과 !는 같은 타입인가? {}",
        TypeId::of::<Infallible>() == TypeId::of::<!>()
    );
    println!(
        "    size_of::<!>() = {}, Vec<!>의 길이 = {}",
        size_of::<!>(),
        Vec::<!>::new().len()
    );
    // 출력:     halve(9) = 4
    // 출력:     Infallible과 !는 같은 타입인가? true
    // 출력:     size_of::<!>() = 0, Vec<!>의 길이 = 0
}
