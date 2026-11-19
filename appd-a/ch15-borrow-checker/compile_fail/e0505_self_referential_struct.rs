// 기대하는 에러: E0505
//   error[E0505]: cannot move out of `line` because it is borrowed
//
// 원문과, 원문을 빌린 토큰을 한 구조체에 담으려는 시도입니다(자기 참조 구조체).
// 토큰 `program`은 지역 변수 `line`을 빌렸는데, 구조체를 만들면서 `line`을 구조체 안으로
// 옮깁니다. 옮긴 뒤에는 구조체가 어디로 가든 따라가야 하는데, 참조는 옮기기 전의
// 자리(`line`)를 가리키므로 대여 검사기는 이동 자체를 거부합니다.
// 라이프타임 매개변수 'a에 "구조체 자기 자신"을 적을 방법도 없습니다.
//
// 확인: rustc --edition 2024 compile_fail/e0505_self_referential_struct.rs --out-dir /tmp/er-ch15

struct ParsedCommand<'a> {
    line: String,
    program: &'a str,
}

fn main() {
    let line = String::from("cp 사진.jpg 백업/");
    let program = line.split_whitespace().next().unwrap_or("");

    let cmd = ParsedCommand { line, program }; // E0505: 빌려준 line을 구조체로 옮김

    println!("{} / {}", cmd.line, cmd.program);
}
