//! C 소스를 컴파일해 정적 라이브러리 libfletcher.a를 만들고 링크를 지시합니다.

use std::fs;

fn main() {
    // C 파일이 바뀔 때만 다시 실행합니다(cc 크레이트는 이 지시문을 대신 출력해 주지 않습니다).
    println!("cargo::rerun-if-changed=csrc/fletcher.c");
    println!("cargo::rerun-if-changed=csrc/fletcher.h");

    // cc가 플랫폼에 맞는 C 컴파일러로 OUT_DIR에 libfletcher.a를 만들고,
    // cargo:rustc-link-lib·cargo:rustc-link-search 지시문을 출력합니다.
    cc::Build::new().file("csrc/fletcher.c").compile("fletcher");

    // 헤더의 버전을 읽어, 이 패키지에 직접 의존하는 패키지의 빌드 스크립트에 넘깁니다.
    let header = fs::read_to_string("csrc/fletcher.h").expect("csrc/fletcher.h");
    let version = header
        .lines()
        .find_map(|l| l.strip_prefix("#define FLETCHER_VERSION "))
        .expect("헤더에 FLETCHER_VERSION이 없음")
        .trim();
    println!("cargo::metadata=version={version}");
}
