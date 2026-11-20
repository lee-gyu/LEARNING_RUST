// 기대하는 에러: 에러 코드 없음(린트 `unsafe_code`를 forbid로 올린 결과)
//   error: usage of an `unsafe` block
//
// `Cargo.toml`에 다음을 적은 패키지에서 unsafe 블록을 쓴 상황입니다.
//
//     [lints.rust]
//     unsafe_code = "forbid"
//
// Cargo는 이 표를 `rustc`의 명령줄 옵션(`-F unsafe-code`)으로 바꿔 패키지의 모든 타깃에
// 넘깁니다. 이 파일은 `rustc`만으로 같은 결과를 보려고 그 설정을 크레이트 속성으로 적었습니다.
// Cargo로 빌드할 때는 note가 `requested on the command line with `-F unsafe-code``로 나옵니다.
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/forbid_unsafe_code.rs

#![forbid(unsafe_code)]

fn main() {
    let value = 7_u8;
    let pointer = &raw const value;
    // 안전한 코드로 쓸 수 있는데도 unsafe를 쓴 예입니다. forbid가 컴파일을 막습니다.
    let copied = unsafe { *pointer };
    println!("{copied}");
}
