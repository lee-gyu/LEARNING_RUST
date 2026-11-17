// minigrep 4단계: 실제 일을 하는 로직을 run 함수로 빼고, 파일 읽기 실패도 패닉 대신
// Result로 돌려줍니다. main에는 "설정 만들기 → run 부르기 → 에러면 알리고 종료"만 남습니다.
//
// 실행 (저장소 루트에서):
//   cargo run -q -p p0-ch12-io-project-minigrep --example 04_run -- the part0-getting-started/ch12-io-project-minigrep/poem.txt
//   cargo run -q -p p0-ch12-io-project-minigrep --example 04_run -- the no_such_file.txt; echo "종료 코드: $?"
//   cargo run -q -p p0-ch12-io-project-minigrep --example 04_run > output.txt   (에러 메시지가 파일로 들어가는 문제 관찰)

use std::env;
use std::error::Error;
use std::fs;
use std::process;

struct Config {
    query: String,
    file_path: String,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("인자가 부족합니다");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        Ok(Config { query, file_path })
    }
}

// 성공하면 돌려줄 값이 따로 없으므로 Ok 쪽은 유닛 타입 ()입니다.
// 실패 쪽 Box<dyn Error>는 "Error 트레이트를 구현한 타입이면 무엇이든"이라는 뜻입니다.
// 지금은 io::Error 하나뿐이지만, 나중에 다른 종류의 에러가 늘어나도 시그니처를 바꾸지 않아도 됩니다.
// (dyn과 트레이트 객체는 ch17에서 다룹니다.)
fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("검색어: {}", config.query); // 출력: 검색어: the
    println!("파일: {}", config.file_path); // 출력: 파일: part0-getting-started/ch12-io-project-minigrep/poem.txt

    // expect 대신 ?를 씁니다. 읽기에 실패하면 io::Error를 Box<dyn Error>로 바꿔
    // 곧바로 호출한 쪽(main)에 돌려줍니다. 패닉을 낼지, 어떤 메시지를 보여 줄지는 main이 정합니다.
    let contents = fs::read_to_string(config.file_path)?;

    println!("파일 내용:\n{contents}"); // 출력: 파일 내용: 다음에 poem.txt 전체

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = match Config::build(&args) {
        Ok(config) => config,
        Err(err) => {
            // 아직 println!(표준 출력)입니다. 이 예제를 `> output.txt`로 실행하면 이 메시지가
            // 화면에 보이지 않고 파일로 들어갑니다. projects/minigrep에서 eprintln!으로 고칩니다.
            println!("인자 문제: {err}");
            println!("사용법: minigrep <검색어> <파일 경로>");
            process::exit(1);
        }
    };

    // run은 성공하면 () 말고는 돌려줄 값이 없으므로, Ok에서 꺼낼 것이 없습니다.
    // 그래서 match 대신 Err인 경우만 골라내는 if let을 씁니다.
    // e는 Box<dyn Error>이고 {}로 출력하면 원래 에러(io::Error)의 메시지가 나옵니다.
    if let Err(e) = run(config) {
        println!("실행 오류: {e}"); // 출력: 실행 오류: No such file or directory (os error 2) (파일이 없는 경우)
        process::exit(1);
    }
}
