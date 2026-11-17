// 바인딩이 값을 가져가는지(이동·복사), 빌리는지(참조)를 정하는 규칙을 보여 줍니다.
// - 참조를 참조가 아닌 패턴과 맞추면 바인딩이 자동으로 참조가 됩니다(기본 바인딩 모드).
// - & 패턴은 반대로 참조를 벗겨 안의 값을 꺼냅니다.
// - ref / ref mut는 값을 맞추면서 "이 바인딩만 참조로 만들어라"라고 직접 적는 키워드입니다.
//
// 실행: cargo run -p p0-ch18-patterns --example 05_binding_modes

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

// 값의 타입 이름을 문자열로 돌려줍니다. 바인딩이 어떤 타입이 됐는지 눈으로 확인하려고 만든 도구입니다.
// std::any::type_name은 String을 alloc::string::String처럼 전체 경로로 적으므로 경로 부분을 지웁니다.
fn type_of<T>(_: &T) -> String {
    std::any::type_name::<T>()
        .replace("alloc::string::", "")
        .replace("core::option::", "")
}

fn main() {
    println!("1. 값을 패턴과 맞추면 바인딩이 값을 가져갑니다");
    let user = User {
        name: String::from("민지"),
        age: 20,
    };
    let User { name, age } = user; // String은 name으로 이동, u32는 age로 복사됩니다.
    println!("    name: {} = {name}", type_of(&name));
    println!("    age: {} = {age}", type_of(&age));
    // 여기서 user를 통째로 쓰면 E0382(부분 이동된 값 사용)가 납니다. name 필드가 빠져나갔기 때문입니다.
    // 출력:
    //     name: String = 민지
    //     age: u32 = 20

    println!("2. 참조를 참조가 아닌 패턴과 맞추면 바인딩이 참조가 됩니다");
    let user = User {
        name: String::from("준호"),
        age: 31,
    };
    // 오른쪽은 &User인데 패턴 User { .. }에는 &가 없습니다. 컴파일러가 참조를 따라가 맞춘 뒤
    // 안쪽 바인딩을 모두 참조로 만듭니다. 아무것도 이동하지 않으므로 user를 계속 쓸 수 있습니다.
    let User { name, age } = &user;
    println!("    name: {}, age: {}", type_of(&name), type_of(&age));
    println!("    user는 그대로: {user:?}");
    // 출력:
    //     name: &String, age: &u32
    //     user는 그대로: User { name: "준호", age: 31 }

    println!("3. 가변 참조를 맞추면 바인딩은 가변 참조가 됩니다");
    let mut user = user;
    let User { name, age } = &mut user;
    println!("    name: {}, age: {}", type_of(&name), type_of(&age));
    name.push_str("(관리자)"); // &mut String이므로 메서드로 고칠 수 있습니다.
    *age += 1; // 정수 자체를 바꾸려면 *로 참조를 따라갑니다.
    println!("    고친 뒤: {user:?}");
    // 출력:
    //     name: &mut String, age: &mut u32
    //     고친 뒤: User { name: "준호(관리자)", age: 32 }

    println!("4. 안쪽 깊은 곳의 바인딩도, for와 if let에서도 같은 규칙입니다");
    let nickname = Some(String::from("러스타"));
    if let Some(s) = &nickname {
        println!("    if let의 s: {}", type_of(&s));
    }
    let scores = vec![(String::from("가은"), 90), (String::from("나래"), 85)];
    // &scores를 순회하면 원소는 &(String, i32)이고, (who, score) 패턴의 바인딩은 참조가 됩니다.
    for (who, score) in &scores {
        println!(
            "    {who}: {score}점 ({}, {})",
            type_of(&who),
            type_of(&score)
        );
    }
    // 출력:
    //     if let의 s: &String
    //     가은: 90점 (&String, &i32)
    //     나래: 85점 (&String, &i32)

    println!("5. & 패턴: 참조를 벗겨 안의 값을 꺼냅니다 (Copy 값만)");
    let numbers = [1, 2, 3];
    // &numbers를 순회하면 원소는 &i32입니다. &n 패턴이 &를 벗기므로 n은 i32(복사본)입니다.
    for &n in &numbers {
        println!("    n = {n} ({})", type_of(&n));
    }
    // 클로저 매개변수에서 자주 봅니다. iter()가 &i32를 내놓으므로 |&n|으로 받아 i32로 계산합니다.
    let doubled: Vec<i32> = numbers.iter().map(|&n| n * 2).collect();
    println!("    {doubled:?}");
    // String처럼 Copy가 아닌 값에 & 패턴을 쓰면 "빌린 것에서 값을 꺼내 옮기는" 셈이라 E0507이 납니다.
    // 출력:
    //     n = 1 (i32)
    //     n = 2 (i32)
    //     n = 3 (i32)
    //     [2, 4, 6]

    println!("6. ref: 값을 맞추면서 바인딩만 참조로 만듭니다");
    let nickname = Some(String::from("러스타"));
    // match nickname은 값을 맞추므로 보통은 String이 이동합니다. ref를 붙이면 빌리기만 합니다.
    // match &nickname { Some(s) => ... }와 같은 효과입니다.
    match nickname {
        Some(ref s) => println!("    s: {} = {s}", type_of(&s)),
        None => println!("    별명 없음"),
    }
    println!("    nickname은 그대로: {nickname:?}");
    // ref mut는 가변 참조로 묶습니다.
    let mut level = Some(1);
    if let Some(ref mut lv) = level {
        *lv += 1;
    }
    println!("    level: {level:?}");
    // ref가 지금도 쓸모 있는 곳: 한 패턴 안에서 어떤 필드는 빌리고, 어떤 필드는 복사하고 싶을 때.
    // &user로 맞추면 age까지 &u32가 되지만, ref를 쓰면 name만 빌리고 age는 u32로 복사합니다.
    let User { ref name, age } = user;
    println!("    name: {}, age: {}", type_of(&name), type_of(&age));
    println!("    user는 그대로: {user:?}");
    // 출력:
    //     s: &String = 러스타
    //     nickname은 그대로: Some("러스타")
    //     level: Some(2)
    //     name: &String, age: u32
    //     user는 그대로: User { name: "준호(관리자)", age: 32 }
}
