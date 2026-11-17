// 벡터는 한 가지 타입의 값만 담을 수 있습니다.
// 여러 종류의 값을 한 벡터에 담아야 할 때는 열거형(ch6)의 배리언트로 감싸 "같은 타입"으로 만듭니다.
//
// 실행: cargo run -p p0-ch8-collections --example 04_vec_enum

// 스프레드시트의 칸 하나에는 정수, 실수, 글자 중 하나가 들어갈 수 있다고 해 봅시다.
// 세 배리언트는 모두 같은 타입 `Cell`이므로 Vec<Cell>에 함께 담을 수 있습니다.
#[derive(Debug)]
enum Cell {
    Int(i32),
    Float(f64),
    Text(String),
}

fn main() {
    println!("1. 서로 다른 종류의 값을 열거형으로 감싸 한 벡터에 담습니다");
    let row = vec![
        Cell::Text(String::from("사과")),
        Cell::Int(3),
        Cell::Float(1200.5),
    ];
    println!("    row = {row:?}");
    // 출력: row = [Text("사과"), Int(3), Float(1200.5)]

    println!("2. 꺼낼 때는 match로 배리언트마다 처리합니다");
    for cell in &row {
        // cell의 타입은 &Cell입니다. 참조에 match를 써도 패턴은 그대로 쓸 수 있고,
        // 이때 n, x, s는 배리언트 안 값의 참조(&i32, &f64, &String)가 됩니다.
        match cell {
            Cell::Int(n) => println!("    정수: {n}"),
            Cell::Float(x) => println!("    실수: {x}"),
            Cell::Text(s) => println!("    글자: {s} ({}바이트)", s.len()),
        }
    }
    // 출력:
    //     글자: 사과 (6바이트)
    //     정수: 3
    //     실수: 1200.5

    println!("3. 숫자 칸만 골라 합계를 구합니다");
    // match는 모든 배리언트를 다뤄야 하므로, 나중에 배리언트를 추가하면 이 코드도 고치라고 컴파일러가 알려 줍니다.
    let mut sum = 0.0;
    for cell in &row {
        match cell {
            Cell::Int(n) => sum += *n as f64, // *n으로 참조를 따라가 i32 값을 꺼낸 뒤 f64로 바꿉니다.
            Cell::Float(x) => sum += *x,
            Cell::Text(_) => {} // 글자 칸은 건너뜁니다.
        }
    }
    println!("    숫자 칸의 합계 = {sum}"); // 출력: 숫자 칸의 합계 = 1203.5
}
