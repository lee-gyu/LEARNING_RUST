// 지침 1: 트레이트의 필수 메서드는 목적을 이루는 데 꼭 필요한 것만 두고,
//         그 위에 만들 수 있는 편의 메서드는 기본 구현으로 넉넉하게 줍니다.
// 비권장: 쓰는 쪽에 줄 메서드 여섯 개를 모두 필수 메서드로 둔 트레이트
//         → 구현하는 쪽은 타입마다 여섯 개를 다시 쓰고, 같은 규칙을 구현마다 따로 정하게 됩니다.
// 권장:   필수 메서드는 at 하나, 나머지 다섯 개는 at만으로 만든 기본 메서드
//         → 구현하는 쪽은 메서드 하나만 쓰고, 쓰는 쪽은 여섯 개를 모두 얻습니다.
//
// 상황: 마라톤 코스의 급수대 위치(출발점에서 m)를 묻는 트레이트입니다.
//       코스마다 위치를 갖고 있는 방법이 다릅니다(위치 목록, 같은 간격).
//
// 실행: cargo run -p er-ch13-default-implementations --example 01_minimal_required

// ---- 비권장: 필수 메서드 6개 -------------------------------------------------

mod wide {
    pub trait Checkpoints {
        /// index번째 지점의 위치(출발점에서 m). 지점이 없으면 None.
        fn at(&self, index: usize) -> Option<u32>;
        /// 지점 수
        fn count(&self) -> usize;
        /// 마지막 지점의 위치
        fn last(&self) -> Option<u32>;
        /// pos에서 앞으로 만날 첫 지점의 위치
        fn next_from(&self, pos: u32) -> Option<u32>;
        /// pos에서 다음 지점까지 남은 거리
        fn distance_to_next(&self, pos: u32) -> Option<u32>;
        /// 출발점과 지점들 사이에서 가장 긴 구간
        fn longest_gap(&self) -> u32;
    }

    /// 위치를 하나하나 적은 코스
    pub struct Listed {
        pub stops: Vec<u32>,
    }

    /// 같은 간격으로 놓인 코스: first부터 every마다 n곳
    pub struct Even {
        pub first: u32,
        pub every: u32,
        pub n: usize,
    }

    // 구현하는 쪽은 타입마다 여섯 개를 모두 써야 합니다.
    impl Checkpoints for Listed {
        fn at(&self, index: usize) -> Option<u32> {
            self.stops.get(index).copied()
        }
        fn count(&self) -> usize {
            self.stops.len()
        }
        fn last(&self) -> Option<u32> {
            self.stops.last().copied()
        }
        // 이 구현을 쓴 사람은 문서의 "앞으로 만날"을 "지금 위치보다 뒤"로 읽었습니다.
        // 급수대 바로 위에 서 있으면 그 급수대를 건너뜁니다.
        fn next_from(&self, pos: u32) -> Option<u32> {
            self.stops.iter().copied().find(|&p| p > pos)
        }
        fn distance_to_next(&self, pos: u32) -> Option<u32> {
            self.next_from(pos).map(|p| p - pos)
        }
        fn longest_gap(&self) -> u32 {
            let mut prev = 0;
            let mut longest = 0;
            for &p in &self.stops {
                longest = longest.max(p - prev);
                prev = p;
            }
            longest
        }
    }

    impl Checkpoints for Even {
        fn at(&self, index: usize) -> Option<u32> {
            if index < self.n {
                Some(self.first + self.every * index as u32)
            } else {
                None
            }
        }
        fn count(&self) -> usize {
            self.n
        }
        fn last(&self) -> Option<u32> {
            self.n.checked_sub(1).and_then(|i| self.at(i))
        }
        // 이 구현을 쓴 사람은 "지금 서 있는 곳도 포함"으로 읽었습니다.
        fn next_from(&self, pos: u32) -> Option<u32> {
            (0..self.n).filter_map(|i| self.at(i)).find(|&p| p >= pos)
        }
        fn distance_to_next(&self, pos: u32) -> Option<u32> {
            self.next_from(pos).map(|p| p - pos)
        }
        fn longest_gap(&self) -> u32 {
            match self.n {
                0 => 0,
                1 => self.first,
                _ => self.first.max(self.every),
            }
        }
    }
}

// ---- 권장: 필수 메서드 1개 + 기본 메서드 5개 ---------------------------------

mod narrow {
    pub trait Checkpoints {
        /// index번째 지점의 위치(출발점에서 m). 지점이 없으면 None.
        /// index가 커질수록 위치도 커져야 하고, 중간에 None이 끼면 안 됩니다.
        /// 필수 메서드는 이것 하나입니다. 위치를 어떻게 갖고 있는지는 타입마다 다르기 때문입니다.
        fn at(&self, index: usize) -> Option<u32>;

        // 아래는 모두 at만으로 만든 기본 메서드입니다. 규칙("앞으로 만날"은 지금 위치를 포함)은
        // 여기 한 곳에만 있으므로, 구현마다 다르게 해석될 일이 없습니다.

        /// 지점 수
        fn count(&self) -> usize {
            (0..).map_while(|i| self.at(i)).count()
        }
        /// 마지막 지점의 위치
        fn last(&self) -> Option<u32> {
            self.count().checked_sub(1).and_then(|i| self.at(i))
        }
        /// pos에서 앞으로 만날 첫 지점의 위치(pos에 있는 지점도 포함)
        fn next_from(&self, pos: u32) -> Option<u32> {
            (0..).map_while(|i| self.at(i)).find(|&p| p >= pos)
        }
        /// pos에서 다음 지점까지 남은 거리
        fn distance_to_next(&self, pos: u32) -> Option<u32> {
            self.next_from(pos).map(|p| p - pos)
        }
        /// 출발점과 지점들 사이에서 가장 긴 구간
        fn longest_gap(&self) -> u32 {
            let mut prev = 0;
            let mut longest = 0;
            for p in (0..).map_while(|i| self.at(i)) {
                longest = longest.max(p - prev);
                prev = p;
            }
            longest
        }
    }

    pub struct Listed {
        pub stops: Vec<u32>,
    }

    pub struct Even {
        pub first: u32,
        pub every: u32,
        pub n: usize,
    }

    /// 나중에 추가한 구현: 급수대가 한 곳뿐인 짧은 코스
    pub struct Single {
        pub stop: u32,
    }

    // 구현하는 쪽은 타입마다 at 하나만 씁니다.
    impl Checkpoints for Listed {
        fn at(&self, index: usize) -> Option<u32> {
            self.stops.get(index).copied()
        }
    }

    impl Checkpoints for Even {
        fn at(&self, index: usize) -> Option<u32> {
            if index < self.n {
                Some(self.first + self.every * index as u32)
            } else {
                None
            }
        }
    }

    impl Checkpoints for Single {
        fn at(&self, index: usize) -> Option<u32> {
            if index == 0 { Some(self.stop) } else { None }
        }
    }
}

// ---- 쓰는 쪽 코드 -------------------------------------------------------------

fn show_next(next: Option<u32>, gap: Option<u32>) -> String {
    match (next, gap) {
        (Some(p), Some(g)) => format!("{p}m({g}m 앞)"),
        _ => "없음".to_string(),
    }
}

// 두 설계 모두 쓰는 쪽이 부를 수 있는 메서드는 같습니다. 다른 것은 구현하는 쪽이 쓴 양입니다.
fn briefing_wide(name: &str, course: &impl wide::Checkpoints, pos: u32) {
    println!(
        "  {name}: {}곳, 마지막 {:?}, 가장 긴 구간 {}m, {pos}m에서 다음 급수대 {}",
        course.count(),
        course.last(),
        course.longest_gap(),
        show_next(course.next_from(pos), course.distance_to_next(pos)),
    );
}

fn briefing(name: &str, course: &impl narrow::Checkpoints, pos: u32) {
    println!(
        "  {name}: {}곳, 마지막 {:?}, 가장 긴 구간 {}m, {pos}m에서 다음 급수대 {}",
        course.count(),
        course.last(),
        course.longest_gap(),
        show_next(course.next_from(pos), course.distance_to_next(pos)),
    );
}

fn main() {
    // 두 코스의 급수대 위치는 똑같이 5000, 10000, 15000, 20000m입니다.
    // 10000m는 급수대 바로 위입니다.
    println!("[비권장] 필수 메서드 6개: 구현 두 개가 메서드 12개를 씁니다");
    let listed = wide::Listed {
        stops: vec![5000, 10000, 15000, 20000],
    };
    let even = wide::Even {
        first: 5000,
        every: 5000,
        n: 4,
    };
    briefing_wide("목록형", &listed, 10000); // 출력:   목록형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 15000m(5000m 앞)
    briefing_wide("간격형", &even, 10000); // 출력:   간격형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 10000m(0m 앞)

    println!("[권장] 필수 메서드 1개 + 기본 메서드 5개: 구현 두 개가 메서드 2개를 씁니다");
    let listed = narrow::Listed {
        stops: vec![5000, 10000, 15000, 20000],
    };
    let even = narrow::Even {
        first: 5000,
        every: 5000,
        n: 4,
    };
    briefing("목록형", &listed, 10000); // 출력:   목록형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 10000m(0m 앞)
    briefing("간격형", &even, 10000); // 출력:   간격형: 4곳, 마지막 Some(20000), 가장 긴 구간 5000m, 10000m에서 다음 급수대 10000m(0m 앞)

    println!("[권장] 새 구현도 at 하나만 쓰면 여섯 메서드를 모두 얻습니다");
    let kids = narrow::Single { stop: 1500 };
    briefing("어린이 코스", &kids, 1000); // 출력:   어린이 코스: 1곳, 마지막 Some(1500), 가장 긴 구간 1500m, 1000m에서 다음 급수대 1500m(500m 앞)
}
