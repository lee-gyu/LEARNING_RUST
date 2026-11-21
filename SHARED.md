# 공유 메모

챕터를 쓰고 검증할 때 **모든 챕터가 함께 봐야 하는 것**만 적습니다. 진행 상황은 [PROGRESS.md](PROGRESS.md)에, 한 챕터에만 해당하는 내용(로드맵과 달라진 점, 일부러 실패하는 예제, 외부 크레이트, 확인하지 못한 것)은 그 챕터의 README에 적습니다.

- 적는 기준: 다음 챕터를 쓰는 사람이 모르면 같은 실수를 하거나 약속을 빠뜨리게 되는 것. 끝난 일의 기록과 한 주제에만 해당하는 세부 사실은 적지 않습니다.
- 2026-11-20 이전에 쌓인 주제별 세부 메모(챕터별 린트 기본 수준, 에러 코드가 예상과 달랐던 사례, 챕터별 비고와 검증 기록)는 이 파일로 옮기지 않았습니다. 필요하면 `git show 1250f86:PROGRESS.md`로 봅니다.
- 개인 경로나 세션 내부 식별자는 적지 않습니다.

## 1. 작업 방식 메모

### 1.1 워크스페이스와 디렉터리

- 루트 워크스페이스가 `members = ["part*/ch*", "appd-*/ch*"]`라서, `Cargo.toml`이 없거나 빌드 타깃이 하나도 없는 챕터 디렉터리가 있으면 **모든 챕터의 cargo 명령이 실패**합니다. 챕터 디렉터리에는 `Cargo.toml`과 첫 예제 파일을 연달아 만든 뒤에 cargo를 실행합니다.
- 저장소 안에서 `cargo new`를 실행하지 않습니다. 루트나 파트 디렉터리에서 실행하면 루트 `Cargo.toml`의 `members`에 새 경로가 조용히 덧붙고, 챕터 디렉터리 안에서 실행하면 만든 프로젝트가 `believes it's in a workspace` 에러로 빌드되지 않습니다. `projects/` 아래 독립 프로젝트는 저장소 밖에서 만들어 복사하고 빈 `[workspace]` 표를 넣습니다.
- 독립 프로젝트의 `Cargo.toml`·`Cargo.lock`·소스를 고치는 실험(`cargo add`, `cargo update`, `cargo fix`, 매니페스트 수정, 연습 문제 풀이)은 저장소 밖 사본에서 합니다. `.cargo/config.toml`을 쓰는 프로젝트는 디렉터리를 통째로 복사합니다.
- 임시 파일과 `rustc --out-dir` 산출물은 메인 세션이 `target/tmp/<패키지>`, reviewer가 `target/review/<패키지>`에 둡니다. `rustc`는 컴파일에 실패해도 `--out-dir` 디렉터리를 만듭니다.
- 워크스페이스 멤버(챕터 패키지)의 `[profile]`과 `[patch]`는 무시됩니다. 프로필이 필요한 실습은 독립 프로젝트에서 합니다. 챕터 예제에서는 `CARGO_PROFILE_RELEASE_DEBUG=…`처럼 환경 변수로 루트 워크스페이스의 프로필 설정 하나를 바꿔 실행할 수 있습니다(Part 3 ch6 예제 02).
- `cargo build -q`는 Cargo 자신의 경고도 숨깁니다. 멤버에 `[profile]`을 적어도 `-q`면 아무것도 출력하지 않고 종료 상태 0입니다. 경고를 확인하는 실험은 `-q` 없이 실행합니다.
- 챕터 README와 소스는 `web/scripts/sync-content.mjs`가 사이트 페이지로 바꿉니다. 상대 링크의 대상 파일이 없으면 `[sync-content] … 링크 대상이 없습니다` 경고가 납니다. 점으로 시작하는 파일(`.cargo/config.toml` 등)은 사이트에 실리지 않으므로 내용을 README에 인용합니다.
- 예제 파일 이름은 워크스페이스 전체에서 겹치지 않게 짓습니다. 겹치면 `cargo build --workspace --examples`에서 `output filename collision` 경고가 납니다(`ls part*/ch*/examples/*.rs appd-*/ch*/examples/*.rs | xargs -n1 basename | sort | uniq -d`로 확인).
- 다른 챕터는 `ch8` 형식으로 부릅니다. "N장"은 기준 문서의 장을 가리킬 때만 씁니다.
- `ch8-collections`의 `examples/07_string_utf8.rs`와 그 README의 07 출력 블록에는 일부러 조합형(NFD) 한글 바이트가 들어 있습니다. 편집 도구가 완성형으로 바꿔 저장할 수 있으므로 이 두 파일은 `perl` 등으로 바이트를 보존하며 고칩니다.

### 1.2 환경

- 설치된 툴체인은 1.98.1과 1.100.0뿐입니다(nightly와 Miri 없음). rustup은 설치되지 않은 툴체인 이름을 받으면 조회 명령이라도 내려받기를 시작합니다. 설치 여부는 `rustup toolchain list`로만 확인하고, `--toolchain`·`rustup run`·`cargo +<이름>`은 목록에 있는 것에만 씁니다. `rust-toolchain.toml`이 있는 프로젝트(부록 A ch32의 `projects/pollcount`)에서도 같은 일이 일어날 수 있습니다.
- `mise` 명령은 실행하지 않습니다. 2026-11-20에 `mise ls rust`를 실행하자 mise가 예전 툴체인(1.98.1)을 제거했습니다(원인 미확인). rustup의 기본 툴체인이 1.98.1이라, 그 상태에서 `RUSTUP_TOOLCHAIN`을 지우고 `rustc`를 실행하면 rustup이 다시 내려받습니다. mise의 출력 예시(Part 0 ch1 README)는 다시 실행해 확인하지 않습니다.
- 이 셸에는 `RUSTUP_TOOLCHAIN`과 `CARGO_HOME`이 이미 잡혀 있습니다. 작업 전에 `rustc --version`을 확인합니다. rustup 프록시가 자식 프로세스에 넣어 주는 값을 확인하려면 `env -u`로 변수를 지우고 실행합니다.
- 툴체인·타깃 설치, `cargo install`, 패키지 관리자 사용은 사용자가 승인한 것만 합니다.
- zsh 주의: `$var`를 단어로 나누지 않습니다(`${=var}`). `"$e:PASS"`는 수식자로 해석됩니다(`"${e}:PASS"`). `echo ===`는 실패합니다(`echo '==='`). 파이프 뒤의 `$?`는 마지막 명령의 종료 코드입니다(`${pipestatus[1]}`). `cmd 2>&1 >/dev/null | wc`는 표준 출력도 함께 셉니다.
- 이 환경의 안전 검사는 `rm -rf "$변수/…"` 꼴을 막습니다. 리터럴 경로를 적어 지웁니다.
- 한글이 든 아주 긴 줄을 Python 소스나 표준 입력에 두면 실패한 적이 있습니다. 긴 문자열은 파일에 두고 읽게 합니다.
- 온라인 문서는 최신 stable 기준이라 저장소 툴체인보다 앞설 수 있습니다. API가 있는지와 안정화 버전, Cargo Book의 내용은 `rustc --print sysroot` 아래 `share/doc/rust/html`의 로컬 문서로 확인합니다(Cargo Book은 그 아래 `cargo/`, 표준 라이브러리 소스는 `src/`).
- 모든 검증은 macOS(aarch64)에서만 했습니다. 파일 에러 메시지, 종료 코드, 타이머에 의존하는 출력은 다른 운영체제에서 다를 수 있습니다.

### 1.3 예제와 README

- 실행마다 같은 출력이 나오게 씁니다. 스레드의 결과는 `main`에서 출력하고, 주소·해시 값·경과 시간은 값 대신 비교 결과를 출력하고, `HashMap`은 정렬한 뒤 출력합니다. 테스트 이름이 출력되는 순서는 실행마다 다를 수 있어 README에 싣지 않습니다.
- 표준 입력을 읽는 예제는 `< /dev/null`이나 파이프를 명시하고 제한 시간을 걸어 실행합니다.
- 일부러 패닉하는 예제는 기본 패닉 훅의 메시지에 실행마다 다른 스레드 번호가 들어갑니다. `println!`은 `BrokenPipe`에서 패닉하므로 출력을 `| head`로 자르지 않습니다.
- `rustc`나 `cargo`를 자식 프로세스로 부르는 예제는 환경 변수 `CARGO`를 쓰고, 출력의 절대 경로는 줄여서 싣습니다. `cargo metadata --no-deps`, `cargo fmt`, `cargo tree`는 빌드하지 않고 `target/`도 만들지 않습니다.
- unsafe 코드는 Miri로 검증할 수 없습니다. 블록마다 `// SAFETY:` 주석이 대는 전제 조건을 바로 위 코드가 확인하는 모양으로만 쓰고, 메인 세션이 블록을 읽어 검토하고, README에 미검증이라고 밝힙니다.
- `#[expect(lint)]`는 기본으로 꺼진 린트도 그 범위에서 켜서 충족시킵니다. "Clippy가 잡는다"고 쓰기 전에 속성을 지우고 경고가 실제로 나는지 확인합니다(기본 수준은 `clippy-driver -W help`). `println!` 문에 붙인 `#[expect]`는 무시되므로 `let` 문에 붙입니다. 매크로가 펼친 코드의 린트는 매크로를 부른 곳에 붙입니다. 조건(에디션, 기능, 생성 옵션)에 따라 나기도 하고 안 나기도 하는 경고에는 `#[allow]`를 씁니다.
- Clippy가 예제에서 자주 잡는 것: `print_literal`, `approx_constant`(deny), `redundant_field_names`, `manual_map`, `single_match`, `question_mark`, `useless_vec`, `needless_range_loop`, `type_complexity`, `unused_io_amount`(deny), `never_loop`(deny). `derive(Debug)`로 출력하기만 한 필드는 `dead_code` 경고가 납니다.
- rustfmt는 한글이 든 긴 `println!`을 여러 줄로 펼칩니다. 출력용 도우미 함수로 줄입니다.
- 예제 안의 `#[cfg(test)]` 코드는 `cargo clippy --examples`로 검사되지 않습니다. `cargo clippy -p <패키지> --examples -- --test -D warnings`를 쓰되 먼저 소스를 `touch`합니다. `cargo test --examples`는 예제의 `main`을 실행하지 않습니다.
- 에러 코드와 메시지는 기억과 다른 경우가 많았습니다. `compile_fail/` 파일과 README의 인용은 반드시 `rustc --edition 2024`로 실행해 확인합니다. 에디션에 따라 결과가 다른 파일은 머리말에 에디션별 결과를 적고 이름 끝에 `_2015`·`_2021`을 붙입니다(Part 2의 방식).
- README에 "앞 챕터에서 봤다"고 쓸 때는 그 챕터를 열어 실제로 있는지 확인합니다. 용어가 처음 나오는 곳에는 영어를 병기합니다. 이 두 가지가 Part 3 검증에서 가장 자주 지적됐습니다.
- README에 출력이나 메시지를 옮긴 뒤에는 저장된 파일을 다시 확인합니다. `\ub3c4`처럼 역슬래시 u 이스케이프가 든 출력은 쓰는 과정에서 글자로 바뀐 적이 있으므로 `grep`으로 역슬래시가 남았는지 보고, 실제 출력 파일과 README 블록을 스크립트로 대조합니다(대조할 출력 목록에 바이너리·독립 프로젝트의 출력도 넣습니다).
- 백틱이 든 메시지를 인라인으로 인용할 때는 `` `` 내용 `` ``처럼 백틱 두 개와 공백으로 감쌉니다(세 개로 열면 세 개로 닫아야 함). 사이트가 쓰는 파서로 렌더링해 코드 밖에 남은 백틱을 찾으면 잡힙니다. `web/node_modules/.pnpm/` 아래의 `micromark`와 `micromark-extension-gfm-table`로 README를 HTML로 바꾸고 `<pre>`·`<code>`를 지운 뒤 백틱이 남은 줄을 셉니다.
- 연습 문제가 `Compiling` 줄 수처럼 빌드 상태에 따라 달라지는 것을 물으면 시작 상태(새 사본, `cargo clean`)를 문제에 적습니다. 앞 연습의 변경이 뒤 연습의 답을 바꾸는지도 확인합니다.

### 1.4 Cargo 관련 사실 (cargo 1.100.0에서 확인, Part 3의 남은 챕터용)

- **연습용 레지스트리**: 프로젝트의 `.cargo/config.toml`에 `[source.crates-io] replace-with = "이름"`과 `[source.이름] directory = "vendor"`를 적고 `vendor/<이름>-<버전>/`마다 `Cargo.toml`, `src/lib.rs`, `.cargo-checksum.json`(`{"files":{},"package":null}`)을 두면 네트워크 없이 버전 선택을 재현할 수 있습니다. 한계: `cargo update --precise`가 무시되고, 잠금 파일에 `checksum`이 없고, 디렉터리 밖에서 `--manifest-path`로 실행하면 설정을 읽지 못해 진짜 crates.io를 찾습니다(가상 크레이트 이름이 crates.io에 실제로 있으면 그것을 받습니다). vendor의 소스를 고친 뒤에는 `cargo clean`이 필요합니다. 옛 버전이 잠긴 `Cargo.lock`은 새 버전 디렉터리를 잠시 치우고 `cargo generate-lockfile`로 만듭니다.
- **잠금 파일**: `Locking … to highest Rust <버전> compatible version` 줄은 잠금 파일이 없거나 바뀔 때만 나옵니다. `--locked` 실패의 종료 코드는 101입니다. `--locked` 없는 cargo 명령은 잠금 파일을 말없이 고치므로 저장소 안의 독립 프로젝트는 `--locked`로 검증합니다. 패키지 자신의 `version`을 바꿔도 잠금 파일이 바뀝니다.
- **빌드 산출물**: `target/debug/`에 `deps/`가 없고 크레이트별 결과는 `build/<패키지>/<해시>/out/`에 있습니다. `target/debug/` 바로 아래의 실행 파일과 `.rlib`은 그 사본입니다.
- **Cargo 자신의 린트**: `unused_dependencies`(dev-dependencies는 검사하지 않음), `missing_lints_inheritance`, `unused_workspace_dependencies`, `unused_workspace_package_fields`가 기본 warn입니다. 모르는 매니페스트 키는 `unused manifest key` 경고만 나고 값은 무시됩니다.
- **`[lints]`**: Cargo가 명령줄 옵션으로 넘기므로 note가 `requested on the command line with -F unsafe-code`처럼 나옵니다. 그룹과 개별 린트의 `priority`가 같으면 Clippy `lint_groups_priority`(deny)가 `cargo clippy`를 멈추고, 그때의 실제 결과는 린트 이름에 따라 다릅니다. `pedantic`을 켜면 `must_use_candidate`, `missing_errors_doc`, `unnecessary_wraps`가 흔히 걸립니다. `missing_docs`를 켜면 바이너리·예제·테스트 파일에도 `//!`가 필요합니다.
- **타깃**: `default-run`이 없는 바이너리를 가리키면 매니페스트를 읽는 단계에서 실패해 `cargo run --bin`의 목록도 볼 수 없습니다. `autobins = false`는 `src/main.rs`도 찾지 않게 합니다. harness를 켠 테스트 파일에 `fn main`이 남아 있으면 경고 없이 `running 0 tests`가 됩니다.
- **의존성**: 요구 사항에 맞는 `[patch]` 항목은 레지스트리에 더 높은 버전이 있어도 먼저 골라집니다(Cargo Book 3.3.1의 설명과 다름). 이미 다른 버전이 잠겨 있으면 `patch … was not used` 경고가 나고 `cargo update 이름`으로 풉니다. `cargo add --git`·`--path`는 `version`도 함께 적습니다. 플랫폼별 의존성은 다른 플랫폼에서도 `Cargo.lock`에 적히고 `cargo tree`에는 `--target all`을 줘야 보입니다. 같은 `links` 값을 가진 두 패키지는 해석 단계에서 거부됩니다.
- **워크스페이스**: `cargo new`는 새 경로가 `members`(글롭 포함)에 걸리지 않으면 덧붙이고, `[workspace.package]`·`[workspace.lints]`가 있으면 상속하는 매니페스트를 만듭니다. 워크스페이스 안에 다른 워크스페이스를 두려면 바깥의 `exclude`와 안쪽의 `[workspace]` 표가 둘 다 필요합니다(이 저장소 안의 독립 워크스페이스가 그 경우). 한 워크스페이스에 같은 이름의 패키지를 둘 수 없습니다. 상속한 의존성에 멤버가 덧붙일 수 있는 키는 `features`·`default-features`·`optional`뿐입니다.
- **기능과 cfg**(부록 A ch26에서 확인): 챕터 패키지에 `[features]`를 두면 기본·`--no-default-features`·`--all-features` 세 조합으로 빌드·clippy·실행을 확인합니다(조합을 바꿀 때 소스를 `touch`). 독립 워크스페이스에서는 라이브러리 멤버마다 `-p <멤버>`로 조합을 검사합니다. 루트의 `--no-default-features`는 다른 멤버가 의존성 선언으로 요청한 기본 기능을 끄지 못합니다(기능 통합). 기능 뒤에 둔 함수만 쓰는 테스트 모듈은 그 기능이 꺼진 조합에서 `dead_code` 경고를 내므로 모듈째 같은 조건으로 가립니다. 기능 뒤에 있는 항목을 부르면 E0425에 ``found an item that was configured out`` / ``the item is gated behind the `x` feature`` note가 붙습니다. `dep:`으로 감춘 선택적 의존성 이름을 `--features`에 주면 대신 쓸 기능을 알려 주는 에러가 납니다. `required-features`가 붙은 예제는 기능 없이 `cargo build --examples`에서 건너뛰고 `cargo run`은 종료 상태 101입니다. 사용자 정의 cfg 이름은 `[lints.rust] unexpected_cfgs`의 `check-cfg`로 알립니다. 2024 에디션에서도 `dep:`을 쓰지 않은 선택적 의존성은 같은 이름의 암묵적 기능을 만듭니다. 의존성의 기능이 켜졌는지는 `cargo tree -e features -i <크레이트>`로 봅니다.
- **프로필과 측정**(부록 A ch18·ch20, Part 3 ch6): `panic = "abort"`인 프로그램의 종료 상태는 macOS에서 134입니다. 시간 측정 출력은 실행마다 달라지는 줄을 시간 줄로만 한정하고 "어느 쪽이 빠르다"를 단정하는 줄을 두지 않습니다. 프로필 설정이 실제로 어떻게 넘어갔는지는 `cargo build -v`의 `-C` 옵션으로 확인합니다(Cargo는 `rustc` 기본값과 같은 설정은 넘기지 않음). Cargo Book의 기본값 목록과 달리 cargo 1.100.0은 `debug`가 꺼진 크레이트에 `-C strip=debuginfo`를 넘깁니다. 내장 `debug` 프로필(`--profile debug`)은 `target/debug/`를 `dev`와 함께 씁니다. macOS는 `split-debuginfo = "unpacked"`라서 디버그 정보가 실행 파일 밖(`.o`)에 있으므로 dev 실행 파일 크기로 디버그 정보의 양을 비교할 수 없습니다. 실행 파일 크기는 빌드한 경로에 따라 dev 계열이 조금 달라집니다. 백트레이스 이름을 확인하는 예제는 `#[inline(never)]` 없이는 release에서 프레임이 사라집니다. 테스트 실행 파일은 build 디렉터리(`target/debug/build/<패키지>/<해시>/out/`)에서 실행됩니다. `CARGO_BUILD_BUILD_DIR`로 옮긴 build 디렉터리는 `cargo clean`이 지우지 않습니다.
- **설정과 CI**(부록 A ch32): `CARGO_BUILD_WARNINGS=deny`는 rustc·Clippy·rustdoc·Cargo의 경고를 모두 실패로 만듭니다. `RUSTFLAGS` 값이 바뀌면 전부 다시 컴파일됩니다. YAML 문법은 시스템 Ruby의 `ruby -ryaml`로 확인할 수 있습니다.
- **빌드 스크립트**(부록 A ch34·ch35): `cc` 1.11.2는 소스의 `rerun-if-changed`를 내지 않으므로 직접 출력합니다. C 컴파일러와 libclang(Homebrew LLVM)은 설치돼 있습니다.
- **테스트와 게시**(부록 A ch30·ch31): `cargo test --all-targets`는 문서 테스트를 빼먹습니다(`--doc`을 따로). criterion과 함께 쓰는 라이브러리에는 `[lib] bench = false`가 필요합니다. `cargo package --list`는 커밋하지 않은 파일이 있으면 `--allow-dirty`가 필요합니다. `cargo install`은 1.100.0부터 `.crate` 안의 `Cargo.lock`을 기본으로 씁니다. `cargo metadata`의 출력에는 절대 경로가 나오고, `resolve.nodes[].deps[].name`은 밑줄로 바뀐 라이브러리 이름입니다.
- **이 저장소의 실제 예**: 워크스페이스에 `rand`가 두 버전(0.10.4, `trpl`이 끌어온 0.8.8) 있어 `-p rand`는 `ambiguous` 에러이고 `rand@0.10.4`처럼 적어야 합니다. `cargo doc -p rand@0.10.4`는 1.100.0에서도 cargo 내부 패닉으로 끝납니다.

### 1.5 reviewer 운영

- reviewer가 사용량 한도로 중단되면 한도가 풀린 뒤 같은 reviewer에게 메시지를 보내 이어서 실행합니다.

## 2. 후속 조치

### 2.1 뒤 챕터가 이어받을 내용

챕터를 계획할 때 자기 행을 찾아 "이어받을 내용"에 넣습니다. 반영한 항목은 지웁니다.

| 맡을 챕터 | 이어받을 내용 (넘긴 곳) |
| --- | --- |
| Part 3 `ch7-configuration-env` | Cargo home의 `config.toml`, Cargo가 설정하는 환경 변수(Part 3 ch1). 패키지 가까이의 `.cargo/config.toml`과 소스 교체 설정(ch2). 설정 파일의 `[profile]`과 `CARGO_PROFILE_<프로필>_<설정>` 환경 변수의 규칙, `build.target-dir`·`build.build-dir`·`CARGO_INCREMENTAL`, 설정을 `Cargo.toml`·프로젝트·Cargo home 중 어디에 둘지(ch6 2.1·2.5·2.7절이 이름만 보임) |
| Part 3 `ch8-build-scripts` | `[build-dependencies]`의 쓰임(Part 3 ch2). 빌드 스크립트가 켜진 기능을 아는 `CARGO_FEATURE_<이름>`(ch5). `build`·`links` 키(ch3). 빌드 스크립트가 만든 코드의 에디션 마이그레이션(Part 2 ch6). 빌드 스크립트는 `build-override` 기본값으로 최적화하지 않고 `panic` 설정을 무시하는 것, 디버깅용 `[profile.dev.build-override] debug = true`(ch6 2.4절), timings 보고서의 주황색 빌드 스크립트 단위(ch6 2.6절). 부록 A ch34·ch35와 서로 링크 |
| Part 3 `ch9-testing-ci` | `benches/`, CI에서 Cargo home 캐시(Part 3 ch1). `[[bench]]`와 벤치마크용 `harness = false`, MSRV를 실제 툴체인으로 확인하기(ch3). 워크스페이스의 CI(ch4). 기능 조합을 돌리는 도구와 CI(ch5. 기본·없음·전부 세 조합과 `-p` 검사는 ch5 2.6절이 보였음). `cargo test 이름`이 문서 테스트를 건너뛰는 동작(Part 0 ch11). 부록 A ch30·ch32와 서로 링크 |
| Part 3 `ch10-publishing-semver` | 레지스트리에 게시, `cargo install`(Part 3 ch1). SemVer 호환성 규칙 전체, git·path 의존성이 있으면 게시가 거절되는 것, 다른 레지스트리(ch2). 게시용 키와 `include`·`exclude`(ch3). 워크스페이스 패키지 게시(ch4). 기능·선택적 의존성을 더하거나 빼는 것과 기본 기능에서 빼는 것의 SemVer 규칙(ch5). 부록 A ch21·ch30과 서로 링크 |
| Part 3 `ch11-commands-tools` | `cargo metadata`(Part 3 ch3·ch4의 예제가 문자열 검색으로만 읽음). 사용자 정의 하위 명령(부록 A ch31과 서로 링크) |
| Part 4 전체 | rustdoc 기능, 숨김 줄과 코드 블록 속성(부록 A ch27·ch30과 서로 링크). docs.rs가 문서를 만들 때 켤 기능(`[package.metadata.docs.rs]`)과 문서의 기능 표시(`doc_cfg`, nightly), 꺼진 기능 뒤 항목으로 가는 문서 링크가 깨지는 문제(Part 3 ch5) |
| Part 5 `ch1-rustc-basics` | `rustc` 옵션(Part 3 ch1), `crate-type`의 종류(ch3) |
| Part 5 `ch3-lints` | 린트 수준이 여러 곳에서 정해질 때의 우선순위(`--force-warn`, `-W`, `#[allow]`, `[lints]`)(Part 2 ch6, Part 3 ch3). 부록 A ch29와 서로 링크 |
| Part 5 `ch4`·`ch5`·`ch6` | 크로스 컴파일(부록 A ch32·ch33), `--cfg`·`--check-cfg`(부록 A ch26. Cargo가 기능을 이 옵션으로 넘기는 모습은 Part 3 ch5 2.2절), 커버리지(부록 A ch31·ch32) |
| Part 6 `ch1-reading-diagnostics` | 에디션과 무관한 `cargo fix`(Part 2 ch6) |
| Part 7 `ch2-implementation-errors` | `anyhow`의 `context`와 종료 코드(부록 A ch4). Part 1 ch8 2.4절이 이 챕터를 가리킴 |
| Part 8 `ch1`·`ch5`·`ch7` | `no_std`(부록 A ch33), 타입 상태 빌더(부록 A ch7) |
| Part 9 `ch2`·`ch4`·`ch6`·`ch8`·`ch9` | 매크로(부록 A ch19·ch28). `#[non_exhaustive]`·`#[deprecated]`·`#[doc(hidden)]`(부록 A ch21~ch26). 2024 에디션 바인딩 모드 규칙의 상세(Part 0 ch18). 가시성과 이름 해석 규칙 전체, 봉인된 트레이트를 다룰지(부록 A ch22). 패닉과 링크(부록 A ch18·ch34) |
| Part 10 `ch2-data-layout` | `Packet { u8, u32, u8 }`은 기본 표현 8바이트·`#[repr(C)]` 12바이트, `Option<Cell<bool>>`은 2바이트(1.100.0). `#[repr(transparent)]`(부록 A ch6) |
| Part 10 `ch7`·`ch8`·`ch9` | 소멸자가 실행되지 않는 경우(부록 A ch11), 이중 패닉과 `catch_unwind`(부록 A ch11·ch18), 동시성(부록 A ch17). 변성·HRTB·drop 검사(부록 A ch14) |
| Part 10 `ch12-ffi-beneath-std` | `CString::into_raw`/`from_raw`, `CStr::from_ptr`이 돌려주는 참조의 라이프타임이 아무것에도 묶여 있지 않다는 점, `Option<NonZero<T>>`의 FFI 호환 보장(Part 1 ch14). 부록 A ch34·ch35와 서로 링크 |
| Part 11 `ch2-compiler-flags` | 새니타이저(부록 A ch31) |

### 2.2 nightly와 Miri를 준비하면 다시 확인할 것

Part 10을 시작할 때 준비합니다. 확인한 뒤 각 README의 미검증 고지를 고칩니다.

- Part 0 ch19 `01`·`02`, Part 1 ch14 `04_calling_c`(Miri가 `strlen`·`getenv`를 지원하는지도 확인)
- Part 2 ch4 `05_unsafe_changes`와 `projects/edition_compare/shared/new_syntax.rs`, ch5 `03_env_unsafe`와 `projects/ecosystem_compare/shared/main.rs`, ch6 `02_fix_leftovers_2024`
- 부록 A ch16(예제 5개와 `04_safe_wrapper`의 테스트), ch27 `03`, ch33 `05`, ch34 `03`~`07`
- nightly가 필요해 설명만 한 것: `gen` 블록과 `edition = "future"`(Part 2), ThreadSanitizer(부록 A ch17), `cargo-fuzz`(부록 A ch30), Cranelift 백엔드·병렬 프런트엔드(`-Zthreads`)·`[resolver] feature-unification`(Part 3 ch6 2.7절)

### 2.3 설치를 승인받으면 실행해 볼 것

README에 "실행해 확인하지 못했다"고 밝힌 도구입니다. 실행한 뒤 고지를 고칩니다.

- `cargo-semver-checks`, `cargo-deny`, `cargo-udeps`, `cargo-machete`, `cargo-audit`, `cargo-hack`, `cargo-expand`, `cargo-tarpaulin`, `bindgen` CLI, `cbindgen`, `rustfilt`, 타깃 `thumbv6m-none-eabi`(부록 A ch21·ch25·ch26·ch28·ch31~ch35). `sccache`, 다른 링커(LLD·mold·wild), `cargo-features-manager`·`cargo-unused-features`(Part 3 ch6 2.5·2.7절)
- 부록 A ch32의 `projects/pollcount/ci/github-workflow.yml`은 GitHub에서 실행해 보지 못했습니다. 이 프로젝트의 `rust-toolchain.toml`(`1.100.0`)은 저장소 툴체인을 올릴 때 함께 고칩니다.

### 2.4 고칠 문서

- Part 0과 Part 1 ch1~ch4에서 **파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력**은 1.98.1에서 확인한 그대로입니다. 2026-11-20의 재확인은 예제 출력(두 툴체인에서 같음)과 `compile_fail/`의 메시지(문구가 바뀐 곳을 고침), 독립 프로젝트의 빌드·테스트만 다뤘습니다. 해당 챕터를 다시 손볼 때 1.100.0에서 재현해 확인합니다.
- 범위 정리(writing-guide 7절)에서 상대 링크로 바꿀 곳: `ch5-features`를 이름으로만 가리키는 Part 3 ch2·ch3·ch4, Part 2 ch3, 부록 A ch26. `ch6-profiles-build-performance`를 이름으로만 가리키는 Part 3 ch1(2.3절)·ch3·ch4(2.7절 표), Part 1 ch8, 부록 A ch18·ch20, 그리고 Part 0 ch1의 "(Part 3에서 자세히 다룹니다)". 바꾼 뒤 이 항목을 지웁니다.
- 정할 것(선택 사항): 분량이 목표를 넘은 README(Part 0의 여러 챕터, Part 1 ch2·ch4, Part 2 ch4)를 줄일지. Part 1 ch12 `03_split_read_write`가 주고받는 64 MiB를 줄일지. 부록 A의 "책에 없는 지침"을 둘지.

### 2.5 crates.io의 시점에 기대는 것

각 프로젝트의 `Cargo.lock`이 버전을 고정합니다. `cargo update`의 결과와 README에 실은 버전 번호는 그 시점 기준입니다.

- 2026-11-19: Part 2 ch5 `projects/msrv_resolver`, 부록 A ch25 `projects/real_graph`(2026-11-20에 다시 확인, 변화 없음), 부록 A ch15·ch16이 소개한 크레이트 버전
- 2026-11-20: 부록 A ch27~ch35의 프로젝트, Part 3 ch1 `text-stat`, ch2 `three-sources`(GitHub `rust-lang/cfg-if`의 태그 `v1.0.4` 포함), ch4 `bookshelf`, 챕터 패키지의 dev-dependency(`rand`, `trpl`, `thiserror`, `anyhow`, `derive_builder`, `semver`)
- 2026-11-21: Part 3 ch6 `collatz`(`thiserror` 2.0.21, 그 아래 `syn` 3.0.10 등)

## 3. 용어 메모

챕터 사이에 표기를 맞추기 위해 정한 용어입니다. 새 용어를 정하면 여기에 더하고, 이미 있는 표기와 다르면 먼저 쓴 쪽이나 더 많은 챕터가 쓰는 쪽으로 맞춥니다.

| 영어 | 표기 |
| --- | --- |
| ownership / move / borrow / reference | 소유권 / 이동 / 대여 / 참조 |
| lifetime / trait / generic | 라이프타임 / 트레이트 / 제네릭 |
| statement / expression | 구문 / 표현식 |
| mutable / immutable / shadowing | 가변 / 불변 / 섀도잉 |
| binding / scope | 바인딩 / 스코프 |
| parameter / argument | 매개변수 / 인자 |
| crate / package / module | 크레이트 / 패키지 / 모듈 |
| struct / enum / method / associated function | 구조체 / 열거형 / 메서드 / 연관 함수 |
| closure / iterator | 클로저 / 반복자 |
| match arm / variant / pattern | 갈래 / 배리언트 / 패턴 |
| type annotation / dependency | 타입 표기 / 의존성 |
| owner / copy / clone | 소유자 / 복사 / 복제 |
| dangling reference / borrow checker / data race | 댕글링 참조 / 대여 검사기 / 데이터 경쟁 |
| double free / use after free | 이중 해제 / 해제 후 사용 |
| dereference / deref coercion | 역참조 / 역참조 강제 변환 |
| drop | "버려지다(drop)", "drop된다" |
| toolchain / workspace / edition / profile / manifest | 툴체인 / 워크스페이스 / 에디션 / 프로필 / 매니페스트 |
| format string / placeholder / format spec | 서식 문자열 / 자리표시자 / 서식 지정 |
| `Display` `{}` / `Debug` `{:?}` | 기본 형식 / 디버그 형식 |
| constant / literal / destructuring / unit | 상수 / 리터럴 / 구조 분해 / 유닛 |
| overflow / wrapping / tail expression / label / range | 오버플로 / 래핑 / 마지막 표현식 / 레이블 / 범위 |
| debug build / release build / exit code | 디버그 빌드 / 릴리스 빌드 / 종료 코드 |
| field / instance / attribute / getter | 필드 / 인스턴스 / 속성 / 게터 |
| field init shorthand / struct update syntax / partial move | 필드 초기화 축약 / 구조체 갱신 문법 / 부분 이동 |
| tuple struct / unit-like struct | 튜플 구조체 / 유닛 구조체 |
| automatic referencing and dereferencing | 자동 참조·역참조 |
| module tree / crate root / binary crate / library crate | 모듈 트리 / 크레이트 루트 / 바이너리 크레이트 / 라이브러리 크레이트 |
| path / absolute path / relative path | 경로 / 절대 경로 / 상대 경로 |
| private / public / re-export / encapsulation | 비공개 / 공개 / 재내보내기 / 캡슐화 |
| nested path / glob / alias / namespace | 중첩 경로 / 글롭 / 별칭 / 이름 공간 |
| item / prelude | 항목 / 프렐류드 |
| catch-all pattern / wildcard `_` | 포괄 패턴 / `_` 패턴(와일드카드) |
| exhaustiveness checking / refutable / irrefutable | 완전성 검사 / 반박 가능한 / 반박 불가능한 |
| happy path / diverge | 행복한 경로 / 흐름을 끊다 |
| collection / vector / hash map / key / value | 컬렉션 / 벡터 / 해시 맵 / 키 / 값 |
| length / capacity / reallocation | 길이 / 용량 / 재할당 |
| grapheme cluster / code point | 자소 클러스터 / 코드 포인트 |
| unrecoverable / recoverable error | 복구할 수 없는 / 복구할 수 있는 에러 |
| unwinding / backtrace / propagating errors | 되감기 / 백트레이스 / 에러 전파 |
| invariant / validation type / type alias | 불변식 / 검증 타입 / 별칭 |
| command line argument / environment variable / redirection | 명령줄 인자 / 환경 변수 / 리다이렉션 |
| separation of concerns / TDD | 관심사 분리 / 테스트 주도 개발 |
| type parameter / concrete type / monomorphization | 타입 매개변수 / 구체 타입 / 단형성화 |
| trait bound / default implementation / override | 트레이트 바운드 / 기본 구현 / 재정의 |
| orphan rule / coherence / blanket implementation | 고아 규칙 / 일관성 / 포괄 구현 |
| lifetime annotation / lifetime parameter / elision rules | 라이프타임 표기 / 라이프타임 매개변수 / 생략 규칙 |
| trait object | 트레이트 객체 |
| test runner / assertion | 테스트 러너 / 단언 |
| unit / integration / doc test | 단위 테스트 / 통합 테스트 / 문서 테스트 |
| capture (test output) | 가로채기 |
| smart pointer / indirection / recursive type | 스마트 포인터 / 간접 참조 / 재귀 타입 |
| destructor / reference counting / strong·weak reference | 소멸자 / 참조 카운팅 / 강한·약한 참조 |
| interior mutability / mock object | 내부 가변성 / 가짜 객체 |
| reference cycle / memory leak / associated type | 순환 참조 / 메모리 누수 / 연관 타입 |
| concurrency / parallelism / thread | 동시성 / 병렬성 / 스레드 |
| message passing / channel / transmitter / receiver | 메시지 전달 / 채널 / 송신자 / 수신자 |
| shared state / mutex / lock / poisoning | 공유 상태 / 뮤텍스 / 잠금 / 포이즈닝 |
| race condition / deadlock / marker trait | 경쟁 조건 / 교착 상태 / 마커 트레이트 |
| capture / iterator adapter / consuming adapter | 캡처 / 어댑터 / 소비 메서드 |
| lazy / turbofish / zero-cost abstraction | 게으르다 / 터보피시 / 비용 없는 추상화 |
| future / task / runtime / executor / stream | 퓨처 / 태스크 / 런타임 / 실행기 / 스트림 |
| await point / cooperative multitasking / starvation / blocking | await 지점 / 협력적 멀티태스킹 / 굶주림 / 블로킹 |
| pin / self-referential / state machine | 고정 / 자기 참조 / 상태 기계 |
| worker thread / extension trait / cancel | 작업자 스레드 / 확장 트레이트 / 취소 |
| static / dynamic dispatch / dyn compatible | 정적 디스패치 / 동적 디스패치 / dyn 호환 |
| state pattern / polymorphism / inheritance | 상태 패턴 / 다형성 / 상속 |
| default binding mode / match guard / let chains | 기본 바인딩 모드 / 매치 가드 / let 체인 |
| range pattern / slice pattern / rest `..` | 범위 패턴 / 슬라이스 패턴 / 나머지 |
| raw pointer / undefined behavior / safe abstraction | 원시 포인터 / 미정의 동작 / 안전한 추상화 |
| operator overloading / fully qualified syntax / supertrait | 연산자 오버로딩 / 완전 정규화 문법 / 슈퍼트레이트 |
| newtype pattern / never type / DST / wide pointer / function pointer | 뉴타입 패턴 / never 타입 / 동적 크기 타입 / 넓은 포인터 / 함수 포인터 |
| declarative / procedural macro / token stream / expand | 선언적 / 절차적 매크로 / 토큰 스트림 / 펼쳐지다 |
| thread pool / graceful shutdown / worker / job / queue | 스레드 풀 / 우아한 종료 / 작업자 / 작업 / 대기열 |
| request line / status line / header / body | 요청 줄 / 상태 줄 / 헤더 / 본문 |
| temporary (value) / bind | 임시값 / 바인드 |
| stdout / stderr | 표준 출력 / 표준 에러 |
| signature / primitive type / combinator | 시그니처 / 원시 타입 / 콤비네이터 |
| pattern (`str` 메서드의 인자) / parsing / round-trip | 검색 패턴 / 파싱 / 왕복 |
| formatting trait / alternate form `#` / helper (`debug_struct` 등) | 서식 트레이트 / 대체 형식 / 도우미 |
| allocation / in-place mutation / replacement character | 할당 / 제자리 수정 / 대체 문자 |
| eager evaluation / short-circuit | 즉시 평가 / 단락 평가 |
| nested `Option<Option<T>>` / `flatten` | 두 겹 / 한 겹 벗긴다 |
| binary search / insertion point / priority queue | 이진 탐색 / 삽입 위치 / 우선순위 큐 |
| stable / unstable sort / total order / lexicographic | 안정 정렬 / 불안정 정렬 / 전순서 / 사전식 |
| deque / ring buffer / set / binary heap / max-heap / min-heap | 덱 / 링 버퍼 / 집합 / 이진 힙 / 최대 힙 / 최소 힙 |
| union / intersection / difference / symmetric difference | 합집합 / 교집합 / 차집합 / 대칭 차집합 |
| subset / superset / amortized / expected cost | 부분집합 / 상위집합 / 분할 상환 / 기대 비용 |
| allocator / seed / cursor | 할당기 / 시드 / 커서 |
| conversion / fallible / infallible / lossless | 변환 / 실패할 수 있는 / 실패하지 않는 / 손실 없는 |
| partial order / equivalence relation / logic error | 부분 순서 / 동치 관계 / 논리 오류 |
| hasher / `BuildHasher` / prefix-free | 해셔 / 해셔 공장 / 경계가 섞이지 않게(prefix-free) |
| clone-on-write / reflexive impl / discriminant | 쓰기 시 복제 / 반사 구현 / 판별값 |
| alignment / padding / layout / representation | 메모리 정렬 / 패딩 / 메모리 배치 / 표현(기본 표현, 표현 지정) |
| niche / tag / thin pointer / zero-sized type | 니치 / 태그 / 얇은 포인터 / 크기가 0인 타입 |
| handle (`JoinHandle`, 표준 입출력·파일·소켓의 핸들, `Vec`·`String`의 포인터·용량·길이 구조체) / placeholder value | 핸들 / 자리 표시 값 |
| guard (`Ref`/`RefMut`) / reentrancy / scrutinee | 대여 가드 / 재진입 / 조건식 |
| panic hook / payload / unwind safety / unwind boundary | 패닉 훅 / 페이로드 / 되감기 안전성 / 되감기 경계 |
| `source` chain / downcast / wrap / error report | `source` 체인 / 다운캐스트 / 감싸다 / 에러 보고서 |
| borrowed / owned (`Cow`, `Borrow`의 문맥) | 빌린 / 소유한 ("빌림과 소유") |
| scoped thread / condition variable / spurious wakeup | 스코프 스레드 / 조건 변수 / 가짜 깨어남 |
| atomic type / memory ordering / compare-and-swap | 원자 타입 / 메모리 순서 / 비교 후 교환 |
| bounded channel / rendezvous channel / backpressure | 용량이 정해진 채널 / 랑데부 채널 / 배압 |
| thread-local / lazy initialization / reader-writer lock | 스레드 지역 변수 / 지연 초기화 / 읽기-쓰기 잠금 |
| snapshot / busy waiting / work stealing | 스냅숏 / 바쁜 대기 / 작업 훔치기 |
| socket address / loopback / listener / datagram / ephemeral port | 소켓 주소 / 루프백 / 리스너 / 데이터그램 / 임시 포트 |
| byte stream / framing / length prefix / delimiter | 바이트 스트림 / 메시지 경계 나누기(프레이밍) / 길이 접두사 / 구분자 |
| timeout / name resolution / retransmission / non-blocking | 제한 시간 / 이름 해석 / 재전송 / 논블로킹 |
| child / parent process / pipe / exit status / builder | 자식 프로세스 / 부모 프로세스 / 파이프 / 종료 상태 / 빌더 |
| monotonic clock / system clock / Unix time / epoch | 단조 시계 / 시스템 시계 / 유닉스 시간 / 기준 시각 |
| elapsed time / deadline / polling / zombie / reap | 경과 시간 / 마감 시점 / 폴링 / 좀비 / 거둬들이다 |
| reader / writer / buffering / flush / line buffering | 리더 / 라이터 / 버퍼링 / 플러시(버퍼 비우기) / 줄 버퍼링 |
| system call / EOF / seek / metadata | 시스템 호출 / EOF(입력의 끝) / 위치 옮기기 / 메타데이터 |
| path component / separator / extension / symbolic link / canonical path | 구성 요소 / 구분자 / 확장자 / 심볼릭 링크 / 정규 경로 |
| poll / waker / wake / pinned / task queue / busy polling | 폴링 / 웨이커 / 깨우다 / 고정된 / 태스크 대기열 / 바쁜 폴링 |
| auto trait / pin projection / oneshot channel | 자동 트레이트 / 고정 투영 / 일회용 채널(oneshot) |
| C string / NUL terminator / interior NUL | C 문자열 / NUL 종료 바이트(0 바이트) / 중간의 0 바이트 |
| OS string (`OsString`/`OsStr`) / upcast / type map | 운영체제 문자열 / 업캐스트 / 타입 맵 |
| fold / helper trait (`DoubleEndedIterator` 등) / iterator struct (`Map`, `Drain` 등) | 접기 / 보조 트레이트 / 반복자 구조체 |
| associated constant / inherent impl / extern prelude | 연관 상수 / 고유 구현 / 외부 프렐류드 |
| checked / saturating 연산 / Euclidean division / endian | 검사 연산 / 포화 연산 / 유클리드 나눗셈 / 엔디언 |
| since (문서의 버전 표시) / nightly-only experimental API / badge | 안정화 버전 / nightly 전용 실험 API / 표시 |
| API 문서의 절 이름 (Implementations, Methods from Deref, Trait Implementations 등) | 영어 그대로 |
| backward compatibility / breaking change / migration | 하위 호환성 / 호환되지 않는 변경 / 마이그레이션 |
| lint / lint group / lint level | 린트 / 린트 그룹 / 린트 수준 (Part 2부터. Part 0·1의 일부 챕터와 위 작업 방식 메모는 lint로 썼음) |
| edition compatibility lint group (`rust-2021-compatibility`) / idiom lint group (`rust_2018_idioms`) | 에디션 호환성 린트 그룹 / 관용구 린트 그룹 |
| strict / reserved / weak keyword / raw identifier | 엄격한 키워드 / 예약 키워드 / 약한 키워드 / raw 식별자 |
| baseline (2015 에디션) / stability without stagnation | 기준선 / 정체 없는 안정성 |
| virtual manifest / sysroot crate / inference variable | 가상 매니페스트 / sysroot 크레이트 / 추론 변수 |
| reserved syntax / prefix / token tree | 예약 문법 / 접두사 / 토큰 트리 |
| feature resolver / feature unification | 기능 리졸버(기능 리졸버 1/2) / 기능 통합 |
| disjoint capture / dummy let (`let _ = &x;`) / method lookup | 분리 캡처 / 더미 let / 메서드 탐색 |
| temporary scope / temporary lifetime extension | 임시값 범위 / 임시값 수명 연장 |
| RPIT / opaque type / hidden type / precise capturing (`use<..>`) | 반환 위치의 `impl Trait`(RPIT) / 불투명 타입 / 숨은 타입 / 정밀 캡처 |
| never type fallback / fallback type | never 타입 폴백 / 폴백 타입 |
| fragment specifier / fully explicit pattern / symbol | 프래그먼트 지정자 / 완전히 명시한 패턴 / 심볼 |
| deprecated | 처음 한 번 "더는 권장하지 않는(deprecated)"으로 풀어 쓰고 이후 deprecated |
| future-compatibility warning / automatic fix / suggestion | 장래 호환성 경고 / 자동 수정 / 제안 |
| 특정 에디션 | "2024 에디션" 꼴 (Part 2부터. 앞 파트의 "Edition 2024"와 같은 뜻) |
| merged doctests / standalone | 문서 테스트 병합(묶음 컴파일, 묶음 바이너리) / 따로 컴파일 |
| rust-version aware resolver / MSRV / yank | 리졸버 3(rust-version 인식 리졸버) / 최소 지원 Rust 버전(MSRV) / 게시 취소(yank) |
| style edition / version sort / optional dependency | 스타일 에디션 / 버전 정렬 / 선택적 의존성 |
| async-signal-safe | 비동기 시그널 안전 |
| (마이그레이션) 자동 수정 뒤 정리 / 고치지 않고 경고만 보기 | 다듬기 / 미리 보기 |
| idiom / working tree / staged | 관용구 / 작업 트리 / 스테이지에 올린 |
| (부록 A) Item / guideline / idiom | Item(영어 그대로, "Item 4") / 지침 / 관용구 |
| (부록 A) README의 고정 표현 | 비권장 코드 / 권장 코드 / 지침을 따르지 않아도 되는 경우 / 기억할 것 / 책과 달라진 점 |
| (부록 A) 같은 코드의 여러 버전 / 변환 메서드를 이어 쓴 것 | "~판"(`match`판, 손으로 쓴 판, `thiserror` 판) / 체인 |
| caller / callee (시그니처 설계의 문맥) | 부르는 쪽 / 받는 쪽 (에러를 돌려받는 문맥에서는 본편처럼 "호출한 쪽"도 씀) |
| function body / method receiver / fn item | 본문 / 수신자 / 함수 아이템 |
| loosest bound / idempotent | 가장 느슨한 바운드 / 멱등 |
| widening / narrowing conversion / boundary | 넓히는 변환 / 좁히는 변환 / 경계(바깥 입력을 안쪽 타입으로 바꾸는 지점) |
| product type / sum type / algebraic data type | 곱 타입 / 합 타입 / 대수적 데이터 타입 |
| sentinel value / invalid state / red flag | 센티널 값 / 무효한 상태 / 경고 신호 |
| transform (`Option`/`Result`의 메서드) | 변환 메서드 |
| nested error / minimal error / type erasure | 중첩 에러 / 최소 에러 / 종류를 지우다 |
| coercion site / sign extension | 강제 변환 자리 / 부호 확장 |
| unit newtype / validated newtype / delegating impl / accessor | 단위 뉴타입 / 검증 뉴타입 / 위임 구현 / 읽기용 접근자 |
| setter / options struct / typestate builder | 세터 / 옵션 구조체(필수 값을 뺀 선택 값 묶음 + `Default`) / 타입 상태 빌더 |
| consuming builder / non-consuming builder | 소비형 빌더 / `&mut` 빌더 |
| vtable / (wide pointer) metadata | vtable(처음 한 번 "메서드 포인터 표"로 풀어 씀) / 메타데이터 |
| accumulator / side effect / early exit / bounds check | 누적 변수 / 부수 효과 / 탈출 조건 / 경계 검사 |
| checklist (공개 타입의 트레이트 판단표) / normalize / (Copy의) copy | 체크리스트 / 정규화 / 사본 |
| 트레이트 구현 사이에 맞아야 하는 성질(`Eq`와 `Hash` 등) | "서로 맞아야 한다", "같은 기준"으로 풀어 씀("일관성"은 coherence에만 씀) |
| heterogeneous collection / combination trait / open·closed set | 이종 컬렉션(처음 한 번 풀어 씀) / 조합 트레이트 / 열린 집합·닫힌 집합 |
| thin generic shell / inner function / symbol / mangling | 얇은 껍데기 / 안쪽 함수 / 심볼 / 맹글링 |
| required method / provided method / implementor / user (of a trait) | 필수 메서드 / 기본 메서드 / 구현하는 쪽 / 쓰는 쪽 |
| RAII / guard (일반 개념) / release method / safety net / double panic | RAII(처음 한 번 "자원 획득은 초기화"로 풀어 씀) / 가드 / 해제 메서드 / 안전망 / 이중 패닉 |
| chain of ownership / view type / non-lexical lifetimes / anonymous lifetime / constant promotion | 소유 사슬 / 뷰 타입 / 마지막 사용에서 끝나는 대여(NLL) / 익명 라이프타임 / 상수 승격 |
| `&'static T` / `T: 'static` | 영원히 유효한 참조 / 짧은 대여를 품지 않은 타입 |
| shared reference (`&T`) | 불변 참조(본편 표기. 책의 "shared reference"는 처음 한 번만 병기) |
| two-phase borrow / disjoint borrow / generation (index) | 2단계 대여 / 나눠 빌리기 / 세대 ID·세대 번호 |
| `take`·`replace`로 `&mut` 뒤의 값을 꺼내는 일 | 바꿔 끼우기 |
| critical section / lock inversion / lock ordering (hierarchy) / rank | 임계 구역 / 잠금 순서 뒤집힘 / 잠금 순서 / 순위 |
| (함께 맞아야 하는 데이터가 서로 맞지 않는 상태) / helper method / snapshot(복사해 둔 값) | 어긋난 상태 / 도우미 메서드 / 사본 |
| abort / exception safety / hidden panic / restriction group | 중단(abort) / 예외 안전성 / 숨은 패닉 지점 / restriction 그룹(처음 한 번 "일부러 고른 프로젝트만 켜는 린트") |
| panicking version / fallible version (같은 기능의 두 판) | 패닉판 / 실패를 돌려주는 판(`Result`판·`Option`판) |
| reflection / RTTI / downcast chain / capability query method / optional method | 리플렉션 / 실행 중 타입 정보(RTTI) / 다운캐스트 사슬 / 기능 조회 메서드 / 선택적 메서드 |
| zero-copy / premature optimization / benchmark / hot path / median | 복사 없는(zero-copy) / 섣부른 최적화 / 벤치마크 / 뜨거운 경로 / 가운데 값 |
| count (결정적인 횟수·바이트) / measure (경과 시간) | 세다 / 재다 |
| soundness / sound / unsound | 건전성 / 건전한 / 건전하지 않은 |
| safety comment (`// SAFETY:`) / precondition / safe wrapper / safety invariant | 안전성 주석 / 전제 조건 / 안전한 래퍼 / 안전성 불변식 |
| visibility / effective visibility / public path (`pub use`로 보이는 경로) / restricted (`pub(…)`) | 가시성 / 실제로 닿는 범위 / 공개 경로 / 범위를 좁힌 항목 |
| handle / passive data (C-STRUCT-PRIVATE) / sealed trait / state type | 핸들 / 수동적인 데이터 / 봉인된 트레이트 / 상태 타입 |
| wildcard import / glob import / explicit import | 와일드카드 가져오기(제목과 첫머리) = 글롭 가져오기(본문에서는 "글롭") / 명시적 가져오기 |
| version pinning (`=x.y.z`) / caret requirement / snapshot (특정 버전의 사본) | 버전 고정 / 캐럿 요구 사항 / 스냅숏 |
| method probing step (`T` → `&T` → `&mut T`) / silent change | 탐색 단계 / 조용히 바뀜(컴파일 에러 없이 다른 항목을 부르게 됨) |
| possibly-breaking (Cargo Book) / lower bound (요구 사항) / Hyrum's Law | 깨질 수도 있음 / 아래쪽 경계 / 하이럼의 법칙 |
| directory source / (vendor로 만든 가짜 crates.io) / changelog / release / publish | 디렉터리 소스 / 연습용 레지스트리 / 변경 기록 / 릴리스 / 게시 |
| conditional compilation / cfg predicate / cfg option / custom cfg / fallback arm | 조건부 컴파일 / cfg 조건 / cfg 이름·값 / 사용자 정의 cfg / 나머지 갈래 |
| additive feature / mutually exclusive features / feature-gate / implicit feature / feature combinations | 더하기만 하는 기능(처음 한 번 "가산적, additive") / 배타적인 기능(서로 함께 켤 수 없는 기능) / 기능으로 가리다 / 암묵적 기능 / 기능 조합 |
| public dependency / private dependency / renaming dependencies (`package` 키) / intermediate wrapper crate | 공개 의존성 / 비공개 의존성 / 의존성 이름 바꾸기 / 중간 래퍼 크레이트 |
| dependency graph / transitive dependency / direct dependency / resolve / backtrack | 의존성 그래프 / 전이 의존성 / 직접 의존성 / 의존성 해석 / 되돌아가다 |
| duplicate versions / (그래프에 들어온 크레이트의 한 버전) / lockfile / supply chain | 중복 버전 / 사본(ch17의 "복사해 둔 값"과 같은 낱말, 뜻은 문맥으로 구별) / 잠금 파일(`Cargo.lock`) / 공급망 |
| min-publish-age (dependency cooldown) / security advisory / RustSec Advisory Database | 최소 게시 경과 시간 / 보안 권고 / RustSec 권고 데이터베이스 |
| (부록 A ch27) intra-doc link / crate-level docs / doc lint / hidden line | 항목 링크 / 크레이트 문서 / 문서 린트 / 숨김 줄. 문서 주석의 절 제목(`# Examples`·`# Panics`·`# Errors`·`# Safety`)은 영어 그대로 |
| abstraction ladder / thin macro / mixed-site hygiene / helper attribute / definition site·call site (매크로) | 추상화 사다리 / 얇은 매크로 / 혼합 위생 / 보조 속성 / 정의한 곳·부른 곳. 매크로가 만든 코드를 손으로 적어 보인 것은 "펼친 모양" |
| unfulfilled lint expectation / default level / inherit (`workspace = true`) | 충족되지 않은 기대 / 기본 수준 / 물려받다. 경고 처리의 세 갈래는 "고침 / 작은 리팩터링 / 이유를 적고 예외로 둠" |
| early-warning test / reproduction test / regression test / example program / property-based test | 조기 경보 테스트 / 재현 테스트 / 회귀 테스트 / 예제 프로그램(라이브러리 패키지의 `examples/`. 챕터의 "예제 0N"과 구별) / 성질 기반 테스트 |
| fuzzing / fuzzer / fuzz target / corpus / coverage-guided / baseline (criterion) | 퍼징 / 퍼저 / 퍼즈 타깃 / 말뭉치 / 커버리지 유도 / 기준선(Part 2의 "기준선(2015 에디션)"과 같은 낱말, 뜻은 문맥으로 구별) |
| (사본에 버그나 문제를 하나씩 넣어 검사가 잡는지 보는 실험) | 변경 실험(ch30), 같은 방식의 ch32 표는 "넣은 문제" |
| fix mode / check mode / proxy (rustup) / human-readable·machine-readable output / verdict | 고치는 모드 / 검사 모드 / 프록시 / 사람용 출력·기계용 출력 / 판정 |
| (도구나 검사를 실행하는 자리와 주기) | 둘 곳: 편집기 / 리뷰 요청마다 / 메인 병합마다(둘을 묶어 "변경마다") / 정기 예약 / 시간을 정해 / 게시 전 |
| step (스크립트) / job / workflow / matrix / runner / required check / flaky test | 단계 / 작업 / 워크플로 / 매트릭스 / 러너 / 필수 검사 / 불안정한(flaky) 테스트 |
| fail-fast / keep-going / reduced set (기능 조합) | 첫 실패에서 멈춤 / 끝까지 돌고 요약 / 줄인 집합(없음 + 하나씩 + 전부) |
| check crate (`#[panic_handler]`로 std 유입을 잡는 크레이트) / budget allocator / overcommit | 검사용 크레이트 / 예산 할당기 / 오버커밋. 할당 실패의 abort는 "중단"(패닉과 구별) |
| FFI boundary / declaration (extern 블록 항목)·definition / bare name / calling convention / status code / prefix | FFI 경계 / 선언·정의 / 맨 이름(맹글링되지 않은 이름) / 호출 규약 / 상태 코드 / 접두사 |
| opaque type·pointer / out-parameter / borrowed·transferred (C 문자열의 소유권 방향) | 불투명 타입·불투명 포인터 / 출력 매개변수 / 빌려줌·넘겨줌 |
| hand-written·generated declarations / committed bindings / layout check / target version (`rust_target`) / bless | 손으로 쓴 선언·생성한 선언("손으로 쓴 판"·"생성판") / 커밋해 둔 생성 결과(↔ 빌드 때 생성) / 레이아웃 검사 / 대상 판 / 덮어쓰기 |
| (선언이 어긋났지만 메모리는 망가지지 않아 값만 틀리게 읽힌 것) | 잘못 해석한 값(미정의 동작과 구별) |
| (Part 3) target / auto-discovery / convention over configuration | 타깃 / 관례로 찾아냄(자동 탐색) / 설정보다 관례 |
| (Part 3) registry / index / Cargo home / lockfile / `.crate` 파일 | 레지스트리 / 색인 / Cargo home(영어 그대로) / 잠금 파일 / 압축 파일. 저장소 밖으로 복사해 실험하는 것은 "사본" |
| (Part 3) version requirement / default·caret / tilde / wildcard / comparison / pre-release | 버전 요구 사항 / 기본 요구 사항(= 캐럿 요구 사항) / 틸드 / 와일드카드 / 비교 / 프리릴리스. 버전의 세 자리는 "첫째·둘째·셋째 자리" |
| (Part 3) source (의존성을 받아 오는 곳) / source replacement / resolver / platform-specific dependencies | 출처 / 소스 교체 / 리졸버 / 플랫폼별 의존성. `[patch]`로 끼워 넣는 고친 크레이트는 "고친 사본", 실험용 복사본은 "저장소 밖에 복사한 …" |
| (Part 3) TOML table / array of tables / target table / auto-discovery / test harness | 표 / 표 배열 / 타깃 표 / 자동 탐색 / 테스트 러너(libtest harness). Part 3 ch3부터 매니페스트의 `[…]`는 "표"로 부름(앞 챕터의 "빈 `[workspace]` 테이블"과 같은 것) |
| (Part 3) MSRV / lint priority | 최소 지원 Rust 버전(MSRV) / 우선순위(`priority`). 수준은 allow·warn·deny·forbid를 영어 그대로 |
| (Part 3) feature / default feature / optional dependency / implicit feature / weak dependency feature (`?`) / feature unification / additive | 기능 / 기본 기능 / 선택적 의존성 / 암묵적 기능 / (`?` 표기는 풀어 씀: "이미 켜져 있을 때만") / 기능 통합 / 가산적(더하기만 하는) |
| (Part 3) configuration (`cfg`) / configured out | 구성 / 조건 때문에 빠진 |
| (Part 3) built-in·custom profile / override / build override / link time optimization / codegen unit / incremental compilation / debug info / symbol table / inline | 내장·사용자 정의 프로필 / 재정의 / `build-override`(키 이름 그대로) / 링크 시점 최적화(LTO) / 조각(설정 이름 `codegen-units`) / 증분 컴파일 / 디버그 정보 / 심볼 표 / 인라인 |
| (Part 3) build cache / target directory / build directory / final·intermediate artifact / dep-info / unit (timings) / critical path / frontend | 빌드 캐시 / target 디렉터리 / build 디렉터리 / 최종 산출물·중간 산출물 / `.d` 파일(dep-info) / 단위 / 가장 긴 의존 경로(critical path) / 프런트엔드 |
