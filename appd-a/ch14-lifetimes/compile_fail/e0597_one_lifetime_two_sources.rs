// 기대하는 에러: E0597
// 핵심 메시지: `query` does not live long enough
//              note: `found` is a collection that stores borrowed references, but `query` does not live long enough to be stored in it
//
// look_up_bound는 결과의 meaning을 언제나 용어집(glossary)에서 가져옵니다. 그런데 시그니처가
// 용어집과 검색어(query)를 라이프타임 'a 하나로 묶었기 때문에, 부르는 쪽에서 'a는 둘 중 짧게
// 사는 query에 맞춰집니다. 반복마다 drop되는 query보다 오래 사는 found에 hit.meaning을 넣으면
// 거부됩니다. 받는 쪽 본문은 그대로 두고 Hit<'g, 'q>처럼 출처마다 라이프타임을 나누면 통과합니다
// (examples/03_split_lifetimes.rs의 2번 구역).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0597_one_lifetime_two_sources.rs --out-dir /tmp/er-ch14

struct BoundHit<'a> {
    query: &'a str,
    meaning: &'a str,
}

fn look_up_bound<'a>(glossary: &'a str, query: &'a str) -> Option<BoundHit<'a>> {
    glossary.lines().find_map(|line| {
        let (term, meaning) = line.split_once(": ")?;
        (term == query).then_some(BoundHit { query, meaning })
    })
}

fn main() {
    let glossary = ["borrow: 대여", "lifetime: 라이프타임"].join("\n");
    let mut found: Vec<&str> = Vec::new();
    for raw in ["  Borrow ", "LIFETIME"] {
        let query = raw.trim().to_lowercase();
        if let Some(hit) = look_up_bound(&glossary, &query) {
            println!("{} → {}", hit.query, hit.meaning);
            found.push(hit.meaning); // E0597: hit.meaning의 타입이 query에 묶여 있음
        }
    }
    println!("{found:?}");
}
