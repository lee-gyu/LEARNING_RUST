//! 스택 프레임 모형: 함수가 돌려주는 참조는 그 함수의 프레임 밖에 있는 값만 가리킬 수 있습니다.
//!
//! 실행: cargo run -p er-ch14-lifetimes --example 01_stack_frames
//!
//! 함수가 끝나면 그 프레임의 지역 값은 drop되고, 프레임이 쓰던 자리는 다음 호출이 다시 씁니다.
//! 그래서 돌아가는 참조는 (1) 부르는 쪽이 넘겨준 값, (2) 어느 프레임에도 속하지 않는 정적 데이터만
//! 가리킬 수 있습니다. 함수 안에서 만든 값은 참조가 아니라 값 자체를 돌려줘 소유권을 넘깁니다.

/// drop될 때 이름을 출력하는 관찰용 타입입니다.
struct Tracked(&'static str);

impl Drop for Tracked {
    fn drop(&mut self) {
        println!("    [drop] {}", self.0);
    }
}

struct Recipe {
    name: String,
    ingredients: Vec<String>,
}

fn recipe(name: &str, ingredients: &[&str]) -> Recipe {
    Recipe {
        name: name.to_string(),
        ingredients: ingredients.iter().map(|s| s.to_string()).collect(),
    }
}

// 1. 받는 쪽 프레임의 지역 값: 함수가 끝날 때 drop됩니다.
//    이 값을 가리키는 참조를 돌려주려 하면 E0515입니다(Part 0 ch10 4.5절).
fn weigh_with_scale() -> usize {
    let _scale = Tracked("저울(받는 쪽의 지역 값)");
    150 * 2 // usize는 복사되어 나가고, 저울은 이 줄 다음의 `}`에서 drop됩니다.
}

// 2. 함수 안에서 만든 값은 값 그대로 돌려줍니다. 프레임을 떠나 부르는 쪽으로 이동하므로
//    받는 쪽의 `}`에서는 drop되지 않습니다.
fn new_pot() -> Tracked {
    let pot = Tracked("냄비(받는 쪽에서 만들어 돌려준 값)");
    println!("    받는 쪽에서 만듦: {}", pot.0);
    pot
}

// 3. 매개변수에서 얻은 참조: 부르는 쪽이 가진 데이터를 가리키므로 돌려줄 수 있습니다.
//    생략 규칙에 따라 반환값의 라이프타임은 `recipe`의 라이프타임과 같습니다.
fn main_ingredient(recipe: &Recipe) -> &str {
    recipe
        .ingredients
        .first()
        .map_or("(재료 없음)", |s| s.as_str())
}

// 함수 안에서 새로 만든 문자열은 String으로 돌려줍니다. &str로 돌려줄 방법은 없습니다.
fn shopping_line(recipe: &Recipe) -> String {
    format!("{}: {}", recipe.name, recipe.ingredients.join(", "))
}

// 4. 'static 참조: 문자열 리터럴은 실행 파일 안에 들어 있어 어느 프레임에도 속하지 않습니다.
fn unit_label(code: u8) -> &'static str {
    match code {
        0 => "g",
        1 => "ml",
        _ => "개",
    }
}

fn main() {
    println!("1. 받는 쪽의 지역 값은 프레임과 함께 사라집니다");
    let grams = weigh_with_scale();
    println!("    돌아온 값: {grams}g");
    // 출력:
    //     [drop] 저울(받는 쪽의 지역 값)
    //     돌아온 값: 300g

    println!("2. 값을 돌려주면 부르는 쪽으로 이동합니다");
    {
        let pot = new_pot();
        println!("    부르는 쪽이 받은 값: {}", pot.0);
    } // pot의 새 소유자(이 블록)가 끝나는 곳에서 drop됩니다.
    // 출력:
    //     받는 쪽에서 만듦: 냄비(받는 쪽에서 만들어 돌려준 값)
    //     부르는 쪽이 받은 값: 냄비(받는 쪽에서 만들어 돌려준 값)
    //     [drop] 냄비(받는 쪽에서 만들어 돌려준 값)

    println!("3. 매개변수에서 얻은 참조는 부르는 쪽의 값을 가리킵니다");
    let stew = recipe("된장찌개", &["된장", "두부", "애호박"]);
    let key_item = main_ingredient(&stew);
    let line = shopping_line(&stew);
    println!("    주재료: {key_item} / 장보기: {line}");
    // 출력:     주재료: 된장 / 장보기: 된장찌개: 된장, 두부, 애호박

    println!("4. 'static 참조는 어느 프레임에도 속하지 않습니다");
    let label = unit_label(0);
    let _ = weigh_with_scale(); // 다른 함수를 불러 프레임이 바뀌어도 label은 그대로입니다.
    println!("    단위: {label}");
    // 출력:
    //     [drop] 저울(받는 쪽의 지역 값)
    //     단위: g

    println!("5. 힙의 값도 소유 사슬의 끝(스택 변수)이 살아 있는 동안만 삽니다");
    let mut shelf: Vec<Box<Recipe>> = Vec::new();
    {
        let in_box = Box::new(recipe("비빔밥", &["밥", "나물", "고추장"]));
        // in_box(이 블록의 스택 변수) → 힙의 Recipe → 힙의 String들로 소유가 이어집니다.
        shelf.push(in_box); // 사슬의 끝이 블록의 변수에서 바깥의 shelf로 옮겨 갑니다.
    } // in_box는 이미 이동했으므로 여기서 drop되는 것이 없습니다.
    let inside_heap = main_ingredient(&shelf[0]); // shelf가 살아 있는 동안 유효한 참조
    println!("    선반의 첫 레시피 주재료: {inside_heap}");
    // 출력:     선반의 첫 레시피 주재료: 밥
}
