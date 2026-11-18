// 이 예제가 보여 주는 것: Cell<T> — 값을 통째로 넣고 빼는 내부 가변성.
//                        get/set/update(Copy 타입), replace/take/swap/into_inner(모든 타입),
//                        &mut가 있을 때의 get_mut, Rc<Cell<T>>로 Fn 클로저끼리 상태 공유
// 실행: cargo run -p p1-ch7-smart-pointers-memory --example 06_cell
//
// Cell은 안의 값을 가리키는 참조(&T)를 절대 내주지 않습니다. 값을 복사해 주거나(get) 통째로 바꿔 넣을(set,
// replace) 뿐입니다. 안을 가리키는 참조가 없으니 "읽는 중에 바뀌는" 상황이 생길 수 없고, 그래서
// RefCell과 달리 실행 중에 대여 상태를 셀 필요가 없습니다.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

// 단어를 찾을 때마다 조회 수를 세는 사전입니다. lookup은 &self를 받습니다.
// 조회 수는 사전의 "내용"이 아니라 내부 통계이므로, 이것 때문에 &mut self를 요구하고 싶지 않습니다.
struct Dictionary {
    words: Vec<(&'static str, &'static str)>,
    lookups: Cell<u32>,
    misses: Cell<u32>,
}

impl Dictionary {
    fn lookup(&self, word: &str) -> Option<&'static str> {
        self.lookups.set(self.lookups.get() + 1); // get으로 복사해 오고, set으로 새 값을 넣습니다.
        let found = self
            .words
            .iter()
            .find(|(w, _)| *w == word)
            .map(|(_, meaning)| *meaning);
        if found.is_none() {
            self.misses.update(|n| n + 1); // update(1.88.0)는 get과 set을 한 번에 합니다.
        }
        found
    }
}

fn main() {
    println!("== 1. &self 메서드 안에서 세기: get / set / update ==");
    let dict = Dictionary {
        words: vec![("cell", "칸"), ("borrow", "빌리다")],
        lookups: Cell::new(0),
        misses: Cell::new(0),
    };
    // 불변 참조 두 개로 같은 사전을 씁니다. 둘 다 &Dictionary이지만 조회 수는 함께 늘어납니다.
    let reader_a = &dict;
    let reader_b = &dict;
    println!("cell → {:?}", reader_a.lookup("cell")); // 출력: cell → Some("칸")
    println!("rust → {:?}", reader_b.lookup("rust")); // 출력: rust → None
    println!("borrow → {:?}", reader_a.lookup("borrow")); // 출력: borrow → Some("빌리다")
    println!(
        "조회 {}번, 실패 {}번",
        dict.lookups.get(),
        dict.misses.get()
    ); // 출력: 조회 3번, 실패 1번

    println!("\n== 2. get은 복사본을 줍니다 ==");
    let level = Cell::new(1);
    let seen = level.get(); // 이 순간의 값을 복사해 왔습니다. 참조가 아닙니다.
    level.set(5);
    println!("seen = {seen}, level = {}", level.get()); // 출력: seen = 1, level = 5
    // 참조를 내주지 않으니 세는 칸이 필요 없습니다. Cell<u32>는 u32와 크기가 같고, RefCell은 대여 카운트만큼 큽니다.
    println!(
        "u32 {}, Cell<u32> {}, RefCell<u32> {}",
        size_of::<u32>(),
        size_of::<Cell<u32>>(),
        size_of::<RefCell<u32>>()
    ); // 출력: u32 4, Cell<u32> 4, RefCell<u32> 16

    println!("\n== 3. Copy가 아닌 값: replace / take / swap / into_inner ==");
    // String은 Copy가 아니라 get을 쓸 수 없습니다(compile_fail/e0599_cell_get_non_copy.rs).
    // 대신 값을 통째로 바꿔 넣고 예전 값을 돌려받는 메서드를 씁니다.
    let title = Cell::new(String::from("초안"));
    let old = title.replace(String::from("2차 원고"));
    println!("replace가 돌려준 예전 값: {old}"); // 출력: replace가 돌려준 예전 값: 초안
    // take는 값을 꺼내고 Default::default()(String이면 빈 문자열)를 남깁니다.
    let mut current = title.take();
    current.push_str(" (검토 완료)");
    title.set(current); // 꺼내서 고친 뒤 다시 넣는 것이 Cell에서 Copy가 아닌 값을 고치는 기본 모양입니다.

    let tags_a = Cell::new(vec!["긴급"]);
    let tags_b = Cell::new(vec!["보류", "검토"]);
    tags_a.swap(&tags_b); // 두 Cell의 내용을 맞바꿉니다. 둘 다 &로만 빌렸습니다.
    println!("title = {}", title.into_inner()); // 출력: title = 2차 원고 (검토 완료)
    println!(
        "tags_a = {:?}, tags_b = {:?}",
        tags_a.into_inner(),
        tags_b.into_inner()
    ); // 출력: tags_a = ["보류", "검토"], tags_b = ["긴급"]
    // into_inner는 Cell을 소비하고 안의 값을 돌려줍니다.

    println!("\n== 4. &mut가 있으면 get_mut으로 바로 고칩니다 ==");
    let mut score = Cell::new(10);
    *score.get_mut() += 5; // &mut Cell이 있다는 것은 다른 누구도 이 Cell을 보고 있지 않다는 뜻입니다.
    println!("score = {}", score.get()); // 출력: score = 15

    println!("\n== 5. Rc<Cell<T>>: 여러 클로저가 같은 칸을 바꿉니다 ==");
    let clicks = Rc::new(Cell::new(0));
    let on_ok = {
        let clicks = Rc::clone(&clicks);
        move || clicks.update(|n| n + 1)
    };
    let on_cancel = {
        let clicks = Rc::clone(&clicks);
        move || clicks.update(|n| n + 10)
    };
    // update는 &self만 필요하므로 두 클로저는 FnMut가 아니라 Fn입니다. 그래서 let mut 없이 부릅니다.
    on_ok();
    on_ok();
    on_cancel();
    println!(
        "클릭 점수 = {}, strong_count = {}",
        clicks.get(),
        Rc::strong_count(&clicks)
    ); // 출력: 클릭 점수 = 12, strong_count = 3
}
