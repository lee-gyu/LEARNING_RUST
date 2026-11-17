// @ 바인딩과 배열·슬라이스 패턴을 보여 줍니다.
// - 이름 @ 패턴: 값이 패턴과 맞는지 검사하면서, 동시에 그 값을 이름에 묶습니다.
// - [a, b], [first, .., last], [head, rest @ ..]: 배열과 슬라이스를 원소 위치로 풉니다.
//
// 실행: cargo run -p p0-ch18-patterns --example 08_at_and_slices

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

// 범위 패턴만 쓰면 "13~18 중 하나"라는 것만 알고 실제 값은 모릅니다.
// age @ 13..=18처럼 쓰면 범위 검사와 바인딩을 한 번에 해서 갈래 안에서 값을 쓸 수 있습니다.
fn ticket(age: u32) -> String {
    match age {
        0..=6 => String::from("무료"), // 값이 필요 없으면 @ 없이 범위만 씁니다.
        child @ 7..=12 => format!("어린이 요금 ({child}세)"),
        teen @ 13..=18 => format!("청소년 요금 ({teen}세)"),
        senior @ 65.. => format!("경로 요금 ({senior}세)"),
        adult => format!("일반 요금 ({adult}세)"),
    }
}

// 슬라이스는 길이가 정해져 있지 않으므로 길이별로 갈래를 나눌 수 있습니다.
// [], [one], [first, second], [first, .., last]가 모든 길이(0, 1, 2, 3 이상)를 덮어 완전합니다.
fn describe(items: &[i32]) -> String {
    match items {
        [] => String::from("비어 있음"),
        [one] => format!("하나뿐: {one}"),
        [first, second] => format!("둘: {first}, {second}"),
        [first, .., last] => format!("처음 {first}, 마지막 {last} (모두 {}개)", items.len()),
    }
}

// [head, rest @ ..]는 첫 원소와 "나머지 슬라이스"로 나눕니다. 재귀 함수와 잘 어울립니다.
fn sum(items: &[i32]) -> i32 {
    match items {
        [] => 0,
        [head, rest @ ..] => head + sum(rest),
    }
}

fn main() {
    println!("1. @ 바인딩: 검사하면서 묶기");
    for age in [5, 10, 16, 40, 70] {
        println!("    {age}세: {}", ticket(age));
    }
    // 출력:
    //     5세: 무료
    //     10세: 어린이 요금 (10세)
    //     16세: 청소년 요금 (16세)
    //     40세: 일반 요금 (40세)
    //     70세: 경로 요금 (70세)

    println!("2. @와 |: 괄호로 묶어야 | 전체가 한 이름에 묶입니다");
    for n in [1, 4, 5, 9] {
        match n {
            // n @ 1 | 4 | 9 라고 쓰면 (n @ 1) | 4 | 9 로 읽혀서 E0408이 납니다.
            square @ (1 | 4 | 9) => println!("    {square}: 한 자리 제곱수"),
            other => println!("    {other}: 제곱수가 아님"),
        }
    }
    // 출력:
    //     1: 한 자리 제곱수
    //     4: 한 자리 제곱수
    //     5: 제곱수가 아님
    //     9: 한 자리 제곱수

    println!("3. @로 값 전체를 묶으면서 안쪽 패턴도 검사하기");
    for p in [Point { x: 0, y: 7 }, Point { x: 2, y: 3 }] {
        match p {
            // whole에는 Point 전체가 옮겨 가고, y에는 필드 값이 복사됩니다(i32는 Copy).
            // 안쪽 바인딩이 String처럼 Copy가 아닌 값이면 같은 값을 두 번 가져가는 셈이라 E0382가 납니다.
            whole @ Point { x: 0, y } => println!("    {whole:?}: y축 위, 높이 {y}"),
            Point { x, y } => println!("    ({x}, {y}): y축 밖"),
        }
    }
    // 출력:
    //     Point { x: 0, y: 7 }: y축 위, 높이 7
    //     (2, 3): y축 밖

    println!("4. 배열 패턴: 길이가 정해져 있으므로 반박 불가능합니다");
    let [r, g, b] = [255, 128, 0];
    println!("    r = {r}, g = {g}, b = {b}");
    // 출력:
    //     r = 255, g = 128, b = 0

    println!("5. 슬라이스 패턴: 길이별로 나누기");
    let data = [10, 20, 30, 40];
    for items in [&data[..0], &data[..1], &data[..2], &data[..]] {
        println!("    {items:?} → {}", describe(items));
    }
    // 출력:
    //     [] → 비어 있음
    //     [10] → 하나뿐: 10
    //     [10, 20] → 둘: 10, 20
    //     [10, 20, 30, 40] → 처음 10, 마지막 40 (모두 4개)

    println!("6. rest @ ..: 나머지를 슬라이스로 묶기");
    println!("    sum({data:?}) = {}", sum(&data));
    // 출력:
    //     sum([10, 20, 30, 40]) = 100

    println!("7. 문자열 슬라이스 패턴으로 명령어 해석하기");
    for line in ["add 우유", "remove 3", "list", "add", "move a b c", ""] {
        let words: Vec<&str> = line.split_whitespace().collect();
        // 리터럴과 바인딩, @ ..를 섞어 "명령 이름과 인자 개수"를 한 번에 검사합니다.
        let result = match words.as_slice() {
            ["add", item] => format!("'{item}' 추가"),
            ["remove", index] => format!("{index}번 삭제"),
            ["list"] => String::from("목록 보기"),
            ["add" | "remove", ..] => String::from("인자 개수가 틀렸습니다"),
            [command, args @ ..] => format!("모르는 명령 {command}, 인자 {args:?}"),
            [] => String::from("빈 줄"),
        };
        println!("    {line:?} → {result}");
    }
    // 출력:
    //     "add 우유" → '우유' 추가
    //     "remove 3" → 3번 삭제
    //     "list" → 목록 보기
    //     "add" → 인자 개수가 틀렸습니다
    //     "move a b c" → 모르는 명령 move, 인자 ["a", "b", "c"]
    //     "" → 빈 줄
}
