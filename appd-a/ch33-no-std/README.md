# ch33. 라이브러리를 `no_std` 호환으로 만들기

표준 라이브러리 가운데 운영체제가 필요한 부분은 `std`뿐이고, 많은 타입은 `core`와 `alloc`에 정의되어 있습니다. 그래서 라이브러리 코드는 생각보다 적은 수고로 마이크로컨트롤러나 커널처럼 `std`가 없는 환경에서도 쓰이게 만들 수 있습니다. 이 챕터는 그 판단을 다룹니다. 언제 `no_std` 호환을 고려할지, `std`가 필요한 일부를 기능(feature)으로 어떻게 떼어 낼지, `std`가 없는데 어떻게 테스트하고 무엇으로 호환을 확인할지, 힙이 제한된 환경에서 할당 실패를 어떻게 다룰지입니다. 범위는 라이브러리 코드뿐이고, `no_std` 실행 파일은 만들지 않습니다.

- 원제: [Item 33: Consider making library code `no_std` compatible](https://effective-rust.com/no-std.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch33-no-std`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

책의 6장(Beyond Standard Rust)의 첫 Item입니다. 다음 내용을 안다고 보고 진행합니다.

- [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.1~2.2절: `core`·`alloc`·`std`의 세 층, 재내보내기, `extern crate alloc;`, 프렐류드
- [ch4](../ch4-idiomatic-errors/README.md): `Error` 트레이트, `thiserror`
- [ch26](../ch26-feature-creep/README.md): 더하기만 하는 기능, 기능 통합, `--no-default-features`/`--all-features`, `required-features`, 기능으로 가린 배리언트와 `#[non_exhaustive]`
- [ch25](../ch25-dependency-graph/README.md): `cargo tree`
- [ch16](../ch16-avoid-unsafe/README.md): `unsafe`와 안전성 주석(예제 05), [ch18](../ch18-dont-panic/README.md): `catch_unwind`와 패닉 훅(예제 04)

`no_std` 실행 파일(패닉 처리기, 링커 스크립트, 타깃)은 Part 8 `ch1-intro-no-std`에서, 전역 할당기와 `heapless`는 Part 8 `ch7-collections-design-patterns`에서, `#[panic_handler]`는 Part 10 `ch12-ffi-beneath-std`에서, CI에 검사를 올리는 일은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 다룹니다. 이 챕터는 "무엇을 검사해야 하는가"까지만 다룹니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 어떤 항목이 `core`·`alloc`·`std` 가운데 어디에 있는지 문서와 컴파일러로 확인하고, `std`에만 있는 항목(`HashMap`, `Mutex`, `std::io`, `f64::sqrt` 등)의 대안을 고를 수 있다.
2. `#![no_std]` 라이브러리에서 프렐류드에 없는 이름을 `core::`·`alloc::` 경로로 가져와 작성할 수 있다.
3. `std`가 필요한 부분을 더하기만 하는 `std`·`alloc` 기능으로 떼어 내고, 두 가지 크레이트 루트 모양(`#![cfg_attr(not(feature = "std"), no_std)]`와 `#![no_std]` + `extern crate std`)의 차이를 설명할 수 있다.
4. `no_std` 라이브러리를 호스트에서 테스트하고, 기능 조합마다 빌드·테스트하며, 호스트 빌드가 잡지 못하는 `std` 누출을 무엇으로 확인하는지 설명할 수 있다.
5. `try_reserve`와 `Allocator`로 할당 실패를 `Result`로 다루고, 실패하지 않는 API가 실패하면 어떻게 되는지 설명할 수 있다.

## 2. 핵심 개념

2.1절에서 판단의 바탕을 정리하고, 2.2~2.6절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 지침 1·2·4·5는 책이 권하는 것이고, 지침 3과 각 지침에 더한 도구는 책 밖의 근거로 더했습니다(2.7절).

### 2.1 지도: 세 환경, 세 층, 그리고 `#![no_std]`

책은 환경을 셋으로 나눕니다. 여기에 보통의 운영체제 환경을 더하면 다음과 같습니다.

| 환경 | 쓸 수 있는 층 | 할당이 실패하면 |
| --- | --- | --- |
| 힙이 전혀 없음(작은 마이크로컨트롤러) | `core` | 할당 자체가 없음 |
| 힙이 넉넉함 | `core` + `alloc` | 실패하지 않는다고 가정 |
| 힙이 있지만 작아서 할당이 실패할 수 있음 | `core` + `alloc` | 실패를 다뤄야 함(지침 5) |
| 운영체제가 있음 | `core` + `alloc` + `std` | 보통 실패하지 않음(2.6절) |

세 층이 무엇이고 왜 나뉘었는지는 Part 1 ch1에서 봤습니다. 이 챕터에 필요한 것은 `#![no_std]`의 정확한 뜻입니다. The Reference는 이 속성이 두 가지를 한다고 설명합니다. `std`를 자동으로 연결하지 않고, 프렐류드를 `std`의 것 대신 `core`의 것으로 바꿉니다. 그리고 `no_std`가 `std`의 연결을 **막지는 않는다**고 경고합니다. 크레이트 자신이나 의존성 가운데 하나가 `extern crate std`를 쓰면 `std`가 연결됩니다. 지침 4의 문제가 여기서 나옵니다.

**항목이 어디에 있는지 확인하는 방법.** 책은 문서의 "Source" 링크가 `src/core/…`를 가리키면 `core`에 있다고 보는 방법을 권하면서, 가끔 틀린다고 덧붙입니다. [`01_where_items_live.rs`](examples/01_where_items_live.rs)는 같은 일을 코드로 합니다. `TypeId`로 두 경로가 같은 타입인지 비교하고, `type_name`으로 정의된 크레이트를 엿봅니다.

```text
1. std 경로와 core/alloc 경로가 같은 타입인가
  dyn core::error::Error (1.81)                true
  core::net::Ipv4Addr (1.77)                   true
  ...
3. core에 정의됐지만 core 경로로는 아직 쓸 수 없는 것
  core::io::error::Error
  dyn core::io::write::Write
```

3번 구역이 책이 말한 "가끔 틀리는" 경우의 현재 판입니다. `std::io::Error`와 `std::io::Write`는 `core::io`에 정의되어 있고 문서의 Source 링크도 `src/core/io/…`를 가리키지만, 1.100.0에서 `core::io` 모듈은 nightly 전용이라 `no_std` 크레이트에서 쓰면 E0658입니다(4.3절). 정의 위치를 본 다음 `core` 문서의 그 페이지에 안정화 버전이 적혀 있는지 확인합니다. 컴파일러도 도와줍니다. `#![no_std]`에서 `std::time::Duration`을 쓰면 E0433과 함께 `use core::time::Duration;`을 제안합니다(4.5절).

**`std`에만 있는 것과 대안.** 1.100.0 로컬 문서로 확인한 것입니다.

| `std` 항목 | `std`에만 있는 이유 | `no_std`에서의 대안 |
| --- | --- | --- |
| `HashMap`, `HashSet` | 기본 해셔의 시드를 운영체제의 난수로 정함 | alloc의 `BTreeMap`, `BTreeSet`(예제 03). 해시 맵이 꼭 필요하면 `hashbrown` 크레이트에 해셔를 직접 줌 |
| `Mutex`, `RwLock`, `OnceLock`, `LazyLock` | 운영체제의 스레드 동기화 | 한 스레드에서만 쓰면 core의 `RefCell`, `OnceCell`, `LazyCell`. 여러 스레드라면 원자 타입이나 `spin` 크레이트(책: 사실상 유일한 선택) |
| `std::io`(`Read`, `Write`) | `core::io`는 nightly 전용 | 바이트 슬라이스를 주고받는 API로 설계하고, `io` 연동은 `std` 기능 뒤에 둠(2.3절) |
| `f64::sqrt`, `floor`, `round`, `powi`, `sin` 등 | `std`가 더한 `impl` 블록에 있음 | `libm` 같은 순수 Rust 수학 크레이트(4.4절) |
| `Instant`, `SystemTime`, `thread`, `fs`, 소켓, `env` | 운영체제가 필요 | 시각이나 입력을 부르는 쪽이 인자로 넘기게 설계 |

크레이트 이름(`hashbrown`, `spin`, `libm`)은 소개만 하고 이 챕터에서 쓰지 않습니다.

### 2.2 지침 1: 라이브러리 코드는 `core`·`alloc` 경로로 쓴다

**문제 상황.** 장치 경보를 세고 보고서를 만드는 작은 라이브러리를 씁니다. 하는 일은 문자열을 나누고, 수를 세고, 문자열을 만드는 것뿐이라 운영체제가 필요 없습니다. 그런데 습관대로 `std::`를 쓰면 `std`가 없는 환경의 사용자는 이 크레이트를 쓸 수 없습니다.

**비권장 코드.** `std`의 프렐류드와 경로에 기댑니다.

```rust
use std::collections::HashMap;

pub fn tally(alarms: &[Device]) -> HashMap<Device, u32> { /* … */ }
pub fn report(n: usize) -> String { format!("품목 {n}개") }
```

**권장 코드.** 크레이트 루트에 `#![no_std]`를 붙이고 `core`·`alloc` 경로로 가져옵니다. [`02_no_std_imports.rs`](examples/02_no_std_imports.rs)는 파일 자체가 `#![no_std]`인 예제이고, 가져오기 목록이 곧 "`std` 프렐류드에는 있지만 `core` 프렐류드에는 없는 이름"입니다.

```rust
#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
```

`Option`, `Result`, `Iterator`, `Clone`, `TryFrom` 같은 것은 `core` 프렐류드에 있어 그대로 씁니다. `.to_string()`을 쓰려면 `ToString` 트레이트를 가져와야 한다는 점이 자주 놓치는 곳입니다(4.1절). 에러 타입도 문제없습니다. `Error` 트레이트가 1.81부터 `core::error::Error`에 있어서 `impl core::error::Error for LineError {}`가 `std` 없이 컴파일되고, `ParseIntError`를 `Box<dyn Error>`로 바꾸는 `From` 구현도 `alloc`에 있어 `?`가 그대로 동작합니다.

`HashMap`은 `BTreeMap`으로 바꿉니다. [`03_hashmap_to_btreemap.rs`](examples/03_hashmap_to_btreemap.rs)의 두 판은 경로와 타입 이름만 다르고 `entry(..).or_insert(0)` 본문은 같습니다. 달라지는 것은 셋입니다.

- **키의 바운드.** `HashMap`은 `Hash + Eq`, `BTreeMap`은 `Ord`를 요구합니다. 예제의 `Device`는 둘 다 derive했습니다.
- **순회 순서.** `BTreeMap`은 키 순서로 돕니다. 예제의 2번 구역이 `랙 1 슬롯 2`, `랙 2 슬롯 1`, `랙 3 슬롯 4` 순서로 출력되고, `range`로 "랙 2의 장치만" 찾을 수도 있습니다. 테스트의 기대값이 실행마다 같아지는 것도 덤입니다.
- **비용.** 조회가 평균 O(1)에서 O(log n)이 됩니다. 크기가 작은 대부분의 경우에는 문제가 되지 않지만, 성능이 중요하면 측정합니다([ch20](../ch20-avoid-over-optimizing/README.md)).

3번 구역은 `HashMap`이 `std`에만 있는 이유를 보여 줍니다. `RandomState::new()`로 만든 두 해셔는 같은 키에 다른 해시 값을 냅니다. 시드가 운영체제에서 받은 난수로 정해지기 때문이고, 그래서 순회 순서도 실행마다 달라질 수 있습니다.

**의존성도 `no_std`여야 합니다.** 내 코드를 바꿔도 의존성 하나가 `std`를 쓰면 사용자는 결국 `std`를 연결해야 합니다. 책은 이 경우에는 `no_std`로 만드는 의미가 없다고 말합니다. `thiserror`처럼 `std` 기능을 끌 수 있는 크레이트는 `default-features = false`로 씁니다(2.3절).

**따르지 않아도 되는 경우.** 책은 "변환이 단순하고 의존성이 지원할 때" 고려하라고 권합니다. 따라서 다음 경우는 `std` 크레이트로 둡니다.

- 크레이트의 본질이 파일, 네트워크, 스레드, 프로세스처럼 운영체제의 기능일 때
- 꼭 필요한 의존성이 `std`를 요구할 때
- 애플리케이션 크레이트일 때. 다른 크레이트가 의존하지 않으므로 얻는 것이 없습니다.

### 2.3 지침 2: `std`·`alloc`이 필요한 부분은 더하기만 하는 기능으로 떼어 낸다

**문제 상황.** 바이트와 16진수 문자열을 오가는 라이브러리 [`projects/tinyhex`](projects/tinyhex)를 만듭니다. 대부분은 `core`만으로 되지만, `String`을 돌려주는 편한 함수는 힙이 필요하고, `std::io` 연동은 운영체제가 필요합니다. 일부 때문에 크레이트 전체를 `std`로 만들고 싶지는 않습니다.

**비권장 코드 ①: 기능을 끄는 `no_std` 기능.**

```toml
[features]
no_std = []   # 켜면 std를 쓰는 코드를 지움
```

[ch26](../ch26-feature-creep/README.md)에서 본 대로 기능은 그래프 어디선가 켜지면 모두에게 켜집니다. 임베디드 사용자 하나가 `no_std`를 켜면 같은 빌드에 있는 `std` 사용자의 API가 사라집니다. 책과 Cargo Book 모두 `no_std`·`no_alloc` 기능을 만들지 말고 `std`·`alloc` 기능으로 **더하라고** 권합니다.

**비권장 코드 ②(함정이 있는 모양): `std` 기능일 때만 `no_std`를 끄는 크레이트 루트.** 책이 드는 관례적인 모양은 `#![cfg_attr(not(feature = "std"), no_std)]`입니다. 동작은 하지만, `std` 기능이 켜진 빌드에서는 크레이트가 보통의 `std` 크레이트가 되어 프렐류드가 바뀝니다. [`compile_fail/e0425_std_feature_prelude.rs`](compile_fail/e0425_std_feature_prelude.rs)는 `Vec`을 가져오지 않고 씁니다. `std` 기능을 켠 빌드(기본 빌드)는 통과하고, `alloc`만 켠 빌드에서야 E0425가 납니다(4.2절). 기본 빌드로만 개발하면 이 실수가 쌓입니다.

**권장 코드.** 크레이트 루트는 늘 `#![no_std]`로 두고, `std`와 `alloc`은 기능이 켜졌을 때 `extern crate`로 연결합니다. Cargo Book의 Features 장이 예로 드는 모양이고, `thiserror` 2.0.21의 `lib.rs`도 같은 모양입니다(레지스트리에 받은 소스로 확인).

```toml
[dependencies]
thiserror = { version = "2.0.21", default-features = false }

[features]
default = ["std"]
alloc = []
std = ["alloc", "thiserror/std"]
```

```rust
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;
```

`tinyhex`의 API는 층마다 다른 파일에 있습니다.

| 기능 | 파일 | 더해지는 API | 필요한 것 |
| --- | --- | --- | --- |
| 없음 | `src/slice.rs`, `src/error.rs` | `Hex`(할당 없는 `Display`), `decode_to_slice`(부르는 쪽 버퍼에 풀어 씀), `HexError` | `core` |
| `alloc` | `src/owned.rs` | `encode`, `decode`, `try_encode`, `try_decode` | 힙 할당기 |
| `std`(기본) | `src/io.rs` | `write_hex`, `read_hex`, `From<HexError> for io::Error` | 운영체제 |

이 설계에서 지킨 것들입니다.

- **`std`는 `alloc`을 켭니다.** `std`가 있으면 할당기도 있으므로 `std = ["alloc", …]`입니다. `#[cfg(feature = "std")]` 코드가 `alloc`의 이름을 바로 쓸 수 있고(`src/io.rs`가 `alloc::string::String`을 가져옴), 조합 하나(`std`만 켜고 `alloc`은 끈 빌드)가 사라집니다.
- **기본 기능에 `std`를 둡니다.** 대부분의 사용자는 `std` 환경이라 아무것도 적지 않아도 모든 API를 씁니다. `no_std` 사용자만 `default-features = false`를 적습니다.
- **의존성의 `std`도 함께 켭니다.** `"thiserror/std"`는 내 `std` 기능이 켜질 때만 `thiserror`의 `std` 기능을 켭니다. 기본으로는 꺼 두어야 `no_std` 빌드에 `std`가 섞이지 않습니다.
- **에러 타입은 `core`에 둡니다.** `thiserror`가 `std` 기능 없이 `core::error::Error`를 구현해 줍니다. `alloc`이 켜지면 `OutOfMemory(TryReserveError)` 배리언트가 더해지므로 `#[non_exhaustive]`를 붙였습니다([ch26](../ch26-feature-creep/README.md) 2.4절). 다른 크레이트의 `match`에는 `_` 갈래가 필요합니다(`tests/host.rs`).
- **`std`가 필요한 이유가 분명한 것만 `std` 뒤에 둡니다.** 1.81 이전에는 `Error` 구현 때문에 `std` 기능이 필요했지만, 지금 `tinyhex`에서 `std`가 꼭 필요한 것은 `std::io`뿐입니다.

**따르지 않아도 되는 경우.** 기능은 조합을 늘립니다. 크레이트의 거의 모든 API가 힙을 쓴다면 `alloc` 기능을 따로 두지 말고 처음부터 `alloc`을 요구하는(`extern crate alloc;`을 조건 없이 쓰는) 편이 단순합니다. 책도 `std`나 `alloc`이 필요한 부분이 **작게 떼어질 때** 기능으로 두라고 권합니다.

### 2.4 지침 3: 호스트에서 테스트하고, 기능 조합마다 검사한다

**문제 상황.** 테스트 하네스는 `std`를 씁니다. `std`가 없는 라이브러리의 테스트를 어디서 어떻게 돌릴까요? 또 기능 셋이 만드는 빌드가 넷입니다.

**호스트에서 테스트하기.** 직접 확인한 것은 다음과 같습니다.

- **`#![no_std]` 라이브러리에 `#[test]`를 쓰면 `cargo test`가 그냥 동작합니다.** 테스트 하네스가 `std`를 스스로 연결하기 때문입니다. 다만 테스트 코드도 `#![no_std]` 크레이트 안에 있어서 `vec!`나 `println!`을 쓰면 `cannot find macro`가 납니다.
- **테스트 코드가 `std`의 이름을 쓰려면 `#[cfg(test)] extern crate std;`를 둡니다.** `tinyhex`는 `#[cfg(any(feature = "std", test))]`로 썼고, 단위 테스트는 `use std::format;`처럼 가져옵니다(`src/slice.rs`).
- **통합 테스트(`tests/`)는 처음부터 보통의 `std` 크레이트입니다.** `std` 프렐류드를 그대로 쓰고, `#[cfg(feature = "alloc")]`처럼 패키지의 기능으로 테스트를 가릴 수 있습니다(`tests/host.rs`). 문서 테스트도 `std` 크레이트로 빌드됩니다.

**비권장 코드: `#![cfg_attr(not(test), no_std)]`.** 테스트를 빌드할 때 크레이트 전체를 `std` 크레이트로 만드는 방법도 있습니다. 문제는 라이브러리 코드까지 테스트 빌드에서는 `std`가 된다는 점입니다. 라이브러리 코드에 가져오지 않은 `Vec`과 `std::time::Instant`를 쓴 작은 크레이트에서 `cargo test --lib`은 통과했습니다. 그냥 `cargo test`는 문서 테스트를 위해 라이브러리를 보통으로도 빌드하므로 E0433과 E0425로 실패했습니다. 어느 명령을 쓰느냐에 따라 결과가 달라지므로, 테스트 빌드와 보통 빌드의 프렐류드가 같은 `#[cfg(test)] extern crate std;` 쪽을 권합니다.

**권장 코드: 기능 조합마다 빌드·clippy·테스트.** `tinyhex` 디렉터리에서 실행한 반복문입니다(zsh).

```sh
for f in "--no-default-features" "--no-default-features --features alloc" "" "--all-features"; do
  cargo build ${=f}
  cargo clippy --all-targets ${=f} -- -D warnings
  cargo test ${=f}
done
```

| 조합 | 단위 테스트 | 통합 테스트 | 문서 테스트 |
| --- | --- | --- | --- |
| `--no-default-features` | 3 | 1 | 1 |
| `--no-default-features --features alloc` | 5 | 2 | 1 |
| 기본(`std`) | 5 | 3 | 1 |
| `--all-features` | 5 | 3 | 1 |

이 검사는 실제로 일을 했습니다. `tinyhex`를 처음 썼을 때 기본 빌드는 경고 없이 통과했지만, `--no-default-features`에서는 `alloc`판에서만 쓰던 상수에 `dead_code` 경고가, 통합 테스트에서는 `std`판 테스트에서만 쓰던 가져오기에 `unused_imports` 경고가 났습니다. `#[cfg]`로 지워진 코드는 검사받지 않으므로([ch26](../ch26-feature-creep/README.md) 2.2절) 조합마다 따로 봐야 합니다. `examples/dump.rs`는 `required-features = ["std"]`라서 `std`가 없는 조합에서는 건너뜁니다.

**따르지 않아도 되는 경우.** 기능이 많아 조합이 감당되지 않으면 ch26 2.6절의 전략(없음·기본·전부만, 또는 `cargo-hack`)을 씁니다. `std`를 끈 조합은 빼지 않습니다. `no_std` 호환을 약속한다면 그 조합이 약속의 내용입니다.

### 2.5 지침 4: `std` 없는 타깃으로 `no_std` 빌드를 확인한다

**문제 상황.** 책은 컴파일러가 "`no_std` 크레이트가 `std`를 쓰는 의존성을 얻었다"고 알려 주지 않는다고 경고합니다. 2.1절의 Reference 경고와 같은 이야기입니다. [`projects/leak_check`](projects/leak_check)는 이것을 재현합니다. `sensor`는 `#![no_std]` 라이브러리이고, 센서 수를 세는 일을 `HashSet`을 쓰는 `std` 라이브러리 `stdful`에 맡깁니다.

**비권장 코드: 호스트에서 `--no-default-features`로만 빌드해 보고 끝냄.** 호스트에는 `std`가 있으므로 다음 둘을 구별하지 못합니다.

- 지금 판: `stdful`은 `optional = true`이고 `sensor`의 `std` 기능만 끌어옵니다. `cargo build -p sensor --no-default-features`가 통과합니다.
- 사본에서 `optional = true`를 지우고 `distinct_sensors`의 `#[cfg]`도 지운 판: `sensor`가 늘 `std`를 연결하는데도 `cargo build -p sensor --no-default-features`와 테스트가 똑같이 경고 없이 통과했습니다.

호스트 빌드가 잡는 것과 잡지 못하는 것을 나누면 다음과 같습니다.

| 호스트 빌드(`--no-default-features`)로 잡힘 | 잡히지 않음 |
| --- | --- |
| 크레이트 자신의 `std::` 경로(E0433) | 의존성이 `std`를 연결하는 것(`stdful`, `default-features = false`를 빠뜨린 `thiserror`) |
| `std` 프렐류드의 이름(E0425, `cannot find macro`) | 타깃에 없는 API. 예: `rustc --print cfg --target thumbv6m-none-eabi`에는 `target_has_atomic` 줄이 하나도 없고, alloc의 `sync` 모듈(`Arc`)은 `#[cfg(… target_has_atomic = "ptr")]`로 가려져 있어 그 타깃에서는 없음 |
| nightly 전용 API(E0658) | 32비트 `usize` 같은 타깃의 차이(`target_pointer_width="32"`) |

**권장 코드: `std`가 없는 타깃으로 빌드합니다(책의 권고).** `std`가 없는 타깃에는 미리 빌드된 `std`가 없으므로, 어디선가 `std`를 연결하면 빌드가 실패합니다.

```sh
rustup target add thumbv6m-none-eabi
cargo build --target thumbv6m-none-eabi --no-default-features
cargo build --target thumbv6m-none-eabi --no-default-features --features alloc
```

**이 저장소에는 `thumbv6m-none-eabi` 타깃이 설치되어 있지 않아 위 명령은 실행해 확인하지 못했습니다.** `rustc --print target-list`에 타깃 이름이 있는 것과 `rustc --print cfg --target thumbv6m-none-eabi`의 출력(`target_os="none"`, `panic="abort"`)만 확인했습니다. 책의 말대로 `std`를 끈 조합은 이런 타깃에서, 나머지 조합은 호스트에서 빌드하는 구성을 CI에 올리는 일은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 다룹니다.

**호스트에서 할 수 있는 보조 확인.** 타깃을 설치할 수 없을 때 쓸 수 있는 것들입니다.

- **`cargo tree -e features -i <크레이트>`로 의존성의 `std` 기능을 봅니다.** `tinyhex`에서 기본 빌드와 `alloc` 빌드를 비교하면, `thiserror`의 `std` 기능을 누가 켰는지가 보입니다(출력은 3절). 다만 `stdful`처럼 기능 없이 `std`를 쓰는 의존성은 이 방법으로 보이지 않습니다. 그래프에 있다는 것만 `cargo tree -i stdful`로 확인할 수 있습니다.
- **`#[panic_handler]`를 정의한 검사용 크레이트를 빌드합니다.** `projects/leak_check/nostd-check`는 `sensor`와 `tinyhex`를 `std` 기능 없이 가져오는 `#![no_std]` 크레이트이고 `#[panic_handler]` 함수를 하나 정의합니다. 이 함수는 `no_std` 프로그램이 패닉했을 때 할 일을 정하며 한 프로그램에 하나만 있을 수 있습니다. `std`에도 하나 있으므로, 그래프 어딘가가 `std`를 연결하면 둘이 겹쳐 빌드가 실패합니다.

```text
$ cargo build --features sensor/std
error[E0152]: found duplicate lang item `panic_impl`
  --> src/lib.rs:16:1
   |
16 | / fn on_panic(_info: &core::panic::PanicInfo<'_>) -> ! {
17 | |     loop {}
18 | | }
   | |_^
   |
   = note: the lang item is first defined in crate `std` (which `stdful` depends on)
   = note: first definition in `std` loaded from …/libstd-….rmeta
   = note: second definition in the local crate (`nostd_check`)
```

note가 `std`를 끌어온 크레이트(`stdful`)의 이름까지 알려 줍니다. `--features tinyhex/std`로 빌드하면 같은 에러에 ``(which `tinyhex` depends on)``이 나옵니다. 기능 없이 빌드하면 통과합니다. 이 검사에는 함정이 셋 있습니다(모두 사본에서 확인).

- 검사할 크레이트를 코드에서 한 번은 가리켜야 합니다(`pub use sensor;`). 이 줄을 지우면 `std`가 섞인 판에서도 빌드가 통과하고 ``warning: unused dependency `sensor` ``만 남았습니다. 쓰지 않는 크레이트는 컴파일러가 불러오지 않기 때문입니다.
- 검사 크레이트는 테스트 하네스로 빌드하면 안 됩니다. `test = false`, `doctest = false`에 더해 `bench = false`도 필요했습니다. 없으면 `cargo clippy --all-targets`가 라이브러리를 테스트 하네스로 빌드해 E0152로 실패했습니다.
- 같은 워크스페이스에 두면 기능 통합에 걸립니다. 사본에서 `nostd-check`를 멤버로 넣자 `cargo build -p nostd-check`는 통과했지만 `cargo build --workspace`는 E0152로 실패했습니다. 함께 빌드하는 `sensor`의 기본 기능(`std`)이 합쳐졌기 때문입니다([ch26](../ch26-feature-creep/README.md) 2.3절). 그래서 `nostd-check`를 워크스페이스의 `exclude`에 두고 자기 `[workspace]`를 줬습니다.

이 검사는 "`std`를 연결하는가"만 봅니다. 위 표의 오른쪽 칸 가운데 타깃에 없는 API는 잡지 못하므로 타깃 빌드를 대신하지 못합니다.

**따르지 않아도 되는 경우.** `no_std` 호환을 약속하지 않는 크레이트에는 필요 없습니다. 반대로 한 번이라도 `no_std` 호환을 알렸다면, 의존성을 더하거나 올릴 때마다 조용히 깨질 수 있으므로 계속 검사해야 합니다.

### 2.6 지침 5: 힙이 제한된 환경에서는 실패할 수 있는 할당을 쓴다

**문제 상황.** `alloc`의 API 대부분은 할당이 실패하지 않는다고 가정합니다. `push`, `reserve`, `with_capacity`, `vec!`, `format!`은 `Result`를 돌려주지 않습니다. 그러면 실패하면 어떻게 될까요? `handle_alloc_error`의 문서는 기본 동작을 이렇게 정합니다. 프로그램이 `std`를 연결했으면 표준 에러에 메시지를 쓰고 **프로세스를 중단**하고, 모든 크레이트가 `#![no_std]`이면 `panic!`을 부릅니다. 힙이 작은 장치에서 메모리가 모자라 장치 전체가 멈추는 것은 받아들이기 어렵습니다.

**비권장 코드.** 실패하지 않는 API만 씁니다. [`05_limited_heap.rs`](examples/05_limited_heap.rs)는 64바이트 예산을 넘는 요청을 거절하는 할당기를 만들어 실패를 결정적으로 일으킵니다. 자식 프로세스에서 `push`만으로 용량을 넘기면 이렇게 끝납니다.

```text
3. 자식의 종료 상태: signal: 6 (SIGABRT)
   ...
   자식의 표준 에러: memory allocation of 64 bytes failed
```

패닉이 아니라 중단이므로 `catch_unwind`로도 붙잡을 수 없습니다. 같은 예제에서 `RUST_BACKTRACE=0`을 줬는데도 `note: run with RUST_BACKTRACE=1 …` 줄이 함께 나왔습니다.

**권장 코드: `try_reserve`로 먼저 확보합니다.** `Vec::try_reserve`와 `try_reserve_exact`(1.57)는 `reserve`의 실패할 수 있는 판이고 `Result<(), TryReserveError>`를 돌려줍니다. [`04_try_reserve.rs`](examples/04_try_reserve.rs)에서 확인한 성질입니다.

- 실패하면 `Err`를 돌려주고 벡터의 내용과 용량은 그대로입니다.
- 같은 요청에 `reserve`는 `capacity overflow`로 패닉합니다.
- 실패할 수 있는 `push`는 stable에 없습니다. 필요한 용량을 먼저 `try_reserve`로 확보하고 그 안에서만 `push`합니다(04의 `append_samples`, `tinyhex`의 `try_encode`). 05의 1번 구역은 확보한 용량 안에서 `push`하면 할당기를 부르지 않아 예산이 줄지 않는 것을 보여 줍니다.

```rust
pub fn try_encode(bytes: &[u8]) -> Result<String, HexError> {
    let mut s = String::new();
    s.try_reserve_exact(bytes.len() * 2).map_err(HexError::OutOfMemory)?;
    for &b in bytes {
        s.extend(digits(b)); // 확보한 용량 안이므로 할당하지 않음
    }
    Ok(s)
}
```

`TryReserveError`의 표시 문구는 원인에 따라 둘입니다. 요청한 크기가 컬렉션의 한도(`isize::MAX` 바이트)를 넘으면 할당기에 묻기 전에 `memory allocation failed because the computed capacity exceeded the collection's maximum`, 할당기가 거절하면 `memory allocation failed because the memory allocator returned an error`입니다. 원인을 고르는 `kind()` 메서드는 1.100.0에서 nightly 전용이므로 디버그 형식으로만 구별할 수 있습니다.

**할당기 API가 안정화되었습니다(1.100.0).** 05의 예산 할당기는 1.100.0에서 안정화된 `Allocator` 트레이트를 구현하고, `Vec::new_in(&budget)`으로 그 할당기를 쓰는 벡터를 만듭니다. 할당기를 받는 컬렉션이 생겨서 힙이 제한된 환경을 이렇게 흉내 내거나, 실제 장치에서 정해진 메모리 영역만 쓰게 할 수 있습니다. 로컬 문서로 확인한 현재 상태는 다음과 같습니다.

| API | 1.100.0 상태 |
| --- | --- |
| `Vec::try_reserve`, `try_reserve_exact`, `String::try_reserve`, `TryReserveError` | stable(1.57) |
| `Allocator`, `Global`, `AllocError`, `Vec::new_in`, `Box::new_in` | stable(1.100.0) |
| `TryReserveError::kind`, `Box::try_new`, `Vec::try_with_capacity`, `String::try_with_capacity`, `Vec::push_within_capacity` | nightly 전용 |
| `std::alloc::set_alloc_error_hook`(중단 대신 할 일 정하기) | nightly 전용 |

**운영체제가 있는 환경에서의 한계.** 이 환경(macOS)에서 `Vec<u8>`에 2^44바이트(16TiB)를 `try_reserve_exact`하자 `Ok`가 나왔고, 2^46바이트에서야 `Err`가 나왔습니다. 운영체제가 실제 메모리 없이 주소 공간만 내주기(오버커밋) 때문입니다. 이런 환경에서는 `try_reserve`가 성공해도 메모리가 있다는 보장이 없습니다. 그래도 04의 `prepare`처럼 **밖에서 온 크기**로 용량을 잡을 때는 터무니없는 값을 패닉 없이 거절하는 데 쓸모가 있습니다. 메모리를 실제로 건드리는 실험은 하지 않았습니다.

**실패하지 않는 API를 아예 없애기.** 책은 `no_global_oom_handling` 설정으로 실패하지 않는 할당 API를 끌 수 있다고 소개합니다. 1.100.0의 `alloc` 소스에서 `reserve` 등에는 `#[cfg(not(no_global_oom_handling))]`가 붙어 있습니다. 즉 `alloc`을 빌드할 때 이 cfg를 **설정하면** 그 API가 사라집니다. `alloc`을 직접 빌드해야 하므로 미리 빌드된 표준 라이브러리를 쓰는 이 저장소에서는 확인하지 못했습니다. 책은 이 작업의 동기로 Linux 커널 같은 사례를 듭니다.

**따르지 않아도 되는 경우.** 힙이 넉넉한 환경을 대상으로 하는 라이브러리라면 실패하지 않는 API가 훨씬 쓰기 편합니다. `tinyhex`처럼 두 판을 함께 제공하고 고르게 하는 방법도 있습니다. 책도 힙이 제한된 환경을 위한 라이브러리 지원은 아직 제한적이라고 봅니다.

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. 1.100.0에서 확인한 차이는 다음과 같습니다.

- **`Error`가 core에 있습니다.** 책은 `core`의 `Error`가 불안정하다고 각주에 적었습니다. `core::error::Error`는 1.81.0에서 안정화되었고(로컬 문서), `thiserror` 2를 `std` 기능 없이 써도 에러 타입이 `Error`를 구현합니다.
- **core로 들어온 항목이 늘었습니다.** `core::net`(1.77), `core::cell::LazyCell`(1.80)이 책의 기준 버전 뒤에 안정화되었습니다(예제 01).
- **Source 링크의 함정은 예가 바뀌었습니다.** 책 각주의 예(`Error`)는 이제 맞지 않고, 1.100.0에서는 `std::io`가 같은 함정입니다(`src/core/io/…`를 가리키지만 `core::io`는 nightly 전용, 4.3절).
- **할당기 API가 안정화되었습니다(1.100.0).** `Allocator`, `AllocError`, `Vec::new_in` 등이 stable이고, 예제 05가 이것으로 할당 실패를 결정적으로 재현합니다. 책 시점처럼 `Box::try_new`는 여전히 nightly 전용(`allocator_ext`)입니다. 책은 실패할 수 있는 할당이 대개 `AllocError`를 돌려준다고 쓰지만, `try_reserve` 계열이 돌려주는 것은 `TryReserveError`입니다(`AllocError`는 `Allocator`와 `Box::try_new`의 에러).
- **할당 실패의 동작.** 책은 "보통 패닉, 환경에 따라 다름"이라고 씁니다. 현재 문서와 예제 05에 따르면 `std`를 연결한 프로그램은 패닉이 아니라 중단이고(macOS에서 `signal: 6 (SIGABRT)`), 모든 크레이트가 `no_std`일 때만 패닉입니다.
- **`no_global_oom_handling`의 방향.** 책은 이 설정을 "끈다"고 표현하지만, `alloc` 소스의 조건은 `cfg(not(no_global_oom_handling))`이라서 cfg를 설정해야 실패하지 않는 API가 사라집니다(직접 빌드해 보지는 못함).
- **책에 없는 지침과 내용.** 다음은 책의 Item 33이 명시적으로 권하지 않는 것입니다(2026-11-20에 온라인판에서 확인).
  - 2.3절의 "크레이트 루트를 늘 `#![no_std]`로, `std`는 `extern crate std`로": Cargo Book의 Features 장(Feature unification 절의 예)과 `compile_fail/e0425_std_feature_prelude.rs`가 근거입니다. 책은 `#![cfg_attr(not(feature = "std"), no_std)]`만 듭니다.
  - 2.3절의 `std = ["alloc", "dep/std"]`, 기본 기능에 `std`, 기능으로 가린 배리언트의 `#[non_exhaustive]`: Cargo Book Features 장과 ch26이 근거입니다.
  - 지침 3 전체(호스트에서 테스트하는 방법과 `cfg_attr(not(test), no_std)`의 함정): 책은 CI에서 기능 조합을 빌드하라고만 합니다. `projects/tinyhex`와 사본에서의 실험이 근거입니다.
  - 2.5절의 호스트 보조 확인(`cargo tree -e features -i`, `#[panic_handler]` 검사 크레이트와 그 함정, 타깃별 API 차이): The Reference의 `no_std` 절, `rustc --print cfg`, `projects/leak_check`의 실험이 근거입니다.
  - 2.6절의 `Allocator`로 흉내 낸 제한된 힙, 오버커밋 관찰, API 상태 표: 1.100.0 로컬 문서와 릴리스 노트, 예제 04·05가 근거입니다.
  - 2.1절의 `std` 전용 항목과 대안 표 가운데 `f64` 수학 메서드, `std::io`, 시간·스레드 항목: 로컬 문서가 근거입니다.

### 2.8 기억할 것

- `std`의 많은 항목은 `core`나 `alloc`의 재내보내기입니다. 라이브러리는 `core::`·`alloc::` 경로로 쓰고 `HashMap`은 `BTreeMap`으로 바꾸면 대개 `no_std`가 됩니다. 의존성도 `no_std`여야 의미가 있습니다.
- `std`가 필요한 부분은 `std`(와 `alloc`) 기능으로 **더하고**, `no_std` 기능은 만들지 않습니다. 크레이트 루트를 늘 `#![no_std]`로 두면 모든 빌드의 프렐류드가 같습니다.
- 테스트는 호스트에서 돌리고(`#[cfg(test)] extern crate std;`, 통합 테스트), 기능 조합마다 빌드·clippy·테스트합니다.
- 호스트 빌드는 의존성이 `std`를 끌어오는 것을 잡지 못합니다. `std` 없는 타깃 빌드로 확인하고, 그것이 안 되면 `cargo tree -e features`와 `#[panic_handler]` 검사 크레이트로 보조합니다.
- 실패하지 않는 할당 API가 실패하면 `std` 프로그램은 중단됩니다. 힙이 제한된 환경에서는 `try_reserve`로 먼저 확보하고 그 안에서만 씁니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행하고, 모두 종료 코드 0으로 끝납니다. `projects/`의 두 프로젝트는 각자의 디렉터리에서 실행합니다. cargo가 출력하는 절대 경로는 `…/`로 줄였습니다.

| 파일 | 지침 | 보여 주는 것 | 관련 절 |
| --- | --- | --- | --- |
| [`01_where_items_live.rs`](examples/01_where_items_live.rs) | 1 | 같은 타입인지(`TypeId`), 정의된 크레이트(`type_name`), `core::io`의 함정 | 2.1 |
| [`02_no_std_imports.rs`](examples/02_no_std_imports.rs) | 1 | `#![no_std]` 파일에서 `alloc`·`core`로 가져오기 | 2.2 |
| [`03_hashmap_to_btreemap.rs`](examples/03_hashmap_to_btreemap.rs) | 1 | `HashMap`판과 `BTreeMap`판, 해셔의 시드 | 2.2 |
| [`04_try_reserve.rs`](examples/04_try_reserve.rs) | 5 | `try_reserve`와 `reserve`, 먼저 확보하고 `push` | 2.6 |
| [`05_limited_heap.rs`](examples/05_limited_heap.rs) | 5 | 예산 할당기, `Err`와 프로세스 중단 | 2.6 |
| [`projects/tinyhex`](projects/tinyhex) | 2, 3 | `core`·`alloc`·`std` 층으로 나눈 라이브러리, 호스트 테스트, 기능 조합 | 2.3, 2.4 |
| [`projects/leak_check`](projects/leak_check) | 4 | `std`를 쓰는 의존성, `cargo tree`, `#[panic_handler]` 검사 크레이트 | 2.5 |

### 01_where_items_live — 항목이 정의된 곳

```sh
cargo run -p er-ch33-no-std --example 01_where_items_live
```

```text
1. std 경로와 core/alloc 경로가 같은 타입인가
  dyn core::error::Error (1.81)                true
  core::net::Ipv4Addr (1.77)                   true
  core::cell::LazyCell (1.80)                  true
  core::cell::OnceCell (1.70)                  true
  core::ffi::CStr (1.64)                       true
  alloc::ffi::CString (1.64)                   true
  alloc::collections::TryReserveError (1.57)   true
2. type_name이 보여 주는 정의 위치
  core::time::Duration
  alloc::sync::Arc<u8>
  alloc::collections::btree::map::BTreeMap<u8, u8>
  std::collections::hash::map::HashMap<u8, u8>
  std::sync::poison::mutex::Mutex<u8>
  std::time::Instant
3. core에 정의됐지만 core 경로로는 아직 쓸 수 없는 것
  core::io::error::Error
  dyn core::io::write::Write
```

`type_name`의 문자열은 문서가 형식을 보장하지 않으므로 이 툴체인의 결과로만 읽습니다. 괄호 안의 버전은 출력용 문자열이고, 로컬 문서의 안정화 표시에서 옮겼습니다.

### 02_no_std_imports — `#![no_std]` 파일의 가져오기

```sh
cargo run -p er-ch33-no-std --example 02_no_std_imports
```

```text
품목 2개
  배: 5
  사과: 5
에러: 2번째 줄 "귤"은 '이름 수량' 모양이 아닙니다
에러: invalid digit found in string
```

실행 파일이라 `extern crate std;`로 `std`를 다시 연결했고 `main`은 `std::println!`으로 출력합니다. 이 줄을 지우면 `no global memory allocator found but one is required`, ``#[panic_handler]` function required, but not found``, `unwinding panics are not supported without std`로 빌드가 실패합니다(사본에서 확인). `std`를 연결해도 프렐류드는 `core`의 것이므로 `portable` 모듈의 가져오기는 하나도 뺄 수 없습니다.

### 03_hashmap_to_btreemap — 해시 맵을 B-트리 맵으로

```sh
cargo run -p er-ch33-no-std --example 03_hashmap_to_btreemap
```

```text
1. 정렬한 HashMap판 == BTreeMap판: true
2. BTreeMap의 순회 순서(키 순서)
  랙 1 슬롯 2: 3번
  랙 2 슬롯 1: 2번
  랙 3 슬롯 4: 1번
  랙 2의 경보 수: [2]
3. 시드가 다른 두 해셔가 같은 키에 같은 해시를 내는가: false
```

해시 값과 `HashMap`의 순회 순서는 실행마다 다를 수 있어 출력하지 않습니다. 3번 구역의 `false`는 64비트 해시가 우연히 같을 확률을 무시한 결과입니다.

### 04_try_reserve — 실패할 수 있는 판과 실패하지 않는 판

```sh
cargo run -p er-ch33-no-std --example 04_try_reserve
```

```text
1. prepare(5) = Ok(()), len 3, capacity 8
2. Err: memory allocation failed because the computed capacity exceeded the collection's maximum
   Debug: TryReserveError { kind: CapacityOverflow }
   버퍼는 그대로: [100, 101, 102], capacity 8
3. reserve는 패닉: "capacity overflow"
4. [16, 32, 48, 64], capacity 4
```

문서가 약속하는 것은 `capacity >= len + additional`뿐이고 1번의 8은 이 툴체인에서 본 값입니다. 3번은 패닉 훅을 잠시 비워 표준 에러에 아무것도 나오지 않게 했습니다.

### 05_limited_heap — 힙이 제한된 환경 흉내 내기

```sh
cargo run -p er-ch33-no-std --example 05_limited_heap
```

```text
1. try_reserve_exact(8) = Ok(()), 남은 예산 32바이트
   push 3번 뒤 남은 예산 32바이트, [21, 22, 23]
2. Err: memory allocation failed because the memory allocator returned an error
   Debug: TryReserveError { kind: AllocError { layout: Layout { size: 412, align: 4 (1 << 2) }, non_exhaustive: () } }
   [21, 22, 23], capacity 8
   Vec을 버린 뒤 남은 예산 64바이트
3. 자식의 종료 상태: signal: 6 (SIGABRT)
   성공? false / 종료 코드 None
   자식의 표준 출력: "push 8번 성공\n"
   자식의 표준 에러: memory allocation of 64 bytes failed
   자식의 표준 에러: note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

예제는 자기 자신을 `--abort-child` 인자로 다시 실행해 중단을 관찰합니다. 3번 구역의 종료 상태 표시는 macOS(유닉스)의 것입니다. 2번의 412바이트는 `try_reserve_exact(100)`이 `(len 3 + 100) × 4바이트`를 요청했다는 뜻입니다. `unsafe impl Allocator`는 받은 요청을 `Global`에 넘기기만 하며, Miri로 검사하지 못했습니다(nightly 없음). 안전성 주석은 파일에 있습니다.

### projects/tinyhex — 층으로 나눈 `no_std` 라이브러리

```sh
cd appd-a/ch33-no-std/projects/tinyhex
cargo test                          # 단위 5, 통합 3, 문서 1
cargo test --no-default-features    # 단위 3, 통합 1, 문서 1
cargo run --example dump
```

```text
52757374
Rust
InvalidData: 위치 4의 'z'는 16진수 숫자가 아닙니다
```

네 가지 기능 조합의 결과는 2.4절의 표와 같습니다. 의존성 `thiserror` 2.0.21과 그 전이 의존성은 `Cargo.lock`에 고정했습니다. `thiserror`의 `std` 기능을 누가 켰는지는 이렇게 봅니다.

```text
$ cargo tree -e features -i thiserror
thiserror v2.0.21
└── tinyhex v0.1.0 (…/projects/tinyhex)
    ├── tinyhex feature "alloc"
    │   └── tinyhex feature "std"
    │       └── tinyhex feature "default" (command-line)
    ├── tinyhex feature "default" (command-line)
    └── tinyhex feature "std" (*)
└── thiserror feature "std"
    └── tinyhex feature "std" (*)

$ cargo tree -e features -i thiserror --no-default-features --features alloc
thiserror v2.0.21
└── tinyhex v0.1.0 (…/projects/tinyhex)
    └── tinyhex feature "alloc" (command-line)
```

`--no-default-features`의 그래프에는 `thiserror feature "std"`가 없습니다. 거꾸로 따라가지 않는 `cargo tree -e features`(`-i` 없이)는 이 줄을 보여 주지 않았습니다. 그래프에 보이는 `thiserror-impl`, `syn`, `proc-macro2`는 `std`를 쓰지만 컴파일러 안에서 실행되는 절차적 매크로라서 대상 플랫폼에 연결되지 않습니다. `nostd-check`가 이들이 있는 그래프에서 통과하는 것으로 확인됩니다.

### projects/leak_check — `std`가 섞여 들어오는 것 확인하기

```sh
cd appd-a/ch33-no-std/projects/leak_check
cargo test                                      # sensor 2개(std 기본), stdful 0개
cargo test -p sensor --no-default-features      # sensor 1개
cd nostd-check
cargo build                                     # 통과
cargo build --features tinyhex/alloc            # 통과
cargo build --features sensor/std               # 실패(종료 코드 101): E0152, stdful
cargo tree -i stdful                            # 실패(종료 코드 101): 그래프에 없음
```

`cargo tree -i stdful`은 ``error: package ID specification `stdful` did not match any packages``로 끝납니다. 검사 크레이트의 그래프에 `stdful`이 없다는 뜻이고, `--features sensor/std`를 붙이면 `stdful ← sensor ← nostd-check`가 나옵니다. `nostd-check`는 `leak_check` 워크스페이스에서 뺀 별도 워크스페이스라서 그 디렉터리 안에서 실행합니다(2.5절).

## 4. 자주 하는 실수

`compile_fail/`의 파일은 `no_std` 실행 파일에 필요한 패닉 처리기 문제를 피하려고 라이브러리로 컴파일합니다. 확인 명령은 챕터 디렉터리에서 실행하며, 각 파일의 머리 주석에도 있습니다.

### 4.1 `alloc`의 이름을 가져오지 않음 — E0425 / `cannot find macro` / E0599

[`compile_fail/e0425_alloc_names_not_imported.rs`](compile_fail/e0425_alloc_names_not_imported.rs)는 `extern crate alloc;`만 적고 `Vec`, `String`, `vec!`, `format!`, `.to_string()`을 씁니다.

```sh
rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0425_alloc_names_not_imported.rs
```

```text
error: cannot find macro `vec` in this scope
  --> compile_fail/e0425_alloc_names_not_imported.rs:21:22
   |
21 |     let mut picked = vec![];
   |                      ^^^
   |
help: consider importing this macro
   |
17 + use alloc::vec;
   |
error[E0425]: cannot find type `Vec` in this scope
  --> compile_fail/e0425_alloc_names_not_imported.rs:20:46
   |
20 | pub fn over(readings: &[u32], limit: u32) -> Vec<u32> {
   |                                              ^^^ not found in this scope
   |
help: consider importing this struct
   |
17 + use alloc::vec::Vec;
   |
```

`format!`(`use alloc::format;`), `String`(`use alloc::string::String;`)도 같은 모양이고, `to_string`은 E0599와 ``trait `ToString` which provides `to_string` is implemented but not in scope``로 실패합니다.

**원인.** `extern crate alloc;`은 크레이트를 연결할 뿐 그 안의 이름을 프렐류드에 넣지 않습니다. `#![no_std]` 크레이트의 프렐류드는 `core`의 것입니다. 매크로는 이름을 못 찾을 때 에러 코드가 없습니다.

**해결.** help가 제안하는 대로 하나씩 가져옵니다. `extern crate alloc;` 줄마저 없으면 같은 에러에서 help가 사라지므로(사본에서 확인), 먼저 그 줄이 있는지 봅니다.

### 4.2 `std` 기능을 켠 빌드에서만 컴파일됨 — E0425

[`compile_fail/e0425_std_feature_prelude.rs`](compile_fail/e0425_std_feature_prelude.rs)는 `#![cfg_attr(not(feature = "std"), no_std)]` 크레이트에서 `Vec`을 가져오지 않고 씁니다.

```sh
rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> --cfg 'feature="std"' compile_fail/e0425_std_feature_prelude.rs   # 통과
rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0425_std_feature_prelude.rs                         # 실패
```

```text
error[E0425]: cannot find type `Vec` in this scope
  --> compile_fail/e0425_std_feature_prelude.rs:23:37
   |
23 | pub fn nonzero(readings: &[u16]) -> Vec<u16> {
   |                                     ^^^ not found in this scope
   |
help: consider importing this struct
   |
20 + use alloc::vec::Vec;
   |
```

**원인.** `std` 기능이 켜지면 `no_std`가 빠져 `std` 프렐류드가 들어옵니다. `--cfg 'feature="std"'`는 Cargo가 `--features std`일 때 넘기는 인자와 같습니다. 기본 기능이 `std`라면 평소 빌드에서는 이 실수가 보이지 않습니다.

**해결.** `use alloc::vec::Vec;`을 적습니다. 다시 생기지 않게 하려면 크레이트 루트를 늘 `#![no_std]`로 두고 `std`를 `#[cfg(feature = "std")] extern crate std;`로 연결합니다(2.3절). 그러면 기본 빌드에서도 바로 E0425가 납니다.

### 4.3 정의 위치만 보고 `core::io`를 씀 — E0658

[`compile_fail/e0658_core_io_unstable.rs`](compile_fail/e0658_core_io_unstable.rs)는 `std::io::Write`가 `core::io`에 정의된 것을 보고 `core::io::Write`를 받습니다.

```text
error[E0658]: use of unstable library feature `core_io`
  --> compile_fail/e0658_core_io_unstable.rs:16:22
   |
16 | pub fn dump<W: core::io::Write>(out: &mut W, readings: &[u8]) {
   |                      ^^
   |
   = note: see issue #154046 <https://github.com/rust-lang/rust/issues/154046> for more information
```

**원인.** 정의된 위치(`type_name`, 문서의 Source 링크)와 안정화 여부는 별개입니다. 1.100.0에서 `core::io`는 nightly 전용입니다.

**해결.** stable에서 입출력은 `std` 기능 뒤에 둡니다(`tinyhex`의 `src/io.rs`). `no_std`에서는 바이트 슬라이스나 `core::fmt::Write`(문자열 출력)를 받는 API로 설계합니다.

### 4.4 `f64::sqrt`가 없음 — E0599

[`compile_fail/e0599_f64_sqrt_in_no_std.rs`](compile_fail/e0599_f64_sqrt_in_no_std.rs)는 가속도의 크기를 `sqrt`로 구합니다.

```text
error[E0599]: no method named `sqrt` found for type `f64` in the current scope
  --> compile_fail/e0599_f64_sqrt_in_no_std.rs:17:29
   |
17 |     (x * x + y * y + z * z).sqrt()
   |                             ^^^^ method not found in `f64`
```

**원인.** `f64`는 `core`의 원시 타입이지만 `sqrt`, `floor`, `round`, `powi`, `sin` 같은 메서드는 `std`가 더한 `impl` 블록에 있습니다. 1.100.0 로컬 문서의 `core`판 `f64` 페이지에는 `abs`, `clamp`, `total_cmp` 등만 있습니다. 컴파일러는 메서드가 `std`에 있다고 알려 주지 않습니다.

**해결.** `core::f64::math`는 nightly 전용(`core_float_math`)입니다. stable에서는 `libm` 같은 크레이트를 쓰거나, 제곱근이 필요 없게 바꿉니다(크기를 비교만 한다면 제곱끼리 비교).

### 4.5 그 밖의 실수

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| `#![no_std]`에서 `std::time::Duration` | E0433 ``cannot find module or crate `std` ``, help가 `use core::time::Duration;` 제안 | `core::` 경로로 | [`e0433_std_path_in_no_std.rs`](compile_fail/e0433_std_path_in_no_std.rs) |
| `alloc::collections::HashMap` 가져오기 | E0432 ``no `HashMap` in `collections` ``, 대안 제안 없음 | `BTreeMap`(키에 `Ord`) | [`e0432_hashmap_not_in_alloc.rs`](compile_fail/e0432_hashmap_not_in_alloc.rs), 03 |
| `#![no_std]` 라이브러리에서 `println!` | ``cannot find macro `println` ``(에러 코드 없음) | 출력은 부르는 쪽에 맡기고 `Display`를 구현 | 2.2 |
| `no_std` 기능으로 `std`를 끔 | 다른 크레이트가 켜면 `std` 사용자의 API가 사라짐 | `std` 기능으로 더함 | 2.3, ch26 |
| `#![no_std]` 크레이트의 테스트에서 `vec!` | ``cannot find macro `vec` `` | `#[cfg(test)] extern crate std;`와 `use std::vec;` | 2.4 |
| 의존성의 기본 기능(`std`)을 끄지 않음 | 호스트에서는 아무 증상 없음 | `default-features = false`, `cargo tree -e features -i` | 2.5 |
| 검사 크레이트에서 검사 대상을 가리키지 않음 | `std`가 섞여도 통과, `unused dependency` 경고 | `pub use 대상;` | 2.5 |

## 5. 연습 문제

### 연습 1. 할당 없는 인코딩 더하기

[`projects/tinyhex`](projects/tinyhex)의 `src/slice.rs`에 `core`만으로 동작하는 `encode_to_slice(bytes: &[u8], out: &mut [u8]) -> Result<&str, HexError>`를 더하세요. 버퍼가 작으면 `BufferTooSmall`을 돌려줘야 합니다. 테스트를 더하고 2.4절의 네 가지 조합으로 검사하세요.

<details>
<summary>힌트</summary>

`decode_to_slice`처럼 `out.get_mut(..bytes.len() * 2)`로 필요한 앞부분을 잘라 내고, `as_chunks_mut::<2>()`로 두 바이트씩 나눠 `digits(b)`의 두 글자를 `u8`로 바꿔 넣습니다. 채운 부분을 `core::str::from_utf8`로 `&str`로 바꿉니다. 넣은 것이 모두 ASCII 숫자라 실패하지 않지만 `Result`를 돌려주므로 어떻게 처리할지 정해야 합니다([ch18](../ch18-dont-panic/README.md)). 함수와 테스트는 기능으로 가리지 않으므로 `--no-default-features`에서 단위 테스트가 하나 늘어야 합니다.

</details>

### 연습 2. `sensor`에 `alloc` 기능 더하기

[`projects/leak_check`](projects/leak_check)의 `sensor`에 읽기 값들의 원시 값을 16진수 문자열 목록(`Vec<String>`)으로 돌려주는 함수를 더하세요. 힙이 필요하므로 `alloc` 기능 뒤에 둡니다. `std` 기능과의 관계는 어떻게 적어야 하나요? `nostd-check`에서 무엇으로 확인하나요?

<details>
<summary>힌트</summary>

`[features]`에 `alloc = ["tinyhex/alloc"]`을 더하고 `std = ["alloc", "dep:stdful"]`로 바꿉니다. `lib.rs`에 `#[cfg(feature = "alloc")] extern crate alloc;`을 두고, 함수는 `tinyhex::encode`로 만든 문자열을 `alloc::vec::Vec<alloc::string::String>`으로 모읍니다. `sensor` 디렉터리에서 세 조합(`--no-default-features`, `--no-default-features --features alloc`, 기본)으로 clippy를 돌리고, `nostd-check`에서 `cargo build --features sensor/alloc`이 통과하는지(힙은 써도 `std`는 연결하지 않음), `--features sensor/std`가 여전히 `stdful`을 가리키며 실패하는지 확인합니다.

</details>

### 연습 3. 예산과 `push` 횟수 예측하기

[`05_limited_heap.rs`](examples/05_limited_heap.rs)의 자식 프로세스에서 예산을 96바이트, 128바이트로 바꾸면 몇 번째 `push`에서 중단될까요? 먼저 계산하고, 바꿔서 실행해 확인하세요.

<details>
<summary>힌트</summary>

이 툴체인에서 `u32` 벡터의 용량은 0 → 4 → 8 → 16 → 32로 늘었습니다. 용량을 늘리는 기본 구현은 새 블록을 먼저 할당하고 옛 블록을 나중에 해제하므로, 늘리는 순간에는 두 블록의 합이 예산 안에 있어야 합니다. 용량 8(32바이트)을 쥔 채 16(64바이트)을 요청하려면 96바이트가 필요하므로, 96바이트 예산에서는 16번까지 성공하고 17번째에서 멈춥니다. 128바이트도 16번입니다. 용량 32(128바이트)로 늘리려면 64 + 128 = 192바이트가 필요하기 때문입니다. 확인할 때는 자식이 출력하는 `push 8번 성공`의 8도 바꿔야 합니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 33: Consider making library code `no_std` compatible](https://effective-rust.com/no-std.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- The Reference, [The `no_std` attribute](https://doc.rust-lang.org/reference/names/preludes.html#the-no_std-attribute): `std`를 자동으로 연결하지 않고 `core` 프렐류드를 씀, `no_std`가 `std` 연결을 막지 않는다는 경고
- [`core` 크레이트 문서](https://doc.rust-lang.org/core/index.html)와 [`alloc` 크레이트 문서](https://doc.rust-lang.org/alloc/index.html)의 첫 페이지
- Cargo Book, [Features](https://doc.rust-lang.org/cargo/reference/features.html)의 [Feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification): `no_std` 기능 대신 `std` 기능, `#![no_std]` + `#[cfg(feature = "std")] extern crate std;`의 예
- The Embedded Rust Book, [A `no_std` Rust Environment](https://docs.rust-embedded.org/book/intro/no-std.html)
- std 문서: [`Vec::try_reserve`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.try_reserve), [`TryReserveError`](https://doc.rust-lang.org/std/collections/struct.TryReserveError.html), [`Allocator`](https://doc.rust-lang.org/std/alloc/trait.Allocator.html), [`handle_alloc_error`](https://doc.rust-lang.org/std/alloc/fn.handle_alloc_error.html)
- 로컬 문서의 릴리스 노트(`rustc --print sysroot` 아래 `share/doc/rust/html/releases.md`): 1.100.0의 "Stabilize custom allocators"와 Stabilized APIs 목록

위 문서는 온라인판 대신 rustc 1.100.0과 함께 설치된 로컬 사본(`share/doc/rust/html`)으로 확인했습니다. 기준 문서만 온라인판을 읽었습니다(2026-11-20).
