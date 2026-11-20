//! 설문 응답을 세어 많은 순으로 보여 주는 예제 프로그램입니다.
//!
//! 실행: cargo run --example survey            (기본 기능: CSV도 출력)
//!       cargo run --example survey --all-features  (JSON도 출력)

use pollcount::Votes;

const ANSWERS: &str = "사과, 배, 귤, 사과, 포도, 배, 사과, , 귤, 사과";

fn main() {
    // 에러 타입이 `Infallible`이라 `Err` 갈래가 없어도 됩니다(값이 없는 타입).
    let Ok(t) = ANSWERS.parse::<Votes>();
    println!("응답 {}개, 서로 다른 답 {}개", t.total(), t.len());
    for (rank, (item, count)) in t.top(3).into_iter().enumerate() {
        println!("{}위 {item}: {count}표", rank + 1);
    }
    #[cfg(feature = "csv")]
    print!("{}", t.to_csv());
    #[cfg(feature = "json")]
    println!("{}", t.to_json());
}
