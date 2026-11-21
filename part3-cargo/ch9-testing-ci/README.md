# ch9. 테스트와 CI

테스트를 쓰는 법(`#[test]`, 단언 매크로, 단위·통합·문서 테스트)은 [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md)에서 배웠습니다. 이 챕터는 그 테스트를 **Cargo가 어떻게 찾아서 빌드하고 실행하는지**, 벤치마크는 어떻게 돌리는지, 그리고 그것을 변경마다 자동으로 돌리는 CI(continuous integration, 지속적 통합)를 Cargo 관점에서 어떻게 구성하는지 다룹니다.

- 패키지: `p3-ch9-testing-ci`
- 기준 문서: The Cargo Book [2.7 Tests](https://doc.rust-lang.org/cargo/guide/tests.html), [2.8 Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html). 명령의 옵션은 [cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html)와 [cargo bench](https://doc.rust-lang.org/cargo/commands/cargo-bench.html) 문서를 함께 봤습니다.
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0, macOS (aarch64). 최소 지원 Rust 버전(MSRV) 확인에는 설치돼 있는 1.98.1 툴체인을 썼습니다.
- 실습 자료: `examples/` 2개, 독립 프로젝트 [`projects/linestat`](projects/linestat/Cargo.toml)(라이브러리·바이너리, 단위·통합·문서 테스트, 벤치마크, CI 워크플로 예시). `compile_fail/`은 없습니다. 외부 크레이트와 네트워크는 쓰지 않습니다.
- CI 워크플로 [`ci/github-workflow.yml`](projects/linestat/ci/github-workflow.yml)은 GitHub에서 실행해 보지 못했습니다. YAML 문법과, 작업마다의 `cargo` 명령을 로컬에서 실행한 것만 확인했습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `cargo test`가 기본으로 빌드·실행하는 타깃(라이브러리·바이너리의 단위 테스트, 통합 테스트, 예제, 문서 테스트)과 그것을 고르는 옵션(`--lib`, `--test`, `--doc`, `--all-targets` 등)을 설명하고, 이름 거르기가 문서 테스트에 주는 영향을 말할 수 있다.
2. 매니페스트의 `test`·`harness`·`doctest` 키가 테스트 타깃을 어떻게 바꾸는지 설명하고, `harness = false`인 타깃을 직접 만들 수 있다.
3. `cargo bench`가 쓰는 프로필과 실행 방식을 설명하고, stable에서 `benches/`에 `harness = false` 벤치마크를 둘 수 있다.
4. 테스트가 프로필에 따라 다르게 판정될 수 있다는 것을 예로 들고, `--release` 테스트가 필요한 이유를 설명할 수 있다.
5. Cargo Book이 권하는 CI 작업(빌드·테스트, 최신 의존성 확인, `rust-version` 확인, 경고 검사)과 Cargo home 캐시를 담은 워크플로를 읽고, 각 작업의 명령을 로컬에서 재현할 수 있다.

## 2. 핵심 개념

### 2.1 `cargo test`가 빌드하고 실행하는 것

Part 0 ch11에서 `cargo test`의 출력이 여러 구역(라이브러리, 통합 테스트 파일마다, `Doc-tests`)으로 나뉘는 것을 봤습니다. 구역 하나가 **테스트 실행 파일 하나**입니다. Cargo는 테스트할 타깃마다 `rustc --test`로 컴파일해 libtest(테스트 러너)와 링크한 실행 파일을 만들고, 그것들을 **차례로** 실행합니다. 실행 파일 하나 안의 테스트 함수들은 libtest가 여러 스레드에서 **동시에** 돌립니다.

아무 옵션 없이 `cargo test`를 실행하면 Cargo는 다음을 빌드합니다.

| 무엇 | 왜 | 실행하나 |
| --- | --- | --- |
| 라이브러리 | 바이너리·예제·통합 테스트·문서 테스트가 링크해 쓰므로 | (일반 빌드) |
| 바이너리(통합 테스트나 벤치마크를 빌드할 때만) | 통합 테스트가 실행해 볼 수 있게(`CARGO_BIN_EXE_<이름>`, [ch7](../ch7-configuration-env/README.md) 2.7절) | (일반 빌드) |
| 예제 | 컴파일되는지만 확인하려고 | 아니요 |
| 라이브러리의 단위 테스트(`src/lib.rs`의 `#[cfg(test)]`) | | 예 |
| 바이너리의 단위 테스트(`src/main.rs`의 `#[cfg(test)]`) | | 예 |
| 통합 테스트(`tests/*.rs`마다 하나) | | 예 |
| 라이브러리의 문서 테스트 | `rustdoc`이 문서 주석의 코드 블록을 모아 컴파일해 실행(2024 에디션은 한 바이너리로 묶어 컴파일, [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.8절) | 예 |

`projects/linestat`에서 실행하면 실행 파일 넷과 문서 테스트가 차례로 돕니다(해시와 시간은 줄였습니다).

```console
$ cargo test --locked
    Finished `test` profile [unoptimized + debuginfo] target(s) in …s
     Running unittests src/lib.rs (target/debug/build/linestat/<해시>/out/linestat-<해시>)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
     Running unittests src/main.rs (target/debug/build/linestat/<해시>/out/linestat-<해시>)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
     Running tests/api.rs (target/debug/build/linestat/<해시>/out/api-<해시>)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
     Running tests/cli.rs (target/debug/build/linestat/<해시>/out/cli-<해시>)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
   Doc-tests linestat
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
all doctests ran in …s; merged doctests compilation took …s
```

(각 구역의 `running N tests`와 테스트 이름 줄은 뺐습니다. 테스트 이름이 출력되는 순서는 실행마다 다를 수 있습니다.) 마지막 줄은 문서 테스트를 한 바이너리로 묶어 컴파일했다는 표시입니다. 1.100.0의 `cargo test` 명령 문서는 아직 "코드 블록마다 실행 파일 하나를 컴파일한다"고 적지만, 2024 에디션의 실제 동작은 이렇습니다. 테스트 실행 파일은 target 디렉터리로 꺼내지 않고 build 디렉터리에서 바로 실행합니다([ch6](../ch6-profiles-build-performance/README.md) 2.5절). `--no-run`을 주면 실행하지 않고 만들기만 하며, 만든 실행 파일의 경로를 `Executable …` 줄로 보여 줍니다. 디버거로 테스트 하나를 붙잡고 싶을 때 이 경로를 씁니다.

**테스트의 작업 디렉터리.** 단위·통합 테스트는 그 테스트가 속한 **패키지 디렉터리**에서 실행됩니다. 그래서 테스트가 `tests/data/input.txt` 같은 상대 경로로 파일을 읽으면 `cargo test`를 어느 디렉터리에서 실행했든 같은 파일을 찾습니다. 통합 테스트가 임시 파일을 둘 곳은 컴파일할 때 `env!("CARGO_TARGET_TMPDIR")`로 읽습니다. 실행 중의 환경 변수로는 들어오지 않습니다(ch7 2.7절 표).

**실패했을 때.** 실행 파일 하나 안에서는 libtest가 모든 테스트를 끝까지 돌리고 실패를 모아 보고합니다. 하지만 실행 파일 하나가 실패하면 Cargo는 **뒤의 실행 파일을 실행하지 않고** 멈춥니다. 모든 실행 파일을 끝까지 돌리려면 `--no-fail-fast`를 줍니다. CI에서 실패를 한 번에 다 보고 싶을 때 씁니다. `cargo test`의 종료 상태는 실패한 테스트 실행 파일의 종료 상태입니다(libtest는 101, 예제 01 같은 `harness = false` 타깃은 그 `main`이 돌려준 값).

**프로필.** `cargo test`는 `test` 프로필을 씁니다([ch6](../ch6-profiles-build-performance/README.md) 2.1절). `test`는 `dev`를 물려받으므로 위 출력처럼 `[unoptimized + debuginfo]`입니다. 1.100.0의 `cargo test` 명령 문서는 `test` 프로필이 "일부 최적화를 켜고 디버그 정보를 끈다"고 적지만, 이 툴체인에서 실제로 쓴 설정은 그렇지 않았습니다. `--release`를 주면 `release` 프로필로 테스트합니다(2.4절).

### 2.2 테스트할 것 고르기

**타깃 고르기.** 옵션을 주면 그 타깃만 테스트합니다.

| 옵션 | 테스트하는 것 |
| --- | --- |
| `--lib` | 라이브러리의 단위 테스트 |
| `--bin 이름`, `--bins` | 바이너리의 단위 테스트 |
| `--test 이름`, `--tests` | 통합 테스트 `tests/이름.rs`, `test = true`인 모든 타깃 |
| `--example 이름`, `--examples` | 예제(예제 안의 `#[cfg(test)]` 테스트) |
| `--bench 이름`, `--benches` | 벤치마크 타깃 `benches/이름.rs`, `bench = true`인 모든 타깃(라이브러리·바이너리 포함)을 테스트 모드로 |
| `--doc` | 문서 테스트만 |
| `--all-targets` | `--lib --bins --tests --benches --examples`와 같음. **문서 테스트는 빠짐** |

`--all-targets`라는 이름과 달리 문서 테스트가 빠지므로, 모든 테스트를 돌리려면 `cargo test --all-targets`와 `cargo test --doc`을 따로 실행합니다(2.6절의 CI 워크플로가 그렇게 합니다). `projects/linestat`에서 `--all-targets`로 실행하면 `Doc-tests` 구역이 없는 대신 `benches/count.rs`와 `examples/demo.rs`가 테스트 모드로 실행됩니다.

**패키지 고르기.** 워크스페이스에서는 `-p`, `--workspace`, `--exclude`를 씁니다([ch4](../ch4-workspaces/README.md) 2.4절). 가상 워크스페이스의 루트에서는 기본이 모든 멤버이고, 루트 패키지가 있는 워크스페이스에서는 루트 패키지뿐입니다. CI에서는 빠뜨리지 않도록 `--workspace`를 명시합니다.

**이름으로 거르기.** `cargo test 이름`은 이름에 그 문자열이 든 테스트만 실행합니다. 그런데 Part 0 ch11 2.7절에서 본 대로, `--` **앞에** 이름을 주면 문서 테스트 구역이 아예 실행되지 않습니다. `projects/linestat`에서 `cargo test count`를 실행하면 단위·통합 테스트 구역은 거른 결과를 보여 주지만 `Doc-tests` 구역은 나오지 않고, `cargo test -- count`로 실행하면 문서 테스트까지 거릅니다(`1 passed; … 2 filtered out`). `cargo test` 명령 문서는 이 차이를 적지 않습니다. 문서 테스트까지 이름으로 거르려면 `--` 뒤에 씁니다. 이 동작은 타깃을 고르지 않았을 때의 일입니다. `cargo test --doc top_words`처럼 `--doc`과 함께 주면 `--` 앞의 이름도 문서 테스트를 거릅니다.

**`--` 뒤의 옵션.** `--` 앞은 Cargo의 옵션이고, 뒤는 테스트 실행 파일(libtest)에 넘어갑니다. Part 0 ch11에서 쓴 `--test-threads`, `--show-output`, `--ignored`가 여기에 속합니다.

### 2.3 매니페스트의 `test`·`harness`·`doctest`

[ch3](../ch3-manifest-targets/README.md) 2.3절의 타깃 표에서 본 키 셋이 `cargo test`의 동작을 바꿉니다.

| 키 | 기본값 | 바꾸면 |
| --- | --- | --- |
| `test` | 라이브러리·바이너리·통합 테스트는 `true`, 예제·벤치마크는 `false` | 기본 `cargo test`에서 그 타깃을 테스트할지. 예제에 `test = true`를 주면 예제도 테스트로 돕니다 |
| `harness` | `true` | `false`면 libtest와 링크하지 않습니다. 그 타깃의 `main`이 그대로 실행되고, **종료 상태**로 성공·실패를 판정합니다 |
| `doctest` | 라이브러리는 `true` | `false`면 그 라이브러리의 문서 테스트를 돌리지 않습니다 |

`harness = false`는 libtest가 맞지 않을 때 씁니다. 테스트를 데이터 파일에서 읽어 만드는 테스트 러너, 프로세스 하나에서 순서대로 돌아야 하는 테스트, 그리고 2.5절의 벤치마크가 그렇습니다. 이 챕터 패키지의 `Cargo.toml`은 예제 01에 `test = true`와 `harness = false`를 함께 줘서, `cargo test`가 예제의 `main`을 테스트로 실행하게 합니다.

```toml
[[example]]
name = "01_custom_harness"
test = true
harness = false
```

libtest가 없으므로 `--` 뒤의 인자는 해석되지 않고 그대로 `main`에 들어옵니다. `--test-threads` 같은 옵션도 쓰려면 직접 해석해야 합니다(예제 01). 반대로 `harness = true`(기본)인 테스트 파일에 `fn main`을 두면 그 `main`은 쓰이지 않습니다.

### 2.4 같은 테스트가 프로필에 따라 다르게 판정된다

`cargo test`는 `dev`를 물려받은 `test` 프로필을 쓰므로 `debug-assertions`와 `overflow-checks`가 켜져 있습니다. 테스트가 이 검사에 기대고 있으면 `--release`에서는 결과가 달라집니다. 예제 02의 `overflow_panics`는 `u8` 덧셈의 오버플로가 패닉하기를 기대하는 `#[should_panic]` 테스트인데, `--release`에서는 덧셈이 래핑되어 패닉하지 않으므로 실패합니다.

반대 방향의 문제도 있습니다. 디버그 빌드에서만 드러나는 버그(오버플로 패닉, `debug_assert!`)는 릴리스로만 테스트하면 놓치고, 최적화에서만 드러나는 문제(잘못된 `unsafe` 코드가 최적화로 다르게 동작하는 경우, 너무 느려 디버그로는 돌리지 못하는 큰 입력)는 디버그로만 테스트하면 놓칩니다. 배포하는 빌드가 `release`라면 CI에 `cargo test --release`도 두는 것을 생각해 볼 만합니다. 오버플로를 검사에 맡기지 말고 `checked_add` 같은 메서드로 의도를 밝히는 것이 근본적인 해결입니다([Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md)).

### 2.5 `cargo bench`와 벤치마크 타깃

`cargo bench`는 벤치마크 타깃(`benches/*.rs`)을 `bench` 프로필로 빌드해 실행합니다. `bench`는 `release`를 물려받아 최적화된 빌드로 잽니다. 성능은 릴리스 빌드로 재야 하기 때문입니다([부록 A ch20](../../appd-a/ch20-avoid-over-optimizing/README.md) 2.5절).

- libtest의 `#[bench]` 함수와 `test::Bencher`는 **nightly 전용**입니다. 그래서 stable에서는 벤치마크 파일에 `harness = false`를 주고 `main`이 직접 시간을 재거나, 그 일을 해 주는 크레이트(`criterion` 등, [부록 A ch30](../../appd-a/ch30-beyond-unit-tests/README.md) 2.6절)를 씁니다.
- `cargo bench`는 벤치마크 실행 파일에 `--bench` 인자를 붙여 실행합니다. 같은 파일이 `cargo test --all-targets`에서 테스트 모드로 실행될 때는 이 인자가 없으므로, `main`이 인자를 보고 짧게 돌지 오래 잴지 정할 수 있습니다(`criterion`이 그렇게 합니다).
- `cargo bench`는 라이브러리·바이너리의 단위 테스트 실행 파일도 벤치마크 모드로 실행합니다. 그 안의 `#[test]` 함수는 `ignored`로 건너뜁니다.

`projects/linestat`의 [`benches/count.rs`](projects/linestat/benches/count.rs)는 같은 계산을 20번 하고 가장 짧은 시간을 출력합니다.

```toml
[[bench]]
name = "count"
harness = false
```

### 2.6 CI 구성

CI는 저장소에 변경이 올 때마다(그리고 정해 둔 주기마다) 빌드와 테스트를 자동으로 돌리는 것입니다. Cargo Book 2.8절은 GitHub Actions, GitLab CI, builds.sr.ht, CircleCI의 기본 설정 예와 함께, Cargo 쪽에서 신경 쓸 일 넷을 듭니다. 기본 빌드·테스트, 최신 의존성 확인, `rust-version` 확인, 경고 검사입니다. CI를 어떤 단계와 주기로 나누고 무엇을 고정할지에 관한 일반 지침은 [부록 A ch32](../../appd-a/ch32-continuous-integration/README.md)가 다룹니다. `projects/linestat`의 [`ci/github-workflow.yml`](projects/linestat/ci/github-workflow.yml)은 이 넷을 GitHub Actions 작업으로 옮긴 예시입니다. `rust-version` 확인은 `test` 작업의 매트릭스에 합쳤고, [ch5](../ch5-features/README.md)에서 본 기능 조합 확인을 `features` 작업으로 더했습니다.

**워크플로 파일 읽기.** GitHub Actions의 설정은 YAML 파일 하나가 워크플로(workflow) 하나입니다. 워크플로는 여러 작업(job)으로 이뤄지고, 작업마다 새 가상 머신인 러너(runner)에서 단계(step)를 차례로 실행합니다. 서로 다른 작업은 동시에 돕니다. 이 예시에 쓴 키는 다음과 같습니다.

| 키 | 뜻 |
| --- | --- |
| `on` | 워크플로를 실행할 사건. `pull_request`(PR을 열거나 커밋을 더할 때), `push`(여기서는 `main`에 들어올 때), `schedule`의 `cron`(정해 둔 시각, UTC) |
| `permissions` | 워크플로가 받는 GitHub 토큰의 권한. `contents: read`로 저장소 읽기만 허락 |
| `defaults.run` | 모든 `run` 단계의 기본값. 셸과 작업 디렉터리(이 저장소에서는 프로젝트가 하위 디렉터리에 있으므로) |
| `env` | 워크플로 전체(또는 작업 하나)의 환경 변수 |
| `jobs.<이름>` | 작업 하나. `runs-on`은 러너의 종류(`ubuntu-latest`) |
| `strategy.matrix` | 값마다 같은 작업을 따로 만들어 동시에 돌림. 매트릭스(matrix) `toolchain: [stable, "1.98"]`이면 작업이 둘. `fail-fast: false`는 하나가 실패해도 나머지를 끝까지 돌림 |
| `steps` | 단계 목록. `uses`는 다른 사람이 만든 액션(action)을 실행하고 `with`로 입력을 주며, `run`은 셸 명령을 실행 |
| `id`와 `$GITHUB_OUTPUT` | 단계가 `이름=값`을 이 파일에 쓰면 뒤 단계가 `steps.<id>.outputs.<이름>`으로 읽음 |
| `if` | 조건이 참일 때만 그 작업을 실행. `${{ … }}`는 GitHub이 실행 전에 계산하는 식(`github.event_name`, `matrix.toolchain` 등) |

액션은 태그(`v7`) 대신 커밋 SHA로 고정하고 버전을 주석으로 붙였습니다. 태그는 나중에 다른 커밋을 가리키도록 옮길 수 있기 때문입니다. `persist-credentials: false`는 저장소를 받아 온 뒤 토큰을 남기지 않게 합니다. 이 두 가지와 권한 좁히기의 이유는 부록 A ch32 2.6절이 다룹니다.

| 작업 | 하는 일 | 핵심 명령 |
| --- | --- | --- |
| `test` | 최신 stable과 MSRV(`rust-version = "1.98"`) 두 툴체인에서 테스트 | `cargo test --locked --all-targets`, `cargo test --locked --doc` |
| `features` | 기능 조합 셋(기본, 없음, 전부) | `cargo test --locked` + `--no-default-features` / `--all-features` |
| `warnings` | 경고가 하나라도 있으면 실패 | `CARGO_BUILD_WARNINGS=deny cargo clippy --locked --all-targets --all-features --keep-going` |
| `latest-deps` | 주 1회, 잠금 파일을 무시하고 최신 의존성으로 | `cargo update`, `cargo test` |

작업마다 알아 둘 Cargo 쪽 사정은 다음과 같습니다.

**`--locked`.** CI의 빌드는 커밋한 `Cargo.lock` 그대로여야 재현할 수 있습니다. `--locked`는 잠금 파일을 고쳐야 하는 상황이면 고치지 않고 실패합니다([ch1](../ch1-getting-started/README.md) 2.4절). 반대로 `latest-deps` 작업은 일부러 `cargo update`로 잠금 파일을 새로 만들어, `cargo add`로 이 패키지를 의존성에 더하는 사용자가 받게 될 최신 버전에서도 동작하는지 봅니다. Cargo Book 2.8절은 `cargo install` 사용자도 드는데, 1.100.0의 `cargo install`은 패키지에 든 `Cargo.lock`을 기본으로 쓰므로 그 경우에는 해당하지 않습니다(`cargo install` 문서의 "Dealing with the Lockfile"). Cargo Book은 이 확인 방법으로 잠금 파일을 커밋하지 않기, 실패해도 계속하는 CI 작업, 정기 예약 작업, Dependabot·RenovateBot 같은 자동 PR을 들고 각각의 대가를 적습니다. 이 예시는 정기 예약 작업을 골랐습니다. 예약 실행은 원래 PR을 막지 않으므로 `continue-on-error`는 두지 않았습니다. 두면 실패해도 실행 전체가 성공으로 표시되어 실패가 눈에 띄지 않습니다. 환경 변수 `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=allow`는 리졸버 3이 `rust-version`에 맞는 버전으로 제한하지 않고 가장 새 버전을 고르게 합니다([Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.6절).

**MSRV 확인.** `rust-version`이 선언하는 MSRV는 선언일 뿐이라 지켜지는지 확인해야 합니다. ch3 2.4절에서 Clippy의 `incompatible_msrv` 린트가 더 새로운 표준 라이브러리 기능을 잡는 것을 봤는데, 가장 확실한 확인은 **그 버전의 툴체인으로 실제로 빌드하는 것**입니다. 워크플로의 `test` 작업은 매트릭스에 `"1.98"`을 넣어 그렇게 합니다. 이 컴퓨터에는 1.98.1이 설치돼 있어 로컬에서도 `cargo +1.98.1 test --locked`로 같은 확인을 할 수 있었습니다(`+툴체인`은 rustup이 고른 툴체인으로 그 명령을 실행하는 문법). Cargo Book은 이 확인을 도와주는 서드파티 도구로 `cargo-msrv`와 `cargo-hack`을 소개합니다. 이 저장소에는 설치하지 않았습니다. 1.98.1 툴체인으로 실행하면 `Running` 줄의 경로가 `target/debug/deps/…`로 나옵니다. build 디렉터리의 구조가 1.100.0에서 바뀌었기 때문입니다(ch6 2.5절).

**기능 조합.** [ch5](../ch5-features/README.md) 2.6절에서 기능이 있는 패키지는 기본·없음·전부 세 조합을 확인하라고 했습니다. 워크플로의 `features` 작업은 그것을 셸 반복문으로 합니다. 기능이 많으면 조합이 기하급수로 늘어나므로, 기능을 하나씩 켜 보는 등 줄인 집합을 돌려 주는 `cargo-hack`(예: `cargo hack test --each-feature`)을 씁니다. 이것도 설치하지 않아 실행해 보지 않았습니다.

**경고 검사.** Cargo Book은 개발할 때는 경고를 허용하되 공식 브랜치는 경고가 없게 하는 관례를 들고, `build.warnings = "deny"`(환경 변수형 `CARGO_BUILD_WARNINGS=deny`, ch7 2.5절)로 CI를 실패시키라고 권합니다. `--keep-going`은 첫 실패에서 멈추지 않고 모든 크레이트의 경고를 한 번에 보여 줍니다. 새 툴체인이 새 경고를 들여올 수 있으므로 툴체인을 고정하고 올리는 일을 따로 관리하라는 권고도 함께 있습니다(부록 A ch32 2.3절).

**Cargo home 캐시.** CI의 가상 머신은 매번 빈 상태로 시작하므로, 그대로 두면 실행할 때마다 의존성을 내려받고 처음부터 컴파일합니다. 내려받은 크레이트가 있는 Cargo home의 `registry/index/`·`registry/cache/`·`git/db/`([ch1](../ch1-getting-started/README.md) 2.6절)와 `target/`을 캐시해 두면 다음 실행이 빨라집니다. 워크플로의 `actions/cache` 단계가 그것입니다. 풀린 소스(`registry/src/`)는 캐시에서 되살린 `.crate`에서 다시 만들 수 있어 넣지 않았습니다. 같은 키의 캐시가 이미 있으면 새로 저장하지 않으므로, 키에는 `Cargo.lock`의 해시와 **실제로 설치된 컴파일러 버전**(`rustc -V`의 출력)을 넣었습니다. 매트릭스 값 `stable`을 그대로 넣으면 stable이 새 버전으로 올라가도 키가 같아, 옛 컴파일러로 만든 `target/`이 계속 되살아납니다.

**워크스페이스.** 워크스페이스라면 `cargo test --workspace`처럼 `--workspace`를 붙입니다. 가상 워크스페이스는 기본이 모든 멤버지만, `default-members`가 있거나 루트 패키지가 있으면 일부만 검사하기 때문입니다(ch4 2.4절).

## 3. 예제 안내

### 01_custom_harness — `harness = false`인 테스트 타깃

[`examples/01_custom_harness.rs`](examples/01_custom_harness.rs)는 libtest 없이 검사 사례를 직접 돌리고, 실패가 있으면 `ExitCode::FAILURE`(1)를 돌려줍니다. 이 챕터의 `Cargo.toml`에 `test = true`, `harness = false`를 줬으므로 `cargo test`가 이 `main`을 테스트로 실행합니다.

```console
$ cargo test -q -p p3-ch9-testing-ci --example 01_custom_harness
받은 인자: []
어는점 ... ok
끓는점 ... ok
2개 중 0개 실패
$ cargo test -q -p p3-ch9-testing-ci --example 01_custom_harness -- 실패
받은 인자: ["실패"]
어는점 ... ok
끓는점 ... ok
틀린 기대값 ... 실패: 37°C → 98.6°F, 기대 100°F
3개 중 1개 실패
error: test failed, to rerun pass `-p p3-ch9-testing-ci --example 01_custom_harness`

Caused by:
  process didn't exit successfully: `…/target/debug/build/p3-ch9-testing-ci/<해시>/out/01_custom_harness-<해시> '실패'` (exit status: 1)
```

두 번째 명령의 종료 상태는 1입니다. `--` 뒤의 `실패`가 libtest의 이름 거르기가 아니라 `main`의 인자로 그대로 들어간 것을 볼 수 있습니다. libtest의 `running N tests`나 `test result:` 줄이 없는 것도 테스트 러너가 없기 때문입니다. 같은 파일을 `cargo run -q -p p3-ch9-testing-ci --example 01_custom_harness`로 실행해도 같은 `main`이 돕니다.

### 02_profile_dependent — 프로필에 따라 갈리는 판정

[`examples/02_profile_dependent.rs`](examples/02_profile_dependent.rs)의 테스트 셋 가운데 `overflow_panics`는 오버플로 패닉을 기대합니다(2.4절).

```console
$ cargo test -p p3-ch9-testing-ci --example 02_profile_dependent
    Finished `test` profile [unoptimized + debuginfo] target(s) in …s
…
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
$ cargo test -p p3-ch9-testing-ci --example 02_profile_dependent --release
    Finished `release` profile [optimized] target(s) in …s
…
test tests::overflow_panics - should panic ... FAILED
…
note: test did not panic as expected at part3-cargo/ch9-testing-ci/examples/02_profile_dependent.rs:31:8
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
error: test failed, to rerun pass `-p p3-ch9-testing-ci --example 02_profile_dependent`
```

`--release`의 종료 상태는 101입니다. `Finished` 줄에서 `--release`로 테스트하면 `test`가 아니라 `release` 프로필을 쓴다는 것도 볼 수 있습니다. 예제의 `main`은 `cargo run`으로 실행하면 `200 + 50 = 250`을 출력합니다.

### projects/linestat — 테스트 종류를 모두 갖춘 패키지와 CI

표준 입력의 줄·단어·글자 수를 세는 라이브러리 겸 바이너리입니다.

| 파일 | 담은 것 |
| --- | --- |
| [`Cargo.toml`](projects/linestat/Cargo.toml) | `rust-version = "1.98"`, 기본 기능 `top`, `[[bench]] harness = false` |
| [`src/lib.rs`](projects/linestat/src/lib.rs) | `count`, `top_words`(기능 `top`), 문서 테스트 3개, 단위 테스트 |
| [`src/main.rs`](projects/linestat/src/main.rs) | 명령줄 프로그램과 그 단위 테스트(바이너리의 단위 테스트) |
| [`tests/api.rs`](projects/linestat/tests/api.rs), [`tests/cli.rs`](projects/linestat/tests/cli.rs) | 라이브러리를 쓰는 통합 테스트, 실행 파일을 실행하는 통합 테스트(`CARGO_BIN_EXE_linestat`) |
| [`benches/count.rs`](projects/linestat/benches/count.rs) | `harness = false` 벤치마크 |
| [`examples/demo.rs`](projects/linestat/examples/demo.rs) | `cargo test`가 컴파일되는지 확인하는 예제 |
| [`ci/github-workflow.yml`](projects/linestat/ci/github-workflow.yml) | 2.6절의 CI 워크플로 예시. 실제로 쓰려면 저장소 루트의 `.github/workflows/`로 옮깁니다 |

프로젝트 디렉터리에서 실행합니다. 시간 값과 해시는 실행마다 다릅니다.

| 명령 | 기대 결과 |
| --- | --- |
| `cargo test --locked` | 2.1절의 다섯 구역, 모두 통과 |
| `cargo test --locked count` / `cargo test --locked -- count` | 앞은 `Doc-tests` 구역 없음, 뒤는 문서 테스트 `1 passed; … 2 filtered out`(2.2절) |
| `cargo test --locked --all-targets` | `Doc-tests` 구역이 없고 `benches/count.rs`, `examples/demo.rs`가 테스트 모드로 실행됨 |
| `cargo test --locked --no-run` | `Executable …` 네 줄 |
| `cargo bench --locked` | 아래 출력 |
| `printf 'x y x\nz\n' \| cargo run -q --locked -- --top 2` | `줄 2 / 단어 4 / 글자 8`, `  2 x`, `  1 y` |
| 같은 입력으로 `cargo run -q --locked --no-default-features -- --top 2` | 첫 줄 뒤에 표준 에러로 `--top 2: top 기능 없이 빌드했습니다` |
| `cargo +1.98.1 test --locked` | 1.98.1 툴체인에서도 모두 통과(1.98.1이 설치돼 있을 때) |
| 워크플로의 작업별 명령(2.6절 표) | `features`의 세 조합 모두 통과, `warnings`의 Clippy 경고 없음 |
| `cargo clean` | `target/` 삭제(실습을 마친 뒤) |

```console
$ cargo bench --locked
    Finished `bench` profile [optimized] target(s) in …s
     Running unittests src/lib.rs (target/release/build/linestat/<해시>/out/linestat-<해시>)
…
     Running unittests src/main.rs (target/release/build/linestat/<해시>/out/linestat-<해시>)
…
     Running benches/count.rs (target/release/build/linestat/<해시>/out/count-<해시>)
받은 인자: ["--bench"]
count: 줄 20000 / 단어 60000 / 글자 240000
20번 가운데 가장 짧은 시간: 508.709µs
```

마지막 줄의 시간은 이 컴퓨터에서 잰 한 번의 값입니다. 단위 테스트 실행 파일 두 구역은 `#[test]`를 모두 `ignored`로 건너뛰므로 줄였습니다.

## 4. 자주 하는 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| `cargo test --all-targets`만 CI에 둠 | 문서 테스트가 돌지 않음(메시지 없음) | `cargo test --doc`을 따로 둡니다(2.2절). |
| `cargo test 이름`으로 문서 테스트까지 거르려 함 | `Doc-tests` 구역이 사라짐 | `cargo test -- 이름`처럼 `--` 뒤에 씁니다. |
| `harness = false` 타깃이 실패해도 `0`을 돌려줌 | 실패를 출력해도 테스트가 통과로 판정됨 | 실패하면 0이 아닌 종료 상태를 돌려줍니다(예제 01의 `ExitCode`). |
| 오버플로 패닉이나 `debug_assert!`에 기대는 테스트 | `--release`에서 `test did not panic as expected` 등으로 실패(예제 02) | 검사 연산(`checked_*`)으로 의도를 밝히고, 배포 빌드로도 테스트합니다. |
| stable에서 `#[bench]`를 씀 | `test` 크레이트를 쓰려고 `#![feature(test)]`를 적으면 ``error[E0554]: `#![feature]` may not be used on the stable release channel``([부록 A ch30](../../appd-a/ch30-beyond-unit-tests/README.md) 4.1절) | `harness = false` 벤치마크나 `criterion`을 씁니다(2.5절). |
| 실행 파일 하나가 실패해 뒤 테스트를 못 봄 | `error: test failed, to rerun pass …` 뒤 멈춤 | `--no-fail-fast`로 끝까지 돌립니다. |
| CI에서 `--locked` 없이 빌드 | 잠금 파일이 말없이 바뀌어 로컬과 다른 버전으로 테스트할 수 있음 | `--locked`를 붙입니다. 최신 의존성 확인은 별도 작업으로 둡니다. |
| `rust-version`을 선언만 하고 확인하지 않음 | 더 새로운 API를 써도 최신 툴체인에서는 통과함 | 그 버전의 툴체인으로 빌드하는 CI 작업을 둡니다(연습 3). |

## 5. 연습 문제

연습마다 `projects/linestat`를 저장소 밖에 새로 복사해 시작합니다.

### 연습 1. 실패를 끝까지 보기

`tests/api.rs`의 기대값을 일부러 틀리게(`lines: 3`) 바꾸고, `tests/cli.rs`의 `prints_counts` 기대값도 틀리게 바꾸세요. `cargo test`와 `cargo test --no-fail-fast`를 각각 실행해, 어느 구역까지 실행되는지와 마지막에 보고되는 실패의 수를 비교하세요.

<details>
<summary>힌트</summary>

구역은 `src/lib.rs` → `src/main.rs` → `tests/api.rs` → `tests/cli.rs` → 문서 테스트 순서로 돕니다. `cargo test`는 `tests/api.rs` 실행 파일이 실패하면 거기서 멈추므로 `tests/cli.rs`와 문서 테스트는 실행되지 않습니다. `--no-fail-fast`를 주면 끝까지 돌고, 마지막에 실패한 실행 파일들을 모아 `error: 2 targets failed:`처럼 보고합니다.

</details>

### 연습 2. 통합 테스트 하나만 돌리기

`tests/cli.rs`만 실행하는 명령, 라이브러리의 단위 테스트만 실행하는 명령, 문서 테스트만 실행하는 명령을 각각 쓰세요. 그다음 문서 테스트 가운데 `top_words`의 것만 실행해 보세요.

<details>
<summary>힌트</summary>

`cargo test --test cli`, `cargo test --lib`, `cargo test --doc`입니다. 문서 테스트의 이름은 `src/lib.rs - top_words (line …)` 꼴이므로 `cargo test --doc -- top_words`로 거릅니다. `--doc`이 문서 테스트만 고르고, `--` 뒤의 이름이 그 안에서 거릅니다.

</details>

### 연습 3. MSRV가 거짓이 되는 순간

`src/main.rs`의 `main`에 1.100.0에서 안정화된 `bool::toggle`을 쓰는 코드(`let mut v = false; v.toggle();`)를 넣으세요. `cargo test`(1.100.0), `cargo clippy`, `cargo +1.98.1 check`의 결과를 비교하고, CI의 어느 작업이 이것을 잡는지 말하세요. 1.98.1이 설치돼 있지 않으면 `cargo clippy`까지만 확인합니다.

<details>
<summary>힌트</summary>

1.100.0의 `cargo test`는 통과합니다. `cargo clippy`는 ``current MSRV (Minimum Supported Rust Version) is `1.98.0` but this item is stable since `1.100.0` `` 경고를 냅니다(경고라서 `CARGO_BUILD_WARNINGS=deny`인 `warnings` 작업에서 실패). `cargo +1.98.1 check`는 ``error[E0599]: no method named `toggle` found for type `bool` in the current scope``로 실패하며, 워크플로의 `test (1.98)` 작업이 이것을 잡습니다.

</details>

### 연습 4. 벤치마크가 테스트 모드를 알아채게 하기

`benches/count.rs`가 `--bench` 인자를 받았을 때만 20번 재고, 받지 않았을 때(`cargo test --all-targets`)는 한 번만 계산해 결과가 맞는지만 확인하고 끝나게 고치세요.

<details>
<summary>힌트</summary>

`let bench = std::env::args().any(|a| a == "--bench");`로 정하고, `bench`가 아니면 `count`의 결과를 `assert_eq!`로 확인한 뒤 `return`합니다. `cargo bench`는 `--bench`를 붙여 실행하고 `cargo test --all-targets`는 붙이지 않으므로(2.5절), 테스트 모드에서는 시간을 들이지 않고 벤치마크 코드가 계속 컴파일·동작하는지만 확인하게 됩니다.

</details>

## 6. 참고 자료

- The Cargo Book: [2.7 Tests](https://doc.rust-lang.org/cargo/guide/tests.html), [2.8 Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html), [cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html), [cargo bench](https://doc.rust-lang.org/cargo/commands/cargo-bench.html), [3.1.1 Cargo Targets — Configuring a target](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#configuring-a-target)
- The Rust Programming Language: [11. Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
