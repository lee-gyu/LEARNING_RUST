// 기대하는 결과: 2024 에디션에서 실패, 2021 에디션에서는 통과
// 핵심 메시지: in expressions, `_` can only be used on the left-hand side of an assignment (에러 코드 없음)
//
// 2024 에디션의 `$x:expr`는 `_`도 표현식으로 받습니다. 그래서 `budget!(_)`이 둘째 규칙 `(_)`까지
// 가지 못하고 첫째 규칙에 걸려 `Budget::Limit(_)`로 펼쳐집니다. `cargo fix --edition`은 `expr`를
// `expr_2021`로 바꿔 예전 동작을 지켜 줍니다. 그 대신 `(_)` 규칙을 앞으로 옮기면 `expr` 그대로 둘 수 있습니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/underscore_matches_expr_fragment.rs --out-dir /tmp/rust-p2-ch6   → 에러
//   rustc --edition 2021 compile_fail/underscore_matches_expr_fragment.rs --out-dir /tmp/rust-p2-ch6   → 통과

#[derive(Debug)]
enum Budget {
    Limit(i64),
    Unlimited,
}

/// 예산 한도: `budget!(30_000)`은 그 금액까지, `budget!(_)`은 한도 없음
macro_rules! budget {
    ($limit:expr) => {
        Budget::Limit($limit)
    };
    (_) => {
        Budget::Unlimited
    };
}

fn main() {
    println!("{:?}", budget!(30_000));
    println!("{:?}", budget!(_)); // → 2024 에디션에서 에러
}
