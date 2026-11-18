// Box<dyn Error>: "Error를 구현한 어떤 에러든" 담는 트레이트 객체입니다(Part 0 ch17).
//
//   1) &str, String, 표준 에러, 직접 만든 에러가 모두 From을 통해 Box<dyn Error>로 바뀝니다.
//   2) 안에 든 구체 타입은 is / downcast_ref / downcast로 되찾습니다(다운캐스트).
//      컴파일러가 종류를 다 따져 주지 않으므로 "그 밖의 경우" 갈래가 늘 필요합니다.
//   3) 에러를 스레드 밖으로 돌려보내려면 Box<dyn Error + Send + Sync>를 씁니다.
//
// 실행: cargo run -p p1-ch8-errors-panic --example 04_box_dyn_error

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::thread;

const LIMIT: u32 = 100;

#[derive(Debug)]
struct QuotaExceeded {
    used: u32,
    limit: u32,
}

impl fmt::Display for QuotaExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "할당량을 넘었습니다 ({}/{})", self.used, self.limit)
    }
}

impl Error for QuotaExceeded {}

// ── 1. 무엇이든 Box<dyn Error>로 ────────────────────────────────────
// "이름=사용량" 한 줄을 읽습니다. 실패의 출처가 네 가지이고 타입이 모두 다르지만
// 표준 라이브러리의 From 구현 덕분에 모두 같은 Box<dyn Error>가 됩니다.
//   impl From<&str>   for Box<dyn Error>
//   impl From<String> for Box<dyn Error>
//   impl<E: Error> From<E> for Box<dyn Error>
fn parse_usage(line: &str) -> Result<u32, Box<dyn Error>> {
    let (name, value) = line.split_once('=').ok_or("'='가 없습니다")?; // &str
    if name.trim().is_empty() {
        return Err(format!("이름이 비었습니다: {line:?}").into()); // String
    }
    let used: u32 = value.trim().parse()?; // ParseIntError
    if used > LIMIT {
        return Err(QuotaExceeded { used, limit: LIMIT }.into()); // 직접 만든 에러
    }
    Ok(used)
}

// ── 2. 다운캐스트로 구체 타입 되찾기 ────────────────────────────────
// downcast_ref::<T>()는 "안에 든 것이 T이면 Some(&T), 아니면 None"입니다.
// 'static이 붙은 이유: 다운캐스트는 실행 중에 타입을 비교하는데, 빌린 참조를 품은 타입은
// 라이프타임까지 같은지 실행 중에 확인할 수 없기 때문입니다(Any와 다운캐스트 일반은 ch14-ffi-any-misc).
fn react(err: &(dyn Error + 'static)) -> String {
    if let Some(q) = err.downcast_ref::<QuotaExceeded>() {
        // 구체 타입을 되찾았으므로 필드를 읽을 수 있습니다.
        format!("할당량 초과 → {}만큼 줄이세요", q.used - q.limit)
    } else if let Some(p) = err.downcast_ref::<ParseIntError>() {
        format!("숫자 형식 오류 → 종류: {:?}", p.kind())
    } else {
        // &str, String에서 만든 에러는 표준 라이브러리의 비공개 타입이라 이름으로 꺼낼 수 없습니다.
        // 열거형의 match와 달리, 무엇이 들어올지 컴파일러가 모르므로 이 갈래를 빠뜨릴 수 없습니다.
        format!("그 밖의 에러 → {err}")
    }
}

// ── 3. 스레드 경계를 넘는 에러 ─────────────────────────────────────
// 본문은 parse_usage와 같고 반환 타입만 다릅니다. thread::spawn의 클로저가 돌려주는 값은
// 다른 스레드로 옮겨지므로 Send여야 하고, Box<dyn Error>에는 그 약속이 없습니다
// (compile_fail/e0277_box_dyn_error_not_send.rs). + Send + Sync를 붙이면 "다른 스레드로
// 보내도 되고, 여러 스레드가 함께 읽어도 되는 에러만 담는다"는 약속이 생깁니다.
fn parse_usage_sendable(line: &str) -> Result<u32, Box<dyn Error + Send + Sync>> {
    let (name, value) = line.split_once('=').ok_or("'='가 없습니다")?;
    if name.trim().is_empty() {
        return Err(format!("이름이 비었습니다: {line:?}").into());
    }
    let used: u32 = value.trim().parse()?;
    if used > LIMIT {
        return Err(QuotaExceeded { used, limit: LIMIT }.into());
    }
    Ok(used)
}

fn main() {
    let lines = ["kim=42", "lee", "=7", "park=12a", "choi=250"];

    println!("1. 네 가지 출처의 에러가 모두 Box<dyn Error>가 됩니다");
    for line in lines {
        match parse_usage(line) {
            Ok(n) => println!("    {line:<9} → Ok({n})"),
            Err(e) => println!("    {line:<9} → {e}  [{e:?}]"),
        }
    }
    // 출력:
    //     kim=42    → Ok(42)
    //     lee       → '='가 없습니다  ["'='가 없습니다"]
    //     =7        → 이름이 비었습니다: "=7"  ["이름이 비었습니다: \"=7\""]
    //     park=12a  → invalid digit found in string  [ParseIntError { kind: InvalidDigit }]
    //     choi=250  → 할당량을 넘었습니다 (250/100)  [QuotaExceeded { used: 250, limit: 100 }]
    // 디버그 형식은 안에 든 실제 타입의 Debug 구현을 따릅니다. 문자열에서 만든 에러는 따옴표로 감싼 문자열로 나옵니다.

    println!("2. downcast_ref와 is로 안에 든 타입을 확인합니다");
    for line in lines {
        if let Err(e) = parse_usage(line) {
            // e는 Box<dyn Error>, e.as_ref()는 &dyn Error입니다.
            println!(
                "    {line:<9} is::<ParseIntError>() = {:<5} → {}",
                e.is::<ParseIntError>(),
                react(e.as_ref())
            );
        }
    }
    // 출력:
    //     lee       is::<ParseIntError>() = false → 그 밖의 에러 → '='가 없습니다
    //     =7        is::<ParseIntError>() = false → 그 밖의 에러 → 이름이 비었습니다: "=7"
    //     park=12a  is::<ParseIntError>() = true  → 숫자 형식 오류 → 종류: InvalidDigit
    //     choi=250  is::<ParseIntError>() = false → 할당량 초과 → 150만큼 줄이세요

    // downcast::<T>()는 Box를 소비하고 Result<Box<T>, Box<dyn Error>>를 돌려줍니다.
    // 맞으면 구체 타입의 Box를 소유권째 받고, 틀리면 원래의 Box를 Err로 돌려받아 계속 쓸 수 있습니다.
    let err = parse_usage("choi=250").unwrap_err();
    match err.downcast::<QuotaExceeded>() {
        Ok(q) => {
            let q: QuotaExceeded = *q; // Box에서 꺼내 값으로 가집니다.
            println!("    downcast 성공: used = {}, limit = {}", q.used, q.limit);
        }
        Err(other) => println!("    다른 에러: {other}"),
    }
    let err = parse_usage("lee").unwrap_err();
    match err.downcast::<QuotaExceeded>() {
        Ok(q) => println!("    downcast 성공: {q}"),
        Err(other) => println!("    downcast 실패, 원래 에러를 돌려받음: {other}"),
    }
    // 출력:
    //     downcast 성공: used = 250, limit = 100
    //     downcast 실패, 원래 에러를 돌려받음: '='가 없습니다

    println!("3. Box<dyn Error + Send + Sync>는 스레드 밖으로 돌려보낼 수 있습니다");
    // 줄마다 스레드를 하나씩 만들어 파싱하고, 결과(에러 포함)를 join으로 돌려받습니다.
    // 출력은 main 스레드에서 lines 순서대로 하므로 실행할 때마다 같습니다.
    let handles: Vec<_> = lines
        .into_iter()
        .map(|line| thread::spawn(move || (line, parse_usage_sendable(line))))
        .collect();
    for handle in handles {
        let (line, result) = handle.join().expect("작업 스레드는 패닉하지 않아야 합니다");
        match result {
            Ok(n) => println!("    {line:<9} → Ok({n})"),
            Err(e) => println!("    {line:<9} → {}", react(e.as_ref())),
        }
    }
    // 출력:
    //     kim=42    → Ok(42)
    //     lee       → 그 밖의 에러 → '='가 없습니다
    //     =7        → 그 밖의 에러 → 이름이 비었습니다: "=7"
    //     park=12a  → 숫자 형식 오류 → 종류: InvalidDigit
    //     choi=250  → 할당량 초과 → 150만큼 줄이세요

    // Send + Sync가 붙은 Box는 붙지 않은 Box로 그냥 대입할 수 있습니다(약속을 덜어 내는 쪽).
    // 다만 ?는 이 변환을 해 주지 않으므로 함수 경계에서는 map_err(|e| e as Box<dyn Error>)가 필요합니다(README 4.6절).
    let sendable = parse_usage_sendable("lee").unwrap_err();
    let plain: Box<dyn Error> = sendable;
    println!("    Send + Sync를 덜어 낸 Box: {plain}");
    // 출력: Send + Sync를 덜어 낸 Box: '='가 없습니다
}
