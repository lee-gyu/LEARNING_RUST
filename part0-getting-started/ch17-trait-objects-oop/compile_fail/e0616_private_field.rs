// 기대하는 에러: E0616 — field `list` of struct `AveragedCollection` is private
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0616_private_field.rs
//
// list와 average는 비공개 필드라서 stats 모듈 밖에서는 읽을 수도, 바꿀 수도 없습니다.
// 이 제약 덕분에 "average는 언제나 list의 평균"이라는 불변식을 add와 remove만 책임지면 됩니다.
// 바깥에서 list.push를 할 수 있었다면 average는 갱신되지 않은 채 남았을 것입니다.
// 해결: 공개 메서드 add를 씁니다. 필드에 pub을 붙여 에러를 없애는 것은 캡슐화를 버리는 것입니다.

mod stats {
    pub struct AveragedCollection {
        list: Vec<i32>,
        average: f64,
    }

    impl AveragedCollection {
        pub fn new() -> Self {
            AveragedCollection {
                list: Vec::new(),
                average: 0.0,
            }
        }

        pub fn add(&mut self, value: i32) {
            self.list.push(value);
            self.update_average();
        }

        pub fn average(&self) -> f64 {
            self.average
        }

        fn update_average(&mut self) {
            let total: i32 = self.list.iter().sum();
            self.average = total as f64 / self.list.len() as f64;
        }
    }
}

use stats::AveragedCollection;

fn main() {
    let mut scores = AveragedCollection::new();
    scores.add(70);
    scores.list.push(100); // add를 거치지 않고 list를 직접 바꾸려 합니다.
    println!("평균: {:.2}", scores.average());
}
