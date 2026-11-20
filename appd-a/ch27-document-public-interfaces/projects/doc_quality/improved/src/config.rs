//! 읽은 설정을 담는 [`Config`]. 비공개 모듈이라 이 문서는 공개 문서에 나오지 않습니다.

/// [`parse`](crate::parse)가 읽은 설정입니다.
///
/// 키를 파일에 적힌 순서대로 기억하고, [`keys`](Config::keys)도 그 순서로 돌려줍니다.
/// [`Default`]로 만든 값은 키가 하나도 없는 설정입니다.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    entries: Vec<(String, String)>,
}

impl Config {
    /// `key`의 값을 돌려줍니다. 키는 대소문자를 구분합니다.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// `key`의 값을 돌려주고, 없으면 패닉합니다.
    ///
    /// # Panics
    ///
    /// `key`가 설정에 없으면 패닉합니다. 패닉 메시지는 이 메서드를 부른 줄을 가리킵니다.
    /// 키가 빠질 수 있는 설정이라면 [`get`](Config::get)으로 `None`을 처리하세요.
    #[track_caller]
    pub fn require(&self, key: &str) -> &str {
        match self.get(key) {
            Some(value) => value,
            None => panic!("설정에 `{key}` 키가 없습니다"),
        }
    }

    /// 키를 파일에 적힌 순서대로 돌려줍니다.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(k, _)| k.as_str())
    }

    /// 항목을 끝에 더합니다. 겹치는 키는 [`parse`](crate::parse)가 미리 걸러 냅니다.
    pub(crate) fn insert(&mut self, key: &str, value: &str) {
        self.entries.push((key.to_string(), value.to_string()));
    }
}
