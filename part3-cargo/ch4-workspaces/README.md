# ch4. 워크스페이스: 가상 매니페스트, 멤버 간 의존, 설정 상속

패키지 하나가 커지면 여러 패키지로 나누게 됩니다. 나눈 패키지들을 한 저장소에서 함께 빌드하고, 함께 테스트하고, 같은 버전의 의존성으로 묶어 두는 장치가 워크스페이스(workspace)입니다. 사실 이 학습 저장소가 처음부터 워크스페이스였습니다. Part 0 ch1에서 "저장소 루트의 `Cargo.toml`이 모든 챕터 패키지를 하나로 묶는다"고만 하고 넘어간 것을 이 챕터에서 풉니다.

- 패키지: `p3-ch4-workspaces`
- 기준 문서: The Cargo Book [3.2 Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64)
- 실습 자료: `examples/` 2개, 독립 프로젝트 [`projects/bookshelf`](projects/bookshelf/Cargo.toml)(라이브러리 둘과 바이너리 하나로 이뤄진 워크스페이스). `compile_fail/`은 없습니다. 이 챕터의 실수는 모두 Cargo가 내는 에러라 4절에 메시지를 실었습니다.
- 외부 크레이트: `bookshelf`가 crates.io의 `unicode-width` 0.2.3을 씁니다(2026년 11월 기준, `Cargo.lock`이 고정). 처음 빌드할 때 네트워크가 필요합니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 워크스페이스의 멤버들이 함께 쓰는 것(`Cargo.lock`, `target/`, 명령)을 설명하고, 루트 패키지가 있는 워크스페이스와 가상 매니페스트를 구별할 수 있다.
2. `members`·`exclude`·`default-members`로 멤버를 정하고, `-p`와 `--workspace`로 명령의 대상을 고를 수 있다.
3. 멤버끼리 path 의존성으로 이어진 워크스페이스에서, 한 멤버를 고쳤을 때 무엇이 다시 컴파일되는지 설명할 수 있다.
4. `[workspace.package]`·`[workspace.dependencies]`·`[workspace.lints]`에 적은 값을 멤버가 물려받게 할 수 있다.
5. 워크스페이스 루트에서만 효력이 있는 설정을 알고, 워크스페이스 디렉터리 안에 멤버가 아닌 패키지를 둘 때 필요한 조치를 할 수 있다.

## 2. 핵심 개념

### 2.1 워크스페이스가 푸는 문제

코드를 여러 패키지로 나누면 좋은 점이 있습니다. 크레이트가 컴파일의 단위이므로 고친 크레이트와 그것에 의존하는 크레이트만 다시 컴파일됩니다. 크레이트 경계에서는 공개한 것만 보이므로 부분들이 서로의 내부에 기대지 못합니다. 그리고 쪼갠 조각을 다른 프로그램에서 다시 쓸 수 있습니다.

그런데 패키지를 그냥 여러 개 만들면 문제가 생깁니다. 패키지마다 `Cargo.lock`이 따로라서 같은 의존성을 서로 다른 버전으로 쓰게 되고, `target/`이 따로라서 같은 의존성을 패키지 수만큼 컴파일하고, 전체를 빌드하거나 테스트하려면 디렉터리를 돌아다니며 명령을 되풀이해야 합니다.

워크스페이스는 패키지들을 **멤버**(member)로 묶어 이것들을 하나로 만듭니다.

| 함께 쓰는 것 | 뜻 |
| --- | --- |
| `Cargo.lock` | 루트에 하나. 의존성 해석을 워크스페이스 전체에 대해 한 번 하므로, 멤버들의 호환되는 요구 사항은 버전 하나로 합쳐집니다(ch2 2.6절의 규칙 3). |
| `target/` | 루트에 하나. 공통 의존성을 한 번만 컴파일합니다. |
| 명령 | `cargo build --workspace`, `cargo test --workspace`처럼 한 번에 실행합니다. |
| 설정 | 버전, 에디션, 의존성 버전, 린트 수준을 루트에 한 번 적고 멤버가 물려받습니다(2.6절). |

### 2.2 이 저장소가 워크스페이스입니다

저장소 루트의 `Cargo.toml` 전체입니다.

```toml
[workspace]
resolver = "3"
members = ["part*/ch*", "appd-*/ch*"]

[workspace.package]
edition = "2024"
publish = false
```

`[package]`가 없고 `[workspace]`만 있습니다. `members`의 글롭 `part*/ch*`가 모든 파트의 모든 챕터 디렉터리를 멤버로 삼습니다. 그동안 써 온 것들이 이 여섯 줄에서 나왔습니다.

- `cargo run -p p0-ch1-hello-cargo --example …`이 저장소 어디서나 통합니다. `-p`가 워크스페이스의 멤버 가운데 하나를 고르기 때문입니다.
- `Cargo.lock`과 `target/`이 루트에만 있습니다. 챕터 디렉터리에는 생기지 않습니다.
- 챕터마다의 `Cargo.toml`에 있던 `edition.workspace = true`는 "에디션은 루트의 `[workspace.package]`에서 가져오라"는 뜻입니다.

예제 [`01_this_repo_is_a_workspace`](examples/01_this_repo_is_a_workspace.rs)가 두 매니페스트를 읽어 마지막 항목을 보여 줍니다.

```text
== 이 챕터의 Cargo.toml: [package] ==
name = "p3-ch4-workspaces"  (직접 적음)
version = "0.0.0"  (직접 적음)
edition.workspace = true  -> 루트의 edition = "2024"
publish.workspace = true  -> 루트의 publish = false
```

멤버가 수십 개인 이 저장소는 워크스페이스의 규모를 보여 주기에는 좋지만 고쳐 가며 실험하기에는 큽니다. 아래부터는 작은 워크스페이스로 실습합니다.

### 2.3 실습 워크스페이스 `bookshelf`

[`projects/bookshelf`](projects/bookshelf/Cargo.toml)는 책장을 출력하는 프로그램을 패키지 셋으로 나눈 워크스페이스입니다.

```text
bookshelf/
├── Cargo.toml            워크스페이스 루트(가상 매니페스트)
├── Cargo.lock            워크스페이스에 하나
└── crates/
    ├── shelf-core/       라이브러리: Book 타입, 예시 책장
    ├── shelf-report/     라이브러리: 표와 요약 만들기 → shelf-core, unicode-width
    ├── shelf-cli/        바이너리 shelf → shelf-core, shelf-report, unicode-width
    └── shelf-wip/        멤버가 아닌 실험용 패키지(2.4절)
```

```console
$ cd part3-cargo/ch4-workspaces/projects/bookshelf
$ cargo run -q
내 책장 (shelf 0.3.0)
=====================
러스트 입문         420쪽  읽음
The Cargo Book      310쪽  읽는 중
시스템 프로그래밍   560쪽  읽음
3권 가운데 2권을 읽었습니다(980쪽).
```

**워크스페이스의 두 모양.** 워크스페이스는 `Cargo.toml`에 `[workspace]` 표를 적으면 만들어지고, 그 파일이 있는 디렉터리가 워크스페이스 루트(workspace root)입니다. 모양은 둘입니다.

| 모양 | 루트 `Cargo.toml` | 루트에서 `cargo build`의 대상 | 언제 |
| --- | --- | --- | --- |
| 루트 패키지(root package)가 있는 워크스페이스 | `[package]`와 `[workspace]`가 함께 있음 | 루트 패키지 | 중심이 되는 패키지 하나와 그것을 돕는 패키지들 |
| 가상 워크스페이스 | `[workspace]`만 있음. 이런 매니페스트를 가상 매니페스트(virtual manifest)라고 함 | 모든 멤버 | 중심 패키지가 따로 없거나, 패키지를 모두 하위 디렉터리에 가지런히 둘 때 |

`bookshelf`와 이 저장소는 가상 워크스페이스입니다. 루트 패키지가 있는 모양은 이미 써 봤습니다. [ch2](../ch2-dependencies/README.md)의 `three-sources`가 그것입니다. 그 `Cargo.toml`에는 `[package]`와 빈 `[workspace]`가 함께 있고, 디렉터리 안의 path 의존성 `crates/frame`이 저절로 멤버가 됐습니다. 그래서 그곳에서 `cargo test`는 루트 패키지만 테스트했고, `cargo test --workspace`나 `cargo test -p frame`을 줘야 `frame`의 문서 테스트가 실행됐습니다.

가상 매니페스트에는 `[package]`가 없으므로 에디션도 없습니다. Cargo는 에디션을 보고 리졸버 버전을 정하는데([Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md)) 볼 것이 없으니 **`resolver`를 직접 적어야 합니다.** 빼면 경고와 함께 옛 리졸버 1로 돌아갑니다(4.4절).

### 2.4 멤버를 정하고 대상을 고르기

`bookshelf`의 `[workspace]` 표입니다.

```toml
[workspace]
members = ["crates/*"]
exclude = ["crates/shelf-wip"]
resolver = "3"
```

**`members`.** 멤버 패키지의 디렉터리를 적습니다. `crates/*` 같은 글롭을 쓸 수 있습니다. Cargo Book은 멤버를 한 디렉터리(흔히 `crates/`) 아래에 나란히 두고 글롭으로 적기를 권합니다. 새 패키지를 만들 때마다 목록을 고치지 않아도 되기 때문입니다. `members`에 적지 않아도, 워크스페이스 디렉터리 안에 있는 path 의존성은 저절로 멤버가 됩니다.

**`exclude`.** 글롭에 걸리지만 멤버로 삼지 않을 디렉터리를 뺍니다. `crates/shelf-wip`은 아직 워크스페이스에 넣지 않은 실험용 패키지라서 뺐습니다. 이 패키지는 자기 `Cargo.toml`에 빈 `[workspace]` 표를 두어 스스로 워크스페이스 루트가 됩니다. 그 디렉터리에서 `cargo test`를 실행하면 자기만의 `Cargo.lock`과 `target/`을 쓰고, 루트의 `cargo build --workspace`에는 끼지 않습니다.

**워크스페이스 디렉터리 안의 멤버가 아닌 패키지.** Cargo는 패키지의 디렉터리에서 위로 올라가며 `[workspace]`가 있는 `Cargo.toml`을 찾아 "내가 속한 워크스페이스"를 정합니다(자기를 `exclude`한 워크스페이스는 지나쳐 더 위로 올라갑니다). 찾았는데 자기가 그 워크스페이스의 멤버가 아니면 빌드를 거부합니다. `bookshelf`의 사본에 `members`에 걸리지 않는 `sandbox/` 패키지를 만들고 그 안에서 빌드한 결과입니다.

```text
error: current package believes it's in a workspace when it's not:
current:   …/bookshelf/sandbox/Cargo.toml
workspace: …/bookshelf/Cargo.toml

this may be fixable by adding `sandbox` to the `workspace.members` array of the manifest located at: …/bookshelf/Cargo.toml
Alternatively, to keep it out of the workspace, add the package to the `workspace.exclude` array, or add an empty `[workspace]` table to the package's manifest.
```

메시지가 세 가지 해결책을 알려 줍니다. `members`에 넣거나, `exclude`에 넣거나, 그 패키지에 빈 `[workspace]` 표를 두는 것입니다. 이 저장소의 `projects/` 아래 프로젝트마다 `Cargo.toml` 끝에 있던 빈 `[workspace]`가 세 번째 방법입니다. 저장소 루트의 워크스페이스 안에 있지만 멤버가 아닌 패키지라서, 스스로 루트가 되게 한 것입니다([Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md) "자주 하는 실수" 4번의 에러가 이것이었습니다).

**명령의 대상 고르기.** 워크스페이스에서 `cargo build`, `cargo test`, `cargo run` 같은 명령은 다음 순서로 대상을 정합니다.

| 준 것 | 대상 |
| --- | --- |
| `-p 이름` (`--package`) | 그 멤버. 여러 번 쓸 수 있음 |
| `--workspace` | 모든 멤버. `--exclude 이름`으로 일부를 뺄 수 있음 |
| 아무것도 없고, 현재 디렉터리가 멤버 안 | 그 멤버 |
| 아무것도 없고, 현재 디렉터리가 워크스페이스 루트 | `default-members`. 적지 않았으면 루트 패키지, 가상 워크스페이스에서는 모든 멤버 |

`bookshelf` 루트에서 확인한 결과입니다.

| 명령 | 한 일 |
| --- | --- |
| `cargo build` | 세 멤버를 모두 빌드(가상 워크스페이스라서) |
| `cargo run` | 바이너리가 `shelf` 하나뿐이라 그것을 실행 |
| `cargo build -p shelf-core` | `shelf-core`만 컴파일. `target/debug/`에 `libshelf_core.rlib`만 생김 |
| `cargo test -p shelf-report` | `shelf-report`의 테스트만 실행(의존하는 `shelf-core`는 빌드만) |
| `cargo test --workspace --exclude shelf-cli` | 두 라이브러리의 단위 테스트와 문서 테스트 |
| `crates/shelf-report`로 이동해 `cargo test` | `-p shelf-report`와 같음. 산출물은 여전히 루트의 `target/`에 |

마지막 줄이 중요합니다. 멤버 디렉터리에서 명령을 실행해도 그 디렉터리에는 `Cargo.lock`도 `target/`도 생기지 않습니다. Cargo가 위로 올라가 루트를 찾고 루트의 것을 쓰기 때문입니다.

**`default-members`.** 루트에서 옵션 없이 실행했을 때의 대상을 바꿉니다. 사본에 `default-members = ["crates/shelf-core"]`를 넣으면 루트의 `cargo build`가 `shelf-core`만 빌드하고, `cargo run`은 실행할 바이너리가 없다는 에러를 냅니다(``a bin target must be available for `cargo run` ``). 멤버가 많고 그 가운데 일부만 평소에 빌드하면 되는 워크스페이스에서 씁니다.

### 2.5 멤버 간 의존

멤버끼리는 ch2에서 본 path 의존성으로 잇습니다. `shelf-report`가 `shelf-core`의 `Book` 타입을 받아 표를 만들고, `shelf-cli`가 둘을 이어 붙입니다.

```console
$ cargo tree -p shelf-cli
shelf-cli v0.3.0 (…/bookshelf/crates/shelf-cli)
├── shelf-core v0.3.0 (…/bookshelf/crates/shelf-core)
├── shelf-report v0.3.0 (…/bookshelf/crates/shelf-report)
│   ├── shelf-core v0.3.0 (…/bookshelf/crates/shelf-core)
│   └── unicode-width v0.2.3
└── unicode-width v0.2.3
```

같은 워크스페이스에 있다고 해서 다른 멤버를 저절로 쓸 수 있는 것은 아닙니다. 멤버도 각자의 `[dependencies]`에 적은 것만 씁니다. 크레이트 경계도 그대로입니다. `shelf-report`는 `shelf-core`가 `pub`으로 내놓은 것만 볼 수 있습니다.

**잠금 파일은 하나.** 루트의 `Cargo.lock`에 세 멤버와 `unicode-width`가 함께 적혀 있습니다. `unicode-width`는 `shelf-report`와 `shelf-cli` 둘이 쓰지만 항목은 하나입니다. 두 멤버가 같은 요구 사항(`"0.2"`)을 적어 버전 하나로 합쳐졌고, 잠금 파일이 그 버전을 고정합니다. 잠금 파일이 하나라고 해서 버전이 늘 하나인 것은 아닙니다. 사본의 `shelf-cli`만 `unicode-width = "0.1"`로 바꾸면 호환되지 않는 요구 사항이 되어 같은 `Cargo.lock`에 0.1.14와 0.2.3이 함께 적힙니다(ch2 2.6절의 규칙 4). 멤버들이 요구 사항을 한 곳에서 물려받게 하면 이런 일이 생기지 않습니다(2.6절).

**다시 컴파일되는 범위.** 전체를 한 번 빌드한 뒤 `shelf-report`의 소스만 고치고 다시 빌드하면 이렇게 나옵니다(`touch crates/shelf-report/src/lib.rs`로 고친 것으로 쳤습니다).

```text
   Compiling shelf-report v0.3.0 (…/bookshelf/crates/shelf-report)
   Compiling shelf-cli v0.3.0 (…/bookshelf/crates/shelf-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
```

`shelf-core`와 `unicode-width`는 다시 컴파일되지 않았습니다. Cargo는 고친 크레이트와 **그것에 의존하는 크레이트**만 다시 컴파일합니다. 화살표의 맨 아래에 있는 `shelf-core`를 고치면 셋 다 다시 컴파일되고, 맨 위의 `shelf-cli`만 고치면 하나만 다시 컴파일됩니다. 자주 고치는 코드와 거의 고치지 않는 코드를 다른 크레이트에 두면 빌드가 빨라지는 이유입니다.

**순환은 안 됩니다.** `shelf-core`가 `shelf-report`에 의존하게 만들면 서로가 서로를 먼저 컴파일해야 하므로 Cargo가 거부합니다(4.3절). 두 크레이트가 서로를 필요로 하면 함께 쓰는 부분을 세 번째 크레이트로 빼냅니다. `shelf-core`가 바로 그런 자리입니다.

### 2.6 설정 상속

멤버가 셋이면 `version`, `edition`, `rust-version`, 의존성 버전, 린트 표를 세 번씩 적어야 합니다. 그러다 하나만 고치고 둘을 빠뜨립니다. 워크스페이스는 이런 값을 루트에 한 번 적고 멤버가 **물려받게**(inherit) 합니다. 물려받는 표는 셋입니다.

```toml
# bookshelf/Cargo.toml
[workspace.package]
version = "0.3.0"
edition = "2024"
rust-version = "1.85"
publish = false

[workspace.dependencies]
shelf-core = { path = "crates/shelf-core" }
shelf-report = { path = "crates/shelf-report" }
unicode-width = "0.2"

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
```

```toml
# bookshelf/crates/shelf-report/Cargo.toml
[package]
name = "shelf-report"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
shelf-core.workspace = true
unicode-width.workspace = true

[lints]
workspace = true
```

| 루트의 표 | 멤버가 적는 것 | 물려받는 것 |
| --- | --- | --- |
| `[workspace.package]` | `[package]`에 `키.workspace = true` | 그 키의 값. `version`, `edition`, `rust-version`, `description`, `license`, `repository`, `publish` 등 |
| `[workspace.dependencies]` | 의존성 표에 `이름.workspace = true` | 버전 요구 사항과 출처(`path`, `git`) |
| `[workspace.lints]` | `[lints]`에 `workspace = true` | 린트 표 전체 |

`키.workspace = true`는 TOML의 점 표기로 `키 = { workspace = true }`와 같습니다.

**상속은 저절로 일어나지 않습니다.** 루트에 적었다고 멤버에 적용되는 것이 아니라, 멤버가 `workspace = true`라고 적은 키만 물려받습니다. 멤버마다, 키마다 고를 수 있습니다. `shelf-core`의 사본에서 `edition.workspace = true`를 `edition = "2021"`로 바꾸면 그 멤버만 2021 에디션으로 컴파일됩니다. 물려받겠다고 했는데 루트에 값이 없으면 에러입니다(4.2절).

**실제로 무엇이 됐는지 확인하기.** 예제 [`02_workspace_members`](examples/02_workspace_members.rs)가 `cargo metadata`로 Cargo가 풀어 낸 값을 보여 줍니다.

```text
name          version  edition  rust-version  member manifest
shelf-cli     0.3.0    2024     1.85          version.workspace = true
shelf-core    0.3.0    2024     1.85          version.workspace = true
shelf-report  0.3.0    2024     1.85          version.workspace = true

crates/shelf-wip은 멤버인가: 아니요(exclude)
```

사본에서 루트의 `version`을 `"0.4.0"`으로 한 번만 고치고 `cargo run -q`를 실행하면 첫 줄이 `내 책장 (shelf 0.4.0)`으로 바뀌고, `Cargo.lock`의 세 멤버가 모두 0.4.0이 됩니다.

**의존성 상속.** `[workspace.dependencies]`는 ch2에서 미뤄 둔 표입니다. 버전 요구 사항을 한 곳에만 적으므로 멤버들이 서로 다른 요구 사항을 적어 두 버전이 섞이는 일(ch2 2.6절의 규칙 4)을 막습니다. 멤버 간 path 의존성도 여기에 적어 두면 멤버는 경로를 몰라도 됩니다. 멤버는 물려받으면서 `features`·`default-features`·`optional`을 정할 수 있습니다(Part 3 `ch5-features`). `version`이나 `path` 같은 다른 키를 덧붙이면 `unused manifest key` 경고와 함께 무시됩니다.

루트의 표에 적어 놓고 아무 멤버도 물려받지 않는 값은 Cargo가 알려 줍니다. 사본의 루트에 `heck = "0.5"`와 `description = "x"`를 더했을 때의 경고입니다(Cargo 린트 `unused_workspace_dependencies`, `unused_workspace_package_fields`. 둘 다 기본으로 warn).

```text
warning: unused workspace dependency `heck`
warning: unused field `description` in `workspace.package`
```

잠금 파일을 새로 만들 때 상속의 효과가 하나 더 보입니다. 사본에서 `Cargo.lock`을 지우고 `cargo build`를 실행하면 `Updating crates.io index` 다음에 이 줄이 나옵니다.

```text
     Locking 1 package to highest Rust 1.85 compatible version
```

ch2에서는 `Rust 1.100.0 compatible`이었습니다. 멤버들이 `rust-version = "1.85"`를 물려받았으므로 리졸버 3이 1.85에서 쓸 수 있는 버전을 고른 것입니다([ch3](../ch3-manifest-targets/README.md) 2.4절의 효과 2).

**린트 상속.** ch3의 `[lints]` 표를 멤버마다 되풀이하는 대신 `[workspace.lints]`에 적고 멤버는 `[lints] workspace = true` 두 줄만 둡니다. 이 두 줄을 빠뜨리면 그 멤버에는 린트 설정이 적용되지 않습니다. 사본의 `shelf-core`에서 두 줄을 지우고 unsafe 블록을 넣으면 `forbid`인데도 컴파일되고, 대신 Cargo가 경고합니다.

```text
warning: missing `[lints]` to inherit `[workspace.lints]`
 --> crates/shelf-core/Cargo.toml
  = note: `cargo::missing_lints_inheritance` is set to `warn` by default
```

ch3 2.5절의 표에 있던 Cargo 린트 `missing_lints_inheritance`입니다. "루트에 적었으니 적용되겠지"라는 착각이 흔해서 생긴 린트입니다.

**`cargo new`가 상속을 적어 줍니다.** 워크스페이스 안에서 `cargo new --lib crates/shelf-extra`를 실행하면 cargo 1.100.0은 루트의 `[workspace.package]`와 `[workspace.lints]`를 보고 다음 매니페스트를 만듭니다.

```toml
[package]
name = "shelf-extra"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]

[lints]
workspace = true
```

새 경로가 `members`의 글롭 `crates/*`에 이미 걸리므로 루트의 `Cargo.toml`은 바뀌지 않았습니다. 걸리지 않는 경로에 만들면 `cargo new`가 그 경로를 `members`에 덧붙입니다. 사본에서 `cargo new --lib tools/shelf-extra`를 실행하면 `members = ["crates/*", "tools/shelf-extra"]`가 됩니다. 이 저장소 루트에서 `cargo new`를 실행했을 때 루트의 `members`가 조용히 바뀐 것(Part 0 ch1 "자주 하는 실수" 4번의 (가))이 이 동작입니다.

### 2.7 루트에서만 읽는 설정

워크스페이스 전체에 하나만 있어야 하는 설정은 루트의 `Cargo.toml`에서만 읽습니다.

| 설정 | 이유 | 다루는 곳 |
| --- | --- | --- |
| `resolver` | 의존성 해석은 워크스페이스 전체에 대해 한 번 합니다. | 2.3절 |
| `[patch]`, `[replace]` | 출처를 바꾸는 일은 그래프 전체에 미칩니다. | [ch2](../ch2-dependencies/README.md) 2.7절 |
| `[profile.*]` | 최적화 수준 같은 빌드 설정은 한 빌드에 하나입니다. | Part 3 `ch6-profiles-build-performance` |

멤버의 `Cargo.toml`에 적으면 무시하고 경고합니다. 사본의 `shelf-cli`에 `[profile.release]`와 `[patch.crates-io]`를 넣었을 때의 경고입니다.

```text
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   …/bookshelf/crates/shelf-cli/Cargo.toml
workspace: …/bookshelf/Cargo.toml
```

```text
warning: patch for the non root package will be ignored, specify patch at the workspace root:
```

이 저장소의 챕터 패키지에 `[profile]`을 적어도 효과가 없는 것이 이 때문입니다. `[workspace.metadata]`는 `[package.metadata]`처럼 Cargo가 읽지 않고 다른 도구에 맡기는 자리입니다.

## 3. 예제 안내

저장소 루트에서 실행합니다.

| 예제 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_this_repo_is_a_workspace`](examples/01_this_repo_is_a_workspace.rs) | 이 저장소의 루트 매니페스트와 챕터 매니페스트의 상속 | 1, 4 |
| [`02_workspace_members`](examples/02_workspace_members.rs) | `bookshelf`의 멤버와 물려받은 값 | 2, 4 |
| [`projects/bookshelf`](projects/bookshelf/Cargo.toml) | 가상 워크스페이스, 멤버 간 의존, 세 가지 상속, `exclude` | 1~5 |

### 01_this_repo_is_a_workspace — 이 저장소의 두 매니페스트

```sh
cargo run -p p3-ch4-workspaces --example 01_this_repo_is_a_workspace
```

저장소 루트의 `Cargo.toml`과 이 챕터의 `Cargo.toml`을 `include_str!`로 읽어, `키.workspace = true`로 적힌 키가 루트의 어느 값을 가리키는지 짝지어 출력합니다.

```text
== 루트 Cargo.toml: [workspace] ==
resolver = "3"
members = ["part*/ch*", "appd-*/ch*"]

== 루트 Cargo.toml: [workspace.package] ==
edition = "2024"
publish = false

== 이 챕터의 Cargo.toml: [package] ==
name = "p3-ch4-workspaces"  (직접 적음)
version = "0.0.0"  (직접 적음)
edition.workspace = true  -> 루트의 edition = "2024"
publish.workspace = true  -> 루트의 publish = false
```

### 02_workspace_members — Cargo가 본 멤버

```sh
cargo run -p p3-ch4-workspaces --example 02_workspace_members
```

`cargo metadata --no-deps`를 자식 프로세스로 실행해 멤버마다 이름, 버전, 에디션, `rust-version`을 꺼내고, 그 멤버의 `Cargo.toml`에 `version`이 어떻게 적혀 있는지를 나란히 보여 줍니다. 출력은 2.6절에 실었습니다. JSON을 문자열 검색으로 읽는 방식은 ch3의 예제 02와 같습니다.

### projects/bookshelf — 패키지 셋으로 나눈 프로그램

```sh
cd part3-cargo/ch4-workspaces/projects/bookshelf
```

| 디렉터리 | 명령 | 기대 결과 |
| --- | --- | --- |
| `bookshelf` | `cargo run -q` | 2.3절의 여섯 줄 |
| `bookshelf` | `cargo test --workspace` | `shelf-core` 단위 테스트 2개와 문서 테스트 1개, `shelf-report` 단위 테스트 1개 통과 |
| `bookshelf` | `cargo tree -p shelf-cli` | 2.5절의 출력 |
| `bookshelf` | `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` | 통과 |
| `bookshelf/crates/shelf-report` | `cargo test` | 단위 테스트 1개. 이 디렉터리에 `target/`이 생기지 않음 |
| `bookshelf/crates/shelf-wip` | `cargo test` | 테스트 1개 통과. **이 디렉터리에** `target/`이 생김(멤버가 아니라서) |
| `bookshelf` | `cargo build -p shelf-wip` | 실패. ``package ID specification `shelf-wip` did not match any packages`` |

확인이 끝나면 `bookshelf`와 `crates/shelf-wip`에서 각각 `cargo clean`을 실행합니다. `Cargo.toml`을 고치는 실험과 연습 문제는 저장소 밖의 사본에서 합니다. 저장소 루트로 돌아와 복사합니다.

```sh
cp -R part3-cargo/ch4-workspaces/projects/bookshelf /tmp/bookshelf
cd /tmp/bookshelf
```

## 4. 자주 하는 실수

모두 `bookshelf`의 사본에서 재현한 Cargo의 메시지입니다.

### 4.1 워크스페이스 디렉터리 안에 멤버가 아닌 패키지를 둠

2.4절의 `current package believes it's in a workspace when it's not` 에러입니다.

- **원인**: 패키지가 워크스페이스 루트 아래에 있는데 `members`에 걸리지 않습니다. Cargo는 위로 올라가다 만난, 자기를 `exclude`하지 않은 첫 `[workspace]`를 자기 워크스페이스로 여기므로 "멤버도 아닌데 그 안에 있는" 상태를 에러로 봅니다. 이 저장소의 챕터 디렉터리 안에서 `cargo new`로 만든 프로젝트가 빌드되지 않는 것이 이 경우입니다(Part 0 ch1 "자주 하는 실수" 4번의 (나)).
- **해결**: 멤버로 삼으려면 `members`에 넣습니다. 따로 두려면 루트의 `exclude`에 넣거나 그 패키지의 `Cargo.toml`에 빈 `[workspace]`를 둡니다. 다만 `exclude`만으로는 부족할 때가 있습니다. 워크스페이스가 다른 워크스페이스 안에 들어 있으면(이 저장소 안의 `bookshelf`처럼) Cargo가 더 위로 올라가 바깥 워크스페이스를 찾기 때문입니다. `shelf-wip`이 `exclude`와 빈 `[workspace]`를 둘 다 쓰는 이유입니다.

### 4.2 물려받을 값이 루트에 없음

`unicode-width.workspace = true` 옆에 `heck.workspace = true`를 적었지만 루트의 `[workspace.dependencies]`에는 `heck`이 없는 경우입니다.

```text
Caused by:
  error inheriting `heck` from workspace root manifest's `workspace.dependencies.heck`

Caused by:
  `dependency.heck` was not found in `workspace.dependencies`
```

`[workspace.package]`에서 `rust-version`을 지웠는데 멤버가 `rust-version.workspace = true`를 적고 있을 때도 같은 모양입니다.

```text
  error inheriting `rust-version` from workspace root manifest's `workspace.package.rust-version`

Caused by:
  `workspace.package.rust-version` was not defined
```

- **원인**: `workspace = true`는 "루트에 있는 값을 가져오라"는 뜻이지 "있으면 가져오라"가 아닙니다.
- **해결**: 루트의 표에 값을 적거나, 멤버에서 그 줄을 지우고 값을 직접 적습니다. 의존성은 루트의 `[workspace.dependencies]`에 먼저 적은 뒤 멤버에서 물려받습니다(`cargo add`도 워크스페이스에 적힌 의존성이면 `workspace = true`로 넣어 줍니다).

### 4.3 멤버끼리 서로 의존함

`shelf-core`의 `[dependencies]`에 `shelf-report.workspace = true`를 넣은 경우입니다.

```text
error: cyclic package dependency: package `shelf-core v0.3.0 (…/crates/shelf-core)` depends on itself. Cycle:
package `shelf-core v0.3.0 (…/crates/shelf-core)`
    ... which satisfies path dependency `shelf-core` (locked to 0.3.0) of package `shelf-report v0.3.0 (…/crates/shelf-report)`
    ... which satisfies path dependency `shelf-report` (locked to 0.3.0) of package `shelf-core v0.3.0 (…/crates/shelf-core)`
```

- **원인**: 의존하는 크레이트보다 의존받는 크레이트를 먼저 컴파일해야 하는데(ch1 2.1절) 순환하면 먼저 컴파일할 것이 없습니다.
- **해결**: 두 크레이트가 함께 쓰는 타입과 함수를 아래쪽 크레이트로 옮기거나 세 번째 크레이트로 빼냅니다.

### 4.4 그 밖의 에러와 경고

| 한 일 | 메시지 | 원인과 해결 |
| --- | --- | --- |
| 가상 매니페스트에서 `resolver`를 뺌 | ``warning: virtual workspace defaulting to `resolver = "1"` despite one or more workspace members being on edition 2024 which implies `resolver = "3"` `` | 가상 매니페스트에는 에디션이 없어 리졸버를 짐작하지 못합니다. `resolver = "3"`을 적습니다. |
| `exclude` 없이, 자기 `[workspace]`를 가진 패키지가 `members` 글롭에 걸림 | `multiple workspace roots found in the same workspace` | 워크스페이스 안에 워크스페이스를 멤버로 둘 수 없습니다. `exclude`에 넣거나 그 패키지의 `[workspace]`를 지워 멤버로 만듭니다. |
| 멤버 디렉터리를 복사해 이름을 안 바꿈 | ``two packages named `shelf-core` in this workspace`` | 한 워크스페이스의 패키지 이름은 서로 달라야 합니다. |
| 멤버가 아닌 패키지를 `-p`로 가리킴 | ``package ID specification `shelf-wip` did not match any packages`` | `-p`는 워크스페이스의 멤버(와 그 의존성)만 가리킵니다. |
| 멤버에 `[lints] workspace = true`가 없음 | ``warning: missing `[lints]` to inherit `[workspace.lints]` `` | 2.6절. 물려받지 않을 생각이면 빈 `[lints]` 표를 적어 의도를 밝힙니다. |
| 멤버에 `[profile.*]`이나 `[patch]`를 적음 | `profiles for the non root package will be ignored` | 2.7절. 루트의 `Cargo.toml`로 옮깁니다. |
| `default-members`에 라이브러리만 넣고 루트에서 `cargo run` | ``a bin target must be available for `cargo run` `` | `-p shelf-cli`를 주거나 `default-members`에 바이너리 패키지를 넣습니다. |

## 5. 연습 문제

연습 1~3은 3절의 명령으로 만든 사본(`/tmp/bookshelf`)에서 이어서 풉니다. 연습 4는 새 사본에서 시작합니다(명령은 문제에 있습니다).

### 연습 1. 멤버 추가하기

`cargo new --lib crates/shelf-stats`로 멤버를 하나 더 만들고, 책 한 권의 평균 쪽수를 돌려주는 `pub fn average_pages(books: &[Book]) -> Option<u32>`를 쓰세요. `shelf-cli`가 이 함수를 불러 마지막 줄에 `평균 430쪽`을 출력하게 하세요. 새 패키지의 경로는 루트의 `[workspace.dependencies]` 한 곳에만 적어야 합니다.

<details>
<summary>힌트</summary>

고칠 곳은 넷입니다. (1) `crates/shelf-stats/Cargo.toml`의 `[dependencies]`에 `shelf-core.workspace = true`. (2) 루트의 `[workspace.dependencies]`에 `shelf-stats = { path = "crates/shelf-stats" }`. (3) `crates/shelf-cli/Cargo.toml`에 `shelf-stats.workspace = true`. (4) `shelf-cli`의 `main`. `members`가 글롭이라 루트의 `[workspace]` 표는 고치지 않아도 됩니다. `missing_docs`가 켜져 있으므로 새 크레이트와 함수에 문서 주석을 답니다.

</details>

### 연습 2. 다시 컴파일되는 범위 맞히기

전체를 한 번 빌드한 뒤, `touch`로 다음 파일을 하나씩 고친 것으로 치고 `cargo build`의 `Compiling` 줄이 몇 개 나올지 먼저 예측하세요: `crates/shelf-cli/src/main.rs`, `crates/shelf-report/src/lib.rs`, `crates/shelf-core/src/lib.rs`.

<details>
<summary>힌트</summary>

연습 1을 하기 전의 사본이라면 차례로 1개, 2개, 3개입니다. 연습 1을 마친 사본이라면 `shelf-stats`도 `shelf-core`에 의존하므로 마지막이 4개입니다. 고친 크레이트와 그것에 (직접이든 거쳐서든) 의존하는 크레이트만 다시 컴파일됩니다. `unicode-width`는 어느 경우에도 다시 컴파일되지 않습니다. `cargo tree -i shelf-core`로 "누가 이 크레이트에 의존하는가"를 볼 수 있습니다.

</details>

### 연습 3. 실험용 패키지를 멤버로 올리기

`crates/shelf-wip`을 워크스페이스의 정식 멤버로 만드세요. `cargo test --workspace`가 `shelf-wip`의 테스트까지 실행하고 경고가 하나도 없어야 합니다. 버전은 다른 멤버와 같은 값을 물려받게 합니다.

<details>
<summary>힌트</summary>

루트의 `exclude` 줄을 지우고, `shelf-wip/Cargo.toml`에서 빈 `[workspace]` 표를 지웁니다. `exclude`만 지우면 4.4절의 `multiple workspace roots` 에러가 나고, `[workspace]`만 지우면 여전히 멤버가 아닙니다. `shelf-wip`에 남아 있던 `Cargo.lock`은 더는 쓰이지 않으니 지웁니다. 그 상태로 테스트하면 `missing [lints] to inherit` 경고가 나옵니다. `version`·`edition`·`publish`를 `키.workspace = true`로 바꾸고 `rust-version.workspace = true`와 `[lints] workspace = true`를 더합니다.

</details>

### 연습 4. `default-members`의 효과 맞히기

이 문제는 앞 연습의 변경과 빌드 결과가 섞이지 않게 **새 사본**에서 시작합니다. 저장소 루트에서 실행합니다.

```sh
cp -R part3-cargo/ch4-workspaces/projects/bookshelf /tmp/bookshelf-fresh
cd /tmp/bookshelf-fresh
```

아직 아무것도 빌드하지 않은 이 사본의 `[workspace]`에 `default-members = ["crates/shelf-cli"]`를 넣으면, 루트에서 차례로 실행한 `cargo build`, `cargo test`, `cargo run`이 각각 무엇을 대상으로 삼을지 먼저 예측하세요. `cargo build`의 `Compiling` 줄은 몇 개이고, `cargo test`는 테스트를 몇 개 실행할까요? 예측한 뒤 실행해 확인하고, 전체 테스트를 돌리려면 무엇을 붙여야 하는지도 확인하세요.

<details>
<summary>힌트</summary>

대상은 `shelf-cli` 하나입니다. 하지만 `shelf-cli`를 빌드하려면 그것이 의존하는 `shelf-core`, `shelf-report`, `unicode-width`도 컴파일해야 하므로 `cargo build`의 `Compiling` 줄은 넷입니다. 이어서 실행한 `cargo test`는 `shelf-cli`의 테스트만 실행하므로 `Running unittests src/main.rs` 한 줄에 `running 0 tests`입니다. 두 라이브러리는 의존성으로 빌드됐을 뿐이고, 그 테스트는 빌드도 실행도 되지 않습니다. `cargo run`은 그대로 `shelf`를 실행합니다. 전체를 테스트하려면 `cargo test --workspace`를 줍니다. (이미 빌드해 둔 사본에서 풀면 `Compiling` 줄이 나오지 않고, 연습 1을 마친 사본에서 `cargo clean` 뒤에 풀면 `shelf-stats`까지 다섯 줄입니다.)

</details>

## 6. 참고 자료

- The Cargo Book
  - [3.2 Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) — `[workspace]` 표의 키, 세 가지 상속 표, 패키지 고르기
  - [3.16 Lints](https://doc.rust-lang.org/cargo/reference/lints.html) — `missing_lints_inheritance`
  - [3.3 Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace) — 워크스페이스에서 의존성 물려받기
- 이 저장소의 관련 챕터
  - [Part 0 ch1](../../part0-getting-started/ch1-hello-cargo/README.md) "자주 하는 실수" 4번 — 이 저장소 안에서 `cargo new`를 실행했을 때
  - [ch2](../ch2-dependencies/README.md) — path 의존성, `[patch]`, 루트 패키지가 있는 워크스페이스 `three-sources`
  - [ch3](../ch3-manifest-targets/README.md) — `rust-version`, `[lints]`, 타깃 표(`shelf-cli`의 `[[bin]]`)
  - [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) — 리졸버 버전과 가상 매니페스트
