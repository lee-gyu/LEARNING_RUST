// Q3.1 · Q3.8 — 메모리 안전하지만 보안 버그인 코드
//
// 파일 서버가 요청받은 경로를 기준 디렉터리 `/srv/files` 아래의 경로로 바꿉니다. 이 예제는
// 경로를 만들기만 하고 파일 시스템에는 닿지 않습니다.
//
//   [비권장] `Path::join`에 그대로 넘기기: `..`로 기준 디렉터리 밖으로 나가고, 절대 경로는
//            기준 디렉터리를 통째로 바꿉니다(경로 순회, CWE-22).
//   [권장] 경로의 구성 요소가 모두 보통 이름(`Component::Normal`)일 때만 받기
//
// 두 판 모두 안전한 코드이고 경고 없이 컴파일됩니다.
//
// 실행: cargo run -p faq-ch3-safety-limits --example 04_logic_bug_compiles

use std::path::{Component, Path, PathBuf};

const BASE: &str = "/srv/files";

/// 요청 경로를 그대로 이어 붙입니다(비권장).
fn resolve_naive(requested: &str) -> PathBuf {
    Path::new(BASE).join(requested)
}

/// 구성 요소가 모두 보통 이름일 때만 이어 붙입니다. `..`, 루트(`/`), `.`은 거부합니다.
fn resolve_checked(requested: &str) -> Result<PathBuf, String> {
    let path = Path::new(requested);
    if path.components().all(|c| matches!(c, Component::Normal(_))) {
        Ok(Path::new(BASE).join(path))
    } else {
        Err(format!("거부: {requested:?}"))
    }
}

fn main() {
    for requested in ["2026/report.txt", "../../etc/passwd", "/etc/passwd"] {
        println!("요청 {requested:?}");
        println!("  [비권장] {}", resolve_naive(requested).display());
        match resolve_checked(requested) {
            Ok(path) => println!("  [권장]   {}", path.display()),
            Err(e) => println!("  [권장]   {e}"),
        }
    }
    // 출력:
    // 요청 "2026/report.txt"
    //   [비권장] /srv/files/2026/report.txt
    //   [권장]   /srv/files/2026/report.txt
    // 요청 "../../etc/passwd"
    //   [비권장] /srv/files/../../etc/passwd
    //   [권장]   거부: "../../etc/passwd"
    // 요청 "/etc/passwd"
    //   [비권장] /etc/passwd
    //   [권장]   거부: "/etc/passwd"
    //
    // 두 번째 경로를 운영체제가 해석하면 /etc/passwd가 됩니다. 세 번째는 `join`의 문서대로
    // 절대 경로가 기준 경로를 대신했습니다. 어느 쪽도 메모리 오류가 아니므로 컴파일러는 막지 않습니다.
}
