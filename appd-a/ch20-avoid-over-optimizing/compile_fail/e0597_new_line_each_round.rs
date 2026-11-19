// 기대하는 에러: E0597
// 핵심 메시지: `line` does not live long enough
//
// e0502_reuse_line_buffer.rs에서 막히자 이번에는 줄마다 새 String을 만들어 읽습니다. 버퍼를
// 다시 쓰지 않으니 E0502는 사라지지만, 그 String은 반복 한 번이 끝나면 drop됩니다.
// 빌린 판 AuctionView<'a>가 그 줄의 이름을 다음 반복(에러가 가리키는 `auction.offer`)과 루프
// 밖까지 들고 가므로 "빌린 값이 충분히 오래 살지 않는다"로 거부됩니다. 버퍼를 다시 써도,
// 새로 만들어도 빌린 판으로는 줄 단위로 읽을 수 없습니다.
//
// 고치는 방법: e0502_reuse_line_buffer.rs와 같습니다. 경매가 이름을 소유하게 합니다
// (examples/01_state_owns_data.rs의 Auction).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0597_new_line_each_round.rs --out-dir /tmp/er-ch20

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
    loop {
        let mut line = String::new(); // 줄마다 새 버퍼
        if input.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        auction.offer(&line); // E0597: 이번 반복에서만 사는 줄을 경매가 빌림
    } // line이 여기서 drop됩니다
    if let Some(best) = auction.best {
        println!("최고 입찰: {} {}원", best.bidder, best.amount); // 루프 뒤에 빌린 이름을 씀
    }
}
