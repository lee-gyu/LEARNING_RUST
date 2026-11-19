// 지침 1: 동작을 받는 매개변수는 `fn` 포인터보다 `Fn*` 트레이트 바운드로 선언합니다.
//
// 받는 쪽이 `fn(i32) -> bool`로 선언하면, 부르는 쪽은 실행 중에 정해진 값을 캡처하는
// 클로저를 넘길 수 없습니다. 미리 써 둔 함수 중 비슷한 것으로 "대충 맞추는" 수밖에 없고,
// 그 결과는 원래 의도와 다를 수 있습니다.
//
// 실행: cargo run -p er-ch2-express-common-behavior --example 01_fn_pointer_vs_fn_bound

/// 비권장: 판단 기준을 `fn` 포인터로 받습니다.
/// `fn` 포인터는 코드의 주소 하나뿐이라, 클로저가 캡처한 값을 함께 담을 자리가 없습니다.
fn first_alert_ptr(readings: &[i32], is_alert: fn(i32) -> bool) -> Option<usize> {
    readings.iter().position(|&t| is_alert(t))
}

/// 권장: `FnMut(i32) -> bool`을 구현한 타입이면 무엇이든 받습니다.
/// 함수, `fn` 포인터, 캡처하지 않는 클로저, 캡처하는 클로저가 모두 들어옵니다.
/// (`Fn`이 아니라 `FnMut`인 이유는 예제 02에서 다룹니다.)
fn first_alert(readings: &[i32], mut is_alert: impl FnMut(i32) -> bool) -> Option<usize> {
    readings.iter().position(|&t| is_alert(t))
}

// 부르는 쪽이 미리 써 둔 판단 함수들입니다.
fn is_warm(t: i32) -> bool {
    t >= 20
}
fn is_heatwave(t: i32) -> bool {
    t >= 33
}
fn is_cold_wave(t: i32) -> bool {
    t <= -12
}

/// `fn` 포인터는 이름을 적을 수 있는 구체 타입입니다. 그래서 `const` 표에 담을 수 있습니다.
/// 클로저의 타입은 이름을 적을 수 없으므로 이런 표를 클로저로는 만들 수 없습니다.
type Rule = fn(i32) -> bool;
const PRESETS: [(&str, Rule); 2] = [("폭염", is_heatwave), ("한파", is_cold_wave)];

fn show(found: Option<usize>, readings: &[i32]) -> String {
    match found {
        Some(i) => format!("{i}번 측정값 {}도", readings[i]),
        None => String::from("경보 없음"),
    }
}

fn main() {
    // 어느 봄날의 시간별 기온입니다. 원하는 기준은 "그날 평균보다 5도 이상 높으면 경보"라서,
    // 기준값은 데이터를 읽어 봐야(실행 중에) 정해집니다.
    let spring = [6, 9, 12, 17, 15, 11];
    let average = spring.iter().sum::<i32>() / spring.len() as i32;
    let limit = average + 5;
    println!("평균 {average}도, 경보 기준 {limit}도 이상");
    // 출력: 평균 11도, 경보 기준 16도 이상

    println!("1. 비권장: fn 포인터로 받는 함수");
    // `|t| t >= limit`는 `limit`을 캡처하므로 넘길 수 없습니다
    // (compile_fail/e0308_capturing_closure_to_fn_pointer.rs).
    // 캡처하지 않는 클로저는 넘길 수 있지만, 그러려면 기준값을 코드에 박아 넣어야 합니다.
    let by_fixed_closure = first_alert_ptr(&spring, |t| t >= 16);
    println!(
        "    값을 박아 넣은 클로저: {}",
        show(by_fixed_closure, &spring)
    );
    // 출력:     값을 박아 넣은 클로저: 3번 측정값 17도
    // 오늘은 맞았지만 내일 평균이 바뀌면 틀립니다. 그래서 미리 써 둔 함수로 대신하면…
    let by_prewritten = first_alert_ptr(&spring, is_warm);
    println!("    미리 써 둔 is_warm: {}", show(by_prewritten, &spring));
    // 출력:     미리 써 둔 is_warm: 경보 없음

    println!("2. 권장: FnMut 바운드로 받는 함수");
    // 기준값을 캡처한 클로저를 그대로 넘깁니다. 데이터가 바뀌어도 기준이 따라 바뀝니다.
    let by_limit = first_alert(&spring, |t| t >= limit);
    println!("    기준을 캡처한 클로저: {}", show(by_limit, &spring));
    // 출력:     기준을 캡처한 클로저: 3번 측정값 17도

    println!("3. 권장 함수는 함수와 fn 포인터도 그대로 받습니다");
    let summer = [27, 31, 34, 36, 30];
    // 함수 이름(함수 아이템)을 바로 넘길 수 있습니다.
    println!(
        "    is_warm: {}",
        show(first_alert(&summer, is_warm), &summer)
    );
    // 출력:     is_warm: 0번 측정값 27도
    // `fn` 포인터 표에서 꺼낸 값도 `FnMut`을 구현하므로 넘길 수 있습니다.
    for (name, rule) in PRESETS {
        println!("    {name}: {}", show(first_alert(&summer, rule), &summer));
    }
    // 출력:     폭염: 2번 측정값 34도
    // 출력:     한파: 경보 없음

    println!("4. fn 포인터가 어울리는 곳: 이름을 적을 수 있는 구체 타입이 필요할 때");
    // `Rule`은 `Copy`이고 크기가 포인터 하나입니다. 캡처한 값이 없으니 언제나 이 크기입니다.
    let copied: Rule = PRESETS[0].1;
    println!(
        "    size_of::<Rule>() = {}, 복사한 규칙으로 36도 판단: {}",
        size_of::<Rule>(),
        copied(36)
    );
    // 출력:     size_of::<Rule>() = 8, 복사한 규칙으로 36도 판단: true
    // (크기 8은 64비트 환경에서의 값입니다.)
}
