//! 지침 1: 기본은 비공개로 두고, 열 때는 실제로 쓰는 곳까지만 연다.
//!
//! 같은 영화관 예매 모듈을 두 판으로 만듭니다.
//! - 비권장 `cinema_loose`: 모든 모듈과 항목이 `pub`입니다. 예매 절차 안에서만 써야 할
//!   `take_seat`을 `main`이 직접 불러, 표도 매출도 없이 좌석이 팔린 상태가 됩니다.
//! - 권장 `cinema`: 항목마다 그것을 쓰는 코드가 있는 범위까지만 엽니다.
//!
//! 권장 판의 모듈 트리와 가시성
//!   crate
//!    ├── audit
//!    │    └── note                    pub(crate)             main과 cinema::booking이 씀
//!    └── cinema                       pub mod. pub use로 Cinema, Ticket, book만 내보냄
//!         ├── hall
//!         │    ├── Cinema             pub                    new, report: pub
//!         │    └── take_seat 등       pub(super)             형제 모듈 booking이 씀
//!         ├── booking
//!         │    └── book, Ticket       pub
//!         └── pricing
//!              └── rates              pub(super) mod
//!                   └── base_price    pub(in crate::cinema)  두 단계 위의 cinema 아래에서 씀
//!
//! `main`에서 권장 판의 `take_seat`을 부르면 E0624입니다(compile_fail/e0624_restricted_method.rs).
//!
//! 실행: cargo run -p er-ch22-minimize-visibility --example 01_visibility_levels

mod audit {
    /// 크레이트 어디서나 쓰는 기록 도우미입니다. 다른 크레이트에 내놓을 기능은 아니므로
    /// `pub`이 아니라 `pub(crate)`로 엽니다.
    pub(crate) fn note(at: &str, msg: &str) {
        println!("    [{at}] {msg}");
    }
}

// 두 판 모두 크레이트 밖에 내놓는 라이브러리 API 역할이므로 모듈 자체는 pub mod로 둡니다.

// ---- 비권장: 모든 모듈과 항목에 pub ----

pub mod cinema_loose {
    pub mod hall {
        pub struct Cinema {
            pub taken: Vec<bool>,
            pub tickets: u32,
            pub revenue: u32,
        }

        impl Cinema {
            pub fn new(seats: usize) -> Cinema {
                Cinema {
                    taken: vec![false; seats],
                    tickets: 0,
                    revenue: 0,
                }
            }

            // 예매 절차의 한 단계일 뿐인데 pub이라서 누구나 부를 수 있습니다.
            pub fn take_seat(&mut self, seat: usize) -> Result<(), String> {
                match self.taken.get_mut(seat) {
                    None => Err(format!("좌석 {seat}은 없습니다")),
                    Some(true) => Err(format!("좌석 {seat}은 이미 팔렸습니다")),
                    Some(t) => {
                        *t = true;
                        Ok(())
                    }
                }
            }

            pub fn record_sale(&mut self, price: u32) {
                self.tickets += 1;
                self.revenue += price;
            }

            pub fn report(&self) -> String {
                let sold = self.taken.iter().filter(|&&t| t).count();
                format!(
                    "팔린 좌석 {sold}석, 발급한 표 {}장, 매출 {}원",
                    self.tickets, self.revenue
                )
            }
        }
    }

    pub mod booking {
        use super::hall::Cinema;
        use super::pricing::rates;

        pub struct Ticket {
            pub seat: usize,
            pub price: u32,
        }

        pub fn book(hall: &mut Cinema, seat: usize, paid: u32) -> Result<Ticket, String> {
            let price = rates::base_price(seat);
            if paid < price {
                return Err(format!("{price}원이 필요합니다"));
            }
            hall.take_seat(seat)?;
            hall.record_sale(price);
            Ok(Ticket { seat, price })
        }
    }

    pub mod pricing {
        pub mod rates {
            pub fn base_price(seat: usize) -> u32 {
                if seat < 3 { 9000 } else { 12000 }
            }
        }
    }
}

// ---- 권장: 쓰는 곳까지만 연다 ----

pub mod cinema {
    // 바깥에 보여 줄 이름만 cinema 바로 아래로 올립니다. 하위 모듈은 모두 비공개입니다.
    pub use booking::{Ticket, book};
    pub use hall::Cinema;

    mod hall {
        pub struct Cinema {
            taken: Vec<bool>,
            tickets: u32,
            revenue: u32,
        }

        impl Cinema {
            pub fn new(seats: usize) -> Cinema {
                Cinema {
                    taken: vec![false; seats],
                    tickets: 0,
                    revenue: 0,
                }
            }

            // pub(super): hall의 부모인 cinema와 그 아래(booking 포함)에서만 보입니다.
            // 비공개였다면 booking이 부를 수 없고, pub이었다면 main도 부를 수 있습니다.
            pub(super) fn take_seat(&mut self, seat: usize) -> Result<(), String> {
                match self.taken.get_mut(seat) {
                    None => Err(format!("좌석 {seat}은 없습니다")),
                    Some(true) => Err(format!("좌석 {seat}은 이미 팔렸습니다")),
                    Some(t) => {
                        *t = true;
                        Ok(())
                    }
                }
            }

            pub(super) fn record_sale(&mut self, price: u32) {
                self.tickets += 1;
                self.revenue += price;
            }

            pub fn report(&self) -> String {
                let sold = self.taken.iter().filter(|&&t| t).count();
                format!(
                    "팔린 좌석 {sold}석, 발급한 표 {}장, 매출 {}원",
                    self.tickets, self.revenue
                )
            }
        }
    }

    mod booking {
        use super::hall::Cinema;
        use super::pricing::rates;

        pub struct Ticket {
            seat: usize,
            price: u32,
        }

        impl Ticket {
            pub fn seat(&self) -> usize {
                self.seat
            }

            pub fn price(&self) -> u32 {
                self.price
            }
        }

        // 좌석을 잡고 매출을 기록하는 길은 이 함수 하나뿐입니다.
        pub fn book(hall: &mut Cinema, seat: usize, paid: u32) -> Result<Ticket, String> {
            let price = rates::base_price(seat);
            if paid < price {
                return Err(format!("{price}원이 필요합니다"));
            }
            hall.take_seat(seat)?;
            hall.record_sale(price);
            crate::audit::note(module_path!(), &format!("좌석 {seat} 판매 기록"));
            Ok(Ticket { seat, price })
        }
    }

    mod pricing {
        // rates 모듈 자체도 cinema 아래에서 보여야 booking이 경로를 따라 들어올 수 있습니다.
        pub(super) mod rates {
            // pub(super)라면 pricing 안에서만 보여 booking이 쓸 수 없습니다.
            // 두 단계 위(cinema)까지 열어야 하므로 pub(in crate::cinema)입니다.
            pub(in crate::cinema) fn base_price(seat: usize) -> u32 {
                if seat < 3 { 9000 } else { 12000 }
            }
        }
    }
}

fn main() {
    println!("1. 비권장: 모든 모듈과 항목이 pub");
    let mut hall = cinema_loose::hall::Cinema::new(5);
    match cinema_loose::booking::book(&mut hall, 0, 9000) {
        Ok(t) => println!("    좌석 {} 예매: 표 발급 ({}원)", t.seat, t.price),
        Err(e) => println!("    예매 실패: {e}"),
    }
    // 결제도 표 발급도 없이 좌석만 잡습니다. 컴파일러는 막지 않습니다.
    let bypass = hall.take_seat(4);
    println!("    main이 take_seat(4)를 직접 부름: {bypass:?}");
    println!("    {}", hall.report()); // 출력: 팔린 좌석 2석, 발급한 표 1장, 매출 9000원
    // 내부 요금표에도 바깥 코드가 기댈 수 있게 됩니다.
    let price = cinema_loose::pricing::rates::base_price(4);
    println!("    main이 내부 요금표를 직접 읽음: base_price(4) = {price}원");

    println!("2. 권장: 쓰는 곳까지만 연다");
    let mut hall = cinema::Cinema::new(5);
    for (seat, paid) in [(0, 9000), (4, 10000), (4, 12000)] {
        match cinema::book(&mut hall, seat, paid) {
            Ok(t) => println!("    좌석 {} 예매: 표 발급 ({}원)", t.seat(), t.price()),
            Err(e) => println!("    좌석 {seat} 예매({paid}원 냄): 거절 - {e}"),
        }
    }
    // hall.take_seat(4); // E0624: method `take_seat` is private
    // cinema::pricing::rates::base_price(4); // E0603: module `pricing` is private
    println!("    {}", hall.report()); // 출력: 팔린 좌석 2석, 발급한 표 2장, 매출 21000원

    println!("3. pub(crate) 도우미는 크레이트 어디서나 부를 수 있다");
    audit::note(module_path!(), "main에서 남긴 기록");
}
