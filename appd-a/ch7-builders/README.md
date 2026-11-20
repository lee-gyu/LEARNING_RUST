# ch7. 필드가 많은 타입은 빌더로 만들기

[Part 0 ch5](../../part0-getting-started/ch5-structs/README.md)에서 구조체 리터럴과 구조체 갱신 문법을, [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)에서 `Default`를 구현하는 방법을, [Part 1 ch10](../../part1-standard-lib/ch10-env-process-time/README.md)에서 표준 라이브러리의 빌더인 `Command`를 써 보았습니다. 이 챕터는 그 문법을 다시 설명하지 않고, 필드가 많은 타입을 **어떤 방법으로 만들게 할지** 고르는 판단을 다룹니다. 필드가 늘어날수록 위치 인자 생성자에서 구조체 리터럴로, `Default`와 구조체 갱신 문법으로, 빌더(builder)로 옮겨 가게 됩니다. 각 단계에서 무엇이 불편해져서 다음 단계로 넘어가는지, 그리고 빌더의 두 형태 중 무엇을 고르는지가 이 챕터의 중심입니다.

- 원제: [Item 7: Use builders for complex types](https://effective-rust.com/builders.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch7-builders`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

모든 예제는 HTTP 요청 하나를 설명하는 값 `Request`를 만듭니다. 요청을 실제로 보내지는 않고 내용만 출력합니다. 예제 05는 챕터 `Cargo.toml`의 `[dev-dependencies]`에 넣은 `derive_builder` 0.20.2를 쓰며, 처음 빌드할 때 crates.io에서 내려받습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 필드가 많은 타입에 위치 인자 생성자를 쓰면 안 되는 이유를 설명하고, 구조체 리터럴이 필드가 늘어날 때 치르는 비용을 설명할 수 있다.
2. `Default`와 구조체 갱신 문법이 안전하게 동작하는 조건 세 가지를 설명하고, 필수 값을 따로 받는 옵션 구조체를 작성할 수 있다.
3. 필수 값은 생성자로, 선택 값은 메서드로 받고, 필드 사이의 규칙을 `build()`에서 검사해 `Result`로 알리는 빌더를 작성할 수 있다.
4. 소비형 빌더와 `&mut` 빌더가 각각 어떤 쓰임새에서 불편한지 실제 코드로 설명하고, 표준 라이브러리의 빌더를 근거로 둘 중 하나를 고를 수 있다.
5. `derive_builder`로 빌더를 만들고, 생성된 빌더의 형태와 손으로 쓴 빌더와의 차이를 설명할 수 있다.

## 2. 핵심 개념

### 2.1 지도: 값을 만드는 네 가지 방법

| 방법 | 부르는 쪽 코드 | 필드 이름이 보이나 | 생략할 수 있나 | 규칙을 검사할 곳 | 예제 |
| --- | --- | --- | --- | --- | --- |
| 위치 인자 생성자 | `Request::new(Get, url, vec![], None, d1, d2, 5, true)` | 아니요 | 아니요 | `new` 안 | 01 |
| 구조체 리터럴 | `Request { method: Get, url, /* 여덟 필드 모두 */ }` | 예 | 아니요 | 없음 | 01 |
| `Default` + `..` | `Options { body: Some(..), ..Default::default() }` | 예 | 예 | 없음 | 02 |
| 빌더 | `Request::builder(Get, url).timeout(d).build()?` | 예(메서드 이름) | 예 | `build()` 안 | 03~05 |

아래로 갈수록 부르는 쪽은 편해지고, 만드는 쪽의 코드는 늘어납니다. 빌더는 타입을 하나 더 정의하고 선택 값마다 메서드(세터)를 하나씩 써야 합니다. 그래서 질문은 "어느 방법이 좋은가"가 아니라 "언제 다음 칸으로 넘어갈 만큼 불편해지는가"입니다.

빌더에는 두 형태가 있습니다. 세터가 `self`를 받아 `Self`를 돌려주는 **소비형 빌더**와, `&mut self`를 받아 `&mut Self`를 돌려주는 **`&mut` 빌더**입니다. 2.4절은 소비형으로 빌더의 설계를 보이고, 2.5절에서 두 형태를 비교합니다.

### 2.2 지침 1: 필드가 많은 타입을 위치 인자 생성자로 만들지 않는다

**문제 상황.** 요청에는 메서드, 주소, 헤더, 본문, 전체 제한 시간, 연결 제한 시간, 리다이렉트 횟수, gzip 사용 여부까지 여덟 필드가 있습니다. 다른 언어에서 하던 대로 필드마다 인자를 하나씩 받는 생성자를 만들고 싶어집니다.

**비권장 코드.** [`01_positional_vs_literal.rs`](examples/01_positional_vs_literal.rs)의 1번 구역입니다.

```rust
fn new(method: Method, url: &str, headers: Vec<String>, body: Option<String>,
       timeout: Duration, connect_timeout: Duration, max_redirects: u8, gzip: bool) -> Request

let swapped = Request::new(
    Method::Get, url, Vec::new(), None,
    Duration::from_secs(3),  // 연결 제한 시간을 넣으려던 자리를 헷갈림
    Duration::from_secs(30),
    5, true,
);
```

의도는 "전체 30s, 연결 3s"였는데 결과는 `전체 3s, 연결 30s`입니다. 다섯째와 여섯째 인자가 모두 `Duration`이라서 컴파일러가 알아챌 방법이 없습니다. 위치 인자는 부르는 쪽 코드에서 필드 이름을 지웁니다. [ch1](../ch1-express-data-structures/README.md) 2.2절에서 본 "같은 타입이 나란하면 이름을 붙인다"는 문제가 생성자 인자에서 그대로 생깁니다. Clippy도 이 모양을 경고합니다(`clippy::too_many_arguments`, 기본 warn). 이 툴체인에서는 인자가 8개일 때 `this function has too many arguments (8/7)`이 나왔습니다. 예제는 비권장 코드를 보여 주려고 `#[expect]`로 이 경고를 표시해 두었습니다.

**권장 코드.** 구조체 리터럴은 `timeout: Duration::from_secs(30)`처럼 이름으로 짝을 맞추므로 순서를 바꿔 적어도 결과가 같고, 어느 값이 어느 필드로 가는지 코드에 보입니다(2번 구역).

대신 비용이 있습니다. 3번 구역은 같은 서버에 보내는 두 번째 요청을 리터럴로 만드는데, 달라진 것은 메서드·헤더·본문뿐인데도 나머지를 다시 적어야 합니다.

```text
    두 리터럴이 같은 값을 되풀이한 필드 5개: url, timeout, connect_timeout, max_redirects, gzip
```

필드가 하나 늘면 이런 리터럴이 모두 E0063(``missing field `...` in initializer``)으로 컴파일되지 않습니다(Part 0 ch5 2.1절). 크레이트 안에서만 쓰는 타입이라면 컴파일러가 고칠 곳을 빠짐없이 알려 주는 셈이니 오히려 장점이지만, 공개 라이브러리라면 사용자의 코드를 깨는 변경이 됩니다([ch21](../ch21-semver/README.md)).

**따르지 않아도 되는 경우.** 인자가 두세 개이고 타입이 서로 달라서 바꿔 넣으면 컴파일되지 않는다면 위치 인자 생성자가 가장 간결합니다. 표준 라이브러리의 `Duration::new(secs: u64, nanos: u32)`나 예제 02의 `Request::new(method, url, options)`가 그런 경우입니다.

### 2.3 지침 2: `Default` + `..`는 모든 필드에 뜻이 있는 기본값이 있을 때 쓴다

**문제 상황.** 2.2절의 반복을 줄이려고 `#[derive(Default)]`를 붙이고 `..Default::default()`로 나머지 필드를 채웁니다.

**비권장 코드.** 필수 값인 주소까지 들어 있는 타입에 `Default`를 derive합니다([`02_default_update.rs`](examples/02_default_update.rs)의 1번 구역).

```rust
#[derive(Default)]
struct DefaultedRequest { method: Method, url: String, /* 나머지 여섯 필드 */ }

let forgot_url = DefaultedRequest {
    method: Method::Post,
    body: Some(json.to_string()),
    ..Default::default() // 주소를 깜빡함
};
```

```text
    POST (URL 없음) | 헤더 0 | 본문 17바이트 | 전체 0ns, 연결 0ns | 리다이렉트 0 | gzip 끔
```

문제가 둘입니다. 첫째, 구조체 리터럴이었다면 E0063으로 잡혔을 "필수 값 빠뜨리기"를 `..`가 빈 문자열로 덮었습니다. 둘째, derive가 고르는 기본값은 **타입의** 기본값(0, `false`, 빈 값)이지 **이 필드에** 맞는 값이 아닙니다. 제한 시간 0은 모든 요청을 바로 실패시킵니다.

**권장 코드.** 필수 값은 생성자 인자로 받고, 뜻이 있는 기본값을 가진 필드만 옵션 구조체로 모은 뒤 `Default`를 손으로 구현합니다(2번 구역).

```rust
struct Options { headers: Vec<String>, body: Option<String>, timeout: Duration,
                 connect_timeout: Duration, max_redirects: u8, gzip: bool }

impl Default for Options {
    fn default() -> Self {
        Options { headers: Vec::new(), body: None, timeout: Duration::from_secs(30),
                  connect_timeout: Duration::from_secs(5), max_redirects: 5, gzip: true }
    }
}

let create = Request::new(Method::Post, url, Options {
    headers: vec!["Content-Type: application/json".to_string()],
    body: Some(json.to_string()),
    ..Default::default()
});
```

부르는 쪽은 기본값과 다른 필드만 적고, 기본값은 `impl Default` 한곳에 모입니다. 주소를 빠뜨리면 `Request::new`의 인자가 모자라 컴파일되지 않습니다. `Options`에 필드를 추가해도 `..Default::default()`를 쓴 코드는 그대로 컴파일됩니다.

**이 방법이 안전한 조건.** `Default` + `..`가 제대로 동작하려면 세 조건이 모두 맞아야 합니다.

1. **모든 필드에 뜻이 있는 기본값이 있다.** 필드 타입 하나라도 `Default`가 아니면 derive부터 실패합니다. 예를 들어 0이 될 수 없는 `NonZeroU8` 필드가 있으면 ``error[E0277]: the trait bound `NonZero<u8>: Default` is not satisfied``입니다. `Default`를 손으로 구현하면 피할 수는 있지만, 기본값을 지어낼 수밖에 없는 필드라면 애초에 옵션 구조체에 넣지 않습니다.
2. **모든 필드가 공개다.** 비공개 필드가 하나라도 있으면 모듈 밖에서는 `..Default::default()`도 쓸 수 없습니다. 적지 않은 필드를 기준 값에서 옮겨 오는 일도 바깥에서 하는 것이기 때문입니다(4.1절, E0451).
3. **필드 사이에 지킬 규칙이 없다.** 3번 구역에서 `연결 60s > 전체 10s`인 요청과 본문이 있는 GET 요청이 모두 아무 말 없이 만들어집니다. 필드가 공개라서 검사할 자리가 없습니다.

검사를 `Request::new`에 넣어 `Result`를 돌려주게 할 수는 있습니다. 하지만 그러면 2.4절의 빌더를 옵션 구조체 모양으로 만든 셈이고, `Options`의 필드는 여전히 공개라서 내부 표현을 바꿀 수 없습니다. 필드를 비공개로 두는 이유는 [ch22](../ch22-minimize-visibility/README.md)에서 다룹니다.

필드에 기본값을 직접 적는 문법(`struct Request { timeout: Duration = …, }`과 `Request { url, .. }`)이 있으면 "기본값이 없는 필드는 반드시 적게" 할 수 있지만, 1.100.0에서는 불안정 기능이라 ``error[E0658]: default values on fields are experimental``입니다.

**따르지 않아도 되는 경우.** 테스트 코드에서 몇 필드만 관심 있는 값을 만들 때는 기본값의 뜻을 따질 필요가 없으므로 `..Default::default()`를 편하게 써도 됩니다. 공개 타입에 `Default`를 구현할지 자체의 판단은 [`ch10-standard-traits`](../ch10-standard-traits/README.md)에서 다룹니다.

### 2.4 지침 3: 필수 값·비공개 필드·필드 사이 규칙이 있으면 빌더를 만든다

**문제 상황.** 2.3절의 조건 중 둘째와 셋째를 풀어야 합니다. 필드를 비공개로 두어 만드는 길을 하나로 줄이고, 모든 값이 정해진 뒤에 규칙을 검사할 자리가 필요합니다. 이 챕터의 규칙은 셋입니다. 주소는 `http://`나 `https://`로 시작하고, GET 요청에는 본문이 없고, 연결 제한 시간은 전체 제한 시간보다 길지 않습니다.

**비권장 코드.** 공개 필드 타입은 규칙을 어긴 값도 받습니다([`03_consuming_builder.rs`](examples/03_consuming_builder.rs)의 1번 구역).

```text
    ftp 주소 → 만들어짐: GET ftp://files.example.com/a.zip | 본문 없음 | 전체 30s, 연결 5s
    본문이 있는 GET → 만들어짐: GET https://api.example.com/items | 본문 있음 | 전체 30s, 연결 5s
    연결 60s > 전체 10s → 만들어짐: GET https://api.example.com/items | 본문 없음 | 전체 10s, 연결 60s
```

**권장 코드.** `Request`의 필드를 비공개로 두고, 같은 필드를 가진 `RequestBuilder`만 `Request`를 만들게 합니다.

```rust
pub struct Request { method: Method, url: String, headers: Vec<String>, /* … 모두 비공개 */ }

impl RequestBuilder {
    pub fn new(method: Method, url: impl Into<String>) -> Self { /* 선택 값은 기본값으로 */ }
    pub fn header(mut self, line: impl Into<String>) -> Self { self.headers.push(line.into()); self }
    pub fn timeout(mut self, d: Duration) -> Self { self.timeout = d; self }
    // body, connect_timeout도 같은 모양
    pub fn build(self) -> Result<Request, BuildError> { /* 규칙 셋을 검사한 뒤 필드를 옮김 */ }
}

let request = Request::builder(Method::Post, api) // RequestBuilder::new를 부르는 관례적인 이름
    .header("Content-Type: application/json")
    .body(json)
    .timeout(Duration::from_secs(10))
    .build()?;
```

같은 입력이 2번 구역에서는 이렇게 됩니다.

```text
    ftp 주소 → 에러: 주소는 http:// 또는 https://로 시작해야 합니다: "ftp://files.example.com/a.zip"
    본문이 있는 GET → 에러: GET 요청에는 본문을 넣을 수 없습니다
    연결 60s > 전체 10s → 에러: 연결 제한 시간(60s)이 전체 제한 시간(10s)보다 깁니다
```

설계의 요점은 다음과 같습니다.

- **필수 값은 빌더 생성자의 인자로 받습니다.** 기본값이 없는 메서드와 주소를 빠뜨리면 E0061(``this function takes 2 arguments but 1 argument was supplied``)로 컴파일되지 않습니다. 필수 값을 세터로 받으면 "아직 정하지 않았음"을 실행 중에야 알릴 수 있습니다(2.6절의 derive 판).
- **선택 값은 이름이 있는 메서드로 받습니다.** 메서드 이름이 부르는 쪽 코드에 남으므로 2.2절의 순서 실수가 생기지 않고, 부르지 않은 값은 기본값이 됩니다. 메서드는 대입 이상의 일도 할 수 있습니다. `header`는 부를 때마다 한 줄씩 덧붙입니다.
- **규칙은 `build()`에서 한 번 검사합니다.** 세터마다 검사하면 `timeout(10s)`과 `connect_timeout(60s)`를 부르는 순서에 따라 결과가 달라집니다. 필드 사이의 규칙은 모든 값이 정해진 시점에만 판단할 수 있습니다.
- **`build()`는 그런 규칙이 있을 때 `Result`를 돌려줍니다.** 규칙이 없고 필수 값을 생성자로 받았다면 실패할 일이 없으므로 `Request`를 바로 돌려줍니다(예제 04의 `build`). 에러 타입은 [ch4](../ch4-idiomatic-errors/README.md)의 지침대로 열거형에 `Error`를 구현했습니다. 규칙을 어겼다고 패닉하지 않는 이유는 [`ch18-dont-panic`](../ch18-dont-panic/README.md)에서 다룹니다.
- **`Request`의 필드는 비공개입니다.** 모듈 밖에서 구조체 리터럴로 만들면 E0451([Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.6절)이므로 `Request`를 만드는 길은 `build()` 하나뿐입니다. 그래서 모든 `Request` 값은 규칙을 통과한 값입니다. [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.12절의 검증 타입과 같은 원리를 필드가 많은 타입에 적용한 것입니다.

**표준 라이브러리의 근거.** `std::thread::Builder`가 같은 모양의 소비형 빌더입니다. 시그니처는 로컬 std 문서에서 확인했습니다.

```rust
pub fn new() -> Builder
pub fn name(self, name: String) -> Builder
pub fn stack_size(self, size: usize) -> Builder
pub fn spawn<F, T>(self, f: F) -> io::Result<JoinHandle<T>>
```

선택 값(이름, 스택 크기)은 세터로 받고, 마지막의 `spawn`이 `build()` 역할을 합니다. `spawn`이 `io::Result`를 돌려주는 것은 운영체제가 스레드를 만들지 못할 수 있기 때문으로, 만드는 순간에야 알 수 있는 실패가 있으면 `Result`를 돌려준다는 원칙과 같습니다. 예제 03의 3번 구역에서 새 스레드가 자기 이름 `Some("fetcher-1")`을 봅니다.

**따르지 않아도 되는 경우(빌더가 과한 경우).**

- 필드가 적고 모두 필수라면 `new(a, b)`나 구조체 리터럴로 충분합니다(2.2절).
- 필드가 모두 공개여도 되고, 모두 뜻이 있는 기본값이 있고, 규칙이 없다면 2.3절의 옵션 구조체가 더 적은 코드로 같은 편의를 줍니다.
- 크레이트 안에서만 쓰는 타입이나 테스트 데이터는 리터럴로 만듭니다. 필드가 늘면 컴파일러가 고칠 곳을 알려 줍니다.
- 빌더에는 비용이 있습니다. 같은 필드를 두 타입에 정의하고, 선택 값마다 메서드를 하나씩 씁니다. 이 반복은 2.6절의 derive로 줄일 수 있습니다.

필수 세터를 불렀는지를 타입으로 추적해 빠뜨리면 컴파일되지 않게 하는 빌더(타입 상태 빌더)도 있습니다. 이 방법은 Part 8 `ch5-static-guarantees`의 타입 상태 프로그래밍에서 다룹니다.

### 2.5 지침 4: 쓰임새에 맞춰 소비형과 `&mut` 빌더를 고른다

**문제 상황.** 빌더 세터의 수신자를 `self`로 할지(소비형) `&mut self`로 할지(`&mut` 빌더) 정해야 합니다. 한 줄 체인에서는 둘이 똑같아 보이지만 다른 쓰임새에서 갈립니다. [`04_mut_builder.rs`](examples/04_mut_builder.rs)는 같은 빌더를 두 형태로 만들어 비교합니다.

| 쓰임새 | 소비형 (`self` → `Self`) | `&mut` 빌더 (`&mut self` → `&mut Self`) | 예제 04 |
| --- | --- | --- | --- |
| 한 문장 안의 체인 `new(..).x(..).build()` | 됨 | 됨 | 3번 구역 |
| 조건에 따라 세터 부르기 | `b = b.x(..);`로 다시 담음. 빠뜨리면 E0382 | `b.x(..);` | 1번 구역 |
| 같은 설정으로 여러 개 만들기 | 빌더에 `Clone`을 붙이고 만들 때마다 `clone()` | `build(&self)`를 여러 번 | 2번 구역 |
| 설정을 채운 빌더를 함수에서 돌려주기 | 표현식 하나 | 변수에 담고, 고치고, 돌려주는 세 문장 | 3번 구역 |
| 체인의 중간 결과를 변수에 담기 | 됨 | E0716 | 4.3절 |
| `build()`가 필드를 | 옮김(복제 없음). 한 번만 부름 | 복제. 몇 번이든 부름 | — |

**비권장 코드.** 조건에 따라 조립하는 일이 잦은데 소비형 빌더를 쓰면, 세터를 부를 때마다 결과를 같은 변수에 다시 담아야 합니다(1번 구역).

```rust
let mut b = owned::RequestBuilder::new(format!("{base}/me"));
if let Some(t) = token {
    b = b.header(format!("Authorization: Bearer {t}")); // `b = `를 빠뜨리면 E0382 (4.2절)
}
if slow_network {
    b = b.timeout(Duration::from_secs(60));
}
let request = b.build();
```

**권장 코드.** `&mut` 빌더는 빌더가 제자리에 있고 세터가 그것을 고치므로 다시 담을 필요가 없습니다.

```rust
let mut b = by_ref::RequestBuilder::new(format!("{base}/me"));
if let Some(t) = token {
    b.header(format!("Authorization: Bearer {t}"));
}
if slow_network {
    b.timeout(Duration::from_secs(60));
}
let request = b.build();
```

`build(&self)`가 빌더를 빌리기만 하므로, 2번 구역처럼 주소만 바꿔 가며 같은 빌더로 요청을 여러 개 만들 수도 있습니다. 소비형은 같은 일을 하려면 빌더 전체를 매번 복제해야 합니다. 다만 `&mut` 빌더의 `build`도 빌더가 계속 가진 필드를 복제해서 내주므로, 비용이 사라진 것이 아니라 `build()` 안으로 옮겨 온 것입니다. 빌더를 다시 쓰면 앞에서 넣은 설정이 남는다는 점도 기억해 둡니다. 2번 구역의 마지막 줄에서 한 요청에만 넣으려던 `X-Trace: on` 헤더가 그다음 요청에도 들어갑니다.

**반대로 소비형이 편한 곳.** 공통 설정을 채운 빌더를 함수에서 돌려줄 때입니다(3번 구역).

```rust
fn json_api_owned(url: &str) -> owned::RequestBuilder {
    owned::RequestBuilder::new(url)
        .header("Accept: application/json")
        .timeout(Duration::from_secs(10))
}

fn json_api_by_ref(url: &str) -> by_ref::RequestBuilder {
    let mut b = by_ref::RequestBuilder::new(url); // 체인은 &mut RequestBuilder를 돌려주므로
    b.header("Accept: application/json")          // 그대로 돌려줄 수 없습니다
        .timeout(Duration::from_secs(10));
    b
}
```

**`&mut` 빌더와 E0716, 그리고 에디션.** `&mut` 빌더로 `let b = RequestBuilder::new(..).header(..);`처럼 쓰면 E0716입니다(4.3절). `new(..)`가 만든 빌더는 어느 변수에도 담기지 않은 임시값이라 그 `let` 문이 끝날 때 버려지는데, `b`는 그것을 빌린 참조이기 때문입니다. `let x = &임시값;`과 달리 메서드 호출의 수신자가 된 임시값은 수명이 늘어나지 않습니다. 한 문장 안에서 `build()`까지 끝내면 임시값이 문장 끝까지 살아 있으므로 괜찮습니다(3번 구역의 `json_api_by_ref(..).header(..).build()`).

이 에러는 2024 에디션의 임시값 범위 변경과 관계가 없습니다. 같은 코드를 2015·2018·2021·2024 에디션으로 컴파일해 모두 E0716임을 확인했습니다. [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md)에서 본 두 변경(`if let` 조건식, 블록의 마지막 표현식) 중 빌더에 닿는 것은 체인을 블록의 마지막 표현식에 둔 모양뿐입니다. `let r = { RequestBuilder::new(..).header(..) }.build();`는 2021 에디션에서는 컴파일되고 2024 에디션에서는 E0716입니다. 블록의 마지막 표현식에서 만든 임시값이 2024부터 블록이 끝날 때 버려지기 때문입니다(Part 2 ch4 2.6절).

**표준 라이브러리의 근거.** 시그니처는 로컬 std 문서에서 확인했습니다.

| 타입 | 세터 | 마지막 단계 | 형태 |
| --- | --- | --- | --- |
| `std::thread::Builder` | `name(self, name: String) -> Builder` | `spawn(self, f)` | 소비형 |
| `std::fs::OpenOptions` | `append(&mut self, append: bool) -> &mut Self` 등 | `open(&self, path) -> io::Result<File>` | `&mut` |
| `std::process::Command` | `arg(&mut self, arg) -> &mut Command` 등 | `status`/`output`/`spawn(&mut self)` | `&mut` |

기준이 보입니다. `thread::Builder`는 만드는 것(스레드)이 빌더의 내용(이름 `String`)을 가져가야 하고 한 빌더로 스레드를 하나 만듭니다. `OpenOptions`와 `Command`는 같은 설정으로 파일을 여러 번 열고 프로그램을 여러 번 실행합니다. 예제 04의 4번 구역은 한 `OpenOptions`로 같은 파일을 두 번 열어 두 줄을 덧붙입니다. `Command`를 임시값에서 시작해 변수에 담을 때의 E0716은 Part 1 ch10 2.8절에서 이미 만났습니다.

**따르지 않아도 되는 경우.** 2.6절의 `derive_builder`는 `&mut` 빌더를 기본으로 만들고 문서에서도 권장하지만, 대부분의 사용이 한 줄 체인이고 결과물이 빌더의 내용을 가져간다면 소비형이 자연스럽습니다. 어느 쪽이든 한 빌더의 세터는 한 형태로 통일합니다.

### 2.6 지침 5: 반복되는 빌더 코드는 derive로 만들되, 생성된 동작을 확인한다

**문제 상황.** 손으로 쓴 빌더는 같은 필드를 두 번 정의하고, 선택 값마다 거의 같은 세터를 씁니다. 빌더가 여러 개라면 이 반복이 부담됩니다.

**비권장 코드.** 예제 05의 `by_hand` 모듈처럼 손으로 쓴 빌더가 틀린 것은 아닙니다. 다만 필드를 하나 추가할 때마다 `Request`의 필드, 빌더의 필드, 빌더 생성자의 기본값, 세터, `build()`의 옮기는 코드까지 다섯 곳을 고쳐야 하고, 하나를 빠뜨려도 컴파일되는 경우(새 세터를 만들지 않은 경우 등)가 있습니다. derive로 바꾼 뒤에도 생성된 빌더의 동작을 모르고 쓰면 틀린 결과가 나올 수 있는데, 그 비권장 코드는 아래 "다른 점 2"에서 봅니다.

**권장 코드.** `derive_builder`의 derive 매크로(절차적 매크로, [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.20절)에 맡깁니다([`05_derive_builder.rs`](examples/05_derive_builder.rs)).

```rust
use derive_builder::Builder;

#[derive(Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Request {
    method: Method,
    #[builder(setter(into))]
    url: String,
    #[builder(setter(each(name = "header", into)), default)]
    headers: Vec<String>,
    #[builder(setter(into, strip_option), default)]
    body: Option<String>,
    #[builder(default = "DEFAULT_TIMEOUT")]
    timeout: Duration,
    #[builder(default = "DEFAULT_CONNECT")]
    connect_timeout: Duration,
}
```

| 속성 | 하는 일 |
| --- | --- |
| (속성 없음) | 필드 이름과 같은 세터. 정하지 않고 `build()`하면 에러 |
| `default`, `default = "식"` | 정하지 않으면 `Default::default()` 또는 그 식의 값 |
| `setter(into)` | 세터가 `Into<필드 타입>`을 구현한 값을 받음 |
| `setter(strip_option)` | `Option<T>` 필드의 세터가 `T`를 받아 `Some`으로 넣음 |
| `setter(each(name = "header"))` | 컬렉션 필드에 하나씩 덧붙이는 세터 |
| `build_fn(validate = "경로")` | `build()`가 필드를 채우기 전에 `fn(&RequestBuilder) -> Result<_, String>`을 부름 |
| `pattern = "owned"` | 소비형 빌더(세터가 `self`를 받아 `Self`를 돌려줌) |

**생성된 빌더의 형태.** 0.20.2에서 확인한 것입니다(예제 05의 1번 구역, 크레이트 문서).

- `RequestBuilder`는 필드마다 `Option`을 담고 `Clone`과 `Default`를 구현합니다. `RequestBuilder::default()`로 시작합니다.
- 기본은 `&mut` 빌더입니다. 세터의 반환 타입이 `&mut derived::RequestBuilder`로 출력됩니다. 문서는 조건에 따라 세터를 부르기 쉽다는 이유로 이 형태를 기본이자 권장으로 듭니다. `pattern = "owned"`를 주면 세터가 빌더를 값으로 돌려주는 것도 확인했습니다.
- `build(&self)`는 `Result<Request, RequestBuilderError>`를 돌려줍니다. 문서에 따르면 `&mut` 빌더의 `build`는 빌더가 가진 값을 복제해서 만듭니다.
- `RequestBuilderError`는 `UninitializedField(&'static str)`와 `ValidationError(String)` 배리언트를 가집니다. 검증 함수가 돌려준 `String`이 `ValidationError`에 담깁니다.

예제 05는 이 derive 판과 예제 04 모양의 손으로 쓴 `&mut` 빌더를 나란히 둡니다. 시작(`new(method, url)`과 `default().method(..).url(..)`)만 다르고 나머지 세터 호출은 글자 하나 다르지 않으며, 결과와 거절 메시지도 같습니다.

**다른 점 1: 필수 값은 실행 중에 확인됩니다.** 손으로 쓴 판은 주소를 빠뜨리면 E0061로 컴파일되지 않습니다. derive 판은 컴파일되고 `build()`가 알립니다.

```text
    derive 판: Err(UninitializedField("url")) → `url` must be initialized
```

**다른 점 2: 기본값은 검증 뒤에 채워집니다.** 크레이트 문서에 적힌 동작이고, 예제 05의 3번 구역에서 확인할 수 있습니다. 연결 제한 시간만 60s로 정하면 검증 함수가 보는 빌더의 값은 이렇습니다.

```text
      검증 함수가 보는 값: timeout = None, connect_timeout = Some(60s)
```

그래서 다음과 같은 검증 함수는 비권장 코드입니다. 둘 다 정해진 경우만 비교하므로 "연결 60s, 전체는 기본값 30s"인 요청이 검사를 건너뛰고 만들어집니다(이 툴체인과 0.20.2에서 따로 확인).

```rust
fn validate(&self) -> Result<(), String> {
    if let (Some(c), Some(t)) = (self.connect_timeout, self.timeout) && c > t {
        return Err(format!("{c:?} > {t:?}"));
    }
    Ok(())
}
```

예제 05의 검증 함수는 `self.timeout.unwrap_or(DEFAULT_TIMEOUT)`처럼 기본값을 직접 채워 넣고 비교합니다. 그래서 기본값을 상수로 빼 두고 `default = "DEFAULT_TIMEOUT"`과 검증 함수가 같은 상수를 쓰게 했습니다.

**따르지 않아도 되는 경우.** 빌더가 한두 개뿐이거나 필수 값을 컴파일 시점에 확인하고 싶다면 손으로 씁니다. `derive_builder`는 절차적 매크로라서 빌드할 때 `derive_builder_macro`, `derive_builder_core`, `darling`, `syn`, `quote`, `proc-macro2` 등이 함께 컴파일됩니다(`cargo tree`로 확인). 의존성을 들일지의 판단은 [ch25](../ch25-dependency-graph/README.md)에서, 매크로를 직접 쓰는 일은 [`ch28-macros-judiciously`](../ch28-macros-judiciously/README.md)에서 다룹니다.

### 2.7 고르는 순서

1. 필드가 두세 개이고 모두 필수이며, 타입이 서로 달라 바꿔 넣으면 컴파일되지 않는가? → `new(a, b)`나 구조체 리터럴.
2. 필수 값을 빼고 나면 남은 필드가 모두 공개여도 되고, 모두 뜻이 있는 기본값이 있고, 필드 사이 규칙이 없는가? → 필수 값은 인자로, 나머지는 옵션 구조체와 `..Default::default()`.
3. 그렇지 않으면(비공개 필드, 필드 사이 규칙, 덧붙이는 세터) → 빌더. 필수 값은 빌더 생성자로 받고, 규칙이 있으면 `build()`가 `Result`를 돌려줍니다.
4. 빌더의 형태: 여러 문장에 걸친 조립이나 반복 생성이 많으면 `&mut` 빌더, 결과물이 빌더의 내용을 가져가고 대개 한 줄 체인이면 소비형.
5. 같은 모양의 빌더가 여럿이면 `derive_builder`. 필수 값이 실행 중 에러가 되고 기본값이 검증 뒤에 채워진다는 점을 받아들일 수 있을 때 씁니다.

### 2.8 책과 달라진 점

책의 설명 가운데 rustc 1.100.0에서 동작이 달라진 것은 없습니다. 소비형 빌더의 E0382와 `&mut` 빌더를 임시값에서 시작할 때의 E0716은 그대로입니다(4.2절, 4.3절). 다만 이 챕터에는 책에 없는 지침이 있습니다.

- **책에 없는 지침.** 지침 1(위치 인자 생성자의 문제), 지침 2의 비공개 필드 이야기, 지침 3에서 `build()`가 `Result`를 돌려주며 필드 사이 규칙을 검사하는 부분은 책의 Item 7이 명시적으로 다루는 내용이 아닙니다(2026-11-19에 온라인판에서 확인). 책은 필드가 많은 구조체의 초기화 부담, 기본값이 맞지 않는 필드가 있을 때의 `Default`의 한계, 소비형과 `&mut` 빌더의 득실, derive 크레이트를 설명하며, 책의 `build()`는 값을 바로 돌려줍니다. 더한 부분은 본편(Part 0 ch7의 비공개 필드, Part 0 ch9의 검증 타입)과 잇기 위해 이 챕터가 넣은 판단입니다.

### 2.9 기억할 것

- 필드가 많은 타입에 위치 인자 생성자를 만들지 않습니다. 이름이 사라진 인자는 순서가 바뀌어도 컴파일됩니다.
- `..Default::default()`는 모든 필드에 뜻이 있는 기본값이 있을 때만 씁니다. 필수 값까지 `Default`에 넣으면 빠뜨린 값이 조용히 빈 값이 됩니다.
- 비공개 필드나 필드 사이 규칙이 있으면 빌더를 만듭니다. 필수 값은 빌더 생성자로, 선택 값은 세터로 받고, 규칙은 `build()`에서 한 번 검사합니다.
- 조건부 조립과 반복 생성에는 `&mut` 빌더, 결과물이 빌더를 가져가는 한 줄 체인에는 소비형이 편합니다. `&mut` 빌더의 중간 결과를 변수에 담으면 에디션과 관계없이 E0716입니다.
- `derive_builder`가 만드는 빌더는 `&mut` 형태이고, 필수 값은 실행 중에 확인되며, 기본값은 검증 뒤에 채워집니다.

## 3. 예제 안내

| 파일 | 지침 | 비권장 형태 | 권장 형태 |
| --- | --- | --- | --- |
| [`examples/01_positional_vs_literal.rs`](examples/01_positional_vs_literal.rs) | 1 | 인자 여덟 개짜리 `new` | 구조체 리터럴 (그리고 그 비용) |
| [`examples/02_default_update.rs`](examples/02_default_update.rs) | 2 | 필수 값까지 `derive(Default)` | 필수 값은 인자로 + 옵션 구조체 |
| [`examples/03_consuming_builder.rs`](examples/03_consuming_builder.rs) | 3 | 공개 필드 타입 | `build()`가 검사하는 소비형 빌더, `thread::Builder` |
| [`examples/04_mut_builder.rs`](examples/04_mut_builder.rs) | 4 | 쓰임새에 맞지 않는 형태 | 같은 빌더의 소비형 판과 `&mut` 판 비교, `OpenOptions` |
| [`examples/05_derive_builder.rs`](examples/05_derive_builder.rs) | 5 | (README 2.6절의 검증 함수) | 손으로 쓴 판과 `derive_builder` 판 비교 |

모든 명령은 저장소 루트에서 실행하며, 모든 예제는 종료 코드 0으로 끝납니다. 출력의 `본문 17바이트`는 `{"name":"연필"}`의 UTF-8 길이입니다.

### 01_positional_vs_literal — 위치 인자 생성자와 구조체 리터럴

```sh
cargo run -p er-ch7-builders --example 01_positional_vs_literal
```

```text
의도: 전체 제한 시간 30s, 연결 제한 시간 3s
1. 비권장: 인자 여덟 개짜리 new
    GET https://api.example.com/items | 헤더 0 | 본문 없음 | 전체 3s, 연결 30s | 리다이렉트 5 | gzip 켬
2. 권장: 필드 이름을 적는 구조체 리터럴
    GET https://api.example.com/items | 헤더 0 | 본문 없음 | 전체 30s, 연결 3s | 리다이렉트 5 | gzip 켬
3. 구조체 리터럴의 비용: 부르는 곳마다 여덟 필드를 모두 적습니다
    POST https://api.example.com/items | 헤더 1 | 본문 17바이트 | 전체 30s, 연결 3s | 리다이렉트 5 | gzip 켬
    두 리터럴이 같은 값을 되풀이한 필드 5개: url, timeout, connect_timeout, max_redirects, gzip
```

3번 구역의 마지막 줄은 2번 구역의 요청과 3번 구역의 요청을 필드마다 비교해 값이 같은 필드를 센 것입니다.

### 02_default_update — `Default`와 구조체 갱신 문법

```sh
cargo run -p er-ch7-builders --example 02_default_update
```

```text
1. 비권장: 필수 값까지 Default를 derive한 타입
    POST (URL 없음) | 헤더 0 | 본문 17바이트 | 전체 0ns, 연결 0ns | 리다이렉트 0 | gzip 끔
2. 권장: 필수 값은 인자로, 나머지는 뜻이 있는 기본값을 가진 옵션 구조체로
    GET https://api.example.com/items | 헤더 0 | 본문 없음 | 전체 30s, 연결 5s | 리다이렉트 5 | gzip 켬
    POST https://api.example.com/items | 헤더 1 | 본문 17바이트 | 전체 30s, 연결 5s | 리다이렉트 5 | gzip 켬
3. 남는 한계: 필드 사이의 규칙을 검사할 곳이 없습니다
    연결 60s > 전체 10s도 만들어짐: GET https://api.example.com/items | 헤더 0 | 본문 없음 | 전체 10s, 연결 60s | 리다이렉트 5 | gzip 켬
    본문이 있는 GET도 만들어짐: GET https://api.example.com/items | 헤더 0 | 본문 17바이트 | 전체 30s, 연결 5s | 리다이렉트 5 | gzip 켬
```

`0ns`는 `Duration::default()`의 디버그 형식입니다. 2번 구역의 두 요청은 예제 01의 두 리터럴과 같은 요청을 기본값과 다른 필드만 적어 만든 것입니다(연결 제한 시간은 기본값 5s).

### 03_consuming_builder — `build()`가 검사하는 소비형 빌더

```sh
cargo run -p er-ch7-builders --example 03_consuming_builder
```

```text
1. 비권장: 공개 필드 타입은 규칙을 어긴 값도 받습니다
    ftp 주소 → 만들어짐: GET ftp://files.example.com/a.zip | 본문 없음 | 전체 30s, 연결 5s
    본문이 있는 GET → 만들어짐: GET https://api.example.com/items | 본문 있음 | 전체 30s, 연결 5s
    연결 60s > 전체 10s → 만들어짐: GET https://api.example.com/items | 본문 없음 | 전체 10s, 연결 60s
2. 권장: 빌더의 build()가 규칙을 한곳에서 검사합니다
    정상 → POST https://api.example.com/items | 헤더 ["Content-Type: application/json"] | 본문 17바이트 | 전체 10s, 연결 5s
    ftp 주소 → 에러: 주소는 http:// 또는 https://로 시작해야 합니다: "ftp://files.example.com/a.zip"
    본문이 있는 GET → 에러: GET 요청에는 본문을 넣을 수 없습니다
    연결 60s > 전체 10s → 에러: 연결 제한 시간(60s)이 전체 제한 시간(10s)보다 깁니다
3. 표준 라이브러리의 소비형 빌더: std::thread::Builder
    새 스레드가 본 자기 이름: Some("fetcher-1")
```

3번 구역은 새 스레드가 `thread::current().name()`을 읽어 돌려준 값을 `main`에서 출력하므로 실행할 때마다 같습니다.

### 04_mut_builder — 소비형 빌더와 `&mut` 빌더

```sh
cargo run -p er-ch7-builders --example 04_mut_builder
```

```text
1. 여러 문장에 걸쳐 조건에 따라 조립하기
    소비형: https://api.example.com/me | 헤더 ["Authorization: Bearer abc123"] | 전체 60s
    &mut:   https://api.example.com/me | 헤더 ["Authorization: Bearer abc123"] | 전체 60s
    결과가 같은가: true
2. 같은 설정으로 여러 개 만들기
    https://api.example.com/items | 헤더 ["Accept: application/json"] | 전체 10s
    https://api.example.com/tags | 헤더 ["Accept: application/json"] | 전체 10s
    https://api.example.com/users | 헤더 ["Accept: application/json"] | 전체 10s
    두 방식의 결과가 같은가: true
    그다음 요청: https://api.example.com/next | 헤더 ["Accept: application/json", "X-Trace: on"] | 전체 10s
3. 한 줄 체인
    소비형: https://api.example.com/items | 헤더 ["Accept: application/json", "X-Page: 2"] | 전체 10s
    &mut:   https://api.example.com/items | 헤더 ["Accept: application/json", "X-Page: 2"] | 전체 10s
4. 표준 라이브러리의 &mut 빌더: std::fs::OpenOptions
    같은 OpenOptions로 두 번 열어 덧붙인 내용: "첫째 줄\n둘째 줄\n"
```

1~3번 구역은 두 형태의 결과가 같다는 것이 요점입니다. 달라지는 것은 부르는 쪽(1·2번)과 빌더를 돌려주는 함수(3번)의 코드입니다. 4번 구역은 운영체제의 임시 디렉터리 아래에 프로세스 ID로 이름을 붙인 디렉터리를 만들어 쓰고, 끝나면 지웁니다.

### 05_derive_builder — 손으로 쓴 빌더와 `derive_builder`

```sh
cargo run -p er-ch7-builders --example 05_derive_builder
```

```text
1. derive_builder가 만든 빌더의 모양
    세터의 반환 타입: &mut derived::RequestBuilder
    build의 반환 타입: core::result::Result<derived::Request, derived::RequestBuilderError>
2. 같은 사용법, 같은 결과
    손으로 쓴 판: POST https://api.example.com/items | 헤더 ["Content-Type: application/json"] | 본문 17바이트 | 전체 10s, 연결 5s
    derive 판:   POST https://api.example.com/items | 헤더 ["Content-Type: application/json"] | 본문 17바이트 | 전체 10s, 연결 5s
    같은가: true
3. 같은 규칙으로 거절
    본문이 있는 GET
      손으로 쓴 판: GET 요청에는 본문을 넣을 수 없습니다
      derive 판:   GET 요청에는 본문을 넣을 수 없습니다
    연결 60s, 전체는 기본값
      손으로 쓴 판: 연결 제한 시간(60s)이 전체 제한 시간(30s)보다 깁니다
      derive 판:   연결 제한 시간(60s)이 전체 제한 시간(30s)보다 깁니다
      검증 함수가 보는 값: timeout = None, connect_timeout = Some(60s)
4. 다른 점: 필수 값을 빠뜨리면
    derive 판: Err(UninitializedField("url")) → `url` must be initialized
```

1번 구역의 타입 이름은 `std::any::type_name_of_val`로 얻은 것에서 크레이트 이름(예제 이름)을 뗀 것입니다. 두 판은 규칙을 한 함수 `check_rules`에 두고 함께 쓰므로 판단이 같습니다. 마지막 줄의 영어 메시지는 `derive_builder`가 만든 에러 타입의 `Display`입니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일하고, 고쳐서 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd appd-a/ch7-builders
rustc --edition 2024 compile_fail/e0451_default_update_private_field.rs --out-dir /tmp/er-ch7
rustc --edition 2024 compile_fail/e0382_consuming_builder_in_if.rs --out-dir /tmp/er-ch7
rustc --edition 2024 compile_fail/e0716_mut_builder_from_temporary.rs --out-dir /tmp/er-ch7
```

세 파일 모두 2015·2018·2021 에디션에서도 같은 에러 코드로 실패합니다.

### 4.1 비공개 필드가 있는 구조체에 `..Default::default()` — E0451

파일: [`compile_fail/e0451_default_update_private_field.rs`](compile_fail/e0451_default_update_private_field.rs)

```text
error[E0451]: field `headers` of struct `Options` is private
  --> compile_fail/e0451_default_update_private_field.rs:54:11
   |
52 |     let options = http::Options {
   |                   ------------- in this type
53 |         timeout: std::time::Duration::from_secs(10),
54 |         ..Default::default() // E0451: 비공개 필드 headers를 여기서 옮겨 와야 함
   |           ^^^^^^^^^^^^^^^^^^ field `headers` is private
```

- **읽는 법**: 밑줄이 `headers`를 적은 곳이 아니라 `..` 뒤의 `Default::default()`에 있습니다. 코드에 `headers`를 한 번도 적지 않았는데도, `..`가 그 필드를 옮겨 와야 하므로 에러입니다.
- **원인**: 구조체 갱신 문법은 적지 않은 필드를 기준 값에서 하나씩 옮겨 오는 것입니다(Part 0 ch5 2.4절). 비공개 필드를 옮기는 일을 모듈 밖에서 할 수는 없습니다. 비공개 필드를 직접 적은 리터럴이 E0451인 것(Part 0 ch7)과 같은 규칙입니다.
- **해결**: 비공개 필드가 있는 타입은 바깥에 생성 경로를 따로 줍니다. 공개 필드만 고치면 된다면 `let mut options = http::Options::default(); options.timeout = …;`처럼 만든 뒤 고치고, 필드 사이 규칙까지 지켜야 한다면 빌더를 둡니다(2.4절).

### 4.2 소비형 빌더의 세터를 부르고 결과를 다시 담지 않음 — E0382

파일: [`compile_fail/e0382_consuming_builder_in_if.rs`](compile_fail/e0382_consuming_builder_in_if.rs)

```text
error[E0382]: use of moved value: `b`
  --> compile_fail/e0382_consuming_builder_in_if.rs:52:19
   |
47 |     let b = RequestBuilder::new("https://api.example.com/me");
   |         - move occurs because `b` has type `RequestBuilder`, which does not implement the `Copy` trait
...
50 |         b.header(line); // 빌더가 여기서 옮겨 감
   |           ------------ `b` moved due to this method call
51 |     }
52 |     let request = b.build(); // E0382
   |                   ^ value used here after move
   |
note: `RequestBuilder::header` takes ownership of the receiver `self`, which moves `b`
```

- **읽는 법**: `note:`가 핵심입니다. `header`가 수신자를 `self`로 받으므로 `b`가 그 호출로 옮겨 갔습니다.
- **원인**: `&mut` 빌더처럼 `b.header(line);`만 썼습니다. 소비형 빌더의 세터는 고친 빌더를 **돌려주는데**, 그 반환값을 아무도 받지 않아 버려졌습니다. 이후의 `b.build()`는 이미 옮겨 간 값을 쓰려는 것입니다.
- **해결**: `let mut b = …;`로 선언하고 `b = b.header(line);`처럼 돌려받은 빌더를 다시 담습니다(예제 04의 1번 구역). 이런 조립이 잦다면 빌더를 `&mut` 형태로 바꾸는 것을 고려합니다(2.5절).

세터의 인자 자리에 `b.header(format!(..))`처럼 매크로를 바로 쓰면, 이 툴체인에서는 같은 상황인데도 메시지가 달라졌습니다. 레이블이 `b` 아래의 `value moved here`로 바뀌고, 수신자 설명 대신 ``note: if `RequestBuilder` implemented `Clone`, you could clone the value``가 나옵니다. 이 제안을 따르면 복제본에 헤더를 넣고 그 결과를 버리게 되므로 문제가 그대로 남습니다. 돌려받은 빌더를 다시 담는 것이 맞는 해결입니다.

### 4.3 `&mut` 빌더를 임시값에서 시작해 변수에 담음 — E0716

파일: [`compile_fail/e0716_mut_builder_from_temporary.rs`](compile_fail/e0716_mut_builder_from_temporary.rs)

```text
error[E0716]: temporary value dropped while borrowed
  --> compile_fail/e0716_mut_builder_from_temporary.rs:62:13
   |
62 |     let b = RequestBuilder::new("https://api.example.com/me").header("Accept: application/json");
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^                                   - temporary value is freed at the end of this statement
   |             |
   |             creates a temporary value which is freed while still in use
63 |     if slow_network {
64 |         b.timeout(Duration::from_secs(60)); // E0716: b가 가리키는 임시 빌더는 이미 버려졌음
   |         - borrow later used here
   |
help: consider using a `let` binding to create a longer lived value
   |
62 ~     let mut binding = RequestBuilder::new("https://api.example.com/me");
63 ~     let b = binding.header("Accept: application/json");
   |
```

- **읽는 법**: 밑줄 친 `RequestBuilder::new(..)`가 임시값이고, 줄 끝의 `-`(세미콜론 자리)에서 버려집니다. 그런데 64행이 그것을 빌린 `b`를 씁니다. `help:`가 고친 코드를 그대로 보여 줍니다.
- **원인**: `header`는 빌더를 빌린 `&mut RequestBuilder`를 돌려주므로, `b`는 빌더 자체가 아니라 임시값을 가리키는 참조입니다. 2.5절에서 설명했듯이 에디션과 관계없는 규칙입니다.
- **해결**: `help:`처럼 빌더를 먼저 변수에 담습니다. 이름은 `binding`보다 뜻이 드러나게 `let mut b = RequestBuilder::new(..); b.header(..);`로 쓰는 편이 좋습니다. 설정을 한 문장에서 끝낸다면 `RequestBuilder::new(..).header(..).build()`처럼 `build()`까지 이어 써도 됩니다.

### 4.4 그 밖의 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| 인자가 8개 이상인 함수 | Clippy `too_many_arguments`(기본 warn): `this function has too many arguments (8/7)` | 이름이 보이는 방법으로 바꿉니다(2.2절). |
| 필수 값까지 들어 있는 타입에 `derive(Default)` | 빠뜨린 필수 값이 빈 값으로 조용히 채워짐(예제 02) | 필수 값은 생성자 인자로 받습니다(2.3절). |
| `Default`가 아닌 필드 타입(`NonZeroU8` 등)이 있는 구조체에 `derive(Default)` | E0277 ``the trait bound `NonZero<u8>: Default` is not satisfied`` | 뜻이 있는 기본값이 있으면 손으로 구현하고, 없으면 그 필드를 필수 값으로 따로 받습니다. |
| `let r = { RequestBuilder::new(..).header(..) }.build();` | 2024 에디션에서 E0716(2021에서는 통과) | 블록을 없애거나, 빌더를 블록 밖의 변수에 담습니다(2.5절). |
| `&mut` 빌더를 다시 쓰며 한 요청에만 세터를 부름 | 그 설정이 다음 요청에도 남음(예제 04의 2번 구역) | 공통 빌더를 복제해 쓰거나, 남지 않아야 할 설정은 빌더 밖에서 넣습니다. |
| `derive_builder`의 검증 함수에서 `Some`끼리만 비교 | 기본값에 맡긴 필드가 `None`이라 검사를 건너뜀(2.6절) | 검증 함수에서 기본값을 직접 채워 넣고 비교합니다(예제 05). |
| 필드에 기본값을 적는 문법 `struct S { t: u32 = 30 }` | 1.100.0에서 E0658 `default values on fields are experimental` | 아직 불안정 기능입니다. `impl Default`나 빌더를 씁니다. |

## 5. 연습 문제

### 연습 1. 빌더에 세터와 규칙 추가하기

예제 03의 `http::RequestBuilder`에 `bearer(token: &str)` 메서드를 추가해 `Authorization: Bearer <token>` 헤더를 넣게 하세요. 그리고 같은 이름의 헤더가 두 번 들어가면 `build()`가 `BuildError::DuplicateHeader(이름)`으로 거절하게 하세요.

```text
정상 → POST https://api.example.com/items | 헤더 ["Authorization: Bearer abc"] | 본문 없음 | 전체 30s, 연결 5s
중복 → 에러: 헤더 "Authorization"이(가) 두 번 들어 있습니다
```

<details>
<summary>힌트</summary>

- `bearer`는 `self.header(format!("Authorization: Bearer {token}"))`를 돌려주면 됩니다. 소비형이므로 `header`가 돌려준 빌더를 그대로 돌려줍니다.
- 헤더 이름은 `line.split_once(": ")`의 앞부분입니다. `build()`에서 `HashSet<&str>`에 이름을 넣다가 `insert`가 `false`를 돌려주면 중복입니다.
- 이 검사는 세터가 아니라 `build()`에 두는 것이 맞습니다. 헤더를 넣는 순서와 관계없이 결과가 같아야 하기 때문입니다(2.4절).

</details>

### 연습 2. 다시 쓰는 `&mut` 빌더에서 설정이 남지 않게 하기

예제 04의 2번 구역에서 `X-Trace: on` 헤더가 다음 요청에 남지 않도록 고치세요. `by_ref::RequestBuilder`의 메서드는 바꾸지 않습니다.

```text
    추적 요청: https://api.example.com/users | 헤더 ["Accept: application/json", "X-Trace: on"] | 전체 10s
    그다음 요청: https://api.example.com/next | 헤더 ["Accept: application/json"] | 전체 10s
```

<details>
<summary>힌트</summary>

- `by_ref::RequestBuilder`에 `#[derive(Clone)]`을 붙이면, 한 번만 쓸 설정은 복제본에 넣을 수 있습니다. `let mut once = b.clone(); once.header("X-Trace: on"); let traced = once.build();`
- 복제하는 비용이 드는 대신 공통 빌더 `b`는 그대로 남습니다. 소비형 빌더가 2번 구역에서 하던 일(`common.clone()`)과 같은 방법이라는 점을 확인하세요.

</details>

### 연습 3. 예제 03의 빌더를 derive 판으로 바꾸기

예제 03의 `http` 모듈에서 손으로 쓴 `RequestBuilder`를 지우고 `derive_builder`로 만드세요. `main`의 코드는 바꾸지 않고 출력도 그대로여야 합니다.

<details>
<summary>힌트</summary>

- `main`이 `Request::builder(method, url).header(..).body(..)…build()?`처럼 한 줄 체인으로 쓰므로 `#[builder(pattern = "owned")]`로 소비형을 고릅니다.
- `Request::builder`는 `RequestBuilder::default().method(method).url(url)`을 돌려주게 하면 됩니다. 필수 값을 이렇게 받으면 `main`에서는 빠뜨릴 수 없습니다.
- 생성된 `build()`의 에러 타입은 `RequestBuilderError`입니다. `main`은 에러를 `{e}`로만 출력하므로 검증 함수가 기존 `BuildError`의 메시지를 `to_string()`으로 돌려주면 출력이 같아집니다.
- 검증 함수에서 `self.url`은 `Option<String>`, `self.body`는 `Option<Option<String>>`, 기본값에 맡긴 제한 시간은 `None`입니다(2.6절).

</details>

### 연습 4. 방법 고르기

다음 상황마다 2.7절의 순서를 따라 무엇을 고를지 정하고 이유를 한 문장으로 적어 보세요.

1. 크레이트 안에서만 쓰는 `Rgb { r: u8, g: u8, b: u8 }`.
2. 공개 라이브러리의 `WrapOptions { width: usize, break_words: bool, indent: String }`. 세 필드 모두 널리 쓰이는 기본값이 있고, 필드 사이 규칙은 없습니다.
3. 메일 메시지. 받는 사람(한 명 이상)은 필수이고, 제목·본문·첨부 파일(여러 개)은 선택이며, 첨부 파일 크기의 합은 10MB 이하여야 합니다. 첨부 파일은 조건에 따라 붙입니다.
4. 같은 설정(주소, 제한 시간, 최대 연결 수)으로 데이터베이스 연결을 여러 개 여는 설정 타입.

<details>
<summary>힌트</summary>

- 1은 필드가 셋이고 모두 필수이므로 구조체 리터럴(또는 `new(r, g, b)`)입니다. 세 필드의 타입이 같지만 이름(`r`, `g`, `b`)으로 짝을 맞추는 리터럴이라면 순서 실수가 드러납니다(2.2절).
- 2는 옵션 구조체와 `..Default::default()`가 맞습니다(2.3절). 필드를 추가해도 `..`를 쓴 사용자 코드는 깨지지 않습니다.
- 3은 필수 값(받는 사람)과 필드 사이 규칙(크기의 합)이 있으므로 빌더이고, `build()`는 `Result`를 돌려줍니다. 첨부를 조건에 따라 붙이므로 `&mut` 빌더가 편합니다(2.4·2.5절).
- 4는 `OpenOptions`와 같은 모양입니다. 같은 설정으로 여러 번 여는 마지막 단계(`connect(&self)`)를 가진 `&mut` 빌더가 어울립니다(2.5절).

</details>

## 6. 참고 자료

- [Item 7: Use builders for complex types](https://effective-rust.com/builders.html): 이 챕터가 다룬 지침의 원문입니다. 필드 초기화의 부담, `Default`의 한계, 소비형과 `&mut` 빌더의 득실, `build(&self)`로 여러 개 만들기 순서로 설명합니다.
- [`std::default::Default`](https://doc.rust-lang.org/std/default/trait.Default.html): derive 조건과 구조체 갱신 문법과 함께 쓰는 예.
- [`std::thread::Builder`](https://doc.rust-lang.org/std/thread/struct.Builder.html), [`std::fs::OpenOptions`](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html), [`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html): 2.4·2.5절에서 근거로 든 표준 라이브러리의 빌더입니다. 이 챕터의 시그니처는 로컬 std 문서(`rustc --print sysroot` 아래 `share/doc/rust/html`)로 확인했습니다.
- [`derive_builder` 0.20.2 문서](https://docs.rs/derive_builder/0.20.2/derive_builder/): 세 가지 빌더 형태(owned, mutable, immutable)와 기본값이 mutable인 이유, 속성 목록, "기본값은 검증 뒤에 적용되어 검증되지 않는다"는 주의, 생성되는 에러 타입.
- 이 저장소의 본편: [Part 0 ch5](../../part0-getting-started/ch5-structs/README.md)(구조체 리터럴, 구조체 갱신 문법, E0063), [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)(`Default` 구현), [Part 1 ch10](../../part1-standard-lib/ch10-env-process-time/README.md)(`Command`), [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)(검증 타입), [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md)(비공개 필드와 E0451), [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md)(2024 에디션의 임시값 범위).
