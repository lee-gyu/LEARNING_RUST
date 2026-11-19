//! 서버 경보를 SMS로 보내는 크레이트. notice의 `attachments` 기능은 모릅니다(broken/과 같음).
//!
//! notice가 공개 API를 기능으로 가리지 않으므로, 같은 빌드의 다른 크레이트가 기능을 켜도
//! 이 코드는 그대로 컴파일됩니다.

use notice::{Channel, Message};

/// 문자 메시지 통로.
pub struct Sms;

// Channel의 필수 메서드는 어떤 빌드에서나 name과 deliver 둘뿐입니다.
impl Channel for Sms {
    fn name(&self) -> &str {
        "SMS"
    }

    fn deliver(&self, msg: &Message) -> String {
        format!("[{}] {} ← {}", self.name(), msg.to(), msg.body())
    }
}

/// 디스크 사용량 경보 메시지를 만듭니다.
pub fn disk_alert(percent: u8) -> Message {
    // 구조체 리터럴 대신 생성자를 씁니다. 숨은 필드가 몇 개든 이 줄은 바뀌지 않습니다.
    Message::new("운영팀", format!("디스크 사용량 {percent}%"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sms_delivers_alert() {
        assert_eq!(
            Sms.deliver(&disk_alert(93)),
            "[SMS] 운영팀 ← 디스크 사용량 93%"
        );
    }
}
