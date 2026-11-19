// 기대하는 에러: E0507
//   error[E0507]: cannot move out of `self.calling` which is behind a mutable reference
//
// `&mut self`로 빌린 구조체에서 필드 값을 옮겨 가면, 다음 줄에서 다시 채우더라도
// 그 사이에 빌려준 쪽에 빈자리가 생깁니다. 컴파일러는 그 순간을 허락하지 않습니다.
// 컴파일러는 `&self.calling`(빌리기)을 제안하고, `Party`가 `Clone`이었다면 복제할 수 있다고
// 덧붙입니다. 이 함수가 원하는 것은 예전 값을 넘겨받는 것이므로 둘 다 답이 아닙니다.
// `mem::replace`나 `Option::replace`로 바꿔 끼웁니다(예제 03).
//
// 확인: rustc --edition 2024 compile_fail/e0507_move_out_then_replace.rs --out-dir /tmp/er-ch15

use std::collections::VecDeque;

struct Party {
    name: String,
}

struct Waitlist {
    calling: Option<Party>,
    waiting: VecDeque<Party>,
}

impl Waitlist {
    /// 다음 손님을 부르고, 앞서 불렀던 손님을 돌려줍니다.
    fn call_next(&mut self) -> Option<Party> {
        let previous = self.calling; // E0507: 빌린 자리에서 옮김
        self.calling = self.waiting.pop_front();
        previous
    }
}

fn main() {
    let mut list = Waitlist {
        calling: None,
        waiting: VecDeque::from([Party {
            name: String::from("김"),
        }]),
    };
    list.call_next();
    if let Some(p) = list.call_next() {
        println!("오지 않은 손님: {}", p.name);
    }
}
