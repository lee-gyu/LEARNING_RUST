// 지침: derive는 "모든 필드를, 선언 순서대로" 비교·해시·복제하는 구현을 만듭니다.
//       그 규칙이 타입의 뜻과 맞을 때만 derive하고, 맞지 않으면 표현을 고치거나 손으로 구현합니다.
// 비권장: 분수 Ratio { num, den }에 비교 트레이트를 모두 derive (1/2 != 2/4, 1/3 > 1/2)
//         Rc를 감싼 Shared<T>에 derive(Clone) (쓸데없는 T: Clone 바운드가 붙음)
// 권장:   생성자에서 기약분수로 맞춘 뒤 PartialEq/Eq/Hash는 derive, Ord는 손으로
//         Shared<T>의 Clone은 Rc만 복제하도록 손으로
//
// 실행: cargo run -p er-ch10-standard-traits --example 01_derive_matches_meaning

use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;

const INPUT: [(i32, i32); 4] = [(1, 2), (2, 4), (1, 3), (2, 3)];

// ---- 비권장: 필드를 그대로 두고 모두 derive ---------------------------------

mod naive {
    // derive한 PartialEq는 (num, den)이 둘 다 같아야 같다고 하므로 1/2과 2/4가 다릅니다.
    // derive한 Ord는 num을 먼저, 같으면 den을 비교하는 사전식이라 1/3이 1/2보다 큽니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Ratio {
        pub num: i32,
        pub den: i32,
    }
}

// ---- 권장: 표현을 하나로 맞추고, 그래도 안 맞는 것만 손으로 -----------------

mod ratio {
    use std::cmp::Ordering;
    use std::fmt;

    /// 기약분수로만 존재하는 분수입니다. 분모는 언제나 양수입니다.
    // 생성자가 표현을 하나로 맞추므로 "필드가 모두 같다 = 같은 수"가 됩니다.
    // 그래서 PartialEq, Eq, Hash는 derive가 만든 구현이 그대로 맞습니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Ratio {
        num: i32,
        den: i32,
    }

    fn gcd(a: i64, b: i64) -> i64 {
        if b == 0 { a.abs() } else { gcd(b, a % b) }
    }

    impl Ratio {
        /// 분모가 0이거나 결과가 i32에 들어가지 않으면 None입니다.
        pub fn new(num: i32, den: i32) -> Option<Ratio> {
            if den == 0 {
                return None;
            }
            let (mut n, mut d) = (i64::from(num), i64::from(den));
            if d < 0 {
                (n, d) = (-n, -d);
            }
            let g = gcd(n, d);
            Some(Ratio {
                num: i32::try_from(n / g).ok()?,
                den: i32::try_from(d / g).ok()?,
            })
        }
    }

    // 크기 순서는 필드 순서로 정해지지 않으므로 손으로 씁니다.
    // a/b와 c/d(b, d > 0)는 a*d와 c*b를 비교하면 됩니다. 넘치지 않게 i64로 곱합니다.
    // 기약분수라서 "cmp가 Equal"과 "필드가 모두 같음(==)"이 정확히 같은 경우입니다.
    impl Ord for Ratio {
        fn cmp(&self, other: &Self) -> Ordering {
            let left = i64::from(self.num) * i64::from(other.den);
            let right = i64::from(other.num) * i64::from(self.den);
            left.cmp(&right)
        }
    }

    // Ord를 손으로 썼으면 PartialOrd도 손으로, Ord에 맡기는 모양으로 씁니다.
    // 여기서 derive(PartialOrd)를 붙이면 clippy의 derive_ord_xor_partial_ord(기본 deny)가 막습니다.
    impl PartialOrd for Ratio {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    // 분수는 사람에게 보여 줄 모양이 "1/2" 하나로 분명하므로 Display를 둡니다.
    impl fmt::Display for Ratio {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

// ---- 제네릭 타입과 derive의 바운드 -------------------------------------------

/// 복제할 수 없는 자원의 예: 연결은 하나뿐이어야 하므로 Clone을 구현하지 않습니다.
#[derive(Debug)]
struct Connection {
    id: u32,
}

// 비권장: derive(Clone)는 impl<T: Clone> Clone for DerivedShared<T>를 만듭니다.
// 복제하는 것은 Rc뿐이라 T가 Clone일 필요가 없는데도 바운드가 붙습니다.
#[derive(Debug, Clone)]
struct DerivedShared<T>(Rc<T>);

// 권장: 바운드 없이 손으로 씁니다. Rc::clone은 참조 카운트만 늘립니다.
#[derive(Debug)]
struct Shared<T>(Rc<T>);

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(Rc::clone(&self.0))
    }
}

fn show_naive<'a>(items: impl IntoIterator<Item = &'a naive::Ratio>) -> String {
    let parts: Vec<String> = items
        .into_iter()
        .map(|r| format!("{}/{}", r.num, r.den))
        .collect();
    format!("[{}]", parts.join(", "))
}

fn show<'a>(items: impl IntoIterator<Item = &'a ratio::Ratio>) -> String {
    let parts: Vec<String> = items.into_iter().map(|r| r.to_string()).collect();
    format!("[{}]", parts.join(", "))
}

fn main() {
    println!("입력: 1/2, 2/4, 1/3, 2/3");
    // 출력: 입력: 1/2, 2/4, 1/3, 2/3

    println!("[비권장] 필드 그대로 모두 derive");
    let values: Vec<naive::Ratio> = INPUT
        .iter()
        .map(|&(num, den)| naive::Ratio { num, den })
        .collect();
    println!("  1/2 == 2/4 ? {}", values[0] == values[1]);
    // 출력:   1/2 == 2/4 ? false
    let sorted: BTreeSet<naive::Ratio> = values.iter().copied().collect();
    println!("  정렬: {}", show_naive(&sorted));
    // 출력:   정렬: [1/2, 1/3, 2/3, 2/4]
    let distinct: HashSet<naive::Ratio> = values.iter().copied().collect();
    println!("  서로 다른 값: {}개", distinct.len());
    // 출력:   서로 다른 값: 4개

    println!("[권장] 생성자에서 기약분수로, Ord는 손으로");
    let values: Vec<ratio::Ratio> = INPUT
        .iter()
        .filter_map(|&(num, den)| ratio::Ratio::new(num, den))
        .collect();
    println!("  1/2 == 2/4 ? {}", values[0] == values[1]);
    // 출력:   1/2 == 2/4 ? true
    let sorted: BTreeSet<ratio::Ratio> = values.iter().copied().collect();
    println!("  정렬: {}", show(&sorted));
    // 출력:   정렬: [1/3, 1/2, 2/3]
    let distinct: HashSet<ratio::Ratio> = values.iter().copied().collect();
    println!("  서로 다른 값: {}개", distinct.len());
    // 출력:   서로 다른 값: 3개
    println!(
        "  Ratio::new(3, -6) = {:?}",
        ratio::Ratio::new(3, -6).map(|r| r.to_string())
    );
    // 출력:   Ratio::new(3, -6) = Some("-1/2")

    println!("[비권장] derive(Clone)한 DerivedShared<T>");
    let name = DerivedShared(Rc::new(String::from("공용 이름")));
    let copy = name.clone(); // T = String은 Clone이라 통과합니다.
    println!(
        "  DerivedShared<String> 복제: 강한 참조 {}개",
        Rc::strong_count(&copy.0)
    );
    // 출력:   DerivedShared<String> 복제: 강한 참조 2개
    // DerivedShared(Rc::new(Connection { .. })).clone()은 E0599입니다
    // (compile_fail/e0599_derive_clone_bound.rs).

    println!("[권장] 손으로 쓴 Clone은 T: Clone을 요구하지 않음");
    let conn = Shared(Rc::new(Connection { id: 7 }));
    let other = conn.clone();
    println!(
        "  Shared<Connection> 복제: 강한 참조 {}개, 같은 연결 #{}? {}",
        Rc::strong_count(&conn.0),
        other.0.id,
        Rc::ptr_eq(&conn.0, &other.0)
    );
    // 출력:   Shared<Connection> 복제: 강한 참조 2개, 같은 연결 #7? true
}
