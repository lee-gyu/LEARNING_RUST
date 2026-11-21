# ch7. 설정과 환경 변수: `.cargo/config.toml`, 별칭, `env!`

`Cargo.toml`은 패키지가 무엇인지 적는 파일입니다. 그런데 Cargo를 쓰다 보면 패키지와 상관없이 "이 컴퓨터에서, 이 디렉터리에서 Cargo를 어떻게 돌릴지" 정하고 싶은 것이 생깁니다. 자주 치는 긴 명령을 짧게 부르기, 빌드 결과를 둘 곳, 컴파일러에 넘길 옵션, 레지스트리 토큰 같은 것들입니다. 이런 것을 적는 곳이 Cargo 설정(configuration)입니다. 이 챕터는 설정 파일을 어디에 두고 Cargo가 어떻게 읽는지, 같은 설정을 환경 변수와 명령줄로 정하는 방법, 그리고 Cargo가 읽는 환경 변수와 Cargo가 크레이트에 알려 주는 환경 변수를 다룹니다.

앞 챕터들이 이 챕터로 미뤄 둔 것이 여럿 있습니다. [ch1](../ch1-getting-started/README.md) 2.6절의 Cargo home에 있는 `config.toml`, [ch2](../ch2-dependencies/README.md) 2.6절의 연습용 레지스트리가 쓴 `.cargo/config.toml`, [ch6](../ch6-profiles-build-performance/README.md)에서 이름만 보인 `CARGO_PROFILE_*` 환경 변수와 `build.target-dir`·`build.build-dir` 설정, [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md) 2.11절의 `env!("CARGO_MANIFEST_DIR")`입니다.

- 패키지: `p3-ch7-configuration-env`
- 기준 문서: The Cargo Book [3.6 Configuration](https://doc.rust-lang.org/cargo/reference/config.html), [3.7 Environment Variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64)
- 실습 자료: `examples/` 2개, `compile_fail/` 1개, 독립 프로젝트 [`projects/configured`](projects/configured/Cargo.toml). 외부 크레이트와 네트워크는 쓰지 않습니다.
- `projects/configured`의 핵심은 점으로 시작하는 디렉터리의 `.cargo/config.toml` 두 개입니다. 이 파일들은 사이트에 실리지 않으므로 내용을 이 README에 그대로 옮겨 두었습니다(2.2절, 3절).
- 이 문서를 쓴 컴퓨터에는 Cargo home의 `config.toml`도, 저장소 위쪽 디렉터리의 `.cargo/config.toml`도 없었습니다. 독자의 컴퓨터에 그런 파일이 있으면 결과가 달라질 수 있습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Cargo가 설정 파일을 찾는 곳(실행한 디렉터리부터 위로, 마지막에 Cargo home)과 여러 파일의 값을 합치는 규칙(값 하나는 깊은 쪽이 이기고 배열은 이어 붙임), 명령줄 `--config` > 환경 변수 > 설정 파일의 우선순위를 설명할 수 있다.
2. `[alias]`로 별칭을 정의하고, 별칭이 다른 별칭을 부르는 것과 내장 명령을 덮을 수 없다는 제약을 설명할 수 있다.
3. `[env]`, `[build]`, `[profile]` 같은 설정을 파일·환경 변수(`CARGO_<키>`)·`--config`로 지정하고, 설정 안의 경로가 어디를 기준으로 해석되는지 말할 수 있다.
4. Cargo가 읽는 환경 변수와 Cargo가 크레이트에 알려 주는 환경 변수를 구별하고, `env!`·`option_env!`(컴파일할 때)와 `std::env::var`(실행할 때)의 차이를 예제로 보일 수 있다.

## 2. 핵심 개념

### 2.1 매니페스트와 설정

| | `Cargo.toml` (매니페스트) | `.cargo/config.toml` 등 (설정) |
| --- | --- | --- |
| 무엇에 대한 것 | 패키지 자체: 이름, 의존성, 타깃, 기능 | Cargo를 돌리는 방식: 별칭, 빌드 결과 위치, 컴파일러 옵션, 네트워크, 레지스트리 |
| 누구에게 같은가 | 패키지를 쓰는 모든 사람(게시하면 함께 나감) | 설정 파일이 놓인 컴퓨터와 디렉터리 |
| 찾는 기준 | 패키지 디렉터리 | `cargo`를 **실행한 디렉터리** |

예를 들어 "이 패키지는 `thiserror`에 의존한다"는 누가 빌드하든 같아야 하므로 매니페스트에 적습니다. "내 컴퓨터에서는 빌드 결과를 외장 디스크에 둔다"나 "이 프로젝트에서는 `cargo lint`로 Clippy를 돌린다"는 사람이나 작업 환경에 따라 다르므로 설정에 적습니다. 프로필처럼 양쪽에 다 적을 수 있는 것도 있습니다. 이때는 설정의 값이 `Cargo.toml`보다 우선합니다(ch6 2.1절).

### 2.2 설정 파일을 찾는 곳과 합치는 규칙

Cargo는 **현재 디렉터리와 그 위의 모든 디렉터리**에서 `.cargo/config.toml`을 찾고, 마지막으로 Cargo home의 `config.toml`을 읽습니다. `/projects/foo/bar/baz`에서 `cargo`를 실행하면 다음 파일들을 이 순서로 찾아 하나로 합칩니다.

1. `/projects/foo/bar/baz/.cargo/config.toml`
2. `/projects/foo/bar/.cargo/config.toml`
3. `/projects/foo/.cargo/config.toml`
4. `/projects/.cargo/config.toml`
5. `/.cargo/config.toml`
6. `$CARGO_HOME/config.toml` (기본은 `~/.cargo/config.toml`, [ch1](../ch1-getting-started/README.md) 2.6절)

이 구조 덕분에 프로젝트별 설정은 프로젝트 디렉터리에 두어 버전 관리에 넣고, 개인의 기본값은 Cargo home에 둘 수 있습니다. 확장자 없는 `.cargo/config`도 읽지만, `.toml`을 붙이는 것이 권장되는 이름입니다(1.39부터).

**여러 파일에 같은 키가 있으면 합칩니다.**

- 숫자·문자열·불리언 같은 값 하나는 **더 깊은 디렉터리**의 것이 이깁니다. Cargo home이 가장 약합니다.
- 배열은 덮어쓰지 않고 **이어 붙입니다.** 우선순위가 높은 쪽의 원소가 뒤에 옵니다.

`projects/configured`는 이것을 보여 주려고 설정 파일을 두 개 둡니다. 프로젝트 디렉터리의 `.cargo/config.toml`은 이렇습니다.

```toml
# 이 디렉터리(또는 그 아래)에서 cargo를 실행하면 읽히는 프로젝트 설정입니다.

[alias]
# cargo hello 이름  →  cargo run --quiet -- 이름
hello = "run --quiet --"
# 별칭은 다른 별칭을 부를 수 있습니다.
hi = "hello 친구"
lint = "clippy --all-targets -- -D warnings"

[env]
# cargo가 실행하는 rustc, cargo run, cargo test에 넣어 줄 환경 변수입니다.
APP_MODE = { value = "local" }
# relative = true: 이 .cargo 디렉터리의 부모(프로젝트 디렉터리)를 기준으로 한 절대 경로가 됩니다.
APP_DATA = { value = "data", relative = true }

[build]
rustflags = ["--cfg", "from_config"]

[profile.dev]
opt-level = 1
```

`build.rustflags`의 `--cfg from_config`는 모든 `rustc` 호출에 `--cfg from_config`를 붙여 코드의 `cfg!(from_config)`를 참으로 만듭니다. [ch5](../ch5-features/README.md) 2.2절에서 Cargo가 켜진 기능을 `--cfg 'feature="…"'`로 넘기는 것을 봤는데, 같은 장치에 이름만 직접 지은 것입니다. Cargo는 기능 이름과 `test`·`docsrs` 같은 정해진 이름만 `--check-cfg`로 알려 주므로, 직접 지은 이름은 매니페스트의 `[lints.rust]`에서 `unexpected_cfgs` 린트의 `check-cfg`로 알려 줘야 ``unexpected `cfg` condition name`` 경고가 나지 않습니다. `projects/configured/Cargo.toml`이 그렇게 합니다(자세한 것은 [부록 A ch26](../../appd-a/ch26-feature-creep/README.md)).

하위 디렉터리 `ci/`에 있는 `ci/.cargo/config.toml`은 이렇습니다.

```toml
# ci/ 디렉터리 안에서 cargo를 실행할 때만 위의 설정에 더해 읽힙니다.

[env]
# force = true: 셸에 같은 이름의 변수가 있어도 이 값으로 덮어씁니다.
APP_MODE = { value = "ci", force = true }

[build]
# 배열은 덮어쓰지 않고 이어 붙입니다.
rustflags = ["--cfg", "from_ci"]
```

`ci/` 안에서 `cargo`를 실행하면 Cargo는 위로 올라가며 매니페스트를 찾아 같은 패키지를 빌드하지만, 설정은 두 파일을 다 읽습니다. 그래서 `APP_MODE`는 더 깊은 `ci/`의 값 `ci`가 되고, `build.rustflags`는 `["--cfg", "from_config", "--cfg", "from_ci"]`로 이어 붙습니다. 3절에서 출력으로 확인합니다.

**설정은 실행한 디렉터리를 기준으로 찾습니다.** 매니페스트는 패키지 위치로 찾지만 설정은 그렇지 않습니다. 같은 패키지라도 어디서 `cargo`를 실행했느냐에 따라 결과가 달라질 수 있다는 뜻입니다(ch6 2.7절에서 Cargo Book이 이 점을 설정 파일의 단점으로 든 것). 워크스페이스 루트에서 실행하면 **멤버 디렉터리 안의 `.cargo/config.toml`은 읽지 않습니다.** 멤버 `m/`에만 별칭 `mm`을 둔 사본에서 루트에서는 ``error: no such command: `mm` ``이 났고, `m/` 안에서는 별칭이 동작했습니다.

**같은 키는 같은 모양으로 적습니다.** 위에서 프로젝트 쪽 `APP_MODE`를 `"local"`이라는 문자열로, `ci/` 쪽을 `{ value = "ci", force = true }`라는 표로 적으면 두 파일을 함께 읽는 `ci/` 안에서 실행할 때 두 값을 합칠 수 없어 `cargo` 명령 자체가 실패합니다(4절). 그래서 프로젝트 쪽도 `{ value = "local" }`로 적었습니다.

**다른 설정 파일 포함하기.** 맨 위에 `include = ["extra.toml"]`처럼 적으면 다른 파일의 설정을 먼저 읽고 그 위에 자기 값을 덮습니다. 경로는 포함하는 파일을 기준으로 하고 `.toml`로 끝나야 합니다. 이 툴체인에서 `.cargo/config.toml`이 같은 디렉터리의 `extra.toml`에 정의한 별칭을 포함해 쓰는 것을 확인했습니다.

### 2.3 같은 설정을 정하는 세 가지 방법

설정 키 하나는 세 가지 방법으로 정할 수 있고, 아래로 갈수록 우선합니다.

| 방법 | 쓰는 모양 | 경로의 기준 |
| --- | --- | --- |
| 설정 파일 | `[build]` 표 아래 `rustflags = […]` | 그 파일이 든 `.cargo` 디렉터리의 부모 |
| 환경 변수 | `CARGO_BUILD_RUSTFLAGS="…"` | 현재 디렉터리 |
| 명령줄 | `cargo --config 'build.rustflags=[…]' build` 또는 `cargo --config 경로.toml build` | 현재 디렉터리(파일을 주면 설정 파일과 같은 규칙으로, 그 파일이 든 디렉터리의 부모) |

**환경 변수 이름 짓는 법.** 키 `foo.bar`는 `CARGO_FOO_BAR`입니다. 대문자로 바꾸고, 점(`.`)과 하이픈(`-`)을 밑줄로 바꿉니다. ch6에서 쓴 `CARGO_PROFILE_RELEASE_DEBUG`는 키 `profile.release.debug`이고, `CARGO_BUILD_BUILD_DIR`은 `build.build-dir`입니다. 별칭도 `CARGO_ALIAS_SAY="hello 환경변수" cargo say`처럼 정할 수 있습니다. 환경 변수로는 정수·불리언·문자열과 일부 배열만 정할 수 있고, Cargo Book의 키 설명마다 환경 변수를 지원하는지 적혀 있습니다.

**명령줄 `--config`.** TOML 문법의 `키=값`을 줍니다. 여러 번 줄 수 있고 왼쪽부터 합칩니다. 셸이 따옴표를 먹으므로 문자열 값은 작은따옴표로 감쌉니다.

```console
$ cargo --config profile.dev.opt-level=0 build
   Compiling configured v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
$ cargo --config 'env.APP_MODE.value="cli"' hello
안녕하세요, 세계
APP_MODE 실행 시: cli, 컴파일 시: cli
…
```

`APP_MODE`는 설정 파일에서 표(`{ value = … }`)로 정의했으므로 `--config`에서도 `env.APP_MODE.value`로 표 안의 값을 줍니다. `env.APP_MODE="cli"`처럼 문자열로 주면 모양이 달라 합칠 수 없다는 에러가 나고, `env.APP_MODE={value="cli"}`처럼 인라인 표를 주면 `--config`가 인라인 표를 받지 않는다는 에러가 납니다(4절).

**설정 안의 경로.** 실행 파일을 가리키는 값에 경로 구분자가 없으면(`runner = "foo"`) `PATH`에서 찾습니다. 그 밖의 상대 경로는 위 표의 기준을 따릅니다. 설정 파일이 `/my/project/.cargo/config.toml`이면 `"vendor"`는 `/my/project/vendor`입니다. [ch2](../ch2-dependencies/README.md) 2.6절의 `resolver-lab`이 `.cargo/config.toml`에 `directory = "vendor"`라고만 적고도 프로젝트의 `vendor/`를 찾은 것이 이 규칙 덕분이었습니다.

### 2.4 별칭

자주 치는 긴 명령에 짧은 이름을 붙입니다. Cargo에는 내장 별칭이 있어서 `cargo b`는 `cargo build`입니다. `cargo --list`가 명령 목록과 함께 별칭을 보여 줍니다.

```console
$ cargo --list
…
    b                    alias: build
    c                    alias: check
    d                    alias: doc
    …
    hello                alias: run --quiet --
    hi                   alias: hello 친구
    lint                 alias: clippy --all-targets -- -D warnings
    …
    r                    alias: run
    rm                   alias: remove
    t                    alias: test
```

위 목록은 `projects/configured`에서 실행한 것이라 그 프로젝트의 별칭 셋이 함께 나옵니다.

- **값은 문자열이나 배열입니다.** 문자열은 공백으로 나눠 명령과 인자가 됩니다. 인자에 공백이 들어가야 하면 `["run", "--", "공백 있는 인자"]`처럼 배열로 씁니다.
- **별칭 뒤에 준 인자는 펼친 명령 뒤에 붙습니다.** `cargo hello 세상`은 `cargo run --quiet -- 세상`입니다.
- **별칭은 다른 별칭을 부를 수 있습니다.** `hi`는 `hello 친구`로, 다시 `run --quiet -- 친구`로 펼쳐집니다. 서로를 부르는 고리가 생기면 에러입니다(4절).
- **내장 명령은 덮을 수 없습니다.** `build = "run"`처럼 적으면 경고와 함께 무시됩니다(4절). 누가 설정 파일에 무엇을 적었든 `cargo build`가 늘 같은 뜻이어야 하기 때문입니다.

별칭은 `cargo` 뒤에 쓸 수 있는 명령만 부릅니다. 셸 명령을 이어 실행하는 스크립트가 필요하면 별칭이 아니라 별도 도구나 사용자 정의 하위 명령을 씁니다(Part 3 `ch11-commands-tools`).

### 2.5 자주 쓰는 설정 키

Cargo Book 3.6절에는 설정 키가 표 단위로 모여 있습니다. 이 챕터까지 나온 것과 앞으로 다룰 것을 묶으면 다음과 같습니다.

| 표 | 주요 키 | 다루는 곳 |
| --- | --- | --- |
| `[alias]` | 별칭 | 2.4절 |
| `[build]` | `jobs`(병렬 작업 수), `target-dir`·`build-dir`, `rustflags`, `incremental`, `warnings`, `rustc-wrapper` | 이 절, ch6 2.5·2.7절 |
| `[env]` | 빌드와 실행에 넣을 환경 변수 | 이 절 |
| `[profile.<이름>]` | 매니페스트의 프로필과 같은 키 | ch6 |
| `[term]` | `quiet`, `verbose`, `color` | 이 절 |
| `[net]`, `[http]` | `offline`, `retry`, 프록시, 제한 시간 | 표만 |
| `[source.<이름>]`, `paths`, `[patch.<레지스트리>]` | 소스 교체, 의존성 바꿔 끼우기 | [ch2](../ch2-dependencies/README.md) 2.6·2.7절 |
| `[registry]`, `[registries.<이름>]` | 기본 레지스트리, 토큰 | Part 3 `ch10-publishing-semver` |
| `[target.<타깃>]`, `[target.<cfg>]` | `linker`, `runner`, `rustflags` | Part 5 |
| `[install]`, `[cargo-new]`, `[doc]`, `[cache]`, `[resolver]` | `cargo install` 위치, `cargo new`의 VCS, 문서를 열 브라우저, 캐시 자동 정리, 리졸버 동작 | 표만 |

이 가운데 몇 가지를 자세히 봅니다.

**`[env]`.** Cargo가 실행하는 빌드 스크립트, `rustc`, `cargo run`, `cargo test`의 프로세스에 환경 변수를 넣습니다. `rustc`에도 들어가므로 코드의 `env!`·`option_env!`로 컴파일할 때 읽을 수 있고, `cargo run`으로 실행한 프로그램은 `std::env::var`로 실행할 때 읽을 수 있습니다. `projects/configured`의 `tests/cli.rs`는 테스트 프로세스에도 들어오는 것을 확인합니다.

- 셸에 같은 이름의 변수가 이미 있으면 **셸의 값을 덮지 않습니다.** 덮으려면 `force = true`를 줍니다. 사용자가 명령마다 다른 값을 줄 수 있게 하려는 기본값입니다.
- `relative = true`이면 값을 설정 파일 기준의 상대 경로로 보고 **절대 경로**로 바꿔 넣습니다. 프로그램이 어느 디렉터리에서 실행되든 같은 파일을 가리키게 됩니다.
- `cargo run`이 아니라 실행 파일을 직접 실행하면 `[env]`는 들어오지 않습니다. 설정은 Cargo가 읽는 것이고, 실행 파일은 Cargo를 모르기 때문입니다.

**`build.rustflags`.** 모든 `rustc` 호출에 붙일 옵션입니다. 옵션을 정하는 출처가 넷 있는데, 이것들은 합쳐지지 않고 **처음 찾은 하나만** 씁니다.

1. 환경 변수 `CARGO_ENCODED_RUSTFLAGS`(옵션 사이를 `0x1f` 문자로 구분)
2. 환경 변수 `RUSTFLAGS`(공백으로 구분)
3. 설정의 `target.<타깃>.rustflags`와 `target.<cfg>.rustflags`(맞는 것을 모두 이어 붙임)
4. 설정의 `build.rustflags`

그래서 `RUSTFLAGS`를 주면 설정 파일의 `build.rustflags`가 통째로 사라집니다. 반면 `CARGO_BUILD_RUSTFLAGS`는 `build.rustflags` 키의 환경 변수형이라 2.2절의 규칙대로 설정 파일의 배열과 이어 붙습니다. 3절에서 둘의 차이를 봅니다. Cargo Book은 프로필이 정하는 옵션(`-C opt-level` 등)을 `rustflags`로 넘기지 말고 프로필 설정을 쓰라고 권합니다. 처음 쓰는 옵션으로 바꾸면 의존성까지 모두 다시 컴파일한다는 점도 기억해 둡니다([부록 A ch32](../../appd-a/ch32-continuous-integration/README.md)의 `RUSTFLAGS=-Dwarnings`가 그 예). 이전에 쓴 옵션으로 돌아가면 그때의 결과가 build 디렉터리에 남아 있어 다시 컴파일하지 않습니다(ch6 2.5절의 해시).

**`build.warnings`.** 작업 중인 패키지의 경고(린트)를 `"warn"`(기본), `"allow"`(숨김), `"deny"`(실패)로 바꿉니다. 부록 A ch32는 CI 스크립트에서 이 키의 환경 변수형 `CARGO_BUILD_WARNINGS=deny`를 썼습니다.

**`[term]`.** `cargo`의 출력 방식입니다. `CARGO_TERM_VERBOSE=true`는 매번 `-v`를 붙인 것과 같고, `CARGO_TERM_QUIET=true`는 `-q`와 같습니다.

**비밀 값은 따로 둡니다.** 레지스트리 토큰 같은 값은 설정 파일이 아니라 Cargo home의 `credentials.toml`에 둡니다. `cargo login`이 이 파일을 만들고 고칩니다. 프로젝트의 `.cargo/config.toml`은 버전 관리에 넣는 경우가 많아서 섞이면 안 되기 때문입니다(Part 3 `ch10-publishing-semver`).

### 2.6 Cargo가 읽는 환경 변수

설정 키의 환경 변수형(`CARGO_<키>`) 말고도, Cargo가 동작을 바꾸려고 읽는 환경 변수가 있습니다. 자주 만나는 것들입니다.

| 환경 변수 | 하는 일 | 이 저장소에서 |
| --- | --- | --- |
| `CARGO_HOME` | Cargo home의 위치 | [ch1](../ch1-getting-started/README.md) 2.6절 |
| `CARGO_TARGET_DIR` | target 디렉터리의 위치(현재 디렉터리 기준) | ch6 2.5절 |
| `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS` | 모든 `rustc` 호출에 붙일 옵션 | 2.5절 |
| `RUSTDOCFLAGS` | 모든 `rustdoc` 호출에 붙일 옵션 | Part 4 |
| `RUSTC`, `RUSTDOC`, `RUSTFMT` | `rustc`·`rustdoc`·`rustfmt` 대신 실행할 프로그램 | |
| `RUSTC_WRAPPER` | `rustc`를 감싸 실행할 프로그램(`sccache` 등) | ch6 2.5절 |
| `CARGO_INCREMENTAL` | `1`이면 증분 컴파일을 켜고 `0`이면 끔 | ch6 2.2절 |
| `CARGO_LOG` | Cargo 자신의 디버그 로그 | |
| `CARGO` | 하위 프로세스에 알려 줄 `cargo`의 경로를 이 값으로 바꿈 | 2.7절 |
| `TERM`, `BROWSER` | `TERM=dumb`이면 진행 막대를 끔, `cargo doc --open`이 쓸 브라우저 | |

`CARGO_TARGET_DIR`, `CARGO_INCREMENTAL=0`(`-v`의 `-C incremental=…`이 사라짐), `CARGO_TERM_VERBOSE=true`, `CARGO_PROFILE_DEV_OPT_LEVEL=0`은 `projects/configured`의 사본에서 동작을 확인했습니다. 나머지는 Cargo Book 3.7절의 설명입니다.

### 2.7 Cargo가 크레이트에 알려 주는 환경 변수

방향이 반대인 환경 변수도 있습니다. Cargo는 크레이트를 **컴파일할 때** `rustc` 프로세스에 패키지 정보를 환경 변수로 넣어 줍니다. 코드는 `env!`로 그 값을 읽어 실행 파일에 넣을 수 있습니다. `cargo run`과 `cargo test`로 **실행할 때**는 이 가운데 일부만 들어옵니다(표의 오른쪽 열). Cargo Book 3.7절은 실행할 때도 이 변수들이 들어온다고 적지만, 1.100.0에서 프로그램이 실행 중에 본 것은 `CARGO`, `CARGO_MANIFEST_DIR`·`CARGO_MANIFEST_PATH`, `CARGO_PKG_*`, 그리고 `cargo test`의 통합 테스트에서 `CARGO_BIN_EXE_<이름>`뿐이었습니다. 빌드 스크립트가 없는 패키지에서 잰 것이고, 빌드 스크립트가 있는 패키지는 `OUT_DIR`과 빌드 스크립트가 넣은 값도 실행할 때 들어옵니다([ch8](../ch8-build-scripts/README.md) 2.4절).

| 환경 변수 | 값 | 실행할 때도 있음 |
| --- | --- | --- |
| `CARGO_PKG_NAME`, `CARGO_PKG_VERSION` | 패키지 이름, 버전 전체 | 예 |
| `CARGO_PKG_VERSION_MAJOR`·`_MINOR`·`_PATCH`·`_PRE` | 버전의 첫째·둘째·셋째 자리와 프리릴리스 부분 | 예 |
| `CARGO_PKG_AUTHORS`, `_DESCRIPTION`, `_HOMEPAGE`, `_REPOSITORY`, `_LICENSE`, `_LICENSE_FILE`, `_RUST_VERSION` | 매니페스트의 같은 키. 없으면 빈 문자열(`AUTHORS`는 콜론으로 구분) | 예 |
| `CARGO_PKG_README` | 매니페스트의 `readme`. 적지 않았어도 패키지 디렉터리에 `README.md`가 있으면 `"README.md"` | 예 |
| `CARGO_MANIFEST_DIR`, `CARGO_MANIFEST_PATH` | 패키지 매니페스트가 있는 디렉터리, 매니페스트 파일의 경로(절대 경로) | 예 |
| `CARGO` | 빌드하고 있는 `cargo`의 경로 | 예 |
| `CARGO_CRATE_NAME` | 지금 컴파일하는 크레이트의 이름(타깃 이름의 `-`를 `_`로) | 아니요 |
| `CARGO_BIN_NAME` | 지금 컴파일하는 바이너리의 이름(바이너리와 바이너리 예제만) | 아니요 |
| `CARGO_PRIMARY_PACKAGE` | 명령줄에서 고른 패키지(`-p` 또는 현재 디렉터리)일 때만 있음 | 아니요 |
| `CARGO_BIN_EXE_<이름>` | 통합 테스트·벤치마크에만: 이 패키지 실행 파일의 절대 경로 | 예(통합 테스트) |
| `CARGO_TARGET_TMPDIR` | 통합 테스트·벤치마크에만: 테스트가 쓸 target 디렉터리 안의 임시 디렉터리 | 아니요 |
| `OUT_DIR` | 빌드 스크립트가 있을 때만: 빌드 스크립트가 결과를 둔 디렉터리([ch8](../ch8-build-scripts/README.md)) | 예(빌드 스크립트가 있을 때) |

- **매니페스트에 없는 값은 빈 문자열입니다.** 변수가 아예 없으면 `env!`가 컴파일 에러를 내므로, Cargo는 적지 않은 키도 빈 문자열로 넣어 줍니다. 예제 01의 `CARGO_PKG_HOMEPAGE`가 `""`인 것이 그 예입니다. `readme`만은 예외입니다. 매니페스트가 `readme`를 적지 않으면 Cargo가 패키지 디렉터리의 `README.md` 등을 찾아 쓰기 때문입니다.
- **`env!`는 컴파일할 때, `std::env::var`는 실행할 때 읽습니다.** `env!("CARGO_PKG_VERSION")`은 실행 파일에 박힌 문자열이라 어디서 어떻게 실행하든 같습니다. `std::env::var("CARGO_PKG_VERSION")`은 `cargo run`으로 실행했을 때만 값이 있습니다. 프로그램이 자기 버전을 출력하려면 `env!`를 씁니다. [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md)의 `02_os_string`이 실행할 때 `CARGO_PKG_NAME`을 읽어, 직접 실행하면 `None`이 되는 것을 이미 보였습니다.
- **`option_env!`는 없어도 되는 변수에 씁니다.** `env!`는 변수가 없으면 컴파일 에러(4절의 `compile_fail`)이고, `option_env!`는 `Option<&'static str>`을 돌려줍니다.
- **컴파일할 때 읽은 변수는 Cargo가 기억합니다.** `env!`나 `option_env!`로 읽은 변수의 값이 바뀌면 Cargo는 그 크레이트를 다시 컴파일합니다. `cargo build -v`의 `Dirty` 줄에 `the environment variable GREETING changed`라고 나옵니다(예제 02).
- [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md) 2.11절이 HTML 파일의 경로를 `env!("CARGO_MANIFEST_DIR")`로 만든 것은, 컴파일할 때의 패키지 디렉터리를 실행 파일에 넣어 두면 어느 디렉터리에서 실행해도 같은 파일을 찾기 때문입니다. 다만 그 경로는 빌드한 컴퓨터의 경로이므로, 실행 파일을 다른 컴퓨터로 옮기면 쓸 수 없습니다.

빌드 스크립트에는 이보다 많은 변수(`TARGET`, `HOST`, `PROFILE`, `OPT_LEVEL`, 켜진 기능마다의 `CARGO_FEATURE_<이름>`, 타깃 정보 `CARGO_CFG_*` 등)가 들어갑니다. 빌드 스크립트는 컴파일된 뒤에 실행되므로 이것들은 `env!`가 아니라 `std::env::var`로 읽습니다. [ch8](../ch8-build-scripts/README.md)에서 다룹니다.

## 3. 예제 안내

### 01_pkg_env — 컴파일할 때와 실행할 때

[`examples/01_pkg_env.rs`](examples/01_pkg_env.rs)는 Cargo가 알려 주는 변수를 `env!`로 읽어 출력하고, 그 가운데 둘을 `std::env::var`로도 읽어 출력합니다. 저장소 루트에서 `cargo run`으로 실행한 뒤, 같은 실행 파일을 Cargo 없이 직접 실행합니다.

```console
$ cargo run -q -p p3-ch7-configuration-env --example 01_pkg_env
[컴파일할 때 넣은 값]
CARGO_PKG_NAME:        p3-ch7-configuration-env
CARGO_PKG_VERSION:     0.0.0
  MAJOR/MINOR/PATCH/PRE: 0/0/0/""
CARGO_PKG_DESCRIPTION: Part 3 ch7: Cargo 설정과 환경 변수
CARGO_PKG_HOMEPAGE:    ""
CARGO_CRATE_NAME:      01_pkg_env
CARGO_BIN_NAME:        01_pkg_env
CARGO_MANIFEST_DIR:    …/ch7-configuration-env
[실행할 때 환경에 있는 값]
CARGO_PKG_NAME:        p3-ch7-configuration-env
CARGO_MANIFEST_DIR:    …/ch7-configuration-env
$ ./target/debug/examples/01_pkg_env
[컴파일할 때 넣은 값]
…(위와 같음)
[실행할 때 환경에 있는 값]
CARGO_PKG_NAME:        (없음)
CARGO_MANIFEST_DIR:    (없음)
```

`description`은 이 챕터의 `Cargo.toml`에 적었고 `homepage`는 적지 않았습니다. 직접 실행하면 `[컴파일할 때 넣은 값]`은 그대로이고 실행할 때의 값만 사라집니다.

### 02_build_time_env — 내가 정한 변수를 컴파일할 때 읽기

[`examples/02_build_time_env.rs`](examples/02_build_time_env.rs)는 환경 변수 `GREETING`을 `option_env!`(컴파일할 때)와 `std::env::var`(실행할 때)로 읽어 출력합니다. 아래 순서대로 실행합니다.

```console
$ cargo run -q -p p3-ch7-configuration-env --example 02_build_time_env
GREETING 컴파일 시: (없음)
GREETING 실행 시:   (없음)
$ GREETING=안녕 cargo run -q -p p3-ch7-configuration-env --example 02_build_time_env
GREETING 컴파일 시: 안녕
GREETING 실행 시:   안녕
$ GREETING=잘가 ./target/debug/examples/02_build_time_env
GREETING 컴파일 시: 안녕
GREETING 실행 시:   잘가
```

두 번째 명령에서 `GREETING`의 값이 바뀌었으므로 Cargo가 예제를 다시 컴파일했고, `안녕`이 실행 파일에 박혔습니다. 세 번째는 Cargo를 거치지 않았으므로 다시 컴파일되지 않고, 컴파일할 때의 `안녕`과 실행할 때의 `잘가`가 갈립니다. 다시 `GREETING` 없이 `cargo run`을 실행하면 또 다시 컴파일되어 첫 번째 출력으로 돌아갑니다. 거기서 `GREETING=안녕`을 주고 `-v`로 빌드하면 다시 컴파일한 이유가 보입니다.

```console
$ cargo run -q -p p3-ch7-configuration-env --example 02_build_time_env
GREETING 컴파일 시: (없음)
GREETING 실행 시:   (없음)
$ GREETING=안녕 cargo build -v -p p3-ch7-configuration-env --example 02_build_time_env
       Dirty p3-ch7-configuration-env v0.0.0 (…): the environment variable GREETING changed
   Compiling p3-ch7-configuration-env v0.0.0 (…)
…
```

### projects/configured — 설정 파일 둘이 바꾸는 것

설정 파일이 실행 결과를 어떻게 바꾸는지 출력하는 프로그램입니다. 설정 파일 두 개의 내용은 2.2절에 있습니다.

| 파일 | 내용 |
| --- | --- |
| [`Cargo.toml`](projects/configured/Cargo.toml) | 빈 `[workspace]`, `[lints.rust]`의 `check-cfg`로 직접 지은 cfg 이름 셋(`from_config`, `from_ci`, `from_env`)을 알림(2.2절) |
| `.cargo/config.toml` | 별칭 셋, `[env]` 둘, `build.rustflags`, `[profile.dev]` (2.2절에 인용) |
| `ci/.cargo/config.toml` | `ci/` 안에서 실행할 때만 더해지는 `[env]`와 `build.rustflags` (2.2절에 인용) |
| [`src/main.rs`](projects/configured/src/main.rs) | 인사, `APP_MODE`(실행할 때·컴파일할 때), `APP_DATA`, 켜진 cfg를 출력 |
| [`tests/cli.rs`](projects/configured/tests/cli.rs) | `CARGO_BIN_EXE_configured`로 실행 파일을 실행하는 테스트, `[env]`와 `CARGO_TARGET_TMPDIR`을 확인하는 테스트 |
| [`data/sample.txt`](projects/configured/data/sample.txt) | `APP_DATA`가 가리키는 디렉터리에 있는지 확인할 파일 |

프로젝트 디렉터리에서 실행합니다. 출력의 `…` 줄은 위 출력과 같은 줄을 줄인 것입니다.

```console
$ cargo hello
안녕하세요, 세계
APP_MODE 실행 시: local, 컴파일 시: local
APP_DATA: …/data (절대 경로: true, sample.txt 있음: true)
cfg: from_config=true from_ci=false from_env=false
$ cargo hi
안녕하세요, 친구
…
$ cd ci && cargo hello && cd ..
안녕하세요, 세계
APP_MODE 실행 시: ci, 컴파일 시: ci
APP_DATA: …/data (절대 경로: true, sample.txt 있음: true)
cfg: from_config=true from_ci=true from_env=false
```

`ci/`에서는 `APP_MODE`가 더 깊은 설정의 값으로 바뀌고, `rustflags`가 이어 붙어 `from_config`와 `from_ci`가 함께 켜졌습니다. 환경 변수와 명령줄로 바꿔 본 결과는 다음과 같습니다(바뀐 줄만 적었습니다).

| 명령 | 바뀐 출력 | 이유 |
| --- | --- | --- |
| `APP_MODE=shell cargo hello` | `APP_MODE 실행 시: shell, 컴파일 시: shell` | `[env]`는 셸의 값을 덮지 않음 |
| `(cd ci && APP_MODE=shell cargo hello)` | `APP_MODE 실행 시: ci, 컴파일 시: ci` | `ci/` 쪽은 `force = true` |
| `cargo --config 'env.APP_MODE.value="cli"' hello` | `APP_MODE 실행 시: cli, 컴파일 시: cli` | 명령줄이 설정 파일보다 우선 |
| `RUSTFLAGS="--cfg from_env" cargo hello` | `cfg: from_config=false from_ci=false from_env=true` | `RUSTFLAGS`가 있으면 `build.rustflags`는 쓰지 않음 |
| `CARGO_BUILD_RUSTFLAGS="--cfg from_env" cargo hello` | `cfg: from_config=true from_ci=false from_env=true` | 키의 환경 변수형은 설정 파일의 배열과 이어 붙음 |
| `cargo build` 뒤 `./target/debug/configured` | `APP_MODE 실행 시: (없음), 컴파일 시: local` / `APP_DATA: (없음)` | 직접 실행하면 `[env]`가 들어오지 않음. 컴파일할 때의 값은 박혀 있음 |

그 밖의 명령과 기대 결과입니다.

| 명령 | 기대 결과 |
| --- | --- |
| `cargo build --locked` | ``Finished `dev` profile [optimized + debuginfo]``. 설정의 `opt-level = 1` 때문에 `dev`인데도 `optimized` |
| `cargo test --locked` | 통합 테스트 2개 통과. `test`는 `dev`를 물려받으므로 ``Finished `test` profile [optimized + debuginfo]`` |
| `cargo lint` | `cargo clippy --all-targets -- -D warnings`와 같음. 경고 없음 |
| `cargo --list` | 2.4절의 목록에 `hello`·`hi`·`lint`가 있음 |
| `cargo clean` | `target/` 삭제(실습을 마친 뒤) |

`RUSTFLAGS`나 `rustflags` 설정을 처음 쓰는 값으로 바꾸면 전부 다시 컴파일됩니다. 의존성이 없는 작은 프로젝트라 금방 끝납니다.

## 4. 자주 하는 실수

### 4.1 컴파일할 때 없는 변수를 `env!`로 읽음 — [`env_not_defined.rs`](compile_fail/env_not_defined.rs)

```console
$ env -u API_TOKEN rustc --edition 2024 --out-dir ../../target/tmp/p3-ch7 compile_fail/env_not_defined.rs
error: environment variable `API_TOKEN` not defined at compile time
  --> compile_fail/env_not_defined.rs:14:17
   |
14 |     let token = env!("API_TOKEN");
   |                 ^^^^^^^^^^^^^^^^^
   |
   = help: use `std::env::var("API_TOKEN")` to read the variable at run time
```

챕터 디렉터리에서 실행합니다. 에러 코드가 없는 에러입니다. `env!`는 컴파일하는 순간의 환경에서 값을 찾는데, 없으면 실행 파일에 넣을 문자열을 만들 수 없습니다. 고치는 방법은 값을 언제 정하느냐에 따라 다릅니다.

- 실행할 때 정할 값(토큰, 설정 경로)이면 도움말대로 `std::env::var`로 실행할 때 읽습니다. 토큰 같은 비밀 값을 `env!`로 실행 파일에 박으면, 실행 파일을 가진 사람 누구나 꺼내 볼 수 있다는 문제도 있습니다.
- 빌드할 때 정하되 없어도 되는 값이면 `option_env!`와 기본값을 씁니다(예제 02).
- 늘 있어야 하는 빌드 정보면 `[env]`에 적어 두거나, Cargo가 넣어 주는 `CARGO_PKG_*`를 씁니다.

셸에 `API_TOKEN`이 있으면 컴파일이 성공하므로 `env -u API_TOKEN`으로 변수를 지우고 확인합니다.

### 4.2 설정과 별칭의 실수

모두 `projects/configured`를 저장소 밖에 복사한 사본(멤버 설정은 따로 만든 워크스페이스)에서 확인했습니다. 경로는 줄였습니다.

| 상황 | 메시지 | 해결 |
| --- | --- | --- |
| 같은 `[env]` 키를 한 파일은 문자열, 다른 파일은 표로 적음 | `error: could not load Cargo configuration` / ``failed to merge key `APP_MODE` between …`` / `expected table, but found string` | 양쪽을 같은 모양(`{ value = … }`)으로 적습니다. |
| 표로 정의한 키를 `--config`로 문자열을 줘 덮음 | ``failed to merge key `APP_MODE` between …/.cargo/config.toml and --config cli option`` | `--config 'env.APP_MODE.value="cli"'`처럼 표 안의 값을 줍니다. |
| `--config`에 인라인 표를 줌 | ``error: --config argument `env.APP_MODE={value="cli"}` sets a value to an inline table, which is not accepted`` | 위처럼 점으로 이은 키를 씁니다. |
| 내장 명령 이름으로 별칭을 정의함 | ``warning: user-defined alias `build` is ignored, because it is shadowed by a built-in command`` | 다른 이름을 씁니다. 원래 명령이 그대로 실행됩니다. |
| 별칭이 서로를 부름 | `error: alias loop1 has unresolvable recursive definition: loop1 -> loop2 -> loop1` | 고리를 끊습니다. |
| 별칭이 없는 명령을 가리킴 | ``error: no such command: `frobnicate` `` | 별칭의 값을 고칩니다. |
| 키를 엉뚱한 표 아래에 적음(예: `[profile.dev]` 아래에 별칭) | ``warning: unused config key `profile.dev.build` in `…/.cargo/config.toml` `` | 경고만 나고 값은 무시됩니다. 표 머리 아래 어디에 적었는지 확인합니다. |
| 워크스페이스 루트에서 실행했는데 멤버의 `.cargo/config.toml`에 둔 별칭을 씀 | ``error: no such command: `mm` `` | 루트의 `.cargo/config.toml`로 옮깁니다(2.2절). |
| `RUSTFLAGS`를 주자 설정의 `build.rustflags`가 사라짐 | (메시지 없음) | 함께 쓰려면 `CARGO_BUILD_RUSTFLAGS`를 쓰거나 설정 파일에 둡니다(2.5절). |
| `[env]`의 값이 적용되지 않음 | (메시지 없음) | 셸에 같은 변수가 있으면 셸이 이깁니다. `force = true`를 주거나, 실행 파일을 직접 실행하지 않았는지 확인합니다. |

## 5. 연습 문제

연습마다 `projects/configured`를 저장소 밖에 새로 복사해 시작합니다(`.cargo` 디렉터리가 함께 복사되게 디렉터리째 복사합니다).

### 연습 1. 별칭 만들기

`hello`처럼 동작하되 릴리스 빌드로 실행하는 별칭 `rr`을 만드세요. `cargo rr 별`의 출력 첫 줄은 `안녕하세요, 별`이어야 합니다. 먼저 `rr = "hello --release"`로 해 보고 왜 원하는 대로 되지 않는지 설명하세요. 빌드가 정말 릴리스인지 확인하는 방법도 찾으세요.

<details>
<summary>힌트</summary>

`hello`는 `run --quiet --`로 펼쳐지므로 `hello` 뒤에 `--release`를 붙이면 `--` 뒤의 프로그램 인자가 되어 버립니다(첫 줄이 `안녕하세요, --release`). 별칭은 `rr = "run --quiet --release --"`처럼 `run`에서 직접 만듭니다. `--quiet`를 빼고 실행하면 ``Finished `release` profile`` 줄로 확인할 수 있습니다.

</details>

### 연습 2. 우선순위 맞히기

다음 두 명령의 `APP_MODE` 줄을 먼저 예상한 뒤 확인하세요. `(cd ci && cargo --config 'env.APP_MODE.value="cli"' hello)`, `(cd ci && CARGO_BUILD_RUSTFLAGS="--cfg from_env" cargo hello)`(이것은 `cfg:` 줄을 예상합니다). 괄호는 하위 셸에서 실행해 명령이 끝난 뒤 원래 디렉터리로 돌아오게 합니다.

<details>
<summary>힌트</summary>

명령줄 `--config`는 설정 파일보다 우선하므로 `ci/`의 `force = true`와 상관없이 `cli`가 됩니다(`force`는 셸의 환경 변수와의 관계만 정합니다). 두 번째는 배열 셋(프로젝트, `ci/`, 환경 변수)이 이어 붙어 `from_config=true from_ci=true from_env=true`입니다.

</details>

### 연습 3. 버전 출력하기

`src/main.rs`가 `--version` 인자를 받으면 `configured 0.1.0`을 출력하고 끝나도록 고치세요. 버전 문자열을 코드에 직접 적지 말고, `Cargo.toml`의 `version`을 `0.2.0`으로 바꾸면 출력도 따라 바뀌게 하세요. 실행 파일을 직접 실행해도 같아야 합니다.

<details>
<summary>힌트</summary>

`env!("CARGO_PKG_NAME")`과 `env!("CARGO_PKG_VERSION")`을 씁니다. 컴파일할 때 박히므로 `./target/debug/configured --version`으로 직접 실행해도 같은 값이 나옵니다. `std::env::var`로 읽으면 직접 실행했을 때 값이 없습니다.

</details>

### 연습 4. 빌드 결과 위치 옮기기

설정 파일에 키 하나를 더해, `cargo build`의 결과가 프로젝트의 `target/`이 아니라 프로젝트 옆의 `../shared-target/`에 놓이게 하세요. 그다음 `ci/` 안에서 `cargo build`를 실행하면 결과가 어디에 놓이는지 예상하고 확인하세요.

<details>
<summary>힌트</summary>

`[build]` 표에 `target-dir = "../shared-target"`을 더합니다. 설정 파일의 상대 경로는 `.cargo` 디렉터리의 부모(프로젝트 디렉터리) 기준이므로, `ci/` 안에서 실행해도 같은 `../shared-target/`에 놓입니다. 같은 값을 환경 변수 `CARGO_BUILD_TARGET_DIR=../shared-target`으로 주면 현재 디렉터리 기준이 되어 `ci/`에서는 다른 곳을 가리킵니다.

</details>

## 6. 참고 자료

- The Cargo Book: [3.6 Configuration](https://doc.rust-lang.org/cargo/reference/config.html), [3.7 Environment Variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
- 표준 라이브러리: [`env!`](https://doc.rust-lang.org/std/macro.env.html), [`option_env!`](https://doc.rust-lang.org/std/macro.option_env.html), [`std::env::var`](https://doc.rust-lang.org/std/env/fn.var.html)
