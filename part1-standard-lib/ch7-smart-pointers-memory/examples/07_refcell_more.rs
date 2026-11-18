// 이 예제가 보여 주는 것: RefCell을 더 깊이 쓰기 — try_borrow_mut으로 재진입 감지, Ref::map/RefMut::map,
//                        가드 없이 통째로 다루는 take/replace/replace_with, 대여가 살아 있는 범위의 함정, get_mut
// 실행: cargo run -p p1-ch7-smart-pointers-memory --example 07_refcell_more
//
// 이 예제는 패닉하지 않습니다. 대여 규칙 위반은 패닉하는 borrow_mut 대신 try_borrow_mut으로 관찰합니다.
// (borrow_mut이 패닉하는 모습은 Part 0 ch14의 06_refcell 예제에 있습니다.)

use std::cell::{Ref, RefCell, RefMut};
use std::collections::VecDeque;

// 게시판. 글 목록을 RefCell로 감싸 &self 메서드에서도 글을 추가할 수 있게 했습니다.
struct Board {
    posts: RefCell<Vec<String>>,
}

impl Board {
    // borrow_mut 대신 try_borrow_mut을 써서, 누가 이미 글 목록을 빌려 쓰는 중이면 패닉 대신 Err를 돌려줍니다.
    fn post(&self, text: &str) -> Result<(), String> {
        match self.posts.try_borrow_mut() {
            Ok(mut posts) => {
                posts.push(text.to_string());
                Ok(())
            }
            Err(e) => Err(format!("글 올리기 실패 [{text}]: {e}")),
        }
    }

    // 글마다 콜백을 부릅니다. 순회하는 동안 글 목록의 불변 대여(Ref)가 살아 있습니다.
    fn for_each(&self, f: impl Fn(&str)) {
        for p in self.posts.borrow().iter() {
            f(p);
        }
    }
}

struct ProfileData {
    name: String,
    visits: u32,
}

struct Profile {
    data: RefCell<ProfileData>,
}

impl Profile {
    // &str을 돌려주면 borrow()가 만든 Ref가 함수 끝에서 사라져 E0515입니다(compile_fail 참고).
    // Ref::map은 Ref를 "안의 일부를 가리키는 Ref"로 바꿉니다. 대여는 돌려받은 Ref가 살아 있는 동안 이어집니다.
    fn name(&self) -> Ref<'_, str> {
        Ref::map(self.data.borrow(), |d| d.name.as_str())
    }

    fn visits_mut(&self) -> RefMut<'_, u32> {
        RefMut::map(self.data.borrow_mut(), |d| &mut d.visits)
    }
}

fn main() {
    println!("== 1. try_borrow_mut으로 재진입 감지 ==");
    let board = Board {
        posts: RefCell::new(vec![String::from("공지")]),
    };
    // 콜백 안에서 같은 게시판에 글을 쓰려 합니다. 바깥의 for_each가 Ref를 쥐고 있어 가변 대여가 실패합니다.
    board.for_each(|p| {
        if let Err(e) = board.post(&format!("{p}에 답글")) {
            println!("{e}");
        }
    });
    // 출력: 글 올리기 실패 [공지에 답글]: RefCell already borrowed
    board
        .post("새 글")
        .expect("지금은 아무도 빌리지 않았습니다");
    println!("글 목록: {:?}", board.posts.borrow()); // 출력: 글 목록: ["공지", "새 글"]

    println!("\n== 2. Ref::map / RefMut::map: 일부만 빌려 주기 ==");
    let profile = Profile {
        data: RefCell::new(ProfileData {
            name: String::from("러스티"),
            visits: 0,
        }),
    };
    *profile.visits_mut() += 1; // RefMut<u32>는 이 문장이 끝날 때 버려지고 대여도 끝납니다.
    {
        let name = profile.name(); // Ref<str>
        println!("이름 = {}, 길이 = {}", &*name, name.len()); // 출력: 이름 = 러스티, 길이 = 9
        // name이 살아 있는 동안에는 profile.data 전체가 불변으로 빌려진 상태입니다.
        println!(
            "name이 살아 있을 때 가변 대여 가능? {}",
            profile.data.try_borrow_mut().is_ok()
        ); // 출력: name이 살아 있을 때 가변 대여 가능? false
    }
    // try_borrow_mut()의 결과를 let으로 먼저 받습니다. 한 println! 안에 try_borrow_mut()과 borrow()를 함께 쓰면,
    // 앞의 임시값(Result<RefMut, _>)이 문장 끝까지 살아 있어서 뒤의 borrow()가 패닉합니다.
    let can_write = profile.data.try_borrow_mut().is_ok();
    let visits = profile.data.borrow().visits;
    println!("블록을 나온 뒤 가변 대여 가능? {can_write}, visits = {visits}");
    // 출력: 블록을 나온 뒤 가변 대여 가능? true, visits = 1

    println!("\n== 3. take / replace / replace_with: 가드를 남기지 않는 통째 연산 ==");
    let inbox = RefCell::new(vec!["메일1", "메일2"]);
    let batch = inbox.take(); // 안의 Vec을 꺼내고 빈 Vec을 남깁니다. 돌려받는 것은 Ref가 아니라 값입니다.
    println!("꺼낸 것 {batch:?}, 남은 것 {:?}", inbox.borrow()); // 출력: 꺼낸 것 ["메일1", "메일2"], 남은 것 []
    let old = inbox.replace(vec!["메일3"]);
    println!("replace가 돌려준 값 {old:?}"); // 출력: replace가 돌려준 값 []
    // replace_with는 예전 값의 &mut를 받아 새 값을 만드는 클로저를 받고, 예전 값을 돌려줍니다.
    let before = inbox.replace_with(|old| {
        let mut next = old.clone();
        next.push("메일4");
        next
    });
    println!("before {before:?}, now {:?}", inbox.borrow()); // 출력: before ["메일3"], now ["메일3", "메일4"]

    println!("\n== 4. 함정: 조건식에서 만든 대여는 본문 끝까지 살아 있습니다 ==");
    let tasks = RefCell::new(VecDeque::from(["빨래", "청소"]));
    // while let 조건식의 borrow_mut()이 만든 RefMut는 반복 본문이 끝날 때까지 버려지지 않습니다.
    while let Some(task) = tasks.borrow_mut().pop_front() {
        println!(
            "{task}: 본문에서 가변 대여 가능? {}",
            tasks.try_borrow_mut().is_ok()
        );
    }
    // 출력:
    // 빨래: 본문에서 가변 대여 가능? false
    // 청소: 본문에서 가변 대여 가능? false
    // 본문에서 borrow_mut()으로 할 일을 추가했다면 여기서 패닉했을 것입니다.

    // 고치는 법: let 문으로 먼저 꺼냅니다. let 문의 임시값(RefMut)은 그 문장이 끝날 때 버려집니다.
    let tasks = RefCell::new(VecDeque::from(["빨래", "청소"]));
    loop {
        let next = tasks.borrow_mut().pop_front();
        let Some(task) = next else { break };
        if task == "청소" {
            tasks.borrow_mut().push_back("정리"); // 이제는 아무도 빌리고 있지 않아 안전합니다.
        }
        println!("처리: {task}");
    }
    // 출력:
    // 처리: 빨래
    // 처리: 청소
    // 처리: 정리

    // match도 같습니다. 조건식의 Ref가 모든 갈래가 끝날 때까지 살아 있습니다.
    let log = RefCell::new(Vec::<String>::new());
    match log.borrow().last() {
        Some(last) => println!("마지막 기록: {last}"),
        None => println!(
            "기록 없음. 이 갈래에서 가변 대여 가능? {}",
            log.try_borrow_mut().is_ok()
        ),
    }
    // 출력: 기록 없음. 이 갈래에서 가변 대여 가능? false

    println!("\n== 5. &mut RefCell이 있으면 get_mut: 실행 중 검사 없이 ==");
    let mut settings = RefCell::new(vec![1, 2]);
    settings.get_mut().push(3); // 컴파일러가 유일한 접근임을 보장하므로 대여 카운트를 건드리지 않습니다.
    println!("settings = {:?}", settings.into_inner()); // 출력: settings = [1, 2, 3]
}
