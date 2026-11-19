// 트레이트 두 개를 함께 요구하는 함수의 제네릭판과 `dyn`판을 비교합니다.
// 제네릭판은 바운드를 `+`로 이어 쓰면 끝나지만, 트레이트 객체는 vtable을 트레이트 하나로만
// 만들 수 있어서 둘을 묶은 조합 트레이트와 포괄 구현을 따로 만들어야 합니다.
// 슈퍼트레이트는 상속이 아니라 "이것도 구현한다"는 조건이라는 것도 확인합니다.
//
// 실행: cargo run -p er-ch12-generics-vs-trait-objects --example 03_combined_bounds

use std::fmt::Debug;

trait Loanable {
    fn title(&self) -> &str;
    fn loan_days(&self) -> u32;
}

#[derive(Debug, Clone)]
struct Book {
    title: String,
    shelf: u32,
}

#[derive(Debug, Clone)]
struct Dvd {
    title: String,
    minutes: u32,
}

impl Loanable for Book {
    fn title(&self) -> &str {
        &self.title
    }
    fn loan_days(&self) -> u32 {
        14
    }
}

impl Loanable for Dvd {
    fn title(&self) -> &str {
        &self.title
    }
    fn loan_days(&self) -> u32 {
        7
    }
}

// ── 제네릭판: 필요한 바운드를 `+`로 이어 씁니다 ──────────────────────

/// 대출 기간(`Loanable`)과 디버그 형식(`Debug`)을 함께 씁니다.
fn audit_line<T: Loanable + Debug>(item: &T) -> String {
    format!("{}일 | {:?}", item.loan_days(), item)
}

/// 조합을 하나 더 만들어도 바운드만 바꾸면 됩니다. `Clone`까지 요구해 대출 시점의 사본을 남깁니다.
/// 이 조합은 `dyn`판으로는 아예 만들 수 없습니다(README 4.2절, E0038).
fn snapshot<T: Loanable + Clone>(item: &T, history: &mut Vec<T>) -> usize {
    history.push(item.clone());
    history.len()
}

// ── dyn판: `dyn Loanable + Debug`는 E0225이므로 조합 트레이트를 만듭니다 ──

/// 메서드가 없는 조합 트레이트. 두 트레이트를 슈퍼트레이트로 둡니다.
trait AuditItem: Loanable + Debug {}

/// 포괄 구현: 두 트레이트를 구현한 타입은 모두 자동으로 `AuditItem`입니다.
/// 그래서 `Book`과 `Dvd`에 `impl AuditItem`을 따로 쓰지 않습니다.
impl<T: Loanable + Debug> AuditItem for T {}

fn audit_line_dyn(item: &dyn AuditItem) -> String {
    // vtable 하나에 두 슈퍼트레이트의 메서드가 모두 들어 있어서 둘 다 부를 수 있습니다.
    format!("{}일 | {:?}", item.loan_days(), item)
}

/// `Loanable`만 필요한 함수. 1.86.0부터 `&dyn AuditItem`을 그대로 넘길 수 있습니다(업캐스트).
fn due_days(item: &dyn Loanable) -> u32 {
    item.loan_days()
}

fn main() {
    let book = Book {
        title: "러스트 입문".to_string(),
        shelf: 3,
    };
    let dvd = Dvd {
        title: "우주 다큐".to_string(),
        minutes: 95,
    };
    println!(
        "자료: {}(서가 {}), {}({}분)",
        book.title(),
        book.shelf,
        dvd.title(),
        dvd.minutes
    );
    // 출력: 자료: 러스트 입문(서가 3), 우주 다큐(95분)

    println!("1. 제네릭판: T: Loanable + Debug");
    println!("    {}", audit_line(&book));
    // 출력:     14일 | Book { title: "러스트 입문", shelf: 3 }
    println!("    {}", audit_line(&dvd));
    // 출력:     7일 | Dvd { title: "우주 다큐", minutes: 95 }
    let mut history = Vec::new();
    snapshot(&book, &mut history);
    let kept = snapshot(&book, &mut history);
    println!("    T: Loanable + Clone 으로 남긴 사본: {kept}개");
    // 출력:     T: Loanable + Clone 으로 남긴 사본: 2개

    println!("2. dyn판: 조합 트레이트 AuditItem + 포괄 구현");
    // 조합 트레이트 덕분에 서로 다른 타입을 한 배열에 담고 두 트레이트의 기능을 함께 씁니다.
    // `Book`과 `Dvd`에는 `impl AuditItem`이 없습니다. 슈퍼트레이트는 구현을 물려주는 상속이
    // 아니라 "이것도 구현한다"는 조건이고, 포괄 구현이 그 조건만 보고 붙여 준 것입니다.
    let cart: [&dyn AuditItem; 2] = [&book, &dvd];
    for item in cart {
        println!("    {}", audit_line_dyn(item));
    }
    // 출력:     14일 | Book { title: "러스트 입문", shelf: 3 }
    // 출력:     7일 | Dvd { title: "우주 다큐", minutes: 95 }

    println!("3. 업캐스트: &dyn AuditItem을 슈퍼트레이트 자리에 넘깁니다");
    // `AuditItem`의 vtable에는 `Loanable`의 메서드도 들어 있으므로, 1.86.0부터는
    // `&dyn AuditItem`을 `&dyn Loanable` 자리에 넘기면 강제 변환이 일어납니다.
    // 책(1.70 기준)은 이 변환이 안 된다고 적었습니다.
    let total: u32 = cart.iter().map(|item| due_days(*item)).sum();
    println!("    &dyn AuditItem → &dyn Loanable 로 넘겨 더한 대출 기간: {total}일");
    // 출력:     &dyn AuditItem → &dyn Loanable 로 넘겨 더한 대출 기간: 21일
    let boxed: Box<dyn AuditItem> = Box::new(dvd.clone());
    let narrowed: Box<dyn Loanable> = boxed;
    println!(
        "    Box<dyn AuditItem> → Box<dyn Loanable>: {}",
        narrowed.title()
    );
    // 출력:     Box<dyn AuditItem> → Box<dyn Loanable>: 우주 다큐
}
