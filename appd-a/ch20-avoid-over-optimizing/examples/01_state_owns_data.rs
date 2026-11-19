//! 지침 1: 상태로 보관하는 타입은 데이터를 소유한다.
//!
//! 실행: cargo run -p er-ch20-avoid-over-optimizing --example 01_state_owns_data
//!
//! 경매의 입찰 기록("입찰자,금액")을 한 줄씩 읽어 최고 입찰을 기억합니다. 입찰자 이름을
//! 입력에서 빌려 두는 판(`AuctionView<'a>`)과 복사해 소유하는 판(`Auction`)을 같은 입력으로
//! 돌려, 복사를 아낀 대가로 부르는 쪽이 무엇을 잃는지 비교합니다. 두 판 모두 컴파일되고
//! 결과도 같습니다. 다른 것은 "그 값을 가지고 무엇을 할 수 있는가"입니다.

use std::io::{self, BufRead, Read};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

/// 경매 물품과 입찰 기록. 실제 프로그램이라면 파일이나 소켓에서 읽을 내용입니다.
const ROUNDS: [(&str, &str); 3] = [
    (
        "자전거",
        "민준,120000\n서연,150000\n도윤,90000\n하은,182000\n지호,175000\n민준,181000\n",
    ),
    ("책상", "도윤,30000\n지호,42000\n서연,41000\n"),
    (
        "카메라",
        "하은,250000\n민준,260000\n서연,255000\n도윤,310000\n",
    ),
];

/// 두 판이 함께 쓰는 줄 나누기. 돌려주는 이름은 입력 줄의 일부를 가리킵니다.
fn split_bid(line: &str) -> Option<(&str, u32)> {
    let (bidder, amount) = line.trim().split_once(',')?;
    Some((bidder.trim(), amount.trim().parse().ok()?))
}

fn describe(bidder: &str, amount: u32, bids: usize) -> String {
    format!("{bidder} {amount}원 (입찰 {bids}건)")
}

// ---- 빌린 판: 이름을 입력에서 빌립니다 ----

#[derive(Clone, Copy)]
struct BidView<'a> {
    bidder: &'a str, // 입력 줄의 일부. 복사가 없습니다.
    amount: u32,
}

/// 'a가 경매 상태 전체에 붙습니다. 이 값을 담는 타입과 함수의 시그니처에도 따라 번집니다.
#[derive(Default)]
struct AuctionView<'a> {
    best: Option<BidView<'a>>,
    bids: usize,
}

impl<'a> AuctionView<'a> {
    /// 줄을 'a 동안 빌려 받습니다. 최고 입찰이 바뀌면 그 줄의 일부를 그대로 보관합니다.
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
        match self.best {
            Some(b) => describe(b.bidder, b.amount, self.bids),
            None => String::from("입찰 없음"),
        }
    }
}

// ---- 소유한 판: 이름을 복사해 가집니다 ----

// 복사가 몇 번, 몇 바이트 일어났는지 셉니다(지침 2). 세는 것 말고는 하는 일이 없습니다.
static NAME_COPIES: AtomicUsize = AtomicUsize::new(0);
static NAME_BYTES: AtomicUsize = AtomicUsize::new(0);

fn copy_name(name: &str) -> String {
    NAME_COPIES.fetch_add(1, Ordering::Relaxed);
    NAME_BYTES.fetch_add(name.len(), Ordering::Relaxed);
    name.to_string() // 눈에 보이는 복사: 새 버퍼를 할당하고 이름을 옮겨 적습니다.
}

fn take_copy_counts() -> (usize, usize) {
    (
        NAME_COPIES.swap(0, Ordering::Relaxed),
        NAME_BYTES.swap(0, Ordering::Relaxed),
    )
}

struct Bid {
    bidder: String,
    amount: u32,
}

impl Bid {
    /// 가장 단순한 소유 설계: 줄마다 이름을 복사해 `Bid`를 만듭니다.
    fn parse(line: &str) -> Option<Bid> {
        let (bidder, amount) = split_bid(line)?;
        Some(Bid {
            bidder: copy_name(bidder),
            amount,
        })
    }
}

/// 라이프타임 매개변수가 없습니다. 입력이 사라져도 이 값은 그대로 쓸 수 있습니다.
#[derive(Default)]
struct Auction {
    best: Option<Bid>,
    bids: usize,
}

impl Auction {
    fn offer(&mut self, bid: Bid) {
        self.bids += 1;
        if self.best.as_ref().is_none_or(|b| bid.amount > b.amount) {
            self.best = Some(bid); // 지는 입찰의 이름은 여기서 버려집니다.
        }
    }

    fn summary(&self) -> String {
        match &self.best {
            Some(b) => describe(&b.bidder, b.amount, self.bids),
            None => String::from("입찰 없음"),
        }
    }
}

/// 입력을 끝까지 읽어 경매 결과를 돌려줍니다. 줄 버퍼 하나를 계속 다시 씁니다.
/// 빌린 판으로는 이 함수를 만들 수 없습니다. 줄 버퍼를 다시 쓰면 E0502이고
/// (compile_fail/e0502_reuse_line_buffer.rs), 입력을 지역 String에 통째로 읽어 두면
/// 그 String을 빌린 경매를 돌려줄 수 없어 E0515입니다.
fn run_auction(mut input: impl BufRead) -> io::Result<Auction> {
    let mut auction = Auction::default();
    let mut line = String::new();
    while input.read_line(&mut line)? > 0 {
        if let Some(bid) = Bid::parse(&line) {
            auction.offer(bid);
        }
        line.clear();
    }
    Ok(auction)
}

fn main() -> io::Result<()> {
    let (item, record) = ROUNDS[0];

    println!("1. 빌린 판: 복사는 없지만, 입력 전체를 경매보다 오래 살려 둬야 합니다");
    {
        // 줄을 읽을 때마다 버퍼를 비우면 빌려 둔 이름이 사라지므로, 입력을 통째로 읽어 둡니다.
        let mut text = String::new();
        record.as_bytes().read_to_string(&mut text)?;
        let mut auction = AuctionView::default();
        for line in text.lines() {
            auction.offer(line);
        }
        println!("    {item}: {}", auction.summary()); // 출력:     자전거: 하은 182000원 (입찰 6건)
        println!(
            "    복사한 이름 0번, 살려 둔 입력 {}줄 {}바이트",
            text.lines().count(),
            text.len()
        ); // 출력:     복사한 이름 0번, 살려 둔 입력 6줄 83바이트
    } // text와 auction이 함께 버려집니다. auction만 블록 밖으로 내보내면 E0597입니다.

    println!("2. 소유한 판: 이름을 복사하고, 줄 버퍼 하나를 다시 쓰며 읽습니다");
    let auction = run_auction(record.as_bytes())?;
    let (copies, bytes) = take_copy_counts();
    println!("    {item}: {}", auction.summary()); // 출력:     자전거: 하은 182000원 (입찰 6건)
    println!("    복사한 이름 {copies}번 {bytes}바이트, 다시 쓴 줄 버퍼 1개"); // 출력:     복사한 이름 6번 36바이트, 다시 쓴 줄 버퍼 1개

    println!("3. 소유한 판이라서 할 수 있는 일: 돌려받고, 모아 두고, 다른 스레드로 넘기기");
    // 경매마다 run_auction이 결과를 돌려주고, 입력 문자열은 그 안에서 버려집니다.
    let auctions = ROUNDS
        .iter()
        .map(|(_, record)| run_auction(record.as_bytes()))
        .collect::<io::Result<Vec<Auction>>>()?;
    // 모은 결과를 정산 스레드로 옮깁니다. Auction은 빌린 것이 없어 'static 바운드를 만족합니다.
    let settle = thread::spawn(move || auctions.iter().map(Auction::summary).collect::<Vec<_>>());
    let settled = settle.join().expect("정산 스레드는 패닉하지 않습니다");
    for ((item, _), line) in ROUNDS.iter().zip(&settled) {
        println!("    {item}: {line}");
    }
    // 출력:     자전거: 하은 182000원 (입찰 6건)
    // 출력:     책상: 지호 42000원 (입찰 3건)
    // 출력:     카메라: 도윤 310000원 (입찰 4건)
    let (copies, bytes) = take_copy_counts();
    println!("    복사한 이름 {copies}번 {bytes}바이트"); // 출력:     복사한 이름 13번 78바이트
    Ok(())
}
