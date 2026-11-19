// 기대하는 에러: E0277
// 핵심 메시지: the `?` operator can only be used in a closure that returns `Result` or `Option`
//             (... cannot use the `?` operator in a closure that returns `()`)
//
// 확인: rustc --edition 2024 compile_fail/e0277_question_in_for_each.rs --out-dir <임시 디렉터리>
//
// `for line in lines { totals.push(line.trim().parse()?); }`를 for_each로 옮기면서
// 본문을 그대로 클로저에 넣었습니다. ?는 바깥 함수가 아니라 클로저에서 빠져나가려고
// 하는데, for_each의 클로저는 ()를 돌려주므로 Err를 돌려줄 수 없습니다.

use std::num::ParseIntError;

fn add_all(lines: &[&str], totals: &mut Vec<u32>) -> Result<(), ParseIntError> {
    lines.iter().for_each(|line| {
        let cups: u32 = line.trim().parse()?;
        totals.push(cups);
    });
    Ok(())
}

// 고치는 방법 (예제 04, 05):
// - for 반복문으로 되돌립니다: for line in lines { totals.push(line.trim().parse()?); }
// - 체인을 유지하려면 try_for_each를 쓰고 클로저 끝에 Ok(())를 돌려줍니다.
// - 값을 모으는 것이 목적이면 collect::<Result<Vec<u32>, _>>()? 로 모은 뒤 extend합니다.

fn main() {
    let mut totals = Vec::new();
    println!("{:?} {totals:?}", add_all(&["128", "75"], &mut totals));
}
