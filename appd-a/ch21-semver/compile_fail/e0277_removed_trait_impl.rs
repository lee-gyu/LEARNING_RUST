// 기대하는 에러: E0277 — the trait bound `TrackingNo: Hash` is not satisfied
// 핵심 메시지: required by a bound in `HashMap::<K, V, S, A>::entry`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_removed_trait_impl.rs --out-dir /tmp/er-ch21
//
// 상황: 모듈 parcel은 다른 사람이 관리하는 라이브러리라고 생각합니다.
//   - 1.0.0의 송장 번호 타입 TrackingNo는 Hash를 derive했고, 사용자는 이것을 HashMap의 키로 썼습니다.
//   - 1.1.0에서 라이브러리 안에서는 해시를 쓰지 않는다며 derive 목록에서 Hash를 뺐습니다.
//     라이브러리의 테스트는 모두 통과합니다. Hash를 쓰는 코드는 라이브러리 밖에 있기 때문입니다.
//   - 트레이트 구현을 떼는 것은 공개 API에서 항목을 지우는 것과 같아서, 그 트레이트를 쓰던 사용자 코드가
//     모두 깨집니다. 반대로 구현을 더하는 것은 대부분 마이너입니다(README 2.3절).
// 해결: 공개 타입에 붙인 트레이트 구현은 메이저 버전이 아니면 떼지 않습니다.
//       처음 붙일 때 "계속 약속할 수 있는가"를 따집니다(ch10 2.9절).
// 참고: 이 파일은 라이브러리와 사용자가 한 크레이트라서 컴파일러가 TrackingNo에 #[derive(Hash)]를
//       붙이라고 제안합니다. 실제로는 다른 크레이트의 타입이라 사용자가 고칠 수 없는 제안입니다.

mod parcel {
    // 1.1.0: 1.0.0의 derive 목록에서 Hash를 뺐습니다
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TrackingNo(String);

    impl TrackingNo {
        pub fn new(no: &str) -> Self {
            TrackingNo(no.to_string())
        }
    }
}

use parcel::TrackingNo;
use std::collections::HashMap;

fn main() {
    // 1.0.0 시절에 쓴 사용자 코드: 송장 번호별 조회 횟수를 셉니다
    let mut lookups = HashMap::new();
    for no in ["KR-1203", "KR-0042", "KR-1203"] {
        *lookups.entry(TrackingNo::new(no)).or_insert(0) += 1; // E0277
    }
    println!("송장 {}건 조회", lookups.len());
}
