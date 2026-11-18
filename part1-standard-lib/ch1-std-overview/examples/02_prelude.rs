// 이 예제가 보여 주는 것: 프렐류드(prelude)에 들어 있어 `use` 없이 쓸 수 있는 것과,
// 프렐류드에 없어서 `use`나 전체 경로가 필요한 것
// 실행: cargo run -p p1-ch1-std-overview --example 02_prelude

// 프렐류드에 없는 항목은 이렇게 가져옵니다.
use std::cmp::Ordering;
use std::collections::HashMap;
use std::mem;

fn main() {
    // ── 1. 프렐류드의 타입과 배리언트 ─────────────────────────────────────
    // `Option`/`Some`/`None`, `Result`/`Ok`/`Err`, `Vec`, `String`, `Box`는 `use` 없이 씁니다.
    let maybe: Option<i32> = Some(3);
    let parsed: Result<i32, String> = Err(String::from("숫자가 아님"));
    let boxed: Box<i32> = Box::new(7);
    let list: Vec<i32> = Vec::new();
    println!("{maybe:?} {parsed:?} {boxed} {list:?}"); // 출력: Some(3) Err("숫자가 아님") 7 []

    // ── 2. 프렐류드의 트레이트: 메서드를 바로 부를 수 있는 이유 ────────────────
    // 트레이트의 메서드는 그 트레이트가 스코프에 있어야 부를 수 있습니다(Part 0 ch10).
    // `Clone`, `ToString`, `Iterator`, `PartialOrd`, `Default`가 프렐류드에 있으므로 따로 가져오지 않습니다.
    let text = 42.to_string(); // ToString
    let copy = text.clone(); // Clone
    let total: i32 = [1, 2, 3].iter().sum(); // Iterator
    let zero = i32::default(); // Default
    println!("{text} {copy} {total} {zero} {}", 1 < 2); // 출력: 42 42 6 0 true

    // 2021 에디션부터 `TryFrom`/`TryInto`도 프렐류드에 들어 있습니다.
    // `u8::try_from(300)`은 범위를 넘으므로 `Err`입니다. (변환 트레이트 자체는 `ch6-core-traits`에서 다룹니다.)
    println!("{:?} {:?}", u8::try_from(200), u8::try_from(300)); // 출력: Ok(200) Err(TryFromIntError(PosOverflow))

    // ── 3. 프렐류드의 함수: `std::mem` 안의 일부만 들어 있습니다 ───────────────
    // `drop`, `size_of`, `size_of_val`, `align_of`, `align_of_val`은 프렐류드에 있습니다.
    println!("size_of::<u64>() = {}", size_of::<u64>()); // 출력: size_of::<u64>() = 8
    drop(copy); // `std::mem::drop`. 이 뒤로 `copy`는 쓸 수 없습니다.

    // 같은 모듈의 `swap`, `replace`, `take`는 프렐류드에 없으므로 경로를 적어야 합니다.
    // `use std::mem;`으로 모듈을 가져와 `mem::swap`처럼 쓰는 것이 관례입니다(그냥 `swap`이라고 쓰면 E0425).
    let mut a = 1;
    let mut b = 2;
    mem::swap(&mut a, &mut b);
    println!("swap 뒤: a = {a}, b = {b}"); // 출력: swap 뒤: a = 2, b = 1

    // ── 4. 매크로 ────────────────────────────────────────────────────────
    // `println!`, `vec!`, `format!`, `assert_eq!` 같은 표준 매크로도 `use` 없이 씁니다.
    let v = vec![1, 2, 3];
    assert_eq!(v.len(), 3);
    let shown = format!("{v:?}");
    println!("{shown}"); // 출력: [1, 2, 3]

    // ── 5. 프렐류드에 없는 타입 ─────────────────────────────────────────────
    // `HashMap`과 `Ordering`은 프렐류드에 없어서 파일 위쪽에서 `use`로 가져왔습니다.
    let mut ages = HashMap::new();
    ages.insert("페리스", 7);
    println!("{:?}", ages.get("페리스")); // 출력: Some(7)

    // `cmp`는 프렐류드의 `Ord` 트레이트 메서드라 바로 부를 수 있지만,
    // 돌려받은 `Ordering`의 배리언트 이름을 쓰려면 `Ordering`을 가져와야 합니다.
    match 3.cmp(&5) {
        Ordering::Less => println!("3 < 5"), // 출력: 3 < 5
        Ordering::Equal => println!("3 == 5"),
        Ordering::Greater => println!("3 > 5"),
    }

    // ── 6. `use` 없이 전체 경로로 쓰기 ──────────────────────────────────────
    // `std`와 `core`라는 크레이트 이름 자체는 어디서나 쓸 수 있으므로, 한 번만 쓸 항목은 경로로 적어도 됩니다.
    let set = std::collections::BTreeSet::from([3, 1, 2]);
    println!("{set:?}"); // 출력: {1, 2, 3}
}
