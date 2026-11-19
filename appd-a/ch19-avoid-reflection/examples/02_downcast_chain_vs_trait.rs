//! 지침 2: 타입별 처리는 다운캐스트 사슬 대신 트레이트 메서드로
//!
//! 강의 하나를 이루는 학습 자료(영상, 퀴즈, 읽기 자료)의 예상 학습 시간과
//! 오프라인 다운로드 크기를 구합니다. 나중에 다른 팀이 실시간 특강(`LiveSession`)을 더합니다.
//! - 비권장(`Any`판): 트레이트 객체를 `&dyn Any`로 올려 `downcast_ref`를 줄줄이 시도합니다.
//! - 권장(트레이트판): 필요한 동작을 트레이트 메서드로 두고 각 타입이 구현합니다.
//! - 닫힌 집합이라면(열거형판): 열거형과 `match`로 씁니다.
//!
//! 실행: cargo run -p er-ch19-avoid-reflection --example 02_downcast_chain_vs_trait

/// 강의 영상
struct Video {
    title: String,
    minutes: u32,
    size_mb: u32,
}

/// 퀴즈. 문항당 2분으로 계산합니다.
struct Quiz {
    title: String,
    questions: u32,
}

/// 읽기 자료(PDF). 쪽당 3분으로 계산합니다.
struct Reading {
    title: String,
    pages: u32,
    size_mb: u32,
}

/// 나중에 다른 팀이 더한 실시간 특강. 녹화본을 내려받을 수 있습니다.
struct LiveSession {
    title: String,
    minutes: u32,
    recording_mb: u32,
}

fn video() -> Video {
    Video {
        title: "1주차 강의 영상".to_string(),
        minutes: 25,
        size_mb: 180,
    }
}

fn quiz() -> Quiz {
    Quiz {
        title: "1주차 퀴즈".to_string(),
        questions: 10,
    }
}

fn reading() -> Reading {
    Reading {
        title: "읽기 자료".to_string(),
        pages: 12,
        size_mb: 4,
    }
}

fn live() -> LiveSession {
    LiveSession {
        title: "실시간 특강".to_string(),
        minutes: 60,
        recording_mb: 850,
    }
}

fn download_text(mb: Option<u32>) -> String {
    match mb {
        Some(mb) => format!("다운로드 {mb}MB"),
        None => "다운로드 없음".to_string(),
    }
}

// ---------------------------------------------------------------
// 비권장: Any판
// ---------------------------------------------------------------
mod any_version {
    use super::*;
    use std::any::Any;

    /// 공통 트레이트에는 제목만 있습니다. 나머지는 "실제 타입이 무엇인지" 물어서 처리합니다.
    /// `Any`를 슈퍼트레이트로 두었으므로 `&dyn Lesson`을 `&dyn Any`로 업캐스트할 수 있습니다(예제 04).
    pub trait Lesson: Any {
        fn title(&self) -> &str;
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
    }
    impl Lesson for Reading {
        fn title(&self) -> &str {
            &self.title
        }
    }
    // 새 타입은 제목만 구현하면 컴파일됩니다. 아래 두 함수를 고쳐야 한다는 사실은 아무도 알려 주지 않습니다.
    impl Lesson for LiveSession {
        fn title(&self) -> &str {
            &self.title
        }
    }

    pub fn study_minutes(item: &dyn Lesson) -> u32 {
        let any: &dyn Any = item;
        if let Some(v) = any.downcast_ref::<Video>() {
            v.minutes
        } else if let Some(q) = any.downcast_ref::<Quiz>() {
            q.questions * 2
        } else if let Some(r) = any.downcast_ref::<Reading>() {
            r.pages * 3
        } else {
            0 // 모르는 타입. LiveSession은 조용히 여기로 떨어집니다.
        }
    }

    pub fn download_mb(item: &dyn Lesson) -> Option<u32> {
        let any: &dyn Any = item;
        if let Some(v) = any.downcast_ref::<Video>() {
            return Some(v.size_mb);
        }
        if let Some(r) = any.downcast_ref::<Reading>() {
            return Some(r.size_mb);
        }
        None // LiveSession의 녹화본도 "다운로드 없음"이 됩니다.
    }
}

// ---------------------------------------------------------------
// 권장: 트레이트판
// ---------------------------------------------------------------
mod trait_version {
    use super::*;

    pub trait Lesson {
        fn title(&self) -> &str;

        /// 필수 메서드: 모든 자료가 답해야 하는 질문이므로 기본 구현을 두지 않습니다.
        /// 새 타입이 이 메서드를 빠뜨리면 E0046으로 컴파일되지 않습니다.
        fn study_minutes(&self) -> u32;

        /// 선택적 기능: 대부분의 자료가 지원하지 않으므로 기본 구현이 "지원하지 않음"을 돌려줍니다.
        /// 지원하는 타입만 재정의합니다.
        fn download_mb(&self) -> Option<u32> {
            None
        }
    }

    impl Lesson for Video {
        fn title(&self) -> &str {
            &self.title
        }
        fn study_minutes(&self) -> u32 {
            self.minutes
        }
        fn download_mb(&self) -> Option<u32> {
            Some(self.size_mb)
        }
    }

    impl Lesson for Quiz {
        fn title(&self) -> &str {
            &self.title
        }
        fn study_minutes(&self) -> u32 {
            self.questions * 2
        }
        // download_mb는 기본 구현(None)을 그대로 씁니다.
    }

    impl Lesson for Reading {
        fn title(&self) -> &str {
            &self.title
        }
        fn study_minutes(&self) -> u32 {
            self.pages * 3
        }
        fn download_mb(&self) -> Option<u32> {
            Some(self.size_mb)
        }
    }

    // 새 타입을 더한 팀이 자기 타입에 대한 답을 자기 impl 블록에 적습니다.
    impl Lesson for LiveSession {
        fn title(&self) -> &str {
            &self.title
        }
        fn study_minutes(&self) -> u32 {
            self.minutes
        }
        fn download_mb(&self) -> Option<u32> {
            Some(self.recording_mb)
        }
    }
}

// ---------------------------------------------------------------
// 닫힌 집합이라면: 열거형판
// ---------------------------------------------------------------
mod enum_version {
    use super::*;

    /// 자료의 종류를 이 크레이트가 모두 정한다면 열거형으로 나열할 수 있습니다.
    pub enum Material {
        Video(Video),
        Quiz(Quiz),
        Reading(Reading),
        Live(LiveSession), // 배리언트를 더하면 아래 모든 match가 E0004로 고칠 곳을 알려 줍니다.
    }

    impl Material {
        pub fn title(&self) -> &str {
            match self {
                Material::Video(v) => &v.title,
                Material::Quiz(q) => &q.title,
                Material::Reading(r) => &r.title,
                Material::Live(l) => &l.title,
            }
        }

        pub fn study_minutes(&self) -> u32 {
            match self {
                Material::Video(v) => v.minutes,
                Material::Quiz(q) => q.questions * 2,
                Material::Reading(r) => r.pages * 3,
                Material::Live(l) => l.minutes,
            }
        }

        pub fn download_mb(&self) -> Option<u32> {
            match self {
                Material::Video(v) => Some(v.size_mb),
                Material::Reading(r) => Some(r.size_mb),
                Material::Live(l) => Some(l.recording_mb),
                Material::Quiz(_) => None,
            }
        }
    }
}

fn main() {
    println!("1. Any판: 구체 타입을 하나씩 다운캐스트");
    let course: Vec<Box<dyn any_version::Lesson>> = vec![
        Box::new(video()),
        Box::new(quiz()),
        Box::new(reading()),
        Box::new(live()),
    ];
    let mut total = 0;
    for item in &course {
        let minutes = any_version::study_minutes(item.as_ref());
        total += minutes;
        let download = download_text(any_version::download_mb(item.as_ref()));
        println!("    {}: {minutes}분, {download}", item.title());
    }
    println!("    합계 {total}분");
    // 출력:
    //     1주차 강의 영상: 25분, 다운로드 180MB
    //     1주차 퀴즈: 20분, 다운로드 없음
    //     읽기 자료: 36분, 다운로드 4MB
    //     실시간 특강: 0분, 다운로드 없음
    //     합계 81분

    println!("2. 트레이트판: 각 타입이 메서드로 답함");
    let course: Vec<Box<dyn trait_version::Lesson>> = vec![
        Box::new(video()),
        Box::new(quiz()),
        Box::new(reading()),
        Box::new(live()),
    ];
    let mut total = 0;
    for item in &course {
        let minutes = item.study_minutes();
        total += minutes;
        let download = download_text(item.download_mb());
        println!("    {}: {minutes}분, {download}", item.title());
    }
    println!("    합계 {total}분");
    // 출력:
    //     1주차 강의 영상: 25분, 다운로드 180MB
    //     1주차 퀴즈: 20분, 다운로드 없음
    //     읽기 자료: 36분, 다운로드 4MB
    //     실시간 특강: 60분, 다운로드 850MB
    //     합계 141분

    println!("3. 열거형판: 닫힌 집합이라면 match로");
    use enum_version::Material;
    let course = [
        Material::Video(video()),
        Material::Quiz(quiz()),
        Material::Reading(reading()),
        Material::Live(live()),
    ];
    let total: u32 = course.iter().map(Material::study_minutes).sum();
    let downloads: Vec<String> = course
        .iter()
        .filter_map(|m| m.download_mb().map(|mb| format!("{} {mb}MB", m.title())))
        .collect();
    println!("    합계 {total}분, 내려받을 것: {}", downloads.join(", "));
    // 출력:     합계 141분, 내려받을 것: 1주차 강의 영상 180MB, 읽기 자료 4MB, 실시간 특강 850MB
}
