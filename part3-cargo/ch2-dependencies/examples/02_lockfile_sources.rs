//! `Cargo.lock`의 `source` 줄로 의존성의 출처를 구별합니다.
//!
//! `projects/three-sources`는 crates.io, git 저장소, 로컬 경로에서 의존성을 하나씩 가져옵니다.
//! `Cargo.toml`에는 사람이 적은 "어디서 무엇을"이 있고, `Cargo.lock`에는 Cargo가 실제로
//! 고른 버전과 커밋이 있습니다. 이 예제는 그 잠금 파일을 읽어 패키지마다 한 줄로 요약합니다.
//!
//! 실행: cargo run -p p3-ch2-dependencies --example 02_lockfile_sources

/// 컴파일할 때 잠금 파일의 내용을 문자열로 넣습니다. 실행할 때 파일을 찾을 필요가 없습니다.
const LOCKFILE: &str = include_str!("../projects/three-sources/Cargo.lock");

/// `key = "value"` 줄에서 값을 꺼냅니다.
fn field<'a>(block: &'a str, key: &str) -> Option<&'a str> {
    block.lines().find_map(|line| {
        line.strip_prefix(key)?
            .strip_prefix(" = \"")?
            .strip_suffix('"')
    })
}

/// `source` 값을 사람이 읽을 설명으로 바꿉니다.
fn describe(source: Option<&str>) -> String {
    let Some(source) = source else {
        // `source`가 없으면 내 디스크에 있는 패키지입니다. path 의존성과 패키지 자신이 그렇습니다.
        return "로컬 경로".to_string();
    };
    if let Some(rest) = source.strip_prefix("git+") {
        // 모양: git+<저장소 주소>?<고른 방법>#<커밋 해시>
        // `#` 뒤의 해시가 Cargo가 고정한 커밋입니다. `Cargo.toml`에는 태그 이름만 적었습니다.
        let (location, commit) = rest.split_once('#').unwrap_or((rest, "?"));
        let (url, selector) = location
            .split_once('?')
            .unwrap_or((location, "기본 브랜치"));
        let short = commit.get(..7).unwrap_or(commit);
        return format!("git 저장소  {url} ({selector}, 커밋 {short})");
    }
    if source == "registry+https://github.com/rust-lang/crates.io-index" {
        return "레지스트리  crates.io".to_string();
    }
    format!("그 밖의 출처  {source}")
}

fn main() {
    // 잠금 파일은 `[[package]]`로 시작하는 덩어리의 나열입니다.
    for block in LOCKFILE.split("[[package]]").skip(1) {
        let name = field(block, "name").unwrap_or("?");
        let version = field(block, "version").unwrap_or("?");
        let title = format!("{name} {version}");
        println!("{title:<20} {}", describe(field(block, "source")));
    }
    // 출력: cfg-if 1.0.4         git 저장소  https://github.com/rust-lang/cfg-if (tag=v1.0.4, 커밋 3510ca6)
    // 출력: frame 0.1.0          로컬 경로
    // 출력: heck 0.5.0           레지스트리  crates.io
    // 출력: three-sources 0.1.0  로컬 경로
    // 출력: unicode-width 0.2.3  레지스트리  crates.io
}
