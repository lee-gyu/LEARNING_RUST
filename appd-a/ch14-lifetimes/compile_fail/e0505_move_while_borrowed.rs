// 기대하는 에러: E0505
// 핵심 메시지: cannot move out of `draft` because it is borrowed
//
// title은 draft를 빌린 참조인데, draft를 archive로 옮긴 뒤에 title을 씁니다.
// 값의 라이프타임은 drop될 때뿐 아니라 이동될 때도 끝나므로, 참조가 쓰이는 구간이
// 가리키는 값의 구간 밖으로 나갑니다. String을 옮겨도 힙의 글자는 그 자리에 있지만,
// 대여 검사기는 메모리가 아니라 소유자를 추적하므로 거부합니다(새 소유자가 언제
// 글자를 지우거나 drop할지 알 수 없기 때문입니다).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0505_move_while_borrowed.rs --out-dir /tmp/er-ch14

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

fn main() {
    let draft = String::from("장보기\n두부, 대파");
    let title = first_line(&draft);
    let mut archive: Vec<String> = Vec::new();
    archive.push(draft); // E0505: 빌려준 값을 옮김
    println!("보관한 목록의 제목: {title}"); // 옮긴 뒤에 참조를 씀
    println!("보관한 목록 수: {}", archive.len());
}
