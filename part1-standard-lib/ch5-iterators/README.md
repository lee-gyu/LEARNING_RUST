# ch5. 반복자: `Iterator`, `IntoIterator`, `FromIterator`와 어댑터

[Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md)에서 반복자의 기초를 배웠습니다. `Iterator` 트레이트와 `next`, 반복자가 게으르다는 것, `for` 문이 `next`를 되풀이해 부른다는 것, `iter`/`iter_mut`/`into_iter`의 차이, `map`·`filter`·`collect` 같은 기본 어댑터와 소비 메서드, `next` 하나로 직접 만든 반복자가 그것입니다. 이 챕터는 그 위에서 세 방향으로 넓힙니다.

- **세 트레이트의 관계**: `for`가 부르는 `IntoIterator`, 반복자 자체인 `Iterator`, `collect`가 부르는 `FromIterator`(와 `Extend`). 앞 챕터들이 "나중에 다룬다"고 미뤄 둔 질문, 예를 들어 `for x in &v`가 왜 `v.iter()`와 같은지, `collect`가 어떻게 `Vec`도 `String`도 `Result<Vec<_>, _>`도 만드는지가 모두 이 세 트레이트로 설명됩니다.
- **어댑터와 소비 메서드의 지도**: `Iterator` 문서에는 메서드가 70개가 넘습니다. 이것을 "무엇을 하고 싶은가"로 묶어 고르는 법을 익힙니다.
- **보조 트레이트와 반복자 만들기**: `rev()`와 `len()`이 되는 반복자와 안 되는 반복자를 문서의 `impl` 블록에서 가려내고, 직접 만든 컬렉션을 `for`와 `collect`에 쓸 수 있게 만듭니다.

- 패키지: `p1-ch5-iterators`
- 기준 문서: [`std::iter`](https://doc.rust-lang.org/std/iter/index.html), [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html), [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html), [`FromIterator`](https://doc.rust-lang.org/std/iter/trait.FromIterator.html), [`Extend`](https://doc.rust-lang.org/std/iter/trait.Extend.html), [`DoubleEndedIterator`](https://doc.rust-lang.org/std/iter/trait.DoubleEndedIterator.html), [`ExactSizeIterator`](https://doc.rust-lang.org/std/iter/trait.ExactSizeIterator.html), [`Peekable`](https://doc.rust-lang.org/std/iter/struct.Peekable.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `Iterator`·`IntoIterator`·`FromIterator`·`Extend`가 각각 무엇을 정하는지 설명할 수 있다. `for x in &v`가 `v.iter()`와 같은 이유, `collect`가 결과 타입에 따라 `Vec`·`String`·`Result<Vec<_>, _>`를 만드는 이유를 API 문서의 구현 목록에서 찾아 설명할 수 있다.
2. 하고 싶은 일(바꾸기·거르기, 자르기·건너뛰기, 합치기, 상태 들고 가기)에 맞는 어댑터를 골라 체인을 작성할 수 있다. 어댑터가 저마다 구조체(`Map<Filter<…>>`)를 돌려주기 때문에 게으르고 추가 비용이 없다는 것을 설명할 수 있다.
3. 소비 메서드(접기, 찾기와 묻기, 실패할 수 있는 접기, 나누기, 비교)를 상황에 맞게 고르고, 어떤 메서드가 답을 얻는 즉시 멈추는지(단락 평가) 설명할 수 있다.
4. `DoubleEndedIterator`·`ExactSizeIterator`·`size_hint`의 뜻을 설명하고, `Drain<'_, T>`나 `Enumerate<I>` 같은 반복자 구조체 문서의 `impl` 블록을 읽어 `rev()`와 `len()`을 쓸 수 있는지 판단할 수 있다.
5. `std::iter`의 함수와 `impl Iterator` 반환으로 반복자를 만들고, 직접 만든 컬렉션에 `IntoIterator`(값·`&`·`&mut`)와 `FromIterator`·`Extend`를 구현할 수 있다.

## 2. 핵심 개념

2.1~2.3절은 세 트레이트(`partition`·`unzip` 포함), 2.4~2.9절은 어댑터, 2.10~2.14절은 소비 메서드, 2.15~2.16절은 보조 트레이트와 반복자 구조체 읽기, 2.17~2.19절은 반복자 만들기를 다룹니다.

### 2.1 세 트레이트의 역할 분담

반복자로 하는 일은 거의 언제나 "컬렉션에서 꺼내서 → 가공하고 → 다시 모은다"입니다. 세 단계를 각각 다른 트레이트가 맡습니다.

```text
  Vec, &Vec, 배열, 맵, 범위 …          map, filter, take …         Vec, String, BTreeMap, Result<Vec<_>, _> …
  ┌────────┐  IntoIterator::into_iter  ┌────────┐   어댑터    ┌────────┐  FromIterator::from_iter  ┌────────┐
  │ 컬렉션 │ ────────────────────────▶ │ 반복자 │ ──────────▶ │ 반복자 │ ────────────────────────▶ │ 컬렉션 │
  └────────┘     (for 문이 부름)        └────────┘ (Iterator)  └────────┘   (collect가 부름)         └────────┘
                                                                         Extend::extend: 있는 컬렉션에 더하기
```

| 트레이트 | 필수 메서드 | 누가 부르나 | 무엇을 정하나 |
| --- | --- | --- | --- |
| `Iterator` | `next(&mut self) -> Option<Self::Item>` | 어댑터, 소비 메서드, `for` | 다음 원소를 어떻게 꺼내는가 |
| `IntoIterator` | `into_iter(self) -> Self::IntoIter` | `for` 문, `chain`·`zip`·`extend` 등의 인자 | 이 값을 어떤 반복자로 바꾸는가, 원소 타입은 무엇인가 |
| `FromIterator<T>` | `from_iter(iter) -> Self` | `collect` | 원소 `T`들로 이 타입을 어떻게 만드는가 |
| `Extend<T>` | `extend(&mut self, iter)` | `extend`, `partition`, `unzip` | 원소 `T`들을 이미 있는 값에 어떻게 더하는가 |

앞 챕터에서 미뤄 둔 질문이 어디서 풀리는지 정리하면 다음과 같습니다.

| 앞 챕터의 질문 | 답이 있는 절 |
| --- | --- |
| `for x in &v`가 왜 `v.iter()`와 같은가 (Part 0 ch13) | 2.2 |
| `collect::<String>()`이 왜 되는가 (ch3) | 2.3 |
| `Result<Vec<_>, _>`, `Option<Vec<_>>`로 `collect`할 수 있는 원리 (ch2) | 2.3 |
| `collect`가 컬렉션마다 다른 결과를 만드는 원리 (ch4) | 2.3 |
| `filter_map`, 반복자의 `flatten`, `copied`/`cloned` (ch2, ch4) | 2.5 |
| `fold`, `chain`, `peekable` (Part 0 ch13) | 2.7, 2.8, 2.10 |
| `position`, `partition` (ch2, ch4) | 2.11, 2.3 |
| `next_back`, `drain`·`range`·집합 연산이 돌려주는 타입 (ch4) | 2.15, 2.16 |

### 2.2 `IntoIterator`: `for`가 부르는 것

```rust
pub trait IntoIterator {
    type Item;                                  // 반복 변수가 받는 값의 타입
    type IntoIter: Iterator<Item = Self::Item>; // into_iter가 돌려주는 반복자의 타입
    fn into_iter(self) -> Self::IntoIter;
}
```

`std::iter` 모듈 문서는 `for` 문이 다음과 같이 풀린다고 설명합니다(문서의 코드를 줄였습니다).

```rust
for x in values { /* 본문 */ }
// 컴파일러가 바꾼 모양
match IntoIterator::into_iter(values) {
    mut iter => loop {
        match iter.next() {
            Some(x) => { /* 본문 */ }
            None => break,
        }
    },
}
```

`for`는 먼저 `into_iter`를 부르고, 그 반복자에 `next`를 되풀이합니다. 그러니 **`for`에 무엇을 넣을 수 있는지, 반복 변수 `x`가 어떤 타입인지는 그 값이 `IntoIterator`를 어떻게 구현했는지로 정해집니다.**

**`Vec`의 세 가지 구현**: `Vec` 문서의 Trait Implementations에는 `IntoIterator` 구현이 세 개 있습니다. 예제 01이 `type_name`으로 출력한 연관 타입을 표로 옮기면 다음과 같습니다.

| `for x in …` | 골라지는 구현 | `IntoIter` | `Item` |
| --- | --- | --- | --- |
| `v` | `impl<T> IntoIterator for Vec<T>` | `vec::IntoIter<T>` | `T` |
| `&v` | `impl<'a, T> IntoIterator for &'a Vec<T>` | `slice::Iter<'a, T>` | `&'a T` |
| `&mut v` | `impl<'a, T> IntoIterator for &'a mut Vec<T>` | `slice::IterMut<'a, T>` | `&'a mut T` |

`&v`의 `IntoIter`가 `v.iter()`의 반환 타입 `slice::Iter`와 **같은 타입**입니다. 실제로 `&Vec`의 `into_iter`는 `self.iter()`를 부를 뿐입니다. 이것이 `for x in &v`와 `for x in v.iter()`가 같은 이유입니다. 모듈 문서는 이것을 관례로 적어 둡니다. "컬렉션 `C`가 `iter()`를 제공하면 보통 `&C`에도 `IntoIterator`를 구현하고, `iter_mut()`를 제공하면 `&mut C`에도 구현한다." 반대로 `HashSet`에는 `iter_mut()`가 없습니다. 원소를 고쳐 해시가 바뀌면 집합이 망가지기 때문이라고 문서는 설명합니다.

`Vec<T>`의 `into_iter`는 `self`를 값으로 받으므로 `for x in v` 뒤에는 `v`를 쓸 수 없습니다(4.1절). 읽기만 할 때는 `&v`를 넘기는 습관을 들입니다.

**다른 타입들**: 예제 01의 3번 구역에서 확인한 것입니다.

- 배열 `[T; N]`: 값으로 넘기면 `T`를 내놓습니다(2021 에디션부터). `&[T; N]`은 `&T`입니다.
- `Option<T>`: `Some(v)`면 `v` 하나, `None`이면 아무것도 내놓지 않습니다. 그래서 `chain(Some(x))`, `extend(opt)`처럼 쓸 수 있습니다(ch2 2.11절).
- 맵: `BTreeMap<K, V>`는 `(K, V)`를, `&BTreeMap<K, V>`는 `(&K, &V)`를 내놓습니다. `for (k, v) in &map`의 튜플 패턴이 이것입니다.
- `String`은 `IntoIterator`가 **아닙니다.** 글자(`chars`)로 돌지 바이트(`bytes`)로 돌지 정할 수 없기 때문입니다. `for c in s`는 E0277이고, 컴파일러가 ``try calling `.chars()` or `.bytes()` ``라고 알려 줍니다.

**반복자도 `IntoIterator`입니다**: 표준 라이브러리에 `impl<I: Iterator> IntoIterator for I`라는 포괄 구현이 있어서, 모든 반복자는 자기 자신을 돌려주는 `into_iter`를 가집니다. 범위 `a..b`처럼 그 자체가 반복자인 값이나 `for x in v.iter().rev()`처럼 어댑터를 붙인 반복자를 `for`에 넣을 수 있는 이유입니다.

**`IntoIterator`를 받는 함수**: 매개변수를 `impl IntoIterator<Item = u32>`로 받으면 `Vec`, 배열, 범위, `Option`, 집합, 반복자를 모두 넘길 수 있습니다. 표준 라이브러리도 `chain`, `zip`, `extend`의 인자를 이렇게 받습니다.

```rust
fn total(prices: impl IntoIterator<Item = u32>) -> u32 { /* for p in prices { … } */ }
total(vec![1200, 3000]); total([1000, 2000]); total(1..=4); total(Some(800)); total(cart.iter().copied());
```

`total(&cart)`는 원소가 `&u32`라서 `Item = u32`에 맞지 않습니다. 그럴 때 `.iter().copied()`로 원소를 값으로 바꿔 넘깁니다. 예제는 [`01_into_iterator.rs`](examples/01_into_iterator.rs)입니다.

### 2.3 `FromIterator`와 `Extend`: `collect`가 부르는 것

```rust
fn collect<B>(self) -> B where B: FromIterator<Self::Item>, Self: Sized;

pub trait FromIterator<T>: Sized {
    fn from_iter<I>(iter: I) -> Self where I: IntoIterator<Item = T>;
}
```

`collect`는 아무것도 결정하지 않습니다. 결과 타입 `B`의 `FromIterator::from_iter`를 부를 뿐이고, `let a: Vec<i32> = (1..=3).collect();`와 `let b = Vec::from_iter(1..=3);`은 같은 일을 합니다. 그래서 **결과 타입이 동작을 정합니다.** 예제 02의 1번 구역은 같은 `["사과", "배", "사과", "감"]`을 여러 타입으로 모읍니다.

| 결과 타입 | 결과 | 그렇게 되는 이유 |
| --- | --- | --- |
| `Vec<&str>` | `["사과", "배", "사과", "감"]` | 순서대로 쌓음 |
| `BTreeSet<&str>` | `{"감", "배", "사과"}` | 집합이라 중복을 버리고 정렬함 |
| `String` | `"사과배사과감"` | `String: FromIterator<&str>`는 이어 붙임 |
| `BTreeMap<&str, usize>` | `{"감": 1, "배": 1, "사과": 2}` | 원소가 `(키, 값)` 튜플이면 맵. 같은 키는 나중 값이 남음 |

`FromIterator` 문서의 Implementors 목록에서 자주 쓰는 것을 골랐습니다(1.100.0 문서).

| 구현 | 뜻 |
| --- | --- |
| `impl<T> FromIterator<T> for Vec<T>` (`VecDeque`, `LinkedList`, `Box<[T]>`도) | 원소를 그대로 모음 |
| `impl<T: Ord> FromIterator<T> for BTreeSet<T>`, `HashSet`, `BinaryHeap` | 집합·힙. 키 조건(ch4 2.11절)이 그대로 붙음 |
| `impl<K: Ord, V> FromIterator<(K, V)> for BTreeMap<K, V>`, `HashMap` | 튜플을 키·값으로 |
| `impl FromIterator<char> for String`, `<&char>`, `<&str>`, `<String>`, `<Cow<str>>` 등 | 글자나 문자열 조각을 이어 붙임 |
| `impl<T, E, V> FromIterator<Result<T, E>> for Result<V, E> where V: FromIterator<T>` | 모두 `Ok`면 `Ok(V)`, 아니면 첫 `Err` |
| `impl<T, V> FromIterator<Option<T>> for Option<V> where V: FromIterator<T>` | 모두 `Some`이면 `Some(V)`, 아니면 `None` |
| `impl<T, ExtendT> FromIterator<(T₁, …, Tₙ)> for (ExtendT₁, …, ExtendTₙ)` (1.79부터) | 튜플 원소를 자리별로 나눠 모음 |
| `impl FromIterator<()> for ()` | 원소를 버림. `Result<(), E>`로 모아 "모두 성공했나"만 볼 때 쓰임 |

**`collect::<String>()`이 되는 원리(ch3)**: `String`이 `FromIterator<char>`, `FromIterator<&str>`, `FromIterator<String>` 등을 구현하기 때문입니다. `"반복자".chars().rev().collect::<String>()`은 `char`를, `["가", "나"].into_iter().collect::<String>()`은 `&str`를 이어 붙입니다. 목록에 없는 원소 타입은 안 됩니다. `["가", "나"].iter()`의 원소는 `&&str`이고 `String: FromIterator<&&str>`는 없으므로 E0277입니다(4.6절). `into_iter()`나 `.copied()`로 원소를 `&str`로 만들어 모읍니다.

**`Result<Vec<_>, _>`로 모을 수 있는 원리(ch2)**: 위 표의 `Result` 구현을 읽어 봅니다. "원소가 `Result<T, E>`이고, **안쪽 값 `T`로 `V`를 만들 수 있으면**(`V: FromIterator<T>`) `Result<V, E>`를 만들 수 있다." `V` 자리에 `Vec`이 오면 ch2에서 쓴 `Result<Vec<T>, E>`이고, `BTreeSet`이 오면 `Result<BTreeSet<T>, E>`, `String`이 오면 `Result<String, E>`입니다. 구현은 원소를 하나씩 꺼내다 `Err`를 만나면 거기서 멈추고 그 `Err`를 돌려줍니다(ch2 예제 08의 단락 평가). `Option`도 같은 모양이라 `[3, 1, 4].into_iter().map(|d| char::from_digit(d, 10))`을 `Option<String>`으로 모으면 `Some("314")`, 하나라도 `None`이면 `None`입니다. 이 구성 방식 덕분에, 직접 만든 타입에 `FromIterator<T>`만 구현하면 `Result<내 타입, E>`로 모으기는 저절로 따라옵니다(2.19절).

**튜플로 모으기와 `unzip`**: 원소가 `(A, B)`인 반복자는 `unzip()`으로 두 컬렉션에 나눠 담습니다. 1.79부터는 `collect`도 튜플을 결과 타입으로 받습니다. 각 자리의 타입은 `Default + Extend`면 되므로 `(BTreeSet<&str>, String)`처럼 자리마다 다른 컬렉션을 써도 됩니다.

**`Extend`: 있는 컬렉션에 더하기**: `collect`가 새 값을 만든다면 `extend`는 이미 있는 값 뒤에 붙입니다. 인자는 `IntoIterator`라서 배열, `Option`, 반복자를 모두 받습니다.

```rust
log.extend([3, 4]);                             // Vec<T>: Extend<T>
log.extend(&[5, 6]);                            // Vec<T>: Extend<&T> (T: Copy일 때)
title.extend([':', ' ']);                       // String: Extend<char>
title.extend(["반복자", "와 어댑터"]);          // String: Extend<&str>
stock.extend([("배", 5), ("사과", 9)]);         // BTreeMap: 같은 키는 덮어씀
```

**`partition`은 `Default + Extend`로 만들어집니다**: 시그니처가 `fn partition<B, F>(self, f: F) -> (B, B) where B: Default + Extend<Self::Item>, F: FnMut(&Self::Item) -> bool`입니다. 빈 값 두 개를 `Default`로 만들고, 조건이 `true`인 원소는 왼쪽에, `false`인 원소는 오른쪽에 `extend`합니다. 그래서 `(Vec<i32>, Vec<i32>)`뿐 아니라 `(String, String)`으로도 나눌 수 있습니다. 예제는 [`02_from_iterator.rs`](examples/02_from_iterator.rs)입니다.

### 2.4 어댑터는 구조체를 돌려줍니다

`map`의 시그니처는 `fn map<B, F>(self, f: F) -> Map<Self, F>`입니다. `map`을 부르면 클로저가 실행되는 것이 아니라 **"안쪽 반복자 + 클로저"를 필드로 가진 `Map` 구조체**가 만들어질 뿐입니다. 예제 03은 `scores.iter().filter(|s| **s >= 70).map(|s| s / 10)`의 타입을 `std::any::type_name`(타입의 이름을 문자열로 돌려주는 함수, `std::any`는 `ch14-ffi-any-misc`에서 다룹니다)으로 출력합니다. 결과(3절)를 줄여 쓰면 `Map<Filter<slice::Iter<'_, i32>, {{closure}}>, {{closure}}>`이고, 바깥부터 읽으면 `Map(Filter(slice::Iter, 클로저), 클로저)`입니다. 어댑터를 붙일 때마다 한 겹씩 감쌉니다. `type_name`은 타입이 정의된 곳(`core::iter::adapters::map`)을 보여 주지만, 문서에서는 다시 내보낸 경로인 `std::iter::Map`으로 찾습니다(`core`와 `std`의 관계는 [ch1](../ch1-std-overview/README.md)). `type_name`의 결과는 디버깅용이라 툴체인에 따라 바뀔 수 있습니다.

**왜 게으른가**: `Map` 구조체의 `next`가 불릴 때 비로소 안쪽 반복자의 `next`를 부르고, 받은 원소에 클로저를 적용합니다. 소비 메서드가 맨 바깥의 `next`를 부르면 그 요청이 안쪽으로 전달되고, 원소 하나가 체인을 끝까지 통과한 뒤에 다음 원소를 꺼냅니다. 예제 03의 2번 구역은 `inspect`로 이 흐름을 찍습니다. `take(2)`가 두 개를 받자 더 요청하지 않으므로 뒤의 원소는 꺼내지도 않습니다.

**왜 비용이 없는가**: 예제 03의 1번 구역에서 크기를 재면(`std::mem::size_of_val`, 값이 차지하는 바이트 수. `ch7-smart-pointers-memory`에서 다룹니다, 64비트 환경 기준) 다음과 같습니다.

| 값 | 크기 | 들어 있는 것 |
| --- | --- | --- |
| `0..10` | 8바이트 | `i32` 두 개 |
| `(0..10).map(\|n\| n * 2)` | 8바이트 | `Range` + 아무것도 캡처하지 않는 클로저(0바이트) |
| `(0..10).map(move \|n\| n * limit)` | 12바이트 | `Range` + 복사한 `limit: i32` |
| `(0..10).map(\|n\| n * limit)` | 16바이트 | `Range` + `&limit` 참조 |

어댑터 체인은 힙 할당도, 중간 결과를 담는 벡터도 없는 평범한 구조체입니다. 게다가 `Map<I, F>`는 클로저 타입 `F`를 제네릭 매개변수로 받으므로 단형성화(Part 0 ch10)로 체인마다 전용 코드가 만들어지고, 최적화가 켜진 빌드에서는 클로저 호출까지 펼쳐져 손으로 쓴 반복문과 같은 수준으로 최적화될 수 있습니다. [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.15절의 "비용 없는 추상화"가 가능한 이유가 이 구조입니다.

**그래서 `std::iter` 문서에 구조체가 많습니다**: 모듈 문서는 "구조체는 대개 이 모듈 트레이트의 여러 메서드가 돌려주는 타입이며, 구조체 자체보다 그것을 만드는 메서드의 문서를 보라"고 안내합니다. `Map`, `Filter`, `Chain`, `Peekable` 페이지에서 읽을 것은 주로 Trait Implementations이고, 이것은 2.15절에서 씁니다. 체인의 타입에는 이름을 쓸 수 없는 클로저 타입이 들어 있으므로 함수가 체인을 돌려줄 때는 `impl Iterator<Item = …>`로 적습니다(2.18절).

### 2.5 바꾸기·거르기

| 어댑터 | 클로저 바운드 | 하는 일 | 원소 수 |
| --- | --- | --- | --- |
| `map(f)` | `FnMut(Item) -> B` | 원소마다 바꿈 | 그대로 |
| `filter(p)` | `FnMut(&Item) -> bool` | `true`인 원소만 남김 | 줄어듦 |
| `filter_map(f)` | `FnMut(Item) -> Option<B>` | `Some(v)`만 `v`로 남김 | 줄어듦 |
| `flat_map(f)` | `FnMut(Item) -> U`, `U: IntoIterator` | 원소 하나를 여러 개로 펼쳐 이어 붙임 | 늘거나 줄어듦 |
| `flatten()` | 없음 (`Item: IntoIterator`) | 원소의 한 겹을 벗김 | 늘거나 줄어듦 |
| `inspect(f)` | `FnMut(&Item)` | 원소를 들여다보기만 함(디버깅) | 그대로 |
| `copied()` / `cloned()` | 없음 (`Item = &T`, `T: Copy` / `T: Clone`) | `&T`를 `T`로 | 그대로 |

- `filter`와 `inspect`의 클로저는 원소를 **빌려서** 봅니다(`&Item`). 원소 자체를 넘겨주면 클로저가 가져가 버려 통과시킬 수 없기 때문입니다. `filter_map`과 `map`의 클로저는 원소를 값으로 받습니다.
- **`flatten`과 `Option`·`Result`(ch2)**: `Option`과 `Result`도 `IntoIterator`(`Some`/`Ok`면 값 하나, `None`/`Err`면 0개)라서 `flatten`이 `None`과 `Err`를 건너뜁니다. `[Some('가'), None, Some('나')]`를 `flatten`해 `String`으로 모으면 `"가나"`입니다. 실패를 조용히 버리는 방법이므로, 실패를 알아야 하면 `Result<Vec<_>, _>`로 `collect`합니다.
- `map(f).flatten()`은 `flat_map(f)`와 같습니다. clippy는 앞의 모양을 보면 뒤의 것으로 바꾸라고 제안합니다(`map_flatten`). 문장들을 단어로 펼치는 `lines.iter().flat_map(|l| l.split_whitespace())`가 전형적인 쓰임입니다.
- `copied`와 `cloned`는 `Item`의 타입을 바꿉니다. `prices.iter().max()`는 `Option<&i32>`, `prices.iter().copied().max()`는 `Option<i32>`입니다. `cloned()`를 `filter` 뒤에 두면 걸러지고 남은 원소만 복제합니다. 슬라이스 전체를 복사해 `Vec`을 만드는 `v.iter().copied().collect()`는 clippy가 `v.to_vec()`으로 쓰라고 제안합니다(`iter_cloned_collect`).

예제는 [`03_transform_filter.rs`](examples/03_transform_filter.rs)입니다.

### 2.6 자르기·건너뛰기

원소를 바꾸지 않고 "어디까지" 내놓을지 정하는 어댑터입니다.

| 어댑터 | 하는 일 | 예 (`temps = [18, 21, 24, 31, 33, 22, 19]`) |
| --- | --- | --- |
| `take(n)` / `skip(n)` | 앞의 `n`개만 / 앞의 `n`개를 건너뜀 | |
| `step_by(k)` | 첫 원소부터 `k`칸씩 | `["월", …, "일"].step_by(2)` → 월, 수, 금, 일 |
| `take_while(p)` | `p`가 `true`인 **동안만**. 처음 `false`에서 끝 | `take_while(< 25)` → 18, 21, 24 |
| `skip_while(p)` | `p`가 `true`인 동안 건너뛰고, 그 뒤는 검사 없이 모두 | `skip_while(< 25)` → 31, 33, 22, 19 |
| `map_while(f)` | `f`가 `Some(v)`인 동안 `v`. 처음 `None`에서 끝 | |

`filter(< 25)`는 18, 21, 24, 22, 19로 **조건에 맞는 것을 모두** 남기지만, `take_while(< 25)`는 31에서 멈추므로 뒤의 22, 19는 보지도 않습니다. "조건에 맞는 것만"과 "조건에 맞는 동안만"은 다른 질문입니다. 같은 관계가 `filter_map`과 `map_while` 사이에도 있습니다. `["3", "14", "15", "끝", "92"]`에서 `filter_map(parse)`은 92까지 모으지만 `map_while(parse)`는 `"끝"`에서 멈춥니다. 정렬된 데이터, 머리글과 본문처럼 **순서에 의미가 있는** 데이터에서 `_while` 어댑터를 씁니다.

### 2.7 합치기·순서 바꾸기

| 어댑터 | 하는 일 | 주의 |
| --- | --- | --- |
| `chain(other)` | 뒤에 이어 붙임 | `other`는 `IntoIterator`, 원소 타입이 **같아야** 함 |
| `zip(other)` | 하나씩 꺼내 `(왼쪽, 오른쪽)` 튜플 | 짧은 쪽이 끝나면 끝남 |
| `enumerate()` | `(0부터 센 순번, 원소)` | 1부터 세려면 `(1..).zip(…)` |
| `rev()` | 뒤에서부터 | 뒤에서 꺼낼 수 있는 반복자만(2.15절) |
| `cycle()` | 끝나면 처음부터 다시, 끝없이 | `take`나 `zip`으로 길이를 정해야 함. 반복자가 `Clone`이어야 함 |

- `chain`의 시그니처는 `fn chain<U>(self, other: U) -> Chain<Self, U::IntoIter> where U: IntoIterator<Item = Self::Item>`입니다. `morning.iter()`(원소 `&&str`) 뒤에 배열(원소 `&str`)을 붙이려면 먼저 `.copied()`로 원소 타입을 맞춥니다. 맞지 않으면 E0271입니다(4.3절). `Option`도 `IntoIterator`라서 `.chain(urgent)`처럼 "있으면 하나 덧붙이기"가 됩니다.
- `zip`은 `(1..)`처럼 끝이 없는 반복자와 짝지어도 다른 쪽이 끝나면 끝납니다. 같은 반복자를 한 칸 밀어 `temps.iter().zip(temps.iter().skip(1))`로 짝지으면 이웃한 원소의 차이를 구할 수 있습니다.
- `cycle`은 당번 배정처럼 "돌아가며" 짝지을 때 씁니다. `["가람", "나래"].iter().cycle().zip(tasks)`는 할 일이 다섯 개여도 두 사람에게 번갈아 배정합니다.
- `zip`의 반대는 소비 메서드 `unzip`입니다(2.3절).

예제는 [`04_take_combine.rs`](examples/04_take_combine.rs)입니다.

### 2.8 상태를 들고 가기: `scan`, `peekable`, `by_ref`

`map`의 클로저는 원소 하나만 보고 결정합니다. "지금까지의 합", "다음 원소가 무엇인가", "앞부분만 쓰고 나머지는 나중에"처럼 앞뒤 맥락이 필요하면 다음 도구를 씁니다.

**`scan(초기 상태, |&mut 상태, 원소| -> Option<내놓을 값>)`**: 상태를 `Scan` 구조체 안에 두고 클로저가 `&mut`로 고칩니다. `fold`(2.10절)가 마지막 결과 하나만 돌려준다면 `scan`은 **중간 결과를 하나씩** 내놓는 어댑터입니다. 거래 내역에서 잔액의 변화를 `[5000, 3800, 800, 2800, -1700, -700]`처럼 얻습니다. 클로저가 `None`을 돌려주면 거기서 끝나므로 "잔액이 마이너스가 되기 직전까지"도 표현할 수 있습니다.

**`peekable()`**: `Peekable` 구조체는 다음 원소를 하나 미리 꺼내 보관할 칸을 가집니다.

| 메서드 | 하는 일 |
| --- | --- |
| `peek()` | 다음 원소를 꺼내지 않고 참조로 봄 (`Option<&Item>`) |
| `peek_mut()` (1.53) | 다음 원소를 가변 참조로 봄 |
| `next_if(조건)` (1.51) | 다음 원소가 조건을 만족할 때만 꺼냄 |
| `next_if_eq(&값)` (1.51) | 다음 원소가 값과 같을 때만 꺼냄 |

`peek`의 시그니처는 `fn peek(&mut self) -> Option<&I::Item>`입니다. 보기만 하는데 `&mut self`인 이유는, 안쪽 반복자에서 원소를 실제로 꺼내 보관 칸에 넣어야 하기 때문입니다. 그래서 `peek`로 얻은 참조를 쥔 채 `next`를 부를 수 없습니다(4.4절). 예제 05는 `peekable`로 세 가지를 합니다. `peek().is_some()`으로 "방금 꺼낸 것이 마지막인가"를 알아 구분자를 넣고, `next_if(|c| c.is_ascii_digit())`로 여러 자리 수를 토큰 하나로 묶고(`"12+345 * 6"` → `[수 12] [연산자 +] [수 345] …`), `next_if_eq(&c)`로 연속된 같은 글자를 셉니다(`"aaabccdddd"` → `a3 b1 c2 d4`).

**`by_ref()`**: `take_while`처럼 `self`를 받는 어댑터를 그냥 부르면 반복자가 이동해서 나머지를 이어서 꺼낼 수 없습니다(4.6절). `by_ref()`는 `&mut self`를 돌려주고, 표준 라이브러리에 `impl<I: Iterator + ?Sized> Iterator for &mut I`가 있어서 그 위에 어댑터를 붙일 수 있습니다. 어댑터는 빌린 반복자를 쓰다가 버려지고, 원래 반복자는 멈춘 자리에서 이어집니다.

```rust
let mut lines = post.lines();
let headers: Vec<&str> = lines.by_ref().take_while(|l| !l.is_empty()).collect();
let body: Vec<&str> = lines.collect(); // 빈 줄 다음부터
```

**`take_while`은 멈춘 원소를 잃습니다.** 조건이 `false`인지 확인하려면 그 원소를 꺼내야 하고, 꺼낸 원소는 돌려놓을 곳이 없어 버려집니다. 위의 빈 줄은 버려져도 괜찮았지만, `[1, 2, 3, 10, 4, 5]`를 `by_ref().take_while(|&n| n < 5)`로 나누면 나머지가 `[4, 5]`가 되어 10이 사라집니다. 원소를 잃으면 안 될 때는 `peekable`의 `next_if`를 `while let`으로 되풀이합니다. `next_if`는 조건이 `false`면 꺼내지 않으므로 나머지가 `[10, 4, 5]`입니다. 예제는 [`05_stateful.rs`](examples/05_stateful.rs)입니다.

### 2.9 어댑터 선택표

| 하고 싶은 일 | 어댑터 | 절 |
| --- | --- | --- |
| 원소마다 바꾼다 | `map` | 2.5 |
| 조건에 맞는 것만 남긴다 | `filter` | 2.5 |
| 바꾸면서 실패(`None`)는 버린다 | `filter_map` | 2.5 |
| 원소 하나를 여러 개로 펼친다 / 한 겹 벗긴다 | `flat_map` / `flatten` | 2.5 |
| 참조를 값으로 | `copied`, `cloned` | 2.5 |
| 중간을 들여다본다(디버깅) | `inspect` | 2.5 |
| 앞에서 몇 개만 / 건너뛰고 / 띄엄띄엄 | `take` / `skip` / `step_by` | 2.6 |
| 조건이 맞는 동안만 / 맞는 동안 건너뛰고 | `take_while` / `skip_while` | 2.6 |
| 바꾸다가 실패하면 멈춘다 | `map_while` | 2.6 |
| 이어 붙인다 / 짝짓는다 / 순번을 붙인다 | `chain` / `zip` / `enumerate` | 2.7 |
| 뒤집는다 / 끝없이 되풀이한다 | `rev` / `cycle` | 2.7 |
| 누적 상태의 중간값을 내놓는다 | `scan` | 2.8 |
| 다음 원소를 보고 결정한다 | `peekable` + `peek`, `next_if` | 2.8 |
| 앞부분만 쓰고 나머지를 이어서 쓴다 | `by_ref` | 2.8 |

문서의 `Iterator` 페이지에는 이 밖에도 `fuse`(한 번 `None`을 돌려준 뒤로는 계속 `None`), `intersperse`(원소 사이에 구분자 끼우기), `array_chunks`, `map_windows` 등이 있습니다. 이 가운데 `intersperse`, `array_chunks`, `map_windows`는 1.100.0에서 `Experimental`(nightly 전용) 표시가 붙어 있어 안정판에서는 쓸 수 없습니다.

### 2.10 소비 메서드: 접기

**`fold(초기값, |누적값, 원소| 새 누적값)`**: 원소를 하나씩 누적값에 접어 넣습니다. 누적값은 어떤 타입이든 되므로, 튜플을 쓰면 개수와 합계를 한 번의 순회로 함께 구하고(`fold((0, 0), |(n, sum), p| (n + 1, sum + p))`), `String`을 쓰면 원소를 이어 붙입니다. 단순한 덧셈을 `fold(0, |a, b| a + b)`로 쓰면 clippy가 `sum()`을 쓰라고 제안합니다(`unnecessary_fold`).

**`reduce(|a, b| …)`**: 첫 원소를 초기값으로 씁니다. 그래서 반복자가 비어 있으면 `None`을 돌려줍니다. "가장 긴 단어"처럼 원소끼리 견주어 하나를 고를 때 씁니다.

**`sum`, `product`**: 시그니처가 `fn sum<S>(self) -> S where S: Sum<Self::Item>`이라서 결과 타입을 알려 줘야 합니다(4.6절). 빈 반복자의 `sum`은 0, `product`는 1입니다. `Option`과 `Result`도 `Sum`·`Product`를 구현하므로 ch2처럼 `Option<u32>`, `Result<i32, _>`로 합할 수 있습니다. 그 밖에 `count()`(원소 수), `last()`(마지막 원소), `max()`·`min()`(원소가 `Ord`여야 함)이 있습니다.

### 2.11 소비 메서드: 찾기와 묻기, 단락 평가

| 메서드 | 돌려주는 것 | 멈추는 때 |
| --- | --- | --- |
| `find(p)` | 조건을 처음 만족하는 원소 `Option<Item>` | 찾으면 |
| `find_map(f)` | `f`가 처음 돌려준 `Some(v)`의 `v` | 찾으면 |
| `position(p)` | 처음 만족하는 원소의 순번 `Option<usize>` | 찾으면 |
| `any(p)` | 하나라도 만족하는가 | 처음 `true`에서 |
| `all(p)` | 모두 만족하는가 | 처음 `false`에서 |
| `max_by_key(f)` / `min_by_key(f)` | 키가 가장 큰/작은 원소 | 끝까지 봐야 함 |
| `max_by(cmp)` / `min_by(cmp)` | 비교 클로저로 고른 원소 | 끝까지 봐야 함 |

**단락 평가**: `find`, `any`, `all`, `position`은 답이 정해지는 순간 더 꺼내지 않습니다. 예제 06은 `inspect`로 몇 개를 검사했는지 셉니다. `[72, 95, 58, 88, 64]`에서 `find(>= 90)`은 2개, `any(< 60)`과 `all(>= 60)`은 3개만 보고 끝납니다. 이 메서드들의 시그니처는 `fn find<P>(&mut self, predicate: P)`처럼 **`&mut self`** 입니다. 반복자를 빌려서 필요한 만큼만 꺼내므로, 끝난 뒤에 남은 원소를 이어서 꺼낼 수 있습니다. 반면 `fold`, `count`, `sum`, `max`는 어차피 끝까지 꺼내므로 `self`를 받습니다.

- 빈 반복자의 `all`은 `true`, `any`는 `false`입니다. "조건을 어기는 원소가 하나도 없다"는 뜻이기 때문입니다.
- `filter(p).next()`는 `find(p)`와 같고 clippy도 `find`를 쓰라고 제안합니다(`filter_next`). `find_map(f)`는 `filter_map(f).next()`와 같습니다.
- **같은 값이 여럿일 때**: 문서는 `max_by_key`가 같은 최댓값 중 **마지막** 원소를, `min_by_key`가 같은 최솟값 중 **첫** 원소를 돌려준다고 정해 둡니다. 예제 06에서 95점이 둘이면 `max_by_key`는 뒤의 "다온"을 고릅니다.
- `f64`는 `Ord`가 아니라서 `max()`와 `max_by_key`에 쓸 수 없습니다. `max_by(|a, b| a.total_cmp(b))`를 씁니다([ch4](../ch4-collections/README.md) 2.8절의 정렬과 같은 이유).

### 2.12 실패할 수 있는 접기: `try_fold`, `try_for_each`

`try_fold(초기값, |누적값, 원소| -> Option 또는 Result)`는 클로저가 `None`이나 `Err`를 돌려주는 순간 멈추고 그것을 돌려줍니다. 끝까지 성공하면 `Some(누적값)`이나 `Ok(누적값)`입니다.

```rust
let fits: Option<u8>     = [100u8, 100, 50].iter().try_fold(0u8, |acc, &x| acc.checked_add(x)); // Some(250)
let overflow: Option<u8> = [100u8, 100, 60].iter().try_fold(0u8, |acc, &x| acc.checked_add(x)); // None
```

`try_for_each(|원소| -> Result<(), E>)`는 원소마다 일을 하다 처음 실패하면 멈춥니다. 둘 다 `&mut self`라서 멈춘 뒤 남은 원소를 꺼낼 수 있습니다. 예제 06에서 회원 등록이 빈 이름에서 멈춘 뒤 `collect`하면 처리하지 않은 `["다온"]`이 남아 있습니다.

시그니처의 바운드는 `R: Try<Output = B>`입니다. ch2 2.10절에서 본 것처럼 `Try`는 불안정 트레이트라서 직접 구현할 수 없고, 실제로는 `Option`과 `Result`를 돌려준다고 읽으면 됩니다. 반복문 안의 `?`를 반복자로 옮길 때 쓰는 도구입니다.

### 2.13 반복자끼리 비교하기

반복자끼리는 원소를 앞에서부터 하나씩 비교합니다(사전식). 앞부분이 모두 같으면 짧은 쪽이 작습니다.

| 메서드 | 돌려주는 것 | 원소에 필요한 것 |
| --- | --- | --- |
| `eq`, `ne` | `bool` | `PartialEq` |
| `lt`, `le`, `gt`, `ge` | `bool` | `PartialOrd` |
| `cmp` / `partial_cmp` | `Ordering` / `Option<Ordering>` | `Ord` / `PartialOrd` |
| `is_sorted`, `is_sorted_by`, `is_sorted_by_key` (1.82) | `bool` | `PartialOrd` |

인자가 `IntoIterator`라서 종류가 다른 반복자끼리도 원소 타입만 맞으면 비교됩니다(`(1..=3).eq([1, 2, 3])`). 새 값을 만들지 않고 비교할 수 있다는 것이 쓸모입니다. 예제 06은 `"Rust".chars().map(소문자로).eq("rUST".chars().map(소문자로))`로 문자열을 할당하지 않고 대소문자를 무시해 비교하고, 버전 문자열을 `.`로 나눠 숫자 반복자로 비교합니다. 문자열로 비교하면 `"1.10.0" < "1.9.1"`(Less)이지만 숫자 반복자로 비교하면 Greater입니다. `cmp_by`, `eq_by`처럼 비교 클로저를 받는 판은 아직 `Experimental`입니다.

### 2.14 소비 메서드 선택표

| 하고 싶은 일 | 메서드 | `self` | 일찍 멈추나 | 절 |
| --- | --- | --- | --- | --- |
| 컬렉션으로 모은다 / 있는 것에 더한다 | `collect` / `extend` | 값 | `Result`/`Option`으로 모을 때 | 2.3 |
| 두 무리로 나눈다 | `partition`, `unzip` | 값 | 아니요 | 2.3 |
| 합·곱·개수 | `sum`, `product`, `count` | 값 | 아니요 | 2.10 |
| 누적값 하나로 접는다 | `fold`, `reduce` | 값 | 아니요 | 2.10 |
| 실패하면 멈추며 접는다 / 처리한다 | `try_fold` / `try_for_each` | `&mut` | 예 | 2.12 |
| 원소마다 부수 효과만 | `for` 문, `for_each` | 값 | 아니요 | |
| 처음 만족하는 원소 / 그 순번 | `find`, `find_map` / `position` | `&mut` | 예 | 2.11 |
| 하나라도 / 모두 | `any` / `all` | `&mut` | 예 | 2.11 |
| 가장 큰·작은 것 | `max`, `min`, `max_by_key`, `min_by`, … | 값 | 아니요 | 2.11 |
| 마지막 원소 / n번째 원소 | `last` / `nth` | 값 / `&mut` | `nth`는 예 | |
| 다른 반복자와 비교 | `eq`, `cmp`, `lt`, … | 값 | 차이가 나면 | 2.13 |
| 뒤에서 찾기 | `rfind`, `rposition` | `&mut` | 예 | 2.15 |

모듈 문서는 끝이 없는 반복자에 `min`처럼 전체를 봐야 하는 메서드를 부르면 끝나지 않을 수 있다고 경고합니다. `(1..)`, `repeat`, `cycle`에는 `take`나 `take_while`로 끝을 만든 뒤 소비합니다. 예제는 [`06_consumers.rs`](examples/06_consumers.rs)입니다.

### 2.15 `DoubleEndedIterator`, `ExactSizeIterator`, `size_hint`

```rust
pub trait DoubleEndedIterator: Iterator {
    fn next_back(&mut self) -> Option<Self::Item>;   // 뒤에서 하나 꺼냄
    // 기본 구현: nth_back, rfold, rfind, try_rfold
}

pub trait ExactSizeIterator: Iterator {
    fn len(&self) -> usize { ... }                   // 남은 원소 수 (기본 구현이 있음)
}
```

두 트레이트 모두 `Iterator`를 슈퍼트레이트로 둡니다. **`DoubleEndedIterator`** 는 뒤에서도 꺼낼 수 있는 반복자입니다. 앞의 `next`와 뒤의 `next_back`은 같은 원소들을 양쪽에서 줄여 가고, 둘이 만나면 끝납니다(예제 07: 앞, 뒤, 앞, 뒤로 꺼내면 1, 5, 2, 4이고 3이 남음). `rev()`, `rfind`, `rposition`이 이 트레이트를 요구합니다. `word.chars().eq(word.chars().rev())`는 문자열을 뒤집어 저장하지 않고 회문인지 검사합니다.

**`ExactSizeIterator`** 는 남은 원소 수를 정확히 아는 반복자입니다. `len()`은 처음 길이가 아니라 **남은** 개수라서, 하나 꺼낸 슬라이스 반복자의 `len()`은 4입니다. `filter` 뒤에는 몇 개가 남을지 모르므로 `len()`이 없고, `filter(..).len()`은 E0599입니다. 개수가 필요하면 `count()`로 셉니다.

**`size_hint()`** 는 모든 반복자가 가진 메서드로, 남은 원소 수의 `(최소, Some(최대) 또는 모르면 None)`을 돌려줍니다. 기본 구현은 `(0, None)`("모름")입니다.

| 반복자 | `size_hint()` |
| --- | --- |
| `v.iter()` (원소 5개) | `(5, Some(5))` |
| `v.iter().filter(…)` | `(0, Some(5))` |
| `v.iter().chain(&v)` | `(10, Some(10))` |
| `(1..)` | `(18446744073709551615, None)` |
| `(1..).take(3)` | `(3, Some(3))` |
| `"가나다abc".chars()` (12바이트) | `(3, Some(12))` |

`chars()`는 글자 수를 모르지만 한 글자가 1~4바이트이므로 3~12개라는 것은 압니다. 문서는 `size_hint`가 주로 원소를 담을 공간을 미리 확보하는 최적화에 쓰이며, `unsafe` 코드에서 믿어서는 안 된다고 적습니다. `ExactSizeIterator`를 구현할 때는 `size_hint`가 정확한 값을 돌려줘야 하고, 그러면 `len()`이 기본 구현으로 따라옵니다(2.19절).

**어댑터를 거친 뒤에도 성질이 남는가**: 각 어댑터 구조체 문서의 Trait Implementations를 보면 조건이 적혀 있습니다. 1.100.0 문서에서 옮긴 것입니다.

| 어댑터 | `DoubleEndedIterator`가 되는 조건 | `ExactSizeIterator`가 되는 조건 |
| --- | --- | --- |
| `Map`, `Inspect`, `Copied`, `Cloned`, `Peekable`, `Fuse` | 안쪽이 DEI | 안쪽이 ESI |
| `Filter`, `FilterMap` | 안쪽이 DEI | 없음 |
| `Enumerate` | 안쪽이 **ESI + DEI** | 안쪽이 ESI |
| `Skip`, `Take`, `StepBy` | 안쪽이 **ESI + DEI** | 안쪽이 ESI |
| `Zip` | 양쪽 모두 ESI + DEI | 양쪽 모두 ESI |
| `Chain` | 양쪽 모두 DEI | 없음 |
| `FlatMap`, `Flatten` | 바깥과 안쪽 반복자 모두 DEI | 없음 |
| `Rev` | 안쪽이 DEI (`rev` 자체가 DEI를 요구) | 안쪽이 ESI + DEI |
| `Scan`, `TakeWhile`, `SkipWhile`, `MapWhile`, `Cycle` | 구현 없음 | 구현 없음 |

(DEI는 `DoubleEndedIterator`, ESI는 `ExactSizeIterator`의 줄임입니다.) 문서의 `Enumerate` 페이지에는 `impl<I> DoubleEndedIterator for Enumerate<I> where I: ExactSizeIterator + DoubleEndedIterator`라고 적혀 있습니다. 뒤에서 꺼낸 원소에 붙일 순번(맨 뒤 원소는 몇 번인가)을 알려면 남은 개수를 알아야 하기 때문입니다. 그래서 슬라이스 반복자의 `enumerate().rev()`는 되지만, 길이를 모르는 `chars()`의 `enumerate().rev()`는 E0277입니다(4.5절). `Chain`에 `ExactSizeIterator`가 없는 이유도 문서에 있습니다. 반복자를 길게 만드는 어댑터는 두 길이의 합이 `usize`를 넘을 수 있어서 정확한 길이를 약속할 수 없습니다. 덧붙여 `enumerate().rev()`는 원래 순번을 유지하고(`(3, 'd'), (2, 'c'), …`), `rev().enumerate()`는 뒤집은 뒤 0부터 셉니다(`(0, 'd'), (1, 'c'), …`).

### 2.16 반복자 구조체 문서 읽기: `Drain<'_, T>` 등

컬렉션 메서드는 저마다 반복자 구조체를 돌려줍니다. [ch4](../ch4-collections/README.md)에서 쓴 `drain`, `range`, 집합 연산의 반환 타입 페이지를 이렇게 읽습니다. `Vec::drain`의 시그니처 `pub fn drain<R>(&mut self, range: R) -> Drain<'_, T, A>` 옆의 ⓘ나 `Drain` 링크를 따라가면 [`std::vec::Drain`](https://doc.rust-lang.org/std/vec/struct.Drain.html) 페이지가 나옵니다.

- **정의 줄** `pub struct Drain<'a, T, A = Global>`: 라이프타임 `'a`는 이 반복자가 `Vec`을 빌리고 있다는 뜻입니다. `drain` 문서도 "돌려준 반복자는 벡터의 가변 대여를 쥐고 있다"고 적습니다. `Drain`이 살아 있는 동안 원래 `Vec`을 쓸 수 없습니다.
- **Trait Implementations**: `Iterator`, `DoubleEndedIterator`, `ExactSizeIterator`, `FusedIterator`, `Drop`이 있습니다. 그러니 `drain(..).len()`과 `drain(..).rev()`가 됩니다. `Drop` 구현이 있는 이유는 `drain` 문서의 "반복자를 끝까지 소비하기 전에 버리면 남은 원소도 버린다"는 동작을 위해서입니다. `FusedIterator`는 한 번 `None`을 돌려준 뒤로는 계속 `None`을 돌려준다는 약속입니다.

예제 07은 `type_name`과 함께 이것을 확인합니다. 자주 만나는 반복자 구조체의 성질을 문서에서 정리하면 다음과 같습니다.

| 메서드 | 반복자 구조체 | DEI | ESI |
| --- | --- | --- | --- |
| `slice.iter()`, `vec.iter()` | `slice::Iter<'a, T>` | 예 | 예 |
| `vec.drain(범위)` | `vec::Drain<'a, T>` | 예 | 예 |
| `vec.extract_if(범위, 조건)` | `vec::ExtractIf<'a, T, F>` | 아니요 | 아니요 |
| `btree_map.range(범위)` | `btree_map::Range<'a, K, V>` | 예 | 아니요 |
| `btree_set.intersection(&b)` 등 집합 연산 | `btree_set::Intersection<'a, T>` 등 | 아니요 | 아니요 |
| `hash_map.iter()` | `hash_map::Iter<'a, K, V>` | 아니요 | 예 |
| `s.chars()` | `str::Chars<'a>` | 예 | 아니요 |
| `s.split(패턴)` | `str::Split<'a, P>` | 패턴이 `char` 등일 때 예, `&str`이면 아니요 | 아니요 |

ch4에서 `range(..=x).next_back()`으로 "x 이하에서 가장 큰 키"를 찾을 수 있었던 것은 `btree_map::Range`가 `DoubleEndedIterator`이기 때문입니다. `HashMap`의 반복자는 순서가 없으니 "뒤"도 없어서 `rev()`가 E0277입니다. `BTreeSet`의 교집합을 내림차순으로 보려면 `Vec`에 모은 뒤 `reverse()`합니다. `split("--").rev()`가 안 되는 이유는 에러 메시지에 나오는 불안정 트레이트 `DoubleEndedSearcher` 때문인데, 지금은 "`&str` 패턴으로 나눈 결과는 뒤에서 꺼낼 수 없다"고만 알아 두면 됩니다. 예제는 [`07_double_ended.rs`](examples/07_double_ended.rs)입니다.

### 2.17 반복자 만들기 ①: `std::iter`의 함수

반복자를 새로 만드는 방법은 세 가지입니다. 아래로 갈수록 손이 많이 가므로, 앞의 방법으로 되면 앞의 방법을 씁니다.

| 함수 (안정화 버전) | 만드는 반복자 | 예제 08의 쓰임 |
| --- | --- | --- |
| `repeat(x)` (1.0) | `x`를 끝없이 복제 | `names.zip(repeat(0))`로 점수판 초기화 |
| `repeat_n(x, n)` (1.82) | `x`를 `n`번 | 테두리 `+--------+` |
| `repeat_with(f)` (1.28) | `f()`를 끝없이 | |
| `once(x)` (1.2), `once_with(f)` (1.43) | 원소 하나 | `once('+').chain(…)` |
| `empty()` (1.2) | 아무것도 내놓지 않음 | 타입만 맞춘 빈 반복자 |
| `from_fn(f)` (1.34) | `f`가 곧 `next`. 상태는 클로저가 캡처 | 피보나치 수열 |
| `successors(첫 값, f)` (1.34) | 이전 값으로 다음 값을 계산. `None`이면 끝 | 콜라츠 수열, `checked_mul`로 넘치기 직전까지 |
| `zip(a, b)` (1.59), `chain(a, b)` (1.91) | 메서드와 같지만 양쪽 모두 `IntoIterator`를 받음 | `iter::zip(&names, &scores)` |

`from_fn`은 "다음 값을 어떻게 만드는가"만 클로저로 쓰면 되므로, 구조체를 따로 만들 만큼은 아닌 반복자에 알맞습니다. `successors`는 상태가 직전 값 하나뿐일 때 더 짧게 씁니다. `iter::successors(Some(1u8), |n| n.checked_mul(2))`는 `u8`이 넘치기 직전인 128에서 끝납니다.

### 2.18 반복자 만들기 ②: `impl Iterator`를 돌려주는 함수

기존 반복자에 어댑터를 붙여 돌려주는 방법입니다. 체인의 타입에는 클로저 타입이 들어 있어 이름을 쓸 수 없으므로 `impl Iterator<Item = …>`로 적습니다([Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.13절).

```rust
fn multiples(n: u32) -> impl Iterator<Item = u32> {
    (1..).map(move |i| i * n)
}

fn long_words(text: &str, min: usize) -> impl Iterator<Item = &str> {
    text.split_whitespace().filter(move |w| w.chars().count() >= min)
}
```

- 돌려준 반복자는 함수가 끝난 뒤에 쓰이므로, 클로저가 매개변수(`n`, `min`)를 빌리면 안 됩니다. `move`로 복사해 넣습니다.
- `long_words`는 입력 `text`를 빌린 반복자를 돌려줍니다. 2024 에디션에서는 반환 위치의 `impl Trait`가 입력의 라이프타임을 자동으로 붙잡으므로 `+ '_` 같은 표기가 필요 없습니다. 에디션별 차이는 Part 2 `ch4-rust-2024-language`에서 다룹니다.
- **`impl Iterator`는 `Iterator`라는 것만 약속합니다.** 실제 타입 `Filter<SplitWhitespace, _>`는 뒤에서도 꺼낼 수 있지만, 호출하는 쪽에서 `long_words(text, 4).rev()`를 쓰면 E0277(``the trait bound `impl Iterator<Item = &str>: DoubleEndedIterator` is not satisfied``)입니다. 더 약속하려면 `impl DoubleEndedIterator<Item = &str>`처럼 그 트레이트를 반환 타입에 적거나, 이름을 쓸 수 있는 타입이면 구체 타입을 그대로 돌려줍니다. 예제 08의 `Playlist::iter`가 `std::slice::Iter<'_, Song>`을 돌려주는 이유입니다.

### 2.19 반복자 만들기 ③: 직접 만든 컬렉션에 트레이트 구현

`Vec<Song>`을 감싼 `Playlist`를 `for`와 `collect`에 쓸 수 있게 만들어 봅니다. 구현하지 않고 `for song in &playlist`를 쓰면 ``E0277: `&Playlist` is not an iterator``입니다(4.2절).

**`IntoIterator` 세 가지**: 직접 반복자를 만들 필요 없이 안쪽 `Vec`의 반복자에 맡깁니다.

```rust
impl IntoIterator for Playlist {                     // for s in playlist → Song
    type Item = Song;
    type IntoIter = std::vec::IntoIter<Song>;
    fn into_iter(self) -> Self::IntoIter { self.songs.into_iter() }
}
impl<'a> IntoIterator for &'a Playlist {             // for s in &playlist → &Song
    type Item = &'a Song;
    type IntoIter = std::slice::Iter<'a, Song>;
    fn into_iter(self) -> Self::IntoIter { self.songs.iter() }
}
impl<'a> IntoIterator for &'a mut Playlist {         // for s in &mut playlist → &mut Song
    type Item = &'a mut Song;
    type IntoIter = std::slice::IterMut<'a, Song>;
    fn into_iter(self) -> Self::IntoIter { self.songs.iter_mut() }
}
```

`&'a Playlist`의 `'a`는 "playlist를 빌린 동안"입니다. 원소의 참조(`&'a Song`)는 그 대여보다 오래 살 수 없으므로 `Item`과 `IntoIter`에 같은 `'a`를 붙입니다. 2.2절의 관례대로 `iter()` 메서드도 함께 제공합니다. 원소를 고치면 불변식이 깨지는 컬렉션(정렬된 목록, 집합)이라면 `HashSet`처럼 `&mut` 구현과 `iter_mut()`를 일부러 빼는 것도 설계입니다.

**`FromIterator`와 `Extend`**: 받은 것을 안쪽 `Vec`에 맡기면 됩니다.

```rust
impl FromIterator<Song> for Playlist {
    fn from_iter<I: IntoIterator<Item = Song>>(iter: I) -> Self { Playlist { songs: iter.into_iter().collect() } }
}
impl Extend<Song> for Playlist {
    fn extend<I: IntoIterator<Item = Song>>(&mut self, iter: I) { self.songs.extend(iter); }
}
```

이제 `collect::<Playlist>()`와 `playlist.extend(…)`가 됩니다. 그리고 2.3절의 `Result<V, E>: FromIterator<Result<T, E>> where V: FromIterator<T>` 덕분에, `"제목, 초"` 줄들을 `parse_song`(`Result<Song, String>`을 돌려줌)으로 바꿔 `Result<Playlist, String>`으로 모으는 것도 추가 코드 없이 됩니다. 하나라도 실패하면 첫 에러가 나옵니다.

**직접 만든 반복자 구조체에 `DoubleEndedIterator`와 `ExactSizeIterator`**: 예제 07의 `Pages`는 1쪽부터 마지막 쪽까지 쪽 번호를 내놓는 반복자입니다. 아직 내놓지 않은 범위를 `front..=back` 두 필드로 들고, `next`는 `front`를, `next_back`은 `back`을 줄입니다. `size_hint`를 정확한 값으로 재정의하고 `impl ExactSizeIterator for Pages {}`(필수 메서드 없음)를 붙이면 `rev()`, `len()`, `enumerate().rev()`가 모두 따라옵니다. `Ord`, `Hash` 같은 다른 트레이트를 직접 구현하는 방법은 `ch6-core-traits`에서 다룹니다. 예제는 [`08_custom_iterators.rs`](examples/08_custom_iterators.rs)입니다.

**이 챕터에서 다루지 않는 것**: 비동기로 값을 하나씩 받는 스트림은 [Part 0 ch16](../../part0-getting-started/ch16-async/README.md)에서 맛보았고 `ch13-async-primitives`에서 `Future`와 함께 다시 봅니다. 문서의 `iter!` 매크로나 `gen` 블록처럼 함수 본문에서 값을 하나씩 내놓는(yield) 문법은 아직 불안정 기능이며 Part 11에서 다룹니다. 반복자를 여러 스레드로 나눠 처리하는 병렬 반복은 표준 라이브러리에 없고 외부 크레이트로 제공됩니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_into_iterator.rs`](examples/01_into_iterator.rs) | `for` 풀어 쓰기, `Vec`/`&Vec`/`&mut Vec`의 `IntoIterator`, 여러 타입의 `Item`, `IntoIterator`를 받는 함수 | 1 |
| [`examples/02_from_iterator.rs`](examples/02_from_iterator.rs) | 같은 반복자를 여러 타입으로 `collect`, `String`·`Result`·`Option`·튜플로 모으기, `extend`, `partition` | 1, 3 |
| [`examples/03_transform_filter.rs`](examples/03_transform_filter.rs) | 어댑터의 타입과 크기, 원소가 흐르는 순서, `map`/`filter`/`filter_map`/`flat_map`/`flatten`/`copied`/`cloned` | 2 |
| [`examples/04_take_combine.rs`](examples/04_take_combine.rs) | `take`/`skip`/`step_by`/`take_while`/`skip_while`/`map_while`, `chain`/`zip`/`enumerate`/`rev`/`cycle` | 2 |
| [`examples/05_stateful.rs`](examples/05_stateful.rs) | `scan`, `peekable`(`peek`, `next_if`, `next_if_eq`), `by_ref`와 `take_while`의 함정 | 2 |
| [`examples/06_consumers.rs`](examples/06_consumers.rs) | `fold`/`reduce`/`sum`/`product`, 단락 평가, `max_by_key`의 같은 값 처리, `try_fold`/`try_for_each`, 반복자 비교 | 3 |
| [`examples/07_double_ended.rs`](examples/07_double_ended.rs) | `next_back`/`rev`/`rfind`, 어댑터를 거친 `rev`, `len`과 `size_hint`, `Drain`·`Range`·`Intersection`, 직접 구현 | 4 |
| [`examples/08_custom_iterators.rs`](examples/08_custom_iterators.rs) | `std::iter` 함수, `impl Iterator` 반환, `Playlist`의 `IntoIterator` 세 가지와 `FromIterator`·`Extend` | 5 |
| [`compile_fail/`](compile_fail) | 반복자에서 자주 만나는 컴파일 에러 5개 (4절) | 1, 2, 4, 5 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 패닉 없이 끝나고 종료 코드는 0이며, 실행할 때마다 같은 결과가 나옵니다. 다만 `type_name` 출력(01, 03, 07)은 툴체인에 따라, `size_of_val` 값(03)은 플랫폼에 따라 달라질 수 있습니다. 순서가 정해지지 않은 `HashMap`/`HashSet`의 내용은 출력하지 않습니다. 분량을 줄이려고 출력은 핵심 구역만 옮겼고, 나머지 구역의 결과는 각 파일의 `// 출력:` 주석과 2절의 본문에 있습니다.

### 01_into_iterator — `for`의 정체

```sh
cargo run -p p1-ch5-iterators --example 01_into_iterator
```

2번과 5번 구역입니다(1번은 `for`와 손으로 쓴 반복의 결과가 같다는 것, 3번은 2.2절의 "다른 타입들", 4번은 `Rev<slice::Iter>`의 `IntoIter`가 자기 자신이라는 것).

```text
== 2. Vec의 세 가지 IntoIterator 구현 ==
for x in v
    IntoIter = alloc::vec::into_iter::IntoIter<alloc::string::String>
    Item     = alloc::string::String
for x in &v
    IntoIter = core::slice::iter::Iter<'_, alloc::string::String>
    Item     = &alloc::string::String
for x in &mut v
    IntoIter = core::slice::iter::IterMut<'_, alloc::string::String>
    Item     = &mut alloc::string::String
&mut로 고친 뒤: ["rust!", "iter!"]

== 5. IntoIterator를 받는 함수 ==
Vec:        4200
배열:       6000
범위:       10
Some/None:  800 / 0
BTreeSet:   1200
반복자:     42
copied:     4200
cart는 그대로: [1200, 3000]
```

3번 구역에서 `&BTreeMap<&str, u32>`의 `Item`은 `(&&str, &u32)`로 나옵니다. 키 타입이 이미 `&str`이라 참조가 두 겹이 됩니다. `describe` 함수의 `type_name::<C::Item>()`처럼 연관 타입을 쓰는 문법은 [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)의 연관 타입입니다.

### 02_from_iterator — `collect`와 `extend`의 정체

```sh
cargo run -p p1-ch5-iterators --example 02_from_iterator
```

3, 4, 6번 구역입니다(1번은 2.3절의 첫 표, 2번은 `String`으로 모으기, 5번은 `extend`).

```text
== 3. Result와 Option 안에 모으기 ==
Result<Vec<_>, _>:      Ok([30, 10, 20, 10])
Result<BTreeSet<_>, _>: Ok({10, 20, 30})
하나라도 실패하면:      Err(ParseIntError { kind: InvalidDigit })
Option<String>: Some("314"), None

== 4. 튜플로 모으기: collect와 unzip ==
unzip:   ["가람", "나래", "다온"] [88, 72, 95]
collect: {"가람", "나래", "다온"} "88 72 95 "

== 6. partition: 결과 타입이 Default + Extend면 무엇이든 ==
Vec 두 개:    [2, 4, 6, 8] / [1, 3, 5, 7, 9]
String 두 개: "반복자" / "Rust  101"
```

3번에서 같은 입력을 `BTreeSet`으로 모으면 중복된 10이 하나로 줄고 정렬됩니다. `Result<V, _>`의 `V`가 무엇이냐에 따라 결과가 달라지는 것입니다. 6번의 `"Rust  101"`에 스페이스가 두 개인 것은 한글 앞뒤의 스페이스가 모두 오른쪽으로 갔기 때문입니다.

### 03_transform_filter — 어댑터의 정체와 바꾸기·거르기

```sh
cargo run -p p1-ch5-iterators --example 03_transform_filter
```

1, 2, 4번 구역입니다(3번은 2.5절의 `map`/`filter`/`filter_map`, 5번은 `copied`/`cloned`).

```text
== 1. 어댑터가 돌려주는 것은 구조체 ==
타입: core::iter::adapters::map::Map<core::iter::adapters::filter::Filter<core::slice::iter::Iter<'_, i32>, 03_transform_filter::main::{{closure}}>, 03_transform_filter::main::{{closure}}>
크기: Range 8바이트, map(캡처 없음) 8, map(move) 12, map(참조 캡처) 16
filter + map 체인: 16바이트
결과: [7, 9, 8]

== 2. 원소는 체인을 하나씩 끝까지 흐릅니다 ==
  꺼냄: 72
    통과: 72
  꺼냄: 95
    통과: 95
앞의 두 개: [7, 9]

== 4. flat_map / flatten ==
flat_map(split_whitespace): ["fn", "main", "let", "x", "=", "1"]
Vec<Vec> flatten:    [1, 2, 3]
Option flatten:      "가나"
Result flatten:      [1, 3]
```

1번의 체인이 16바이트인 것은 `slice::Iter`(포인터 두 개)만 들어 있고 두 클로저는 아무것도 캡처하지 않아 0바이트이기 때문입니다. 2번에서 58과 88은 "꺼냄"조차 출력되지 않았습니다.

### 04_take_combine — 자르기·건너뛰기·합치기

```sh
cargo run -p p1-ch5-iterators --example 04_take_combine
```

2, 3, 5, 6번 구역입니다(1번은 `take`/`skip`/`step_by`, 4번은 2.7절의 `chain`).

```text
== 2. take_while / skip_while와 filter의 차이 ==
filter(< 25):     [18, 21, 24, 22, 19]
take_while(< 25): [18, 21, 24]
skip_while(< 25): [31, 33, 22, 19]

== 3. map_while: 바꾸다가 실패하면 멈추기 ==
map_while:  [3, 14, 15]
filter_map: [3, 14, 15, 92]

== 5. zip / enumerate ==
zip:       [("가람", 88), ("나래", 72), ("다온", 95)]
enumerate: ["0:가람", "1:나래", "2:다온"]
(1..).zip: ["1번 가람", "2번 나래", "3번 다온"]
이웃한 차이: [3, 3, 7, 2, -11, -3]

== 6. rev / cycle ==
rev:   [5, 4, 3, 2, 1]
cycle: ["설거지=가람", "빨래=나래", "청소=가람", "장보기=나래", "분리수거=가람"]
```

5번의 `zip`은 점수가 하나 더 많아도 이름 수에 맞춰 3쌍입니다.

### 05_stateful — 상태를 들고 가기

```sh
cargo run -p p1-ch5-iterators --example 05_stateful
```

```text
== 1. scan: 중간 상태를 하나씩 내놓기 ==
잔액 변화: [5000, 3800, 800, 2800, -1700, -700]
최종 잔액: -700
마이너스 직전까지: [5000, 3800, 800, 2800]

== 2. peekable: 꺼내지 않고 다음 원소 보기 ==
peek: Some(10)
peek: Some(10)
next: Some(10)
peek: Some(20)
마지막 판별: 가람 → 나래 → 다온

== 3. next_if / next_if_eq ==
tokenize: [수 12] [연산자 +] [수 345] [연산자 *] [수 6]
run-length: a3 b1 c2 d4

== 4. by_ref: 앞부분만 쓰고 나머지는 나중에 ==
머리글: ["제목: 반복자", "작성자: 페리스"]
본문:   ["본문 첫 줄", "본문 둘째 줄"]
take_while: [1, 2, 3], 나머지 [4, 5]  ← 10이 사라짐
next_if:    [1, 2, 3], 나머지 [10, 4, 5]
```

2번에서 `peek`를 두 번 불러도 같은 원소를 봅니다. `tokenize`는 공백을 건너뛰므로 `"345 * 6"`의 공백은 토큰이 되지 않습니다.

### 06_consumers — 소비 메서드

```sh
cargo run -p p1-ch5-iterators --example 06_consumers
```

2, 3, 4번 구역입니다(1번은 2.10절의 접기, 5번은 2.13절의 비교).

```text
== 2. 찾기와 묻기: 답이 나오면 멈춥니다(단락 평가) ==
find(>= 90) = Some(95), 검사한 원소 2개
any(< 60) = true, 검사한 원소 3개
all(>= 60) = false, 검사한 원소 3개
빈 반복자: any = false, all = true
find_map: Some("8080"), position(빈 줄): Some(1)

== 3. 최대·최소 고르기 ==
max_by_key: Some(("다온", 95))
min_by_key: Some(("라온", 72))
max_by(total_cmp): Some(38.2)

== 4. 실패할 수 있는 접기: try_fold, try_for_each ==
u8 합계: Some(250), 넘치면: None
  등록: 가람
  등록: 나래
결과: Err("빈 이름은 등록할 수 없습니다")
처리하지 않은 원소: ["다온"]
```

2번의 `find_map`은 `"# 설정"`과 빈 줄에서 `split_once(" = ")`가 `None`이라 세 번째 줄의 값 `"8080"`을 돌려줍니다. 4번에서 `"다온"`은 등록되지 않았지만 반복자에 남아 있습니다.

### 07_double_ended — 양쪽 끝과 길이

```sh
cargo run -p p1-ch5-iterators --example 07_double_ended
```

1, 2, 4, 5번 구역입니다(3번은 2.15절의 `len`과 `size_hint` 표).

```text
== 1. next_back: 뒤에서 꺼내기 ==
앞, 뒤, 앞, 뒤: [Some(1), Some(5), Some(2), Some(4)], 남은 것: [3]
기러기: 회문인가? true
토마토: 회문인가? true
반복자: 회문인가? false
rfind(마지막 WARN): Some("WARN 재시도"), rposition: Some(3)

== 2. 어댑터를 거친 뒤의 rev ==
filter + map + rev: DCA
enumerate().rev(): [(3, 'd'), (2, 'c'), (1, 'b'), (0, 'a')]
rev().enumerate(): [(0, 'd'), (1, 'c'), (2, 'b'), (3, 'a')]
char_indices().rev(): [(3, '글'), (0, '한')]

== 4. 컬렉션이 돌려주는 반복자 구조체 ==
drain 타입: alloc::vec::drain::Drain<'_, i32>
drain 길이: 3
뒤에서부터 꺼냄: [4, 3, 2], 남은 queue: [1, 5, 6]
range 타입: alloc::collections::btree::map::Range<'_, &str, i32>
범위의 마지막 항목: Some(("11-10", 8000))
intersection 타입: alloc::collections::btree::set::Intersection<'_, i32>
교집합 내림차순: [7, 5, 3]

== 5. 직접 만든 반복자에 구현하기 ==
앞에서:   [1, 2, 3, 4, 5]
rev:      [5, 4, 3, 2, 1]
len:      5
enumerate().rev(): [(2, 3), (1, 2), (0, 1)]
```

2번의 `char_indices().rev()`가 내놓는 3과 0은 글자 순번이 아니라 바이트 위치입니다(한글 한 글자가 3바이트). 3번 구역의 `(1..).size_hint()`가 `(18446744073709551615, None)`인 것은 끝이 없으므로 최소값이 `usize::MAX`(64비트 환경의 값)이고, 최대값은 `usize`로 나타낼 수 없기 때문입니다. 문서는 `None`이 "상한을 모르거나 `usize`보다 크다"는 뜻이라고 설명합니다.

### 08_custom_iterators — 반복자 만들기

```sh
cargo run -p p1-ch5-iterators --example 08_custom_iterators
```

```text
== 1. std::iter의 함수 ==
repeat + zip:  {"가람": 0, "나래": 0}
once + repeat_n: +--------+
empty: count = 0
from_fn(피보나치): [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]
successors(콜라츠): [6, 3, 10, 5, 16, 8, 4, 2, 1]
successors(u8 2의 거듭제곱): [1, 2, 4, 8, 16, 32, 64, 128]
iter::zip: [("가람", 88), ("나래", 72)], iter::chain: ["가람", "나래", "다온"]

== 2. impl Iterator를 돌려주는 함수 ==
multiples(7): [7, 14, 21, 28, 35]
long_words:            ["어댑터는", "게으르다"]
long_words_both_ends:  ["게으르다", "어댑터는"] (rev)
세 글자 이상, 뒤에서부터: ["빠르다", "그래서", "게으르다", "어댑터는", "반복자"]

== 3. 직접 만든 컬렉션의 IntoIterator 세 가지 ==
봄날(245초) 밤편지(253초) Hype Boy(179초) → 합계 677초
&mut로 고친 뒤 합계 647초, 마지막 곡: Some("Hype Boy")
값으로 꺼낸 제목: ["봄날", "밤편지", "Hype Boy"]

== 4. FromIterator와 Extend 구현 ==
collect + extend: 3곡, 590초
Ok: [Song { title: "가", secs: 120 }, Song { title: "나", secs: 95 }]
Err: 쉼표가 없음: "나 95"
```

3번의 "마지막 곡"은 `playlist.iter().next_back()`으로 얻었습니다. `iter()`가 `slice::Iter`를 그대로 돌려주므로 호출하는 쪽에서 `next_back`을 쓸 수 있습니다. 4번의 `bad`에는 실패하는 줄이 둘(`"나 95"`, `"다, x"`) 있지만 첫 `Err`에서 멈추므로 첫 번째 에러만 나옵니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch5-iterators
rustc --edition 2024 compile_fail/e0382_for_moves_collection.rs --out-dir /tmp/rust-p1-ch5
```

에러 메시지의 `note: required by a bound in ...` 아래에 나오는 표준 라이브러리 소스 위치(`/rustc/0ef1bc38.../library/...`)는 툴체인마다 다르므로 아래 인용에서는 생략했습니다.

### 4.1 `for x in v` 뒤에 `v` 사용 — E0382

파일: [`compile_fail/e0382_for_moves_collection.rs`](compile_fail/e0382_for_moves_collection.rs)

```text
error[E0382]: borrow of moved value: `names`
  --> compile_fail/e0382_for_moves_collection.rs:21:24
   |
17 |     let names = vec![String::from("가람"), String::from("나래")];
   |         ----- move occurs because `names` has type `Vec<String>`, which does not implement the `Copy` trait
18 |     for name in names {
   |                 ----- `names` moved due to this implicit call to `.into_iter()`
...
21 |     println!("모두 {}명", names.len());
   |                           ^^^^^ value borrowed here after move
   |
note: the `for` loop is desugared into a call to `std::iter::IntoIterator::into_iter`, which takes ownership of the receiver `self`, which moves `names`
help: consider iterating over a slice of the `Vec<String>`'s content to avoid moving into the `for` loop
   |
18 |     for name in &names {
   |                 +
```

- **읽는 법**: 18행의 표시가 "`.into_iter()`의 **암묵적인** 호출 때문에 이동했다"고 말합니다. `note:`는 2.2절의 내용 그대로, `for`가 `IntoIterator::into_iter`로 풀리고 그 메서드가 `self`를 받는다고 설명합니다.
- **원인**: `Vec<String>`에 대한 `IntoIterator` 구현은 값을 받는 것이므로 `names`의 소유권이 반복자로 옮겨 갑니다.
- **해결**: `help:`대로 `for name in &names`로 바꿉니다. 그러면 `&Vec<String>`의 구현이 골라져 `name`은 `&String`이 됩니다. 원소의 소유권이 정말 필요하다면(다른 컬렉션으로 옮기는 등) 반복 뒤에 `names`를 쓰지 않도록 고칩니다.

### 4.2 `IntoIterator`가 없는 타입을 `for`에 — E0277

파일: [`compile_fail/e0277_struct_not_iterator.rs`](compile_fail/e0277_struct_not_iterator.rs)

```text
error[E0277]: `&Playlist` is not an iterator
  --> compile_fail/e0277_struct_not_iterator.rs:28:17
   |
28 |     for song in &playlist {
   |                 ^^^^^^^^^ `&Playlist` is not an iterator
   |
   = help: the trait `Iterator` is not implemented for `&Playlist`
   = note: required for `&Playlist` to implement `IntoIterator`
```

- **읽는 법**: `for`가 원하는 것은 `IntoIterator`인데 메시지는 "iterator가 아니다"라고 말합니다. `note:`가 연결 고리입니다. 컴파일러는 `&Playlist`에 대한 `IntoIterator` 구현을 찾지 못했고, 남은 유일한 길인 포괄 구현 `impl<I: Iterator> IntoIterator for I`를 쓰려면 `&Playlist`가 `Iterator`여야 하는데 그것도 아니라는 뜻입니다.
- **원인**: `Playlist` 안에 `Vec`이 있어도 컴파일러는 그것으로 "어떻게 반복할지"를 추측하지 않습니다. `String`을 `for`에 넣었을 때 같은 에러가 나는 것과 같은 이유입니다.
- **해결**: 당장은 `for song in &playlist.songs`로 안쪽 필드를 돌고, 컬렉션처럼 쓰게 하려면 2.19절처럼 `IntoIterator`를 구현합니다.

### 4.3 원소 타입이 다른 반복자를 `chain` — E0271

파일: [`compile_fail/e0271_chain_item_mismatch.rs`](compile_fail/e0271_chain_item_mismatch.rs)

```text
error[E0271]: type mismatch resolving `<Vec<{integer}> as IntoIterator>::Item == &{integer}`
  --> compile_fail/e0271_chain_item_mismatch.rs:19:46
   |
19 |     let all: Vec<i32> = morning.iter().chain(evening).copied().collect();
   |                                        ----- ^^^^^^^ expected `&{integer}`, found integer
   |                                        |
   |                                        required by a bound introduced by this call
   |
note: required by a bound in `std::iter::Iterator::chain`

error[E0599]: the method `copied` exists for struct `std::iter::Chain<std::slice::Iter<'_, {integer}>, std::vec::IntoIter<{integer}>>`, but its trait bounds were not satisfied
```

- **읽는 법**: 첫 줄은 "`Vec`을 반복자로 바꾼 원소 타입(`<Vec<{integer}> as IntoIterator>::Item`)이 `&{integer}`여야 하는데 맞지 않는다"는 뜻입니다. 그 조건은 `chain`의 바운드 `U: IntoIterator<Item = Self::Item>`에서 왔습니다. 뒤따르는 E0599는 첫 에러 때문에 `Chain`이 반복자가 되지 못해 생긴 것이므로, 첫 에러만 고치면 함께 사라집니다.
- **원인**: `morning.iter()`의 원소는 `&i32`, 값으로 넘긴 `evening`의 원소는 `i32`입니다. 참조와 값은 다른 타입입니다.
- **해결**: 원소 타입을 한쪽으로 맞춥니다. `morning.iter().chain(&evening).copied()`(둘 다 `&i32`로 맞춘 뒤 복사)나 `morning.iter().copied().chain(evening)`(둘 다 `i32`). `zip`은 두 원소 타입이 달라도 되므로 이 에러가 나지 않습니다.

### 4.4 `peek`로 얻은 참조를 쥔 채 `next` — E0499

파일: [`compile_fail/e0499_peek_then_next.rs`](compile_fail/e0499_peek_then_next.rs)

```text
error[E0499]: cannot borrow `tokens` as mutable more than once at a time
  --> compile_fail/e0499_peek_then_next.rs:18:20
   |
17 |     let next_char = tokens.peek();
   |                     ------ first mutable borrow occurs here
18 |     let consumed = tokens.next();
   |                    ^^^^^^ second mutable borrow occurs here
19 |     println!("미리 본 글자: {next_char:?}, 꺼낸 글자: {consumed:?}");
   |                              --------- first borrow later used here
```

- **읽는 법**: `peek`가 가변 대여(first mutable borrow)로 표시됩니다. 보기만 하는 메서드인데 가변 대여인 것이 핵심입니다.
- **원인**: `peek(&mut self) -> Option<&Item>`은 다음 원소를 `Peekable` 안의 칸에 꺼내 두고 그 칸을 가리키는 참조를 돌려줍니다(2.8절). 그 참조를 쥔 채 `next`를 부르면 `next`가 바로 그 원소를 칸 밖으로 옮겨 버려 참조가 가리킬 곳이 사라집니다.
- **해결**: `char`처럼 `Copy`인 원소는 `tokens.peek().copied()`로 복사해 참조를 끊습니다. 또는 `if tokens.peek() == Some(&'1') { tokens.next(); }`처럼 판단을 먼저 끝낸 뒤 꺼냅니다. "조건이 맞으면 꺼내기"가 목적이면 `next_if`, `next_if_eq`가 이 모양을 한 번에 해 줍니다.

### 4.5 길이를 모르는 반복자의 `enumerate().rev()` — E0277

파일: [`compile_fail/e0277_rev_needs_exact_size.rs`](compile_fail/e0277_rev_needs_exact_size.rs)

```text
error[E0277]: the trait bound `Chars<'_>: ExactSizeIterator` is not satisfied
  --> compile_fail/e0277_rev_needs_exact_size.rs:22:66
   |
22 |     let from_back: Vec<(usize, char)> = word.chars().enumerate().rev().collect();
   |                                                                  ^^^ the trait `ExactSizeIterator` is not implemented for `Chars<'_>`
   |
   = help: the following other types implement trait `ExactSizeIterator`: (목록 생략)
   = note: required for `Enumerate<Chars<'_>>` to implement `DoubleEndedIterator`
note: required by a bound in `rev`

error[E0599]: the method `collect` exists for struct `Rev<Enumerate<Chars<'_>>>`, but its trait bounds were not satisfied
```

- **읽는 법**: 아래에서 위로 읽습니다. `rev`는 `DoubleEndedIterator`를 요구하고(`note: required by a bound in rev`), `Enumerate<Chars>`가 그것이 되려면(`note: required for ...`) `Chars`가 `ExactSizeIterator`여야 하는데, 아니라는 것(첫 줄)입니다. 2.15절의 표에서 `Enumerate` 줄의 조건 그대로입니다.
- **원인**: `chars().rev()`는 됩니다(`Chars`는 DEI). 하지만 `enumerate`를 거치면 뒤에서 꺼낸 글자의 순번을 정하려고 남은 개수를 알아야 하는데, `Chars`는 UTF-8 바이트를 다 읽기 전에는 글자 수를 모릅니다.
- **해결**: 바이트 위치로 충분하면 `char_indices().rev()`, 글자 순번이 필요하면 `Vec<char>`로 모은 뒤 `chars.iter().enumerate().rev()`, 뒤집은 기준으로 세도 되면 `rev().enumerate()`(예제 07의 2번 구역).

### 4.6 그 밖의 실수

모두 이 툴체인에서 직접 컴파일하거나 실행해 확인했습니다. 어댑터를 만들기만 하고 소비하지 않을 때의 경고(``unused `Map` that must be used``)는 [Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md) 2.9절에 있습니다.

| 실수 | 결과 | 바로잡기 |
| --- | --- | --- |
| `if prices.iter().sum() > 4000` | E0283 ``cannot infer type of the type parameter `S` declared on the method `sum` `` | `sum::<i32>()`. 비교 상대가 정수여도 `sum`의 결과 타입은 정해지지 않음 (2.10절) |
| `(1..=10).filter(…).len()` | E0599 ``no method named `len` found for struct `Filter<I, P>` `` | `count()` (2.15절) |
| `for c in s` (`s: String`) | E0277 `` `String` is not an iterator; try calling `.chars()` or `.bytes()` `` | `s.chars()`, `s.bytes()` (2.2절) |
| `["가", "나"].iter().collect::<String>()` | E0277 ``a value of type `String` cannot be built from an iterator over elements of type `&&str` `` | `into_iter()`나 `.copied()`로 원소를 `&str`로 (2.3절) |
| `FromIterator`가 없는 타입으로 `collect` | E0277 ``the trait `FromIterator<String>` is not implemented for `Playlist` `` | `FromIterator` 구현 (2.19절) |
| `lines.take_while(…)` 뒤에 `lines` 사용 | E0382. `help:`는 `lines.clone()`을 제안하지만, 복제본을 쓰면 원래 반복자는 한 칸도 나아가지 않음 | `lines.by_ref().take_while(…)` (2.8절) |
| `impl Iterator`를 돌려받아 `rev()` | E0277 ``the trait bound `impl Iterator<Item = &str>: DoubleEndedIterator` is not satisfied`` | 반환 타입을 `impl DoubleEndedIterator<…>`로 (2.18절) |
| `take_while`로 앞부분을 떼어 내고 나머지를 이어서 씀 | 컴파일은 되지만 멈춘 원소 하나가 사라짐 | `peekable` + `next_if` (2.8절) |
| 같은 최댓값이 여럿인데 `max_by_key`가 첫 원소를 줄 것이라 기대 | 마지막 원소가 나옴 | 문서에 정해진 동작. 첫 원소가 필요하면 `min_by_key(\|x\| Reverse(키))` 등으로 (2.11절) |

## 5. 연습 문제

### 연습 1. run-length 인코딩 풀기

예제 05의 run-length 인코딩을 되돌리는 `fn decode(s: &str) -> String`을 작성하세요. 글자 뒤의 숫자는 여러 자리일 수 있습니다.

```rust
decode("a3b1c12") // "aaab" + 'c' 열두 개
decode("가2나1")  // "가가나"
```

<details>
<summary>힌트</summary>

- `s.chars().peekable()`로 글자를 하나 꺼낸 뒤, `while let Some(d) = chars.next_if(|c| c.is_ascii_digit())`로 숫자를 이어 읽어 개수를 만듭니다(예제 05의 `tokenize`와 같은 모양).
- 글자를 개수만큼 늘리는 데에는 `iter::repeat_n(글자, 개수)`가 맞습니다. 결과 `String`에 `extend`하면 됩니다(`String: Extend<char>`).
- 숫자가 없는 글자는 1개로 볼지, 에러로 볼지 정해 보세요. 에러로 본다면 반환 타입을 `Result<String, String>`으로 바꿉니다.

</details>

### 연습 2. 괄호 검사

문자열의 괄호 `(`와 `)`가 올바르게 짝지어졌는지 검사하는 `fn balanced(s: &str) -> bool`과, 가장 깊은 중첩 깊이를 돌려주는 `fn max_depth(s: &str) -> usize`를 반복자로 작성하세요. 반복문과 가변 변수를 쓰지 않습니다.

```rust
balanced("(a(b)c)") // true      max_depth("(a(b)c)") // 2
balanced("())(")    // false     max_depth("x")       // 0
```

<details>
<summary>힌트</summary>

- `balanced`: `try_fold(0, |depth, c| …)`로 깊이를 접습니다. `)`를 만났는데 깊이가 0이면 `None`을 돌려 즉시 멈춥니다. 최종 결과가 `Some(0)`이어야 올바릅니다. `"())("`는 세 번째 글자에서 멈춥니다.
- `max_depth`: 깊이의 **변화 과정**이 필요하므로 `scan`으로 글자마다 현재 깊이를 내놓고 `max()`를 씁니다. 빈 문자열이면 `max()`가 `None`이므로 `unwrap_or(0)`을 붙입니다.
- `u32`나 `usize`로 깊이를 세면 `checked_sub`로 "0에서 빼기"를 `None`으로 바꿀 수 있습니다.

</details>

### 연습 3. 버전 번호 정렬

`["1.10.0", "1.9.1", "1.2", "1.2.1", "0.9"]`를 버전 순서(`["0.9", "1.2", "1.2.1", "1.9.1", "1.10.0"]`)로 정렬하세요. 문자열을 숫자 벡터로 바꿔 저장하지 말고, 비교할 때 반복자를 만들어 비교합니다.

<details>
<summary>힌트</summary>

- 예제 06의 `parts` 클로저처럼 `v.split('.').map(|p| p.parse::<u32>().unwrap_or(0))`을 돌려주는 함수를 만들고, `versions.sort_by(|a, b| parts(a).cmp(parts(b)))`로 정렬합니다(`sort_by`는 [ch4](../ch4-collections/README.md) 2.8절).
- 반복자의 `cmp`는 앞부분이 같으면 짧은 쪽을 작게 보므로 `"1.2" < "1.2.1"`이 됩니다.
- 함수로 만들 때 반환 타입은 `impl Iterator<Item = u32>`이고, 입력 `&str`을 빌리는 반복자입니다(2.18절).

</details>

### 연습 4. 정렬된 목록 `SortedVec`

항상 오름차순으로 정렬된 상태를 유지하는 `struct SortedVec<T: Ord> { items: Vec<T> }`를 만드세요. `collect`로 만들 수 있고(`FromIterator`), `extend`로 원소를 더해도 정렬이 유지되며(`Extend`), `for x in &sorted`(`&T`)와 `for x in sorted`(`T`)로 돌 수 있어야 합니다. 그런데 `for x in &mut sorted`는 **일부러** 제공하지 마세요. 왜 그래야 하는지 설명해 보세요.

```rust
let mut s: SortedVec<i32> = [5, 1, 4].into_iter().collect();
s.extend([3, 2]);
let v: Vec<&i32> = (&s).into_iter().collect(); // [1, 2, 3, 4, 5]
```

<details>
<summary>힌트</summary>

- `from_iter`는 `Vec`으로 모은 뒤 `sort()`합니다. `extend`는 안쪽 `Vec`에 `extend`한 뒤 다시 `sort()`하거나, 원소마다 `binary_search`로 자리를 찾아 `insert`합니다(ch4 2.9절).
- `IntoIterator`는 2.19절의 `Playlist`처럼 값과 `&` 두 가지를 안쪽 `Vec`에 맡깁니다. 바운드가 있는 타입이라 `impl<T: Ord> IntoIterator for SortedVec<T>`, `impl<'a, T: Ord> IntoIterator for &'a SortedVec<T>`처럼 적습니다.
- `&mut T`를 내주면 바깥에서 원소를 고쳐 정렬을 깰 수 있습니다. 2.2절에서 본 `HashSet`에 `iter_mut()`가 없는 이유와 같습니다.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::iter`](https://doc.rust-lang.org/std/iter/index.html): "for loops and IntoIterator"(2.2절의 `for` 풀어 쓰기), "Iterating by reference"(`iter`/`iter_mut`와 `&C`/`&mut C`의 관례), "Implementing Iterator", "Laziness", "Infinity" 절이 있습니다. 모듈의 Functions 목록이 2.17절의 함수들입니다.
  - [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html): 모든 어댑터와 소비 메서드. `max_by_key`/`min_by_key`의 같은 값 처리, `size_hint`의 의미와 "unsafe 코드에서 믿지 말 것"이라는 주의가 적혀 있습니다. `Experimental` 표시가 붙은 메서드는 안정판에서 쓸 수 없습니다.
  - [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html): 직접 만든 컬렉션에 구현하는 예제와, 함수 인자를 `IntoIterator`로 받는 예제가 있습니다.
  - [`FromIterator`](https://doc.rust-lang.org/std/iter/trait.FromIterator.html), [`Extend`](https://doc.rust-lang.org/std/iter/trait.Extend.html): Implementors 목록에서 무엇으로 `collect`할 수 있는지, 무엇에 `extend`할 수 있는지 찾습니다(2.3절의 표).
  - [`DoubleEndedIterator`](https://doc.rust-lang.org/std/iter/trait.DoubleEndedIterator.html), [`ExactSizeIterator`](https://doc.rust-lang.org/std/iter/trait.ExactSizeIterator.html): `ExactSizeIterator` 문서의 "When shouldn't an adapter be ExactSizeIterator?" 절이 `Chain`에 `len()`이 없는 이유를 설명합니다.
  - [`Peekable`](https://doc.rust-lang.org/std/iter/struct.Peekable.html): `peek`, `peek_mut`, `next_if`, `next_if_eq`와 1.94에 안정화된 `next_if_map` 등의 예제가 있습니다.
  - [`Vec::drain`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.drain)과 [`vec::Drain`](https://doc.rust-lang.org/std/vec/struct.Drain.html): 2.16절에서 읽은 반복자 구조체 페이지입니다.
- Rust By Example
  - [Iterators](https://doc.rust-lang.org/rust-by-example/trait/iter.html): `Fibonacci` 반복자를 직접 구현하고 `take`, `skip`을 씁니다. 예제 08의 `from_fn` 판과 비교해 보세요.
  - [Searching through iterators](https://doc.rust-lang.org/rust-by-example/fn/closures/closure_examples/iter_find.html), [Iterator::any](https://doc.rust-lang.org/rust-by-example/fn/closures/closure_examples/iter_any.html): `find`와 `any`가 `&mut self`를 받는다는 것과, `iter()`/`into_iter()`에 따라 클로저 인자가 `&&i32`/`&i32`로 달라지는 것을 보여 줍니다.
  - [Iterating over `Result`s](https://doc.rust-lang.org/rust-by-example/error/iter_result.html): 실패를 건너뛰기(`filter_map`), 따로 모으기, 전체 실패로 만들기(`collect`), 성공과 실패를 나누기(`partition`). 2.3절을 읽은 뒤 마지막 방법까지 읽어 보세요.
  - [impl Trait](https://doc.rust-lang.org/rust-by-example/trait/impl_trait.html): 반복자를 `impl Iterator`로 돌려주는 예(2.18절)입니다.
- Rustlings
  - [`18_iterators`](https://github.com/rust-lang/rustlings/tree/main/exercises/18_iterators): `iterators3.rs`는 `Result<Vec<_>, _>`와 `Vec<Result<_, _>>` 두 가지로 모으기(2.3절), `iterators4.rs`는 반복문과 재귀 없이 팩토리얼 구하기(`product`나 `fold`, 2.10절), `iterators5.rs`는 `HashMap`의 값을 반복자로 세기(`filter`와 `count`, 맵 여러 개면 `flat_map`이나 `sum`)를 연습합니다.
