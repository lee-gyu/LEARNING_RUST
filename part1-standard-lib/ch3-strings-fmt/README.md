# ch3. 문자열과 서식: `String`, `&str`, `std::fmt`, `FromStr`

[Part 0 ch8](../../part0-getting-started/ch8-collections/README.md)에서 `String`과 `&str`의 차이, 이어 붙이기, UTF-8 때문에 정수 인덱스를 쓸 수 없는 이유를 배웠습니다. [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md)에서는 `println!`의 서식 지정을, [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md)에서는 `Display`를 `write!`로 구현하는 방법을 배웠습니다. 이 챕터는 그 위에 세 가지를 넓힙니다.

- **문자열 메서드**: 찾기, 나누기, 다듬기, 바꾸기, 제자리 수정, 안전하게 자르기, 바이트와의 변환. 메서드가 많지만 시그니처(signature)를 읽으면 "새 `String`을 만드는가, 원본을 빌린 `&str`을 돌려주는가"를 바로 알 수 있습니다.
- **서식(`std::fmt`)**: 서식 문자열 문법 전체와, 직접 구현한 `Display`·`Debug`가 너비·정렬 같은 서식 지정을 따르게 만드는 방법.
- **파싱(parsing, `FromStr`)**: `parse::<T>()`가 실제로 무엇을 부르는지, 직접 만든 타입을 문자열에서 만드는 방법.

- 패키지: `p1-ch3-strings-fmt`
- 기준 문서: [`std::string`](https://doc.rust-lang.org/std/string/index.html), [`std::str`](https://doc.rust-lang.org/std/str/index.html)과 [원시 타입 `str`](https://doc.rust-lang.org/std/primitive.str.html), [`std::char`](https://doc.rust-lang.org/std/char/index.html), [`std::fmt`](https://doc.rust-lang.org/std/fmt/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024). 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 하고 싶은 일(찾기·나누기·다듬기·바꾸기·잇기·제자리 수정)에 맞는 문자열 메서드를 고를 수 있고, 시그니처의 `&self`/`&mut self`와 반환 타입을 보고 그 메서드가 새 `String`을 할당하는지, 원본을 빌린 `&str`을 돌려주는지 설명할 수 있다. 검색 패턴 자리에 `char`, `&str`, `char` 배열, 클로저를 넣어 쓸 수 있다.
2. 바이트 위치와 글자 경계를 구분해 `get`, `is_char_boundary`, `char_indices`로 패닉 없이 문자열을 자를 수 있고, `from_utf8`과 `from_utf8_lossy`로 바이트를 문자열로 바꿀 수 있다.
3. 서식 문법 전체(위치·이름·캡처 인자, 채움·정렬·너비, 부호·`#`·`0`, 정밀도와 `$`·`.*`, 서식 트레이트)를 써서 원하는 모양을 만들 수 있고, `format!`·`write!`·`format_args!`의 관계를 설명할 수 있다.
4. `write!`만 쓴 `Display` 구현이 `{:>10}`을 무시하는 이유를 설명하고, `Formatter`의 `pad`·`width`·`precision`·`alternate`로 서식 지정을 따르게 고칠 수 있다. `Debug`를 `debug_struct` 같은 도우미로 직접 구현할 수 있다.
5. `parse::<T>()`가 `FromStr::from_str`을 부른다는 것을 설명하고, 직접 만든 타입에 `Display`와 왕복되는 `FromStr`을 구현할 수 있다.

## 2. 핵심 개념

2.1~2.6절은 문자열 메서드, 2.7절은 글자 경계와 바이트, 2.8~2.11절은 서식, 2.12절은 파싱을 다룹니다.

### 2.1 시그니처로 읽는 문자열 메서드

`String`은 `Deref<Target = str>`를 구현하므로 `str`의 메서드를 모두 쓸 수 있습니다(API 문서의 "Methods from `Deref<Target = str>`", [ch1](../ch1-std-overview/README.md)). 그래서 문자열 메서드는 대부분 `str`에 있고, `String`에는 내용을 바꾸는 메서드가 따로 있습니다. 메서드가 수십 개지만 시그니처의 모양은 몇 가지뿐입니다.

| 시그니처 모양 | 뜻 | 할당 | 예 |
| --- | --- | --- | --- |
| `fn contains<P: Pattern>(&self, pat: P) -> bool` | 조사만 합니다 | 없음 | `contains`, `starts_with`, `is_char_boundary` |
| `fn find<P: Pattern>(&self, pat: P) -> Option<usize>` | 바이트 위치를 알려 줍니다 | 없음 | `find`, `rfind` |
| `fn trim(&self) -> &str` | 원본을 빌린 조각을 돌려줍니다 | 없음 | `trim` 계열, `strip_prefix`(`Option<&str>`), `split_once`(`Option<(&str, &str)>`), `get` |
| `fn split<P: Pattern>(&self, pat: P) -> Split<'_, P>` | 원본을 빌린 조각을 내주는 반복자 | 없음 | `split` 계열, `split_whitespace`, `lines`, `matches` |
| `fn replace<P: Pattern>(&self, from: P, to: &str) -> String` | 새 `String`을 만듭니다 | 있음 | `replace`, `to_uppercase`, `repeat` |
| `fn truncate(&mut self, new_len: usize)` | `String` 자체를 바꿉니다 | 용량이 모자랄 때만 재할당 | `insert`, `remove`, `retain`, `drain` |
| `fn into_bytes(self) -> Vec<u8>` | `String`을 소비해 다른 타입으로 넘깁니다 | 없음 | `into_bytes` |

두 가지만 기억하면 됩니다.

- **반환 타입이 `String`이면 할당(allocation)입니다.** `&self`만 받으므로 원본은 바뀌지 않고, 결과를 담을 힙 메모리를 새로 받습니다.
- **반환 타입에 `&str`이나 `'_`가 있으면 원본을 빌립니다.** `fn trim(&self) -> &str`는 라이프타임 생략 규칙(Part 0 ch10)에 따라 "돌려준 `&str`은 `self`를 빌린다"는 뜻입니다. 복사가 없어 빠르지만, 결과를 쓰는 동안 원본이 살아 있어야 합니다(4.2절의 E0716).

`&mut self` 메서드는 `String`에만 있습니다. `&str`은 빌린 값이라 길이를 바꿀 수 없기 때문입니다. 예외로 `make_ascii_uppercase`처럼 바이트 수가 변하지 않는 수정은 `&mut str`에도 있습니다.

### 2.2 검색 패턴

`contains`, `find`, `split`, `trim_matches`, `replace` 등은 찾을 대상을 제네릭 매개변수 `P: Pattern`으로 받습니다. 이 챕터에서는 이것을 `match`의 패턴과 구분해 **검색 패턴**(pattern)이라고 부릅니다. `Pattern` 트레이트 자체는 아직 불안정(nightly 전용)이라 직접 구현할 수는 없지만, 표준 라이브러리가 구현해 둔 타입은 stable에서 그대로 쓸 수 있습니다.

| 넣는 것 | 일치하는 것 | 예 (`line = "2025-03-14 ERROR [db] ..."`) |
| --- | --- | --- |
| `char` | 그 글자 | `line.find('[')` → `Some(17)` |
| `&str`, `&String` | 부분 문자열 전체 | `line.find("db")` → `Some(18)` |
| `[char; N]`, `&[char]` | 배열 안의 글자 중 아무거나 | `line.find(['[', ':'])` → `Some(17)` |
| `FnMut(char) -> bool` 클로저 | 클로저가 `true`인 글자 | `line.find(\|c: char\| c.is_ascii_uppercase())` → `Some(11)` |
| `char::is_whitespace` 같은 함수 | 함수가 `true`인 글자 | `line.find(char::is_whitespace)` → `Some(10)` |

클로저는 매개변수 타입을 `c: char`로 적어야 합니다. 여러 타입이 `Pattern`을 구현하므로 `|c| ...`만으로는 컴파일러가 `c`의 타입을 정하지 못합니다(4.6절). `char`의 분류 메서드(`is_ascii_digit`, `is_alphanumeric` 등) 자체는 [ch1](../ch1-std-overview/README.md)에서 다뤘습니다.

### 2.3 찾기와 나누기

찾기와 나누기는 모두 원본을 빌립니다. 찾기 결과의 위치는 **바이트** 위치이고, 언제나 글자 경계이므로 그대로 잘라도 패닉하지 않습니다.

```rust
if let Some(i) = line.find(':') {
    let before = &line[..i];     // find가 준 위치는 글자 경계
    let after = &line[i + 1..];  // ':'은 1바이트. 일반적으로는 i + c.len_utf8()
}
```

| 하고 싶은 일 | 메서드 | 결과 |
| --- | --- | --- |
| 있는가 | `contains`, `starts_with`, `ends_with` | `bool` |
| 처음·마지막 위치 | `find`, `rfind` | `Option<usize>` |
| 모두 찾기 | `matches`, `match_indices` | `&str`, `(usize, &str)`을 내주는 반복자 |
| 구분자마다 나누기 | `split`, `rsplit` | 빈 조각도 그대로 나옴 |
| 끝의 구분자는 무시 | `split_terminator` | `"a;b;"` → `["a", "b"]` |
| 최대 n조각 | `splitn`, `rsplitn` | 마지막 조각에 나머지 전부 |
| 처음(마지막) 구분자에서 둘로 | `split_once`, `rsplit_once` | `Option<(&str, &str)>` |
| 공백 문자로 나누기 | `split_whitespace` | 연속 공백은 하나로, 빈 조각 없음 |
| 줄 단위 | `lines` | `\n`, `\r\n` 모두 처리, 줄 끝 문자는 뗌 |

`split(' ')`과 `split_whitespace()`는 다릅니다. 앞의 것은 스페이스 하나하나가 구분자라서 빈 조각이 생기고 탭과 줄바꿈은 자르지 못합니다. 사람이 입력한 단어를 나눌 때는 거의 항상 `split_whitespace`가 맞습니다. 반복자는 게으르므로 `collect`, `count` 같은 소비 메서드를 붙이거나 `for`로 돌려야 실제로 나눕니다(Part 0 ch13).

### 2.4 다듬기

| 메서드 | 하는 일 | 예 |
| --- | --- | --- |
| `trim`, `trim_start`, `trim_end` | 앞뒤의 공백 문자(유니코드 기준)를 걷어 냄 | `" \t 안녕 \n"` → `"안녕"` |
| `trim_matches(p)`, `trim_start_matches(p)`, `trim_end_matches(p)` | 검색 패턴과 일치하는 것을 **반복해서** 걷어 냄 | `"000120".trim_start_matches('0')` → `"120"` |
| `strip_prefix(p)`, `strip_suffix(p)` | **정확히 한 번** 떼고, 없으면 `None` | `"v1.2.3".strip_prefix('v')` → `Some("1.2.3")` |

`trim`은 유니코드의 공백 문자를 모두 걷어 내므로 한글 문서에 섞인 전각 공백(U+3000)도 지웁니다. ASCII 공백만 지우는 `trim_ascii`는 전각 공백을 남깁니다. "떼었는지"가 중요하면 `strip_prefix`를, 몇 개든 걷어 내기만 하면 되면 `trim_start_matches`를 씁니다. 접두사가 있으면 떼고 없으면 그대로 쓰는 모양은 `s.strip_prefix('#').unwrap_or(s)`입니다.

### 2.5 바꾸기와 잇기: 새 `String`을 만드는 메서드

| 메서드 | 하는 일 |
| --- | --- |
| `replace(p, to)`, `replacen(p, to, n)` | 검색 패턴과 일치하는 곳을 `to`로 바꾼 사본 (`replacen`은 앞에서 n번만) |
| `to_uppercase`, `to_lowercase` | 유니코드 규칙의 대소문자 변환 |
| `to_ascii_uppercase`, `to_ascii_lowercase` | ASCII 글자만 변환 (제자리 버전: `make_ascii_uppercase` 등) |
| `repeat(n)` | n번 반복 |
| `[s1, s2].concat()`, `[s1, s2].join(sep)` | 슬라이스(배열·벡터)의 문자열을 이어 붙임 |
| `chars().rev().collect::<String>()` | `char`(또는 `&str`)를 내주는 반복자를 모아 `String` 하나로 |
| `to_string()`, `to_owned()`, `String::from` | `&str`(또는 `char`)의 소유한 사본 |

대소문자 변환이 새 `String`을 돌려주는 데는 이유가 있습니다. 유니코드에서는 변환하면서 글자 수와 바이트 수가 달라질 수 있습니다. `"Straße"`(6글자)를 대문자로 바꾸면 `"STRASSE"`(7글자)가 되고, 합자 `"ﬁ"`(3바이트)는 `"FI"`(2바이트)가 됩니다. 그리스어 `Σ`는 단어 끝에서 `ς`가 되는 것처럼 문맥도 봅니다. 이런 변환은 원래 자리에서 할 수 없으므로 새 버퍼에 결과를 만듭니다. ASCII만 바꾸는 변환은 바이트 수가 그대로라서 제자리 버전(`make_ascii_*`)이 있습니다.

`concat`과 `join`은 `str`이 아니라 슬라이스의 메서드라서 `[&str]`, `Vec<String>` 어디에나 씁니다. 대소문자를 무시하고 찾고 싶을 때 `to_lowercase()`를 양쪽에 부르는 방법도 할당이 생기므로, 짧은 ASCII 문자열끼리 비교한다면 할당이 없는 `eq_ignore_ascii_case`가 낫습니다.

### 2.6 `String` 제자리 수정

| 메서드 | 하는 일 | 위치가 글자 경계가 아니면 |
| --- | --- | --- |
| `insert(i, c)`, `insert_str(i, s)` | `i`번 바이트에 넣음 | 패닉 |
| `pop()` | 마지막 글자를 꺼냄 → `Option<char>` | 해당 없음 |
| `remove(i)` | `i`번 바이트에서 시작하는 글자를 꺼냄 → `char` | 패닉 |
| `truncate(n)` | 앞의 n바이트만 남김 | 패닉 |
| `retain(f)` | 클로저가 `true`인 글자만 남김 | 해당 없음 |
| `drain(range)` | 범위를 지우고, 지운 글자를 내주는 반복자를 돌려줌 | 패닉 |
| `replace_range(range, s)` | 범위를 `s`로 바꿈 | 패닉 |
| `split_off(i)` | `i`번 바이트부터 뒤를 새 `String`으로 떼어 돌려줌 | 패닉 |
| `clear()` | 내용을 모두 지움 | 해당 없음 |

위치를 받는 메서드는 모두 **바이트 위치**를 받고, 글자 경계가 아니면 패닉합니다. 그래서 위치를 계산해서 넘길 때는 `find`, `char_indices`가 준 위치를 쓰거나 `is_char_boundary`로 먼저 확인합니다. `truncate`와 `clear`는 길이만 줄이고 할당받은 용량은 그대로 둡니다. 같은 `String`을 반복해서 비우고 다시 채우면 재할당이 줄어듭니다.

### 2.7 글자 경계와 바이트

Part 0 ch8에서 `&s[a..b]`가 글자 경계가 아니면 패닉하고, `get`은 `None`을 돌려준다는 것을 봤습니다. 위치를 다루는 도구를 정리하면 다음과 같습니다.

| 하고 싶은 일 | 방법 (`s = "가나다abc"`) | 결과 |
| --- | --- | --- |
| 경계인지 확인 | `s.is_char_boundary(4)` | `false` |
| 경계가 아니면 `None` | `s.get(0..4)`, `s.split_at_checked(7)` | `None`, `None` |
| 가장 가까운 경계로 내리기·올리기 | `s.floor_char_boundary(7)`, `s.ceil_char_boundary(7)` | `6`, `9` |
| 글자와 그 바이트 위치 | `s.char_indices()` | `(0, '가')`, `(3, '나')`, ... |
| 앞에서 n글자만 | `s.char_indices().nth(n)`의 위치로 자르기 | 예제 05의 `take_chars` |

`floor_char_boundary`와 `ceil_char_boundary`는 Rust 1.91에서 안정화되었습니다. "최대 N바이트까지만 저장"처럼 바이트 제한을 지키면서 글자를 깨뜨리지 않아야 할 때 씁니다.

바이트와 문자열 사이의 변환은 "빌리는가, 넘기는가"로 나뉩니다.

| 방향 | 빌림 (복사 없음) | 소유권을 넘김 (복사 없음) |
| --- | --- | --- |
| 문자열 → 바이트 | `s.as_bytes()` → `&[u8]` | `string.into_bytes()` → `Vec<u8>` |
| 바이트 → 문자열 (검사) | `str::from_utf8(&bytes)` → `Result<&str, Utf8Error>` | `String::from_utf8(vec)` → `Result<String, FromUtf8Error>` |
| 바이트 → 문자열 (대체) | `String::from_utf8_lossy(&bytes)` → `Cow<str>` | |

`str`과 `String`은 언제나 올바른 UTF-8이라는 것을 타입이 보장합니다. 그래서 바이트를 문자열로 바꿀 때는 반드시 검사를 거치고, 실패하면 `Err`를 돌려줍니다. `Utf8Error::valid_up_to()`는 앞에서부터 몇 바이트까지가 올바른지 알려 줍니다. 잘못된 부분을 대체 문자(replacement character) `�`(U+FFFD)로 바꿔서라도 읽고 싶으면 `from_utf8_lossy`를 씁니다. 반환 타입 `Cow<str>`는 "빌린 `&str` 또는 소유한 `String`"을 담는 타입으로, 바꿀 것이 없으면 원본을 빌리기만 하고 바꿔야 할 때만 새 `String`을 만듭니다. `{}`로 출력하거나 `&str` 메서드를 그대로 쓸 수 있고, `to_string()`으로 `String`을 얻습니다(`Cow` 자체는 [ch6](../ch6-core-traits/README.md)에서 다룹니다).

EUC-KR(CP949)로 저장된 한글 파일을 UTF-8로 읽으면 깨지는 이유도 여기에 있습니다. 표준 라이브러리는 UTF-8만 다루므로 다른 인코딩은 crates.io의 변환 크레이트로 바꿔야 합니다. 마찬가지로 유니코드 정규화(완성형·조합형 맞추기)와 자소 클러스터 단위 나누기(Part 0 ch8)도 표준 라이브러리에 없습니다.

### 2.8 서식 문자열 문법 전체

Part 0 ch1에서 너비·정렬·정밀도·진법을 맛봤습니다. 자리표시자의 전체 모양은 다음과 같고, 쓰는 항목만 이 순서대로 적습니다.

```text
{인자:[[채움]정렬][부호][#][0][너비][.정밀도][형식]}
```

**인자 고르기**

| 쓰는 법 | 뜻 | 예 |
| --- | --- | --- |
| `{}` | 다음 위치 인자 | `format!("{}는 {}살", name, age)` |
| `{0}`, `{1}` | 번호로 (0부터) | `format!("{1}는 {0}살", age, name)` |
| `{n}` + `n = 값` | 이름 인자 (위치 인자 뒤에 둠) | `format!("{n}는", n = name)` |
| `{name}` | 같은 이름의 변수를 캡처 (식은 안 됨) | `format!("{name}는 {age}살")` |

인자는 모두 한 번 이상 쓰여야 하고, 같은 인자를 여러 번 써도 됩니다. 번호나 `$`로 가리킨 자리표시자는 "다음 인자" 카운터를 움직이지 않습니다. 그래서 `format!("{1} {} {0} {}", 'A', 'B')`는 `B A A B`입니다. 섞어 쓰면 헷갈리므로 이름을 쓰는 편이 안전합니다.

**서식 지정 항목**

| 항목 | 쓰는 법 | 예 | 결과 | 참고 |
| --- | --- | --- | --- | --- |
| 정렬 | `<` `^` `>` | `{:^6}` ← `"ab"` | `  ab  ` | 숫자는 오른쪽, 그 밖에는 왼쪽이 기본 |
| 채움 | 정렬 앞에 글자 하나 | `{:*^8}` ← `"ab"` | `***ab***` | 기본은 스페이스 |
| 너비 | 정수, `N$`, `이름$` | `{:>w$}` ← `"ab"`, `w = 5` | `   ab` | 최소 칸 수(글자 수). 가리키는 값은 `usize` |
| 부호 | `+` | `{:+}` ← `9` | `+9` | `-`는 문법에는 있지만 쓰이지 않음 |
| `#` | 대체 형식 | `{:#x}` ← `2026` | `0x7ea` | `{:#?}`는 여러 줄 디버그 형식 |
| `0` | 부호 인식 0 채움 | `{:06}` ← `-42` | `-00042` | 부호·접두사 뒤에 0. 정렬 지정보다 우선 |
| 정밀도 | `.N`, `.N$`, `.이름$`, `.*` | `{:.2}` ← π | `3.14` | 실수: 소수 자릿수, 문자열: 최대 글자 수, 정수: 무시 |
| 형식 | `?` `x` `X` `o` `b` `e` `E` `p` `x?` `X?` | `{:b}` ← `5` | `101` | 아래 서식 트레이트 표 |

몇 가지는 직접 보지 않으면 틀리기 쉽습니다(모두 예제 06에서 확인합니다).

- `{:0<5}`는 0 플래그가 아니라 "0을 채움 문자로 왼쪽 정렬"입니다. `7`이 `70000`으로 보입니다. 숫자 앞을 0으로 채우려면 `{:05}`를 씁니다.
- 실수의 정밀도는 정확히 가운데인 값을 짝수 쪽으로 반올림합니다. `{:.0}`로 `0.5`, `1.5`, `2.5`를 출력하면 `0`, `2`, `2`입니다.
- 문자열의 정밀도는 바이트가 아니라 글자(`char`) 수입니다. `{:.2}` ← `"안녕하세요"`는 `안녕`입니다. 너비도 글자 수로 세므로 한글이 섞이면 터미널에서 줄이 맞지 않습니다(Part 0 ch1).
- `.*`는 인자 두 개를 가져갑니다. 먼저 정밀도, 다음이 값입니다: `format!("{:.*}", 1, pi)` → `3.1`.

**서식 트레이트**(formatting trait): 맨 끝의 형식 문자는 "어떤 트레이트의 `fmt`를 부를지"를 고릅니다. `{:x}`는 "16진수로 바꿔 줘"가 아니라 "`LowerHex::fmt`를 불러 줘"라는 뜻이므로, 그 트레이트를 구현한 타입만 쓸 수 있습니다.

| 형식 | 트레이트 | 표준 라이브러리가 구현한 타입 | 예 |
| --- | --- | --- | --- |
| (없음) | `Display` | 숫자, `char`, `bool`, `str`, `String` 등 | `2026` |
| `?` | `Debug` | 거의 모든 표준 타입 | `"ab"`, `[1, 2]` |
| `x?`, `X?` | `Debug` (안의 정수를 16진수로) | `Debug`와 같음 | `[a, ff]` |
| `x`, `X` | `LowerHex`, `UpperHex` | 정수 | `7ea`, `7EA` |
| `o` | `Octal` | 정수 | `3752` |
| `b` | `Binary` | 정수 | `11111101010` |
| `e`, `E` | `LowerExp`, `UpperExp` | 정수와 실수 | `2.026e3`, `1.2E-4` |
| `p` | `Pointer` | 참조와 포인터 | 실행마다 다른 주소 |

음수를 `{:x}`, `{:b}`로 출력하면 부호 없이 2의 보수 비트가 그대로 나옵니다(`-1i8` → `ff`). 실수는 `LowerHex`가 없으므로 `{:x}`가 컴파일 에러입니다(4.3절). 중괄호 문자 자체는 `{{`, `}}`로 씁니다.

**디버그 형식의 정렬**: 문자열의 디버그 형식은 너비를 무시합니다. `{:>8?}` ← `"ab"`는 `"ab"`일 뿐입니다. 반대로 `derive(Debug)`한 타입은 너비를 필드에 그대로 넘겨서 `{:>8?}` ← `Some(1)`이 `Some(       1)`이 됩니다. 디버그 형식은 정렬을 위한 것이 아니므로, 정렬하려면 `format!("{x:?}")`로 문자열을 먼저 만든 뒤 그 문자열을 정렬합니다.

### 2.9 서식 결과를 보내는 곳: `format!`, `write!`, `format_args!`

서식 매크로는 문법이 모두 같고 결과를 보내는 곳만 다릅니다.

| 매크로 | 결과가 가는 곳 | 반환 |
| --- | --- | --- |
| `format!` | 새 `String` | `String` |
| `write!(dst, ..)`, `writeln!(dst, ..)` | `dst` (`fmt::Write`나 `io::Write`를 구현한 것) | `fmt::Result` 또는 `io::Result<()>` |
| `print!`, `println!`, `eprint!`, `eprintln!` | 표준 출력, 표준 에러 | `()` |
| `format_args!` | 아무 데도 쓰지 않음 | `fmt::Arguments` |

`format_args!`는 서식 문자열과 인자를 묶어 **할당 없이** `fmt::Arguments` 값을 만듭니다. 다른 매크로는 모두 내부에서 이것을 만든 뒤 각자의 목적지에 씁니다.

```text
                   format_args!("…", 인자들)  →  fmt::Arguments (할당 없음)
                                  │
     ┌─────────────────┬──────────┴──────────┬───────────────────────────┐
  format!         write!(s, …)         println!(…)              Display::fmt 안의 write!(f, …)
  → 새 String     → s 뒤에 덧붙임       → 표준 출력               → Formatter에 씀
```

이미 있는 `String` 뒤에 서식 결과를 붙일 때는 `s.push_str(&format!(..))`보다 `write!(s, ..)`가 낫습니다. 임시 `String`을 만들지 않기 때문입니다. `String`에 `write!`를 쓰려면 `fmt::Write` 트레이트를 가져와야 합니다(트레이트 메서드는 트레이트가 범위에 있어야 부를 수 있습니다, Part 0 ch10).

```rust
use std::fmt::Write; // String에 write!를 쓰려면 필요

let mut report = String::new();
writeln!(report, "{item:<4}{price:>6}원").unwrap(); // String에 쓰기는 실패하지 않음
```

`write!`는 `fmt::Result`를 돌려주는데, `String`에 쓰는 것은 실패하지 않으므로 `unwrap()`해도 됩니다. 파일이나 표준 출력에 쓰는 `io::Write`는 [ch9](../ch9-io-fs-path/README.md)에서 다룹니다. 두 `Write`를 한 파일에서 함께 가져오면 이름이 겹치므로 `use std::fmt::Write as _;`처럼 이름 없이 가져옵니다.

`fmt::Arguments`는 `Display`를 구현하므로 함수 매개변수로 받아 `{args}`로 출력할 수 있습니다. 로깅처럼 "서식은 지금 받고, 어디에 쓸지는 나중에 정하는" 코드에 씁니다. 다만 `fmt::Arguments`의 `Display`는 바깥의 너비를 무시합니다(`{:>12}`로 출력해도 정렬되지 않음). 이유는 바로 다음 절과 같습니다.

### 2.10 `Display` 직접 구현과 `Formatter`

Part 0 ch10에서 이렇게 구현했습니다.

```rust
impl fmt::Display for NaiveVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.0, self.1, self.2)
    }
}
```

이 타입을 `{:>10}`으로 출력하면 `v1.98.1`이 정렬되지 않고 그대로 나옵니다. `{:>10}`의 너비와 정렬은 `f`(`Formatter`) 안에 담겨 `fmt`로 전달되는데, 위 구현은 그 값을 한 번도 읽지 않습니다. `write!(f, "v{}.{}.{}", ..)`는 **새 서식 문자열**로 쓰기 때문에, 그 안의 `{}`에는 너비가 없습니다. 서식 지정을 지키는 것은 `fmt`를 구현한 쪽의 책임이고, 숫자나 `str` 같은 표준 타입은 그 일을 해 두었을 뿐입니다.

`Formatter`는 요청된 서식 지정을 읽는 메서드와, 그것을 적용해 쓰는 도우미를 제공합니다.

| 메서드 | 돌려주는 것 | 서식 문자열의 대응 부분 |
| --- | --- | --- |
| `width()` | `Option<usize>` | `{:10}`의 `10` |
| `precision()` | `Option<usize>` | `{:.3}`의 `3` |
| `fill()` | `char` | `{:*<10}`의 `*` |
| `align()` | `Option<fmt::Alignment>` | `<`, `^`, `>` |
| `alternate()` | `bool` | `#` |
| `sign_plus()` | `bool` | `+` |
| `sign_aware_zero_pad()` | `bool` | `0` |
| `pad(s)` | `fmt::Result` | 채움·정렬·너비·정밀도를 적용해 `s`를 씀 |
| `pad_integral(..)` | `fmt::Result` | 숫자처럼 부호·`0` 채움·접두사까지 적용해 씀 |
| `write_str(s)` | `fmt::Result` | 서식 없이 그대로 씀 |

(1.98.1 문서에는 `new`, `sign`, `options`, `with_options`도 보이지만 nightly 전용이고, `flags`는 1.24부터 쓰지 말라는 표시(deprecated)가 붙어 있습니다.)

고치는 방법은 세 가지입니다(예제 07).

1. **완성한 문자열을 `pad`로 쓰기**: 가장 간단합니다. 정밀도는 "최대 글자 수"로 적용되어 `{:.3}` → `v1.`처럼 잘린다는 점에 주의합니다. 서식 지정이 없을 때 임시 `String`을 만들지 않으려면 `width()`와 `precision()`이 모두 `None`인지 먼저 확인합니다.

   ```rust
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       if f.width().is_none() && f.precision().is_none() {
           return write!(f, "v{}.{}.{}", self.0, self.1, self.2);
       }
       f.pad(&format!("v{}.{}.{}", self.0, self.1, self.2))
   }
   ```

2. **안쪽 값에 `f`를 그대로 넘기기**: 감싼 값의 `fmt`를 같은 `f`로 부르면 너비·정밀도·부호가 모두 안쪽 값에 적용됩니다. `Meters(12.345)`를 `{:+.2}`로 출력하면 `+12.35 m`입니다. 다만 너비도 안쪽 숫자에만 적용되어 `{:>8.1}`이 `    12.3 m`(숫자 부분만 8칸)이 됩니다.

   ```rust
   fmt::Display::fmt(&self.0, f)?; // 서식 지정이 그대로 self.0에 적용됨
   f.write_str(" m")
   ```

3. **서식 지정을 직접 해석하기**: `alternate()`, `precision()` 등을 읽어 출력 모양을 바꿉니다. 예제 07의 `Rgb`는 `LowerHex`를 구현하면서 `#` 플래그가 있으면 CSS처럼 `#ff8000`을 출력합니다. 서식 트레이트를 직접 구현하면 그 형식 문자(`{:x}`)를 쓸 수 있게 되고, 구현하지 않은 트레이트(`{:X}`)는 여전히 컴파일 에러입니다.

`fmt`는 `Formatter`가 돌려준 에러를 `?`로 전달하는 것 말고는 에러를 만들어 내면 안 됩니다. 문서에 따르면 서식 지정 자체는 실패하지 않는 작업이고, `fmt::Result`는 결과를 쓰는 대상(파일 등)의 실패를 전달하기 위해서만 있습니다.

### 2.11 `Debug` 직접 구현

`Display`는 사용자에게 보여 줄 모양, `Debug`는 개발자가 값의 내부를 확인하기 위한 모양입니다. 보통 `derive(Debug)`로 충분하지만, 비밀번호처럼 출력되면 안 되는 필드가 있거나 내부 구조 대신 의미 있는 모양을 보여 주고 싶으면 직접 구현합니다. 이때 `write!` 대신 `Formatter`의 도우미를 쓰면 `{:#?}`의 여러 줄 출력과 들여쓰기가 저절로 처리됩니다.

| 도우미 | 만드는 모양 | 주요 메서드 |
| --- | --- | --- |
| `f.debug_struct("이름")` | `이름 { a: 1, b: 2 }` | `.field("a", &v)`, `.finish()`, `.finish_non_exhaustive()` (`..` 붙임) |
| `f.debug_tuple("이름")` | `이름(1, 2)` | `.field(&v)`, `.finish()` |
| `f.debug_list()` | `[1, 2]` | `.entry(&v)`, `.entries(iter)`, `.finish()` |
| `f.debug_set()` | `{1, 2}` | `.entry(&v)`, `.entries(iter)`, `.finish()` |
| `f.debug_map()` | `{"k": 1}` | `.entry(&k, &v)`, `.entries(iter)`, `.finish()` |

```rust
impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Account")
            .field("id", &self.id)
            .field("owner", &self.owner)
            .finish_non_exhaustive() // password는 출력하지 않음
    }
}
// {:?}  → Account { id: 7, owner: "페리스", .. }
```

### 2.12 `FromStr`과 `parse`

`str::parse`의 시그니처와 `FromStr` 트레이트는 다음과 같습니다.

```rust
pub fn parse<F: FromStr>(&self) -> Result<F, F::Err>

pub trait FromStr: Sized {
    type Err;
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}
```

`"42".parse::<i32>()`는 `i32::from_str("42")`를 부르는 것과 같습니다. `F`가 무엇인지 알아야 어느 `from_str`을 부를지 정할 수 있으므로 결과 타입을 알려 줘야 합니다. 방법은 변수의 타입 표기(`let a: i32 = ..`), 터보피시(`parse::<i64>()`), 결과를 받는 쪽의 타입(함수 매개변수, `collect`의 대상 등)입니다. 단서가 없으면 E0282입니다([Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md)의 4.3절).

`FromStr`에는 라이프타임 매개변수가 없으므로 입력 문자열을 빌리는 타입(`&str` 필드를 가진 구조체 등)은 만들 수 없고, 필요한 부분을 복사해 소유하는 타입만 만들 수 있습니다.

표준 타입의 규칙은 생각보다 엄격합니다(예제 08).

| 타입 | 받아들이는 것 | 거부하는 것 |
| --- | --- | --- |
| 정수 | `"42"`, `"+42"` | `" 42"`(공백), `"1_000"`, `""`, 범위를 넘는 값(`"300"` → `u8`) |
| 실수 | `"3.14"`, `"1e3"`, `".5"`, `"inf"`, `"NaN"` | `"3,14"` (쉼표 소수점) |
| `bool` | `"true"`, `"false"` | `"True"` |
| `char` | 글자가 정확히 하나 | `"가나"` |

`parse`는 공백을 걷어 내지 않으므로 사용자 입력은 `trim()`한 뒤에 파싱합니다. 표준 에러 타입(`ParseIntError` 등)은 `Display`를 구현하므로 `{e}`로 사람이 읽을 메시지를, `{e:?}`로 종류(`PosOverflow` 등)를 볼 수 있습니다.

직접 만든 타입에 구현할 때는 에러 타입을 정하고 `from_str`을 작성합니다. 2.3~2.4절의 나누기·다듬기 메서드와 [ch2](../ch2-option-result/README.md)의 `map_err`, `?`가 그대로 쓰입니다.

```rust
impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix('v').unwrap_or(s);
        let parts: Vec<&str> = s.split('.').collect();
        let [major, minor, patch] = parts[..] else {
            return Err(ParseVersionError::PartCount(parts.len()));
        };
        let num = |part: &str| {
            part.parse::<u32>()
                .map_err(|_| ParseVersionError::BadNumber(part.to_string()))
        };
        Ok(Version { major: num(major)?, minor: num(minor)?, patch: num(patch)? })
    }
}
```

`FromStr` 문서는 `Display` 출력이 기계가 다시 읽을 수 있는 모양이라면 `FromStr`도 같은 모양을 받아들이는 것이 좋다고 설명합니다. `Display`로 만든 문자열을 `parse`했을 때 원래 값이 나오면 **왕복(round-trip)** 된다고 합니다. 입력은 출력보다 너그러워도 됩니다. 예제의 `Version`은 `"v1.2.3"`도 받지만 출력은 `1.2.3`입니다. 에러 타입은 `derive(Debug)`한 열거형에 사용자에게 보여 줄 `Display`를 구현하면 충분합니다. `std::error::Error` 트레이트까지 구현하는 방법은 [ch8](../ch8-errors-panic/README.md)에서 다룹니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_search_patterns.rs`](examples/01_search_patterns.rs) | 찾기 메서드와 검색 패턴 다섯 가지 | 1 |
| [`examples/02_split_trim.rs`](examples/02_split_trim.rs) | 나누기·다듬기, 결과가 원본을 빌린다는 증거(`substr_range`) | 1 |
| [`examples/03_new_strings.rs`](examples/03_new_strings.rs) | 새 `String`을 만드는 바꾸기·잇기, `char`를 모아 `String` 만들기 | 1 |
| [`examples/04_string_mutate.rs`](examples/04_string_mutate.rs) | `String` 제자리 수정과 바이트 위치 | 1, 2 |
| [`examples/05_bytes_boundaries.rs`](examples/05_bytes_boundaries.rs) | 패닉 없이 자르기, 바이트 ↔ 문자열 변환 | 2 |
| [`examples/06_format_syntax.rs`](examples/06_format_syntax.rs) | 서식 문법 전체, `format!`·`write!`·`format_args!` | 3 |
| [`examples/07_custom_fmt.rs`](examples/07_custom_fmt.rs) | `Display`를 `Formatter`로 고치기, `LowerHex` 구현, `Debug` 도우미 | 4 |
| [`examples/08_parse_fromstr.rs`](examples/08_parse_fromstr.rs) | 표준 타입의 파싱 규칙, `FromStr` 구현과 왕복 | 5 |
| [`compile_fail/`](compile_fail) | 문자열·서식·파싱에서 자주 만나는 컴파일 에러 | 1, 3~5 |

예제 명령은 저장소 루트에서 실행한다고 보고 썼습니다.

### 01_search_patterns — 찾기와 검색 패턴

```console
$ cargo run -q -p p1-ch3-strings-fmt --example 01_search_patterns
== 1. 들어 있는가: bool ==
contains("ERROR")   = true
starts_with("2025") = true
ends_with("ms")     = false

== 2. 어디에 있는가: Option<usize> (바이트 위치) ==
find("실패") = Some(29)
find("WARN") = None
find('-')    = Some(4)
rfind('-')   = Some(7)
콜론 앞: "2025-03-14 ERROR [db] 연결 실패"
콜론 뒤: " timeout=30s"

== 3. 모두 찾기: matches, match_indices ==
'-'의 개수 = 2
'-'의 위치 = [(4, "-"), (7, "-")]

== 4. 검색 패턴 자리에 올 수 있는 것 ==
char        '['   → Some(17)
&str        "db"  → Some(18)
&String     "timeout" → Some(37)
[char; 2]   ['[', ':'] → Some(17)
클로저      대문자 → Some(11)
함수        char::is_whitespace → Some(10)
숫자만 모으기 = 2025031430
한글 음절 개수 = 4

== 5. 대소문자 ==
contains("error") = false
to_lowercase().contains("error") = true
"ERROR".eq_ignore_ascii_case("error") = true
```

볼 점:

- `find("실패")`가 `29`인 이유: "실패" 앞에는 글자가 25개 있지만 그중 '연', '결'이 3바이트씩이라 바이트 위치는 29입니다.
- `matches`에 클로저를 넣고 `collect`하면 조건에 맞는 글자만 모은 `String`이 됩니다. 범위 `'가'..='힣'`의 `contains`로 한글 음절만 셉니다.

### 02_split_trim — 나누기와 다듬기

```console
$ cargo run -q -p p1-ch3-strings-fmt --example 02_split_trim
== 1. split 계열 ==
split(',')            = ["사과", "배", "", "포도"]
split('/')            = ["", "usr", "local", "bin"]
rsplit('/')           = ["bin", "local", "usr", ""]
split(';')            = ["a=1", "b=2", ""]
split_terminator(';') = ["a=1", "b=2"]
splitn(2, '=')        = ["url", "https://a.b/?x=1"]
split([',', ';'])     = ["하나", " 둘", "셋"]

== 2. split_once, rsplit_once ==
Some(("url", "https://a.b/?x=1"))
Some(("archive.tar", "gz"))
None

== 3. split_whitespace, lines ==
split(' ')         = ["", "", "하나", "", "", "둘\t셋\n넷", "", ""]
split_whitespace() = ["하나", "둘", "셋", "넷"]
lines()            = ["첫 줄", "둘째 줄", "", "넷째 줄"]

== 4. trim 계열 ==
trim()       = "안녕 하세요"
trim_start() = "안녕 하세요 \n"
trim_end()   = " \t 안녕 하세요"
"중요"
"120"
"report_"

== 5. strip_prefix, strip_suffix ==
Some("1.2.3")
None
Some("v1")
"1"
Some("main")
"rust"

== 6. 조각은 원본을 가리킵니다 ==
key = "name", value = "페리스"
key는 line의 Some(0..4)
value는 line의 Some(7..16)
"name" 리터럴은 None
```

볼 점:

- 6번 구역의 `substr_range`(Rust 1.98에서 안정화)는 문자열을 다시 검색하지 않고 두 `&str`의 메모리 위치를 비교합니다. `key`와 `value`가 `Some(범위)`라는 것은 `split_once`와 `trim`이 복사 없이 `line`의 바이트를 가리킨다는 증거입니다. 내용이 같아도 다른 곳에 있는 리터럴 `"name"`은 `None`입니다.
- `lines()`는 `"첫 줄\r\n"`의 `\r`까지 떼고, 마지막 `\n` 뒤에 빈 줄을 만들지 않습니다.

### 03_new_strings — 새 `String`을 만드는 메서드

```console
$ cargo run -q -p p1-ch3-strings-fmt --example 03_new_strings
== 1. replace, replacen ==
phone = 010-1234-5678, digits = 01012345678
replacen(숫자, "*", 3) = ***-1234-5678
"a b  c"

== 2. 대소문자 ==
HELLO, 러스트!
Straße (6글자) → STRASSE (7글자)
ﬁ (3바이트) → FI (2바이트)
σας
STRAßE
ABC-ß

== 3. repeat ==
============
하하하하하하

== 4. concat, join ==
concat() = 20261118
join("-") = 2026-11-18
join(", ") = 페리스, 코랄, 러스티
names는 그대로: ["페리스", "코랄", "러스티"]

== 5. char를 모아 String 만들기 ==
뒤집기: 다나가
머리글자: RIF
漢 漢 (len_utf8 = 3)

== 6. String ↔ &str ==
리터럴 / 리터럴 / 리터럴
```

볼 점:

- `"a  b   c".replace("  ", " ")`가 `"a b  c"`인 이유: 앞에서부터 겹치지 않게 한 번만 훑으며 바꾸고, 바꾼 결과를 다시 검사하지 않습니다. 연속 공백을 하나로 줄이려면 `split_whitespace`로 나눈 뒤 `join(" ")`합니다(연습 1).
- `to_ascii_uppercase`는 `ß`를 건드리지 않습니다. ASCII 글자만 바꾸기 때문입니다.

### 04_string_mutate — `String` 제자리 수정

```console
$ cargo run -q -p p1-ch3-strings-fmt --example 04_string_mutate
== 1. insert, insert_str ==
[안녕하세요]
[안녕, 러스트하세요]
10번 바이트가 글자 경계인가? false

== 2. pop, remove ==
pop() = Some(']'), remove(0) = '[', s = 안녕, 러스트하세요

== 3. truncate ==
s = "안녕", len = 6, 용량 그대로? true

== 4. retain ==
ab12cd34

== 5. drain ==
꺼낸 것 = "[경고]", 남은 것 = " 디스크 부족"

== 6. replace_range ==
2026-03-14

== 7. split_off ==
성 = 홍, 이름 = 길동

== 8. clear ==
len = 0, 비었나? true, 용량 ≥ 32? true
```

볼 점:

- `pop`은 빈 문자열에서 실패할 수 있어 `Option<char>`를, `remove`는 잘못된 위치에서 패닉하므로 그냥 `char`를 돌려줍니다. 시그니처가 실패 방식을 알려 줍니다.
- 5번 구역의 `drain(..end)`에 넘긴 `end`는 `find(']')`가 준 위치에 1을 더한 것입니다. `]`가 1바이트이므로 글자 경계입니다.

### 05_bytes_boundaries — 글자 경계와 바이트 변환

```console
$ cargo run -q -p p1-ch3-strings-fmt --example 05_bytes_boundaries
== 1. get, is_char_boundary, split_at_checked ==
len() = 12 (바이트)
글자 경계인 위치: [0, 3, 6, 9, 10, 11, 12]
get(0..3) = Some("가")
get(0..4) = None
get(9..)  = Some("abc")
split_at_checked(6) = Some(("가나", "다abc"))
split_at_checked(7) = None
floor_char_boundary(7) = 6 → "가나"
ceil_char_boundary(7)  = 9

== 2. char_indices로 글자 단위 자르기 ==
[(0, '가'), (3, 'a'), (4, '나')]
"러스트로 배"
"짧음"

== 3. as_bytes, into_bytes ==
as_bytes() = [82, 117, 115, 116]
into_bytes() = [82, 117, 115, 116]

== 4. from_utf8: 검사하고 변환 ==
str::from_utf8    = Ok("한글")
String::from_utf8 = Ok("한글")
실패: incomplete utf-8 byte sequence from index 3
올바른 부분: Ok("caf")

== 5. from_utf8_lossy ==
caf�
�ѱ� (3글자)
```

볼 점:

- 4번 구역의 Latin-1 바이트 `[b'c', b'a', b'f', 0xE9]`를 리터럴로 적으면 컴파일러가 "항상 `Err`가 된다"는 `invalid_from_utf8` 경고를 냅니다. 예제는 일부러 쓴 코드이므로 `#[expect(invalid_from_utf8, reason = "...")]`를 붙였습니다.
- 5번 구역의 마지막 줄은 EUC-KR로 저장된 "한글"(4바이트)을 UTF-8로 읽은 결과입니다. 가운데 두 바이트가 우연히 올바른 UTF-8이라 엉뚱한 글자 `ѱ`가 나왔습니다. 인코딩이 맞지 않으면 "에러"가 아니라 "그럴듯한 쓰레기"가 나올 수도 있다는 뜻입니다.

### 06_format_syntax — 서식 문법 전체

```console
$ cargo run -q -p p1-ch3-strings-fmt --example 06_format_syntax
== 1. 인자 고르기 ==
페리스는 9살
페리스는 9살
페리스는 9살
페리스는 9살
B A A B

== 2. 채움, 정렬, 너비 ==
[     9] [ab    ]
[ab    ] [  ab  ] [    ab]
[***ab***] [----ab]
[     ab] [  ab] [   ab]

== 3. 부호, #, 0 ==
[+9] [-42] [+2.0]
[-00042] [0x00ff] [0b00000101]
[00007] [00007] [70000]

== 4. 정밀도 ==
[3.14] [   3.142] [3.1     ]
[0] [2] [2]
[안녕] [    안녕]
[9]
[3.142] [3.1] [  3.1416]

== 5. 서식 트레이트 ==
2026 7ea 7EA 3752 11111101010
0x7ea 0x7EA 0o3752 0b11111101010
ff 11111110
1.2345e3 1.2E-4 2.026e3
[10, 255] [a, ff] [A, FF]
0x16b078b0c

== 6. 디버그 형식 ==
탭	"인용"
"탭\t\"인용\""
["ab"] [    "ab"]
(
    1,
    "가",
)

== 7. 중괄호 ==
{} {9}

== 8. format!, write!, format_args! ==
페리스 |  9
커피    4500원
빵     3200원
합계   7700원
[로그] 페리스의 나이는 9살
[9살]
```

볼 점:

- 5번 구역의 마지막 줄(`{r:p}`의 주소)은 실행할 때마다 다릅니다.
- 마지막 줄 `[9살]`은 `{:>12}`로 출력했는데도 정렬되지 않았습니다. Clippy도 이것을 알고 `unused_format_specs` 경고를 내므로, 예제는 `let` 문에 `#[expect]`를 붙였습니다(`#[expect]`는 `println!` 호출 자체에는 붙일 수 없습니다).
- 8번 구역의 표는 한글이 섞여 터미널에서 줄이 조금 어긋나 보일 수 있습니다. 너비는 글자 수로 세기 때문입니다(Part 0 ch1).

### 07_custom_fmt — `Display`와 `Debug` 직접 구현

```console
$ cargo run -q -p p1-ch3-strings-fmt --example 07_custom_fmt
== 1. write!만 쓰면 서식 지정이 무시됩니다 ==
[v1.98.1] [v1.98.1]
[   v1.98.1] [--v1.98.1---]
[v1.98.1] [v1.]

== 2. 표로 비교 ==
rustc   |v1.98.1|
clippy  |v0.1.98|
rustc   |   v1.98.1|
clippy  |   v0.1.98|

== 3. 안쪽 값에 Formatter 넘기기 ==
[12.345 m] [12.3 m] [+12.35 m]
[    12.3 m]

== 4. LowerHex 구현 ==
ff8000 #ff8000 [    ff8000]

== 5. Debug 도우미 ==
Account { id: 7, owner: "페리스", .. }
Account {
    id: 7,
    owner: "페리스",
    ..
}
비밀번호 확인: true
["산책", "Coffee Break"]
```

볼 점:

- 1번 구역의 첫 줄(`NaiveVersion`)은 `{:>10}`과 `{:-^12}`를 모두 무시했고, 둘째 줄(`Version`, `pad` 사용)은 둘 다 따랐습니다. 2번 구역의 표에서 차이가 분명히 보입니다.
- `Account`의 `{:#?}` 출력은 따로 구현하지 않았습니다. `debug_struct`가 `#` 플래그를 보고 알아서 여러 줄로 씁니다.

### 08_parse_fromstr — `FromStr`과 파싱

```console
$ cargo run -q -p p1-ch3-strings-fmt --example 08_parse_fromstr
== 1. parse와 FromStr ==
42 42 42

== 2. 표준 타입의 파싱 규칙 ==
i32  "+42"    → Ok(42)
i32  " 42"    → Err(ParseIntError { kind: InvalidDigit })
i32  "1_000"  → Err(ParseIntError { kind: InvalidDigit })
i32  ""       → Err(ParseIntError { kind: Empty })
u8   "300"    → Err(ParseIntError { kind: PosOverflow })
f64  "1e3"    → Ok(1000.0)
f64  "3,14"   → Err(ParseFloatError { kind: Invalid })
bool "True"   → Err(ParseBoolError)
char "가"      → Ok('가')
char "가나"     → Err(ParseCharError { kind: TooManyChars })
에러 메시지: number too large to fit in target type
trim 뒤 → Ok(42)

== 3. 직접 만든 타입 파싱 ==
"1.98.1"  → Ok: 1.98.1
"v2.0.10" → Ok: 2.0.10
"1.2"     → Err: 점으로 나눈 부분이 3개여야 하는데 2개입니다 (PartCount(2))
"1.x.3"   → Err: 숫자가 아닌 부분이 있습니다: "x" (BadNumber("x"))

== 4. 문자열 비교와 값 비교 ==
문자열: 1.10.0 > 1.9.0 ? false
Version: 1.10.0 > 1.9.0 ? true

== 5. Display ↔ FromStr 왕복 ==
Version { major: 1, minor: 10, patch: 0 } → "1.10.0" → Ok(Version { major: 1, minor: 10, patch: 0 })
같은 값인가? true

== 6. 목록 파싱 ==
Ok([Version { major: 1, minor: 2, patch: 3 }, Version { major: 1, minor: 10, patch: 0 }, Version { major: 0, minor: 9, patch: 12 }])
가장 높은 버전: Some("1.10.0")
Err(BadNumber("x"))
```

볼 점:

- 2번 구역의 `show::<T>` 함수는 `T: FromStr + Debug`, `T::Err: Debug`인 아무 타입이나 받는 제네릭 함수입니다. `parse`가 "`FromStr`을 구현한 타입이면 무엇이든" 만든다는 것을 그대로 보여 줍니다. `char` 두 줄이 어긋나 보이는 것은 한글 너비 문제입니다.
- 4번 구역이 파싱하는 이유입니다. 문자열은 글자 순서로 비교하므로 `"1.10.0" < "1.9.0"`이지만, `Version`은 derive한 `PartialOrd`로 숫자를 필드 순서대로 비교합니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch3-strings-fmt
rustc --edition 2024 compile_fail/e0369_add_two_str.rs --out-dir /tmp/rust-p1-ch3
```

### 4.1 `&str` 두 개를 `+`로 이음 — E0369

파일: [`compile_fail/e0369_add_two_str.rs`](compile_fail/e0369_add_two_str.rs)

```text
error[E0369]: cannot add `&str` to `&str`
  --> compile_fail/e0369_add_two_str.rs:15:28
   |
15 |     let message = greeting + name;
   |                   -------- ^ ---- &str
   |                   |        |
   |                   |        `+` cannot be used to concatenate two `&str` strings
   |                   &str
   |
   = note: string concatenation requires an owned `String` on the left
help: create an owned `String` from a string reference
   |
15 |     let message = greeting.to_owned() + name;
   |                           +++++++++++
```

- **읽는 법**: `note:`가 핵심입니다. 문자열을 `+`로 이으려면 왼쪽이 소유한 `String`이어야 합니다.
- **원인**: `+`는 `String`에만 정의되어 있고, 왼쪽 `String`의 버퍼 뒤에 오른쪽 `&str`을 덧붙입니다(Part 0 ch8). `&str` 두 개는 둘 다 빌린 값이라 결과를 담을 버퍼가 없습니다. Rust는 새 버퍼를 몰래 할당하지 않고, 할당이 일어나는 곳을 코드에 드러내게 합니다.
- **해결**: `help:`대로 왼쪽을 `to_owned()`로 `String`으로 만들거나, `format!("{greeting}{name}")`, `[greeting, name].concat()`을 씁니다.

### 4.2 임시 `String`에서 빌린 `&str`을 계속 씀 — E0716

파일: [`compile_fail/e0716_trim_temporary.rs`](compile_fail/e0716_trim_temporary.rs)

```text
error[E0716]: temporary value dropped while borrowed
  --> compile_fail/e0716_trim_temporary.rs:16:17
   |
16 |     let label = format!("  {} 번 ", number).trim();
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^       - temporary value is freed at the end of this statement
   |                 |
   |                 creates a temporary value which is freed while still in use
17 |     println!("[{label}]");
   |                 ----- borrow later used here
   |
   = note: consider using a `let` binding to create a longer lived value
```

- **읽는 법**: `format!`이 만든 임시값이 문장 끝(`;`)에서 해제되는데, 그 임시값을 빌린 `label`이 다음 줄에서 쓰인다는 뜻입니다.
- **원인**: `format!`은 새 `String`을 만들지만 변수에 담지 않았으므로 이 문장이 끝나면 버려집니다. `trim()`은 `fn trim(&self) -> &str`이라서 그 `String`을 빌린 조각을 돌려줍니다(2.1절). 소유자가 사라진 뒤에 빌린 조각을 쓸 수는 없습니다.
- **해결**: `note:`대로 소유자를 먼저 `let`으로 묶습니다: `let raw = format!(..); let label = raw.trim();`. 빌린 조각만 오래 필요하다면 `.trim().to_string()`으로 소유한 사본을 만듭니다.

### 4.3 실수를 `{:x}`로 출력 — E0277

파일: [`compile_fail/e0277_float_lower_hex.rs`](compile_fail/e0277_float_lower_hex.rs)

```text
error[E0277]: the trait bound `{float}: LowerHex` is not satisfied
  --> compile_fail/e0277_float_lower_hex.rs:14:15
   |
14 |     println!("{ratio:x}");
   |               ^^^^^^^^^ the trait `LowerHex` is not implemented for `{float}`
   |
   = help: the following other types implement trait `LowerHex`:
             i128
             i16
             i32
             i64
             i8
             isize
             u128
             u16
           and 4 others
```

- **읽는 법**: `{float}`은 "아직 `f32`인지 `f64`인지 정해지지 않은 실수"라는 뜻입니다. `help:` 목록이 `LowerHex`를 구현한 타입(정수들)을 보여 줍니다.
- **원인**: `{:x}`는 `LowerHex` 트레이트를 부르는 자리표시자입니다(2.8절). 표준 라이브러리는 실수에 `LowerHex`를 구현하지 않았습니다.
- **해결**: 실수의 비트 모양을 보고 싶다면 `ratio.to_bits()`로 같은 비트의 정수(`u64`)를 얻어 `{:x}`로 출력합니다. 단순히 숫자를 보고 싶었다면 `{}`나 `{:e}`를 씁니다.

### 4.4 `Display` 없이 `to_string()` — E0599

파일: [`compile_fail/e0599_to_string_without_display.rs`](compile_fail/e0599_to_string_without_display.rs)

```text
error[E0599]: `Point` doesn't implement `std::fmt::Display`
  --> compile_fail/e0599_to_string_without_display.rs:21:18
   |
14 | struct Point {
   | ------------ method `to_string` not found for this struct because it doesn't satisfy `Point: ToString` or `Point: std::fmt::Display`
...
21 |     let text = p.to_string();
   |                  ^^^^^^^^^ method cannot be called on `Point` due to unsatisfied trait bounds
   |
   = note: the following trait bounds were not satisfied:
           `Point: std::fmt::Display`
           which is required by `Point: ToString`
```

- **읽는 법**: `note:`의 두 줄이 연결 고리입니다. `Point: ToString`이 되려면 `Point: Display`가 필요한데 그것이 없습니다.
- **원인**: `to_string()`은 `impl<T: Display + ?Sized> ToString for T`라는 포괄 구현에서 생깁니다(Part 0 ch10). `derive(Debug)`만으로는 생기지 않습니다.
- **해결**: `impl fmt::Display for Point`를 작성합니다. 디버그 형식의 문자열이면 충분할 때는 `format!("{p:?}")`를 씁니다.

### 4.5 `FromStr`이 없는 타입으로 `parse` — E0277

파일: [`compile_fail/e0277_parse_without_fromstr.rs`](compile_fail/e0277_parse_without_fromstr.rs)

```text
error[E0277]: the trait bound `Color: FromStr` is not satisfied
  --> compile_fail/e0277_parse_without_fromstr.rs:20:23
   |
20 |     let color = "red".parse::<Color>();
   |                       ^^^^^ unsatisfied trait bound
   |
help: the trait `FromStr` is not implemented for `Color`
  --> compile_fail/e0277_parse_without_fromstr.rs:14:1
   |
14 | enum Color {
   | ^^^^^^^^^^
   = help: the following other types implement trait `FromStr`:
             ByteString
             CString
             IpAddr
             Ipv4Addr
             Ipv6Addr
             NonZero<i128>
             NonZero<i16>
             NonZero<i32>
           and 32 others
```

- **읽는 법**: `parse`의 바운드 `F: FromStr`을 `Color`가 만족하지 않는다는 뜻입니다. 두 번째 `help:`는 `FromStr`을 구현한 표준 타입 목록입니다.
- **원인**: `"red"`를 `Color::Red`로 바꾸는 규칙은 타입을 만든 사람만 정할 수 있습니다. 컴파일러는 배리언트 이름을 보고 짐작해 주지 않습니다.
- **해결**: `impl FromStr for Color`를 작성합니다. `match s { "red" => Ok(Color::Red), "green" => Ok(Color::Green), _ => Err(..) }`처럼 쓰면 됩니다(연습 3).

### 4.6 그 밖의 에러

`rustc`로 확인한 메시지입니다. 파일로 따로 두지는 않았습니다.

| 코드 | 에러 | 원인과 해결 |
| --- | --- | --- |
| `println!("{nme}")` (변수는 `name`) | ``error[E0425]: cannot find value `nme` in this scope`` | 캡처할 변수 이름의 오타. `help:`가 비슷한 이름을 제안합니다. |
| `println!("{a}", a = 1, b = 2)` | `error: named argument never used` | 쓰지 않은 이름 인자. 서식 문자열 자체를 검사하는 에러에는 E 코드가 붙지 않습니다. |
| `println!("{v.len()}")` | ``error: invalid format string: expected `}`, found `.` `` | 캡처는 변수 이름만 됩니다. 식은 `println!("{}", v.len())`처럼 인자로 넘깁니다. |
| `let w = 5i32; println!("{:w$}", 1)` | `error[E0308]: mismatched types` (``expected `&usize`, found `&i32` ``) | 너비·정밀도로 쓰는 인자는 `usize`여야 합니다. |
| `line.find(\|c\| c.is_ascii_uppercase())` | `error[E0282]: type annotations needed` | 검색 패턴 클로저의 매개변수 타입을 정할 수 없습니다. `\|c: char\|`로 적습니다. |
| `let n = s.parse().unwrap();` 뒤 타입 단서 없음 | `error[E0282]: type annotations needed` | [Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md) 4.3절과 같습니다. |

실행 중 패닉도 하나 기억해 둡니다. `&s[a..b]`, `insert`, `remove`, `truncate`, `drain`, `split_off`에 글자 경계가 아닌 바이트 위치를 주면 컴파일은 되지만 실행 중에 패닉합니다(Part 0 ch8 4.5절). 위치를 계산했다면 `get`, `is_char_boundary`, `char_indices`로 확인합니다(2.7절).

## 5. 연습 문제

### 연습 1. 공백 정리하기

`fn normalize_spaces(s: &str) -> String`을 작성하세요. 앞뒤 공백을 없애고, 단어 사이의 공백(스페이스, 탭, 줄바꿈, 전각 공백 등)이 몇 개든 스페이스 하나로 바꿉니다. `"  러스트는\t\t재미\u{3000}있다 \n"` → `"러스트는 재미 있다"`. 그다음 `replace`로는 왜 한 번에 되지 않는지 예제 03의 `"a b  c"`를 보고 설명해 보세요.

<details>
<summary>힌트</summary>

- `split_whitespace()`는 유니코드 공백 문자 기준으로 나누고 빈 조각을 버리므로 앞뒤 공백도 함께 사라집니다(전각 공백 U+3000도 공백 문자입니다).
- 나눈 조각을 `Vec<&str>`로 `collect`한 뒤 `join(" ")`합니다. 조각은 원본을 빌린 `&str`이라 복사되지 않고, 결과 문자열은 마지막 `join`에서 한 번 할당됩니다(조각을 담는 `Vec`의 할당은 따로 있습니다).
- `replace("  ", " ")`는 바꾼 결과를 다시 검사하지 않으므로 공백 세 개가 두 개로 남습니다.

</details>

### 연습 2. 표 출력과 `Display`의 너비

`struct Money(u64)`에 천 단위 쉼표를 넣는 `Display`를 구현하세요(`Money(1234567)` → `1,234,567원`). 그리고 `println!("[{:>14}]", Money(1234567))`가 `[    1,234,567원]`처럼 오른쪽 정렬되게 만드세요. `{}`로 출력할 때는 임시 `String`을 만들지 않는 버전도 시도해 보세요.

<details>
<summary>힌트</summary>

- 숫자를 `to_string()`으로 바꾼 뒤 `char_indices()`나 `chars()`로 돌면서, 오른쪽에서 센 자릿수가 3의 배수인 위치 앞에 `','`를 `push`합니다. 앞에서 셀 때 위치 `i`(0부터)에서 남은 자릿수는 `len - i`입니다.
- 쉼표를 넣은 결과를 `String`으로 만든 뒤 `f.pad(&text)`로 쓰면 너비·정렬·채움이 모두 적용됩니다(2.10절의 방법 1). `"원"`도 `text`에 포함해야 정렬 폭에 들어갑니다.
- `f.width().is_none()`이면 `write!`로 바로 쓰는 분기를 두면 서식 지정이 없을 때 할당을 피할 수 있습니다.

</details>

### 연습 3. 색 이름 파싱

4.5절의 `Color`에 `Red`, `Green`, `Blue`를 두고 `FromStr`과 `Display`를 구현하세요. 대소문자와 앞뒤 공백은 무시하고(`" RED "` → `Color::Red`), 모르는 이름이면 그 이름을 담은 에러를 돌려줍니다. 모든 배리언트에 대해 `c.to_string().parse::<Color>() == Ok(c)`인지 확인하세요.

<details>
<summary>힌트</summary>

- 에러 타입은 `#[derive(Debug, PartialEq)] struct UnknownColor(String);`으로 충분합니다. 사용자에게 보여 줄 메시지는 `Display`로 구현합니다.
- `from_str` 안에서 `s.trim().to_ascii_lowercase()`로 정규화한 `String`을 만들고, `match normalized.as_str() { "red" => ..., _ => Err(UnknownColor(s.trim().to_string())) }`로 나눕니다. `match`는 `String`이 아니라 `&str`과 문자열 리터럴을 비교하므로 `as_str()`이 필요합니다.
- `Display`는 `"red"`처럼 `from_str`이 받아들이는 모양으로 출력해야 왕복이 됩니다. 비교하려면 `Color`에 `PartialEq`, `Clone, Copy`(또는 비교 전에 복제)를 derive합니다.

</details>

### 연습 4. 로그 줄 해석

예제 01의 `"2025-03-14 ERROR [db] 연결 실패: timeout=30s"` 같은 줄을 `struct LogLine { date: String, level: Level, module: String, message: String }`으로 파싱하는 `FromStr`을 구현하세요. `Level`은 `Info`, `Warn`, `Error`이고 그 자체도 `FromStr`을 구현합니다. 형식이 맞지 않으면 어느 부분이 잘못되었는지 알려 주는 에러를 돌려주세요.

<details>
<summary>힌트</summary>

- `split_once(' ')`를 두 번 써서 날짜, 레벨, 나머지로 나눕니다. 나머지는 `strip_prefix('[')`로 대괄호를 떼고 `split_once("] ")`로 모듈과 메시지를 나눕니다.
- `Option`을 돌려주는 단계는 `.ok_or(LogError::MissingModule)?`처럼 `Result`로 바꿔 `?`로 잇습니다([ch2](../ch2-option-result/README.md)).
- `level_text.parse::<Level>()?`처럼 안쪽 타입의 `FromStr`을 그대로 쓰려면 두 에러 타입이 같아야 합니다. 같지 않다면 `.map_err(LogError::BadLevel)?`로 바꿉니다.
- `FromStr`은 입력을 빌리는 타입을 만들 수 없으므로(2.12절) 필드는 `&str`이 아니라 `String`입니다.
- 에러 배리언트의 필드를 `{:?}`로 출력하기만 하면 읽은 것으로 치지 않아 `dead_code` 경고가 납니다. 에러에 `Display`를 구현해 필드를 읽거나 `match`로 꺼내 쓰세요.

</details>

## 6. 참고 자료

- API 문서 (기준 문서)
  - [`std::string::String`](https://doc.rust-lang.org/std/string/struct.String.html): 제자리 수정 메서드(2.6절)와 `from_utf8`, `from_utf8_lossy`. 페이지 아래의 "Methods from `Deref<Target = str>`"에 `str` 메서드가 모두 다시 나옵니다.
  - [원시 타입 `str`](https://doc.rust-lang.org/std/primitive.str.html): 찾기·나누기·다듬기·바꾸기 메서드 전체. 각 메서드의 시그니처에서 반환 타입을 보며 2.1절의 표와 맞춰 보세요.
  - [`std::str`](https://doc.rust-lang.org/std/str/index.html): `from_utf8`, `Utf8Error`, 그리고 `split` 같은 메서드가 돌려주는 반복자 타입(`Split`, `Lines` 등)이 있는 모듈입니다.
  - [`std::str::FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html): "Input format and round-tripping" 절이 2.12절의 왕복 권고입니다. 예제는 `Point`를 `strip_prefix`, `split_once`, `map_err`로 파싱합니다.
  - [`std::char`](https://doc.rust-lang.org/std/char/index.html): `char` 관련 함수와 타입. `char`의 메서드 자체는 [ch1](../ch1-std-overview/README.md)에서 다뤘습니다.
  - [`std::fmt`](https://doc.rust-lang.org/std/fmt/index.html): 서식 문법 전체(Syntax 절의 문법), 서식 트레이트 대응표, `format_args!` 설명. 2.8~2.9절의 원본입니다.
  - [`std::fmt::Formatter`](https://doc.rust-lang.org/std/fmt/struct.Formatter.html): `pad`, `width`, `precision`, `alternate`, `debug_struct` 등의 시그니처와 예제(2.10~2.11절).
- Rust By Example
  - [Formatted print](https://doc.rust-lang.org/rust-by-example/hello/print.html): 위치·이름 인자, 진법, 너비와 채움. 끝의 활동 과제(π를 소수점 셋째 자리까지)는 2.8절 복습입니다.
  - [Formatting](https://doc.rust-lang.org/rust-by-example/hello/print/fmt.html): 같은 값을 `{}`, `{:X}`, `{:o}`로 다르게 보여 주는 것이 트레이트 덕분이라는 설명과 `City` 예제. 끝의 `Color`를 `RGB (128, 255, 90) 0x80FF5A`로 출력하는 과제는 `{:02X}`와 `Display` 구현 연습입니다.
  - [Strings](https://doc.rust-lang.org/rust-by-example/std/str.html): `trim_matches`가 할당 없이 원본을 빌린다는 것, `replace`가 새 메모리를 할당한다는 것을 주석으로 보여 줍니다(2.1절).
  - [To and from Strings](https://doc.rust-lang.org/rust-by-example/conversion/string.html): `Display`를 구현하면 `to_string`이 생긴다는 것과 `FromStr` 구현 예(`Circle`).
- Rustlings
  - [`09_strings`](https://github.com/rust-lang/rustlings/tree/main/exercises/09_strings): `strings1.rs`~`strings4.rs`. 특히 `strings4.rs`는 식마다 `&str`인지 `String`인지 고르는 문제라 2.1절의 표로 풀 수 있습니다.
  - [`23_conversions`](https://github.com/rust-lang/rustlings/tree/main/exercises/23_conversions): `conversions3.rs`가 `FromStr` 구현 연습입니다. 나머지(`as`, `From`, `TryFrom`, `AsRef`)는 [ch6](../ch6-core-traits/README.md)의 주제입니다.
