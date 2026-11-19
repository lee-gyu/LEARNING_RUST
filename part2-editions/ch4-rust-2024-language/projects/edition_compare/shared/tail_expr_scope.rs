//! 같은 코드를 2021 에디션과 2024 에디션으로 컴파일해서, 블록의 마지막 표현식(tail
//! expression)에서 만든 임시값과 그 블록의 지역 변수 중 무엇이 먼저 버려지는지 비교합니다.
//!
//! 실행(이 프로젝트 디렉터리에서):
//!   cargo run -q --bin tail_expr_scope_2021
//!   cargo run -q --bin tail_expr_scope_2024

/// 버려질 때 자기 이름을 출력하는 값입니다.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.0);
    }
}

impl Noisy {
    /// 이름의 글자 수를 돌려줍니다. 임시값을 빌려서 값을 하나 계산하는 메서드입니다.
    fn char_count(&self) -> usize {
        self.0.chars().count()
    }
}

/// 함수 본문의 마지막 표현식이 임시값을 만듭니다.
fn from_function() -> usize {
    let _local = Noisy("지역 변수");
    // `Noisy(..)`는 이 줄에서 만든 임시값이고, `char_count`가 그것을 빌립니다.
    Noisy("마지막 표현식의 임시값").char_count()
}

fn main() {
    println!("1. 함수 본문의 마지막 표현식");
    let n = from_function();
    println!("    돌려받은 값: {n}");
    // 출력(2021): [drop] 지역 변수 → [drop] 마지막 표현식의 임시값 → 돌려받은 값: 12
    // 출력(2024): [drop] 마지막 표현식의 임시값 → [drop] 지역 변수 → 돌려받은 값: 12

    println!("2. 블록의 마지막 표현식");
    let m = {
        let _local = Noisy("블록 안 지역 변수");
        Noisy("블록 마지막 표현식의 임시값").char_count()
    };
    println!("    블록의 값: {m}");
    // 출력(2021): [drop] 블록 안 지역 변수 → [drop] 블록 마지막 표현식의 임시값 → 블록의 값: 15
    // 출력(2024): [drop] 블록 마지막 표현식의 임시값 → [drop] 블록 안 지역 변수 → 블록의 값: 15

    println!("3. 마지막 표현식이 아닌 let 문 (두 에디션이 같음)");
    let k = Noisy("let 문의 임시값").char_count();
    println!("    let 문 다음: {k}");
    // 출력(두 에디션): [drop] let 문의 임시값 → let 문 다음: 10
}
