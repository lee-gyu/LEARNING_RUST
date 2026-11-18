// 이 예제가 보여 주는 것: String을 제자리에서 고치는 메서드(insert, insert_str, pop, remove,
//                        truncate, retain, drain, replace_range, split_off, clear)와 바이트 인덱스·글자 경계
// 실행: cargo run -p p1-ch3-strings-fmt --example 04_string_mutate
//
// 이 메서드들은 `&mut self`를 받습니다. 원본 String 자체를 바꾸므로 변수가 `mut`이어야 하고,
// &str(빌린 문자열)에는 없습니다. 위치를 받는 메서드의 인덱스는 모두 "바이트" 위치이며,
// 글자 경계가 아닌 위치를 주면 패닉합니다. 그래서 이 예제는 경계를 먼저 확인합니다.

fn main() {
    // 1. 넣기: insert(바이트 위치, char), insert_str(바이트 위치, &str)
    println!("== 1. insert, insert_str ==");
    let mut s = String::from("안녕하세요");
    s.insert(0, '[');
    s.push(']');
    println!("{s}"); // 출력: [안녕하세요]
    // '[' 1바이트 + "안녕" 6바이트 = 7번 바이트가 "안녕"과 "하세요" 사이입니다.
    let at = 7;
    if s.is_char_boundary(at) {
        s.insert_str(at, ", 러스트");
    }
    println!("{s}"); // 출력: [안녕, 러스트하세요]
    // '러'는 9~11번 바이트를 차지하므로 10번 바이트는 '러'의 중간입니다.
    // 여기에 insert하면 패닉하므로 확인만 합니다.
    println!("10번 바이트가 글자 경계인가? {}", s.is_char_boundary(10)); // 출력: 10번 바이트가 글자 경계인가? false

    // 2. 빼기: pop은 마지막 글자를, remove는 주어진 위치의 글자를 꺼내 돌려줍니다.
    println!("\n== 2. pop, remove ==");
    let last = s.pop(); // 비어 있으면 None이므로 Option<char>입니다.
    let first = s.remove(0); // 위치가 잘못되면 패닉하므로 Option이 아니라 char입니다.
    println!("pop() = {last:?}, remove(0) = {first:?}, s = {s}"); // 출력: pop() = Some(']'), remove(0) = '[', s = 안녕, 러스트하세요

    // 3. 줄이기: truncate(바이트 길이)는 그 뒤를 버립니다. 용량(capacity)은 그대로입니다.
    println!("\n== 3. truncate ==");
    let before = s.capacity();
    s.truncate(6); // "안녕" = 6바이트. 글자 경계가 아니면 패닉합니다.
    println!(
        "s = {s:?}, len = {}, 용량 그대로? {}",
        s.len(),
        s.capacity() == before
    ); // 출력: s = "안녕", len = 6, 용량 그대로? true

    // 4. 조건으로 거르기: retain은 클로저가 true를 돌려준 글자만 남깁니다.
    println!("\n== 4. retain ==");
    let mut id = String::from("  ab-12 cd_34 ");
    id.retain(|c| c.is_ascii_alphanumeric());
    println!("{id}"); // 출력: ab12cd34

    // 5. 범위를 꺼내기: drain(범위)은 그 범위의 글자를 내주는 반복자를 돌려주고, 원본에서 지웁니다.
    println!("\n== 5. drain ==");
    let mut msg = String::from("[경고] 디스크 부족");
    let end = msg.find(']').map(|i| i + 1).unwrap_or(0); // ']' 다음 바이트 위치
    let header: String = msg.drain(..end).collect();
    println!("꺼낸 것 = {header:?}, 남은 것 = {msg:?}"); // 출력: 꺼낸 것 = "[경고]", 남은 것 = " 디스크 부족"

    // 6. 범위를 바꾸기: replace_range는 그 범위를 다른 문자열로 바꿉니다(길이가 달라도 됩니다).
    println!("\n== 6. replace_range ==");
    let mut date = String::from("2025-03-14");
    date.replace_range(..4, "2026");
    println!("{date}"); // 출력: 2026-03-14

    // 7. 둘로 가르기: split_off(위치)는 뒷부분을 새 String으로 떼어 돌려주고, 원본에는 앞부분만 남깁니다.
    println!("\n== 7. split_off ==");
    let mut full = String::from("홍길동");
    let given = full.split_off(3); // '홍'이 3바이트
    println!("성 = {full}, 이름 = {given}"); // 출력: 성 = 홍, 이름 = 길동

    // 8. 비우기: clear는 길이를 0으로 만들지만 할당받은 메모리(용량)는 남겨 둡니다.
    // 같은 String을 반복해서 다시 채울 때 재할당을 줄일 수 있습니다.
    println!("\n== 8. clear ==");
    let mut buf = String::with_capacity(32);
    buf.push_str("임시 데이터");
    buf.clear();
    println!(
        "len = {}, 비었나? {}, 용량 ≥ 32? {}",
        buf.len(),
        buf.is_empty(),
        buf.capacity() >= 32
    ); // 출력: len = 0, 비었나? true, 용량 ≥ 32? true
}
