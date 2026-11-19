//! 첨부 파일을 보내는 애플리케이션. 이 워크스페이스에서는 빌드되지 않습니다.
//!
//!   cargo build -p app
//!
//! 이 파일은 `attachments`가 켜진 notice에 맞게 올바르게 썼습니다. 에러는 alerts 안에서 납니다.

use notice::{Channel, Message};

struct Email;

impl Channel for Email {
    fn name(&self) -> &str {
        "이메일"
    }

    fn deliver(&self, msg: &Message) -> String {
        let (to, body, n) = (&msg.to, &msg.body, msg.attachments.len());
        format!("[{}] {to} ← {body} (첨부 {n}개)", self.name())
    }

    fn max_attachments(&self) -> usize {
        2
    }
}

fn main() {
    println!("{}", alerts::Sms.deliver(&alerts::disk_alert(93)));

    let report = Message {
        to: "팀장".to_string(),
        body: "주간 보고".to_string(),
        attachments: vec!["보고서.pdf".to_string(), "지표.csv".to_string()],
    };
    if report.attachments.len() <= Email.max_attachments() {
        println!("{}", Email.deliver(&report));
    }
}
