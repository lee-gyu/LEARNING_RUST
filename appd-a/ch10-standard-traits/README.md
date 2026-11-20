# ch10. 표준 트레이트를 알고 골라 구현하기

`#[derive(Debug, Clone, PartialEq)]` 한 줄은 쓰기 쉽지만, 공개 타입에 붙이는 트레이트 하나하나는 사용자와 맺는 약속입니다. `Copy`를 붙이면 "이 값은 몰래 복사돼도 괜찮다"고, `Eq`와 `Hash`를 붙이면 "같은 값은 같은 해시를 낸다"고, `Default`를 붙이면 "아무것도 지정하지 않은 값에 뜻이 있다"고 약속합니다. 반대로 빠뜨린 트레이트는 사용자가 대신 채울 수 없습니다. 트레이트도 타입도 사용자의 것이 아니어서 고아 규칙에 걸리기 때문입니다. 이 챕터는 트레이트를 **구현하는 방법**이 아니라, 공개 타입을 내놓을 때 **무엇을 구현하고 무엇을 구현하지 않을지** 판단하는 기준을 다룹니다. 앞 챕터들이 이 챕터로 미룬 숙제도 함께 풉니다. [ch2](../ch2-express-common-behavior/README.md)의 "`Eq`·`Send`·`Sync` 같은 마커 트레이트를 언제 구현하나", [ch6](../ch6-newtype-pattern/README.md)의 "뉴타입에 무엇을 derive하나", [ch7](../ch7-builders/README.md)의 "공개 타입에 `Default`를 구현하나"입니다.

- 원제: [Item 10: Familiarize yourself with standard traits](https://effective-rust.com/std-traits.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch10-standard-traits`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

구현 문법은 다시 설명하지 않습니다. 다음 본편 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch5](../../part0-getting-started/ch5-structs/README.md) 2.6절, [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.7~2.8절: `derive`, 고아 규칙
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.9절: 연산자 오버로딩
- [Part 1 ch3](../../part1-standard-lib/ch3-strings-fmt/README.md) 2.10~2.11절: `Display`와 `Debug` 직접 구현
- [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 2.8~2.13절: 연산자, 비교, 해시, `Default`, `Clone`/`Copy`의 구현법과 규칙

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 표준 트레이트마다 "구현하면 사용자가 무엇을 할 수 있게 되는가, 무엇을 약속하게 되는가, derive로 충분한가, 구현하면 안 되는 경우는 언제인가"를 설명하고, 공개 타입 하나에 체크리스트로 적용할 수 있다.
2. derive가 만드는 구현이 타입의 뜻과 어긋나는 경우(값의 뜻과 다른 필드 비교, 필드 순서에 기댄 순서, 제네릭 타입에 붙는 바운드)를 알아보고, 표현을 정규화하거나 손으로 구현해 바로잡을 수 있다.
3. `Copy`를 붙이면 안 되는 타입을 판단하고, `PartialEq`를 손으로 쓸 때 `Hash`를 같은 기준으로 맞추지 않으면 `HashSet`이 넣은 값을 못 찾는 이유를 설명할 수 있다.
4. `f64` 필드가 있는 타입에서 `NaN`이 만드는 문제를 설명하고, `total_cmp`와 정수 표현으로 정렬과 키를 해결할 수 있다.
5. `Default`, `Display`, 연산자를 구현할지 판단하는 기준을 말하고, 구현한 연산자끼리 어긋나지 않게 작성할 수 있다.

## 2. 핵심 개념

2.1절에서 체크리스트 전체를 보고, 2.2~2.8절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 2.9절은 체크리스트를 공개 타입 하나에 적용하며 "넉넉하게 구현한다"는 지침의 근거를 봅니다. 코드는 핵심만 싣고, 두 형태를 함께 실행해 비교하는 전체 코드는 `examples/`에 있습니다.

### 2.1 지도: 공개 타입의 트레이트 체크리스트

| 트레이트 | 구현하면 사용자가 할 수 있는 것 | 구현하면 약속하는 것 | derive로 충분한가 | 구현하지 않는 경우 | 절 |
| --- | --- | --- | --- | --- | --- |
| `Debug` | `{:?}`, `assert_eq!`의 실패 메시지, 자기 타입에 `derive(Debug)` | 거의 없음(형식은 안정적이지 않음) | 대부분. 비밀이 든 타입은 손으로 | 거의 없음 | 2.7 |
| `Clone` | `.clone()`, 자기 타입에 `derive(Clone)` | 복제본을 따로 써도 뜻이 맞음 | 제네릭이면 바운드를 확인 | 유일해야 하는 값(승인, 연결, 비밀 키) | 2.2, 2.3 |
| `Copy` | 대입·전달 뒤에도 원본을 씀 | 비트 사본이 원본과 똑같이 쓰여도 됨. 앞으로도 그럴 것 | 예(메서드 없음) | 유일한 값, 제자리에서 고쳐 쓰는 상태, 필드가 늘 수 있는 타입 | 2.3 |
| `PartialEq` | `==`, `contains`, `assert_eq!` | 대칭, 추이 | 모든 필드가 "같음"에 그대로 참여할 때 | "같다"의 기준이 하나로 정해지지 않을 때 | 2.2, 2.4 |
| `Eq` | `Hash`와 함께 `HashMap`/`HashSet`의 키, `Ord`의 전제 | 반사(`a == a`) | 예(메서드 없음) | `f64` 같은 필드가 있을 때(불가) | 2.5 |
| `PartialOrd`, `Ord` | `<`, `sort`, `max`, `BTreeMap`/`BTreeSet` | 전순서, `==`와 일치 | 필드 선언 순서가 곧 뜻하는 순서일 때 | 자연스러운 순서가 하나로 정해지지 않을 때 | 2.2 |
| `Hash` | `HashMap`/`HashSet`의 키 | `a == b`이면 `hash(a) == hash(b)` | `PartialEq`도 derive했을 때 | `PartialEq`를 손으로 썼다면 같은 기준으로 손으로 | 2.4 |
| `Default` | `T::default()`, `..Default::default()`, `mem::take`, `or_default`, 자기 타입에 `derive(Default)` | 기본값에 뜻이 있음 | 필드 타입의 기본값이 이 타입에도 맞을 때 | 불변식이 있거나 기본값을 지어내야 할 때 | 2.6 |
| `Display` | `{}`, `to_string()` | 사용자에게 보여 줄 표현은 이것 하나 | 불가 | 표현이 여럿일 때 | 2.7 |
| 연산자(`Add` 등) | `+`, `+=` 등 | 기호의 통념(순서, `+`와 `+=`의 일치) | 불가 | 기호만으로 뜻을 짐작할 수 없을 때 | 2.8 |
| `Send`, `Sync` | 스레드로 넘기기, 공유하기 | (자동 트레이트) | 구현하지 않음. 필드가 정함 | 직접 구현은 원시 포인터를 감싼 타입뿐 | 2.9 |

각 트레이트의 구현 문법과 트레이트끼리 지켜야 할 성질(동치 관계, 전순서, `Hash`와 `Eq`의 관계)은 Part 1 ch6이 다뤘습니다. 이 표의 가운데 세 열이 판단의 재료입니다. 사용자에게 열어 주는 것이 많을수록 넉넉하게 구현할 이유가 되고, 약속을 지킬 수 없거나 derive가 뜻과 다른 구현을 만든다면 구현하지 않거나 손으로 씁니다.

**다른 챕터에서 다루는 표준 트레이트.** 변환(`From`/`TryFrom`)은 [ch5](../ch5-type-conversions/README.md), `Error`는 [ch4](../ch4-idiomatic-errors/README.md), `Fn`/`FnMut`/`FnOnce`는 [ch2](../ch2-express-common-behavior/README.md), 포인터 트레이트(`Deref`, `AsRef`, `Borrow`)는 [ch8](../ch8-references-pointers/README.md), `Drop`은 [`ch11-drop-raii`](../ch11-drop-raii/README.md), 반복자 트레이트는 [`ch9-iterator-transforms`](../ch9-iterator-transforms/README.md)에서 다룹니다. 반복자 쪽에서 내 타입에 구현할지 따질 것은 하나입니다. 내 타입이 원소를 담는 컬렉션이라면 `FromIterator`(`collect()`로 만들기), `Extend`(`extend()`로 덧붙이기), `IntoIterator`(`for` 문으로 돌기)를 구현해 표준 컬렉션처럼 쓰게 해 줍니다. 컬렉션이 아닌 타입에는 붙이지 않습니다.

### 2.2 지침 1: derive가 만드는 구현이 타입의 뜻과 맞는지 확인한다

**문제 상황.** 분수를 나타내는 공개 타입 `Ratio { num, den }`을 만듭니다. 사용자가 분수를 비교하고, 정렬하고, 집합에 넣을 수 있게 비교 트레이트를 모두 열어 주려 합니다.

**비권장 코드.** 필드를 그대로 두고 모두 derive합니다.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ratio { pub num: i32, pub den: i32 }
```

derive는 "모든 필드를, 선언 순서대로" 비교하고 해시합니다(Part 1 ch6 2.9절). [`01_derive_matches_meaning.rs`](examples/01_derive_matches_meaning.rs)에서 이 규칙은 분수의 뜻과 두 군데에서 어긋납니다. `1/2 == 2/4`가 `false`라서 `HashSet`에 넣은 네 값이 모두 다른 값으로 남고, 순서는 분자를 먼저 비교하는 사전식이라 정렬 결과가 `[1/2, 1/3, 2/3, 2/4]`입니다. 컴파일러와 clippy는 아무 말도 하지 않습니다. derive가 만든 구현은 규칙대로 정확하고, 틀린 것은 그 규칙이 타입의 뜻과 다르다는 점이기 때문입니다.

**권장 코드.** 두 단계로 고칩니다.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] // 정규화 덕분에 derive가 맞음
pub struct Ratio { num: i32, den: i32 }             // 필드는 비공개: new만이 만드는 길

impl Ratio {
    pub fn new(num: i32, den: i32) -> Option<Ratio> { /* 기약분수로, 분모는 양수로 */ }
}

impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> Ordering {
        (i64::from(self.num) * i64::from(other.den)).cmp(&(i64::from(other.num) * i64::from(self.den)))
    }
}
impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
```

첫째, 생성자에서 표현을 하나로 맞춥니다. 모든 분수가 기약분수로 저장되면 "필드가 모두 같다"와 "같은 수다"가 같은 말이 되므로, derive한 `PartialEq`·`Eq`·`Hash`가 그대로 맞습니다. [ch8](../ch8-references-pointers/README.md) 2.5절이 공항 코드를 대문자로 정규화한 것과 같은 방법입니다. 둘째, 정규화해도 맞지 않는 순서는 손으로 씁니다. 분모가 양수인 두 분수는 교차해서 곱한 값을 비교하면 됩니다. 이때 `cmp`가 `Equal`인 경우와 `==`가 `true`인 경우가 정확히 같아야 하는데, 기약분수라서 그렇습니다. 결과는 `[1/3, 1/2, 2/3]`, 서로 다른 값 3개입니다. `PartialOrd`는 `Some(self.cmp(other))`로 손으로 씁니다. 여기서 `derive(PartialOrd)`를 남겨 두면 `<`는 사전식, `cmp`는 수의 크기로 비교해 두 연산이 서로 다른 답을 내는데, clippy가 ``you are implementing `Ord` explicitly but have derived `PartialOrd` ``(`derive_ord_xor_partial_ord`, 기본 deny)로 막습니다.

**필드 순서도 약속입니다.** `PartialOrd`와 `Ord`를 derive하면 필드(열거형이면 배리언트)의 선언 순서가 곧 정렬 순서가 됩니다. 나중에 누군가 필드를 보기 좋게 재배치하면 컴파일 에러 없이 사용자의 정렬 결과가 바뀝니다. derive한 순서가 맞는 타입이라면(2.9절의 `YearMonth`처럼) 선언 옆에 "순서를 바꾸지 말 것"을 적어 둡니다. 공개 동작이 바뀌는 문제는 [ch21](../ch21-semver/README.md)에서 다룹니다.

**제네릭 타입에는 바운드가 붙습니다.** 같은 예제의 두 번째 구역은 `Rc`로 값을 함께 쓰는 핸들 `Shared<T>(Rc<T>)`입니다. `Clone` 문서의 Derivable 절이 밝히듯 `derive(Clone)`는 타입 매개변수에 바운드를 붙여 `impl<T: Clone> Clone for Shared<T>`를 만듭니다. 복제하는 것은 `Rc`뿐이라 `T`가 `Clone`일 필요가 없는데도, 하나뿐이어야 해서 `Clone`을 빼 둔 `Connection`을 담으면 `.clone()`이 사라집니다(4.1절, E0599). 바운드 없이 손으로 씁니다.

```rust
impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self { Shared(Rc::clone(&self.0)) }
}
```

`Default`(`Option<T>`나 `Vec<T>` 필드), `PartialEq`, `Debug`도 같은 방식으로 바운드를 붙입니다. 제네릭 공개 타입이라면 derive한 트레이트마다 "이 바운드가 정말 필요한가"를 확인합니다.

**따르지 않아도 되는 경우.**

- 모든 필드가 "같음"과 "순서"에 그대로 참여하는 데이터 묶음이라면 derive가 정답입니다. 손으로 쓴 구현은 필드를 추가할 때 함께 고치는 것을 잊기 쉬워 오히려 위험합니다.
- 자연스러운 순서가 하나로 정해지지 않는 타입(이름순도, 날짜순도 말이 되는 할 일 목록 등)이라면 `Ord`를 derive하지도 손으로 쓰지도 않습니다. 사용자가 `sort_by_key`로 기준을 고르게 둡니다(Part 1 ch6 2.10절).

### 2.3 지침 2: `Copy`는 값처럼 행동하는 작은 타입에만 붙인다

**문제 상황.** 송금 승인 타입을 만듭니다. 승인 하나로는 한 번만 송금할 수 있어야 해서, 송금 함수는 승인을 값으로 받아 소비합니다. 승인의 필드는 금액 하나라 `Copy`를 붙일 수 있습니다.

**비권장 코드.**

```rust
#[derive(Debug, Clone, Copy)]
struct CopyApproval { amount: i64 }

fn transfer_copy(account: &mut Account, approval: CopyApproval) { account.balance -= approval.amount; }

transfer_copy(&mut account, approval);
transfer_copy(&mut account, approval); // 실수로 한 번 더. 컴파일됨
```

[`02_copy_promise.rs`](examples/02_copy_promise.rs)에서 30000원짜리 승인 한 건으로 10만 원이 든 계좌의 잔액이 40000원이 됩니다. 값으로 받아 소비하는 시그니처가 "한 번만"을 보장하는 것은 그 타입이 `Copy`가 아닐 때뿐입니다. `Copy`를 붙이면 넘길 때마다 비트 사본이 만들어지고 원본은 그대로 남습니다.

**권장 코드.** `Copy`도 `Clone`도 붙이지 않습니다. 두 번째 호출은 E0382(``use of moved value: `approval` ``)가 됩니다. 이 에러가 바로 원하던 보호입니다. 그런데 1.100.0의 E0382에는 ``note: consider changing this parameter type in function `transfer` to borrow instead if owning the value isn't necessary``와 ``note: if `Approval` implemented `Clone`, you could clone the value``가 붙습니다(같은 모양의 코드로 확인). 둘 다 일회성을 깨는 제안이라 따르지 않습니다. `Clone`까지 빼는 이유도 여기에 있습니다. `Clone`이 있으면 `.clone()` 한 번으로 승인이 둘이 됩니다.

같은 예제의 두 번째 구역은 다른 종류의 함정입니다. 투표함 안의 집계 `Tally`가 `Copy`이면 `let mut tally = ballot_box.tally; tally.yes += 1;`은 집계를 꺼내 고치는 것이 아니라 사본을 고치는 것입니다. 찬성 2표를 넣어도 집계는 0표입니다. 고친 사본을 아무도 읽지 않으면 rustc의 `unused_variables`와 `unused_assignments` 경고가 나지만, 예제처럼 사본으로 합계를 계산해 돌려주면 경고도 없습니다. `Tally`가 `Copy`가 아니면 같은 줄이 ``error[E0507]: cannot move out of `ballot_box.tally` which is behind a mutable reference``가 되고 ``help: consider borrowing here``가 붙어, `&mut ballot_box.tally`로 고치게 됩니다. 함께 나오는 ``help: consider cloning the value if the performance cost is acceptable``은 다시 사본을 고치는 길이므로 따르지 않습니다.

**`Copy`는 떼기 어렵습니다.** `Copy` 문서는 "구현할 수 있으면 보통 구현하라"고 하면서도, `Copy`는 공개 API의 일부라서 나중에 `Copy`가 아니게 될 수 있는 타입이면 처음부터 빼 두는 편이 낫다고 덧붙입니다. 지금은 숫자 필드뿐인 옵션 타입에 다음 버전에서 `String` 필드를 더하면 `Copy`를 떼야 하고, 값을 넘긴 뒤 다시 쓰던 사용자 코드가 모두 E0382로 깨집니다. 컴파일러가 아예 막는 경우도 있습니다. `Copy`가 아닌 필드가 있으면 E0204, `Drop`을 구현했으면 E0184입니다(Part 1 ch6 4.4~4.5절). 이 둘은 자원을 소유하는 타입을 컴파일러가 걸러 주는 것이고, 위의 승인과 집계처럼 필드는 모두 `Copy`인데 뜻이 맞지 않는 경우는 사람이 판단해야 합니다. 책은 큰 타입도 다시 생각해 보라고 권합니다. `Copy`인 큰 배열 필드는 대입할 때마다 코드에 보이지 않는 복사를 일으킵니다.

rustc의 `missing_copy_implementations` 린트(기본 allow)를 켜면 `Copy`가 될 수 있는 공개 타입에 ``type could implement `Copy`; consider adding `impl Copy` ``를 경고합니다. 이 린트는 필드만 보므로, 공개 타입으로 바꾼 `Approval`에도 같은 경고를 냈습니다. 경고를 받으면 이 절의 기준으로 판단하고, 붙이지 않기로 했다면 `#[expect(missing_copy_implementations, reason = "...")]`로 이유를 남깁니다.

**따르지 않아도 되는 경우(`Copy`가 맞는 경우).** 복사본이 원본과 따로 놀아도 뜻이 같은 작은 값입니다. 좌표, 연월(2.9절), 데이터 없는 열거형, ID 같은 타입은 구성이 개념으로 정해져 있어 필드가 늘 일이 적고, `Copy`를 붙이면 사용자가 `&`와 `.clone()`을 신경 쓰지 않아도 됩니다. 이런 타입에는 붙입니다.

### 2.4 지침 3: `PartialEq`를 손으로 썼다면 `Hash`도 같은 기준으로 쓴다

**문제 상황.** 카페 정보 `Cafe { id, name, reviews }`를 즐겨찾기 `HashSet`에 넣습니다. 같은 카페인지는 `id`로만 판단해야 합니다. 리뷰 수는 계속 바뀌는 상태이기 때문입니다. 그래서 `PartialEq`는 손으로 쓰고, `Hash`는 derive로 두었습니다.

**비권장 코드.**

```rust
#[derive(Debug, Clone, Hash)] // 모든 필드를 해시
pub struct Cafe { pub id: u32, pub name: &'static str, pub reviews: u32 }

impl PartialEq for Cafe {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl Eq for Cafe {}
```

이 조합은 clippy가 ``you are deriving `Hash` but have implemented `PartialEq` explicitly``(`derived_hash_with_manual_eq`, 기본 deny)로 막으므로, [`03_eq_hash_agree.rs`](examples/03_eq_hash_agree.rs)는 `#[expect]`를 붙여 일부러 통과시켰습니다. 리뷰가 128개일 때 저장한 카페를, 리뷰가 131개로 늘어난 뒤 다시 불러와 찾습니다.

```text
  saved == fresh? true
  해시가 같은가? false
  즐겨찾기에 있나? false
  없다길래 넣었더니 즐겨찾기 2개
```

`==`는 같다고 하는데 집합은 없다고 합니다. `HashSet`은 해시로 찾아갈 자리를 정하고 그 자리에서 `==`로 확인하므로, 해시가 다르면 엉뚱한 자리를 뒤집니다. 마지막 줄은 이 모순이 실제 버그가 되는 모습입니다. "없으면 추가"하는 코드가 같은 카페를 두 번 넣었습니다.

예제는 결과가 실행마다 같도록 시드가 고정된 해셔(`BuildHasherDefault<DefaultHasher>`)로 집합을 만들었습니다. 기본 해셔(`RandomState`)는 집합마다 시드가 달라서, 해시가 다른 두 값이 우연히 같은 자리 후보에 놓이면 찾기도 합니다. 같은 조회를 `HashSet::new()`로 2만 번 시험했더니 세 번의 실행에서 136번, 159번, 154번 찾았고 나머지는 찾지 못했습니다. 결과가 정해지지 않는다는 것, 표준 라이브러리 문서가 이것을 논리 오류라고 부르는 이유입니다.

**권장 코드.** `eq`가 보는 것만 해셔에 넣습니다.

```rust
impl Hash for Cafe {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state); }
}
```

이제 해시가 같고, 조회는 `true`, 다시 넣으면 `insert`가 `false`를 돌려주어 즐겨찾기는 1개로 남습니다. 규칙은 Part 1 ch6 2.11절과 같습니다. `Hash`는 `eq`가 보는 정보의 일부만 넣어도 되지만, `eq`가 무시하는 정보를 넣으면 안 됩니다. [ch8](../ch8-references-pointers/README.md) 2.5절은 같은 약속이 `Borrow`로 빌려준 형태와 소유한 형태 사이에서 깨지는 경우를 다뤘고, 이 절은 한 타입 안에서 손으로 쓴 `PartialEq`와 derive한 `Hash`가 어긋나는 경우입니다.

**더 나은 설계를 먼저 생각합니다.** "같은 카페"의 기준이 ID라면, 동일성을 ID 타입에 맡기는 방법이 있습니다. `CafeId(u32)`에 비교·해시 트레이트를 모두 derive하고 `HashMap<CafeId, Cafe>`로 관리하면, `Cafe`는 `PartialEq`를 손으로 쓸 필요가 없고 어긋날 구현도 없습니다(연습 2).

**따르지 않아도 되는 경우.** `PartialEq`를 derive했다면 `Hash`도 derive하면 됩니다. 둘 다 같은 필드를 같은 방식으로 다루므로 맞춰 줄 것이 없습니다.

### 2.5 지침 4: `f64` 필드가 있으면 `PartialEq`까지만, 정렬은 `total_cmp`, 키는 정수 표현으로

**문제 상황.** 지도 앱이 주소를 좌표로 바꿔 핀 `Pin { lat: f64, lng: f64 }`를 찍습니다. 주소 변환에 실패한 핀은 위도가 `NaN`입니다. 핀을 목록에서 찾고, 위도로 정렬하고, 같은 좌표를 걸러 내야 합니다.

`Pin`에는 `PartialEq`와 `PartialOrd`만 derive할 수 있습니다. `NaN`은 자기 자신과도 같지 않아서 `f64`가 `Eq`(반사)를 약속할 수 없고, 그래서 `Hash`와 `Ord`도 구현하지 않기 때문입니다(Part 1 ch6 2.9절). `derive(Eq, Hash)`를 붙이면 필드마다 E0277입니다(4.2절). [ch6](../ch6-newtype-pattern/README.md)의 `f64` 뉴타입이 `Eq`·`Ord`·`Hash`를 derive하지 못한 것도 같은 이유입니다.

**비권장 코드.** `PartialEq`만 있는 값을 `==`로 찾고, 정렬에서는 `NaN`과의 비교를 `Equal`로 덮습니다.

```rust
pins.contains(&failed)                                             // ==로 찾음
pins.sort_by(|a, b| a.lat.partial_cmp(&b.lat).unwrap_or(Ordering::Equal))
```

[`04_float_fields.rs`](examples/04_float_fields.rs)에서 목록에 분명히 들어 있는 `failed`를 `contains`가 찾지 못하고(`false`), 남에서 북으로 정렬한 결과가 `["서울시청", "주소 변환 실패", "부산역", "대전역", "강릉역"]`입니다. 서울이 맨 앞에 그대로 남았습니다. `NaN`을 모든 값과 `Equal`이라고 하면 "서울 = NaN = 부산"인데 "서울 > 부산"이 되어 비교 함수가 전순서가 아니게 됩니다. `sort_by` 문서는 이때 결과 순서가 정해지지 않고 패닉할 수도 있다고 적고 있습니다(이 입력은 이 툴체인에서 패닉하지 않음을 확인했습니다). `partial_cmp(..).unwrap()`으로 쓰면 `NaN`을 만나는 순간 패닉합니다.

**권장 코드.**

```rust
pins.sort_by(|a, b| a.lat.total_cmp(&b.lat)); // 전순서. 양의 NaN은 맨 뒤

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PinKey { lat_e6: i32, lng_e6: i32 }    // 1e-6도 단위 정수

impl PinKey {
    fn from_pin(pin: Pin) -> Option<PinKey> { /* 범위 밖과 NaN은 None */ }
}
```

정렬에는 `f64::total_cmp`를 씁니다. IEEE 754의 전순서를 따르는 비교라서 `NaN`도 자리가 정해지고(`f64::NAN`은 모든 수보다 뒤), 결과는 `["부산역", "대전역", "서울시청", "강릉역", "주소 변환 실패"]`입니다. 다만 `total_cmp`는 `-0.0`을 `0.0`보다 작다고 보므로 `==`와 판단이 다른 경우가 있다는 점을 알아 둡니다. 집합이나 맵의 키가 필요하면 정수 표현으로 바꾼 타입을 따로 둡니다. 정수 필드뿐인 `PinKey`에는 모든 비교·해시 트레이트를 derive할 수 있고, `from_pin`이 경계에서 `NaN`과 범위 밖 좌표를 거절합니다. 예제는 같은 좌표가 두 번 들어온 핀 6개를 서로 다른 좌표 4개와 거절 1개로 나눕니다.

근본적으로는 "실패"를 `NaN`으로 표현하지 않는 것이 낫습니다. [ch1](../ch1-express-data-structures/README.md) 2.5절의 센티널 값과 같은 문제이므로, 주소 변환 결과를 `Option<Pin>`으로 돌려주면 `Pin` 안에는 처음부터 `NaN`이 없습니다.

**따르지 않아도 되는 경우.** 생성자에서 `NaN`을 막아 둔 타입이라면 `==`와 `partial_cmp`도 안전합니다. 그런 타입에 `Eq`·`Ord`·`Hash`까지 열고 싶다면 `total_cmp`로 `Ord`를, 비트 표현(`to_bits`)으로 `Hash`를 손으로 구현하는 뉴타입을 만들 수 있습니다. 세 구현이 서로 맞아야 하므로 손이 많이 갑니다(연습 4).

### 2.6 지침 5: `Default`는 뜻이 분명한 기본값이 있을 때만 구현한다

[ch7](../ch7-builders/README.md) 2.3절은 `Default`와 `..Default::default()`를 쓰려면 모든 필드에 뜻이 있는 기본값이 있어야 한다고 했고, Part 1 ch6 2.12절은 기본값이 0이 아닐 때 손으로 구현하는 법을 다뤘습니다. 이 절은 그 앞의 질문, 공개 타입에 `Default`를 구현할지 자체를 다룹니다.

**문제 상황.** 인쇄 라이브러리에 공개 타입이 둘 있습니다. 문서를 쌓아 두는 대기열 `JobQueue`와, 1~99만 허용하는 인쇄 부수 `Copies`입니다.

**비권장 코드 1.** 빈 대기열을 만드는 `new()`만 두고 `Default`는 구현하지 않습니다. clippy가 ``you should consider adding a `Default` implementation for `JobQueue` ``(`new_without_default`, 기본 warn)로 알려 주는 모양입니다. [`05_default_meaningful.rs`](examples/05_default_meaningful.rs)의 사용자 코드는 층마다 대기열을 두는 `HashMap`에서 `or_default()` 대신 `or_insert_with(JobQueue::new)`를, 대기열을 비우며 꺼낼 때 `mem::take` 대신 `mem::replace(&mut q, JobQueue::new())`를 씁니다. 사용자가 `JobQueue` 필드를 가진 자기 타입에 `derive(Default)`를 붙이면 ``the trait bound `JobQueue: Default` is not satisfied``(E0277)입니다.

**권장 코드 1.** 빈 대기열은 누가 봐도 같은 뜻이므로 `derive(Default)`를 붙이고, `new()`는 `Self::default()`를 돌려줍니다. 결과는 같지만 사용자는 `or_default`, `unwrap_or_default`, `mem::take`를 그대로 쓰고, 자기 타입 `Office`에도 `derive(Default)`를 붙입니다. 열거형은 기본 배리언트에 `#[default]`를 붙여 derive합니다(예제의 `Orientation`). 빠뜨리면 E0665입니다(4.3절).

**비권장 코드 2.** 불변식이 있는 `Copies`에 `derive(Default)`를 붙입니다.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Copies(u8); // new는 1~99만 받음
```

`Copies::new(0)`은 `None`인데 `Copies::default()`는 0부를 만듭니다. derive한 `Default`는 필드마다 `default()`를 부를 뿐 생성자를 거치지 않으므로, [ch6](../ch6-newtype-pattern/README.md) 2.3절에서 말한 대로 검증 뉴타입에 무효한 값을 만드는 뒷문이 됩니다.

**권장 코드 2.** `Default`를 두지 않습니다. 자주 쓰는 값이 있다면 이름이 그 뜻을 말하는 상수로 줍니다(`Copies::ONE`). "기본값"은 상황에 따라 다르지만 "1부"는 누구에게나 같은 뜻입니다.

**판단 기준.** "아무것도 지정하지 않은 값"이 그 타입에서 뜻이 분명하면 구현합니다. 빈 컬렉션, 0인 카운터, "필터 없음", 세로 방향처럼요. 다음 경우에는 구현하지 않습니다.

- 불변식이 있어 기본값이 무효하거나(`Copies`), 기본값을 지어내야 하는 타입(연결 핸들, 사용자 ID, 2.9절의 `YearMonth`)
- 기본값이 쓰는 곳마다 달라 라이브러리가 하나로 정할 수 없는 타입

**따르지 않아도 되는 경우.** 크레이트 안에서만 쓰는 비공개 타입이라면 필요해질 때 붙이면 됩니다. 테스트에서 값을 쉽게 만들려고 `Default`를 쓰는 것도 괜찮습니다. 다만 공개 타입에 테스트 편의만을 위한 `Default`를 붙이면 그 약속이 사용자에게도 열린다는 점을 기억합니다.

### 2.7 지침 6: `Debug`는 거의 항상, `Display`는 표현이 하나로 분명할 때만

**문제 상황 1.** 결제 기록을 `{:?}`로 로그에 남깁니다. 결제에는 카드 번호가 들어 있습니다.

**비권장 코드 1.** 카드 번호를 `String` 필드로 두고 결제 타입에 `derive(Debug)`를 붙입니다. [`06_debug_display.rs`](examples/06_debug_display.rs)의 로그는 `RawPayment { order: 1042, card: "1234-5678-9012-3456" }`입니다.

**권장 코드 1.** 비밀을 담는 타입을 따로 만들고, **그 타입의** `Debug`에서 가립니다.

```rust
pub struct CardNumber(String);

impl fmt::Debug for CardNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CardNumber").field(&format_args!("****-{}", self.last4())).finish()
    }
}

#[derive(Debug)]
pub struct Payment { pub order: u32, pub card: CardNumber } // 그대로 derive
```

`derive(Debug)`는 필드마다 그 필드 타입의 `Debug`를 부르므로, `Payment`의 로그는 `Payment { order: 1042, card: CardNumber(****-3456) }`가 됩니다. [Part 1 ch3](../../part1-standard-lib/ch3-strings-fmt/README.md) 2.11절은 비밀번호를 담은 구조체의 `Debug`를 손으로 써서 그 필드를 뺐습니다. 비밀을 자기 타입으로 감싸 그 타입에서 가리면, 그 값을 담는 모든 타입이 derive만으로 안전해진다는 점이 다릅니다. `CardNumber`에는 `Clone`도 붙이지 않았습니다. 비밀이 복제되어 퍼질 길을 줄이려는 것으로, 책도 암호 키처럼 복제를 제한해야 하는 값에는 `Clone`을 구현하지 말라고 권합니다.

`Debug`는 개발자를 위한 표현이라 형식이 바뀌어도 되고, 실제로 `Debug` 문서는 derive한 형식이 앞으로의 Rust 버전에서 바뀔 수 있다고 밝힙니다. 그래서 `Debug` 출력을 파싱하거나 화면에 그대로 보여 주면 안 되고, 그 대신 거의 모든 공개 타입에 부담 없이 구현할 수 있습니다. 빠뜨린 `Debug`를 찾는 린트는 2.9절에서 봅니다.

**문제 상황 2.** 파일 크기 타입 `ByteSize(u64)`를 사용자에게 보여 줍니다. 1572864바이트는 1000 단위로 1.6 MB, 1024 단위로 1.5 MiB입니다. 둘 다 표준 표기이고 쓰는 곳이 다릅니다.

**비권장 코드 2.** `Display`를 구현하면서 1024 단위를 골라 넣습니다. `{}`는 언제나 `1.5 MiB`이고, 1000 단위가 필요한 사용자는 고를 방법이 없습니다. 사용자가 이 타입에 자기 `Display`를 구현하는 것은 고아 규칙에 막히므로 뉴타입으로 감싸야 합니다([ch6](../ch6-newtype-pattern/README.md) 2.5절).

**권장 코드 2.** `Display`를 구현하지 않고, 표현마다 이름 붙은 메서드가 `Display`를 구현한 도우미를 돌려줍니다.

```rust
impl ByteSize {
    pub fn si(self) -> Si { Si(self.0) }    // 1000 단위
    pub fn iec(self) -> Iec { Iec(self.0) } // 1024 단위
}
impl fmt::Display for Si { /* "1.6 MB" */ }
impl fmt::Display for Iec { /* "1.5 MiB" */ }
```

표준 라이브러리의 `Path::display()`와 같은 모양입니다. 반대로 표현이 하나로 분명하면 `Display`를 구현합니다. 분수 `1/2`(예제 01), 연월 `2026-11`(예제 08)이 그렇습니다. `Display`를 구현하면 `to_string()`이 포괄 구현으로 따라오므로, `to_string()`을 고유 메서드로 따로 만들지 않습니다. 그렇게 하면 clippy가 ``implementation of inherent method `to_string(&self) -> String` for type ..``(`inherent_to_string`, 기본 warn)로 `Display`를 구현하라고 권합니다.

**따르지 않아도 되는 경우.** 에러 타입은 표현이 여럿이라도 `Display`가 필요합니다. `Error` 트레이트가 `Display`를 요구하기 때문입니다([ch4](../ch4-idiomatic-errors/README.md)). 이때는 사용자에게 보여 줄 메시지 하나를 정해 `Display`로 둡니다.

### 2.8 지침 7: 연산자는 기호만 보고 결과를 짐작할 수 있을 때만 오버로딩한다

[ch6](../ch6-newtype-pattern/README.md) 2.4절은 단위 뉴타입에 "뜻이 있는 연산만" 여는 기준을 다뤘습니다. 이 절은 연산자 기호에 따라붙는 통념, 그리고 함께 구현한 연산자끼리의 일관성을 봅니다.

**문제 상황 1.** 글자 스타일 `Style { bold, color }`를 병합합니다. 오른쪽 스타일의 값이 왼쪽을 덮어씁니다.

**비권장 코드 1.** 병합을 `+`로 구현합니다.

```rust
impl Add for Style {
    type Output = Style;
    fn add(self, rhs: Style) -> Style {
        Style { bold: rhs.bold.or(self.bold), color: rhs.color.or(self.color) }
    }
}
```

[`07_operator_conventions.rs`](examples/07_operator_conventions.rs)에서 `theme + warning`은 굵은 빨강, `warning + theme`은 보통 검정입니다. 덧셈은 순서를 바꿔도 결과가 같다는 통념이 있어서, 두 번째 줄을 쓴 사람은 경고 문구가 검정 보통 글씨로 나올 줄 몰랐습니다. 기호만 보고는 어느 쪽이 이기는지 알 수 없습니다.

**권장 코드 1.** 방향을 이름에 담은 메서드 `theme.overridden_by(warning)`로 씁니다. 메서드 이름을 `add`로 지으면 clippy가 ``method `add` can be confused for the standard trait method `std::ops::Add::add` ``(`should_implement_trait`, 기본 warn)로 `Add`를 구현하라고 권하므로, 연산자로 오해받지 않는 이름을 고릅니다.

**문제 상황 2.** 0~100 사이의 볼륨에 `+`를 구현했습니다. 올릴 때 100에서 멈추는 포화 덧셈이라 표준 라이브러리의 `Saturating<u8>`과 같은 뜻이고, 기호의 통념에도 맞습니다. 나중에 `+=`를 따로 구현했습니다.

**비권장 코드 2.** `+=`를 `+`와 따로 쓰면서 상한을 잊었습니다. 예제에서 `LooseVolume(90) + 30`은 100인데 `+= 30`은 120입니다. 사용자는 `a += b`가 `a = a + b`와 같다고 믿고 씁니다.

**권장 코드 2.** `+=`는 `+`를 불러서 씁니다. 규칙이 한 곳에만 있으므로 둘이 어긋날 수 없습니다.

```rust
impl AddAssign<u8> for Volume {
    fn add_assign(&mut self, rhs: u8) { *self = *self + rhs; }
}
```

`Saturating(250u8) + Saturating(10)`과 `+= Saturating(10)`이 모두 255인 것도 같은 원칙입니다. 책은 같은 원칙을 `Sub` 예로 듭니다. `Add`와 `Neg`를 구현했다면 `Sub`도 구현하고, `x - y`가 `x + (-y)`와 같아야 합니다. clippy는 구현 안에서 다른 연산자를 쓴 실수 일부를 잡습니다. `Add` 구현 안의 `-`는 `suspicious_arithmetic_impl`, `AddAssign` 구현 안의 `-=`는 `suspicious_op_assign_impl`(둘 다 기본 warn)입니다. 연산자 트레이트가 피연산자를 값으로 받아 `Copy`가 아닌 타입을 소비하는 문제와 참조에 대한 구현은 Part 1 ch6 2.8절에 있습니다.

**따르지 않아도 되는 경우.** 기호의 통념이 분야마다 정해져 있으면 그 통념을 따릅니다. `String + &str`(이어 붙이기)은 순서를 바꾸면 결과가 다르지만 누구나 아는 뜻이고, 행렬 곱 `*`도 교환 법칙이 없지만 수학의 표기 그대로입니다. 집합의 `&BTreeSet - &BTreeSet`은 차집합입니다. 기준은 교환 법칙 자체가 아니라 "읽는 사람이 기호만 보고 결과를 맞힐 수 있는가"입니다.

### 2.9 지침 8: 체크리스트로 따져 넉넉하게 구현한다

**문제 상황.** 가계부 라이브러리가 연월 타입 `YearMonth`를 공개합니다. 생성자(월은 1~12)와 접근자만 있고 트레이트는 하나도 구현하지 않았습니다.

**비권장 코드.** 사용자는 월별 지출을 합산하고, 시간 순서로 출력하고, 보고서 타입에 담으려 합니다. [`08_checklist_applied.rs`](examples/08_checklist_applied.rs)의 비권장 판은 `Eq + Hash`가 없어 접근자로 꺼낸 `(연, 월)` 튜플을 키로 쓰고, `Ord`가 없어 튜플로 정렬하고, `Display`가 없어 `{year:04}-{month:02}`를 직접 씁니다. 결과는 같지만 사용자 코드에 우회로가 가득하고, `YearMonth`를 필드로 가진 보고서 타입에는 `derive(Debug)`부터 붙지 않습니다(4.4절, E0277). 사용자가 `impl Debug for YearMonth`를 직접 쓰는 것은 트레이트도 타입도 남의 것이라 고아 규칙(E0117)에 걸립니다. 남은 길은 뉴타입으로 감싸고 하나하나 위임하는 것뿐인데, 그 비용은 [ch6](../ch6-newtype-pattern/README.md) 2.6절에서 봤습니다. 라이브러리 작성자가 한 줄로 해결할 일을 모든 사용자가 따로 치르는 셈입니다. Rust API 가이드라인도 같은 이유로 공개 타입에 흔한 트레이트를 넉넉하게 구현하라고 권합니다(C-COMMON-TRAITS).

**권장 코드.** 2.1절의 표를 위에서부터 따지고, 결정과 이유를 선언 옆에 남깁니다.

```rust
// Debug          예: 숨길 비밀 없음 → derive
// Clone, Copy    예: 정수 두 개짜리 값, 필드가 늘 일 없음 → derive
// PartialEq, Eq  예: 필드가 모두 같으면 같은 달, NaN 같은 예외 없음 → derive
// Hash           예: Eq와 같은 필드 → derive
// PartialOrd, Ord 예: 시간 순서 = (year, month) 선언 순서 → derive. 필드 순서를 바꾸지 않음
// Default        아니오: "기본 달"이 없고, derive하면 생성자가 거부하는 0년 0월이 생김
// Display        예: 표준 표기 YYYY-MM 하나 → 손으로
// 연산자          아니오: 연월 + 연월은 뜻이 없음. 다음 달은 next()로
// Send, Sync     자동: 필드가 모두 Send + Sync
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct YearMonth { year: u16, month: u8 }
```

권장 판의 사용자 코드는 `BTreeMap<YearMonth, u32>`에 바로 합산해 시간 순서로 출력하고, `HashMap`의 키로도 쓰고, 자기 보고서 타입에 `derive(Debug, Clone, PartialEq)`를 붙입니다.

**빠뜨림을 찾는 린트.** rustc에는 기본으로 꺼진(allow) 두 린트가 있습니다. 크레이트 루트에 `#![warn(missing_debug_implementations)]`를 두면 `Debug`가 없는 공개 타입마다 ``type does not implement `std::fmt::Debug`; consider adding `#[derive(Debug)]` or a manual implementation``을 경고합니다(예제 08에서 `#[expect]`를 지워 확인한 문구입니다. 다른 파일에서는 트레이트 이름이 `Debug`로만 나오기도 합니다). 공개되지 않은 타입에는 경고하지 않습니다. 함께 켠 `missing_copy_implementations`는 2.3절에서 본 대로 필드만 보고 권하므로 판단이 필요합니다. 예제 08은 두 린트를 켜고, 일부러 트레이트를 뺀 v0 판에 `#[expect]`로 이유를 남겼습니다.

**`Send`와 `Sync`는 구현하지 않습니다.** 둘은 자동 트레이트라 필드가 모두 `Send`이면 타입도 `Send`, 모두 `Sync`이면 `Sync`가 됩니다([Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 2.9절). 막는 것은 필드입니다. `Rc`는 둘 다 아니고, `Cell`과 `RefCell`은 `Sync`가 아니고, 원시 포인터는 둘 다 아닙니다. 그래서 신경 쓸 일은 구현이 아니라 "필드 하나를 바꿨다가 몰래 잃는 것"입니다. 예제 08의 `assert_send_sync::<v1::YearMonth>()` 한 줄은 `fn assert_send_sync<T: Send + Sync>() {}`를 부를 뿐이지만, 누군가 필드에 `Rc`를 넣으면 ``error[E0277]: `Rc<u8>` cannot be sent between threads safely``로 바뀌어 빌드 단계에서 알려 줍니다(Rust API 가이드라인 C-SEND-SYNC도 이런 확인을 권합니다). 직접 `unsafe impl Send`를 쓰는 것은 원시 포인터를 감싼 타입에서 스레드 안전성을 사람이 보증할 때뿐이며, [ch16](../ch16-avoid-unsafe/README.md)과 [ch17](../ch17-shared-state-parallelism/README.md), Part 10에서 다룹니다.

**따르지 않아도 되는 경우.**

- 크레이트 밖으로 나가지 않는 비공개 타입은 고아 규칙과 상관없이 언제든 덧붙일 수 있으므로, 필요해질 때 붙입니다.
- "넉넉하게"는 "약속할 수 있는 것만"의 범위 안에서입니다. `YearMonth`의 `Default`처럼 지킬 수 없는 약속은 넉넉함을 이유로 붙이지 않습니다. 나중에 트레이트를 더하는 것보다 붙였던 트레이트를 떼는 것이 사용자에게 훨씬 큰 변경이기 때문입니다([ch21](../ch21-semver/README.md)).

### 2.10 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **책에 없는 지침.** 다음은 책의 Item 10이 명시적으로 권하지 않는 내용입니다(2026-11-19에 온라인판에서 확인). 책은 트레이트마다 뜻과 derive 가능 여부, 구현하면 안 되는 대표 경우를 소개하는 데 무게를 둡니다.
  - 지침 1의 제네릭 바운드: `Clone` 문서의 Derivable 절과 E0599의 ``consider manually implementing the trait to avoid undesired bounds`` 도움말이 근거입니다.
  - 지침 2의 "제자리에서 고쳐 쓰는 상태"와 "`Copy`는 떼기 어렵다": 앞의 것은 이 챕터가 더한 판단이고, 뒤의 것은 `Copy` 문서의 "When should my type be Copy?" 절이 근거입니다.
  - 지침 4의 `total_cmp`와 정수 표현: 책은 `NaN`이 `PartialOrd`만 구현한 타입을 놀라게 한다는 데까지 다룹니다. `f64::total_cmp` 문서가 근거입니다.
  - 지침 5 전체: 책은 `Default`를 구조체 갱신 문법과 함께 소개합니다. clippy `new_without_default`와 ch6·ch7이 넘긴 판단(검증 뉴타입의 뒷문, 공개 타입의 `Default`)이 근거입니다.
  - 지침 6의 "표현이 여럿이면 `Display` 대신 이름 붙은 메서드": 표준 라이브러리의 `Path::display()`가 근거입니다.
  - 지침 7의 "`+=`는 `+`와 일치": 책의 "연산자는 일관된 묶음으로"(`Sub`와 `Neg`의 예)를 `AddAssign`으로 넓힌 것입니다.
  - 지침 8의 근거인 고아 규칙과 `Send`/`Sync` 확인: 책은 표준 트레이트를 구현해 두면 쓸모 있다고 하고 `Send`/`Sync`는 컴파일러가 자동으로 구현할 수 있다고만 합니다. Rust API 가이드라인 C-COMMON-TRAITS(고아 규칙)와 C-SEND-SYNC(자동 트레이트를 잃지 않았는지 확인)가 근거입니다.
- **달라지지 않은 점.** `missing_copy_implementations`와 `missing_debug_implementations`는 1.100.0에서도 기본 allow이고(`rustc -W help`로 확인), `Display`는 여전히 derive할 수 없습니다. 책이 `Copy`가 아닌 값을 옮긴 뒤 쓰는 예에서 보여 주는 E0382의 ``help: consider cloning the value if the performance cost is acceptable``도 `Clone`을 구현한 타입에서는 그대로 나옵니다. `Clone`이 없는 타입에서는 그 자리에 ``note: if `X` implemented `Clone`, you could clone the value``가 나옵니다(2.3절).

### 2.11 기억할 것

- 공개 타입의 트레이트는 사용자가 대신 채울 수 없습니다. 체크리스트로 하나씩 따져 약속할 수 있는 것은 넉넉하게 구현합니다.
- derive는 "모든 필드를, 선언 순서대로"입니다. 뜻과 다르면 표현을 정규화해 derive를 맞게 만들거나 손으로 쓰고, 제네릭 타입은 붙는 바운드를 확인합니다.
- `Copy`는 복사본이 원본과 따로 놀아도 되는 작은 값에만 붙입니다. 유일해야 하는 값에는 `Clone`도 붙이지 않습니다.
- `PartialEq`를 손으로 썼다면 `Hash`도 같은 기준으로 씁니다. `f64`가 끼면 정렬은 `total_cmp`, 키는 정수 표현입니다.
- `Default`는 뜻이 분명한 기본값이 있을 때, `Display`는 표현이 하나일 때, 연산자는 기호만 보고 결과를 맞힐 수 있을 때 구현합니다. `+=`는 `+`로 구현합니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행합니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_derive_matches_meaning.rs`](examples/01_derive_matches_meaning.rs) | derive가 뜻과 맞는지 | 분수에 모두 derive, 제네릭에 `derive(Clone)` | 정규화 + 손으로 쓴 `Ord`, 바운드 없는 `Clone` | 2.2 |
| [`02_copy_promise.rs`](examples/02_copy_promise.rs) | `Copy`는 값에만 | `Copy`인 승인과 집계 | `Copy` 없음 | 2.3 |
| [`03_eq_hash_agree.rs`](examples/03_eq_hash_agree.rs) | `Eq`와 `Hash`를 맞춤 | 손으로 쓴 `PartialEq` + derive한 `Hash` | 둘 다 `id`만 | 2.4 |
| [`04_float_fields.rs`](examples/04_float_fields.rs) | `f64` 필드 | `==`로 찾기, `unwrap_or(Equal)` 정렬 | `total_cmp`, 정수 키 | 2.5 |
| [`05_default_meaningful.rs`](examples/05_default_meaningful.rs) | `Default` 판단 | `new()`만, 검증 타입에 `derive(Default)` | `derive(Default)`, 이름 붙은 상수 | 2.6 |
| [`06_debug_display.rs`](examples/06_debug_display.rs) | `Debug`와 `Display` | 비밀이 드러나는 derive, 하나를 골라 버린 `Display` | 비밀 타입에서 가림, `si()`/`iec()` | 2.7 |
| [`07_operator_conventions.rs`](examples/07_operator_conventions.rs) | 연산자의 통념 | 덮어쓰기 `+`, 따로 노는 `+=` | 이름 붙은 메서드, `+`로 구현한 `+=` | 2.8 |
| [`08_checklist_applied.rs`](examples/08_checklist_applied.rs) | 체크리스트 적용 | 트레이트 없는 `YearMonth` | 체크리스트를 적용한 `YearMonth` | 2.9 |

### 01_derive_matches_meaning — derive가 뜻과 맞는지

1/2, 2/4, 1/3, 2/3을 두 판의 `Ratio`로 만들어 비교, 정렬, 집합에 넣기를 해 봅니다. 뒤의 두 구역은 `Rc`를 감싼 핸들의 `Clone`입니다. derive 판은 `T = String`일 때만 복제되고(`Connection`을 담으면 E0599), 손으로 쓴 판은 `Clone`이 아닌 `Connection`을 담아도 복제됩니다.

```sh
cargo run -p er-ch10-standard-traits --example 01_derive_matches_meaning
```

```text
입력: 1/2, 2/4, 1/3, 2/3
[비권장] 필드 그대로 모두 derive
  1/2 == 2/4 ? false
  정렬: [1/2, 1/3, 2/3, 2/4]
  서로 다른 값: 4개
[권장] 생성자에서 기약분수로, Ord는 손으로
  1/2 == 2/4 ? true
  정렬: [1/3, 1/2, 2/3]
  서로 다른 값: 3개
  Ratio::new(3, -6) = Some("-1/2")
[비권장] derive(Clone)한 DerivedShared<T>
  DerivedShared<String> 복제: 강한 참조 2개
[권장] 손으로 쓴 Clone은 T: Clone을 요구하지 않음
  Shared<Connection> 복제: 강한 참조 2개, 같은 연결 #7? true
```

### 02_copy_promise — `Copy`는 값에만

10만 원이 든 계좌에 30000원짜리 승인 한 건을 씁니다. 비권장 판은 같은 승인으로 두 번 송금되고, 권장 판의 두 번째 호출은 주석 처리했습니다(풀면 E0382). 집계 구역은 찬성 2표를 넣습니다.

```sh
cargo run -p er-ch10-standard-traits --example 02_copy_promise
```

```text
[비권장] Copy인 송금 승인
  승인 30000원 1건, 잔액 40000원
[권장] Copy도 Clone도 없는 송금 승인
  승인 30000원 1건, 잔액 70000원
[비권장] Copy인 집계
  찬성 투표 뒤 총 1표, 한 번 더 투표 뒤 총 1표
  찬성 2표를 넣었는데 찬성 0, 반대 0
[권장] Copy가 아닌 집계
  찬성 투표 뒤 총 1표, 한 번 더 투표 뒤 총 2표
  찬성 2표를 넣었고 찬성 2, 반대 0
```

비권장 판의 "총 1표"는 매번 원래 집계(0표)의 사본에 1을 더한 값입니다. 함수가 돌려주는 값만 보면 첫 투표는 맞아 보입니다.

### 03_eq_hash_agree — `Eq`와 `Hash`를 맞춤

리뷰 128개일 때 즐겨찾기에 넣은 카페를 리뷰 131개가 된 뒤 다시 찾습니다. 해시 값은 출력하지 않고 "같은가"만 출력합니다.

```sh
cargo run -p er-ch10-standard-traits --example 03_eq_hash_agree
```

```text
[비권장] PartialEq는 id만, Hash는 모든 필드
  저장한 값: 골목 커피 리뷰 128개, 다시 불러온 값: 골목 커피 리뷰 131개
  saved == fresh? true
  해시가 같은가? false
  즐겨찾기에 있나? false
  없다길래 넣었더니 즐겨찾기 2개
[권장] PartialEq와 Hash 모두 id만
  saved == fresh? true
  해시가 같은가? true
  즐겨찾기에 있나? true
  다시 넣기: false, 즐겨찾기 1개
  집합에 남은 값: 골목 커피 리뷰 128개
```

시드가 고정된 해셔를 썼으므로 이 출력은 실행마다 같습니다. 마지막 줄은 이미 같은 값이 있으면 `insert`가 원래 값을 바꾸지 않는다는 것을 보여 줍니다. 최신 리뷰 수로 바꾸려면 `replace`를 씁니다.

### 04_float_fields — `f64` 필드

핀 5개(그중 하나는 위도가 `NaN`)를 `==`로 찾고, 위도로 정렬하고, 정수 키로 걸러 냅니다. 리터럴 `f64::NAN`만 쓰므로 `total_cmp`의 결과도 실행마다 같습니다.

```sh
cargo run -p er-ch10-standard-traits --example 04_float_fields
```

```text
[비권장] NaN이 든 값을 ==로 찾기
  failed == 그 사본? false
  목록에 들어 있는 failed를 contains로 찾기: false
  서울시청 찾기: true
[비권장] partial_cmp(..).unwrap_or(Equal)로 위도 정렬(남→북)
  ["서울시청", "주소 변환 실패", "부산역", "대전역", "강릉역"]
[권장] total_cmp로 위도 정렬(남→북)
  ["부산역", "대전역", "서울시청", "강릉역", "주소 변환 실패"]
[권장] 키가 필요하면 정수 표현으로
  핀 6개 → 서로 다른 좌표 4개, 거절 1개
  가장 남쪽: Some(PinKey { lat_e6: 35115200, lng_e6: 129042200 })
```

비권장 정렬의 결과는 이 툴체인의 정렬 구현이 이 입력에 낸 결과이며, 문서가 약속하는 순서가 아닙니다.

### 05_default_meaningful — `Default` 판단

3층 대기열에 문서 두 건을 넣고 비우며 꺼냅니다. 두 판의 결과는 같고 사용자 코드만 다릅니다. 뒤의 두 구역은 인쇄 부수 `Copies`의 `Default`입니다.

```sh
cargo run -p er-ch10-standard-traits --example 05_default_meaningful
```

```text
[비권장] new()만 있는 대기열: 사용자가 우회합니다
  3층에서 2건을 보냄, 남은 0건
[권장] Default가 있는 대기열: 표준 API가 그대로 열립니다
  3층에서 2건을 보냄, 남은 0건
  사용자의 Office::default(): 대기열 비었나 true, 방향 Portrait
[비권장] 불변식이 있는 타입에 derive(Default)
  Copies::new(0) = None, Copies::default() = 0부
[권장] Default 대신 이름 붙은 상수
  Copies::ONE = 1부, Copies::new(30) = 30부, Copies::new(0) = None
```

### 06_debug_display — `Debug`와 `Display`

같은 결제를 두 판으로 로그에 남기고, 1572864바이트를 두 판으로 출력합니다. `{:#?}` 출력은 들여쓰기 없이 줄의 처음부터 나옵니다.

```sh
cargo run -p er-ch10-standard-traits --example 06_debug_display
```

```text
[비권장] 카드 번호가 든 구조체에 derive(Debug)
  로그: RawPayment { order: 1042, card: "1234-5678-9012-3456" }
  주문 1042, 카드 끝자리 3456
[권장] 비밀을 담는 타입의 Debug에서 가림
  로그: Payment { order: 1042, card: CardNumber(****-3456) }
  여러 줄 로그:
Payment {
    order: 1042,
    card: CardNumber(
        ****-3456,
    ),
}
[비권장] Display가 단위 체계 하나를 골라 버림
  1.5 MiB
[권장] 표현을 이름으로 고름
  si: 1.6 MB, iec: 1.5 MiB, 바이트: 1572864
```

### 07_operator_conventions — 연산자의 통념

두 스타일을 양쪽 순서로 더해 보고, 볼륨 90에 30을 `+`와 `+=`로 더합니다.

```sh
cargo run -p er-ch10-standard-traits --example 07_operator_conventions
```

```text
[비권장] 덮어쓰기를 +로
  theme + warning = 굵게, 빨강
  warning + theme = 보통, 검정
  같은가? false
[권장] 방향이 이름에 드러나는 메서드
  theme.overridden_by(warning) = 굵게, 빨강
[비권장] +와 +=가 따로 구현됨
  LooseVolume(90) + 30 = LooseVolume(100), += 30 → LooseVolume(120)
[권장] +=는 +를 불러서
  Volume(90) + 30 = Volume(100), += 30 → Volume(100)
```

### 08_checklist_applied — 체크리스트 적용

지출 네 건(2026년 11월 두 건)을 월별로 합산합니다. 비권장 판은 `(연, 월)` 튜플로 우회하고, 권장 판은 `YearMonth`를 `BTreeMap`·`HashMap`의 키와 사용자 타입의 필드로 바로 씁니다. 출력에 없는 마지막 줄 `assert_send_sync::<v1::YearMonth>()`는 컴파일되는 것으로 확인이 끝납니다.

```sh
cargo run -p er-ch10-standard-traits --example 08_checklist_applied
```

```text
[비권장] 트레이트가 없는 YearMonth: (연, 월) 튜플로 우회
  2025-12: 8000원
  2026-10: 41000원
  2026-11: 63500원
[권장] 체크리스트를 적용한 YearMonth
  2025-12: 8000원
  2026-10: 41000원
  2026-11: 63500원
  HashMap에서 2026-11 찾기: Some(63500)
  최근 보고서: MonthlyReport { month: YearMonth { year: 2026, month: 11 }, total: 63500 }
  복제본과 같은가? true, 다음 달: 2026-12
  연 2026 / 월 11
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch10-standard-traits
rustc --edition 2024 compile_fail/e0599_derive_clone_bound.rs --out-dir /tmp/er-ch10
rustc --edition 2024 compile_fail/e0277_hash_with_f64_field.rs --out-dir /tmp/er-ch10
rustc --edition 2024 compile_fail/e0665_default_enum_without_attr.rs --out-dir /tmp/er-ch10
rustc --edition 2024 compile_fail/e0277_user_derive_debug.rs --out-dir /tmp/er-ch10
```

### 4.1 derive한 `Clone`이 타입 매개변수에 바운드를 붙임 — E0599

파일: [`compile_fail/e0599_derive_clone_bound.rs`](compile_fail/e0599_derive_clone_bound.rs)

```text
error[E0599]: the method `clone` exists for struct `Shared<Connection>`, but its trait bounds were not satisfied
  --> compile_fail/e0599_derive_clone_bound.rs:27:22
   |
18 | struct Connection {
   | ----------------- doesn't satisfy `Connection: Clone`
...
23 | struct Shared<T>(Rc<T>);
   | ---------------- method `clone` not found for this struct because it doesn't satisfy `Shared<Connection>: Clone`
...
27 |     let other = conn.clone(); // E0599
   |                      ^^^^^ method cannot be called on `Shared<Connection>` due to unsatisfied trait bounds
   |
note: trait bound `Connection: Clone` was not satisfied
  --> compile_fail/e0599_derive_clone_bound.rs:23:15
   |
22 | #[derive(Debug, Clone)]
   |                 ----- in this derive macro expansion
23 | struct Shared<T>(Rc<T>);
   |               ^ type parameter would need to implement `Clone`
   = help: consider manually implementing the trait to avoid undesired bounds
help: consider annotating `Connection` with `#[derive(Clone)]`
   (이하 고친 코드 제안 생략)
```

- **읽는 법**: 첫 줄의 "the method exists, but its trait bounds were not satisfied"가 핵심입니다. `clone`은 있지만 `Shared<Connection>`에는 적용되지 않는다는 뜻이고, `note`가 그 조건이 derive가 만든 `T: Clone`이라고 짚어 줍니다.
- **원인**: derive는 타입 매개변수마다 같은 트레이트를 바운드로 붙입니다(2.2절).
- **해결**: 도움말이 둘입니다. 마지막 도움말은 `Connection`에 `derive(Clone)`를 붙이라고 하지만, 연결은 하나뿐이어야 해서 일부러 `Clone`을 뺀 타입이므로 따르지 않습니다. 그 위의 ``consider manually implementing the trait to avoid undesired bounds``가 맞는 해결입니다. `impl<T> Clone for Shared<T>`를 손으로 씁니다(예제 01의 권장 판).

### 4.2 `f64` 필드가 있는 타입에 `derive(Eq, Hash)` — E0277

파일: [`compile_fail/e0277_hash_with_f64_field.rs`](compile_fail/e0277_hash_with_f64_field.rs)

```text
error[E0277]: the trait bound `f64: Eq` is not satisfied
  --> compile_fail/e0277_hash_with_f64_field.rs:17:5
   |
15 | #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
   |                                         -- in this derive macro expansion
16 | struct Pin {
17 |     lat: f64,
   |     ^^^^^^^^ the trait `Eq` is not implemented for `f64`
   |
   = help: the following other types implement trait `Eq`:
             (정수 타입 목록 생략)
note: required by a bound in `std::cmp::AssertParamIsEq`

error[E0277]: the trait bound `f64: Hash` is not satisfied
  --> compile_fail/e0277_hash_with_f64_field.rs:17:5
   (… lat 필드)

error[E0277]: the trait bound `f64: Hash` is not satisfied
  --> compile_fail/e0277_hash_with_f64_field.rs:18:5
   (… lng 필드)
```

- **읽는 법**: 에러가 derive 줄이 아니라 필드를 가리킵니다. derive가 필드마다 같은 트레이트를 요구하기 때문입니다. `Hash`는 필드마다 에러가 나고, `Eq`는 첫 필드 하나만 보고됩니다. `Eq`의 `note`에 나오는 `AssertParamIsEq`는 메서드가 없는 `Eq`를 derive할 때 컴파일러가 필드 타입을 확인하려고 쓰는 내부 도우미입니다.
- **원인**: `f64`는 `NaN` 때문에 `Eq`를 약속할 수 없고, `Hash`도 구현하지 않습니다(2.5절).
- **해결**: 비교만 필요하면 `PartialEq`까지만 derive합니다. 키가 필요하면 정수 표현으로 바꾼 타입을 따로 둡니다(예제 04의 `PinKey`).

### 4.3 `#[default]` 없는 열거형에 `derive(Default)` — E0665

파일: [`compile_fail/e0665_default_enum_without_attr.rs`](compile_fail/e0665_default_enum_without_attr.rs)

```text
error[E0665]: `#[derive(Default)]` on enum with no `#[default]`
  --> compile_fail/e0665_default_enum_without_attr.rs:11:45
   |
11 |   #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
   |                                               ^^^^^^^
12 | / enum Orientation {
13 | |     Portrait,
14 | |     Landscape,
15 | | }
   | |_- this enum needs a unit variant marked with `#[default]`
   |
help: make this unit variant default by placing `#[default]` on it
   |
13 |     #[default] Portrait,
   |     ++++++++++
help: make this unit variant default by placing `#[default]` on it
   |
14 |     #[default] Landscape,
   |     ++++++++++
```

- **읽는 법**: 열거형은 어느 배리언트가 기본값인지 컴파일러가 정할 수 없으므로, 배리언트마다 `#[default]`를 붙일 후보로 제안합니다. 첫 배리언트를 저절로 고르지 않습니다.
- **원인**: `#[derive(Default)]`는 구조체에서는 필드마다 `default()`를 부르면 되지만, 열거형에서는 사람이 골라야 합니다. `Default` 문서에 따르면 `#[default]`는 데이터가 없는(유닛) 배리언트에만 붙일 수 있습니다.
- **해결**: 누구에게나 같은 뜻인 배리언트(여기서는 `Portrait`)에 `#[default]`를 붙입니다. 고를 배리언트가 없다면 `Default`를 derive하지 않는 것이 맞습니다(2.6절).

### 4.4 사용자 타입의 `derive(Debug)`가 라이브러리 타입에서 막힘 — E0277

파일: [`compile_fail/e0277_user_derive_debug.rs`](compile_fail/e0277_user_derive_debug.rs)

```text
error[E0277]: `YearMonth` doesn't implement `Debug`
  --> compile_fail/e0277_user_derive_debug.rs:24:5
   |
22 | #[derive(Debug)]
   |          ----- in this derive macro expansion
23 | struct Report {
24 |     month: ledger::YearMonth,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Debug` is not implemented for `YearMonth`
   |
help: consider annotating `YearMonth` with `#[derive(Debug)]`
   |
16 +     #[derive(Debug)]
17 |     pub struct YearMonth {
   |
```

- **읽는 법**: 사용자가 자기 타입 `Report`에 붙인 derive가, 필드 타입 `YearMonth`에 `Debug`가 없어서 실패합니다. [Part 0 ch5](../../part0-getting-started/ch5-structs/README.md) 4.1절의 E0277은 내 타입에 derive를 빠뜨린 경우였고, 여기서는 원인이 남의 타입에 있습니다.
- **원인**: 이 파일은 한 크레이트라 `help:`가 `YearMonth`에 derive를 붙이라고 제안합니다. `YearMonth`를 별도 크레이트로 컴파일해 같은 코드를 시험하면 이 `help:`가 나오지 않습니다. 사용자는 그 타입을 고칠 수 없고, `impl Debug for ledger::YearMonth`를 쓰면 E0117(고아 규칙)입니다.
- **해결**: 라이브러리 쪽에서는 공개 타입에 `Debug`를 구현합니다(2.9절). 사용자 쪽에서 라이브러리를 고칠 수 없다면 `Report`의 `Debug`를 손으로 쓰며 그 필드를 접근자로 꺼내 출력하거나, 뉴타입으로 감쌉니다([ch6](../ch6-newtype-pattern/README.md) 2.5절).

### 4.5 그 밖의 에러와 린트

`Copy`와 관련된 컴파일 에러는 Part 1 ch6에 파일이 있습니다. 표의 clippy 메시지는 이 챕터의 예제에서 `#[expect]`를 지우거나 같은 모양의 코드를 컴파일해 1.100.0에서 확인했습니다.

| 코드 | 결과 | 다룬 곳 |
| --- | --- | --- |
| `Copy`가 아닌 필드가 있는 타입에 `derive(Copy)` | E0204 | [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 4.4절 |
| `Drop`을 구현한 타입에 `Copy` | E0184 | Part 1 ch6 4.5절 |
| derive한 `Hash` + 손으로 쓴 `PartialEq` | clippy `derived_hash_with_manual_eq`(deny) | 2.4절, 예제 03 |
| 손으로 쓴 `Ord` + derive한 `PartialOrd` | clippy `derive_ord_xor_partial_ord`(deny) | 2.2절 |
| 인자 없는 `pub fn new()`만 있고 `Default` 없음 | clippy `new_without_default`(warn) | 2.6절, 예제 05 |
| 고유 메서드 `to_string()` | clippy `inherent_to_string`(warn) | 2.7절 |
| 메서드 이름 `add`, `eq`, `default` 등 | clippy `should_implement_trait`(warn) | 2.8절 |
| `Add` 안의 `-`, `AddAssign` 안의 `-=` | clippy `suspicious_arithmetic_impl`, `suspicious_op_assign_impl`(warn) | 2.8절 |
| `Debug`나 `Copy`가 없는 공개 타입 | rustc `missing_debug_implementations`, `missing_copy_implementations`(allow, 켜야 경고) | 2.3절, 2.9절, 예제 08 |

`PartialEq`만 derive하고 `Eq`를 빠뜨린 것은 clippy의 `derive_partial_eq_without_eq`가 잡지만 기본으로 꺼져 있습니다(`clippy-driver -W help`로 확인). 비교·해시·복제 구현의 나머지 린트는 Part 1 ch6 4.6절의 표에 있습니다.

## 5. 연습 문제

### 연습 1. 체크리스트 적용하기

다음 공개 타입마다 2.1절의 체크리스트를 적용해, derive할 트레이트, 손으로 구현할 트레이트, 구현하지 않을 트레이트를 이유와 함께 정하세요.

1. 로그인 세션 토큰 `SessionToken(String)`
2. 요일 `enum Weekday { Mon, Tue, Wed, Thu, Fri, Sat, Sun }`
3. 섭씨온도 `Celsius(f64)`
4. 1000×1000 크기의 픽셀 격자 `Canvas { width: usize, height: usize, pixels: Vec<u8> }`

<details>
<summary>힌트</summary>

1번은 비밀입니다. `Debug`는 손으로 써서 가리고(2.7절), 토큰을 비교해야 하면 `PartialEq`/`Eq`, 맵의 키라면 `Hash`를 derive합니다. `Clone`은 토큰을 여러 곳에 들고 다녀야 하는지에 따라 정하고, `Display`·`Default`·`Copy`(어차피 불가)는 두지 않습니다. 2번은 거의 모든 것을 derive합니다(`Copy` 포함). `Ord`는 "월요일이 앞"이라는 순서가 문화권마다 다르다는 점을 따져 보고, `Default`는 고를 요일이 없으므로 두지 않습니다. 3번은 `PartialEq`/`PartialOrd`까지이고(2.5절), `Display`는 `21.5°C` 하나로 분명합니다. 같은 단위끼리의 `+`는 뜻이 있지만 `Celsius * Celsius`는 뜻이 없습니다([ch6](../ch6-newtype-pattern/README.md) 2.4절). 4번은 `Clone`은 두되 `Copy`는 붙일 수 없고(`Vec`), 큰 값이라 복제 비용이 `.clone()`으로 드러나는 것이 오히려 좋습니다. `Debug`를 derive하면 픽셀 100만 개가 출력되므로 크기만 보여 주는 `Debug`를 손으로 쓰는 것이 낫습니다(책도 출력이 너무 커지는 경우를 손으로 쓸 이유로 듭니다).

</details>

### 연습 2. 동일성은 키 타입에 맡기기

[`03_eq_hash_agree.rs`](examples/03_eq_hash_agree.rs)를 `CafeId(u32)` 키로 다시 설계하세요. `HashMap<CafeId, Cafe>`에 카페를 넣고, 리뷰 수가 바뀐 카페를 다시 받으면 같은 항목이 갱신되어야 합니다. `Cafe`에는 `PartialEq`도 `Hash`도 구현하지 않습니다.

<details>
<summary>힌트</summary>

`#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)] struct CafeId(u32);`를 만들고 `Cafe`의 `id` 필드 타입을 `CafeId`로 바꿉니다. 갱신은 `favorites.insert(fresh.id, fresh)`이고, 이미 있던 값이 `Some(옛 값)`으로 돌아옵니다. 집합 대신 맵을 쓰면 "같은 카페인가"(키)와 "카페의 현재 상태"(값)가 분리되어, 어느 필드가 동일성에 참여하는지 고민할 일이 없어집니다.

</details>

### 연습 3. 분수의 연산자

[`01_derive_matches_meaning.rs`](examples/01_derive_matches_meaning.rs)의 권장 판 `Ratio`에 `Add`, `Neg`, `Sub`, `Mul`, `AddAssign`, `Sum`을 구현하세요. `1/2 + 1/3`은 `5/6`, `1/2 - 1/3`은 `1/6`, `[1/2, 1/3, 1/6]`의 합은 `1/1`이어야 합니다.

<details>
<summary>힌트</summary>

덧셈은 `a/b + c/d = (ad + cb) / bd`를 `i64`로 계산한 뒤 `Ratio::new`로 정규화합니다. `new`가 `None`(결과가 `i32`를 넘음)을 돌려주면 연산자는 `Option`을 돌려줄 수 없으므로, 정수 덧셈처럼 패닉하거나 `checked_add` 같은 메서드를 따로 두는 것 중에서 고릅니다(패닉하는 API의 문제는 [`ch18-dont-panic`](../ch18-dont-panic/README.md)). 2.8절의 원칙대로 `Sub`는 `self + (-rhs)`, `AddAssign`은 `*self = *self + rhs`로 써서 규칙을 한 곳에 둡니다. `Sum`은 `iter.fold(zero, |acc, r| acc + r)` 모양이고, `sum`은 `Option`이 아니라 `Self`를 돌려주므로 0은 모듈 안에서 `Ratio { num: 0, den: 1 }`로 직접 만듭니다(이미 기약분수라 정규화가 필요 없습니다).

</details>

### 연습 4. 전순서를 갖춘 `f64` 뉴타입

`NaN`을 거절하는 생성자를 가진 `struct Finite(f64);`를 만들고, `Eq`, `Ord`, `Hash`를 손으로 구현해 `BTreeSet<Finite>`과 `HashSet<Finite>`에 넣어 보세요. `Finite::new(-0.0)`과 `Finite::new(0.0)`이 같은 원소로 취급되는지도 확인하세요.

<details>
<summary>힌트</summary>

`Ord::cmp`는 `self.0.total_cmp(&other.0)`, `PartialEq::eq`는 `self.cmp(other) == Ordering::Equal`, `Hash`는 `self.0.to_bits().hash(state)`로 쓰면 세 구현이 같은 기준(비트 표현)을 봅니다. 이 기준에서 `-0.0`과 `0.0`은 비트가 다르므로 서로 다른 원소입니다. `f64`의 `==`로는 둘이 같으므로, 같게 다루고 싶다면 생성자에서 `-0.0`을 `0.0`으로 바꿔 저장합니다(2.2절의 정규화와 같은 방법). `PartialEq`를 `self.0 == other.0`으로 쓰면 `-0.0`과 `0.0`에서 `==`는 `true`인데 해시가 달라 2.4절의 버그가 됩니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 10: Familiarize yourself with standard traits](https://effective-rust.com/std-traits.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- Rust API 가이드라인
  - [Types eagerly implement common traits (C-COMMON-TRAITS)](https://rust-lang.github.io/api-guidelines/interoperability.html#c-common-traits): 고아 규칙 때문에 공개 타입에 흔한 트레이트를 넉넉하게 구현하라는 지침(2.9절)
  - [Types are Send and Sync where possible (C-SEND-SYNC)](https://rust-lang.github.io/api-guidelines/interoperability.html#c-send-sync): 자동 트레이트를 몰래 잃지 않게 확인하라는 지침(2.9절)
- 표준 라이브러리 문서
  - [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html)의 Derivable 절(제네릭 바운드), [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html)의 "When should my type be Copy?" 절
  - [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html)의 `#[default]` 설명, [`Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html)의 Stability 절
  - [`Hash`](https://doc.rust-lang.org/std/hash/trait.Hash.html), [`Eq`](https://doc.rust-lang.org/std/cmp/trait.Eq.html), [`Ord`](https://doc.rust-lang.org/std/cmp/trait.Ord.html), [`f64::total_cmp`](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp), [`slice::sort_by`](https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by)(전순서가 아닌 비교 함수)
  - [`Path::display`](https://doc.rust-lang.org/std/path/struct.Path.html#method.display), [`std::num::Saturating`](https://doc.rust-lang.org/std/num/struct.Saturating.html), [`std::ops`](https://doc.rust-lang.org/std/ops/index.html)
- rustc 린트: [`missing_copy_implementations`](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#missing-copy-implementations), [`missing_debug_implementations`](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#missing-debug-implementations)
- 에러 설명: `rustc --explain E0599`, `rustc --explain E0277`, `rustc --explain E0665`
