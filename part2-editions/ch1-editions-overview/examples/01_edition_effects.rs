// 01_edition_effects: 이 크레이트의 에디션(2024)이 정한 동작들
//
// 이 저장소의 examples/는 모두 2024 에디션으로 컴파일됩니다(루트 Cargo.toml의
// [workspace.package] edition = "2024"). Part 0과 Part 1에서 "2021 에디션부터", "2024 에디션에서"라고
// 짚고 넘어간 동작 다섯 가지를 한곳에 모았습니다. 모두 표준 라이브러리나 컴파일러 버전이 아니라
// **이 크레이트의 에디션**이 정한 것입니다. 옛 에디션에서의 결과는 README 2.3절과
// 예제 02(02_rustc_edition_matrix)에서 확인합니다.
//
// 실행: cargo run -p p2-ch1-editions-overview --example 01_edition_effects

use std::any::type_name_of_val;
use std::cell::RefCell;

// gen은 2024 에디션의 예약 키워드라서 그대로는 함수 이름으로 쓸 수 없습니다.
// r#를 붙인 raw 식별자로 "키워드가 아니라 이름"이라고 표시합니다.
fn r#gen(seed: u32) -> u32 {
    seed.wrapping_mul(1_103_515_245).wrapping_add(12_345)
}

fn main() {
    println!("1. 배열의 into_iter()는 원소를 값으로 내놓습니다 (2021 에디션부터)");
    // 2015·2018 에디션에서 배열에 .into_iter()를 부르면 참조(&i32)를 내놓았습니다.
    let first = [10, 20, 30].into_iter().next();
    println!("    {first:?}, {}", type_name_of_val(&first)); // 출력: Some(10), core::option::Option<i32>

    println!("2. TryFrom은 use 없이 씁니다 (2021 에디션 프렐류드)");
    // 2015·2018 에디션에서는 `use std::convert::TryFrom;`이 없으면 E0599입니다.
    // 트레이트 자체는 모든 에디션에 있고, 에디션은 "자동으로 가져오는 목록"만 바꿉니다.
    println!("    u8::try_from(300) = {:?}", u8::try_from(300)); // 출력: u8::try_from(300) = Err(TryFromIntError(PosOverflow))

    println!("3. let 체인 (2024 에디션 전용 문법)");
    // 2021 에디션까지는 `let chains are only allowed in Rust 2024 or later` 에러입니다.
    let input = "42";
    if let Ok(n) = input.parse::<u32>()
        && n > 10
    {
        println!("    {n}는 10보다 큽니다"); // 출력: 42는 10보다 큽니다
    }

    println!("4. if let 조건식의 임시값은 else로 가기 전에 버려집니다 (2024 에디션)");
    // log.borrow()가 만든 Ref 가드는 임시값입니다. 2024 에디션에서는 패턴이 맞지 않으면
    // else 블록에 들어가기 전에 버려지므로, else 안에서 다시 가변으로 빌릴 수 있습니다.
    // 2021 에디션에서는 가드가 else 끝까지 살아 있어 try_borrow_mut가 실패합니다.
    let log = RefCell::new(Vec::<String>::new());
    if let Some(last) = log.borrow().last() {
        println!("    마지막 기록: {last}");
    } else {
        let can_write = log.try_borrow_mut().is_ok();
        println!("    else 안에서 가변 대여: {can_write}"); // 출력: else 안에서 가변 대여: true
    }

    println!("5. gen은 예약 키워드입니다 (2024 에디션)");
    println!("    r#gen(1) = {}", r#gen(1)); // 출력: r#gen(1) = 1103527590
}
