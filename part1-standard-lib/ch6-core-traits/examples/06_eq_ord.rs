// 이 예제가 보여 주는 것: PartialEq/Eq/PartialOrd/Ord의 관계.
//                        derive가 필드 선언 순서로 비교한다는 것, 일부 필드로만 비교하도록 손으로 구현하기,
//                        Ordering의 then/then_with/reverse, max/min/clamp, 부분 순서(f64), 서로 다른 타입끼리의 ==
// 실행: cargo run -p p1-ch6-core-traits --example 06_eq_ord
//
// 손으로 구현할 때의 원칙(Ord 문서): 비교 논리는 Ord::cmp 한 곳에만 쓰고,
// PartialOrd는 Some(self.cmp(other)), PartialEq는 cmp가 보는 필드와 같은 필드로 맞춥니다.
// 네 트레이트가 서로 다른 답을 내면 정렬·BTreeMap·BinaryHeap이 엉뚱하게 동작합니다.

use std::cmp::Ordering;
use std::collections::BTreeSet;

// ---------- 1. derive는 필드를 선언한 순서대로 비교합니다 ----------
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ByName {
    name: &'static str, // 이름이 먼저
    score: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ByScore {
    score: u32, // 점수가 먼저
    name: &'static str,
}

// 열거형은 먼저 선언한 배리언트가 작습니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Low,
    Normal,
    High,
}

// ---------- 2. 일부 필드로만 비교하도록 손으로 구현 ----------
// 순위: 점수 내림차순, 같으면 이름 오름차순. joined(가입 순번)는 비교에 쓰지 않습니다.
#[derive(Debug)]
struct Player {
    name: &'static str,
    score: u32,
    joined: u32,
}

impl Ord for Player {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .score
            .cmp(&self.score) // 자리를 바꿔 내림차순
            .then_with(|| self.name.cmp(other.name)) // 점수가 같을 때만 계산됩니다.
    }
}

// PartialOrd는 Ord에 맡깁니다. clippy의 non_canonical_partial_ord_impl이 이 모양을 권합니다.
impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ==도 cmp와 같은 필드(score, name)만 봅니다. joined까지 비교하면
// "cmp는 Equal인데 ==는 false"인 쌍이 생겨 규칙이 깨집니다.
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.name == other.name
    }
}

impl Eq for Player {} // 메서드가 없는 표시용 트레이트입니다. "a == a는 언제나 참"이라고 약속합니다.

// ---------- 3. 서로 다른 타입끼리의 == ----------
#[derive(Debug)]
struct Code(String);

// Code == &str. 문서는 "내 타입 쪽에만" 구현하라고 권합니다(&str 쪽에 PartialEq<Code>를 붙이지 않음).
impl PartialEq<&str> for Code {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

fn main() {
    println!("== 1. derive와 필드 순서 ==");
    // 같은 데이터를 담았지만 필드 순서가 다른 두 타입을 정렬합니다(슬라이스의 sort는 배열에도 쓸 수 있습니다).
    let mut a = [
        ByName {
            name: "지영",
            score: 70,
        },
        ByName {
            name: "민수",
            score: 95,
        },
    ];
    let mut b = [
        ByScore {
            score: 95,
            name: "민수",
        },
        ByScore {
            score: 70,
            name: "지영",
        },
    ];
    a.sort();
    b.sort();
    let show_a: Vec<String> = a
        .iter()
        .map(|p| format!("{}({})", p.name, p.score))
        .collect();
    let show_b: Vec<String> = b
        .iter()
        .map(|p| format!("{}({})", p.name, p.score))
        .collect();
    println!("이름 먼저: {show_a:?}"); // 출력: 이름 먼저: ["민수(95)", "지영(70)"]
    println!("점수 먼저: {show_b:?}"); // 출력: 점수 먼저: ["지영(70)", "민수(95)"]
    let urgent = [Priority::Normal, Priority::High, Priority::Low]
        .into_iter()
        .max();
    println!("{urgent:?} {}", Priority::Low < Priority::High); // 출력: Some(High) true

    println!("\n== 2. 손으로 구현한 Ord ==");
    let mut ranking = [
        Player {
            name: "하늘",
            score: 80,
            joined: 3,
        },
        Player {
            name: "민수",
            score: 95,
            joined: 1,
        },
        Player {
            name: "가온",
            score: 80,
            joined: 2,
        },
    ];
    ranking.sort();
    for (i, p) in ranking.iter().enumerate() {
        println!("{}위 {} {}점 (가입 {}번)", i + 1, p.name, p.score, p.joined);
    }
    // 출력:
    // 1위 민수 95점 (가입 1번)
    // 2위 가온 80점 (가입 2번)
    // 3위 하늘 80점 (가입 3번)

    // Ord와 Eq가 "같다"고 하면 BTreeSet은 같은 원소로 봅니다. joined가 달라도 마찬가지입니다.
    let mut set = BTreeSet::new();
    let first = set.insert(Player {
        name: "민수",
        score: 95,
        joined: 1,
    });
    let second = set.insert(Player {
        name: "민수",
        score: 95,
        joined: 7,
    });
    println!(
        "첫 insert {first}, 둘째 insert {second}, 원소 {}개",
        set.len()
    ); // 출력: 첫 insert true, 둘째 insert false, 원소 1개

    // max는 같으면 두 번째 인자를, min은 첫 번째 인자를 돌려준다고 문서에 정해져 있습니다.
    let p1 = Player {
        name: "지영",
        score: 70,
        joined: 4,
    };
    let p2 = Player {
        name: "지영",
        score: 70,
        joined: 9,
    };
    println!("p1 == p2? {}", p1 == p2); // 출력: p1 == p2? true
    let bigger = Player {
        name: "지영",
        score: 70,
        joined: 4,
    }
    .max(p2);
    let smaller = p1.min(Player {
        name: "지영",
        score: 70,
        joined: 9,
    });
    println!(
        "max → 가입 {}번, min → 가입 {}번",
        bigger.joined, smaller.joined
    ); // 출력: max → 가입 9번, min → 가입 4번

    println!("\n== 3. Ordering 다루기 ==");
    let o = "가방".cmp("나무"); // 문자열은 사전식(코드 포인트 순)으로 비교합니다.
    println!("{o:?} {:?} {}", o.reverse(), o.is_lt()); // 출력: Less Greater true
    println!("{:?}", 3.cmp(&3).then(10.cmp(&2))); // 출력: Greater
    println!("{:?}", 1.cmp(&2).then(10.cmp(&2))); // 출력: Less  (앞 비교가 Equal이 아니면 뒤는 무시)

    println!("\n== 4. max, min, clamp ==");
    println!("{} {}", std::cmp::max(3, 8), 150.clamp(0, 100)); // 출력: 8 100
    // f64는 Ord가 아니라서 std::cmp::max를 쓸 수 없고, f64 자신의 메서드를 씁니다.
    println!("{} {}", 2.5_f64.max(1.0), 1.7_f64.clamp(0.0, 1.0)); // 출력: 2.5 1

    println!("\n== 5. 부분 순서: f64와 NaN ==");
    let x = 1.0_f64;
    let nan = f64::NAN;
    println!("{:?} {:?}", x.partial_cmp(&2.0), nan.partial_cmp(&x)); // 출력: Some(Less) None
    println!("{} {} {}", nan < x, nan > x, nan >= x); // 출력: false false false
    let other_nan = f64::NAN;
    println!("NaN == NaN? {}", nan == other_nan); // 출력: NaN == NaN? false

    println!("\n== 6. 서로 다른 타입끼리 == ==");
    let code = Code(String::from("A-100"));
    println!("{}", code == "A-100"); // 출력: true
    // 표준 라이브러리도 여러 조합을 구현해 둡니다.
    let greeting = String::from("hi");
    let list: Vec<i32> = (1..=2).collect();
    println!("{} {}", greeting == "hi", list == [1, 2]); // 출력: true true  (String == &str, Vec == 배열)
}
