// 예제 05: 출력 매크로 가족 — print!, println!, eprint!, eprintln!, format!
//
// 보여 주는 것: 서식 문법은 모두 같고, 결과를 "어디로 보내는지"만 다른 매크로들.
//               - print! / println!   → 표준 출력(stdout)
//               - eprint! / eprintln! → 표준 에러(stderr)
//               - format!             → 화면에 찍지 않고 문자열 값으로 돌려줌
//
// 실행: cargo run -p p0-ch1-hello-cargo --example 05_print_family
// 표준 출력만 보기:  cargo run -q -p p0-ch1-hello-cargo --example 05_print_family 2>/dev/null
// 표준 에러만 보기:  cargo run -q -p p0-ch1-hello-cargo --example 05_print_family 1>/dev/null

fn main() {
    // `print!`은 줄바꿈을 붙이지 않습니다. 이어서 출력한 내용이 같은 줄에 붙습니다.
    // `println!`의 `ln`은 line의 줄임말로, 출력 끝에 줄바꿈을 하나 붙여 줍니다.
    print!("하나, ");
    print!("둘, ");
    println!("셋"); // 출력: 하나, 둘, 셋

    // `eprintln!`은 표준 에러로 출력합니다. 터미널에서는 표준 출력과 똑같이 보이지만,
    // 리다이렉션(`>`, `2>`)이나 파이프(`|`)로 나누면 서로 다른 곳으로 갑니다.
    // 프로그램의 "결과"는 표준 출력으로, 진행 상황이나 경고·에러 메시지는 표준 에러로
    // 보내면 결과만 파일에 저장하거나 다른 프로그램에 넘길 수 있습니다.
    eprintln!("[알림] 이 줄은 표준 에러(stderr)로 갑니다.");
    // 출력(stderr): [알림] 이 줄은 표준 에러(stderr)로 갑니다.
    println!("이 줄은 표준 출력(stdout)으로 갑니다.");
    // 출력: 이 줄은 표준 출력(stdout)으로 갑니다.

    // `format!`은 아무것도 출력하지 않고, 서식을 적용한 결과를 문자열(`String`)로 돌려줍니다.
    // 만든 문자열은 변수에 담아 두었다가 나중에 쓸 수 있습니다. `String`은 ch8에서 다룹니다.
    let a = 7;
    let b = 6;
    let line = format!("{a} × {b} = {}", a * b);
    println!("만든 문자열: [{line}]"); // 출력: 만든 문자열: [7 × 6 = 42]

    // 만든 문자열을 다시 다른 서식에 넣을 수도 있습니다.
    // "7 × 6 = 42"는 10글자이므로 너비 16에서 남는 6칸을 양쪽에 3칸씩 나눕니다.
    println!("[{line:^16}]"); // 출력: [   7 × 6 = 42   ]
}
