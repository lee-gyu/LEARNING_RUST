// 에디션별 결과: rustc 1.100.0에서는 2024와 2021 모두 실패
// 기대하는 에러(두 에디션): E0277 — the trait bound `!: Default` is not satisfied
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_never_type_fallback.rs --out-dir /tmp/rust-p2-ch4   # 실패
//   rustc --edition 2021 compile_fail/e0277_never_type_fallback.rs --out-dir /tmp/rust-p2-ch4   # 실패
//
// `load()?;`에서 `load`의 타입 매개변수 `T`를 정해 줄 단서가 없습니다(결과를 버리므로).
// 이럴 때 컴파일러가 고르는 "폴백 타입"이 예전에는 `()`였고, 2024 에디션에서 `!`로 바뀌었습니다.
// `!`는 `Default`를 구현하지 않으므로 `T: Default` 바운드를 만족하지 못합니다.
// Edition Guide는 rustc 1.100.0부터 이 변경이 에디션과 상관없이 모든 에디션에 적용된다고
// 적고 있고, 실제로 2021 에디션으로 컴파일해도 같은 에러가 납니다.
// (이 저장소가 예전에 쓰던 rustc 1.98.1에서는 2021 에디션일 때 에러 대신 deny 린트
//  `dependency_on_unit_never_type_fallback`이 났습니다. 그 린트는 1.100.0에서 없어졌습니다.)
// 해결: 타입을 직접 적습니다. `load::<()>()?;` 또는 `let () = load()?;`

fn load<T: Default>() -> Result<T, String> {
    Ok(T::default())
}

fn run() -> Result<u32, String> {
    load()?; // → E0277
    Ok(1)
}

fn main() {
    println!("{:?}", run());
}
