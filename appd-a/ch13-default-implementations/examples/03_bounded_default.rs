// 지침 3: 기본 메서드 하나가 추가 능력(여기서는 Display)을 필요로 하면, 트레이트 전체가 아니라
//         그 메서드에만 바운드를 붙입니다. 조건을 만족하는 구현만 그 메서드를 얻습니다.
// 비권장: Display를 슈퍼트레이트로 요구
//         → 안내 문구를 쓰지 않는 구현도 Display를 지어내야 트레이트를 구현할 수 있습니다.
// 권장:   announce에만 `where Self: fmt::Display`
//         → 이름 있는 코스만 announce를 얻고, 이름 없는 코스는 여전히 at 하나만 구현합니다.
//
// 실행: cargo run -p er-ch13-default-implementations --example 03_bounded_default

use std::fmt;

// ---- 비권장: 슈퍼트레이트로 요구 ---------------------------------------------

mod with_supertrait {
    use std::fmt;

    // 슈퍼트레이트는 트레이트 전체의 조건입니다. 이 트레이트를 구현하려면 누구나 Display부터 구현해야 합니다.
    pub trait Checkpoints: fmt::Display {
        fn at(&self, index: usize) -> Option<u32>;

        fn next_from(&self, pos: u32) -> Option<u32> {
            (0..).map_while(|i| self.at(i)).find(|&p| p >= pos)
        }
        /// 코스 이름을 넣은 안내 문구
        fn announce(&self, pos: u32) -> String {
            match self.next_from(pos) {
                Some(p) => format!("{self}: 다음 급수대까지 {}m", p - pos),
                None => format!("{self}: 남은 급수대 없음"),
            }
        }
    }

    /// 이름이 있는 대회 코스
    pub struct NamedCourse {
        pub name: &'static str,
        pub stops: Vec<u32>,
    }

    /// 연습할 때 그때그때 만드는 이름 없는 코스. 안내 문구는 쓰지 않습니다.
    pub struct Stops(pub Vec<u32>);

    impl fmt::Display for NamedCourse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.name)
        }
    }
    impl Checkpoints for NamedCourse {
        fn at(&self, index: usize) -> Option<u32> {
            self.stops.get(index).copied()
        }
    }

    // Stops에는 보여 줄 이름이 없지만, 트레이트를 구현하려고 Display를 지어냈습니다.
    // 이제 {}로 출력할 수 있는 타입이 되었고, 그 표현은 아무 뜻이 없습니다.
    impl fmt::Display for Stops {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("(이름 없음)")
        }
    }
    impl Checkpoints for Stops {
        fn at(&self, index: usize) -> Option<u32> {
            self.0.get(index).copied()
        }
    }
}

// ---- 권장: 그 메서드에만 바운드 ----------------------------------------------

mod with_bound {
    use std::fmt;

    pub trait Checkpoints {
        fn at(&self, index: usize) -> Option<u32>;

        fn next_from(&self, pos: u32) -> Option<u32> {
            (0..).map_while(|i| self.at(i)).find(|&p| p >= pos)
        }
        /// 코스 이름을 넣은 안내 문구. 이름을 보여 줄 수 있는(Display) 코스에서만 쓸 수 있습니다.
        /// 바운드 덕분에 본문에서 {self}를 쓸 수 있습니다.
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

    pub struct NamedCourse {
        pub name: &'static str,
        pub stops: Vec<u32>,
    }

    pub struct Stops(pub Vec<u32>);

    impl fmt::Display for NamedCourse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.name)
        }
    }
    impl Checkpoints for NamedCourse {
        fn at(&self, index: usize) -> Option<u32> {
            self.stops.get(index).copied()
        }
    }

    // Display 없이 at 하나만 구현합니다. announce를 뺀 나머지 메서드는 모두 씁니다.
    impl Checkpoints for Stops {
        fn at(&self, index: usize) -> Option<u32> {
            self.0.get(index).copied()
        }
    }
}

// 쓰는 쪽의 제네릭 함수도 필요한 만큼만 요구합니다.
// announce를 쓰는 함수만 Display를 함께 요구하고, 위치만 보는 함수는 Checkpoints로 충분합니다.
fn morning_call<C: with_bound::Checkpoints + fmt::Display>(course: &C, pos: u32) -> String {
    course.announce(pos)
}

fn next_stop<C: with_bound::Checkpoints>(course: &C, pos: u32) -> Option<u32> {
    course.next_from(pos)
}

fn main() {
    println!("[비권장] Display를 슈퍼트레이트로 요구");
    {
        use with_supertrait::{Checkpoints, NamedCourse, Stops};
        let river = NamedCourse {
            name: "강변 코스",
            stops: vec![2500, 5000, 7500, 10000],
        };
        let practice = Stops(vec![1000, 2000, 3000]);
        println!("  {}", river.announce(4000)); // 출력:   강변 코스: 다음 급수대까지 1000m
        println!("  연습 코스의 다음 급수대: {:?}", practice.next_from(1500)); // 출력:   연습 코스의 다음 급수대: Some(2000)
        // 구현하는 쪽이 지어낸 Display가 쓰는 쪽에 그대로 열립니다.
        println!("  지어낸 Display: {practice}"); // 출력:   지어낸 Display: (이름 없음)
        println!("  {}", practice.announce(1500)); // 출력:   (이름 없음): 다음 급수대까지 500m
    }

    println!("[권장] announce에만 where Self: Display");
    {
        use with_bound::{Checkpoints, NamedCourse, Stops};
        let river = NamedCourse {
            name: "강변 코스",
            stops: vec![2500, 5000, 7500, 10000],
        };
        let practice = Stops(vec![1000, 2000, 3000]);
        println!("  {}", river.announce(4000)); // 출력:   강변 코스: 다음 급수대까지 1000m
        println!("  {}", morning_call(&river, 10500)); // 출력:   강변 코스: 남은 급수대 없음
        println!("  연습 코스의 다음 급수대: {:?}", practice.next_from(1500)); // 출력:   연습 코스의 다음 급수대: Some(2000)
        let found = next_stop(&practice, 2500);
        println!("  next_stop(연습 코스, 2500): {found:?}"); // 출력:   next_stop(연습 코스, 2500): Some(3000)
        // practice.announce(1500)은 컴파일되지 않습니다(E0277, compile_fail/e0277_bounded_default_method.rs).
    }
}
