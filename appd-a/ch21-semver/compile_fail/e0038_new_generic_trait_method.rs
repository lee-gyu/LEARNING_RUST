// 기대하는 에러: E0038 — the trait `Notifier` is not dyn compatible
// 핵심 메시지: ...because method `notify_all` has generic type parameters
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0038_new_generic_trait_method.rs --out-dir /tmp/er-ch21
//
// 상황: 모듈 parcel은 다른 사람이 관리하는 라이브러리라고 생각합니다.
//   - 1.0.0의 트레이트 Notifier(배송 알림 수단)는 dyn 호환이었고, 사용자는 문자·메일 알림을
//     Vec<Box<dyn Notifier>>에 섞어 담았습니다.
//   - 1.1.0에서 라이브러리가 기본 구현이 있는 메서드 notify_all을 더했습니다. 기본 구현이 있으므로
//     사용자의 impl은 고칠 것이 없습니다(ch13 2.5절). 그런데 이 메서드는 타입 매개변수 I를 받습니다.
//   - 타입 매개변수가 있는 메서드는 vtable에 넣을 수 없으므로 트레이트 전체가 dyn 호환이 아니게 되고,
//     `dyn Notifier`를 쓰던 사용자 코드가 깨집니다.
// 해결: 라이브러리가 새 메서드에 `where Self: Sized`를 붙이면 됩니다(ch12 2.6절). 그 메서드는
//       트레이트 객체에서 부를 수 없게 되지만 트레이트는 dyn 호환으로 남습니다.

mod parcel {
    pub trait Notifier {
        fn notify(&self, message: &str) -> String;

        // 1.1.0에서 더한 기본 메서드
        fn notify_all<I: IntoIterator<Item = String>>(&self, messages: I) -> Vec<String> {
            messages.into_iter().map(|m| self.notify(&m)).collect()
        }
    }
}

use parcel::Notifier;

struct Sms;
struct Mail;

impl Notifier for Sms {
    fn notify(&self, message: &str) -> String {
        format!("문자: {message}")
    }
}

impl Notifier for Mail {
    fn notify(&self, message: &str) -> String {
        format!("메일: {message}")
    }
}

fn main() {
    // 1.0.0 시절에 쓴 사용자 코드
    let channels: Vec<Box<dyn Notifier>> = vec![Box::new(Sms), Box::new(Mail)]; // E0038
    for c in &channels {
        println!("{}", c.notify("KR-1203 배송 출발"));
    }
}
