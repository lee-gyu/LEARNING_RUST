//! 지침 3: 여러 곳이 함께 갖는 큰 데이터는 라이프타임 대신 공유 소유로 나눈다.
//!
//! 실행: cargo run -p er-ch20-avoid-over-optimizing --example 03_share_instead_of_borrow
//!
//! 경매장의 출품 목록(카탈로그)을 응찰자 화면 여러 개가 함께 봅니다. 화면이 카탈로그를
//! 빌리는 빌린 판, 화면마다 통째로 복제하는 복제판, `Rc`로 나눠 가지는 `Rc`판을 같은
//! 응찰자들로 만들어 비교합니다.
//! 빌린 판과 `Rc`판은 둘 다 복사가 없지만, 라이프타임 매개변수가 있는 쪽은 하나뿐입니다.

use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIDDERS: [&str; 20] = [
    "민준", "서연", "도윤", "하은", "지호", "수아", "예준", "지유", "시우", "하린", "주원", "서윤",
    "건우", "채원", "우진", "지민", "선우", "다은", "현우", "소율",
];

#[derive(Clone)]
struct Lot {
    title: String,
    notes: String, // 감정서처럼 긴 설명
}

struct Catalog {
    lots: Vec<Lot>,
}

impl Catalog {
    /// 실제 프로그램이라면 파일이나 서버에서 읽어 옵니다. 40개 물품, 설명이 긴 큰 값입니다.
    fn load() -> Catalog {
        let lots = (1..=40)
            .map(|n| Lot {
                title: format!("{n}번 출품작"),
                notes: "보관 상태와 감정 의견. ".repeat(40),
            })
            .collect();
        Catalog { lots }
    }

    /// 힙에 든 글자의 바이트 수. 복제 한 번에 이만큼을 새로 할당해 옮겨 적습니다.
    fn text_bytes(&self) -> usize {
        self.lots
            .iter()
            .map(|l| l.title.len() + l.notes.len())
            .sum()
    }
}

// 카탈로그가 복제될 때마다 셉니다(지침 2). Rc::clone은 이 함수를 부르지 않습니다.
static CATALOG_CLONES: AtomicUsize = AtomicUsize::new(0);
static CLONED_BYTES: AtomicUsize = AtomicUsize::new(0);

impl Clone for Catalog {
    fn clone(&self) -> Catalog {
        CATALOG_CLONES.fetch_add(1, Ordering::Relaxed);
        CLONED_BYTES.fetch_add(self.text_bytes(), Ordering::Relaxed);
        Catalog {
            lots: self.lots.clone(),
        }
    }
}

fn take_clone_counts() -> (usize, usize) {
    (
        CATALOG_CLONES.swap(0, Ordering::Relaxed),
        CLONED_BYTES.swap(0, Ordering::Relaxed),
    )
}

/// 응찰자마다 관심 물품 두 개(카탈로그의 위치)를 정합니다.
fn watch_list(index: usize) -> Vec<usize> {
    vec![index % 40, (index * 7 + 3) % 40]
}

// ---- 빌린 판: 화면이 카탈로그를 빌립니다 ----

struct ScreenView<'a> {
    bidder: String,
    catalog: &'a Catalog,
    watching: Vec<usize>,
}

/// 화면을 담는 홀에도 'a가 번집니다. 이 홀을 담는 타입이 있다면 그 타입에도 번집니다.
struct HallView<'a> {
    screens: Vec<ScreenView<'a>>,
}

/// 카탈로그를 이 함수 안에서 불러올 수 없습니다. 지역 값을 빌린 홀은 돌려줄 수 없기 때문입니다(E0515).
/// 그래서 부르는 쪽이 카탈로그를 먼저 불러 홀보다 오래 들고 있어야 합니다.
fn open_hall_view(catalog: &Catalog) -> HallView<'_> {
    let screens = BIDDERS
        .iter()
        .enumerate()
        .map(|(i, name)| ScreenView {
            bidder: name.to_string(),
            catalog,
            watching: watch_list(i),
        })
        .collect();
    HallView { screens }
}

impl ScreenView<'_> {
    fn first_watched(&self) -> &str {
        &self.catalog.lots[self.watching[0]].title
    }
}

// ---- 복제판: 화면마다 카탈로그를 통째로 복제합니다 ----

struct ScreenCopy {
    catalog: Catalog,
    watching: Vec<usize>,
}

fn open_hall_copy() -> Vec<ScreenCopy> {
    let catalog = Catalog::load();
    (0..BIDDERS.len())
        .map(|i| ScreenCopy {
            catalog: catalog.clone(), // 화면 하나에 카탈로그 한 벌
            watching: watch_list(i),
        })
        .collect()
}

// ---- Rc판: 화면들이 카탈로그 하나를 나눠 가집니다 ----

struct Screen {
    bidder: String,
    catalog: Rc<Catalog>,
    watching: Vec<usize>,
}

/// 라이프타임 매개변수가 없습니다. 카탈로그를 안에서 불러와 홀째로 돌려줄 수 있습니다.
struct Hall {
    catalog: Rc<Catalog>,
    screens: Vec<Screen>,
}

fn open_hall() -> Hall {
    let catalog = Rc::new(Catalog::load());
    let screens = BIDDERS
        .iter()
        .enumerate()
        .map(|(i, name)| Screen {
            bidder: name.to_string(),
            catalog: Rc::clone(&catalog), // 카운트만 1 올라갑니다
            watching: watch_list(i),
        })
        .collect();
    Hall { catalog, screens }
}

impl Screen {
    fn first_watched(&self) -> &str {
        &self.catalog.lots[self.watching[0]].title
    }
}

fn main() {
    println!("1. 빌린 판: 복사는 없지만 카탈로그를 홀 밖에서 따로 들고 있어야 합니다");
    let catalog = Catalog::load(); // 홀보다 먼저 만들어 홀보다 오래 살려 둡니다
    let hall = open_hall_view(&catalog);
    let first = &hall.screens[0];
    println!(
        "    화면 {}개, {} 님의 첫 관심 물품: {}",
        hall.screens.len(),
        first.bidder,
        first.first_watched()
    ); // 출력:     화면 20개, 민준 님의 첫 관심 물품: 1번 출품작
    let (clones, bytes) = take_clone_counts();
    println!("    카탈로그 복제 {clones}번 {bytes}바이트"); // 출력:     카탈로그 복제 0번 0바이트

    println!("2. 복제판: 라이프타임은 없지만 화면마다 카탈로그를 한 벌씩 복사합니다");
    let copies = open_hall_copy();
    let last = &copies[copies.len() - 1];
    println!(
        "    화면 {}개, 마지막 화면의 첫 관심 물품: {}",
        copies.len(),
        last.catalog.lots[last.watching[0]].title
    ); // 출력:     화면 20개, 마지막 화면의 첫 관심 물품: 20번 출품작
    let (clones, bytes) = take_clone_counts();
    println!(
        "    카탈로그 복제 {clones}번 {bytes}바이트 (한 벌 {}바이트)",
        catalog.text_bytes()
    ); // 출력:     카탈로그 복제 20번 1035820바이트 (한 벌 51791바이트)

    println!("3. Rc판: 라이프타임도 복사도 없이 카탈로그 하나를 나눠 가집니다");
    let hall = open_hall(); // 카탈로그를 안에서 불러와 돌려받습니다
    let first = &hall.screens[0];
    println!(
        "    화면 {}개, {} 님의 첫 관심 물품: {}",
        hall.screens.len(),
        first.bidder,
        first.first_watched()
    ); // 출력:     화면 20개, 민준 님의 첫 관심 물품: 1번 출품작
    let all_same = hall
        .screens
        .iter()
        .all(|s| Rc::ptr_eq(&s.catalog, &hall.catalog));
    let (clones, bytes) = take_clone_counts();
    println!(
        "    카탈로그 복제 {clones}번 {bytes}바이트, 모두 같은 카탈로그 {all_same}, 카운트 {}",
        Rc::strong_count(&hall.catalog)
    ); // 출력:     카탈로그 복제 0번 0바이트, 모두 같은 카탈로그 true, 카운트 21

    // 경매가 끝나 화면을 닫으면 카운트가 1로 돌아옵니다. 마지막 소유자는 복사 없이 값을 꺼냅니다.
    let kept = Rc::clone(&hall.catalog);
    drop(hall);
    println!("    홀을 닫은 뒤 카운트 {}", Rc::strong_count(&kept)); // 출력:     홀을 닫은 뒤 카운트 1
    let archived: Catalog = Rc::unwrap_or_clone(kept); // 혼자 가진 Rc라면 복제하지 않고 꺼냅니다
    let (clones, _) = take_clone_counts();
    println!(
        "    보관용으로 꺼낸 카탈로그: 물품 {}개, 복제 {clones}번",
        archived.lots.len()
    ); // 출력:     보관용으로 꺼낸 카탈로그: 물품 40개, 복제 0번
}
