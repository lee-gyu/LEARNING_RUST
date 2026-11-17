//! minigrep의 라이브러리 크레이트입니다.
//!
//! 프로그램의 로직(설정 해석, 실행, 검색)은 모두 여기에 둡니다. 바이너리 크레이트
//! (src/main.rs)는 이 라이브러리를 `minigrep::...` 경로로 불러 쓰기만 합니다.
//! 로직이 라이브러리에 있으면 #[test]로 검사하기 쉽고, 다른 크레이트도 가져다 쓸 수 있습니다.

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
    /// 명령줄 인자(args[0]은 실행 파일 경로)와 환경 변수로 설정을 만듭니다.
    ///
    /// 인자가 모자라면 패닉 대신 Err를 돌려줍니다. 사용자가 충분히 저지를 수 있는
    /// 실수이므로, 어떻게 알릴지는 호출한 쪽(main)이 정하게 합니다.
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("인자가 부족합니다");
        }

        // args는 빌려 온 슬라이스라서 안의 String을 옮겨 올 수 없으므로 복제합니다.
        let query = args[1].clone();
        let file_path = args[2].clone();

        // env::var는 변수가 있으면 Ok(값), 없으면 Err를 돌려줍니다.
        // 값은 보지 않고 "설정돼 있는가"만 따지므로 is_ok()로 bool을 얻습니다.
        // 그래서 IGNORE_CASE=0이나 빈 값으로 설정해도 true가 됩니다.
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
/// 파일을 읽지 못하면 그 에러를 Box<dyn Error>에 담아 돌려줍니다.
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
/// 돌려주는 &str은 contents의 일부를 가리키는 슬라이스입니다. 그래서 반환값의
/// 라이프타임을 contents와 같은 'a로 묶습니다. query와는 상관이 없으므로
/// query에는 'a를 붙이지 않습니다.
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();

    // lines()는 문자열을 줄 단위로 나눠 하나씩 꺼내 줍니다. 각 줄은 contents를
    // 가리키는 &str이며, 줄 끝의 개행 문자는 들어 있지 않습니다.
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }

    results
}

/// search와 같지만 대소문자를 구분하지 않습니다.
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    // to_lowercase()는 소문자로 바꾼 새 String을 만듭니다. 검색어는 한 번만 바꾸면 되므로
    // 반복문 밖에서 바꾸고, 같은 이름으로 섀도잉합니다(이제 query의 타입은 String).
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for line in contents.lines() {
        // contains는 &str 같은 패턴을 받으므로 String인 query를 &query로 빌려 넘깁니다.
        // 결과에는 소문자로 바꾼 줄이 아니라 원래 줄(line)을 넣어 원문 그대로 출력되게 합니다.
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }

    results
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

    #[test]
    fn build_needs_query_and_file_path() {
        // main에서 하던 인자 검사가 라이브러리 함수로 옮겨 왔으므로, 이것도 테스트할 수 있습니다.
        let args = [String::from("minigrep"), String::from("duct")];

        assert!(Config::build(&args).is_err());
    }

    #[test]
    fn build_reads_query_and_file_path() {
        let args = [
            String::from("minigrep"),
            String::from("duct"),
            String::from("poem.txt"),
        ];

        let config = Config::build(&args).unwrap();

        assert_eq!(config.query, "duct");
        assert_eq!(config.file_path, "poem.txt");
    }
}
