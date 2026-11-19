// 기대하는 에러: E0277
//   error[E0277]: the trait bound `Bin: Copy` is not satisfied
//   = help: consider using `core::array::from_fn` to initialize the array
//
// `[값; N]` 배열 식은 값 하나를 N번 복사해 채우므로, 원소 타입이 Copy이거나 값이 상수여야 합니다.
// Bin은 String과 Vec을 가지므로 Copy가 아닙니다. 이 에러를 만나면 MaybeUninit으로 칸을 하나씩
// 채우고 싶어지지만, 컴파일러의 help가 이미 안전한 길(array::from_fn)을 알려 줍니다(예제 03).
//
// 확인: rustc --edition 2024 compile_fail/e0277_array_repeat_non_copy.rs --out-dir /tmp/er-ch16

struct Bin {
    label: String,
    letters: Vec<&'static str>,
}

impl Bin {
    fn empty() -> Bin {
        Bin {
            label: String::from("미정"),
            letters: Vec::new(),
        }
    }
}

fn main() {
    let bins: [Bin; 10] = [Bin::empty(); 10]; // E0277
    println!(
        "{}칸, 첫 칸 {} ({}통)",
        bins.len(),
        bins[0].label,
        bins[0].letters.len()
    );
}
