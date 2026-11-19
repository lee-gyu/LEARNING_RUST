# ch4. 관용적인 에러 타입 고르기

[Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)에서 에러 타입을 **만드는 방법**을 배웠습니다. `Error`를 직접 구현하고, 에러 열거형에 `From`을 붙이고, `Box<dyn Error>`에 담았다가 다운캐스트로 되찾았습니다. 이 챕터는 그다음 질문을 다룹니다. 같은 실패를 `String`, 뉴타입, 열거형, 트레이트 객체, `anyhow::Error` 중 **어떤 모양으로 알릴지**, 그리고 라이브러리와 애플리케이션에서 그 답이 왜 달라지는지입니다. 구현 문법은 다시 설명하지 않고 해당 챕터로 연결합니다.

- 원제: [Item 4: Prefer idiomatic Error types](https://effective-rust.com/errors.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch4-idiomatic-errors`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

예제 04·05는 챕터 `Cargo.toml`의 `[dev-dependencies]`에 넣은 `thiserror` 2.0.21과 `anyhow` 1.0.104를 씁니다. 처음 빌드할 때 crates.io에서 내려받습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `Error` 트레이트가 요구하는 것과, 에러 타입에 `Error`를 구현해야 하는 이유를 설명할 수 있다. `String` 에러를 뉴타입으로 바꾸고 `From<String>`으로 기존 코드와 이을 수 있다.
2. 호출한 쪽이 실패 종류마다 다르게 대응해야 할 때 중첩 에러 열거형을 설계할 수 있고, 메시지 문자열로 종류를 짐작하는 코드가 틀리는 이유를 설명할 수 있다.
3. `Box<dyn Error>`로 에러의 종류를 지웠을 때 얻는 것과 잃는 것을 설명하고, 대응할 실패만 배리언트로 남기는 절충형 에러 타입을 작성할 수 있다.
4. 같은 에러 열거형을 `thiserror`로 작성하고, 생성된 코드가 손으로 쓴 구현과 같은 `Display`·`source`·`From`을 만든다는 것을 확인할 수 있다.
5. 라이브러리와 애플리케이션에서 각각 어떤 에러 타입을 쓸지 근거를 들어 고르고, `anyhow::Error`가 `Box<dyn Error>`와 어떻게 다른지 설명할 수 있다.

## 2. 핵심 개념

### 2.1 지도: 에러 타입의 다섯 가지 모양

함수가 실패하는 방법이 한 가지뿐이면 고민할 것이 없습니다. `"42".parse::<u32>()`가 `ParseIntError`를 돌려주듯, 그 에러 타입을 그대로 돌려줍니다. 고민은 **서로 다른 종류의 실패가 한 함수에 모일 때** 시작됩니다. 그때 고를 수 있는 모양은 다음과 같습니다.

| 모양 | 호출한 쪽이 할 수 있는 것 | 만드는 비용 | 어울리는 곳 | 예제 |
| --- | --- | --- | --- | --- |
| `String` | 메시지 출력 | 없음 | 모듈 안의 도우미 함수 정도 | 01, 02 |
| 뉴타입 `struct E(String)` | 메시지 출력, `Error`가 필요한 곳 어디든 | 구현 세 개 | 종류를 구분할 필요가 없는 최소 에러 | 01 |
| 에러 열거형 | `match`, 필드 읽기, `source` 체인 | 배리언트마다 `Display`·`source`·`From` | 라이브러리의 공개 API | 02, 04 |
| `Box<dyn Error>` | 출력, `source` 체인, 다운캐스트 | 없음 | 종류를 보지 않는 위층 | 03 |
| `anyhow::Error` | 출력(체인 보고서), 다운캐스트 | 없음 | 애플리케이션의 위층 | 05 |

위에서 아래로 갈수록 만들기 쉬워지고, 호출한 쪽이 컴파일러의 도움을 받아 알 수 있는 것은 줄어듭니다. 이 챕터의 지침은 모두 "호출한 쪽이 에러를 받아서 **무엇을 하는가**"라는 한 가지 질문에서 나옵니다.

### 2.2 지침 1: 에러 타입에는 `Error`를 구현한다

**문제 상황.** 상품 코드가 `KR-102`처럼 "영문 대문자 2자 + `-` + 숫자 3자"인지 검사하는 함수가 있습니다. 실패하면 사용자에게 이유를 보여 주기만 하면 되므로 메시지 하나면 충분합니다.

**비권장 코드.** 메시지만 필요하니 `String`을 에러 타입으로 씁니다.

```rust
fn check_sku_str(code: &str) -> Result<(), String> { /* ... */ }
```

메시지 자체는 아무 문제가 없습니다([`01_string_to_newtype.rs`](examples/01_string_to_newtype.rs)의 2번 구역에서 두 판의 메시지는 똑같습니다). 문제는 이 에러가 다른 에러 처리 코드와 섞일 때 나타납니다. `String`은 `Error`를 구현하지 않기 때문입니다.

- **원인으로 품을 수 없습니다.** 위층 에러의 `source()`는 `&(dyn Error + 'static)`를 돌려줘야 하는데 `&String`은 그렇게 바뀌지 않습니다(4.2절). `String`을 그대로 품으면 위층 메시지에 아래층 메시지를 이어 붙이는 수밖에 없고, 그러면 `source` 체인이 끊깁니다.
- **`Box<dyn Error>`에 들어간 뒤 정체를 물을 수 없습니다.** `?`가 `String`을 `Box<dyn Error>`로 바꿀 때 표준 라이브러리의 비공개 타입에 담기 때문에, 받은 쪽은 이 에러가 어디서 왔는지 물어볼 타입이 없습니다. `e.is::<String>()`은 `String`이 `Error`가 아니라서 컴파일되지 않습니다.
- **`E: Error`를 요구하는 코드에 넘길 수 없습니다.** 에러를 기록하는 제네릭 함수, `thiserror`의 `#[source]` 필드처럼 `Error`를 전제로 한 도구를 쓸 수 없습니다.

그럼 `String`에 `Error`를 구현하면 될까요? `Error`도 `String`도 내 크레이트의 것이 아니므로 고아 규칙(orphan rule, [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.7절)에 걸립니다. `type SkuError = String;`처럼 별칭을 붙여도 새 타입이 생기지 않으므로 결과는 같습니다(4.1절).

**권장 코드.** `String`을 내 타입으로 한 번 감싼 뉴타입을 만들고 `Debug`, `Display`, `Error`를 구현합니다. 뉴타입 패턴 자체는 [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.12절에서 배웠고, 이 부록에서는 `ch6-newtype-pattern`에서 다시 다룹니다.

```rust
#[derive(Debug)]
struct SkuError(String);

impl fmt::Display for SkuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for SkuError {} // 필수 메서드가 없으므로 빈 블록

impl From<String> for SkuError {
    fn from(message: String) -> Self {
        SkuError(message)
    }
}

fn check_sku(code: &str) -> Result<(), SkuError> {
    check_format(code)?; // String을 돌려주던 기존 함수: ?가 From<String>으로 SkuError로 바꿈
    Ok(())
}
```

`Error`가 요구하는 것은 슈퍼트레이트인 `Debug`와 `Display`뿐입니다. deprecated 메서드와 nightly 전용 메서드를 빼면 남는 메서드는 `source()` 하나이고, 그것도 기본 구현(`None`)이 있습니다([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 2.2절). 요구가 이렇게 적으니 구현하지 않을 이유가 없습니다. `From<String>`을 함께 구현해 두면 `String`을 돌려주던 기존 함수에도 `?`를 쓸 수 있어 한꺼번에 고치지 않아도 됩니다. 뉴타입은 감싸기만 하므로 비용도 없습니다. 예제 01의 1번 구역에서 `size_of::<String>()`과 `size_of::<SkuError>()`는 모두 24입니다.

같은 상황에서 두 판의 차이는 예제 01의 3·4번 구역에서 보입니다(출력은 3절). 뉴타입 판에서만 위층 에러와 원인이 두 층으로 나뉘고, `Box<dyn Error>`에 담긴 뒤에도 `is::<SkuError>()`로 정체를 확인할 수 있습니다.

**따르지 않아도 되는 경우.** 모듈 안에서만 쓰고 공개 경계에서 곧바로 뉴타입으로 바꾸는 도우미 함수라면 `String`을 돌려줘도 됩니다. 예제 01의 `check_format`이 그런 함수입니다. 애플리케이션에서 일회성 메시지 에러가 필요하다면 `String` 대신 2.6절의 `anyhow!`나 `bail!`을 씁니다.

### 2.3 지침 2: 호출한 쪽이 구분해야 하면 중첩 에러 열거형을 만든다

**문제 상황.** 네트워크로 받은 주문 줄 `b"KR-102:3"`을 바이트 그대로 읽습니다. 실패하는 방법은 네 가지입니다. UTF-8이 아님(`Utf8Error`), `:`가 없음, 수량이 숫자가 아님(`ParseIntError`), 수량이 최대 99를 넘음. 이 파서를 쓰는 쪽에는 "성공하면 접수하고, 수량만 넘친 줄은 담당자가 확인하도록 보류하고, 나머지는 버린다"는 방침이 있습니다. 즉 **호출한 쪽이 실패 종류를 구분해야 합니다.**

**비권장 코드.** 실패를 모두 `String` 메시지로 알리면, 호출한 쪽은 메시지에 든 낱말로 종류를 짐작할 수밖에 없습니다([`02_nested_enum.rs`](examples/02_nested_enum.rs)).

```rust
match parse_line_str(raw) {
    Ok(line) => format!("접수 {} x{}", line.sku, line.qty),
    Err(msg) if msg.contains("최대") => "보류".to_string(), // 수량 초과 메시지에 "최대"가 있으니까
    Err(_) => "버림".to_string(),
}
```

주문 목록에 메모 줄 `최대한 빨리 보내 주세요`가 섞여 들어오면 이 코드는 틀린 결정을 내립니다. 컴파일 에러도 경고도 없습니다.

```text
    최대한 빨리 보내 주세요
      String 판: 보류
      열거형 판: 버림
```

`:`가 없다는 메시지에 사람이 읽기 좋도록 입력을 넣어 두었는데(`':' 구분자가 없습니다: "최대한 빨리 보내 주세요"`), 그 입력에 "최대"가 들어 있었기 때문입니다. 메시지는 사람을 위한 것이라 입력이 섞이고, 라이브러리가 판을 올리며 문구를 다듬어도 아무도 호환성 문제로 여기지 않습니다. 메시지를 프로그램이 해석하는 순간 그 모든 변화가 버그가 됩니다.

**권장 코드.** 실패 종류마다 배리언트를 두고, 아래 계층의 에러(`Utf8Error`, `ParseIntError`)는 배리언트 안에 품습니다. 에러 안에 에러가 들어 있으므로 중첩 에러(nested error)라고 부릅니다.

```rust
#[derive(Debug)]
enum OrderError {
    Encoding(Utf8Error),                  // 아래 계층: 바이트 → 문자열
    MissingColon { line: String },        // 이 계층이 판단한 실패
    BadQty(ParseIntError),                // 아래 계층: 문자열 → 숫자
    TooMany { requested: u32, max: u32 }, // 대응에 필요한 값을 담음
}

// Display는 이 계층의 사정만, source()는 Encoding과 BadQty의 원인을 내줌,
// From<Utf8Error>와 From<ParseIntError>로 본문에서는 ?만 씀 (예제 02 참고)

match parse_line(raw) {
    Ok(line) => format!("접수 {} x{}", line.sku, line.qty),
    Err(OrderError::TooMany { requested, max }) => format!("보류 ({requested} > {max})"),
    Err(OrderError::Encoding(_) | OrderError::MissingColon { .. } | OrderError::BadQty(_)) => {
        "버림".to_string()
    }
}
```

`Display`, `source`, `From`을 쓰는 방법과 각 원칙(이 계층의 실패만 `Display`에, 원인은 `source()`로)은 Part 1 ch8 2.3·2.4절에서 다뤘으므로 여기서는 이 모양을 고르는 이유만 정리합니다.

- **종류는 타입이 말합니다.** 호출한 쪽은 문자열이 아니라 배리언트로 나눕니다. 메시지 문구를 바꿔도 `match`는 영향을 받지 않습니다.
- **대응에 필요한 값이 필드로 옵니다.** `TooMany`의 `requested`와 `max`를 메시지에서 다시 파싱할 필요가 없습니다.
- **아래 계층의 정보가 남습니다.** `BadQty`가 품은 `ParseIntError`는 `source()`로 꺼낼 수 있고, `kind()`로 더 자세히 물어볼 수도 있습니다.
- **배리언트가 늘면 컴파일러가 알려 줍니다.** 위 `match`처럼 `_`로 뭉뚱그리지 않고 배리언트를 모두 적어 두면, 라이브러리에 배리언트가 추가될 때 처리하지 않은 곳이 E0004로 드러납니다.
- **`From`은 이 크레이트만 쓸 수 있습니다.** `impl From<Utf8Error> for OrderError`는 `OrderError`를 정의한 크레이트에서만 쓸 수 있습니다(고아 규칙). 라이브러리가 변환을 제공하지 않으면 사용자가 대신 만들어 줄 방법이 없으므로, 라이브러리 쪽에서 미리 구현해 둡니다.

**따르지 않아도 되는 경우.** 호출한 쪽이 실패 종류를 보지 않고 보고만 한다면 열거형은 쓸모없는 비용입니다. 그때는 2.2절의 뉴타입이나 2.4절의 트레이트 객체로 충분합니다. 원인 에러를 품을지 버릴지(감싸기와 대신하기)는 Part 1 ch8 2.4절의 기준을 따릅니다.

### 2.4 지침 3: 종류를 지우는 트레이트 객체는 호출한 쪽이 종류를 보지 않을 때 쓴다

**문제 상황.** 아래 계층에서 올 수 있는 에러의 종류가 많거나 미리 알 수 없고, 호출한 쪽은 대부분 보고만 합니다. 배리언트를 하나하나 만드는 대신 `Box<dyn Error>`에 담으면 됩니다.

**얻는 것.** 타입을 하나도 정의하지 않고 `?`만으로 여러 종류의 에러를 담을 수 있습니다. 담긴 에러의 `Display`와 `source` 체인은 그대로 남습니다. [`03_trait_object.rs`](examples/03_trait_object.rs)의 1번 구역은 2.3절과 같은 파서를 에러 타입 정의 없이 씁니다.

```rust
fn parse_line_boxed(raw: &[u8]) -> Result<(String, u32), Box<dyn Error>> {
    let text = str::from_utf8(raw)?;                                       // Utf8Error
    let (sku, qty) = text.split_once(':').ok_or("':' 구분자가 없습니다")?; // &str
    let qty: u32 = qty.trim().parse()?;                                    // ParseIntError
    // ...
}
```

**잃는 것.** 호출한 쪽이 종류를 알려면 다운캐스트로 하나씩 물어봐야 합니다(Part 1 ch8 2.5절). 이것은 생각보다 깨지기 쉽습니다. 예제 03의 2번 구역에서 라이브러리의 v1은 `ParseIntError`를 그대로 상자에 담았고, v2는 메시지에 입력을 넣으려고 자기 에러 `QtyError`로 한 겹 감쌌습니다. 시그니처는 둘 다 `Result<u32, Box<dyn Error>>`로 같습니다.

```text
    "x7"
      v1 바깥만 다운캐스트: 형식 오류 → 0개로 처리
      v2 바깥만 다운캐스트: 모르는 에러 → 작업 중단 (수량 "x7"을(를) 읽지 못했습니다)
      v2 체인에서 찾기:     형식 오류(InvalidDigit) → 0개로 처리
```

라이브러리 입장에서는 좋은 개선이었는데, 호출한 쪽의 `e.is::<ParseIntError>()`가 컴파일 에러 없이 `false`가 되어 처리 방침이 바뀌었습니다. 열거형이었다면 같은 변경은 공개된 배리언트나 필드의 타입을 바꾸는 일이라 시그니처에 드러나고, 그 필드를 쓰던 호출한 쪽 코드는 컴파일 단계에서 멈춥니다. `source` 체인을 따라 내려가며 찾는 `find_cause`로 바꾸면 감싸는 층이 늘어도 버티지만, 그것도 실행해 봐야 아는 약속입니다.

그 밖에 잃는 것이 더 있습니다.

- **되돌아갈 수 없습니다.** 한 번 `Box<dyn Error>`가 된 에러는 `?`로 다시 구체 타입이 되지 않습니다(4.3절). 종류를 지우는 쪽은 일방통행입니다.
- **`Box<dyn Error>` 자신은 `Error`가 아닙니다.** `E: Error`를 요구하는 제네릭 함수에 넘길 수 없습니다(Part 1 ch8 4.5절). 스레드로 보내려면 `+ Send + Sync`가 필요합니다(Part 1 ch8 4.4절).

**감싸서 해결하려 하면 생기는 매듭.** `Box<dyn Error>`를 내 타입으로 감싸 `Error`를 구현하고, "`Error`를 구현한 모든 `E`"에서 오는 포괄 `From`까지 붙이면 편할 것 같습니다. 하지만 그 타입 자신도 `Error`이므로 `E`가 자기 자신일 때 표준 라이브러리의 `impl<T> From<T> for T`와 겹쳐 E0119가 납니다. Part 1 ch8 4.2절의 그 충돌입니다. 내 타입에 `Error`를 구현하는 것과 모든 에러에서 `?`로 바뀌는 것, 둘 중 하나는 포기해야 합니다. 2.6절의 `anyhow::Error`는 `Error`를 구현하지 않는 쪽을 골랐습니다.

**권장 코드.** 호출한 쪽이 대응할 실패만 배리언트로 남기고, 나머지는 `Box<dyn Error + Send + Sync>`를 담는 배리언트 하나로 모읍니다. 예제 03의 3번 구역입니다.

```rust
#[derive(Debug)]
enum StockError {
    OutOfStock { sku: String, have: u32, want: u32 }, // 대응하는 실패
    Storage(Box<dyn Error + Send + Sync>),            // 보고만 하는 실패
}

impl Error for StockError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OutOfStock { .. } => None,
            Self::Storage(e) => Some(e.as_ref()), // 상자 안의 에러를 빌려 체인을 잇는다
        }
    }
}

// 포괄 From은 막히지만 Box 한 타입에서 오는 From은 문제없음 → read_stock(..)?가 됨
impl From<Box<dyn Error + Send + Sync>> for StockError { /* Self::Storage(e) */ }
```

```text
    재고 기록 "3", 5개 예약 → 입고 요청: KR-102 2개
    재고 기록 "1a", 5개 예약 → 보고: 재고 기록을 읽지 못했습니다
        원인: invalid digit found in string
```

`OutOfStock`은 `match`로 대응하고, 저장소에서 온 에러는 종류를 몰라도 체인을 따라 보고할 수 있습니다.

**따르지 않아도 되는 경우.** 이 지침은 "언제 써도 되는가"에 관한 것입니다. 받은 에러를 보고하고 끝내는 애플리케이션의 위층이라면 종류를 지워도 잃을 것이 없으므로 마음껏 씁니다. 다만 그 자리에는 2.6절의 `anyhow::Error`가 더 편합니다.

### 2.5 지침 4: 라이브러리의 에러 열거형은 `thiserror`로 반복을 줄인다

**문제 상황.** 2.3절의 `OrderError`를 손으로 쓰면 `Display`의 `match`, `source()`의 `match`, 원인 타입마다 `From`이 필요합니다. 예제 04의 손으로 쓴 판은 열거형 정의부터 마지막 `From`까지 42줄이고, 배리언트를 하나 추가할 때마다 세 곳을 고쳐야 합니다. 같은 일을 하는 `thiserror` 판은 11줄입니다.

**비권장 코드.** 예제 04의 `by_hand` 모듈처럼 손으로 쓴 구현이 틀린 것은 아닙니다. 다만 반복이 많아 실수하기 쉽습니다. 예를 들어 `From`은 추가하고 `source()`의 갈래는 `None`으로 남겨 두면 컴파일은 되지만 체인이 끊깁니다.

**권장 코드.** `thiserror`의 derive 매크로(절차적 매크로, [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.20절)에 맡깁니다([`04_thiserror.rs`](examples/04_thiserror.rs)).

```rust
#[derive(Debug, thiserror::Error)]
pub enum OrderError {
    #[error("주문 줄이 UTF-8이 아닙니다")]
    Encoding(#[from] Utf8Error),
    #[error("':' 구분자가 없습니다: {line:?}")]
    MissingColon { line: String },
    #[error("수량을 읽지 못했습니다")]
    BadQty(#[from] ParseIntError),
    #[error("수량 {requested}이(가) 최대 {max}를 넘습니다")]
    TooMany { requested: u32, max: u32 },
}
```

| 속성 | 만들어 주는 것 |
| --- | --- |
| `#[error("…")]` | 그 배리언트(또는 구조체)의 `Display`. 필드는 `{line}`, `{0}`처럼 이름이나 번호로, 서식 지정도 `{line:?}`처럼 씁니다. |
| `#[from]` | 그 필드 타입에서 오는 `From` 구현. 그 필드가 `source()`의 반환값도 됩니다. |
| `#[source]` 또는 이름이 `source`인 필드 | `source()`가 그 필드를 돌려줍니다. `From`은 만들지 않습니다. |
| `#[error(transparent)]` | `Display`와 `source()`를 감싼 에러에 그대로 넘깁니다. |

예제 04는 손으로 쓴 판과 `thiserror` 판을 나란히 두고 같은 입력을 넣습니다. 네 가지 에러 모두에서 `Display`, `source`의 `Display`, `Debug`가 글자 하나 다르지 않고, `#[from]`이 만든 `From` 덕분에 두 판의 `parse_line` 본문도 똑같습니다.

```text
    "KR-310:x": Display 같음 = true, source 같음 = true, Debug 같음 = true
        Display: 수량을 읽지 못했습니다
        source:  invalid digit found in string
        Debug:   BadQty(ParseIntError { kind: InvalidDigit })
```

중요한 성질이 하나 있습니다. `thiserror`가 만드는 것은 표준 트레이트(`Display`, `Error`, `From`)의 구현뿐이라서, **공개 API에 `thiserror`의 타입이 드러나지 않습니다.** 라이브러리 사용자는 `thiserror`를 의존성에 넣을 필요가 없고, 나중에 손으로 쓴 구현으로 바꿔도 사용자 코드는 깨지지 않습니다. `thiserror` 문서도 이 점을 명시합니다.

구조체에도 쓸 수 있습니다. 예제 04의 `LineError`는 줄 번호처럼 원인 에러만으로는 채울 수 없는 필드가 있어서 `#[from]` 대신 `#[source]`를 붙였고, 만들 때는 `map_err`로 줄 번호를 넣습니다.

**따르지 않아도 되는 경우.** 배리언트가 한두 개뿐이거나 의존성을 늘리고 싶지 않은 작은 라이브러리라면 손으로 써도 충분합니다. `thiserror`는 절차적 매크로라서 빌드할 때 `thiserror-impl`, `syn`, `quote`, `proc-macro2`가 함께 컴파일됩니다(`cargo tree`로 확인). 이미 다른 의존성이 이 크레이트들을 쓰고 있다면 추가 비용은 거의 없습니다.

### 2.6 지침 5: 라이브러리는 구체 타입을, 애플리케이션의 위층은 `anyhow`를 쓴다

**문제 상황.** 애플리케이션은 여러 라이브러리의 에러를 한곳에 모아 사용자에게 보여 줍니다. 라이브러리는 자신이 어떤 프로그램에서 어떤 방침으로 쓰일지 모릅니다. 두 쪽의 필요가 다릅니다.

**비권장 코드.** 라이브러리가 `anyhow::Result`를 공개합니다. 라이브러리를 쓰기는 편하지만, 호출한 쪽에서 보면 `match`할 배리언트가 없습니다. [`05_library_vs_app.rs`](examples/05_library_vs_app.rs)의 1번 구역에서, 2.3절과 같은 "수량 초과는 보류" 방침을 지키려 해도 `bail!`로 만든 에러는 타입이 없어 메시지를 읽는 것 말고는 방법이 없습니다.

```text
    KR-205:120
      anyhow를 공개한 라이브러리:   타입 없음, 메시지뿐 → 수량 120이(가) 최대 99를 넘습니다
      구체 타입을 공개한 라이브러리: 보류 (120 > 99)
```

게다가 `anyhow::Error`는 `Error`를 구현하지 않으므로 `E: Error`를 요구하는 제네릭 함수에 넘길 수 없고(E0277 ``the trait bound `anyhow::Error: std::error::Error` is not satisfied``, 이 툴체인에서 확인), 라이브러리의 공개 API가 `anyhow`의 판에 묶입니다.

**권장 코드.** 라이브러리는 2.3·2.5절의 구체 타입을 공개하고, 애플리케이션은 위층에서 `anyhow::Result<T>`(`Result<T, anyhow::Error>`의 별칭)로 모읍니다.

```rust
fn plan_shipment(lines: &[&[u8]], daily_limit: u32) -> anyhow::Result<u32> {
    let total = orders::total_qty(lines)?; // 라이브러리의 SheetError → anyhow::Error
    if total > daily_limit {
        // 애플리케이션에서만 쓰는 실패는 타입 없이 메시지로 충분
        anyhow::bail!("총 수량 {total}개가 하루 처리량 {daily_limit}개를 넘습니다");
    }
    Ok(total.div_ceil(10))
}
```

`?`는 `Error + Send + Sync + 'static`을 만족하는 에러라면 무엇이든 `anyhow::Error`로 바꿉니다. `bail!(...)`은 `return Err(anyhow!(...))`의 줄임이고, `anyhow!(...)`는 서식 문자열로 타입 없는 에러를 만듭니다. 애플리케이션에서 `String` 에러가 필요했던 자리를 이것이 대신합니다. 보고는 서식 지정 하나로 고릅니다.

```text
    bad,  한도 100 → {}  3번째 줄을 처리하지 못했습니다
    bad,  한도 100 → {:#} 3번째 줄을 처리하지 못했습니다: 수량을 읽지 못했습니다: invalid digit found in string
    bad,  한도 100 → {:?} 아래 여러 줄
        | 3번째 줄을 처리하지 못했습니다
        |
        | Caused by:
        |     0: 수량을 읽지 못했습니다
        |     1: invalid digit found in string
```

`{}`는 맨 바깥 에러만, `{:#}`는 체인 전체를 `: `로 이어 한 줄로, `{:?}`는 `Caused by:` 아래에 원인을 나열합니다(원인이 하나뿐이면 번호 없이 한 줄입니다). `main`이 `anyhow::Result<()>`를 돌려주고 `Err`로 끝나면 Rust가 `Error: ` 뒤에 이 `{:?}` 형식을 출력합니다([Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.10절). Part 1 ch8 2.7절에서 손으로 쓴 `report` 함수가 하던 일을 `anyhow`가 맡는 셈입니다. 애플리케이션도 특정 실패에 대응해야 할 때는 `e.downcast_ref::<orders::SheetError>()`로 라이브러리의 구체 타입을 되찾고, `chain()`과 `root_cause()`로 체인을 훑습니다(예제 05의 3번 구역).

**`anyhow::Error`는 `Box<dyn Error>`와 무엇이 다른가.**

- **포인터 하나 크기입니다.** `size_of::<anyhow::Error>()`는 8, `size_of::<Box<dyn Error + Send + Sync>>()`는 16입니다(64비트 플랫폼). 트레이트 객체의 vtable 포인터를 힙에 둔 값 쪽으로 옮겨, 한 단계 간접 참조를 더하는 대신 `Result<(), anyhow::Error>`도 8바이트가 됩니다. 성공하는 경로에서 오가는 값이 작아집니다.
- **`Error`를 구현하지 않습니다.** 2.4절의 매듭 때문입니다. `impl<E> From<E> for anyhow::Error`(`E: Error + Send + Sync + 'static`)가 있으므로 `Error`까지 구현하면 `From<T> for T`와 겹칩니다. 대신 `Deref`와 `AsRef`로 `&(dyn Error + Send + Sync)`를 빌려줍니다(`e.as_ref()`).
- **백트레이스를 담습니다.** 만들어질 때 `std::backtrace::Backtrace::capture()`를 부르므로 `RUST_LIB_BACKTRACE`나 `RUST_BACKTRACE`가 켜져 있으면 `{:?}` 출력 끝에 `Stack backtrace:`가 붙습니다. 환경 변수의 규칙은 Part 1 ch8 2.10절과 같습니다.
- **`Send + Sync + 'static`만 담습니다.** 그래서 스레드 사이에서 바로 주고받을 수 있습니다.

| | 라이브러리 | 애플리케이션 |
| --- | --- | --- |
| 에러를 받는 쪽 | 모르는 누군가. 방침을 정할 수 없음 | 자기 자신. 대부분 보고하고 끝남 |
| 필요한 것 | 종류별 대응, 안정된 공개 API | 모든 의존성의 에러를 한 타입으로, 읽기 좋은 보고서 |
| 고를 것 | 에러 열거형(손으로 또는 `thiserror`) | `anyhow::Result` + 필요할 때 `downcast_ref` |

**따르지 않아도 되는 경우.** 애플리케이션 안에서도 특정 실패에 대응하는 내부 모듈은 라이브러리처럼 구체 타입을 씁니다. 애플리케이션은 "라이브러리 여러 개와 그것을 묶는 위층"으로 보면 됩니다. `anyhow`의 문맥 붙이기(`context`)와 종료 코드 정하기 등 명령줄 프로그램에서의 사용법은 Part 7 `ch2-implementation-errors`에서, 패닉과 `Result` 중 무엇을 고를지는 `ch18-dont-panic`에서 다룹니다.

### 2.7 책과 달라진 점

- **`Error`는 `core`에도 있습니다.** 책은 `Error`가 `std`에만 있어 `no_std` 환경에서 쓸 수 없고, `core`로 옮기는 작업은 아직 안정화되지 않았다고 설명합니다. 현재는 `core::error::Error`가 1.81.0에서 안정화되었고(로컬 문서의 안정화 버전 표시로 확인), `std::error::Error`는 그것을 다시 내보낸 것입니다. 예제 01의 1번 구역에서 `type_name::<dyn std::error::Error>()`가 `dyn core::error::Error`를 출력합니다. `#![no_std]` 라이브러리에서 `thiserror`를 `default-features = false`로 넣고 `derive(thiserror::Error)`를 쓴 코드가 이 툴체인에서 빌드되는 것도 확인했습니다. `no_std`에서의 에러는 `ch33-no-std`에서 다룹니다.
- **object safety는 이제 "dyn 호환"입니다.** 책이 트레이트 객체의 제약을 부르는 이름이 바뀌었습니다. 컴파일러 메시지도 `the trait ... is not dyn compatible`이라고 씁니다([Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 2.9절).
- **크레이트 판.** 책은 `thiserror`와 `anyhow`의 판을 적지 않았습니다. 이 챕터는 `thiserror` 2.0.21과 `anyhow` 1.0.104로 확인했습니다. `thiserror` 2는 기본으로 켜진 `std` 기능을 끄면 `no_std`에서도 쓸 수 있습니다(위 항목). `anyhow`는 별도 기능 없이 stable 툴체인에서 표준 라이브러리의 백트레이스를 담습니다(2.6절).

### 2.8 기억할 것

- 에러 타입에는 `Error`를 구현합니다. 요구하는 것은 `Debug`와 `Display`뿐입니다. `String`에는 구현할 수 없으므로 뉴타입으로 감쌉니다.
- 호출한 쪽이 종류마다 다르게 대응해야 하면 에러 열거형을 만듭니다. 메시지 문자열로 종류를 짐작하는 코드는 컴파일러가 지켜 주지 않습니다.
- `Box<dyn Error>`는 종류를 지우는 일방통행입니다. 대응할 실패는 배리언트로 남기고 나머지만 상자에 담습니다.
- 라이브러리는 구체 타입을 공개하고(`thiserror`가 반복을 줄여 줍니다), 애플리케이션의 위층은 `anyhow::Result`로 모아 보고합니다.
- 어느 쪽을 고르든 그 결정을 타입에 적어 둡니다. 시그니처가 "이 함수는 어떻게 실패하는가"를 말하게 합니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_string_to_newtype.rs`](examples/01_string_to_newtype.rs) | `String` 에러와 뉴타입 비교: `source` 체인, `Box<dyn Error>` 안에서의 정체, `From<String>` | 1 |
| [`examples/02_nested_enum.rs`](examples/02_nested_enum.rs) | 메시지로 종류를 짐작하는 비권장 코드가 틀리는 입력, 중첩 에러 열거형과 `From` | 2 |
| [`examples/03_trait_object.rs`](examples/03_trait_object.rs) | `Box<dyn Error>`의 득실, 한 겹 감싸면 빗나가는 다운캐스트, 절충형 `StockError` | 3 |
| [`examples/04_thiserror.rs`](examples/04_thiserror.rs) | 손으로 쓴 판과 `thiserror` 판의 `Display`·`source`·`Debug`·`From` 비교, `#[source]` | 4 |
| [`examples/05_library_vs_app.rs`](examples/05_library_vs_app.rs) | `anyhow`를 공개한 라이브러리와 구체 타입을 공개한 라이브러리, 애플리케이션의 보고서, `anyhow::Error`의 성질 | 5 |
| [`compile_fail/`](compile_fail) | `String` 별칭과 고아 규칙, `String`을 `source`로, `Box<dyn Error>`에서 구체 타입으로 (4절) | 1, 3 |

모든 명령은 저장소 루트에서 실행하며, 모든 예제는 종료 코드 0으로 끝납니다. 입력에 일부러 넣은 `0xFF` 바이트는 `String::from_utf8_lossy`로 보여 주므로 출력에 대체 문자 `�`로 나옵니다.

### 01_string_to_newtype — `String` 에러와 뉴타입

```sh
cargo run -p er-ch4-idiomatic-errors --example 01_string_to_newtype
```

```text
1. Error 트레이트의 정체와 뉴타입의 크기
    type_name::<dyn std::error::Error>() = dyn core::error::Error
    size_of::<String>() = 24, size_of::<SkuError>() = 24
2. 메시지는 똑같습니다
    KR-102  String: 통과
            뉴타입: 통과
    KR102   String: 상품 코드에 '-'가 없습니다: "KR102"
            뉴타입: 상품 코드에 '-'가 없습니다: "KR102"
    kr-102  String: 상품 코드의 앞부분은 영문 대문자 2자여야 합니다: "kr-102"
            뉴타입: 상품 코드의 앞부분은 영문 대문자 2자여야 합니다: "kr-102"
    KR-1O2  String: 상품 코드의 뒷부분은 숫자 3자여야 합니다: "KR-1O2"
            뉴타입: 상품 코드의 뒷부분은 숫자 3자여야 합니다: "KR-1O2"
3. 위층 에러에 원인으로 넣기
    [String]
      에러: 2번째 줄을 처리하지 못했습니다: 상품 코드의 뒷부분은 숫자 3자여야 합니다: "KR-1O2"
      체인의 맨 아래가 SkuError인가: false
    [뉴타입]
      에러: 2번째 줄을 처리하지 못했습니다
        원인: 상품 코드의 뒷부분은 숫자 3자여야 합니다: "KR-1O2"
      체인의 맨 아래가 SkuError인가: true
4. Box<dyn Error>에 담긴 뒤 어디서 온 에러인지 묻기
    [String] {:?} = "상품 코드에 '-'가 없습니다: \"KR102\""
             is::<SkuError>() = false
    [뉴타입] {:?} = SkuError("상품 코드에 '-'가 없습니다: \"KR102\"")
             is::<SkuError>() = true
```

`KR-1O2`의 가운데 글자는 숫자 0이 아니라 영문 O입니다. 3번 구역의 `String` 판은 `source`가 없어 체인의 맨 아래가 위층 에러 자신입니다. 4번 구역의 `String` 판 디버그 형식은 따옴표로 감싼 문자열이라, 상자 안에 든 것이 어느 모듈의 에러인지 알 수 없습니다.

### 02_nested_enum — 중첩 에러 열거형

```sh
cargo run -p er-ch4-idiomatic-errors --example 02_nested_enum
```

```text
1. 같은 처리 방침, 두 가지 에러 타입
    KR-102:3
      String 판: 접수 KR-102 x3
      열거형 판: 접수 KR-102 x3
    KR-205:120
      String 판: 보류
      열거형 판: 보류 (120 > 99)
    KR-310:x
      String 판: 버림
      열거형 판: 버림
    최대한 빨리 보내 주세요
      String 판: 보류
      열거형 판: 버림
    KR-4�2:1
      String 판: 버림
      열거형 판: 버림
2. 메시지만 보면 무엇이 문제였나
    String 판: 수량 120이(가) 최대 99를 넘습니다
    String 판: 수량을 읽지 못했습니다: invalid digit found in string
    String 판: ':' 구분자가 없습니다: "최대한 빨리 보내 주세요"
    String 판: 주문 줄이 UTF-8이 아닙니다: invalid utf-8 sequence of 1 bytes from index 4
3. 열거형은 아래 계층의 에러를 source()로 내줍니다
    TooMany { requested: 120, max: 99 }
      Display: 수량 120이(가) 최대 99를 넘습니다
      원인:    없음 (이 계층이 판단한 실패)
    BadQty(ParseIntError { kind: InvalidDigit })
      Display: 수량을 읽지 못했습니다
      원인:    invalid digit found in string
    MissingColon { line: "최대한 빨리 보내 주세요" }
      Display: ':' 구분자가 없습니다: "최대한 빨리 보내 주세요"
      원인:    없음 (이 계층이 판단한 실패)
    Encoding(Utf8Error { valid_up_to: 4, error_len: Some(1) })
      Display: 주문 줄이 UTF-8이 아닙니다
      원인:    invalid utf-8 sequence of 1 bytes from index 4
```

1번 구역에서 두 판의 결정은 `최대한 빨리 보내 주세요` 줄에서만 갈립니다. 2번 구역의 세 번째 메시지에 "최대"가 들어 있는 것이 이유입니다. `String` 판은 아래 계층의 메시지를 `: ` 뒤에 이어 붙였고, 열거형 판은 같은 정보를 `Display`와 `source()`로 나눠 내줍니다.

### 03_trait_object — `Box<dyn Error>`의 득실

```sh
cargo run -p er-ch4-idiomatic-errors --example 03_trait_object
```

```text
1. Box<dyn Error>는 정의 없이 무엇이든 담습니다
    KR-102:3    → KR-102 x3
    KR-4�2:1    → [Utf8Error] invalid utf-8 sequence of 1 bytes from index 4
    KR-310:x    → [ParseIntError] invalid digit found in string
    KR-205:120  → [알 수 없음 (문자열에서 만든 에러)] 수량 120이(가) 최대 99를 넘습니다
2. 라이브러리가 에러를 한 겹 감싸면
    "7"
      v1 바깥만 다운캐스트: 7개
      v2 바깥만 다운캐스트: 7개
      v2 체인에서 찾기:     7개
    "x7"
      v1 바깥만 다운캐스트: 형식 오류 → 0개로 처리
      v2 바깥만 다운캐스트: 모르는 에러 → 작업 중단 (수량 "x7"을(를) 읽지 못했습니다)
      v2 체인에서 찾기:     형식 오류(InvalidDigit) → 0개로 처리
3. 대응할 실패만 배리언트로, 나머지는 Box로
    재고 기록 "12", 5개 예약 → 완료 (남은 재고 7개)
    재고 기록 "3", 5개 예약 → 입고 요청: KR-102 2개
    재고 기록 "1a", 5개 예약 → 보고: 재고 기록을 읽지 못했습니다
        원인: invalid digit found in string
    재고 기록 "�", 5개 예약 → 보고: 재고 기록을 읽지 못했습니다
        원인: invalid utf-8 sequence of 1 bytes from index 0
```

1번 구역의 대괄호 안은 `is::<T>()`로 확인한 안쪽 타입입니다. 문자열에서 만든 에러는 물어볼 타입이 없어 "알 수 없음"이 됩니다. 2번 구역의 `find_cause`는 `source()`를 따라 내려가며 각 층에 `downcast_ref`를 시도하는 함수입니다.

### 04_thiserror — 손으로 쓴 판과 `thiserror` 판

```sh
cargo run -p er-ch4-idiomatic-errors --example 04_thiserror
```

```text
1. 손으로 쓴 판과 thiserror 판 비교
    "KR-102:3": 둘 다 Ok, 결과 같음 = true
    "KR-4�2:1": Display 같음 = true, source 같음 = true, Debug 같음 = true
        Display: 주문 줄이 UTF-8이 아닙니다
        source:  invalid utf-8 sequence of 1 bytes from index 4
        Debug:   Encoding(Utf8Error { valid_up_to: 4, error_len: Some(1) })
    "KR-102": Display 같음 = true, source 같음 = true, Debug 같음 = true
        Display: ':' 구분자가 없습니다: "KR-102"
        source:  없음
        Debug:   MissingColon { line: "KR-102" }
    "KR-310:x": Display 같음 = true, source 같음 = true, Debug 같음 = true
        Display: 수량을 읽지 못했습니다
        source:  invalid digit found in string
        Debug:   BadQty(ParseIntError { kind: InvalidDigit })
    "KR-205:120": Display 같음 = true, source 같음 = true, Debug 같음 = true
        Display: 수량 120이(가) 최대 99를 넘습니다
        source:  없음
        Debug:   TooMany { requested: 120, max: 99 }
2. #[from]이 만든 From
    손으로: BadQty(ParseIntError { kind: InvalidDigit })
    thiserror: BadQty(ParseIntError { kind: InvalidDigit })
3. 구조체와 #[source]
    에러: 3번째 줄을 처리하지 못했습니다 (line_no = 3)
      원인: 수량을 읽지 못했습니다
      원인: invalid digit found in string
    원인이 BadQty 배리언트인가: true
```

`Debug`는 두 판 모두 `#[derive(Debug)]`가 만든 것입니다. `thiserror`는 `Debug`를 만들지 않으므로 `derive(Debug, thiserror::Error)`처럼 함께 적어야 합니다(`Error`의 슈퍼트레이트이기 때문입니다).

### 05_library_vs_app — 라이브러리와 애플리케이션

```sh
cargo run -p er-ch4-idiomatic-errors --example 05_library_vs_app
```

```text
1. 같은 처리 방침(수량 초과는 보류), 두 가지 라이브러리
    KR-102:3
      anyhow를 공개한 라이브러리:   접수 KR-102 x3
      구체 타입을 공개한 라이브러리: 접수 KR-102 x3
    KR-205:120
      anyhow를 공개한 라이브러리:   타입 없음, 메시지뿐 → 수량 120이(가) 최대 99를 넘습니다
      구체 타입을 공개한 라이브러리: 보류 (120 > 99)
    KR-310:x
      anyhow를 공개한 라이브러리:   버림 → invalid digit found in string
      구체 타입을 공개한 라이브러리: 버림 → 수량을 읽지 못했습니다
2. 애플리케이션의 에러 보고
    good, 한도 100 → 상자 6개
    good, 한도 50  → {}  총 수량 55개가 하루 처리량 50개를 넘습니다
    bad,  한도 100 → {}  3번째 줄을 처리하지 못했습니다
    bad,  한도 100 → {:#} 3번째 줄을 처리하지 못했습니다: 수량을 읽지 못했습니다: invalid digit found in string
    bad,  한도 100 → {:?} 아래 여러 줄
        | 3번째 줄을 처리하지 못했습니다
        |
        | Caused by:
        |     0: 수량을 읽지 못했습니다
        |     1: invalid digit found in string
3. 필요하면 구체 타입을 되찾습니다
    SheetError: line_no = 3, cause = BadQty(ParseIntError { kind: InvalidDigit })
    체인 길이 = 3, 맨 아래 원인 = invalid digit found in string
4. anyhow::Error의 성질
    as_ref()로 빌린 &dyn Error: 3번째 줄을 처리하지 못했습니다
    size_of: anyhow::Error = 8, Box<dyn Error + Send + Sync> = 16
    size_of: Result<(), anyhow::Error> = 8, Result<(), Box<dyn Error + Send + Sync>> = 16
```

- 1번 구역의 `KR-310:x`에서 `anyhow`를 공개한 라이브러리는 `ParseIntError`를 감싸지 않고 올려 보냈기 때문에, 호출한 쪽에는 "어느 단계에서" 실패했는지 없이 표준 라이브러리의 메시지만 보입니다.
- `RUST_BACKTRACE=1`이나 `RUST_LIB_BACKTRACE=1`을 켜고 실행하면 2번 구역의 `{:?}` 출력 끝에 빈 줄과 `Stack backtrace:`로 시작하는 백트레이스가 붙습니다. 위 출력은 두 변수가 모두 없을 때의 결과입니다.
- 4번 구역의 크기는 64비트 플랫폼 기준입니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드라서 외부 크레이트를 쓰지 않습니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일하고, 고쳐서 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd appd-a/ch4-idiomatic-errors
rustc --edition 2024 compile_fail/e0117_error_for_string_alias.rs --out-dir /tmp/rust-er-ch4
```

### 4.1 `String` 별칭에 `Error` 구현 — E0117

파일: [`compile_fail/e0117_error_for_string_alias.rs`](compile_fail/e0117_error_for_string_alias.rs)

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
  --> compile_fail/e0117_error_for_string_alias.rs:14:1
   |
14 | impl Error for SkuError {}
   | ^^^^^^^^^^^^^^^--------
   |                |
   |                `String` is not defined in the current crate
   |
   = note: impl doesn't have any local type before any uncovered type parameters
   = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
   = note: define and implement a trait or new type instead
```

- **읽는 법**: 밑줄 친 `SkuError` 아래에 `` `String` is not defined in the current crate ``라고 적혀 있습니다. 컴파일러는 별칭 `SkuError`를 풀어서 `String`으로 봅니다. 마지막 `note:`가 해결 방법(새 트레이트를 만들거나 새 타입으로 감싸기)을 알려 줍니다.
- **원인**: `type SkuError = String;`은 이름을 하나 더 붙일 뿐 새 타입을 만들지 않습니다(Part 0 ch19 2.13절). 그래서 이 구현은 `impl Error for String`과 같고, 트레이트도 타입도 표준 라이브러리의 것이라 고아 규칙에 걸립니다. `impl Error for Box<dyn Error>`를 써도 같은 E0117이 나옵니다(`` `dyn std::error::Error` is not defined in the current crate ``).
- **해결**: `struct SkuError(String);`으로 감싸고 `Display`와 `Error`를 구현합니다. 예제 01의 `SkuError`입니다(2.2절).

### 4.2 `String` 에러를 `source()`로 내주기 — E0277

파일: [`compile_fail/e0277_string_as_source.rs`](compile_fail/e0277_string_as_source.rs)

```text
error[E0277]: the trait bound `String: std::error::Error` is not satisfied
  --> compile_fail/e0277_string_as_source.rs:37:14
   |
37 |         Some(&self.cause)
   |              ^^^^^^^^^^^ the trait `std::error::Error` is not implemented for `String`
   |
   = note: required for the cast from `&String` to `&dyn std::error::Error`
```

- **읽는 법**: `note:`가 핵심입니다. `source()`의 반환 타입에 맞추려면 `&String`을 `&dyn Error`로 바꿔야(cast) 하는데, 그러려면 `String`이 `Error`를 구현해야 합니다.
- **원인**: 아래 계층이 실패를 `String`으로 알리기 때문에, 위 계층의 `LineError`는 그것을 원인으로 품더라도 체인에 연결할 수 없습니다. 트레이트 객체는 그 트레이트를 구현한 타입으로만 만들 수 있습니다([Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 4.1절).
- **해결**: 필드 타입을 `Error`를 구현한 뉴타입(`SkuError`)으로 바꿉니다. 예제 01의 `LineError`입니다. 아래 계층의 함수를 고칠 수 없더라도, 받은 `String`을 경계에서 뉴타입으로 감싸면 됩니다(예제 01의 `check_sku`가 `check_format`의 `String`을 `From<String>`으로 바꾸는 방법). `String`을 그대로 품겠다면 `source()`는 포기하고 메시지를 `Display`에 이어 붙여야 합니다(예제 01의 `LineErrorFlat`).

### 4.3 `Box<dyn Error>`를 `?`로 구체 에러 타입으로 — E0277

파일: [`compile_fail/e0277_box_dyn_error_into_enum.rs`](compile_fail/e0277_box_dyn_error_into_enum.rs)

```text
error[E0277]: `?` couldn't convert the error to `StockError`
  --> compile_fail/e0277_box_dyn_error_into_enum.rs:43:34
   |
42 | fn reserve(record: &str, want: u32) -> Result<u32, StockError> {
   |                                        ----------------------- expected `StockError` because of this
43 |     let have = read_stock(record)?;
   |                ------------------^ the trait `From<Box<dyn std::error::Error>>` is not implemented for `StockError`
   |                |
   |                this can't be annotated with `?` because it has type `Result<_, Box<(dyn std::error::Error + 'static)>>`
   |
note: `StockError` needs to implement `From<Box<dyn std::error::Error>>`
...
help: the trait `From<Box<dyn std::error::Error>>` is not implemented for `StockError`
      but trait `From<ParseIntError>` is implemented for it
```

- **읽는 법**: [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 4.3절과 같은 "`?`의 `From` 변환이 없다"는 에러입니다. 눈여겨볼 곳은 마지막 `help:`입니다. `From<ParseIntError>`는 있지만 `?`가 받은 것은 `Box<dyn Error>`라는 뜻입니다.
- **원인**: `read_stock`이 안에 `ParseIntError`를 담아 돌려주더라도, 그 사실은 타입에서 지워졌습니다. `?`는 타입만 보고 `From`을 고르므로 지워진 종류를 되살릴 수 없습니다(2.4절의 "일방통행").
- **해결**: 세 가지 중에서 고릅니다. (1) 아래 계층이 구체 타입을 돌려주게 바꿉니다. (2) 에러 타입에 상자를 담는 배리언트와 `From<Box<dyn Error + Send + Sync>>`를 둡니다(예제 03의 `StockError`). (3) 그 자리에서 `e.downcast::<ParseIntError>()`로 꺼내되, 꺼내지 못한 경우를 담을 곳도 정해야 합니다.

### 4.4 그 밖의 실수

컴파일은 되지만 결과가 틀리거나 정보를 잃는 경우입니다.

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| 에러 메시지에 `contains`/`starts_with`를 써서 종류를 판단 | 입력이 메시지에 섞이거나 문구가 바뀌면 다른 갈래로 감(예제 02) | 종류는 배리언트로 나눕니다(2.3절). |
| `Box<dyn Error>`의 맨 바깥만 다운캐스트 | 라이브러리가 한 겹 감싸면 조용히 빗나감(예제 03) | `source` 체인을 따라가며 찾거나, 구체 타입을 받습니다(2.4절). |
| 라이브러리 함수가 `anyhow::Result`를 공개 | 호출한 쪽이 `match`할 수 없고, `E: Error` 자리에 넘길 수 없음(E0277) | 라이브러리는 구체 타입을 공개합니다(2.6절). |
| 애플리케이션에서 `anyhow::Error`를 `{}`로만 출력 | 맨 바깥 메시지만 보이고 원인이 사라짐 | `{:#}`나 `{:?}`로 출력합니다(예제 05의 2번 구역). |
| `thiserror`에서 `#[error("수량을 읽지 못했습니다: {0}")] BadQty(#[from] ParseIntError)`처럼 원인 메시지를 `Display`에도 넣음 | `#[from]`이 원인을 `source()`로도 내주므로 보고서에 같은 말이 두 번 나옴 | `Display`와 `source` 중 하나만 씁니다(Part 1 ch8 2.3절). |

## 5. 연습 문제

### 연습 1. 뉴타입에서 열거형으로

예제 01의 `check_sku`에 "국가 코드 `XX`는 판매 중지"라는 규칙이 추가되었습니다. 호출한 쪽은 판매 중지 코드만 따로 모아 담당자에게 보내야 합니다. `SkuError`를 열거형으로 바꾸되, `String`을 돌려주는 기존 `check_format`에는 계속 `?`를 쓸 수 있게 하세요.

```text
KR-102 → 통과
XX-310 → 판매 중지 목록에 추가: XX
KR102  → 형식 오류: 상품 코드에 '-'가 없습니다: "KR102"
```

<details>
<summary>힌트</summary>

- `enum SkuError { Format(String), Blocked { country: String } }`처럼 두 배리언트를 둡니다.
- `impl From<String> for SkuError`가 `SkuError::Format(message)`를 돌려주게 하면 `check_format(code)?`는 그대로 동작합니다.
- 판매 중지 검사는 형식 검사 뒤에 `code.starts_with("XX-")`로 하고 `return Err(SkuError::Blocked { country: "XX".to_string() })`를 씁니다.
- 호출한 쪽은 `Err(SkuError::Blocked { country })` 갈래로 나눕니다. 메시지에서 "판매 중지"라는 낱말을 찾지 않는다는 것이 이 연습의 요점입니다.

</details>

### 연습 2. 절충형 에러를 `thiserror`로

예제 03의 `StockError`를 `thiserror`로 다시 쓰세요. 손으로 쓴 `Display`, `Error`, `From<Box<dyn Error + Send + Sync>>` 구현을 모두 지우고도 `reserve`의 본문과 3번 구역의 출력이 그대로여야 합니다.

<details>
<summary>힌트</summary>

- `#[derive(Debug, thiserror::Error)]`를 붙이고 배리언트마다 `#[error("…")]`를 씁니다. `OutOfStock`은 `"{sku} 재고가 부족합니다 ({have}개 있음, {want}개 필요)"`처럼 필드 이름을 그대로 씁니다.
- `Storage(#[from] Box<dyn Error + Send + Sync>)`로 쓰면 `From`과 `source()`가 함께 만들어집니다. `source()`가 상자 안의 에러를 돌려주는지는 3번 구역의 `원인:` 줄로 확인합니다.
- 03 예제 파일 안에서 고친다면 `thiserror`가 이미 챕터의 `[dev-dependencies]`에 있으므로 따로 추가할 것은 없습니다.

</details>

### 연습 3. 애플리케이션에서 특정 실패에만 다르게 안내하기

예제 05의 `main`에서 `plan_shipment`가 실패했을 때, 원인이 `orders::OrderError::TooMany`이면 "N번째 줄의 수량을 99개 이하로 나눠 주문하세요"라고 안내하고, 그 밖의 에러는 지금처럼 `{:#}`로 출력하세요. 입력 `[b"KR-102:3", b"KR-205:120"]`, 한도 1000으로 확인합니다.

```text
2번째 줄의 수량을 99개 이하로 나눠 주문하세요
```

<details>
<summary>힌트</summary>

- `e.downcast_ref::<orders::SheetError>()`로 `SheetError`를 꺼내면 `line_no`와 `cause` 필드를 읽을 수 있습니다.
- `if let Some(sheet) = … && let orders::OrderError::TooMany { .. } = sheet.cause { … }`처럼 let 체인으로 두 조건을 한 번에 검사할 수 있습니다(2024 에디션).
- 애플리케이션이 `anyhow`를 쓰더라도 라이브러리가 구체 타입을 공개했기 때문에 이런 대응이 가능하다는 점을 확인하세요. 예제 05의 `orders_anyhow`였다면 무엇을 해야 했을지도 생각해 보세요.

</details>

### 연습 4. 모양 고르기

다음 상황마다 2.1절의 다섯 가지 모양 중 무엇을 고를지 정하고 이유를 한 문장으로 적어 보세요.

1. crates.io에 공개할 이미지 디코더 라이브러리. 사용자는 "파일이 잘렸음"과 "지원하지 않는 형식"을 구분해 다르게 안내하고 싶어 합니다.
2. 사내 배치 작업의 `main`. 실패하면 로그에 원인을 모두 남기고 끝냅니다.
3. 플러그인을 불러오는 라이브러리. 플러그인은 아무 에러나 돌려줄 수 있지만, "플러그인을 찾지 못함"은 호출한 쪽이 기본 플러그인으로 대신하는 데 씁니다.
4. 설정 파일 검사기 모듈. 결과는 사용자에게 메시지로만 보여 주지만, 이 에러는 상위 모듈 `ConfigError`의 원인으로 들어갑니다.

<details>
<summary>힌트</summary>

- 1은 호출한 쪽이 종류를 구분하는 라이브러리이므로 에러 열거형(손으로 또는 `thiserror`)입니다(2.3·2.5절).
- 2는 보고하고 끝나는 애플리케이션의 위층이므로 `anyhow::Result`입니다(2.6절).
- 3은 대응할 실패 하나만 배리언트로, 나머지는 `Box<dyn Error + Send + Sync>` 배리언트에 담는 절충형입니다(2.4절, 예제 03의 `StockError`).
- 4는 종류를 구분할 필요는 없지만 다른 에러의 원인이 되어야 하므로 `String`이 아니라 뉴타입입니다(2.2절, 4.2절).

</details>

## 6. 참고 자료

- [Item 4: Prefer idiomatic Error types](https://effective-rust.com/errors.html): 이 챕터가 다룬 지침의 원문입니다. 최소 에러, 중첩 에러, 트레이트 객체, 라이브러리와 애플리케이션의 구분 순서로 설명합니다.
- [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html): 트레이트 정의, 메시지 관례, `dyn Error`의 다운캐스트 메서드. 같은 트레이트가 [`core::error::Error`](https://doc.rust-lang.org/core/error/trait.Error.html)로도 문서화되어 있습니다.
- The Reference의 [Orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules): E0117의 `note:`가 가리키는 절입니다.
- [`thiserror` 2.0.21 문서](https://docs.rs/thiserror/2.0.21/thiserror/): 속성 목록, "공개 API에 드러나지 않으며 손으로 쓴 구현으로 바꿔도 호환성이 깨지지 않는다"는 설명, `#[error(transparent)]`의 쓰임새(불투명한 공개 에러 타입 뒤에 내부 표현 숨기기).
- [`anyhow::Error` 1.0.104 문서](https://docs.rs/anyhow/1.0.104/anyhow/struct.Error.html): 포인터 하나 크기라는 설명, `{}`·`{:#}`·`{:?}`·`{:#?}` 출력 형식, 백트레이스와 환경 변수, `chain`·`root_cause`·`downcast_ref`·`is`.
- 이 저장소의 본편: [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)(`Result`와 `?`), [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)(`Error` 구현, 에러 열거형, `Box<dyn Error>`, 에러 보고서), [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)(트레이트 객체와 dyn 호환).
