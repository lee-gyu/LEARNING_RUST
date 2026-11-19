// 기대하는 에러: E0742
//   error[E0742]: visibilities can only be restricted to ancestor modules
//
// 요금 도우미 `seat_rate`를 좌석 모듈 `seats`에서만 쓰게 하려고 `pub(in crate::cinema::seats)`를
// 붙였습니다. 하지만 `pub(in 경로)`의 경로는 그 항목을 품은 조상 모듈이어야 합니다.
// `seats`는 `pricing::rates`의 조상이 아니라 형제 쪽 가지입니다.
// 고치려면 두 모듈의 가장 가까운 공통 조상인 `crate::cinema`를 적습니다.
//
// 확인: rustc --edition 2024 compile_fail/e0742_pub_in_not_ancestor.rs --out-dir /tmp/er-ch22

pub mod cinema {
    mod seats {
        pub(super) fn price_of(seat: usize) -> u32 {
            super::pricing::rates::seat_rate(seat)
        }
    }

    mod pricing {
        pub(super) mod rates {
            // E0742: crate::cinema::seats는 이 모듈의 조상이 아닙니다.
            pub(in crate::cinema::seats) fn seat_rate(seat: usize) -> u32 {
                if seat < 3 { 9000 } else { 12000 }
            }
        }
    }

    pub fn quote(seat: usize) -> u32 {
        seats::price_of(seat)
    }
}

fn main() {
    println!("좌석 4: {}원", cinema::quote(4));
}
