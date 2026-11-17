// 컴파일 에러 예제: 서식 문자열의 `{}` 개수와 인자 개수가 다름
//
// 기대하는 에러: 에러 코드 없음 (서식 문자열 검사 에러에는 E 코드가 붙지 않습니다)
// 핵심 메시지:   error: 2 positional arguments in format string, but there is 1 argument
//
// 확인 (챕터 디렉터리에서, 산출물은 임시 디렉터리로):
//   rustc --edition 2024 compile_fail/format_args_mismatch.rs --out-dir /tmp/rust-ch1
//
// 왜 에러인가: `println!`은 매크로이므로 컴파일할 때 서식 문자열을 읽어 `{}`가 몇 개인지
// 셉니다. 자리표시자는 두 개인데 채울 값이 하나뿐이면, 실행해 보기 전에 컴파일 단계에서
// 바로 거부됩니다. C의 `printf`처럼 실행 중에 엉뚱한 값을 찍는 일이 생기지 않습니다.

fn main() {
    let name = "페리스";
    println!("{}님, {}번째 방문을 환영합니다!", name);
}
