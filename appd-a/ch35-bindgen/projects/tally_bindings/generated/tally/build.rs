//! 빌드 스크립트: tally-sys가 넘긴 메타데이터를 받아 이 크레이트의 코드에 전합니다.
//!
//! tally-sys의 빌드 스크립트가 `cargo::metadata=c_version=…`처럼 출력한 값은, tally-sys에
//! 직접 의존하는 이 패키지의 빌드 스크립트에 `DEP_<links 값>_<키>` 환경 변수로 들어옵니다.
//! links 값이 "tally"이고 키는 대문자로 바뀌므로 `DEP_TALLY_C_VERSION`이 됩니다.
//! 직접 의존하는 패키지에만 전해지고, 이 크레이트를 쓰는 데모에는 전해지지 않습니다.

use std::env;

fn main() {
    let version = env::var("DEP_TALLY_C_VERSION").expect("tally-sys의 빌드 스크립트가 넘기는 값");
    let declarations =
        env::var("DEP_TALLY_DECLARATIONS").expect("tally-sys의 빌드 스크립트가 넘기는 값");
    // 헤더가 있는 디렉터리(DEP_TALLY_INCLUDE)도 받을 수 있습니다. C 코드를 함께 컴파일하는
    // 크레이트라면 cc::Build::include에 넘겨 같은 헤더를 쓰게 합니다. 이 크레이트는 쓰지 않습니다.

    // 라이브러리 코드가 env!로 읽을 수 있게 넘깁니다.
    println!("cargo::rustc-env=TALLY_C_VERSION={version}");
    println!("cargo::rustc-env=TALLY_DECLARATIONS={declarations}");
}
