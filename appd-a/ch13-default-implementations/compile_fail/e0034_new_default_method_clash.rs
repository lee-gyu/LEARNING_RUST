// 기대하는 에러: E0034 — multiple applicable items in scope
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0034_new_default_method_clash.rs --out-dir /tmp/er-ch13
//
// 상황: course_lib는 다른 사람이 관리하는 라이브러리라고 생각합니다.
//   - v1의 Checkpoints에는 at과 count만 있었습니다. 그때 이 파일의 사용자 코드는 컴파일됐습니다.
//   - v2에서 기본 메서드 remaining(남은 지점 수)이 추가되었습니다.
//     구현하는 쪽(impl Checkpoints for Trail)은 한 글자도 고칠 필요가 없습니다.
//   - 그런데 사용자는 v1 시절에 자기 트레이트 TrailExt에 같은 이름의 remaining을 만들어 두었습니다.
//     두 트레이트를 모두 가져온 곳에서 trail.remaining(..)은 어느 쪽인지 정할 수 없게 됩니다.
// 해결: 부르는 곳에 트레이트 이름을 적습니다. TrailExt::remaining(&trail, 30000)
//       자기 트레이트의 메서드 이름을 바꿀 수 있다면 그것도 방법입니다.

mod course_lib {
    pub trait Checkpoints {
        fn at(&self, index: usize) -> Option<u32>;

        fn count(&self) -> usize {
            (0..).map_while(|i| self.at(i)).count()
        }

        /// v2에서 추가한 기본 메서드: pos 이후에 남은 지점 수
        fn remaining(&self, pos: u32) -> usize {
            (0..).map_while(|i| self.at(i)).filter(|&p| p >= pos).count()
        }
    }
}

use course_lib::Checkpoints;

struct Trail {
    stops: Vec<u32>,
    length: u32,
}

// v1에 맞춰 쓴 구현. v2에서도 그대로 컴파일됩니다.
impl Checkpoints for Trail {
    fn at(&self, index: usize) -> Option<u32> {
        self.stops.get(index).copied()
    }
}

// 사용자가 v1 시절에 만든 확장 트레이트: pos에서 결승점까지 남은 거리(m)
trait TrailExt {
    fn remaining(&self, pos: u32) -> u32;
}

impl TrailExt for Trail {
    fn remaining(&self, pos: u32) -> u32 {
        self.length - pos
    }
}

fn main() {
    let trail = Trail {
        stops: vec![10000, 20000, 30000, 40000],
        length: 42195,
    };
    println!("지점 {}곳", trail.count());
    println!("남은 거리 {}m", trail.remaining(30000)); // E0034
}
