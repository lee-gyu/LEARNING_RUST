// 이 예제가 보여 주는 것: 같은 함수를 match → if let → let...else → ? → 변환 메서드 체인으로 한 단계씩 바꾸기
// 실행: cargo run -q -p er-ch3-option-result-transforms --example 01_refactor_steps
//
// 다섯 판의 함수는 모든 입력에서 같은 값을 돌려줍니다. 바뀌는 것은 "실패했을 때 어떻게 되는가"를
// 코드에 몇 번, 어디에 적느냐뿐입니다. 단계마다 그 부분이 줄어들어 행복한 경로가 드러납니다.
//
// 1번 구역의 함수는 값 하나가 단계를 거쳐 흘러가므로 체인까지 가는 것이 가장 읽기 좋습니다.
// 2번 구역의 함수는 중간 값 두 개를 마지막에 함께 쓰므로 ?에서 멈추는 것이 더 읽기 좋습니다.

struct Student {
    name: &'static str,
    // 시험을 보지 않은 학생은 None입니다. "점수 없음"은 에러가 아니라 정상적인 상태입니다.
    score: Option<u32>,
    // 짝 학생이 정해지지 않았으면 None입니다.
    buddy: Option<&'static str>,
}

const ROSTER: [Student; 4] = [
    Student {
        name: "가온",
        score: Some(92),
        buddy: Some("다솜"),
    },
    Student {
        name: "나래",
        score: None,
        buddy: Some("가온"),
    },
    Student {
        name: "다솜",
        score: Some(78),
        buddy: None,
    },
    Student {
        name: "라온",
        score: Some(85),
        buddy: Some("미르"), // 명단에 없는 학생
    },
];

fn grade(score: u32) -> char {
    match score {
        90.. => 'A',
        80..=89 => 'B',
        _ => 'C',
    }
}

fn find<'a>(roster: &'a [Student], name: &str) -> Option<&'a Student> {
    roster.iter().find(|s| s.name == name)
}

// ---------------------------------------------------------------------------
// 1. 값 하나가 흘러가는 함수: 학생 이름 → 학생 → 점수 → 등급
// ---------------------------------------------------------------------------

// 0단계: match. 실패 갈래(None => None)가 단계마다 하나씩 생기고, 단계가 늘수록 한 겹씩 깊어집니다.
// clippy는 안쪽 match만 map으로 바꾸라고 알려 줍니다. 바깥 match(Some이면 Option을 돌려주는 식,
// None이면 None)는 and_then과 같은 모양이지만 기본 린트로는 잡히지 않습니다.
#[expect(clippy::manual_map, reason = "비권장 형태를 비교하려고 남겨 둡니다")]
fn grade_of_match(roster: &[Student], name: &str) -> Option<char> {
    match find(roster, name) {
        Some(student) => match student.score {
            Some(score) => Some(grade(score)),
            None => None,
        },
        None => None,
    }
}

// 1단계: if let. None 갈래를 적지 않아도 되지만, 중첩은 그대로이고 실패했을 때의 값(None)이
// 함수 맨 끝으로 멀어집니다. clippy는 두 if를 let 체인 하나로 합치라고 알려 줍니다(collapsible_if).
#[expect(
    clippy::collapsible_if,
    reason = "비권장 형태를 비교하려고 남겨 둡니다"
)]
fn grade_of_if_let(roster: &[Student], name: &str) -> Option<char> {
    if let Some(student) = find(roster, name) {
        if let Some(score) = student.score {
            return Some(grade(score));
        }
    }
    None
}

// 2단계: let...else. 중첩이 풀려 행복한 경로가 왼쪽에 한 줄로 섭니다.
// 다만 else 블록이 하는 일이 "None을 돌려주며 끝내기"뿐이라면 그것은 ?가 하는 일 그대로입니다.
// clippy가 두 문장 모두 ?로 바꾸라고 알려 줍니다(question_mark).
#[expect(clippy::question_mark, reason = "비권장 형태를 비교하려고 남겨 둡니다")]
fn grade_of_let_else(roster: &[Student], name: &str) -> Option<char> {
    let Some(student) = find(roster, name) else {
        return None;
    };
    let Some(score) = student.score else {
        return None;
    };
    Some(grade(score))
}

// 3단계: ?. 실패 처리가 한 글자로 줄었습니다. 이름(student, score)은 아직 남아 있습니다.
fn grade_of_question(roster: &[Student], name: &str) -> Option<char> {
    let student = find(roster, name)?;
    let score = student.score?;
    Some(grade(score))
}

// 4단계: 변환 메서드 체인. 중간 이름이 사라지고 단계의 이름(and_then, map)만 남습니다.
// and_then: 다음 단계가 다시 Option을 돌려줄 때(점수가 없을 수 있음)
// map: 다음 단계가 실패하지 않을 때(등급 계산)
fn grade_of_chain(roster: &[Student], name: &str) -> Option<char> {
    find(roster, name).and_then(|s| s.score).map(grade)
}

// ---------------------------------------------------------------------------
// 2. 중간 값 두 개를 함께 쓰는 함수: 학생 점수와 짝 학생 점수의 차이
// ---------------------------------------------------------------------------

// ?판: 값마다 이름을 붙여 위에서 아래로 읽힙니다. 마지막 줄에서 두 값을 함께 씁니다.
fn buddy_gap_question(roster: &[Student], name: &str) -> Option<u32> {
    let student = find(roster, name)?;
    let my_score = student.score?;
    let buddy = find(roster, student.buddy?)?;
    let buddy_score = buddy.score?;
    Some(my_score.abs_diff(buddy_score))
}

// 체인판: 마지막 계산이 my_score를 써야 하므로, 뒤 단계가 그 값을 붙잡은 클로저 안으로 들어가야 합니다.
// 결과는 같지만 괄호와 클로저가 겹겹이 쌓여 1번 구역의 체인처럼 단계별로 읽히지 않습니다.
fn buddy_gap_chain(roster: &[Student], name: &str) -> Option<u32> {
    find(roster, name).and_then(|student| {
        student.score.and_then(|my_score| {
            student
                .buddy
                .and_then(|buddy| find(roster, buddy))
                .and_then(|buddy| buddy.score)
                .map(|buddy_score| my_score.abs_diff(buddy_score))
        })
    })
}

fn main() {
    println!("== 1. 값 하나가 흘러가는 함수: 다섯 판의 결과 비교 ==");
    for name in ["가온", "나래", "다솜", "없는이"] {
        let results = [
            grade_of_match(&ROSTER, name),
            grade_of_if_let(&ROSTER, name),
            grade_of_let_else(&ROSTER, name),
            grade_of_question(&ROSTER, name),
            grade_of_chain(&ROSTER, name),
        ];
        // 다섯 판이 모두 같은 값을 돌려줬는지 확인합니다.
        let same = results.iter().all(|r| *r == results[0]);
        println!("{name} → {:?} (다섯 판 일치: {same})", results[0]);
    }
    // 출력:
    // 가온 → Some('A') (다섯 판 일치: true)
    // 나래 → None (다섯 판 일치: true)
    // 다솜 → Some('C') (다섯 판 일치: true)
    // 없는이 → None (다섯 판 일치: true)

    println!();
    println!("== 2. 중간 값 두 개를 함께 쓰는 함수: ?판과 체인판 비교 ==");
    for name in ["가온", "나래", "다솜", "라온"] {
        let question = buddy_gap_question(&ROSTER, name);
        let chain = buddy_gap_chain(&ROSTER, name);
        println!("{name} → ?판: {question:?}, 체인판: {chain:?}");
    }
    // 출력:
    // 가온 → ?판: Some(14), 체인판: Some(14)
    // 나래 → ?판: None, 체인판: None
    // 다솜 → ?판: None, 체인판: None
    // 라온 → ?판: None, 체인판: None
}
