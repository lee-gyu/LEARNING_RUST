// 세 가지 출처(crates.io, git 저장소, 로컬 경로)의 의존성을 함께 쓰는 프로그램입니다.
// 코드에서는 세 크레이트를 쓰는 방법이 똑같습니다. 출처는 Cargo.toml만 압니다.
//
// 실행: cargo run -q   (projects/three-sources 디렉터리에서)

// crates.io에서 온 크레이트: 이름 표기를 바꾸는 트레이트
use heck::ToKebabCase;

// git 저장소에서 온 크레이트: `#[cfg]` 갈래를 if/else 모양으로 쓰게 해 주는 매크로
cfg_if::cfg_if! {
    if #[cfg(unix)] {
        const PLATFORM: &str = "유닉스 계열";
    } else if #[cfg(windows)] {
        const PLATFORM: &str = "Windows";
    } else {
        const PLATFORM: &str = "그 밖의 플랫폼";
    }
}

fn main() {
    let name = "ThreeSources".to_kebab_case();
    // 로컬 경로에서 온 크레이트
    println!("{}", frame::boxed(&format!("{name}: 세 가지 출처")));
    println!("플랫폼: {PLATFORM}");
}
