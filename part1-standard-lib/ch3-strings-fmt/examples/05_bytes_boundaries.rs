// 이 예제가 보여 주는 것: 글자 경계를 지키며 안전하게 자르기(get, is_char_boundary, split_at_checked,
//                        floor_char_boundary, char_indices)와 바이트 ↔ 문자열 변환
//                        (as_bytes, into_bytes, str::from_utf8, String::from_utf8, String::from_utf8_lossy)
// 실행: cargo run -p p1-ch3-strings-fmt --example 05_bytes_boundaries
//
// &s[a..b]는 a나 b가 글자 경계가 아니면 패닉합니다(Part 0 ch8). 위치를 미리 알 수 없을 때는
// Option을 돌려주는 메서드로 확인하거나, char_indices로 경계 위치를 직접 구합니다.

/// 앞에서부터 최대 n글자(char)까지만 잘라 원본을 빌린 &str로 돌려줍니다.
fn take_chars(s: &str, n: usize) -> &str {
    // char_indices는 (그 글자가 시작하는 바이트 위치, 글자)를 내줍니다.
    // n번째 글자(0부터)의 시작 위치가 곧 "앞의 n글자"가 끝나는 경계입니다.
    match s.char_indices().nth(n) {
        Some((byte_pos, _)) => &s[..byte_pos],
        None => s, // 글자가 n개 이하이면 통째로 돌려줍니다.
    }
}

fn main() {
    let s = "가나다abc";

    // 1. 바이트 위치로 자르되 패닉하지 않기
    println!("== 1. get, is_char_boundary, split_at_checked ==");
    println!("len() = {} (바이트)", s.len()); // 출력: len() = 12 (바이트)
    let boundaries: Vec<usize> = (0..=s.len()).filter(|&i| s.is_char_boundary(i)).collect();
    println!("글자 경계인 위치: {boundaries:?}"); // 출력: 글자 경계인 위치: [0, 3, 6, 9, 10, 11, 12]
    println!("get(0..3) = {:?}", s.get(0..3)); // 출력: get(0..3) = Some("가")
    println!("get(0..4) = {:?}", s.get(0..4)); // 출력: get(0..4) = None
    println!("get(9..)  = {:?}", s.get(9..)); // 출력: get(9..)  = Some("abc")
    println!("split_at_checked(6) = {:?}", s.split_at_checked(6)); // 출력: split_at_checked(6) = Some(("가나", "다abc"))
    println!("split_at_checked(7) = {:?}", s.split_at_checked(7)); // 출력: split_at_checked(7) = None
    // floor/ceil_char_boundary(Rust 1.91에서 안정화)는 가장 가까운 경계로 내리거나 올립니다.
    // "최대 N바이트까지만 저장" 같은 제한을 지킬 때 씁니다.
    let cut = s.floor_char_boundary(7);
    println!("floor_char_boundary(7) = {cut} → {:?}", &s[..cut]); // 출력: floor_char_boundary(7) = 6 → "가나"
    println!("ceil_char_boundary(7)  = {}", s.ceil_char_boundary(7)); // 출력: ceil_char_boundary(7)  = 9

    // 2. 글자 수로 자르기: char_indices로 바이트 위치를 얻습니다.
    println!("\n== 2. char_indices로 글자 단위 자르기 ==");
    let indices: Vec<(usize, char)> = "가a나".char_indices().collect();
    println!("{indices:?}"); // 출력: [(0, '가'), (3, 'a'), (4, '나')]
    let title = "러스트로 배우는 시스템 프로그래밍";
    println!("{:?}", take_chars(title, 6)); // 출력: "러스트로 배"
    println!("{:?}", take_chars("짧음", 6)); // 출력: "짧음"

    // 3. 문자열 → 바이트: 빌려 보기(as_bytes)와 넘겨주기(into_bytes)
    println!("\n== 3. as_bytes, into_bytes ==");
    let word = String::from("Rust");
    let view: &[u8] = word.as_bytes(); // 복사하지 않고 같은 바이트를 &[u8]로 봅니다.
    println!("as_bytes() = {view:?}"); // 출력: as_bytes() = [82, 117, 115, 116]
    let owned: Vec<u8> = word.into_bytes(); // String을 소비하고 버퍼를 Vec<u8>로 넘겨줍니다(복사 없음).
    println!("into_bytes() = {owned:?}"); // 출력: into_bytes() = [82, 117, 115, 116]

    // 4. 바이트 → 문자열: UTF-8인지 검사한 뒤에만 문자열로 인정합니다.
    println!("\n== 4. from_utf8: 검사하고 변환 ==");
    let good = vec![237, 149, 156, 234, 184, 128]; // "한글"의 UTF-8 바이트
    // str::from_utf8(&[u8]) -> Result<&str, Utf8Error>: 빌린 바이트를 그대로 &str로 봅니다(복사 없음).
    println!("str::from_utf8    = {:?}", str::from_utf8(&good)); // 출력: str::from_utf8    = Ok("한글")
    // String::from_utf8(Vec<u8>) -> Result<String, FromUtf8Error>: Vec을 받아 그 버퍼로 String을 만듭니다.
    println!("String::from_utf8 = {:?}", String::from_utf8(good)); // 출력: String::from_utf8 = Ok("한글")

    // Latin-1(ISO-8859-1)로 저장된 "café": é가 0xE9 한 바이트라서 UTF-8로는 올바르지 않습니다.
    let latin1 = [b'c', b'a', b'f', 0xE9];
    // 바이트가 리터럴로 적혀 있으면 컴파일러가 "항상 Err가 된다"고 미리 경고합니다(invalid_from_utf8).
    // 실제 프로그램에서는 바이트를 파일이나 네트워크에서 받으므로 이 경고가 나지 않습니다.
    // 여기서는 Err를 관찰하려고 일부러 쓴 것이므로 #[expect]로 경고를 예상했다고 표시합니다.
    #[expect(
        invalid_from_utf8,
        reason = "일부러 잘못된 UTF-8을 넣어 Err를 관찰합니다"
    )]
    match str::from_utf8(&latin1) {
        Ok(text) => println!("성공: {text}"),
        Err(e) => {
            // valid_up_to()는 앞에서부터 몇 바이트까지가 올바른 UTF-8인지 알려 줍니다.
            println!("실패: {e}"); // 출력: 실패: incomplete utf-8 byte sequence from index 3
            println!(
                "올바른 부분: {:?}",
                str::from_utf8(&latin1[..e.valid_up_to()])
            ); // 출력: 올바른 부분: Ok("caf")
        }
    }

    // 5. 잘못된 바이트를 대체 문자(U+FFFD, �)로 바꿔서라도 읽기: from_utf8_lossy
    // 반환 타입 Cow<str>는 "빌린 &str 또는 소유한 String"을 담는 타입입니다. 바꿀 것이 없으면
    // 원본을 빌리기만 하고, 바꿔야 할 때만 새 String을 만듭니다(Cow는 ch6-core-traits).
    // {}로 출력하거나 &str 메서드를 그대로 쓸 수 있고, to_string()으로 String을 얻습니다.
    println!("\n== 5. from_utf8_lossy ==");
    println!("{}", String::from_utf8_lossy(&latin1)); // 출력: caf�
    // EUC-KR(CP949)로 저장된 "한글"을 UTF-8로 읽으면 깨집니다. 일부 바이트가 우연히 다른 글자(ѱ)로
    // 읽히기도 합니다. 표준 라이브러리에는 EUC-KR 변환기가 없으므로 이런 파일은 크레이트로 변환합니다.
    let euc_kr = [0xC7, 0xD1, 0xB1, 0xDB];
    let garbled = String::from_utf8_lossy(&euc_kr);
    println!("{garbled} ({}글자)", garbled.chars().count()); // 출력: �ѱ� (3글자)
}
