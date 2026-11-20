# ch13. 기본 구현으로 필수 메서드 줄이기

트레이트를 설계하는 사람은 두 독자를 함께 생각해야 합니다. 트레이트를 **구현하는 쪽**은 써야 할 메서드가 적기를 바라고, 트레이트를 **쓰는 쪽**은 부를 수 있는 메서드가 많기를 바랍니다. 두 바람은 부딪히는 것처럼 보이지만 기본 구현(default implementation)으로 함께 채울 수 있습니다. 구현하는 쪽이 반드시 써야 하는 필수 메서드(required method)는 꼭 필요한 것만 두고, 그것으로 만들 수 있는 편의 메서드는 트레이트 안에 본문을 가진 기본 메서드(provided method)로 넉넉하게 주면 됩니다. 표준 라이브러리의 `Iterator`가 이 모양입니다. 구현하는 쪽은 `next` 하나만 쓰고, 쓰는 쪽은 `map`, `filter`, `count` 같은 메서드를 수십 개 얻습니다.

기본 구현의 문법과 재정의하는 법은 본편에서 배웠으므로, 이 챕터는 트레이트를 **설계하는 쪽의 판단**만 다룹니다. 무엇을 필수로 남기고 무엇에 기본 구현을 줄지, 구현하는 쪽은 언제 기본 구현을 재정의할지, 일부 구현에만 맞는 메서드는 어떻게 줄지입니다. [ch2](../ch2-express-common-behavior/README.md)가 미뤄 둔 "공개 트레이트에 메서드를 나중에 더하는 문제"도 2.5절에서 다룹니다.

- 원제: [Item 13: Use default implementations to minimize required trait methods](https://effective-rust.com/default-impl.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch13-default-implementations`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 본편 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.5절: 기본 구현, 재정의, 필수 메서드를 빠뜨렸을 때의 E0046. 2.11절: 바운드를 이용한 조건부 구현
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.10~2.11절: 완전 정규화 문법, 슈퍼트레이트
- [Part 1 ch5](../../part1-standard-lib/ch5-iterators/README.md) 2.1절, 2.15절, 2.19절: `Iterator`의 필수 메서드, `size_hint`와 `ExactSizeIterator`, 직접 만든 반복자
- [Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.2절: 메서드 탐색 순서(고유 메서드와 트레이트 메서드)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 필수 메서드를 다른 메서드로 만들 수 없는 것만으로 줄이고 편의 메서드를 기본 메서드로 넉넉하게 주는 트레이트를 설계할 수 있고, 이 설계가 구현하는 쪽과 쓰는 쪽에 각각 무엇을 주는지 설명할 수 있다.
2. 구현하는 타입이 더 나은 방법을 알 때 기본 메서드를 재정의하고, 재정의한 구현이 기본 구현과 같은 답을 내는지 확인하는 코드를 작성할 수 있다.
3. 추가 능력이 필요한 기본 메서드에만 트레이트 바운드(`where Self: Display` 등)를 붙일 수 있고, 슈퍼트레이트로 요구할 때와의 차이를 설명할 수 있다.
4. 트레이트에 기본 메서드를 더하는 것이 필수 메서드를 더하는 것과 달리 대체로 하위 호환되는 이유와, 이름이 겹칠 때 생기는 예외를 설명할 수 있다.

## 2. 핵심 개념

2.1절에서 표준 라이브러리 트레이트의 모양을 보고, 2.2~2.4절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 세 예제 모두 마라톤 코스 위의 지점(급수대, 거리 표지) 위치를 묻는 트레이트 `Checkpoints`를 씁니다. 위치는 출발점에서 잰 미터(`u32`)입니다.

### 2.1 두 독자: 구현하는 쪽과 쓰는 쪽

표준 라이브러리 문서의 트레이트 페이지는 메서드를 Required Methods와 Provided Methods 두 절로 나눠 보여 줍니다. 앞의 것이 필수 메서드, 뒤의 것이 기본 메서드입니다. 1.100.0 로컬 문서에서 세어 보면 다음과 같습니다.

| 트레이트 | 필수 메서드 | 기본 메서드 |
| --- | --- | --- |
| `Iterator` | `next` | 75개(`size_hint`, `count`, `nth`, `map`, `filter`, `fold` 등, 그중 15개는 nightly 전용) |
| `DoubleEndedIterator` | `next_back` | `nth_back`, `rfold`, `rfind`, `try_rfold`(그 밖에 nightly 전용 2개) |
| `ExactSizeIterator` | 없음 | `len`, `is_empty`(nightly 전용) |
| `PartialEq` | `eq` | `ne` |
| `PartialOrd` | `partial_cmp` | `lt`, `le`, `gt`, `ge` |
| `Hasher` | `finish`, `write` | `write_u8`부터 `write_isize`까지 12개(그 밖에 nightly 전용 2개) |
| `io::Write` | `write`, `flush` | `write_all`, `write_fmt`, `write_vectored`, `by_ref`(그 밖에 nightly 전용 2개) |

필수 메서드가 꼭 하나여야 하는 것은 아닙니다. `Hasher`의 `finish`(결과 꺼내기)와 `write`(바이트 넣기)는 어느 쪽으로도 다른 쪽을 만들 수 없으므로 둘 다 필수입니다. 기준은 "다른 메서드로 만들 수 없는가"입니다. `ExactSizeIterator`처럼 필수 메서드가 하나도 없는 트레이트도 있습니다. `len`의 기본 구현이 `Iterator::size_hint`의 결과를 돌려주기 때문에, 구현하는 쪽은 `size_hint`를 정확하게 재정의하고 `impl ExactSizeIterator for Pages {}`라는 빈 블록만 쓰면 됩니다(Part 1 ch5 2.19절). 표준 라이브러리 소스를 보면 이 기본 구현은 `size_hint`의 최솟값과 최댓값이 같은지 `assert_eq!`로 확인한 뒤 돌려줍니다. 기본 메서드가 기대는 약속(정확한 `size_hint`)을 구현하는 쪽이 지키지 않으면 패닉하도록 만든 것입니다.

### 2.2 지침 1: 필수 메서드는 최소로, 편의 메서드는 기본 구현으로

**문제 상황.** 마라톤 대회 앱에 급수대 위치를 묻는 트레이트를 만듭니다. 코스마다 위치를 갖고 있는 방법이 다릅니다. 위치 목록을 그대로 가진 코스(`Listed`)도 있고, "5000m부터 5000m마다 4곳"처럼 규칙만 가진 코스(`Even`)도 있습니다. 쓰는 쪽에 주고 싶은 질문은 여섯 가지입니다. 몇 번째 급수대의 위치, 급수대 수, 마지막 급수대, 지금 위치에서 다음 급수대, 거기까지 남은 거리, 급수대 사이가 가장 먼 구간입니다.

**비권장 코드.** 여섯 질문을 모두 필수 메서드로 둡니다.

```rust
pub trait Checkpoints {
    fn at(&self, index: usize) -> Option<u32>;
    fn count(&self) -> usize;
    fn last(&self) -> Option<u32>;
    fn next_from(&self, pos: u32) -> Option<u32>;
    fn distance_to_next(&self, pos: u32) -> Option<u32>;
    fn longest_gap(&self) -> u32;
}
```

구현하는 쪽은 타입마다 여섯 메서드를 모두 써야 합니다. [`01_minimal_required.rs`](examples/01_minimal_required.rs)의 비권장 판에서 두 구현은 57줄입니다. 일이 많은 것보다 더 큰 문제는 같은 규칙을 구현마다 따로 정하게 된다는 점입니다. 예제의 두 코스는 급수대가 똑같이 5000, 10000, 15000, 20000m에 있는데, 급수대 바로 위인 10000m에서 물으면 답이 갈립니다.

```text
  목록형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 15000m(5000m 앞)
  간격형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 10000m(0m 앞)
```

목록형을 쓴 사람은 `next_from`의 "앞으로 만날"을 "지금 위치보다 뒤"(`p > pos`)로, 간격형을 쓴 사람은 "지금 위치 포함"(`p >= pos`)으로 읽었습니다. 둘 다 컴파일되고, 쓰는 쪽은 같은 트레이트의 같은 메서드가 타입에 따라 다른 답을 낸다는 것을 알 수 없습니다.

**권장 코드.** 필수 메서드는 `at` 하나로 두고, 나머지 다섯은 `at`만으로 만든 기본 메서드로 줍니다.

```rust
pub trait Checkpoints {
    /// index번째 지점의 위치. 지점이 없으면 None.
    /// index가 커질수록 위치도 커져야 하고, 중간에 None이 끼면 안 됩니다.
    fn at(&self, index: usize) -> Option<u32>;

    fn count(&self) -> usize {
        (0..).map_while(|i| self.at(i)).count()
    }
    fn next_from(&self, pos: u32) -> Option<u32> {
        (0..).map_while(|i| self.at(i)).find(|&p| p >= pos)
    }
    fn distance_to_next(&self, pos: u32) -> Option<u32> {
        self.next_from(pos).map(|p| p - pos)
    }
    // last, longest_gap도 같은 방식
}
```

무엇을 필수로 남길지는 "타입마다 데이터를 어떻게 갖고 있는지에 달린 연산이 무엇인가"로 정합니다. 여기서는 "몇 번째 위치를 꺼낸다"가 그것이고, 나머지 질문은 모두 그 연산을 되풀이하면 답할 수 있습니다. `Iterator`가 "다음 원소를 꺼낸다"(`next`)만 요구하는 것과 같은 모양입니다.

결과는 세 가지입니다. 첫째, 구현하는 쪽은 `at` 하나만 씁니다. 같은 두 타입의 구현이 14줄로 줄었습니다. 둘째, "지금 위치 포함"이라는 규칙이 트레이트 안 한 곳에만 있으므로 두 코스가 같은 답(`10000m(0m 앞)`)을 냅니다. 셋째, 나중에 추가한 구현도 바로 모든 메서드를 얻습니다. 예제의 `Single`(급수대가 한 곳뿐인 어린이 코스)은 `at` 하나를 다섯 줄로 구현하고, 쓰는 쪽 함수 `briefing`에 그대로 넘어갑니다. 쓰는 쪽이 부를 수 있는 메서드는 두 설계 모두 여섯 개로 같습니다.

**지침을 따르지 않아도 되는 경우.** 기본 구현을 만들 수 있더라도, 그 기본 구현이 어떤 구현에서는 틀린 답을 낸다면 기본 메서드로 두지 않고 필수로 둡니다. 재정의를 잊은 구현이 컴파일 에러 없이 틀린 답을 내기 때문입니다. 반대로 모든 구현에 맞는 답이라면 쓸모가 적더라도 기본값으로 둘 수 있습니다. `Iterator::size_hint`의 기본 구현은 `(0, None)`("남은 개수를 모름")인데, 문서는 이것이 어떤 반복자에도 맞는 답이라고 밝힙니다. 틀리지 않는 대신 쓸모가 적은 값을 기본으로 둔 것입니다.

### 2.3 지침 2: 더 나은 방법을 아는 구현은 기본 메서드를 재정의한다

**문제 상황.** 1000m부터 1000m마다 거리 표지가 있는 코스(표지 42개)를 `Checkpoints`로 구현합니다. 이 타입은 첫 위치, 간격, 개수를 필드로 들고 있어서 어떤 질문이든 계산 한 번으로 답할 수 있습니다. 그런데 `at`만 구현하면 기본 메서드는 그 사실을 모릅니다.

**비권장 코드.** 필수 메서드만 구현하고 나머지는 기본 구현에 맡깁니다.

```rust
impl Checkpoints for PlainMarkers {
    fn at(&self, index: usize) -> Option<u32> { self.0.get(index) }
    // count, last, next_from은 기본 구현: at(0), at(1), …을 차례로 부릅니다.
}
```

[`02_override_default.rs`](examples/02_override_default.rs)는 시간 대신 `at`이 불린 횟수를 셉니다. `count()`는 `None`이 나올 때까지 43번, `last()`는 그 43번에 한 번을 더해 44번, `next_from(37500)`은 앞에서부터 38번 부릅니다. 답은 맞지만, 표지가 많아질수록 일도 그만큼 늘어납니다.

**권장 코드.** 간격을 아는 타입이 세 메서드를 계산으로 재정의합니다.

```rust
impl Checkpoints for FastMarkers {
    fn at(&self, index: usize) -> Option<u32> { self.0.get(index) }
    fn count(&self) -> usize { self.0.n }
    fn last(&self) -> Option<u32> { self.0.n.checked_sub(1).map(|i| self.0.position(i)) }
    fn next_from(&self, pos: u32) -> Option<u32> {
        let s = &self.0;
        let index = if pos <= s.first { 0 } else { (pos - s.first).div_ceil(s.every) as usize };
        if index < s.n { Some(s.position(index)) } else { None }
    }
}
```

세 메서드의 답은 그대로이고 `at` 호출은 0번이 됩니다. 쓰는 쪽 코드는 한 줄도 바뀌지 않습니다. 기본 구현은 말 그대로 "기본"이고, 구현하는 쪽이 자기 타입에 대해 더 아는 것이 있으면 그 지식을 넣을 자리가 재정의입니다.

표준 라이브러리도 이렇게 합니다. 범위 `a..b`(`ops::Range`)의 `Iterator` 구현은 `next` 말고도 `size_hint`, `count`, `nth`, `last`, `min`, `max`를 재정의합니다(표준 라이브러리 소스 `core/src/iter/range.rs`). 예를 들어 `count`는 원소를 하나씩 꺼내지 않고 시작과 끝 사이의 걸음 수(`Step::steps_between`)로 계산하고, `max`는 `next_back()` 한 번으로 마지막 원소를 돌려줍니다. `ExactSizeIterator` 문서도 `len`에 대해 "보통은 직접 구현하지 않지만, 기본 구현보다 빠르게 만들 수 있다면 재정의하는 것이 맞다"고 안내합니다. 책이 재정의의 예로 든 `ExactSizeIterator::is_empty`는 1.100.0에서도 nightly 전용입니다(2.6절).

**재정의한 메서드는 기본 구현과 같은 답을 내야 합니다.** 쓰는 쪽은 메서드가 재정의됐는지 모르고 부릅니다. 그러니 재정의한 구현은 기본 구현이 냈을 답을 그대로 내야 합니다. 예제의 세 번째 판 `SloppyMarkers`는 `next_from`을 계산으로 재정의하면서 올림(`div_ceil`) 대신 "내림한 뒤 1 더하기"를 썼습니다. 대부분의 위치에서 답이 같아서 몇 군데 시험해서는 놓치기 쉽습니다. 그래서 예제는 재정의하지 않은 `PlainMarkers`를 기준으로 삼아 0m부터 45000m까지 100m 간격 451곳에서 두 판의 답을 비교합니다.

```text
  재정의한 판: 다른 곳 0곳
  잘못 재정의한 판: 다른 곳 41곳, 처음 세 곳 [2000, 3000, 4000]
  next_from(2000): 기본 구현 Some(2000), 잘못 재정의한 판 Some(3000)
```

표지 바로 위(나누어떨어지는 위치)에서만 표지 하나를 건너뜁니다. 2.2절의 비권장 판에서 구현마다 규칙이 갈린 것과 같은 문제가, 재정의한 메서드 하나에서 다시 생긴 것입니다. 기본 메서드를 재정의할 때는 이렇게 기본 구현을 쓰는 판과 나란히 비교하는 검사를 함께 둡니다. 표준 라이브러리 문서도 같은 약속을 요구합니다. `PartialEq`는 `a != b`와 `!(a == b)`가 언제나 같아야 한다고 적고, `Iterator::size_hint`는 틀린 추정을 돌려주는 것이 트레이트의 약속을 어기는 것이라고 적습니다.

**지침을 따르지 않아도 되는 경우.** 더 나은 방법이 없거나 이득이 작다면 재정의하지 않습니다. 재정의한 메서드마다 기본 구현과 어긋날 위험이 생기기 때문입니다. `PartialEq` 문서는 `ne`의 기본 구현이 거의 언제나 충분하므로 아주 좋은 이유가 없으면 재정의하지 말라고 적습니다. 원소가 몇 개뿐인 타입이라면 기본 구현이 하는 일도 몇 번의 호출뿐입니다.

### 2.4 지침 3: 추가 능력이 필요한 기본 메서드에는 그 메서드에만 바운드를 붙인다

**문제 상황.** 대회 안내 방송에 쓸 문구를 만드는 기본 메서드 `announce(pos)`를 더하려 합니다. "강변 코스: 다음 급수대까지 1000m"처럼 코스 이름이 들어가야 하므로 본문에서 `{self}`를 쓰고, 그러려면 `Self: Display`가 필요합니다. 하지만 모든 코스에 이름이 있지는 않습니다. 연습할 때 그때그때 만드는 코스(`Stops`)는 위치 목록뿐이고 안내 방송도 쓰지 않습니다.

**비권장 코드.** `Display`를 슈퍼트레이트로 요구합니다.

```rust
pub trait Checkpoints: fmt::Display {
    fn at(&self, index: usize) -> Option<u32>;
    fn next_from(&self, pos: u32) -> Option<u32> { /* .. */ }
    fn announce(&self, pos: u32) -> String { /* format!("{self}: ...") */ }
}
```

슈퍼트레이트는 트레이트 전체의 조건이라서(Part 0 ch19 2.11절), `Stops`도 `Checkpoints`를 구현하려면 `Display`부터 구현해야 합니다. 보여 줄 이름이 없으니 [`03_bounded_default.rs`](examples/03_bounded_default.rs)의 비권장 판은 `"(이름 없음)"`을 지어냈습니다. 구현하는 쪽의 일이 늘었을 뿐 아니라, 지어낸 표현이 쓰는 쪽에 그대로 열려 `{practice}`가 `(이름 없음)`을 출력하고 `practice.announce(1500)`이 `(이름 없음): 다음 급수대까지 500m`라는 뜻 없는 문구를 만듭니다.

**권장 코드.** 바운드를 `announce` 하나에만 붙입니다.

```rust
pub trait Checkpoints {
    fn at(&self, index: usize) -> Option<u32>;
    fn next_from(&self, pos: u32) -> Option<u32> { /* .. */ }
    fn announce(&self, pos: u32) -> String
    where
        Self: fmt::Display,
    { /* format!("{self}: ...") */ }
}
```

이제 `Stops`는 다시 `at` 하나만 구현하고, `next_from` 같은 나머지 기본 메서드를 모두 씁니다. `announce`는 `Display`를 구현한 `NamedCourse`에서만 부를 수 있고, `Stops`에서 부르면 E0277입니다(4.1절). 쓰는 쪽의 제네릭 함수도 같은 원리로 필요한 만큼만 요구합니다. 예제에서 `announce`를 부르는 `morning_call`만 `C: Checkpoints + fmt::Display`이고, 위치만 보는 `next_stop`은 `C: Checkpoints`로 충분합니다. [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.11절에서 `impl<T: Display + PartialOrd> Pair<T>`처럼 조건을 만족하는 타입에만 메서드를 준 것과 같은 생각을, 트레이트의 기본 메서드에 적용한 것입니다.

표준 라이브러리의 `Iterator::cloned`가 대표적인 예입니다. 1.100.0 문서의 시그니처는 다음과 같습니다.

```rust
fn cloned<'a, T>(self) -> Cloned<Self>
where
    T: Clone + 'a,
    Self: Sized + Iterator<Item = &'a T>,
```

원소가 `Clone`인 타입의 참조를 내놓는 반복자에서만 쓸 수 있다는 뜻입니다. `Clone`이 아닌 `Ticket`의 벡터에서 `v.iter().cloned()`를 부르면 ``error[E0277]: the trait bound `Ticket: Clone` is not satisfied``와 ``note: required by a bound in `cloned` ``가 나옵니다(같은 모양의 코드로 확인). 표준 라이브러리의 기본 메서드에 자주 붙어 있는 `where Self: Sized`는 트레이트 객체와 관련된 바운드라서 [`ch12-generics-vs-trait-objects`](../ch12-generics-vs-trait-objects/README.md)에서 다룹니다.

**지침을 따르지 않아도 되는 경우.** 트레이트의 뜻 자체가 그 능력을 전제한다면 슈퍼트레이트가 맞습니다. `Ord`는 `pub trait Ord: Eq + PartialOrd`로 선언되어 있습니다. 순서를 정할 수 있는 타입이라면 같은지도 판단할 수 있어야 하기 때문입니다. `Error`도 `pub trait Error: Debug + Display`라서 모든 에러가 메시지를 보여 줄 수 있습니다([ch4](../ch4-idiomatic-errors/README.md)). 기준은 "이 능력이 없는 구현이 있을 수 있는가"입니다. 있을 수 있다면 그 능력을 쓰는 메서드에만 바운드를 붙입니다.

### 2.5 기본 메서드를 나중에 더할 때

[ch2](../ch2-express-common-behavior/README.md) 2.4절은 공개 트레이트에 메서드를 나중에 더하는 문제를 이 챕터로 미뤘습니다. 필수 메서드를 더하면 그 트레이트의 구현이 모두 ``error[E0046]: not all trait items implemented, missing: `remaining` ``으로 깨지고, 구현하는 쪽은 다른 크레이트에 있으므로 트레이트를 만든 쪽이 고쳐 줄 수도 없습니다. 기본 메서드는 본문이 트레이트 안에 있으므로 기존 구현은 고칠 것이 없고 새 메서드를 그대로 얻습니다. 그래서 기본 메서드 추가는 대체로 하위 호환되는 변경입니다. "대체로"인 이유는 이름입니다. 그 타입이 구현한 다른 트레이트에 같은 이름의 메서드가 있으면, 두 트레이트를 함께 가져온 쓰는 쪽의 `trail.remaining(..)`이 E0034로 깨집니다(4.2절). 타입의 고유 메서드와 이름이 같으면 기존 호출은 계속 고유 메서드를 부르고, 새 메서드는 `Checkpoints::remaining(&trail, pos)`처럼 트레이트 이름을 적어 부릅니다. 단, 고유 메서드가 이기는 것은 두 메서드의 수신자가 같은 모양일 때입니다. 메서드 탐색은 `T`, `&T`, `&mut T` 단계 순서로 진행하므로, 고유 메서드가 `&mut self`이고 새 기본 메서드가 `&self`이면 새 기본 메서드가 먼저 잡힙니다. 고유 메서드 `fn remaining(&mut self, pos: u32) -> u32`를 가진 타입으로 확인해 보니, 같은 `m.remaining(1500)` 호출이 컴파일 에러 없이 새 기본 메서드의 결과를 돌려주었습니다(탐색 순서는 Part 2 ch3 2.2절). 어떤 변경이 호환되지 않는지의 전체 규칙은 [ch21](../ch21-semver/README.md)에서 다룹니다.

### 2.6 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **책에 없는 지침.** 다음은 책의 Item 13이 직접 권하지 않는 내용입니다(2026-11-19에 온라인판에서 확인). 책은 필수 메서드를 줄이고 기본 메서드를 넉넉하게 줄 것, 기본 구현은 재정의할 수 있다는 것, 기본 메서드에 바운드를 붙일 수 있다는 것, 기본 메서드 추가는 대체로 호환된다는 것까지 다룹니다.
  - 지침 2의 "재정의한 메서드는 기본 구현과 같은 답을 내야 하고, 기본 구현을 쓰는 판과 비교해 확인한다": `PartialEq` 문서(`eq`와 `ne`가 서로 맞아야 함), `Iterator::size_hint` 문서(틀린 추정은 트레이트의 약속 위반)와 예제 02의 실행 결과가 근거입니다.
  - 지침 3의 슈퍼트레이트와의 비교: 책은 기본 메서드에 바운드를 붙일 수 있다는 것까지만 말합니다. 비교와 판단 기준("이 능력이 없는 구현이 있을 수 있는가")은 이 챕터가 더한 것이고, `Ord`와 `Error`의 선언이 근거입니다.
  - 세 지침의 "따르지 않아도 되는 경우": 책은 예외를 다루지 않습니다. 지침 1은 `Iterator::size_hint` 문서(기본 구현 `(0, None)`은 모든 반복자에 맞는 답), 지침 2는 `PartialEq` 문서(`ne`를 재정의하지 말 것), 지침 3은 `Ord`와 `Error`의 선언이 근거입니다.
  - 4.3절의 `Read` 설명(서로를 부르는 두 기본 메서드와, 둘 다 재정의하지 않으면 생기는 문제): 1.100.0 표준 라이브러리 소스와 직접 컴파일한 결과가 근거입니다.
- **`Iterator`의 기본 메서드 수.** 책은 50개가 넘는다고 적습니다. 1.100.0 로컬 문서의 Provided Methods는 75개이고, 그중 15개(`next_chunk`, `intersperse`, `map_windows`, `try_collect` 등)는 nightly 전용입니다. 필수 메서드는 여전히 `next` 하나입니다.
- **`ExactSizeIterator::is_empty`는 1.100.0에서도 nightly 전용입니다(달라지지 않은 점).** 책은 이 메서드를 재정의의 예로 들면서 nightly 전용이라고 각주에 적었고, 지금도 같습니다(`exact_size_is_empty`, 이슈 #35428). stable 툴체인에서 `impl ExactSizeIterator` 블록에 `is_empty`를 쓰면 ``error[E0658]: use of unstable library feature `exact_size_is_empty` ``입니다. stable에서 재정의할 수 있는 같은 종류의 메서드는 `len`입니다(2.3절).
- **고유 메서드가 이긴다는 각주의 조건.** 책은 새 기본 메서드와 같은 이름의 고유 메서드가 있으면 고유 메서드가 우선한다고 각주에 적습니다. 1.100.0에서 확인해 보니 두 메서드의 수신자가 같은 모양일 때 그렇고, 고유 메서드가 `&mut self`이고 기본 메서드가 `&self`이면 기본 메서드가 먼저 잡혔습니다(2.5절).

### 2.7 기억할 것

- 필수 메서드는 "다른 메서드로 만들 수 없는 것"만 둡니다. 나머지는 필수 메서드로 만든 기본 메서드로 넉넉하게 줍니다. 구현하는 쪽의 일이 줄고, 규칙이 한 곳에 모입니다.
- 기본 구현은 모든 구현에 맞는 답을 내야 합니다. 그럴 수 없는 메서드는 필수로 둡니다.
- 더 나은 방법을 아는 구현은 기본 메서드를 재정의합니다. 재정의한 결과는 기본 구현과 같아야 하므로, 기본 구현을 쓰는 판과 나란히 비교해 확인합니다.
- 일부 구현에만 있는 능력이 필요한 기본 메서드에는 그 메서드에만 `where` 바운드를 붙입니다. 슈퍼트레이트는 트레이트의 뜻이 그 능력을 전제할 때 씁니다.
- 기본 메서드 추가는 대체로 하위 호환되지만, 다른 트레이트의 메서드나 고유 메서드와 이름이 겹치는지 확인합니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행합니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_minimal_required.rs`](examples/01_minimal_required.rs) | 필수는 적게 | 필수 메서드 6개 | 필수 1개 + 기본 5개 | 2.2 |
| [`02_override_default.rs`](examples/02_override_default.rs) | 재정의 | 기본 구현에 맡김, 잘못 재정의함 | 계산으로 재정의하고 기본 구현과 비교 | 2.3 |
| [`03_bounded_default.rs`](examples/03_bounded_default.rs) | 메서드에만 바운드 | `Display` 슈퍼트레이트 | `announce`에만 `where Self: Display` | 2.4 |

### 01_minimal_required — 필수 메서드는 적게

두 설계를 `wide`와 `narrow` 모듈에 나눠 두고, 같은 위치의 급수대를 가진 두 코스에 같은 질문을 합니다. 쓰는 쪽 함수(`briefing_wide`, `briefing`)는 같은 메서드를 부릅니다. 마지막 줄은 권장 판에 나중에 추가한 구현 `Single`입니다.

```sh
cargo run -p er-ch13-default-implementations --example 01_minimal_required
```

```text
[비권장] 필수 메서드 6개: 구현 두 개가 메서드 12개를 씁니다
  목록형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 15000m(5000m 앞)
  간격형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 10000m(0m 앞)
[권장] 필수 메서드 1개 + 기본 메서드 5개: 구현 두 개가 메서드 2개를 씁니다
  목록형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 10000m(0m 앞)
  간격형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 10000m(0m 앞)
[권장] 새 구현도 at 하나만 쓰면 여섯 메서드를 모두 얻습니다
  어린이 코스: 1곳, 마지막 Some(1500), 가장 긴 구간 1500m, 1000m에서 다음 급수대 1500m(500m 앞)
```

비권장 판의 두 줄이 다른 것은 구현마다 `next_from`을 따로 써서이고, 권장 판에서는 규칙이 기본 메서드 한 곳에 있어 두 줄이 같습니다.

### 02_override_default — 재정의와 그 약속

세 판은 모두 같은 `Spacing`(1000m부터 1000m마다 42곳)을 감쌉니다. `Spacing::get`이 불린 횟수를 `Cell`에 세어 두었다가 `take_calls`로 꺼냅니다([Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md)의 `Cell`은 `&self`로도 값을 바꾸기 위해 썼습니다). 재정의한 메서드는 횟수를 세지 않는 `position`으로 계산합니다.

```sh
cargo run -p er-ch13-default-implementations --example 02_override_default
```

```text
1. 기본 구현에 맡긴 판: at을 처음부터 차례로 부릅니다
  count() = 42 (at 호출 43번)
  last() = Some(42000) (at 호출 44번)
  next_from(37500) = Some(38000) (at 호출 38번)
2. 재정의한 판: 같은 답을 계산으로 바로 냅니다
  count() = 42 (at 호출 0번)
  last() = Some(42000) (at 호출 0번)
  next_from(37500) = Some(38000) (at 호출 0번)
3. 재정의한 next_from이 기본 구현과 같은 답을 내는지 451곳에서 비교합니다
  재정의한 판: 다른 곳 0곳
  잘못 재정의한 판: 다른 곳 41곳, 처음 세 곳 [2000, 3000, 4000]
  next_from(2000): 기본 구현 Some(2000), 잘못 재정의한 판 Some(3000)
```

잘못 재정의한 판이 틀리는 41곳은 2000m부터 42000m까지 표지가 놓인 위치입니다. 1000m는 `pos <= first` 갈래가 따로 처리해서 맞습니다.

### 03_bounded_default — 메서드에만 바운드

두 설계를 `with_supertrait`와 `with_bound` 모듈에 나눠 두고, 이름 있는 코스(`NamedCourse`)와 이름 없는 연습 코스(`Stops`)를 각각 구현합니다. 권장 판에서 `practice.announce(1500)`은 컴파일되지 않으므로 주석으로만 남겼습니다(4.1절).

```sh
cargo run -p er-ch13-default-implementations --example 03_bounded_default
```

```text
[비권장] Display를 슈퍼트레이트로 요구
  강변 코스: 다음 급수대까지 1000m
  연습 코스의 다음 급수대: Some(2000)
  지어낸 Display: (이름 없음)
  (이름 없음): 다음 급수대까지 500m
[권장] announce에만 where Self: Display
  강변 코스: 다음 급수대까지 1000m
  강변 코스: 남은 급수대 없음
  연습 코스의 다음 급수대: Some(2000)
  next_stop(연습 코스, 2500): Some(3000)
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch13-default-implementations
rustc --edition 2024 compile_fail/e0277_bounded_default_method.rs --out-dir /tmp/er-ch13
rustc --edition 2024 compile_fail/e0034_new_default_method_clash.rs --out-dir /tmp/er-ch13
```

### 4.1 바운드가 붙은 기본 메서드를 조건이 안 맞는 타입에서 부름 — E0277

파일: [`compile_fail/e0277_bounded_default_method.rs`](compile_fail/e0277_bounded_default_method.rs)

```text
error[E0277]: `Stops` doesn't implement `std::fmt::Display`
  --> compile_fail/e0277_bounded_default_method.rs:44:29
   |
44 |     println!("{}", practice.announce(1500)); // E0277
   |                             ^^^^^^^^ unsatisfied trait bound
   |
help: the trait `std::fmt::Display` is not implemented for `Stops`
  --> compile_fail/e0277_bounded_default_method.rs:33:1
   |
33 | struct Stops(Vec<u32>);
   | ^^^^^^^^^^^^
note: required by a bound in `Checkpoints::announce`
  --> compile_fail/e0277_bounded_default_method.rs:24:15
   |
15 | trait Checkpoints {
...
22 |     fn announce(&self, pos: u32) -> String
23 |     where
24 |         Self: fmt::Display,
   |               ^^^^^^^^^^^^ required by this bound in `Checkpoints::announce`
```

- **읽는 법**: `note`가 핵심입니다. `Stops`는 `Checkpoints`를 구현했지만, `announce`라는 메서드 하나에 붙은 바운드 `Self: fmt::Display`를 만족하지 못합니다. 바로 앞 줄의 `practice.next_from(1500)`은 바운드가 없는 기본 메서드라서 문제없이 컴파일됩니다.
- **원인**: 바운드가 붙은 기본 메서드는 트레이트를 구현했다고 해서 생기지 않고, 바운드까지 만족해야 생깁니다(2.4절).
- **해결**: 이름이 필요 없는 코스라면 `announce` 대신 `next_from`처럼 바운드 없는 메서드를 씁니다. 이름을 보여 줄 수 있는 타입이라면 `Display`를 구현합니다(예제 03의 `NamedCourse`). 이 에러를 없애려고 뜻 없는 `Display`를 지어내는 것은 비권장 판과 같은 결과가 되므로 피합니다.

### 4.2 새 기본 메서드가 다른 트레이트의 메서드와 이름이 겹침 — E0034

파일: [`compile_fail/e0034_new_default_method_clash.rs`](compile_fail/e0034_new_default_method_clash.rs)

파일 안의 `course_lib` 모듈을 다른 사람이 관리하는 라이브러리라고 생각합니다. v1의 `Checkpoints`에는 `at`과 `count`만 있었고, 사용자는 그때 자기 트레이트 `TrailExt`에 "결승점까지 남은 거리" `remaining`을 만들어 두었습니다. v2에서 라이브러리가 기본 메서드 `remaining`("남은 지점 수")을 추가했습니다. `impl Checkpoints for Trail`은 그대로 컴파일되지만, 부르는 곳이 깨집니다.

```text
error[E0034]: multiple applicable items in scope
  --> compile_fail/e0034_new_default_method_clash.rs:61:33
   |
61 |     println!("남은 거리 {}m", trail.remaining(30000)); // E0034
   |                                     ^^^^^^^^^ multiple `remaining` found
   |
note: candidate #1 is defined in an impl of the trait `Checkpoints` for the type `Trail`
  --> compile_fail/e0034_new_default_method_clash.rs:24:9
   |
24 |         fn remaining(&self, pos: u32) -> usize {
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
note: candidate #2 is defined in an impl of the trait `TrailExt` for the type `Trail`
  --> compile_fail/e0034_new_default_method_clash.rs:50:5
   |
50 |     fn remaining(&self, pos: u32) -> u32 {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: disambiguate the method for candidate #1
   |
61 -     println!("남은 거리 {}m", trail.remaining(30000)); // E0034
61 +     println!("남은 거리 {}m", Checkpoints::remaining(&trail, 30000)); // E0034
   |
help: disambiguate the method for candidate #2
   |
61 -     println!("남은 거리 {}m", trail.remaining(30000)); // E0034
61 +     println!("남은 거리 {}m", TrailExt::remaining(&trail, 30000)); // E0034
   |
```

- **읽는 법**: 후보 #1은 라이브러리 트레이트의 **기본 메서드 본문**(24행)을 가리킵니다. `Trail`의 `impl` 블록에는 `remaining`이 없지만 기본 메서드로 생겼기 때문입니다. 두 후보 모두 수신자가 `&self`라 메서드 탐색의 같은 단계에서 둘이 잡혔습니다. [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.10절의 `Pilot`/`Wizard`는 처음부터 이름이 겹치게 만든 경우이고, Part 2 [ch3](../../part2-editions/ch3-rust-2021/README.md)와 [ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md)의 E0034는 에디션이 프렐류드에 트레이트를 더해서 생긴 경우입니다. 여기서는 의존하는 라이브러리가 기본 메서드를 하나 더한 것만으로 생겼습니다.
- **원인**: 기본 메서드 추가는 구현하는 쪽을 깨지 않지만, 쓰는 쪽의 메서드 호출 문법은 이름으로 후보를 찾으므로 같은 이름이 둘이 되면 모호해집니다(2.5절).
- **해결**: 사용자가 원래 뜻한 메서드의 트레이트 이름을 적습니다. 여기서는 두 번째 도움말대로 `TrailExt::remaining(&trail, 30000)`입니다. 고쳐서 실행하면 `남은 거리 12195m`가 나오고, 새 메서드는 `Checkpoints::remaining(&trail, 30000)`으로 불러 `2`(30000m 이후의 지점 30000, 40000m)를 얻습니다. 자기 트레이트의 메서드 이름을 바꿀 수 있다면 그것도 방법입니다. 라이브러리를 만드는 쪽에서는 흔한 이름의 기본 메서드를 더할 때 이런 충돌이 생길 수 있다는 것을 알고 이름을 고릅니다.

### 4.3 그 밖의 에러

모두 같은 모양의 코드를 1.100.0에서 컴파일해 확인했습니다.

| 코드 | 결과 | 다룬 곳 |
| --- | --- | --- |
| 필수 메서드를 구현하지 않음 | E0046 ``not all trait items implemented, missing: `..` `` | [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.5절 |
| 이미 구현이 있는 트레이트에 필수 메서드를 추가 | 모든 구현에서 E0046 | 2.5절 |
| `impl Read for X {}`(메서드 없이) | E0046 ``missing one of: `read_buf`, `read` `` | 아래 설명 |
| `Clone`이 아닌 원소의 반복자에 `cloned()` | E0277 ``the trait bound `Ticket: Clone` is not satisfied`` | 2.4절 |
| stable에서 `ExactSizeIterator`의 `is_empty`를 재정의 | E0658 ``use of unstable library feature `exact_size_is_empty` `` | 2.6절 |

세 번째 줄은 1.100.0의 `io::Read`에서 볼 수 있는 모양입니다. `Read` 문서는 여전히 "필수 메서드는 `read` 하나"라고 설명하지만, 로컬 문서에서 `read`는 Provided Methods 절에 있습니다. 표준 라이브러리 소스에서는 `read`의 기본 구현이 nightly 전용 `read_buf`를 부르고, `read_buf`의 기본 구현이 `read`를 부릅니다. 둘 다 구현하지 않으면 서로를 끝없이 부르게 되므로, 트레이트에 붙은 표준 라이브러리 내부 전용 속성 `#[rustc_must_implement_one_of(read_buf, read)]`가 "둘 중 하나는 구현하라"는 E0046으로 막습니다. 이 속성을 우리 트레이트에 붙이면 ``error[E0658]: use of an internal attribute``이고, 함께 나오는 설명은 이 속성이 앞으로도 안정화되지 않을 내부 구현이라고 밝힙니다. 우리가 만드는 트레이트에서 두 기본 메서드가 서로를 부르게 정의하면, 둘 다 재정의하지 않은 구현을 컴파일러가 막아 주지 않는다는 뜻입니다. stable에서 `Read`를 구현할 때는 지금까지처럼 `read`를 구현하면 됩니다.

## 5. 연습 문제

### 연습 1. 기본 메서드 하나 더하기

[`01_minimal_required.rs`](examples/01_minimal_required.rs)의 권장 트레이트(`narrow::Checkpoints`)에 `from` 이상 `to` 미만 구간에 있는 지점 수를 돌려주는 기본 메서드 `count_between(&self, from: u32, to: u32) -> usize`를 더하고, 세 코스에서 `count_between(4000, 16000)`을 출력하세요. 세 구현(`Listed`, `Even`, `Single`)은 고치지 않습니다. 같은 기능을 비권장 트레이트(`wide::Checkpoints`)에 필수 메서드로 더하면 무엇을 고쳐야 하는지도 비교해 보세요.

<details>
<summary>힌트</summary>

본문은 `(0..).map_while(|i| self.at(i)).filter(|p| (from..to).contains(p)).count()`처럼 쓸 수 있습니다. 반열린 범위 `from..to`의 `contains`가 "`from` 이상 `to` 미만"을 그대로 표현합니다. 목록형과 간격형은 5000, 10000, 15000m의 3곳, 어린이 코스는 0곳입니다. 비권장 트레이트에 필수 메서드로 더하면 두 구현이 모두 E0046으로 깨지고, 구현마다 같은 계산을 다시 써야 합니다. 그때 구간의 끝을 포함하는지를 구현마다 다르게 정할 위험도 다시 생깁니다.

</details>

### 연습 2. 재정의 검사 넓히기

[`02_override_default.rs`](examples/02_override_default.rs)의 `mismatches`는 `next_from`만 비교합니다. `count`와 `last`도 비교하도록 검사를 넓히세요. 그다음 `FastMarkers`의 `last`를 일부러 `self.0.n.checked_sub(1).map(|i| self.0.position(i + 1))`로 바꿔 검사가 잡는지 확인하고, 지점이 하나도 없는 코스(`Spacing::new(1000, 1000, 0)`)에서도 두 판이 같은 답을 내는지 확인하세요.

<details>
<summary>힌트</summary>

`count`와 `last`는 위치 인자가 없으므로 한 번씩만 비교하면 됩니다. 일부러 넣은 실수는 `Some(43000)`을 돌려주므로 기본 구현의 `Some(42000)`과 다릅니다. 지점이 없는 코스에서는 기본 구현의 `count`가 `at(0)`이 `None`이라 0, `last`가 `checked_sub`에서 `None`이고, 재정의한 판도 `n`이 0이므로 같은 답을 냅니다. 경계(0개, 1개, 위치가 지점 바로 위)는 재정의에서 실수가 가장 많이 나는 곳이므로 검사 입력에 꼭 넣습니다.

</details>

### 연습 3. 무엇을 필수로 둘까

주차장의 칸 상태를 묻는 트레이트 `ParkingLot`을 설계하세요. 쓰는 쪽에 주고 싶은 메서드는 다음 여섯 개입니다.

- `is_free(&self, slot: usize) -> Option<bool>`: 그 칸이 비었는지. 칸이 없으면 `None`
- `capacity(&self) -> usize`: 칸 수
- `free_count(&self) -> usize`: 빈 칸 수
- `first_free(&self) -> Option<usize>`: 가장 앞의 빈 칸 번호
- `is_full(&self) -> bool`: 빈 칸이 없는지
- `report(&self) -> String`: `"3층 주차장: 12칸 중 4칸 빔"`처럼 주차장 이름을 넣은 문구

어느 것을 필수로, 어느 것을 기본 메서드로 둘지 정하고, `Vec<bool>`로 칸 상태를 가진 구현과 입구 센서가 빈 칸 수를 따로 세어 두는 구현을 만들어 보세요.

<details>
<summary>힌트</summary>

`is_free` 하나로 나머지를 모두 만들 수 있습니다. `capacity`는 `(0..).map_while(|i| self.is_free(i)).count()`, `free_count`는 그중 `true`의 개수, `first_free`는 `position`, `is_full`은 `first_free().is_none()`입니다. `report`는 주차장 이름이 필요하므로 `where Self: fmt::Display`를 붙입니다(2.4절). 센서가 빈 칸 수를 세어 두는 구현은 `free_count`와 `is_full`을 재정의할 이유가 있고, 그렇다면 연습 2처럼 기본 구현과 같은 답을 내는지 검사를 함께 둡니다. 센서의 숫자가 실제 칸 상태와 어긋날 수 있다면, 그 구현은 재정의하지 않는 편이 낫다는 판단도 가능합니다(2.3절의 "따르지 않아도 되는 경우").

</details>

## 6. 참고 자료

- 기준 문서: [Item 13: Use default implementations to minimize required trait methods](https://effective-rust.com/default-impl.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- 표준 라이브러리 문서
  - [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html): Required Methods와 Provided Methods, [`size_hint`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.size_hint)의 구현 노트, [`cloned`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.cloned)의 바운드
  - [`ExactSizeIterator`](https://doc.rust-lang.org/std/iter/trait.ExactSizeIterator.html): `len`을 재정의해도 되는 경우, nightly 전용 `is_empty`(추적 이슈 [#35428](https://github.com/rust-lang/rust/issues/35428))
  - [`PartialEq`](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html): `eq`와 `ne`가 서로 맞아야 한다는 약속과 `ne`를 재정의하지 말라는 안내
  - [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html): `cloned`의 바운드에 쓰인 트레이트
- [The Rust Reference: Inherent implementations](https://doc.rust-lang.org/reference/items/implementations.html#inherent-implementations): 고유 구현. 책이 고유 메서드가 우선한다는 각주에 연결한 절입니다.
- 에러 설명: `rustc --explain E0277`, `rustc --explain E0034`, `rustc --explain E0046`
