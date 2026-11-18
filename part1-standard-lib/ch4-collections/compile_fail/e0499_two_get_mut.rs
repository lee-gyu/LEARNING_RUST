// 기대하는 에러: error[E0499]: cannot borrow `balance` as mutable more than once at a time
// 핵심 메시지: first mutable borrow occurs here / second mutable borrow occurs here
//             first borrow later used here
//
// get_mut은 맵 전체를 가변으로 빌려서 값 하나의 &mut를 돌려줍니다. 첫 번째 &mut(from)가 아직 쓰이는데
// 두 번째 get_mut을 부르면, 컴파일러는 두 키가 같은지 알 수 없으므로 같은 값을 가리키는 &mut 두 개가
// 생길 수 있다고 보고 거부합니다.
// 해결 1: 한 번에 하나씩 고칩니다.
//         *balance.get_mut("민수").unwrap() -= 1000;
//         *balance.get_mut("지영").unwrap() += 1000;
// 해결 2: 서로 다른 키 여러 개의 &mut를 한 번에 받는 get_disjoint_mut을 씁니다(키가 겹치면 패닉).
//         if let [Some(from), Some(to)] = balance.get_disjoint_mut(["민수", "지영"]) { ... }
//
// 확인: rustc --edition 2024 compile_fail/e0499_two_get_mut.rs --out-dir <임시 디렉터리>

use std::collections::HashMap;

fn main() {
    let mut balance: HashMap<&str, i64> = HashMap::from([("민수", 5000), ("지영", 3000)]);
    let from = balance.get_mut("민수").unwrap();
    let to = balance.get_mut("지영").unwrap();
    *from -= 1000;
    *to += 1000;
    println!("민수 {}, 지영 {}", balance["민수"], balance["지영"]);
}
