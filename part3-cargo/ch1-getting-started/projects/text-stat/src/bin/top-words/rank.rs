// `top-words` 바이너리만 쓰는 모듈입니다. 라이브러리(`text_stat`)에는 들어가지 않습니다.

use std::collections::HashMap;

/// 많이 나온 낱말부터 `n`개를 돌려줍니다. 횟수가 같으면 사전 순입니다.
pub fn top(text: &str, n: usize) -> Vec<(&str, usize)> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    let mut ranked: Vec<(&str, usize)> = counts.into_iter().collect();
    // `HashMap`의 순회 순서는 실행마다 다르므로, 출력이 같도록 낱말까지 넣어 정렬합니다.
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    ranked.truncate(n);
    ranked
}
