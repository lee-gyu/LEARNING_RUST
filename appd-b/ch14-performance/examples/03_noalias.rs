// 부록 B ch14 · Q14.1 — &mut가 컴파일러에 주는 정보
//
// add_twice는 *a에 *b를 두 번 더합니다. C에서 같은 함수를 int *a, const int *b로 쓰면 컴파일러는 a와 b가 같은
// 곳을 가리킬 수도 있다고 보아야 하므로, *a에 쓴 뒤 *b를 다시 읽습니다(compare/02_aliasing.c).
// Rust의 &mut i32와 &i32는 같은 곳을 가리킬 수 없으므로(부록 B ch10 Q10.7) rustc는 LLVM에 두 매개변수가 서로
// 겹치지 않는다(noalias)고 알려 줍니다. 그래서 *b를 한 번만 읽고 두 배로 만들어 한 번에 더할 수 있습니다.
// 같은 정보를 C에서 주려면 프로그래머가 restrict를 붙여야 합니다.
//
// 실행: cargo run -p faq-ch14-performance --example 03_noalias
// LLVM IR 보기(저장소 루트에서. README Q14.1에 결과를 옮겼습니다):
//   mkdir -p target/tmp/faq-ch14-performance
//   rustc --edition 2024 -C opt-level=3 --emit=llvm-ir -o target/tmp/faq-ch14-performance/noalias.ll appd-b/ch14-performance/examples/03_noalias.rs
//   grep -A6 'define.*add_twice' target/tmp/faq-ch14-performance/noalias.ll

use std::hint::black_box;

// C 쪽 add_twice처럼 다른 파일에서도 부를 수 있는 함수로 두려고 심볼 이름을 그대로 내보냅니다(부록 B ch13 Q13.6).
// 크레이트 안에서만 쓰는 함수라면 LLVM이 &i32를 값으로 바꿔 넘기는 등 더 고쳐 버려 비교가 흐려지기 때문입니다.
#[unsafe(no_mangle)]
pub fn add_twice(a: &mut i32, b: &i32) {
    *a += *b;
    *a += *b;
}

fn main() {
    let mut total = black_box(10);
    let step = black_box(3);
    add_twice(&mut total, &step);
    println!("total = {total}"); // 출력: total = 16
}
