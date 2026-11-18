// 기대하는 에러: error[E0689]: can't call method `sqrt` on ambiguous numeric type `{float}`
//
// 원인: 실수 리터럴 `2.0`은 `f32`일 수도 `f64`일 수도 있습니다. 보통은 쓰임새를 보고 타입을 정하고,
//       끝내 단서가 없으면 `f64`로 정합니다. 하지만 메서드를 부르는 시점에는 어느 타입의 메서드를
//       부를지 알아야 하므로 아직 정해지지 않은 `{float}`에는 메서드를 부를 수 없습니다.
// 해결: 리터럴에 접미사를 붙이거나(`2.0_f64`) 변수에 타입을 적습니다(`let x: f64 = 2.0;`).
//
// 확인: rustc --edition 2024 compile_fail/e0689_ambiguous_float_method.rs --out-dir /tmp/rust-p1-ch1

fn main() {
    let root = 2.0.sqrt();
    println!("{root}");

    let x = 2.0;
    println!("{}", x.powi(2));
}
