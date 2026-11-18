// 이 예제가 보여 주는 것: std::ops의 연산자 트레이트 지도.
//                        산술(Add, Sub, Mul)과 단항(Neg), 복합 대입(AddAssign 등),
//                        비트 연산(BitOr, BitAnd, Not)으로 만든 권한 플래그, Index/IndexMut, 범위 타입
// 실행: cargo run -p p1-ch6-core-traits --example 05_operators
//
// Part 0 ch19에서 Add와 Rhs = Self 기본 타입 매개변수를 배웠습니다.
// 다른 연산자도 같은 방식입니다. "연산자 → 트레이트 → 메서드"의 대응만 알면 됩니다.
//   a + b → Add::add(a, b)        a += b → AddAssign::add_assign(&mut a, b)
//   -a    → Neg::neg(a)           !a     → Not::not(a)
//   a | b → BitOr::bitor(a, b)    a[i]   → *Index::index(&a, i)

use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitOr, BitOrAssign, Index, IndexMut, Mul, Neg, Not, RangeBounds,
    RangeInclusive, Sub, SubAssign,
};

// ---------- 1. 금액: 산술 연산자 ----------
#[derive(Debug, Clone, Copy, PartialEq)]
struct Won(i64);

impl fmt::Display for Won {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}원", self.0)
    }
}

impl Add for Won {
    type Output = Won;
    fn add(self, rhs: Won) -> Won {
        Won(self.0 + rhs.0)
    }
}

impl Sub for Won {
    type Output = Won;
    fn sub(self, rhs: Won) -> Won {
        Won(self.0 - rhs.0)
    }
}

// 단항 -: 환불 금액을 만들 때 씁니다.
impl Neg for Won {
    type Output = Won;
    fn neg(self) -> Won {
        Won(-self.0)
    }
}

// 금액 × 수량. Rhs가 Self가 아닌 경우입니다. 금액 × 금액은 뜻이 없으므로 구현하지 않습니다.
impl Mul<i64> for Won {
    type Output = Won;
    fn mul(self, count: i64) -> Won {
        Won(self.0 * count)
    }
}

// 수량 × 금액. 연산자 트레이트는 왼쪽 피연산자의 타입에 구현하므로 순서마다 따로 필요합니다.
// i64는 남의 타입이지만 Mul<Won>의 타입 인자에 내 타입이 있어 고아 규칙이 허용합니다.
impl Mul<Won> for i64 {
    type Output = Won;
    fn mul(self, price: Won) -> Won {
        price * self
    }
}

// 복합 대입 +=, -=는 별도의 트레이트입니다. &mut self를 받아 제자리에서 고칩니다.
impl AddAssign for Won {
    fn add_assign(&mut self, rhs: Won) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Won {
    fn sub_assign(&mut self, rhs: Won) {
        self.0 -= rhs.0;
    }
}

// ---------- 2. 권한 플래그: 비트 연산자 ----------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Perm(u8);

impl Perm {
    const NONE: Perm = Perm(0);
    const READ: Perm = Perm(0b100);
    const WRITE: Perm = Perm(0b010);
    const EXEC: Perm = Perm(0b001);

    fn contains(self, other: Perm) -> bool {
        self & other == other
    }
}

impl BitOr for Perm {
    type Output = Perm;
    fn bitor(self, rhs: Perm) -> Perm {
        Perm(self.0 | rhs.0)
    }
}

impl BitAnd for Perm {
    type Output = Perm;
    fn bitand(self, rhs: Perm) -> Perm {
        Perm(self.0 & rhs.0)
    }
}

impl BitOrAssign for Perm {
    fn bitor_assign(&mut self, rhs: Perm) {
        self.0 |= rhs.0;
    }
}

// !는 Not 트레이트입니다. 쓰지 않는 위쪽 비트까지 켜지지 않도록 아래 3비트만 남깁니다.
impl Not for Perm {
    type Output = Perm;
    fn not(self) -> Perm {
        Perm(!self.0 & 0b111)
    }
}

impl fmt::Display for Perm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let flag = |p: Perm, c: char| if self.contains(p) { c } else { '-' };
        write!(
            f,
            "{}{}{}",
            flag(Perm::READ, 'r'),
            flag(Perm::WRITE, 'w'),
            flag(Perm::EXEC, 'x')
        )
    }
}

// ---------- 3. 게임판: Index / IndexMut ----------
struct Board {
    cells: [[char; 3]; 3],
}

// board[(행, 열)]로 읽기. index는 &Output을 돌려주고, board[..] 표현식은 그것을 역참조한 자리입니다.
impl Index<(usize, usize)> for Board {
    type Output = char;
    fn index(&self, (row, col): (usize, usize)) -> &char {
        &self.cells[row][col]
    }
}

// board[(행, 열)] = 값으로 쓰기. IndexMut은 Index를 전제로 하며 Output 타입을 공유합니다.
impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut char {
        &mut self.cells[row][col]
    }
}

// ---------- 4. 범위 타입 ----------
// a..b, a.., ..b, ..=b, a..=b, ..은 모두 std::ops의 구조체이고, RangeBounds 트레이트를 구현합니다.
fn describe(value: i32, range: impl RangeBounds<i32> + fmt::Debug) -> String {
    format!("{value} in {range:?}: {}", range.contains(&value))
}

fn main() {
    println!("== 1. 산술 연산자 ==");
    let coffee = Won(4500);
    let cake = Won(6000);
    let mut total = coffee * 2 + cake; // Mul<i64> 다음 Add
    println!("주문: {total}"); // 출력: 주문: 15000원
    total -= Won(1000); // SubAssign: 쿠폰
    total += 3 * Won(500); // Mul<Won> for i64, AddAssign: 샷 추가 3번
    println!("결제: {total}"); // 출력: 결제: 15500원
    let refund = -cake; // Neg
    println!("환불: {refund}, 정산: {}", total + refund); // 출력: 환불: -6000원, 정산: 9500원

    println!("\n== 2. 비트 연산자로 만든 권한 ==");
    let mut perm = Perm::READ | Perm::WRITE;
    println!("{perm} 쓰기 가능? {}", perm.contains(Perm::WRITE)); // 출력: rw- 쓰기 가능? true
    perm |= Perm::EXEC;
    println!("{perm}"); // 출력: rwx
    let without_write = perm & !Perm::WRITE; // 쓰기만 끄기
    println!(
        "{without_write} {}",
        without_write == (Perm::READ | Perm::EXEC)
    ); // 출력: r-x true
    println!("{}", Perm::NONE); // 출력: ---

    println!("\n== 3. Index와 IndexMut ==");
    let mut board = Board {
        cells: [['.'; 3]; 3],
    };
    board[(1, 1)] = 'X'; // IndexMut
    board[(0, 2)] = 'O';
    for row in 0..3 {
        let line: String = (0..3).map(|col| board[(row, col)]).collect(); // Index
        println!("{line}");
    }
    // 출력:
    // ..O
    // .X.
    // ...

    println!("\n== 4. 범위도 값입니다 ==");
    let months: RangeInclusive<u32> = 1..=12;
    println!(
        "{}~{}, 12 포함? {}",
        months.start(),
        months.end(),
        months.contains(&12)
    ); // 출력: 1~12, 12 포함? true
    let r = 3..7;
    println!("{}", r == std::ops::Range { start: 3, end: 7 }); // 출력: true
    println!("{}", describe(7, 3..7)); // 출력: 7 in 3..7: false
    println!("{}", describe(7, 3..=7)); // 출력: 7 in 3..=7: true
    println!("{}", describe(-5, ..0)); // 출력: -5 in ..0: true
    println!("{}", describe(100, 10..)); // 출력: 100 in 10..: true
}
