//! C 프로그램이 Rust 함수를 부르는 모습을 cargo만으로 확인하는 실행 파일입니다.
//!
//! 실제로는 Rust 쪽을 정적 라이브러리(staticlib)나 동적 라이브러리(cdylib)로 빌드해 C 프로그램에
//! 링크합니다(크레이트 타입은 Part 9 ch9-memory-panic-linkage). 여기서는 build.rs가 C 코드
//! (csrc/driver.c)를 컴파일해 이 실행 파일에 함께 링크하고, main은 C 쪽 진입점 하나만 부릅니다.
//!
//! 실행: cargo run -q (이 디렉터리에서)

mod textstat;

use std::process::ExitCode;

unsafe extern "C" {
    // int c_program_main(void);  (csrc/driver.c)
    // 인자는 없지만 C 코드가 하는 일을 컴파일러가 확인하지 못하므로 unsafe fn으로 둡니다.
    unsafe fn c_program_main() -> i32;
}

fn main() -> ExitCode {
    // SAFETY: c_program_main은 인자를 받지 않고, textstat.h의 규칙대로(핸들은 한 번만 해제,
    // 버퍼 길이는 실제 크기) textstat_* 함수를 부릅니다. driver.c를 읽어 확인했습니다.
    let status = unsafe { c_program_main() };
    if status == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
