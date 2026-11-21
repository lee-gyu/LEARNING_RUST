// Q9.7 — 순회하면서 컨테이너를 고치기
//
// C++에서 범위 기반 for로 vector를 돌며 push_back하면, 용량이 모자랄 때 원소가 새 공간으로 옮겨지고
// 순회 중인 반복자가 해제된 옛 공간을 가리킵니다(compare/06_iterator_invalidation.cpp). Rust에서 같은 모양
// (`for x in &v { v.push(…) }`)은 순회가 v를 빌린 채로 v를 고치려 하므로 E0502로 컴파일되지 않습니다
// (Part 0 ch8 4.1절, 맵에서 지우는 경우는 Part 1 ch4 4.4절). 그러면 같은 일은 어떻게 쓰는지 봅니다.
//
//   1. 더할 것을 모아 두었다가 순회가 끝난 뒤 extend
//   2. 인덱스로 순회: 시작할 때의 길이까지만 보고, 새로 넣은 원소는 보지 않음
//   3. 조건에 맞는 것 지우기는 retain(Part 1 ch4 예제 01)
//
// 실행: cargo run -p faq-ch9-c-memory-errors --example 03_modify_while_iterating

fn main() {
    println!("1. 모았다가 나중에 더하기");
    let mut v = vec![1, 2, 3];
    let extra: Vec<i32> = v.iter().filter(|&&x| x == 2).map(|_| 4).collect();
    v.extend(extra); // 순회(빌림)가 끝난 뒤에 고침
    println!("  {v:?}");
    // 출력:   [1, 2, 3, 4]

    println!("2. 인덱스로 순회하며 더하기");
    let mut w = vec![1, 2, 3];
    let len = w.len(); // 시작할 때의 길이
    for i in 0..len {
        // w[i]를 읽는 빌림은 이 줄에서 끝나므로, 같은 반복 안에서 push할 수 있습니다.
        if w[i] == 2 {
            w.push(4);
        }
    }
    println!("  {w:?}");
    // 출력:   [1, 2, 3, 4]

    println!("3. 조건에 맞는 것 지우기");
    let mut z = vec![1, 2, 3, 4, 5];
    z.retain(|&x| x % 2 == 1);
    println!("  {z:?}");
    // 출력:   [1, 3, 5]
}
