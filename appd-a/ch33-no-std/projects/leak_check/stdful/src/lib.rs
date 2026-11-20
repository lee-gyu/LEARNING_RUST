//! 보통의 std 라이브러리입니다. `HashSet`은 std에만 있으므로 std 없이는 빌드되지 않습니다.

use std::collections::HashSet;

/// 서로 다른 ID의 수를 셉니다.
pub fn count_distinct(ids: impl IntoIterator<Item = u32>) -> usize {
    ids.into_iter().collect::<HashSet<_>>().len()
}
