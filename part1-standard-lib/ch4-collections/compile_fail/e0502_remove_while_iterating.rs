// 기대하는 에러: error[E0502]: cannot borrow `stock` as mutable because it is also borrowed as immutable
// 핵심 메시지: immutable borrow occurs here / immutable borrow later used here (둘 다 `&stock`)
//             mutable borrow occurs here (`stock.remove(name)`)
//
// for 문은 반복이 끝날 때까지 맵을 불변으로 빌립니다. 그 사이에 remove로 맵을 바꾸면
// 순회 중인 내부 구조가 바뀌어 반복자가 엉뚱한 곳을 가리킬 수 있으므로 대여 검사기가 막습니다.
// 해결: stock.retain(|_, count| *count > 0);  조건에 맞지 않는 항목을 한 번에 지웁니다.
//
// 확인: rustc --edition 2024 compile_fail/e0502_remove_while_iterating.rs --out-dir <임시 디렉터리>

use std::collections::HashMap;

fn main() {
    let mut stock: HashMap<&str, u32> = HashMap::from([("사과", 3), ("배", 0), ("감", 5)]);
    for (name, count) in &stock {
        if *count == 0 {
            stock.remove(name);
        }
    }
    println!("남은 상품 수: {}", stock.len());
}
