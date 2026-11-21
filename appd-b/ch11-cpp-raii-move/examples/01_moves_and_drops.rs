// Q11.1 · Q11.2 · Q11.3 · Q11.4 — 이동은 원본을 남기지 않고, 소멸은 값마다 한 번
//
// compare/01_moved_from_object.cpp와 같은 일을 Rust로 합니다. C++에서는 std::move 뒤에도 원본 객체가 살아 있어서
// 소멸자가 객체 수만큼(원본 포함) 불렸습니다. Rust의 이동은 값을 그대로 옮기고 원본 변수를 쓸 수 없게 표시할
// 뿐이라, 이동 생성자에 해당하는 코드가 없고 소멸(Drop)도 값마다 한 번입니다.
//
//   1. let b = a; — "이동" 출력이 없고, 소멸은 한 번
//   2. C++처럼 빈 값을 남기는 이동이 필요하면 std::mem::take: 남은 빈 값도 따로 소멸
//   3. 복사는 명시적으로: clone하면 소멸도 둘. Copy 타입(i32 등)만 대입이 복사
//
// 이동하면 해제가 한 번, clone하면 둘이라는 것은 ch9 예제 01(C의 free 관점)에서, mem::take·replace·swap 자체는
// Part 1 ch7 예제 03, Drop과 drop 순서는 Part 0 ch14 2.6절에서 봤습니다. 여기서 더하는 것은 C++의 이동 생성자·
// 이동된 객체와 소멸 횟수를 나란히 놓는 것과, mem::take가 남긴 빈 값도 따로 소멸한다는 점입니다.
//
// 실행: cargo run -p faq-ch11-cpp-raii-move --example 01_moves_and_drops

#[derive(Default)]
struct Buffer {
    name: String,
}

impl Buffer {
    fn new(name: &str) -> Buffer {
        println!("    생성: {name}");
        Buffer {
            name: name.to_string(),
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let shown = if self.name.is_empty() {
            "\"\""
        } else {
            &self.name
        };
        println!("    소멸: {shown}");
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Buffer {
        println!("    복제: {}", self.name);
        Buffer {
            name: self.name.clone(),
        }
    }
}

fn main() {
    println!("1. let b = a;");
    {
        let a = Buffer::new("요청 버퍼");
        let b = a; // 이동. 원본 a는 이제 쓸 수 없고, 소멸자도 a에 대해서는 돌지 않습니다.
        println!("    b.name = {}", b.name);
    }
    // 출력:     생성: 요청 버퍼
    // 출력:     b.name = 요청 버퍼
    // 출력:     소멸: 요청 버퍼

    println!("2. 빈 값을 남기는 이동: mem::take");
    {
        let mut a = Buffer::new("응답 버퍼");
        let b = std::mem::take(&mut a); // a 자리에는 Buffer::default()가 남습니다.
        println!("    a.name = {:?}, b.name = {}", a.name, b.name);
    }
    // 출력:     생성: 응답 버퍼
    // 출력:     a.name = "", b.name = 응답 버퍼
    // 출력:     소멸: 응답 버퍼
    // 출력:     소멸: ""
    // (지역 변수는 선언의 역순으로 버려지므로 b가 먼저, a가 나중입니다.)

    println!("3. 복사는 명시적으로");
    {
        let a = Buffer::new("설정");
        let b = a.clone();
        println!("    a.name = {}, b.name = {}", a.name, b.name);
        let x = 7;
        let y = x; // i32는 Copy라서 대입이 복사이고, x도 계속 쓸 수 있습니다.
        println!("    x = {x}, y = {y}");
    }
    // 출력:     생성: 설정
    // 출력:     복제: 설정
    // 출력:     a.name = 설정, b.name = 설정
    // 출력:     x = 7, y = 7
    // 출력:     소멸: 설정
    // 출력:     소멸: 설정
}
