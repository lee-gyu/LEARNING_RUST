//! 지침 2: 눈에 보이는 복사를 지우려 하지 말고, 먼저 센다.
//!
//! 실행: cargo run -p er-ch20-avoid-over-optimizing --example 02_count_the_copies
//!
//! 1번 구역은 낙찰 알림을 만들 때 문자열 복제를 피하려고 판매 기록을 빌린 판과, 그냥 `clone()`한
//! 판을 비교합니다. 2번 구역은 타입마다 `clone()` 한 번이 실제로 하는 일(새 버퍼를 할당하는지,
//! 카운트만 올리는지)을 확인합니다. 복사의 값은 "몇 번 × 한 번에 얼마"로 셀 수 있습니다.

use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

/// 끝난 경매 하나. 경매 기록을 정리할 때 함께 버려집니다.
struct Sale {
    item: String,
    winner: String,
    amount: u32,
}

fn closed_sales() -> Vec<Sale> {
    [
        ("자전거", "하은", 182_000),
        ("책상", "지호", 42_000),
        ("카메라", "도윤", 310_000),
    ]
    .into_iter()
    .map(|(item, winner, amount)| Sale {
        item: item.to_string(),
        winner: winner.to_string(),
        amount,
    })
    .collect()
}

// ---- 비권장: 복제를 피하려고 판매 기록을 빌립니다 ----

struct NoticeView<'a> {
    item: &'a str,
    winner: &'a str,
    amount: u32,
}

/// 결과가 `sales`에 묶입니다. 알림을 보내기 전까지 판매 기록을 버릴 수 없습니다.
fn notices_view(sales: &[Sale]) -> Vec<NoticeView<'_>> {
    sales
        .iter()
        .map(|s| NoticeView {
            item: &s.item,
            winner: &s.winner,
            amount: s.amount,
        })
        .collect()
}

// ---- 권장: 알림이 자기 글자를 가집니다 ----

struct Notice {
    item: String,
    winner: String,
    amount: u32,
}

/// 알림 하나에 `clone()`이 두 번 보입니다. 비용은 코드에 적힌 그대로입니다.
fn notices(sales: &[Sale]) -> Vec<Notice> {
    sales
        .iter()
        .map(|s| Notice {
            item: s.item.clone(),
            winner: s.winner.clone(),
            amount: s.amount,
        })
        .collect()
}

fn render(item: &str, winner: &str, amount: u32) -> String {
    format!("{item}→{winner} {amount}원")
}

fn main() {
    println!("1. 낙찰 알림: 복제를 피한 빌린 판과 그냥 복제하는 복제판");
    let sales = closed_sales();

    let viewed: Vec<String> = notices_view(&sales)
        .iter()
        .map(|n| render(n.item, n.winner, n.amount))
        .collect();
    println!("    빌린 판: 알림 {}건, clone() 0번", viewed.len()); // 출력:     빌린 판: 알림 3건, clone() 0번

    let outbox = notices(&sales);
    // 알림 하나에 clone()이 두 번이므로 횟수는 알림 수의 두 배, 바이트는 복제한 두 필드의 길이 합입니다.
    let clones = outbox.len() * 2;
    let bytes: usize = outbox.iter().map(|n| n.item.len() + n.winner.len()).sum();
    println!(
        "    복제판: 알림 {}건, clone() {clones}번, 복사한 글자 {bytes}바이트",
        outbox.len()
    ); // 출력:     복제판: 알림 3건, clone() 6번, 복사한 글자 42바이트

    // 경매 기록을 정리합니다. 빌린 판의 알림이 아직 남아 있었다면 이 줄은 E0505입니다.
    drop(sales);
    let sent: Vec<String> = outbox
        .iter()
        .map(|n| render(&n.item, &n.winner, n.amount))
        .collect();
    println!("    판매 기록을 정리한 뒤 발송: {}", sent.join(", ")); // 출력:     판매 기록을 정리한 뒤 발송: 자전거→하은 182000원, 책상→지호 42000원, 카메라→도윤 310000원
    println!("    두 판의 알림이 같은가: {}", viewed == sent); // 출력:     두 판의 알림이 같은가: true

    println!("2. clone() 한 번의 값은 타입이 정합니다");
    // Copy 타입과 참조: 값의 비트를 그대로 복사합니다. 참조는 가리키는 곳(과 길이)만 복사하므로
    // 글자는 원래 자리에 그대로 있습니다.
    let name: &str = "자전거 경매";
    let name_copy = name;
    println!("    u32: 비트 복사 {}바이트", size_of::<u32>()); // 출력:     u32: 비트 복사 4바이트
    println!(
        "    &str: 비트 복사 {}바이트(참조), 글자가 같은 곳 {}",
        size_of::<&str>(),
        ptr::eq(name.as_ptr(), name_copy.as_ptr())
    ); // 출력:     &str: 비트 복사 16바이트(참조), 글자가 같은 곳 true

    // String, Box<T>, Vec<T>: 새 버퍼를 할당하고 내용을 모두 복사합니다. 그래서 가리키는 곳이 다릅니다.
    let title = String::from("자전거 경매");
    let title_clone = title.clone();
    println!(
        "    String: 새 버퍼에 {}바이트 복사, 같은 곳 {}",
        title_clone.len(),
        ptr::eq(title.as_ptr(), title_clone.as_ptr())
    ); // 출력:     String: 새 버퍼에 16바이트 복사, 같은 곳 false

    let boxed: Box<str> = Box::from("자전거 경매");
    let boxed_clone = boxed.clone();
    println!(
        "    Box<str>: 새 버퍼에 {}바이트 복사, 같은 곳 {}",
        boxed_clone.len(),
        ptr::eq(boxed.as_ptr(), boxed_clone.as_ptr())
    ); // 출력:     Box<str>: 새 버퍼에 16바이트 복사, 같은 곳 false

    let amounts: Vec<u32> = (0..1000).collect();
    let amounts_clone = amounts.clone();
    println!(
        "    Vec<u32> 1000개: 새 버퍼에 {}바이트 복사, 같은 곳 {}",
        size_of_val(amounts_clone.as_slice()),
        ptr::eq(amounts.as_ptr(), amounts_clone.as_ptr())
    ); // 출력:     Vec<u32> 1000개: 새 버퍼에 4000바이트 복사, 같은 곳 false

    // 원소도 할당을 가진 타입이면 원소마다 다시 복제합니다(깊은 복제).
    let bidders: Vec<String> = ["민준", "서연", "도윤"].map(String::from).to_vec();
    let bidders_clone = bidders.clone();
    let shared_any = bidders
        .iter()
        .zip(&bidders_clone)
        .any(|(a, b)| ptr::eq(a.as_ptr(), b.as_ptr()));
    println!(
        "    Vec<String> 3개: 새 버퍼 {}개(목록 1 + 원소 {}), 원소가 같은 곳 {shared_any}",
        1 + bidders_clone.len(),
        bidders_clone.len()
    ); // 출력:     Vec<String> 3개: 새 버퍼 4개(목록 1 + 원소 3), 원소가 같은 곳 false

    // Rc, Arc: 새 버퍼 없이 카운트만 1 올라갑니다. Arc는 스레드 사이에서 안전하게 원자적으로 올립니다.
    let shared: Rc<str> = Rc::from("자전거 경매");
    let before = Rc::strong_count(&shared);
    let shared_clone = Rc::clone(&shared);
    println!(
        "    Rc<str>: 같은 곳 {}, 카운트 {before} → {}",
        Rc::ptr_eq(&shared, &shared_clone),
        Rc::strong_count(&shared)
    ); // 출력:     Rc<str>: 같은 곳 true, 카운트 1 → 2

    let sync_shared: Arc<str> = Arc::from("자전거 경매");
    let before = Arc::strong_count(&sync_shared);
    let sync_clone = Arc::clone(&sync_shared);
    println!(
        "    Arc<str>: 같은 곳 {}, 카운트 {before} → {}",
        Arc::ptr_eq(&sync_shared, &sync_clone),
        Arc::strong_count(&sync_shared)
    ); // 출력:     Arc<str>: 같은 곳 true, 카운트 1 → 2
}
