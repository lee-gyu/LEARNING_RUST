// 이 예제가 보여 주는 것: 구문(statement)과 표현식(expression)의 차이, 블록이 값을 내는 성질, 세미콜론의 의미
// 실행: cargo run -p p0-ch3-common-concepts --example 08_expressions

fn main() {
    // ── 1. 구문은 동작만 하고 값을 내지 않습니다 ──────────────────────────
    // `let a = 6;` 한 줄 전체가 구문입니다. 오른쪽의 `6`은 값을 내는 표현식입니다.
    let a = 6;
    // `let` 구문은 값이 없으므로 다른 변수에 담을 수 없습니다.
    // `let b = (let c = 6);`는 "expected expression, found `let` statement" 에러가 납니다.
    // C처럼 `x = y = 6`으로 여러 변수에 한꺼번에 대입하는 문법도 없습니다.

    // ── 2. 표현식은 값을 냅니다 ──────────────────────────────────────────
    // 리터럴, 연산, 함수 호출, 매크로 호출, 중괄호 블록이 모두 표현식입니다.
    let b = a * 2 + 1;
    println!("b = {b}"); // 출력: b = 13

    // ── 3. 블록 표현식: 마지막 표현식이 블록 전체의 값이 됩니다 ─────────────
    let y = {
        let x = 3;
        x + 1 // 세미콜론이 없으므로 이 값(4)이 블록의 값입니다.
    };
    println!("y = {y}"); // 출력: y = 4

    // 블록 안에서 만든 임시 변수(`width`, `height`)는 블록 밖으로 새어 나가지 않습니다.
    // 계산에만 필요한 이름을 좁은 스코프에 가두고 결과만 꺼낼 때 유용합니다.
    let area = {
        let width = 3;
        let height = 4;
        width * height
    };
    println!("area = {area}"); // 출력: area = 12

    // ── 4. 세미콜론은 표현식을 구문으로 바꿉니다 ───────────────────────────
    // 마지막 줄에 세미콜론을 붙이면 그 값은 계산된 뒤 버려지고,
    // 블록에는 마지막 표현식이 없으므로 블록의 값은 유닛 `()`가 됩니다.
    #[expect(
        clippy::let_unit_value,
        reason = "세미콜론 때문에 블록의 값이 ()가 되는 것을 보여 주려고 일부러 ()를 변수에 담습니다"
    )]
    let nothing = {
        let x = 3;
        plus_one(x); // 세미콜론: 결과(4)를 버립니다.
    };
    println!("nothing = {nothing:?}"); // 출력: nothing = ()

    // ── 5. 함수 본문도 블록입니다 ─────────────────────────────────────────
    // 그래서 함수의 반환값도 같은 규칙을 따릅니다. 마지막 표현식이 반환값입니다.
    println!("plus_one(4) = {}", plus_one(4)); // 출력: plus_one(4) = 5
    println!("square_of_sum(1, 2) = {}", square_of_sum(1, 2)); // 출력: square_of_sum(1, 2) = 9

    // `if`와 `loop`도 값을 내는 표현식입니다. 09_if, 10_loop 예제에서 이어서 봅니다.
}

fn plus_one(x: i32) -> i32 {
    x + 1
}

// 본문 안에 구문(`let`)이 몇 줄 있든, 마지막 표현식 `sum * sum`이 반환값입니다.
fn square_of_sum(a: i32, b: i32) -> i32 {
    let sum = a + b;
    sum * sum
}
