// 기대하는 에러: E0502 (두 번)
// 핵심 메시지: cannot borrow `line` as mutable because it is also borrowed as immutable
//
// 빌린 판 AuctionView<'a>는 최고 입찰자의 이름을 입력 줄에서 빌려 보관합니다. 그런데 이 코드는
// 줄 버퍼 `line` 하나를 계속 다시 씁니다. 다음 줄을 읽으려고 `read_line(&mut line)`을 부르거나
// `line.clear()`로 비우는 순간, 경매가 빌려 둔 이름이 지워지거나 바뀔 수 있으므로 거부됩니다.
// 에러가 "그 뒤에 쓰는 곳"으로 가리키는 것은 다음 반복의 `auction.offer`입니다. 경매가 앞
// 반복에서 빌린 이름을 들고 있는 채로 다시 쓰이므로, 대여가 루프 전체에 걸쳐 이어집니다.
//
// 고치는 방법: 경매가 이름을 소유하게 합니다(examples/01_state_owns_data.rs의 Auction과
// run_auction). 입력 전체를 String 하나에 담아 경매보다 오래 살려 두는 방법도 있지만
// (같은 예제의 1번 구역), 그러면 줄 버퍼를 다시 쓰는 장점이 사라집니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0502_reuse_line_buffer.rs --out-dir /tmp/er-ch20

use std::io::BufRead;

const RECORD: &str = "민준,120000\n서연,150000\n도윤,90000\n";

fn split_bid(line: &str) -> Option<(&str, u32)> {
    let (bidder, amount) = line.trim().split_once(',')?;
    Some((bidder.trim(), amount.trim().parse().ok()?))
}

#[derive(Clone, Copy)]
struct BidView<'a> {
    bidder: &'a str,
    amount: u32,
}

#[derive(Default)]
struct AuctionView<'a> {
    best: Option<BidView<'a>>,
}

impl<'a> AuctionView<'a> {
    fn offer(&mut self, line: &'a str) {
        let Some((bidder, amount)) = split_bid(line) else {
            return;
        };
        if self.best.is_none_or(|b| amount > b.amount) {
            self.best = Some(BidView { bidder, amount });
        }
    }
}

fn main() {
    let mut input = RECORD.as_bytes();
    let mut auction = AuctionView::default();
    let mut line = String::new(); // 줄 버퍼 하나를 계속 다시 씁니다
    while input.read_line(&mut line).unwrap_or(0) > 0 { // E0502: 다음 줄을 읽으려고 버퍼를 빌림
        auction.offer(&line); // 경매가 버퍼를 빌려 이름을 보관
        line.clear(); // E0502: 빌려 준 버퍼를 비움
    }
    if let Some(best) = auction.best {
        println!("최고 입찰: {} {}원", best.bidder, best.amount); // 루프 뒤에 빌린 이름을 씀
    }
}
