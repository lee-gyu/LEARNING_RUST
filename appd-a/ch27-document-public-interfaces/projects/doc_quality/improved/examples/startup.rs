//! 설정 파일을 읽어 서버를 띄우기 직전까지를 흉내 냅니다. `kvconf`를 처음 쓰는 사람이
//! 복사해서 시작할 수 있는 크기로 둡니다.
//!
//! 실행(이 프로젝트 디렉터리에서): cargo run --example startup

fn main() {
    let text = "\
# 개발용 설정
host = 127.0.0.1
port = 8080
";
    match kvconf::parse(text) {
        Ok(config) => {
            for key in config.keys() {
                println!("{key} = {}", config.require(key));
            }
            // 꼭 있어야 하는 값은 require, 빠져도 되는 값은 get과 기본값.
            println!(
                "서버 주소: {}:{}",
                config.require("host"),
                config.require("port")
            );
            println!(
                "제한 시간: {}",
                config.get("timeout").unwrap_or("30초(기본값)")
            );
        }
        Err(e) => println!("설정 파일 오류: {e}"),
    }

    // 둘째 줄에 `=`가 없는 설정 파일.
    let broken = "host = 127.0.0.1\n8080\n";
    if let Err(e) = kvconf::parse(broken) {
        println!("설정 파일 오류: {e}");
    }
}
