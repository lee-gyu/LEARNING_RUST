//! 레이아웃 검사가 잡는 어긋남과 놓치는 어긋남을 견줍니다.
//!
//! 실행: cargo run -p er-ch35-bindgen --example 01_layout_checks
//!
//! bindgen이 생성하는 레이아웃 검사는 구조체의 크기, 정렬, 필드마다 이름으로 찾은 오프셋을
//! 확인합니다(예제 02). 헤더의 구조체가 바뀌었는데 Rust 쪽 정의는 낡은 채라면, 그 검사가
//! 원리적으로 무엇을 볼 수 있을까요? 같은 C 구조체를 옮긴 `#[repr(C)]` 정의 둘(바뀐 헤더에
//! 맞는 것과 낡은 것)을 견줘 봅니다. C 코드 없이 "`#[repr(C)]`는 C 컴파일러와 같은 배치 규칙을
//! 따른다"는 사실만 이용합니다. 숫자는 64비트 플랫폼(이 저장소는 macOS aarch64) 기준입니다.
//!
//! 이 예제는 두 정의의 배치를 재기만 하고, 낡은 정의로 C 함수를 부르지 않습니다.

use std::mem::{align_of, size_of};

/// 구조체 하나의 배치: 크기, 정렬, (필드 이름, 오프셋) 목록
struct Layout {
    size: usize,
    align: usize,
    offsets: Vec<(&'static str, usize)>,
}

/// `layout!(타입, 필드, …)`: 타입의 배치를 잽니다. offset_of!는 필드를 이름으로 찾습니다.
macro_rules! layout {
    ($t:ty, $($field:ident),+) => {
        Layout {
            size: size_of::<$t>(),
            align: align_of::<$t>(),
            offsets: vec![$((stringify!($field), std::mem::offset_of!($t, $field))),+],
        }
    };
}

impl Layout {
    fn describe(&self) -> String {
        let fields: Vec<String> = self
            .offsets
            .iter()
            .map(|(n, o)| format!("{n}={o}"))
            .collect();
        format!(
            "크기 {}, 정렬 {}, 오프셋 {}",
            self.size,
            self.align,
            fields.join(" ")
        )
    }
}

/// 바뀐 헤더에서 잰 값(header)으로 낡은 정의(stale)를 검사한다면 어느 검사가 실패하는지 셉니다.
fn compare(title: &str, header: Layout, stale: Layout) {
    println!("{title}");
    println!("   헤더: {}", header.describe());
    println!("   낡음: {}", stale.describe());
    let mut failed = Vec::new();
    if header.size != stale.size {
        failed.push("크기".to_string());
    }
    if header.align != stale.align {
        failed.push("정렬".to_string());
    }
    // 두 정의에 모두 있는 이름만 견줍니다(한쪽에만 있는 필드는 이름으로 찾을 수 없음).
    let moved: Vec<&str> = header
        .offsets
        .iter()
        .filter(|(name, off)| stale.offsets.iter().any(|(n, o)| n == name && o != off))
        .map(|(name, _)| *name)
        .collect();
    if !moved.is_empty() {
        failed.push(format!("오프셋({})", moved.join(", ")));
    }
    if failed.is_empty() {
        println!("   → 크기·정렬·오프셋 검사를 모두 통과: 놓침");
    } else {
        println!("   → 실패하는 검사: {} — 잡힘", failed.join(", "));
    }
}

// 1. 같은 타입의 두 필드가 자리를 바꿈 (C v2의 tally_summary: max가 min 앞으로)
mod case1 {
    #[repr(C)]
    pub struct Header {
        pub count: u64,
        pub below: u64,
        pub above: u64,
        pub max: i64,
        pub min: i64,
    }
    #[repr(C)]
    pub struct Stale {
        pub count: u64,
        pub below: u64,
        pub above: u64,
        pub min: i64,
        pub max: i64,
    }
}

// 2. 크기가 같은 다른 타입으로 바뀜 (uint32_t bucket_width → float bucket_width)
mod case2 {
    #[repr(C)]
    pub struct Header {
        pub low: i64,
        pub bucket_width: f32,
        pub bucket_count: u32,
    }
    #[repr(C)]
    pub struct Stale {
        pub low: i64,
        pub bucket_width: u32,
        pub bucket_count: u32,
    }
}

// 3. 끝 필드가 좁아졌지만 패딩에 가려짐 (uint32_t bucket_count → uint16_t bucket_count)
mod case3 {
    #[repr(C)]
    pub struct Header {
        pub low: i64,
        pub bucket_width: u32,
        pub bucket_count: u16, // 뒤에 패딩 2바이트가 붙어 전체 크기는 그대로 16
    }
    #[repr(C)]
    pub struct Stale {
        pub low: i64,
        pub bucket_width: u32,
        pub bucket_count: u32,
    }
}

// 4. 끝에 필드가 더해짐 (tally_summary에 uint64_t total 추가)
mod case4 {
    #[repr(C)]
    pub struct Header {
        pub count: u64,
        pub below: u64,
        pub above: u64,
        pub min: i64,
        pub max: i64,
        pub total: u64,
    }
    #[repr(C)]
    pub struct Stale {
        pub count: u64,
        pub below: u64,
        pub above: u64,
        pub min: i64,
        pub max: i64,
    }
}

// 5. 앞쪽 필드가 넓어짐 (uint32_t bucket_width → uint64_t bucket_width)
mod case5 {
    #[repr(C)]
    pub struct Header {
        pub low: i64,
        pub bucket_width: u64,
        pub bucket_count: u32,
    }
    #[repr(C)]
    pub struct Stale {
        pub low: i64,
        pub bucket_width: u32,
        pub bucket_count: u32,
    }
}

fn main() {
    compare(
        "1. 같은 타입의 두 필드가 자리를 바꿈 (min과 max)",
        layout!(case1::Header, count, below, above, max, min),
        layout!(case1::Stale, count, below, above, min, max),
    );
    compare(
        "2. 크기가 같은 다른 타입으로 바뀜 (u32 → f32)",
        layout!(case2::Header, low, bucket_width, bucket_count),
        layout!(case2::Stale, low, bucket_width, bucket_count),
    );
    compare(
        "3. 끝 필드가 좁아졌지만 패딩에 가려짐 (u32 → u16)",
        layout!(case3::Header, low, bucket_width, bucket_count),
        layout!(case3::Stale, low, bucket_width, bucket_count),
    );
    compare(
        "4. 끝에 필드가 더해짐 (total 추가)",
        layout!(case4::Header, count, below, above, min, max, total),
        layout!(case4::Stale, count, below, above, min, max),
    );
    compare(
        "5. 앞쪽 필드가 넓어짐 (u32 → u64)",
        layout!(case5::Header, low, bucket_width, bucket_count),
        layout!(case5::Stale, low, bucket_width, bucket_count),
    );
    println!("레이아웃 검사가 보는 것: 크기, 정렬, 이름으로 찾은 오프셋");
    println!("보지 않는 것: 필드의 타입과 폭, #define·enum 상수의 값, 함수 시그니처");
}
