# ch24. API에 타입이 드러나는 의존성은 재내보내기

라이브러리의 공개 함수가 다른 크레이트의 타입을 받거나 돌려주면, 그 크레이트는 라이브러리 사용자에게도 보이는 의존성이 됩니다. 사용자가 같은 크레이트의 **호환되지 않는 다른 버전**에 직접 의존하고 있으면, 사용자가 만든 값은 이름과 모양이 같아도 라이브러리가 기대하는 타입이 아닙니다. 이 챕터는 그런 일이 왜 생기는지 보고, 라이브러리 작성자가 의존성을 `pub use`로 재내보내 사용자가 "라이브러리가 쓰는 바로 그 버전"에 닿게 하는 방법과, 아예 의존성을 API에서 감추는 대안을 비교합니다.

- 원제: [Item 24: Re-export dependencies whose types appear in your API](https://effective-rust.com/re-export.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch24-re-export-dependencies`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 내용을 안다고 보고 진행합니다.

- [Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md) 2.3절: 버전 요구 사항과 `Cargo.lock`
- [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.10절: `pub use` 재내보내기
- [ch21](../ch21-semver/README.md) 2.1절(0.x에서는 가장 왼쪽의 0이 아닌 자리가 메이저), 2.2절(연습용 레지스트리), 2.3절(호환되지 않는 변경)
- [ch23](../ch23-avoid-wildcard-imports/README.md) 2.1절: 글롭으로 들어온 이름의 순위
- [ch6](../ch6-newtype-pattern/README.md) 2.6절(위임 구현의 비용), [ch19](../ch19-avoid-reflection/README.md) 2.2절(`type_name`은 진단에만)

[ch22](../ch22-minimize-visibility/README.md) 2.5절은 자기 크레이트 안의 항목을 `pub use`로 공개 경로에 올리는 데까지 다루고, 다른 크레이트를 재내보내는 일을 이 챕터로 미뤘습니다. 부록은 Cargo를 다루는 본편(Part 3)을 전제로 하지 않으므로, 이 챕터에 필요한 Cargo 동작은 그때마다 한두 문장으로 풀고 자세한 것은 [Part 3 ch2](../../part3-cargo/ch2-dependencies/README.md)로 넘깁니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Cargo가 같은 크레이트의 호환되지 않는 두 버전을 한 프로그램에 함께 넣는 이유와, 컴파일러에게 두 버전의 같은 이름 타입이 서로 다른 타입이라는 것을 설명할 수 있다. 의존성이 공개 의존성이 되는 자리를 코드에서 찾을 수 있다.
2. 두 버전이 섞여 난 E0308·E0277에서 `multiple different versions of crate` note를 찾아 원인을 읽고, 라이브러리를 고칠 수 없을 때 앱 쪽에서 할 수 있는 대처를 고를 수 있다.
3. 공개 API에 타입이 드러나는 의존성을 `pub use`로 재내보내고, 타입만 내보낼지 크레이트 전체를 내보낼지 사용자가 해야 할 일을 기준으로 고를 수 있다.
4. 공개 의존성의 메이저 버전을 올리는 일이 내 크레이트의 메이저 변경이 되는 이유를 설명하고, 의존성을 드러낼지 뉴타입으로 감출지 판단할 수 있다.

## 2. 핵심 개념

2.1절에서 두 버전이 공존하는 원리를 정리하고, 2.2~2.3절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 예제는 모두 이 저장소에서 지어낸 크레이트를 씁니다. 금액 크레이트 `money`는 [`projects/two_versions`](projects/two_versions/Cargo.toml)의 연습용 레지스트리(ch21 2.2절의 디렉터리 소스)에 두 버전이 게시되어 있습니다.

| 크레이트 | 내용 |
| --- | --- |
| money 0.3.2 | 원 단위 금액 `Money`, 생성자 함수 `won(i64)`, 메서드 `as_won`, 트레이트 `Priced { fn price(&self) -> Money; }` |
| money 0.4.0 | `Money`가 통화를 함께 가짐. `as_won` 대신 `minor_units`, 달러를 만드는 `usd`가 생김. `won`과 `Priced`는 이름과 모양이 그대로 |
| receipt 1.0.0 | 영수증 라이브러리. `money = "0.3"`에 의존하고 `add(name, Money)`, `add_item<T: Priced + Display>`, `total() -> Money`로 money를 API에 드러냄. 정수를 받는 `add_won(name, i64)`도 있음 |
| cafe-* | 카페 앱. 관광객용 달러 가격이 필요해서 `money = "0.4"`와 receipt에 함께 의존 |

0.3 → 0.4는 0.x의 둘째 자리가 오른 것이므로 Cargo가 보기에 메이저 변경입니다(ch21 2.1절). 두 버전 사이에 무엇이 바뀌었는지는 [`vendor/money-0.4.0/src/lib.rs`](projects/two_versions/vendor/money-0.4.0/src/lib.rs)의 머리 주석에 적어 두었습니다.

### 2.1 같은 크레이트의 두 버전이 한 프로그램에 들어오는 이유

Cargo는 의존성마다 버전 요구 사항을 만족하는 가장 높은 버전을 고르되, **서로 호환되는 요구 사항은 한 버전으로 합칩니다.** 두 패키지가 `money = "0.3"`과 `money = "0.3.1"`을 요구하면 0.3.2 하나로 빌드합니다. 하지만 receipt의 `"0.3"`과 cafe의 `"0.4"`는 호환 범위가 겹치지 않으므로 둘을 모두 빌드해 한 프로그램에 넣습니다(Cargo Book의 Dependency Resolution). 어느 버전이 어디서 왔는지는 `cargo tree`로 봅니다(경로는 `<저장소>/…`로 줄였습니다).

```text
$ cargo tree -p cafe-separate
cafe-separate v0.1.0 (<저장소>/…/two_versions/cafe-separate)
├── money v0.4.0
└── receipt v1.0.0 (<저장소>/…/two_versions/receipt-1.0.0)
    └── money v0.3.2
```

`cargo tree -d`는 두 버전 이상 들어온 패키지만 골라 보여 줍니다. 중복을 찾아 줄이는 일은 [ch25](../ch25-dependency-graph/README.md)에서 다룹니다. 컴파일러에게 money 0.3.2와 0.4.0은 **이름이 우연히 같은 서로 다른 크레이트**입니다. 그래서 0.3.2의 `money::Money`와 0.4.0의 `money::Money`는 다른 타입이고, 두 정의가 글자 하나 다르지 않더라도 마찬가지입니다. 한 크레이트 안의 두 모듈에 같은 구조체를 정의한 것과 같은 상황이라, [`01_same_name_different_types.rs`](examples/01_same_name_different_types.rs)는 두 버전을 모듈 `money_0_3`, `money_0_4`로 흉내 내어 `TypeId`가 다르다는 것을 보여 줍니다. 실제 크레이트에서는 차이가 더 감춰집니다. `cafe-separate`가 `type_name`으로 출력한 receipt의 합계 타입과 앱의 `Money` 타입은 둘 다 버전 없이 `money::Money`입니다([ch19](../ch19-avoid-reflection/README.md) 2.2절에서 본 `type_name`의 한계입니다).

그래도 `cafe-separate`는 빌드되고 잘 동작합니다. cafe는 receipt에 원 단위 정수만 넘기고(`add_won`), 돌려받은 합계는 출력만 하기 때문입니다. **두 버전이 공존하는 것 자체는 문제가 아니고, 한 버전의 값이 다른 버전을 기대하는 곳으로 넘어갈 때 문제가 됩니다.** 그 경계가 라이브러리의 공개 API입니다.

어떤 의존성의 타입이나 트레이트가 내 공개 API에 나타나면 그 의존성을 **공개 의존성**(public dependency), 내부에서만 쓰면 **비공개 의존성**(private dependency)이라고 부릅니다. Rust API Guidelines는 공개 의존성이 "생각지 못한 곳으로 숨어든다"고 경고합니다. receipt 기준으로 money를 공개 의존성으로 만드는 자리는 다음과 같습니다.

| 자리 | 예 |
| --- | --- |
| 공개 함수의 매개변수·반환 타입 | `add(&mut self, name: &str, price: Money)`, `total(&self) -> Money` |
| 공개 제네릭의 바운드 | `add_item<T: Priced + Display>` |
| 공개 필드 | `pub price: money::Money` |
| 내 공개 타입의 트레이트 구현 | `impl From<money::Money> for Price` (API Guidelines가 에러 타입으로 드는 예와 같은 모양) |
| 재내보내기 | `pub use money;` |

반대로 `add_won(&mut self, name: &str, won: i64)`처럼 원시 타입만 주고받거나, 비공개 필드에 담기만 하면 money는 드러나지 않습니다. 경계에서 실제로 무엇이 깨지는지는 2.2절에서 보고, 여기서는 컴파일러가 잡지 못하는 경우 하나만 짚습니다. 값을 `Box<dyn Any>`처럼 타입을 지운 채 넘기면 컴파일 에러가 없고, 받는 쪽의 다운캐스트가 실행 중에 `None`을 냅니다(Cargo Book이 드는 예, 예제 01의 3번).

### 2.2 지침 1: API에 타입이 드러나는 의존성은 재내보낸다

**문제 상황.** cafe가 마들렌 가격을 money의 `won(3200)`으로 만들어 `add`에 넘기고, 자기 타입 `Latte`에 money의 `Priced`를 구현해 `add_item`에 넘기려 합니다. cafe가 아는 `money`는 자기 `Cargo.toml`에 적은 0.4뿐입니다.

**비권장 코드.** receipt 1.0.0은 money의 타입을 API에 쓰면서 재내보내지 않습니다. 그러면 사용자가 receipt가 쓰는 money 0.3에 닿을 경로가 없습니다. [`cafe-mixed-fails`](projects/two_versions/cafe-mixed-fails/src/main.rs)는 자기 money로 값을 만듭니다.

```rust
impl money::Priced for Latte {             // 앱의 money = 0.4
    fn price(&self) -> money::Money { money::won(5000) }
}
bill.add("마들렌", money::won(3200));        // E0308
bill.add_item(&Latte);                     // E0277
```

두 줄 모두 컴파일되지 않습니다. 첫 줄은 "`money::Money`를 기대했는데 `Money`를 받았다"고 하고, 둘째 줄은 `Latte`가 `money::Priced`를 구현하지 않았다고 합니다. 바로 위에서 구현했는데도 말입니다. 두 에러 모두 ``note: there are multiple different versions of crate `money` in the dependency graph``가 붙어 원인을 알려 줍니다(전체 메시지와 읽는 법은 4.1·4.2절).

**권장 코드.** receipt 1.1.0은 1.0.0에 한 줄을 더합니다.

```rust
/// 공개 API에 드러나는 의존성을 통째로 재내보냅니다.
pub use money;
```

이제 사용자는 `receipt::money`라는 공개 경로로 receipt가 쓰는 바로 그 money에 닿습니다. [`cafe-re-exported`](projects/two_versions/cafe-re-exported/src/main.rs)는 receipt로 넘어가는 값과 트레이트만 이 경로로 바꾸고, 달러 가격 같은 자기 일에는 계속 money 0.4를 씁니다.

```rust
impl receipt::money::Priced for Latte {
    fn price(&self) -> receipt::money::Money { receipt::money::won(5000) }
}
bill.add("마들렌", receipt::money::won(3200));
bill.add_item(&Latte);
println!("  관광객 메뉴판의 아메리카노: {}", money::usd(350)); // 앱의 money 0.4
```

빌드되고, 출력 첫 줄이 `[cafe-re-exported] receipt 1.1.0, receipt의 money 0.3.2, 앱의 money 0.4.0`입니다. 사용자는 receipt가 money의 몇 버전을 쓰는지 몰라도 됩니다. 나중에 money 0.3.3이 게시되어 Cargo가 그것을 고르더라도 `receipt::money`는 receipt가 실제로 쓰는 버전을 가리킵니다. 예제 01의 2번이 보이듯 `pub use`는 새 타입을 만들지 않고 같은 타입에 닿는 경로를 하나 더 만들 뿐입니다(Reference의 Use declarations). 공개 항목을 더하는 일이므로 1.0.0 → 1.1.0은 마이너 변경입니다(ch21 2.3절). 다만 [ch23](../ch23-avoid-wildcard-imports/README.md)에서 본 대로, 공개 항목 추가는 글롭으로 가져온 사용자에게 영향을 줄 수 있습니다(아래 "사용자 쪽").

**타입만 재내보낼까, 크레이트 전체를 재내보낼까.** 책은 두 방법을 들고, 사용자가 의존성의 생성자 함수까지 써야 하는 예에서 크레이트 전체를 고릅니다. [`02_re_export_type_or_crate.rs`](examples/02_re_export_type_or_crate.rs)가 두 판을 비교합니다.

| 사용자가 하려는 일 | `pub use money::{Money, Priced};` | `pub use money;` |
| --- | --- | --- |
| 시그니처에 `Money`를 적고 받은 값을 다룸 | 됨 | 됨 |
| `won(5000)`으로 값을 만듦 | 안 됨(E0433, 4.3절) | 됨 |
| 자기 타입에 `Priced`를 구현함 | 이름은 보이지만 `price`가 돌려줄 `Money`를 만들 수 없음 | 됨 |
| 내 공개 API에 더해지는 것 | 그 타입과 트레이트 | 의존성의 공개 API 전체 |

money처럼 생성자가 크레이트 수준 함수이거나 사용자가 트레이트를 구현해야 하면 크레이트 전체를 재내보냅니다. 라이브러리가 만들어 돌려주기만 하고 사용자는 이름만 적으면 되는 타입이라면 타입만으로 충분하고, 내 공개 API도 덜 넓어집니다. 어느 쪽이든 내 버전이 의존성의 메이저 버전에 묶이는 것은 같습니다(2.3절).

**사용자 쪽: 재내보낸 경로는 이름을 적어 가져온다.** `cafe-re-exported`는 `receipt::money::…`처럼 경로를 적고 `Receipt`도 `receipt::Receipt`로 씁니다. 여기에 `use receipt::*;`를 더하면 글롭이 `money`라는 이름까지 들여와 앱이 의존하는 `money` 크레이트를 가립니다([ch23](../ch23-avoid-wildcard-imports/README.md) 2.1절의 규칙 1). 그러면 `money::usd(350)`은 ``cannot find function `usd` in crate `money` ``(E0425)가 되고, 그 줄을 지우면 에러 없이 `앱의 money 0.3.2`가 출력됩니다. 앱의 `money::…`가 모두 조용히 0.3을 가리키게 된 것이고, 단서는 cargo 1.100.0이 기본으로 내는 ``warning: unused dependency `money` `` 하나입니다(연습 2). 의존성을 재내보내는 라이브러리는 글롭으로 가져오지 않습니다.

**라이브러리를 고칠 수 없을 때.** receipt 1.0.0을 써야 하는 앱 작성자에게는 두 가지 방법이 있습니다.

- **요구 사항을 라이브러리와 같은 호환 범위로 맞춥니다.** 앱도 `money = "0.3"`에 의존하면 Cargo가 receipt의 것과 한 버전으로 합칩니다. 가장 간단하지만, cafe처럼 0.4에만 있는 `usd`가 필요하면 쓸 수 없습니다.
- **의존성 이름 바꾸기**(renaming dependencies)로 두 버전에 모두 의존합니다. [`cafe-renamed`](projects/two_versions/cafe-renamed/Cargo.toml)는 `Cargo.toml`에 다음 줄을 더하고, receipt와 주고받는 값은 `money03::won(3200)`으로 만듭니다. `package` 키는 "이 의존성의 실제 패키지 이름"을 적는 곳이고, 코드에서는 왼쪽의 이름(`money03`)으로 씁니다([Part 3 ch2](../../part3-cargo/ch2-dependencies/README.md)에서 다룸).

```toml
money = "0.4"
money03 = { package = "money", version = "0.3" }  # receipt의 "0.3"과 합쳐져 같은 0.3.2가 됨
```

**지침을 따르지 않아도 되는 경우.** 의존성이 API에 드러나지 않는 비공개 의존성이면 재내보내지 않습니다. 재내보내는 순간 그 의존성 전체가 내 공개 API가 되어, 2.3절의 결합만 새로 생깁니다. 공개 API가 없는 바이너리 크레이트(cafe 같은 앱)도 재내보낼 것이 없습니다. 그리고 2.3절대로 의존성을 감출 수 있다면 재내보내기보다 그쪽이 낫습니다.

### 2.3 지침 2: 의존성의 타입을 API에 드러내기 전에, 감출 수 있는지 따진다

**문제 상황.** receipt 1.1.0을 낸 뒤, receipt도 달러 영수증을 지원하려고 money를 0.4로 올리려 합니다.

**비권장 코드.** money를 드러낸 채로 올립니다. 그러면 `receipt::money`가 가리키는 것이 0.4로 바뀌고, `add`와 `total`의 `Money`도 0.4의 것이 됩니다. receipt의 공개 API에 있던 `receipt::money::Money::as_won`이 사라졌으므로 receipt 자신의 코드는 한 줄도 깨지지 않았어도 이것은 **receipt의 호환되지 않는 변경**이고, receipt는 2.0.0을 내야 합니다. 책도 이 점을 들어 다른 크레이트의 타입을 API에 쓰기 전에 신중히 생각하라고 합니다. [`03_hide_behind_newtype.rs`](examples/03_hide_behind_newtype.rs)에서 같은 사용자 코드를 money 0.4로 올린 드러낸 판에 대해 컴파일하면 ``error[E0599]: no method named `as_won` found for struct `money_0_4::Money` ``입니다.

Rust API Guidelines는 같은 이유로 공개 의존성이 모두 안정(1.0 이상)이어야 그 크레이트도 안정(1.0 이상)일 수 있다고 정합니다(C-STABLE). receipt 1.x가 0.x인 money를 드러내면, money가 0.x에서 메이저를 올릴 때마다 receipt도 메이저를 올리거나 옛 money에 머물러야 합니다.

**권장 코드(책에 없는 지침).** 의존성의 타입을 자기 뉴타입으로 감싸고 필드를 비공개로 둡니다. money는 비공개 의존성이 되고, 사용자는 `Price`만 봅니다. C-STABLE도 다른 크레이트의 타입을 비공개 필드에 담기만 하는 것은 괜찮다는 예를 듭니다.

```rust
pub struct Price(money::Money);           // 필드 비공개: 공개 API에는 Price만 보임
impl Price {
    pub fn won(amount: i64) -> Price { Price(money::won(amount)) }
    pub fn as_won(self) -> i64 { self.0.as_won() }  // 0.4에서는 이 한 줄을 minor_units로 고침
}
```

예제 03은 money 0.3 위의 판과 0.4로 올린 판에 같은 사용자 코드를 펼쳐 같은 결과를 얻습니다. money의 바뀐 API에 맞추는 일이 receipt 안에서 끝나므로 receipt는 마이너 버전으로 올릴 수 있습니다.

```text
감춘 판
  receipt 1.0.0 (money 0.3): 7700원
  receipt 1.1.0 (money 0.4): 7700원
```

대가는 [ch6](../ch6-newtype-pattern/README.md) 2.6절의 위임 구현의 비용입니다. money가 제공하던 표시 형식, 덧셈, 통화 같은 기능을 사용자가 쓰려면 `Price`에 하나씩 다시 열어 줘야 하고, 사용자는 `Price`를 money를 받는 다른 크레이트에 넘길 수 없습니다.

**지침을 따르지 않아도 되는 경우.** 책은 널리 쓰이는 준표준 크레이트이고 자기 의존성이 적다면 그 타입을 API에 드러내도 대체로 괜찮다고 봅니다. 널리 쓰이는 크레이트라면 사용자도 같은 버전을 쓰고 있을 가능성이 큽니다. 또 라이브러리의 목적이 그 의존성과 함께 쓰이는 것이라면(money 값을 다루는 확장 크레이트처럼) 감추면 쓸모가 없어집니다. 이때는 드러내고, 2.2절대로 재내보내고, 의존성의 메이저를 올릴 때 내 메이저도 올립니다.

### 2.4 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc·cargo 1.100.0에서 확인한 차이는 다음과 같습니다.

- **에러 메시지가 원인을 알려 줍니다.** 책은 이 상황의 에러 메시지가 Rust답지 않게 도움이 되지 않는다고 평가합니다. 1.100.0은 E0308과 E0277 모두에 ``there are multiple different versions of crate `money` in the dependency graph`` note를 붙이고, 기대한 쪽과 받은 쪽의 정의 위치를 각각 `this is the expected type`(트레이트는 `trait`), `this is the found type`으로 가리킨 뒤 `cargo tree`를 써 보라는 help를 줍니다. 다만 첫 줄(``expected `money::Money`, found `Money` ``)과 E0277의 help(``the trait `money::Priced` is not implemented for `Latte` ``)만 보면 여전히 헷갈리므로 note까지 읽어야 합니다.
- **바이너리도 두 버전에 직접 의존할 수 있습니다.** 책은 바이너리가 같은 크레이트의 두 버전을 직접 쓸 수는 없다고 하고, 대처로 다른 버전의 타입을 감추는 중간 래퍼 크레이트를 들면서 번거로운 방법이라고 덧붙입니다. Cargo Book은 의존성 이름 바꾸기의 용도로 "한 크레이트의 여러 버전에 의존하기"를 들고, `cafe-renamed`가 이 방법으로 빌드됩니다. 래퍼 크레이트도 여전히 쓸 수 있지만 더 번거롭습니다.
- **같은 버전의 두 사본(책의 각주)도 확인했습니다.** `cafe-renamed`의 `money03`을 path 의존성으로 `vendor/money-0.3.2`에 연결하면 두 쪽 모두 0.3.2인데도 같은 에러가 나고, note는 여전히 "multiple different versions"라고 합니다(연습 3).
- **책에 없는 지침과 내용.** 다음은 책의 Item 24가 직접 권하지 않는 것입니다(2026-11-19에 온라인판에서 확인). 책은 두 버전이 함께 들어오는 원리, E0277, 재내보내기(타입 또는 크레이트 전체), 바이너리 쪽의 래퍼 크레이트, 공개 의존성의 메이저 변경이 내 메이저 변경이 된다는 점, 준표준 크레이트의 예외까지 다룹니다.
  - 2.1절의 공개 의존성이 드러나는 자리 표: Rust API Guidelines의 C-STABLE이 근거입니다.
  - 2.1절의 다운캐스트가 실행 중에 실패하는 경우: Cargo Book의 Dependency Resolution(Version-incompatibility hazards)과 예제 01이 근거입니다.
  - 2.2절의 "사용자가 이름만 적는 타입은 타입만 재내보내도 된다"는 기준: 책은 크레이트 전체를 고른 이유만 말합니다. 예제 02와 `compile_fail/e0433_type_only_re_export.rs`가 근거입니다.
  - 2.2절의 "재내보낸 의존성을 글롭으로 가져오지 않는다"와 cargo의 `unused dependency` 경고: ch23과 연습 2의 실행 결과가 근거입니다.
  - 2.2절의 앱 쪽 대처(요구 사항 맞추기, 의존성 이름 바꾸기): Cargo Book의 Specifying Dependencies와 `cafe-renamed`가 근거입니다.
  - 2.3절의 권장 코드(뉴타입으로 감추기)와 C-STABLE: Rust API Guidelines, ch6, 예제 03이 근거입니다.
  - `cargo tree`로 두 버전 확인하기: 컴파일러의 help가 권하는 도구입니다.

### 2.5 기억할 것

- 호환되지 않는 두 버전은 한 프로그램에 함께 들어올 수 있고, 컴파일러에게는 이름만 같은 다른 크레이트입니다. 값이 공개 API를 건너갈 때 깨집니다.
- 에러에 `multiple different versions of crate` note가 보이면 타입이 아니라 버전을 의심하고 `cargo tree`로 확인합니다.
- 공개 API에 타입이 드러나는 의존성은 `pub use`로 재내보냅니다. 사용자가 값을 만들거나 트레이트를 구현해야 하면 크레이트 전체를 내보냅니다.
- 재내보내도 결합은 그대로입니다. 공개 의존성의 메이저를 올리면 내 메이저도 올라가므로, 드러내기 전에 뉴타입으로 감출 수 있는지 먼저 따집니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행합니다. `projects/`는 해당 디렉터리로 이동해 실행합니다. 출력에 든 절대 경로는 `<저장소>/…`로 바꿔 적었습니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_same_name_different_types.rs`](examples/01_same_name_different_types.rs) | 원리 | 다른 버전의 같은 이름 타입 | `pub use`로 같은 타입에 닿는 경로 | 2.1, 2.2 |
| [`02_re_export_type_or_crate.rs`](examples/02_re_export_type_or_crate.rs) | 1 | 타입만 재내보냄 | 크레이트 전체를 재내보냄 | 2.2 |
| [`03_hide_behind_newtype.rs`](examples/03_hide_behind_newtype.rs) | 2 | 의존성을 드러낸 채 메이저를 올림 | 뉴타입으로 감춤 | 2.3 |
| [`projects/two_versions`](projects/two_versions/Cargo.toml) | 1 | `cafe-mixed-fails`(일부러 실패) | `cafe-re-exported`, `cafe-renamed` | 2.1, 2.2, 4.1, 4.2 |

### 01_same_name_different_types — 같은 이름, 다른 타입

```sh
cargo run -p er-ch24-re-export-dependencies --example 01_same_name_different_types
```

```text
1) 정의가 같아도 출처가 다르면 다른 타입
   01_same_name_different_types::money_0_3::Money
   01_same_name_different_types::money_0_4::Money
   같은 타입인가? false
2) pub use는 같은 타입에 닿는 경로를 하나 더 만든다
   receipt::money::Money = money_0_3::Money? true
   receipt::money::Money = money_0_4::Money? false
   카페라테: 5000원
3) 컴파일러가 잡지 못하는 곳: 다운캐스트는 실행 중에 실패
   0.3의 Money로 꺼내기: None
   0.4의 Money로 꺼내기: Some(Money { won: 4500 })
4) 경계에서 원시 타입으로 옮겨 담으면 두 버전이 함께 있어도 문제없음
   마들렌: 3200원
```

한 파일 안에서는 E0308 메시지가 ``expected `money_0_3::Money`, found `money_0_4::Money` ``로 모듈 경로까지 보여 줍니다(파일의 주석 처리한 줄). 실제 크레이트 두 버전에서는 경로가 둘 다 `money::Money`라서 4.1절처럼 note가 필요합니다.

### 02_re_export_type_or_crate — 타입만, 또는 크레이트 전체

```sh
cargo run -p er-ch24-re-export-dependencies --example 02_re_export_type_or_crate
```

```text
타입만 재내보낸 receipt
  7700원 (부가세 700원 포함)
크레이트 전체를 재내보낸 receipt
  합계 12700원
```

타입만 재내보낸 판의 사용자는 receipt가 돌려준 합계를 `Money`라는 이름으로 받아 다루지만, 품목은 정수로만(`add_won`) 넣을 수 있습니다. 크레이트 전체를 재내보낸 판의 사용자는 `money::won`으로 값을 만들고 `money::Priced`를 구현해 품목 셋을 넣습니다.

### 03_hide_behind_newtype — 드러내기와 감추기

```sh
cargo run -p er-ch24-re-export-dependencies --example 03_hide_behind_newtype
```

```text
드러낸 판
  receipt 1.1.0 (money 0.3): 7700원
감춘 판
  receipt 1.0.0 (money 0.3): 7700원
  receipt 1.1.0 (money 0.4): 7700원
```

`main`의 주석 처리한 `exposed_user_code!(receipt_exposed_2_0)`을 풀면 2.3절의 E0599가 납니다.

### projects/two_versions — 두 버전이 한 그래프에 들어온 Cargo 프로젝트

```text
two_versions/
├── .cargo/config.toml     # crates.io 대신 vendor/를 쓰게 함(연습용 레지스트리)
├── Cargo.toml             # 가상 워크스페이스: 빌드에 성공하는 cafe 셋만 멤버
├── vendor/
│   ├── money-0.3.2/       # 연습용 레지스트리에 게시된 money 두 버전
│   └── money-0.4.0/
├── receipt-1.0.0/         # money = "0.3", 재내보내지 않음 (path 의존성 스냅숏)
├── receipt-1.1.0/         # 1.0.0 + `pub use money;`
├── cafe-separate/         # receipt 1.0.0 + money 0.4, 값을 넘기지 않음 → 빌드됨
├── cafe-mixed-fails/      # receipt 1.0.0 + money 0.4, 값을 넘김 → 일부러 실패(E0308, E0277)
├── cafe-renamed/          # receipt 1.0.0 + money 0.4 + money03(0.3) → 빌드됨
└── cafe-re-exported/      # receipt 1.1.0 + money 0.4, receipt::money로 넘김 → 빌드됨
```

`.cargo/config.toml`의 내용은 ch21의 `projects/version_requirements`와 같은 방식입니다(점으로 시작하는 파일이라 사이트에는 실리지 않습니다).

```toml
[source.crates-io]
replace-with = "practice-registry"

[source.practice-registry]
directory = "vendor"
```

receipt 두 스냅숏은 패키지 이름이 같아 한 워크스페이스의 멤버가 될 수 없으므로 워크스페이스의 `exclude`에 두고 각자 빈 `[workspace]`를 가집니다(ch23의 `glob_upgrade`와 같은 구성). 두 receipt의 `src/lib.rs`는 모듈 문서 주석의 버전과 설명, `pub use money;`와 그 문서 주석, `VERSION`만 다릅니다.

```sh
cd appd-a/ch24-re-export-dependencies/projects/two_versions
cargo run -q -p cafe-separate
cargo run -q -p cafe-renamed
cargo run -q -p cafe-re-exported
cd cafe-mixed-fails && cargo build   # 일부러 실패합니다(종료 코드 101, 4.1·4.2절)
```

```text
[cafe-separate] receipt 1.0.0, 앱의 money 0.4.0
  관광객 메뉴판의 아메리카노: $3.50
  아메리카노: 4,500원
  마들렌: 3,200원
  합계: 7,700원
  합계의 타입:     money::Money
  앱의 Money 타입: money::Money
[cafe-renamed] receipt 1.0.0, money03 0.3.2, money 0.4.0
  관광객 메뉴판의 아메리카노: $3.50
  아메리카노: 4,500원
  마들렌: 3,200원
  카페라테: 5,000원
  합계: 12,700원
[cafe-re-exported] receipt 1.1.0, receipt의 money 0.3.2, 앱의 money 0.4.0
  관광객 메뉴판의 아메리카노: $3.50
  아메리카노: 4,500원
  마들렌: 3,200원
  카페라테: 5,000원
  합계: 12,700원
```

## 4. 자주 하는 실수

4.1·4.2절은 `cafe-mixed-fails`에서 `cargo build`를 실행한 출력입니다. `<프로젝트>`는 `<저장소>/appd-a/ch24-re-export-dependencies/projects/two_versions`를 줄인 것입니다. 4.3절의 `compile_fail/` 파일은 챕터 디렉터리에서 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch24-re-export-dependencies
rustc --edition 2024 compile_fail/e0433_type_only_re_export.rs --out-dir /tmp/er-ch24
```

### 4.1 다른 버전의 값을 넘김 — E0308

파일: [`projects/two_versions/cafe-mixed-fails`](projects/two_versions/cafe-mixed-fails/src/main.rs)

```text
error[E0308]: mismatched types
  --> src/main.rs:29:21
   |
29 |     bill.add("마들렌", money::won(3200)); // E0308: money 0.4의 Money를 넘김
   |          ---           ^^^^^^^^^^^^^^^^ expected `money::Money`, found `Money`
   |          |
   |          arguments to this method are incorrect
   |
note: there are multiple different versions of crate `money` in the dependency graph
  --> <프로젝트>/vendor/money-0.3.2/src/lib.rs:13:1
   |
13 | pub struct Money {
   | ^^^^^^^^^^^^^^^^ this is the expected type
   |
  ::: <프로젝트>/vendor/money-0.4.0/src/lib.rs:22:1
   |
22 | pub struct Money {
   | ---------------- this is the found type
   = help: you can use `cargo tree` to explore your dependency tree
```

- **읽는 법**: 첫 줄의 `money::Money`와 `Money`는 같은 타입을 다르게 적은 것처럼 보입니다. 단서는 note입니다. 기대한 타입은 `vendor/money-0.3.2`, 받은 타입은 `vendor/money-0.4.0`에 정의되어 있다고 경로에 버전이 드러납니다. 실제 crates.io 의존성이라면 이 경로가 Cargo의 레지스트리 캐시 안의 `money-0.3.2/`, `money-0.4.0/`으로 나옵니다.
- **원인**: receipt 1.0.0은 money 0.3의 `Money`를 받는데, 앱의 `money::won`은 0.4의 `Money`를 만듭니다.
- **해결**: 라이브러리가 의존성을 재내보내면 `receipt::money::won(3200)`으로 바꿉니다(`cafe-re-exported`). 라이브러리를 고칠 수 없으면 2.2절의 "라이브러리를 고칠 수 없을 때"를 따릅니다. 정수로 옮겨 담아 `add_won("마들렌", 3200)`을 쓰는 것도 방법입니다.

### 4.2 다른 버전의 트레이트를 구현함 — E0277

같은 빌드의 두 번째 에러입니다(마지막 note는 생략했습니다).

```text
error[E0277]: the trait bound `Latte: money::Priced` is not satisfied
  --> src/main.rs:30:19
   |
30 |     bill.add_item(&Latte); // E0277: money 0.4의 Priced만 구현함
   |          -------- ^^^^^^ unsatisfied trait bound
   |          |
   |          required by a bound introduced by this call
   |
help: the trait `money::Priced` is not implemented for `Latte`
  --> src/main.rs:7:1
   |
 7 | struct Latte;
   | ^^^^^^^^^^^^
note: there are multiple different versions of crate `money` in the dependency graph
  --> <프로젝트>/vendor/money-0.3.2/src/lib.rs:53:1
   |
53 | pub trait Priced {
   | ^^^^^^^^^^^^^^^^ this is the expected trait
   |
  ::: <프로젝트>/vendor/money-0.4.0/src/lib.rs:73:1
   |
73 | pub trait Priced {
   | ---------------- this is the found trait
   = help: you can use `cargo tree` to explore your dependency tree
```

- **읽는 법**: help는 `Latte`에 `money::Priced`가 구현되지 않았다고 하지만, 소스에는 `impl money::Priced for Latte`가 있습니다. 구현이 "있는데 없다"고 하면 버전을 의심합니다. note의 `expected trait`과 `found trait`이 두 버전의 `Priced`를 가리킵니다. 책이 이 에러를 도움이 되지 않는다고 한 것은 이 note가 없던 때의 이야기입니다(2.4절).
- **원인**: 앱은 0.4의 `Priced`를 구현했고, `add_item`의 바운드는 0.3의 `Priced`입니다. 두 트레이트는 이름과 메서드가 같아도 다른 트레이트입니다.
- **해결**: 재내보낸 경로의 트레이트를 구현합니다(`impl receipt::money::Priced for Latte`). `price`가 돌려주는 값도 `receipt::money::won(5000)`으로 만들어야 합니다.

### 4.3 타입만 재내보내 값을 만들 수 없음 — E0433

파일: [`compile_fail/e0433_type_only_re_export.rs`](compile_fail/e0433_type_only_re_export.rs) (help 두 개는 생략했습니다)

```text
error[E0433]: cannot find `money` in `receipt`
  --> compile_fail/e0433_type_only_re_export.rs:43:26
   |
43 |     let latte = receipt::money::won(5000); // E0433
   |                          ^^^^^ could not find `money` in `receipt`
   |
help: a struct with a similar name exists
   |
43 -     let latte = receipt::money::won(5000); // E0433
43 +     let latte = receipt::Money::won(5000); // E0433
   |
```

- **읽는 법**: 사용자는 money 문서의 `money::won(5000)`을 receipt 경로로 옮겨 적었지만, receipt는 `Money` 타입만 재내보냈습니다.
- **원인**: 생성자 `won`이 `Money`의 연관 함수가 아니라 money의 크레이트 수준 함수라서, 타입을 재내보낸 것만으로는 닿을 수 없습니다.
- **해결**: 첫 help(`receipt::Money::won`)를 따르면 `Money`에 연관 함수 `won`이 없어 E0599로 바뀔 뿐입니다. 생략한 나머지 help 둘(`use crate::money;`를 더해 `money::won`으로 쓰라는 것)은 한 파일 안에 money가 있어서 나오는 것이고, receipt와 사용자를 실제 두 크레이트로 나눠 컴파일하면 첫 help만 나옵니다. 이것은 라이브러리 쪽의 실수이므로 receipt가 `pub use money;`로 크레이트 전체를 재내보내야 합니다.

## 5. 연습 문제

### 연습 1. 재내보낸 경로로 고치기

`projects/two_versions`에서 `cafe-mixed-fails`를 receipt 1.1.0을 쓰도록 고쳐 `cargo run`이 성공하게 만드세요. 앱의 달러 가격 줄은 그대로 money 0.4를 써야 합니다. `cafe-re-exported`는 보지 않고 해 봅니다.

<details>
<summary>힌트</summary>

`Cargo.toml`의 `receipt` 경로를 `../receipt-1.1.0`으로 바꾸고, `src/main.rs`에서 receipt로 넘어가는 것만 바꿉니다. `impl money::Priced for Latte`, `fn price(&self) -> money::Money`, `money::won(5000)`, `money::won(3200)`의 `money::`를 `receipt::money::`로 바꾸고 `money::usd(350)`은 둡니다. 출력의 품목 줄은 `cafe-re-exported`와 같고 합계는 `12,700원`입니다. 끝나면 파일을 원래대로 돌려놓습니다.

</details>

### 연습 2. 재내보낸 크레이트를 글롭으로 가져오면

`cafe-re-exported/src/main.rs`의 맨 위에 `use receipt::*;`를 더하고 `cargo run -p cafe-re-exported`를 실행하세요. 어떤 에러가 나나요? 그 줄(`money::usd(350)`이 있는 `println!`)을 주석으로 막고 다시 실행하면 첫 줄은 어떻게 바뀌고, cargo는 어떤 경고를 내나요?

<details>
<summary>힌트</summary>

글롭이 receipt의 `money`(0.3)를 들여와 앱의 `money` 크레이트를 가리므로 ``error[E0425]: cannot find function `usd` in crate `money` ``가 납니다. 그 줄을 막으면 빌드되고 첫 줄이 `… 앱의 money 0.3.2`로 바뀝니다. 앱의 `money::…`가 모두 0.3을 가리키게 되어 0.4를 아무도 쓰지 않으므로 cargo가 ``warning: unused dependency `money` ``(``cargo::unused_dependencies` is set to `warn` by default``)를 냅니다. `use receipt::*;`를 지우면 원래대로 돌아옵니다.

</details>

### 연습 3. 같은 버전인데도 다른 타입

`cafe-renamed/Cargo.toml`의 `money03`을 `{ package = "money", path = "../vendor/money-0.3.2" }`로 바꾸고 `cargo build -p cafe-renamed`를 실행하세요. receipt가 쓰는 money도 0.3.2인데 왜 에러가 나는지 note를 읽고 설명하세요. 끝나면 원래대로 돌려놓습니다.

<details>
<summary>힌트</summary>

E0308과 E0277이 다시 나고, E0308의 첫 줄은 ``expected `money::Money`, found a different `money::Money` ``입니다. note의 두 위치가 같은 파일(`vendor/money-0.3.2/src/lib.rs:13:1`)을 한 번은 절대 경로로, 한 번은 상대 경로로 가리킵니다. 같은 소스가 레지스트리 의존성(receipt의 `"0.3"`)과 path 의존성(`money03`)으로 두 번 컴파일되어 서로 다른 크레이트가 된 것입니다. Cargo는 출처가 다른 패키지를 합치지 않으므로 버전 번호가 같아도 두 사본이 생깁니다. `cargo tree -p cafe-renamed`에서 `money v0.3.2`가 한 번은 경로와 함께, 한 번은 경로 없이 나옵니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 24: Re-export dependencies whose types appear in your API](https://effective-rust.com/re-export.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- [Rust API Guidelines: Necessities](https://rust-lang.github.io/api-guidelines/necessities.html#public-dependencies-of-a-stable-crate-are-stable-c-stable): C-STABLE(안정 크레이트의 공개 의존성은 안정적이어야 함)과 공개 의존성이 숨어드는 `impl From` 예
- The Cargo Book
  - [Dependency Resolution](https://doc.rust-lang.org/cargo/reference/resolver.html): 호환되는 버전을 합치는 규칙, 호환되지 않는 버전이 함께 빌드되는 경우와 [Version-incompatibility hazards](https://doc.rust-lang.org/cargo/reference/resolver.html#version-incompatibility-hazards)(다운캐스트가 실행 중에 실패하는 예, `cargo tree -d`)
  - [Specifying Dependencies: Renaming dependencies in `Cargo.toml`](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml): `package` 키와 여러 버전에 의존하기
- The Rust Reference: [Use declarations: `use` Visibility](https://doc.rust-lang.org/reference/items/use-declarations.html#use-visibility): `pub use`가 이름을 다른 정의로 다시 내보낸다는 설명
- 에러 설명: `rustc --explain E0308`, `rustc --explain E0277`, `rustc --explain E0433`
