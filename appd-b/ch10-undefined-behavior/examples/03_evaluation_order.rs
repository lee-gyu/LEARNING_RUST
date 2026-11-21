// Q10.6 — 평가 순서와 전역 초기화 순서
//
// C에서 i = i++ + 1처럼 순서가 정해지지 않은 두 수정이 한 식에 있으면 미정의 동작이고(compare/06_unsequenced.c),
// 함수 인자의 평가 순서도 정해져 있지 않습니다. Rust Reference는 피연산자를 왼쪽에서 오른쪽으로 평가한다고
// 정하고, 대입식(과 원시 타입끼리의 복합 대입)은 대입할 값을 먼저, 대입받을 자리를 나중에 평가한다고 따로 정합니다.
//
//   1. 함수 인자와 이항 연산: 왼쪽에서 오른쪽
//   2. 대입: 값을 먼저, 자리(인덱스)를 나중에
//   3. static의 초기값은 컴파일 시간에 계산되므로, 실행 중 "어느 전역이 먼저 초기화되나"를 따질 일이 없음
//
// 실행: cargo run -p faq-ch10-undefined-behavior --example 03_evaluation_order

/// 평가되는 순간 이름을 출력하고 값을 그대로 돌려줍니다.
fn log(tag: &str, value: usize) -> usize {
    println!("    {tag}");
    value
}

fn pair(a: usize, b: usize) -> (usize, usize) {
    (a, b)
}

/// const fn이라 static의 초기값에서 부를 수 있고, 그 계산은 컴파일할 때 끝납니다.
const fn squares() -> [u32; 4] {
    let mut table = [0; 4];
    let mut i = 0;
    while i < 4 {
        table[i] = (i * i) as u32;
        i += 1;
    }
    table
}

static SQUARES: [u32; 4] = squares();

fn main() {
    println!("1. 함수 인자와 이항 연산");
    let p = pair(log("첫째 인자", 1), log("둘째 인자", 2));
    let s = log("왼쪽 피연산자", 10) + log("오른쪽 피연산자", 20);
    println!("  {p:?}, {s}");
    // 출력:     첫째 인자
    // 출력:     둘째 인자
    // 출력:     왼쪽 피연산자
    // 출력:     오른쪽 피연산자
    // 출력:   (1, 2), 30

    println!("2. 대입");
    let mut v = vec![0; 3];
    v[log("자리(인덱스)", 1)] = log("값", 7);
    println!("  {v:?}");
    // 출력:     값
    // 출력:     자리(인덱스)
    // 출력:   [0, 7, 0]

    println!("3. 컴파일 시간에 정해진 static");
    println!("  SQUARES = {SQUARES:?}");
    // 출력:   SQUARES = [0, 1, 4, 9]
}
