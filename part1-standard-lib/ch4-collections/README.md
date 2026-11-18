# ch4. 컬렉션: 고르는 기준과 깊이 있는 사용법

[Part 0 ch8](../../part0-getting-started/ch8-collections/README.md)에서 벡터 `Vec<T>`와 해시 맵 `HashMap<K, V>`의 기본(만들기, `push`, 인덱싱과 `get`, 순회, `entry().or_insert`, 용량과 재할당)을 배웠습니다. 이 챕터는 그 위에서 두 방향으로 넓힙니다.

- **고르기**: 표준 라이브러리 `std::collections`에는 `Vec`과 `HashMap` 말고도 `VecDeque`, `BTreeMap`, `HashSet`, `BTreeSet`, `BinaryHeap`, `LinkedList`가 있습니다. 문제 상황을 보고 어떤 컬렉션을 왜 고르는지, 그 판단의 근거가 되는 연산 비용 표를 읽는 법을 배웁니다.
- **깊이 쓰기**: `Vec`과 슬라이스의 편집·정렬·이진 탐색 메서드, 맵의 entry API, 직접 만든 타입을 키로 쓰는 조건을 다룹니다.

컬렉션마다 "이것이 아니면 곤란한 상황"을 예제로 하나씩 보입니다. 문서의 표를 외우는 것보다, 예제의 상황을 떠올려 "여기서 `Vec`을 쓰면 무엇이 느려지는가"를 따져 보는 것이 목표입니다.

- 패키지: `p1-ch4-collections`
- 기준 문서: [`std::collections`](https://doc.rust-lang.org/std/collections/index.html), [`std::vec`](https://doc.rust-lang.org/std/vec/index.html), [`std::slice`](https://doc.rust-lang.org/std/slice/index.html)와 [슬라이스 원시 타입](https://doc.rust-lang.org/std/primitive.slice.html)
- 확인한 툴체인: rustc 1.98.1, cargo 1.98.1 (Edition 2024)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `std::collections` 문서의 선택 기준과 연산 비용 표를 읽고, 주어진 문제 상황에 맞는 컬렉션을 고른 이유를 비용으로 설명할 수 있다. `LinkedList`를 거의 쓰지 않는 이유도 설명할 수 있다.
2. `Vec`의 중간 삽입·삭제(`insert`, `remove`, `swap_remove`)가 얼마나 드는지 설명하고, 여러 요소를 한꺼번에 다루는 메서드(`extend`, `drain`, `retain`, `dedup`, `truncate`, `split_off`)와 용량 관리 메서드(`with_capacity`, `reserve`, `shrink_to_fit`)를 상황에 맞게 쓸 수 있다.
3. `sort`/`sort_unstable`/`sort_by`/`sort_by_key`/`sort_by_cached_key`를 구분해 쓰고 안정 정렬의 의미를 설명할 수 있으며, `f64`를 `total_cmp`로 정렬할 수 있다. `binary_search`가 돌려주는 `Ok`/`Err`를 해석하고, `partition_point`로 경계를 찾아 정렬 상태를 유지하며 삽입할 수 있다.
4. entry API(`or_insert_with`, `or_default`, `and_modify`, `Entry` 열거형의 `match`)로 "조회 후 삽입"을 한 번의 탐색으로 쓸 수 있다. 해시 맵과 해시 집합의 키에는 `Eq + Hash`, `BTreeMap`/`BTreeSet`의 키와 `BinaryHeap`의 원소에는 `Ord`가 필요하다는 것을 API 문서의 바운드에서 읽어 내고, 직접 만든 타입에 `derive`로 해결할 수 있다.
5. `VecDeque`로 큐와 "최근 N개"를, `BTreeMap`/`BTreeSet`으로 범위 조회와 최소·최대·기준점 찾기를, `HashSet`/`BTreeSet`으로 집합 연산을, `BinaryHeap`과 `Reverse`로 우선순위 큐와 top-k를 구현할 수 있다.

## 2. 핵심 개념

2.1~2.4절은 컬렉션을 고르는 기준, 2.5~2.9절은 `Vec`과 슬라이스, 2.10~2.11절은 맵과 키, 2.12~2.15절은 나머지 컬렉션을 하나씩 다룹니다.

### 2.1 표준 컬렉션 한눈에 보기

`std::collections` 모듈 문서는 표준 컬렉션을 네 무리로 나눕니다.

| 분류 | 컬렉션 | 한 줄 설명 |
| --- | --- | --- |
| 순차(sequence) | `Vec<T>` | 늘어나는 배열. 요소가 메모리에 빈틈없이 이어져 있습니다. |
| | `VecDeque<T>` | 양쪽 끝에서 넣고 빼는 일이 모두 빠른 덱(deque). 링 버퍼로 만듭니다. |
| | `LinkedList<T>` | 이중 연결 리스트. 거의 쓰지 않습니다(2.4절). |
| 맵(map) | `HashMap<K, V>` | 해시 함수로 키의 자리를 계산하는 맵. 순서가 없습니다. |
| | `BTreeMap<K, V>` | B-트리로 만든 맵. 키가 항상 정렬되어 있습니다. |
| 집합(set) | `HashSet<T>` | 값만 있는 `HashMap`. 실제로 `HashMap<T, ()>`를 감싼 타입입니다. |
| | `BTreeSet<T>` | 값만 있는 `BTreeMap`. |
| 기타 | `BinaryHeap<T>` | 이진 힙으로 만든 우선순위 큐(priority queue). 가장 큰 값을 빨리 꺼냅니다. |

`Vec`은 프렐류드에 있지만 나머지는 모두 `use std::collections::{VecDeque, BTreeMap};`처럼 가져와야 합니다.

모듈 문서는 첫머리에서 이렇게 못 박습니다. "아마 그냥 `Vec`이나 `HashMap`을 쓰면 됩니다. 이 둘이 대부분의 경우를 다루고, 다른 컬렉션은 그것이 최선인 특정한 경우가 있지만 비교하면 꽤 드뭅니다." 그러니 기본값은 `Vec`과 `HashMap`이고, 이 챕터에서 배우는 것은 **언제 기본값을 벗어나야 하는가**입니다.

### 2.2 비용 표 읽기

컬렉션을 고르는 근거는 "자주 하는 연산이 얼마나 드는가"입니다. 문서는 비용을 O 표기로 적고, 다음 약속을 씁니다.

- `n`은 컬렉션의 크기, `m`은 두 번째 컬렉션의 크기, `i`는 인덱스입니다.
- **`*` 분할 상환(amortized) 비용**: 가끔은 비싸지만 길게 보면 평균이 그 값이라는 뜻입니다. `Vec`의 `push`는 용량이 꽉 차면 더 큰 공간을 할당해 모든 요소를 옮기므로(O(n)) 그 한 번은 느립니다. 하지만 용량을 넉넉히 늘려 두므로 그런 일은 드물게 일어나고, `push` 여러 번에 나눠 계산하면 한 번에 O(1)입니다. Rust의 컬렉션은 저절로 줄어들지 않으므로 삭제 연산에는 `*`가 붙지 않습니다.
- **`~` 기대(expected) 비용**: 확률적으로 그 값이라는 뜻입니다. 해시 맵은 서로 다른 키의 해시가 겹치면 추가 작업이 필요한데, 그런 일이 많이 생길 가능성은 매우 낮습니다.

다음은 문서의 "Cost of Collection Operations" 표입니다(열 이름만 옮겼습니다).

| | `get(i)` | `insert(i)` | `remove(i)` | `append(Vec(m))` | `split_off(i)` | `range` | `append` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Vec` | O(1) | O(n-i)* | O(n-i) | O(m)* | O(n-i) | 없음 | 없음 |
| `VecDeque` | O(1) | O(min(i, n-i))* | O(min(i, n-i)) | O(m)* | O(min(i, n-i)) | 없음 | 없음 |
| `LinkedList` | O(min(i, n-i)) | O(min(i, n-i)) | O(min(i, n-i)) | O(1) | O(min(i, n-i)) | 없음 | 없음 |
| `HashMap` | O(1)~ | O(1)~* | O(1)~ | 없음 | 없음 | 없음 | 없음 |
| `BTreeMap` | O(log(n)) | O(log(n)) | O(log(n)) | 없음 | 없음 | O(log(n)) | O(n+m) |

- 맵에서 `get(i)`, `insert(i)`, `remove(i)`의 `i`는 인덱스가 아니라 키입니다.
- 집합(`HashSet`, `BTreeSet`)의 비용은 같은 종류의 맵과 같습니다.
- `Vec`의 `insert(i)`가 O(n-i)인 것은 `i`번 뒤의 요소를 모두 한 칸씩 밀어야 하기 때문입니다. 끝(`i = n`)에 넣으면 O(1)이 됩니다. `VecDeque`는 앞과 뒤 중 가까운 쪽을 밀면 되므로 O(min(i, n-i))이고, 맨 앞(`i = 0`)도 O(1)입니다.
- 문서는 비용이 같으면 대체로 `Vec`이 `VecDeque`보다, `VecDeque`가 `LinkedList`보다 빠르다고 덧붙입니다.

위 표와 각 타입의 문서를 바탕으로, 자주 하는 일을 컬렉션별로 정리하면 다음과 같습니다.

| 하고 싶은 일 | `Vec` | `VecDeque` | `HashMap`/`HashSet` | `BTreeMap`/`BTreeSet` | `BinaryHeap` |
| --- | --- | --- | --- | --- | --- |
| 끝에 넣기·빼기 | O(1)* / O(1) | O(1)* / O(1) | | | |
| 앞에 넣기·빼기 | O(n) | O(1)* / O(1) | | | |
| 값(키)으로 찾기 | O(n), 정렬했다면 O(log n) | O(n) | O(1)~ | O(log n) | |
| 가장 작은/큰 값 | O(n), 정렬했다면 O(1) | O(n) | O(n) | O(log n) | 큰 값 보기 O(1), 꺼내기 O(log n) |
| 넣기 | | | O(1)~* | O(log n) | O(1)~ (`push`) |
| 정렬된 순서로 순회 | 정렬 O(n log n) 뒤 | 정렬 뒤 | 모아서 정렬해야 함 | 바로 가능 | `into_sorted_vec` O(n log n) |

O 표기는 요소 수가 커질 때 시간이 늘어나는 **경향**입니다. 요소가 몇십 개뿐이면 어느 것을 골라도 차이가 거의 없고, 요소가 메모리에 이어져 있는 `Vec`이 의외로 빠른 경우도 많습니다. 비용 표는 요소가 많아질 수 있는 곳에서 "이대로 두면 n²이 되는가"를 판단할 때 씁니다.

### 2.3 선택 기준: 질문으로 고르기

모듈 문서의 "When Should You Use Which Collection?" 절을 질문 순서로 바꾸면 다음과 같습니다.

1. **키로 값을 찾는가?** → 맵입니다. 키의 **순서**가 필요하면(정렬된 순회, 범위 조회, 가장 작은/큰 키, "x보다 작은 것 중 가장 큰 키") `BTreeMap`, 아니면 `HashMap`입니다.
2. **값이 있는지 없는지만 알면 되는가?** (짝지을 값이 없다) → 집합입니다. 순서 기준은 1과 같습니다.
3. **언제나 가장 크거나 가장 중요한 것만 꺼내는가?** → `BinaryHeap`입니다.
4. **순서 있는 나열인가?** → 끝(또는 끝 근처)에서만 넣고 빼면 `Vec`(스택), 앞에서도 꺼내면 `VecDeque`(큐)입니다.
5. 그 밖에는 `Vec`입니다. 일단 모아 두었다가 나중에 처리하는 경우, 크기가 바뀌는 배열이 필요한 경우가 여기에 듭니다.

| 상황 | 고를 것 | 다른 것을 쓰면 곤란한 이유 | 예제 |
| --- | --- | --- | --- |
| 일단 모아 두고 차례로 처리, 스택 | `Vec` | (기본값) | 01 |
| 줄 서기(먼저 온 것 먼저), 최근 N개 유지 | `VecDeque` | `Vec::remove(0)`은 매번 나머지를 모두 당기므로 n번 꺼내면 O(n²) | 05 |
| 날짜·점수처럼 순서 있는 키, 범위 조회, 최소·최대 | `BTreeMap` | `HashMap`은 순서가 없어서 매번 전부 꺼내 정렬하거나 다 훑어야 함 | 06 |
| 키 → 값 찾기, 개수 세기, 캐시 | `HashMap` | (맵의 기본값) | 04 |
| "본 적 있나?", 중복 제거, 공통·차이 구하기 | `HashSet` / `BTreeSet` | `Vec::contains`는 O(n)이라 n번 물으면 O(n²) | 07 |
| 우선순위 높은 작업부터, 가장 이른 이벤트부터, 상위 k개 | `BinaryHeap` | 넣을 때마다 정렬하면 O(n log n), 꺼낼 때마다 최댓값을 찾으면 O(n) | 08 |
| 한 번 만들고 여러 번 찾는 정렬된 목록 | 정렬된 `Vec` + `binary_search` | `BTreeSet`도 되지만, 삽입이 드물면 `Vec`이 메모리를 덜 쓰고 단순함 | 03 |

### 2.4 `LinkedList`를 거의 쓰지 않는 이유

다른 언어의 자료 구조 수업에서는 연결 리스트가 "중간 삽입·삭제가 O(1)"이라고 배웁니다. 그런데 Rust의 `LinkedList` 문서는 이렇게 적습니다. "배열 기반 컨테이너가 대체로 더 빠르고, 메모리를 덜 쓰고, CPU 캐시를 더 잘 쓰기 때문에 거의 항상 `Vec`이나 `VecDeque`를 쓰는 편이 낫습니다."

- **노드마다 따로 할당합니다.** 요소 하나마다 앞뒤 포인터 두 개가 붙고, 노드가 메모리 여기저기에 흩어집니다. 순회할 때마다 포인터를 따라가야 해서 CPU 캐시가 잘 맞지 않습니다. `Vec`은 요소가 이어져 있어 한 번 읽어 온 캐시 줄에 다음 요소들이 함께 들어 있습니다.
- **"중간 삽입 O(1)"은 그 위치를 이미 쥐고 있을 때 이야기입니다.** 위치를 찾는 데 O(min(i, n-i))가 듭니다(비용 표의 `get(i)`). 위치를 쥐고 있게 해 주는 커서 API(`cursor_front_mut` 등)는 이 툴체인에서 아직 nightly 전용 실험 기능입니다.
- 모듈 문서가 꼽는 쓸 만한 경우는 "리스트를 효율적으로 나누고 이어 붙여야 할 때"(이어 붙이기 `append`가 O(1)), "분할 상환을 견딜 수 없어 가끔 생기는 O(n) 재할당도 안 될 때", "정말로, 진짜로 이중 연결 리스트가 필요하다고 확신할 때"뿐입니다.

### 2.5 `Vec`: 중간 삽입·삭제와 비용

`Vec`은 요소를 빈틈없이 이어 붙여 두므로, 중간에서 넣거나 빼면 그 뒤의 요소가 모두 움직입니다.

```
remove(1) 전:  [ 가 | 나 | 다 | 라 | 마 ]
remove(1) 후:  [ 가 | 다 | 라 | 마 ]        ← 다, 라, 마가 한 칸씩 당겨짐 (O(n-i))

swap_remove(1) 전:  [ 가 | 나 | 다 | 라 | 마 ]
swap_remove(1) 후:  [ 가 | 마 | 다 | 라 ]   ← 마지막 요소 하나만 옮김 (O(1))
```

| 메서드 | 하는 일 | 비용 | 순서 유지 | 패닉 조건 |
| --- | --- | --- | --- | --- |
| `push(x)` / `pop()` | 끝에 넣기 / 끝에서 꺼내기(`Option<T>`) | O(1)* / O(1) | 예 | 없음 |
| `insert(i, x)` | `i`번에 넣고 뒤를 민다 | O(n-i) | 예 | `i > len` |
| `remove(i)` | `i`번을 꺼내고(`T`) 뒤를 당긴다 | O(n-i) | 예 | `i >= len` |
| `swap_remove(i)` | `i`번을 꺼내고 그 자리에 마지막 요소를 옮긴다 | O(1) | **아니요** | `i >= len` |

`remove(0)`을 반복하는 코드는 n번 × O(n) = O(n²)입니다. 앞에서 꺼내는 일이 잦다면 `VecDeque`(2.12절)를, 순서가 상관없다면 `swap_remove`를 씁니다. 예제는 [`01_vec_edit.rs`](examples/01_vec_edit.rs)의 1, 2번 구역입니다.

### 2.6 `Vec`: 여러 요소를 한꺼번에 다루기

요소를 하나씩 `remove`하는 대신 한 번에 처리하는 메서드를 쓰면 코드도 짧고 요소를 옮기는 횟수도 줄어듭니다.

| 메서드 | 하는 일 | 지운 요소는 |
| --- | --- | --- |
| `extend(모음)` | 배열·반복자·다른 컬렉션의 요소를 끝에 붙임 | |
| `append(&mut other)` | `other`의 요소를 모두 옮겨 붙이고 `other`를 비움 | |
| `drain(범위)` | 범위를 잘라 내고, 잘라 낸 요소를 하나씩 내놓는 반복자를 돌려줌 | 돌려받음 |
| `retain(조건)` | 조건이 `true`인 요소만 남김. 한 번씩만 보고, 남은 요소의 순서 유지 | drop |
| `extract_if(범위, 조건)` | 조건이 `true`인 요소를 빼내 반복자로 돌려줌 | 돌려받음 |
| `dedup()` | **연속된** 같은 값을 하나만 남김 | drop |
| `truncate(n)` | 앞의 `n`개만 남김 | drop |
| `split_off(at)` | `at`번부터 끝까지를 새 `Vec`으로 떼어 돌려줌 | 새 `Vec`으로 |
| `clear()` | 모두 지움 | drop |

몇 가지 짚어 둡니다.

- **`for` 문 안에서 `remove(i)`를 부르며 지우지 않습니다.** 지울 때마다 뒤가 당겨져 인덱스가 어긋나 요소를 건너뛰기 쉽고, 지울 때마다 O(n)이 듭니다. 조건으로 지울 때는 `retain`이 정답입니다. 지운 요소도 필요하면 `extract_if`(1.87부터 안정화)를 씁니다.
- **`dedup`은 이웃한 중복만 지웁니다.** `[3, 3, 5, 3]`은 `[3, 5, 3]`이 됩니다. 모든 중복을 지우려면 먼저 정렬해서 같은 값끼리 붙여 놓습니다. 처음 나온 순서를 지키며 지우는 방법은 2.14절에 있습니다.
- `drain`, `extract_if`가 돌려주는 반복자는 `collect()`로 `Vec`에 모으거나 `for` 문에 넘깁니다. 반복자 자체는 `ch5-iterators`에서 자세히 다룹니다.
- 벡터 전체의 소유권을 꺼내면서 빈 벡터를 남기고 싶을 때는 `std::mem::take`를 쓰는데, 이것은 `ch7-smart-pointers-memory`에서 다룹니다.

### 2.7 `Vec`: 용량 관리

[Part 0 ch8](../../part0-getting-started/ch8-collections/README.md)에서 길이(len)와 용량(capacity)의 차이, 용량이 꽉 차면 재할당이 일어난다는 것을 보았습니다. 요소가 몇 개 들어올지 알면 재할당을 미리 막을 수 있습니다.

| 메서드 | 하는 일 |
| --- | --- |
| `Vec::with_capacity(n)` | 처음부터 최소 `n`개 분량을 확보한 빈 벡터를 만듦 |
| `reserve(k)` | 앞으로 `k`개를 더 넣어도 재할당이 없도록 확보 (조금 더 확보할 수 있음) |
| `reserve_exact(k)` | `reserve`와 같지만 일부러 더 확보하지는 않음 |
| `shrink_to_fit()` | 남는 용량을 가능한 한 돌려줌 |

`Vec` 문서의 "Guarantees" 절이 보장하는 것과 보장하지 않는 것을 구분해 두면 좋습니다.

- **보장함**: 용량이 충분하면 `push`와 `insert`는 절대 재할당하지 않습니다. `Vec::with_capacity(n)`, `vec![x; n]`은 정확히 `n`개 분량을 요청합니다(할당기가 더 줄 수는 있음). `Vec`은 비어도 저절로 줄어들지 않으므로 `clear`나 `truncate` 뒤에도 용량은 그대로입니다. 같은 벡터를 비웠다가 다시 채우는 일을 반복할 때 할당이 일어나지 않게 하려는 것입니다.
- **보장하지 않음**: 꽉 찼을 때 얼마나 늘리는지(증가 전략)는 정해져 있지 않습니다. 보장하는 것은 `push`가 분할 상환 O(1)이라는 것뿐입니다.

[`01_vec_edit.rs`](examples/01_vec_edit.rs)의 8번 구역에서 `with_capacity(10)` → 10개 넣기 → `reserve(100)` → `clear` → `shrink_to_fit`의 용량 변화를 봅니다. 출력의 용량 값은 이 툴체인에서의 결과이며, 보장된 성질("110 이상인가?")은 `true`/`false`로 출력합니다.

### 2.8 정렬

정렬 메서드는 `Vec`이 아니라 **슬라이스 `[T]`** 에 정의되어 있습니다. `Vec<T>`는 `[T]`로 역참조되므로 `v.sort()`처럼 부를 수 있고, 배열이나 `&mut [T]`에도 그대로 쓸 수 있습니다. `Vec` 문서에서는 "Methods from `Deref<Target = [T]>`" 아래에 있습니다(이 문서 읽는 법은 `ch1-std-overview`).

| 메서드 | 비교 기준 | 안정 정렬 | 요구 |
| --- | --- | --- | --- |
| `sort()` | 요소의 기본 순서 | 예 | `T: Ord` |
| `sort_unstable()` | 요소의 기본 순서 | 아니요 | `T: Ord` |
| `sort_by(\|a, b\| ...)` | 두 요소를 비교해 `Ordering`을 돌려주는 클로저 | 예 | 클로저 |
| `sort_by_key(\|x\| 키)` | 요소마다 뽑은 키 | 예 | 키: `Ord` |
| `sort_by_cached_key(\|x\| 키)` | 요소마다 **한 번만** 뽑은 키 | 예 | 키: `Ord` |
| `sort_unstable_by`, `sort_unstable_by_key` | 위와 같음 | 아니요 | |

**안정 정렬(stable sort)** 은 비교 결과가 같은 요소들의 원래 순서를 지키는 정렬입니다. **불안정 정렬(unstable sort)** 은 그 순서를 보장하지 않는 대신 보통 더 빠르고 추가 메모리를 쓰지 않습니다. 문서도 "가능하면 불안정 정렬이 낫다"고 권합니다.

언제 차이가 날까요? 정수처럼 같으면 구별할 수 없는 값은 결과가 똑같으므로 `sort_unstable`을 써도 됩니다. 차이는 **키만 같고 다른 정보는 다른 요소**를 정렬할 때 생깁니다. 안정 정렬에는 "보조 기준으로 먼저 정렬한 뒤 주 기준으로 다시 정렬하면 두 기준이 함께 적용된다"는 쓸모가 있습니다.

```rust
students.sort_by_key(|s| s.name);  // 보조 기준: 이름순
students.sort_by_key(|s| s.class); // 주 기준: 반. 같은 반 안에서는 이름순이 유지됩니다.
```

예제 02의 3번 구역은 주문 50건을 지역별로 정렬한 뒤 같은 지역 안에서 주문 번호 순서가 유지되는지 확인합니다. 이 툴체인에서 안정 정렬은 세 지역 모두 유지됐고, 불안정 정렬은 두 지역에서 순서가 섞였습니다. 요소가 적으면 불안정 정렬도 우연히 순서가 유지되곤 해서, 짧은 예시로 시험해 보고 "괜찮다"고 믿으면 안 됩니다. 핵심은 "보장하지 않는다"입니다.

**`sort_by`와 `sort_by_key`, `Reverse`**: 두 기준을 한 번에 적용할 수도 있습니다. `sort_by`에서는 `then_with`로 비교를 잇고([Part 0 ch13](../../part0-getting-started/ch13-closures-iterators/README.md)의 `then`과 같지만 뒤의 비교를 필요할 때만 계산), `sort_by_key`에서는 튜플 키를 씁니다. 튜플은 앞 요소부터 비교하고(사전식), `std::cmp::Reverse(x)`는 `x`의 비교 결과를 뒤집은 값을 만듭니다.

```rust
// 점수 내림차순, 같으면 이름 오름차순 — 두 줄은 같은 결과입니다.
students.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(b.name)));
students.sort_by_key(|s| (Reverse(s.score), s.name));
```

`sort_by_key`의 클로저는 요소를 빌린 참조를 키로 돌려줄 수 없습니다. 이름이 `String` 필드라서 `v.sort_by_key(|s| &s.name)`처럼 쓰면 E 코드 없는 에러 `lifetime may not live long enough`가 납니다. 이때는 `sort_by(|a, b| a.name.cmp(&b.name))`를 씁니다.

**`sort_by_cached_key`**: `sort_by_key`는 비교할 때마다 키 함수를 다시 부릅니다. 키가 `to_lowercase()`처럼 새 `String`을 만드는 계산이면 그만큼 할당이 되풀이됩니다. `sort_by_cached_key`는 요소마다 키를 한 번만 계산해 임시 저장소에 두고 정렬합니다. 예제 02의 5번 구역에서 단어 8개를 대소문자 구분 없이 정렬하면 키 함수 호출이 `sort_by_key`는 34번(이 툴체인에서의 결과), `sort_by_cached_key`는 8번이었습니다. 반대로 필드 하나를 읽는 정도의 가벼운 키라면 문서는 `sort_by_key`가 더 빠를 것이라고 안내합니다.

**실수(`f64`) 정렬**: `f64`에 `sort()`를 쓰면 컴파일되지 않습니다(4.1절). `sort`는 `T: Ord`, 즉 **전순서(total order)** 를 요구합니다. 전순서는 어떤 두 값이든 크다·같다·작다 중 하나로 정해지는 순서인데, `f64`의 `NaN`은 자기 자신과도 같지 않아서 이 조건을 깹니다. 그래서 `f64`는 "일부만 비교할 수 있다"는 `PartialOrd`만 구현합니다. 해결책은 둘입니다.

| 방법 | 동작 | 언제 |
| --- | --- | --- |
| `sort_by(\|a, b\| a.total_cmp(b))` | IEEE 754의 totalOrder 규칙으로 모든 값에 순서를 매김. `-0.0 < 0.0`이고, 양의 `NaN`은 무한대보다도 뒤로 감 | 기본으로 권장 |
| `sort_by(\|a, b\| a.partial_cmp(b).unwrap())` | `NaN`을 만나면 `partial_cmp`가 `None`을 돌려줘 **패닉** | `NaN`이 없다고 확신할 때 |

내림차순은 `b.total_cmp(a)`처럼 자리를 바꿉니다. 예제는 [`02_sort.rs`](examples/02_sort.rs)입니다.

### 2.9 이진 탐색

`contains`는 앞에서부터 하나씩 비교하므로 O(n)입니다. 슬라이스가 **정렬되어 있다면** 가운데 값과 비교해 절반씩 버려 나가는 **이진 탐색(binary search)** 으로 O(log n)에 찾을 수 있습니다. 요소가 100만 개여도 비교는 20번 정도입니다.

`binary_search(&x)`는 `Result<usize, usize>`를 돌려줍니다. 에러 처리용 `Result`를 "두 가지 경우를 나누는 값"으로 쓴 것입니다.

| 결과 | 뜻 |
| --- | --- |
| `Ok(i)` | 찾았습니다. `i`번에 있습니다. 같은 값이 여러 개면 그중 **아무거나** 하나입니다. |
| `Err(i)` | 없습니다. 하지만 `i`번에 넣으면 정렬 상태가 유지됩니다(삽입 위치). |

`[10, 20, 30, 40, 50]`에서 30을 찾으면 `Ok(2)`, 35를 찾으면 30과 40 사이를 뜻하는 `Err(3)`입니다.

**정렬 상태를 유지하며 삽입하기**: `Err(pos)`의 `pos`가 바로 넣을 자리입니다.

```rust
match booked.binary_search(&hour) {
    Ok(_) => println!("{hour}시: 이미 예약됨"),
    Err(pos) => booked.insert(pos, hour),
}
```

찾기는 O(log n)이지만 `insert`가 뒤를 밀어야 하므로 전체는 O(n)입니다. 삽입이 잦다면 `BTreeSet`/`BTreeMap`(O(log n))이 맞고, 한 번 만들어 두고 여러 번 찾는 목록이라면 정렬된 `Vec`이 메모리도 덜 쓰고 단순합니다.

**`partition_point(조건)`**: 슬라이스가 "조건이 `true`인 앞부분"과 "`false`인 뒷부분"으로 나뉘어 있을 때 그 경계의 인덱스를 이진 탐색으로 찾습니다. 정렬된 슬라이스라면 다음과 같이 씁니다.

| 알고 싶은 것 | 코드 (`scores = [55, 70, 80, 80, 80, 95]`) | 결과 |
| --- | --- | --- |
| 첫 80의 위치 (80 미만의 개수) | `scores.partition_point(\|&s\| s < 80)` | `2` |
| 마지막 80 다음 위치 (80 이하의 개수) | `scores.partition_point(\|&s\| s <= 80)` | `5` |
| 80이 몇 개인가 | 두 값의 차 | `3` |
| 중복을 허용하며 80을 넣을 자리 | `partition_point(\|&s\| s <= 80)` | `5` (같은 값들의 맨 뒤) |

`binary_search(&80)`은 셋 중 어느 80을 가리킬지 정해져 있지 않으므로(이 툴체인에서는 `Ok(4)`), 범위가 필요하면 `partition_point`를 씁니다.

구조체 목록은 키를 뽑는 클로저를 받는 `binary_search_by_key(&키, |요소| 키)`, 직접 비교하는 `binary_search_by(|요소| 요소의_키.cmp(&목표))`로 찾습니다. 두 메서드 모두 슬라이스가 그 키 순서로 정렬되어 있어야 합니다.

**정렬되지 않은 슬라이스에 쓰면 결과는 의미가 없습니다.** 예제 03의 5번 구역에서 `[5, 1, 4, 2, 3]`에 `binary_search(&5)`를 하면 5가 분명히 들어 있는데도 `Err(5)`가 나옵니다. 패닉도 경고도 없으니, 정렬했는지 확신이 없으면 `is_sorted()`로 확인하세요. 예제는 [`03_binary_search.rs`](examples/03_binary_search.rs)입니다.

### 2.10 entry API

Part 0 ch8에서 `*map.entry(key).or_insert(0) += 1`로 단어 빈도를 셌습니다. entry API가 왜 있는지부터 봅니다. 다음은 entry 없이 같은 일을 하는 코드입니다.

```rust
if !counts.contains_key(menu) {   // 탐색 1
    counts.insert(menu, 0);       // 탐색 2 (없을 때만)
}
*counts.get_mut(menu).unwrap() += 1; // 탐색 3
```

동작은 올바르지만 같은 키를 두세 번 찾습니다. 해시 맵이라면 키의 해시를 매번 다시 계산하고, `BTreeMap`이라면 트리를 매번 다시 내려갑니다. 모듈 문서는 entry API의 목적을 "찾은 다음 넣느라 탐색을 두 번 하는 일"을 없애는 것이라고 설명합니다. `map.entry(key)`는 키를 **한 번** 찾고, 찾은 자리를 `Entry` 값으로 돌려줍니다. 그 자리에서 읽고, 넣고, 고칩니다.

`Entry`는 열거형입니다(`std::collections::hash_map::Entry`의 정의에서 할당기 매개변수를 뺐습니다).

```rust
pub enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>), // 키가 있음
    Vacant(VacantEntry<'a, K, V>),     // 키가 없음
}
```

대부분은 `Entry`의 메서드로 충분합니다.

| 메서드 | 키가 없을 때 | 키가 있을 때 | 돌려주는 것 |
| --- | --- | --- | --- |
| `or_insert(값)` | 값을 넣음 | 그대로 | `&mut V` |
| `or_insert_with(\|\| 값)` | 클로저를 불러 넣음 | 그대로(클로저를 부르지 않음) | `&mut V` |
| `or_insert_with_key(\|k\| 값)` | 키를 보고 만든 값을 넣음 | 그대로 | `&mut V` |
| `or_default()` | `V::default()`(0, 빈 `String`, 빈 `Vec` 등)를 넣음 | 그대로 | `&mut V` |
| `and_modify(\|v\| ...)` | 아무것도 안 함 | 클로저로 값을 고침 | `Entry` (뒤에 `or_insert` 등을 이어 씀) |

예를 들어 `teams.entry(team).or_default().push(name)`은 처음 보는 팀이면 빈 `Vec`을 넣고 그 `Vec`에 바로 `push`하고, `best.entry(name).and_modify(|b| *b = (*b).max(score)).or_insert(score)`는 기록이 있으면 더 큰 값으로 고치고 없으면 첫 기록을 넣습니다.

**`or_insert(값)`과 `or_insert_with(클로저)`의 차이**: 메서드의 인자는 메서드를 부르기 전에 계산됩니다. 그래서 `or_insert(load_profile("민수"))`는 키가 이미 있어서 값이 쓰이지 않아도 `load_profile`을 부릅니다. 기본값을 만드는 데 비용이 들면 `or_insert_with(|| load_profile("민수"))`로 "필요할 때만" 만들게 합니다. clippy에도 이 모양을 잡는 `or_fun_call` lint가 있지만 기본으로는 꺼져 있어서, 직접 켜야(`-W clippy::or_fun_call`) 경고가 납니다. `ch2-option-result`의 `unwrap_or`와 `unwrap_or_else`의 관계와 같습니다.

**`Entry`를 `match`하기**: "있을 때"와 "없을 때"에 하는 일이 크게 다르면 직접 `match`합니다. `OccupiedEntry`에서는 `get`, `get_mut`, `insert`(값 바꾸기), `remove`(항목 지우기)를, `VacantEntry`에서는 `insert`와 `key`를 쓸 수 있습니다. 예제 04의 재고 출고 함수는 수량이 0이 되면 `entry.remove()`로 항목을 지웁니다. 키를 다시 찾지 않습니다.

```rust
use std::collections::hash_map::{self, HashMap};

match stock.entry(item) {
    hash_map::Entry::Occupied(mut entry) if *entry.get() > qty => *entry.get_mut() -= qty,
    hash_map::Entry::Occupied(entry) => { entry.remove(); }      // 재고 부족 확인은 생략
    hash_map::Entry::Vacant(entry) => println!("{}: 없는 상품", entry.key()),
}
```

- `BTreeMap`에도 똑같은 entry API가 있습니다. 열거형은 `std::collections::btree_map::Entry`입니다(예제 04의 3, 4번 구역).
- `entry`는 키를 **값으로** 받습니다. 키가 `String`이면 이미 있는 키여도 `entry(word.to_string())`처럼 `String`을 만들어 넘겨야 합니다. 대부분의 키가 이미 있을 것으로 예상되면 `get_mut`으로 먼저 찾아보고 없을 때만 `insert`하는 편이 할당을 줄입니다.
- `or_insert` 등이 돌려준 `&mut V`가 살아 있는 동안 맵은 가변으로 빌려진 상태입니다. 그 참조를 쥔 채 같은 맵의 다른 키를 고치려 하면 대여 에러가 납니다(4.5절).

예제는 [`04_entry_api.rs`](examples/04_entry_api.rs)입니다.

### 2.11 키가 될 수 있는 조건

직접 만든 구조체를 `HashMap`의 키로 넣으려 하면 컴파일러가 거부합니다(4.3절). 어떤 조건이 필요한지는 API 문서의 `impl` 블록 머리에 적혀 있습니다. `HashMap` 문서를 내려가 보면 메서드들이 여러 `impl` 블록으로 나뉘어 있습니다.

```rust
impl<K, V> HashMap<K, V, RandomState>          // new, with_capacity
// ...
impl<K, V, S, A> HashMap<K, V, S, A>           // len, iter, keys, values, retain, clear ...
// ...
impl<K, V, S, A> HashMap<K, V, S, A>
where
    K: Eq + Hash,                              // ← 키에 대한 요구
    S: BuildHasher,
    A: Allocator,
// insert, get, get_mut, entry, remove, contains_key, reserve ...
```

- `S`는 해시 함수를 만드는 타입(기본값 `RandomState`), `A`는 메모리 할당기 자리입니다. 사용자 지정 할당기는 아직 불안정 기능이라 기본값 그대로 두면 됩니다.
- 첫 블록에는 키 조건이 없으므로 `HashMap::new()`는 어떤 키 타입으로도 만들어집니다. 하지만 `insert`, `get`, `entry`는 `K: Eq + Hash` 블록 안에 있어서 키가 조건을 채우지 못하면 부를 수 없습니다. 그래서 에러가 `new`가 아니라 `insert`에서 납니다.

`BTreeMap`은 바운드를 메서드마다 `where K: Ord`로 붙여 둡니다(`insert`, `get`, `entry`, `range`, `first_key_value` 등). `BinaryHeap`은 `push`, `pop`, `peek_mut`, `into_sorted_vec`이 `T: Ord` 블록에 있고, `peek`, `len`, `iter`는 바운드 없는 블록에 있습니다. 모양은 달라도 읽는 법은 같습니다. **메서드 시그니처 근처의 `where`와 그 메서드가 속한 `impl` 블록의 `where`를 함께 봅니다.**

| 컬렉션 | 키·원소에 필요한 것 | 이유 |
| --- | --- | --- |
| `HashMap<K, V>`, `HashSet<T>` | `Eq + Hash` | 해시로 자리를 찾고, 같은 자리에 있는 키들을 `==`로 가려냄 |
| `BTreeMap<K, V>`, `BTreeSet<T>` | `Ord` | 트리 안에서 키를 크기 순으로 배치 |
| `BinaryHeap<T>` | `Ord` | "가장 큰 것"을 정하려면 언제나 비교할 수 있어야 함 |
| `Vec<T>`의 `sort`, `binary_search` | `Ord` | 같은 이유 |
| `Vec<T>`의 `contains`, `dedup` | `PartialEq` | `==`만 있으면 됨 |

정수, `bool`, `char`, `String`, `&str`, 그리고 이런 타입으로 이뤄진 튜플·배열·`Vec`은 `Eq`, `Hash`, `Ord`를 모두 구현합니다. `f32`/`f64`는 셋 다 구현하지 않습니다. `NaN`이 자기 자신과도 같지 않아 `Eq`와 `Ord`의 조건을 깨기 때문입니다. 실수를 키로 쓰고 싶다면 0.1도 단위 정수(`365`는 36.5도)처럼 정수로 바꾸는 방법을 먼저 생각해 보세요.

**직접 만든 타입은 `derive`로 해결합니다.** 모든 필드가 그 트레이트를 구현하면 `derive`로 붙일 수 있습니다.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]      // HashMap/HashSet의 키
struct Cell { x: i32, y: i32 }

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]        // BTreeMap 키, BinaryHeap 원소
struct Version { major: u32, minor: u32, patch: u32 }
```

- `Eq`는 `PartialEq`를, `Ord`는 `Eq`와 `PartialOrd`를 전제로 하므로 함께 적습니다. 컴파일러의 `help:`도 `#[derive(Eq, Hash, PartialEq)]`처럼 필요한 것을 모아 제안합니다.
- **`derive(PartialOrd, Ord)`는 필드를 선언한 순서대로 비교합니다.** 첫 필드가 같을 때만 다음 필드를 봅니다. `Version`은 `major` → `minor` → `patch` 순서라서 버전 번호의 순서와 맞습니다(문자열 `"1.10.0"`과 `"1.9.1"`을 비교하면 거꾸로 됩니다). 예제 08의 `Job`처럼 필드 순서로 우선순위를 정할 수도 있습니다. 열거형에 `derive(Ord)`를 붙이면 먼저 선언한 배리언트가 작습니다.
- 이 트레이트들을 손으로 구현하는 방법과 지켜야 할 규칙("`a == b`이면 `hash(a) == hash(b)`" 등)은 `ch6-core-traits`에서 다룹니다. 맵에 들어 있는 동안 키의 해시나 순서가 바뀌게 만들면 안 된다는 것도 문서가 강조하는 규칙입니다.

**해시 함수와 순회 순서**: `HashMap`의 기본 해시 함수는 현재 SipHash 1-3입니다. 공격자가 같은 해시를 내는 키를 대량으로 보내 해시 맵을 극도로 느리게 만드는 공격(HashDoS)을 막도록 설계되었고, 맵을 만들 때마다 무작위 시드로 초기화됩니다. 그래서 `HashMap`과 `HashSet`의 순회 순서는 실행할 때마다 다르고, 같은 프로그램 안에서 같은 내용을 넣은 두 집합끼리도 다를 수 있습니다(이 툴체인에서 실제로 확인했습니다). 출력이나 테스트에서 순서가 필요하면 정렬하거나 `BTreeMap`/`BTreeSet`에 모읍니다. 이 챕터의 예제가 모두 그렇게 합니다. SipHash는 정수 같은 작은 키에서는 가장 빠른 해시 함수가 아니어서, 성능이 중요하면 `with_hasher`로 다른 해시 함수(crates.io에 여럿 있음)를 끼울 수 있지만, 대신 HashDoS 방어를 잃을 수 있습니다.

덧붙여, `HashMap<String, V>`에서 `map.get("청팀")`처럼 `&str`로 찾을 수 있는 것은 `get`의 시그니처가 `K: Borrow<Q>`를 요구하고 `String`이 `Borrow<str>`를 구현하기 때문입니다. `Borrow`는 `ch6-core-traits`에서 다룹니다.

### 2.12 `VecDeque`: 양쪽 끝이 모두 빠른 큐

`VecDeque`는 고정된 크기의 배열을 고리처럼 이어 쓰는 **링 버퍼(ring buffer)** 입니다. 앞에서 꺼내면 "시작 위치"만 한 칸 옮기고, 배열 끝에 닿으면 처음으로 넘어가 이어 씁니다. 그래서 앞뒤 어느 쪽에서 넣고 빼도 다른 요소를 옮기지 않습니다.

```
용량 4인 VecDeque에 10, 20, 30을 넣고 앞에서 둘을 뺀 뒤 40, 50, 60을 넣으면
(예제 05의 4번 구역, 메모리 배치는 이 툴체인에서의 결과)

메모리:   [ 50 | 60 | 30 | 40 ]
                     ↑ 논리적인 맨 앞(0번)
논리 순서: 30, 40, 50, 60      as_slices() = ([30, 40], [50, 60])
```

| 메서드 | 하는 일 |
| --- | --- |
| `push_back(x)` / `pop_front()` | 큐로 쓸 때: 뒤에 넣고 앞에서 꺼냄 |
| `push_front(x)` / `pop_back()` | 반대쪽 끝 |
| `front()` / `back()` | 양 끝을 꺼내지 않고 봄 (`Option<&T>`) |
| `dq[i]`, `get(i)` | 논리적인 앞에서부터 센 인덱스로 접근. O(1) |
| `rotate_left(k)` | 앞의 `k`개를 뒤로 보냄 (당번 순환 등) |
| `as_slices()` | 메모리상의 두 조각을 슬라이스 두 개로 |
| `make_contiguous()` | 요소를 한 조각으로 모아 `&mut [T]`를 돌려줌 |

요소가 두 조각으로 나뉠 수 있으므로 `VecDeque`는 슬라이스로 역참조되지 않고, 슬라이스 메서드인 `sort`가 없습니다(4.2절). 정렬하려면 `dq.make_contiguous().sort()`처럼 한 조각으로 모은 뒤 정렬합니다. `binary_search`, `partition_point`, `contains`는 `VecDeque`에도 직접 정의되어 있습니다.

**"최근 N개 유지"** 도 대표적인 쓰임입니다. 뒤에 넣고 길이가 N을 넘으면 앞에서 버립니다. 이동 평균, 최근 본 상품, 최근 로그 몇 줄 같은 곳에 씁니다.

**이것이 아니면 곤란한 상황**: 예제 05의 5번 구역은 요소 n개를 앞에서부터 모두 꺼내는 시간을 `Vec::remove(0)`과 `VecDeque::pop_front`로 잽니다. 이 문서를 쓸 때 실행한 결과(3절)에서 n이 25000에서 50000으로 2배가 되자 `Vec` 쪽은 약 4배(O(n)짜리를 n번 하므로 O(n²)), `VecDeque` 쪽은 약 2배(O(n))가 되었고, 두 쪽의 차이는 수백 배였습니다. 요소가 늘어날수록 격차는 계속 벌어집니다.

### 2.13 `BTreeMap`과 `BTreeSet`: 순서가 있는 맵과 집합

`BTreeMap`은 키를 정렬된 상태로 B-트리에 저장합니다. 찾기·넣기·지우기가 O(log n)이라 `HashMap`의 O(1)~보다 느릴 수 있지만, 대신 **키의 순서를 이용하는 질문**에 바로 답합니다.

| 하고 싶은 일 | 코드 |
| --- | --- |
| 키 순서로 순회 | `for (k, v) in &map` |
| 범위 조회 | `map.range("2026-11-10"..="2026-11-21")`, `map.range(..x)`, `map.range(x..)` |
| 가장 작은/큰 키 | `first_key_value()`, `last_key_value()` (`Option<(&K, &V)>`) |
| 가장 작은/큰 항목 꺼내기 | `pop_first()`, `pop_last()` |
| x **이하**에서 가장 큰 키 | `map.range(..=x).next_back()` |
| x **이상**에서 가장 작은 키 | `map.range(x..).next()` |
| 키 하나를 기준으로 둘로 나누기 | `split_off(&key)` |

`range`는 범위 안의 항목을 오름차순으로 내놓는 반복자를 돌려줍니다. `next()`는 맨 앞 하나를, `next_back()`은 맨 뒤 하나를 꺼냅니다(`next_back`은 뒤에서도 꺼낼 수 있는 반복자의 메서드이며 `ch5-iterators`에서 다룹니다). 범위 문법은 [Part 0 ch3](../../part0-getting-started/ch3-common-concepts/README.md)의 `a..b`, `a..=b`를 그대로 씁니다. 시작이 끝보다 크면 패닉합니다(4.6절).

"x 이하에서 가장 큰 키"는 생각보다 자주 나옵니다. 예제 06은 점수 구간표 `{0: 'F', 60: 'D', 70: 'C', 80: 'B', 90: 'A'}`에서 `range(..=79).next_back()`으로 79점의 학점 `'C'`를 찾습니다. `HashMap`이었다면 모든 기준점을 훑어야 합니다. `BTreeSet`에도 같은 `range`, `first`, `last`, `pop_first`, `pop_last`가 있어서, 버스 출발 시각 집합에서 `range(now..).next()`로 "지금 이후 첫 버스"를 찾습니다.

날짜를 `"2026-11-03"`처럼 "연-월-일"로 자릿수를 맞춘 문자열로 쓰면 문자열 순서와 날짜 순서가 같아져서 키로 그대로 쓸 수 있습니다. 직접 만든 타입을 키로 쓸 때는 2.11절의 `derive(Ord)`와 필드 순서를 확인합니다. 예제는 [`06_btreemap.rs`](examples/06_btreemap.rs)입니다.

### 2.14 `HashSet`과 `BTreeSet`: 집합 연산

집합은 "이 값이 있는가"만 기억하는 맵입니다. 같은 값은 하나만 들어갑니다.

- `insert(x)`는 **새로 넣었는지**를 `bool`로 돌려줍니다. 이미 있으면 `false`이고 집합은 그대로입니다. "처음 보는 값인가?"를 `contains`와 `insert` 두 번에 나눠 묻지 않아도 됩니다.
- `contains(&x)`, `remove(&x)`는 `HashSet`에서 평균 O(1), `BTreeSet`에서 O(log n)입니다.

이 성질로 **처음 나온 순서를 지키며 중복을 없앨** 수 있습니다. `sort` + `dedup`은 순서를 잃지만, `retain`의 클로저에서 `seen.insert`를 부르면 처음 보는 값만 남습니다.

```rust
let mut seen = HashSet::new();
tags.retain(|t| seen.insert(*t)); // ["rust", "web", "rust", "cli"] → ["rust", "web", "cli"]
```

**집합 연산**: 메서드는 결과를 하나씩 내놓는 반복자를 돌려주고, 연산자는 두 집합의 **참조**에 써서 새 집합을 만듭니다.

| 연산 | 메서드 | 연산자 | 결과 (`a = {가온, 나래, 다솜, 라온}`, `b = {다솜, 라온, 마루, 바다}`) |
| --- | --- | --- | --- |
| 교집합 (둘 다) | `a.intersection(&b)` | `&a & &b` | {다솜, 라온} |
| 합집합 (어느 쪽이든) | `a.union(&b)` | `&a \| &b` | {가온, 나래, 다솜, 라온, 마루, 바다} |
| 차집합 (`a`에만) | `a.difference(&b)` | `&a - &b` | {가온, 나래} |
| 대칭 차집합 (한쪽에만) | `a.symmetric_difference(&b)` | `&a ^ &b` | {가온, 나래, 마루, 바다} |
| 부분집합인가 | `a.is_subset(&b)` | | `bool` |
| 상위집합인가 | `a.is_superset(&b)` | | `bool` |
| 겹치는 게 없는가 | `a.is_disjoint(&b)` | | `bool` |

- 메서드가 내놓는 것은 원소의 참조(`&T`)입니다. 새 집합으로 모으려면 `a.intersection(&b).copied().collect()`처럼 씁니다. `copied()`는 참조를 값으로 복사해 내놓는 어댑터입니다(`Copy` 타입일 때. 아니면 `cloned()`). 어댑터는 `ch5-iterators`에서 자세히 다룹니다.
- 연산자는 원소를 복제해 새 집합을 만들므로 원소 타입이 `Clone`이어야 하고, 원래 두 집합은 그대로 남습니다.
- `HashSet`의 결과는 순서가 정해져 있지 않으므로 출력할 때는 정렬합니다. `BTreeSet`의 결과는 정렬된 순서로 나옵니다.

예제는 [`07_sets.rs`](examples/07_sets.rs)입니다. 6번 구역은 `derive(PartialEq, Eq, Hash)`한 `Cell`을 원소로 써서 로봇이 지나간 서로 다른 칸을 셉니다.

### 2.15 `BinaryHeap`: 우선순위 큐

**우선순위 큐**는 넣은 순서와 상관없이 "지금 가장 중요한 것"부터 꺼내는 큐입니다. `BinaryHeap`은 이것을 **이진 힙**으로 만듭니다. (메모리의 "힙 영역"과는 관계없는 자료 구조 이름입니다.) 이진 힙은 "부모가 자식보다 크거나 같다"는 조건만 지키는 이진 트리를 배열에 담은 것입니다.

```
push 순서: 3, 9, 1, 7, 5  →  내부 배열 [9, 7, 1, 3, 5]  (이 툴체인에서의 결과)

          9          ← 맨 위가 언제나 가장 큰 값 (peek: O(1))
        /   \
       7     1
      / \
     3   5           ← 7 > 3, 7 > 5, 9 > 1. 형제끼리는 순서가 없음
```

| 메서드 | 하는 일 | 비용 |
| --- | --- | --- |
| `push(x)` | 넣기 | 기대 O(1)~ (오름차순으로 계속 넣으면 O(log n)) |
| `pop()` | 가장 큰 값을 꺼냄 (`Option<T>`) | O(log n) |
| `peek()` | 가장 큰 값을 꺼내지 않고 봄 | O(1) |
| `into_sorted_vec()` | 모두 꺼내 오름차순 `Vec`으로 (`iter()`는 정렬되지 않은 내부 순서) | O(n log n) |

**최소 힙은 `Reverse`로**: `BinaryHeap`은 최대 힙입니다. 가장 작은 값부터 꺼내려면 `std::cmp::Reverse`로 감싸 비교를 뒤집습니다. 꺼낼 때는 패턴으로 벗깁니다.

```rust
events.push(Reverse((930, "회의")));
events.push(Reverse((815, "출근")));
while let Some(Reverse((time, what))) = events.pop() { /* 08:15 출근, 09:30 회의 순서 */ }
```

**같은 우선순위의 순서**: 힙은 같은 값들 사이의 순서를 보장하지 않습니다. "우선순위가 같으면 먼저 들어온 것부터"를 원하면 접수 번호를 비교 기준에 넣습니다. 예제 08의 `Job`은 필드를 `priority`, `seq: Reverse<u32>`, `name` 순서로 두고 `derive(Ord)`해서, 우선순위가 높을수록, 같으면 접수 번호가 작을수록 크게 만듭니다.

**상위 k개(top-k)**: 크기가 k인 **최소** 힙을 유지합니다. 새 값을 넣고 k개를 넘으면 가장 작은 값을 버리면, 끝에 남는 것이 가장 큰 k개입니다. 전체를 정렬하면 O(n log n)과 n개 분량의 메모리가 들지만 이 방법은 O(n log k)와 k개 분량이면 되므로, 데이터가 계속 흘러 들어오는 상황에도 쓸 수 있습니다.

**정렬과의 선택**: 모든 데이터가 한꺼번에 있고 한 번 순서대로 처리하면 끝이라면 `sort`로 충분합니다. 처리하는 도중에 새 항목이 계속 들어오고, 매번 그 시점의 최댓값(최솟값)이 필요할 때 힙을 씁니다. 예제 08의 스케줄러는 작업 두 개를 처리한 뒤 들어온 급한 작업(우선순위 5)이 바로 다음에 실행되는 것을 보여 줍니다. 예제는 [`08_binary_heap.rs`](examples/08_binary_heap.rs)입니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_vec_edit.rs`](examples/01_vec_edit.rs) | `insert`/`remove`/`swap_remove`, `extend`/`append`/`drain`/`retain`/`extract_if`/`dedup`/`truncate`/`split_off`, 용량 관리 | 2 |
| [`examples/02_sort.rs`](examples/02_sort.rs) | 정렬 메서드 5가지, 안정 정렬의 의미, `Reverse`, 키 함수 호출 횟수, `f64`와 `total_cmp` | 3 |
| [`examples/03_binary_search.rs`](examples/03_binary_search.rs) | `binary_search`의 `Ok`/`Err`, 정렬 유지 삽입, `partition_point`, `_by_key`/`_by` | 3 |
| [`examples/04_entry_api.rs`](examples/04_entry_api.rs) | 두 번 찾기와 entry 비교, `or_insert_with`, `or_default`, `and_modify`, `Entry` 매치 | 4 |
| [`examples/05_vecdeque.rs`](examples/05_vecdeque.rs) | 큐, 양쪽 끝, 최근 N개, 링 버퍼와 `make_contiguous`, `Vec::remove(0)`과 시간 비교 | 1, 5 |
| [`examples/06_btreemap.rs`](examples/06_btreemap.rs) | 정렬된 순회, `range`, 최소·최대, 기준점 찾기, `derive(Ord)` 키 | 1, 4, 5 |
| [`examples/07_sets.rs`](examples/07_sets.rs) | `insert`의 `bool`, 순서 지키며 중복 제거, 집합 연산, `derive(Hash, Eq)` 원소 | 1, 4, 5 |
| [`examples/08_binary_heap.rs`](examples/08_binary_heap.rs) | 최대 힙, `Reverse` 최소 힙, 작업 스케줄러, top-k | 1, 4, 5 |
| [`compile_fail/`](compile_fail) | 컬렉션에서 자주 만나는 컴파일 에러 5개 (4절) | 3, 4 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 패닉 없이 끝나고 종료 코드는 0입니다. `HashMap`/`HashSet`의 내용은 정렬하거나 `BTreeMap`/`BTreeSet`에 모아 출력하므로 실행할 때마다 같은 순서로 나옵니다. 실행마다 달라지는 것은 05의 시간뿐입니다.

### 01_vec_edit — `Vec` 편집과 용량

```sh
cargo run -p p1-ch4-collections --example 01_vec_edit
```

```text
== 1. insert / remove ==
insert(1, "장마") 후: ["봄", "장마", "여름", "가을", "겨울"]
remove(0) = 봄, 남은 목록: ["장마", "여름", "가을", "겨울"]
끝에 insert: ["장마", "여름", "가을", "겨울", "입춘"]

== 2. swap_remove ==
swap_remove(1) = 나래, 남은 목록: ["가온", "마루", "다솜", "라온"]

== 3. extend / append ==
extend 후: [1200, 3500, 800, 1500, 2000, 900], more는 그대로: [2000, 900]
append 후: [1200, 3500, 800, 1500, 2000, 900, 100, 200], coupon_items: []

== 4. drain ==
이번에 처리: ["빨래", "설거지", "청소"]
남은 일: ["장보기", "분리수거"]

== 5. retain / extract_if ==
60점 이상만: [72, 90, 85, 60]
스팸함: ["광고1", "광고2", "광고3"], 받은편지함: ["공지", "회의"]

== 6. dedup ==
dedup: [3, 5, 3, 7]
sort 후 dedup: [3, 5, 7]

== 7. truncate / split_off ==
상위: ["A", "B", "C"], 나머지: ["D", "E", "F"]
truncate(1): ["A"]

== 8. with_capacity / reserve / shrink_to_fit ==
with_capacity(10): len = 0, capacity = 10
10개 넣은 뒤: len = 10, capacity = 10
reserve(100) 뒤: capacity >= 110? true
clear 뒤: len = 0, capacity >= 110? true
shrink_to_fit 뒤: len = 3, capacity = 3
```

2번에서 "나래"를 지운 자리에 마지막 요소 "마루"가 들어갔고, 3번의 `append`는 `coupon_items`를 비웠습니다. 8번의 `capacity` 값(10, 3)은 이 툴체인에서의 결과입니다. 문서가 보장하는 것은 "최소한 요청한 만큼"이므로 다른 환경에서는 더 클 수 있습니다.

### 02_sort — 정렬

```sh
cargo run -p p1-ch4-collections --example 02_sort
```

```text
== 1. sort / sort_unstable ==
sort: [3, 7, 7, 19, 42], sort_unstable: [3, 7, 7, 19, 42]

== 2. 안정 정렬로 두 기준 적용하기 ==
1반 도윤 92점
1반 민준 92점
1반 지우 64점
2반 서연 75점
2반 하윤 88점

== 3. 불안정 정렬은 같은 요소의 순서를 보장하지 않습니다 ==
서울: 안정 [3, 6, 9, 12].. 유지=true / 불안정 [21, 9, 24, 27].. 유지=false
부산: 안정 [1, 4, 7, 10].. 유지=true / 불안정 [25, 19, 4, 22].. 유지=false
대구: 안정 [2, 5, 8, 11].. 유지=true / 불안정 [2, 5, 8, 11].. 유지=true

== 4. sort_by / sort_by_key와 Reverse ==
sort_by:     ["도윤", "민준", "하윤", "서연", "지우"]
sort_by_key: ["도윤", "민준", "하윤", "서연", "지우"]

== 5. sort_by_key와 sort_by_cached_key의 키 함수 호출 횟수 ==
sort:               ["Banana", "Cherry", "Grape", "Lemon", "apple", "date", "fig", "kiwi"]
sort_by_cached_key: ["apple", "Banana", "Cherry", "date", "fig", "Grape", "kiwi", "Lemon"]
결과가 같은가? true
키 함수 호출: sort_by_key 34번, sort_by_cached_key 8번

== 6. f64 정렬과 total_cmp ==
오름차순: [-1.0, -0.0, 0.0, 36.5, 37.25, inf, NaN]
내림차순: [NaN, inf, 37.25, 36.5, 0.0, -0.0, -1.0]
partial_cmp: [3.0, 7.75, 12.5]
```

3번의 불안정 정렬 결과와 5번의 `sort_by_key` 호출 횟수는 이 툴체인에서의 결과이며, 정렬 구현이 바뀌면 달라질 수 있습니다. 5번의 `sort`는 바이트 순이라 대문자로 시작하는 단어가 모두 앞에 옵니다(문자열 메서드 `to_lowercase`는 `ch3-strings-fmt`). 6번에서는 `-0.0`이 `0.0`보다 앞에, `NaN`이 `inf`보다 뒤에 놓입니다.

### 03_binary_search — 이진 탐색

```sh
cargo run -p p1-ch4-collections --example 03_binary_search
```

```text
== 1. binary_search의 Ok와 Err ==
30 찾기: Ok(2)
35 찾기: Err(3)
 5 찾기: Err(0)
99 찾기: Err(5)

== 2. 정렬 상태를 유지하며 삽입 ==
11시: 1번 자리에 추가 → [9, 11, 13, 15]
13시: 이미 예약됨
8시: 0번 자리에 추가 → [8, 9, 11, 13, 15]
17시: 5번 자리에 추가 → [8, 9, 11, 13, 15, 17]

== 3. 중복 값과 partition_point ==
binary_search(&80) = Ok(4)
80의 범위: 2..5, 개수 = 3
60점 미만인 학생 수: 1
80 추가 위치: 4, 결과: [55, 70, 80, 80, 80, 95]

== 4. binary_search_by_key / binary_search_by ==
152번: 종각
160번: Err(5)

== 5. 정렬되지 않은 슬라이스 ==
정렬되어 있는가? false
contains(&5) = true
binary_search(&5) = Err(5)
```

1번의 `Err(5)`는 "길이 5인 슬라이스의 맨 끝에 넣으면 된다"는 뜻입니다. 3번의 `Ok(4)`("아무거나 하나")와 5번의 `Err(5)`("의미 없는 값")는 이 툴체인에서의 결과입니다.

### 04_entry_api — entry API

```sh
cargo run -p p1-ch4-collections --example 04_entry_api
```

```text
== 1. contains_key + insert 대 entry ==
결과가 같은가? true
{"김밥": 3, "떡볶이": 1, "라면": 2, "순대": 1}

== 2. or_insert 대 or_insert_with ==
or_insert, 이미 있는 키:
  (load_profile("민수") 호출됨)
or_insert_with, 이미 있는 키:
or_insert_with, 없는 키:
  (load_profile("지영") 호출됨)
  돌려받은 값: 지영의 프로필

== 3. or_default로 묶기 (BTreeMap) ==
개발: ["민수", "아라"]
디자인: ["하늘"]
영업: ["지영", "철수"]

== 4. and_modify + or_insert로 최고 기록 갱신 ==
{"민수": 90, "지영": 85}

== 5. Entry 열거형을 match하기 ==
사과 3개 출고, 남은 수량 2
배 2개 출고, 품절되어 목록에서 삭제
사과 재고 부족: 2개뿐입니다
감: 등록되지 않은 상품입니다
남은 재고: {"사과": 2}
```

2번에서 키가 이미 있는데도 `or_insert`는 `load_profile`을 불렀고, `or_insert_with`는 부르지 않았습니다. 1번과 5번은 `HashMap`의 `(&키, &값)` 쌍을 `BTreeMap`에 `collect`해 키 순서로 출력합니다.

### 05_vecdeque — 덱과 큐

```sh
cargo run -p p1-ch4-collections --example 05_vecdeque
```

```text
== 1. 큐 (push_back / pop_front) ==
맨 앞: Some("1번 손님"), 맨 뒤: Some("3번 손님")
1번 손님 처리, 남은 대기 2명
2번 손님 처리, 남은 대기 1명
3번 손님 처리, 남은 대기 0명

== 2. 양쪽 끝 다루기 ==
앞에 추가: ["응급 환자", "가온", "나래", "다솜"]
뒤에서 빼기: Some("다솜"), ["응급 환자", "가온", "나래"]
rotate_left(1): ["화", "수", "목", "월"]

== 3. 최근 3개의 평균 (이동 평균) ==
21.0 입력 → 최근 [21.0], 평균 21.00
23.5 입력 → 최근 [21.0, 23.5], 평균 22.25
22.0 입력 → 최근 [21.0, 23.5, 22.0], 평균 22.17
26.5 입력 → 최근 [23.5, 22.0, 26.5], 평균 24.00
25.0 입력 → 최근 [22.0, 26.5, 25.0], 평균 24.50

== 4. 인덱스 접근과 링 버퍼 ==
ring = [30, 40, 50, 60], ring[0] = 30, get(9) = None
as_slices = ([30, 40], [50, 60])
정렬 후 = [30, 40, 45, 50, 60], as_slices = ([30, 40, 45, 50, 60], [])

== 5. 앞에서 꺼내기: Vec::remove(0) 대 VecDeque::pop_front ==
n = 25000: Vec 34.1ms, VecDeque 158.3µs, 합계 일치: true
n = 50000: Vec 134.5ms, VecDeque 305.5µs, 합계 일치: true
```

4번의 `as_slices` 결과는 이 툴체인에서의 결과입니다. 어떻게 나뉘든 `ring[0]`은 논리적인 맨 앞이고, `make_contiguous` 뒤에는 한 조각이 되었습니다. 5번의 시간은 컴퓨터와 실행할 때마다 다릅니다. 숫자보다 "n이 2배일 때 몇 배가 되는가"를 보세요. 이 문서를 쓸 때 `--release`로도 실행해 보았는데, `VecDeque` 쪽은 몇 배 빨라졌지만 `Vec` 쪽은 거의 그대로였습니다. 시간 대부분이 요소를 옮기는 메모리 복사라서 최적화로 줄어들 부분이 적기 때문입니다. 알고리즘의 비용(O(n²))은 컴파일러 최적화로 바뀌지 않습니다.

### 06_btreemap — 순서가 있는 맵

```sh
cargo run -p p1-ch4-collections --example 06_btreemap
```

```text
== 1. 정렬된 순회 ==
2026-11-03: 4500원
2026-11-10: 8000원
2026-11-15: 12000원
2026-11-21: 5500원
2026-11-28: 30000원

== 2. range로 범위 조회 ==
2026-11-10: 8000원
2026-11-15: 12000원
2026-11-21: 5500원
11/10 ~ 11/21 합계: 25500원

== 3. first_key_value / last_key_value / pop_first ==
가장 이른 기록: Some(("2026-11-03", 4500))
가장 늦은 기록: Some(("2026-11-28", 30000))
5일 마감: 세금 신고
12일 마감: 발표 준비
20일 마감: 보고서

== 4. 기준점 찾기 ==
95점 → Some('A')
80점 → Some('B')
79점 → Some('C')
42점 → Some('F')
다음 버스: 07:20

== 5. derive(Ord)한 타입을 키로 ==
1.9.1: 성능 개선
1.10.0: 검색 기능
최신: Version { major: 2, minor: 0, patch: 0 } (새 디자인)
```

1번은 넣은 순서(15일, 3일, 28일, ...)와 상관없이 날짜순으로 나옵니다. 5번의 범위 `v(1, 5, 0)..v(2, 0, 0)`에는 1.2.3이 들어가지 않고, 끝이 열린 범위라 2.0.0도 빠집니다.

### 07_sets — 집합

```sh
cargo run -p p1-ch4-collections --example 07_sets
```

```text
== 1. insert의 반환값으로 중복 찾기 ==
민수: 이미 로그인한 사용자
지영: 이미 로그인한 사용자
민수: 이미 로그인한 사용자
서로 다른 사용자 수: 3

== 2. 순서를 지키며 중복 없애기 ==
sort + dedup: ["async", "cli", "rust", "web"]
retain + set: ["rust", "web", "cli", "async"]

== 3. 집합 연산 (BTreeSet) ==
교집합 (둘 다): {"다솜", "라온"}
합집합 (어느 쪽이든): {"가온", "나래", "다솜", "라온", "마루", "바다"}
차집합 (밴드만): {"가온", "나래"}
대칭 차집합 (한쪽만): {"가온", "나래", "마루", "바다"}

== 4. 집합 연산 (HashSet) ==
맞힌 번호: [7, 25, 40]
12가 들어 있나? true

== 5. is_subset / is_superset / is_disjoint ==
필수 과목을 모두 들었나? true
taken이 required를 포함하나? true
겹치는 과목이 없나? true

== 6. derive(Hash, Eq)한 타입을 원소로 ==
이동 11번, 서로 다른 칸 9개
Cell { x: 1, y: 2 }에 갔었나? true
```

4번의 교집합은 `HashSet`에서 나와 순서가 정해져 있지 않으므로 `Vec`에 모아 정렬해 출력합니다. 6번의 로봇은 같은 칸을 다시 지나기도 해서, 서로 다른 칸은 출발점을 포함해 9개입니다.

### 08_binary_heap — 우선순위 큐

```sh
cargo run -p p1-ch4-collections --example 08_binary_heap
```

```text
== 1. 최대 힙 ==
peek = Some(9), len = 5
내부 순서: [9, 7, 1, 3, 5]
pop 순서: [9, 7, 5, 3, 1]
into_sorted_vec: [2, 4, 6, 8]

== 2. Reverse로 최소 힙 ==
08:15 출근
09:00 메일 확인
09:30 회의
12:00 점심

== 3. 작업 스케줄러 ==
실행: 결제 처리 (우선순위 3, 접수 1번)
실행: 메일 발송 (우선순위 2, 접수 3번)
실행: 장애 대응 (우선순위 5, 접수 4번)
실행: 로그 정리 (우선순위 1, 접수 0번)
실행: 캐시 갱신 (우선순위 1, 접수 2번)
대기 중인 작업이 없습니다

== 4. 가장 큰 3개 고르기 ==
상위 3개: [98, 95, 91]
```

1번의 "내부 순서"는 이 툴체인에서의 결과이며, 정렬되어 있지 않다는 것만 확인하면 됩니다. 3번에서 우선순위 1인 두 작업은 `Job`의 `seq` 필드가 `Reverse`로 감싸여 있어 접수 번호가 작은 "로그 정리"가 먼저 실행됩니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch4-collections
rustc --edition 2024 compile_fail/e0277_sort_f64.rs --out-dir /tmp/rust-p1-ch4
```

에러 메시지의 `note: required by a bound in ...` 아래에 표준 라이브러리 소스 위치(`/rustc/48a229ce.../library/...`)가 나오는 경우가 있습니다. 경로의 해시 값은 툴체인마다 다르며, 4.1절의 인용에서는 그 줄을 생략했습니다.

### 4.1 `Vec<f64>`에 `sort()` — E0277

파일: [`compile_fail/e0277_sort_f64.rs`](compile_fail/e0277_sort_f64.rs)

```text
error[E0277]: the trait bound `f64: Ord` is not satisfied
  --> compile_fail/e0277_sort_f64.rs:12:11
   |
12 |     temps.sort();
   |           ^^^^ the trait `Ord` is not implemented for `f64`
   |
   = help: the following other types implement trait `Ord`:
             i128
             (정수 타입 목록 생략)
           and 4 others
note: required by a bound in `slice::<impl [T]>::sort`
```

- **읽는 법**: "`f64: Ord`라는 트레이트 바운드가 만족되지 않는다"는 뜻입니다. `note:`는 그 바운드가 슬라이스의 `sort`에 붙어 있다고 알려 줍니다. `help:`의 목록은 `Ord`를 구현한 타입의 예시로, 정수 타입만 있고 실수 타입은 없습니다.
- **원인**: `sort`는 `T: Ord`(전순서)를 요구하는데, `f64`는 `NaN` 때문에 `PartialOrd`만 구현합니다(2.8절).
- **해결**: `temps.sort_by(|a, b| a.total_cmp(b));`. `NaN`이 없다고 확신하면 `sort_by(|a, b| a.partial_cmp(b).unwrap())`도 되지만 `NaN`을 만나면 패닉합니다. `BinaryHeap<f64>`, `BTreeSet<f64>`도 같은 이유로 안 됩니다.

### 4.2 `VecDeque`에 `sort()` — E0599

파일: [`compile_fail/e0599_vecdeque_sort.rs`](compile_fail/e0599_vecdeque_sort.rs)

```text
error[E0599]: no method named `sort` found for struct `VecDeque<T, A>` in the current scope
  --> compile_fail/e0599_vecdeque_sort.rs:15:10
   |
15 |     line.sort();
   |          ^^^^ method not found in `VecDeque<i32>`
```

- **읽는 법**: `VecDeque`에는 `sort`라는 메서드가 아예 없다는 뜻입니다. 4.3절의 "메서드는 있지만 바운드가 맞지 않는다"와 구별합니다.
- **원인**: `sort`는 슬라이스의 메서드입니다. `Vec`은 슬라이스로 역참조되어 쓸 수 있지만, `VecDeque`는 요소가 메모리에서 두 조각으로 나뉠 수 있어 슬라이스로 역참조되지 않습니다(2.12절).
- **해결**: `line.make_contiguous().sort();`. `make_contiguous`가 요소를 한 조각으로 모은 뒤 `&mut [T]`를 돌려줍니다. 정렬을 자주 해야 한다면 정말 `VecDeque`가 맞는지, 처음부터 `Vec`이나 `BTreeSet`/`BinaryHeap`이 맞지 않는지 다시 생각해 봅니다.

### 4.3 `derive` 없는 구조체를 `HashMap` 키로 — E0599 (또는 E0277)

파일: [`compile_fail/e0599_hashmap_key_without_derive.rs`](compile_fail/e0599_hashmap_key_without_derive.rs)

```text
error[E0599]: the method `insert` exists for struct `HashMap<Cell, u32>`, but its trait bounds were not satisfied
  --> compile_fail/e0599_hashmap_key_without_derive.rs:23:12
   |
16 | struct Cell {
   | ----------- doesn't satisfy `Cell: Eq` or `Cell: Hash`
...
23 |     visits.insert(Cell { x: 1, y: 2 }, 1);
   |            ^^^^^^
   |
   = note: the following trait bounds were not satisfied:
           `Cell: Eq`
           `Cell: Hash`
help: consider annotating `Cell` with `#[derive(Eq, Hash, PartialEq)]`
   |
16 + #[derive(Eq, Hash, PartialEq)]
17 | struct Cell {
   |
```

- **읽는 법**: "`insert` 메서드는 있지만 그 바운드가 만족되지 않는다"는 뜻입니다. `note:`가 빠진 바운드(`Cell: Eq`, `Cell: Hash`)를 나열하고, `help:`가 붙일 `derive`를 제안합니다. 2.11절에서 본 `impl` 블록의 `where K: Eq + Hash`가 바로 이 바운드입니다. 에러가 `HashMap::new()`가 아니라 `insert`에서 나는 것도 그 때문입니다.
- **같은 원인, 다른 코드**: `let mut visits = HashMap::new();`처럼 타입을 적지 않으면 키 타입이 `insert` 호출에서 정해지므로, 같은 상황이 E0277(``the trait bound `Cell: Eq` is not satisfied``, `note: required by a bound in HashMap::<K, V, S, A>::insert`)로 두 번(`Eq`, `Hash`) 보고됩니다. `BinaryHeap`에 `Ord` 없는 구조체를 넣을 때도 `push`에서는 E0277(`Task: Ord`), 그 뒤의 `pop`에서는 E0599(``the method `pop` exists for struct `BinaryHeap<Task>`, but its trait bounds were not satisfied``)가 함께 나왔습니다. 에러 코드는 달라도 읽을 것은 같습니다. **어떤 바운드가 빠졌는가.**
- **해결**: 구조체 위에 `#[derive(PartialEq, Eq, Hash)]`를 붙입니다(예제 07의 `Cell`). `BTreeMap` 키나 `BinaryHeap` 원소라면 `#[derive(PartialEq, Eq, PartialOrd, Ord)]`입니다. 필드 중에 `f64`처럼 그 트레이트를 구현하지 않는 타입이 있으면 `derive`도 실패하므로, 그 필드를 정수 등으로 바꿉니다.

### 4.4 순회 중인 맵에서 `remove` — E0502

파일: [`compile_fail/e0502_remove_while_iterating.rs`](compile_fail/e0502_remove_while_iterating.rs)

```text
error[E0502]: cannot borrow `stock` as mutable because it is also borrowed as immutable
  --> compile_fail/e0502_remove_while_iterating.rs:17:13
   |
15 |     for (name, count) in &stock {
   |                          ------
   |                          |
   |                          immutable borrow occurs here
   |                          immutable borrow later used here
16 |         if *count == 0 {
17 |             stock.remove(name);
   |             ^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

- **읽는 법**: `&stock` 한 곳에 "immutable borrow occurs here"와 "later used here"가 함께 표시됩니다. `for` 문이 반복 내내 맵을 빌리고 있고, 다음 반복으로 넘어갈 때 그 대여를 다시 쓴다는 뜻입니다.
- **원인**: 순회하는 동안 항목을 지우면 반복자가 가리키던 내부 구조가 바뀝니다. 다른 언어에서는 이것이 실행 중 예외("순회 중 수정")나 조용한 버그가 되지만, Rust는 대여 규칙으로 컴파일할 때 막습니다.
- **해결**: 조건으로 지울 때는 `stock.retain(|_, count| *count > 0);`처럼 `retain`을 씁니다. 맵의 `retain` 클로저는 키의 참조(`&K`)와 값의 가변 참조(`&mut V`)를 받습니다. 지운 항목이 필요하면 `extract_if`(`HashMap`은 1.88부터 안정화)를, 지울 키를 먼저 모은 뒤 반복이 끝나고 지우는 방법도 있습니다. `Vec`, `HashSet`, `BTreeMap`, `VecDeque`, `BinaryHeap`에도 모두 `retain`이 있습니다.

### 4.5 `get_mut`으로 얻은 참조를 쥔 채 또 `get_mut` — E0499

파일: [`compile_fail/e0499_two_get_mut.rs`](compile_fail/e0499_two_get_mut.rs)

```text
error[E0499]: cannot borrow `balance` as mutable more than once at a time
  --> compile_fail/e0499_two_get_mut.rs:21:14
   |
20 |     let from = balance.get_mut("민수").unwrap();
   |                ------- first mutable borrow occurs here
21 |     let to = balance.get_mut("지영").unwrap();
   |              ^^^^^^^ second mutable borrow occurs here
22 |     *from -= 1000;
   |     ------------- first borrow later used here
```

- **읽는 법**: 첫 번째 `get_mut`의 결과(`from`)가 22행에서 쓰이므로, 21행의 두 번째 `get_mut` 시점에 첫 번째 가변 대여가 살아 있습니다.
- **원인**: `get_mut`은 맵 **전체**를 가변으로 빌려서 값 하나의 `&mut`를 돌려줍니다. 컴파일러는 두 키가 서로 다른지 알 수 없으므로, 같은 값을 가리키는 `&mut` 두 개가 생길 가능성을 막습니다. entry API의 `or_insert`가 돌려준 `&mut V`를 쥔 채 같은 맵을 다시 쓰는 경우도 같습니다.
- **해결**: `*balance.get_mut("민수").unwrap() -= 1000;` 다음 줄에 `*balance.get_mut("지영").unwrap() += 1000;`처럼 한 번에 하나씩 고칩니다. 앞 줄의 `&mut`는 그 줄에서 끝나므로 문제가 없습니다. 두 값을 동시에 다뤄야 하면 서로 다른 키 여러 개의 `&mut`를 한 번에 받는 `get_disjoint_mut`(1.86부터 안정화)을 씁니다. 키가 겹치면 패닉합니다.

  ```rust
  if let [Some(from), Some(to)] = balance.get_disjoint_mut(["민수", "지영"]) {
      *from -= 1000;
      *to += 1000;
  }
  ```

### 4.6 실행 중 패닉

다음은 컴파일은 되지만 실행하면 패닉하고 종료 코드 101로 끝나는 경우입니다. 메시지는 이 툴체인에서 직접 실행해 확인했습니다.

| 코드 (`v = vec![1, 2, 3]`) | 패닉 메시지 | 패닉하지 않게 하려면 |
| --- | --- | --- |
| `v.insert(5, 9)` | `insertion index (is 5) should be <= len (is 3)` | `i <= v.len()`인지 먼저 확인 |
| `v.remove(5)` | `removal index (is 5) should be < len (is 3)` | `i < v.len()` 확인. `VecDeque::remove`는 `Option`을 돌려줌 |
| `v.swap_remove(3)` | `swap_remove index (is 3) should be < len (is 3)` | 위와 같음 |
| `v.drain(2..10)` | `range end index 10 out of range for slice of length 3` | 범위 끝을 `v.len()` 이하로 |
| `v.split_off(5)` | `` `at` split index (is 5) should be <= len (is 3) `` | `at <= v.len()` 확인 |
| `btree_map.range(5..3)` | `range start is greater than range end in BTreeMap` | 시작 ≤ 끝인지 확인 |
| `hash_map.get_disjoint_mut(["a", "a"])` | `duplicate keys found` | 키가 서로 다른지 확인 |

### 4.7 그 밖의 실수

| 실수 | 결과 | 바로잡기 |
| --- | --- | --- |
| `HashMap`/`HashSet`의 순회 순서에 기대는 코드나 테스트 | 실행할 때마다 결과가 달라짐 | 정렬하거나 `BTreeMap`/`BTreeSet`에 모음 (2.11절) |
| 정렬하지 않고 `dedup` | 떨어져 있는 중복이 남음 | 먼저 `sort`, 순서를 지켜야 하면 `HashSet`과 `retain` (2.14절) |
| 정렬되지 않은 슬라이스에 `binary_search` | 있는 값도 `Err`, 경고 없음 | 정렬 확인(`is_sorted`), 또는 `contains` (2.9절) |
| `Vec`을 큐로 쓰며 `remove(0)` 반복 | 요소가 많아지면 O(n²)으로 느려짐 | `VecDeque` (2.12절) |
| `sort_by_key(\|s\| &s.name)` | `lifetime may not live long enough` (E 코드 없음) | `sort_by(\|a, b\| a.name.cmp(&b.name))` (2.8절) |

## 5. 연습 문제

### 연습 1. 최근 연 파일 목록

편집기의 "최근 연 파일" 목록을 만드세요. 파일을 열면 목록 맨 앞에 오고, 이미 목록에 있던 파일을 다시 열면 원래 자리에서 빠져 맨 앞으로 옵니다. 목록은 최대 3개까지만 유지합니다.

```text
열기 순서: a.rs, b.rs, c.rs, a.rs, d.rs
최종 목록: ["d.rs", "a.rs", "c.rs"]
```

`Vec`과 `VecDeque` 중 무엇을 골랐는지, 그 이유를 비용 표로 설명해 보세요.

<details>
<summary>힌트</summary>

- 맨 앞에 넣는 일이 매번 일어나므로 `VecDeque`의 `push_front`가 어울립니다. `Vec`이라면 `insert(0, ..)`이 O(n)입니다.
- 이미 있는 파일의 위치는 `recent.iter().position(|f| *f == name)`으로 찾습니다(`position`은 조건을 처음 만족하는 위치를 `Option<usize>`로 돌려주는 반복자 메서드). 찾았으면 `recent.remove(i)`로 뺍니다. `VecDeque::remove`는 `Option<T>`를 돌려줍니다.
- 넣은 뒤 `recent.truncate(3)`으로 길이를 맞춥니다.
- 목록이 3개뿐이라 실제 속도 차이는 없습니다. 2.2절의 "O 표기는 경향"이라는 말을 떠올리며, 목록이 1만 개라면 어떨지도 생각해 보세요.

</details>

### 연습 2. 사전에서 접두사로 찾기

단어와 뜻을 담은 `BTreeMap<&str, &str>`에서 주어진 접두사로 시작하는 단어를 사전 순서로 모두 출력하는 `fn print_prefix(dict: &BTreeMap<&str, &str>, prefix: &str)`를 작성하세요. 모든 단어를 훑지 말고 `range`를 써야 합니다.

```text
단어: apple, application, apply, banana, band, bandage, can
"ban" → banana, band, bandage
"app" → apple, application, apply
"z"   → (없음)
```

<details>
<summary>힌트</summary>

- 사전 순서에서 접두사로 시작하는 단어들은 연속해서 붙어 있고, 접두사 자신보다 크거나 같습니다. 그래서 `dict.range(prefix..)`로 시작 위치를 찾은 뒤, `for (word, meaning) in dict.range(prefix..)`로 돌면서 `!word.starts_with(prefix)`가 되는 순간 `break`합니다.
- 키 타입이 `&str`이므로 `range`에 `prefix..`(`RangeFrom<&str>`)를 그대로 넘길 수 있습니다.
- 접두사로 시작하는 단어가 k개라면 이 방법은 O(log n + k)입니다. `HashMap`이었다면 n개를 모두 훑어야 합니다.

</details>

### 연습 3. 정렬된 목록 여러 개 합치기

이미 오름차순으로 정렬된 `Vec<u32>` 여러 개를 받아, 전체를 오름차순으로 합친 `Vec<u32>`를 돌려주는 `fn merge_sorted(lists: &[Vec<u32>]) -> Vec<u32>`를 `BinaryHeap`으로 작성하세요. 모두 이어 붙인 뒤 `sort()`하는 방법과 결과를 비교해 테스트해 보세요.

```rust
merge_sorted(&[vec![1, 4, 9], vec![2, 3, 10], vec![5]]) // [1, 2, 3, 4, 5, 9, 10]
```

<details>
<summary>힌트</summary>

- 각 목록의 "다음에 꺼낼 위치"를 기억합니다. 힙에는 `Reverse((값, 목록 번호, 목록 안의 위치))`를 넣어 최소 힙으로 씁니다.
- 처음에 목록마다 첫 요소를 넣고, `while let Some(Reverse((value, li, idx))) = heap.pop()`으로 꺼낼 때마다 결과에 `value`를 넣고, 그 목록에 다음 요소(`lists[li].get(idx + 1)`)가 있으면 힙에 넣습니다.
- 힙에는 언제나 목록 수(k)만큼만 들어 있으므로 전체 비용은 O(N log k)입니다(N은 전체 요소 수).

</details>

### 연습 4. 친구 추천

사용자마다 친구 목록이 `HashMap<&str, HashSet<&str>>`로 주어집니다. 한 사용자에게 "친구의 친구 중 아직 친구가 아니고 본인도 아닌 사람"을 추천하되, 함께 아는 친구 수가 많은 순서로, 같으면 이름 가나다순으로 정렬해 출력하세요.

```rust
let friends = HashMap::from([
    ("민수", HashSet::from(["지영", "하늘"])),
    ("지영", HashSet::from(["민수", "하늘", "철수"])),
    ("하늘", HashSet::from(["민수", "지영", "철수", "아라"])),
    ("철수", HashSet::from(["지영", "하늘"])),
    ("아라", HashSet::from(["하늘"])),
]);
// 민수에게 추천: 철수(함께 아는 친구 2명), 아라(1명)
```

<details>
<summary>힌트</summary>

- 민수의 친구 집합 `mine`을 꺼내고, 친구마다 그 친구의 친구 집합에 대해 `difference(mine)`을 돌면서 본인이 아니면 `counts.entry(name).or_insert(0)`에 1을 더합니다. `counts`는 `HashMap<&str, u32>`입니다.
- `counts`를 `Vec<(&str, u32)>`로 모은 뒤 `sort_by_key(|&(name, n)| (Reverse(n), name))`로 정렬합니다(2.8절).
- 결과가 `HashMap`의 순회 순서에 영향을 받지 않는지 확인하세요. 마지막에 정렬하므로 실행할 때마다 같아야 합니다.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::collections`](https://doc.rust-lang.org/std/collections/index.html): 2.1~2.3절의 분류, "When Should You Use Which Collection?", "Performance"의 비용 표, "Capacity Management", "Entries"(entry API의 목적)가 모두 이 한 페이지에 있습니다.
  - [`std::vec`](https://doc.rust-lang.org/std/vec/index.html), [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html): "Guarantees" 절(2.7절), `insert`/`remove`/`swap_remove`/`drain`/`retain`/`extract_if`/`dedup`/`split_off`의 설명과 예제.
  - [슬라이스 `[T]`](https://doc.rust-lang.org/std/primitive.slice.html): `sort`부터 `sort_by_cached_key`까지 정렬 메서드의 안정성·비용·현재 구현 설명, `binary_search`와 `partition_point`의 예제(같은 값의 범위 찾기, 정렬 유지 삽입). [`std::slice`](https://doc.rust-lang.org/std/slice/index.html) 모듈은 슬라이스 반복자 타입들을 모아 둔 곳입니다.
  - [`f64::total_cmp`](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp): totalOrder 규칙으로 정한 순서가 적혀 있습니다.
  - [`VecDeque`](https://doc.rust-lang.org/std/collections/struct.VecDeque.html): 링 버퍼 설명과 `make_contiguous`, `as_slices`.
  - [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html): 기본 해시 함수(SipHash 1-3)와 HashDoS, 키에 대한 규칙, `impl` 블록별 바운드(2.11절).
  - [`hash_map::Entry`](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html): `or_insert`부터 `and_modify`, `or_default`까지 2.10절의 메서드.
  - [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html): `range`, `first_key_value`, `pop_first`, `split_off`. 메서드마다 붙은 `where K: Ord`를 확인해 보세요.
  - [`HashSet`](https://doc.rust-lang.org/std/collections/struct.HashSet.html), [`BTreeSet`](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html): 집합 연산 메서드와 `BitAnd`, `BitOr`, `Sub`, `BitXor` 연산자 구현.
  - [`BinaryHeap`](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html): 최소 힙 만들기, 연산별 비용 표, `push`의 기대 비용 분석.
  - [`cmp::Reverse`](https://doc.rust-lang.org/std/cmp/struct.Reverse.html): `sort_by_key`에서 키의 일부만 뒤집는 예제가 있습니다.
  - [`LinkedList`](https://doc.rust-lang.org/std/collections/struct.LinkedList.html): "거의 항상 `Vec`이나 `VecDeque`가 낫다"는 안내(2.4절).
- Rust By Example
  - [Vectors](https://doc.rust-lang.org/rust-by-example/std/vec.html): `collect`로 벡터 만들기, `push`, `pop`, `iter_mut` 등 기본 사용법입니다. Part 0 ch8의 복습입니다.
  - [HashMap](https://doc.rust-lang.org/rust-by-example/std/hash.html): `insert`가 덮어쓴 값을 돌려주는 것, `remove`, 순서 없는 순회.
  - [Alternate/custom key types](https://doc.rust-lang.org/rust-by-example/std/hash/alt_key_types.html): `derive(PartialEq, Eq, Hash)`한 `Account` 구조체를 키로 쓰는 예제. `f32`/`f64`가 `Hash`를 구현하지 않는 이유도 짧게 설명합니다(2.11절).
  - [HashSet](https://doc.rust-lang.org/rust-by-example/std/hash/hashset.html): `union`, `difference`, `intersection`, `symmetric_difference`(2.14절).
- Rustlings: Part 0 ch8에서 소개한 연습을 이 챕터의 내용으로 다시 풀어 볼 수 있습니다.
  - [`05_vecs`](https://github.com/rust-lang/rustlings/tree/main/exercises/05_vecs): 벡터 만들기와 순회.
  - [`11_hashmaps`](https://github.com/rust-lang/rustlings/tree/main/exercises/11_hashmaps): `hashmaps2.rs`는 열거형 `Fruit`을 키로 쓰고 "없는 과일만 넣기"를 요구합니다(2.10절의 entry API, 2.11절의 키 조건). `hashmaps3.rs`의 점수표 만들기도 entry API로 짧게 쓸 수 있습니다.
- 에러 설명: `rustc --explain E0599`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
