// 기대하는 에러: E0034
// 핵심 메시지: multiple applicable items in scope — multiple `try_into` found
// 에디션: 2021, 2024에서 실패 / 2018에서 통과(실행하면 Ok(255))
//
// 2021 에디션 프렐류드에 TryInto가 들어오면서, 같은 이름의 메서드를 가진 사용자 트레이트와
// 부딪힙니다. 두 메서드가 모두 self를 값으로 받으므로 메서드 탐색의 같은 단계에서 둘 다 찾아집니다.
// 2018 에디션에서는 TryInto가 스코프에 없어서 Clamp::try_into 하나뿐이었습니다.
// 고치려면 Clamp::try_into(x)처럼 트레이트 이름을 경로에 적습니다.
//
// 확인: rustc --edition 2024 compile_fail/e0034_prelude_try_into_ambiguous.rs --out-dir /tmp/rust-p2-ch3
//       rustc --edition 2018 compile_fail/e0034_prelude_try_into_ambiguous.rs --out-dir /tmp/rust-p2-ch3

/// TryInto가 표준 라이브러리에 들어오기 전에 만든 트레이트라고 생각합니다.
trait Clamp {
    fn try_into(self) -> Result<u8, String>;
}

impl Clamp for i32 {
    fn try_into(self) -> Result<u8, String> {
        Ok(self.clamp(0, 255) as u8)
    }
}

fn main() {
    let x: i32 = 300;
    let r: Result<u8, String> = x.try_into(); // → E0034
    println!("{r:?}");
}
