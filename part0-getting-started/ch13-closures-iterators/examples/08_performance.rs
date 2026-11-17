// 이 예제가 보여 주는 것: 같은 일을 하는 for 반복문과 반복자 체인의 실행 시간 비교(비용 없는 추상화),
//                        디버그 빌드와 릴리스 빌드에서 결과가 어떻게 달라지는지
// 실행: cargo run -p p0-ch13-closures-iterators --example 08_performance             (디버그 빌드)
//       cargo run -p p0-ch13-closures-iterators --example 08_performance --release   (릴리스 빌드)
//
// 측정한 시간은 컴퓨터, 실행할 때의 부하, 빌드 방식에 따라 매번 달라집니다. 그래서 아래 출력 주석에는
// 시간 값을 적지 않았습니다. 두 빌드로 직접 실행해 for 반복문과 반복자의 시간을 비교해 보세요.

use std::hint::black_box;
use std::time::{Duration, Instant};

// ch12 minigrep의 search를 for 반복문으로 쓴 판입니다.
fn search_loop<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}

// 같은 일을 반복자 어댑터로 쓴 판입니다(이 챕터에서 개선한 minigrep의 search).
fn search_iter<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

// 짝수만 골라 제곱해 더하기: for 반복문 판
fn sum_even_squares_loop(numbers: &[u64]) -> u64 {
    let mut total = 0;
    for n in numbers {
        if n % 2 == 0 {
            total += n * n;
        }
    }
    total
}

// 같은 일의 반복자 판
fn sum_even_squares_iter(numbers: &[u64]) -> u64 {
    numbers.iter().filter(|n| *n % 2 == 0).map(|n| n * n).sum()
}

// 클로저 f를 여러 번 실행해 가장 빠른 시간을 출력하고, 마지막 결과를 돌려줍니다.
// 여러 번 부르고, 부를 때마다 클로저가 하는 일이 달라도 되므로 FnMut을 요구합니다.
// 한 번만 재면 다른 프로그램의 영향 등으로 값이 크게 튈 수 있어 가장 빠른 값을 씁니다.
fn measure<T>(label: &str, mut f: impl FnMut() -> T) -> T {
    let mut best = Duration::MAX;
    let mut result = f(); // 첫 실행은 준비 운동으로 보고 시간에 넣지 않습니다.
    for _ in 0..5 {
        let start = Instant::now();
        // 결과를 black_box에 통과시켜 "이 결과는 쓰인다"고 알립니다. 그러지 않으면 바로 다음 반복에서
        // 덮어써질 결과라서 최적화가 계산 자체를 지워 버릴 수 있습니다. 이 예제를 처음 쓸 때 이 black_box가
        // 없었더니, 릴리스 빌드에서 숫자 합계 시간이 0ns로 나왔습니다.
        result = black_box(f());
        let elapsed = start.elapsed();
        if elapsed < best {
            best = elapsed;
        }
    }
    println!("  {label}: {best:?}");
    result
}

fn main() {
    // 검색 대상: ch12의 poem.txt 내용을 여러 번 이어 붙여 큰 문자열을 만듭니다.
    let poem = "\
I'm nobody! Who are you?
Are you nobody, too?
Then there's a pair of us - don't tell!
They'd banish us, you know.

How dreary to be somebody!
How public, like a frog
To tell your name the livelong day
To an admiring bog!
";
    let contents = poem.repeat(20_000);
    let numbers: Vec<u64> = (0..2_000_000).collect();

    if cfg!(debug_assertions) {
        println!("디버그 빌드로 실행했습니다(최적화 꺼짐).");
    } else {
        println!("릴리스 빌드로 실행했습니다(최적화 켜짐).");
    }
    // 출력: 디버그 빌드로 실행했습니다(최적화 꺼짐).   (--release로 실행하면 "릴리스 빌드로 ...")

    // black_box는 "이 값이 무엇인지 컴파일러가 미리 안다고 가정하지 말라"는 표시입니다.
    // 최적화가 켜진 빌드에서 컴파일러가 입력을 보고 계산을 미리 끝내 버리거나 결과를 쓰지 않는
    // 계산을 지워 버리면, 우리가 재려는 일을 재지 못하게 됩니다. 그것을 막으려고 씁니다.
    println!("\n[1] \"the\" 검색: {}줄 중에서", contents.lines().count());
    // 출력: [1] "the" 검색: 180000줄 중에서
    let found_loop = measure("for 반복문", || {
        search_loop(black_box("the"), black_box(&contents))
    });
    let found_iter = measure("반복자    ", || {
        search_iter(black_box("the"), black_box(&contents))
    });
    // 출력:   for 반복문: (실행마다 다름)
    // 출력:   반복자    : (실행마다 다름)
    println!("  찾은 줄 수: {} / {}", found_loop.len(), found_iter.len());
    // 출력:   찾은 줄 수: 40000 / 40000

    println!("\n[2] 짝수의 제곱의 합: 수 {}개", numbers.len());
    // 출력: [2] 짝수의 제곱의 합: 수 2000000개
    let sum_loop = measure("for 반복문", || {
        sum_even_squares_loop(black_box(&numbers))
    });
    let sum_iter = measure("반복자    ", || {
        sum_even_squares_iter(black_box(&numbers))
    });
    // 출력:   for 반복문: (실행마다 다름)
    // 출력:   반복자    : (실행마다 다름)
    println!("  결과: {sum_loop} / {sum_iter}");
    // 출력:   결과: 1333331333334000000 / 1333331333334000000
}
