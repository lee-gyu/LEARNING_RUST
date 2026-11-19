// 기대하는 에러: E0624
//   error[E0624]: method `take_seat` is private
//
// 예제 01의 권장 판을 줄인 코드입니다. `take_seat`은 `pub(super)`라서 `cinema`와 그 아래에서만
// 보입니다. `main`(크레이트 루트)은 그 범위 밖이므로, `pub(super)`로 좁힌 메서드도
// 비공개 메서드와 똑같은 에러가 납니다.
//
// 확인: rustc --edition 2024 compile_fail/e0624_restricted_method.rs --out-dir /tmp/er-ch22

pub mod cinema {
    pub use booking::book;
    pub use hall::Cinema;

    mod hall {
        pub struct Cinema {
            taken: Vec<bool>,
        }

        impl Cinema {
            pub fn new(seats: usize) -> Cinema {
                Cinema {
                    taken: vec![false; seats],
                }
            }

            // 예매 절차(형제 모듈 booking)만 부르게 하려고 pub(super)로 열었습니다.
            pub(super) fn take_seat(&mut self, seat: usize) -> bool {
                match self.taken.get_mut(seat) {
                    Some(t) if !*t => {
                        *t = true;
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    mod booking {
        use super::hall::Cinema;

        pub fn book(hall: &mut Cinema, seat: usize, paid: u32) -> bool {
            paid >= 9000 && hall.take_seat(seat)
        }
    }
}

fn main() {
    let mut hall = cinema::Cinema::new(5);
    println!("예매: {}", cinema::book(&mut hall, 0, 9000));
    println!("직접 잡기: {}", hall.take_seat(4)); // E0624
}
