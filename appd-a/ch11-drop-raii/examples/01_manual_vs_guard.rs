// 지침 1: 해제해야 하는 자원은 "얻기·돌려주기" 짝을 사람이 맞추지 말고 가드 타입에 맡깁니다.
//
// 데이터베이스 연결 풀(연결 3개)에서 연결을 빌려 요청을 처리합니다. 빌린 연결을 풀에 돌려주지
// 않으면 연결은 그대로 사라지고, 풀이 바닥나면 정상 요청까지 처리하지 못합니다.
//
//   1. 비권장: take()로 빌리고 give_back()으로 돌려주는 수동 짝 — 조기 반환, ?, 패닉에서 빠뜨립니다.
//   2. 권장: get()이 가드를 돌려주고, 가드가 버려질 때 연결을 돌려줍니다.
//   3. 권장: 짝으로 된 API를 고칠 수 없으면 쓰는 자리에서 DropGuard(1.100.0)로 감쌉니다.
//   4. 가드를 필드로 가진 타입에는 Drop을 구현하지 않습니다. 필드가 알아서 돌려줍니다.
//
// 1~3번 구역의 "7" 요청은 일부러 패닉합니다. catch_unwind로 붙잡으므로 종료 코드는 0이지만,
// 표준 에러에 패닉 메시지가 세 번 출력됩니다.
//
// 실행: cargo run -p er-ch11-drop-raii --example 01_manual_vs_guard

use std::cell::RefCell;
use std::mem::DropGuard;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use std::panic::{self, AssertUnwindSafe};

const USERS: [&str; 3] = ["민지", "서준", "하린"];
const REQUESTS: [&str; 5] = ["1", "", "x", "7", "2"];

/// 연결은 상태(처리한 질의 수)를 가지므로 질의에는 &mut가 필요합니다.
struct Conn {
    id: u32,
    queries: u32,
}

impl Conn {
    /// 사용자 이름을 찾습니다. 범위 검사를 빠뜨린 버그가 있어 없는 번호면 패닉합니다.
    fn query(&mut self, user: usize) -> String {
        self.queries += 1;
        format!("{} (연결 #{})", USERS[user], self.id)
    }
}

struct Pool {
    idle: RefCell<Vec<Conn>>,
}

impl Pool {
    fn new(size: u32) -> Pool {
        let conns = (1..=size).map(|id| Conn { id, queries: 0 }).collect();
        Pool {
            idle: RefCell::new(conns),
        }
    }

    fn idle_count(&self) -> usize {
        self.idle.borrow().len()
    }

    /// 수동 짝의 "얻기". 돌려주는 것은 부르는 쪽의 책임입니다.
    fn take(&self) -> Option<Conn> {
        self.idle.borrow_mut().pop()
    }

    /// 수동 짝의 "돌려주기".
    fn give_back(&self, conn: Conn) {
        self.idle.borrow_mut().push(conn);
    }

    /// 가드를 돌려주는 "얻기". 돌려주는 일은 가드의 Drop이 맡습니다.
    // 가드가 Option에 싸여 나오면 타입에 붙인 #[must_use]가 닿지 않으므로 메서드에도 붙입니다.
    #[must_use = "버리면 연결이 바로 풀로 돌아갑니다"]
    fn get(&self) -> Option<PooledConn<'_>> {
        let conn = self.take()?;
        Some(PooledConn {
            pool: self,
            conn: Some(conn),
        })
    }
}

/// 빌린 연결의 가드. 가드가 있는 동안만 연결에 닿을 수 있습니다(Deref/DerefMut).
// 가드를 문장으로 버리는 실수에 경고가 나도록 #[must_use]를 붙입니다(02 예제 참고).
#[must_use = "버리면 연결이 바로 풀로 돌아갑니다"]
struct PooledConn<'a> {
    pool: &'a Pool,
    // Drop::drop은 &mut self를 받으므로 Conn을 값으로 꺼낼 수 없습니다.
    // Option에 담아 두고 drop에서 take()로 꺼냅니다(README 2.4절).
    conn: Option<Conn>,
}

impl Deref for PooledConn<'_> {
    type Target = Conn;
    fn deref(&self) -> &Conn {
        self.conn.as_ref().expect("drop 전에는 언제나 Some")
    }
}

impl DerefMut for PooledConn<'_> {
    fn deref_mut(&mut self) -> &mut Conn {
        self.conn.as_mut().expect("drop 전에는 언제나 Some")
    }
}

impl Drop for PooledConn<'_> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.pool.give_back(conn);
        }
    }
}

#[derive(Debug)]
enum RequestError {
    NoConn,
    Empty,
    BadNumber,
}

impl From<ParseIntError> for RequestError {
    fn from(_: ParseIntError) -> Self {
        RequestError::BadNumber
    }
}

/// 비권장: 처음에는 take와 give_back의 짝이 맞았지만, 검사와 파싱이 더해지며 빠지는 길이 생겼습니다.
fn handle_manual(pool: &Pool, req: &str) -> Result<String, RequestError> {
    let mut conn = pool.take().ok_or(RequestError::NoConn)?;
    if req.is_empty() {
        return Err(RequestError::Empty); // 돌려주기를 빠뜨림
    }
    let user: usize = req.parse()?; // 파싱에 실패하면 여기서 반환: 돌려주기를 빠뜨림
    let name = conn.query(user); // 패닉하면 되감기로 빠져나감: 돌려주기를 빠뜨림
    pool.give_back(conn);
    Ok(name)
}

/// 권장: 가드가 함수의 모든 출구에서 연결을 돌려줍니다. 본문은 수동판에서 give_back만 뺀 것입니다.
fn handle_guarded(pool: &Pool, req: &str) -> Result<String, RequestError> {
    let mut conn = pool.get().ok_or(RequestError::NoConn)?;
    if req.is_empty() {
        return Err(RequestError::Empty);
    }
    let user: usize = req.parse()?;
    Ok(conn.query(user))
}

/// 권장: Pool의 API를 고칠 수 없을 때. 얻자마자 DropGuard로 감싸 "돌려주기"를 예약합니다.
fn handle_with_drop_guard(pool: &Pool, req: &str) -> Result<String, RequestError> {
    let conn = pool.take().ok_or(RequestError::NoConn)?;
    let mut conn = DropGuard::new(conn, |c| pool.give_back(c));
    if req.is_empty() {
        return Err(RequestError::Empty);
    }
    let user: usize = req.parse()?;
    Ok(conn.query(user)) // DropGuard도 DerefMut으로 안의 Conn을 빌려 줍니다.
}

fn run_requests(handler: fn(&Pool, &str) -> Result<String, RequestError>) {
    let pool = Pool::new(3);
    for req in REQUESTS {
        // Pool은 RefCell을 담고 있어 UnwindSafe가 아닙니다. 이 예제는 패닉 뒤에 풀의 상태를
        // 확인하려는 것이므로 AssertUnwindSafe로 감쌉니다(되감기 안전성은 ch18-dont-panic).
        let result = panic::catch_unwind(AssertUnwindSafe(|| handler(&pool, req)));
        let shown = match result {
            Ok(Ok(name)) => name,
            Ok(Err(e)) => format!("{e:?}"),
            Err(_) => String::from("패닉"),
        };
        println!(
            "    요청 {req:?} → {shown}, 쉬는 연결 {}개",
            pool.idle_count()
        );
    }
}

/// 가드를 필드로 가진 타입. Drop을 구현하지 않아도 conn 필드가 버려질 때 연결이 돌아갑니다.
struct Session<'a> {
    conn: PooledConn<'a>,
    user: String,
}

fn main() {
    println!("1. 비권장: 수동으로 짝을 맞춤 (take / give_back)");
    run_requests(handle_manual);
    // 출력:     요청 "1" → 서준 (연결 #3), 쉬는 연결 3개
    // 출력:     요청 "" → Empty, 쉬는 연결 2개
    // 출력:     요청 "x" → BadNumber, 쉬는 연결 1개
    // 출력:     요청 "7" → 패닉, 쉬는 연결 0개
    // 출력:     요청 "2" → NoConn, 쉬는 연결 0개

    println!("2. 권장: get()이 돌려주는 가드에 맡김");
    run_requests(handle_guarded);
    // 출력:     요청 "1" → 서준 (연결 #3), 쉬는 연결 3개
    // 출력:     요청 "" → Empty, 쉬는 연결 3개
    // 출력:     요청 "x" → BadNumber, 쉬는 연결 3개
    // 출력:     요청 "7" → 패닉, 쉬는 연결 3개
    // 출력:     요청 "2" → 하린 (연결 #3), 쉬는 연결 3개

    println!("3. 권장: 고칠 수 없는 짝 API는 쓰는 자리에서 DropGuard로 감쌈");
    run_requests(handle_with_drop_guard);
    // 출력:     요청 "1" → 서준 (연결 #3), 쉬는 연결 3개
    // 출력:     요청 "" → Empty, 쉬는 연결 3개
    // 출력:     요청 "x" → BadNumber, 쉬는 연결 3개
    // 출력:     요청 "7" → 패닉, 쉬는 연결 3개
    // 출력:     요청 "2" → 하린 (연결 #3), 쉬는 연결 3개

    println!("4. 가드를 필드로 가진 타입에는 Drop이 필요 없음");
    let pool = Pool::new(3);
    {
        let session = Session {
            conn: pool.get().expect("연결 3개 중 하나"),
            user: String::from("민지"),
        };
        println!(
            "    {}의 세션이 연결 #{}을 씀, 쉬는 연결 {}개",
            session.user,
            session.conn.id,
            pool.idle_count()
        );
        // 출력:     민지의 세션이 연결 #3을 씀, 쉬는 연결 2개
    } // session이 버려지며 필드 conn(PooledConn)의 Drop이 연결을 돌려줍니다.
    println!("    세션이 끝난 뒤 쉬는 연결 {}개", pool.idle_count());
    // 출력:     세션이 끝난 뒤 쉬는 연결 3개

    // Drop을 구현하지 않았으므로 구조 분해로 필드를 꺼낼 수 있습니다.
    // Session에 Drop을 구현하면 이 줄은 E0509가 됩니다(compile_fail/e0509_move_out_of_drop_type.rs).
    let session = Session {
        conn: pool.get().expect("연결 3개 중 하나"),
        user: String::from("서준"),
    };
    let Session { conn, user } = session;
    println!(
        "    세션을 풀어 {user}의 연결 #{}만 남김, 쉬는 연결 {}개",
        conn.id,
        pool.idle_count()
    );
    // 출력:     세션을 풀어 서준의 연결 #3만 남김, 쉬는 연결 2개
    drop(conn);
    println!("    drop(conn) 뒤 쉬는 연결 {}개", pool.idle_count());
    // 출력:     drop(conn) 뒤 쉬는 연결 3개
}
