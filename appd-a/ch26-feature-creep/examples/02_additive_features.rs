//! 기능은 더하기만 합니다. 기능을 켜면 함수가 늘어날 뿐, 이미 있던 함수의 동작은 그대로여야 합니다.
//!
//! 실행: cargo run -p er-ch26-feature-creep --example 02_additive_features
//!       cargo run -p er-ch26-feature-creep --example 02_additive_features --features fast-hash
//!
//! 이 챕터 패키지에는 라이브러리가 없어서, 라이브러리였다면 공개했을 함수를 `v1`·`v2` 모듈로
//! 흉내 냅니다. 명령줄에서 `--features fast-hash`를 주는 것은 "빌드 그래프의 다른 크레이트가 이
//! 기능을 켠 상황"을 대신합니다. 기능 통합 때문에 그런 일은 내가 고를 수 없습니다.

/// 기본 빌드로 예전에 계산해서 디스크에 저장해 둔 지문(FNV-1a 32비트)이라고 칩니다.
const SAVED: [(&str, u32); 3] = [
    ("사과", 0xf284_9d82),
    ("배", 0x41b3_74ba),
    ("포도", 0x7a66_ac8b),
];

/// FNV-1a 32비트 해시. 저장 형식이 기대하는 지문입니다.
fn fnv1a(s: &str) -> u32 {
    s.bytes().fold(0x811c_9dc5, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    })
}

/// 더 단순한 지문. 실제 라이브러리라면 더 빠른 알고리즘이 들어갈 자리입니다.
fn poly31(s: &str) -> u32 {
    s.bytes()
        .fold(0, |h: u32, b| h.wrapping_mul(31).wrapping_add(u32::from(b)))
}

// 비권장: 기능이 같은 함수의 동작을 바꿉니다.
mod v1 {
    /// `fast-hash`가 켜지면 다른 알고리즘으로 바뀝니다. 이 함수를 쓰는 크레이트는 그 기능을
    /// 켜지 않았어도, 같은 빌드의 누군가 켜면 다른 값을 받습니다.
    pub fn fingerprint(s: &str) -> u32 {
        if cfg!(feature = "fast-hash") {
            super::poly31(s)
        } else {
            super::fnv1a(s)
        }
    }
}

// 권장: 기능은 새 이름의 함수를 더하기만 합니다.
mod v2 {
    /// 기능과 상관없이 늘 FNV-1a입니다.
    pub fn fingerprint(s: &str) -> u32 {
        super::fnv1a(s)
    }

    /// `fast-hash` 기능이 더하는 함수. 이 함수를 부르는 크레이트는 기능을 직접 켜야 합니다.
    #[cfg(feature = "fast-hash")]
    pub fn fingerprint_fast(s: &str) -> u32 {
        super::poly31(s)
    }
}

/// "사과"의 지문과, 저장해 둔 지문 가운데 `f`로 다시 계산해서 맞는 것의 개수를 출력합니다.
fn check(name: &str, f: fn(&str) -> u32) {
    let apple = f("사과");
    let found = SAVED.iter().filter(|(word, h)| f(word) == *h).count();
    println!("  {name}(\"사과\") = {apple:#010x}");
    println!("  저장해 둔 지문 3개 중 {found}개를 다시 찾음");
}

fn main() {
    println!("이 빌드의 fast-hash 기능: {}", cfg!(feature = "fast-hash")); // 출력: false

    println!("1. 비권장: 기능이 같은 함수의 결과를 바꿈");
    check("v1::fingerprint", v1::fingerprint); // 출력: 0xf2849d82, 3개 중 3개

    println!("2. 권장: fingerprint는 그대로 두고 fingerprint_fast를 더함");
    check("v2::fingerprint", v2::fingerprint); // 출력: 0xf2849d82, 3개 중 3개

    // 기능 전용 함수를 부르는 곳은 #[cfg]로 가립니다. if cfg!(..)로 감싸면 기능이 꺼진
    // 빌드에서도 두 갈래가 모두 컴파일되므로, 없는 함수를 찾지 못해 E0425가 납니다
    // (compile_fail/e0425_cfg_macro_branch.rs).
    #[cfg(feature = "fast-hash")]
    {
        let fast = v2::fingerprint_fast("사과");
        println!("  v2::fingerprint_fast(\"사과\") = {fast:#010x}");
    }
    #[cfg(not(feature = "fast-hash"))]
    println!("  v2::fingerprint_fast는 이 빌드에 없음 (--features fast-hash로 켬)");
}
