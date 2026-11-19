// 기대하는 에러(기능이 꺼진 빌드): E0425 cannot find function `fingerprint_fast` in this scope
//   note: found an item that was configured out
//         ... the item is gated behind the `fast-hash` feature
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0425_cfg_macro_branch.rs
//     → E0425로 실패합니다.
//   rustc --edition 2024 --out-dir <임시 디렉터리> --cfg 'feature="fast-hash"' compile_fail/e0425_cfg_macro_branch.rs
//     → 컴파일됩니다. `--cfg 'feature="fast-hash"'`는 Cargo가 `--features fast-hash`일 때
//       rustc에 넘기는 인자와 같습니다.
//
// `cfg!`는 조건을 `true`/`false` 리터럴로 바꿀 뿐 코드를 지우지 않습니다. 기능이 꺼진 빌드에서도
// `if`의 두 갈래가 모두 컴파일되므로, `#[cfg]`로 지워진 함수를 부르는 갈래에서 에러가 납니다.
// 고치려면 부르는 쪽도 `#[cfg(feature = "fast-hash")]`로 가립니다(examples/02_additive_features.rs).

#[cfg(feature = "fast-hash")]
fn fingerprint_fast(s: &str) -> u32 {
    s.bytes()
        .fold(0, |h: u32, b| h.wrapping_mul(31).wrapping_add(u32::from(b)))
}

fn main() {
    if cfg!(feature = "fast-hash") {
        println!("{:#010x}", fingerprint_fast("사과"));
    } else {
        println!("fast-hash 기능이 꺼져 있습니다");
    }
}
