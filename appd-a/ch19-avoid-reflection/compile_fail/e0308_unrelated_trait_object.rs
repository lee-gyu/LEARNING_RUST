// 기대 에러: E0308 — mismatched types: expected trait `Gradable`, found trait `Lesson`
//
// `Quiz`는 `Lesson`과 `Gradable`을 모두 구현하지만, 일단 `&dyn Lesson`이 되면
// 컴파일러가 아는 것은 "`Lesson`을 구현한 무언가"뿐입니다. `Gradable`은 `Lesson`의
// 슈퍼트레이트가 아니므로 업캐스트할 수 없고, "이 값이 `Gradable`도 구현했는가"를
// 실행 중에 물어볼 수단도 없습니다(`Any`도 구체 타입만 확인합니다).
//
// 확인: rustc --edition 2024 compile_fail/e0308_unrelated_trait_object.rs --out-dir <임시 디렉터리>

trait Lesson {
    fn title(&self) -> &str;
}

trait Gradable {
    fn max_points(&self) -> u32;
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

impl Gradable for Quiz {
    fn max_points(&self) -> u32 {
        self.questions * 10
    }
}

fn points_of(item: &dyn Lesson) -> u32 {
    let gradable: &dyn Gradable = item; // E0308
    gradable.max_points()
}

fn main() {
    let quiz = Quiz {
        title: "1주차 퀴즈".to_string(),
        questions: 10,
    };
    println!("{}: {}", quiz.title(), points_of(&quiz));
}
