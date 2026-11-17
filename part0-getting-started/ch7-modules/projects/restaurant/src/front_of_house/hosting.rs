// 모듈 front_of_house::hosting의 내용입니다 (src/front_of_house/hosting.rs).
// 파일의 위치가 모듈 트리의 위치(front_of_house 아래의 hosting)와 같습니다.
// 파일 전체가 이미 hosting 모듈의 본문이므로, 여기에 `mod hosting { ... }` 블록을 다시 쓰지 않습니다.
//
// 출력 앞의 [...]는 module_path!()로 얻은, 이 코드가 있는 모듈의 경로입니다.

pub fn add_to_waitlist(name: &str) {
    println!("  [{}] {name}님을 대기 명단에 추가했습니다", module_path!());
}

pub fn seat_at_table(name: &str) {
    println!("  [{}] {name}님을 자리로 안내했습니다", module_path!());
}
