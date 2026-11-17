// 벡터(`Vec<T>`)를 만들고, 요소를 추가하고, 읽는 기본 방법을 보여 줍니다.
// 특히 인덱싱(`v[i]`)과 `get(i)`이 범위 밖 인덱스에서 어떻게 다르게 동작하는지 비교합니다.
//
// 실행: cargo run -p p0-ch8-collections --example 01_vec_basics

fn main() {
    println!("1. 아무것도 넣지 않을 빈 벡터에는 요소 타입을 적어 줍니다");
    // 넣는 값이 없으면 컴파일러가 요소 타입을 추론할 근거가 없습니다(적지 않으면 E0282).
    // `Vec<i32>`는 "i32 값들을 담는 벡터"라는 뜻입니다. <> 안의 타입을 바꿔 쓰는 원리(제네릭)는 ch10에서 배웁니다.
    let empty: Vec<i32> = Vec::new();
    println!("    empty = {empty:?}, 길이 = {}", empty.len()); // 출력: empty = [], 길이 = 0

    println!("2. vec! 매크로는 초깃값으로 벡터를 만들고, 타입은 값에서 추론합니다");
    let numbers = vec![10, 20, 30];
    // 정수 리터럴의 기본 타입이 i32이므로 numbers는 Vec<i32>가 됩니다.
    let zeros = vec![0; 4]; // [값; 개수] 형태: 0을 4개
    println!("    numbers = {numbers:?}, zeros = {zeros:?}");
    // 출력: numbers = [10, 20, 30], zeros = [0, 0, 0, 0]

    println!("3. push로 끝에 추가하려면 벡터가 mut여야 합니다");
    // 여기서는 타입을 적지 않았지만, 아래 push("러스트")를 보고 컴파일러가 Vec<&str>로 추론합니다.
    let mut words = Vec::new();
    println!("    push 전: 길이 = {}", words.len()); // 출력: push 전: 길이 = 0
    words.push("러스트");
    words.push("벡터");
    words.push("연습");
    println!("    push 후: words = {words:?}, 길이 = {}", words.len());
    // 출력: push 후: words = ["러스트", "벡터", "연습"], 길이 = 3

    println!("4. 인덱싱 v[i]는 요소를 바로 읽습니다 (0부터 셉니다)");
    let third: &i32 = &numbers[2]; // &를 붙이면 요소를 빌립니다(참조).
    let first: i32 = numbers[0]; // i32는 Copy 타입이라 &없이 쓰면 값이 복사됩니다.
    println!("    third = {third}, first = {first}"); // 출력: third = 30, first = 10

    println!("5. get(i)는 Option<&T>를 돌려줍니다");
    // 범위 안이면 Some(요소의 참조), 범위 밖이면 None입니다.
    match numbers.get(1) {
        Some(n) => println!("    get(1) = Some({n})"), // 출력: get(1) = Some(20)
        None => println!("    get(1) = None"),
    }
    match numbers.get(99) {
        Some(n) => println!("    get(99) = Some({n})"),
        None => println!("    get(99) = None (99번 요소는 없습니다)"), // 출력: get(99) = None (99번 요소는 없습니다)
    }

    println!("6. 범위 밖일 수 있는 인덱스는 get으로 안전하게 처리합니다");
    // 사용자 입력처럼 실행해 봐야 아는 인덱스라고 생각해 보세요.
    // `numbers[index]`로 읽었다면 index가 3 이상일 때 프로그램이 패닉으로 멈춥니다.
    //     index out of bounds: the len is 3 but the index is 5
    // `get`을 쓰면 None 갈래에서 "없음"을 처리하고 계속 진행할 수 있습니다.
    for index in [0, 2, 5] {
        if let Some(n) = numbers.get(index) {
            println!("    {index}번 요소: {n}");
        } else {
            println!("    {index}번 요소: 없음 (길이는 {})", numbers.len());
        }
    }
    // 출력:
    //     0번 요소: 10
    //     2번 요소: 30
    //     5번 요소: 없음 (길이는 3)

    // 아래 줄의 주석을 풀면 컴파일은 되지만, 실행하면 이 줄에서 위의 메시지로 패닉합니다(종료 코드 101).
    // 배열과 달리 벡터의 길이는 실행 중에 바뀔 수 있어서, 상수 인덱스라도 컴파일러가 미리 막지 않습니다.
    // println!("{}", numbers[5]);
}
