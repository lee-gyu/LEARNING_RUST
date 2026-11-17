// 기대하는 에러: E0382 borrow of partially moved value: `pair`
// 원인: _greeting은 이름이 _로 시작할 뿐 보통의 변수입니다. "쓰지 않는 변수" 경고만 꺼질 뿐,
//       값을 묶기는 하므로 String의 소유권이 _greeting으로 이동합니다.
//       그래서 pair는 일부가 빠져나간 값(부분 이동)이 되어 통째로 쓸 수 없습니다.
// 해결: 값이 필요 없다면 이름 대신 _를 씁니다. _는 아무것도 묶지 않으므로 이동도 일어나지 않습니다.
//       let (_, count) = pair;    (examples/06_ignoring.rs의 4번 구역)
//
// 확인: rustc --edition 2024 compile_fail/e0382_underscore_name_moves.rs

fn main() {
    let pair = (String::from("안녕"), 3);

    let (_greeting, count) = pair;
    println!("count = {count}");

    println!("{pair:?}");
}
