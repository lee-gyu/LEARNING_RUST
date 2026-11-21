# ch6. 프로필과 빌드 성능: `[profile]`, 빌드 캐시, `--timings`

[Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md)에서 `cargo build`의 마지막 줄에 나오는 `dev`와 `release`가 빌드 설정 묶음인 프로필(profile)의 이름이라는 것만 보고 넘어갔습니다. 이 챕터는 그 묶음 안에 무엇이 들어 있는지, 묶음을 직접 만들고 패키지마다 다르게 적용하는 방법을 다룹니다. 이어서 빌드 결과가 쌓이는 빌드 캐시(build cache)의 구조와, 빌드가 어디서 시간을 쓰는지 재고 줄이는 방법을 봅니다.

- 패키지: `p3-ch6-profiles-build-performance`
- 기준 문서: The Cargo Book [2.11 Optimizing Build Performance](https://doc.rust-lang.org/cargo/guide/build-performance.html), [3.5 Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html), [3.9 Build Cache](https://doc.rust-lang.org/cargo/reference/build-cache.html), [3.15 Reporting build timings](https://doc.rust-lang.org/cargo/reference/timings.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64)
- 실습 자료: `examples/` 2개, 독립 프로젝트 [`projects/collatz`](projects/collatz/Cargo.toml)(바이너리 패키지와 라이브러리 패키지). `compile_fail/`은 없습니다. 이 챕터의 실수는 `rustc`가 아니라 Cargo가 매니페스트를 읽을 때 거부하므로 4절에 메시지를 표로 모았습니다.
- 외부 크레이트: `projects/collatz`가 `thiserror` 2.0.21을 씁니다. `thiserror`가 끌어오는 `thiserror-impl`, `proc-macro2`, `quote`, `syn`, `unicode-ident`까지 처음 빌드할 때 crates.io에서 내려받습니다.
- 실행 시간과 파일 크기는 위 환경의 한 컴퓨터에서 잰 값입니다. 운영체제, 툴체인, 컴퓨터에 따라 달라집니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 내장 프로필(`dev`·`release`·`test`·`bench`·`debug`)과 명령마다 고르는 프로필을 말하고, 주요 설정(`opt-level`, `debug`, `strip`, `debug-assertions`, `overflow-checks`, `lto`, `codegen-units`, `panic`, `incremental`)이 무엇과 무엇을 맞바꾸는지 설명할 수 있다.
2. `inherits`로 사용자 정의 프로필을 만들어 `--profile`로 고르고, 패키지별 재정의(`package.<이름>`, `package."*"`, `build-override`)의 우선순위와 제네릭 함수에서의 한계를 설명할 수 있다.
3. 같은 소스를 프로필만 바꿔 빌드해 실행 파일 크기와 실행 시간을 비교하고, `cargo build -v`에서 프로필 설정이 `rustc` 옵션으로 넘어간 모습을 읽을 수 있다.
4. target 디렉터리와 build 디렉터리를 구별하고, 프로필별 출력 디렉터리와 `.d` 파일, 다시 컴파일된 이유(`Dirty`)를 읽을 수 있다.
5. `cargo build --timings` 보고서를 읽고, Cargo Book이 권하는 빌드 시간 단축 방법을 그 대가와 함께 설명할 수 있다.

## 2. 핵심 개념

### 2.1 프로필이 푸는 문제

컴파일러에게 바라는 것은 상황마다 다릅니다. 코드를 고치고 바로 확인하는 동안에는 컴파일이 빨라야 하고, 패닉이 나면 어느 줄인지 알아야 하고, 디버거로 변수를 들여다볼 수 있어야 합니다. 배포할 프로그램은 컴파일이 오래 걸려도 빨리 실행되고 작아야 합니다. 이 바람들은 서로 부딪칩니다. 최적화를 많이 할수록 컴파일이 느려지고, 최적화된 코드는 원래 소스와 모양이 달라 디버거로 따라가기 어렵습니다. 디버그 정보를 넣으면 디버거는 편해지지만 컴파일·링크 시간과 디스크 사용량이 늘어납니다.

`rustc`에는 이것들을 정하는 옵션(`-C opt-level`, `-C debuginfo` 등)이 따로따로 있습니다. 프로필은 그 옵션들을 목적에 맞게 미리 묶어 이름을 붙인 것입니다. Cargo에 내장된 프로필은 다음과 같습니다.

| 프로필 | 기본으로 쓰는 명령 | 물려받는 프로필 | 출력 디렉터리 |
| --- | --- | --- | --- |
| `dev` | `cargo build`, `cargo run`, `cargo check`, `cargo rustc` | (없음) | `target/debug/` |
| `release` | `--release`를 붙인 명령, `cargo install` | (없음) | `target/release/` |
| `test` | `cargo test` | `dev` | `target/debug/` |
| `bench` | `cargo bench` | `release` | `target/release/` |
| `debug` | `--profile debug`, `cargo install --debug` | `dev` | `target/debug/` |

- `test`와 `bench`는 `dev`·`release`를 그대로 물려받으므로 따로 고치지 않으면 설정이 같습니다. `cargo test`의 `Finished` 줄은 ``Finished `test` profile [unoptimized + debuginfo]``, `cargo bench`는 ``Finished `bench` profile [optimized]``입니다.
- `dev`의 결과가 `target/dev/`가 아니라 `target/debug/`에 놓이는 것은 역사적인 이유입니다(Cargo Book 3.9절).
- `debug` 프로필은 디버거로 실행할 실행 파일을 만들 때 쓰라는 프로필입니다. Cargo Book 3.5절의 첫머리는 내장 프로필이 넷이라고 적지만, 같은 절의 기본 프로필 목록에는 `debug`가 있습니다. 이 툴체인에서 `cargo build` 뒤에 `cargo build --profile debug`를 실행하면 아무것도 다시 컴파일하지 않고 `target/debug/`의 결과를 그대로 썼습니다.
- 다른 프로필을 고를 때는 `--profile <이름>`을 씁니다. `--release`는 `--profile release`와 같습니다. 둘을 함께 주면 에러입니다(4절).
- 고른 프로필은 그 빌드의 모든 타깃(라이브러리, 바이너리, 예제, 테스트, 벤치마크)에 적용됩니다. 패키지마다 다르게 하려면 2.4절의 재정의를 씁니다.

**프로필은 워크스페이스 루트의 매니페스트에만 적습니다.** 의존성 패키지나 워크스페이스 멤버의 `[profile]`은 무시됩니다. 최적화 수준 같은 빌드 설정은 한 빌드에 하나이고, 그것을 정하는 것은 최종 산출물을 만드는 쪽이기 때문입니다. 의존성이 자기 프로필을 강요할 수 있다면, 같은 의존성을 쓰는 프로젝트마다 원하지 않는 설정을 떠안게 됩니다. [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.3절과 [ch4](../ch4-workspaces/README.md) 2.7절에서 멤버에 적은 프로필이 경고와 함께 무시되는 것을 봤습니다. 이 저장소의 챕터 패키지는 모두 루트 워크스페이스의 멤버라서, 이 챕터의 예제는 `dev`와 `release`만 쓰고 프로필을 직접 만드는 실습은 독립 프로젝트 `projects/collatz`에서 합니다.

매니페스트 말고 Cargo 설정 파일이나 환경 변수로도 프로필 설정을 바꿀 수 있고, 그 값이 `Cargo.toml`보다 우선합니다. 예제 02에서 `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only`처럼 환경 변수로 한 설정만 바꿔 보고, 설정 파일은 Part 3 `ch7-configuration-env`에서 다룹니다.

### 2.2 프로필 설정

자주 쓰는 설정과 두 기본 프로필의 값입니다. 각 설정은 같은 이름의 `rustc` 옵션(`-C <설정>`)으로 넘어갑니다.

| 설정 | 값 | `dev` | `release` | 맞바꾸는 것 |
| --- | --- | --- | --- | --- |
| `opt-level` | `0`~`3`, `"s"`, `"z"` | `0` | `3` | 실행 속도 ↔ 컴파일 시간, 디버거로 따라가기 쉬움 |
| `debug` | `false`(`0`, `"none"`), `"line-tables-only"`, `"limited"`(`1`), `true`(`2`, `"full"`) | `true` | `false` | 백트레이스·디버거에 쓸 정보 ↔ 컴파일·링크 시간, 디스크 |
| `strip` | `"none"`, `"debuginfo"`, `"symbols"`(`true`) | 아래 참고 | 아래 참고 | 실행 파일 크기 ↔ 백트레이스·디버거에 쓸 정보 |
| `debug-assertions` | `true`, `false` | `true` | `false` | 개발 중에만 하는 검사 ↔ 실행 속도 |
| `overflow-checks` | `true`, `false` | `true` | `false` | 정수 오버플로를 패닉으로 잡기 ↔ 실행 속도 |
| `lto` | `false`, `true`(`"fat"`), `"thin"`, `"off"` | `false` | `false` | 크레이트를 넘나드는 최적화 ↔ 링크 시간 |
| `codegen-units` | 1 이상의 정수 | `256` | `16` | 병렬 컴파일 ↔ 최적화 기회 |
| `panic` | `"unwind"`, `"abort"` | `"unwind"` | `"unwind"` | 되감기와 `catch_unwind` ↔ 실행 파일 크기 |
| `incremental` | `true`, `false` | `true` | `false` | 다시 컴파일할 때의 속도 ↔ 디스크, 처음 빌드 시간 |

설정마다 알아 둘 점입니다.

- **`opt-level`**: `"s"`와 `"z"`는 크기를 줄이는 쪽으로 최적화합니다(`"z"`는 반복문 벡터화도 끕니다). Cargo Book은 `3`이 `2`보다 느리거나 `"s"`·`"z"`가 꼭 더 작지는 않은 "놀라운 결과"가 있을 수 있으니 직접 재 보라고 합니다. `collatz`에서 `opt-level = "z"`만 바꾼 빌드는 `release`보다 오히려 몇백 바이트 컸습니다(연습 1에서 직접 잽니다).
- **`debug`**: `"line-tables-only"`는 백트레이스에 파일 이름과 줄 번호를 남길 만큼만 넣고 변수·매개변수 정보는 넣지 않습니다. 예제 02에서 차이를 봅니다.
- **`split-debuginfo`**: 디버그 정보를 실행 파일 안에 넣을지 옆에 둘지 정합니다. macOS에서 Cargo의 기본값은 `"unpacked"`이고, 이때 디버그 정보는 컴파일 중간에 만든 오브젝트 파일(`.o`)에 남고 실행 파일에는 들어가지 않습니다(`rustc` 문서). 그래서 이 환경에서는 `dev` 실행 파일이 `release`보다 크게 커지지 않습니다(3절).
- **`strip`**: 링크할 때 실행 파일에서 디버그 정보(`"debuginfo"`)나 함수 이름 등이 든 심볼 표(symbol table) 전체(`"symbols"`)를 지웁니다. Cargo Book의 기본값 목록에는 두 프로필 모두 `strip = "none"`으로 적혀 있지만, cargo 1.100.0은 프로필이나 재정의의 값으로 `debug`가 꺼진 크레이트에 `-C strip=debuginfo`를 넘겼습니다(3절의 `-v` 표). 다만 `dev`의 빌드 의존성은 2.4절의 기본값 때문에 디버그 정보를 만들지 않는데도 `strip`이 붙지 않았습니다. `"symbols"`는 함수 이름까지 지우므로 백트레이스가 읽을 수 없게 됩니다(예제 02).
- **`debug-assertions`와 `overflow-checks`**: `debug-assertions`는 `cfg(debug_assertions)`를 켜고 끄며, 꺼지면 `debug_assert!`의 조건식은 평가되지도 않습니다. `overflow-checks`는 [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md)에서 본 "디버그 빌드는 오버플로에서 패닉, 릴리스 빌드는 래핑"을 정하는 설정입니다. 두 설정 모두 빌드 모드가 아니라 프로필의 값이므로, 릴리스 빌드에서도 `overflow-checks = true`로 켤 수 있습니다. 예제 01에서 봅니다.
- **`lto`**: 링크 시점 최적화(link time optimization)입니다. 보통은 크레이트마다 따로 최적화하지만, LTO는 링크할 때 의존성 그래프 전체를 한 덩어리로 보고 다시 최적화합니다. `true`(`"fat"`)는 전체를, `"thin"`은 비슷한 효과를 훨씬 짧은 시간에 노립니다. 기본값 `false`는 "자기 크레이트의 조각(아래 `codegen-units`) 사이에서만 하는 thin LTO"이고(`codegen-units`가 1이거나 `opt-level`이 0이면 하지 않음), `"off"`는 그것마저 끕니다.
- **`codegen-units`**: 크레이트 하나를 몇 조각으로 나눠 기계어를 만들지 정합니다. 조각이 많으면 병렬로 빨리 컴파일하지만 조각 사이의 최적화 기회를 잃습니다. `1`은 최적화를 가장 많이, 컴파일은 가장 느리게 합니다.
- **`panic`**: `"abort"`는 패닉하면 되감지 않고 프로세스를 즉시 끝냅니다. 되감기와 abort의 차이는 [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.3절에서 봤고, 사용자 정의 프로필로 둘을 비교하는 실습과 종료 상태(macOS에서 134)는 [부록 A ch18](../../appd-a/ch18-dont-panic/README.md)의 `projects/panic_strategy`에 있습니다. 테스트와 벤치마크, 빌드 스크립트, 절차적 매크로는 이 설정을 무시합니다. 테스트와 벤치마크는 테스트 러너가 되감기를 전제로 하기 때문입니다.
- **`incremental`**: 증분 컴파일(incremental compilation)은 다음 컴파일 때 다시 쓸 정보를 디스크에 남겨, 조금 고친 코드를 빨리 다시 컴파일하게 합니다. 워크스페이스 멤버와 path 의존성에만 적용되고, crates.io에서 받은 의존성은 고칠 일이 없으므로 쓰지 않습니다.
- 이 밖에 `rpath`가 있습니다. 내장 프로필의 기본값을 바꿀 일이 드문 설정이라 이 챕터에서는 다루지 않습니다.

**`Finished` 줄의 대괄호.** `[unoptimized + debuginfo]`와 `[optimized]`는 고른 프로필의 `opt-level`과 `debug`를 요약한 것입니다. `strip`이나 `lto` 같은 나머지 설정은 나타내지 않습니다. 3절의 `small` 프로필은 크기를 줄이는 설정을 다섯 개 바꿨지만 `[optimized]`로만 나옵니다.

### 2.3 사용자 정의 프로필

내장 프로필 말고도 이름을 붙인 프로필을 만들 수 있습니다. 개발용과 배포용 말고도 "크기를 최대한 줄인 배포용", "CI에서 쓰는 빌드"처럼 작업 방식이 여럿일 때 씁니다. `projects/collatz`의 `Cargo.toml`에 있는 프로필 하나입니다.

```toml
# release를 물려받고, 실행 파일 크기를 줄이는 설정을 모았습니다.
[profile.small]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

- **`inherits`는 반드시 적습니다.** 적지 않은 설정을 어느 프로필에서 가져올지 정하는 키입니다. 빠뜨리면 ``profile `small` is missing an `inherits` directive`` 에러가 납니다(4절). 내장 프로필뿐 아니라 다른 사용자 정의 프로필을 물려받을 수도 있습니다. 자기 자신을 물려받는 식으로 고리가 생기면 에러입니다.
- **`--profile small`로 고릅니다.** 결과는 프로필 이름과 같은 디렉터리 `target/small/`에 놓입니다. 프로필마다 디렉터리가 따로 있으므로 프로필을 오가도 서로의 결과를 지우지 않습니다.

```console
$ cargo build --profile small
…
    Finished `small` profile [optimized] target(s) in 2.35s
$ ls target
CACHEDIR.TAG  debug  dev-hot  lean  release  small
```

위 목록은 `projects/collatz`에서 다섯 프로필(`dev`, `dev-hot`, `release`, `small`, `lean`)로 모두 빌드한 뒤의 모습입니다. `dev`는 `debug/`에 들어갑니다.

### 2.4 패키지마다 다르게: 재정의

프로필은 빌드 전체에 적용되지만, 패키지 하나만 다르게 하고 싶을 때가 있습니다. 개발 중에도 계산이 몰린 라이브러리 하나만은 최적화해서 테스트를 빨리 돌리고 싶은 경우가 그렇습니다. 프로필 아래에 `package` 표를 두어 패키지별로 재정의(override)합니다.

```toml
# dev를 물려받고, 계산이 몰린 `chain` 패키지만 최적화합니다.
[profile.dev-hot]
inherits = "dev"

[profile.dev-hot.package.chain]
opt-level = 3
```

재정의를 적는 자리는 세 가지이고, 한 설정에 여러 값이 걸리면 위에서부터 처음 맞는 것을 씁니다.

| 순서 | 자리 | 적용 대상 |
| --- | --- | --- |
| 1 | `[profile.<프로필>.package.<이름>]` | 이름을 적은 패키지 |
| 2 | `[profile.<프로필>.package."*"]` | 워크스페이스 멤버가 **아닌** 모든 패키지(의존성) |
| 3 | `[profile.<프로필>.build-override]` | 빌드 스크립트, 절차적 매크로, 그리고 그것들의 의존성 |
| 4 | `[profile.<프로필>]` | 프로필 자체의 설정 |
| 5 | (Cargo의 기본값) | |

- 패키지 이름 자리는 패키지 ID 지정(Package ID Spec)이라서 `[profile.dev.package."foo:2.1.0"]`처럼 버전까지 골라 적을 수 있습니다.
- `"*"`는 멤버에는 적용되지 않습니다. 멤버는 지금 고치고 있는 코드라서 디버그 정보와 증분 컴파일이 필요하고, 의존성은 한 번 빌드하면 거의 다시 빌드하지 않기 때문입니다. 2.7절의 권장 설정이 이 점을 이용합니다.
- `panic`, `lto`, `rpath`는 재정의할 수 없습니다(4절). 셋 다 크레이트마다 달라서는 안 되고 최종 링크 결과 전체에 걸리는 설정입니다.

**빌드 의존성은 기본으로 최적화하지 않습니다.** 빌드 스크립트(Part 3 `ch8-build-scripts`)와 절차적 매크로는 컴파일하는 도중에 한 번 실행되고 끝나는 프로그램이라, 오래 걸려 최적화해도 얻는 것이 적습니다. 그래서 Cargo는 어느 프로필에서든 `build-override`의 기본값으로 `opt-level = 0`을 쓰고, 런타임 의존성으로도 쓰이지 않는 빌드 의존성에는 가능하면 디버그 정보도 만들지 않습니다. 3절의 `-v` 표에서 `release` 빌드인데도 `syn`에 `-C opt-level`이 붙지 않은 것이 그 결과입니다. 빌드 스크립트가 실패해 원인을 찾아야 하면 `[profile.dev.build-override]`에 `debug = true`를 줍니다.

**재정의는 제네릭 함수에 닿지 않을 수 있습니다.** 제네릭 함수는 구체 타입이 정해지는 곳, 즉 부르는 크레이트에서 단형성화(monomorphization)되어 기계어가 만들어집니다([Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md) 2.4절). 그러면 그 기계어에는 함수를 정의한 크레이트가 아니라 부르는 크레이트의 `opt-level`이 적용됩니다. `projects/collatz`의 `chain` 크레이트에는 같은 계산을 하는 두 함수가 있습니다.

```rust
pub fn longest(limit: u64) -> (u64, u32) { … }                        // 제네릭이 아님
pub fn longest_by<F: Fn(u64) -> u64>(limit: u64, next: F) -> (u64, u32) { … } // 제네릭
```

`dev-hot` 프로필로 `chain`만 최적화하면 `longest`는 `release`에 가깝게 빨라지지만, `longest_by`는 `collatz`(부르는 쪽, `opt-level = 0`) 안에서 만들어지므로 조금 빨라지는 데 그칩니다(3절의 측정). Cargo Book은 개발 중에 의존성을 최적화해 볼 때 `opt-level = 1`도 시도해 보라고 권합니다. `rustc`는 크레이트 사이에서 단형성화 결과를 나눠 쓰는 최적화를 하는데, `opt-level`이 `2`나 `3`이면 이 공유를 하지 않기 때문입니다.

### 2.5 빌드 캐시: target 디렉터리와 build 디렉터리

[ch1](../ch1-getting-started/README.md) 2.3절에서 `target/debug/`의 목록을 보고, 실행 파일과 `.rlib`은 `build/<패키지>/<해시>/out/`에 있는 결과의 사본이라는 것까지 봤습니다. Cargo Book 3.9절은 빌드 결과를 두 갈래로 나눕니다.

| 갈래 | 무엇 | 놓이는 곳 |
| --- | --- | --- |
| 최종 산출물(final artifacts) | 사용자가 쓰라고 만든 것. 바이너리 크레이트의 실행 파일, `cargo doc`의 결과, `--timings` 보고서 | target 디렉터리 |
| 중간 산출물(intermediate artifacts) | Cargo와 `rustc`가 내부에서 쓰는 것. 크레이트별 컴파일 결과, 빌드 스크립트의 실행 결과, 증분 컴파일 캐시 | build 디렉터리 |

기본으로는 두 디렉터리가 모두 워크스페이스 루트의 `target/`이라서 한곳에 섞여 보입니다. 위치를 바꾸는 방법은 다음과 같습니다(설정 파일의 키는 Part 3 `ch7-configuration-env`에서 다룹니다).

- target 디렉터리: 환경 변수 `CARGO_TARGET_DIR`, 명령줄 옵션 `--target-dir`, 설정 `build.target-dir`
- build 디렉터리: 환경 변수 `CARGO_BUILD_BUILD_DIR`, 설정 `build.build-dir`

build 디렉터리를 따로 두고 빌드하면 `target/`에는 최종 산출물만 남습니다. 저장소 밖에 복사한 `projects/collatz`에서 build 디렉터리를 그 옆의 `../bd`로 옮겨 빌드한 결과입니다(점으로 시작하는 잠금 파일은 뺐습니다). 저장소 안에서 따라 하면 챕터 디렉터리에 `bd/`가 생기고 `cargo clean`으로도 지워지지 않으므로 사본에서 합니다.

```console
$ CARGO_BUILD_BUILD_DIR=../bd cargo build -q
$ find target -not -name '.*' | sort
target
target/CACHEDIR.TAG
target/debug
target/debug/collatz
target/debug/collatz.d
target/debug/examples
$ ls ../bd/debug
build        incremental
```

**target 디렉터리의 구조.** 프로필마다 디렉터리가 하나씩 있고(2.1절의 표), 그 안에 패키지의 실행 파일과 라이브러리, `examples/`가 있습니다. `cargo doc`의 결과는 `target/doc/`, `cargo package`의 결과는 `target/package/`처럼 명령에 따라 맨 위에 디렉터리가 따로 생깁니다. `--target`으로 다른 플랫폼용으로 빌드하면 `target/<타깃 이름>/debug/`처럼 한 단계가 더 생깁니다(크로스 컴파일은 Part 5에서 다룹니다).

**build 디렉터리의 구조.** `<프로필 디렉터리>/build/<패키지>/<해시>/` 아래에 컴파일 결과(`out/`)와 다시 컴파일할지 판단할 기록(`fingerprint/`)이 있고, 빌드 스크립트를 실행한 결과는 `run/`에 있습니다. `incremental/`은 증분 컴파일 캐시입니다. Cargo Book은 이 구조를 Cargo 내부의 것으로 보고 바뀔 수 있다고 적습니다. 실제로 1.100.0에서 바뀌었으므로, 예전 버전을 설명한 글에 나오는 `target/debug/deps/`는 지금은 없습니다. 테스트 실행 파일은 target 디렉터리로 꺼내지 않고 build 디렉터리에서 바로 실행합니다. `cargo test`가 출력하는 `Running unittests src/lib.rs (target/debug/build/chain/<해시>/out/chain-<해시>)` 줄에서 볼 수 있습니다.

**해시는 설정을 반영합니다.** 같은 패키지라도 프로필 설정이나 켠 기능이 바뀌면 해시가 달라져 새 디렉터리에 따로 컴파일되고, 예전 결과는 지워지지 않습니다. 저장소 밖 사본에서 `cargo build` 뒤에 `[profile.dev.package.chain] opt-level = 1`을 더해 빌드하면 `chain`과 그것에 의존하는 `collatz`가 새 해시로 다시 컴파일되고, `chain`의 디렉터리가 둘이 됩니다. 그 줄을 다시 지우고 빌드하면 예전 결과가 남아 있으므로 아무것도 컴파일하지 않고 끝납니다. 해시 값은 환경마다 다릅니다.

```console
$ ls target/debug/build/chain
bea4484dba987800  ebd7b677151f8316
```

**`.d` 파일.** 산출물 옆의 `.d` 파일(dep-info)은 그 산출물이 어느 소스 파일들로 만들어졌는지 Makefile 문법으로 적은 목록입니다. Make 같은 바깥 빌드 시스템이 "Cargo를 다시 불러야 하는가"를 판단하라고 두는 파일입니다. 경로는 기본으로 절대 경로라서 아래는 앞부분을 줄였습니다.

```text
…/collatz/target/debug/collatz: …/collatz/chain/src/lib.rs …/collatz/src/main.rs
```

**다시 컴파일한 이유 보기.** [ch4](../ch4-workspaces/README.md) 2.5절에서 멤버 하나를 고치면 무엇이 다시 컴파일되는지 봤습니다. `cargo build -v`는 그 이유를 `Dirty` 줄에 적습니다. `chain/src/lib.rs`를 `touch`한 뒤의 출력입니다(경로와 시각은 줄였습니다).

```console
$ touch chain/src/lib.rs
$ cargo build -v
       Fresh unicode-ident v1.0.27
       Dirty chain v0.1.0 (…/collatz/chain): the file `chain/src/lib.rs` has changed (…)
   Compiling chain v0.1.0 (…/collatz/chain)
       Fresh proc-macro2 v1.0.107
       Fresh quote v1.0.47
       Fresh syn v3.0.10
       Fresh thiserror-impl v2.0.21
       Fresh thiserror v2.0.21
       Dirty collatz v0.1.0 (…/collatz): the dependency `chain` was rebuilt
   Compiling collatz v0.1.0 (…/collatz)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
```

**워크스페이스 사이에서 캐시 나누기.** build 디렉터리는 워크스페이스마다 따로 있어서, 같은 버전의 `syn`을 쓰는 프로젝트가 열 개면 열 번 컴파일합니다. Cargo Book은 서드파티 도구 `sccache`를 소개합니다. `cargo install --locked sccache`로 설치하고 환경 변수 `RUSTC_WRAPPER=sccache`를 두면, Cargo가 `rustc` 대신 `sccache`를 부르고 `sccache`가 같은 입력의 컴파일 결과를 여러 워크스페이스에 나눠 줍니다. 이 저장소에서는 설치하지 않아 실행해 보지 못했습니다.

### 2.6 빌드 시간 재기: `--timings`

빌드가 느리다면 먼저 어디서 시간을 쓰는지 봐야 합니다. `--timings`를 붙이면 Cargo가 컴파일마다 걸린 시간과 동시에 실행된 정도를 기록해 HTML 보고서로 남깁니다.

```console
$ cargo build --timings
   Compiling proc-macro2 v1.0.107
   …
   Compiling collatz v0.1.0 (…/collatz)
      Timing report saved to …/collatz/target/cargo-timings/cargo-timing-20261120T234111294Z-4adde8bb70d107be.html
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.56s
```

같은 디렉터리의 `cargo-timing.html`이 늘 가장 최근 보고서이고, 시각이 붙은 파일은 지난 실행을 비교하라고 남는 사본입니다. 보고서는 최종 산출물이므로 target 디렉터리에 놓입니다. 브라우저로 열면 표 둘과 그래프 둘이 있습니다.

- **첫 번째 표(요약)**: 빌드한 단위(unit) 수, 최대 동시 실행 수, 전체 시간, 컴파일러 버전. 단위는 `rustc` 호출 하나, 또는 빌드 스크립트 실행 하나입니다.
- **단위 그래프**: 단위마다 시작과 끝을 가로 막대로 그립니다. 한 단위가 끝나서 실행할 수 있게 된(unblocked) 단위들이 선으로 이어지므로, 마우스를 올려 가장 긴 의존 경로(critical path)를 찾을 수 있습니다. 기계어를 만드는 codegen 구간은 연보라색, 빌드 스크립트 실행은 주황색입니다.
- **동시 실행 그래프**: CPU 자리를 기다리는 단위(Waiting, 빨강), 의존성이 끝나기를 기다리는 단위(Inactive, 파랑), 실행 중인 단위(Active, 초록)의 수를 시간에 따라 그립니다. `rustc` 안에서의 병렬 처리는 나타내지 않습니다.
- **두 번째 표(단위별)**: 단위마다 전체 시간, 프런트엔드(frontend, 소스를 읽고 검사하는 앞부분)와 codegen 시간, 켜진 기능. 2.7절의 병렬 프런트엔드가 빠르게 하려는 것이 이 프런트엔드 구간입니다.

`projects/collatz`의 첫 빌드에서 표 두 개를 글자로 옮긴 것입니다. 시간은 실행마다 다릅니다.

```text
Profile: dev  Fresh units: 0  Dirty units: 14  Total units: 14
Max concurrency: 5 (jobs=18 ncpu=18)  Total time: 1.6s

 1. syn v3.0.10                               0.5s  frontend 0.4s (84%)  codegen 0.1s (16%)
 2. thiserror-impl v2.0.21                    0.3s
 3. proc-macro2 v1.0.107 build-script (run)   0.2s
 4. proc-macro2 v1.0.107                      0.1s
 …
10. collatz v0.1.0 collatz "bin"              0.1s
…
13. chain v0.1.0                              0.0s
```

`Cargo.toml`에 적은 의존성은 둘(`chain`, `thiserror`)이지만 전이 의존성까지 여덟 패키지를 빌드하고, 단위는 14개입니다. 빌드 스크립트가 있는 패키지(`proc-macro2`, `quote`, `thiserror`)는 "빌드 스크립트 컴파일"과 "빌드 스크립트 실행"이 따로 단위가 되기 때문입니다. 우리가 쓴 두 패키지(`chain`, `collatz`)보다 `thiserror`의 `#[derive(Error)]`를 위한 절차적 매크로 쪽(`syn`, `thiserror-impl`, `proc-macro2`)이 시간을 거의 다 씁니다. Cargo Book이 보고서를 읽고 해 볼 일로 드는 것은 다음과 같습니다.

- 느린 의존성을 찾습니다. 끌 수 있는 기능이 있는지([ch5](../ch5-features/README.md)), 의존성을 아예 뺄 수 있는지 봅니다.
- 같은 크레이트가 여러 버전으로 빌드되는지 봅니다. 오래된 버전을 그래프에서 없앱니다.
- 큰 크레이트는 작은 조각으로 나눕니다.
- 많은 크레이트가 한 크레이트를 기다리고 있으면 그 크레이트를 먼저 개선해 병렬성을 높입니다.

### 2.7 빌드 시간 줄이기

Cargo Book 2.11절은 빌드 성능을 다른 것(디버깅 편의, 실행 속도, 크기)보다 앞세우는 방법들을 모아 둡니다. 먼저 강조하는 것은 **자기 작업 방식으로 재 보라**는 것입니다. 고친 뒤의 `cargo check`, 고친 뒤의 `cargo test`, CI 빌드처럼 실제로 기다리는 일을 재야 하고, 일반적인 권장이 어떤 프로젝트에서는 오히려 느리게 만들 수도 있습니다.

| 방법 | 설정 | 얻는 것 | 잃는 것 | 이 챕터에서 |
| --- | --- | --- | --- | --- |
| 디버그 정보 줄이기 | 아래 코드 블록 | 기계어 생성과 링크가 빨라지고 `target/`이 작아짐 | 디버거를 제대로 쓰려면 다른 프로필로 전부 다시 빌드 | `collatz`의 `lean` 프로필 |
| 다른 코드 생성 백엔드(Cranelift) | `codegen-backend = "cranelift"` | 기계어 생성이 빨라짐 | nightly 필요, 만든 코드가 느림, 일부 타깃·기능만 지원 | 설명만 |
| 병렬 프런트엔드 | `rustflags = "-Zthreads=8"` | `check`와 `build`가 빨라짐 | nightly 필요 | 설명만 |
| 다른 링커(LLD, mold, wild) | `linker`, `-C link-arg=…` | 링크가 빨라짐 | C·C++ 의존성 등 일부 경우를 지원하지 않을 수 있음 | 설명만 |
| 워크스페이스 전체로 기능 고르기 | `[resolver] feature-unification = "workspace"` | 멤버를 바꿔 가며 빌드할 때 의존성을 덜 다시 빌드 | nightly 필요, 기능 통합이 버그를 가릴 수 있음 | [ch5](../ch5-features/README.md) 2.5절의 기능 통합 |
| 쓰지 않는 의존성 빼기 | Cargo 린트 `cargo::unused_dependencies` | 전체 빌드와 링크가 빨라짐 | 오탐을 가려내는 수고 | [ch1](../ch1-getting-started/README.md)·[ch3](../ch3-manifest-targets/README.md)에서 본 경고 |
| 쓰지 않는 의존성 기능 끄기 | 서드파티 도구(cargo-features-manager, cargo-unused-features) | 빌드할 크레이트와 코드가 줄어듦 | 동작이나 성능에 쓰이던 기능을 잘못 끌 수 있음 | 설명만 |

이 저장소에는 nightly 툴체인과 위의 링커·서드파티 도구가 설치돼 있지 않아 "설명만"인 줄은 실행해 보지 못했습니다. 디버그 정보에 관한 권장은 다음과 같습니다.

```toml
[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = false

[profile.debugging]
inherits = "dev"
debug = true
```

멤버에는 백트레이스에 필요한 줄 정보만 남기고, 의존성에는 디버그 정보를 만들지 않고, 디버거가 필요할 때 쓸 `debugging` 프로필을 따로 둡니다. `collatz`의 `lean` 프로필은 이 가운데 앞의 두 표를 `dev`를 물려받은 별도 프로필로 옮긴 것입니다. 이 환경에서 `dev`와 `lean`으로 처음부터 빌드한 디렉터리 크기는 `du -sk` 기준 16476 KB와 16208 KB로 거의 같았습니다. 의존성의 대부분을 차지하는 빌드 의존성은 원래 디버그 정보를 만들지 않고(2.4절), 우리 코드는 작기 때문입니다. 디버그 정보를 많이 만드는 큰 프로젝트일수록 차이가 커집니다. 권장을 그대로 따르기 전에 재 봐야 하는 이유입니다.

이런 설정을 어디에 둘지도 고릅니다. `Cargo.toml`에 두면 프로젝트의 모든 개발자가 같은 설정을 쓰지만 적을 수 있는 설정이 제한되고, 프로젝트의 `.cargo/config.toml`은 모든 개발자가 쓰되 `cargo`를 실행한 디렉터리에 따라 달라지고, Cargo home의 설정 파일은 개발자 한 사람의 기본값이 됩니다(Part 3 `ch7-configuration-env`).

## 3. 예제 안내

예제는 저장소 루트에서 실행합니다. 챕터 패키지는 루트 워크스페이스의 멤버이므로 `dev`와 `release`의 기본 설정만 씁니다.

### 01_profile_checks — 프로필에 따라 달라지는 동작

[`examples/01_profile_checks.rs`](examples/01_profile_checks.rs)는 자기 실행 파일이 놓인 디렉터리, `cfg!(debug_assertions)`의 값, `debug_assert!` 안의 검사 함수가 불린 횟수, `250u8 + 10`의 결과를 출력합니다. 오버플로 패닉은 `catch_unwind`로 붙잡아 결과만 출력합니다([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 2.9절).

```console
$ cargo run -q -p p3-ch6-profiles-build-performance --example 01_profile_checks
실행 파일: …/debug/examples/01_profile_checks
cfg!(debug_assertions): true
debug_assert! 안의 검사 함수 호출: 1번
250u8 + 10: 패닉 (오버플로 검사)
$ cargo run -q -p p3-ch6-profiles-build-performance --example 01_profile_checks --release
실행 파일: …/release/examples/01_profile_checks
cfg!(debug_assertions): false
debug_assert! 안의 검사 함수 호출: 0번
250u8 + 10: 4 (검사 없이 래핑)
```

소스는 한 글자도 다르지 않습니다. `release`에서는 `debug_assert!`의 조건식이 아예 평가되지 않아 검사 함수가 한 번도 불리지 않았습니다(예제는 이것을 보이려고 일부러 검사 함수 안에서 호출 횟수를 셉니다). 그래서 `debug_assert!`에는 비싸지만 결과에 영향을 주지 않는 검사만 넣어야 합니다. 부수 효과가 있는 코드를 넣으면 프로필에 따라 프로그램의 동작이 달라집니다.

### 02_debuginfo_backtrace — 디버그 정보와 백트레이스

[`examples/02_debuginfo_backtrace.rs`](examples/02_debuginfo_backtrace.rs)는 `Backtrace::force_capture()`([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 2.10절)로 백트레이스를 문자열로 받아, 그것을 기록한 함수 `inner`의 프레임에서 함수 이름과 위치(파일:줄:열)를 찾아 출력합니다. 백트레이스 전체는 툴체인과 경로에 따라 달라지므로 출력하지 않습니다. `inner`와 `outer`에는 `#[inline(never)]` 속성을 붙였습니다. 컴파일러가 함수 본문을 부르는 자리에 펼쳐 넣는 최적화(인라인, [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 2.6절)를 하지 못하게 하는 속성입니다. 이것이 없으면 최적화된 `release` 빌드에서 `inner`가 `main` 안에 펼쳐져 자기 프레임이 사라지고, 아래의 `release` 결과 대신 `inner 프레임: 이름으로 찾지 못함`이 나옵니다. [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)의 `08_backtrace`를 `--release`로 실행했을 때 프레임이 줄어든 것과 같은 이유입니다.

```console
$ cargo run -q -p p3-ch6-profiles-build-performance --example 02_debuginfo_backtrace
inner 프레임의 이름: 02_debuginfo_backtrace::inner
inner 프레임의 위치: 02_debuginfo_backtrace.rs:15:5
$ cargo run -q -p p3-ch6-profiles-build-performance --example 02_debuginfo_backtrace --release
inner 프레임의 이름: 02_debuginfo_backtrace::inner
inner 프레임의 위치: (정보 없음)
```

`release`는 `debug = false`라서 위치가 없습니다. 함수 이름은 디버그 정보가 아니라 실행 파일의 심볼 표에서 온 것입니다. 환경 변수 `CARGO_PROFILE_<프로필>_<설정>`(대문자, `-`는 `_`)으로 `release`의 설정 하나만 바꿔 실행하면 차이가 더 드러납니다.

```console
$ CARGO_PROFILE_RELEASE_DEBUG=line-tables-only cargo run -q -p p3-ch6-profiles-build-performance --example 02_debuginfo_backtrace --release
inner 프레임의 이름: inner
inner 프레임의 위치: 02_debuginfo_backtrace.rs:15:5
$ CARGO_PROFILE_RELEASE_STRIP=symbols cargo run -q -p p3-ch6-profiles-build-performance --example 02_debuginfo_backtrace --release
inner 프레임: 이름으로 찾지 못함
```

| 설정 | 이름 | 위치 |
| --- | --- | --- |
| `dev` (`debug = true`) | 경로까지 붙은 이름 | 있음 |
| `release` (`debug = false`) | 경로까지 붙은 이름(심볼 표) | 없음 |
| `release` + `debug = "line-tables-only"` | 경로가 빠진 짧은 이름 | 있음 |
| `release` + `strip = "symbols"` | 없음 | 없음 |

디버그 정보가 있으면 이름도 디버그 정보에서 읽는 것으로 보입니다. 같은 소스를 `rustc -C opt-level=3 -C debuginfo=1`로 빌드하면 경로가 붙은 이름이, `-C debuginfo=line-tables-only`로 빌드하면 경로가 빠진 `inner`가 나왔습니다. 줄 정보만 남기는 이 값에는 모듈 경로가 들어 있지 않습니다. `strip = "symbols"`로 빌드한 실행 파일에서는 macOS가 모든 프레임의 이름을 `__mh_execute_header`로 보여 주므로, 예제는 `inner`를 찾지 못합니다.

`"line-tables-only"`는 최적화된 빌드에서도 패닉 위치를 알려 줄 만큼의 정보를 남깁니다. 2.7절의 권장이 `dev`의 디버그 정보를 완전히 끄지 않고 이 값으로 줄이는 이유입니다. 반대로 `strip = "symbols"`는 크기를 줄이는 대신 백트레이스를 쓸모없게 만듭니다. 이 출력은 macOS에서 확인한 것입니다. 백트레이스가 이름을 찾는 방법은 운영체제마다 다르므로 다른 운영체제에서는 결과가 다를 수 있습니다.

환경 변수로 프로필 설정을 바꾸면 그 설정이 해시에 들어가므로 다시 컴파일됩니다(2.5절). 여기서 쓴 환경 변수의 규칙은 Part 3 `ch7-configuration-env`에서 다룹니다.

### projects/collatz — 프로필만 바꿔 크기와 속도 비교하기

`1..=N`(기본 1,000,000)에서 콜라츠 수열이 가장 긴 시작 값을 구하는 프로그램입니다. 같은 계산을 제네릭이 아닌 함수 `chain::longest`와 제네릭 함수 `chain::longest_by`로 한 번씩 하고, 각각 걸린 시간을 출력합니다. 입력 `N`을 실행할 때 받으므로 컴파일러가 결과를 미리 계산해 둘 수 없습니다.

| 파일 | 내용 |
| --- | --- |
| [`Cargo.toml`](projects/collatz/Cargo.toml) | 루트 패키지 `collatz`, 빈 `[workspace]`, 사용자 정의 프로필 `dev-hot`·`small`·`lean` |
| [`src/main.rs`](projects/collatz/src/main.rs) | 인자 파싱(`thiserror`로 만든 에러 타입), 두 함수의 실행과 시간 출력 |
| [`chain/src/lib.rs`](projects/collatz/chain/src/lib.rs) | path 의존성이자 워크스페이스 멤버인 `chain`. 계산과 단위 테스트 |

프로젝트 디렉터리에서 실행합니다. 시간(ms) 값은 실행할 때마다 달라집니다.

| 명령 | 기대 결과 |
| --- | --- |
| `cargo test --locked --workspace` | `chain`의 테스트 2개 통과 |
| `cargo run --locked --release` | 아래 출력(시간 값만 다름) |
| `cargo run --locked --release -- 27` | `27 (111단계)` |
| `cargo run --locked -- 0` | 표준 에러에 `에러: 범위의 끝은 1 이상의 정수여야 합니다: "0"`, 종료 상태 2 |
| `cargo build --locked --profile <프로필>` 뒤 `./target/<디렉터리>/collatz` | 아래 두 표 |
| `cargo build --locked -v --profile <프로필>` | `Running` 줄의 `-C` 옵션이 아래 `-v` 표와 같음 |
| `cargo build --locked --timings` | 2.6절의 보고서 |
| `cargo clean` | `target/` 삭제(실습을 마친 뒤) |

```console
$ cargo run --locked --release
…
범위: 1..=1000000
일반 함수:   837799 (524단계), 95 ms
제네릭 함수: 837799 (524단계), 92 ms
```

**실행 파일 크기.** 다섯 프로필로 빌드한 실행 파일을 `wc -c`로 잰 값입니다.

| 프로필 (디렉터리) | 크기(바이트) |
| --- | --- |
| `dev` (`debug/`) | 535,880 |
| `dev-hot` (`dev-hot/`) | 534,712 |
| `release` (`release/`) | 456,384 |
| `small` (`small/`) | 302,688 |
| `lean` (`lean/`) | 536,040 |

`small`이 `release`의 3분의 2쯤입니다. `dev` 계열이 `release`보다 크게 크지 않은 것은 macOS에서 디버그 정보가 실행 파일 밖에 있기 때문입니다(2.2절의 `split-debuginfo`). `small`의 다섯 설정 가운데 어느 것이 크기를 줄였는지는 연습 1에서 직접 찾습니다.

**실행 시간.** 각 실행 파일을 세 번씩 실행한 범위입니다.

| 프로필 | 일반 함수(`longest`) | 제네릭 함수(`longest_by`) |
| --- | --- | --- |
| `dev` | 710~743 ms | 894~895 ms |
| `dev-hot` | 135~155 ms | 698~718 ms |
| `release` | 92~95 ms | 92 ms |
| `small` | 92~116 ms | 92 ms |
| `lean` | 708~728 ms | 918~978 ms |

- `dev`와 `release`는 같은 소스인데 7~10배 차이가 났습니다. 성능은 릴리스 빌드로 재야 하는 이유입니다([부록 A ch20](../../appd-a/ch20-avoid-over-optimizing/README.md) 2.5절).
- `dev-hot`은 `chain`만 `opt-level = 3`으로 만듭니다. 일반 함수는 약 5배 빨라져 `release`에 가까워졌지만, 제네릭 함수는 20%쯤 빨라지는 데 그쳤습니다. `longest_by`의 기계어는 클로저를 넘긴 `collatz` 크레이트에서 `opt-level = 0`으로 만들어지기 때문입니다(2.4절).
- `small`은 `opt-level = "z"`인데도 이 계산에서는 `release`와 비슷했습니다. `lean`은 디버그 정보만 줄였으므로 `dev`와 비슷합니다.

**`-v`로 본 `rustc` 옵션.** `cargo build -v --profile <프로필>`의 `Running` 줄에서 프로필에 관한 `-C` 옵션만 골라 적었습니다(앞의 `-C `는 뺐습니다). 빈칸은 해당 옵션이 하나도 붙지 않았다는 뜻입니다.

| 크레이트 | `dev` | `release` | `dev-hot` | `lean` |
| --- | --- | --- | --- | --- |
| `collatz` (멤버, 바이너리) | `debuginfo=2` | `opt-level=3` `strip=debuginfo` | `debuginfo=2` | `debuginfo=line-tables-only` |
| `chain` (멤버, 라이브러리) | `debuginfo=2` | `opt-level=3` `strip=debuginfo` | `opt-level=3` `debuginfo=2` `debug-assertions=on` | `debuginfo=line-tables-only` |
| `thiserror` (런타임 의존성) | `debuginfo=2` | `opt-level=3` `strip=debuginfo` | `debuginfo=2` | `strip=debuginfo` |
| `syn`, `thiserror-impl`, 빌드 스크립트들 (빌드 의존성) | | `debug-assertions=off` `strip=debuginfo` | | `strip=debuginfo` |

읽는 법은 이렇습니다.

- **Cargo는 `rustc`의 기본값과 같은 설정은 넘기지 않습니다.** `rustc`의 기본 `opt-level`은 0이고 디버그 정보는 없음이라서, `dev`의 `opt-level = 0`은 옵션으로 나타나지 않습니다.
- **`release`의 빌드 의존성에는 `opt-level`이 없습니다.** `build-override`의 기본값 `opt-level = 0` 때문입니다(2.4절). 그 대신 `debug-assertions=off`가 명시돼 있습니다. `rustc`는 `opt-level`이 0이면 디버그 단언을 기본으로 켜는데, `release`의 값은 "끔"이기 때문입니다.
- **`dev-hot`의 `chain`에는 `debug-assertions=on`이 붙었습니다.** 반대의 경우입니다. `rustc`는 `opt-level`이 0이 아니면 디버그 단언을 기본으로 끄므로, `dev`에서 물려받은 "켬"을 명시해 넘깁니다. `rustc`의 오버플로 검사는 따로 정하지 않으면 디버그 단언을 따르므로 함께 켜져 있습니다.
- **`lean`의 `"*"` 재정의는 멤버가 아닌 크레이트에만 걸렸습니다.** `thiserror`와 빌드 의존성은 `debug = false`가 되어 `strip=debuginfo`가 붙었고, 멤버인 `collatz`와 `chain`은 프로필 자체의 `line-tables-only`를 썼습니다. 빌드 의존성에도 `"*"`가 걸린 것은 우선순위 표에서 `"*"`가 `build-override`보다 앞서기 때문입니다.
- `small`에서는 `collatz`에 `opt-level=z panic=abort lto codegen-units=1 strip=symbols`가, 런타임 라이브러리(`chain`, `thiserror`)에는 `lto` 대신 `linker-plugin-lto`가 붙었습니다. 런타임 라이브러리는 링크할 때 LTO가 다시 최적화할 수 있는 형태로 만들어 두고, 실제 LTO는 마지막 실행 파일을 링크할 때 합니다. 빌드 의존성(`syn`, `thiserror-impl`, 빌드 스크립트들)에는 `opt-level`도 LTO 관련 옵션도 붙지 않고, `release`와 같은 `debug-assertions=off`에 `strip=symbols`만 붙었습니다. 프로필 자체의 `strip`은 빌드 의존성에도 걸린다는 것을 알 수 있습니다.
- 그 밖에 `-C embed-bitcode=no`, `-C split-debuginfo=unpacked`, `-C incremental=…`도 붙지만 표에서는 뺐습니다.

## 4. 자주 하는 실수

`projects/collatz`를 저장소 밖에 복사한 사본의 `Cargo.toml`에 각 항목을 더해 `cargo build`로 확인했습니다. 종료 상태는 `--release`와 `--profile`을 함께 준 경우만 1이고 나머지 에러는 101입니다.

| 상황 | 메시지 | 해결 |
| --- | --- | --- |
| 워크스페이스 멤버의 `Cargo.toml`에 `[profile]`을 적음 | `warning: profiles for the non root package will be ignored, specify profiles at the workspace root:` | 워크스페이스 루트의 매니페스트로 옮깁니다([ch4](../ch4-workspaces/README.md) 2.7절). |
| 사용자 정의 프로필에 `inherits`가 없음 | ``error: profile `fast` is missing an `inherits` directive (`inherits` is required for all profiles except `dev` or `release`)`` | `inherits = "release"`처럼 물려받을 프로필을 적습니다. |
| 프로필이 자기 자신을 물려받음 | ``error: profile inheritance loop detected with profile `debug2` inheriting `debug2` `` | 고리가 생기지 않게 `inherits`를 고칩니다. |
| 패키지 재정의에 `panic`이나 `lto`를 적음 | ``failed to parse manifest`` / ``Caused by: `panic` may not be specified in a `package` profile`` | 이 둘과 `rpath`는 프로필 자체에만 적습니다. |
| `--release`와 `--profile`을 함께 줌 | ``error: the argument '--release' cannot be used with '--profile <PROFILE-NAME>'`` | 둘 중 하나만 씁니다. `--release`는 `--profile release`와 같습니다. |
| 정의하지 않은 프로필 이름 | ``error: profile `nope` is not defined`` | 루트 `Cargo.toml`에 `[profile.nope]`이 있는지, 멤버에 적지 않았는지 확인합니다. |
| `cargo run`(dev)으로 성능을 잼 | (메시지 없음) | 3절에서 `release`보다 7~10배 느렸습니다. 성능은 `--release`나 `release`를 물려받은 프로필로 잽니다. |
| 의존성 재정의로 제네릭 코드를 빠르게 하려 함 | (메시지 없음) | 제네릭 함수는 부르는 크레이트에서 만들어집니다. 부르는 패키지도 재정의하거나 프로필 자체의 `opt-level`을 올립니다(연습 2). |
| `opt-level = "z"`면 늘 작다고 여김 | (메시지 없음) | 이 프로젝트에서는 `"z"`만으로는 작아지지 않았습니다. 설정마다 재 봅니다(연습 1). |

## 5. 연습 문제

연습마다 `projects/collatz`를 저장소 밖에 새로 복사해 시작합니다. 앞 연습의 변경이 뒤 연습에 영향을 주지 않게 하기 위해서입니다.

### 연습 1. 무엇이 크기를 줄였나

`small` 프로필은 `release`에서 다섯 설정(`opt-level`, `lto`, `codegen-units`, `panic`, `strip`)을 바꿉니다. 설정을 하나씩만 바꾼 프로필 다섯 개를 만들어 실행 파일 크기를 재고, 크기를 가장 많이 줄인 설정 두 개를 찾으세요.

<details>
<summary>힌트</summary>

`[profile.t-strip] inherits = "release"`에 `strip = true` 한 줄만 두는 식으로 프로필을 만들고, `cargo build --profile t-strip` 뒤 `wc -c < target/t-strip/collatz`로 잽니다. 이 환경에서는 `strip = true`(약 360 KB)와 `lto = true`(약 379 KB)가 크게 줄였고, `opt-level = "z"`·`codegen-units = 1`·`panic = "abort"`는 각각 1 KB도 줄이지 않았습니다(`release`는 약 456 KB). 실행 파일의 많은 부분은 미리 컴파일돼 있는 표준 라이브러리입니다. 심볼 표를 지우거나 LTO로 쓰지 않는 표준 라이브러리 코드를 걷어 내야 크게 줄어듭니다.

</details>

### 연습 2. 제네릭 판도 빠르게

소스는 고치지 않고 `Cargo.toml`만 고쳐서, `dev-hot` 프로필에서 제네릭 함수도 `release`에 가까운 시간이 나오게 하세요. 다른 패키지(`thiserror` 등)의 설정은 바꾸지 마세요.

<details>
<summary>힌트</summary>

`longest_by`의 기계어는 `collatz`에서 만들어집니다. `[profile.dev-hot.package.collatz]`에 `opt-level = 3`을 더합니다. 패키지 이름을 적은 재정의는 멤버에도 적용됩니다. 이 환경에서는 일반 함수 151 ms, 제네릭 함수 136 ms가 나왔습니다. 소스를 고쳐도 된다면, `chain` 안에 클로저를 넘겨 `longest_by`를 부르는 제네릭이 아닌 함수를 두는 방법도 있습니다. 그러면 단형성화가 `chain` 안에서 일어납니다.

</details>

### 연습 3. 다시 컴파일되는 범위 맞히기

`cargo build --profile small`을 한 번 실행한 뒤, `[profile.small]`에서 `strip = true` 줄을 지우고 다시 `cargo build --profile small`을 실행합니다. `Compiling` 줄이 몇 개 나올지 먼저 예상하고 확인하세요. `chain`이나 `collatz`의 소스는 바뀌지 않았는데 왜 그런지 설명하세요.

<details>
<summary>힌트</summary>

`strip`은 프로필 자체의 설정이라 모든 크레이트에 걸립니다(3절 `-v` 표 아래의 `small` 항목에서 빌드 의존성에도 `strip=symbols`가 붙었음). 설정이 바뀌면 크레이트마다 해시가 바뀌므로(2.5절) 의존성까지 여덟 패키지가 모두 다시 컴파일됩니다. `Compiling` 줄은 8개입니다.

</details>

### 연습 4. 권장 설정의 효과 재기

`cargo clean` 뒤 `cargo build --timings`와 `cargo build --timings --profile lean`을 차례로 실행하고, 두 보고서의 `Total time`과 `du -sk target/debug target/lean`의 값을 비교하세요. 2.7절의 디버그 정보 권장이 이 프로젝트에서 얼마나 도움이 되는지, 어떤 프로젝트라면 더 도움이 될지 설명하세요.

<details>
<summary>힌트</summary>

두 번째 보고서는 덮어쓰이기 전에 `cargo-timing.html`을 다른 이름으로 복사해 두거나, 시각이 붙은 사본을 엽니다. 이 환경에서는 시간과 크기 모두 차이가 작았습니다(크기 16476 KB와 16208 KB). 빌드 시간의 대부분을 디버그 정보를 원래 만들지 않는 절차적 매크로 쪽이 쓰고, 멤버 코드가 작기 때문입니다. 디버그 정보를 만드는 런타임 의존성이 많고 멤버 코드가 큰 프로젝트일수록 효과가 커집니다.

</details>

## 6. 참고 자료

- The Cargo Book: [2.11 Optimizing Build Performance](https://doc.rust-lang.org/cargo/guide/build-performance.html), [3.5 Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html), [3.9 Build Cache](https://doc.rust-lang.org/cargo/reference/build-cache.html), [3.15 Reporting build timings](https://doc.rust-lang.org/cargo/reference/timings.html)
- The rustc book: [Codegen Options](https://doc.rust-lang.org/rustc/codegen-options/index.html) (`opt-level`, `debuginfo`, `strip`, `debug-assertions`, `overflow-checks`, `split-debuginfo`의 `rustc` 쪽 설명과 기본값)
- [`std::backtrace::Backtrace`](https://doc.rust-lang.org/std/backtrace/struct.Backtrace.html)
