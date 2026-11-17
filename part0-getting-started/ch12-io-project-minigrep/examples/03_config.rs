// minigrep 3단계: 설정 값을 Config 구조체로 묶고, 인자 검사를 Config::build로 옮깁니다.
// 인자가 모자라면 패닉 대신 Err를 돌려주고, main이 사용법을 알린 뒤 종료 코드 1로 끝냅니다.
//
// 실행 (저장소 루트에서):
//   cargo run -q -p p0-ch12-io-project-minigrep --example 03_config -- the part0-getting-started/ch12-io-project-minigrep/poem.txt
//   cargo run -q -p p0-ch12-io-project-minigrep --example 03_config; echo "종료 코드: $?"

use std::env;
use std::fs;
use std::process;

// query와 file_path는 "프로그램을 어떻게 실행할지"를 정하는 설정 값입니다.
// 구조체 하나로 묶으면 두 값이 한 묶음이라는 것이 타입에 드러나고, 함수에 넘길 때도 하나로 넘깁니다.
struct Config {
    query: String,
    file_path: String,
}

impl Config {
    // 이름이 new가 아니라 build인 이유: new는 실패하지 않는다고 기대하는 관례가 있습니다.
    // 이 함수는 사용자의 입력에 따라 실패할 수 있으므로 Result를 돌려줍니다.
    // 에러 타입 &'static str은 문자열 리터럴의 타입입니다. 리터럴은 프로그램이 끝날 때까지
    // 살아 있으므로 라이프타임이 'static입니다.
    fn build(args: &[String]) -> Result<Config, &'static str> {
        // 인자 부족은 프로그램의 버그가 아니라 사용자가 충분히 저지를 수 있는 실수입니다.
        // 그래서 panic!이 아니라 Err로 알리고, 어떻게 알릴지는 호출한 쪽(main)이 정합니다.
        if args.len() < 3 {
            return Err("인자가 부족합니다");
        }

        // args는 &[String]으로 빌려 온 것이라서 안의 String을 Config로 옮길(move) 수 없습니다.
        // clone()으로 복제본을 만들어 Config가 자기 값을 소유하게 합니다.
        // 복제는 비용이 들지만 인자 두 개를 한 번 복사하는 정도는 문제 되지 않고,
        // 덕분에 Config에 참조와 라이프타임을 넣지 않아도 됩니다.
        let query = args[1].clone();
        let file_path = args[2].clone();

        Ok(Config { query, file_path })
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Ok면 안의 Config를 꺼내고, Err면 메시지를 알린 뒤 프로그램을 끝냅니다.
    // process::exit는 그 자리에서 프로그램을 끝내고 인자로 준 숫자를 종료 코드로 돌려줍니다.
    // 0은 성공, 0이 아닌 값은 실패라는 관례가 있어서 셸이나 다른 프로그램이 결과를 판단할 수 있습니다.
    // process::exit는 돌아오지 않는(흐름을 끊는) 함수라서 이 갈래는 Config 값이 없어도 됩니다.
    let config = match Config::build(&args) {
        Ok(config) => config,
        Err(err) => {
            // 지금은 println!(표준 출력)으로 알립니다. 결과를 파일로 저장할 때 이 메시지까지
            // 파일에 들어가는 문제가 있는데, 마지막 단계(projects/minigrep)에서 고칩니다.
            println!("인자 문제: {err}"); // 출력: 인자 문제: 인자가 부족합니다 (인자 없이 실행한 경우)
            println!("사용법: minigrep <검색어> <파일 경로>");
            process::exit(1);
        }
    };

    println!("검색어: {}", config.query); // 출력: 검색어: the
    println!("파일: {}", config.file_path); // 출력: 파일: part0-getting-started/ch12-io-project-minigrep/poem.txt

    // 파일 읽기는 아직 expect로 처리합니다. 4단계(04_run)에서 고칩니다.
    let contents = fs::read_to_string(&config.file_path).expect("파일을 읽을 수 있어야 합니다");

    println!("파일 내용:\n{contents}"); // 출력: 파일 내용: 다음에 poem.txt 전체
}
