// 지침 3: 매크로는 인자를 한 번만 평가합니다.
//
// 매크로는 인자의 "값"이 아니라 "코드"를 받아 펼칩니다. 본문에 `$value`를 두 번 쓰면
// 부르는 쪽이 넘긴 식이 두 번 실행됩니다. 부수 효과가 있는 식이면 그 효과도 두 번 일어나고,
// 두 번의 결과가 다르면 비교한 값과 돌려준 값이 서로 다른 값이 됩니다.
//
// 실행: cargo run -p er-ch28-macros-judiciously --example 04_evaluate_once

/// 읽을 때마다 다음 값을 내놓는 센서입니다. 몇 번 읽었는지 셉니다.
struct Sensor {
    readings: Vec<i32>,
    reads: usize,
}

impl Sensor {
    fn new(readings: &[i32]) -> Self {
        Sensor {
            readings: readings.to_vec(),
            reads: 0,
        }
    }

    fn read(&mut self) -> i32 {
        let value = self.readings[self.reads % self.readings.len()];
        self.reads += 1;
        value
    }
}

// 비권장: `$value`를 조건에 한 번, 결과에 한 번 씁니다.
macro_rules! at_least {
    ($value:expr, $min:expr) => {
        if $value < $min { $min } else { $value }
    };
}

// 권장 ①: 지역 변수에 한 번 바인드한 뒤 그 변수만 씁니다.
// 매크로 안의 `value`, `min`은 위생(hygiene) 덕분에 부르는 쪽의 같은 이름 변수와 섞이지 않습니다.
macro_rules! at_least_once {
    ($value:expr, $min:expr) => {{
        let value = $value;
        let min = $min;
        if value < min { min } else { value }
    }};
}

// 권장 ②: 프래그먼트를 `ident`로 좁혀, 부르는 쪽이 먼저 변수에 담게 합니다.
// 변수 이름은 몇 번 평가해도 부수 효과가 없습니다. `at_least_ident!(sensor.read(), 5)`는 컴파일되지 않습니다
// (compile_fail/no_rules_expected_ident.rs).
macro_rules! at_least_ident {
    ($value:ident, $min:expr) => {
        if $value < $min { $min } else { $value }
    };
}

// 권장 ③: 애초에 함수로 씁니다. 함수의 인자는 부르기 전에 정확히 한 번 평가됩니다.
// (표준 라이브러리의 `Ord::max`가 이미 같은 일을 합니다: `sensor.read().max(5)`)
fn at_least<T: PartialOrd>(value: T, min: T) -> T {
    if value < min { min } else { value }
}

fn report(label: &str, result: i32, sensor: &Sensor) {
    println!(
        "  {label:<28} 결과 {result}, 센서를 읽은 횟수 {}",
        sensor.reads
    );
}

fn main() {
    // 센서는 7, 2, 7, 2, ... 를 차례로 내놓습니다. 최솟값은 5입니다.
    let readings = [7, 2];

    println!("1. 비권장: 인자를 두 번 평가하는 매크로");
    let mut sensor = Sensor::new(&readings);
    // 펼친 모양: if sensor.read() < 5 { 5 } else { sensor.read() }
    // 조건에서 7을 읽어 "5 이상"이라고 판단한 뒤, 결과로는 다시 읽은 2를 돌려줍니다.
    let r = at_least!(sensor.read(), 5);
    report("at_least!(sensor.read(), 5)", r, &sensor); // 출력:   at_least!(sensor.read(), 5)  결과 2, 센서를 읽은 횟수 2

    println!("2. 권장: 한 번만 평가");
    let mut sensor = Sensor::new(&readings);
    let r = at_least_once!(sensor.read(), 5);
    report("at_least_once!", r, &sensor); // 출력:   at_least_once!               결과 7, 센서를 읽은 횟수 1

    // 부르는 쪽에도 `value`라는 변수가 있지만, 매크로 안의 `let value = ...`와 섞이지 않습니다.
    // 이번 센서는 2를 내놓습니다. 글자를 바꿔 끼우는 C 전처리기 매크로였다면 펼친 코드의
    // `let min = value;`가 방금 읽은 2를 가리켜 결과가 2였을 것입니다. 위생 덕분에 부르는 쪽의 5를 가리킵니다.
    let mut sensor = Sensor::new(&[2]);
    let value = 5;
    let r = at_least_once!(sensor.read(), value);
    report("at_least_once!(.., value)", r, &sensor); // 출력:   at_least_once!(.., value)    결과 5, 센서를 읽은 횟수 1

    let mut sensor = Sensor::new(&readings);
    let reading = sensor.read();
    let r = at_least_ident!(reading, 5);
    report("at_least_ident!(reading, 5)", r, &sensor); // 출력:   at_least_ident!(reading, 5)  결과 7, 센서를 읽은 횟수 1

    let mut sensor = Sensor::new(&readings);
    let r = at_least(sensor.read(), 5);
    report("at_least(sensor.read(), 5)", r, &sensor); // 출력:   at_least(sensor.read(), 5)   결과 7, 센서를 읽은 횟수 1
}
