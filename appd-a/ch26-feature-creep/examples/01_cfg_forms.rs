//! `#[cfg]` 속성, `cfg!` 매크로, `cfg_select!`, `#[cfg_attr]`로 플랫폼마다 다른 코드를 쓰는 법.
//! 지침: 빠진 플랫폼이 없게 나머지 갈래를 두고, 값만 다르면 `cfg!`로 한 벌만 씁니다.
//!
//! 실행: cargo run -p er-ch26-feature-creep --example 01_cfg_forms
//!
//! 1번과 2번 구역의 값은 빌드한 플랫폼에 따라 다릅니다. 주석의 출력은 macOS(aarch64)에서
//! 실행한 결과입니다. 4번 구역은 `--release`로 실행하면 달라집니다.

/// cfg 조건을 글자 그대로 적고, 그 조건이 이 빌드에서 참인지를 한 줄로 출력합니다.
/// `show!(unix)`는 `println!("{} {}", "unix", cfg!(unix))`가 됩니다.
macro_rules! show {
    ($($pred:tt)*) => {
        println!("  {:<28} {}", stringify!($($pred)*), cfg!($($pred)*))
    };
}

// ---------------------------------------------------------------------------
// 2번 구역. 비권장: 플랫폼마다 함수를 한 벌씩 따로 정의합니다.
// ---------------------------------------------------------------------------
mod v1 {
    // 목록에 없는 플랫폼(FreeBSD, wasm 등)에서는 `config_dir` 자체가 없습니다. 이 함수를
    // 부르는 코드는 그 플랫폼에서 E0425로 컴파일되지 않습니다. 또 macOS에서 빌드하는 동안
    // 아래의 linux 판과 windows 판은 컴파일러가 보지도 않습니다.
    #[cfg(target_os = "macos")]
    pub fn config_dir() -> &'static str {
        "~/Library/Application Support"
    }

    #[cfg(target_os = "linux")]
    pub fn config_dir() -> &'static str {
        "~/.config"
    }

    #[cfg(windows)]
    pub fn config_dir() -> &'static str {
        "%APPDATA%"
    }
}

// ---------------------------------------------------------------------------
// 2번 구역. 권장: 나머지 갈래를 두고, 값만 다르면 한 벌로 씁니다.
// ---------------------------------------------------------------------------
mod v2 {
    /// `cfg_select!`(1.95)는 위에서부터 처음 참인 갈래 하나만 남깁니다. `_` 갈래가 목록에 없는
    /// 플랫폼을 받으므로, 어떤 플랫폼에서든 이 함수가 있습니다.
    pub fn config_dir() -> &'static str {
        cfg_select! {
            target_os = "macos" => { "~/Library/Application Support" }
            target_os = "linux" => { "~/.config" }
            windows => { "%APPDATA%" }
            _ => { "./config" }
        }
    }

    /// 값만 다르면 `cfg!`로 충분합니다. `cfg!`는 `true`/`false` 리터럴이 될 뿐이라 두 갈래가
    /// 모두 컴파일되고, 어느 플랫폼에서 빌드하든 두 갈래 모두 타입 검사를 받습니다.
    pub fn line_ending() -> &'static str {
        if cfg!(windows) { "\r\n" } else { "\n" }
    }
}

// ---------------------------------------------------------------------------
// 3번 구역. #[cfg]로 지운 항목은 타입 검사를 받지 않습니다.
// ---------------------------------------------------------------------------
// 본문의 타입이 맞지 않지만(`&str`을 `u32`로 돌려줌) 이 파일은 경고 없이 빌드됩니다.
// `cfg(false)`(1.88)는 늘 거짓인 조건입니다. `#[cfg(windows)]` 아래의 코드도 macOS에서는
// 이렇게 지워지므로, 그 안의 실수는 Windows에서 빌드하기 전까지 드러나지 않습니다.
#[cfg(false)]
fn never_checked() -> u32 {
    "이 본문은 컴파일러가 보기 전에 지워집니다"
}

// ---------------------------------------------------------------------------
// 4번 구역. #[cfg_attr(조건, 속성)]은 조건이 참일 때만 속성을 붙입니다.
// ---------------------------------------------------------------------------
// 여기서는 디버그 빌드(debug_assertions)에서만 Debug 구현을 붙입니다. 기능과 함께 쓰면
// "기능이 켜졌을 때만 derive를 붙인다"는 모양이 됩니다(`cfg_attr(feature = "...", derive(...))`).
#[cfg_attr(debug_assertions, derive(Debug))]
struct BuildInfo {
    debug: bool,
    pointer_bits: u32,
}

fn main() {
    println!("1. 툴체인이 정해 주는 cfg 값 (플랫폼마다 다름)");
    // 각 줄의 출력은 조건과 true/false입니다. 주석에는 true/false만 적었습니다.
    show!(target_os = "macos"); // 출력: true
    show!(target_os = "linux"); // 출력: false
    show!(unix); // 출력: true
    show!(windows); // 출력: false
    show!(target_arch = "aarch64"); // 출력: true
    show!(target_pointer_width = "64"); // 출력: true
    show!(target_endian = "little"); // 출력: true
    show!(debug_assertions); // 출력: true (--release로 실행하면 false)
    show!(panic = "unwind"); // 출력: true

    println!("2. 플랫폼마다 다른 값");
    // 비권장 판은 부르는 쪽도 같은 조건을 되풀이해야 목록 밖 플랫폼에서 빌드가 깨지지 않습니다.
    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    println!("  비권장 v1::config_dir() = {}", v1::config_dir()); // 출력: ~/Library/Application Support
    #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
    println!("  비권장 v1::config_dir()는 이 플랫폼에 없습니다");
    // 권장 판은 조건 없이 부릅니다.
    println!("  권장 v2::config_dir()   = {}", v2::config_dir()); // 출력: ~/Library/Application Support
    println!("  권장 v2::line_ending()  = {:?}", v2::line_ending()); // 출력: "\n"

    println!("3. #[cfg(false)] 아래의 타입 에러는 빌드를 막지 않았습니다");

    let info = BuildInfo {
        debug: cfg!(debug_assertions),
        pointer_bits: usize::BITS,
    };
    let (debug, bits) = (info.debug, info.pointer_bits);
    println!("4. cfg_attr: 디버그 빌드 {debug}, 포인터 {bits}비트"); // 출력: 디버그 빌드 true, 포인터 64비트
    #[cfg(debug_assertions)]
    println!("  Debug 구현이 있습니다: {info:?}"); // 출력: BuildInfo { debug: true, pointer_bits: 64 }
    #[cfg(not(debug_assertions))]
    println!("  Debug 구현이 없습니다");
}
