//! 서버 경보를 SMS로 보내는 크레이트. notice의 `attachments` 기능은 모릅니다.
//!
//! 이 크레이트의 코드는 notice의 공개 API를 문서대로 썼고, 혼자 빌드하면 문제가 없습니다.
//! 그런데 같은 빌드의 다른 크레이트가 `attachments`를 켜면 아래 두 곳이 깨집니다.

use notice::{Channel, Message};

/// 문자 메시지 통로.
pub struct Sms;

// 기능이 켜지면 E0046: `max_attachments`를 구현하지 않았다는 에러가 납니다.
impl Channel for Sms {
    fn name(&self) -> &str {
        "SMS"
    }

    fn deliver(&self, msg: &Message) -> String {
        format!("[{}] {} ← {}", self.name(), msg.to, msg.body)
    }
}

/// 디스크 사용량 경보 메시지를 만듭니다.
pub fn disk_alert(percent: u8) -> Message {
    // 기능이 켜지면 E0063: `attachments` 필드가 빠졌다는 에러가 납니다.
    // 반대로 이 필드를 미리 적어 두면, 기능이 꺼진 빌드에서 E0560(그런 필드 없음)이 납니다.
    Message {
        to: "운영팀".to_string(),
        body: format!("디스크 사용량 {percent}%"),
    }
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
