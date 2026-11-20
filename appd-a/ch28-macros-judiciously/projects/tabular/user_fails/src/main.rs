// 일부러 컴파일되지 않는 크레이트입니다.
// tabular의 매크로 넷을 "다른 크레이트"에서 불러, $crate 없이 쓴 매크로가 쓰는 쪽에서 깨지는 것을 보여 줍니다.
// tabular 자신의 테스트에서는 넷 다 통과합니다(tabular/src/lib.rs의 테스트).
//
// 실행(이 디렉터리에서): cargo build   → 종료 코드 101

use tabular::{row, row_bare, row_crate, row_private};

fn main() {
    let total = 3;
    let average = 1.5;
    println!("{}", row!(total, average)); // $crate::__format_row: 에러 없음
    println!("{}", row_bare!(total, average)); // E0425: 이 크레이트에서 __format_row를 찾음
    println!("{}", row_crate!(total, average)); // E0425: crate::는 이 크레이트(user_fails)의 루트
    println!("{}", row_private!(total, average)); // E0603: $crate로 찾아갔지만 비공개 함수
}
