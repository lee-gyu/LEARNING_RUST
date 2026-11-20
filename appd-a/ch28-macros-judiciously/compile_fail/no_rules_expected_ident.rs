// 기대하는 에러(에러 코드 없음): no rules expected `.`
//   note: while trying to match `,`
//
// examples/04_evaluate_once.rs의 `at_least_ident!`는 첫 인자를 `ident`(이름 하나)로만 받습니다.
// `sensor.read()`를 넘기면 `sensor`까지는 ident로 맞지만, 다음 토큰 `.`이 규칙의 `,` 자리와 맞지 않습니다.
// 프래그먼트를 좁힌 것이 의도대로 동작한 것입니다. 부르는 쪽에서 `let reading = sensor.read();`로 먼저 바인드합니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/no_rules_expected_ident.rs

struct Sensor {
    value: i32,
}

impl Sensor {
    fn read(&mut self) -> i32 {
        self.value
    }
}

macro_rules! at_least_ident {
    ($value:ident, $min:expr) => {
        if $value < $min { $min } else { $value }
    };
}

fn main() {
    let mut sensor = Sensor { value: 7 };
    let r = at_least_ident!(sensor.read(), 5);
    println!("{r}");
}
