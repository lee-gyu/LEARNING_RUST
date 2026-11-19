// 기대 에러: E0310 — the parameter type `T` may not live long enough
//
// 제네릭 함수 안에서 "T가 Quiz일 때만" 다르게 처리하려고 `TypeId::of::<T>()`를 불렀습니다.
// `TypeId::of`는 `T: 'static`을 요구합니다. 라이프타임은 `TypeId`에 들어가지 않으므로,
// 빌린 참조를 품은 타입까지 허락하면 다운캐스트로 라이프타임을 속일 수 있기 때문입니다.
// 이 함수의 `T`에는 그런 조건이 없으므로 컴파일러가 거부합니다.
//
// 확인: rustc --edition 2024 compile_fail/e0310_type_id_in_generic.rs --out-dir <임시 디렉터리>

use std::any::TypeId;

trait Lesson {
    fn title(&self) -> &str;
}

struct Quiz {
    title: String,
}

impl Lesson for Quiz {
    fn title(&self) -> &str {
        &self.title
    }
}

/// 퀴즈만 제목 앞에 "[퀴즈]"를 붙이고 싶습니다.
fn label<T: Lesson>(item: &T) -> String {
    if TypeId::of::<T>() == TypeId::of::<Quiz>() {
        // E0310
        format!("[퀴즈] {}", item.title())
    } else {
        item.title().to_string()
    }
}

fn main() {
    let quiz = Quiz {
        title: "1주차 퀴즈".to_string(),
    };
    println!("{}", label(&quiz));
}
