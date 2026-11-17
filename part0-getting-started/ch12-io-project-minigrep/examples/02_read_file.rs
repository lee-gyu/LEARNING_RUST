// minigrep 2단계: 인자를 변수에 담고, 두 번째 인자로 받은 경로의 파일을 읽습니다.
// 일부러 아무 검사도 하지 않은 "첫 번째 초안"이라서, 잘못 실행하면 패닉으로 끝납니다.
//
// 실행 (저장소 루트에서):
//   cargo run -q -p p0-ch12-io-project-minigrep --example 02_read_file -- the part0-getting-started/ch12-io-project-minigrep/poem.txt
//   cargo run -q -p p0-ch12-io-project-minigrep --example 02_read_file             (인자 없음: 패닉, 종료 코드 101)
//   cargo run -q -p p0-ch12-io-project-minigrep --example 02_read_file -- the no_such_file.txt  (파일 없음: 패닉, 종료 코드 101)

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 인자에 이름을 붙여 두면 아래 코드가 무엇을 쓰는지 읽기 쉬워집니다.
    // args가 String들의 소유자이므로 여기서는 &로 빌려 오기만 합니다.
    // 길이를 확인하지 않았기 때문에, 인자가 모자라면 이 줄에서
    // "index out of bounds" 패닉이 납니다. 3단계(03_config)에서 고칩니다.
    let query = &args[1];
    let file_path = &args[2];

    println!("검색어: {query}"); // 출력: 검색어: the
    println!("파일: {file_path}"); // 출력: 파일: part0-getting-started/ch12-io-project-minigrep/poem.txt

    // fs::read_to_string은 파일 전체를 읽어 String으로 돌려주며, 결과 타입은
    // std::io::Result<String>입니다. 상대 경로는 소스 파일 위치가 아니라 프로그램을
    // 실행한 현재 디렉터리를 기준으로 찾습니다.
    // expect는 실패하면 이 메시지와 함께 패닉을 냅니다. 원인(파일 없음, 권한 없음 등)이
    // 무엇이든 같은 메시지가 앞에 붙으므로 사용자에게 친절한 처리는 아닙니다.
    let contents = fs::read_to_string(file_path).expect("파일을 읽을 수 있어야 합니다");

    // 출력: 파일 내용:
    //       I'm nobody! Who are you?
    //       ... (poem.txt 전체, 9줄)
    println!("파일 내용:\n{contents}");
}
