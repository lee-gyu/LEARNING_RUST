// 지침 4-②: 매크로가 인자에 몰래 `&`나 `&mut`를 붙이지 않게 합니다.
//
// 보통의 Rust에서 `f(scores)`는 scores를 옮기거나(이동) 복사합니다. 고치려면 `f(&mut scores)`라고 써야 합니다.
// 매크로는 받은 코드 앞에 `&mut`를 붙여 펼칠 수 있으므로, `m!(scores)`가 scores를 그 자리에서 고쳐도
// 부르는 쪽에는 아무 표시가 없습니다. 빌림은 부르는 쪽이 `m!(&mut scores)`처럼 직접 쓰게 합니다.
//
// 실행: cargo run -p er-ch28-macros-judiciously --example 06_hidden_reference

// 비권장 ①: 매크로 안에서 `&mut`를 붙여 부르는 쪽의 벡터를 고칩니다.
// 최고점을 100으로 맞춰 다른 점수를 비례해 바꿉니다.
macro_rules! normalize_v1 {
    ($scores:expr) => {{
        let scores = &mut $scores; // 여기서 몰래 가변으로 빌립니다.
        let max = scores.iter().copied().max().unwrap_or(1).max(1);
        for s in scores.iter_mut() {
            *s = *s * 100 / max;
        }
    }};
}

// 비권장 ②: 매크로 안에서 `&`를 붙입니다. 부르는 쪽은 names를 넘겨준 것처럼 보입니다.
macro_rules! roster_v1 {
    ($names:expr) => {
        roster(&$names)
    };
}

// 권장 ①: 함수. 시그니처가 `&mut [u32]`이므로 부르는 쪽은 `&mut scores`라고 써야 합니다.
fn normalize(scores: &mut [u32]) {
    let max = scores.iter().copied().max().unwrap_or(1).max(1);
    for s in scores.iter_mut() {
        *s = *s * 100 / max;
    }
}

// 권장 ②: 매크로가 꼭 필요하더라도 빌림은 부르는 쪽에 둡니다.
// 본문이 인자를 `&mut [u32]`로 받으므로 `normalize_v2!(scores)`라고 쓰면 타입 에러(E0308)입니다.
macro_rules! normalize_v2 {
    ($scores:expr) => {
        normalize($scores)
    };
}

fn roster(names: &[String]) -> String {
    format!("{}명: {}", names.len(), names.join(", "))
}

fn main() {
    println!("1. 비권장: 매크로가 몰래 빌림");
    let mut scores = vec![40, 80, 20];
    normalize_v1!(scores);
    // 함수 호출처럼 보이는 줄 뒤에도 scores를 쓸 수 있고, 값은 그 자리에서 바뀌었습니다.
    println!("  normalize_v1!(scores) 뒤 scores = {scores:?}"); // 출력:   normalize_v1!(scores) 뒤 scores = [50, 100, 25]

    let names = vec!["하나".to_string(), "두리".to_string(), "세찬".to_string()];
    let line = roster_v1!(names);
    // `f(names)`였다면 names는 옮겨져 여기서 쓸 수 없습니다(E0382). 매크로라서 아무 표시 없이 빌렸습니다.
    let count = names.len();
    println!("  roster_v1!(names) = {line}, 뒤에도 names.len() = {count}"); // 출력:   roster_v1!(names) = 3명: 하나, 두리, 세찬, 뒤에도 names.len() = 3

    println!("2. 권장: 빌림을 부르는 쪽에 드러냄");
    let mut scores = vec![40, 80, 20];
    normalize(&mut scores);
    println!("  normalize(&mut scores) 뒤 scores = {scores:?}"); // 출력:   normalize(&mut scores) 뒤 scores = [50, 100, 25]

    let mut scores = vec![30, 60, 15];
    normalize_v2!(&mut scores);
    println!("  normalize_v2!(&mut scores) 뒤 scores = {scores:?}"); // 출력:   normalize_v2!(&mut scores) 뒤 scores = [50, 100, 25]

    let line = roster(&names);
    println!("  roster(&names) = {line}"); // 출력:   roster(&names) = 3명: 하나, 두리, 세찬
}
