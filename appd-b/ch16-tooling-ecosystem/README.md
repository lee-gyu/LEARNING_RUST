# ch16. 도구와 생태계

새 언어를 고를 때는 언어 자체만큼 그 주변을 봅니다. 패키지는 어떻게 받고, 표준 라이브러리로 어디까지 되며, 언어는 얼마나 자주 바뀌고, 누가 만들고, 어디서 돌아가는가입니다. 이 챕터는 그 질문들에 답하며, Cargo를 npm·pip·Maven·Go 모듈과 나란히 놓고 같은 점과 다른 점을 짚습니다. Cargo의 사용법은 Part 3에, 의존성 그래프를 다루는 판단은 부록 A ch21·ch25·ch31에 있으므로 이 챕터는 그것을 가리키고 "왜 그렇게 정했나"에 집중합니다.

- 패키지: `faq-ch16-tooling-ecosystem`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 자료 확인일: 2026-11-21
- `cargo-audit`, `cargo-deny`, `cargo-vet`은 설치하지 않아 이름과 용도만 적었습니다.

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q16.1](#q161-cargo는-npm-pip-mavengradle-go-modules와-무엇이-같고-다른가요) | Cargo는 npm, pip, Maven/Gradle, Go modules와 무엇이 같고 다른가요? | 선언 파일과 잠금 파일, 중앙 저장소를 쓰는 점은 npm과 비슷합니다. 호환되는 범위에서 가장 높은 버전을 고르고 잠금 파일로 고정하는 방식은 Go의 최소 버전 선택(minimal version selection, MVS)과 다르고, 빌드·테스트·문서·게시까지 한 도구가 맡습니다. |
| [Q16.2](#q162-표준-라이브러리가-왜-작은가요-난수-정규식-http-json이-없는-이유) | 표준 라이브러리가 왜 작은가요? (난수, 정규식, HTTP, JSON이 없는 이유) | 표준 라이브러리에 들어간 API는 영원히 유지해야 하므로, 설계가 빨리 바뀌거나 선택지가 여럿인 기능은 크레이트로 둡니다. 난수 API(`std::random`)는 이 툴체인에서 아직 nightly 전용입니다. |
| [Q16.3](#q163-크레이트는-어떻게-고르고-믿나요-공급망-위험은-어떻게-줄이나요) | 크레이트는 어떻게 고르고 믿나요? 공급망 위험은 어떻게 줄이나요? | 관리 상태, 의존성 수, 라이선스, `unsafe`와 빌드 스크립트를 보고 고르고, `Cargo.lock`과 체크섬으로 고정하고, 취약점 데이터베이스를 확인하는 도구로 점검합니다. 언어가 막아 주는 영역은 아닙니다. |
| [Q16.4](#q164-cratesio에는-왜-네임스페이스가-없나요) | crates.io에는 왜 네임스페이스가 없나요? | 처음부터 하나의 평평한 이름 공간으로 시작했기 때문입니다. 짧은 이름을 먼저 차지하는 문제와 비슷한 이름으로 속이는 문제가 생깁니다. 크레이트 주인만 `foo::bar`처럼 자기 크레이트 이름 아래에 게시하게 하는 RFC 3243이 채택됐지만 아직 안정화되지 않았습니다(Cargo에는 nightly 전용 기능으로 있음). |
| [Q16.5](#q165-언어가-자주-바뀌지-않나요-6주-릴리스-안정성-보장-에디션) | 언어가 자주 바뀌지 않나요? (6주 릴리스, 안정성 보장, 에디션) | 6주마다 새 버전이 나오지만 안정판에서 컴파일되던 코드는 계속 컴파일되도록 지킵니다. 호환되지 않는 변경은 프로젝트가 고르는 에디션으로 묶습니다. |
| [Q16.6](#q166-nightly는-무엇이고-써야-하나요) | nightly는 무엇이고 써야 하나요? | 매일 나오는 개발판으로, 아직 안정화되지 않은 기능을 쓸 수 있습니다. 그 기능이 꼭 필요할 때만 쓰고, 보통은 안정판을 씁니다. |
| [Q16.7](#q167-언어-명세나-iso-표준이-있나요) | 언어 명세나 ISO 표준이 있나요? | ISO 표준은 없습니다. The Rust Reference가 주 참고서이고, Rust 프로젝트는 2025년에 Ferrocene Language Specification(FLS)을 명세 작업의 일부로 받아들였습니다. |
| [Q16.8](#q168-컴파일러는-rustc-하나뿐인가요) | 컴파일러는 rustc 하나뿐인가요? | 실제로 쓰는 컴파일러는 rustc 하나입니다. GCC 기반의 gccrs가 개발 중이지만 아직 초기 단계이고 대여 검사기가 없습니다. |
| [Q16.9](#q169-안정-abi가-없다는-것은-무슨-뜻인가요-동적-라이브러리나-플러그인은-어떻게-하나요) | 안정 ABI가 없다는 것은 무슨 뜻인가요? 동적 라이브러리나 플러그인은 어떻게 하나요? | Rust 타입의 메모리 배치와 함수 호출 방식은 컴파일러가 바꿀 수 있어, 다른 버전으로 컴파일한 Rust 코드끼리 바이너리로 섞을 수 없습니다. 경계는 C ABI(`extern "C"`, `#[repr(C)]`, `cdylib`)로 만듭니다. |
| [Q16.10](#q1610-누가-만들고-관리하나요-라이선스는-무엇인가요) | 누가 만들고 관리하나요? 라이선스는 무엇인가요? | 여러 팀으로 이루어진 오픈소스 프로젝트이고, 팀 대표로 이루어진 Leadership Council이 프로젝트 전체를 맡습니다. 언어와 공식 프로젝트는 대개 Apache-2.0과 MIT의 이중 라이선스입니다. |
| [Q16.11](#q1611-편집기-포매터-린터는-무엇을-쓰나요) | 편집기, 포매터, 린터는 무엇을 쓰나요? | 편집기는 rust-analyzer(언어 서버, language server)를 붙여 쓰고, 포매터는 `rustfmt`, 린터는 Clippy가 공식 도구입니다. |
| [Q16.12](#q1612-어떤-플랫폼을-지원하나요-웹webassembly-모바일-임베디드는요) | 어떤 플랫폼을 지원하나요? 웹(WebAssembly), 모바일, 임베디드는요? | 주요 데스크톱·서버 플랫폼은 Tier 1이고, WebAssembly·Android·iOS와 주요 임베디드 타깃은 Tier 2(빌드 보장)입니다. 운영체제 없는 타깃 가운데는 Tier 3도 많습니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 3](../../part3-cargo/README.md): Cargo의 의존성, 매니페스트, 워크스페이스, 기능, 프로필, 게시
- [Part 2](../../part2-editions/README.md): 에디션
- [부록 A ch21](../../appd-a/ch21-semver/README.md), [부록 A ch25](../../appd-a/ch25-dependency-graph/README.md), [부록 A ch31](../../appd-a/ch31-tooling-ecosystem/README.md): SemVer, 의존성 그래프, 도구 생태계

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Cargo를 npm·pip·Maven·Go 모듈과 비교해, 버전을 고르는 규칙과 잠금 파일의 역할이 어떻게 다른지 설명할 수 있다.
2. 표준 라이브러리가 작은 이유와 크레이트를 고르고 공급망(supply chain) 위험을 줄이는 방법을 말할 수 있다.
3. 6주 릴리스, 안정성 보장, 에디션, nightly의 관계를 설명할 수 있다.
4. 명세와 컴파일러 구현, ABI, 관리 주체와 라이선스, 지원 플랫폼의 현황을 말할 수 있다.

## 2. 핵심 개념

### Q16.1 Cargo는 npm, pip, Maven/Gradle, Go modules와 무엇이 같고 다른가요?

**짧은 답** 의존성을 선언하는 파일(`Cargo.toml`), 고른 버전을 고정하는 잠금 파일(`Cargo.lock`), 중앙 저장소(crates.io)를 쓴다는 점은 npm과 비슷합니다. 버전 요구 사항은 기본이 캐럿(SemVer로 호환되는 범위)이고, 그 범위에서 가장 높은 버전을 고른 뒤 잠금 파일로 고정합니다. Go 모듈은 요구된 최소 버전들 가운데 가장 높은 것을 고르는 최소 버전 선택(MVS)을 써서, 잠금 파일이 없어도 새 버전이 나왔다고 고르는 버전이 바뀌지 않는다는 점이 다릅니다. 그리고 Cargo는 빌드, 테스트, 문서, 벤치마크, 게시까지 한 도구가 맡습니다.

**왜 그런가** [01_version_selection](examples/01_version_selection.rs)은 두 규칙을 표준 라이브러리만으로 흉내 내어 같은 요구 사항에서 무엇을 고르는지 비교합니다.

```text
1. 레지스트리: 1.2.0, 1.4.0, 1.6.0, 2.0.0
  요구 사항: 1.2.0, 1.4.0
  Cargo(잠금 파일 없음): 1.6.0
  Go MVS: 1.4.0
2. 1.7.0이 새로 올라온 뒤
  요구 사항: 1.2.0, 1.4.0
  Cargo(잠금 파일 없음): 1.7.0
  Go MVS: 1.4.0
  Cargo.lock이 있으면: cargo update 전까지 1.6.0을 그대로 씀
3. 0.x 버전의 캐럿 규칙
  ^0.4.1 요구 사항에 0.4.1: true
  ^0.4.1 요구 사항에 0.4.9: true
  ^0.4.1 요구 사항에 0.5.0: false
```

Cargo Book은 캐럿 요구 사항이 기본이고 `log = "1.2.3"`이 `"^1.2.3"`과 같다고 적습니다. 요구 사항 문법마다 받아들이는 버전은 [Part 3 ch2](../../part3-cargo/ch2-dependencies/README.md) 2.2절과 예제 `01_version_requirements`(`semver` 크레이트로 판정)에서, 리졸버가 범위 안의 가장 높은 버전을 고르는 모습은 같은 챕터 2.6절에서 봤습니다. 같은 상황에서 실제 Cargo가 1.6.0 하나를 고르고, 호환되지 않는 메이저 버전(1.x와 2.x)은 사본 둘을 함께 컴파일하는 것은 [부록 A ch25](../../appd-a/ch25-dependency-graph/README.md) 2.3절에서 실행으로 확인했습니다. Go Modules Reference는 MVS가 모듈마다 요구된 버전 가운데 가장 높은 것을 고르며, 그것이 모든 요구를 만족하는 최소 버전이라고 설명합니다. 같은 문서에 따르면 `go.sum`은 버전을 고정하는 파일이 아니라 내려받은 모듈의 암호 해시를 적는 파일입니다.

두 규칙은 무엇을 기본으로 삼는지가 다릅니다. Cargo는 호환 범위의 최신을 골라 버그 수정과 새 기능을 기본으로 받고, 빌드를 재현하는 일은 `Cargo.lock`에 맡깁니다(부록 A ch21). Go Modules Reference는 MVS가 결정적이고 새 버전이 나와도 빌드 목록이 바뀌지 않으므로, 빌드 목록을 잠금 파일에 저장하지 않고 모듈을 다루는 명령마다 처음에 계산한다고 설명합니다.

**다른 언어에서는** 비교를 공식 문서로 확인한 범위에서 표로 모았습니다.

| | Cargo | npm | pip | Maven | Go 모듈 |
| --- | --- | --- | --- | --- | --- |
| 선언 파일 | `Cargo.toml` | `package.json` | requirements 파일 등 | `pom.xml` | `go.mod` |
| 고른 버전의 고정 | `Cargo.lock` | `package-lock.json`(생성한 트리를 그대로 다시 만들 수 있게 기록) | `pip lock`(문서에 실험 기능으로 표시, `pylock.toml`) | 기본 잠금 파일 없음 | 잠금 파일 없음(MVS가 `go.mod`만으로 정함), `go.sum`은 해시 |
| 같은 의존성의 두 버전이 만나면 | 호환되면 가장 높은 버전 하나, 메이저 버전이 다르면 둘 다 | — | — | 의존성 트리에서 가장 가까운 정의를 고름 | 모듈마다 요구된 버전 가운데 가장 높은 것. v2부터는 경로가 다른 모듈(`/v2`) |
| 빌드·테스트·문서·게시 | `cargo` 하나 | 테스트·빌드는 `package.json`의 `scripts`에 적은 명령을 `npm test`·`npm run`으로 부르고, 게시는 `npm publish` | 빌드 도구는 따로 | 플러그인으로 | 빌드·테스트·문서는 `go`, 게시는 저장소에 태그를 푸시(별도 업로드 명령 없음) |

npm과 pip의 칸에서 "—"는 이 챕터가 공식 문서로 확인하지 않은 부분입니다.

**오해와 한계** Cargo의 규칙은 "잠금 파일이 있으면 그것대로, 없으면 호환 범위의 최신"이라, 라이브러리를 만들 때는 의존성의 새 버전이 나오는 순간 사용자의 빌드가 달라질 수 있습니다. 호환 범위 안에서의 깨짐은 SemVer를 지키는 관례에 기대므로, 버전을 올리는 쪽의 규칙이 중요합니다(부록 A ch21). Go의 MVS는 새 버전을 자동으로 받지 않는 대신, 보안 수정도 직접 올려야 받습니다. 예제는 비교를 위한 흉내라 기능(features), 프리릴리스, 되돌아가기(backtracking)는 다루지 않습니다. 세 도구에서 같은 일을 하는 명령은 아래 "한눈에 보기"의 명령 대응 표에 모았습니다.

**더 읽기** 부록 A ch21·ch25 / Part 3 ch2 / [ch13](../ch13-headers-build-templates/README.md) Q13.4(C/C++의 빌드와 패키지 관리와 비교) / Cargo Book "Specifying Dependencies"

### Q16.2 표준 라이브러리가 왜 작은가요? (난수, 정규식, HTTP, JSON이 없는 이유)

**짧은 답** 표준 라이브러리에 들어간 API는 안정성 보장(Q16.5) 때문에 지울 수 없고, 모든 프로그램이 같은 버전을 쓰게 됩니다. 그래서 설계가 빨리 바뀌거나, 좋은 설계가 여럿이거나, 운영체제와 표준(HTTP, TLS)에 따라 바뀌는 기능은 크레이트로 둡니다. 난수 API(`std::random`)는 이 툴체인에서 아직 nightly 전용이고, 정규식·JSON·HTTP는 crates.io의 크레이트(`regex`, `serde_json`, `reqwest` 등)를 씁니다.

**왜 그런가** 이 툴체인의 std 문서는 `std::random` 모듈을 "nightly-only experimental API"로 표시하고, 안정판에서 `std::random::random`을 쓰면 E0658(`` use of unstable library feature `random` ``)입니다. 불안정한 라이브러리 API를 쓰면 E0658이 나는 모습은 [Part 1 ch1](../../part1-standard-lib/ch1-std-overview/README.md)에서, 그래서 난수에는 여전히 `rand` 크레이트가 필요하다는 것은 [부록 A ch16](../../appd-a/ch16-avoid-unsafe/README.md)의 표에서 봤습니다. 크레이트로 두면 각 크레이트가 자기 속도로 메이저 버전을 올릴 수 있고, 한 프로그램 안에 같은 크레이트의 호환되지 않는 두 버전이 함께 있을 수도 있습니다(Q16.1).

**다른 언어에서는** Python은 HTTP 클라이언트·서버와 JSON을 표준 라이브러리에 두고, 튜토리얼에서 이를 "batteries included" 철학이라고 부릅니다. Go도 HTTP와 JSON을 표준 라이브러리에 둡니다. 바로 쓸 수 있는 대신 표준 라이브러리 안의 오래된 API를 오래 유지해야 합니다. Node.js는 HTTP 클라이언트·서버를 핵심 모듈(`node:http`)로 두고 JSON과 정규식은 JavaScript 언어에 있지만, 그 밖의 많은 기능은 npm의 패키지에 기댑니다.

**오해와 한계** 표준 라이브러리가 작다는 것은 처음 시작할 때 고를 크레이트가 많다는 뜻입니다. 같은 일을 하는 크레이트가 여럿이면 무엇을 고를지 판단하는 비용이 들고(Q16.3), 의존성 하나가 다른 의존성을 끌고 들어옵니다(부록 A ch25). 반대로 스레드, 채널, 파일, TCP, 컬렉션처럼 표준 라이브러리만으로 되는 일도 넓습니다([Part 1](../../part1-standard-lib/README.md)).

**더 읽기** Part 1 ch1 / Q16.3 / 부록 A ch25

### Q16.3 크레이트는 어떻게 고르고 믿나요? 공급망 위험은 어떻게 줄이나요?

**짧은 답** 최근 릴리스와 관리 상태, 내려받기 수와 쓰는 프로젝트, 끌고 오는 의존성의 수, 라이선스, `unsafe`와 빌드 스크립트·절차적 매크로의 유무를 보고 고릅니다. 고른 버전은 `Cargo.lock`이 체크섬과 함께 고정하고, CI에서는 `--locked`로 잠금 파일과 다른 해석을 막습니다. 알려진 취약점은 RustSec 데이터베이스를 확인하는 도구(`cargo-audit`)나, 라이선스·출처 정책까지 검사하는 도구(`cargo-deny`), 사람이 검토한 기록을 나누는 도구(`cargo-vet`)로 점검합니다. 막 게시된 버전을 일정 시간 고르지 않게 하는 Cargo의 최소 게시 경과 시간(min-publish-age) 설정도 있습니다. 공급망 안전은 언어가 보장하는 영역이 아닙니다.

**왜 그런가** 크레이트는 안전한 Rust만으로도 나쁜 일을 할 수 있고, 빌드 스크립트와 절차적 매크로는 빌드하는 순간에 실행됩니다. crates.io에 악성 크레이트가 올라왔던 사례는 [ch3](../ch3-safety-limits/README.md) Q3.9에서 봤습니다. 레지스트리에서 받은 의존성은 `Cargo.lock`에 `checksum` 줄이 함께 적혀(부록 A ch25의 `real_graph` 프로젝트에서 볼 수 있음), 같은 버전의 내용이 바뀌면 알아챌 수 있습니다. 의존성을 들이기 전에 따질 것(유지 상태, 라이선스, 공급망)과 최소 게시 경과 시간 설정은 [부록 A ch25](../../appd-a/ch25-dependency-graph/README.md) 2.6절에서, 도구를 고르고 둘 곳은 [부록 A ch31](../../appd-a/ch31-tooling-ecosystem/README.md) 2.5·2.6절에서 봤습니다.

**다른 언어에서는** npm, PyPI, Maven Central도 같은 종류의 위험을 안고 있습니다(ch3 Q3.9). 공급망 위험은 언어가 아니라 패키지 생태계의 문제입니다.

**오해와 한계** 도구는 알려진 취약점만 찾습니다. 아직 알려지지 않은 악성 코드나, 성실하지만 실수가 있는 코드는 사람이 검토하거나 의존성을 줄여야 막을 수 있습니다. Cargo FAQ는 `cargo new`가 기본으로 `Cargo.lock`을 버전 관리에 넣으며, 넣을지는 패키지의 필요에 달렸다고 적고, 잠금 파일이 주는 결정적 빌드의 이점(버그를 찾는 `git bisect`, 외부 요인 없이 실패하는 CI 등)을 듭니다. 이 챕터는 세 도구를 설치하지 않아 실행해 보지 않았습니다.

**더 읽기** ch3 Q3.9 / 부록 A ch25·ch31 / Cargo Book "Cargo.toml vs Cargo.lock"·FAQ

### Q16.4 crates.io에는 왜 네임스페이스가 없나요?

**짧은 답** 이 질문의 네임스페이스(namespace)는 크레이트 이름 앞에 붙여 여러 크레이트를 한 주인 아래 묶는 접두사를 말합니다(npm의 범위 같은 것). crates.io는 처음부터 그런 접두사 없이, 모든 크레이트 이름이 한 목록 안에서 겹치지 않아야 하는 하나의 평평한 이름 공간으로 시작했습니다. 그래서 `serde` 같은 짧은 이름은 한 크레이트만 가질 수 있고, 좋은 이름을 먼저 차지하는 문제(스쿼팅, squatting)와 비슷한 이름으로 속이는 문제(타이포스쿼팅, typosquatting)가 생깁니다. RFC 3243(packages as optional namespaces)은 크레이트 `foo`의 주인만 `foo::bar`처럼 그 이름 아래 크레이트를 만들 수 있게 하는 제안으로 2024년에 채택됐습니다. 다만 일반적인 스쿼팅을 막으려는 것은 아니고(RFC의 FAQ), 확인일(2026-11-21) 기준 아직 안정화되지 않아 Cargo에는 nightly에서만 켤 수 있는 `open-namespaces` 기능으로 들어 있습니다.

**왜 그런가** RFC 3243의 안내 절은 크레이트 `foo`의 주인만 `foo::bar` 크레이트를 만들 수 있고, `foo`의 주인은 모두 `foo::bar`의 주인이 된다고 적습니다. 동기(Motivation) 절은 조직이 프로젝트를 여러 크레이트로 나누는 흔한 방식(`tokio`, `async-std` 등)을 예로 들고, 이것이 남보다 먼저 좋은 이름을 차지하는 일반적인 스쿼팅과는 다른 문제, 곧 이미 고르고 쓰고 있는 이름에 다른 사람이 끼어드는 문제라고 설명합니다. RFC는 2024-03-11에 병합되어 채택됐지만, 추적 이슈 rust-lang/rust#122349는 확인일에 아직 열려 있어(마지막 갱신 2026-08-05) 안정판과 crates.io에서 쓸 수 있는 기능은 아닙니다. Cargo Book의 "Unstable Features"에는 `open-namespaces`(추적 이슈 rust-lang/cargo#13576)가 있고, 이 저장소의 cargo 1.100.0에서 패키지 이름을 `"foo::bar"`로 적으면 ``feature `open-namespaces` is required``로, `cargo-features = ["open-namespaces"]`를 더하면 ``the cargo feature `open-namespaces` requires a nightly version of Cargo, but this is the `stable` channel``로 거부됩니다. 비슷한 이름으로 속인 악성 크레이트의 사례는 ch3 Q3.9에서 봤습니다.

**다른 언어에서는** npm은 `@somescope/somepackagename`처럼 범위(scope)로 관련 패키지를 묶고(npm 문서 "scope"), Maven은 그룹 ID로, Go는 모듈 경로(보통 저장소 주소)로 이름이 겹치지 않게 합니다.

**오해와 한계** RFC의 FAQ는 이 제안이 일반적인 스쿼팅을 다루려는 것이 아니고, `coolcratename`의 주인이 `coolcratename::derive`를 남에게 빼앗기지 않게 할 뿐이라고 답합니다. 그래서 `foo-bar`처럼 네임스페이스 밖의 비슷한 이름은 여전히 누구나 게시할 수 있습니다. 네임스페이스가 생겨도 이름만 보고 믿지 말고 주인과 저장소를 확인해야 합니다. 반대로 평평한 이름 공간은 이름이 짧고 기억하기 쉽다는 장점이 있습니다.

**더 읽기** ch3 Q3.9 / RFC 3243

### Q16.5 언어가 자주 바뀌지 않나요? (6주 릴리스, 안정성 보장, 에디션)

**짧은 답** 새 안정판은 6주마다 나오고, 릴리스 채널(release channel)은 nightly, beta, stable 셋입니다. 하지만 1.0 이후 "정체 없는 안정성(stability without stagnation)"을 원칙으로, 안정판에서 컴파일되던 코드는 새 버전에서도 컴파일되도록 지킵니다. 키워드 추가처럼 호환되지 않는 변경은 몇 년에 한 번 나오는 에디션으로 묶고, 각 크레이트가 자기 에디션을 고르며 에디션이 다른 크레이트끼리도 함께 쓸 수 있습니다.

**왜 그런가** The Book의 부록 G는 Rust가 "6주마다, 시계처럼" 릴리스하고 nightly·beta·stable 세 채널이 있으며, 기능은 nightly에서 시작해 beta를 거쳐 stable로 간다고 설명합니다. 같은 부록이 이 방식을 "정체 없는 안정성"이라 부릅니다. 에디션이 무엇이고 2015·2018·2021·2024 에디션에서 무엇이 바뀌었으며, 에디션이 다른 크레이트를 섞는 방법은 Part 2 전체에서 봤습니다.

**다른 언어에서는** C++는 몇 년마다 새 표준을 냅니다(ch12 Q12.4). Go는 Go 1 명세로 쓴 프로그램이 그 명세가 유지되는 동안 바꾸지 않고 계속 컴파일되고 실행되도록 하겠다는 호환성 약속(Go 1 and the Future of Go Programs)을 둡니다. Rust의 안정성 보장도 같은 방향이고, 거기에 에디션이라는 장치를 더한 것입니다.

**오해와 한계** 안정성 보장에도 예외가 있습니다. RFC 1122(Language SemVer)는 건전성(soundness) 버그를 고치는 변경과 타입 추론의 일부 세부가 바뀌는 변경을 허용된 변경으로 들므로, 아주 드물게 컴파일되던 코드가 깨질 수 있습니다. 앞으로 에러가 될 코드는 먼저 장래 호환성 경고로 알립니다([Part 3 ch11](../../part3-cargo/ch11-commands-tools/README.md) 2.6절). 또 6주마다 새 기능이 나오므로, 의존성이 새 기능을 쓰면 최소 지원 Rust 버전(MSRV)을 맞춰야 합니다([Part 2 ch5](../../part2-editions/ch5-rust-2024-ecosystem/README.md)).

**더 읽기** Part 2 / Part 3 ch11 / The Book 부록 G

### Q16.6 nightly는 무엇이고 써야 하나요?

**짧은 답** nightly는 매일 만들어지는 개발판으로, `#![feature(...)]`로 켜는 불안정 기능과 불안정 API를 쓸 수 있습니다. 그 기능이 꼭 필요할 때(아직 안정화되지 않은 언어 기능, Miri, 일부 퍼징·벤치마크 도구 등)만 쓰고, 제품 코드는 안정판으로 빌드하는 것이 보통입니다. 불안정 기능은 바뀌거나 없어질 수 있습니다.

**왜 그런가** 안정판에서 `#![feature]`를 쓰면 E0554, 불안정 API를 쓰면 E0658입니다([부록 A ch30](../../appd-a/ch30-beyond-unit-tests/README.md) 4.1절, Part 1 ch1). Polonius Alpha 같은 새 대여 검사기도 nightly에서 먼저 시험합니다([ch15](../ch15-pain-points/README.md) Q15.3). Miri와 `cargo-fuzz`가 nightly를 쓴다는 것은 [부록 A ch16](../../appd-a/ch16-avoid-unsafe/README.md)·ch30에서 봤습니다.

**다른 언어에서는** 다른 언어도 실험 기능을 미리 쓰게 하는 장치가 있습니다. Java는 미리보기 기능을 `--enable-preview` 옵션으로 켜고(JEP 12), Go는 툴체인 실험 기능을 환경 변수 `GOEXPERIMENT`로 켭니다(`go help environment`). Rust의 nightly는 아예 다른 툴체인이라는 점이 다릅니다.

**오해와 한계** 프로젝트가 nightly를 요구하면 사용자도 nightly를 설치해야 하고, 날짜마다 동작이 달라질 수 있어 특정 날짜의 nightly로 고정하는 경우가 많습니다. 이 저장소는 nightly를 설치하지 않아, nightly가 필요한 확인은 설명만 했습니다.

**더 읽기** 부록 A ch16·ch30 / ch15 Q15.3 / Part 1 ch1

### Q16.7 언어 명세나 ISO 표준이 있나요?

**짧은 답** ISO 같은 공식 표준은 없습니다. The Rust Reference가 언어의 주 참고서이고, 컴파일러의 동작과 Reference가 다르면 어느 쪽이 맞는지 따져 고치는 방식입니다. Rust 프로젝트는 2025년 3월 Ferrocene Language Specification(FLS)을 명세 작업의 일부로 받아들인다고 발표했습니다.

**왜 그런가** The Rust Reference의 소개는 이 책을 "Rust 언어의 주 참고서"라고 하고, 알려진 버그와 누락은 이슈로 관리하며, 컴파일러 동작과 본문이 맞지 않으면 어느 쪽이 맞는지 생각할 수 있게 이슈를 올려 달라고 적습니다. 이 저장소가 사실의 최종 기준을 실제 툴체인으로 두는 것도 이 때문입니다. Rust 블로그 "Adopting the FLS"(2025-03-26)는 Ferrous Systems와 AdaCore가 함께 만들고 Ferrous Systems가 관리해 온 FLS를 Rust 프로젝트의 명세 작업 일부로 받아들이겠다고 밝혔고, 프로젝트 안에서는 풀어 쓴 이름 대신 FLS라고만 부르겠다고 적었습니다.

**다른 언어에서는** C와 C++는 ISO 표준이 있고 여러 컴파일러가 그 표준을 구현합니다([ch12](../ch12-modern-cpp-enough/README.md) Q12.4·Q12.6). Java는 언어 명세(JLS)를, Go는 언어 명세를 공식 문서로 둡니다.

**오해와 한계** 공식 표준이 없다는 것은 규제 산업(자동차, 항공 등)에서 도구를 인증받을 때 걸림돌이 될 수 있고, FLS는 그런 요구에서 시작된 명세입니다. 다만 표준이 있어도 컴파일러마다 해석이 다를 수 있다는 점은 C/C++가 보여 줍니다.

**더 읽기** The Rust Reference / Rust 블로그 "Adopting the FLS" / Q16.8

### Q16.8 컴파일러는 rustc 하나뿐인가요?

**짧은 답** 실제로 쓰는 컴파일러는 rustc 하나입니다. rustc는 기본으로 LLVM으로 기계어를 만들고, 다른 코드 생성 백엔드(Cranelift 등)는 nightly에서 시험 중입니다. GCC 기반의 독립 구현 gccrs가 개발 중이지만, 스스로 초기 단계라고 밝히고 아직 대여 검사기가 없습니다.

**왜 그런가** gccrs 첫 페이지는 프로젝트가 아직 초기 단계이고 공식 Rust 테스트 모음을 컴파일하는 것이 목표라고 적으며, 대여 검사기는 아직 없고 Polonius 작업과 통합할 계획이라고 밝힙니다(확인일 기준 최신 소식은 2026-11-02의 "October 2026 Monthly report"). Cranelift 백엔드는 [Part 3 ch6](../../part3-cargo/ch6-profiles-build-performance/README.md) 2.7절에서 nightly로만 쓸 수 있다고 봤습니다.

**다른 언어에서는** C/C++는 GCC, Clang, MSVC처럼 여러 독립 구현이 있고, Java도 여러 JVM 구현이 있습니다. Go는 공식 구현(gc) 말고 gccgo가 있습니다. 여러 구현은 명세의 빈틈을 드러내고 공급자를 고를 수 있게 하지만, 구현마다 다른 동작도 생깁니다.

**오해와 한계** 구현이 하나라는 것은 "rustc가 곧 언어의 정의"처럼 되기 쉽다는 뜻이고, 명세 작업(Q16.7)이 그 균형을 잡으려는 것입니다. 지금의 gccrs는 대여 검사를 하지 않으므로, gccrs가 받아들인 코드라고 해서 rustc의 대여 검사를 통과한다는 뜻은 아닙니다.

**더 읽기** Q16.7 / Part 3 ch6 2.7절 / ch12 Q12.6

### Q16.9 안정 ABI가 없다는 것은 무슨 뜻인가요? 동적 라이브러리나 플러그인은 어떻게 하나요?

**짧은 답** ABI(application binary interface)는 컴파일된 코드끼리 만나는 방식(타입의 메모리 배치, 함수를 부르는 방식, 심볼 이름)입니다. Rust 타입의 기본 배치(`repr(Rust)`)는 건전성에 필요한 것 말고는 보장하지 않아 컴파일러가 필드 순서를 바꿀 수 있으므로, 다른 컴파일러 버전으로 만든 Rust 라이브러리끼리 바이너리로 섞는 것을 기대할 수 없습니다. 동적 라이브러리와 플러그인의 경계는 C ABI로 만듭니다. `extern "C"` 함수와 `#[repr(C)]` 타입을 쓰고, 크레이트 종류를 `cdylib`으로 빌드합니다.

**왜 그런가** The Rust Reference는 기본 표현(Rust representation)이 보장하는 데이터 배치가 건전성에 필요한 것(필드 오프셋이 정렬의 배수, 필드가 겹치지 않음 등)뿐이라고 적습니다. 실제로 컴파일러가 필드를 재배치해 크기를 줄이는 모습과 `#[repr(C)]`로 순서를 고정하는 방법은 [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 예제 01과 2.2절에서 봤습니다. 이 환경에서 1.98.1로 `lib98`이라는 rlib(Rust 라이브러리)을 만들고 1.100.0에서 `--extern`으로 쓰려 하자 ``error[E0514]: found crate `lib98` compiled by an incompatible version of rustc``로 거부됐습니다. `cdylib`은 다른 언어에서 불러 쓸 동적 라이브러리(Linux의 `.so`, macOS의 `.dylib`, Windows의 `.dll`)를 만드는 크레이트 종류이고(The Rust Reference "Linkage"), Cargo에서는 `[lib]`에 `crate-type = ["cdylib"]`로 적습니다. `#[unsafe(no_mangle)] pub extern "C" fn c_answer()`를 담은 소스를 `rustc --edition 2024 --crate-type cdylib`으로 빌드하자, macOS의 도구 `nm -gU`로 C 심볼 `_c_answer`가 내보내진 것이 보였고 `otool -L`로는 시스템 라이브러리(`libSystem.B.dylib`)만 링크했습니다. C와 경계를 만드는 방법은 [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md)와 [부록 A ch34](../../appd-a/ch34-ffi-boundaries/README.md)에서 봤습니다.

**다른 언어에서는** C는 플랫폼마다 사실상 정해진 호출 규약(calling convention)이 있어 언어 사이를 잇는 기준이 되고, Rust의 FFI도 그 C ABI를 씁니다(Part 1 ch14). C++도 클래스와 템플릿을 경계로 넘기면 컴파일러와 표준 라이브러리에 따라 맞지 않을 수 있어, 라이브러리 경계를 C 함수로 두는 경우가 많습니다.

**오해와 한계** C ABI로 경계를 만들면 제네릭, 트레이트 객체, `String`·`Vec` 같은 Rust 타입을 그대로 넘길 수 없고, 넘기는 쪽과 받는 쪽이 메모리 해제 책임을 직접 정해야 합니다. 플러그인을 Rust로만 만들 때도 같은 제약이 있어, 같은 컴파일러로 함께 빌드하거나(정적 링크) WebAssembly 같은 별도의 경계를 쓰는 방법을 고릅니다.

**더 읽기** Part 1 ch7·ch14 / 부록 A ch34, [부록 A ch35](../../appd-a/ch35-bindgen/README.md) / [ch17](../ch17-interop-migration/README.md) Q17.2·Q17.4

### Q16.10 누가 만들고 관리하나요? 라이선스는 무엇인가요?

**짧은 답** Rust는 여러 팀(컴파일러, 언어, 라이브러리, Cargo 등)으로 이루어진 오픈소스 프로젝트이고, 팀 대표들로 이루어진 Leadership Council이 프로젝트 전체를 맡습니다. 언어와 공식 프로젝트는 대개 Apache License 2.0과 MIT 라이선스의 이중 라이선스로 배포됩니다.

**왜 그런가** rust-lang.org의 Governance 페이지는 Leadership Council을 "최상위 팀의 대표들로 이루어지며 프로젝트 전체의 성공을 맡는" 조직으로 소개하고, 그 아래의 팀들을 나열합니다. Licenses 페이지는 Rust 언어와 이 웹사이트를 포함한 공식 프로젝트가 대개 Apache-2.0과 MIT의 이중 라이선스라고 적습니다. crates.io의 크레이트는 각자 라이선스를 정하며, `Cargo.toml`의 `license` 필드에 적습니다([Part 3 ch10](../../part3-cargo/ch10-publishing-semver/README.md)).

**다른 언어에서는** C와 C++는 ISO 위원회가 표준을 정하고(ch12 Q12.4), 여러 언어가 특정 회사의 주도 아래 개발됩니다. Rust는 프로젝트의 팀 구조와 RFC 과정이 결정을 맡습니다.

**오해와 한계** 회사가 소유하지 않는 대신, 결정은 RFC와 팀의 합의로 이루어져 시간이 걸릴 수 있습니다. 의존하는 크레이트의 라이선스는 크레이트마다 다르므로, 배포할 때는 의존성 전체의 라이선스를 확인해야 합니다(Q16.3의 `cargo-deny`).

**더 읽기** rust-lang.org Governance·Licenses / Part 3 ch10

### Q16.11 편집기, 포매터, 린터는 무엇을 쓰나요?

**짧은 답** 편집기는 무엇을 쓰든 언어 서버(language server) rust-analyzer를 붙여 자동 완성, 정의로 이동, 타입 표시, 인라인 에러를 얻습니다. 포매터는 `rustfmt`(`cargo fmt`), 린터는 Clippy(`cargo clippy`)가 공식 도구이고 툴체인과 함께 설치됩니다.

**왜 그런가** rust-lang.org의 Tools 페이지는 VS Code, Vim/Neovim, RustRover, Helix, Emacs, Sublime Text, Visual Studio, Zed 같은 편집기 통합을 소개하고, 직접 만들 때는 rust-analyzer를 쓰라고 안내합니다. `cargo test`·`cargo doc`·`cargo fmt`·`cargo clippy`가 모두 공식 도구라 따로 고르지 않아도 된다는 점은 [ch13](../ch13-headers-build-templates/README.md) Q13.9에서, 도구를 고르고 CI에 둘 곳은 부록 A ch31과 [부록 A ch29](../../appd-a/ch29-listen-to-clippy/README.md)(Clippy)에서 봤습니다.

**다른 언어에서는** Go는 `gofmt`와 `go vet`을 공식 도구로 두어 Rust와 비슷하고, JavaScript·Python은 Prettier·ESLint·Black·Ruff처럼 커뮤니티 도구를 골라 씁니다.

**오해와 한계** 편집기 통합은 rust-analyzer가 매크로와 빌드 스크립트까지 분석해야 정확해지므로, 빌드 스크립트가 무겁거나 매크로가 많은 프로젝트에서는 분석이 느려질 수 있습니다. Clippy의 기본 린트 묶음은 일부만 켜므로 더 엄격한 묶음은 직접 켭니다(부록 A ch29).

**더 읽기** ch13 Q13.9 / 부록 A ch29·ch31

### Q16.12 어떤 플랫폼을 지원하나요? 웹(WebAssembly), 모바일, 임베디드는요?

**짧은 답** `rustc`는 타깃을 세 등급으로 나눕니다. Linux·macOS·Windows의 주요 타깃은 Tier 1(동작 보장)이고, WebAssembly(`wasm32-unknown-unknown`, `wasm32-wasip1`, `wasm32-wasip2`), Android(`aarch64-linux-android`), iOS(`aarch64-apple-ios`)와 주요 임베디드 타깃(`thumbv7em-none-eabihf`, `riscv32imac-unknown-none-elf` 등)은 Tier 2(빌드 보장)이고, 나머지 운영체제 없는 타깃 가운데는 Tier 3도 많습니다. Tier 2는 공식 빌드를 내지만 자동 테스트가 늘 돌지는 않습니다.

**왜 그런가** 이 툴체인의 Platform Support 문서에서 `x86_64-unknown-linux-gnu`와 `aarch64-apple-darwin`은 "Tier 1 with Host Tools"에, 위의 WebAssembly·Android·iOS·임베디드 타깃은 "Tier 2 without Host Tools"에 있습니다(그 타깃에서 컴파일러를 돌리지는 않고, 그 타깃용으로 빌드만 함). 등급의 뜻과 운영체제 없는 타깃의 현황은 ch12 Q12.7에서 봤고, `no_std` 라이브러리는 [부록 A ch33](../../appd-a/ch33-no-std/README.md)에서, 임베디드 실습은 Part 8 `ch1-intro-no-std`에서 다룹니다.

**다른 언어에서는** Go와 Java는 런타임이 지원하는 플랫폼에서 돕니다. 모바일에서는 그 플랫폼의 주 언어로 앱을 만들고, Rust는 그 안에서 부르는 라이브러리로 쓰는 경우가 많습니다.

**오해와 한계** Tier 2 타깃도 실제로 많이 쓰이지만, 자동 테스트가 늘 돌지는 않으므로 문제가 생기면 사용자가 먼저 발견할 수 있습니다. 웹에서 Rust를 쓰려면 WebAssembly와 JavaScript를 잇는 도구가, 모바일에서는 그 플랫폼의 언어(Kotlin, Swift)와 잇는 바인딩이 필요합니다([ch17](../ch17-interop-migration/README.md)).

**더 읽기** ch12 Q12.7 / 부록 A ch33 / The rustc book "Platform Support"

### 한눈에 보기

| 질문 | Rust의 답 | 다른 생태계와의 차이 |
| --- | --- | --- |
| 패키지 관리 | Cargo + crates.io, `Cargo.lock` | Go는 MVS로 잠금 파일 없이, Maven은 가장 가까운 정의 |
| 버전 선택 | 캐럿(호환 범위)의 가장 높은 버전 | Go는 요구된 최소 버전 가운데 가장 높은 것 |
| 표준 라이브러리 | 작음(난수·정규식·HTTP·JSON은 크레이트) | Python·Go는 HTTP·JSON 포함 |
| 네임스페이스 | 평평함(RFC 3243 채택, 안정화 전) | npm 범위, Maven 그룹 ID, Go 모듈 경로 |
| 릴리스 | 6주, 안정성 보장, 에디션 | Go 1 호환성 약속, C++ 몇 년마다 표준 |
| 불안정 기능 | nightly 툴체인과 `#![feature]` | Java `--enable-preview`, Go `GOEXPERIMENT` |
| 명세 | Reference, FLS 채택(ISO 표준 없음) | C/C++ ISO, Java JLS |
| 구현 | rustc(LLVM), gccrs는 초기 단계 | C/C++·Java는 여러 구현 |
| ABI | Rust ABI 불안정, 경계는 C ABI | C의 호출 규약이 언어 사이의 기준 |
| 관리 | Leadership Council과 팀, Apache-2.0/MIT | C/C++는 ISO 위원회 |
| 플랫폼 | 주요 OS Tier 1, wasm·모바일·주요 임베디드 Tier 2 | |

패키지 관리자의 자주 쓰는 명령은 다음처럼 대응합니다. 각 도구의 공식 문서로 확인한 것만 적었습니다.

| 하려는 일 | Cargo | npm | Go 모듈 |
| --- | --- | --- | --- |
| 의존성 추가 | `cargo add 이름` | `npm install 이름` | `go get 경로` |
| 고른 버전을 새 버전으로 올리기 | `cargo update`(요구 사항 범위 안에서) | `npm update`(SemVer 제약 안에서) | `go get -u ./...`(main 모듈이 쓰는 모듈을 각각 최신으로) |
| 잠금 파일과 다른 해석이면 실패 | `--locked`(`cargo build --locked` 등) | `npm ci` | 잠금 파일은 없고, Go 1.16부터 `go.mod`를 고쳐야 하면 기본으로 에러를 냄(`-mod=readonly`처럼 동작) |

## 3. 예제 안내

로드맵의 예제 계획 가운데 "의존성 하나를 가진 프로젝트의 `Cargo.lock`과 `cargo tree`"는 Part 3 ch2의 `three-sources`, 부록 A ch25의 `real_graph`, 부록 A ch31의 연습 2에 이미 있어서 만들지 않았습니다. 또 하나의 계획인 "표준 라이브러리만으로 되는 일의 범위"는 스레드, 채널, 파일, TCP, 컬렉션을 다룬 Part 1 전체가 실행으로 보이므로 따로 만들지 않았습니다(Q16.2). 대신 다른 패키지 관리자와의 차이(Q16.1)를 보이는 예제 하나를 둡니다.

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_version_selection](examples/01_version_selection.rs) | Cargo의 캐럿 규칙과 Go의 최소 버전 선택을 흉내 내어 같은 요구 사항에서 고르는 버전 비교 | Q16.1 |

### 01_version_selection — 버전을 고르는 두 규칙

```sh
cargo run -p faq-ch16-tooling-ecosystem --example 01_version_selection
```

```text
1. 레지스트리: 1.2.0, 1.4.0, 1.6.0, 2.0.0
  요구 사항: 1.2.0, 1.4.0
  Cargo(잠금 파일 없음): 1.6.0
  Go MVS: 1.4.0
2. 1.7.0이 새로 올라온 뒤
  요구 사항: 1.2.0, 1.4.0
  Cargo(잠금 파일 없음): 1.7.0
  Go MVS: 1.4.0
  Cargo.lock이 있으면: cargo update 전까지 1.6.0을 그대로 씀
3. 0.x 버전의 캐럿 규칙
  ^0.4.1 요구 사항에 0.4.1: true
  ^0.4.1 요구 사항에 0.4.9: true
  ^0.4.1 요구 사항에 0.5.0: false
```

- `caret_matches`는 Cargo Book의 캐럿 규칙대로 "가장 왼쪽의 0이 아닌 자리까지 같고, 요구 버전 이상"인지 봅니다. Part 3 ch2 2.2절과 예제 `01_version_requirements`에서 `semver` 크레이트로 확인한 규칙을 손으로 옮긴 것이고, 이 예제가 더하는 것은 그 규칙을 Go의 규칙과 나란히 놓는 부분입니다. 그래서 `1.x`끼리는 호환되지만 `0.4.x`와 `0.5.x`는 호환되지 않습니다(SemVer의 0.x 관례는 부록 A ch21).
- `cargo_pick`은 모든 요구 사항과 호환되는 버전 가운데 가장 높은 것을, `mvs_pick`은 요구된 최소 버전 가운데 가장 높은 것을 고릅니다.
- 2번은 새 버전이 나왔을 때의 차이를 보입니다. 잠금 파일이 없는 Cargo는 새 버전을 고르고, `Cargo.lock`이 있으면 `cargo update`를 할 때까지 그대로입니다. Go의 MVS는 `go.mod`의 요구가 바뀌지 않는 한 그대로입니다.
- 흉내일 뿐이라 메이저 버전이 다른 요구(1.x와 2.x)를 제대로 다루지 못합니다(연습 1).

## 4. 자주 하는 실수

이 챕터의 실수는 컴파일 에러가 아니라 도구와 생태계를 다루는 방식에서 나오므로 `compile_fail/`을 두지 않았습니다.

| 실수 | 왜 틀렸나 | 바로잡기 | 관련 |
| --- | --- | --- | --- |
| (재현이 필요한 프로젝트에서) `Cargo.lock`을 커밋하지 않음 | CI와 다른 사람의 빌드가 다른 버전을 고를 수 있습니다 | 커밋하고 CI에서 `--locked`를 씁니다 | Q16.3, Part 3 ch2 |
| 이름만 보고 크레이트를 고름 | 비슷한 이름의 악성 크레이트가 있었습니다 | 주인, 저장소, 내려받기 수, 의존성을 확인합니다 | Q16.4, ch3 Q3.9 |
| 불안정 기능 하나 때문에 프로젝트 전체를 nightly로 | 사용자도 nightly가 필요해지고 날짜마다 달라질 수 있습니다 | 안정판의 대안을 찾거나 그 부분을 떼어 냅니다 | Q16.6 |
| Rust 타입을 동적 라이브러리 경계로 넘김 | Rust ABI는 안정되어 있지 않습니다 | `extern "C"`, `#[repr(C)]`, `cdylib`으로 경계를 만듭니다 | Q16.9 |
| Tier 2 타깃을 Tier 1처럼 믿음 | 자동 테스트가 늘 돌지는 않습니다 | 그 타깃에서 직접 테스트합니다 | Q16.12 |
| 안정판에서 `#![feature]`·불안정 API를 씀 | E0554·E0658 | nightly가 정말 필요한지 봅니다 | 부록 A ch30 4.1절, Part 1 ch1 |

## 5. 연습 문제

### 연습 1. 메이저 버전이 다른 요구

`examples/01_version_selection.rs`의 `reqs`를 `[Version(1, 4, 0), Version(2, 0, 0)]`으로 바꿔 실행해 보세요. 예제는 무엇을 출력하고, 실제 Cargo와 Go는 어떻게 할까요?

<details>
<summary>힌트</summary>

예제는 `Cargo(잠금 파일 없음): 맞는 버전 없음`과 `Go MVS: 2.0.0`을 출력합니다. 둘 다 실제와 다릅니다. 실제 Cargo는 호환되지 않는 두 요구를 위해 1.x 사본과 2.x 사본을 함께 컴파일합니다(부록 A ch25 2.3절). Go는 v2부터 모듈 경로에 `/v2`를 붙여 다른 모듈로 다루므로(Go Modules Reference "Major version suffixes"), 1.4.0과 2.0.0을 함께 씁니다. 예제의 흉내가 메이저 버전을 따로 다루지 않았기 때문에 생긴 차이입니다.

</details>

### 연습 2. Go 쪽 흉내에 메이저 버전 경로 더하기

연습 1에서 본 것처럼 `mvs_pick`은 메이저 버전을 구분하지 않습니다. `reqs`가 `[1.4.0, 2.0.0]`일 때 실제 Go처럼 두 버전을 함께 고르게 하려면 `mvs_pick`을 어떻게 고쳐야 할까요? v0과 v1은 어떻게 다뤄야 할까요?

<details>
<summary>힌트</summary>

Go에서 v2부터는 모듈 경로에 `/v2` 같은 접미사가 붙어 다른 모듈이 되고, v0과 v1에는 접미사를 붙이지 않습니다(Go Modules Reference "Major version suffixes"). 그래서 요구 사항을 "경로"별로 묶고, 묶음마다 가장 높은 요구를 고르면 됩니다. 경로는 메이저 버전으로 정하되 0과 1은 같은 묶음으로 봅니다. 예를 들어 `BTreeMap<u64, Version>`([Part 1 ch4](../../part1-standard-lib/ch4-collections/README.md))에 `r.0.max(1)`을 키로 하여 묶음별 최댓값을 넣고, 돌려주는 값을 `Vec<Version>`으로 바꿉니다. Part 0에서 배운 `HashMap`에 모은 뒤 키로 정렬해도 됩니다. 이렇게 고치면 `[1.4.0, 2.0.0]`에서 `[1.4.0, 2.0.0]`을, `[0.9.0, 1.1.0, 2.3.0, 2.1.0]`에서 `[1.1.0, 2.3.0]`을 고릅니다.

</details>

### 연습 3. WebAssembly 타깃 찾기

`rustc --print target-list`로 `wasm32`로 시작하는 타깃이 몇 개인지 세어 보고, 그 가운데 `wasm32-unknown-unknown`과 `wasm32-wasip2`의 등급을 The rustc book의 Platform Support에서 찾아 보세요.

<details>
<summary>힌트</summary>

이 툴체인에서는 `rustc --print target-list | grep -c '^wasm32'`가 8이었습니다. 두 타깃 모두 "Tier 2 without Host Tools"입니다. `--print target-list`는 이 `rustc`가 컴파일할 수 있는 타깃의 이름을 보여 줄 뿐이고, 그 타깃용 표준 라이브러리가 설치되어 있다는 뜻은 아닙니다.

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Cargo Book: [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#caret-requirements)(캐럿 요구 사항), [Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html), [FAQ](https://doc.rust-lang.org/cargo/faq.html)(`Cargo.lock`을 버전 관리에 넣는 이유)
- The Rust Programming Language: [부록 G How Rust is Made and "Nightly Rust"](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html)
- The Rust Reference: [Introduction](https://doc.rust-lang.org/reference/introduction.html), [Type Layout](https://doc.rust-lang.org/reference/type-layout.html)(Rust representation), [Linkage](https://doc.rust-lang.org/reference/linkage.html)(`cdylib`)
- The rustc book: [Platform Support](https://doc.rust-lang.org/rustc/platform-support.html)
- 표준 라이브러리 문서: [`std::random`](https://doc.rust-lang.org/std/random/index.html)(nightly 전용 표시)
- Rust 블로그: [Adopting the FLS](https://blog.rust-lang.org/2025/03/26/adopting-the-fls/)(2025-03-26)
- RFC 3243 [Packages as (optional) namespaces](https://rust-lang.github.io/rfcs/3243-packages-as-optional-namespaces.html), 추적 이슈 [rust-lang/rust#122349](https://github.com/rust-lang/rust/issues/122349)
- RFC 1122 [Language SemVer](https://rust-lang.github.io/rfcs/1122-language-semver.html)
- The Cargo Book: [cargo build](https://doc.rust-lang.org/cargo/commands/cargo-build.html)(`--locked`), [Unstable Features](https://doc.rust-lang.org/cargo/reference/unstable.html#open-namespaces)(`open-namespaces`)
- [Cargo Vet](https://mozilla.github.io/cargo-vet/)
- rust-lang.org: [Governance](https://www.rust-lang.org/governance), [Licenses](https://www.rust-lang.org/policies/licenses), [Tools](https://www.rust-lang.org/tools)
- [gccrs](https://rust-gcc.github.io/)

다른 생태계

- Go: [Go Modules Reference](https://go.dev/ref/mod)([Minimal version selection](https://go.dev/ref/mod#minimal-version-selection), [go.sum files](https://go.dev/ref/mod#go-sum-files), [Major version suffixes](https://go.dev/ref/mod#major-version-suffixes))
- Go: [Go 1 and the Future of Go Programs](https://go.dev/doc/go1compat), [Publishing a module](https://go.dev/doc/modules/publishing), [go get](https://go.dev/ref/mod#go-get)
- npm: [package-lock.json](https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json), [scope](https://docs.npmjs.com/cli/v11/using-npm/scope), [npm test](https://docs.npmjs.com/cli/v11/commands/npm-test), [npm publish](https://docs.npmjs.com/cli/v11/commands/npm-publish), [npm install](https://docs.npmjs.com/cli/v11/commands/npm-install), [npm update](https://docs.npmjs.com/cli/v11/commands/npm-update), [npm ci](https://docs.npmjs.com/cli/v11/commands/npm-ci)
- Node.js: [HTTP](https://nodejs.org/api/http.html)
- Java: [JEP 12: Preview Features](https://openjdk.org/jeps/12)
- Python 튜토리얼: [Batteries Included](https://docs.python.org/3/tutorial/stdlib.html#batteries-included)
- pip: [User Guide: Requirements Files](https://pip.pypa.io/en/stable/user_guide/#requirements-files), [pip lock](https://pip.pypa.io/en/stable/cli/pip_lock/)
- Maven: [Introduction to the Dependency Mechanism](https://maven.apache.org/guides/introduction/introduction-to-dependency-mechanism.html)
