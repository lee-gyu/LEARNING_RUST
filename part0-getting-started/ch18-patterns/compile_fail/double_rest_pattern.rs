// 기대하는 에러: (에러 코드 없음) `..` can only be used once per tuple pattern
// 원인: ..는 "몇 개인지 모르는 나머지"를 건너뜁니다. 한 패턴에 ..가 두 번 있으면
//       앞의 ..가 몇 개를 건너뛰고 second가 몇 번째 원소인지 정할 수 없습니다.
// 해결: ..는 한 번만 쓰고, 나머지 위치는 _로 하나씩 적습니다.
//       let (_, second, ..) = numbers;
//
// 확인: rustc --edition 2024 compile_fail/double_rest_pattern.rs

fn main() {
    let numbers = (2, 4, 8, 16, 32);

    let (.., second, ..) = numbers;
    println!("두 번째 값: {second}");
}
