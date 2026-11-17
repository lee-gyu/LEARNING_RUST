// 패턴에서 값을 무시하는 네 가지 방법: _, 중첩된 _, _로 시작하는 이름, ..
// 특히 _(묶지 않음)와 _x(묶음)는 비슷해 보이지만 소유권과 drop 시점이 다릅니다.
//
// 실행: cargo run -p p0-ch18-patterns --example 06_ignoring

// drop될 때 메시지를 출력하는 구조체입니다(ch14의 Drop). drop 시점을 눈으로 보려고 씁니다.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.0);
    }
}

struct Point3 {
    x: i32,
    y: i32,
    z: i32,
}

// 첫 번째 매개변수를 _로 무시합니다. 함수의 모양(시그니처)은 정해져 있는데
// 그 값은 필요 없을 때 씁니다. 예를 들어 다른 코드가 "클릭 위치, 버튼 이름"을 넘겨주기로
// 약속했지만, 이 함수는 버튼 이름만 쓰는 경우입니다.
fn on_click(_: (i32, i32), label: &str) {
    println!("    '{label}' 버튼이 눌렸습니다");
}

fn main() {
    println!("1. _: 값 전체를 무시합니다");
    on_click((120, 45), "확인");
    // 출력:
    //     '확인' 버튼이 눌렸습니다

    println!("2. 중첩된 _: 값의 일부만 무시합니다");
    // 설정값이 이미 있으면 덮어쓰지 않고, 둘 중 하나라도 None이면 새 값으로 바꿉니다.
    // 안의 숫자는 필요 없고 "Some인가"만 보면 되므로 Some(_)로 씁니다.
    let mut setting = Some(5);
    let new_setting = Some(10);
    match (setting, new_setting) {
        (Some(_), Some(_)) => println!("    이미 설정된 값을 덮어쓸 수 없습니다"),
        _ => setting = new_setting,
    }
    println!("    setting = {setting:?}");
    let numbers = (2, 4, 8, 16, 32);
    let (first, _, third, _, fifth) = numbers; // 둘째와 넷째만 건너뜁니다.
    println!("    {first}, {third}, {fifth}");
    // 출력:
    //     이미 설정된 값을 덮어쓸 수 없습니다
    //     setting = Some(5)
    //     2, 8, 32

    println!("3. _로 시작하는 이름: 묶기는 하지만 '쓰지 않는 변수' 경고를 끕니다");
    let _draft = 5; // 그냥 draft라고 쓰면 warning: unused variable: `draft`가 납니다.
    println!("    경고 없이 컴파일됐습니다");
    // 출력:
    //     경고 없이 컴파일됐습니다

    println!("4. _와 _x의 차이 (1): _는 값을 옮기지 않습니다");
    let pair = (String::from("안녕"), 3);
    // _는 아무것도 묶지 않으므로 String이 이동하지 않습니다. count에는 i32가 복사됩니다.
    // (_greeting, count)라고 쓰면 String이 _greeting으로 이동해 아래 줄에서 E0382가 납니다
    // (compile_fail/e0382_underscore_name_moves.rs).
    let (_, count) = pair;
    println!("    count = {count}, pair는 그대로: {pair:?}");
    // 출력:
    //     count = 3, pair는 그대로: ("안녕", 3)

    println!("5. _와 _x의 차이 (2): drop 시점");
    {
        // _에는 값이 묶이지 않으므로, 새로 만든 값은 이 문장이 끝나자마자 drop됩니다.
        let _ = Noisy("_에 넘긴 값");
        // _guard에는 값이 묶이므로 블록이 끝날 때까지 살아 있습니다.
        // ch15의 MutexGuard처럼 "살아 있는 동안 효과가 있는 값"을 _에 넘기면 곧바로 사라지니 주의합니다.
        // (let _ = m.lock().unwrap();은 컴파일러가 let_underscore_lock 에러로 막아 주지만,
        //  직접 만든 타입은 이 Noisy처럼 아무 경고 없이 바로 drop됩니다.)
        let _guard = Noisy("_guard에 묶은 값");
        println!("    블록의 마지막 줄");
    }
    // 출력:
    //     [drop] _에 넘긴 값
    //     블록의 마지막 줄
    //     [drop] _guard에 묶은 값

    println!("6. ..: 나머지를 한꺼번에 무시합니다");
    let origin = Point3 { x: 0, y: 0, z: 0 };
    let p = Point3 { x: 1, y: 2, z: 3 };
    // 구조체: 필요한 필드만 적고 나머지는 ..로 넘깁니다. 필드가 많을수록 편합니다.
    let Point3 { x, .. } = p;
    println!("    x = {x}");
    // match에서도 씁니다. x가 0인지만 보고 나머지 필드는 보지 않습니다.
    for point in [origin, p] {
        match point {
            Point3 { x: 0, .. } => println!("    x가 0인 점"),
            Point3 { y, z, .. } => println!("    x가 0이 아닌 점 (y = {y}, z = {z})"),
        }
    }
    // 튜플: 처음과 마지막만 꺼내고 가운데는 몇 개든 건너뜁니다.
    let (first, .., last) = numbers;
    println!("    처음 {first}, 마지막 {last}");
    // ..는 한 패턴에 한 번만 쓸 수 있습니다. (.., second, ..)는 어느 것이 second인지 알 수 없으므로
    // 컴파일 에러입니다(compile_fail/double_rest_pattern.rs).
    // 출력:
    //     x = 1
    //     x가 0인 점
    //     x가 0이 아닌 점 (y = 2, z = 3)
    //     처음 2, 마지막 32
}
