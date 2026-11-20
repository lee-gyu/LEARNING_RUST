// 기대하는 에러: error[E0382]: borrow of moved value: `nickname`
// 핵심 메시지: `nickname` moved due to this method call / value borrowed here after move
//             note: `std::option::Option::<T>::map` takes ownership of the receiver `self`, which moves `nickname`
//             help: consider calling `.as_ref()` to borrow the value's contents
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0382_map_consumes_option.rs --out-dir /tmp/rust-p1-ch2
//
// map은 self를 받으므로 nickname의 소유권이 map 안으로 옮겨 가고, 클로저가 안의 String을 받아 씁니다.
// 그 뒤에 nickname을 출력하려 하면 이미 옮겨 간 값을 빌리려는 것이라 거부됩니다.
// Option<i32>처럼 안의 타입이 Copy면 Option도 Copy라서 같은 코드가 컴파일됩니다(복사본이 넘어가므로).
// 이 에러는 Copy가 아닌 타입(String, Vec 등)을 담은 Option에서만 납니다.
// 고치는 방법: 원본을 계속 쓸 것이라면 참조를 담은 Option을 소비하게 합니다.
//   let len = nickname.as_ref().map(|s| s.chars().count());   // 또는 nickname.as_deref().map(str::len) 등
// 원본이 더 필요 없다면 map을 마지막에 쓰도록 순서를 바꿉니다.

fn main() {
    let nickname: Option<String> = Some(String::from("페리스"));
    let len = nickname.map(|s| s.chars().count());
    println!("글자 수: {len:?}");
    println!("원본: {nickname:?}");
}
