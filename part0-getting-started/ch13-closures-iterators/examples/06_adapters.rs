// 이 예제가 보여 주는 것: 반복자 어댑터(map, filter, rev, take, skip, enumerate, zip)와
//                        소비 메서드(collect, sum, count, max, min), 환경을 캡처하는 클로저를 쓰는 filter,
//                        for 반복문을 어댑터 체인으로 바꿔 쓰기
// 실행: cargo run -p p0-ch13-closures-iterators --example 06_adapters
//
// - 어댑터: 반복자를 받아 "다른 반복자"를 돌려줍니다. 게으르므로 그것만으로는 아무 일도 하지 않습니다.
// - 소비 메서드: 내부에서 next를 끝까지(또는 필요한 만큼) 불러 반복자를 다 써 버리고 결과값 하나를 돌려줍니다.
// 그래서 체인은 "반복자 만들기 → 어댑터 여러 개 → 소비 메서드 하나" 모양이 됩니다.

struct Student {
    name: String,
    score: u32,
}

// cutoff 이상인 학생의 이름만 모읍니다. filter의 클로저가 매개변수 cutoff를 캡처합니다.
// 돌려주는 &str은 students 안의 String을 가리키므로, 참조 매개변수가 하나뿐인 이 함수에서는
// 라이프타임 생략 규칙(ch10)에 따라 students와 같은 라이프타임으로 정해집니다.
fn passed_names(students: &[Student], cutoff: u32) -> Vec<&str> {
    students
        .iter()
        .filter(|s| s.score >= cutoff) // s는 &&Student지만 필드 접근은 자동으로 역참조됩니다.
        .map(|s| s.name.as_str())
        .collect() // 반환 타입 Vec<&str>을 보고 무엇을 모을지 정합니다.
}

fn main() {
    // 1. 어댑터를 하나씩
    // 결과를 눈으로 보려고 매번 collect로 벡터에 모읍니다. 범위(1..=6)는 i32 값을 내놓습니다.
    println!("== 1. 어댑터 ==");
    let mapped: Vec<i32> = (1..=6).map(|n| n * 10).collect();
    println!("map(n * 10):    {mapped:?}"); // 출력: map(n * 10):    [10, 20, 30, 40, 50, 60]

    // filter의 클로저는 항목의 참조(&i32)를 받습니다. 항목을 가져가지 않고 들여다보기만 하고,
    // true를 돌려준 항목만 다음 단계로 내보냅니다.
    let evens: Vec<i32> = (1..=6).filter(|n| n % 2 == 0).collect();
    println!("filter(짝수):   {evens:?}"); // 출력: filter(짝수):   [2, 4, 6]

    let reversed: Vec<i32> = (1..=6).rev().collect();
    println!("rev():          {reversed:?}"); // 출력: rev():          [6, 5, 4, 3, 2, 1]

    let first_three: Vec<i32> = (1..=6).take(3).collect();
    println!("take(3):        {first_three:?}"); // 출력: take(3):        [1, 2, 3]

    let after_four: Vec<i32> = (1..=6).skip(4).collect();
    println!("skip(4):        {after_four:?}"); // 출력: skip(4):        [5, 6]

    // enumerate는 (인덱스, 항목) 튜플을 내놓습니다. ch3의 weekdays.iter().enumerate()가 이것입니다.
    let days = ["월", "화", "수"];
    let numbered: Vec<(usize, &&str)> = days.iter().enumerate().collect();
    println!("enumerate():    {numbered:?}"); // 출력: enumerate():    [(0, "월"), (1, "화"), (2, "수")]

    // zip은 두 반복자에서 하나씩 꺼내 (왼쪽, 오른쪽) 튜플을 만듭니다. 짧은 쪽이 끝나면 함께 끝납니다.
    let names = ["가람", "나래", "다온"];
    let scores = [88, 72, 95, 60];
    let pairs: Vec<(&&str, &i32)> = names.iter().zip(scores.iter()).collect();
    println!("zip():          {pairs:?}"); // 출력: zip():          [("가람", 88), ("나래", 72), ("다온", 95)]

    // 2. 소비 메서드
    println!("\n== 2. 소비 메서드 ==");
    // sum은 여러 타입으로 합할 수 있어서 결과 타입을 적어 줘야 합니다(collect와 같은 이유).
    let total: i32 = scores.iter().sum();
    println!("sum:   {total}"); // 출력: sum:   315

    // iter()로 만든 반복자의 항목은 &i32이므로 filter의 클로저는 &&i32를 받습니다. 그래서 **로 두 번 따라갑니다.
    let high = scores.iter().filter(|s| **s >= 80).count();
    println!("count: 80점 이상 {high}명"); // 출력: count: 80점 이상 2명

    // max와 min은 빈 반복자일 수도 있으므로 Option을 돌려줍니다.
    println!("max:   {:?}", scores.iter().max()); // 출력: max:   Some(95)
    let nothing: [i32; 0] = [];
    println!("min:   {:?}", nothing.iter().min()); // 출력: min:   None

    // collect는 벡터뿐 아니라 String 등 여러 컬렉션을 만들 수 있습니다. 문자를 모으면 String이 됩니다.
    let backwards: String = "안녕하세요".chars().rev().collect();
    println!("collect(String): {backwards}"); // 출력: collect(String): 요세하녕안

    // 소비 메서드를 부른 반복자는 다 써 버린 것이라 다시 쓸 수 없습니다.
    // 예: let it = scores.iter(); let a: i32 = it.sum(); it.count(); → E0382
    // (compile_fail/e0382_consumed_iterator.rs)

    // 3. 환경을 캡처하는 클로저와 filter
    println!("\n== 3. 환경을 캡처하는 filter ==");
    let students = vec![
        Student {
            name: String::from("가람"),
            score: 88,
        },
        Student {
            name: String::from("나래"),
            score: 72,
        },
        Student {
            name: String::from("다온"),
            score: 95,
        },
    ];
    println!("80점 이상: {:?}", passed_names(&students, 80)); // 출력: 80점 이상: ["가람", "다온"]
    println!("90점 이상: {:?}", passed_names(&students, 90)); // 출력: 90점 이상: ["다온"]
    // 같은 함수가 cutoff 값에 따라 다르게 걸러 냅니다. 클로저가 cutoff를 캡처했기 때문입니다.
    // 이름 없는 함수를 그 자리에서 만드는 것만이 아니라, 주변 값을 쓸 수 있다는 점이 클로저의 힘입니다.

    // 4. 반복문을 어댑터 체인으로
    println!("\n== 4. for 반복문 vs 어댑터 체인 ==");
    // 1부터 10까지 중 짝수의 제곱을 모두 더합니다.
    let mut loop_total = 0;
    for n in 1..=10 {
        if n % 2 == 0 {
            loop_total += n * n;
        }
    }
    let chain_total: i32 = (1..=10).filter(|n| n % 2 == 0).map(|n| n * n).sum();
    println!("for 반복문: {loop_total}"); // 출력: for 반복문: 220
    println!("어댑터 체인: {chain_total}"); // 출력: 어댑터 체인: 220
    // 체인은 "걸러서(filter) 바꾸고(map) 더한다(sum)"를 그대로 읽을 수 있고, 가변 변수 loop_total이 필요 없습니다.
}
