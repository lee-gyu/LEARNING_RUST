// 이 예제가 보여 주는 것: Vec의 중간 삽입·삭제와 그 비용(insert, remove, swap_remove),
//                        여러 요소를 한꺼번에 다루는 메서드(extend, append, drain, retain, extract_if,
//                        dedup, truncate, split_off), 용량 관리(with_capacity, reserve, shrink_to_fit)
// 실행: cargo run -p p1-ch4-collections --example 01_vec_edit
//
// Vec은 요소를 메모리에 빈틈없이 이어 붙여 둡니다. 그래서 끝에서 넣고 빼는 일은 싸지만,
// 중간에 넣거나 빼면 그 뒤의 요소를 모두 한 칸씩 밀거나 당겨야 합니다.
// 이 예제는 "무엇을 쓰면 몇 개를 옮기게 되는가"를 의식하며 메서드를 고르는 연습입니다.

fn main() {
    // 1. insert와 remove: 중간에 넣고 빼면 뒤의 요소가 모두 움직입니다.
    println!("== 1. insert / remove ==");
    let mut playlist = vec!["봄", "여름", "가을", "겨울"];
    // insert(1, ..)는 1번 자리부터 끝까지(3개)를 오른쪽으로 한 칸씩 민 뒤 빈자리에 넣습니다. O(n - i)
    playlist.insert(1, "장마");
    println!("insert(1, \"장마\") 후: {playlist:?}"); // 출력: insert(1, "장마") 후: ["봄", "장마", "여름", "가을", "겨울"]
    // remove(0)은 0번을 꺼낸 뒤 나머지 4개를 왼쪽으로 한 칸씩 당깁니다. 꺼낸 값의 소유권은 돌려받습니다.
    let first = playlist.remove(0);
    println!("remove(0) = {first}, 남은 목록: {playlist:?}"); // 출력: remove(0) = 봄, 남은 목록: ["장마", "여름", "가을", "겨울"]
    // insert(len, ..)은 끝에 넣는 것이라 허용되지만, len보다 큰 인덱스는 패닉합니다.
    // remove도 범위 밖 인덱스에서 패닉하므로, 확실하지 않으면 먼저 len()으로 확인합니다.
    playlist.insert(playlist.len(), "입춘");
    println!("끝에 insert: {playlist:?}"); // 출력: 끝에 insert: ["장마", "여름", "가을", "겨울", "입춘"]

    // 2. swap_remove: 순서를 포기하면 O(1)로 지울 수 있습니다.
    println!("\n== 2. swap_remove ==");
    let mut players = vec!["가온", "나래", "다솜", "라온", "마루"];
    // 지운 자리에 마지막 요소를 옮겨 넣으므로, 다른 요소는 하나도 밀리지 않습니다.
    let out = players.swap_remove(1);
    println!("swap_remove(1) = {out}, 남은 목록: {players:?}"); // 출력: swap_remove(1) = 나래, 남은 목록: ["가온", "마루", "다솜", "라온"]
    // 순서가 의미 없는 모음(게임의 살아 있는 적 목록 등)에서 쓰기 좋습니다.

    // 3. extend와 append: 다른 모음의 요소를 한꺼번에 끝에 붙입니다.
    println!("\n== 3. extend / append ==");
    let mut cart = vec![1200, 3500];
    // extend는 배열, 슬라이스의 반복자, 다른 Vec 등 "하나씩 꺼낼 수 있는 것"이면 무엇이든 받습니다.
    cart.extend([800, 1500]);
    let more = vec![2000, 900];
    cart.extend(more.iter()); // 참조를 내놓는 반복자도 받습니다(i32는 Copy라 값이 복사되어 들어감).
    println!("extend 후: {cart:?}, more는 그대로: {more:?}"); // 출력: extend 후: [1200, 3500, 800, 1500, 2000, 900], more는 그대로: [2000, 900]
    // append는 다른 Vec의 요소를 모두 옮겨 오고, 그 Vec은 빈 채로 남깁니다.
    let mut coupon_items = vec![100, 200];
    cart.append(&mut coupon_items);
    println!("append 후: {cart:?}, coupon_items: {coupon_items:?}"); // 출력: append 후: [1200, 3500, 800, 1500, 2000, 900, 100, 200], coupon_items: []

    // 4. drain: 범위를 잘라 내면서 잘라 낸 요소를 받습니다.
    println!("\n== 4. drain ==");
    let mut jobs = vec!["빨래", "설거지", "청소", "장보기", "분리수거"];
    // 앞의 3개를 한 번에 처리하려고 꺼냅니다. remove(0)을 세 번 하면 뒤의 요소를 세 번 당기지만,
    // drain(..3)은 범위를 지운 뒤 남은 요소를 한 번만 당깁니다.
    let batch: Vec<&str> = jobs.drain(..3).collect();
    println!("이번에 처리: {batch:?}"); // 출력: 이번에 처리: ["빨래", "설거지", "청소"]
    println!("남은 일: {jobs:?}"); // 출력: 남은 일: ["장보기", "분리수거"]

    // 5. retain과 extract_if: 조건에 맞지 않는 요소를 한 번에 지웁니다.
    println!("\n== 5. retain / extract_if ==");
    let mut scores = vec![72, 45, 90, 38, 85, 60];
    // retain은 클로저가 true를 돌려준 요소만 남깁니다. 앞에서부터 한 번씩만 보고, 남는 요소의 순서는 유지합니다.
    // for 문에서 remove(i)를 부르며 지우면 인덱스가 밀려 요소를 건너뛰기 쉽고, 지울 때마다 O(n)이 듭니다.
    scores.retain(|&s| s >= 60);
    println!("60점 이상만: {scores:?}"); // 출력: 60점 이상만: [72, 90, 85, 60]
    // 지운 요소도 필요하면 extract_if(범위, 조건)를 씁니다. 조건이 true인 요소를 빼내 내놓습니다.
    let mut inbox = vec!["광고1", "공지", "광고2", "회의", "광고3"];
    let spam: Vec<&str> = inbox.extract_if(.., |m| m.starts_with("광고")).collect();
    println!("스팸함: {spam:?}, 받은편지함: {inbox:?}"); // 출력: 스팸함: ["광고1", "광고2", "광고3"], 받은편지함: ["공지", "회의"]

    // 6. dedup: "연속된" 중복만 지웁니다.
    println!("\n== 6. dedup ==");
    let mut readings = vec![3, 3, 5, 5, 5, 3, 7, 7];
    readings.dedup();
    println!("dedup: {readings:?}"); // 출력: dedup: [3, 5, 3, 7]
    // 떨어져 있는 3은 남았습니다. 모든 중복을 지우려면 먼저 정렬해서 같은 값끼리 붙여 놓습니다.
    readings.sort();
    readings.dedup();
    println!("sort 후 dedup: {readings:?}"); // 출력: sort 후 dedup: [3, 5, 7]

    // 7. truncate와 split_off: 길이를 줄이거나 둘로 나눕니다.
    println!("\n== 7. truncate / split_off ==");
    let mut ranking = vec!["A", "B", "C", "D", "E", "F"];
    // split_off(at)는 at번부터 끝까지를 새 Vec으로 떼어 돌려주고, 원래 Vec에는 앞부분만 남깁니다.
    let rest = ranking.split_off(3);
    println!("상위: {ranking:?}, 나머지: {rest:?}"); // 출력: 상위: ["A", "B", "C"], 나머지: ["D", "E", "F"]
    // truncate(n)은 앞의 n개만 남기고 나머지를 drop합니다. 떼어 낸 요소가 필요 없을 때 씁니다.
    ranking.truncate(1);
    println!("truncate(1): {ranking:?}"); // 출력: truncate(1): ["A"]

    // 8. 용량 관리: 미리 확보하고, 다 쓴 뒤에 돌려줍니다.
    // 용량 값은 이 툴체인(rustc 1.98.1, macOS)에서의 결과입니다.
    // 표준 라이브러리는 "최소한 요청한 만큼"만 보장하므로 다른 환경에서는 더 클 수 있습니다.
    println!("\n== 8. with_capacity / reserve / shrink_to_fit ==");
    let mut log: Vec<u32> = Vec::with_capacity(10);
    println!(
        "with_capacity(10): len = {}, capacity = {}",
        log.len(),
        log.capacity()
    );
    // 출력: with_capacity(10): len = 0, capacity = 10
    log.extend(1..=10); // 10개까지는 재할당 없이 들어갑니다.
    println!(
        "10개 넣은 뒤: len = {}, capacity = {}",
        log.len(),
        log.capacity()
    );
    // 출력: 10개 넣은 뒤: len = 10, capacity = 10
    // 곧 100개를 더 넣을 것을 안다면 reserve로 한 번에 확보합니다. 그러면 재할당이 한 번으로 끝납니다.
    log.reserve(100);
    println!(
        "reserve(100) 뒤: capacity >= 110? {}",
        log.capacity() >= 110
    );
    // 출력: reserve(100) 뒤: capacity >= 110? true
    // clear와 truncate는 요소만 지우고 용량은 그대로 둡니다. 같은 Vec을 다시 채울 때 할당이 필요 없게 하려는 것입니다.
    log.clear();
    println!(
        "clear 뒤: len = {}, capacity >= 110? {}",
        log.len(),
        log.capacity() >= 110
    );
    // 출력: clear 뒤: len = 0, capacity >= 110? true
    // 더 이상 늘어나지 않을 것이 확실하면 shrink_to_fit으로 남는 공간을 돌려줍니다.
    log.extend([7, 8, 9]);
    log.shrink_to_fit();
    println!(
        "shrink_to_fit 뒤: len = {}, capacity = {}",
        log.len(),
        log.capacity()
    );
    // 출력: shrink_to_fit 뒤: len = 3, capacity = 3
}
