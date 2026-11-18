// 이 예제가 보여 주는 것: 제자리에서 꺼내고 채우기 — take, replace, take_if, insert, get_or_insert_with
// 실행: cargo run -p p1-ch2-option-result --example 06_in_place
//
// 이 메서드들은 self가 아니라 &mut self를 받습니다. Option을 소비하지 않고, 변수나 구조체 필드에
// 그대로 둔 채 안의 값만 꺼내거나 바꿉니다. 구조체의 Option 필드를 &mut self 메서드에서 다룰 때 특히 필요합니다.

// 사물함 한 칸. 물건이 있을 수도, 없을 수도 있습니다.
struct Locker {
    item: Option<String>,
}

impl Locker {
    // take: 안의 값을 꺼내 돌려주고, 그 자리에는 None을 남깁니다.
    // 시그니처: fn take(&mut self) -> Option<T>
    // `self.item`을 그냥 돌려주면 E0507입니다. &mut self는 빌린 것이라 필드를 밖으로 옮길 수 없는데,
    // 옮기면 구조체에 "비어 있는 구멍"이 생기기 때문입니다. take는 None을 채워 넣어 구멍을 막습니다.
    fn pick_up(&mut self) -> Option<String> {
        self.item.take()
    }

    // replace: 새 값을 넣고, 예전 값(있었다면)을 돌려줍니다.
    // 시그니처: fn replace(&mut self, value: T) -> Option<T>
    fn swap_in(&mut self, new_item: &str) -> Option<String> {
        self.item.replace(String::from(new_item))
    }
}

// 요약 문자열을 처음 요청할 때 한 번만 만들어 두는 보고서입니다.
struct Report {
    scores: Vec<i32>,
    summary: Option<String>,
}

impl Report {
    // get_or_insert_with: None이면 클로저로 값을 만들어 넣고, 어느 쪽이든 안의 값에 대한 &mut T를 돌려줍니다.
    // 시그니처: fn get_or_insert_with<F>(&mut self, f: F) -> &mut T where F: FnOnce() -> T
    // 돌려받은 &mut String은 반환 타입 &str로 자동 변환됩니다(역참조 강제 변환, Part 0 ch14).
    fn summary(&mut self) -> &str {
        self.summary.get_or_insert_with(|| {
            println!("  (요약을 계산합니다)");
            let total: i32 = self.scores.iter().sum();
            format!("{}명, 합계 {total}점", self.scores.len())
        })
    }
}

fn main() {
    println!("== 1. take: 꺼내고 None 남기기 ==");
    let mut locker = Locker {
        item: Some(String::from("우산")),
    };
    let first = locker.pick_up();
    let second = locker.pick_up();
    println!(
        "첫 번째: {first:?}, 두 번째: {second:?}, 사물함: {:?}",
        locker.item
    );
    // 출력: 첫 번째: Some("우산"), 두 번째: None, 사물함: None

    println!("\n== 2. replace: 바꿔 넣고 예전 값 받기 ==");
    let old = locker.swap_in("가방");
    println!("예전 물건: {old:?}, 사물함: {:?}", locker.item);
    let old = locker.swap_in("책");
    println!("예전 물건: {old:?}, 사물함: {:?}", locker.item);
    // 출력: 예전 물건: None, 사물함: Some("가방")
    // 출력: 예전 물건: Some("가방"), 사물함: Some("책")

    println!("\n== 3. take_if: 조건이 맞을 때만 꺼내기 ==");
    // 시그니처: fn take_if<P>(&mut self, predicate: P) -> Option<T> where P: FnOnce(&mut T) -> bool
    let taken = locker.item.take_if(|item| item.as_str() == "우산");
    println!("우산이면 꺼냄 → {taken:?}, 사물함: {:?}", locker.item);
    let taken = locker.item.take_if(|item| item.as_str() == "책");
    println!("책이면 꺼냄   → {taken:?}, 사물함: {:?}", locker.item);
    // 출력: 우산이면 꺼냄 → None, 사물함: Some("책")
    // 출력: 책이면 꺼냄   → Some("책"), 사물함: None

    println!("\n== 4. insert: 무조건 넣고 &mut T 받기 ==");
    // 시그니처: fn insert(&mut self, value: T) -> &mut T
    // 넣은 값을 가리키는 가변 참조를 돌려주므로, 넣자마자 이어서 고칠 수 있습니다. 예전 값은 버려집니다.
    let mut retries: Option<u32> = None;
    let count = retries.insert(1);
    *count += 2;
    println!("retries = {retries:?}");
    // 출력: retries = Some(3)

    println!("\n== 5. get_or_insert_with: 없을 때만 만들어 넣기 ==");
    let mut report = Report {
        scores: vec![80, 95, 70],
        summary: None,
    };
    println!("첫 요청: {}", report.summary());
    println!("두 번째 요청: {}", report.summary());
    // 출력:   (요약을 계산합니다)
    // 출력: 첫 요청: 3명, 합계 245점
    // 출력: 두 번째 요청: 3명, 합계 245점
    // 클로저는 summary가 None이었던 첫 요청에서만 실행되었습니다. 이것도 게으른 평가입니다.
}
