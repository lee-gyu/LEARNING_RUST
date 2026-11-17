// 바이너리 크레이트의 루트입니다 (src/main.rs).
// 실제 기능은 모두 라이브러리 크레이트(src/lib.rs)에 있고, 여기서는 라이브러리를 불러 결과를 출력하기만 합니다.
//
// 이렇게 나누는 이유: tests/의 통합 테스트는 별도의 크레이트라서 라이브러리 크레이트의 공개 API만 쓸 수 있습니다.
// 기능이 main.rs에만 있으면 통합 테스트에서 그 함수를 가져올 방법이 없습니다.
// main.rs를 얇게 유지하면, 라이브러리를 테스트하는 것만으로 거의 모든 코드를 검사할 수 있습니다.

use adder::{add_two, parse_sum};

fn main() {
    println!("add_two(3) = {}", add_two(3));

    for expr in ["10 + 20 + 30", "10 + 스물"] {
        match parse_sum(expr) {
            Ok(total) => println!("{expr} = {total}"),
            Err(e) => println!("{expr}: 계산할 수 없습니다 ({e})"),
        }
    }
}
