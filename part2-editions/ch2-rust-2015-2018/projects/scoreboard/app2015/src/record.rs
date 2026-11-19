// 한 줄을 Record로 읽는 모듈입니다.

// 2015: `use` 경로는 크레이트 루트에서 출발합니다. main.rs의 `extern crate scorekit;`이
// 크레이트 루트에 `scorekit`이라는 이름을 만들어 두었기 때문에 이 경로가 통합니다.
use scorekit::ScoreError;

pub struct Record {
    pub name: String,
    pub score: u32,
}

/// "이름 점수" 한 줄을 읽습니다.
// try!는 deprecated 매크로라서 2015에서도 경고가 납니다. 옛 코드의 모습을 보이려고 일부러 씁니다.
#[allow(deprecated)]
pub fn parse_line(line: &str) -> Result<Record, ScoreError> {
    let mut parts = line.split_whitespace();
    let name = parts.next().unwrap_or("");
    // 2015: try는 키워드가 아니므로 scorekit::try를 그대로 부릅니다.
    // 에러는 try! 매크로로 전파합니다(? 연산자와 같은 일을 합니다).
    let score = try!(scorekit::try(parts.next().unwrap_or("")));
    Ok(Record {
        name: name.to_string(),
        score,
    })
}
