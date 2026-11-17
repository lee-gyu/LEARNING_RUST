// 함수 포인터 타입 fn과 Fn 트레이트들의 관계, 열거형·튜플 구조체 생성자를 함수로 쓰기,
// 클로저를 반환하는 두 가지 방법(impl Fn, Box<dyn Fn>)을 보여 줍니다.
//
// 실행: cargo run -p p0-ch19-advanced-features --example 07_fn_pointers_closures

fn add_one(x: i32) -> i32 {
    x + 1
}

fn double(x: i32) -> i32 {
    x * 2
}

// 1. fn(i32) -> i32는 트레이트가 아니라 "함수 포인터"라는 타입입니다.
fn do_twice(f: fn(i32) -> i32, arg: i32) -> i32 {
    f(arg) + f(arg)
}

// 2. Fn 트레이트로 받으면 함수와 클로저(캡처하는 것까지)를 모두 받습니다.
fn do_twice_generic(f: impl Fn(i32) -> i32, arg: i32) -> i32 {
    f(arg) + f(arg)
}

#[derive(Debug)]
enum Status {
    Value(u32),
    Stop,
}

#[derive(Debug)]
struct Meters(u32);

// 3. 클로저 반환: 클로저 타입에는 이름이 없으므로 impl Fn으로 "어떤 Fn"이라고 적습니다.
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

// 갈래마다 다른 클로저를 돌려줘야 하면 클로저마다 타입이 달라 impl Fn으로는 안 됩니다(E0308).
// Box<dyn Fn>(ch17의 트레이트 객체)로 감싸 하나의 타입으로 맞춥니다.
fn make_op(kind: &str, n: i32) -> Box<dyn Fn(i32) -> i32> {
    match kind {
        "더하기" => Box::new(move |x| x + n),
        "곱하기" => Box::new(move |x| x * n),
        _ => Box::new(|x| x),
    }
}

// 아무것도 캡처하지 않는 클로저는 함수 포인터로 바뀔 수 있어서 fn 타입으로도 반환할 수 있습니다.
fn pick(negate: bool) -> fn(i32) -> i32 {
    if negate { |x| -x } else { add_one }
}

fn main() {
    println!("1. 함수 포인터");
    println!("    do_twice(add_one, 5) = {}", do_twice(add_one, 5)); // 출력:     do_twice(add_one, 5) = 12
    // 함수 포인터는 값이므로 배열에 담을 수 있습니다. 캡처가 없는 클로저(|x| -x)도 fn으로 바뀝니다.
    // 타입이 길어 05 예제에서 본 타입 별칭(UnaryOp)으로 줄였습니다.
    type UnaryOp = fn(i32) -> i32;
    let table: [(&str, UnaryOp); 3] =
        [("add_one", add_one), ("double", double), ("negate", |x| -x)];
    for (name, f) in table {
        println!("    {name}(10) = {}", f(10));
    }
    // 출력:
    //     add_one(10) = 11
    //     double(10) = 20
    //     negate(10) = -10

    println!("2. fn과 Fn 트레이트");
    let bonus = 100;
    // bonus를 캡처한 클로저는 fn이 될 수 없으므로 do_twice에는 넘길 수 없습니다(E0308).
    // Fn 트레이트로 받는 함수에는 함수와 클로저를 모두 넘길 수 있습니다.
    println!(
        "    do_twice_generic(add_one, 5) = {}",
        do_twice_generic(add_one, 5)
    ); // 출력:     do_twice_generic(add_one, 5) = 12
    println!(
        "    do_twice_generic(|x| x + bonus, 5) = {}",
        do_twice_generic(|x| x + bonus, 5)
    ); // 출력:     do_twice_generic(|x| x + bonus, 5) = 210

    // map에 클로저 대신 함수 이름을 넘길 수 있습니다. 같은 결과입니다.
    let numbers = [1, 2, 3];
    let by_closure: Vec<String> = numbers.iter().map(|n| n.to_string()).collect();
    let by_function: Vec<String> = numbers.iter().map(ToString::to_string).collect();
    println!(
        "    {by_closure:?} == {by_function:?}? {}",
        by_closure == by_function
    ); // 출력:     ["1", "2", "3"] == ["1", "2", "3"]? true

    println!("3. 생성자도 함수입니다");
    // Status::Value는 u32를 받아 Status를 돌려주는 함수처럼 쓸 수 있습니다.
    let mut statuses: Vec<Status> = (1..=3).map(Status::Value).collect();
    statuses.push(Status::Stop);
    println!("    {statuses:?}"); // 출력:     [Value(1), Value(2), Value(3), Stop]
    let total: u32 = statuses
        .iter()
        .map(|s| match s {
            Status::Value(v) => *v,
            Status::Stop => 0,
        })
        .sum();
    println!("    Value들의 합 = {total}"); // 출력:     Value들의 합 = 6
    // 튜플 구조체 이름과 Some도 마찬가지입니다.
    let lengths: Vec<Meters> = [5, 10].into_iter().map(Meters).collect();
    let options: Vec<Option<u32>> = lengths.iter().map(|m| m.0).map(Some).collect();
    println!("    {lengths:?}, {options:?}"); // 출력:     [Meters(5), Meters(10)], [Some(5), Some(10)]

    println!("4. 클로저 반환하기");
    let add_ten = make_adder(10);
    println!("    make_adder(10)(5) = {}", add_ten(5)); // 출력:     make_adder(10)(5) = 15
    // 서로 다른 클로저를 한 배열(또는 Vec)에 담으려면 타입이 같아야 하므로 Box<dyn Fn>을 씁니다.
    let ops = [
        make_op("더하기", 3),
        make_op("곱하기", 3),
        make_op("그대로", 3),
    ];
    let results: Vec<i32> = ops.iter().map(|op| op(5)).collect();
    println!("    [+3, *3, 그대로] 적용 결과 = {results:?}"); // 출력:     [+3, *3, 그대로] 적용 결과 = [8, 15, 5]
    println!(
        "    pick(true)(5) = {}, pick(false)(5) = {}",
        pick(true)(5),
        pick(false)(5)
    ); // 출력:     pick(true)(5) = -5, pick(false)(5) = 6
}
