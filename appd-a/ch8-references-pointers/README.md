# ch8. 참조와 포인터 타입 고르기

본편에서 포인터를 여러 모양으로 만났습니다. 참조와 슬라이스는 [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md)에서, `Box`·`Rc`·`RefCell`·`Weak`는 [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)에서, 트레이트 객체는 [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)에서, `AsRef`·`Borrow`는 [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)에서, 크기와 넓은 포인터는 [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md)에서 배웠습니다. 이 챕터는 문법과 API를 다시 설명하지 않습니다. 함수의 매개변수와 자료구조의 필드에 **어떤 포인터 타입을 둘지** 고르는 판단만 다룹니다.

여섯 지침은 한 가지 생각에서 나옵니다. 포인터 타입은 "무엇을 가리키는가"와 "그것을 어떻게 가지는가(빌림, 소유, 공유 소유)"를 함께 말합니다. 필요한 것보다 많이 말하면(`&Vec<T>`, 소유자가 하나인데 `Rc`, 칸 하나를 고치려고 값 전체에 `RefCell`) 그 비용은 부르는 쪽이나 함께 쓰는 쪽이 치릅니다. 반대로 필요한 약속을 빠뜨리면(`Borrow`의 비교 규칙, 순환을 끊는 `Weak`) 컴파일은 되지만 틀린 결과나 누수가 생깁니다.

- 원제: [Item 8: Familiarize yourself with reference and pointer types](https://effective-rust.com/references.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch8-references-pointers`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

이 챕터에 나오는 크기는 64비트 환경(macOS aarch64)에서 잰 값입니다. `T`가 크기가 정해진 타입일 때 `&T`와 `Box<T>`가 `usize`와 같은 크기라는 것은 [`size_of` 문서](https://doc.rust-lang.org/std/mem/fn.size_of.html)가 보장합니다. 넓은 포인터가 16바이트라는 것과 구조체의 크기는 이 툴체인에서 잰 결과입니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 읽기만 하는 매개변수를 `&[T]`·`&str`·`&T`로 선언해 받을 수 있는 범위를 넓힐 수 있고, `&mut Vec<T>`나 `&Vec<T>`가 꼭 필요한 경우를 구별할 수 있다.
2. 슬라이스와 트레이트 객체가 주소 말고 무엇을 더 들고 다니는지 설명하고, 범위를 인덱스 대신 슬라이스로 넘기는 시그니처를 작성할 수 있다. 포인터가 두 겹이 되어야 하는 경우도 설명할 수 있다.
3. 제네릭 매개변수의 바운드로 `AsRef<T>`와 `Borrow<T>` 중 무엇을 쓸지, 함수가 무엇에 기대는지 보고 고를 수 있다.
4. `Borrow`가 약속하는 것을 설명하고, 약속을 어긴 구현이 `HashMap` 조회를 어떻게 틀리게 만드는지 보일 수 있다.
5. "소유자가 하나인가 → 스레드를 넘는가 → 내부 가변성이 필요한가 → 순환이 생기는가"의 순서로 물어 스마트 포인터 조합을 고를 수 있다.

## 2. 핵심 개념

### 2.1 지도: 포인터 타입과 포인터 트레이트

Rust에서 "포인터"라고 부를 수 있는 것은 다섯 갈래입니다.

| 갈래 | 예 | 가리키는 값과의 관계 | 주소 말고 더 드는 것 | 본편 |
| --- | --- | --- | --- | --- |
| 참조 | `&T`, `&mut T` | 빌림. 대여 규칙을 컴파일할 때 검사 | 없음 | Part 0 ch4 |
| 넓은 포인터(wide pointer) | `&[T]`, `&str`, `&dyn Trait`, `Box<[T]>`, `Box<dyn Trait>` | 빌림 또는 소유 | 길이 또는 vtable | [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.15절, Part 1 ch7 2.4절 |
| 스마트 포인터 | `Box<T>`, `Rc<T>`, `Arc<T>` (넓게는 `Vec<T>`, `String`) | 소유 또는 공유 소유 | 타입마다 다름(`Rc`의 카운트는 힙에) | Part 0 ch14 |
| 대여 가드 | `Ref<T>`, `RefMut<T>`, `MutexGuard<T>` | 빌림. 실행 중에 검사 | 대여 표시나 잠금으로 가는 참조 | Part 1 ch7, [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) |
| 원시 포인터(raw pointer) | `*const T`, `*mut T` | 아무 보장 없음 | 넓은 포인터면 똑같이 | Part 0 ch19 2.2절, [`ch16-avoid-unsafe`](../ch16-avoid-unsafe/README.md) |

포인터처럼 동작하게 해 주는 트레이트도 정리해 둡니다. 컴파일러가 특별히 대우하는 것은 `Deref`뿐이고, 나머지는 라이브러리가 정한 약속입니다.

| 트레이트 | 약속 | 컴파일러가 따로 해 주는 일 | 본편 |
| --- | --- | --- | --- |
| `Deref`, `DerefMut` | `*`로 따라가면 `Target`이 나온다 | 역참조 강제 변환, 메서드 탐색 | Part 0 ch14 2.4~2.5절 |
| `AsRef<T>`, `AsMut<T>` | `&T`로 싸게 볼 수 있다 | 없음 | Part 1 ch6 2.5절 |
| `Borrow<T>`, `BorrowMut<T>` | 빌린 형태가 원래 값과 똑같이 비교·해시된다 | 없음 | Part 1 ch6 2.6절 |
| `ToOwned` | 빌린 형태에서 소유한 형태를 만든다 | 없음 | Part 1 ch6 2.7절 |
| `fmt::Pointer` | `{:p}`로 주소를 출력한다 | 서식 | — |

`Deref`와 `AsRef`의 차이 하나는 지침 3에서 중요해집니다. `Deref`는 대상을 **연관 타입** `Target`으로 정하므로 타입마다 하나뿐입니다(`String`은 `str`). `AsRef<T>`는 대상을 **타입 매개변수**로 받으므로 한 타입이 여러 대상에 구현할 수 있습니다. `String`은 `AsRef<str>`, `AsRef<[u8]>`, `AsRef<OsStr>`, `AsRef<Path>`를 모두 구현합니다.

### 2.2 지침 1: 읽기만 하는 매개변수는 가리키는 대상으로 받는다

**문제 상황.** 운동 기록 앱에서 평균 걸음 수를 구하는 함수, 운동 이름이 유산소인지 판별하는 함수, 운동 하나를 요약하는 함수를 만듭니다. 부르는 쪽이 가진 데이터는 모양이 제각각입니다. 이번 주 기록은 배열 `[u32; 7]`, 이번 달 기록은 `Vec<u32>`, 최근 3일은 그 `Vec`의 일부, 운동 이름은 메모 문자열 `"수영 40분"`의 일부, 오늘 운동은 스택에 있는 `Workout` 값입니다.

**비권장 코드.** 데이터를 보관하는 타입을 그대로 매개변수에 씁니다.

```rust
fn average_vec(days: &Vec<u32>) -> u32
fn is_cardio_string(kind: &String) -> bool
fn summary_boxed(workout: &Box<Workout>) -> String
```

`average_vec(&week)`는 E0308입니다(4.1절). 부르는 쪽은 `week.to_vec()`, `month[7..].to_vec()`, `kind.to_string()`, `Box::new(today.clone())`처럼 함수에 맞춘 임시 값을 만들어야 합니다. [`01_borrow_the_target.rs`](examples/01_borrow_the_target.rs)의 1번 구역에서는 이렇게 만든 임시 값이 4개, 복사한 바이트가 87입니다(구조체 크기를 포함한 64비트 기준 값). 결과는 맞지만 할당과 복사는 모두 시그니처 때문에 생긴 비용입니다.

clippy는 세 시그니처를 모두 잡습니다. 예제는 비교를 위해 `#[expect(clippy::ptr_arg, ..)]`와 `#[expect(clippy::borrowed_box, ..)]`로 막아 두었고, 속성을 지우면 다음 경고가 납니다(핵심 줄만 옮겼습니다).

```text
warning: writing `&Vec` instead of `&[_]` involves a new object where a slice will do
warning: writing `&String` instead of `&str` involves a new object where a slice will do
warning: you seem to be trying to use `&Box<T>`
  help: consider using just `&T`: `&Workout`
```

`ptr_arg`는 `&PathBuf` 매개변수에도 `&Path`를 쓰라고 경고합니다.

**권장 코드.** 컨테이너가 아니라 컨테이너가 가리키는 대상으로 받습니다. 본문은 한 글자도 바뀌지 않습니다.

```rust
fn average(days: &[u32]) -> u32
fn is_cardio(kind: &str) -> bool
fn summary(workout: &Workout) -> String
```

비권장 판에 넘길 수 있던 것은 권장 판에도 모두 넘어갑니다. `&Vec<u32>` → `&[u32]`, `&String` → `&str`, `&Box<Workout>` → `&Workout`은 역참조 강제 변환(Part 0 ch14 2.5절)이 처리하고, `&[u32; 7]` → `&[u32]`는 크기가 정해진 배열의 참조를 슬라이스 참조로 바꾸는 강제 변환이 처리합니다. 받을 수 있는 범위는 넓어지기만 합니다. 강제 변환이 일어나는 자리는 [`ch5-type-conversions`](../ch5-type-conversions/README.md)에서 정리합니다.

**크기 차이가 말해 주는 것.** `&Vec<u32>`는 8바이트 얇은 포인터이고 `&[u32]`는 16바이트 넓은 포인터입니다. 좁은 타입이 더 작지만, 가리키는 곳이 다릅니다. 예제 01의 3번 구역은 주소만 비교합니다.

```text
    &Vec이 가리키는 곳이 첫 원소인가: false
    &[u32]가 가리키는 곳이 첫 원소인가: true
```

`&Vec<u32>`는 `Vec` 핸들(포인터, 용량, 길이)을 가리키므로 원소에 닿으려면 핸들의 포인터를 한 번 더 따라가야 합니다. `&[u32]`는 원소를 바로 가리키고 길이를 직접 들고 있습니다. 매개변수 하나가 8바이트 커지는 것은 거의 모든 프로그램에서 문제가 되지 않습니다. 그래서 크기는 선택의 이유가 아니고, 더 많은 데이터를 받으면서 한 단계 덜 따라간다는 점이 이유입니다.

**지침을 따르지 않아도 되는 경우.** 함수가 대상이 아니라 **컨테이너 자체**를 다룰 때는 컨테이너 타입이 맞습니다.

- **길이를 바꿀 때**: `push`, `truncate`, `clear`처럼 원소 수를 바꾸면 `&mut Vec<T>`나 `&mut String`이 필요합니다(예제 01의 `record`). 원소를 제자리에서 고치기만 한다면 `&mut [T]`로 충분하고 배열도 받습니다(`cap_outliers`). 본문이 원소만 고치는 `&mut Vec<T>` 매개변수에는 `ptr_arg`가 ``writing `&mut Vec` instead of `&mut [_]` ``로 경고합니다.
- **컨테이너에만 있는 정보를 읽을 때**: `capacity()`가 필요하면 `&Vec<T>`입니다(`fits_without_realloc`). 이때는 본문이 `Vec`의 메서드를 쓰므로 `ptr_arg`가 경고하지 않습니다.
- **포인터 자체를 나눠 가질 수 있을 때**: 함수가 경우에 따라 `Rc::clone`으로 소유권을 나눠 가져야 한다면 `&Rc<T>`로 받습니다. clippy도 `&Rc<T>`에는 경고하지 않습니다. 언제나 보관한다면 `Rc<T>`를 값으로 받는 편이 부르는 쪽에 의도를 더 잘 알립니다.

본문이 `kind.clone()`으로 `String`을 만들어 돌려주더라도 `&String` 매개변수에는 `ptr_arg`가 경고하고, `&str`로 받고 `to_owned()`를 쓰라고 제안합니다. 받은 문자열을 저장하는 함수라면 `String`이나 `impl Into<String>`으로 받는 방법도 있습니다(Part 1 ch6 2.5절).

### 2.3 지침 2: 넓은 포인터가 들고 다니는 정보에 맡긴다

넓은 포인터는 주소와 함께 추가 정보(메타데이터)를 한 칸 들고 다닙니다. 슬라이스와 `str`은 **길이**를, 트레이트 객체는 **vtable**(메서드 포인터 표: 구체 타입의 메서드 주소, 값의 크기와 메모리 정렬, drop 방법)의 주소를 듭니다. 크기 표는 Part 1 ch7 2.4절에 있으므로 여기서는 이 정보가 시그니처에 주는 의미만 봅니다.

**문제 상황.** 한 달 동안의 일별 강수량 배열에서 장마 기간(11~15일)의 합계를 구합니다.

**비권장 코드.** 전체 데이터와 범위를 따로 받습니다.

```rust
fn rain_total_in(days: &[u32], start: usize, count: usize) -> u32 {
    days[start..start + count].iter().sum()
}
```

부르는 쪽은 Rust의 범위 문법 `10..15`에 익숙해서 둘째 인자를 끝 위치로 읽고 `rain_total_in(&month, 10, 15)`라고 씁니다. 받는 쪽은 개수로 읽으므로 15일치를 더하고, 그 사이에 든 센서 오류값 999까지 더해집니다. [`02_wide_pointer_metadata.rs`](examples/02_wide_pointer_metadata.rs)의 1번 구역 결과는 131mm가 아니라 1136mm입니다. 두 인자가 모두 `usize`라서 컴파일러가 막을 방법이 없습니다. C 언어는 배열을 넘길 때 포인터와 길이를 따로 넘기는데, 둘이 어긋나는 것이 오래된 버그의 원천입니다. 위 시그니처는 그 방식을 Rust로 옮겨 온 것입니다.

**권장 코드.** 범위를 슬라이스 하나로 받습니다.

```rust
fn rain_total(days: &[u32]) -> u32 {
    days.iter().sum()
}

rain_total(&month[10..15]) // 131
```

범위는 부르는 쪽에서 범위 문법으로 정하므로 "끝 위치인가 개수인가"를 헷갈릴 일이 없고, 길이는 슬라이스가 들고 갑니다. 슬라이스는 원본을 가리킬 뿐이라 복사도 없습니다(2번 구역에서 `rainy`가 원본의 11일째를 가리키는지 확인합니다). 가변 슬라이스로 넘기면 받는 쪽이 고칠 수 있는 곳도 그 범위로 제한됩니다. `clear_errors(&mut month[20..25])`는 21~25일의 999만 0으로 바꾸고, 범위 밖 26일째의 999는 그대로 남습니다. 받는 쪽 코드에 실수가 있어도 범위 밖은 건드릴 수 없습니다.

**길이는 포인터 값의 일부입니다.** 3번 구역은 같은 주소에서 시작하는 두 슬라이스 `&month[..7]`과 `&month[..]`를 비교합니다. `ptr::addr_eq`(주소만 비교)는 `true`, `ptr::eq`(주소와 메타데이터를 함께 비교)는 `false`입니다. 길이가 다르면 다른 포인터입니다. `split_at`, `chunks`, `split_first` 같은 메서드가 복사 없이 슬라이스를 나눌 수 있는 것도 새 주소와 새 길이를 가진 넓은 포인터를 만들기만 하면 되기 때문입니다.

**포인터 하나에는 추가 정보도 하나입니다.** 트레이트 객체는 길이 자리에 vtable을 싣습니다. 그래서 길이와 vtable이 모두 필요하면 포인터가 두 겹이 됩니다.

| 넘길 것 | 매개변수 | 바깥 포인터가 든 것 | 안쪽에 든 것 |
| --- | --- | --- | --- |
| 같은 타입의 값 여러 개 | `&[T]` | 길이 | 값 자체가 이어져 있음 |
| 타입을 모르는 값 하나 | `&dyn Trait` | vtable | 값 |
| 타입이 섞인 값 여러 개 | `&[&dyn Trait]`, `&[Box<dyn Trait>]` | 길이 | 원소마다 vtable을 든 넓은 포인터 |
| 크기를 모르는 값(`str`, `[T]`) 하나를 트레이트 객체로 | `&dyn Display`에 `&&str`을 넘김 | vtable | 길이를 든 `&str` |

4번 구역에서 장비 두 개를 담은 `[&dyn Instrument; 2]`는 32바이트입니다. 값이 아니라 넓은 포인터 두 개가 들어 있기 때문입니다. 마지막 줄의 경우, `&str`을 곧바로 `&dyn Display`로 바꾸면 E0277입니다(4.2절). `&str`은 이미 길이를 실었으므로 vtable을 더 실을 자리가 없습니다. 한 겹 감싼 `&&str`은 8바이트 얇은 포인터라 vtable을 실을 수 있고, 컴파일러의 도움말도 이것을 제안합니다. 타입이 섞인 값을 받는 시그니처의 여러 모양은 [ch2](../ch2-express-common-behavior/README.md) 2.6절에서 다뤘습니다.

**넓은 포인터의 주소를 비교할 때.** 슬라이스의 `ptr::eq`는 길이까지 비교하므로 결과가 분명합니다. 트레이트 객체는 다릅니다. [`ptr::eq` 문서](https://doc.rust-lang.org/std/ptr/fn.eq.html)는 같은 타입의 vtable이 코드 생성 단위마다 따로 만들어질 수도 있고, 내용이 같은 vtable이 하나로 합쳐질 수도 있어서 트레이트 객체 포인터의 비교는 믿을 수 없다고 적고 있습니다. 그래서 이 챕터의 예제도 vtable이 섞인 비교 결과는 출력하지 않습니다. "같은 값을 가리키는가"를 알고 싶다면 `ptr::addr_eq`로 주소만 비교합니다. 넓은 원시 포인터를 `==`로 비교하면 기본으로 켜진 린트가 경고합니다.

```text
warning: ambiguous wide pointer comparison, the comparison includes metadata which may not be expected
  = note: `#[warn(ambiguous_wide_pointer_comparisons)]` on by default
help: use `std::ptr::addr_eq` or untyped pointers to only compare their addresses
```

**지침을 따르지 않아도 되는 경우.** 위치 자체가 함수의 입력이나 결과일 때는 인덱스를 씁니다. 찾은 위치를 돌려주는 `position`이 그렇고, 여러 자료구조가 같은 원소를 가리켜야 해서 참조 대신 인덱스를 보관하는 설계도 그렇습니다([`ch15-borrow-checker`](../ch15-borrow-checker/README.md)). 슬라이스는 원본을 빌리고 있으므로 원본보다 오래 보관할 수 없다는 점도 인덱스를 고르는 이유가 됩니다([`ch14-lifetimes`](../ch14-lifetimes/README.md)).

### 2.4 지침 3: 보기만 하면 `AsRef`, 같은 타입의 여러 형태를 받으면 `Borrow`

지침 1은 형태가 하나인 매개변수의 이야기였습니다. 여러 타입을 한 함수로 받고 싶으면 제네릭 매개변수에 바운드를 겁니다. 이때 `AsRef<T>`와 `Borrow<T>`는 메서드 모양(`&self`에서 `&T`)이 같아서 아무것이나 고르기 쉽습니다.

**문제 상황.** 전송할 데이터의 체크섬(바이트를 더한 값)을 구하는 함수입니다. 부르는 쪽은 `&str`, `String`, `Vec<u8>`, 바이트 배열 등을 넘기고 싶어 합니다.

**비권장 코드.**

```rust
fn checksum_borrow<B: Borrow<[u8]>>(data: B) -> u8
```

[`03_asref_or_borrow.rs`](examples/03_asref_or_borrow.rs)의 1번 구역에서 그대로 넘어가는 것은 `Vec<u8>`, `[u8; 4]`, `&[u8]` 세 가지뿐입니다. `String`을 넘기면 ``the trait bound `String: Borrow<[u8]>` is not satisfied``(E0277)이고, `&str`과 `&Vec<u8>`도 같은 모양의 E0277이라서 부르는 쪽이 `.as_bytes()`, `.as_slice()`를 붙여야 합니다. 이유는 둘입니다.

- **`Borrow`는 비교·해시 결과가 같다고 약속해야 합니다.** 3번 구역에서 `"ok".as_bytes() == b"ok"`는 `true`지만 `str`과 `[u8]`의 해시는 다릅니다(`false`). `[u8]`의 해시는 길이를 먼저 넣고, `str`의 해시는 끝에 구분용 바이트를 넣기 때문입니다. 그래서 `String`은 바이트로 *보일* 수는 있어도 `[u8]`과 *같은 키*는 아니고, 표준 라이브러리는 `String: Borrow<[u8]>`를 구현하지 않습니다. `String`과 `str`은 해시가 같으므로 `String: Borrow<str>`는 있습니다.
- **참조로 전달되지 않습니다.** `Borrow`에는 `&T: Borrow<T>`라는 구현만 있어서, `&Vec<u8>`는 `Borrow<Vec<u8>>`일 뿐 `Borrow<[u8]>`가 아닙니다.

**권장 코드.**

```rust
fn checksum<B: AsRef<[u8]>>(data: B) -> u8
```

2번 구역에서 일곱 가지가 모두 그대로 들어가고 결과는 1번 구역과 같습니다. `AsRef`는 "싸게 볼 수 있다"는 것만 약속하므로 `String`과 `str`도 `AsRef<[u8]>`를 구현하고, `T: AsRef<U>`이면 `&T: AsRef<U>`라는 포괄 구현이 있어서 참조도 받습니다. 표준 라이브러리의 `fs::read<P: AsRef<Path>>(path: P)` 같은 함수가 `&str`, `String`, `PathBuf`를 모두 받는 것도 같은 방법입니다.

**반대 방향: `Borrow`가 맞는 경우.** 재생 목록의 총 재생 시간을 구하는 함수를 생각합니다. 곡을 소유한 라이브러리(`Vec<Track>`), 라이브러리의 곡을 빌려 가리키는 재생 목록(`Vec<&Track>`), 곡을 함께 소유한 공유 목록(`Vec<Rc<Track>>`)을 모두 받고 싶습니다. 이번에는 다른 타입으로 *보는* 것이 아니라 **같은 타입을 소유한 형태로도 빌린 형태로도** 받는 것이 목적입니다.

```rust
fn total_secs_as_ref<T: AsRef<Track>>(list: &[T]) -> u32 // 비권장
fn total_secs<T: Borrow<Track>>(list: &[T]) -> u32       // 권장
```

`AsRef`에는 `impl<T> AsRef<T> for T` 같은 반사 구현이 없습니다. 그래서 `AsRef<Track>` 판은 `Rc<Track>`와 `Box<Track>`의 목록만 받고, `&library`와 `&playlist`는 E0277입니다(4.3절). `Borrow`는 모든 `T`, `&T`, `&mut T`, `Box<T>`, `Rc<T>`, `Arc<T>`가 `Borrow<T>`를 구현하므로 4번 구역에서 세 목록이 모두 들어갑니다(636초, 391초, 512초).

| | `AsRef<T>` | `Borrow<T>` |
| --- | --- | --- |
| 약속 | `&T`로 싸게 볼 수 있음 | 여기에 더해 `Eq`·`Hash`·`Ord`의 결과가 같음 |
| 한 타입이 구현하는 대상 | 여럿 (`String`: `str`, `[u8]`, `OsStr`, `Path`) | 자기 자신 + 대개 하나 (`String`: `String`, `str`) |
| `T` 자신 | 구현되지 않음 | 언제나 구현됨 |
| `&T` | `T: AsRef<U>`이면 `&T: AsRef<U>` | `&T: Borrow<T>`만 |
| 고를 때 | 다른 타입으로 들여다보기만 함(경로, 바이트, 문자열) | 같은 타입의 소유·빌린 형태를 모두 받음, 비교·해시로 찾음 |

**지침을 따르지 않아도 되는 경우.** 부르는 쪽이 넘길 형태가 사실상 하나라면 제네릭을 쓰지 않고 `&[u8]`, `&Path`, `&[Track]`처럼 지침 1의 모양으로 받습니다. 제네릭 함수는 쓰인 타입마다 코드가 한 벌씩 만들어지고(단형성화, [`ch12-generics-vs-trait-objects`](../ch12-generics-vs-trait-objects/README.md)) 시그니처도 길어집니다. 다양한 입력이 예상되는 공개 API에서 바운드가 값어치를 합니다. 참조로 받는 `&S` 매개변수에 `AsRef`를 걸 때 `?Sized`가 필요하다는 점은 Part 1 ch6 2.5절에서 다뤘습니다.

### 2.5 지침 4: `Borrow`를 구현할 때는 비교와 해시를 맞춘다

[`Borrow` 문서](https://doc.rust-lang.org/std/borrow/trait.Borrow.html)는 구현하는 타입에 "`Eq`, `Ord`, `Hash`가 빌린 값과 소유한 값에서 같아야 한다", 즉 `x.borrow() == y.borrow()`가 `x == y`와 같은 답을 내야 한다고 요구합니다. `HashMap::get(&Q)`는 `K: Borrow<Q>`를 바운드로 요구하고, 질의 `q`의 해시로 자리를 찾은 뒤 `k.borrow() == q`로 키를 확인합니다. 이 약속을 믿는 것입니다.

**문제 상황.** 공항 코드를 키로 하는 맵을 만듭니다. 사용자는 `icn`, `Gmp`, `CJU`처럼 대소문자를 섞어 입력하므로 대소문자를 무시하고 같은 공항으로 다뤄야 합니다. `&str`로도 찾고 싶어서 `Borrow<str>`를 구현합니다.

**비권장 코드.** 입력을 그대로 보관하고, `Eq`와 `Hash`만 대소문자를 무시하게 구현합니다.

```rust
struct LooseCode(String); // 입력 그대로

impl PartialEq for LooseCode { /* eq_ignore_ascii_case로 비교 */ }
impl Hash for LooseCode { /* 대문자로 바꾼 문자열의 해시 */ }

impl Borrow<str> for LooseCode {
    fn borrow(&self) -> &str { &self.0 } // 약속 위반: LooseCode로는 같은 "icn"과 "ICN"이 str로는 다름
}
```

`LooseCode`끼리는 `Eq`와 `Hash`가 일관되므로 키 타입으로 찾으면 문제가 없습니다. 문제는 `&str`로 찾을 때입니다. [`04_borrow_contract.rs`](examples/04_borrow_contract.rs)의 1번 구역 결과입니다.

```text
    LooseCode로 "ICN" 찾기: Some("인천"), &str로 "ICN" 찾기: None
    &str로 "CJU" 찾기: Some("제주"), "cju" 찾기: None
    ICN을 새로 추가했는데 항목 수 3, icn의 값: Some("인천(신규 등록)")
```

같은 맵이 키 타입으로 물으면 "있다", `&str`로 물으면 "없다"고 답합니다. `&str`로 찾을 때는 입력한 그대로의 글자가 맞아야만 찾습니다. 마지막 줄은 이 모순이 실제 버그가 되는 모습입니다. "`ICN`이 없으면 추가한다"는 코드가 `contains_key("ICN")`의 `false`를 믿고 `insert`했는데, `insert`는 키 타입으로 비교하므로 기존 `icn` 항목의 값을 조용히 덮어썼습니다.

결과가 이 정도로 예측 가능한 것도 운입니다. 입력한 그대로인 `get("icn")`은 해시(소문자 `icn`의 해시)와 맵에 저장된 해시(대문자 `ICN`의 해시)가 달라서, 맵이 찾아가는 자리에 그 항목이 우연히 걸리느냐에 따라 결과가 달라질 수 있습니다. `HashMap`의 기본 해셔는 맵마다 다른 시드를 쓰므로 같은 질의가 실행마다 다른 답을 냅니다. 같은 세 항목으로 맵을 2만 번 새로 만들어 시험했을 때 `get("icn")`은 153번 `Some`, 나머지는 `None`이었습니다(이 수도 실행마다 달라집니다). 그래서 예제는 결과가 하나로 정해지는 질의만 출력합니다. 컴파일러는 이 약속을 검사하지 않고, clippy도 이 예제에 경고하지 않았습니다. `BTreeMap::get`은 같은 방식으로 `Ord`에 기댑니다(연습 3).

**권장 코드.** 만들 때 정규화해 두고, 저장한 문자열을 그대로 비교합니다.

```rust
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
struct AirportCode(String);

impl AirportCode {
    fn new(raw: &str) -> Self {
        AirportCode(raw.trim().to_ascii_uppercase())
    }
}

impl Borrow<str> for AirportCode {
    fn borrow(&self) -> &str { &self.0 }
}
```

`derive`한 `Eq`·`Hash`·`Ord`는 필드 하나인 `String`의 결과를 그대로 쓰므로 `str`의 결과와 같고, 약속이 지켜집니다. 2번 구역에서 `AirportCode::new("icn")`과 `"ICN"`이 같은 항목을 찾습니다. `" gmp "`처럼 정규화하지 않은 문자열로 찾으면 `None`이지만, 이것은 `str`의 비교 규칙(대소문자 구분)을 일관되게 따른 결과입니다. 부르는 쪽은 입력을 정규화해서 찾으면 됩니다. 판단 기준을 한 줄로 줄이면, **빌려주는 값만 보고 `Eq`·`Hash`·`Ord`를 계산할 수 있어야 합니다.** 사용자 정의 타입별로 판단한 예는 Part 1 ch6 2.6절의 표에 있습니다.

**지침을 따르지 않아도 되는 경우: `Borrow`를 아예 구현하지 않는다.** 정규화가 어렵거나 약속을 지킬 자신이 없다면 `Borrow<str>`를 구현하지 않고, 문자열을 보여 줄 필요가 있으면 아무것도 약속하지 않는 `AsRef<str>`만 구현합니다(3번 구역의 `StrictCode`). 그러면 `map.get("ICN")`이 컴파일되지 않으므로(4.4절) 부르는 쪽은 `map.get(&StrictCode::new(input))`처럼 키 타입을 만들어 찾습니다. 찾을 때마다 `String`을 하나 할당하지만, 잘못 찾을 길이 없습니다.

### 2.6 지침 5: 스마트 포인터는 소유 방식부터 고른다

스마트 포인터를 고를 때는 다음 네 질문을 차례로 던집니다. 앞의 질문일수록 설계 전체에 영향을 주므로 순서가 중요합니다. 이 절은 ①②, 다음 절은 ③④를 다룹니다.

| 순서 | 질문 | 아니다 | 그렇다 |
| --- | --- | --- | --- |
| ① | 소유자가 하나인가? | ②로 | 값을 그대로 둡니다. 간접 참조가 꼭 필요할 때만 `Box<T>` |
| ② | 소유자들이 스레드를 넘는가? | `Rc<T>` | `Arc<T>`. 빌려 쓰는 것으로 충분하면(스코프 스레드) `&T` |
| ③ | 공유한 값을 고쳐야 하는가? | 그대로 | 고칠 칸에만 `Cell`/`RefCell` (스레드를 넘으면 원자 타입, `Mutex`, `RwLock`) |
| ④ | 거꾸로 가리키는 연결이 있는가(순환)? | 그대로 | 거꾸로 가리키는 쪽을 `Weak<T>` |

**문제 상황.** 사진 앨범 앱을 만듭니다. 사진은 이름과 픽셀 버퍼(`Vec<u8>`)를 가집니다.

**① 소유자가 하나: 비권장 코드.** 다른 언어에서 "객체는 힙에 둔다"는 습관을 가져와 앨범의 사진을 하나씩 상자에 담습니다.

```rust
struct BoxedAlbum {
    photos: Vec<Box<Photo>>,
}
```

`Vec`은 이미 원소를 힙의 버퍼에 둡니다. `Vec<Box<Photo>>`의 버퍼에는 포인터만 들어 있고 사진은 각자 따로 할당된 곳에 있으므로, 사진 수만큼 할당이 늘고 원소에 닿을 때마다 포인터를 한 번 더 따라갑니다. [`05_choose_owner_pointer.rs`](examples/05_choose_owner_pointer.rs)의 1번 구역은 주소만 비교해 이것을 확인합니다. `Vec<Box<Photo>>`의 버퍼는 16바이트(포인터 2개)이고 사진은 버퍼 밖에 있습니다(`false`). `Vec<Photo>`의 버퍼는 96바이트(이 툴체인에서 `Photo` 하나가 48바이트)이고 사진이 버퍼 안에 있습니다(`true`).

clippy는 겹친 할당을 여러 린트로 잡습니다. 모두 기본으로 켜져 있고, 구조체 필드나 함수 시그니처에 쓴 타입에 경고합니다(`let` 문의 타입 표기에는 경고하지 않았습니다). 예제는 `BoxedAlbum`의 필드에 `#[expect(clippy::vec_box, ..)]`를 붙여 두었습니다.

| 린트 | 잡는 모양 | 경고 메시지 |
| --- | --- | --- |
| `vec_box` | `Vec<Box<T>>` | ``warning: `Vec<T>` is already on the heap, the boxing is unnecessary`` |
| `box_collection` | `Box<Vec<T>>` 등 | ``warning: you seem to be trying to use `Box<Vec<..>>`. Consider using just `Vec<..>` `` |
| `redundant_allocation` | `Rc<Box<T>>` 등 | ``warning: usage of `Rc<Box<Workout>>` ``와 ``help: consider using just `Rc<Workout>` or `Box<Workout>` ``(`Workout` 필드로 확인) |

**① 권장 코드.** 소유자가 하나면 값을 그대로 둡니다(`Vec<Photo>`). `Box`는 간접 참조가 꼭 필요할 때만 씁니다. 재귀 타입(Part 0 ch14 2.3절), 크기가 다른 구현을 담는 트레이트 객체(Part 0 ch17), 다른 배리언트보다 훨씬 큰 열거형 배리언트(Part 1 ch7 2.7절)가 그런 경우입니다.

**② 소유자가 여럿: 비권장 코드.** 같은 사진을 "가족" 앨범과 "여행" 앨범에 모두 넣으려고 `clone()`합니다. 2번 구역에서 1MB짜리 사진의 픽셀 버퍼가 두 벌, 2000000바이트가 됩니다.

**② 권장 코드.** `Rc<Photo>`로 감싸고 `Rc::clone`으로 소유자를 늘립니다. 복제되는 것은 포인터와 카운트뿐이라 버퍼는 한 벌(1000000바이트)이고, `Rc::strong_count`는 3, `Rc::ptr_eq`는 `true`입니다. `Rc`가 안의 값을 고치지 못하게 막는 것도 이 상황에 맞습니다. 여러 앨범이 함께 보는 사진을 한 앨범이 몰래 바꾸면 안 되기 때문입니다.

**② 스레드를 넘는가.** `Rc`는 카운트를 원자적으로 바꾸지 않으므로 `Send`가 아니고, `thread::spawn`에 넘기면 E0277입니다([Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 4.4절). 스레드를 넘는 공유 소유에는 원자적 카운트를 쓰는 `Arc`를 씁니다. 그런데 공유 소유가 필요한 이유는 `spawn`한 스레드가 언제 끝날지 몰라서 `'static`이어야 하기 때문입니다. 스코프 스레드(Part 1 ch11 2.2절)는 스코프가 끝나기 전에 반드시 합류하므로 지역 값을 `&Photo`로 빌려 쓰면 됩니다. 3번 구역에서 두 방법의 결과는 같고(`[124, 124]`), 스코프 판에는 카운트가 없습니다.

**지침을 따르지 않아도 되는 경우.** 값이 작으면 복제가 더 단순합니다. `Rc`는 할당마다 카운트 두 개를 더 두고 안의 값을 고칠 수 없게 만듭니다. 복제본마다 따로 고쳐야 한다면 공유가 아니라 복제가 맞는 모델입니다. 복제를 두려워하지 않아도 되는 이유는 [`ch20-avoid-over-optimizing`](../ch20-avoid-over-optimizing/README.md)에서 다룹니다. 또 소유자가 분명하고 다른 쪽은 그보다 짧게 쓰기만 한다면, 스마트 포인터 없이 참조로 충분합니다([`ch14-lifetimes`](../ch14-lifetimes/README.md)).

### 2.7 지침 6: 내부 가변성은 고칠 칸에만, 순환은 `Weak`로 끊는다

**③ 공유한 값을 고쳐야 한다: 문제 상황.** 앨범 화면과 좋아요 버튼의 처리기가 사진 카드 목록을 `Rc`로 함께 소유합니다. 화면은 목록을 빌려 카드를 차례로 그리고, 그리는 도중에도 사용자가 좋아요를 누를 수 있습니다.

**③ 비권장 코드.** 목록 전체를 `RefCell`로 감쌉니다.

```rust
let gallery: Rc<RefCell<Vec<Card>>> = /* .. */;
```

[`06_mutability_and_cycles.rs`](examples/06_mutability_and_cycles.rs)의 1번 구역에서 화면이 `gallery.borrow()`로 목록을 빌린 동안 처리기의 `try_borrow_mut()`은 `Err("RefCell already borrowed")`입니다. 좋아요 수는 카드 제목과 아무 관계가 없는데도, 목록 전체가 한 덩어리로 빌려져 있어서 고칠 수 없습니다. `try_borrow_mut()` 대신 `borrow_mut()`을 썼다면 패닉입니다.

**③ 권장 코드.** 바뀌는 칸만 내부 가변성으로 둡니다.

```rust
struct LiveCard {
    title: &'static str,
    likes: Cell<u32>, // 바뀌는 것은 이 칸뿐
}

let gallery: Rc<Vec<LiveCard>> = /* .. */;
```

2번 구역에서 목록을 읽는 도중에도 좋아요가 반영됩니다. `Cell`은 안의 값을 가리키는 참조를 내주지 않으므로 실행 중 대여 검사도, 패닉할 일도 없습니다. 칸에 무엇을 둘지는 Part 1 ch7 2.12절의 표로 고릅니다. 원소마다 큰 값을 고쳐야 한다면 `Rc<Vec<RefCell<Card>>>`처럼 원소 단위로 `RefCell`을 두는 중간 단계도 있습니다. 그러면 한 카드를 빌린 동안에도 다른 카드는 고칠 수 있습니다. 스레드를 넘는 경우도 같은 원리입니다.

| 고칠 범위 | 한 스레드 | 스레드를 넘음 |
| --- | --- | --- |
| 목록 전체(길이도 바뀜) | `Rc<RefCell<Vec<T>>>` | `Arc<Mutex<Vec<T>>>`, `Arc<RwLock<Vec<T>>>` |
| 원소 하나씩 | `Rc<Vec<RefCell<T>>>` | `Arc<Vec<Mutex<T>>>` |
| 숫자·플래그 칸 하나 | `Cell<u32>` 필드 | `AtomicU32` 필드 |

**④ 순환이 생기는가: 비권장 코드.** 사진이 자기가 속한 앨범을 알아야 해서, 사진에 `album: Rc<Album>` 필드를 둡니다. 앨범은 사진을 `Rc`로 소유하므로 앨범 → 사진 → 앨범의 순환이 생깁니다. 3번 구역에서 앨범 핸들을 버린 뒤에도 사진이 앨범을 붙잡고 있어 강한 참조가 1개 남고, 앨범은 살아 있습니다. 이 앨범과 사진은 프로그램이 끝날 때까지 해제되지 않으며, `Drop`의 해제 메시지도 나오지 않습니다.

**④ 권장 코드.** 소유 방향을 "앨범 → 사진" 한쪽으로 정하고, 거꾸로 가리키는 쪽을 `Weak`로 둡니다.

```rust
struct WeakPhoto {
    album: Weak<Album<WeakPhoto>>, // 앨범을 살려 두지 않음
}
```

4번 구역에서 앨범 핸들을 버리면 `[앨범 '가족' 해제]`가 출력되고, 사진의 `album.upgrade()`는 `None`이 됩니다. 어느 쪽을 `Weak`로 둘지는 "이 연결이 끊기면 상대가 사라져야 하는가"로 판단합니다. 앨범이 사라지면 사진도 사라져야 하지만(소유), 사진이 사라진다고 앨범이 사라지면 안 됩니다(참조만). 순환과 `Weak`의 동작은 Part 0 ch14 2.10~2.11절에서, 만들면서 자기 자신을 가리키는 `Rc::new_cyclic`은 Part 1 ch7 2.8절에서 다뤘습니다.

**지침을 따르지 않아도 되는 경우.** 고치는 일이 드물고 대여가 짧게 끝난다면 값 전체를 감싼 `Rc<RefCell<T>>` 하나가 칸마다 셀을 두는 것보다 단순할 수 있습니다. 대여가 겹칠 수 있는 구조인지가 기준입니다. 순환을 일부러 만들고 다 쓰면 직접 끊는 설계도 가능하지만, 끊는 코드를 빠뜨리면 그대로 누수입니다. 노드가 서로 복잡하게 가리키는 그래프라면 `Rc`와 `Weak` 대신 인덱스로 가리키는 편이 낫습니다([`ch15-borrow-checker`](../ch15-borrow-checker/README.md)).

### 2.8 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **"fat pointer"와 "wide pointer".** 책은 넓은 포인터를 "fat pointer"라고 부릅니다. The Reference와 rustc 린트 이름(`ambiguous_wide_pointer_comparisons`)은 "wide pointer"를 쓰고, 표준 라이브러리 문서에는 두 말이 섞여 있습니다(`ptr::eq` 문서는 wide, `ptr::addr_eq` 문서는 fat). 이 저장소는 "넓은 포인터"로 씁니다.
- **주소만 비교하는 도구가 생겼습니다.** 책은 `Pointer` 트레이트와 `{:p}`로 주소를 출력하는 방법을 소개합니다. 그 뒤 1.76.0에서 메타데이터를 무시하고 주소만 비교하는 `ptr::addr_eq`가 안정화되었고(로컬 문서의 안정화 버전 표시로 확인), 넓은 원시 포인터를 `==`로 비교하면 기본으로 켜진 린트가 경고합니다(2.3절). 이 챕터의 예제는 주소 값을 출력하지 않고 이 함수로 "같은 곳인가"만 출력합니다.
- **`size_of`를 가져오지 않아도 됩니다.** 책의 코드는 `std::mem::size_of`를 `use`로 가져오지만, 지금은 `size_of`, `size_of_val`, `align_of`가 프렐류드에 있습니다([Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.2절).
- **책에 없는 지침.** 지침 1(`&Vec<T>` 대신 `&[T]`로 받기)과 지침 4(`Borrow`를 구현할 때 비교와 해시를 맞추기)는 책의 Item 8이 명시적으로 권하는 내용이 아닙니다(2026-11-19에 온라인판에서 확인). 책은 참조·포인터 트레이트·넓은 포인터·스마트 포인터를 소개하는 데 무게를 두고, `Borrow`에 대해서는 `HashMap::get`이 이 트레이트를 쓴다는 것까지 설명합니다. 두 지침은 표준 라이브러리 문서(`Borrow`의 `Eq`·`Ord`·`Hash` 약속)와 clippy `ptr_arg`를 근거로 이 챕터가 더한 것입니다.
- **달라지지 않은 점.** 넓은 포인터의 메타데이터(슬라이스의 길이, vtable)를 직접 꺼내는 `std::ptr::metadata`는 1.100.0에서도 불안정합니다. 안정판에서는 `len()`과 `size_of_val`/`align_of_val`처럼 메타데이터를 이용하는 함수로만 간접적으로 봅니다. `RefCell`이 대여 상태를 `isize` 하나로 센다는 책의 설명도 그대로입니다(1.100.0 소스의 `type BorrowCounter = isize`). 그래서 `RefCell<u32>`는 16바이트입니다(Part 1 ch7 2.9절).

### 2.9 기억할 것

- 읽기만 하는 매개변수는 `&[T]`, `&str`, `&T`로 받습니다. 길이를 바꾸거나 컨테이너에만 있는 정보를 쓸 때만 컨테이너 타입으로 받습니다. clippy의 `ptr_arg`와 `borrowed_box`가 알려 줍니다.
- 슬라이스는 길이를, 트레이트 객체는 vtable을 들고 다닙니다. 범위는 위치와 개수 대신 슬라이스로 넘기고, 둘 다 필요하면 포인터가 두 겹이 됩니다.
- 들여다보기만 하는 제네릭 매개변수는 `AsRef<T>`, 같은 타입의 소유·빌린 형태를 모두 받거나 비교·해시로 찾는다면 `Borrow<T>`로 받습니다.
- `Borrow`를 구현할 때는 빌려주는 값만으로 `Eq`·`Hash`·`Ord`가 같은 답을 내게 합니다. 지킬 수 없으면 구현하지 않습니다.
- 스마트 포인터는 소유자 수 → 스레드 → 내부 가변성 → 순환 순서로 고르고, 각 단계에서 필요한 만큼만 감쌉니다.

## 3. 예제 안내

| 파일 | 지침 | 비권장 형태 | 권장 형태 |
| --- | --- | --- | --- |
| `01_borrow_the_target` | 1 | `&Vec<u32>`, `&String`, `&Box<Workout>` | `&[u32]`, `&str`, `&Workout` |
| `02_wide_pointer_metadata` | 2 | `(days, start, count)` | `&month[10..15]` |
| `03_asref_or_borrow` | 3 | `Borrow<[u8]>`, `AsRef<Track>` | `AsRef<[u8]>`, `Borrow<Track>` |
| `04_borrow_contract` | 4 | 대소문자를 무시하는 `Eq` + 입력 그대로의 `Borrow<str>` | 정규화한 `AirportCode` |
| `05_choose_owner_pointer` | 5 | `Vec<Box<Photo>>`, 복제 | `Vec<Photo>`, `Rc<Photo>`, `Arc`/`&Photo` |
| `06_mutability_and_cycles` | 6 | `Rc<RefCell<Vec<Card>>>`, 사진 → 앨범을 `Rc`로 | `Cell<u32>` 칸, 사진 → 앨범을 `Weak`로 |

모든 명령은 저장소 루트에서 실행하며, 모든 예제는 종료 코드 0으로 끝납니다. 크기가 나오는 줄은 64비트 환경의 값입니다.

### 01_borrow_the_target — 가리키는 대상으로 받기

[`examples/01_borrow_the_target.rs`](examples/01_borrow_the_target.rs)

1번 구역은 컨테이너 타입으로 받는 함수에 맞추느라 만든 임시 값의 수와 복사한 바이트 수를 셉니다. 87바이트는 `u32` 10개(40), `"수영"`(6), `Workout` 구조체(이 툴체인에서 32)와 그 안의 `"달리기"`(9)를 더한 값입니다. 2번 구역은 같은 데이터를 권장 함수에 그대로 넘기고, 3번 구역은 두 참조가 가리키는 곳을 비교합니다. 4번 구역은 컨테이너 타입이 필요한 세 경우입니다. `with_capacity(31)`은 용량이 31 이상임을 보장하므로 마지막 줄은 언제나 `true`입니다.

```sh
cargo run -p er-ch8-references-pointers --example 01_borrow_the_target
```

```text
1. 비권장: 컨테이너 타입으로 받는 함수
    평균 걸음: 이번 주 8071, 이번 달 7350, 최근 3일 8566
    수영 유산소인가: true, 오늘 운동: 달리기 30분
    함수에 맞추느라 만든 임시 값 4개, 복사한 바이트 87
2. 권장: 가리키는 대상으로 받는 함수
    평균 걸음: 이번 주 8071, 이번 달 7350, 최근 3일 8566
    수영 유산소인가: true, 요가 유산소인가: false
    오늘 운동: 달리기 30분, 기록의 첫 운동: 달리기 30분, 상자 안의 운동: 자전거 50분
    함수에 맞추느라 만든 임시 값 0개
3. &Vec<u32>는 핸들을, &[u32]는 원소를 가리킵니다 (64비트 기준)
    size_of::<&Vec<u32>>() = 8, size_of::<&[u32]>() = 16
    &Vec이 가리키는 곳이 첫 원소인가: false
    &[u32]가 가리키는 곳이 첫 원소인가: true
4. 컨테이너를 받아야 하는 경우
    10000 넘는 기록 자르기: 이번 주 최댓값 Some(10000), 이번 달 최댓값 Some(10000)
    오늘 기록을 추가한 뒤 기록 수: 11
    5일치를 더 넣어도 재할당이 없는가: true
```

### 02_wide_pointer_metadata — 넓은 포인터의 메타데이터

[`examples/02_wide_pointer_metadata.rs`](examples/02_wide_pointer_metadata.rs)

1번 구역은 위치와 개수를 따로 받는 함수가 틀린 합계를 내는 모습을, 2번 구역은 슬라이스로 넘긴 결과와 가변 슬라이스가 고칠 수 있는 범위를 보여 줍니다. 3번 구역은 `ptr::eq`와 `ptr::addr_eq`의 차이로 길이가 포인터 값의 일부임을 확인하고, 4번 구역은 트레이트 객체가 섞인 슬라이스와 `&&str`을 담은 `&dyn Display`의 크기를 잽니다.

```sh
cargo run -p er-ch8-references-pointers --example 02_wide_pointer_metadata
```

```text
1. 비권장: 범위를 위치와 개수로 따로 넘깁니다
    장마 기간 강수량: 1136mm
2. 권장: 범위의 슬라이스를 넘깁니다
    장마 기간 강수량: 131mm (5일치)
    슬라이스가 원본의 11일째를 가리키는가: true
    21~25일의 오류값 1개 정리, 범위 밖 26일째는 그대로: 999
3. 길이는 포인터 값의 일부입니다
    주소가 같은가(addr_eq): true, 포인터가 같은가(eq): false
    split_at(15): 앞 15일, 뒤 15일, 뒤쪽이 16일째를 가리키는가: true
4. 트레이트 객체는 길이 대신 vtable을 들고 다닙니다 (64비트 기준)
    ["강수량 42mm", "풍향 270도, 돌풍"]
    &[&dyn Instrument] 자체: 16바이트, 그것이 가리키는 원소들: 32바이트
    제주 관측소: &&str은 8바이트, 그것을 담은 &dyn Display는 16바이트
```

### 03_asref_or_borrow — `AsRef`와 `Borrow` 바운드

[`examples/03_asref_or_borrow.rs`](examples/03_asref_or_borrow.rs)

1·2번 구역은 같은 체크섬 함수를 `Borrow<[u8]>`와 `AsRef<[u8]>` 바운드로 만들어 받는 범위를 비교합니다. 결과 값이 같고, 달라진 것은 부르는 쪽에서 변환을 붙였느냐뿐입니다. 3번 구역은 `String`이 `Borrow<[u8]>`를 구현하지 않는 이유를 해시 비교로 보여 줍니다. `DefaultHasher::new()`는 키가 고정되어 같은 입력에 같은 해시를 내지만, 해시값 자체는 Rust 버전에 따라 바뀔 수 있으므로 "같은가"만 출력합니다. 4번 구역은 반대로 `Borrow`가 맞는 재생 목록 함수입니다.

```sh
cargo run -p er-ch8-references-pointers --example 03_asref_or_borrow
```

```text
1. 비권장: Borrow<[u8]>로 받는 함수
    그대로: Vec<u8> 60, [u8; 4] 252, &[u8] 50
    바꿔서: &str 56, String 206, &Vec<u8> 60
2. 권장: AsRef<[u8]>로 받는 함수
    &str 56, String 206, &String 206, Vec<u8> 60
    &Vec<u8> 60, [u8; 4] 252, &[u8] 50
3. String이 Borrow<[u8]>를 구현하지 않는 이유
    "ok".as_bytes() == b"ok": true, 해시가 같은가: false
    String과 str의 해시가 같은가: true (그래서 String: Borrow<str>)
4. 같은 타입을 소유한 형태로도 빌린 형태로도 받으려면 Borrow<T>
    재생 목록: ["새벽 산책", "비 오는 오후"]
    AsRef<Track> 판: 공유 목록 512초
    Borrow<Track> 판: 라이브러리 636초, 재생 목록 391초, 공유 목록 512초
```

### 04_borrow_contract — `Borrow`의 약속

[`examples/04_borrow_contract.rs`](examples/04_borrow_contract.rs)

1번 구역은 약속을 어긴 `LooseCode`로 만든 맵이 질의하는 타입에 따라 모순된 답을 내고, "없으면 추가"하는 코드가 기존 항목을 덮어쓰는 모습을 보여 줍니다. 결과가 실행마다 같은 질의만 골랐습니다(2.5절). 2번 구역은 정규화한 `AirportCode`, 3번 구역은 `Borrow` 없이 키 타입으로만 찾는 `StrictCode`입니다.

```sh
cargo run -p er-ch8-references-pointers --example 04_borrow_contract
```

```text
1. 비권장: Eq·Hash와 어긋나는 Borrow<str>
    LooseCode로 "ICN" 찾기: Some("인천"), &str로 "ICN" 찾기: None
    &str로 "CJU" 찾기: Some("제주"), "cju" 찾기: None
    ICN을 새로 추가했는데 항목 수 3, icn의 값: Some("인천(신규 등록)")
2. 권장: 만들 때 정규화해 Borrow의 약속을 지킵니다
    AirportCode로 "icn" 찾기: Some("인천"), &str로 "ICN" 찾기: Some("인천")
    &str로 " gmp " 찾기: None, 정규화한 뒤 찾기: Some("김포")
    PUS를 새로 추가한 뒤 항목 수: 4
3. 다른 선택: Borrow 없이 키 타입으로만 찾게 합니다
    CJU 찾기: Some("제주")
```

### 05_choose_owner_pointer — 소유자 수와 스레드

[`examples/05_choose_owner_pointer.rs`](examples/05_choose_owner_pointer.rs)

1번 구역은 `Vec<Box<Photo>>`와 `Vec<Photo>`에서 사진이 `Vec`의 버퍼 안에 있는지를 주소로만 확인합니다(`lies_in`은 주소를 비교할 뿐 역참조하지 않습니다). 따로 할당된 두 메모리는 겹치지 않으므로 `false`는 언제나 같은 결과입니다. 2번 구역은 앨범 둘에 같은 사진을 넣을 때 실제 픽셀 버퍼가 몇 벌인지 세고, 3번 구역은 `spawn` + `Arc`와 스코프 스레드 + `&Photo`로 같은 계산을 합니다. 스레드 안에서는 출력하지 않고 결과를 돌려받아 출력하므로 출력 순서가 일정합니다.

```sh
cargo run -p er-ch8-references-pointers --example 05_choose_owner_pointer
```

```text
1. 소유자가 하나: 값을 그대로 둡니다 (64비트 기준)
    비권장 Vec<Box<Photo>>: 버퍼 16바이트, 사진이 버퍼 안에 있는가: false
    권장 Vec<Photo>: 버퍼 96바이트, 사진이 버퍼 안에 있는가: true
    두 앨범의 두 번째 사진: 숲 / 숲
2. 소유자가 여럿(같은 스레드): 복제 대신 Rc로 공유합니다
    복제: 픽셀 버퍼 2벌, 2000000바이트
    Rc: 픽셀 버퍼 1벌, 1000000바이트, 소유자 3명, 같은 사진인가: true
3. 스레드를 넘는가: spawn이면 Arc, 스코프 스레드면 &T로 충분합니다
    spawn + Arc: 밝기 [124, 124], 스레드가 끝난 뒤 소유자 1명
    scope + &Photo: 밝기 [124, 124], 카운트 없음
```

### 06_mutability_and_cycles — 내부 가변성의 범위와 순환

[`examples/06_mutability_and_cycles.rs`](examples/06_mutability_and_cycles.rs)

1·2번 구역은 목록 전체를 `RefCell`로 감쌌을 때와 좋아요 칸만 `Cell`로 두었을 때, 목록을 읽는 도중의 좋아요가 어떻게 되는지 비교합니다. 이중 대여는 패닉 대신 `try_borrow_mut`으로 관찰합니다. 3·4번 구역은 사진이 앨범을 `Rc`로 가리킬 때와 `Weak`로 가리킬 때 앨범이 해제되는지 비교합니다. 해제 메시지는 4번 구역에서만 나옵니다.

```sh
cargo run -p er-ch8-references-pointers --example 06_mutability_and_cycles
```

```text
1. 비권장: 목록 전체를 RefCell로 감싼 Rc<RefCell<Vec<Card>>>
    그리는 중 ["등대", "노을"], 좋아요 결과: Err("RefCell already borrowed")
    다 그린 뒤 좋아요 결과: Ok(1)
2. 권장: 바뀌는 칸만 Cell로 둔 Rc<Vec<LiveCard>>
    그리는 중 ["등대", "노을"], 좋아요 결과: 1
3. 비권장: 사진이 앨범을 Rc로 가리키면 순환이 생깁니다
    사진이 속한 앨범: "여행"
    핸들을 버린 뒤: 앨범을 붙잡은 강한 참조 1개, 아직 살아 있는가: true
4. 권장: 거꾸로 가리키는 쪽(사진 → 앨범)을 Weak로 둡니다
    사진이 속한 앨범: Some("가족")
    [앨범 '가족' 해제]
    핸들을 버린 뒤 사진이 속한 앨범: None
```

## 4. 자주 하는 실수

4.1절과 4.3절은 **에러가 부르는 쪽 줄에서 나지만 원인은 받는 쪽 시그니처**에 있습니다. 받는 쪽을 고칠 수 있다면 받는 쪽을 고칩니다. 직접 확인하려면 챕터 디렉터리에서 다음을 실행합니다(`--out-dir`은 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
rustc --edition 2024 compile_fail/e0308_vec_param_rejects_array.rs --out-dir /tmp/er-ch8
rustc --edition 2024 compile_fail/e0277_str_to_dyn_display.rs --out-dir /tmp/er-ch8
rustc --edition 2024 compile_fail/e0277_asref_not_reflexive.rs --out-dir /tmp/er-ch8
rustc --edition 2024 compile_fail/e0308_hashmap_get_without_borrow.rs --out-dir /tmp/er-ch8
```

### 4.1 `&Vec<T>` 매개변수에 배열을 넘김 — E0308

[`compile_fail/e0308_vec_param_rejects_array.rs`](compile_fail/e0308_vec_param_rejects_array.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_vec_param_rejects_array.rs:17:32
   |
17 |     println!("{}", average_vec(&week)); // E0308
   |                    ----------- ^^^^^ expected `&Vec<u32>`, found `&[u32; 7]`
   |                    |
   |                    arguments to this function are incorrect
   |
   = note: expected reference `&Vec<u32>`
              found reference `&[u32; 7]`
note: function defined here
  --> compile_fail/e0308_vec_param_rejects_array.rs:11:4
   |
11 | fn average_vec(days: &Vec<u32>) -> u32 {
   |    ^^^^^^^^^^^ ---------------
```

- **읽는 법**: 마지막 `note`가 받는 쪽 매개변수 `days: &Vec<u32>`를 가리킵니다. 배열의 참조 `&[u32; 7]`은 슬라이스 참조로는 강제 변환되지만, `Vec`은 힙 버퍼를 소유하는 별개의 타입이라 어떤 강제 변환으로도 만들어지지 않습니다. 컴파일러는 고치는 방법을 제안하지 않습니다.
- **해결**: 받는 쪽을 `days: &[u32]`로 바꿉니다(예제 01의 `average`). 받는 쪽을 고칠 수 없다면 부르는 쪽에서 `&week.to_vec()`으로 복사해 넘깁니다. 할당과 복사가 생기는데, 이것이 지침 1이 막으려는 비용입니다.

### 4.2 `&str`을 `&dyn Display`로 바꾸려 함 — E0277

[`compile_fail/e0277_str_to_dyn_display.rs`](compile_fail/e0277_str_to_dyn_display.rs)

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
  --> compile_fail/e0277_str_to_dyn_display.rs:19:27
   |
19 |     println!("{}", framed(station)); // E0277
   |                           ^^^^^^^ doesn't have a size known at compile-time
   |
   = help: the trait `Sized` is not implemented for `str`
   = note: required for the cast from `&str` to `&dyn std::fmt::Display`
help: consider borrowing the value, since `&&str` can be coerced into `&dyn std::fmt::Display`
   |
19 |     println!("{}", framed(&station)); // E0277
   |                           +
```

- **읽는 법**: `str`은 `Display`를 구현하지만, `note`가 말하듯 `&str`에서 `&dyn Display`로 바꾸는 데 `str: Sized`가 필요합니다. 트레이트 객체로 바꿀 수 있는 것은 크기가 정해진 타입의 참조뿐입니다. 2.3절의 말로 하면 `&str`은 이미 길이를 실은 넓은 포인터라 vtable을 실을 자리가 없습니다. 크기를 모르는 값을 지역 변수에 담으려 할 때의 같은 에러 코드(Part 0 ch19 4.6절)와 달리, 여기서는 포인터를 바꾸는 과정에서 납니다.
- **해결**: 도움말대로 `&station`(`&&str`)을 넘깁니다. 한 겹 더 감싼 얇은 포인터가 vtable을 싣습니다. 받는 쪽을 고칠 수 있다면 트레이트 객체 대신 제네릭 `item: &(impl Display + ?Sized)`로 받는 방법도 있습니다. 이때는 vtable이 필요 없으므로 `&str`이 그대로 들어갑니다(이 툴체인에서 확인).

### 4.3 `AsRef<T>` 바운드에 `T` 자신과 `&T`를 넘김 — E0277

[`compile_fail/e0277_asref_not_reflexive.rs`](compile_fail/e0277_asref_not_reflexive.rs)

```text
error[E0277]: the trait bound `Track: AsRef<Track>` is not satisfied
  --> compile_fail/e0277_asref_not_reflexive.rs:27:38
   |
27 |     println!("{}", total_secs_as_ref(&library)); // E0277: Track: AsRef<Track>
   |                    ----------------- ^^^^^^^^ unsatisfied trait bound
   |                    |
   |                    required by a bound introduced by this call
   |
help: the trait `AsRef<Track>` is not implemented for `Track`
  --> compile_fail/e0277_asref_not_reflexive.rs:11:1
   |
11 | struct Track {
   | ^^^^^^^^^^^^
note: required by a bound in `total_secs_as_ref`
  --> compile_fail/e0277_asref_not_reflexive.rs:16:25
   |
16 | fn total_secs_as_ref<T: AsRef<Track>>(list: &[T]) -> u32 {
   |                         ^^^^^^^^^^^^ required by this bound in `total_secs_as_ref`
```

28행의 `&playlist`(`&[&Track]`)에서도 같은 에러가 나고, ``= note: required for `&Track` to implement `AsRef<Track>` ``가 더 붙습니다(생략).

- **읽는 법**: `help`가 `Track`에 `AsRef<Track>`이 없다고 알려 줍니다. `AsRef`는 반사적이지 않으므로 자기 자신으로 보는 구현이 저절로 생기지 않습니다. `&Track`의 `AsRef<Track>`도 `Track: AsRef<Track>`에 기대는 포괄 구현이라 함께 실패합니다.
- **해결**: 같은 타입의 소유한 형태와 빌린 형태를 받는 것이 목적이므로 바운드를 `T: Borrow<Track>`으로 바꿉니다(예제 03의 `total_secs`). 내 타입이라면 `impl AsRef<Track> for Track`을 직접 써서 맞출 수도 있지만, 표준 라이브러리 타입(`String: AsRef<String>`은 없습니다)에는 그럴 수 없으므로 `Borrow`가 일반적인 답입니다.

### 4.4 `Borrow<str>` 없는 키 타입의 맵을 `&str`로 찾음 — E0308

[`compile_fail/e0308_hashmap_get_without_borrow.rs`](compile_fail/e0308_hashmap_get_without_borrow.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_hashmap_get_without_borrow.rs:26:35
   |
26 |     println!("{:?}", airports.get("ICN")); // E0308
   |                               --- ^^^^^ expected `&StrictCode`, found `&str`
   |                               |
   |                               arguments to this method are incorrect
   |
   = note: expected reference `&StrictCode`
              found reference `&'static str`
note: method defined here
  --> /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/std/src/collections/hash/map.rs:1034:11
```

- **읽는 법**: `get<Q>`의 바운드는 `StrictCode: Borrow<Q>`인데, 이 타입이 만족하는 `Borrow`는 반사 구현인 `Borrow<StrictCode>` 하나뿐입니다. 컴파일러는 `Q = StrictCode`로 정한 뒤 인자 `&str`이 `&StrictCode`와 맞지 않는다고 알립니다. 바운드 문제처럼 보이지만 E0277이 아니라 E0308로 나온다는 점에 주의합니다.
- **해결**: 2.5절의 판단에 따릅니다. 약속을 지킬 수 있는 타입(정규화한 `AirportCode`)이면 `impl Borrow<str>`를 추가하고, 지킬 수 없으면 `airports.get(&StrictCode::new("ICN"))`처럼 키 타입을 만들어 찾습니다. 이 에러가 나는 것 자체가 원하는 결과일 수 있습니다.

## 5. 연습 문제

### 연습 1. 시그니처 고르기

다음 함수의 매개변수 타입을 이 챕터의 기준으로 고르고, 예제 파일 하나에 직접 써서 clippy로 확인하세요.

1. `fn longest(names: &Vec<String>) -> Option<&String>`: 가장 긴 이름을 돌려줍니다.
2. `fn trim_all(tags: &mut Vec<String>)`: 각 태그의 앞뒤 공백을 제자리에서 지웁니다.
3. `fn dedup_tags(tags: &mut Vec<String>)`: 정렬한 뒤 중복을 지웁니다.
4. `fn register(cache: &mut Vec<Rc<Photo>>, photo: &Rc<Photo>)`: 사진을 언제나 캐시에 넣습니다.

<details>
<summary>힌트</summary>

1번은 읽기만 하므로 `&[String]`이고 결과는 그대로 `Option<&String>`입니다(`ptr_arg`의 제안도 같습니다). 2번은 각 `String`의 내용은 바꾸지만 목록의 길이는 그대로이므로 `&mut [String]`입니다. 3번은 `dedup`이 길이를 바꾸므로 `&mut Vec<String>`이 맞고, clippy도 경고하지 않습니다. 4번의 `cache`는 `push`하므로 `&mut Vec<Rc<Photo>>`가 맞고, `photo`는 언제나 보관하므로 `Rc<Photo>`를 값으로 받아 부르는 쪽이 `Rc::clone`할지 옮길지 정하게 합니다.

</details>

### 연습 2. 슬라이스 기준의 위치

예제 02에 `fn wettest(days: &[u32]) -> Option<(usize, u32)>`를 추가해 가장 비가 많이 온 날의 위치와 강수량을 돌려주게 하세요. 그런 다음 `wettest(&month[10..20])`(11~20일)이 돌려주는 위치를 "그 달의 며칠째"로 바꿔 출력해 보세요.

<details>
<summary>힌트</summary>

`days.iter().copied().enumerate().max_by_key(|&(_, mm)| mm)`로 구할 수 있습니다. 돌려받는 값은 `Some((1, 42))`이고, 위치는 넘겨받은 슬라이스 기준이라 `1`(실제로는 12일째)이 나오므로, 원래 배열의 위치로 바꾸려면 부르는 쪽이 시작 위치 10을 더해야 합니다. 위치가 결과인 함수는 이렇게 "어디를 기준으로 한 위치인가"를 문서에 적어 둡니다. 이것이 2.3절에서 말한 "위치 자체가 결과일 때"입니다.

</details>

### 연습 3. `BTreeMap`도 같은 약속에 기댑니다

예제 04의 `LooseCode`에 대소문자를 무시하는 `PartialOrd`와 `Ord`를 구현하고(대문자로 바꾼 문자열끼리 비교), 같은 세 항목으로 `BTreeMap<LooseCode, &str>`을 만들어 `get(&LooseCode::new("ICN"))`, `get("ICN")`, `get("icn")`의 결과를 비교해 보세요.

<details>
<summary>힌트</summary>

`BTreeMap::get`도 `K: Borrow<Q>`와 `Q: Ord`를 요구합니다. 키는 `LooseCode`의 `Ord`(대소문자 무시)로 정렬되어 들어갔는데, `&str`로 찾을 때는 `str`의 `Ord`(대문자가 소문자보다 앞)로 비교하며 찾아갑니다. 이 툴체인에서 확인한 결과는 `Some("인천")`, `None`, `Some("인천")`입니다. 항목이 많아져 트리가 여러 단계가 되면 어긋나는 질의가 더 늘어납니다. `AirportCode`처럼 정규화하면 세 질의가 일관된 답을 냅니다.

</details>

### 연습 4. 포인터 고르기

다음 상황마다 2.6절의 네 질문을 차례로 적용해 타입을 고르세요.

1. 프로그램이 시작할 때 읽은 설정을 작업자 스레드 4개가 `thread::spawn`으로 실행되는 동안 계속 읽습니다.
2. 편집기에서 문서 하나를 창 여러 개가 함께 보여 주고, 어느 창에서든 문서를 고칠 수 있습니다(한 스레드).
3. 폴더 트리에서 각 폴더가 하위 폴더 목록을 가지고, 하위 폴더는 상위 폴더의 경로를 알아야 합니다.
4. 명령 열거형의 배리언트 하나가 4KB짜리 배열을 담고, 나머지 배리언트는 데이터가 거의 없습니다.

<details>
<summary>힌트</summary>

1번은 소유자가 여럿이고 `spawn`으로 스레드를 넘으며 읽기만 하므로 `Arc<Config>`입니다(③④는 "아니다"). 2번은 소유자가 여럿, 한 스레드, 고치기가 필요하고 문서 전체를 바꾸므로 `Rc<RefCell<Document>>`가 출발점입니다. 3번은 상위 → 하위를 `Rc`(또는 값)로 소유하고, 하위 → 상위를 `Weak`로 둡니다. 4번은 소유자가 하나지만 큰 값 때문에 열거형 전체가 커지므로 그 배리언트만 `Box<[u8; 4096]>`으로 감쌉니다. clippy의 `large_enum_variant`가 알려 주는 상황입니다(Part 1 ch7 2.7절).

</details>

## 6. 참고 자료

- [Effective Rust, Item 8: Familiarize yourself with reference and pointer types](https://effective-rust.com/references.html): 이 챕터가 다루는 지침의 출처입니다. 참조, 포인터 트레이트, 넓은 포인터, 스마트 포인터를 차례로 다룹니다.
- [`std::convert::AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html), [`std::borrow::Borrow`](https://doc.rust-lang.org/std/borrow/trait.Borrow.html): 두 트레이트의 문서는 각각 상대와의 차이를 설명하고, `Borrow` 문서는 `Eq`·`Ord`·`Hash`가 같아야 한다는 요구 사항을 적고 있습니다.
- [`std::ptr::eq`](https://doc.rust-lang.org/std/ptr/fn.eq.html), [`std::ptr::addr_eq`](https://doc.rust-lang.org/std/ptr/fn.addr_eq.html): 넓은 포인터 비교에서 메타데이터를 어떻게 다루는지, 트레이트 객체 포인터의 비교가 왜 믿을 수 없는지 설명합니다.
- [rustc 린트 목록: `ambiguous_wide_pointer_comparisons`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#ambiguous-wide-pointer-comparisons): 넓은 원시 포인터를 `==`로 비교할 때 나는 경고입니다.
- [The Rust Reference: Dynamically Sized Types](https://doc.rust-lang.org/reference/dynamically-sized-types.html): 동적 크기 타입과 그것을 가리키는 넓은 포인터의 규칙입니다.
- [Clippy 린트 목록(1.100.0)](https://clippy.rust-lang.org/rust-1.100.0/index.html): `ptr_arg`, `borrowed_box`, `vec_box`, `box_collection`, `redundant_allocation`의 설명과 예가 있습니다.
- 본편: [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md)(참조와 슬라이스), [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)(`Box`·`Rc`·`RefCell`·`Weak`, 역참조 강제 변환), [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)(트레이트 객체와 vtable), [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)(`AsRef`·`Borrow`·`Cow`), [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md)(크기, 넓은 포인터, `Cell`), [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md)(`Arc`·`Mutex`·`RwLock`, 스코프 스레드)
