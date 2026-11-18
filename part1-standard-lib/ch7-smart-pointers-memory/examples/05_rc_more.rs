// 이 예제가 보여 주는 것: Rc를 더 깊이 쓰기 — ptr_eq, Rc<str>/Rc<[T]>, get_mut과 make_mut(쓰기 시 복제),
//                        try_unwrap / into_inner / unwrap_or_clone, weak_count와 new_cyclic
// 실행: cargo run -p p1-ch7-smart-pointers-memory --example 05_rc_more
//
// 주소 값은 실행마다 다르므로 출력하지 않고, 같은 곳을 가리키는지는 Rc::ptr_eq의 결과로 보입니다.

use std::rc::{Rc, Weak};

// 자기 자신을 가리키는 Weak를 품은 위젯입니다. &self만 있어도 자신의 Rc를 다시 얻을 수 있습니다.
struct Widget {
    name: String,
    me: Weak<Widget>,
}

impl Widget {
    fn new(name: &str) -> Rc<Widget> {
        // new_cyclic은 할당을 먼저 해 두고, 그곳을 가리키는 Weak를 클로저에 넘겨 줍니다.
        // 클로저가 돌려준 값이 그 자리에 들어가야 Rc가 완성되므로, 클로저 안에서 upgrade하면 None입니다.
        Rc::new_cyclic(|me: &Weak<Widget>| {
            println!(
                "  new_cyclic 안에서 upgrade 성공? {}",
                me.upgrade().is_some()
            );
            Widget {
                name: name.to_string(),
                me: me.clone(),
            }
        })
    }

    // 콜백 목록 등에 자신을 등록할 때처럼, &self에서 Rc<Self>가 필요할 때 씁니다.
    fn handle(&self) -> Rc<Widget> {
        self.me
            .upgrade()
            .expect("Rc로 살아 있는 동안에만 부를 수 있습니다")
    }
}

fn main() {
    println!("== 1. == 는 값을, Rc::ptr_eq는 가리키는 곳을 비교합니다 ==");
    let a = Rc::new(String::from("설정"));
    let b = Rc::clone(&a);
    let c = Rc::new(String::from("설정"));
    println!("a == c: {}, ptr_eq(a, c): {}", a == c, Rc::ptr_eq(&a, &c));
    // 출력: a == c: true, ptr_eq(a, c): false
    println!("a == b: {}, ptr_eq(a, b): {}", a == b, Rc::ptr_eq(&a, &b));
    // 출력: a == b: true, ptr_eq(a, b): true

    println!("\n== 2. Rc<str>, Rc<[T]>: 바뀌지 않는 문자열·슬라이스를 공유하기 ==");
    // Rc<String>은 Rc 할당 안에 String 손잡이가 있고, 글자는 또 다른 할당에 있습니다(두 번 따라감).
    // Rc<str>은 카운트와 글자를 한 할당에 담고, 손잡이는 넓은 포인터(주소 + 길이)입니다.
    let author: Rc<str> = Rc::from("김작가");
    let books: Vec<(&str, Rc<str>)> = vec![
        ("첫 책", Rc::clone(&author)),
        ("둘째 책", Rc::clone(&author)),
    ];
    println!(
        "{} - {}, {} - {}",
        books[0].0, books[0].1, books[1].0, books[1].1
    );
    // 출력: 첫 책 - 김작가, 둘째 책 - 김작가
    println!("author의 strong_count = {}", Rc::strong_count(&author)); // 출력: author의 strong_count = 3
    println!(
        "Rc<String> {}바이트, Rc<str> {}바이트",
        size_of::<Rc<String>>(),
        size_of::<Rc<str>>()
    ); // 출력: Rc<String> 8바이트, Rc<str> 16바이트
    let primes: Rc<[u32]> = vec![2, 3, 5, 7].into();
    let view = Rc::clone(&primes);
    println!("primes = {view:?}, 합 = {}", view.iter().sum::<u32>()); // 출력: primes = [2, 3, 5, 7], 합 = 17

    println!("\n== 3. get_mut과 make_mut: 혼자일 때만 바로 고칩니다 ==");
    let mut config = Rc::new(vec![1, 2, 3]);
    // get_mut은 다른 Rc나 Weak가 하나도 없을 때만 &mut를 줍니다.
    if let Some(v) = Rc::get_mut(&mut config) {
        v.push(4);
    }
    println!("혼자일 때 get_mut → {config:?}"); // 출력: 혼자일 때 get_mut → [1, 2, 3, 4]
    let reader = Rc::clone(&config);
    println!(
        "공유 중 get_mut은 None? {}",
        Rc::get_mut(&mut config).is_none()
    ); // 출력: 공유 중 get_mut은 None? true

    // make_mut은 공유 중이면 안의 값을 복제해 새 할당으로 갈라선 뒤 &mut를 줍니다(쓰기 시 복제).
    Rc::make_mut(&mut config).push(5);
    println!("config = {config:?}, reader = {reader:?}"); // 출력: config = [1, 2, 3, 4, 5], reader = [1, 2, 3, 4]
    println!(
        "갈라선 뒤 ptr_eq: {}, 카운트: {} / {}",
        Rc::ptr_eq(&config, &reader),
        Rc::strong_count(&config),
        Rc::strong_count(&reader)
    ); // 출력: 갈라선 뒤 ptr_eq: false, 카운트: 1 / 1
    // 이제 config는 혼자이므로 make_mut은 복제 없이 제자리에서 고칩니다.
    // Rc::as_ptr는 안의 값을 가리키는 원시 포인터(Part 0 ch19)입니다. 따라가지 않고 비교만 합니다.
    let snapshot = Rc::as_ptr(&config);
    Rc::make_mut(&mut config).push(6);
    println!(
        "혼자일 때 make_mut은 같은 할당? {}",
        Rc::as_ptr(&config) == snapshot
    );
    // 출력: 혼자일 때 make_mut은 같은 할당? true

    // Weak만 남아 있을 때 make_mut은 복제하지 않고 Weak와의 연결을 끊습니다.
    let watcher = Rc::downgrade(&config);
    println!(
        "Weak가 있을 때 get_mut은 None? {}",
        Rc::get_mut(&mut config).is_none()
    ); // 출력: Weak가 있을 때 get_mut은 None? true
    Rc::make_mut(&mut config).push(7);
    println!("make_mut 뒤 watcher.upgrade(): {:?}", watcher.upgrade()); // 출력: make_mut 뒤 watcher.upgrade(): None

    println!("\n== 4. 안의 값을 꺼내기: try_unwrap / into_inner / unwrap_or_clone ==");
    let only = Rc::new(String::from("혼자"));
    println!("try_unwrap(혼자) = {:?}", Rc::try_unwrap(only)); // 출력: try_unwrap(혼자) = Ok("혼자")

    let shared = Rc::new(String::from("공유"));
    let other = Rc::clone(&shared);
    match Rc::try_unwrap(shared) {
        Ok(s) => println!("꺼냄: {s}"),
        // 실패하면 넘겼던 Rc를 Err에 담아 돌려주므로 잃어버리지 않습니다.
        Err(back) => println!("Err: 아직 {}곳이 공유 중", Rc::strong_count(&back)),
    }
    // 출력: Err: 아직 2곳이 공유 중

    // into_inner는 실패하면 None을 돌려주고 그 Rc는 버립니다.
    // 모든 복제본에 into_inner를 부르면 마지막 한 번이 반드시 값을 받습니다.
    let first = Rc::clone(&other);
    println!("into_inner(first) = {:?}", Rc::into_inner(first)); // 출력: into_inner(first) = None
    println!("into_inner(other) = {:?}", Rc::into_inner(other)); // 출력: into_inner(other) = Some("공유")

    // unwrap_or_clone은 혼자면 꺼내고, 공유 중이면 안의 값을 복제해 돌려줍니다.
    let doc = Rc::new(vec!["초안"]);
    let keep = Rc::clone(&doc);
    let mut mine = Rc::unwrap_or_clone(doc); // keep이 있으므로 복제됩니다.
    mine.push("수정");
    println!("mine = {mine:?}, keep = {keep:?}"); // 출력: mine = ["초안", "수정"], keep = ["초안"]

    println!("\n== 5. weak_count와 new_cyclic: 자기 자신을 가리키는 Weak ==");
    let button = Widget::new("확인 버튼");
    // 출력:   new_cyclic 안에서 upgrade 성공? false
    println!(
        "{}: strong {}, weak {}",
        button.name,
        Rc::strong_count(&button),
        Rc::weak_count(&button)
    ); // 출력: 확인 버튼: strong 1, weak 1
    let handle = button.handle();
    println!(
        "handle을 얻은 뒤: strong {}, 같은 위젯? {}",
        Rc::strong_count(&button),
        Rc::ptr_eq(&button, &handle)
    ); // 출력: handle을 얻은 뒤: strong 2, 같은 위젯? true
    // 자기 자신을 Rc로 품었다면 strong이 절대 0이 되지 않아 누수였을 것입니다. Weak라서 순환 소유가 아닙니다.
}
