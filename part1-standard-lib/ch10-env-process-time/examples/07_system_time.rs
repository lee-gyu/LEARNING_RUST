// SystemTime: 운영체제의 시스템 시계(벽시계)가 가리키는 시각입니다.
//
//   1) SystemTime::now()와 기준 시각 UNIX_EPOCH(1970-01-01 00:00:00 UTC)로 "유닉스 시간"을 얻습니다.
//   2) UNIX_EPOCH + Duration으로 정해진 시각을 만듭니다.
//   3) duration_since와 elapsed는 Result를 돌려줍니다. 시스템 시계는 뒤로 갈 수 있기 때문입니다.
//   4) 표준 라이브러리에는 달력(연·월·일)과 시간대가 없습니다. 날짜는 직접 계산하거나 크레이트를 씁니다.
//
// 실행: cargo run -p p1-ch10-env-process-time --example 07_system_time
//
// 현재 시각은 실행할 때마다 다르므로 비교 결과로만 출력합니다. 나머지는 정해진 시각이라 늘 같습니다.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

// 1970-01-01부터 센 날 수를 (연, 월, 일)로 바꿉니다(그레고리력, UTC).
// 400년마다 날짜 배치가 반복된다는 점을 이용한 공개 알고리즘(Howard Hinnant의 civil_from_days)을
// 1970년 이후만 다루도록 부호 없는 정수로 옮겼습니다.
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468; // 0000-03-01부터 센 날 수로 옮깁니다(윤일이 해의 끝에 오도록 3월을 해의 시작으로 봄)
    let era = z / 146_097; // 400년(146097일) 단위
    let doe = z - era * 146_097; // 400년 안에서 몇 번째 날 [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // 400년 안에서 몇 번째 해 [0, 399]
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // 3월 1일부터 몇 번째 날 [0, 365]
    let mp = (5 * doy + 2) / 153; // 3월을 0으로 센 달 [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = era * 400 + yoe + u64::from(month <= 2);
    (year, month, day)
}

// SystemTime을 "YYYY-MM-DD hh:mm:ss UTC" 문자열로 바꿉니다. 1970년 이전이면 None입니다.
fn format_utc(t: SystemTime) -> Option<String> {
    let secs = t.duration_since(UNIX_EPOCH).ok()?.as_secs();
    let (year, month, day) = civil_from_days(secs / 86_400);
    let rest = secs % 86_400;
    Some(format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        rest / 3600,
        rest / 60 % 60,
        rest % 60
    ))
}

fn main() {
    println!("1. 지금의 유닉스 시간");
    let now = SystemTime::now();
    match now.duration_since(UNIX_EPOCH) {
        // 값은 실행할 때마다 다르므로 2023-11-14(1_700_000_000초) 이후인지만 봅니다.
        Ok(since) => println!(
            "    1_700_000_000초 이후: {}",
            since.as_secs() > 1_700_000_000
        ), // 출력:     1_700_000_000초 이후: true
        Err(e) => println!("    시스템 시계가 1970년 이전을 가리킵니다: {e}"),
    }

    println!("2. 정해진 시각 만들기");
    let billion = UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    println!("    UNIX_EPOCH  → {:?}", format_utc(UNIX_EPOCH)); // 출력:     UNIX_EPOCH  → Some("1970-01-01 00:00:00 UTC")
    println!("    10억 초     → {:?}", format_utc(billion)); // 출력:     10억 초     → Some("2001-09-09 01:46:40 UTC")
    // 부호 있는 32비트 정수로 초를 세던 시스템이 넘치는 시각입니다("2038년 문제").
    let y2038 = UNIX_EPOCH + Duration::from_secs(i32::MAX as u64);
    println!("    i32::MAX 초 → {:?}", format_utc(y2038)); // 출력:     i32::MAX 초 → Some("2038-01-19 03:14:07 UTC")
    // 디버그 형식은 플랫폼의 내부 표현을 그대로 보여 줍니다(유닉스 계열의 모양).
    println!("    {billion:?}"); // 출력:     SystemTime { tv_sec: 1000000000, tv_nsec: 0 }

    println!("3. 두 시각의 차이는 Result입니다");
    println!(
        "    billion.duration_since(UNIX_EPOCH) = {:?}",
        billion.duration_since(UNIX_EPOCH)
    ); // 출력:     billion.duration_since(UNIX_EPOCH) = Ok(1000000000s)
    // 순서를 거꾸로 넣으면 Err입니다. 에러 안에 "얼마나 거꾸로였는지"가 들어 있습니다.
    match UNIX_EPOCH.duration_since(billion) {
        Ok(d) => println!("    {d:?}"),
        Err(e) => println!("    거꾸로: {e} (차이 {:?})", e.duration()), // 출력:     거꾸로: second time provided was later than self (차이 1000000000s)
    }
    // elapsed()는 now().duration_since(self)와 같습니다. 미래의 시각이면 Err입니다.
    let tomorrow = now + Duration::from_secs(86_400);
    println!(
        "    내일 시각의 elapsed() 성공: {}",
        tomorrow.elapsed().is_ok()
    ); // 출력:     내일 시각의 elapsed() 성공: false
    // 시스템 시계는 사용자가 바꾸거나 시간 동기화(NTP)로 조정될 수 있어서,
    // 나중에 잰 SystemTime이 먼저 잰 것보다 앞설 수도 있습니다. 그래서 정상 순서라도 Err를 처리해야 합니다.
    let a = SystemTime::now();
    let b = SystemTime::now();
    let gap = b.duration_since(a).unwrap_or(Duration::ZERO); // 거꾸로면 0으로 봅니다
    println!(
        "    연달아 잰 두 시각의 차이 1초 미만: {}",
        gap < Duration::from_secs(1)
    ); // 출력:     연달아 잰 두 시각의 차이 1초 미만: true

    println!("4. 지금 날짜 (UTC)");
    let today = format_utc(now).unwrap_or_default();
    // 날짜는 실행하는 날에 따라 다르므로 모양만 확인합니다.
    println!(
        "    \"YYYY-MM-DD hh:mm:ss UTC\" 모양: {}",
        today.len() == 23 && today.ends_with(" UTC")
    ); // 출력:     "YYYY-MM-DD hh:mm:ss UTC" 모양: true
}
