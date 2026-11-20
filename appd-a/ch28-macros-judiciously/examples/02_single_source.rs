// 지침 2-①: 서로 맞아야 하는 코드가 여러 곳에 흩어지면, 정보를 한 곳에 두고 매크로로 모두 만들어 냅니다.
//
// 요금제 열거형에 "모든 요금제 목록", "코드 문자열", "코드에서 요금제 찾기"가 따라다닌다고 합시다.
// 손으로 쓰면 배리언트를 하나 더할 때 네 곳을 고쳐야 하는데, 컴파일러가 챙겨 주는 곳은 그중 `_` 없는 match뿐입니다.
// 표 하나를 매크로에 넘기면 네 곳이 늘 같은 목록에서 만들어집니다.
//
// 실행: cargo run -p er-ch28-macros-judiciously --example 02_single_source

// 비권장: 손으로 맞춘 네 곳. 처음에는 Free, Basic, Pro였고 나중에 Team을 더했습니다.
mod v1 {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Plan {
        Free,
        Basic,
        Pro,
        Team, // 새로 더한 배리언트
    }

    impl Plan {
        // 배열 길이까지 손으로 적었으므로 Team을 빠뜨려도 컴파일 에러가 나지 않습니다.
        pub const ALL: [Plan; 3] = [Plan::Free, Plan::Basic, Plan::Pro];

        // `_` 갈래가 없는 match라서 Team을 더하자마자 컴파일러가 이 함수를 고치게 했습니다(E0004).
        pub fn code(self) -> &'static str {
            match self {
                Plan::Free => "free",
                Plan::Basic => "basic",
                Plan::Pro => "pro",
                Plan::Team => "team",
            }
        }

        // 문자열을 받는 match에는 `_` 갈래가 꼭 있어야 하므로, "team"을 빠뜨려도 아무도 알려 주지 않습니다.
        pub fn from_code(code: &str) -> Option<Plan> {
            match code {
                "free" => Some(Plan::Free),
                "basic" => Some(Plan::Basic),
                "pro" => Some(Plan::Pro),
                _ => None,
            }
        }
    }
}

// 권장: 요금제마다 한 줄씩, 모든 정보를 한 표에 둡니다. 표의 모양은 구조체 리터럴처럼 보이게 골랐습니다.
mod v2 {
    macro_rules! plans {
        ($( $variant:ident { code: $code:literal, label: $label:literal, price: $price:literal } ),+ $(,)?) => {
            #[derive(Debug, Clone, Copy, PartialEq)]
            pub enum Plan {
                $($variant),+
            }

            impl Plan {
                pub const ALL: &'static [Plan] = &[$(Plan::$variant),+];

                pub fn code(self) -> &'static str {
                    match self {
                        $(Plan::$variant => $code),+
                    }
                }

                pub fn label(self) -> &'static str {
                    match self {
                        $(Plan::$variant => $label),+
                    }
                }

                pub fn monthly_price(self) -> u32 {
                    match self {
                        $(Plan::$variant => $price),+
                    }
                }

                pub fn from_code(code: &str) -> Option<Plan> {
                    match code {
                        $($code => Some(Plan::$variant),)+
                        _ => None,
                    }
                }
            }
        };
    }

    plans! {
        Free  { code: "free",  label: "무료",   price: 0 },
        Basic { code: "basic", label: "베이직", price: 9_900 },
        Pro   { code: "pro",   label: "프로",   price: 19_900 },
        Team  { code: "team",  label: "팀",     price: 49_000 }, // 한 줄만 더하면 네 곳이 함께 바뀝니다.
    }
}

fn main() {
    println!("1. 비권장: 손으로 맞춘 목록");
    let listed: Vec<&str> = v1::Plan::ALL.iter().map(|p| p.code()).collect();
    println!("  ALL = {listed:?}"); // 출력:   ALL = ["free", "basic", "pro"]
    // 검사하는 쪽도 배리언트 목록을 손으로 적어야 합니다. ALL을 믿으면 Team은 검사조차 되지 않습니다.
    for plan in [
        v1::Plan::Free,
        v1::Plan::Basic,
        v1::Plan::Pro,
        v1::Plan::Team,
    ] {
        let back = v1::Plan::from_code(plan.code());
        println!(
            "  {plan:?}: code() = {:?}, from_code(code()) = {back:?}",
            plan.code()
        );
    }
    // 출력:   Free: code() = "free", from_code(code()) = Some(Free)
    // 출력:   Basic: code() = "basic", from_code(code()) = Some(Basic)
    // 출력:   Pro: code() = "pro", from_code(code()) = Some(Pro)
    // 출력:   Team: code() = "team", from_code(code()) = None

    println!("2. 권장: 표 하나에서 열거형과 메서드를 모두 만듦");
    let listed: Vec<&str> = v2::Plan::ALL.iter().map(|p| p.code()).collect();
    println!("  ALL = {listed:?}"); // 출력:   ALL = ["free", "basic", "pro", "team"]
    for &plan in v2::Plan::ALL {
        let back = v2::Plan::from_code(plan.code());
        println!(
            "  {plan:?}: {} {}원, from_code(code()) = {back:?}",
            plan.label(),
            plan.monthly_price()
        );
    }
    // 출력:   Free: 무료 0원, from_code(code()) = Some(Free)
    // 출력:   Basic: 베이직 9900원, from_code(code()) = Some(Basic)
    // 출력:   Pro: 프로 19900원, from_code(code()) = Some(Pro)
    // 출력:   Team: 팀 49000원, from_code(code()) = Some(Team)
}
