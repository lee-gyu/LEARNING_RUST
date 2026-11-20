# ch27. 공개 인터페이스를 문서화하기

공개 API의 문서는 그 API를 쓰는 사람이 코드를 열지 않고 읽는 유일한 설명입니다. 그런데 문서 주석을 다는 것과 쓸모 있는 문서를 쓰는 것은 다릅니다. 시그니처를 말로 옮긴 주석은 읽을 것이 없고, 리팩터링 한 번에 코드와 어긋나며, 컴파일러는 그 어긋남을 알려 주지 않습니다. 이 챕터는 **무엇을 쓰고 무엇을 쓰지 않을지**를 판단하는 기준을 다룹니다. 코드가 말하지 못하는 약속을 쓰는 법, 그 약속을 타입으로 옮기는 법, 실패 조건을 정해진 절에 쓰는 법을 예제로 확인하고, 같은 라이브러리의 부실한 문서판과 고친 판을 `cargo doc`·`cargo test --doc`·린트에 넣어 도구가 잡는 것과 사람이 읽어야만 잡히는 것을 나눠 봅니다.

- 원제: [Item 27: Document public interfaces](https://effective-rust.com/documentation.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch27-document-public-interfaces`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

책의 5장(Tooling)을 여는 Item입니다. 앞 챕터들이 이 챕터로 미룬 "재내보낸 항목이 문서에 보이는 모양"도 2.6절에서 받습니다. 다음 내용을 안다고 보고 진행합니다.

- [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md)의 "8. 주석", [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md)의 "문서 테스트 맛보기": `///` 문서 주석, `cargo test`의 `Doc-tests` 구역
- [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md) 2.10절: `# Panics`와 `# Errors` 절의 첫 예
- [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.8절: 2024 에디션의 문서 테스트 병합
- [ch16](../ch16-avoid-unsafe/README.md) 2.6절(`# Safety`와 안전성 주석), [ch18](../ch18-dont-panic/README.md) 2.3절(패닉판과 `Result`판, `#[track_caller]`)
- [ch6](../ch6-newtype-pattern/README.md) 2.3절(범위를 검증하는 생성자), [ch22](../ch22-minimize-visibility/README.md) 2.5절(`pub use`로 정하는 공개 경로), [ch24](../ch24-re-export-dependencies/README.md)(의존성 재내보내기)

rustdoc의 기능 전반(명령줄 인자, `#[doc]` 속성 전체, 문서 테스트의 코드 블록 속성과 숨김 줄)은 Part 4 `ch1-rustdoc-basics`·`ch2-writing-documentation`·`ch3-doc-tests`·`ch4-lints-advanced`에서, 테스트 종류의 역할은 [`ch30-beyond-unit-tests`](../ch30-beyond-unit-tests/README.md)에서, 린트를 프로젝트 단위로 켜고 수준을 고르는 방법은 [`ch29-listen-to-clippy`](../ch29-listen-to-clippy/README.md)에서, CI에 올리는 일은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 문서 주석에 쓸 것(전제 조건, 불변식, 놀라운 동작, 왜 그렇게 했고 왜 다르게 하지 않았는지)과 쓰지 말 것(시그니처의 되풀이, 부르는 쪽의 사정)을 구별해 부실한 문서를 고칠 수 있다.
2. 문서의 문장으로만 알리던 조건을 `NonZero`나 검증 뉴타입 같은 타입으로 옮길 수 있는지 판단할 수 있다.
3. `# Errors`·`# Panics`·`# Safety` 절에 실패 조건을 빠짐없이 적고, 적은 약속을 실행해서 확인할 수 있다.
4. 크레이트 문서(`//!`)와 `# Examples`로 입구를 만들고, 항목 링크로 문서를 잇고, `pub use`한 항목이 생성된 문서의 어디에 보이는지 설명할 수 있다.
5. 문서 관련 린트의 기본 수준과 잡는 것·못 잡는 것을 설명하고, `cargo doc`과 `cargo test --doc`으로 문서가 코드와 어긋나지 않았는지 검사할 수 있다.

## 2. 핵심 개념

2.1절에서 문서 주석과 rustdoc을 정리하고, 2.2~2.7절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 지침 1~3은 `examples/`로, 지침 4~6은 [`projects/doc_quality`](projects/doc_quality)로 확인합니다. 그 프로젝트의 `poor/`와 `improved/`는 이름도 함수 본문도 같은 라이브러리 `kvconf`(`키 = 값` 설정 파일 읽기)이고, 문서와 모듈 공개 여부만 다릅니다.

### 2.1 지도: 문서 주석과 rustdoc

**두 가지 문서 주석.** `///`는 바로 뒤에 오는 항목의 문서이고, `//!`는 그것을 감싸는 항목의 문서입니다. `src/lib.rs` 맨 위의 `//!`는 크레이트 문서가 되고, 모듈 파일 맨 위의 `//!`는 그 모듈의 문서가 됩니다. 둘 다 Markdown(CommonMark)으로 쓰며, rustdoc이 이것을 HTML로 바꿉니다. 둘 다 "주석의 다른 모양"이 아니라 붙을 대상이 정해진 문법이라서, 대상이 없는 곳에 쓰면 컴파일 에러가 납니다(4.1·4.2절).

**rustdoc이 이미 보여 주는 것.** 생성된 문서는 항목마다 시그니처를 맨 위에 보여 주고, 시그니처 속 타입마다 링크를 겁니다. 트레이트 구현, derive, 공개 필드도 자동으로 나옵니다. 그래서 문서 주석이 할 일은 **그 밖의 것**입니다. 첫 빈 줄 앞까지의 첫 문단은 요약으로 따로 쓰입니다. 모듈의 항목 목록과 검색 결과에 이 문단만 나오므로 한두 문장으로 짧게 씁니다(rustdoc Book, How to write documentation).

**정해진 절 이름.** `# Examples`, `# Panics`, `# Errors`, `# Safety`는 표준 라이브러리와 생태계가 함께 쓰는 제목입니다. 한국어 문서에서도 영어 그대로 씁니다. clippy가 이 제목을 글자 그대로 찾기 때문입니다(2.4절).

**명령.**

| 명령 | 하는 일 |
| --- | --- |
| `cargo doc` | 크레이트와 모든 의존성의 문서를 `target/doc/<크레이트>/index.html`에 만듭니다. |
| `cargo doc --no-deps` | 의존성은 빼고 이 크레이트만 만듭니다. |
| `cargo doc --open` | 만든 뒤 브라우저로 엽니다. 쓴 문서를 렌더링된 모양으로 읽는 가장 쉬운 방법입니다. |
| `cargo doc --example <이름>` | 예제 파일의 문서를 만듭니다. 이 챕터의 예제는 `pub mod v1`·`pub mod v2`의 문서를 나란히 비교할 수 있습니다. |
| `cargo test --doc` | 문서 주석 속 코드 블록을 문서 테스트로 실행합니다. |

**누가 어디서 읽는가.** 책은 두 곳을 구분합니다. crates.io는 패키지의 `README.md`(`Cargo.toml`의 `readme` 필드)를 보여 주고, 이것을 읽는 사람은 **이 크레이트를 쓸지 고르는 사람**입니다. docs.rs는 `src/lib.rs`의 크레이트 문서를 첫 화면으로 보여 주고, 이것을 읽는 사람은 **이미 고른 크레이트를 쓰려는 사람**입니다. 저장소의 `examples/`와 `tests/`도 사용법을 보여 주는 자료입니다.

### 2.2 지침 1: 시그니처가 말하는 것은 되풀이하지 않고, 코드가 말하지 못하는 것을 쓴다

**문제 상황.** 정산 앱에서 총액을 사람 수로 나누는 함수를 공개합니다. 나누어떨어지지 않을 때 1원을 누가 더 내는지, 몫의 합이 총액과 같은지는 시그니처 `fn split(total: u64, people: usize) -> Vec<u64>`만 봐서는 알 수 없습니다.

**비권장 코드.** [`01_what_to_write.rs`](examples/01_what_to_write.rs)의 `v1` 문서입니다.

```rust
/// 금액을 나눕니다.
///
/// # Arguments
///
/// * `total` - 총액 (`u64`)
/// * `people` - 사람 수 (`u32`)
///
/// # Returns
///
/// 각자 낼 금액을 담은 `Vec<u64>`
///
/// 나누어떨어지지 않는 나머지는 버립니다.
///
/// 정산 화면에서 "나누기" 버튼을 누르면 이 함수를 부릅니다.
pub fn split(total: u64, people: usize) -> Vec<u64> { /* … */ }
```

주석이 길지만 독자가 얻는 것은 없고, 세 가지가 틀렸거나 틀리게 됩니다.

- **시그니처의 되풀이.** 매개변수 이름과 타입, 반환 타입은 rustdoc이 문서 바로 위에 보여 줍니다. 같은 정보를 손으로 적어 두면 시그니처를 바꾸는 리팩터링 때 문서만 남습니다. 이 예의 `people`은 `u32`에서 `usize`로 바뀌었는데 문서는 그대로입니다. 렌더링된 페이지에는 맨 위의 `pub fn split(total: u64, people: usize)`와 그 아래 Arguments 절의 `people - 사람 수 (u32)`가 한 화면에 함께 나옵니다.
- **부르는 쪽의 사정.** "정산 화면의 버튼"은 지금 이 함수를 부르는 곳의 이야기입니다. 부르는 곳이 늘거나 바뀌면 낡고, 함수가 무엇을 하는지에 대해서는 아무것도 알려 주지 않습니다.
- **낡은 동작 설명.** "나머지는 버립니다"는 예전 구현의 설명입니다. 컴파일러는 문장을 읽지 않으므로 이런 어긋남은 아무 경고 없이 남습니다. 예제가 실행해 보면 `split(10000, 3)`의 합계는 10000이고, 이 문장은 `틀림`으로 나옵니다.

**권장 코드.** `v2`는 본문이 같고 문서만 다릅니다.

```rust
/// `total`원을 `people`명이 낼 몫으로 나눕니다.
///
/// 나누어떨어지지 않으면 앞사람부터 1원씩 더 냅니다. 그래서 몫을 모두 더하면 언제나
/// `total`이고, 가장 많이 내는 사람과 가장 적게 내는 사람의 차이는 1원을 넘지 않습니다.
/// 결과의 순서가 곧 사람의 순서이므로, 1원을 더 낼 사람을 정하려면 부르기 전에 순서를
/// 정해 둡니다.
///
/// 금액을 `f64`로 나눈 뒤 반올림하지 않는 이유는, 그렇게 하면 몫의 합이 `total`과
/// 어긋날 수 있기 때문입니다(100원을 3명이 나누면 33원씩, 합계 99원).
///
/// # Panics
///
/// `people`이 0이면 패닉합니다.
```

적은 것은 코드에서 바로 보이지 않는 것들입니다. **불변식**(합계가 보존되고 차이는 1원 이하), **놀라운 동작**(앞사람이 더 낸다, 결과의 순서에 뜻이 있다), **왜 다른 방법이 아닌지**(반올림하지 않는 이유)입니다. 그리고 모두 검사할 수 있는 문장입니다. 예제의 `main`은 `total` 0~200, `people` 1~9의 1809가지 입력으로 세 문장을 확인하고, "하지 않는 이유"로 든 반올림 방식이 정말 합계 99를 내는 것도 보여 줍니다(3절). 문서의 문장을 이렇게 실행으로 확인할 수 있게 쓰면, 문장이 코드와 어긋났을 때 알아챌 방법이 생깁니다.

표준 라이브러리도 같은 기준으로 씁니다. `Vec::remove`의 문서는 시그니처를 풀어 쓰지 않고, 뒤 요소를 당기므로 최악의 경우 O(n)이 든다는 것과 순서가 필요 없으면 `swap_remove`를 쓰라는 대안을 적습니다. 책은 표준 라이브러리 문서를 본보기로 삼으라고 권합니다.

**따르지 않아도 되는 경우.**

- **이름과 시그니처로 충분한 항목.** 03의 `page_count`처럼 뻔한 항목은 "쪽 수를 돌려줍니다" 한 줄이면 됩니다. 이 한 줄은 시그니처의 되풀이라기보다 목록과 검색에 보일 요약입니다. 억지로 내용을 늘릴 필요는 없습니다.
- **매개변수의 뜻이 타입에서 드러나지 않을 때.** `timeout: u64`가 초인지 밀리초인지는 문서에 적어야 합니다. 이때도 `# Arguments` 목록보다 문장 하나가 낫고, 더 나은 답은 `Duration` 같은 타입입니다. 다음 지침의 이야기입니다.

### 2.3 지침 2: 타입으로 말할 수 있는 조건은 문서 대신 타입에 맡긴다

**문제 상황.** 같은 정산 앱에 할인 함수가 있습니다. 사람 수는 0이면 안 되고 할인율은 100 이하여야 합니다. 책은 지나가며 이렇게 말합니다. 타입 시스템으로 표현한 뜻은 타입에 맡기고 문서에서는 뺀다(Item 1, 부록 [ch1](../ch1-express-data-structures/README.md)).

**비권장 코드.** [`02_types_over_prose.rs`](examples/02_types_over_prose.rs)의 `v1`은 조건을 문장으로만 알립니다.

```rust
/// `price`원에서 `percent`퍼센트를 깎은 값을 돌려줍니다. 1원 미만은 버립니다.
///
/// `percent`는 100 이하여야 합니다. 100보다 크면 디버그 빌드에서는 패닉하고, 릴리스
/// 빌드에서는 엉뚱하게 큰 값을 돌려줍니다.
pub fn discount(price: u64, percent: u8) -> u64 {
    price * u64::from(100 - percent) / 100
}
```

문서는 정확합니다. 하지만 컴파일러는 그 문장을 읽지 않으므로, 설정 파일에서 읽은 120을 그대로 넘기는 코드도 컴파일됩니다. 디버그 빌드에서는 `attempt to subtract with overflow`로 패닉하고, `--release`로 실행하면 `100 - 120`이 236으로 넘쳐 5000원짜리 물건의 할인가가 11800원이 됩니다(3절). `split`의 `# Panics`(0명)도 같은 처지입니다.

**권장 코드.** `v2`는 조건을 매개변수 타입으로 옮깁니다.

```rust
pub fn split(total: u64, people: NonZero<usize>) -> Vec<u64> { /* … */ }

/// 0 이상 100 이하의 백분율입니다.
///
/// 범위를 확인하는 [`Percent::new`]로만 만들 수 있으므로, 이 타입의 값은 늘 범위 안에 있습니다.
pub struct Percent(u8);

impl Percent {
    /// `value`가 100 이하이면 `Some`, 아니면 `None`을 돌려줍니다.
    pub fn new(value: u8) -> Option<Percent> { /* … */ }
}

/// `price`원에서 `percent`만큼 깎은 값을 돌려줍니다. 1원 미만은 버립니다.
pub fn discount(price: u64, percent: Percent) -> u64 { /* … */ }
```

달라진 것은 둘입니다. 첫째, **조건을 검사하는 자리가 함수 안에서 값을 만드는 자리로 옮겨 갑니다.** 바깥에서 들어온 0과 120은 `NonZero::new(0)`와 `Percent::new(120)`에서 `None`이 되어, 함수를 부르기 전에 처리됩니다. 둘째, **문서가 짧아집니다.** 조건을 시그니처가 말하므로 지침 1에 따라 문서에서 빼고, `# Panics` 절이 사라집니다. 남은 문서는 타입이 말하지 못하는 것("1원 미만은 버립니다")뿐입니다. 검증하는 생성자를 만드는 법은 [ch6](../ch6-newtype-pattern/README.md) 2.3절에서 다뤘습니다.

**따르지 않아도 되는 경우.**

- **조건이 다른 값과의 관계일 때.** 03의 `page(n)`은 "`n`이 문서의 쪽 수 이하"라는 조건을 가집니다. 쪽 수는 문서마다 다르므로 `n`의 타입만으로는 표현하기 어렵습니다. 이런 조건은 문서에 남깁니다(다음 지침).
- **이미 공개한 시그니처일 때.** 매개변수 타입을 바꾸는 것은 호환되지 않는 변경입니다([ch21](../ch21-semver/README.md)). 다음 메이저 버전까지는 문서와 검사로 버팁니다.

### 2.4 지침 3: 실패 조건은 `# Errors`·`# Panics`·`# Safety` 절에 쓴다

**문제 상황.** 인쇄할 쪽 범위(`"3-7"`)를 해석하고 쪽의 내용을 꺼내는 API입니다. 범위 해석은 여러 이유로 실패하고, 쪽 꺼내기는 없는 쪽이면 패닉하며, 검사를 건너뛰는 `unsafe` 판도 있습니다. 실패는 시그니처에 `Result`나 `unsafe`로 흔적만 남고, **언제** 실패하는지는 코드를 읽어야 압니다.

**비권장 코드.** [`03_failure_sections.rs`](examples/03_failure_sections.rs)의 `v1` 문서입니다.

```rust
/// 쪽 범위를 파싱합니다. 잘못된 입력이면 실패할 수 있습니다.
pub fn parse_range(&self, spec: &str) -> Result<RangeInclusive<usize>, RangeError>
/// 쪽을 가져옵니다.
pub fn page(&self, n: usize) -> &str
/// 쪽을 빠르게 가져옵니다.
pub unsafe fn page_unchecked(&self, n: usize) -> &str
```

"실패할 수 있습니다"는 `Result`를 말로 옮겼을 뿐이고, `page`가 패닉한다는 사실과 `page_unchecked`를 부를 때 지켜야 할 조건은 아예 없습니다. 부르는 쪽은 어떤 입력을 미리 걸러야 하는지, `unsafe` 블록의 안전성 주석에 무엇을 써야 하는지 알 수 없습니다.

**권장 코드.** `v2`는 실패마다 정해진 절을 둡니다.

```rust
/// `"3-7"`이나 `"4"` 같은 쪽 범위를 해석합니다. 범위는 양 끝을 포함하고, 숫자 앞뒤의
/// 공백은 무시합니다.
///
/// # Errors
///
/// - 공백뿐이면 [`RangeError::Empty`]
/// - `-`의 앞이나 뒤가 0 이상의 정수가 아니면 [`RangeError::NotANumber`]
/// - 시작이 끝보다 크면 [`RangeError::Reversed`]
/// - 0쪽이나 [`page_count`](Self::page_count)보다 큰 쪽이 있으면
///   [`RangeError::OutOfDocument`]
pub fn parse_range(&self, spec: &str) -> Result<RangeInclusive<usize>, RangeError>

/// `n`쪽의 내용을 돌려줍니다.
///
/// # Panics
///
/// `n`이 0이거나 [`page_count`](Self::page_count)보다 크면 패닉합니다. `n`이 사용자
/// 입력처럼 바깥에서 왔다면 [`parse_range`](Self::parse_range)로 먼저 확인합니다.
pub fn page(&self, n: usize) -> &str

/// `n`쪽의 내용을 범위 검사 없이 돌려줍니다.
///
/// # Safety
///
/// `1 <= n <= self.page_count()`이어야 합니다. 어기면 미정의 동작입니다.
pub unsafe fn page_unchecked(&self, n: usize) -> &str
```

| 절 | 적는 것 | 근거 |
| --- | --- | --- |
| `# Errors` | 에러 배리언트마다 그 에러가 나는 조건 | Rust API 가이드라인 C-FAILURE (책에 없음, 2.8절) |
| `# Panics` | 패닉하는 조건, 그리고 피하는 방법이나 패닉하지 않는 대안 | 책, [ch18](../ch18-dont-panic/README.md) 2.3절 |
| `# Safety` | 부르는 쪽이 지켜야 할 조건 전부. 코드에서 보이지 않으므로 문서가 유일한 전달 수단 | 책, [ch16](../ch16-avoid-unsafe/README.md) 2.6절 |

표준 라이브러리의 `String::from_utf8`(Errors), `Vec::split_off`(Panics: `at > len`이면 패닉), `<[T]>::get_unchecked`(Safety: 범위 밖 인덱스는 결과를 쓰지 않아도 미정의 동작, 안전한 대안은 `get`)가 같은 모양입니다. 예제의 `main`은 `v2` 문서의 목록을 표로 옮겨 입력마다 실행하고, 결과가 모두 문서와 일치하는지 확인합니다(`모든 결과가 문서의 목록과 일치: true`). `page_unchecked`는 `# Safety`의 조건을 바로 위에서 확인한 뒤에만 부르고, 안전성 주석이 그 조건을 가리킵니다.

**린트가 확인하는 것.** clippy에는 세 절을 확인하는 린트가 있습니다. 예제의 `v1`에 붙인 `#[expect]`를 지우고 실행하면, 기본 설정에서는 `missing_safety_doc` 하나만 나옵니다.

```text
warning: unsafe function's docs are missing a `# Safety` section
   = note: `#[warn(clippy::missing_safety_doc)]` on by default
```

`missing_errors_doc`와 `missing_panics_doc`는 pedantic 그룹이라 기본으로 꺼져 있고, `-W`로 켜야 나옵니다(`cargo clippy -p er-ch27-document-public-interfaces --example 03_failure_sections -- -W clippy::missing_errors_doc -W clippy::missing_panics_doc`).

```text
warning: docs for function returning `Result` missing `# Errors` section
warning: docs for function which may panic missing `# Panics` section
note: first possible panic found here
   |             assert!(n >= 1 && n <= count, "{n}쪽은 없습니다 (1..={count})");
```

예제 바이너리의 `pub` 항목에도 이 린트들이 걸린다는 것을 함께 볼 수 있습니다. 다만 린트는 **절의 제목**과 **본문에 보이는 패닉**만 봅니다. 임시 크레이트로 확인해 보니 `missing_panics_doc`는 본문의 `panic!`, `assert!`, `unwrap`, `expect`(클로저 안 포함)는 찾았지만, 인덱싱(`v[i]`), 나눗셈(`a / b`), 패닉하는 다른 함수의 호출, `unreachable!`은 찾지 못했습니다. 01의 `split`이 0으로 나눌 때 패닉하는 것도 이 린트는 모릅니다. `missing_errors_doc`는 내용이 빈 `# Errors` 제목 하나로 조용해졌습니다. 조건을 빠짐없이 적었는지는 사람이 확인해야 합니다(연습 3).

**따르지 않아도 되는 경우.** API 가이드라인은 생각할 수 있는 모든 패닉을 적을 필요는 없다고 합니다. 부르는 쪽이 넘긴 코드(`Display` 구현이나 클로저)가 일으키는 패닉이 그런 예입니다. 라이브러리 내부의 불변식이 깨졌을 때만 나는 패닉, 즉 부르는 쪽이 일으킬 수 없는 패닉도 `# Panics`에 적을 대상이 아닙니다. 그것은 라이브러리의 버그입니다([ch18](../ch18-dont-panic/README.md)). 다른 함수의 에러를 그대로 돌려주는 얇은 함수라면 목록을 되풀이하지 말고 "[`parse`]의 에러를 그대로 돌려줍니다"처럼 링크로 넘깁니다.

### 2.5 지침 4: 크레이트 문서로 입구를 만들고, 예제는 문서 테스트로 검사되게 둔다

**문제 상황.** 처음 보는 크레이트의 문서를 열면 크레이트 문서가 첫 화면입니다. 이 크레이트가 무엇을 하는지, 어디서 시작하는지가 여기서 정해집니다.

**비권장 코드.** `poor/`에는 크레이트 문서가 없습니다. 생성된 첫 화면(`target/doc/kvconf/index.html`)에는 설명 없이 다음만 나옵니다.

```text
Crate kvconf
Re-exports
  pub use config::Config;
  pub use parse::ParseError;
  pub use parse::parse;
Modules
  config
  parse
```

사용 예는 `parse`의 문서 안에 ```` ```text ```` 블록으로만 있습니다. `text` 블록은 문서 테스트가 아니므로 `cargo test --doc`은 `running 0 tests`이고, 그 사이 예제 속 `config.value("port")`는 이름이 바뀐 메서드를 부르는 낡은 코드가 되었습니다. 복사한 사람만 그 사실을 알게 됩니다.

**권장 코드.** `improved/src/lib.rs`의 크레이트 문서는 무엇을 하는지(첫 문장), 무엇을 하지 않는지와 그럴 때 무엇을 쓸지(범위), 놀라기 쉬운 형식 규칙(첫 `=`에서 나눔, 줄 중간의 `#`은 값), 실행되는 예제를 차례로 둡니다.

```rust
//! `키 = 값` 줄로 된 작은 설정 파일을 읽습니다.
//!
//! 표준 라이브러리만 쓰고, 형식도 일부러 작게 잡았습니다. 섹션(`[server]`), 따옴표,
//! 줄 끝 주석, 여러 줄에 걸친 값은 지원하지 않습니다. …
//!
//! # 형식
//! - 한 줄에 `키 = 값`을 하나 씁니다. 첫 `=`에서 나누므로 값에는 `=`가 들어가도 됩니다.
//! …
//! # Examples
//!
//! ```
//! let config = kvconf::parse(
//!     "# 서버 설정
//!      host = 127.0.0.1
//!      port = 8080
//!      motd = 어서 오세요 # 줄 끝 주석이 아닙니다",
//! )?;
//!
//! assert_eq!(config.get("port"), Some("8080"));
//! assert_eq!(config.get("motd"), Some("어서 오세요 # 줄 끝 주석이 아닙니다"));
//! …
//! # Ok::<(), kvconf::ParseError>(())
//! ```
```

예제는 `?`로 에러를 다룹니다. 독자는 예제를 복사해 쓰므로 `unwrap`보다 낫다는 것이 API 가이드라인(C-QUESTION-MARK)과 rustdoc Book의 권고입니다. 마지막 `# Ok::<…>(())` 줄은 렌더링된 문서에는 보이지 않고 문서 테스트를 컴파일할 때만 쓰이는 숨김 줄로, `?`가 동작하게 해 줍니다. 숨김 줄의 문법은 Part 4 `ch3-doc-tests`에서 다룹니다.

이 예제는 문서 테스트로 실행됩니다. `improved/`에서 `cargo test`를 실행하면 크레이트 문서와 `parse`의 `# Examples`가 실행됩니다.

```text
running 2 tests
test src/lib.rs - (line 16) ... ok
test src/parse.rs - parse::parse (line 60) ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.37s; merged doctests compilation took 0.13s
```

마지막 줄은 2024 에디션의 문서 테스트 병합 때문에 붙습니다. 시간과 두 `test` 줄의 순서는 실행마다 다릅니다. 테스트 이름이 `parse::parse`인 것은 공개 경로(`kvconf::parse`)가 아니라 정의된 자리(비공개 모듈 `parse`)를 따르기 때문입니다.

문서 테스트가 무엇을 막는지는 리팩터링을 흉내 내 보면 보입니다. `improved/`의 사본에서 `Config::get`을 `lookup`으로 바꾸고 코드 속 호출만 고치자(문서는 그대로), `cargo build`는 그대로 통과했지만 `cargo doc --no-deps`는 깨진 링크로, `cargo test --doc`은 크레이트 문서의 예제에서 ``error[E0599]: no method named `get` found for struct `Config` ``로 실패했습니다(둘 다 종료 코드 101). `poor/`의 `text` 블록은 같은 상황에서 아무 일도 일으키지 않았습니다.

**README와 크레이트 문서는 독자가 다릅니다.** `improved/README.md`는 crates.io에서 **고르는 사람**을 위해 무엇을 지원하고 무엇을 지원하지 않는지만 짧게 적고, 사용법은 크레이트 문서와 `examples/startup.rs`로 보냅니다. `readme` 필드는 적지 않아도 Cargo가 패키지 루트의 `README.md`를 찾아 씁니다(필드를 지운 사본에서 `cargo metadata`가 `README.md`를 보고함). 다른 파일을 쓰거나 끄려면 필드를 적습니다. `examples/startup.rs`는 `cargo run --example startup`으로 실행되는 사용 예이고, 책은 `examples/`와 `tests/`도 사용법을 보여 주는 자료로 꼽습니다.

**따르지 않아도 되는 경우.**

- **모든 항목에 `# Examples`를 달 필요는 없습니다.** 책은 진입점이 뻔하지 않을 때 고려하라고 합니다. `improved/`는 크레이트 문서(첫 사용)와 `parse`(에러를 맞춰 보는 법)에만 두고, `Config::get`에는 두지 않았습니다. API 가이드라인 C-EXAMPLE은 "적당한 범위에서" 모든 항목에 예제를 두라고 더 강하게 권합니다.
- **두 독자가 겹치는 작은 크레이트.** `#![doc = include_str!("../README.md")]`로 README를 크레이트 문서로 함께 쓰는 방법도 있습니다. README의 코드 블록이 문서 테스트가 되고, 그 안의 경로 기준이 에디션마다 다르다는 점은 [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.9절에서 봤습니다.

### 2.6 지침 5: 코드 서식과 항목 링크로 잇고, 공개 경로가 문서에 어떻게 보이는지 확인한다

**문제 상황.** 문서에는 다른 항목의 이름이 자주 나옵니다(`page`의 대안은 `parse_range`, 에러는 `RangeError::Empty`). 이름을 그냥 글자로 적으면 독자가 직접 찾아가야 하고, 이름이 바뀌어도 아무도 모릅니다.

**권장 코드: 백틱과 항목 링크.** 소스에 그대로 치는 것은 백틱으로 감싸 코드 서식으로 쓰고, 다른 항목은 ``[`이름`]``으로 링크합니다. 이것을 항목 링크(intra-doc link)라고 부릅니다. 이 챕터에서 쓴 모양은 다음과 같습니다.

| 쓴 것 | 가리키는 것 |
| --- | --- |
| ``[`Config`]``, ``[`parse`]`` | 지금 스코프에 있는 이름 |
| ``[`RangeError::Empty`]`` | 열거형의 배리언트 |
| ``[`page_count`](Self::page_count)`` | 보이는 글자는 `page_count`, 대상은 `Self::page_count` |
| ``[`parse`](crate::parse)``, ``[크레이트 문서](crate)`` | 크레이트 루트 기준 경로, 크레이트 문서 자체 |

링크는 **항목이 정의된 모듈의 스코프에서** 풀립니다(rustdoc Book, Linking to items by name). `improved/src/config.rs`가 ``[`parse`](crate::parse)``로 적은 것은 `config` 모듈 안에서는 `parse`라는 이름이 보이지 않기 때문입니다. 사본에서 ``[`parse`]``로 줄이자 ``unresolved link to `parse` ``(``no item named `parse` in scope``)로 실패했습니다. 표준 라이브러리 항목에 건 링크는 `https://doc.rust-lang.org/1.100.0/…`처럼 툴체인 버전의 온라인 문서로 연결되었습니다.

**비권장 코드: 깨진 링크와 비공개 항목 링크.** `poor/src/config.rs`에는 이름이 바뀐 메서드(`value` → `get`)를 가리키는 링크와 `pub(crate)` 메서드를 가리키는 링크가 있습니다. `cargo build`와 기본 설정의 `cargo clippy`는 아무 말이 없고, `cargo doc --no-deps`만 경고합니다.

```text
warning: public documentation for `Config` links to private item `Config::insert`
 --> src/config.rs:1:15
  |
1 | /// 설정. 항목은 [`Config::insert`]로 넣습니다.
  |                    ^^^^^^^^^^^^^^ this item is private
  |
  = note: this link will resolve properly if you pass `--document-private-items`
  = note: `#[warn(rustdoc::private_intra_doc_links)]` on by default

warning: unresolved link to `Config::value`
  --> src/config.rs:16:21
   |
16 |     /// 값을 가져옵니다. [`Config::value`]와 같지만 `Option`이 아니라 `&str`을 돌려줍니다.
   |                            ^^^^^^^^^^^^^ the struct `Config` has no field or associated item named `value`
   |
   = note: `#[warn(rustdoc::broken_intra_doc_links)]` on by default
```

경고일 뿐이라 종료 코드는 0이고, 문서는 만들어집니다. 렌더링된 페이지에서 깨진 링크는 `[Config::value]`라는 대괄호 글자로 남고, 비공개 항목 링크는 그 페이지에 없는 `#method.insert`를 가리키는 죽은 링크가 됩니다. `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`로 실행하면 두 경고가 에러가 되어 종료 코드 101로 멈춥니다. `improved/`는 책의 권고대로 `src/lib.rs`에 `#![deny(rustdoc::broken_intra_doc_links)]`를 두었습니다. rustdoc 린트는 rustc가 검사하지 않으므로, **`cargo doc`을 실행하는 곳에서만** 걸린다는 점을 기억합니다. 책은 이것을 CI에서 돌리라고 권합니다([`ch32-continuous-integration`](../ch32-continuous-integration/README.md)).

**재내보낸 항목이 보이는 모양.** [ch22](../ch22-minimize-visibility/README.md)와 [ch24](../ch24-re-export-dependencies/README.md)가 미뤄 둔 내용입니다. 두 판의 차이는 모듈 공개 여부뿐인데, 생성된 문서의 구조가 다릅니다. 표의 앞 두 줄은 `projects/doc_quality`에서, 나머지는 path 의존성을 둔 임시 크레이트에서 `cargo doc`으로 만든 HTML을 읽어 확인했습니다.

| 소스 | 생성된 문서 |
| --- | --- |
| 비공개 모듈의 항목을 `pub use` (`improved/`) | 루트에 항목 페이지(`struct.Config.html`)가 생기고, Structs·Enums·Functions 목록에 루트에 정의된 것처럼 나옴. 원래 자리(`config/struct.Config.html`)는 루트로 넘겨 주는 페이지 |
| 공개 모듈의 항목을 `pub use` (`poor/`) | 루트의 Re-exports 절에 `pub use config::Config;` 한 줄과 링크. 항목 페이지는 `config/struct.Config.html`에 하나뿐이고, 제목의 경로도 `kvconf::config::Config` |
| 공개 모듈의 항목 + `#[doc(inline)]` | 루트와 모듈 두 곳에 항목 페이지 |
| 비공개 모듈의 항목 + `#[doc(no_inline)]` | Re-exports 절에 링크 없는 `pub use hidden_mod::NoInline;` 한 줄. 항목 페이지가 어디에도 없음 |
| 다른 크레이트의 항목 `pub use money::Won;` | 루트에 항목 페이지(의존성의 문서 주석을 가져옴) |
| 다른 크레이트 자체 `pub use money;` | Re-exports 절에 한 줄. 의존성 문서도 만들 때(`--no-deps` 없이)만 링크가 걸림 |
| `#[doc(hidden)] pub fn secret()` | 문서에 나오지 않음. 그래도 의존하는 다른 크레이트에서 `secret()`을 부를 수 있음 |

판단은 이렇습니다. **기본(비공개 모듈 + `pub use`)이 대개 맞습니다.** ch22의 권고대로 모듈 구조를 감추면 문서에도 경로가 하나만 보입니다. `#[doc(inline)]`은 공개 모듈과 루트 양쪽에 같은 페이지를 보여 줄 이유가 있을 때만 씁니다. `#[doc(no_inline)]`을 비공개 모듈의 항목에 붙이면 독자가 그 항목의 문서를 볼 방법이 사라집니다. `#[doc(hidden)]`은 문서에서만 감출 뿐 **가시성을 바꾸지 않습니다.** 숨긴 항목도 공개 API이고 호환성 약속([ch21](../ch21-semver/README.md))의 대상입니다. 쓰는 사람이 쓰면 안 되는 항목은 `pub(crate)`로 닫는 것이 먼저입니다(API 가이드라인 C-HIDDEN). 각 속성의 세부 규칙은 Part 4 `ch2-writing-documentation`에서 다룹니다.

**따르지 않아도 되는 경우.** 모든 이름을 링크할 필요는 없습니다. 같은 문단에서 같은 이름이 거듭 나오면 처음 한 번만 링크해도 충분합니다. 반대로 백틱은 소스에 치는 것이면 언제나 붙입니다. clippy의 `doc_markdown`(pedantic)을 켜면 백틱 밖의 `ParseError`, `line_no` 같은 낱말에 경고합니다(한글 조사가 붙어도 찾았지만, `Config`처럼 대문자 한 덩어리인 낱말은 찾지 못했습니다).

### 2.7 지침 6: 문서 린트는 빈칸을 찾을 뿐이니, 렌더링된 문서를 직접 읽는다

**문제 상황.** 공개 항목이 늘면 문서가 빠진 곳이 생깁니다. 린트로 빈칸을 찾을 수 있지만, 책은 `missing_docs`가 유용한 문서의 대리 지표일 뿐이고 경고를 끄려고 서둘러 쓴 부실한 주석을 낳을 수 있다고 경고합니다.

**문서 관련 린트.** 1.100.0의 기본 수준은 `rustc -W help`, `clippy-driver -W help`, rustdoc Book의 Lints 장과 직접 실행으로 확인했습니다.

| 린트 | 도구 | 기본 수준 | 잡는 것 |
| --- | --- | --- | --- |
| `missing_docs` | rustc | allow | 문서 없는 공개 항목(크레이트, 모듈, 필드, 배리언트 포함) |
| `unused_doc_comments` | rustc | warn | rustdoc이 문서를 만들지 않는 곳(함수 본문의 문장 등)의 `///` |
| `rustdoc::broken_intra_doc_links` | rustdoc | warn | 풀리지 않거나 모호한 항목 링크 |
| `rustdoc::private_intra_doc_links` | rustdoc | warn | 공개 문서에서 비공개 항목으로 건 링크 |
| `rustdoc::missing_crate_level_docs` | rustdoc | allow | 크레이트 문서가 없음 |
| `clippy::missing_safety_doc` | clippy | warn | `# Safety`가 없는 `pub unsafe fn` |
| `clippy::missing_errors_doc`, `missing_panics_doc` | clippy | allow (pedantic) | `# Errors`, `# Panics`가 없음 |
| `clippy::doc_markdown` | clippy | allow (pedantic) | 백틱 밖의 코드 낱말 |

`missing_docs`는 rustc 린트라서 `cargo build`에서도 검사되고, 바이너리 크레이트의 `pub` 항목에도 걸립니다(임시 파일로 확인). `poor/`에서 세 린트를 켜 보면(`cargo clippy -- -W missing_docs -W clippy::missing_errors_doc -W clippy::missing_panics_doc`) 경고 12개가 나옵니다. 크레이트 문서 1, 모듈 2, 배리언트 3, 배리언트의 필드 4, `# Errors` 1, `# Panics` 1입니다.

```text
warning: docs for function which may panic missing `# Panics` section
warning: docs for function returning `Result` missing `# Errors` section
warning: missing documentation for the crate
warning: missing documentation for a module
warning: missing documentation for a variant
warning: missing documentation for a struct field
…
warning: `kvconf` (lib) generated 12 warnings
```

**린트가 못 잡는 것.** 같은 `poor/`에서 린트가 하나도 짚지 못한 문제가 더 많습니다. `parse`의 `# Arguments`가 `text`를 `String`이라고 적은 것(시그니처는 `&str`), "같은 키가 여러 번 나오면 마지막 값이 이깁니다"라는 낡은 설명(실제로는 `DuplicateKey` 에러), "서버의 `main`이 … 부릅니다"라는 부르는 쪽 이야기, `get`의 문서가 시그니처를 옮긴 것(`` `key: &str`를 받아 `Option<&str>`을 돌려줍니다 ``), 에러의 `line`이 0부터인지 1부터인지 알 수 없는 것입니다. 모두 문서가 **있기는** 하므로 린트는 만족합니다. 반대로 `missing_docs`를 맞추려고 `/// 적립.`처럼 이름을 되풀이하는 주석을 달아도 린트는 만족합니다([`compile_fail/deny_missing_docs.rs`](compile_fail/deny_missing_docs.rs), 4.3절).

**권장 코드.** `improved/`는 `src/lib.rs`에서 이 세 린트와 깨진 링크 린트를 켜고 경고 0개를 유지합니다. 그런데 `missing_docs`를 켜면 `ParseError`의 `line` 필드에도 문서가 필요해지는데, 여기서 "줄 번호"라고 이름을 되풀이하면 책이 경고한 그 주석이 됩니다. `improved/`는 필드마다 이름이 말하지 못하는 것(`1부터 센 줄 번호`, `두 번째로 나온 줄의 번호(1부터)`, `겹친 키(앞뒤 공백을 지운 것)`)을 적었습니다. 린트가 강제하는 자리에도 지침 1을 적용하는 것입니다. 그리고 쓴 뒤에는 `cargo doc --open`으로 렌더링된 결과를 직접 읽습니다. 이 챕터에서 "`poor/`의 첫 화면에 설명이 하나도 없다", "깨진 링크가 대괄호 글자로 남는다"를 알아낸 것도 생성된 HTML을 읽어서였습니다.

**따르지 않아도 되는 경우.** 책은 일회용 코드까지 이렇게 문서화할 필요는 없다고 말합니다. 혼자 쓰는 도구나 바이너리의 내부 모듈에 `missing_docs`를 켜면 얻는 것보다 이름 되풀이 주석이 더 많이 생깁니다. 반대로 남이 쓰는 라이브러리라면 rustdoc Book의 말대로 `#![warn(missing_docs)]`로 시작해 다 채운 뒤 `deny`로 올리는 길이 있습니다. 린트를 `Cargo.toml`의 `[lints]` 테이블로 켜는 방법(`[lints.rustdoc]`도 됨)과 수준을 고르는 기준은 [`ch29-listen-to-clippy`](../ch29-listen-to-clippy/README.md)에서 다룹니다.

### 2.8 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0과 cargo 1.100.0에서 확인한 차이는 다음과 같습니다.

- **깨진 링크 린트의 이름.** 책은 `#![deny(broken_intra_doc_links)]`로 적습니다. 로컬 문서의 릴리스 노트에 따르면 rustdoc 린트는 1.52.0부터 `rustdoc::` 접두사가 붙는 도구 린트가 되었습니다. 1.100.0에서 접두사 없이 쓰면 rustdoc이 ``lint `broken_intra_doc_links` has been renamed to `rustdoc::broken_intra_doc_links` `` 경고를 내고(린트 자체는 동작함), rustc는 아무 말도 하지 않았습니다. 책이 함께 언급한 옛 이름 `intra_doc_link_resolution_failure`도 같은 경고가 납니다.
- **`[lints]` 테이블.** Cargo 1.74.0부터 `Cargo.toml`의 `[lints.rustdoc]`로도 rustdoc 린트를 켤 수 있습니다. 임시 크레이트에서 `cargo doc -v`로 보니 Cargo가 `--deny=rustdoc::broken_intra_doc_links`를 rustc와 rustdoc 양쪽에 넘겼습니다.
- **문서 테스트 병합.** 2024 에디션에서는 문서 테스트를 한 바이너리로 묶어 컴파일합니다(2.5절의 마지막 줄, [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.8절).
- **책에 없는 지침과 내용.** 다음은 책의 Item 27이 명시적으로 권하지 않는 것입니다(2026-11-20에 온라인판에서 확인). 책은 공개 API 문서화, 백틱과 항목 링크, `# Examples`·`# Panics`·`# Safety`, 표준 라이브러리를 본보기로 삼기, 렌더링 결과 읽기, 깨진 링크 린트와 `missing_docs`의 한계, CI, docs.rs와 crates.io의 독자, `examples/`·`tests/`, 시그니처를 되풀이하지 않기, "무엇을 왜"를 쓰기, 타입에 맡기기(지나가며)를 다룹니다.
  - 지침 3의 **`# Errors` 절**: 책의 절 목록에는 없습니다. Rust API 가이드라인 C-FAILURE(에러·패닉·안전성 고려 사항을 함수 문서에 쓴다)가 근거이고, 패닉 절의 예외(부르는 쪽 코드가 일으키는 패닉)도 C-FAILURE에서 가져왔습니다.
  - 지침 1·3의 **"문서의 약속을 실행으로 확인한다"**(01·03의 `main`): 책은 문서 테스트가 코드와 어긋나지 않게 해 준다고만 말합니다. 실행 확인의 틀은 이 챕터가 더한 것입니다.
  - 지침 3·6의 **clippy 문서 린트**(`missing_safety_doc`·`missing_errors_doc`·`missing_panics_doc`·`doc_markdown`)와 그 한계(빈 절, 본문 밖의 패닉), rustc `unused_doc_comments`, rustdoc `private_intra_doc_links`·`missing_crate_level_docs`: `clippy-driver -W help`, rustdoc Book의 Lints 장, 직접 컴파일한 결과가 근거입니다.
  - 지침 5의 **재내보낸 항목이 보이는 모양**과 `#[doc(inline)]`·`#[doc(no_inline)]`·`#[doc(hidden)]`의 판단: rustdoc Book의 Re-exports 장, API 가이드라인 C-HIDDEN, 직접 생성한 HTML이 근거입니다.
  - 2.1절의 **첫 문단 요약**과 2.5절의 C-QUESTION-MARK·C-EXAMPLE: rustdoc Book의 How to write documentation과 API 가이드라인이 근거입니다.

### 2.9 기억할 것

- 시그니처와 부르는 쪽 이야기는 쓰지 않습니다. 불변식, 놀라운 동작, 왜 그렇게 했는지와 왜 다르게 하지 않았는지를 검사할 수 있는 문장으로 씁니다.
- 문장으로만 알리는 조건은 컴파일러가 읽지 않습니다. 타입으로 옮길 수 있으면 옮기고, 그러면 문서에서 그 조건이 빠집니다.
- 실패는 `# Errors`(에러마다 조건), `# Panics`(조건과 대안), `# Safety`(부르는 쪽이 지킬 조건 전부)에 씁니다. 린트는 제목만 봅니다.
- 크레이트 문서에 무엇을 하고 하지 않는지와 실행되는 `# Examples`를 둡니다. `text` 블록 예제와 손으로 쓴 이름은 조용히 낡고, 문서 테스트와 항목 링크는 깨지면 알려 줍니다.
- 문서 린트는 빈칸만 찾습니다. `cargo doc`(깨진 링크)과 `cargo test --doc`을 돌리고, 렌더링된 문서를 직접 읽습니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 끝나고, 02와 03은 패닉 훅을 바꿔 패닉 메시지를 표준 출력으로 옮기므로 표준 에러에는 아무것도 나오지 않습니다. `cargo doc -p er-ch27-document-public-interfaces --examples --no-deps`를 실행하면 `target/doc/01_what_to_write/index.html` 등에 예제의 `v1`·`v2` 문서가 생기니 렌더링된 모양을 나란히 읽어 보세요.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_what_to_write.rs`](examples/01_what_to_write.rs) | 1 | 시그니처 되풀이, 부르는 쪽 이야기, 낡은 설명 | 불변식·놀라운 동작·이유 | 2.2 |
| [`02_types_over_prose.rs`](examples/02_types_over_prose.rs) | 2 | 조건을 문장으로만 알림 | `NonZero<usize>`, `Percent` | 2.3 |
| [`03_failure_sections.rs`](examples/03_failure_sections.rs) | 3 | "실패할 수 있습니다", 절 없음 | `# Errors`·`# Panics`·`# Safety` | 2.4 |
| [`projects/doc_quality`](projects/doc_quality) | 4~6 | `poor/` | `improved/` | 2.5~2.7 |

### 01_what_to_write — 쓸 것과 쓰지 말 것

```sh
cargo run -p er-ch27-document-public-interfaces --example 01_what_to_write
```

```text
split(10000, 3) = [3334, 3333, 3333], 합계 10000
1. 비권장 v1 문서의 주장
  "나머지는 버립니다" → 틀림
  "people은 u32" → 틀림 (시그니처는 usize)
  나머지 문장(총액, 사람 수, Vec<u64>, 버튼)은 확인할 약속이 아님
2. 권장 v2 문서의 주장 (total 0..=200, people 1..=9의 모든 조합)
  1809가지 입력으로 확인
  "몫의 합은 언제나 total" → 맞음
  "앞사람부터 1원씩 더 낸다" → 맞음
  "차이는 1원을 넘지 않는다" → 맞음
3. v2 문서가 "하지 않는 이유"로 든 방식
  반올림 방식 split(100, 3) = [33, 33, 33], 합계 99
  v2::split(100, 3) = [34, 33, 33]
```

### 02_types_over_prose — 문장 대신 타입

```sh
cargo run -p er-ch27-document-public-interfaces --example 02_types_over_prose
```

```text
1. 비권장 v1: 조건이 문서의 문장에만 있음
  split(10000, 3) → [3334, 3333, 3333]
  split(10000, 0) → 패닉: attempt to divide by zero
  discount(5000, 30) → 3500
  discount(5000, 120) → 패닉: attempt to subtract with overflow
2. 권장 v2: 조건을 어긴 값은 값을 만드는 자리에서 걸러짐
  split(10000, 3) → [3334, 3333, 3333]
  NonZero::new(0) → None: 함수를 부르기 전에 처리
  discount(5000, 30) → 3500
  Percent::new(120) → None: 함수를 부르기 전에 처리
```

`--release`를 붙이면 넘침 검사가 꺼져 1번 구역의 넷째 줄이 `  discount(5000, 120) → 11800`으로 바뀝니다. 0으로 나누기는 릴리스 빌드에서도 패닉합니다.

### 03_failure_sections — 실패 조건을 적는 세 절

```sh
cargo run -p er-ch27-document-public-interfaces --example 03_failure_sections
```

```text
1. # Errors: 문서의 조건마다 그 에러가 나오는지 (10쪽 문서)
  "3-7" → Ok(3..=7)
  "4" → Ok(4..=4)
  "  " → Err(Empty)
  "3-x" → Err(NotANumber("x"))
  "7-3" → Err(Reversed { start: 7, end: 3 })
  "0-2" → Err(OutOfDocument { page: 0, page_count: 10 })
  "5-12" → Err(OutOfDocument { page: 12, page_count: 10 })
  모든 결과가 문서의 목록과 일치: true
2. # Panics: 문서의 조건에서만 패닉하는지
  page(1) → "표지"
  page(10) → "뒤표지"
  page(0) → 패닉: 0쪽은 없습니다 (1..=10)
  page(11) → 패닉: 11쪽은 없습니다 (1..=10)
3. # Safety: 문서의 조건을 확인한 뒤에만 page_unchecked를 부름
  n = 2: 조건 확인 통과 → "차례"
  n = 11: 조건 확인 실패 → 부르지 않음
4. 비권장 v1: 본문이 같으니 결과도 같지만, 문서로는 무엇을 대비할지 알 수 없음
  v1 parse_range("7-3") → Err(Reversed { start: 7, end: 3 })
  v1 page(11) → 패닉: 11쪽은 없습니다 (1..=10)
```

`v1`의 `#[expect(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::missing_safety_doc, …)]`를 지우면 2.4절의 clippy 출력을 볼 수 있습니다.

### projects/doc_quality — 같은 라이브러리의 두 문서

`poor/`와 `improved/`는 각자의 워크스페이스 루트이고 패키지 이름이 모두 `kvconf`입니다. 각 디렉터리 안에서 실행합니다.

```text
doc_quality/
├── poor/       문서가 부실한 판: 크레이트 문서 없음, pub mod, 깨진 링크, text 블록 예제
│   └── src/    lib.rs, config.rs, parse.rs
└── improved/   고친 판: 크레이트 문서와 # Examples, 비공개 모듈 + pub use, 문서 린트
    ├── README.md            crates.io용 소개
    ├── examples/startup.rs  사용 예
    └── src/                 lib.rs, config.rs, parse.rs (함수 본문은 poor와 같음)
```

```sh
cd appd-a/ch27-document-public-interfaces/projects/doc_quality/poor
cargo build                                  # 통과, 경고 없음
cargo clippy --all-targets -- -D warnings    # 통과 (문서 린트는 기본으로 꺼져 있음)
cargo doc --no-deps                          # 통과, rustdoc 경고 2개 (2.6절)
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps   # 일부러 실패: 종료 코드 101
cargo clippy -- -W missing_docs -W clippy::missing_errors_doc -W clippy::missing_panics_doc   # 경고 12개 (2.7절)
cargo test --doc                             # running 0 tests
```

```sh
cd ../improved
cargo clippy --all-targets -- -D warnings    # 통과 (lib.rs가 문서 린트를 켬)
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps   # 통과
cargo test                                   # 단위 테스트 0개, 문서 테스트 2개 통과 (2.5절)
cargo run --example startup
```

```text
host = 127.0.0.1
port = 8080
서버 주소: 127.0.0.1:8080
제한 시간: 30초(기본값)
설정 파일 오류: 2번째 줄에 `=`가 없습니다
```

두 판에서 `cargo doc --no-deps`를 실행한 뒤 `target/doc/kvconf/`의 파일 목록을 비교하면 2.6절의 재내보내기 차이가 보입니다. `poor/`는 `config/`·`parse/` 아래에만 항목 페이지가 있고, `improved/`는 루트에 `struct.Config.html`·`enum.ParseError.html`·`fn.parse.html`이 있습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 각 머리 주석의 명령으로 확인합니다. 출력의 경로는 챕터 디렉터리 기준입니다.

### 4.1 대상이 없는 곳의 `///` — E0585

[`compile_fail/e0585_doc_comment_documents_nothing.rs`](compile_fail/e0585_doc_comment_documents_nothing.rs)는 함수 본문 끝에 `///`로 메모를 남겼습니다.

```sh
rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0585_doc_comment_documents_nothing.rs
```

```text
error[E0585]: found a documentation comment that doesn't document anything
  --> compile_fail/e0585_doc_comment_documents_nothing.rs:17:5
   |
17 |     /// 할인은 여기서 적용하지 않습니다. 할인은 결제 단계의 일입니다.
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: doc comments must come before what they document, if a comment was intended use `//`
```

**원인.** `///`는 바로 뒤 항목의 문서라는 문법입니다. 블록의 끝에는 뒤따르는 것이 없어 붙을 대상이 없습니다. 같은 `///`를 `let` 문장 앞에 두면 에러 대신 `unused_doc_comments` 경고(`rustdoc does not generate documentation for statements`)가 납니다.

**해결.** 코드를 읽는 사람을 위한 설명은 `//`로 씁니다. 문서 주석은 공개 항목의 독자를 위한 것입니다.

### 4.2 항목 뒤에 온 `//!` — E0753

[`compile_fail/e0753_inner_doc_after_item.rs`](compile_fail/e0753_inner_doc_after_item.rs)는 크레이트 문서를 `use` 아래에 적었습니다.

```sh
rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0753_inner_doc_after_item.rs
```

```text
error[E0753]: expected outer doc comment
  --> compile_fail/e0753_inner_doc_after_item.rs:16:1
   |
16 |   //! 이름과 전화번호를 담는 작은 주소록입니다.
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
17 |
18 | / pub fn empty() -> HashMap<String, String> {
19 | |     HashMap::new()
20 | | }
   | |_- the inner doc comment doesn't annotate this function
   |
help: to annotate the function, change the doc comment from inner to outer style
```

**원인.** `//!`는 감싸는 것의 문서라서 감싸는 것의 내용, 즉 첫 항목보다 앞에만 올 수 있습니다.

**해결.** help는 `///`로 바꾸라고 하지만, 그러면 크레이트 소개가 `empty`의 문서가 됩니다. 아래 함수에 이미 `///` 문서가 있으면 help가 `//`로 바꾸라고 제안하는데, 이것도 크레이트 문서를 없애는 수정입니다. 의도가 크레이트 문서였다면 `//!` 줄들을 파일 맨 위로 옮깁니다. 컴파일러의 제안은 문법을 맞춰 줄 뿐 의도는 모릅니다.

### 4.3 `deny(missing_docs)` 아래의 문서 없는 공개 항목 — 에러 코드 없음

[`compile_fail/deny_missing_docs.rs`](compile_fail/deny_missing_docs.rs)는 `#![deny(missing_docs)]`를 켠 라이브러리입니다.

```sh
rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/deny_missing_docs.rs
```

```text
error: missing documentation for the crate
error: missing documentation for an enum
  --> compile_fail/deny_missing_docs.rs:17:1
error: missing documentation for a variant
  --> compile_fail/deny_missing_docs.rs:18:5
error: missing documentation for a variant
  --> compile_fail/deny_missing_docs.rs:19:5
error: missing documentation for a function
  --> compile_fail/deny_missing_docs.rs:22:1
error: aborting due to 5 previous errors
```

**원인.** 린트 에러라서 에러 코드가 없습니다. 크레이트, 공개 열거형과 배리언트, 공개 함수마다 문서가 필요하고, 비공개 함수 `helper`는 검사하지 않습니다.

**해결.** 문서를 씁니다. 이때 `/// 적립.`, `/// 포인트.`처럼 이름을 되풀이해 에러만 없애면 린트는 통과하지만 독자는 얻는 것이 없습니다(2.7절). 배리언트라면 언제 그 값이 되는지, 함수라면 `amount`의 단위(원)와 100원당 1포인트라는 규칙처럼 이름이 말하지 못하는 것을 적습니다.

### 4.4 그 밖의 실수

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| 시그니처의 타입을 문서에 다시 적음 | 리팩터링 뒤 시그니처와 문서가 어긋남. 아무 경고 없음 | 타입은 rustdoc에 맡기고 뜻만 적음 | 2.2, 01 |
| 사용 예를 ```` ```text ```` 블록에 둠 | 문서 테스트가 아니라 조용히 낡음(`running 0 tests`) | ```` ``` ```` 블록으로 바꿔 실행되게 함 | 2.5, `poor/` |
| 깨진 항목 링크 | `cargo build`·`cargo clippy`는 통과, `cargo doc`만 `unresolved link` 경고 | `cargo doc`을 돌리고 `-D warnings`나 `deny`로 실패시킴 | 2.6 |
| 공개 문서에서 비공개 항목으로 링크 | `links to private item` 경고, 렌더링 결과는 죽은 링크 | 공개 항목을 가리키거나 문장으로 설명 | 2.6, `poor/` |
| 비공개 모듈의 항목에 `#[doc(no_inline)]` | 링크 없는 `pub use` 한 줄만 남고 항목 페이지가 없음 | 속성을 빼서 루트에 인라인되게 함 | 2.6 |
| `#[doc(hidden)]`을 가시성 도구로 씀 | 문서에서만 사라지고 다른 크레이트에서 여전히 부를 수 있음 | `pub(crate)`로 닫음 | 2.6 |
| 빈 `# Errors` 제목, 본문 밖의 패닉 | `missing_errors_doc`·`missing_panics_doc`가 조용함 | 절의 내용은 리뷰로 확인 | 2.4 |
| 책의 이름 그대로 `#![deny(broken_intra_doc_links)]` | rustdoc의 `renamed_and_removed_lints` 경고 | `rustdoc::broken_intra_doc_links` | 2.8 |

## 5. 연습 문제

### 연습 1. 부실한 `parse` 문서 고치기

[`projects/doc_quality/poor/src/parse.rs`](projects/doc_quality/poor/src/parse.rs)의 `parse` 문서를 고치세요. 먼저 ```` ```text ````를 ```` ``` ````로만 바꿔 `cargo test --doc`을 실행하고, 어떤 에러가 나는지 보세요. 그다음 시그니처의 되풀이와 낡은 설명, 부르는 쪽 이야기를 지우고 `# Errors`와 실행되는 `# Examples`를 넣어 `cargo test --doc`이 1개 이상 통과하게 하세요.

<details>
<summary>힌트</summary>

블록만 바꾸면 ``cannot find value `text` in this scope``와 ``cannot find function `parse` in this scope``(둘 다 E0425)로 실패합니다. 문서 테스트는 크레이트 바깥 코드처럼 컴파일되므로 `kvconf::parse`로 부르고, 입력 문자열도 예제 안에서 만들어야 합니다. 그것을 고치면 이번에는 `value`가 없다는 E0599가 나옵니다. 이 에러들이 바로 `text` 블록이 숨기고 있던 낡음입니다. `# Errors`는 `improved/`처럼 배리언트마다 조건을 적고, 같은 키가 두 번 나오면 에러라는 것을 예제의 `assert_eq!`로 보이면 낡은 설명이 다시 생길 수 없습니다.

</details>

### 연습 2. 패닉하지 않는 대안 더하기

[`03_failure_sections.rs`](examples/03_failure_sections.rs)의 `v2::Document`에 `get(&self, n: usize) -> Option<&str>`을 더하세요. `page`의 `# Panics`와 `page_unchecked`의 `# Safety`가 이 메서드를 대안으로 가리키게 고친 뒤, `RUSTDOCFLAGS='-D warnings' cargo doc -p er-ch27-document-public-interfaces --example 03_failure_sections --no-deps`가 통과하는지 확인하세요. `get`에는 `# Panics`가 필요한가요?

<details>
<summary>힌트</summary>

본문은 `n.checked_sub(1).and_then(|i| self.pages.get(i)).map(String::as_str)`처럼 쓸 수 있습니다. `n == 0`일 때 `n - 1`이 넘치지 않도록 `checked_sub`를 씁니다. 링크는 ``[`get`](Self::get)``으로 겁니다. `get`은 어떤 `n`에도 패닉하지 않으므로 `# Panics`가 필요 없고, 문서에는 "`n`이 0이거나 쪽 수보다 크면 `None`"이라는 조건만 적으면 됩니다. 이것은 `Option`이 말하지 못하는 정보입니다. 표준 라이브러리의 `<[T]>::get_unchecked` 문서도 안전한 대안으로 `get`을 가리킵니다.

</details>

### 연습 3. 린트가 놓치는 문서

[`projects/doc_quality/improved`](projects/doc_quality/improved)의 사본을 만들어 두 가지를 바꾸세요. (1) `parse`의 `# Errors` 아래 목록을 지우고 제목만 남깁니다. (2) `Config::require`의 `# Panics` 절을 지우고, 본문의 `panic!`을 `-> !`를 돌려주는 비공개 함수 `missing(key)` 호출로 옮깁니다. `cargo clippy -- -D warnings`와 `cargo doc --no-deps`의 결과를 보고, 이 두 문서의 빈자리를 무엇으로 막을 수 있을지 생각해 보세요.

<details>
<summary>힌트</summary>

`lib.rs`가 `missing_errors_doc`와 `missing_panics_doc`를 켜 두었는데도 둘 다 통과합니다. `missing_errors_doc`는 `# Errors` 제목이 있는지만 보고, `missing_panics_doc`는 함수 본문에 보이는 `panic!`·`assert!`·`unwrap`·`expect`만 찾기 때문입니다. 패닉하는 함수를 부르기만 하면 찾지 못합니다. 막는 방법은 사람의 리뷰와, 문서의 약속을 실행하는 테스트입니다. 예를 들어 `require`의 패닉은 `# Panics`에 적고, 그 조건을 Part 0 ch11의 `#[should_panic]` 테스트로 확인할 수 있습니다.

</details>

### 연습 4. 공개 경로를 바꾸면 문서는

[`projects/doc_quality/poor/src/lib.rs`](projects/doc_quality/poor/src/lib.rs)의 `pub mod config;`를 `mod config;`로 바꾸고 `cargo doc --no-deps`를 실행한 뒤 `target/doc/kvconf/`의 파일과 첫 화면을 바꾸기 전과 비교하세요. 이어서 `pub use config::Config;` 위에 `#[doc(no_inline)]`을 붙이면 어떻게 되나요?

<details>
<summary>힌트</summary>

`config`가 비공개가 되면 `Config`는 루트에 인라인되어 `target/doc/kvconf/struct.Config.html`이 생기고, 첫 화면의 Re-exports 절에서 `Config` 줄이 빠지며 Structs 목록에 나옵니다. `config/struct.Config.html`은 루트로 넘겨 주는 페이지가 됩니다. 여기에 `#[doc(no_inline)]`을 붙이면 Re-exports 절에 링크 없는 `pub use config::Config;`만 남고 `Config`의 페이지는 어디에도 없습니다. 공개 모듈이 사라졌으니 연결할 곳이 없기 때문입니다. rustdoc 경고도 보세요. 모듈만 감췄을 때는 두 경고가 그대로 남지만, `no_inline`을 붙이면 `Config::value` 경고가 사라집니다. 링크가 고쳐져서가 아니라 `require`의 문서가 아예 만들어지지 않아서입니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 27: Document public interfaces](https://effective-rust.com/documentation.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- The rustdoc Book: [How to write documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)(첫 문단 요약, 항목 문서의 구성, 타입을 문서에 다시 적지 않기), [What to include (and exclude)](https://doc.rust-lang.org/rustdoc/write-documentation/what-to-include.html)(`missing_docs`, 예제의 에러 처리, `#[doc(hidden)]`), [Linking to items by name](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html), [Re-exports](https://doc.rust-lang.org/rustdoc/write-documentation/re-exports.html), [Lints](https://doc.rust-lang.org/rustdoc/lints.html). 로컬 문서(`rustc --print sysroot` 아래 `share/doc/rust/html/rustdoc/`)로 읽었습니다.
- Rust API Guidelines, [Documentation](https://rust-lang.github.io/api-guidelines/documentation.html): C-CRATE-DOC, C-EXAMPLE, C-QUESTION-MARK, C-FAILURE, C-LINK, C-HIDDEN
- 본보기로 읽은 표준 라이브러리 문서: [`Vec::remove`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.remove), [`Vec::split_off`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.split_off), [`String::from_utf8`](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8), [`<[T]>::get_unchecked`](https://doc.rust-lang.org/std/primitive.slice.html#method.get_unchecked)
- The Cargo Book, [The `readme` field](https://doc.rust-lang.org/cargo/reference/manifest.html#the-readme-field), [The `[lints]` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section)
- 로컬 문서의 릴리스 노트(`share/doc/rust/html/releases.md`): 1.52.0(rustdoc 린트의 `rustdoc::` 접두사), 1.74.0(`[lints]`)
