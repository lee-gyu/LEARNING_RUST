// 반박 불가능한 패턴(어떤 값이 와도 맞음)과 반박 가능한 패턴(맞지 않는 값이 있음)을 구분하고,
// 패턴 자리마다 어느 쪽을 요구하는지 보여 줍니다.
//
// 실행: cargo run -p p0-ch18-patterns --example 02_refutability

// 필드가 하나인 튜플 구조체입니다. Meters 타입의 값은 언제나 Meters(어떤 f64)이므로
// 패턴 Meters(m)은 이 타입의 모든 값과 맞습니다. 즉 반박 불가능한 패턴입니다.
struct Meters(f64);

// 매개변수 자리에는 반박 불가능한 패턴만 올 수 있습니다.
// 호출하는 쪽이 어떤 값을 넘기든 m에 넣을 값이 반드시 있어야 하기 때문입니다.
fn to_centimeters(Meters(m): Meters) -> f64 {
    m * 100.0
}

// if let에 반박 불가능한 패턴을 쓰면 "언제나 맞으니 if가 쓸모없다"는 경고가 납니다.
// 여기서는 경고를 직접 확인하려고 일부러 그렇게 썼으므로 #[expect]로 표시합니다.
// 이 속성을 지우고 빌드하면 warning: irrefutable `if let` pattern 이 나옵니다.
fn always_matches(distance: Meters) {
    #[expect(irrefutable_let_patterns)]
    if let Meters(m) = distance {
        println!("    언제나 이 블록이 실행됩니다: {m}m");
    } else {
        println!("    이 줄은 절대 실행되지 않습니다");
    }
}

fn main() {
    println!("1. 반박 불가능한 패턴: 어떤 값이 와도 맞습니다");
    let (w, h) = (3, 4); // 원소가 둘인 튜플은 언제나 (무엇, 무엇) 모양입니다.
    let Meters(m) = Meters(2.5); // Meters는 언제나 Meters(무엇)입니다.
    println!("    w = {w}, h = {h}, m = {m}");
    println!("    2.5m = {}cm", to_centimeters(Meters(2.5)));
    // 출력:
    //     w = 3, h = 4, m = 2.5
    //     2.5m = 250cm

    println!("2. 반박 가능한 패턴: 맞지 않는 값이 있습니다");
    // Ok(n)은 Err 값과 맞지 않습니다. 그래서 "맞지 않을 때 할 일"을 정할 수 있는
    // if let, while let, let...else, match에서만 쓸 수 있습니다.
    for text in ["42", "사십이"] {
        if let Ok(n) = text.parse::<i32>() {
            println!("    {text} → 숫자 {n}");
        } else {
            println!("    {text} → 숫자가 아님");
        }
    }
    // 출력:
    //     42 → 숫자 42
    //     사십이 → 숫자가 아님

    println!("3. 같은 모양이라도 리터럴이 들어가면 반박 가능해집니다");
    // (x, _)는 모든 튜플과 맞지만, (x, 0)은 둘째 값이 0인 튜플과만 맞습니다.
    let pairs = [(1, 0), (2, 5)];
    for pair in pairs {
        let (x, _) = pair; // 반박 불가능: let에 바로 씁니다.
        // 반박 가능: 맞지 않을 수 있으므로 if let이 필요합니다.
        if let (y, 0) = pair {
            println!("    x = {x}, 둘째 값이 0인 쌍 (y = {y})");
        } else {
            println!("    x = {x}, 둘째 값이 0이 아닌 쌍");
        }
    }
    // 출력:
    //     x = 1, 둘째 값이 0인 쌍 (y = 1)
    //     x = 2, 둘째 값이 0이 아닌 쌍

    println!("4. match는 갈래들을 합쳐서 모든 값을 덮어야 합니다");
    // 앞 갈래들은 반박 가능한 패턴이고, 마지막 갈래(other)는 반박 불가능한 패턴으로 나머지를 모두 받습니다.
    for code in [200, 404, 500] {
        let text = match code {
            200 => String::from("성공"),
            404 => String::from("찾을 수 없음"),
            other => format!("기타 상태 {other}"),
        };
        println!("    {code}: {text}");
    }
    // 출력:
    //     200: 성공
    //     404: 찾을 수 없음
    //     500: 기타 상태 500

    println!("5. 반박 불가능한 패턴을 if let에 쓰면 else는 절대 실행되지 않습니다");
    always_matches(Meters(7.0));
    // 출력:
    //     언제나 이 블록이 실행됩니다: 7m
}
