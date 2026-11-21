# ch10. 게시와 SemVer

지금까지 이 저장소의 패키지는 모두 내 컴퓨터 안에서만 썼습니다. 이 챕터는 패키지를 crates.io 같은 레지스트리(registry)에 **게시**(publish)해 다른 사람이 의존성으로 쓰게 하는 과정과, 게시한 뒤 새 버전을 낼 때 버전 번호를 어떻게 올려야 사용자의 빌드를 깨지 않는지(SemVer 호환성)를 다룹니다. 게시한 프로그램을 `cargo install`로 설치하는 쪽도 함께 봅니다.

- 패키지: `p3-ch10-publishing-semver`
- 기준 문서: The Cargo Book [2.9 Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html), [3.12 Registries](https://doc.rust-lang.org/cargo/reference/registries.html), [3.13 SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html). 명령은 [cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html), [cargo publish](https://doc.rust-lang.org/cargo/commands/cargo-publish.html), [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html) 문서를 함께 봤습니다.
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64)
- 실습 자료: `examples/` 2개, `compile_fail/` 1개, 독립 프로젝트 [`projects/tally-units`](projects/tally-units/Cargo.toml)(게시 정보를 갖춘 두 패키지의 워크스페이스). 외부 크레이트는 쓰지 않습니다.
- **실제로 게시하지는 않았습니다.** 게시는 되돌릴 수 없으므로 `cargo package`와 `cargo publish --dry-run`(올리기 직전에 멈춤)까지만 실행했습니다. `cargo login`, `cargo yank`, `cargo owner`, 다른 레지스트리로의 게시도 실행하지 않고 설명만 합니다. `cargo publish --dry-run`은 crates.io의 색인을 읽으므로 네트워크가 필요합니다.
- `cargo install`은 Cargo home을 바꾸지 않도록 `--root`로 임시 디렉터리에 설치해 확인했습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. crates.io에 게시하는 절차(토큰, 메타데이터, 확인, 올리기)를 설명하고, 게시가 되돌릴 수 없다는 것과 게시 취소(yank)의 뜻을 말할 수 있다.
2. `cargo package`로 게시될 `.crate` 파일의 내용(정규화된 `Cargo.toml`, 포함·제외할 파일)을 확인하고, 게시가 거절되는 경우를 고칠 수 있다.
3. path·git 의존성이 있는 패키지와 워크스페이스의 여러 패키지를 게시하는 규칙을 설명하고, `cargo publish --workspace --dry-run`으로 확인할 수 있다.
4. Cargo Book의 SemVer 분류(메이저·마이너·깨질 수도 있음)로 변경을 판정하고, 구조체 필드·고유 메서드(inherent method)·기능·의존성에 관한 규칙을 예로 설명할 수 있다.
5. `cargo install`이 무엇을 어디에 설치하고 어떻게 기록하는지 설명할 수 있다.

## 2. 핵심 개념

### 2.1 게시는 되돌릴 수 없다

게시는 패키지의 특정 버전을 압축한 `.crate` 파일을 레지스트리에 올리는 일입니다. [ch1](../ch1-getting-started/README.md) 2.6절에서 Cargo home에 내려받혀 쌓이던 `.crate` 파일이 그것입니다. crates.io는 **영구 보관소**를 목표로 하므로, 한 번 올린 버전은 덮어쓸 수 없고 코드도 대체로 지울 수 없습니다(Cargo Book의 표현은 "generally permanent"이고, 게시 직후 등 좁은 조건에서만 삭제를 허용하는 crates.io 정책이 따로 있습니다). 고친 내용은 새 버전으로 냅니다. 버전 수에는 제한이 없습니다.

그래서 이 챕터의 실습은 실제로 올리지 않고, 올리기 직전까지 같은 검사를 하는 `cargo package`와 `cargo publish --dry-run`으로 합니다. 이 저장소의 모든 챕터 패키지는 루트 매니페스트의 `[workspace.package]`에서 `publish = false`를 물려받으므로, 실수로 게시하려 해도 Cargo가 먼저 거절합니다.

```console
$ cargo publish --dry-run -p p3-ch10-publishing-semver
error: `p3-ch10-publishing-semver` cannot be published.
`package.publish` must be set to `true` or a non-empty list in Cargo.toml to publish.
```

### 2.2 게시하기 전에

**계정과 토큰.** crates.io에 GitHub 계정으로 로그인하고 이메일을 확인한 뒤 API 토큰(API token)을 만듭니다. `cargo login`을 실행하고 토큰을 붙여 넣으면 Cargo home의 `credentials.toml`에 저장됩니다([ch7](../ch7-configuration-env/README.md) 2.5절). 토큰은 비밀번호와 같으므로 다른 사람과 나누지 않고, 새어 나가면 바로 폐기합니다. `cargo logout`은 저장된 토큰을 지웁니다. 설정 파일 대신 환경 변수 `CARGO_REGISTRY_TOKEN`으로 줄 수도 있습니다(CI에서 비밀 값으로 넘길 때).

**이름은 먼저 쓰는 사람 것입니다.** crates.io의 크레이트 이름은 선착순이고, 한 번 쓰인 이름은 다른 크레이트가 쓸 수 없습니다. `projects/tally-units`의 두 이름은 이 문서를 쓴 날(2026-11-21) `cargo info`(레지스트리에 있는 패키지의 정보를 보여 주는 명령)로 crates.io에 없는 것을 확인했습니다.

**메타데이터.** Cargo Book은 게시하기 전에 다음 키를 채우라고 합니다([ch3](../ch3-manifest-targets/README.md) 2.2절 표의 "게시할 때 쓰는 정보").

| 키 | 뜻 |
| --- | --- |
| `license` 또는 `license-file` | 라이선스(SPDX 식, 예: `"MIT OR Apache-2.0"`) 또는 라이선스 파일의 경로 |
| `description` | 한두 문장의 설명. crates.io 검색 결과에 나옴 |
| `homepage`, `repository` | 홈페이지, 소스 저장소의 URL |
| `readme` | README 파일의 경로. 적지 않아도 `README.md` 등이 있으면 그것을 씀([ch7](../ch7-configuration-env/README.md) 2.7절) |
| `keywords`, `categories` | 검색어(최대 5개)와 crates.io의 분류. 필수는 아님 |

`description`과 `license`(또는 `license-file`)가 없으면 `cargo package`가 ``warning: manifest has no description, license or license-file``처럼 경고합니다. crates.io는 이 둘이 없는 패키지를 받지 않습니다.

### 2.3 `cargo package`: 게시될 파일 미리 보기

`cargo publish`는 다음을 차례로 합니다.

1. 패키지에 몇 가지 검사를 합니다.
2. 소스를 `.crate` 파일로 압축합니다.
3. 그 `.crate`를 임시 디렉터리에 풀어 **컴파일되는지** 확인합니다.
4. 레지스트리에 올립니다.
5. 레지스트리가 추가 검사를 한 뒤 받아들입니다.

`cargo package`와 `cargo publish --dry-run`은 3단계까지 합니다. 만든 `.crate`는 `target/package/`에 남습니다. crates.io는 `.crate` 하나를 10 MB까지 받으므로, 큰 테스트 데이터나 문서용 그림이 실수로 들어가지 않았는지 `--list`로 확인합니다.

```console
$ cargo package --list --allow-dirty -p tally-units-core
.cargo_vcs_info.json
Cargo.lock
Cargo.toml
Cargo.toml.orig
README.md
src/lib.rs
```

- **버전 관리 시스템이 무시하는 파일은 자동으로 빠집니다**(`.gitignore`의 `target/` 등).
- **`exclude`와 `include`.** 그 밖에 뺄 파일은 `[package]`의 `exclude`에 글롭으로 적습니다. 반대로 넣을 파일만 나열하고 싶으면 `include`를 쓰며, `include`가 있으면 `exclude`는 무시됩니다. `tally-units-core`는 `exclude = ["tests/"]`로 테스트와 큰 테스트 데이터를 뺐습니다. 그래서 목록에 `tests/`가 없고, 패키징할 때 ``warning: ignoring test `data_file` as `tests/data_file.rs` is not included in the published package``가 나옵니다. 파일이 빠진 테스트 타깃을 정규화된 매니페스트에서도 지운다는 뜻입니다. 테스트를 게시에 넣을지 뺄지는 [부록 A ch30](../../appd-a/ch30-beyond-unit-tests/README.md) 2.9절도 다룹니다.
- **`Cargo.lock`.** 라이브러리라도 `Cargo.lock`이 들어갑니다. 라이브러리를 의존성으로 쓰는 쪽은 이 파일을 쓰지 않지만, 바이너리를 `cargo install`할 때는 이 파일을 씁니다(2.8절).
- **`Cargo.toml`과 `Cargo.toml.orig`.** 올라가는 `Cargo.toml`은 Cargo가 다시 쓴 정규화(normalize)된 판입니다. 워크스페이스에서 물려받은 값(`version.workspace = true` 등)을 실제 값으로 채우고, `[workspace]`·`[patch]`를 지우고, path 의존성의 `path`를 지우고, 자동 탐색한 타깃을 명시합니다. 내가 쓴 원본은 `Cargo.toml.orig`로 함께 들어갑니다. `tar xzf target/package/tally-units-core-0.1.0.crate -O tally-units-core-0.1.0/Cargo.toml`로 보면 맨 위에 `# THIS FILE IS AUTOMATICALLY GENERATED BY CARGO`가 있습니다.
- **`.cargo_vcs_info.json`.** git 저장소 안에서 패키징하면 그 커밋의 해시와 패키지의 경로가 기록됩니다.
- **커밋하지 않은 변경.** 버전 관리되는 파일 가운데 커밋하지 않은 변경이 있으면 `cargo package`는 ``error: 3 files in the working directory contain changes that were not yet committed into git:``처럼 거절합니다. 올라간 코드가 어느 커밋인지 기록과 맞아야 하기 때문입니다. 이 문서의 명령에 `--allow-dirty`를 붙인 것은 저장소에 커밋하기 전에도, 연습으로 고친 사본에서도 실행할 수 있게 하려는 것입니다. 이때 `.cargo_vcs_info.json`에는 `"dirty": true`가 기록됩니다.
- **확인 빌드는 테스트를 돌리지 않습니다.** 3단계는 풀어 놓은 `.crate`가 컴파일되는지만 봅니다. `--no-verify`를 주면 이 단계도 건너뜁니다. 이 확인 빌드에서 빌드 스크립트가 패키지의 소스를 고치면 ``Source directory was modified by build.rs during cargo publish``로 거절됩니다([ch8](../ch8-build-scripts/README.md) 4절의 "`OUT_DIR` 밖에 쓰지 않는다").

### 2.4 path·git 의존성과 워크스페이스 게시

[ch2](../ch2-dependencies/README.md) 2.3절에서 의존성의 출처를 crates.io·git·path 셋으로 나눴습니다. crates.io에 올라간 패키지는 다른 사람의 컴퓨터에서 빌드되므로 **내 디스크의 경로나 git 저장소에 기댈 수 없습니다.** 그래서 게시할 패키지의 path·git 의존성에는 `version`도 적어야 하고, 게시할 때 `path`·`git`이 지워지고 `version`만 남아 그 버전을 crates.io에서 받게 됩니다.

```toml
# tally-units/Cargo.toml
[dependencies]
tally-units-core = { path = "../tally-units-core", version = "0.1.0" }
```

`version`을 빼고 `cargo publish --dry-run -p tally-units`를 실행하면 다음처럼 거절됩니다. git 의존성이면 마지막 줄이 ``the `git` specification will be removed``가 되고, `cargo package`로 실행하면 `publishing`·`published`가 `packaging`·`packaged`로 바뀝니다.

```text
error: failed to verify manifest at `…/tally-units/Cargo.toml`

Caused by:
  all dependencies must have a version requirement specified when publishing.
  dependency `tally-units-core` does not specify a version
  Note: The published dependency will use the version from crates.io,
  the `path` specification will be removed from the dependency declaration.
```

**의존하는 패키지보다 의존받는 패키지를 먼저 올립니다.** `tally-units`만 따로 패키징하면, 정규화된 매니페스트가 crates.io의 `tally-units-core`를 찾으려 하므로 실패합니다(아직 게시되지 않았으므로).

```text
error: failed to prepare local package for uploading

Caused by:
  no matching package named `tally-units-core` found
  location searched: crates.io index
  required by package `tally-units v0.1.0 (…/tally-units/tally-units)`
```

`cargo publish --workspace`(또는 `-p`를 여럿)는 워크스페이스의 게시할 수 있는 패키지를 의존 순서대로 묶어 처리합니다. 확인 빌드에서는 먼저 패키징한 `tally-units-core`를 `target/package/tmp-registry`라는 임시 레지스트리에 두고 `tally-units`가 그것을 쓰게 합니다. `publish = false`인 멤버는 건너뜁니다.

```console
$ cargo publish --dry-run --workspace --allow-dirty --locked
    Updating crates.io index
   Packaging tally-units-core v0.1.0 (…/tally-units/tally-units-core)
warning: ignoring test `data_file` as `tests/data_file.rs` is not included in the published package
    Packaged 6 files, 3.2KiB (1.7KiB compressed)
   Packaging tally-units v0.1.0 (…/tally-units/tally-units)
    Updating crates.io index
    Packaged 5 files, 2.7KiB (1.6KiB compressed)
   Verifying tally-units-core v0.1.0 (…/tally-units/tally-units-core)
   Compiling tally-units-core v0.1.0 (…/tally-units/target/package/tally-units-core-0.1.0)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in …s
   Verifying tally-units v0.1.0 (…/tally-units/tally-units)
    Updating crates.io index
   Unpacking tally-units-core v0.1.0 (registry `…/tally-units/target/package/tmp-registry`)
   Compiling tally-units-core v0.1.0
   Compiling tally-units v0.1.0 (…/tally-units/target/package/tally-units-0.1.0)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in …s
   Uploading tally-units-core v0.1.0 (…/tally-units/tally-units-core)
warning: aborting upload due to dry run
   Uploading tally-units v0.1.0 (…/tally-units/tally-units)
warning: aborting upload due to dry run
```

`Unpacking …` 줄은 그 경로에서 처음 실행할 때만 나옵니다. dry-run이 임시 레지스트리의 `.crate`를 Cargo home의 `registry/src/` 아래에 풀어 두고 다음부터는 그것을 쓰기 때문입니다. 이 풀린 디렉터리는 `cargo clean`으로 지워지지 않고 Cargo home에 남습니다.

두 패키지가 같은 `version`·`license`·`repository`를 `[workspace.package]`에서 물려받는 것은 [ch4](../ch4-workspaces/README.md) 2.6절의 설정 상속입니다.

### 2.5 게시한 뒤: 새 버전, 게시 취소, 소유자

**새 버전.** `Cargo.toml`의 `version`을 올리고 다시 `cargo publish`합니다. 얼마나 올릴지는 2.7절의 SemVer 규칙이 정합니다. Cargo Book은 버전마다 변경 기록(changelog)과 게시한 커밋을 가리키는 git 태그를 남기고, 릴리스 과정을 자동화하라고 권하며, 그런 도구로 `cargo-release`, `cargo-smart-release`, `release-plz`를 듭니다.

**게시 취소(yank).** 망가진 버전을 올렸을 때 `cargo yank --version 1.0.1`로 그 버전을 게시 취소할 수 있습니다(`--undo`로 되돌림). 게시 취소는 코드를 지우지 않습니다. 이미 그 버전을 `Cargo.lock`에 잠근 프로젝트는 계속 빌드되고, **새로** 그 버전을 고르는 것만 막습니다. 그래서 실수로 올린 비밀 값을 지우는 수단이 아닙니다. 그런 일이 생기면 비밀 값 자체를 바로 바꿔야 합니다. 리졸버가 게시 취소된 버전을 새로 고르지 않는 실제 예는 [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.6절에 있습니다(`smol_str` 0.3.3).

**소유자.** 크레이트의 새 버전을 게시할 수 있는 사람은 소유자(owner)뿐입니다. `cargo owner --add <GitHub 사용자>`는 소유자를 더하는 권한까지 가진 "이름 있는 소유자(named owner)"를, `cargo owner --add github:<조직>:<팀>`은 게시와 게시 취소만 할 수 있는 "팀 소유자(team owner)"를 초대합니다.

### 2.6 다른 레지스트리

회사 안에서만 쓰는 패키지처럼 crates.io가 아닌 레지스트리에 둘 수도 있습니다.

- **레지스트리 정의.** Cargo 설정 파일(ch7)의 `[registries]` 표에 이름과 색인 URL을 적습니다. `my-registry = { index = "https://my-intranet:8080/git/index" }`. 환경 변수로는 `CARGO_REGISTRIES_MY_REGISTRY_INDEX`입니다. 색인 URL이 `sparse+`로 시작하면 필요한 크레이트의 정보만 HTTP로 받는 sparse 프로토콜을, 아니면 색인 전체를 git으로 받는 git 프로토콜을 씁니다.
- **그 레지스트리의 의존성.** 의존성 선언에 `registry = "my-registry"`를 적습니다. crates.io는 다른 레지스트리의 크레이트에 의존하는 패키지를 받지 않습니다.
- **그 레지스트리에 게시.** `cargo login --registry=my-registry`로 토큰을 저장하고 `cargo publish --registry=my-registry`로 올립니다. 토큰은 `credentials.toml`의 `[registries.my-registry]` 표나 환경 변수 `CARGO_REGISTRIES_MY_REGISTRY_TOKEN`에 둡니다. 매번 `--registry`를 주지 않으려면 설정의 `registry.default`를 정합니다.
- **`publish` 키로 막기.** 매니페스트의 `publish = ["my-registry"]`는 그 레지스트리에만 게시할 수 있게 합니다. 사내 코드를 실수로 crates.io에 올리는 것을 막습니다. `publish = false`는 빈 목록과 같아 어디에도 게시할 수 없습니다(2.1절).

### 2.7 SemVer 호환성: 버전 번호로 하는 약속

[ch2](../ch2-dependencies/README.md) 2.2절에서 `"1.2.3"`이라는 요구 사항이 `>=1.2.3, <2.0.0`이라는 것을 봤습니다. 사용자가 `cargo update`로 1.2.3에서 1.9.0까지 말없이 올라가도 빌드가 깨지지 않으려면, 라이브러리 작성자가 **호환되지 않는 변경을 할 때 첫째 자리(메이저)를 올린다**는 약속을 지켜야 합니다. Cargo Book 3.13절은 어떤 변경이 호환되지 않는지를 규칙으로 정리합니다. 규칙마다 변경의 수준을 셋 가운데 하나로 나눕니다.

| 분류 | 뜻 | 1.0.0 이상에서 | 0.y.z에서 |
| --- | --- | --- | --- |
| 메이저(major) | 사용자의 빌드를 깨뜨릴 수 있는 변경 | 첫째 자리를 올림 | 둘째 자리(y)를 올림 |
| 마이너(minor) | 호환되는 변경 | 둘째 자리를 올림 | 셋째 자리(z)를 올림 |
| 깨질 수도 있음(possibly-breaking) | 사용자의 코드에 따라 깨질 수 있음 | 프로젝트가 판단 | 프로젝트가 판단 |

0.y.z에서 자리가 하나씩 밀리는 것은 Cargo가 `0.1.3`을 `>=0.1.3, <0.2.0`으로 해석하기 때문입니다. `0.0.z`는 모든 변경이 메이저입니다. 다음은 Cargo Book의 규칙 가운데 일부입니다.

| 변경 | 분류 | 깨지는 사용자 코드 |
| --- | --- | --- |
| 공개 항목의 이름을 바꾸거나 옮기거나 지움 | 메이저 | 그 항목을 쓰는 코드 |
| 새 공개 항목을 더함 | 마이너 | (글롭 가져오기와 이름이 겹치는 드문 경우뿐) |
| 필드가 모두 공개인 구조체에 비공개 필드를 더함 | 메이저 | 구조체 리터럴, `..` 없는 구조 분해(예제 01, `compile_fail`) |
| 비공개 필드가 없는 구조체에 공개 필드를 더함 | 메이저 | 같은 두 가지(E0063, E0027) |
| 비공개 필드가 이미 하나 이상 있는 구조체에 비공개 필드를 더하거나 뺌 | 마이너 | 없음(예제 01의 `closed_v1_1`) |
| `#[non_exhaustive]` 없는 열거형에 배리언트를 더함 | 메이저 | 모든 배리언트를 나열한 `match` |
| 트레이트에 기본 구현이 없는 항목을 더함 / 기본 구현이 있는 항목을 더함 | 메이저 / 깨질 수도 있음 | 그 트레이트를 구현한 사용자 코드 / 같은 이름의 메서드가 둘이 되어 모호해지는 코드 |
| 이미 있는 열거형·배리언트, 비공개 필드가 없는 구조체에 `#[non_exhaustive]`를 붙임 | 메이저 | 크레이트 밖의 `match`·리터럴·`..` 없는 구조 분해(비공개 필드가 이미 있는 구조체라면 해당 없음) |
| 고유 메서드 등 고유 항목(inherent item)을 더함 | 깨질 수도 있음 | 같은 이름의 트레이트 메서드를 부르던 코드(예제 02) |
| 제네릭의 바운드를 조임 / 느슨하게 함 | 메이저 / 마이너 | 조이면 새 바운드를 만족하지 않는 타입을 넘기던 코드 |
| 함수의 매개변수를 더하거나 뺌 | 메이저 | 그 함수를 부르는 코드 |
| 필요한 최소 Rust 버전을 올림 | 깨질 수도 있음 | 옛 툴체인을 쓰는 사용자 |
| 새 린트를 더함(`#[deprecated]` 등) | 마이너 | 경고를 에러로 다루는 사용자(Cargo Book은 마이너로 봄) |
| Cargo 기능을 더함 | 마이너 | 대개 없음 |
| Cargo 기능을 지움 | 메이저 | 그 기능을 켜던 의존성 선언 |
| 기본 기능 등 기능 목록에서 기능을 뺌(기능이나 공개 항목이 바뀌면) | 메이저 | 그 기능에 기대던 코드 |
| 선택적 의존성을 지움 | 깨질 수도 있음 | 그 암묵적 기능을 켜던 선언([ch5](../ch5-features/README.md) 2.4절의 `dep:`로 숨겼다면 해당 없음) |
| 의존성의 기능을 바꾸거나 의존성을 더함 | 마이너 | 대개 없음(stable에서 되던 프로젝트에 nightly가 필요한 의존성을 더하는 등은 메이저라고 Cargo Book은 적음) |

예제 01과 02는 두 가지 규칙을 보여 줍니다.

- **구조체 필드.** 필드가 모두 공개인 구조체는 사용자가 리터럴로 만들고 모든 필드를 나열해 구조 분해할 수 있습니다. 그러면 필드를 하나만 더해도(공개든 비공개든) 그 코드가 깨집니다. 처음부터 비공개 필드를 하나 두고 생성자를 주면, 사용자는 생성자로만 만들고 구조 분해에 `..`를 써야 하므로 나중에 필드를 더해도 호환됩니다. 같은 목적으로 쓰는 것이 `#[non_exhaustive]` 속성입니다. 열거형에 붙이면 크레이트 밖의 `match`에 `_` 갈래를 요구하므로 나중에 배리언트를 더해도 마이너가 되고, 구조체에 붙이면 크레이트 밖에서 리터럴로 만들 수 없게 됩니다. `projects/tally-units`의 `Unit`이 이 속성을 붙였습니다. 자세한 쓰임은 [부록 A ch21](../../appd-a/ch21-semver/README.md) 2.5절이, 속성 자체는 Part 9 `ch4-attributes`가 다룹니다.
- **고유 메서드.** 메서드를 부를 때 Rust는 같은 탐색 단계에서 고유 메서드를 트레이트 메서드보다 먼저 찾습니다([Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.2절). 그래서 라이브러리가 사용자의 확장 트레이트(extension trait, 남의 타입에 메서드를 더하려고 사용자가 만든 트레이트)와 같은 이름의 고유 메서드를 더하면, 사용자 코드가 한 글자도 바뀌지 않았는데 다른 메서드가 불릴 수 있습니다. 인자 개수처럼 시그니처가 맞지 않으면 컴파일 에러가 날 수 있고, 시그니처가 같거나 반환 타입만 달라 부르는 코드가 그대로 컴파일되면 예제 02처럼 **조용히 동작이 바뀝니다**. 수신자의 모양이 다르면 아예 가리지 않기도 합니다. 고유 메서드가 `&mut self`를, 트레이트 메서드가 `&self`를 받으면, 값(`P`)에 대고 부를 때는 탐색이 `&P` 단계를 `&mut P` 단계보다 먼저 보므로 트레이트 메서드가 불리고, `&mut P`에 대고 부를 때는 고유 메서드가 불립니다. 결과가 사용자 코드에 달려 있어 Cargo Book이 "깨질 수도 있음"으로 둡니다. 사용자 쪽에서는 `Describe::describe(&p)`처럼 트레이트 이름을 경로에 적어 부르면 피할 수 있습니다.

이 규칙들은 "사용자의 코드가 다시 컴파일되는가"에 관한 것입니다. 컴파일러가 잡지 못하는 **동작**의 변경(같은 함수가 다른 값을 돌려줌)도 호환되지 않는 변경이고, 이것을 어떻게 다루고 사용자에게 알릴지(더는 권장하지 않음 표시 뒤 다음 메이저에서 지우기 등)는 부록 A ch21이 지침으로 다룹니다. 두 버전의 공개 API를 비교해 SemVer 위반을 찾는 도구 `cargo-semver-checks`도 거기서 소개합니다(이 저장소에는 설치하지 않았습니다).

### 2.8 `cargo install`: 게시된 프로그램 설치하기

`cargo install 크레이트`는 crates.io에서 바이너리 크레이트를 받아 **`release` 프로필로** 빌드하고([ch6](../ch6-profiles-build-performance/README.md) 2.1절), 실행 파일을 Cargo home의 `bin/`에 넣습니다([ch1](../ch1-getting-started/README.md) 2.6절). 크레이트 이름 대신 `--path 디렉터리`나 `--git URL`로 받을 곳을 정할 수도 있고, `--root 디렉터리`로 설치할 곳을 바꿀 수 있습니다.

- **설치 기록.** 설치할 곳의 `.crates.toml`과 `.crates2.json`에 무엇을 어디서 받아 설치했는지 적습니다. `cargo install --list`가 이 기록을 보여 주고, `cargo uninstall`이 이 기록을 보고 실행 파일을 지웁니다.
- **잠금 파일.** 1.100.0의 `cargo install`은 패키지에 든 `Cargo.lock`을 기본으로 씁니다(`cargo install` 문서의 "Dealing with the Lockfile"). 게시할 때 고른 의존성 버전 그대로 빌드되는 것입니다. 1.98.1의 같은 문서는 잠금 파일을 무시한다고 적으므로 최근 버전에서 바뀐 동작입니다. [ch9](../ch9-testing-ci/README.md) 2.6절에서 본 Cargo Book 2.8절의 "`cargo install` 사용자를 위해 최신 의존성을 확인하라"는 문장이 이 동작과 맞지 않는 이유입니다.
- **다시 설치.** 같은 경로에서 다시 설치하면 실행 파일을 바꿔 넣습니다(`Replacing …`). `--path` 설치는 그 프로젝트의 `target/`에 빌드하므로 빌드가 최신이면 다시 컴파일하지 않습니다.

`projects/tally-units`의 바이너리를 임시 디렉터리에 설치하고 지운 결과입니다(경로는 줄였습니다).

```console
$ cargo install --path tally-units --root …/inst
  Installing tally-units v0.1.0 (…/tally-units/tally-units)
   Compiling tally-units-core v0.1.0 (…/tally-units/tally-units-core)
   Compiling tally-units v0.1.0 (…/tally-units/tally-units)
    Finished `release` profile [optimized] target(s) in …s
  Installing …/inst/bin/tally-units
   Installed package `tally-units v0.1.0 (…/tally-units/tally-units)` (executable `tally-units`)
warning: be sure to add `…/inst/bin` to your PATH to be able to run the installed binaries
$ find …/inst -type f
…/inst/.crates.toml
…/inst/.crates2.json
…/inst/bin/tally-units
$ …/inst/bin/tally-units 3 m cm
300
$ cargo uninstall --root …/inst tally-units
    Removing …/inst/bin/tally-units
```

이 저장소의 셸에는 `RUSTUP_TOOLCHAIN`이 잡혀 있어 `Installing` 줄 다음에 ``warning: default toolchain implicitly overridden with `1.100.0-aarch64-apple-darwin` by environment variable``이 더 나왔습니다. 보통의 환경에서는 나오지 않으므로 위에서 뺐습니다.

## 3. 예제 안내

예제는 저장소 루트에서 실행합니다.

### 01_struct_fields — 필드를 더해도 되는 구조체와 안 되는 구조체

[`examples/01_struct_fields.rs`](examples/01_struct_fields.rs)는 가상의 라이브러리 `size`의 버전들을 모듈로 흉내 냅니다(2.7절). 필드가 모두 공개인 `open_v1_0`과, 처음부터 비공개 필드와 생성자를 둔 `closed_v1_0`·`closed_v1_1`을 같은 방식으로 씁니다.

```console
$ cargo run -q -p p3-ch10-publishing-semver --example 01_struct_fields
open 1.0.0  : 3 x 4
closed 1.0.0: 3 x 4
closed 1.1.0: 3 x 4, 넓이 12
```

`closed_v1_1`은 `closed_v1_0`에 비공개 필드 `cached_area`를 더했지만, 사용자 코드(`Size::new(3, 4)`, `..`를 쓴 구조 분해)는 그대로 컴파일됩니다. `open_v1_0`에 같은 필드를 더한 판은 `compile_fail/new_private_field.rs`(4.1절)입니다.

### 02_inherent_shadows_trait — 새 고유 메서드가 사용자의 메서드를 가림

[`examples/02_inherent_shadows_trait.rs`](examples/02_inherent_shadows_trait.rs)의 사용자 코드는 직접 만든 트레이트 `Describe`의 `describe`를 부릅니다. 라이브러리 1.1.0이 같은 이름의 고유 메서드를 더했습니다.

```console
$ cargo run -q -p p3-ch10-publishing-semver --example 02_inherent_shadows_trait
1.0.0: 1.5 kg 소포
1.1.0: parcel(1500g)
1.1.0: 1.5 kg 소포 (Describe::describe로 부름)
```

둘째 줄은 컴파일 에러도 경고도 없이 라이브러리의 메서드가 불린 결과입니다. 이 예제는 한 크레이트 안의 모듈로 흉내 냈지만, 실제로 크레이트 버전이 바뀔 때도 메서드 탐색 규칙은 같습니다.

### projects/tally-units — 게시 정보를 갖춘 워크스페이스

길이 단위를 바꾸는 라이브러리 `tally-units-core`와 그것을 쓰는 명령줄 도구 `tally-units`의 가상 워크스페이스입니다.

| 파일 | 담은 것 |
| --- | --- |
| [`Cargo.toml`](projects/tally-units/Cargo.toml) | 가상 매니페스트, `[workspace.package]`의 `version`·`edition`·`rust-version`·`license`·`repository` |
| [`tally-units-core/Cargo.toml`](projects/tally-units/tally-units-core/Cargo.toml) | `description`·`readme`·`keywords`·`categories`, `exclude = ["tests/"]` |
| [`tally-units-core/src/lib.rs`](projects/tally-units/tally-units-core/src/lib.rs) | `Unit`(`#[non_exhaustive]`), `convert`, 문서 테스트 |
| [`tally-units-core/tests/`](projects/tally-units/tally-units-core/tests/data_file.rs) | 데이터 파일(`tests/data/many.txt`)을 읽는 통합 테스트. 게시하는 `.crate`에서는 빠짐 |
| [`tally-units/Cargo.toml`](projects/tally-units/tally-units/Cargo.toml), [`src/main.rs`](projects/tally-units/tally-units/src/main.rs) | `path`와 `version`을 함께 적은 의존성, 명령줄 도구 |

프로젝트 디렉터리에서 실행합니다. `cargo publish --dry-run`과 `cargo package -p tally-units`(바이너리 단독)는 crates.io 색인을 읽으므로 네트워크가 필요합니다. 라이브러리 단독인 `cargo package -p tally-units-core`는 의존성이 없어 색인을 읽지 않습니다. `--allow-dirty`는 2.3절에서 설명한 대로 커밋하지 않은 상태에서도 실행하려고 붙입니다.

| 명령 | 기대 결과 |
| --- | --- |
| `cargo test --locked` | 문서 테스트 2개와 통합 테스트 1개 통과 |
| `cargo run -q --locked -p tally-units -- 2.5 km m` | `2500` |
| `cargo run -q --locked -p tally-units -- 1 mi m` | 표준 에러에 `값은 수, 단위는 mm·cm·m·km 가운데 하나여야 합니다`, 종료 상태 2 |
| `cargo package --list --allow-dirty -p tally-units-core` | 2.3절의 목록 |
| `cargo package --allow-dirty -p tally-units-core` | `target/package/tally-units-core-0.1.0.crate` 생성, 확인 빌드 성공 |
| `cargo package --allow-dirty -p tally-units` | 2.4절의 `no matching package named` 에러(종료 상태 101) |
| `cargo publish --dry-run --workspace --allow-dirty --locked` | 2.4절의 출력, 종료 상태 0 |
| `cargo install --path tally-units --root <임시 디렉터리>` | 2.8절의 출력 |
| `cargo clean` | `target/` 삭제(실습을 마친 뒤) |

## 4. 자주 하는 실수

### 4.1 필드가 모두 공개인 구조체에 필드를 더함 — [`new_private_field.rs`](compile_fail/new_private_field.rs)

```console
$ rustc --edition 2024 --out-dir ../../target/tmp/p3-ch10 compile_fail/new_private_field.rs
error: cannot construct `Size` with struct literal syntax due to private fields
  --> compile_fail/new_private_field.rs:30:13
…
error: pattern requires `..` due to inaccessible fields
  --> compile_fail/new_private_field.rs:31:9
```

챕터 디렉터리에서 실행합니다. 두 에러 모두 에러 코드가 없습니다. 모듈 `size`를 라이브러리의 새 버전이라고 생각하면, 1.0.0에 맞춰 쓴 사용자 코드가 비공개 필드 하나 때문에 두 곳에서 깨진 것입니다. 첫째는 모듈(실제로는 라이브러리 크레이트) 밖에서는 비공개 필드를 채울 수 없어 리터럴로 만들 수 없다는 에러이고, ``help: you might have meant to use the `new` associated function``처럼 생성자를 권합니다. 둘째는 접근할 수 없는 필드가 있으면 구조 분해에 `..`가 필요하다는 에러입니다. 라이브러리 쪽에서 할 일은 이 변경을 메이저로 내거나, 처음부터 비공개 필드를 두는 것입니다(예제 01). 공개 필드를 더한 경우는 ``error[E0063]: missing field `depth` in initializer of `Size` ``와 ``error[E0027]: pattern does not mention field `depth` ``가 납니다.

### 4.2 게시할 때의 실수

`projects/tally-units`의 사본에서 확인했습니다. 에러 행의 종료 상태는 101입니다. `description`·`license` 행은 경고만 내고 0으로 끝나고, 마지막 행은 패키징이 0으로 성공한 뒤 그 `.crate`를 풀어 `cargo test`를 돌릴 때 101로 실패합니다.

| 상황 | 메시지 | 해결 |
| --- | --- | --- |
| path·git 의존성에 `version`이 없음 | `all dependencies must have a version requirement specified when publishing.` | `version`을 함께 적습니다(2.4절). |
| 의존받는 패키지를 게시하지 않고 의존하는 패키지만 게시 | ``no matching package named `tally-units-core` found`` / `location searched: crates.io index` | 의존받는 패키지를 먼저 게시하거나 `--workspace`로 함께 처리합니다. |
| `publish = false`인 패키지를 게시 | ``error: `tally-units` cannot be published.`` | 의도한 것이면 그대로 둡니다. 게시할 것이면 키를 지우거나 레지스트리 목록을 적습니다. |
| 커밋하지 않은 변경이 있음 | `error: N files in the working directory contain changes that were not yet committed into git:` | 커밋하고 게시합니다. 연습이라면 `--allow-dirty`. |
| `description`·`license`가 없음 | ``warning: manifest has no description, license or license-file`` | 채웁니다. crates.io는 둘이 없는 패키지를 받지 않습니다. |
| 버전을 올리고 의존성 요구 사항은 그대로 둠 | ``failed to select a version for the requirement `tally-units-core = "^0.1.0"` `` / `candidate versions found which didn't match: 0.2.0` | 의존하는 쪽의 `version`도 올립니다(연습 3). |
| 테스트 데이터를 빼고 그 테스트 파일은 남김 | (패키징은 경고 없이 성공) 올라간 `.crate`로 테스트를 돌리면 데이터 파일이 없어 실패 | 데이터와 그것을 쓰는 테스트를 함께 빼거나 함께 넣습니다. `tally-units-core`는 `tests/` 전체를 뺐습니다(부록 A ch30 2.9절). |

## 5. 연습 문제

연습마다 `projects/tally-units`를 저장소 밖에 새로 복사해 시작합니다.

### 연습 1. `exclude`를 `include`로

`tally-units-core/Cargo.toml`의 `exclude = ["tests/"]`를 지우고, 같은 파일이 들어가도록 `include`로 바꾸세요. `cargo package --list --allow-dirty -p tally-units-core`의 결과가 2.3절과 같아야 합니다(`.cargo_vcs_info.json`은 사본이 git 저장소가 아니면 없습니다).

<details>
<summary>힌트</summary>

`include = ["src/**", "README.md"]`처럼 넣을 파일만 적습니다. `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`은 `include`에 적지 않아도 Cargo가 넣습니다. 새 파일을 더할 때 `include`는 빠뜨리면 안 들어가고, `exclude`는 빠뜨리면 들어간다는 차이가 있습니다.

</details>

### 연습 2. 변경 판정하기

`tally-units-core`가 1.0.0이라고 할 때, 다음 변경을 메이저·마이너·깨질 수도 있음으로 나누세요. (가) `Unit`에 배리언트 `Mile`을 더함. (나) `convert`의 매개변수 순서를 `(from, to, value)`로 바꿈. (다) `Unit`에 고유 메서드 `symbol(&self) -> &str`을 더함. (라) `rust-version`을 1.85에서 1.95로 올림. (마) 새 Cargo 기능 `imperial`을 더함.

<details>
<summary>힌트</summary>

(가) 마이너: `Unit`이 `#[non_exhaustive]`라서 크레이트 밖의 `match`에는 이미 `_` 갈래가 필요합니다. (나) 메이저: 타입이 달라 부르는 코드가 깨집니다. (다) 깨질 수도 있음: 사용자가 같은 이름의 트레이트 메서드를 쓰고 있었다면 예제 02처럼 됩니다. (라) 깨질 수도 있음: 옛 툴체인 사용자에게는 깨집니다. (마) 마이너. 0.y.z라면 "메이저"는 둘째 자리, "마이너"는 셋째 자리를 올립니다.

</details>

### 연습 3. 버전을 올리다 빠뜨린 것

워크스페이스 루트의 `[workspace.package]`에서 `version`을 `"0.2.0"`으로 올리고 `cargo publish --dry-run --workspace`를 실행하세요. 무엇이 실패하고, 어디를 함께 고쳐야 하나요? 고친 뒤 다시 실행해 통과하는지 확인하세요.

<details>
<summary>힌트</summary>

`tally-units`의 의존성 선언이 `version = "0.1.0"`(= `^0.1.0`, `<0.2.0`)이라서 0.2.0인 `tally-units-core`와 맞지 않아 ``failed to select a version for the requirement `tally-units-core = "^0.1.0"` ``가 납니다. 게시하기 전에 빌드부터 실패합니다. `tally-units/Cargo.toml`의 요구 사항도 `"0.2.0"`으로 올립니다. 이 요구 사항을 `[workspace.dependencies]`에 적어 두면(ch4 2.6절), 의존하는 멤버가 여럿이어도 요구 사항은 한 곳에서 고치면 됩니다(`[workspace.package]`의 `version`과 함께 같은 파일의 두 곳).

</details>

### 연습 4. 설치 기록 읽기

`cargo install --path tally-units --root /tmp/tu-inst`(임시 디렉터리는 원하는 곳)로 설치한 뒤 `.crates.toml`의 내용과 `cargo install --list --root /tmp/tu-inst`의 출력을 보세요. 같은 명령으로 한 번 더 설치하면 무엇이 달라지나요? 마지막에 `cargo uninstall --root /tmp/tu-inst tally-units`로 지웁니다.

<details>
<summary>힌트</summary>

`.crates.toml`에는 `"tally-units 0.1.0 (path+file://…/tally-units)" = ["tally-units"]`처럼 패키지와 출처, 설치한 실행 파일 목록이 적힙니다. path에서 설치한 것을 다시 설치하면, 빌드가 최신이라 `Compiling` 없이 `Replacing …/bin/tally-units`로 실행 파일만 바꿔 넣습니다. `--root`를 주지 않으면 Cargo home의 `bin/`에 설치되어 내 환경이 바뀌므로, 연습에서는 꼭 `--root`를 줍니다.

</details>

## 6. 참고 자료

- The Cargo Book: [Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html), [Registries](https://doc.rust-lang.org/cargo/reference/registries.html), [SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html), [cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html), [cargo publish](https://doc.rust-lang.org/cargo/commands/cargo-publish.html), [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html), [cargo yank](https://doc.rust-lang.org/cargo/commands/cargo-yank.html)
- [Semantic Versioning 2.0.0](https://semver.org/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) (Cargo Book이 라이브러리를 게시하기 전에 보라고 권함)
