// Q3.2 · Q3.3 — `unsafe`는 안전한 API 안쪽에 갇힌다
//
//   1. 표준 라이브러리의 `split_at_mut`: 한 `Vec`에서 겹치지 않는 가변 참조 둘을 얻습니다.
//      대여 검사기만으로는 증명할 수 없는 일이라 표준 라이브러리 안에서 `unsafe`로 구현되어
//      있지만, 부르는 쪽은 안전한 함수로 씁니다(Part 0 ch19 2.3절).
//   2. 직접 만든 작은 안전한 래퍼: 경계 검사를 건너뛰는 `get_unchecked`(unsafe 함수)를 쓰되,
//      바로 위에서 전제 조건을 확인하고 `// SAFETY:` 주석으로 적습니다.
//
// 이 저장소에는 Miri가 없어 아래 `unsafe` 블록을 Miri로 검사하지 못했습니다. 블록의 전제 조건은
// 바로 위 코드가 확인합니다.
//
// 실행: cargo run -p faq-ch3-safety-limits --example 05_unsafe_boundary

/// 비어 있지 않으면 가운데 원소를 돌려줍니다.
fn middle(values: &[i32]) -> Option<i32> {
    if values.is_empty() {
        return None;
    }
    let mid = values.len() / 2;
    // SAFETY: 위에서 `values`가 비어 있지 않음을 확인했으므로 len >= 1이고,
    // mid = len / 2 < len 입니다. 따라서 mid는 범위 안의 인덱스입니다.
    Some(unsafe { *values.get_unchecked(mid) })
}

fn main() {
    println!("1. split_at_mut: 안전한 함수 안의 unsafe");
    let mut scores = vec![10, 20, 30, 40];
    let (front, back) = scores.split_at_mut(2);
    front[0] += back[0]; // 같은 Vec의 두 부분을 동시에 가변으로 빌림
    println!("  {scores:?}");
    // 출력:   [40, 20, 30, 40]

    println!("2. 직접 만든 안전한 래퍼");
    println!("  middle([1, 2, 3]) = {:?}", middle(&[1, 2, 3]));
    // 출력:   middle([1, 2, 3]) = Some(2)
    println!("  middle([]) = {:?}", middle(&[]));
    // 출력:   middle([]) = None
    // 부르는 쪽은 어떤 슬라이스를 넘겨도 범위 밖을 읽을 수 없습니다. 검토할 곳은 `middle` 하나입니다.
}
