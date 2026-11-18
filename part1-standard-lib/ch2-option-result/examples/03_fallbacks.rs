// 이 예제가 보여 주는 것: 대안 고르기 — unwrap_or, unwrap_or_else, unwrap_or_default, or, or_else,
//                        그리고 즉시 평가(eager)와 게으른 평가(lazy)의 차이
// 실행: cargo run -p p1-ch2-option-result --example 03_fallbacks
//
// 메서드 이름이 _else로 끝나면 대안을 "값"이 아니라 "클로저"로 받습니다.
// - 값으로 받는 메서드(unwrap_or, or, map_or, ok_or): 메서드를 부르기 전에 인자를 먼저 계산합니다. 즉시 평가.
//   Rust는 함수를 부르기 전에 인자부터 모두 계산하므로, 대안이 필요 없는 Some/Ok일 때도 계산은 이미 끝나 있습니다.
// - 클로저로 받는 메서드(unwrap_or_else, or_else, map_or_else, ok_or_else): 대안이 정말 필요할 때만 클로저를 부릅니다. 게으른 평가.
// 아래 함수들은 불릴 때마다 한 줄을 출력하므로, 출력을 보면 언제 계산되었는지 알 수 있습니다.

use std::num::ParseIntError;

// 회원 번호로 이름을 찾습니다. 없는 번호는 None입니다.
fn lookup(id: u32) -> Option<&'static str> {
    match id {
        1 => Some("김하나"),
        2 => Some("이두리"),
        _ => None,
    }
}

// 대안 값을 만드는 함수. 실제 프로그램이라면 설정 파일을 읽는 등 비용이 드는 일이라고 생각해 봅니다.
fn default_name() -> &'static str {
    println!("    (default_name 실행됨)");
    "손님"
}

// 두 번째 저장소에서 찾아보는 함수. 이것도 실패할 수 있으므로 Option을 돌려줍니다.
fn lookup_backup(id: u32) -> Option<&'static str> {
    println!("    (lookup_backup 실행됨)");
    if id == 3 { Some("박세나") } else { None }
}

// 10진수로 읽어 보고, 실패하면 "0x"로 시작하는 16진수로 다시 읽어 봅니다.
// i64::from_str_radix(문자열, 진법)는 문자열을 주어진 진법의 정수로 읽는 연관 함수입니다.
// Result::or_else의 시그니처: fn or_else<F, O>(self, op: O) -> Result<T, F> where O: FnOnce(E) -> Result<T, F>
// 클로저는 첫 번째 에러를 받지만 여기서는 쓰지 않으므로 _로 버립니다.
fn parse_number(s: &str) -> Result<i64, ParseIntError> {
    s.parse::<i64>()
        .or_else(|_| i64::from_str_radix(s.trim_start_matches("0x"), 16))
}

fn main() {
    println!("== 1. unwrap_or(값)과 unwrap_or_else(클로저) ==");
    for id in [1, 9] {
        println!("회원 {id}");
        let eager = lookup(id).unwrap_or(default_name());
        println!("  unwrap_or      → {eager}");
        // 함수 이름 default_name을 그대로 넘깁니다. 인자 없이 &str을 돌려주므로 FnOnce() -> &str에 맞습니다.
        let lazy = lookup(id).unwrap_or_else(default_name);
        println!("  unwrap_or_else → {lazy}");
    }
    // 출력: 회원 1
    // 출력:     (default_name 실행됨)
    // 출력:   unwrap_or      → 김하나
    // 출력:   unwrap_or_else → 김하나
    // 출력: 회원 9
    // 출력:     (default_name 실행됨)
    // 출력:   unwrap_or      → 손님
    // 출력:     (default_name 실행됨)
    // 출력:   unwrap_or_else → 손님
    // 회원 1은 이름이 있어서 대안이 필요 없는데도 unwrap_or 쪽에서 default_name이 실행되었습니다.
    // unwrap_or_else는 None일 때(회원 9)만 실행합니다.

    println!("\n== 2. unwrap_or_default: 타입의 기본값 ==");
    // 시그니처: fn unwrap_or_default(self) -> T where T: Default
    // 정수는 0, String은 빈 문자열, Vec은 빈 벡터가 기본값입니다(Default 트레이트는 ch6-core-traits).
    let count: u32 = "열".parse().unwrap_or_default();
    let nickname: String = lookup(9).map(String::from).unwrap_or_default();
    println!("count = {count}, nickname = {nickname:?}");
    // 출력: count = 0, nickname = ""

    println!("\n== 3. or(Option)과 or_else(클로저): 다른 Option으로 대신하기 ==");
    // unwrap_or는 값을 꺼내지만, or/or_else는 결과가 여전히 Option입니다. "첫 번째로 찾은 것"을 고를 때 씁니다.
    // 시그니처: fn or(self, optb: Option<T>) -> Option<T>
    //          fn or_else<F>(self, f: F) -> Option<T> where F: FnOnce() -> Option<T>
    for id in [2, 3] {
        println!("회원 {id}");
        let eager = lookup(id).or(lookup_backup(id));
        println!("  or      → {eager:?}");
        let lazy = lookup(id).or_else(|| lookup_backup(id));
        println!("  or_else → {lazy:?}");
    }
    // 출력: 회원 2
    // 출력:     (lookup_backup 실행됨)
    // 출력:   or      → Some("이두리")
    // 출력:   or_else → Some("이두리")
    // 출력: 회원 3
    // 출력:     (lookup_backup 실행됨)
    // 출력:   or      → Some("박세나")
    // 출력:     (lookup_backup 실행됨)
    // 출력:   or_else → Some("박세나")
    // 회원 2는 첫 저장소에서 찾았으므로 or_else 쪽에서는 lookup_backup이 실행되지 않았습니다.
    // 회원 3은 첫 저장소에 없어서 두 방법 모두 두 번째 저장소에서 찾았습니다.

    // 여러 대안을 차례로 시도하고, 끝까지 없으면 기본값을 쓰는 체인입니다.
    let name = lookup(4)
        .or_else(|| lookup_backup(4))
        .unwrap_or("이름 없음");
    println!("회원 4의 최종 이름: {name}");
    // 출력:     (lookup_backup 실행됨)
    // 출력: 회원 4의 최종 이름: 이름 없음
    // 마지막 대안 "이름 없음"은 이미 만들어진 리터럴이라 계산 비용이 없으므로 unwrap_or로 충분합니다.

    println!("\n== 4. Result에서 대안 고르기 ==");
    // or_else는 Err일 때 다른 방법으로 다시 시도합니다. 성공하면 Ok가 되고, 그것도 실패하면 새 Err가 남습니다.
    for input in ["255", "0xff", "zz"] {
        println!("{input:?} → {:?}", parse_number(input));
    }
    // 출력: "255" → Ok(255)
    // 출력: "0xff" → Ok(255)
    // 출력: "zz" → Err(ParseIntError { kind: InvalidDigit })

    // Result의 unwrap_or_else는 클로저에 에러 값을 넘겨줍니다: F: FnOnce(E) -> T
    // 그래서 "에러를 알리고 기본값으로 진행"하는 처리를 한 곳에 쓸 수 있습니다.
    for input in ["0x10", "열여섯"] {
        let n = parse_number(input).unwrap_or_else(|e| {
            println!("  {input:?} 읽기 실패, 0을 씁니다: {e}");
            0
        });
        println!("{input:?} → {n}");
    }
    // 출력: "0x10" → 16
    // 출력:   "열여섯" 읽기 실패, 0을 씁니다: invalid digit found in string
    // 출력: "열여섯" → 0
}
