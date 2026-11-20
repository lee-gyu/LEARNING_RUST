//! 퍼즈 테스트의 흉내: 고정 시드의 의사 난수로 입력을 많이 만들어 `parse`에 넣습니다.
//!
//! 확인하는 성질
//! 1. 어떤 바이트열에도 `parse`가 패닉하지 않는다(패닉은 서비스 거부가 될 수 있습니다).
//! 2. `parse`가 성공한 입력은 `encode`로 다시 기록하면 같은 바이트가 된다.
//! 3. 아무 레코드 목록이나 `encode`한 뒤 `parse`하면 같은 목록이 된다.
//!
//! 진짜 퍼저(cargo-fuzz, AFL)는 실행 중에 어느 분기를 지났는지 보고 다음 입력을 고르는
//! "커버리지 유도" 방식이고, 끝없이 돕니다. 이 파일은 정해진 개수의 입력을 무작위로 만들 뿐이라
//! 몇 초 안에 끝나고 결과가 늘 같습니다. 그래서 일반 테스트처럼 CI에서 매번 돌릴 수 있습니다.

use std::panic;

use tinytlv::{Record, encode, parse};

/// xorshift64*: 시드가 같으면 늘 같은 수열을 냅니다. 실패한 입력을 다시 만들 수 있게 시드를 고정합니다.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap()
    }

    fn byte(&mut self) -> u8 {
        self.next().to_be_bytes()[0]
    }

    fn bytes(&mut self, max_len: usize) -> Vec<u8> {
        let len = self.below(max_len + 1);
        (0..len).map(|_| self.byte()).collect()
    }
}

const SEED: u64 = 0x5eed_2026_1120;
const CASES: usize = 20_000;

/// `parse`를 부르되, 패닉하면 그 입력을 메시지에 담아 실패합니다.
/// 퍼저가 패닉을 일으킨 입력을 파일로 남기는 것과 같은 역할입니다.
fn parse_or_report(case: usize, input: &[u8]) -> Option<Vec<Record>> {
    match panic::catch_unwind(|| parse(input)) {
        Ok(result) => result.ok(),
        Err(_) => panic!("{case}번째 입력에서 parse가 패닉했습니다: {input:?}"),
    }
}

#[test]
fn random_bytes_never_panic_and_round_trip() {
    let mut rng = XorShift(SEED);
    for case in 0..CASES {
        let input = rng.bytes(24);
        if let Some(records) = parse_or_report(case, &input) {
            assert_eq!(
                encode(&records),
                input,
                "{case}번째 입력이 왕복하지 않습니다"
            );
        }
    }
}

/// 완전히 무작위인 바이트는 길이 바이트가 대개 커서 앞쪽에서 바로 에러가 납니다.
/// 올바른 메시지를 만든 뒤 한 곳을 바꾸거나 잘라 내면, 더 깊은 곳까지 읽는 입력이 됩니다.
#[test]
fn mutated_messages_never_panic_and_round_trip() {
    let mut rng = XorShift(SEED ^ 1);
    for case in 0..CASES {
        let records: Vec<Record> = (0..rng.below(5))
            .map(|_| Record::new(rng.byte(), rng.bytes(6)).unwrap())
            .collect();
        let mut input = encode(&records);
        if !input.is_empty() {
            let at = rng.below(input.len());
            match rng.below(3) {
                0 => input[at] = rng.byte(),
                1 => input.truncate(at),
                _ => input.insert(at, rng.byte()),
            }
        }
        if let Some(parsed) = parse_or_report(case, &input) {
            assert_eq!(
                encode(&parsed),
                input,
                "{case}번째 입력이 왕복하지 않습니다"
            );
        }
    }
}

#[test]
fn any_records_round_trip() {
    let mut rng = XorShift(SEED ^ 2);
    for case in 0..CASES {
        let records: Vec<Record> = (0..rng.below(8))
            .map(|_| Record::new(rng.byte(), rng.bytes(Record::MAX_VALUE_LEN)).unwrap())
            .collect();
        assert_eq!(
            parse(&encode(&records)),
            Ok(records),
            "{case}번째 레코드 목록이 왕복하지 않습니다"
        );
    }
}
