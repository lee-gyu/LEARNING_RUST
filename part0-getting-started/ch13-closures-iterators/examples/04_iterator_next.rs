// 이 예제가 보여 주는 것: Iterator 트레이트의 next, 반복자는 게으르다(소비하기 전에는 아무 일도 하지 않는다),
//                        for 문은 반복자의 next를 None이 나올 때까지 부르는 문법이라는 것
// 실행: cargo run -p p0-ch13-closures-iterators --example 04_iterator_next
//
// 표준 라이브러리의 Iterator 트레이트는 대략 이렇게 생겼습니다.
//
//     pub trait Iterator {
//         type Item;                                   // 이 반복자가 내놓는 값의 타입
//         fn next(&mut self) -> Option<Self::Item>;    // 다음 값을 Some으로, 끝나면 None
//         // ... 기본 구현이 있는 메서드 수십 개(map, filter, sum, collect 등)
//     }
//
// 구현하는 쪽은 next 하나만 만들면 되고, 나머지 메서드는 모두 next를 써서 기본 구현으로 따라옵니다.

// for 문이 실제로 하는 일을 손으로 풀어 쓴 함수입니다.
// clippy는 "이 loop는 while let으로 쓸 수 있다"고 제안합니다. while let은 if let을 조건이 맞는 동안
// 되풀이하는 문법(ch18)인데, 여기서는 next와 None의 관계를 그대로 보여 주려고 loop와 match로 씁니다.
#[expect(
    clippy::while_let_loop,
    reason = "for 문의 동작을 loop와 match로 풀어 보여 주는 예제"
)]
fn manual_for(words: &[&str]) {
    let mut iter = words.iter(); // for 문도 맨 처음에 반복자를 하나 얻습니다.
    loop {
        match iter.next() {
            Some(word) => println!("  손으로 쓴 반복: {word}"),
            None => break, // None이 나오면 반복을 끝냅니다.
        }
    }
}

fn main() {
    // 1. next로 하나씩 꺼내기
    println!("== 1. next ==");
    let v1 = vec![1, 2, 3];
    // next는 반복자 안의 "현재 위치"를 바꾸므로(&mut self) 반복자 변수에 mut이 필요합니다.
    let mut v1_iter = v1.iter();
    println!("{:?}", v1_iter.next()); // 출력: Some(1)
    println!("{:?}", v1_iter.next()); // 출력: Some(2)
    println!("{:?}", v1_iter.next()); // 출력: Some(3)
    println!("{:?}", v1_iter.next()); // 출력: None
    // iter()가 내놓는 값은 원소의 참조(&i32)입니다. 디버그 형식은 참조를 따라가 값을 보여 줍니다.
    // v1 자체는 그대로이므로 다시 쓸 수 있습니다.
    println!("v1은 그대로: {v1:?}"); // 출력: v1은 그대로: [1, 2, 3]

    // 문자열의 chars()(ch8), lines()(ch9·ch12), 범위(1..4)(ch3)도 모두 반복자입니다.
    let mut chars = "러스트".chars();
    println!("{:?} {:?}", chars.next(), chars.next()); // 출력: Some('러') Some('스')

    // 2. 반복자는 게으릅니다
    println!("\n== 2. 게으름 ==");
    let numbers = [1, 2, 3]; // 배열도 iter()로 반복자를 만들 수 있습니다.
    // map은 "원소마다 클로저를 적용하는 새 반복자"를 만들 뿐, 이 자리에서 클로저를 부르지 않습니다.
    let doubled = numbers.iter().map(|n| {
        println!("  map이 {n}을(를) 처리");
        n * 2
    });
    println!("map을 호출했습니다(아직 아무 처리도 하지 않음)");
    // for가 next를 부를 때마다 map이 원소 하나를 꺼내 클로저를 적용합니다.
    // 그래서 "map 처리 → for 받음"이 원소 하나씩 번갈아 나옵니다.
    for d in doubled {
        println!("for가 {d}을(를) 받음");
    }
    // 출력: map을 호출했습니다(아직 아무 처리도 하지 않음)
    // 출력:   map이 1을(를) 처리
    // 출력: for가 2을(를) 받음
    // 출력:   map이 2을(를) 처리
    // 출력: for가 4을(를) 받음
    // 출력:   map이 3을(를) 처리
    // 출력: for가 6을(를) 받음

    // 게으르기 때문에 끝이 없는 반복자도 쓸 수 있습니다. (1..)은 1부터 끝없이 세는 범위입니다.
    // take(5)가 다섯 개를 받으면 더 이상 next를 부르지 않으므로 반복이 끝납니다.
    let squares: Vec<u64> = (1..).map(|n| n * n).take(5).collect();
    println!("처음 다섯 제곱수: {squares:?}"); // 출력: 처음 다섯 제곱수: [1, 4, 9, 16, 25]

    // 3. for 문은 반복자 위에서 동작합니다
    println!("\n== 3. for와 next ==");
    let words = ["하나", "둘", "셋"];
    for word in words.iter() {
        println!("  for 문: {word}");
    }
    manual_for(&words); // 위 for 문과 같은 일을 next와 match로 합니다.
    // 출력:   for 문: 하나
    // 출력:   for 문: 둘
    // 출력:   for 문: 셋
    // 출력:   손으로 쓴 반복: 하나
    // 출력:   손으로 쓴 반복: 둘
    // 출력:   손으로 쓴 반복: 셋
    // for 문에서는 반복자 변수에 mut을 붙이지 않았습니다. for가 반복자의 소유권을 가져가
    // 안에서 가변으로 쓰기 때문입니다.
}
