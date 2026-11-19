// 반환 위치의 `impl Trait`(RPIT)가 2024 에디션에서 인자의 라이프타임을 캡처하는 규칙과,
// `use<..>`로 캡처할 매개변수를 직접 고르는 정밀 캡처를 보여 줍니다.
//
// 실행: cargo run -p p2-ch4-rust-2024-language --example 01_rpit_capture
//
// "캡처한다"는 것은 반환된 값의 타입(불투명 타입)이 그 라이프타임에 묶인다는 뜻입니다.
// 부르는 쪽에서는 "반환된 값이 살아 있는 동안 그 인자를 계속 빌리고 있다"고 취급됩니다.

/// 1. 숨은 타입이 인자를 실제로 빌리는 경우.
///    2024 에디션에서는 인자의 라이프타임을 자동으로 캡처하므로 아무것도 덧붙이지 않아도 됩니다.
///    (2021 에디션에서는 E0700이 나서 `+ '_`나 `+ use<'_>`를 붙여야 했습니다.)
fn evens(numbers: &[i32]) -> impl Iterator<Item = i32> {
    numbers.iter().copied().filter(|n| n % 2 == 0)
}

/// 2. 숨은 타입(`Range<usize>`)은 인자를 빌리지 않지만, 2024 에디션은 그래도 캡처합니다.
///    시그니처만 보고 판단하므로, 부르는 쪽에서는 `numbers`를 빌린 것으로 취급됩니다.
fn positions(numbers: &[i32]) -> impl Iterator<Item = usize> {
    0..numbers.len()
}

/// 3. `use<>`: 아무 라이프타임도 캡처하지 않는다고 직접 적습니다(2021 에디션의 기본 동작과 같음).
fn positions_detached(numbers: &[i32]) -> impl Iterator<Item = usize> + use<> {
    0..numbers.len()
}

/// 4. 라이프타임이 둘일 때 하나만 캡처합니다. 반환값은 `items`만 빌리고 `prefix`는 복사해 둡니다.
fn prefixed<'a>(items: &'a [String], prefix: &str) -> impl Iterator<Item = String> + use<'a> {
    let prefix = prefix.to_owned();
    items.iter().map(move |item| format!("{prefix}{item}"))
}

/// 5. 타입 매개변수는 어느 에디션에서든 언제나 캡처합니다.
///    그래서 `use<..>`를 쓸 때도 타입 매개변수 `T`는 빼놓을 수 없습니다(`use<>`로 쓰면 에러).
fn repeat_first<T: Clone>(items: &[T], times: usize) -> impl Iterator<Item = T> + use<T> {
    std::iter::repeat_n(items[0].clone(), times)
}

fn main() {
    println!("1. 인자를 빌리는 반복자는 그대로 됩니다");
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let even: Vec<i32> = evens(&numbers).collect();
    println!("    evens = {even:?}");
    // 출력:     evens = [2, 4, 6]

    println!("2. 캡처했으므로, 반복자를 다 쓴 뒤에야 원본을 고칠 수 있습니다");
    let mut scores = vec![70, 85];
    let idx: Vec<usize> = positions(&scores).collect(); // 여기서 반복자를 다 씀 → 대여 끝
    scores.push(90);
    println!("    positions = {idx:?}, scores = {scores:?}");
    // 출력:     positions = [0, 1], scores = [70, 85, 90]
    // let it = positions(&scores); scores.push(90); it.count(); 처럼 쓰면 2024에서 E0502입니다
    // (compile_fail/e0502_rpit_captures_borrow.rs).

    println!("3. use<>로 캡처하지 않으면 반복자가 살아 있어도 원본을 고칠 수 있습니다");
    let it = positions_detached(&scores);
    scores.push(95);
    let idx: Vec<usize> = it.collect();
    println!("    positions_detached = {idx:?}, scores = {scores:?}");
    // 출력:     positions_detached = [0, 1, 2], scores = [70, 85, 90, 95]
    // 반복자는 만들 때의 길이(3)를 기억하므로 push한 95의 위치는 들어 있지 않습니다.

    println!("4. use<'a>: 두 인자 중 items만 캡처합니다");
    let items = vec![String::from("사과"), String::from("배")];
    let mut prefix = String::from("- ");
    let lines = prefixed(&items, &prefix);
    prefix.push('>'); // lines가 prefix를 빌리지 않으므로 고칠 수 있습니다.
    for line in lines {
        println!("    {line}");
    }
    println!("    prefix는 이제 {prefix:?}");
    // 출력:     - 사과
    // 출력:     - 배
    // 출력:     prefix는 이제 "- >"

    println!("5. use<T>: 타입 매개변수는 적고, 라이프타임은 뺍니다");
    let mut words = vec!["안녕"];
    let echo = repeat_first(&words, 2);
    words.clear(); // echo가 words를 빌리지 않으므로 비울 수 있습니다.
    println!(
        "    echo = {:?}, words = {words:?}",
        echo.collect::<Vec<_>>()
    );
    // 출력:     echo = ["안녕", "안녕"], words = []
}
