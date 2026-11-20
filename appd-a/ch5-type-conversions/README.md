# ch5. 타입 변환 이해하기

Rust에서 값의 타입을 바꾸는 길은 세 갈래입니다. `From`·`TryFrom`처럼 코드에 적어 부르는 변환(conversion), `x as u32` 같은 캐스트(cast), 그리고 컴파일러가 알아서 해 주는 강제 변환(coercion)입니다. 세 가지 모두 본편에서 만났습니다. 이 챕터는 문법과 구현 방법을 다시 설명하지 않고 세 가지 판단을 다룹니다. 변환을 제공할 때 **어느 트레이트를 구현하고, 받을 때 무엇을 바운드로 쓰는가**, **`as` 대신 `from`/`try_from`을 쓰는 이유와 그래도 `as`가 맞는 자리**, 그리고 **컴파일러의 강제 변환을 어디까지 기대해도 되는가**입니다.

[ch1](../ch1-express-data-structures/README.md) 2.1절은 "정수 타입이 맞지 않는다는 에러를 `as`로 덮지 말고, 변환은 경계에서 한 번 한다"고 정하면서 `as`·`From`·`TryFrom` 가운데 무엇을 고를지는 이 챕터로 미뤘습니다. 다섯 지침은 모두 한 원칙에서 나옵니다. **변환이 실패하거나 값을 바꿀 수 있다면, 그 사실이 코드에 드러나야 합니다.** `as`와 대체값으로 덮는 `From`은 그 사실을 숨기고, `TryFrom`의 `Result`와 컴파일 에러는 드러냅니다.

- 원제: [Item 5: Understand type conversions](https://effective-rust.com/casts.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch5-type-conversions`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 본편 챕터의 내용을 안다고 보고 진행합니다.

- [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 2.2~2.4절: `From`/`Into`/`TryFrom`을 구현하는 방법, 포괄 구현과 반사 구현, `From`을 구현할 때의 네 가지 기대
- [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.9~2.10절: `as`가 정수와 실수를 바꾸는 규칙, 검사 연산(`checked_*`)
- [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 2.9절: 오버플로 처리를 타입으로 고르는 `Wrapping`·`Saturating`
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.5절: 역참조 강제 변환
- [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 2.4~2.9절: 트레이트 객체와 dyn 호환

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 실패할 수 있는 변환을 `From`으로 구현하면 무엇이 숨겨지는지 설명하고, 같은 변환을 `TryFrom`으로 구현할 수 있다.
2. 변환을 제공할 때는 `From`을 구현하고, 변환할 수 있는 값을 받을 때는 `Into` 바운드를 쓰는 이유를 설명할 수 있다. `Into` 바운드가 어울리지 않는 경우(정수 리터럴을 받는 매개변수, 트레이트 객체로 쓸 트레이트의 메서드)를 판단할 수 있다.
3. 좁히는 변환에서 `as`가 만드는 값을 예측하고 `try_from`으로 바꿔 실패를 드러낼 수 있으며, `as`가 맞는 자리(실수 → 정수, `const`, 의도한 잘림, 필드 없는 열거형)를 구별할 수 있다.
4. 넓히는 변환에도 `as` 대신 `from`을 쓰면 나중에 타입이 바뀔 때 컴파일러가 알려 준다는 것을 설명하고, 부호가 바뀌는 변환을 넓히는 변환과 구별할 수 있다.
5. 강제 변환이 일어나는 자리와 종류를 나열하고, 제네릭 매개변수와 컨테이너 안쪽에서는 일어나지 않는다는 것을 알고 직접 변환하는 코드를 작성할 수 있다.

## 2. 핵심 개념

지침마다 문제 상황, 비권장 코드, 권장 코드, 지침을 따르지 않아도 되는 경우 순서로 설명합니다. 코드는 핵심만 싣고, 두 형태를 함께 실행해 비교하는 전체 코드는 `examples/`에 있습니다.

### 2.1 변환의 세 갈래

| 방법 | 누가 일으키나 | 실패나 손실을 알리나 | 사용자 정의 타입 | 문법은 |
| --- | --- | --- | --- | --- |
| `From`/`Into` | 코드에 적은 `T::from(x)`, `x.into()` | 실패하지 않는다는 약속 | 구현할 수 있음 | Part 1 ch6 2.2절 |
| `TryFrom`/`TryInto` | 코드에 적은 `T::try_from(x)`, `x.try_into()` | `Result`로 알림 | 구현할 수 있음 | Part 1 ch6 2.4절 |
| `as` 캐스트 | 코드에 적은 `x as T` | 알리지 않음 | 필드 없는 열거형 → 정수만 | Part 1 ch1 2.9~2.10절 |
| 강제 변환 | 컴파일러 | 해당 없음(참조와 포인터의 모양만 바꿈) | `Deref` 구현, 트레이트 객체 | Part 0 ch14 2.5절 |

`as`는 사용자 정의 변환을 부르지 못합니다. 구조체 `Celsius(f64)`에 `c as f64`라고 쓰면 ``error[E0605]: non-primitive cast: `Celsius` as `f64` ``이고, 컴파일러는 `as`가 원시 타입 사이의 변환과 트레이트 객체로의 강제 변환에만 쓰인다고 알려 줍니다. 그래서 내 타입의 변환은 언제나 `From`이나 `TryFrom`으로 제공합니다.

변환할 일이 생기면 위에서부터 차례로 따져 봅니다.

1. 언제나 성공하고 정보를 잃지 않는다 → `From` (`u64::from(x_u32)`)
2. 실패할 수 있다 → `TryFrom` (`u32::try_from(x_u64)`)
3. 두 트레이트가 없지만 뜻이 분명하다(실수 → 정수, 의도한 잘림, `const` 문맥) → `as`. 반올림과 범위 확인은 `as` 앞에서 직접 합니다.
4. 참조의 모양만 바꾼다(`&String` → `&str`) → 강제 변환에 맡깁니다.

### 2.2 지침 1: 실패할 수 있는 변환은 `TryFrom`으로 구현한다

**문제 상황.** 분리수거 요일을 번호(1=월 … 7=일)로 받아 `Weekday` 열거형으로 바꿉니다. 번호는 대부분 1~7이지만, 사람이 입력한 목록에는 8 같은 오타가 섞일 수 있습니다. `match`는 모든 `u8` 값을 다뤄야 하므로 `_ =>` 갈래에 무엇을 둘지가 문제가 됩니다.

**비권장 코드.**

```rust
impl From<u8> for Day {
    fn from(n: u8) -> Self {
        Day(match n {
            1 => Weekday::Mon,
            // ... 2~6
            _ => Weekday::Sun, // 7도, 8도, 0도 일요일
        })
    }
}
```

`From`의 시그니처 `fn from(n: u8) -> Self`는 "모든 `u8`에서 성공한다"는 약속입니다. 이 약속에 맞추려고 모르는 번호를 일요일로 덮었고, 결과만 봐서는 원래 번호가 7이었는지 8이었는지 알 수 없습니다. [`01_try_from_for_fallible.rs`](examples/01_try_from_for_fallible.rs)의 1번 구역에서 `[1, 4, 8]`은 월·목·일이 되어, 아무도 정하지 않은 일요일 수거가 생깁니다. `_ =>` 갈래에 `panic!`을 두는 것도 답이 아닙니다. `From` 문서는 실패할 수 있는 변환이면 `TryFrom`을 쓰고 패닉하는 `From`은 만들지 말라고 적어 두었습니다(Part 1 ch6 2.2절의 "실패하지 않음").

**권장 코드.**

```rust
impl TryFrom<u8> for Weekday {
    type Error = NotAWeekday;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Weekday::Mon),
            // ... 2~6
            7 => Ok(Weekday::Sun),
            other => Err(NotAWeekday(other)),
        }
    }
}
```

실패할 수 있다는 사실이 반환 타입 `Result`에 드러나고, 실패를 어떻게 다룰지는 부르는 쪽이 정합니다. 틀린 번호만 알리고 넘어갈 수도, `collect::<Result<Vec<_>, _>>()`로 목록 전체를 거부할 수도 있습니다(예제 01의 2번 구역). 에러 값에 받은 번호를 담아 두면 무엇이 틀렸는지도 보여 줄 수 있습니다. 에러 타입의 설계는 [`ch4-idiomatic-errors`](../ch4-idiomatic-errors/README.md), `Result`를 다루는 메서드는 [`ch3-option-result-transforms`](../ch3-option-result-transforms/README.md)에서 다룹니다.

**지침을 따르지 않아도 되는 경우.**

- **정말로 언제나 성공하는 방향.** 요일 → 번호는 모든 요일에서 성공하고 정보도 잃지 않으므로 `impl From<Weekday> for u8`이 맞습니다(예제 01의 3번 구역). 필드 없는 열거형을 판별값으로 바꾸는 일은 `as`가 정확히 하는 일이라 그 본문은 `day as u8` 한 줄입니다.
- **고치는 것이 도메인의 규칙일 때.** "100을 넘는 음량은 100으로 맞춘다"처럼 값을 고치는 것이 원하는 동작이라면, 그 규칙은 `From` 안에 숨기지 말고 `Volume::saturating(150)`처럼 규칙이 드러나는 이름의 생성자에 둡니다. `From` 문서도 그럴듯한 변환이 하나뿐이 아니면 이름 있는 메서드나 생성자를 쓰라고 권합니다.
- **부르는 쪽이 실패를 무시하기로 했을 때.** `Weekday::try_from(n).unwrap_or(Weekday::Sun)`처럼 그 결정을 호출부에 적습니다. 같은 일을 하지만 결정이 받는 쪽 `From` 안에 숨지 않고 읽는 사람에게 보입니다.

### 2.3 지침 2: `From`을 구현하고, `Into` 바운드로 받는다

**문제 상황.** 색을 칠하는 함수가 `Rgb`를 다룹니다. 부르는 쪽은 색을 `[u8; 3]` 배열, 회색조 `Gray(u8)`, 옛 크레이트의 `legacy::Cmy`처럼 여러 모양으로 가지고 있습니다. 정할 것이 두 가지입니다. 각 타입에서 `Rgb`로 가는 변환을 무엇으로 구현할지, 그리고 칠하는 함수가 매개변수를 무엇으로 받을지입니다.

**비권장 코드(받는 쪽).**

```rust
fn fill_exact(color: Rgb) -> String

fill_exact(Rgb::from([255, 136, 0]));
fill_exact(Gray(200).into());
```

변환이 부르는 쪽의 몫이 되어 호출마다 `Rgb::from(..)`이나 `.into()`가 붙습니다.

**비권장 코드(구현하는 쪽).**

```rust
impl Into<Rgb> for Cmy {
    fn into(self) -> Rgb { /* .. */ }
}
```

Rust 1.41 이전에는 고아 규칙 때문에 다른 크레이트의 타입으로 가는 `From`을 구현할 수 없어서 이렇게 쓰곤 했습니다(Part 1 ch6 2.2절). 포괄 구현은 `From` → `Into` 방향으로만 있으므로 이 타입에는 `Rgb::from(cmy)`가 없습니다. `.map(Rgb::from)`처럼 변환 함수를 넘기는 자리와 `Rgb: From<T>`를 바운드로 요구하는 함수(4.1절)에서 빠집니다. 예제 02의 3번 구역은 `Cmy` 목록을 바꾸려고 `.map(Into::into)`를 넘기고 목적 타입을 따로 적습니다. clippy는 기본 설정에서 이 구현을 `from_over_into`로 경고합니다.

```text
warning: an implementation of `From` is preferred since it gives you `Into<_>` for free where the reverse isn't true
help: replace the `Into` implementation with `From<legacy::Cmy>`
```

**권장 코드.**

```rust
impl From<Gray> for Rgb {
    fn from(Gray(level): Gray) -> Self {
        Rgb(level, level, level)
    }
}

fn fill(color: impl Into<Rgb>) -> String {
    fill_exact(color.into())
}
```

구현은 `From`, 바운드는 `Into`입니다. 방향이 반대처럼 보이지만 이유는 같습니다. 둘 다 **받아들이는 범위가 가장 넓은 쪽**입니다. `From`을 구현하면 `From`과 `Into`가 함께 생깁니다. `Into` 바운드는 `From`을 구현한 타입, `Into`만 구현한 옛 타입, 반사 구현 덕분에 `Rgb` 자신까지 받습니다([`02_from_impl_into_bound.rs`](examples/02_from_impl_into_bound.rs)의 2번 구역). `From` 문서도 "구현할 때는 `From`, 제네릭 함수의 바운드에는 `Into`"를 권합니다.

`fill`의 본문이 `fill_exact(color.into())` 한 줄인 데에도 이유가 있습니다. 제네릭 함수는 부르는 타입마다 따로 컴파일되므로(단형성화), 제네릭 부분은 변환 한 줄로 줄이고 실제 일은 제네릭이 아닌 함수에 맡기면 복제되는 코드가 작아집니다. 제네릭과 코드 크기의 관계는 [`ch12-generics-vs-trait-objects`](../ch12-generics-vs-trait-objects/README.md)에서 다룹니다.

**지침을 따르지 않아도 되는 경우(`Into` 바운드 대신 구체 타입으로 받을 때).**

- **정수를 받는 매개변수.** `fn schedule_retry(delay_ms: impl Into<u64>)`에 `schedule_retry(500)`을 넘기면 컴파일되지 않습니다. 인자 자리가 리터럴의 타입을 정해 주지 못해 리터럴이 `i32`가 되고, `u64: From<i32>`는 없기 때문입니다(4.2절). 받을 것이 정수 하나라면 `delay_ms: u64`로 받는 편이 부르는 쪽에 편합니다.
- **트레이트 객체로 쓸 트레이트의 메서드.** `impl Into<Rgb>` 매개변수는 타입 매개변수이므로, 그런 메서드를 가진 트레이트는 dyn 호환이 아닙니다. 메서드 `fill(&mut self, color: impl Into<Rgb>)`를 가진 트레이트 `Canvas`를 `&mut dyn Canvas`로 쓰면 ``error[E0038]: the trait `Canvas` is not dyn compatible``와 ``...because method `fill` has generic type parameters``가 나옵니다([ch2](../ch2-express-common-behavior/README.md) 2.6절). 이때는 메서드가 `Rgb`를 받게 하고 변환은 부르는 쪽에 맡깁니다.
- **변환할 모양이 하나뿐인 내부 함수.** 제네릭으로 얻는 편의보다 시그니처를 읽는 비용이 클 수 있습니다. 반대로 받은 문자열을 저장하는 함수의 `impl Into<String>`처럼 부르는 쪽이 여러 모양을 가진 것이 분명하면 `Into` 바운드가 제값을 합니다(Part 1 ch6 2.5절).

### 2.4 지침 3: 좁히는 변환은 `as` 대신 `try_from`

**문제 상황.** 묶음 파일(archive)의 목록 헤더를 씁니다. 헤더 형식은 파일 크기와 수정 시각을 `u32`로 저장하고, 프로그램 안에서는 크기를 `u64`, 수정 시각을 `i64`(유닉스 시간, 초)로 다룹니다. `u32` 필드에 `u64` 값을 넣으려 하면 E0308이 나고([ch1](../ch1-express-data-structures/README.md) 4.1절), 가장 손쉬운 해결은 `as`입니다.

**비권장 코드.**

```rust
fn header_with_as(e: &Entry) -> Header {
    Header {
        size: e.size as u32,         // 위쪽 32비트를 버림
        modified: e.modified as u32, // 음수는 비트 그대로 읽혀 큰 양수가 됨
    }
}
```

`as`는 실패하지 않는 대신 값을 바꿉니다. [`03_narrowing_try_from.rs`](examples/03_narrowing_try_from.rs)에서 약 4.7 GiB인 `movie.mkv`의 크기 5000000000은 705032704가 되어 정확히 2^32(4294967296)바이트가 사라지고, 1969-12-31에 수정된 `scan.tif`의 -86400은 4294880896, 즉 2106년이 됩니다. 헤더만 읽는 쪽은 이 값이 잘린 것인지 알 방법이 없습니다.

**권장 코드.**

```rust
fn header(e: &Entry) -> Result<Header, HeaderError> {
    let size = u32::try_from(e.size).map_err(HeaderError::Size)?;
    let modified = u32::try_from(e.modified).map_err(HeaderError::Modified)?;
    Ok(Header { size, modified })
}
```

같은 자리에서 `try_from`은 `number too large to fit in target type`과 `number too small to fit in target type`으로 실패를 알립니다. 실패할 수 있다는 사실이 반환 타입에 드러나므로, 부르는 쪽은 그 항목을 건너뛸지, 크기 필드가 64비트인 다른 형식으로 쓸지 정할 수 있습니다.

**clippy는 이 `as`를 잡는가.** 기본 설정에서는 잡지 않습니다. `as` 관련 린트는 대부분 pedantic이나 restriction 그룹이라 꺼져 있습니다. 기본 수준은 `clippy-driver -W help`로 확인했습니다.

| 린트 | 그룹 | 기본 수준 | 잡는 것 |
| --- | --- | --- | --- |
| `cast_possible_truncation` | pedantic | 꺼짐 | 잘릴 수 있는 캐스트(`u64 as u32`, `f64 as u8`) |
| `cast_sign_loss` | pedantic | 꺼짐 | 부호를 잃을 수 있는 캐스트(`i64 as u32`) |
| `cast_possible_wrap` | pedantic | 꺼짐 | 부호 있는 타입으로 바뀌며 값이 뒤집힐 수 있는 캐스트(`u32 as i32`) |
| `cast_lossless` | pedantic | 꺼짐 | `from`으로 쓸 수 있는 손실 없는 캐스트(2.5절) |
| `as_conversions` | restriction | 꺼짐 | 모든 `as` |
| `cast_nan_to_int` | suspicious | 경고 | `f64::NAN as u8`처럼 NaN 상수를 정수로 캐스트 |
| `cast_abs_to_unsigned` | suspicious | 경고 | `n.abs() as u32`(`n.unsigned_abs()`를 제안) |

꺼진 린트를 켜고 예제 03을 검사하면 비권장 코드의 두 줄을 잡습니다(핵심 줄만 옮겼습니다).

```sh
cargo clippy -p er-ch5-type-conversions --example 03_narrowing_try_from -- -W clippy::cast_possible_truncation -W clippy::cast_sign_loss
```

```text
warning: casting `u64` to `u32` may truncate the value
help: ... or use `try_from` and handle the error accordingly
warning: casting `i64` to `u32` may truncate the value
warning: casting `i64` to `u32` may lose the sign of the value
```

같은 명령은 4번 구역의 `as`(진행률 계산, `const`)에도 경고합니다. `clamp(0.0, 100.0)`으로 범위를 맞춘 뒤의 `as u8`에도 `cast_sign_loss`가 나옵니다. 린트는 맞게 쓴 `as`를 구별하지 못하고, 이것이 이 린트들이 기본으로 꺼져 있는 이유입니다. 프로젝트에서 켠다면 맞게 쓴 자리마다 `#[expect(clippy::cast_possible_truncation, reason = "…")]`로 이유를 남깁니다. 린트 수준을 프로젝트 단위로 정하는 방법은 [`ch29-listen-to-clippy`](../ch29-listen-to-clippy/README.md)에서 다룹니다.

**지침을 따르지 않아도 되는 경우(`as`가 맞는 자리).**

- **실수 → 정수.** 표준 라이브러리에는 실수 → 정수의 `From`/`TryFrom`이 없습니다. 반올림 방식을 부르는 쪽이 정하라는 뜻입니다. `as`는 소수점 아래를 버리고 범위 밖은 끝값에 붙이며 NaN은 0으로 만들므로, 그 앞에서 반올림과 "계산할 수 없는 경우"를 직접 처리합니다. 예제 03의 `percent`는 `total == 0`(0.0 / 0.0은 NaN)을 먼저 걸러 `None`을 돌려주고, `round()`와 `clamp`를 거친 값에만 `as u8`을 씁니다.
- **`const` 문맥.** 1.100.0에서 `From`은 const 트레이트가 아니어서 `const LIMIT: u64 = u64::from(5_u32);`는 ``error[E0658]: cannot call conditionally-const associated function `<u64 as From<u32>>::from` in constants``입니다. `as`를 쓰고, 값이 들어가는지는 `const _: () = assert!(BLOCK_SIZE <= u32::MAX as u64);`처럼 컴파일할 때 단언으로 확인합니다(예제 03).
- **잘림이 원하는 동작일 때.** 해시 값의 아래 32비트만 쓰는 경우가 그렇습니다. 다만 "넘치면 한 바퀴 돈다"가 값의 성질이라면 Part 1 ch14 2.9절의 `Wrapping`이 의도를 더 분명히 드러냅니다.
- **같은 크기에서 부호만 바꿀 때.** 비트를 그대로 다시 읽는 것이 목적이면 1.87.0에 안정화된 `cast_unsigned()`/`cast_signed()`가 의도를 드러냅니다. `(-1_i32).cast_unsigned()`는 4294967295입니다. 크기가 같은 짝으로만 바꿀 수 있어서 잘림이 섞여 들 수 없습니다.
- **필드 없는 열거형 → 정수.** 지침 1의 `day as u8`입니다.

참고로 `300 as u8`처럼 리터럴을 캐스트하면 기본으로 deny인 rustc 린트 `overflowing_literals`가 ``error: literal out of range for `u8` ``로 막습니다. `as`의 위험은 실행 중에 정해지는 값에 있습니다.

### 2.5 지침 4: 넓히는 변환도 `as` 대신 `from`

**문제 상황.** 글 조회 수를 `u16`으로 세던 통계 구조체가 있고, 통계 차트에 넘길 때 `u32`로 바꿉니다. 지금은 넓히는 변환이라 `as`로 써도 값이 바뀌지 않습니다. 몇 달 뒤 조회 수가 65535를 넘기 시작해 필드를 `u64`로 넓힙니다.

**비권장 코드.**

```rust
pub fn chart_value_as(s: &PageStats) -> u32 {
    s.views as u32 // v1(u16)에서는 넓히는 변환, v2(u64)에서는 좁히는 변환
}
```

`as`는 변환의 방향을 따지지 않습니다. 필드 타입을 바꾼 수정은 경고 하나 없이 컴파일되고, 그날부터 이 줄은 값을 자릅니다. [`04_widening_from.rs`](examples/04_widening_from.rs)에서 v2의 조회 수 4294968530은 1234가 됩니다. 필드를 넓힌 사람이 다른 파일의 이 줄까지 확인했으리라 기대하기는 어렵습니다.

**권장 코드.**

```rust
pub fn chart_value(s: &PageStats) -> u32 {
    u32::from(s.views) // 손실이 없다는 것을 컴파일러가 확인
}
```

`From`은 손실 없는 짝에만 구현되어 있으므로, 필드가 `u64`가 되는 순간 이 줄이 컴파일 에러가 됩니다.

```text
error[E0277]: the trait bound `u32: From<u64>` is not satisfied
  |     ^^^ the trait `From<u64>` is not implemented for `u32`
```

에러가 변환하는 자리를 정확히 가리키므로, 거기서 `u32::try_from`으로 바꾸고 실패를 처리합니다(예제 04의 2번 구역). 지침 3과 지침 4는 같은 원칙의 두 면입니다. **`from`은 "손실이 없다"를, `try_from`은 "실패할 수 있다"를 코드에 적는 방법이고, `as`는 아무것도 적지 않습니다.** 적어 둔 사실은 타입이 바뀔 때마다 컴파일러가 다시 확인합니다.

pedantic 그룹의 `cast_lossless`를 켜면 이런 `as`를 찾아 ``casts from `u16` to `u32` can be expressed infallibly using `From` ``와 ``help: use `u32::from` instead``로 알려 줍니다. 반대 실수, 즉 손실 없는 짝에 `try_from`을 쓰면 기본으로 켜진 `unnecessary_fallible_conversions`가 ``use of a fallible conversion when an infallible one could be used``로 경고합니다.

**부호가 바뀌는 변환은 넓히는 변환이 아닙니다.** `i32` → `u64`는 비트 수가 늘지만 음수를 담을 수 없습니다. `as`는 부호를 확장한 비트를 그대로 읽어 `-3`을 18446744073709551613으로 만들고, `u64::from(i32)`는 처음부터 없어서 컴파일러가 `try_from`을 고르게 합니다. 부호를 버리고 크기만 원했다면 `unsigned_abs()`처럼 그 뜻을 가진 메서드를 씁니다(예제 04의 3번 구역).

**지침을 따르지 않아도 되는 경우.**

- **`usize`가 끼는 변환.** `usize`는 16비트 이상이라는 것만 보장되므로 `usize: From<u16>`까지만 있고 `usize: From<u32>`는 없습니다. 64비트 플랫폼에서 `u32` → `usize`는 언제나 성공하지만, `usize::try_from(x)`를 쓰면 일어나지 않을 `Err`도 다뤄야 합니다(예제 04의 4번 구역). 이런 변환이 많은 코드라면 `x as usize`를 쓰고 32비트 이상 플랫폼만 지원한다고 문서에 밝히는 선택지도 있습니다. 그때는 `as`를 함수 하나에 모아 두면 나중에 찾기 쉽습니다.
- **`const` 문맥.** 지침 3과 같습니다.

### 2.6 지침 5: 강제 변환은 받는 타입이 적힌 자리에서만 기대한다

강제 변환은 두 조건이 맞을 때만 일어납니다.

1. **자리**: 받는 쪽 타입이 코드에 적혀 있는 곳입니다. 함수와 메서드의 인자, 타입을 적은 `let`, 함수의 반환 값, 구조체 필드 초기화, `static`과 `const`, 그리고 이런 자리 안에 있는 배열 원소 등입니다.
2. **종류**: 아래 표의 정해진 변환들입니다. 위쪽 여섯 줄은 [`05_coercion_sites.rs`](examples/05_coercion_sites.rs)의 1번 구역에서 실행됩니다.

| 강제 변환 | 예 |
| --- | --- |
| `&T` → `&U` (`T: Deref<Target = U>`, 역참조 강제 변환. 필요한 만큼 이어짐) | `&String` → `&str`, `&Rc<String>` → `&str`, `&Vec<u32>` → `&[u32]` |
| `&mut T` → `&T` | `let view: &u32 = &mut count;` |
| 배열 → 슬라이스 | `&[u32; 3]` → `&[u32]` |
| 구체 타입 → 트레이트 객체 | `&Button` → `&dyn Widget`, `Box<Text>` → `Box<dyn Widget>` |
| 트레이트 객체 → 슈퍼트레이트의 트레이트 객체(업캐스트, 1.86.0부터) | `&dyn Clickable` → `&dyn Widget` |
| 캡처하지 않는 클로저, 함수 → 함수 포인터 | `apply(\|x\| x * 2, 21)` |
| `&T` → `*const T`, `&mut T` → `*mut T` | 원시 포인터([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.2절) |
| `!` → 모든 타입 | `return`이나 `panic!`이 있는 `match` 갈래 |

공통점은 **참조와 포인터의 모양만 바뀐다**는 것입니다. 가리키는 값을 새로 만들거나 고치지 않으므로 실패할 일이 없고 비용도 거의 없습니다(트레이트 객체가 될 때 vtable 포인터가 붙는 정도입니다). 그래서 컴파일러가 묻지 않고 해 줄 수 있습니다. 정수 사이의 변환이 이 목록에 없는 것도 같은 이유입니다. 값이 바뀔 수 있는 변환은 코드에 적게 합니다(ch1 2.1절). 사용자 정의 타입이 강제 변환에 끼는 길은 `Deref`를 구현하는 것과 트레이트를 구현해 트레이트 객체가 되는 것, 두 가지뿐입니다. 어떤 타입에 `Deref`를 구현해야 하는지는 [`ch8-references-pointers`](../ch8-references-pointers/README.md)에서 다룹니다.

**문제 상황.** 캐시에 넣을 값이 차지하는 바이트 수를 재는 함수를, 어떤 값이든 받도록 제네릭으로 만들었습니다.

```rust
fn measure<T: ?Sized>(value: &T) -> (usize, &'static str) {
    (size_of_val(value), type_name::<T>())
}
```

**비권장 코드.** `&String`을 넘기면 `&str`로 바뀌어 글자의 바이트 수를 잴 것이라고 기대합니다.

```rust
measure(&short) // (24, "alloc::string::String")
measure(&long)  // (24, "alloc::string::String")
```

매개변수 타입이 `&T`이므로 컴파일러는 `T = String`으로 정하면 그만이고, 강제 변환이 필요하지 않습니다. 잰 것은 글자가 아니라 `String` 자체(포인터, 용량, 길이)의 크기라서 내용과 상관없이 24(64비트 기준)입니다. 컴파일은 되지만 기대와 다른 값입니다. 강제 변환은 **받는 타입이 이미 정해진 자리**에서 타입을 맞추는 일이라, 타입 매개변수처럼 받는 타입을 추론해야 하는 자리에서는 일어나지 않습니다.

**권장 코드.** 원하는 타입을 직접 만들어 넘깁니다.

```rust
measure(short.as_str()) // (5, "str")
measure(long.as_str())  // (39, "str")
```

같은 이유로 **컨테이너 안쪽도 강제 변환 자리가 아닙니다.** `Box<Button>` 하나는 `Box<dyn Widget>`이 되고 `&Vec<T>`는 `&[T]`가 되지만, `&Vec<Box<Button>>`을 `&[Box<dyn Widget>]` 자리에 넘기면 E0308입니다(4.3절). 원소를 하나씩 바꾸려면 새 `Vec`을 만들어야 하고, 할당이 필요한 일을 컴파일러가 몰래 하지는 않습니다. 처음부터 `Vec<Box<dyn Widget>>`으로 만들면 `vec![..]`의 원소 하나하나가 강제 변환 자리가 되고, 이미 만든 `Vec`은 `.map(|b| b as Box<dyn Widget>)`으로 원소를 바꿔 모읍니다(예제 05의 3번 구역). 이 코드처럼 `as`로 강제 변환을 일으킬 수도 있습니다. 강제 변환으로 가능한 변환이면 `as`가 그 자리를 만들어 줍니다. `Option<&String>`을 `Option<&str>` 자리에 넘길 수 없는 것(연습 3)도 컨테이너 안쪽의 경우입니다.

[Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 2.7절의 함정도 같은 맥락입니다. `&Box<dyn Shape>`를 `&dyn Shape` 자리에 넘기면 역참조 대신 "`Box<dyn Shape>`를 트레이트 객체로 만드는" 강제 변환이 먼저 시도되어 E0277이 납니다. 강제 변환은 받는 타입에 맞추는 한 겹의 변환이고, 기대한 경로로 일어난다는 보장은 없습니다. 헷갈리면 `.as_ref()`처럼 직접 적습니다.

**지침을 따르지 않아도 되는 경우(손으로 변환하지 않아도 되는 경우).** 거꾸로, 받는 타입이 적힌 자리에서는 강제 변환에 맡기고 손으로 변환하지 않습니다. `shout(&*name)`은 `shout(&name)`과 결과가 같고 읽을 것만 늘며, clippy가 기본 설정에서 `explicit_auto_deref`(``deref which would be done by auto-deref``)로 잡습니다. `total(&scores[..])`나 `shout(name.as_str())`도 결과는 같지만 기본 설정의 clippy는 잡지 않습니다. 또 메서드 호출의 수신자는 강제 변환이 아니라 자동 참조·역참조([Part 0 ch5](../../part0-getting-started/ch5-structs/README.md) 2.10절)로 메서드를 찾으므로, `Rc<String>`에 바로 `.len()`을 부를 수 있습니다. 강제 변환 자리와 메서드 탐색의 전체 규칙은 Part 9 `ch7-type-system`과 Part 10 `ch5-type-conversions`에서 다룹니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **`Infallible`은 이제 `!`의 별칭입니다.** 책은 실패하지 않는 `TryFrom`의 에러 타입 `Infallible`이 언젠가 never 타입 `!`로 바뀔 수 있다고 적었습니다. 1.100.0에서 never 타입이 안정화되어 `std::convert::Infallible`의 정의는 `pub type Infallible = !;`이고, `type_name::<Infallible>()`도 `!`를 출력합니다. 문서는 오래된 Rust를 지원할 필요가 없다면 `!`를 직접 쓰라고 안내합니다.
- **강제 변환에 업캐스트가 더해졌습니다.** 책의 강제 변환 목록에는 없지만, 1.86.0부터 `&dyn Clickable`을 슈퍼트레이트의 `&dyn Widget`으로 바꾸는 강제 변환이 안정화되었습니다(2.6절). 업캐스트의 쓰임은 [`ch19-avoid-reflection`](../ch19-avoid-reflection/README.md)에서 다룹니다.
- **부호만 바꾸는 메서드가 생겼습니다.** 책의 기준 버전(1.70)에는 없던 `cast_signed()`/`cast_unsigned()`가 1.87.0에 안정화되어, 같은 크기의 정수 사이에서 부호만 바꾸는 `as`를 이름 있는 메서드로 바꿀 수 있습니다(2.4절). 1.100.0 문서에는 `truncate`, `widen`, `saturating_cast` 같은 정수 변환 메서드도 보이지만 모두 nightly 전용 실험 API라서, stable에서 `x.truncate()`를 부르면 ``error[E0658]: use of unstable library feature `integer_widen_truncate` ``입니다. 크기를 줄이면서 잘림을 원할 때 stable에서 가장 직접적인 방법은 지금도 `as`입니다.
- **E0277의 도움말 목록이 다릅니다.** `u16::from(x_u32)`에 대해 책은 `u16`이 구현한 `From`으로 `NonZeroU16`, `bool`, `u8`을 보여 주지만, 1.100.0은 `` `From<bool>` ``, `` `From<u8>` ``, `` `From<std::ascii::Char>` ``를 표준 라이브러리 소스 위치와 함께 보여 줍니다(`std::ascii::Char`는 불안정 타입입니다). 에러 코드와 첫 줄(``the trait bound `u16: From<u32>` is not satisfied``)은 같습니다.
- **책에 없는 지침.** 지침 3이 권하는 `try_from`은 책의 Item 5가 `as`의 대안으로 명시한 것이 아닙니다(2026-11-19에 온라인판에서 확인). 책은 `as`보다 `from`/`into`를 쓰라고 권하고 `as`가 손실 있는 변환을 허용한다는 점까지 설명합니다. `from`이 없는 좁히는 방향에 `try_from`을 쓰라는 부분은 이 챕터가 더했습니다.
- **달라지지 않은 점.** `as`의 손실 있는 변환은 그대로이고, 모든 `as`를 잡는 clippy `as_conversions`는 여전히 restriction 그룹이라 기본으로 꺼져 있습니다.

### 2.8 기억할 것

- 실패할 수 있는 변환은 `TryFrom`으로 구현합니다. `From` 안에서 대체값으로 덮거나 패닉하지 않습니다.
- 구현은 `From`, 제네릭 바운드는 `Into`입니다. 다만 정수를 받거나 dyn 호환이 필요한 메서드라면 구체 타입으로 받습니다.
- 좁히는 변환은 `try_from`, 넓히는 변환은 `from`으로 씁니다. `as`는 아무것도 적지 않으므로 타입이 바뀌어도 컴파일러가 알려 주지 못합니다.
- `as`는 `TryFrom`이 없는 변환(실수 → 정수, `const`, 의도한 잘림, 필드 없는 열거형)에만 쓰고, 반올림과 범위 확인은 그 앞에서 직접 합니다.
- 강제 변환은 받는 타입이 적힌 자리에서 참조와 포인터의 모양만 바꿉니다. 제네릭 매개변수와 컨테이너 안쪽에서는 직접 변환합니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 함께 실행해 결과를 비교합니다. 저장소 루트에서 실행합니다. 패닉하는 예제는 없습니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 |
| --- | --- | --- | --- |
| [`01_try_from_for_fallible.rs`](examples/01_try_from_for_fallible.rs) | 1 | 모르는 번호를 덮는 `From<u8>` | `TryFrom<u8>` |
| [`02_from_impl_into_bound.rs`](examples/02_from_impl_into_bound.rs) | 2 | `Rgb` 매개변수, `Into`만 구현, `From` 바운드 | `From` 구현, `impl Into<Rgb>` |
| [`03_narrowing_try_from.rs`](examples/03_narrowing_try_from.rs) | 3 | `as u32` | `u32::try_from` |
| [`04_widening_from.rs`](examples/04_widening_from.rs) | 4 | 넓히는 변환의 `as u32` | `u32::from` |
| [`05_coercion_sites.rs`](examples/05_coercion_sites.rs) | 5 | 제네릭에 `&String`을 넘김 | `as_str()`로 직접 바꿔 넘김 |

### 01_try_from_for_fallible — 실패할 수 있는 변환

1번 구역은 `_ =>` 갈래에서 모르는 번호를 일요일로 바꾸는 `From<u8>`을, 2번 구역은 같은 번호를 `Err`로 돌려주는 `TryFrom<u8>`을 씁니다. 2번 구역의 마지막 줄은 `try_into()`로 바꾼 결과를 `Result<Vec<_>, _>`로 모아, 하나라도 틀리면 목록 전체를 거부하는 방법입니다. 3번 구역은 반대 방향(요일 → 번호)이 `From`인 경우입니다.

```sh
cargo run -p er-ch5-type-conversions --example 01_try_from_for_fallible
```

```text
1. 비권장: From이 모르는 번호를 일요일로 덮습니다
    [1, 4, 8] → ["월", "목", "일"]
2. 권장: TryFrom은 같은 번호를 에러로 돌려줍니다
    1 → 월요일
    4 → 목요일
    8 → 거부: 1~7 범위 밖
    전부 모으기: Err(NotAWeekday(8))
3. 반대 방향은 언제나 성공하므로 From
    [월, 일] → [1, 7]
```

### 02_from_impl_into_bound — `From` 구현과 `Into` 바운드

1번 구역은 `Rgb`만 받는 함수에 부르는 쪽이 변환해서 넘기고, 2번 구역은 `impl Into<Rgb>`로 받는 함수에 가진 값을 그대로 넘깁니다. 두 구역의 출력이 같다는 것이 요점입니다. 달라진 것은 부르는 쪽 코드뿐이고, 2번 구역은 `Rgb(0, 0, 0)` 자신도 받습니다. 3번 구역은 `From`을 구현한 `Gray`는 `.map(Rgb::from)`으로, `Into`만 구현한 `legacy::Cmy`는 `.map(Into::into)`와 목적 타입 표기로 바꿔야 한다는 차이를 보여 줍니다. `legacy::Cmy`의 `Into` 구현에는 `#[expect(clippy::from_over_into, reason = "…")]`를 붙였고, 속성을 지우면 2.3절의 경고가 납니다.

```sh
cargo run -p er-ch5-type-conversions --example 02_from_impl_into_bound
```

```text
1. 비권장: Rgb만 받는 함수 — 부르는 쪽이 호출마다 변환합니다
    #ff8800 #c8c8c8 #ff0000
2. 권장: impl Into<Rgb>로 받는 함수 — 가진 값을 그대로 넘깁니다
    #ff8800 #c8c8c8 #ff0000 #000000
3. Into만 구현한 타입은 From으로 쓰는 곳에서 빠집니다
    [Rgb(0, 0, 0), Rgb(255, 255, 255)]
    [Rgb(0, 0, 255), Rgb(255, 255, 255)]
    fill_from(Gray(128)) = #808080
```

### 03_narrowing_try_from — 좁히는 변환

같은 세 항목을 `as u32`로 쓰는 헤더(1번 구역)와 `u32::try_from`으로 쓰는 헤더(2번 구역)에 넣습니다. 3번 구역은 비권장 헤더를 다시 읽었을 때 무엇이 사라졌는지 계산합니다. 4번 구역은 `TryFrom`이 없어 `as`가 맞는 자리, 즉 반올림과 0 확인을 거친 실수 → 정수 변환과 컴파일할 때 단언으로 확인한 `const` 변환입니다.

```sh
cargo run -p er-ch5-type-conversions --example 03_narrowing_try_from
```

```text
1. 비권장: as u32로 헤더를 씁니다
    notes.txt  size=     12800 modified=1700000000
    movie.mkv  size= 705032704 modified=1700000000
    scan.tif   size=      1200 modified=4294880896
2. 권장: u32::try_from으로 헤더를 씁니다
    notes.txt  size=     12800 modified=1700000000
    movie.mkv  거부(size): number too large to fit in target type
    scan.tif   거부(modified): number too small to fit in target type
3. 비권장 헤더를 다시 읽으면
    movie.mkv: 4294967296 바이트를 잃었습니다
    scan.tif: 1970년부터 약 136년 뒤의 파일로 기록됐습니다
4. as가 맞는 자리: TryFrom이 없는 변환
    진행률 3/8 = Some(38)
    진행률 0/0 = None
    블록 크기 = 4096
```

진행률 3/8은 37.5%이고, `round()`가 0.5를 0에서 먼 쪽으로 보내 38이 됩니다.

### 04_widening_from — 넓히는 변환

모듈 `v1`(조회 수 `u16`)과 `v2`(조회 수 `u64`)에 같은 `as` 줄을 둡니다. 1번 구역에서는 두 방법의 결과가 같고, 2번 구역에서는 같은 `as` 줄이 4294968530(`u32::MAX`보다 1235 큼)을 1234로 자릅니다. `v2`의 권장 함수는 `u32::from`이 컴파일되지 않아 `try_from`으로 고친 결과입니다. 3번 구역은 부호가 바뀌는 변환, 4번 구역은 `usize`의 `From`이 `u16`까지만 있다는 것을 보여 줍니다. 4번 구역의 `Ok(70000)`은 64비트 환경에서의 결과입니다.

```sh
cargo run -p er-ch5-type-conversions --example 04_widening_from
```

```text
1. v1(u16): 두 방법의 결과가 같습니다
    as   → 50000
    from → 50000
2. v2(u64): 같은 as 줄이 값을 자릅니다
    as       → 1234
    try_from → Err(TryFromIntError(PosOverflow))
3. 부호가 바뀌는 변환은 넓어 보여도 넓히는 변환이 아닙니다
    as u64        → 18446744073709551613
    u64::try_from → Err(TryFromIntError(NegOverflow))
    unsigned_abs  → 3
4. usize: 플랫폼마다 크기가 달라 From이 적습니다
    usize::from(u16)     = 640
    usize::try_from(u32) = Ok(70000)
```

`TryFromIntError`의 디버그 형식에 보이는 `PosOverflow`/`NegOverflow`는 내부 표현이고, 기본 형식(`{}`)으로 출력하면 예제 03처럼 문장이 나옵니다.

### 05_coercion_sites — 강제 변환이 일어나는 자리

1번 구역은 2.6절 표의 강제 변환을 함수 인자, 타입을 적은 `let`, 반환 값 자리에서 일으킵니다. 2번 구역은 제네릭 함수 `measure`에 `&String`을 넘긴 결과(비권장)와 `as_str()`로 바꿔 넘긴 결과(권장)를 비교합니다. 3번 구역은 `Vec<Box<Button>>`을 원소마다 바꿔 `Vec<Box<dyn Widget>>`을 만드는 방법과, 처음부터 `Vec<Box<dyn Widget>>`으로 만드는 방법입니다.

```sh
cargo run -p er-ch5-type-conversions --example 05_coercion_sites
```

```text
1. 받는 타입이 적힌 자리에서 일어나는 강제 변환
    &String → &str            : RUST
    &Rc<String> → &str        : SHARED
    &Vec<u32> → &[u32]        : 155
    &[u32; 3] → &[u32]        : 6
    &Button → &dyn Widget     : [확인]
    클로저 → fn(u32) -> u32   : 42
    &mut u32 → &u32           : 7
    Box<Text> → Box<dyn ..>   : 안내문
    &dyn Clickable → &dyn ..  : [확인]
    반환 값 &String → &str    : 변환 정리
2. 제네릭 매개변수에는 강제 변환이 없습니다
    비권장 &short        : (24, "alloc::string::String")
    비권장 &long         : (24, "alloc::string::String")
    권장 short.as_str()  : (5, "str")
    권장 long.as_str()   : (39, "str")
3. 컨테이너 안쪽에도 강제 변환이 없습니다
    원소를 바꿔 만든 Vec : [저장] [취소]
    처음부터 dyn인 Vec   : [열기] 파일 없음
```

2번 구역의 24는 64비트 환경에서 `String`의 크기(포인터, 용량, 길이 각 8바이트)입니다. 39는 `"강제 변환은 자리를 가립니다"`의 UTF-8 바이트 수(한글 12자 × 3 + 공백 3 × 1)입니다. `type_name`이 출력하는 경로(`alloc::string::String`)는 타입이 정의된 위치이며, 실행마다 같습니다.

## 4. 자주 하는 실수

세 실수 모두 "변환이 일어나리라 기대했는데 일어나지 않는" 경우입니다. 직접 확인하려면 챕터 디렉터리에서 다음을 실행합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않게 하려는 것입니다).

```sh
cd appd-a/ch5-type-conversions
rustc --edition 2024 compile_fail/e0277_into_only_no_from.rs --out-dir /tmp/er-ch5
rustc --edition 2024 compile_fail/e0277_into_bound_int_literal.rs --out-dir /tmp/er-ch5
rustc --edition 2024 compile_fail/e0308_no_coercion_inside_vec.rs --out-dir /tmp/er-ch5
```

### 4.1 `Into`만 구현한 타입을 `From` 바운드에 넘김 — E0277

[`compile_fail/e0277_into_only_no_from.rs`](compile_fail/e0277_into_only_no_from.rs)

```text
error[E0277]: the trait bound `Rgb: From<Cmy>` is not satisfied
  --> compile_fail/e0277_into_only_no_from.rs:41:32
   |
41 |     println!("{:?}", fill_from(Cmy(0, 255, 255))); // E0277
   |                      --------- ^^^^^^^^^^^^^^^^ unsatisfied trait bound
   |                      |
   |                      required by a bound introduced by this call
   |
help: the trait `From<Cmy>` is not implemented for `Rgb`
      but trait `From<[u8; 3]>` is implemented for it
  --> compile_fail/e0277_into_only_no_from.rs:17:1
   |
17 | impl From<[u8; 3]> for Rgb {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^
   = help: for that trait implementation, expected `[u8; 3]`, found `Cmy`
note: required by a bound in `fill_from`
  --> compile_fail/e0277_into_only_no_from.rs:34:10
   |
32 | fn fill_from<T>(color: T) -> Rgb
33 | where
34 |     Rgb: From<T>,
   |          ^^^^^^^ required by this bound in `fill_from`
```

- **읽는 법**: `note`가 요구의 출처, 즉 받는 쪽의 바운드 `Rgb: From<T>`를 가리킵니다. `Cmy`에는 `Into<Rgb>`가 있지만, `Into`에서 `From`을 만들어 주는 포괄 구현은 없으므로 이 바운드를 만족하지 못합니다. `help`는 `Rgb`가 구현한 다른 `From`을 보여 줄 뿐 해결책은 아닙니다.
- **해결**: 두 쪽 모두 고칠 수 있습니다. 구현하는 쪽은 `impl From<Cmy> for Rgb`로 바꿉니다(clippy `from_over_into`의 제안과 같습니다). 받는 쪽은 바운드를 `T: Into<Rgb>`로 바꾸고 본문을 `color.into()`로 씁니다(예제 02의 `fill`). 하나만 고쳐도 컴파일되지만, 둘 다 고칠 수 있다면 둘 다 고칩니다.
- **참고**: `Rgb`에 다른 `From` 구현이 하나도 없으면 같은 `Rgb::from(Cmy(..))`가 E0308(``expected `Rgb`, found `Cmy` ``)로 보고됩니다. 후보가 반사 구현 `From<Rgb> for Rgb` 하나뿐이라 컴파일러가 인자 타입을 `Rgb`로 정하기 때문입니다. 이때 붙는 ``help: call `Into::into` on this expression``을 따르면 `Rgb::from(cmy.into())`가 되는데, 이것은 변환을 두 번 적은 우회로입니다.

### 4.2 `impl Into<u64>` 매개변수에 정수 리터럴을 넘김 — E0277

[`compile_fail/e0277_into_bound_int_literal.rs`](compile_fail/e0277_into_bound_int_literal.rs)

```text
error[E0277]: the trait bound `u64: From<i32>` is not satisfied
  --> compile_fail/e0277_into_bound_int_literal.rs:23:35
   |
23 |     println!("{}", schedule_retry(500)); // E0277
   |                    -------------- ^^^ the trait `From<i32>` is not implemented for `u64`
   |                    |
   |                    required by a bound introduced by this call
   |
   = help: `u64` implements trait `From<T>`:
             From<bool>
             From<char>
             From<std::ascii::Char>
             From<u16>
             From<u32>
             From<u8>
   = note: required for `i32` to implement `Into<u64>`
note: required by a bound in `schedule_retry`
  --> compile_fail/e0277_into_bound_int_literal.rs:15:34
   |
15 | fn schedule_retry(delay_ms: impl Into<u64>) -> u64 {
   |                                  ^^^^^^^^^ required by this bound in `schedule_retry`
```

- **읽는 법**: 500은 `u64`에 들어가는 값인데 `i32`가 나온 것이 핵심입니다. 매개변수 타입이 `u64`라면 리터럴이 `u64`로 정해지지만, `impl Into<u64>`는 "`u64`로 바꿀 수 있는 무엇이든"이라 리터럴의 타입을 정해 주지 못합니다. 단서가 없는 정수 리터럴은 `i32`가 되고, `i32`는 음수를 담을 수 있으므로 `u64: From<i32>`가 없습니다. `help`의 목록에 `u8`, `u16`, `u32`가 있지만 컴파일러는 그중 하나를 골라 주지 않습니다.
- **Part 1 ch6 4.2절과의 차이**: 그 절은 `i32` 변수를 `u8`로 `into()`하는, 정말로 좁히는 변환이었습니다. 여기서는 의도한 변환이 넓히는 변환인데도 리터럴의 기본 타입 때문에 실패합니다.
- **해결**: 부르는 쪽은 `schedule_retry(500_u32)`처럼 접미사로 타입을 정합니다. 받는 쪽은 받을 것이 정수 하나뿐이면 `delay_ms: u64`로 받습니다. 그러면 리터럴은 `u64`가 되고, `u16` 변수를 가진 부르는 쪽은 `schedule_retry(u64::from(short))`로 넘깁니다(2.3절).

### 4.3 `&Vec<Box<구체 타입>>`을 `&[Box<dyn Trait>]` 자리에 넘김 — E0308

[`compile_fail/e0308_no_coercion_inside_vec.rs`](compile_fail/e0308_no_coercion_inside_vec.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_no_coercion_inside_vec.rs:33:29
   |
33 |     println!("{}", draw_all(&buttons)); // E0308
   |                    -------- ^^^^^^^^ expected `&[Box<dyn Widget>]`, found `&Vec<Box<Button>>`
   |                    |
   |                    arguments to this function are incorrect
   |
   = note: expected reference `&[Box<(dyn Widget + 'static)>]`
              found reference `&Vec<Box<Button>>`
note: function defined here
  --> compile_fail/e0308_no_coercion_inside_vec.rs:27:4
   |
27 | fn draw_all(widgets: &[Box<dyn Widget>]) -> String {
   |    ^^^^^^^^ ---------------------------
```

- **읽는 법**: `expected`와 `found`를 바깥부터 비교합니다. `&Vec<..>` → `&[..]`는 강제 변환이 해 줄 수 있지만, 그 안의 원소 타입 `Box<Button>`과 `Box<dyn Widget>`이 다릅니다. 강제 변환은 바깥 한 겹의 모양만 바꾸고 원소 타입은 바꾸지 않습니다. `'static`은 `Box<dyn Widget>`에 생략된 기본 라이프타임 바운드입니다.
- **해결**: 처음부터 `let buttons: Vec<Box<dyn Widget>> = vec![..];`로 만듭니다. 그러면 원소마다 강제 변환이 일어납니다. 이미 `Vec<Box<Button>>`이 있다면 `.into_iter().map(|b| b as Box<dyn Widget>).collect::<Vec<_>>()`로 새 `Vec`을 만듭니다(예제 05의 3번 구역). 받는 쪽을 고칠 수 있고 한 가지 타입만 다룬다면 `fn draw_all<W: Widget>(widgets: &[Box<W>])` 같은 제네릭도 방법이며, 두 선택의 비교는 [ch2](../ch2-express-common-behavior/README.md) 2.6절에 있습니다.

## 5. 연습 문제

### 연습 1. 헤더에 항목 수 더하기

예제 03의 헤더 앞에 항목 수를 `u16`으로 적는다고 합시다. `ENTRIES.len()`(`usize`)을 `u16`으로 바꾸는 함수를 `as`와 `try_from`으로 각각 써 보고, 항목이 100000개일 때 두 함수가 무엇을 돌려주는지 확인하세요. 실제로 100000개를 만들지 않고 확인하는 방법도 생각해 보세요.

<details>
<summary>힌트</summary>

`fn count_as(n: usize) -> u16 { n as u16 }`와 `fn count(n: usize) -> Result<u16, TryFromIntError> { u16::try_from(n) }`를 만들고 `count_as(100_000)`, `count(100_000)`을 부르면 됩니다. 변환하는 일을 `len()`을 부르는 곳과 떼어 함수로 만든 덕분에 큰 값을 직접 넣어 볼 수 있습니다. `as`는 100000 − 65536 = 34464를 돌려주고, `try_from`은 `Err`입니다.

</details>

### 연습 2. `as` 분류하기

다음 다섯 줄을 `from`, `try_from`, `as` 그대로, 다른 메서드 중 하나로 바꿔 보세요. 각 변수의 타입은 주석과 같습니다.

```rust
let total = count as u64;           // count: u32
let index = offset as usize;        // offset: i64, 음수일 수 있음
let level = (ratio * 10.0) as u8;   // ratio: f64, 0.0~1.0
let bits = signed as u32;           // signed: i32, 비트 패턴을 그대로 쓰려는 것
let low = hash as u32;              // hash: u64, 아래 32비트만 쓰려는 것
```

<details>
<summary>힌트</summary>

첫째는 손실이 없으므로 `u64::from(count)`입니다. 둘째는 음수와 32비트 플랫폼 두 가지 이유로 실패할 수 있으므로 `usize::try_from(offset)`입니다. 셋째는 `TryFrom`이 없는 실수 → 정수라 `as`가 맞지만, 반올림할지(`round()`) 버릴지를 먼저 정하고 `ratio`가 범위를 벗어날 수 있다면 `clamp`도 씁니다. 넷째는 같은 크기의 부호만 바꾸므로 `signed.cast_unsigned()`가 의도를 드러냅니다. 다섯째는 잘림이 원하는 동작이라 `as`가 맞습니다. 주석이 이유를 말해 주므로 그 주석은 코드에도 남깁니다. 다섯 줄을 파일에 넣고 `-W clippy::cast_possible_truncation -W clippy::cast_lossless`를 켜서 clippy가 어느 줄을 잡는지도 보세요.

</details>

### 연습 3. `Option<&String>`을 `Option<&str>`로

다음 코드는 E0308로 컴파일되지 않습니다. 강제 변환이 왜 일어나지 않는지 설명하고 고쳐 보세요.

```rust
fn greet(nick: Option<&str>) -> String {
    format!("안녕, {}", nick.unwrap_or("손님"))
}

let mut nicks: HashMap<String, String> = HashMap::new();
nicks.insert("kim".to_string(), "민수".to_string());
let text = greet(nicks.get("kim")); // E0308
```

<details>
<summary>힌트</summary>

`get`은 `Option<&String>`을 돌려줍니다. `&String` → `&str`은 강제 변환되지만, `Option` 안쪽은 4.3절의 `Vec` 안쪽과 마찬가지로 강제 변환 자리가 아닙니다. 에러는 ``expected `Option<&str>`, found `Option<&String>` ``입니다. 안쪽 값을 하나씩 바꾸는 `nicks.get("kim").map(String::as_str)`로 고칩니다. 값이 `Option<String>`이라면 [Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md)의 `as_deref()`가 같은 일을 합니다.

</details>

### 연습 4. `Into` 바운드 메서드를 dyn 호환으로

예제 02의 `fill`을 메서드로 가진 트레이트 `Canvas`를 만들고, `Screen`과 `Printer` 두 타입에 구현한 뒤 `Vec<Box<dyn Canvas>>`에 담아 보세요. 메서드를 `fn fill(&mut self, color: impl Into<Rgb>)`로 선언했을 때의 에러를 확인하고, 부르는 쪽의 편의를 되도록 잃지 않으면서 dyn 호환이 되도록 고쳐 보세요.

<details>
<summary>힌트</summary>

처음 선언은 E0038(``because method `fill` has generic type parameters``)입니다. 트레이트 메서드는 `fn fill(&mut self, color: Rgb)`로 두고, 편의 함수는 트레이트 밖에 `fn fill_any(canvas: &mut dyn Canvas, color: impl Into<Rgb>) { canvas.fill(color.into()) }`처럼 따로 만듭니다. 트레이트 객체로 부르는 부분과 변환하는 제네릭 부분을 나누는 것으로, 예제 02의 `fill`과 `fill_exact`를 나눈 것과 같은 모양입니다. 트레이트에 `where Self: Sized`를 붙인 제네릭 메서드를 두는 방법은 [`ch12-generics-vs-trait-objects`](../ch12-generics-vs-trait-objects/README.md)에서 다룹니다.

</details>

## 6. 참고 자료

- [Effective Rust, Item 5: Understand type conversions](https://effective-rust.com/casts.html): 이 챕터가 다루는 지침의 출처입니다. 사용자 정의 변환, 캐스트, 강제 변환을 차례로 다룹니다.
- [`std::convert::From`](https://doc.rust-lang.org/std/convert/trait.From.html): "When to implement From" 절에 `From`을 구현할 때의 기대(실패하지 않음, 손실 없음, 값 보존, 명백함)와 "바운드에는 `Into`를 쓰라"는 권고가 있습니다.
- [`std::convert::Into`](https://doc.rust-lang.org/std/convert/trait.Into.html): Rust 1.41 이전에 `Into`를 직접 구현하던 이유를 설명합니다.
- [`std::convert::Infallible`](https://doc.rust-lang.org/std/convert/type.Infallible.html): 1.100.0에서 never 타입의 별칭이 된 실패하지 않는 변환의 에러 타입입니다.
- [The Rust Reference: Type coercions](https://doc.rust-lang.org/reference/type-coercions.html): 강제 변환 자리와 종류의 전체 목록입니다.
- [The Rust Reference: Type cast expressions](https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions): `as`가 할 수 있는 변환과 정수·실수 변환의 정확한 규칙(잘림, 부호 확장, 실수의 포화)입니다.
- 본편: [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)(변환 트레이트 구현), [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md)(정수·실수의 `as`와 검사 연산), [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md)(`Wrapping`/`Saturating`), [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)(역참조 강제 변환)
