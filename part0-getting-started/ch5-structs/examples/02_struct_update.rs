// 이 예제가 보여 주는 것: 구조체 갱신 문법 `..기존_인스턴스`와, 그때 필드 단위로 일어나는 이동
// 실행: cargo run -p p0-ch5-structs --example 02_struct_update

struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

fn main() {
    println!("1. ..user1은 직접 적지 않은 나머지 필드를 user1에서 가져옵니다");
    let user1 = User {
        email: String::from("ferris@example.com"),
        username: String::from("ferris"),
        active: true,
        sign_in_count: 3,
    };
    // `as_ptr()`은 String이 가리키는 힙 데이터의 시작 주소입니다(ch4).
    // 주소 값은 실행마다 달라지므로 출력하지 않고, 나중에 비교하는 데만 씁니다.
    let heap_of_username = user1.username.as_ptr();

    // email만 새 값을 주고, active·username·sign_in_count는 user1에서 가져옵니다.
    // `..user1`은 반드시 마지막에 오고, 뒤에 쉼표를 붙이지 않습니다.
    let user2 = User {
        email: String::from("another@example.com"),
        ..user1
    };
    println!(
        "    user2: {} <{}>, 활성: {}, 로그인 {}회",
        user2.username, user2.email, user2.active, user2.sign_in_count
    );
    // 출력: user2: ferris <another@example.com>, 활성: true, 로그인 3회

    println!("2. 가져온 String 필드는 복사가 아니라 이동합니다");
    // `..user1`은 `username: user1.username`이라고 적은 것과 같습니다. 즉 대입이므로 String은 이동합니다.
    // 힙 주소가 같다는 것은 글자를 복사하지 않고 소유자만 user2로 바뀌었다는 뜻입니다.
    println!(
        "    user2.username이 원래 user1.username의 힙 데이터를 가리키는가? {}",
        user2.username.as_ptr() == heap_of_username
    );
    // 출력: user2.username이 원래 user1.username의 힙 데이터를 가리키는가? true

    println!("3. 이동하지 않은 필드는 user1에서 계속 쓸 수 있습니다");
    // email은 user2에 새 값을 줬으므로 user1에서 가져가지 않았습니다.
    println!("    user1.email = {}", user1.email);
    // active(bool)와 sign_in_count(u64)는 Copy 타입이라 이동이 아니라 복사되었습니다.
    println!(
        "    user1.active = {}, user1.sign_in_count = {}",
        user1.active, user1.sign_in_count
    );
    // 출력:
    //     user1.email = ferris@example.com
    //     user1.active = true, user1.sign_in_count = 3

    // 아래 두 줄은 컴파일되지 않습니다. 주석을 풀어 확인해 보세요.
    // println!("{}", user1.username); // E0382: borrow of moved value: `user1.username`
    // let whole = user1;              // E0382: use of partially moved value: `user1`
    // user1은 일부 필드가 빠져나간 "부분 이동" 상태라서, 남은 필드는 쓸 수 있지만 인스턴스 전체는 쓸 수 없습니다.

    println!("4. String 필드를 모두 새로 주면 원본은 온전히 남습니다");
    let base = User {
        email: String::from("base@example.com"),
        username: String::from("base"),
        active: false,
        sign_in_count: 10,
    };
    // 이번에 base에서 가져오는 것은 active와 sign_in_count뿐이고, 둘 다 Copy라서 복사됩니다.
    let user3 = User {
        username: String::from("crab"),
        email: String::from("crab@example.com"),
        ..base
    };
    println!(
        "    user3: {} <{}>, 활성: {}, 로그인 {}회",
        user3.username, user3.email, user3.active, user3.sign_in_count
    );
    let base_again = base; // base 전체를 옮길 수 있습니다. 아무 필드도 빠져나가지 않았기 때문입니다.
    println!(
        "    base:  {} <{}>, 활성: {}, 로그인 {}회",
        base_again.username, base_again.email, base_again.active, base_again.sign_in_count
    );
    // 출력:
    //     user3: crab <crab@example.com>, 활성: false, 로그인 10회
    //     base:  base <base@example.com>, 활성: false, 로그인 10회
}
