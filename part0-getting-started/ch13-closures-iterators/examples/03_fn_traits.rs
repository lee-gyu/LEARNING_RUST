// 이 예제가 보여 주는 것: FnOnce / FnMut / Fn 트레이트, 클로저를 매개변수로 받는 함수(impl Fn과 제네릭 바운드),
//                        클로저 자리에 함수 이름 넘기기, unwrap_or_else(FnOnce)와 sort_by_key·sort_by(FnMut)
// 실행: cargo run -p p0-ch13-closures-iterators --example 03_fn_traits
//
// 어떤 Fn 트레이트를 구현하는지는 "본문이 캡처한 값을 어떻게 다루는가"로 정해집니다.
//   - 캡처한 값을 밖으로 내보냄(이동)      → FnOnce만      (한 번만 부를 수 있음)
//   - 캡처한 값을 바꿈                    → FnMut, FnOnce  (여러 번, 단 바꿀 수 있게 빌려야 함)
//   - 읽기만 하거나 아무것도 캡처하지 않음 → Fn, FnMut, FnOnce (여러 번, 아무 제약 없이)

// 클로저를 한 번만 부르는 함수입니다. 한 번만 부르므로 가장 느슨한 FnOnce를 요구합니다.
// impl Trait 문법(ch10)으로 "FnOnce()를 구현한 어떤 타입이든"이라고 적었습니다.
fn call_once(f: impl FnOnce()) {
    f();
}

// 클로저를 두 번 부르고, 클로저가 캡처한 값을 바꿔도 되게 하려면 FnMut을 요구합니다.
// 제네릭 바운드 문법으로도 같은 뜻을 적을 수 있습니다. FnMut 클로저를 부르면 클로저의 상태가
// 바뀌므로 매개변수 f에도 mut이 필요합니다.
fn call_twice_mut<F: FnMut()>(mut f: F) {
    f();
    f();
}

// 두 번 부르되 클로저가 아무것도 바꾸지 못하게 하려면 Fn을 요구합니다(where 절로 적은 바운드).
fn call_twice<F>(f: F)
where
    F: Fn(),
{
    f();
    f();
}

// 클로저 대신 넘길 보통 함수입니다. 함수는 환경을 캡처하지 않으므로 세 트레이트를 모두 구현합니다.
fn say_hello() {
    println!("  함수 say_hello 실행");
}

// 회원 번호로 별명을 찾는 흉내를 냅니다. 1번 회원만 별명이 있습니다.
fn find_nickname(user_id: u32) -> Option<String> {
    if user_id == 1 {
        Some(String::from("러스타시안"))
    } else {
        None
    }
}

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

fn main() {
    // 1. 읽기만 하는 클로저: Fn
    println!("== 1. 읽기만 하는 클로저 (Fn) ==");
    let greeting = String::from("안녕하세요");
    let greet = || println!("  {greeting}");
    // greet는 &greeting만 들고 있습니다. 참조는 Copy이므로 이 클로저도 통째로 복사되어
    // 세 함수에 차례로 넘길 수 있습니다. Fn을 구현하면 FnMut과 FnOnce도 자동으로 구현합니다.
    call_twice(greet);
    call_twice_mut(greet);
    call_once(greet);
    // 출력:   안녕하세요   (다섯 번 출력됩니다)

    // 2. 캡처한 값을 바꾸는 클로저: FnMut
    println!("\n== 2. 캡처한 값을 바꾸는 클로저 (FnMut) ==");
    let mut count = 0;
    call_twice_mut(|| count += 1); // FnMut을 요구하는 자리: 됩니다.
    call_once(|| count += 1); // FnOnce를 요구하는 자리: FnMut은 FnOnce도 구현하므로 됩니다.
    // call_twice(|| count += 1); // Fn을 요구하는 자리: error[E0594]: cannot assign to `count`,
    //                            // as it is a captured variable in a `Fn` closure
    println!("count = {count}"); // 출력: count = 3

    // 3. 캡처한 값을 밖으로 내보내는 클로저: FnOnce
    println!("\n== 3. 캡처한 값을 내보내는 클로저 (FnOnce) ==");
    let name = String::from("페리스");
    call_once(|| {
        let taken = name; // 캡처한 name을 클로저 밖의 변수로 옮깁니다. 두 번째 호출 때는 옮길 name이 없습니다.
        println!("  가져간 값: {taken}");
    });
    // 출력:   가져간 값: 페리스
    // 같은 클로저를 call_twice_mut에 넘기면 error[E0507]: cannot move out of `name`,
    // a captured variable in an `FnMut` closure 입니다.

    // 4. 클로저 자리에 함수 이름을 넘길 수 있습니다
    println!("\n== 4. 함수 이름 넘기기 ==");
    call_twice(say_hello);
    // 출력:   함수 say_hello 실행   (두 번)

    // 5. Option::unwrap_or_else는 FnOnce() -> T를 요구합니다
    println!("\n== 5. unwrap_or_else ==");
    // Some이면 안의 값을 꺼내고 클로저는 부르지 않습니다. None일 때만 클로저를 "한 번" 부릅니다.
    let shown = find_nickname(1).unwrap_or_else(|| {
        println!("  (기본값 클로저 실행)");
        String::from("손님")
    });
    println!("Some일 때: {shown}"); // 출력: Some일 때: 러스타시안   (클로저는 실행되지 않음)

    let default_name = String::from("손님");
    let shown = find_nickname(2).unwrap_or_else(|| {
        println!("  (기본값 클로저 실행)");
        default_name // 캡처한 String을 내보냅니다. FnOnce만 구현하지만 unwrap_or_else는 받아 줍니다.
    });
    println!("None일 때: {shown}");
    // 출력:   (기본값 클로저 실행)
    // 출력: None일 때: 손님

    // Result::unwrap_or_else는 FnOnce(E) -> T를 요구합니다. Err일 때 에러 값을 클로저 인자로 넘깁니다(ch9).
    let number: i32 = "42a".parse().unwrap_or_else(|err| {
        println!("  파싱 실패: {err}");
        0
    });
    println!("number = {number}");
    // 출력:   파싱 실패: invalid digit found in string
    // 출력: number = 0

    // 6. sort_by_key와 sort_by는 FnMut을 요구합니다
    println!("\n== 6. sort_by_key, sort_by ==");
    let mut list = [
        Rectangle {
            width: 10,
            height: 1,
        },
        Rectangle {
            width: 3,
            height: 5,
        },
        Rectangle {
            width: 7,
            height: 12,
        },
        Rectangle {
            width: 2,
            height: 5,
        },
    ];

    // 정렬하는 동안 원소마다 클로저를 부르므로(여러 번) FnOnce로는 안 되고 FnMut이어야 합니다.
    // 이 클로저는 바깥의 num_sort_operations를 바꾸므로 FnMut입니다. 그래도 sort_by_key는 받아 줍니다.
    let mut num_sort_operations = 0;
    list.sort_by_key(|r| {
        num_sort_operations += 1;
        r.width // 이 값(키)을 기준으로 오름차순 정렬합니다.
    });
    println!("너비 오름차순: {list:?}");
    println!("키 클로저 호출 횟수: {num_sort_operations}");
    // 출력: 너비 오름차순: [Rectangle { width: 2, height: 5 }, Rectangle { width: 3, height: 5 }, Rectangle { width: 7, height: 12 }, Rectangle { width: 10, height: 1 }]
    // 출력: 키 클로저 호출 횟수: 12
    // (호출 횟수는 표준 라이브러리의 정렬 구현에 달린 값이라 Rust 버전에 따라 달라질 수 있습니다.)

    // sort_by는 두 원소를 받아 순서(Ordering)를 돌려주는 클로저를 받습니다. ch2에서 본 cmp가
    // a와 b의 Ordering을 돌려주고, b와 a의 자리를 바꿔 비교하면 내림차순이 됩니다.
    // then은 앞의 비교 결과가 Equal(같음)일 때만 뒤의 비교 결과를 씁니다. 그래서 "높이 오름차순,
    // 높이가 같으면 너비 내림차순"처럼 방향이 섞인 기준도 표현할 수 있습니다.
    list.sort_by(|a, b| a.height.cmp(&b.height).then(b.width.cmp(&a.width)));
    println!("높이 오름차순, 같으면 너비 내림차순: {list:?}");
    // 출력: 높이 오름차순, 같으면 너비 내림차순: [Rectangle { width: 10, height: 1 }, Rectangle { width: 3, height: 5 }, Rectangle { width: 2, height: 5 }, Rectangle { width: 7, height: 12 }]
    // 높이가 5로 같은 두 사각형은 너비 3이 2보다 앞에 옵니다.
}
