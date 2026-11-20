# ch25. 의존성 그래프 관리하기

`Cargo.toml`에 한 줄을 더하면 그 크레이트가 의존하는 크레이트까지 함께 들어옵니다. 이렇게 모인 크레이트 전체가 의존성 그래프(dependency graph)입니다. 이 챕터는 그래프를 다루는 네 가지 판단을 다룹니다. 버전 요구 사항을 얼마나 넓게 적을지, 같은 크레이트가 여러 버전으로 들어왔을 때 어떻게 찾아 줄일지, `Cargo.lock`을 언제 커밋하고 언제 올릴지, 그리고 애초에 무엇에 의존할지입니다. 실습은 네트워크 없이 Cargo의 버전 선택을 재현하는 연습용 레지스트리와, crates.io의 실제 크레이트 셋으로 만든 작은 애플리케이션으로 합니다.

- 원제: [Item 25: Manage your dependency graph](https://effective-rust.com/dep-graph.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch25-dependency-graph`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

앞의 부록 챕터가 이 챕터로 미룬 것도 여기서 받습니다(괄호 안은 이 챕터의 절). [ch21](../ch21-semver/README.md)의 "버전을 고정할지 범위로 둘지, `cargo update`로 언제 올릴지"(2.2절, 2.4절)와 라이선스 점검(2.5절), [ch23](../ch23-avoid-wildcard-imports/README.md)의 "고정의 비용과 `Cargo.lock`의 역할"(2.2절), [ch7](../ch7-builders/README.md)의 "`derive_builder` 하나가 여러 크레이트를 끌고 온다"(2.6절), [ch16](../ch16-avoid-unsafe/README.md)의 "의존성을 고를 때 유지 상태와 RustSec 권고를 확인한다"(2.5절, 2.6절)입니다. 다음 내용을 안다고 보고 진행합니다.

- [Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md) 2.3절: `cargo add`, `Cargo.lock`이 하는 일, `cargo update`와 `--precise`
- [ch21](../ch21-semver/README.md) 2.1~2.2절: 버전 요구 사항 문법(캐럿, `~`, `=`, `*`, 0.x 규칙), 아래쪽 경계, 연습용 레지스트리
- [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.6절: `rust-version`을 고려하는 리졸버 3
- [ch4](../ch4-idiomatic-errors/README.md)와 [ch7](../ch7-builders/README.md): 실습에 쓰는 `thiserror`, `anyhow`, `derive_builder`
- [ch26](../ch26-feature-creep/README.md): 기능 통합과 `cargo tree -e features`

Cargo가 의존성을 해석하는 규칙 전반과 path·git 의존성, `[patch]`는 Part 3 `ch2-dependencies`에서, 빌드 스크립트를 쓰는 법은 Part 3 `ch8-build-scripts`에서 다룹니다. 여기서는 판단에 필요한 만큼만 씁니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Cargo가 같은 크레이트에 대한 여러 요구 사항을 처리하는 규칙(호환되면 하나로 합치고, 호환되지 않으면 따로 두고, 겹치지 않으면 실패한다)을 설명하고 실행 결과로 확인할 수 있다.
2. 버전 고정(`=x.y.z`)이 그래프에서 일으키는 문제를 실제 출력으로 설명하고, 캐럿 요구 사항과 `Cargo.lock`에 역할을 나눠 맡길 수 있다.
3. `cargo tree`, `-d`, `-i 이름@버전`으로 중복 버전과 그 출처를 찾고, 중간 크레이트를 올려 중복을 없앨 수 있다.
4. `Cargo.lock`을 커밋할지 판단하고, `cargo update --dry-run`, 패키지 지정, `--precise`로 잠금 파일을 정해 둔 때에 올릴 수 있다.
5. 쓰지 않는 의존성, 중복, 라이선스, 보안 권고를 잡는 도구를 구별하고, 새 의존성의 비용(끌고 오는 패키지, 빌드 때 실행되는 코드)을 그래프로 따져 볼 수 있다.

## 2. 핵심 개념

2.1절에서 Cargo가 버전을 고르는 규칙과 실습 환경을 정리하고, 2.2~2.6절에서 지침마다 "문제 상황 → 비권장 설정 → 권장 설정 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 이 Item은 코드보다 설정과 습관을 다루므로 "비권장 코드" 자리에 `Cargo.toml` 설정이나 작업 습관이 옵니다. 출력에 든 절대 경로는 `<저장소>`로 바꿔 적었습니다.

### 2.1 지도: 의존성 그래프와 Cargo가 버전을 고르는 규칙

`[dependencies]`에 직접 적은 크레이트가 **직접 의존성**이고, 그것들이 다시 의존해서 따라 들어온 크레이트가 **전이 의존성**(transitive dependency)입니다. Cargo는 그래프 전체의 버전 요구 사항을 모아 패키지마다 실제 버전을 정하고(의존성 해석, resolve), 결과를 `Cargo.lock`에 적습니다. 그 뒤의 빌드는 다시 고르지 않고 `Cargo.lock`을 그대로 씁니다.

같은 크레이트를 그래프의 여러 곳에서 요구하면 Cargo는 다음 규칙을 따릅니다(Cargo Book의 Dependency Resolution). 두 버전이 호환되는지는 ch21 2.1절의 규칙(1.x끼리, 0.4.x끼리 호환)으로 정합니다.

| 같은 크레이트에 대한 요구 사항들 | Cargo가 하는 일 | 확인한 곳 |
| --- | --- | --- |
| 서로 호환되는 범위이고 겹친다(`"1.2"`, `"1.4"`) | 겹치는 범위에서 가장 높은 버전 **하나**를 고른다(1.6.0) | 2.3절 `before` |
| 서로 호환되지 않는다(`"1.4"`, `"2"`) | 버전마다 사본을 하나씩 두고 **둘 다** 컴파일한다 | 2.3절 `before` |
| 호환되는 범위인데 겹치지 않는다(`"=2.0.0"`, `"2.1"`) | 다른 조합을 찾아 되돌아가 보고(backtrack), 없으면 해석에 실패한다 | 2.2절 |
| 호환되지 않는 두 버전이 같은 `links` 값을 선언했다 | 해석에 실패한다 | 2.3절 `links_fails` |

호환되는 두 버전을 함께 두지 않는 이유는, 그렇게 하면 같은 1.x API를 기대하는 크레이트들이 서로 다른 타입을 주고받게 되기 때문입니다. 호환되지 않는 버전은 어차피 API가 다르므로 따로 둘 수밖에 없습니다.

크레이트 이름도 짚어 둡니다. crates.io의 이름은 하나의 평평한 이름 공간을 이루고, 먼저 등록한 쪽이 그 이름을 가집니다. 그래서 비슷한 이름의 다른 크레이트를 잘못 적지 않도록 정확한 이름을 확인해야 합니다. 이름의 `-`는 코드에서 `_`가 됩니다. 아래 실습의 `tiny-date`는 `Cargo.toml`에는 `tiny-date = "2"`로, 코드에서는 `tiny_date::Date`로 씁니다.

**실습 환경: 연습용 레지스트리.** [`projects/planner`](projects/planner/before/Cargo.toml)는 ch21 2.2절과 같은 방법으로 crates.io 대신 `vendor/` 디렉터리를 레지스트리로 씁니다. `projects/planner/.cargo/config.toml`의 내용은 다음과 같습니다(점으로 시작하는 파일이라 사이트에는 실리지 않습니다).

```toml
[source.crates-io]
replace-with = "practice-registry"

[source.practice-registry]
directory = "vendor"
```

`vendor/`에는 "게시된 것으로 치는" 가상의 크레이트 12개가 들어 있습니다. 같은 이름의 실제 crates.io 크레이트와는 관계가 없습니다.

| 크레이트 | 버전과 의존성 | 메모 |
| --- | --- | --- |
| `tiny-date` | 1.2.0, 1.4.0, 1.6.0, 2.0.0, 2.1.0 | 2.0.0에서 `Date::new`가 `Option`을 돌려주게 바뀜(호환되지 않는 변경), 2.1.0에서 `weekday` 추가 |
| `remind-kit` | 0.4.2(`tiny-date = "1.2"`), 0.4.3(`"2.1"`) | 공개 API는 같고 안에서 쓰는 `tiny-date`만 다름 |
| `cal-grid` | 0.9.1(`"1.4"`), 0.10.0(`"2"`) | 0.10.0에서 `month_title`의 반환 타입이 `Option<String>`으로 바뀜 |
| `chime-sys` | 0.2.0, 0.3.0 | 둘 다 `links = "chime"`(네이티브 라이브러리를 감싸는 크레이트라고 침) |
| `ring-tone` | 1.0.0(`chime-sys = "0.2"`) | |

이 아래의 `before`, `after`, `pinned_fails`, `links_fails`는 같은 레지스트리를 쓰는 별개의 패키지입니다. 연습용 레지스트리가 진짜 레지스트리와 다른 점도 실험으로 확인했습니다. `cargo update 이름 --precise 버전`의 버전 지정이 적용되지 않았고(없는 버전을 줘도 에러 없이 그 패키지를 최신으로 올림), 게시 시각이 없어서 2.6절의 최소 게시 경과 시간을 줘도 걸러지는 버전이 없었습니다. 이 두 동작은 [`projects/real_graph`](projects/real_graph/Cargo.toml)에서 crates.io로 봅니다.

### 2.2 지침 1: 버전을 고정하지 말고 캐럿 요구 사항을 쓴다. 재현은 `Cargo.lock`에 맡긴다

**문제 상황.** `planner` 0.2.0을 만들던 개발자가 `tiny-date` 2.0.0으로 시험을 마쳤습니다. 다음 버전이 무엇을 깨뜨릴지 모르니 시험한 버전만 쓰자는 생각으로 요구 사항을 고정합니다. ch23 2.3절에서 본 "글롭 가져오기를 지키려는 고정"도 같은 모양입니다.

**비권장 설정.** [`projects/planner/pinned_fails`](projects/planner/pinned_fails/Cargo.toml)

```toml
[dependencies]
tiny-date = "=2.0.0" # 비권장: 버전 고정
remind-kit = "0.4.3"
cal-grid = "0.10"
```

```text
$ cd appd-a/ch25-dependency-graph/projects/planner/pinned_fails
$ cargo build
error: failed to select a version for `tiny-date`.
    ... required by package `remind-kit v0.4.3`
    ... which satisfies dependency `remind-kit = "^0.4.3"` of package `planner v0.2.0 (<저장소>/appd-a/ch25-dependency-graph/projects/planner/pinned_fails)`
versions that meet the requirements `^2.1` are: 2.1.0

all possible versions conflict with previously selected packages

  previously selected package `tiny-date v2.0.0`
    ... which satisfies dependency `tiny-date = "=2.0.0"` of package `planner v0.2.0 (<저장소>/appd-a/ch25-dependency-graph/projects/planner/pinned_fails)`

failed to select a version for `tiny-date` which could resolve this conflict
```

위에서부터 읽으면 됩니다. `remind-kit` 0.4.3은 `weekday`가 생긴 2.1 이상(`^2.1`)을 요구하는데, `planner`가 이미 2.0.0을 골라 두었습니다. 둘 다 2.x라서 Cargo는 사본 둘을 만들 수 없고, 겹치는 버전이 없으니 해석이 멈춥니다. 고정은 "내가 받아들이는 버전"을 하나로 줄여서 **그래프의 다른 크레이트와 겹칠 여지**를 없앱니다. 라이브러리가 이렇게 고정하면 그 라이브러리를 쓰는 모든 프로젝트가 같은 벽에 부딪힙니다.

실패하는 편이 차라리 낫습니다. `remind-kit`의 요구 사항이 `"0.4"`처럼 넓으면 Cargo는 되돌아가서 다른 조합을 찾고, 조용히 성공합니다. `pinned_fails`의 복사본에서 `remind-kit = "0.4"`로 바꿔 실행한 결과입니다.

```text
$ cargo run
     Locking 4 packages to highest Rust 1.100.0 compatible versions
      Adding remind-kit v0.4.2 (available: v0.4.3)
      Adding tiny-date v2.0.0 (available: v2.1.0)
   (Compiling 줄 생략)
마감일: 2026-11-22
[알림] 2026-11-22 원고 마감
2026년 11월 (30일)
tiny-date 사본
  planner가 쓰는 것:    2.0.0
  remind-kit이 쓰는 것: 1.6.0
  cal-grid가 쓰는 것:   2.0.0
```

`tiny-date` 2.1을 요구하지 않는 옛 `remind-kit` 0.4.2로 물러났고, 그 0.4.2가 `tiny-date` 1.x를 끌고 와서 2.3절의 중복이 되살아났습니다. 알림 문구의 요일도 사라졌습니다. 이 사실을 알려 주는 것은 `Adding … (available: v0.4.3)` 한 줄뿐이고, `Cargo.lock`이 생긴 뒤의 빌드나 `cargo -q`에서는 그 줄도 나오지 않습니다. 고정은 그 밖에도 2.0.1 같은 패치 릴리스의 버그 수정과 보안 수정을 받지 못하게 합니다.

**권장 설정.** [`projects/planner/after`](projects/planner/after/Cargo.toml)

```toml
[dependencies]
tiny-date = "2"      # >=2.0.0, <3.0.0
remind-kit = "0.4.3"
cal-grid = "0.10"
```

Cargo는 세 요구 사항(`^2`, `remind-kit`의 `^2.1`, `cal-grid`의 `^2`)을 모두 만족하는 2.1.0 하나를 고릅니다. 캐럿 요구 사항은 작성자의 약속(같은 메이저 안에서는 호환)과 맞물리므로, 깨질 수 있는 3.0.0은 받지 않으면서 겹칠 여지는 최대로 남깁니다. 아래쪽 경계는 실제로 쓰는 기능이 생긴 버전으로 적습니다(ch21 2.2절).

"시험한 그 버전으로 빌드한다"는 목표는 `Cargo.lock`이 이룹니다. `after/Cargo.lock`에 2.1.0이 적혀 있으므로 3.x는 물론 2.2.0이 나와도 `cargo build`는 2.1.0을 씁니다. 버전이 바뀌는 때는 `cargo update`를 실행하거나 `Cargo.toml`의 요구 사항을 바꿨을 때뿐입니다(2.4절). 새 릴리스에 문제가 있어 한동안 옛 버전에 머물러야 한다면 `Cargo.toml`에 `=`를 적지 말고 `cargo update 이름 --precise 버전`으로 `Cargo.lock`만 되돌립니다. 요구 사항은 그대로이므로 문제가 풀리면 `cargo update`로 바로 돌아오고, 다른 크레이트의 요구 사항도 그대로 지켜집니다(2.4절에서 다른 크레이트의 아래쪽 경계보다 낮게 내리려다 거절되는 모습을 봅니다).

**지침을 따르지 않아도 되는 경우.** 함께 만들어 함께 게시하는 크레이트 쌍은 `=`로 묶습니다. `thiserror` 2.0.21이 `thiserror-impl`을 `=2.0.21`로 요구하는 것이 그 예입니다(ch21 2.2절). 2.4절에서 `thiserror`만 내렸는데 `thiserror-impl`도 함께 내려가는 것이 이 묶음 때문입니다. ch23의 경우처럼 의존성이 무엇을 내보내든 받아들이는 글롭 가져오기를 당장 고칠 수 없을 때도 고정이 차선책이 됩니다. 두 경우 모두 고정은 의존성의 수정 사항을 막는다는 비용을 알고 고르는 예외입니다.

### 2.3 지침 2: 중복 버전을 `cargo tree -d`로 찾아 줄인다

**문제 상황.** [`projects/planner/before`](projects/planner/before/Cargo.toml)의 `planner` 0.1.0은 `tiny-date` 2.x를 직접 쓰면서, 안에서 1.x를 쓰는 `remind-kit` 0.4.2와 `cal-grid` 0.9.1에도 의존합니다. 두 중간 크레이트의 공개 API에는 `tiny-date`의 타입이 드러나지 않으므로 컴파일 에러는 없습니다. 실행해 보면 각 크레이트가 링크한 `tiny-date`가 다릅니다.

```text
$ cd appd-a/ch25-dependency-graph/projects/planner/before
$ cargo build
   Compiling tiny-date v1.6.0
   Compiling tiny-date v2.1.0
   Compiling cal-grid v0.9.1
   Compiling remind-kit v0.4.2
   Compiling planner v0.1.0 (<저장소>/appd-a/ch25-dependency-graph/projects/planner/before)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
$ cargo run -q
마감일: 2026-11-22
[알림] 2026-11-22 원고 마감
2026년 11월 (30일)
tiny-date 사본
  planner가 쓰는 것:    2.1.0
  remind-kit이 쓰는 것: 1.6.0
  cal-grid가 쓰는 것:   1.6.0
```

(`Compiling` 줄의 순서와 시간은 실행마다 다릅니다.) 2.1절의 규칙 두 가지가 한 번에 보입니다. `remind-kit`의 `"1.2"`와 `cal-grid`의 `"1.4"`는 호환되므로 1.6.0 하나로 합쳐졌고, 1.x와 2.x는 호환되지 않으므로 둘 다 컴파일되어 실행 파일에 들어갔습니다.

**비권장 습관.** 빌드가 되니 그대로 둡니다. 이 정도 크기에서는 티가 나지 않지만, 큰 크레이트가 두 벌이면 컴파일 시간과 실행 파일 크기가 그만큼 늘어납니다.

**권장 습관.** 먼저 그래프를 봅니다. `cargo tree`는 그래프를 트리로 보여 주고, `-d`(`--duplicates`)는 여러 버전으로 들어온 패키지만 골라 거꾸로(누가 그것을 요구했는지) 보여 줍니다.

```text
$ cargo tree
planner v0.1.0 (<저장소>/appd-a/ch25-dependency-graph/projects/planner/before)
├── cal-grid v0.9.1
│   └── tiny-date v1.6.0
├── remind-kit v0.4.2
│   └── tiny-date v1.6.0
└── tiny-date v2.1.0
$ cargo tree -d
tiny-date v1.6.0
├── cal-grid v0.9.1
│   └── planner v0.1.0 (<저장소>/appd-a/ch25-dependency-graph/projects/planner/before)
└── remind-kit v0.4.2
    └── planner v0.1.0 (<저장소>/appd-a/ch25-dependency-graph/projects/planner/before)

tiny-date v2.1.0
└── planner v0.1.0 (<저장소>/appd-a/ch25-dependency-graph/projects/planner/before)
```

패키지 하나의 출처만 보려면 `-i`(`--invert`)를 씁니다. 이름이 두 버전에 걸쳐 있으면 `cargo tree -i tiny-date`는 ``error: specification `tiny-date` is ambiguous``로 거절하고 `tiny-date@1.6.0`, `tiny-date@2.1.0` 중에서 고르라고 안내합니다. `cargo tree -i tiny-date@1.6.0`은 위 `-d` 출력의 첫 덩어리와 같습니다.

옛 사본을 끌고 오는 것은 `remind-kit` 0.4.2와 `cal-grid` 0.9.1입니다. 둘에게 2.x를 쓰는 새 버전이 있는지는 `cargo update --dry-run`(`-n`)으로 봅니다. `Cargo.lock`을 바꾸지 않고 바꿀 내용만 보여 줍니다.

```text
$ cargo update --dry-run --verbose
     Locking 1 package to highest Rust 1.100.0 compatible version
   Unchanged cal-grid v0.9.1 (available: v0.10.0)
    Updating remind-kit v0.4.2 -> v0.4.3
note: to see how you depend on a package, run `cargo tree --invert <dep>@<ver>`
warning: not updating lockfile due to dry run
```

두 줄의 의미가 다릅니다. `remind-kit` 0.4.3은 요구 사항 `"0.4"` 안에 있으므로 `cargo update`만으로 올라옵니다. `cal-grid` 0.10.0은 `"0.9"` 밖이라 `Unchanged … (available: v0.10.0)`으로 알려 주기만 합니다. `before`의 복사본에서 `cargo update`를 실행한 뒤 다시 보면, 중복은 줄었지만 사라지지 않았습니다.

```text
$ cargo update
     Locking 1 package to highest Rust 1.100.0 compatible version
    Updating remind-kit v0.4.2 -> v0.4.3
note: pass `--verbose` to see 1 unchanged dependencies behind latest
$ cargo tree -d
tiny-date v1.6.0
└── cal-grid v0.9.1
    └── planner v0.1.0 (<저장소>/…/before)

tiny-date v2.1.0
├── planner v0.1.0 (<저장소>/…/before)
└── remind-kit v0.4.3
    └── planner v0.1.0 (<저장소>/…/before)
```

`tiny-date` 1.6.0은 두 중간 크레이트가 **함께** 쓰던 사본이라 하나만 올려서는 남습니다([`02_dependency_cost`](examples/02_dependency_cost.rs)의 "나눠 쓰는 것"). `cal-grid`를 `"0.10"`으로 올리고 바뀐 반환 타입에 맞춰 한 줄을 고친 것이 `after`입니다.

```text
$ cd ../after
$ cargo tree -d
warning: nothing to print.

To find dependencies that require specific target platforms, try to use option `--target all` first, and then narrow your search scope accordingly.
$ cargo run -q
마감일: 2026-11-22
[알림] 2026-11-22(일) 원고 마감
2026년 11월 (30일)
tiny-date 사본
  planner가 쓰는 것:    2.1.0
  remind-kit이 쓰는 것: 2.1.0
  cal-grid가 쓰는 것:   2.1.0
```

정리하면, 중복을 줄이는 방법은 대개 **옛 버전을 끌고 오는 중간 크레이트를 올리는 것**입니다. 새 버전이 호환 범위 안이면 `cargo update`로, 밖이면 요구 사항을 고치고 코드를 맞춥니다. 반대로 내 직접 의존성을 중간 크레이트에 맞춰 내리는 방법도 있습니다(`before`라면 `tiny-date = "1.6"`). 중간 크레이트에 새 버전이 없으면 그 저장소에 올려 달라고 요청하는 것도 방법입니다.

`nothing to print.` 아래의 안내처럼 `cargo tree`는 기본으로 지금 컴퓨터의 플랫폼만 봅니다. 다른 플랫폼 전용 의존성까지 보려면 `--target all`을, 내 패키지의 기능을 모두 켠 그래프를 보려면 `--all-features`를 붙입니다(Cargo Book의 Troubleshooting 절이 중복을 찾을 때 둘을 함께 쓰는 명령을 예로 듭니다). 이 저장소의 루트 워크스페이스(2026-11-19)에서 `cargo tree -d --workspace`에 `--target all`을 붙이자 `windows-sys`, `wasi`, `r-efi`의 중복이 더 나왔습니다. 기능이 그래프에 어떻게 이어지는지는 `cargo tree -e features`로 보며, ch26에서 다룹니다.

**지침을 따르지 않아도 되는 경우.**

- **없애는 비용이 더 큰 중복.** crates.io의 실제 그래프인 `real_graph`(2.6절)에는 `syn` 2.0.119와 3.0.10이 함께 있습니다. `derive_builder` 0.20.2가 `darling` 0.20을 거쳐 `syn` 2를, `thiserror` 2.0.21이 `syn` 3을 씁니다. 2026-11-19에 `cargo search`로 본 `derive_builder`의 최신 버전은 0.20.2라서 위로 올려 맞출 길은 없습니다. crates.io 인덱스를 보면 `thiserror-impl`은 2.0.19부터 `syn` 3을 쓰므로, 아래로 맞출 길은 있습니다.

  ```text
  $ cargo update thiserror --precise 2.0.18 --dry-run
      Updating crates.io index
      Removing syn v3.0.10
   Downgrading thiserror v2.0.21 -> v2.0.18
   Downgrading thiserror-impl v2.0.21 -> v2.0.18 (available: v2.0.21)
  warning: not updating lockfile due to dry run
  ```

  하지만 그 대가로 2.0.19~2.0.21의 수정 사항을 받지 못하고, 다음 `cargo update`가 다시 2.0.21로 올립니다. 두 `syn`은 절차적 매크로를 컴파일할 때만 쓰이고 실행 파일에는 들어가지 않으므로(2.6절) 이 중복의 비용은 빌드 시간뿐입니다. 이럴 때는 중복을 받아들이고, 점검 도구에 이유를 적어 예외로 둡니다(2.5절).
- **타입이 API에서 만나면 비용이 아니라 에러가 됩니다.** 위 실습은 중간 크레이트가 `tiny-date`의 타입을 공개 API에 드러내지 않아서 문제없이 컴파일되었습니다. `remind-kit`이 `tiny_date::Date`(1.x)를 인자로 받았다면 `planner`가 넘기는 2.x의 `Date`와 이름만 같은 다른 타입이라 컴파일 에러가 납니다. 그 에러를 읽는 법과 재내보내기로 푸는 방법은 [ch24](../ch24-re-export-dependencies/README.md)에서 다룹니다.
- **C 라이브러리를 링크하는 크레이트는 중복이 허용되지 않습니다.** Rust는 버전마다 심볼을 구별하지만, 같은 C 라이브러리를 두 번 링크하면 같은 이름의 심볼이 둘이 됩니다. 그래서 `-sys` 크레이트는 `links` 키로 "이 네이티브 라이브러리를 링크한다"고 선언하고, Cargo는 같은 `links` 값을 가진 패키지를 그래프에 하나만 허용합니다. [`projects/planner/links_fails`](projects/planner/links_fails/Cargo.toml)는 `chime-sys = "0.3"`과, `chime-sys` 0.2를 쓰는 `ring-tone`에 함께 의존합니다.

```text
$ cd ../links_fails
$ cargo build
error: failed to select a version for `chime-sys`.
    ... required by package `ring-tone v1.0.0`
    ... which satisfies dependency `ring-tone = "^1"` of package `planner v0.3.0 (<저장소>/appd-a/ch25-dependency-graph/projects/planner/links_fails)`
versions that meet the requirements `^0.2` are: 0.2.0

package `chime-sys` links to the native library `chime`, but it conflicts with a previous package which links to `chime` as well:
package `chime-sys v0.3.0`
    ... which satisfies dependency `chime-sys = "^0.3"` of package `planner v0.3.0 (<저장소>/appd-a/ch25-dependency-graph/projects/planner/links_fails)`
note: only one package in the dependency graph may specify the same links value to ensure that only one copy of a native library is linked in the final binary
for more information, see https://doc.rust-lang.org/cargo/reference/resolver.html#links
help: try to adjust your dependencies so that only one package uses the `links = "chime"` value

failed to select a version for `chime-sys` which could resolve this conflict
```

0.2와 0.3은 호환되지 않으므로 보통이면 함께 들어올 두 버전인데, `links` 때문에 해석 단계에서 거절되었습니다. 실험해 보니 `ring-tone`을 `[dev-dependencies]`로 옮겨도 같은 에러가 났습니다. 고치려면 두 쪽이 같은 버전을 쓰게 맞춥니다(연습 3). C 라이브러리를 링크하는 일 자체는 Part 3 `ch8-build-scripts`와 [`ch34-ffi-boundaries`](../ch34-ffi-boundaries/README.md)에서 다룹니다.

### 2.4 지침 3: `Cargo.lock`을 커밋하고, 정해 둔 때에 올린다

**문제 상황.** `Cargo.lock`은 마지막으로 성공한 해석의 결과입니다. 패키지마다 이름, 버전, 출처(레지스트리면 `source` 줄)와 의존성을 적고, 같은 이름이 두 버전이면 의존성 표기에 버전을 붙여 구별합니다. `before/Cargo.lock`의 일부입니다.

```toml
[[package]]
name = "planner"
version = "0.1.0"
dependencies = [
 "cal-grid",
 "remind-kit",
 "tiny-date 2.1.0",
]

[[package]]
name = "remind-kit"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
dependencies = [
 "tiny-date 1.6.0",
]
```

연습용 레지스트리에서 온 패키지도 `source`가 crates.io로 적히는 것은 디렉터리 소스가 crates.io를 "대신"하기 때문입니다(ch21 2.2절). [`01_lockfile_duplicates`](examples/01_lockfile_duplicates.rs)는 이 파일만 읽어 `cargo tree -d`와 같은 중복을 찾습니다. 이 파일이 저장소에 없으면 동료의 컴퓨터와 CI는 그날의 레지스트리를 보고 다시 해석하므로, 같은 커밋이 서로 다른 그래프로 빌드됩니다. 반대로 커밋만 하고 올리지 않으면 그래프가 굳어서 버그 수정과 보안 수정이 쌓이기만 합니다.

**비권장 습관.**

- 애플리케이션인데 `Cargo.lock`을 커밋하지 않습니다.
- 문제가 생기면 `Cargo.lock`을 지우거나 손으로 고칩니다. 지우면 모든 패키지가 한꺼번에 다시 골라지고, 파일 첫머리의 `# It is not intended for manual editing.`처럼 손으로 고치는 파일이 아닙니다.
- `cargo update`를 아무 때나 통째로 실행하거나, 아예 실행하지 않습니다.

**권장 습관.** 애플리케이션과 바이너리는 `Cargo.lock`을 커밋합니다. 라이브러리는 2.7절에서 정리한 대로 지금의 Cargo 안내가 "프로젝트에 맞게 정하되 커밋을 출발점으로"이고, 저장소 밖에서 실행한 `cargo new --lib`가 만든 `.gitignore`에도 `/target`만 있었습니다. 다만 라이브러리의 `Cargo.lock`은 그 라이브러리를 쓰는 사람에게 아무 영향이 없습니다. 사용자의 그래프는 사용자의 `Cargo.lock`이 정하므로, 라이브러리는 커밋한 잠금 파일과 별도로 최신 의존성으로도 시험해야 합니다. 어느 쪽이든 올리는 일은 작게 나눠서 합니다.

| 명령 | 하는 일 |
| --- | --- |
| `cargo update --dry-run` | 바꿀 내용만 보여 줌(`-n`). `--verbose`를 붙이면 범위 밖의 새 버전이 있는 직접 의존성을 `Unchanged … (available: …)`로 알려 줌 |
| `cargo update 이름` | 그 패키지만 올림. 그 패키지를 올리는 데 필요한 경우에만 그 의존성도 함께 올림 |
| `cargo update 이름@버전` | 같은 이름이 여러 버전일 때 하나를 고름(이름만 쓰면 `is ambiguous` 에러) |
| `cargo update 이름 --precise 버전` | 정확히 그 버전으로 올리거나 내림 |
| `Cargo.toml`의 요구 사항 수정 | 범위 밖의 버전으로 갈 때(2.3절의 `cal-grid` 0.10) |

`--precise`는 crates.io를 쓰는 [`projects/real_graph`](projects/real_graph/Cargo.toml)에서 확인했습니다(2026-11-19, `--dry-run`이라 `Cargo.lock`은 바뀌지 않음).

```text
$ cd appd-a/ch25-dependency-graph/projects/real_graph
$ cargo update thiserror --precise 2.0.20 --dry-run
    Updating crates.io index
 Downgrading thiserror v2.0.21 -> v2.0.20
 Downgrading thiserror-impl v2.0.21 -> v2.0.20 (available: v2.0.21)
warning: not updating lockfile due to dry run
```

`thiserror`만 지정했는데 `thiserror-impl`도 함께 내려갔습니다. `thiserror` 2.0.20이 `thiserror-impl`을 `=2.0.20`으로 요구하기 때문입니다(2.2절). `--precise`도 그래프의 다른 요구 사항을 넘지는 못합니다. `quote`를 1.0.0으로 내리려 하면 거절됩니다(경로는 줄였습니다).

```text
$ cargo update quote --precise 1.0.0 --dry-run
    Updating crates.io index
error: failed to select a version for the requirement `quote = "^1.0.35"`
candidate versions found which didn't match: 1.0.0
location searched: crates.io index
required by package `thiserror-impl v2.0.21`
    ... which satisfies dependency `thiserror-impl = "=2.0.21"` (locked to 2.0.21) of package `thiserror v2.0.21`
    ... which satisfies dependency `thiserror = "^2"` (locked to 2.0.21) of package `ticket_desk v0.1.0 (<저장소>/…/real_graph)`
```

`thiserror-impl`이 적어 둔 아래쪽 경계 1.0.35가 그대로 지켜집니다. ch21 2.2절에서 "아래쪽 경계는 실제로 필요한 가장 낮은 버전"이라고 한 약속이 여기서 사용자를 지켜 줍니다. `--verbose`의 `Unchanged` 알림은 이 실행들에서 직접 의존성에만 붙었습니다. `real_graph`의 전이 의존성 `darling` 0.20.11은 crates.io에 0.24.1이 있는데도(2026-11-19 `cargo search darling`) 알림이 없었습니다. 전이 의존성이 얼마나 뒤처졌는지는 2.5절의 도구가 맡습니다.

정해 둔 때에 올리는 일은 사람의 기억보다 자동화에 맡깁니다. 책은 GitHub의 Dependabot을 들고, Cargo Book의 Continuous Integration 장은 Dependabot이나 Renovate가 의존성 갱신 PR을 정기적으로 만들게 하는 방법과, `cargo update` 뒤에 빌드와 테스트를 돌리되 실패해도 전체를 막지 않는 "Latest Dependencies" CI 작업을 예로 듭니다. 그 작업은 `rust-version` 때문에 새 버전이 걸러지지 않도록 `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=allow`를 줍니다(리졸버 3은 Part 2 ch5 2.6절). CI 구성은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서 다룹니다.

**지침을 따르지 않아도 되는 경우.** 라이브러리가 CI에서 늘 최신 의존성으로만 시험하려고 `Cargo.lock`을 커밋하지 않는 것은 Cargo Book도 드는 선택지입니다. 대신 같은 커밋의 빌드가 날마다 달라질 수 있다는 것을 받아들여야 합니다. 보안 권고가 나온 의존성은 정해 둔 때를 기다리지 않고 바로 `cargo update 이름`으로 올립니다.

### 2.5 지침 4: 그래프 점검은 도구에 맡기고 정기적으로 돌린다

**문제 상황.** 그래프는 내가 `Cargo.toml`을 고치지 않아도 바뀝니다. `cargo update` 한 번에 전이 의존성 수십 개가 바뀌고, 어제 안전했던 버전에 오늘 보안 권고가 붙기도 합니다. 쓰지 않게 된 의존성은 아무도 지우지 않습니다.

**비권장 습관.** 가끔 `Cargo.toml`을 눈으로 훑습니다. 직접 의존성만 보이고, 전이 의존성과 라이선스와 권고는 보이지 않습니다.

**권장 습관.** 무엇을 잡을지에 따라 도구를 고릅니다.

| 잡을 것 | 도구 | 이 저장소에서 |
| --- | --- | --- |
| 쓰지 않는 의존성 | Cargo의 `cargo::unused_dependencies` 린트(1.100.0부터 기본 warn), `cargo-udeps`, `cargo-machete` | 린트만 실행해 확인 |
| 중복 버전, 금지한 크레이트, 라이선스, 출처, 보안 권고 | `cargo-deny` | 설정 예시만 작성 |
| 보안 권고(RustSec) | `cargo-audit`, `cargo deny check advisories` | 설명만 |
| 그래프와 라이선스 보기 | `cargo tree`, `cargo tree -f "{p} {l}"` | 실행해 확인 |

**쓰지 않는 의존성: Cargo의 린트.** cargo 1.100.0은 빌드할 때 쓰지 않는 의존성을 경고합니다. `after`의 복사본에 `chime-sys = "0.3"`을 더하고 빌드한 결과입니다.

```text
$ cargo build
     Locking 1 package to highest Rust 1.100.0 compatible version
      Adding chime-sys v0.3.0
warning: unused dependency `chime-sys`
  --> Cargo.toml:12:1
   |
12 | chime-sys = "0.3"
   | ^^^^^^^^^^^^^^^^^
   |
   = note: `cargo::unused_dependencies` is set to `warn` by default
help: consider removing the dependency on `chime-sys`
warning: `planner` (manifest) generated 1 warning
```

`Cargo.toml`에 `[lints.cargo]`와 `unused_dependencies = "deny"`를 적으면 같은 자리가 `error: unused dependency`가 되고 빌드가 실패했습니다. 이 린트는 패키지의 어느 타깃(라이브러리, 바이너리 등)도 쓰지 않은 의존성을 알려 주는데, 그 의존성 표를 쓰는 타깃을 모두 빌드했을 때만 판단합니다. 그래서 빌드 명령에 따라 결과가 다릅니다. Cargo Book의 lints 장과 직접 실험한 결과를 합치면 다음과 같습니다.

- `[dependencies]`와 `[build-dependencies]`만 검사하고 `[dev-dependencies]`는 검사하지 않습니다(문서, 실험 모두).
- 선택적 의존성(optional dependency)은 그 기능을 켜고 빌드했을 때만 검사되었습니다. 기능을 켜지 않은 `cargo build`에서는 쓰지 않아도 경고가 없었습니다.
- `#[cfg(test)]` 코드에서만 쓰는 `[dependencies]` 항목은 `cargo build`에서 경고가 나고 `cargo test`에서는 나지 않았습니다. `[dev-dependencies]`로 옮기라는 신호입니다.
- `cargo check --bin 이름`처럼 타깃을 골라 빌드하면 `[dependencies]`를 검사하지 않습니다(문서, 실험 모두).
- 전이 의존성의 기능을 켜거나 최소 버전을 올리려고 직접 적은 의존성은 거짓 경고가 될 수 있습니다. 문서는 버전을 올리려고 적은 의존성을 `[target."cfg(false)".dependencies]`로 옮기라고 안내합니다.

rustc에도 크레이트마다 쓰지 않은 `--extern`을 알려 주는 린트 `unused_crate_dependencies`(기본 allow)가 있습니다. `cargo build -v`로 보면 cargo 1.100.0은 모든 rustc 호출에 `--json=…,unused-externs-silent`를 붙여 이 정보를 조용히 받아 모은 뒤 위 린트로 판단합니다. 그래서 라이브러리는 쓰고 바이너리는 쓰지 않는 의존성처럼 크레이트 하나만 보면 쓰지 않은 것으로 보이는 경우는 경고하지 않았고, `[lints.rust]`나 `RUSTFLAGS`로 `unused_crate_dependencies`를 켜도 rustc의 경고가 따로 나오지 않고 같은 Cargo 경고만 나왔습니다.

책이 드는 `cargo-udeps`는 같은 일을 하는 외부 도구로, 설치는 stable로 하지만 실행에는 nightly가 필요합니다(`cargo +nightly udeps`). 거짓 경고는 `Cargo.toml`의 `[package.metadata.cargo-udeps.ignore]`에 `normal = ["이름"]`처럼 적어 뺍니다. `cargo-machete`는 컴파일하지 않고 소스를 검색해서 빠르지만 덜 정확하다고 스스로 밝히는 도구이고(`cargo machete`, 예외는 `[package.metadata.cargo-machete]`의 `ignored`), 찾으면 종료 코드 1을 돌려줘 CI에 넣기 쉽습니다. 두 도구는 이 저장소에 설치돼 있지 않아 실행해 확인하지 못했습니다. 설명은 각 도구의 README를 따랐습니다.

**그래프 정책: `cargo-deny`.** `cargo-deny`는 그래프 전체(전이 의존성 포함)를 설정 파일 `deny.toml`의 정책과 대조하는 도구입니다. 네 가지 검사가 있습니다. `advisories`는 RustSec 권고 데이터베이스와 대조하고, `bans`는 금지한 크레이트와 중복 버전과 `"*"` 요구 사항을, `licenses`는 허용 목록 밖의 라이선스를, `sources`는 허용하지 않은 레지스트리나 git 저장소를 잡습니다. [`projects/real_graph/deny.toml`](projects/real_graph/deny.toml)에 이 챕터의 판단을 옮긴 예시를 두었습니다. 핵심 부분입니다.

```toml
[licenses]
allow = ["MIT", "Apache-2.0", "Unicode-3.0"]

[bans]
multiple-versions = "deny"
wildcards = "deny"
skip = [
    { crate = "syn@2.0.119", reason = "derive_builder 0.20.2가 darling 0.20(syn 2)을 씀. syn 3을 쓰는 판이 나오면 지운다" },
]
```

중복은 기본으로 경고(`"warn"`)라서 `"deny"`로 올렸고, 2.3절에서 없애는 비용이 더 크다고 판단한 `syn` 2를 이유와 함께 `skip`에 적었습니다. 검사 전체를 끄지 않고 예외 하나에 이유를 붙여 두면, 이유가 사라졌을 때(예: `derive_builder`의 새 버전) 지울 곳이 분명합니다. 실행은 `cargo deny check`(네 검사 모두)나 `cargo deny check bans`(하나만)이고, 종료 코드는 실패한 검사의 비트를 합친 값입니다(`advisories` 1, `bans` 2, `licenses` 4, `sources` 8). **`cargo-deny`는 이 저장소에 설치돼 있지 않아 이 파일로 검사를 돌려 보지 못했습니다.** 키 이름은 2026-11-19에 `cargo-deny` 문서의 설정 절에서 확인한 것만 썼습니다.

라이선스 허용 목록은 설치 없이도 만들 수 있습니다. `cargo tree`의 `-f`(`--format`)에서 `{l}`은 라이선스를 뜻합니다.

```text
$ cargo tree -f "{p} {l}" --prefix none | sort -u
anyhow v1.0.104 MIT OR Apache-2.0
darling v0.20.11 MIT
   (가운데 생략)
fnv v1.0.7 Apache-2.0 / MIT
ident_case v1.0.1 MIT/Apache-2.0
   (가운데 생략)
unicode-ident v1.0.27 (MIT OR Apache-2.0) AND Unicode-3.0
```

`unicode-ident`에는 MIT나 Apache-2.0이 아닌 `Unicode-3.0`이 **함께**(`AND`) 붙어 있어 허용 목록에 넣었습니다. `fnv`와 `ident_case`처럼 `/`로 적은 옛 표기도 섞여 있습니다.

**보안 권고: RustSec.** RustSec 권고 데이터베이스는 crates.io 크레이트의 알려진 취약점을 모은 데이터베이스이고, Rust Secure Code Working Group이 관리합니다. `cargo-audit`(`cargo audit`)는 `Cargo.lock`을 이 데이터베이스와 대조하고, `cargo-deny`의 `advisories` 검사도 같은 데이터베이스를 씁니다. `cargo-deny`의 `[advisories]` 설정에 `unmaintained`와 `unsound` 키가 따로 있는 것처럼, 권고는 취약점만이 아니라 유지되지 않는 크레이트와 건전하지 않은 API도 다룹니다. ch15·ch16에서 본 건전성 문제가 여기에 올라옵니다. 권고는 내 코드가 그대로여도 새로 붙으므로, 이 검사는 커밋할 때만이 아니라 정기적으로 돌려야 의미가 있습니다. `cargo-audit`도 설치돼 있지 않아 실행해 확인하지 못했습니다. 이 도구들을 CI에서 정기적으로 돌리는 구성은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md)에서, 도구 전체의 지도는 [`ch31-tooling-ecosystem`](../ch31-tooling-ecosystem/README.md)에서 다룹니다.

**지침을 따르지 않아도 되는 경우.** 버리고 갈 실험용 프로젝트에 정책 파일까지 둘 필요는 없습니다. 도구가 틀리게 잡은 것은 도구를 끄지 말고 이유를 적어 예외로 둡니다(`deny.toml`의 `skip`·`ignore`, 위 도구들의 metadata 표).

### 2.6 지침 5: 의존성을 들이기 전에 비용을 따진다

**문제 상황.** ch7에서는 빌더를 손으로 쓰는 대신 `derive_builder`를 썼고, ch4에서는 `thiserror`와 `anyhow`를 썼습니다. 셋을 한 프로그램에 모은 것이 [`projects/real_graph`](projects/real_graph/Cargo.toml)의 `ticket_desk`입니다. `Cargo.toml`에는 세 줄뿐입니다.

```text
$ cd appd-a/ch25-dependency-graph/projects/real_graph
$ cargo tree
ticket_desk v0.1.0 (<저장소>/appd-a/ch25-dependency-graph/projects/real_graph)
├── anyhow v1.0.104
├── derive_builder v0.20.2
│   └── derive_builder_macro v0.20.2 (proc-macro)
│       ├── derive_builder_core v0.20.2
│       │   ├── darling v0.20.11
│       │   │   ├── darling_core v0.20.11
│       │   │   │   ├── fnv v1.0.7
│       │   │   │   ├── ident_case v1.0.1
│       │   │   │   ├── proc-macro2 v1.0.107
│       │   │   │   │   └── unicode-ident v1.0.27
│       │   │   │   ├── quote v1.0.47
│       │   │   │   │   └── proc-macro2 v1.0.107 (*)
│       │   │   │   ├── strsim v0.11.1
│       │   │   │   └── syn v2.0.119
│       │   │   │       ├── proc-macro2 v1.0.107 (*)
│       │   │   │       ├── quote v1.0.47 (*)
│       │   │   │       └── unicode-ident v1.0.27
│       │   │   └── darling_macro v0.20.11 (proc-macro)
│       │   │       ├── darling_core v0.20.11 (*)
│       │   │       ├── quote v1.0.47 (*)
│       │   │       └── syn v2.0.119 (*)
│       │   ├── proc-macro2 v1.0.107 (*)
│       │   ├── quote v1.0.47 (*)
│       │   └── syn v2.0.119 (*)
│       └── syn v2.0.119 (*)
└── thiserror v2.0.21
    └── thiserror-impl v2.0.21 (proc-macro)
        ├── proc-macro2 v1.0.107 (*)
        ├── quote v1.0.47 (*)
        └── syn v3.0.10
            ├── proc-macro2 v1.0.107 (*)
            ├── quote v1.0.47 (*)
            └── unicode-ident v1.0.27
```

`(*)`는 그 패키지의 하위 트리를 이미 위에서 보였으므로 되풀이하지 않았다는 표시입니다(`--no-dedupe`로 모두 펼칠 수 있습니다). `(proc-macro)`는 절차적 매크로 크레이트입니다. 직접 의존성 셋에 패키지 17개가 따라왔습니다.

**비권장 습관.** 이름이 그럴듯한 크레이트를 `cargo add`하고 끝냅니다. 그래프가 얼마나 커지는지, 빌드할 때 무엇이 실행되는지, 누가 유지하는지 보지 않습니다. 반대로 비용이 무섭다고 모든 것을 직접 짜는 것도 비권장입니다(아래 "따르지 않아도 되는 경우").

**권장 습관.** 들이기 전에 다음 질문에 답해 봅니다.

1. **그래프가 얼마나 커지는가.** [`02_dependency_cost`](examples/02_dependency_cost.rs)는 직접 의존성마다 "빼면 함께 사라지는 패키지"를 셉니다. `derive_builder`는 13개를 거쳐 오고 그중 10개가 혼자만 쓰는 것이며, `thiserror`는 6개 중 3개입니다. `proc-macro2`, `quote`, `unicode-ident`는 둘이 나눠 쓰므로 하나를 빼도 남습니다. 새 의존성의 진짜 비용은 이미 그래프에 있는 것을 뺀 나머지입니다.
2. **무엇이 실행 파일에 들어가는가.** `cargo tree -e no-proc-macro`는 절차적 매크로로 가는 간선을 뺍니다.

   ```text
   $ cargo tree -e no-proc-macro
   ticket_desk v0.1.0 (<저장소>/appd-a/ch25-dependency-graph/projects/real_graph)
   ├── anyhow v1.0.104
   ├── derive_builder v0.20.2
   └── thiserror v2.0.21
   ```

   나머지 14개는 컴파일할 때만 쓰입니다. 그래서 이 그래프의 비용은 실행 파일 크기보다 **빌드 시간**에 있습니다(`syn`이 두 번 컴파일되는 것도 여기에 들어갑니다).
3. **빌드할 때 누구의 코드가 실행되는가.** 절차적 매크로와 빌드 스크립트(`build.rs`)는 컴파일하는 컴퓨터에서 실행되는 프로그램입니다. 의존성 하나가 악의적으로 바뀌면 개발자의 컴퓨터와 CI에서 그 코드가 돕니다. 이 그래프에서 절차적 매크로는 위 트리의 `(proc-macro)` 셋이고, 빌드 스크립트는 넷입니다.

   ```text
   $ cargo clean && cargo build -v 2>&1 | grep -o '[^/]*/build\.rs' | sort -u
   anyhow-1.0.104/build.rs
   proc-macro2-1.0.107/build.rs
   quote-1.0.47/build.rs
   thiserror-2.0.21/build.rs
   ```

4. **누가, 얼마나 유지하는가.** 최근 릴리스와 저장소의 활동, RustSec 권고(2.5절), 라이선스(`cargo tree -f "{p} {l}"`)를 봅니다. 그래프가 클수록 크레이트가 지워지거나, 라이선스가 바뀌거나, 비슷한 이름의 가짜 크레이트를 잘못 들이거나(이름 공간이 평평하므로), 관리자 계정을 빼앗기는 일에 더 많이 노출됩니다. 책이 드는 공급망(supply chain)의 위험입니다.
5. **이미 있는 것으로 되는가.** 그래프에 이미 같은 일을 하는 크레이트가 있으면 비슷한 크레이트를 하나 더 들이지 않습니다. 쓰지 않는 기본 기능을 꺼서 따라오는 것을 줄일 수도 있습니다(`default-features = false`, ch26).

막 게시된 버전을 바로 받지 않는 것도 공급망 방어가 됩니다. cargo 1.100.0에 생긴 최소 게시 경과 시간(min-publish-age) 설정을 주면, 리졸버는 게시된 지 그 시간이 지나지 않은 버전을 `Cargo.lock`에 이미 있지 않은 한 고르지 않습니다. 악의적인 릴리스가 발견되어 게시 취소될 시간을 버는 것입니다. `real_graph`의 복사본에서 `Cargo.lock`을 지우고 실행한 결과입니다(2026-11-19 기준).

```text
$ CARGO_REGISTRY_GLOBAL_MIN_PUBLISH_AGE="30 days" cargo generate-lockfile
    Updating crates.io index
     Locking 17 packages to highest Rust 1.100.0 compatible versions as of 30 days ago
      Adding syn v3.0.9 (available: v3.0.10, published 10 days ago)
```

계속 쓰려면 `.cargo/config.toml`의 `[registry]`에 `global-min-publish-age = "7 days"`처럼 적습니다. 보안 수정 릴리스도 같은 시간만큼 늦게 받게 되므로, Cargo Book의 Dependency Resolution 장(Publish age 절)은 설정 `resolver.incompatible-publish-age = "allow"`와 `cargo update --precise`를 함께 써서 특정 버전을 바로 받는 길을 설명합니다.

**지침을 따르지 않아도 되는 경우.** 비용을 따진 결론은 대개 "의존한다"입니다. 책도 직접 짜는 쪽이 대개 더 비싸다고 결론짓습니다. 직접 쓴 코드도 시험하고 유지하고 버그를 고쳐야 하고, 널리 쓰이는 크레이트는 그 일을 많은 사람이 나눠 하고 있기 때문입니다. 직접 쓰는 편이 나은 것은 필요한 부분이 몇 줄뿐인데 크레이트가 크거나, 출력 모양을 꾸미는 정도의 부수적인 용도일 때입니다. `no_std` 대상은 쓸 수 있는 크레이트가 줄어 선택지가 좁아집니다([`ch33-no-std`](../ch33-no-std/README.md)).

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. cargo 1.100.0에서 확인한 차이는 다음과 같습니다.

- **라이브러리의 `Cargo.lock` 안내가 바뀌었습니다.** 책은 Cargo 개발자들이 라이브러리에는 커밋하지 말라고 권한다고 소개합니다. Rust 블로그의 "Change in Guidance on Committing Lockfiles"(2023-08-29)가 이 안내를 "프로젝트에 맞게 정하되 커밋을 출발점으로"로 바꿨고, 같은 글에 따르면 nightly-2023-08-24부터 `cargo new`가 라이브러리에서도 `Cargo.lock`을 무시하지 않습니다. 지금의 Cargo FAQ("Why have Cargo.lock in version control?")도 커밋 여부는 패키지의 필요에 달렸다고 적습니다. 저장소 밖에서 `cargo new --lib`를 실행해 `.gitignore`가 `/target` 한 줄뿐인 것을 확인했습니다.
- **쓰지 않는 의존성을 Cargo가 직접 경고합니다.** 책은 nightly가 필요한 `cargo-udeps`를 소개합니다. 1.100.0 릴리스 노트(로컬 문서의 `releases.md`)에 "Cargo's own linting system"과 `cargo::unused_dependencies`가 새로 들어왔고, 2.5절의 실험으로 동작과 한계를 확인했습니다.
- **리졸버 3.** 2024 에디션 패키지는 의존성을 고를 때 `rust-version`을 봅니다. 2.2~2.4절의 출력에 나온 `to highest Rust 1.100.0 compatible versions`가 그 표시입니다(Part 2 ch5 2.6절).
- **`cargo update`가 더 많이 알려 줍니다.** `(available: …)` 알림, `--dry-run`, `--verbose`의 `Unchanged` 줄은 책에 없습니다. 같은 이름이 두 버전이면 `cargo tree -i`와 `cargo update`에 `이름@버전`을 써야 합니다(2.3절).
- **책에 없는 지침과 내용.** 다음은 책의 Item 25가 다루지 않는 것입니다(2026-11-19에 온라인판에서 확인).
  - 2.2절의 "고정하면 조용히 옛 중간 크레이트로 물러날 수 있다"와 "머무를 버전은 `Cargo.lock`의 `--precise`로": `pinned_fails`와 그 변형의 실행 결과, `cargo update` 문서가 근거입니다.
  - 2.3절의 `links` 거부: Cargo Book Dependency Resolution의 `links` 절과 `links_fails`의 실행 결과가 근거입니다.
  - 2.4절의 `cargo update --dry-run`·`--precise`·`이름@버전`, Renovate와 "Latest Dependencies" CI 작업: `cargo update --help`, 실행 결과, Cargo Book의 Continuous Integration 장이 근거입니다.
  - 2.5절의 `cargo::unused_dependencies`와 rustc `unused_crate_dependencies`의 관계, `cargo-machete`, `cargo-audit`과 RustSec, `cargo tree -f "{p} {l}"`: 1.100.0 릴리스 노트와 Cargo Book lints 장, `cargo build -v`의 실행 결과, 각 도구의 문서가 근거입니다.
  - 2.6절의 "빼면 사라지는 패키지"로 비용 세기, `cargo tree -e no-proc-macro`, 최소 게시 경과 시간: 예제 02, `cargo tree` 문서, 1.100.0 릴리스 노트, Cargo Book의 config 장과 Dependency Resolution 장(Publish age 절), 실행 결과가 근거입니다.
  - 2.3절에서 `syn` 중복을 `--precise`로 없앨 수 있지만 받아들이는 판단: crates.io 인덱스(`thiserror-impl` 2.0.19부터 `syn = "^3"`)와 `--dry-run` 결과가 근거입니다.

### 2.8 기억할 것

- 같은 크레이트의 요구 사항은 호환되면 하나로 합쳐지고, 호환되지 않으면 사본이 따로 생기고, 겹치지 않으면 해석이 실패합니다. `links`가 같은 두 버전은 아예 함께 둘 수 없습니다.
- 요구 사항은 캐럿 범위로 적고 `=` 고정은 피합니다. 같은 버전으로 빌드하는 일은 `Cargo.lock`이 하고, 잠시 머물러야 하면 `--precise`로 잠금 파일만 바꿉니다.
- 중복은 `cargo tree -d`와 `-i 이름@버전`으로 출처를 찾아, 옛 버전을 끌고 오는 중간 크레이트를 모두 올려서 없앱니다. 없앨 수 없는 중복은 이유를 적어 둡니다.
- 애플리케이션은 `Cargo.lock`을 커밋하고, `cargo update --dry-run`으로 보고 작게 올리며, 갱신은 Dependabot·Renovate 같은 자동화에 맡깁니다.
- 쓰지 않는 의존성, 중복, 라이선스, 보안 권고는 도구로 정기적으로 점검합니다. 새 의존성은 들이기 전에 늘어나는 패키지와 빌드 때 실행되는 코드를 확인하되, 결론은 대개 "의존한다"입니다.

## 3. 예제 안내

예제 둘은 저장소 루트에서 실행하고, 이 챕터의 `projects/`에 둔 `Cargo.lock`을 `include_str!`로 읽습니다. 외부 크레이트와 네트워크를 쓰지 않으며 출력은 실행마다 같습니다. 프로젝트는 각 디렉터리에서 실행합니다.

| 파일 | 지침 | 보여 주는 것 | 관련 절 |
| --- | --- | --- | --- |
| [`01_lockfile_duplicates.rs`](examples/01_lockfile_duplicates.rs) | 2, 3 | 잠금 파일에서 중복 버전과 그것을 요구한 패키지 찾기 | 2.3, 2.4 |
| [`02_dependency_cost.rs`](examples/02_dependency_cost.rs) | 5 | 직접 의존성마다 거쳐 오는 패키지와 빼면 사라지는 패키지 | 2.3, 2.6 |
| [`projects/planner`](projects/planner/before/Cargo.toml) | 1, 2, 3, 4 | 연습용 레지스트리와 패키지 넷(`before`, `after`, `pinned_fails`, `links_fails`) | 2.1~2.5 |
| [`projects/real_graph`](projects/real_graph/Cargo.toml) | 3, 4, 5 | crates.io의 실제 그래프, `deny.toml` | 2.3~2.6 |

### 01_lockfile_duplicates — 잠금 파일이 적는 그래프

```sh
cargo run -p er-ch25-dependency-graph --example 01_lockfile_duplicates
```

```text
[projects/planner/before] 패키지 5개 (바깥 출처 4개)
  중복: tiny-date 1.6.0 ← cal-grid 0.9.1, remind-kit 0.4.2
  중복: tiny-date 2.1.0 ← planner 0.1.0
  planner 0.1.0의 dependencies = ["cal-grid", "remind-kit", "tiny-date 2.1.0"]
[projects/planner/after] 패키지 4개 (바깥 출처 3개)
  중복: 없음
  planner 0.2.0의 dependencies = ["cal-grid", "remind-kit", "tiny-date"]
[projects/real_graph] 패키지 18개 (바깥 출처 17개)
  중복: syn 2.0.119 ← darling_core 0.20.11, darling_macro 0.20.11, derive_builder_core 0.20.2, derive_builder_macro 0.20.2
  중복: syn 3.0.10 ← thiserror-impl 2.0.21
  ticket_desk 0.1.0의 dependencies = ["anyhow", "derive_builder", "thiserror"]
```

결과는 2.3절의 `cargo tree -d`와 같습니다. 같은 이름이 하나뿐인 `after`에서는 `"tiny-date"`, 둘인 `before`에서는 `"tiny-date 2.1.0"`처럼 Cargo가 필요할 때만 버전을 붙여 적는 것도 보입니다. 다만 잠금 파일에는 다른 플랫폼에서만 쓰는 패키지도 적히므로, 실제로 빌드되는 그래프는 `cargo tree`가 기준입니다.

### 02_dependency_cost — 빼면 사라지는 패키지

```sh
cargo run -p er-ch25-dependency-graph --example 02_dependency_cost
```

```text
[projects/real_graph] ticket_desk 0.1.0: 직접 의존성 3개, 함께 들어온 패키지 17개
  anyhow 1.0.104: 거쳐 들어오는 패키지 1개, 빼면 사라지는 패키지 1개
    사라지는 것: anyhow 1.0.104
  derive_builder 0.20.2: 거쳐 들어오는 패키지 13개, 빼면 사라지는 패키지 10개
    사라지는 것: darling 0.20.11, darling_core 0.20.11, darling_macro 0.20.11, derive_builder 0.20.2, derive_builder_core 0.20.2, derive_builder_macro 0.20.2, fnv 1.0.7, ident_case 1.0.1, strsim 0.11.1, syn 2.0.119
  thiserror 2.0.21: 거쳐 들어오는 패키지 6개, 빼면 사라지는 패키지 3개
    사라지는 것: syn 3.0.10, thiserror 2.0.21, thiserror-impl 2.0.21
  나눠 쓰는 것: proc-macro2 1.0.107, quote 1.0.47, unicode-ident 1.0.27
[projects/planner/before] planner 0.1.0: 직접 의존성 3개, 함께 들어온 패키지 4개
  cal-grid 0.9.1: 거쳐 들어오는 패키지 2개, 빼면 사라지는 패키지 1개
    사라지는 것: cal-grid 0.9.1
  remind-kit 0.4.2: 거쳐 들어오는 패키지 2개, 빼면 사라지는 패키지 1개
    사라지는 것: remind-kit 0.4.2
  tiny-date 2.1.0: 거쳐 들어오는 패키지 1개, 빼면 사라지는 패키지 1개
    사라지는 것: tiny-date 2.1.0
  나눠 쓰는 것: tiny-date 1.6.0
```

`real_graph`의 `syn` 2와 3은 각각 `derive_builder`와 `thiserror`만의 비용입니다. `planner`의 `tiny-date` 1.6.0은 "나눠 쓰는 것"이라, `remind-kit` 하나만 올려서는 사라지지 않습니다(2.3절). 두 잠금 파일의 패키지 목록은 `cargo tree`에 나오는 것과 같습니다.

### projects/planner — 연습용 레지스트리

`projects/planner/.cargo/config.toml`이 crates.io를 `vendor/`로 바꾸므로(2.1절), 아래 명령은 `projects/planner` 아래의 각 디렉터리에서 실행해야 합니다. Cargo는 설정 파일을 현재 디렉터리에서 위로 찾기 때문에, 저장소 루트에서 `--manifest-path`로 실행하면 이 설정을 읽지 않고 진짜 crates.io를 뒤지다가 ``error: no matching package named `cal-grid` found``로 끝납니다. 네 패키지는 각자 빈 `[workspace]`를 가진 별개의 워크스페이스입니다.

| 디렉터리 | 패키지 | 실행 | 기대 결과 | 관련 절 |
| --- | --- | --- | --- | --- |
| `before` | `planner` 0.1.0 | `cargo run -q`, `cargo tree -d`, `cargo update --dry-run --verbose` | 종료 코드 0. `tiny-date` 사본 2.1.0과 1.6.0 | 2.3 |
| `after` | `planner` 0.2.0 | `cargo run -q`, `cargo tree -d` | 종료 코드 0. 사본 2.1.0 하나, `warning: nothing to print.` | 2.2, 2.3 |
| `pinned_fails` | `planner` 0.2.0 | `cargo build` | **일부러 실패**, 종료 코드 101, ``failed to select a version for `tiny-date`.`` | 2.2 |
| `links_fails` | `planner` 0.3.0 | `cargo build` | **일부러 실패**, 종료 코드 101, ``package `chime-sys` links to the native library `chime` `` | 2.3 |

`before`와 `after`는 `cargo clippy -- -D warnings`와 `cargo fmt --check`를 통과하고, 실패하는 두 패키지도 `cargo fmt --check`는 통과합니다. `before/Cargo.lock`은 `remind-kit` 0.4.3이 "게시되기 전"에 만든 것으로 쳐서 0.4.2를 적어 두었습니다. 이 디렉터리에서 `cargo update`를 실행하면 2.3절처럼 0.4.3으로 바뀌니, 원래대로 돌리려면 `git checkout Cargo.lock`을 실행합니다.

### projects/real_graph — crates.io의 실제 그래프

```sh
cd appd-a/ch25-dependency-graph/projects/real_graph
cargo run -q
```

```text
겨울 음악회: 100석
30석 예약 → 남은 좌석 70석
500석 예약 → 좌석이 모자랍니다: 남은 100석, 요청 500석
```

처음 빌드할 때 로컬 캐시에 없는 크레이트를 crates.io에서 내려받습니다(전부 17개). **버전과 출력은 2026-11-19의 crates.io 기준**이며, 함께 둔 `Cargo.lock`이 그날의 버전(`anyhow` 1.0.104, `derive_builder` 0.20.2, `thiserror` 2.0.21 등)을 고정하므로 `cargo build`의 결과는 같습니다. `cargo update`나 `--dry-run`의 결과, `Updating crates.io index` 뒤의 내용은 그 시점의 crates.io를 보므로 달라질 수 있습니다. `cargo tree -d`를 실행하면 2.3절의 `syn` 두 버전이 나오고, `cargo tree`·`cargo tree -e no-proc-macro`·`cargo update --precise … --dry-run`의 출력은 2.4~2.6절에 있습니다. `cargo clippy -- -D warnings`, `cargo fmt --check`가 통과합니다. 테스트는 없습니다. [`deny.toml`](projects/real_graph/deny.toml)은 설치하지 않은 `cargo-deny`용 예시입니다(2.5절).

## 4. 자주 하는 실수

이 챕터의 실수는 컴파일 에러보다 의존성 해석과 습관에서 나오므로 `compile_fail/`을 두지 않았습니다. 증상 칸의 메시지는 모두 이 툴체인에서 직접 실행해 얻은 것입니다.

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| 시험한 버전으로 `=x.y.z` 고정 | ``failed to select a version for `tiny-date` ``, 또는 경고 없이 옛 중간 크레이트로 물러나 중복이 되살아남 | 캐럿 요구 사항 + `Cargo.lock`, 머물려면 `--precise` | 2.2 |
| 두 버전이 있는 이름을 그대로 지정 | ``error: specification `tiny-date` is ambiguous`` (`cargo tree -i`, `cargo update`) | `tiny-date@1.6.0`처럼 버전을 붙임 | 2.3 |
| `cargo update`로 범위 밖 버전이 올 거라 기대 | `Unchanged cal-grid v0.9.1 (available: v0.10.0)`만 나오고 그대로 | 요구 사항을 고치고 코드를 맞춤 | 2.3 |
| 중간 크레이트 하나만 올리고 중복이 사라졌다고 여김 | `cargo tree -d`에 옛 사본이 남음 | `-i 이름@버전`으로 옛 사본을 쓰는 곳을 모두 찾음 | 2.3, 02 |
| `cargo tree -d`가 비었으니 중복이 없다고 단정 | 다른 플랫폼 빌드에서 중복이 나옴 | `--target all`, `--all-features`로도 확인 | 2.3 |
| `links`가 같은 `-sys` 크레이트의 두 버전에 의존(dev-dependency 포함) | ``package `chime-sys` links to the native library `chime`, but it conflicts …`` | 두 쪽이 같은 버전을 쓰게 맞춤 | 2.3 |
| 문제가 생기면 `Cargo.lock`을 지움 | 모든 패키지가 한꺼번에 새로 골라져 원인을 가리기 어려움 | `cargo update 이름`, `--precise`로 하나씩 | 2.4 |
| 테스트에서만 쓰는 크레이트를 `[dependencies]`에 둠 | `cargo build`에서 ``warning: unused dependency`` | `[dev-dependencies]`로 옮김 | 2.5 |
| 중복 검사를 통째로 끔 | 새로 생긴 중복도 보이지 않음 | 없앨 수 없는 것만 이유와 함께 `skip` | 2.5 |

## 5. 연습 문제

### 연습 1. 중복이 어디까지 줄어드나

[`projects/planner/before`](projects/planner/before/Cargo.toml)에서 `cargo update`만 실행하면 `cargo tree -d`와 `cargo run -q`의 "tiny-date 사본" 세 줄이 어떻게 될지 먼저 예측하세요. 그다음 디렉터리를 복사해서(또는 실행한 뒤 `git checkout Cargo.lock`으로 되돌려서) 확인하세요.

<details>
<summary>힌트</summary>

`cargo update --dry-run --verbose`가 무엇을 올리고 무엇을 그대로 두는지부터 봅니다. 올라오는 것은 요구 사항 `"0.4"` 안에 있는 `remind-kit` 0.4.3뿐이고, 0.4.3은 `tiny-date` 2.x를 씁니다. `cal-grid` 0.9.1은 그대로이므로 `tiny-date` 1.6.0은 `cal-grid`를 위해 남습니다. 사본 세 줄은 2.1.0, 2.1.0, 1.6.0이 됩니다. `02_dependency_cost`의 `before` 출력에서 1.6.0이 "나눠 쓰는 것"이었던 이유입니다.

</details>

### 연습 2. 고정 풀기

[`projects/planner/pinned_fails`](projects/planner/pinned_fails/Cargo.toml)가 빌드되도록 `tiny-date`의 요구 사항을 고치세요. 고칠 수 있는 값을 둘 이상 찾고, 각각 `Cargo.lock`에 어떤 버전이 적히는지 확인하세요. `"=2.1.0"`도 지금은 빌드되는데 왜 권하지 않는지 설명해 보세요.

<details>
<summary>힌트</summary>

`"2"`, `"2.0"`, `"2.1"`은 모두 `^2.1`과 겹쳐 2.1.0이 골라집니다. 이 패키지가 쓰는 API(`Date::new`가 `Option`을 돌려주는 것)는 2.0.0부터 있으므로 아래쪽 경계로는 `"2"`면 충분합니다. `"=2.1.0"`은 지금은 겹치지만, 나중에 `remind-kit`이나 `cal-grid`의 새 버전이 `"2.2"`를 요구하면 같은 에러가 다시 납니다. 2.2.x의 수정 사항도 받지 못합니다. 고친 뒤 생긴 `Cargo.lock`은 지워 두어야 이 디렉터리가 "일부러 실패하는 패키지"로 남습니다.

</details>

### 연습 3. `links` 충돌 풀기

[`projects/planner/links_fails`](projects/planner/links_fails/Cargo.toml)를 빌드되게 고치세요. 연습용 레지스트리에는 `ring-tone`의 버전이 1.0.0 하나뿐입니다. 실행하면 두 줄 모두 같은 `chime-sys` 버전을 출력해야 합니다.

<details>
<summary>힌트</summary>

`ring-tone`이 `chime-sys` 0.3을 쓰는 버전을 낼 때까지는 내 쪽을 `chime-sys = "0.2"`로 맞추는 것이 방법입니다. 두 요구 사항이 모두 0.2.x이면 호환되므로 하나로 합쳐지고, `links` 값을 가진 패키지도 하나가 됩니다. `chime-sys`를 직접 쓸 필요가 없다면 직접 의존성을 지우는 것도 방법입니다(그때는 `main.rs`의 첫 줄도 지웁니다). 실제 프로젝트라면 `ring-tone` 저장소에 새 버전을 요청합니다.

</details>

### 연습 4. 중복까지 가는 경로

[`01_lockfile_duplicates.rs`](examples/01_lockfile_duplicates.rs)를 고쳐, 중복된 버전마다 프로젝트 자신에서 그 버전까지 가는 경로 하나를 `planner 0.1.0 → cal-grid 0.9.1 → tiny-date 1.6.0`처럼 출력하세요. `real_graph`의 `syn` 2.0.119까지 가는 경로는 몇 단계인가요?

<details>
<summary>힌트</summary>

패키지 ID에서 그 의존성 목록으로 가는 맵을 만들고, 루트에서 너비 우선 탐색을 하면서 처음 도착한 패키지마다 "어디서 왔는지"를 기록합니다. 목표에 닿으면 기록을 거꾸로 따라가 경로를 만듭니다. 의존성 표기를 패키지 ID로 바꾸는 방법은 02 예제의 `resolve` 함수를 참고합니다. 너비 우선 탐색은 가장 짧은 경로를 찾으므로 `ticket_desk → derive_builder → derive_builder_macro → syn`처럼 나올 수 있습니다. `cargo tree -i syn@2.0.119`의 출력과 비교해 보세요.

</details>

## 6. 참고 자료

- 기준 문서: [Item 25: Manage your dependency graph](https://effective-rust.com/dep-graph.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- Cargo Book
  - [Dependency Resolution](https://doc.rust-lang.org/cargo/reference/resolver.html): SemVer compatibility, `links`, Recommendations, Troubleshooting(중복과 `cargo tree`), Publish age 절
  - [Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html), [FAQ: Why have Cargo.lock in version control?](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control)
  - [Continuous Integration: Verifying Latest Dependencies](https://doc.rust-lang.org/cargo/guide/continuous-integration.html#verifying-latest-dependencies)
  - [Lints: `unused_dependencies`](https://doc.rust-lang.org/cargo/reference/lints.html)
  - [Configuration](https://doc.rust-lang.org/cargo/reference/config.html): `registry.global-min-publish-age`, `resolver.incompatible-publish-age`
  - [Source Replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html): 연습용 레지스트리에 쓴 디렉터리 소스
  - 명령: [`cargo tree`](https://doc.rust-lang.org/cargo/commands/cargo-tree.html), [`cargo update`](https://doc.rust-lang.org/cargo/commands/cargo-update.html)
- Rust 블로그: [Change in Guidance on Committing Lockfiles](https://blog.rust-lang.org/2023/08/29/committing-lockfiles/) (2023-08-29)
- Rust 1.100.0 릴리스 노트의 Cargo 항목: Cargo의 린트 체계와 `cargo::unused_dependencies`, 최소 게시 경과 시간(로컬 문서의 `releases.md`에서 확인)
- 점검 도구(이 저장소에는 설치돼 있지 않음): [cargo-deny](https://embarkstudios.github.io/cargo-deny/), [cargo-udeps](https://github.com/est31/cargo-udeps), [cargo-machete](https://github.com/bnjbvr/cargo-machete), [RustSec 권고 데이터베이스](https://rustsec.org/)
