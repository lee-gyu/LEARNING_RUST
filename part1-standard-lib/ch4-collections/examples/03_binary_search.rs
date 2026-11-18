// 이 예제가 보여 주는 것: 정렬된 슬라이스에서의 이진 탐색(binary_search 계열)과 Ok/Err의 뜻,
//                        partition_point로 경계 찾기, 정렬 상태를 유지하며 삽입하기
// 실행: cargo run -p p1-ch4-collections --example 03_binary_search
//
// contains는 앞에서부터 하나씩 비교하므로 O(n)입니다. 슬라이스가 정렬되어 있다면
// 가운데 값과 비교해 절반씩 버려 나가는 이진 탐색으로 O(log n)에 찾을 수 있습니다.
// 요소가 100만 개여도 비교는 20번 정도면 됩니다.

fn main() {
    // 1. binary_search는 Result<usize, usize>를 돌려줍니다.
    //    Ok(i)  : 찾았고, i번 자리에 있습니다.
    //    Err(i) : 없습니다. 하지만 i번 자리에 넣으면 정렬 상태가 유지됩니다.
    println!("== 1. binary_search의 Ok와 Err ==");
    let floors = [10, 20, 30, 40, 50];
    for target in [30, 35, 5, 99] {
        println!("{target:>2} 찾기: {:?}", floors.binary_search(&target));
    }
    // 출력:
    // 30 찾기: Ok(2)
    // 35 찾기: Err(3)
    //  5 찾기: Err(0)
    // 99 찾기: Err(5)

    // 2. Err(i)의 i를 쓰면 정렬 상태를 유지하며 삽입할 수 있습니다.
    println!("\n== 2. 정렬 상태를 유지하며 삽입 ==");
    let mut booked = vec![9, 13, 15]; // 예약된 시각(정렬되어 있고 중복 없음)
    for hour in [11, 13, 8, 17] {
        match booked.binary_search(&hour) {
            Ok(_) => println!("{hour}시: 이미 예약됨"),
            Err(pos) => {
                booked.insert(pos, hour); // pos 뒤의 요소를 밀어야 하므로 insert 자체는 O(n)입니다.
                println!("{hour}시: {pos}번 자리에 추가 → {booked:?}");
            }
        }
    }
    // 출력:
    // 11시: 1번 자리에 추가 → [9, 11, 13, 15]
    // 13시: 이미 예약됨
    // 8시: 0번 자리에 추가 → [8, 9, 11, 13, 15]
    // 17시: 5번 자리에 추가 → [8, 9, 11, 13, 15, 17]

    // 3. 같은 값이 여러 개면 Ok는 그중 "아무거나" 하나를 가리킵니다.
    //    같은 값의 범위가 필요하면 partition_point로 양 끝을 찾습니다.
    println!("\n== 3. 중복 값과 partition_point ==");
    let scores = [55, 70, 80, 80, 80, 95];
    // 어느 80을 가리킬지는 정해져 있지 않습니다. 아래 값은 이 툴체인에서의 결과입니다.
    println!("binary_search(&80) = {:?}", scores.binary_search(&80)); // 출력: binary_search(&80) = Ok(4)
    // partition_point(조건)은 "조건이 true인 앞부분"과 "false인 뒷부분"의 경계 인덱스를 돌려줍니다.
    // 슬라이스가 그 조건으로 앞뒤가 나뉘어 있어야(정렬되어 있어야) 의미가 있습니다.
    let lo = scores.partition_point(|&s| s < 80); // 80보다 작은 값이 끝나는 곳 = 첫 80의 위치
    let hi = scores.partition_point(|&s| s <= 80); // 80 이하가 끝나는 곳 = 마지막 80 다음 위치
    println!("80의 범위: {lo}..{hi}, 개수 = {}", hi - lo); // 출력: 80의 범위: 2..5, 개수 = 3
    let below_60 = scores.partition_point(|&s| s < 60);
    println!("60점 미만인 학생 수: {below_60}"); // 출력: 60점 미만인 학생 수: 1

    // 중복을 허용하며 삽입할 때는 partition_point(|x| x <= 새 값)을 쓰면
    // 같은 값들의 맨 뒤에 들어가므로 먼저 들어온 값의 순서가 유지됩니다.
    let mut ranking = vec![55, 70, 80, 80, 95];
    let pos = ranking.partition_point(|&s| s <= 80);
    ranking.insert(pos, 80);
    println!("80 추가 위치: {pos}, 결과: {ranking:?}"); // 출력: 80 추가 위치: 4, 결과: [55, 70, 80, 80, 80, 95]

    // 4. 구조체 목록은 binary_search_by_key나 binary_search_by로 찾습니다.
    println!("\n== 4. binary_search_by_key / binary_search_by ==");
    // (역 번호, 역 이름) 목록. 역 번호 기준으로 정렬되어 있습니다.
    let stations = [
        (150, "서울역"),
        (151, "시청"),
        (152, "종각"),
        (153, "종로3가"),
        (154, "종로5가"),
    ];
    // binary_search_by_key(&찾을 키, 요소에서 키를 뽑는 클로저)
    match stations.binary_search_by_key(&152, |&(code, _)| code) {
        Ok(i) => println!("152번: {}", stations[i].1), // 출력: 152번: 종각
        Err(_) => println!("152번: 없음"),
    }
    // binary_search_by(요소와 목표를 비교해 Ordering을 돌려주는 클로저)
    // 클로저는 "이 요소가 목표보다 작은지/같은지/큰지"를 돌려줘야 합니다(요소.cmp(목표) 방향).
    let found = stations.binary_search_by(|&(code, _)| code.cmp(&160));
    println!("160번: {found:?}"); // 출력: 160번: Err(5)

    // 5. 정렬되지 않은 슬라이스에서는 결과가 의미 없습니다.
    println!("\n== 5. 정렬되지 않은 슬라이스 ==");
    let messy = [5, 1, 4, 2, 3];
    println!("정렬되어 있는가? {}", messy.is_sorted()); // 출력: 정렬되어 있는가? false
    println!("contains(&5) = {}", messy.contains(&5)); // 출력: contains(&5) = true
    // 5가 분명히 들어 있는데도 Err가 나옵니다. 값은 이 툴체인에서의 결과이고, 어떤 값이 나오든 믿을 수 없습니다.
    println!("binary_search(&5) = {:?}", messy.binary_search(&5)); // 출력: binary_search(&5) = Err(5)
}
