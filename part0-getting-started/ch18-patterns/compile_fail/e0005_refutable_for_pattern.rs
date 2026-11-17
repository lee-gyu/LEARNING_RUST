// 기대하는 에러: E0005 refutable pattern in `for` loop binding
// 원인: for 바로 뒤의 패턴은 반박 불가능해야 합니다. 원소가 None이면 Some(r)과 맞지 않는데,
//       for에는 "맞지 않을 때 할 일"을 적을 자리가 없습니다.
//       함수·클로저 매개변수와 일반 let에도 같은 규칙이 적용됩니다.
// 해결: 원소를 변수 하나로 받고, 반복문 안에서 if let이나 match로 나눕니다.
//       for reading in readings { if let Some(r) = reading { ... } }
//
// 확인: rustc --edition 2024 compile_fail/e0005_refutable_for_pattern.rs

fn main() {
    // 센서 측정값입니다. 측정에 실패한 칸은 None입니다.
    let readings = vec![Some(21), None, Some(23)];

    for Some(r) in readings {
        println!("측정값: {r}");
    }
}
