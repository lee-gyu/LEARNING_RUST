//! 비권장: 공개 필드와 트레이트 메서드를 기능으로 가린 알림 라이브러리.
//!
//! `attachments` 기능이 켜지면 `Message`에 공개 필드가, `Channel`에 필수 메서드가 하나씩
//! 생깁니다. 이 크레이트를 쓰는 쪽은 그 기능이 켜졌는지 알 수 없습니다. 자기가 켜지 않았어도
//! 빌드 그래프의 다른 크레이트가 켤 수 있기 때문입니다.

/// 보낼 메시지. 필드가 모두 공개라서 쓰는 쪽이 구조체 리터럴로 만듭니다.
#[derive(Debug, Clone)]
pub struct Message {
    pub to: String,
    pub body: String,
    /// 첨부 파일 이름. `attachments` 기능이 켜졌을 때만 있습니다.
    #[cfg(feature = "attachments")]
    pub attachments: Vec<String>,
}

/// 메시지를 보내는 통로. 쓰는 쪽이 자기 통로 타입에 구현합니다.
pub trait Channel {
    /// 통로 이름.
    fn name(&self) -> &str;

    /// 메시지를 보내고, 보낸 내용을 한 줄로 돌려줍니다.
    fn deliver(&self, msg: &Message) -> String;

    /// 한 번에 보낼 수 있는 첨부 파일 수. `attachments` 기능이 켜졌을 때만 있습니다.
    #[cfg(feature = "attachments")]
    fn max_attachments(&self) -> usize;
}
