// 기대 에러: E0599 — no method named `downcast_ref` found for reference `&(dyn Lesson + 'static)`
//
// `Any`를 슈퍼트레이트로 두었으니 `&dyn Lesson`에서 바로 다운캐스트할 수 있을 것 같지만,
// `downcast_ref`·`is`는 `dyn Any`(와 `dyn Any + Send` 등)에만 있는 고유 메서드라서
// 하위 트레이트의 트레이트 객체에는 없습니다. 먼저 `&dyn Any`로 업캐스트해야 합니다.
// (`Any`를 슈퍼트레이트로 두지 않았을 때도 같은 E0599가 나고, 그때는 업캐스트도 할 수 없습니다.)
//
// 확인: rustc --edition 2024 compile_fail/e0599_downcast_on_subtrait_object.rs --out-dir <임시 디렉터리>

use std::any::Any;

trait Lesson: Any {
    fn title(&self) -> &str;
}

struct Quiz {
    title: String,
    questions: u32,
}

impl Lesson for Quiz {
    fn title(&self) -> &str {
        &self.title
    }
}

fn questions_of(item: &dyn Lesson) -> Option<u32> {
    item.downcast_ref::<Quiz>().map(|q| q.questions) // E0599
}

fn main() {
    let quiz = Quiz {
        title: "1주차 퀴즈".to_string(),
        questions: 10,
    };
    println!("{}: {:?}", quiz.title(), questions_of(&quiz));
}
