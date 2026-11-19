// 기대하는 에러: 에러 코드 없음 — the `send_each` method cannot be invoked on a trait object
// 핵심 메시지: this has a `Sized` requirement
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/sized_method_on_dyn.rs --out-dir /tmp/er-ch12
//
// `where Self: Sized`를 붙인 메서드는 vtable에 들어가지 않습니다. 덕분에 트레이트는 dyn 호환으로
// 남지만, 그 메서드는 크기를 아는 구체 타입에서만 부를 수 있습니다. `Box<dyn Notifier>`에서
// 부르면 자동 역참조로 `dyn Notifier`의 메서드를 찾게 되고, `dyn Notifier`는 `Sized`가 아니므로
// 거부됩니다.
// 해결: 구체 타입의 값에서 부르거나, 트레이트 밖에 `N: Notifier + ?Sized`로 받는 함수를 두고
//       그 함수를 부릅니다(examples/05_where_self_sized.rs의 `send_each_any`).

trait Notifier {
    fn send(&self, to: &str, text: &str) -> String;

    fn send_each<I>(&self, to: &str, texts: I) -> Vec<String>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        Self: Sized,
    {
        texts
            .into_iter()
            .map(|text| self.send(to, text.as_ref()))
            .collect()
    }
}

struct Sms;

impl Notifier for Sms {
    fn send(&self, to: &str, text: &str) -> String {
        format!("[문자 → {to}] {text}")
    }
}

fn main() {
    let channel: Box<dyn Notifier> = Box::new(Sms);
    let sent = channel.send_each("010-1234", ["도착", "마감 임박"]); // 에러
    println!("{sent:?}");
}
