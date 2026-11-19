//! 지침 4: 서로 다른 부분은 따로 빌린다. (책에 없는 지침)
//!
//! 정원의 화분을 돌며 물을 주고, 물탱크에서 물을 덜고, 일지에 적습니다.
//! 화분 목록을 `&mut`로 돌면서 `&mut self` 메서드를 부르면 E0499입니다
//! (compile_fail/e0499_method_borrows_whole_self.rs). 메서드는 필드 하나가 아니라 `self` 전체를 빌리기 때문입니다.
//! 에러를 피하려고 목록의 복제본을 도는 판(비권장)은 컴파일되지만 화분이 젖지 않습니다.
//! 필드를 직접 나눠 빌리는 판, 메서드를 필드 단위 함수로 쪼갠 판(권장)과 비교하고,
//! 마지막 구역은 같은 슬라이스의 두 원소를 함께 고치는 `get_disjoint_mut`입니다.
//!
//! 실행: cargo run -p er-ch15-borrow-checker --example 04_split_borrows

#[derive(Debug, Clone)]
struct Plant {
    name: &'static str,
    moisture: u32, // 0~100
}

struct Garden {
    plants: Vec<Plant>,
    tank_liters: u32,
    journal: Vec<String>,
}

impl Garden {
    fn new() -> Garden {
        Garden {
            plants: vec![
                Plant {
                    name: "바질",
                    moisture: 20,
                },
                Plant {
                    name: "로즈메리",
                    moisture: 35,
                },
                Plant {
                    name: "토마토",
                    moisture: 10,
                },
            ],
            tank_liters: 5,
            journal: Vec::new(),
        }
    }

    /// 탱크에서 물을 덥니다. 남은 물이 모자라면 있는 만큼만.
    fn pour(&mut self, liters: u32) -> u32 {
        let used = self.tank_liters.min(liters);
        self.tank_liters -= used;
        used
    }

    fn note(&mut self, line: String) {
        self.journal.push(line);
    }

    /// 비권장: `for plant in &mut self.plants { self.pour(2); .. }`가 E0499라서
    /// 복제본을 돕니다. 컴파일은 되지만 물을 준 것은 사본입니다.
    fn water_all_on_copy(&mut self) {
        for mut plant in self.plants.clone() {
            let used = self.pour(2);
            plant.moisture += used * 10;
            self.note(format!("{} {}%", plant.name, plant.moisture));
        }
    }

    /// 권장 1: 필드를 구조 분해해 각각 따로 빌립니다.
    /// `plants`, `tank_liters`, `journal`은 서로 겹치지 않으므로 대여 검사기가 받아 줍니다.
    fn water_all_split_fields(&mut self) {
        let Garden {
            plants,
            tank_liters,
            journal,
        } = self;
        for plant in plants.iter_mut() {
            let used = (*tank_liters).min(2);
            *tank_liters -= used;
            plant.moisture += used * 10;
            journal.push(format!("{} {}%", plant.name, plant.moisture));
        }
    }

    /// 권장 2: `&mut self` 대신 필요한 필드만 받는 연관 함수로 쪼갭니다.
    fn pour_from(tank_liters: &mut u32, liters: u32) -> u32 {
        let used = (*tank_liters).min(liters);
        *tank_liters -= used;
        used
    }

    fn water_all_field_functions(&mut self) {
        for plant in &mut self.plants {
            let used = Self::pour_from(&mut self.tank_liters, 2); // tank_liters만 빌림
            plant.moisture += used * 10;
            // self.note(..)는 self 전체를 빌리지만, 필드를 직접 쓰면 journal만 빌립니다.
            self.journal
                .push(format!("{} {}%", plant.name, plant.moisture));
        }
    }

    fn show(&self, label: &str) {
        println!("{label}");
        let state: Vec<String> = self
            .plants
            .iter()
            .map(|p| format!("{} {}%", p.name, p.moisture))
            .collect();
        println!("  화분: {}", state.join(", "));
        println!(
            "  탱크 남은 물: {}L, 일지: {:?}",
            self.tank_liters, self.journal
        );
    }
}

fn main() {
    let mut garden = Garden::new();
    garden.water_all_on_copy();
    garden.show("[비권장] 복제본을 돌며 물 주기");
    // 출력:   화분: 바질 20%, 로즈메리 35%, 토마토 10%      ← 그대로
    // 출력:   탱크 남은 물: 0L, 일지: ["바질 40%", "로즈메리 55%", "토마토 20%"]

    let mut garden = Garden::new();
    garden.water_all_split_fields();
    garden.show("[권장 1] 필드를 나눠 빌리기");
    // 출력:   화분: 바질 40%, 로즈메리 55%, 토마토 20%
    // 출력:   탱크 남은 물: 0L, 일지: ["바질 40%", "로즈메리 55%", "토마토 20%"]

    let mut garden = Garden::new();
    garden.water_all_field_functions();
    garden.show("[권장 2] 메서드를 필드 단위 함수로 쪼개기");
    // 출력: (권장 1과 같음)

    println!("[슬라이스] 심지로 이은 두 화분의 수분을 고르게 맞추기");
    let mut plants = Garden::new().plants;
    // &mut plants[0]과 &mut plants[2]를 함께 만들면 E0499입니다.
    // get_disjoint_mut은 인덱스가 겹치지 않는지 실행 중에 확인하고 &mut 두 개를 줍니다.
    if let Ok([a, b]) = plants.get_disjoint_mut([0, 2]) {
        let even = (a.moisture + b.moisture) / 2;
        a.moisture = even;
        b.moisture = even;
    }
    println!(
        "  {} {}%, {} {}%",
        plants[0].name, plants[0].moisture, plants[2].name, plants[2].moisture
    );
    // 출력:   바질 15%, 토마토 15%
    for indices in [[1, 1], [0, 3]] {
        match plants.get_disjoint_mut(indices) {
            Ok(_) => println!("  {indices:?}: 성공"),
            Err(e) => println!("  {indices:?}: {e}"),
        }
    }
    // 출력:   [1, 1]: there were overlapping indices
    // 출력:   [0, 3]: an index is out of bounds
    // 앞뒤로 나눌 수 있는 경우는 split_at_mut으로도 됩니다.
    let (front, back) = plants.split_at_mut(1);
    back[0].moisture = front[0].moisture;
    println!("  split_at_mut 뒤 로즈메리: {}%", plants[1].moisture);
    // 출력:   split_at_mut 뒤 로즈메리: 15%
}
