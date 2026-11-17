// 이 예제가 보여 주는 것: 정수 오버플로(overflow)가 일어날 때의 동작
//   - 오버플로를 명시적으로 처리하는 메서드 4종(`wrapping_*`, `checked_*`, `overflowing_*`, `saturating_*`)
//   - `+` 연산자: 디버그 빌드에서는 패닉(panic), 릴리스 빌드에서는 값이 한 바퀴 돌아감(래핑)
//
// 주의: 디버그 빌드에서 246 이상을 입력하면 마지막 단계에서 일부러 패닉하며 끝납니다(종료 코드 101).
//
// 실행(오버플로 없음): echo 100 | cargo run -p p0-ch3-common-concepts --example 04_integer_overflow
// 실행(디버그, 패닉):  echo 250 | cargo run -p p0-ch3-common-concepts --example 04_integer_overflow
// 실행(릴리스, 래핑):  echo 250 | cargo run --release -p p0-ch3-common-concepts --example 04_integer_overflow

use std::io;

fn main() {
    println!("0~255 사이의 수를 입력하세요. 여기에 10을 더합니다.");

    // 값을 표준 입력에서 받는 이유: `let x: u8 = 250; x + 10`처럼 컴파일러가 값을 미리 알면
    // 오버플로를 컴파일 에러(this arithmetic operation will overflow)로 막아 버립니다.
    // 실행해 봐야 알 수 있는 값이어야 런타임 동작을 관찰할 수 있습니다.
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("입력을 읽지 못했습니다");
    let stock: u8 = input
        .trim()
        .parse()
        .expect("0~255 사이의 정수를 입력하세요");

    // 1. wrapping_add: 범위를 넘으면 0부터 다시 셉니다. 빌드 모드와 상관없이 항상 같습니다.
    //    250 + 10 = 260 → 260 - 256 = 4
    println!("wrapping_add    : {}", stock.wrapping_add(10));
    // 출력(입력 100): wrapping_add    : 110
    // 출력(입력 250): wrapping_add    : 4

    // 2. checked_add: 성공하면 `Some(값)`, 오버플로가 나면 `None`을 돌려줍니다.
    //    이런 "값이 있을 수도 없을 수도 있는" 타입을 `Option`이라고 하며 ch6에서 자세히 다룹니다.
    //    `{:?}`는 디버그 형식 출력으로, `Some(110)`이나 `None` 같은 모양을 그대로 보여 줍니다.
    println!("checked_add     : {:?}", stock.checked_add(10));
    // 출력(입력 100): checked_add     : Some(110)
    // 출력(입력 250): checked_add     : None

    // 3. overflowing_add: (래핑된 값, 오버플로 여부) 튜플을 돌려줍니다.
    println!("overflowing_add : {:?}", stock.overflowing_add(10));
    // 출력(입력 100): overflowing_add : (110, false)
    // 출력(입력 250): overflowing_add : (4, true)

    // 4. saturating_add: 범위를 넘으면 그 타입의 최댓값(또는 최솟값)에서 멈춥니다.
    println!("saturating_add  : {}", stock.saturating_add(10));
    // 출력(입력 100): saturating_add  : 110
    // 출력(입력 250): saturating_add  : 255

    // 5. `+` 연산자: 오버플로를 어떻게 처리할지 코드에 적지 않은 경우입니다.
    //    디버그 빌드(`cargo run`)는 오버플로를 검사해서 패닉으로 프로그램을 멈춥니다.
    //    릴리스 빌드(`cargo run --release`)는 속도를 위해 검사를 생략하고 wrapping_add처럼 래핑합니다.
    //    어느 쪽이든 의도하지 않은 오버플로는 버그이므로, 일어날 수 있다면 위의 메서드로 의도를 밝힙니다.
    println!("+ 연산자로 더해 봅니다...");
    let sum = stock + 10;
    println!("+ 연산자        : {sum}");
    // 출력(입력 100): + 연산자        : 110
    // 출력(입력 250, 디버그): 패닉 — thread 'main' (스레드 ID는 실행마다 다름) panicked at ...:
    //                         attempt to add with overflow
    // 출력(입력 250, 릴리스): + 연산자        : 4
}
