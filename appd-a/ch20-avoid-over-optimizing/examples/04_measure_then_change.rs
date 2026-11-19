//! 지침 4: 바꾸기 전에 재고, 측정이 가리키는 곳만 바꾼다.
//!
//! 실행(두 빌드로 모두 실행해 비교합니다):
//!   cargo run -p er-ch20-avoid-over-optimizing --example 04_measure_then_change
//!   cargo run -p er-ch20-avoid-over-optimizing --example 04_measure_then_change --release
//!
//! 예제 01의 경매를 입찰 10만 줄로 키워 세 판의 경과 시간을 잽니다.
//! - 소유한 판: 예제 01의 단순한 설계. 줄마다 이름을 복사해 `Bid`를 만듭니다.
//! - 빌린 판: 예제 01의 `AuctionView<'a>`. 복사가 없지만 라이프타임이 번집니다.
//! - 경계판: 잰 뒤에 바꾼 판. 줄은 빌려서 훑고, 최고 입찰이 바뀔 때만 이름을 복사합니다.
//!   `Auction` 타입은 그대로이므로 라이프타임 매개변수가 없습니다.
//!
//! 실행마다 달라지는 줄은 2번 구역의 시간 세 줄뿐입니다. 입력은 시드를 고정해 만들므로
//! 결과와 복사 횟수는 언제나 같습니다.

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::{Duration, Instant};

const NAMES: [&str; 8] = [
    "민준", "서연", "도윤", "하은", "지호", "수아", "예준", "지유",
];
const LINES: u32 = 100_000;
const RUNS: u32 = 5;

/// "입찰자,금액" 줄을 LINES개 만듭니다. 시드를 고정한 간단한 의사 난수라 실행마다 같습니다.
fn make_record() -> String {
    let mut seed: u32 = 20;
    let mut text = String::new();
    for i in 0..LINES {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        let amount = 10_000 + (seed >> 8) % 990_000;
        let name = NAMES[i as usize % NAMES.len()];
        writeln!(text, "{name},{amount}").expect("String에 쓰기는 실패하지 않습니다");
    }
    text
}

fn split_bid(line: &str) -> Option<(&str, u32)> {
    let (bidder, amount) = line.trim().split_once(',')?;
    Some((bidder.trim(), amount.trim().parse().ok()?))
}

fn describe(bidder: &str, amount: u32, bids: usize) -> String {
    format!("{bidder} {amount}원 (입찰 {bids}건)")
}

// ---- 빌린 판(예제 01과 같음) ----

#[derive(Clone, Copy)]
struct BidView<'a> {
    bidder: &'a str,
    amount: u32,
}

#[derive(Default)]
struct AuctionView<'a> {
    best: Option<BidView<'a>>,
    bids: usize,
}

impl<'a> AuctionView<'a> {
    fn offer(&mut self, line: &'a str) {
        let Some((bidder, amount)) = split_bid(line) else {
            return;
        };
        self.bids += 1;
        if self.best.is_none_or(|b| amount > b.amount) {
            self.best = Some(BidView { bidder, amount });
        }
    }

    fn summary(&self) -> String {
        self.best
            .map_or_else(String::new, |b| describe(b.bidder, b.amount, self.bids))
    }
}

// ---- 소유한 판(예제 01과 같음)과, 메서드 하나를 더한 경계판 ----

struct Bid {
    bidder: String,
    amount: u32,
}

impl Bid {
    fn parse(line: &str) -> Option<Bid> {
        let (bidder, amount) = split_bid(line)?;
        Some(Bid {
            bidder: bidder.to_string(), // 줄마다 복사
            amount,
        })
    }
}

#[derive(Default)]
struct Auction {
    best: Option<Bid>,
    bids: usize,
}

impl Auction {
    /// 소유한 판이 쓰는 메서드: 이미 복사된 `Bid`를 받습니다.
    fn offer(&mut self, bid: Bid) {
        self.bids += 1;
        if self.best.as_ref().is_none_or(|b| bid.amount > b.amount) {
            self.best = Some(bid);
        }
    }

    /// 경계판이 쓰는 메서드: 줄을 빌려 받고, 보관할 때만 복사합니다.
    /// 최고 입찰이 바뀌었으면(그래서 이름을 복사했으면) true를 돌려줍니다.
    fn offer_line(&mut self, line: &str) -> bool {
        let Some((bidder, amount)) = split_bid(line) else {
            return false;
        };
        self.bids += 1;
        if self.best.as_ref().is_none_or(|b| amount > b.amount) {
            self.best = Some(Bid {
                bidder: bidder.to_string(), // 보관하는 순간에만 복사
                amount,
            });
            return true;
        }
        false
    }

    fn summary(&self) -> String {
        self.best
            .as_ref()
            .map_or_else(String::new, |b| describe(&b.bidder, b.amount, self.bids))
    }
}

// 세 판의 실행 함수. 복사 횟수는 지역 변수로 세어 함께 돌려줍니다.

fn run_owned(text: &str) -> (Auction, usize) {
    let mut auction = Auction::default();
    let mut copies = 0;
    for line in text.lines() {
        if let Some(bid) = Bid::parse(line) {
            copies += 1;
            auction.offer(bid);
        }
    }
    (auction, copies)
}

fn run_view(text: &str) -> AuctionView<'_> {
    let mut auction = AuctionView::default();
    for line in text.lines() {
        auction.offer(line);
    }
    auction
}

fn run_boundary(text: &str) -> (Auction, usize) {
    let mut auction = Auction::default();
    let mut copies = 0;
    for line in text.lines() {
        if auction.offer_line(line) {
            copies += 1;
        }
    }
    (auction, copies)
}

/// `work`를 RUNS번 실행해 가장 짧은 경과 시간을 돌려줍니다. 다른 프로그램이 끼어든 측정은
/// 길어지기만 하므로, 가장 짧은 값이 그 코드의 비용에 가장 가깝습니다.
/// 결과를 `black_box`에 넘겨, 쓰이지 않는 결과라며 계산을 지우는 최적화를 막습니다.
fn fastest<T>(mut work: impl FnMut() -> T) -> Duration {
    (0..RUNS)
        .map(|_| {
            let start = Instant::now();
            black_box(work());
            start.elapsed()
        })
        .min()
        .unwrap_or_default()
}

fn report(label: &str, took: Duration) {
    let per_line = took.as_secs_f64() * 1e9 / f64::from(LINES);
    println!("    {label}: {took:.2?} (줄당 {per_line:.1}ns)");
}

fn main() {
    // 디버그 빌드에서는 debug_assertions가 켜져 있습니다. 성능은 릴리스 빌드에서 잽니다.
    if cfg!(debug_assertions) {
        println!("빌드: 디버그(최적화 꺼짐)"); // 출력: 빌드: 디버그(최적화 꺼짐)
    } else {
        println!("빌드: 릴리스(최적화 켜짐)"); // 출력(--release): 빌드: 릴리스(최적화 켜짐)
    }

    println!("1. 재기 전에: 세 판의 결과가 같은지, 복사가 몇 번인지 확인합니다");
    let text = make_record();
    println!("    입찰 {LINES}줄, 입력 {}바이트", text.len()); // 출력:     입찰 100000줄, 입력 1390971바이트
    let (owned, owned_copies) = run_owned(&text);
    let view = run_view(&text);
    let (boundary, boundary_copies) = run_boundary(&text);
    println!(
        "    소유한 판: {}, 이름 복사 {owned_copies}번",
        owned.summary()
    ); // 출력:     소유한 판: 지호 999991원 (입찰 100000건), 이름 복사 100000번
    println!("    빌린 판: {}, 이름 복사 0번", view.summary()); // 출력:     빌린 판: 지호 999991원 (입찰 100000건), 이름 복사 0번
    println!(
        "    경계판: {}, 이름 복사 {boundary_copies}번",
        boundary.summary()
    ); // 출력:     경계판: 지호 999991원 (입찰 100000건), 이름 복사 15번
    let same = owned.summary() == view.summary() && view.summary() == boundary.summary();
    println!("    세 판의 결과가 같은가: {same}"); // 출력:     세 판의 결과가 같은가: true

    println!("2. 같은 일을 {RUNS}번씩 하고 가장 짧은 시간을 봅니다(실행마다 다릅니다)");
    // 입력도 black_box로 감싸, 입력을 안다고 가정한 최적화를 막습니다.
    // 아래 세 줄의 <시간>은 실행마다, 빌드마다 다릅니다. 실제로 잰 값은 README의 3절에 있습니다.
    report("소유한 판", fastest(|| run_owned(black_box(&text)))); // 출력:     소유한 판: <시간> (줄당 <시간>)
    report("빌린 판", fastest(|| run_view(black_box(&text)))); // 출력:     빌린 판: <시간> (줄당 <시간>)
    report("경계판", fastest(|| run_boundary(black_box(&text)))); // 출력:     경계판: <시간> (줄당 <시간>)
}
