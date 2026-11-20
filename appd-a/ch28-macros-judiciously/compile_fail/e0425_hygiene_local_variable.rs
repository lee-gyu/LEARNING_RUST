// 기대하는 에러: E0425 cannot find value `total` in this scope
//   help: an identifier with the same name exists, but is not accessible due to macro hygiene
//
// macro_rules! 매크로는 위생적(hygienic)입니다. 매크로 본문에 적은 지역 변수 이름은
// 매크로를 부른 곳의 지역 변수를 가리키지 못합니다. 부르는 쪽의 `total`을 몰래 고치려던 매크로가 거부됩니다.
// 고치는 방법: 고칠 대상을 인자로 받습니다. 예: `add_to!(total, 5)` → `$target += $x`
// (그리고 지침 4에 따라 `add_to!(&mut total, 5)`처럼 빌림을 드러내면 더 좋습니다.)
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0425_hygiene_local_variable.rs

macro_rules! add_to_total {
    ($x:expr) => {
        total += $x
    };
}

fn main() {
    let mut total = 0;
    add_to_total!(5);
    println!("{total}");
}
