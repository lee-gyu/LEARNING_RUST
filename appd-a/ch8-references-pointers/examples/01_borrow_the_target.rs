// 지침 1: 읽기만 하는 매개변수는 컨테이너가 아니라 "가리키는 대상"으로 받습니다.
//         `&Vec<T>` 대신 `&[T]`, `&String` 대신 `&str`, `&Box<T>` 대신 `&T`
//
// `&Vec<u32>`는 "어딘가에 있는 Vec 핸들"을 가리키는 포인터입니다. 그래서 부르는 쪽이 배열이나
// Vec의 일부를 가지고 있으면, 함수를 부르려고 임시 Vec을 새로 만들어(할당하고 복사해서) 넘겨야
// 합니다. `&[u32]`는 "어딘가에 연속으로 놓인 u32들"을 가리키므로 그런 비용이 없습니다.
//
// 실행: cargo run -p er-ch8-references-pointers --example 01_borrow_the_target

use std::ptr;

#[derive(Clone)]
struct Workout {
    kind: String,
    minutes: u32,
}

const CARDIO: [&str; 3] = ["달리기", "수영", "자전거"];

// ---------- 비권장: 컨테이너 타입으로 받습니다 ----------

#[expect(clippy::ptr_arg, reason = "지침 1의 비권장 형태를 보여 주는 코드")]
fn average_vec(days: &Vec<u32>) -> u32 {
    days.iter().sum::<u32>() / days.len() as u32
}

#[expect(clippy::ptr_arg, reason = "지침 1의 비권장 형태를 보여 주는 코드")]
fn is_cardio_string(kind: &String) -> bool {
    CARDIO.contains(&kind.as_str())
}

#[expect(clippy::borrowed_box, reason = "지침 1의 비권장 형태를 보여 주는 코드")]
fn summary_boxed(workout: &Box<Workout>) -> String {
    format!("{} {}분", workout.kind, workout.minutes)
}

// ---------- 권장: 가리키는 대상으로 받습니다 ----------
// 본문은 한 글자도 바뀌지 않았습니다. 바뀐 것은 매개변수 타입뿐입니다.

fn average(days: &[u32]) -> u32 {
    days.iter().sum::<u32>() / days.len() as u32
}

fn is_cardio(kind: &str) -> bool {
    CARDIO.contains(&kind)
}

fn summary(workout: &Workout) -> String {
    format!("{} {}분", workout.kind, workout.minutes)
}

// ---------- 지침을 따르지 않아도 되는 경우 ----------

/// 길이를 바꾸는(push) 함수는 `&mut Vec<u32>`가 맞습니다. `&mut [u32]`로는 원소를 늘릴 수 없습니다.
/// 본문이 `push`를 쓰므로 clippy의 ptr_arg도 경고하지 않습니다.
fn record(days: &mut Vec<u32>, steps: u32) {
    days.push(steps);
}

/// 원소를 제자리에서 고치기만 한다면 `&mut [u32]`로 충분합니다. 배열도 받을 수 있습니다.
fn cap_outliers(days: &mut [u32], max: u32) {
    for d in days.iter_mut() {
        *d = (*d).min(max);
    }
}

/// 용량처럼 Vec에만 있는 정보를 읽는 드문 경우에는 `&Vec<u32>`가 필요합니다.
/// 본문이 `capacity()`를 쓰므로 ptr_arg도 경고하지 않습니다.
fn fits_without_realloc(days: &Vec<u32>, more: usize) -> bool {
    days.len() + more <= days.capacity()
}

fn main() {
    // 부르는 쪽이 가진 데이터는 모양이 제각각입니다.
    let week: [u32; 7] = [6200, 8400, 7100, 9900, 5300, 12000, 7600]; // 배열
    let mut month: Vec<u32> = Vec::with_capacity(31); // 이번 달 기록 (용량은 31 이상)
    month.extend([4300, 6100, 8800, 7700, 9200, 6600, 5100, 10400, 8300, 7000]);
    let memo = "수영 40분"; // 문자열 리터럴. 운동 이름은 이 문자열의 일부
    let today = Workout {
        kind: String::from("달리기"),
        minutes: 30,
    }; // 스택에 있는 값

    println!("1. 비권장: 컨테이너 타입으로 받는 함수");
    // 함수에 맞추느라 만든 임시 값의 개수와 복사한 바이트 수를 셉니다.
    let mut temporaries = 0;
    let mut copied_bytes = 0;

    let week_vec = week.to_vec(); // 배열 → 임시 Vec
    temporaries += 1;
    copied_bytes += size_of_val(&week_vec[..]);
    let recent_vec = month[7..].to_vec(); // 최근 3일 → 임시 Vec
    temporaries += 1;
    copied_bytes += size_of_val(&recent_vec[..]);
    println!(
        "    평균 걸음: 이번 주 {}, 이번 달 {}, 최근 3일 {}",
        average_vec(&week_vec),
        average_vec(&month),
        average_vec(&recent_vec)
    );
    // 출력:     평균 걸음: 이번 주 8071, 이번 달 7350, 최근 3일 8566

    let (kind, _) = memo.split_once(' ').unwrap_or((memo, ""));
    let kind_string = kind.to_string(); // 리터럴의 일부 → 임시 String
    temporaries += 1;
    copied_bytes += kind_string.len();
    let today_boxed = Box::new(today.clone()); // 스택의 값 → 임시 Box (안의 String도 복제)
    temporaries += 1;
    copied_bytes += size_of::<Workout>() + today.kind.len();
    println!(
        "    {kind} 유산소인가: {}, 오늘 운동: {}",
        is_cardio_string(&kind_string),
        summary_boxed(&today_boxed)
    );
    // 출력:     수영 유산소인가: true, 오늘 운동: 달리기 30분
    println!("    함수에 맞추느라 만든 임시 값 {temporaries}개, 복사한 바이트 {copied_bytes}");
    // 출력:     함수에 맞추느라 만든 임시 값 4개, 복사한 바이트 87

    println!("2. 권장: 가리키는 대상으로 받는 함수");
    // 배열, Vec, Vec의 일부가 모두 &[u32]로 들어갑니다.
    // &week는 &[u32; 7]인데, 크기가 정해진 배열의 참조는 슬라이스 참조로 강제 변환됩니다.
    println!(
        "    평균 걸음: 이번 주 {}, 이번 달 {}, 최근 3일 {}",
        average(&week),
        average(&month),
        average(&month[7..])
    );
    // 출력:     평균 걸음: 이번 주 8071, 이번 달 7350, 최근 3일 8566
    // 리터럴의 일부도, String도 &str로 들어갑니다(&String은 역참조 강제 변환으로 &str이 됨).
    let owned_kind = String::from("요가");
    println!(
        "    {kind} 유산소인가: {}, {owned_kind} 유산소인가: {}",
        is_cardio(kind),
        is_cardio(&owned_kind)
    );
    // 출력:     수영 유산소인가: true, 요가 유산소인가: false
    // 스택의 값, Vec의 원소, Box 안의 값이 모두 &Workout으로 들어갑니다.
    let log = [today.clone()];
    let boxed = Box::new(Workout {
        kind: String::from("자전거"),
        minutes: 50,
    });
    println!(
        "    오늘 운동: {}, 기록의 첫 운동: {}, 상자 안의 운동: {}",
        summary(&today),
        summary(&log[0]),
        summary(&boxed)
    );
    // 출력:     오늘 운동: 달리기 30분, 기록의 첫 운동: 달리기 30분, 상자 안의 운동: 자전거 50분
    println!("    함수에 맞추느라 만든 임시 값 0개");
    // 출력:     함수에 맞추느라 만든 임시 값 0개

    println!("3. &Vec<u32>는 핸들을, &[u32]는 원소를 가리킵니다 (64비트 기준)");
    // &Vec<u32>는 얇은 포인터(8바이트)지만 Vec 핸들을 가리키므로, 원소에 닿으려면 한 번 더 따라가야 합니다.
    // &[u32]는 넓은 포인터(16바이트: 주소 + 길이)이고 원소를 바로 가리킵니다.
    let by_vec: &Vec<u32> = &month;
    let by_slice: &[u32] = &month;
    println!(
        "    size_of::<&Vec<u32>>() = {}, size_of::<&[u32]>() = {}",
        size_of::<&Vec<u32>>(),
        size_of::<&[u32]>()
    );
    // 출력:     size_of::<&Vec<u32>>() = 8, size_of::<&[u32]>() = 16
    println!(
        "    &Vec이 가리키는 곳이 첫 원소인가: {}",
        ptr::addr_eq(by_vec, month.as_ptr())
    );
    // 출력:     &Vec이 가리키는 곳이 첫 원소인가: false
    println!(
        "    &[u32]가 가리키는 곳이 첫 원소인가: {}",
        ptr::addr_eq(by_slice, month.as_ptr())
    );
    // 출력:     &[u32]가 가리키는 곳이 첫 원소인가: true

    println!("4. 컨테이너를 받아야 하는 경우");
    // 원소를 고치기만 하면 &mut [u32]: 배열과 Vec을 모두 받습니다.
    let mut week_copy = week;
    cap_outliers(&mut week_copy, 10000);
    cap_outliers(&mut month, 10000);
    println!(
        "    10000 넘는 기록 자르기: 이번 주 최댓값 {:?}, 이번 달 최댓값 {:?}",
        week_copy.iter().max(),
        month.iter().max()
    );
    // 출력:     10000 넘는 기록 자르기: 이번 주 최댓값 Some(10000), 이번 달 최댓값 Some(10000)
    // 길이를 바꾸면 &mut Vec<u32>.
    record(&mut month, 9100);
    println!("    오늘 기록을 추가한 뒤 기록 수: {}", month.len());
    // 출력:     오늘 기록을 추가한 뒤 기록 수: 11
    // 용량을 읽으면 &Vec<u32>. with_capacity(31)은 용량이 31 이상임을 보장하므로 결과는 true입니다.
    println!(
        "    5일치를 더 넣어도 재할당이 없는가: {}",
        fits_without_realloc(&month, 5)
    );
    // 출력:     5일치를 더 넣어도 재할당이 없는가: true
}
