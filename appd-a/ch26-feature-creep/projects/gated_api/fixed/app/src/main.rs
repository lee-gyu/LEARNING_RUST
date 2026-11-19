//! 첨부 파일을 보내는 애플리케이션. 기능을 켜도 alerts가 깨지지 않습니다.
//!
//!   cargo run -p app

use notice::{AttachmentChannel, Channel, Message};

struct Email;

impl Channel for Email {
    fn name(&self) -> &str {
        "이메일"
    }

    fn deliver(&self, msg: &Message) -> String {
        let (to, body, n) = (msg.to(), msg.body(), msg.attachments().len());
        format!("[{}] {to} ← {body} (첨부 {n}개)", self.name())
    }
}

// 첨부 파일을 보낼 수 있는 통로만 이 트레이트를 더 구현합니다.
impl AttachmentChannel for Email {
    fn max_attachments(&self) -> usize {
        2
    }
}

fn main() {
    // 출력: [SMS] 운영팀 ← 디스크 사용량 93%
    println!("{}", alerts::Sms.deliver(&alerts::disk_alert(93)));

    let report = Message::new("팀장", "주간 보고")
        .attach("보고서.pdf")
        .attach("지표.csv");
    match Email.deliver_checked(&report) {
        // 출력: [이메일] 팀장 ← 주간 보고 (첨부 2개)
        Ok(line) => println!("{line}"),
        Err(e) => println!("{e}"),
    }
}
