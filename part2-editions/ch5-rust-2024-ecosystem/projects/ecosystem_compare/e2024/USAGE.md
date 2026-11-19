인사말을 돌려줍니다. `formal`이 `true`이면 격식 있는 인사말입니다.

이 문서는 `src/lib.rs`가 `#[doc = include_str!("../USAGE.md")]`로 가져온 것입니다.
아래 문서 테스트의 `include_str!` 경로는 2024 에디션 규칙에 따라 **이 파일(USAGE.md)을 기준으로** 적었습니다.

이 코드 블록에는 `standalone_crate`를 붙였습니다. 이 툴체인에서 이 테스트를 다른 문서 테스트와
한 바이너리로 묶으면 묶음 전체의 컴파일이 실패하고, rustdoc은 아무 말 없이 모든 문서 테스트를
따로 컴파일합니다. 이 테스트만 따로 컴파일하게 표시하면 나머지는 다시 묶입니다.

```standalone_crate
let plain = include_str!("data/greeting.txt");
assert_eq!(e2024::greeting(false), plain.trim());
assert_eq!(e2024::greeting(true), "안녕하세요");
```
