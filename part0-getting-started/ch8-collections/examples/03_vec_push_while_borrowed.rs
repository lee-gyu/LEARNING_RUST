// 벡터 요소의 참조를 쥔 채로 push할 수 없는 이유를 길이(len)와 용량(capacity)으로 확인하고,
// 그 제약을 피해 같은 일을 하는 방법들을 보여 줍니다.
//
// 실행: cargo run -p p0-ch8-collections --example 03_vec_push_while_borrowed

fn main() {
    println!("1. 벡터는 길이(len)와 별도로 용량(capacity)을 가집니다");
    // 길이는 실제로 들어 있는 요소 수, 용량은 다시 할당하지 않고 담을 수 있는 요소 수입니다.
    // 용량이 바뀌는 순간에만 줄을 출력해서, 언제 더 큰 공간을 새로 확보하는지 봅니다.
    let mut v: Vec<i32> = Vec::new();
    let mut last_capacity = v.capacity();
    println!("    시작: len = {}, capacity = {}", v.len(), last_capacity);
    for n in 1..=9 {
        v.push(n);
        if v.capacity() != last_capacity {
            println!(
                "    {n}번째 push: len = {}, capacity {} → {}",
                v.len(),
                last_capacity,
                v.capacity()
            );
            last_capacity = v.capacity();
        }
    }
    // 출력:
    //     시작: len = 0, capacity = 0
    //     1번째 push: len = 1, capacity 0 → 4
    //     5번째 push: len = 5, capacity 4 → 8
    //     9번째 push: len = 9, capacity 8 → 16
    //
    // 용량을 몇 배로 늘리는지는 표준 라이브러리가 보장하는 규칙이 아니라 현재 구현의 동작입니다.
    // 보장되는 것은 "길이가 용량과 같을 때 push하면 더 큰 공간을 새로 할당한다"는 사실입니다.

    println!("2. 용량이 꽉 찬 벡터에 push하면 요소가 새 공간으로 옮겨질 수 있습니다");
    let mut full = vec![1, 2, 3]; // vec!로 만든 벡터는 요소 수만큼만 용량을 잡습니다.
    println!(
        "    push 전: len = {}, capacity = {}",
        full.len(),
        full.capacity()
    );
    full.push(4);
    println!(
        "    push 후: len = {}, capacity = {}",
        full.len(),
        full.capacity()
    );
    // 출력:
    //     push 전: len = 3, capacity = 3
    //     push 후: len = 4, capacity = 6
    //
    // 용량 3이 꽉 찬 상태에서 push하면, 벡터는 더 큰 힙 공간을 새로 할당해 요소를 옮기고
    // 예전 공간을 해제할 수 있습니다. 만약 push 전에 `let first = &full[0];`을 만들어 두었다면,
    // first는 해제된 예전 공간을 가리키는 댕글링 참조가 됩니다.
    // 그래서 컴파일러는 "요소를 빌린 동안 벡터를 바꾸는 것" 자체를 막습니다(compile_fail/e0502_push_while_borrowed.rs).
    // 용량이 남아 있어도 마찬가지입니다. 실행 중의 용량은 컴파일러가 미리 알 수 없기 때문입니다.

    println!("3. 해결 방법 ①: 참조 대신 값을 복사해 둡니다");
    let mut numbers = vec![10, 20, 30];
    let first = numbers[0]; // i32는 Copy 타입이므로 값이 복사되고, 대여는 생기지 않습니다.
    numbers.push(40);
    println!("    first = {first}, numbers = {numbers:?}");
    // 출력: first = 10, numbers = [10, 20, 30, 40]

    println!("4. 해결 방법 ②: 참조를 다 쓴 뒤에 push합니다");
    let mut fruits = vec![String::from("사과"), String::from("배")];
    let first = &fruits[0];
    println!("    push 전 첫 요소: {first}"); // first의 마지막 사용. 여기서 대여가 끝납니다.
    fruits.push(String::from("감"));
    println!("    fruits = {fruits:?}");
    // 출력:
    //     push 전 첫 요소: 사과
    //     fruits = ["사과", "배", "감"]

    println!("5. 해결 방법 ③: 위치(인덱스)만 기억했다가 push 뒤에 다시 읽습니다");
    let target = 1; // 인덱스는 usize 값일 뿐 벡터를 빌리지 않습니다.
    fruits.push(String::from("귤"));
    println!("    {target}번 요소: {}", fruits[target]); // 출력: 1번 요소: 배

    println!("6. 해결 방법 ④: String처럼 Copy가 아닌 요소는 clone으로 독립된 사본을 만듭니다");
    let copy = fruits[0].clone(); // 힙 데이터까지 복제하므로 비용이 듭니다.
    fruits.push(String::from("포도"));
    println!("    copy = {copy}, 길이 = {}", fruits.len()); // 출력: copy = 사과, 길이 = 5
}
