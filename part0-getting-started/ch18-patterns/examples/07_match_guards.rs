// 매치 가드: 갈래의 패턴 뒤에 붙이는 if 조건입니다. 패턴이 맞고 조건도 참일 때만 그 갈래로 갑니다.
// 패턴만으로는 표현할 수 없는 조건(짝수인가, 바깥 변수와 같은가)을 적을 때 씁니다.
//
// 실행: cargo run -p p0-ch18-patterns --example 07_match_guards

// 가드는 완전성 검사에 들어가지 않습니다. 컴파일러는 가드 조건이 참일지 거짓일지 따지지 않고,
// "가드가 있는 갈래는 맞지 않을 수도 있다"고 봅니다. 그래서 마지막에 가드 없는 갈래가 필요합니다.
// 마지막 갈래를 `x if x > 0 => "양수"`로 바꾸면 E0004가 납니다(compile_fail/e0004_guard_not_exhaustive.rs).
fn sign(n: i32) -> &'static str {
    match n {
        x if x < 0 => "음수",
        0 => "0",
        _ => "양수",
    }
}

fn main() {
    println!("1. 가드: 패턴이 맞아도 조건이 거짓이면 다음 갈래로 넘어갑니다");
    for num in [Some(4), Some(5), None] {
        match num {
            Some(x) if x % 2 == 0 => println!("    {x}: 짝수"),
            // Some(5)는 첫 갈래의 패턴과 맞지만 가드가 거짓이므로 여기로 옵니다.
            Some(x) => println!("    {x}: 홀수"),
            None => println!("    값 없음"),
        }
    }
    // 출력:
    //     4: 짝수
    //     5: 홀수
    //     값 없음

    println!("2. 가드로 바깥 변수와 비교하기 (03의 섀도잉 함정 해결)");
    let target = 10;
    for x in [Some(5), Some(10)] {
        match x {
            // 패턴에는 새 이름 n을 쓰고, 가드에서 바깥의 target과 비교합니다.
            // 가드는 패턴이 아니므로 target이라는 새 변수를 만들지 않습니다.
            Some(n) if n == target => println!("    {n}: 목표값과 같습니다"),
            Some(n) => println!("    {n}: 목표값이 아닙니다"),
            None => println!("    값 없음"),
        }
    }
    // 출력:
    //     5: 목표값이 아닙니다
    //     10: 목표값과 같습니다

    println!("3. |와 가드: 가드는 | 로 묶인 패턴 전체에 걸립니다");
    let x = 4;
    let y = false;
    match x {
        // (4 | 6 | 8) if y 로 읽힙니다. 4 | 6 | (8 if y)가 아닙니다.
        // x는 4라서 패턴은 맞지만 y가 false이므로 이 갈래는 선택되지 않습니다.
        4 | 6 | 8 if y => println!("    yes"),
        _ => println!("    no"),
    }
    // 출력:
    //     no

    println!("4. 가드가 있으면 마지막에 가드 없는 갈래가 필요합니다");
    for n in [-3, 0, 8] {
        println!("    {n}: {}", sign(n));
    }
    // 출력:
    //     -3: 음수
    //     0: 0
    //     8: 양수

    println!("5. if let에는 가드가 없지만, let 체인으로 같은 일을 합니다 (Edition 2024)");
    for num in [Some(4), Some(5)] {
        if let Some(x) = num
            && x % 2 == 0
        {
            println!("    {x}: 짝수만 처리합니다");
        } else {
            println!("    {num:?}: 건너뜁니다");
        }
    }
    // 출력:
    //     4: 짝수만 처리합니다
    //     Some(5): 건너뜁니다

    println!("6. 가드 자리에 if let 쓰기");
    for input in [Some("42"), Some("사십이"), None] {
        match input {
            // 가드 안의 let 패턴이 맞을 때만 이 갈래로 오고, 묶인 n을 갈래 안에서 씁니다.
            Some(text) if let Ok(n) = text.parse::<i32>() => println!("    {text} → {n}"),
            Some(text) => println!("    {text} → 숫자가 아닙니다"),
            None => println!("    입력 없음"),
        }
    }
    // 출력:
    //     42 → 42
    //     사십이 → 숫자가 아닙니다
    //     입력 없음
}
