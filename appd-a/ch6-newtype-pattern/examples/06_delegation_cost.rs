// 지침: 뉴타입은 안쪽 타입의 트레이트와 메서드를 물려받지 않습니다. derive로 되는 것은 derive하고,
//       나머지는 필요한 것만 손으로 위임하되, 안쪽 구현에 그대로 넘겨서 동작을 잃지 않게 합니다.
// 비권장: write!(f, "{}", self.0)로 Display 위임, 합계를 낼 때마다 .0을 꺼내 더함
// 권장:   fmt::Display::fmt(&self.0, f)로 위임, Sum 구현, 읽기용 as_str() 하나
//
// 주문서의 상품 코드와 수량을 표로 출력합니다. 표는 `{:<8}`, `{:>4}`로 칸을 맞춥니다.
//
// 실행: cargo run -p er-ch6-newtype-pattern --example 06_delegation_cost

use std::fmt;

const ORDER: [(&str, u32); 3] = [("A-100", 3), ("BX-7", 12), ("C-2048", 1)];

// 두 판이 함께 쓰는 출력 함수입니다. 너비와 정렬은 부르는 쪽이 서식 지정으로 정합니다.
fn print_table<S: fmt::Display, Q: fmt::Display>(lines: &[(S, Q)]) {
    for (sku, qty) in lines {
        println!("    |{sku:<8}|{qty:>4}|");
    }
}

// ---- 비권장 ----------------------------------------------------------------

mod loose {
    use std::fmt;

    pub struct Sku(pub String);
    pub struct Qty(pub u32);

    // write!는 서식 문자열 "{}"로 새 출력을 만듭니다. 부르는 쪽이 준 너비·정렬(f에 담긴 것)은
    // 이 "{}"에 전달되지 않고 버려집니다.
    impl fmt::Display for Sku {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl fmt::Display for Qty {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

// ---- 권장 ------------------------------------------------------------------

mod strict {
    use std::fmt;
    use std::iter::Sum;
    use std::ops::Add;

    // 안쪽 String이 구현한 트레이트 가운데 "필드마다 같은 트레이트를 부른다"로 충분한 것은
    // derive 한 줄로 되찾습니다.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Sku(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct Qty(pub u32);

    impl Sku {
        pub fn new(code: &str) -> Sku {
            Sku(code.to_string())
        }

        // String의 메서드를 하나씩 위임하는 대신, 읽기 전용 &str을 내주는 메서드 하나를 둡니다.
        // 부르는 쪽은 sku.as_str().starts_with(..)처럼 &str의 메서드를 모두 씁니다.
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    // Display는 derive할 수 없으므로 손으로 씁니다. 안쪽 값의 Display::fmt에 f를 그대로 넘기면
    // 너비, 정렬, 정밀도 같은 서식 지정이 함께 넘어갑니다.
    impl fmt::Display for Sku {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&self.0, f)
        }
    }

    impl fmt::Display for Qty {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&self.0, f)
        }
    }

    // 수량 + 수량 = 수량. Iterator::sum()이 요구하는 Sum도 derive가 안 되므로 손으로 씁니다.
    impl Add for Qty {
        type Output = Qty;
        fn add(self, rhs: Qty) -> Qty {
            Qty(self.0 + rhs.0)
        }
    }

    impl Sum for Qty {
        fn sum<I: Iterator<Item = Qty>>(iter: I) -> Qty {
            iter.fold(Qty::default(), |acc, q| acc + q)
        }
    }
}

fn main() {
    println!("[비권장] write!로 위임");
    let lines: Vec<(loose::Sku, loose::Qty)> = ORDER
        .iter()
        .map(|&(code, n)| (loose::Sku(code.to_string()), loose::Qty(n)))
        .collect();
    print_table(&lines);
    // 합계를 낼 때마다 .0으로 꺼내 u32로 더하고 다시 감쌉니다. 이 사이에서는 단위 보호가 없습니다.
    let total = loose::Qty(lines.iter().map(|(_, q)| q.0).sum());
    println!("    합계 {total}");
    // 출력:
    //     |A-100|3|
    //     |BX-7|12|
    //     |C-2048|1|
    //     합계 16

    println!("[권장] 안쪽 구현에 그대로 위임");
    let lines: Vec<(strict::Sku, strict::Qty)> = ORDER
        .iter()
        .map(|&(code, n)| (strict::Sku::new(code), strict::Qty(n)))
        .collect();
    print_table(&lines);
    let total: strict::Qty = lines.iter().map(|(_, q)| *q).sum();
    println!("    합계 {total}");
    // 출력:
    //     |A-100   |   3|
    //     |BX-7    |  12|
    //     |C-2048  |   1|
    //     합계 16

    // sku.starts_with("A")는 E0599입니다. Sku에는 String의 메서드가 없기 때문입니다.
    let a_codes: Vec<&str> = lines
        .iter()
        .map(|(sku, _)| sku.as_str())
        .filter(|code| code.starts_with('A'))
        .collect();
    println!("    A로 시작하는 코드: {a_codes:?}");
    // 출력:     A로 시작하는 코드: ["A-100"]

    // derive한 Ord 덕분에 정렬과 최댓값이 바로 됩니다.
    let biggest = lines.iter().max_by_key(|(_, q)| *q).map(|(sku, _)| sku);
    println!("    가장 많이 주문한 상품: {biggest:?}");
    // 출력:     가장 많이 주문한 상품: Some(Sku("BX-7"))
}
