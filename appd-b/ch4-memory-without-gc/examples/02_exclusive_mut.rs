// Q4.5 — 가변 참조가 한 번에 하나뿐이라서 막히는 일
//
// `append_copy(dst, src)`는 `src`의 원소를 `dst` 뒤에 덧붙입니다. 다른 벡터를 덧붙이는 것은 문제가
// 없지만, 같은 벡터를 `dst`와 `src`로 동시에 넘기는 `append_copy(&mut v, &v)`는 컴파일되지 않습니다
// (compile_fail/e0502_dst_src_alias.rs). `push`는 용량이 모자라면 더 큰 버퍼로 원소를 옮기고 옛 버퍼를
// 해제할 수 있으므로(Part 0 ch8 예제 03의 2번), 허용했다면 아직 옛 버퍼를 읽던 `src`가 해제된
// 메모리를 가리킬 수 있기 때문입니다. 실제로 옮길지는 실행 중에야 정해지므로, 컴파일러는 옮길 수
// 있다고 보고 검사합니다.
//
// 이 예제는 Part 0 ch8의 "요소를 빌린 채 push"를 함수 인자로 옮긴 경우와, 같은 벡터를 자기 뒤에
// 덧붙이는 안전한 방법 두 가지를 봅니다.
//
// 실행: cargo run -p faq-ch4-memory-without-gc --example 02_exclusive_mut

fn append_copy(dst: &mut Vec<i32>, src: &[i32]) {
    for &x in src {
        dst.push(x);
    }
}

fn main() {
    println!("1. 다른 벡터를 덧붙이기");
    let mut a = vec![1, 2, 3];
    let b = vec![4, 5];
    append_copy(&mut a, &b);
    println!("  {a:?}");
    // 출력:   [1, 2, 3, 4, 5]

    println!("2. 같은 벡터를 자기 뒤에 덧붙이기");
    let mut c = vec![1, 2, 3];
    // append_copy(&mut c, &c); // E0502: 같은 값을 가변 참조와 불변 참조로 동시에 빌림
    c.extend_from_within(..); // 표준 라이브러리가 안에서 안전하게 처리합니다.
    println!("  extend_from_within: {c:?}");
    // 출력:   extend_from_within: [1, 2, 3, 1, 2, 3]

    let mut d = vec![1, 2, 3];
    let snapshot = d.clone(); // 또는 사본을 먼저 만들어 다른 벡터로 넘깁니다.
    append_copy(&mut d, &snapshot);
    println!("  사본을 덧붙이기: {d:?}");
    // 출력:   사본을 덧붙이기: [1, 2, 3, 1, 2, 3]
}
