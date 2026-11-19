인사말을 돌려줍니다. `formal`이 `true`이면 격식 있는 인사말입니다.

이 문서는 `src/lib.rs`가 `#[doc = include_str!("../USAGE.md")]`로 가져온 것입니다.
아래 문서 테스트의 `include_str!` 경로는 2021 에디션 규칙에 따라 **`src/lib.rs`를 기준으로** 적었습니다.

```
let plain = include_str!("../data/greeting.txt");
assert_eq!(e2021::greeting(false), plain.trim());
assert_eq!(e2021::greeting(true), "안녕하세요");
```
