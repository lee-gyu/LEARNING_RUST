// 기대하는 에러: E0277 — `YearMonth` doesn't implement `Debug`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_user_derive_debug.rs --out-dir /tmp/er-ch10
//
// 라이브러리(여기서는 모듈 ledger로 흉내 냄)의 공개 타입에 Debug가 없으면, 그 타입을 필드로 담은
// 사용자 타입도 derive(Debug)를 쓸 수 없습니다. derive는 필드마다 같은 트레이트를 요구하기 때문입니다.
// 이 파일은 한 크레이트라서 `help:`가 YearMonth에 derive(Debug)를 붙이라고 제안하지만,
// 실제로 다른 크레이트의 타입이면 이 `help:`는 나오지 않고, 사용자가
// impl Debug for ledger::YearMonth를 쓰면 고아 규칙(E0117)에 걸립니다.
// 해결(라이브러리 쪽): 공개 타입에는 Debug를 구현합니다(examples/08_checklist_applied.rs의 v1).
// 해결(사용자 쪽, 라이브러리를 고칠 수 없을 때): Report의 Debug를 손으로 쓰면서 그 필드를
//       접근자로 꺼내 출력하거나, 뉴타입으로 감쌉니다(ch6).

mod ledger {
    pub struct YearMonth {
        pub year: u16,
        pub month: u8,
    }
}

#[derive(Debug)]
struct Report {
    month: ledger::YearMonth,
    total: u32,
}

fn main() {
    let report = Report {
        month: ledger::YearMonth {
            year: 2026,
            month: 11,
        },
        total: 63_500,
    };
    println!("{report:?}");
}
