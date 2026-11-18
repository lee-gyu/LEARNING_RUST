# ch6. 핵심 트레이트: 변환, 연산자, 비교, 해시, 기본값, 복제

앞 챕터들은 여러 번 "이것은 트레이트 덕분인데, 자세한 것은 `ch6-core-traits`에서"라고 미뤄 두었습니다.

- [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)와 [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md): `?`와 `"메시지".into()`가 값을 `Box<dyn Error>`로 바꾸는 것은 `From` 트레이트입니다.
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md): `Add`를 구현해 `+`를 썼고, `std::ops`의 나머지 연산자와 비교·변환 트레이트를 남겨 두었습니다. [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)의 Rustlings 안내는 `Cow`를 미뤘습니다.
- [ch1](../ch1-std-overview/README.md): `300_i32 as u8`이 조용히 `44`가 되는 대신 쓸 `From`/`TryFrom`, 그리고 `PartialOrd`/`Ord`/`Hash` 트레이트 자체.
- [ch2](../ch2-option-result/README.md): "`?`가 에러를 `From`으로 변환한다"와 `Default`. [ch3](../ch3-strings-fmt/README.md): `from_utf8_lossy`가 돌려주는 `Cow<str>`.
- [ch4](../ch4-collections/README.md): 키 조건 `Eq + Hash`와 `Ord`를 `derive`로만 해결했습니다. 손으로 구현하는 방법과 규칙, `HashMap<String, V>`를 `&str`로 찾을 수 있는 이유(`Borrow`), `with_hasher`가 남았습니다.

이 챕터는 이 약속들을 한꺼번에 지킵니다. 트레이트를 하나씩 나열하는 대신 두 가지 질문을 따라갑니다. **이 트레이트를 구현하면 무엇이 따라오는가**, 그리고 **언제 `derive`로 충분하고 언제 손으로 구현해야 하는가**입니다.

- 패키지: `p1-ch6-core-traits`
- 기준 문서: [`std::convert`](https://doc.rust-lang.org/std/convert/index.html), [`std::ops`](https://doc.rust-lang.org/std/ops/index.html), [`std::cmp`](https://doc.rust-lang.org/std/cmp/index.html), [`std::hash`](https://doc.rust-lang.org/std/hash/index.html), [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html), [`std::borrow`](https://doc.rust-lang.org/std/borrow/index.html), [`std::marker`](https://doc.rust-lang.org/std/marker/index.html), [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `From`을 구현하면 `Into`(그리고 `TryFrom`·`TryInto`)가 포괄 구현으로 따라오는 원리와, `?`가 에러를 `From::from`으로 바꾼다는 것을 설명할 수 있다. 상황에 맞게 `as`, `From`, `TryFrom` 중 하나를 고르고, 직접 만든 타입에 `From`/`TryFrom`을 구현할 수 있다.
2. `AsRef`, `Into`, `Borrow`의 차이를 설명하고, 문자열이나 슬라이스를 받는 함수의 매개변수를 목적에 맞게 설계할 수 있다. `HashMap<String, V>`를 `&str`로 찾을 수 있는 이유를 `Borrow`의 약속으로 설명할 수 있다.
3. `ToOwned`와 `Cow`로 "고칠 것이 있을 때만 복제하는" 함수를 작성하고, `to_mut`과 `into_owned`가 언제 복제하는지 설명할 수 있다.
4. `std::ops`에서 연산자에 대응하는 트레이트를 찾아 산술·복합 대입·단항·비트 연산자와 `Index`/`IndexMut`를 내 타입에 구현할 수 있다.
5. `PartialEq`/`Eq`/`PartialOrd`/`Ord`/`Hash`가 서로 지켜야 할 성질을 설명하고, `derive`로 안 되는 경우(일부 필드만 비교, 대소문자 무시)에 다섯 트레이트를 어긋나지 않게 손으로 구현할 수 있다. `Default`, `Clone`, `Copy`도 `derive`로 충분한 경우와 손으로 구현할 경우를 구분할 수 있다.

## 2. 핵심 개념

2.1절은 챕터 전체의 지도입니다. 2.2~2.4절은 변환, 2.5~2.7절은 빌림과 소유, 2.8절은 연산자, 2.9~2.11절은 비교와 해시, 2.12~2.13절은 기본값과 복제를 다룹니다.

### 2.1 지도: 구현하면 따라오는 것, `derive`로 되는 것

트레이트 하나를 구현하면 메서드 하나만 생기는 것이 아닙니다. 표준 라이브러리의 포괄 구현([Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.11절)과, 그 트레이트를 바운드로 요구하는 API들이 함께 열립니다.

| 구현하는 것 | 따라오는 것 |
| --- | --- |
| `From<T> for U` | `Into<U> for T`, `TryFrom<T> for U`(에러 타입은 `Infallible`), `TryInto<U> for T`, `?`에서의 에러 변환 |
| `TryFrom<T> for U` | `TryInto<U> for T` |
| `AsRef<T>` | `impl AsRef<T>`를 받는 함수에 넘기기. `&내타입`도 자동으로 `AsRef<T>` |
| `Borrow<Q>` | `HashMap`/`HashSet`/`BTreeMap`에서 `&Q`로 찾기 |
| `Clone` | `ToOwned`(`Owned = Self`), 그래서 `Cow`에 담기. `derive(Clone)`한 바깥 타입 |
| `PartialEq` | `==`, `!=`, `contains`, `assert_eq!` |
| `PartialOrd` | `<`, `<=`, `>`, `>=` |
| `Ord` (+ `Eq`, `PartialOrd`) | `max`/`min`/`clamp`, `sort`, `binary_search`, `BTreeMap` 키, `BinaryHeap` 원소 |
| `Eq` + `Hash` | `HashMap`/`HashSet`의 키 |
| `Default` | `T::default()`, `..Default::default()`, `unwrap_or_default`, `or_default` |
| `Add` | `+` 만. `+=`는 `AddAssign`을 따로 구현해야 합니다 |

`derive`는 "필드마다 같은 트레이트를 불러 합친다"는 구현을 만들어 줍니다. 그 규칙이 원하는 뜻과 맞으면 `derive`로 충분하고, 맞지 않으면 손으로 구현합니다.

| 트레이트 | `derive`가 만드는 것 | 손으로 구현하는 대표적인 경우 |
| --- | --- | --- |
| `Clone` | 필드마다 `clone` | 거의 없음. 제네릭 타입에 붙는 불필요한 `T: Clone` 바운드를 피할 때 |
| `Copy` | 표시만(메서드 없음) | `Clone`과 같은 이유로 제네릭 바운드를 피할 때(드묾) |
| `PartialEq`, `Eq` | 모든 필드가 같으면 같음 | 일부 필드만 비교(ID만), 대소문자 무시 |
| `PartialOrd`, `Ord` | 필드 선언 순서대로 사전식 비교 | 내림차순, 일부 필드만, 필드 순서와 다른 기준 |
| `Hash` | 모든 필드를 차례로 해시 | `Eq`를 손으로 구현했을 때(반드시 함께) |
| `Default` | 필드마다 `default()` | 0·빈 문자열이 뜻에 맞지 않을 때(포트 8080 등) |
| `From`, `TryFrom`, `AsRef`, `Borrow`, 연산자 | `derive` 불가 | 항상 손으로 |

마지막으로, 상황별로 어느 트레이트를 쓰는지 모아 둡니다.

| 하고 싶은 일 | 쓸 것 | 절 |
| --- | --- | --- |
| 실패하지 않고 정보를 잃지 않는 변환 | `From` 구현, `.into()`로 사용 | 2.2 |
| 범위·형식 검사가 필요한 변환 | `TryFrom` 구현, `?`나 `match`로 처리 | 2.4 |
| 함수가 `&str`/`&[T]`를 읽기만 함 | `&str`, `&[T]` 또는 `impl AsRef<str>` | 2.5 |
| 함수가 받은 문자열을 저장함 | `impl Into<String>` | 2.5 |
| 맵에서 소유 키를 빌린 형태로 찾기 | `Borrow` | 2.6 |
| 대부분 그대로 돌려주고 가끔만 고침 | `Cow` 반환 | 2.7 |
| 순서·같음의 기준을 내가 정함 | `Ord` 먼저, 나머지를 맞춤 | 2.10 |

### 2.2 `From`과 `Into`: 한쪽만 구현하면 됩니다

```rust
pub trait From<T>: Sized { fn from(value: T) -> Self; }
pub trait Into<T>: Sized { fn into(self) -> T; }

impl<T, U> Into<U> for T where U: From<T> { /* U::from(self) */ } // 표준 라이브러리의 포괄 구현
```

`From<Minutes> for Seconds`를 구현하면 포괄 구현이 `Into<Seconds> for Minutes`를 만들어 줍니다. 그래서 `Seconds::from(m)`과 `m.into()` 두 가지로 쓸 수 있습니다. 반대로 `Into`만 구현하면 `From`은 생기지 않으므로, 문서와 clippy(`from_over_into`)는 **언제나 `From`을 구현하라**고 권합니다. `From`과 `Into`를 둘 다 구현하면 포괄 구현과 겹쳐 E0119입니다(4.1절).

```rust
impl From<Minutes> for Seconds {
    fn from(m: Minutes) -> Self { Seconds(m.0 * 60) }
}

let a = Seconds::from(Minutes(3));  // 목적 타입::from(값)
let b: Seconds = Hours(2).into();   // 값.into() — 목적 타입을 변수 타입에서 알아냄
```

**`into()`는 목적 타입을 알아야 합니다.** `"민수".into()`만 쓰면 `String`, `Box<str>`, `Vec<u8>`, `Box<dyn Error>` 중 무엇으로 바꿀지 정할 수 없어 E0283입니다(4.3절). 변수 타입, 함수 매개변수 타입, 반환 타입 같은 단서가 있어야 합니다.

**반사 구현.** 표준 라이브러리에는 `impl<T> From<T> for T`도 있습니다. 모든 타입이 자기 자신으로 변환된다는 뜻입니다. 쓸모없어 보이지만, 덕분에 `fn timer_label(d: impl Into<Seconds>)`가 `Minutes`, `Hours`뿐 아니라 `Seconds` 자신도 받습니다.

**고아 규칙과 `From`.** `impl From<Seconds> for u64`는 트레이트(`From`)도 구현 대상(`u64`)도 표준 라이브러리의 것이지만 허용됩니다. 타입 인자 자리에 내 타입 `Seconds`가 있기 때문입니다(Rust 1.41부터). 예전에 `Into`를 직접 구현해야 했던 이유가 이것이었고, 지금은 그럴 필요가 없습니다.

**언제 `From`을 구현하는가.** `From` 문서는 네 가지 기대를 적어 둡니다.

| 기대 | 뜻 | 예 |
| --- | --- | --- |
| 실패하지 않음 | 실패할 수 있으면 `TryFrom`. 패닉하는 `From`은 만들지 않음 | `i64: From<i32>` 있음, `i32: From<i64>` 없음 |
| 손실 없음 | 정보를 버리지 않음 | `f64: From<i32>` 있음, `f64: From<i64>` 없음(2.4절) |
| 값 보존 | 개념상 같은 값 | `String: From<char>` 있음, `String: From<u32>` 없음(숫자를 글로 바꾸는 것은 `Display`) |
| 명백함 | 그럴듯한 변환이 하나뿐 | 바이트 순서가 여럿인 `u32::from_le_bytes`는 `From`이 아니라 이름 있는 함수 |

`From`/`Into`는 소유권을 가져가는(값을 소비하는) 변환입니다. 참조만 빌려 보는 변환은 2.5절의 `AsRef`가 맡습니다.

### 2.3 `?`는 `From::from`을 부릅니다

`Result`를 돌려주는 함수에서 `expr?`는 대략 다음과 같이 펼쳐집니다. 실제로는 아직 불안정한 `Try` 트레이트를 거치지만 `Result`에서 하는 일은 같습니다.

```rust
match expr {
    Ok(v) => v,
    Err(e) => return Err(From::from(e)), // 함수의 에러 타입으로 바꿔서 돌려줌
}
```

그래서 반환 타입이 `Result<_, Box<dyn Error>>`인 함수에서 `ParseIntError`에 `?`를 쓸 수 있습니다. 표준 라이브러리에 "`Error`를 구현한 모든 타입 `E`에 대해 `Box<dyn Error>: From<E>`"라는 구현이 있기 때문입니다. `&str`과 `String`에서 `Box<dyn Error>`로 가는 `From`도 있어서 `Err("메시지".into())`가 됩니다. [`01_from_into.rs`](examples/01_from_into.rs)의 `parse_minutes`와 `?`를 손으로 펼친 `parse_minutes_by_hand`가 같은 결과를 냅니다.

반대로 `From` 구현이 없는 두 에러 타입 사이에서는 `?`가 E0277(`` `?` couldn't convert the error ``)로 거부됩니다([Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)의 4.3절). 직접 만든 에러 타입에 `From<io::Error>` 같은 구현을 붙여 `?`가 저절로 변환하게 만드는 설계와 `Error` 트레이트는 `ch8-errors-panic`에서 다룹니다.

### 2.4 `TryFrom`과 `as`: 무엇을 고를까

```rust
pub trait TryFrom<T>: Sized {
    type Error;                                         // 실패했을 때의 값 타입
    fn try_from(value: T) -> Result<Self, Self::Error>;
}
```

`TryFrom`을 구현하면 `TryInto`가 포괄 구현으로 따라옵니다. 또 `From`이 있는 모든 쌍에는 `TryFrom`도 자동으로 있습니다(`impl<T, U> TryFrom<U> for T where U: Into<T>`). 이때 `Error` 타입은 값이 하나도 없는 `Infallible`로, 실패할 수 없다는 뜻입니다. Rust 1.100.0에서 never 타입 `!`이 안정화되어 `Infallible`은 이제 `!`의 별칭이고, 문서는 앞으로 `!`를 직접 쓰라고 안내합니다. 이렇게 실패하지 않는 변환에 `try_from`을 쓰면 clippy가 `unnecessary_fallible_conversions`로 `From`을 쓰라고 알려 줍니다.

검증이 필요한 값을 만드는 경로를 `TryFrom` 하나로 두면 잘못된 값이 들어올 수 없습니다([Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)의 검증 타입과 같은 생각입니다).

```rust
impl TryFrom<i32> for Month {
    type Error = InvalidMonth;
    fn try_from(n: i32) -> Result<Self, Self::Error> {
        match n {
            1..=12 => Ok(Month(n as u8)), // 1..=12는 u8에 반드시 들어감
            _ => Err(InvalidMonth(n)),
        }
    }
}

let m: Result<Month, _> = 12.try_into(); // try_into도 목적 타입이 필요합니다.
```

**`as`, `From`, `TryFrom` 고르기.** [ch1](../ch1-std-overview/README.md)에서 본 것처럼 `as`는 실패를 알리지 않습니다. 정수끼리는 넘치는 비트를 버리고, 실수 → 정수는 소수점을 버린 뒤 범위 끝에 붙이며(`300.7 as u8`은 `255`), `NaN`은 `0`이 됩니다. 정수 → 실수는 가장 가까운 값으로 반올림합니다.

| 상황 | 고를 것 | 예제 02 |
| --- | --- | --- |
| 언제나 들어가는 변환(넓히기) | `From` / `into()` | `i64::from(x_i32)`, `usize::from(x_u16)` |
| 넘칠 수 있고, 넘치면 처리해야 함 | `TryFrom` / `try_into()` | `u8::try_from(300)` → `Err(..PosOverflow)` |
| 잘림·포화가 원하는 동작이거나 실수 → 정수 | `as` (필요하면 `round` 등을 먼저) | `x.round() as u16` |
| `const` 문맥 | `as` (`From::from`은 `const`에서 아직 부를 수 없음) | |

몇 가지는 직관과 다릅니다. **`usize: From<u32>`는 없습니다.** `usize`가 16비트인 플랫폼도 지원하기 때문이며, `u16`까지만 `From`이 있습니다. **`f64: From<i64>`도 없습니다.** `f64`는 2^53까지만 모든 정수를 정확히 담으므로, `(2^53 + 1) as f64`는 `9007199254740992`가 됩니다(예제 02의 3번 구역). 표준 라이브러리에는 실수 → 정수의 `From`/`TryFrom`도 없습니다. 반올림 방식을 호출자가 정하라는 뜻입니다. 글자 쪽에서는 `char::from(u8)`은 언제나 성공하지만 `u8::try_from('가')`와 `char::try_from(0xD800_u32)`(UTF-16용으로 예약된 자리)는 실패합니다.

### 2.5 `AsRef`/`AsMut`와 매개변수 설계

```rust
pub trait AsRef<T: ?Sized> { fn as_ref(&self) -> &T; }
pub trait AsMut<T: ?Sized> { fn as_mut(&mut self) -> &mut T; }
```

`AsRef<T>`는 "`&self`에서 `&T`를 **싸게** 얻을 수 있다"는 트레이트입니다. 새 값을 만들지 않고 참조만 바꾸므로 실패하지도, 비용이 들지도 않아야 합니다. 비용이 드는 변환이면 `From`을 씁니다. `String`, `&str`, `str`은 모두 `AsRef<str>`이고, `Vec<T>`, 배열, 슬라이스는 `AsRef<[T]>`입니다. 파일 경로를 받는 `std::fs` 함수들이 `P: AsRef<Path>`로 `&str`, `String`, `PathBuf`를 모두 받는 것도 같은 방법입니다(`ch9-io-fs-path`).

문자열을 받는 함수는 **그 문자열로 무엇을 하는지**에 따라 매개변수 타입을 고릅니다.

| 함수가 하는 일 | 매개변수 | 호출하는 쪽 |
| --- | --- | --- |
| 읽기만 함 | `&str` | `&String`도 역참조 강제 변환으로 들어감. 대부분 이것으로 충분 |
| 읽기만 하는데 `String`·`Cow` 등을 값으로도 받고 싶음 | `impl AsRef<str>` | `"a"`, `String`, `&String`, 직접 만든 타입 |
| 받아서 저장함 | `impl Into<String>` | `&str`이면 새로 할당, `String`이면 버퍼를 그대로 옮김 |

마지막 줄이 중요합니다. 저장하는 함수가 `&str`을 받으면, 이미 `String`을 가진 호출자도 함수 안에서 다시 복사됩니다. `impl Into<String>`은 `String`을 받으면 반사 구현으로 그대로 옮기므로 복사가 없습니다. 예제 01의 4번 구역이 `Member::new(owned)` 전후의 버퍼 주소를 비교해 `true`를 출력합니다.

**`?Sized`와 함께 쓰기.** 참조로 받는 `fn shout<S: AsRef<str>>(text: &S)`에 `"hello"`를 넘기면 `S = str`이 되는데, 타입 매개변수에는 보이지 않는 `Sized` 바운드가 붙어 있어 거부됩니다([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.15절). `S: AsRef<str> + ?Sized`로 풀어 줍니다(예제 03의 2번 구역).

**`AsRef`는 반사적이지 않습니다.** `From`과 달리 `impl<T> AsRef<T> for T`라는 포괄 구현이 없습니다(문서는 다른 포괄 구현과 겹쳐서라고 설명합니다). 대신 `&T`와 `&mut T`에 대한 포괄 구현이 있어서, 내 타입에 `AsRef<str>`를 구현하면 `&내타입`도 저절로 `AsRef<str>`가 됩니다. 예제 03의 `initial(&email)`이 그 경우입니다.

`AsMut`는 같은 일을 가변 참조로 합니다. 예제 03의 `wipe<T: AsMut<[u8]>>`는 `Vec<u8>`과 `[u8; 4]`를 모두 0으로 채웁니다.

### 2.6 `Borrow`: 빌린 값이 원래 값과 똑같이 행동한다는 약속

`Borrow<T>`의 메서드는 `AsRef<T>`와 모양이 같습니다(`fn borrow(&self) -> &T`). 다른 것은 **약속**입니다. 문서는 `Borrow`를 구현하는 타입에 "빌린 값의 `Eq`, `Ord`, `Hash` 결과가 원래 값의 결과와 같아야 한다"고 요구합니다. 즉 `x.borrow() == y.borrow()`는 `x == y`와 같은 답을 내야 합니다.

해시 맵이 이 약속에 기댑니다. `HashMap`의 `get` 시그니처는 다음과 같습니다(할당기 등은 생략).

```rust
impl<K, V, S> HashMap<K, V, S> {
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
}
```

`HashMap<String, u32>`에서 `scores.get("지영")`을 부르면 `K = String`, `Q = str`이고, `String: Borrow<str>`이므로 조건이 맞습니다. 맵은 키를 넣을 때 `String`의 해시로 자리를 정했고, 찾을 때는 `str`의 해시로 자리를 찾습니다. 둘이 같다는 것을 `Borrow`가 약속하므로 찾을 수 있습니다. 덕분에 찾을 때마다 `String`을 새로 만들 필요가 없습니다. `Vec<T>: Borrow<[T]>`, `Box<T>: Borrow<T>`도 있고, 모든 `T`는 `Borrow<T>`입니다.

직접 만든 키 타입에 `Borrow`를 구현해도 되는지는 이 약속으로 판단합니다.

| 타입 | `Borrow<str>` | 이유 (예제 03의 6번 구역) |
| --- | --- | --- |
| `struct UserId(String)` + `derive(PartialEq, Eq, Hash)` | 구현해도 됨 | `derive`가 안쪽 `String`의 결과를 그대로 쓰므로 `hash(UserId) == hash(str)`이 `true` |
| `struct Email { address: String, verified: bool }` | 구현하면 안 됨 | `Hash`가 두 필드를 모두 넣으므로 주소 문자열 하나의 해시와 다름(`false`) |
| 대소문자를 무시하는 `Tag(String)`(2.11절) | 구현하면 안 됨 | `"Rust"`와 `"rust"`가 `Tag`로는 같지만 `str`로는 다름 |

약속을 지킬 수 없는 타입이 안쪽 문자열을 내보이려면 아무것도 약속하지 않는 `AsRef<str>`를 구현합니다. 문서의 정리도 같습니다. 일반 코드가 "어떤 타입의 참조를 얻을 수 있기만 하면 된다"면 `AsRef`가, 비교·해시 결과의 일치에 기대면 `Borrow`가 맞습니다.

### 2.7 `ToOwned`와 `Cow`: 고칠 때만 복제하기

**`ToOwned`** 는 `Clone`을 일반화한 트레이트입니다. `Clone`은 `&T`에서 같은 타입 `T`만 만들 수 있어서 `&str` → `String`, `&[T]` → `Vec<T>`처럼 빌린 타입과 소유 타입이 다른 경우를 표현하지 못합니다. (`&str`에 `clone()`을 부르면 `&str`이 나옵니다.) `ToOwned`는 소유 타입을 연관 타입으로 둡니다.

```rust
pub trait ToOwned {
    type Owned: Borrow<Self>;            // str → String, [T] → Vec<T>, Clone인 T → T
    fn to_owned(&self) -> Self::Owned;
    fn clone_into(&self, target: &mut Self::Owned) { ... } // 기존 버퍼를 다시 쓰는 버전
}
```

`Clone`을 구현한 모든 타입은 포괄 구현으로 `ToOwned`(`Owned = Self`)가 됩니다.

**`Cow`**(clone-on-write, 쓰기 시 복제)는 "빌린 값 또는 소유한 값"을 담는 열거형입니다.

```rust
pub enum Cow<'a, B: ?Sized + ToOwned + 'a> {
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

`Cow<'a, str>`은 "`&'a str` 또는 `String`", `Cow<'a, [T]>`는 "`&'a [T]` 또는 `Vec<T>`"입니다. 쓰임새는 **대부분 입력을 그대로 돌려주고 가끔만 고치는 함수**입니다. 예제 04의 `escape_html`은 `<`, `>`, `&`가 없으면 `Cow::Borrowed(input)`을, 있을 때만 새 `String`을 만들어 `Cow::Owned`를 돌려줍니다. 댓글 6개 중 바꿀 것이 2개라면 할당도 2번뿐입니다. 언제나 `String`을 돌려주는 버전은 6번 할당합니다.

| 메서드·기능 | 동작 |
| --- | --- |
| `Deref` | `cow.len()`, `cow.starts_with(..)`처럼 안쪽 `&B`의 메서드를 그대로 씀 |
| `to_mut()` | `Borrowed`면 그때 한 번 복제해 `Owned`로 바꾼 뒤 `&mut Owned`를 줌. 이미 `Owned`면 복제 없음 |
| `into_owned()` | `Borrowed`면 복제, `Owned`면 안의 값을 그대로 꺼냄(복제 없음) |
| `Cow::from(..)` | `&str`/`String`/`&[T]`/`Vec<T>` 등에서 만들기 |
| `==` | `Cow<str> == &str` 등 비교 가능 |

`String::from_utf8_lossy`([ch3](../ch3-strings-fmt/README.md))가 `Cow<str>`을 돌려주는 이유도 같습니다. 올바른 UTF-8이면 원본을 빌리고, 대체 문자로 고칠 곳이 있을 때만 `String`을 만듭니다. 구조체 필드를 `Cow<'a, str>`로 두면 `&'static str` 기본값과 실행 중에 만든 `String`을 같은 필드에 담을 수도 있습니다.

주의할 점이 둘 있습니다. 첫째, `Cow`는 빌린 쪽의 라이프타임 `'a`를 지니므로 원본보다 오래 살 수 없습니다. 둘째, 함수 **매개변수**로 `&Cow<str>`을 받으면 clippy가 `ptr_arg`로 `&str`을 받으라고 권합니다. 읽기만 하는 함수에는 `&str`이 더 일반적이기 때문입니다(예제 04의 `is_borrowed`는 배리언트를 검사해야 해서 `#[expect]`로 예외를 둡니다). `Cow` 문서는 참조 카운팅 포인터에도 같은 기능(`Rc::make_mut`)이 있다고 안내합니다. `Rc`는 `ch7-smart-pointers-memory`에서 자세히 다룹니다.

### 2.8 연산자: `std::ops` 지도

Part 0 ch19에서 `Add`와 기본 타입 매개변수 `Rhs = Self`, 연관 타입 `Output`을 배웠습니다. 다른 연산자도 같은 틀이므로 "연산자 → 트레이트 → 메서드"의 대응만 알면 됩니다.

| 분류 | 연산자 → 트레이트 | 메서드 |
| --- | --- | --- |
| 산술 | `+` `-` `*` `/` `%` → `Add` `Sub` `Mul` `Div` `Rem` | `fn add(self, rhs: Rhs) -> Output` |
| 비트 | `&` `\|` `^` `<<` `>>` → `BitAnd` `BitOr` `BitXor` `Shl` `Shr` | `fn bitand(self, rhs: Rhs) -> Output` |
| 단항 | `-a` → `Neg`, `!a` → `Not` | `fn neg(self) -> Output` |
| 복합 대입 | `+=` `-=` … `\|=` `<<=` → `AddAssign` … `BitOrAssign` `ShlAssign` | `fn add_assign(&mut self, rhs: Rhs)` |
| 인덱스 | `a[i]` → `Index`, `a[i] = v` → `IndexMut` | `fn index(&self, i: Idx) -> &Output` |
| 역참조 | `*a` → `Deref`, `DerefMut` | [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) |
| 호출 | `f(x)` → `Fn`, `FnMut`, `FnOnce` | [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) |
| 정리 | 스코프 끝 → `Drop` | Part 0 ch14 |
| 범위 | `a..b`, `a..`, `..b`, `a..=b`, `..=b`, `..` → `Range` 등 구조체 | `RangeBounds` 트레이트 |
| 비교 | `==` `<` 등 | `std::ops`가 아니라 `std::cmp` (2.9절) |

`=`, `&&`, `||`는 오버로딩할 수 없습니다. `&&`/`||`는 오른쪽을 계산하지 않을 수도 있는(단락 평가) 연산이라 트레이트 메서드 하나로 표현할 수 없기 때문입니다. 새 연산자를 만드는 방법도 없습니다.

예제 05에서 확인할 점은 다음과 같습니다.

- **`Rhs`가 다른 타입인 경우와 순서.** `Won * 2`는 `impl Mul<i64> for Won`, `3 * Won(500)`은 `impl Mul<Won> for i64`입니다. 연산자 트레이트는 왼쪽 피연산자의 타입에 구현하므로 순서마다 따로 필요합니다. `Mul<Won> for i64`는 남의 타입 `i64`에 대한 구현이지만 타입 인자에 내 타입이 있어 고아 규칙이 허용합니다. 금액 × 금액처럼 뜻이 없는 조합은 구현하지 않아 컴파일러가 막게 둡니다.
- **`+`와 `+=`는 별개입니다.** `Add`만 구현하고 `a += b`를 쓰면 E0368(`` binary assignment operation `+=` cannot be applied to type `Won` ``)이고, 컴파일러가 `` an implementation of `AddAssign` might be missing ``이라고 알려 줍니다. 복합 대입은 `&mut self`로 제자리에서 고치므로 새 값을 만들지 않아도 됩니다.
- **비트 연산자로 플래그 만들기.** `Perm::READ | Perm::WRITE`, `perm & !Perm::WRITE`처럼 씁니다. `Not`을 구현할 때 쓰지 않는 위쪽 비트까지 켜지지 않도록 마스크를 씌운 것에 주목하세요. 문서는 연산자 구현이 "놀랍지 않아야" 한다고 권합니다. `*`라면 곱셈다운 성질(결합 법칙 등)을 지켜야 합니다.
- **`Index`는 참조를 돌려줍니다.** `board[(1, 1)]`은 `*board.index((1, 1))`의 줄임말이고, 대입하는 자리에서는 `*board.index_mut((1, 1))`이 됩니다. 그래서 `index`는 값이 아니라 `&Output`을 돌려줘야 합니다. 범위를 벗어난 인덱스는 보통 패닉으로 처리하며, 패닉하지 않는 조회가 필요하면 `get` 같은 메서드를 따로 둡니다.
- **범위도 값입니다.** `3..7`은 `std::ops::Range { start: 3, end: 7 }` 구조체입니다. 모든 범위 타입이 `RangeBounds`를 구현하므로 `impl RangeBounds<i32>`를 받으면 `..0`, `10..`, `3..=7`을 모두 받습니다. `Vec::drain(..)`과 `BTreeMap::range(..)`([ch4](../ch4-collections/README.md))가 이렇게 범위를 받습니다.

**참조에 대한 구현.** 연산자 메서드는 피연산자를 값으로 받습니다. `Won`처럼 `Copy`인 타입은 문제없지만, `Copy`가 아닌 타입이면 `a + b` 뒤에 `a`와 `b`를 다시 쓸 수 없습니다. 그래서 큰 값을 다루는 타입은 `impl Add<&T> for &T`처럼 참조에도 구현합니다. ch4의 `&band | &choir`(집합의 합집합)가 표준 라이브러리의 `impl BitOr<&BTreeSet<T>> for &BTreeSet<T>`입니다.

### 2.9 비교 트레이트 네 개의 관계

| 트레이트 | 연산자·메서드 | 뜻 |
| --- | --- | --- |
| `PartialEq<Rhs = Self>` | `==`, `!=` (`eq`, `ne`) | 같음. 대칭(`a == b`이면 `b == a`)과 추이(`a == b`, `b == c`이면 `a == c`) |
| `Eq: PartialEq` | 메서드 없음 | 여기에 반사(`a == a`)까지. 즉 **동치 관계(equivalence relation)** |
| `PartialOrd<Rhs = Self>: PartialEq<Rhs>` | `<` `<=` `>` `>=`, `partial_cmp` → `Option<Ordering>` | **부분 순서(partial order)**: 비교할 수 없는 쌍이 있을 수 있음(`None`) |
| `Ord: Eq + PartialOrd` | `cmp` → `Ordering`, `max`, `min`, `clamp` | **전순서(total order)**: 어떤 두 값이든 `<`, `==`, `>` 중 정확히 하나 |

"부분"이 붙은 쪽이 기본이고, 더 강한 약속을 하는 쪽이 그 위에 쌓입니다. 이렇게 나눈 이유는 `f64` 때문입니다. `NaN`은 자기 자신과도 같지 않으므로(`NaN == NaN`이 `false`) 반사가 깨져 `Eq`가 될 수 없고, `NaN`과 다른 수의 `partial_cmp`는 `None`이라 `Ord`도 될 수 없습니다. 그래서 `NaN < 1.0`, `NaN > 1.0`, `NaN >= 1.0`이 모두 `false`입니다(예제 06의 5번 구역). [ch4](../ch4-collections/README.md)에서 `Vec<f64>`에 `sort()`를 쓸 수 없던 이유입니다.

`Eq`에는 메서드가 없습니다. 반사성은 컴파일러가 확인할 수 없으므로, `Eq`는 "이 타입의 `==`는 동치 관계다"라는 **약속의 표시**입니다. 해시 맵이 `Eq`를 요구하는 것은 "키가 자기 자신과 같다"는 것을 믿어야 넣은 키를 다시 찾을 수 있기 때문입니다.

문서는 이 성질들이 깨지는 것을 **논리 오류(logic error)** 라고 부릅니다. 컴파일러가 막지 못하고, 미정의 동작으로 이어지지는 않지만, 정렬 결과가 이상해지거나 `BTreeMap`에서 넣은 키를 못 찾는 등 "결과가 정해지지 않은" 동작이 됩니다.

**`derive`의 규칙.** 구조체는 **필드 선언 순서대로 사전식**으로 비교합니다. 첫 필드가 같을 때만 다음 필드를 봅니다. 예제 06의 1번 구역은 같은 데이터를 담은 `ByName { name, score }`와 `ByScore { score, name }`이 정반대로 정렬되는 것을 보여 줍니다. 열거형은 먼저 선언한 배리언트가 작고(판별값을 `= 2`처럼 직접 적었다면 그 값 순서), 같은 배리언트끼리는 필드로 비교합니다. `derive`는 네 트레이트가 언제나 서로 맞도록 만들어 주므로, 기준이 "모든 필드를 선언 순서로"라면 `derive`가 가장 안전합니다.

### 2.10 손으로 구현하기: `Ord`를 먼저, 나머지를 맞춘다

기준이 필드 순서와 다르거나(내림차순, 이름보다 점수 먼저) 일부 필드를 무시해야 하면 손으로 구현합니다. `Ord` 문서가 권하는 순서는 다음과 같습니다.

1. **비교 논리는 `Ord::cmp` 한 곳에만** 씁니다.
2. `PartialOrd::partial_cmp`는 `Some(self.cmp(other))`로 위임합니다. 다른 모양으로 쓰면 clippy가 `non_canonical_partial_ord_impl`로 경고합니다.
3. `PartialEq::eq`는 **`cmp`가 보는 필드와 같은 필드**만 비교합니다. `Eq`는 빈 `impl`입니다.
4. 넷 중 일부만 `derive`하고 나머지를 손으로 쓰지 않습니다. `Ord`를 손으로 쓰고 `PartialOrd`를 `derive`하면 clippy의 `derive_ord_xor_partial_ord`가 **에러(기본 deny)** 로 막습니다(4.6절).

```rust
impl Ord for Player {               // 점수 내림차순, 같으면 이름 오름차순. joined는 무시
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.cmp(&self.score)
            .then_with(|| self.name.cmp(other.name))
    }
}
impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool { self.score == other.score && self.name == other.name }
}
impl Eq for Player {}
```

`PartialEq`에서 `joined`까지 비교하면 "`cmp`는 `Equal`인데 `==`는 `false`"인 쌍이 생겨 `PartialOrd`의 첫 조건(`a == b`이면 `partial_cmp(a, b) == Some(Equal)`)이 깨집니다.

**비교 기준이 곧 "같은 원소"의 기준입니다.** 예제 06의 2번 구역에서 `joined`만 다른 두 `Player`를 `BTreeSet`에 넣으면 두 번째 `insert`가 `false`를 돌려주고 원소는 1개입니다. 정렬만 하고 싶은데 같음의 뜻까지 바꾸고 싶지 않다면, 타입에 `Ord`를 구현하지 말고 `sort_by`/`sort_by_key`에 비교 함수를 넘기는 편이 낫습니다(ch4 2.8절).

**`Ordering` 다루기.** `cmp`의 결과 `Ordering`(`Less`, `Equal`, `Greater`)에는 다음 메서드가 있습니다.

| 메서드 | 동작 |
| --- | --- |
| `a.then(b)` | `a`가 `Equal`이면 `b`, 아니면 `a`. `b`는 미리 계산됨 |
| `a.then_with(\|\| ..)` | 같지만 클로저를 `a`가 `Equal`일 때만 부름 |
| `reverse()` | `Less` ↔ `Greater` |
| `is_lt()`, `is_eq()`, `is_ge()` 등 | `bool`로 확인 |

내림차순은 `other.x.cmp(&self.x)`처럼 자리를 바꾸거나 `.reverse()`를 붙입니다. 정렬 키의 일부만 뒤집을 때는 `std::cmp::Reverse`를 씁니다(ch4).

**`max`, `min`, `clamp`.** `Ord`의 제공 메서드입니다. 문서는 두 값이 같을 때 `max`는 **두 번째** 인자를, `min`은 **첫 번째** 인자를 돌려준다고 정해 둡니다. 예제 06에서 `joined`만 다른 두 선수에 `max`를 쓰면 가입 9번(두 번째), `min`은 가입 4번(첫 번째)이 나옵니다. `clamp(min, max)`는 범위 밖의 값을 끝값으로 붙이며, `min > max`이면 패닉합니다. `f64`는 `Ord`가 아니라서 `std::cmp::max(1.0, 2.0)`은 안 되고, 같은 이름의 `f64` 메서드(`2.5_f64.max(1.0)`, `1.7_f64.clamp(0.0, 1.0)`)를 씁니다. `if x > 100 { 100 } else if x < 0 { 0 } else { x }` 같은 코드는 clippy가 `manual_clamp`로 `clamp`를 권합니다.

**서로 다른 타입끼리의 `==`.** `PartialEq<Rhs>`의 `Rhs`를 바꾸면 됩니다. 표준 라이브러리에는 `String == &str`, `Vec<T> == [T; N]` 같은 구현이 있고, 예제 06은 `impl PartialEq<&str> for Code`로 `code == "A-100"`을 씁니다. 문서는 이런 구현을 **내 타입 쪽에만** 두라고 권합니다. `impl PartialEq<Code> for &str`처럼 남의 타입에 구현하면, 여러 크레이트의 구현이 엮여 추이성이 깨질 수 있기 때문입니다. 그래서 `"A-100" == code`는 컴파일되지 않습니다.

### 2.11 해시: `Hash`, `Hasher`, `BuildHasher`

해시 맵에는 세 역할이 있습니다.

| 트레이트 | 누가 구현하나 | 하는 일 |
| --- | --- | --- |
| `Hash` | 키 타입 | `fn hash<H: Hasher>(&self, state: &mut H)`: 어떤 바이트들을 해셔에 넣을지 정함 |
| `Hasher` | 해시 알고리즘 | `write(&[u8])`로 바이트를 받고 `finish()`로 `u64` 해시를 냄 |
| `BuildHasher` | 해셔 공장 | 키마다 새 `Hasher`를 만듦. `HashMap<K, V, S>`의 `S`(기본값 `RandomState`) |

해셔(hasher)는 상태를 가지므로 키마다 새로 만들어야 하고, 그래서 공장이 따로 있습니다. 해시 값 하나를 직접 계산할 때는 `BuildHasher::hash_one(&값)`을 씁니다.

**규칙: `a == b`이면 `hash(a) == hash(b)`.** `HashMap`은 해시로 찾을 자리를 정하고, 그 자리에서 `==`로 키를 가려냅니다. 같은 키의 해시가 다르면 맵은 엉뚱한 자리를 뒤져 넣은 키를 못 찾거나 같은 키를 두 번 넣습니다. 반대 방향(해시가 같으면 같은 값)은 요구하지 않습니다. 해시는 충돌할 수 있고, 그때는 `==`가 가려냅니다.

`derive(PartialEq, Eq, Hash)`는 모든 필드를 같은 방식으로 다루므로 규칙이 저절로 지켜집니다. 문제는 **`Eq`를 손으로 구현했을 때**입니다. `Eq`가 무시하는 정보를 `Hash`가 넣으면 규칙이 깨지므로, `Hash`도 같은 기준으로 손으로 구현합니다.

```rust
impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool { self.0.eq_ignore_ascii_case(&other.0) }
}
impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.0.len());                 // 경계 표시(아래 참고)
        for b in self.0.bytes() { state.write_u8(b.to_ascii_lowercase()); } // eq와 같은 기준
    }
}
```

- `Hash`는 `Eq`가 보는 정보의 **일부만** 넣어도 규칙이 지켜집니다(같은 값이면 그 일부도 같으므로). 반대로 `Eq`가 무시하는 정보를 넣으면 깨집니다. 예를 들어 `Eq`가 ID만 비교하면 `Hash`도 ID만 넣습니다.
- `derive(Hash)`와 손으로 쓴 `PartialEq`를 섞으면 clippy의 `derived_hash_with_manual_eq`가 **에러(기본 deny)** 로 막습니다. 예제 07의 `BadTag`는 무슨 일이 생기는지 보이려고 `#[expect]`로 이 lint를 허용했고, `a == b`는 `true`인데 해시는 다릅니다.
- **경계가 섞이지 않게(prefix-free).** 여러 값을 이어 해시할 때 `("ab", "c")`와 `("a", "bc")`가 같은 바이트열이 되면 일부러 충돌을 만드는 셈입니다. `&str`의 `Hash`는 끝에 구분 바이트(`0xFF`)를 하나 더 넣어 이것을 막고, 예제 07의 `Tag`는 길이를 먼저 넣습니다.
- **해시 값을 코드에 적어 두지 않습니다.** `RandomState`는 맵마다 시드가 다르고, `DefaultHasher`의 알고리즘과 표준 타입이 해셔에 넣는 바이트도 버전 사이에 보장되지 않습니다. 문서는 테스트에서도 해시 값 대신 `Eq`와의 일치를 확인하라고 권합니다. 이 챕터의 예제가 "해시가 같은가"만 출력하는 이유입니다.

**맵에 들어간 키를 바꾸면 안 되는 이유.** 키의 해시(또는 순서)가 맵에 넣은 뒤에 바뀌면, 키는 옛 해시로 정한 자리에 그대로 있는데 찾을 때는 새 해시로 자리를 계산하므로 찾을 수 없습니다. `HashMap` 문서는 이것을 논리 오류라고 못 박습니다. 그래서 `HashMap`은 키의 `&mut`를 내주지 않습니다(`iter_mut`도 `(&K, &mut V)`). 문서는 그래도 `Cell`, `RefCell`, 전역 상태 등으로 키 내부를 고칠 수 있다고 경고합니다. 이런 내부 가변성 타입은 `ch7-smart-pointers-memory`에서 다룹니다. 키를 바꿔야 하면 `remove`한 뒤 고쳐서 다시 `insert`합니다. `BTreeMap`도 순서에 대해 같은 규칙이 있습니다.

**`with_hasher`로 해시 함수 바꾸기.** 기본 해셔는 HashDoS 공격을 막는 SipHash 1-3이지만(ch4 2.11절), 작은 키에서는 가장 빠른 알고리즘이 아닙니다. `Hasher`를 직접 구현하거나 crates.io의 해셔를 끼울 수 있습니다. 예제 07은 짧은 FNV-1a 알고리즘을 구현했습니다.

```rust
impl Hasher for Fnv1a {
    fn write(&mut self, bytes: &[u8]) { /* 바이트마다 xor 후 곱셈 */ }
    fn finish(&self) -> u64 { self.0 }
}
type FnvBuildHasher = BuildHasherDefault<Fnv1a>;   // Default인 Hasher를 BuildHasher로 감쌈

let mut stock: HashMap<&str, u32, FnvBuildHasher> = HashMap::with_hasher(FnvBuildHasher::default());
```

맵의 타입에 세 번째 매개변수 `S`가 붙는 것 말고는 사용법이 같습니다. `S: Default`이면 `HashMap::default()`로도 만듭니다(`new()`는 `RandomState` 전용입니다). 시드가 없는 해셔를 쓰면 같은 순서로 넣은 두 맵의 순회 순서가 같아지지만, 공격자가 충돌하는 키를 골라 보낼 수 있으므로 외부 입력을 키로 쓰는 맵에는 기본값을 두는 것이 안전합니다.

### 2.12 `Default`: 쓸모 있는 기본값

```rust
pub trait Default: Sized { fn default() -> Self; }
```

정수는 `0`, `bool`은 `false`, `String`·`Vec`·`HashMap`은 비어 있는 값, `Option`은 `None`이 기본값입니다. 12개 이하의 원소를 가진 튜플도 원소가 모두 `Default`이면 `Default`입니다.

| 쓰는 법 | 예 (예제 08) |
| --- | --- |
| `#[derive(Default)]` | 필드마다 기본값. `SearchOptions { query: "", limit: 0, .. }` |
| 손으로 구현 | 필드의 기본값이 뜻에 맞지 않을 때. 포트 `0` 대신 `8080` |
| 열거형 `#[default]` | `enum Theme { #[default] System, Light, Dark }`. 데이터 없는 배리언트에만 붙일 수 있음 |
| 구조체 갱신 문법 | `ServerConfig { port: 3000, ..Default::default() }` |
| 바운드로 쓰기 | `fn reset<T: Default>(v: &mut T) { *v = T::default(); }` |
| 표준 API | `unwrap_or_default()`, `entry(k).or_default()`(ch4), ch2의 `map_or_default` |

손으로 구현한 `Default`가 `derive`로 만들 수 있는 것과 같으면 clippy가 `derivable_impls`로 `derive`를 권합니다. 필드에 기본값을 직접 적는 문법(`struct S { port: u16 = 8080 }`)은 1.100.0에서 아직 불안정 기능이라 E0658입니다.

### 2.13 `Clone`과 `Copy`, 그리고 `Sized`

| | `Copy` | `Clone` |
| --- | --- | --- |
| 언제 | 대입·전달할 때 **암묵적으로** | `.clone()`을 **명시적으로** 부를 때 |
| 무엇을 | 비트를 그대로 복사. 동작을 바꿀 수 없음 | 타입이 정한 대로. 임의의 코드를 실행할 수 있음 |
| 비용 | 언제나 싸다(크기만큼 복사) | 쌀 수도 비쌀 수도 있음 |
| 관계 | `trait Copy: Clone {}` — `Copy`이면 반드시 `Clone` | |

[Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md)에서 `Copy` 타입은 대입해도 원본을 계속 쓸 수 있다고 배웠습니다. 사실 이동도 복사도 메모리에서는 비트를 옮기는 같은 일입니다. 차이는 **원본을 계속 쓸 수 있는가**뿐입니다.

**`Copy`가 될 수 없는 타입.** 문서는 두 경우를 듭니다.

- **`Copy`가 아닌 필드가 있는 타입**(E0204, 4.4절). `String`이나 `Vec`은 힙 버퍼를 소유하므로, 비트만 복사하면 두 값이 같은 버퍼를 가리키고 둘 다 해제하려 합니다(이중 해제). `&T`는 `Copy`이지만 `&mut T`는 아닙니다. 가변 참조를 복사하면 같은 값을 가리키는 `&mut`가 둘이 되기 때문입니다.
- **`Drop`을 구현한 타입**(E0184, 4.5절). `Drop`이 있다는 것은 "자기 크기 밖의 무언가를 관리한다"는 뜻이라, 복사본마다 정리 작업이 실행되면 안 됩니다.

**`Copy`는 공개 약속입니다.** 문서는 "구현할 수 있으면 보통 구현하라"고 하면서도, 나중에 `Copy`가 아닌 필드를 추가하면 `Copy`를 빼야 하고 그것은 쓰는 쪽 코드를 깨는 변경이라고 경고합니다.

**`Clone`의 뜻은 타입마다 다릅니다.** 대부분은 독립된 깊은 복사지만, `&T`의 `clone`은 같은 값을 가리키는 참조를 하나 더 만들고, `Rc`/`Arc`의 `clone`은 데이터를 복사하지 않고 참조 카운트만 늘립니다(예제 08의 5번 구역). 그래서 `Rc`를 필드로 가진 구조체를 `derive(Clone)`하면 복제본이 데이터를 공유합니다. `Copy`인 타입의 `Clone`은 `*self`와 같아야 하며, 다르게 쓰면 clippy가 `non_canonical_clone_impl`로 경고합니다. `Copy` 값에 `.clone()`을 부르면 `clone_on_copy` 경고가 납니다.

**`clone_from`.** `a.clone_from(&b)`는 `a = b.clone()`과 결과가 같지만, `a`가 가진 자원(버퍼)을 다시 쓸 수 있습니다. `String`과 `Vec`의 문서는 "가능하면 재할당을 피한다"고 적고 있고, 예제 08의 6번 구역에서 용량 64인 버퍼에 세 줄을 차례로 `clone_from`하는 동안 버퍼 주소가 그대로였습니다(이 툴체인에서 확인). 반복문 안에서 같은 변수에 계속 복제해 넣을 때 쓸모가 있습니다. `ToOwned`에는 같은 역할의 `clone_into`가 있습니다(예제 04의 6번 구역).

**`derive`와 제네릭 바운드.** `#[derive(Clone)] struct Wrapper<T>`는 `impl<T: Clone> Clone for Wrapper<T>`를 만듭니다. 필드가 `Option<T>`나 `&T`처럼 `T`가 `Clone`이 아니어도 복제할 수 있는 모양이어도 바운드가 붙습니다. `Default`도 같아서, `#[derive(Default)] struct Wrapper<T> { inner: Option<T> }`에 `Default`가 없는 `T`를 넣으면 컴파일러가 `` consider manually implementing `Default` to avoid undesired bounds caused by "imperfect derives" ``라고 안내합니다. 이럴 때 손으로 구현합니다.

**`std::marker`의 나머지.** `Copy`와 함께 `std::marker`에 있는 `Sized`는 "크기를 컴파일할 때 아는 타입"이라는 표시로, 타입 매개변수에 자동으로 붙고 `?Sized`로 풉니다(Part 0 ch19의 2.15절, 이 챕터 2.5절의 `shout`). `Send`/`Sync`는 [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md)에서 다뤘고 `ch11-threads-sync`에서 다시 봅니다. `Unpin`은 `ch13-async-primitives`, `PhantomData`는 Part 10 `ch4-variance-dropck`에서 다룹니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_from_into.rs`](examples/01_from_into.rs) | `From` 구현과 따라오는 `Into`, 표준 `From`, `impl Into<_>` 매개변수, `?`와 `From::from` | 1, 2 |
| [`examples/02_try_from_as.rs`](examples/02_try_from_as.rs) | `as`와 `try_from` 비교, 실수·`usize`·`char` 변환, 검증 타입에 `TryFrom` 구현 | 1 |
| [`examples/03_as_ref_borrow.rs`](examples/03_as_ref_borrow.rs) | `AsRef`/`AsMut` 매개변수, `?Sized`, `Borrow`로 맵 조회, `Borrow`의 약속 확인 | 2 |
| [`examples/04_cow.rs`](examples/04_cow.rs) | `from_utf8_lossy`, `Cow` 반환 함수와 할당 횟수, `to_mut`, `into_owned`, `ToOwned` | 3 |
| [`examples/05_operators.rs`](examples/05_operators.rs) | 산술·단항·복합 대입, 비트 연산자 플래그, `Index`/`IndexMut`, 범위 타입 | 4 |
| [`examples/06_eq_ord.rs`](examples/06_eq_ord.rs) | `derive`의 필드 순서, 손으로 구현한 `Ord`, `Ordering`, `max`/`min`/`clamp`, `NaN`, 다른 타입끼리 `==` | 5 |
| [`examples/07_hash.rs`](examples/07_hash.rs) | 대소문자를 무시하는 `Eq`+`Hash`, 규칙을 어긴 경우, prefix-free, `with_hasher` | 2, 5 |
| [`examples/08_default_clone_copy.rs`](examples/08_default_clone_copy.rs) | `Default`의 여러 쓰임, `Copy`와 `Clone`, `Rc`의 `clone`, `clone_from` | 5 |
| [`compile_fail/`](compile_fail) | 변환과 `Copy`에서 자주 만나는 컴파일 에러 5개 (4절) | 1, 5 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 패닉 없이 끝나고 종료 코드는 0입니다. 해시 값과 메모리 주소는 실행마다 달라지므로 출력하지 않고 "같은가"만 `true`/`false`로 출력합니다. 그래서 출력은 실행할 때마다 같습니다.

### 01_from_into — `From`과 `Into`

```sh
cargo run -p p1-ch6-core-traits --example 01_from_into
```

```text
== 1. From과 Into ==
Seconds(180) Seconds(7200)
u64로: 7200

== 2. 표준 라이브러리의 From ==
-3 44032 'A'
"한" [1, 2, 3] Some(5)

== 3. impl Into<Seconds> 매개변수 ==
45초 타이머
300초 타이머
3600초 타이머

== 4. impl Into<String> 매개변수 ==
민수, 지영
같은 버퍼인가? true

== 5. ?는 From::from을 부릅니다 ==
"2" → Seconds(120)
"0" → 에러: 0분 타이머는 만들 수 없습니다
"두 시간" → 에러: invalid digit found in string
손으로 푼 버전: Err("invalid digit found in string")
```

3번에서 `Seconds(45)`가 받아지는 것은 반사 구현 `From<T> for T` 덕분입니다. 5번의 두 에러는 서로 다른 타입(`&str`, `ParseIntError`)에서 출발했지만 모두 `From`으로 `Box<dyn Error>`가 되었습니다.

### 02_try_from_as — 실패할 수 있는 변환

```sh
cargo run -p p1-ch6-core-traits --example 02_try_from_as
```

```text
== 1. as와 try_from ==
 200: as u8 = 200, u8::try_from = Ok(200)
 300: as u8 =  44, u8::try_from = Err(TryFromIntError(PosOverflow))
  -1: as u8 = 255, u8::try_from = Err(TryFromIntError(NegOverflow))

== 2. 실수 → 정수의 as ==
255 0 0
반올림 후 변환: 301

== 3. 정수 → 실수 ==
16777217
9007199254740993 as f64 = 9007199254740992

== 4. usize ==
500 Ok(70000)

== 5. 검증 타입 Month ==
2월: 28일까지
9월: 30일까지
13: 거절 InvalidMonth(13)
0: 거절 InvalidMonth(0)
12.try_into() = Ok(Month(12))

== 6. char와 TryFrom ==
Ok(65)
Err(TryFromCharError(()))
Ok('가')
0xD800: converted integer out of range for `char`
```

`TryFromIntError`의 디버그 형식에 보이는 `PosOverflow`/`NegOverflow`는 내부 표현이며, 기본 형식(`{}`)으로는 `number too large to fit in target type` 같은 문장이 나옵니다.

### 03_as_ref_borrow — `AsRef`와 `Borrow`

```sh
cargo run -p p1-ch6-core-traits --example 03_as_ref_borrow
```

```text
== 1. AsRef<str> ==
Some('민')
Some('지')
Some('지')
Some('m')
인증됨? true

== 2. ?Sized와 함께 ==
HELLO! RUST!

== 3. 슬라이스로 보기 ==
5500 3000 4300
[0, 0, 0, 0] [0, 0, 0, 0]

== 4. Borrow와 해시 맵 조회 ==
지영: Some(85)

== 5. 직접 만든 키와 Borrow ==
u-002: Some("11-18")

== 6. Borrow의 약속 확인 ==
hash(UserId) == hash(str)? true
hash(Email) == hash(주소)? false
```

6번은 `RandomState`로 해시를 계산하므로 해시 값 자체는 실행마다 다르지만 비교 결과는 같습니다. `Email`의 해시가 주소의 해시와 우연히 같을 확률은 사실상 0입니다.

### 04_cow — 고칠 때만 복제하기

```sh
cargo run -p p1-ch6-core-traits --example 04_cow
```

```text
== 1. from_utf8_lossy ==
안녕 빌림? true
안� 빌림? false

== 2. 필요할 때만 할당하기 ==
바뀜: a < b 이면 참 → a &lt; b 이면 참
바뀜: Tom & Jerry → Tom &amp; Jerry
String 버전: 6번 할당, Cow 버전: 2번 할당

== 3. Deref와 비교 ==
15 true true

== 4. to_mut ==
[3, 5, 2] 빌림? true
[4, 0, 7, 0] 빌림? false
원본은 그대로: [4, -1, 7, -3]

== 5. into_owned ==
&lt;b&gt; 같은 버퍼인가? true

== 6. ToOwned ==
러스트 [1, 2, 3] 새 내용
```

4번의 `[4, -1, 7, -3]`에는 음수가 두 개지만 복제는 첫 `to_mut()`에서 한 번만 일어납니다. 두 번째부터는 이미 `Owned`이기 때문입니다.

### 05_operators — 연산자 트레이트

```sh
cargo run -p p1-ch6-core-traits --example 05_operators
```

```text
== 1. 산술 연산자 ==
주문: 15000원
결제: 15500원
환불: -6000원, 정산: 9500원

== 2. 비트 연산자로 만든 권한 ==
rw- 쓰기 가능? true
rwx
r-x true
---

== 3. Index와 IndexMut ==
..O
.X.
...

== 4. 범위도 값입니다 ==
1~12, 12 포함? true
true
7 in 3..7: false
7 in 3..=7: true
-5 in ..0: true
100 in 10..: true
```

### 06_eq_ord — 비교 트레이트

```sh
cargo run -p p1-ch6-core-traits --example 06_eq_ord
```

```text
== 1. derive와 필드 순서 ==
이름 먼저: ["민수(95)", "지영(70)"]
점수 먼저: ["지영(70)", "민수(95)"]
Some(High) true

== 2. 손으로 구현한 Ord ==
1위 민수 95점 (가입 1번)
2위 가온 80점 (가입 2번)
3위 하늘 80점 (가입 3번)
첫 insert true, 둘째 insert false, 원소 1개
p1 == p2? true
max → 가입 9번, min → 가입 4번

== 3. Ordering 다루기 ==
Less Greater true
Greater
Less

== 4. max, min, clamp ==
8 100
2.5 1

== 5. 부분 순서: f64와 NaN ==
Some(Less) None
false false false
NaN == NaN? false

== 6. 서로 다른 타입끼리 == ==
true
true true
```

2번에서 80점인 "가온"과 "하늘"은 이름 오름차순으로 놓였습니다. 3번의 `then`은 앞 비교가 `Equal`일 때만 뒤 비교의 결과를 씁니다.

### 07_hash — 해시

```sh
cargo run -p p1-ch6-core-traits --example 07_hash
```

```text
== 1. derive한 Hash ==
p == q? true, 해시 같음? true

== 2. 대소문자를 무시하는 키 ==
입력 6개 → 서로 다른 태그 3개
["CLI", "Rust", "WASM"]
wAsM이 있나? true

== 3. 규칙을 어기면 ==
a == b? true
해시 같음? false

== 4. 경계가 섞이지 않게 ==
("ab", "c")와 ("a", "bc")의 해시가 같은가? false

== 5. with_hasher로 해시 함수 바꾸기 ==
사과 Some(15), 귤 None
FNV 맵 두 개의 순회 순서가 같은가? true
```

2번에서 남은 표기가 `"Rust"`, `"WASM"`인 것은 이미 같은 값이 있으면 `insert`가 원래 값을 바꾸지 않기 때문입니다. 3번의 `BadTag`를 실제로 `HashSet`에 넣으면 `"Rust"`와 `"rust"`가 둘 다 들어갈 수도, 하나만 들어갈 수도 있습니다. 해시가 다른 두 키가 맵 안에서 우연히 같은 자리 후보에 놓이는지에 따라 달라지므로, 결과가 실행마다 바뀔 수 있어 예제에서는 출력하지 않았습니다.

### 08_default_clone_copy — 기본값과 복제

```sh
cargo run -p p1-ch6-core-traits --example 08_default_clone_copy
```

```text
== 1. derive(Default) ==
SearchOptions { query: "", limit: 0, case_sensitive: false, tags: [], page: None }
러스트 20 false [] None

== 2. 손으로 구현한 Default와 #[default] ==
127.0.0.1:3000 작업자 4명
System false

== 3. Default를 쓰는 표준 API ==
"" []
0
Some(["민수"])

== 4. Copy: 대입하면 비트가 복사되고 원본도 그대로 씁니다 ==
Pixel { x: 3, y: 4 } Pixel { x: 100, y: 4 }

== 5. Clone: 명시적으로 부르고, 무엇을 복사할지는 타입이 정합니다 ==
출근길 ["노래1", "노래2"]
출근길 (편집) ["노래1", "노래2", "노래3"]
같은 데이터? true, 강한 참조 2개

== 6. clone_from: 이미 있는 버퍼를 다시 씁니다 ==
셋째 / 같은 버퍼를 계속 썼나? true
첫 줄 / 새 버퍼인가? true
```

6번의 첫 줄(`clone_from`이 버퍼를 재사용함)은 문서가 "가능하면"이라고만 약속하는 동작을 이 툴체인에서 확인한 결과입니다. 둘째 줄은 언제나 `true`입니다. `lines[0].clone()`이 새 버퍼를 만드는 동안 `fresh`의 원래 버퍼가 아직 살아 있으므로 두 주소가 같을 수 없습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch6-core-traits
rustc --edition 2024 compile_fail/e0119_into_with_from.rs --out-dir /tmp/rust-p1-ch6
```

### 4.1 `From`과 `Into`를 둘 다 구현 — E0119

파일: [`compile_fail/e0119_into_with_from.rs`](compile_fail/e0119_into_with_from.rs)

```text
error[E0119]: conflicting implementations of trait `Into<Seconds>` for type `Minutes`
  --> compile_fail/e0119_into_with_from.rs:21:1
   |
21 | impl Into<Seconds> for Minutes {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: conflicting implementation in crate `core`:
           - impl<T, U> Into<U> for T
             where U: From<T>;
```

- **읽는 법**: "`Minutes`에 대한 `Into<Seconds>` 구현이 둘이라 충돌한다"는 뜻입니다. `note:`가 다른 하나가 `core`의 포괄 구현이라고 알려 줍니다.
- **원인**: `From<Minutes> for Seconds`를 구현한 순간 포괄 구현이 `Into<Seconds> for Minutes`를 이미 만들었습니다. 한 (트레이트, 타입) 쌍에는 구현이 하나만 있어야 합니다(일관성, Part 0 ch10 2.7절).
- **해결**: `Into` 구현을 지웁니다. `From` 하나로 `into()`도 쓸 수 있습니다. `Into`만 구현했다면 컴파일은 되지만 clippy가 `from_over_into`(`` an implementation of `From` is preferred since it gives you `Into<_>` for free where the reverse isn't true ``)로 `From`으로 바꾸라고 권합니다.

### 4.2 범위가 좁아지는 변환에 `into()` — E0277

파일: [`compile_fail/e0277_into_narrowing.rs`](compile_fail/e0277_into_narrowing.rs)

```text
error[E0277]: the trait bound `u8: From<i32>` is not satisfied
  --> compile_fail/e0277_into_narrowing.rs:13:27
   |
13 |     let level: u8 = total.into();
   |                           ^^^^ the trait `From<i32>` is not implemented for `u8`
   |
help: `u8` implements trait `From<T>`
   (표준 라이브러리 소스 위치 생략)
   = note: `From<bool>`
   = note: `From<std::ascii::Char>`
   = note: required for `i32` to implement `Into<u8>`
```

- **읽는 법**: 마지막 `note:`가 핵심입니다. `i32: Into<u8>`가 필요했는데, 그것은 `u8: From<i32>`가 있어야 생기는 구현이라는 뜻입니다. `help:`는 `u8`이 구현한 `From`의 예를 보여 줍니다(`std::ascii::Char`는 아직 불안정한 타입입니다).
- **원인**: `i32`의 값은 `u8`에 들어가지 않을 수 있으므로 `From`이 없습니다(2.2절의 "실패하지 않음").
- **해결**: `u8::try_from(total)`로 바꾸고 `Err`를 처리합니다. 넘칠 때 255로 붙이고 싶다면 `total.clamp(0, 255) as u8`처럼 의도를 드러냅니다.

### 4.3 `into()`의 목적 타입을 알 수 없음 — E0283

파일: [`compile_fail/e0283_into_unknown_target.rs`](compile_fail/e0283_into_unknown_target.rs)

```text
error[E0283]: type annotations needed
  --> compile_fail/e0283_into_unknown_target.rs:12:9
   |
12 |     let name = "민수".into();
   |         ^^^^          ---- type must be known at this point
   |
   = note: the type must implement `From<&str>`
   = note: required for `&str` to implement `Into<_>`
help: consider giving `name` an explicit type
   |
12 |     let name: /* Type */ = "민수".into();
   |             ++++++++++++
```

- **읽는 법**: `Into<_>`의 `_`가 정해지지 않았다는 뜻입니다. `From<&str>`를 구현한 타입이 여럿이라 하나를 고를 수 없습니다.
- **원인**: `println!("{name}")`은 `Display`이기만 하면 되므로 타입의 단서가 되지 못합니다. 이후에 `name.len()`처럼 메서드를 부르면 같은 상황이 E0282(``type annotations needed``)로 보고됩니다. 변수 위치는 같고 코드만 다릅니다.
- **해결**: `let name: String = "민수".into();`처럼 타입을 적거나, 처음부터 `String::from("민수")`라고 씁니다. 목적 타입이 분명하지 않은 곳에서는 `T::from(x)` 형태가 더 읽기 쉽습니다.

### 4.4 `Copy`가 아닌 필드를 가진 타입에 `derive(Copy)` — E0204

파일: [`compile_fail/e0204_copy_with_string_field.rs`](compile_fail/e0204_copy_with_string_field.rs)

```text
error[E0204]: the trait `Copy` cannot be implemented for this type
  --> compile_fail/e0204_copy_with_string_field.rs:12:8
   |
11 | #[derive(Debug, Clone, Copy)]
   |                        ---- in this derive macro expansion
12 | struct Member {
   |        ^^^^^^
13 |     id: u32,
14 |     name: String,
   |     ------------ this field does not implement `Copy`
```

- **읽는 법**: 어느 필드가 걸림돌인지 `this field does not implement Copy`가 짚어 줍니다.
- **원인**: `String`은 힙 버퍼를 소유하므로 비트 복사로 복제할 수 없습니다(2.13절).
- **해결**: `Copy`를 빼고 필요한 곳에서 `.clone()`을 부릅니다. 자주 복사해야 하는 작은 값이라면 이름 대신 정수 ID를 담는 식으로 필드를 `Copy` 타입으로 바꿀 수 있는지 생각해 봅니다.

### 4.5 `Drop`을 구현한 타입에 `Copy` — E0184

파일: [`compile_fail/e0184_copy_with_drop.rs`](compile_fail/e0184_copy_with_drop.rs)

```text
error[E0184]: the trait `Copy` cannot be implemented for this type; the type has a destructor
  --> compile_fail/e0184_copy_with_drop.rs:11:8
   |
10 | #[derive(Clone, Copy)]
   |                 ---- in this derive macro expansion
11 | struct Ticket {
   |        ^^^^^^ `Copy` not allowed on types with destructors
   |
note: destructor declared here
  --> compile_fail/e0184_copy_with_drop.rs:16:5
```

- **읽는 법**: 필드는 모두 `Copy`(`u32`)라서 E0204는 아닙니다. 걸림돌은 `note:`가 가리키는 `drop`입니다.
- **원인**: 대입할 때마다 몰래 복사본이 생기면 복사본마다 `drop`(표 반납)이 실행됩니다. 한 번만 해야 할 정리를 여러 번 하게 되므로 금지됩니다.
- **해결**: `Copy`를 빼고 `Clone`만 남깁니다. 복제가 일어나는 곳이 코드에 `.clone()`으로 드러납니다.

### 4.6 clippy가 막는 비교·해시·복제 구현

다음은 컴파일은 되지만 clippy가 기본 설정에서 잡는 실수입니다. 모두 2.10~2.13절의 규칙을 어긴 경우이며, 메시지는 `#[expect]` 없이 clippy 1.100.0을 실행해 확인했습니다.

| 코드 | clippy 메시지 (lint, 기본 수준) | 바로잡기 |
| --- | --- | --- |
| `#[derive(Hash)]` + 손으로 쓴 `impl PartialEq` | ``you are deriving `Hash` but have implemented `PartialEq` explicitly`` (`derived_hash_with_manual_eq`, deny) | `Hash`도 `eq`와 같은 기준으로 손으로 구현 (2.11절) |
| `#[derive(PartialOrd)]` + 손으로 쓴 `impl Ord` | `` you are implementing `Ord` explicitly but have derived `PartialOrd` `` (`derive_ord_xor_partial_ord`, deny) | `PartialOrd`를 `Some(self.cmp(other))`로 손으로 구현 (2.10절) |
| `Ord`가 있는데 `partial_cmp`를 다른 모양으로 | ``non-canonical implementation of `partial_cmp` on an `Ord` type`` (`non_canonical_partial_ord_impl`, warn) | `Some(self.cmp(other))` |
| `Copy` 타입의 `clone`을 필드 복사로 작성 | ``non-canonical implementation of `clone` on a `Copy` type`` (`non_canonical_clone_impl`, warn) | `*self` 또는 `derive(Clone)` |
| `Copy` 값에 `.clone()` | ``using `clone` on type `P` which implements the `Copy` trait`` (`clone_on_copy`, warn) | `.clone()`을 지움 |
| `impl Into<B> for A` | ``an implementation of `From` is preferred ...`` (`from_over_into`, warn) | `impl From<A> for B` (4.1절) |
| `u32::try_from(x_u8)` | `use of a fallible conversion when an infallible one could be used` (`unnecessary_fallible_conversions`, warn) | `u32::from(x_u8)` (2.4절) |
| `n.max(0).min(100)` | `clamp-like pattern without using clamp function` (`manual_clamp`, warn) | `n.clamp(0, 100)` |

그 밖에 `derive` 목록에서 전제 트레이트를 빠뜨리면 E0277입니다. `#[derive(PartialEq, Eq, Ord)]`(`PartialOrd` 없음)는 `` can't compare `Version` with `Version` ``와 `` help: consider annotating `Version` with `#[derive(PartialOrd)]` ``, `#[derive(Eq)]`만 쓰면 같은 모양의 `PartialEq` 요구가 나옵니다. `f64` 필드가 있는 구조체에 `derive(Eq)`를 붙이면 ``the trait bound `f64: Eq` is not satisfied``입니다.

## 5. 연습 문제

### 연습 1. 온도 타입과 변환

`Celsius(f64)`와 `Kelvin(f64)` 타입을 만들고, 두 방향의 `From`을 구현하세요(K = C + 273.15). 그다음 `fn report(t: impl Into<Kelvin>) -> String`이 `Celsius`와 `Kelvin`을 모두 받는지 확인하세요. 마지막으로 "절대 영도보다 낮은 온도는 만들 수 없다"는 규칙을 넣으려면 어느 변환을 `TryFrom`으로 바꿔야 하는지 생각해 보세요.

<details>
<summary>힌트</summary>

- `impl From<Celsius> for Kelvin { fn from(c: Celsius) -> Self { Kelvin(c.0 + 273.15) } }`와 반대 방향을 씁니다. `Into`는 구현하지 않습니다(4.1절).
- `report`가 `Kelvin` 자신을 받는 것은 반사 구현 덕분입니다(2.2절).
- `f64` → `Kelvin`을 만드는 경로에서 음수를 거절해야 하므로 `impl TryFrom<f64> for Kelvin`이 맞습니다. 그러면 `Celsius` → `Kelvin`도 언제나 성공한다고 할 수 없게 되는지, `Celsius`도 같은 검사를 거쳐 만들어진다면 `From`으로 남겨도 되는지 따져 보세요.
- 실수 덧셈은 오차가 있어서 `25.0 + 273.15`의 출력이 기대와 조금 다를 수 있습니다. `{:.2}`로 출력하세요.

</details>

### 연습 2. 공백 정리 함수

문자열의 앞뒤 공백을 지우고, 단어 사이의 연속된 공백을 하나로 줄이는 `fn tidy(s: &str) -> Cow<'_, str>`를 작성하세요. 이미 깔끔한 문자열이면 할당하지 않아야 합니다.

```rust
tidy("안녕하세요")          // Borrowed("안녕하세요")
tidy("  안녕   하세요 ")    // Owned("안녕 하세요")
```

<details>
<summary>힌트</summary>

- 먼저 "고칠 것이 있는가"만 검사합니다. `s.trim().len() != s.len()`이거나 `s.contains("  ")`이면 고칠 것이 있습니다.
- 고칠 것이 없으면 `Cow::Borrowed(s)`를 돌려줍니다.
- 있으면 `s.split_whitespace()`로 단어를 나눠 공백 하나로 이어 붙인 `String`을 `Cow::Owned`로 돌려줍니다. 반복자의 `collect::<Vec<_>>().join(" ")`을 쓸 수 있습니다.
- `matches!(tidy(..), Cow::Borrowed(_))`로 할당 여부를 확인하는 테스트를 써 보세요(예제 04).
- 탭(`\t`)이나 줄바꿈이 섞인 경우도 고쳐야 한다면 검사 조건을 어떻게 바꿔야 할까요?

</details>

### 연습 3. 버전 번호 비교

`"1.10.2"` 같은 문자열을 `TryFrom<&str>`로 받아 `Version { major: u32, minor: u32, patch: u32 }`를 만들고, `Version`을 `BTreeSet`에 넣어 정렬하세요. 이어서 "빌드 메타데이터"를 담는 `build: String` 필드를 추가하되, 비교와 같음 판단에서는 `build`를 무시하도록 바꿔 보세요(`"1.2.0+abc"`와 `"1.2.0+xyz"`는 같은 버전).

<details>
<summary>힌트</summary>

- 첫 단계는 `#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]`만으로 충분합니다. 필드 순서(`major`, `minor`, `patch`)가 비교 순서입니다(2.9절).
- `try_from`에서는 `split('.')`로 세 조각을 얻고 각 조각을 `parse::<u32>()`합니다. 에러 타입은 `String` 정도로 간단히 둡니다. 조각 수가 3이 아니면 `Err`입니다.
- `build`를 추가하면 `derive`가 `build`까지 비교하므로 손으로 구현해야 합니다. `Ord::cmp`에서 `(self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))`처럼 튜플로 비교하면 짧습니다. `PartialOrd`, `PartialEq`, `Eq`를 2.10절의 순서대로 맞추세요.
- `Hash`도 쓸 계획이라면 `build`를 넣지 않아야 합니다(2.11절).

</details>

### 연습 4. 2차원 벡터 연산자

`#[derive(Debug, Clone, Copy, PartialEq)] struct V2 { x: f64, y: f64 }`에 `+`, `-`, 단항 `-`, `V2 * f64`, `f64 * V2`, `+=`를 구현하고, `v[0]`이 `x`, `v[1]`이 `y`를 돌려주는 `Index<usize>`도 구현하세요. `Eq`나 `Hash`를 `derive`할 수 있는지도 확인해 보세요.

<details>
<summary>힌트</summary>

- 연산자마다 `impl 트레이트 for 왼쪽타입`입니다. `f64 * V2`는 `impl Mul<V2> for f64`입니다(예제 05).
- `+=`는 `AddAssign`을 따로 구현해야 합니다. `fn add_assign(&mut self, rhs: V2) { *self = *self + rhs; }`처럼 `Add`를 재사용할 수 있습니다.
- `Index<usize>`의 `index`는 `&f64`를 돌려줍니다. `0`, `1` 말고 다른 인덱스는 `panic!("V2 인덱스는 0 또는 1입니다")`로 처리합니다.
- `f64`는 `Eq`와 `Hash`를 구현하지 않으므로 `derive(Eq)`는 E0277입니다(4.6절). `V2`를 `HashSet`에 넣고 싶다면 무엇을 포기하거나 바꿔야 할지 생각해 보세요.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::convert`](https://doc.rust-lang.org/std/convert/index.html): 네 변환 트레이트의 역할 요약과 "`From`을 구현하라"는 권고. [`From`](https://doc.rust-lang.org/std/convert/trait.From.html)의 "When to implement `From`" 절(2.2절의 네 가지 기대), [`Into`](https://doc.rust-lang.org/std/convert/trait.Into.html)의 1.41 이전 사정, [`TryFrom`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html), [`AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)의 "Relation to `Borrow`"와 "Reflexivity" 절.
  - [`std::borrow`](https://doc.rust-lang.org/std/borrow/index.html): [`Borrow`](https://doc.rust-lang.org/std/borrow/trait.Borrow.html)의 `HashMap` 예와 `CaseInsensitiveString` 예(2.6절), [`Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html)의 `abs_all` 예(예제 04의 `clamp_negatives`와 같은 모양), [`ToOwned`](https://doc.rust-lang.org/std/borrow/trait.ToOwned.html).
  - [`std::ops`](https://doc.rust-lang.org/std/ops/index.html): 오버로딩할 수 있는 연산자 목록, `&&`/`||`를 오버로딩할 수 없는 이유, 참조에 대한 구현 권고. [`Index`](https://doc.rust-lang.org/std/ops/trait.Index.html)의 `*container.index(index)` 설명.
  - [`std::cmp`](https://doc.rust-lang.org/std/cmp/index.html): [`PartialEq`](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html)의 "Cross-crate considerations"(2.10절), [`Eq`](https://doc.rust-lang.org/std/cmp/trait.Eq.html), [`PartialOrd`](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html)의 일관성 조건, [`Ord`](https://doc.rust-lang.org/std/cmp/trait.Ord.html)의 "How can I implement `Ord`?"와 잘못된 구현의 예, [`Ordering`](https://doc.rust-lang.org/std/cmp/enum.Ordering.html)의 `then`/`then_with`.
  - [`std::hash`](https://doc.rust-lang.org/std/hash/index.html): [`Hash`](https://doc.rust-lang.org/std/hash/trait.Hash.html)의 "Hash and Eq", "Prefix collisions", "Portability" 절, [`BuildHasher`](https://doc.rust-lang.org/std/hash/trait.BuildHasher.html)의 `hash_one`, [`BuildHasherDefault`](https://doc.rust-lang.org/std/hash/struct.BuildHasherDefault.html). 키를 바꾸면 안 된다는 규칙은 [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) 문서 첫머리에 있습니다.
  - [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html): `#[default]`와 `..Default::default()`.
  - [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html)의 "Clone and PartialEq/Eq"와 `clone_from`, [`std::marker`](https://doc.rust-lang.org/std/marker/index.html)의 [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html)("When can't my type be `Copy`?")와 [`Sized`](https://doc.rust-lang.org/std/marker/trait.Sized.html).
- Rust By Example
  - [Conversion](https://doc.rust-lang.org/rust-by-example/conversion.html)의 [From and Into](https://doc.rust-lang.org/rust-by-example/conversion/from_into.html)와 [TryFrom and TryInto](https://doc.rust-lang.org/rust-by-example/conversion/try_from_try_into.html): `Number`와 `EvenNumber`로 2.2~2.4절을 짧게 복습합니다. From and Into 절은 `Into`를 직접 구현하는 모양도 보여 주는데, 같은 타입에 `From`까지 구현하면 4.1절의 E0119가 납니다.
  - [Derive](https://doc.rust-lang.org/rust-by-example/trait/derive.html), [Operator Overloading](https://doc.rust-lang.org/rust-by-example/trait/ops.html), [Clone](https://doc.rust-lang.org/rust-by-example/trait/clone.html): `derive`할 수 있는 트레이트 목록, `Foo + Bar`처럼 서로 다른 타입을 더하는 예, `Copy`인 유닛 구조체와 `Clone`만 가능한 `Pair(Box<i32>, Box<i32>)`.
- Rustlings
  - [`23_conversions`](https://github.com/rust-lang/rustlings/tree/main/exercises/23_conversions): `conversions1.rs`는 `as`, `conversions2.rs`는 섭씨·화씨의 `From`, `conversions4.rs`는 `Color`에 대한 `TryFrom`(튜플·배열·슬라이스), `conversions5.rs`는 `AsRef`/`AsMut` 바운드입니다. `conversions3.rs`(`FromStr`)는 ch3에서 소개했습니다.
  - [`19_smart_pointers`](https://github.com/rust-lang/rustlings/tree/main/exercises/19_smart_pointers): `smart_pointers4.rs`가 `Cow`의 `abs_all`로 "언제 복제되는가"를 테스트로 확인하는 연습입니다(2.7절).
- 에러 설명: `rustc --explain E0119`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
