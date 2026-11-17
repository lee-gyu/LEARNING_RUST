// 기존 값을 바탕으로 해시 맵의 값을 갱신하는 대표적인 예인 단어 빈도수 세기입니다.
// `entry(키).or_insert(0)`이 돌려주는 가변 참조로 개수를 1씩 올립니다.
//
// 실행: cargo run -p p0-ch8-collections --example 09_word_count

use std::collections::HashMap;

fn main() {
    // 점심 메뉴 투표 결과를 공백으로 구분해 적은 문자열입니다.
    let text = "김밥 라면 떡볶이 김밥 라면 김밥 순대 떡볶이 김밥";

    println!("1. entry().or_insert(0)으로 단어마다 개수를 셉니다");
    let mut counts = HashMap::new();
    // split_whitespace()는 공백을 기준으로 나눈 조각(&str)을 하나씩 내줍니다.
    // 각 조각은 text의 일부를 가리키는 슬라이스이므로, 맵의 키(&str)는 text를 빌리고 있습니다.
    for word in text.split_whitespace() {
        // or_insert는 그 키의 값에 대한 가변 참조(&mut i32)를 돌려줍니다.
        // 처음 보는 단어면 0을 넣은 뒤 그 자리의 참조를, 이미 있으면 기존 값의 참조를 돌려줍니다.
        let count = counts.entry(word).or_insert(0);
        *count += 1; // 참조가 가리키는 값을 바꾸려면 *로 역참조합니다.
    } // count의 가변 대여는 반복 한 번이 끝날 때마다 끝나므로, 다음 반복에서 다시 entry를 부를 수 있습니다.

    // 맵의 순회 순서는 실행마다 달라질 수 있으므로, (단어, 개수) 쌍을 벡터에 모아 정렬해서 출력합니다.
    let mut pairs = Vec::new();
    for (word, count) in &counts {
        // 맵의 키가 &str이므로 word는 그 참조인 &&str, count는 &i32입니다.
        // *로 한 겹 벗겨 값을 담습니다(&str도 i32도 Copy라 복사됩니다).
        pairs.push((*word, *count));
    }
    pairs.sort(); // 튜플은 첫 번째 요소(단어)부터 비교하므로 가나다순으로 정렬됩니다.
    for (word, count) in &pairs {
        println!("    {word}: {count}표");
    }
    // 출력:
    //     김밥: 4표
    //     떡볶이: 2표
    //     라면: 2표
    //     순대: 1표

    println!("2. 가장 많이 나온 단어를 찾습니다");
    let mut best_word = "";
    let mut best_count = 0;
    for (word, count) in &pairs {
        // 개수가 같으면 먼저 본(가나다순으로 앞선) 단어가 남습니다. 정렬된 pairs를 쓰므로 결과가 항상 같습니다.
        // pairs를 빌려 순회하므로 word는 &&str, count는 &i32입니다. *로 한 겹 벗겨 값을 담습니다.
        if *count > best_count {
            best_word = *word;
            best_count = *count;
        }
    }
    println!("    1위: {best_word} ({best_count}표)"); // 출력: 1위: 김밥 (4표)

    println!("3. entry 없이 get과 insert로도 셀 수 있지만, 키를 두 번 찾게 됩니다");
    let mut manual = HashMap::new();
    for word in text.split_whitespace() {
        // 먼저 get으로 지금 개수를 읽어 오고(없으면 0), 1을 더한 값을 insert로 덮어씁니다.
        let current = match manual.get(word) {
            Some(n) => *n,
            None => 0,
        };
        manual.insert(word, current + 1);
    }
    // 두 맵의 내용이 같은지 ==로 비교합니다. 순서와 상관없이 (키, 값)이 모두 같으면 true입니다.
    println!("    두 방법의 결과가 같은가? {}", manual == counts); // 출력: 두 방법의 결과가 같은가? true
}
