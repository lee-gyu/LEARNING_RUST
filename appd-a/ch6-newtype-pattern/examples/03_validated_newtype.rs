// 지침: 타입은 맞지만 범위가 틀린 값(percent: 150)은, 필드를 감추고 생성자에서 검증하는 뉴타입으로 막습니다.
// 비권장: fn progress_bar_raw(percent: u8)        (0~255를 모두 받음)
// 권장:   fn progress_bar(p: Percent)              (Percent는 0~100만 담을 수 있음)
//
// 다운로드 서버가 보내 온 진행률을 막대로 그립니다. 서버에 버그가 있어 150이 섞여 들어옵니다.
//
// 실행: cargo run -p er-ch6-newtype-pattern --example 03_validated_newtype

const WIDTH: usize = 10;
const REPORTS: [u8; 4] = [0, 40, 100, 150];

// ---- 비권장 ----------------------------------------------------------------

// u8은 0~255를 담습니다. "0~100이어야 한다"는 규칙은 함수 이름과 주석에만 있습니다.
// 이 함수가 검사하지 않으면 막대가 틀을 뚫고, 검사한다면 같은 검사를 쓰는 함수마다 반복해야 합니다.
fn progress_bar_raw(percent: u8) -> String {
    let filled = usize::from(percent) * WIDTH / 100;
    format!("[{:<WIDTH$}] {percent:>3}%", "#".repeat(filled))
}

// ---- 권장 ------------------------------------------------------------------

mod percent {
    // 필드에 pub이 없습니다. 모듈 밖에서는 `Percent(150)`처럼 직접 만들 수 없고(E0603),
    // 아래의 `new`를 거쳐야만 Percent를 얻을 수 있습니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Percent(u8);

    impl Percent {
        // Percent를 만드는 유일한 통로입니다. 범위 검사는 여기서 한 번만 합니다.
        // 실패 이유가 "범위를 벗어남" 하나뿐이라 Option으로 돌려줍니다.
        pub fn new(value: u8) -> Option<Percent> {
            (value <= 100).then_some(Percent(value))
        }

        // 읽기만 열어 줍니다. 값을 바꾸는 메서드는 만들지 않으므로 불변식이 깨질 틈이 없습니다.
        pub fn get(self) -> u8 {
            self.0
        }

        // self.0 <= 100이 보장되므로 이 뺄셈은 넘치지 않습니다. 검사도 필요 없습니다.
        pub fn remaining(self) -> u8 {
            100 - self.0
        }
    }
}

use percent::Percent;

// Percent를 받는 함수는 범위를 다시 검사하지 않습니다. 시그니처가 검사가 끝났음을 보장합니다.
fn progress_bar(p: Percent) -> String {
    let filled = usize::from(p.get()) * WIDTH / 100; // 많아야 WIDTH
    format!("[{:<WIDTH$}] {:>3}%", "#".repeat(filled), p.get())
}

fn main() {
    println!("[비권장] u8을 그대로 받음");
    for raw in REPORTS {
        println!("  {raw:>3} -> {}", progress_bar_raw(raw));
    }
    // 출력:
    //     0 -> [          ]   0%
    //    40 -> [####      ]  40%
    //   100 -> [##########] 100%
    //   150 -> [###############] 150%

    println!("[권장] 경계에서 Percent로 바꾸고, 안쪽은 Percent만 받음");
    for raw in REPORTS {
        // 바깥에서 들어온 값을 안쪽 타입으로 바꾸는 이 자리가 경계입니다.
        match Percent::new(raw) {
            Some(p) => println!("  {raw:>3} -> {} 남음 {}%", progress_bar(p), p.remaining()),
            None => println!("  {raw:>3} -> 거부: 0~100 범위가 아님"),
        }
    }
    // 출력:
    //     0 -> [          ]   0% 남음 100%
    //    40 -> [####      ]  40% 남음 60%
    //   100 -> [##########] 100% 남음 0%
    //   150 -> 거부: 0~100 범위가 아님
}
