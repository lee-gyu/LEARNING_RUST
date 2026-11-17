// 내부 가변성이 필요한 실제 상황: 테스트용 가짜 객체(mock object)를 만듭니다.
// - Messenger 트레이트의 send는 &self를 받으므로, 구현하는 쪽은 자기 필드를 바꿀 수 없습니다.
// - 가짜 메신저는 "무엇을 보냈는지" 기록해야 합니다. 기록 칸을 RefCell로 감싸면 &self로도 바꿀 수 있습니다.
// - RefCell 없이 Vec<String>에 바로 push하면 E0596으로 컴파일되지 않습니다(README 2.8절).
//
// 실행:   cargo run -p p0-ch14-smart-pointers --example 07_mock_messenger
// 테스트: cargo test -p p0-ch14-smart-pointers --example 07_mock_messenger

use std::cell::RefCell;

// 알림을 보내는 능력을 나타내는 트레이트입니다. 메일이든 문자든, 보내는 방법은 구현하는 쪽이 정합니다.
// 보내기는 "읽기 전용" 동작으로 설계되어 &self를 받습니다.
pub trait Messenger {
    fn send(&self, msg: &str);
}

// 사용량이 한도에 얼마나 가까운지 추적하다가 메신저로 알림을 보냅니다.
// 메신저를 참조로 빌려 쓰므로 라이프타임 'a가 필요합니다(ch10).
pub struct LimitTracker<'a, T: Messenger> {
    messenger: &'a T,
    value: usize,
    max: usize,
}

impl<'a, T> LimitTracker<'a, T>
where
    T: Messenger,
{
    pub fn new(messenger: &'a T, max: usize) -> LimitTracker<'a, T> {
        LimitTracker {
            messenger,
            value: 0,
            max,
        }
    }

    pub fn set_value(&mut self, value: usize) {
        self.value = value;

        let percentage_of_max = self.value as f64 / self.max as f64;

        if percentage_of_max >= 1.0 {
            self.messenger.send("에러: 할당량을 넘었습니다!");
        } else if percentage_of_max >= 0.9 {
            self.messenger
                .send("긴급 경고: 할당량의 90% 이상을 사용했습니다!");
        } else if percentage_of_max >= 0.75 {
            self.messenger
                .send("경고: 할당량의 75% 이상을 사용했습니다!");
        }
    }
}

// 실제 프로그램에서 쓸 메신저입니다. 여기서는 화면에 출력하는 것으로 대신합니다.
struct ConsoleMessenger;

impl Messenger for ConsoleMessenger {
    fn send(&self, msg: &str) {
        println!("    [알림] {msg}");
    }
}

// 테스트용 가짜 메신저입니다. 보내지 않고 기록만 합니다.
// The Book은 이 타입을 tests 모듈 안에 두지만, 여기서는 main에서도 관찰하려고 밖에 두었습니다.
struct MockMessenger {
    sent_messages: RefCell<Vec<String>>,
}

impl MockMessenger {
    fn new() -> MockMessenger {
        MockMessenger {
            sent_messages: RefCell::new(vec![]),
        }
    }
}

impl Messenger for MockMessenger {
    fn send(&self, message: &str) {
        // self는 &MockMessenger(불변)이지만, RefCell이 실행 중에 가변 대여를 내줍니다.
        // 반환된 RefMut는 이 문장이 끝날 때 버려지므로 가변 대여도 바로 끝납니다.
        self.sent_messages.borrow_mut().push(String::from(message));
    }
}

fn main() {
    println!("1. 실제 메신저: 사용량에 따라 알림을 보냅니다");
    let console = ConsoleMessenger;
    let mut tracker = LimitTracker::new(&console, 100);
    for value in [50, 80, 95, 120] {
        println!("    사용량 {value}/100");
        tracker.set_value(value);
    }
    // 출력:
    //     사용량 50/100
    //     사용량 80/100
    //     [알림] 경고: 할당량의 75% 이상을 사용했습니다!
    //     사용량 95/100
    //     [알림] 긴급 경고: 할당량의 90% 이상을 사용했습니다!
    //     사용량 120/100
    //     [알림] 에러: 할당량을 넘었습니다!

    println!("2. 가짜 메신저: &self인 send 안에서도 RefCell 덕분에 기록이 남습니다");
    let mock = MockMessenger::new();
    let mut tracker = LimitTracker::new(&mock, 100);
    tracker.set_value(80);
    tracker.set_value(95);
    // 기록을 읽을 때는 borrow()로 불변 대여합니다.
    let sent = mock.sent_messages.borrow();
    println!("    기록된 메시지: {}개", sent.len()); // 출력: 기록된 메시지: 2개
    for message in sent.iter() {
        println!("    - {message}");
    }
    // 출력:
    //     - 경고: 할당량의 75% 이상을 사용했습니다!
    //     - 긴급 경고: 할당량의 90% 이상을 사용했습니다!
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_sends_an_over_75_percent_warning_message() {
        let mock_messenger = MockMessenger::new();
        let mut limit_tracker = LimitTracker::new(&mock_messenger, 100);

        limit_tracker.set_value(80);

        assert_eq!(mock_messenger.sent_messages.borrow().len(), 1);
    }

    #[test]
    fn it_sends_nothing_under_75_percent() {
        let mock_messenger = MockMessenger::new();
        let mut limit_tracker = LimitTracker::new(&mock_messenger, 100);

        limit_tracker.set_value(74);

        assert!(mock_messenger.sent_messages.borrow().is_empty());
    }

    #[test]
    fn it_sends_an_error_when_over_quota() {
        let mock_messenger = MockMessenger::new();
        let mut limit_tracker = LimitTracker::new(&mock_messenger, 100);

        limit_tracker.set_value(101);

        // 메시지 내용까지 확인합니다. borrow()가 돌려준 Ref는 이 문장이 끝나면 버려집니다.
        assert_eq!(
            mock_messenger.sent_messages.borrow()[0],
            "에러: 할당량을 넘었습니다!"
        );
    }
}
