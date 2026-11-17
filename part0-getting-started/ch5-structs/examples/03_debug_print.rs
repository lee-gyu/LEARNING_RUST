// 이 예제가 보여 주는 것: `#[derive(Debug)]`로 구조체를 디버그 형식 `{:?}`, `{:#?}`, `dbg!`로 출력하는 방법
// 실행: cargo run -p p0-ch5-structs --example 03_debug_print
//   `dbg!`는 표준 에러로 출력합니다. 표준 출력만 보려면 끝에 `2>/dev/null`을 붙여 실행해 보세요.

// `#[derive(Debug)]`는 속성(attribute)입니다. 바로 아래 구조체에 대해
// "디버그 형식으로 출력하는 코드"를 컴파일러가 자동으로 만들어 줍니다.
// 이 줄을 지우면 `{:?}`를 쓰는 곳마다 E0277 에러가 납니다(compile_fail 참고).
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

// 필드의 타입도 모두 Debug를 지원해야 derive할 수 있습니다.
// String, u32는 표준 라이브러리가, Point는 위의 derive가 지원을 만들어 줍니다.
#[derive(Debug)]
struct Player {
    name: String,
    position: Point,
    hp: u32,
}

fn main() {
    let player = Player {
        name: String::from("페리스"),
        position: Point { x: 3, y: -2 },
        hp: 100,
    };

    println!("1. {{}}(기본 형식)에는 구조체를 바로 넣을 수 없어 필드를 하나씩 꺼내 씁니다");
    // 구조체를 "사람에게 보여 줄 모양"은 프로그램마다 다르므로 Rust가 대신 정해 주지 않습니다.
    // `println!("{player}")`는 E0277입니다. 사용자에게 보여 줄 문장은 이렇게 직접 만듭니다.
    println!(
        "    {}의 위치는 ({}, {}), 체력은 {}입니다",
        player.name, player.position.x, player.position.y, player.hp
    );
    // 출력: 페리스의 위치는 (3, -2), 체력은 100입니다

    println!("2. {{:?}}는 구조체 이름과 필드를 한 줄로 보여 줍니다");
    // 문자열 필드는 따옴표로 감싸고, 안쪽 구조체도 같은 모양으로 펼쳐 보여 줍니다.
    println!("    {player:?}");
    // 출력: Player { name: "페리스", position: Point { x: 3, y: -2 }, hp: 100 }

    println!("3. {{:#?}}는 필드마다 줄을 바꿔 들여 씁니다");
    // 필드가 많거나 구조체가 겹겹이 들어 있을 때 읽기 쉽습니다.
    println!("{player:#?}");
    // 출력:
    // Player {
    //     name: "페리스",
    //     position: Point {
    //         x: 3,
    //         y: -2,
    //     },
    //     hp: 100,
    // }

    println!("4. dbg!는 파일 위치, 식, 값을 표준 에러로 출력하고 값을 그대로 돌려줍니다");
    let bonus = 20;
    // dbg!(식)은 식의 값을 돌려주므로 계산 중간에 끼워 넣어도 결과가 바뀌지 않습니다.
    let boosted = Player {
        name: String::from("강해진 페리스"),
        position: Point { x: 0, y: 0 },
        hp: dbg!(player.hp + bonus),
    };
    // dbg!는 인자의 소유권을 가져갔다가 돌려줍니다. 인스턴스를 계속 쓰려면 참조를 넘깁니다.
    // `dbg!(boosted);`처럼 값을 넘기면 boosted가 dbg! 안으로 이동해 아래 줄에서 쓸 수 없습니다.
    dbg!(&boosted);
    // 출력(표준 에러, 위치는 이 파일의 줄:열):
    // [part0-getting-started/ch5-structs/examples/03_debug_print.rs:63:13] player.hp + bonus = 120
    // [part0-getting-started/ch5-structs/examples/03_debug_print.rs:67:5] &boosted = Player {
    //     name: "강해진 페리스",
    //     position: Point {
    //         x: 0,
    //         y: 0,
    //     },
    //     hp: 120,
    // }

    println!("    {}의 체력: {}", boosted.name, boosted.hp);
    // 출력: 강해진 페리스의 체력: 120
}
