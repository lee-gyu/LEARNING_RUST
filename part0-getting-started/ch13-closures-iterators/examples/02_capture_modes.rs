// 이 예제가 보여 주는 것: 클로저가 환경을 캡처하는 세 가지 방식(불변 대여, 가변 대여, 소유권 이동)과
//                        각 방식이 대여 규칙과 맞물리는 모습, move 키워드, move로 클로저를 함수 밖으로 돌려주기
// 실행: cargo run -p p0-ch13-closures-iterators --example 02_capture_modes
//
// 캡처 방식은 우리가 고르지 않습니다. 컴파일러가 클로저 본문이 그 변수로 "무엇을 하는지" 보고,
// 그 일을 할 수 있는 가장 약한 방식을 고릅니다: 읽기만 하면 불변 대여, 바꾸면 가변 대여, 가져가면 이동.

// 클로저를 돌려주는 함수입니다. 클로저의 타입에는 이름이 없으므로 ch10에서 본 반환 위치의 impl Trait로
// "FnMut() -> u32를 구현한 어떤 타입"이라고만 적습니다(FnMut은 03 예제에서 다룹니다).
fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    // count는 이 함수의 지역 변수라서 함수가 끝나면 사라집니다. 클로저가 count를 빌리기만 하면
    // 사라진 값을 가리키게 되므로, move로 count를 클로저 안으로 옮깁니다.
    // move를 빼면 error[E0373]: closure may outlive the current function 입니다.
    move || {
        count += 1;
        count
    }
}

fn main() {
    // 1. 불변 대여: 본문이 list를 읽기만 합니다
    println!("== 1. 불변 대여 ==");
    let list = vec![1, 2, 3];
    println!("정의 전: {list:?}");
    let only_borrows = || println!("클로저 안: {list:?}"); // 클로저가 &list를 들고 있습니다.
    // 불변 참조는 여러 개 있어도 되므로, 클로저가 살아 있는 동안에도 list를 읽을 수 있습니다.
    println!("호출 전: {list:?}");
    only_borrows();
    println!("호출 후: {list:?}");
    // 출력: 정의 전: [1, 2, 3]
    // 출력: 호출 전: [1, 2, 3]
    // 출력: 클로저 안: [1, 2, 3]
    // 출력: 호출 후: [1, 2, 3]

    // 2. 가변 대여: 본문이 list를 바꿉니다
    println!("\n== 2. 가변 대여 ==");
    let mut list = vec![1, 2, 3];
    // 클로저가 &mut list를 들고 있습니다. 호출할 때마다 클로저 안의 상태(빌린 list)를 바꾸므로
    // 클로저 변수에도 mut이 필요합니다(빼면 E0596).
    let mut borrows_mutably = || list.push(7);
    // 여기서 println!("{list:?}")로 list를 읽으면 E0502입니다. 가변 대여가 살아 있는 동안에는
    // 다른 대여가 있을 수 없기 때문입니다(compile_fail/e0502_mutable_capture_alive.rs).
    borrows_mutably();
    borrows_mutably();
    // 클로저를 마지막으로 쓴 뒤에는 가변 대여가 끝났으므로 다시 읽을 수 있습니다(ch4의 NLL과 같은 원리).
    println!("호출 후: {list:?}"); // 출력: 호출 후: [1, 2, 3, 7, 7]

    // 3. 소유권 이동: 본문이 name을 클로저 밖으로 내보냅니다
    println!("\n== 3. 소유권 이동 ==");
    let name = String::from("페리스");
    // 본문이 name을 그대로 돌려주므로, 클로저는 name을 빌리는 것으로는 부족하고 소유해야 합니다.
    // 그래서 정의하는 순간 name이 클로저 안으로 이동합니다. 이 뒤에 name을 쓰면 E0382입니다.
    let give_away = || name;
    let owner = give_away(); // 클로저 안의 String이 owner로 옮겨 갑니다.
    println!("새 소유자: {owner}"); // 출력: 새 소유자: 페리스
    // 클로저 안에는 이제 내보낼 name이 없으므로 give_away()를 한 번 더 부를 수 없습니다(03 예제의 FnOnce).

    // 4. move 키워드: 본문은 읽기만 해도 소유권을 가져가게 강제합니다
    println!("\n== 4. move ==");
    let list = vec![1, 2, 3];
    let print_list = move || println!("move 클로저 안: {list:?}");
    print_list();
    print_list(); // move는 "어떻게 들여오는가"만 바꿉니다. 본문은 읽기만 하므로 여러 번 불러도 됩니다.
    // 출력: move 클로저 안: [1, 2, 3]
    // 출력: move 클로저 안: [1, 2, 3]
    // 여기서 list를 쓰면 E0382: borrow of moved value 입니다(compile_fail/e0382_use_after_move_closure.rs).

    // 5. Copy 타입을 move하면 복사본이 들어갑니다
    println!("\n== 5. move와 Copy 타입 ==");
    let mut count = 0;
    let mut increment = move || {
        count += 1; // 클로저 안의 복사본을 바꿉니다.
        count
    };
    println!("increment() = {}", increment()); // 출력: increment() = 1
    println!("increment() = {}", increment()); // 출력: increment() = 2
    // i32는 Copy라서 move해도 원본을 계속 쓸 수 있지만, 클로저 안의 count와는 별개의 값입니다.
    println!("바깥의 count = {count}"); // 출력: 바깥의 count = 0

    // 6. 클로저를 함수 밖으로 돌려주기
    println!("\n== 6. 클로저 반환 ==");
    let mut counter_a = make_counter();
    let mut counter_b = make_counter();
    // 클로저마다 자기 count를 소유하므로 두 카운터는 서로 영향을 주지 않습니다.
    println!("a: {}", counter_a()); // 출력: a: 1
    println!("a: {}", counter_a()); // 출력: a: 2
    println!("b: {}", counter_b()); // 출력: b: 1
}
