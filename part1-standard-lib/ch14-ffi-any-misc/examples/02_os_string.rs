// OsString/OsStr: 운영체제가 준 문자열을 잃지 않고 들고 다니는 타입.
// 어디서 만나는지, UTF-8로 바꾸는 세 가지 방법, UTF-8이 아닌 값을 직접 만들어 관찰하기,
// 그리고 C 문자열로 넘기는 길을 보여 줍니다.
//
// 실행: cargo run -p p1-ch14-ffi-any-misc --example 02_os_string
//
// 3·5번 구역은 유닉스 전용 확장 트레이트(std::os::unix::ffi)를 씁니다.
// #[cfg(unix)]로 감쌌으므로 Windows에서는 그 구역 대신 안내 문장만 출력합니다.
// 1번 구역의 CARGO_PKG_NAME은 cargo run이 실행하는 프로그램에 넣어 주는 환경 변수라서,
// 실행 파일을 직접 실행하면 None이 나옵니다.

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::Path;

fn main() {
    println!("1. OsString/OsStr를 만나는 곳");
    // 환경 변수와 명령줄 인자는 운영체제가 준 바이트이므로 UTF-8이라는 보장이 없습니다.
    // 그래서 var_os와 args_os는 String이 아니라 OsString을 돌려줍니다.
    let pkg: Option<OsString> = env::var_os("CARGO_PKG_NAME");
    let missing: Option<OsString> = env::var_os("CH14_SURELY_UNSET");
    println!("    var_os(\"CARGO_PKG_NAME\")    = {pkg:?}");
    println!("    var_os(\"CH14_SURELY_UNSET\") = {missing:?}");
    // 출력:     var_os("CARGO_PKG_NAME")    = Some("p1-ch14-ffi-any-misc")
    // 출력:     var_os("CH14_SURELY_UNSET") = None
    // 0번 인자는 실행 파일 경로라 환경마다 다르므로 값은 출력하지 않습니다.
    let first_arg: Option<OsString> = env::args_os().next();
    println!("    args_os()의 0번 인자가 있는가: {}", first_arg.is_some());
    // 출력:     args_os()의 0번 인자가 있는가: true
    // 경로의 구성 요소도 OsStr입니다. 파일 이름이 UTF-8이 아닐 수 있기 때문입니다.
    let name: Option<&OsStr> = Path::new("문서/회의록.md").file_name();
    println!("    file_name() = {name:?}"); // 출력:     file_name() = Some("회의록.md")

    println!("2. UTF-8이라면 세 가지 변환이 모두 성공합니다");
    let os = OsString::from("회의록.md");
    // to_str: 빌린 &str (Option). to_string_lossy: Cow<str>. into_string: 소유권을 넘기고 String (Result).
    println!("    to_str()          = {:?}", os.to_str());
    println!("    to_string_lossy() = {:?}", os.to_string_lossy());
    println!("    into_string()     = {:?}", os.clone().into_string());
    // 출력:     to_str()          = Some("회의록.md")
    // 출력:     to_string_lossy() = "회의록.md"
    // 출력:     into_string()     = Ok("회의록.md")

    #[cfg(unix)]
    unix_only::non_utf8();
    #[cfg(not(unix))]
    println!("3. (유닉스 전용 구역이라 건너뜁니다)");

    println!("4. OsString으로 할 수 있는 일은 많지 않습니다");
    let mut log_name = OsString::from("server");
    log_name.push("-2026"); // push는 &str, &OsStr 등 AsRef<OsStr>인 것을 받습니다.
    log_name.push(OsStr::new(".LOG"));
    println!("    push로 이어 붙이기: {log_name:?}"); // 출력:     push로 이어 붙이기: "server-2026.LOG"
    // &str과 바로 비교할 수 있고, ASCII 범위의 대소문자 처리도 있습니다.
    println!(
        "    == \"server-2026.LOG\": {}, eq_ignore_ascii_case(\"SERVER-2026.log\"): {}",
        log_name == "server-2026.LOG",
        log_name.eq_ignore_ascii_case("SERVER-2026.log")
    );
    // 출력:     == "server-2026.LOG": true, eq_ignore_ascii_case("SERVER-2026.log"): true
    // split, starts_with 같은 문자열 처리는 OsStr에 (안정 버전에는) 없습니다.
    // to_str()로 UTF-8인지 확인한 뒤 &str의 메서드를 쓰거나, 바이트로 다룹니다.
    let is_log = log_name
        .to_str()
        .is_some_and(|s| s.to_ascii_lowercase().ends_with(".log"));
    let starts = log_name.as_encoded_bytes().starts_with(b"server");
    println!("    to_str 후 ends_with: {is_log}, as_encoded_bytes 후 starts_with: {starts}");
    // 출력:     to_str 후 ends_with: true, as_encoded_bytes 후 starts_with: true

    #[cfg(unix)]
    unix_only::to_c_string();
    #[cfg(not(unix))]
    println!("5. (유닉스 전용 구역이라 건너뜁니다)");
}

#[cfg(unix)]
mod unix_only {
    use std::ffi::{CString, OsStr, OsString};
    // 유닉스에서 OsStr과 바이트를 서로 바꾸는 확장 트레이트입니다.
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    pub fn non_utf8() {
        println!("3. UTF-8이 아닌 OsStr 만들기 (유닉스 기준)");
        // 0xE9는 Latin-1 인코딩의 'é'입니다. 옛 시스템에서 만든 파일 이름에 이런 바이트가 남아 있습니다.
        // UTF-8에서 0xE9는 3바이트 글자의 첫 바이트인데 뒤가 이어지지 않으므로 올바르지 않습니다.
        let raw: &OsStr = OsStr::from_bytes(b"caf\xE9.txt");
        println!("    디버그 형식: {raw:?}"); // 출력:     디버그 형식: "caf\xE9.txt"
        println!("    to_str():          {:?}", raw.to_str()); // 출력:     to_str():          None
        // 고칠 수 없는 바이트는 대체 문자(U+FFFD)로 바뀝니다. 새 String을 만들어야 하므로 Cow::Owned입니다.
        let lossy = raw.to_string_lossy();
        println!(
            "    to_string_lossy(): {lossy:?} (새로 할당했는가: {})",
            matches!(lossy, std::borrow::Cow::Owned(_))
        );
        // 출력:     to_string_lossy(): "caf�.txt" (새로 할당했는가: true)
        println!("    display():         {}", raw.display()); // 출력:     display():         caf�.txt
        // into_string은 실패하면 원래 OsString을 Err에 담아 돌려줍니다. 아무것도 잃지 않습니다.
        match raw.to_os_string().into_string() {
            Ok(s) => println!("    into_string(): Ok({s:?})"),
            Err(original) => println!("    into_string(): Err({original:?})"),
        }
        // 출력:     into_string(): Err("caf\xE9.txt")
        // OsStr 자체는 손실 없이 원래 바이트를 그대로 들고 있습니다.
        println!("    as_bytes():        {:x?}", raw.as_bytes());
        // 출력:     as_bytes():        [63, 61, 66, e9, 2e, 74, 78, 74]
        let rebuilt = OsString::from_vec(raw.as_bytes().to_vec());
        println!("    바이트 → OsString → 비교: {}", rebuilt == raw); // 출력:     바이트 → OsString → 비교: true
    }

    pub fn to_c_string() {
        println!("5. OsString → CString (유닉스 기준)");
        // OsStr과 CStr 사이에는 직접 바꾸는 메서드가 없습니다.
        // 유닉스에서는 둘 다 "0이 아닌 바이트들"이므로 바이트를 거쳐 갑니다.
        // 시스템 호출에 넘길 경로가 이런 식으로 C 문자열이 됩니다.
        let path = OsString::from_vec(b"/tmp/caf\xE9.txt".to_vec());
        let c_path = CString::new(path.into_vec());
        println!("    {c_path:?}"); // 출력:     Ok("/tmp/caf\xe9.txt")
        // 운영체제가 준 값이라도 중간에 0 바이트를 넣어 만들면 CString::new가 거절합니다.
        let with_nul = OsString::from_vec(b"a\0b".to_vec());
        println!(
            "    {:?}",
            CString::new(with_nul.into_vec()).map_err(|e| e.nul_position())
        );
        // 출력:     Err(1)
    }
}
