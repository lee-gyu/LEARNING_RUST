// 기대하는 에러: (E 코드 없음)
// 핵심 메시지: `$x:pat` is followed by `|`, which is not allowed for `pat` fragments
// 에디션: 2021, 2024에서 실패 / 2018에서 통과(실행하면 "둘 중 하나")
//
// 2018 에디션의 $x:pat은 맨 바깥의 |를 받지 않으므로 | 앞에서 멈췄고, 그래서 |를 구분자로 쓸 수 있었습니다.
// 2021 에디션부터 $x:pat은 A | B 전체를 받을 수 있으므로, 뒤에 |가 오는 규칙은 뜻이 모호해 거절됩니다.
// 매크로를 정의할 때 에러가 나므로 매크로를 부르지 않아도 실패합니다.
// 고치려면 컴파일러의 help대로 $x:pat_param을 씁니다.
//
// 확인: rustc --edition 2024 compile_fail/pat_fragment_followed_by_or.rs --out-dir /tmp/rust-p2-ch3
//       rustc --edition 2018 compile_fail/pat_fragment_followed_by_or.rs --out-dir /tmp/rust-p2-ch3

macro_rules! either {
    ($x:pat | $y:pat) => { // → `$x:pat` is followed by `|`
        "둘 중 하나"
    };
}

fn main() {
    println!("{}", either!(1 | 2));
}
