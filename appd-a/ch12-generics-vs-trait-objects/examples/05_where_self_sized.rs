// 트레이트 객체로도 쓸 트레이트에 제네릭 메서드와 `Self`를 돌려주는 메서드를 함께 두는 방법입니다.
// 두 메서드에 `where Self: Sized`를 붙이면 "크기를 아는 구체 타입에서만 부르는 메서드"가 되어
// vtable에서 빠지고, 트레이트 전체는 dyn 호환으로 남습니다.
// 대신 그 메서드는 트레이트 객체로는 부를 수 없습니다(compile_fail/sized_method_on_dyn.rs).
//
// 실행: cargo run -p er-ch12-generics-vs-trait-objects --example 05_where_self_sized

/// 예약한 자료가 도착했을 때 이용자에게 알리는 채널.
trait Notifier {
    /// 핵심 메서드: 수신자 말고는 `Self`가 없고 제네릭도 아니므로 vtable에 들어갑니다.
    fn send(&self, to: &str, text: &str) -> String;

    /// 제네릭 메서드: `I`마다 한 벌씩 만들어지므로 vtable의 칸 하나에 담을 수 없습니다.
    /// `Self: Sized`로 트레이트 객체에서 빼면 트레이트는 dyn 호환으로 남습니다.
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

    /// `Self`를 돌려주는 메서드: 트레이트 객체로 부르는 쪽은 돌려받을 값의 크기를 모르므로
    /// 마찬가지로 `Self: Sized`로 뺍니다.
    fn with_sender(&self, sender: &str) -> Self
    where
        Self: Sized;
}

struct Sms {
    sender: String,
}

struct Email {
    sender: String,
}

impl Notifier for Sms {
    fn send(&self, to: &str, text: &str) -> String {
        format!("[문자 {} → {to}] {text}", self.sender)
    }
    fn with_sender(&self, sender: &str) -> Self {
        Sms {
            sender: sender.to_string(),
        }
    }
}

impl Notifier for Email {
    fn send(&self, to: &str, text: &str) -> String {
        format!("[메일 {} → {to}] {text}", self.sender)
    }
    fn with_sender(&self, sender: &str) -> Self {
        Email {
            sender: sender.to_string(),
        }
    }
}

/// 트레이트 객체에도 "여러 개 보내기"가 필요하다면 트레이트 밖에 함수를 둡니다.
/// `?Sized`를 붙였으므로 구체 타입과 트레이트 객체를 모두 받습니다.
fn send_each_any<N, I>(notifier: &N, to: &str, texts: I) -> Vec<String>
where
    N: Notifier + ?Sized,
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    texts
        .into_iter()
        .map(|text| notifier.send(to, text.as_ref()))
        .collect()
}

fn main() {
    let sms = Sms {
        sender: "중앙도서관".to_string(),
    };
    let email = Email {
        sender: "중앙도서관".to_string(),
    };

    println!("1. 구체 타입으로는 모든 메서드를 부릅니다");
    for line in sms.send_each("010-1234", ["예약 도서 도착", "3일 안에 찾아가세요"]) {
        println!("    {line}");
    }
    // 출력:     [문자 중앙도서관 → 010-1234] 예약 도서 도착
    // 출력:     [문자 중앙도서관 → 010-1234] 3일 안에 찾아가세요
    let branch = sms.with_sender("동네 분관");
    println!(
        "    {}",
        branch.send("010-1234", "분관에서도 찾을 수 있습니다")
    );
    // 출력:     [문자 동네 분관 → 010-1234] 분관에서도 찾을 수 있습니다

    println!("2. 트레이트 객체로는 vtable에 있는 핵심 메서드만 부릅니다");
    // 트레이트가 dyn 호환이라 서로 다른 채널을 한 Vec에 담을 수 있습니다.
    let channels: Vec<Box<dyn Notifier>> = vec![Box::new(sms), Box::new(email)];
    for channel in &channels {
        println!("    {}", channel.send("김러스트", "예약 도서 도착"));
    }
    // 출력:     [문자 중앙도서관 → 김러스트] 예약 도서 도착
    // 출력:     [메일 중앙도서관 → 김러스트] 예약 도서 도착
    // channels[0].send_each(..) 와 channels[0].with_sender(..) 는 컴파일되지 않습니다.

    println!("3. 트레이트 밖의 ?Sized 함수는 트레이트 객체도 받습니다");
    let sent = send_each_any(
        channels[1].as_ref(),
        "kim@example.com",
        ["도착", "마감 임박"],
    );
    println!("    {}", sent.join(" / "));
    // 출력:     [메일 중앙도서관 → kim@example.com] 도착 / [메일 중앙도서관 → kim@example.com] 마감 임박

    println!("4. 표준 라이브러리의 Iterator도 같은 설계입니다");
    // `map`, `filter`, `sum` 같은 메서드에는 `Self: Sized`가 붙어 있어서 `dyn Iterator`가 가능합니다.
    // 그런데도 `Box<dyn Iterator>`에 `map`을 부를 수 있는 것은 `Box<I>`가 다시 `Iterator`를
    // 구현하기 때문입니다(`impl<I: Iterator + ?Sized> Iterator for Box<I>`). `Box`는 크기를 아는
    // 구체 타입이므로 `Self: Sized` 메서드를 부를 수 있습니다.
    let days: Box<dyn Iterator<Item = u32>> = Box::new([14, 7, 3].into_iter());
    let doubled: u32 = days.map(|d| d * 2).sum();
    println!("    연장한 대출 기간 합: {doubled}일");
    // 출력:     연장한 대출 기간 합: 48일
}
