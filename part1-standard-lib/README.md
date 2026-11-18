# Part 1. 표준 라이브러리

[표준 라이브러리 API 문서](https://doc.rust-lang.org/std/index.html)(`std`)를 모듈 단위로 읽고 실제로 사용해 보는 파트입니다. Part 0에서 맛만 본 타입들의 메서드를 넓고 깊게 익히고, 처음 보는 타입도 API 문서만 보고 쓸 수 있게 되는 것이 목표입니다.

[Part 0](../part0-getting-started/)의 내용을 모두 전제로 합니다.

## 학습 방법

챕터 번호 순서대로 진행합니다. 각 챕터의 `README.md`를 읽으면서 예제를 직접 실행해 봅니다.

```sh
cargo run -p <패키지 이름> --example <예제 이름>
```

패키지 이름과 예제 이름은 각 챕터 README의 "예제 안내"에 적혀 있습니다. 챕터마다 "참고 자료"에 해당 모듈의 API 문서 링크가 있으니, 예제에 나온 메서드를 문서에서 직접 찾아 시그니처를 읽어 보면 좋습니다.

## 챕터 목차

| 챕터 | 주제 |
| --- | --- |
| [ch1-std-overview](ch1-std-overview/) | `core`/`alloc`/`std`, 프렐류드, API 문서 읽는 법(타입 페이지·시그니처·표시·검색), 정수·실수·`char`·`bool`·배열·슬라이스·튜플의 메서드 |
| [ch2-option-result](ch2-option-result/) | `Option`/`Result` 콤비네이터(`map`·`and_then`·`map_err`·`unwrap_or_else` 등)와 선택표, 즉시 평가와 게으른 평가, 참조로 들여다보기(`as_ref`·`as_deref`), 상호 변환(`ok_or`·`transpose`), `?`와의 관계, `collect`로 모으기 |
| [ch3-strings-fmt](ch3-strings-fmt/) | 문자열 찾기·나누기·다듬기·바꾸기와 검색 패턴, 글자 경계와 바이트 변환, 서식 문법 전체, `Display`/`Debug` 직접 구현(`Formatter`), `FromStr`과 파싱 |
| [ch4-collections](ch4-collections/) | 컬렉션 선택 기준과 연산별 비용, `Vec` 편집·용량 관리, 정렬과 이진 탐색, entry API, `VecDeque`, `BTreeMap`/`BTreeSet`의 범위 조회, 집합 연산, `BinaryHeap` 우선순위 큐 |

작성된 챕터만 목차에 있습니다. 전체 계획은 [로드맵](../docs/roadmap.md)을, 진행 상황은 [PROGRESS.md](../PROGRESS.md)를 참고합니다.
