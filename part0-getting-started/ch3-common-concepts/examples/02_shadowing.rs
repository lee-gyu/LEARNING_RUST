// 이 예제가 보여 주는 것: 섀도잉(shadowing)으로 같은 이름의 새 변수를 만드는 방법과 `mut`와의 차이
// 실행: cargo run -p p0-ch3-common-concepts --example 02_shadowing

fn main() {
    // 1. 섀도잉: `let`을 다시 쓰면 같은 이름의 "새 변수"가 생기고, 이전 변수는 가려집니다.
    //    값을 바꾸는 것이 아니라 새로 만드는 것이므로 `x`는 계속 불변입니다.
    let x = 5;
    let x = x + 1; // 오른쪽의 `x`는 이전 변수(5), 왼쪽의 `x`는 새 변수(6)입니다.

    {
        // 안쪽 스코프(scope)에서 만든 `x`는 이 중괄호 블록 안에서만 유효합니다.
        let x = x * 2;
        println!("안쪽 스코프의 x = {x}"); // 출력: 안쪽 스코프의 x = 12
    }
    // 블록이 끝나면 안쪽 `x`가 사라지고, 가려져 있던 바깥 `x`가 다시 보입니다.
    println!("바깥 스코프의 x = {x}"); // 출력: 바깥 스코프의 x = 6

    // 2. 같은 상황을 `mut`로 쓰면 결과가 다릅니다.
    //    안쪽 블록의 대입은 새 변수를 만들지 않고 바깥 변수 자체를 바꾸기 때문입니다.
    let mut y = 6;
    {
        y *= 2;
        println!("안쪽 스코프의 y = {y}"); // 출력: 안쪽 스코프의 y = 12
    }
    println!("바깥 스코프의 y = {y}"); // 출력: 바깥 스코프의 y = 12

    // 3. 섀도잉은 타입을 바꿀 수 있습니다. 새 변수이므로 타입도 새로 정해집니다.
    //    `spaces_str`, `spaces_num`처럼 이름을 따로 지어낼 필요가 없습니다.
    let spaces = "   "; // 문자열
    let spaces = spaces.len(); // 숫자(usize)
    println!("공백 개수 = {spaces}"); // 출력: 공백 개수 = 3
    // `mut` 변수에는 다른 타입의 값을 넣을 수 없습니다.
    // (compile_fail/e0308_mut_cannot_change_type.rs)

    // 4. ch2 숫자 맞히기 게임에서 본 패턴도 섀도잉입니다.
    //    입력받은 문자열을 숫자로 바꾼 뒤, 같은 이름으로 계속 씁니다.
    //    `parse`는 여러 숫자 타입으로 바꿀 수 있으므로 `: u32`로 원하는 타입을 알려 줘야 합니다.
    let guess = " 42 ";
    let guess: u32 = guess.trim().parse().expect("숫자가 아닙니다");
    println!("guess + 1 = {}", guess + 1); // 출력: guess + 1 = 43

    // 5. 값을 다 만든 뒤 불변으로 "고정"할 때도 섀도잉을 씁니다.
    //    아래 줄 이후로 `total`을 실수로 바꾸려 하면 컴파일 에러가 납니다.
    let mut total = 0;
    total += 10;
    total += 20;
    let total = total;
    println!("total = {total}"); // 출력: total = 30
}
