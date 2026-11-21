# ch8. 에러와 패닉: 에러 타입 설계

[Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)에서 `panic!`과 `Result`, `?`, 그리고 "어떤 에러든 담는" `Box<dyn Error>`를 배웠습니다. 그 뒤로 여러 챕터가 이 챕터로 일을 미뤄 두었습니다.

- Part 0 ch9: 에러 타입을 직접 설계하고 `From`으로 변환하는 방법, 패닉을 붙잡는 `catch_unwind`와 패닉 훅
- [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md): 에러마다 다르게 처리해야 할 때 `Box<dyn Error>` 대신 열거형을 쓰는 기준
- [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md): `PoolCreationError`에 `std::error::Error`까지 구현하는 방법
- [Part 1 ch2](../ch2-option-result/README.md): `map_err` 없이 `?`만으로 에러를 바꾸는 설계
- [Part 1 ch3](../ch3-strings-fmt/README.md): `FromStr`의 에러 타입에 `Error` 구현하기

이 챕터는 두 갈래로 나뉩니다. 앞부분(2.2~2.7절)은 **복구할 수 있는 에러**를 값으로 잘 만드는 방법, 즉 `Error` 트레이트와 에러 타입 설계, `Box<dyn Error>`, 에러 보고입니다. 뒷부분(2.8~2.10절)은 **복구할 수 없는 에러**인 패닉을 다루는 도구, 즉 패닉 훅, `catch_unwind`, 백트레이스입니다. 패닉과 `Result` 중 무엇을 고를지는 Part 0 ch9의 2.11절에서 다뤘으므로 반복하지 않습니다.

- 패키지: `p1-ch8-errors-panic`
- 기준 문서: [`std::error`](https://doc.rust-lang.org/std/error/index.html), [`std::panic`](https://doc.rust-lang.org/std/panic/index.html), [`std::backtrace`](https://doc.rust-lang.org/std/backtrace/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024), macOS (파일 에러 메시지와 백트레이스 내용은 운영체제마다 다를 수 있습니다)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `Error` 트레이트의 정의(`Debug + Display` 슈퍼트레이트, `source`)를 읽고 그렇게 생긴 이유를 설명할 수 있으며, 직접 만든 타입에 관례에 맞는 `Display`와 `Error`를 구현할 수 있다.
2. 실패 종류를 배리언트로 가진 에러 열거형을 설계하고, 원인 에러를 품어 `source()`로 내주며, `From`을 구현해 `map_err` 없이 `?`로 변환되게 할 수 있다. 같은 원인 타입이 여러 곳에서 올 때와, 원인을 감쌀지 버릴지를 상황에 맞게 판단할 수 있다.
3. `Box<dyn Error>`와 `Box<dyn Error + Send + Sync>`로 여러 에러를 묶고 `is`·`downcast_ref`·`downcast`로 구체 타입을 되찾을 수 있으며, 구체 에러 타입과 `Box<dyn Error>` 중 무엇을 쓸지 근거를 들어 고를 수 있다.
4. `source` 체인을 따라가며 사람이 읽을 에러 보고서를 출력할 수 있고, `Backtrace::capture`가 환경 변수에 따라 어떻게 동작하는지 설명하며 에러 타입에 백트레이스를 담을 수 있다.
5. 패닉 훅을 바꾸고 되돌릴 수 있고, `catch_unwind`로 패닉을 붙잡고 `resume_unwind`로 다시 일으킬 수 있으며, `UnwindSafe`가 막는 문제와 `catch_unwind`를 일반 에러 처리에 쓰면 안 되는 이유를 설명할 수 있다.

## 2. 핵심 개념

### 2.1 두 체계의 지도

`std::error` 모듈 문서는 Rust의 에러 처리를 서로 보완하는 두 체계로 설명합니다. 패닉 체계는 주로 프로그램에서 발견된 **버그**를 나타내고, `Result`와 에러 트레이트 체계는 **예상할 수 있는 실행 중 실패**를 나타냅니다. 문서가 정리한 역할별 도구를 표로 옮기면 다음과 같습니다.

| 역할 | 패닉 체계 | 에러 체계 | 이 챕터 |
| --- | --- | --- | --- |
| 만들기 | `panic!`, `panic_any` | 직접 만든 에러 타입 | 2.2~2.4절 |
| 전파하기 | 되감기(자동), `resume_unwind` | `Result`, `?` | 2.4, 2.9절 |
| 보고하기 | 패닉 훅(`set_hook`, `take_hook`, `PanicHookInfo`) | `Error` 트레이트(`Display`, `source`) | 2.7, 2.8절 |
| 대응하기·버리기 | `catch_unwind` | `match`, 다운캐스트 | 2.3, 2.5, 2.9절 |

두 체계는 완전히 떨어져 있지 않습니다. `unwrap`과 `expect`는 `Err`를 패닉으로 바꾸는 다리이고(Part 0 ch9), `catch_unwind`는 반대로 패닉을 `Result`로 받는 다리입니다(2.9절).

### 2.2 `Error` 트레이트

`std::error::Error`의 정의는 다음과 같습니다(1.100.0 문서 기준).

```rust
pub trait Error: Debug + Display {
    // Provided methods
    fn source(&self) -> Option<&(dyn Error + 'static)> { ... } // 기본 구현은 None
    fn description(&self) -> &str { ... }                       // 1.42.0부터 쓰지 않음(deprecated)
    fn cause(&self) -> Option<&dyn Error> { ... }               // 1.33.0부터 쓰지 않음(deprecated)
    fn provide<'a>(&'a self, request: &mut Request<'a>) { ... } // nightly 전용 실험 API
}
```

**`Debug + Display`가 슈퍼트레이트인 이유.** 에러는 두 부류의 사람에게 보여 줍니다. 사용자에게는 무엇이 잘못됐는지 알려 주는 문장(`Display`, `{}`)을, 개발자에게는 내부 구조(`Debug`, `{:?}`)를 보여 줍니다. `Error`가 두 트레이트를 요구하므로, 구체 타입을 모르는 코드도 "에러라면 두 형식으로 출력할 수 있다"고 믿을 수 있습니다. `main`이 `Err`를 돌려줄 때 Rust가 디버그 형식으로 출력하고(Part 0 ch9의 2.10절), `Box<dyn Error>`를 `{}`로 출력할 수 있는 것이 이 약속 덕분입니다. 슈퍼트레이트 문법은 [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)에서 배웠습니다.

**`source()`** 는 "이 에러를 일으킨 더 낮은 수준의 에러"를 돌려줍니다. 설정 파일을 읽는 모듈이 `io::Error`를 받아 자기 에러로 바꿔 알릴 때, 원래의 `io::Error`를 `source()`로 꺼낼 수 있게 해 두는 식입니다. 문서는 이것을 "추상화 경계를 넘을 때" 쓴다고 설명합니다. 위층은 자기 말로 실패를 설명하고, 디버깅에 필요한 아래층의 사정은 `source()`로 열어 둡니다. 기본 구현이 `None`이므로 원인이 없는 에러는 구현하지 않아도 됩니다.

나머지 메서드는 쓰지 않습니다. `description()`은 `Display`(또는 `to_string()`)로, `cause()`는 다운캐스트(2.5절)를 지원하는 `source()`로 대체되어 쓰지 않는 메서드(deprecated)로 표시되어 있습니다. `provide()`는 에러에서 백트레이스 같은 부가 정보를 타입별로 꺼내기 위한 nightly 전용 실험 API입니다.

모든 메서드에 기본 구현이 있으므로 가장 작은 구현은 빈 블록입니다([`01_error_trait.rs`](examples/01_error_trait.rs)).

```rust
#[derive(Debug)]
struct EmptyInput;

impl fmt::Display for EmptyInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "입력이 비어 있습니다")
    }
}

impl Error for EmptyInput {}
```

**`Display` 메시지의 관례.** `Error` 문서는 에러 메시지가 보통 "마침표 없는 짧은 소문자 문장"이라고 설명하고 `"invalid digit found in string"`을 예로 듭니다. 실제로 표준 에러의 메시지는 모두 이 모양입니다(예제 01의 2번 구역). 에러 메시지는 혼자 쓰이기보다 `"원인: {e}"`나 `expect`의 `"메시지: {e:?}"`처럼 다른 문장 안에 끼워 넣어지는 일이 많아서, 문장부호를 붙이면 출력이 어색해지기 때문입니다. 한국어에는 대소문자가 없으므로 이 챕터는 "짧게, 마침표 없이"만 따릅니다.

**앞 챕터의 에러 타입에 한 줄 더하기.** [Part 1 ch3](../ch3-strings-fmt/README.md)의 `ParseVersionError`와 Part 0 ch20의 `PoolCreationError`는 `Debug`와 `Display`를 이미 구현했으므로 `impl Error for 타입 {}` 한 줄이면 `Error`가 됩니다. 이 한 줄이 중요한 이유는 `Box<dyn Error>`로의 변환(`impl<E: Error> From<E> for Box<dyn Error>`)이 `E: Error`를 요구하기 때문입니다. 이 줄이 없으면 `Box<dyn Error>`를 돌려주는 함수에서 `version.parse::<Version>()?`가 컴파일되지 않습니다(4.6절의 표). 예제 01의 3번 구역은 `&str` 메시지, `ParseVersionError`, `ParseIntError` 세 가지 에러를 `?` 하나로 `Box<dyn Error>`에 담습니다.

### 2.3 에러 열거형 설계

Part 0 ch9의 `read_count`는 파일 읽기(`io::Error`)와 숫자 변환(`ParseIntError`)의 실패를 `Box<dyn Error>`로 돌려줬습니다. 호출한 쪽이 "파일이 없으면 0, 숫자가 아니면 안내"처럼 실패 종류마다 다르게 대응해야 한다면 에러 열거형을 만듭니다. [`02_error_enum.rs`](examples/02_error_enum.rs)는 이 함수를 열거형 판으로 다시 씁니다.

```rust
#[derive(Debug)]
enum ReadCountError {
    Io(io::Error),                     // 파일을 읽지 못함
    Parse(ParseIntError),              // 내용이 숫자가 아님
    TooMany { count: u32, max: u32 },  // 범위를 넘음 (원인 에러 없음)
}

impl fmt::Display for ReadCountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => write!(f, "개수 파일을 읽지 못했습니다"),
            Self::Parse(_) => write!(f, "개수 파일의 내용이 숫자가 아닙니다"),
            Self::TooMany { count, max } => write!(f, "개수가 최대값을 넘습니다 ({count} > {max})"),
        }
    }
}

impl Error for ReadCountError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
            Self::TooMany { .. } => None,
        }
    }
}
```

설계할 때 지키는 원칙은 다음과 같습니다.

1. **실패 종류마다 배리언트 하나.** 호출한 쪽은 `match`로 종류를 나누고, 완전성 검사 덕분에 배리언트가 늘면 처리하지 않은 곳에서 컴파일 에러가 납니다.
2. **대응에 필요한 정보를 담기.** `TooMany`는 판단에 쓴 `count`와 `max`를, `Io`는 `kind()`를 물어볼 수 있는 `io::Error`를 담습니다. 문자열 하나만 담으면 호출한 쪽은 문장을 해석해야 합니다.
3. **`Display`에는 이 층의 실패만.** `"개수 파일의 내용이 숫자가 아닙니다"`까지만 쓰고 `ParseIntError`의 메시지는 붙이지 않습니다. `Error` 문서는 원인 에러를 `source()`로 내주거나 `Display`에 넣거나 **둘 중 하나만** 하라고 권합니다. 둘 다 하면 보고서에 같은 말이 두 번 나옵니다(예제 05의 4번 구역).
4. **원인은 `source()`로.** 반환 타입의 `dyn Error + 'static`은 "원인 에러의 실제 타입이 `'static`이 아닌 참조를 품지 않는다"는 조건입니다. 이 조건이 있어야 받은 쪽이 원인을 구체 타입으로 다운캐스트할 수 있습니다(2.5절).

호출한 쪽은 이렇게 대응합니다. 매치 가드(Part 0 ch18)로 `io::Error`의 종류까지 함께 봅니다.

```rust
match read_count(path) {
    Ok(n) => n,
    Err(ReadCountError::Io(e)) if e.kind() == io::ErrorKind::NotFound => 0,
    Err(ReadCountError::TooMany { max, .. }) => max,
    Err(e) => { /* 사용자에게 알리고 */ 0 }
}
```

### 2.4 `From`으로 `?`가 변환하게 하기

`?`는 `Err(e)`를 만나면 `return Err(From::from(e))`처럼 동작합니다. `From`의 일반 원리는 [ch6](../ch6-core-traits/README.md)에서 배웠으므로, 여기서는 에러에 적용하는 일만 남았습니다. 원인 에러 타입마다 `From`을 구현하면 `?`가 알아서 열거형으로 감쌉니다.

```rust
impl From<io::Error> for ReadCountError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
// impl From<ParseIntError> for ReadCountError { ... } 도 같은 모양

fn read_count(path: &Path) -> Result<u32, ReadCountError> {
    let text = fs::read_to_string(path)?;   // io::Error → ReadCountError::Io
    let count: u32 = text.trim().parse()?;  // ParseIntError → ReadCountError::Parse
    if count > MAX_COUNT {
        return Err(ReadCountError::TooMany { count, max: MAX_COUNT });
    }
    Ok(count)
}
```

본문이 Part 0 ch9의 `Box<dyn Error>` 판과 거의 같다는 점에 주목하세요. 바뀐 것은 반환 타입뿐인데, 호출한 쪽이 얻는 정보는 훨씬 많아졌습니다. 예제 08에서 백트레이스를 찍어 보면 `parse_limit`의 `?`가 `FromResidual::from_residual`을 거쳐 `From::from`을 부르는 모습이 실제로 보입니다.

**Part 1 ch2의 `map_err`가 사라지는 모습.** ch2의 `AgeError`에는 변환이 정의되어 있지 않아서 `map_err`로 직접 감쌌습니다. `From`을 구현하면 그 줄이 없어집니다([`03_from_conversion.rs`](examples/03_from_conversion.rs)의 1번 구역).

```rust
// ch2 판: 변환을 호출할 때마다 적습니다.
input.trim().parse::<i32>().map_err(AgeError::NotNumber).and_then(check_range)

// From 판: impl From<ParseIntError> for AgeError가 변환을 한 곳에 적어 둡니다.
check_range(input.trim().parse()?)
```

**같은 원인 타입이 두 군데서 올 때.** `"가로x세로"`를 읽는다면 가로와 세로 모두 `ParseIntError`로 실패합니다. 한 타입에 `From<ParseIntError>`는 하나만 구현할 수 있고(4.1절), `?`는 에러의 **타입**만 보고 변환을 고르므로 어느 자리에서 왔는지 구분할 수 없습니다. `From`만 쓰면 "숫자가 아닌 값이 있습니다"까지만 말할 수 있습니다. 위치가 중요하면 `From`을 구현하지 말고 자리마다 `map_err`로 맥락을 붙입니다.

```rust
let w = w.parse().map_err(|source| SizeErrorB::BadNumber { field: "가로", source })?;
let h = h.parse().map_err(|source| SizeErrorB::BadNumber { field: "세로", source })?;
```

| 상황 | 방법 |
| --- | --- |
| 원인 타입마다 배리언트 하나이고, 어디서 났는지는 중요하지 않음 | `From` 구현 + `?` |
| 같은 원인 타입이 여러 곳에서 오고, 위치나 맥락이 필요함 | 맥락 필드를 가진 배리언트 + `map_err(..)?` |
| 우리 코드가 판단한 실패(범위 초과 등) | `return Err(MyError::TooMany { .. })` |

**감쌀까, 버릴까.** 원인 에러를 다루는 방법은 두 가지입니다(예제 03의 3번 구역).

- **감싸기**: 원인을 필드로 품고 `source()`로 내줍니다. `"70000"`을 포트로 읽으면 보고서에 `number too large to fit in target type`이라는 원인이 남습니다. 대신 원인 타입이 공개 열거형에 드러나므로, 그 타입을 바꾸면(예: 다른 파싱 크레이트로 교체) 에러 타입도 바뀝니다.
- **버리고 새 에러로 대신하기**: `from`에서 원인을 버리고 내 에러만 만듭니다. 원인 타입이 드러나지 않지만, `"http"`와 `"70000"`이 왜 실패했는지 구분할 수 없게 됩니다.

원인 타입은 숨기면서 체인은 남기고 싶다면 원인을 `Box<dyn Error + Send + Sync>` 필드로 감싸고 `source()`에서 `Some(self.0.as_ref())`를 돌려주는 방법도 있습니다. 대부분은 감싸기가 기본입니다. 정보는 한 번 버리면 되찾을 수 없기 때문입니다.

`Display`, `Error`, `From`을 배리언트마다 손으로 쓰는 일은 반복이 많습니다. 표준 라이브러리의 이 패턴을 매크로로 줄여 주는 크레이트가 있습니다. [부록 A ch4](../../appd-a/ch4-idiomatic-errors/README.md)가 `thiserror`와 `anyhow`로 같은 종류의 에러 타입을 만들어 보이고, 명령줄 프로그램에서의 쓰임은 Part 7 `ch2-implementation-errors`에서 다룹니다. 크레이트가 만들어 주는 코드도 결국 이 절의 구현과 같으므로, 손으로 한 번 써 보는 것이 그 크레이트를 이해하는 지름길입니다.

### 2.5 `Box<dyn Error>`

`Box<dyn Error>`는 "`Error`를 구현한 어떤 타입의 값을 담은 `Box`"인 트레이트 객체입니다(Part 0 ch17). 표준 라이브러리가 다음 `From` 구현을 제공하므로 여러 종류의 값이 `?`나 `.into()`로 이 타입이 됩니다.

| 변환 | 쓰는 모양 | 안에 담기는 것 |
| --- | --- | --- |
| `From<&str>` | `ok_or("'='가 없습니다")?`, `"..".into()` | 표준 라이브러리의 비공개 타입 |
| `From<String>` | `format!(..).into()` | 표준 라이브러리의 비공개 타입 |
| `From<Cow<str>>` | `Cow<str>` 값에 `.into()` | 표준 라이브러리의 비공개 타입 |
| `impl<E: Error> From<E>` | 표준 에러나 직접 만든 에러에 `?` | `E` 그대로 |

같은 구현이 `Box<dyn Error + Send + Sync>`에도 있습니다(이때 `E`는 `Error + Send + Sync`여야 합니다). 문자열에서 만든 에러를 디버그 형식으로 출력하면 따옴표로 감싼 문자열이 나옵니다(예제 04의 1번 구역).

**다운캐스트로 구체 타입 되찾기.** `dyn Error`에는 안에 든 타입을 실행 중에 확인하는 메서드가 있습니다. 모두 `T: Error + 'static`을 요구합니다.

| 메서드 | 받는 것 | 돌려주는 것 |
| --- | --- | --- |
| `is::<T>()` | `&self` | 안에 든 것이 `T`인가 (`bool`) |
| `downcast_ref::<T>()` | `&self` | `Option<&T>` |
| `downcast_mut::<T>()` | `&mut self` | `Option<&mut T>` |
| `downcast::<T>()` | `Box<dyn Error>` (소비) | `Result<Box<T>, Box<dyn Error>>`, 틀리면 원래 `Box`를 돌려받음 |

```rust
fn react(err: &(dyn Error + 'static)) -> String {
    if let Some(q) = err.downcast_ref::<QuotaExceeded>() {
        format!("할당량 초과 → {}만큼 줄이세요", q.used - q.limit)
    } else if let Some(p) = err.downcast_ref::<ParseIntError>() {
        format!("숫자 형식 오류 → 종류: {:?}", p.kind())
    } else {
        format!("그 밖의 에러 → {err}")
    }
}
```

`'static`이 필요한 이유는 다운캐스트가 실행 중에 타입을 비교하는 방식이라, 빌린 참조를 품은 타입의 라이프타임까지 같은지는 확인할 수 없기 때문입니다(`Any`와 다운캐스트 일반은 [ch14](../ch14-ffi-any-misc/README.md)). Part 0 ch17은 "`Box<dyn Error>`는 안에 무엇이 들었는지 타입으로 알 수 없다"고 했는데, 정확히는 **실행 중에 물어볼 수는 있지만 컴파일러가 도와주지 않는다**는 뜻입니다. 위의 `else` 갈래를 빠뜨려도 경고가 없고, 문자열에서 만든 에러는 타입 이름이 공개되지 않아 꺼낼 방법도 없습니다.

**스레드로 넘길 때: `+ Send + Sync`.** `thread::spawn`의 클로저가 돌려주는 값은 `join`을 부른 스레드로 옮겨지므로 `Send`여야 합니다(Part 0 ch15). `Box<dyn Error>`는 `Rc`를 품은 에러처럼 `Send`가 아닌 것도 담을 수 있으므로 다른 스레드로 보낼 수 없습니다(4.4절). `Box<dyn Error + Send + Sync>`는 "다른 스레드로 보내도 되고(`Send`), 여러 스레드가 함께 참조해도 되는(`Sync`) 에러만 담는다"는 약속이 붙은 타입입니다. 표준 라이브러리의 에러는 대부분 이 조건을 만족하므로, 스레드와 함께 쓰는 코드에서는 처음부터 이 모양을 쓰는 경우가 많습니다. `Box<dyn Trait>`의 라이프타임은 따로 적지 않으면 `'static`이라서 `Box<dyn Error + Send + Sync + 'static>`과 같은 타입입니다.

`Send + Sync`가 붙은 `Box`는 붙지 않은 `Box`에 그냥 대입할 수 있지만, `?`는 이 변환을 해 주지 않습니다(4.6절). 그리고 `Box<dyn Error>` 자신은 `Error`를 구현하지 않는다는 점도 기억해 둡니다(4.5절).

### 2.6 구체 타입이냐, `Box<dyn Error>`냐

| | 에러 열거형 (구체 타입) | `Box<dyn Error>` |
| --- | --- | --- |
| 호출한 쪽의 대응 | `match`, 완전성 검사가 빠뜨림을 잡음 | 다운캐스트, 빠뜨려도 컴파일러가 모름 |
| 새 종류의 실패 | 배리언트를 추가 (호출한 쪽도 고쳐야 할 수 있음) | 무엇이든 그냥 담김 |
| 작성 비용 | `Display`·`Error`·`From` 구현 | 거의 없음 (`?`, `.into()`) |
| 실행 비용 | 값 크기가 가장 큰 배리언트만큼 | 에러가 날 때 힙 할당 |

기준은 **에러를 받은 쪽이 그 내용을 보고 판단하는가**입니다.

- **라이브러리**처럼 호출하는 쪽이 따로 있고, 그쪽이 실패 종류에 따라 재시도·기본값·안내를 정해야 한다면 구체 타입을 돌려줍니다. Rustlings `errors6.rs`의 머리 주석도 같은 이유로, 호출한 쪽이 에러 내용을 보고 결정해야 할 수 있는 라이브러리 코드에는 `Box<dyn Error>` 같은 포괄 에러 타입을 권하지 않는다고 설명합니다.
- **애플리케이션의 위쪽**(예: `main` 가까이)처럼 에러를 받으면 결국 보고하고 끝낼 뿐이라면 `Box<dyn Error>`가 간단합니다.

두 방법은 섞어 쓸 수 있습니다. 모듈마다 에러 열거형을 두고 `Error`를 구현해 두면, `main`의 `Box<dyn Error>`로는 `?` 하나로 올라갑니다(예제 01의 3번 구역, 예제 05).

### 2.7 에러 보고: `source` 체인 따라가기

에러가 여러 층으로 감싸여 있으면 `source()`를 반복해서 불러 맨 아래 원인까지 내려갈 수 있습니다. 이 연결을 `source` 체인이라고 부릅니다. [`05_error_report.rs`](examples/05_error_report.rs)는 `AppError` → `ConfigError` → `ParseIntError`의 세 층을 보고합니다.

```rust
fn report(err: &dyn Error) {
    println!("    에러: {err}");
    let mut current = err.source();
    let mut depth = 1;
    while let Some(cause) = current {
        println!("    {:indent$}원인 {depth}: {cause}", "", indent = depth * 2);
        current = cause.source();
        depth += 1;
    }
}
```

```text
    에러: 서버를 시작하지 못했습니다
      원인 1: 2번째 줄의 port 값을 읽지 못했습니다
        원인 2: invalid digit found in string
```

각 층의 `Display`가 자기 층의 사정만 말하기 때문에 보고서가 위에서 아래로 "무엇을 하려다 → 왜 → 근본 원인" 순서로 읽힙니다. 같은 에러를 `main`이 `Err`로 돌려줬다면 Rust는 디버그 형식으로 출력합니다(Part 0 ch9의 2.10절).

```text
Error: Startup(BadValue { line: 2, key: "port", source: ParseIntError { kind: InvalidDigit } })
```

개발자에게는 쓸모 있지만 사용자에게 보여 줄 문장은 아닙니다. 그래서 사용자가 보는 프로그램은 `main`에서 에러를 `match`로 받아 보고서를 직접 출력하고, 실패를 뜻하는 종료 코드로 끝냅니다. 종료 코드를 정하는 `ExitCode`와 `process::exit`은 [ch10](../ch10-env-process-time/README.md)에서 다룹니다. 표준 라이브러리에도 이 보고를 해 주는 `std::error::Report`와 체인을 반복자로 내주는 `sources()`가 있지만, 둘 다 아직 nightly 전용 실험 API입니다.

### 2.8 패닉 훅

패닉이 일어나면 두 단계가 차례로 실행됩니다.

1. **패닉 훅(panic hook)** 이 실행됩니다. 기본 훅은 `thread '이름' (스레드 ID) panicked at 파일:줄:열:`과 메시지를 표준 에러에 출력하고, `RUST_BACKTRACE`가 켜져 있으면 백트레이스도 출력합니다. `note: run with RUST_BACKTRACE=1 ...` 안내는 기본 훅이 처음 출력할 때 한 번만 나옵니다.
2. **되감기**(또는 `panic = "abort"`이면 즉시 종료)가 시작됩니다(Part 0 ch9의 2.3절).

훅은 되감기보다 먼저 실행되므로 abort 전략에서도 실행됩니다. 훅은 `std::panic`의 함수로 바꿉니다.

| 함수 | 시그니처(요약) | 하는 일 |
| --- | --- | --- |
| `set_hook` | `fn set_hook(hook: Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>)` | 새 훅을 등록합니다. 이전 훅은 버려집니다. |
| `take_hook` | `fn take_hook() -> Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>` | 지금 훅을 떼어 돌려주고 기본 훅을 다시 등록합니다. |

훅의 타입에 `Sync + Send + 'static`이 붙은 이유는 훅이 **프로세스 전체에 하나뿐인 전역 상태**이고 어느 스레드에서 패닉하든 불리기 때문입니다. 패닉 중인 스레드에서 두 함수를 부르면 패닉합니다.

훅이 받는 `PanicHookInfo`(1.81.0에서 안정화, 예전 이름 `PanicInfo`는 쓰지 않는 별칭)의 메서드는 다음과 같습니다.

| 메서드 | 돌려주는 것 | 안정화 |
| --- | --- | --- |
| `location()` | `Option<&'static Location<'static>>`. `file()`, `line()`, `column()`으로 위치를 읽습니다. 문서에 따르면 현재는 늘 `Some`입니다. | 1.10.0 |
| `payload()` | `&(dyn Any + Send)`. `panic!`에 넘긴 값인 페이로드(payload)입니다. | 1.10.0 |
| `payload_as_str()` | `Option<&str>`. 페이로드가 `&'static str`이나 `String`이면 `Some`입니다. | 1.91.0 |

2021 에디션부터 `panic!` 매크로의 페이로드는 언제나 `&'static str` 또는 `String`이라서 `payload_as_str()`로 충분합니다. 다른 타입의 페이로드는 `panic_any(값)`으로 패닉했을 때만 생깁니다.

[`07_panic_hook.rs`](examples/07_panic_hook.rs)는 훅을 바꾸고, 되돌리고, 이어 붙입니다. 이어 붙이기는 기본 훅을 `take_hook`으로 받아 새 훅 안에서 부르는 모양입니다.

```rust
let default_hook = panic::take_hook(); // 지금 등록된 기본 훅을 받습니다.
panic::set_hook(Box::new(move |info| {
    eprintln!("[기록] {}", one_line_report(info)); // 내 처리
    default_hook(info);                             // 원래 출력
}));
```

훅을 바꾸는 곳은 보통 프로그램 시작 부분 한 곳입니다. 패닉을 로그 파일이나 모니터링 시스템에 남기거나, 명령줄 프로그램이 사용자에게 "버그를 신고해 주세요" 같은 안내를 보여 줄 때 씁니다. 전역 상태이므로 라이브러리가 마음대로 바꾸지 않고, 바꿨다면 다 쓴 뒤 되돌려 두는 것이 예의입니다(예제 07의 마지막 부분).

### 2.9 `catch_unwind`: 되감기 중인 패닉 붙잡기

```rust
pub fn catch_unwind<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> Result<R>
```

반환 타입의 `Result<R>`는 `std::thread::Result<R>`, 즉 `Result<R, Box<dyn Any + Send + 'static>>`의 별칭입니다. Part 0 ch15에서 `JoinHandle::join`이 돌려주던 바로 그 타입입니다. 클로저가 정상적으로 끝나면 `Ok(반환값)`, 패닉하면 `Err(페이로드)`입니다. 페이로드는 `Error`를 구현하지 않으므로 `?`로 전파하지 않고, `downcast_ref`로 문자열을 꺼냅니다. `dyn Any`는 "어떤 `'static` 타입이든"이라는 뜻이고 `downcast_ref::<T>()`는 "안에 든 것이 `T`이면 `Some(&T)`"입니다(`Any` 자체는 [ch14](../ch14-ffi-any-misc/README.md)).

```rust
fn payload_text(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        format!("&str 페이로드: {s}")
    } else if let Some(s) = payload.downcast_ref::<String>() {
        format!("String 페이로드: {s}")
    } else {
        "문자열이 아닌 페이로드".to_string()
    }
}

match panic::catch_unwind(job) {
    Ok(v) => println!("성공: {v}"),
    Err(payload) => println!("패닉을 붙잡음 → {}", payload_text(&*payload)),
}
```

`payload`의 타입은 `Box<dyn Any + Send>`이고, `&*payload`는 상자 안의 값을 빌린 `&(dyn Any + Send)`입니다. 여기서 `&payload`를 넘기면 컴파일은 되지만 `Box` 자체가 `Any` 값으로 취급되어 다운캐스트가 모두 실패합니다(4.6절).

[`06_catch_unwind.rs`](examples/06_catch_unwind.rs)에서 확인한 페이로드 타입은 다음과 같습니다. `panic!`이 어느 쪽을 만들지는 구현이 정하는 부분이므로 두 타입을 모두 확인해야 합니다.

| 패닉 | 페이로드 타입 |
| --- | --- |
| 0으로 나눔(`divide(10, 0)`), `panic!("고정 문자열")`, `assert!(조건, "고정 문자열")` | `&'static str` |
| `panic!("알 수 없는 모드: {other}")`처럼 실행 중의 값을 넣은 서식 | `String` |
| 범위 밖 인덱스, `Result::unwrap` | `String` |

**붙잡아도 훅은 이미 실행됐습니다.** 문서대로 훅은 패닉이 붙잡히기 전에 실행되므로, 기본 훅이면 `catch_unwind` 안의 패닉도 표준 에러에 메시지를 남깁니다. 조용히 붙잡고 싶다면 훅을 바꿔야 합니다(연습 4).

**`UnwindSafe`와 `AssertUnwindSafe`.** `catch_unwind`의 클로저에는 `UnwindSafe` 바운드가 있습니다. 패닉은 함수를 중간에서 빠져나가게 하므로, 데이터를 고치던 도중이라면 불변식이 깨진 채로 남을 수 있습니다. 보통은 패닉한 스레드가 끝나므로 그 상태를 아무도 보지 못하지만, `catch_unwind`는 패닉 뒤에도 같은 변수를 계속 쓸 수 있게 해 줍니다. 그래서 `&mut T`나 `&RefCell<T>`처럼 **패닉 뒤에 반쯤 바뀐 상태를 보여 줄 수 있는 타입**은 `UnwindSafe`가 아니고, 이런 것을 캡처한 클로저는 컴파일되지 않습니다(4.3절). 이렇게 패닉 경계를 넘어도 안전한지를 되감기 안전성(unwind safety)이라고 부릅니다.

예제 06의 3번 구역이 그 상황입니다. 출금 뒤 입금에서 패닉하면 두 잔액의 합이 1000에서 700으로 줄어든 상태가 남습니다. `AssertUnwindSafe(클로저)`로 감싸면 컴파일되지만, 그것은 "패닉 뒤의 상태는 내가 확인하겠다"는 약속입니다. 문서는 클로저 전체 대신 캡처하는 값 하나하나를 `AssertUnwindSafe`로 감싸는 방법도 보여 줍니다. 나중에 위험한 캡처가 추가되면 그때 컴파일 에러로 알 수 있기 때문입니다. `UnwindSafe`는 `unsafe` 트레이트가 아니라 "과속 방지턱" 같은 경고 장치이며, `unsafe` 없는 코드에서 패닉이 메모리 안전성을 깨뜨리지는 않습니다. `&Mutex<T>`는 포이즈닝(Part 0 ch15)이라는 자체 방지턱이 있어 `UnwindSafe`입니다. 예외 안전성 전반은 Part 10 `ch8-unwinding`에서 다룹니다.

**`resume_unwind`** 는 붙잡은 페이로드로 되감기를 다시 시작합니다. 시그니처는 `fn resume_unwind(payload: Box<dyn Any + Send>) -> !`이고, **패닉 훅을 부르지 않습니다.** 붙잡아 기록만 하고 원래 패닉을 그대로 위로 올려보낼 때 씁니다. 예제 06의 4번 구역에서 표준 에러의 패닉 메시지는 처음 패닉한 한 번만 나옵니다.

**써도 되는 곳과 안 되는 곳.** `catch_unwind`는 "한 작업의 버그가 프로그램 전체를 멈추지 않게 하는 경계"에 씁니다. 스레드 풀이 작업마다 패닉을 가두거나, 서버가 요청 하나의 패닉을 그 요청의 실패로 바꾸거나, C 코드가 부르는 Rust 함수(FFI)에서 패닉을 에러 코드로 바꾸는 경우입니다. 문서에 따르면 `extern "C"`로 정의한 함수는 패닉하면 자동으로 프로세스를 끝내므로, 끝내지 않고 더 부드럽게 처리하고 싶을 때 `catch_unwind`를 씁니다. 일반적인 에러 처리(다른 언어의 try/catch)로 쓰면 안 됩니다. 문서도 자주 실패할 수 있는 함수에는 `Result`가 맞다고 말합니다.

- 패닉은 버그를 뜻합니다. 예상되는 실패를 패닉으로 알리면 시그니처에서 실패 가능성이 사라지고, 호출한 쪽이 `?`와 `match`로 다룰 수 없습니다.
- 모든 패닉을 붙잡는다는 보장이 없습니다. `panic = "abort"`로 빌드하면 패닉이 곧 프로세스 종료라서 `catch_unwind`가 아무것도 붙잡지 못합니다. 예제 06을 `rustc --edition 2024 -C panic=abort`로 컴파일해 실행하면 첫 패닉 메시지를 출력한 뒤 종료 코드 134(macOS의 zsh 기준)로 끝납니다. 프로필에서 패닉 전략을 고르는 방법은 [Part 3 ch6](../../part3-cargo/ch6-profiles-build-performance/README.md)와 Part 5 `ch2-codegen-options`에서 다룹니다.
- 붙잡아도 훅이 메시지를 출력하고, 되감기 비용과 `UnwindSafe` 검토가 따라옵니다.

### 2.10 백트레이스

Part 0 ch9의 2.4절에서는 패닉할 때 기본 훅이 출력하는 백트레이스를 읽었습니다. `std::backtrace::Backtrace`(1.65.0)는 백트레이스를 **값으로** 만들어 에러에 담아 둘 수 있게 해 줍니다.

| 함수·메서드 | 하는 일 |
| --- | --- |
| `Backtrace::capture()` | 환경 변수가 허락할 때만 현재 스레드의 백트레이스를 기록합니다. 꺼져 있으면 비어 있는 값을 만듭니다. |
| `Backtrace::force_capture()` | 환경 변수와 상관없이 늘 기록합니다. |
| `Backtrace::disabled()` | 기록하지 않은 빈 값을 만듭니다. |
| `status()` | `BacktraceStatus::Captured`(기록됨), `Disabled`(환경 변수로 꺼짐), `Unsupported`(플랫폼이 지원하지 않음) |
| `Display`, `Debug` | `{}`로 출력하면 패닉 백트레이스와 같은 모양의 목록이 나옵니다. |

`BacktraceStatus`는 배리언트가 늘어날 수 있는 열거형(`#[non_exhaustive]`)이라 `match`에 `_` 갈래가 필요합니다(Part 0 ch9의 `ErrorKind`와 같습니다).

**환경 변수 두 개.** `capture()`는 `RUST_LIB_BACKTRACE`가 있으면 그 값을, 없으면 `RUST_BACKTRACE`를 봅니다. 값이 `0`이면 끄고, 다른 값이면 켭니다. 둘 다 없으면 꺼져 있습니다. 반면 기본 패닉 훅은 `RUST_BACKTRACE`만 봅니다. 이 저장소의 툴체인에서 확인한 결과는 다음과 같습니다.

| `RUST_BACKTRACE` | `RUST_LIB_BACKTRACE` | 패닉할 때 백트레이스 출력 | `capture()`의 상태 |
| --- | --- | --- | --- |
| 없음 | 없음 | 안 함 | `Disabled` |
| `1` | 없음 | 함 | `Captured` |
| 없음 | `1` | 안 함 | `Captured` |
| `1` | `0` | 함 | `Disabled` |
| `0` | `1` | 안 함 | `Captured` |

변수가 둘인 이유는 비용 때문입니다. 백트레이스를 기록하는 일은 느리고 메모리를 씁니다. 에러는 패닉보다 훨씬 자주 만들어지므로, "패닉 백트레이스는 보고 싶지만 에러마다 기록하는 비용은 내고 싶지 않다"(`RUST_BACKTRACE=1 RUST_LIB_BACKTRACE=0`)처럼 따로 정할 수 있게 했습니다. 문서에 따르면 환경 변수의 값은 처음 백트레이스를 만들 때 읽어 저장해 두므로, 실행 중에 바꿔도 반영되지 않을 수 있습니다.

**에러에 담아 두기.** 에러가 처음 만들어지는 곳, 즉 `From::from` 안에서 `capture()`를 부르는 것이 일반적인 모양입니다([`08_backtrace.rs`](examples/08_backtrace.rs)).

```rust
impl From<ParseIntError> for LimitError {
    fn from(source: ParseIntError) -> Self {
        Self { source, backtrace: Backtrace::capture() }
    }
}
```

`Error` 트레이트에는 백트레이스를 꺼내는 안정 메서드가 없으므로(`provide`는 nightly 전용) 에러 타입에 `fn backtrace(&self) -> &Backtrace` 같은 게터를 직접 둡니다. 꺼져 있으면 `capture()`는 거의 비용이 없으므로, 평소에는 끄고 문제를 조사할 때만 `RUST_LIB_BACKTRACE=1`로 실행하면 됩니다. 파일과 줄 번호가 나오려면 디버그 정보가 있어야 하는 것은 패닉 백트레이스와 같습니다(Part 0 ch9의 2.4절).

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_error_trait.rs`](examples/01_error_trait.rs) | 가장 작은 `Error` 구현, 표준 에러의 메시지, ch3 `ParseVersionError`에 `Error` 구현 | 1 |
| [`examples/02_error_enum.rs`](examples/02_error_enum.rs) | `read_count`의 에러 열거형 판: 배리언트, `Display`, `source`, `From`, 호출한 쪽의 `match` | 2 |
| [`examples/03_from_conversion.rs`](examples/03_from_conversion.rs) | ch2 `AgeError`의 `map_err` → `From`, 같은 원인 타입이 두 곳에서 올 때, 감싸기와 버리기 | 2 |
| [`examples/04_box_dyn_error.rs`](examples/04_box_dyn_error.rs) | `Box<dyn Error>`로의 변환 네 가지, `is`/`downcast_ref`/`downcast`, 스레드와 `+ Send + Sync` | 3 |
| [`examples/05_error_report.rs`](examples/05_error_report.rs) | `source` 체인 보고서, `main`의 디버그 출력과 비교, 맨 아래 원인 다운캐스트, 중복 보고 | 4 |
| [`examples/06_catch_unwind.rs`](examples/06_catch_unwind.rs) | `catch_unwind`의 `Ok`/`Err`, 페이로드 타입, `AssertUnwindSafe`와 깨진 불변식, `resume_unwind` | 5 |
| [`examples/07_panic_hook.rs`](examples/07_panic_hook.rs) | `set_hook`, `PanicHookInfo`, `take_hook`, 기본 훅 이어 붙이기, 다른 스레드의 패닉 | 5 |
| [`examples/08_backtrace.rs`](examples/08_backtrace.rs) | `capture`/`force_capture`/`disabled`의 상태, 에러에 담은 백트레이스 | 4 |
| [`compile_fail/`](compile_fail) | 에러 타입·`Box<dyn Error>`·`catch_unwind`에서 자주 만나는 컴파일 에러 5개 (4절) | 2, 3, 5 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 끝납니다. 06과 07은 일부러 패닉하지만 모두 안에서 붙잡습니다. 02는 `std::env::temp_dir()` 아래에 `p1_ch8_02_`로 시작하는 파일을 만들고 끝날 때 지웁니다.

### 01_error_trait — `Error` 트레이트의 생김새

```sh
cargo run -p p1-ch8-errors-panic --example 01_error_trait
```

```text
1. Error는 Debug + Display를 요구하고, 나머지는 기본 구현이 있습니다
    {}   → 입력이 비어 있습니다
    {:?} → EmptyInput
2. 표준 에러도 직접 만든 에러도 &dyn Error로 똑같이 다룹니다
    [직접 만든 에러]
      Display: 입력이 비어 있습니다
      Debug:   EmptyInput
      source:  없음
    [ParseIntError]
      Display: invalid digit found in string
      Debug:   ParseIntError { kind: InvalidDigit }
      source:  없음
    [ParseFloatError]
      Display: invalid float literal
      Debug:   ParseFloatError { kind: Invalid }
      source:  없음
    [io::Error]
      Display: entity not found
      Debug:   Kind(NotFound)
      source:  없음
3. Error를 구현한 ParseVersionError는 Box<dyn Error>로 ?를 쓸 수 있습니다
    1.2.3@4      → 1.2.3 버전 4개
    v2.0.1@10    → 2.0.1 버전 10개
    1.2.3        → 에러: '@'가 없습니다
    1.2@4        → 에러: 점으로 나눈 부분이 3개여야 하는데 2개입니다
    1.x.3@4      → 에러: 숫자가 아닌 부분이 있습니다: "x"
    1.2.3@-1     → 에러: invalid digit found in string
```

2번 구역의 `io::Error`는 `io::Error::from(ErrorKind::NotFound)`로 운영체제를 거치지 않고 만든 값이라 Part 0 ch9의 `No such file or directory (os error 2)`와 메시지가 다릅니다. 3번 구역의 세 가지 에러는 각각 `&str`, `ParseVersionError`, `ParseIntError`에서 `Box<dyn Error>`로 바뀌었습니다.

### 02_error_enum — 에러 열거형 설계

```sh
cargo run -p p1-ch8-errors-panic --example 02_error_enum
```

```text
1. read_count는 실패 종류를 배리언트로 구분해 돌려줍니다 ({:?})
    good    → Ok(12)
    bad     → Err(Parse(ParseIntError { kind: InvalidDigit }))
    big     → Err(TooMany { count: 500, max: 100 })
    missing → Err(Io(Os { code: 2, kind: NotFound, message: "No such file or directory" }))
2. Display는 이 단계의 실패만, source()는 그 원인을 알려 줍니다
    bad     → 개수 파일의 내용이 숫자가 아닙니다
              원인: invalid digit found in string
    big     → 개수가 최대값을 넘습니다 (500 > 100)
              원인: 없음
    missing → 개수 파일을 읽지 못했습니다
              원인: No such file or directory (os error 2)
3. 호출한 쪽은 match로 종류마다 다르게 대응합니다
  good
    → 12
  bad
    (에러: 개수 파일의 내용이 숫자가 아닙니다 / 원인: invalid digit found in string)
    → 0
  big
    (개수 500을 최대값 100에 맞춥니다)
    → 100
  missing
    (파일이 없어 0에서 시작합니다)
    → 0
```

`missing`의 `io::Error` 메시지는 macOS 기준입니다. 3번 구역은 같은 함수의 결과를 실패 종류마다 다르게 처리합니다. 파일이 없으면 0에서 시작하고, 너무 크면 최대값을 쓰고, 나머지는 알립니다.

### 03_from_conversion — `From`과 그 한계

```sh
cargo run -p p1-ch8-errors-panic --example 03_from_conversion
```

```text
1. map_err로 감싸던 코드가 From 구현 뒤에는 ?만으로 줄어듭니다
     42: ch2 판  42살
         From 판 42살
    abc: ch2 판  숫자가 아님 (invalid digit found in string)
         From 판 나이가 숫자가 아닙니다 (원인: invalid digit found in string)
    200: ch2 판  범위 밖 (200)
         From 판 나이는 0~150이어야 합니다 (받은 값: 200) (원인 없음)
2. 같은 원인 타입이 두 군데서 오면 From만으로는 어디서 실패했는지 잃습니다
    12x34: From만 Ok((12, 34))
           map_err Ok((12, 34))
    12xab: From만 Err("숫자가 아닌 값이 있습니다 (원인: invalid digit found in string)")
           map_err Err("세로 값이 숫자가 아닙니다 (원인: invalid digit found in string)")
    abx34: From만 Err("숫자가 아닌 값이 있습니다 (원인: invalid digit found in string)")
           map_err Err("가로 값이 숫자가 아닙니다 (원인: invalid digit found in string)")
     1234: From만 Err("'x'가 없습니다 (원인 없음)")
           map_err Err("'x'가 없습니다 (원인 없음)")
3. 감싸면 원인이 남고, 대신하면 원인이 사라집니다
     8080: 감싸기   Ok(8080)
           대신하기 Ok(8080)
     http: 감싸기   Err("포트 번호를 읽지 못했습니다 (원인: invalid digit found in string)")
           대신하기 Err("포트 번호를 읽지 못했습니다 (원인 없음)")
    70000: 감싸기   Err("포트 번호를 읽지 못했습니다 (원인: number too large to fit in target type)")
           대신하기 Err("포트 번호를 읽지 못했습니다 (원인 없음)")
```

1번 구역의 ch2 판은 `Display`가 없어서 메시지를 호출한 쪽(`describe` 함수)이 배리언트마다 만들었습니다. From 판은 메시지와 원인을 에러 타입이 스스로 제공합니다. 2·3번 구역은 비교하기 쉽도록 에러를 `one_line` 함수로 한 줄 문자열로 바꿔 `{:?}`로 출력했습니다.

### 04_box_dyn_error — `Box<dyn Error>`와 다운캐스트

```sh
cargo run -p p1-ch8-errors-panic --example 04_box_dyn_error
```

```text
1. 네 가지 출처의 에러가 모두 Box<dyn Error>가 됩니다
    kim=42    → Ok(42)
    lee       → '='가 없습니다  ["'='가 없습니다"]
    =7        → 이름이 비었습니다: "=7"  ["이름이 비었습니다: \"=7\""]
    park=12a  → invalid digit found in string  [ParseIntError { kind: InvalidDigit }]
    choi=250  → 할당량을 넘었습니다 (250/100)  [QuotaExceeded { used: 250, limit: 100 }]
2. downcast_ref와 is로 안에 든 타입을 확인합니다
    lee       is::<ParseIntError>() = false → 그 밖의 에러 → '='가 없습니다
    =7        is::<ParseIntError>() = false → 그 밖의 에러 → 이름이 비었습니다: "=7"
    park=12a  is::<ParseIntError>() = true  → 숫자 형식 오류 → 종류: InvalidDigit
    choi=250  is::<ParseIntError>() = false → 할당량 초과 → 150만큼 줄이세요
    downcast 성공: used = 250, limit = 100
    downcast 실패, 원래 에러를 돌려받음: '='가 없습니다
3. Box<dyn Error + Send + Sync>는 스레드 밖으로 돌려보낼 수 있습니다
    kim=42    → Ok(42)
    lee       → 그 밖의 에러 → '='가 없습니다
    =7        → 그 밖의 에러 → 이름이 비었습니다: "=7"
    park=12a  → 숫자 형식 오류 → 종류: InvalidDigit
    choi=250  → 할당량 초과 → 150만큼 줄이세요
    Send + Sync를 덜어 낸 Box: '='가 없습니다
```

1번 구역의 대괄호 안은 디버그 형식입니다. 3번 구역은 줄마다 스레드를 만들어 파싱하지만, 결과를 `join`으로 돌려받아 `main` 스레드에서 순서대로 출력하므로 실행할 때마다 같습니다.

### 05_error_report — `source` 체인 보고서

```sh
cargo run -p p1-ch8-errors-panic --example 05_error_report
```

```text
1. 성공하면 포트를, 실패하면 보고서를 출력합니다
  [good]
    포트 8080에서 시작합니다
  [bad]
    에러: 서버를 시작하지 못했습니다
      원인 1: 2번째 줄의 port 값을 읽지 못했습니다
        원인 2: invalid digit found in string
  [missing]
    에러: 서버를 시작하지 못했습니다
      원인 1: 설정에 port 항목이 없습니다
2. main이 이 에러를 Err로 돌려줬다면 이렇게 찍힙니다 (디버그 형식)
    Error: Startup(BadValue { line: 2, key: "port", source: ParseIntError { kind: InvalidDigit } })
3. Box<dyn Error>에 담겨도 체인은 그대로 따라갈 수 있습니다
    에러: 서버를 시작하지 못했습니다
      원인 1: 2번째 줄의 port 값을 읽지 못했습니다
        원인 2: invalid digit found in string
    맨 아래 원인은 ParseIntError, 종류: InvalidDigit
4. Display와 source()에 같은 원인을 담으면 보고서에 두 번 나옵니다
    에러: 포트 값을 읽지 못했습니다: invalid digit found in string
      원인 1: invalid digit found in string
```

2번 구역은 `main`이 실제로 `Err`를 돌려주게 하는 대신, 그때 출력될 모양(`Error: ` + 디버그 형식)을 직접 찍었습니다. 4번 구역이 2.3절에서 말한 "둘 중 하나만" 규칙을 어긴 결과입니다.

### 06_catch_unwind — 패닉 붙잡기

패닉 메시지는 `RUST_BACKTRACE`가 켜져 있으면 백트레이스까지 출력하므로, 값을 정해 실행합니다(`RUST_BACKTRACE=0`은 설정하지 않았을 때와 출력이 같습니다).

```sh
RUST_BACKTRACE=0 cargo run -p p1-ch8-errors-panic --example 06_catch_unwind
```

표준 출력과 표준 에러가 터미널에 섞여 나온 모습입니다. `thread 'main' ...`으로 시작하는 줄과 그 아래 메시지, `note:` 줄이 기본 패닉 훅이 표준 에러에 쓴 것이고, 각 메시지 앞의 빈 줄도 훅이 출력합니다.

```text
1. 패닉이 없으면 Ok(클로저의 반환값)입니다
    Ok(42)
2. 작업마다 경계를 두면 한 작업의 패닉이 다른 작업을 멈추지 않습니다
    [정상 나눗셈] 성공: 5

thread 'main' (7175852) panicked at part1-standard-lib/ch8-errors-panic/examples/06_catch_unwind.rs:33:5:
attempt to divide by zero
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    [0으로 나눔] 패닉을 붙잡음 → &str 페이로드: attempt to divide by zero

thread 'main' (7175852) panicked at part1-standard-lib/ch8-errors-panic/examples/06_catch_unwind.rs:71:13:
index out of bounds: the len is 3 but the index is 7
    [범위 밖 인덱스] 패닉을 붙잡음 → String 페이로드: index out of bounds: the len is 3 but the index is 7

thread 'main' (7175852) panicked at part1-standard-lib/ch8-errors-panic/examples/06_catch_unwind.rs:41:18:
알 수 없는 모드: turbo
    [직접 panic!] 패닉을 붙잡음 → String 페이로드: 알 수 없는 모드: turbo
    작업 4개 중 3개가 패닉했지만 main은 계속 실행됩니다
3. &mut를 캡처하려면 AssertUnwindSafe로 직접 약속해야 합니다

thread 'main' (7175852) panicked at part1-standard-lib/ch8-errors-panic/examples/06_catch_unwind.rs:47:5:
입금 후 잔액이 한도 600을 넘습니다
    결과: 패닉
    잔액 [300, 400], 합계 700
4. resume_unwind는 패닉 훅을 거치지 않고 패닉을 다시 일으킵니다

thread 'main' (7175852) panicked at part1-standard-lib/ch8-errors-panic/examples/06_catch_unwind.rs:121:53:
안쪽 작업 실패
    안쪽에서 붙잡아 기록만 하고 다시 던집니다: &str 페이로드: 안쪽 작업 실패
    바깥에서 다시 붙잡음: &str 페이로드: 안쪽 작업 실패
```

- 괄호 안의 스레드 ID(`7175852`)는 실행할 때마다 달라집니다.
- `2>/dev/null`을 붙여 표준 에러를 버리면 표준 출력의 줄만 남습니다. 예제 파일의 `// 출력:` 주석은 표준 출력만 적었습니다.
- 4번 구역의 패닉 메시지는 한 번뿐입니다. `resume_unwind`로 다시 일으킨 패닉은 훅을 거치지 않았습니다.

### 07_panic_hook — 패닉 훅 바꾸기

```sh
RUST_BACKTRACE=0 cargo run -p p1-ch8-errors-panic --example 07_panic_hook
```

```text
1. set_hook으로 패닉 메시지를 바꿉니다
[내 훅] 스레드 'main', 위치 part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:48:14, 메시지: index out of bounds: the len is 0 but the index is 0
2. 페이로드가 문자열이 아니면 payload()를 다운캐스트합니다
[내 훅] 스레드 'main', 위치 part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:56:36, 메시지: (i32 페이로드: 404)
3. take_hook은 훅을 떼어 내고 기본 훅을 다시 등록합니다

thread 'main' (7178719) panicked at part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:62:36:
기본 훅으로 돌아왔습니다
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
4. 기본 훅을 새 훅 안에서 부르면 두 출력을 이어 붙일 수 있습니다
[기록] 스레드 'main', 위치 part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:77:36, 메시지: 기록과 기본 출력 둘 다

thread 'main' (7178719) panicked at part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:77:36:
기록과 기본 출력 둘 다
5. 훅은 프로세스 전체에 하나라서 다른 스레드의 패닉에도 실행됩니다
[기록] 스레드 'worker', 위치 part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:88:28, 메시지: 작업자 스레드에서 실패

thread 'worker' (7178720) panicked at part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:88:28:
작업자 스레드에서 실패
    join 결과가 Err인가: true
6. 기본 훅으로 되돌렸습니다
```

- `[내 훅]`과 `[기록]` 줄은 훅이 `eprintln!`으로 표준 에러에 쓴 것입니다. 1·2번 구역에는 기본 훅의 세 줄이 나오지 않습니다.
- `note: run with ...` 줄은 기본 훅이 처음 출력하는 3번 구역에서만 나옵니다.
- 위치의 파일 경로는 cargo가 저장소 루트에서 컴파일하기 때문에 루트 기준입니다. 스레드 ID는 실행할 때마다 달라집니다.

### 08_backtrace — 백트레이스를 값으로

환경 변수에 따라 결과가 달라지므로 값을 정해 두 번 실행합니다. `RUST_LIB_BACKTRACE`를 정하면 `RUST_BACKTRACE`가 무엇이든 `capture()`는 그 값을 따릅니다. 1번 구역의 `RUST_BACKTRACE` 줄은 독자의 환경에 따라 다르게 나올 수 있습니다.

```sh
RUST_LIB_BACKTRACE=0 cargo run -p p1-ch8-errors-panic --example 08_backtrace
RUST_LIB_BACKTRACE=1 cargo run -p p1-ch8-errors-panic --example 08_backtrace
```

`RUST_LIB_BACKTRACE=1`로 실행한 결과입니다.

```text
1. 환경 변수
    RUST_LIB_BACKTRACE = 1
    RUST_BACKTRACE     = (설정 안 됨)
2. 세 가지 생성 함수의 상태
    capture()       → Captured
    force_capture() → Captured
    disabled()      → Disabled
3. 에러에 담아 둔 백트레이스
    에러: 한도 값을 읽지 못했습니다
      원인: invalid digit found in string
    백트레이스가 기록되어 있습니다. 이 예제의 함수만 골라 보면:
      <08_backtrace::LimitError as core::convert::From<core::num::error::ParseIntError>>::from
      <core::result::Result<u32, 08_backtrace::LimitError> as core::ops::try_trait::FromResidual<core::result::Result<!, core::num::error::ParseIntError>>>::from_residual
      08_backtrace::parse_limit
      08_backtrace::load_settings
      08_backtrace::main
```

`RUST_LIB_BACKTRACE=0`으로 실행하면 2번 구역의 `capture()`가 `Disabled`가 되고, 3번 구역의 마지막 줄들이 `백트레이스가 꺼져 있습니다 (RUST_LIB_BACKTRACE=1로 실행해 보세요)` 한 줄로 바뀝니다.

백트레이스 전체는 프레임 번호와 표준 라이브러리 경로가 환경마다 다르므로, 예제는 `08_backtrace::`가 들어간 함수 이름만 골라 출력합니다. 위에서 아래로 "가장 안쪽 → 바깥쪽" 순서입니다. `main` → `load_settings` → `parse_limit`에서 `?`가 `from_residual`을 거쳐 `From::from`을 불렀고, 그 안에서 `capture()`가 실행되었다는 것이 그대로 보입니다. 이 다섯 줄은 macOS의 디버그 빌드에서 확인한 것입니다. 같은 명령에 `--release`를 붙여 최적화 빌드로 실행하면 함수가 다른 함수 안으로 합쳐져(인라인) `08_backtrace::main` 한 줄만 남았습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch8-errors-panic
rustc --edition 2024 compile_fail/e0119_duplicate_from.rs --out-dir /tmp/rust-p1-ch8
```

`note: required by a bound in ...` 아래에 표준 라이브러리 소스 위치(`/rustc/0ef1bc38.../library/...`)가 나오는 경우가 있습니다. 경로의 해시 값은 툴체인마다 다르며, 아래 인용에서는 그 줄을 생략했습니다.

### 4.1 같은 원인 타입에서 오는 `From`을 두 번 구현 — E0119

파일: [`compile_fail/e0119_duplicate_from.rs`](compile_fail/e0119_duplicate_from.rs)

```text
error[E0119]: conflicting implementations of trait `From<ParseIntError>` for type `SizeError`
  --> compile_fail/e0119_duplicate_from.rs:38:1
   |
32 | impl From<ParseIntError> for SizeError {
   | -------------------------------------- first implementation here
...
38 | impl From<ParseIntError> for SizeError {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `SizeError`
```

- **읽는 법**: `From<ParseIntError>`를 `SizeError`에 대해 구현한 것이 두 개(32번 줄과 38번 줄) 있어 충돌한다는 뜻입니다.
- **원인**: 가로와 세로를 구분하려고 `BadWidth`와 `BadHeight` 배리언트를 만들고, 각각으로 가는 `From`을 구현했습니다. 하지만 한 타입에 같은 트레이트의 구현은 하나만 있을 수 있습니다. 둘을 허락하더라도 `?`는 에러의 타입만 보므로 어느 쪽을 고를지 알 수 없습니다.
- **해결**: `From`을 지우고 자리마다 `map_err`로 배리언트를 고릅니다. `w.parse().map_err(SizeError::BadWidth)?`, `h.parse().map_err(SizeError::BadHeight)?`. 예제 03의 `parse_size_b`가 같은 방법입니다(2.4절).

### 4.2 모든 에러를 받는 포괄 `From` 구현 — E0119

파일: [`compile_fail/e0119_blanket_from_error.rs`](compile_fail/e0119_blanket_from_error.rs)

```text
error[E0119]: conflicting implementations of trait `From<AppError>` for type `AppError`
  --> compile_fail/e0119_blanket_from_error.rs:26:1
   |
26 | impl<E: Error> From<E> for AppError {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: conflicting implementation in crate `core`:
           - impl<T> From<T> for T;
```

- **읽는 법**: 문제가 되는 것은 `From<AppError> for AppError`, 즉 `E`가 `AppError` 자신인 경우입니다. `note:`는 그 경우를 이미 `core`의 `impl<T> From<T> for T`(모든 타입은 자기 자신으로 바뀐다)가 맡고 있다고 알려 줍니다.
- **원인**: `From`을 원인 타입마다 쓰기 귀찮아서 "`Error`를 구현한 모든 `E`"에 대한 포괄 구현을 만들었습니다. 그런데 `AppError`도 `Error`를 구현하므로 `E = AppError`가 포함되고, 두 구현이 겹칩니다. Part 0 ch10에서 배운 일관성(coherence) 규칙이 막는 상황입니다.
- **해결**: 원인 타입마다 `From`을 따로 구현합니다. 정말 모든 에러를 받아야 한다면 그것이 `Box<dyn Error>`의 역할입니다. 참고로 `AppError`에서 `impl Error`를 지우면 겹치지 않아 컴파일되지만, 그러면 `AppError`를 다른 에러의 원인이나 `Box<dyn Error>`에 담을 수 없게 됩니다.

### 4.3 `&mut`를 캡처한 클로저를 `catch_unwind`에 — E0277

파일: [`compile_fail/e0277_catch_unwind_mut_ref.rs`](compile_fail/e0277_catch_unwind_mut_ref.rs)

```text
error[E0277]: the type `&mut [i32; 2]` may not be safely transferred across an unwind boundary
  --> compile_fail/e0277_catch_unwind_mut_ref.rs:14:38
   |
14 |       let result = panic::catch_unwind(|| {
   |                    ------------------- ^-
   |                    |                   |
   |  __________________|___________________within this `{closure@compile_fail/e0277_catch_unwind_mut_ref.rs:14:38: 14:40}`
   | |                  |
   | |                  required by a bound introduced by this call
15 | |         accounts[0] -= 300;
16 | |         accounts[1] += 300;
17 | |     });
   | |_____^ `&mut [i32; 2]` may not be safely transferred across an unwind boundary
   |
   = help: within `{closure@compile_fail/e0277_catch_unwind_mut_ref.rs:14:38: 14:40}`, the trait `UnwindSafe` is not implemented for `&mut [i32; 2]`
   = note: `UnwindSafe` is implemented for `&[i32; 2]`, but not for `&mut [i32; 2]`
note: required because `accounts` is used within this closure
```

- **읽는 법**: "되감기 경계(unwind boundary)를 안전하게 넘을 수 없는 타입"이 클로저 안에 있다는 뜻입니다. `help:`가 범인을 `&mut [i32; 2]`로, 마지막 `note:`가 그 출처를 `accounts`로 짚어 줍니다. `&`였다면 괜찮다는 `note:`도 있습니다.
- **원인**: 클로저가 `accounts`를 수정하므로 `&mut`로 캡처했습니다. 두 줄 사이에서 패닉하면 반쯤 바뀐 `accounts`를 `catch_unwind` 뒤에서 보게 됩니다(2.9절).
- **해결**: 패닉 뒤의 상태를 직접 확인하겠다면 `panic::catch_unwind(AssertUnwindSafe(|| { ... }))`로 감쌉니다. 예제 06의 3번 구역이 이렇게 고친 코드이고, 실제로 합계가 700이 된 상태를 확인합니다. 상태를 확인할 방법이 없다면 `catch_unwind`를 쓰지 않는 것이 맞습니다.

### 4.4 `Box<dyn Error>`를 돌려주는 클로저를 `thread::spawn`에 — E0277

파일: [`compile_fail/e0277_box_dyn_error_not_send.rs`](compile_fail/e0277_box_dyn_error_not_send.rs)

```text
error[E0277]: `dyn std::error::Error` cannot be sent between threads safely
  --> compile_fail/e0277_box_dyn_error_not_send.rs:19:18
   |
19 |     let handle = thread::spawn(|| parse_usage("kim=42"));
   |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `dyn std::error::Error` cannot be sent between threads safely
   |
   = help: the trait `Send` is not implemented for `dyn std::error::Error`
   = note: required for `std::ptr::Unique<dyn std::error::Error>` to implement `Send`
note: required because it appears within the type `Box<dyn std::error::Error>`
note: required because it appears within the type `Result<u32, Box<dyn std::error::Error>>`
note: required by a bound in `spawn`
```

- **읽는 법**: `note:`를 아래에서 위로 읽으면 사슬이 보입니다. `spawn`은 클로저의 반환값이 `Send`이기를 요구하고 → 반환값 `Result<u32, Box<dyn Error>>` 안에 → `Box<dyn Error>`가 있고 → 그 안의 `dyn Error`가 `Send`가 아닙니다. (`Unique`는 `Box`의 내부 구현에 쓰이는 타입이라 신경 쓰지 않아도 됩니다.)
- **원인**: `dyn Error`는 "`Error`를 구현한 아무 타입"이므로 `Send`가 아닌 에러도 들어올 수 있습니다. Part 0 ch15의 `Rc`가 `Send`가 아니어서 막혔던 것과 같은 원리입니다.
- **해결**: 에러 타입을 `Box<dyn Error + Send + Sync>`로 바꿉니다. 함수 본문은 그대로 둬도 `?`가 `&str`과 `ParseIntError`를 새 타입으로 바꿔 줍니다. 예제 04의 `parse_usage_sendable`입니다.

### 4.5 `Box<dyn Error>`를 `E: Error`인 제네릭 함수에 — E0277

파일: [`compile_fail/e0277_box_dyn_error_is_not_error.rs`](compile_fail/e0277_box_dyn_error_is_not_error.rs)

```text
error[E0277]: the size for values of type `dyn std::error::Error` cannot be known at compilation time
  --> compile_fail/e0277_box_dyn_error_is_not_error.rs:28:16
   |
28 |         report(&e);
   |         ------ ^^ doesn't have a size known at compile-time
   |         |
   |         required by a bound introduced by this call
   |
   = help: the trait `Sized` is not implemented for `dyn std::error::Error`
help: the trait `std::error::Error` is conditionally implemented for `Box<E, A>`
   = note: unsatisfied requirement introduced here: `dyn std::error::Error: Sized`
   = note: required for `Box<dyn std::error::Error>` to implement `std::error::Error`
note: required by a bound in `report`
  --> compile_fail/e0277_box_dyn_error_is_not_error.rs:13:14
   |
13 | fn report<E: Error>(err: &E) {
   |              ^^^^^ required by this bound in `report`
```

- **읽는 법**: 첫 줄만 보면 크기 이야기라 엉뚱해 보입니다. 아래의 `note:`가 핵심입니다. `report`는 `E: Error`를 요구하는데, `Box<dyn Error>`가 `Error`를 구현하려면 `Box<E>`의 `E`가 크기가 정해진(`Sized`) 타입이어야 하고, `dyn Error`는 그렇지 않습니다.
- **원인**: 표준 라이브러리는 `impl<E: Error> Error for Box<E>`만 제공하므로 `Box<dyn Error>` 자신은 `Error`가 아닙니다. 만약 `Box<dyn Error>`가 `Error`였다면 2.5절의 `impl<E: Error> From<E> for Box<dyn Error>`가 `E = Box<dyn Error>`일 때 `impl<T> From<T> for T`와 겹칩니다(4.2절과 같은 충돌).
- **해결**: 함수가 트레이트 객체를 받게 하고(`fn report(err: &dyn Error)`) `report(e.as_ref())`로 상자 안의 에러를 빌려 넘깁니다. 제네릭을 유지하려면 `fn report<E: Error + ?Sized>(err: &E)`로 크기 조건을 풀고 `report(&*e)`로 넘깁니다. 예제 05의 `report`가 앞의 방법입니다.

### 4.6 그 밖의 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| `Display` 없이 `impl Error for NoDisplay {}` | E0277 `` `NoDisplay` doesn't implement `std::fmt::Display` `` | `impl fmt::Display for NoDisplay`를 먼저 씁니다. 슈퍼트레이트를 빠뜨린 Part 0 ch19의 `OutlinePrint`와 같은 에러입니다. |
| `Error`를 구현하지 않은 `VErr`에 `?`를 써서 `Box<dyn Error>`로 | E0277 `` `?` couldn't convert the error: `VErr: std::error::Error` is not satisfied `` | `impl Error for VErr {}`를 추가합니다(2.2절). |
| `Box<dyn Error + Send + Sync>`를 돌려주는 함수의 결과에 `?`를 써서 `Box<dyn Error>`로 | E0277 `` `?` couldn't convert the error: `dyn std::error::Error + Send + Sync: Sized` is not satisfied `` | 대입은 되지만 `?`의 `From` 변환은 없습니다(4.5절과 같은 이유). `.map_err(\|e\| e as Box<dyn Error>)?`로 바꾸거나 양쪽을 같은 타입으로 맞춥니다. |
| 페이로드를 `payload_text(&payload)`처럼 `&Box<..>`로 넘겨 다운캐스트 | 컴파일은 되지만 `downcast_ref`가 늘 `None` | `Box` 자체가 `Any`로 취급된 것입니다. `&*payload`로 상자 안의 값을 넘깁니다(2.9절). |
| `Display`에 원인 메시지를 넣고 `source()`로도 내줌 | 보고서에 같은 말이 두 번 나옴 | 둘 중 하나만 합니다(2.3절, 예제 05의 4번 구역). |
| `panic = "abort"`로 빌드한 프로그램에서 `catch_unwind`에 기대기 | 첫 패닉에서 프로세스가 끝남 (종료 코드 134, macOS zsh) | 일반 에러는 `Result`로 다룹니다(2.9절). |

## 5. 연습 문제

### 연습 1. 섭씨 온도 파싱

`"23.5C"`, `"-40C"` 같은 문자열을 읽는 `struct Celsius(f64)`에 `FromStr`을 구현하세요. 에러 타입 `CelsiusError`는 다음 배리언트를 갖고 `Display`와 `Error`를 구현합니다. 함수 본문에서 `map_err`를 쓰지 말고 `?`로 변환되게 하세요.

- `MissingUnit`: 끝에 `C`가 없음
- `BadNumber(ParseFloatError)`: 숫자 부분을 읽지 못함 (원인으로 내줍니다)
- `BelowAbsoluteZero(f64)`: −273.15보다 낮음

```text
"23.5C" → Ok(Celsius(23.5))
"23.5"  → 단위 C가 없습니다 (원인 없음)
"abcC"  → 온도 값을 읽지 못했습니다 (원인: invalid float literal)
"-300C" → 절대 영도(-273.15)보다 낮습니다: -300 (원인 없음)
```

<details>
<summary>힌트</summary>

- 단위는 `s.trim().strip_suffix('C').ok_or(CelsiusError::MissingUnit)?`로 떼어 냅니다(Part 1 ch3의 `strip_prefix`와 같은 모양).
- `impl From<ParseFloatError> for CelsiusError`를 구현하면 `let value: f64 = number.parse()?;`가 됩니다. `ParseFloatError`는 `std::num::ParseFloatError`입니다.
- `Display`에는 원인 메시지를 넣지 말고 `source()`에서 `BadNumber(e) => Some(e)`를 돌려줍니다. 에러 쪽 출력은 예제 03의 `one_line` 함수를 가져다 쓰면 됩니다.
- `-300.0`은 `{}`로 출력하면 `-300`으로 나옵니다.

</details>

### 연습 2. 배리언트를 추가하면 생기는 일

[`02_error_enum.rs`](examples/02_error_enum.rs)의 `count_or_default`에서 마지막 `Err(e) =>` 갈래를 `Err(ReadCountError::Parse(e)) =>`와 `Err(ReadCountError::Io(e)) =>` 두 갈래로 바꿔, 모든 배리언트를 이름으로 처리하게 하세요. 그다음 `ReadCountError`에 "파일이 비어 있음"을 뜻하는 `Empty` 배리언트를 추가하고 `read_count`가 빈 파일에서 `Empty`를 돌려주게 하세요. 컴파일러가 어디에서 무엇을 알려 주는지 확인하고, `Box<dyn Error>`였다면 같은 변경을 어떻게 알아챌 수 있었을지 생각해 보세요.

<details>
<summary>힌트</summary>

- 빈 파일인지는 `parse` 전에 `text.trim().is_empty()`로 확인하고 `return Err(ReadCountError::Empty);`를 씁니다. 그냥 두면 `ParseIntError { kind: Empty }`가 되어 `Parse` 배리언트로 갑니다.
- 배리언트를 추가하면 `Display`의 `match`와 `source()`의 `match`가 E0004 ``non-exhaustive patterns: `&ReadCountError::Empty` not covered``로, `count_or_default`의 `match`가 ``non-exhaustive patterns: `Err(ReadCountError::Empty)` not covered``로 멈춥니다. 처리하지 않은 곳을 컴파일러가 빠짐없이 알려 준다는 것이 열거형의 장점입니다(2.6절).
- `Box<dyn Error>`였다면 다운캐스트의 `else` 갈래로 조용히 흘러가므로 실행해 보기 전에는 알 수 없습니다.

</details>

### 연습 3. 스레드 결과 집계

[`04_box_dyn_error.rs`](examples/04_box_dyn_error.rs)의 `parse_usage_sendable`을 써서 줄 목록 `["kim=42", "lee", "park=12a", "choi=250", "jung=7", "han=x"]`을 줄마다 스레드에서 파싱하세요. `main` 스레드에서 결과를 모아 성공한 사용량의 합계와, 실패를 `ParseIntError`·`QuotaExceeded`·그 밖으로 나눈 개수를 출력합니다.

```text
합계: 49
숫자 형식 오류 2개, 할당량 초과 1개, 그 밖 1개
```

<details>
<summary>힌트</summary>

- 스레드 만들기와 `join`은 예제 04의 3번 구역을 그대로 씁니다.
- `Err(e)`에서 `e.is::<ParseIntError>()`와 `e.is::<QuotaExceeded>()`로 나눠 세면 됩니다. `e`의 타입은 `Box<dyn Error + Send + Sync>`이고, `is`는 `dyn Error + Send + Sync`에도 있습니다(2.5절의 표).
- 출력 순서를 고정하려면 스레드 안에서 출력하지 말고 결과를 돌려받아 `main`에서 처리합니다.

</details>

### 연습 4. 패닉을 조용히 모으는 훅

작업 클로저 목록을 `catch_unwind`로 실행하되, 표준 에러에 아무것도 출력하지 않고 패닉 메시지를 목록에 모았다가 마지막에 한 번에 출력하세요. 다 쓴 뒤에는 기본 훅으로 되돌립니다.

```text
작업 3개 중 2개 실패
  - 작업 0 실패
  - 작업 2 실패
```

<details>
<summary>힌트</summary>

- 모을 곳은 `Arc<Mutex<Vec<String>>>`입니다(Part 0 ch15). `Arc::clone`한 것을 `move` 클로저로 훅에 넣습니다. 훅은 `Send + Sync + 'static`이어야 하므로 `Rc<RefCell<..>>`는 쓸 수 없습니다.
- 훅 안에서는 `info.payload_as_str().unwrap_or("문자열이 아닌 페이로드").to_string()`을 `lock().unwrap().push(..)`합니다. 기본 훅을 부르지 않으므로 표준 에러에는 아무것도 나오지 않습니다.
- 작업은 `for i in 0..3 { let _ = panic::catch_unwind(move || if i % 2 == 0 { panic!("작업 {i} 실패") }); }`처럼 만들 수 있습니다.
- 끝에서 `let _ = panic::take_hook();`으로 기본 훅을 되돌린 뒤 목록을 출력합니다.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::error`](https://doc.rust-lang.org/std/error/index.html): 2.1절의 두 체계와 역할 목록, `expect` 메시지를 "무엇이 참이어야 했는가"로 쓰라는 권고("expect as precondition")가 있습니다.
  - [`Error`](https://doc.rust-lang.org/std/error/trait.Error.html): 메시지 관례, `source`를 쓰는 때와 "`source` 또는 `Display` 중 하나만" 규칙, 쓰지 않는 메서드, `dyn Error`의 다운캐스트 메서드, `Box<dyn Error>`로의 `From` 구현 목록(Trait Implementations 절).
  - [`std::panic`](https://doc.rust-lang.org/std/panic/index.html): 패닉 관련 함수와 타입 목록. `update_hook`, `always_abort` 등은 nightly 전용 실험 API입니다.
  - [`catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html): 일반 try/catch로 쓰지 말라는 권고, abort 전략에서는 붙잡지 못한다는 Notes 절.
  - [`set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html), [`take_hook`](https://doc.rust-lang.org/std/panic/fn.take_hook.html), [`PanicHookInfo`](https://doc.rust-lang.org/std/panic/struct.PanicHookInfo.html): 2.8절의 표와 예제.
  - [`resume_unwind`](https://doc.rust-lang.org/std/panic/fn.resume_unwind.html): 훅을 부르지 않는 패닉.
  - [`UnwindSafe`](https://doc.rust-lang.org/std/panic/trait.UnwindSafe.html), [`AssertUnwindSafe`](https://doc.rust-lang.org/std/panic/struct.AssertUnwindSafe.html): 되감기 안전성이 무엇인지, 누가 구현하는지, 캡처를 하나씩 감싸는 방법.
  - [`std::backtrace`](https://doc.rust-lang.org/std/backtrace/index.html), [`Backtrace`](https://doc.rust-lang.org/std/backtrace/struct.Backtrace.html): 환경 변수 규칙(Environment Variables 절)과 정확도·플랫폼 지원에 대한 설명.
- Rust By Example: [Error handling](https://doc.rust-lang.org/rust-by-example/error.html) 장의 Multiple error types 절
  - [Defining an error type](https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html): 좋은 에러 타입의 조건(같은 타입으로 여러 에러 표현, 좋은 메시지, 비교하기 쉬움, 정보를 담음, 다른 에러와 잘 어울림). 예제의 `DoubleError`는 `Error`를 구현하지 않았는데, 이 챕터를 배운 뒤라면 한 줄을 더하고 싶어질 것입니다.
  - [`Box`ing errors](https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/boxing_errors.html), [Other uses of `?`](https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/reenter_question_mark.html): `map_err(From::from)`이 `?` 하나로 줄어드는 과정. 2.4절과 같은 이야기를 `Box<dyn Error>` 쪽에서 봅니다.
  - [Wrapping errors](https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/wrap_error.html): 예제 02와 같은 구조(`Parse(ParseIntError)` 배리언트, `source`, `From`)와 "Caused by:" 출력. 끝부분에 이 반복을 줄여 주는 크레이트가 있다는 언급이 있습니다(Part 7 `ch2-implementation-errors`).
- Rustlings: [`13_error_handling`](https://github.com/rust-lang/rustlings/tree/main/exercises/13_error_handling)
  - `errors5.rs`: `main`의 반환 타입 `Result<(), Box<dyn ???>>` 채우기. 2.5절의 `impl<E: Error> From<E>` 덕분에 `ParseIntError`와 직접 만든 `CreationError`가 모두 `?`로 들어간다는 점을 확인해 보세요.
  - `errors6.rs`: `ParsePosNonzeroError`의 `parse`에서 `unwrap` 대신 에러를 돌려주기. 머리 주석이 2.6절의 선택 기준을 설명합니다. `map_err(ParsePosNonzeroError::ParseInt)?`로 풀 수도 있고, `impl From<ParseIntError> for ParsePosNonzeroError`를 추가해 `?`만으로 풀 수도 있습니다(2.4절). 두 방법을 모두 시도해 보세요.
- 에러 설명: `rustc --explain E0119`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
