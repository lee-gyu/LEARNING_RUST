// 2015 시절 문법과 지금 문법을 나란히: try!, 익명 매개변수, dyn 없는 트레이트 객체, ... 패턴, 원시 포인터
//
// 실행: cargo run -p p2-ch2-rust-2015-2018 --example 03_then_and_now
//
// 각 구역의 주석에 2015 에디션의 모양을 적고, 코드는 지금(2024 에디션)의 모양으로 썼습니다.
// 2015 모양 그대로는 이 파일에서 컴파일되지 않으므로 주석으로만 남겼습니다.
// 실제로 2015 에디션으로 컴파일되는 코드는 projects/scoreboard/app2015에 있습니다.

use std::num::ParseIntError;

// ── 1. try!(식) → 식? ──────────────────────────────────────────────
// 2015:  let n = try!(text.parse::<i32>());
// 2018 이후 try는 예약 키워드라서 try!(..)는 에러입니다. 표준 라이브러리의 try! 매크로는
// 지금도 있으므로 raw 식별자로 r#try!라고 부를 수는 있지만, deprecated 경고가 납니다.
#[expect(
    deprecated,
    reason = "옛 try! 매크로가 ?와 같은 일을 한다는 것을 보이려고 일부러 씁니다"
)]
fn double_old(text: &str) -> Result<i32, ParseIntError> {
    let n = r#try!(text.parse::<i32>());
    Ok(n * 2)
}

fn double_new(text: &str) -> Result<i32, ParseIntError> {
    let n = text.parse::<i32>()?;
    Ok(n * 2)
}

// ── 2. 익명 매개변수 → 이름 또는 _ ────────────────────────────────
// 2015:  trait Greeter { fn greet(&self, &str) -> String; }
// 2018부터 트레이트 메서드 선언의 매개변수에도 이름(패턴)이 있어야 합니다.
trait Greeter {
    fn greet(&self, name: &str) -> String;

    // 이름을 지을 필요가 없으면 _를 씁니다.
    fn volume(&self, _: u8) -> &'static str {
        "보통"
    }

    // 2018부터 본문이 있는 트레이트 메서드는 매개변수 자리에 반박 불가능한 패턴을 쓸 수 있습니다.
    // 2015에서는 본문이 있어도 E0642로 거절됐습니다(본문이 없는 선언에서는 지금도 E0642).
    fn add(&self, (a, b): (i32, i32)) -> i32 {
        a + b
    }
}

struct Korean;
struct English;

impl Greeter for Korean {
    fn greet(&self, name: &str) -> String {
        format!("안녕하세요, {name}님")
    }
}

impl Greeter for English {
    fn greet(&self, name: &str) -> String {
        format!("Hello, {name}")
    }
}

// ── 3. Box<Trait> → Box<dyn Trait>, &Trait → &dyn Trait ─────────────
// 2015:  fn pick(korean: bool) -> Box<Greeter> { .. }
//        fn greet_all(gs: &[Box<Greeter>]) { .. }
// 2018에서는 경고(bare_trait_objects), 2021부터는 에러(E0782)입니다.
// dyn이 붙으면 "실행 중에 vtable로 고르는 타입"이라는 것이 눈에 보이고, impl Trait와 짝이 맞습니다.
fn pick(korean: bool) -> Box<dyn Greeter> {
    if korean {
        Box::new(Korean)
    } else {
        Box::new(English)
    }
}

// 정적 디스패치: 호출마다 구체 타입이 정해집니다(Part 0 ch17).
fn greet_static(g: &impl Greeter) -> String {
    g.greet("민수")
}

// 동적 디스패치: 트레이트 객체를 받습니다.
fn greet_dynamic(g: &dyn Greeter) -> String {
    g.greet("민수")
}

// ── 4. a...b 범위 패턴 → a..=b ────────────────────────────────────
// 2015:  match score { 90...100 => 'A', .. }
// 식에서 끝을 포함하는 범위는 처음부터 ..=였습니다(...는 식에서 쓸 수 없었습니다).
// 패턴의 ...는 ..와 헷갈리기 쉽고 식의 문법과도 달라서 ..=로 통일했습니다.
// 2018에서는 경고(ellipsis_inclusive_range_patterns), 2021부터는 에러(E0783)입니다.
fn grade(score: u32) -> char {
    match score {
        90..=100 => 'A',
        80..=89 => 'B',
        _ => 'F',
    }
}

fn main() {
    println!("1. try!(식) → 식?");
    println!("    r#try! 로: {:?}", double_old("21")); // 출력:     r#try! 로: Ok(42)
    println!("    ?      로: {:?}", double_new("21")); // 출력:     ?      로: Ok(42)
    println!(
        "    실패도 같음: {}",
        double_old("x").is_err() && double_new("x").is_err()
    ); // 출력:     실패도 같음: true

    println!("2. 트레이트 메서드의 매개변수에는 이름이나 패턴");
    println!("    greet  = {}", Korean.greet("지영")); // 출력:     greet  = 안녕하세요, 지영님
    println!("    volume = {}", Korean.volume(7)); // 출력:     volume = 보통
    println!("    add    = {}", Korean.add((2, 3))); // 출력:     add    = 5

    println!("3. 트레이트 객체에는 dyn");
    for korean in [true, false] {
        let g = pick(korean);
        println!("    {}", greet_dynamic(g.as_ref()));
    }
    // 출력:     안녕하세요, 민수님
    // 출력:     Hello, 민수
    println!("    impl Greeter로: {}", greet_static(&English)); // 출력:     impl Greeter로: Hello, 민수

    println!("4. 범위 패턴은 ..=");
    let grades: String = [95, 85, 40].iter().map(|&s| grade(s)).collect();
    println!("    95, 85, 40 → {grades}"); // 출력:     95, 85, 40 → ABF
    let inclusive: Vec<u32> = (1..=3).collect();
    println!("    식의 1..=3 → {inclusive:?}"); // 출력:     식의 1..=3 → [1, 2, 3]

    println!("5. 가리키는 타입을 모르는 원시 포인터에 메서드 호출");
    // 2015:  let p = value as *mut _;  p.is_null()
    // `_`가 무엇인지 정해지기 전에 메서드를 부르는 코드입니다. 2015에서는 경고
    // (tyvar_behind_raw_pointer)였고 2018부터 에러(E0282)입니다. 가리키는 타입을 적어 줍니다.
    let mut value = 10;
    let p: *mut i32 = &mut value;
    println!("    p.is_null() = {}", p.is_null()); // 출력:     p.is_null() = false
}
