// CString/CStr: C 문자열을 만들고, 바이트에서 빌리고, Rust 문자열로 되돌리기.
// 핵심 규칙은 "끝에 0 바이트가 하나 있고, 중간에는 0 바이트가 없다"입니다.
//
// 실행: cargo run -p p1-ch14-ffi-any-misc --example 03_c_string

use std::ffi::{CStr, CString};
use std::num::NonZero;

fn main() {
    println!("1. CString::new: 0 바이트를 붙인 새 버퍼를 만듭니다");
    // &str, String, Vec<u8>, &[u8]처럼 Into<Vec<u8>>인 것은 무엇이든 받습니다.
    let greeting = CString::new("안녕").unwrap();
    let from_vec = CString::new(vec![b'o', b'k']).unwrap();
    println!(
        "    {greeting:?}: to_bytes {:?}, to_bytes_with_nul {:?}",
        greeting.to_bytes(),
        greeting.to_bytes_with_nul()
    );
    // 출력:     "안녕": to_bytes [236, 149, 136, 235, 133, 149], to_bytes_with_nul [236, 149, 136, 235, 133, 149, 0]
    // count_bytes는 C의 strlen처럼 0 바이트를 빼고 셉니다.
    println!(
        "    {from_vec:?}.count_bytes() = {}",
        from_vec.count_bytes()
    );
    // 출력:     "ok".count_bytes() = 2

    println!("2. 중간에 0 바이트가 있으면 NulError");
    // C는 0 바이트를 문자열의 끝으로 읽으므로, 중간의 0 바이트 뒤는 C 쪽에서 보이지 않게 됩니다.
    // 그런 문자열을 조용히 잘라 넘기는 대신 만들 때 거절합니다.
    let err = CString::new("이름\0비밀").unwrap_err();
    println!("    에러 메시지: {err}");
    // 출력:     에러 메시지: nul byte found in provided data at position: 6
    println!("    nul_position(): {}", err.nul_position()); // 출력:     nul_position(): 6
    // into_vec으로 넘겼던 바이트를 돌려받을 수 있습니다. 실패해도 데이터를 잃지 않습니다.
    let bytes = err.into_vec();
    println!("    돌려받은 바이트 수: {}", bytes.len()); // 출력:     돌려받은 바이트 수: 13

    println!("3. c\"...\" 리터럴은 &'static CStr입니다");
    // 컴파일러가 끝에 0 바이트를 붙이고, 중간의 0 바이트가 없는지도 컴파일할 때 확인합니다.
    let literal: &'static CStr = c"리터럴";
    println!(
        "    {literal:?}: {}바이트 + 0 바이트",
        literal.count_bytes()
    );
    // 출력:     "리터럴": 9바이트 + 0 바이트
    // 소유한 CString이 필요하면 복사합니다.
    let owned: CString = literal.to_owned();
    println!("    to_owned() == 리터럴: {}", owned.as_c_str() == literal); // 출력:     to_owned() == 리터럴: true

    println!("4. 이미 있는 바이트에서 &CStr 빌리기");
    // from_bytes_with_nul: 마지막 바이트가 0이고 중간에는 0이 없어야 합니다.
    let exact = b"hello\0".to_vec();
    let no_nul = b"hello".to_vec();
    let interior = b"he\0llo\0".to_vec();
    println!("    {:?}", CStr::from_bytes_with_nul(&exact)); // 출력:     Ok("hello")
    println!("    {:?}", CStr::from_bytes_with_nul(&no_nul)); // 출력:     Err(NotNulTerminated)
    println!("    {:?}", CStr::from_bytes_with_nul(&interior)); // 출력:     Err(InteriorNul { position: 2 })
    // from_bytes_until_nul: 첫 0 바이트까지를 C 문자열로 봅니다.
    // C 구조체의 char name[16] 같은 고정 크기 버퍼를 읽을 때 꼭 맞습니다.
    let mut name_field = [0u8; 16]; // 처음에는 모두 0
    let name = "김철수".as_bytes(); // 9바이트. 나머지 7바이트는 0으로 남습니다.
    name_field[..name.len()].copy_from_slice(name);
    println!("    {:?}", CStr::from_bytes_until_nul(&name_field)); // 출력:     Ok("김철수")
    // 버퍼가 0 바이트 없이 꽉 차 있으면 에러입니다(C 쪽에서 끝을 표시하지 않은 경우).
    let full = [b'x'; 4];
    println!("    {:?}", CStr::from_bytes_until_nul(&full)); // 출력:     Err(FromBytesUntilNulError(()))

    println!("5. Rust 문자열로 되돌리기");
    // C 문자열은 UTF-8이라는 보장이 없습니다. 0xFF는 UTF-8 어디에도 나올 수 없는 바이트입니다.
    let latin = CString::new(vec![b'f', 0xFF, b'o']).unwrap();
    println!("    디버그 형식: {latin:?}"); // 출력:     디버그 형식: "f\xffo"
    println!("    to_str():          {:?}", greeting.to_str()); // 출력:     to_str():          Ok("안녕")
    println!(
        "    to_str():          {:?}",
        latin.to_str().map_err(|e| e.valid_up_to())
    );
    // 출력:     to_str():          Err(1)
    println!("    to_string_lossy(): {:?}", latin.to_string_lossy()); // 출력:     to_string_lossy(): "f�o"
    // into_string은 실패하면 IntoStringError에 원래 CString을 담아 돌려줍니다.
    match latin.into_string() {
        Ok(s) => println!("    into_string(): Ok({s:?})"),
        Err(e) => println!("    into_string(): Err, 되찾은 값 {:?}", e.into_cstring()),
    }
    // 출력:     into_string(): Err, 되찾은 값 "f\xffo"

    println!("6. 0이 아닌 바이트만 모았다면 검사가 필요 없습니다");
    // NonZero<u8>(예제 08)는 0일 수 없는 바이트입니다. 그런 바이트만 모은 Vec이라면
    // 중간에 0 바이트가 있을 수 없으므로 From으로 바로 바꿀 수 있습니다(Result가 아님).
    let letters: Vec<NonZero<u8>> = b"abc".iter().filter_map(|&b| NonZero::new(b)).collect();
    let from_nonzero = CString::from(letters);
    println!("    {from_nonzero:?}"); // 출력:     "abc"
}
