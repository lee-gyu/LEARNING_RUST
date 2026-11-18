// 이 예제가 보여 주는 것: 튜플과 유닛은 고유 메서드가 없고, 트레이트 구현으로 기능을 얻습니다
//   사전식 비교, `Default`, 배열과의 변환, 원소 12개까지라는 제한, 유닛 `()`
// 실행: cargo run -p p1-ch1-std-overview --example 08_tuples_unit

fn main() {
    // ── 1. 튜플에는 고유 메서드가 없습니다 ───────────────────────────────────
    // 튜플 문서에는 "Implementations" 절이 없고 "Trait Implementations"만 있습니다.
    // 튜플로 할 수 있는 일은 `.0`, `.1` 같은 필드 접근, 패턴 분해, 그리고 트레이트가 주는 기능입니다.
    let point = (3, -2);
    let (x, y) = point;
    println!("x = {x}, y = {y}, point.0 = {}", point.0); // 출력: x = 3, y = -2, point.0 = 3

    // 패턴 분해로 두 값을 맞바꿀 수 있습니다.
    let (a, b) = (1, 2);
    let (a, b) = (b, a);
    println!("a = {a}, b = {b}"); // 출력: a = 2, b = 1

    // ── 2. 사전식 비교 ───────────────────────────────────────────────────
    // `PartialOrd`/`Ord` 구현은 앞 원소부터 비교하다가 처음으로 다른 원소에서 결과를 정합니다.
    // 사전에서 단어를 첫 글자부터 비교하는 것과 같아서 사전식(lexicographic) 비교라고 부릅니다.
    println!("{}", (1, "z") < (2, "a")); // 출력: true   (첫 원소 1 < 2에서 결정)
    println!("{}", (2, "a") < (2, "b")); // 출력: true   (첫 원소가 같아 둘째 원소로 결정)
    // `cmp`(Ord)와 `max`(Ord)도 같은 규칙을 따릅니다. 버전 번호 비교에 그대로 쓸 수 있습니다.
    let current = (1, 98, 1);
    let required = (1, 85, 0);
    println!("{:?}", current.cmp(&required)); // 출력: Greater
    println!("{:?}", current.max(required)); // 출력: (1, 98, 1)

    // ── 3. Default와 배열 변환 ──────────────────────────────────────────────
    // 원소가 모두 `Default`면 튜플도 `Default`입니다.
    let empty: (i32, bool, String) = Default::default();
    println!("{empty:?}"); // 출력: (0, false, "")
    // 원소 타입이 모두 같은 튜플과 배열은 `into()`로 서로 바꿀 수 있습니다(원소 1~12개).
    let rgb: [u8; 3] = (255, 128, 0).into();
    let tuple: (u8, u8, u8) = [10, 20, 30].into();
    println!("{rgb:?} {tuple:?}"); // 출력: [255, 128, 0] (10, 20, 30)

    // ── 4. 원소 12개까지 ─────────────────────────────────────────────────
    // `Debug`, `PartialEq`, `Ord`, `Default`, `Hash` 등은 원소 12개 이하의 튜플에만 구현되어 있습니다(튜플 문서 참고).
    let twelve = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
    println!("{twelve:?}"); // 출력: (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    // 원소가 13개면 `{:?}`로 출력할 수 없습니다(E0277). 이렇게 큰 묶음은 구조체(Part 0 ch5)로 만드는 편이 낫습니다.
    // let thirteen = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
    // println!("{thirteen:?}");

    // ── 5. 유닛 `()` ─────────────────────────────────────────────────────
    // 값이 `()` 하나뿐인 타입입니다. 반환 타입을 적지 않은 함수는 `()`를 돌려줍니다.
    // "성공했지만 돌려줄 값은 없음"을 `Result<(), E>`로 나타낼 때 자주 봅니다.
    // 유닛도 `Debug`, `PartialEq`, `Default` 등을 구현하므로 `Ok(())`를 그대로 출력할 수 있습니다.
    println!("{:?} {:?}", check_name("ferris"), check_name("")); // 출력: Ok(()) Err("이름이 비어 있음")
}

fn check_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        Err(String::from("이름이 비어 있음"))
    } else {
        Ok(())
    }
}
