//! 권장: 기능이 공개 API를 바꾸지 않고 더하기만 하는 알림 라이브러리.
//!
//! broken/의 notice와 비교하면 두 곳이 다릅니다.
//!
//! - `Message`의 필드를 비공개로 두고 생성자 `new`로만 만들게 했습니다. 기능으로 가린 필드가
//!   있어도 밖에서는 보이지 않으므로, 쓰는 쪽 코드는 기능이 켜졌든 꺼졌든 똑같이 컴파일됩니다.
//!   기능은 `attach`, `attachments` 메서드를 더하기만 합니다.
//! - 첨부 파일에 관한 메서드는 `Channel`에 넣지 않고, 기능이 켜졌을 때만 있는 별도의 트레이트
//!   `AttachmentChannel`로 뺐습니다. `Channel`의 필수 메서드는 어떤 빌드에서나 같습니다.

/// 보낼 메시지. 필드가 비공개라서 `Message::new`로 만듭니다.
#[derive(Debug, Clone)]
pub struct Message {
    to: String,
    body: String,
    // 비공개 필드는 기능으로 가려도 괜찮습니다. 이 필드를 아는 코드는 이 크레이트 안에만 있고,
    // 이 크레이트는 자기 기능이 켜졌는지 `cfg`로 알 수 있습니다.
    #[cfg(feature = "attachments")]
    attachments: Vec<String>,
}

impl Message {
    /// 받는 사람과 본문으로 메시지를 만듭니다. 기능과 상관없이 늘 같은 시그니처입니다.
    pub fn new(to: impl Into<String>, body: impl Into<String>) -> Self {
        Message {
            to: to.into(),
            body: body.into(),
            // 구조체 리터럴의 필드에도 cfg를 붙일 수 있습니다. 크레이트 안이라서 가능합니다.
            #[cfg(feature = "attachments")]
            attachments: Vec::new(),
        }
    }

    /// 받는 사람.
    pub fn to(&self) -> &str {
        &self.to
    }

    /// 본문.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// 첨부 파일을 하나 더합니다. `attachments` 기능이 켜졌을 때만 있습니다.
    #[cfg(feature = "attachments")]
    pub fn attach(mut self, file: impl Into<String>) -> Self {
        self.attachments.push(file.into());
        self
    }

    /// 첨부 파일 이름들. `attachments` 기능이 켜졌을 때만 있습니다.
    #[cfg(feature = "attachments")]
    pub fn attachments(&self) -> &[String] {
        &self.attachments
    }
}

/// 메시지를 보내는 통로. 필수 메서드는 기능과 상관없이 늘 이 둘입니다.
pub trait Channel {
    /// 통로 이름.
    fn name(&self) -> &str;

    /// 메시지를 보내고, 보낸 내용을 한 줄로 돌려줍니다.
    fn deliver(&self, msg: &Message) -> String;
}

/// 첨부 파일을 보낼 수 있는 통로. `attachments` 기능이 켜졌을 때만 있습니다.
///
/// 이 트레이트를 구현하는 크레이트는 이 트레이트의 이름을 써야 하므로, 자기 `Cargo.toml`에서
/// 기능을 직접 켤 수밖에 없습니다. 기능을 모르는 크레이트는 `Channel`만 구현하면 됩니다.
#[cfg(feature = "attachments")]
pub trait AttachmentChannel: Channel {
    /// 한 번에 보낼 수 있는 첨부 파일 수.
    fn max_attachments(&self) -> usize;

    /// 첨부 파일 수를 확인한 뒤 보냅니다.
    fn deliver_checked(&self, msg: &Message) -> Result<String, String> {
        let (n, max) = (msg.attachments().len(), self.max_attachments());
        if n > max {
            let name = self.name();
            return Err(format!("[{name}] 첨부 파일이 {n}개입니다(최대 {max}개)"));
        }
        Ok(self.deliver(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works_in_every_build() {
        let m = Message::new("운영팀", "점검 완료");
        assert_eq!((m.to(), m.body()), ("운영팀", "점검 완료"));
    }

    #[cfg(feature = "attachments")]
    #[test]
    fn attach_adds_files() {
        let m = Message::new("팀장", "주간 보고").attach("보고서.pdf");
        assert_eq!(m.attachments(), ["보고서.pdf"]);
    }

    #[cfg(feature = "attachments")]
    #[test]
    fn deliver_checked_rejects_too_many() {
        struct Fax;
        impl Channel for Fax {
            fn name(&self) -> &str {
                "팩스"
            }
            fn deliver(&self, msg: &Message) -> String {
                msg.body().to_string()
            }
        }
        impl AttachmentChannel for Fax {
            fn max_attachments(&self) -> usize {
                1
            }
        }

        let m = Message::new("팀장", "주간 보고")
            .attach("a.pdf")
            .attach("b.pdf");
        let expected = "[팩스] 첨부 파일이 2개입니다(최대 1개)";
        assert_eq!(Fax.deliver_checked(&m), Err(expected.to_string()));
    }
}
