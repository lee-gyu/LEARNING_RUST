// 기대하는 에러(std 기능이 꺼진 빌드): E0425 cannot find type `Vec` in this scope
//   help: consider importing this struct
//         use alloc::vec::Vec;
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> --cfg 'feature="std"' compile_fail/e0425_std_feature_prelude.rs
//     → 컴파일됩니다(std 기능을 켠 빌드).
//   rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0425_std_feature_prelude.rs
//     → E0425로 실패합니다(std 기능을 끄고 alloc만 쓰는 빌드).
//   `--cfg 'feature="std"'`는 Cargo가 `--features std`일 때 rustc에 넘기는 인자와 같습니다.
//
// `#![cfg_attr(not(feature = "std"), no_std)]`는 std 기능이 켜지면 크레이트를 보통의 std 크레이트로
// 만듭니다. 그 빌드에서는 std의 프렐류드가 있어서 `Vec`을 가져오지 않아도 컴파일되므로, 기본 기능
// (std)으로만 빌드하고 테스트하면 이 실수가 드러나지 않습니다. 크레이트 루트를 늘 `#![no_std]`로 두고
// std는 `#[cfg(feature = "std")] extern crate std;`로 연결하면, 모든 빌드가 같은 core 프렐류드를 써서
// 기본 빌드에서도 바로 E0425가 납니다(projects/tinyhex/src/lib.rs).

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// 0이 아닌 측정값만 남깁니다.
pub fn nonzero(readings: &[u16]) -> Vec<u16> {
    readings.iter().copied().filter(|r| *r != 0).collect()
}
