// 컴파일 에러 예제: 매크로 이름 뒤의 `!`를 빠뜨림
//
// 기대하는 에러: error[E0423]: expected function, found macro `println`
// 핵심 메시지:   help: use `!` to invoke the macro
//
// 확인 (챕터 디렉터리에서, 산출물은 임시 디렉터리로):
//   rustc --edition 2024 compile_fail/e0423_println_without_bang.rs --out-dir /tmp/rust-ch1
//
// 왜 에러인가: `println`은 함수가 아니라 매크로입니다. 매크로를 부를 때는 이름 뒤에
// 반드시 `!`를 붙여야 합니다. `!`가 없으면 컴파일러는 `println`이라는 함수를 찾는데,
// 그런 함수는 없고 같은 이름의 매크로만 있으므로 "함수를 기대했는데 매크로를 찾았다"고 알립니다.

fn main() {
    println("Hello, world!");
}
