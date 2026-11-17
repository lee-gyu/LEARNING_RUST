// 이 예제가 보여 주는 것: iter / iter_mut / into_iter가 내놓는 값의 차이(&T, &mut T, T)와 소유권,
//                        for x in &v / &mut v / v 가 각각 어느 것과 같은지
// 실행: cargo run -p p0-ch13-closures-iterators --example 05_iter_ownership
//
// 세 메서드는 함수의 매개변수가 값을 받는 세 가지 방식(불변 대여, 가변 대여, 소유권 이동)과 대응합니다.
// 아래에서 `let _check: &String = name;` 같은 줄은 실행에는 의미가 없고, 이 줄이 컴파일된다는 사실로
// 반복 변수의 타입을 확인하려고 넣었습니다. 타입이 다르면 E0308로 컴파일되지 않습니다.

fn main() {
    // 1. iter(): 원소의 불변 참조(&T)를 내놓습니다. 원본은 빌려줄 뿐이므로 그대로 남습니다.
    println!("== 1. iter() → &T ==");
    let names = vec![
        String::from("가람"),
        String::from("나래"),
        String::from("다온"),
    ];
    for name in names.iter() {
        let _check: &String = name;
        println!("  안녕, {name}");
    }
    println!("names는 그대로: {names:?}");
    // 출력:   안녕, 가람
    // 출력:   안녕, 나래
    // 출력:   안녕, 다온
    // 출력: names는 그대로: ["가람", "나래", "다온"]

    // 2. iter_mut(): 원소의 가변 참조(&mut T)를 내놓습니다. 원소를 그 자리에서 바꿀 수 있습니다.
    println!("\n== 2. iter_mut() → &mut T ==");
    let mut scores = vec![70, 85, 92];
    for score in scores.iter_mut() {
        let _check: &mut i32 = score;
        *score += 5; // 참조가 가리키는 정수를 바꾸려면 역참조 *가 필요합니다(ch4).
    }
    println!("5점씩 올린 점수: {scores:?}"); // 출력: 5점씩 올린 점수: [75, 90, 97]

    // 3. into_iter(): 원소 자체(T)를 내놓습니다. 벡터의 소유권을 가져가므로 원본은 더 쓸 수 없습니다.
    println!("\n== 3. into_iter() → T ==");
    let mut members: Vec<String> = Vec::new();
    for name in names.into_iter() {
        let _check: &String = &name; // name은 String 자체입니다(참조가 아님).
        members.push(name); // 복제 없이 String을 그대로 옮겨 담습니다.
    }
    println!("옮겨 담은 이름: {members:?}"); // 출력: 옮겨 담은 이름: ["가람", "나래", "다온"]
    // 여기서 names를 쓰면 error[E0382]: borrow of moved value: `names` 입니다.
    // into_iter()가 names를 가져갔고, 그 안의 String들은 이제 members가 소유합니다.

    // 4. for 문의 세 가지 모양
    println!("\n== 4. for x in &v / &mut v / v ==");
    // for는 in 뒤에 온 값의 into_iter()를 불러 반복자를 얻습니다. 그런데 into_iter()는
    // 무엇에 부르느냐에 따라 내놓는 값이 다릅니다.
    //   for x in &v      →  v.iter()와 같음      (x: &T)
    //   for x in &mut v  →  v.iter_mut()와 같음  (x: &mut T)
    //   for x in v       →  v.into_iter()와 같음 (x: T, v는 이동)
    let mut words = vec![String::from("rust"), String::from("crab")];
    for w in &words {
        let _check: &String = w;
    }
    for w in &mut words {
        let _check: &mut String = w;
        w.push('!'); // 메서드 호출은 자동 역참조 덕분에 *가 필요 없습니다.
    }
    println!("&mut로 고친 뒤: {words:?}"); // 출력: &mut로 고친 뒤: ["rust!", "crab!"]
    for w in words {
        let owned: String = w; // String 자체를 받았으므로 다른 변수로 옮길 수 있습니다.
        println!("  소유한 값: {owned}");
    }
    // 출력:   소유한 값: rust!
    // 출력:   소유한 값: crab!
    // ch3·ch8에서 for x in &v라고 쓰던 것이 바로 v.iter()였습니다.
}
