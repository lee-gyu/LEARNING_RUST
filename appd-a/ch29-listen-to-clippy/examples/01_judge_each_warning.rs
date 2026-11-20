//! 지침 1: Clippy 경고마다 설명을 읽고, 고칠지·작은 리팩터링으로 없앨지·이유를 적고 예외로 둘지
//! 고릅니다.
//!
//! 실행: cargo run -p er-ch29-listen-to-clippy --example 01_judge_each_warning
//!
//! 세 구역은 모두 **기본 설정의** Clippy가 경고하는 코드입니다. 비권장 판에 붙인
//! `#[expect(…, reason = "…")]`를 지우고
//! `cargo clippy -p er-ch29-listen-to-clippy --example 01_judge_each_warning`을 실행하면
//! 구역마다 적은 린트가 경고합니다(README 2.2절). 3번 구역은 반대로, 경고를 따른 판이 비권장이고
//! 경고를 이유와 함께 예외로 둔 판이 권장입니다.

// ── 1. 진짜 실수: 고친다 (clippy::zero_prefixed_literal, complexity 그룹) ──────────────
mod fix {
    pub mod v1 {
        // C 계열 언어에서 `0644`는 8진수지만, Rust에서는 앞의 0이 무시되는 10진수 644입니다.
        // 린트 설명 페이지가 바로 이 차이를 경고하는 이유로 듭니다.
        #[expect(
            clippy::zero_prefixed_literal,
            reason = "비권장 판: Clippy가 잡는 실수를 보이려고 남김"
        )]
        pub const MODE: u32 = 0644;
    }

    pub mod v2 {
        // help가 제안한 두 갈래(10진수면 0을 지우고, 8진수면 `0o`) 가운데 뜻에 맞는 쪽을 골랐습니다.
        pub const MODE: u32 = 0o644;
    }

    /// 파일 권한에서 그룹 읽기 비트(8진수 040)가 켜져 있는지 봅니다.
    pub fn group_can_read(mode: u32) -> bool {
        mode & 0o040 != 0
    }
}

// ── 2. 취향처럼 보이는 경고: 작은 리팩터링으로 없앤다 (clippy::type_complexity) ──────────
// "이 타입은 복잡하지 않다"고 따지는 대신, 별칭 두 줄로 경고를 없애면 시그니처도 읽기 쉬워집니다.
mod refactor {
    pub mod v1 {
        use std::collections::BTreeMap;

        #[expect(clippy::type_complexity, reason = "비권장 판: 복잡한 타입을 그대로 둠")]
        pub fn rules() -> BTreeMap<&'static str, Vec<Box<dyn Fn(u32) -> Option<u32>>>> {
            let mut table: BTreeMap<&'static str, Vec<Box<dyn Fn(u32) -> Option<u32>>>> =
                BTreeMap::new();
            table.insert("일반", vec![Box::new(|p| (p >= 30_000).then(|| p - 3_000))]);
            table.insert(
                "회원",
                vec![
                    Box::new(|p| Some(p / 10 * 9)),
                    Box::new(|p| (p >= 50_000).then(|| p - 5_000)),
                ],
            );
            table
        }

        #[expect(clippy::type_complexity, reason = "비권장 판: 복잡한 타입을 그대로 둠")]
        pub fn price(
            table: &BTreeMap<&'static str, Vec<Box<dyn Fn(u32) -> Option<u32>>>>,
            grade: &str,
            base: u32,
        ) -> u32 {
            table
                .get(grade)
                .into_iter()
                .flatten()
                .fold(base, |p, rule| rule(p).unwrap_or(p))
        }
    }

    pub mod v2 {
        use std::collections::BTreeMap;

        /// 가격을 받아 할인한 가격을 돌려주는 규칙입니다. 적용되지 않으면 `None`입니다.
        pub type Rule = Box<dyn Fn(u32) -> Option<u32>>;
        /// 회원 등급별 규칙 목록입니다.
        pub type RuleTable = BTreeMap<&'static str, Vec<Rule>>;

        pub fn rules() -> RuleTable {
            let mut table = RuleTable::new();
            table.insert("일반", vec![Box::new(|p| (p >= 30_000).then(|| p - 3_000))]);
            table.insert(
                "회원",
                vec![
                    Box::new(|p| Some(p / 10 * 9)),
                    Box::new(|p| (p >= 50_000).then(|| p - 5_000)),
                ],
            );
            table
        }

        pub fn price(table: &RuleTable, grade: &str, base: u32) -> u32 {
            table
                .get(grade)
                .into_iter()
                .flatten()
                .fold(base, |p, rule| rule(p).unwrap_or(p))
        }
    }
}

// ── 3. 의도한 코드: 이유를 적고 예외로 둔다 (clippy::large_enum_variant, perf 그룹) ──────
// 린트 설명의 "Known problems"는 이 린트가 실제로 어느 배리언트가 많은지 모르며, 작은 배리언트가
// 드물면 Box가 오히려 손해라고 적고 있습니다. 이 프로토콜은 프레임 거의 전부가 Data입니다.
mod exception {
    pub const PAYLOAD: usize = 512;

    /// 프레임 1000개 가운데 100번째마다 Ack인 흐름입니다.
    fn is_ack(i: usize) -> bool {
        i % 100 == 99
    }

    pub mod v1 {
        use super::PAYLOAD;

        // 비권장: help대로 큰 필드를 Box에 넣었습니다. 경고는 사라지지만 Data마다 힙 할당이 생깁니다.
        pub enum Frame {
            Data(Box<[u8; PAYLOAD]>),
            Ack(u16),
        }

        impl Frame {
            pub fn nth(i: usize) -> Self {
                if super::is_ack(i) {
                    Frame::Ack(1)
                } else {
                    Frame::Data(Box::new([0; PAYLOAD]))
                }
            }

            /// (본문 바이트 수, 확인 번호)
            pub fn parts(&self) -> (usize, u16) {
                match self {
                    Frame::Data(body) => (body.len(), 0),
                    Frame::Ack(no) => (0, *no),
                }
            }
        }
    }

    pub mod v2 {
        use super::PAYLOAD;

        // 권장: 그대로 두고, 왜 그대로 두는지를 reason에 남깁니다.
        #[expect(
            clippy::large_enum_variant,
            reason = "프레임의 99%가 Data라서 Box에 넣으면 할당만 늘고 메모리는 줄지 않음"
        )]
        pub enum Frame {
            Data([u8; PAYLOAD]),
            Ack(u16),
        }

        impl Frame {
            pub fn nth(i: usize) -> Self {
                if super::is_ack(i) {
                    Frame::Ack(1)
                } else {
                    Frame::Data([0; PAYLOAD])
                }
            }

            /// (본문 바이트 수, 확인 번호)
            pub fn parts(&self) -> (usize, u16) {
                match self {
                    Frame::Data(body) => (body.len(), 0),
                    Frame::Ack(no) => (0, *no),
                }
            }
        }
    }
}

/// (본문 바이트 수, 확인 번호) 쌍을 모두 더합니다.
fn totals(parts: impl Iterator<Item = (usize, u16)>) -> (usize, u32) {
    parts.fold((0, 0), |(body, acks), (b, a)| {
        (body + b, acks + u32::from(a))
    })
}

fn main() {
    println!("1. 진짜 실수: 고친다 (clippy::zero_prefixed_literal)");
    for (name, mode) in [("비권장 v1", fix::v1::MODE), ("권장 v2  ", fix::v2::MODE)] {
        println!(
            "  {name}: MODE = 0o{mode:o}, 그룹 읽기 {}",
            fix::group_can_read(mode)
        );
    }
    // 출력: 비권장 v1: MODE = 0o1204, 그룹 읽기 false
    // 출력: 권장 v2  : MODE = 0o644, 그룹 읽기 true

    println!("2. 취향처럼 보이는 경고: 작은 리팩터링으로 없앤다 (clippy::type_complexity)");
    let t1 = refactor::v1::rules();
    let t2 = refactor::v2::rules();
    for (grade, base) in [("일반", 40_000), ("회원", 60_000), ("손님", 20_000)] {
        println!(
            "  {grade} {base}원 → 비권장 v1 {}원, 권장 v2 {}원",
            refactor::v1::price(&t1, grade, base),
            refactor::v2::price(&t2, grade, base)
        );
    }
    // 출력: 일반 40000원 → 37000원, 회원 60000원 → 49000원, 손님 20000원 → 20000원 (두 판 같음)

    println!("3. 의도한 코드: 이유를 적고 예외로 둔다 (clippy::large_enum_variant)");
    let n = 1000;
    let boxed: Vec<_> = (0..n).map(exception::v1::Frame::nth).collect();
    let inline: Vec<_> = (0..n).map(exception::v2::Frame::nth).collect();
    // 두 판이 같은 흐름을 담았는지 본문 바이트와 확인 번호(모두 1)의 합으로 확인합니다.
    let (body1, acks1) = totals(boxed.iter().map(exception::v1::Frame::parts));
    let (body2, acks2) = totals(inline.iter().map(exception::v2::Frame::parts));
    println!(
        "  프레임 {n}개: 본문 v1 {body1}바이트 = v2 {body2}바이트, Ack v1 {acks1}개 = v2 {acks2}개"
    ); // 출력: 본문 v1 506880바이트 = v2 506880바이트, Ack v1 10개 = v2 10개

    // Box판은 Data 하나마다 상자(힙 할당) 하나가 생깁니다. 바이트 수는 Vec 본체 + 상자 속 배열입니다.
    // 크기는 64비트 플랫폼(이 저장소는 macOS aarch64)에서 확인한 값입니다.
    let boxes = body1 / exception::PAYLOAD;
    let v1_size = size_of::<exception::v1::Frame>();
    let v2_size = size_of::<exception::v2::Frame>();
    println!(
        "  비권장 v1 (Box): 프레임 하나 {v1_size}바이트, 모두 {}바이트, 상자 {boxes}개",
        n * v1_size + boxes * exception::PAYLOAD
    ); // 출력: 프레임 하나 16바이트, 모두 522880바이트, 상자 990개
    println!(
        "  권장 v2 (그대로): 프레임 하나 {v2_size}바이트, 모두 {}바이트, 상자 0개",
        n * v2_size
    ); // 출력: 프레임 하나 514바이트, 모두 514000바이트, 상자 0개
}
