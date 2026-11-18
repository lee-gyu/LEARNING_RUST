// 이 예제가 보여 주는 것: 서식 문자열의 문법 전체 — 인자 고르기(위치·이름·캡처), 채움/정렬/너비,
//                        부호·#·0, 정밀도와 $·.*, 서식 트레이트(? x X o b e E p), 중괄호 이스케이프,
//                        그리고 format!·write!·format_args!의 관계
// 실행: cargo run -p p1-ch3-strings-fmt --example 06_format_syntax
//
// 자리표시자의 모양: {인자:채움 정렬 부호 # 0 너비 .정밀도 형식}
// 모든 항목은 생략할 수 있고, 적을 때는 이 순서를 지켜야 합니다.

// write!로 String에 쓰려면 std::fmt::Write 트레이트가 범위에 있어야 합니다(트레이트 메서드이므로).
use std::fmt::{self, Write};

// fmt::Arguments는 format_args!가 만든 "아직 문자열로 만들지 않은 서식 결과"입니다.
// Display를 구현하므로 {}로 출력할 수 있습니다.
fn log(args: fmt::Arguments) {
    println!("[로그] {args}");
}

fn main() {
    let name = "페리스";
    let age = 9;
    let word = "ab";

    // 1. 인자 고르기
    println!("== 1. 인자 고르기 ==");
    println!("{}는 {}살", name, age); // 출력: 페리스는 9살
    println!("{1}는 {0}살", age, name); // 출력: 페리스는 9살
    println!("{n}는 {a}살", n = name, a = age); // 출력: 페리스는 9살
    println!("{name}는 {age}살"); // 출력: 페리스는 9살 (같은 이름의 변수를 캡처)
    // 번호를 적은 자리표시자는 "다음 인자" 카운터를 움직이지 않습니다.
    // 그래서 아래의 {}는 0번, 1번 인자를 차례로 가져갑니다.
    println!("{1} {} {0} {}", 'A', 'B'); // 출력: B A A B

    // 2. 채움·정렬·너비
    println!("\n== 2. 채움, 정렬, 너비 ==");
    println!("[{age:6}] [{word:6}]"); // 출력: [     9] [ab    ]  ← 숫자는 오른쪽, 그 밖에는 왼쪽이 기본
    println!("[{word:<6}] [{word:^6}] [{word:>6}]"); // 출력: [ab    ] [  ab  ] [    ab]
    println!("[{word:*^8}] [{word:->6}]"); // 출력: [***ab***] [----ab]
    // 너비를 값으로 정하기: 이름$ 또는 번호$. 가리키는 값은 반드시 usize여야 합니다.
    let width = 7;
    println!("[{word:>width$}] [{:>1$}] [{word:>w$}]", word, 4, w = 5); // 출력: [     ab] [  ab] [   ab]

    // 3. 부호, #, 0
    println!("\n== 3. 부호, #, 0 ==");
    let neg = -42;
    println!("[{age:+}] [{neg:+}] [{:+.1}]", 2.0); // 출력: [+9] [-42] [+2.0]
    // 0 플래그: 부호(와 #의 접두사) 뒤, 숫자 앞에 0을 채웁니다. 정렬 지정보다 우선합니다.
    println!("[{neg:06}] [{:#06x}] [{:#010b}]", 255, 5); // 출력: [-00042] [0x00ff] [0b00000101]
    // 주의: "0을 채움 문자로 왼쪽 정렬"은 0 플래그와 다릅니다. 7이 70000처럼 보입니다.
    println!("[{:05}] [{:0>5}] [{:0<5}]", 7, 7, 7); // 출력: [00007] [00007] [70000]

    // 4. 정밀도
    println!("\n== 4. 정밀도 ==");
    let pi = std::f64::consts::PI;
    println!("[{pi:.2}] [{pi:8.3}] [{pi:<8.1}]"); // 출력: [3.14] [   3.142] [3.1     ]
    // 정확히 가운데인 값은 짝수 쪽으로 반올림합니다(IEEE 754의 기본 반올림 방식).
    println!("[{:.0}] [{:.0}] [{:.0}]", 0.5, 1.5, 2.5); // 출력: [0] [2] [2]
    // 문자열에서 정밀도는 "최대 글자 수"입니다. 바이트가 아니라 char 개수로 셉니다.
    let greeting = "안녕하세요";
    println!("[{greeting:.2}] [{greeting:>6.2}]"); // 출력: [안녕] [    안녕]
    // 정수에서는 정밀도를 무시합니다.
    println!("[{age:.3}]"); // 출력: [9]
    // 정밀도를 값으로 정하기: .이름$, .번호$, 또는 .*(인자 두 개: 정밀도, 값 순서)
    let digits = 3;
    println!("[{pi:.digits$}] [{:.*}] [{pi:>w$.p$}]", 1, pi, w = 8, p = 4); // 출력: [3.142] [3.1] [  3.1416]

    // 5. 서식 트레이트: 맨 끝 형식 문자가 어느 트레이트를 쓸지 고릅니다.
    println!("\n== 5. 서식 트레이트 ==");
    let n = 2026;
    println!("{n} {n:x} {n:X} {n:o} {n:b}"); // 출력: 2026 7ea 7EA 3752 11111101010
    println!("{n:#x} {n:#X} {n:#o} {n:#b}"); // 출력: 0x7ea 0x7EA 0o3752 0b11111101010
    // 음수는 2의 보수 비트 그대로 보여 줍니다(부호를 붙이지 않습니다).
    println!("{:x} {:b}", -1i8, -2i8); // 출력: ff 11111110
    println!("{:e} {:E} {:e}", 1234.5, 0.00012, n); // 출력: 1.2345e3 1.2E-4 2.026e3
    // x?, X?는 디버그 형식 안의 정수를 16진수로 보여 줍니다.
    let bytes = [10u8, 255];
    println!("{bytes:?} {bytes:x?} {bytes:X?}"); // 출력: [10, 255] [a, ff] [A, FF]
    // p는 참조가 가리키는 메모리 주소입니다. 값은 실행할 때마다 달라집니다.
    let r = &n;
    println!("{r:p}"); // 출력: 0x로 시작하는 주소 (실행마다 다름)

    // 6. 디버그 형식의 특징
    println!("\n== 6. 디버그 형식 ==");
    let tricky = "탭\t\"인용\"";
    println!("{tricky}"); // 출력: 탭	"인용"
    println!("{tricky:?}"); // 출력: "탭\t\"인용\""  ← 따옴표와 이스케이프가 붙어 경계가 보입니다.
    // 문자열의 디버그 형식은 너비를 무시합니다. 정렬이 필요하면 먼저 String으로 만든 뒤 정렬합니다.
    println!("[{word:>8?}] [{:>8}]", format!("{word:?}")); // 출력: ["ab"] [    "ab"]
    println!("{:#?}", (1, "가")); // {:#?}는 여러 줄로 펼칩니다.
    // 출력:
    // (
    //     1,
    //     "가",
    // )

    // 7. 중괄호 자체를 출력하려면 두 번 씁니다.
    println!("\n== 7. 중괄호 ==");
    println!("{{}} {{{age}}}"); // 출력: {} {9}

    // 8. format!, write!, format_args!
    println!("\n== 8. format!, write!, format_args! ==");
    // format!: 새 String을 만들어 돌려줍니다.
    let line = format!("{name:<4}|{age:>3}");
    println!("{line}"); // 출력: 페리스 |  9
    // write!/writeln!: 이미 있는 String 뒤에 씁니다. format!으로 임시 String을 만든 뒤
    // push_str하는 것보다 할당이 적습니다. 반환값은 fmt::Result이고, String에 쓰기는 실패하지 않습니다.
    let mut report = String::new();
    for (item, price) in [("커피", 4500), ("빵", 3200)] {
        writeln!(report, "{item:<4}{price:>6}원").unwrap();
    }
    write!(report, "합계 {:>6}원", 4500 + 3200).unwrap();
    println!("{report}");
    // 출력:
    // 커피    4500원
    // 빵     3200원
    // 합계   7700원

    // format_args!: 할당 없이 서식과 인자를 묶은 fmt::Arguments를 만듭니다.
    // 위의 매크로들은 모두 내부에서 format_args!를 씁니다.
    log(format_args!("{name}의 나이는 {age}살"));
    // 출력: [로그] 페리스의 나이는 9살
    // fmt::Arguments의 Display는 바깥의 너비를 무시합니다(07 예제에서 이유를 봅니다).
    // Clippy도 이것을 알고 unused_format_specs 경고를 냅니다. 무시된다는 것을 보이려고 일부러 쓴 코드입니다.
    // (#[expect]는 println! 호출 자체에는 붙일 수 없어서 let 문에 붙였습니다.)
    #[expect(
        clippy::unused_format_specs,
        reason = "너비가 무시되는 것을 보여 주려는 코드"
    )]
    let ignored = format!("[{:>12}]", format_args!("{age}살"));
    println!("{ignored}"); // 출력: [9살]
}
