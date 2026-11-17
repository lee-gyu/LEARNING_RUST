//! minigrep의 라이브러리 크레이트입니다(ch13 개선판).
//!
//! ch12 판과 하는 일은 같고, 반복자와 클로저로 다음 두 곳을 고쳤습니다.
//! - `Config::build`가 `&[String]` 대신 명령줄 인자 반복자를 받아 `clone` 없이 값을 가져옵니다.
//! - `search`와 `search_case_insensitive`가 `for` 반복문 대신 `filter`와 `collect`를 씁니다.
//!
//! 바이너리 크레이트(src/main.rs)는 이 라이브러리를 `minigrep::...` 경로로 불러 쓰기만 합니다.

use std::env;
use std::error::Error;
use std::fs;

/// 프로그램을 어떻게 실행할지 정하는 설정 값의 묶음입니다.
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    /// 명령줄 인자 반복자(첫 값은 실행 파일 경로)와 환경 변수로 설정을 만듭니다.
    ///
    /// `impl Iterator<Item = String>`은 "String을 하나씩 내놓는 반복자라면 무엇이든"이라는 뜻입니다.
    /// main은 `env::args()`를, 테스트는 배열로 만든 반복자를 넘깁니다.
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // 반복자에는 길이(len)도 인덱스([1])도 없고, next로 앞에서부터 하나씩 꺼낼 수만 있습니다.
        // next는 반복자의 위치를 바꾸므로 매개변수에 mut을 붙였습니다.
        // 첫 값은 실행 파일 경로라서 꺼내서 버립니다.
        args.next();

        // next가 돌려주는 것은 반복자가 내놓은 String 그 자체(소유권)입니다. 빌려 온 슬라이스에서
        // 꺼내던 ch12 판과 달리 Config로 그대로 옮길 수 있으므로 clone이 필요 없습니다.
        // None이면 그 인자가 없다는 뜻이므로, 무엇이 빠졌는지 구체적으로 알릴 수 있습니다.
        let Some(query) = args.next() else {
            return Err("검색어가 없습니다");
        };

        let Some(file_path) = args.next() else {
            return Err("파일 경로가 없습니다");
        };

        // env::var는 변수가 있으면 Ok(값), 없으면 Err를 돌려줍니다.
        // 값은 보지 않고 "설정돼 있는가"만 따지므로 is_ok()로 bool을 얻습니다.
        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

/// 설정대로 파일을 읽고, 검색어가 들어 있는 줄을 표준 출력에 씁니다.
///
/// 파일을 읽지 못하면 그 에러를 `Box<dyn Error>`에 담아 돌려줍니다.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    // 검색 결과는 프로그램의 "출력"이므로 표준 출력(println!)으로 보냅니다.
    for line in results {
        println!("{line}");
    }

    Ok(())
}

/// contents에서 query가 들어 있는 줄을 모두 찾아 돌려줍니다(대소문자 구분).
///
/// 돌려주는 &str은 contents의 일부를 가리키는 슬라이스라서, 반환값의 라이프타임을
/// contents와 같은 'a로 묶습니다.
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    // lines()가 내놓는 줄 가운데 filter의 클로저가 true를 돌려준 줄만 남기고, collect로 모읍니다.
    // 클로저는 바깥의 query를 캡처해서 씁니다. 결과를 모아 둘 가변 벡터가 필요 없습니다.
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

/// search와 같지만 대소문자를 구분하지 않습니다.
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    // 검색어는 한 번만 소문자로 바꿔 두고(이제 query의 타입은 String), 클로저가 그것을 캡처합니다.
    // 결과에는 소문자로 바꾼 줄이 아니라 원래 줄이 들어갑니다. filter는 항목을 바꾸지 않고
    // 통과시킬지만 정하기 때문입니다.
    let query = query.to_lowercase();
    contents
        .lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        // 여는 따옴표 뒤의 \는 문자열 맨 앞에 개행이 들어가지 않게 합니다.
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }

    #[test]
    fn hangul() {
        // 한글도 UTF-8 문자열이므로 contains로 그대로 찾을 수 있습니다.
        let query = "바람";
        let contents = "\
죽는 날까지 하늘을 우러러
한 점 부끄럼이 없기를,
잎새에 이는 바람에도
나는 괴로워했다.";

        assert_eq!(vec!["잎새에 이는 바람에도"], search(query, contents));
    }

    // 아래 두 테스트는 ch12 판과 같은 내용입니다. build가 슬라이스 대신 반복자를 받게 되었으므로
    // &args를 args.into_iter()로 바꾼 것만 다릅니다. 배열의 into_iter()는 String을 하나씩 내놓습니다.
    #[test]
    fn build_needs_query_and_file_path() {
        let args = [String::from("minigrep"), String::from("duct")];

        assert!(Config::build(args.into_iter()).is_err());
    }

    #[test]
    fn build_reads_query_and_file_path() {
        let args = [
            String::from("minigrep"),
            String::from("duct"),
            String::from("poem.txt"),
        ];

        let config = Config::build(args.into_iter()).unwrap();

        assert_eq!(config.query, "duct");
        assert_eq!(config.file_path, "poem.txt");
    }

    // ch13에서 새로 추가한 테스트입니다. 인자를 하나씩 꺼내게 되면서 무엇이 빠졌는지 따로 알릴 수 있습니다.
    // 반복자라면 무엇이든 받으므로, 문자열 리터럴 배열에 map(String::from)을 붙인 반복자도 넘길 수 있습니다.
    #[test]
    fn build_reports_missing_argument() {
        let only_program = ["minigrep"].into_iter().map(String::from);
        match Config::build(only_program) {
            Ok(_) => panic!("검색어가 없으면 Err여야 합니다"),
            Err(err) => assert_eq!(err, "검색어가 없습니다"),
        }

        let no_file = ["minigrep", "duct"].into_iter().map(String::from);
        match Config::build(no_file) {
            Ok(_) => panic!("파일 경로가 없으면 Err여야 합니다"),
            Err(err) => assert_eq!(err, "파일 경로가 없습니다"),
        }
    }
}
