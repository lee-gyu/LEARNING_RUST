// 이 예제가 보여 주는 것: 구조체를 정의하고, 인스턴스를 만들고, 필드를 읽고 바꾸는 방법
//   필드 초기화 축약과 "가변성은 인스턴스 전체에 붙는다"는 규칙도 함께 확인합니다.
// 실행: cargo run -p p0-ch5-structs --example 01_define_and_instantiate

// 구조체 정의: 서로 관련된 값 여러 개를 하나로 묶고, 묶음 전체와 각 값(필드)에 이름을 붙입니다.
// 튜플과 달리 필드마다 이름이 있으므로 "몇 번째 값이 무엇이었는지" 기억할 필요가 없습니다.
// 구조체 이름은 UpperCamelCase, 필드 이름은 snake_case로 짓는 것이 관례입니다.
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

// 필드 이름을 두 번씩 쓰는 방식입니다. `email: email`은 "email 필드에 email 매개변수를 넣는다"는 뜻입니다.
// Clippy는 이렇게 같은 이름을 반복하면 축약하라고 경고합니다(redundant_field_names).
// 여기서는 축약 전의 모습을 보여 주려고 일부러 썼으므로 경고를 예상된 것으로 표시합니다.
#[expect(
    clippy::redundant_field_names,
    reason = "필드 초기화 축약을 쓰기 전의 모습을 비교하려고 일부러 반복합니다"
)]
fn build_user_verbose(email: String, username: String) -> User {
    // 함수 본문의 마지막 표현식이 구조체이므로, 만든 인스턴스가 그대로 반환됩니다.
    User {
        active: true,
        username: username,
        email: email,
        sign_in_count: 1,
    }
}

// 필드 초기화 축약(field init shorthand): 변수 이름과 필드 이름이 같으면 한 번만 씁니다.
// 위 함수와 완전히 같은 동작입니다.
fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username,
        email,
        sign_in_count: 1,
    }
}

fn main() {
    println!("1. 인스턴스를 만들고 점(.)으로 필드를 읽습니다");
    // 필드를 적는 순서는 정의와 달라도 됩니다. 이름으로 짝을 맞추기 때문입니다.
    // 대신 모든 필드에 값을 줘야 합니다. 하나라도 빠뜨리면 컴파일 에러(E0063)가 납니다.
    let user1 = User {
        email: String::from("ferris@example.com"),
        username: String::from("ferris"),
        active: true,
        sign_in_count: 1,
    };
    println!(
        "    {} <{}>, 활성: {}, 로그인 {}회",
        user1.username, user1.email, user1.active, user1.sign_in_count
    );
    // 출력: ferris <ferris@example.com>, 활성: true, 로그인 1회

    println!("2. 필드를 바꾸려면 인스턴스 전체를 mut로 선언합니다");
    // Rust에는 "이 필드만 가변" 같은 표시가 없습니다. 가변성은 필드가 아니라 변수(바인딩)에 붙습니다.
    // `let user2`로 선언했다면 아래 두 줄은 E0594로 거부됩니다(compile_fail 참고).
    let mut user2 = User {
        email: String::from("old@example.com"),
        username: String::from("crab"),
        active: true,
        sign_in_count: 1,
    };
    user2.email = String::from("new@example.com"); // 이전 String은 소유자를 잃어 이 자리에서 drop됩니다.
    user2.sign_in_count += 1;
    println!(
        "    {} <{}>, 로그인 {}회",
        user2.username, user2.email, user2.sign_in_count
    );
    // 출력: crab <new@example.com>, 로그인 2회

    println!("3. 불변 인스턴스도 mut 변수로 옮기면 바꿀 수 있습니다");
    // 값 자체에 "불변"이 새겨진 것이 아니라, user1이라는 변수가 바꾸지 않겠다고 약속했을 뿐입니다.
    // 소유권을 mut 변수로 옮기면(이동) 새 소유자는 값을 바꿀 수 있습니다. 이후 user1은 쓸 수 없습니다.
    let mut moved = user1;
    moved.active = false;
    println!("    {}, 활성: {}", moved.username, moved.active);
    // 출력: ferris, 활성: false

    println!("4. 함수가 구조체를 만들어 돌려줄 수 있습니다");
    let user3 = build_user_verbose(String::from("verbose@example.com"), String::from("verbose"));
    println!("    {} <{}>", user3.username, user3.email);
    // 출력: verbose <verbose@example.com>

    println!("5. 필드 초기화 축약을 써도 결과는 같습니다");
    let user4 = build_user(String::from("short@example.com"), String::from("short"));
    println!(
        "    {} <{}>, 활성: {}, 로그인 {}회",
        user4.username, user4.email, user4.active, user4.sign_in_count
    );
    // 출력: short <short@example.com>, 활성: true, 로그인 1회

    // 축약은 함수 매개변수에만 쓰는 것이 아닙니다. 같은 이름의 지역 변수가 있으면 어디서든 쓸 수 있습니다.
    let username = String::from("local");
    let email = String::from("local@example.com");
    let user5 = User {
        username,
        email,
        active: false,
        sign_in_count: 0,
    };
    println!(
        "    {} <{}>, 활성: {}, 로그인 {}회",
        user5.username, user5.email, user5.active, user5.sign_in_count
    );
    // 출력: local <local@example.com>, 활성: false, 로그인 0회
}
