//! `std`의 많은 항목은 `core`나 `alloc`에 정의된 것을 다시 내보낸 것입니다. 어느 층에 있는지가
//! 곧 `no_std` 라이브러리에서 쓸 수 있는지를 정합니다.
//!
//! 실행: cargo run -p er-ch33-no-std --example 01_where_items_live
//!
//! 같은 타입인지는 `TypeId`로, 정의된 크레이트는 `type_name`으로 봅니다(Part 1 ch1 2.1절과 같은 방법).
//! `type_name`의 문자열은 문서가 형식을 보장하지 않으므로 구경만 합니다. 아래 출력은 rustc 1.100.0의
//! 결과입니다. `TypeId` 값 자체는 출력하지 않고 비교 결과만 출력합니다.

// `core`와 `std`는 경로에 바로 쓸 수 있지만 `alloc`은 이 선언이 있어야 합니다.
extern crate alloc;

use std::any::{TypeId, type_name};

/// 두 경로가 같은 타입을 가리키는지 비교합니다.
fn same<A: ?Sized + 'static, B: ?Sized + 'static>() -> bool {
    TypeId::of::<A>() == TypeId::of::<B>()
}

fn row(label: &str, is_same: bool) {
    println!("  {label:<44} {is_same}");
}

fn main() {
    // ── 1. core·alloc 경로로도 쓸 수 있는 항목 ─────────────────────────────
    // 괄호 안은 core/alloc 경로로 안정화된 버전입니다(로컬 문서의 안정화 버전 표시). 앞의 셋은
    // 책의 기준 버전(1.70)보다 뒤에 core로 들어왔습니다. 모두 `no_std` 라이브러리에서 쓸 수 있습니다.
    println!("1. std 경로와 core/alloc 경로가 같은 타입인가");
    row(
        "dyn core::error::Error (1.81)",
        same::<dyn core::error::Error, dyn std::error::Error>(),
    ); // 출력:   dyn core::error::Error (1.81)                true
    row(
        "core::net::Ipv4Addr (1.77)",
        same::<core::net::Ipv4Addr, std::net::Ipv4Addr>(),
    ); // 출력:   core::net::Ipv4Addr (1.77)                   true
    row(
        "core::cell::LazyCell (1.80)",
        same::<core::cell::LazyCell<u8>, std::cell::LazyCell<u8>>(),
    ); // 출력:   core::cell::LazyCell (1.80)                  true
    row(
        "core::cell::OnceCell (1.70)",
        same::<core::cell::OnceCell<u8>, std::cell::OnceCell<u8>>(),
    ); // 출력:   core::cell::OnceCell (1.70)                  true
    row(
        "core::ffi::CStr (1.64)",
        same::<core::ffi::CStr, std::ffi::CStr>(),
    ); // 출력:   core::ffi::CStr (1.64)                       true
    row(
        "alloc::ffi::CString (1.64)",
        same::<alloc::ffi::CString, std::ffi::CString>(),
    ); // 출력:   alloc::ffi::CString (1.64)                   true
    row(
        "alloc::collections::TryReserveError (1.57)",
        same::<alloc::collections::TryReserveError, std::collections::TryReserveError>(),
    ); // 출력:   alloc::collections::TryReserveError (1.57)   true

    // ── 2. 정의된 크레이트를 type_name으로 엿보기 ─────────────────────────────
    // 첫 경로 조각이 정의된 크레이트입니다. `std::`로 시작하는 것은 `no_std`에서 쓸 수 없습니다.
    println!("2. type_name이 보여 주는 정의 위치");
    println!("  {}", type_name::<std::time::Duration>()); // 출력:   core::time::Duration
    println!("  {}", type_name::<std::sync::Arc<u8>>()); // 출력:   alloc::sync::Arc<u8>
    println!("  {}", type_name::<std::collections::BTreeMap<u8, u8>>()); // 출력:   alloc::collections::btree::map::BTreeMap<u8, u8>
    // HashMap은 무작위 시드를, Mutex와 Instant는 운영체제를 필요로 하므로 std에만 있습니다.
    println!("  {}", type_name::<std::collections::HashMap<u8, u8>>()); // 출력:   std::collections::hash::map::HashMap<u8, u8>
    println!("  {}", type_name::<std::sync::Mutex<u8>>()); // 출력:   std::sync::poison::mutex::Mutex<u8>
    println!("  {}", type_name::<std::time::Instant>()); // 출력:   std::time::Instant

    // ── 3. 정의 위치만으로는 판단할 수 없는 경우 ─────────────────────────────
    // `std::io`의 일부는 `core::io`에 정의되어 있지만, `core::io` 모듈은 1.100.0에서 nightly 전용
    // (`core_io`)입니다. `no_std` 크레이트에서 `core::io::Write`를 쓰면 E0658입니다
    // (compile_fail/e0658_core_io_unstable.rs). 문서의 "Source" 링크도 같은 이유로 `src/core/…`를 가리킵니다.
    println!("3. core에 정의됐지만 core 경로로는 아직 쓸 수 없는 것");
    println!("  {}", type_name::<std::io::Error>()); // 출력:   core::io::error::Error
    println!("  {}", type_name::<dyn std::io::Write>()); // 출력:   dyn core::io::write::Write
}
