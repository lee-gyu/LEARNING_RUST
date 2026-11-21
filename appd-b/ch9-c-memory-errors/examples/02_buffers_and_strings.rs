// Q9.4 · Q9.8 — 고정 크기 버퍼와 문자열이 넘치지 않는 이유
//
// C의 배열과 문자열은 길이를 따로 들고 다니지 않습니다. strcpy는 끝의 0 바이트가 나올 때까지 복사하므로
// 받는 쪽이 작으면 옆 메모리를 덮어씁니다(compare/07_strcpy_overflow.c). Rust의 배열과 슬라이스는
// 길이를 알고, 모든 접근이 그 길이 안에서만 일어납니다.
//
//   1. 8바이트 칸에 긴 이름 넣기: 길이를 비교해 들어갈 만큼만 복사. 칸 밖은 get이 None
//   2. String은 길이와 용량을 알고, 모자라면 스스로 늘어남. 0 바이트는 그냥 한 바이트
//   3. 사용자 입력을 서식에 넣어도 데이터일 뿐(C의 printf(user) 문제가 생기지 않음)
//
// 범위를 벗어난 get이 None을 돌려주는 기본은 ch2 예제 01, 인덱싱이 패닉으로 멈추는 것은 ch2 예제 03에서
// 봤습니다. 여기서는 C에서 넘치던 작업을 Rust로 옮기면 어떻게 쓰는지를 봅니다.
//
// 실행: cargo run -p faq-ch9-c-memory-errors --example 02_buffers_and_strings

/// `src`를 `dst`에 들어갈 만큼만 복사하고, 복사한 바이트 수를 돌려줍니다(C의 strncpy 자리).
fn copy_name(dst: &mut [u8; 8], src: &str) -> usize {
    let n = src.len().min(dst.len());
    // copy_from_slice는 두 슬라이스의 길이가 다르면 덮어쓰는 대신 패닉합니다. 그래서 길이를 먼저 맞춥니다.
    dst[..n].copy_from_slice(&src.as_bytes()[..n]);
    n
}

fn main() {
    println!("1. 8바이트 칸에 이름 넣기");
    let mut name = [0u8; 8];
    let n = copy_name(&mut name, "hana-kim-long");
    let copied = std::str::from_utf8(&name[..n]).unwrap();
    println!("  {n}바이트 복사: {copied:?}");
    println!("  칸 밖 name.get(8): {:?}", name.get(8));
    // 출력:   8바이트 복사: "hana-kim"
    // 출력:   칸 밖 name.get(8): None
    // (바이트 단위로 자르므로 한글처럼 여러 바이트인 글자는 중간에서 잘릴 수 있습니다. 글자 경계에서
    //  자르는 방법은 Part 1 ch3 예제 05)

    println!("2. String은 길이를 알고 늘어남");
    let mut s = String::with_capacity(4);
    s.push_str("hana");
    s.push_str("-kim-long"); // 용량 4를 넘으면 더 큰 공간을 다시 잡습니다(제자리에서 늘 수도, 옮길 수도 있음).
    println!(
        "  {s:?}: 길이 {}, 용량이 길이 이상인가: {}",
        s.len(),
        s.capacity() >= s.len()
    );
    println!("  \"a\\0b\"의 길이: {}", "a\0b".len());
    // 출력:   "hana-kim-long": 길이 13, 용량이 길이 이상인가: true
    // 출력:   "a\0b"의 길이: 3

    println!("3. 입력은 서식이 아니라 데이터");
    let input = String::from("%s%s%n");
    println!("  입력: {input}");
    // 출력:   입력: %s%s%n
}
