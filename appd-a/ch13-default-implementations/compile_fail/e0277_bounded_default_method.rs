// 기대하는 에러: E0277 — `Stops` doesn't implement `std::fmt::Display`
//               (note: required by a bound in `Checkpoints::announce`)
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_bounded_default_method.rs --out-dir /tmp/er-ch13
//
// announce는 `where Self: fmt::Display`가 붙은 기본 메서드라서, Display를 구현한 타입에서만 부를 수 있습니다.
// Stops는 at만 구현했고 Display는 없으므로, 트레이트를 구현했어도 announce는 쓸 수 없습니다.
// 나머지 기본 메서드(next_from)는 바운드가 없으므로 그대로 씁니다.
// 해결: 이름이 필요 없는 코스라면 announce 대신 next_from을 씁니다.
//       이름을 보여 줄 수 있는 타입이라면 Display를 구현합니다(examples/03_bounded_default.rs의 NamedCourse).

use std::fmt;

trait Checkpoints {
    fn at(&self, index: usize) -> Option<u32>;

    fn next_from(&self, pos: u32) -> Option<u32> {
        (0..).map_while(|i| self.at(i)).find(|&p| p >= pos)
    }

    fn announce(&self, pos: u32) -> String
    where
        Self: fmt::Display,
    {
        match self.next_from(pos) {
            Some(p) => format!("{self}: 다음 급수대까지 {}m", p - pos),
            None => format!("{self}: 남은 급수대 없음"),
        }
    }
}

struct Stops(Vec<u32>);

impl Checkpoints for Stops {
    fn at(&self, index: usize) -> Option<u32> {
        self.0.get(index).copied()
    }
}

fn main() {
    let practice = Stops(vec![1000, 2000, 3000]);
    println!("{:?}", practice.next_from(1500)); // 바운드 없는 기본 메서드는 됩니다.
    println!("{}", practice.announce(1500)); // E0277
}
