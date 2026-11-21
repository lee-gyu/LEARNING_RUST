// 부록 B ch13 · Q13.1 — 전방 선언 없이 쓰기
//
// C와 C++는 위에서 아래로 읽으므로, 아래에 정의한 함수를 위에서 부르거나 서로를 부르는 함수를 쓰려면
// 먼저 선언(프로토타입)을 적고, 다른 파일에서 쓰려면 그 선언을 헤더에 모아 #include합니다.
// Rust에는 그런 선언이 없습니다. 크레이트 하나가 컴파일 단위이고, 그 안의 항목(함수, 타입, 모듈, use)은
// 적은 순서와 상관없이 서로를 가리킬 수 있습니다. 다른 파일의 항목도 mod 선언과 경로로 가리킬 뿐
// 선언을 따로 적지 않습니다(Part 0 ch7).
// 아래에 정의한 함수를 부르는 것 자체는 Part 0 ch3 예제 07에서 봤습니다. 여기서 더하는 것은 서로를 부르는 함수,
// 서로를 가리키는 타입, 아래에 적은 모듈과 use입니다.
//
// 실행: cargo run -p faq-ch13-headers-build-templates --example 01_no_forward_declarations

// 4번: use도 아래에 정의한 모듈을 가리킬 수 있습니다.
use geometry::area;

fn main() {
    println!("1. 아래에 정의한 함수를 위에서 부름");
    println!("  {}", greet("헤더")); // 출력:   안녕, 헤더

    println!("2. 서로를 부르는 함수");
    println!("  is_even(10) = {}, is_odd(7) = {}", is_even(10), is_odd(7)); // 출력:   is_even(10) = true, is_odd(7) = true

    println!("3. 아직 정의하지 않은 타입을 필드로 씀");
    let dept = Department {
        name: String::from("개발"),
        members: vec![Employee {
            name: String::from("민지"),
            team: None,
        }],
    };
    println!(
        "  {}: {}명, 첫 사람 {}",
        dept.name,
        dept.members.len(),
        dept.members[0].name
    ); // 출력:   개발: 1명, 첫 사람 민지
    println!("  첫 사람의 소속 있음? {}", dept.members[0].team.is_some()); // 출력:   첫 사람의 소속 있음? false

    println!("4. 아래에 정의한 모듈의 함수");
    println!("  넓이 = {}", area(3, 4)); // 출력:   넓이 = 12
}

fn greet(name: &str) -> String {
    format!("안녕, {name}")
}

// C라면 둘 중 하나를 먼저 선언해야 합니다.
fn is_even(n: u32) -> bool {
    if n == 0 { true } else { is_odd(n - 1) }
}

fn is_odd(n: u32) -> bool {
    if n == 0 { false } else { is_even(n - 1) }
}

// Department가 Employee를, Employee가 다시 Department를 가리킵니다.
// C++라면 한쪽을 `struct Department;`로 미리 선언해야 합니다. Rust는 선언이 필요 없습니다.
// Department는 Employee를 Vec(원소를 힙에 둠)으로 들고 있어 크기가 정해지므로 Box가 필요 없습니다.
// 간접 참조 없이 서로를 직접 품으면 크기가 무한해져 E0072입니다(재귀 타입과 Box는 Part 0 ch14).
struct Department {
    name: String,
    members: Vec<Employee>,
}

struct Employee {
    name: String,
    team: Option<Department>,
}

mod geometry {
    pub fn area(w: u32, h: u32) -> u32 {
        w * h
    }
}
