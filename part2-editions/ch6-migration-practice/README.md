# ch6. 마이그레이션 실습: `cargo fix --edition`으로 2018 → 2021 → 2024 옮기기

Part 2의 마지막 챕터입니다. [ch1](../ch1-editions-overview/README.md)에서 에디션이라는 제도를, [ch2](../ch2-rust-2015-2018/README.md)~[ch5](../ch5-rust-2024-ecosystem/README.md)에서 에디션마다 **무엇이 바뀌었는지**를 보았습니다. 이 챕터는 그 지식을 **실제 프로젝트를 어떻게 옮기는가**로 묶습니다. 2018 에디션으로 쓴 작은 가계부 프로젝트(라이브러리 + 바이너리 + 테스트)를 Edition Guide의 절차대로 2021로, 다시 2024로 옮기면서 다음을 직접 확인합니다.

- `cargo fix --edition`이 실제로 무엇을 고치고, 무엇을 고치지 못하는지
- 자동 수정의 결과를 사람이 어디서 다시 판단해야 하는지
- 테스트가 매 단계에서 "동작이 그대로인지"를 알려 주는 안전망이 된다는 것

- 패키지: `p2-ch6-migration-practice`
- 기준 문서: [Edition Guide 1.2 Transitioning an existing project to a new edition](https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html), [1.3 Advanced migrations](https://doc.rust-lang.org/edition-guide/editions/advanced-migrations.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, rustfmt 1.11.0, git 2.55.0, macOS (aarch64)
- 실습 자료: [`projects/migration`](projects/migration/step0-2018/Cargo.toml)(단계별 스냅숏 세 벌과 `cargo fix` 직후의 diff 두 개), `examples/` 2개(옮긴 뒤의 정리), `compile_fail/` 3개

> **`cargo fix`는 소스 파일을 직접 고칩니다.** 이 챕터의 실습은 모두 `projects/migration/step0-2018`을 저장소 **밖**(예: `/tmp/rust-p2-ch6`)으로 복사한 사본에서 합니다. 이 학습 저장소 안에서 `cargo fix`를 실행하지 마세요.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `cargo fix --edition`이 하는 일(다음 에디션의 에디션 호환성 린트 그룹을 `--force-warn`으로 켠 `cargo check`와 제안의 자동 적용)을 설명하고, 고치지 않고 경고만 미리 보는 명령을 쓸 수 있다.
2. Edition Guide의 절차(깨끗한 커밋 → `cargo update` → `cargo fix --edition` → `edition` 올리기 → `cargo build`/`cargo test` → `cargo fmt`는 따로 커밋)대로 프로젝트를 한 에디션씩 옮기고, 테스트로 동작이 그대로인지 확인할 수 있다.
3. 자동 수정의 결과를 "그대로 받아들일 것 / 사람이 다시 판단할 것 / 자동 수정이 없는 것"으로 나누고, 더미 let, 기계적으로 바뀐 `match`, `+ use<>`, `expr_2021`, `unsafe { set_var }`를 상황에 맞게 다듬을 수 있다.
4. `cargo fix`가 보지 못하는 코드(내보내기만 한 매크로의 본문, 문서 테스트, 꺼진 기능과 다른 플랫폼의 코드, 빌드 스크립트가 만든 코드)와 린트가 잡지 못하는 동작 변화를 알고 직접 찾을 수 있다.
5. `--all-features`·`--target`, 대상 선택, `--broken-code`, `--edition-idioms`, Cargo 없이 `rustc`로 린트 그룹 켜기 같은 고급 마이그레이션 방법을 고를 수 있다.

## 2. 핵심 개념

### 2.1 이 챕터의 지도

| 절 | 내용 | 실습 |
| --- | --- | --- |
| 2.2 | `cargo fix --edition`이 하는 일, 고치지 않고 미리 보기 | `step0-2018` 사본 |
| 2.3 | 실습 프로젝트와 심어 둔 변경 | `projects/migration/step0-2018` |
| 2.4 | 준비: 사본, 깨끗한 커밋, `cargo update` | |
| 2.5 | 1단계 2018 → 2021: 자동 수정 → `edition` 올리기 → 손으로 고치기 → 다듬기 | 결과 `step1-2021` |
| 2.6 | 2단계 2021 → 2024: 같은 절차 + `cargo fmt`(스타일 에디션) | 결과 `step2-2024` |
| 2.7 | 사람이 고쳐야 하는 경우 정리 | 표, 예제 01·02 |
| 2.8 | 고급 마이그레이션 (Edition Guide 1.3) | 사본 실험 |

### 2.2 `cargo fix --edition`이 하는 일

**린트의 제안을 소스에 적용합니다.** Edition Guide 1.3절의 설명은 이렇습니다. `cargo fix --edition`은 다음 에디션에서 컴파일되지 않을 코드를 찾는 특별한 린트를 켜고 `cargo check`에 해당하는 일을 합니다. 이 린트들은 **지금 에디션과 다음 에디션에서 모두 컴파일되는** 모양으로 고치는 제안(suggestion)을 내고, `cargo fix`가 그 제안을 소스에 적용한 뒤 다시 `cargo check`로 확인합니다. 적용이 새 경고를 낳으면 경고가 없어질 때까지 되풀이합니다. 그 특별한 린트가 ch3~ch5의 표에 나온 에디션 호환성 린트 그룹입니다.

| 지금 에디션 | `cargo fix --edition`이 켜는 그룹 | 들어 있는 린트 수 | 정리한 곳 |
| --- | --- | --- | --- |
| 2015 | `rust-2018-compatibility` | 4 | [ch2](../ch2-rust-2015-2018/README.md) |
| 2018 | `rust-2021-compatibility` | 8 | [ch3](../ch3-rust-2021/README.md) 2.1절 |
| 2021 | `rust-2024-compatibility` | 14 | [ch4](../ch4-rust-2024-language/README.md) 2.1절, [ch5](../ch5-rust-2024-ecosystem/README.md) 2.1절 |

린트 수는 `rustc -W help`의 "Lint groups" 목록에서 셌습니다. 그룹을 켜는 방식은 `cargo fix`가 출력하는 경고의 마지막 note에서 보입니다(2.6절의 출력).

```text
   = note: `--force-warn tail-expr-drop-order` implied by `--force-warn rust-2024-compatibility`
```

`--force-warn`은 소스의 `#[allow(...)]`보다 우선하는 린트 수준입니다. 그래서 `cargo fix --edition`은 **경고를 `#[allow]`로 막아 둔 코드도 고칩니다**(2.5절에서 확인합니다). 린트 수준과 그 우선순위는 Part 5 `ch3-lints`에서 다룹니다.

**한 번에 한 에디션씩, `edition` 값은 그대로.** 출력의 `Migrating src/lib.rs from 2018 edition to 2021`처럼 `cargo fix --edition`은 `Cargo.toml`에 적힌 에디션의 **바로 다음 에디션**으로만 옮깁니다. 그리고 `edition` 값은 바꾸지 않습니다(`cargo help fix`의 `--edition` 설명: "This will not update the edition in the Cargo.toml manifest"). 고친 코드는 두 에디션에서 모두 컴파일되므로, 그 상태로 커밋하고 테스트한 뒤 사람이 `edition`을 올립니다. 2018에서 2024로 가려면 이 과정을 두 번 거칩니다(2.8절).

**`Cargo.toml`도 고칩니다.** 출력의 첫 줄 `Migrating Cargo.toml from 2018 edition to 2021`은 매니페스트를 검사한다는 뜻입니다. 2024로 옮길 때는 [ch5](../ch5-rust-2024-ecosystem/README.md) 2.7절의 옛 키 이름과 상속 `default-features = false`를 고칩니다. 이 실습의 매니페스트에는 고칠 것이 없어 `Fixed Cargo.toml`이 나오지 않았고, 옛 키를 넣은 2021 에디션 임시 패키지로 따로 확인한 결과는 다음과 같습니다(`Fixed Cargo.toml (4 fixes)`).

```diff
-[project]
+[package]
 ...
-[dev_dependencies]
-helper = { path = "helper", default_features = false }
+[dev-dependencies]
+helper = { path = "helper", default-features = false }
 ...
-crate_type = ["lib"]
+crate-type = ["lib"]
```

워크스페이스에서 물려받는 의존성의 `helper = { workspace = true, default-features = false }`는 `helper = { workspace = true}`로 바뀌었습니다(닫는 중괄호 앞의 공백까지 출력 그대로). 2021에서 무시되던 설정을 지워 예전 동작을 지키는 것입니다.

**고치지 않고 미리 보기.** 무엇이 바뀔지 먼저 보고 싶으면 같은 린트 그룹을 켜고 `cargo check`만 합니다. 실습 프로젝트의 2018판(2.3절)에서 실행한 결과입니다.

| 명령 | 보이는 경고 |
| --- | --- |
| `RUSTFLAGS="--force-warn rust-2021-compatibility" cargo check --all-targets --all-features` | 10개(`cargo fix`가 고치는 10곳과 같음) |
| `cargo rustc --lib --all-features -- --force-warn rust-2021-compatibility` | 라이브러리의 10개 |
| `RUSTFLAGS="-W rust-2021-compatibility" cargo check --all-targets --all-features` | 2개뿐 |

세 번째 줄은 `-W`(warn)가 소스의 `#[allow]`에 지기 때문에 막아 둔 8곳이 보이지 않은 것입니다. 미리 볼 때는 `--force-warn`을 씁니다. `RUSTFLAGS`는 모든 크레이트의 컴파일 옵션을 바꿔 처음부터 다시 빌드하게 만들고, `cargo rustc`는 한 대상(`--lib`, `--bin 이름` 등)만 다룹니다. 크레이트 루트에 `#![warn(rust_2021_compatibility)]`를 적는 방법도 있지만 `-W`와 같이 `#[allow]`에 집니다. 2021판에서 `--force-warn rust-2024-compatibility`로 미리 보면 경고가 11개 나오고, 그중 10개를 `cargo fix`가 고쳤습니다(2.6절).

### 2.3 실습 프로젝트: `projects/migration`

```text
projects/migration/
├── step0-2018/         출발점. edition = "2018"
│   ├── Cargo.toml      [features] csv = [] (기본으로 꺼짐), 빈 [workspace]
│   ├── src/lib.rs      크레이트 문서(문서 테스트), Config, Budget, C용 함수, 매크로 book!·budget!
│   ├── src/entry.rs    Category, Entry::parse, 사용자 트레이트 Percent
│   ├── src/report.rs   Style 트레이트(Plain, Bar), grade, render
│   ├── src/book.rs     Ledger, IdGen, 감사 기록(Audit, Scope)
│   ├── src/csv.rs      CSV 내보내기 — 기능 csv를 켰을 때만 컴파일
│   ├── src/main.rs     예시 자료로 월말 보고서를 출력하는 바이너리
│   └── tests/ledger.rs 통합 테스트 12개(기능 csv를 끄면 11개)
├── step1-2021/         2021로 옮기고 다듬은 결과
├── step2-2024/         2024로 옮기고 다듬고 서식한 결과
└── diffs/              cargo fix --edition 직후의 git diff 두 개
```

가계부는 `"식비 32000 장보기"` 같은 줄을 읽어 카테고리별 합계와 비율, 등급을 보고서로 만듭니다. `Audit`은 "정산 시작", "합산 끝" 같은 사건을 순서대로 기록하는 감사 기록이고, `Scope`는 버려질 때 "끝"을 기록하는 값입니다. 테스트는 이 기록의 **순서까지** 단언하므로, 에디션이 drop 순서를 바꾸면 테스트가 알려 줍니다.

**심어 둔 변경.** 앞 챕터들이 가르친 변경이 실제로 걸리도록 코드를 넣었습니다. 부류는 실제로 `cargo fix --edition`을 돌려 본 결과로 나눴습니다.

| 옮기는 단계 | 그대로 받아들인 자동 수정 | 자동 수정을 다시 판단한 것 | 자동 수정이 없거나 보지 못한 것 |
| --- | --- | --- | --- |
| 2018 → 2021 | `dyn` 없는 트레이트 객체(4곳), `...` 범위 패턴(2곳), 배열의 `.into_iter()`, 사용자 `try_into(&self)` | 클로저의 더미 let, 인자가 하나뿐인 `panic!` | 내보내기만 한 매크로 본문의 `panic!(e)`, 문서 테스트의 `Box<report::Style>`, 기능 `csv`의 `&mut io::Write`(`--all-features` 없이) |
| 2021 → 2024 | `gen` 식별자(2곳), `extern` 블록, `#[no_mangle]` | `if let` → `match`, `+ use<>`, `expr_2021`(2곳), `unsafe { set_var }`(2곳) | 마지막 표현식의 임시값(`tail_expr_drop_order`), 문서 테스트의 `set_var` |

**막아 둔 경고.** `step0-2018`은 2018 시절의 모양을 일부러 남기느라 나는 경고를 `#[allow(...)]`와 이유 주석으로 막았습니다. 막은 린트는 `bare_trait_objects`(4곳), `ellipsis_inclusive_range_patterns`, `array_into_iter`와 `clippy::into_iter_on_ref`, `non_fmt_panics`입니다. 이 `#[allow]` 덕분에 `step0-2018`도 다른 두 스냅숏처럼 그 디렉터리에서 `cargo clippy --all-targets --all-features -- -D warnings`와 `cargo fmt --check`가 통과합니다. 오래된 프로젝트에 이런 `#[allow]`가 남아 있는 것은 흔한 일이고, `cargo fix --edition`은 이것들을 무시하고 고친다는 것을 2.5절에서 봅니다.

**실행해 보기.** 저장소의 스냅숏은 읽기 전용으로 쓰고 실행만 합니다. 세 스냅숏의 `cargo run` 출력은 2.6.3절에서 비교합니다.

```sh
cd part2-editions/ch6-migration-practice/projects/migration/step0-2018
cargo run
cargo test --all-features
```

```text
== 4건 정산 ==
식비   28000원  57%
교통   12500원  25%
기타    8000원  16%
합계 48500원 (알뜰)
(금액 단위: 원)
예산: 식비 true, 기타 true
기존 번호 [1, 2, 3, 4], 새 번호 5
감사 기록:
  정산 시작
  잔액 시작
  합산 시작
  잔액 끝
  합산 끝
  보고서 완성
  정산 끝
```

### 2.4 준비: 사본, 깨끗한 커밋, `cargo update`

Edition Guide 1.2절의 절차는 다섯 단계입니다. ① `cargo update`로 의존성을 최신으로 → ② `cargo fix --edition` → ③ `Cargo.toml`의 `edition`을 다음 에디션으로 → ④ `cargo build`나 `cargo test`로 확인 → ⑤ `cargo fmt`. 그 전에 **되돌릴 수 있는 상태**를 만듭니다.

```sh
mkdir -p /tmp/rust-p2-ch6
cp -R part2-editions/ch6-migration-practice/projects/migration/step0-2018 /tmp/rust-p2-ch6/ledger
cd /tmp/rust-p2-ch6/ledger
git init
git add -A
git commit -m "2018 에디션"
```

스냅숏에는 `/target`을 무시하는 `.gitignore`와 `Cargo.lock`이 들어 있습니다. git에 이름과 메일을 설정하지 않았다면 `git -c user.name=나 -c user.email=me@example.invalid commit …`처럼 그 명령에만 값을 줄 수 있습니다.

**왜 커밋부터 하나.** `cargo fix`는 작업 트리에 커밋하지 않은 변경이 있으면 시작하지 않습니다. 자동 수정이 내 변경과 섞이면 무엇을 도구가 바꿨는지 가려낼 수 없기 때문입니다. 이 실습에서 나온 메시지 세 가지입니다(`src/main.rs`에 한 줄을 더해 확인).

```text
error: no VCS found for this package and `cargo fix` can potentially perform destructive changes; if you'd like to suppress this error pass `--allow-no-vcs`
```

```text
error: the working directory of this package has uncommitted changes, and `cargo fix` can potentially perform destructive changes; if you'd like to suppress this error pass `--allow-dirty`, or commit the changes to these files:

  * src/main.rs (dirty)
```

`git add`만 하고 커밋하지 않았으면 `* src/main.rs (staged)`로 나옵니다.

| 옵션 | 뜻 |
| --- | --- |
| `--allow-no-vcs` | git 같은 버전 관리 도구가 없어도 고침 |
| `--allow-staged` | 스테이지에 올린(`git add`한) 변경이 있어도 고침. 올리지 않은 변경이 있으면 여전히 거절 |
| `--allow-dirty` | 스테이지 여부와 상관없이 변경이 있어도 고침 |

**추적하지 않는 파일도 "변경"입니다.** `git add`하지 않은 새 파일(`NOTES.md`)이 하나만 있어도 `* NOTES.md (dirty)`로 거절되었습니다. 처음 빌드할 때 생긴 `Cargo.lock`을 커밋하지 않아 이 메시지를 만나는 경우가 많습니다.

**① `cargo update`.** 의존성의 새 버전이 새 에디션과의 호환 문제를 이미 고쳐 두었을 수 있으므로 먼저 올립니다. 이 프로젝트는 의존성이 없어서 `Locking 0 packages to highest compatible versions`만 나오고 바뀌는 것이 없습니다. 실제 프로젝트라면 이 단계에서 `cargo test`를 실행하고, 바뀐 `Cargo.lock`을 따로 커밋합니다. 그리고 **옮기기 전에 테스트가 모두 통과하는지** 확인해 둡니다. 이것이 이후 모든 단계의 비교 기준입니다.

```sh
cargo update
cargo test --all-features     # 통합 테스트 12개, 문서 테스트 2개 통과
```

### 2.5 1단계: 2018 → 2021

#### 2.5.1 `cargo fix --edition`

이 프로젝트에는 기능 `csv`가 있으므로 `--all-features`를 붙입니다(빼면 무슨 일이 생기는지는 2.8절).

```sh
cargo fix --edition --all-features
```

```text
   Migrating Cargo.toml from 2018 edition to 2021
    Checking ledger v0.1.0 (/tmp/rust-p2-ch6/ledger)
   Migrating src/lib.rs from 2018 edition to 2021
       Fixed src/report.rs (5 fixes)
       Fixed src/entry.rs (1 fix)
       Fixed src/csv.rs (1 fix)
       Fixed src/book.rs (3 fixes)
   Migrating src/main.rs from 2018 edition to 2021
   Migrating tests/ledger.rs from 2018 edition to 2021
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s
```

(`Checking` 줄의 경로는 독자용 경로로 바꿔 적었습니다. `Fixed` 줄의 순서와 시간은 실행마다 다를 수 있습니다.) 크레이트마다 `Migrating` 줄이 나옵니다. 라이브러리(`src/lib.rs`와 그 모듈들), 바이너리(`src/main.rs`), 통합 테스트(`tests/ledger.rs`)는 각각 다른 크레이트입니다. 고친 곳은 모두 라이브러리에 있었습니다. 10곳 중 8곳은 `#[allow]`로 경고를 막아 둔 자리였고, 나머지 2곳(클로저와 `try_into`)은 기본 수준이 allow인 린트가 찾은 자리입니다. `git diff` 전체는 [`diffs/step0-cargo-fix-2021.diff`](projects/migration/diffs/step0-cargo-fix-2021.diff)에 있습니다. 직접 실행했다면 `git diff | diff - <저장소>/…/diffs/step0-cargo-fix-2021.diff`로 비교할 수 있습니다(`<저장소>`는 이 저장소의 경로). 핵심을 고르면 다음과 같습니다.

```diff
-pub fn style_for(name: &str) -> Box<Style> {
+pub fn style_for(name: &str) -> Box<dyn Style> {
-        0...99_999 => "알뜰",
+        0..=99_999 => "알뜰",
-            .into_iter()
+            .iter()
             .find(|c| c.name() == name)
             .copied()
-            (sum * 100 / total).try_into().ok()
+            Percent::try_into(&(sum * 100 / total)).ok()
-            Err(_) => panic!("잘못된 줄: {line}"),
+            Err(_) => panic!("{}", "잘못된 줄: {line}"),
-        let header = move || format!("== {} ==", job.title);
+        let header = move || { let _ = &job; format!("== {} ==", job.title) };
```

모든 수정의 목표는 **2018 에디션의 동작을 그대로 지키는 것**입니다.

- `.iter()`: 2018의 배열 `.into_iter()`는 참조를 내놓았으므로([ch3](../ch3-rust-2021/README.md) 2.4절) 모든 에디션에서 참조를 내놓는 `.iter()`로 바꿨습니다.
- `Percent::try_into(&…)`: 2021 프렐류드의 `TryInto`와 이름이 겹치는 사용자 메서드를 트레이트 이름으로 부르게 했습니다([ch3](../ch3-rust-2021/README.md) 2.2절). 이 수정이 없으면 2021에서 **조용히** `TryInto::try_into`가 불립니다(2.8절에서 확인).
- `panic!("{}", …)`: 2018에서 인자가 하나뿐인 `panic!`은 서식 처리를 하지 않았으므로 `{line}`이라는 **글자 그대로**가 메시지였습니다. 자동 수정은 그 동작을 지킵니다. 2.5.3절에서 다시 판단합니다.
- 더미 let `let _ = &job;`: 클로저가 `job` 전체를 캡처하게 해 2018과 같은 drop 시점을 지킵니다([ch3](../ch3-rust-2021/README.md) 2.5절).

자동 수정이 넣은 코드는 rustfmt로 서식하지 않은 상태입니다. 이 시점에 `cargo fmt --check`를 하면 더미 let 줄에서 차이가 납니다. 이 상태를 커밋합니다.

```sh
git commit -am "cargo fix --edition (2018 → 2021)"
```

#### 2.5.2 `edition = "2021"`로 올리고 확인

`Cargo.toml`의 `edition = "2018"`을 `edition = "2021"`로 바꾸고 빌드합니다.

```text
$ cargo build
error: format argument must be a string literal
  --> src/main.rs:13:20
   |
13 |       let mut book = book![
   |  ____________________^
14 | |         "식비 32000 장보기",
...
18 | |     ];
   | |_____^
   |
   = note: this error originates in the macro `book` (in Nightly builds, run with -Z macro-backtrace for more info)
help: you might be missing a string literal to format with
  --> src/lib.rs:71:24
   |
71 |                 panic!("{}", e);
   |                        +++++
```

(14~18번 줄의 소스 인용은 줄였습니다.) **자동 수정이 보지 못한 매크로 본문**입니다. `book!`은 `#[macro_export]`로 내보낸 매크로이고, 본문에 2018 시절의 `panic!(e)`가 있습니다. 그런데 라이브러리 자신은 `book!`을 한 번도 부르지 않습니다. 그래서 라이브러리를 검사할 때는 펼쳐진 `panic!(e)`가 없어 린트가 경고할 일이 없었고, `book!`을 부르는 바이너리와 테스트에서는 매크로가 다른 크레이트의 것이라 린트가 보고되지 않았습니다. 에디션을 올리자 본문의 토큰이 2021 규칙으로 읽혀([ch1](../ch1-editions-overview/README.md) 2.8절) 매크로를 **부른 쪽**에서 에러가 났습니다. `help:`가 고칠 곳(`src/lib.rs`)을 알려 주므로 그대로 `panic!("{}", e)`로 고칩니다.

다시 빌드하면 통과합니다. 이제 테스트입니다.

```text
$ cargo test --all-features
…
   Doc-tests ledger

running 2 tests
test src/lib.rs - (line 3) ... FAILED
test src/lib.rs - Config::from_env (line 31) ... ok

failures:

---- src/lib.rs - (line 3) stdout ----
error[E0782]: expected a type, found a trait
 --> src/lib.rs:9:16
  |
9 | let style: Box<report::Style> = report::style_for("plain");
  |                ^^^^^^^^^^^^^
  |
help: you can add the `dyn` keyword if you want a trait object
  |
9 | let style: Box<dyn report::Style> = report::style_for("plain");
  |                +++
…
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
```

(통합 테스트 12개는 통과했고, 앞뒤 출력은 줄였습니다.) **자동 수정은 문서 테스트를 고치지 않습니다.** 크레이트 문서의 예제가 2018 시절의 `Box<report::Style>`을 쓰고 있었습니다. 문서 테스트는 크레이트의 에디션으로 컴파일되므로 2021에서 E0782가 됩니다. 소스의 `Box<Style>`은 고쳐졌는데 문서 주석 속 같은 모양은 그대로였다는 점에 주의합니다. `Box<dyn report::Style>`로 고치면 모두 통과합니다. 여기까지를 커밋합니다.

```sh
git commit -am "edition = 2021, 매크로 본문과 문서 테스트를 손으로 고침"
```

#### 2.5.3 다듬기

컴파일되고 테스트가 통과하는 지금이 다듬기 좋은 때입니다. 자동 수정은 "2018과 같은 동작"을 지키려고 보수적으로 고쳤으므로, 2021에서 더 자연스러운 모양으로 바꿀 곳을 찾습니다. 바꾼 뒤마다 `cargo test --all-features`로 동작을 확인합니다. 다듬은 결과가 [`step1-2021`](projects/migration/step1-2021/Cargo.toml)입니다.

| 다듬은 곳 | 판단 |
| --- | --- |
| 더미 let 지우기 | `header` 클로저는 함수 끝까지 살아 있으므로 `job.title`만 캡처해도 "정산 끝"이 기록되는 시점이 같습니다. 지운 뒤에도 `close_month_audit` 테스트가 통과했습니다. 클로저를 먼저 버리는 코드였다면 순서가 바뀝니다(예제 01의 1·2번 구역) |
| `panic!("{}", "잘못된 줄: {line}")` → `panic!("잘못된 줄: {line}")` | 2018의 메시지 `잘못된 줄: {line}`은 버그였습니다. 원래 의도대로 줄 내용을 넣고, 테스트의 `should_panic(expected = …)`도 `"잘못된 줄: 식비 abc"`로 강화했습니다. "예전과 같게"와 "원래 의도대로"를 구분하는 자리입니다 |
| `.iter()…copied()` → `.into_iter()` | 2021에서는 배열의 `.into_iter()`가 값을 내놓으므로 `.copied()`가 필요 없습니다 |
| `use std::convert::TryFrom;` 지우기 | 2021 프렐류드에 있습니다. 남겨 두어도 경고는 나지 않았습니다 |
| `#[allow(...)]`와 이유 주석 지우기 | 막을 경고가 더는 없습니다. 남겨 두어도 경고는 나지 않지만, 다음에 같은 경고가 생겨도 보이지 않게 됩니다 |

`Percent::try_into(&…)`는 그대로 두었습니다. 사용자 트레이트를 부르는 뜻을 정확히 드러내는 모양이기 때문입니다.

**⑤ `cargo fmt`.** 2018과 2021은 같은 스타일(2015 스타일 에디션)을 쓰므로([ch5](../ch5-rust-2024-ecosystem/README.md) 2.10절) 더미 let을 지운 뒤에는 바뀌는 것이 없었습니다. 마지막으로 `cargo clippy --all-targets --all-features -- -D warnings`가 통과하고 `cargo run`의 출력이 2018판과 한 글자도 다르지 않은 것을 확인한 뒤 커밋합니다. 이 디렉터리와 `step1-2021`을 비교하면(`diff -r -x .git -x target /tmp/rust-p2-ch6/ledger <저장소>/…/step1-2021`) 차이가 없어야 합니다.

### 2.6 2단계: 2021 → 2024

1단계를 직접 따라 하지 않았다면 `step1-2021`을 새로 복사해 같은 준비(2.4절)를 하고 시작해도 됩니다.

#### 2.6.1 `cargo fix --edition`

```text
$ cargo fix --edition --all-features
   Migrating Cargo.toml from 2021 edition to 2024
    Checking ledger v0.1.0 (/tmp/rust-p2-ch6/ledger)
   Migrating src/lib.rs from 2021 edition to 2024
       Fixed src/lib.rs (3 fixes)
       Fixed src/book.rs (4 fixes)
warning: relative drop order changing in Rust 2024
   --> src/book.rs:129:9
    |
128 |           let _scope = self.audit.enter("잔액");
    |               ------
    |               |
    |               `_scope` calls a custom destructor
    |               `_scope` will be dropped later as of Edition 2024
129 | /         self.audit
130 | |             .enter("합산")
    | |                          ^
    | |                          |
    | |__________________________this value will be stored in a temporary; let us call it `#1`
    |                            up until Edition 2021 `#1` is dropped last but will be dropped earlier in Edition 2024
131 |               .run(|| self.entries.iter().map(|e| e.amount).sum())
132 |       }
    |       - now the temporary value is dropped here, before the local variables in the block or statement
…
    = note: most of the time, changing drop order is harmless; inspect the `impl Drop`s for side effects like releasing locks or sending messages
    = warning: this changes meaning in Rust 2024
    = note: `--force-warn tail-expr-drop-order` implied by `--force-warn rust-2024-compatibility`

warning: `ledger` (lib) generated 1 warning
   Migrating src/main.rs from 2021 edition to 2024
   Migrating tests/ledger.rs from 2021 edition to 2024
       Fixed tests/ledger.rs (3 fixes)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.37s
```

(note 몇 줄과 `(lib test)` 줄은 줄였습니다.) **고친 곳은 조용히, 고치지 못한 곳은 경고로** 보입니다. 이번에 남은 경고는 [ch4](../ch4-rust-2024-language/README.md) 2.6절의 `tail_expr_drop_order`이고, 이 린트에는 자동 수정이 없습니다. 이 경고를 기억해 두면 다음 단계에서 테스트가 실패할 때 원인을 바로 찾을 수 있습니다. 10곳의 수정 전체는 [`diffs/step1-cargo-fix-2024.diff`](projects/migration/diffs/step1-cargo-fix-2024.diff)에 있습니다.

```diff
-    pub fn gen(&mut self) -> u32 {
+    pub fn r#gen(&mut self) -> u32 {
-    pub fn ids(&self) -> impl Iterator<Item = u32> {
+    pub fn ids(&self) -> impl Iterator<Item = u32> + use<> {
-        let total = if let Some(cached) = self.totals.lock().unwrap()[slot] {
+        let total = match self.totals.lock().unwrap()[slot] { Some(cached) => {
             cached
-        } else {
+        } _ => {
 ...
-        };
+        }};
-#[no_mangle]
+#[unsafe(no_mangle)]
-    ($limit:expr) => {
+    ($limit:expr_2021) => {
-extern "C" {
+unsafe extern "C" {
-    std::env::set_var("LEDGER_UNIT", "엔");
+    // FIXME: Audit that the environment access only happens in single-threaded code.
+    unsafe { std::env::set_var("LEDGER_UNIT", "엔") };
```

(`self.ids.gen()` → `self.ids.r#gen()`, `book!`의 `$line:expr` → `$line:expr_2021`, `remove_var` 쪽의 같은 수정은 생략했습니다.) 대응하는 설명은 [ch4](../ch4-rust-2024-language/README.md)의 2.11절(`gen`), 2.3절(`use<>`), 2.4절(`if let`), 2.8절(unsafe 속성), 2.12절(`expr`), 2.7절(`unsafe extern`)과 [ch5](../ch5-rust-2024-ecosystem/README.md) 2.5절(`set_var`)에 있습니다. Edition Guide의 예시는 `// TODO: Audit that …`이지만 1.100.0이 넣은 주석은 `// FIXME: Audit that …`이었습니다.

```sh
git commit -am "cargo fix --edition (2021 → 2024)"
```

#### 2.6.2 `edition = "2024"`로 올리고 확인

`edition = "2024"`로 바꾸면 `cargo build`는 바로 통과합니다. 테스트가 실패합니다.

```text
$ cargo test --all-features
…
test balance_audit ... FAILED
test close_month_audit ... FAILED

failures:

---- balance_audit stdout ----
thread 'balance_audit' (8067296) panicked at tests/ledger.rs:69:5:
assertion `left == right` failed
  left: ["잔액 시작", "합산 시작", "합산 끝", "잔액 끝"]
 right: ["잔액 시작", "합산 시작", "잔액 끝", "합산 끝"]
…
test result: FAILED. 10 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(스레드 ID `(8067296)`와 테스트가 끝나는 순서는 실행마다 다릅니다.) **컴파일은 되는데 동작이 바뀐 곳을 테스트가 잡았습니다.** 원인은 `cargo fix`가 경고만 남긴 그 자리입니다. `balance`는 지역 변수 `_scope`("잔액" 구간)를 만들고, 마지막 표현식에서 임시값("합산" 구간)을 만듭니다.

```rust
pub fn balance(&self) -> i64 {
    let _scope = self.audit.enter("잔액");
    self.audit
        .enter("합산")
        .run(|| self.entries.iter().map(|e| e.amount).sum())
}
```

2021에서는 마지막 표현식의 임시값이 지역 변수보다 **나중에** 버려져 바깥 구간이 안쪽 구간보다 먼저 끝났습니다. 2024에서는 임시값이 먼저 버려져 구간이 제대로 포개집니다. 여기서 사람이 판단합니다. 이 프로젝트에서는 **새 순서가 옳다**고 보고 두 테스트의 기대값을 바꿨습니다. 반대로 예전 순서가 꼭 필요했다면 두 구간을 변수에 묶고 `drop`을 차례로 불러 순서를 코드에 직접 적습니다. 에디션 규칙에 기대지 않는 코드가 됩니다([ch4](../ch4-rust-2024-language/README.md) 2.6절). 자동 수정이 없는 변경은 이렇게 **경고를 읽고 테스트로 확인하는 것**이 유일한 방법입니다.

기대값을 고치고 다시 실행하면 이번에는 문서 테스트가 실패합니다.

```text
---- src/lib.rs - Config::from_env (line 31) stdout ----
error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block
  --> src/lib.rs:33:1
   |
33 | std::env::set_var("LEDGER_UNIT", "달러");
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
…
all doctests ran in 0.33s; merged doctests compilation took 0.03s
```

통합 테스트의 `set_var`는 `unsafe { }`로 감싸졌지만, 문서 테스트 속의 `set_var`는 그대로였습니다(마지막 줄은 [ch5](../ch5-rust-2024-ecosystem/README.md) 2.8절의 문서 테스트 병합이 켜졌다는 표시이고, 시간은 실행마다 다릅니다). 여기서 `FIXME` 주석이 묻는 질문, 즉 "이 환경 접근은 단일 스레드에서만 일어나는가"에 답해야 합니다. `cargo test`는 테스트를 여러 스레드에서 동시에 실행하므로 테스트 안의 `set_var`는 그 조건을 지킬 수 없습니다. 그래서 [ch5](../ch5-rust-2024-ecosystem/README.md) 2.5절의 표대로 **환경을 바꾸지 않고 값을 넘기게** 고쳤습니다.

```rust
impl Config {
    /// 환경 변수에서 읽습니다.
    pub fn from_env() -> Config {
        Config::from_lookup(|key| std::env::var(key).ok())
    }

    /// `lookup`이 돌려주는 값으로 만듭니다. 환경 변수를 바꾸지 않고 시험할 수 있습니다.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Config { … }
}
```

문서 테스트와 통합 테스트는 `Config::from_lookup(|_| Some(String::from("엔")))`처럼 바꿔 `unsafe`가 없어졌습니다. 모든 테스트가 통과하면 커밋합니다.

#### 2.6.3 다듬기와 `cargo fmt`

| 다듬은 곳 | 판단 |
| --- | --- |
| `if let` → `match`를 다시 `if let`으로 | `match`는 조건식의 잠금을 끝까지 쥐는 2021 동작을 지킵니다. 이 함수는 그럴 필요가 없고, 2024의 `if let`은 else 전에 잠금을 풀므로 오히려 else 안에서 바로 캐시를 채울 수 있게 됐습니다. 2021이었다면 잠금을 쥔 스레드가 같은 `Mutex`를 다시 잠그는 코드라서 교착 상태나 패닉이 됩니다(`Mutex::lock` 문서의 설명). 예제 02의 1번 구역이 잠금 시점의 차이를 보여 줍니다 |
| `+ use<>` 유지 | `main.rs`가 `book.ids()`의 반복자를 쥔 채 `book.push_line(..)`을 부릅니다. 지우면 아래 E0502가 납니다. 왜 붙었는지 문서 주석으로 남겼습니다 |
| `expr_2021`을 `expr`로 되돌림 | `book!`은 `_`나 `const { .. }`를 다르게 다룰 규칙이 없으므로 그냥 되돌렸습니다. `budget!`은 `(_)` 규칙을 앞으로 옮긴 뒤 되돌렸습니다. 옮기지 않고 되돌리면 `budget!(_)`이 에러입니다(4.3절) |
| `r#gen` → `next_id` | 우리 코드만 쓰는 메서드라 더 나은 이름으로 바꿨습니다. 다른 크레이트가 쓰는 공개 API였다면 이름을 바꾸는 것은 호환되지 않는 변경이므로, `r#gen`을 남기거나 새 이름을 더하고 옛 이름을 deprecated로 둡니다([ch4](../ch4-rust-2024-language/README.md) 2.11절의 `rand` 사례) |
| `#[unsafe(no_mangle)]`에 `SAFETY:` 주석 | 자동 수정은 감싸기만 합니다. 이 심벌 이름이 겹치지 않는 이유를 적었습니다 |
| 테스트의 `unsafe extern "C"` 블록 | 선언이 `lib.rs`의 함수와 맞는지 확인하고, 어떤 `i64`로 불러도 미정의 동작이 없으므로 `safe fn`으로 선언해 부르는 곳의 `unsafe { }`를 없앴습니다 |
| `balance`에 주석 | 2024의 drop 순서를 기대하는 코드라는 것을 남겼습니다 |

`+ use<>`를 지웠을 때의 에러입니다. 2024 컴파일러는 이 에러가 캡처 규칙 변경 때문일 수 있다고 알려 주고 `use<>`를 제안합니다.

```text
error[E0502]: cannot borrow `book` as mutable because it is also borrowed as immutable
   --> src/main.rs:34:18
    |
 33 |     let ids = book.ids();
    |               ---- immutable borrow occurs here
 34 |     let new_id = book.push_line("교통 1250 버스");
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
…
note: this call may capture more lifetimes than intended, because Rust 2024 has adjusted the `impl Trait` lifetime capture rules
help: if you can modify this crate, add a precise capturing bound to avoid overcapturing: `+ use<>`
```

다듬은 것을 커밋한 다음 **⑤ `cargo fmt`** 를 실행합니다. 2024 에디션은 스타일 에디션도 2024로 바꾸므로([ch5](../ch5-rust-2024-ecosystem/README.md) 2.10절) 이번에는 서식이 바뀝니다. Edition Guide의 권고대로 서식 변경만 담은 커밋을 따로 만듭니다. 그래야 리뷰에서 "코드 변경"과 "서식 변경"을 나눠 볼 수 있고, `git blame`에서 서식 커밋을 건너뛸 수도 있습니다.

```diff
-use ledger::{book, budget, report, Category, Config};
+use ledger::{Category, Config, book, budget, report};
-use crate::entry::{Percent, CATEGORIES};
+use crate::entry::{CATEGORIES, Percent};
```

(`tests/ledger.rs`의 `use`도 같은 식으로 바뀌어 세 파일이 바뀌었습니다.) [ch5](../ch5-rust-2024-ecosystem/README.md) 2.10절의 버전 정렬입니다. 2021 스타일은 소문자로 시작하는 이름, 대문자로 시작하는 이름, 모두 대문자인 이름 순서로 무리를 지어 정렬했고, 2024 스타일은 그런 무리 없이 글자 순서대로(대문자가 소문자보다 앞) 정렬했습니다.

```sh
cargo fmt
git commit -am "cargo fmt (스타일 에디션 2024)"
```

이 상태가 [`step2-2024`](projects/migration/step2-2024/Cargo.toml)입니다. 1단계와 같은 방법으로 비교하면 차이가 없어야 합니다. `cargo run`의 출력은 감사 기록 두 줄의 순서만 2018·2021판과 다릅니다.

```text
감사 기록:
  정산 시작
  잔액 시작
  합산 시작
  합산 끝
  잔액 끝
  보고서 완성
  정산 끝
```

### 2.7 사람이 고쳐야 하는 경우 정리

모두 이 툴체인에서 `cargo fix --edition`을 실제로 실행해 확인했습니다. 실습 프로젝트에 없는 항목은 임시 프로젝트로 확인했습니다.

**자동 수정이 없는 것.** `cargo fix`가 경고로만 보여 줍니다.

| 린트 | 확인한 곳 | 어떻게 고치나 |
| --- | --- | --- |
| `tail_expr_drop_order` | 2.6절 | 순서가 중요한지 판단. 예전 순서가 필요하면 변수와 `drop`으로 순서를 직접 적음 |
| `static_mut_refs` | [`compile_fail/static_mut_shared_reference.rs`](compile_fail/static_mut_shared_reference.rs), 4.2절 | 원자 타입, `Mutex`, `&raw const` 등([ch4](../ch4-rust-2024-language/README.md) 2.9절) |
| `impl_trait_overcaptures` — 인자 위치 `impl Trait`가 있을 때 | `fn ids_where(v: &[u32], keep: impl Fn(u32) -> bool) -> impl Iterator<Item = u32>` | help가 제안하는 대로 이름 있는 제네릭 `T`로 바꾸고 `+ use<T>` |
| `if_let_rescope` — 매크로 본문 안의 `if let` | 지역 매크로 `pick!` | 경고만 남음. 매크로 본문을 손으로 |

**자동 수정이 보수적이라 다시 판단할 것.** 자동 수정은 언제나 "옛 에디션과 같은 동작"을 지키는 쪽을 고릅니다. 옛 동작이 버그였거나 필요 없는 경우가 있습니다. [`01_fix_leftovers_2021.rs`](examples/01_fix_leftovers_2021.rs)와 [`02_fix_leftovers_2024.rs`](examples/02_fix_leftovers_2024.rs)가 두 모양을 나란히 실행합니다.

| 자동 수정이 남긴 모양 | 다시 볼 것 |
| --- | --- |
| 더미 let `let _ = &x;` | 클로저가 `x`보다 먼저 버려지지 않으면 지워도 같음 |
| `panic!("{}", "…{name}…")` | 옛 메시지가 의도였는지 |
| `Trait::method(&x)` | 대개 그대로 둠 |
| `if let` → `match` | 2021의 동작(잠금을 끝까지 쥠)이 필요한지. Edition Guide의 경고대로, 받아들이면 읽기 잠금 뒤 쓰기 잠금을 얻는 교착 상태 버그도 그대로 남습니다. `else if let`이 이어지면 `match`가 겹겹이 중첩되어 읽기도 어려워집니다 |
| `+ use<>` | 부르는 쪽이 정말 필요로 하는지. 필요 없으면 지워서 나중에 숨은 타입이 인자를 빌릴 여지를 남김([ch4](../ch4-rust-2024-language/README.md) 2.3절) |
| `expr_2021` | 크레이트의 **모든** `expr`가 바뀝니다. 충돌하는 규칙이 없으면 `expr`로 되돌림 |
| `unsafe { set_var(..) }`와 `// FIXME: Audit …` | 주석의 질문에 답하고, 답이 "아니오"면 구조를 바꿈([ch5](../ch5-rust-2024-ecosystem/README.md) 2.5절) |
| `unsafe extern`, `#[unsafe(no_mangle)]` | 선언과 심벌 이름을 확인하고 `SAFETY:` 주석. 안전한 함수는 `safe fn` |

**린트가 잡지 못하는 동작 변화.** 컴파일 에러도 경고도 없이 결과만 바뀝니다. 테스트가 유일한 안전망입니다.

| 상황 | 확인한 결과 |
| --- | --- |
| 가드의 메서드를 부르는 `if let` 조건식 — `if let Some(&v) = m.lock().unwrap().get(&k) { .. } else { .. }` | `if_let_rescope`가 경고하지 않음. `*m.lock().unwrap()`이나 `m.lock().unwrap()[i]`처럼 역참조·색인이면 경고함. `Deref`를 거쳐 부르는 메서드(`get`, `first`, `len`)의 수신자인 임시값을 놓치는 것으로 보임 |
| 규칙이 여럿인 매크로의 `$p:pat` | 규칙 `($p:pat)`와 `($($t:tt)*)`를 가진 2018 크레이트에서 `cargo fix --edition`이 아무것도 고치지 않았고, 2021로 올리자 `classify!(1 \| 2)`의 결과가 "규칙 2"에서 "규칙 1"로 바뀜([ch3](../ch3-rust-2021/README.md) 2.11절) |
| `&self`를 받는 사용자 `try_into`, `poll` | `cargo fix --edition`을 거치면 고쳐짐(allow 린트를 켜므로). `edition`부터 올리면 조용히 표준 트레이트 메서드가 불림(2.8절, [ch5](../ch5-rust-2024-ecosystem/README.md) 2.3절) |

ch4 2.4절은 `RefCell`의 `Ref` 가드를 쓰는 `if let`을 이 린트가 알려 주지 않았다고 적었습니다. 이번 확인에 따르면 `RefCell`이라서가 아니라 `cache.borrow().first()`처럼 **가드의 메서드를 부르는 모양**이라서입니다. 같은 `RefCell`이라도 `if let Some(v) = *c.borrow()`는 경고가 났습니다.

**`cargo fix`가 보지 못하는 코드.**

| 코드 | 확인한 결과 |
| --- | --- |
| 내보내기만 하고 정의한 크레이트에서 부르지 않는 매크로 | 펼쳐야 알 수 있는 변경(`panic!(e)`, 배열 `.into_iter()`)은 고치지 못함(2.5절, 연습 2). 단, 키워드 이름은 매크로 정의의 토큰에서 바로 찾아 고침(`let async` → `let r#async`). Edition Guide가 예로 든 `let dyn = 1;`은 고치지 않았음 |
| 문서 테스트 | 고치지 않음(2.5절, 2.6절) |
| 꺼진 기능, `#[cfg]`로 다른 플랫폼에만 컴파일되는 코드 | 컴파일되는 구성만 봄(2.8절) |
| 빌드 스크립트가 만든 코드 | `target/…/out/gen.rs`를 `Fixed …/out/gen.rs (2 fixes)`로 고쳤지만 그것은 빌드 결과물이라, `edition`을 올리자 빌드 스크립트가 옛 코드를 다시 만들어 E0783·E0782. 빌드 스크립트를 고쳐야 함 |

### 2.8 고급 마이그레이션 (Edition Guide 1.3)

**여러 구성: `--all-features`, `--features`, `--target`.** `cargo fix`는 `cargo check`처럼 **한 구성**만 컴파일해 봅니다. `--all-features` 없이 1단계를 하면 `csv.rs`가 그대로 남습니다(`Fixed`가 9곳). 그래도 `edition`을 올리고 2.5.2절의 두 곳을 고치면 `cargo build`와 `cargo test`는 통과합니다. 기능을 켰을 때만 터집니다.

```text
$ cargo test --all-features
error[E0782]: expected a type, found a trait
  --> src/csv.rs:10:43
   |
10 | pub fn write_csv(book: &Ledger, out: &mut io::Write) -> io::Result<()> {
   |                                           ^^^^^^^^^
   |
help: use a new generic type parameter, constrained by `io::Write`
…
help: alternatively, use a trait object to accept any type that implements `io::Write`, accessing its methods at runtime using dynamic dispatch
```

(매개변수 자리라서 `help:`가 제네릭, `impl io::Write`, `dyn io::Write` 세 가지를 제안했습니다. 가운데는 줄였습니다.) 기능이 서로 배타적이면 `--features a`, `--features b`로 나눠 여러 번 실행합니다. 플랫폼별 코드는 `--target 타깃`으로 실행해야 하는데, 그 타깃의 표준 라이브러리가 설치되어 있어야 합니다. 이 환경에는 없어서 이렇게 끝났습니다(크로스 컴파일과 타깃 설치는 Part 5 `ch4-targets-cross-compilation`).

```text
$ cargo fix --edition --target x86_64-unknown-linux-gnu
error[E0463]: can't find crate for `std`
  |
  = note: the `x86_64-unknown-linux-gnu` target may not be installed
  = help: consider downloading the target with `rustup target add x86_64-unknown-linux-gnu`
```

**큰 프로젝트는 나눠서.** 에디션은 패키지마다 정하므로 워크스페이스는 `cargo fix --edition -p 패키지`로 한 패키지씩 옮깁니다(연습 1). 한 패키지 안에서도 `--lib`, `--bin 이름`, `--tests`, `--examples`로 대상을 고를 수 있고, 기본은 `--all-targets`입니다. `--lib`만 주자 라이브러리의 9곳만 고쳤습니다. Edition Guide는 대상마다 `[[bin]] edition = "2018"`처럼 에디션을 따로 적는 방법도 소개하지만, 1.100.0에서는 `` warning: Cargo.toml: `edition` is set on binary `ledger` which is deprecated ``가 나옵니다([ch1](../ch1-editions-overview/README.md) 2.6절). 대상을 나눠 옮기고 싶으면 대상 선택 옵션으로 고치되 `edition`은 패키지 단위로 올립니다.

**`--broken-code`.** Edition Guide에 따르면 `cargo fix`는 적용한 수정 때문에 컴파일이 깨지면 수정을 되돌리고 경고를 냅니다. 이 실습에서는 그런 경우를 만나지 못했습니다. 이미 컴파일 에러가 있는 코드에서 차이를 확인했습니다. `lib.rs`에 `&str`을 `u32`로 돌려주는 잘못된 함수를 더하고 실행하면, 그냥은 `error: could not compile` 끝에 **아무것도 고치지 않습니다**. `--broken-code`를 붙이면 고칠 수 있는 것은 적용하고 에러는 남겨 둡니다.

```text
$ cargo fix --edition --all-features --broken-code
…
error: errors present after applying fixes to crate `ledger`
 --> src/lib.rs
  = cause: error[E0308]: mismatched types
```

이때 고쳐진 것은 10곳 중 8곳이었고, `panic!`과 배열 `.into_iter()`는 고쳐지지 않았습니다. 일부 린트는 타입 검사가 끝나야 실행되는데 그 전에 에러가 났기 때문으로 보입니다. 되돌려진 수정을 손으로 마무리할 때나 이런 상황에서 쓰는 옵션이고, 쓴 뒤에는 `git diff`로 무엇이 바뀌었는지 꼭 확인합니다.

**린트 하나씩 적용하기.** Edition Guide는 크레이트 루트에 `#![warn(keyword_idents)]`처럼 린트 하나만 켜고 `--edition` 없이 `cargo fix`를 실행하는 방법도 소개합니다. `cargo fix`(옵션 없음)는 지금 켜진 경고 가운데 자동 수정이 있는 것을 적용하는 명령이기 때문입니다. 연습 1에서 `#[allow]`를 지운 뒤 `cargo fix`로 경고 5개를 고쳐 봅니다. 에디션과 상관없는 `cargo fix`의 쓰임은 Part 6 `ch1-reading-diagnostics`에서 다룹니다.

**Cargo 없이 옮기기.** `rustc`에 직접 린트 그룹을 켭니다. 단, Edition Guide에 적힌 `--force-warns=rust-2021-compatibility`는 1.100.0에서 `error: Unrecognized option: 'force-warns'`였습니다. 옵션 이름은 `--force-warn`입니다.

```sh
rustc --edition 2018 --force-warn rust-2021-compatibility main.rs
rustc --edition 2018 -W rust-2021-compatibility main.rs     # #[allow]가 있으면 그곳은 안 보임
```

경고를 읽고 손으로 고치거나, 에디터의 "Quick Fix"(rust-analyzer)로 제안을 적용합니다. Edition Guide는 `cargo fix`가 안에서 쓰는 `rustfix` 라이브러리로 도구를 만드는 방법도 소개합니다.

**관용구 린트와 `cargo fix --edition-idioms`.** 에디션은 문법뿐 아니라 관용구(idiom)도 바꿉니다. 1.100.0에 있는 관용구 린트 그룹은 `rust-2018-idioms` 하나뿐입니다(`bare-trait-objects`, `unused-extern-crates`, `ellipsis-inclusive-range-patterns`, `elided-lifetimes-in-paths`, `explicit-outlives-requirements`). `rust-2021-idioms`나 `rust-2024-idioms`는 `rustc -W help`에 없고, `step1-2021`과 `step2-2024`에서 `cargo fix --edition-idioms`를 실행하자 아무것도 하지 않고 끝났습니다. `step0-2018`에서는 다음 경고 하나만 보이고 **고친 것은 0개**였습니다.

```text
warning: hidden lifetime parameters in types are deprecated
  --> src/entry.rs:49:32
   |
49 |     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
   |                           -----^^^^^^^^^
...
   = note: `-W elided-lifetimes-in-paths` implied by `-W rust-2018-idioms`
help: indicate the anonymous lifetime
   |
49 |     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

`--edition-idioms`는 `--force-warn`이 아니라 `-W`로 켜므로 `#[allow]`로 막은 `dyn`·`...`은 보이지도 않았고, 보인 제안도 적용하지 않았습니다. Edition Guide도 관용구 린트에는 알려진 문제가 있어 잘못된 제안을 할 수 있다고 경고합니다. 결과를 기대하기보다, 실행한 뒤 남은 경고를 읽고 손으로 고치는 출발점으로 씁니다.

**에디션을 건너뛰거나 `edition`부터 올리면.** `step0-2018`의 사본으로 확인했습니다.

| 해 본 것 | 결과 |
| --- | --- |
| `edition = "2024"`로 바로 올림 | 라이브러리에서 에러 9개(`gen` 2, unsafe 속성 1, E0782 3, E0783 2, E0599 1)와 경고 `` unused import: `Percent` ``. 라이브러리가 컴파일되지 않아 테스트 쪽의 `extern` 블록과 `set_var`는 아직 보이지도 않음. `cargo fix --edition`은 한 번에 한 에디션씩이므로 2021을 거쳐 두 번 실행합니다 |
| `edition = "2021"`로 먼저 올리고 `cargo fix --edition` | `Migrating … from 2021 edition to 2024`, 즉 **엉뚱한 방향**으로 옮기려다 컴파일 에러로 끝나고 아무것도 고치지 않음 |
| `edition = "2021"`로 먼저 올리고 에러를 손으로 고침(라이브러리 6개, 매크로 본문과 문서 테스트 각 1개) | 컴파일은 되지만 `report_hides_odd_percent`가 실패. `left: "식비   30000원 150%\n…"` — `Percent::try_into` 대신 `TryInto::try_into`가 불려 150%가 표시됨. `unused import: Percent` 경고가 유일한 단서 |
| 2024 에디션 패키지에서 `cargo fix --edition` | `` warning: `src/lib.rs` is already on the latest edition (2024), unable to migrate further ``와 함께 올바른 절차 4단계를 출력 |

마지막 줄의 안내가 이 절의 결론입니다.

```text
If you are trying to migrate from the previous edition (2021), the
process requires following these steps:

1. Start with `edition = "2021"` in `Cargo.toml`
2. Run `cargo fix --edition`
3. Modify `Cargo.toml` to set `edition = "2024"`
4. Run `cargo build` or `cargo test` to verify the fixes worked
```

### 2.9 이 챕터에서 다루지 않는 것

| 주제 | 다루는 곳 |
| --- | --- |
| 각 에디션 변경의 원리 | Part 2 [ch2](../ch2-rust-2015-2018/README.md)~[ch5](../ch5-rust-2024-ecosystem/README.md) |
| 매니페스트, 워크스페이스, 기능의 상세 | [Part 3 ch3](../../part3-cargo/ch3-manifest-targets/README.md), [ch4](../../part3-cargo/ch4-workspaces/README.md), [ch5](../../part3-cargo/ch5-features/README.md) |
| 빌드 스크립트 | [Part 3 ch8](../../part3-cargo/ch8-build-scripts/README.md) |
| CI에서 마이그레이션 확인하기 | Part 3 `ch9-testing-ci` |
| 린트 수준과 우선순위, `[lints]` 테이블 | Part 5 `ch3-lints` |
| 에디션과 상관없는 `cargo fix`, 진단 메시지 읽기 | Part 6 `ch1-reading-diagnostics` |
| nightly의 다음 에디션(`cargo-features = ["edition20xx"]`) | Part 11 `ch1-nightly-toolchain` |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_fix_leftovers_2021.rs`](examples/01_fix_leftovers_2021.rs) | 2018 → 2021 자동 수정이 남긴 모양과 다듬은 모양 | 3 |
| [`02_fix_leftovers_2024.rs`](examples/02_fix_leftovers_2024.rs) | 2021 → 2024 자동 수정이 남긴 모양과 다듬은 모양 | 3 |
| [`projects/migration`](projects/migration/step0-2018/Cargo.toml) | 2018 → 2021 → 2024 단계별 스냅숏과 `cargo fix` 직후의 diff | 1, 2, 3, 4 |
| [`compile_fail/`](compile_fail/static_mut_shared_reference.rs) | 자동 수정이 없거나 펼쳐야 드러나는 에러 3개(4절) | 4 |

두 예제는 이 저장소의 에디션(2024)으로 컴파일되고, 저장소 루트에서 실행합니다. 모두 종료 코드 0으로 곧바로 끝나고 실행할 때마다 같은 출력이 나옵니다.

### 01_fix_leftovers_2021 — 2021로 옮긴 뒤의 정리

```sh
cargo run -p p2-ch6-migration-practice --example 01_fix_leftovers_2021
```

```text
1. 더미 let: 클로저가 블록 끝까지 살아 있을 때
    cargo fix 모양: ["== 정산 ==", "보고서 완성", "구간 끝"]
    다듬은 모양:    ["== 정산 ==", "보고서 완성", "구간 끝"]
2. 더미 let: 클로저를 먼저 버릴 때
    cargo fix 모양: ["== 정산 ==", "구간 끝", "보고서 완성"]
    다듬은 모양:    ["== 정산 ==", "보고서 완성", "구간 끝"]
3. 인자가 하나뿐이던 panic!
    cargo fix 모양: 잘못된 줄: {line}
    다듬은 모양:    잘못된 줄: 식비 abc
4. 사용자 트레이트의 try_into와 프렐류드의 TryInto
    Percent::try_into(&share) = Err("150%는 백분율이 아닙니다")
    share.try_into()          = Ok(150)
5. 배열의 .into_iter()
    Some("교통") == Some("교통"): true
```

- **1·2번 구역**: 더미 let은 "클로저가 버려질 때 `job` 전체가 함께 버려진다"는 2018의 동작을 지킵니다. 클로저가 블록 끝까지 살아 있으면 어차피 같은 때 버려지므로 지워도 결과가 같고(1번), 클로저를 먼저 `drop`하면 결과가 달라집니다(2번). 실습 프로젝트의 `close_month`는 1번의 경우라서 지웠습니다.
- **3번 구역**: 패닉 메시지를 보려고 `catch_unwind`로 붙잡고, 그동안 패닉 훅을 빈 함수로 바꿔 표준 에러에 아무것도 나오지 않게 했습니다([Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)).
- **4번 구역**: 자동 수정이 트레이트 이름을 적어 준 이유입니다. 메서드 호출 문법으로 두면 2021부터는 프렐류드의 `TryInto`가 먼저 잡혀 결과가 바뀝니다.

### 02_fix_leftovers_2024 — 2024로 옮긴 뒤의 정리

```sh
cargo run -p p2-ch6-migration-practice --example 02_fix_leftovers_2024
```

```text
1. if let → match
    cargo fix 모양: 합계 3000, 빈 캐시일 때 다시 잠금 false
    다듬은 모양:    합계 500, 빈 캐시일 때 다시 잠금 true
    두 번째 호출(캐시): (3000, false), (500, false)
2. + use<>
    쥐고 있던 번호 [1, 2], 새 번호 3, 구체 타입 1..=3
3. expr_2021
    cargo fix 모양: 30000원까지, 한도 없음
    다듬은 모양:    30000원까지, 한도 없음, 30000원까지
4. unsafe { set_var }
    cargo fix 모양: 엔, 다듬은 모양: 엔
```

- **1번 구역**: 캐시가 비어 있을 때 대신 계산하는 갈래에서 `try_lock()`으로 같은 `Mutex`를 다시 잠가 봅니다. `match` 모양은 조건식의 잠금을 아직 쥐고 있어 실패(`false`)하고, 2024의 `if let`은 else 전에 풀어서 성공(`true`)합니다. 그래서 다듬은 모양은 else 안에서 바로 캐시를 채웁니다. `try_lock`은 기다리지 않고 바로 결과를 돌려주므로 교착 상태 없이 관찰할 수 있습니다.
- **2번 구역**: `+ use<>` 대신 구체 타입(`RangeInclusive<u32>`)을 돌려주는 방법도 있습니다. 감출 필요가 없는 타입이라면 이쪽이 단순합니다.
- **3번 구역**: 다듬은 매크로는 2024의 `expr`를 쓰므로 `const { 3 * 10_000 }`도 받습니다. `expr_2021` 쪽에 같은 입력을 주면 `` no rules expected keyword `const` `` 에러입니다.
- **4번 구역**: `cargo fix`가 남긴 모양을 보이려고 `unsafe { std::env::set_var(..) }`를 씁니다. 이 예제는 그 시점까지 스레드를 만들지 않으므로 `SAFETY:` 주석에 그렇게 적었습니다. 이 챕터의 `examples/`에서 unsafe 코드는 이 두 줄뿐입니다. 다듬은 모양은 값을 직접 넘겨 unsafe가 필요 없습니다.

### projects/migration — 단계별 스냅숏

세 디렉터리는 각각 독립 프로젝트입니다. 저장소 안에서는 **실행과 검사만** 합니다.

```sh
cd part2-editions/ch6-migration-practice/projects/migration/step2-2024   # step0-2018, step1-2021도 같음
cargo run                      # 2.3절·2.6.3절의 출력
cargo run -- bar               # 막대 스타일
cargo test --all-features      # 통합 테스트 12개 + 문서 테스트 2개
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

`cargo run -- bar`의 처음 다섯 줄입니다(세 단계 모두 같습니다).

```text
== 4건 정산 ==
식비 #####
교통 ##
기타 #
합계 48500원 (알뜰)
```

- 세 단계의 `cargo run` 출력은 2018과 2021이 같고, 2024는 감사 기록의 "합산 끝"과 "잔액 끝"의 순서만 다릅니다(2.6.2절).
- 기능 `csv`를 끄고 `cargo test`만 하면 통합 테스트가 11개입니다.
- [`diffs/`](projects/migration/diffs/step0-cargo-fix-2021.diff)의 두 파일은 각각 `step0-2018`과 `step1-2021`의 사본에서 `cargo fix --edition --all-features`를 실행한 직후의 `git diff`입니다. 스냅숏이 아니라 diff로 둔 이유는, 그 상태가 서식(`cargo fmt --check`)을 통과하지 않는 "중간 상태"이기 때문입니다.
- 실습이 끝나면 사본(`/tmp/rust-p2-ch6`)을 지워도 됩니다. 저장소의 스냅숏에서 `cargo build`를 했다면 `cargo clean`으로 `target/`을 지웁니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 `cargo fix`가 고쳐 주지 못하거나 펼쳐진 뒤에야 드러나는 에러입니다. 챕터 디렉터리에서 에디션을 바꿔 가며 컴파일합니다.

```sh
cd part2-editions/ch6-migration-practice
rustc --edition 2024 compile_fail/static_mut_shared_reference.rs --out-dir /tmp/rust-p2-ch6   # 에러
rustc --edition 2021 compile_fail/static_mut_shared_reference.rs --out-dir /tmp/rust-p2-ch6   # 경고와 함께 통과
```

| 파일 | 실패하는 에디션 | 통과하는 에디션 |
| --- | --- | --- |
| `e0783_range_pattern_in_macro.rs` | 2021, 2024 | 2018 (경고) |
| `static_mut_shared_reference.rs` | 2024 | 2018, 2021 (경고) |
| `underscore_matches_expr_fragment.rs` | 2024 | 2018, 2021 |

### 4.1 매크로 본문에 남은 `...` 범위 패턴 — E0783

파일: [`compile_fail/e0783_range_pattern_in_macro.rs`](compile_fail/e0783_range_pattern_in_macro.rs)

```text
error[E0783]: `...` range patterns are deprecated
  --> compile_fail/e0783_range_pattern_in_macro.rs:19:13
   |
16 | macro_rules! grade {
...
19 |             90...100 => 'A', // → 2021 에디션부터 E0783
   |             ^^---^^^
   |               |
   |               help: use `..=` for an inclusive range
...
29 |         println!("{} → {}", score, grade!(*score));
   |                                    -------------- in this macro invocation
```

(같은 모양의 둘째 에러는 줄였습니다.)

- **읽는 법**: 에러의 위치(19번 줄)는 매크로 본문이고, 마지막의 `in this macro invocation`이 매크로를 부른 곳(29번 줄)입니다. 매크로 본문의 에러는 **펼쳐질 때** 보고되므로 이 두 위치를 함께 봅니다. 2.5.2절의 `book!`도 같은 모양이었습니다.
- **원인**: [ch3](../ch3-rust-2021/README.md) 2.10절의 경고에서 에러로 승격된 문법입니다. 2018에서는 `ellipsis_inclusive_range_patterns` 경고와 함께 통과합니다.
- **해결**: `help:`대로 `90..=100`. 이 파일처럼 같은 크레이트가 매크로를 부르면 `cargo fix --edition`이 고쳐 줍니다. 매크로를 내보내기만 하는 라이브러리라면 `cargo fix`가 볼 기회가 없으므로, 매크로를 실제로 펼쳐 보는 테스트를 그 크레이트 안에 두어야 합니다. Edition Guide도 매크로의 문법을 모두 덮는 테스트를 권합니다.

### 4.2 `static mut`의 공유 참조 — 에러 코드 없음

파일: [`compile_fail/static_mut_shared_reference.rs`](compile_fail/static_mut_shared_reference.rs)

```text
error: creating a shared reference to mutable static
  --> compile_fail/static_mut_shared_reference.rs:23:14
   |
23 |     unsafe { LAST_ERROR.clone() } // → 2024 에디션에서 에러(공유 참조를 만듦)
   |              ^^^^^^^^^^^^^^^^^^ shared reference to mutable static
   |
   = note: shared references to mutable statics are dangerous; it's undefined behavior if the static is mutated or if a mutable reference is created for it while the shared reference lives
   = help: use a type that relies on "interior mutability" instead; to read more on this, visit <https://doc.rust-lang.org/reference/interior-mutability.html>
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html>
   = note: `#[deny(static_mut_refs)]` (part of `#[deny(rust_2024_compatibility)]`) on by default
```

- **읽는 법**: 마지막 note가 `rust_2024_compatibility` 그룹의 `static_mut_refs` 린트가 deny 수준이라서 에러라고 알려 줍니다. 2021에서는 같은 자리가 `#[warn(static_mut_refs)]`의 경고입니다.
- **원인**: `.clone()`이 `&LAST_ERROR`를 몰래 만듭니다. 값을 대입하는 `LAST_ERROR = Some(..)`은 참조가 아니라 문제없습니다([ch4](../ch4-rust-2024-language/README.md) 2.9절).
- **해결**: 자동 수정이 없습니다. `cargo fix --edition`은 이 린트를 `--force-warn`으로 켜서 경고로 보여 줄 뿐입니다. `static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);`처럼 바꾸면 `unsafe`도 필요 없어집니다. 2021의 경고를 `#[allow(static_mut_refs)]`로 막아 두었다면 2024에서도 그 `#[allow]`가 deny를 이겨 에러가 나지 않으므로, 옮길 때 `#[allow]`를 함께 찾아봅니다.

### 4.3 `_`가 `$x:expr` 규칙에 걸림 — 에러 코드 없음

파일: [`compile_fail/underscore_matches_expr_fragment.rs`](compile_fail/underscore_matches_expr_fragment.rs)

```text
error: in expressions, `_` can only be used on the left-hand side of an assignment
  --> compile_fail/underscore_matches_expr_fragment.rs:30:30
   |
30 |     println!("{:?}", budget!(_)); // → 2024 에디션에서 에러
   |                              ^ `_` not allowed here
```

- **읽는 법**: 에러는 `_`가 표현식 자리에 왔다는 것만 말하고, 매크로나 에디션 이야기는 없습니다. 매크로 인자에서 이런 에러가 나면 "다른 규칙으로 가야 할 입력이 앞 규칙에 걸렸나"를 의심합니다.
- **원인**: 2024의 `expr`는 `_`를 받으므로 첫 규칙 `($limit:expr)`가 `budget!(_)`을 가져가 `Budget::Limit(_)`로 펼쳤습니다. 2021에서는 `expr`가 `_`를 받지 않아 둘째 규칙 `(_)`로 갔습니다([ch4](../ch4-rust-2024-language/README.md) 2.12절).
- **해결**: `cargo fix --edition`은 `$limit:expr_2021`로 바꿔 줍니다(2.6.1절). 2.6.3절처럼 `(_)` 규칙을 앞으로 옮기면 `expr`를 그대로 쓸 수 있습니다.

### 4.4 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| 커밋하지 않은 변경(추적하지 않는 새 파일 포함)이 있는 채로 `cargo fix` | `the working directory of this package has uncommitted changes` | 커밋하거나, 의도한 것이면 `--allow-dirty` (2.4절) |
| `edition`부터 올리고 `cargo fix --edition` | 다음 에디션(2024)으로 옮기려다 컴파일 에러로 끝남 | `edition`을 되돌리고 `cargo fix --edition`부터 (2.8절) |
| `--all-features` 없이 `cargo fix` | 기능 뒤의 코드가 그대로, 기능을 켜야 E0782 | `--all-features` 또는 기능마다 `--features` (2.8절) |
| `cargo fix` 뒤 서식 검사 | 자동 수정한 줄 때문에 `cargo fmt --check` 실패 | 다듬은 뒤 `cargo fmt` (2.5.3절) |
| 2024로 옮기며 코드 수정과 `cargo fmt`를 한 커밋에 | 서식 변경(버전 정렬)이 섞여 리뷰가 어려움 | 서식만 따로 커밋 (2.6.3절) |
| `-W rust-2021-compatibility`로 미리 보기 | `#[allow]`로 막힌 곳이 안 보임 | `--force-warn` (2.2절) |
| 이 학습 저장소 안의 스냅숏에서 `cargo fix` | 스냅숏이 바뀌어 단계별 비교와 `diffs/`가 맞지 않게 됨 | 저장소 밖 사본에서 (2.4절) |

## 5. 연습 문제

### 연습 1. ch2의 2015판을 `cargo fix`로 2018로 옮기기

[ch2](../ch2-rust-2015-2018/README.md)의 `projects/scoreboard`를 `/tmp/rust-p2-ch6/scoreboard`로 복사하고 git 저장소로 만든 뒤, `app2015` 패키지만 2018 에디션으로 옮기세요. 옮긴 뒤 `cargo run -p app2015`의 출력이 옮기기 전과 같은지 확인하고, 남아 있는 `#[allow(...)]`를 모두 지워 경고까지 없애 보세요. (ch2 디렉터리 자체는 건드리지 않습니다.)

<details>
<summary>힌트</summary>

- 워크스페이스이므로 `cargo fix --edition -p app2015`처럼 패키지를 고릅니다. 수정은 6곳입니다. `try!(scorekit::try(..))` → `r#try!(scorekit::r#try(..))`, 익명 매개변수 → `_: &str, _: u32`, `use record::Record` → `use crate::record::Record`, `::report::grade` → `crate::report::grade`.
- ch2에서 `edition`만 2018로 바꾸면 에러 5개가 났던 코드입니다. 자동 수정 뒤에는 `edition = "2018"`로 바꿔도 경고 없이 빌드되고 출력도 같습니다.
- `dyn`과 `..=`는 고쳐지지 않았습니다. `rust-2018-compatibility` 그룹이 아니라 경고 수준의 린트이고 `#[allow]`로 막혀 있기 때문입니다. `#[allow]` 5개를 지우면 경고 6개(`...` 3, `dyn` 2, deprecated `r#try` 1)가 나오고, `--edition` 없는 `cargo fix -p app2015`가 5개를 고칩니다. `r#try!`는 자동 수정이 없으므로 손으로 `?`로 바꿉니다.

</details>

### 연습 2. ch1의 2015 크레이트를 2021까지 올리고 출력 비교하기

[ch1](../ch1-editions-overview/README.md)의 `projects/edition_mix`를 복사해, `jobs_2015` 패키지를 `cargo fix --edition`으로 2018로, 다시 2021로 옮기세요. 단계마다 `cargo run`의 출력을 옮기기 전과 비교하세요. ch1 연습 1에서 손으로 했던 일과 무엇이 다른가요?

<details>
<summary>힌트</summary>

- 2015 → 2018: 6곳을 고칩니다. 함수 이름 `async`·`await`에 더해 `run_now!` 매크로 본문의 `let async`도 `let r#async`로 고쳐 줍니다. 키워드 이름은 매크로 정의의 토큰만 보고도 찾을 수 있기 때문입니다(2.7절). 출력은 같습니다.
- 2018 → 2021: `Fixed`가 하나도 없습니다. 그런데 2021로 올리면 3번 구역이 `2015 크레이트의 매크로: Some(10), core::option::Option<i32>`로 바뀝니다. `first_item!`의 `.into_iter()`는 `jobs_2015` 안에서 한 번도 펼쳐지지 않아 `array_into_iter` 린트가 볼 수 없었습니다. 2018과 같게 하려면 본문을 `.iter()`로 손으로 고칩니다.
- 이 연습이 2.5.2절의 교훈을 다시 보여 줍니다. 내보내기만 하는 매크로는 그 크레이트 안에서 펼쳐 보는 테스트가 있어야 안전하게 옮길 수 있습니다.

</details>

### 연습 3. `edition`부터 올린 사고 수습하기

`step0-2018`의 사본에서 `cargo fix` 없이 `edition = "2021"`로 바꾸고, 컴파일 에러만 손으로 고쳐 보세요. 그다음 `cargo test`를 실행해 어떤 테스트가 왜 실패하는지 찾고, 실패를 고치세요. 컴파일러가 남긴 단서도 찾아보세요.

<details>
<summary>힌트</summary>

- 에러는 6개입니다(E0782 3, E0783 2, E0599 1). E0599는 `.copied()`가 `Option<Category>`에는 없다는 것으로, 배열의 `.into_iter()`가 값을 내놓게 되어 생깁니다. `.copied()`를 지우면 됩니다. 매크로 본문의 `panic!(e)`와 문서 테스트도 고쳐야 테스트가 끝까지 돕니다(2.5.2절).
- 실패하는 테스트는 `report_hides_odd_percent`입니다. `식비   30000원 150%`가 나옵니다. 단서는 빌드할 때의 `` warning: unused import: `Percent` ``입니다. 트레이트를 가져왔는데 쓰이지 않는다는 것은, `.try_into()`가 더는 `Percent`의 메서드가 아니라는 뜻입니다.
- `Percent::try_into(&(sum * 100 / total))`로 고치면 통과합니다. 2.5.1절의 자동 수정이 한 일과 같습니다. 클로저의 캡처와 `panic!` 메시지도 바뀌었지만 테스트는 통과합니다. 각각 왜 괜찮은지 2.5.3절의 표와 맞춰 보세요.

</details>

### 연습 4. 미리 보기와 실제 수정 맞춰 보기

`step1-2021`의 사본에서 `RUSTFLAGS="--force-warn rust-2024-compatibility" cargo check --all-targets --all-features`로 경고를 먼저 보고, 경고마다 `cargo fix --edition --all-features`가 고칠지, 어떻게 고칠지 예상한 뒤 실제 결과([`diffs/step1-cargo-fix-2024.diff`](projects/migration/diffs/step1-cargo-fix-2024.diff))와 비교하세요.

<details>
<summary>힌트</summary>

- 경고는 11개입니다. `gen` 2, `if let` 1, `impl Iterator` 캡처 1, `set_var`·`remove_var` 각 1, `extern` 블록 1, unsafe 속성 1, `expr` 프래그먼트 2, 마지막 표현식의 drop 순서 1.
- 마지막 하나만 자동 수정이 없습니다. 나머지 10개는 각 경고의 `help:`에 적힌 모양 그대로 고쳐집니다. 자동으로 적용되는 것은 컴파일러가 "기계적으로 적용해도 된다"고 표시한 제안뿐이어서, 인자 위치 `impl Trait`의 경우(2.7절)처럼 `help:`에 제안이 있어도 적용되지 않을 수 있습니다.
- `RUSTFLAGS`를 바꾸면 전체를 다시 컴파일합니다. 끝나면 `cargo clean`으로 정리해도 됩니다.

</details>

## 6. 참고 자료

- The Rust Edition Guide (기준 문서)
  - [1.2 Transitioning an existing project to a new edition](https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html): 다섯 단계(`cargo update` → `cargo fix --edition` → `edition` → build/test → `cargo fmt`), 서식 변경을 따로 커밋하라는 권고, nightly에서 다음 에디션 시험하기.
  - [1.3 Advanced migrations](https://doc.rust-lang.org/edition-guide/editions/advanced-migrations.html): 동작 원리, 여러 구성, 큰 프로젝트, `--broken-code`, 매크로와 매크로 위생, 문서 테스트, 생성된 코드, Cargo 없이 옮기기, 관용구 린트. 이 챕터에서 확인한 1.100.0과의 차이: 타깃별 `edition` 키는 deprecated 경고, `--force-warns`는 없는 옵션(`--force-warn`), 매크로 본문의 키워드 이름(`async`)은 고쳐짐(`dyn`은 고쳐지지 않음).
  - [5.2.3 Unsafe functions](https://doc.rust-lang.org/edition-guide/rust-2024/newly-unsafe-functions.html)의 마이그레이션 절: 예시는 `// TODO: Audit that …`이지만 1.100.0은 `// FIXME: Audit that …`을 넣습니다.
- Cargo Book: [`cargo fix`](https://doc.rust-lang.org/cargo/commands/cargo-fix.html) — `--edition`이 `Cargo.toml`의 `edition`을 바꾸지 않는다는 설명, `--allow-dirty`/`--allow-staged`/`--allow-no-vcs`, 대상·기능·`--target` 선택.
- rustc Book: [Lint Groups](https://doc.rust-lang.org/rustc/lints/groups.html) — `rust-2018-compatibility`, `rust-2021-compatibility`, `rust-2024-compatibility`, `rust-2018-idioms`의 목록. 터미널에서는 `rustc -W help`.
- 로컬 문서: 위 문서는 툴체인과 함께 설치되어 있습니다(`rustup doc --edition-guide`). 이 챕터는 1.100.0과 함께 설치된 판으로 확인했습니다.

## Part 2를 마치며

Part 2에서는 에디션이 하위 호환성을 지키면서 호환되지 않는 변경을 들여오는 제도라는 것(ch1)에서 시작해, 2018·2021·2024 에디션이 언어와 표준 라이브러리, Cargo, rustdoc, rustfmt에서 바꾼 것을 같은 코드를 두 에디션으로 컴파일해 비교했고(ch2~ch5), 이 챕터에서 그 변경들을 `cargo fix --edition`과 테스트로 실제 프로젝트에 적용했습니다. 자동 수정은 옛 동작을 지키는 쪽으로만 고치고, 판단과 확인은 언제나 사람과 테스트의 몫이라는 것이 이 파트의 결론입니다. 다음 Part 3(Cargo)에서는 이 파트에서 여러 번 "Part 3에서 다룬다"고 미뤄 둔 매니페스트, 의존성과 `Cargo.lock`, 워크스페이스, 기능, 빌드 스크립트, 테스트와 CI를 Cargo Book을 따라 하나씩 살펴봅니다. 전체 순서는 [로드맵](../../docs/roadmap.md)에서 확인할 수 있습니다.
