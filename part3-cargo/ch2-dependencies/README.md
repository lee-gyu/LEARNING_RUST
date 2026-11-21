# ch2. 의존성: 버전, 출처, 해석

[ch1](../ch1-getting-started/README.md)에서 `Cargo.toml`은 의존성을 넓게 적고 `Cargo.lock`은 Cargo가 고른 결과를 정확하게 적는다는 것을 보았습니다. 이 챕터는 그 사이를 채웁니다. 사람이 적는 "넓게"의 문법(버전 요구 사항), 의존성을 가져올 수 있는 곳(crates.io, git 저장소, 로컬 경로), 그리고 Cargo가 요구 사항들을 모아 버전을 하나씩 정하는 과정(의존성 해석)입니다. 마지막으로 의존성을 잠시 다른 사본으로 바꿔 끼우는 `[patch]`를 봅니다.

- 패키지: `p3-ch2-dependencies`
- 기준 문서: The Cargo Book [2.4 Dependencies](https://doc.rust-lang.org/cargo/guide/dependencies.html), [3.3 Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html), [3.3.1 Overriding Dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html), [3.3.3 Dependency Resolution](https://doc.rust-lang.org/cargo/reference/resolver.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64)
- 실습 자료: `examples/` 2개, `compile_fail/` 1개, 독립 프로젝트 [`projects/three-sources`](projects/three-sources/Cargo.toml)(세 출처), [`projects/resolver-lab`](projects/resolver-lab/pick/Cargo.toml)(연습용 레지스트리와 패키지 다섯 개)
- 외부 크레이트: 예제 01이 `semver` 1.0.28을 씁니다(dev-dependency). `three-sources`는 crates.io의 `heck` 0.5.0·`unicode-width` 0.2.3과 GitHub의 `rust-lang/cfg-if` 저장소를 받으므로 처음 빌드할 때 네트워크가 필요합니다. `resolver-lab`은 네트워크를 쓰지 않습니다. 버전은 2026년 11월 기준이고 각 프로젝트의 `Cargo.lock`이 고정합니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 버전 요구 사항(기본·캐럿, 틸드, 와일드카드, 비교, 프리릴리스)을 버전의 범위로 풀어 읽고, 보통은 기본 요구 사항을 쓰는 이유를 설명할 수 있다.
2. crates.io, git 저장소, 로컬 경로의 의존성을 `Cargo.toml`에 적고, `Cargo.lock`의 `source`로 출처를 구별할 수 있다.
3. `cargo add`·`cargo remove`·`cargo update`·`cargo tree`로 의존성을 추가하고, 갱신하고, 어디서 들어왔는지 조사할 수 있다.
4. Cargo가 버전을 고르는 규칙(범위 안의 가장 높은 버전, 호환되면 하나로 합침, 호환되지 않으면 둘 다 빌드, 겹치지 않으면 실패, 잠금 파일 우선)을 설명하고 해석 실패 메시지를 읽을 수 있다.
5. `[patch]`로 의존성을 로컬 사본으로 바꿔 끼우고, 그 효과가 그래프 전체에 미친다는 것을 확인할 수 있다.

## 2. 핵심 개념

### 2.1 의존성 한 줄이 말하는 것

```toml
[dependencies]
heck = "0.5"
```

이 한 줄에는 세 가지가 들어 있습니다.

| 무엇 | 이 줄에서 | 다루는 절 |
| --- | --- | --- |
| 이름 | `heck` | 2.4절(이름 바꾸기) |
| 버전 요구 사항(version requirement) | `"0.5"`: 0.5.0 이상 0.6.0 미만 | 2.2절 |
| 출처 | 적지 않았으므로 crates.io | 2.3절 |

`[dependencies]`에 직접 적은 크레이트를 **직접 의존성**(direct dependency), 그 크레이트가 다시 의존해서 따라 들어온 크레이트를 **전이 의존성**(transitive dependency)이라고 합니다. 직접 의존성과 전이 의존성을 모두 이은 것이 **의존성 그래프**(dependency graph)입니다. `Cargo.toml`에는 직접 의존성만 적고, 그래프 전체는 Cargo가 계산해 `Cargo.lock`에 적습니다.

### 2.2 버전 요구 사항 문법

`"0.5"`는 버전 하나가 아니라 범위입니다. 요구 사항의 종류마다 범위가 어떻게 달라지는지를 예제 [`01_version_requirements`](examples/01_version_requirements.rs)가 같은 버전 목록에 대 보여 줍니다. 판정에는 Cargo 방식의 버전을 해석하는 `semver` 크레이트를 썼습니다.

```text
게시된 버전: 0.0.3 0.2.3 0.2.9 0.3.0 1.2.3 1.2.9 1.3.0 1.9.0 2.0.0 2.1.0-beta.1

== 기본(캐럿): 호환되는 범위 ==
"1.2.3"          1.2.3 1.2.9 1.3.0 1.9.0
"^1.2.3"         1.2.3 1.2.9 1.3.0 1.9.0
"1.2"            1.2.3 1.2.9 1.3.0 1.9.0
"1"              1.2.3 1.2.9 1.3.0 1.9.0

== 0으로 시작하는 버전의 캐럿 ==
"0.2.3"          0.2.3 0.2.9
"0.2"            0.2.3 0.2.9
"0.0.3"          0.0.3
"0"              0.0.3 0.2.3 0.2.9 0.3.0
```

**기본 요구 사항은 "이 버전 이상, 호환되는 범위 안"입니다.** 연산자 없이 버전만 적은 것이 기본 요구 사항(default requirement)이고, 앞에 `^`를 붙인 캐럿 요구 사항(caret requirement)과 뜻이 같습니다. Cargo는 두 버전의 **0이 아닌 가장 왼쪽 자리가 같으면 호환된다**고 봅니다.

| 버전의 모양 | 호환을 가르는 자리 | 예 |
| --- | --- | --- |
| `1.2.3` (1 이상) | 첫째 자리 | `"1.2.3"`은 1.2.3 이상 2.0.0 미만 |
| `0.2.3` | 둘째 자리 | `"0.2.3"`은 0.2.3 이상 0.3.0 미만 |
| `0.0.3` | 셋째 자리 | `"0.0.3"`은 0.0.3 하나뿐 |

적은 자리가 모두 0이면 가를 자리가 정해지지 않아 범위가 한 자리 더 열립니다. 위 출력의 `"0"`이 서로 호환되지 않는 0.2.x와 0.3.0을 함께 받아들이는 것이 그래서입니다(`"0"`은 1.0.0 미만 전부, `"0.0"`은 0.1.0 미만 전부).

이 규칙은 크레이트 작성자들이 지키는 약속인 시맨틱 버저닝(SemVer)에 기대고 있습니다. 호환되지 않는 변경을 하면 그 자리를 올린다는 약속입니다. 그래서 `"1.2.3"`이라고 적으면 "1.2.3에 있던 기능이 필요하고, 그 뒤에 나온 1.x는 그대로 쓸 수 있을 것"이라는 뜻이 됩니다. 0.x 버전에서 둘째 자리를 그 자리로 쓰는 것은 Cargo의 방식입니다. 무엇이 호환되지 않는 변경인지는 [ch10](../ch10-publishing-semver/README.md)에서 다루고, 작성자와 사용자 양쪽의 지침은 [부록 A ch21](../../appd-a/ch21-semver/README.md)에 있습니다.

나머지 요구 사항은 범위를 더 좁히거나 직접 정합니다.

```text
== 틸드: 더 좁은 범위 ==
"~1.2.3"         1.2.3 1.2.9
"~1.2"           1.2.3 1.2.9
"~1"             1.2.3 1.2.9 1.3.0 1.9.0

== 와일드카드: 별표 자리는 무엇이든 ==
"1.*"            1.2.3 1.2.9 1.3.0 1.9.0
"1.2.*"          1.2.3 1.2.9
"*"              0.0.3 0.2.3 0.2.9 0.3.0 1.2.3 1.2.9 1.3.0 1.9.0 2.0.0

== 비교와 여러 조건 ==
">=1.2.9"        1.2.9 1.3.0 1.9.0 2.0.0
"<1.3"           0.0.3 0.2.3 0.2.9 0.3.0 1.2.3 1.2.9
"=1.2.3"         1.2.3
">=1.2.9, <1.9"  1.2.9 1.3.0
"<1.2, ^1.2.2"   (없음)

== 프리릴리스 ==
"2"              2.0.0
"2.1.0-beta.1"   2.1.0-beta.1
">=2.0.0"        2.0.0
```

| 종류 | 모양 | 뜻 |
| --- | --- | --- |
| 기본·캐럿 | `"1.2.3"`, `"^1.2.3"` | 적은 버전 이상, 호환되는 범위 안 |
| 틸드(tilde) | `"~1.2.3"`, `"~1.2"` | 적은 버전 이상이되, 둘째 자리 이상 적었으면 첫째·둘째 자리는 그대로이고 셋째 자리만 올라감(1.2.x). 첫째 자리만 적은 `"~1"`은 `"1"`과 같음 |
| 와일드카드(wildcard) | `"1.*"`, `"1.2.*"`, `"*"` | 별표 자리는 무엇이든. `"*"`만 적은 크레이트는 crates.io에 게시할 수 없음 |
| 비교(comparison requirement) | `">=1.2.9"`, `"<1.3"`, `"=1.2.3"` | 범위의 끝이나 정확한 버전을 직접 정함 |
| 여러 조건(multiple version requirements) | `">=1.2.9, <1.9"` | 쉼표로 이은 조건을 모두 만족 |

**프리릴리스(pre-release)** 는 `2.1.0-beta.1`처럼 `-` 뒤에 이름이 붙은 버전입니다. 요구 사항에 프리릴리스를 직접 적었을 때만 후보가 됩니다. 위 출력에서 `"2"`와 `">=2.0.0"`은 숫자로는 더 높은 `2.1.0-beta.1`을 받아들이지 않습니다. 시험판이 모르는 사이에 섞여 들지 않게 하려는 규칙입니다.

**무엇을 쓸까.** Cargo Book의 권장은 간단합니다. 잘 모르겠으면 기본 요구 사항을 쓰고, 세 자리를 모두 적습니다.

- 기본 요구 사항(`"1.2.3"`)은 Cargo가 고를 수 있는 폭이 가장 넓으면서 호환성은 지킵니다. 2.6절에서 보듯 범위가 좁을수록 다른 크레이트의 요구 사항과 겹치지 않아 해석에 실패하기 쉽습니다.
- 세 자리를 모두 적으면 "내가 실제로 써 본 가장 낮은 버전"이 기록됩니다. 1.2.0에서 생긴 함수를 쓰면서 `"1"`이라고 적어 두면, 내 라이브러리를 쓰는 누군가가 1.0.0으로 빌드하다가 컴파일 에러를 만날 수 있습니다. ch1에서 본 대로 내 `Cargo.lock`은 나를 의존성으로 쓰는 쪽에 전해지지 않고 `Cargo.toml`의 범위만 전해지기 때문입니다.
- `"*"`와 `">=1.2"`처럼 위쪽이 열린 요구 사항은 호환되지 않는 새 버전(2.0.0)까지 받아들이므로 피합니다. 위 출력에서 `">=1.2.9"`가 2.0.0을 포함하는 것이 그 예입니다.
- `"~1.2"`나 `"=1.2.3"`처럼 호환 범위보다 좁은 요구 사항은 꼭 필요할 때만 씁니다. 특정 버전을 붙잡아 두는 일은 요구 사항이 아니라 `Cargo.lock`이 합니다.

### 2.3 세 가지 출처: crates.io, git, 경로

[`projects/three-sources`](projects/three-sources/Cargo.toml)는 세 출처에서 의존성을 하나씩 가져옵니다.

```toml
[dependencies]
# 1. crates.io: 이름과 버전 요구 사항만 적습니다.
heck = "0.5"

# 2. git 저장소: 저장소 주소와 커밋을 고르는 키(tag, branch, rev 가운데 하나)를 적습니다.
cfg-if = { git = "https://github.com/rust-lang/cfg-if", tag = "v1.0.4" }

# 3. 로컬 경로: 이 Cargo.toml이 있는 디렉터리를 기준으로 한 상대 경로입니다.
frame = { path = "crates/frame" }
```

값이 문자열 하나(`"0.5"`)면 crates.io의 버전 요구 사항이고, 중괄호로 묶은 표(`{ … }`)면 키로 출처와 옵션을 적습니다. `heck = "0.5"`는 `heck = { version = "0.5" }`의 줄임입니다. 코드에서는 세 크레이트를 쓰는 방법이 똑같습니다(`heck::…`, `cfg_if::…`, `frame::…`). 출처는 `Cargo.toml`만 압니다.

| 출처 | 적는 법 | 어느 버전·커밋이 쓰이나 | 언제 쓰나 |
| --- | --- | --- | --- |
| crates.io | `이름 = "요구 사항"` | 범위 안에서 Cargo가 고른 버전 | 게시된 크레이트. 대부분의 경우 |
| git 저장소 | `{ git = "주소" }`에 `tag`·`branch`·`rev` 가운데 하나 | 적은 태그·브랜치·커밋. 아무것도 안 적으면 기본 브랜치의 최신 커밋 | 아직 게시되지 않은 수정, 비공개 저장소 |
| 로컬 경로 | `{ path = "상대 경로" }` | 그 디렉터리에 지금 있는 소스 | 한 저장소 안에서 나눈 크레이트 |

**git 의존성.** Cargo는 저장소를 내려받아 그 안에서 이름이 맞는 패키지의 `Cargo.toml`을 찾습니다. 저장소의 하위 디렉터리에 있는 크레이트도 저장소 주소만으로 가리킬 수 있습니다. `tag`·`branch`·`rev`는 하나만 쓸 수 있고, `rev`에는 보통 커밋 해시를 적습니다. 브랜치를 적었더라도 Cargo는 처음 받은 커밋을 `Cargo.lock`에 고정하고, `cargo update`를 실행하기 전에는 새 커밋을 받지 않습니다. 그래서 git 의존성도 빌드가 재현됩니다.

**path 의존성.** 경로는 의존할 패키지의 `Cargo.toml`이 있는 바로 그 디렉터리여야 합니다. git과 달리 하위 디렉터리를 뒤지지 않습니다. `three-sources`의 `crates/frame`은 자기 `Cargo.toml`을 가진 별개의 패키지이고, `unicode-width`에 의존합니다. `three-sources`에서 보면 `unicode-width`는 전이 의존성입니다.

**잠금 파일에 남는 출처.** 세 출처는 [`Cargo.lock`](projects/three-sources/Cargo.lock)의 `source` 줄로 구별됩니다. 아래는 세 패키지의 항목에서 `dependencies` 목록을 뺀 일부입니다.

```toml
[[package]]
name = "cfg-if"
version = "1.0.4"
source = "git+https://github.com/rust-lang/cfg-if?tag=v1.0.4#3510ca6abea34cbbc702509a4e50ea9709925eda"

[[package]]
name = "frame"
version = "0.1.0"

[[package]]
name = "heck"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"
```

git 의존성의 `source`는 `git+주소?고른 방법#커밋 해시` 모양입니다. `Cargo.toml`에는 태그 이름만 적었지만 잠금 파일에는 그 태그가 가리키던 커밋이 통째로 적혀 있습니다. 누군가 태그를 다른 커밋으로 옮겨도 이 빌드는 같은 커밋을 씁니다. path 의존성 `frame`에는 `source`가 없습니다. 예제 [`02_lockfile_sources`](examples/02_lockfile_sources.rs)가 이 파일을 읽어 한 줄씩 요약합니다.

git 의존성을 처음 받으면 ch1에서 "없음"이던 Cargo home의 `git/db/`와 `git/checkouts/`가 생깁니다. `three-sources`를 빌드한 뒤 ch1의 예제 `02_cargo_home`을 다시 실행해 보세요.

**게시와 출처.** crates.io에 게시하는 크레이트는 crates.io 밖의 코드에 의존할 수 없습니다(dev-dependencies는 예외). git이나 path로만 적은 의존성이 있으면 게시가 거절됩니다. 그래서 한 저장소의 크레이트 여럿을 함께 게시할 때는 `path`와 `version`을 같이 적습니다.

```toml
frame = { path = "crates/frame", version = "0.1.0" }
```

내 컴퓨터에서는 `path`의 소스를 쓰고, 게시된 뒤에 받아 쓰는 쪽에서는 crates.io의 0.1.0 이상을 씁니다. 이때 Cargo는 경로에 있는 패키지의 버전이 요구 사항에 맞는지 검사합니다(맞지 않을 때의 에러는 4.3절). `git`과 `version`도 같은 방식으로 함께 쓸 수 있습니다. `git`과 `path`는 함께 쓸 수 없습니다.

### 2.4 의존성 표의 종류와 이름 바꾸기

의존성을 적는 표는 쓰임에 따라 넷입니다. 적는 문법은 모두 같습니다.

| 표 | 누가 쓸 수 있나 | 나를 의존성으로 쓰는 쪽에 전해지나 |
| --- | --- | --- |
| `[dependencies]` | 라이브러리, 바이너리, 테스트, 예제, 벤치마크(빌드 스크립트는 쓸 수 없음) | 전해짐 |
| `[dev-dependencies]` | 테스트, 예제, 벤치마크만 | 전해지지 않음 |
| `[build-dependencies]` | 빌드 스크립트(`build.rs`)만 | 전해짐(빌드 스크립트를 실행해야 하므로) |
| `[target.'cfg(…)'.dependencies]` | 조건에 맞는 플랫폼에서만 | 전해짐 |

이 저장소의 챕터 패키지가 `rand`나 `semver`를 `[dev-dependencies]`에 적는 것은 예제(`examples/`)에서만 쓰기 때문입니다. `src/`의 코드에서 dev-dependency를 쓰면 4.1절의 에러가 납니다. 빌드 스크립트는 [ch8](../ch8-build-scripts/README.md)에서 다룹니다.

플랫폼별 의존성(platform-specific dependencies)은 `#[cfg]`와 같은 문법으로 조건을 적습니다.

```toml
[target.'cfg(windows)'.dependencies]
itoa = "1"
```

`three-sources`의 사본에 이 표를 넣고 macOS에서 빌드하면 `itoa`는 컴파일되지 않고 `cargo tree`에도 나오지 않지만, `Cargo.lock`에는 `itoa`가 적힙니다. Cargo는 **모든 플랫폼을 켠 것으로 보고** 잠금 파일을 만들기 때문입니다. 잠금 파일 하나로 어느 운영체제에서든 같은 버전을 쓰게 하려는 것입니다. `cargo tree --target all`을 주면 `itoa`가 보입니다.

**이름 바꾸기(renaming dependencies).** 코드에서 쓰는 이름을 패키지 이름과 다르게 하려면 `package` 키를 씁니다.

```toml
int_text = { version = "1.0.19", package = "itoa" }
```

왼쪽의 `int_text`가 코드에서 쓰는 이름이고(`int_text::Buffer`), `package`가 crates.io의 실제 이름입니다. 한 크레이트의 두 버전에 동시에 의존하거나 이름이 같은 두 출처의 크레이트를 함께 쓸 때 필요합니다.

의존성 한 줄에는 이 밖에 `features`·`default-features`·`optional`([ch5](../ch5-features/README.md))과 `workspace = true`([ch4](../ch4-workspaces/README.md))를 적을 수 있습니다.

### 2.5 의존성을 다루는 명령

**`cargo add`와 `cargo remove`** 는 `Cargo.toml`을 대신 고쳐 줍니다. 손으로 고친 것과 결과는 같습니다. `three-sources`의 사본에서 실행해 얻은 결과입니다.

| 명령 | `Cargo.toml`에 적힌 줄 |
| --- | --- |
| `cargo add itoa` | `[dependencies]`에 `itoa = "1.0.19"` (그때의 최신 버전을 세 자리로) |
| `cargo add itoa@1.0.5` | `itoa = "1.0.5"` |
| `cargo add --dev itoa` | `[dev-dependencies]`에 `itoa = "1.0.19"` |
| `cargo add itoa --rename int_text` | `int_text = { version = "1.0.19", package = "itoa" }` |
| `cargo add --git https://github.com/rust-lang/cfg-if --tag v1.0.4 cfg-if` | `cfg-if = { git = "https://github.com/rust-lang/cfg-if", tag = "v1.0.4", version = "1.0.4" }` |
| `cargo add --path crates/frame` | `frame = { version = "0.1.0", path = "crates/frame" }` |
| `cargo remove itoa` | 그 줄을 지움(`--dev`로 넣은 것은 `cargo remove --dev itoa`) |

`cargo add itoa@1.0.5`는 요구 사항에 `"1.0.5"`를 적지만 잠금 파일에는 1.0.19가 들어갑니다. `"1.0.5"`는 범위이고 Cargo는 범위 안의 가장 높은 버전을 고르기 때문입니다(2.6절). git과 path에는 `cargo add`가 `version`도 함께 적어 줍니다(2.3절의 "게시와 출처").

**`cargo update`** 는 잠금 파일을 고칩니다. `Cargo.toml`은 건드리지 않습니다.

| 명령 | 하는 일 |
| --- | --- |
| `cargo update` | 모든 의존성을 요구 사항 범위 안의 가장 높은 버전으로 다시 고릅니다. |
| `cargo update 이름` | 그 패키지만 올립니다. 다른 패키지는 꼭 필요할 때만 함께 바뀝니다. |
| `cargo update 이름 --precise 버전` | 그 패키지를 정확히 그 버전으로 맞춥니다. 내릴 때도 씁니다. |
| `cargo update --dry-run` | 무엇이 바뀔지만 보여 주고 잠금 파일은 그대로 둡니다. |

`three-sources`의 사본에서 전이 의존성 `unicode-width`를 내렸다가 되돌린 결과입니다.

```console
$ cargo update unicode-width --precise 0.2.0
    Updating crates.io index
 Downgrading unicode-width v0.2.3 -> v0.2.0
$ cargo update --dry-run
    Updating git repository `https://github.com/rust-lang/cfg-if`
    Updating crates.io index
     Locking 1 package to highest Rust 1.100.0 compatible version
    Updating unicode-width v0.2.0 -> v0.2.3
warning: not updating lockfile due to dry run
```

`--precise`로 줄 수 있는 것은 요구 사항 범위 안의 버전뿐입니다. `frame`의 요구 사항이 `"0.2"`라서 0.2.0은 되지만 0.1.14는 `failed to select a version` 에러가 납니다. 새 버전이 문제를 일으켰을 때 `Cargo.toml`은 그대로 두고 잠금 파일에서만 한 버전을 붙잡아 두는 방법이 이것입니다.

**`cargo tree`** 는 해석이 끝난 의존성 그래프를 보여 줍니다.

```console
$ cargo tree
three-sources v0.1.0 (…/projects/three-sources)
├── cfg-if v1.0.4 (https://github.com/rust-lang/cfg-if?tag=v1.0.4#3510ca6a)
├── frame v0.1.0 (…/projects/three-sources/crates/frame)
│   └── unicode-width v0.2.3
└── heck v0.5.0
$ cargo tree -i unicode-width
unicode-width v0.2.3
└── frame v0.1.0 (…/projects/three-sources/crates/frame)
    └── three-sources v0.1.0 (…/projects/three-sources)
```

| 옵션 | 보여 주는 것 |
| --- | --- |
| (없음) | 내 패키지에서 아래로 뻗는 그래프. 괄호 안은 crates.io가 아닌 출처 |
| `--depth 1` | 직접 의존성까지만 |
| `-i 이름` (`--invert`) | 거꾸로 본 그래프. "이 크레이트는 누가 끌어왔는가" |
| `-d` (`--duplicates`) | 두 버전 이상 들어온 크레이트와 각각을 끌어온 경로 |
| `--target all` | 다른 플랫폼 전용 의존성까지 |

같은 이름의 크레이트가 두 버전 들어와 있으면 이름만으로는 가리킬 수 없어 `이름@버전`으로 적습니다. 이 학습 저장소가 그런 경우입니다. 저장소 루트에서 `cargo tree -i rand`를 실행하면 다음이 나옵니다. Part 0 ch2가 직접 쓰는 0.10.4와, Part 0 ch16의 `trpl`이 끌어온 0.8.8이 함께 있기 때문입니다.

```text
error: specification `rand` is ambiguous
help: re-run this command with one of the following specifications
  rand@0.8.8
  rand@0.10.4
```

### 2.6 의존성 해석: Cargo가 버전을 고르는 규칙

그래프의 모든 요구 사항을 모아 패키지마다 실제 버전을 정하는 일을 **의존성 해석**(dependency resolution)이라 하고, 그 일을 하는 Cargo의 부분을 리졸버(resolver)라고 합니다. 결과가 `Cargo.lock`입니다.

**실습 환경: 연습용 레지스트리.** 버전을 고르는 모습을 보려면 "여러 버전이 게시된 크레이트"가 필요한데, 진짜 crates.io는 시간이 지나면 새 버전이 올라와 결과가 달라집니다. 그래서 [`projects/resolver-lab`](projects/resolver-lab/pick/Cargo.toml)은 crates.io 대신 디렉터리 하나를 레지스트리로 씁니다. `resolver-lab/.cargo/config.toml`의 내용은 다음과 같습니다(점으로 시작하는 파일이라 사이트에는 실리지 않습니다).

```toml
[source.crates-io]
replace-with = "practice-registry"

[source.practice-registry]
directory = "vendor"
```

"crates.io를 찾을 일이 생기면 `vendor/` 디렉터리에서 찾으라"는 설정입니다(소스 교체, Cargo Book 3.3.2 Source Replacement). 패키지 가까이에 두는 `.cargo/config.toml` 자체는 [ch7](../ch7-configuration-env/README.md)에서 다룹니다. `vendor/` 아래에는 `이름-버전/` 디렉터리마다 그 버전의 `Cargo.toml`과 `src/lib.rs`, 그리고 Cargo가 요구하는 `.cargo-checksum.json`이 들어 있습니다. 모두 이 실습을 위해 만든 가상의 크레이트이며, 실제 crates.io의 같은 이름 크레이트와는 관계가 없습니다.

| 크레이트 | 버전 | 내용 |
| --- | --- | --- |
| `tint` | 1.0.0 | `hex(색 이름) -> Option<&str>` |
| | 1.1.0 | `yellow` 추가. 일부러 넣은 버그: `#ffff00` 대신 `#ff00ff`를 돌려줌 |
| | 1.2.0 | `names()` 추가. 버그는 그대로 |
| | 2.0.0 | 호환되지 않는 변경: `hex`가 `Result`를 돌려줌 |
| | 2.1.0-beta.1 | 프리릴리스 |
| `banner` | 0.3.0 | `tint = "1.1"`에 의존. `line(색, 글)`, `tint_version()` |

`resolver-lab` 아래의 패키지 다섯 개가 이 레지스트리를 함께 씁니다. Cargo는 설정 파일을 현재 디렉터리에서 위로 올라가며 찾으므로 **명령은 반드시 각 패키지 디렉터리 안에서 실행합니다**. 밖에서 실행하면 Cargo가 진짜 crates.io에서 이름만 같은 무관한 `tint`를 찾습니다(4.3절의 에러). 고쳐 보는 실험은 `resolver-lab` 디렉터리를 통째로 저장소 밖에 복사한 사본에서 합니다.

**규칙 1. 범위 안에서 가장 높은 버전을 고른다.** [`pick`](projects/resolver-lab/pick/Cargo.toml)은 `tint` 하나에만 의존합니다. 사본에서 요구 사항을 바꾸고 `Cargo.lock`을 지운 뒤 `cargo run -q`를 실행해 얻은 결과입니다.

| `tint = …` | Cargo가 고른 버전 | 이유 |
| --- | --- | --- |
| `"1.0"` | 1.2.0 | 1.x 가운데 가장 높음. 2.0.0은 범위 밖 |
| `"~1.0"` | 1.0.0 | 1.0.x만 |
| `"=1.1.0"` | 1.1.0 | |
| `">=1.1, <1.2"` | 1.1.0 | |
| `"2"` | 2.0.0 | 2.1.0-beta.1은 프리릴리스라 후보가 아님 |
| `"2.1.0-beta.1"` | 2.1.0-beta.1 | 프리릴리스를 직접 적음 |
| `"*"`, `">=1.0"` | 2.0.0 | 위쪽이 열려 있어 호환되지 않는 2.0.0까지 감 |
| `"1.3"` | 실패 | 맞는 버전이 없음(4.3절) |

예제 01의 표와 같은 판정이고, 여기에 "그 가운데 가장 높은 것"이 더해졌습니다.

**규칙 2. 잠금 파일에 적힌 버전이 먼저다.** 저장소에 둔 `pick/Cargo.lock`은 `tint` 1.2.0이 "게시되기 전"에 만든 것으로 쳐서 1.1.0이 적혀 있습니다. 그대로 실행하면 더 높은 1.2.0이 레지스트리에 있어도 1.1.0을 씁니다.

```console
$ cargo run -q
Cargo가 고른 tint: 1.1.0
red = Some("#ff0000")
$ cargo update --dry-run
     Locking 1 package to highest Rust 1.100.0 compatible version
    Updating tint v1.1.0 -> v1.2.0 (available: v2.0.0)
warning: not updating lockfile due to dry run
```

잠긴 버전이 요구 사항(`"1.0"`)을 만족하는 한 Cargo는 다시 고르지 않습니다. 올리는 것은 `cargo update`를 실행했을 때이고, 그때도 범위 안에서만 올립니다. `(available: v2.0.0)`은 범위 밖에 더 새 버전이 있다는 알림입니다. 2.0.0으로 가려면 사람이 `Cargo.toml`의 요구 사항을 `"2"`로 고쳐야 합니다. 요구 사항을 `"1.2"`로 올리면 잠긴 1.1.0이 범위를 벗어나므로 다음 빌드에서 Cargo가 그 항목만 다시 고릅니다.

`Locking … to highest Rust 1.100.0 compatible version`이라는 문구는 리졸버 3이 툴체인의 Rust 버전에서 쓸 수 있는 버전을 먼저 고른다는 뜻입니다([Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md)).

**규칙 3. 호환되는 요구 사항들은 버전 하나로 합친다.** [`unified`](projects/resolver-lab/unified/Cargo.toml)는 `tint = "1.0"`을, 그 의존성 `banner`는 `tint = "1.1"`을 요구합니다. 둘 다 1.x이고 범위가 겹치므로 Cargo는 둘을 모두 만족하는 가장 높은 버전 하나만 그래프에 넣습니다.

```console
$ cargo run -q
내가 쓰는 tint:      1.2.0
banner가 쓰는 tint:  1.2.0
[#ff00ff] 주의
tint::hex("yellow") = Some("#ff00ff")
$ cargo tree
unified v0.1.0 (…/projects/resolver-lab/unified)
├── banner v0.3.0
│   └── tint v1.2.0
└── tint v1.2.0
```

하나로 합치는 이유는 둘입니다. 같은 크레이트를 두 번 컴파일하지 않아도 되고, 무엇보다 내가 `tint`에서 얻은 값을 `banner`에 넘길 수 있습니다. 같은 크레이트이므로 타입이 같기 때문입니다. (`[#ff00ff]`는 `tint` 1.1.0과 1.2.0에 심어 둔 버그입니다. 2.7절에서 고칩니다.)

**규칙 4. 호환되지 않는 요구 사항들은 버전마다 따로 넣는다.** [`two-majors`](projects/resolver-lab/two-majors/Cargo.toml)는 `tint = "2"`를, `banner`는 여전히 `tint = "1.1"`을 요구합니다. 두 범위를 함께 만족하는 버전은 없지만 실패하지 않습니다. Cargo는 두 버전을 모두 빌드해 한 프로그램에 넣습니다.

```console
$ cargo run -q
내가 쓰는 tint:      2.0.0
banner가 쓰는 tint:  1.2.0
[#ff00ff] 주의
tint::hex("yellow") = Ok("#ffff00")
$ cargo tree -d
tint v1.2.0
└── banner v0.3.0
    └── two-majors v0.1.0 (…/projects/resolver-lab/two-majors)

tint v2.0.0
└── two-majors v0.1.0 (…/projects/resolver-lab/two-majors)
```

`rustc`에게 `tint` 1.2.0과 `tint` 2.0.0은 이름만 같은 별개의 크레이트입니다. 그래서 이 프로그램에서는 `hex`라는 함수가 둘이고 반환 타입도 다릅니다(`Option`과 `Result`). 덕분에 의존성 하나가 옛 버전에 머물러 있어도 나는 새 버전을 쓸 수 있습니다. 대신 컴파일할 것이 늘고, 한 버전의 타입을 다른 버전을 쓰는 크레이트에 넘기면 "같은 이름인데 다른 타입"이라는 컴파일 에러가 납니다. 그 상황과 해결은 [부록 A ch24](../../appd-a/ch24-re-export-dependencies/README.md)가, 이렇게 한 크레이트가 여러 버전으로 들어온 중복 버전(duplicate versions)을 찾아 줄이는 방법은 [부록 A ch25](../../appd-a/ch25-dependency-graph/README.md)가 다룹니다.

**규칙 5. 호환되는 범위인데 겹치지 않으면 실패한다.** [`conflict-fails`](projects/resolver-lab/conflict-fails/Cargo.toml)는 일부러 실패하는 패키지입니다. `tint = "=1.0.0"`과 `banner`의 `tint = "1.1"`은 둘 다 1.x라서 규칙 3에 따라 하나로 합쳐야 하는데, 두 범위에 함께 드는 버전이 없습니다. 호환되는 버전을 둘 넣는 일은 Cargo가 하지 않습니다.

```console
$ cargo build
error: failed to select a version for `tint`.
    ... required by package `banner v0.3.0`
    ... which satisfies dependency `banner = "^0.3"` of package `conflict-fails v0.1.0 (…/projects/resolver-lab/conflict-fails)`
versions that meet the requirements `^1.1` are: 1.2.0, 1.1.0

all possible versions conflict with previously selected packages

  previously selected package `tint v1.0.0`
    ... which satisfies dependency `tint = "=1.0.0"` of package `conflict-fails v0.1.0 (…/projects/resolver-lab/conflict-fails)`

failed to select a version for `tint` which could resolve this conflict
```

메시지는 아래에서 위로 읽으면 쉽습니다. 먼저 고른 것은 내 패키지의 `"=1.0.0"`이 정한 `tint` 1.0.0이고(`previously selected package`), 그다음 `banner`의 `^1.1`에 맞는 후보 1.2.0과 1.1.0이 모두 그것과 부딪혔습니다. `^1.1`처럼 캐럿이 붙어 나오는 것은 Cargo가 기본 요구 사항을 캐럿으로 바꿔 보여 주기 때문입니다. 고치는 방법은 범위를 넓히는 것입니다(연습 3). 2.2절에서 좁은 요구 사항을 피하라고 한 이유가 이것입니다.

| 같은 크레이트에 대한 두 요구 사항 | Cargo가 하는 일 | 패키지 |
| --- | --- | --- |
| 호환되고 겹친다(`"1.0"`, `"1.1"`) | 버전 하나로 합침 | `unified` |
| 호환되지 않는다(`"2"`, `"1.1"`) | 두 버전을 모두 빌드 | `two-majors` |
| 호환되는데 겹치지 않는다(`"=1.0.0"`, `"1.1"`) | 해석 실패 | `conflict-fails` |

**연습용 레지스트리가 진짜와 다른 점.** 잠금 파일의 `source`는 crates.io로 적히지만(crates.io를 "대신"하는 것이라서) `checksum` 줄이 없습니다. 그리고 `cargo update tint --precise 1.1.0`의 버전 지정이 적용되지 않았습니다(에러 없이 1.2.0 그대로). `--precise`는 2.5절처럼 진짜 crates.io의 의존성에서 확인합니다.

### 2.7 `[patch]`로 의존성 바꿔 끼우기

`unified`의 출력에는 버그가 있었습니다. `tint`의 `yellow`가 `#ff00ff`입니다. 원인이 내 코드가 아니라 의존성에 있을 때 할 수 있는 일은 고친 버전이 게시되기를 기다리는 것만이 아닙니다. `[patch]`는 "이 출처의 이 크레이트를 찾을 때는 내가 가리키는 사본을 쓰라"고 Cargo에 알립니다.

[`patched`](projects/resolver-lab/patched/Cargo.toml)는 `unified`와 소스가 같고 `Cargo.toml`에 표 하나가 더 있습니다.

```toml
[dependencies]
tint = "1.0"
banner = "0.3"

[patch.crates-io]
tint = { path = "../tint-fix" }
```

[`tint-fix`](projects/resolver-lab/tint-fix/Cargo.toml)는 `tint` 1.2.0을 복사해 버그를 고치고 버전을 1.2.1로 올린 것입니다. 아래에서는 "고친 사본"이라고 부릅니다.

```console
$ cargo run -q
내가 쓰는 tint:      1.2.1
banner가 쓰는 tint:  1.2.1
[#ffff00] 주의
tint::hex("yellow") = Some("#ffff00")
$ cargo tree
patched v0.1.0 (…/projects/resolver-lab/patched)
├── banner v0.3.0
│   └── tint v1.2.1 (…/projects/resolver-lab/tint-fix)
└── tint v1.2.1 (…/projects/resolver-lab/tint-fix)
```

`[dependencies]`는 그대로인데 `tint`가 로컬 경로에서 옵니다. 그리고 **`banner`가 쓰는 `tint`까지 바뀌었습니다.** `[dependencies]`에서 `tint`를 path로 바꿨다면 내 패키지의 `tint`만 바뀌고 `banner`는 여전히 레지스트리의 `tint`를 찾았을 것입니다. `[patch]`는 의존성 한 줄이 아니라 출처를 고치므로 그래프 전체에 미칩니다. 알아 둘 점은 다음과 같습니다.

- `[patch.crates-io]`의 `crates-io`는 고칠 출처의 이름입니다. git 의존성을 바꿔 끼울 때는 `[patch."저장소 주소"]`로 적습니다. 값은 `[dependencies]`와 같은 문법이라 `path` 대신 `git`과 `branch`를 적어 고친 브랜치를 가리킬 수도 있습니다.
- 고친 사본의 버전이 요구 사항에 맞아야 쓰입니다. `tint-fix`의 버전을 2.0.5로 바꾸면 `"1.0"`에도 `"1.1"`에도 맞지 않아 쓰이지 않고, 다음 경고와 함께 레지스트리의 1.2.0으로 돌아갑니다.

  ```text
  warning: patch `tint v2.0.5 (…/tint-fix)` was not used in the crate graph
  ```

- Cargo Book은 고친 사본의 버전이 레지스트리의 가장 높은 버전보다 낮으면 쓰이지 않을 수 있다고 설명하지만, cargo 1.100.0에서 `tint-fix`의 버전을 1.1.5로 낮춰 `Cargo.lock` 없이 빌드하니 레지스트리에 1.2.0이 있어도 고친 사본이 쓰였습니다. 요구 사항에 맞는 `[patch]` 항목이 있으면 그것을 먼저 고른 것입니다.
- 이미 `Cargo.lock`이 있는 패키지에 `[patch]`를 넣으면 잠긴 버전이 먼저입니다. 저장소 밖에 복사한 `unified`에 위의 `[patch.crates-io]` 표를 그대로 넣고 빌드하면 잠긴 1.2.0으로 빌드되면서 같은 `was not used in the crate graph` 경고가 나옵니다. `cargo update tint`를 실행하면 `Adding tint v1.2.1 (…/tint-fix)`, `Removing tint v1.2.0`이 출력되고 고친 사본으로 바뀝니다.
- `[patch]`는 워크스페이스 루트(지금은 내 패키지)의 `Cargo.toml`에 적은 것만 효력이 있습니다. 의존성의 `Cargo.toml`에 적힌 `[patch]`는 무시됩니다. 내 라이브러리에 `[patch]`를 적어 게시해도 사용자에게는 전해지지 않습니다.
- `[patch]`는 임시 조치입니다. 고친 버전이 게시되면 요구 사항을 그 버전으로 올리고 표를 지웁니다.

같은 일을 하던 옛 방법 `[replace]`는 더는 권장하지 않는(deprecated) 표입니다. `.cargo/config.toml`의 `paths` 키로도 바꿔 끼울 수 있지만 의존성 목록이 달라지는 수정에는 쓸 수 없어 `[patch]`가 표준입니다.

## 3. 예제 안내

| 예제 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_version_requirements`](examples/01_version_requirements.rs) | 요구 사항마다 받아들이는 버전, 두 버전의 호환 여부 | 1 |
| [`02_lockfile_sources`](examples/02_lockfile_sources.rs) | 잠금 파일의 `source`로 출처 구별 | 2 |
| [`projects/three-sources`](projects/three-sources/Cargo.toml) | 세 출처의 의존성, `cargo add`·`update`·`tree` | 2, 3 |
| [`projects/resolver-lab`](projects/resolver-lab/pick/Cargo.toml) | 버전을 고르는 규칙, 해석 실패, `[patch]` | 3, 4, 5 |

### 01_version_requirements — 요구 사항이 받아들이는 버전

```sh
cargo run -p p3-ch2-dependencies --example 01_version_requirements
```

출력의 앞부분은 2.2절에 실었습니다. 마지막 구역은 Cargo의 호환 기준(0이 아닌 가장 왼쪽 자리)을 두 버전에 적용한 결과입니다. 2.6절의 규칙 3과 4가 이 판정에 따라 갈립니다.

```text
== 두 버전은 호환되는가 ==
1.2.3, 1.9.0: 호환됨 (Cargo가 하나로 합칩니다)
1.9.0, 2.0.0: 호환되지 않음 (함께 필요하면 둘 다 빌드합니다)
0.2.3, 0.2.9: 호환됨 (Cargo가 하나로 합칩니다)
0.2.9, 0.3.0: 호환되지 않음 (함께 필요하면 둘 다 빌드합니다)
0.0.3, 0.0.4: 호환되지 않음 (함께 필요하면 둘 다 빌드합니다)
```

`semver`는 crates.io의 설명란에 "Parser and evaluator for Cargo's flavor of Semantic Versioning"이라고 적힌 크레이트입니다. 이 예제의 판정이 실제 Cargo와 같은지는 2.6절의 표(`pick`에서 실제 cargo로 얻은 결과)로 확인했습니다.

### 02_lockfile_sources — 잠금 파일에서 출처 읽기

```sh
cargo run -p p3-ch2-dependencies --example 02_lockfile_sources
```

`projects/three-sources/Cargo.lock`을 `include_str!`로 컴파일할 때 읽어 들여 `[[package]]`마다 한 줄로 요약합니다. `include_str!`은 파일의 내용을 문자열 리터럴로 넣어 주는 표준 매크로입니다.

```text
cfg-if 1.0.4         git 저장소  https://github.com/rust-lang/cfg-if (tag=v1.0.4, 커밋 3510ca6)
frame 0.1.0          로컬 경로
heck 0.5.0           레지스트리  crates.io
three-sources 0.1.0  로컬 경로
unicode-width 0.2.3  레지스트리  crates.io
```

`Cargo.toml`에 적지 않은 `unicode-width`(전이 의존성)와 패키지 자신도 잠금 파일에는 들어 있습니다.

### projects/three-sources — 세 출처

```sh
cd part3-cargo/ch2-dependencies/projects/three-sources
```

| 명령 | 기대 결과 |
| --- | --- |
| `cargo run -q` | 아래 출력 |
| `cargo test` | 통합 테스트 1개 통과(`frame`의 문서 테스트는 `cargo test -p frame`으로 실행) |
| `cargo tree`, `cargo tree -i unicode-width` | 2.5절의 출력 |

```text
+-----------------------------+
| three-sources: 세 가지 출처 |
+-----------------------------+
플랫폼: 유닉스 계열
```

`heck`(crates.io)이 `ThreeSources`를 `three-sources`로 바꾸고, `cfg-if`(git)의 매크로가 플랫폼 이름을 고르고, `frame`(path)이 상자를 그립니다. `frame`은 한글이 두 칸을 차지하는 것을 `unicode-width`로 계산합니다. 마지막 줄은 Windows에서 `플랫폼: Windows`입니다. 2.5절의 `cargo add`·`cargo update` 실험은 `Cargo.toml`과 `Cargo.lock`을 고치므로 저장소 밖의 사본에서 합니다.

### projects/resolver-lab — 연습용 레지스트리

각 패키지 디렉터리 안에서 실행합니다.

```sh
cd part3-cargo/ch2-dependencies/projects/resolver-lab/pick
```

| 디렉터리 | 명령 | 기대 결과 | 절 |
| --- | --- | --- | --- |
| `pick` | `cargo run -q`, `cargo update --dry-run` | `Cargo가 고른 tint: 1.1.0`, `Updating tint v1.1.0 -> v1.2.0 (available: v2.0.0)` | 2.6 규칙 1·2 |
| `unified` | `cargo run -q`, `cargo tree` | 두 줄 모두 1.2.0, `[#ff00ff] 주의` | 2.6 규칙 3 |
| `two-majors` | `cargo run -q`, `cargo tree -d` | 2.0.0과 1.2.0, 중복 두 건 | 2.6 규칙 4 |
| `conflict-fails` | `cargo build` | **일부러 실패**, 종료 코드 101, ``failed to select a version for `tint`.`` | 2.6 규칙 5 |
| `patched` | `cargo run -q`, `cargo tree` | 두 줄 모두 1.2.1, `[#ffff00] 주의` | 2.7 |

`conflict-fails`를 뺀 네 패키지는 `cargo fmt --check`와 `cargo clippy -- -D warnings`를 통과합니다. `pick`에서 `--dry-run` 없이 `cargo update`를 실행하면 저장소의 `Cargo.lock`이 1.2.0으로 바뀝니다. 되돌리려면 `git checkout Cargo.lock`을 실행합니다. 요구 사항을 바꿔 보는 실험은 사본에서 합니다.

```sh
cp -R part3-cargo/ch2-dependencies/projects/resolver-lab /tmp/resolver-lab
cd /tmp/resolver-lab/pick
```

## 4. 자주 하는 실수

### 4.1 `[dependencies]`에 없는 크레이트를 코드에서 씀 — E0433, [`e0433_crate_not_in_dependencies.rs`](compile_fail/e0433_crate_not_in_dependencies.rs)

```sh
cd part3-cargo/ch2-dependencies
rustc --edition 2024 --out-dir /tmp/p3-ch2 compile_fail/e0433_crate_not_in_dependencies.rs
```

```text
error[E0433]: cannot find module or crate `itoa` in this scope
  --> compile_fail/e0433_crate_not_in_dependencies.rs:18:22
   |
18 |     let mut buffer = itoa::Buffer::new();
   |                      ^^^^ use of unresolved module or unlinked crate `itoa`
   |
   = help: you might be missing a crate named `itoa`
```

- **원인**: Cargo는 `[dependencies]`에 적힌 크레이트만 `rustc`에 연결해 줍니다(ch1의 `--extern`). `unlinked crate`가 그 뜻입니다. 같은 에러가 나는 경우가 셋입니다. 크레이트를 아예 적지 않았거나, `[dev-dependencies]`에만 적고 `src/`에서 썼거나, 전이 의존성을 직접 쓰려고 한 경우입니다. `three-sources`의 `tests/`에서 `use unicode_width::UnicodeWidthStr;`를 쓰면 `frame`이 그 크레이트를 쓰고 있어도 E0432 `unresolved import`가 납니다(`use` 선언에서는 E0432, 경로를 바로 쓰면 E0433).
- **해결**: 직접 쓰는 크레이트는 직접 의존성으로 적습니다(`cargo add itoa`). Cargo로 빌드할 때는 메시지의 help가 `cargo add`를 안내합니다. 전이 의존성과 같은 버전이면 2.6절의 규칙 3에 따라 하나로 합쳐지므로 두 번 컴파일되지 않습니다.

### 4.2 정확한 버전으로 고정해 해석이 실패함

2.6절 규칙 5의 `conflict-fails`입니다. `"=1.0.0"`처럼 좁은 요구 사항은 지금은 문제가 없어 보여도, 나중에 다른 의존성이 같은 크레이트의 더 새 버전을 요구하는 순간 해석을 막습니다. 내 패키지가 라이브러리라면 그 실패는 내가 아니라 사용자의 빌드에서 일어납니다. 버전을 붙잡아 두는 일은 `Cargo.lock`과 `cargo update --precise`에 맡기고 요구 사항은 기본 형태로 둡니다.

### 4.3 그 밖의 에러

| 한 일 | 메시지 | 원인과 해결 |
| --- | --- | --- |
| 범위에 맞는 버전이 없는 요구 사항(`tint = "1.3"`) | ``failed to select a version for the requirement `tint = "^1.3"` `` / `candidate versions found which didn't match: 2.1.0-beta.1, 2.0.0, 1.2.0, ...` | 후보 목록에서 있는 버전을 확인하고 요구 사항을 고칩니다. |
| `resolver-lab` 밖에서 `cargo run --manifest-path …/pick/Cargo.toml` | ``failed to select a version for the requirement `tint = "^1.0"` (locked to 1.1.0)`` / `candidate versions found which didn't match: 1.0.1, 1.0.0` / `location searched: crates.io index` | `.cargo/config.toml`을 읽지 못해 진짜 crates.io를 찾았습니다. 후보로 나온 1.0.1과 1.0.0은 crates.io에 있는, 이름만 같은 무관한 크레이트입니다. 패키지 디렉터리 안에서 실행합니다. |
| 없는 크레이트 이름으로 `cargo add` | ``the crate `no-such-crate-zzz-12345` could not be found in registry index.`` | crates.io에서 정확한 이름을 확인합니다. 비슷한 이름의 다른 크레이트를 넣지 않도록 주의합니다. |
| path가 틀림(`path = "crates/frmae"`) | ``failed to load manifest for dependency `frame` `` / ``failed to read `…/crates/frmae/Cargo.toml` `` | 경로는 의존할 패키지의 `Cargo.toml`이 있는 디렉터리여야 합니다. |
| `path`와 `version`이 어긋남(`version = "0.2"`인데 실제는 0.1.0) | ``failed to select a version for the requirement `frame = "^0.2"` `` / `candidate versions found which didn't match: 0.1.0` | 경로에 있는 패키지의 `version`과 요구 사항을 맞춥니다. |
| `tag`와 `branch`를 함께 적음 | ``dependency (cfg-if) specification is ambiguous. Only one of `branch`, `tag` or `rev` is allowed.`` | 하나만 남깁니다. |
| `git`과 `path`를 함께 적음 | ``dependency (cfg-if) specification is ambiguous. Only one of `git` or `path` is allowed.`` | 임시로 바꿔 끼우려던 것이라면 `[patch]`를 씁니다. |
| 버전이 맞지 않는 `[patch]` | ``warning: patch `tint v2.0.5 (…)` was not used in the crate graph`` | 사본의 `version`을 요구 사항 범위 안으로 맞춥니다(2.7절). |
| 두 버전이 있는 크레이트를 이름만으로 가리킴 | ``specification `rand` is ambiguous`` | `rand@0.10.4`처럼 버전을 붙입니다(2.5절). |

## 5. 연습 문제

`resolver-lab`을 고치는 문제는 3절의 명령으로 만든 사본(`/tmp/resolver-lab`)에서 풉니다.

### 연습 1. 범위 읽기

게시된 버전이 `0.0.3 0.2.3 0.2.9 0.3.0 1.2.3 1.2.9 1.3.0 1.9.0 2.0.0 2.1.0-beta.1`일 때, 다음 요구 사항이 받아들이는 버전을 먼저 종이에 적으세요: `"0.3"`, `"~0.2.3"`, `">=1.3, <2"`, `"1.2.9"`, `"^0.0.3"`. 그다음 예제 01에 확인용 호출을 추가해 맞춰 보세요.

<details>
<summary>힌트</summary>

`main`에 `show("연습", &["0.3", "~0.2.3", ">=1.3, <2", "1.2.9", "^0.0.3"]);`을 한 줄 넣습니다. 기본 요구 사항은 "0이 아닌 가장 왼쪽 자리"가 같은 범위입니다. `"0.3"`은 0.3.0 이상 0.4.0 미만이고, `"^0.0.3"`은 0.0.3 하나뿐입니다.

</details>

### 연습 2. Cargo가 고를 버전 맞히기

사본의 `pick/Cargo.toml`에서 `tint`의 요구 사항을 `"1.1"`, `"~1.1"`, `"<2"`, `">1.2"`로 바꿔 가며 Cargo가 고를 버전을 먼저 예측하고 실행해 확인하세요. 요구 사항을 바꾼 뒤 `Cargo.lock`을 지우지 않으면 어떤 경우에 예측과 다른 결과가 나오는지도 설명하세요.

<details>
<summary>힌트</summary>

`rm Cargo.lock && cargo run -q`로 확인합니다. `">1.2"`는 위쪽이 열려 있습니다. 잠금 파일을 지우지 않으면 잠긴 1.1.0이 새 요구 사항도 만족하는 동안에는 그대로 쓰입니다(규칙 2). 그래서 `"1.1"`과 `"<2"`는 잠금 파일을 지우면 1.2.0, 두면 1.1.0입니다. `"~1.1"`은 어느 쪽이든 1.1.0이고, `">1.2"`는 1.1.0이 범위 밖이라 잠금 파일이 있어도 다시 골라 2.0.0이 됩니다.

</details>

### 연습 3. 해석 실패 고치기

사본의 `conflict-fails`가 빌드되도록 `Cargo.toml`의 `tint` 줄을 두 가지 방법으로 고쳐 보세요. 하나는 범위를 넓히는 방법, 하나는 여전히 정확한 버전을 적는 방법입니다. 각각 실행해 "내가 쓰는 tint"가 무엇인지 확인하고, 라이브러리라면 어느 쪽이 나은지 이유를 적으세요.

<details>
<summary>힌트</summary>

`tint = "1.0"`이면 1.2.0으로, `tint = "=1.1.0"`이면 1.1.0으로 합쳐집니다. 뒤쪽은 지금은 되지만 `banner`의 다음 버전이 `tint = "1.2"`를 요구하면 다시 실패합니다. 정확한 버전이 필요하면 요구 사항은 넓게 두고 `Cargo.lock`으로 고정합니다.

</details>

### 연습 4. patch가 미치는 범위

사본의 `tint-fix/src/lib.rs`에 `"orange" => Some("#ffa500")` 갈래를 추가하고, `patched/src/main.rs`의 `banner::line("yellow", "주의")`를 `banner::line("orange", "주의")`로 바꿔 실행하세요. 같은 수정을 `unified/src/main.rs`에도 해서 두 출력을 비교하고, 차이가 나는 이유를 설명하세요.

<details>
<summary>힌트</summary>

`patched`는 `[#ffa500] 주의`, `unified`는 `[?] 주의`를 출력합니다. `banner`의 소스는 두 패키지에서 같지만 `patched`에서는 `banner`가 링크되는 `tint`가 `tint-fix`이기 때문입니다. `[patch]`는 내 패키지의 의존성 한 줄이 아니라 그래프 전체의 `tint`를 바꿉니다.

</details>

## 6. 참고 자료

- The Cargo Book
  - [2.4 Dependencies](https://doc.rust-lang.org/cargo/guide/dependencies.html)
  - [3.3 Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) — 요구 사항 문법, 출처, 의존성 표의 종류, 이름 바꾸기
  - [3.3.1 Overriding Dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html) — `[patch]`의 쓰임새, `[replace]`, `paths`
  - [3.3.2 Source Replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html) — 연습용 레지스트리에 쓴 디렉터리 소스
  - [3.3.3 Dependency Resolution](https://doc.rust-lang.org/cargo/reference/resolver.html) — 해석 규칙, 권장 사항, 문제 해결
  - [cargo add](https://doc.rust-lang.org/cargo/commands/cargo-add.html), [cargo update](https://doc.rust-lang.org/cargo/commands/cargo-update.html), [cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html)
- 이 저장소의 관련 챕터
  - [ch1](../ch1-getting-started/README.md) 2.4절 — `Cargo.toml`과 `Cargo.lock`, `--locked`
  - [Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md) 2.3절 — `cargo add`, `cargo update --precise`를 처음 써 본 곳
  - [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) — 리졸버 3과 `rust-version`
  - [부록 A ch21](../../appd-a/ch21-semver/README.md) — 시맨틱 버저닝의 약속, 작성자와 사용자의 지침
  - [부록 A ch24](../../appd-a/ch24-re-export-dependencies/README.md) — 두 버전의 타입이 섞일 때
  - [부록 A ch25](../../appd-a/ch25-dependency-graph/README.md) — 의존성 그래프를 관리하는 지침, 중복 버전 줄이기
