//! 지침 3: "이 기능도 있나?"는 `Any`가 아니라 트레이트로 묻는다
//!
//! 학습 자료 목록에서 채점할 수 있는 자료(`Gradable`)의 만점을 더합니다.
//! 나중에 다른 팀이 채점할 수 있는 코딩 과제(`CodingTask`)를 더합니다.
//! - 비권장(`Any`판): `Any`는 "Gradable을 구현했는가"를 물을 수 없으므로 아는 구체 타입을 하나씩 확인합니다.
//! - 권장 1: 트레이트에 기능 조회 메서드 `as_gradable`을 둡니다.
//! - 권장 2: 채점할 자료는 처음부터 `Lesson + Gradable`을 요구하는 타입으로 따로 담습니다.
//! - 권장 3: 메서드가 없는 성질(수료 인정)은 마커 트레이트로 표시합니다.
//!
//! 실행: cargo run -p er-ch19-avoid-reflection --example 03_capability_query

use std::any::Any;

/// 채점할 수 있는 자료
trait Gradable {
    fn max_points(&self) -> u32;
}

trait Lesson: Any {
    fn title(&self) -> &str;

    /// 기능 조회 메서드: 채점할 수 있는 자료라면 자기 자신을 `&dyn Gradable`로 빌려줍니다.
    /// 기본 구현은 "채점할 수 없음"이고, `Gradable`을 구현한 타입만 재정의합니다.
    fn as_gradable(&self) -> Option<&dyn Gradable> {
        None
    }
}

struct Video {
    title: String,
}

/// 문항당 10점
struct Quiz {
    title: String,
    questions: u32,
}

/// 나중에 더한 코딩 과제. 테스트 하나당 20점
struct CodingTask {
    title: String,
    test_cases: u32,
}

impl Gradable for Quiz {
    fn max_points(&self) -> u32 {
        self.questions * 10
    }
}

impl Gradable for CodingTask {
    fn max_points(&self) -> u32 {
        self.test_cases * 20
    }
}

impl Lesson for Video {
    fn title(&self) -> &str {
        &self.title
    }
}

impl Lesson for Quiz {
    fn title(&self) -> &str {
        &self.title
    }
    fn as_gradable(&self) -> Option<&dyn Gradable> {
        Some(self) // &Quiz → &dyn Gradable 강제 변환
    }
}

impl Lesson for CodingTask {
    fn title(&self) -> &str {
        &self.title
    }
    fn as_gradable(&self) -> Option<&dyn Gradable> {
        Some(self)
    }
}

fn video() -> Video {
    Video {
        title: "1주차 강의 영상".to_string(),
    }
}

fn quiz() -> Quiz {
    Quiz {
        title: "1주차 퀴즈".to_string(),
        questions: 10,
    }
}

fn coding_task() -> CodingTask {
    CodingTask {
        title: "1주차 코딩 과제".to_string(),
        test_cases: 3,
    }
}

// ---------------------------------------------------------------
// 비권장: Any판
// ---------------------------------------------------------------

/// `&dyn Any`가 답할 수 있는 질문은 "정확히 이 구체 타입인가?"뿐입니다.
/// 이 함수를 쓸 때 채점할 수 있는 타입은 Quiz뿐이었고, CodingTask는 나중에 생겼습니다.
fn total_points_any(items: &[Box<dyn Lesson>]) -> u32 {
    items
        .iter()
        .map(|item| {
            let any: &dyn Any = &**item; // &Box<dyn Lesson> → &dyn Lesson → &dyn Any
            any.downcast_ref::<Quiz>().map_or(0, Gradable::max_points)
        })
        .sum()
}

// ---------------------------------------------------------------
// 권장 1: 기능 조회 메서드
// ---------------------------------------------------------------

/// "채점할 수 있느냐"는 질문에 각 타입이 직접 답합니다. 구체 타입 목록이 없습니다.
fn total_points_capability(items: &[Box<dyn Lesson>]) -> u32 {
    items
        .iter()
        .filter_map(|item| item.as_gradable())
        .map(Gradable::max_points)
        .sum()
}

// ---------------------------------------------------------------
// 권장 2: 처음부터 바운드로 요구
// ---------------------------------------------------------------

/// 두 트레이트를 모두 구현한 자료를 가리키는 조합 트레이트(부록 ch12 2.4절)
trait GradedLesson: Lesson + Gradable {}
impl<T: Lesson + Gradable> GradedLesson for T {}

/// 제목만 필요한 함수는 `&dyn Lesson`을 받습니다.
fn title_of(item: &dyn Lesson) -> &str {
    item.title()
}

// ---------------------------------------------------------------
// 권장 3: 메서드가 없는 성질은 마커 트레이트로
// ---------------------------------------------------------------

/// 수료 요건에 들어가는 자료. 메서드는 없고 "이 자료를 끝내야 수료한다"는 약속만 표시합니다.
/// 다른 언어라면 클래스 이름 규칙이나 실행 중에 읽는 표시(애너테이션)로 했을 일입니다.
trait Certified: Lesson {}
impl Certified for Quiz {}
impl Certified for CodingTask {}
// Video에는 붙이지 않습니다. `&video`를 `&dyn Certified` 자리에 넣으면 E0277입니다.

fn main() {
    let course: Vec<Box<dyn Lesson>> =
        vec![Box::new(video()), Box::new(quiz()), Box::new(coding_task())];

    println!("1. Any판: 아는 구체 타입(Quiz)만 확인");
    println!("    만점 합계 {} (기대 160)", total_points_any(&course));
    // 출력:     만점 합계 100 (기대 160)

    println!("2. 기능 조회 메서드: item.as_gradable()");
    for item in &course {
        let answer = match item.as_gradable() {
            Some(g) => format!("채점 가능, 만점 {}", g.max_points()),
            None => "채점 없음".to_string(),
        };
        println!("    {}: {answer}", item.title());
    }
    println!("    만점 합계 {}", total_points_capability(&course));
    // 출력:
    //     1주차 강의 영상: 채점 없음
    //     1주차 퀴즈: 채점 가능, 만점 100
    //     1주차 코딩 과제: 채점 가능, 만점 60
    //     만점 합계 160

    println!("3. 처음부터 바운드로: Vec<Box<dyn GradedLesson>>");
    let graded: Vec<Box<dyn GradedLesson>> = vec![Box::new(quiz()), Box::new(coding_task())];
    let total: u32 = graded.iter().map(|g| g.max_points()).sum();
    // g.as_ref()는 &dyn GradedLesson입니다. 슈퍼트레이트 Lesson의 자리에 넘기면 업캐스트됩니다.
    let titles: Vec<&str> = graded.iter().map(|g| title_of(g.as_ref())).collect();
    println!("    {} → 만점 합계 {total}", titles.join(", "));
    // 출력:     1주차 퀴즈, 1주차 코딩 과제 → 만점 합계 160

    println!("4. 마커 트레이트: 수료 요건");
    let (q, c) = (quiz(), coding_task());
    let requirements: [&dyn Certified; 2] = [&q, &c];
    let names: Vec<&str> = requirements.iter().map(|r| r.title()).collect();
    println!("    {}", names.join(", "));
    // 출력:     1주차 퀴즈, 1주차 코딩 과제
}
