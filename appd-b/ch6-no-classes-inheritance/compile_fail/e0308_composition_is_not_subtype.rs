// 기대하는 에러: E0308
// 핵심 메시지: mismatched types — expected `&Employee`, found `&Manager`
//
// Java에서는 `Manager extends Employee`이면 `Employee`를 받는 메서드에 `Manager`를 넘길 수 있습니다(업캐스트).
// 01 예제처럼 상속을 조합으로 옮긴 뒤에도 같은 습관으로 `&Manager`를 `&Employee` 자리에 넘긴 코드입니다.
// `Manager`가 `Employee`를 필드로 품고 있어도 둘은 서로 다른 타입입니다(Rust의 하위 타입 관계는 라이프타임에만 있음).
//
// 고치는 법:
//   1) 품은 값을 직접 넘깁니다: print_name(&lead.base)
//   2) 여러 타입을 받아야 하는 함수라면 트레이트로 받습니다: fn print_name(p: &impl Payable)
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch6-no-classes-inheritance compile_fail/e0308_composition_is_not_subtype.rs

struct Employee {
    name: String,
}

struct Manager {
    base: Employee,
    bonus: u32,
}

fn print_name(employee: &Employee) {
    println!("이름: {}", employee.name);
}

fn main() {
    let lead = Manager {
        base: Employee {
            name: String::from("이두리"),
        },
        bonus: 100,
    };
    print_name(&lead); // E0308
    println!("보너스: {}", lead.bonus);
}
