//! 라이프타임 매개변수를 나눌 때와 묶을 때: 결과가 실제로 어느 입력에서 오는지를 그대로 적습니다.
//!
//! 실행: cargo run -p er-ch14-lifetimes --example 03_split_lifetimes
//!
//! 라이프타임 이름이 말할 수 있는 것은 "이 둘은 같다(같은 구간 안에 들어간다)"는 관계뿐입니다.
//! 서로 다른 출처를 같은 이름으로 묶으면, 결과는 두 출처 중 짧게 사는 쪽에 맞춰집니다.
//! 받는 쪽의 본문은 그대로인데 부르는 쪽에서 정당한 코드가 거부되는 것은 이 때문입니다.

/// 비권장: 검색어(query)와 용어집(glossary)이라는 두 출처를 라이프타임 하나로 묶은 결과
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

/// 권장: 출처마다 라이프타임을 따로 둔 결과. 'g는 용어집, 'q는 검색어입니다.
struct Hit<'g, 'q> {
    query: &'q str,
    meaning: &'g str,
}

fn look_up<'g, 'q>(glossary: &'g str, query: &'q str) -> Option<Hit<'g, 'q>> {
    glossary.lines().find_map(|line| {
        let (term, meaning) = line.split_once(": ")?;
        (term == query).then_some(Hit { query, meaning })
    })
}

/// 묶어야 하는 경우: 결과가 조건에 따라 용어집에서도, 검색어에서도 올 수 있습니다.
/// 이때는 두 입력이 모두 살아 있는 구간이 결과의 구간이라고 적는 것이 사실 그대로입니다.
fn meaning_or_query<'a>(glossary: &'a str, query: &'a str) -> &'a str {
    look_up_bound(glossary, query).map_or(query, |hit| hit.meaning)
}

fn main() {
    // 용어집은 실행 중에 만든 String이고 main 끝까지 삽니다(파일에서 읽었다고 생각합니다).
    let glossary = ["borrow: 대여", "lifetime: 라이프타임", "trait: 트레이트"].join("\n");
    // 사용자가 입력한 검색어. 정규화할 때마다 새 String이 생기고 그 반복이 끝나면 drop됩니다.
    let inputs = ["  Borrow ", "LIFETIME", "closure", "trait"];

    println!("1. 비권장: 두 출처를 'a 하나로 묶은 결과");
    let mut copied: Vec<String> = Vec::new();
    for raw in inputs {
        let query = raw.trim().to_lowercase();
        if let Some(hit) = look_up_bound(&glossary, &query) {
            println!("    {} → {}", hit.query, hit.meaning);
            // hit.meaning은 용어집을 가리키지만 타입은 &'a str이고, 'a는 query에 맞춰져 있습니다.
            // 그래서 반복 밖의 Vec<&str>에 넣을 수 없고(E0597), 복사해서 넣어야 합니다.
            copied.push(hit.meaning.to_string());
        }
    }
    println!("    찾은 뜻 {:?}, 복사한 String {}개", copied, copied.len());
    // 출력:
    //     borrow → 대여
    //     lifetime → 라이프타임
    //     trait → 트레이트
    //     찾은 뜻 ["대여", "라이프타임", "트레이트"], 복사한 String 3개

    println!("2. 권장: 출처마다 라이프타임을 따로 둔 결과");
    let mut found: Vec<&str> = Vec::new();
    for raw in inputs {
        let query = raw.trim().to_lowercase();
        if let Some(hit) = look_up(&glossary, &query) {
            println!("    {} → {}", hit.query, hit.meaning);
            found.push(hit.meaning); // &'g str: 용어집이 살아 있는 동안 유효합니다.
        }
    } // query는 여기서 drop되지만 found에 든 참조는 query를 빌리지 않습니다.
    println!("    찾은 뜻 {:?}, 복사한 String 0개", found);
    // 출력:
    //     borrow → 대여
    //     lifetime → 라이프타임
    //     trait → 트레이트
    //     찾은 뜻 ["대여", "라이프타임", "트레이트"], 복사한 String 0개

    println!("3. 묶어야 하는 경우: 결과가 어느 입력에서든 올 수 있을 때");
    for raw in inputs {
        let query = raw.trim().to_lowercase();
        let shown = meaning_or_query(&glossary, &query);
        // shown은 query에서 왔을 수도 있으므로 이 반복 안에서만 씁니다.
        println!("    {query} → {shown}");
    }
    // 출력:
    //     borrow → 대여
    //     lifetime → 라이프타임
    //     closure → closure
    //     trait → 트레이트
}
