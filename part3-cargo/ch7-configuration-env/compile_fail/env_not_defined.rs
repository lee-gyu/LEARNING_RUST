// 컴파일 에러 예제: 컴파일할 때 없는 환경 변수를 `env!`로 읽음
//
// 기대하는 에러: 에러 코드 없음 (`env!` 매크로가 내는 에러에는 E 코드가 붙지 않습니다)
// 핵심 메시지:   error: environment variable `API_TOKEN` not defined at compile time
//                help: use `std::env::var("API_TOKEN")` to read the variable at run time
//
// 확인 (챕터 디렉터리에서, 산출물은 임시 디렉터리로):
//   rustc --edition 2024 --out-dir ../../target/tmp/p3-ch7 compile_fail/env_not_defined.rs
// 셸에 API_TOKEN이 있으면 컴파일이 성공합니다. `env -u API_TOKEN rustc …`로 확인합니다.

fn main() {
    // `env!`는 컴파일하는 순간의 환경에서 값을 읽습니다. 없으면 실행 파일을 만들 수 없습니다.
    // 실행할 때 정할 값이면 `std::env::var`를, 없어도 되는 값이면 `option_env!`를 씁니다.
    let token = env!("API_TOKEN");
    println!("{token}");
}
