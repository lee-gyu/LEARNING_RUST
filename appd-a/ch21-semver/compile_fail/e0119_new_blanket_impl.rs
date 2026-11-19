// 기대하는 에러: E0119 — conflicting implementations of trait `Label` for type `Ticket`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0119_new_blanket_impl.rs --out-dir /tmp/er-ch21
//
// 상황: 모듈 parcel은 다른 사람이 관리하는 라이브러리라고 생각합니다.
//   - 1.0.0의 트레이트 Label(송장에 찍을 문구)에는 구현이 하나도 없었습니다.
//     사용자는 자기 타입 Ticket에 Label을 직접 구현했고, 이 파일은 컴파일됐습니다.
//   - 1.1.0에서 라이브러리가 "Display를 구현한 타입이면 모두 Label"이라는 포괄 구현을 더하고
//     마이너 버전으로 냈습니다. 항목을 "더하기만" 한 변경입니다.
//   - Ticket은 Display도 구현하므로 `Ticket: Label`의 구현이 둘(포괄 구현, 사용자의 구현)이 됩니다.
//     같은 타입에 같은 트레이트의 구현은 하나만 있어야 하므로(일관성) 사용자의 impl이 거부됩니다.
// 해결: 사용자는 자기 구현을 지우고 포괄 구현을 받아들여야 하는데, 그러면 찍히는 문구가 바뀝니다.
//       고칠 수 있는 쪽은 라이브러리입니다. 이미 공개한 트레이트에 포괄 구현을 더하는 일은
//       메이저 버전에서 합니다. 마이너에서는 라이브러리 자신의 타입에 대한 구현만 더합니다.

mod parcel {
    use std::fmt::Display;

    pub trait Label {
        fn label(&self) -> String;
    }

    // 1.1.0에서 더한 포괄 구현
    impl<T: Display> Label for T {
        fn label(&self) -> String {
            format!("{self}")
        }
    }
}

use parcel::Label;
use std::fmt;

struct Ticket {
    no: &'static str,
    fragile: bool,
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "송장 {}", self.no)
    }
}

// 1.0.0 시절에 쓴 사용자의 구현
impl Label for Ticket {
    // E0119
    fn label(&self) -> String {
        if self.fragile {
            format!("[취급주의] 송장 {}", self.no)
        } else {
            format!("송장 {}", self.no)
        }
    }
}

fn main() {
    let t = Ticket {
        no: "KR-1203",
        fragile: true,
    };
    println!("{}", t.label());
}
