# ch30. 단위 테스트보다 더 많이 테스트하기

[Part 0 ch11](../../part0-getting-started/ch11-testing/README.md)에서 `#[test]`와 `cargo test`의 사용법을 배웠습니다. 이 챕터는 그다음 질문을 다룹니다. 단위 테스트, 통합 테스트, 문서 테스트, 예제 프로그램, 벤치마크, 퍼즈 테스트는 각각 무엇을 볼 수 있고 무엇을 볼 수 없을까요? 그래서 어떤 문제를 어느 종류에 맡겨야 할까요? 작은 이진 코덱 라이브러리 하나에 여섯 종류를 모두 갖춘 뒤, 사본마다 변경을 하나씩 넣어 어느 종류가 그 변경을 잡는지 확인합니다. 단위 테스트에 무엇을 넣고 뺄지, 버그를 고칠 때 테스트를 어떤 순서로 쓸지는 작은 예제로 봅니다.

- 원제: [Item 30: Write more than unit tests](https://effective-rust.com/testing.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch30-beyond-unit-tests`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

책의 5장(Tooling)에 속한 Item입니다. 다음 본편·부록 내용을 안다고 보고 진행합니다.

- [Part 0 ch11](../../part0-getting-started/ch11-testing/README.md): 단언 매크로, `#[should_panic]`, `cargo test`의 옵션과 출력, `tests/`의 통합 테스트, 문서 테스트 맛보기
- [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.10절: `main`이 `Err`를 돌려주면 `Error: `와 디버그 형식을 출력하고 종료 코드 1로 끝나는 것
- [Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md) 2.8절: 2024 에디션의 문서 테스트 병합
- [ch18](../ch18-dont-panic/README.md): 패닉판과 `# Panics` 문서, 테스트 코드는 패닉해도 된다는 것
- [ch20](../ch20-avoid-over-optimizing/README.md) 2.5절: 세다와 재다, `black_box`, 측정의 함정
- [ch21](../ch21-semver/README.md) 2.4절: 컴파일러가 잡지 못하는 동작 변경
- [ch23](../ch23-avoid-wildcard-imports/README.md), [ch26](../ch26-feature-creep/README.md) 2.6절: 테스트 모듈의 `use super::*`, 기능 조합 검사

`cargo test`·`cargo bench`의 Cargo 쪽 상세는 [Part 3 ch9](../../part3-cargo/ch9-testing-ci/README.md), 문서 테스트 코드 블록의 속성과 숨긴 줄(`#`)은 Part 4 `ch3-doc-tests`, 문서 주석에 무엇을 쓸지는 [`ch27-document-public-interfaces`](../ch27-document-public-interfaces/README.md), CI 구성은 [`ch32-continuous-integration`](../ch32-continuous-integration/README.md), 도구 전반은 [`ch31-tooling-ecosystem`](../ch31-tooling-ecosystem/README.md)에서 다룹니다. 이 README에서 "예제 01"처럼 번호로 부르는 것은 챕터 패키지의 `examples/` 파일이고, **예제 프로그램**은 라이브러리 패키지의 `examples/`에 두는 프로그램(이 챕터에서는 `projects/tinytlv/examples/`)입니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 단위·통합·문서 테스트, 예제 프로그램, 벤치마크, 퍼즈 테스트가 각각 어디서 무엇을 보는지 설명하고, 한 문제를 어느 종류에 맡길지 고를 수 있다.
2. 단위 테스트에서 타입 시스템이 이미 보장하는 것을 빼고 보장하지 않는 것을 겨냥하며, 기대고 있는 의존성의 동작을 조기 경보 테스트로 적어 둘 수 있다. 테스트가 정말 버그를 잡는지 버그를 넣어 확인할 수 있다.
3. 예제 프로그램을 `main() -> Result<(), Box<dyn Error>>`와 `?`로 쓰고, `cargo test`, `cargo test --examples`, `cargo run --example`, `cargo test --doc`이 각각 무엇을 실행하는지 구별할 수 있다.
4. `criterion`으로 벤치마크를 작성하고(`harness = false`, `black_box`, 기준선 비교), 그 판정을 흔들림이 있는 측정값으로 읽을 수 있다.
5. 버그를 고칠 때 재현 테스트가 실패하는 것을 먼저 보고, 임의의 바이트를 받는 진입점에 퍼즈 테스트가 필요한 이유와 정규 CI에서 돌리지 않는 이유를 설명할 수 있다.

## 2. 핵심 개념

2.1절에서 테스트 종류의 지도와 변경 실험의 결과를 보이고, 2.2~2.8절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 2.9절은 테스트를 돌리는 범위에 대한 책의 조언입니다.

실습 라이브러리는 [`projects/tinytlv`](projects/tinytlv)입니다. 레코드 하나를 `[태그 1바이트][길이 1바이트][값 0~255바이트]`로 기록하는 TLV(태그-길이-값) 코덱이고, 공개 API는 `Record`(`new`, `tag`, `value`, `encoded_len`, `write_to`), `encode`, `parse`, 에러 타입 `ParseError`와 `ValueTooLong`뿐입니다. `parse(&[u8])`는 파일이나 네트워크에서 온 아무 바이트열이나 받는 진입점입니다.

### 2.1 지도: 테스트 종류마다 보는 자리가 다르다

| 종류 | tinytlv에서의 위치 | 보는 것 | 실행 명령 | 언제 돌리나 |
| --- | --- | --- | --- | --- |
| 단위 테스트 | `src/` 안의 `#[cfg(test)]` 모듈 | 비공개 항목까지 전부 | `cargo test` (`--lib`) | 변경마다 |
| 통합 테스트 | `tests/*.rs`, 파일마다 별도 크레이트 | 공개 API만, 동작 위주 | `cargo test` (`--test 이름`) | 변경마다 |
| 문서 테스트 | 문서 주석의 코드 블록 | 공개 항목 하나하나의 사용법 | `cargo test` (`--doc`) | 변경마다 |
| 예제 프로그램 | `examples/*.rs` | 공개 API 전체의 흔한 사용법 | `cargo run --example 이름` | 변경마다 |
| 벤치마크 | `benches/*.rs` (`criterion`) | 성능(경과 시간) | `cargo bench` | 정기적으로 |
| 퍼즈 테스트 | 별도 크레이트 `fuzz/` (이 저장소에는 없음) | 공격자가 고를 수 있는 입력 | `cargo fuzz run 타깃` | 릴리스·큰 변경 때, 시간을 정해서 |

**변경 실험.** tinytlv의 사본마다 변경을 하나씩 넣고 종류별로 실행했습니다. ✗는 그 종류가 변경을 잡았다(실패했다)는 뜻이고, 벤치마크는 ⑥에서만 비교했습니다.

| 변경 | 단위 | 통합 `public_api` | 임의 바이트 `arbitrary_bytes` | 문서 | 예제 실행 | 벤치마크 | 절 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ① 비공개 `Cursor::take`에서 넘침 검사(`checked_add`)를 뺌 | ✗ 1개 | 통과 | 통과 | 통과 | 통과 | | 2.2 |
| ② `ParseError`의 `PartialEq`를 `#[cfg_attr(test, derive(…))]`로 바꿈 | 통과 | ✗ 컴파일 에러 | ✗ 컴파일 에러 | ✗ 1개 | 통과 | | 2.4 |
| ③ `value()`를 `bytes()`로 바꾸고 문서 주석은 그대로 둠 | 통과 | 통과 | 통과 | ✗ 2개 | 통과 | | 2.4 |
| ④ 값의 최대 길이를 255에서 32로 줄이고, 깨진 단위 테스트는 새 값에 맞춤 | 통과 | 통과 | 통과 | 통과 | ✗ 종료 코드 1 | | 2.5 |
| ⑤ 길이 바이트가 있는지 확인하지 않음(회귀 테스트도 아직 없음) | 통과 | 통과 | ✗ 2개 | 통과 | 통과 | | 2.7 |
| ⑥ `encode`가 레코드마다 지금까지의 결과를 통째로 다시 복사함 | 통과 | 통과 | 통과 | 통과 | 통과 | ✗ 느려짐 | 2.6 |

모든 종류가 함께 실패한 줄도, 모든 줄을 잡은 종류도 없습니다. 각 종류는 다른 종류가 보지 못하는 자리를 맡고 있습니다. 실험 방법은 [`examples/01_what_types_guarantee.rs`](examples/01_what_types_guarantee.rs)와 같습니다. "버그를 넣었을 때 실패하지 않는 테스트는 그 버그로부터 아무것도 지키지 않는다"는 생각입니다.

### 2.2 지침 1: 단위 테스트는 타입이 보장하지 않는 것을 겨냥한다

**문제 상황.** 0~100점을 등급으로 바꾸는 `grade(score: u8) -> Result<Grade, OutOfRange>`를 테스트합니다. 무엇을 확인해야 할까요?

**비권장 코드.** 타입이 이미 보장하는 것을 확인합니다([`01_what_types_guarantee.rs`](examples/01_what_types_guarantee.rs)의 `suite_type_guarantees`).

```rust
let non_negative = score >= 0; // u8에는 음수가 없음
let known_grade = matches!(g, Grade::A | Grade::B | Grade::C | Grade::F); // 열거형이라 다른 값이 없음
```

이 묶음은 정상 판에서도, 버그를 넣은 두 판에서도 통과합니다(3절). 컴파일러도 이 검사가 쓸모없다는 것을 압니다. 첫 줄은 rustc `unused_comparisons`(`comparison is useless due to type limits`, 기본 warn)와 clippy `absurd_extreme_comparisons`(기본 deny)에 걸려서, 예제는 `#[expect]`를 붙였습니다. "음수 점수를 거부하는가"는 아예 쓸 수조차 없습니다. `grade(-1)`은 E0600입니다(4.2절). 책도 같은 말을 합니다. 타입 시스템이 보장하는 것까지 테스트할 필요는 없고, 그 보장이 깨졌다면 테스트 실패보다 큰 문제가 있는 것입니다.

**권장 코드.** 타입이 막아 주지 않는 것을 확인합니다. `u8`은 0~255를 담으므로 101~255는 실제로 들어올 수 있는 입력이고, 등급 경계는 타입과 상관없는 규칙입니다.

```rust
let cases = [(0, Ok(F)), (59, Ok(F)), (60, Ok(C)), (79, Ok(C)), /* … */ (100, Ok(A)),
             (101, Err(OutOfRange(101))), (255, Err(OutOfRange(255)))];
```

이 묶음은 두 버그를 모두 잡습니다. 예제의 `boundary_suite_catches_both_bugs`처럼 "버그를 넣은 판에서는 실패한다"를 확인해 보면, 테스트가 실제로 무언가를 지키는지 알 수 있습니다.

**단위 테스트만 닿는 곳.** 단위 테스트는 같은 크레이트 안에 있어 비공개 항목까지 씁니다. tinytlv의 비공개 타입 `Cursor`는 "어떤 인자에도 패닉하지 않는다"고 약속합니다. 그런데 `parse`는 길이를 1바이트에서 읽으므로 `take`에 255보다 큰 `n`을 넘기는 일이 없고, `pos + n`이 넘치는 경로는 공개 API로는 닿을 수 없습니다. 그래서 변경 ①은 다른 테스트를 모두 통과하고 단위 테스트 `huge_take_does_not_overflow` 하나만 `attempt to add with overflow`로 실패했습니다. 또 `src/tests.rs`의 `parse_builds_records_directly`는 비공개 필드로 기대값 `Record { tag: 1, value: vec![0xaa, 0xbb] }`를 직접 만들어, `Record::new`에 버그가 있어도 `parse`만 따로 확인합니다.

책이 단위 테스트에 덧붙이는 것들입니다.

- 테스트 모듈의 `use super::*`는 글롭 가져오기를 피하라는 ch23의 예외입니다.
- 테스트 코드에는 ch18의 "패닉하지 말라"가 적용되지 않습니다. 실패하면 패닉하는 것이 테스트의 일이므로 `unwrap()`과 `expect()`를 씁니다.
- 잘못된 입력에 패닉하는 공개 함수는 `#[should_panic]`으로 그 패닉을 테스트하고, 문서에 `# Panics` 절을 둡니다(ch18 2.3절). tinytlv의 `Record::write_to`는 버퍼가 작으면 패닉하고(표준 라이브러리의 `char::encode_utf8`과 같은 관례), `write_to_panics_on_short_buffer`가 그것을 확인합니다.
- 테스트 모듈은 같은 파일에 두어도 되고 `#[cfg(test)] mod tests;`로 별도 파일에 두어도 됩니다. tinytlv의 `cursor.rs`는 같은 파일에, `lib.rs`는 `src/tests.rs`에 둡니다.

**지침을 따르지 않아도 되는 경우.** 그 보장을 컴파일러가 아니라 사람이 지키는 곳, 곧 `unsafe` 코드와 FFI 경계에서는 "타입이 보장한다"는 것이 약속일 뿐입니다. 불변식을 확인하는 테스트를 두고 Miri로 돌립니다([ch16](../ch16-avoid-unsafe/README.md)).

### 2.3 지침 2: 기대고 있는 의존성의 동작은 조기 경보 테스트로 적어 둔다

**문제 상황.** 연락처 CSV를 읽는 코드가 의존성 `fieldsplit`의 `split`에 기댑니다. 칸의 **위치**로 이름·이메일·전화번호를 읽으므로 `split`이 빈 칸을 지우지 않아야 합니다. `^1.4` 요구 사항이면 `cargo update` 한 번에 1.5.0이 들어올 수 있습니다(ch21). 1.5.0은 "빈 칸 정리"를 개선으로 넣었고 시그니처는 그대로입니다.

**비권장 코드.** 내 기능만 테스트합니다([`02_early_warning_dependency.rs`](examples/02_early_warning_dependency.rs)의 `feature_test`). 흔한 줄 `김하늘, sky@example.com, 010-1234-5678`은 두 버전에서 모두 맞게 읽히므로 이 테스트는 1.5.0에서도 통과합니다. 그동안 이메일이 빈 줄에서는 전화번호가 이메일 칸으로 밀립니다.

**권장 코드.** 내 코드가 기대는 동작 **하나만** 확인하는 테스트를 따로 둡니다.

```rust
/// 의존성에 대한 가정: `split`은 빈 칸을 지우지 않는다.
#[test]
fn dependency_keeps_empty_fields() {
    assert_eq!(fieldsplit_1_4::split("a,,b"), ["a", "", "b"]);
}
```

1.5.0으로 올리는 순간 이 테스트가 실패하고, 이름과 메시지가 원인을 바로 가리킵니다. 기능 테스트가 운 좋게 실패하더라도 "이메일 칸이 틀렸다"는 증상에서 의존성까지 거슬러 올라가야 하는 것과 다릅니다. 책은 이것을 동작 변화에 대한 조기 경보라고 부릅니다. 버전 번호만 보고는 알 수 없는 변화라서 필요합니다. 의존성 관리자는 개선이라 여기고 마이너 버전으로 냈을 수 있고, ch21 2.4절에서 본 대로 동작 변경은 컴파일러가 잡지 못합니다.

**지침을 따르지 않아도 되는 경우.** 문서가 분명히 약속한 동작(예: `slice::sort`는 안정 정렬)을 모두 다시 테스트할 필요는 없습니다. 조기 경보가 필요한 것은 문서가 약속하지 않았거나, 내 코드가 그 세부에 크게 기대는 동작입니다. 문서가 "약속하지 않는다"고 밝힌 동작(`HashMap`의 순회 순서, 디버그 형식의 출력 모양)에 기대는 코드라면 테스트보다 코드를 고칩니다.

### 2.4 지침 3: 통합 테스트와 문서 테스트로 사용자 자리에서 본다

**문제 상황.** 단위 테스트는 크레이트 안쪽에서 보므로 사용자에게 보이는 모습과 다른 것을 볼 수 있습니다. `#[cfg(test)]`는 그 크레이트의 단위 테스트를 빌드할 때만 켜지고, 문서 주석 안의 코드는 단위 테스트가 컴파일하지 않습니다.

**비권장 코드 ①: 테스트에서만 필요한 트레이트를 테스트 빌드에서만 구현합니다.** 에러를 `assert_eq!`로 비교하려면 `PartialEq`가 필요한데, 공개 API에 약속하기 싫어서 `#[cfg_attr(test, derive(PartialEq, Eq))]`로 씁니다(변경 ②). 단위 테스트 9개는 통과합니다. 통합 테스트는 이렇게 실패합니다.

```text
error[E0369]: binary operation `==` cannot be applied to type `Result<Vec<Record>, ParseError>`
   --> tests/public_api.rs:25:5
    |
 25 |     assert_eq!(parse(&[]), Ok(vec![]));
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
note: `ParseError` does not implement `PartialEq`
   --> src/lib.rs:131:1
    |
131 | pub enum ParseError {
    | ^^^^^^^^^^^^^^^^^^^ `ParseError` is defined in another crate
```

note의 "defined in another crate"가 핵심입니다. 통합 테스트에게 tinytlv는 남의 크레이트이고, 사용자에게도 그렇습니다. "사용자는 에러를 비교할 수 없다"는 사실을 단위 테스트는 알려 주지 못했습니다. `parse`의 문서 테스트도 같은 E0369로 실패했습니다. 같은 이유로 라이브러리에 `#[cfg(test)]`로 둔 도우미 함수는 통합 테스트에서 쓸 수 없습니다.

**비권장 코드 ②: 문서 주석의 예제를 따로 둡니다.** 변경 ③은 `Record::value`를 `bytes`로 바꾸며 `src/`, `tests/`, `examples/`의 코드는 모두 고쳤지만 문서 주석은 그대로 두었습니다. 단위·통합 테스트는 통과하고 예제 프로그램도 컴파일되며, 문서 테스트 2개만 실패합니다.

```text
---- src/lib.rs - Record::new (line 41) stdout ----
error[E0599]: no method named `value` found for struct `Record` in the current scope
  --> src/lib.rs:47:14
```

문서 주석은 컴파일러에게 주석일 뿐이지만, `cargo test`는 그 안의 코드 블록을 사용자 크레이트처럼 컴파일하고 실행합니다. 문서의 예제가 API와 어긋나는 것을 잡는 것은 문서 테스트뿐입니다.

**권장 코드.** 공개 API가 약속할 트레이트는 그냥 derive합니다(tinytlv의 `ParseError`). 통합 테스트는 내부가 아니라 **동작**을 확인합니다. [`tests/public_api.rs`](projects/tinytlv/tests/public_api.rs)는 왕복(`encode` 뒤 `parse`), 빈 입력, 에러의 비교와 메시지, 사용자 함수에서 `?`로 이어 쓰기를 확인할 뿐 `Cursor`를 모릅니다. 그래서 내부를 다시 짜도 고칠 필요가 없습니다. 공개 항목마다 문서 테스트를 두고(tinytlv는 5개), 하나만 돌릴 때는 이름으로 거릅니다.

```sh
cargo test --doc parse    # test result: ok. 1 passed; 0 failed; … 4 filtered out
```

**지침을 따르지 않아도 되는 경우.** 내부 구조에 묶인 세부(예: 실패한 `take`가 위치를 옮기지 않는다는 커서의 규칙)는 공개 API로 확인하기 어렵고 그럴 필요도 없으니 단위 테스트에 둡니다. 바이너리만 있는 패키지는 통합 테스트가 가져다 쓸 라이브러리가 없으므로, Part 0 ch11 2.8절처럼 기능을 `src/lib.rs`로 옮기는 것이 먼저입니다.

### 2.5 지침 4: 예제 프로그램은 사용자 코드처럼 쓰고, 빌드만이 아니라 실행한다

**문제 상황.** 예제 프로그램은 공개 API를 처음부터 끝까지 쓰는 모습을 보여 주고, 사용자는 그 코드를 그대로 옮겨 씁니다. 그런데 `cargo test`는 예제 프로그램을 **빌드만** 하고 실행하지 않습니다.

**비권장 코드 ①: `unwrap()`으로 씁니다.** [`03_example_program_style.rs`](examples/03_example_program_style.rs)의 `total_with_unwrap`입니다. 잘못된 입력에서는 사용자에게 의미 없는 소스 위치와 함께 패닉하고 종료 코드 101로 끝납니다(3절). 더 큰 문제는 사용자가 이 코드를 자기 함수로 옮기면 패닉도 함께 옮겨 간다는 것입니다(ch18).

**권장 코드 ①.** `main`이 `Result<(), Box<dyn Error>>`를 돌려주고 `?`로 잇습니다(`total_with_question_mark`). 실패하면 `Error: UnknownUnit { unit: 'x' }`를 출력하고 종료 코드 1로 끝나며, 코드는 `Result`를 돌려주는 사용자 함수에 그대로 들어갑니다. tinytlv의 [`sensor_report`](projects/tinytlv/examples/sensor_report.rs)는 슬라이스를 배열로 바꾸는 `try_into()?`와 `str::from_utf8(…)?`처럼 종류가 다른 에러도 `?` 하나로 넘깁니다. 예제 프로그램에는 경계 사례를 넣지 않습니다. 구석진 경우는 단위 테스트가 맡고, 예제는 흔한 사용법을 보입니다.

**비권장 코드 ②: `cargo test`만 돌립니다.** 변경 ④는 값의 최대 길이를 255에서 32로 줄이고, 그 때문에 깨진 단위 테스트는 새 값에 맞게 고쳤습니다. `cargo test`(21개)와 `cargo test --examples`가 모두 통과합니다. 그러나 `sensor_report`를 실행하면 42바이트짜리 장치 이름 레코드가 거부됩니다.

```text
$ cargo run --example sensor_report
Error: ValueTooLong { len: 42 }
```

세 명령이 예제 프로그램에 하는 일은 다음과 같습니다(Cargo Book과 tinytlv에서 확인).

| 명령 | 예제를 컴파일 | 예제 안의 `#[test]` 실행 | `main` 실행 |
| --- | --- | --- | --- |
| `cargo test` | 함(타입 에러가 있으면 실패) | 안 함 | 안 함 |
| `cargo test --examples` | 함(테스트 모드로) | 함 | 안 함 |
| `cargo run --example 이름` | 함 | 안 함 | 함 |

**권장 코드 ②.** 모든 예제 프로그램을 실행합니다. 이 프로젝트를 검증할 때 쓴 반복문입니다.

```sh
for ex in sensor_report inspect; do cargo run -q --example "$ex" || exit 1; done
```

책은 예제가 깨지는 것을 큰 신호로 봅니다. 많은 사용자에게 영향을 줄 진짜 버그이거나, 예제를 고쳐야 할 만큼 API가 바뀌었다는 뜻이기 때문입니다. 변경 ④는 전에 받던 값을 거부하므로 사용자 코드가 실행 중에 깨지는 변경, 곧 메이저 버전을 올려야 하는 변경입니다(ch21).

**지침을 따르지 않아도 되는 경우.** 사람의 입력을 기다리거나 네트워크·하드웨어가 필요한 예제는 CI에서 그대로 실행할 수 없습니다. `inspect`처럼 인자가 없으면 안에 든 입력을 쓰게 만들거나, 그것도 어렵다면 적어도 `cargo test`로 컴파일되는지는 확인합니다.

### 2.6 지침 5: 성능이 중요한 코드는 벤치마크로 재고 기준선과 비교한다

**문제 상황.** ch20은 "먼저 세고, 문제가 될 만한 곳만 잰다"고 했습니다. 성능이 요구 사항인 코드라면 한 번 재고 끝낼 수 없습니다. 변경이 성능을 떨어뜨리는지(성능 회귀) 계속 알아야 하는데, 테스트는 결과가 맞는지만 봅니다.

**비권장 코드 ①: 내장 벤치 하네스.** 표준 라이브러리에 딸린 `#[bench]`와 `test::Bencher`는 1.100.0에서도 nightly 전용입니다. stable에서는 `#![feature(test)]` 줄에서 E0554로 멈춥니다(4.1절).

**비권장 코드 ②: 테스트가 통과했으니 됐다고 봅니다.** 변경 ⑥은 `encode`를 이렇게 바꿉니다.

```rust
out = [out, one].concat(); // 레코드마다 지금까지의 out 전체를 새 Vec에 복사
```

결과는 같으므로 21개 테스트와 두 예제 프로그램이 모두 통과합니다. 레코드가 n개면 복사량이 n²에 비례한다는 것은 벤치마크만 보여 줍니다.

**권장 코드.** stable에서는 `criterion`을 dev-dependency로 두고 벤치 타깃의 내장 하네스를 끕니다([`Cargo.toml`](projects/tinytlv/Cargo.toml)).

```toml
[lib]
bench = false     # 4.4절의 `Unrecognized option`을 피함

[dev-dependencies]
criterion = "0.8.2"

[[bench]]
name = "codec"
harness = false   # benches/codec.rs의 criterion_main!이 main을 만듦
```

[`benches/codec.rs`](projects/tinytlv/benches/codec.rs)는 레코드 10개와 1000개로 `encode`와 `parse`를 잽니다.

```rust
group.bench_with_input(BenchmarkId::from_parameter(n), &input, |b, input| {
    b.iter(|| encode(black_box(input)))
});
```

`std::hint::black_box`는 ch20에서 본 대로 컴파일러가 입력을 안다고 보고 계산을 미리 하거나 지우지 못하게 하는 "최선의 노력"입니다. criterion은 `iter` 클로저가 돌려준 값을 스스로 `black_box`에 넣습니다(0.8.2 소스에서 확인). 사본에서 입력의 `black_box`를 빼 보았더니 차이가 측정의 흔들림을 넘지 않았습니다. 6바이트짜리 상수 배열을 넣고 결과까지 버리는 클로저(`let _ = parse(&[1, 2, 0xaa, 0xbb, 2, 0]);`)도 28ns 남짓 걸려, 계산이 지워지지 않았습니다. `parse`가 할당을 하는 함수라서 그랬을 뿐 보장은 아닙니다. 계산이 지워졌는지 의심스러우면 책의 말대로 Compiler Explorer(Godbolt)에서 기계어를 봅니다.

기준선(baseline)을 저장해 두고 변경 뒤에 비교하면 criterion이 통계로 변화를 판정합니다. 아래는 변경 ⑥으로 이 환경에서 한 번 실행한 결과입니다(수치는 실행마다 다르고, `thrpt` 줄 등은 줄였습니다).

```sh
cargo bench -- --save-baseline before   # 바꾸기 전
cargo bench -- --baseline before        # 바꾼 뒤
```

```text
encode/10               time:   [354.04 ns 356.80 ns 359.50 ns]
                 change:
                        time:   [+1017.9% +1030.4% +1043.5%] (p = 0.00 < 0.05)
                        Performance has regressed.
encode/1000             time:   [108.65 µs 108.91 µs 109.38 µs]
                 change:
                        time:   [+5242.6% +5280.0% +5331.8%] (p = 0.00 < 0.05)
                        Performance has regressed.
parse/1000              time:   [12.758 µs 12.924 µs 13.202 µs]
                 change:
                        time:   [−1.4814% +0.0707% +1.6822%] (p = 0.93 > 0.05)
                        No change in performance detected.
```

대괄호의 세 값은 추정값의 신뢰 구간(아래, 가운데, 위)입니다. 3절의 원래 판에서는 레코드가 100배가 될 때 `encode`의 시간이 약 67배(31.8ns → 2.13µs) 늘었고, 변경 ⑥에서는 약 305배(357ns → 109µs) 늘었습니다. 레코드 수보다 빨리 느는 것이 n²의 흔적입니다.

**판정을 측정값으로 읽기.** 같은 비교를 처음 해 봤을 때는 손대지 않은 `parse/1000`이 `+3.7739%`, `Performance has regressed.`로 판정되었습니다. 측정 시간을 줄였고 다른 빌드가 함께 돌던 환경의 흔들림입니다. "통계적으로 유의하다"는 판정이 "의미 있는 변화"라는 뜻은 아닙니다. criterion의 FAQ도 가상 머신 위의 CI에서 잰 시간은 흔들림이 커서 그대로 믿지 말라고 하고, 명령어 수를 세는 도구(Iai)를 대안으로 듭니다. ch20의 "재기 전에 먼저 센다"와 같은 방향입니다.

**지침을 따르지 않아도 되는 경우.** 책은 성능이 중요할 때만 벤치마크를 두라고 합니다. 성능이 요구 사항이 아닌 코드의 벤치마크는 유지할 코드만 늘리고, ch20의 말대로 모든 코드를 끝까지 최적화할 필요는 없습니다. 변경 ⑥처럼 세어 보아 분명한 낭비는 재지 않고 고칩니다.

### 2.7 지침 6: 공격자가 고를 수 있는 입력을 받는 코드는 퍼즈 테스트를 돌린다

**문제 상황.** `parse`는 네트워크에서 온 바이트를 받을 수 있습니다. 안전한 Rust는 메모리 안전성을 보장하지만(ch16) 패닉은 막지 않고, 서버가 요청 하나에 패닉해 멈추면 그 입력은 서비스 거부(DoS) 공격이 됩니다(ch18). 사람이 쓴 테스트는 사람이 떠올린 입력만 시험합니다.

**비권장 코드.** 변경 ⑤는 "태그 뒤에는 길이 바이트가 있다"고 믿고 `cur.byte().expect(…)`로 읽습니다. 잘못된 입력 테스트로 "값이 잘린 경우"는 썼지만 "머리가 반만 있는 경우"는 아무도 떠올리지 못했다고 합시다(그래서 이 사본에서는 회귀 테스트를 지웠습니다). 단위·통합·문서 테스트가 모두 통과합니다.

**권장 코드 ①: 퍼저.** 퍼징(fuzzing)은 무작위에 가까운 입력을 끝없이 만들어 넣으며 패닉이나 잘못된 메모리 접근을 찾는 방법입니다. AFL이나 libFuzzer 같은 퍼저는 입력이 코드의 어느 분기를 지났는지 보고 새 분기에 닿는 입력을 키워 가는 커버리지 유도(coverage-guided) 방식입니다. Rust에서는 libFuzzer를 감싼 `cargo-fuzz`를 씁니다. Rust Fuzz Book에 따르면 nightly 툴체인이 필요하고(sanitizer를 켜는 `-Z` 플래그 때문), x86-64 Linux, x86-64·Apple Silicon macOS, Windows를 지원합니다.

```sh
cargo install cargo-fuzz   # 도구 설치
cargo fuzz init            # fuzz/ 아래에 퍼즈용 크레이트와 fuzz_targets/를 만듦
cargo fuzz list            # 퍼즈 타깃 목록
cargo fuzz run <타깃>       # 패닉을 일으킨 입력은 fuzz/artifacts/<타깃>/crash-… 파일로 남음
```

첫 요건은 책의 말대로 **임의의 바이트를 받는 진입점**입니다. tinytlv의 `parse(&[u8])`가 바로 그것이라 퍼즈 타깃은 몇 줄이면 됩니다.

```rust
// fuzz/fuzz_targets/parse.rs — 이 저장소에서는 컴파일·실행해 확인하지 못한 코드입니다.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = tinytlv::parse(data); // Err는 정상이고, 패닉만 찾습니다
});
```

이 저장소에는 nightly와 `cargo-fuzz`가 설치돼 있지 않아 위 명령과 코드는 실행해 보지 못했습니다. 책은 퍼저가 모은 입력(말뭉치, corpus)을 보관해 다음 실행에 다시 쓰라고 하고, 널리 쓰이는 오픈 소스 크레이트는 OSS-Fuzz가 대신 퍼징해 줄 수 있다고 덧붙입니다.

**권장 코드 ②: stable에서 하는 흉내(책에 없는 지침).** [`tests/arbitrary_bytes.rs`](projects/tinytlv/tests/arbitrary_bytes.rs)는 고정 시드의 xorshift 생성기로 입력을 2만 개씩 만들어 세 성질을 확인합니다. 어떤 바이트열에도 `parse`가 패닉하지 않는다, 성공한 입력은 `encode`하면 같은 바이트가 된다, 아무 레코드 목록이나 `encode` 뒤 `parse`하면 같은 목록이 된다. 패닉은 `catch_unwind`로 받아 그 입력을 메시지에 담습니다. 퍼저가 패닉한 입력을 파일로 남기는 것과 같은 역할입니다. 변경 ⑤에서는 이 테스트만 실패했습니다.

```text
thread 'random_bytes_never_panic_and_round_trip' (…) panicked at src/lib.rs:193:30:
태그 뒤에는 길이 바이트가 있습니다
thread 'random_bytes_never_panic_and_round_trip' (…) panicked at tests/arbitrary_bytes.rs:51:19:
41번째 입력에서 parse가 패닉했습니다: [19]
```

완전히 무작위인 바이트는 길이 바이트가 대개 커서 앞에서 바로 에러가 납니다. 이 시드에서 무작위 입력 2만 개 중 `parse`가 성공한 것은 876개이고 그중 빈 입력이 아닌 것은 95개뿐이었습니다. 그래서 둘째 테스트는 올바른 메시지를 만든 뒤 한 바이트를 바꾸거나 잘라 내거나 끼워 넣어, 2만 개 중 9317개가 끝까지 읽히는 입력을 만듭니다(통계를 내려고 임시로 고친 사본에서 셈). 이것은 퍼징이 아닙니다. 분기를 보지 않고 정해진 개수만 만듭니다. 대신 0.1초 남짓에 끝나고 결과가 늘 같으니 일반 테스트처럼 매번 돌릴 수 있습니다. 이런 일을 체계적으로 해 주는 성질 기반 테스트(property-based testing) 크레이트로 `proptest` 등이 있습니다(이 저장소에서는 쓰지 않음). 테스트 함수가 입력을 인자로 받게 쓰면 컴파일되지 않으므로(4.3절) 그런 크레이트는 매크로로 반복문을 만들어 줍니다.

**지침을 따르지 않아도 되는 경우와 돌리는 때.** 믿을 수 있는 입력만 받는 코드(내가 쓴 설정 파일만 읽는 내부 도구 등)에는 비용에 비해 이득이 작습니다. 퍼즈 테스트는 끝이 없으므로 **정규 CI에서는 돌리지 않습니다**. 책은 릴리스 전, 큰 변경 뒤, 또는 시간을 정해 돌리라고 합니다. 퍼저가 찾은 입력은 고친 뒤 회귀 테스트로 옮깁니다. tinytlv의 `regression_tag_without_length_byte`가 그 모양입니다.

### 2.8 지침 7: 버그를 고칠 때는 재현 테스트부터 쓴다

**문제 상황.** "글이 정확히 20개일 때 목록 아래에 빈 3페이지가 보입니다"라는 신고가 들어왔습니다. 원인은 `items / per_page + 1`입니다([`04_reproduce_bug_first.rs`](examples/04_reproduce_bug_first.rs)).

**비권장 코드.** 코드를 먼저 고치고 테스트를 나중에 씁니다. 그러면 그 테스트가 고치기 전 코드에서 실패했을지 알 수 없습니다. 예제의 "재현하지 못한 테스트"(25개 → 3페이지)는 신고를 잘못 읽고 쓴 테스트라 고치기 전에도 통과합니다. 고친 뒤에만 돌렸다면 버그를 지키는 테스트를 썼다고 착각했을 것입니다.

**권장 코드.** 순서를 지킵니다(3절의 출력).

1. 신고된 입력 그대로 재현 테스트를 쓰고, 고치기 전 코드에서 **실패하는 것을 봅니다**(`글 20개 → 3페이지 (기대: 2)`). 버그를 정확히 재현했다는 확인입니다. 기존 테스트는 모두 나누어떨어지지 않는 경우라 이 버그를 지나쳤습니다.
2. 고칩니다(`items.div_ceil(per_page.get())`).
3. 재현 테스트와 기존 테스트가 모두 통과하는지 봅니다. 고치다가 다른 것을 깨지 않았다는 확인입니다.
4. 재현 테스트를 회귀 테스트로 남기고, 이름과 주석에 신고 내용을 적습니다(`regression_exact_multiple_has_no_empty_page`).

재현 테스트를 쓰다 보면 같은 원인을 가진 입력(10개, 0개)이 보입니다. 글이 0개일 때 0페이지인지 1페이지인지는 이때 정해 테스트에 적습니다. `per_page`는 `NonZeroUsize`로 받으므로 0으로 나누는 경우는 테스트하지 않습니다(2.2절).

**지침을 따르지 않아도 되는 경우.** 재현에 실제 운영 환경(특정 하드웨어, 드문 경쟁 조건)이 필요해 자동 테스트로 만들 수 없는 버그도 있습니다. 그때도 재현 절차를 기록하고, 원인을 좁힌 뒤 그 원인을 테스트로 만들 수 있는지 다시 봅니다.

### 2.9 테스트를 돌리는 범위: CI, 기능 조합, 플랫폼, 게시

- **퍼즈 테스트를 뺀 모든 테스트를 변경마다 CI에서 돌립니다.** 명령 하나로는 "모든 테스트"가 되지 않습니다. Cargo Book에 따르면 `--all-targets`는 `--lib --bins --tests --benches --examples`와 같고 문서 테스트를 포함하지 않으며, `--doc`은 다른 타깃 옵션과 함께 쓸 수 없습니다. 실제로 tinytlv에서 `cargo test --all-targets`의 출력에는 `Doc-tests` 구역이 없었습니다. 벤치 타깃을 테스트 모드로 돌리면 criterion은 재지 않고 한 번씩만 실행해(`Testing encode/10` 다음 줄에 `Success`) 벤치마크 코드가 깨지지 않았는지 확인해 줍니다. `cargo test`는 처음 실패한 테스트 바이너리에서 멈추므로 실패를 한 번에 모두 보려면 `--no-fail-fast`를 줍니다.

  ```sh
  cargo test --all-targets --no-fail-fast   # 단위, 통합, 벤치(한 번씩), 예제 안의 테스트
  cargo test --doc                          # 문서 테스트
  for ex in sensor_report inspect; do cargo run -q --example "$ex" || exit 1; done
  ```

- **기능이 있으면 모든 기능 조합을 테스트합니다.** ch26 2.6절의 `--no-default-features`·`--all-features` 반복문이 그것입니다.
- **플랫폼별 `cfg` 코드가 있으면 코드가 달라지는 플랫폼마다 테스트합니다.** `#[cfg]`로 지워진 쪽은 타입 검사조차 받지 않습니다(ch26 2.2절).
- **crates.io에 게시한다면 패키지에 무엇이 들어가는지 봅니다.** 기본으로는 단위·통합 테스트, 벤치마크, 예제 프로그램이 모두 들어갑니다. `cargo package --list`로 확인하고(커밋하지 않은 파일이 있으면 `--allow-dirty`가 필요), 책은 필요 없는 테스트 파일을 `exclude`로 빼거나 동작 테스트를 별도 크레이트로 옮기라고 합니다. 사본에서 `exclude = ["/benches", "/tests"]`를 주자 두 디렉터리가 목록에서 빠졌고, `cargo package`가 ``warning: ignoring test `public_api` as `tests/public_api.rs` is not included in the published package``처럼 경고하며 게시용 매니페스트에서 그 타깃을 지웠습니다. 자기 `Cargo.toml`을 가진 하위 디렉터리(`cargo fuzz init`이 만드는 `fuzz/` 같은)는 설정과 상관없이 빠집니다(Cargo Book, 사본에 빈 하위 패키지를 두고 확인).

### 2.10 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0, cargo 1.100.0, criterion 0.8.2에서 확인한 차이는 다음과 같습니다.

- **내장 벤치 하네스는 여전히 nightly 전용입니다.** 책이 "쓰는 시점에는 불안정"이라고 한 `#[bench]`는 1.100.0에서도 `#![feature(test)]`가 E0554, 그 줄이 없으면 쓰는 곳마다 E0658입니다(4.1절). Cargo Book도 같게 적습니다.
- **criterion의 `black_box`는 deprecated입니다.** criterion 0.8.2의 `criterion::black_box`에는 `use std::hint::black_box() instead`라는 deprecated 표시가 있습니다(`std::hint::black_box`는 1.66 안정화).
- **`cargo test --examples`는 예제의 `main`을 실행하지 않습니다.** 책의 정리는 예제를 `cargo test --examples`나 `cargo run --example`로 돌린다고 나란히 적지만, 앞의 것은 예제 안의 `#[test]`만 실행합니다(2.5절의 표, 변경 ④).
- **`cargo test --all-targets`는 문서 테스트를 빼먹습니다**(2.9절). "모든 테스트를 CI에서"를 지키려면 `--doc`을 따로 돌립니다.
- **벤치마크와 CI.** 책은 성능이 중요하면 벤치마크를 정기적으로(이상적으로는 CI에서) 돌리라고 합니다. criterion의 FAQ는 가상 머신 위 CI의 측정값을 그대로 믿지 말라고 경고하고, 이 챕터에서도 손대지 않은 코드가 `regressed`로 판정된 일이 있었습니다(2.6절).
- **문서 테스트 병합.** 책은 문서 주석의 코드 샘플마다 `fn main`으로 감싸 실행한다고 설명합니다. 2024 에디션에서는 여러 문서 테스트를 한 바이너리로 묶어 컴파일하고 실행만 따로 합니다(Part 2 ch5 2.8절, tinytlv의 `merged doctests compilation took …` 줄).
- **`exclude`로 뺀 타깃.** cargo 1.100.0은 파일이 빠진 테스트·벤치 타깃을 경고와 함께 게시용 매니페스트에서 지웁니다(2.9절). 책은 `exclude`만 언급합니다.
- **책에 없는 지침과 내용.** 다음은 책의 Item 30이 명시적으로 권하지 않는 것입니다(2026-11-20에 온라인판에서 확인).
  - 테스트가 버그를 잡는지 버그를 넣어 확인하기(예제 01, 2.1절의 변경 실험): 예제와 사본의 실행 결과가 근거입니다.
  - 고정 시드 생성기로 퍼징을 흉내 내 일반 테스트로 돌리기(`tests/arbitrary_bytes.rs`): 이 챕터에서 더한 방법이고, 사본의 실행 결과(변경 ⑤)가 근거입니다.
  - `#[cfg_attr(test, …)]`로 공개 트레이트 구현을 가리지 않기(변경 ②): Cargo Book Cargo Targets의 `cfg(test)` 설명과 사본의 컴파일 결과가 근거입니다.
  - `[lib] bench = false`, `--save-baseline`/`--baseline` 기준선 비교, 판정을 측정값으로 읽기: criterion 문서(FAQ, Command-Line Options)와 실행 결과가 근거입니다.
  - `cargo test --all-targets`와 `--doc`을 함께 돌리기, `--no-fail-fast`, 벤치를 테스트 모드로 한 번씩 돌리기, `cargo test --examples`와 `cargo run --example`의 차이: Cargo Book(cargo test, Cargo Targets)과 실행 결과가 근거입니다.
  - 회귀 테스트의 이름·주석에 신고 내용을 적고, 같은 원인을 가진 입력까지 재현 테스트에 넣기(예제 04).

### 2.11 기억할 것

- 단위 테스트는 내부까지, 통합 테스트는 공개 API로 동작을, 문서 테스트는 공개 항목 하나하나의 사용법을, 예제 프로그램은 API 전체의 흔한 사용법을 봅니다. 서로 대신하지 못합니다.
- 타입이 이미 보장하는 것은 테스트하지 않습니다. 테스트가 무언가를 지키는지 의심되면 버그를 넣어 실패하는지 봅니다.
- 예제 프로그램은 `?`로 쓰고 CI에서 실행까지 합니다. `cargo test`는 예제를 빌드만 하고, `--all-targets`는 문서 테스트를 뺍니다.
- 성능이 요구 사항이면 criterion으로 재고 기준선과 비교하되, 판정은 흔들림이 있는 측정값으로 읽습니다.
- 임의의 바이트를 받는 진입점은 퍼즈 테스트 대상입니다. 퍼징은 CI 밖에서 시간을 정해 돌리고, 찾은 입력은 회귀 테스트로 옮깁니다.
- 버그는 재현 테스트가 실패하는 것을 먼저 보고 고칩니다.

## 3. 예제 안내

`examples/`는 저장소 루트에서 실행합니다. 일부러 실패시키는 03의 두 실행을 빼면 모두 종료 코드 0으로 끝납니다. 예제 안의 `#[cfg(test)]` 테스트 7개(01에 2개, 02에 2개, 03에 1개, 04에 2개)는 `cargo test -p er-ch30-beyond-unit-tests --examples`로 실행합니다. `projects/tinytlv`는 그 디렉터리에서 실행합니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_what_types_guarantee.rs`](examples/01_what_types_guarantee.rs) | 1 | 타입이 보장하는 것을 확인하는 묶음 | 경계와 범위 밖을 확인하는 묶음 | 2.2 |
| [`02_early_warning_dependency.rs`](examples/02_early_warning_dependency.rs) | 2 | 기능 테스트만 | 의존성 동작 하나를 고정하는 테스트 | 2.3 |
| [`03_example_program_style.rs`](examples/03_example_program_style.rs) | 4 | `unwrap()` 판 | `main() -> Result`와 `?` 판 | 2.5 |
| [`04_reproduce_bug_first.rs`](examples/04_reproduce_bug_first.rs) | 7 | 재현하지 못한 테스트 | 실패를 먼저 보는 재현 테스트 | 2.8 |
| [`projects/tinytlv`](projects/tinytlv) | 1~7 | (2.1절의 변경 ①~⑥) | 여섯 종류를 갖춘 라이브러리 | 2.1~2.9 |

### 01_what_types_guarantee — 타입이 보장하는 것과 보장하지 않는 것

```sh
cargo run -p er-ch30-beyond-unit-tests --example 01_what_types_guarantee
```

```text
비권장 묶음(타입이 보장하는 것)
  정상 판: 통과
  버그 ① 100점 초과를 받음: 통과
  버그 ② 60점 경계가 밀림: 통과
권장 묶음(경계와 범위 밖)
  정상 판: 통과
  버그 ① 100점 초과를 받음: 실패: 101점 → Ok(A) (기대: Err(OutOfRange(101)))
  버그 ② 60점 경계가 밀림: 실패: 60점 → Ok(F) (기대: Ok(C))
```

버그 ①은 `u8`이 담을 수 있는 101~255점을 A로 받습니다. 타입이 막아 주지 않는 입력이라 실제로 일어나는 버그입니다.

### 02_early_warning_dependency — 의존성의 동작 변화를 알리는 테스트

```sh
cargo run -p er-ch30-beyond-unit-tests --example 02_early_warning_dependency
```

```text
[fieldsplit 1.4.0]
  기능 테스트: 통과
  조기 경보 테스트: 통과
  이메일이 빈 줄: email=None, phone=Some("010-9876-5432")
[fieldsplit 1.5.0]
  기능 테스트: 통과
  조기 경보 테스트: 실패 — fieldsplit::split가 빈 칸을 남긴다는 가정이 깨졌습니다: ["a", "b"]
  이메일이 빈 줄: email=Some("010-9876-5432"), phone=None
```

두 버전은 같은 파일 안의 모듈 `fieldsplit_1_4`, `fieldsplit_1_5`로 흉내 냈습니다.

### 03_example_program_style — `?` 판과 `unwrap()` 판

```sh
cargo run -p er-ch30-beyond-unit-tests --example 03_example_program_style
```

```text
1h30m = 5400초
45m = 2700초
90s = 90초
합계 8190초
```

인자로 잘못된 시간을 주면 두 판의 차이가 보입니다. 둘 다 일부러 실패시키는 실행입니다.

```sh
cargo run -p er-ch30-beyond-unit-tests --example 03_example_program_style -- 1h30m 2x            # ? 판, 종료 코드 1
cargo run -p er-ch30-beyond-unit-tests --example 03_example_program_style -- --unwrap 1h30m 2x   # unwrap 판, 종료 코드 101
```

```text
1h30m = 5400초
Error: UnknownUnit { unit: 'x' }
```

```text
1h30m = 5400초

thread 'main' (…) panicked at appd-a/ch30-beyond-unit-tests/examples/03_example_program_style.rs:87:49:
called `Result::unwrap()` on an `Err` value: UnknownUnit { unit: 'x' }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

`Error: …`와 패닉 메시지는 표준 에러로 나갑니다. 패닉 메시지의 스레드 번호는 실행마다 다릅니다. `?` 판이 출력하는 것은 에러의 디버그 형식입니다(Part 0 ch9 2.10절).

### 04_reproduce_bug_first — 재현 테스트를 먼저 쓰는 순서

```sh
cargo run -p er-ch30-beyond-unit-tests --example 04_reproduce_bug_first
```

```text
1. 고치기 전 + 기존 테스트: 통과
2. 고치기 전 + 재현 테스트: 실패: 글 20개 → 3페이지 (기대: 2)
3. 고친 뒤   + 재현 테스트: 통과
4. 고친 뒤   + 기존 테스트: 통과
(비교) 재현하지 못한 테스트
   고치기 전: 통과
   고친 뒤:   통과
```

### projects/tinytlv — 여섯 종류의 테스트를 갖춘 라이브러리

```text
projects/tinytlv/
├── Cargo.toml                 [lib] bench = false, criterion, [[bench]] harness = false, 빈 [workspace]
├── src/lib.rs                 공개 API와 문서 테스트 5개
├── src/cursor.rs              비공개 Cursor와 단위 테스트 4개
├── src/tests.rs               lib.rs의 단위 테스트 5개(should_panic, 회귀 테스트 포함)
├── tests/public_api.rs        통합 테스트 4개
├── tests/arbitrary_bytes.rs   임의 바이트 테스트 3개(퍼징의 흉내)
├── examples/sensor_report.rs  예제 프로그램: 레코드를 기록하고 다시 읽기
├── examples/inspect.rs        예제 프로그램: 16진수 인자를 레코드로 풀어 보기
└── benches/codec.rs           criterion 벤치마크 4개
```

외부 크레이트는 dev-dependency `criterion` 0.8.2 하나입니다(2026-11-20에 `cargo add`로 받음). 처음 빌드할 때 crates.io에서 받습니다.

```sh
cd appd-a/ch30-beyond-unit-tests/projects/tinytlv
cargo test
```

테스트 바이너리마다 구역이 생기고, 결과 줄은 다음과 같습니다(끝나는 순서와 시간은 실행마다 다름).

```text
     Running unittests src/lib.rs (…)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; …
     Running tests/arbitrary_bytes.rs (…)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; …
     Running tests/public_api.rs (…)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; …
   Doc-tests tinytlv
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; …
```

```sh
cargo run --example sensor_report
```

```text
보낼 바이트 53개
장치: 3층 서버실 북쪽 벽 온습도 센서
온도 21.5도
습도 48%
문 닫힘
```

```sh
cargo run --example inspect -- 01 02 00 d7 03 00
```

```text
레코드 2개
  #0 태그   1  길이   2  값 [00, d7]
  #1 태그   3  길이   0  값 []
```

인자가 없으면 안에 든 예시 9바이트(레코드 3개)를 씁니다. `-- 01 05 aa`는 `Error: ValueOverrun { offset: 0, len: 5, remaining: 1 }`, `-- 01 zz`는 `Error: ParseIntError { kind: InvalidDigit }`를 출력하고 종료 코드 1로 끝납니다.

```sh
cargo bench               # 이 환경에서 7초 안팎
cargo test --benches      # 재지 않고 벤치마크마다 한 번씩 실행(Testing … / Success)
```

`cargo bench`를 이 환경에서 한 번 실행한 결과의 `time` 줄입니다. 수치는 실행마다 다르고, 다른 작업이 함께 돌면 크게 흔들립니다. `thrpt`(처리량) 줄과 이상값(outlier) 줄은 줄였습니다. 처음에는 `Gnuplot not found, using plotters backend` 줄이 나옵니다.

```text
encode/10               time:   [31.395 ns 31.847 ns 32.617 ns]
encode/1000             time:   [2.0440 µs 2.1327 µs 2.2237 µs]
parse/10                time:   [201.74 ns 212.06 ns 221.42 ns]
parse/1000              time:   [12.659 µs 13.025 µs 13.408 µs]
```

`parse`가 `encode`보다 느린 까닭을 이 숫자만으로 단정하지 않습니다. 굳이 짐작한다면 `parse`는 레코드마다 `Vec`을 하나씩 할당하고, `encode`는 전체 크기를 먼저 세어 한 번만 할당합니다. 이런 짐작은 ch20 2.3절처럼 할당 횟수를 세어 확인합니다.

`cargo package --list --allow-dirty`(커밋한 뒤라면 `--allow-dirty` 없이)는 2.9절에서 본 대로 `src/`, `tests/`, `benches/`, `examples/`의 파일과 `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `.cargo_vcs_info.json`을 나열합니다. 이 프로젝트에는 `license` 등이 없어 `manifest has no license, …` 경고가 함께 나옵니다. 게시(`cargo publish`)는 하지 않습니다(`publish = false`).

## 4. 자주 하는 실수

`compile_fail/`의 세 파일은 모두 `rustc --test`로 확인합니다. `--test`가 없으면 `#[cfg(test)]` 코드가 빠져 4.2절의 파일은 컴파일되어 버립니다.

### 4.1 stable에서 내장 벤치 하네스를 씀 — E0554

[`compile_fail/e0554_builtin_bench_on_stable.rs`](compile_fail/e0554_builtin_bench_on_stable.rs)는 `#![feature(test)]`, `extern crate test;`, `#[bench] fn …(b: &mut Bencher)`로 쓴 벤치마크입니다.

```sh
rustc --edition 2024 --test --out-dir <임시 디렉터리> compile_fail/e0554_builtin_bench_on_stable.rs
```

```text
error[E0554]: `#![feature]` may not be used on the stable release channel
  --> compile_fail/e0554_builtin_bench_on_stable.rs:14:1
   |
14 | #![feature(test)]
   | ^^^^^^^^^^^^^^^^^
```

**원인.** `#[bench]`와 `test::Bencher`는 불안정 기능 `test`에 속하고, `#![feature]`는 nightly에서만 받아들여집니다. 첫 줄을 지우면 `extern crate test`, `use test::Bencher`, `#[bench]`, `Bencher`, `b.iter`, `test::black_box` 여섯 곳에서 E0658(``use of unstable library feature `test` ``)이 납니다.

**해결.** `criterion`을 dev-dependency로 두고 `benches/`의 타깃에 `harness = false`를 줍니다(2.6절). `black_box`는 `std::hint::black_box`를 씁니다.

### 4.2 타입이 막는 입력을 테스트하려 함 — E0600

[`compile_fail/e0600_negative_score_test.rs`](compile_fail/e0600_negative_score_test.rs)는 `grade(score: u8)`에 "음수 점수를 거부하는가"를 테스트합니다.

```sh
rustc --edition 2024 --test --out-dir <임시 디렉터리> compile_fail/e0600_negative_score_test.rs
```

```text
error[E0600]: cannot apply unary operator `-` to type `u8`
  --> compile_fail/e0600_negative_score_test.rs:30:23
   |
30 |         assert!(grade(-1).is_err());
   |                       ^^ cannot apply unary operator `-`
   |
   = note: unsigned values cannot be negated
help: you may have meant the maximum value of `u8`
   |
30 -         assert!(grade(-1).is_err());
30 +         assert!(grade(u8::MAX).is_err());
   |
```

**원인.** `u8`에는 음수가 없으니 이 테스트가 확인하려는 상황은 생길 수 없습니다. 컴파일러가 테스트 자체를 거부한 것이 곧 "타입이 보장한다"는 뜻입니다.

**해결.** 테스트를 지우고, 타입이 막아 주지 않는 입력(101~255점)을 테스트합니다. help가 제안한 `u8::MAX`(255)가 마침 그런 입력이라, 이 경우에는 help를 따라도 뜻 있는 테스트가 됩니다.

### 4.3 테스트 함수가 인자를 받음 — 에러 코드 없음

[`compile_fail/test_fn_with_arguments.rs`](compile_fail/test_fn_with_arguments.rs)는 여러 입력으로 같은 성질을 확인하고 싶어 `#[test] fn never_panics(input: u8)`로 썼습니다.

```sh
rustc --edition 2024 --test --out-dir <임시 디렉터리> compile_fail/test_fn_with_arguments.rs
```

```text
error: functions used as tests can not have any arguments
  --> compile_fail/test_fn_with_arguments.rs:16:1
   |
16 | / fn never_panics(input: u8) {
17 | |     let _ = parse_digit(input);
18 | | }
   | |_^
```

**원인.** 테스트 러너는 테스트 함수를 인자 없이 부릅니다.

**해결.** 입력은 테스트 함수 안에서 만들어 반복문으로 넣습니다(`tests/arbitrary_bytes.rs`). 성질 기반 테스트 크레이트는 매크로로 이 반복문을 만들어 줍니다.

### 4.4 그 밖의 실수

| 실수 | 증상 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| `cargo bench -- --save-baseline x`인데 `[lib] bench = false`가 없음 | 라이브러리의 단위 테스트 하네스가 먼저 실행되어 `error: Unrecognized option: 'save-baseline'` | `[lib] bench = false`, 또는 `cargo bench --bench codec -- …` | 2.6 |
| 공개 트레이트 구현을 `#[cfg_attr(test, derive(…))]`로 가림 | 단위 테스트는 통과, 통합·문서 테스트는 E0369(`… is defined in another crate`) | 공개 API로 약속하고 그냥 derive | 2.4, 변경 ② |
| 통합 테스트에서 라이브러리의 `#[cfg(test)]` 도우미를 씀 | E0425 ``cannot find function `…` in crate `tinytlv` ``, note `found an item that was configured out` | 도우미를 `tests/common/mod.rs`로(Part 0 ch11 2.8절) | 2.4 |
| `cargo test --all-targets`만 돌림 | `Doc-tests` 구역이 없음 | `cargo test --doc`을 따로 | 2.9 |
| `cargo test --examples`로 예제가 동작한다고 믿음 | `main`이 실행되지 않아 실행 중 실패를 놓침 | `cargo run --example`으로 하나씩 | 2.5, 변경 ④ |
| 실패를 모두 보려는데 `--no-fail-fast`가 없음 | 처음 실패한 테스트 바이너리에서 멈춤 | `cargo test --no-fail-fast` | 2.9 |
| 변경 실험용 사본 여러 개가 `CARGO_TARGET_DIR` 하나를 함께 씀 | 이 챕터의 실험에서 다른 사본의 낡은 테스트 바이너리가 그대로 실행됨(원인은 확인하지 못함) | 사본마다 자기 `target/`을 씀 | 2.1 |
| 커밋하지 않은 파일이 있는데 `cargo package --list` | `… contain changes that were not yet committed into git` | `--allow-dirty` | 2.9 |

## 5. 연습 문제

### 연습 1. 변경 하나 더: 공개 함수를 크레이트 안으로 숨기기

tinytlv의 사본에서 `pub fn parse`를 `pub(crate) fn parse`로 바꾸면 2.1절의 표에서 어느 칸이 ✗가 될지 먼저 예측하세요. 그다음 `cargo test --lib`, `cargo test`, `cargo test --doc`을 차례로 실행해 확인하세요.

<details>
<summary>힌트</summary>

`cargo test --lib`은 9개가 모두 통과합니다. 단위 테스트는 같은 크레이트 안이라 `pub(crate)`도 보입니다. 다만 라이브러리를 빌드할 때 `function `parse` is never used`, `struct `Cursor` is never constructed` 같은 `dead_code` 경고가 나옵니다. 크레이트 안에서 `parse`를 부르는 곳이 테스트뿐이기 때문입니다. `cargo test`는 두 통합 테스트와 두 예제 프로그램에서 E0603(``function `parse` is private``)으로 컴파일에 실패합니다(예제 프로그램도 `cargo test`가 빌드하므로). `cargo test --doc`은 `parse`를 쓰는 문서 테스트 3개가 같은 E0603으로 실패합니다.

</details>

### 연습 2. 임의 바이트 테스트가 찾은 입력을 회귀 테스트로 옮기기

tinytlv의 사본에 변경 ⑤를 넣으세요. `src/lib.rs`의 `parse`에서 `cur.byte().ok_or(ParseError::MissingLength { offset })?`를 `cur.byte().expect("태그 뒤에는 길이 바이트가 있습니다")`로 바꾸고, `src/tests.rs`의 `regression_tag_without_length_byte`를 지웁니다. `cargo test --test arbitrary_bytes`가 알려 준 입력으로 2.8절의 순서(실패를 먼저 보고, 고치고, 남기기)를 따라 회귀 테스트를 다시 만드세요.

<details>
<summary>힌트</summary>

`41번째 입력에서 parse가 패닉했습니다: [19]`와 `3번째 입력에서 parse가 패닉했습니다: [130, 2, 94, 180, 97, 6, 183, 21, 240, 126, 194, 181, 148]`가 나옵니다. 둘째 입력도 앞의 레코드 둘(2바이트, 6바이트 값)을 읽고 나면 `148` 한 바이트만 남는 같은 경우입니다. 가장 짧은 `[19]`로 `assert_eq!(parse(&[19]), Err(ParseError::MissingLength { offset: 0 }))`를 쓰면 고치기 전에는 패닉으로 실패하고, `ok_or(…)?`로 되돌리면 통과합니다. 퍼저도 찾은 입력을 줄이는 기능을 갖고 있지만, 회귀 테스트에는 사람이 읽을 수 있는 가장 짧은 입력을 남기는 것이 좋습니다.

</details>

### 연습 3. 조기 경보 테스트 하나 더

예제 02에 `fieldsplit` 1.6.0을 더하세요. 이 버전은 빈 칸을 남기지만 앞뒤 공백을 지우지 않습니다(`line.split(',').collect()`). 기능 테스트와 지금의 조기 경보 테스트는 어떻게 되나요? 내 코드가 기대는 동작이 하나 더 있다는 것을 알리는 조기 경보 테스트를 쓰세요.

<details>
<summary>힌트</summary>

이번에는 기능 테스트가 `Contact { name: "김하늘", email: Some(" sky@example.com"), phone: Some(" 010-1234-5678") }`로 실패하고, `split("a,,b")`를 보는 조기 경보 테스트는 공백이 없는 입력이라 통과합니다. 이메일이 빈 줄은 `email=Some(" ")`가 됩니다. 기능 테스트의 실패 메시지만으로는 원인이 내 코드인지 의존성인지 알기 어렵습니다. `assert_eq!(split(" a , b "), ["a", "b"])` 같은 테스트를 더하면 "앞뒤 공백을 지운다"는 둘째 가정이 깨졌다고 바로 알려 줍니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 30: Write more than unit tests](https://effective-rust.com/testing.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- The Cargo Book: [`cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html)(기본으로 빌드·실행하는 타깃, `--all-targets`와 `--doc`, `--no-fail-fast`), [`cargo bench`](https://doc.rust-lang.org/cargo/commands/cargo-bench.html), [Cargo Targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html)(예제의 `test` 기본값, `harness = false`, `#[bench]`가 nightly 전용이라는 것), [The `exclude` and `include` fields](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields)(늘 빠지는 하위 패키지, `cargo package --list`)
- Criterion.rs 문서: [FAQ](https://bheisler.github.io/criterion.rs/book/faq.html)(`Unrecognized option`과 `bench = false`, `black_box`, CI에서의 측정), [Command-Line Options](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html)(`--save-baseline`, `--baseline`, 이름으로 거르기)
- Rust Fuzz Book: [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html), [Setup](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html)(nightly, 지원 플랫폼, 설치), [Tutorial](https://rust-fuzz.github.io/book/cargo-fuzz/tutorial.html)(`cargo fuzz init`·`list`·`run`, `fuzz/artifacts/`)
- [`std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html): 벤치마크용이고 "최선의 노력"일 뿐이라는 설명
