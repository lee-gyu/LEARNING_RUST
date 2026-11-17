// match로 배리언트마다 다른 코드를 실행하고, 패턴으로 배리언트 안의 값을 꺼내는 방법(바인딩)과
// 열거형에 impl로 연관 함수와 메서드를 정의하는 방법을 보여 줍니다.
//
// 실행: cargo run -p p0-ch6-enums-pattern-matching --example 02_match_binding

#[derive(Debug)]
enum Drink {
    Water,                             // 데이터 없음
    Coffee { shots: u32, iced: bool }, // 구조체형: 샷 수, 아이스 여부
    Juice(u32),                        // 튜플형: 용량(ml)
}

// 음료 값을 받아(소유권을 가져와) 가격을 계산합니다.
fn price(drink: Drink) -> u32 {
    // match는 drink를 위에서부터 차례로 각 갈래의 패턴과 맞춰 보고, 처음 맞는 갈래만 실행합니다.
    // 실행된 갈래의 값이 match 전체의 값이 되고, 함수의 마지막 표현식이므로 그대로 반환됩니다.
    match drink {
        Drink::Water => 0,
        // 패턴 안의 shots, iced는 여기서 새로 만드는 변수입니다.
        // drink가 Coffee이면 그 안에 든 값이 이 이름들에 묶입니다. 이것을 바인딩이라고 합니다.
        // 갈래가 여러 줄이면 중괄호 블록으로 감싸고, 블록의 마지막 표현식이 갈래의 값이 됩니다.
        Drink::Coffee { shots, iced } => {
            let base = 2000 + 500 * shots;
            if iced { base + 500 } else { base }
        }
        // 튜플형은 괄호 안의 위치로 값을 꺼냅니다. 변수 이름은 자유롭게 지을 수 있습니다.
        Drink::Juice(ml) => ml * 10,
    }
}

// 구조체처럼 열거형에도 impl 블록으로 연관 함수와 메서드를 붙일 수 있습니다(ch5).
impl Drink {
    // 연관 함수: self 없이 Drink::americano()로 호출합니다. impl 안에서 Self는 Drink를 뜻합니다.
    fn americano() -> Self {
        Self::Coffee {
            shots: 2,
            iced: false,
        }
    }

    // 메서드: &self로 빌려서 읽기만 합니다. 열거형의 메서드는 대개 match self로 배리언트를 나눕니다.
    fn describe(&self) -> String {
        // self의 타입은 &Drink(참조)입니다. 참조를 match하면 바인딩된 변수도 참조가 되어
        // shots는 &u32, iced는 &bool입니다. 이 규칙은 05 예제에서 자세히 다룹니다.
        match self {
            Self::Water => String::from("물"),
            Self::Coffee { shots, iced } => {
                // if의 조건은 bool이어야 하므로 *로 참조를 따라가 값을 읽습니다(ch4의 역참조).
                let temperature = if *iced { "아이스" } else { "따뜻한" };
                // format!은 println!과 같은 서식 문자열을 받아, 출력하는 대신 String으로 돌려줍니다.
                // {}로 출력할 때는 참조도 값처럼 그대로 쓸 수 있습니다.
                format!("{temperature} 커피 {shots}샷")
            }
            Self::Juice(ml) => format!("주스 {ml}ml"),
        }
    }
}

fn main() {
    println!("1. 배리언트마다 다른 갈래가 실행되고, 그 갈래의 값이 match의 값이 됩니다");
    println!("    물: {}원", price(Drink::Water));
    println!("    주스 350ml: {}원", price(Drink::Juice(350)));
    // 출력:
    //     물: 0원
    //     주스 350ml: 3500원

    println!("2. 패턴에 쓴 이름에 배리언트 안의 값이 묶입니다");
    println!(
        "    따뜻한 커피 2샷: {}원",
        price(Drink::Coffee {
            shots: 2,
            iced: false
        })
    );
    println!(
        "    아이스 커피 3샷: {}원",
        price(Drink::Coffee {
            shots: 3,
            iced: true
        })
    );
    // 출력:
    //     따뜻한 커피 2샷: 3000원
    //     아이스 커피 3샷: 4000원

    println!("3. 열거형에도 연관 함수와 메서드를 정의할 수 있습니다");
    let order = Drink::americano();
    println!("    {order:?} → {}", order.describe());
    // 출력:
    //     Coffee { shots: 2, iced: false } → 따뜻한 커피 2샷

    println!("4. 메뉴판: 메서드로 이름을 만들고(빌림), 함수로 가격을 계산합니다(이동)");
    let menu = [
        Drink::Water,
        Drink::Coffee {
            shots: 1,
            iced: true,
        },
        Drink::Juice(500),
    ];
    for drink in menu {
        let name = drink.describe(); // &self 메서드: drink를 빌려 쓰기만 합니다.
        let won = price(drink); // drink의 소유권이 price로 이동합니다. 이 뒤로 drink는 쓸 수 없습니다.
        println!("    {name}: {won}원");
    }
    // 출력:
    //     물: 0원
    //     아이스 커피 1샷: 3000원
    //     주스 500ml: 5000원
}
