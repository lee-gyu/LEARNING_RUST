# ch12. 제네릭과 트레이트 객체의 득실 따지기

트레이트로 묶은 동작을 받는 방법은 두 가지입니다. `fn f<T: Trait>(x: &T)`처럼 제네릭으로 받으면 쓰인 구체 타입마다 함수가 한 벌씩 만들어지고, `fn f(x: &dyn Trait)`처럼 트레이트 객체로 받으면 함수는 한 벌이고 실행 중에 vtable을 거쳐 메서드를 찾습니다. 두 방법의 문법은 본편에서 배웠습니다. 이 챕터는 **같은 일을 하는 제네릭판과 `dyn`판을 나란히 놓고 무엇이 달라지는지** 확인한 뒤, 무엇을 기준으로 고를지 정리합니다.

앞 챕터들이 이 챕터로 넘긴 내용도 여기서 다룹니다. [ch2](../ch2-express-common-behavior/README.md)가 넘긴 두 방법의 비용 비교와 dyn 호환 조건, [ch5](../ch5-type-conversions/README.md)가 넘긴 "제네릭 부분은 `into()` 한 줄" 모양과 `where Self: Sized`, [ch8](../ch8-references-pointers/README.md)과 [ch9](../ch9-iterator-transforms/README.md)가 넘긴 제네릭 매개변수의 코드 크기입니다.

- 원제: [Item 12: Understand the trade-offs between generics and trait objects](https://effective-rust.com/generics.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch12-generics-vs-trait-objects`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 본편 챕터의 내용은 안다고 보고 다시 설명하지 않습니다.

- [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.4절(단형성화), 2.9절(트레이트 바운드), 2.11절(포괄 구현)
- [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 2.4~2.10절: 트레이트 객체, 정적·동적 디스패치, vtable, dyn 호환
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.11절(슈퍼트레이트), 2.15절(`Sized`와 `?Sized`)
- [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.4절: 얇은 포인터와 넓은 포인터

이 챕터의 크기와 함수 개수는 macOS aarch64, rustc 1.100.0에서 잰 값입니다. `type_name`이 돌려주는 문자열은 진단용이라 형식이 보장되지 않으므로 툴체인이 바뀌면 달라질 수 있습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 같은 함수의 제네릭판과 `dyn`판이 각각 어떤 코드로 컴파일되는지 설명하고, 코드 크기·컴파일 시간·호출 속도의 득실을 근거로 기본 선택이 제네릭인 이유를 말할 수 있다.
2. 단형성화로 늘어나는 코드를 줄이기 위해 제네릭 부분을 얇은 껍데기로 두고 실제 일은 제네릭이 아닌 함수에 맡기는 코드를 작성할 수 있고, 그 효과를 빌드한 바이너리에서 확인할 수 있다.
3. 트레이트 여러 개를 함께 요구할 때 제네릭은 `+` 바운드로, 트레이트 객체는 조합 트레이트와 포괄 구현으로 표현할 수 있고, 슈퍼트레이트가 상속이 아니라 "이것도 구현한다"는 조건임을 설명할 수 있다.
4. 여러 구체 타입의 값을 한 컬렉션에 담아야 할 때 제네릭, 트레이트 객체, 열거형 중 무엇을 쓸지 집합이 열려 있는지로 판단할 수 있다.
5. 트레이트 객체로 쓸 트레이트를 dyn 호환으로 설계하고, 제네릭 메서드와 `Self`를 돌려주는 메서드를 `where Self: Sized`로 남겨 둘 수 있다.

## 2. 핵심 개념

2.1절에서 두 판의 차이를 표로 본 뒤, 2.2~2.6절에서 지침마다 문제 상황과 두 형태를 비교합니다. 이 Item은 득실을 따지는 내용이라 2.2절과 2.4절은 "비권장/권장" 대신 제네릭판과 `dyn`판을 나란히 두고 어느 상황에서 어느 쪽이 나은지 봅니다. 예제는 모두 도서관 자료를 다루는 작은 트레이트 `Loanable`(제목과 대출 기간)을 씁니다.

### 2.1 지도: 제네릭판과 `dyn`판

| | 제네릭판 `fn f<T: Loanable>(x: &T)` | `dyn`판 `fn f(x: &dyn Loanable)` |
| --- | --- | --- |
| 만들어지는 함수 | 쓰인 구체 타입마다 한 벌(단형성화) | 한 벌 |
| 부를 메서드가 정해지는 때 | 컴파일할 때(정적 디스패치) | 실행 중, vtable에서(동적 디스패치) |
| 넘기는 인자 | 얇은 포인터 `&Book` (8바이트) | 넓은 포인터 `&dyn Loanable` (16바이트) |
| 코드 크기와 컴파일 시간 | 쓰인 타입 수만큼 늘어남 | 늘지 않음 |
| 호출 | 직접 호출, 인라인 가능 | 포인터를 두 번 따라간 간접 호출, 대개 인라인 불가 |
| 바운드 여러 개 | `T: Loanable + Debug` | 조합 트레이트가 필요 (2.4절) |
| 여러 구체 타입 섞어 담기 | 불가 | 가능 (2.5절) |
| 트레이트에 대한 조건 | 없음 | dyn 호환이어야 함 (2.6절) |

고르는 순서는 다음과 같습니다.

1. 호출 한 번에 구체 타입 하나를 다룬다면 **제네릭**이 기본입니다(2.2절).
2. 단형성화로 늘어난 코드 크기나 컴파일 시간이 실제로 문제라면, 제네릭 부분을 얇게 줄이거나 안쪽을 트레이트 객체로 받습니다(2.3절).
3. 여러 구체 타입을 섞어 담거나 실행 중에야 타입이 정해진다면, 종류가 열려 있으면 **트레이트 객체**, 닫혀 있으면 **열거형**입니다(2.5절).
4. 트레이트 객체로 쓸 트레이트라면 처음부터 dyn 호환으로 설계합니다(2.6절).

### 2.2 지침 1: 기본은 제네릭으로, 비용을 알고 고른다

**문제 상황.** 자료를 받아 "며칠 뒤 반납"이라는 안내문을 만드는 함수가 있습니다. 책은 14일, DVD는 7일이고, 둘 다 `Loanable`을 구현합니다. 매개변수를 제네릭으로 받을지 트레이트 객체로 받을지 정해야 합니다.

**제네릭판.**

```rust
fn due_note<T: Loanable>(item: &T) -> String {
    format!("{} → {}일 뒤 반납 (본문이 아는 타입: {})",
        item.title(), item.loan_days(), short(type_name::<T>()))
}
```

**`dyn`판.**

```rust
fn due_note_dyn(item: &dyn Loanable) -> String {
    format!("{} → {}일 뒤 반납 (본문이 아는 타입: {})",
        item.title(), item.loan_days(), short(type_name_of_val(item)))
}
```

[`01_generic_vs_dyn.rs`](examples/01_generic_vs_dyn.rs)에서 두 판에 책과 DVD를 넘기면 안내문은 똑같습니다. 다른 것은 괄호 안의 "본문이 아는 타입"입니다. 제네릭판의 본문은 `Book`, `Dvd`를 알고, `dyn`판의 본문은 `dyn Loanable`밖에 모릅니다. 제네릭판은 `due_note::<Book>`과 `due_note::<Dvd>`라는 별개의 함수 두 개로 컴파일되어 각 사본 안에서 `T`가 이미 구체 타입이기 때문입니다. `dyn`판은 한 벌뿐이라 실제 타입은 넓은 포인터에 붙은 vtable만 압니다.

함수가 몇 개 만들어졌는지는 빌드한 바이너리의 함수 이름 목록(심볼)으로 직접 볼 수 있습니다. 저장소 루트에서 다음을 실행합니다. `nm`은 바이너리의 심볼을 나열하는 개발 도구이고, `-C`는 컴파일러가 바꿔 적은(맹글링한) 이름을 Rust 경로로 되돌려 보여 줍니다. 이 챕터의 `nm` 결과는 macOS의 `nm`으로 확인했습니다.

```sh
cargo build -p er-ch12-generics-vs-trait-objects --examples
nm -C target/debug/examples/01_generic_vs_dyn | grep due_note
```

이 환경의 디버그 빌드에서 주소를 빼고 옮긴 결과입니다(줄 순서는 주소 순이라 다를 수 있습니다).

```text
t 01_generic_vs_dyn::due_note::<01_generic_vs_dyn::Dvd>
t 01_generic_vs_dyn::due_note::<01_generic_vs_dyn::Book>
t core::any::type_name_of_val::<01_generic_vs_dyn::due_note<01_generic_vs_dyn::Dvd>>
t core::any::type_name_of_val::<01_generic_vs_dyn::due_note<01_generic_vs_dyn::Book>>
t core::any::type_name_of_val::<01_generic_vs_dyn::due_note_dyn>
t 01_generic_vs_dyn::due_note_dyn
```

`due_note`는 두 벌, `due_note_dyn`은 한 벌입니다. 표준 라이브러리의 제네릭 함수 `type_name_of_val`도 예제가 넘긴 타입마다 세 벌이 만들어졌습니다. 단형성화는 내 코드에만 일어나는 일이 아닙니다.

**비용 비교.** 책은 세 가지를 비교합니다.

- **코드 크기.** 제네릭 함수는 쓰인 타입마다 본문이 복사되므로 바이너리가 커질 수 있습니다. `dyn`판은 한 벌입니다.
- **호출 속도.** `dyn`판의 호출은 넓은 포인터에서 vtable 주소를 읽고, vtable에서 함수 주소를 읽어 부르므로 포인터를 두 번 따라갑니다. 컴파일러가 부를 함수를 모르니 본문을 호출 자리에 펼쳐 넣는 인라인도 대개 할 수 없습니다. 제네릭판은 직접 호출이라 조금 빠릅니다.
- **컴파일 시간.** 제네릭은 만들 함수가 많아져 컴파일과 링크가 오래 걸리는 경향이 있습니다.

책의 결론은 이 차이가 대부분의 프로그램에서 크지 않으므로 **측정으로 문제가 확인되기 전에는 성능을 근거로 고르지 않는다**는 것입니다. 실행 시간 측정은 `ch20-avoid-over-optimizing`에서 다룹니다. 성능 말고도 제네릭이 기본인 이유가 있습니다. 바운드를 `+`로 자유롭게 조합할 수 있고(2.4절), 트레이트에 dyn 호환 조건이 붙지 않으며(2.6절), 본문이 구체 타입을 알기 때문에 `T::default()`처럼 타입에 딸린 연관 함수도 부를 수 있습니다.

단형성화되는 것은 함수의 코드입니다. 제네릭 함수 안에 `static` 변수를 두어도 타입마다 따로 생기지 않고 하나뿐입니다. `fn bump<T>(_: T) -> u32` 안의 `static` 카운터를 `u8`, `&str`, `u8` 순서로 부르면 1, 2, 3이 나옵니다(이 툴체인에서 확인). 그래서 `static`으로 "인스턴스마다 따로 센다"는 관찰은 할 수 없습니다.

**제네릭판의 두 문법.** `fn f<T: Loanable>(x: &T)`와 `fn f(x: &impl Loanable)`는 거의 같지만 하나가 다릅니다. 이름 붙은 타입 매개변수는 부르는 쪽이 `f::<Book>(..)`처럼 터보피시로 타입을 적을 수 있고, 매개변수 자리의 `impl Trait`는 적을 수 없습니다. 적으면 1.100.0에서는 다음 에러가 납니다(핵심 줄만 옮겼습니다).

```text
error[E0107]: function takes 0 generic arguments but 1 generic argument was supplied
  = note: `impl Trait` cannot be explicitly specified as a generic argument
```

인자에서 타입이 추론되면 상관없지만, 부르는 쪽이 타입을 직접 정해야 하는 함수라면 이름 붙은 타입 매개변수로 선언합니다.

**지침을 따르지 않아도 되는 경우.** 책은 트레이트 객체가 나은 경우를 셋 듭니다.

- 코드 크기나 컴파일 시간이 실제로 문제일 때. 제네릭을 통째로 버리기 전에 2.3절의 방법을 먼저 봅니다.
- 여러 구체 타입의 값을 한 컬렉션에 담을 때(2.5절).
- 타입이 실행 중에야 정해질 때. 설정에서 읽은 대로 구현을 고르는 경우는 [ch2](../ch2-express-common-behavior/README.md) 2.6절에서 봤습니다. 실행 중에 공유 라이브러리를 불러오는 플러그인처럼 컴파일할 때 소스가 아예 없는 코드라면 단형성화할 수 없으므로 트레이트 객체가 유일한 방법입니다.

### 2.3 지침 2: 단형성화 비용이 문제라면 제네릭 껍데기를 얇게 만든다

**문제 상황.** 서가 라벨을 만드는 함수가 제목을 `&str`, `&String`, `Box<str>`, `Rc<str>`, `Cow<str>`로 받을 수 있게 `S: AsRef<str>` 바운드를 썼습니다. 본문은 공백을 정리하고, 단어의 첫 글자를 모으고, 글자 수를 셉니다.

**비권장 코드.** 본문 전체가 제네릭입니다.

```rust
fn shelf_label_fat<S: AsRef<str>>(title: S) -> String {
    let title = title.as_ref();      // S에 기대는 곳은 이 줄뿐
    let words: Vec<&str> = title.split_whitespace().collect();
    let initials: String = words.iter().filter_map(|w| w.chars().next()).collect();
    // ...
}
```

본문이 `S`에 기대는 곳은 첫 줄의 `as_ref()`뿐인데, 다섯 타입으로 부르면 본문 전체가 다섯 벌 만들어집니다.

**권장 코드.** 제네릭 부분은 변환 한 줄로 줄이고, 실제 일은 제네릭이 아닌 함수에 맡깁니다.

```rust
fn shelf_label<S: AsRef<str>>(title: S) -> String {
    shelf_label_str(title.as_ref())
}

fn shelf_label_str(title: &str) -> String { /* 비권장 코드와 같은 본문 */ }
```

[ch5](../ch5-type-conversions/README.md) 2.3절의 `fill(color: impl Into<Rgb>)`가 본문을 `fill_exact(color.into())` 한 줄로 둔 것도 같은 모양입니다. 변환이 `as_ref()`든 `into()`든, 제네릭 부분에는 변환만 남깁니다.

[`02_thin_generic_shell.rs`](examples/02_thin_generic_shell.rs)의 1번 구역에서 두 꼴의 결과는 같습니다. 차이는 바이너리에 있습니다. 2.2절과 같은 방법으로 디버그 빌드의 함수 이름을 세어 봅니다.

```sh
nm -C target/debug/examples/02_thin_generic_shell | grep -c shelf_label_fat
nm -C target/debug/examples/02_thin_generic_shell | grep -c 'shelf_label::'
nm -C target/debug/examples/02_thin_generic_shell | grep -c shelf_label_str
```

| 찾은 이름 | 줄 수 | 뜻 |
| --- | --- | --- |
| `shelf_label_fat` | 70 | 타입 5개 × 함수 14개 |
| `shelf_label::` | 5 | 타입마다 한 줄짜리 껍데기 |
| `shelf_label_str` | 14 | 본문과 그것이 부르는 함수들, 한 벌 |

본문 하나가 함수 14개가 된 이유가 중요합니다. 제네릭 함수 안에서 만든 클로저 `|w| w.chars().next()`는 `S`를 쓰지 않는데도 `S`마다 다른 타입이 됩니다(`nm` 출력에 `shelf_label_fat<&str>::{closure#0}`처럼 나옵니다). 그래서 그 클로저를 받는 `filter_map`, `collect`, `extend`, `fold` 같은 표준 라이브러리의 제네릭 함수도 `S`마다 다시 단형성화됩니다. 제네릭 함수의 본문이 길수록, 안에서 반복자와 클로저를 많이 쓸수록 복사되는 양은 생각보다 빨리 불어납니다.

**안쪽을 트레이트 객체로 받는 방법.** 변환할 구체 타입이 없을 때는 안쪽 함수가 트레이트 객체를 받게 합니다. 2번 구역의 `write_slip`은 출력 대상 `W: fmt::Write`를 받아 `&mut dyn fmt::Write`로 넘기기만 합니다.

```rust
fn write_slip<W: Write>(out: &mut W, title: &str, days: u32) -> fmt::Result {
    write_slip_dyn(out, title, days)   // &mut W → &mut dyn Write 강제 변환
}

fn write_slip_dyn(out: &mut dyn Write, title: &str, days: u32) -> fmt::Result { /* 본문 */ }
```

`String`과 `CharCounter` 두 타입으로 부르면, 본문 전체가 제네릭인 `write_slip_fat`은 두 벌이고, 얇은 꼴은 껍데기 두 벌과 본문 한 벌입니다(`grep -c`로 센 `write_slip_fat`, `write_slip::`, `write_slip_dyn`이 각각 2, 2, 1). 부르는 쪽은 여전히 `&mut String`을 그대로 넘기므로 편의는 잃지 않습니다. 대신 본문 안의 출력 호출은 동적 디스패치가 됩니다. 책이 "코드 크기가 문제면 트레이트 객체"라고 한 것을 함수 전체가 아니라 안쪽에만 적용한 셈입니다.

표준 라이브러리도 두 방법을 씁니다. 1.100.0 소스에서 `std::fs::read<P: AsRef<Path>>`의 본문은 안쪽에 정의한 `fn inner(path: &Path)`를 `inner(path.as_ref())`로 부르는 한 줄이고, `fmt::Write`의 `write!`가 서식 인자가 있을 때 결국 부르는 `core::fmt::write`는 `output: &mut dyn Write`를 받습니다.

릴리스 빌드에서는 이 예제의 함수들이 `main`에 인라인되어 이름이 거의 사라지므로, 위 방법으로는 셀 수 없습니다. 실제 프로그램에서 크기가 얼마나 줄었는지는 그 프로그램의 릴리스 빌드로 재 봐야 합니다.

**지침을 따르지 않아도 되는 경우.** 본문이 짧거나 부르는 타입이 한두 개라면 함수를 둘로 나눌 이유가 없습니다. 안쪽을 트레이트 객체로 받는 방법은 자주 불리는 경로에 동적 디스패치를 넣으므로, 코드 크기와 속도 중 무엇이 문제인지 먼저 재 봅니다. 이 모양은 clippy가 알려 주지 않습니다.

### 2.4 지침 3: 트레이트를 여럿 요구할 때, 제네릭은 `+`로, 트레이트 객체는 조합 트레이트로

**문제 상황.** 감사 기록에 자료의 대출 기간(`Loanable`)과 디버그 형식(`Debug`)을 함께 남기려 합니다. 함수가 두 트레이트를 모두 요구해야 합니다.

**제네릭판.** 바운드를 `+`로 이어 쓰면 끝입니다.

```rust
fn audit_line<T: Loanable + Debug>(item: &T) -> String {
    format!("{}일 | {:?}", item.loan_days(), item)
}
```

**`dyn`판.** 같은 모양의 `&(dyn Loanable + Debug)`는 E0225입니다(4.1절). 트레이트 객체의 vtable은 트레이트 하나를 기준으로 만들어지므로 `+` 뒤에는 메서드가 없는 자동 트레이트(`Send`, `Sync` 등)만 올 수 있습니다. 둘을 묶은 **조합 트레이트**를 만들고 포괄 구현을 붙여야 합니다. 컴파일러의 도움말도 같은 방법을 제안합니다.

```rust
trait AuditItem: Loanable + Debug {}               // 메서드 없음
impl<T: Loanable + Debug> AuditItem for T {}       // 두 트레이트를 구현한 타입은 모두 AuditItem

fn audit_line_dyn(item: &dyn AuditItem) -> String {
    format!("{}일 | {:?}", item.loan_days(), item)
}
```

[`03_combined_bounds.rs`](examples/03_combined_bounds.rs)의 2번 구역은 `[&dyn AuditItem; 2]`에 책과 DVD를 함께 담아 두 판과 같은 결과를 냅니다. `dyn AuditItem`의 vtable에는 두 슈퍼트레이트의 메서드가 모두 들어 있습니다.

**슈퍼트레이트는 상속이 아닙니다.** `trait AuditItem: Loanable + Debug`는 "`AuditItem`을 구현하려면 `Loanable`과 `Debug`도 구현해야 한다"는 조건입니다. `AuditItem`이 `Loanable`의 한 종류라는 뜻도, 구현을 물려준다는 뜻도 아닙니다. 예제의 `Book`과 `Dvd`에는 `impl AuditItem`이 한 줄도 없습니다. 포괄 구현이 "두 트레이트를 구현했다"는 사실만 보고 붙여 준 것입니다.

**업캐스트.** `&dyn AuditItem`을 `Loanable`만 필요한 `fn due_days(item: &dyn Loanable)`에 넘기면, 1.86.0부터는 슈퍼트레이트의 트레이트 객체로 바꾸는 강제 변환이 일어납니다. 3번 구역은 `&dyn AuditItem` → `&dyn Loanable`과 `Box<dyn AuditItem>` → `Box<dyn Loanable>`을 보여 줍니다. 책을 쓸 때(1.70)는 이 변환이 안 되어서, 제네릭 코드에서는 되는 일이 트레이트 객체에서는 안 된다는 것이 책의 지적이었습니다(2.7절). 업캐스트의 쓰임은 `ch19-avoid-reflection`에서 다룹니다.

**조합이 늘어나면.** 제네릭판은 함수마다 필요한 바운드를 적으면 됩니다. 예제 03 1번 구역의 `snapshot<T: Loanable + Clone>`은 대출 시점의 사본을 남깁니다. `dyn`판은 조합마다 트레이트와 포괄 구현을 하나씩 더 만들어야 하고, 어떤 조합은 아예 만들 수 없습니다. `Clone`은 `Sized`를 슈퍼트레이트로 가지므로 `trait SnapshotItem: Loanable + Clone`은 dyn 호환이 아닙니다(4.2절). 자동 트레이트는 예외라서 `&(dyn Loanable + Send + Sync)`처럼 조합 트레이트 없이 붙일 수 있습니다.

**어느 쪽이 나은가.** 바운드를 여러 개 요구하는 함수가 많고 조합이 함수마다 다르다면 제네릭이 훨씬 간단합니다. 트레이트 객체가 꼭 필요한 곳(2.5절)에서는 쓰는 조합을 조합 트레이트 하나로 정해 두고, 그 조합에 `Clone`처럼 `Sized`를 요구하는 트레이트가 끼지 않게 합니다.

### 2.5 지침 4: 여러 타입을 섞어 담을 때, 종류가 열려 있으면 트레이트 객체, 닫혀 있으면 열거형

**문제 상황.** 반납함에서 꺼낸 자료를 꺼낸 순서대로 카트 하나에 담습니다. 책과 DVD가 섞여 있고, 다른 팀이 만든 보드게임 대출 기능도 같은 카트를 씁니다.

제네릭 `Shelf<T: Loanable> { items: Vec<T> }`의 `T`는 `Shelf` 값 하나에 대해 구체 타입 하나로 정해지므로, 책 서가와 DVD 서가는 만들 수 있어도 섞인 카트는 만들 수 없습니다(섞으면 E0308, Part 0 ch17 2.5절). 여러 구체 타입의 값을 함께 담는 컬렉션, 즉 **이종 컬렉션(heterogeneous collection)** 을 만드는 방법은 둘입니다. [`04_heterogeneous_collection.rs`](examples/04_heterogeneous_collection.rs)가 세 형태를 차례로 보여 줍니다.

```rust
// 1. 한 종류만: 제네릭
let shelf: Shelf<Book> = Shelf { items: vec![book("러스트 입문"), /* .. */] };

// 2. 섞어 담고, 종류가 열려 있음: 트레이트 객체
let cart: Vec<Box<dyn Loanable>> = vec![
    Box::new(book("러스트 입문")),
    Box::new(dvd("우주 다큐", 12)),
    Box::new(games::BoardGame { title: "보드게임 카탄".to_string() }), // 다른 모듈의 타입
];

// 3. 섞어 담고, 종류가 닫혀 있음: 열거형
enum Media { Book(Book), Dvd(Dvd) }
impl Loanable for Media { /* 배리언트마다 match로 위임 */ }
let returns: Shelf<Media> = Shelf { items: vec![Media::Book(..), Media::Dvd(..)] };
```

| | `Shelf<Book>` | `Vec<Box<dyn Loanable>>` | `Shelf<Media>` |
| --- | --- | --- | --- |
| 섞어 담기 | 불가 | 가능 | 가능 |
| 새 종류 추가 | 해당 없음 | 새 타입에 `impl Loanable`만. 다른 크레이트에서도 가능 | 배리언트와 모든 `match`를 고침. 열거형을 정의한 크레이트만 가능 |
| 원소 하나의 크기(이 환경) | 24바이트(`Book`) | 16바이트(넓은 포인터) + 값마다 따로 할당 | 32바이트(가장 큰 배리언트 기준) |
| 그 타입에만 있는 정보 | 그대로 읽음 | 트레이트 메서드로만 | `match`로 꺼내 읽음(예제의 연령 제한) |
| 메서드 호출 | 정적 디스패치 | 동적 디스패치 | `match` 분기 |

트레이트 객체의 강점은 **열린 집합**입니다. 2번 구역의 `games::BoardGame`은 카트 코드가 모르는 모듈에서 `Loanable`을 구현했을 뿐인데 같은 카트에 들어갑니다. 책이 트레이트 객체의 전통적인 쓰임으로 드는 경우가 이것입니다.

종류가 **닫혀 있고** 모두 내 크레이트 안에 있다면 열거형이 세 번째 선택지입니다. 3번 구역의 `Media`는 `Loanable`을 구현하므로 1번의 제네릭 `Shelf<T>`를 그대로 씁니다. 열거형은 구체 타입 하나라서 섞인 값을 제네릭 코드에 넘길 수 있는 것입니다. 값은 `Vec` 버퍼 안에 그대로 있고, `match`로 DVD에만 있는 연령 제한을 꺼낼 수 있습니다. 트레이트 객체로 같은 정보를 보려면 트레이트에 메서드를 더하거나 다운캐스트(`ch19-avoid-reflection`)를 해야 합니다. 열거형과 트레이트 객체의 비교는 Part 0 ch17 2.10절에, 열거형으로 상태를 표현하는 지침은 [ch1](../ch1-express-data-structures/README.md)에 있습니다.

이종 컬렉션의 원소를 소유할지 빌릴지도 정합니다. 값이 다른 곳에 이미 있고 잠깐 모아 보기만 한다면 예제 03의 `[&dyn AuditItem; 2]`처럼 빌린 트레이트 객체로 담아 할당을 피합니다. 컬렉션이 값을 소유해야 하면 `Box<dyn Trait>`입니다(Part 0 ch17 2.7절, [ch8](../ch8-references-pointers/README.md) 2.3절).

**지침을 따르지 않아도 되는 경우.** 한 컬렉션에 한 종류만 담는다면 섞을 필요가 없으므로 제네릭이 맞습니다. 열거형이 닫힌 집합에 어울린다고 해도, 다른 크레이트가 새 종류를 더해야 하는 공개 라이브러리라면 트레이트 객체가 맞습니다. 공개 열거형에 배리언트를 더하는 일의 호환성은 `ch21-semver`에서 다룹니다.

### 2.6 지침 5: 트레이트 객체로 쓸 트레이트는 dyn 호환으로 설계하고, 나머지 메서드는 `where Self: Sized`로 남긴다

**문제 상황.** 예약한 자료가 도착하면 이용자에게 알리는 `Notifier` 트레이트를 문자(`Sms`)와 메일(`Email`)로 구현합니다. 채널을 섞어 `Vec<Box<dyn Notifier>>`에 담고 싶습니다. 구체 타입으로 쓰는 사람을 위해서는 문장 여러 개를 한 번에 보내는 제네릭 메서드 `send_each<I: IntoIterator>`와, 보내는 이름만 바꾼 사본을 돌려주는 `with_sender(&self, ..) -> Self`도 두고 싶습니다.

**비권장 코드.** 세 메서드를 그대로 트레이트에 둡니다.

```rust
trait Notifier {
    fn send(&self, to: &str, text: &str) -> String;
    fn send_each<I>(&self, to: &str, texts: I) -> Vec<String>
    where I: IntoIterator, I::Item: AsRef<str> { /* send를 여러 번 부르는 기본 구현 */ }
    fn with_sender(&self, sender: &str) -> Self;
}
```

`Vec<Box<dyn Notifier>>`를 만들면 E0038입니다(핵심 줄만 옮겼습니다).

```text
error[E0038]: the trait `Notifier` is not dyn compatible
  ...because method `send_each` has generic type parameters
  ...because method `with_sender` references the `Self` type in its return type
  = help: consider moving `send_each` to another trait
  = help: consider moving `with_sender` to another trait
  = help: the following types implement `Notifier`:
            Sms
            Email
          consider defining an enum where each variant holds one of these types,
          implementing `Notifier` for this new enum and using it instead
```

제네릭 메서드는 `I`마다 한 벌씩 만들어지므로 vtable의 칸 하나에 담을 수 없고, `Self`를 돌려주는 메서드는 트레이트 객체로 부르는 쪽이 돌려받을 값의 크기를 모릅니다(Part 0 ch17 2.9절). 컴파일러의 도움말은 메서드를 다른 트레이트로 옮기거나 열거형을 쓰라고 제안할 뿐, 메서드를 남기는 방법은 알려 주지 않습니다.

**권장 코드.** 트레이트 객체로 부를 필요가 없는 두 메서드에 `where Self: Sized`를 붙입니다.

```rust
trait Notifier {
    fn send(&self, to: &str, text: &str) -> String;     // vtable에 들어가는 핵심 메서드

    fn send_each<I>(&self, to: &str, texts: I) -> Vec<String>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        Self: Sized,                                      // 트레이트 객체에서 뺌
    { /* .. */ }

    fn with_sender(&self, sender: &str) -> Self
    where
        Self: Sized;                                      // 트레이트 객체에서 뺌
}
```

`Self: Sized`는 "크기를 아는 구체 타입에서만 부르는 메서드"라는 표시입니다. 이런 메서드는 vtable에 들어가지 않으므로 dyn 호환 검사에서 빠지고, 트레이트 전체는 dyn 호환이 됩니다. [`05_where_self_sized.rs`](examples/05_where_self_sized.rs)의 1번 구역은 구체 타입 `Sms`로 세 메서드를 모두 부르고, 2번 구역은 `Vec<Box<dyn Notifier>>`에 두 채널을 담아 `send`를 부릅니다. 트레이트 객체로 `send_each`를 부르면 에러 코드 없는 에러가 납니다(4.3절).

트레이트 객체를 쓰는 쪽에도 "여러 개 보내기"가 필요하다면 두 방법이 있습니다.

- **트레이트 밖에 `?Sized` 함수를 둡니다.** 3번 구역의 `fn send_each_any<N: Notifier + ?Sized, I>(notifier: &N, ..)`는 구체 타입과 `&dyn Notifier`를 모두 받습니다([ch2](../ch2-express-common-behavior/README.md) 2.6절의 `?Sized`). [ch5](../ch5-type-conversions/README.md) 연습 4의 힌트가 제안한 모양입니다.
- **포인터 타입에 트레이트를 다시 구현합니다.** 표준 라이브러리의 `Iterator`가 이렇게 설계되어 있습니다. `map`, `filter`, `sum` 같은 메서드에는 모두 `Self: Sized`가 붙어 있어 `dyn Iterator`가 가능하고, `impl<I: Iterator + ?Sized> Iterator for Box<I>`가 있어서 `Box<dyn Iterator>` 자체가 크기를 아는 `Iterator`가 됩니다. 4번 구역은 `Box<dyn Iterator<Item = u32>>`에 `map`과 `sum`을 부릅니다. `Notifier`에 같은 방법을 쓰는 것은 연습 4입니다.

dyn 호환을 깨는 것을 정리하면 다음과 같습니다. 이 툴체인에서 하나씩 컴파일해 확인했고, 전체 규칙은 The Reference의 Dyn compatibility 절에 있습니다.

| 트레이트에 있는 것 | E0038의 `...because` 문구 | 고치는 방법 |
| --- | --- | --- |
| 제네릭 메서드 | ``method `send_each` has generic type parameters`` | `where Self: Sized`, 또는 트레이트 밖 함수 |
| `Self`를 돌려주는 메서드(`Box<Self>`도 같음) | ``method `with_sender` references the `Self` type in its return type`` | `where Self: Sized` |
| `self`가 없는 연관 함수 `fn blank() -> Self` | ``associated function `blank` has no `self` parameter`` | `where Self: Sized` (컴파일러도 제안) |
| 연관 상수 `const LIMIT: u32;` | ``it contains associated const `LIMIT` `` | 메서드로 바꾸거나 다른 트레이트로 |
| `Sized`를 요구하는 슈퍼트레이트(`Clone`, `Default`) | ``it requires `Self: Sized` `` | 슈퍼트레이트에서 빼고 제네릭 바운드로 요구 |

수신자에는 `Self`를 써도 됩니다. `&self`, `&mut self`, `self: Box<Self>`, `self: Rc<Self>`, `self: Arc<Self>`, `self: Pin<&mut Self>`를 가진 메서드는 모두 트레이트 객체로 부를 수 있었습니다. 값으로 받는 `self` 메서드가 있어도 트레이트는 dyn 호환이지만, 그 메서드를 `Box<dyn Notifier>`에서 부르면 E0161(``cannot move a value of type `dyn Notifier` ``)입니다. 크기를 모르는 값을 옮길 수 없기 때문입니다. The Reference도 값으로 받는 `self`는 `Self: Sized`를 함께 뜻한다고 설명합니다.

**지침을 따르지 않아도 되는 경우.** 제네릭 바운드로만 쓰고 트레이트 객체로 쓸 일이 없는 트레이트라면 dyn 호환을 신경 쓸 필요가 없고, 모든 메서드에 `where Self: Sized`를 붙이면 읽을 것만 늘어납니다. 다만 공개 트레이트는 사용자가 트레이트 객체로 쓰기 시작할 수 있고, 나중에 dyn 호환이 아닌 메서드를 더하면 그 사용자의 코드가 깨집니다. 공개 API를 바꿀 때의 호환성은 `ch21-semver`에서, 트레이트에 기본 구현을 두는 지침은 `ch13-default-implementations`에서 다룹니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **업캐스트가 됩니다.** 책은 쓰는 시점에 트레이트 객체를 슈퍼트레이트의 트레이트 객체로 바꿀 수 없다고 적었습니다. 트레이트 객체 업캐스트는 1.86.0에서 안정화되어 `&dyn AuditItem` → `&dyn Loanable`, `Box<dyn AuditItem>` → `Box<dyn Loanable>`이 강제 변환으로 됩니다(예제 03의 3번 구역). 강제 변환 목록에서의 위치는 [ch5](../ch5-type-conversions/README.md) 2.6절에 있습니다.
- **object safety → dyn compatible.** 책은 "object safe"라고 부르지만, 현재 컴파일러와 The Reference는 "dyn compatible"이라고 부릅니다(2.6절의 E0038 메시지). 이 저장소는 "dyn 호환"으로 씁니다.
- **`where Self: Sized` 메서드를 트레이트 객체로 부를 때의 에러.** 책이 보여 준 것과 같은 ``the `..` method cannot be invoked on a trait object`` 문구이고, 지금도 에러 코드가 없습니다. `Self: Sized`가 적힌 곳을 ``this has a `Sized` requirement``로 가리킵니다(4.3절).
- **`impl Trait` 매개변수와 터보피시.** 책은 각주에서 매개변수 자리의 `impl Trait`에는 터보피시로 타입을 적을 수 없다고만 말합니다. 1.100.0에서 그렇게 하면 E0107과 ``note: `impl Trait` cannot be explicitly specified as a generic argument``가 나옵니다(2.2절).
- **책에 없는 지침.** 다음은 책의 Item 12가 명시적으로 권하지 않는 내용입니다(2026-11-19에 온라인판에서 확인).
  - 지침 2 전체(제네릭 껍데기를 얇게): 책은 코드 크기가 문제면 트레이트 객체를 쓰라고 할 뿐, 제네릭 함수를 껍데기와 안쪽 함수로 나누는 방법은 다루지 않습니다. ch5가 넘긴 판단이고, 1.100.0 표준 라이브러리 소스의 `fs::read`(안쪽 `fn inner`)와 `core::fmt::write`(`&mut dyn Write`)가 근거입니다.
  - 지침 4의 열거형: 책은 이종 컬렉션의 해법으로 트레이트 객체만 듭니다. E0038의 도움말(``consider defining an enum where each variant holds one of these types``), Part 0 ch17 2.10절, ch1이 근거입니다.
  - 지침 5의 제네릭 메서드에 붙이는 `where Self: Sized`, 트레이트 밖의 `?Sized` 함수, 포인터 타입에 다시 구현하기: 책은 `Self`를 돌려주는 메서드에 `where Self: Sized`를 붙이는 예만 보여 줍니다. The Reference의 Dyn compatibility 절(명시적으로 디스패치하지 않는 함수)과 표준 라이브러리 `Iterator`의 설계가 근거입니다.
- **달라지지 않은 점.** 책의 각주가 "나중에 풀릴 수도 있다"고 한 `Box<Self>`를 돌려주는 메서드의 제약은 1.100.0에서도 그대로입니다. `fn boxed_copy(&self) -> Box<Self>`가 있으면 ``...because method `boxed_copy` references the `Self` type in its return type``로 E0038입니다.

### 2.8 기억할 것

- 호출 한 번에 한 타입이면 제네릭이 기본입니다. 트레이트 객체는 코드 크기·컴파일 시간이 실제로 문제일 때, 섞어 담을 때, 타입이 실행 중에 정해질 때 고릅니다. 성능을 근거로 삼으려면 먼저 잽니다.
- 제네릭 함수는 본문 전체가, 안의 클로저와 그것을 받는 함수까지 타입마다 복사됩니다. 제네릭 부분은 변환 한 줄로 두고 실제 일은 제네릭이 아닌 함수에 맡깁니다.
- 바운드 여러 개는 제네릭이면 `T: A + B`, 트레이트 객체면 조합 트레이트와 포괄 구현입니다. 슈퍼트레이트는 상속이 아니라 "이것도 구현한다"는 조건입니다.
- 섞어 담을 종류가 열려 있으면 `Box<dyn Trait>`, 닫혀 있으면 열거형입니다.
- 트레이트 객체로 쓸 트레이트는 수신자 말고 `Self`가 없고 제네릭 메서드가 없게 설계하고, 구체 타입에서만 쓸 메서드에는 `where Self: Sized`를 붙입니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 두 형태(또는 세 형태)를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행하며, 모든 예제는 종료 코드 0으로 끝납니다.

| 파일 | 지침 | 비교하는 것 | 관련 절 |
| --- | --- | --- | --- |
| [`01_generic_vs_dyn.rs`](examples/01_generic_vs_dyn.rs) | 1 | `due_note<T: Loanable>`와 `due_note_dyn(&dyn Loanable)` | 2.2 |
| [`02_thin_generic_shell.rs`](examples/02_thin_generic_shell.rs) | 2 | 본문 전체가 제네릭인 함수와 얇은 껍데기 + 안쪽 함수 | 2.3 |
| [`03_combined_bounds.rs`](examples/03_combined_bounds.rs) | 3 | `T: Loanable + Debug`와 조합 트레이트 `dyn AuditItem`, 업캐스트 | 2.4 |
| [`04_heterogeneous_collection.rs`](examples/04_heterogeneous_collection.rs) | 4 | `Shelf<Book>`, `Vec<Box<dyn Loanable>>`, 열거형 `Shelf<Media>` | 2.5 |
| [`05_where_self_sized.rs`](examples/05_where_self_sized.rs) | 5 | 구체 타입과 트레이트 객체로 부를 수 있는 메서드 | 2.6 |

### 01_generic_vs_dyn — 같은 함수의 두 판

1번 구역은 두 판에 책과 DVD를 넘겨 결과와 "본문이 아는 타입"을 비교합니다. 2번 구역은 함수 아이템의 타입 이름으로 만들어진 함수를, 3번 구역은 넘기는 인자의 크기를 보여 줍니다. 크기는 64비트 기준입니다.

```sh
cargo run -p er-ch12-generics-vs-trait-objects --example 01_generic_vs_dyn
```

```text
1. 결과는 같습니다
    제네릭판: 러스트 입문 → 14일 뒤 반납 (본문이 아는 타입: Book)
    제네릭판: 우주 다큐 → 7일 뒤 반납 (본문이 아는 타입: Dvd)
    dyn판:    러스트 입문 → 14일 뒤 반납 (본문이 아는 타입: dyn Loanable)
    dyn판:    우주 다큐 → 7일 뒤 반납 (본문이 아는 타입: dyn Loanable)
2. 만들어진 함수
    제네릭판: due_note<Book>, due_note<Dvd>
    dyn판:    due_note_dyn
3. 넘기는 인자의 크기 (64비트 기준)
    제네릭판 &Book: 8바이트, dyn판 &dyn Loanable: 16바이트
```

### 02_thin_generic_shell — 얇은 제네릭 껍데기

1번 구역은 서가 라벨 함수를 다섯 타입으로 부르고 두 꼴의 결과가 같은지 확인합니다. 2번 구역은 대출증을 `String`과 글자 수만 세는 `CharCounter`에 써서 두 꼴을 비교합니다(25는 줄바꿈을 포함한 글자 수입니다). 만들어진 함수의 수는 2.3절의 `nm` 명령으로 확인합니다.

```sh
cargo run -p er-ch12-generics-vs-trait-objects --example 02_thin_generic_shell
```

```text
1. 문자열을 여러 모양으로 받는 함수 (부르는 타입 5가지)
    러스트 입문 [러입-06]
    러스트 프로그래밍 공식 가이드 [러프공가-16]
    함께 배우는 러스트 [함배러-10]
    러스트 동시성 [러동-07]
    러스트 실전 [러실-06]
    두 꼴의 결과가 같은가: true
2. 출력 대상을 여러 타입으로 받는 함수 (출력 대상 2가지)
    | 대출증
    |   자료: 우주 다큐
    |   기간: 7일
    두 꼴의 결과가 같은가: true, 센 글자 수: 25 / 25
```

### 03_combined_bounds — 바운드 여러 개

1번 구역은 제네릭판(`Loanable + Debug`, `Loanable + Clone`), 2번 구역은 조합 트레이트 `AuditItem`으로 만든 `dyn`판입니다. 3번 구역은 `&dyn AuditItem`과 `Box<dyn AuditItem>`을 슈퍼트레이트 `Loanable`의 트레이트 객체로 업캐스트합니다.

```sh
cargo run -p er-ch12-generics-vs-trait-objects --example 03_combined_bounds
```

```text
자료: 러스트 입문(서가 3), 우주 다큐(95분)
1. 제네릭판: T: Loanable + Debug
    14일 | Book { title: "러스트 입문", shelf: 3 }
    7일 | Dvd { title: "우주 다큐", minutes: 95 }
    T: Loanable + Clone 으로 남긴 사본: 2개
2. dyn판: 조합 트레이트 AuditItem + 포괄 구현
    14일 | Book { title: "러스트 입문", shelf: 3 }
    7일 | Dvd { title: "우주 다큐", minutes: 95 }
3. 업캐스트: &dyn AuditItem을 슈퍼트레이트 자리에 넘깁니다
    &dyn AuditItem → &dyn Loanable 로 넘겨 더한 대출 기간: 21일
    Box<dyn AuditItem> → Box<dyn Loanable>: 우주 다큐
```

### 04_heterogeneous_collection — 이종 컬렉션의 세 형태

1번 구역은 한 종류만 담는 제네릭 `Shelf<Book>`, 2번 구역은 다른 모듈의 `BoardGame`까지 섞어 담은 `Vec<Box<dyn Loanable>>`, 3번 구역은 열거형 `Media`를 담은 `Shelf<Media>`입니다. 3번 구역은 `match`로 DVD에만 있는 연령 제한을 꺼냅니다. 크기는 64비트 환경에서 잰 값이고, 구조체와 열거형의 크기는 컴파일러가 정하는 메모리 배치에 따라 달라질 수 있습니다.

```sh
cargo run -p er-ch12-generics-vs-trait-objects --example 04_heterogeneous_collection
```

```text
1. 한 종류만 담는다면: 제네릭 Shelf<Book>
    책 2권, 대출 기간 합 28일, 원소 하나의 크기 24바이트
2. 섞어 담고, 종류가 열려 있다면: Vec<Box<dyn Loanable>>
    러스트 입문(14일), 우주 다큐(7일), 보드게임 카탄(3일)
    원소 하나의 크기 16바이트(넓은 포인터), 값은 원소마다 따로 할당한 상자 안에
3. 섞어 담고, 종류가 닫혀 있다면: 열거형 Shelf<Media>
    러스트 입문
    우주 다큐 - 12세 이상
    동물 애니메이션
    대출 기간 합 28일, 원소 하나의 크기 32바이트(가장 큰 배리언트 기준)
```

### 05_where_self_sized — dyn 호환으로 남기기

1번 구역은 구체 타입 `Sms`로 `send_each`와 `with_sender`를 부르고, 2번 구역은 `Vec<Box<dyn Notifier>>`에서 `send`만 부릅니다. 3번 구역은 트레이트 밖의 `?Sized` 함수에 트레이트 객체를 넘기고, 4번 구역은 같은 설계를 쓰는 표준 라이브러리의 `Box<dyn Iterator>`에 `map`과 `sum`을 부릅니다.

```sh
cargo run -p er-ch12-generics-vs-trait-objects --example 05_where_self_sized
```

```text
1. 구체 타입으로는 모든 메서드를 부릅니다
    [문자 중앙도서관 → 010-1234] 예약 도서 도착
    [문자 중앙도서관 → 010-1234] 3일 안에 찾아가세요
    [문자 동네 분관 → 010-1234] 분관에서도 찾을 수 있습니다
2. 트레이트 객체로는 vtable에 있는 핵심 메서드만 부릅니다
    [문자 중앙도서관 → 김러스트] 예약 도서 도착
    [메일 중앙도서관 → 김러스트] 예약 도서 도착
3. 트레이트 밖의 ?Sized 함수는 트레이트 객체도 받습니다
    [메일 중앙도서관 → kim@example.com] 도착 / [메일 중앙도서관 → kim@example.com] 마감 임박
4. 표준 라이브러리의 Iterator도 같은 설계입니다
    연장한 대출 기간 합: 48일
```

## 4. 자주 하는 실수

세 실수 모두 **에러는 트레이트 객체를 쓰는 줄에서 나지만, 결정은 트레이트를 설계할 때** 내립니다. 직접 확인하려면 챕터 디렉터리에서 다음을 실행합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch12-generics-vs-trait-objects
rustc --edition 2024 compile_fail/e0225_dyn_two_traits.rs --out-dir /tmp/er-ch12
rustc --edition 2024 compile_fail/e0038_clone_supertrait.rs --out-dir /tmp/er-ch12
rustc --edition 2024 compile_fail/sized_method_on_dyn.rs --out-dir /tmp/er-ch12
```

### 4.1 트레이트 객체에 트레이트 둘을 `+`로 적음 — E0225

[`compile_fail/e0225_dyn_two_traits.rs`](compile_fail/e0225_dyn_two_traits.rs)

```text
error[E0225]: only auto traits can be used as additional traits in a trait object
  --> compile_fail/e0225_dyn_two_traits.rs:33:42
   |
33 | fn audit_line_dyn(item: &(dyn Loanable + Debug)) -> String {
   |                               --------   ^^^^^ additional non-auto trait
   |                               |
   |                               first non-auto trait
   |
   = help: consider creating a new trait with all of these as supertraits and using that trait here instead: `trait NewTrait: Loanable + Debug {}`
   = note: auto-traits like `Send` and `Sync` are traits that have special properties; for more information on them, visit <https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits>
```

- **읽는 법**: `first non-auto trait`와 `additional non-auto trait`가 문제의 두 트레이트를 가리킵니다. 트레이트 객체는 vtable을 트레이트 하나로만 만들 수 있으므로, 두 번째부터는 메서드가 없는 자동 트레이트만 허락됩니다.
- **해결**: 도움말대로 `trait AuditItem: Loanable + Debug {}`를 만들고 `impl<T: Loanable + Debug> AuditItem for T {}`를 붙입니다(예제 03). 도움말은 포괄 구현까지는 말하지 않는데, 포괄 구현이 없으면 타입마다 `impl AuditItem for Book {}`을 따로 써야 합니다. 트레이트 객체가 꼭 필요하지 않다면 제네릭 `T: Loanable + Debug`로 받는 것이 가장 간단합니다.

### 4.2 조합 트레이트에 `Clone`을 넣음 — E0038

[`compile_fail/e0038_clone_supertrait.rs`](compile_fail/e0038_clone_supertrait.rs)

```text
error[E0038]: the trait `SnapshotItem` is not dyn compatible
  --> compile_fail/e0038_clone_supertrait.rs:34:16
   |
34 | fn keep(item: &dyn SnapshotItem, history: &mut Vec<u32>) {
   |                ^^^^^^^^^^^^^^^^ `SnapshotItem` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> compile_fail/e0038_clone_supertrait.rs:20:32
   |
20 | trait SnapshotItem: Loanable + Clone {}
   |       ------------             ^^^^^ ...because it requires `Self: Sized`
   |       |
   |       this trait is not dyn compatible...
```

같은 에러가 메서드를 부르는 35행과 인자를 넘기는 40행에서도 나서 모두 세 개입니다(생략).

- **읽는 법**: `...because` 줄이 가리키는 것은 메서드가 아니라 슈퍼트레이트 `Clone`입니다. `Clone`은 `trait Clone: Sized`로 정의되어 있어서, 그것을 슈퍼트레이트로 둔 `SnapshotItem`도 `Self: Sized`를 요구하게 됩니다. 크기를 모르는 `dyn SnapshotItem`은 이 조건을 만족할 수 없습니다. ch2와 Part 0 ch17의 E0038은 메서드의 시그니처가 원인이었지만, 여기서는 트레이트 선언의 슈퍼트레이트 목록이 원인입니다.
- **해결**: 사본이 필요한 곳은 제네릭 `T: Loanable + Clone`으로 받습니다(예제 03의 `snapshot`). 트레이트 객체로 사본을 만들어야 한다면 `Self` 대신 `Box<dyn Loanable>`을 돌려주는 메서드 `fn boxed_copy(&self) -> Box<dyn Loanable>`을 트레이트에 두고 타입마다 구현합니다. 이 메서드는 `Self`를 쓰지 않으므로 dyn 호환을 깨지 않습니다.

### 4.3 `where Self: Sized` 메서드를 트레이트 객체로 부름 — 에러 코드 없음

[`compile_fail/sized_method_on_dyn.rs`](compile_fail/sized_method_on_dyn.rs)

```text
error: the `send_each` method cannot be invoked on a trait object
  --> compile_fail/sized_method_on_dyn.rs:40:24
   |
21 |         Self: Sized,
   |               ----- this has a `Sized` requirement
...
40 |     let sent = channel.send_each("010-1234", ["도착", "마감 임박"]); // 에러
   |                        ^^^^^^^^^
```

- **읽는 법**: 에러 코드가 없으므로 `rustc --explain`으로 찾을 수 없습니다. ``this has a `Sized` requirement``가 가리키는 `Self: Sized` 줄이 원인입니다. 트레이트 설계자가 일부러 이 메서드를 트레이트 객체에서 뺐다는 뜻입니다. Part 0 ch17에서는 `Self`를 돌려주는 메서드로 같은 에러를 봤고, 여기서는 제네릭 메서드입니다.
- **해결**: 구체 타입의 값에서 부르거나, 트레이트 밖의 `fn send_each_any<N: Notifier + ?Sized, I>(..)`를 부릅니다(예제 05의 3번 구역). 트레이트를 고칠 수 있다면 연습 4처럼 `Box<N>`에 트레이트를 다시 구현하는 방법도 있습니다.

### 4.4 그 밖의 에러

| 상황 | 에러 | 해결 |
| --- | --- | --- |
| `fn f(x: &impl Loanable)`를 `f::<Book>(..)`로 부름 | E0107 (``function takes 0 generic arguments but 1 generic argument was supplied``) | 타입을 적어야 하는 함수는 `fn f<T: Loanable>(x: &T)`로 선언합니다(2.2절). |
| 제네릭 메서드나 `Self`를 돌려주는 메서드가 있는 트레이트로 `dyn` | E0038 (`...because method .. has generic type parameters` 등) | 해당 메서드에 `where Self: Sized`를 붙입니다(2.6절). |
| 값으로 받는 `self` 메서드를 `Box<dyn Trait>`에서 부름 | E0161 (``cannot move a value of type `dyn Notifier` ``) | `self: Box<Self>` 수신자로 바꿉니다(2.6절). |
| 제네릭 `Vec<T>`에 다른 구체 타입을 섞음 | E0308 (Part 0 ch17 4.5절) | 종류가 열려 있으면 `Box<dyn Trait>`, 닫혀 있으면 열거형(2.5절). |

## 5. 연습 문제

### 연습 1. 무엇으로 받을까

다음 상황마다 제네릭, 트레이트 객체, 열거형 중 무엇을 쓸지 고르고 이유를 적어 보세요.

1. 목록을 정렬하는 라이브러리 함수가 비교 기준을 받습니다. 호출 한 번에 기준은 하나입니다.
2. 사진 편집기가 실행 중에 설정 폴더의 공유 라이브러리에서 필터를 불러와 차례로 적용합니다.
3. 결제 화면에서 카드, 계좌 이체, 포인트 세 가지 결제 수단을 한 목록에 보여 줍니다. 결제 수단은 앱이 정합니다.
4. 큰 프로그램의 여러 곳에서 보고서를 `String`, 파일, 표준 출력에 씁니다. 보고서를 쓰는 함수는 300줄이고, 빌드 시간이 길다는 불만이 있습니다.

<details>
<summary>힌트</summary>

1번은 호출 한 번에 한 타입이므로 제네릭(`F: FnMut(&T, &T) -> Ordering`)입니다. 표준 라이브러리의 `sort_by`도 그렇습니다. 2번은 컴파일할 때 소스가 없는 타입이라 트레이트 객체(`Vec<Box<dyn Filter>>`)만 가능합니다. 3번은 종류가 닫혀 있고 앱 안에 모두 있으므로 열거형이 어울립니다. 4번은 2.3절의 상황입니다. 바깥은 `W: io::Write` 제네릭으로 두어 부르는 쪽 편의를 지키고, 300줄 본문은 `&mut dyn io::Write`를 받는 함수 하나로 옮깁니다. 정말 빌드 시간이 줄었는지는 바꾸기 전과 후에 재 봅니다.

</details>

### 연습 2. 복사되는 함수 세기

예제 02의 1번 구역에 여섯 번째 타입 `Arc<str>`(`std::sync::Arc`)로 부르는 줄을 두 꼴 모두에 더하세요. 다시 빌드하기 전에 `shelf_label_fat`, `shelf_label::`, `shelf_label_str`의 줄 수를 예측하고, 2.3절의 `nm` 명령으로 확인해 보세요.

<details>
<summary>힌트</summary>

비권장 꼴은 타입 하나마다 14개가 늘어 84, 얇은 껍데기는 하나 늘어 6, 안쪽 함수는 그대로 14입니다(이 환경에서 확인한 값). 안쪽 함수의 클로저는 제네릭 함수 안에 있지 않으므로 타입이 늘어도 다시 만들어지지 않습니다. 확인한 뒤에는 추가한 줄을 지워 예제를 원래대로 돌려 둡니다.

</details>

### 연습 3. 잡지를 추가하면

예제 04에 대출 기간이 3일인 `Magazine`을 추가해, 2번 구역의 카트와 3번 구역의 `Shelf<Media>`에 하나씩 넣어 보세요. 두 곳에서 고친 곳의 수를 비교해 보세요.

<details>
<summary>힌트</summary>

트레이트 객체 쪽은 `struct Magazine`과 `impl Loanable for Magazine`을 쓰고 카트에 `Box::new(..)`를 하나 넣으면 끝입니다. `describe_cart`는 고치지 않습니다. 열거형 쪽은 `Media::Magazine(Magazine)` 배리언트를 더한 뒤 `age_limit`, `title`, `loan_days`의 `match`에 갈래를 하나씩 더해야 합니다. 빠뜨리면 완전성 검사가 E0004로 알려 주므로 놓칠 걱정은 없지만, 고칠 곳은 `match`의 수만큼 늘어납니다. 이 차이가 2.5절의 "열린 집합과 닫힌 집합"입니다.

</details>

### 연습 4. `Box<dyn Notifier>`에서도 `send_each` 부르기

예제 05에 `impl<N: Notifier + ?Sized> Notifier for Box<N>`을 더해 `channels[0].send_each(..)`가 컴파일되게 해 보세요. `send`는 `(**self).send(to, text)`로 위임합니다.

<details>
<summary>힌트</summary>

그대로 더하면 E0046(``not all trait items implemented, missing: `with_sender` ``)입니다. `Box<N>`은 크기를 아는 타입이라 `where Self: Sized` 메서드도 구현해야 하는데, `N`이 `dyn Notifier`일 때 `Box<N>` 사본을 만들 방법이 없습니다. E0038의 도움말(``consider moving `with_sender` to another trait``)처럼 `with_sender`를 별도 트레이트(`trait Rename { fn with_sender(&self, sender: &str) -> Self; }`)로 옮기고 `Sms`, `Email`에 구현하면 됩니다. 그러면 `Box<dyn Notifier>`가 `Notifier`를 구현하고 `Sized`이므로 기본 구현의 `send_each`를 부를 수 있습니다. `Iterator`가 `Box<I>`에 대해 하는 일이 이것입니다.

</details>

## 6. 참고 자료

- [Effective Rust, Item 12: Understand the trade-offs between generics and trait objects](https://effective-rust.com/generics.html): 이 챕터가 다루는 지침의 출처입니다. 제네릭과 트레이트 객체를 소개하고, 코드 크기·속도·컴파일 시간, 바운드 조합, 슈퍼트레이트, dyn 호환(책의 object safety)을 차례로 비교합니다.
- [The Rust Reference: Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility): 트레이트 객체로 쓸 수 있는 트레이트의 전체 규칙입니다. 디스패치할 수 있는 함수와 `where Self: Sized`로 명시적으로 뺀 함수, 허용되는 수신자 타입이 정리되어 있습니다.
- [Announcing Rust 1.86.0](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/): 트레이트 객체 업캐스트가 안정화된 릴리스의 발표입니다.
- [rust-lang/rust#47649](https://github.com/rust-lang/rust/issues/47649): 책의 각주가 가리키는, `Box<Self>`를 돌려주는 메서드의 제약에 관한 이슈입니다.
- 본편: [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md)(단형성화, 트레이트 바운드, 포괄 구현), [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)(트레이트 객체, 동적 디스패치, dyn 호환, 열거형과의 비교), [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)(슈퍼트레이트, `Sized`), [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md)(넓은 포인터)
