// 문자열 타입 가족의 지도: String/str, OsString/OsStr, CString/CStr, PathBuf/Path.
// 같은 글자를 네 가족으로 만들어 보고, 소유한 것과 빌린 것의 짝,
// 그리고 서로 바꿀 때 "공짜인 방향"과 "검사가 필요한 방향"을 확인합니다.
//
// 실행: cargo run -p p1-ch14-ffi-any-misc --example 01_string_family

use std::ffi::{CStr, CString, OsStr, OsString};
use std::path::{Path, PathBuf};

fn main() {
    println!("1. 같은 글자, 네 가족");
    let text = "메모.txt"; // 한글 두 글자(3바이트씩) + ".txt"(4바이트) = 10바이트
    let string = String::from(text);
    let os_string = OsString::from(text);
    // CString만 Result를 돌려줍니다. 중간에 0 바이트가 있으면 만들 수 없기 때문입니다(예제 03).
    let c_string = CString::new(text).unwrap();
    let path_buf = PathBuf::from(text);
    println!("    String   {string:?}: {}바이트", string.len());
    println!("    OsString {os_string:?}: {}바이트", os_string.len());
    println!(
        "    CString  {c_string:?}: 내용 {}바이트, 0 바이트까지 {}바이트",
        c_string.as_bytes().len(),
        c_string.as_bytes_with_nul().len()
    );
    println!(
        "    PathBuf  {path_buf:?}: {}바이트",
        path_buf.as_os_str().len()
    );
    // 출력:     String   "메모.txt": 10바이트
    // 출력:     OsString "메모.txt": 10바이트
    // 출력:     CString  "메모.txt": 내용 10바이트, 0 바이트까지 11바이트
    // 출력:     PathBuf  "메모.txt": 10바이트

    println!("2. 소유한 것과 빌린 것은 짝을 이룹니다");
    // String이 Deref로 str이 되듯, 나머지 셋도 각자의 빌린 타입으로 역참조됩니다.
    // 그래서 함수는 빌린 타입(&str, &OsStr, &CStr, &Path)으로 받는 것이 관례입니다.
    let s: &str = &string;
    let o: &OsStr = &os_string;
    let c: &CStr = &c_string;
    let p: &Path = &path_buf;
    println!("    &str {s:?}, &OsStr {o:?}, &CStr {c:?}, &Path {p:?}");
    // 빌린 것에서 소유한 것으로 돌아갈 때는 to_owned(ToOwned 트레이트)를 씁니다.
    let owned_again = (s.to_owned(), o.to_owned(), c.to_owned(), p.to_owned());
    println!(
        "    to_owned로 되돌린 값이 원래와 같은가: {}",
        owned_again
            == (
                string.clone(),
                os_string.clone(),
                c_string.clone(),
                path_buf.clone()
            )
    );
    // 출력:     &str "메모.txt", &OsStr "메모.txt", &CStr "메모.txt", &Path "메모.txt"
    // 출력:     to_owned로 되돌린 값이 원래와 같은가: true

    println!("3. UTF-8에서 출발하는 변환은 검사가 필요 없습니다");
    // 올바른 UTF-8은 운영체제 문자열로도, 경로로도 언제나 올바릅니다.
    // OsStr::new와 Path::new는 복사하지 않고 같은 바이트를 다른 타입으로 볼 뿐입니다.
    let as_os: &OsStr = OsStr::new(s);
    let as_path: &Path = Path::new(s);
    println!(
        "    OsStr::new, Path::new가 같은 바이트를 가리키는가: {}, {}",
        as_os.as_encoded_bytes().as_ptr() == s.as_ptr(),
        as_path.as_os_str().as_encoded_bytes().as_ptr() == s.as_ptr()
    );
    // 출력:     OsStr::new, Path::new가 같은 바이트를 가리키는가: true, true
    // 경로와 운영체제 문자열 사이도 공짜입니다. Path는 OsStr 위에 만들어진 타입입니다.
    println!("    Path → OsStr: {:?}", p.as_os_str()); // 출력:     Path → OsStr: "메모.txt"
    // 하지만 CString으로 가는 길은 다릅니다. 끝에 0 바이트를 붙이려면 새 버퍼에 복사해야 하고,
    // 중간에 0 바이트가 있으면 실패합니다.
    println!("    \"a\\0b\" → CString: {:?}", CString::new("a\0b"));
    // 출력:     "a\0b" → CString: Err(NulError(1, [97, 0, 98]))

    println!("4. UTF-8로 돌아오는 변환은 실패할 수 있습니다");
    // OsStr, Path, CStr에는 UTF-8이 아닌 바이트가 들어 있을 수 있으므로
    // &str로 바꾸는 메서드는 Option이나 Result를 돌려줍니다.
    println!("    OsStr::to_str → {:?}", o.to_str()); // 출력:     OsStr::to_str → Some("메모.txt")
    println!("    Path::to_str  → {:?}", p.to_str()); // 출력:     Path::to_str  → Some("메모.txt")
    println!("    CStr::to_str  → {:?}", c.to_str()); // 출력:     CStr::to_str  → Ok("메모.txt")
    // 실패를 대체 문자(U+FFFD)로 덮고 싶으면 셋 모두 to_string_lossy를 씁니다.
    // 올바른 UTF-8이면 복사 없이 빌린 값(Cow::Borrowed)을 돌려줍니다.
    println!(
        "    to_string_lossy → {:?}, {:?}, {:?}",
        o.to_string_lossy(),
        p.to_string_lossy(),
        c.to_string_lossy()
    );
    // 출력:     to_string_lossy → "메모.txt", "메모.txt", "메모.txt"
}
