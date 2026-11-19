//! 예제 02: 누적 변수를 소비 메서드로 바꾸는 기준
//!
//! 실행: cargo run -q -p er-ch9-iterator-transforms --example 02_accumulator_to_consumer
//!
//! 반복문 앞에서 `let mut` 변수를 만들고 본문에서 고치는 코드는 대부분
//! "무엇을 만드는가"에 따라 정해진 소비 메서드로 바꿀 수 있습니다.
//! 받은편지함 하나를 두고 다섯 가지 누적을 반복문판과 체인판으로 비교합니다.

/// 받은편지함의 메일 한 통.
struct Mail {
    subject: &'static str,
    size_kb: u32,
    unread: bool,
}

// ---- 1. 새 Vec에 push → filter/map + collect ------------------------------

fn unread_subjects_loop(inbox: &[Mail]) -> Vec<&'static str> {
    let mut subjects = Vec::new();
    for mail in inbox {
        if mail.unread {
            subjects.push(mail.subject);
        }
    }
    subjects
}

fn unread_subjects_chain(inbox: &[Mail]) -> Vec<&'static str> {
    inbox
        .iter()
        .filter(|m| m.unread)
        .map(|m| m.subject)
        .collect()
}

// ---- 2. 카운터와 합계 → count, sum ----------------------------------------

fn unread_count_loop(inbox: &[Mail]) -> usize {
    let mut count = 0;
    for mail in inbox {
        if mail.unread {
            count += 1;
        }
    }
    count
}

fn unread_count_chain(inbox: &[Mail]) -> usize {
    inbox.iter().filter(|m| m.unread).count()
}

// fold로 덧셈을 직접 쓰면 "무엇을 만드는가"를 클로저를 읽어야 압니다.
#[expect(
    clippy::unnecessary_fold,
    reason = "sum이 있는데 fold로 더하는 비권장 코드를 보여 줍니다"
)]
fn total_kb_fold(inbox: &[Mail]) -> u32 {
    inbox.iter().map(|m| m.size_kb).fold(0, |acc, kb| acc + kb)
}

fn total_kb_sum(inbox: &[Mail]) -> u32 {
    inbox.iter().map(|m| m.size_kb).sum()
}

// ---- 3. 찾으면 return, 순번 변수 → find, enumerate ------------------------

#[expect(
    clippy::manual_find,
    reason = "찾으면 return하는 반복문(비권장 코드)을 보여 줍니다"
)]
fn first_larger_loop(inbox: &[Mail], limit_kb: u32) -> Option<&Mail> {
    for mail in inbox {
        if mail.size_kb > limit_kb {
            return Some(mail);
        }
    }
    None
}

fn first_larger_chain(inbox: &[Mail], limit_kb: u32) -> Option<&Mail> {
    inbox.iter().find(|m| m.size_kb > limit_kb)
}

#[expect(
    clippy::explicit_counter_loop,
    reason = "순번을 손으로 세는 비권장 코드를 보여 줍니다"
)]
fn numbered_loop(inbox: &[Mail]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut no = 1;
    for mail in inbox {
        lines.push(format!("{no}. {}", mail.subject));
        no += 1;
    }
    lines
}

fn numbered_chain(inbox: &[Mail]) -> Vec<String> {
    inbox
        .iter()
        .enumerate()
        .map(|(i, m)| format!("{}. {}", i + 1, m.subject))
        .collect()
}

// ---- 4. 최댓값 갱신 → max_by_key? 바꾸기 전에 동작을 확인합니다 ------------

// 더 클 때만(>) 바꾸므로 크기가 같으면 먼저 받은 메일이 남습니다.
fn largest_loop(inbox: &[Mail]) -> Option<&Mail> {
    let mut largest: Option<&Mail> = None;
    for mail in inbox {
        if largest.is_none_or(|best| mail.size_kb > best.size_kb) {
            largest = Some(mail);
        }
    }
    largest
}

// max_by_key는 같은 최댓값 중 마지막 원소를 돌려준다고 문서에 정해져 있습니다.
// 그래서 크기가 같은 메일이 있으면 반복문판과 결과가 다릅니다.
fn largest_max_by_key(inbox: &[Mail]) -> Option<&Mail> {
    inbox.iter().max_by_key(|m| m.size_kb)
}

// 반복문의 규칙(먼저 나온 것 유지)을 그대로 옮기려면 reduce에 같은 비교를 씁니다.
// 정해진 소비 메서드가 원하는 규칙과 맞지 않을 때 쓰는 것이 reduce와 fold입니다.
fn largest_reduce(inbox: &[Mail]) -> Option<&Mail> {
    inbox
        .iter()
        .reduce(|best, m| if m.size_kb > best.size_kb { m } else { best })
}

// ---- 5. 누적값이 둘 → fold ------------------------------------------------

fn unread_average_loop(inbox: &[Mail]) -> Option<u32> {
    let mut count = 0;
    let mut total = 0;
    for mail in inbox {
        if mail.unread {
            count += 1;
            total += mail.size_kb;
        }
    }
    total.checked_div(count) // 읽지 않은 메일이 없으면 None
}

// 개수와 합계를 튜플 하나로 들고 다니면 한 번 돌면서 둘을 함께 구합니다.
fn unread_average_fold(inbox: &[Mail]) -> Option<u32> {
    let (count, total) = inbox
        .iter()
        .filter(|m| m.unread)
        .fold((0, 0), |(n, sum), m| (n + 1, sum + m.size_kb));
    total.checked_div(count)
}

fn mail(subject: &'static str, size_kb: u32, unread: bool) -> Mail {
    Mail {
        subject,
        size_kb,
        unread,
    }
}

fn subject_of(m: Option<&Mail>) -> &'static str {
    m.map_or("없음", |m| m.subject)
}

fn main() {
    let inbox = vec![
        mail("회의록", 240, true),
        mail("견적서", 3100, false),
        mail("휴가 신청", 18, true),
        mail("설계 도면", 3100, true), // 견적서와 크기가 같습니다
        mail("뉴스레터", 95, false),
    ];

    println!("== 1. push → collect ==");
    let (a, b) = (unread_subjects_loop(&inbox), unread_subjects_chain(&inbox));
    println!("반복문판 {a:?}, 체인판 {b:?} (일치: {})", a == b);
    // 출력: 반복문판 ["회의록", "휴가 신청", "설계 도면"], 체인판 ["회의록", "휴가 신청", "설계 도면"] (일치: true)

    println!();
    println!("== 2. 카운터와 합계 → count, sum ==");
    println!(
        "읽지 않은 메일: 반복문판 {}통, 체인판 {}통",
        unread_count_loop(&inbox),
        unread_count_chain(&inbox)
    );
    println!(
        "전체 용량: fold판 {}KB, sum판 {}KB",
        total_kb_fold(&inbox),
        total_kb_sum(&inbox)
    );
    // 출력: 읽지 않은 메일: 반복문판 3통, 체인판 3통
    // 출력: 전체 용량: fold판 6553KB, sum판 6553KB

    println!();
    println!("== 3. 찾으면 return, 순번 변수 → find, enumerate ==");
    for limit in [1000, 5000] {
        println!(
            "{limit}KB보다 큰 첫 메일: 반복문판 {}, 체인판 {}",
            subject_of(first_larger_loop(&inbox, limit)),
            subject_of(first_larger_chain(&inbox, limit))
        );
    }
    let (a, b) = (numbered_loop(&inbox), numbered_chain(&inbox));
    println!("번호 붙이기: {:?} … (일치: {})", &b[..2], a == b);
    // 출력: 1000KB보다 큰 첫 메일: 반복문판 견적서, 체인판 견적서
    // 출력: 5000KB보다 큰 첫 메일: 반복문판 없음, 체인판 없음
    // 출력: 번호 붙이기: ["1. 회의록", "2. 견적서"] … (일치: true)

    println!();
    println!("== 4. 최댓값 갱신: 같은 값이 있을 때 ==");
    println!("반복문판(>로 갱신): {}", subject_of(largest_loop(&inbox)));
    println!("max_by_key판: {}", subject_of(largest_max_by_key(&inbox)));
    println!(
        "reduce판(같은 비교): {}",
        subject_of(largest_reduce(&inbox))
    );
    // 출력: 반복문판(>로 갱신): 견적서
    // 출력: max_by_key판: 설계 도면
    // 출력: reduce판(같은 비교): 견적서

    println!();
    println!("== 5. 누적값이 둘 → fold ==");
    let all_read = [mail("공지", 12, false)];
    for (label, mails) in [
        ("받은편지함", &inbox[..]),
        ("모두 읽은 편지함", &all_read[..]),
    ] {
        println!(
            "{label}의 읽지 않은 메일 평균: 반복문판 {:?}, fold판 {:?}",
            unread_average_loop(mails),
            unread_average_fold(mails)
        );
    }
    // 출력: 받은편지함의 읽지 않은 메일 평균: 반복문판 Some(1119), fold판 Some(1119)
    // 출력: 모두 읽은 편지함의 읽지 않은 메일 평균: 반복문판 None, fold판 None
}
