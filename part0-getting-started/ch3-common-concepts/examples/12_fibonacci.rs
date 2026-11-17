// 이 예제가 보여 주는 것: n번째 피보나치 수 구하기 (The Book 3장 끝 연습 과제의 풀이)
//   이 챕터에서 배운 함수, `for`와 범위, 튜플, `loop`와 `break 값`, 오버플로 검사를 함께 씁니다.
// 실행: cargo run -p p0-ch3-common-concepts --example 12_fibonacci

// 피보나치 수열: F(0) = 0, F(1) = 1, F(n) = F(n-1) + F(n-2)
// 값이 빠르게 커지므로 부호 없는 64비트 정수 `u64`를 씁니다.
fn fibonacci(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut prev: u64 = 0; // F(0)
    let mut curr: u64 = 1; // F(1)
    // 반복할 때마다 (prev, curr)가 한 칸씩 앞으로 이동합니다. n-1번 반복하면 curr = F(n)입니다.
    // 반복 변수를 쓰지 않을 때는 이름 대신 `_`를 씁니다.
    for _ in 1..n {
        // 튜플 구조 분해 대입: 오른쪽 튜플 `(curr, prev + curr)`를 먼저 다 계산한 뒤 한꺼번에 대입하므로
        // 값을 잠시 보관할 임시 변수가 필요 없습니다.
        (prev, curr) = (curr, prev + curr);
    }
    curr
}

fn main() {
    // 처음 15개의 피보나치 수를 쉼표로 구분해 출력합니다.
    for n in 0..15 {
        if n > 0 {
            print!(", ");
        }
        print!("{}", fibonacci(n));
    }
    println!();
    // 출력: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377

    println!("F(50) = {}", fibonacci(50)); // 출력: F(50) = 12586269025
    println!("F(93) = {}", fibonacci(93)); // 출력: F(93) = 12200160415121876738

    // F(94)는 u64의 최댓값(18446744073709551615)을 넘습니다. `fibonacci(94)`를 호출하면
    // 디버그 빌드에서는 `prev + curr`에서 오버플로 패닉이 납니다(04_integer_overflow 참고).
    // 그 경계를 패닉 없이 찾아봅니다. `overflowing_add`는 (결과, 오버플로 여부) 튜플을 돌려주고,
    // 오버플로가 나는 순간 `break 값`으로 루프를 끝내며 그때의 번호를 꺼냅니다.
    let mut n = 1;
    let mut prev: u64 = 0;
    let mut curr: u64 = 1;
    let last_n = loop {
        let (next, overflowed) = prev.overflowing_add(curr);
        if overflowed {
            break n; // 다음 수 F(n+1)이 u64에 들어가지 않으므로 F(n)이 마지막입니다.
        }
        (prev, curr) = (curr, next);
        n += 1;
    };
    println!("u64에 담을 수 있는 마지막 피보나치 수: F({last_n}) = {curr}");
    // 출력: u64에 담을 수 있는 마지막 피보나치 수: F(93) = 12200160415121876738
}
