# ch14. 라이프타임 이해하기

[Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md)에서 라이프타임(lifetime) 표기 문법과 생략 규칙(elision rules) 세 가지를 **규칙**으로 배웠습니다. 규칙만으로도 에러를 고칠 수는 있지만, 왜 그 규칙인지 모르면 컴파일러가 제안하는 `'a`나 `'static`을 붙였다 뗐다 하며 맞추게 됩니다. 이 챕터는 그 규칙을 하나의 **모형**으로 다시 세웁니다. 참조가 위험해지는 것은 가리키던 값의 스택 프레임이나 소유자가 사라질 때이고, 라이프타임은 "그 일이 일어나기 전에만 참조를 쓴다"는 것을 컴파일러가 검사할 수 있게 붙인 이름입니다. 라이프타임 표기(lifetime annotation)는 값이 사는 기간을 바꾸지 않고 **관계**만 적습니다. 이 모형 위에서 세 가지 판단을 정리합니다. 라이프타임 매개변수(lifetime parameter)를 하나로 묶을지 나눌지, `'static`을 요구할지, 구조체에 참조를 담을지 소유할지입니다.

- 원제: [Item 14: Understand lifetimes](https://effective-rust.com/lifetimes.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch14-lifetimes`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 본편 챕터의 내용을 안다고 보고 진행합니다. 문법과 규칙의 나열은 그쪽으로 넘깁니다.

- [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 2.8~2.11절: 참조와 대여, 마지막 사용까지만 유효한 참조, 댕글링 참조(dangling reference)
- Part 0 ch10 2.12~2.17절: 라이프타임 표기, `longest`, 구조체의 라이프타임, 생략 규칙, 메서드의 라이프타임, `'static`
- [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 2.4절, [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 2.2절: `thread::spawn`의 `'static` 바운드, 스코프 스레드
- [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.3·2.4·2.6절: RPIT 캡처, 임시값 범위
- [ch11](../ch11-drop-raii/README.md) 2.3·2.5절: `Drop`이 있는 값이 붙잡는 대여, drop 순서

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 스택 프레임과 소유 사슬로 "어떤 참조가 함수 밖으로 나갈 수 있는가"를 설명하고, 값·참조·임시값의 라이프타임이 각각 어디서 끝나는지 짚을 수 있다.
2. 라이프타임 표기가 "같은 구간 안에서 유효하다"는 관계만 적는다는 것을 설명하고, 결과가 실제로 어느 입력에서 오는지 보고 라이프타임 매개변수를 묶을지 나눌지 고를 수 있다.
3. 생략 규칙이 채워 넣은 관계를 읽어 의도와 다를 때 직접 적고, 경로에 숨은 라이프타임을 `'_`로 드러낼 수 있다.
4. `&'static T`와 `T: 'static`을 구별하고, `'static` 참조를 얻는 방법과 대가를 말하고, `'static`을 요구하거나 `Box::leak`으로 에러를 "해결"하는 것이 왜 비권장인지 설명할 수 있다.
5. 구조체에 참조를 담을지 소유할지 판단 기준으로 고르고, 참조를 담은 타입의 라이프타임이 그것을 담는 타입으로 번지는 모습을 설명할 수 있다.

## 2. 핵심 개념

이해가 목적인 2.2·2.3·2.6절은 "무엇을 헷갈리기 쉬운가 → 잘못된 짐작으로 쓴 코드와 컴파일러의 답 → 맞는 모형 → 이 모형이 설명하지 못하는 곳" 순서로, 판단이 걸린 2.4·2.5·2.7·2.8절은 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 씁니다. 전체 코드는 `examples/`에 있고, 비권장 형태와 권장 형태를 같은 파일에서 함께 실행합니다.

### 2.1 지도: 짐작과 모형

| 헷갈리기 쉬운 짐작 | 맞는 모형 | 절 | 예제 |
| --- | --- | --- | --- |
| 힙에 올리면 라이프타임과 상관없다 | 힙의 값도 소유 사슬의 끝(지역 변수나 `static`)만큼 산다 | 2.2 | 01 |
| 변수가 스코프에 있는 동안 대여가 이어진다 | 참조는 마지막 사용에서, 값은 drop이나 이동에서 끝난다 | 2.3 | 02 |
| `'a`를 붙이면 값이 더 오래 산다 | 표기는 관계만 적는다. 이름을 묶으면 짧은 쪽에 맞춰진다 | 2.4 | 03 |
| 생략하면 라이프타임이 없다 | 규칙이 채워 넣은 관계가 시그니처에 그대로 있다 | 2.5 | 04 |
| `'static`은 영원히 사는 값이다 | `&'static T`는 영원히 유효한 참조, `T: 'static`은 짧은 대여를 품지 않은 타입 | 2.6·2.7 | 05·06 |
| 참조를 담은 구조체가 언제나 더 효율적이다 | 복사가 없는 대신 라이프타임이 번지고 원본이 따로 살아 있어야 한다 | 2.8 | 07 |

### 2.2 스택 프레임과 소유 사슬: 참조가 나갈 수 있는 곳

**무엇을 헷갈리기 쉬운가.** "함수 안에서 만든 값의 참조는 돌려줄 수 없다"를 Rust만의 깐깐한 규칙으로 여기기 쉽습니다. C나 C++에서는 지역 변수의 주소를 돌려주는 코드가 컴파일되고, 그 주소는 다음 함수 호출이 같은 스택 공간을 다시 쓰면서 엉뚱한 값을 가리키게 됩니다. Rust의 규칙은 이 사고를 컴파일할 때 막는 장치입니다.

**잘못된 짐작으로 쓴 코드.** "참조를 돌려주면 복사가 없으니 빠르다"는 생각으로, 함수 안에서 만든 문자열을 빌려 돌려줍니다.

```rust
fn shopping_line(recipe: &Recipe) -> &str {
    let line = format!("{}: {}", recipe.name, recipe.ingredients.join(", "));
    &line // error[E0515]: cannot return reference to local variable `line`
}
```

이 모양의 E0515는 Part 0 ch10 4.5절에서 봤습니다.

**맞는 모형.** 함수가 불리면 매개변수, 지역 변수, 임시값을 담는 **스택 프레임(stack frame)** 이 생깁니다. 함수가 돌아가면 프레임의 값이 drop되고, 그 공간은 다음에 불리는 함수가 다시 씁니다. 그러니 함수가 돌려주는 참조는 그 함수의 프레임 **밖**에 있는 값만 가리킬 수 있습니다. [`01_stack_frames.rs`](examples/01_stack_frames.rs)가 경우마다 보여 줍니다.

| 돌려주는 것 | 가리키는 곳 | 결과 | 예제 01 |
| --- | --- | --- | --- |
| 지역 값의 참조 | 받는 쪽의 프레임 | E0515 | 1번 구역(지역 값이 함수 끝에서 drop됨) |
| 값 자체(`Tracked`, `String`) | 없음. 값이 부르는 쪽으로 이동 | 됨 | 2·3번 구역 |
| 매개변수에서 얻은 참조 | 부르는 쪽이 가진 값 | 됨 | 3번 구역 |
| `'static` 참조 | 실행 파일의 정적 데이터 | 됨 | 4번 구역 |

힙의 값도 이 모형 안에 있습니다. `Box`, `Vec`, `String`이 가리키는 힙의 값에는 언제나 소유자가 있고, 소유자를 거슬러 올라간 **소유 사슬(chain of ownership)** 은 어느 프레임의 지역 변수나 `static` 항목에서 끝납니다. 5번 구역에서 레시피는 힙에 있지만, 소유 사슬은 안쪽 블록의 변수 `in_box`에서 시작했다가 `shelf.push(in_box)`로 `main`의 변수 `shelf`로 옮겨 갑니다. 힙의 레시피를 가리키는 참조는 사슬의 끝인 `shelf`가 살아 있는 동안 유효하고, `shelf`가 drop되면 사슬을 따라 모두 해제됩니다. 사슬이 `static`에서 끝나면 라이프타임은 `'static`입니다(2.6절).

**이 모형이 설명하지 못하는 곳.** 스택 프레임은 단순화한 그림입니다. 최적화된 코드에서는 값이 레지스터에만 있거나 프레임이 생기지 않기도 합니다. 대여 검사기(borrow checker)는 실제 메모리 배치를 보지 않고 이 모형에서 나오는 규칙만 따집니다. 그래서 다음 절의 이동처럼, 메모리로 보면 멀쩡한데도 거부되는 코드가 있습니다.

### 2.3 라이프타임의 범위: 값, 참조, 임시값

**무엇을 헷갈리기 쉬운가.** 라이프타임을 변수의 스코프(중괄호)로 생각하기 쉽습니다. 실제로는 세 구간이 따로 끝나고, 대여 검사기는 "참조의 구간이 가리키는 값의 구간 안에 들어가는가" 하나만 확인합니다.

| 구간 | 시작 | 끝 | 예제 02 |
| --- | --- | --- | --- |
| 값 | 만들어질 때 | drop되거나 **이동될 때** | 2번 구역 |
| 참조 | `&`로 빌릴 때 | 그 참조의 **마지막 사용** | 1번 구역 |
| 임시값 | 표현식을 평가할 때 | 대개 그 **문장의 끝** | 3·4번 구역 |

**잘못된 짐작으로 쓴 코드.** "`String`을 다른 곳으로 옮겨도 힙의 글자는 그대로이니, 옮기기 전에 빌린 `&str`은 계속 써도 된다."

```rust
let title = first_line(&draft);
archive.push(draft); // error[E0505]: cannot move out of `draft` because it is borrowed
println!("{title}");
```

짐작은 메모리로 보면 맞습니다. `String`을 옮기면 핸들(포인터·길이·용량) 세 칸만 복사되고 힙의 글자는 그 자리에 있습니다. 그래도 거부되는 것은 대여 검사기가 **메모리가 아니라 소유자**를 추적하기 때문입니다. 새 소유자 `archive`가 언제 그 글자를 지우거나 drop할지는 알 수 없으므로, 소유자가 바뀌는 순간 값의 구간이 끝난 것으로 봅니다(4.1절).

**맞는 모형.** [`02_lifetime_spans.rs`](examples/02_lifetime_spans.rs)로 세 구간의 끝을 확인합니다.

- **참조는 마지막 사용에서 끝납니다.** 1번 구역에서 `title`이라는 이름은 `main` 끝까지 남아 있지만 대여는 마지막 `println!`에서 끝나므로, 그 뒤에 `draft`를 고칠 수 있습니다. 2번 구역처럼 옮길 수도 있습니다. 중괄호가 아니라 마지막 사용을 기준으로 하는 이 방식을 **마지막 사용에서 끝나는 대여(NLL, non-lexical lifetimes)** 라고 부릅니다(Part 0 ch4 2.10절).
- **임시값은 문장이 끝날 때 drop됩니다.** 표현식을 계산하다 생긴 이름 없는 값을 임시값(temporary)이라고 합니다. 3번 구역에서 `Tracked("임시 쪽지").label()`이 돌려준 `&str`은 같은 `println!` 안에서만 쓰이므로 문제가 없고, `[drop] 임시 쪽지`는 그 줄을 출력한 직후에 나옵니다. 임시값을 빌린 결과를 다음 문장까지 들고 가면 E0716 `temporary value dropped while borrowed`입니다([Part 1 ch3](../../part1-standard-lib/ch3-strings-fmt/README.md) 4.2절).
- **`let`이 임시값의 참조를 바로 묶으면 임시값이 변수만큼 삽니다.** 이것을 **임시값 수명 연장(temporary lifetime extension)** 이라고 합니다. 4번 구역의 `[drop] 묶어 둔 쪽지`는 블록이 끝날 때 나옵니다. 함수의 인자로 넘긴 임시값은 연장되지 않습니다.

**이 모형이 설명하지 못하는 곳.** "참조는 마지막 사용에서 끝난다"에는 예외가 있습니다. 빌린 것을 품은 값이 `Drop`을 구현하면, 스코프 끝에서 실행되는 `drop`이 그 참조를 다시 쓸 수 있으므로 대여가 그 값의 스코프 끝까지 이어집니다([ch11](../ch11-drop-raii/README.md) 2.3절). 지역 변수·필드·임시값이 버려지는 순서는 ch11 2.5절에 정리돼 있습니다. "임시값은 문장 끝에서 drop된다"도 근사입니다. `match`의 조건식에서 만든 임시값은 `match` 전체가 끝날 때까지 살고, 2024 에디션에서는 블록의 마지막 표현식과 `if let` 조건식의 임시값이 문장 끝보다 **일찍** drop됩니다(Part 2 ch4 2.4·2.6절). 연장 규칙도 넓어져 왔습니다. 이 툴체인의 릴리스 노트에 따르면 1.79.0부터 `if`/`match` 갈래 안으로, 1.89.0부터 `Some(&임시값)` 같은 튜플 구조체·배리언트 생성자 안으로 연장이 이어집니다(두 모양 모두 직접 컴파일해 확인). 정확한 규칙은 Part 9 `ch5-statements-expressions`에서 다룹니다.

### 2.4 지침 1: 라이프타임 매개변수는 결과의 출처대로 묶거나 나눈다

라이프타임 이름이 시그니처에서 말할 수 있는 것은 "이 참조들은 같은 구간 `'a` 동안 유효하다"는 관계뿐입니다. `'static` 말고는 구체적인 기간을 적을 방법이 없고, 표기를 바꾼다고 값이 더 오래 살지도 않습니다(Part 0 ch10 2.13절). 부르는 쪽에서 `'a`는 같은 이름이 붙은 입력이 **모두** 유효한 구간, 즉 그중 가장 짧게 사는 쪽에 맞춰집니다. 그래서 이름을 어떻게 나눠 주느냐가 곧 "부르는 쪽에 무엇을 요구하느냐"입니다. 함수 하나에서 이것을 본 예는 Part 0 ch10 2.13절의 `strip_tag`와 [Part 0 ch12](../../part0-getting-started/ch12-io-project-minigrep/README.md) 2.8절의 `search`입니다. 여기서는 결과를 구조체에 담는 경우로 넓힙니다.

**문제 상황.** Rust 용어집에서 검색어의 뜻을 찾습니다. 용어집은 프로그램이 시작할 때 읽어 `main` 끝까지 살고, 검색어는 사용자가 입력할 때마다 `trim().to_lowercase()`로 정규화한 새 `String`이라 반복 한 번이 끝나면 drop됩니다. 결과에는 검색어와 뜻을 함께 담고, 찾은 뜻은 반복 밖의 목록에 모읍니다.

**비권장 코드.** 결과 구조체와 함수에 라이프타임 하나를 씁니다.

```rust
struct BoundHit<'a> {
    query: &'a str,
    meaning: &'a str,
}

fn look_up_bound<'a>(glossary: &'a str, query: &'a str) -> Option<BoundHit<'a>>
```

`meaning`은 언제나 용어집에서 오지만 타입은 `&'a str`이고, `'a`는 짧게 사는 검색어에 맞춰집니다. 찾은 뜻을 반복 밖의 `Vec<&str>`에 넣으면 E0597입니다(4.2절). [`03_split_lifetimes.rs`](examples/03_split_lifetimes.rs)의 1번 구역은 이 시그니처를 그대로 두고 부르는 쪽이 할 수 있는 우회, 곧 뜻을 `to_string()`으로 복사해 모으는 모습을 보여 줍니다. 찾은 뜻이 셋이면 복사한 `String`도 셋입니다.

이 시그니처는 쉽게 생깁니다. 표기 없이 쓰면 나는 E0106의 도움말이 결과가 어느 입력에서 오든 모든 입력을 `'a` 하나로 묶자고 제안하기 때문입니다.

```text
help: consider introducing a named lifetime parameter
  |
1 | fn meaning<'a>(glossary: &'a str, query: &'a str) -> &'a str {
  |           ++++            ++              ++          ++
```

컴파일러는 본문을 보지 않고 시그니처만으로 제안합니다. 이 제안은 "컴파일되는 시그니처"이지 "맞는 시그니처"가 아닙니다.

**권장 코드.** 출처마다 이름을 따로 둡니다.

```rust
struct Hit<'g, 'q> {
    query: &'q str,   // 검색어에서
    meaning: &'g str, // 용어집에서
}

fn look_up<'g, 'q>(glossary: &'g str, query: &'q str) -> Option<Hit<'g, 'q>>
```

본문은 한 글자도 바뀌지 않았습니다. 2번 구역에서 `hit.meaning`은 `&'g str`이므로 검색어가 drop된 뒤에도 `found`에 남고, 복사한 `String`은 0개입니다. 받는 쪽도 이 약속으로 검사받습니다. `fn meaning_of<'g, 'q>(glossary: &'g str, query: &'q str) -> &'g str`의 본문이 `query`를 돌려주려 하면, 에러 코드 없이 함수 정의에서 거부됩니다.

```text
error: lifetime may not live long enough
  |     ^^^^^ function was supposed to return data with lifetime `'g` but it is returning data with lifetime `'q`
  = help: consider adding the following bound: `'q: 'g`
```

`'q: 'g`는 "`'q`가 적어도 `'g`만큼 유효하다"는 바운드입니다. 이름으로 표현할 수 있는 관계는 이렇게 "같다"와 "안에 들어간다"뿐입니다. 도움말을 따르면 검색어가 결과만큼 살아야 하므로, 묶은 판과 같은 제약으로 돌아갑니다.

**판단 기준.**

| 결과(또는 필드)가 오는 곳 | 라이프타임 | 예 |
| --- | --- | --- |
| 한 입력에서만 | 그 입력에만 이름을 주고 나머지는 생략 | Part 0 ch12의 `search<'a>(query: &str, contents: &'a str)` |
| 조건에 따라 여러 입력 중 하나 | 같은 이름으로 **묶어야** 함 | 예제 03의 `meaning_or_query`, Part 0 ch10의 `longest` |
| 서로 다른 데이터를 가리키는 필드들 | 출처마다 하나씩 | `Hit<'g, 'q>` |
| 같은 데이터의 여러 부분을 가리키는 필드들 | 하나 | 예제 07의 `TrackView<'a>` |

3번 구역의 `meaning_or_query`는 뜻을 찾지 못하면 검색어를 그대로 돌려줍니다. 결과가 어느 쪽에서든 올 수 있으므로 `'a` 하나로 묶는 것이 사실 그대로이고, 그 대가로 결과는 그 반복 안에서만 씁니다.

**지침을 따르지 않아도 되는 경우.** 이름을 나누면 타입이 길어지고(`Hit<'g, 'q>`), 그 타입을 담는 타입에도 매개변수가 둘씩 번집니다(2.8절). 모든 부르는 쪽이 두 입력을 같은 스코프에서 만들고 결과를 그 안에서만 쓴다면, 하나로 묶어도 거부되는 코드가 없습니다. 불변 참조는 더 짧은 구간으로 줄여서 맞출 수 있기 때문입니다(이 성질을 변성이라고 하며 Part 10 `ch4-variance-dropck`에서 다룹니다). 반대로 부르는 쪽을 모르는 공개 API라면 출처대로 나눠 두어야 부르는 쪽의 코드를 덜 막습니다.

### 2.5 지침 2: 생략된 라이프타임을 읽고, 의도와 다르면 직접 적고, 숨기지 않는다

**문제 상황.** 블로그 글을 복사 없이 들여다보는 뷰 타입 `PostView<'a>`에 제목을 꺼내는 메서드를 둡니다. 여러 글의 제목을 모으거나, 뷰를 함수 안에서 잠깐 만들고 제목만 돌려주는 도우미도 필요합니다.

**비권장 코드.** 표기 없이 씁니다.

```rust
impl<'a> PostView<'a> {
    fn title_elided(&self) -> &str {
        self.text.lines().next().unwrap_or("")
    }
}
```

생략 규칙 3(Part 0 ch10 2.15절)은 `&self`의 라이프타임을 반환값에 붙이므로, 이 시그니처는 `fn title_elided<'s>(&'s self) -> &'s str`, 즉 "제목은 **뷰를 빌린 동안** 유효하다"는 뜻입니다. 실제로 돌려주는 것은 뷰가 가리키는 글의 조각인데도 그렇습니다. 뷰가 살아 있는 동안에는 차이가 드러나지 않지만([`04_elided_lifetimes.rs`](examples/04_elided_lifetimes.rs)의 1번 구역), 뷰를 지역 변수로 만들고 제목을 돌려주는 도우미는 E0515이고(4.3절), 글마다 임시 뷰를 만들어 제목을 모으는 `map(|p| PostView::new(p).title_elided())`도 클로저가 임시값을 빌린 값을 돌려준다는 E0515입니다.

**권장 코드.** 반환값의 출처인 `'a`를 적습니다. 본문은 같습니다.

```rust
fn title(&self) -> &'a str {
    self.text.lines().next().unwrap_or("")
}
```

이제 제목은 글이 살아 있는 동안 유효하고, 뷰는 쓰고 바로 버려도 됩니다(2번 구역). 생략 규칙은 흔한 모양을 정한 약속일 뿐 본문을 보고 정하지 않습니다. 뷰나 파서처럼 **참조를 담은 타입의 메서드**가 담고 있는 데이터의 조각을 돌려준다면, `&self`가 아니라 타입의 라이프타임을 적어야 하는지 확인합니다. 반환 타입이 `impl Iterator<Item = &'a str>`이면 한 가지를 더 봅니다. 2024 에디션의 반환 위치 `impl Trait`은 스코프의 라이프타임을 모두 캡처하므로 `&self`의 구간에도 묶입니다. 뷰와 상관없게 하려면 `+ use<'a>`로 캡처를 좁힙니다(연습 2, Part 2 ch4 2.3절).

**숨은 라이프타임은 `'_`로 드러냅니다.** 라이프타임 매개변수가 있는 타입을 `<>` 없이 쓰면 생략된 라이프타임이 경로 안에 숨습니다.

```rust
fn open_hidden(post: &str) -> PostView { .. }  // 비권장
fn open(post: &str) -> PostView<'_> { .. }     // 권장: 뜻은 같음

impl fmt::Display for PostView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { .. }
}
```

**익명 라이프타임(anonymous lifetime)** `'_`는 새 라이프타임을 만들지 않고 "여기 생략된 라이프타임이 있다"고만 표시합니다. 그래서 두 함수의 뜻과 결과는 같습니다(3번 구역). 차이는 읽는 사람에게 있습니다. `open_hidden`의 시그니처만 보면 결과가 `post`를 빌린다는 것을 알 수 없어서, 결과를 `post`보다 오래 쓰다가 에러를 만나고서야 알게 됩니다. `impl` 블록의 타입이나 `Formatter`처럼 이름을 붙여도 다시 쓸 곳이 없는 자리에도 `'_`를 씁니다.

반환 타입에 숨긴 경우는 기본으로 켜진 rustc 린트가 잡습니다. 예제는 `#[expect(mismatched_lifetime_syntaxes, reason = "..")]`로 막아 두었고, 속성을 지우면 다음 경고가 납니다(핵심 줄만 옮겼습니다).

```text
warning: hiding a lifetime that's elided elsewhere is confusing
   |
   | fn open_hidden(post: &str) -> PostView {
   |                      ^^^^     ^^^^^^^^ the same lifetime is hidden here
   |                      |
   |                      the lifetime is elided here
   |
   = note: `#[warn(mismatched_lifetime_syntaxes)]` on by default
help: use `'_` for type paths
```

`fn show(view: PostView)`처럼 입력에만 숨은 경우와 `Formatter`의 `<'_>`를 빠뜨린 경우는 이 린트가 잡지 않습니다. 그런 곳까지 찾으려면 기본으로 꺼진 `elided_lifetimes_in_paths`(`rust_2018_idioms` 그룹)를 켭니다. clippy는 거꾸로 **필요 없는 이름**을 잡습니다. `fn pick<'a>(s: &'a str) -> &'a str`처럼 생략할 수 있는 이름은 기본으로 켜진 `needless_lifetimes`가, `impl<'a> Display for PostView<'a>`처럼 `'_`로 바꿀 수 있는 이름은 기본으로 꺼진 `elidable_lifetime_names`(pedantic)가 알려 줍니다(`rustc -W help`, `clippy-driver -W help`와 직접 컴파일로 확인).

**지침을 따르지 않아도 되는 경우.** 생략 규칙이 고른 관계가 의도와 같다면 적지 않습니다. 메서드가 정말로 타입 자신이 소유한 데이터(예: 구조체 안의 `String` 필드)를 빌려준다면 생략된 `&self` 쪽이 맞습니다.

### 2.6 `'static` 참조는 어디서 오는가

**무엇을 헷갈리기 쉬운가.** `'static`을 "특별히 오래 사는 라이프타임" 정도로 알고, 에러 메시지가 제안하면 붙이는 이름으로 쓰기 쉽습니다. `&'static T`는 "프로그램이 끝날 때까지 유효한 참조"이고, 2.2절의 말로 하면 **어느 프레임의 소유 사슬에도 속하지 않아 drop되지 않는 값**을 가리키는 참조입니다. 입력 참조가 없는 함수가 참조를 돌려준다면 돌려줄 수 있는 것은 이런 참조뿐입니다.

**잘못된 짐작으로 쓴 코드.** "`Box`로 힙에 올렸으니 스택 프레임과 상관없이 `'static`이다."

```rust
fn probe_name(no: u32) -> &'static str {
    let name: Box<str> = format!("탐사선-{no}호").into_boxed_str();
    &name // error[E0515]: cannot return reference to local variable `name`
}
```

글자는 힙에 있지만, 그 힙 값의 소유자는 이 함수의 지역 변수 `name`입니다. 함수가 끝나면 `name`이 drop되고 소유 사슬을 따라 힙의 글자도 해제되므로, 2.2절의 지역 값과 똑같이 거부됩니다. `main` 안에서 `let r: &'static str = &name;`처럼 빌려도 같은 이유로 E0597(``type annotation requires that `name` is borrowed for `'static` ``)입니다.

**맞는 모형.** `'static` 참조를 얻는 길은 넷이고, 대가가 저마다 다릅니다([`05_static_sources.rs`](examples/05_static_sources.rs)).

| 출처 | 예 | 대가와 한계 | 예제 05 |
| --- | --- | --- | --- |
| 문자열 리터럴 | `"일"` | 컴파일할 때 정해진 글자만 | 1번 구역 |
| `static` 항목 | `static PLANETS: [&str; 4]` | 프로그램 전체에 하나. 고치려면 `Mutex`나 원자 타입이 필요 | 2번 구역 |
| 상수 승격 | `&EARTH_DAYS`, `&[88, 225, EARTH_DAYS, 687]` | 컴파일할 때 계산되고 `Drop`도 내부 가변성도 없는 값만 | 3번 구역 |
| `Box::leak` | `Box::leak(label.into_boxed_str())` | 소유자를 없애 영영 해제하지 않음(메모리 누수) | 4번 구역 |

**상수 승격(constant promotion)** 은 `&` 뒤의 표현식을 컴파일할 때 계산할 수 있으면 컴파일러가 그 값을 정적 데이터로 올려 `'static` 참조를 만들어 주는 것입니다. 3번 구역의 `let doubled: &'static u32 = &(EARTH_DAYS * 2);`도 승격 덕분에 컴파일됩니다. 임시값 수명 연장만으로는 `'static`이 될 수 없습니다. `-> &'static T`를 돌려주는 함수의 본문으로 경계를 시험한 결과입니다.

| `&` 뒤의 표현식 | 결과 |
| --- | --- |
| 필드만 있는 구조체의 `const`, 리터럴 배열, `const`를 쓴 산술 | 승격됨 |
| `Drop`을 구현한 타입의 `const`나 값 | E0515 `cannot return reference to temporary value` |
| `Cell`처럼 내부 가변성이 있는 타입의 `const` | E0515 |
| 함수 호출(`String::new()`, `const fn`인 `u32::max(1, 2)`도) | E0515 |

`const`는 이름 붙은 값이라 쓰는 곳마다 따로 복사될 수 있으므로, 주소가 하나여야 하는 데이터는 `static`에 둡니다.

`Box::leak`은 안전한 함수입니다. 아무도 해제하지 않으니 해제된 메모리를 읽을 일도 없습니다. 대신 4번 구역처럼 부를 때마다 되찾을 수 없는 메모리가 늘어납니다. 프로그램이 시작할 때 한 번 만들어 끝까지 쓰는 값이라면 이 대가가 문제되지 않지만, 요청마다 부르는 코드라면 누수가 쌓입니다. `String::leak`(1.72.0)과 `Vec::leak`도 같은 일을 합니다. 누수와 `mem::forget`은 Part 10 `ch7-obrm`에서 다룹니다.

**이 모형이 설명하지 못하는 곳.** `'static`은 "언제까지 유효한가"만 말하고 "바뀌지 않는다"는 말하지 않습니다. `static`에 둔 `Mutex`나 `OnceLock`(Part 1 ch11 2.9절)은 `&'static`으로 빌려도 안의 값이 바뀔 수 있습니다.

### 2.7 지침 3: `'static`을 요구하지 말고, 오래 남겨야 하면 `T: 'static`과 소유한 값으로 받는다

**`&'static T`와 `T: 'static`.** 두 표기는 비슷해 보이지만 요구가 다릅니다. The Reference는 `T: 'a`를 "`T`의 모든 라이프타임 매개변수가 `'a`보다 오래 유효하다"로 정의합니다.

| 표기 | 뜻 | 만족하는 예 | 만족하지 않는 예 |
| --- | --- | --- | --- |
| `&'static T` | 프로그램 끝까지 유효한 **참조** | 리터럴, `static`, 승격된 상수, 누수시킨 값 | 실행 중에 만든 `String`을 빌린 `&str` |
| `T: 'static` | `T` 안에 `'static`보다 짧은 **대여가 없다** | `String`, `Vec<u8>`, `i32`, `Arc<str>`, `&'static str` | 지역 값을 빌린 `&'a str`, 그것을 담은 구조체나 클로저 |

`T: 'static`은 "`T` 값이 영원히 산다"가 아닙니다. 소유한 값은 아무것도 빌리지 않으므로 이 바운드를 만족하지만, 언제든 drop될 수 있습니다. `thread::spawn`이 클로저에 `F: 'static`을 요구하는 이유도 클로저를 오래 살리려는 것이 아니라, 새 스레드가 언제 끝날지 모르니 **빌린 것을 품지 말라**는 것입니다(Part 0 ch15 2.4절).

**문제 상황.** 원고의 문장 수를 백그라운드 스레드에서 셉니다. 원고는 실행 중에 읽은 `String`입니다.

**비권장 코드.** `&str`을 받아 `spawn`에 넘기면 E0521 `borrowed data escapes outside of function`입니다(4.4절). 메시지의 ``argument requires that `'1` must outlive `'static` ``를 보고 매개변수를 `&'static str`로 바꾸면 컴파일은 됩니다.

```rust
fn spawn_count_static(text: &'static str) -> thread::JoinHandle<usize>
```

에러는 사라졌지만 문제는 부르는 쪽으로 옮겨 갔습니다. 실행 중에 읽은 원고는 `'static`이 아니므로, 부르는 쪽은 `Box::leak`으로 누수시켜야만 이 함수를 부를 수 있습니다. [`06_static_bound.rs`](examples/06_static_bound.rs)의 1번 구역에서 원고 두 개를 넘기는 데 70바이트가 영영 해제되지 않습니다.

**권장 코드.** 필요한 것은 "참조가 영원히 유효한 것"이 아니라 "스레드가 가진 값이 빌린 것을 품지 않는 것"이므로 소유한 값을 받습니다.

```rust
fn spawn_count(text: String) -> thread::JoinHandle<usize>

fn spawn_count_any<T>(text: T) -> thread::JoinHandle<usize>
where
    T: AsRef<str> + Send + 'static,
```

2번 구역의 `String`은 스레드로 옮겨 가서 스레드가 끝날 때 drop되므로 누수가 없습니다. 3번 구역의 `spawn_count_any`는 리터럴(`&'static str`), `String`, `Arc<str>`을 모두 받습니다. 스레드로 옮겨 간 `Arc`는 스레드가 끝나며 drop되어 `Arc::strong_count`가 다시 1이 됩니다. `'static` 바운드를 만족한 값도 오래 살지 않았습니다.

**판단 기준.** `'static`이 시그니처에 들어가려 할 때 다음 순서로 묻습니다.

1. 값이 정말 함수(또는 그 함수가 시작한 일)보다 오래 남아야 하는가? 부르는 쪽이 일이 끝날 때까지 기다릴 수 있으면 `'static`이 필요 없습니다. 스코프 스레드(Part 1 ch11 2.2절)로 빌려 줍니다(4번 구역).
2. 오래 남아야 한다면 `T: 'static` 바운드와 **소유한 값**(`String`, `Vec<T>`, `Arc<T>`)으로 받습니다.
3. `&'static T`는 받는 쪽이 리터럴이나 `static` 같은 고정 데이터만 다룬다고 정할 수 있을 때만 씁니다. 에러를 없애려고 붙이는 `'static`은 대개 문제를 부르는 쪽으로 미루는 것입니다.

**지침을 따르지 않아도 되는 경우.** 단위, 명령 이름, 에러 종류처럼 처음부터 리터럴로만 정해지는 값은 `&'static str`이 가장 단순하고 할당도 없습니다(Part 0 ch10 2.17절의 `level_name`). 프로그램이 시작할 때 한 번 읽어 끝날 때까지 쓰는 값이라면 `Box::leak`으로 `&'static`을 얻는 것도 합리적입니다. 누수가 한 번뿐이고, 그 뒤로는 `Arc` 없이 어느 스레드에나 참조를 넘길 수 있습니다. 같은 일을 누수 없이 하는 `OnceLock`은 Part 1 ch11 2.9절에서 다뤘습니다. 스레드와 `'static`의 나머지 이야기는 `ch17-shared-state-parallelism`에서 다룹니다.

### 2.8 지침 4: 짧게 들여다보는 타입에는 참조, 오래 보관하는 타입에는 소유

[ch8](../ch8-references-pointers/README.md)이 이 챕터로 미룬 질문입니다. 슬라이스나 참조를 구조체에 보관해도 되는가, 참조로 충분한 때와 소유가 필요한 때는 언제인가.

**문제 상황.** 자막 파일을 읽어 대사를 다룹니다. 화면에 띄우려고 한 번 훑기도 하고, 여러 편을 불러 목록으로 보관하기도 하고, 책갈피를 오래 남기기도 합니다.

**참조를 담은 판.** 원문을 빌려 가리키는 뷰입니다. 제목과 대사가 모두 같은 원문의 일부이므로 라이프타임은 하나입니다(2.4절의 표).

```rust
struct CueView<'a> { time: &'a str, line: &'a str }
struct TrackView<'a> { title: &'a str, cues: Vec<CueView<'a>> }

fn parse_view(source: &str) -> TrackView<'_>
```

[`07_view_or_owned.rs`](examples/07_view_or_owned.rs)의 1번 구역처럼 원문이 살아 있는 동안 훑어보는 데는 이상적입니다. 대사를 위해 만드는 `String`이 하나도 없습니다. 대신 두 가지 제약이 따라옵니다.

- **라이프타임이 번집니다.** `CueView<'a>`를 담은 `TrackView`에도 `'a`가 필요하고, `TrackView`를 담는 목록, 그 목록을 가진 구조체에도 차례로 번집니다. 그 타입들을 쓰는 함수의 시그니처에도 따라붙습니다.
- **원문이 따로 살아 있어야 합니다.** 2번 구역은 여러 편을 모으려고 원문 보관소 `sources`를 뷰 목록을 다 쓸 때까지 살게 따로 둡니다. 원문을 안쪽 블록에서 읽고 뷰만 바깥 목록에 남기면 E0597이고(4.5절), "파일을 읽어 뷰를 돌려주는 함수"는 만들 수 없습니다. 원문이 그 함수의 지역 값이기 때문입니다(2.2절). 뷰 타입에 `Drop`을 구현하면 제약이 하나 더 붙어, 원문 보관소를 뷰 목록보다 **먼저** 선언해야 합니다. 목록이 버려질 때 뷰의 `drop`이 원문을 볼 수 있기 때문입니다(ch11 2.3절의 규칙. 순서를 바꿔 시험하면 ``borrow might be used here, when `library` is dropped and runs the `Drop` code for type `Vec` ``가 붙은 E0597).

**소유하는 판.**

```rust
struct Cue { time: String, line: String }
struct Track { title: String, cues: Vec<Cue> }

fn load(name: &str) -> Track
```

3번 구역의 `load`는 원문을 함수 안에서 읽어 `Track`을 만들고 원문은 버립니다. 타입에 라이프타임이 없으니 어디에든 넣고 어디로든 옮길 수 있습니다. 대가는 복사입니다. 두 편을 불러오는 데 `String`이 12개 생깁니다.

**판단 기준.**

| 묻는 것 | 참조(뷰) | 소유 |
| --- | --- | --- |
| 가리키는 데이터보다 오래 남는가 | 아니다. 한 함수나 한 블록 안에서 만들고 버림 | 그렇다. 컬렉션, 캐시, 다른 스레드에 보관 |
| 원본의 소유자가 분명하고 따로 살아 있는가 | 그렇다 | 아니다, 또는 따로 관리하기 번거롭다 |
| 복사 비용이 의미 있는가 | 큰 데이터를 자주 훑음 | 작거나 드물게 만듦 |
| 라이프타임이 번질 범위 | 몇 개의 지역 타입 | 공개 타입, 오래 사는 구조체 |

둘을 섞을 수도 있습니다. 4번 구역은 뷰로 훑고, 오래 남길 책갈피만 `to_owned_cue`로 소유한 값으로 바꿉니다. 표준 라이브러리의 `str::lines()`나 `split_whitespace()`가 돌려주는 반복자도 원문을 빌린 뷰라서 원문보다 오래 쓸 수 없습니다. 확신이 없으면 소유하는 판으로 시작합니다. 측정해서 복사가 문제라고 드러난 곳만 뷰로 바꾸는 편이 반대 방향보다 쉽습니다(`ch20-avoid-over-optimizing`).

**지침을 따르지 않아도 되는 경우.** 여러 곳이 같은 큰 데이터를 오래 함께 써야 한다면 복사도 참조도 아닌 공유 소유가 답일 수 있습니다. 원문을 `Rc<str>`이나 `Arc<str>`([Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.8절)로 공유하고 대사의 위치를 바이트 범위로 보관하면, 라이프타임 매개변수 없이 복사도 한 번으로 끝납니다(연습 4). 자기 자신이 가진 데이터를 가리키는 자기 참조 구조체는 이 방법들로도 만들기 어렵고, `ch15-borrow-checker`에서 다룹니다.

### 2.9 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **숨은 라이프타임에 기본 경고가 생겼습니다.** 책은 `'_`를 읽는 사람을 위한 표시로 소개합니다. 이 툴체인의 릴리스 노트에 따르면 1.89.0부터 기본으로 켜진 `mismatched_lifetime_syntaxes`가 생겨, 생략된 입력 라이프타임이 반환 타입의 경로에 숨으면 경고하고 `'_`를 제안합니다. 입력에만 숨은 경우는 여전히 기본으로 꺼진 `elided_lifetimes_in_paths`가 맡습니다(2.5절).
- **임시값 범위.** "임시값은 문장 끝에서 drop된다"는 설명은 2024 에디션에서 블록의 마지막 표현식과 `if let` 조건식에 대해 "더 일찍"으로 바뀌었고, 임시값 수명 연장은 1.79.0과 1.89.0에서 넓어졌습니다(2.3절). 함수 인자로 넘긴 임시값을 빌린 결과를 다음 문장에서 쓰면 지금도 E0716입니다.
- **에러 메시지의 도움말.** 입력 참조가 없는 함수의 E0106은 `'static`을 쓰라는 도움말 뒤에 `help: instead, you are more likely to want to return an owned value`를 덧붙이고 반환 타입을 `String`으로 바꾼 코드를 보여 줍니다. 본문은 바꾸지 않으므로 그대로 따르면 E0308이 나고, 그 도움말이 `.to_string()`을 제안합니다. 참조를 컬렉션에 모으다 난 E0597에는 `note: ... is a collection that stores borrowed references` 같은 설명이 붙습니다(4.2절).
- **`String::leak`(1.72.0).** `Box::leak(s.into_boxed_str())` 대신 쓸 수 있습니다. 상수 승격의 조건(`Drop`이나 내부 가변성이 있으면 승격하지 않음)은 책의 설명과 같습니다.
- **책에 없는 지침.** 다음은 책의 Item 14가 다루지 않거나 권하지 않는 내용입니다(2026-11-19에 온라인판에서 확인).
  - 2.5절의 RPIT 캡처와 `use<'a>`: 2024 에디션 규칙(Part 2 ch4 2.3절)이 근거입니다.
  - 2.7절 전체(`&'static T`와 `T: 'static`의 구별, 오래 남길 값은 `T: 'static`과 소유한 값으로 받기): 책은 `'static`을 스코프를 벗어나지 않는 데이터(전역, 누수시킨 힙 값)에 쓴다고 정리하고 `T: 'static` 바운드는 다루지 않습니다. `thread::spawn`의 시그니처와 The Reference의 Lifetime bounds 절이 근거입니다.
  - 2.8절의 판단 기준표와 섞어 쓰기: 책은 데이터를 소유하는 편이 쉽고 스마트 포인터가 라이프타임 제약을 풀어 준다고 권합니다. "짧게 보는 뷰에는 참조, 오래 보관하는 값에는 소유"라는 구분은 이 챕터가 더한 것입니다.

### 2.10 기억할 것

- 함수가 돌려주는 참조는 그 함수의 프레임 밖에 있는 값만 가리킬 수 있습니다. 힙의 값도 소유 사슬의 끝(지역 변수나 `static`)만큼 삽니다.
- 값은 drop되거나 이동될 때, 참조는 마지막 사용에서, 임시값은 대개 문장 끝에서 끝납니다. 대여 검사기는 메모리가 아니라 소유자를 추적합니다.
- 라이프타임 표기는 "같은 구간 동안 유효하다"는 관계만 적습니다. 결과가 오는 입력에만 이름을 주고, 출처가 다르면 나눕니다. 컴파일러의 도움말은 모두 묶자고 제안합니다.
- 생략된 라이프타임도 시그니처에 있습니다. 의도와 다르면 직접 적고, 경로에 숨은 것은 `'_`로 드러냅니다.
- `&'static T`는 영원히 유효한 참조, `T: 'static`은 짧은 대여를 품지 않은 타입입니다. 에러를 없애려고 `'static`이나 `Box::leak`을 쓰지 말고 소유한 값으로 받습니다.
- 짧게 들여다보는 타입에는 참조, 오래 보관하는 타입에는 소유를 고릅니다.

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행하며, 모든 예제는 종료 코드 0으로 끝납니다. 스레드를 쓰는 06은 스레드 안에서 출력하지 않고 결과를 돌려받아 `main`에서 출력하므로 출력이 실행마다 같습니다.

| 파일 | 다루는 것 | 비권장 형태(또는 잘못된 짐작) | 권장 형태(또는 맞는 모형) | 절 |
| --- | --- | --- | --- | --- |
| [`01_stack_frames.rs`](examples/01_stack_frames.rs) | 참조가 나갈 수 있는 곳 | 지역 값의 참조 반환(주석) | 값 반환, 매개변수에서 얻은 참조, `'static` | 2.2 |
| [`02_lifetime_spans.rs`](examples/02_lifetime_spans.rs) | 값·참조·임시값의 구간 | 이동한 뒤 참조 사용(주석) | 마지막 사용 뒤 이동, 임시값 수명 연장 | 2.3 |
| [`03_split_lifetimes.rs`](examples/03_split_lifetimes.rs) | 묶을지 나눌지 | `BoundHit<'a>` | `Hit<'g, 'q>`, 묶어야 하는 `meaning_or_query` | 2.4 |
| [`04_elided_lifetimes.rs`](examples/04_elided_lifetimes.rs) | 생략과 `'_` | `title_elided`, `-> PostView` | `-> &'a str`, `-> PostView<'_>` | 2.5 |
| [`05_static_sources.rs`](examples/05_static_sources.rs) | `'static` 참조의 출처 | — | 리터럴, `static`, 상수 승격, `Box::leak` | 2.6 |
| [`06_static_bound.rs`](examples/06_static_bound.rs) | `&'static T`와 `T: 'static` | `&'static str` 매개변수 + 누수 | `String`, `T: 'static`, 스코프 스레드 | 2.7 |
| [`07_view_or_owned.rs`](examples/07_view_or_owned.rs) | 참조를 담을지 소유할지 | — | 뷰, 소유, 섞어 쓰기 | 2.8 |

### 01_stack_frames — 참조가 나갈 수 있는 곳

관찰용 타입 `Tracked`는 drop될 때 `[drop] 이름`을 출력합니다. 1번 구역은 받는 쪽의 지역 값이 함수 끝에서 drop되는 것을, 2번 구역은 돌려준 값이 받는 쪽의 `}`에서 drop되지 않고 부르는 쪽 블록이 끝날 때 drop되는 것을 보여 줍니다. 4번 구역은 다른 함수를 불러 프레임이 바뀌어도 `'static` 참조가 그대로임을, 5번 구역은 힙의 레시피를 가리키는 참조가 소유 사슬의 끝인 `shelf`를 따른다는 것을 보여 줍니다.

```sh
cargo run -p er-ch14-lifetimes --example 01_stack_frames
```

```text
1. 받는 쪽의 지역 값은 프레임과 함께 사라집니다
    [drop] 저울(받는 쪽의 지역 값)
    돌아온 값: 300g
2. 값을 돌려주면 부르는 쪽으로 이동합니다
    받는 쪽에서 만듦: 냄비(받는 쪽에서 만들어 돌려준 값)
    부르는 쪽이 받은 값: 냄비(받는 쪽에서 만들어 돌려준 값)
    [drop] 냄비(받는 쪽에서 만들어 돌려준 값)
3. 매개변수에서 얻은 참조는 부르는 쪽의 값을 가리킵니다
    주재료: 된장 / 장보기: 된장찌개: 된장, 두부, 애호박
4. 'static 참조는 어느 프레임에도 속하지 않습니다
    [drop] 저울(받는 쪽의 지역 값)
    단위: g
5. 힙의 값도 소유 사슬의 끝(스택 변수)이 살아 있는 동안만 삽니다
    선반의 첫 레시피 주재료: 밥
```

### 02_lifetime_spans — 값·참조·임시값의 구간

1·2번 구역은 참조의 마지막 사용 뒤에 원본을 고치고 옮기는 코드, 3·4번 구역은 임시값이 drop되는 시점입니다. 3번 구역의 `[drop]`은 그 `println!` 문장이 끝난 직후에, 4번 구역의 `[drop]`은 블록이 끝날 때 나옵니다.

```sh
cargo run -p er-ch14-lifetimes --example 02_lifetime_spans
```

```text
1. 참조는 마지막 사용에서 끝납니다
    제목: 장보기
    고친 초안: "장보기\n두부, 대파, 계란"
2. 값의 라이프타임은 이동에서도 끝납니다
    보관하기 전 제목: 장보기
    보관함의 첫 줄: 장보기
3. 임시값은 문장이 끝날 때 drop됩니다
    문장 안에서 쓰기: 임시 쪽지
    [drop] 임시 쪽지
    다음 문장
4. let으로 바로 묶은 임시값은 블록 끝까지 삽니다(임시값 수명 연장)
    블록 안: 묶어 둔 쪽지
    블록의 마지막 문장
    [drop] 묶어 둔 쪽지
```

### 03_split_lifetimes — 묶을지 나눌지

같은 입력 네 개(`"  Borrow "`, `"LIFETIME"`, `"closure"`, `"trait"`)로 세 함수를 부릅니다. `closure`는 용어집에 없습니다. 1·2번 구역의 결과는 같고, 달라진 것은 부르는 쪽이 복사해야 했는가뿐입니다.

```sh
cargo run -p er-ch14-lifetimes --example 03_split_lifetimes
```

```text
1. 비권장: 두 출처를 'a 하나로 묶은 결과
    borrow → 대여
    lifetime → 라이프타임
    trait → 트레이트
    찾은 뜻 ["대여", "라이프타임", "트레이트"], 복사한 String 3개
2. 권장: 출처마다 라이프타임을 따로 둔 결과
    borrow → 대여
    lifetime → 라이프타임
    trait → 트레이트
    찾은 뜻 ["대여", "라이프타임", "트레이트"], 복사한 String 0개
3. 묶어야 하는 경우: 결과가 어느 입력에서든 올 수 있을 때
    borrow → 대여
    lifetime → 라이프타임
    closure → closure
    trait → 트레이트
```

### 04_elided_lifetimes — 생략과 `'_`

1번 구역은 생략된 판이 문제없이 동작하는 경우, 2번 구역은 `'a`를 적은 판이 필요한 두 경우(지역 뷰에서 제목을 돌려주는 `title_of`, 임시 뷰로 제목 모으기)입니다. 3번 구역은 숨긴 판과 `'_` 판의 결과가 같음을 확인합니다. 본문 글자 수는 공백과 마침표를 포함한 `chars().count()`입니다.

```sh
cargo run -p er-ch14-lifetimes --example 04_elided_lifetimes
```

```text
1. 생략된 판도 뷰가 살아 있는 동안에는 잘 동작합니다
    제목: 러스트 첫걸음
2. 결과가 뷰보다 오래 살아야 하면 'a를 직접 적은 판이 필요합니다
    도우미가 돌려준 제목: 대여 검사기와 친해지기
    모은 제목: ["러스트 첫걸음", "대여 검사기와 친해지기"]
3. '_는 생략된 라이프타임이 있다는 표시일 뿐, 뜻은 같습니다
    숨긴 판: 《러스트 첫걸음》 본문 21자
    '_ 판: 《러스트 첫걸음》 본문 21자
```

### 05_static_sources — `'static` 참조의 출처

네 출처를 차례로 씁니다. 4번 구역은 실행 중에 만든 이름 세 개를 누수시켜 `&'static str`로 바꿉니다. 42바이트는 `"탐사선-N호"` 하나(14바이트)의 세 배이고, 이 메모리는 프로그램이 끝날 때까지 해제되지 않습니다.

```sh
cargo run -p er-ch14-lifetimes --example 05_static_sources
```

```text
1. 문자열 리터럴
    공전 주기의 단위: 일
2. static 항목
    ["수성", "금성", "지구", "화성"]
3. 상수 승격
    지구의 1년: 365일
    안쪽 행성의 공전 주기: [88, 225, 365, 687]
    두 해: 730일
4. Box::leak (누수)
    ["탐사선-1호", "탐사선-2호", "탐사선-3호"]
    되찾을 수 없게 된 바이트: 42 (부를 때마다 늘어납니다)
```

### 06_static_bound — `&'static T`와 `T: 'static`

1·2번 구역은 같은 원고 두 개를 `&'static str` 판과 `String` 판에 넘깁니다. 3번 구역은 `T: 'static` 바운드에 세 종류의 값을 넘기고(리터럴 원고의 문장은 하나), 4번 구역은 스코프 스레드로 빌려 줍니다.

```sh
cargo run -p er-ch14-lifetimes --example 06_static_bound
```

```text
1. 비권장: &'static str을 요구하면 부르는 쪽이 누수시켜야 합니다
    문장 수 [2, 2], 되찾을 수 없게 된 바이트 70
2. 권장: 소유한 String을 넘기면 스레드가 끝날 때 drop됩니다
    문장 수 [2, 2], 누수 0바이트
3. T: 'static은 '오래 산다'가 아니라 '짧은 대여가 없다'입니다
    문장 수 [1, 2, 2]
    스레드가 끝난 뒤 Arc의 소유자 수: 1
4. 기다릴 수 있다면 'static 자체가 필요 없습니다(스코프 스레드)
    문장 수 [2, 2], 복사 0번
```

### 07_view_or_owned — 참조를 담을지 소유할지

가짜 파일 시스템 `read_subtitle`이 자막 두 편을 새 `String`으로 돌려줍니다. 3번 구역의 12는 편마다 제목 1개와 대사마다 2개(시각, 대사)를 더한 값입니다(1화 7개, 2화 5개).

```sh
cargo run -p er-ch14-lifetimes --example 07_view_or_owned
```

```text
1. 뷰: 원문이 살아 있는 동안 복사 없이 훑어봅니다
    1화: 대사 3줄, 가장 긴 대사 "문이 닫히기 전에 타야 해"
    대사를 위해 만든 String: 0개
2. 뷰를 모아 두려면 원문을 더 오래 사는 곳에 따로 보관해야 합니다
    1화: 3줄
    2화: 2줄
3. 소유: 원문을 버려도 되므로 불러오는 함수가 단순해집니다
    2편 불러옴, Track에 든 String: 12개
    2화 첫 대사: 00:02 다시 만났네
4. 섞어 쓰기: 뷰로 훑고, 오래 보관할 것만 소유로 바꿉니다
    책갈피: 00:06 우산 가져왔어?
```

## 4. 자주 하는 실수

이 챕터의 에러는 모두 "참조가 가리키는 값보다 오래 쓰인다"는 한 가지 모양입니다. 4.2절과 4.3절처럼 **에러가 부르는 쪽 줄을 가리켜도 원인은 받는 쪽 시그니처**인 경우가 많으므로, 표시된 줄보다 그 줄이 부르는 함수의 시그니처를 먼저 봅니다. 챕터 디렉터리에서 다음처럼 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch14-lifetimes
rustc --edition 2024 compile_fail/e0505_move_while_borrowed.rs --out-dir /tmp/er-ch14
rustc --edition 2024 compile_fail/e0597_one_lifetime_two_sources.rs --out-dir /tmp/er-ch14
rustc --edition 2024 compile_fail/e0515_elided_output_tied_to_view.rs --out-dir /tmp/er-ch14
rustc --edition 2024 compile_fail/e0521_spawn_borrowed_param.rs --out-dir /tmp/er-ch14
rustc --edition 2024 compile_fail/e0597_view_outlives_source.rs --out-dir /tmp/er-ch14
```

### 4.1 빌려준 값을 옮긴 뒤 참조를 씀 — E0505

파일: [`compile_fail/e0505_move_while_borrowed.rs`](compile_fail/e0505_move_while_borrowed.rs)

```text
error[E0505]: cannot move out of `draft` because it is borrowed
  --> compile_fail/e0505_move_while_borrowed.rs:21:18
   |
18 |     let draft = String::from("장보기\n두부, 대파");
   |         ----- binding `draft` declared here
19 |     let title = first_line(&draft);
   |                            ------ borrow of `draft` occurs here
20 |     let mut archive: Vec<String> = Vec::new();
21 |     archive.push(draft); // E0505: 빌려준 값을 옮김
   |                  ^^^^^ move out of `draft` occurs here
22 |     println!("보관한 목록의 제목: {title}"); // 옮긴 뒤에 참조를 씀
   |                                    ----- borrow later used here
   |
help: consider cloning the value if the performance cost is acceptable
   |
19 |     let title = first_line(&draft.clone());
   |                                  ++++++++
```

- **읽는 법**: 세 표시가 "빌림(19행) → 값의 구간이 끝남(21행, 이동) → 그 뒤 참조 사용(22행)"을 차례로 보여 줍니다. drop 대신 이동으로 끝났다는 것만 E0597과 다릅니다.
- **원인**: 값의 라이프타임은 이동에서도 끝납니다. 힙의 글자가 그대로여도 대여 검사기는 소유자를 따집니다(2.3절).
- **해결**: 도움말을 그대로 따르면 안 됩니다. `first_line(&draft.clone())`은 복제본이 그 문장 끝에서 drop되는 임시값이라 E0716 `temporary value dropped while borrowed`로 바뀔 뿐입니다. (가) 참조를 이동보다 먼저 다 쓰거나(예제 02의 2번 구역), (나) 옮긴 뒤 새 소유자에게서 다시 빌리거나(`first_line(&archive[0])`), (다) 제목이 이동 뒤에도 필요하면 `title.to_string()`으로 소유한 값을 만들어 둡니다.

### 4.2 출처가 다른 두 입력을 라이프타임 하나로 묶음 — E0597

파일: [`compile_fail/e0597_one_lifetime_two_sources.rs`](compile_fail/e0597_one_lifetime_two_sources.rs)

```text
error[E0597]: `query` does not live long enough
  --> compile_fail/e0597_one_lifetime_two_sources.rs:31:53
   |
28 |     let mut found: Vec<&str> = Vec::new();
   |         --------- variable `found` declared here
29 |     for raw in ["  Borrow ", "LIFETIME"] {
30 |         let query = raw.trim().to_lowercase();
   |             ----- binding `query` declared here
31 |         if let Some(hit) = look_up_bound(&glossary, &query) {
   |                                                     ^^^^^^ borrowed value does not live long enough
...
35 |     }
   |     - `query` dropped here while still borrowed
36 |     println!("{found:?}");
   |                ----- borrow later used here
   |
   = note: `found` is a collection that stores borrowed references, but `query` does not live long enough to be stored in it
   = help: buffer reuse with borrowed references requires unsafe code or restructuring
```

- **읽는 법**: 에러는 `query`를 탓하지만 `found`에 넣은 것은 용어집에서 온 `hit.meaning`입니다. 이렇게 "넣은 적 없는 값이 오래 살아야 한다"는 에러가 나면, 그 값을 거쳐 간 함수나 타입의 시그니처가 두 출처를 같은 이름으로 묶었는지 봅니다.
- **원인**: `look_up_bound`와 `BoundHit`이 용어집과 검색어를 `'a` 하나로 묶어서, `'a`가 반복마다 drop되는 검색어에 맞춰졌습니다(2.4절).
- **해결**: 받는 쪽을 고칠 수 있으면 `Hit<'g, 'q>`처럼 출처마다 라이프타임을 나눕니다(예제 03의 2번 구역). 본문은 바꾸지 않아도 됩니다. 마지막 도움말(unsafe나 구조 변경)은 버퍼를 재사용하는 경우를 위한 일반적인 안내라서 여기에는 맞지 않습니다. 받는 쪽을 고칠 수 없다면 부르는 쪽에서 `hit.meaning.to_string()`으로 복사합니다(예제 03의 1번 구역).

### 4.3 생략 규칙이 반환값을 뷰에 묶음 — E0515

파일: [`compile_fail/e0515_elided_output_tied_to_view.rs`](compile_fail/e0515_elided_output_tied_to_view.rs)

```text
error[E0515]: cannot return value referencing local variable `view`
  --> compile_fail/e0515_elided_output_tied_to_view.rs:30:5
   |
30 |     view.title_elided() // E0515: 지역 변수 view를 빌린 값을 돌려줌
   |     ----^^^^^^^^^^^^^^^
   |     |
   |     returns a value referencing data owned by the current function
   |     `view` is borrowed here
```

- **읽는 법**: `title_of`는 글 `post`를 받았고 제목은 그 글의 조각인데, 컴파일러는 "지역 변수 `view`를 빌린 값"이라고 말합니다. 컴파일러가 `title_elided`의 본문이 아니라 생략된 시그니처 `(&'s self) -> &'s str`를 믿기 때문입니다.
- **원인**: 생략 규칙 3이 반환값을 `&self`, 즉 함수가 끝나면 사라지는 뷰에 묶었습니다(2.5절).
- **해결**: 메서드를 `fn title(&self) -> &'a str`로 고칩니다(예제 04). 메서드를 고칠 수 없다면 뷰를 부르는 쪽에 만들어 넘기거나, 제목을 `to_string()`으로 복사해 돌려줍니다.

### 4.4 빌린 매개변수를 `'static` 바운드에 넘김 — E0521

파일: [`compile_fail/e0521_spawn_borrowed_param.rs`](compile_fail/e0521_spawn_borrowed_param.rs)

```text
error[E0521]: borrowed data escapes outside of function
  --> compile_fail/e0521_spawn_borrowed_param.rs:21:5
   |
20 | fn spawn_count_borrowed(text: &str) -> thread::JoinHandle<usize> {
   |                         ----  - let's call the lifetime of this reference `'1`
   |                         |
   |                         `text` is only valid in the function body
21 |     thread::spawn(move || count_sentences(text)) // E0521
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |     |
   |     `text` escapes the function body here
   |     argument requires that `'1` must outlive `'static`
   |
note: `'static` lifetime requirement from `spawn` introduced here
   (표준 라이브러리 소스 위치 생략)
```

- **읽는 법**: `'1`은 컴파일러가 이름 없는 라이프타임에 붙인 임시 이름입니다. "`'1`이 `'static`보다 오래 살아야 한다"는 요구가 `spawn`의 바운드에서 왔다고 `note`가 짚어 줍니다. 지역 변수를 빌린 클로저를 `main`에서 바로 넘기면 같은 원인이 E0373(Part 0 ch15 4.1절)이나 E0597로 나타납니다.
- **원인**: `F: 'static`은 클로저가 빌린 것을 품지 않기를 요구하는데, 클로저가 부르는 쪽이 정한 구간 `'1`의 참조를 품었습니다(2.7절).
- **해결**: 매개변수를 `&'static str`로 바꾸면 컴파일되지만 부르는 쪽에 누수를 떠넘깁니다(예제 06의 1번 구역). `String`이나 `T: AsRef<str> + Send + 'static`으로 소유한 값을 받거나(2·3번 구역), 결과를 기다릴 수 있으면 스코프 스레드로 빌려 줍니다(4번 구역).

### 4.5 뷰를 담은 컬렉션이 원문보다 오래 삶 — E0597

파일: [`compile_fail/e0597_view_outlives_source.rs`](compile_fail/e0597_view_outlives_source.rs)

```text
error[E0597]: `source` does not live long enough
  --> compile_fail/e0597_view_outlives_source.rs:41:33
   |
40 |         let source = read_subtitle("1화");
   |             ------ binding `source` declared here
41 |         library.push(parse_view(&source)); // E0597
   |                                 ^^^^^^^ borrowed value does not live long enough
42 |     } // source가 여기서 drop됩니다.
   |     - `source` dropped here while still borrowed
43 |     for track in &library {
   |                  -------- borrow later used here
```

- **읽는 법**: `library`에는 원문이 아니라 `TrackView`를 넣었지만, `TrackView<'a>`의 라이프타임이 `Vec<TrackView<'a>>`로 번졌으므로 `library`를 쓰는 43행이 원문의 대여를 이어 갑니다.
- **원인**: 참조를 담은 타입은 가리키는 값보다 오래 살 수 없고, 그것을 담은 컬렉션도 마찬가지입니다(2.8절).
- **해결**: 원문을 `library`보다 오래 사는 곳에 따로 보관하거나(예제 07의 2번 구역), 오래 보관할 것을 소유한 타입으로 바꿔 담습니다(3·4번 구역).

### 4.6 그 밖의 에러

| 코드 | 결과 | 다룬 곳 |
| --- | --- | --- |
| 함수 안에서 만든 값의 참조를 돌려줌 | E0515 | Part 0 ch10 4.5절, 이 챕터 2.2절 |
| 입력 참조가 둘인데 반환 참조에 표기 없음, 참조 필드에 표기 없음 | E0106 | Part 0 ch10 4.3절, [Part 0 ch5](../../part0-getting-started/ch5-structs/README.md) 4.5절 |
| 임시값을 빌린 결과를 다음 문장에서 씀 | E0716 | Part 1 ch3 4.2절 |
| 지역 변수의 `Box`를 `&'static`으로 돌려주거나 빌림 | E0515, E0597 | 2.6절 |
| `Drop`이나 내부 가변성이 있는 `const`의 참조를 `&'static`으로 돌려줌 | E0515 | 2.6절 |
| 본문이 시그니처의 라이프타임 관계를 지키지 못함 | 코드 없는 `lifetime may not live long enough` | 2.4절, Part 0 ch10 2.16절 |
| 2024 에디션 RPIT가 `&self`를 캡처해 지역 뷰에서 돌려줄 수 없음 | E0597 | 연습 2 (같은 캡처 때문에 원본을 고치지 못하는 E0502는 Part 2 ch4 4.4절) |
| 지역 값을 빌린 `&str`을 `&dyn Any`로 바꿈 | E0597 | [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 4.2절 |

## 5. 연습 문제

### 연습 1. 시그니처의 관계 읽기

다음 시그니처마다 반환값이 어느 입력에 묶이는지 말하고, 이 챕터의 기준으로 고칠 곳이 있으면 고치세요.

1. `fn tag_or_default<'a>(tag: &'a str, default: &'a str) -> &'a str`: 태그가 비어 있으면 기본값을 돌려줍니다.
2. `fn find_label<'a>(labels: &'a [String], key: &'a str) -> Option<&'a String>`: 키로 시작하는 라벨을 찾습니다.
3. `impl<'a> Tokens<'a> { fn rest(&self) -> &str }`: `Tokens`가 빌린 입력 문자열에서 아직 읽지 않은 부분을 돌려줍니다.
4. `fn parse(input: &str) -> Tokens`

<details>
<summary>힌트</summary>

1번은 결과가 두 입력 중 어느 쪽에서든 오므로 묶는 것이 맞습니다. 2번은 결과가 `labels`에서만 오므로 `key`의 `'a`를 지워 `key: &str`로 둡니다. 그러면 반복마다 만든 검색 키로 찾은 라벨을 반복 밖에 모을 수 있습니다. 3번은 생략 규칙 3 때문에 결과가 `&self`에 묶이므로, 입력 문자열에서 온다는 사실대로 `-> &'a str`로 적습니다. 4번은 뜻은 맞지만 결과가 `input`을 빌린다는 것이 숨어 있으므로 `-> Tokens<'_>`로 적습니다. 그대로 두면 `mismatched_lifetime_syntaxes` 경고가 납니다.

</details>

### 연습 2. 반복자를 돌려주는 뷰 메서드

예제 04의 `PostView`에 본문 줄을 차례로 내주는 메서드 `fn body_lines(&self) -> impl Iterator<Item = &'a str>`(`self.text.lines().skip(1)`)를 추가하고, 다음 도우미를 컴파일해 보세요. 에러가 나면 메서드의 시그니처만 고쳐 통과시키세요.

```rust
fn body_of(post: &str) -> impl Iterator<Item = &str> {
    let view = PostView::new(post);
    view.body_lines()
}
```

<details>
<summary>힌트</summary>

`Item`을 `&'a str`로 적었는데도 ``E0597: `view` does not live long enough``가 납니다(이 툴체인에서 확인). 2024 에디션의 반환 위치 `impl Trait`은 스코프의 라이프타임을 모두 캡처하므로, 반환된 반복자가 `&self`의 구간에도 묶이기 때문입니다. 반환 타입에 `+ use<'a>`를 붙여 `'a`만 캡처하게 하면 통과합니다. 반복자가 실제로 쓰는 것은 `self.text`(`&'a str`)뿐이므로 사실 그대로의 표기입니다.

</details>

### 연습 3. `'static`을 돌려주는 함수

사용자 이름을 프로그램 시작 때 한 번 입력받고, 프로그램 곳곳에서 `fn current_user() -> &'static str`로 읽으려 합니다. 이 시그니처를 지키는 방법 두 가지를 쓰고, 각각의 대가를 비교하세요. 시그니처를 바꾼다면 무엇으로 바꾸는 것이 좋을지도 말해 보세요.

<details>
<summary>힌트</summary>

(가) 입력받은 `String`을 `String::leak`으로 한 번 누수시켜 얻은 `&'static str`을 `static NAME: OnceLock<&'static str>`에 넣어 둡니다. 누수는 한 번뿐이라 이 경우에는 합리적입니다(2.7절). (나) `static USER: OnceLock<String>`에 넣고 `USER.get().map_or("손님", |s| s.as_str())`로 돌려줍니다. 누수가 없고, 아직 입력받지 않았을 때의 처리도 드러납니다. 이름을 실행 중에 바꿔야 한다면 `&'static str`로는 표현할 수 없으므로 `String`이나 `Arc<str>`을 돌려주는 시그니처로 바꿉니다.

</details>

### 연습 4. 라이프타임 없는 공유 자막

예제 07을 바꿔, 원문을 `Rc<str>`로 한 번만 가지고 대사의 위치를 바이트 범위로 보관하는 `SharedTrack { source: Rc<str>, lines: Vec<Range<usize>> }`를 만드세요. `fn line(&self, index: usize) -> Option<&str>`로 대사를 꺼내고, `load` 같은 함수가 원문을 읽어 `SharedTrack`을 돌려줄 수 있는지 확인하세요.

<details>
<summary>힌트</summary>

`Rc::from(string)`으로 원문을 옮겨 담고, `split_inclusive('\n')`로 줄을 돌며 시작 위치를 더해 나가면 범위를 구할 수 있습니다. `line`은 `self.lines.get(index).map(|r| &self.source[r.clone()])`이고, 생략 규칙 3대로 결과가 `&self`에 묶이는 것이 이번에는 맞습니다. 원문을 `SharedTrack` 자신이 가지기 때문입니다. 타입에 라이프타임 매개변수가 없으므로 `load`가 돌려줄 수 있고, 여러 곳에서 `Rc::clone`으로 같은 원문을 나눠 가져도 글자는 복사되지 않습니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 14: Understand lifetimes](https://effective-rust.com/lifetimes.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- The Rust Reference
  - [Lifetime elision](https://doc.rust-lang.org/reference/lifetime-elision.html): 생략 규칙과 `'_`, `static`·`const` 항목의 기본 `'static`
  - [Trait and lifetime bounds](https://doc.rust-lang.org/reference/trait-bounds.html#lifetime-bounds): `'a: 'b`와 `T: 'a`의 정의(2.4절, 2.7절)
  - [Destructors](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes): 임시값 범위, [임시값 수명 연장](https://doc.rust-lang.org/reference/destructors.html#temporary-lifetime-extension), [상수 승격](https://doc.rust-lang.org/reference/destructors.html#constant-promotion)
- 표준 라이브러리 문서: [`thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html), [`Box::leak`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak), [`String::leak`](https://doc.rust-lang.org/std/string/struct.String.html#method.leak)
- rustc 린트: [`mismatched_lifetime_syntaxes`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#mismatched-lifetime-syntaxes), [`elided_lifetimes_in_paths`](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#elided-lifetimes-in-paths)
- [Clippy 린트 목록(1.100.0)](https://clippy.rust-lang.org/rust-1.100.0/index.html): `needless_lifetimes`, `elidable_lifetime_names`
- 에러 설명: `rustc --explain E0505`, `rustc --explain E0515`, `rustc --explain E0521`, `rustc --explain E0597`
