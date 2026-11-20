//! C 라이브러리 tally의 선언만 담은 `-sys` 크레이트입니다.
//!
//! - 이 크레이트는 libtally를 링크하고(`build.rs`) bindgen이 헤더에서 만든 선언을 내보낼 뿐,
//!   안전한 API를 만들지 않습니다. 안전한 래퍼는 이 크레이트에 의존하는 `tally`가 맡습니다.
//! - 선언은 두 곳 중 하나에서 옵니다.
//!   - `bindgen` 기능을 켜면: 빌드할 때 헤더에서 새로 생성한 `OUT_DIR/bindings.rs`
//!   - 끄면(기본): 저장소에 커밋해 둔 `src/committed_bindings.rs`
//! - 커밋해 둔 파일이 헤더와 어긋나지 않았는지는 아래 `freshness` 테스트가 확인합니다.

// 헤더의 C 이름을 그대로 쓰므로 Rust의 이름 규칙 린트를 끕니다. 여기서는 #[expect]가 아니라
// #[allow]를 씁니다. 무엇이 생성될지는 헤더가 정하므로, 어느 날 그 린트에 걸리는 이름이
// 하나도 생성되지 않으면 #[expect]는 "기대한 경고가 없다"는 경고를 냅니다.
#![allow(non_camel_case_types, non_upper_case_globals)]

#[cfg(feature = "bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "bindgen"))]
include!("committed_bindings.rs");

/// 커밋해 둔 생성 결과가 지금 헤더에서 새로 생성한 결과와 같은지 확인합니다.
///
/// `cargo test -p tally-sys --features bindgen`으로 실행합니다(libclang이 필요합니다).
/// 헤더나 bindgen의 버전이 바뀌어 결과가 달라지면 실패합니다. 달라진 내용을 검토한 뒤
/// `TALLY_BLESS=1`을 붙여 다시 실행하면 커밋할 파일을 새 결과로 덮어씁니다.
#[cfg(all(test, feature = "bindgen"))]
mod freshness {
    const COMMITTED: &str = include_str!("committed_bindings.rs");
    const FRESH: &str = include_str!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    #[test]
    fn committed_bindings_match_header() {
        if COMMITTED == FRESH {
            return;
        }
        if std::env::var_os("TALLY_BLESS").is_some() {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/committed_bindings.rs");
            std::fs::write(path, FRESH).expect("committed_bindings.rs를 쓰지 못했습니다");
            return;
        }
        // 처음으로 다른 줄을 찾아 보여 줍니다(전체 diff는 git diff로 봅니다).
        let mut committed = COMMITTED.lines();
        let mut fresh = FRESH.lines();
        let mut line = 1;
        loop {
            match (committed.next(), fresh.next()) {
                (Some(a), Some(b)) if a == b => line += 1,
                (a, b) => panic!(
                    "커밋해 둔 생성 결과가 헤더에서 새로 생성한 결과와 다릅니다({line}행)\n  \
                     커밋: {}\n  새로: {}\n\
                     검토한 뒤 TALLY_BLESS=1을 붙여 다시 실행하면 새 결과로 덮어씁니다.",
                    a.unwrap_or("(파일 끝)"),
                    b.unwrap_or("(파일 끝)"),
                ),
            }
        }
    }
}
