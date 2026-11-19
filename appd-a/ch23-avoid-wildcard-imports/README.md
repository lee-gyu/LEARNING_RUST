# ch23. 와일드카드 가져오기 피하기

`use templog::*;`처럼 경로 끝에 `*`를 붙이면 그 모듈의 공개 항목이 전부 내 스코프에 들어옵니다. 이것을 와일드카드 가져오기(wildcard import), 또는 본편의 표기대로 글롭 가져오기(glob import)라고 하며, 이 챕터에서는 이후 "글롭"이라고 부릅니다. 글롭은 편하지만 "무엇이 들어오는가"를 내가 아니라 그 모듈을 관리하는 사람이 정합니다. 의존 크레이트가 마이너 버전에서 공개 항목을 하나 더하면, 내 코드는 한 글자도 바꾸지 않았는데 컴파일이 멈추거나, 더 나쁘게는 컴파일은 되면서 다른 함수를 부르게 될 수 있습니다.

글롭의 문법은 본편에서 배웠으므로, 이 챕터는 **언제 글롭을 쓰지 말고 언제 써도 되는지**와 그 판단의 근거가 되는 이름 해석 규칙만 다룹니다.

- 원제: [Item 23: Avoid wildcard imports](https://effective-rust.com/wildcard.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch23-avoid-wildcard-imports`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 본편 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md) 2.8~2.10절: 같은 이름을 두 번 `use`한 E0252, 글롭, `pub use` 재내보내기
- [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.6절: 트레이트는 스코프에 있어야 메서드를 쓸 수 있음
- [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md) 2.8절: 테스트 모듈의 `use super::*`
- [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md) 2.2절: 프렐류드
- [Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.2절: 메서드 탐색 순서와, 프렐류드에 트레이트가 더해질 때 생기는 일

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 의존 크레이트가 마이너 버전에서 공개 항목을 더했을 때 글롭으로 가져온 코드가 **그대로**인지, **컴파일 에러**인지, **조용히 다른 것을 부르게** 되는지를 스코프와 메서드 탐색 규칙으로 설명할 수 있다.
2. 글롭을 쓰는 이름만 나열한 명시적 가져오기로 바꿔, 의존성의 두 버전에서 같은 결과를 내는 코드를 작성할 수 있다.
3. 글롭이 원인인 E0034, E0659, E0107을 알아보고, 컴파일러의 help 가운데 무엇을 따를지 판단할 수 있다.
4. 글롭을 써도 되는 경우(테스트의 `use super::*`, 내 크레이트 안 모듈의 `pub use` 재내보내기, 프렐류드)를 가려내고, 글롭을 남겨야 할 때 의존성 버전을 고정하는 이유를 설명할 수 있다.

## 2. 핵심 개념

2.1절에서 원리를 표로 정리하고, 2.2~2.3절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 예제는 모두 이 저장소에서 지어낸 온도 기록 라이브러리 `templog`를 씁니다. 1.0.0에는 `Log`와 `mean`이 있고, 1.1.0은 1.0.0의 항목을 그대로 둔 채 새 항목을 **더하기만** 합니다.

### 2.1 글롭으로 들어온 이름은 언제 지고 언제 이기는가

시맨틱 버저닝에서 공개 항목을 더하는 것은 마이너 변경입니다. 그리고 `Cargo.toml`에 `templog = "1.0.0"`이라고 적으면 Cargo는 이것을 "1.0.0 이상, 2.0.0 미만"으로 읽어 1.1.0도 받아들입니다(버전 요구 사항 문법은 [ch21](../ch21-semver/README.md)). 그래서 1.1.0이 나오면 내 코드를 고치지 않아도 다음 의존성 갱신 때 1.1.0으로 빌드됩니다. 명시적으로 가져온 코드에는 새 이름이 들어오지 않지만, 글롭으로 가져온 코드에는 새 공개 이름이 모두 들어옵니다. 그 새 이름이 내 코드의 이름과 겹칠 때 무슨 일이 생기는지를 1.100.0에서 직접 컴파일해 정리했습니다.

| templog 1.1.0이 더한 것 | 내 코드에 이미 있던 같은 이름 | 결과 | 확인한 곳 |
| --- | --- | --- | --- |
| 함수·타입 | 같은 모듈에 직접 정의한 항목 | 지역 정의가 이김. 그대로 | 예제 01 (가), 프로젝트의 `Alert` |
| 함수·타입 | 명시적 `use`로 가져온 항목 | 명시적 가져오기가 이김. 그대로 | 예제 01 (나) |
| 함수·타입 | 바깥 모듈에 정의한 항목(글롭은 함수 본문 안) | **글롭이 이김. 조용히 바뀜** | 예제 01 (다) |
| 함수·타입 | 다른 글롭으로 들어오던 항목 | 그 이름을 **쓰는 곳에서** E0659 | 4.2절 |
| 타입 별칭 `Result` | 프렐류드의 `Result` | 글롭이 프렐류드를 가려 E0107 | 4.3절 |
| 트레이트와 그 구현 | 내 트레이트의 메서드, 수신자 모양이 같음 | E0034 | 프로젝트, 4.1절 |
| 트레이트와 그 구현 | 내 트레이트의 메서드, 내 것은 `&mut self`, 새 것은 `&self` | **새 메서드가 불림. 조용히 바뀜** | 예제 02 |
| 트레이트와 그 구현 | 내 트레이트의 메서드, 내 것은 `&self`, 새 것은 `&mut self` | 내 메서드가 불림. 그대로 | 같은 모양의 코드 |
| 트레이트와 그 구현 | 고유 메서드, 수신자 모양이 같음 | 고유 메서드가 이김. 그대로 | 같은 모양의 코드 |
| 트레이트와 그 구현 | 고유 메서드, 고유 것은 `&mut self`, 새 것은 `&self` | **새 메서드가 불림. 조용히 바뀜** | 같은 모양의 코드 |

표의 결과는 세 가지 규칙에서 나옵니다.

**규칙 1: 이름은 안쪽 스코프부터 찾고, 한 스코프 안에서는 글롭이 가장 약합니다.** Reference는 같은 이름 공간에서 직접 정의한 항목과 이름을 적은 `use`가 글롭으로 들어온 이름을 가린다고 정합니다. 그래서 같은 모듈 안에서는 1.1.0이 무엇을 더해도 내 정의와 명시적 가져오기가 이깁니다(표의 1, 2행). 하지만 이것은 **같은 스코프 안에서의** 순위입니다. 함수 본문은 모듈보다 안쪽 스코프라서, 본문에 `use templog::*;`를 쓰면 그 글롭이 바깥 모듈에 정의한 `format_temp`보다 먼저 찾아집니다(3행). 반대로 프렐류드는 가장 바깥이라 글롭에 가려집니다(5행). [Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.2절에서 프렐류드에 타입이나 함수가 더해져도 기존 코드가 깨지지 않은 것은 내가 정의하거나 가져온 이름이 프렐류드보다 우선하기 때문이었습니다. 글롭으로 들어온 이름도 프렐류드보다 우선하므로, 의존성이 프렐류드와 같은 이름을 더하면 내가 쓰던 프렐류드의 이름이 가려집니다.

**규칙 2: 두 글롭이 같은 이름을 들여오는 것은 그 이름을 쓰기 전까지 에러가 아닙니다.** Reference는 글롭끼리 겹친 이름으로는 이름을 해석할 수 없고, 에러는 가져올 때가 아니라 쓸 때 난다고 정합니다(4행). 겹친 이름을 쓰지 않는 모듈은 아무 일 없이 컴파일되므로, 같은 업데이트가 어떤 파일은 깨고 어떤 파일은 그대로 둡니다.

**규칙 3: 트레이트는 스코프에 들어오는 순간 메서드 탐색의 후보가 됩니다.** 글롭은 타입과 함수뿐 아니라 트레이트도 들여옵니다. [Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.2절의 탐색 순서대로 컴파일러는 수신자를 `T`, `&T`, `&mut T` 단계로 맞춰 보고, 한 단계 안에서는 고유 메서드를 트레이트 메서드보다 먼저 봅니다. 같은 단계에 트레이트 메서드가 둘이면 E0034이고(6행), 새 트레이트의 메서드가 **더 이른 단계**에 있으면 에러 없이 그쪽이 불립니다(7행, 10행). [ch13](../ch13-default-implementations/README.md) 2.5절에서 "고유 메서드가 이기는 것은 수신자 모양이 같을 때"라고 확인한 것과 같은 규칙이고, 받는 인자의 개수나 타입이 달라도 이름만 같으면 후보가 됩니다(1.100.0에서 확인).

명시적 가져오기는 새 트레이트가 내 스코프에 들어오는 길을 막으므로 규칙 3의 문제도 함께 막습니다. 다만 의존성이 **자기 타입에 고유 메서드**를 더하는 경우는 가져오기와 무관하게 탐색 후보가 됩니다. Cargo Book의 SemVer 호환성 장이 이것을 "깨질 수도 있는 변경"으로 따로 다룹니다.

### 2.2 지침 1: 통제하지 않는 크레이트는 글롭으로 가져오지 않는다

**문제 상황.** 사용자는 templog 1.0.0에 맞춰 프로그램을 썼습니다. 경보를 나타내는 자기 타입 `Alert`를 정의했고, 확장 트레이트 `LogExt`에 표준편차를 구하는 `spread(&self)`를 만들었습니다. 그 뒤 templog 1.1.0이 공개 타입 `Alert`와, 일교차를 구하는 `spread(&self)`를 가진 트레이트 `Stats`(와 `Log`에 대한 구현)를 더했습니다. [`projects/glob_upgrade`](projects/glob_upgrade)는 이 두 버전을 path 의존성(crates.io 대신 디스크의 디렉터리에서 가져오는 의존성)으로 두고, 같은 사용자 코드를 두 버전과 각각 빌드합니다.

**비권장 코드.** [`shared/glob_import.rs`](projects/glob_upgrade/shared/glob_import.rs)는 글롭으로 가져옵니다.

```rust
use templog::*;

struct Alert { message: String }          // 1.1.0의 Alert와 이름이 같음
trait LogExt { fn spread(&self) -> f64; } // 1.1.0의 Stats::spread와 이름이 같음
// ...
println!("  평균 {m:.2}도, 표준편차 {:.2}도", log.spread());
```

1.0.0과는 빌드되지만 1.1.0과 빌드하면 `log.spread()`가 E0034로 멈춥니다(전체 메시지는 4.1절). `Alert`는 규칙 1대로 지역 정의가 이겨서 에러가 나지 않습니다. 책이 말하듯 타입 이름이 겹치는 것만으로는 깨지지 않고, 진짜 위험은 트레이트 메서드입니다.

컴파일러가 멈추는 경우는 차라리 다행입니다. [`02_new_trait_method.rs`](examples/02_new_trait_method.rs)에서 사용자의 `LogExt::calibrate(&mut self)`는 기록을 제자리에서 보정하는데, 1.1.0이 "보정한 사본을 돌려주는" `Calibrated::calibrate(&self) -> Log`를 더합니다. 글롭으로 가져온 코드의 `log.calibrate(-0.5);`는 `&Log` 단계에서 새 메서드를 먼저 찾아 사본을 만들고 버립니다. 컴파일 에러가 없고, 결과만 달라집니다.

```text
  글롭, templog 1.0:   21.00도
  글롭, templog 1.1:   21.50도
```

이 예제에서 유일한 단서는 사용자의 `use crate::LogExt;`가 더는 쓰이지 않는다는 `unused_imports` 경고입니다. 같은 모듈에서 `LogExt`의 다른 메서드를 하나라도 부르면 이 경고마저 나지 않습니다(연습 2). [`01_glob_priority.rs`](examples/01_glob_priority.rs)의 (다)는 함수와 함수의 겹침에서 같은 일이 생기는 모습입니다. 본문 안의 글롭이 바깥 모듈의 `format_temp`를 가려 `21.5도`가 `21.5°C`로 바뀝니다.

**권장 코드.** 쓰는 이름만 가져옵니다. [`shared/explicit_import.rs`](projects/glob_upgrade/shared/explicit_import.rs)는 `glob_import.rs`와 이 한 줄만 다릅니다.

```rust
use templog::{Log, VERSION, mean};
```

`Stats`를 가져오지 않았으므로 1.1.0에서도 `spread` 후보는 `LogExt::spread` 하나뿐입니다. 세 조합 모두 같은 결과를 냅니다.

```text
[glob-on-v1-0] templog 1.0.0
  평균 21.50도, 표준편차 2.21도
[explicit-on-v1-0] templog 1.0.0
  평균 21.50도, 표준편차 2.21도
[explicit-on-v1-1] templog 1.1.0
  평균 21.50도, 표준편차 2.21도
```

(경보 줄은 생략했습니다. 전체 출력은 3절.) 새 기능을 쓰고 싶어지면 그때 `use templog::Stats;`처럼 이름을 적어 들여옵니다. 그 순간 충돌이 생기더라도 그것은 내가 고친 커밋에서 드러나므로 원인을 바로 압니다.

**지침을 따르지 않아도 되는 경우.** 책은 세 가지를 듭니다. 공통점은 글롭이 가리키는 모듈의 내용을 내가 정하거나, 글롭으로 가져가라고 골라 둔 목록이라는 것입니다. [`03_allowed_globs.rs`](examples/03_allowed_globs.rs)가 세 경우를 모두 보여 줍니다.

- **테스트 모듈의 `use super::*`.** 부모 모듈은 내 코드이고, 테스트는 부모의 비공개 함수까지 써야 하므로 글롭이 알맞습니다. 한 가지 덧붙이면 `use super::*`는 부모가 **가져온** 이름(부모의 글롭으로 들어온 이름 포함)까지 들여옵니다(1.100.0에서 확인). 그러니 테스트 모듈은 부모 모듈만큼 안전하고, 부모가 외부 크레이트를 글롭으로 가져오면 그 위험도 함께 물려받습니다.
- **내 크레이트 안 모듈을 `pub use`로 재내보내기.** 예제의 `units`는 `celsius`, `fahrenheit` 하위 모듈을 `pub use self::celsius::*;`로 모아 바깥에 `units::to_kelvin`처럼 보여 줍니다. 두 하위 모듈에 같은 이름이 생겨도 내 빌드에서 바로 드러나므로, 남이 배포한 업데이트가 내 코드를 깨는 일은 없습니다.
- **프렐류드.** `std::io::prelude`의 문서는 이 모듈이 "I/O를 많이 쓰는 모듈의 맨 위에 글롭 하나를 더해" 자주 쓰는 트레이트의 가져오기를 줄이려고 있다고 설명하고, 내용은 `BufRead`, `Read`, `Seek`, `Write` 네 트레이트뿐입니다. 크레이트의 프렐류드는 함께 쓰는 핵심 타입과 트레이트만 골라 둔 목록이라(예제의 `templog::prelude`에는 `mean`이 없습니다) 무엇이 더해질지 작성자가 신중히 정합니다. 그래도 글롭이므로 위험이 0은 아니고, 편의가 작은 위험보다 크다고 판단하는 것입니다.

Clippy의 `wildcard_imports` 린트가 같은 기준을 씁니다. 이 린트는 pedantic 그룹이라 기본으로 꺼져 있으며, 켜면 `use templog::*;`에는 경고하고 경로에 `prelude`가 든 글롭, `pub use` 재내보내기, 테스트 모듈의 `use super::*`에는 경고하지 않습니다. 예제 03은 이 린트를 켜 두었는데도 경고가 없습니다. 다만 1.100.0에서 확인해 보니 내 크레이트의 모듈이라도 `pub` 없이 `use units::*;`로 가져오면 경고하고, 테스트가 아닌 모듈의 `use super::*`에도 경고합니다. 린트를 프로젝트 단위로 켜는 방법은 `ch29-listen-to-clippy`에서 다룹니다.

### 2.3 지침 2: 글롭을 남긴다면 그 의존성의 버전을 고정한다

**문제 상황.** 수많은 이름을 가져다 써서 당장 명시적 가져오기로 바꾸기 어렵거나, 프렐류드가 없는 크레이트를 글롭으로 써야 할 때가 있습니다. 2.1절에서 본 대로 `templog = "1.0.0"`은 1.1.0을 받아들이므로, 글롭을 둔 채로는 다음 갱신이 언제든 코드를 깰 수 있습니다.

**비권장 코드.** 글롭으로 가져오면서 기본 버전 요구 사항을 그대로 둡니다.

```toml
[dependencies]
templog = "1.0.0"   # 1.0.0 이상 2.0.0 미만: 1.1.0도 받아들임
```

**권장 코드.** 책은 이 경우 그 의존성을 정확한 버전으로 고정하라고 권합니다. `=`를 붙이면 그 버전만 받아들입니다.

```toml
[dependencies]
templog = "=1.0.0"  # 1.0.0만 받아들임
```

path 의존성으로도 확인할 수 있습니다. path 의존성에 `version`을 함께 적으면 Cargo는 그 디렉터리의 패키지 버전이 요구 사항을 만족하는지 검사합니다. 일부러 실패하는 패키지 `glob-on-v1-1-fails`의 의존성을 `{ path = "../templog-1.1.0", version = "=1.0.0" }`으로 바꾸면 E0034까지 가지 않고 의존성 해석에서 멈춥니다(경로는 `<저장소>`로 줄였습니다).

```text
error: failed to select a version for the requirement `templog = "=1.0.0"`
candidate versions found which didn't match: 1.1.0
location searched: <저장소>/appd-a/ch23-avoid-wildcard-imports/projects/glob_upgrade/templog-1.1.0
```

`version = "1.0.0"`(`=` 없음)으로 바꾸면 1.1.0을 받아들여 다시 E0034가 납니다. 같은 요구 사항으로 2.0.0 패키지를 가리키면 ``failed to select a version for the requirement `templog = "^1.0.0"` ``으로 거부되는데, Cargo가 `=` 없는 요구 사항을 `^`(캐럿) 요구 사항으로 읽는다는 것이 메시지에 드러납니다.

**지침을 따르지 않아도 되는 경우.** 고정은 글롭의 위험을 막는 대신, 그 의존성의 버그 수정과 보안 수정도 자동으로 받지 못하게 합니다. 그래서 고정은 명시적 가져오기로 바꿀 수 없을 때의 차선책이고, 명시적으로 가져오는 의존성은 고정할 이유가 없습니다. 고정의 비용과 `Cargo.lock`의 역할은 [ch25](../ch25-dependency-graph/README.md)에서 다룹니다.

### 2.4 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. 이 챕터의 예제와 `compile_fail/` 파일을 `rustc --edition 2018`로도 컴파일해 보았고, 실행 결과와 에러 코드가 2024 에디션과 같았습니다. 그 밖에 1.100.0에서 확인한 차이는 다음과 같습니다.

- **책에 없는 지침.** 다음은 책의 Item 23이 직접 다루지 않는 내용입니다(2026-11-19에 온라인판에서 확인). 책은 지역 정의가 글롭보다 우선한다는 것, 새 트레이트로 메서드 호출이 E0034가 된다는 것, 세 가지 예외, 버전 고정까지 다룹니다.
  - 함수 본문 안의 글롭이 바깥 모듈의 정의를 이기는 경우(2.1절 규칙 1): Reference의 "Use declarations"(글롭은 같은 이름 공간의 항목과 명시적 가져오기에 가려짐)와 예제 01이 근거입니다.
  - 새 트레이트가 E0034 대신 조용히 다른 메서드를 부르게 하는 경우(규칙 3): 예제 02와, 같은 모양의 코드로 고유 메서드·트레이트 메서드와 수신자 모양을 조합해 컴파일한 결과가 근거입니다.
  - 두 글롭이 같은 이름을 들여올 때 쓰는 곳에서만 E0659가 나는 것(규칙 2): Reference의 "Name resolution"과 `compile_fail/e0659_two_glob_imports.rs`가 근거입니다.
  - 글롭이 프렐류드의 `Result`를 가려 E0107이 나는 것: `compile_fail/e0107_glob_shadows_prelude.rs`가 근거입니다.
  - `use super::*`가 부모 모듈이 가져온 이름까지 들여온다는 단서: 같은 모양의 테스트 코드로 확인했습니다.
  - Clippy의 `wildcard_imports`와, 열거형 배리언트를 `use Level::*`처럼 가져오는 것을 잡는 `enum_glob_use`(둘 다 pedantic): `cargo clippy --explain`과 `clippy-driver -W help`의 출력이 근거입니다.
- **E0034의 두 번째 후보.** 프로젝트에서 새 트레이트가 다른 크레이트에 있으면 두 번째 후보는 소스 위치 없이 ``= note: candidate #2 is defined in an impl of the trait `templog::Stats` …`` 한 줄로만 나옵니다. help는 책의 설명대로 후보마다 트레이트 경로를 적은 호출(`LogExt::spread(&log)`, `templog::Stats::spread(&log)`)을 제안합니다.

### 2.5 기억할 것

- 의존성이 공개 항목을 더하는 것은 마이너 변경이고 Cargo는 기본으로 이를 받아들입니다. 글롭은 그 새 이름을 모두 내 스코프에 들입니다.
- 같은 스코프 안에서는 내 정의와 명시적 가져오기가 글롭을 이기지만, 함수 본문 안의 글롭은 바깥의 정의를 이기고 프렐류드의 이름도 가립니다.
- 글롭이 들여온 새 트레이트는 같은 수신자 단계의 메서드와 부딪쳐 E0034를 내고, 더 이른 단계에 있으면 에러 없이 호출을 가져갑니다.
- 통제하지 않는 크레이트는 쓰는 이름만 명시적으로 가져옵니다. 테스트의 `use super::*`, 내 모듈의 `pub use` 재내보내기, 프렐류드는 예외입니다.
- 글롭을 남겨야 한다면 그 의존성을 `=`로 고정하되, 차선책이라는 것을 기억합니다.

## 3. 예제 안내

예제는 저장소 루트에서 실행합니다. 01과 02는 의존 크레이트의 두 버전을 한 파일 안의 모듈 `templog_1_0`, `templog_1_1`로 흉내 내고(1.1은 `pub use super::templog_1_0::*;`로 1.0의 항목을 그대로 내보낸 뒤 새 항목을 더함), 같은 사용자 코드를 두 버전에 대해 실행합니다. 이름을 찾는 규칙은 모듈과 크레이트에서 같습니다.

| 파일 | 보여 주는 것 | 관련 절 |
| --- | --- | --- |
| [`01_glob_priority.rs`](examples/01_glob_priority.rs) | 같은 모듈의 정의, 명시적 가져오기, 함수 안의 글롭 | 2.1, 2.2 |
| [`02_new_trait_method.rs`](examples/02_new_trait_method.rs) | 새 트레이트가 에러 없이 호출을 가져가는 경우와 명시적 가져오기 | 2.1, 2.2 |
| [`03_allowed_globs.rs`](examples/03_allowed_globs.rs) | 글롭을 써도 되는 세 경우(clippy 린트를 켠 채로) | 2.2 |
| [`projects/glob_upgrade`](projects/glob_upgrade) | 실제 Cargo 프로젝트에서 1.0.0 → 1.1.0 갱신 재현 | 2.2, 2.3, 4.1 |

### 01_glob_priority — 글롭으로 들어온 이름의 순위

사용자 모듈 `on_1_0`과 `on_1_1`은 첫 줄(`use crate::templog_1_x as templog;`)만 다릅니다. 1.1의 `to_fahrenheit`는 명시적 가져오기에 가려져 아무도 부르지 않으므로, 이 한 파일 안에서는 `dead_code` 경고가 나서 `#[expect]`를 붙였습니다(실제 외부 크레이트의 공개 함수라면 나지 않는 경고입니다).

```sh
cargo run -p er-ch23-avoid-wildcard-imports --example 01_glob_priority
```

```text
(가) 같은 모듈의 지역 정의 vs 글롭
  templog 1.0: 21.5도, templog 1.1: 21.5도 → 그대로
(나) 명시적 가져오기 vs 글롭
  templog 1.0: 70.70F, templog 1.1: 70.70F → 그대로
(다) 함수 안의 글롭 vs 바깥 모듈의 지역 정의
  templog 1.0: 21.5도, templog 1.1: 21.5°C → 바뀜
```

(다)의 함수는 설명을 위해 모듈 수준 글롭과 별도로 본문에도 `use templog::*;`를 두었습니다. 모듈 수준 글롭이 없어도 결과는 같습니다.

### 02_new_trait_method — 에러 없이 바뀌는 메서드 호출

세 모듈의 `run` 본문은 주석을 빼면 같고, 위쪽 두 줄의 `use`만 다릅니다. `glob_on_1_1`의 `use crate::LogExt;`에는 `unused_imports`를 기대하는 `#[expect]`를 붙였습니다. 이 경고가 동작이 바뀌었다는 유일한 단서라는 것을 보이려는 것입니다.

```sh
cargo run -p er-ch23-avoid-wildcard-imports --example 02_new_trait_method
```

```text
기록 20.5, 21.5, 22.5도를 -0.5도 보정한 뒤의 평균
  글롭, templog 1.0:   21.00도
  글롭, templog 1.1:   21.50도
  명시적, templog 1.1: 21.00도
```

### 03_allowed_globs — 글롭을 써도 되는 경우

파일 맨 위의 `#![warn(clippy::wildcard_imports)]`로 기본으로 꺼진 린트를 켰고, `cargo clippy`가 경고 없이 통과합니다. 테스트 모듈이 있으므로 테스트도 실행합니다.

```sh
cargo run -p er-ch23-avoid-wildcard-imports --example 03_allowed_globs
cargo test -p er-ch23-avoid-wildcard-imports --example 03_allowed_globs
```

```text
기록 4개, 평균 21.50도, 일교차 6.0도
21.5도 = 294.65K = 70.7F
```

테스트는 `load_skips_bad_lines`와 `summary_mentions_spread` 두 개가 통과합니다(`test result: ok. 2 passed`).

### projects/glob_upgrade — Cargo 프로젝트로 재현한 갱신

```text
glob_upgrade/
├── Cargo.toml              # 가상 워크스페이스: 빌드에 성공하는 세 패키지만 멤버
├── templog-1.0.0/          # 의존 크레이트 스냅숏 (패키지 이름 templog, version 1.0.0)
├── templog-1.1.0/          # 같은 이름, version 1.1.0: Alert와 Stats를 더함
├── shared/
│   ├── glob_import.rs      # 사용자 코드, 글롭 판
│   └── explicit_import.rs  # 사용자 코드, 명시적 가져오기 판 (use 한 줄만 다름)
├── glob-on-v1-0/           # glob_import.rs × 1.0.0
├── explicit-on-v1-0/       # explicit_import.rs × 1.0.0
├── explicit-on-v1-1/       # explicit_import.rs × 1.1.0
└── glob-on-v1-1-fails/     # glob_import.rs × 1.1.0 → 일부러 실패(E0034), 자기 [workspace]를 가짐
```

조합 패키지는 `[[bin]] path = "../shared/…"`로 같은 소스 파일을 공유하고, `templog = { path = "../templog-1.x.0" }`의 경로만 다릅니다. 두 스냅숏은 패키지 이름이 같아서 한 워크스페이스의 멤버가 될 수 없으므로(멤버로 두면 ``two packages named `templog` in this workspace`` 에러) 워크스페이스의 `exclude`에 두고, 각자 빈 `[workspace]`를 가집니다.

```sh
cd appd-a/ch23-avoid-wildcard-imports/projects/glob_upgrade
cargo run -q -p glob-on-v1-0
cargo run -q -p explicit-on-v1-0
cargo run -q -p explicit-on-v1-1
cd glob-on-v1-1-fails && cargo build   # 일부러 실패합니다(4.1절)
```

```text
[glob-on-v1-0] templog 1.0.0
  평균 21.50도, 표준편차 2.21도
  경보: 평균 21.5도가 기준 20.0도를 넘었습니다
[explicit-on-v1-0] templog 1.0.0
  평균 21.50도, 표준편차 2.21도
  경보: 평균 21.5도가 기준 20.0도를 넘었습니다
[explicit-on-v1-1] templog 1.1.0
  평균 21.50도, 표준편차 2.21도
  경보: 평균 21.5도가 기준 20.0도를 넘었습니다
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch23-avoid-wildcard-imports
rustc --edition 2024 compile_fail/e0659_two_glob_imports.rs --out-dir /tmp/er-ch23
rustc --edition 2024 compile_fail/e0107_glob_shadows_prelude.rs --out-dir /tmp/er-ch23
```

### 4.1 글롭이 들여온 새 트레이트와 메서드 이름이 겹침 — E0034

파일: [`projects/glob_upgrade/glob-on-v1-1-fails`](projects/glob_upgrade/glob-on-v1-1-fails) (`Compiling` 줄과 마지막 요약 줄은 생략했습니다)

```text
error[E0034]: multiple applicable items in scope
  --> ../shared/glob_import.rs:48:47
   |
48 |     println!("  평균 {m:.2}도, 표준편차 {:.2}도", log.spread());
   |                                                       ^^^^^^ multiple `spread` found
   |
note: candidate #1 is defined in an impl of the trait `LogExt` for the type `templog::Log`
  --> ../shared/glob_import.rs:23:5
   |
23 |     fn spread(&self) -> f64 {
   |     ^^^^^^^^^^^^^^^^^^^^^^^
   = note: candidate #2 is defined in an impl of the trait `templog::Stats` for the type `templog::Log`
help: disambiguate the method for candidate #1
   |
48 -     println!("  평균 {m:.2}도, 표준편차 {:.2}도", log.spread());
48 +     println!("  평균 {m:.2}도, 표준편차 {:.2}도", LogExt::spread(&log));
   |
help: disambiguate the method for candidate #2
   |
48 -     println!("  평균 {m:.2}도, 표준편차 {:.2}도", log.spread());
48 +     println!("  평균 {m:.2}도, 표준편차 {:.2}도", templog::Stats::spread(&log));
   |
```

- **읽는 법**: 후보 #2가 `templog::Stats`이고, 사용자 코드에는 `Stats`라는 단어가 한 번도 나오지 않습니다. 이름을 적은 적 없는 트레이트가 후보로 나오면 대개 글롭(또는 프렐류드)이 들여온 것입니다. 두 후보 모두 수신자가 `&self`라서 같은 탐색 단계에 잡혔습니다.
- **원인**: templog 1.1.0이 더한 `Stats`를 `use templog::*;`가 스코프에 들여왔습니다. [ch13](../ch13-default-implementations/README.md) 4.2절의 E0034는 이미 가져와 쓰던 트레이트에 **기본 메서드가 더해져서** 났고, 여기서는 **글롭이 새 트레이트를 들여와서** 났습니다. 같은 에러 코드가 다른 경로로 생긴 것입니다.
- **해결**: help 두 개는 결과가 다릅니다. 후보 #1을 따르면 표준편차 `2.21도`가 그대로 나오고, 후보 #2를 따르면 일교차 `6.00도`가 나옵니다. 사용자가 뜻한 것은 #1입니다. 하지만 이 호출 하나만 고치면 글롭은 남아 다음 갱신에서 또 깨질 수 있으므로, `use templog::{Log, VERSION, mean};`로 바꾸는 것이 근본적인 해결입니다(`explicit-on-v1-1`).

### 4.2 두 글롭이 같은 이름을 들여옴 — E0659

파일: [`compile_fail/e0659_two_glob_imports.rs`](compile_fail/e0659_two_glob_imports.rs) (뒤따르는 경고와 요약 줄은 생략했습니다)

```text
error[E0659]: `mean` is ambiguous
  --> compile_fail/e0659_two_glob_imports.rs:41:27
   |
41 |     println!("평균 {:.2}도", mean(&readings)); // E0659
   |                              ^^^^ ambiguous name
   |
   = note: ambiguous because of multiple glob imports of a name in the same module
note: `mean` could refer to the function imported here
  --> compile_fail/e0659_two_glob_imports.rs:35:5
   |
35 | use chartkit::*;
   |     ^^^^^^^^^^^
   = help: consider adding an explicit import of `mean` to disambiguate
note: `mean` could also refer to the function imported here
  --> compile_fail/e0659_two_glob_imports.rs:36:5
   |
36 | use templog::*;
   |     ^^^^^^^^^^
   = help: consider adding an explicit import of `mean` to disambiguate
```

- **읽는 법**: 에러는 `use` 줄이 아니라 `mean`을 **쓴** 줄에서 납니다. 바로 앞 줄의 `bar`는 `chartkit`에만 있으므로 문제없습니다. 이 에러 뒤에 ``unused import: `templog::*` `` 경고가 따라 나오는데, `mean`을 어느 글롭으로도 해석하지 못해 `templog`에서 쓴 이름이 없어졌기 때문입니다.
- **원인**: 사용자는 `templog`의 `mean`을 써 왔는데, chartkit 1.1.0이 같은 이름의 `mean`을 더했습니다. 두 의존성 중 어느 쪽의 마이너 업데이트도 이 에러를 낼 수 있습니다.
- **해결**: help대로 `use templog::mean;`을 더하면 명시적 가져오기가 글롭을 이겨서 풀립니다(규칙 1). 더 나은 해결은 두 글롭을 모두 `use chartkit::bar;`, `use templog::mean;`으로 바꾸는 것입니다. 두 방법 모두 `평균 21.50도`를 출력합니다.

### 4.3 글롭이 프렐류드의 `Result`를 가림 — E0107

파일: [`compile_fail/e0107_glob_shadows_prelude.rs`](compile_fail/e0107_glob_shadows_prelude.rs) (뒤따르는 E0308은 핵심 줄만 옮겼습니다)

```text
error[E0107]: type alias takes 1 generic argument but 2 generic arguments were supplied
  --> compile_fail/e0107_glob_shadows_prelude.rs:37:33
   |
37 | fn parse_reading(line: &str) -> Result<f64, String> {
   |                                 ^^^^^^    -------- help: remove the unnecessary generic argument
   |                                 |
   |                                 expected 1 generic argument
   |
note: type alias defined here, with 1 generic parameter: `T`
  --> compile_fail/e0107_glob_shadows_prelude.rs:31:14
   |
31 |     pub type Result<T> = std::result::Result<T, Error>;
   |              ^^^^^^ -

error[E0308]: mismatched types
  --> compile_fail/e0107_glob_shadows_prelude.rs:38:5
...
   = note: expected enum `Result<_, templog::Error>`
              found enum `Result<_, String>`
```

- **읽는 법**: "type alias"라는 말이 단서입니다. 프렐류드의 `Result`는 열거형인데 컴파일러는 별칭으로 읽었고, `note`가 가리키는 정의는 templog 안에 있습니다. 함께 나온 E0308은 컴파일러가 반환 타입을 templog의 `Result<f64>`, 곧 `Result<f64, templog::Error>`로 읽고 본문을 검사한 결과입니다.
- **원인**: templog 1.1.0이 많은 크레이트가 두는 편의 별칭 `pub type Result<T>`를 더했고, 글롭이 이것을 들여와 가장 바깥 스코프인 프렐류드의 `Result`를 가렸습니다(규칙 1).
- **해결**: help("remove the unnecessary generic argument")를 따르면 E0107은 사라지지만 E0308만 남습니다. 에러 타입이 `String`에서 `templog::Error`로 바뀌기 때문입니다. `use templog::{Log, mean};`로 바꾸거나, 글롭을 남겨야 한다면 `std::result::Result<f64, String>`처럼 경로를 적습니다. 두 방법 모두 `평균 21.17도`를 출력합니다.

## 5. 연습 문제

### 연습 1. 버전 고정 확인하기

[`glob-on-v1-1-fails/Cargo.toml`](projects/glob_upgrade/glob-on-v1-1-fails/Cargo.toml)의 의존성을 `templog = { path = "../templog-1.1.0", version = "=1.0.0" }`으로 바꾸고 `cargo build`를 실행해 에러를 읽으세요. 그다음 `version = "1.0.0"`으로, 마지막으로 경로를 `../templog-1.0.0`으로 바꿔 가며 결과를 비교하세요. 끝나면 파일을 원래대로 돌려놓습니다.

<details>
<summary>힌트</summary>

`=1.0.0`은 1.1.0을 거부하므로 컴파일 전에 `failed to select a version for the requirement` 에러로 멈춥니다(2.3절). `1.0.0`은 1.1.0을 받아들여 E0034가 다시 납니다. 경로를 1.0.0 스냅숏으로 바꾸면 `=1.0.0`과 `1.0.0` 모두 빌드되고, 출력 첫 줄이 `[glob-on-v1-1-fails] templog 1.0.0`이 됩니다. 실제 crates.io 의존성에서는 경로 대신 Cargo가 고르는 버전이 바뀐다고 생각하면 됩니다.

</details>

### 연습 2. 단서가 사라지는 경우

[`02_new_trait_method.rs`](examples/02_new_trait_method.rs)의 `LogExt`에 최고 온도를 돌려주는 `fn peak(&self) -> f64`를 더해 구현하고, `glob_on_1_1::run`에서 `log.peak()`를 한 번 부르세요. 빌드할 때 어떤 경고가 나고, 보정 결과는 어떻게 되나요? 그다음 `glob_on_1_1`을 명시적 가져오기로 고쳐 결과를 `21.00도`로 되돌리세요.

<details>
<summary>힌트</summary>

`LogExt`가 `peak` 때문에 쓰이게 되므로 `unused_imports` 경고가 사라지고, 대신 붙여 둔 `#[expect]`가 `this lint expectation is unfulfilled` 경고를 냅니다. 보정 결과는 여전히 `21.50도`입니다. 실제 코드에는 `#[expect]`가 없으므로 아무 경고 없이 동작만 바뀐 상태가 됩니다. 고칠 때는 `use templog::*;`를 `use templog::{Log, mean};`로 바꾸고 `#[expect]` 속성을 지웁니다. 그러면 결과는 `21.00도`로 돌아오고, 이제 아무도 부르지 않게 된 `templog_1_1::Calibrated`에 ``trait `Calibrated` is never used`` 경고가 납니다. 한 파일로 흉내 낸 탓에 생기는 경고이므로 `#[expect(dead_code, reason = "…")]`를 붙입니다.

</details>

### 연습 3. 린트로 글롭 찾기

[`01_glob_priority.rs`](examples/01_glob_priority.rs) 맨 위에 `#![warn(clippy::wildcard_imports)]`를 붙이고 `cargo clippy -p er-ch23-avoid-wildcard-imports --example 01_glob_priority`를 실행하세요. 경고가 몇 곳에 나는지, `templog_1_1`의 `pub use super::templog_1_0::*;`에는 왜 나지 않는지 설명하고, 경고가 난 글롭을 모두 명시적 가져오기로 바꿔 (다)의 결과가 `그대로`가 되는지 확인하세요.

<details>
<summary>힌트</summary>

경고는 사용자 모듈의 글롭 네 곳(두 모듈의 모듈 수준 글롭과 함수 안 글롭)에 납니다. `pub use` 재내보내기는 린트의 예외입니다(2.2절). 두 모듈에서 실제로 templog에서 쓰는 이름은 `mean`뿐이므로 모듈 수준 글롭을 `use templog::mean;`으로 바꾸고 함수 안의 글롭은 지우면 됩니다. 그러면 세 줄 모두 `그대로`가 됩니다. 이제 아무도 부르지 않게 된 `templog_1_1::format_temp`에는 `dead_code` 경고가 나는데, 한 파일로 흉내 낸 탓에 생기는 경고이므로 `to_fahrenheit`처럼 `#[expect(dead_code, reason = "…")]`를 붙입니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 23: Avoid wildcard imports](https://effective-rust.com/wildcard.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- The Rust Reference
  - [Use declarations: Glob imports](https://doc.rust-lang.org/reference/items/use-declarations.html#glob-imports): 글롭이 같은 이름 공간의 항목과 명시적 가져오기에 가려지는 규칙
  - [Name resolution](https://doc.rust-lang.org/reference/names/name-resolution.html): 글롭끼리 겹친 이름은 쓸 때 에러가 나는 규칙(`names.resolution.expansion.imports.ambiguity.glob-vs-glob`)
- [The Cargo Book: SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html): [공개 항목 추가](https://doc.rust-lang.org/cargo/reference/semver.html#item-new)가 마이너 변경이면서 글롭과 충돌할 수 있다는 설명, [고유 항목 추가](https://doc.rust-lang.org/cargo/reference/semver.html#impl-item-new)가 깨질 수도 있는 변경인 이유
- [`std::io::prelude`](https://doc.rust-lang.org/std/io/prelude/index.html): 글롭으로 가져가라고 만든 프렐류드의 예
- [Clippy 린트 목록](https://rust-lang.github.io/rust-clippy/master/index.html#wildcard_imports): `wildcard_imports`, `enum_glob_use`. 로컬에서는 `cargo clippy --explain wildcard_imports`로 볼 수 있습니다.
- 에러 설명: `rustc --explain E0034`, `rustc --explain E0659`, `rustc --explain E0107`
