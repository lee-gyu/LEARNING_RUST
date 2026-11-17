// Option<T>는 "값이 있을 수도(Some) 없을 수도(None) 있음"을 나타내는 표준 라이브러리의 열거형입니다.
// Rust에는 null이 없고, 값이 없을 수 있는 곳에서는 Option을 씁니다.
// Option<i32>는 i32와 다른 타입이라서, 안의 값을 꺼낸 뒤에야 i32처럼 쓸 수 있다는 것을 보여 줍니다.
//
// 실행: cargo run -p p0-ch6-enums-pattern-matching --example 03_option

// 표준 라이브러리에 정의된 모양은 다음과 같습니다. 직접 정의할 필요는 없습니다.
//
//     enum Option<T> {
//         None,
//         Some(T),
//     }
//
// <T>는 "T 자리에 어떤 타입이든 올 수 있다"는 표시입니다(제네릭, ch10).
// Option, Some, None은 prelude(모든 프로그램에 자동으로 가져와지는 이름들)에 들어 있어서
// Option::Some이라고 쓰지 않고 Some, None으로 바로 씁니다.

// 슬라이스에서 첫 번째 짝수를 찾습니다. 짝수가 없을 수도 있으므로 i32가 아니라 Option<i32>를 돌려줍니다.
// 반환 타입만 봐도 "없을 수 있다"는 사실이 드러나므로, 호출하는 쪽은 그 경우를 처리하지 않을 수 없습니다.
fn first_even(numbers: &[i32]) -> Option<i32> {
    for n in numbers {
        // n은 슬라이스 원소의 참조(&i32)이므로 *로 값을 꺼내 Some에 담습니다.
        if n % 2 == 0 {
            return Some(*n);
        }
    }
    None
}

// 값이 있으면 1을 더하고, 없으면 None을 그대로 돌려줍니다.
// Clippy는 이 match를 x.map(|i| i + 1)로 줄여 쓰라고 제안합니다(clippy::manual_map).
// map과 클로저는 Part 1 ch2-option-result와 ch13에서 배우므로, 여기서는 match의 동작을 보이려고 그대로 둡니다.
#[expect(clippy::manual_map)]
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        // Some(i) 패턴이 안에 든 i32를 i에 묶습니다. 이제 i는 평범한 i32입니다.
        Some(i) => Some(i + 1),
    }
}

fn main() {
    println!("1. Some(값)으로 값을 담고, None으로 값이 없음을 나타냅니다");
    let some_number = Some(5); // Option<i32>로 추론됩니다.
    let some_char = Some('가'); // Option<char>로 추론됩니다.
    // None만 보고는 무엇의 "없음"인지 알 수 없으므로 타입을 적어 줘야 합니다.
    // 적지 않으면 E0282(type annotations needed for `Option<_>`)가 납니다.
    let absent_number: Option<i32> = None;
    println!("    {some_number:?}, {some_char:?}, {absent_number:?}");
    // 출력:
    //     Some(5), Some('가'), None

    println!("2. 표준 라이브러리도 결과가 없을 수 있으면 Option을 돌려줍니다");
    println!("    \"hello\".find('l') = {:?}", "hello".find('l')); // 찾은 바이트 위치
    println!("    \"hello\".find('z') = {:?}", "hello".find('z'));
    println!("    10.checked_div(3) = {:?}", 10_i32.checked_div(3)); // 0으로 나누면 None
    println!("    10.checked_div(0) = {:?}", 10_i32.checked_div(0));
    // 출력:
    //     "hello".find('l') = Some(2)
    //     "hello".find('z') = None
    //     10.checked_div(3) = Some(3)
    //     10.checked_div(0) = None

    println!("3. 직접 만든 함수도 Option으로 \"없을 수 있음\"을 알립니다");
    let a = first_even(&[1, 3, 4, 6]);
    let b = first_even(&[1, 3, 5]);
    let c = first_even(&[]);
    println!("    [1, 3, 4, 6] → {a:?}");
    println!("    [1, 3, 5]    → {b:?}");
    println!("    []           → {c:?}");
    // 출력:
    //     [1, 3, 4, 6] → Some(4)
    //     [1, 3, 5]    → None
    //     []           → None

    println!("4. Option<i32>는 i32가 아니므로, 꺼낸 뒤에야 더할 수 있습니다");
    let x: i32 = 5;
    let y: Option<i32> = Some(5);
    // let sum = x + y; // E0277: cannot add `Option<i32>` to `i32` (compile_fail/e0277_add_option_to_int.rs)
    // match로 Some일 때와 None일 때를 모두 정해야 합니다. 어느 쪽이든 결과는 i32입니다.
    let sum = match y {
        Some(v) => x + v,
        None => x,
    };
    println!("    sum = {sum}");
    // 출력:
    //     sum = 10

    println!("5. Option을 받아 Option을 돌려주는 함수");
    println!("    plus_one(Some(5)) = {:?}", plus_one(Some(5)));
    println!("    plus_one(None)    = {:?}", plus_one(None));
    // 출력:
    //     plus_one(Some(5)) = Some(6)
    //     plus_one(None)    = None

    println!("6. 자주 쓰는 메서드: unwrap_or, is_some, is_none");
    // unwrap_or(기본값): Some이면 안의 값을, None이면 기본값을 돌려줍니다. 4번의 match를 줄여 쓴 것과 같습니다.
    let found = first_even(&[7, 8, 9]).unwrap_or(0);
    let not_found = first_even(&[7, 9]).unwrap_or(0);
    println!("    unwrap_or(0): {found}, {not_found}");
    println!(
        "    is_some: {}, is_none: {}",
        some_number.is_some(),
        absent_number.is_none()
    );
    // 출력:
    //     unwrap_or(0): 8, 0
    //     is_some: true, is_none: true
}
