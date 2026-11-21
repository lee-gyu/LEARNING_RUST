# ch18. 패닉하지 않기

패닉은 Rust에서 복구할 수 없는 에러, 즉 버그를 알리는 수단입니다. 사용자 입력처럼 일어날 수 있는 실패는 `Result`로, 약속을 어긴 호출은 패닉으로 다룬다는 기본 기준은 [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.11절에서 배웠고, `catch_unwind`와 패닉 훅은 [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)에서 다뤘습니다. 이 챕터는 그 위에서 두 가지 판단을 다룹니다. 하나는 **공개 API를 설계하는 쪽의 판단**입니다. 라이브러리가 패닉하면 부르는 쪽에서 무엇을 빼앗는지, 패닉판을 꼭 주고 싶다면 표준 라이브러리처럼 실패를 돌려주는 판과 어떻게 짝지어 주는지 봅니다. 다른 하나는 **숨은 패닉을 찾아 없애는 방법**입니다. 인덱싱, 나눗셈, 뺄셈, 문자열 자르기, `RefCell`처럼 `panic!`이라는 글자가 보이지 않는 곳에서도 패닉이 나고, `catch_unwind`는 그것을 믿고 맡길 그물이 되지 못합니다.

- 원제: [Item 18: Don't panic](https://effective-rust.com/panic.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch18-dont-panic`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

앞의 부록 챕터가 이 챕터로 미룬 것도 여기서 받습니다(괄호 안은 이 챕터의 절). [ch3](../ch3-option-result-transforms/README.md)의 `unwrap`/`expect`(2.3절, 2.6절), [ch7](../ch7-builders/README.md)의 `build()`가 `Result`를 돌려주는 이유(2.2절), [ch9](../ch9-iterator-transforms/README.md)의 인덱싱 패닉과 [ch15](../ch15-borrow-checker/README.md)의 `RefCell` 패닉(2.4절), [ch10](../ch10-standard-traits/README.md)의 연산자 안에서 넘침이 났을 때의 판단(2.3절), [ch11](../ch11-drop-raii/README.md)의 이중 패닉과 `catch_unwind`·`AssertUnwindSafe`(2.5절)입니다. 다음 본편 내용을 안다고 보고 진행합니다.

- [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.2~2.4절, 2.7절, 2.11절: `panic!`, 되감기와 abort, `unwrap`/`expect`, 패닉과 `Result`의 선택 기준
- [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md) 2.10절: `new`와 `build`, 문서의 `# Panics` 절
- [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.9절, [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 2.9절: 검사·래핑·포화 연산, `Wrapping`/`Saturating`
- [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.10절: `RefCell`, `try_borrow_mut`
- [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 2.8~2.9절: 패닉 훅, `catch_unwind`, `UnwindSafe`와 `AssertUnwindSafe`
- [ch4](../ch4-idiomatic-errors/README.md): 돌려줄 에러 타입을 어떻게 설계하는가

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 라이브러리 함수가 패닉하면 부르는 쪽이 잃는 것을 부르는 쪽 코드로 설명하고, 같은 기능을 `Result`를 돌려주는 API로 바꿀 수 있다.
2. 패닉해도 되는 경우(불변식 위반, 아주 드문 실패, 코드에 적은 값)를 구별하고, 패닉판을 줄 때는 `# Panics` 문서와 `#[track_caller]`를 붙여 실패를 돌려주는 판과 짝으로 제공할 수 있다.
3. 인덱싱, 슬라이싱, 나눗셈, 정수 오버플로, `RefCell` 같은 숨은 패닉 지점을 찾아 `get`, `checked_*`, `try_borrow_mut` 같은 검사 연산으로 바꿀 수 있고, 오버플로가 빌드에 따라 다르게 동작한다는 것을 설명할 수 있다.
4. `catch_unwind`가 예외 처리의 대용품이 될 수 없는 이유(붙잡지 못하는 경우, 패닉이 지나간 뒤 어긋난 상태)를 설명하고, 꼭 써야 할 때 상태를 지키는 방법을 고를 수 있다.
5. `grep`과 clippy의 restriction 린트(`[lints.clippy]`)로 새로 들어오는 패닉 지점을 자동으로 막을 수 있다.

## 2. 핵심 개념

2.1절에서 패닉이 어디서 멈추는지 정리하고, 2.2~2.6절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 이 Item은 "패닉해도 되는 경우"가 지침의 절반이라 마지막 항목을 특히 자세히 적었습니다. 지침 1~5는 책의 Item 18이 권하는 범위이고, 각 지침 안에서 책에 없는 판단을 더한 곳은 2.7절에 모았습니다.

### 2.1 지도: 패닉은 어디서 멈추는가

| 상황 | 패닉 뒤에 일어나는 일 | `catch_unwind`로 붙잡나 | 확인한 곳 |
| --- | --- | --- | --- |
| 기본(되감기) | 패닉 훅이 메시지를 출력하고, 스택을 되감으며 지역 변수를 버리고, 그 스레드가 끝남. `main` 스레드면 종료 코드 101, 다른 스레드면 `join`이 `Err` | 붙잡음 | Part 0 ch9 2.3절, Part 1 ch8 2.9절 |
| 프로필에 `panic = "abort"` | 훅이 메시지를 출력한 뒤 프로세스가 바로 중단(abort)됨 | 못 붙잡음 | [`projects/panic_strategy`](projects/panic_strategy) |
| 기본 전략이 중단인 타깃 | 위와 같음 | 못 붙잡음 | `rustc --print cfg --target wasm32-unknown-unknown` |
| `extern "C"` 함수 밖으로 나가려는 패닉 | 중단 (1.81부터) | 못 붙잡음 | `projects/panic_strategy` |
| 되감기 중에 `drop`이 또 패닉(이중 패닉) | 중단 | 못 붙잡음 | `projects/panic_strategy` |

`rustc --print cfg --target <타깃>`은 그 타깃의 설정을 출력합니다. 이 툴체인에서 `wasm32-unknown-unknown`, `wasm32-wasip1`, `thumbv7em-none-eabihf`(임베디드)는 `panic="abort"`를, 이 저장소를 확인한 `aarch64-apple-darwin`은 `panic="unwind"`를 출력했습니다. 책은 WebAssembly 같은 타깃은 설정과 상관없이 늘 중단한다고 설명합니다.

표의 아래 네 줄이 이 챕터의 전제입니다. 라이브러리는 자신이 어떤 프로필과 타깃으로 빌드되어 어떤 함수 안에서 불릴지 모릅니다. 그래서 "패닉해도 부르는 쪽이 붙잡으면 된다"는 가정 위에 API를 세울 수 없습니다.

### 2.2 지침 1: 라이브러리는 패닉 대신 `Result`를 돌려준다

**문제 상황.** 체스 기보 프로그램이 쓰는 라이브러리가 칸 이름(`"e4"`)을 `Square`로 바꿉니다. 이름은 사용자가 손으로 적은 기보에서 옵니다. 잘못 적은 이름은 버그가 아니라 늘 일어나는 일입니다.

**비권장 코드.** 잘못된 이름에 패닉하는 함수만 공개합니다([`01_result_not_panic.rs`](examples/01_result_not_panic.rs)의 `board_v1`).

```rust
/// # Panics
///
/// 이름이 "열 a~h + 행 1~8"의 두 글자가 아니면 패닉합니다.
pub fn square(name: &str) -> Square {
    // ...
    if !('1'..='8').contains(&rank) {
        panic!("행은 1~8이어야 합니다: {name:?}");
    }
    // ...
}
```

문서에 패닉 조건을 적었으니 성실한 라이브러리처럼 보입니다. 그런데 이 함수를 받은 부르는 쪽에는 두 가지 길만 남습니다.

- **그대로 부릅니다.** 기보 `["e2", "e4", "G8", "f6", "b9", "c3"]`를 `map(..).collect()`로 읽으면, 다섯째 칸 `"b9"`에서 패닉해 앞에서 읽은 네 칸까지 모두 잃습니다(1-a). 예제는 `catch_unwind`로 붙잡았지만, 실제 프로그램이라면 기보 하나를 잘못 적은 사용자 때문에 프로그램이 끝납니다.
- **미리 거릅니다.** 라이브러리가 패닉할 입력을 넘기지 않으려면, 부르는 쪽이 라이브러리의 규칙을 짐작해서 다시 써야 합니다. 1-b의 `looks_like_square`는 대문자 열을 몰라서 라이브러리가 받아 주는 `"G8"`을 버리고, 행 검사를 빠뜨려서 `"b9"`를 통과시켜 결국 패닉합니다. 규칙을 정확히 베껴도, 라이브러리가 판을 올리며 규칙을 바꾸면 다시 어긋납니다.

셋째 길로 `catch_unwind`가 있지만, 2.5절에서 보듯 믿을 수 있는 그물이 아닙니다.

**권장 코드.** 같은 규칙을 `Result`로 알립니다(`board_v2`). 에러 타입을 어떻게 설계할지는 [ch4](../ch4-idiomatic-errors/README.md)를 따랐습니다.

```rust
pub fn parse_square(name: &str) -> Result<Square, SquareError> {
    // ...
    if !('1'..='8').contains(&rank) {
        return Err(SquareError::Rank(rank));
    }
    // ...
}
```

이제 실패를 어떻게 다룰지 부르는 쪽이 정합니다. 예제의 2번 구역은 같은 기보를 세 정책으로 읽습니다.

| 정책 | 부르는 쪽 코드 | 결과 |
| --- | --- | --- |
| 잘못된 칸만 건너뛰고 알림 | `match parse_square(name) { Ok(..) => .., Err(e) => .. }` | 다섯 칸을 읽고 `"b9"`의 이유를 출력 |
| 첫 실패에서 멈추고 위로 넘김 | `parse_square(name).map_err(\|source\| RecordError { line, source })?` | "기보 5번째 칸"이라는, 부르는 쪽만 아는 문맥을 붙여 전파 |
| 실패할 리 없는 값이면 패닉을 고름 | `parse_square("e1").expect("코드에 적은 칸 이름 \"e1\"은 올발라야 합니다")` | `e1` |

마지막 줄이 핵심입니다. `Result`를 받은 쪽은 `expect` 한 줄로 언제든 패닉을 고를 수 있지만, 패닉을 받은 쪽은 그것을 `Result`로 되돌릴 방법이 없습니다. 선택권은 한 방향으로만 흐르므로, 어떻게 쓰일지 모르는 라이브러리는 선택권이 넓은 쪽을 줍니다. [ch7](../ch7-builders/README.md) 2.4절의 `build()`가 규칙 위반에 패닉하지 않고 `Result`를 돌려준 것도 같은 이유입니다. 규칙에 맞는지는 사용자가 넣은 값에 달려 있고, 틀린 값을 어떻게 다룰지는 빌더를 쓰는 쪽이 정해야 합니다.

**따르지 않아도 되는 경우.**

- **`main`을 쥐고 있을 때.** 애플리케이션의 맨 위에는 에러를 넘길 곳이 더 없으므로 패닉, `unwrap`, `expect`를 써도 됩니다(책). 다만 사용자에게 보여 줄 실패라면 메시지와 종료 코드로 끝내는 편이 친절합니다(Part 0 ch9 2.10절).
- **테스트, 예제, 프로토타입.** Part 0 ch9 2.11절의 표 그대로입니다. 테스트에서는 실패하면 패닉하는 것이 정답입니다(2.6절의 `clippy.toml`).
- **부르는 쪽이 일으킬 수 없는 실패.** 라이브러리 자신의 버그로만 생기는 실패는 다음 절에서 봅니다.

### 2.3 지침 2: 패닉판을 주려면 실패를 돌려주는 판과 짝으로 준다

**문제 상황.** 글자로 그림을 그리는 캔버스 라이브러리가 있습니다. 부르는 쪽 코드에는 두 종류의 좌표가 섞여 있습니다. 코드에 적은 상수 좌표는 틀리면 버그이고, 사용자가 정한 도형의 좌표는 캔버스 밖으로 나가는 것이 정상입니다. 게다가 `canvas[(x, y)]` 문법을 주고 싶은데, `Index::index`는 `&Self::Output`을 돌려줘야 하므로 실패를 값으로 알릴 수 없습니다.

표준 라이브러리는 이 문제를 **두 판을 함께 주는** 관례로 풉니다. 아래는 이 챕터의 예제와 표준 라이브러리 문서의 Panics 절로 확인한 짝입니다.

| 패닉판 | 실패를 돌려주는 판 | 패닉하는 조건 |
| --- | --- | --- |
| `v[i]`, `&v[a..b]` | `v.get(i)`, `v.get(a..b)` | 범위 밖 |
| `s.split_at(mid)` | `s.split_at_checked(mid)` (1.80) | `mid`가 길이를 넘음. `str`은 글자 경계가 아닐 때도 |
| `cell.borrow_mut()` | `cell.try_borrow_mut()` | 이미 빌려 있음 |
| `a - b` (정수) | `a.checked_sub(b)` | 결과가 타입의 범위를 벗어남(디버그 빌드, 2.4절) |
| `d1 - d2` (`Duration`) | `d1.checked_sub(d2)` | 결과가 음수 |
| `thread::spawn(f)` | `thread::Builder::new().spawn(f)` | 운영체제가 스레드를 만들지 못함 |
| `v.push(x)` | `v.try_reserve(1)` 뒤에 `push` | 새 용량이 `isize::MAX` 바이트를 넘음 |

아래 두 줄은 책이 말하는 "아주 드문 실패"에 해당합니다. 스레드를 만들 때마다 `Result`를 돌려주면 부르는 쪽 코드에 `unwrap()`이 흩어지므로, 흔히 쓰는 쪽을 패닉판으로 두고 실패를 다뤄야 하는 드문 쪽에 `Result`판을 줍니다.

**비권장 코드.** `canvas_v1`처럼 패닉판만 줍니다([`02_panic_flavor_pairs.rs`](examples/02_panic_flavor_pairs.rs)). 좌표가 밖으로 나갈 수 있는 쪽은 2.2절의 `looks_like_square`처럼 경계 검사를 직접 다시 써야 합니다. 게다가 패닉 메시지가 가리키는 위치가 라이브러리 안입니다.

```text
    plot(9, 1) → 패닉: 8x4 캔버스 밖의 좌표입니다: (9, 1)
      패닉 위치: 02_panic_flavor_pairs.rs:37 (부른 줄: 194행)
```

37행은 `plot` 본문의 `panic!`입니다. 부르는 쪽 코드에 `plot` 호출이 수십 곳 있다면, 이 메시지로는 어느 호출이 틀렸는지 알 수 없습니다. 백트레이스를 켜야 비로소 찾을 수 있습니다.

**권장 코드.** 경계 검사를 한 곳(`offset`)에 두고, 실패를 돌려주는 판 위에 패닉판을 얇게 얹습니다.

```rust
/// # Panics
///
/// (x, y)가 캔버스 밖이면 패닉합니다. 좌표가 사용자 입력처럼 바깥에서 온다면
/// [`Canvas::try_plot`]을 쓰세요.
#[track_caller] // 패닉 위치를 이 함수의 본문이 아니라 부른 줄로 보고합니다.
pub fn plot(&mut self, x: usize, y: usize, ch: char) {
    if let Err(e) = self.try_plot(x, y, ch) {
        panic!("{e}");
    }
}

/// # Errors
///
/// (x, y)가 캔버스 밖이면 아무것도 바꾸지 않고 `OutOfCanvas`를 돌려줍니다.
pub fn try_plot(&mut self, x: usize, y: usize, ch: char) -> Result<(), OutOfCanvas> {
    let i = self.offset(x, y).ok_or(OutOfCanvas { x, y })?;
    self.cells[i] = ch; // offset이 범위를 확인했으므로 패닉하지 않습니다.
    Ok(())
}
```

세 가지를 갖췄습니다.

- **짝으로 줍니다.** 상수 좌표는 `plot`이나 `canvas[(x, y)]`로, 바깥에서 온 좌표는 `try_plot`이나 `get`으로 씁니다. 예제의 2번 구역은 사용자가 정한 대각선을 `try_plot`으로 그어, 캔버스 밖의 점 2개를 패닉 없이 건너뜁니다.
- **`# Panics` 절에 조건과 대안을 적습니다.** 책은 패닉을 문서의 전용 절에 적으라고 권하고, Rust API 가이드라인(C-FAILURE)도 같은 것을 권합니다. 표준 라이브러리의 `RefCell::borrow_mut` 문서도 Panics 절에 "패닉하지 않는 판은 `try_borrow_mut`"이라고 적습니다. 실패를 돌려주는 판에는 `# Errors` 절을 둡니다.
- **패닉판에는 `#[track_caller]`를 붙입니다.** 이 속성이 붙은 함수 안에서 패닉하면, 패닉 위치가 그 함수를 부른 곳으로 보고됩니다. `Option::unwrap`, `RefCell::borrow_mut`, `v[i]`가 내 코드의 줄을 가리키는 것도 이 속성 덕분입니다. 예제에서 같은 호출의 패닉 위치가 부른 줄과 같아집니다.

```text
    plot(9, 1) → 패닉: 캔버스 밖의 좌표입니다: (9, 1)
      패닉 위치: 02_panic_flavor_pairs.rs:201 (부른 줄: 201행)
    canvas[(9, 1)] → 패닉: 캔버스 밖의 좌표입니다: (9, 1)
      패닉 위치: 02_panic_flavor_pairs.rs:204 (부른 줄: 204행)
```

`Index` 구현에는 속성을 붙이지 않았는데도 위치가 부른 줄입니다. 1.100.0의 `core` 소스에서 `Index::index`와 `IndexMut::index_mut`은 트레이트 선언에 `#[track_caller]`가 붙어 있고, 직접 만든 구현도 그것을 따랐습니다.

**연산자도 같은 판단입니다.** `Add`, `Sub`, `Index` 같은 연산자 트레이트는 반환 타입이 정해져 있어 실패를 값으로 알릴 수 없으므로, 연산자 쪽은 패닉판이 됩니다. 그래서 정수 타입은 `+`가 넘침에 패닉하게(디버그 빌드) 두고 `checked_add`를 함께 줍니다. [ch10](../ch10-standard-traits/README.md) 연습 문제의 분수 덧셈이 `i32`를 넘을 때의 답도 이것입니다. 연산자는 정수처럼 패닉하고 `# Panics`에 적되, `checked_add(self, rhs) -> Option<Self>` 메서드를 함께 줍니다.

**따르지 않아도 되는 경우.**

- **불변식 위반.** 책은 입력이 틀려서가 아니라 내부 데이터가 깨져서 생기는 실패에는 패닉이 맞다고 말합니다. `Canvas`는 "`cells`는 `width * height`칸"이라는 불변식을 지키고, `row`는 그것에 기댑니다.

  ```rust
  let cells = self
      .cells
      .get(start..start + self.width)
      // 이 expect는 Canvas 자신의 코드가 틀렸을 때만 실패합니다(불변식 위반).
      .expect("Canvas의 cells는 width * height칸이어야 합니다");
  ```

  부르는 쪽이 일으킬 수 있는 실패(범위 밖의 행)는 그 위에서 `None`으로 돌려주고, 남은 실패는 `Canvas`의 버그뿐입니다. 부르는 쪽이 고칠 수 있는 일이 아니므로 `Result`판을 두지 않습니다. 예제의 3번 구역은 버그를 흉내 내는 `buggy_clear`로 불변식을 깨고 `row(0)`을 부릅니다. 이번에는 패닉 위치가 `row` 안의 `expect` 줄(125행)입니다. 고칠 곳이 라이브러리 안이므로 그것이 맞고, 그래서 이런 함수에는 `#[track_caller]`를 붙이지 않습니다.
- **`expect` 메시지는 "왜 실패할 리 없는가"로 씁니다.** 표준 라이브러리 문서(`std::error` 모듈의 Common Message Styles 절)는 `expect`의 메시지를 "무엇이 잘못됐다"가 아니라 "무엇이 참이어야 했는가"로 쓰라고 권합니다. 영어로는 "should"를 넣으라는 조언이고, 한국어로는 "~이어야 합니다"입니다. `Result::expect`는 메시지 뒤에 원인 에러를 붙여 출력하므로(04의 `회원 번호가 올발라야 합니다: BadMember("1203")`), 메시지에는 원인과 겹치지 않는 정보, 즉 가정과 그 근거를 담는 것이 좋습니다. 2.2절의 `"코드에 적은 칸 이름 \"e1\"은 올발라야 합니다"`도 이 모양입니다. 나중에 이 값이 사용자 입력으로 바뀌면 메시지가 거짓이 되므로, 고쳐야 할 `expect`가 눈에 띕니다.
- **아주 드문 실패.** 위 표의 `thread::spawn`처럼 패닉판을 기본으로 둡니다. 다만 표준 라이브러리도 드문 쪽을 위해 `Result`판을 함께 줍니다.

### 2.4 지침 3: 숨은 패닉 지점을 검사 연산으로 바꾼다

`panic!`과 `unwrap`은 눈에 띄지만, 패닉하는 코드의 대부분은 평범한 연산자와 대괄호입니다. 책은 `unwrap`/`expect`/`unwrap_err`/`expect_err`, `unreachable!`, 범위 밖 인덱싱, 0으로 나누기를 꼽습니다. 아래 표는 거기에 자주 만나는 지점을 더한 것으로, 메시지는 모두 이 툴체인에서 실행해 확인했습니다.

| 코드 | 이 툴체인의 패닉 메시지 | 릴리스 빌드 | 바꿀 것 |
| --- | --- | --- | --- |
| `o.unwrap()` (`None`) | ``called `Option::unwrap()` on a `None` value`` | 패닉 | `?`, `match`, `unwrap_or*`, `ok_or` |
| `unreachable!()` | `internal error: entered unreachable code` | 패닉 | 그 경우가 생기지 않는 타입(열거형) |
| `v[i]` | `index out of bounds: the len is 3 but the index is 5` | 패닉 | `v.get(i)` |
| `&v[a..b]` | `range end index 5 out of range for slice of length 3` | 패닉 | `v.get(a..b)`, 반복자 |
| `&s[..n]` (`str`) | `end byte index 1 is not a char boundary; it is inside '가' (bytes 0..3 of string)` | 패닉 | `s.chars().take(n)`, `s.get(..n)` |
| `map[&key]` (`HashMap`) | `no entry found for key` | 패닉 | `map.get(&key)` |
| `a / b`, `a % b` (`b == 0`) | `attempt to divide by zero` | 패닉 | `checked_div`, `checked_rem` |
| `a - b`, `len() - 1` 등 | `attempt to subtract with overflow` | 래핑(틀린 값) | `checked_*`, `saturating_*`, `strict_*` |
| `d1 - d2` (`Duration`) | `overflow when subtracting durations` | 패닉 | `checked_sub`, `saturating_sub` |
| `cell.borrow_mut()` | `RefCell already borrowed` | 패닉 | `try_borrow_mut`, 가드 범위 줄이기 |
| `v.remove(i)` | `removal index (is 5) should be < len (is 3)` | 패닉 | 길이 확인 뒤 호출 |
| `v.chunks(0)` | `chunk size must be non-zero` | 패닉 | 0을 받지 않는 타입(`NonZero`, Part 1 ch14 2.10절) |

**문제 상황.** 강의 시청 기록을 한 줄로 요약하는 함수를 씁니다. 제목이 길면 줄이고, 평균 시청 시간, 최근 3회 기록, 목표까지 남은 시간을 보여 줍니다.

**비권장 코드.** 처음 떠오르는 대로 씁니다([`03_hidden_panics.rs`](examples/03_hidden_panics.rs)의 `summary_panicky`). `panic!`도 `unwrap`도 없지만 패닉 지점이 네 곳입니다.

```rust
let title = if c.title.len() > 10 {
    format!("{}…", &c.title[..10])                // ① 글자 중간을 자르면 패닉
} else {
    c.title.to_string()
};
let total: u32 = c.sessions.iter().sum();
let average = total / c.sessions.len() as u32;    // ② 기록이 없으면 0으로 나누기
let recent = &c.sessions[c.sessions.len() - 3..]; // ③ 3회 미만이면 usize 아래로 넘침
let left = c.goal_min - total;                    // ④ 목표를 넘기면 u32 아래로 넘침
```

영문 제목에 기록이 넉넉한 첫 강의로만 시험하면 네 곳 모두 통과합니다. 예제는 각 지점을 하나씩 건드리는 강의 넷을 더 넣어, ②·③·①·④가 차례로 패닉하는 것을 보입니다. ①은 한글이 한 글자에 3바이트라서 생깁니다. `"소유권과 대여 완전 정복"`의 10번째 바이트는 `'과'`의 한가운데입니다.

**권장 코드.** 같은 일을 패닉하지 않는 연산으로 씁니다(`summary_checked`).

```rust
let title = if c.title.chars().count() > 10 {
    format!("{}…", c.title.chars().take(10).collect::<String>()) // ① 글자 단위로 자름
} else {
    c.title.to_string()
};
let Some(total) = c.sessions.iter().try_fold(0u32, |acc, &m| acc.checked_add(m)) else {
    return format!("{title} | 합계가 너무 큼"); // sum()도 넘칠 수 있음
};
let Some(average) = total.checked_div(c.sessions.len() as u32) else { // ② 0이면 None
    return format!("{title} | 기록 없음");
};
let skip = c.sessions.len().saturating_sub(3);                         // ③ 0 아래로 안 내려감
let recent: Vec<u32> = c.sessions.iter().skip(skip).copied().collect();
let left = match c.goal_min.checked_sub(total) {                       // ④ 넘으면 None
    Some(m) => format!("목표까지 {m}분"),
    None => format!("목표보다 {}분 더 봄", total.abs_diff(c.goal_min)),
};
```

바꾼 뒤에는 실패가 결과의 일부가 되었습니다. "기록 없음"이나 "목표보다 25분 더 봄"은 패닉을 피하려고 덧붙인 처리가 아니라, 패닉판이 감추고 있던 **요구 사항**입니다. 숨은 패닉 지점을 찾는 일은 처리하지 않은 경우를 찾는 일이기도 합니다. 합계를 내는 `sum()`도 디버그 빌드에서는 넘침에 패닉하므로 `try_fold`와 `checked_add`로 바꿨습니다.

**오버플로는 빌드에 따라 다르게 드러납니다.** 정수 연산의 넘침 검사는 디버그 빌드에서만 켜져 있습니다(Part 0 ch3). 예제를 `--release`로 실행하면 ④는 패닉하지 않고 `목표까지 4294967271분`이라는 조용히 틀린 값을 내고, ③은 `len() - 3`이 래핑된 뒤 슬라이싱에서 `range start index 18446744073709551615 out of range for slice of length 2`로 패닉합니다. 디버그 빌드에서는 패닉으로 드러나던 버그가 릴리스 빌드에서는 틀린 값으로 숨는 것입니다. 빌드와 상관없이 넘침을 패닉으로 알리고 싶다면 1.91에서 안정화된 `strict_sub` 같은 `strict_*` 연산을 쓰고(예제의 2번 구역), 넘침을 값으로 다루려면 `checked_*`를 씁니다. 0으로 나누기와 `Duration`의 뺄셈은 두 빌드 모두에서 패닉했습니다. 프로필의 `overflow-checks`로 릴리스 빌드에서도 검사를 켤 수 있으며, 프로필은 [Part 3 ch6](../../part3-cargo/ch6-profiles-build-performance/README.md)에서 다룹니다.

**`RefCell`의 패닉.** `RefCell`은 대여 규칙을 실행 중에 검사하므로, 규칙을 어기면 컴파일 에러 대신 패닉이 납니다. 예제의 3번 구역은 `borrow()`로 빌려 둔 동안 `borrow_mut()`이 `RefCell already borrowed`로 패닉하고, `try_borrow_mut()`은 같은 상황을 `Err`로 돌려주는 것을 보입니다. 패닉을 피하는 일반적인 방법은 세 가지입니다. 대여 가드를 짧게 쥐고(조건식에서 가드를 만드는 함정은 Part 1 ch7 2.10절), 가드를 쥔 채로 콜백을 부르지 않으며, 재진입이 가능한 곳에서는 `try_borrow_mut`으로 실패를 값으로 받습니다. 애초에 `RefCell`이 필요 없는 구조로 바꾸는 방법은 [ch15](../ch15-borrow-checker/README.md) 2.7~2.8절에 있습니다.

**따르지 않아도 되는 경우.**

- **범위 안임이 바로 보이는 인덱스.** `offset`이 방금 확인한 `i`로 `self.cells[i]`를 쓰는 것처럼, 바로 위의 검사가 범위를 보장하면 인덱싱이 더 읽기 쉽습니다. 01의 `b'a' + self.file`도 `file`이 0~7이므로 넘칠 수 없습니다. 이런 곳에는 근거를 주석으로 남깁니다. 2.6절의 린트를 켠 크레이트라면 `#[expect(clippy::indexing_slicing, reason = "…")]`로 그 자리만 예외로 둡니다.
- **넘침이 버그일 때.** 넘침이 일어나면 코드가 틀린 것이라면 기본 연산자로 두어 디버그 빌드의 패닉으로 알게 하는 것도 방법입니다(Part 1 ch14 2.9절의 표). 다만 릴리스 빌드에서는 위처럼 틀린 값이 되므로, 외부 입력이 닿는 계산이라면 `checked_*`나 `strict_*`를 씁니다.
- 인덱싱의 경계 검사가 성능에 주는 영향은 측정해 보기 전에는 판단하지 않습니다([`ch20-avoid-over-optimizing`](../ch20-avoid-over-optimizing/README.md)).

### 2.5 지침 4: `catch_unwind`를 예외 처리로 쓰지 않는다

**문제 상황.** 패닉하는 함수를 받았는데 고칠 수 없거나, Java·C++의 예외처럼 "실패하면 붙잡아서 넘어가자"는 생각이 들 때가 있습니다. `catch_unwind`는 패닉을 `Err`로 바꿔 주므로 try/catch처럼 보입니다. 책은 이것이 믿을 수 없는 대용품이라고 말합니다. 이유는 두 가지입니다.

**이유 1: 붙잡아도 상태가 어긋난 채 남습니다.** 공용 세탁실의 배정표는 "쉬는 세탁기와 돌아가는 세탁기를 합하면 3대"라는 불변식을 지킵니다. `start`는 세탁기를 먼저 꺼낸 뒤 회원 번호를 라벨로 바꾸는데, 형식이 틀린 번호에 패닉합니다([`04_catch_unwind_is_not_try.rs`](examples/04_catch_unwind_is_not_try.rs)).

```rust
fn start(&mut self, member: &str) -> u8 {
    let machine = self.idle.pop().expect("쉬는 세탁기가 있어야 합니다");
    // 여기서 패닉하면 꺼낸 세탁기는 idle에도 running에도 없습니다.
    let label = label(member).expect("회원 번호가 올발라야 합니다");
    self.running.push((machine, label));
    machine
}

// 비권장: 예외처럼 붙잡고 넘어감
match panic::catch_unwind(AssertUnwindSafe(|| laundry.start(member))) { /* ... */ }
```

요청 넷 중 `"1203"`이 실패한 뒤 배정표에는 세탁기가 2대만 남고, 네 번째 회원은 2번 세탁기가 실제로는 비어 있는데도 "쉬는 세탁기가 있어야 합니다"로 다시 패닉합니다. 패닉이 함수를 중간에서 빠져나가면서 꺼낸 세탁기가 어디에도 기록되지 않았습니다. 패닉이 자료구조를 어긋난 상태로 남기는 문제를 예외 안전성(exception safety)이라고 부르고, 책은 C++ 진영이 이 문제로 오래 고생했다고 짚습니다. 컴파일러도 이 위험을 알려 줍니다. `&mut Laundry`를 캡처한 클로저는 `UnwindSafe`가 아니라서 `AssertUnwindSafe`로 감싸야 컴파일되고([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 4.3절), 그 감싸기가 "패닉 뒤의 상태는 내가 책임진다"는 약속입니다. 1번 구역은 그 약속을 지키지 못한 결과입니다.

**이유 2: 붙잡지 못하는 패닉이 있습니다.** 2.1절 표의 아래 네 줄입니다. [`projects/panic_strategy`](projects/panic_strategy)는 같은 프로그램으로 세 경우를 보입니다(출력은 3절의 `panic_strategy`).

- **중단 프로필.** 썸네일 작업 셋을 하나씩 `catch_unwind`로 감싸면, 되감기 프로필에서는 0으로 나눈 작업만 실패하고 끝까지 갑니다. `[profile.abort]`에 `panic = "abort"`를 두고 실행하면 첫 패닉에서 프로세스가 중단되어 나머지 작업은 실행되지 않습니다.
- **`extern "C"` 함수.** C 라이브러리에 넘길 콜백처럼 `extern "C"`로 정의한 함수에서 패닉하면, 되감기 프로필이고 `catch_unwind` 안에서 불렀어도 `panic in a function that cannot unwind`와 함께 중단됩니다. `extern "C"` 함수는 패닉을 밖으로 내보낼 수 없기 때문입니다(1.81부터, 2.7절).
- **이중 패닉.** 되감기 중에 버려지는 값의 `drop`이 다시 패닉하면 `panic in a destructor during cleanup`과 함께 중단됩니다. [ch11](../ch11-drop-raii/README.md) 2.4절이 `drop`에서 패닉하지 말라고 한 이유입니다.

라이브러리는 자신이 어느 쪽으로 빌드될지 모르므로, `catch_unwind`에 기대는 설계는 프로젝트 전체의 설정에 묶이게 됩니다.

**권장 코드.** 실패를 `Result`로 돌려주고, 실패할 수 있는 일을 먼저 끝낸 뒤 상태를 마지막에 바꿉니다.

```rust
fn try_start(&mut self, member: &str) -> Result<u8, StartError> {
    let label = label(member)?;
    let machine = self.idle.pop().ok_or(StartError::AllBusy)?;
    self.running.push((machine, label)); // 여기부터는 실패할 일이 없습니다.
    Ok(machine)
}
```

`?`로 빠져나가는 지점이 모두 상태를 바꾸기 전에 있으므로, 실패해도 배정표는 그대로입니다(2번 구역, 세탁기 3대). 쉬는 세탁기가 없는 것도 이제 `AllBusy`라는 값입니다. 되감기가 일어나는 동안 불변식을 지키는 일반적인 기법은 Part 10 `ch8-unwinding`에서 다룹니다.

**따르지 않아도 되는 경우: `catch_unwind`를 쓰는 곳.**

- **작업 하나의 버그가 전체를 멈추지 않게 하는 경계.** 스레드 풀이 작업마다, 서버가 요청마다 패닉을 가두는 경우입니다(Part 1 ch8 2.9절). 이때는 패닉이 지나간 상태를 믿지 않습니다. 04의 3번 구역은 고칠 수 없는 패닉판 `start`를 배정표의 사본에서 실행하고, 성공했을 때만 사본을 반영합니다. 대신 요청마다 복제 비용이 듭니다. `thread::spawn`과 `join`도 같은 경계입니다. 스레드가 패닉하면 그 스레드의 상태와 함께 끝나고, `join`이 `Err`를 돌려줍니다. 여러 스레드가 함께 쓰는 `Mutex`는 잠금을 쥔 채 패닉하면 포이즈닝으로 이 사실을 알려 줍니다([`ch17-shared-state-parallelism`](../ch17-shared-state-parallelism/README.md)).
- **FFI 경계.** C 코드가 부르는 Rust 함수에서 패닉이 밖으로 나가지 않게 막는 곳입니다(책). 지금은 막지 않아도 중단되지만, 프로세스를 끝내는 대신 C 쪽에 에러 코드를 돌려주려면 여전히 `catch_unwind`로 붙잡아 바꿉니다. 자세한 것은 [`ch34-ffi-boundaries`](../ch34-ffi-boundaries/README.md)에서 다룹니다.

### 2.6 지침 5: 사람의 주의 대신 도구로 막는다

지침 1~4를 지킨 코드도 다음 수정에서 `unwrap()` 하나, `v[i]` 하나가 들어오면 깨집니다. 책은 사람의 주의에 기대지 말고, 패닉 지점을 자동으로 찾는 검사를 CI에 넣으라고 권합니다.

**문제 상황.** [`projects/panic_free_lib`](projects/panic_free_lib)은 타이머 길이(`"1h30m"`)를 초로 읽고 쓰는 라이브러리이고, "이 크레이트는 패닉하지 않는다"고 약속합니다. 이 약속을 사람이 리뷰로만 지키면 언젠가 빠집니다.

**비권장 코드: `grep`만 씁니다.** 책이 소개하는 가장 단순한 검사는 패닉하는 호출의 이름을 `grep`으로 찾는 것입니다.

```console
$ grep -nE '\.(unwrap|expect)(_err)?\(|panic!|unreachable!|todo!' src/lib.rs
111:        assert_eq!(parse_timer("1h30m").unwrap(), 5400);
112:        assert_eq!(parse_timer(" 45s ").unwrap(), 45);
113:        assert_eq!(parse_timer("2h5s").unwrap(), 7205);
126:            assert_eq!(parse_timer(spec).unwrap_err(), expected, "{spec:?}");
134:            assert_eq!(parse_timer(&format_timer(seconds)).unwrap(), seconds);
```

찾은 다섯 줄은 모두 테스트 코드라서 고칠 필요가 없고, 반대로 `v[i]`나 `a * b`처럼 이름이 없는 패닉 지점은 처음부터 찾지 못합니다. 책도 인덱싱과 나눗셈은 이런 검사로 찾기 어렵다고 말합니다. 시작으로는 쓸 만하지만 이것만으로는 부족합니다.

**권장 코드: clippy의 restriction 린트를 켭니다.** clippy에는 패닉할 수 있는 코드를 잡는 린트가 있지만, 모두 restriction 그룹(일부러 고른 프로젝트만 켜는 린트)이라 기본으로 꺼져 있습니다(`clippy-driver -W help`로 확인). `Cargo.toml`의 `[lints.clippy]` 테이블로 패키지 전체에 켭니다.

```toml
[lints.clippy]
unwrap_used = "deny"             # .unwrap()
expect_used = "deny"             # .expect(..)
panic = "deny"                   # panic!
unreachable = "deny"             # unreachable!
todo = "deny"                    # todo!
indexing_slicing = "deny"        # v[i], &v[a..b] (범위 안임을 컴파일러가 아는 상수 인덱스는 제외)
string_slice = "deny"            # &s[a..b] (글자 경계가 아니면 패닉)
arithmetic_side_effects = "deny" # 넘치거나 0으로 나눌 수 있는 + - * / %
# pedantic 그룹. 패닉할 수 있는 pub fn에 문서의 # Panics 절이 없으면 알려 줍니다.
missing_panics_doc = "warn"
```

테스트는 실패하면 패닉하는 것이 맞으므로 `clippy.toml`에서 테스트 코드만 풀어 줍니다.

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
```

누군가 `parse_timer`를 "간단하게" 고쳐 `number.unwrap_or(0) * 10 + digit`, `.expect(..)`, `total += n * unit`을 넣으면 `cargo clippy`가 이렇게 멈춥니다(프로젝트의 사본을 고쳐 확인했습니다. 같은 모양의 에러 블록은 줄였습니다).

```text
error: docs for function which may panic missing `# Panics` section
  --> src/lib.rs:53:1
   |
53 | pub fn parse_timer(spec: &str) -> Result<u32, TimerError> {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
note: first possible panic found here
  --> src/lib.rs:72:17
   |
72 |         let n = number.take().expect("단위 앞에는 숫자가 있어야 합니다");
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: `-D clippy::missing-panics-doc` implied by `-D warnings`

error: arithmetic operation that can potentially result in unexpected side-effects
  --> src/lib.rs:63:27
   |
63 |             number = Some(number.unwrap_or(0) * 10 + digit);
   |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: requested on the command line with `-D clippy::arithmetic-side-effects`

error: used `expect()` on an `Option` value
  --> src/lib.rs:72:17
   |
72 |         let n = number.take().expect("단위 앞에는 숫자가 있어야 합니다");
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: if this value is `None`, it will panic

error: arithmetic operation that can potentially result in unexpected side-effects
  --> src/lib.rs:73:9
   |
73 |         total += n * unit;
   |         ^^^^^^^^^^^^^^^^^
```

`grep`과 달리 이름 없는 패닉 지점(곱셈과 덧셈)을 잡고, 테스트 코드는 건너뜁니다. 직접 확인한 몇 가지를 덧붙입니다.

- `arithmetic_side_effects`는 `seconds / 3600`, `rest % 60`처럼 0이 아닌 상수로 나누는 연산에는 경고하지 않았고, `d / i`처럼 나누는 수가 변수이면 경고했습니다. `indexing_slicing`은 `let v = [1, 2];`의 `v[0]`, `&v[1..]`처럼 컴파일러가 범위 안임을 아는 상수 인덱스는 건너뛰었습니다.
- `indexing_slicing`은 `&s[..2]` 같은 문자열 슬라이싱을 잡지 않았습니다. 그것은 `string_slice`가 잡습니다.
- `[lints.clippy]` 대신 크레이트 루트에 `#![deny(clippy::unwrap_used)]`처럼 속성으로 켜도 같습니다. `[lints]` 테이블은 Cargo 1.74에서 생겼고, 설정을 소스 밖 한 곳에 모을 수 있습니다.
- 증명된 곳은 그 자리만 예외로 둡니다. `#[expect(clippy::indexing_slicing, reason = "길이가 3 이상임을 바로 위에서 확인했습니다")]`를 슬라이싱하는 `let` 문에 붙이면 통과하고, 속성을 지우면 `slicing may panic`으로 멈추는 것을 확인했습니다. `#[allow]`가 아니라 `#[expect]`를 쓰면, 나중에 그 자리의 슬라이싱이 사라져 린트가 나지 않게 되었을 때 쓸모없어진 속성도 경고로 알려 줍니다.

**`no-panic` 크레이트.** 책이 소개하는 더 강한 도구입니다. crates.io의 이름은 `no-panic`(2026-11-19 기준 0.1.37)이고, 함수에 `#[no_panic]` 속성을 붙이면 컴파일러가 그 함수에서 패닉할 길이 없다는 것을 증명하지 못할 때 **링크 단계의 에러**(`ERROR[no-panic]: detected panic in function ...`)를 냅니다. 문서에 따르면 검사가 링크 단계에서 일어나므로 링커를 부르지 않는 `cargo check`에서는 동작하지 않고, 최적화해야 증명되는 함수는 디버그 빌드에서 실패할 수 있어 릴리스 빌드에서만 켜거나 `opt-level`을 올리라고 안내하며, `panic = "abort"` 프로필에서는 효과가 없습니다. 이 챕터는 외부 크레이트를 추가하지 않았으므로 직접 실행해 보지는 않았습니다.

**따르지 않아도 되는 경우.** restriction 린트는 비용이 큽니다. `arithmetic_side_effects`를 켜면 평범한 `i + 1`도 모두 `checked_*`나 `wrapping_*`으로 바꿔야 합니다. 그래서 프로젝트 전체가 아니라 "패닉하지 않겠다"고 약속한 라이브러리의 핵심부나 FFI 경계 가까운 코드에 켭니다. 애플리케이션 전체에 켤 린트를 고르는 일반론은 [`ch29-listen-to-clippy`](../ch29-listen-to-clippy/README.md), CI에서 돌리는 방법은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md), 퍼즈 테스트로 패닉하는 입력을 찾는 방법은 [`ch30-beyond-unit-tests`](../ch30-beyond-unit-tests/README.md)에서 다룹니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **`extern "C"` 함수 밖으로 나가는 패닉은 중단됩니다.** 책은 패닉의 전파가 FFI 경계와 잘 어울리지 않는다고만 설명합니다. 1.81.0 릴리스 노트에 "Abort on uncaught panics in `extern "C"` functions"가 있고, `projects/panic_strategy`의 `extern-c` 모드에서 `catch_unwind` 안에서 부른 `extern "C"` 함수의 패닉이 `panic in a function that cannot unwind`, `thread caused non-unwinding panic. aborting.`과 함께 중단되는 것을 확인했습니다. 경계를 넘어 되감기를 의도적으로 허용하는 `extern "C-unwind"`는 1.71.0에서 안정화되었습니다(Part 9 `ch9-memory-panic-linkage`).
- **중단 메시지.** 책의 중단 예시는 셸이 출력하는 `Abort trap`입니다. 이 툴체인에서 `panic = "abort"` 프로필은 기본 훅의 패닉 메시지만 출력하고 별도 안내 없이 중단했고(이 환경의 zsh에서 종료 상태 134 = 128 + SIGABRT의 번호 6), `extern "C"`와 이중 패닉의 중단은 `RUST_BACKTRACE=0`이어도 백트레이스를 출력한 뒤 위의 `aborting.` 줄로 끝났습니다.
- **`String::from_utf8_unchecked`는 패닉판이 아닙니다.** 책은 `from_utf8_unchecked`와 `from_utf8`을 두 판을 함께 주는 예로 들고, 잘못된 입력의 패닉이 그 `String`을 쓸 때로 미뤄진다고 설명합니다. 1.100.0 문서에서 `from_utf8_unchecked`는 `unsafe fn`이고, Safety 절은 잘못된 UTF-8이 메모리 안전성 문제를 일으킬 수 있다고 적습니다. 패닉이 아니라 미정의 동작이므로, 이 챕터는 안전한 짝(2.3절의 표)을 예로 썼습니다.
- **책 이후 생긴 도구와 API.** `[lints]` 테이블(Cargo 1.74), `#[expect]` 린트 속성(1.81), `split_at_checked`(1.80), `PanicHookInfo`(1.81, 예제 02의 훅이 받는 타입), 페이로드를 문자열로 바로 꺼내는 `PanicHookInfo::payload_as_str`(1.91), 빌드와 상관없이 넘침에 패닉하는 `strict_*` 연산(1.91), `const`에서 쓸 수 있게 된 `Option::unwrap`/`expect`(1.83). 마지막 것 덕분에 `const`의 값을 계산하다 `expect`가 실패하면 실행 중 패닉이 아니라 컴파일 에러(E0080)가 됩니다([Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 4.5절). 모두 로컬 문서의 안정화 버전 표시와 릴리스 노트로 확인했습니다.
- **책에 없는 지침.** 다음은 책의 Item 18이 명시적으로 권하지 않는 내용입니다(2026-11-19에 온라인판에서 확인). 책은 `Result` 선호, 패닉해도 되는 네 경우와 문서화, 숨은 패닉 지점(`unwrap` 계열, `unreachable!`, 인덱싱, 0으로 나누기), `catch_unwind`의 한계와 FFI, `grep`·`no_panic`·CI를 다룹니다. clippy 린트는 이름으로 들지 않습니다.
  - 지침 2의 `#[track_caller]`와 연산자 트레이트에 `checked_*`를 짝지어 주는 판단: `core`의 `Index` 트레이트 선언(1.100.0 소스), 정수 타입의 관례, 예제 02의 실행 결과가 근거입니다.
  - 지침 2의 `expect` 메시지 관례: `std::error` 모듈 문서의 Common Message Styles 절과 `Option::expect`·`Result::expect` 문서의 Recommended Message Style 절이 근거입니다.
  - 지침 3 표에서 책이 들지 않은 지점(문자열 슬라이싱, `HashMap` 인덱싱, 정수 오버플로와 빌드 차이, `Duration`, `RefCell`, `Vec::remove`, `chunks(0)`)과 검사 연산으로 바꾸는 방법: 각 API 문서의 Panics 절과 직접 실행한 결과가 근거입니다.
  - 지침 4의 "실패할 수 있는 일을 먼저, 상태는 마지막에"와 "붙잡은 뒤에는 사본을 버린다", 이중 패닉과 중단 타깃: `catch_unwind` 문서(`UnwindSafe`가 예외 안전성을 타입에 담는다는 설명), `rustc --print cfg`, 예제 04와 `projects/panic_strategy`의 실행 결과가 근거입니다.
  - 지침 5의 clippy restriction 린트, `[lints.clippy]`, `clippy.toml`의 `allow-*-in-tests`, `#[expect]`로 예외 두기: clippy 1.100.0의 기본 수준(`clippy-driver -W help`), Clippy 문서의 Lint Configuration, `projects/panic_free_lib`의 실행 결과가 근거입니다.

### 2.8 기억할 것

- 라이브러리는 실패를 `Result`로 돌려줍니다. 받은 쪽은 `expect`로 언제든 패닉을 고를 수 있지만, 패닉을 받은 쪽은 되돌릴 수 없습니다.
- 패닉판을 주려면 실패를 돌려주는 판과 짝으로 주고, `# Panics`에 조건과 대안을 적고, `#[track_caller]`를 붙입니다. 부르는 쪽이 일으킬 수 없는 불변식 위반에는 "~이어야 합니다" 모양의 `expect`로 충분합니다.
- 인덱싱, 슬라이싱, 나눗셈, 뺄셈, `RefCell`이 숨은 패닉 지점입니다. `get`, `chars`, `checked_*`, `try_borrow_mut`으로 바꾸면 감춰져 있던 경우가 결과로 드러납니다. 넘침은 릴리스 빌드에서 패닉이 아니라 틀린 값이 됩니다.
- `catch_unwind`는 try/catch가 아닙니다. 중단 프로필, `extern "C"`, 이중 패닉에서는 붙잡지 못하고, 붙잡아도 상태가 어긋난 채 남습니다. 작업 경계와 FFI 경계에서만 쓰고, 패닉이 지나간 상태는 버립니다.
- 약속은 도구로 지킵니다. `[lints.clippy]`의 restriction 린트로 새 패닉 지점을 막고, 테스트는 `clippy.toml`로 풀어 줍니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행하며, 모든 예제는 종료 코드 0으로 끝납니다. 일부러 낸 패닉은 모두 `catch_unwind`로 안에서 붙잡습니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_result_not_panic.rs`](examples/01_result_not_panic.rs) | 1 | 패닉판만 있는 `board_v1::square` | `board_v2::parse_square`와 부르는 쪽의 세 정책 | 2.2 |
| [`02_panic_flavor_pairs.rs`](examples/02_panic_flavor_pairs.rs) | 2 | 패닉판만 있는 `canvas_v1` | `#[track_caller]` 패닉판 + `try_plot`/`get`, 불변식 위반의 `expect` | 2.3 |
| [`03_hidden_panics.rs`](examples/03_hidden_panics.rs) | 3 | `summary_panicky`, `-`, `borrow_mut` | `summary_checked`, `checked_*`·`strict_*`, `try_borrow_mut` | 2.4 |
| [`04_catch_unwind_is_not_try.rs`](examples/04_catch_unwind_is_not_try.rs) | 4 | 패닉하는 `start`를 `catch_unwind`로 감쌈 | `try_start`, 사본에서 실행하고 버리기 | 2.5 |
| [`projects/panic_strategy`](projects/panic_strategy) | 4 | `catch_unwind`가 붙잡지 못하는 세 경우 | — | 2.1, 2.5 |
| [`projects/panic_free_lib`](projects/panic_free_lib) | 5 | — | `[lints.clippy]`로 패닉 지점을 막은 라이브러리 | 2.6 |

**표준 에러에 대해.** 01과 04는 기본 패닉 훅을 그대로 두므로, 붙잡은 패닉마다 표준 에러에 `thread 'main' (스레드 번호) panicked at 파일:줄:열:`과 메시지가 나옵니다. 괄호 안의 스레드 번호는 실행마다 다르고, `note: run with RUST_BACKTRACE=1 ...` 줄은 처음 한 번만 나옵니다. `RUST_BACKTRACE`를 켜 두었다면 백트레이스도 나옵니다. 02와 03은 이유를 주석에 적고 훅을 바꿨으므로 표준 에러에 아무것도 내지 않습니다. 아래 출력은 표준 출력이며, 예제마다 25번 실행해 같은 결과가 나오는 것을 확인했습니다.

### 01_result_not_panic — 패닉판과 `Result`판을 부르는 쪽

```sh
cargo run -p er-ch18-dont-panic --example 01_result_not_panic
```

```text
1. 비권장: 패닉판만 있는 라이브러리
  1-a. 그대로 부르면 잘못된 칸 하나가 기보 전체를 끝냄
    패닉, 앞에서 읽은 칸도 모두 잃음 (페이로드: 행은 1~8이어야 합니다: "b9")
  1-b. 부르는 쪽이 규칙을 짐작해 미리 거름
    "e2" 통과 → e2
    "e4" 통과 → e4
    "G8" 부르는 쪽 검사에서 걸러 버림
    "f6" 통과 → f6
    패닉으로 멈춤: 행은 1~8이어야 합니다: "b9"
2. 권장: Result판을 받은 부르는 쪽이 정책을 고름
  2-a. 잘못된 칸만 건너뛰고 이유를 알림
    "b9" 건너뜀: 행은 1~8이어야 합니다: '9'
    읽은 칸: e2 e4 g8 f6 c3
  2-b. 첫 실패에서 멈추고 ?로 위에 넘김
    Err: 기보 5번째 칸을 읽지 못했습니다
      원인: 행은 1~8이어야 합니다: '9'
  2-c. 코드에 적은 값처럼 실패할 리 없으면 부르는 쪽이 expect로 패닉을 고름
    백 킹의 시작 칸: e1
```

표준 에러에는 1번 구역의 두 패닉이 나옵니다. 위치가 모두 `board_v1::square` 안(48행)이라는 점도 보아 두세요. 02에서 이것을 고칩니다.

```text
thread 'main' (스레드 번호) panicked at appd-a/ch18-dont-panic/examples/01_result_not_panic.rs:48:13:
행은 1~8이어야 합니다: "b9"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### 02_panic_flavor_pairs — 짝으로 주는 패닉판

패닉 위치(파일:줄)를 표준 출력에서 비교하려고, 메시지와 위치를 기록만 하는 훅으로 바꿨습니다. `(부른 줄: N행)`은 호출 바로 위에서 `line!() + 1`로 구한 값입니다.

```sh
cargo run -p er-ch18-dont-panic --example 02_panic_flavor_pairs
```

```text
1. 비권장: 패닉판만 있는 라이브러리
    plot(9, 1) → 패닉: 8x4 캔버스 밖의 좌표입니다: (9, 1)
      패닉 위치: 02_panic_flavor_pairs.rs:37 (부른 줄: 194행)
2. 권장: #[track_caller]를 붙인 패닉판과 실패를 돌려주는 판을 함께 줌
    plot(9, 1) → 패닉: 캔버스 밖의 좌표입니다: (9, 1)
      패닉 위치: 02_panic_flavor_pairs.rs:201 (부른 줄: 201행)
    canvas[(9, 1)] → 패닉: 캔버스 밖의 좌표입니다: (9, 1)
      패닉 위치: 02_panic_flavor_pairs.rs:204 (부른 줄: 204행)
    get(9, 1) = None, canvas[(2, 1)] = '#'
    try_plot으로 대각선 긋기: 찍은 점 4개, 캔버스 밖이라 건너뛴 점 2개
      ....\...
      ..#..\..
      ......\.
      .......\
3. 불변식이 깨진 내부 버그에는 패닉이 맞음
    row(0) → 패닉: Canvas의 cells는 width * height칸이어야 합니다
      패닉 위치: 02_panic_flavor_pairs.rs:125 (부른 줄: 239행)
```

1번 구역과 3번 구역은 둘 다 위치가 라이브러리 안이지만 뜻이 다릅니다. 1번은 부르는 쪽의 실수인데 위치가 엉뚱한 곳을 가리킨 것이고, 3번은 라이브러리의 버그이므로 위치가 고칠 곳을 바로 가리킨 것입니다.

### 03_hidden_panics — 숨은 패닉 지점과 검사 연산

패닉판의 메시지는 붙잡은 페이로드에서 꺼내 표준 출력에 보여 주므로, 같은 메시지가 표준 에러에 또 나오지 않게 아무것도 하지 않는 훅으로 바꿨습니다.

```sh
cargo run -p er-ch18-dont-panic --example 03_hidden_panics
```

```text
1. 같은 요약 함수의 패닉판과 검사판
    [Rust 101] 기록 3회, 목표 180분
      패닉판: Rust 101 | 평균 41분 | 최근 [30, 45, 50] | 목표까지 55분
      검사판: Rust 101 | 평균 41분 | 최근 [30, 45, 50] | 목표까지 55분
    [Go 입문] 기록 0회, 목표 120분
      패닉판: 패닉 — attempt to divide by zero
      검사판: Go 입문 | 기록 없음
    [Git 기초] 기록 2회, 목표 90분
      패닉판: 패닉 — attempt to subtract with overflow
      검사판: Git 기초 | 평균 22분 | 최근 [20, 25] | 목표까지 45분
    [소유권과 대여 완전 정복] 기록 3회, 목표 200분
      패닉판: 패닉 — end byte index 10 is not a char boundary; it is inside '과' (bytes 9..12 of string)
      검사판: 소유권과 대여 완전… | 평균 41분 | 최근 [40, 50, 35] | 목표까지 75분
    [Rust 101] 기록 3회, 목표 100분
      패닉판: 패닉 — attempt to subtract with overflow
      검사판: Rust 101 | 평균 41분 | 최근 [30, 45, 50] | 목표보다 25분 더 봄
2. 오버플로는 빌드에 따라 패닉하거나 조용히 틀린 값이 됨
    디버그 빌드의 goal - total → 패닉: attempt to subtract with overflow
    wrapping_sub(릴리스 빌드의 -와 같음) → 4294967271
    strict_sub(두 빌드 모두 검사) → 패닉: attempt to subtract with overflow
    checked_sub → None, saturating_sub → 0
3. RefCell: 빌려 둔 동안 고치려 하면 패닉
    borrow_mut → 패닉: RefCell already borrowed
    try_borrow_mut → Err: RefCell already borrowed
    빌려 둔 기록: [30, 45]
    대여를 끝낸 뒤 try_borrow_mut → [30, 45, 50]
```

`--release`를 붙여 실행하면 세 줄이 바뀝니다(2.4절). 1번 구역 `[Git 기초]`의 패닉판이 `패닉 — range start index 18446744073709551615 out of range for slice of length 2`가, 마지막 강의의 패닉판이 `Rust 101 | 평균 41분 | 최근 [30, 45, 50] | 목표까지 4294967271분`이, 2번 구역 첫 줄이 `릴리스 빌드의 goal - total → 4294967271`이 됩니다.

### 04_catch_unwind_is_not_try — `catch_unwind`와 어긋난 상태

```sh
cargo run -p er-ch18-dont-panic --example 04_catch_unwind_is_not_try
```

```text
1. 비권장: 패닉하는 start를 catch_unwind로 감싸 예외처럼 처리
    "101-1203" → 3번 세탁기
    "1203" → 실패 (패닉을 붙잡음)
    "102-305" → 1번 세탁기
    "103-1101" → 실패 (패닉을 붙잡음)
    세탁기 2대 (돌아감 [3, 1], 쉼 [])
2. 권장: 실패를 Result로 돌려주는 try_start
    "101-1203" → 3번 세탁기
    "1203" → Err: 회원 번호는 '동-호' 형식이어야 합니다: "1203"
    "102-305" → 2번 세탁기
    "103-1101" → 1번 세탁기
    세탁기 3대 (돌아감 [3, 2, 1], 쉼 [])
3. 고칠 수 없는 패닉판을 불러야 한다면: 사본에서 실행하고 패닉하면 사본을 버림
    "101-1203" → 3번 세탁기
    "1203" → 실패 (패닉이 지나간 사본은 버림)
    "102-305" → 2번 세탁기
    "103-1101" → 1번 세탁기
    세탁기 3대 (돌아감 [3, 2, 1], 쉼 [])
이 예제의 패닉 전략: unwind(되감기)
```

표준 에러에는 패닉 메시지가 세 번 나옵니다(1번 구역의 `"1203"`과 `"103-1101"`, 3번 구역의 `"1203"`). `catch_unwind`로 붙잡아도 훅은 붙잡히기 전에 실행되기 때문입니다.

```text
thread 'main' (스레드 번호) panicked at appd-a/ch18-dont-panic/examples/04_catch_unwind_is_not_try.rs:76:35:
회원 번호가 올발라야 합니다: BadMember("1203")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' (스레드 번호) panicked at appd-a/ch18-dont-panic/examples/04_catch_unwind_is_not_try.rs:74:39:
쉬는 세탁기가 있어야 합니다
```

### projects/panic_strategy — 붙잡지 못하는 패닉

`Cargo.toml`에 빈 `[workspace]`를 두어 루트 워크스페이스와 분리했습니다. 워크스페이스 멤버 패키지의 `[profile]`은 무시되기 때문입니다(Part 0 ch9 2.3절). `[profile.abort]`는 `dev`를 물려받고 `panic = "abort"`만 바꾼 사용자 정의 프로필입니다. 프로젝트 디렉터리에서 실행합니다. 아래 출력은 표준 출력과 표준 에러가 터미널에 섞여 나온 모습이고, 종료 상태는 macOS의 zsh에서 확인한 값입니다.

```sh
cd appd-a/ch18-dont-panic/projects/panic_strategy
cargo run -q; echo "종료 상태 $?"
```

```text
패닉 전략: unwind(되감기)
  표지.png → 너비 300px

thread 'main' (스레드 번호) panicked at src/main.rs:18:5:
attempt to divide by zero
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
  배너.png → 실패 (패닉을 붙잡음)
  아이콘.png → 너비 64px
작업 3개 중 2개 성공
종료 상태 0
```

```sh
cargo run -q --profile abort; echo "종료 상태 $?"
```

```text
패닉 전략: abort(중단)
  표지.png → 너비 300px

thread 'main' (스레드 번호) panicked at src/main.rs:18:5:
attempt to divide by zero
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
종료 상태 134
```

코드는 한 글자도 다르지 않은데, 중단 프로필에서는 `catch_unwind`가 아무것도 붙잡지 못하고 `아이콘.png` 작업도 실행되지 않았습니다. 134는 셸이 "시그널 6(SIGABRT)으로 끝남"을 128 + 6으로 나타낸 값입니다.

```sh
cargo run -q -- extern-c; echo "종료 상태 $?"
```

```text
패닉 전략: unwind(되감기)
  진행률 40%

thread 'main' (스레드 번호) panicked at src/main.rs:45:9:
진행률은 0~100이어야 합니다: 140
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' (스레드 번호) panicked at /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/core/src/panicking.rs:225:5:
panic in a function that cannot unwind
stack backtrace:
   (백트레이스 수십 줄 생략)
thread caused non-unwinding panic. aborting.
종료 상태 134
```

```sh
cargo run -q -- drop-panic; echo "종료 상태 $?"
```

```text
패닉 전략: unwind(되감기)

thread 'main' (스레드 번호) panicked at src/main.rs:74:9:
업로드 중 연결이 끊겼습니다
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' (스레드 번호) panicked at src/main.rs:67:9:
임시 파일 photo.tmp을(를) 지우지 못했습니다
stack backtrace:
   (백트레이스 수십 줄 생략)

thread 'main' (스레드 번호) panicked at /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/core/src/panicking.rs:233:5:
panic in a destructor during cleanup
thread caused non-unwinding panic. aborting.
종료 상태 134
```

두 모드 모두 되감기 프로필인데도 `붙잡았나:` 줄이 나오지 않았습니다. `/rustc/` 뒤의 해시는 툴체인의 커밋이라 버전마다 다릅니다.

### projects/panic_free_lib — 린트로 지키는 약속

```sh
cd appd-a/ch18-dont-panic/projects/panic_free_lib
cargo clippy --all-targets -- -D warnings
cargo test
```

`cargo clippy`는 경고 없이 끝나고, `cargo test`는 단위 테스트 3개와 문서 테스트 1개를 통과합니다(`test result: ok. 3 passed`, `test result: ok. 1 passed`). 린트가 실제로 막는 모습은 2.6절에 옮겼습니다. 직접 `src/lib.rs`의 `parse_timer`에 `.unwrap()`이나 `v[i]`를 넣고 `cargo clippy`를 다시 실행해 보세요.

## 4. 자주 하는 실수

패닉은 실행 중의 일이라 컴파일 에러로 보일 만한 것이 많지 않습니다. 컴파일러가 미리 잡는 경우(상수 인덱스의 범위 밖 접근, 상수 산술의 넘침)는 [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md)의 "실행 중 검사" 절에서, `catch_unwind`에 `&mut`를 캡처한 E0277은 [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 4.3절에서, `const`에서 실패하는 `unwrap`의 E0080은 [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 4.5절에서 봤습니다. 이 챕터는 `compile_fail/`을 두지 않고, **컴파일은 되지만 실행 중에 패닉하는 실수**를 모았습니다. 모두 이 툴체인에서 직접 실행해 확인했습니다.

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| 라이브러리가 사용자 입력에 패닉 | 잘못된 입력 하나가 부르는 쪽 스레드를 끝냄. 부르는 쪽은 규칙을 다시 써서 미리 걸러야 함 | `Result`를 돌려줌 | 2.2, 01 |
| 사용자 입력에 `expect` | `회원 번호가 올발라야 합니다: BadMember("1203")` | `?`로 전파 | 2.2, 04 |
| 패닉판에 `#[track_caller]`를 빠뜨림 | 패닉 위치가 라이브러리 안의 `panic!` 줄 | 패닉판에 `#[track_caller]` | 2.3, 02 |
| 한글 문자열을 바이트 위치로 자름 | `end byte index 10 is not a char boundary; it is inside '과' ...` | `chars().take(n)` | 2.4, 03 |
| 개수가 0일 수 있는 평균 | `attempt to divide by zero` (두 빌드 모두) | `checked_div` | 2.4, 03 |
| 짧은 컬렉션에서 `len() - 3` | 디버그 `attempt to subtract with overflow`, 릴리스 `range start index 18446744073709551615 out of range ...` | `saturating_sub`, `iter().skip(..)` | 2.4, 03 |
| 넘침을 디버그 빌드에서만 시험 | 릴리스 빌드에서 패닉 대신 `4294967271` 같은 틀린 값 | `checked_*`, `strict_*` | 2.4, 03 |
| `RefCell`을 빌려 둔 채 `borrow_mut()` | `RefCell already borrowed` | 가드 범위 줄이기, `try_borrow_mut` | 2.4, 03 |
| 상태를 바꾼 뒤 실패할 수 있는 일을 함 | 패닉이나 `?`로 빠져나가면 상태가 어긋남(세탁기 2대) | 실패할 수 있는 일을 먼저 | 2.5, 04 |
| `catch_unwind`로 붙잡은 뒤 같은 상태를 계속 씀 | 어긋난 상태에서 다음 요청이 또 실패 | `Result`판, 또는 사본에서 실행하고 버림 | 2.5, 04 |
| 중단 프로필에서 `catch_unwind`에 기댐 | 첫 패닉에서 종료 상태 134 | 일반 에러는 `Result` | 2.5, `panic_strategy` |
| `extern "C"` 콜백에서 패닉 | `panic in a function that cannot unwind` 후 중단 | 콜백 안에서 `Result`를 에러 코드로 바꿈 | 2.5, `panic_strategy` |
| `drop`에서 패닉 | 되감기 중이면 `panic in a destructor during cleanup` 후 중단 | `drop`에서는 패닉하지 않음([ch11](../ch11-drop-raii/README.md) 2.4절) | 2.5, `panic_strategy` |

## 5. 연습 문제

### 연습 1. 사각형 채우기의 두 판

[`02_panic_flavor_pairs.rs`](examples/02_panic_flavor_pairs.rs)의 `canvas::Canvas`에 `try_fill(&mut self, x, y, w, h, ch) -> Result<(), OutOfCanvas>`와 패닉판 `fill`을 더하세요. 사각형의 일부가 캔버스 밖이면 `try_fill`은 **아무 칸도 바꾸지 않고** `Err`를 돌려줘야 합니다. `fill`에는 `# Panics` 절과 `#[track_caller]`를 붙이고, `fill(6, 2, 4, 1, '#')`의 패닉 위치가 부른 줄인지 확인하세요.

<details>
<summary>힌트</summary>

칸을 하나씩 `try_plot`으로 찍다가 실패하면, 이미 찍은 칸이 남습니다. 2.5절의 `try_start`처럼 실패할 수 있는 검사를 먼저 끝냅니다. 사각형의 오른쪽 아래 모서리 `(x + w - 1, y + h - 1)`이 캔버스 안인지 `offset`으로 확인하면 나머지 칸도 모두 안입니다. 이 계산의 `x + w - 1`도 숨은 패닉 지점입니다. `x`와 `w`가 모두 0이면 0 아래로 넘치고, `x + w`가 `usize`를 넘을 수도 있으므로 `checked_add`와 `checked_sub`로 씁니다(`w == 0`이면 바꿀 칸이 없으니 `Ok(())`). `#[track_caller]`가 붙은 `fill` 안에서 부른 `panic!`은 `fill`을 부른 줄로 보고됩니다.

</details>

### 연습 2. 검사판에 남은 함정

[`03_hidden_panics.rs`](examples/03_hidden_panics.rs)의 `summary_checked`에는 패닉하지는 않지만 틀린 값을 낼 수 있는 곳이 하나 남아 있습니다. 찾아서 고치세요.

<details>
<summary>힌트</summary>

`c.sessions.len() as u32`입니다. `as`는 넘치는 위쪽 비트를 조용히 버리므로(Part 1 ch1 2.9절), 기록이 `u32::MAX`개를 넘으면 나누는 수가 엉뚱한 값이 됩니다. 패닉도 경고도 없이 틀리는 점이 2.4절의 릴리스 빌드 오버플로와 같습니다. `u32::try_from(c.sessions.len())`으로 바꾸고 `Err`이면 "기록이 너무 많음"을 돌려줍니다. 이런 `as`는 기본으로 꺼진 clippy 린트 `cast_possible_truncation`(pedantic)이 잡습니다. `cargo clippy -p er-ch18-dont-panic --example 03_hidden_panics -- -W clippy::cast_possible_truncation`으로 켜 보면 두 판 모두에 ``casting `usize` to `u32` may truncate the value on targets with 64-bit wide pointers``가 나옵니다(이미 검사한 파일이면 먼저 `touch`로 고친 것처럼 만듭니다).

</details>

### 연습 3. 린트를 통과하는 배속 조절

[`projects/panic_free_lib`](projects/panic_free_lib)에 `pub fn scale_timer(seconds: u32, percent: u32) -> Option<u32>`를 더하세요. 타이머를 `percent`%로 늘이거나 줄인 초를 돌려주고(`scale_timer(60, 150)`은 `Some(90)`), 결과가 `u32`를 넘으면 `None`입니다. 먼저 `seconds * percent / 100`으로 쓰고 `cargo clippy --all-targets -- -D warnings`가 무엇을 잡는지 본 뒤, 통과하게 고치고 테스트를 추가하세요.

<details>
<summary>힌트</summary>

`seconds * percent`는 `arithmetic_side_effects`에 걸리고, `/ 100`은 0이 아닌 상수로 나누므로 걸리지 않습니다. `seconds.checked_mul(percent).map(|n| n / 100)`처럼 씁니다. 곱이 `u32`를 넘어도 100으로 나누면 들어오는 경우까지 살리려면 `u64`로 넓혀 계산한 뒤 `u32::try_from(n / 100).ok()`로 좁힙니다. 이때 `u64::from(seconds) * u64::from(percent)`도 여전히 걸립니다. 두 `u32`의 곱은 `u64`를 넘지 않지만 린트는 그것까지 따지지 않으므로 `checked_mul`을 쓰고 `?`로 잇습니다(이 툴체인에서 확인). 함수는 `mod tests`보다 위에 둡니다. 아래에 두면 기본으로 켜진 `items_after_test_module`이 경고합니다. 테스트에서는 `clippy.toml` 덕분에 `unwrap()`을 써도 됩니다.

</details>

### 연습 4. 중단하지 않는 임시 파일 정리

[`projects/panic_strategy`](projects/panic_strategy)의 `TempUpload`는 `drop`에서 패닉해 `drop-panic` 모드를 중단시킵니다. `drop`이 패닉하지 않도록 고쳐서, `drop-panic` 모드가 `붙잡았나: true`를 출력하고 종료 상태 0으로 끝나게 하세요. 정리 실패를 부르는 쪽이 알아야 한다면 어떻게 할지도 생각해 보세요.

<details>
<summary>힌트</summary>

`drop`에서는 실패를 `eprintln!`으로 기록만 하고 끝냅니다. 부르는 쪽이 실패를 받아야 한다면 [ch11](../ch11-drop-raii/README.md) 2.4절처럼 `fn finish(self) -> Result<(), String>` 같은 해제 메서드를 따로 두고, `drop`은 `finish`를 부르지 못한 경우의 안전망으로만 남깁니다. `std::thread::panicking()`은 지금 스레드가 되감기 중인지 알려 주므로, 되감기 중에는 정리를 건너뛰는 데 쓸 수도 있습니다. 고친 뒤에도 업로드 중의 첫 패닉은 표준 에러에 출력됩니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 18: Don't panic](https://effective-rust.com/panic.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- 표준 라이브러리 문서
  - [`std::panic::catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html): try/catch로 쓰지 말라는 설명, `extern "C"`, Notes 절(중단하는 패닉은 붙잡지 못함)
  - [`std::error`](https://doc.rust-lang.org/std/error/index.html): Converting Errors into Panics 절, Common Message Styles 절(`expect` 메시지)
  - [`Option::expect`](https://doc.rust-lang.org/std/option/enum.Option.html#method.expect), [`RefCell::try_borrow_mut`](https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.try_borrow_mut), [`slice::split_at_checked`](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_checked), [`u32::strict_sub`](https://doc.rust-lang.org/std/primitive.u32.html#method.strict_sub), [`Duration::checked_sub`](https://doc.rust-lang.org/std/time/struct.Duration.html#method.checked_sub), [`thread::Builder::spawn`](https://doc.rust-lang.org/std/thread/struct.Builder.html#method.spawn), [`Vec::try_reserve`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.try_reserve), [`PanicHookInfo::payload_as_str`](https://doc.rust-lang.org/std/panic/struct.PanicHookInfo.html#method.payload_as_str)
  - [`String::from_utf8_unchecked`](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_unchecked): Safety 절
- The Reference: [the `track_caller` attribute](https://doc.rust-lang.org/reference/attributes/codegen.html#the-track_caller-attribute)
- Rust API 가이드라인: [Function docs include error, panic, and safety considerations (C-FAILURE)](https://rust-lang.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure)
- Cargo: [The `[lints]` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section), [프로필의 `overflow-checks`](https://doc.rust-lang.org/cargo/reference/profiles.html#overflow-checks)
- Clippy: [린트 목록(1.100.0)](https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html), [Lint Configuration](https://doc.rust-lang.org/clippy/lint_configuration.html)(`allow-unwrap-in-tests` 등)
- [`no-panic`](https://docs.rs/no-panic/latest/no_panic/) 크레이트 문서
- 1.81.0 릴리스 노트의 항목 "Abort on uncaught panics in `extern "C"` functions": [rust-lang/rust#116088](https://github.com/rust-lang/rust/pull/116088/)(로컬 문서의 `releases.md`에서 확인)
