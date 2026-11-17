// 스레드 풀로 요청을 동시에 처리하는 웹 서버입니다. (The Book 21장의 완성판)
//
// 실행(이 프로젝트 디렉터리에서):
//   cargo run          # 연결 3개를 처리하면 종료합니다.
//   cargo run -- 10    # 연결 10개를 처리하면 종료합니다.
// 다른 터미널에서 curl http://127.0.0.1:7878/ 처럼 접속하거나 브라우저로 같은 주소를 엽니다.
//
// 경로별 응답:
//   GET /       → 200, hello.html
//   GET /sleep  → 5초 기다린 뒤 200, hello.html (느린 요청 흉내)
//   그 밖       → 404, 404.html

use hello_server::ThreadPool;
use std::{
    env, fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process, thread,
    time::Duration,
};

const ADDRESS: &str = "127.0.0.1:7878";
const POOL_SIZE: usize = 4;
const DEFAULT_CONNECTIONS: usize = 3;

fn main() {
    let max_connections = connection_limit();

    // 7878 포트를 이미 다른 프로그램(예: 아직 끝나지 않은 이전 서버)이 쓰고 있으면 bind가 실패합니다.
    let listener = TcpListener::bind(ADDRESS).unwrap_or_else(|err| {
        eprintln!("{ADDRESS}에서 기다릴 수 없습니다: {err}");
        process::exit(1);
    });

    // 크기가 상수이므로 0일 수 없습니다. 프로그래머의 실수만 패닉하는 new로 충분합니다(ch9).
    let pool = ThreadPool::new(POOL_SIZE);
    println!("http://{ADDRESS} 에서 기다립니다 (연결 {max_connections}개를 처리하면 종료)");

    // incoming()은 끝나지 않는 반복자입니다. take로 정해진 수의 연결만 받고 반복문을 끝냅니다.
    for stream in listener.incoming().take(max_connections) {
        let stream = stream.unwrap();

        // handle_connection이 stream의 소유권을 가져가므로, 이 클로저는 move 없이도 stream을
        // 값으로 캡처합니다. 그래서 execute가 요구하는 FnOnce() + Send + 'static을 모두 만족합니다.
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("연결 {max_connections}개를 받았으므로 서버를 종료합니다.");
} // 여기서 pool이 버려지며 ThreadPool의 Drop이 실행됩니다. 실행 중인 작업이 끝날 때까지 기다립니다.

/// 처리할 연결 수입니다. `cargo run -- 10`처럼 첫 번째 명령줄 인자로 바꿀 수 있습니다.
fn connection_limit() -> usize {
    match env::args().nth(1) {
        None => DEFAULT_CONNECTIONS,
        Some(arg) => arg.parse().unwrap_or_else(|_| {
            eprintln!("연결 수는 0 이상의 정수여야 합니다: {arg}");
            process::exit(1);
        }),
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    // 요청을 보내지 않고 연결을 닫는 클라이언트도 있습니다(예: nc -z 같은 포트 검사).
    // The Book처럼 unwrap하면 그때 작업자 스레드가 패닉하므로, 요청 줄이 없으면 그냥 돌아갑니다.
    let Some(Ok(request_line)) = buf_reader.lines().next() else {
        println!("요청 줄 없이 닫힌 연결을 건너뜁니다");
        return;
    };

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(page_path(filename)).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    println!("응답 완료: {request_line} → {status_line}");
}

/// HTML 파일의 경로를 만듭니다.
///
/// `fs::read_to_string("hello.html")`처럼 상대 경로를 쓰면 프로그램을 **실행한 디렉터리**를
/// 기준으로 찾으므로, 이 프로젝트 디렉터리가 아닌 곳에서 실행하면 파일을 찾지 못합니다.
/// `env!("CARGO_MANIFEST_DIR")`은 컴파일할 때 Cargo가 알려 주는 이 프로젝트 디렉터리의
/// 절대 경로라서, 어디서 실행하든 같은 파일을 가리킵니다.
fn page_path(filename: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(filename)
}
