// The Book 8장 끝의 연습 과제 풀이 ①: 정수 목록의 중앙값(median)과 최빈값(mode)
// 중앙값은 벡터를 정렬해서, 최빈값은 해시 맵으로 개수를 세서 구합니다.
//
// 실행: cargo run -p p0-ch8-collections --example 10_median_mode

use std::collections::HashMap;

// 중앙값: 정렬했을 때 가운데에 오는 값입니다. 개수가 짝수면 가운데 두 값의 평균으로 정합니다.
// 빈 목록에는 중앙값이 없으므로 Option으로 돌려줍니다.
// 매개변수를 &[i32](슬라이스, ch4)로 받으면 벡터든 배열이든 넘길 수 있습니다.
fn median(numbers: &[i32]) -> Option<f64> {
    if numbers.is_empty() {
        return None;
    }
    // 빌려 온 원본을 정렬하면 호출한 쪽의 데이터가 바뀌므로, to_vec()으로 복사한 새 벡터를 정렬합니다.
    let mut sorted = numbers.to_vec();
    sorted.sort(); // 오름차순으로 그 자리에서 정렬합니다.

    let mid = sorted.len() / 2; // 정수 나눗셈이므로 소수점 아래는 버려집니다.
    if sorted.len() % 2 == 1 {
        Some(sorted[mid] as f64)
    } else {
        // 두 i32를 먼저 더하면 넘칠 수 있으므로 f64로 바꾼 뒤에 더합니다.
        Some((sorted[mid - 1] as f64 + sorted[mid] as f64) / 2.0)
    }
}

// 최빈값: 가장 많이 나온 값입니다. 공동 1위가 여럿일 수 있으므로 모두 벡터에 담아 돌려줍니다.
fn modes(numbers: &[i32]) -> Vec<i32> {
    // 1단계: 값마다 몇 번 나왔는지 셉니다(09_word_count와 같은 방법).
    let mut counts = HashMap::new();
    for n in numbers {
        // n은 &i32이므로 *n으로 값을 꺼내 키로 씁니다.
        let count = counts.entry(*n).or_insert(0);
        *count += 1;
    }

    // 2단계: 가장 큰 개수를 찾습니다. 키는 필요 없으므로 values()로 값(&i32)만 하나씩 꺼냅니다.
    let mut max_count = 0;
    for count in counts.values() {
        if *count > max_count {
            max_count = *count;
        }
    }

    // 3단계: 그 개수만큼 나온 값을 모두 모읍니다.
    // 맵의 순회 순서는 정해져 있지 않으므로, 결과를 정렬해 항상 같은 순서로 돌려줍니다.
    let mut result = Vec::new();
    for (n, count) in &counts {
        if *count == max_count {
            result.push(*n);
        }
    }
    result.sort();
    result
}

fn main() {
    println!("1. 개수가 홀수인 목록");
    let odd = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
    println!("    목록: {odd:?}");
    println!("    중앙값: {:?}", median(&odd));
    println!("    최빈값: {:?}", modes(&odd));
    // 출력:
    //     목록: [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5]
    //     중앙값: Some(4.0)
    //     최빈값: [5]
    // 정렬하면 [1, 1, 2, 3, 3, 4, 5, 5, 5, 6, 9]이고, 가운데(5번 인덱스)는 4입니다.
    // median 안에서 복사본을 정렬했으므로 odd의 순서는 그대로입니다.

    println!("2. 개수가 짝수이고 최빈값이 둘인 목록");
    let even = vec![10, 40, 20, 30, 40, 10];
    println!("    목록: {even:?}");
    println!("    중앙값: {:?}", median(&even));
    println!("    최빈값: {:?}", modes(&even));
    // 출력:
    //     목록: [10, 40, 20, 30, 40, 10]
    //     중앙값: Some(25.0)
    //     최빈값: [10, 40]
    // 정렬하면 [10, 10, 20, 30, 40, 40]이고, 가운데 두 값 20과 30의 평균은 25입니다.

    println!("3. 빈 목록");
    let empty: Vec<i32> = Vec::new();
    println!("    중앙값: {:?}", median(&empty));
    println!("    최빈값: {:?}", modes(&empty));
    // 출력:
    //     중앙값: None
    //     최빈값: []
}
