//! 지침 1: 시그니처가 이미 말하는 것은 되풀이하지 않고, 코드가 말하지 못하는 것을 씁니다.
//!
//! 실행: cargo run -p er-ch27-document-public-interfaces --example 01_what_to_write
//! 문서: cargo doc -p er-ch27-document-public-interfaces --example 01_what_to_write --no-deps
//!
//! `v1::split`과 `v2::split`은 본문이 한 글자도 다르지 않고 문서 주석만 다릅니다. `main`은 두
//! 문서가 주장하는 문장을 실행해서 확인합니다. 문서의 문장이 "확인할 수 있는 약속"으로 쓰여
//! 있으면 이렇게 검사할 수 있고, 시그니처를 되풀이한 문장은 검사할 것이 없습니다.
//!
//! 예제 바이너리라 라이브러리처럼 공개할 함수를 `pub mod v1`·`pub mod v2`에 둡니다. `pub`이어야
//! `cargo doc`이 두 모듈의 문서를 만들어 주므로, 렌더링된 결과를 나란히 읽어 볼 수 있습니다.

/// 비권장: 시그니처를 되풀이하고, 부르는 쪽을 설명하고, 리팩터링 뒤 낡은 문서.
pub mod v1 {
    /// 금액을 나눕니다.
    ///
    /// # Arguments
    ///
    /// * `total` - 총액 (`u64`)
    /// * `people` - 사람 수 (`u32`)
    ///
    /// # Returns
    ///
    /// 각자 낼 금액을 담은 `Vec<u64>`
    ///
    /// 나누어떨어지지 않는 나머지는 버립니다.
    ///
    /// 정산 화면에서 "나누기" 버튼을 누르면 이 함수를 부릅니다.
    pub fn split(total: u64, people: usize) -> Vec<u64> {
        let n = people as u64;
        let base = total / n;
        let extra = total % n;
        (0..n)
            .map(|i| if i < extra { base + 1 } else { base })
            .collect()
    }
}

/// 권장: 코드가 말하지 못하는 약속(합계, 나머지를 누가 내는지, 차이의 상한)과 그 이유를 적은 문서.
pub mod v2 {
    /// `total`원을 `people`명이 낼 몫으로 나눕니다.
    ///
    /// 나누어떨어지지 않으면 앞사람부터 1원씩 더 냅니다. 그래서 몫을 모두 더하면 언제나
    /// `total`이고, 가장 많이 내는 사람과 가장 적게 내는 사람의 차이는 1원을 넘지 않습니다.
    /// 결과의 순서가 곧 사람의 순서이므로, 1원을 더 낼 사람을 정하려면 부르기 전에 순서를
    /// 정해 둡니다.
    ///
    /// 금액을 `f64`로 나눈 뒤 반올림하지 않는 이유는, 그렇게 하면 몫의 합이 `total`과
    /// 어긋날 수 있기 때문입니다(100원을 3명이 나누면 33원씩, 합계 99원).
    ///
    /// # Panics
    ///
    /// `people`이 0이면 패닉합니다.
    pub fn split(total: u64, people: usize) -> Vec<u64> {
        let n = people as u64;
        let base = total / n;
        let extra = total % n;
        (0..n)
            .map(|i| if i < extra { base + 1 } else { base })
            .collect()
    }
}

/// `f64`로 나눈 뒤 반올림하는 방식. `v2` 문서가 "왜 이렇게 하지 않는지" 설명한 방식입니다.
fn split_by_rounding(total: u64, people: usize) -> Vec<u64> {
    let share = (total as f64 / people as f64).round() as u64;
    vec![share; people]
}

fn verdict(ok: bool) -> &'static str {
    if ok { "맞음" } else { "틀림" }
}

fn main() {
    let shares = v1::split(10_000, 3);
    let sum: u64 = shares.iter().sum();
    println!("split(10000, 3) = {shares:?}, 합계 {sum}");
    // 출력: split(10000, 3) = [3334, 3333, 3333], 합계 10000

    println!("1. 비권장 v1 문서의 주장");
    // 출력: 1. 비권장 v1 문서의 주장
    // "나머지는 버립니다"가 참이라면 합계가 10000보다 작아야 합니다.
    println!("  \"나머지는 버립니다\" → {}", verdict(sum < 10_000));
    // 출력:   "나머지는 버립니다" → 틀림
    // `people`의 타입은 실행으로 확인할 것도 없이 rustdoc이 문서 바로 위에 `usize`로 보여 줍니다.
    println!("  \"people은 u32\" → 틀림 (시그니처는 usize)");
    // 출력:   "people은 u32" → 틀림 (시그니처는 usize)
    println!("  나머지 문장(총액, 사람 수, Vec<u64>, 버튼)은 확인할 약속이 아님");
    // 출력:   나머지 문장(총액, 사람 수, Vec<u64>, 버튼)은 확인할 약속이 아님

    println!("2. 권장 v2 문서의 주장 (total 0..=200, people 1..=9의 모든 조합)");
    // 출력: 2. 권장 v2 문서의 주장 (total 0..=200, people 1..=9의 모든 조합)
    let mut cases = 0;
    let mut sum_ok = true;
    let mut front_first = true;
    let mut gap_ok = true;
    for total in 0..=200 {
        for people in 1..=9 {
            let s = v2::split(total, people);
            cases += 1;
            sum_ok &= s.iter().sum::<u64>() == total;
            // 앞사람부터 더 낸다 = 몫이 앞에서 뒤로 줄어들기만 한다.
            front_first &= s.windows(2).all(|w| w[0] >= w[1]);
            let (max, min) = (s.iter().max().unwrap(), s.iter().min().unwrap());
            gap_ok &= max - min <= 1;
        }
    }
    println!("  {cases}가지 입력으로 확인");
    // 출력:   1809가지 입력으로 확인
    println!("  \"몫의 합은 언제나 total\" → {}", verdict(sum_ok));
    // 출력:   "몫의 합은 언제나 total" → 맞음
    println!("  \"앞사람부터 1원씩 더 낸다\" → {}", verdict(front_first));
    // 출력:   "앞사람부터 1원씩 더 낸다" → 맞음
    println!("  \"차이는 1원을 넘지 않는다\" → {}", verdict(gap_ok));
    // 출력:   "차이는 1원을 넘지 않는다" → 맞음

    println!("3. v2 문서가 \"하지 않는 이유\"로 든 방식");
    // 출력: 3. v2 문서가 "하지 않는 이유"로 든 방식
    let rounded = split_by_rounding(100, 3);
    let rounded_sum: u64 = rounded.iter().sum();
    println!("  반올림 방식 split(100, 3) = {rounded:?}, 합계 {rounded_sum}");
    // 출력:   반올림 방식 split(100, 3) = [33, 33, 33], 합계 99
    println!("  v2::split(100, 3) = {:?}", v2::split(100, 3));
    // 출력:   v2::split(100, 3) = [34, 33, 33]
}
