// 호환되는 변경(마이너 버전)에서는 사용자 코드가 한 글자도 바뀌지 않고 같은 결과를 냅니다.
//
// 실행: cargo run -p er-ch21-semver --example 01_compatible_changes
//
// 가상의 택배 라이브러리 `parcel`의 두 버전을 모듈 `v1_0`(1.0.0)과 `v1_1`(1.1.0)로 흉내 냅니다.
// 1.1.0은 1.0.0에 다음 다섯 가지를 더했습니다. 모두 Cargo Book의 SemVer 규칙에서 마이너로
// 분류하는 변경입니다(괄호 안은 그 규칙의 이름).
//   1. 새 공개 함수 `estimate_days` (item-new)
//   2. `Parcel`의 새 고유 메서드 `is_heavy` (impl-item-new)
//   3. 트레이트 `Carrier`의 기본 구현이 있는 새 메서드 `describe` (trait-new-default-item)
//   4. 이미 비공개 필드가 있는 `Parcel`에 비공개 필드 `fragile` 추가 (struct-private-fields-with-private)
//   5. `summarize`의 바운드 `T: Display + Copy`를 `T: Display`로 느슨하게 (generic-bounds-loosen)
//
// 사용자 코드는 매크로 `user_code!` 하나로 적고 두 버전에 대해 각각 펼칩니다. 같은 글자가 두 번
// 컴파일된다는 것을 보장하려고 매크로를 썼습니다(실제로는 Cargo.toml의 버전만 바꿔 다시 빌드하는 것).

/// parcel 1.0.0
mod v1_0 {
    use std::fmt::Display;

    pub struct Parcel {
        weight_g: u32, // 비공개 필드: 사용자는 new로만 만듭니다
    }

    impl Parcel {
        pub fn new(weight_g: u32) -> Self {
            Parcel { weight_g }
        }
        pub fn weight_g(&self) -> u32 {
            self.weight_g
        }
    }

    /// 배송사. 사용자가 자기 배송사를 구현할 수 있습니다.
    pub trait Carrier {
        fn name(&self) -> String;
        fn fee(&self, parcel: &Parcel) -> u32;
    }

    /// 1.0.0의 바운드에는 본문이 쓰지 않는 `Copy`가 들어 있었습니다.
    pub fn summarize<T: Display + Copy>(weights: &[T]) -> String {
        let parts: Vec<String> = weights.iter().map(|w| format!("{w}kg")).collect();
        parts.join(" + ")
    }
}

/// parcel 1.1.0: 1.0.0에 다섯 가지를 더했습니다.
mod v1_1 {
    use std::fmt::Display;

    pub struct Parcel {
        weight_g: u32,
        fragile: bool, // 4. 새 비공개 필드. 사용자는 원래 리터럴로 만들 수 없었으므로 깨지지 않습니다
    }

    impl Parcel {
        pub fn new(weight_g: u32) -> Self {
            Parcel {
                weight_g,
                fragile: false,
            }
        }
        pub fn weight_g(&self) -> u32 {
            self.weight_g
        }
        /// 새로 더한 세터
        pub fn fragile(mut self) -> Self {
            self.fragile = true;
            self
        }
        /// 2. 새 고유 메서드
        pub fn is_heavy(&self) -> bool {
            self.weight_g >= 5000
        }
        pub fn is_fragile(&self) -> bool {
            self.fragile
        }
    }

    pub trait Carrier {
        fn name(&self) -> String;
        fn fee(&self, parcel: &Parcel) -> u32;

        /// 3. 기본 구현이 있는 새 메서드. 기존 구현은 고칠 것이 없습니다.
        fn describe(&self, parcel: &Parcel) -> String {
            format!("{}: {}원", self.name(), self.fee(parcel))
        }
    }

    /// 5. 바운드를 느슨하게 했습니다. 전에 넘기던 타입은 여전히 모두 넘길 수 있습니다.
    pub fn summarize<T: Display>(weights: &[T]) -> String {
        let parts: Vec<String> = weights.iter().map(|w| format!("{w}kg")).collect();
        parts.join(" + ")
    }

    /// 1. 새 공개 함수
    pub fn estimate_days(weight_g: u32) -> u32 {
        if weight_g >= 5000 { 3 } else { 1 }
    }
}

/// 1.0.0 시절에 쓴 사용자 코드. 라이브러리 버전만 바꿔 같은 글자를 컴파일합니다.
macro_rules! user_code {
    ($parcel:ident) => {{
        use $parcel::{Carrier, Parcel};

        // 사용자가 구현한 배송사. 1.1.0에서 Carrier에 메서드가 늘었지만 이 구현은 그대로입니다.
        struct QuickBike;
        impl Carrier for QuickBike {
            fn name(&self) -> String {
                String::from("번개 퀵")
            }
            fn fee(&self, parcel: &Parcel) -> u32 {
                3000 + parcel.weight_g() / 100 * 10
            }
        }

        let bike = QuickBike;
        let parcels = [Parcel::new(1200), Parcel::new(800)];
        let mut lines = Vec::new();
        for p in &parcels {
            lines.push(format!("{} {}g → {}원", bike.name(), p.weight_g(), bike.fee(p)));
        }
        // f64는 Copy이므로 1.0.0의 바운드도, 1.1.0의 느슨한 바운드도 만족합니다.
        lines.push(format!("무게 합계: {}", $parcel::summarize(&[1.2_f64, 0.8])));
        lines
    }};
}

fn main() {
    let on_1_0 = user_code!(v1_0);
    let on_1_1 = user_code!(v1_1);

    println!("[같은 사용자 코드를 parcel 1.0.0으로 컴파일]");
    for line in &on_1_0 {
        println!("  {line}");
    }
    println!("[같은 사용자 코드를 parcel 1.1.0으로 컴파일]");
    for line in &on_1_1 {
        println!("  {line}");
    }
    println!("두 결과가 같은가: {}", on_1_0 == on_1_1);
    // 출력:
    // [같은 사용자 코드를 parcel 1.0.0으로 컴파일]
    //   번개 퀵 1200g → 3120원
    //   번개 퀵 800g → 3080원
    //   무게 합계: 1.2kg + 0.8kg
    // [같은 사용자 코드를 parcel 1.1.0으로 컴파일]
    //   번개 퀵 1200g → 3120원
    //   번개 퀵 800g → 3080원
    //   무게 합계: 1.2kg + 0.8kg
    // 두 결과가 같은가: true

    // 1.1.0으로 올린 뒤에 새로 쓴 코드는 더해진 것을 씁니다.
    // 이 코드는 1.0.0으로는 컴파일되지 않으므로, 이런 코드를 쓰기 시작했다면
    // Cargo.toml의 요구 사항도 "1.1"로 올려야 합니다(README 2.2절).
    println!("[1.1.0에서 새로 쓴 코드]");
    let vase = v1_1::Parcel::new(6400).fragile();
    println!(
        "  6400g, 무거움 {}, 깨지기 쉬움 {}, 예상 {}일",
        vase.is_heavy(),
        vase.is_fragile(),
        v1_1::estimate_days(vase.weight_g())
    );
    // Copy가 아닌 String도 이제 summarize에 넘길 수 있습니다.
    let labels = [String::from("1.5"), String::from("2.0")];
    println!("  무게 합계: {}", v1_1::summarize(&labels));

    // 필수 메서드 두 개만 구현해도 새 기본 메서드 describe를 얻습니다.
    struct NightPost;
    impl v1_1::Carrier for NightPost {
        fn name(&self) -> String {
            String::from("심야 우편")
        }
        fn fee(&self, parcel: &v1_1::Parcel) -> u32 {
            if parcel.is_heavy() { 7000 } else { 4000 }
        }
    }
    use v1_1::Carrier;
    println!("  {}", NightPost.describe(&vase));
    // 출력:
    // [1.1.0에서 새로 쓴 코드]
    //   6400g, 무거움 true, 깨지기 쉬움 true, 예상 3일
    //   무게 합계: 1.5kg + 2.0kg
    //   심야 우편: 7000원
}
