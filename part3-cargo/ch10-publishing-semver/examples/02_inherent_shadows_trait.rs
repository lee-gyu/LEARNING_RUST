//! SemVer: 라이브러리가 고유 메서드를 더하면, 사용자가 만든 트레이트의 같은 이름 메서드 대신
//! 그것이 불려 동작이 조용히 바뀔 수 있습니다(Cargo Book: Possibly-breaking).
//!
//! 실행: cargo run -q -p p3-ch10-publishing-semver --example 02_inherent_shadows_trait
//!
//!   v1_0 : 1.0.0. `Parcel`에는 `describe` 메서드가 없습니다.
//!   v1_1 : 1.1.0. `Parcel`에 고유 메서드 `describe`를 더했습니다. 새 항목을 더하는 것이라 마이너로 냈습니다.

mod v1_0 {
    pub struct Parcel {
        pub weight_g: u32,
    }
}

mod v1_1 {
    pub struct Parcel {
        pub weight_g: u32,
    }
    impl Parcel {
        /// 1.1.0에서 더한 고유 메서드
        pub fn describe(&self) -> String {
            format!("parcel({}g)", self.weight_g)
        }
    }
}

/// 사용자 쪽 코드: 라이브러리 타입에 설명을 붙이려고 직접 만든 확장 트레이트
trait Describe {
    fn describe(&self) -> String;
}

impl Describe for v1_0::Parcel {
    fn describe(&self) -> String {
        format!("{:.1} kg 소포", self.weight_g as f64 / 1000.0)
    }
}

impl Describe for v1_1::Parcel {
    fn describe(&self) -> String {
        format!("{:.1} kg 소포", self.weight_g as f64 / 1000.0)
    }
}

fn main() {
    // 사용자 코드는 두 줄 모두 `p.describe()`로 똑같습니다.
    let p = v1_0::Parcel { weight_g: 1500 };
    println!("1.0.0: {}", p.describe());

    // 메서드 탐색은 같은 단계에서 고유 메서드를 트레이트 메서드보다 먼저 찾으므로,
    // 1.1.0에서는 라이브러리의 describe가 불립니다. 컴파일 에러도 경고도 없습니다.
    let p = v1_1::Parcel { weight_g: 1500 };
    println!("1.1.0: {}", p.describe());

    // 사용자의 메서드를 확실히 부르려면 트레이트 이름을 경로에 적어 부릅니다.
    // 완전 정규화 문법 `<v1_1::Parcel as Describe>::describe(&p)`도 같습니다.
    println!(
        "1.1.0: {} (Describe::describe로 부름)",
        Describe::describe(&p)
    );

    // 출력: 1.0.0: 1.5 kg 소포
    // 출력: 1.1.0: parcel(1500g)
    // 출력: 1.1.0: 1.5 kg 소포 (Describe::describe로 부름)
}
