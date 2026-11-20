/// 설정. 항목은 [`Config::insert`]로 넣습니다.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    entries: Vec<(String, String)>,
}

impl Config {
    /// `key: &str`를 받아 `Option<&str>`을 돌려줍니다.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// 값을 가져옵니다. [`Config::value`]와 같지만 `Option`이 아니라 `&str`을 돌려줍니다.
    #[track_caller]
    pub fn require(&self, key: &str) -> &str {
        match self.get(key) {
            Some(value) => value,
            None => panic!("설정에 `{key}` 키가 없습니다"),
        }
    }

    /// 키들의 반복자를 돌려줍니다.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(k, _)| k.as_str())
    }

    pub(crate) fn insert(&mut self, key: &str, value: &str) {
        self.entries.push((key.to_string(), value.to_string()));
    }
}
