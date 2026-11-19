// 기대하는 에러: E0277
// 핵심 메시지: a value of type `Vec<String>` cannot be built from an iterator over elements of type `&String`
// 에디션: 2015·2018·2021에서 실패 / 2024에서 통과(실행하면 ["가람", "나래", "다온"])
//
// 이번에는 반대 방향입니다. 2024 에디션에 익숙해져서 Box<[String]>의 .into_iter()가 String을
// 값으로 내놓는다고 믿고 쓴 코드를 2021 에디션 크레이트에 넣었습니다. 2021 에디션에서는 같은
// 호출이 &String을 내놓으므로 Vec<String>을 만들 수 없습니다.
// 고치려면 2021 에디션에서도 값을 내놓는 IntoIterator::into_iter(names)나 names.into_vec()을
// 쓰거나, 크레이트의 에디션을 2024로 올립니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2021 compile_fail/e0277_box_slice_by_value_2021.rs --out-dir /tmp/rust-p2-ch5   → 에러
//   rustc --edition 2024 compile_fail/e0277_box_slice_by_value_2021.rs --out-dir /tmp/rust-p2-ch5   → 통과

fn main() {
    let names: Box<[String]> = vec!["가람".to_string(), "나래".to_string()].into_boxed_slice();
    // 고정된 목록을 다시 늘릴 수 있는 Vec으로 옮겨 담습니다. String은 복제하지 않고 옮깁니다.
    let mut members: Vec<String> = names.into_iter().collect(); // → 2021 에디션에서 E0277
    members.push("다온".to_string());
    println!("{:?}", members);
}
