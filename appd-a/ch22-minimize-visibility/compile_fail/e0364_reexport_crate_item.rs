// 기대하는 에러: E0364
//   error[E0364]: `total_of` is only public within the crate, and cannot be re-exported outside
//
// 크레이트 안에서만 쓰려고 `pub(crate)`로 연 도우미를, 나중에 `pub use`로 공개 경로에
// 올리려 했습니다. 재내보내기는 원래 항목보다 넓게 열 수 없습니다. 공개 범위는 항목을
// 선언한 곳에서 정하고, `pub use`는 이미 열린 항목의 경로를 하나 더 만들 뿐입니다.
//
// 확인: rustc --edition 2024 compile_fail/e0364_reexport_crate_item.rs --out-dir /tmp/er-ch22

pub mod order {
    pub use cart::Cart;
    pub use cart::total_of; // E0364

    mod cart {
        pub struct Cart {
            prices: Vec<u32>,
        }

        impl Cart {
            pub fn new(prices: &[u32]) -> Cart {
                Cart {
                    prices: prices.to_vec(),
                }
            }

            pub fn total(&self) -> u32 {
                total_of(&self.prices)
            }
        }

        pub(crate) fn total_of(prices: &[u32]) -> u32 {
            prices.iter().sum()
        }
    }
}

fn main() {
    let cart = order::Cart::new(&[59000, 21000]);
    println!("합계 {}원", cart.total());
    println!("합계 {}원", order::total_of(&[59000, 21000]));
}
