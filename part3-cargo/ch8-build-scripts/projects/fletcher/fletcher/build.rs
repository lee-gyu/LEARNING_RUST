//! fletcher-sys의 빌드 스크립트가 넘긴 C 라이브러리 버전에 따라 cfg를 켭니다.

use std::env;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    // 직접 지은 cfg 이름은 알려 둬야 unexpected_cfgs 경고가 나지 않습니다.
    println!("cargo::rustc-check-cfg=cfg(fletcher_v2)");

    // fletcher-sys(links = "fletcher")가 cargo::metadata=version=…으로 넘긴 값이
    // DEP_<links 값>_<키> 이름으로 들어옵니다.
    let version: u32 = env::var("DEP_FLETCHER_VERSION")
        .expect("fletcher-sys가 version을 넘기지 않음")
        .parse()
        .expect("version이 정수가 아님");
    if version >= 2 {
        println!("cargo::rustc-cfg=fletcher_v2");
    }
    println!("cargo::rustc-env=FLETCHER_C_VERSION={version}");
}
