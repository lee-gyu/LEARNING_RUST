//! 지침 4: 다운캐스트가 꼭 필요하면 `Any`를 슈퍼트레이트로 두고 업캐스트한다
//!
//! 트레이트 객체 업캐스트(1.86.0 안정화)의 현재 동작을 확인합니다.
//! - 예전 모양: 구현마다 `as_any` 우회 메서드를 적어 `&dyn Any`를 얻었습니다.
//! - 지금 모양: `trait Lesson: Any`라면 `&dyn Lesson`을 `&dyn Any`로 바로 바꿀 수 있습니다.
//! - 업캐스트는 슈퍼트레이트 쪽으로만, 한 방향으로만 일어납니다.
//!
//! 실행: cargo run -p er-ch19-avoid-reflection --example 04_upcast_to_any
//!
//! `TypeId`의 값은 실행마다 같다는 보장이 없으므로 출력하지 않고 비교 결과만 출력합니다.

use std::any::{Any, TypeId};

struct Video {
    title: String,
}

struct Quiz {
    title: String,
    questions: u32,
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

// ---------------------------------------------------------------
// 예전 모양: as_any 우회 메서드 (업캐스트가 안정화되기 전)
// ---------------------------------------------------------------
mod before_upcast {
    use super::*;

    pub trait Lesson {
        fn title(&self) -> &str;
        /// 트레이트 객체를 `&dyn Any`로 바꿀 방법이 없어서, 구현마다 이 한 줄을 적어야 했습니다.
        fn as_any(&self) -> &dyn Any;
    }

    impl Lesson for Video {
        fn title(&self) -> &str {
            &self.title
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl Lesson for Quiz {
        fn title(&self) -> &str {
            &self.title
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}

// ---------------------------------------------------------------
// 지금 모양: Any를 슈퍼트레이트로 두고 업캐스트
// ---------------------------------------------------------------

/// `Any`가 슈퍼트레이트이므로 `dyn Lesson`의 vtable에서 `dyn Any`의 vtable을 찾을 수 있습니다.
trait Lesson: Any {
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

/// 업캐스트는 여러 단계도 됩니다: GradedLesson → Lesson → Any
trait Gradable {
    fn max_points(&self) -> u32;
}
impl Gradable for Quiz {
    fn max_points(&self) -> u32 {
        self.questions * 10
    }
}
trait GradedLesson: Lesson + Gradable {}
impl<T: Lesson + Gradable> GradedLesson for T {}

/// 구체 타입 이름을 얻는 데 다운캐스트를 씁니다(설명용).
fn kind(any: &dyn Any) -> &'static str {
    if any.is::<Quiz>() {
        "Quiz"
    } else if any.is::<Video>() {
        "Video"
    } else {
        "모름"
    }
}

fn main() {
    println!("1. 예전 모양: 구현마다 as_any()를 적음");
    let old: Vec<Box<dyn before_upcast::Lesson>> = vec![Box::new(video()), Box::new(quiz())];
    for item in &old {
        println!("    {} → {}", item.title(), kind(item.as_any()));
    }
    // 출력:
    //     1주차 강의 영상 → Video
    //     1주차 퀴즈 → Quiz

    println!("2. 지금 모양: &dyn Lesson → &dyn Any 업캐스트");
    let course: Vec<Box<dyn Lesson>> = vec![Box::new(video()), Box::new(quiz())];
    for item in &course {
        let lesson: &dyn Lesson = item.as_ref();
        let any: &dyn Any = lesson; // 강제 변환. 우회 메서드가 필요 없습니다.
        let questions = any.downcast_ref::<Quiz>().map(|q| q.questions);
        println!(
            "    {} → {}, 문항 수 {questions:?}",
            lesson.title(),
            kind(any)
        );
    }
    // 출력:
    //     1주차 강의 영상 → Video, 문항 수 None
    //     1주차 퀴즈 → Quiz, 문항 수 Some(10)

    println!("3. 여러 단계 업캐스트: &dyn GradedLesson → &dyn Any");
    let graded: &dyn GradedLesson = &quiz();
    let any: &dyn Any = graded;
    println!(
        "    {} (만점 {}) → {}",
        graded.title(),
        graded.max_points(),
        kind(any)
    );
    // 출력:     1주차 퀴즈 (만점 100) → Quiz

    println!("4. Box째 올리고 downcast로 소유권째 꺼내기");
    let boxed: Box<dyn Lesson> = Box::new(video());
    let as_any: Box<dyn Any> = boxed; // Box<dyn Lesson> → Box<dyn Any>
    match as_any.downcast::<Quiz>() {
        Ok(q) => println!("    Quiz를 꺼냄: {}", q.title),
        Err(returned) => {
            // 돌려받은 상자는 Box<dyn Any>입니다. 업캐스트는 한 방향이라
            // 다시 Box<dyn Lesson>으로 내려갈 수 없고, title()도 부를 수 없습니다.
            println!("    Quiz가 아니라 Box<dyn Any>로 돌려받음");
            if let Ok(v) = returned.downcast::<Video>() {
                println!("    Video로 다시 시도해 꺼냄: {}", v.title);
            }
        }
    }
    // 출력:
    //     Quiz가 아니라 Box<dyn Any>로 돌려받음
    //     Video로 다시 시도해 꺼냄: 1주차 강의 영상

    println!("5. type_id(): 무엇에 부르느냐에 따라 답이 다름");
    let boxed: Box<dyn Lesson> = Box::new(quiz());
    let lesson: &dyn Lesson = boxed.as_ref();
    // dyn Lesson은 Any의 하위 트레이트 객체라서 안에 든 구체 타입의 TypeId를 돌려줍니다.
    let via_ref = lesson.type_id() == TypeId::of::<Quiz>();
    // Box<dyn Lesson> 자체도 'static 타입이라 Any입니다. 메서드 탐색이 상자의 구현을 먼저 찾습니다.
    #[expect(
        clippy::type_id_on_box,
        reason = "상자 자체의 TypeId가 나오는 함정을 보이려고 일부러 부름"
    )]
    let via_box = boxed.type_id() == TypeId::of::<Quiz>();
    println!("    &dyn Lesson에서: Quiz와 같은가 {via_ref}");
    println!("    Box<dyn Lesson>에서: Quiz와 같은가 {via_box}");
    // 출력:
    //     &dyn Lesson에서: Quiz와 같은가 true
    //     Box<dyn Lesson>에서: Quiz와 같은가 false
}
