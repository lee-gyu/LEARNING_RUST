# ch11. 자원 해제는 `Drop`과 가드에 맡기기

잠금, 데이터베이스 연결, 파일, 내보내지 않은 버퍼처럼 "얻었으면 반드시 돌려줘야 하는" 자원이 있습니다. 얻는 코드와 돌려주는 코드를 짝지어 직접 쓰면, 처음에는 짝이 맞아도 조기 반환이나 `?`나 패닉으로 함수를 빠져나가는 길이 하나 늘 때마다 돌려주기가 빠집니다. RAII(Resource Acquisition Is Initialization, 자원 획득은 초기화)는 자원의 수명을 값의 수명에 묶어서 이 문제를 풉니다. 값을 만들 때 자원을 얻고, 값이 버려질(drop) 때 자원을 놓습니다. 값이 버려지는 일은 컴파일러가 모든 출구에서 챙기므로 돌려주기를 잊을 수 없습니다. RAII는 C++에서 생긴 이름이고, Rust에서는 `Drop` 트레이트의 `drop` 메서드가 "버려질 때 실행할 코드"입니다.

`Drop`을 구현하는 방법과 drop 순서, `std::mem::drop`은 [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md)에서 배웠습니다. 이 챕터는 그 위에서 판단을 다룹니다. 어떤 타입에 `Drop`을 구현하는가, 가드 타입을 언제 직접 만드는가, 가드가 사는 범위를 어떻게 정하는가, 실패할 수 있는 해제는 어떻게 설계하는가입니다. [ch10](../ch10-standard-traits/README.md)이 이 챕터로 미룬 `Drop`도 여기서 다룹니다. ch10 2.3절의 "하나뿐이어야 하는 값"은 대개 자원을 쥔 값이고, 그런 타입에 `Drop`을 구현하면 `Copy`는 아예 붙일 수 없습니다(E0184, [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 4.5절).

- 원제: [Item 11: Implement the Drop trait for RAII patterns](https://effective-rust.com/raii.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch11-drop-raii`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 본편 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 2.4절: 스코프가 끝나면 값이 버려진다
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.6절: `Drop` 구현, drop 순서, `.drop()`을 부를 수 없는 이유(E0040), `std::mem::drop`
- [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) 2.7절, [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 2.4절: `Mutex`와 `MutexGuard`, `try_lock`, 포이즈닝
- [Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.7절: `let _ =`와 `let _x =`의 drop 시점
- [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.10절: `RefCell`의 대여 가드, 조건식에서 만든 가드의 수명
- [Part 1 ch9](../../part1-standard-lib/ch9-io-fs-path/README.md) 2.7절, 2.9절: `BufWriter`의 `flush`, `File`을 닫을 때의 에러

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 수동으로 짝지은 "얻기·돌려주기"가 조기 반환, `?`, 패닉에서 깨지는 이유를 설명하고, 해제해야 하는 자원을 가진 타입에 `Drop`을 구현한 가드로 바꿀 수 있다. API를 고칠 수 없을 때는 쓰는 자리에서 `DropGuard`로 감쌀 수 있다.
2. `Drop`을 구현해야 하는 타입과, 필드가 이미 해제를 맡고 있어 구현하지 않아도 되는 타입을 구별하고, 필요 없는 `Drop`이 만드는 비용을 설명할 수 있다.
3. 블록과 `drop(x)`로 가드의 범위를 좁힐 수 있고, 이름 없이 버린 가드가 그 자리에서 풀리는 함정을 `#[must_use]`로 막을 수 있다.
4. `Drop::drop`이 `&mut self`를 받고 아무것도 돌려주지 못한다는 제약을 설명하고, 실패할 수 있는 해제를 `Result`를 돌려주는 메서드와 안전망 역할의 `Drop`으로 설계할 수 있다.
5. 지역 변수, 구조체 필드, 임시값이 버려지는 순서를 이용해 가드끼리의 해제 순서를 정할 수 있다.

## 2. 핵심 개념

2.1절에서 표준 라이브러리의 가드를 훑고, 2.2~2.5절에서 지침마다 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 설명합니다. 지침 1~3은 책의 Item 11이 권하는 것이고, 지침 4는 이 챕터가 더한 것입니다(2.6절). 코드는 핵심만 싣고, 두 형태를 함께 실행해 비교하는 전체 코드는 `examples/`에 있습니다.

### 2.1 지도: 표준 라이브러리의 가드

이 챕터에서는 "자원을 쥐고 있다가 버려질 때 놓는 값"을 **가드(guard)** 라고 부릅니다. 본편에서 만난 잠금 가드(`MutexGuard`)와 대여 가드(`Ref`/`RefMut`)가 대표적입니다.

| 타입 | 쥐고 있는 것 | 버려질 때 하는 일 | 해제가 실패하면 | 본편 |
| --- | --- | --- | --- | --- |
| `MutexGuard`, `RwLockReadGuard`, `RwLockWriteGuard` | 잠금 | 잠금을 풂 | 실패하지 않음 | Part 0 ch15, Part 1 ch11 |
| `Ref`, `RefMut` | `RefCell`의 대여 | 대여 개수를 되돌림 | 실패하지 않음 | Part 1 ch7 |
| `Box`, `Vec`, `String`, `Rc` | 힙 메모리(`Rc`는 참조 카운트) | 메모리 해제(`Rc`는 카운트를 줄임) | 실패하지 않음 | Part 0 ch14 |
| `File` | 운영체제의 파일 핸들 | 파일을 닫음 | 에러를 무시함. 확인하려면 `sync_all()` | Part 1 ch9 |
| `BufWriter<W>` | 아직 내보내지 않은 데이터 | 버퍼를 비우려 시도함 | 에러를 무시함. 확인하려면 `flush()`나 `into_inner()` | Part 1 ch9 |
| `DropGuard<T, F>` (1.100.0) | 값 `T`와 클로저 `F` | 값을 클로저에 넘겨 실행 | 클로저가 정함 | 이 챕터 2.2절 |

"해제가 실패하면" 열이 2.4절의 주제입니다. `drop`은 아무것도 돌려주지 못하므로 `File`과 `BufWriter`는 버려질 때 난 에러를 버리고, 에러를 받고 싶은 사람을 위해 따로 메서드를 둡니다.

Rust의 RAII에는 소유권이 보태 주는 것이 있습니다. 소유권 덕분에 한 값은 정확히 한 번 버려지고, `drop(guard)`로 버린 가드는 다시 쓸 수 없습니다(E0382, 4.3절). 대여 검사기는 가드가 빌려 온 대상보다 오래 살지 못하게 막습니다. 수동으로 잠금을 푸는 언어에서 흔한 "풀고 나서 또 쓰기"와 "두 번 풀기"가 컴파일 단계에서 걸러집니다.

### 2.2 지침 1: 해제해야 하는 자원을 가진 타입에는 `Drop`을 구현한다

**문제 상황.** 데이터베이스 연결 풀에 연결이 3개 있습니다. 요청을 처리할 때 연결을 빌리고, 다 쓰면 풀에 돌려줘야 다음 요청이 쓸 수 있습니다. 풀은 `take()`(빌리기)와 `give_back()`(돌려주기)을 제공합니다.

**비권장 코드.** 처음에는 `take`와 `give_back`의 짝이 맞았는데, 빈 요청 검사와 번호 파싱이 더해지면서 빠지는 길이 생겼습니다.

```rust
fn handle_manual(pool: &Pool, req: &str) -> Result<String, RequestError> {
    let mut conn = pool.take().ok_or(RequestError::NoConn)?;
    if req.is_empty() {
        return Err(RequestError::Empty); // 돌려주기를 빠뜨림
    }
    let user: usize = req.parse()?; // 파싱에 실패하면 여기서 반환
    let name = conn.query(user);    // 패닉하면 되감기로 빠져나감
    pool.give_back(conn);
    Ok(name)
}
```

[`01_manual_vs_guard.rs`](examples/01_manual_vs_guard.rs)에서 요청 5개(`"1"`, `""`, `"x"`, `"7"`, `"2"`)를 차례로 처리하면, 빈 요청과 파싱 실패와 패닉(없는 사용자 번호 7)이 연결을 하나씩 가져가 버리고, 마지막 정상 요청 `"2"`는 `NoConn`으로 실패합니다. 연결의 메모리는 소유권 규칙대로 해제됐습니다. 사라진 것은 "풀에 돌려준다"는, 프로그램만 아는 규칙입니다. 컴파일러는 이 규칙을 모르므로 경고하지 않습니다.

**권장 코드.** `get()`이 연결 대신 가드를 돌려주고, 가드의 `drop`이 연결을 돌려줍니다.

```rust
#[must_use = "버리면 연결이 바로 풀로 돌아갑니다"]
struct PooledConn<'a> {
    pool: &'a Pool,
    conn: Option<Conn>,
}

impl Drop for PooledConn<'_> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.pool.give_back(conn);
        }
    }
}
// Deref/DerefMut으로 안의 Conn을 빌려 줍니다.
```

요청 처리 함수는 수동판에서 `give_back` 한 줄을 지운 것이 전부인데, 빈 요청이든 파싱 실패든 패닉이든 쉬는 연결이 언제나 3개로 돌아오고 마지막 요청도 성공합니다. 패닉 경로가 동작하는 것은 패닉으로 함수를 빠져나갈 때도 되감기가 지역 변수를 버리기 때문입니다. 패닉과 되감기의 자세한 동작은 [`ch18-dont-panic`](../ch18-dont-panic/README.md)과 Part 10 `ch8-unwinding`에서 다룹니다.

가드를 설계할 때 세 가지를 봅니다.

- **가드를 거쳐서만 자원에 닿게 합니다.** 연결은 가드 안에 있고 `Deref`/`DerefMut`으로만 빌려 줍니다. 책이 RAII를 설명하며 강조하는 성질, "자원에 닿을 수 있는 것은 그 값이 살아 있는 동안뿐"이 타입으로 보장됩니다. 값을 `Mutex` 안에 두어 `MutexGuard`로만 닿게 한 표준 라이브러리의 설계와 같습니다.
- **`drop`에서 값을 꺼내야 하면 `Option`에 담습니다.** `drop`은 `&mut self`를 받으므로 필드를 값으로 옮길 수 없습니다. `Option::take()`가 값을 꺼내고 `None`을 남깁니다(2.4절).
- **`#[must_use]`를 붙입니다.** 가드를 이름 없이 버리는 실수를 막습니다(2.3절). `get()`처럼 가드를 `Option`에 감싸 돌려주면 타입에 붙인 속성이 경고를 내지 않으므로 메서드에도 붙입니다(이 툴체인에서 `pool.get();`으로 확인).

**API를 고칠 수 없을 때: `DropGuard`.** 짝으로 된 API가 남의 라이브러리에 있다면, 얻자마자 그 자리에서 돌려주기를 예약합니다. 1.100.0에서 안정화된 `std::mem::DropGuard`는 값과 클로저를 받아, 가드가 버려질 때 값을 클로저에 넘깁니다.

```rust
let conn = pool.take().ok_or(RequestError::NoConn)?;
let mut conn = DropGuard::new(conn, |c| pool.give_back(c));
// conn.query(..)는 DerefMut으로 안의 Conn을 씁니다.
```

예제의 3번 구역은 2번 구역과 같은 결과를 냅니다. 정리를 취소하고 값을 되찾으려면 `DropGuard::dismiss(guard)`를 부릅니다(클로저는 실행되지 않습니다). 여러 곳에서 쓰는 자원이라면 API가 직접 가드 타입을 돌려주게 하는 편이 낫습니다. 이름이 있는 타입에는 메서드와 문서와 `#[must_use]` 메시지를 붙일 수 있고, 구조체 필드의 타입으로 적을 수도 있습니다. 한 함수 안에서 한 번 쓰는 정리라면 `DropGuard`로 충분합니다(연습 3).

**어떤 타입에 `Drop`이 필요한가.** 책은 세 부류를 꼽습니다. 파일 디스크립터 같은 운영체제 자원, 잠금 같은 동기화 자원, 그리고 unsafe 코드나 FFI가 다루는 원시 메모리입니다([`ch16-avoid-unsafe`](../ch16-avoid-unsafe/README.md), [`ch34-ffi-boundaries`](../ch34-ffi-boundaries/README.md)). `Box`나 `Vec`이 소유한 메모리는 이미 그 타입들의 `Drop`이 해제합니다.

**따르지 않아도 되는 경우: 필드가 이미 해제를 맡을 때.** `Drop` 문서의 말대로 Rust는 값의 필드마다 소멸자를 자동으로 부르므로, 대부분의 타입은 `Drop`을 구현할 필요가 없습니다. 예제의 `Session { conn: PooledConn<'a>, user: String }`은 `Drop`이 없지만, 세션이 버려지면 `conn` 필드가 연결을 돌려줍니다(4번 구역). 자원을 직접 다루는 타입 하나에만 `Drop`을 두고, 그것을 담는 타입에는 두지 않습니다. 필요 없는 `Drop`은 비용이 있습니다.

- 필드를 옮겨 꺼낼 수 없습니다. `let Session { conn, user } = session;` 같은 구조 분해가 E0509가 됩니다(4.1절).
- `Copy`를 붙일 수 없습니다(E0184).
- 가드가 빌린 것을 스코프 끝까지 붙잡습니다(2.3절, 4.2절).

**`Drop`에 맡길 수 없는 것.** 소멸자가 반드시 실행된다는 보장은 없습니다. `mem::forget`은 값을 버리지 않고 잊어버리며, `Rc` 순환 참조는 영영 버려지지 않고(Part 0 ch14 2.10절), `process::exit`는 문서대로 어느 스레드의 스택에 있는 소멸자도 실행하지 않고 프로세스를 끝냅니다. 그래서 `Drop`은 자원 정리에 쓰고, 메모리 안전성처럼 "반드시"가 필요한 일을 `Drop`에만 기대지 않습니다. 이 주제는 Part 10 `ch7-obrm`에서 다룹니다.

### 2.3 지침 2: 가드가 사는 범위를 블록으로 좁힌다

책이 굵은 글씨로 권하는 지침입니다. 가드가 살아 있는 동안 자원은 다른 쪽이 쓸 수 없으므로, 가드는 꼭 필요한 동안만 살아야 합니다. [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 2.4절이 잠금으로 본 것을 이 절에서는 가드 일반으로 넓히고, 반대로 범위가 너무 짧아지는 실수도 봅니다.

**문제 상황.** 여러 스레드가 함께 쓰는 게임 순위표 `Mutex<Board>`에 점수를 넣고, "몇 위가 되었다"는 알림을 만듭니다. 알림 만들기는 순위표가 필요 없는 느린 일(실제로는 메시지 전송)입니다.

**비권장 코드.** 가드를 함수 끝까지 쥡니다.

```rust
let mut board = lb.inner.lock().unwrap();
let rank = Leaderboard::insert(&mut board, name, score);
// ---- 여기부터는 순위표가 필요 없는 일 ----
let others_can_lock = lb.other_thread_can_lock();
(announce(name, rank), others_can_lock)
```

[`02_narrow_guard_scope.rs`](examples/02_narrow_guard_scope.rs)는 알림을 만드는 순간 스코프 스레드에서 `try_lock()`을 시도해 봅니다. 결과는 `false`입니다. 알림이 느릴수록 다른 스레드가 모두 그만큼 기다립니다.

**권장 코드.** 순위표를 고치는 일만 블록에 넣고, 블록의 값으로 필요한 것(순위)만 꺼냅니다.

```rust
let rank = {
    let mut board = lb.inner.lock().unwrap();
    Leaderboard::insert(&mut board, name, score)
}; // 가드는 여기서 버려집니다.
let others_can_lock = lb.other_thread_can_lock(); // true
```

블록 때문에 들여쓰기가 한 단계 늘지만, 잠금이 어디서 풀리는지 눈에 보이고 블록 밖에서는 가드를 쓸 수 없습니다. 블록으로 감싸기 어색하면 다 쓴 자리에서 `drop(board)`를 부릅니다(예제의 3번 구역). `x.drop()`은 E0040이므로 `drop(x)` 함수를 씁니다([Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 4.3절). `drop(x)`가 가드를 옮겨 가므로, 그 뒤에 가드를 쓰면 E0382입니다(4.3절). 한 문장으로 끝나는 일이라면 가드를 변수에 담지 않고 `lb.inner.lock().unwrap().scores.len()`처럼 쓰면 그 문장이 끝날 때 풀립니다(2.5절). 다만 `match`, `if let`, `while let`의 조건식에서 만든 가드는 본문이 끝날 때까지 살아 있으므로, 조건식에서 가드를 만들지 말고 필요한 값을 `let` 문으로 먼저 꺼냅니다([Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.10절, 2024 에디션의 `if let` 변경은 [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.4절).

**너무 짧은 범위: 이름 없이 버린 가드.** 반대 방향의 실수도 있습니다. 순위표에는 시즌을 정산하는 동안 늦은 제출을 막는 동결 가드 `freeze()`가 있습니다. 이 가드는 쓰지 않고 쥐고만 있으면 되는 가드라서 이름을 붙이기 귀찮아집니다.

| 코드 | 동결 가드가 버려지는 때 | 정산 중 늦은 제출을 받음 |
| --- | --- | --- |
| `lb.freeze();` | 그 문장 끝 | `true` |
| `let _ = lb.freeze();` | 그 문장 끝(`_`는 값을 묶지 않음) | `true` |
| `let _frozen = lb.freeze();` | 블록 끝 | `false` |

`_`와 `_x`의 차이는 [Part 0 ch18](../../part0-getting-started/ch18-patterns/README.md) 2.7절에서 배웠습니다. 표준 라이브러리의 잠금이라면 `let _ = m.lock()`을 rustc의 `let_underscore_lock`(기본 deny)이 컴파일 에러로 막지만, 직접 만든 가드는 아무 말 없이 버려집니다. 그래서 가드 타입에는 `#[must_use]`를 이유와 함께 붙입니다. 표준 라이브러리의 `MutexGuard`에도 `#[must_use = "if unused the Mutex will immediately unlock"]`이 붙어 있습니다(1.100.0 소스). 예제 02에서 `#[expect(unused_must_use, ..)]`를 지우면 첫째 줄에 다음 경고가 납니다.

```text
warning: unused `Frozen` that must be used
    |
    |         lb.freeze(); // 문장 끝에서 바로 버려집니다.
    |         ^^^^^^^^^^^
    |
    = note: 버리면 동결이 바로 풀립니다. `let _frozen = ...`처럼 이름을 붙이세요
    = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
help: use `let _ = ...` to ignore the resulting value
```

`help:`는 `let _ = ...`을 제안하는데, 가드에서는 이것이 둘째 줄의 함정 그대로입니다. 일반적인 `#[must_use]` 값에 맞춘 제안이기 때문입니다. `note:`에 나오는 속성의 메시지가 올바른 방법을 알려 주도록 써 둡니다. `let _ =` 쪽은 `#[must_use]`로 잡히지 않습니다. rustc의 `let_underscore_drop` 린트(기본 allow)를 켜면 ``non-binding let on a type that has a destructor``로 경고하고 `let _unused = ...` 또는 `drop(...)`을 제안합니다(같은 모양의 코드에 `-W let_underscore_drop`을 주어 확인).

**`Drop`이 있는 가드는 빌린 것을 스코프 끝까지 붙잡습니다.** 단일 스레드에서도 범위를 좁혀야 하는 이유가 있습니다. 보통의 대여는 마지막으로 쓴 곳에서 끝나지만, `Drop`을 구현한 값은 스코프 끝에서 `drop(&mut self)`가 실행되며 빌린 것을 다시 쓸 수 있습니다. 그래서 `&mut server`를 빌린 가드를 마지막으로 쓴 뒤에도 스코프가 끝날 때까지 `server`를 읽을 수 없습니다. 컴파일러는 ``mutable borrow might be used here, when `sync` is dropped and runs the `Drop` code``라고 알려 줍니다(4.2절). 블록으로 가드를 먼저 끝내면 됩니다. 예제 03의 1번 구역이 그렇게 씁니다.

**따르지 않아도 되는 경우.**

- 함수 전체가 잠금 안에서 해야 하는 짧은 일이라면 가드를 함수 끝까지 두어도 됩니다.
- 범위를 좁힌다는 것은 필요 없는 일을 빼는 것이지, 한 번에 끝나야 하는 일을 쪼개는 것이 아닙니다. 점수를 넣는 잠금과 순위를 읽는 잠금을 따로 잡으면, 그 사이에 다른 스레드가 점수를 넣어 순위가 틀릴 수 있습니다. 잠금을 어디서 잡고 풀지, 교착 상태를 어떻게 피할지는 [`ch17-shared-state-parallelism`](../ch17-shared-state-parallelism/README.md)에서 다룹니다.
- 동결 가드처럼 범위 자체가 목적인 가드는 지켜야 할 구간 전체에 걸쳐 둡니다.

### 2.4 지침 3: 실패할 수 있는 해제는 `Result`를 돌려주는 메서드로 따로 둔다

`Drop`의 메서드는 `fn drop(&mut self)` 하나입니다. 이 시그니처에서 두 가지 제약이 나옵니다.

- **실패를 돌려줄 수 없습니다.** 반환 타입이 `()`이므로 해제가 실패해도 부르는 쪽에 알릴 길이 없습니다. `drop` 안에서 할 수 있는 일은 무시하기, 로그 남기기, 패닉하기인데, 패닉은 피해야 합니다. `Drop` 문서는 되감기 중에 실행된 `drop`이 다시 패닉하면(이중 패닉) 프로그램이 중단될 가능성이 크다고 경고합니다([`ch18-dont-panic`](../ch18-dont-panic/README.md)).
- **필드를 값으로 꺼낼 수 없습니다.** `self`를 빌려 받을 뿐이므로, 필드를 옮기려면 2.2절처럼 `Option::take()`나 `mem::take()`로 꺼내고 빈 값을 남깁니다.

`drop`을 직접 부를 수 없는 것(E0040)도 같은 시그니처 때문입니다. `x.drop()`이 컴파일된다면 `drop`은 `x`를 빌리기만 하므로, 정리가 끝난 `x`를 계속 쓸 수 있고 스코프 끝에서 한 번 더 정리하게 됩니다. 그래서 일찍 버릴 때는 값을 옮겨 가는 `drop(x)` 함수를 씁니다([Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.6절).

**문제 상황.** 메모 앱이 편집한 메모를 모아 두었다가 서버에 한꺼번에 올립니다. 서버에는 할당량이 있어서 넘치면 올리기가 실패하고, 그때는 하나도 저장되지 않습니다.

**비권장 코드 1.** 버려질 때 올리도록 `Drop`에만 맡깁니다. 사용자가 올리기를 잊을 수 없으니 좋아 보입니다.

```rust
impl Drop for AutoSync<'_> {
    fn drop(&mut self) {
        let _ = self.server.upload(&self.pending); // 결과를 버릴 수밖에 없음
    }
}
```

[`03_fallible_release.rs`](examples/03_fallible_release.rs)의 1번 구역에서 할당량이 2개인 서버에 메모 3개를 모아 두면, 서버에 저장된 메모는 0개이고 실패를 알려 준 곳은 없습니다. [Part 1 ch9](../../part1-standard-lib/ch9-io-fs-path/README.md) 2.7절에서 본 `BufWriter`가 flush 없이 버려질 때와 같은 일입니다.

**권장 코드.** 해제를 정식 메서드 `finish`로 두어 `Result`를 돌려주고, `Drop`은 `finish`를 부르지 못한 경우를 위한 안전망으로 남깁니다.

```rust
fn finish(mut self) -> Result<usize, SyncError> {
    let pending = mem::take(&mut self.pending); // 빈 Vec을 남김 → 뒤의 Drop은 할 일이 없음
    match self.server.upload(&pending) {
        Ok(()) => Ok(pending.len()),
        Err(reason) => Err(SyncError { reason, unsent: pending }),
    }
}

impl Drop for NoteSync<'_> {
    fn drop(&mut self) {
        if self.pending.is_empty() {
            return; // finish를 불렀음
        }
        // finish 없이 버려졌습니다(조기 반환, ?, 패닉). 최선을 다해 올려 보되, 결과는 로그로만 남깁니다.
    }
}
```

세 가지를 확인합니다.

- **에러가 부르는 쪽에 도착합니다.** 예제에서 할당량이 2개이면 `finish`가 `Err`를 돌려주고, 에러에 담겨 돌아온 메모 3개를 기기에 보관합니다. 실패한 해제가 자원을 함께 돌려주는 것은 `BufWriter::into_inner`의 에러(`IntoInnerError`)가 `into_inner()`로 원래 `BufWriter`를 돌려주는 것과 같은 설계입니다.
- **안전망은 `finish`를 부르지 못한 길을 지킵니다.** `?`로 일찍 반환한 `sync_lines`에서는 `Drop`이 모아 둔 2개를 올립니다. 결과는 여전히 돌려줄 수 없으므로, 실제 코드라면 로그로 남길 자리입니다(예제는 관찰을 위해 출력합니다).
- **`finish` 뒤에는 `Drop`이 할 일이 없어야 합니다.** 바로 다음의 비권장 코드가 그 이유입니다.

**비권장 코드 2: `self`를 받는 메서드도 `Drop`을 막지 못합니다.** `finish(self)`에서 `let pending = self.pending;`으로 필드를 꺼내면 E0509입니다(4.1절). 컴파일러의 `help:` 중 하나가 `self.pending.clone()`을 제안하는데, 따르면 컴파일은 되지만 2번 구역처럼 메모 3개를 올렸는데 서버에는 6개가 저장됩니다. `finish`가 `self`를 받아도, `finish`가 끝나는 순간 `self`가 버려지며 `Drop`이 실행되기 때문입니다. 복제한 쪽을 올린 뒤 `Drop`이 원본을 한 번 더 올렸습니다. `self`를 받는 해제 메서드는 값을 `Drop`이 할 일이 없는 상태로 남겨야 합니다. 기본값이 있는 필드는 `mem::take`, 없는 필드는 `Option`에 담아 `take()`를 쓰고, 그것도 어려우면 "끝났음" 표시 필드를 둡니다. 안전망이 할 일은 자원마다 다릅니다. 버퍼라면 `BufWriter`처럼 최선을 다해 비우고, 트랜잭션이라면 되돌립니다(연습 2).

**따르지 않아도 되는 경우.**

- 해제가 실패할 수 없다면(메모리 해제, 잠금 풀기) `Drop`만으로 충분합니다. 2.1절 표의 위쪽 세 줄이 그렇습니다.
- 실패해도 부르는 쪽이 할 수 있는 일이 없다면(프로그램을 끝내며 임시 파일 지우기 같은 최선의 노력) `Drop`에서 조용히 무시해도 됩니다. 이때는 `File` 문서처럼 "버려질 때의 에러는 무시된다"고 문서에 적습니다.

### 2.5 지침 4: drop 순서를 가드 설계에 쓴다 (책에 없는 지침)

drop 순서의 규칙은 Part 0 ch14 2.6절에서 봤습니다. 지역 변수는 선언의 역순으로, 구조체 필드는 선언 순서대로 버려집니다. `Drop` 문서는 필드 순서를 언어가 보장한다고 밝힙니다. 임시값은 대개 그 문장이 끝날 때 버려집니다(예외는 2.3절의 조건식과, [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.6절의 마지막 표현식). 가드가 둘 이상이고 둘 사이에 순서가 필요할 때 이 규칙이 설계 문제가 됩니다. 컴파일러는 가드끼리의 순서를 확인해 주지 않습니다.

**문제 상황.** 공유 문서를 고칠 때 편집 잠금 `EditLock`을 얻고, 고친 줄을 초안 `Draft`에 모았다가, 초안이 버려질 때 문서에 반영합니다. 초안은 잠금이 살아 있는 동안 반영돼야 합니다. 잠금이 먼저 풀리면 그 틈에 다른 편집자가 끼어들 수 있습니다.

**지역 변수.** [`04_drop_order_design.rs`](examples/04_drop_order_design.rs)의 1번 구역입니다.

```rust
let mut draft = Draft::new(&doc); // 비권장: 초안을 먼저 만들면
let _lock = doc.lock("민지");     // 잠금이 먼저 풀리고 초안이 나중에 반영됨

let _lock = doc.lock("민지");     // 권장: 잠금을 먼저 얻으면
let mut draft = Draft::new(&doc); // 초안이 먼저 반영되고 잠금이 나중에 풀림
```

"먼저 얻은 것을 나중에 놓는다"는 규칙은 블록을 겹쳐 쓴 모양과 같아서, 의존하는 쪽(초안)을 나중에 만들면 저절로 맞습니다. 선언 순서를 바꿀 수 없는 사정이 있으면 `drop(draft); drop(lock);`으로 순서를 직접 적습니다.

**구조체 필드.** 두 가드를 한 구조체에 담으면 필드 선언 순서가 해제 순서입니다. 구조체 리터럴에서 어느 필드를 먼저 적었는지(먼저 만들었는지)는 상관없습니다.

```rust
/// 필드는 선언 순서대로 버려집니다. 초안이 잠금보다 먼저 정리돼야 하므로
/// 이 두 필드의 순서를 바꾸지 마세요.
struct Session<'a> {
    draft: Draft<'a>,
    lock: EditLock<'a>,
}
```

`lock`을 위에 선언한 `SessionLockFirst`는 잠금을 먼저 풀고, 위 `Session`은 초안을 먼저 반영합니다. 필드 순서는 보기 좋게 재배치하기 쉬운 곳이라, 순서에 기대고 있다면 선언 옆에 주석으로 남깁니다. 순서를 필드 배치에 맡기기 싫다면 `Session`에 `Drop`을 구현해 그 안에서 초안을 먼저 반영할 수 있습니다(연습 4). 대신 2.2절의 비용이 따라옵니다.

**임시값.** `let holder = doc.lock("하린").holder();`의 잠금 가드는 이 `let` 문이 끝날 때 풀립니다. 다음 문장에서 본 잠금 상태는 `None`입니다. 한 문장으로 끝나는 일에는 편하지만, 여러 문장에 걸쳐 잠금이 필요하면 변수에 담습니다.

**따르지 않아도 되는 경우.** 가드끼리 서로 관계가 없다면 순서를 신경 쓸 필요가 없습니다. 순서가 중요한 경우는 한 가드의 `drop`이 다른 가드가 지키는 상태를 읽거나 고칠 때입니다.

### 2.6 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **`std::mem::DropGuard`가 생겼습니다.** 1.100.0에서 안정화되었습니다(로컬 문서와 소스의 `#[stable(feature = "drop_guard", since = "1.100.0")]`로 확인). 책은 가드를 타입으로 만드는 방법만 다룹니다. 한 번 쓰는 정리는 이제 타입을 만들지 않고 `DropGuard::new(값, 클로저)`로 쓸 수 있습니다(2.2절). `DropGuard::new`에는 `#[must_use]`가 붙어 있습니다.
- **로컬 문서의 `Drop` 정의 표시.** 1.100.0 문서는 `drop`을 "Provided Methods"에 두고 nightly 전용 `pin_drop`(`pin_ergonomics`)을 함께 보여 줍니다. 안정판에서는 여전히 `drop`을 구현해야 하며, `impl Drop for X {}`는 E0046(``not all trait items implemented, missing: `drop` ``)입니다.
- **E0040의 표시 방식.** 메시지(``explicit use of destructor method``)는 책과 같고, `help:`가 `drop(x)`로 바꾼 줄을 보여 주는 형식만 다릅니다(Part 0 ch14 4.3절의 파일로 확인).
- **책에 없는 지침.** 다음은 책의 Item 11이 명시적으로 권하지 않는 내용입니다(2026-11-19에 온라인판에서 확인). 책은 자원 타입에 `Drop` 구현, 블록으로 범위 좁히기, `drop(x)` 사용, 실패할 수 있는 해제를 위한 별도 메서드까지 다룹니다.
  - 지침 1의 "필드가 해제를 맡으면 `Drop`을 구현하지 않는다"와 그 비용(E0509, E0184, 대여 연장): `Drop` 문서의 "you don't have to implement Drop in most cases"와 Drop check 절, 직접 컴파일한 결과가 근거입니다.
  - 지침 2의 "이름 없이 버린 가드"와 `#[must_use]`: `MutexGuard`의 `#[must_use]` 속성(1.100.0 소스), rustc `let_underscore_drop` 린트가 근거입니다.
  - 지침 2의 "`Drop`이 있는 가드는 대여를 스코프 끝까지 붙잡는다": E0502의 메시지와 `Drop` 문서의 Drop check 절이 근거입니다.
  - 지침 3의 "`Drop`은 안전망, 해제 메서드 뒤에는 할 일이 없게"와 "에러에 자원을 담아 돌려준다": `BufWriter` 문서(flush한 뒤에는 drop이 파일 작업을 시도하지도 않음)와 `IntoInnerError::into_inner`, 직접 실행한 결과(clone 판의 중복 저장)가 근거입니다.
  - 지침 4 전체: `Drop` 문서의 Drop order 절과 The Reference의 Destructors 장이 근거입니다.
- **달라지지 않은 점.** `std::mem::drop`의 정의는 여전히 `pub fn drop<T>(_x: T) {}`이고, `File`과 `BufWriter`는 여전히 버려질 때의 에러를 무시합니다(로컬 문서로 확인). 책이 다루지 않은 `mem::forget`과 소멸자 실행이 보장되지 않는다는 점은 2.2절 끝에서 짧게 짚었습니다.

### 2.7 기억할 것

- 돌려줘야 하는 자원은 수동 짝 대신 가드에 맡깁니다. 가드를 거쳐서만 자원에 닿게 하고, `#[must_use]`를 붙입니다. 고칠 수 없는 짝 API는 `DropGuard`로 감쌉니다.
- `Drop`은 자원을 직접 다루는 타입에만 구현합니다. 그것을 담는 타입은 필드가 알아서 해제합니다.
- 가드는 블록이나 `drop(x)`로 필요한 동안만 살게 합니다. `let _ =`나 문장으로 버린 가드는 그 자리에서 풀립니다.
- `drop`은 실패를 알릴 수 없습니다. 실패할 수 있는 해제는 `Result`를 돌려주는 메서드로 두고, `Drop`은 그 메서드를 부르지 못한 경우의 안전망으로 남깁니다.
- 가드 사이에 순서가 있으면, 지역 변수는 의존하는 쪽을 나중에 선언하고 필드는 먼저 정리할 쪽을 위에 둡니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행하며, 모든 예제는 종료 코드 0으로 끝납니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 관련 절 |
| --- | --- | --- | --- | --- |
| [`01_manual_vs_guard.rs`](examples/01_manual_vs_guard.rs) | 1 | `take`/`give_back` 수동 짝 | `get()`이 돌려주는 `PooledConn`, `DropGuard`, `Drop` 없는 `Session` | 2.2 |
| [`02_narrow_guard_scope.rs`](examples/02_narrow_guard_scope.rs) | 2 | 가드를 함수 끝까지 쥠, `let _ =` | 블록, `drop(guard)`, `let _frozen =` | 2.3 |
| [`03_fallible_release.rs`](examples/03_fallible_release.rs) | 3 | `Drop`에서만 올림, clone한 `finish` | `finish(mut self) -> Result` + 안전망 `Drop` | 2.4 |
| [`04_drop_order_design.rs`](examples/04_drop_order_design.rs) | 4 | 초안을 먼저 선언, `lock` 필드를 위에 | 잠금을 먼저 선언, `draft` 필드를 위에 | 2.5 |

### 01_manual_vs_guard — 수동 짝과 가드

연결 3개짜리 풀로 요청 5개를 세 가지 방식으로 처리합니다. 풀은 맨 뒤의 연결부터 빌려 주고 돌려받은 연결을 맨 뒤에 넣으므로, 출력에 이름이 보이는 연결은 모두 #3입니다. 4번 구역은 가드를 필드로 가진 `Session`을 버리거나 구조 분해합니다.

```sh
cargo run -p er-ch11-drop-raii --example 01_manual_vs_guard
```

```text
1. 비권장: 수동으로 짝을 맞춤 (take / give_back)
    요청 "1" → 서준 (연결 #3), 쉬는 연결 3개
    요청 "" → Empty, 쉬는 연결 2개
    요청 "x" → BadNumber, 쉬는 연결 1개
    요청 "7" → 패닉, 쉬는 연결 0개
    요청 "2" → NoConn, 쉬는 연결 0개
2. 권장: get()이 돌려주는 가드에 맡김
    요청 "1" → 서준 (연결 #3), 쉬는 연결 3개
    요청 "" → Empty, 쉬는 연결 3개
    요청 "x" → BadNumber, 쉬는 연결 3개
    요청 "7" → 패닉, 쉬는 연결 3개
    요청 "2" → 하린 (연결 #3), 쉬는 연결 3개
3. 권장: 고칠 수 없는 짝 API는 쓰는 자리에서 DropGuard로 감쌈
    요청 "1" → 서준 (연결 #3), 쉬는 연결 3개
    요청 "" → Empty, 쉬는 연결 3개
    요청 "x" → BadNumber, 쉬는 연결 3개
    요청 "7" → 패닉, 쉬는 연결 3개
    요청 "2" → 하린 (연결 #3), 쉬는 연결 3개
4. 가드를 필드로 가진 타입에는 Drop이 필요 없음
    민지의 세션이 연결 #3을 씀, 쉬는 연결 2개
    세션이 끝난 뒤 쉬는 연결 3개
    세션을 풀어 서준의 연결 #3만 남김, 쉬는 연결 2개
    drop(conn) 뒤 쉬는 연결 3개
```

요청 `"7"`은 범위 검사를 빠뜨린 `query`가 일부러 패닉하는 경우입니다. 예제가 `catch_unwind`로 붙잡아 계속 진행하지만, 기본 패닉 훅이 구역마다 한 번씩 모두 세 번 표준 에러에 다음과 같이 출력합니다. 괄호 안의 스레드 번호는 실행마다 다르고, `note:` 줄은 처음 한 번만 나옵니다.

```text
thread 'main' (스레드 번호) panicked at appd-a/ch11-drop-raii/examples/01_manual_vs_guard.rs:35:34:
index out of bounds: the len is 3 but the index is 7
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

`catch_unwind`에 넘기는 클로저는 `RefCell`을 담은 풀을 빌리므로 `UnwindSafe`가 아니어서 `AssertUnwindSafe`로 감쌌습니다. 되감기 안전성은 [`ch18-dont-panic`](../ch18-dont-panic/README.md)에서 다룹니다.

### 02_narrow_guard_scope — 가드의 범위

같은 점수(도윤 800점)를 세 가지 방식으로 넣고, 알림을 만드는 순간 다른 스레드가 순위표를 잠글 수 있었는지 출력합니다. 관찰용 스레드는 `try_lock` 결과만 돌려주고 바로 합류하므로 출력은 실행마다 같습니다. 4번 구역은 동결 가드를 세 가지로 바인드합니다.

```sh
cargo run -p er-ch11-drop-raii --example 02_narrow_guard_scope
```

```text
1. 비권장: 잠금 가드를 쥔 채 알림까지 만듦
    기록: 도윤 님이 2위가 되었습니다
    알림을 만드는 동안 다른 스레드가 잠글 수 있었나: false
2. 권장: 블록으로 가드의 범위를 좁힘
    기록: 도윤 님이 2위가 되었습니다
    알림을 만드는 동안 다른 스레드가 잠글 수 있었나: true
3. 권장: drop(guard)로 다 쓴 자리에서 풂
    기록: 도윤 님이 2위가 되었습니다 (1위 민지)
    알림을 만드는 동안 다른 스레드가 잠글 수 있었나: true
4. 너무 짧은 범위: 시즌 정산 중에는 늦은 제출을 막아야 합니다
    lb.freeze();              → 정산 중 늦은 제출을 받음: true
    let _ = lb.freeze();      → 정산 중 늦은 제출을 받음: true
    let _frozen = lb.freeze(); → 정산 중 늦은 제출을 받음: false
    정산이 끝난 뒤의 제출을 받음: true
```

`Frozen`의 `drop`은 잠금이 포이즈닝돼 있어도 패닉하지 않도록 `unwrap_or_else(PoisonError::into_inner)`로 잠급니다(2.4절의 "drop에서 패닉을 피한다").

### 03_fallible_release — 실패할 수 있는 해제

같은 메모 3개("우유 사기", "치과 예약", "택배 찾기")를 세 판으로 올립니다. 3번 구역은 성공, 할당량 초과, `?`로 일찍 반환한 경우를 차례로 봅니다. `[안전망]` 줄은 `NoteSync`의 `drop`이 출력합니다.

```sh
cargo run -p er-ch11-drop-raii --example 03_fallible_release
```

```text
1. 비권장: Drop에서만 올림 (할당량 2개)
    편집 3개, 서버에 저장된 메모 0개 — 실패를 알려 준 곳이 없음
2. 비권장: E0509를 clone으로 피한 finish (할당량 10개)
    finish 결과 Ok(3), 서버에 저장된 메모 6개
3. 권장: Result를 돌려주는 finish, Drop은 안전망
    할당량 10개: finish → Ok(3)
    서버에 저장된 메모 3개
    할당량 2개: 실패 (요청 3개, 남은 자리 2개)
    기기에 보관한 메모 ["우유 사기", "치과 예약", "택배 찾기"]
    [안전망] finish 없이 버려짐: 2개 올리기 성공
    ?로 일찍 반환: Err("3번째 줄이 비어 있음")
    서버에 저장된 메모 ["우유 사기", "치과 예약"]
```

2번 구역은 컴파일러가 아무 경고도 하지 않는, 컴파일되지만 틀린 코드입니다. `finish`가 돌려준 `Ok(3)`만 보면 맞아 보입니다.

### 04_drop_order_design — drop 순서와 가드 설계

`[drop]` 줄의 순서를 각 구역의 선언 순서와 나란히 놓고 보세요. 반영된 줄이 모두 문서에 남는 것은 같고, 다른 것은 반영되는 순간 잠금이 있었는가입니다.

```sh
cargo run -p er-ch11-drop-raii --example 04_drop_order_design
```

```text
1. 지역 변수는 선언의 역순으로 버려집니다
    [비권장] 초안을 먼저 만들고 잠금을 나중에 얻음
      [drop] 편집 잠금 해제 (민지)
      [drop] 초안 1줄 반영: 잠금이 풀린 뒤 → 다른 편집자가 끼어들 수 있음
    [권장] 잠금을 먼저 얻고 초안을 나중에 만듦
      [drop] 초안 1줄 반영: 민지의 잠금 안에서
      [drop] 편집 잠금 해제 (민지)
    [권장] 선언 순서를 바꿀 수 없다면 drop으로 순서를 직접 정함
      [drop] 초안 1줄 반영: 민지의 잠금 안에서
      [drop] 편집 잠금 해제 (민지)
2. 구조체 필드는 선언 순서대로 버려집니다 (만든 순서와는 상관없음)
    [비권장] lock 필드를 위에 선언
      잠금 보유자: 서준
      [drop] 편집 잠금 해제 (서준)
      [drop] 초안 1줄 반영: 잠금이 풀린 뒤 → 다른 편집자가 끼어들 수 있음
    [권장] draft 필드를 위에 선언
      잠금 보유자: 서준
      [drop] 초안 1줄 반영: 서준의 잠금 안에서
      [drop] 편집 잠금 해제 (서준)
3. 임시값은 그 문장이 끝날 때 버려집니다
      [drop] 편집 잠금 해제 (하린)
    잠금을 얻은 사람 하린, 다음 문장에서 본 잠금 상태 None
문서에 반영된 줄: ["1안: 표지 제목 수정", "2안: 목차 정리", "3안: 그림 번호 수정", "4안: 참고 문헌 추가", "5안: 맺음말 추가"]
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch11-drop-raii
rustc --edition 2024 compile_fail/e0509_move_out_of_drop_type.rs --out-dir /tmp/er-ch11
rustc --edition 2024 compile_fail/e0502_drop_guard_keeps_borrow.rs --out-dir /tmp/er-ch11
rustc --edition 2024 compile_fail/e0382_use_guard_after_drop.rs --out-dir /tmp/er-ch11
```

### 4.1 `Drop`을 구현한 타입에서 필드를 옮겨 꺼냄 — E0509

파일: [`compile_fail/e0509_move_out_of_drop_type.rs`](compile_fail/e0509_move_out_of_drop_type.rs)

```text
error[E0509]: cannot move out of type `NoteSync<'_>`, which implements the `Drop` trait
  --> compile_fail/e0509_move_out_of_drop_type.rs:26:23
   |
26 |         let pending = self.pending; // E0509
   |                       ^^^^^^^^^^^^
   |                       |
   |                       cannot move out of here
   |                       move occurs because `self.pending` has type `Vec<String>`, which does not implement the `Copy` trait
   |
help: consider borrowing here
   |
26 |         let pending = &self.pending; // E0509
   |                       +
help: consider cloning the value if the performance cost is acceptable
   |
26 |         let pending = self.pending.clone(); // E0509
   |                                   ++++++++
```

- **읽는 법**: `finish(self)`가 `self`를 소유하는데도 "`Drop`을 구현한 타입에서는 옮길 수 없다"고 합니다. 소유했느냐가 아니라 그 타입에 `Drop`이 있느냐가 문제입니다.
- **원인**: `finish`가 끝나면 `self`에 대해 `drop(&mut self)`가 실행됩니다. 필드 하나가 빠져나간 값으로 `drop`을 부를 수는 없으므로 옮기기를 막습니다. 구조 분해(`let NoteSync { server, pending } = self;`)도 같은 이유로 E0509입니다.
- **해결**: 두 `help:`는 따르지 않습니다. 빌리면 옮길 수 없고, 복제하면 2.4절에서 본 대로 `Drop`이 한 번 더 올립니다. `fn finish(mut self)`로 바꾸고 `mem::take(&mut self.pending)`으로 꺼내 빈 `Vec`을 남깁니다(예제 03의 권장 판). 기본값이 없는 필드는 `Option<T>`에 담아 `take()`로 꺼냅니다(예제 01의 `PooledConn`). 애초에 `Drop`이 필요 없는 타입이었다면 `impl Drop`을 지우는 것이 답입니다(2.2절).

### 4.2 가드가 빌린 값을 스코프 안에서 다시 읽음 — E0502

파일: [`compile_fail/e0502_drop_guard_keeps_borrow.rs`](compile_fail/e0502_drop_guard_keeps_borrow.rs)

```text
error[E0502]: cannot borrow `server.notes` as immutable because it is also borrowed as mutable
  --> compile_fail/e0502_drop_guard_keeps_borrow.rs:37:32
   |
33 |         server: &mut server,
   |                 ----------- mutable borrow occurs here
...
37 |     println!("서버에 저장된 메모 {}개", server.notes.len()); // E0502
   |                                         ^^^^^^^^^^^^ immutable borrow occurs here
38 | }
   | - mutable borrow might be used here, when `sync` is dropped and runs the `Drop` code for type `AutoSync`
```

- **읽는 법**: 마지막 줄이 핵심입니다. `sync`를 마지막으로 쓴 것은 36행인데, 대여가 끝나는 곳은 38행의 `}`입니다. 그곳에서 `AutoSync`의 `drop` 코드가 `server`를 빌린 채로 실행되기 때문입니다.
- **원인**: `Drop`이 없는 타입이라면 대여는 마지막 사용에서 끝나고 이 코드는 컴파일됩니다(`impl Drop`을 지워 확인했습니다). `Drop`이 있으면 스코프 끝까지 빌린 상태가 이어집니다(2.3절).
- **해결**: 가드를 블록에 넣어 범위를 좁히거나, `server`를 읽기 전에 `drop(sync)`를 부릅니다. 예제 03의 1번 구역이 블록으로 썼습니다. `finish(self)`처럼 가드를 소비하는 메서드가 있다면 그것을 부른 뒤에 읽어도 됩니다.

### 4.3 `drop(guard)` 뒤에 가드를 다시 씀 — E0382

파일: [`compile_fail/e0382_use_guard_after_drop.rs`](compile_fail/e0382_use_guard_after_drop.rs)

```text
error[E0382]: borrow of moved value: `scores`
  --> compile_fail/e0382_use_guard_after_drop.rs:20:5
   |
16 |     let mut scores = board.lock().unwrap();
   |         ---------- move occurs because `scores` has type `std::sync::MutexGuard<'_, Vec<(&str, i32)>>`, which does not implement the `Copy` trait
17 |     scores.push(("도윤", 800));
18 |     drop(scores); // 잠금 해제
   |          ------ value moved here
19 |     let announcement = format!("참가자 {}명", board.lock().unwrap().len());
20 |     scores.push(("하린", 600)); // E0382
   |     ^^^^^^ value borrowed here after move
```

- **읽는 법**: 흔한 "옮긴 값을 다시 씀" 에러이지만, 옮겨 간 것이 잠금 가드라는 점이 다릅니다. 18행에서 잠금을 풀었으므로 20행은 잠그지 않고 공유 값을 고치려는 코드입니다.
- **원인**: `drop(scores)`는 가드를 `drop` 함수로 옮깁니다. 수동으로 `unlock`하는 언어에서는 이런 코드가 컴파일되어 데이터 경쟁을 만들지만, Rust에서는 풀린 잠금으로 가는 길 자체가 사라집니다.
- **해결**: 잠금 안에서 할 일을 `drop(scores)` 앞으로 옮깁니다. 정말 다시 고쳐야 한다면 `board.lock()`으로 새 가드를 얻습니다. 다만 두 번 잠그는 사이에 다른 스레드가 끼어들 수 있다는 점을 따져 봅니다(2.3절의 "따르지 않아도 되는 경우").

### 4.4 그 밖의 실수

모두 이 툴체인에서 직접 확인했습니다.

| 상황 | 결과 | 바로잡기 |
| --- | --- | --- |
| `guard.drop()`으로 일찍 정리 | E0040 ``explicit use of destructor method`` | `drop(guard)` ([Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 4.3절) |
| `Drop`을 구현한 타입에 `Copy` | E0184 | 자원을 쥔 타입은 `Copy`가 아님 ([Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 4.5절) |
| `impl Drop for X {}` | E0046 ``not all trait items implemented, missing: `drop` `` | `fn drop(&mut self)`를 구현하거나, 할 일이 없으면 `impl Drop`을 지움 |
| `let _ = m.lock().unwrap();` | 컴파일 에러 ``non-binding let on a synchronization lock`` (rustc `let_underscore_lock`, 기본 deny) | `let _guard = ..` ([Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md) 4.5절) |
| 직접 만든 가드를 `let _ =`에 담음 | 경고 없음. 그 자리에서 풀림 | `let _guard = ..`. rustc `let_underscore_drop`(기본 allow)을 켜면 경고 (2.3절) |
| `#[must_use]` 가드를 문장으로 버림 | 경고 `unused ... that must be used`. `help:`는 `let _ = ...`을 제안 | 제안을 따르지 말고 이름을 붙임 (2.3절) |
| 가드의 참조로 `drop(&guard)` | 경고 ``calls to `std::mem::drop` with a reference instead of an owned value does nothing`` (rustc `dropping_references`, 기본 warn). 잠금은 그대로 | 가드 자체를 넘김: `drop(guard)` |

## 5. 연습 문제

### 연습 1. 들여쓰기 가드

트리 구조를 출력하는 `Printer { depth: Cell<usize> }`에 `indent(&self) -> Indent<'_>`를 추가하세요. `Indent`가 살아 있는 동안 출력에 들여쓰기가 한 단계 늘고, 버려지면 원래대로 돌아와야 합니다. 하위 항목을 출력하다가 `?`로 일찍 반환해도 깊이가 원래대로 돌아오는지 확인하세요.

<details>
<summary>힌트</summary>

`indent`는 `depth`를 1 늘리고 `Indent { printer: self }`를 돌려줍니다. `Drop`에서 `depth`를 1 줄입니다. 받는 쪽은 `let _indent = printer.indent();`로 써야 하므로 타입에 `#[must_use = "..."]`를 붙이고 메시지에 이름을 붙이라고 적습니다(2.3절). 재귀 함수 안에서 이 가드를 쓰면 반환 경로가 몇 개든 깊이가 맞습니다. 같은 일을 `DropGuard::new(&printer.depth, |d| d.set(d.get() - 1))`로도 할 수 있는데, 둘 중 어느 쪽이 나은지 2.2절의 기준(여러 곳에서 쓰는가)으로 따져 보세요.

</details>

### 연습 2. 되돌리는 안전망

[`03_fallible_release.rs`](examples/03_fallible_release.rs)의 안전망은 버려질 때 "최선을 다해 올리기"를 합니다. 이번에는 목록에 항목을 바로 넣되, `commit`하지 않고 버려지면 넣은 항목을 모두 되돌리는 `Edit<'a> { list: &'a mut Vec<String>, start_len: usize, .. }`를 만드세요. `?`로 일찍 반환하면 목록이 원래 길이로 돌아와야 합니다.

<details>
<summary>힌트</summary>

`Drop`에서 `self.list.truncate(self.start_len)`으로 되돌립니다. `commit(mut self)`는 `Drop`이 할 일이 없게 만들어야 합니다(2.4절). `committed: bool` 필드를 두고 `commit`에서 `true`로 바꾼 뒤, `Drop`이 그 값을 확인하게 하면 됩니다. `start_len`을 `self.list.len()`으로 바꿔 두는 방법도 있습니다. 데이터베이스 트랜잭션이 이 모양이고, "버려질 때 무엇을 하는 것이 안전한가"는 자원마다 다르다는 점을 예제 03과 비교해 보세요.

</details>

### 연습 3. `DropGuard`로 풀 가드 만들기

[`01_manual_vs_guard.rs`](examples/01_manual_vs_guard.rs)의 `Pool::get`이 `PooledConn` 대신 `DropGuard`를 돌려주게 바꿔 보세요. 그런 다음 4번 구역의 `Session`을 새 `get`으로 다시 쓰려고 하면 무엇이 어려워지는지 확인하세요.

<details>
<summary>힌트</summary>

반환 타입은 `Option<DropGuard<Conn, impl FnOnce(Conn) + '_>>`이고, 본문은 `Some(DropGuard::new(conn, move |c| self.give_back(c)))`입니다(이 툴체인에서 컴파일을 확인했습니다). `Option`과 `#[must_use]`의 관계는 같으므로 `get`에 속성을 붙입니다. 어려워지는 곳은 `Session`의 필드 타입입니다. 클로저의 타입은 이름이 없어서 필드에 적을 수 없으므로, `Session`을 제네릭으로 만들거나 클로저를 `Box<dyn FnOnce(Conn)>`으로 바꿔야 합니다. 여러 곳에서 쓰는 자원에 이름 있는 가드 타입이 나은 이유입니다.

</details>

### 연습 4. 필드 순서에 기대지 않는 해제 순서

[`04_drop_order_design.rs`](examples/04_drop_order_design.rs)의 `SessionLockFirst`(잠금 필드가 위)를 필드 순서는 그대로 두고, 초안이 잠금 안에서 반영되게 고치세요.

<details>
<summary>힌트</summary>

`Draft`에 `apply(&mut self)`를 추가해 `mem::take(&mut self.pending)`으로 꺼낸 줄을 문서에 반영하게 하고, `Draft`의 `drop`은 `pending`이 비어 있으면 아무것도 하지 않게 합니다. 그리고 `impl Drop for SessionLockFirst`에서 `self.draft.apply()`를 부릅니다. 구조체 자신의 `drop`은 필드보다 먼저 실행되므로(Part 0 ch14 2.6절), 그때는 아직 잠금이 살아 있습니다. 대신 `SessionLockFirst`는 이제 구조 분해할 수 없습니다(E0509). 필드 순서 주석 하나와 이 비용 중 무엇을 고를지 따져 보세요.

</details>

## 6. 참고 자료

- 기준 문서: [Item 11: Implement the Drop trait for RAII patterns](https://effective-rust.com/raii.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- 표준 라이브러리 문서
  - [`Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html): Drop order 절, Copy and Drop are exclusive 절, Drop check 절, `drop`의 Panics 절
  - [`std::mem::drop`](https://doc.rust-lang.org/std/mem/fn.drop.html), [`std::mem::DropGuard`](https://doc.rust-lang.org/std/mem/struct.DropGuard.html)(1.100.0), [`std::mem::take`](https://doc.rust-lang.org/std/mem/fn.take.html), [`std::mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html)
  - [`MutexGuard`](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html), [`BufWriter`](https://doc.rust-lang.org/std/io/struct.BufWriter.html)(drop 전에 flush해야 하는 이유), [`IntoInnerError`](https://doc.rust-lang.org/std/io/struct.IntoInnerError.html), [`File`](https://doc.rust-lang.org/std/fs/struct.File.html)(닫을 때의 에러), [`process::exit`](https://doc.rust-lang.org/std/process/fn.exit.html)
- The Reference: [Destructors](https://doc.rust-lang.org/reference/destructors.html)(drop 순서, 임시값 범위)
- rustc 린트: [`let_underscore_drop`](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#let-underscore-drop), [`dropping_references`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#dropping-references)
- 에러 설명: `rustc --explain E0509`, `rustc --explain E0502`, `rustc --explain E0382`, `rustc --explain E0040`
