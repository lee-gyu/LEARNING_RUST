# ch16. `unsafe` 코드를 직접 쓰지 않기

[Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)에서 `unsafe`가 다섯 가지 능력(원시 포인터 역참조, unsafe 함수 호출, 가변 정적 변수 접근, unsafe 트레이트 구현, 유니언 필드 읽기)만 더한다는 것을 배웠고, `split_at_mut`을 원시 포인터로 직접 구현해 안전한 함수로 감싸 보았습니다. 이 챕터는 문법이 아니라 판단을 다룹니다. `unsafe`를 쓰고 싶어지는 순간에 무엇을 먼저 찾아보는가, 그래도 써야 한다면 어떻게 가두는가입니다. 책의 답은 제목 그대로입니다. 필요한 `unsafe` 코드는 대부분 누군가 이미 써서 시험해 두었으니 **직접 쓰지 않습니다**. 책의 집계로 표준 라이브러리에만 `alloc`에 약 1,000개, `core`에 약 1,500개, `std`에 약 2,000개의 `unsafe`가 있습니다. `Vec`, `Rc`, `Mutex`를 `unsafe` 없이 쓸 수 있는 것은 그 코드가 안전한 API 뒤에 들어 있기 때문입니다.

- 원제: [Item 16: Avoid writing unsafe code](https://effective-rust.com/unsafe.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch16-avoid-unsafe`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

> **이 챕터의 `unsafe` 코드는 Miri로 검증하지 못했습니다.** 이 환경에는 nightly 툴체인이 설치돼 있지 않습니다. 모든 `unsafe` 블록에 `// SAFETY:`로 시작하는 안전성 주석(safety comment)으로 근거를 적었고, 표준 라이브러리 문서가 전제 조건(precondition)을 분명히 적어 둔 함수(`get_unchecked`, `from_utf8_unchecked`, `read_unaligned`, `MaybeUninit::assume_init`)만 그 조건을 코드로 확인한 자리에서 부릅니다. 전제 조건을 어기면 무슨 일이 생기는지는 실행하지 않고 글로만 설명합니다.

다음 챕터의 내용을 안다고 보고 진행합니다. 문법과 API 설명은 그쪽으로 넘깁니다.

- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.1~2.7절: `unsafe`의 다섯 능력, 원시 포인터, `unsafe fn`과 `# Safety`, 안전한 추상화, `static mut`, Miri 소개
- [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.5절, 2.11절: `mem::swap`/`replace`/`take`, `OnceCell`/`LazyCell`
- [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 2.8~2.9절: 원자 타입, `OnceLock`/`LazyLock`
- [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 2.5절: C 함수 부르기와 `&CStr`을 받는 안전한 래퍼
- [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.7~2.9절, [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.5절: `unsafe extern`, unsafe 속성, `static mut` 참조 금지, `set_var`
- [ch6](../ch6-newtype-pattern/README.md) 2.3절: 생성자에서 검증하는 뉴타입
- [ch15](../ch15-borrow-checker/README.md) 2.5·2.6·2.9절: 바꿔 끼우기, 나눠 빌리기, 자기 참조 구조체의 대안

앞 챕터들이 이 챕터로 미룬 것도 받습니다. [ch8](../ch8-references-pointers/README.md)의 원시 포인터, [ch10](../ch10-standard-traits/README.md)의 `unsafe impl Send/Sync`, [ch11](../ch11-drop-raii/README.md)의 "원시 메모리를 해제하는 `Drop`", [ch15](../ch15-borrow-checker/README.md)의 "자기 참조를 `unsafe`로 구현하기"입니다. 답은 모두 "직접 쓰지 말고 이미 있는 것을 쓴다"라서 2.2절 표의 한 줄씩으로 받고, 직접 구현하는 방법은 Part 10(Rustonomicon)으로 넘깁니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `unsafe`가 규칙을 없애지 않고 지킬 책임을 프로그래머에게 옮긴다는 것과 건전성의 뜻을 설명하고, `unsafe`를 쓰고 싶어지는 상황을 유형별로 표준 라이브러리의 안전한 API와 짝지을 수 있다.
2. 경계 검사(bounds check) 건너뛰기, 바이트 재해석, 초기화하지 않은 배열을 안전한 API(`array_windows`, `as_chunks`, `from_le_bytes`, `str::from_utf8`, `array::from_fn`)로 바꾸고, 건전한 `unsafe` 판도 결국 같은 확인을 해야 한다는 것을 설명할 수 있다.
3. 피할 수 없는 `unsafe`를 비공개 필드와 검사하는 생성자를 가진 안전한 래퍼(wrapper) 한 곳에 가두고, 모듈 경계가 안전성 검토의 범위가 되는 이유를 설명하며, 래퍼에 불변식(invariant)의 경계를 겨냥한 단위 테스트를 달 수 있다.
4. `# Safety` 절과 `// SAFETY:` 주석을 쓰고, unsafe 연산 하나에 블록 하나를 두고, `unsafe_code`·`unsafe_op_in_unsafe_fn`·`missing_safety_doc`·`undocumented_unsafe_blocks`·`multiple_unsafe_ops_per_block` 린트로 그것을 강제할 수 있다.
5. Miri가 무엇을 찾아 주고 무엇을 보장하지 못하는지 설명하고, 이 챕터의 예제를 Miri로 돌리는 명령을 쓸 수 있다.

## 2. 핵심 개념

2.1절은 `unsafe`가 무엇을 바꾸는지 정리하는 **이해** 부분이고, 2.2절은 이 챕터 전체의 지도입니다. 2.3~2.7절이 지침이며, 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 씁니다. 지침 1의 비권장 코드는 "직접 쓴 `unsafe`"이고 권장 코드는 "같은 일을 하는 표준 라이브러리의 안전한 API"입니다. 지침 3·4의 비권장 코드는 "흩어진 `unsafe`"이고 권장 코드는 "한 곳에 모으고 안전성 주석을 단 `unsafe`"입니다. 책이 권하지 않은 판단은 2.8절에 "책에 없는 지침"으로 밝혔습니다.

### 2.1 `unsafe`는 규칙을 끄지 않고 책임을 옮긴다

`unsafe` 블록 안에서도 대여 검사기와 타입 검사는 그대로 동작합니다(Part 0 ch19 2.1절). 달라지는 것은 다섯 가지 연산이 허락된다는 것, 그리고 그 연산이 미정의 동작(undefined behavior)을 일으키지 않는다는 **증명을 컴파일러 대신 프로그래머가 맡는다**는 것입니다. 책이 Item을 맺으며 강조하는 점도 이것입니다. `unsafe`를 붙여도 규칙은 없어지지 않고, 규칙을 지킬 책임이 옮겨 올 뿐입니다. 컴파일러는 미정의 동작이 일어나지 않는다고 가정하고 최적화하므로, 증명이 틀리면 프로그램이 무엇을 할지 예측할 수 없습니다.

그 증명이 맞는지를 가리키는 말이 **건전성(soundness)** 입니다. 안전한 코드만으로는 미정의 동작을 일으킬 수 없다는 성질입니다. 함수 단위로 말하면 다음과 같습니다.

- **안전한 함수**(`unsafe fn`이 아닌 함수)는 어떤 인자로 불러도 미정의 동작이 없어야 건전합니다. 안에 `unsafe` 블록이 있다면, 그 블록의 전제 조건을 함수가 스스로 확인해야 합니다.
- **`unsafe fn`** 은 전제 조건을 확인하는 일을 부르는 쪽에 넘깁니다. 넘긴 조건은 문서의 `# Safety` 절에 적고, 부르는 쪽은 `unsafe` 블록과 `// SAFETY:` 주석으로 "그 조건을 지켰다"고 서명합니다.

다음 함수는 건전하지 않습니다.

```rust
fn name_of(bytes: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(bytes) }
}
```

이 함수는 컴파일되고 rustc와 Clippy 모두 아무 경고도 하지 않습니다(직접 확인). 정상적인 이름으로 시험하면 잘 동작합니다. 그런데 `name_of(b"\xFF")`라는 안전한 코드 한 줄이 올바르지 않은 `&str`을 만들어 미정의 동작이 됩니다. 이 챕터는 이런 호출을 실행하지 않습니다. 고치는 방법은 둘입니다. 함수 안에서 UTF-8을 확인하거나(그러면 `std::str::from_utf8`을 쓰면 됩니다), 함수를 `unsafe fn`으로 바꿔 확인을 부르는 쪽에 넘기는 것입니다.

증명이 `unsafe` 블록 안에만 있는 것도 아닙니다. [Rustonomicon의 Working with Unsafe](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)는 `idx < arr.len()`을 `idx <= arr.len()`으로 바꾸는, **안전한 코드만 고친** 수정이 함수를 건전하지 않게 만드는 예를 듭니다. `unsafe` 블록의 근거는 그 앞의 검사, 그 값을 만든 생성자, 그 필드를 고치는 다른 메서드에 흩어져 있습니다. 그래서 `unsafe`는 쓸수록 검토할 범위가 넓어지고, 이것이 직접 쓰지 말라는 지침(2.3·2.4절)과 쓰더라도 한 곳에 가두라는 지침(2.5절)의 이유입니다.

### 2.2 지도: 쓰고 싶어지는 `unsafe`와 먼저 찾아볼 안전한 API

책은 "`unsafe`를 감싸 둔 표준 라이브러리 기능"으로 스마트 포인터(`Rc`, `RefCell`, `Arc`), 동기화 도구(`Mutex`, `RwLock`과 가드), `Pin`, `std::sync::atomic`, `std::mem`의 `take`/`swap`/`replace`를 꼽습니다. 아래 표는 그 목록을 "무엇을 하고 싶을 때"로 다시 묶은 것입니다. 이 짝짓기는 책에 없는 정리이며(2.8절), (책)으로 표시한 칸이 책이 꼽은 기능입니다.

| 하고 싶은 일 | 손이 가는 `unsafe` | 먼저 찾아볼 안전한 API | 다루는 곳 |
| --- | --- | --- | --- |
| 경계 검사를 건너뜀 | `get_unchecked` | 반복자, `array_windows`, `as_chunks`, `chunks_exact`, `get` | 2.3절, 예제 01 |
| 바이트를 수로 읽음 | 포인터 캐스트, `transmute` | `from_le_bytes`/`from_be_bytes`, `first_chunk` | 2.3절, 예제 02 |
| 바이트를 문자열로 봄 | `str::from_utf8_unchecked` | `str::from_utf8`, `String::from_utf8`, `String::from_utf8_lossy` | 2.3절, 예제 02 |
| 원소가 `Copy`가 아닌 배열을 만듦 | `MaybeUninit` | `array::from_fn`, `[const { .. }; N]`, `array::repeat`, `Default` | 2.3절, 예제 03 |
| 한 슬라이스의 두 곳을 함께 고침 | 원시 포인터 둘 | `split_at_mut`, `get_disjoint_mut`, `swap` | [ch15](../ch15-borrow-checker/README.md) 2.6절 |
| `&mut` 뒤의 값을 꺼냄 | `ptr::read` + `ptr::write` | `mem::take`/`replace`/`swap` (책) | ch15 2.5절, Part 1 ch7 2.5절 |
| 전역 가변 상태 | `static mut` | 원자 타입(책), `Mutex`(책), `OnceLock`/`LazyLock` | Part 1 ch11 2.8~2.9절, Part 2 ch4 2.9절 |
| 여러 곳이 한 값을 소유하거나 함께 고침 | 서로 가리키는 원시 포인터 | `Rc`/`Arc`, `RefCell`/`Mutex`, `Weak` (책) | [ch8](../ch8-references-pointers/README.md) 2.6~2.7절, ch15 2.8절 |
| 자기 참조 구조체 | 자기 필드를 가리키는 원시 포인터 | 원문과 뷰 나누기, 범위로 가리키기, `Pin`(책), 자기 참조 크레이트 | ch15 2.9절 |
| 메모리를 직접 할당·해제 | 할당기 호출 + `Drop` | `Box`, `Vec` (해제는 이미 그 타입들의 `Drop`이 함) | [ch11](../ch11-drop-raii/README.md) 2.2절, Part 10 |
| `Send`가 아닌 타입을 스레드로 보냄 | `unsafe impl Send/Sync` | `Arc<Mutex<T>>`, 채널 | [`ch17-shared-state-parallelism`](../ch17-shared-state-parallelism/README.md), Part 10 |
| C 함수를 부름 | `unsafe extern` | 안전한 대안이 없음 → 2.5절의 래퍼 | Part 1 ch14 2.5절, [`ch34-ffi-boundaries`](../ch34-ffi-boundaries/README.md) |

`ptr::read`/`ptr::write` 줄에는 덧붙일 것이 있습니다. `ptr::read` 문서의 Ownership of the Returned Value 절대로, `read`는 `Copy`가 아닌 값도 비트 그대로 복사해 같은 값의 주인을 둘로 만듭니다. 그 사이에 패닉으로 빠져나가면 같은 값이 두 번 해제됩니다(문서의 `swap` 구현 예가 이 틈을 주석으로 경고합니다). `mem::replace`에는 이 틈이 없습니다. 같은 일을 하는 안전한 함수는 대개 `unsafe` 판보다 짧을 뿐 아니라 이런 틈도 막아 둔 것입니다.

### 2.3 지침 1: `unsafe`를 쓰기 전에 표준 라이브러리에서 찾는다

책이 내놓는 첫 번째 절차입니다. 특히 대여 검사기와 씨름하다 `unsafe`가 떠오를 때(ch14, ch15) 먼저 표준 라이브러리를 뒤집니다. 아래 세 사례는 이 챕터가 고른 것으로, 모두 "건전하게 쓴 `unsafe` 판"과 "표준 라이브러리 판"을 같은 입력에 실행해 비교합니다.

#### 사례 1. 경계 검사를 건너뛰고 싶을 때

**문제 상황.** 등산 앱이 1분마다 고도를 기록합니다. 기록 17개로 오르막 누적 고도와 5분 묶음 평균을 구합니다. 인덱스로 접근할 때마다 경계 검사가 붙어 느릴 것 같습니다.

**비권장 코드.** [`01_checked_access.rs`](examples/01_checked_access.rs)의 `unchecked` 모듈입니다.

```rust
for g in 0..groups {
    let mut sum = 0;
    for k in 0..GROUP {
        // SAFETY: g < len / GROUP이고 k < GROUP이므로
        // g * GROUP + k <= (len / GROUP) * GROUP - 1 <= len - 1입니다.
        sum += unsafe { *alts.get_unchecked(g * GROUP + k) };
    }
    averages.push(f64::from(sum) / GROUP as f64);
}
```

결과는 맞습니다. 그러나 건전하게 쓰려고 블록마다 인덱스가 범위 안이라는 증명을 산수로 적어야 했습니다. 이 증명은 블록 바깥의 안전한 코드(`0..groups`라는 반복 범위)에 기댑니다. 누군가 `0..=groups`로 고치면 컴파일러는 아무 말 없이 받아들이고, 범위 밖을 읽는 미정의 동작이 됩니다(실행하지 않았습니다). 또 이 판은 묶음에 들지 못한 나머지 2개를 말없이 버립니다. 3번 구역의 `altitude_at`은 더 분명합니다. 건전하게 쓰려니 `if minute < alts.len()` 검사를 직접 써야 했고, 그것은 `get`이 하는 일 그대로입니다.

**권장 코드.** 인덱스를 직접 만들지 않으면 증명할 것도 없습니다.

```rust
alts.array_windows().map(|[prev, cur]| (cur - prev).max(0)).sum()   // 오르막 누적 고도

let (groups, rest) = alts.as_chunks::<GROUP>(); // &[[i32; 5]]와 남은 원소
```

`array_windows`(1.94.0)는 겹치는 `[앞, 뒤]` 쌍을 배열로, `as_chunks`(1.88.0)는 꽉 찬 묶음과 나머지를 나눠 줍니다. 원소가 배열 타입이라 크기가 타입에 드러나고, 나머지를 버릴지는 부르는 쪽이 정합니다. 더 오래된 툴체인이라면 `windows(2)`와 `chunks_exact(5)`(1.31.0)가 같은 일을 합니다.

**검사는 실제로 남는가.** "검사가 느리다"는 짐작을 확인해 보았습니다. 같은 계산을 `alts[i]` 인덱싱으로 쓴 판과 위의 권장 판을 릴리스 수준(`-C opt-level=3`)으로 컴파일하고 `--emit asm`으로 만든 어셈블리에서 경계 검사 실패 시 부르는 `panic_bounds_check`를 찾았습니다(macOS aarch64, rustc 1.100.0).

| 판 | `panic_bounds_check` 호출 |
| --- | --- |
| 오르막 누적: `for i in 1..alts.len()`과 `alts[i - 1]`, `alts[i]` | 없음 |
| 오르막 누적: `array_windows` | 없음 |
| 5분 평균: `alts[g * GROUP + k]` | 남음 |
| 5분 평균: `as_chunks` | 없음 |

컴파일러가 검사를 지울 수 있으면 지우고, 손으로 계산한 인덱스처럼 증명하기 어려운 곳에는 남깁니다. 남은 곳도 `unsafe`가 아니라 `as_chunks` 같은 표현으로 바꾸면 사라졌습니다. 이 결과는 이 컴파일러와 이 기계에서 본 것이고 보장되지 않습니다. 남은 검사가 실제로 느린지는 측정해야 알 수 있으며, 측정하는 방법은 [`ch20-avoid-over-optimizing`](../ch20-avoid-over-optimizing/README.md)에서 다룹니다.

#### 사례 2. 바이트를 수와 문자열로 읽을 때

**문제 상황.** 블루투스 체중계가 보내는 패킷을 읽습니다. 설명서의 형식은 "0..2 측정 번호(u16 리틀 엔디언), 2..4 체중(u16 리틀 엔디언, 0.1kg 단위), 4.. 이름(UTF-8)"입니다.

**비권장 코드.** [`02_bytes_to_values.rs`](examples/02_bytes_to_values.rs)의 `pointer_cast` 모듈은 C에서 하던 대로 포인터를 `u16`으로 바꿔 읽고, 이름은 `from_utf8_unchecked`로 만듭니다.

```rust
let field = bytes.get(at..at.checked_add(2)?)?;
// SAFETY: field는 정확히 2바이트이므로 u16 하나를 읽어도 범위를 넘지 않습니다. (생략)
Some(unsafe { field.as_ptr().cast::<u16>().read_unaligned() })
```

건전하게 썼지만 문제가 셋 있습니다.

- **바이트 순서가 기계에 따라 다릅니다.** 포인터로 읽으면 이 기계의 엔디언으로 해석합니다. 확인한 기계(aarch64)는 리틀 엔디언이라 결과가 맞았지만, 빅 엔디언 기계라면 체중 바이트 `[D4, 02]`를 72.4kg이 아니라 5427.4kg으로 읽습니다(예제의 3번 구역이 `from_be_bytes`로 계산해 보여 줍니다). 형식이 엔디언을 정해 두었다면 코드에도 그 이름이 보여야 합니다.
- **한 글자 차이로 미정의 동작입니다.** `ptr::read` 문서의 Safety 절은 포인터가 그 타입에 맞게 정렬돼 있어야 하며, 아니면 `read_unaligned`를 쓰라고 합니다. 바이트 슬라이스의 임의 위치는 `u16`의 메모리 정렬(2)을 보장하지 않으므로, `read_unaligned`를 `read`로 바꾸면 정렬되지 않은 위치를 읽을 때 미정의 동작입니다. 바꿔도 컴파일은 그대로 되므로 컴파일러는 알려 주지 않습니다(실행하지 않았습니다).
- **이름 검사가 틀어집니다.** `from_utf8_unchecked`를 건전하게 부르려면 UTF-8인지 먼저 확인해야 하는데, UTF-8 검사기를 직접 쓰기는 어렵습니다. 그래서 확인하기 쉬운 ASCII로 좁혔고, 그 결과 올바른 UTF-8인 "민지"(패킷 B)도 거부합니다.

같은 일을 `transmute::<[u8; 2], u16>(raw)`로 쓰면 rustc가 기본으로 경고합니다(`unnecessary_transmutes`, 1.100.0에서 확인. 핵심 줄만 옮겼습니다).

```text
warning: unnecessary transmute
  = help: there's also `from_le_bytes` and `from_be_bytes` if you expect a particular byte order
help: replace this with
9 +     let y = unsafe { u16::from_ne_bytes(x) };
```

**권장 코드.** 엔디언을 이름에 드러내는 안전한 변환을 씁니다.

```rust
let field = bytes.get(at..)?.first_chunk::<2>()?; // &[u8; 2], 모자라면 None
Some(u16::from_le_bytes(*field))

std::str::from_utf8(bytes) // Result<&str, Utf8Error>
```

`from_le_bytes`(1.32.0)는 어느 기계에서나 같은 값을 내고, `first_chunk`(1.77.0)는 길이 확인과 배열 변환을 한 번에 합니다. `str::from_utf8`은 "민지"를 받아들이고, 깨진 패킷 C에는 ``invalid utf-8 sequence of 1 bytes from index 2``라는 위치가 담긴 에러를 돌려줍니다. 이름을 받은 뒤에도 바이트를 계속 갖고 있어야 한다면 `String::from_utf8`, 깨진 부분을 대체 문자로 바꿔도 되면 `String::from_utf8_lossy`가 있습니다. 비권장 판이 결국 ASCII 검사를 해야 했던 것처럼, **건전한 `unsafe` 판은 같은 확인을 피할 수 없습니다.** 그렇다면 표준 라이브러리가 이미 잘 만들어 둔 확인을 쓰는 편이 낫습니다.

#### 사례 3. 원소가 `Copy`가 아닌 배열을 만들 때

**문제 상황.** 우체국 분류기에는 우편번호 첫 자리(0~9)마다 칸(`Bin`)이 있고, 칸마다 이름표와 우편물 목록이 있습니다. `[Bin::empty(); 10]`은 `Bin`이 `Copy`가 아니라서 E0277입니다(4.1절).

**비권장 코드.** 이 에러를 만나면 `MaybeUninit`으로 칸을 하나씩 채우고 싶어집니다. [`03_build_arrays.rs`](examples/03_build_arrays.rs)의 `uninit` 모듈은 `MaybeUninit` 문서의 "배열을 원소 하나씩 초기화하기" 예를 따랐습니다.

```rust
let mut bins: [MaybeUninit<Bin>; BINS] = [const { MaybeUninit::uninit() }; BINS];
for (digit, slot) in bins.iter_mut().enumerate() {
    slot.write(Bin::new(digit));
}
// SAFETY: 위 반복문이 BINS칸 모두에 write를 한 번씩 했으므로 모든 칸이 초기화돼 있습니다.
unsafe { MaybeUninit::<[Bin; BINS]>::from(bins).assume_init() }
```

초기화가 실패하지 않으면 잘 동작합니다. 문제는 실패할 수 있을 때입니다. 담당 집배원 목록으로 칸을 만들다가 4번 칸에서 `?`로 반환하면, 이미 채운 0~3번 칸은 아무도 해제하지 않습니다. `MaybeUninit`은 버려질 때 안의 값을 해제하지 않기 때문입니다. 예제는 살아 있는 칸의 수를 세어 이 누수를 보여 줍니다(비권장 판 "새어 나간 칸 4개"). 누수는 미정의 동작이 아니지만, 막으려면 채운 칸까지만 골라 `assume_init_drop`을 부르는 `unsafe` 코드를 더 써야 합니다(`MaybeUninit` 문서의 두 번째 예).

**권장 코드.** 컴파일러가 E0277의 `help:`로 이미 알려 줍니다.

```rust
std::array::from_fn(Bin::new) // 인덱스 0..10마다 Bin::new(i)

// 실패할 수 있는 초기화: Vec으로 모은 뒤 배열로
let bins: Vec<Bin> = carriers.iter().enumerate()
    .map(|(digit, name)| Bin::with_carrier(digit, name))
    .collect::<Result<_, _>>()?;   // 실패하면 이미 만든 칸은 Vec과 함께 해제됨
bins.try_into().map_err(|v: Vec<Bin>| format!("칸이 {}개뿐", v.len()))
```

권장 판은 같은 실패에서 "새어 나간 칸 0개"입니다. 상황에 따라 다음도 씁니다.

| 만들려는 배열 | 안전한 방법 | 비고 |
| --- | --- | --- |
| 인덱스마다 다른 값 | `std::array::from_fn(\|i\| ..)` (1.63.0) | E0277의 `help:`가 제안 |
| 상수로 만들 수 있는 같은 값(`Vec::new()` 등) | `[const { Vec::new() }; N]` | `[Vec::new(); N]`의 E0277에 `help:`가 이 모양을 제안 |
| `Clone`인 같은 값 | `std::array::repeat(v)` (1.91.0) | |
| `Default`인 값 | `Default::default()` | 길이 32까지. `[Vec<String>; 40]`은 E0277 |
| 실패할 수 있는 초기화 | `Vec`으로 모은 뒤 `try_into()` | `array::try_from_fn`은 1.100.0에서 nightly 전용(E0658) |

**지침을 따르지 않아도 되는 경우.**

- 측정해 보니 남은 검사가 실제 병목이고, 안전한 표현으로 바꿔도 검사가 사라지지 않을 때입니다. 그때도 2.5~2.7절의 방법으로 가둡니다(측정은 [`ch20-avoid-over-optimizing`](../ch20-avoid-over-optimizing/README.md)).
- 일부만 초기화된 버퍼처럼 표준 라이브러리 수준의 저수준 자료구조를 직접 만들 때입니다. `Vec`을 직접 구현하는 일은 Part 10에서 다룹니다.
- C 함수처럼 안전한 대안이 원리적으로 없을 때입니다(2.5절).

### 2.4 지침 2: 표준 라이브러리에 없으면 검증된 크레이트를 찾는다

**문제 상황.** 난수, 구조체를 바이트로 통째로 보기, C++ 연동처럼 표준 라이브러리에 없는 기능이 필요합니다.

**비권장 코드.** 운영체제의 난수 함수를 `unsafe extern`으로 직접 선언해 부르거나, 구조체를 `transmute`로 바이트 배열로 바꾸는 코드를 직접 씁니다. 책은 이런 문제가 나에게만 있을 가능성은 낮다고 봅니다. 누군가 이미 풀어서 공개해 두었을 가능성이 큽니다.

**권장 코드.** crates.io에서 그 `unsafe`를 감싸 둔 크레이트를 찾습니다. 책이 예로 든 네 크레이트의 지금 상태는 다음과 같습니다.

| 크레이트 | 하는 일 | 1.100.0에서 |
| --- | --- | --- |
| `once_cell` | 한 번만 초기화하는 값 | 표준에 들어왔습니다. `OnceCell`/`OnceLock`(1.70.0), `LazyCell`/`LazyLock`(1.80.0). 새 코드는 표준을 씁니다(Part 1 ch7 2.11절, ch11 2.9절) |
| `byteorder` | 바이트와 수의 변환 | 정수 하나는 `from_le_bytes`/`to_be_bytes` 등(1.32.0)으로 충분합니다(사례 2). 리더·라이터에 바로 붙는 읽기·쓰기 메서드가 필요하면 크레이트를 씁니다 |
| `rand` | 난수 | 표준의 `std::random`은 1.100.0에서 nightly 전용(E0658)이라 여전히 크레이트가 필요합니다 |
| `cxx` | C++와 Rust 연동 | [`ch35-bindgen`](../ch35-bindgen/README.md)에서 소개합니다 |

책에 없는 예를 하나 더 들면, 구조체와 바이트를 안전하게 바꿔 보는 `bytemuck`과 `zerocopy`가 있습니다. 둘 다 "이 타입은 어떤 바이트 배열이든 올바른 값"이라는 조건을 트레이트로 확인한 뒤에만 변환을 허락합니다(2026-11-19에 crates.io에서 각각 1.25.2, 0.8.63을 확인했고 이 저장소에서는 써 보지 않았습니다).

크레이트의 `unsafe`도 누군가 쓴 `unsafe`입니다. ch15 2.9절에서 본 것처럼 널리 쓰이는 크레이트에도 건전성 문제가 보고된 적이 있으므로, 고를 때 RustSec 권고 데이터베이스와 유지 상태를 확인합니다. 의존성을 고르는 기준은 [ch25](../ch25-dependency-graph/README.md)에서 다룹니다.

**지침을 따르지 않아도 되는 경우.** 필요한 것이 작고 안전한 코드로 쓸 수 있다면, 의존성을 늘리는 비용이 더 클 수 있습니다. 크레이트의 기능이 표준에 들어왔다면(`once_cell`) 표준으로 옮깁니다.

### 2.5 지침 3: 피할 수 없는 `unsafe`는 안전한 래퍼 한 곳에 가둔다

책은 FFI처럼 `unsafe`를 피할 수 없을 때 **모든 `unsafe` 코드를 담는 래퍼 계층**을 두라고 권합니다. 나머지 코드는 래퍼의 안전한 API만 쓰게 하고, 문제가 생기면 볼 곳을 래퍼로 좁히는 것입니다. 이 챕터는 FFI를 실행하지 않으므로([`ch34-ffi-boundaries`](../ch34-ffi-boundaries/README.md)), 표준 라이브러리의 `String`이 쓰는 설계를 작게 따라 해 봅니다. 1.100.0 소스에서 `String`은 `Vec<u8>` 필드 하나를 감싼 구조체이고, `String::from_utf8`이 만들 때 한 번 UTF-8을 확인하며, `as_str`은 ``// SAFETY: String contents are stipulated to be valid UTF-8``라는 주석과 함께 `from_utf8_unchecked`를 부릅니다.

**문제 상황.** 염기 서열 파일에서 읽은 바이트(A, C, G, T)를 화면에 보여 주고, 패턴을 찾고, 상보 서열(A↔T, C↔G)을 만듭니다. A/C/G/T만 있다는 것을 확인했다면 ASCII이므로 검사 없이 `&str`로 볼 수 있습니다.

**비권장 코드.** [`04_safe_wrapper.rs`](examples/04_safe_wrapper.rs)의 `scattered` 모듈은 바이트를 `Vec<u8>` 그대로 주고받고, `unsafe fn as_text(bytes: &[u8]) -> &str`(`# Safety`: A/C/G/T만 있어야 함)을 열어 둡니다. 부르는 곳마다 이렇게 됩니다.

```rust
if let Some(at) = first_non_base(&bytes) { /* 건너뜀 */ }
// SAFETY: 바로 위의 first_non_base가 None이었으므로 A/C/G/T만 있습니다.
let shown = unsafe { scattered::as_text(&bytes) }.to_string();
```

`Vec<u8>`라는 타입은 "확인했음"을 기억하지 못합니다. 보여 주기, 찾기, 상보 서열이 서로 다른 함수에 있다면 받은 바이트가 확인된 것인지, 그 뒤에 누가 고치지 않았는지 알 수 없어 함수마다 다시 확인하고 `unsafe` 블록을 씁니다. 예제에서 샘플 세 개에 확인이 7번, 부르는 곳의 `unsafe` 블록이 3개입니다. 하나라도 확인을 빠뜨리면 미정의 동작이고, 검토할 곳은 바이트를 만지는 프로그램 전체입니다.

**권장 코드.** `dna` 모듈의 `DnaSeq`가 불변식("`bases`에는 A/C/G/T만 있다")을 지킵니다.

```rust
pub struct DnaSeq {
    bases: Vec<u8>, // 비공개: 모듈 밖에서는 만들 수도, 고칠 수도 없습니다.
}

impl DnaSeq {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<DnaSeq, InvalidBase> { /* 여기서 한 번 확인 */ }

    pub fn as_str(&self) -> &str {
        // SAFETY: 불변식에 따라 bases에는 A/C/G/T만 있습니다. 그런 바이트열은 ASCII이고,
        // ASCII 바이트열은 올바른 UTF-8입니다.
        unsafe { std::str::from_utf8_unchecked(&self.bases) }
    }

    pub fn complement(&mut self) { /* A↔T, C↔G로만 바꾸므로 불변식 유지 */ }
}
```

같은 샘플에 확인은 만들 때의 3번뿐이고, `unsafe` 블록은 `as_str` 안의 1개입니다. 부르는 쪽(`wrapper_demo`)에는 `unsafe`가 하나도 없습니다. 설계에서 볼 점은 다섯 가지입니다.

- **불변식을 세우는 길은 생성자 하나입니다.** [ch6](../ch6-newtype-pattern/README.md) 2.3절의 검증 뉴타입과 같은 모양입니다. ch6에서는 틀린 값을 막았다면, 여기서는 미정의 동작을 막습니다.
- **비공개 필드가 불변식을 지킵니다.** 모듈 밖에서 `DnaSeq { bases: .. }`로 직접 만들면 E0451, `seq.bases.push(0xFF)`로 고치면 E0616입니다(4.3절). 안전한 코드만으로는 불변식을 깰 수 없다는 것을 컴파일러가 가시성 규칙으로 보장합니다. 그래서 `as_str`은 `unsafe fn`이 아니어도 건전합니다.
- **안전성 검토의 범위는 모듈입니다.** `as_str`의 블록은 한 줄이지만 그 근거는 `bases`를 고치는 모든 코드(`from_bytes`, `complement`)에 있습니다. `complement`가 실수로 0xFF를 쓰면 `unsafe`를 한 글자도 고치지 않았는데 미정의 동작이 됩니다. Rustonomicon도 "`unsafe`의 범위를 확실히 제한하는 방법은 사실상 가시성을 이용한 모듈 경계뿐"이라고 설명합니다. 래퍼 모듈은 작게 두고, 모듈 문서에 불변식을 적습니다. 모듈이 밖의 함수를 불러 불변식을 세우거나 유지한다면 그 함수도 검토 범위에 듭니다. 예제 04의 `dna` 모듈은 두 판을 비교하려고 `first_non_base`와 `complement_base`를 모듈 밖에 두고 함께 쓰므로 이 두 함수까지가 범위이고, 실제 코드라면 모듈 안에 둡니다.
- **불변식을 깰 수 있는 메서드는 `unsafe fn`입니다.** 검사 없이 만드는 생성자나 안쪽 바이트를 `&mut`로 내주는 메서드가 그렇습니다. `String`도 `from_utf8_unchecked(bytes: Vec<u8>)`와 `as_mut_vec(&mut self)`를 `unsafe fn`으로 둡니다(1.100.0 소스로 확인).
- **린트로 경계를 표시합니다.** 예제 파일 첫 줄의 `#![deny(unsafe_code)]`는 파일 전체에서 `unsafe`를 금지하고, `mod dna`에 붙인 `#[allow(unsafe_code, reason = "..")]`만 예외로 둡니다. 비권장 판의 `#[expect(unsafe_code, ..)]`를 지우면 rustc가 ``declaration of an `unsafe` function``과 ``usage of an `unsafe` block``을 에러로 냅니다. 금지를 `deny`가 아니라 `forbid`로 하면 아래에서 다시 허락할 수 없습니다(E0453, 4.2절). 패키지 전체에 걸려면 `Cargo.toml`의 `[lints.rust]`에 `unsafe_code = "forbid"`를 적습니다(Cargo 문서의 예와 같은 모양. 빈 `[workspace]` 테이블을 넣은 임시 프로젝트에서 ``requested on the command line with `-F unsafe-code` ``라는 노트가 붙은 에러로 동작을 확인했습니다).

**지침을 따르지 않아도 되는 경우.** `unsafe`가 함수 하나 안에서 끝나고 그 전제 조건도 같은 함수 안에서 확인된다면, 그 함수가 이미 래퍼입니다. 타입을 따로 만들 필요는 없습니다(예제 05의 `split_fields`). 반대로 `unsafe`가 필요한 부분이 크면 그 부분을 별도 크레이트로 떼고, 나머지 크레이트에 `forbid`를 거는 방법도 있습니다(연습 3).

### 2.6 지침 4: 전제 조건을 적고, 블록을 작게 만들고, 린트로 강제한다

책은 `unsafe`가 필요할 때 지킬 것으로 안전성 주석(Clippy `missing_safety_doc`), 작은 `unsafe` 블록, `unsafe_op_in_unsafe_fn` 린트를 켜는 것을 꼽습니다.

**문제 상황.** 기차표 예매 기록은 한 줄이 25바이트인 고정 폭 ASCII 형식입니다(0..3 열차 종류, 3..7 번호, 7..14 출발역, 14..21 도착역, 21..25 시각). 줄에서 칸 하나를 잘라 `&str`로 만드는 `unsafe fn`을 씁니다.

**비권장 코드.** [`05_safety_comments.rs`](examples/05_safety_comments.rs)의 두 판입니다.

```rust
pub unsafe fn field_2021(line: &[u8], range: Range<usize>) -> &str {
    std::str::from_utf8_unchecked(line.get_unchecked(range))
}

/// # Safety
///
/// `range`가 줄 안에 있고 그 부분이 올바른 UTF-8이어야 합니다.
pub unsafe fn field_one_block(line: &[u8], range: Range<usize>) -> &str {
    // SAFETY: 부르는 쪽이 보장합니다.
    unsafe { std::str::from_utf8_unchecked(line.get_unchecked(range)) }
}
```

`field_2021`은 2021 에디션까지의 습관입니다. 그때는 `unsafe fn`의 본문 전체가 `unsafe` 블록처럼 취급됐습니다. 2024 에디션에서는 rustc가 연산마다 경고하고(같은 파일을 `--edition 2021`로 컴파일하면 경고가 없음을 확인), Clippy가 문서의 빈자리를 지적합니다. 예제의 `#[expect]`를 지운 결과의 핵심 줄입니다.

```text
warning[E0133]: call to unsafe function `std::str::from_utf8_unchecked` is unsafe and requires unsafe block
note: an unsafe function restricts its caller, but its body is safe by default
   = note: `#[warn(unsafe_op_in_unsafe_fn)]` (part of `#[warn(rust_2024_compatibility)]`) on by default
warning[E0133]: call to unsafe function `core::slice::<impl [T]>::get_unchecked` is unsafe and requires unsafe block

warning: unsafe function's docs are missing a `# Safety` section
```

`field_one_block`은 블록을 쓰긴 했지만 전제 조건이 다른 연산 둘을 한 블록에 넣고, 주석은 아무것도 증명하지 않습니다. 이 파일이 켠 `multiple_unsafe_ops_per_block`이 잡습니다.

```text
warning: this `unsafe` block contains 2 unsafe operations, expected only one
note: unsafe function call occurs here
note: unsafe method call occurs here
```

**권장 코드.** 문서에는 부르는 쪽이 지킬 조건을 번호를 붙여 적고, 연산마다 블록 하나와 그 조건을 가리키는 주석을 둡니다.

```rust
/// # Safety
///
/// 1. `range.start <= range.end <= line.len()`이어야 합니다.
/// 2. `line[range]`가 올바른 UTF-8이어야 합니다. 줄 전체가 UTF-8이어도 칸 경계가 글자 중간을
///    자르면 그 칸은 UTF-8이 아니므로, 줄 전체가 ASCII인지 확인하는 편이 쉽습니다.
pub unsafe fn field(line: &[u8], range: Range<usize>) -> &str {
    // SAFETY: 조건 1에 따라 range는 line 안에 있습니다.
    let bytes = unsafe { line.get_unchecked(range) };
    // SAFETY: 조건 2에 따라 bytes는 올바른 UTF-8입니다.
    unsafe { std::str::from_utf8_unchecked(bytes) }
}
```

부르는 쪽(`split_fields`)은 길이와 ASCII를 한 번 확인한 뒤, 블록의 주석에 **어느 확인이 어느 조건을 채우는지** 적습니다. 좋은 안전성 주석은 "안전하다"는 선언이 아니라 "조건 N이 이 줄의 확인 때문에 성립한다"는 근거입니다. 조건 2의 단서도 그런 근거를 따지다 나온 것입니다. 줄 전체를 `str::from_utf8`로 확인하는 것으로는 부족하고, 4번 줄처럼 한글이 섞인 줄은 칸 경계에 따라 칸이 UTF-8이 아닐 수 있습니다.

**린트가 확인하는 것과 확인하지 못하는 것.** 기본으로 꺼진 `undocumented_unsafe_blocks`를 켜면 주석 없는 블록에 ``unsafe block missing a safety comment``가 납니다(`field_one_block`의 주석을 지워 확인). 하지만 이 린트는 주석이 **있는지**만 봅니다. `// SAFETY: 부르는 쪽이 보장합니다.`도 통과합니다. 내용이 맞는지는 리뷰가 확인해야 합니다. 1.100.0의 기본 수준은 `rustc -W help`, `clippy-driver -W help`와 직접 컴파일로 확인했습니다.

| 린트 | 도구 | 기본 수준 | 잡는 것 |
| --- | --- | --- | --- |
| `unsafe_code` | rustc | allow | `unsafe` 블록, `unsafe fn` 선언 등. `deny`/`forbid`로 켬 |
| `unsafe_op_in_unsafe_fn` | rustc | 2021: allow, 2024: warn | `unsafe fn` 본문의 블록 없는 unsafe 연산 |
| `unused_unsafe` | rustc | warn | 필요 없는 `unsafe` 블록 |
| `unnecessary_transmutes` | rustc | warn | 다른 함수로 바꿀 수 있는 `transmute`(사례 2) |
| `invalid_from_utf8_unchecked` | rustc | deny | 올바르지 않은 UTF-8 **리터럴**을 `from_utf8_unchecked`에 넘김 |
| `missing_safety_doc` | Clippy | warn | `# Safety` 절이 없는 `pub unsafe fn` |
| `not_unsafe_ptr_arg_deref` | Clippy | deny | 원시 포인터 인자를 역참조하면서 `unsafe`가 아닌 공개 함수 |
| `undocumented_unsafe_blocks` | Clippy | allow (restriction) | 안전성 주석이 없는 `unsafe` 블록 |
| `multiple_unsafe_ops_per_block` | Clippy | allow (restriction) | unsafe 연산이 둘 이상인 블록 |

꺼진 린트는 파일 첫머리의 `#![warn(clippy::undocumented_unsafe_blocks)]`나 `Cargo.toml`의 `[lints.clippy]`에 `undocumented_unsafe_blocks = "warn"`으로 켭니다. 린트 수준을 고르는 기준은 [`ch29-listen-to-clippy`](../ch29-listen-to-clippy/README.md)에서, `[lints]` 테이블은 Part 3 `ch3-manifest-targets`에서 더 다룹니다.

**지침을 따르지 않아도 되는 경우.** 두 연산이 정확히 같은 전제 조건 하나에 기대고, 떼어 놓으면 오히려 읽기 어렵다면 한 블록에 두고 주석이 두 연산을 모두 설명하게 합니다. restriction 그룹의 린트는 모든 프로젝트에 켜라는 것이 아니라, `unsafe`가 있는 크레이트에서 골라 켜라는 것입니다.

### 2.7 지침 5: 테스트를 더 쓰고, Miri로 돌리고, 스레드를 따져 본다

책은 `unsafe`가 있는 코드에 평소보다 테스트를 더 쓰고, 추가 진단 도구, 특히 Miri를 돌리고, 여러 스레드에서의 쓰임을 신중히 생각하라고 권합니다.

**문제 상황.** 래퍼의 `unsafe` 블록은 하나여도, 그 블록의 건전성은 생성자와 모든 메서드가 불변식을 지키는지에 달려 있습니다(2.5절). 컴파일러는 이것을 확인하지 않습니다.

**비권장 코드.** 정상 입력 몇 개로 결과만 확인하는 테스트입니다. 2.1절의 `name_of`도 정상 입력으로는 통과합니다.

**권장 코드.** 예제 04의 `dna` 모듈 안에 `#[cfg(test)]` 테스트 다섯 개를 두었습니다. 겨냥하는 것은 불변식의 경계입니다.

- 거부해야 할 입력: 소문자 `acgt`, RNA의 `U`, 빈칸, 0xFF, 한글 바이트(`accepts_only_upper_acgt`)
- 에러가 가리키는 위치(`reports_first_invalid_position`)
- 빈 입력(`empty_sequence_is_valid`)
- `as_str`이 바이트를 그대로 보여 주는지(`as_str_shows_same_bytes`)
- 바꾸는 메서드가 불변식을 유지하는지: 상보 서열을 다시 검사해도 통과하고, 두 번 하면 원래대로(`complement_keeps_invariant`)

```sh
cargo test -p er-ch16-avoid-unsafe --example 04_safe_wrapper
```

**Miri.** 테스트는 "결과가 맞는가"를 보고, Miri는 "실행하는 동안 미정의 동작이 있었는가"를 봅니다. [Miri README](https://github.com/rust-lang/miri)에 따르면 Miri는 Rust의 미정의 동작을 찾는 도구로, 범위 밖 메모리 접근과 해제 후 사용, 초기화되지 않은 데이터의 잘못된 사용, 정렬되지 않은 메모리 접근, 데이터 경쟁, 에일리어싱 규칙 위반(실험적 검사)을 찾고, 실행이 끝날 때 남은 메모리 누수도 보고합니다. README가 안내하는 명령은 다음과 같습니다(nightly 툴체인이 필요합니다).

```sh
rustup +nightly component add miri   # 설치
cargo +nightly miri test             # 테스트를 Miri로 실행
cargo +nightly miri run              # 바이너리를 Miri로 실행
```

README는 `cargo miri run/test`가 `cargo run/test`와 같은 인자를 받는다고 설명하므로, 이 챕터의 예제는 `cargo +nightly miri test -p er-ch16-avoid-unsafe --example 04_safe_wrapper`나 `cargo +nightly miri run -p er-ch16-avoid-unsafe --example 03_build_arrays`로 돌릴 수 있을 것입니다. **이 저장소에서는 실행하지 못했습니다.** README의 설명대로라면 예제 03의 2번 구역이 일부러 낸 누수가 보고될 것으로 보이지만 확인하지 않았습니다.

README가 밝히는 한계도 알아 둡니다. Miri는 프로그램의 여러 가능한 실행 중 하나만 시험하므로, 다른 실행에서만 드러나는 버그는 놓칠 수 있습니다. Miri가 미정의 동작을 찾으면 코드는 확실히 건전하지 않지만, 아무것도 찾지 못했다고 건전하다는 증명이 되지는 않습니다. 네트워크 같은 플랫폼 API는 지원이 제한됩니다. 그래서 Miri는 테스트를 대신하지 않고, 테스트가 실행하는 경로를 더 엄격하게 확인해 줍니다. 불변식의 경계를 겨냥한 테스트가 많을수록 Miri가 볼 경로도 많아집니다. 퍼즈 테스트 같은 다른 도구는 [`ch30-beyond-unit-tests`](../ch30-beyond-unit-tests/README.md)와 [`ch31-tooling-ecosystem`](../ch31-tooling-ecosystem/README.md)에서 다룹니다.

**스레드.** 여러 스레드가 함께 쓰는 값에서는 `unsafe` 블록의 전제 조건이 "다른 스레드가 그 사이에 고치지 않는다"까지 늘어납니다. `static mut`이 위험한 이유가 이것이고(Part 0 ch19 2.5절), 이 챕터의 예제가 횟수를 셀 때 `static` 원자 타입을 쓴 이유이기도 합니다. 원시 포인터를 담은 타입에 `unsafe impl Send/Sync`를 붙이는 일은 그 타입이 스레드 사이에서 안전하다는 증명을 떠맡는 일입니다. 스레드와 공유 상태는 [`ch17-shared-state-parallelism`](../ch17-shared-state-parallelism/README.md)에서, 그 증명을 직접 하는 방법은 Part 10에서 다룹니다.

**지침을 따르지 않아도 되는 경우.** 표준 라이브러리와 크레이트가 건전하다면 미정의 동작은 `unsafe` 코드에서만 생기므로, 직접 쓴 `unsafe`가 없는 크레이트에서 Miri가 찾을 것은 거의 없습니다. 그래도 테스트는 언제나 씁니다.

### 2.8 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **`unsafe_op_in_unsafe_fn`이 2024 에디션에서 기본 경고입니다.** 책은 이 린트를 켜는 것을 고려하라고 권하며 rustc의 allowed-by-default 목록을 가리킵니다. 1.100.0의 `rustc -W help`도 여전히 allow로 표시하지만, 2024 에디션에서는 `rust_2024_compatibility` 그룹이 warn으로 올립니다(같은 파일을 두 에디션으로 컴파일해 확인, 2.6절). 2024 에디션의 다른 `unsafe` 변화(`unsafe extern`, `#[unsafe(no_mangle)]`, `static mut` 참조 금지, `set_var`)는 [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md)와 [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md)가 다뤘습니다.
- **`once_cell`의 기능이 표준에 들어왔습니다.** `OnceCell`/`OnceLock`은 1.70.0, `LazyCell`/`LazyLock`은 1.80.0에서 안정화됐습니다(로컬 문서의 안정화 버전 표시로 확인).
- **`byteorder`가 하던 일의 상당 부분은 책 이전부터 표준에 있었습니다.** 정수의 `from_le_bytes` 등은 1.32.0부터 안정입니다. 책은 이것을 언급하지 않습니다.
- **책 이후에 안정화된 안전한 대안이 있습니다.** `first_chunk`(1.77.0), `split_at_checked`(1.80.0), `get_disjoint_mut`(1.86.0), `as_chunks`(1.88.0), `array::repeat`(1.91.0), `array_windows`(1.94.0), `[MaybeUninit<T>; N]`과 `MaybeUninit<[T; N]>` 사이의 `From` 변환(1.95.0. 예제 03이 `transmute` 대신 씀)입니다. 모두 로컬 문서의 안정화 버전 표시로 확인했습니다. 실패할 수 있는 `array::try_from_fn`과 `std::random`은 아직 nightly 전용입니다.
- **책에 없는 린트가 있습니다.** rustc의 `unnecessary_transmutes`(warn)와 `invalid_from_utf8_unchecked`(deny)가 1.100.0에서 기본으로 켜져 있습니다. 패키지 단위로 린트 수준을 정하는 Cargo의 `[lints]` 테이블은 Cargo 문서에 "MSRV: Respected as of 1.74"로 적혀 있습니다.
- **책에 없는 지침.** 다음은 책의 Item 16이 명시적으로 권하지 않는 내용입니다(2026-11-19에 온라인판에서 확인). 책은 표준 라이브러리와 크레이트에서 찾기, 래퍼 계층, 안전성 주석과 `missing_safety_doc`, 작은 블록과 `unsafe_op_in_unsafe_fn`, 테스트, Miri, 스레드까지 다룹니다.
  - 2.2절의 짝짓기 표 전체와 2.3절의 세 사례(경계 검사, 바이트 재해석, 배열 초기화)에서 고른 구체적인 API: 표준 라이브러리 문서, 컴파일러의 `help:`(E0277의 `array::from_fn` 제안, `unnecessary_transmutes`의 `from_le_bytes` 제안), 직접 실행한 결과가 근거입니다.
  - 2.3절의 "검사가 실제로 남는지 확인하고 측정한 뒤에 판단한다": `--emit asm`으로 본 결과가 근거입니다. 측정 자체는 책의 Item 20([`ch20-avoid-over-optimizing`](../ch20-avoid-over-optimizing/README.md))이 다룹니다.
  - 2.4절의 `bytemuck`·`zerocopy`와 RustSec 확인: crates.io와 ch15가 근거입니다.
  - 2.5절의 "모듈이 안전성 검토의 범위", "불변식을 깰 수 있는 메서드는 `unsafe fn`", `#![deny(unsafe_code)]`와 모듈 단위 허락, `forbid`와 `[lints]`: Rustonomicon의 Working with Unsafe, 1.100.0의 `String` 소스, Cargo 문서의 `[lints]` 절, 직접 컴파일한 결과(E0453)가 근거입니다.
  - 2.6절의 `undocumented_unsafe_blocks`·`multiple_unsafe_ops_per_block`, "린트는 주석의 내용을 확인하지 않는다": Clippy 1.100.0의 린트 목록과 직접 컴파일한 결과가 근거입니다.
- **달라지지 않은 점.** `unsafe`가 규칙을 없애지 않는다는 결론, Miri가 nightly 툴체인의 구성 요소라는 점(README의 설치 명령은 Part 0 ch19와 같음), 표준 라이브러리에 난수가 없다는 점은 그대로입니다.

### 2.9 기억할 것

- `unsafe`는 규칙을 끄지 않고 증명할 책임을 옮깁니다. 안전한 코드만으로 미정의 동작을 일으킬 수 없어야 건전합니다.
- `unsafe`를 쓰고 싶으면 먼저 표준 라이브러리를, 다음으로 검증된 크레이트를 찾습니다. 건전한 `unsafe` 판은 대개 표준 API와 같은 확인을 직접 하게 됩니다.
- 피할 수 없으면 비공개 필드와 검사하는 생성자를 가진 래퍼 하나에 가둡니다. 검토할 범위는 그 모듈 전체이므로 모듈을 작게 둡니다.
- `# Safety`에 조건을 번호로 적고, 연산 하나에 블록 하나, 블록마다 "어느 확인이 어느 조건을 채우는지" 적습니다. `unsafe_code`와 Clippy 린트로 강제합니다.
- 불변식의 경계를 겨냥한 테스트를 쓰고 Miri로 돌립니다. Miri가 조용하다고 건전하다는 증명은 아닙니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행하며, 모두 종료 코드 0으로 끝나고 출력은 실행마다 같습니다(예제마다 25번 실행해 확인). **예제의 `unsafe` 블록은 Miri로 검증하지 못했습니다**(nightly 툴체인을 설치하지 않음). 미정의 동작이 되는 입력은 어떤 예제도 `unsafe` 코드에 넘기지 않습니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 절 |
| --- | --- | --- | --- | --- |
| [`01_checked_access.rs`](examples/01_checked_access.rs) | 1 | `get_unchecked` | `array_windows`, `as_chunks`, `get` | 2.3 |
| [`02_bytes_to_values.rs`](examples/02_bytes_to_values.rs) | 1 | 포인터 캐스트 + `read_unaligned`, ASCII 확인 + `from_utf8_unchecked` | `first_chunk` + `from_le_bytes`, `str::from_utf8` | 2.3 |
| [`03_build_arrays.rs`](examples/03_build_arrays.rs) | 1 | `MaybeUninit` + `assume_init` | `array::from_fn`, `Vec` + `try_into` | 2.3 |
| [`04_safe_wrapper.rs`](examples/04_safe_wrapper.rs) | 3, 5 | 부르는 곳마다 확인 + `unsafe` | 래퍼 `DnaSeq`, `deny(unsafe_code)`, 단위 테스트 | 2.5, 2.7 |
| [`05_safety_comments.rs`](examples/05_safety_comments.rs) | 4 | 블록 없는 `unsafe fn`, 연산 둘을 담은 블록 | `# Safety` + 연산마다 블록과 주석 | 2.6 |

### 01_checked_access — 경계 검사 건너뛰기

고도 기록 17개로 오르막 누적 고도, 5분 묶음 평균, 고른 시각의 고도를 두 판으로 계산합니다. 2번 구역의 비권장 판은 나머지 2개를 말없이 버리고, `as_chunks`는 나머지를 따로 돌려줍니다.

```sh
cargo run -p er-ch16-avoid-unsafe --example 01_checked_access
```

```text
기록 17개 (1분 간격)
1. 오르막 누적 고도
    비권장 get_unchecked: 109m
    권장 array_windows:   109m
2. 5분 묶음 평균
    비권장 get_unchecked: [322.0, 356.8, 389.6]
    권장 as_chunks      : [322.0, 356.8, 389.6]
    as_chunks가 따로 돌려준 나머지: [401, 398]
3. 고른 시각의 고도
    3분 → 비권장 Some(321), 권장 Some(321)
    16분 → 비권장 Some(398), 권장 Some(398)
    20분 → 비권장 None, 권장 None
```

### 02_bytes_to_values — 바이트를 수와 문자열로

체중계 패킷 네 개(A: 영문 이름, B: 한글 이름, C: 이름에 0xFF, D: 1바이트)를 두 판으로 읽습니다. 3번 구역의 첫 줄은 **실행하는 기계에 따라 다릅니다.** 확인한 기계(aarch64 macOS)는 `little`입니다.

```sh
cargo run -p er-ch16-avoid-unsafe --example 02_bytes_to_values
```

```text
1. 비권장: 포인터로 읽기 + ASCII 확인 뒤 from_utf8_unchecked
    패킷 A → #513, 72.4kg, 이름 "minji"
    패킷 B → #514, 65.5kg, 이름을 읽지 못함(ASCII가 아님)
    패킷 C → #515, 80.1kg, 이름을 읽지 못함(ASCII가 아님)
    패킷 D → 너무 짧음
2. 권장: from_le_bytes + str::from_utf8
    패킷 A → #513, 72.4kg, 이름 "minji"
    패킷 B → #514, 65.5kg, 이름 "민지"
    패킷 C → #515, 80.1kg, 이름 오류: invalid utf-8 sequence of 1 bytes from index 2
    패킷 D → 너무 짧음
3. 바이트 순서
    이 기계의 바이트 순서: little
    체중 바이트 [D4, 02]: from_le_bytes → 724, from_be_bytes → 54274
    빅 엔디언 기계라면 포인터 판은 72.4kg 대신 5427.4kg을 읽습니다
```

### 03_build_arrays — 원소가 `Copy`가 아닌 배열

1번 구역은 칸 10개를 두 판으로 만들고 우편물 6통을 분류합니다. 2번 구역은 담당자 목록의 4번이 비어 있어 칸 만들기가 중간에 실패하는 경우입니다. "새어 나간 칸"은 함수가 `Err`를 돌려준 뒤에도 살아 있는(아무도 해제하지 않은) 칸의 수입니다. 비권장 판의 누수는 미정의 동작이 아니며, 프로그램이 끝날 때 운영체제가 메모리를 회수합니다.

```sh
cargo run -p er-ch16-avoid-unsafe --example 03_build_arrays
```

```text
1. 칸 10개 만들기
    비권장 MaybeUninit:  0구역 … 9구역 (10칸, 담당 미정)
    권장 array::from_fn: 0구역 … 9구역 (10칸, 담당 미정)
    우편물 6통 분류: 0구역 2통, 1구역 1통, 3구역 1통, 4구역 1통, 6구역 1통
    살아 있는 칸: 0개
2. 담당자 목록으로 칸 만들기 (4번 칸 담당자가 비어 있음)
    비권장 MaybeUninit + ?: Err("4번 칸의 담당자 이름이 비어 있음"), 새어 나간 칸 4개
    권장 collect + try_into: Err("4번 칸의 담당자 이름이 비어 있음"), 새어 나간 칸 0개
    빈 이름을 채운 뒤 권장 판: Ok(()), 새어 나간 칸 0개
```

### 04_safe_wrapper — 안전한 래퍼

샘플 세 개(`ATGCGTAC`, `ATGUGA`, `GGCGCCAT`)를 두 판으로 보여 주고, `"GCG"`를 찾고, 상보 서열을 만듭니다. "전제 조건을 확인한 횟수"는 A/C/G/T만 있는지 확인한 횟수입니다.

```sh
cargo run -p er-ch16-avoid-unsafe --example 04_safe_wrapper
```

```text
1. 비권장: 바이트를 주고받고, 부르는 곳마다 확인한 뒤 unsafe
    샘플 1: ATGCGTAC, "GCG" 위치 Some(2), 상보 TACGCATG
    샘플 2: 위치 3의 바이트가 염기가 아니라 건너뜀
    샘플 3: GGCGCCAT, "GCG" 위치 Some(1), 상보 CCGCGGTA
    전제 조건을 확인한 횟수: 7번
2. 권장: 안전한 래퍼 DnaSeq
    샘플 1: ATGCGTAC, "GCG" 위치 Some(2), 상보 TACGCATG
    샘플 2: 만들 때 거부: 위치 3의 'U'는 염기가 아님
    샘플 3: GGCGCCAT, "GCG" 위치 Some(1), 상보 CCGCGGTA
    전제 조건을 확인한 횟수: 3번
```

`dna` 모듈의 테스트는 다음 명령으로 실행합니다. 테스트가 끝나는 순서는 실행마다 다를 수 있습니다.

```sh
cargo test -p er-ch16-avoid-unsafe --example 04_safe_wrapper
```

```text
running 5 tests
test dna::tests::accepts_only_upper_acgt ... ok
test dna::tests::as_str_shows_same_bytes ... ok
test dna::tests::complement_keeps_invariant ... ok
test dna::tests::empty_sequence_is_valid ... ok
test dna::tests::reports_first_invalid_position ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo clippy -p er-ch16-avoid-unsafe --examples`는 `#[cfg(test)]` 안의 코드를 검사하지 않습니다. 테스트 코드까지 검사하려면 `cargo clippy -p er-ch16-avoid-unsafe --examples -- --test`를 씁니다.

### 05_safety_comments — 안전성 주석과 블록 크기

예매 기록 네 줄(정상 두 줄, 14바이트짜리 줄, 도착역에 한글이 든 줄)을 세 판의 `unsafe fn`으로 자르고, 2번 구역에서 `unsafe` 없는 판과 비교합니다. 세 판은 같은 결과를 내므로 차이는 실행 결과가 아니라 2.6절의 경고에 있습니다. 4번 줄은 부르는 쪽의 ASCII 확인 때문에 `unsafe` 판에서는 거부되고, 칸마다 `str::from_utf8`로 확인하는 안전한 판에서는 읽힙니다.

```sh
cargo run -p er-ch16-avoid-unsafe --example 05_safety_comments
```

```text
1. 세 판의 unsafe fn (부르는 쪽이 길이와 ASCII를 한 번 확인)
    줄 1: KTX 0101 SEOUL → BUSAN 09:30 (세 판이 같음: true)
    줄 2: SRT 0305 SUSEO → MOKPO 14:15 (세 판이 같음: true)
    줄 3: 길이가 25가 아님(14바이트) → unsafe fn을 부르지 않음
    줄 4: ASCII가 아님 → unsafe fn을 부르지 않음
2. 안전한 판 (get + str::from_utf8, unsafe 없음)
    줄 1: KTX 0101 SEOUL → BUSAN 09:30
    줄 2: SRT 0305 SUSEO → MOKPO 14:15
    줄 3: 길이가 25가 아님(14바이트)
    줄 4: KTX 0107 SEOUL → 대전 08:00
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다). 본편에 `unsafe` 블록 없이 unsafe 연산을 쓴 E0133 예제가 여럿 있으므로(Part 0 ch19 4.1절, Part 2 ch5 4.4·4.5절) 여기서는 다루지 않습니다.

```sh
cd appd-a/ch16-avoid-unsafe
rustc --edition 2024 compile_fail/e0277_array_repeat_non_copy.rs --out-dir /tmp/er-ch16
rustc --edition 2024 compile_fail/e0453_allow_under_forbid_unsafe.rs --out-dir /tmp/er-ch16
rustc --edition 2024 compile_fail/e0451_private_field_bypass.rs --out-dir /tmp/er-ch16
```

### 4.1 `Copy`가 아닌 값으로 배열 식을 씀 — E0277

파일: [`compile_fail/e0277_array_repeat_non_copy.rs`](compile_fail/e0277_array_repeat_non_copy.rs)

```text
error[E0277]: the trait bound `Bin: Copy` is not satisfied
  --> compile_fail/e0277_array_repeat_non_copy.rs:26:28
   |
26 |     let bins: [Bin; 10] = [Bin::empty(); 10]; // E0277
   |                            ^^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `Copy` is not implemented for `Bin`
  --> compile_fail/e0277_array_repeat_non_copy.rs:11:1
   |
11 | struct Bin {
   | ^^^^^^^^^^
   = note: the `Copy` trait is required because this value will be copied for each element of the array
   = help: consider using `core::array::from_fn` to initialize the array
   = help: see https://doc.rust-lang.org/stable/std/array/fn.from_fn.html for more information
```

- **읽는 법**: `note:`가 이유를 말합니다. `[값; N]`은 값 하나를 N번 **복사**해 채우므로 `Copy`가 필요합니다.
- **원인**: `Bin`은 `String`과 `Vec`을 가지므로 `Copy`가 될 수 없습니다. 복사로 채우면 같은 힙 메모리를 가리키는 칸이 열 개 생기기 때문입니다.
- **해결**: 마지막 두 `help:`대로 `std::array::from_fn(|_| Bin::empty())`를 씁니다(예제 03). 값이 `Vec::new()`처럼 상수로 만들 수 있는 것이면 컴파일러가 `` help: create an inline `const` block ``으로 `[const { Vec::new() }; 10]`을 제안합니다. 여기서 `MaybeUninit`으로 가는 것이 2.3절 사례 3의 비권장 코드입니다.

### 4.2 `forbid`로 금지한 `unsafe`를 모듈에서 다시 허락함 — E0453

파일: [`compile_fail/e0453_allow_under_forbid_unsafe.rs`](compile_fail/e0453_allow_under_forbid_unsafe.rs)

```text
error[E0453]: allow(unsafe_code) incompatible with previous forbid
  --> compile_fail/e0453_allow_under_forbid_unsafe.rs:14:9
   |
12 | #![forbid(unsafe_code)]
   |           ----------- `forbid` level set here
13 |
14 | #[allow(unsafe_code)] // E0453
   |         ^^^^^^^^^^^ overruled by previous forbid

error: usage of an `unsafe` block
  --> compile_fail/e0453_allow_under_forbid_unsafe.rs:23:14
   |
23 |         Some(unsafe { std::str::from_utf8_unchecked(head) })
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
note: the lint level is defined here
  --> compile_fail/e0453_allow_under_forbid_unsafe.rs:12:11
   |
12 | #![forbid(unsafe_code)]
   |           ^^^^^^^^^^^
```

- **읽는 법**: 에러가 둘입니다. 첫째(E0453)는 `allow`가 `forbid`에 "overruled"됐다는 것이고, 둘째는 그 결과 `unsafe` 블록이 여전히 금지라는 린트 에러입니다. 둘째에는 에러 코드가 없습니다. 린트가 낸 에러이기 때문입니다.
- **원인**: `forbid`는 `deny`와 달리 크레이트 안의 어느 곳도 수준을 낮출 수 없는 금지입니다.
- **해결**: 한 모듈에만 `unsafe`를 허락하려면 `#![deny(unsafe_code)]`로 바꿉니다(예제 04의 모양, 이 파일을 그렇게 고치면 `Some("KTX")`를 출력함을 확인). `unsafe`가 하나도 필요 없는 크레이트라면 `forbid`가 맞으며, 그때는 이 모듈의 `unsafe`를 지웁니다(이 경우는 `&str`을 돌려주는 `str::from_utf8(head).ok()`로 충분합니다).

### 4.3 래퍼의 비공개 필드를 밖에서 직접 채움 — E0451

파일: [`compile_fail/e0451_private_field_bypass.rs`](compile_fail/e0451_private_field_bypass.rs)

```text
error[E0451]: field `bases` of struct `DnaSeq` is private
  --> compile_fail/e0451_private_field_bypass.rs:36:32
   |
36 |     let forged = dna::DnaSeq { bases: bad }; // E0451
   |                                ^^^^^ private field
```

- **읽는 법**: 흔한 가시성 에러지만, 여기서는 이 에러가 래퍼의 건전성을 지켜 줍니다.
- **원인**: 검사하는 생성자 `from_bytes`를 건너뛰고 0xFF가 든 `DnaSeq`를 만들려 했습니다. 이것이 컴파일된다면 안전한 메서드 `as_str()`가 올바르지 않은 `&str`을 만들어 미정의 동작이 됩니다. 만든 뒤에 고치는 길(`seq.bases.push(0xFF)`)도 E0616(``field `bases` of struct `DnaSeq` is private``)으로 막힙니다.
- **해결**: 이 에러는 고칠 대상이 아니라 지킬 대상입니다. 값은 `DnaSeq::from_bytes`로 만듭니다. 래퍼를 설계할 때 반대로 생각하면, 불변식에 기대는 `unsafe`가 있는 타입의 필드를 `pub`으로 여는 순간 그 타입의 안전한 API는 건전하지 않게 됩니다(2.5절).

### 4.4 그 밖의 실수

모두 이 툴체인에서 컴파일만 해서 확인했습니다. 미정의 동작이 되는 코드는 실행하지 않았습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| `unsafe { str::from_utf8_unchecked(bytes) }`를 감싼 **안전한** 함수 | 경고 없음. 건전하지 않음 | 함수 안에서 확인하거나 `unsafe fn`으로 (2.1절) |
| `transmute::<[u8; 2], u16>(x)` | 경고 ``unnecessary transmute`` (rustc `unnecessary_transmutes`) | `u16::from_le_bytes` 등 (2.3절) |
| `str::from_utf8_unchecked(b"A\xFF")` | 에러 ``calls to `std::str::from_utf8_unchecked` with an invalid literal are undefined behavior`` (rustc `invalid_from_utf8_unchecked`, deny) | `str::from_utf8`. 리터럴이 아닌 값은 이 린트가 잡지 못함 |
| `pub unsafe fn`에 `# Safety` 절이 없음 | 경고 ``unsafe function's docs are missing a `# Safety` section`` (Clippy `missing_safety_doc`) | 부르는 쪽이 지킬 조건을 적음 (2.6절) |
| 2024 에디션에서 `unsafe fn` 본문에 블록 없이 unsafe 연산 | 경고 E0133 (`unsafe_op_in_unsafe_fn`) | 연산마다 `unsafe { }`와 `// SAFETY:` (2.6절) |
| `[Vec<String>; 40]`을 `Default::default()`로 | E0277 (`Default`는 길이 32까지) | `[const { Vec::new() }; 40]` 또는 `array::from_fn` |
| `std::array::try_from_fn` | E0658 (nightly 전용) | `Vec`으로 모은 뒤 `try_into()` (예제 03) |
| `static mut`의 참조 | 에러 (`static_mut_refs`, 2024에서 deny) | 원자 타입, `Mutex`, `OnceLock` ([Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.9절) |

## 5. 연습 문제

### 연습 1. 측정 시각 읽기

[`02_bytes_to_values.rs`](examples/02_bytes_to_values.rs)의 체중계가 새 펌웨어에서 패킷 맨 앞에 4바이트 측정 시각(유닉스 시간, `u32` 빅 엔디언)을 붙인다고 합시다. 권장 판에 `read_u32_be`를 추가하고 이름의 시작 위치를 고치세요. `unsafe`와 `transmute`는 쓰지 않습니다.

<details>
<summary>힌트</summary>

`bytes.get(at..)?.first_chunk::<4>()?`로 `&[u8; 4]`를 얻고 `u32::from_be_bytes(*field)`로 읽습니다. 앞에서부터 차례로 떼어 내고 싶다면 `split_first_chunk::<4>()`(1.77.0)가 `(&[u8; 4], &[u8])`을 돌려줍니다. 이 경우 번호·체중은 리틀 엔디언, 시각은 빅 엔디언처럼 한 패킷 안에서 순서가 섞여도 함수 이름이 그것을 드러냅니다. 비권장 판이었다면 시각을 읽을 때만 `swap_bytes`를 불러야 하는데, 그것이 맞는지는 기계에 따라 다릅니다.

</details>

### 연습 2. 래퍼에 메서드 더하기

[`04_safe_wrapper.rs`](examples/04_safe_wrapper.rs)의 `DnaSeq`에 두 메서드를 추가하세요. 하나는 염기 하나를 덧붙이는 `push(&mut self, base: u8) -> Result<(), InvalidBase>`, 다른 하나는 검사 없이 만드는 `from_bytes_unchecked(bytes: Vec<u8>) -> DnaSeq`입니다. 각각 안전한 함수로 둘지 `unsafe fn`으로 둘지 정하고, `push`의 테스트를 쓰세요.

<details>
<summary>힌트</summary>

`push`는 받은 바이트가 A/C/G/T인지 확인한 뒤에만 넣으면 불변식을 지키므로 안전한 함수입니다. 확인을 빠뜨리면 `push(0xFF)` 한 줄로 `as_str`이 미정의 동작을 일으키므로, 확인이 있는 판과 없는 판의 차이가 곧 건전성의 차이입니다. `from_bytes_unchecked`는 안전한 코드에서 불변식을 깰 수 있으므로 `unsafe fn`이어야 하고, 문서에 `# Safety` 절(`bytes`에는 A/C/G/T만 있어야 함)을 씁니다. `String::from_utf8_unchecked`와 같은 모양입니다. 테스트는 거부해야 할 바이트(`b'a'`, `b'N'`, 0xFF)를 넣은 뒤에도 `as_str()`이 원래 서열 그대로인지 확인합니다.

</details>

### 연습 3. `forbid`를 걸 수 있게 나누기

예제 04 파일 첫 줄을 `#![forbid(unsafe_code)]`로 바꾸면 어떤 에러가 나는지 예측한 뒤 확인하세요. 그런 다음 비권장 판(`scattered`, `scattered_demo`)을 지워도 `forbid`로는 컴파일되지 않는 이유를 설명하고, 실제 프로젝트라면 어떻게 나눌지 적어 보세요.

<details>
<summary>힌트</summary>

`#[allow]`와 `#[expect]` 세 곳이 모두 E0453이 되고, `unsafe` 블록과 `unsafe fn` 선언이 린트 에러로 남습니다(4.2절). `dna` 모듈은 비권장 판을 지워도 `as_str` 안에 `unsafe`가 남으므로 `forbid`와 함께 둘 수 없습니다. 실제 프로젝트라면 `DnaSeq`를 별도 라이브러리 크레이트로 떼어 그 크레이트만 `unsafe`를 쓰게 하고(`deny` + 모듈 허락), 그것을 쓰는 애플리케이션 크레이트의 `Cargo.toml`에 `[lints.rust] unsafe_code = "forbid"`를 겁니다. 그러면 검토할 `unsafe`가 어느 크레이트에 있는지가 의존성 목록에 드러납니다. 한 워크스페이스에 크레이트 여럿을 두는 방법은 Part 3 `ch4-workspaces`에서 다룹니다.

</details>

### 연습 4. 나머지까지 평균 내기

[`01_checked_access.rs`](examples/01_checked_access.rs)의 두 `group_averages`를, 묶음에 들지 못한 나머지(2개)도 마지막 평균으로 넣도록 고치세요(결과는 `[322.0, 356.8, 389.6, 399.5]`). 비권장 판은 고친 뒤 `SAFETY:` 주석의 증명을 다시 쓰고, 두 판에서 고쳐야 했던 곳을 비교하세요.

<details>
<summary>힌트</summary>

권장 판은 `as_chunks`가 돌려준 `rest`가 비어 있지 않으면 `rest.iter().sum::<i32>()`를 `rest.len()`으로 나눈 값을 덧붙이면 끝입니다. 비권장 판은 나머지 구간의 시작 `groups * GROUP`부터 `alts.len()`까지 도는 반복문을 더하고, 그 인덱스가 왜 범위 안인지를 새로 증명해야 합니다. 증명을 쓰다 보면 `for i in groups * GROUP..alts.len()`의 `i`는 반복 범위 때문에 이미 `len`보다 작다는 것, 즉 `get_unchecked` 대신 `alts[groups * GROUP..]`처럼 슬라이스를 쓰면 증명이 필요 없다는 것이 보입니다. 코드를 고칠 때마다 증명도 고쳐야 한다는 것이 `unsafe`의 유지 비용입니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 16: Avoid writing unsafe code](https://effective-rust.com/unsafe.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다. 책의 이 Item에는 Rust 코드 예제가 없습니다.
- 표준 라이브러리 문서
  - [슬라이스](https://doc.rust-lang.org/std/primitive.slice.html): `get_unchecked`(Safety 절), `array_windows`, `as_chunks`, `chunks_exact`, `first_chunk`, `split_first_chunk`, `get_disjoint_mut`
  - [`u16::from_le_bytes`](https://doc.rust-lang.org/std/primitive.u16.html#method.from_le_bytes), [`std::str::from_utf8`](https://doc.rust-lang.org/std/str/fn.from_utf8.html), [`std::str::from_utf8_unchecked`](https://doc.rust-lang.org/std/str/fn.from_utf8_unchecked.html)
  - [`std::ptr::read`](https://doc.rust-lang.org/std/ptr/fn.read.html)(Safety 절, Ownership of the Returned Value 절), [`std::ptr::read_unaligned`](https://doc.rust-lang.org/std/ptr/fn.read_unaligned.html), [`std::mem::transmute`](https://doc.rust-lang.org/std/mem/fn.transmute.html)
  - [`MaybeUninit`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html)(Initializing an array element-by-element 절), [`std::array::from_fn`](https://doc.rust-lang.org/std/array/fn.from_fn.html), [`std::array::repeat`](https://doc.rust-lang.org/std/array/fn.repeat.html)
  - [`String`](https://doc.rust-lang.org/std/string/struct.String.html): `from_utf8`, `from_utf8_unchecked`, `as_mut_vec`
  - [`OnceLock`](https://doc.rust-lang.org/std/sync/struct.OnceLock.html), [`LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html)
- The Rustonomicon: [Working with Unsafe](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)(모듈 경계와 가시성)
- Miri: [README](https://github.com/rust-lang/miri)(설치, `cargo miri run/test`, 찾는 것과 한계)
- rustc 린트: [`unsafe_code`](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unsafe-code), [`unsafe_op_in_unsafe_fn`](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unsafe-op-in-unsafe-fn), [`unnecessary_transmutes`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unnecessary-transmutes), [`invalid_from_utf8_unchecked`](https://doc.rust-lang.org/rustc/lints/listing/deny-by-default.html#invalid-from-utf8-unchecked)
- Clippy 린트: [`missing_safety_doc`](https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#missing_safety_doc), [`undocumented_unsafe_blocks`](https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#undocumented_unsafe_blocks), [`multiple_unsafe_ops_per_block`](https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#multiple_unsafe_ops_per_block)
- Edition Guide: [`unsafe_op_in_unsafe_fn` warning](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html)(2024 에디션에서 기본 경고가 된 이유)
- The Cargo Book: [The `[lints]` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section)
- 에러 설명: `rustc --explain E0277`, `rustc --explain E0453`, `rustc --explain E0451`, `rustc --explain E0616`
