// 같은 이름의 메서드·연관 함수를 구분하는 완전 정규화 문법(<Type as Trait>::function)과
// 다른 트레이트를 전제로 하는 슈퍼트레이트를 보여 줍니다.
//
// 실행: cargo run -p p0-ch19-advanced-features --example 04_qualified_syntax_supertraits

use std::fmt;

// 1. 이름이 같은 메서드 세 개: 트레이트 두 개와 타입 자신의 메서드
trait Pilot {
    fn fly(&self) -> String;
}

trait Wizard {
    fn fly(&self) -> String;
}

struct Human;

impl Pilot for Human {
    fn fly(&self) -> String {
        String::from("기장입니다. 곧 이륙하겠습니다.")
    }
}

impl Wizard for Human {
    fn fly(&self) -> String {
        String::from("날아올라라!")
    }
}

impl Human {
    fn fly(&self) -> String {
        String::from("*팔을 힘차게 휘젓는다*")
    }
}

// 2. self가 없는 연관 함수는 "어느 타입의 구현인가"를 인자로 알 수 없습니다.
trait Animal {
    fn baby_name() -> String;
}

struct Dog;

impl Dog {
    fn baby_name() -> String {
        String::from("바둑이") // 타입 자신의 연관 함수: 이 개의 이름
    }
}

impl Animal for Dog {
    fn baby_name() -> String {
        String::from("강아지") // Animal 트레이트의 구현: 새끼 개를 부르는 말
    }
}

// 3. 슈퍼트레이트: OutlinePrint를 구현하려면 Display도 구현해야 합니다.
trait OutlinePrint: fmt::Display {
    fn outline_print(&self) {
        // Display가 전제되어 있으므로 to_string()을 쓸 수 있습니다(ch10의 포괄 구현).
        let output = self.to_string();
        let len = output.len(); // 바이트 길이라서 ASCII 문자열일 때만 상자가 맞습니다.
        println!("    {}", "*".repeat(len + 4));
        println!("    *{}*", " ".repeat(len + 2));
        println!("    * {output} *");
        println!("    *{}*", " ".repeat(len + 2));
        println!("    {}", "*".repeat(len + 4));
    }
}

struct Point {
    x: i32,
    y: i32,
}

// Display 구현을 지우면 아래 impl OutlinePrint for Point에서 E0277이 납니다.
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl OutlinePrint for Point {} // 기본 구현을 그대로 씁니다.

// 바운드에 OutlinePrint만 적어도 Display의 기능({} 출력)을 쓸 수 있습니다.
fn describe<T: OutlinePrint>(item: &T) -> String {
    format!("{item}을(를) 상자에 담았습니다")
}

fn main() {
    println!("1. 같은 이름의 메서드");
    let person = Human;
    println!("    person.fly()         → {}", person.fly()); // 출력:     person.fly()         → *팔을 힘차게 휘젓는다*
    println!("    Pilot::fly(&person)  → {}", Pilot::fly(&person)); // 출력:     Pilot::fly(&person)  → 기장입니다. 곧 이륙하겠습니다.
    println!("    Wizard::fly(&person) → {}", Wizard::fly(&person)); // 출력:     Wizard::fly(&person) → 날아올라라!
    // 완전 정규화 문법으로 써도 같습니다. 메서드는 &person으로 타입을 알 수 있어 보통 생략합니다.
    println!(
        "    <Human as Wizard>::fly(&person) → {}",
        <Human as Wizard>::fly(&person)
    ); // 출력:     <Human as Wizard>::fly(&person) → 날아올라라!

    println!("2. 같은 이름의 연관 함수");
    println!("    Dog::baby_name()             → {}", Dog::baby_name()); // 출력:     Dog::baby_name()             → 바둑이
    // Animal::baby_name()은 어느 타입의 구현인지 알 수 없어 E0790입니다.
    println!(
        "    <Dog as Animal>::baby_name() → {}",
        <Dog as Animal>::baby_name()
    ); // 출력:     <Dog as Animal>::baby_name() → 강아지

    println!("3. 슈퍼트레이트");
    let p = Point { x: 1, y: 3 };
    p.outline_print();
    // 출력:
    //     **********
    //     *        *
    //     * (1, 3) *
    //     *        *
    //     **********
    println!("    {}", describe(&p)); // 출력:     (1, 3)을(를) 상자에 담았습니다
}
