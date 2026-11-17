// 벡터를 for 문으로 순회하는 세 가지 방법을 비교합니다.
// `&v`(읽기), `&mut v`(수정), `v`(소유권까지 넘김) 중 무엇을 쓰느냐에 따라 할 수 있는 일이 달라집니다.
//
// 실행: cargo run -p p0-ch8-collections --example 02_vec_iterate

fn main() {
    println!("1. for x in &v: 요소를 하나씩 빌려 읽습니다");
    let scores = vec![72, 85, 90];
    let mut total = 0;
    for score in &scores {
        // score의 타입은 &i32(요소의 불변 참조)입니다.
        // 덧셈 같은 산술 연산은 참조에도 바로 쓸 수 있습니다.
        total += score;
    }
    // 불변 대여였으므로 반복이 끝난 뒤에도 scores를 그대로 쓸 수 있습니다.
    println!("    scores = {scores:?}, 합계 = {total}"); // 출력: scores = [72, 85, 90], 합계 = 247

    println!("2. for x in &mut v: 요소를 하나씩 빌려 그 자리에서 고칩니다");
    let mut prices = vec![1000, 2500, 4000];
    for price in &mut prices {
        // price의 타입은 &mut i32입니다. 참조가 가리키는 값 자체를 바꾸려면
        // 역참조 연산자 *로 참조를 따라가야 합니다(ch4).
        *price += 500;
    }
    println!("    prices = {prices:?}"); // 출력: prices = [1500, 3000, 4500]

    println!("3. 인덱스가 함께 필요하면 iter().enumerate()를 씁니다");
    let mut menu = vec!["김밥", "라면"];
    menu.push("떡볶이"); // 벡터는 배열과 달리 실행 중에 길이가 늘어납니다.
    // enumerate는 (인덱스, 요소의 참조) 쌍을 차례로 내줍니다(ch3에서 배열에 썼던 것과 같습니다).
    for (i, item) in menu.iter().enumerate() {
        println!("    {}번: {item}", i + 1);
    }
    // 출력:
    //     1번: 김밥
    //     2번: 라면
    //     3번: 떡볶이

    println!("4. for x in v: 벡터의 소유권까지 넘겨 요소를 하나씩 꺼냅니다");
    let names = vec![String::from("민수"), String::from("지영")];
    for name in names {
        // name의 타입은 &String이 아니라 String입니다. 요소의 소유권이 name으로 옮겨 왔습니다.
        println!("    {name}님, 안녕하세요");
    }
    // 출력:
    //     민수님, 안녕하세요
    //     지영님, 안녕하세요
    //
    // 반복이 시작될 때 names의 소유권이 for 문으로 이동했으므로, 여기서부터 names는 쓸 수 없습니다.
    // 아래 줄의 주석을 풀면 E0382(borrow of moved value: `names`)가 납니다.
    // println!("{names:?}");
    println!("    반복이 끝난 뒤 names는 더 이상 쓸 수 없습니다 (소유권이 이동함)");
    // 출력: 반복이 끝난 뒤 names는 더 이상 쓸 수 없습니다 (소유권이 이동함)
}
