// 객체(데이터 + 동작)와 캡슐화를 확인합니다.
// - 구조체와 impl 블록이 객체 지향에서 말하는 "객체"(데이터와 그 데이터를 다루는 메서드)에 해당합니다.
// - 필드를 비공개로 두고 pub 메서드만 열어 두면, 바깥 코드가 불변식을 깨뜨릴 수 없습니다.
//   여기서 지킬 불변식은 "average는 언제나 list에 든 값들의 평균"입니다.
// - 공개 API가 같으면 내부 표현을 바꿔도 사용하는 코드는 고칠 필요가 없습니다.
//
// 실행: cargo run -p p0-ch17-trait-objects-oop --example 01_encapsulation

// 평균을 미리 계산해 필드에 저장해 두는(캐시하는) 버전입니다. The Book 18.1절의 AveragedCollection과 같습니다.
mod cached {
    // 구조체는 pub이지만 필드는 pub이 아닙니다. 모듈 밖에서는 필드를 읽지도 쓰지도 못합니다.
    // derive(Default)는 빈 Vec과 0.0으로 채운 값을 만드는 default()를 만들어 줍니다(ch10의 derive 목록).
    #[derive(Default)]
    pub struct AveragedCollection {
        list: Vec<i32>,
        average: f64,
    }

    impl AveragedCollection {
        // 필드가 비공개이므로 모듈 밖에서는 `AveragedCollection { ... }`으로 만들 수 없습니다.
        // 그래서 값을 만드는 공개 연관 함수가 꼭 필요합니다(The Book의 코드에는 생략되어 있습니다).
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add(&mut self, value: i32) {
            self.list.push(value);
            self.update_average(); // list가 바뀌는 곳마다 평균도 함께 갱신합니다.
        }

        pub fn remove(&mut self) -> Option<i32> {
            let value = self.list.pop()?; // 비어 있으면 None을 그대로 돌려줍니다(ch9의 `?`).
            self.update_average();
            Some(value)
        }

        // 읽기만 허용하는 게터입니다. 바깥에서 average를 바꿀 방법은 없습니다.
        pub fn average(&self) -> f64 {
            self.average
        }

        pub fn count(&self) -> usize {
            self.list.len()
        }

        // pub이 없으므로 모듈 밖에서는 부를 수 없는 내부 도우미입니다.
        fn update_average(&mut self) {
            if self.list.is_empty() {
                // 0으로 나누면 f64는 NaN이 되므로, 빈 목록의 평균은 0.0으로 정해 둡니다.
                self.average = 0.0;
                return;
            }
            let total: i32 = self.list.iter().sum();
            self.average = total as f64 / self.list.len() as f64;
        }
    }
}

// 공개 API(new, add, remove, average, count)는 똑같고 내부 표현만 다른 버전입니다.
// 평균을 저장하지 않고 합계를 들고 있다가, average()를 부를 때 계산합니다.
mod on_demand {
    #[derive(Default)]
    pub struct AveragedCollection {
        list: Vec<i32>,
        total: i64,
    }

    impl AveragedCollection {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add(&mut self, value: i32) {
            self.list.push(value);
            self.total += i64::from(value);
        }

        pub fn remove(&mut self) -> Option<i32> {
            let value = self.list.pop()?;
            self.total -= i64::from(value);
            Some(value)
        }

        pub fn average(&self) -> f64 {
            if self.list.is_empty() {
                return 0.0;
            }
            self.total as f64 / self.list.len() as f64
        }

        pub fn count(&self) -> usize {
            self.list.len()
        }
    }
}

// 비교용: 필드를 모두 공개한 버전입니다. 구조는 cached와 같습니다.
mod leaky {
    pub struct LeakyCollection {
        pub list: Vec<i32>,
        pub average: f64,
    }

    impl LeakyCollection {
        pub fn add(&mut self, value: i32) {
            self.list.push(value);
            let total: i32 = self.list.iter().sum();
            self.average = total as f64 / self.list.len() as f64;
        }
    }
}

fn main() {
    println!("1. 객체 = 데이터(필드) + 동작(메서드)");
    let mut scores = cached::AveragedCollection::new();
    scores.add(70);
    scores.add(85);
    scores.add(90);
    println!("    값 {}개, 평균 {:.2}", scores.count(), scores.average()); // 출력: 값 3개, 평균 81.67

    println!("2. 공개 메서드로만 바꿀 수 있으므로 평균이 언제나 맞습니다");
    let removed = scores.remove();
    println!(
        "    remove() = {removed:?}, 남은 값 {}개, 평균 {:.2}",
        scores.count(),
        scores.average()
    ); // 출력: remove() = Some(90), 남은 값 2개, 평균 77.50
    scores.remove();
    scores.remove();
    let none = scores.remove(); // 빈 목록에서 꺼내면 None입니다.
    println!(
        "    모두 꺼낸 뒤: remove() = {none:?}, 평균 {:.2}",
        scores.average()
    ); // 출력: 모두 꺼낸 뒤: remove() = None, 평균 0.00
    // scores.list.push(100); 처럼 필드에 직접 손대면 E0616으로 거부됩니다(compile_fail/e0616_private_field.rs).

    println!("3. 필드가 공개되어 있으면 바깥 코드가 불변식을 깰 수 있습니다");
    let mut leaky = leaky::LeakyCollection {
        list: Vec::new(),
        average: 0.0,
    };
    leaky.add(70);
    leaky.add(85);
    // add를 거치지 않고 list를 직접 바꿨습니다. average는 아무도 갱신하지 않습니다.
    leaky.list.push(0);
    println!(
        "    list = {:?}, average = {:.2}",
        leaky.list, leaky.average
    ); // 출력: list = [70, 85, 0], average = 77.50
    // 실제 평균은 51.67인데 77.50이 남아 있습니다. average를 직접 덮어쓰는 것도 막을 수 없습니다.
    leaky.average = 100.0;
    println!("    average를 덮어쓴 뒤: {:.2}", leaky.average); // 출력: average를 덮어쓴 뒤: 100.00

    println!("4. 내부 표현이 달라도 공개 API가 같으면 쓰는 코드는 그대로입니다");
    // 두 블록의 코드는 `use` 줄만 다르고 글자 하나까지 같습니다.
    {
        use cached::AveragedCollection;
        let mut c = AveragedCollection::new();
        c.add(3);
        c.add(4);
        c.add(8);
        let last = c.remove();
        println!(
            "    cached:    꺼낸 값 {last:?}, 남은 값 {}개, 평균 {:.2}",
            c.count(),
            c.average()
        ); // 출력: cached:    꺼낸 값 Some(8), 남은 값 2개, 평균 3.50
    }
    {
        use on_demand::AveragedCollection;
        let mut c = AveragedCollection::new();
        c.add(3);
        c.add(4);
        c.add(8);
        let last = c.remove();
        println!(
            "    on_demand: 꺼낸 값 {last:?}, 남은 값 {}개, 평균 {:.2}",
            c.count(),
            c.average()
        ); // 출력: on_demand: 꺼낸 값 Some(8), 남은 값 2개, 평균 3.50
    }
    // 만약 list가 공개되어 바깥 코드가 c.list.push(..)를 쓰고 있었다면,
    // on_demand로 바꾸는 순간 total이 맞지 않게 되어 그 코드도 모두 고쳐야 했을 것입니다.
}
