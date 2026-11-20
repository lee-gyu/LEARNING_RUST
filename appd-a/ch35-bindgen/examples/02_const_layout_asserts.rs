//! bindgen이 생성하는 레이아웃 검사를 손으로 흉내 냅니다. 검사는 컴파일할 때 평가됩니다.
//!
//! 실행: cargo run -p er-ch35-bindgen --example 02_const_layout_asserts
//!
//! bindgen 0.73.2는 구조체마다 `const _: () = { … };` 블록을 생성합니다. 그 안의 식은 상수로
//! 평가되므로, 값이 헤더와 다르면 프로그램이 실행되기 전에 컴파일이 실패합니다
//! (`compile_fail/e0080_layout_assert_stale.rs`). 이 파일이 컴파일되어 실행된다는 것 자체가
//! 아래 검사를 모두 통과했다는 뜻입니다. 숫자는 64비트 플랫폼 기준입니다.

use std::mem::{align_of, offset_of, size_of};

// typedef struct tally_config { int64_t low; uint32_t bucket_width; uint32_t bucket_count; } tally_config;
// C 이름을 그대로 씁니다. rustc의 non_camel_case_types 린트는 #[repr(C)] 타입에는 경고하지 않습니다.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct tally_config {
    pub low: i64,
    pub bucket_width: u32,
    pub bucket_count: u32,
}

// 1) bindgen 0.73.2가 projects/의 tally.h에서 생성한 검사를 그대로 옮긴 모양입니다.
//    값이 같으면 `크기 - 16`이 0이 되어 원소가 하나뿐인 배열의 0번을 읽고 끝납니다.
//    값이 크면 범위를 벗어난 인덱스, 작으면 뺄셈 오버플로가 되어 상수 평가가 실패합니다.
//    배열의 원소인 문자열은 실패했을 때 에러 메시지 옆에 보이는 설명 역할을 합니다.
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of tally_config"][::std::mem::size_of::<tally_config>() - 16usize];
    ["Alignment of tally_config"][::std::mem::align_of::<tally_config>() - 8usize];
    ["Offset of field: tally_config::low"][::std::mem::offset_of!(tally_config, low) - 0usize];
    ["Offset of field: tally_config::bucket_width"]
        [::std::mem::offset_of!(tally_config, bucket_width) - 8usize];
    ["Offset of field: tally_config::bucket_count"]
        [::std::mem::offset_of!(tally_config, bucket_count) - 12usize];
};

// 2) 손으로 쓴 선언에 붙인다면 assert!가 읽기 쉽습니다(상수 문맥의 assert!는 Rust 1.57부터).
//    숫자의 출처가 중요합니다. 바뀐 헤더에서 다시 재지 않은 숫자라면 낡은 정의와 함께 낡습니다.
const _: () = {
    assert!(
        size_of::<tally_config>() == 16,
        "tally_config의 크기가 헤더와 다릅니다"
    );
    assert!(
        align_of::<tally_config>() == 8,
        "tally_config의 정렬이 헤더와 다릅니다"
    );
    assert!(
        offset_of!(tally_config, low) == 0,
        "low의 오프셋이 헤더와 다릅니다"
    );
    assert!(
        offset_of!(tally_config, bucket_width) == 8,
        "bucket_width의 오프셋이 다릅니다"
    );
    assert!(
        offset_of!(tally_config, bucket_count) == 12,
        "bucket_count의 오프셋이 다릅니다"
    );
};

/// 검사한 항목: (설명, 실제 값, 헤더의 값). 실행 중 출력용으로 같은 값을 다시 모읍니다.
const CHECKS: [(&str, usize, usize); 5] = [
    ("크기", size_of::<tally_config>(), 16),
    ("정렬", align_of::<tally_config>(), 8),
    ("low의 오프셋", offset_of!(tally_config, low), 0),
    (
        "bucket_width의 오프셋",
        offset_of!(tally_config, bucket_width),
        8,
    ),
    (
        "bucket_count의 오프셋",
        offset_of!(tally_config, bucket_count),
        12,
    ),
];

fn main() {
    println!("tally_config의 레이아웃 검사 (모두 컴파일할 때 통과함)");
    for (what, actual, header) in CHECKS {
        println!("   {what}: Rust {actual}, 헤더 {header}");
    }
    // 실제로 값을 만들어 필드를 읽어 봅니다(검사와 상관없이, 구조체가 정상인지 확인).
    let c = tally_config {
        low: -5,
        bucket_width: 10,
        bucket_count: 4,
    };
    println!(
        "   값: low={} bucket_width={} bucket_count={}",
        c.low, c.bucket_width, c.bucket_count
    );
    println!("검사가 확인하는 것: Rust의 #[repr(C)] 배치가 생성할 때 잰 C의 배치와 같은가");
    println!("확인하지 않는 것: 그 숫자를 잰 뒤에 헤더가 바뀌었는가");
}
