// 이 예제가 보여 주는 것: 직접 만든 타입에 Iterator를 구현하기(type Item과 next),
//                        next 하나만 구현하면 for 문과 모든 어댑터·소비 메서드를 쓸 수 있다는 것,
//                        impl Iterator<Item = ...>로 반복자를 돌려주는 함수
// 실행: cargo run -p p0-ch13-closures-iterators --example 07_custom_iterator

// 1부터 5까지 세는 반복자입니다. 반복자는 "지금 어디까지 왔는지"를 상태로 들고 있어야 하므로
// 그 상태(count)를 필드로 둡니다.
struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

impl Iterator for Counter {
    // 이 반복자가 내놓는 값의 타입입니다. 트레이트 안에 선언된 타입 자리(연관 타입)를
    // 구현하는 쪽에서 채웁니다. 자세한 내용은 ch19에서 다룹니다.
    type Item = u32;

    // next는 상태를 한 칸 진행하고 값을 Some으로 돌려주다가, 끝나면 None을 돌려줍니다.
    // 상태를 바꿔야 하므로 &mut self를 받습니다.
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

// 반복자를 돌려주는 함수입니다. (1..).map(...)의 실제 타입은 클로저 타입이 들어 있어 이름을 쓸 수
// 없으므로, ch10에서 본 반환 위치의 impl Trait로 "u32를 내놓는 어떤 반복자"라고만 적습니다.
// Item = u32는 Iterator의 연관 타입 Item이 u32라는 뜻입니다.
fn multiples_of(n: u32) -> impl Iterator<Item = u32> {
    // 반환된 반복자는 이 함수가 끝난 뒤에 쓰이므로, 클로저가 n을 빌리면 사라진 값을 가리키게 됩니다.
    // move로 n을 클로저 안에 복사해 넣습니다(빼면 E0373, 02 예제의 make_counter와 같은 이유).
    (1..).map(move |i| i * n)
}

fn main() {
    // 1. next를 직접 부르기
    println!("== 1. next ==");
    let mut counter = Counter::new();
    print!("next 여섯 번:");
    for _ in 0..6 {
        print!(" {:?}", counter.next());
    }
    println!();
    // 출력: next 여섯 번: Some(1) Some(2) Some(3) Some(4) Some(5) None

    // 2. for 문
    println!("\n== 2. for ==");
    print!("for로 받은 값:");
    for n in Counter::new() {
        print!(" {n}");
    }
    println!();
    // 출력: for로 받은 값: 1 2 3 4 5

    // 3. 어댑터와 소비 메서드는 Iterator의 기본 구현이므로 공짜로 따라옵니다
    println!("\n== 3. 어댑터 ==");
    // (1,2) (2,3) (3,4) (4,5)를 곱하면 2, 6, 12, 20이고, 그중 3의 배수는 6과 12입니다.
    let sum: u32 = Counter::new()
        .zip(Counter::new().skip(1))
        .map(|(a, b)| a * b)
        .filter(|x| x % 3 == 0)
        .sum();
    println!("합: {sum}"); // 출력: 합: 18

    let evens: Vec<u32> = Counter::new().filter(|n| n % 2 == 0).collect();
    println!("짝수만: {evens:?}"); // 출력: 짝수만: [2, 4]

    // 4. 반복자를 돌려주는 함수
    println!("\n== 4. impl Iterator ==");
    // multiples_of(3)은 끝이 없는 반복자이지만, 게으르므로 take(5)만큼만 계산합니다.
    let first_five: Vec<u32> = multiples_of(3).take(5).collect();
    println!("3의 배수 다섯 개: {first_five:?}"); // 출력: 3의 배수 다섯 개: [3, 6, 9, 12, 15]
}
