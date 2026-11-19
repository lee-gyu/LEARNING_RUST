// 한 줄을 Record로 읽는 모듈입니다.

// 2018: 경로가 외부 크레이트 이름으로 시작하면 그 크레이트를 가리킵니다.
// 글자는 2015판과 같지만, 크레이트 루트가 아니라 외부 프렐류드에서 `scorekit`을 찾습니다.
use scorekit::ScoreError;

pub struct Record {
    pub name: String,
    pub score: u32,
}

/// "이름 점수" 한 줄을 읽습니다.
pub fn parse_line(line: &str) -> Result<Record, ScoreError> {
    let mut parts = line.split_whitespace();
    let name = parts.next().unwrap_or("");
    // 2018: try는 예약 키워드이므로 2015 크레이트의 try 함수는 raw 식별자 r#try로 부릅니다.
    // 에러는 ? 연산자로 전파합니다.
    let score = scorekit::r#try(parts.next().unwrap_or(""))?;
    Ok(Record {
        name: name.to_string(),
        score,
    })
}
