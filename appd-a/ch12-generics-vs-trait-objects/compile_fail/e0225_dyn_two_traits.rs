// 기대하는 에러: E0225 — only auto traits can be used as additional traits in a trait object
// 핵심 메시지: help: consider creating a new trait with all of these as supertraits and using that
//             trait here instead: `trait NewTrait: Loanable + Debug {}`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0225_dyn_two_traits.rs --out-dir /tmp/er-ch12
//
// 제네릭에서는 `T: Loanable + Debug`처럼 바운드를 `+`로 이어 쓸 수 있습니다. 같은 모양을
// 트레이트 객체에 쓰면 거부됩니다. 트레이트 객체의 vtable은 트레이트 하나를 기준으로 만들어지므로,
// `+` 뒤에는 메서드가 없는 자동 트레이트(`Send`, `Sync` 등)만 올 수 있습니다.
// 해결: 두 트레이트를 슈퍼트레이트로 둔 조합 트레이트를 만들고 포괄 구현을 붙입니다
//       (examples/03_combined_bounds.rs의 `AuditItem`). 한 가지 타입만 다룬다면
//       제네릭 `fn audit_line<T: Loanable + Debug>(item: &T)`로 받습니다.

use std::fmt::Debug;

trait Loanable {
    fn loan_days(&self) -> u32;
}

#[derive(Debug)]
struct Book {
    title: String,
}

impl Loanable for Book {
    fn loan_days(&self) -> u32 {
        14
    }
}

// E0225: 트레이트 객체에 자동 트레이트가 아닌 트레이트를 둘 적음
fn audit_line_dyn(item: &(dyn Loanable + Debug)) -> String {
    format!("{}일 | {:?}", item.loan_days(), item)
}

fn main() {
    let book = Book {
        title: "러스트 입문".to_string(),
    };
    println!("{} ({})", audit_line_dyn(&book), book.title);
}
