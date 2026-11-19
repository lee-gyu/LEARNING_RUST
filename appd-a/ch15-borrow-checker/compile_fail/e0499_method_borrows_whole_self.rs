// 기대하는 에러: E0499
//   error[E0499]: cannot borrow `*self` as mutable more than once at a time
//
// `for plant in &mut self.plants`는 plants 필드만 빌리지만, 반복문 안의 `self.pour(..)`는
// `&mut self`를 받는 메서드라 구조체 전체를 빌리려 합니다. 컴파일러는 시그니처만 보고
// 판단하므로 pour가 plants를 건드리지 않는다는 것을 알지 못합니다.
// help의 "인덱스 반복문" 제안도 컴파일되지만, 필드를 직접 나눠 빌리거나
// 필요한 필드만 받는 함수로 쪼개는 편이 낫습니다(예제 04).
//
// 확인: rustc --edition 2024 compile_fail/e0499_method_borrows_whole_self.rs --out-dir /tmp/er-ch15

struct Plant {
    name: &'static str,
    moisture: u32,
}

struct Garden {
    plants: Vec<Plant>,
    tank_liters: u32,
}

impl Garden {
    fn pour(&mut self, liters: u32) -> u32 {
        let used = self.tank_liters.min(liters);
        self.tank_liters -= used;
        used
    }

    fn water_all(&mut self) {
        for plant in &mut self.plants {
            let used = self.pour(2); // E0499: self 전체를 다시 가변으로 빌림
            plant.moisture += used * 10;
        }
    }
}

fn main() {
    let mut garden = Garden {
        plants: vec![Plant {
            name: "바질",
            moisture: 20,
        }],
        tank_liters: 5,
    };
    garden.water_all();
    println!("{} {}%", garden.plants[0].name, garden.plants[0].moisture);
}
